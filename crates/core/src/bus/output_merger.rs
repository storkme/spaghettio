//! Final-product output merger.
//!
//! After ghost routing places all trunk / tap / return belts, this
//! module merges the east-flowing output belts of rows producing the
//! same final product into a single south-facing splitter chain at
//! the bottom-right of the layout. Called once per final product at
//! the end of `route_bus_ghost` (Step 7).

use rustc_hash::FxHashSet;

use crate::bus::balancer::splitter_for_belt;
use crate::bus::placer::RowSpan;
use crate::bus::stacking_ctx::StackingCtx;
use crate::models::{EntityDirection, PlacedEntity};

/// Capacity-aware contiguous partition of merger columns (#727 unit 2,
/// #728 round 1): greedy first-fit over per-column rates — optimal for
/// the minimum contiguous group count (codex-verified) — sizing and
/// assignment from ONE walk so the two can never disagree. A column
/// above `single_cap` gets its own group (the per-row output ceiling is
/// the placer's domain). Returns `(n_output, group_of)` with `group_of`
/// non-decreasing and every group `0..n_output` non-empty.
pub(crate) fn partition_columns(
    col_rates: &[f64],
    single_cap: f64,
    total_rate: f64,
) -> (usize, Vec<usize>) {
    if single_cap >= total_rate || col_rates.is_empty() {
        return (1, vec![0; col_rates.len()]);
    }
    let mut group_of: Vec<usize> = Vec::with_capacity(col_rates.len());
    let mut g = 0usize;
    let mut acc = 0.0_f64;
    for &r in col_rates {
        if acc > 0.0 && acc + r > single_cap + 1e-9 {
            g += 1;
            acc = r;
        } else {
            acc += r;
        }
        group_of.push(g);
    }
    (g + 1, group_of)
}

pub(crate) fn merge_output_rows(
    output_rows: &[usize],
    output_ys: &[i32],
    item: &str,
    row_spans: &[RowSpan],
    merge_start_y: i32,
    max_belt_tier: Option<&str>,
    min_merge_x: i32,
    blocked_columns: &[i32],
    ctx: &StackingCtx,
    existing_tiles: &FxHashSet<(i32, i32)>,
    row_tile_overrides: &mut FxHashSet<(i32, i32)>,
) -> (Vec<PlacedEntity>, Vec<PlacedEntity>, i32, i32) {
    use crate::bus::balancer::underground_for_belt;
    use crate::common::{belt_entity_for_rate_stacked, belt_throughput_stacked, ug_max_reach};

    debug_assert_eq!(
        output_rows.len(),
        output_ys.len(),
        "merge_output_rows: output_rows and output_ys must be 1:1 — output_ys[idx] is the \
         belt row for output_rows[idx] (RFC Fulgora D2a/D2b: primary rows use \
         `output_belt_y`, secondary-output rows use `secondary_output_belt`'s y)"
    );
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let n = output_rows.len();
    if n == 0 {
        return (entities, Vec::new(), merge_start_y, 0);
    }
    let merger_seg_id = Some(format!("merger:{}", item));

    let total_rate = output_rows
        .iter()
        .map(|&ri| {
            if ri >= row_spans.len() {
                0.0
            } else {
                let rs = &row_spans[ri];
                rs.spec
                    .outputs
                    .iter()
                    .filter(|o| o.item == item)
                    .map(|o| {
                        o.rate
                            * rs.machine_count as f64
                            * crate::common::utilization_for(&rs.spec)
                    })
                    .sum::<f64>()
            }
        })
        .sum::<f64>();
    let belt_name =
        belt_entity_for_rate_stacked(total_rate * 2.0, max_belt_tier, ctx.for_item(item));
    // #567: how many PARALLEL output belts the merged stream needs. The
    // cascade divides the N row-columns into `n_output` contiguous groups,
    // each folded to ONE output belt (a per-group N/M->1 merge). n_output == 1
    // for every under-cap fixture -> byte-identical to the pre-#567 path.
    // Stack-aware via the same per-item stack `ctx.for_item` the pick uses.
    let single_cap = belt_throughput_stacked(belt_name, ctx.for_item(item));
    // #727/RFC-072 Phase 1 unit 2: the group partition below is
    // CONTIGUOUS, so the belt count must come from a contiguous
    // capacity packing, not from ceil(total/cap) alone — 3 rows x 30/s
    // into ceil(90/45)=2 count-based groups put 60/s on a 45/s tail
    // (sim-anchored: cable-90 delivered 74.4/90 with the deficit equal
    // to the over-subscription). Greedy first-fit over contiguous
    // per-column rates is optimal for the minimum group count.
    let column_rate = |&ri: &usize| -> f64 {
        if ri >= row_spans.len() {
            0.0
        } else {
            let rs = &row_spans[ri];
            rs.spec
                .outputs
                .iter()
                .filter(|o| o.item == item)
                .map(|o| {
                    o.rate
                        * rs.machine_count as f64
                        * crate::common::utilization_for(&rs.spec)
                })
                .sum::<f64>()
        }
    };
    // COLUMN ORDER: the east extensions place row 0 at the RIGHTMOST
    // column (x = merge_x + n-1) and row n-1 at merge_x — the committed
    // geometry is the receipt (#728 round 1: the un-reversed form read
    // the wrong row's rate per column; symmetric specimen rates masked
    // it). col_rates[i] is the rate of the column at merge_x + i.
    let col_rates: Vec<f64> = output_rows.iter().rev().map(column_rate).collect();
    let (n_output, group_of) = partition_columns(&col_rates, single_cap, total_rate);
    // Hops may need more reach than the rate-picked tier offers
    // (alternating blocked columns with 1-tile gaps are unhoppable
    // at yellow reach and split into exit-abuts-next-entrance
    // pairs — the USP mega merger forensics). The hop TIER may
    // escalate up to the USER's belt cap: the cap is the
    // constraint, the rate pick is not; hop mouths are plumbing.
    // Function-scoped (not per-row): `hop_cap` depends only on
    // `max_belt_tier`, so every row in this merge shares one reach
    // budget — including the #309 headroom pre-pass below, which
    // needs the SAME reach the placement loop will end up using to
    // predict where each row's hop actually lands.
    let hop_cap: &str = max_belt_tier.unwrap_or("express-transport-belt");
    let reach = ug_max_reach(hop_cap) as i32;
    // Tier floor = the RATE-PICKED surface tier (#421 review: a
    // smallest-spanning pick could throttle an express-rate line
    // through a yellow hop — silently, since the throughput check
    // only flags overlapping routes); ceiling = the user's cap.
    let hop_tier_for_gap = |gap: i32| -> &'static str {
        for t in [
            "transport-belt",
            "fast-transport-belt",
            "express-transport-belt",
        ] {
            if ug_max_reach(t) as i32 >= gap && ug_max_reach(t) >= ug_max_reach(belt_name) {
                if ug_max_reach(t) <= ug_max_reach(hop_cap) {
                    return underground_for_belt(t);
                }
                break;
            }
        }
        underground_for_belt(hop_cap)
    };
    let splitter_name = splitter_for_belt(belt_name);

    // Column position: east of the widest participating row, but never west
    // of `min_merge_x` — the caller threads a running cursor across
    // successive per-item merges so two output items' splitter cascades and
    // south columns tile left-to-right instead of stamping the same tiles
    // (multi-item solid output support, Phase 2 of rfc-solver-net-flow).
    let mut merge_x = (output_rows
        .iter()
        .map(|&ri| {
            if ri < row_spans.len() {
                row_spans[ri].row_width
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
        + 1)
    .max(min_merge_x);

    // #309 review finding: a row whose east extension starts blocked (its
    // own `row_width` tile occupied by Step 4-6 residue — a dual-fate
    // item's `ret` belt) needs the underground bridge's EXIT tile placed
    // before the south column starts. The formula above doesn't know
    // about that yet, so a tight `min_merge_x` (e.g. this is the only /
    // first item processed) can put `col_x` exactly where the exit would
    // land — the exit and the south column's own top tile would then
    // double-place on the same coordinate. Mirror the main loop's
    // blocked-run walk here (unbounded by `col_x`, which isn't known
    // yet — bounded only by `reach`, same as the walk will end up doing
    // once `col_x` is resolved, so they agree on where each run ends)
    // and widen `merge_x` just enough that every row's exit — bridged or
    // not — lands strictly before its own column. A no-op (same `merge_x`
    // as before) for every row whose `row_width` tile isn't blocked,
    // which is every layout except this one.
    for (idx, &ri) in output_rows.iter().enumerate() {
        if ri >= row_spans.len() {
            continue;
        }
        let out_y = output_ys[idx];
        let rw = row_spans[ri].row_width;
        if !(blocked_columns.contains(&rw) || existing_tiles.contains(&(rw, out_y))) {
            continue;
        }
        // Mirrors the placement loop's clustering walk below (single-gap
        // runs joined across a free tile) so this predicts the SAME
        // `run_end` the real walk will reach once `col_x` is resolved —
        // an under-estimate here would silently reopen the exit/south-
        // column collision for a clustered blocked pattern.
        let mut run_end = rw;
        loop {
            let next_blocked = (blocked_columns.contains(&(run_end + 1))
                || existing_tiles.contains(&(run_end + 1, out_y)))
                && (run_end + 1) - rw < reach;
            if next_blocked {
                run_end += 1;
                continue;
            }
            let gap_then_blocked = !(blocked_columns.contains(&(run_end + 1))
                || existing_tiles.contains(&(run_end + 1, out_y)))
                && (blocked_columns.contains(&(run_end + 2))
                    || existing_tiles.contains(&(run_end + 2, out_y)))
                && (run_end + 2) - rw < reach;
            if gap_then_blocked {
                run_end += 2;
                continue;
            }
            break;
        }
        // This row's column sits at `merge_x + (n - 1 - idx)`; need that
        // to be `>= run_end + 2` (strictly past the exit at `run_end + 1`).
        let needed = run_end + 2 - (n as i32 - 1 - idx as i32);
        merge_x = merge_x.max(needed);
    }

    for (idx, &ri) in output_rows.iter().enumerate() {
        if ri >= row_spans.len() {
            continue;
        }
        let out_y = output_ys[idx];
        let col_x = merge_x + (n - 1 - idx) as i32; // first row rightmost, last row at merge_x

        // Extend EAST belts from the row's rightmost tile to the merge
        // column. Earlier items' south columns (`blocked_columns`) AND
        // tiles Step 4-6 (ghost-routed lanes) already claimed there
        // (`existing_tiles` — #309: a DUAL-FATE byproduct, produced by a
        // row and BOTH partly consumed internally AND registered surplus,
        // `docs/rfc-fulgora-scrap.md` 2026-07-11 decision log: stone/ice
        // on the scrap-recycling sushi row, gets an ordinary
        // intermediate-lane `ret` belt walking the consumed portion back
        // to its bus trunk from this exact row-exit tile) lie in this
        // run's path — bridge each contiguous blocked range with an
        // underground pair instead of stamping over (or sideloading
        // into) the foreign belt.
        let rw = row_spans[ri].row_width;
        let mut x = rw;
        while x < col_x {
            if blocked_columns.contains(&x) || existing_tiles.contains(&(x, out_y)) {
                // Contiguous blocked run [x, run_end], clamped by UG reach
                // (entrance at x-1, exit at run_end+1; gap ≤ reach).
                // CLUSTER runs separated by a single free tile: hopping
                // them independently would put run B's entrance exactly
                // on run A's exit — the mutation below then destroys
                // A's pair (two consecutive entrances; the game leaves
                // the first unpaired). Same defect class as the ghost
                // router's fluid-branch bridging (#412/USP forensics).
                // ALSO clamped by `existing_tiles` (#309: Step 4-6
                // residue — a dual-fate item's `ret` belt) alongside
                // `blocked_columns`, everywhere the latter is checked.
                let mut run_end = x;
                loop {
                    let next_blocked = run_end + 1 < col_x
                        && (blocked_columns.contains(&(run_end + 1))
                            || existing_tiles.contains(&(run_end + 1, out_y)))
                        && (run_end + 1) - x < reach;
                    if next_blocked {
                        run_end += 1;
                        continue;
                    }
                    let gap_then_blocked = run_end + 2 < col_x
                        && !(blocked_columns.contains(&(run_end + 1))
                            || existing_tiles.contains(&(run_end + 1, out_y)))
                        && (blocked_columns.contains(&(run_end + 2))
                            || existing_tiles.contains(&(run_end + 2, out_y)))
                        && (run_end + 2) - x < reach;
                    if gap_then_blocked {
                        run_end += 2;
                        continue;
                    }
                    break;
                }
                let gap = run_end + 1 - x;
                let hop_ug = hop_tier_for_gap(gap);
                if x == rw {
                    // No local tile to convert — the row's own last belt
                    // tile at (x-1, out_y) is placed by `place_rows` long
                    // before this function runs, and lives in the
                    // CALLER's `row_entities`: an immutable slice this
                    // module never sees, merged with the routed bus
                    // entities only after `route_bus_ghost` returns
                    // (`bus/layout.rs`). Push a FRESH entrance there
                    // instead and record the override — `layout.rs`
                    // already drops a `row_entities` tile whenever a
                    // routed bus entity claims the same coordinate (the
                    // existing splitter-eviction mechanism, generalized
                    // by #309 to any override reported here), so the
                    // row's now-stale plain belt at (x-1, out_y) is
                    // filtered out of the final layout rather than
                    // double-placed alongside this entrance. Distinct
                    // from the "refuse" branch below: this tile is KNOWN
                    // to be the row's own belt, not an ambiguous miss.
                    row_tile_overrides.insert((x - 1, out_y));
                    entities.push(PlacedEntity {
                        name: hop_ug.to_string(),
                        x: x - 1,
                        y: out_y,
                        direction: EntityDirection::East,
                        io_type: Some("input".to_string()),
                        carries: Some(item.to_string()),
                        segment_id: merger_seg_id.clone(),
                        rate: Some(total_rate),
                        ..Default::default()
                    });
                } else {
                    // Replace the belt stamped at x-1 with a UG entrance —
                    // only ever a plain surface belt of this run (the
                    // clustering guarantees it; the guard refuses to
                    // corrupt an existing mouth).
                    let converted = entities
                        .iter_mut()
                        .rev()
                        .find(|e| {
                            e.x == x - 1 && e.y == out_y && e.name.ends_with("transport-belt")
                        })
                        .map(|prev| {
                            prev.name = hop_ug.to_string();
                            prev.io_type = Some("input".to_string());
                        })
                        .is_some();
                    // No panic on a miss: fulgora's merger hits runs whose
                    // west tile is row machinery — the old code mutated
                    // whatever sat there (silent corruption); refusing the
                    // hop (below) leaves a gap instead, strictly safer.
                    if !converted {
                        // Refuse the hop rather than emit an unpaired exit.
                        x = run_end + 2;
                        continue;
                    }
                }
                entities.push(PlacedEntity {
                    name: hop_ug.to_string(),
                    x: run_end + 1,
                    y: out_y,
                    direction: EntityDirection::East,
                    io_type: Some("output".to_string()),
                    carries: Some(item.to_string()),
                    segment_id: merger_seg_id.clone(),
                    rate: Some(total_rate),
                    ..Default::default()
                });
                x = run_end + 2;
                continue;
            }
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x,
                y: out_y,
                direction: EntityDirection::East,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
            x += 1;
        }

        // SOUTH column from out_y to merge_start_y.
        for y in out_y..merge_start_y {
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x: col_x,
                y,
                direction: EntityDirection::South,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
        }
    }

    // Sequential splitter cascade merging N south columns into 1.
    // Columns are at x = merge_x (row n-1) through merge_x + n-1 (row 0).
    //
    // At each step we place a SOUTH splitter that merges two adjacent columns.
    // A SOUTH splitter at (x, y) spans tiles (x, y) and (x+1, y), accepting
    // input from (x, y-1) and (x+1, y-1), outputting at (x, y+1) and (x+1, y+1).
    // We use the left output (x) and discard the right.
    //
    // Between steps, ALL surviving columns need a continuation belt at each row
    // so they stay connected through to the next splitter.
    let mut y_cursor = merge_start_y;
    // Active columns, sorted left-to-right.
    let all_x: Vec<i32> = (0..n as i32).map(|i| merge_x + i).collect();

    // #567: N-to-M output. Partition the columns into `n_output` contiguous
    // groups, each folded to ONE balancer-merged output belt (e.g. 10 rows ->
    // two 5->1 groups = two 15/s belts). A single GLOBAL fold-to-M is wrong:
    // it dumps the whole tail onto the leftmost survivors (one gets ~all the
    // columns and saturates, the rest idle) — the measured half-empty belts.
    // Per-group folds keep each output belt fed by exactly its own columns.

    let mut surviving: Vec<i32> = all_x.clone(); // columns not yet merged away

    // Fold right-to-left WITHIN each group. At every step merge the rightmost
    // surviving column that still has a same-group left neighbour.
    loop {
        // Pick the rightmost surviving column whose left neighbour survives
        // AND is in the same group (i.e. not a group boundary).
        let mut merge_idx: Option<usize> = None;
        for i in (1..surviving.len()).rev() {
            let ri = surviving[i];
            let li = surviving[i - 1];
            let rg = group_of[(ri - merge_x) as usize];
            let lg = group_of[(li - merge_x) as usize];
            if rg == lg {
                merge_idx = Some(i);
                break;
            }
        }
        let Some(idx) = merge_idx else { break };
        let right_x = surviving.remove(idx);
        let left_x = surviving[idx - 1];

        // Splitter merging left_x and left_x+1 (adjacent; same-group pair).
        if right_x != left_x + 1 {
            for x in ((left_x + 2)..=right_x).rev() {
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x,
                    y: y_cursor,
                    direction: EntityDirection::West,
                    carries: Some(item.to_string()),
                    segment_id: merger_seg_id.clone(),
                    rate: Some(total_rate),
                    ..Default::default()
                });
            }
        }
        // Pass-through belts at the splitter row for every uninvolved
        // surviving column (all groups).
        for &ax in &surviving {
            if ax != left_x && ax != left_x + 1 {
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x: ax,
                    y: y_cursor,
                    direction: EntityDirection::South,
                    carries: Some(item.to_string()),
                    segment_id: merger_seg_id.clone(),
                    rate: Some(total_rate),
                    ..Default::default()
                });
            }
        }
        entities.push(PlacedEntity {
            name: splitter_name.to_string(),
            x: left_x,
            y: y_cursor,
            direction: EntityDirection::South,
            carries: Some(item.to_string()),
            segment_id: merger_seg_id.clone(),
            rate: Some(total_rate),
            ..Default::default()
        });
        y_cursor += 1;

        // Continuation belts below the splitter for all surviving columns.
        for &ax in &surviving {
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x: ax,
                y: y_cursor,
                direction: EntityDirection::South,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
        }
        y_cursor += 1;
    }

    // Zero-fold case (#727 unit 2): when every group is a single column
    // the loop never runs and y_cursor still sits at merge_start_y — the
    // tails would land INSIDE the row region and never register as
    // boundary sinks. Emit one row of continuation belts so each column
    // gets a real southbound tail. (Reachable pre-unit-2 only when the
    // count-based n_output equaled n; no fixture did.)
    if y_cursor == merge_start_y && n_output > 1 {
        for &ax in &surviving {
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x: ax,
                y: y_cursor,
                direction: EntityDirection::South,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
        }
        y_cursor += 1;
    }

    // Survivor columns (one per group = one per output belt), each with a tail
    // at `y_cursor - 1`. n_output == 1 -> single tail, byte-identical to the
    // pre-#567 behavior. n_output > 1 -> n_output parallel exit belts.
    let mut tails: Vec<PlacedEntity> = Vec::with_capacity(n_output);
    for &ax in &surviving {
        tails.push(PlacedEntity {
            name: belt_name.to_string(),
            x: ax,
            y: y_cursor - 1,
            direction: EntityDirection::South,
            carries: Some(item.to_string()),
            segment_id: merger_seg_id.clone(),
            rate: Some(total_rate),
            ..Default::default()
        });
    }

    (entities, tails, y_cursor, merge_x + n as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #728 round 1: the partition's own unit pins — sizing and
    /// assignment from one walk, asymmetric rates included (the
    /// symmetric specimen masked the column-order reversal).
    #[test]
    fn partition_symmetric_three_rows_one_tail_each() {
        let (n, groups) = partition_columns(&[30.0, 30.0, 30.0], 45.0, 90.0);
        assert_eq!((n, groups), (3, vec![0, 1, 2]));
    }

    #[test]
    fn partition_asymmetric_pairs_the_light_column() {
        // Column order [3, 40, 40] (rates already reversed to columns):
        // 3+40 = 43 fits one 45/s tail; the last 40 gets its own.
        let (n, groups) = partition_columns(&[3.0, 40.0, 40.0], 45.0, 83.0);
        assert_eq!((n, groups.clone()), (2, vec![0, 0, 1]));
        // No group over cap.
        for g in 0..n {
            let d: f64 = [3.0, 40.0, 40.0]
                .iter()
                .zip(&groups)
                .filter(|(_, &gg)| gg == g)
                .map(|(r, _)| r)
                .sum();
            assert!(d <= 45.0 + 1e-9, "group {g} over cap: {d}");
        }
    }

    #[test]
    fn partition_over_cap_column_gets_its_own_tail() {
        // The codex-HIGH counterexample: [60, 60] at 45 — no feasible
        // capacity partition exists; each column gets its own tail and
        // nothing empty-groups.
        let (n, groups) = partition_columns(&[60.0, 60.0], 45.0, 120.0);
        assert_eq!((n, groups), (2, vec![0, 1]));
    }

    #[test]
    fn partition_under_cap_is_single_group() {
        let (n, groups) = partition_columns(&[10.0, 10.0, 10.0], 45.0, 30.0);
        assert_eq!((n, groups), (1, vec![0, 0, 0]));
    }

    /// #728 round 2: drive `merge_output_rows` through the ZERO-FOLD
    /// branch end-to-end — three sub-cap rows whose partition is all
    /// singletons must yield three distinct southbound tails below the
    /// merge region (the previously fixed-and-reverted branch).
    #[test]
    fn merger_zero_fold_emits_three_distinct_tails() {
        let mk = |y: i32| {
            let mut rs = make_test_row_span(
                "copper-cable",
                y,
                vec![],
                vec![ItemFlow {
                    item: "copper-cable".to_string(),
                    rate: 5.0,
                    is_fluid: false,
                    module_id: 0,
                }],
                6, // 30/s per row
                vec![],
            );
            rs.output_belt_y = y + 2;
            rs
        };
        let rows = [mk(0), mk(5), mk(10)];
        let output_ys: Vec<i32> = rows.iter().map(|r| r.output_belt_y).collect();
        let (_entities, tails, end_y, _mx) = merge_output_rows(
            &[0, 1, 2],
            &output_ys,
            "copper-cable",
            &rows,
            15,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );
        assert_eq!(tails.len(), 3, "three 30/s rows at 45/s need three tails");
        let mut xs: Vec<i32> = tails.iter().map(|t| t.x).collect();
        xs.sort_unstable();
        xs.dedup();
        assert_eq!(xs.len(), 3, "tails must sit on distinct columns");
        assert!(end_y > 15, "the zero-fold branch must advance past merge_start_y");
    }

    /// #728 round 2: the column-order REVERSAL pin, end-to-end. Rows
    /// [40, 40, 3] (row order) land on columns [3, 40, 40] (row 0 is
    /// the RIGHTMOST column): the light column pairs with its 40/s
    /// neighbour under one 45/s tail and the far 40 gets its own —
    /// survivors at merge_x and merge_x+2. With the `.rev()` reverted
    /// the partition reads [40, 40, 3], groups {40}/{40+3}, and the
    /// second survivor sits at merge_x+1 — this pin fails.
    #[test]
    fn merger_asymmetric_rates_pin_the_column_order() {
        let mk = |y: i32, machines: usize| {
            let mut rs = make_test_row_span(
                "copper-cable",
                y,
                vec![],
                vec![ItemFlow {
                    item: "copper-cable".to_string(),
                    rate: 5.0,
                    is_fluid: false,
                    module_id: 0,
                }],
                machines,
                vec![],
            );
            rs.output_belt_y = y + 2;
            rs
        };
        // row 0 = 40/s, row 1 = 40/s, row 2 = 5/s (light).
        let rows = [mk(0, 8), mk(5, 8), mk(10, 1)];
        let output_ys: Vec<i32> = rows.iter().map(|r| r.output_belt_y).collect();
        let (_entities, tails, _end_y, _mx) = merge_output_rows(
            &[0, 1, 2],
            &output_ys,
            "copper-cable",
            &rows,
            15,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );
        let mut xs: Vec<i32> = tails.iter().map(|t| t.x).collect();
        xs.sort_unstable();
        assert_eq!(
            xs.len(),
            2,
            "40+40+5 at 45/s packs as {{5+40}}/{{40}} — two tails"
        );
        // Columns are merge_x + i with row 0 rightmost; group 0 folds
        // columns 0..=1 (survivor merge_x = xs[0]) and group 1 is the
        // singleton column 2 (survivor merge_x + 2).
        assert_eq!(
            xs[1] - xs[0],
            2,
            "the second tail must be the RIGHTMOST column (row 0's 40/s) — \
             a reversed column order folds columns 1..=2 instead and puts \
             it at merge_x + 1: {xs:?}"
        );
    }
    use crate::models::{ItemFlow, MachineSpec};

    fn make_test_row_span(
        recipe: &str,
        y_start: i32,
        inputs: Vec<ItemFlow>,
        outputs: Vec<ItemFlow>,
        machine_count: usize,
        input_belt_y: Vec<i32>,
    ) -> RowSpan {
        RowSpan {
            y_start,
            y_end: y_start + 3,
            spec: MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: recipe.to_string(),
                self_loop: vec![],
                voider: false,
                game_modules: Vec::new(),
                count: machine_count as f64,
                inputs,
                outputs,
            },
            machine_count,
            module_id: 0,
            input_belt_y,
            output_belt_y: y_start + 2,
            row_width: 10,
            fluid_port_ys: Vec::new(),
            fluid_port_pipes: Vec::new(),
            fluid_output_port_pipes: Vec::new(),
            output_east: true,
            output_belt_x_min: 0,
            output_belt_x_max: 9,
            output_feed_x_min: None,
            horizontal_stack: None,
            secondary_output_belt: None,
            sorted_output_belts: Vec::new(),
            di_input: Vec::new(),
        }
    }

    /// Phase 2 (rfc-solver-net-flow): two output items' merge blocks must
    /// tile east via the threaded cursor instead of stamping the same
    /// tiles. Regression for the review finding that per-item merge_x was
    /// computed independently.
    #[test]
    fn test_two_items_merge_blocks_do_not_overlap() {
        use rustc_hash::FxHashSet;
        let row0 = make_test_row_span(
            "iron-gear-wheel",
            0,
            vec![],
            vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 2.0,
                is_fluid: false,
                module_id: 0,
            }],
            2,
            vec![0],
        );
        let row1 = make_test_row_span(
            "iron-stick",
            5,
            vec![],
            vec![ItemFlow {
                item: "iron-stick".to_string(),
                rate: 2.0,
                is_fluid: false,
                module_id: 0,
            }],
            2,
            vec![5],
        );
        let rows = [row0, row1];
        let (a_ents, _tails, a_end_y, a_max_x) = merge_output_rows(
            &[0],
            &[rows[0].output_belt_y],
            "iron-gear-wheel",
            &rows,
            15,
            None,
            11,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );
        // Caller threads: next min_merge_x = returned max_x + 1, start_y = max_y.
        let blocked: Vec<i32> = ((a_max_x - 1)..a_max_x).collect();
        let (b_ents, _tails, _b_end_y, b_max_x) = merge_output_rows(
            &[1],
            &[rows[1].output_belt_y],
            "iron-stick",
            &rows,
            a_end_y.max(15),
            None,
            a_max_x + 1,
            &blocked,
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );
        assert!(b_max_x > a_max_x);
        let a_tiles: FxHashSet<(i32, i32)> = a_ents.iter().map(|e| (e.x, e.y)).collect();
        let overlap: Vec<(i32, i32)> = b_ents
            .iter()
            .map(|e| (e.x, e.y))
            .filter(|t| a_tiles.contains(t))
            .collect();
        assert!(overlap.is_empty(), "merge blocks overlap at {overlap:?}");
        // And without the cursor they WOULD overlap (guard the guard):
        let (c_ents, _tails, _c_end_y, _c) = merge_output_rows(
            &[1],
            &[rows[1].output_belt_y],
            "iron-stick",
            &rows,
            15,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );
        let c_overlap = c_ents
            .iter()
            .map(|e| (e.x, e.y))
            .any(|t| a_tiles.contains(&t));
        assert!(
            c_overlap,
            "expected uncursored merges to collide — geometry changed?"
        );
    }

    #[test]
    fn test_merge_output_rows_single_row() {
        let row_span = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 10.0,
                is_fluid: false,
                module_id: 0,
            }],
            1,
            vec![],
        );

        let output_rows = vec![0];
        let output_ys = vec![row_span.output_belt_y];
        let (entities, _tails, _end_y, _merge_max_x) = merge_output_rows(
            &output_rows,
            &output_ys,
            "iron-plate",
            &[row_span],
            20,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );

        // Single row should extend EAST and SOUTH without splitters
        assert!(!entities.is_empty());
        assert!(entities
            .iter()
            .all(|e| e.carries.as_deref() == Some("iron-plate")));
    }

    #[test]
    fn test_merge_output_rows_multiple_rows() {
        let row_span1 = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 10.0,
                is_fluid: false,
                module_id: 0,
            }],
            1,
            vec![],
        );
        let row_span2 = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 10.0,
                is_fluid: false,
                module_id: 0,
            }],
            1,
            vec![],
        );

        let output_rows = vec![0, 1];
        let output_ys = vec![row_span1.output_belt_y, row_span2.output_belt_y];
        let (entities, _tails, _end_y, _merge_max_x) = merge_output_rows(
            &output_rows,
            &output_ys,
            "iron-plate",
            &[row_span1, row_span2],
            20,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );

        // Multiple rows should include splitters
        let splitters = entities
            .iter()
            .filter(|e| e.name.contains("splitter"))
            .count();
        assert!(splitters > 0, "Expected splitters for multiple rows");
    }

    #[test]
    fn test_merge_output_rows_two_rows_have_splitters_and_correct_item() {
        // Two rows producing iron-gear-wheel: the merger must emit splitters and
        // all entities must carry iron-gear-wheel.
        let row0 = {
            let mut rs = make_test_row_span(
                "iron-gear-wheel",
                0,
                vec![],
                vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 5.0,
                    is_fluid: false,
                    module_id: 0,
                }],
                2,
                vec![],
            );
            rs.output_belt_y = 2;
            rs.row_width = 8;
            rs
        };
        let row1 = {
            let mut rs = make_test_row_span(
                "iron-gear-wheel",
                5,
                vec![],
                vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 5.0,
                    is_fluid: false,
                    module_id: 0,
                }],
                2,
                vec![],
            );
            rs.output_belt_y = 7;
            rs.row_width = 8;
            rs
        };

        let output_ys = vec![row0.output_belt_y, row1.output_belt_y];
        let (entities, _tails, end_y, merge_max_x) = merge_output_rows(
            &[0, 1],
            &output_ys,
            "iron-gear-wheel",
            &[row0, row1],
            15,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );

        // Splitters must be present
        let splitters: Vec<_> = entities
            .iter()
            .filter(|e| e.name.contains("splitter"))
            .collect();
        assert!(
            !splitters.is_empty(),
            "Expected splitter(s) in merger for 2 rows"
        );

        // Every entity must carry the correct item
        for e in &entities {
            assert_eq!(
                e.carries.as_deref(),
                Some("iron-gear-wheel"),
                "All merger entities should carry iron-gear-wheel, got {:?}",
                e
            );
        }

        // end_y and merge_max_x should be sane
        assert!(end_y > 15, "end_y should be greater than merge_start_y");
        assert!(merge_max_x > 0, "merge_max_x should be positive");
    }

    #[test]
    fn test_merge_output_rows_splitters_face_south() {
        // Splitters produced by merge_output_rows should face SOUTH (merging
        // parallel SOUTH-flowing trunks).
        let row0 = make_test_row_span(
            "electronic-circuit",
            0,
            vec![],
            vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: 5.0,
                is_fluid: false,
                module_id: 0,
            }],
            1,
            vec![],
        );
        let row1 = make_test_row_span(
            "electronic-circuit",
            5,
            vec![],
            vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: 5.0,
                is_fluid: false,
                module_id: 0,
            }],
            1,
            vec![],
        );

        let output_ys = vec![row0.output_belt_y, row1.output_belt_y];
        let (entities, _tails, _end_y, _merge_max_x) = merge_output_rows(
            &[0, 1],
            &output_ys,
            "electronic-circuit",
            &[row0, row1],
            20,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &FxHashSet::default(),
            &mut FxHashSet::default(),
        );

        let splitters: Vec<_> = entities
            .iter()
            .filter(|e| e.name.contains("splitter"))
            .collect();
        for s in &splitters {
            assert_eq!(
                s.direction,
                EntityDirection::South,
                "Merger splitters should face SOUTH"
            );
        }
    }

    /// #309 regression: when something Step 4-6 already placed (a
    /// dual-fate item's intermediate-lane `ret` belt) occupies the row's
    /// own exit tile — `row_width`, the FIRST tile this function's east
    /// extension would otherwise stamp a plain belt onto — the row-exit
    /// bridge must detour underground around it instead of colliding
    /// (issue #309's illegal entity overlaps). No entity may land on the
    /// occupied tile, and the row's own last belt tile (`row_width - 1`,
    /// owned by the caller's `row_entities` — this function never places
    /// it) must be reported via `row_tile_overrides` so the caller drops
    /// it before merging, rather than double-placing it alongside the
    /// fresh UG entrance this function pushes there.
    #[test]
    fn test_merge_output_rows_bridges_pre_occupied_row_exit() {
        let row = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            1,
            vec![],
        );
        let out_y = row.output_belt_y;
        let row_width = row.row_width;
        // Simulate a `ret` belt already sitting on the row's own exit
        // tile — exactly the #309 collision.
        let mut existing_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
        existing_tiles.insert((row_width, out_y));
        let mut row_tile_overrides: FxHashSet<(i32, i32)> = FxHashSet::default();

        let (entities, _tails, _end_y, _merge_max_x) = merge_output_rows(
            &[0],
            &[out_y],
            "iron-plate",
            &[row],
            20,
            None,
            0,
            &[],
            &StackingCtx::unstacked(),
            &existing_tiles,
            &mut row_tile_overrides,
        );

        assert!(
            entities.iter().all(|e| (e.x, e.y) != (row_width, out_y)),
            "no entity may be placed on the pre-occupied tile: {entities:#?}"
        );
        // Tile-uniqueness, not just the pre-occupied tile: a review finding
        // on this exact fixture shape caught a SECOND, latent collision —
        // when `col_x` (the south column) landed adjacent to the bridge's
        // exit tile, both the exit and the south column's own top tile got
        // placed at the same coordinate. The narrower "not on the blocked
        // tile" check above can't see that; only a full uniqueness sweep
        // over every emitted entity does.
        let mut seen: FxHashSet<(i32, i32)> = FxHashSet::default();
        let dups: Vec<(i32, i32)> = entities
            .iter()
            .map(|e| (e.x, e.y))
            .filter(|&tile| !seen.insert(tile))
            .collect();
        assert!(
            dups.is_empty(),
            "duplicate tile(s) among emitted entities {dups:?}: {entities:#?}"
        );
        assert!(
            row_tile_overrides.contains(&(row_width - 1, out_y)),
            "the row's own last belt tile must be reported for eviction: {row_tile_overrides:?}"
        );
        let entrance = entities
            .iter()
            .find(|e| (e.x, e.y) == (row_width - 1, out_y))
            .expect("a fresh UG entrance must replace the row's own last belt tile");
        assert_eq!(entrance.name, "underground-belt");
        assert_eq!(entrance.io_type.as_deref(), Some("input"));
        assert_eq!(entrance.direction, EntityDirection::East);
    }

    // -----------------------------------------------------------------------
    // plan_bus_lanes via solver - integration
    // -----------------------------------------------------------------------
}
