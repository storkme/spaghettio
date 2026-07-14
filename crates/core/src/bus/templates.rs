//! Assembly row templates: patterns of belts, inserters, and machines.
//!
//! Every belt and inserter entity is tagged with `carries` so the validator
//! can trace item flow through the layout.
//!
//! Machines are packed with zero gap (3-tile pitch for 3×3 machines).
//! When lane splitting is active, machines are split into two groups with a
//! sideload bridge in between so both output belt lanes are utilised.
//!
//! Port of `src/bus/templates.py`.

use crate::bus::inserter_ladder::{
    capped_limit, contest_favors_far, size_side, InserterTier, Reach, SidePlan,
};
use crate::models::{EntityDirection, PlacedEntity};

// Gap between machine groups when lane-splitting output belts.
// 3 tiles: 1 for sideload target filler, 1 for through-belt filler,
// 1 for the NORTH lift from group 2.
// (LANE_SPLIT_GAP = 3 deleted in the inline-bridge unification —
// machines now pack tight with the bridge stamped inline.)

// Fluid port dx (relative to machine tile_position) for each machine type.
fn fluid_input_port_dx(machine_entity: &str) -> i32 {
    match machine_entity {
        "assembling-machine-2" | "assembling-machine-3" => 1,
        _ => 0,
    }
}

/// Map `output_east` flag to the corresponding belt direction.
fn output_dir(output_east: bool) -> EntityDirection {
    if output_east { EntityDirection::East } else { EntityDirection::West }
}

/// Number of tiles to drop from the east end of the last machine's belt stamp.
///
/// A belt tile is "orphan" when it sits east of every inserter that picks from
/// (east-flow input) or drops onto (west-flow output) the belt: items never
/// flow through it, so it has no functional role. Skipping these tiles frees
/// corridor for the ghost router. `last_adjacency_dx` is the rightmost `dx`
/// of an inserter adjacency on the belt for one machine.
fn east_tail_skip(msz: i32, last_adjacency_dx: i32) -> i32 {
    (msz - 1 - last_adjacency_dx).max(0)
}

/// Generate the 6-entity sideload bridge between two machine groups.
///
/// `output_row_dy` is the output belt's offset from `y_offset`
/// (6 for `single_input_row`, 7 for `dual_input_row`).
///
/// When `output_east` is `true`, the bridge is mirrored: group 1 items
/// flow EAST across the bridge into group 2 (instead of group 2 → group 1).
///
/// `segment_id` is applied to all 6 emitted tiles so the bridge
/// inherits the row's belt-out provenance — required by HS rows where
/// the bridge sits inside the output belt run, and harmless for VS
/// rows where it slots into the lane-split gap.
fn sideload_bridge(
    gap_start_x: i32,
    y_offset: i32,
    output_row_dy: i32,
    belt: &str,
    item: &str,
    output_east: bool,
    segment_id: Option<String>,
) -> Vec<PlacedEntity> {
    let bridge_y = y_offset + output_row_dy - 1;
    let output_y = y_offset + output_row_dy;

    let carries = Some(item.to_string());
    let belt = belt.to_string();

    if output_east {
        // EAST flow: group 1 → bridge EAST → group 2
        vec![
            // Bridge row
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: bridge_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: bridge_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: bridge_y,
                direction: EntityDirection::South,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            },
            // Output belt row — gap tiles
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: output_y,
                direction: EntityDirection::North,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            }, // lifts group1 items up to bridge
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: output_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            }, // through-belt filler
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: output_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                segment_id,
                ..Default::default()
            }, // sideload target (through-belt)
        ]
    } else {
        vec![
            // Bridge row (y+5 or y+6 depending on template)
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: bridge_y,
                direction: EntityDirection::South,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: bridge_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: bridge_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            },
            // Output belt row — gap tiles
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: output_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            }, // sideload target (through-belt)
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: output_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                segment_id: segment_id.clone(),
                ..Default::default()
            }, // through-belt filler
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: output_y,
                direction: EntityDirection::North,
                carries: carries.clone(),
                segment_id,
                ..Default::default()
            }, // lifts group2 items up to bridge
        ]
    }
}

// ─── inline output-bridge helpers ─────────────────────────────────────────────
//
// All row templates with a `lane_split` flag share the same lane-balance
// problem: south-facing output inserters drop on the near lane only
// (rule I5/B8), so a single-row recipe runs at half belt capacity until
// items are sideloaded onto the far lane somewhere along the row. The
// 6-tile `sideload_bridge` solves this by lifting items up to a bridge
// row, running them west/east, and dropping them back down to sideload.
//
// Strategy A (used by single_input_row, dual_input_row, fluid_dual_input_row,
// fluid_multi_input_row, dual_input_row_horizontal): pack machines tight
// at `pitch` per machine; the anchor's output inserter shifts from `mx+1`
// to `mx`; the bridge stamps at cols `[mx_anchor+1, mx_anchor+3)` on the
// inserter-out row + output belt row. Suitable when `inserter_out_y` has
// at most one inserter per machine.
//
// Strategy B (used by triple_input_row): the anchor's input-3 long-hand
// inserter sits at `mx+2` on `inserter_out_y` alongside the output
// inserter at `mx+1`. We shift it to `mx` (it can pick input-3 from the
// belt row's left col, drop into the machine's left col) and add 1 tile
// of gap between the anchor machine and its successor. The bridge then
// occupies cols `[mx_anchor+2, mx_anchor+5)` — the first 2 cols of the
// (now-free) gap zone plus the next machine's left col.

/// Pick the inline-bridge anchor index for a row of `machine_count`
/// machines, or `None` when the row should not have a bridge.
pub(crate) fn inline_bridge_anchor(machine_count: usize, enabled: bool) -> Option<usize> {
    if !enabled || machine_count < 2 {
        None
    } else {
        Some((machine_count - 1) / 2)
    }
}

/// The 3 columns owned by an inline Strategy-A bridge anchored at
/// `mx_anchor`. Output-belt loops in Strategy-A row templates skip
/// these to avoid double-stamping at the bridge tiles (no entity
/// dedup — duplicate `entities.push` produces broken blueprints).
pub(crate) fn inline_bridge_x_set_a(mx_anchor: i32) -> rustc_hash::FxHashSet<i32> {
    (mx_anchor + 1..mx_anchor + 4).collect()
}

/// The 3 columns owned by a Strategy-B bridge anchored at `mx_anchor`
/// (with a 1-tile gap east of the anchor machine). Bridge starts 2
/// cols east of the anchor's left edge.
pub(crate) fn inline_bridge_x_set_b(mx_anchor: i32) -> rustc_hash::FxHashSet<i32> {
    (mx_anchor + 2..mx_anchor + 5).collect()
}

/// Stamp the 6-tile inline bridge for Strategy A. Caller is responsible
/// for shifting the anchor's output inserter from `mx+1` to `mx` and
/// for skipping the cols in `inline_bridge_x_set_a` when emitting the
/// row's output belt.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stamp_inline_bridge_a(
    entities: &mut Vec<PlacedEntity>,
    mx_anchor: i32,
    y_offset: i32,
    output_row_dy: i32,
    output_belt: &str,
    output_item: &str,
    output_east: bool,
    segment_id: Option<String>,
) {
    entities.extend(sideload_bridge(
        mx_anchor + 1,
        y_offset,
        output_row_dy,
        output_belt,
        output_item,
        output_east,
        segment_id,
    ));
}

/// Stamp the 6-tile inline bridge for Strategy B. Caller is responsible
/// for inserting 1 tile of gap east of the anchor machine, shifting the
/// anchor's input-3 long-hand inserter from `mx+2` to `mx`, and
/// skipping the cols in `inline_bridge_x_set_b` when emitting the row's
/// output belt.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stamp_inline_bridge_b(
    entities: &mut Vec<PlacedEntity>,
    mx_anchor: i32,
    y_offset: i32,
    output_row_dy: i32,
    output_belt: &str,
    output_item: &str,
    output_east: bool,
    segment_id: Option<String>,
) {
    entities.extend(sideload_bridge(
        mx_anchor + 2,
        y_offset,
        output_row_dy,
        output_belt,
        output_item,
        output_east,
        segment_id,
    ));
}

// (GapRow + stamp_lane_split_gap deleted in the inline-bridge
// unification — every template now stamps its bridge inline via
// stamp_inline_bridge_a or stamp_inline_bridge_b. No template
// continues input belts across an empty gap any more, since machines
// pack tight (Strategy A) or carry their own per-row continuation
// at the 1-tile gap (Strategy B).)

/// Free non-baseline dx offsets within `[0, stamped_stop)`, ascending,
/// excluding every dx in `occupied` — the candidate columns
/// `inserter_ladder::size_side`'s extra picks (beyond the one baseline
/// slot every side already has) land on. Deriving this from the same
/// stamped-range/occupancy variables the belts and bridge use (rather
/// than a hardcoded budget table) means the ladder's column budget can
/// never drift out of sync with the geometry that actually places
/// entities — see `docs/rfp-inserter-sizing.md`'s free-column-budget
/// table, which this reproduces for every `single_input_row` position.
fn free_extra_dx(stamped_stop: i32, occupied: &[i32]) -> Vec<i32> {
    (0..stamped_stop).filter(|dx| !occupied.contains(dx)).collect()
}

/// Stamp one machine side's ladder-sized inserters: one at `baseline_dx`,
/// then `extra_dx` in ascending order until `plan.count` inserters are
/// placed. `extra_dx` must have at least `plan.count - 1` entries —
/// guaranteed by construction when its length is the `position_budget`
/// passed to the `size_side` call that produced `plan`.
#[allow(clippy::too_many_arguments)]
fn stamp_side_inserters(
    entities: &mut Vec<PlacedEntity>,
    plan: &SidePlan,
    mx: i32,
    y: i32,
    direction: EntityDirection,
    item: &str,
    segment_id: &Option<String>,
    baseline_dx: i32,
    extra_dx: &[i32],
) {
    debug_assert!(
        extra_dx.len() + 1 >= plan.count,
        "stamp_side_inserters: extra_dx has {} entries but plan.count is {} — \
         under-placement with no signal; caller must pass at least plan.count - 1 extras",
        extra_dx.len(),
        plan.count
    );
    let dxs = std::iter::once(baseline_dx).chain(extra_dx.iter().copied()).take(plan.count);
    for dx in dxs {
        entities.push(PlacedEntity {
            name: plan.entity.to_string(),
            x: mx + dx,
            y,
            direction,
            carries: Some(item.to_string()),
            segment_id: segment_id.clone(),
            ..Default::default()
        });
    }
}

/// Emit `InserterSideCapped` when `plan` couldn't cover its required
/// rate even at the richest tier `max_inserter_tier` allows and every
/// free column used. No-op (and no trace event) when the side is fully
/// covered.
///
/// `(machine_x, machine_y)` is the MACHINE ORIGIN (the tile the
/// validator's warnings anchor at), and `lost_contest` is true only at
/// the near/far shared-column contest sites for the LOSING side — the
/// binding-constraint `limit` itself is derived centrally in
/// `inserter_ladder::capped_limit` from the plan.
fn emit_shortfall_trace(
    recipe: &str,
    side_is_output: bool,
    required: f64,
    plan: &SidePlan,
    machine_x: i32,
    machine_y: i32,
    lost_contest: bool,
) {
    if let Some(shortfall) = plan.shortfall {
        crate::trace::emit(crate::trace::TraceEvent::InserterSideCapped {
            recipe: recipe.to_string(),
            side_is_output,
            required,
            placed_entity: plan.entity.to_string(),
            placed_count: plan.count,
            shortfall,
            machine_x,
            machine_y,
            limit: capped_limit(required, plan, lost_contest).to_string(),
        });
    }
}

/// Row for a recipe with 1 solid input.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : input belt (EAST)
///   y+1 : input inserter(s) (SOUTH)
///   y+2..y+2+msz-1 : machine (msz×msz)
///   y+2+msz : output inserter(s) (SOUTH) [+ secondary output's long-handed
///             extraction inserter at mx+2, when `secondary_output` is Some]
///   y+2+msz+1 : output belt (WEST -- toward bus)
///   y+2+msz+2 : secondary output belt, when `secondary_output` is Some
/// ```
///
/// Each side's inserter count/tier is sized by
/// `inserter_ladder::size_side` (`docs/rfp-inserter-sizing.md`):
/// in-place tier upgrade (regular → fast → stack) first, extra
/// inserters into whatever free columns the position's own belt/bridge
/// geometry leaves (via [`free_extra_dx`]) only if the tier ladder alone
/// can't cover `input_rate`/`output_rate`, capped at `max_inserter_tier`.
/// `input_rate`/`output_rate` are per-machine, utilization-scaled —
/// callers compute them the same way `check_inserter_throughput` scales
/// its required rate, so the ladder and the check never size against
/// different numbers.
///
/// When `lane_split=true`, machines are split into two groups with a
/// sideload bridge between them so the output belt uses both lanes.
///
/// `secondary_output`, when `Some((item, belt))`, stamps a SECOND
/// output belt one row south of the primary (RFP Fulgora D2b —
/// uranium-processing's uranium-235 target + uranium-238 surplus:
/// `spec.outputs[1]`, which owns no belt otherwise). Extracted via a
/// dedicated long-handed inserter (reach-2, sized against
/// `secondary_output_rate` but with a hard-0 extra-column budget — v2
/// census: `mx+2` is its only slot) sharing the primary output
/// inserter's row (`y+2+msz`) at column `mx+2`: picks from the
/// machine's middle row (`y+2+msz-2`, inside the machine footprint for
/// `msz>=3`) and drops onto the new belt (`y+2+msz+2`), clearing the
/// primary belt row entirely. Column `mx+2` collides with the sideload
/// bridge's anchor columns, so callers must pass `lane_split=false`
/// whenever `secondary_output.is_some()` — `placer::can_lane_split`
/// enforces this. Requires `msz>=3` (debug-asserted); no caller today
/// passes a smaller machine with a second solid output.
///
/// Returns `(entities, row_height)`.
#[allow(clippy::too_many_arguments)]
pub fn single_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_item: &str,
    output_item: &str,
    input_belt: &str,
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
    secondary_output: Option<(&str, &str)>,
    input_rate: f64,
    output_rate: f64,
    secondary_output_rate: Option<f64>,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "single_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 4 + if secondary_output.is_some() { 1 } else { 0 };
    let mut entities = Vec::new();
    let belt_in_seg = Some(format!("row:{recipe}:belt-in:{input_item}"));
    let inserter_in_seg = Some(format!("row:{recipe}:inserter-in:{input_item}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    debug_assert!(
        secondary_output.is_none() || !lane_split,
        "single_input_row: secondary_output and lane_split are mutually exclusive \
         (bridge anchor and secondary inserter both want column mx+2)"
    );
    debug_assert!(
        secondary_output.is_none() || msz >= 3,
        "single_input_row: secondary_output needs msz>=3 for the reach-2 extraction \
         inserter to land inside the machine footprint"
    );
    let secondary_out_seg = secondary_output
        .map(|(item, _)| format!("row:{recipe}:belt-out2:{item}"));
    let secondary_ins_seg = secondary_output
        .map(|(item, _)| format!("row:{recipe}:inserter-out2:{item}"));

    let lane_split = lane_split && machine_count >= 2;
    // Pack machines tight at `pitch` (no LANE_SPLIT_GAP). Strategy A:
    // bridge stamps inline by shifting the anchor's output inserter
    // from `mx+1` to `mx`, freeing 3 cols `[mx+1, mx+3)` for the
    // bridge tiles.
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");

    let bridge_anchor = inline_bridge_anchor(machine_count, lane_split);
    let bridge_x_set = bridge_anchor
        .map(|a| inline_bridge_x_set_a(mxs[a]))
        .unwrap_or_default();

    // Input belt: east-flow, inserter picks at mx+1 → last_adjacency_dx=1.
    // West-flow output belt: inserter drops at mx+1 → last_adjacency_dx=1.
    // East-flow output belt: tiles east of the drop carry items to the merger,
    // so no tail trim.
    let in_tail = east_tail_skip(msz, 1);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    let out_belt_y = y_offset + 2 + msz + 1;
    let out_dir = output_dir(output_east);

    // Secondary belt's last-adjacency sits at dx=2 (the extraction
    // inserter's column, see below) regardless of the primary's own
    // dx=1 — trimmed independently.
    let out_tail2 = if output_east { 0 } else { east_tail_skip(msz, 2) };

    for (i, &mx) in mxs.iter().enumerate() {
        let is_last = mx == last_mx;
        let in_stop = if is_last { msz - in_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };
        let out_stop2 = if is_last { msz - out_tail2 } else { msz };

        // Input belt (machine_size tiles wide, continuous with adjacent machines)
        for dx in 0..in_stop {
            entities.push(PlacedEntity {
                name: input_belt.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input_item.to_string()),
                segment_id: belt_in_seg.clone(),
                ..Default::default()
            });
        }

        // Input inserter(s) — ladder-sized. Baseline slot at dx=1 (the
        // column every pre-ladder layout used); extra picks land on
        // whatever the input belt's own stamped range (`in_stop`) leaves
        // free. The bridge lives on the output row only, so it never
        // contests input columns.
        const INPUT_BASELINE_DX: i32 = 1;
        let input_extra_dx = free_extra_dx(in_stop, &[INPUT_BASELINE_DX]);
        let input_plan = size_side(input_rate, Reach::Near, input_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &input_plan,
            mx,
            y_offset + 1,
            EntityDirection::South,
            input_item,
            &inserter_in_seg,
            INPUT_BASELINE_DX,
            &input_extra_dx,
        );
        emit_shortfall_trace(recipe, false, input_rate, &input_plan, mx, y_offset + 2, false);

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 2,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output inserter(s) — ladder-sized. Baseline slot shifts to
        // `mx` at the bridge anchor to free col `mx+1` for the bridge's
        // South-belt entry; extra picks exclude every column the bridge
        // owns (mapped from `bridge_x_set`'s absolute x's) and, when a
        // secondary output rides this row, column `mx+2` (the
        // secondary's own dedicated inserter).
        let out_ins_y = y_offset + 2 + msz;
        let is_anchor = Some(i) == bridge_anchor;
        let out_baseline_dx = if is_anchor { 0 } else { 1 };
        let mut out_occupied = vec![out_baseline_dx];
        for dx in 0..msz {
            if bridge_x_set.contains(&(mx + dx)) {
                out_occupied.push(dx);
            }
        }
        if secondary_output.is_some() {
            out_occupied.push(2);
        }
        let out_extra_dx = free_extra_dx(out_stop, &out_occupied);
        let output_plan = size_side(output_rate, Reach::Near, out_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &output_plan,
            mx,
            out_ins_y,
            EntityDirection::South,
            output_item,
            &inserter_out_seg,
            out_baseline_dx,
            &out_extra_dx,
        );
        emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, y_offset + 2, false);

        // Output belt (machine_size tiles wide) — skip cols owned by
        // the bridge to avoid duplicate-tile stamps.
        for dx in 0..out_stop {
            let x = mx + dx;
            if bridge_x_set.contains(&x) {
                continue;
            }
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }

        // Secondary output (RFP Fulgora D2b): dedicated long-handed
        // inserter at `mx+2`, same row as the primary output inserter —
        // reach 2 picks the machine's middle row (inside the footprint
        // for msz>=3) and drops onto a new belt one row south of the
        // primary, clearing it entirely. `mx+2` is guaranteed free of
        // both the primary inserter(s) (mx or mx+1, plus any extra pick
        // — excluded above) and the bridge (`lane_split` is always
        // false here, see module doc comment). Sized on `Reach::Far`
        // with a hard-0 extra-column budget (v2 census: no free column
        // at this slot), so it always places exactly 1 inserter — a
        // shortfall is recorded, never a second inserter, when the rate
        // exceeds long-handed's reach-2 ceiling.
        if let Some((sec_item, sec_belt)) = secondary_output {
            let sec_rate = secondary_output_rate.unwrap_or(0.0);
            let sec_plan = size_side(sec_rate, Reach::Far, 0, max_inserter_tier);
            debug_assert_eq!(sec_plan.count, 1, "secondary output has no extra-column budget");
            entities.push(PlacedEntity {
                name: sec_plan.entity.to_string(),
                x: mx + 2,
                y: out_ins_y,
                direction: EntityDirection::South,
                carries: Some(sec_item.to_string()),
                segment_id: secondary_ins_seg.clone(),
                ..Default::default()
            });
            emit_shortfall_trace(recipe, true, sec_rate, &sec_plan, mx, y_offset + 2, false);
            let sec_belt_y = out_belt_y + 1;
            for dx in 0..out_stop2 {
                entities.push(PlacedEntity {
                    name: sec_belt.to_string(),
                    x: mx + dx,
                    y: sec_belt_y,
                    direction: out_dir,
                    carries: Some(sec_item.to_string()),
                    segment_id: secondary_out_seg.clone(),
                    ..Default::default()
                });
            }
        }
    }

    if let Some(anchor) = bridge_anchor {
        stamp_inline_bridge_a(
            &mut entities,
            mxs[anchor],
            y_offset,
            2 + msz + 1, // output_row_dy
            output_belt,
            output_item,
            output_east,
            belt_out_seg.clone(),
        );
    }

    (entities, row_height)
}

/// Row for a recipe with 2 solid inputs.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : input belt 1 (EAST) -- far belt
///   y+1 : input belt 2 (EAST) -- close belt
///   y+2 : long-handed inserter (picks y+0) + inserter (picks y+1)
///   y+3..y+3+msz-1 : machine (msz×msz)
///   y+3+msz : output inserter (SOUTH)
///   y+3+msz+1 : output belt (WEST -- toward bus)
/// ```
///
/// When `lane_split=true`, machines are split into two groups with a
/// sideload bridge between them so the output belt uses both lanes.
///
/// Returns `(entities, row_height)`.
/// `input_items`/`input_rates` are `(far, near)` — already assigned by the
/// caller via `inserter_ladder::reassign_near_far` (the hungrier ingredient
/// goes near). `far_rate`/`near_rate`/`output_rate` are per-machine,
/// utilization-scaled (`check_inserter_throughput`'s convention).
///
/// Each side's inserter count/tier is sized by `inserter_ladder::size_side`,
/// same discipline as `single_input_row`. The near input and the far input
/// share ONE extra column (`dx=1` at `y_offset+2`, free only when the far
/// belt itself still covers it — trimmed away at `LastInRow`, see `in1_tail`
/// below): `inserter_ladder::contest_favors_far` decides who's entitled to
/// it. `docs/rfp-inserter-sizing.md` Phase 3: far's own reach-2 count-ladder
/// is now ACTIVE — a far win genuinely places a second long-handed inserter
/// (no fast/stack long-handed exists, so `max_inserter_tier` never affects
/// it; a far loss, or LastInRow ineligibility, hands the column to near
/// instead). The output side is `single_input_row`-output-shaped (2
/// interior, bridge-anchor 0, bridge-anchor-successor 1) and reuses that
/// exact derivation.
#[allow(clippy::too_many_arguments)]
pub fn dual_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str),
    output_item: &str,
    input_belts: (&str, &str),
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
    far_rate: f64,
    near_rate: f64,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "dual_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 5;
    let mut entities = Vec::new();

    let (input1, input2) = input_items;
    let (belt1, belt2) = input_belts;
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    let lane_split = lane_split && machine_count >= 2;
    // Strategy A: pack tight, shift anchor's output inserter, inline bridge.
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");

    let bridge_anchor = inline_bridge_anchor(machine_count, lane_split);
    let bridge_x_set = bridge_anchor
        .map(|a| inline_bridge_x_set_a(mxs[a]))
        .unwrap_or_default();

    // Belt 1 (far): long-handed inserter picks at mx → dx=0, trim = msz-1.
    // Belt 2 (close): regular inserter picks at mx+2 → dx=2, trim = 0 for msz=3.
    // West-flow output belt: drop at mx+1 → dx=1, trim = 1 for msz=3.
    // East-flow output belt: no trim (tiles east of drop flow to merger).
    let in1_tail = east_tail_skip(msz, 0);
    let in2_tail = east_tail_skip(msz, 2);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    let out_belt_y = y_offset + 3 + msz + 1;
    let out_dir = output_dir(output_east);

    for (i, &mx) in mxs.iter().enumerate() {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Input belt 1 -- far belt
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }

        // Input belt 2 -- close belt
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: y_offset + 1,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Near/far contested column (dx=1 at Interior, near-only at
        // LastInRow — derived from each belt's own stamped range,
        // excluding both baselines, rather than a lookup table).
        // `docs/rfp-inserter-sizing.md` Phase 3: the far side's own
        // reach-2 count-ladder is now ACTIVE — a far win genuinely
        // places a second long-handed inserter within budget.
        const FAR_BASELINE_DX: i32 = 0;
        const NEAR_BASELINE_DX: i32 = 2;
        let far_candidates = free_extra_dx(in1_stop, &[FAR_BASELINE_DX, NEAR_BASELINE_DX]);
        let near_candidates = free_extra_dx(in2_stop, &[FAR_BASELINE_DX, NEAR_BASELINE_DX]);
        let far_eligible = !far_candidates.is_empty();
        let shared_dx: Vec<i32> =
            near_candidates.iter().copied().filter(|dx| far_candidates.contains(dx)).collect();
        let near_only_dx: Vec<i32> =
            near_candidates.iter().copied().filter(|dx| !shared_dx.contains(dx)).collect();
        let far_wins = !shared_dx.is_empty() && contest_favors_far(near_rate, far_rate, far_eligible);
        let far_extra_dx: Vec<i32> = if far_wins { shared_dx.clone() } else { vec![] };
        let near_extra_dx: Vec<i32> = if far_wins { near_only_dx } else { near_candidates.clone() };

        // Far input — ladder-sized (reach-2 count-ladder; no fast/stack
        // long-handed exists, so `max_inserter_tier` never affects it).
        let far_plan = size_side(far_rate, Reach::Far, far_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &far_plan,
            mx,
            y_offset + 2,
            EntityDirection::South,
            input1,
            &inserter_in1_seg,
            FAR_BASELINE_DX,
            &far_extra_dx,
        );
        emit_shortfall_trace(
            recipe, false, far_rate, &far_plan,
            mx, y_offset + 3,
            !shared_dx.is_empty() && !far_wins,
        );

        // Near input — ladder-sized.
        let near_plan = size_side(near_rate, Reach::Near, near_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &near_plan,
            mx,
            y_offset + 2,
            EntityDirection::South,
            input2,
            &inserter_in2_seg,
            NEAR_BASELINE_DX,
            &near_extra_dx,
        );
        emit_shortfall_trace(recipe, false, near_rate, &near_plan, mx, y_offset + 3, far_wins);

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 3,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output — ladder-sized, single_input_row-output-shaped: baseline
        // shifts to `mx` at the bridge anchor, extra picks exclude the
        // bridge's own columns.
        let out_ins_y = y_offset + 3 + msz;
        let is_anchor = Some(i) == bridge_anchor;
        let out_baseline_dx = if is_anchor { 0 } else { 1 };
        let mut out_occupied = vec![out_baseline_dx];
        for dx in 0..msz {
            if bridge_x_set.contains(&(mx + dx)) {
                out_occupied.push(dx);
            }
        }
        let out_extra_dx = free_extra_dx(out_stop, &out_occupied);
        let output_plan = size_side(output_rate, Reach::Near, out_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &output_plan,
            mx,
            out_ins_y,
            EntityDirection::South,
            output_item,
            &inserter_out_seg,
            out_baseline_dx,
            &out_extra_dx,
        );
        emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, y_offset + 3, false);

        // Output belt — skip cols owned by the bridge.
        for dx in 0..out_stop {
            let x = mx + dx;
            if bridge_x_set.contains(&x) {
                continue;
            }
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }
    }

    if let Some(anchor) = bridge_anchor {
        stamp_inline_bridge_a(
            &mut entities,
            mxs[anchor],
            y_offset,
            3 + msz + 1, // output_row_dy
            output_belt,
            output_item,
            output_east,
            belt_out_seg.clone(),
        );
    }

    (entities, row_height)
}

/// HorizontalStack variant of `dual_input_row` — see
/// `docs/rfp-horizontal-trunks.md`. One long row with `k_trunks`
/// stacked east-flowing input₀ trunks on top, a single continuous
/// input₁ belt, and machines packed into `block_size`-sized blocks
/// with a 1-tile gap between blocks for trunk dives + low-demand
/// E-axis UG cross.
///
/// Y layout (msz=3 example, K=2):
/// ```text
///   y+0..y+K-1 : K stacked east-flowing trunks of input₀ (high-demand)
///   y+K        : input₁ continuous east-flowing belt (low-demand,
///                E-axis UG cross at each block boundary)
///   y+K+1      : input₀ current-feed east-flowing belt (per-block
///                segments, fed by trunk dives)
///   y+K+2      : inserter row (long-handed picks input₁ at y+K,
///                regular/stack picks input₀ at y+K+1)
///   y+K+3..y+K+5 : 3×3 machine
///   y+K+6      : output inserter row
///   y+K+7      : output belt
/// ```
///
/// Phase 1 first cut: emits the row geometry without trunk-dive
/// connectivity — the K trunks and current-feed are disconnected
/// surface belts. The validator will flag the missing item source on
/// current-feed; the dive geometry will be added in subsequent
/// iterations once the visible row shape has been validated.
///
/// `input_items` = `(input0, input1)` = `(near/high-demand, far/low-demand)`
/// — already ranked by the caller for the K-trunk assignment (unrelated to,
/// but structurally equivalent to, `dual_input_row`'s near-far reassignment:
/// the hungrier item already lands near here for a different reason).
/// `near_rate`/`far_rate`/`output_rate` are per-machine, utilization-scaled.
/// Both belts cover every machine's full width (no per-machine LastInRow
/// trim — HorizontalStack's belts are continuous across the row), so the
/// near/far contested column (`dx=1` at `inserter_in_y`) is always eligible
/// for both sides; `docs/rfp-inserter-sizing.md` Phase 3: far's reach-2
/// count-ladder is ACTIVE, same as `dual_input_row`. Output is ladder-sized
/// `single_input_row`-output-shaped (2 free columns, bridge-anchor 0).
#[allow(clippy::too_many_arguments)]
pub fn dual_input_row_horizontal(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str),
    output_item: &str,
    input_belts: (&str, &str),
    output_belt: &str,
    k_trunks: usize,
    block_size: usize,
    output_east: bool,
    near_rate: f64,
    far_rate: f64,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "dual_input_row_horizontal assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    let k = k_trunks.max(1) as i32;
    let block_size = block_size.max(1);
    let mut entities = Vec::new();

    let (input0, input1) = input_items;
    let (belt0, belt1) = input_belts;
    let trunk_seg = Some(format!("row:{recipe}:trunk:{input0}"));
    let belt1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let cf_seg = Some(format!("row:{recipe}:current-feed:{input0}"));
    let dive_seg = Some(format!("row:{recipe}:trunk-dive:{input0}"));
    let inserter_in0_seg = Some(format!("row:{recipe}:inserter-in:{input0}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Block layout: each block is `block_size` machines wide; a 1-tile
    // dive column precedes each block (including block 0). Trunk b
    // dives at `dive_xs[b]`, then block b's machines start at
    // `dive_xs[b] + 1` and span `block_size * msz` tiles.
    //
    // The leftmost dive sits at `x_offset + 1` (NOT `x_offset`) so the
    // iron-plate row's east-axis UG cross around it — UG INPUT at
    // `dive - 1`, UG OUTPUT at `dive + 1` — has both endpoints inside
    // the row template's iteration range. Without this offset the UG
    // INPUT would land at `x_offset - 1` (bus area, outside the
    // template), leaving the UG OUTPUT orphaned and the bus router's
    // straight iron-plate delivery sideloading onto the dive's
    // south-belt at `(x_offset, y+K)`.
    let n_blocks = machine_count.div_ceil(block_size);
    let dive_xs: Vec<i32> = (0..n_blocks)
        .map(|b| x_offset + 1 + (b as i32) * (block_size as i32 * pitch + 1))
        .collect();
    let mut machine_xs: Vec<i32> = Vec::with_capacity(machine_count);
    for (b, &dive_x) in dive_xs.iter().enumerate() {
        let block_count = if b == n_blocks - 1 {
            machine_count - b * block_size
        } else {
            block_size
        };
        for m in 0..block_count {
            machine_xs.push(dive_x + 1 + (m as i32) * pitch);
        }
    }
    let row_west_x = x_offset;
    let row_east_x_excl = machine_xs.last().copied().unwrap_or(x_offset) + pitch;

    // Y row indices.
    let trunk_y = |k_idx: i32| y_offset + k_idx;
    let belt1_y = y_offset + k;
    let cf_y = y_offset + k + 1;
    let inserter_in_y = y_offset + k + 2;
    let machine_y = y_offset + k + 3;
    let inserter_out_y = y_offset + k + 3 + msz;
    let out_belt_y = y_offset + k + 3 + msz + 1;
    let row_height = k + 8;

    let belt1_ug = ug_belt_name(belt1);

    let is_dive_col = |x: i32| dive_xs.contains(&x);

    // Lane assignment to trunk rows. We REVERSE the natural order so
    // the trunk closest to iron-plate (y+K-1) is lane 0 and dives at
    // the leftmost column. Each subsequent lane is one row higher and
    // dives one column further east. This means a trunk's dive only
    // ever has to bridge iron-plate — the trunks BELOW it have already
    // dove west (rows are empty), and the trunks ABOVE it are at
    // shallower rows that the dive's south-belts never touch.
    let lane_trunk_row = |b: usize| y_offset + (k - 1 - b as i32);

    // K stacked east-flowing trunks. Lane b's trunk lives at row
    // lane_trunk_row(b) from row_west_x up to dive_xs[b]; past that
    // column it's gone (the items have dove south). At intermediate
    // dive columns of *deeper* lanes (b' < b) the trunk continues
    // surface-east — those dives' south-belts are at rows below this
    // trunk, no entity conflict.
    for (b, &trunk_end_x) in dive_xs.iter().enumerate().take(k_trunks) {
        let trunk_row = lane_trunk_row(b);
        for x in row_west_x..trunk_end_x {
            entities.push(PlacedEntity {
                name: belt0.to_string(),
                x,
                y: trunk_row,
                direction: EntityDirection::East,
                carries: Some(input0.to_string()),
                segment_id: trunk_seg.clone(),
                ..Default::default()
            });
        }
    }

    // Iron-plate (low-demand input₁) continuous belt at y+K, with
    // east-axis UG crosses at each dive column.
    for x in row_west_x..row_east_x_excl {
        if is_dive_col(x) {
            continue;
        }
        let is_ug_input = dive_xs.iter().any(|&dx| x == dx - 1);
        let is_ug_output = dive_xs.iter().any(|&dx| x == dx + 1);
        let ent = if is_ug_input {
            PlacedEntity {
                name: belt1_ug.to_string(),
                x,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt1_seg.clone(),
                io_type: Some("input".to_string()),
                ..Default::default()
            }
        } else if is_ug_output {
            PlacedEntity {
                name: belt1_ug.to_string(),
                x,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt1_seg.clone(),
                io_type: Some("output".to_string()),
                ..Default::default()
            }
        } else {
            PlacedEntity {
                name: belt1.to_string(),
                x,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt1_seg.clone(),
                ..Default::default()
            }
        };
        entities.push(ent);
    }

    // Trunk dives. Lane b dives at column dive_xs[b], starting at its
    // trunk row `lane_trunk_row(b)` (= y+K-1-b) and descending through
    // any rows of deeper lanes (already empty — those lanes dove west)
    // plus iron-plate at y+K, landing on current-feed at y+K+1 via a
    // south→east corner.
    //   (dx, lane_trunk_row(b))           : SOUTH belt corner.
    //   (dx, lane_trunk_row(b)+1..y+K)    : SOUTH belts.
    //   (dx, y+K+1)                       : EAST belt corner — start of
    //                                       current-feed segment b.
    // Per B11 the south→east turn preserves both lanes, so the trunk's
    // full belt capacity feeds straight into current-feed.
    let _ = trunk_y;
    for (b, &dx) in dive_xs.iter().enumerate() {
        let trunk_row = lane_trunk_row(b);
        // South corner at the trunk's row.
        entities.push(PlacedEntity {
            name: belt0.to_string(),
            x: dx,
            y: trunk_row,
            direction: EntityDirection::South,
            carries: Some(input0.to_string()),
            segment_id: dive_seg.clone(),
            ..Default::default()
        });
        // South belts down to (and including) iron-plate row.
        for y in (trunk_row + 1)..=(y_offset + k) {
            entities.push(PlacedEntity {
                name: belt0.to_string(),
                x: dx,
                y,
                direction: EntityDirection::South,
                carries: Some(input0.to_string()),
                segment_id: dive_seg.clone(),
                ..Default::default()
            });
        }
        // East corner at current-feed row — first tile of segment b.
        entities.push(PlacedEntity {
            name: belt0.to_string(),
            x: dx,
            y: cf_y,
            direction: EntityDirection::East,
            carries: Some(input0.to_string()),
            segment_id: cf_seg.clone(),
            ..Default::default()
        });
    }

    // Current-feed segments per block. The east-corner above stamps
    // (dive_xs[b], cf_y); fill the rest of segment b only out to the
    // last machine's near-inserter column. Stopping at that column
    // (instead of continuing into the gap before the next dive)
    // prevents a straight-feed merge into the next dive's east-corner,
    // which would otherwise demote that corner's input from B7
    // straight-feed to B8 sideload (near lane only).
    for (b, &dx) in dive_xs.iter().enumerate() {
        let block_count = if b + 1 == n_blocks {
            machine_count - b * block_size
        } else {
            block_size
        };
        // Near-inserter (regular) sits at machine_x for each machine
        // (post-swap). The last current-feed tile we need is the column
        // of the last machine in this block.
        let last_machine_col = dx + 1 + (block_count as i32 - 1) * pitch;
        for x in (dx + 1)..=last_machine_col {
            entities.push(PlacedEntity {
                name: belt0.to_string(),
                x,
                y: cf_y,
                direction: EntityDirection::East,
                carries: Some(input0.to_string()),
                segment_id: cf_seg.clone(),
                ..Default::default()
            });
        }
    }

    // Output lane-balance bridge: with output inserters all dropping
    // South onto the output belt, only the near lane is filled (rule
    // I5/B8). Stamp a 6-tile sideload bridge halfway through the row
    // so both lanes are utilised on the merger-pickup side. Skip when
    // there's only 1 machine — no room to bridge without widening the
    // row, and a 1-machine row at half throughput is acceptable.
    let bridge_anchor: Option<usize> = if machine_count >= 2 {
        Some((machine_count - 1) / 2)
    } else {
        None
    };
    let bridge_x = bridge_anchor.map(|i| machine_xs[i] + 1);
    let bridge_x_skip: rustc_hash::FxHashSet<i32> = match bridge_x {
        Some(bx) => (bx..bx + 3).collect(),
        None => rustc_hash::FxHashSet::default(),
    };

    // Inserters, machines, output inserters per machine.
    //
    // Inserter columns are SWAPPED relative to vertical-split's
    // `dual_input_row`: in horizontal-stack the high-demand input₀
    // sits on the near (current-feed) row at y+K+1 and the low-demand
    // input₁ sits on the far row at y+K. So the regular reach-1
    // inserter goes at machine_x (near, picks input₀) and the
    // long-handed reach-2 inserter goes at machine_x+2 (far, picks
    // input₁). This also keeps current-feed from needing to extend
    // past the last machine's column (see segment loop above).
    //
    // Output inserter sits at mx+1 by default, but for the bridge
    // anchor machine we shift it to mx (leftmost of the machine
    // footprint) so the 3-tile bridge has a clean inserter row.
    // Pickup tile shifts from (mx+1, machine_y+msz-1) to (mx,
    // machine_y+msz-1) — both on the machine boundary. Drop tile
    // shifts from (mx+1, out_belt_y) to (mx, out_belt_y) — both on
    // the row's output belt.
    for (i, &mx) in machine_xs.iter().enumerate() {
        // Near/far contested column: baseline near=dx=0, far=dx=2; the
        // ONE extra column (dx=1) is shared — both belts cover the full
        // row width for every machine (no LastInRow trim in
        // HorizontalStack), so it's always contested here, unlike
        // `dual_input_row`'s LastInRow exception. `docs/rfp-inserter-
        // sizing.md` Phase 3: far's reach-2 count-ladder is ACTIVE.
        const NEAR_BASELINE_DX: i32 = 0;
        const FAR_BASELINE_DX: i32 = 2;
        let shared_dx = free_extra_dx(msz, &[NEAR_BASELINE_DX, FAR_BASELINE_DX]);
        let far_wins = !shared_dx.is_empty() && contest_favors_far(near_rate, far_rate, true);
        let far_extra_dx: Vec<i32> = if far_wins { shared_dx.clone() } else { vec![] };
        let near_extra_dx: Vec<i32> = if far_wins { vec![] } else { shared_dx.clone() };

        let near_plan = size_side(near_rate, Reach::Near, near_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &near_plan,
            mx,
            inserter_in_y,
            EntityDirection::South,
            input0,
            &inserter_in0_seg,
            NEAR_BASELINE_DX,
            &near_extra_dx,
        );
        emit_shortfall_trace(recipe, false, near_rate, &near_plan, mx, machine_y, far_wins);

        // Far input (input1) — ladder-sized (reach-2 count-ladder).
        let far_plan = size_side(far_rate, Reach::Far, far_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &far_plan,
            mx,
            inserter_in_y,
            EntityDirection::South,
            input1,
            &inserter_in1_seg,
            FAR_BASELINE_DX,
            &far_extra_dx,
        );
        emit_shortfall_trace(recipe, false, far_rate, &far_plan, mx, machine_y, !shared_dx.is_empty() && !far_wins);
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output — ladder-sized, single_input_row-output-shaped: baseline
        // shifts to `mx` at the bridge anchor, extra picks exclude the
        // bridge's own columns.
        let is_anchor = Some(i) == bridge_anchor;
        let out_baseline_dx = if is_anchor { 0 } else { 1 };
        let mut out_occupied = vec![out_baseline_dx];
        for dx in 0..msz {
            if bridge_x_skip.contains(&(mx + dx)) {
                out_occupied.push(dx);
            }
        }
        let out_extra_dx = free_extra_dx(msz, &out_occupied);
        let output_plan = size_side(output_rate, Reach::Near, out_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &output_plan,
            mx,
            inserter_out_y,
            EntityDirection::South,
            output_item,
            &inserter_out_seg,
            out_baseline_dx,
            &out_extra_dx,
        );
        emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, machine_y, false);
    }

    // Output belt — single continuous east- (or west-) flowing belt
    // across the whole row, including dive columns. Skip the 3 columns
    // owned by the bridge so its custom directions aren't overwritten
    // by straight-belt placements (templates.rs has no entity dedup).
    let out_dir = output_dir(output_east);
    for x in row_west_x..row_east_x_excl {
        if bridge_x_skip.contains(&x) {
            continue;
        }
        entities.push(PlacedEntity {
            name: output_belt.to_string(),
            x,
            y: out_belt_y,
            direction: out_dir,
            carries: Some(output_item.to_string()),
            segment_id: belt_out_seg.clone(),
            ..Default::default()
        });
    }

    if let Some(bx) = bridge_x {
        entities.extend(sideload_bridge(
            bx,
            y_offset,
            out_belt_y - y_offset,
            output_belt,
            output_item,
            output_east,
            belt_out_seg.clone(),
        ));
    }

    (entities, row_height)
}

/// Map a transport-belt entity name to the matching underground-belt name.
fn ug_belt_name(belt: &str) -> &str {
    match belt {
        "fast-transport-belt" => "fast-underground-belt",
        "express-transport-belt" => "express-underground-belt",
        _ => "underground-belt",
    }
}

/// Row for a recipe with 3 solid inputs.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : input belt 1 (EAST) -- far belt (long-handed reach)
///   y+1 : input belt 2 (EAST) -- close belt (regular reach)
///   y+2 : long-handed-inserter at mx (picks y+0) + inserter at mx+2 (picks y+1)
///   y+3..y+3+msz-1 : machine (msz×msz)
///   y+3+msz : output inserter at mx+1 (SOUTH) + long-handed inserter at mx+2 (NORTH)
///   y+3+msz+1 : output belt (WEST or EAST)
///   y+3+msz+2 : input belt 3 (EAST) -- delivered from south side
/// ```
///
/// When `lane_split=true` (and `machine_count >= 2`), machines are
/// split into two groups with a sideload bridge between them so the
/// output belt uses both lanes. The gap continues all three input
/// belts (y+0, y+1, y+8) so items still reach the second group.
///
/// Returns `(entities, row_height)`.
/// `input_items`/rates: `(far, near, input3)` — `far`/`near` are the
/// reassigned near-far pair (caller uses `inserter_ladder::
/// reassign_near_far`, same as `dual_input_row`); `input3` is fixed
/// reach-2, never reassigned. Rates are per-machine, utilization-scaled.
///
/// Near/far share the same ONE contested column as `dual_input_row`
/// (derived from belt coverage, see there). Input3's own reach-2 ladder
/// and the near-far pair's far side are Phase 3 scope (unchanged,
/// hardcoded) — only near and output are ladder-sized here. Output shares
/// its own extra column with input3's slot (`docs/rfp-inserter-sizing.md`:
/// dx=0 at `ins_y`, free only when input3 hasn't shifted there at the
/// bridge anchor, and dead entirely at the bridge-anchor successor where
/// the bridge itself covers it — both derived from `bridge_x_set`, not a
/// lookup table).
#[allow(clippy::too_many_arguments)]
pub fn triple_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str, &str),
    output_item: &str,
    input_belts: (&str, &str, &str),
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
    far_rate: f64,
    near_rate: f64,
    input3_rate: f64,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "triple_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 6;
    let mut entities = Vec::new();

    let (input1, input2, input3) = input_items;
    let (belt1, belt2, belt3) = input_belts;
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let belt_in3_seg = Some(format!("row:{recipe}:belt-in:{input3}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let inserter_in3_seg = Some(format!("row:{recipe}:inserter-in:{input3}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Belt 1 (far, long-hand at mx): trim msz-1.
    // Belts 2 and 3 (regular at mx+2 / long-hand at mx+2 picking from south): trim 0 for msz=3.
    // West-flow output: trim 1 for msz=3; east-flow output: no trim.
    let in1_tail = east_tail_skip(msz, 0);
    let in2_tail = east_tail_skip(msz, 2);
    let in3_tail = east_tail_skip(msz, 2);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    let lane_split = lane_split && machine_count >= 2;
    // Strategy B: triple_input_row has TWO inserters per machine on
    // `inserter_out_y` (output at mx+1, input-3 long-hand at mx+2),
    // leaving only mx free. Strategy A's single-inserter shift can't
    // free 3 contiguous cols. We add 1 tile of gap east of the anchor
    // machine + shift the anchor's input-3 long-hand from mx+2 to mx;
    // bridge spans cols `[mx_anchor+2, mx_anchor+5)`.
    let bridge_anchor = inline_bridge_anchor(machine_count, lane_split);
    let mxs: Vec<i32> = (0..machine_count)
        .map(|i| {
            let i = i as i32;
            let extra = match bridge_anchor {
                Some(a) if (i as usize) > a => 1, // machines AFTER anchor shift east by 1
                _ => 0,
            };
            x_offset + i * pitch + extra
        })
        .collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");
    let bridge_x_set = bridge_anchor
        .map(|a| inline_bridge_x_set_b(mxs[a]))
        .unwrap_or_default();
    // Continuation cols on input belts (y+0, y+1, belt3_y) at the
    // 1-tile gap east of the anchor machine. Bridge cols on the
    // output-belt row are stamped by the bridge itself, but the input
    // belts pass through this same x and need filling so flow doesn't
    // break.
    let gap_col: Option<i32> = bridge_anchor.map(|a| mxs[a] + pitch);
    let belt3_y = y_offset + 3 + msz + 2;

    for (mi, &mx) in mxs.iter().enumerate() {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let in3_stop = if is_last { msz - in3_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Input belt 1 -- far belt (long-handed range)
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }

        // Input belt 2 -- close belt (regular inserter range)
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: y_offset + 1,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Far input: long-handed, hardcoded (Phase 3 scope, same
        // rationale as `dual_input_row`). Picks from y+0 (input1), drops
        // into machine at y+3.
        // Near/far contested column, same derivation as `dual_input_row`
        // (identical belt/inserter geometry for this pair). `docs/rfp-
        // inserter-sizing.md` Phase 3: far's reach-2 count-ladder is ACTIVE.
        const FAR_BASELINE_DX: i32 = 0;
        const NEAR_BASELINE_DX: i32 = 2;
        let far_candidates = free_extra_dx(in1_stop, &[FAR_BASELINE_DX, NEAR_BASELINE_DX]);
        let near_candidates = free_extra_dx(in2_stop, &[FAR_BASELINE_DX, NEAR_BASELINE_DX]);
        let near_far_eligible = !far_candidates.is_empty();
        let near_far_shared: Vec<i32> =
            near_candidates.iter().copied().filter(|dx| far_candidates.contains(dx)).collect();
        let near_only_dx: Vec<i32> =
            near_candidates.iter().copied().filter(|dx| !near_far_shared.contains(dx)).collect();
        let near_far_far_wins = !near_far_shared.is_empty()
            && contest_favors_far(near_rate, far_rate, near_far_eligible);
        let far_extra_dx: Vec<i32> = if near_far_far_wins { near_far_shared.clone() } else { vec![] };
        let near_extra_dx: Vec<i32> = if near_far_far_wins { near_only_dx } else { near_candidates.clone() };

        // Far input — ladder-sized (reach-2 count-ladder).
        let far_plan = size_side(far_rate, Reach::Far, far_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &far_plan,
            mx,
            y_offset + 2,
            EntityDirection::South,
            input1,
            &inserter_in1_seg,
            FAR_BASELINE_DX,
            &far_extra_dx,
        );
        emit_shortfall_trace(
            recipe, false, far_rate, &far_plan,
            mx, y_offset + 3,
            !near_far_shared.is_empty() && !near_far_far_wins,
        );

        // Near input — ladder-sized.
        let near_plan = size_side(near_rate, Reach::Near, near_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &near_plan,
            mx,
            y_offset + 2,
            EntityDirection::South,
            input2,
            &inserter_in2_seg,
            NEAR_BASELINE_DX,
            &near_extra_dx,
        );
        emit_shortfall_trace(
            recipe, false, near_rate, &near_plan,
            mx, y_offset + 3, near_far_far_wins,
        );

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 3,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Input-3 baseline: shifted from mx+2 to mx at the bridge anchor —
        // picks from (mx, belt3_y) and drops into machine left col (mx,
        // machine_y). This frees mx+2 on inserter_out_y for the bridge.
        let ins_y = y_offset + 3 + msz;
        let in3_x = if Some(mi) == bridge_anchor { mx } else { mx + 2 };
        let in3_baseline_dx = in3_x - mx;

        // Input-3/output contested column: both would use `ins_y`'s free
        // dx (input3 baseline, output baseline at `mx+1`), plus any tile
        // the bridge itself occupies on this row (`bridge_x_set`, the
        // "wall-2" alignment — the bridge's belt lands on `ins_y`
        // exactly). Deriving occupancy this way reproduces "Interior/Last
        // =1 shared, BridgeAnchor=0, Successor=0" from pure geometry: at
        // the anchor, input3's own shift to `mx` already claims dx=0 AND
        // the bridge claims mx+2; at the successor, the bridge's span
        // reaches into its dx=0. Belt3 covers the full row width at every
        // position (`in3_tail` is always 0 for msz=3 — its last-adjacency
        // dx=2 is already the last column), so the shared tile is always
        // within input3's reach when it exists. `docs/rfp-inserter-
        // sizing.md` Phase 3: input3's reach-2 count-ladder is now ACTIVE.
        let mut shared_occupied = vec![1i32, in3_baseline_dx];
        for dx in 0..msz {
            if bridge_x_set.contains(&(mx + dx)) {
                shared_occupied.push(dx);
            }
        }
        let shared_dx = free_extra_dx(msz, &shared_occupied);
        let tile_exists = !shared_dx.is_empty();
        let input3_wins = tile_exists && contest_favors_far(output_rate, input3_rate, true);
        let input3_extra_dx: Vec<i32> = if input3_wins { shared_dx.clone() } else { vec![] };
        let output_extra_dx: Vec<i32> = if input3_wins { vec![] } else { shared_dx };

        // Input-3 — ladder-sized (reach-2 count-ladder).
        let input3_plan = size_side(input3_rate, Reach::Far, input3_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &input3_plan,
            mx,
            ins_y,
            EntityDirection::North,
            input3,
            &inserter_in3_seg,
            in3_baseline_dx,
            &input3_extra_dx,
        );
        emit_shortfall_trace(recipe, false, input3_rate, &input3_plan, mx, y_offset + 3, tile_exists && !input3_wins);

        // Output — ladder-sized.
        let output_plan = size_side(output_rate, Reach::Near, output_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &output_plan,
            mx,
            ins_y,
            EntityDirection::South,
            output_item,
            &inserter_out_seg,
            1,
            &output_extra_dx,
        );
        emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, y_offset + 3, input3_wins);

        // Output belt — skip cols owned by the bridge.
        let out_belt_y = y_offset + 3 + msz + 1;
        let out_dir = output_dir(output_east);
        for dx in 0..out_stop {
            let x = mx + dx;
            if bridge_x_set.contains(&x) {
                continue;
            }
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }

        // Input belt 3 -- south-side belt
        for dx in 0..in3_stop {
            entities.push(PlacedEntity {
                name: belt3.to_string(),
                x: mx + dx,
                y: belt3_y,
                direction: EntityDirection::East,
                carries: Some(input3.to_string()),
                segment_id: belt_in3_seg.clone(),
                ..Default::default()
            });
        }
    }

    // Strategy B's 1-tile gap east of the anchor needs continuation
    // belts on input rows (y+0, y+1, belt3_y) so flow doesn't break.
    // The output-belt row at the gap col is owned by the bridge; the
    // inserter row at the gap col is also covered by the bridge.
    if let Some(gap_x) = gap_col {
        entities.push(PlacedEntity {
            name: belt1.to_string(),
            x: gap_x, y: y_offset,
            direction: EntityDirection::East,
            carries: Some(input1.to_string()),
            segment_id: belt_in1_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: belt2.to_string(),
            x: gap_x, y: y_offset + 1,
            direction: EntityDirection::East,
            carries: Some(input2.to_string()),
            segment_id: belt_in2_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: belt3.to_string(),
            x: gap_x, y: belt3_y,
            direction: EntityDirection::East,
            carries: Some(input3.to_string()),
            segment_id: belt_in3_seg.clone(),
            ..Default::default()
        });
    }

    if let Some(anchor) = bridge_anchor {
        stamp_inline_bridge_b(
            &mut entities,
            mxs[anchor],
            y_offset,
            3 + msz + 1, // output_row_dy
            output_belt,
            output_item,
            output_east,
            belt_out_seg.clone(),
        );
    }

    (entities, row_height)
}

/// Row for a recipe with four solid inputs.
///
/// Stacks three input belts on the north side and one on the south side. The
/// closest north belt (`y+2`) lives on the same row as a pair of stacked
/// long-handed inserters at `mx+1`: belt segments go underground around the
/// LHI tile, and the LHI on the belt row reaches two tiles north to pull
/// from the topmost belt. The fourth input is delivered south-side via the
/// same north-facing LHI pattern that `triple_input_row` uses.
///
/// ```text
///   y+0  : input1 belt (continuous, east)            ← far north, picked by LHI on y+2
///   y+1  : input2 belt (continuous, east)            ← mid,       picked by LHI on y+3
///   y+2  : input3 belt (UG-in / [LHI-A] / UG-out per machine)
///   y+3  : inserter row
///            (mx+0): regular inserter for input3   (drops machine top)
///            (mx+1): LHI for input2                (drops machine middle)
///            (mx+2): regular inserter for input3   (drops machine top)
///   y+4..y+6 : machine (msz=3)
///   y+7  : output inserter (mx+1) + LHI for input4 facing North (mx+2)
///   y+8  : output belt
///   y+9  : input4 belt (continuous, east)            ← south side
/// ```
///
/// Per-machine reach math (msz=3, machine at y+4..y+6):
/// - LHI-A at `(mx+1, y+2)` facing south: picks `(mx+1, y+0)` = belt 1, drops
///   `(mx+1, y+4)` = machine top middle.
/// - LHI-B at `(mx+1, y+3)` facing south: picks `(mx+1, y+1)` = belt 2, drops
///   `(mx+1, y+5)` = machine middle middle. Has to be a long-handed inserter
///   because `(mx+1, y+2)` is LHI-A, not a belt.
/// - Regular inserters at `(mx+0, y+3)` / `(mx+2, y+3)`: pick `(mx, y+2)` =
///   belt 3 surface (UG-in / UG-out tile for input3), drop machine top.
/// - South LHI at `(mx+2, y+7)` facing north: picks `(mx+2, y+9)` = belt 4,
///   drops `(mx+2, y+5)` = machine middle right.
///
/// Per-machine col allocation: 5 inserters per machine (LHI-A + LHI-B at
/// mx+1, regulars at mx+0 and mx+2, output inserter at mx+1, south LHI at
/// mx+2). Drops cover machine top (3 tiles) + machine middle (2 tiles).
///
/// `lane_split` is currently ignored — the inserter row is too dense for
/// the existing inline-bridge anchor patterns. Multi-machine rows still
/// work; they just don't get a sideload bridge.
///
/// Returns `(entities, row_height)`.
/// Inputs 1/2 (north) are dead budget always (`docs/rfp-inserter-sizing.md`:
/// "0 for 3 of 4 inputs, north rows fully packed") — hardcoded, unchanged.
/// Input3 (the dual-baseline pair at `mx+0`/`mx+2`) upgrades both existing
/// regular inserters to the SAME cheapest-sufficient tier for HALF the
/// demand each (the row's two structural pickup points are never removed —
/// "upgrade both", not count-laddered from a 1-inserter baseline). Output
/// and input4 (south) share one extra column (`mx+0` at `south_ins_y`) —
/// `docs/rfp-inserter-sizing.md` Phase 3: input4's reach-2 count-ladder is
/// now ACTIVE, so a win genuinely places a second long-handed inserter.
#[allow(clippy::too_many_arguments)]
pub fn quad_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str, &str, &str),
    output_item: &str,
    input_belts: (&str, &str, &str, &str),
    output_belt: &str,
    _lane_split: bool,
    output_east: bool,
    input3_rate: f64,
    input4_rate: f64,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32) {
    use crate::bus::balancer::underground_for_belt;
    use crate::common::inserter_throughput;

    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "quad_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    // 3 north belts + inserter + msz machine + south inserter + output
    // belt + south input belt = msz + 7.
    let row_height = msz + 7;
    let mut entities = Vec::new();

    let (input1, input2, input3, input4) = input_items;
    let (belt1, belt2, belt3, belt4) = input_belts;
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let belt_in3_seg = Some(format!("row:{recipe}:belt-in:{input3}"));
    let belt_in4_seg = Some(format!("row:{recipe}:belt-in:{input4}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let inserter_in3_seg = Some(format!("row:{recipe}:inserter-in:{input3}"));
    let inserter_in4_seg = Some(format!("row:{recipe}:inserter-in:{input4}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // East-tail trims (last machine only). Belt 1 / 2 are picked at mx+1
    // by long-handed inserters, so the tile at mx+2 is post-pickup orphan.
    // Belt 3 has its UG-out at mx+2 and a regular inserter consumes it
    // there; no trim. Belt 4 is picked at mx+2 by the south LHI; no trim.
    // West-flow output: items end at mx+0, so trim 1 east tile.
    let in1_tail = east_tail_skip(msz, 1);
    let in2_tail = east_tail_skip(msz, 1);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");
    let ug_belt3 = underground_for_belt(belt3);
    let machine_y = y_offset + 4;
    let south_ins_y = machine_y + msz;
    let out_belt_y = south_ins_y + 1;
    let belt4_y = out_belt_y + 1;

    for &mx in &mxs {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Belt 1 (top, y+0)
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }

        // Belt 2 (middle, y+1)
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: y_offset + 1,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Belt 3 (close, y+2): UG-IN at mx+0, LHI-A occupies mx+1, UG-OUT
        // at mx+2. The 1-tile UG span is well within transport-belt's
        // 4-tile reach. Adjacent machines' UG-OUT abuts the next machine's
        // UG-IN, so item flow continues across the row.
        entities.push(PlacedEntity {
            name: ug_belt3.to_string(),
            x: mx,
            y: y_offset + 2,
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            carries: Some(input3.to_string()),
            segment_id: belt_in3_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: ug_belt3.to_string(),
            x: mx + 2,
            y: y_offset + 2,
            direction: EntityDirection::East,
            io_type: Some("output".to_string()),
            carries: Some(input3.to_string()),
            segment_id: belt_in3_seg.clone(),
            ..Default::default()
        });

        // LHI-A at (mx+1, y+2) facing south: picks belt 1 at y+0,
        // drops machine top y+4.
        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: mx + 1,
            y: y_offset + 2,
            direction: EntityDirection::South,
            carries: Some(input1.to_string()),
            segment_id: inserter_in1_seg.clone(),
            ..Default::default()
        });

        // LHI-B at (mx+1, y+3) facing south: picks belt 2 at y+1
        // (reaching over LHI-A at y+2), drops machine middle y+5.
        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: mx + 1,
            y: y_offset + 3,
            direction: EntityDirection::South,
            carries: Some(input2.to_string()),
            segment_id: inserter_in2_seg.clone(),
            ..Default::default()
        });

        // Input3 (dual-baseline) at mx+0 and mx+2, y+3: pick belt 3
        // surface tile at y+2 (UG-IN at mx+0, UG-OUT at mx+2), drop
        // machine top y+4. Both slots are structural (the UG belt's two
        // exposed pickup points), never removed — ladder-sized as ONE
        // in-place tier upgrade applied to both, at half the demand each
        // (cheapest-sufficient for `input3_rate/2`, mirrored to both
        // tiles) rather than a count-ladder from a 1-inserter baseline.
        let input3_per_slot = input3_rate / 2.0;
        let input3_plan = size_side(input3_per_slot, Reach::Near, 0, max_inserter_tier);
        for &dx in &[0i32, 2] {
            entities.push(PlacedEntity {
                name: input3_plan.entity.to_string(),
                x: mx + dx,
                y: y_offset + 3,
                direction: EntityDirection::South,
                carries: Some(input3.to_string()),
                segment_id: inserter_in3_seg.clone(),
                ..Default::default()
            });
        }
        {
            let total_avail = 2.0 * inserter_throughput(input3_plan.entity);
            let shortfall = (input3_rate - total_avail).max(0.0);
            if shortfall > 0.0 {
                // This site sizes per-slot (rate/2 mirrored to two fixed
                // structural tiles), so `capped_limit`'s plan-derived
                // budget doesn't apply; classify directly: both slots are
                // structural and count can't grow, so the only actionable
                // constraint is the tier ceiling.
                let limit = if input3_plan.entity != crate::bus::inserter_ladder::STACK
                    && input3_rate <= 2.0 * inserter_throughput(crate::bus::inserter_ladder::STACK)
                {
                    "tier-cap"
                } else {
                    "geometry"
                };
                crate::trace::emit(crate::trace::TraceEvent::InserterSideCapped {
                    recipe: recipe.to_string(),
                    side_is_output: false,
                    required: input3_rate,
                    placed_entity: input3_plan.entity.to_string(),
                    placed_count: 2,
                    shortfall,
                    machine_x: mx,
                    machine_y,
                    limit: limit.to_string(),
                });
            }
        }

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output/input4 contested column (relative dx=0, i.e. `mx+0`).
        // `docs/rfp-inserter-sizing.md` Phase 3: input4's reach-2 count-
        // ladder is now ACTIVE — a win genuinely places a second LHI.
        let input4_wins = contest_favors_far(output_rate, input4_rate, true);
        let output_extra_dx: Vec<i32> = if input4_wins { vec![] } else { vec![0] };
        let input4_extra_dx: Vec<i32> = if input4_wins { vec![0] } else { vec![] };

        // Output — ladder-sized.
        let output_plan = size_side(output_rate, Reach::Near, output_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &output_plan,
            mx,
            south_ins_y,
            EntityDirection::South,
            output_item,
            &inserter_out_seg,
            1,
            &output_extra_dx,
        );
        emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, machine_y, input4_wins);

        // South input4 — ladder-sized (reach-2 count-ladder). Baseline at
        // mx+2 (picks belt 4 at y+9, drops machine middle mx+2,y+5).
        let input4_plan = size_side(input4_rate, Reach::Far, input4_extra_dx.len(), max_inserter_tier);
        stamp_side_inserters(
            &mut entities,
            &input4_plan,
            mx,
            south_ins_y,
            EntityDirection::North,
            input4,
            &inserter_in4_seg,
            2,
            &input4_extra_dx,
        );
        emit_shortfall_trace(recipe, false, input4_rate, &input4_plan, mx, machine_y, !input4_wins);

        // Output belt
        let out_dir = output_dir(output_east);
        for dx in 0..out_stop {
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x: mx + dx,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }

        // Belt 4 (south input)
        for dx in 0..msz {
            entities.push(PlacedEntity {
                name: belt4.to_string(),
                x: mx + dx,
                y: belt4_y,
                direction: EntityDirection::East,
                carries: Some(input4.to_string()),
                segment_id: belt_in4_seg.clone(),
                ..Default::default()
            });
        }
    }

    (entities, row_height)
}

/// Row for a recipe with 1 solid input + 1 fluid input.
///
/// For chemical-plant, uses the simple single-fluid pattern: a continuous
/// east-west pipe line on `y+0` spans the full machine width and carries
/// fluid to each machine's port via a short UG tunnel under the solid belt.
/// ```text
///   y+0 : pipe — pipe — pipe — ...            ← continuous east-west pipe row
///   y+1 : empty — UG pipe IN (SOUTH) — empty  ← dedicated UG-in row at port_x
///   y+2 : solid input belt (EAST, msz wide)
///   y+3 : UG pipe OUT (NORTH, port_x) + inserter (mx+1)
///   y+4..y+4+msz-1 : machine (msz×msz)
///   y+4+msz : output inserter (SOUTH)
///   y+4+msz+1 : output belt (WEST or EAST)
/// ```
/// The `y+0` pipe row gives machine-to-machine connectivity for free (all
/// machines share one fluid, so there's no isolation concern) and the bus
/// router just has to extend it west/east until it hits the fluid trunk
/// column. Multi-fluid-per-side rows need the richer UG-pipe-UG isolation
/// pattern; see `docs/archive/rfp-multi-fluid-rows.md` for that.
///
/// For other machines (assembling-machine-2/3 with fluid): uses a regular pipe
/// at the port position:
/// ```text
///   y+0 : solid input belt (EAST, msz wide)
///   y+1 : inserter (solid) + pipe at (mx+port_dx)
///   y+2..y+2+msz-1 : machine (msz×msz)
///   y+2+msz : output inserter (SOUTH)
///   y+2+msz+1 : output belt (WEST or EAST)
/// ```
///
/// Returns `(entities, row_height, fluid_port_pipes)` where
/// `fluid_port_pipes` is a list of `(x, y)` giving the bus tap-point for each
/// machine's fluid connection (a tile on the port-adjacent pipe row for
/// chemical-plant, the pipe tile for other machines).
/// `solid_rate`/`output_rate` are per-machine, utilization-scaled. Solid
/// input is ladder-sized: one extra column exists at Interior (the third
/// dx not claimed by the inserter or the fluid port), trimmed away at
/// LastInRow (both `port_dx` variants — `in_tail` traps the free column's
/// belt tile), reproducing the frozen table's "0 at LastInRow" cell from
/// geometry. Output reuses `single_input_row`'s output-shaped derivation.
#[allow(clippy::too_many_arguments)]
pub fn fluid_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    solid_item: &str,
    fluid_item: &str,
    output_item: &str,
    input_belt: &str,
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
    solid_rate: f64,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "fluid_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    let port_dx = fluid_input_port_dx(machine_entity);
    let mut entities = Vec::new();
    let mut fluid_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let belt_in_seg = Some(format!("row:{recipe}:belt-in:{solid_item}"));
    let inserter_in_seg = Some(format!("row:{recipe}:inserter-in:{solid_item}"));
    let fluid_in_seg = Some(format!("row:{recipe}:belt-in:{fluid_item}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Solid input belt pickup at mx+1, west-flow output drop at mx+1 → trim 1 tile each.
    let in_tail = east_tail_skip(msz, 1);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    // Strategy A: pack tight, shift anchor's output inserter, inline bridge.
    let lane_split = lane_split && machine_count >= 2;
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");

    let bridge_anchor = inline_bridge_anchor(machine_count, lane_split);
    let bridge_x_set = bridge_anchor
        .map(|a| inline_bridge_x_set_a(mxs[a]))
        .unwrap_or_default();

    // Inserter column: must be distinct from the fluid-port column
    // (`port_dx`) so pipe and inserter never share a tile. chemical-plant
    // has `port_dx == 0` so the inserter sits at `mx + 1`; AM2/AM3 have
    // `port_dx == 1` so the inserter sits at `mx + 0`.
    let inserter_dx: i32 = if port_dx == 0 { 1 } else { 0 };
    // `in_tail` captures the current hard-coded assumption that the solid
    // belt is picked at `mx + 1`. Post-unification the actual pickup
    // column is `mx + inserter_dx`, so recompute the trim to match.
    let _ = in_tail;
    let in_tail = east_tail_skip(msz, inserter_dx);

    {
        // Unified T-junction pattern (formerly chemical-plant-only):
        // continuous east-west pipe line at y+0, a dedicated UG-in row at
        // y+1, the solid belt at y+2, and UG-out + inserter at y+3 adjacent
        // to the machine port. The same layout works for both chemical-
        // plant (`port_dx == 0`, inserter at `mx + 1`) and AM2/AM3
        // (`port_dx == 1`, inserter at `mx + 0`): the UG pair gives
        // machine-to-machine fluid connectivity via the y+0 header pipe
        // regardless of which column the fluid port sits on.
        //
        //   y+0: pipe row (msz tiles per machine, continuous across the row).
        //         Bus router extends this line west/east to the fluid trunk.
        //   y+1: UG pipe IN (SOUTH) at (port_x, y+1); rest of row empty.
        //   y+2: solid belt (crosses above the UG tunnel)
        //   y+3: UG pipe OUT (NORTH) at (port_x, y+3) + inserter at (mx+inserter_dx)
        //   y+4..y+6: machine (3×3)
        //   y+7: output inserter
        //   y+8: output belt
        let row_height = msz + 6;
        let belt_y = y_offset + 2;
        let interface_y = y_offset + 3; // UG pipe OUT + inserter row
        let machine_y = y_offset + 4;
        let out_ins_y = machine_y + msz;
        let out_belt_y = machine_y + msz + 1;
        let out_dir = output_dir(output_east);

        for (mi, &mx) in mxs.iter().enumerate() {
            let is_last = mx == last_mx;
            let in_stop = if is_last { msz - in_tail } else { msz };
            let out_stop = if is_last { msz - out_tail } else { msz };

            // y+0: continuous east-west pipe line spanning the machine's
            // full width. Adjacent machines' pipes abut so the whole row
            // forms one pipe network, giving machine-to-machine fluid
            // connectivity. The bus router extends this line to the trunk.
            for dx in 0..msz {
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: y_offset,
                    carries: Some(fluid_item.to_string()),
                    segment_id: fluid_in_seg.clone(),
                    ..Default::default()
                });
            }

            // y+1: UG pipe IN facing south — enters tunnel, connects to the
            // T-junction pipe above and tunnels under the solid belt at y+2.
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: mx + port_dx,
                y: y_offset + 1,
                direction: EntityDirection::South,
                io_type: Some("input".to_string()),
                carries: Some(fluid_item.to_string()),
                segment_id: fluid_in_seg.clone(),
                ..Default::default()
            });

            // y+2: solid input belt (msz tiles wide)
            for dx in 0..in_stop {
                entities.push(PlacedEntity {
                    name: input_belt.to_string(),
                    x: mx + dx,
                    y: belt_y,
                    direction: EntityDirection::East,
                    carries: Some(solid_item.to_string()),
                    segment_id: belt_in_seg.clone(),
                    ..Default::default()
                });
            }

            // y+3: UG pipe OUT facing north (back toward input) adjacent to machine fluid port
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: mx + port_dx,
                y: interface_y,
                direction: EntityDirection::North,
                io_type: Some("output".to_string()),
                carries: Some(fluid_item.to_string()),
                segment_id: fluid_in_seg.clone(),
                ..Default::default()
            });
            // Solid input — ladder-sized. Baseline at `inserter_dx`
            // (distinct from the fluid UG at `port_dx`); the ONE extra
            // column is the third dx (neither inserter_dx nor port_dx),
            // free only when `in_stop` still covers it (Interior) —
            // reproduces the frozen "0 at LastInRow" cell from geometry.
            let solid_extra_dx = free_extra_dx(in_stop, &[inserter_dx, port_dx]);
            let solid_plan = size_side(solid_rate, Reach::Near, solid_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &solid_plan,
                mx,
                interface_y,
                EntityDirection::South,
                solid_item,
                &inserter_in_seg,
                inserter_dx,
                &solid_extra_dx,
            );
            emit_shortfall_trace(recipe, false, solid_rate, &solid_plan, mx, machine_y, false);

            // Machine
            entities.push(PlacedEntity {
                name: machine_entity.to_string(),
                x: mx,
                y: machine_y,
                direction: EntityDirection::North,
                recipe: Some(recipe.to_string()),
                segment_id: machine_seg.clone(),
                ..Default::default()
            });

            // Output — ladder-sized, single_input_row-output-shaped:
            // baseline shifts to `mx` at the bridge anchor.
            let is_anchor = Some(mi) == bridge_anchor;
            let out_baseline_dx = if is_anchor { 0 } else { 1 };
            let mut out_occupied = vec![out_baseline_dx];
            for dx in 0..msz {
                if bridge_x_set.contains(&(mx + dx)) {
                    out_occupied.push(dx);
                }
            }
            let out_extra_dx = free_extra_dx(out_stop, &out_occupied);
            let output_plan = size_side(output_rate, Reach::Near, out_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &output_plan,
                mx,
                out_ins_y,
                EntityDirection::South,
                output_item,
                &inserter_out_seg,
                out_baseline_dx,
                &out_extra_dx,
            );
            emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, machine_y, false);

            // Output belt — skip cols owned by the bridge.
            for dx in 0..out_stop {
                let x = mx + dx;
                if bridge_x_set.contains(&x) {
                    continue;
                }
                entities.push(PlacedEntity {
                    name: output_belt.to_string(),
                    x,
                    y: out_belt_y,
                    direction: out_dir,
                    carries: Some(output_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }

            // Report a tile on the pipe row (y+0) as the bus tap-point for
            // each machine. The whole row at y+0 is pipe, so the bus router
            // can connect its horizontal branch to any x between mx and
            // mx+msz-1; we pick port_x for consistency with the UG column.
            fluid_port_pipes.push((fluid_item.to_string(), mx + port_dx, y_offset));
        }

        // Inline bridge on output belt + inserter row above. Machines
        // pack tight so the y+0 pipe header is naturally continuous;
        // the y+2 solid belt likewise. PTG / inserter rows above are
        // unaffected.
        if let Some(anchor) = bridge_anchor {
            stamp_inline_bridge_a(
                &mut entities,
                mxs[anchor],
                y_offset,
                out_belt_y - y_offset,
                output_belt,
                output_item,
                output_east,
                belt_out_seg.clone(),
            );
        }

        (entities, row_height, fluid_port_pipes)
    }
}

/// Row for a recipe with 2 solid inputs + 1 fluid input.
///
/// Fluid is delivered via a horizontal pipe header ABOVE the machine row,
/// with vertical pipe-to-ground tunnels per machine dropping fluid down to
/// the machine's fluid input port. This frees the inserter row for two inserters.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : horizontal fluid header (pipes carrying fluid_item)
///   y+1 : pipe-to-ground input at mx+port_dx (direction SOUTH)
///   y+2 : solid input belt 1 (EAST) -- far belt
///   y+3 : solid input belt 2 (EAST) -- close belt
///   y+4 : long-handed-inserter at mx+1 + inserter at mx+2 +
///           pipe-to-ground output at mx+port_dx (direction NORTH, faces input)
///   y+5..y+5+msz-1 : machine (msz×msz)
///   y+5+msz : fluid output pipes (if output_is_fluid) OR output inserter
///   y+5+msz+1 : output belt (solid output only)
/// ```
///
/// Returns `(entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)`.
/// `solid_items`/rates are `(far, near)` — already assigned by the caller
/// via `inserter_ladder::reassign_near_far` (v3's extension of the lever to
/// this row kind). Both solid inserters have a HARD 0 extra-column budget
/// at every position (2 solid inserters + the fluid PTG pack all 3 dx of
/// `inserter_y`) — no contest needed, both sides are simple in-place tier
/// upgrades. Solid output (when `!output_is_fluid`) reuses
/// `single_input_row`'s output-shaped derivation; fluid output uses a
/// continuous pipe row and has no inserter to size.
#[allow(clippy::too_many_arguments)]
pub fn fluid_dual_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    solid_items: (&str, &str),
    fluid_item: &str,
    output_item: &str,
    output_is_fluid: bool,
    input_belts: (&str, &str),
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
    far_rate: f64,
    near_rate: f64,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "fluid_dual_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    // Fluid output occupies y+5+msz; add a trailing empty row so sub-row
    // stacking doesn't put output pipes adjacent to the next sub-row's
    // fluid header row (which would trip pipe-isolation).
    let row_height = msz + 7;
    let mut entities = Vec::new();

    let (input1, input2) = solid_items;
    let (belt1, belt2) = input_belts;
    let port_dx = fluid_input_port_dx(machine_entity);
    let fluid_in_seg = Some(format!("row:{recipe}:belt-in:{fluid_item}"));
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    let header_y = y_offset;
    let ptg_in_y = y_offset + 1;
    let belt1_y = y_offset + 2;
    let belt2_y = y_offset + 3;
    let inserter_y = y_offset + 4;
    let machine_y = y_offset + 5;
    let output_y = y_offset + 5 + msz;

    // Lane-split only makes sense when the output belt is actually a belt
    // (not a fluid pipe row). Fluid outputs go through the y+msz+5 pipe
    // row, which doesn't benefit from `sideload_bridge`.
    let lane_split = lane_split && machine_count >= 2 && !output_is_fluid;
    // Strategy A: pack tight, shift anchor's output inserter, inline bridge.
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");

    let bridge_anchor = inline_bridge_anchor(machine_count, lane_split);
    let bridge_x_set = bridge_anchor
        .map(|a| inline_bridge_x_set_a(mxs[a]))
        .unwrap_or_default();

    // Horizontal fluid header chain: spans x_offset .. last machine's mx+(msz-1).
    // Machines are now packed tight, so the header naturally runs continuous.
    let header_end_x = last_mx + msz - 1;
    for x in x_offset..=header_end_x {
        entities.push(PlacedEntity {
            name: "pipe".to_string(),
            x,
            y: header_y,
            carries: Some(fluid_item.to_string()),
            segment_id: fluid_in_seg.clone(),
            ..Default::default()
        });
    }

    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();

    // Inserter placement branches on port_dx:
    //   port_dx=0 (chemical-plant): long_x=mx+1, reg_x=mx+2.
    //   port_dx=1 (assembling-machine-2/3): long_x=mx+2, reg_x=mx.
    // Belt 1 (far, y+2) is picked by the long-handed inserter; belt 2 (close,
    // y+3) by the regular inserter.
    let (long_dx, reg_dx) = if port_dx == 1 { (2, 0) } else { (1, 2) };
    let in1_tail = east_tail_skip(msz, long_dx);
    let in2_tail = east_tail_skip(msz, reg_dx);
    // Solid output (output_is_fluid=false) has its drop at mx+1 → trim 1 for msz=3 west-flow.
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    for (mi, &mx) in mxs.iter().enumerate() {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Vertical PTG pair: input at y+1 tunnels SOUTH to output at y+4
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: mx + port_dx,
            y: ptg_in_y,
            direction: EntityDirection::South,
            io_type: Some("input".to_string()),
            carries: Some(fluid_item.to_string()),
            segment_id: fluid_in_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: mx + port_dx,
            y: inserter_y,
            direction: EntityDirection::North,
            io_type: Some("output".to_string()),
            carries: Some(fluid_item.to_string()),
            segment_id: fluid_in_seg.clone(),
            ..Default::default()
        });

        // Solid input belts (machine_size tiles wide each)
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: belt2_y,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Inserter placement depends on which column the fluid PTG occupies.
        // port_dx == 0 (chemical-plant): PTG at mx+0, inserters at mx+1 (long) and mx+2 (regular).
        // port_dx == 1 (assembling-machine-2/3): PTG at mx+1, so move the
        //   long-handed inserter to mx+2 and the regular inserter to mx+0.
        let long_x = mx + long_dx;
        let reg_x = mx + reg_dx;

        // Both solid sides — ladder-sized, hard 0 extra-column budget (2
        // solid inserters + the fluid PTG pack all 3 dx of `inserter_y`,
        // no free tile at any position) — simple in-place tier upgrades,
        // no contest needed.
        let far_plan = size_side(far_rate, Reach::Far, 0, max_inserter_tier);
        entities.push(PlacedEntity {
            name: far_plan.entity.to_string(),
            x: long_x,
            y: inserter_y,
            direction: EntityDirection::South,
            carries: Some(input1.to_string()),
            segment_id: inserter_in1_seg.clone(),
            ..Default::default()
        });
        emit_shortfall_trace(recipe, false, far_rate, &far_plan, mx, machine_y, false);

        let near_plan = size_side(near_rate, Reach::Near, 0, max_inserter_tier);
        entities.push(PlacedEntity {
            name: near_plan.entity.to_string(),
            x: reg_x,
            y: inserter_y,
            direction: EntityDirection::South,
            carries: Some(input2.to_string()),
            segment_id: inserter_in2_seg.clone(),
            ..Default::default()
        });
        emit_shortfall_trace(recipe, false, near_rate, &near_plan, mx, machine_y, false);

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output row
        if output_is_fluid {
            // Continuous pipe row one tile south of the machine spanning the
            // full machine width. Chemical-plant's two output fluid boxes sit
            // at dx=0 and dx=2, both of which this pipe row covers. Adjacent
            // machines' rows abut, giving machine-to-machine connectivity and
            // a clean run out to the bus trunk for downstream consumers.
            for dx in 0..msz {
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: output_y,
                    carries: Some(output_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }
            fluid_output_port_pipes.push((output_item.to_string(), mx, output_y));
            fluid_output_port_pipes.push((output_item.to_string(), mx + 2, output_y));
        } else {
            // Solid output — ladder-sized, single_input_row-output-shaped:
            // baseline shifts to `mx` at the bridge anchor, extra picks
            // (into `output_y+1`'s belt range) exclude the bridge's own
            // columns.
            let is_anchor = Some(mi) == bridge_anchor;
            let out_baseline_dx = if is_anchor { 0 } else { 1 };
            let mut out_occupied = vec![out_baseline_dx];
            for dx in 0..msz {
                if bridge_x_set.contains(&(mx + dx)) {
                    out_occupied.push(dx);
                }
            }
            let out_extra_dx = free_extra_dx(out_stop, &out_occupied);
            let output_plan = size_side(output_rate, Reach::Near, out_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &output_plan,
                mx,
                output_y,
                EntityDirection::South,
                output_item,
                &inserter_out_seg,
                out_baseline_dx,
                &out_extra_dx,
            );
            emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, machine_y, false);
            let out_dir = output_dir(output_east);
            for dx in 0..out_stop {
                let x = mx + dx;
                if bridge_x_set.contains(&x) {
                    continue;
                }
                entities.push(PlacedEntity {
                    name: output_belt.to_string(),
                    x,
                    y: output_y + 1,
                    direction: out_dir,
                    carries: Some(output_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }
        }
    }

    // Stamp the inline bridge on the solid output belt + the inserter
    // row above it. Pipe header (y+0), PTG (y+1, y+4), and solid input
    // belts (y+2, y+3) sit above the bridge rows and are not affected
    // by the inserter shift or bridge stamps. Only fires when
    // output_is_fluid=false (guarded by `lane_split` already).
    if let Some(anchor) = bridge_anchor {
        let output_row_dy = 5 + msz + 1;
        stamp_inline_bridge_a(
            &mut entities,
            mxs[anchor],
            y_offset,
            output_row_dy,
            output_belt,
            output_item,
            output_east,
            belt_out_seg.clone(),
        );
    }

    let fluid_input_port_pipes = vec![(fluid_item.to_string(), x_offset, header_y)];

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Column pitch between consecutive machines in a fluid-only row (issue
/// #277). At `machine_size` pitch, adjacent machines' PTGs on the
/// ≥3-distinct-fluid-output staircase collide (machine N's east R-flank at
/// `mx + msz` lands on the same tile as machine N+1's west drop UG at
/// `mx + msz`); adding 1 tile of pitch separates them and keeps them
/// F5a-isolated. Used by both `fluid_only_row_staggered_3output` (to space
/// machine stamps) and `placer.rs`'s row-width calculation (which must
/// agree, or the row's reserved width undercounts).
pub(crate) fn fluid_only_row_pitch(machine_size: u32, distinct_fluid_outputs: usize) -> i32 {
    let msz = machine_size as i32;
    if distinct_fluid_outputs >= 3 {
        msz + 1
    } else {
        msz
    }
}

/// Staggered 3-output layout for advanced-oil-processing-style recipes on 5×5
/// refineries. Each output fluid gets its own full-width trunk at a distinct
/// y row so flanks and drop-UGs of adjacent fluids never share a tile.
///
/// Output staircase ordering (from the user's sketch in `misc/Untitled.png`):
/// - **West** (smallest dx, fluid_outputs[0]) → **lowest** (largest y)
/// - **Middle** (middle dx, fluid_outputs[1]) → **highest** (smallest y,
///   adjacent to the port row)
/// - **East** (largest dx, fluid_outputs[2]) → middle y
///
/// Inputs use a vertically mirrored staircase: when ≥2 distinct fluids are
/// consumed, each gets its own full-width trunk row above the machine plus a
/// drop UG into the port row, same UG-pipe-UG / perpendicular-axis F5a
/// isolation as the output side. With 1 fluid input the whole input side
/// collapses to a single continuous strip (same as the non-staggered path).
///
/// For refinery output ports at `dx = [0, 2, 4]` and 1 fluid input
/// (basic-oil-processing-style), y_offset = 0, a single machine:
///
/// ```text
///   y+0          : fluid input pipe strip (continuous)
///   y+1..y+5     : oil-refinery (mirror=true, dir=North)
///   y+6          : output port row — west UG-S/in, middle pipe, east UG-S/in
///   y+7 (middle) : middle L-flank · T-drop · R-flank  plus east drop UG at col_E
///   y+8 (east)   : east L-flank · T-drop · R-flank    plus west drop UG at col_W
///   y+9 (west)   : west L-flank · T-drop · R-flank
/// ```
///
/// For 2 fluid inputs at dx `[a, b]` (e.g. AOP: crude-oil dx=3, water dx=1),
/// the input side grows by 4 extra rows. Same UG-pipe-UG / drop-UG-N pattern
/// as `fluid_multi_input_row`, including the 1-tile gap row above the inner
/// trunk that protects against adjacent bus-lane collisions:
///
/// ```text
///   y+0           : outer trunk — UG-W · pipe · UG-E at col_b
///   y+1           : outer drop UG-S at col_b (gap row otherwise)
///   y+2           : inner trunk — UG-W · pipe · UG-E at col_a
///   y+3           : inner drop UG-S at col_a
///   y+4           : input port row — UG-N at col_a and col_b (surface-S into
///                   the machine's north fluid ports)
///   y+5..y+9      : oil-refinery
///   y+10..y+13    : output port row + 3-trunk staircase as above
/// ```
///
/// Perpendicular-axis PTG rule (F5a) keeps fluids isolated where a drop UG
/// passes the flank column of a neighbouring trunk. Adjacent input ports
/// (`port_dx` differing by exactly 1) would put the inner trunk's R-flank on
/// the same tile as the outer drop UG, so the staircase asserts spacing ≥2.
///
/// Row height = `msz + 5` for 1 fluid input, `msz + 9` for 2 distinct inputs
/// (general formula: `msz + 5 + n_inputs + 2` for n ≥ 2 with the gap row).
///
/// ## Multi-machine (issue #277)
///
/// Machines repeat every [`fluid_only_row_pitch`] columns instead of every
/// `msz` columns. At `pitch == msz`, machine N's east R-flank PTG at
/// `(mx + msz, y_east)` collides with machine N+1's west drop UG at the same
/// tile; `fluid_only_row_pitch` adds a 1-tile gap for the ≥3-distinct-output
/// case, which separates the two and keeps them F5a-isolated (no shared
/// tile between the perpendicular-oriented PTGs). At `machine_count == 1`
/// the pitch is unused, so output is byte-identical to the single-machine
/// layout.
fn fluid_only_row_staggered_3output(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    fluid_inputs: &[(i32, &str)],
    fluid_outputs: &[(i32, &str)],
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    assert_eq!(
        fluid_outputs.len(), 3,
        "fluid_only_row_staggered_3output requires exactly 3 output ports",
    );
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "fluid_only_row_staggered_3output assumes square machines; see rfp-fulgora-scrap Phase 0"
    );

    let msz = machine_size as i32;
    assert!(msz >= 5, "staggered 3-output requires a 5×5+ machine");

    let mut entities = Vec::new();
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let machine_seg = Some(format!("row:{recipe}:machine"));

    // Pitch is a no-op (== msz) at machine_count == 1, so the single-machine
    // output stays byte-identical to the pre-#277 layout.
    let pitch = fluid_only_row_pitch(machine_size, fluid_outputs.len());
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();

    let input_distinct: std::collections::BTreeSet<&str> =
        fluid_inputs.iter().map(|&(_, it)| it).collect();
    let n_inputs = input_distinct.len() as i32;

    // For 2+ distinct fluids we add a 1-tile gap above the inner trunk so the
    // bus router has a clear column tile to bridge through when an outer
    // fluid's bus lane is adjacent to an inner fluid's bus lane (same
    // reasoning as `fluid_multi_input_row`'s gap=1 — bus lane adjacency
    // can't be predicted at template-emission time, so we always pay the
    // 1-tile cost).
    //
    // input_section_height counts: outer trunk + outer drop + gap row + inner
    // trunk + inner drop + UG-out row = n + 2 + gap.
    let gap: i32 = if n_inputs >= 2 { 1 } else { 0 };
    let input_section_height: i32 = if n_inputs <= 1 { 1 } else { n_inputs + 2 + gap };
    let machine_y = y_offset + input_section_height;
    let port_row_y = machine_y + msz;

    // ≥2 distinct fluid inputs: sort by port_dx ascending so fi=0 = smallest
    // dx = innermost. Same for every machine stamp, so sort once up front.
    let mut by_dx: Vec<(i32, &str)> = fluid_inputs.to_vec();
    by_dx.sort_by_key(|&(dx, _)| dx);
    let input_port_y = machine_y - 1;
    let n = n_inputs;

    // Adjacent ports (dx delta == 1) would put the inner trunk's R-flank
    // on the outer drop UG's tile. Refinery uses dx∈{1,3} (delta 2), so
    // this is fine in practice; assert to flag any future caller that
    // breaks the assumption.
    if n_inputs >= 2 {
        for w in by_dx.windows(2) {
            assert!(
                (w[1].0 - w[0].0).abs() >= 2,
                "fluid_only_row_staggered_3output multi-fluid input requires \
                 port_dx values to differ by ≥2 (got {} and {})",
                w[0].0, w[1].0,
            );
        }
    }

    // Output staircase dx/item assignments — invariant across machines.
    // Order: fluid_outputs sorted by dx ascending — [0]=west, [1]=middle, [2]=east.
    // (Caller in placer.rs already passes them in this order for refinery outputs.)
    let (dx_w, item_w) = fluid_outputs[0];
    let (dx_m, item_m) = fluid_outputs[1];
    let (dx_e, item_e) = fluid_outputs[2];

    let y_middle = port_row_y + 1;
    let y_east = port_row_y + 2;
    let y_west = port_row_y + 3;

    let seg_w = Some(format!("row:{recipe}:fluid-out:{item_w}"));
    let seg_m = Some(format!("row:{recipe}:fluid-out:{item_m}"));
    let seg_e = Some(format!("row:{recipe}:fluid-out:{item_e}"));

    for &mx in &mxs {
        // --- Input side. ---
        if n_inputs <= 1 {
            // 1 distinct fluid (or 0): continuous pipe strip across the machine
            // width. Same as the non-staggered single-fluid path.
            if let Some(&(_, item)) = fluid_inputs.first() {
                let seg = Some(format!("row:{recipe}:belt-in:{item}"));
                for dx in 0..msz {
                    entities.push(PlacedEntity {
                        name: "pipe".to_string(),
                        x: mx + dx, y: y_offset,
                        carries: Some(item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                for &(dx, port_item) in fluid_inputs {
                    fluid_input_port_pipes.push((port_item.to_string(), mx + dx, y_offset));
                }
            }
        } else {
            // ≥2 distinct fluid inputs: vertically mirrored output staircase with
            // the same gap-row protection as `fluid_multi_input_row`. Each fluid
            // gets its own trunk row (UG-W · pipe · UG-E), a drop UG one row
            // below the trunk, and a UG-out at the port row; the port row sits
            // directly above the machine.
            for (fi, &(port_dx, item)) in by_dx.iter().enumerate() {
                let fi = fi as i32;
                // Outermost fluid (fi=n-1) anchors at y_offset+0; inner fluids
                // shift down by `gap` so the outer fluid's drop UG lands in the
                // empty gap row.
                let trunk_y = y_offset + (n - 1 - fi) + if fi < n - 1 { gap } else { 0 };
                let col_t = mx + port_dx;
                let seg = Some(format!("row:{recipe}:fluid-in:{item}"));

                // Trunk row: UG-W · pipe · UG-E (same shape as output trunk rows).
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x: col_t - 1, y: trunk_y,
                    direction: EntityDirection::West,
                    io_type: Some("output".to_string()),
                    carries: Some(item.to_string()),
                    segment_id: seg.clone(),
                    ..Default::default()
                });
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: col_t, y: trunk_y,
                    carries: Some(item.to_string()),
                    segment_id: seg.clone(),
                    ..Default::default()
                });
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x: col_t + 1, y: trunk_y,
                    direction: EntityDirection::East,
                    io_type: Some("input".to_string()),
                    carries: Some(item.to_string()),
                    segment_id: seg.clone(),
                    ..Default::default()
                });

                // Drop UG-S one row below the trunk row. Surface-N merges with
                // T-drop above (same fluid). Tunnel goes south to the UG-out.
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x: col_t, y: trunk_y + 1,
                    direction: EntityDirection::South,
                    io_type: Some("input".to_string()),
                    carries: Some(item.to_string()),
                    segment_id: seg.clone(),
                    ..Default::default()
                });

                // UG-out on port row, direction=North so surface-S merges with
                // the machine's north fluid port at (col_t, machine_y).
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x: col_t, y: input_port_y,
                    direction: EntityDirection::North,
                    io_type: Some("output".to_string()),
                    carries: Some(item.to_string()),
                    segment_id: seg.clone(),
                    ..Default::default()
                });

                // Tap point: the trunk's T-drop pipe. The bus router emits an
                // east-facing UG at `(lane.x + 1, trunk_y)` to pair with the
                // L-flank UG-W at `(col_t - 1, trunk_y)`.
                fluid_input_port_pipes.push((item.to_string(), col_t, trunk_y));
            }
        }

        // --- Machine. ---
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            mirror: true,
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // --- Output side: staggered 3 trunks. ---
        let col_w = mx + dx_w;
        let col_m = mx + dx_m;
        let col_e = mx + dx_e;

        // Port row — one tile per fluid, each connecting to the refinery's
        // corresponding fluid box. West and east use UG-S/input so the tunnel
        // continues south to the trunk's drop UG. Middle uses a regular pipe
        // since its trunk is directly below (0-tile gap).
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_w, y: port_row_y,
            direction: EntityDirection::South,
            io_type: Some("input".to_string()),
            carries: Some(item_w.to_string()),
            segment_id: seg_w.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe".to_string(),
            x: col_m, y: port_row_y,
            carries: Some(item_m.to_string()),
            segment_id: seg_m.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_e, y: port_row_y,
            direction: EntityDirection::South,
            io_type: Some("input".to_string()),
            carries: Some(item_e.to_string()),
            segment_id: seg_e.clone(),
            ..Default::default()
        });

        // Middle trunk at y_middle (adjacent to port row).
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_m - 1, y: y_middle,
            direction: EntityDirection::West,
            io_type: Some("output".to_string()),
            carries: Some(item_m.to_string()),
            segment_id: seg_m.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe".to_string(),
            x: col_m, y: y_middle,
            carries: Some(item_m.to_string()),
            segment_id: seg_m.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_m + 1, y: y_middle,
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            carries: Some(item_m.to_string()),
            segment_id: seg_m.clone(),
            ..Default::default()
        });

        // East drop UG (at y_middle row, col_e) — pairs with machine-side UG at port row.
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_e, y: y_east - 1, // = y_middle
            direction: EntityDirection::North,
            io_type: Some("output".to_string()),
            carries: Some(item_e.to_string()),
            segment_id: seg_e.clone(),
            ..Default::default()
        });

        // East trunk at y_east.
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_e - 1, y: y_east,
            direction: EntityDirection::West,
            io_type: Some("output".to_string()),
            carries: Some(item_e.to_string()),
            segment_id: seg_e.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe".to_string(),
            x: col_e, y: y_east,
            carries: Some(item_e.to_string()),
            segment_id: seg_e.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_e + 1, y: y_east,
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            carries: Some(item_e.to_string()),
            segment_id: seg_e.clone(),
            ..Default::default()
        });

        // West drop UG (at y_east row, col_w) — pairs with machine-side UG at port row,
        // tunnel spans y_middle in between (perpendicular to middle trunk's PTGs, F5a).
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_w, y: y_west - 1, // = y_east
            direction: EntityDirection::North,
            io_type: Some("output".to_string()),
            carries: Some(item_w.to_string()),
            segment_id: seg_w.clone(),
            ..Default::default()
        });

        // West trunk at y_west.
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_w - 1, y: y_west,
            direction: EntityDirection::West,
            io_type: Some("output".to_string()),
            carries: Some(item_w.to_string()),
            segment_id: seg_w.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe".to_string(),
            x: col_w, y: y_west,
            carries: Some(item_w.to_string()),
            segment_id: seg_w.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: col_w + 1, y: y_west,
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            carries: Some(item_w.to_string()),
            segment_id: seg_w.clone(),
            ..Default::default()
        });

        // Tap points at each trunk's T-drop — the ghost router connects the bus
        // lanes for each output fluid to these positions.
        fluid_output_port_pipes.push((item_w.to_string(), col_w, y_west));
        fluid_output_port_pipes.push((item_m.to_string(), col_m, y_middle));
        fluid_output_port_pipes.push((item_e.to_string(), col_e, y_east));
    }

    // Row spans y_offset (input row) down to y_west.
    let row_height = y_west - y_offset + 1; // = msz + 5

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Row for fluid-only recipes on large machines (5×5: oil-refinery, cryogenic-plant, foundry).
///
/// Machines are placed at `direction=NORTH` with `mirror=true` so
/// fluid inputs sit at the NORTH edge (matching the bus trunk-above
/// pattern) and fluid outputs sit at the SOUTH edge.
///
/// `fluid_inputs` and `fluid_outputs` specify the port assignments as
/// `(dx_from_machine_left_edge, item_name)` pairs.  For oil-refinery (mirrored):
/// - Input ports are at dx=1 (box 1) and dx=3 (box 2).
/// - Output ports are at dx=0 (box 3), dx=2 (box 4), and dx=4 (box 5).
///
/// ## Single-fluid-per-side (simple pattern)
///
/// When all inputs carry the same fluid (len ≤ 1) and all outputs carry the
/// same fluid (len ≤ 1), each side gets a continuous east-west pipe row
/// spanning the full machine width. Adjacent machines' rows abut, giving
/// machine-to-machine connectivity and a straight run to the bus trunk with
/// no per-port isolation needed.
///
/// ```text
///   y+0     : pipe ── pipe ── pipe ── ...        ← continuous input pipe row
///   y+1..y+msz : machine (msz×msz, mirrored)
///   y+msz+1 : pipe ── pipe ── pipe ── ...        ← continuous output pipe row
/// ```
///
/// ## Multi-fluid-per-side (isolated pipes)
///
/// When a side has ≥2 distinct fluids, we fall back to per-port isolated
/// pipes — a continuous row would merge them and violate F3 (fluid isolation).
/// The proper multi-fluid pattern is the stacked-T design (see
/// `docs/archive/rfp-multi-fluid-rows.md`); until that lands, these rows will not
/// connect to the bus.
///
/// Returns `(entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)`.
/// Port pipe lists have the form `(item, x, y)`.
pub fn fluid_only_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    fluid_inputs: &[(i32, &str)],   // (dx_from_machine_left, item_name) per input port
    fluid_outputs: &[(i32, &str)],  // (dx_from_machine_left, item_name) per output port
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "fluid_only_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    // Is this row carrying a single fluid on each side? The continuous-pipe
    // simplification only applies when a side has 0 or 1 fluid.
    let input_distinct_items: std::collections::BTreeSet<&str> =
        fluid_inputs.iter().map(|&(_, it)| it).collect();
    let output_distinct_items: std::collections::BTreeSet<&str> =
        fluid_outputs.iter().map(|&(_, it)| it).collect();

    // 3 distinct fluid outputs: switch to the staggered 3-trunk staircase
    // layout (advanced-oil-processing, coal-liquefaction). Each output gets
    // its own trunk row below the machine so flanks and drop-UGs don't
    // share tiles.
    //
    // Multi-machine is supported (issue #277): the template spaces machine
    // stamps by `fluid_only_row_pitch` instead of `machine_size`, which
    // separates machine N's east R-flank from machine N+1's west drop UG
    // (they'd otherwise collide at the same tile at plain `machine_size`
    // pitch). `placer.rs`'s row-width calculation uses the same pitch
    // helper so the reserved row width agrees.
    if output_distinct_items.len() >= 3 {
        return fluid_only_row_staggered_3output(
            recipe, machine_entity, machine_size, machine_count,
            y_offset, x_offset, fluid_inputs, fluid_outputs,
        );
    }

    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 2;
    let mut entities = Vec::new();
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let machine_seg = Some(format!("row:{recipe}:machine"));

    let single_input_fluid = input_distinct_items.len() <= 1;
    let single_output_fluid = output_distinct_items.len() <= 1;

    let input_y = y_offset;
    let output_y = y_offset + 1 + msz;

    for i in 0..machine_count {
        let mx = x_offset + i as i32 * pitch;

        if single_input_fluid {
            // Continuous input pipe row spanning the machine's full width.
            // Even when the recipe uses only one input port (e.g. dx=3 for
            // basic-oil-processing's crude-oil), the extra pipe tiles touch
            // inactive fluid boxes which Factorio simply ignores.
            //
            // For 5×5 oil-refinery rows the continuous strip leaves no free
            // tile at `input_y` for power poles — the only y where a
            // medium-electric-pole reaches the machine center. Bridge the
            // strip with a 1-tile UG pair at dx=1 and dx=3 (the two
            // physical fluid input port positions per `fluid_ports`),
            // leaving dx=2 free for `place_poles` to drop a pole that
            // covers each refinery's center. dx=0 and dx=4 stay as regular
            // pipes so adjacent machines' strips connect on the surface.
            if let Some(&(_, item)) = fluid_inputs.first() {
                let seg = Some(format!("row:{recipe}:belt-in:{item}"));
                let gap_dx: Option<i32> = if msz == 5 && machine_entity == "oil-refinery" {
                    Some(2)
                } else {
                    None
                };
                for dx in 0..msz {
                    if Some(dx) == gap_dx {
                        continue;
                    }
                    let (name, direction, io_type) = if Some(dx + 1) == gap_dx {
                        ("pipe-to-ground", EntityDirection::East, Some("input".to_string()))
                    } else if Some(dx - 1) == gap_dx {
                        ("pipe-to-ground", EntityDirection::West, Some("output".to_string()))
                    } else {
                        ("pipe", EntityDirection::North, None)
                    };
                    entities.push(PlacedEntity {
                        name: name.to_string(),
                        x: mx + dx,
                        y: input_y,
                        direction,
                        io_type,
                        carries: Some(item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                if gap_dx == Some(2) {
                    // Report only dx=1 (the leftmost UG-bridge port) so the
                    // pole-tap reservation around the lane→port path
                    // (`hi - 1` in `layout.rs`) doesn't claim the dx=2 gap
                    // tile before `place_poles` runs. The dx=3 port is
                    // still a `pipe-to-ground` so the validator sees an
                    // adjacent pipe at both physical input ports, and
                    // fluid reaches it via the underground tunnel.
                    fluid_input_port_pipes.push((item.to_string(), mx + 1, input_y));
                } else {
                    for &(dx, port_item) in fluid_inputs {
                        fluid_input_port_pipes.push((port_item.to_string(), mx + dx, input_y));
                    }
                }
            }
        } else {
            // Multi-fluid side: keep per-port isolated pipes until the
            // stacked-T multi-fluid pattern lands.
            for &(dx, item) in fluid_inputs {
                let seg = Some(format!("row:{recipe}:belt-in:{item}"));
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: input_y,
                    carries: Some(item.to_string()),
                    segment_id: seg,
                    ..Default::default()
                });
                fluid_input_port_pipes.push((item.to_string(), mx + dx, input_y));
            }
        }

        // Machine direction=NORTH. Mirror only matters for oil-refinery (it
        // swaps input/output faces — `validate/fluids.rs::fluid_ports` lists
        // OIL_MIRROR vs OIL). Chemical-plant ports are symmetric across the
        // mirror axis, but we keep `mirror=false` to match the unmirrored
        // chemical-plant geometry used elsewhere in the codebase.
        let machine_mirror = machine_entity == "oil-refinery";
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 1,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            mirror: machine_mirror,
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        if single_output_fluid {
            if let Some(&(_, item)) = fluid_outputs.first() {
                let seg = Some(format!("row:{recipe}:belt-out:{item}"));
                for dx in 0..msz {
                    entities.push(PlacedEntity {
                        name: "pipe".to_string(),
                        x: mx + dx,
                        y: output_y,
                        carries: Some(item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                for &(dx, port_item) in fluid_outputs {
                    fluid_output_port_pipes.push((port_item.to_string(), mx + dx, output_y));
                }
            }
        } else {
            for &(dx, item) in fluid_outputs {
                let seg = Some(format!("row:{recipe}:belt-out:{item}"));
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: output_y,
                    carries: Some(item.to_string()),
                    segment_id: seg,
                    ..Default::default()
                });
                fluid_output_port_pipes.push((item.to_string(), mx + dx, output_y));
            }
        }
    }

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Multi-fluid input row: machines that consume ≥2 distinct fluids on the
/// same face. Uses the stacked-T pattern from `docs/archive/rfp-multi-fluid-rows.md`
/// with UG-pipe-UG isolation flanks.
///
/// Currently handles **2 fluid inputs on a 3×3 chemical-plant** with no solid
/// input (heavy-oil-cracking, light-oil-cracking, sulfur). Solid output (sulfur)
/// and fluid output (cracking) both supported.
///
/// ## Geometry for N=2 fluids, chemical-plant, ports at dx=0 (fluid 0) and dx=2 (fluid 1)
///
/// ```text
/// y+0:      UG(1)e  pipe(1)  UG(1)w                  ← fluid 1 trunk (outer; T-drop at mx+2)
/// y+1:  UG(0)e  pipe(0)  UG(0)w  UG(1)s              ← fluid 0 trunk + fluid 1 drop
/// y+2:         UG(0)s                                ← fluid 0 drop
/// y+3:         UG-out(0)n        UG-out(1)n          ← surface south, adjacent to machine port
/// y+4..y+6:    ▓ machine 3×3 ▓
/// y+7..       [output inserter + belt for solid output, or pipe row for fluid output]
/// ```
///
/// Direction suffixes: `e/w/n/s` = East/West/North/South. Trunk-row UG flanks
/// face toward their T-drop so their surface-sides merge with the pipe (same
/// fluid). Drop UGs face SOUTH (surface north joins T-drop above, tunnel south
/// to UG-out below). UG-outs face NORTH (surface south joins machine port).
///
/// ## Isolation
///
/// Per [F5a](../../../../docs/factorio-mechanics.md), PTG's perpendicular sides
/// have no surface. Fluid A's east-west-oriented right flank at `(col_0+1, y+1)`
/// meets fluid B's north-south-oriented drop UG at `(col_1, y+1)` edge-to-edge
/// with no surface on either side — no cross-fluid merge.
///
/// ## Trunk feed
///
/// Caller is responsible for routing the fluid trunk at `lane.x` across to the
/// left flank of each T-drop stamp (ghost_router step 3.6 + an additional
/// horizontal UG pair). This template returns per-fluid tap positions so the
/// router can target each fluid's trunk row at the correct y.
///
/// ## Multi-machine
///
/// For `machine_count > 1`, the stamp repeats per machine. Between machines the
/// trunk on each fluid's trunk row continues east via another UG pair.
///
/// Returns `(entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)`.
/// 0 solid inputs by construction (`classify_row_kind`'s `FluidMultiInput`
/// guard) — nothing to ladder on the input side. The solid output (when
/// present) IS ladder-eligible (`docs/rfp-inserter-sizing.md` Phase 3, per
/// `phase_for`'s frozen mapping — not explicitly named in the phase
/// prose, but the frozen mapping takes priority, same precedent as the
/// Phase 2 far-ladder deviation). Unlike `single_input_row`'s output, this
/// row never trims its output belt by position (no `is_last`/`_tail`
/// logic exists anywhere in this template — verified by inspection), so
/// the extra-column budget is a uniform 2 free columns at every position
/// except where the bridge intersects (derived from `bridge_x_set`, not
/// `single_input_row`'s position-dependent LastInRow asymmetry).
#[allow(clippy::too_many_arguments)]
pub fn fluid_multi_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    fluid_inputs: &[(i32, &str)],     // (port_dx, item) — must have ≥2 distinct items
    solid_output_item: Option<&str>,
    fluid_outputs: &[(i32, &str)],    // (port_dx, item) for fluid output(s)
    output_belt: Option<&str>,
    lane_split: bool,
    output_east: bool,
    output_rate: f64,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    assert!(fluid_inputs.len() >= 2, "fluid_multi_input_row requires ≥2 fluid inputs");
    assert!(
        solid_output_item.is_some() || !fluid_outputs.is_empty(),
        "fluid_multi_input_row requires either a solid output or a fluid output",
    );
    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "fluid_multi_input_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );

    let msz = machine_size as i32;
    let pitch = msz;
    let n = fluid_inputs.len() as i32;

    // Always-gap=1 for n≥2 fluids. Two structural reasons:
    //
    // 1. Adjacent-port collision: when two fluids' `port_dx` values differ
    //    by exactly 1, the outer fluid's drop UG at `(col_t_outer,
    //    y_offset+1)` lands on the inner fluid's flank UG at
    //    `(col_t_inner ± 1, y_offset+1)`. The gap row gives the outer
    //    drop UG somewhere to sit that isn't a flank.
    //
    // 2. Bus-router lane adjacency: when the outer fluid's bus column is
    //    adjacent to the inner fluid's bus column (outer.x = inner.x + 1),
    //    the bus router places the inner fluid's branch UG-East at
    //    (inner.x + 1, inner_trunk_y) — i.e. inside the outer fluid's
    //    column. Without a gap row above the inner trunk, the outer
    //    fluid has no clear tile in its column to bridge from its own
    //    tap row down to its UG-S input. Inserting a gap gives the
    //    outer fluid a clear bridging tile at (outer.x, y_offset + 1).
    //
    // We can't predict bus column adjacency at template-emission time
    // (lane_order runs after row placement), so we always pay the
    // 1-tile cost. The existing N≥3 path with adjacent port_dx still
    // needs cumulative gap accounting; that case is unreachable for
    // currently tested recipes.
    let gap: i32 = if n >= 2 {
        1
    } else {
        0
    };
    if n > 2
        && fluid_inputs
            .windows(2)
            .any(|w| (w[1].0 - w[0].0).abs() == 1)
    {
        todo!("3+ fluid inputs with adjacent port_dx — separate fix pending");
    }

    // Row layout (offsets from y_offset), with gap=1 case noted inline:
    //   0              : outermost trunk (fluid_inputs[n-1])
    //   1              : gap row when gap=1 (otherwise inner trunk starts here)
    //   1+gap..n+gap-1 : inner trunk rows (fluid_inputs[n-2..0])
    //   n+gap          : drop UG row for the innermost fluid
    //   n+1+gap        : UG-out row (adjacent to machine port)
    //   n+2+gap..      : machine (msz rows), then output inserter + belt
    //                    (solid out) or single output pipe row (fluid out).
    //
    // With gap=0 this matches the original dense layout. With gap=1 the
    // inner trunk and everything below slide down by one; the outer trunk
    // stays at y_offset+0. Visually equivalent to "move the outer T one
    // tile higher".
    let ug_out_row_idx = n + 1 + gap;
    let machine_row_idx = n + 2 + gap;
    let output_first_idx = machine_row_idx + msz;

    let row_height = if solid_output_item.is_some() {
        output_first_idx + 2   // inserter + belt
    } else {
        output_first_idx + 1   // pipe row only
    };

    // Per-fluid trunk y positions. The outermost (fi == n-1) stays
    // anchored at y_offset+0; inner fluids shift down by `gap` so the
    // outer fluid's drop UG lands in the empty gap row.
    let trunk_ys: Vec<i32> = (0..fluid_inputs.len())
        .map(|fi| {
            let fi = fi as i32;
            y_offset
                + (n - 1 - fi)
                + if fi < n - 1 { gap } else { 0 }
        })
        .collect();

    let mut entities = Vec::new();
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Lane-split only applies to solid-output recipes (the fluid-output path
    // has no belt to sideload onto). The `LANE_SPLIT_GAP` (3) between groups
    // widens each fluid's per-machine trunk-row UG pair from its original
    // adjacent layout (distance 1) to distance `pitch + LANE_SPLIT_GAP - 2
    // = 4`, well under `FLUID_UG_MAX_DISTANCE` (10) — so the fluid trunks
    // tunnel across the gap with no additional gap-filling entities on the
    // trunk rows. The drop-UG / UG-out / machine rows have no inter-machine
    // entities, so the gap stays empty there too. The bridge goes on the
    // output belt row.
    let lane_split = lane_split && machine_count >= 2 && solid_output_item.is_some();
    // Strategy A: pack tight, shift anchor's output inserter, inline bridge.
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| x_offset + i * pitch).collect();

    let bridge_anchor = inline_bridge_anchor(machine_count, lane_split);
    let bridge_x_set = bridge_anchor
        .map(|a| inline_bridge_x_set_a(mxs[a]))
        .unwrap_or_default();

    for (mi, &mx) in mxs.iter().enumerate() {
        // Stamp each fluid's T-drop + trunk flanks + drop UG.
        //
        // Fluid index `fi` in `fluid_inputs`. fi=0 is innermost (closest to
        // machine, lowest trunk row = y_offset + n - 1); fi=n-1 is outermost
        // (topmost, y_offset + 0).
        for (fi, &(port_dx, item)) in fluid_inputs.iter().enumerate() {
            let trunk_y = trunk_ys[fi];
            let col_t = mx + port_dx;
            let fluid_seg = Some(format!("row:{recipe}:fluid-in:{item}"));

            // Trunk row stamp: UG, pipe, UG at (col_t - 1), (col_t), (col_t + 1).
            // Left flank faces WEST (tunnel west, surface east connects to T-drop).
            // Right flank faces EAST (tunnel east, surface west connects to T-drop).
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t - 1,
                y: trunk_y,
                direction: EntityDirection::West,
                io_type: Some("output".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                name: "pipe".to_string(),
                x: col_t,
                y: trunk_y,
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t + 1,
                y: trunk_y,
                direction: EntityDirection::East,
                io_type: Some("input".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });

            // Drop UG at one row below this fluid's trunk row, at col_t.
            // Direction=SOUTH so surface faces north (merges with T-drop pipe
            // above, same fluid) and tunnel goes south to the UG-out.
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t,
                y: trunk_y + 1,
                direction: EntityDirection::South,
                io_type: Some("input".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });

            // UG-out on the machine-adjacent row, direction=NORTH so surface
            // faces south (merges with machine port at (col_t, machine_y)).
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t,
                y: y_offset + ug_out_row_idx,
                direction: EntityDirection::North,
                io_type: Some("output".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });

            // Report the T-drop pipe as the tap point for ghost router step 3.6.
            // Router emits the horizontal feed UG pair from lane.x to col_t - 1.
            fluid_input_port_pipes.push((item.to_string(), col_t, trunk_y));
        }

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + machine_row_idx,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output: solid output uses inserter + belt; fluid output uses a pipe row.
        if let Some(out_item) = solid_output_item {
            let belt_name = output_belt.unwrap_or("transport-belt");
            let ins_y = y_offset + output_first_idx;
            let belt_y = ins_y + 1;
            // Output — ladder-sized: baseline shifts to `mx` at the
            // bridge anchor, extra picks (uniform 2 free columns, no
            // position-based trim — see the doc comment above) exclude
            // the bridge's own columns.
            let is_anchor = Some(mi) == bridge_anchor;
            let out_baseline_dx = if is_anchor { 0 } else { 1 };
            let mut out_occupied = vec![out_baseline_dx];
            for dx in 0..msz {
                if bridge_x_set.contains(&(mx + dx)) {
                    out_occupied.push(dx);
                }
            }
            let out_extra_dx = free_extra_dx(msz, &out_occupied);
            let output_plan = size_side(output_rate, Reach::Near, out_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &output_plan,
                mx,
                ins_y,
                EntityDirection::South,
                out_item,
                &inserter_out_seg,
                out_baseline_dx,
                &out_extra_dx,
            );
            emit_shortfall_trace(recipe, true, output_rate, &output_plan, mx, y_offset + machine_row_idx, false);
            // Output belt row — skip cols owned by the bridge.
            let out_dir = output_dir(output_east);
            for dx in 0..msz {
                let x = mx + dx;
                if bridge_x_set.contains(&x) {
                    continue;
                }
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x,
                    y: belt_y,
                    direction: out_dir,
                    carries: Some(out_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }
        } else {
            // Fluid output: continuous pipe row (single-fluid output branch)
            let pipe_y = y_offset + output_first_idx;
            if let Some(&(_, out_item)) = fluid_outputs.first() {
                let seg = Some(format!("row:{recipe}:fluid-out:{out_item}"));
                for dx in 0..msz {
                    entities.push(PlacedEntity {
                        name: "pipe".to_string(),
                        x: mx + dx,
                        y: pipe_y,
                        carries: Some(out_item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                for &(dx, item) in fluid_outputs {
                    fluid_output_port_pipes.push((item.to_string(), mx + dx, pipe_y));
                }
            }
        }
    }

    // Inline bridge on the solid output belt. Pipe trunk rows (above
    // the machine) and the fluid PTGs sit far above the bridge rows,
    // unaffected by the inserter shift.
    if let Some(anchor) = bridge_anchor {
        let belt_name = output_belt.unwrap_or("transport-belt");
        let solid_out = solid_output_item.expect("lane_split guard ensures solid output");
        let output_row_dy = output_first_idx + 1;
        stamp_inline_bridge_a(
            &mut entities,
            mxs[anchor],
            y_offset,
            output_row_dy,
            belt_name,
            solid_out,
            output_east,
            belt_out_seg.clone(),
        );
    }

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Row for a self-loop recipe (kovarex-class: an item appears on both
/// sides of the recipe). `spec.self_loop` carries the RAW per-machine
/// consumed/produced rates for each self-referencing item; `spec.inputs`
/// /`spec.outputs` carry only the NET flow (see `models::MachineSpec`
/// doc comment). This template physically reconciles the two: a "loop
/// corridor" recirculates the majority of each self-loop item's
/// production back into its own consumption, and the bus is tapped only
/// for the net external demand.
///
/// Two shapes are supported (`docs/rfp-solver-net-flow.md` Phase 2(c)):
/// - **1-item** (bacteria cultivations): one self-loop item with net
///   production (`major_item`), plus an ordinary solid input
///   (nutrients-class, `near_item`) that is NOT self-referencing.
/// - **2-item** (kovarex): a net-positive `major_item` (U-235) and a
///   net-negative minor item that is the SAME item as `near_item`
///   (U-238) — the bus taps `near_item` for its net demand on one
///   belt, and a THIRD north belt (`near2`) independently recirculates
///   the minor item's raw production. Keeping these on separate belts
///   avoids needing a shared merge point for the bus tap and the
///   corridor (a shared merge was explored and found geometrically
///   intractable within a single row's footprint — every column at the
///   near belt's row is claimed either by the continuous bus-tapped
///   run or by a per-machine inserter, and the corridor's return path
///   would sideload into a spot the router expects to extend from,
///   colliding with the tap-off gap the router owns).
///
/// Layout (msz=3, 2-item shape; 1-item drops the `near2` belt, its
/// stacked inserter row collapses into a single `dy_ins`, and there is
/// no `near2` collector/corridor):
/// ```text
///   y+0  : far corridor return (major item, flows WEST, then turns
///          south at machine 0's column)
///   y+1  : (major corridor descent transit, machine-0 column only)
///   y+2  : far belt (major item; fed ONLY by its corridor, no bus tap)
///   y+3  : near belt (bus-tapped net demand)
///   y+4  : near2 belt (minor item; UG-in / LHI-A / UG-out per machine)
///   y+5  : LHI-B (picks near) / regular×2 (pick near2 surface); ALSO
///          near2's own corridor return-transit row west of machine 0
///          (that column is unused by any machine there)
///   y+6..y+6+msz-1 : machine
///   y+6+msz        : output inserters (regular picks major, LHI picks minor)
///   y+6+msz+1      : major collector (raw production; feeds a priority
///                    splitter east of the machines)
///   y+6+msz+2      : minor collector (raw production; 100% recirculated,
///                    no splitter)
///   y+6+msz+3      : near2 corridor east-side transit row
/// ```
/// East of the machines: major's collector feeds a priority splitter
/// (`loop_priority_rate = major_consumed_rate * count`); the export
/// branch becomes the row's declared output (untouched item-isolation
/// segment), the loop branch jogs one column east (clearing the export
/// tile, which sits directly north of the splitter's loop output),
/// rises, and returns via y+0/y+1 into the far belt. Minor's collector
/// (2-item shape only) turns south, transits east past major's
/// splitter zone, rises on its own column, returns west at row y+5,
/// and drops one tile into y+4 via a sideload into machine 0's UG-in —
/// crossing major's riser with a single 1-tile underground hop, the
/// only crossing the two corridors need.
///
/// `fluid_input` — `Some((item, per_machine_rate))` — adds a single
/// non-self-loop fluid ingredient (pentapod-egg's water, fish-breeding's
/// water) via a header row directly above the machine (`dy_machine - 1`).
/// The near-item's inserter also lives on this row (it can no longer
/// reach the machine from `dy_ins1` once the header pushes the machine
/// one row further south), so the header is a per-machine
/// pipe-to-ground bridge — not a plain continuous strip — hopping over
/// the inserter's column; see the "Fluid header" section below for the
/// exact tile layout and the pairing-direction rationale. Only legal
/// with `has_minor == false` (asserted) — the 2-item (kovarex) shape has
/// no fluid-header row. Inserts one extra row between the inserter row
/// and the machine row, so every dy offset from `dy_machine` down shifts
/// by 1 relative to the no-fluid 1-item shape.
///
/// Returns `(entities, row_height, fluid_input_port_pipes)` — the third
/// element mirrors `fluid_input_row`'s convention (one `(item, x, y)` tap
/// point per machine on the header row), empty when `fluid_input` is `None`.
///
/// Ladder-eligible sides (`docs/rfp-inserter-sizing.md` Phase 3): only
/// `near_item`'s own inserter and the output side(s) are check-visible
/// (present in `spec.inputs`/`spec.outputs`) — major's INPUT demand
/// (`major_consumed_rate`) and, in the has-minor shape, minor's INPUT
/// demand both live in `spec.self_loop`, invisible to
/// `check_inserter_throughput`'s required-rate math (the RFP's documented,
/// deliberately-unfixed known ceiling). Their inserters stay hardcoded.
/// Near's reach varies by shape: `Reach::Near` (regular) in the no-fluid/
/// no-minor shape, `Reach::Far` (LHI) in both the has-fluid and has-minor
/// shapes — always with a hard 0 extra-column budget except the no-fluid/
/// no-minor shape's near (1 extra, uncontested — major's true demand is
/// unmeasured, credited optimistically per the census). Major's output is
/// uncontested (2 extra) except in the has-minor shape, where it shares
/// one column with minor's export output (contested, `contest_favors_far`).
#[allow(clippy::too_many_arguments)]
pub fn self_loop_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    major_item: &str,
    major_consumed_rate: f64,
    major_produced_rate: f64,
    near_item: &str,
    near_net_rate: f64,
    minor: Option<(f64, f64)>,
    fluid_input: Option<(&str, f64)>,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>) {
    use crate::bus::balancer::{splitter_for_belt, underground_for_belt};
    use crate::common::belt_entity_for_rate;

    debug_assert_eq!(
        crate::common::machine_dims(machine_entity),
        (machine_size, machine_size),
        "self_loop_row assumes square machines; see rfp-fulgora-scrap Phase 0"
    );
    let msz = machine_size as i32;
    let pitch = msz;
    let count = machine_count as i32;
    let has_minor = minor.is_some();
    let has_fluid = fluid_input.is_some();
    assert!(
        !has_fluid || !has_minor,
        "self_loop_row: fluid_input is only supported for the 1-item shape (has_minor == false)"
    );

    // Prefix zone: 3 dead columns west of machine 0. `x_offset + 1`
    // (= `mx0 - 2`) is the near2 corridor's vertical descent column
    // (2-item shape only) — `near_belt` dips underground for that one
    // tile so the descent has a clear surface path. `mx0` itself
    // carries the major corridor's descent.
    const PREFIX: i32 = 3;
    let mx0 = x_offset + PREFIX;
    let mxs: Vec<i32> = (0..machine_count as i32).map(|i| mx0 + i * pitch).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");
    let east_x = last_mx + msz;

    let major_total = major_produced_rate * count as f64;
    let major_loop_rate = major_consumed_rate * count as f64;
    let major_export_rate = (major_total - major_loop_rate).max(0.0);
    let near_total = (near_net_rate * count as f64).max(1e-6);
    let minor_total = minor.map(|(_, p)| p * count as f64).unwrap_or(0.0).max(1e-6);

    let far_belt = belt_entity_for_rate(major_total, max_belt_tier);
    let near_belt_name = belt_entity_for_rate(near_total, max_belt_tier);
    let near2_belt = belt_entity_for_rate(minor_total, max_belt_tier);
    let collector_belt = belt_entity_for_rate(major_total, max_belt_tier);
    let minor_collector_belt = belt_entity_for_rate(minor_total, max_belt_tier);
    let splitter_name = splitter_for_belt(collector_belt);
    let near2_ug = underground_for_belt(near2_belt);

    let mut entities: Vec<PlacedEntity> = Vec::new();

    let loop_major_seg = Some(format!("row:{recipe}:selfloop:{major_item}"));
    let collect_major_seg = Some(format!("row:{recipe}:belt-out"));
    let near_in_seg = Some(format!("row:{recipe}:belt-in:{near_item}"));
    let near_ins_seg = Some(format!("row:{recipe}:inserter-in:{near_item}"));
    let far_ins_seg = Some(format!("row:{recipe}:inserter-in:{major_item}"));
    let loop_minor_seg = Some(format!("row:{recipe}:selfloop:{near_item}:recirc"));
    let near2_ins_seg = Some(format!("row:{recipe}:inserter-in:{near_item}:recirc"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let major_out_ins_seg = Some(format!("row:{recipe}:inserter-out:{major_item}"));
    let minor_out_ins_seg = Some(format!("row:{recipe}:inserter-out:{near_item}"));
    let fluid_hdr_seg = fluid_input.map(|(item, _)| format!("row:{recipe}:belt-in:{item}"));
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();

    // ---- Row Y offsets (relative to y_offset) ----
    let dy_far_ret = 0;
    let dy_major_descent = 1;
    let dy_far = 2;
    let dy_near = 3;
    let dy_near2 = 4; // 2-item shape only
    let dy_ins2 = 5; // 2-item shape; ALSO near2's return-transit row
    let dy_ins1 = 4; // 1-item shape's single inserter row
    // 1-item shape gains a fluid-header row directly above the machine
    // when `has_fluid` — shifts the machine row (and everything south of
    // it) down by 1 relative to the no-fluid 1-item shape.
    let dy_machine = if has_minor { 6 } else { dy_ins1 + 1 + if has_fluid { 1 } else { 0 } };
    let dy_fluid_hdr = dy_machine - 1; // only meaningful when has_fluid
    let dy_out_ins = dy_machine + msz;
    let dy_major_collect = dy_out_ins + 1;
    let dy_minor_collect = dy_out_ins + 2; // 2-item only
    // dy_out_ins + 3 (dy_minor_collect + 1) is left as a dedicated pass
    // for the major loop's own detour (a straight, non-sideload feed
    // into its priority-splitter UG entrance — see the loop-corridor
    // section below); near2's east-side transit row sits one further
    // south so the two never share a tile.
    let dy_near2_transit = dy_out_ins + 4; // 2-item only

    let row_height = if has_minor { dy_near2_transit + 1 } else { dy_major_collect + 2 };

    let y = |dy: i32| y_offset + dy;

    // ---- North belts ----

    // Far belt (major item): spans machine 0 through the last machine,
    // fed exclusively by the corridor descent that lands at (mx0, dy_far).
    for x in mx0..east_x {
        entities.push(PlacedEntity {
            name: far_belt.to_string(),
            x,
            y: y(dy_far),
            direction: EntityDirection::East,
            carries: Some(major_item.to_string()),
            segment_id: loop_major_seg.clone(),
            rate: Some(major_loop_rate),
            ..Default::default()
        });
    }

    // Near belt (bus-tapped): standard west-anchored tap-off run, same
    // shape as `dual_input_row`'s belt2. In the 2-item shape, dips
    // underground for the single tile at `x_offset + 1` (`mx0 - 2`) so
    // the near2 corridor's vertical descent has a clear surface column
    // there — see the module doc comment.
    let near_dip_x = mx0 - 2;
    let near_ug = underground_for_belt(near_belt_name);
    for x in x_offset..east_x {
        if has_minor && x == x_offset {
            entities.push(PlacedEntity {
                name: near_ug.to_string(),
                x,
                y: y(dy_near),
                direction: EntityDirection::East,
                io_type: Some("input".to_string()),
                carries: Some(near_item.to_string()),
                segment_id: near_in_seg.clone(),
                rate: Some(near_total),
                ..Default::default()
            });
            continue;
        }
        if has_minor && x == near_dip_x {
            continue;
        }
        if has_minor && x == near_dip_x + 1 {
            entities.push(PlacedEntity {
                name: near_ug.to_string(),
                x,
                y: y(dy_near),
                direction: EntityDirection::East,
                io_type: Some("output".to_string()),
                carries: Some(near_item.to_string()),
                segment_id: near_in_seg.clone(),
                rate: Some(near_total),
                ..Default::default()
            });
            continue;
        }
        entities.push(PlacedEntity {
            name: near_belt_name.to_string(),
            x,
            y: y(dy_near),
            direction: EntityDirection::East,
            carries: Some(near_item.to_string()),
            segment_id: near_in_seg.clone(),
            rate: Some(near_total),
            ..Default::default()
        });
    }

    if has_minor {
        // Near2 belt (minor item, recirculated): per-machine UG-in /
        // LHI-A / UG-out, matching `quad_input_row`'s belt3 pattern —
        // the LHI-A tile at mx+1 needs the surface clear, so belt3
        // dives underground for that single tile.
        for &mx in &mxs {
            entities.push(PlacedEntity {
                name: near2_ug.to_string(),
                x: mx,
                y: y(dy_near2),
                direction: EntityDirection::East,
                io_type: Some("input".to_string()),
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                name: near2_ug.to_string(),
                x: mx + 2,
                y: y(dy_near2),
                direction: EntityDirection::East,
                io_type: Some("output".to_string()),
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
    }

    // ---- Fluid header (1-item shape only; `has_fluid`) ----
    //
    // One row directly above the machine (`dy_fluid_hdr == dy_machine -
    // 1`), at the SAME row the near-item's inserter must now occupy (see
    // the "Inserters + machines" section below — with the machine pushed
    // one row south to make room for this header, the near-item's
    // formerly-regular inserter no longer reaches it from `dy_ins1` and
    // becomes a long-handed inserter living on this row instead).
    //
    // CHEM-shape ports (chemical-plant/biochamber) sit at dx=0 and dx=2;
    // the near-item's LHI takes the middle column (dx=1), so the header
    // bridges over it with a per-machine pipe-to-ground pair (UG-in at
    // dx=0, UG-out at dx=2) rather than a plain continuous strip.
    //
    // Unlike belt UGs (paired by matching direction), fluid PTGs pair by
    // FACING EACH OTHER: `find_ptg_pairs` in `validate/fluids.rs` requires
    // the output's direction to be the opposite of the input's — the same
    // convention `fluid_input_row` uses (South in / North out). Here:
    // dx=0 faces East (tunnels toward dx=2, surface mouth points West,
    // toward the previous machine / bus trunk); dx=2 faces West (surface
    // mouth points East). Both UG surface tiles land exactly on the
    // machine's physical fluid ports (only one is strictly required by
    // the validator, but connecting both is free here). Machines pack at
    // `pitch == msz` with no gap, so machine N's dx=2 mouth (East) lands
    // exactly on machine N+1's dx=0 tile, whose own mouth (West) points
    // back — a mutual surface match chains every machine's bridge into
    // one continuous network across the row.
    if let Some((fluid_item, _fluid_rate)) = fluid_input {
        for &mx in &mxs {
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: mx,
                y: y(dy_fluid_hdr),
                direction: EntityDirection::East,
                io_type: Some("input".to_string()),
                carries: Some(fluid_item.to_string()),
                segment_id: fluid_hdr_seg.clone(),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                // Faces WEST (toward the input), not East: fluid PTG
                // pairing (`find_ptg_pairs` in `validate/fluids.rs`)
                // requires the output to face the OPPOSITE direction of
                // its input partner — same convention `fluid_input_row`
                // uses (South in / North out). Facing West also makes
                // this tile's OWN surface mouth point EAST, landing
                // exactly on the next machine's dx=0 input tile
                // (`pitch == msz`), so consecutive machines' bridges
                // chain into one network without an explicit gap-fill.
                name: "pipe-to-ground".to_string(),
                x: mx + 2,
                y: y(dy_fluid_hdr),
                direction: EntityDirection::West,
                io_type: Some("output".to_string()),
                carries: Some(fluid_item.to_string()),
                segment_id: fluid_hdr_seg.clone(),
                ..Default::default()
            });
            // Bus tap-point for this machine: the dx=0 UG surface tile.
            fluid_input_port_pipes.push((fluid_item.to_string(), mx, y(dy_fluid_hdr)));
        }
    }

    // ---- Inserters + machines ----

    if has_minor {
        for &mx in &mxs {
            // LHI-A: major's own INPUT demand (`spec.self_loop`, check-
            // invisible per the known ceiling) — hardcoded, unchanged.
            // Picks far belt (dy_far, reach 2), drops machine top.
            entities.push(PlacedEntity {
                name: "long-handed-inserter".to_string(),
                x: mx + 1,
                y: y(dy_near2),
                direction: EntityDirection::South,
                carries: Some(major_item.to_string()),
                segment_id: far_ins_seg.clone(),
                rate: Some(major_loop_rate),
                ..Default::default()
            });
            // LHI-B: near_item's OWN inserter — check-visible
            // (`spec.inputs`), ladder-sized. Hard 0 extra-column budget:
            // this row (dy_ins2) is 100% packed (LHI-B at dx=1 plus the
            // near2 regular pair below at dx=0/dx=2).
            let near_plan = size_side(near_net_rate, Reach::Far, 0, max_inserter_tier);
            entities.push(PlacedEntity {
                name: near_plan.entity.to_string(),
                x: mx + 1,
                y: y(dy_ins2),
                direction: EntityDirection::South,
                carries: Some(near_item.to_string()),
                segment_id: near_ins_seg.clone(),
                rate: Some(near_total),
                ..Default::default()
            });
            emit_shortfall_trace(recipe, false, near_net_rate, &near_plan, mx, y(dy_machine), false);
            // Regulars: minor's OWN INPUT demand (`spec.self_loop`,
            // check-invisible) — hardcoded, unchanged. Pick near2's UG
            // surface tiles (dy_near2, reach 1), drop machine top.
            for &dx in &[0i32, 2] {
                entities.push(PlacedEntity {
                    name: "inserter".to_string(),
                    x: mx + dx,
                    y: y(dy_ins2),
                    direction: EntityDirection::South,
                    carries: Some(near_item.to_string()),
                    segment_id: near2_ins_seg.clone(),
                    rate: Some(minor_total),
                    ..Default::default()
                });
            }
        }
    } else {
        for &mx in &mxs {
            // Major's LHI: major's own INPUT demand (`spec.self_loop`,
            // check-invisible) — hardcoded, unchanged, regardless of
            // `has_fluid`: reach 2 already lands its pickup on `dy_far`
            // (2 rows up) and its drop inside the machine's footprint (2
            // rows down), and both of those still hold whether the
            // machine starts at `dy_ins1 + 1` (no fluid) or `dy_ins1 + 2`
            // (has_fluid) — `msz == 3` machines overlap both possible
            // footprints at that fixed drop row.
            entities.push(PlacedEntity {
                name: "long-handed-inserter".to_string(),
                x: mx,
                y: y(dy_ins1),
                direction: EntityDirection::South,
                carries: Some(major_item.to_string()),
                segment_id: far_ins_seg.clone(),
                rate: Some(major_loop_rate),
                ..Default::default()
            });
            if has_fluid {
                // The fluid header row now sits between `dy_ins1` and
                // the machine, so a reach-1 inserter here can no longer
                // reach the machine. Move to `dy_fluid_hdr` itself
                // (dx=1, the column the header's UG bridge already
                // skips — see the "Fluid header" section above) as a
                // long-handed inserter: pickup 2 rows up lands back on
                // `dy_near` (unchanged), drop 2 rows down lands inside
                // the machine. near_item is check-visible; ladder-sized
                // with a hard 0 extra-column budget (dy_fluid_hdr's 3 dx
                // are all packed: PTG-in, this LHI, PTG-out).
                let near_plan = size_side(near_net_rate, Reach::Far, 0, max_inserter_tier);
                entities.push(PlacedEntity {
                    name: near_plan.entity.to_string(),
                    x: mx + 1,
                    y: y(dy_fluid_hdr),
                    direction: EntityDirection::South,
                    carries: Some(near_item.to_string()),
                    segment_id: near_ins_seg.clone(),
                    rate: Some(near_total),
                    ..Default::default()
                });
                emit_shortfall_trace(recipe, false, near_net_rate, &near_plan, mx, y(dy_machine), false);
            } else {
                // near_item is check-visible; ladder-sized. Baseline at
                // dx=2 (major's LHI owns dx=0); the ONE extra column
                // (dx=1) is uncontested — major's true input demand is
                // check-invisible, so near is credited it optimistically
                // (the census's documented caveat; doesn't change this
                // corpus's verdict).
                let near_extra_dx = vec![1i32];
                let near_plan = size_side(near_net_rate, Reach::Near, near_extra_dx.len(), max_inserter_tier);
                stamp_side_inserters(
                    &mut entities,
                    &near_plan,
                    mx,
                    y(dy_ins1),
                    EntityDirection::South,
                    near_item,
                    &near_ins_seg,
                    2,
                    &near_extra_dx,
                );
                emit_shortfall_trace(recipe, false, near_net_rate, &near_plan, mx, y(dy_machine), false);
            }
        }
    }

    for &mx in &mxs {
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y(dy_machine),
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });
    }

    // ---- Output inserters — ladder-sized (check-visible: `spec.outputs`).
    // No-minor shapes: major is uncontested (2 free columns, dx=0/dx=2).
    // Has-minor shape: major (baseline dx=1) and minor's export (baseline
    // dx=0) share the ONE remaining free tile (dx=2) — contested.
    let minor_produced_rate = minor.map(|(_, p)| p).unwrap_or(0.0);
    for &mx in &mxs {
        if has_minor {
            let shared_dx = vec![2i32];
            let minor_wins = contest_favors_far(major_produced_rate, minor_produced_rate, true);
            let major_extra_dx: Vec<i32> = if minor_wins { vec![] } else { shared_dx.clone() };
            let minor_extra_dx: Vec<i32> = if minor_wins { shared_dx } else { vec![] };

            let major_plan = size_side(major_produced_rate, Reach::Near, major_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &major_plan,
                mx,
                y(dy_out_ins),
                EntityDirection::South,
                major_item,
                &major_out_ins_seg,
                1,
                &major_extra_dx,
            );
            emit_shortfall_trace(recipe, true, major_produced_rate, &major_plan, mx, y(dy_machine), minor_wins);

            let minor_plan = size_side(minor_produced_rate, Reach::Far, minor_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &minor_plan,
                mx,
                y(dy_out_ins),
                EntityDirection::South,
                near_item,
                &minor_out_ins_seg,
                0,
                &minor_extra_dx,
            );
            emit_shortfall_trace(recipe, true, minor_produced_rate, &minor_plan, mx, y(dy_machine), !minor_wins);
        } else {
            let major_extra_dx = vec![0i32, 2i32];
            let major_plan = size_side(major_produced_rate, Reach::Near, major_extra_dx.len(), max_inserter_tier);
            stamp_side_inserters(
                &mut entities,
                &major_plan,
                mx,
                y(dy_out_ins),
                EntityDirection::South,
                major_item,
                &major_out_ins_seg,
                1,
                &major_extra_dx,
            );
            emit_shortfall_trace(recipe, true, major_produced_rate, &major_plan, mx, y(dy_machine), false);
        }
    }

    // ---- Major collector + priority splitter + loop corridor ----
    let major_collect_seg = collect_major_seg.clone();
    for x in mx0..=east_x {
        entities.push(PlacedEntity {
            name: collector_belt.to_string(),
            x,
            y: y(dy_major_collect),
            direction: EntityDirection::East,
            carries: Some(major_item.to_string()),
            segment_id: major_collect_seg.clone(),
            rate: Some(major_total),
            ..Default::default()
        });
    }

    let split_x = east_x + 1;
    entities.push(PlacedEntity {
        name: splitter_name.to_string(),
        x: split_x,
        y: y(dy_major_collect),
        direction: EntityDirection::East,
        loop_priority_rate: Some(major_loop_rate),
        output_priority: Some("right".to_string()),
        carries: Some(major_item.to_string()),
        segment_id: major_collect_seg.clone(),
        rate: Some(major_total),
        ..Default::default()
    });

    // Export tile: splitter's "left" (north) output, continues the
    // row's declared output belt east toward the merger.
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: split_x + 1,
        y: y(dy_major_collect),
        direction: EntityDirection::East,
        carries: Some(major_item.to_string()),
        segment_id: major_collect_seg,
        rate: Some(major_export_rate),
        ..Default::default()
    });

    // Loop tile: splitter's "right" (south) output. `dy_major_collect`
    // (and `riser_x + 1`, the router's merge column) is where the
    // ghost router's output-merger extends the row's declared export
    // belt east then turns it south — a surface riser there collides
    // with the merger. Route the loop DOWN one more row (to
    // `dy_major_collect + 2`, otherwise unused) and jog east there
    // before turning north into the priority-splitter UG entrance:
    // the entrance needs a STRAIGHT (non-sideload) feed to load both
    // lanes, which means its immediate predecessor must already be
    // travelling north — turning directly off the splitter's own
    // east-flowing output would sideload the entrance instead (see
    // `tier_kovarex_self_loop` iteration history).
    let loop_stub_x = split_x + 1;
    let riser_x = split_x + 2;
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: loop_stub_x,
        y: y(dy_major_collect + 1),
        direction: EntityDirection::South,
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: loop_stub_x,
        y: y(dy_major_collect + 2),
        direction: EntityDirection::East,
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: riser_x,
        y: y(dy_major_collect + 2),
        direction: EntityDirection::North,
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });

    // Riser: north from the loop's approach row up to the return row,
    // turning west at dy_far_ret. Dips underground for the single tile
    // at `dy_major_collect`, fed straight from the tile just placed.
    let riser_ug = underground_for_belt(collector_belt);
    entities.push(PlacedEntity {
        name: riser_ug.to_string(),
        x: riser_x,
        y: y(dy_major_collect + 1),
        direction: EntityDirection::North,
        io_type: Some("input".to_string()),
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: riser_ug.to_string(),
        x: riser_x,
        y: y(dy_major_collect - 1),
        direction: EntityDirection::North,
        io_type: Some("output".to_string()),
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });
    for dy in (dy_far_ret + 1..dy_major_collect - 1).rev() {
        entities.push(PlacedEntity {
            name: collector_belt.to_string(),
            x: riser_x,
            y: y(dy),
            direction: EntityDirection::North,
            carries: Some(major_item.to_string()),
            segment_id: loop_major_seg.clone(),
            rate: Some(major_loop_rate),
            ..Default::default()
        });
    }
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: riser_x,
        y: y(dy_far_ret),
        direction: EntityDirection::West,
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });
    // Return row: west from the riser back to machine 0's column.
    for x in (mx0 + 1..riser_x).rev() {
        entities.push(PlacedEntity {
            name: collector_belt.to_string(),
            x,
            y: y(dy_far_ret),
            direction: EntityDirection::West,
            carries: Some(major_item.to_string()),
            segment_id: loop_major_seg.clone(),
            rate: Some(major_loop_rate),
            ..Default::default()
        });
    }
    // Turn south at machine 0's column, descend past dy_major_descent
    // into the far belt's own first tile (dy_far, already stamped
    // above as part of the far-belt loop — this just overwrites its
    // direction is unnecessary since that loop already used East).
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: mx0,
        y: y(dy_far_ret),
        direction: EntityDirection::South,
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg.clone(),
        rate: Some(major_loop_rate),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: collector_belt.to_string(),
        x: mx0,
        y: y(dy_major_descent),
        direction: EntityDirection::South,
        carries: Some(major_item.to_string()),
        segment_id: loop_major_seg,
        rate: Some(major_loop_rate),
        ..Default::default()
    });

    // ---- Minor collector + corridor (2-item shape only) ----
    if has_minor {
        for x in mx0..east_x - 1 {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x,
                y: y(dy_minor_collect),
                direction: EntityDirection::East,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        // Turn south at the last machine's own column, continuing one
        // extra tile past `dy_minor_collect + 1` (left clear for
        // major's own detour — see the module doc comment) before
        // turning east into the transit row.
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: east_x - 1,
            y: y(dy_minor_collect),
            direction: EntityDirection::South,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: east_x - 1,
            y: y(dy_minor_collect + 1),
            direction: EntityDirection::South,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        // Transit east, clearing major's splitter zone entirely.
        let transit_riser_x = riser_x + 3;
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: east_x - 1,
            y: y(dy_near2_transit),
            direction: EntityDirection::East,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        // `riser_x` and `riser_x + 1` (= `merge_x`) are the ghost
        // router's output-merger columns (east-extension + south
        // column — see the major riser's UG dip above); dip
        // underground under both so this transit row doesn't collide
        // with the merger regardless of how far south its column runs.
        for x in east_x..riser_x {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x,
                y: y(dy_near2_transit),
                direction: EntityDirection::East,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: riser_x,
            y: y(dy_near2_transit),
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: riser_x + 2,
            y: y(dy_near2_transit),
            direction: EntityDirection::East,
            io_type: Some("output".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        for x in riser_x + 3..transit_riser_x {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x,
                y: y(dy_near2_transit),
                direction: EntityDirection::East,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        // Riser: north from the transit row up to y+1 (major's own
        // descent-transit row, otherwise clear — see below), turning
        // west there. Note this rises past dy_ins2, dy_machine, and
        // dy_out_ins at `transit_riser_x`, a column no other entity in
        // this template touches. Dips underground at `dy_major_collect`
        // for the same reason the major riser does.
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: transit_riser_x,
            y: y(dy_major_collect + 1),
            direction: EntityDirection::North,
            io_type: Some("input".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: transit_riser_x,
            y: y(dy_major_collect - 1),
            direction: EntityDirection::North,
            io_type: Some("output".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        for dy in (dy_major_descent + 1..dy_major_collect - 1).rev() {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x: transit_riser_x,
                y: y(dy),
                direction: EntityDirection::North,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        for dy in (dy_major_collect + 2..=dy_near2_transit).rev() {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x: transit_riser_x,
                y: y(dy),
                direction: EntityDirection::North,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: transit_riser_x,
            y: y(dy_major_descent),
            direction: EntityDirection::West,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        // Return west at y+1, crossing TWO major structures with 1-tile
        // underground hops: major's riser (column `riser_x`) and
        // major's descent (column `mx0`) — the only crossings the two
        // corridors need (`near_belt`'s own dip at `mx0 - 2` handles
        // the third, on the vertical leg below).
        let x1_entrance = riser_x + 1;
        let x1_exit = riser_x - 1;
        let x2_entrance = mx0 + 1;
        let x2_exit = mx0 - 1;
        for x in (x1_entrance + 1..transit_riser_x).rev() {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x,
                y: y(dy_major_descent),
                direction: EntityDirection::West,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: x1_entrance,
            y: y(dy_major_descent),
            direction: EntityDirection::West,
            io_type: Some("input".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: x1_exit,
            y: y(dy_major_descent),
            direction: EntityDirection::West,
            io_type: Some("output".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        for x in (x2_entrance + 1..x1_exit).rev() {
            entities.push(PlacedEntity {
                name: minor_collector_belt.to_string(),
                x,
                y: y(dy_major_descent),
                direction: EntityDirection::West,
                carries: Some(near_item.to_string()),
                segment_id: loop_minor_seg.clone(),
                rate: Some(minor_total),
                ..Default::default()
            });
        }
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: x2_entrance,
            y: y(dy_major_descent),
            direction: EntityDirection::West,
            io_type: Some("input".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: near2_ug.to_string(),
            x: x2_exit,
            y: y(dy_major_descent),
            direction: EntityDirection::West,
            io_type: Some("output".to_string()),
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        // Turn south at `mx0 - 2` (the near2 descent column — the
        // exact tile `near_belt`'s own dip left clear at dy_near).
        let descent_x = mx0 - 2;
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: descent_x,
            y: y(dy_major_descent),
            direction: EntityDirection::South,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: descent_x,
            y: y(dy_far),
            direction: EntityDirection::South,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: descent_x,
            y: y(dy_near),
            direction: EntityDirection::South,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        // Turn east into near2's own row, feeding straight into machine
        // 0's UG-in (mx0, dy_near2).
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: descent_x,
            y: y(dy_near2),
            direction: EntityDirection::East,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: minor_collector_belt.to_string(),
            x: descent_x + 1,
            y: y(dy_near2),
            direction: EntityDirection::East,
            carries: Some(near_item.to_string()),
            segment_id: loop_minor_seg.clone(),
            rate: Some(minor_total),
            ..Default::default()
        });
    }

    (entities, row_height, fluid_input_port_pipes)
}

/// Voider row: a recycler bank that self-consumes a solid surplus stream
/// (RFP Fulgora Phase 2, `docs/rfp-fulgora-scrap.md` D1). Inspired by
/// `self_loop_row`'s vocabulary (loop corridor, recirculation, per-machine
/// rate convention) but the plumbing inverts: recirculates the ENTIRE
/// ejected stream (no export — there's no splitter at all), and is built
/// around `recycler`'s real physicals (2×4, direct belt ejection,
/// inserter-fed input) instead of a square inserter-only machine.
///
/// The external demand (near belt) is fed from a dedicated SOUTH-facing
/// supply column, not a west-side bus tap — see `ghost_router`'s Step
/// 7c. Voider rows are excluded from the ordinary west-trunk lane system
/// (`lane_planner`'s voider-consumer skip): the row producing the voided
/// item is typically EAST-flowing (its primary output is the solve's own
/// target — e.g. uranium-processing's U-235 target + U-238 surplus), so
/// a west-directed return spec would have to run backwards across the
/// entire row against its own flow. Step 7c instead reuses the same
/// producer-gathering + `merge_output_rows` machinery Step 7b (D2a/D2b
/// export) already uses, then routes the merged tail to this row's
/// supply column via a corridor below every other south-flushed tail.
///
/// Layout per machine (2-tile pitch, flush, `count` recyclers), relative
/// to `(x_offset, y_offset)`:
/// ```text
///   y+0        : collector/eject row — recycler i ejects directly onto
///                (mx_i, y+0) with NO inserter (mining-drill-style);
///                flows WEST to a descent column west of the bank
///   y+1..y+4   : recycler bank (2 wide x 4 tall each), facing NORTH
///   y+5        : inserter row — regular (dx=0, reach 1) picks the near
///                (tap) belt; long-handed (dx=1, reach 2) picks the far
///                (recirc) belt
///   y+6        : near belt — external tap demand, east-flowing, one UG
///                dip near the west end so the descent column can cross
///                it
///   y+7        : far belt (desc_x east) — 100% recirculated ejections,
///                fed by the descent's own turn; supply entry
///                (x_offset, SAME row, west of desc_x) — Step 7c's
///                landing tile, turns north into the near belt's UG
/// ```
/// ASCII sketch (2 recyclers, dx=0/1 are the two machine columns):
/// ```text
///           belt-in →→→[collect]←←←[collect]  <- eject row (y+0)
///   descent  ↓          ┌──┐         ┌──┐
///     col    ↓          │R1│         │R2│      <- recycler bank
///     v      ↓          └──┘         └──┘         (y+1..y+4)
///     v      ↓        near↑far     near↑far     <- inserters (y+5)
///     v   ug-dip→[near tap belt]────────────→   <- near/tap (y+6)
///     ^──────────[far recirc belt]───────────→  <- far/recirc (y+7)
///   supply
///   (Step 7c)
/// ```
/// Rate math (`bus::voider::synthesize_voiders`): a surplus stream of
/// `r` items/s needs gross throughput `g = r / (1 - fraction)` (each
/// pass returns `fraction` back), `machines = ceil(g / per_recycler_rate)`.
/// `near_rate_per_machine`/`far_rate_per_machine` are PER-MACHINE (this
/// function multiplies by `count` internally), matching
/// `self_loop_row`'s calling convention.
///
/// Returns `(entities, row_height)`. `row_height` matches
/// `RowKind::Voider::row_height()` (8).
#[allow(clippy::too_many_arguments)]
/// Both sides have a HARD 0 extra-column budget (2-wide recycler footprint,
/// both dx already claimed by the near/far baseline) — simple in-place tier
/// upgrades, no contest needed.
pub fn voider_row(
    recipe: &str,
    item: &str,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    near_rate_per_machine: f64,
    far_rate_per_machine: f64,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32) {
    use crate::bus::balancer::underground_for_belt;
    use crate::common::belt_entity_for_rate;

    debug_assert_eq!(
        crate::common::machine_dims("recycler"),
        (2, 4),
        "voider_row is built around the recycler's real 2x4 footprint; see rfp-fulgora-scrap Phase 0"
    );
    let count = machine_count.max(1) as i32;
    let near_total = (near_rate_per_machine * count as f64).max(1e-6);
    let far_total = (far_rate_per_machine * count as f64).max(1e-6);

    let near_belt = belt_entity_for_rate(near_total, max_belt_tier);
    let recirc_belt = belt_entity_for_rate(far_total, max_belt_tier);
    let near_ug = underground_for_belt(near_belt);

    // Prefix zone west of machine 0: 5 columns, two independent purposes
    // that must NOT share a tile:
    //   x_offset          : supply_col — the near belt's own west-most
    //                       tile, fed by a PLAIN corner turn from the
    //                       south (`ghost_router`'s Step 7c). A plain
    //                       belt corner is fine (B11: preserves both
    //                       lanes) — but a UG-entrance specifically
    //                       needs a STRAIGHT feed to load both lanes
    //                       (see `feedback_sideload_ug` — a perpendicular
    //                       feed into a UG-entrance only loads one lane,
    //                       unlike an ordinary belt corner), so this
    //                       column must stay clear of the UG dip below.
    //   x_offset+2/+3/+4  : near belt's UG dip (mirrors
    //                       `self_loop_row`'s PREFIX=3 shape, just
    //                       shifted 2 east) — UG-in / descent's own
    //                       column / UG-out, straight-fed from
    //                       supply_col's plain run.
    const PREFIX: i32 = 5;
    let mx0 = x_offset + PREFIX;
    let mxs: Vec<i32> = (0..count).map(|i| mx0 + i * 2).collect();
    let last_mx = *mxs.last().expect("machine_count >= 1");
    let east_x = last_mx + 2;
    let supply_col = x_offset;
    let desc_x = mx0 - 2; // == x_offset + 3

    let row_height = 8;
    let dy_collect = 0;
    let dy_machine = 1;
    let dy_ins = 5;
    let dy_near = 6;
    let dy_far = 7;
    let y = |dy: i32| y_offset + dy;

    let mut entities: Vec<PlacedEntity> = Vec::new();

    let machine_seg = Some(format!("row:{recipe}:machine"));
    let near_in_seg = Some(format!("row:{recipe}:belt-in:{item}"));
    let near_ins_seg = Some(format!("row:{recipe}:inserter-in:{item}"));
    let far_ins_seg = Some(format!("row:{recipe}:inserter-in:{item}:recirc"));
    // `:voider:` — the closed recirculation loop (eject -> collect ->
    // descent -> far belt -> back into the machines). Tagged so
    // `check_belt_loops` can extend its `:selfloop:` exemption to cover
    // this physically-legitimate cycle (RFP brief point 6).
    let loop_seg = Some(format!("row:{recipe}:voider:{item}"));

    // ---- Recyclers ----
    for &mx in &mxs {
        entities.push(PlacedEntity {
            name: "recycler".to_string(),
            x: mx,
            y: y(dy_machine),
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });
    }

    // ---- Inserters (dy_ins): near (dx=0, reach 1) + far (dx=1, reach 2) —
    // ladder-sized, hard 0 extra-column budget both sides (2-wide
    // footprint, both dx already claimed by the baseline pair).
    for &mx in &mxs {
        let near_plan = size_side(near_rate_per_machine, Reach::Near, 0, max_inserter_tier);
        entities.push(PlacedEntity {
            name: near_plan.entity.to_string(),
            x: mx,
            y: y(dy_ins),
            direction: EntityDirection::North,
            carries: Some(item.to_string()),
            segment_id: near_ins_seg.clone(),
            rate: Some(near_total),
            ..Default::default()
        });
        emit_shortfall_trace(recipe, false, near_rate_per_machine, &near_plan, mx, y(dy_machine), false);

        let far_plan = size_side(far_rate_per_machine, Reach::Far, 0, max_inserter_tier);
        entities.push(PlacedEntity {
            name: far_plan.entity.to_string(),
            x: mx + 1,
            y: y(dy_ins),
            direction: EntityDirection::North,
            carries: Some(item.to_string()),
            segment_id: far_ins_seg.clone(),
            rate: Some(far_total),
            ..Default::default()
        });
        emit_shortfall_trace(recipe, false, far_rate_per_machine, &far_plan, mx, y(dy_machine), false);
    }

    // ---- Near belt (dy_near): external tap demand, east-flowing, UG
    // dip at desc_x so the descent column can cross it. Trimmed to
    // `last_mx` (dx=0 of the last machine) — the regular inserter's
    // last adjacency; nothing picks from `last_mx + 1`.
    //
    // Fed from the SOUTH at `supply_col`, not the west bus
    // (`ghost_router`'s Step 7c connects the producer row's own output
    // belt directly to the dedicated supply tile below — see the block
    // after this loop) — voider rows are excluded from the ordinary
    // west-trunk lane system because their producer row is typically
    // EAST-flowing (its primary output is the solve's own target), and
    // a west-directed return spec would have to cross the entire row
    // backwards against its own flow. `supply_col` is a PLAIN belt (not
    // the UG dip below) — a perpendicular corner feed into a UG-entrance
    // only loads one lane (`feedback_sideload_ug`), unlike an ordinary
    // belt corner (B11), so the south-fed corner and the descent-hop UG
    // dip must be different tiles.
    for x in supply_col..=last_mx {
        if x == desc_x {
            continue;
        }
        let ent = if x == desc_x - 1 {
            PlacedEntity {
                name: near_ug.to_string(),
                x,
                y: y(dy_near),
                direction: EntityDirection::East,
                io_type: Some("input".to_string()),
                carries: Some(item.to_string()),
                segment_id: near_in_seg.clone(),
                rate: Some(near_total),
                ..Default::default()
            }
        } else if x == desc_x + 1 {
            PlacedEntity {
                name: near_ug.to_string(),
                x,
                y: y(dy_near),
                direction: EntityDirection::East,
                io_type: Some("output".to_string()),
                carries: Some(item.to_string()),
                segment_id: near_in_seg.clone(),
                rate: Some(near_total),
                ..Default::default()
            }
        } else {
            PlacedEntity {
                name: near_belt.to_string(),
                x,
                y: y(dy_near),
                direction: EntityDirection::East,
                carries: Some(item.to_string()),
                segment_id: near_in_seg.clone(),
                rate: Some(near_total),
                ..Default::default()
            }
        };
        entities.push(ent);
    }

    // Supply entry: one tile south of `supply_col`'s plain near-belt
    // tile, turning the external feed NORTH into it — a plain corner
    // (B11: preserves both lanes). `ghost_router`'s Step 7c extends this
    // column further south to connect to the producer row. `supply_col`
    // is untouched by every other part of this template (the UG dip
    // lives at `desc_x = x_offset + 3`, the machine bank starts at
    // `mx0 = x_offset + 5`), so this is always free.
    entities.push(PlacedEntity {
        name: near_belt.to_string(),
        x: supply_col,
        y: y(dy_near + 1),
        direction: EntityDirection::North,
        carries: Some(item.to_string()),
        segment_id: near_in_seg.clone(),
        rate: Some(near_total),
        ..Default::default()
    });

    // ---- Far belt (dy_far): 100% recirculated, east-flowing. Starts at
    // the descent's own turn (desc_x) and runs through `east_x - 1`
    // (dx=1 of the last machine — the long-handed inserter's pickup).
    for x in desc_x..east_x {
        entities.push(PlacedEntity {
            name: recirc_belt.to_string(),
            x,
            y: y(dy_far),
            direction: EntityDirection::East,
            carries: Some(item.to_string()),
            segment_id: loop_seg.clone(),
            rate: Some(far_total),
            ..Default::default()
        });
    }

    // ---- Collector/eject row (dy_collect): each recycler ejects
    // directly onto (mx, dy_collect) with NO inserter — mining-drill-
    // style, per the Phase 0 physicals finding (`vector_to_place_result`
    // lands one tile north of the machine's north edge, on the west
    // column). Flows WEST from `last_mx` (trimmed — nothing ejects onto
    // `last_mx + 1`) back to the descent corner.
    for x in (mx0 - 1)..=last_mx {
        entities.push(PlacedEntity {
            name: recirc_belt.to_string(),
            x,
            y: y(dy_collect),
            direction: EntityDirection::West,
            carries: Some(item.to_string()),
            segment_id: loop_seg.clone(),
            rate: Some(far_total),
            ..Default::default()
        });
    }
    // Corner: turn south into the descent column.
    entities.push(PlacedEntity {
        name: recirc_belt.to_string(),
        x: desc_x,
        y: y(dy_collect),
        direction: EntityDirection::South,
        carries: Some(item.to_string()),
        segment_id: loop_seg.clone(),
        rate: Some(far_total),
        ..Default::default()
    });

    // ---- Descent column (desc_x): straight south from the collector
    // corner, through the recycler bank's own column (clear — machines
    // start at mx0) and the near belt's UG dip, landing on the far
    // belt's own start tile (dy_far, already stamped above with EAST
    // direction — this just fills the vertical run above it).
    for dy in (dy_collect + 1)..dy_far {
        entities.push(PlacedEntity {
            name: recirc_belt.to_string(),
            x: desc_x,
            y: y(dy),
            direction: EntityDirection::South,
            carries: Some(item.to_string()),
            segment_id: loop_seg.clone(),
            rate: Some(far_total),
            ..Default::default()
        });
    }

    (entities, row_height)
}

/// Scrap-recycling sushi-sorter row (RFP Fulgora Phase 3,
/// `docs/rfp-fulgora-scrap.md` D3, architecture (a)).
///
/// A bank of south-facing recyclers running `scrap-recycling` ejects its
/// ~12 mixed products directly (mining-drill-style, `recycler_eject_tile`)
/// onto a single east-flowing "sushi" belt one tile south of the machines.
/// A bank of filter inserters — one per item type — lifts each product off
/// the sushi belt onto its own single-item east-flowing output belt. Those
/// per-item belts fan out south in a crossing-free staircase so each ends
/// at its own y, reaching the row's east edge, where the ordinary bus
/// machinery treats each as a normal producer output (consumers via the
/// lane planner, surplus via the step-7b merger).
///
/// ```text
///   dy0  scrap input belt (EAST) ── fed by the scrap trunk tap
///   dy1  scrap input inserters (SOUTH, one per recycler)
///   dy2  ┌── recyclers (2×4, SOUTH-facing) ──┐
///   ..5  └──────────────────────────────────┘
///   dy6  sushi belt (EAST, :sushi:) ── ejects land at (mx+1, dy6)
///   dy7                     sort filter inserters (SOUTH) ▓ ▓ ▓ ▓ …
///   dy8+ per-item output belts fan out east, one y each ───────►
/// ```
///
/// Returns `(entities, row_height, sorted_output_belts)` where
/// `sorted_output_belts[i] = (item, absolute_y)` is the belt each item
/// exits on — used to populate `RowSpan::sorted_output_belts`.
///
/// The row is inherently EAST-flowing (`output_east == true`): every
/// per-item belt reaches the east edge so the merger can continue it, and
/// consumed items route to their trunk from the same east exit.
#[allow(clippy::too_many_arguments)]
/// The scrap input inserter (`dy_scrap_ins`) is ladder-sized: baseline at
/// `mx` (dx=0), ONE extra column at `mx+1` (dx=1) — the 2-wide recycler
/// footprint only stamps an inserter at dx=0; the scrap belt above spans
/// the full width including dx=1, always free (no per-position trim). The
/// sort filter inserters already size dynamically by rate (unrelated,
/// pre-existing) and are exempt from this ladder like recycler outputs.
pub fn scrap_recycling_row(
    recipe: &str,
    input_item: &str,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_total_rate: f64,
    sorted_items: &[(String, f64)],
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32)>) {
    use crate::common::belt_entity_for_rate;

    debug_assert_eq!(
        crate::common::machine_dims("recycler"),
        (2, 4),
        "scrap_recycling_row is built around the recycler's real 2x4 footprint; \
         see rfp-fulgora-scrap Phase 0"
    );

    let count = machine_count.max(1) as i32;
    let k = sorted_items.len() as i32;

    // Geometry (see the module doc sketch above).
    const PITCH: i32 = 2; // recycler width
    let dy_scrap_in = 0;
    let dy_scrap_ins = 1;
    let dy_machine = 2; // recyclers occupy dy 2..=5 (height 4)
    let dy_sushi = 6; // == dy_machine + 4 (South eject: y+4)
    let dy_sort_ins = 7;
    let dy_lanes = 8;
    let y = |dy: i32| y_offset + dy;

    let mxs: Vec<i32> = (0..count).map(|i| x_offset + i * PITCH).collect();
    let mx0 = mxs[0];
    let machine_east = mx0 + PITCH * count; // one past the last machine column
    let eject_west = mx0 + 1; // machine 0's east column
    // Sort inserters sit strictly EAST of every ejection tile so the
    // east-flowing sushi carries each item past its own inserter.
    let sort_x0 = machine_east;
    let sort_cols: Vec<i32> = (0..k).map(|j| sort_x0 + j).collect();
    let sushi_east = sort_cols.last().copied().unwrap_or(sort_x0);
    // Each per-item horizontal belt runs one tile past the last inserter
    // column, so even the eastmost item has a horizontal run the merger
    // can pick up from the row's east edge.
    let east_x = sushi_east + 1;

    let scrap_belt = belt_entity_for_rate(input_total_rate, max_belt_tier);
    let sushi_total: f64 = sorted_items.iter().map(|(_, r)| *r).sum();
    let sushi_belt = belt_entity_for_rate(sushi_total, max_belt_tier);

    let machine_seg = Some(format!("row:{recipe}:machine"));
    let scrap_in_seg = Some(format!("row:{recipe}:belt-in:{input_item}"));
    let scrap_ins_seg = Some(format!("row:{recipe}:inserter-in:{input_item}"));
    // `:sushi:` — the mixed-item collection belt. Tagged so the validators
    // exempt sushi↔sushi adjacency from item-isolation and lane-walking,
    // and require every off-sushi transition to pass a filter inserter
    // (RFP Fulgora Phase 3 KC5 containment).
    let sushi_seg = Some(format!("row:{recipe}:sushi:{input_item}"));

    let mut entities: Vec<PlacedEntity> = Vec::new();

    // ---- Scrap input belt (dy0), east-flowing across the machine span ----
    for x in mx0..machine_east {
        entities.push(PlacedEntity {
            name: scrap_belt.to_string(),
            x,
            y: y(dy_scrap_in),
            direction: EntityDirection::East,
            carries: Some(input_item.to_string()),
            segment_id: scrap_in_seg.clone(),
            rate: Some(input_total_rate),
            ..Default::default()
        });
    }

    // ---- Recyclers + scrap input inserters (ladder-sized: baseline
    // dx=0, one extra column at dx=1 — always free, no per-position
    // trim) ----
    let input_rate_per_machine = input_total_rate / count.max(1) as f64;
    let scrap_extra_dx = vec![1i32];
    let scrap_plan = size_side(input_rate_per_machine, Reach::Near, scrap_extra_dx.len(), max_inserter_tier);
    for &mx in &mxs {
        stamp_side_inserters(
            &mut entities,
            &scrap_plan,
            mx,
            y(dy_scrap_ins),
            EntityDirection::South,
            input_item,
            &scrap_ins_seg,
            0,
            &scrap_extra_dx,
        );
        entities.push(PlacedEntity {
            name: "recycler".to_string(),
            x: mx,
            y: y(dy_machine),
            direction: EntityDirection::South,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });
        // Per-machine (moved inside the loop for the D2 machine join —
        // the shared plan means identical events, one per recycler).
        emit_shortfall_trace(
            recipe, false, input_rate_per_machine, &scrap_plan,
            mx, y(dy_machine), false,
        );
    }

    // ---- Sushi belt (dy6), east-flowing from the westmost eject to the
    // last sort inserter. Ejection tiles at (mx+1, dy6) land on it. ----
    for x in eject_west..=sushi_east {
        entities.push(PlacedEntity {
            name: sushi_belt.to_string(),
            x,
            y: y(dy_sushi),
            direction: EntityDirection::East,
            // Sushi tiles carry a mixed set — no single `carries` tag; the
            // saturation check owns their rate story. The `:sushi:` segment
            // marker is what the validators key on.
            segment_id: sushi_seg.clone(),
            rate: Some(sushi_total),
            ..Default::default()
        });
    }

    // ---- Sort filter inserters + per-item fan-out output belts ----
    // Crossing-free staircase: westmost inserter turns east at the DEEPEST
    // y, eastmost at the shallowest, so no vertical drop crosses a
    // horizontal run (see the RFP Phase 3 geometry note).
    let mut sorted_output_belts: Vec<(String, i32)> = Vec::new();
    for (j, (item, rate)) in sorted_items.iter().enumerate() {
        let jx = sort_cols[j];
        let rank = (k - 1) - j as i32; // westmost (j=0) => deepest
        let turn_dy = dy_lanes + rank;
        let out_belt = belt_entity_for_rate(*rate * 2.0, max_belt_tier);
        let out_seg = Some(format!("row:{recipe}:belt-out:{item}"));
        // `:sushi-sort:` marks the belt-to-belt filter inserter that lifts
        // one item off the sushi belt. Validators key on it:
        // `check_inserter_direction` exempts it (it touches belts, not a
        // machine), and the sushi boundary check verifies its filter.
        let sort_ins_seg = Some(format!("row:{recipe}:sushi-sort:{item}"));

        // Filter inserter: picks from the sushi tile to its north, drops
        // onto the head of this item's own belt to its south.
        entities.push(PlacedEntity {
            name: "fast-inserter".to_string(),
            x: jx,
            y: y(dy_sort_ins),
            direction: EntityDirection::South,
            carries: Some(item.clone()),
            filters: vec![item.clone()],
            segment_id: sort_ins_seg.clone(),
            rate: Some(*rate),
            ..Default::default()
        });

        // Vertical drop from the inserter's drop tile (dy_lanes) down to
        // the turn row, then a corner turning EAST.
        for dy in dy_lanes..turn_dy {
            entities.push(PlacedEntity {
                name: out_belt.to_string(),
                x: jx,
                y: y(dy),
                direction: EntityDirection::South,
                carries: Some(item.clone()),
                segment_id: out_seg.clone(),
                rate: Some(*rate),
                ..Default::default()
            });
        }
        // Corner (or drop tile itself, when turn_dy == dy_lanes): turn EAST.
        entities.push(PlacedEntity {
            name: out_belt.to_string(),
            x: jx,
            y: y(turn_dy),
            direction: EntityDirection::East,
            carries: Some(item.clone()),
            segment_id: out_seg.clone(),
            rate: Some(*rate),
            ..Default::default()
        });
        // Horizontal run east to the row's east edge.
        for x in (jx + 1)..=east_x {
            entities.push(PlacedEntity {
                name: out_belt.to_string(),
                x,
                y: y(turn_dy),
                direction: EntityDirection::East,
                carries: Some(item.clone()),
                segment_id: out_seg.clone(),
                rate: Some(*rate),
                ..Default::default()
            });
        }
        sorted_output_belts.push((item.clone(), y(turn_dy)));
    }

    let row_height = dy_lanes + k;
    (entities, row_height, sorted_output_belts)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: assert at least one entity at (x, y) with given name; returns the first match.
    fn assert_entity<'a>(entities: &'a [PlacedEntity], x: i32, y: i32, name: &str) -> &'a PlacedEntity {
        let found: Vec<_> = entities.iter().filter(|e| e.x == x && e.y == y).collect();
        assert!(!found.is_empty(), "No entity at ({x}, {y}), expected '{name}'");
        assert_eq!(found[0].name, name, "Wrong entity at ({x}, {y}): got '{}', expected '{name}'", found[0].name);
        found[0]
    }

    // ---- single_input_row ----

    #[test]
    fn single_input_row_basic_entity_count() {
        // 2 machines, west-flow output, no lane split. Machine 1 gets the full
        // 9 entities (3 input + 1 inserter + 1 machine + 1 out inserter + 3 output).
        // Machine 2 (last) drops one input belt tile (east-tail orphan at mx+2)
        // and one output belt tile (west-flow orphan at mx+2): 7 entities.
        let (entities, height) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3, // machine_size
            2,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
            None,
            0.5, // input_rate -- well under the regular-inserter ceiling
            0.5, // output_rate
            None,
            InserterTier::default(),
        );
        assert_eq!(height, 7);
        assert_eq!(entities.len(), 9 + 7);
    }

    #[test]
    fn single_input_row_one_machine_positions() {
        let (entities, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0, // y_offset
            0, // x_offset
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
            None,
            0.5,
            0.5,
            None,
            InserterTier::default(),
        );

        // Input belts at y=0: x=0,1 (x=2 is orphan east-tail, trimmed).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 0, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
            assert_eq!(e.carries.as_deref(), Some("iron-plate"));
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none(),
            "input belt at (2, 0) should be trimmed (east of inserter pickup)");

        // Inserter at (1, 1) facing SOUTH
        let ins = assert_entity(&entities, 1, 1, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("iron-plate"));

        // Machine at (0, 2) facing NORTH
        let machine = assert_entity(&entities, 0, 2, "assembling-machine-3");
        assert_eq!(machine.direction, EntityDirection::North);
        assert_eq!(machine.recipe.as_deref(), Some("iron-gear-wheel"));

        // Output inserter at (1, 5) facing SOUTH
        let out_ins = assert_entity(&entities, 1, 5, "inserter");
        assert_eq!(out_ins.direction, EntityDirection::South);
        assert_eq!(out_ins.carries.as_deref(), Some("iron-gear-wheel"));

        // Output belts at y=6: x=0,1 (x=2 is orphan east-tail for west-flow output).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 6, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
            assert_eq!(e.carries.as_deref(), Some("iron-gear-wheel"));
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 6).is_none(),
            "output belt at (2, 6) should be trimmed");
    }

    #[test]
    fn single_input_row_x_y_offset() {
        // With x_offset=6, y_offset=10, first machine should be at (6, 12).
        let (entities, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            10,
            6,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
            None,
            0.5,
            0.5,
            None,
            InserterTier::default(),
        );
        assert_entity(&entities, 6, 12, "assembling-machine-3");
    }

    #[test]
    fn single_input_row_output_east() {
        let (entities, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            true, // output_east
            None,
            0.5,
            0.5,
            None,
            InserterTier::default(),
        );
        // Output belts at y=6 should face EAST
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 6, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
        }
    }

    #[test]
    fn single_input_row_lane_split_two_machines() {
        // 2 machines, Strategy A (inline shift, tight pack): machines
        // at x=0 and x=3. Anchor index = 0 → bridge cols 1,2,3.
        let (entities, height) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            2,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            true, // lane_split
            false,
            None,
            0.5,
            0.5,
            None,
            InserterTier::default(),
        );
        assert_eq!(height, 7);

        // Machine 1 at x=0
        assert_entity(&entities, 0, 2, "assembling-machine-3");
        // Machine 2 at x=3 (tight pack, no LANE_SPLIT_GAP)
        assert_entity(&entities, 3, 2, "assembling-machine-3");

        // Anchor's output inserter shifted from mx+1=1 to mx=0 to free
        // col 1 for the bridge. Output inserter row at y_offset+2+msz=5.
        let anchor_ins = assert_entity(&entities, 0, 5, "inserter");
        assert_eq!(anchor_ins.direction, EntityDirection::South);
        // Machine 2's output inserter stays at mx+1=4.
        let m2_ins = assert_entity(&entities, 4, 5, "inserter");
        assert_eq!(m2_ins.direction, EntityDirection::South);

        // Bridge tiles (west-flow):
        // bridge_y = y_offset + 2+msz = 5; output_y = 6.
        // (1, 5) SOUTH, (2, 5) WEST, (3, 5) WEST
        let b0 = assert_entity(&entities, 1, 5, "transport-belt");
        assert_eq!(b0.direction, EntityDirection::South);
        let b1 = assert_entity(&entities, 2, 5, "transport-belt");
        assert_eq!(b1.direction, EntityDirection::West);
        let b2 = assert_entity(&entities, 3, 5, "transport-belt");
        assert_eq!(b2.direction, EntityDirection::West);
        // (1, 6) WEST, (2, 6) WEST, (3, 6) NORTH
        let b3 = assert_entity(&entities, 1, 6, "transport-belt");
        assert_eq!(b3.direction, EntityDirection::West);
        let b4 = assert_entity(&entities, 2, 6, "transport-belt");
        assert_eq!(b4.direction, EntityDirection::West);
        let b5 = assert_entity(&entities, 3, 6, "transport-belt");
        assert_eq!(b5.direction, EntityDirection::North);
    }

    #[test]
    fn single_input_row_lane_split_ignored_for_one_machine() {
        // lane_split with only 1 machine should be a no-op
        let (entities_split, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            true,
            false,
            None,
            0.5,
            0.5,
            None,
            InserterTier::default(),
        );
        let (entities_no_split, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
            None,
            0.5,
            0.5,
            None,
            InserterTier::default(),
        );
        assert_eq!(entities_split.len(), entities_no_split.len());
    }

    // ---- dual_input_row ----

    #[test]
    fn dual_input_row_basic() {
        let (entities, height) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(height, 8);

        // Input belt 1 (far, y=0): only x=0 survives — long-handed inserter
        // picks at mx, so both x=1 and x=2 are east-tail orphans.
        let e = assert_entity(&entities, 0, 0, "transport-belt");
        assert_eq!(e.direction, EntityDirection::East);
        assert_eq!(e.carries.as_deref(), Some("copper-cable"));
        assert!(entities.iter().find(|e| e.x == 1 && e.y == 0).is_none());
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none());

        // Input belt 2 (close, y=1): regular inserter picks at mx+2, so all 3
        // tiles remain (trim = 0 for msz=3).
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 1, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
            assert_eq!(e.carries.as_deref(), Some("iron-plate"));
        }

        // Long-handed inserter at (0, 2) SOUTH, carries copper-cable
        let lh = assert_entity(&entities, 0, 2, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        assert_eq!(lh.carries.as_deref(), Some("copper-cable"));

        // Regular inserter at (2, 2) SOUTH, carries iron-plate
        let ri = assert_entity(&entities, 2, 2, "inserter");
        assert_eq!(ri.direction, EntityDirection::South);
        assert_eq!(ri.carries.as_deref(), Some("iron-plate"));

        // Machine at (0, 3)
        assert_entity(&entities, 0, 3, "assembling-machine-3");

        // Output inserter at (1, 6) SOUTH
        let oi = assert_entity(&entities, 1, 6, "inserter");
        assert_eq!(oi.direction, EntityDirection::South);

        // Output belts at y=7: x=0,1 (x=2 is trimmed, west-flow tail past drop).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 7, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 7).is_none());
    }

    // ---- dual_input_row: ladder + reassignment (RFP Phase 2) ----

    #[test]
    fn dual_input_row_near_upgrades_to_stack_far_stays_single_lhi() {
        // Near (iron-plate) at 3.0/s exceeds fast's 2.31/s ceiling ->
        // stack. Far (copper-cable) at 1.0/s wins the contested column
        // (its relative shortfall against 1.2/s beats near's zero
        // shortfall against 12/s) but doesn't NEED the extra slot — one
        // long-handed inserter (1.2/s) already covers 1.0/s, so it stays
        // at count=1 (cheapest-sufficient, not "hardcoded" — Phase 3's
        // far ladder is active, it just doesn't need to escalate here).
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            1.0, // far_rate (copper-cable)
            3.0, // near_rate (iron-plate)
            0.5, // output_rate
            InserterTier::default(),
        );
        let near = assert_entity(&entities, 2, 2, "stack-inserter");
        assert_eq!(near.carries.as_deref(), Some("iron-plate"));
        let far = assert_entity(&entities, 0, 2, "long-handed-inserter");
        assert_eq!(far.carries.as_deref(), Some("copper-cable"));
        assert_eq!(entities.iter().filter(|e| e.y == 2 && e.name == "stack-inserter").count(), 1);
        assert_eq!(entities.iter().filter(|e| e.y == 2 && e.name == "long-handed-inserter").count(), 1);
    }

    #[test]
    fn dual_input_row_far_wins_contest_and_needs_it_places_second_lhi() {
        // Far (copper-cable) at 2.0/s exceeds one long-handed inserter's
        // 1.2/s ceiling and wins the contested column (its relative
        // shortfall beats near's zero shortfall against 12/s) -> a
        // second long-handed inserter genuinely lands at dx=1. Requires
        // 2 machines: a lone machine is always LastInRow, where the far
        // belt itself is trimmed and far is never eligible for the
        // contested column — checked on machine 0 (Interior, mx=0).
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            2,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            2.0, // far_rate (copper-cable) -- exceeds 1.2, needs 2 LHIs
            0.5, // near_rate (iron-plate)
            0.5, // output_rate
            InserterTier::default(),
        );
        let far_lhis: Vec<_> = entities
            .iter()
            .filter(|e| e.x < 3 && e.y == 2 && e.name == "long-handed-inserter" && e.carries.as_deref() == Some("copper-cable"))
            .collect();
        assert_eq!(far_lhis.len(), 2, "far should place 2 LHIs to cover 2.0/s: {far_lhis:?}");
        let far_xs: Vec<i32> = far_lhis.iter().map(|e| e.x).collect();
        assert!(far_xs.contains(&0), "baseline dx=0 must be present: {far_xs:?}");
        assert!(far_xs.contains(&1), "extra pick must land at the contested dx=1: {far_xs:?}");
        // Near still gets its single regular (0.5/s, well within 0.84/s).
        let near = assert_entity(&entities, 2, 2, "inserter");
        assert_eq!(near.carries.as_deref(), Some("iron-plate"));
    }

    #[test]
    fn dual_input_row_output_upgrades_to_fast() {
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5,
            0.5,
            1.5, // output_rate: exceeds regular (0.84), within fast (2.31)
            InserterTier::default(),
        );
        let out = assert_entity(&entities, 1, 6, "fast-inserter");
        assert_eq!(out.carries.as_deref(), Some("electronic-circuit"));
    }

    #[test]
    fn dual_input_row_max_tier_regular_never_places_fast_or_stack() {
        // Regular-capped: even a high near rate must degrade to
        // best-effort regular, never escalate past the user's cap.
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5,
            5.0,
            0.5,
            InserterTier::Regular,
        );
        let near = assert_entity(&entities, 2, 2, "inserter");
        assert_eq!(near.carries.as_deref(), Some("iron-plate"));
    }

    #[test]
    fn dual_input_row_lane_split_four_machines() {
        // 4 machines, Strategy A (tight pack, inline shift): machines
        // at x=0, 3, 6, 9. Anchor index = (4-1)/2 = 1 → mx_anchor = 3.
        // Bridge cols [4, 7).
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            4,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            true,
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );

        // Machines packed tight at x=0, 3, 6, 9
        assert_entity(&entities, 0, 3, "assembling-machine-3");
        assert_entity(&entities, 3, 3, "assembling-machine-3");
        assert_entity(&entities, 6, 3, "assembling-machine-3");
        assert_entity(&entities, 9, 3, "assembling-machine-3");

        // Anchor's output inserter shifted from mx+1=4 to mx=3.
        // Output inserter row at y_offset+3+msz=6.
        let anchor_ins = assert_entity(&entities, 3, 6, "inserter");
        assert_eq!(anchor_ins.direction, EntityDirection::South);

        // Bridge tiles (west-flow) at cols 4, 5, 6:
        // bridge_y = 0 + 3+msz = 6; output_y = 7.
        // (4, 6) SOUTH, (5, 6) WEST, (6, 6) WEST
        let b0 = assert_entity(&entities, 4, 6, "transport-belt");
        assert_eq!(b0.direction, EntityDirection::South);
        let b1 = assert_entity(&entities, 5, 6, "transport-belt");
        assert_eq!(b1.direction, EntityDirection::West);
        // (4, 7) WEST, (6, 7) NORTH (bridge bottom row)
        let b3 = assert_entity(&entities, 4, 7, "transport-belt");
        assert_eq!(b3.direction, EntityDirection::West);
        let b5 = assert_entity(&entities, 6, 7, "transport-belt");
        assert_eq!(b5.direction, EntityDirection::North);
    }

    #[test]
    fn tail_orphans_trimmed_dual_input_row() {
        // Regression: belts east of the last inserter pickup/drop must not be
        // stamped. Dual-input row at mx=10, msz=3: long-handed picks at mx=10,
        // regular at mx+2=12, output drop at mx+1=11.
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,   // y_offset
            10,  // x_offset
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false, // west-flow output
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // Belt 1 (y=0) — only x=10 survives; x=11 and x=12 are orphan tail.
        assert!(entities.iter().any(|e| e.x == 10 && e.y == 0 && e.carries.as_deref() == Some("copper-cable")));
        assert!(entities.iter().find(|e| e.x == 11 && e.y == 0).is_none());
        assert!(entities.iter().find(|e| e.x == 12 && e.y == 0).is_none());
        // Belt 2 (y=1) — regular picks at mx+2=12, no trim.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 1 && e.carries.as_deref() == Some("iron-plate")));
        }
        // Output belt (y=7, west-flow) — drop at mx+1=11, x=12 trimmed.
        assert!(entities.iter().any(|e| e.x == 10 && e.y == 7));
        assert!(entities.iter().any(|e| e.x == 11 && e.y == 7));
        assert!(entities.iter().find(|e| e.x == 12 && e.y == 7).is_none());
    }

    #[test]
    fn east_flow_output_tail_preserved_dual_input_row() {
        // When output flows EAST, tiles east of the drop are on the path to
        // the output merger — they must NOT be trimmed.
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            10,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            true, // east-flow output
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // Output belt at y=7, x=10,11,12 — all three preserved.
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, 10 + dx, 7, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
        }
    }

    // ---- triple_input_row ----

    #[test]
    fn triple_input_row_basic() {
        let (entities, height) = triple_input_row(
            "advanced-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "plastic-bar", "iron-plate"),
            "advanced-circuit",
            ("transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(height, 9);

        // Input belt 1 at y=0 (copper-cable): only x=0 — long-handed picks at mx.
        let e = assert_entity(&entities, 0, 0, "transport-belt");
        assert_eq!(e.carries.as_deref(), Some("copper-cable"));
        assert!(entities.iter().find(|e| e.x == 1 && e.y == 0).is_none());
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none());
        // Input belt 2 at y=1 (plastic-bar): regular picks at mx+2, all 3 tiles.
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 1, "transport-belt");
            assert_eq!(e.carries.as_deref(), Some("plastic-bar"));
        }
        // Long-handed inserter at (0, 2) SOUTH
        let lh = assert_entity(&entities, 0, 2, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        // Regular inserter at (2, 2) SOUTH
        let ri = assert_entity(&entities, 2, 2, "inserter");
        assert_eq!(ri.direction, EntityDirection::South);
        // Machine at (0, 3)
        assert_entity(&entities, 0, 3, "assembling-machine-3");
        // Output inserter at (1, 6) SOUTH
        let oi = assert_entity(&entities, 1, 6, "inserter");
        assert_eq!(oi.direction, EntityDirection::South);
        // Long-handed inserter at (2, 6) NORTH (picks iron-plate from y+8)
        let lh3 = assert_entity(&entities, 2, 6, "long-handed-inserter");
        assert_eq!(lh3.direction, EntityDirection::North);
        assert_eq!(lh3.carries.as_deref(), Some("iron-plate"));
        // Output belt at y=7: west-flow, x=0,1 (x=2 is trimmed).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 7, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 7).is_none());
        // Input belt 3 at y=8 (iron-plate): long-handed picks at mx+2, all 3 tiles.
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 8, "transport-belt");
            assert_eq!(e.carries.as_deref(), Some("iron-plate"));
        }
    }

    // ---- triple_input_row: ladder (RFP Phase 2) ----

    #[test]
    fn triple_input_row_near_upgrades_output_upgrades_input3_stays_single_lhi() {
        let (entities, _) = triple_input_row(
            "advanced-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "plastic-bar", "iron-plate"),
            "advanced-circuit",
            ("transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5, // far_rate (copper-cable) -- doesn't need the extra
            3.0, // near_rate (plastic-bar) -- exceeds fast, needs stack
            0.5, // input3_rate (iron-plate) -- wins the output contest but doesn't need the extra
            1.5, // output_rate -- exceeds regular, needs fast
            InserterTier::default(),
        );
        let near = assert_entity(&entities, 2, 2, "stack-inserter");
        assert_eq!(near.carries.as_deref(), Some("plastic-bar"));
        let far = assert_entity(&entities, 0, 2, "long-handed-inserter");
        assert_eq!(far.carries.as_deref(), Some("copper-cable"));
        let out = assert_entity(&entities, 1, 6, "fast-inserter");
        assert_eq!(out.carries.as_deref(), Some("advanced-circuit"));
        // Input3 wins the contested column (0.5/s vs 1.2/s ceiling beats
        // output's 1.5/s vs 12/s), but doesn't NEED the extra slot -- one
        // LHI already covers it, so it stays at count=1.
        let in3 = assert_entity(&entities, 2, 6, "long-handed-inserter");
        assert_eq!(in3.carries.as_deref(), Some("iron-plate"));
        assert_eq!(entities.iter().filter(|e| e.y == 6 && e.name == "long-handed-inserter").count(), 1);
    }

    #[test]
    fn triple_input_row_input3_wins_contest_and_needs_it_places_second_lhi() {
        // Input3 (iron-plate) at 2.0/s exceeds one LHI's 1.2/s ceiling and
        // wins the contested column (its relative shortfall beats
        // output's zero shortfall against 12/s) -> a second LHI genuinely
        // lands at the shared dx (mx+0, since input3's baseline sits at
        // mx+2 and output's baseline at mx+1, non-anchor).
        let (entities, _) = triple_input_row(
            "advanced-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "plastic-bar", "iron-plate"),
            "advanced-circuit",
            ("transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5,
            0.5,
            2.0, // input3_rate -- exceeds 1.2, needs 2 LHIs
            0.5, // output_rate
            InserterTier::default(),
        );
        let in3_lhis: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 6 && e.name == "long-handed-inserter" && e.carries.as_deref() == Some("iron-plate"))
            .collect();
        assert_eq!(in3_lhis.len(), 2, "input3 should place 2 LHIs to cover 2.0/s: {in3_lhis:?}");
        let in3_xs: Vec<i32> = in3_lhis.iter().map(|e| e.x).collect();
        assert!(in3_xs.contains(&2), "baseline dx=2 must be present: {in3_xs:?}");
        assert!(in3_xs.contains(&0), "extra pick must land at the shared dx=0: {in3_xs:?}");
        // Output still gets its single regular (0.5/s, within 0.84/s).
        let out = assert_entity(&entities, 1, 6, "inserter");
        assert_eq!(out.carries.as_deref(), Some("advanced-circuit"));
    }

    // ---- quad_input_row ----

    #[test]
    fn quad_input_row_basic() {
        // 1 machine, msz=3, west-flow output. Items 1..4 placeholder names.
        let (entities, height) = quad_input_row(
            "flying-robot-frame",
            "assembling-machine-3",
            3,
            1,
            0, // y_offset
            0, // x_offset
            ("input1", "input2", "input3", "input4"),
            "flying-robot-frame",
            ("transport-belt", "transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false, // output_east → west-flow
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(height, 10);

        // Belt 1 at y=0: continuous x=0..1 (LHI-A picks at mx+1, x=2 trimmed).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 0, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
            assert_eq!(e.carries.as_deref(), Some("input1"));
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none(),
            "x=2,y=0 should be trimmed (east-tail)");

        // Belt 2 at y=1: continuous x=0..1 (LHI-B picks at mx+1, x=2 trimmed).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 1, "transport-belt");
            assert_eq!(e.carries.as_deref(), Some("input2"));
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 1).is_none());

        // Belt 3 at y=2: UG-IN at x=0, LHI-A at x=1 (sits in gap), UG-OUT at x=2.
        let ug_in = assert_entity(&entities, 0, 2, "underground-belt");
        assert_eq!(ug_in.direction, EntityDirection::East);
        assert_eq!(ug_in.io_type.as_deref(), Some("input"));
        assert_eq!(ug_in.carries.as_deref(), Some("input3"));
        let ug_out = assert_entity(&entities, 2, 2, "underground-belt");
        assert_eq!(ug_out.direction, EntityDirection::East);
        assert_eq!(ug_out.io_type.as_deref(), Some("output"));

        // LHI-A at (1, 2) facing south, carrying input1.
        let lha = assert_entity(&entities, 1, 2, "long-handed-inserter");
        assert_eq!(lha.direction, EntityDirection::South);
        assert_eq!(lha.carries.as_deref(), Some("input1"));

        // Inserter row y=3: LHI-B at x=1, regulars at x=0 and x=2.
        let lhb = assert_entity(&entities, 1, 3, "long-handed-inserter");
        assert_eq!(lhb.direction, EntityDirection::South);
        assert_eq!(lhb.carries.as_deref(), Some("input2"));
        for &dx in &[0i32, 2] {
            let r = assert_entity(&entities, dx, 3, "inserter");
            assert_eq!(r.direction, EntityDirection::South);
            assert_eq!(r.carries.as_deref(), Some("input3"));
        }

        // Machine at (0, 4)
        let m = assert_entity(&entities, 0, 4, "assembling-machine-3");
        assert_eq!(m.direction, EntityDirection::North);
        assert_eq!(m.recipe.as_deref(), Some("flying-robot-frame"));

        // South inserter row y=7: output inserter at x=1, south LHI at x=2.
        let oi = assert_entity(&entities, 1, 7, "inserter");
        assert_eq!(oi.direction, EntityDirection::South);
        assert_eq!(oi.carries.as_deref(), Some("flying-robot-frame"));
        let south_lh = assert_entity(&entities, 2, 7, "long-handed-inserter");
        assert_eq!(south_lh.direction, EntityDirection::North);
        assert_eq!(south_lh.carries.as_deref(), Some("input4"));

        // Output belt at y=8: west-flow, x=0,1 (x=2 trimmed).
        for dx in 0..2_i32 {
            let b = assert_entity(&entities, dx, 8, "transport-belt");
            assert_eq!(b.direction, EntityDirection::West);
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 8).is_none());

        // Belt 4 (south input) at y=9: continuous x=0..2.
        for dx in 0..3_i32 {
            let b = assert_entity(&entities, dx, 9, "transport-belt");
            assert_eq!(b.direction, EntityDirection::East);
            assert_eq!(b.carries.as_deref(), Some("input4"));
        }
    }

    // ---- quad_input_row: ladder (RFP Phase 2) ----

    #[test]
    fn quad_input_row_input3_upgrades_both_slots_output_upgrades_input4_stays_single_lhi() {
        // input3_rate=3.0 -> 1.5/slot -> exceeds regular (0.84), within
        // fast (2.31) -> BOTH mx+0 and mx+2 upgrade to fast, in lockstep.
        let (entities, _) = quad_input_row(
            "flying-robot-frame",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("input1", "input2", "input3", "input4"),
            "flying-robot-frame",
            ("transport-belt", "transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            3.0, // input3_rate
            0.5, // input4_rate -- wins the output contest but doesn't need the extra
            1.5, // output_rate -- exceeds regular, needs fast
            InserterTier::default(),
        );
        for &dx in &[0i32, 2] {
            let e = assert_entity(&entities, dx, 3, "fast-inserter");
            assert_eq!(e.carries.as_deref(), Some("input3"));
        }
        // input2's own LHI at mx+1 on the same row is untouched.
        let lhb = assert_entity(&entities, 1, 3, "long-handed-inserter");
        assert_eq!(lhb.carries.as_deref(), Some("input2"));
        let out = assert_entity(&entities, 1, 7, "fast-inserter");
        assert_eq!(out.carries.as_deref(), Some("flying-robot-frame"));
        let south_lh = assert_entity(&entities, 2, 7, "long-handed-inserter");
        assert_eq!(south_lh.carries.as_deref(), Some("input4"));
        assert_eq!(entities.iter().filter(|e| e.y == 7 && e.name == "long-handed-inserter").count(), 1);
    }

    #[test]
    fn quad_input_row_input4_wins_contest_and_needs_it_places_second_lhi() {
        // input4 at 2.0/s exceeds one LHI's 1.2/s ceiling and wins the
        // contested column -> a second LHI lands at the shared dx=0.
        let (entities, _) = quad_input_row(
            "flying-robot-frame",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("input1", "input2", "input3", "input4"),
            "flying-robot-frame",
            ("transport-belt", "transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5,
            2.0, // input4_rate -- exceeds 1.2, needs 2 LHIs
            0.5, // output_rate
            InserterTier::default(),
        );
        let in4_lhis: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 7 && e.name == "long-handed-inserter" && e.carries.as_deref() == Some("input4"))
            .collect();
        assert_eq!(in4_lhis.len(), 2, "input4 should place 2 LHIs to cover 2.0/s: {in4_lhis:?}");
        let in4_xs: Vec<i32> = in4_lhis.iter().map(|e| e.x).collect();
        assert!(in4_xs.contains(&2), "baseline dx=2 must be present: {in4_xs:?}");
        assert!(in4_xs.contains(&0), "extra pick must land at the shared dx=0: {in4_xs:?}");
        let out = assert_entity(&entities, 1, 7, "inserter");
        assert_eq!(out.carries.as_deref(), Some("flying-robot-frame"));
    }

    #[test]
    fn quad_input_row_two_machines_belt3_uses_chain_of_ugs() {
        // 2 machines: belt 3 should be a chain UG-IN, [LHI-A], UG-OUT,
        // UG-IN (next machine), [LHI-A], UG-OUT — UG-OUT of m1 abuts UG-IN
        // of m2 at adjacent tiles.
        let (entities, _) = quad_input_row(
            "flying-robot-frame",
            "assembling-machine-3",
            3,
            2, // 2 machines
            0,
            0,
            ("input1", "input2", "input3", "input4"),
            "flying-robot-frame",
            ("transport-belt", "transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );

        // m1: UG-IN @ (0,2), LHI-A @ (1,2), UG-OUT @ (2,2).
        // m2: UG-IN @ (3,2), LHI-A @ (4,2), UG-OUT @ (5,2).
        for &(x, kind, io) in &[
            (0, "underground-belt", "input"),
            (2, "underground-belt", "output"),
            (3, "underground-belt", "input"),
            (5, "underground-belt", "output"),
        ] {
            let e = assert_entity(&entities, x, 2, kind);
            assert_eq!(e.io_type.as_deref(), Some(io),
                "io_type at ({x},2) should be {io}");
        }
        for &x in &[1i32, 4] {
            let lh = assert_entity(&entities, x, 2, "long-handed-inserter");
            assert_eq!(lh.direction, EntityDirection::South);
            assert_eq!(lh.carries.as_deref(), Some("input1"));
        }
        // UG-OUT of m1 at (2,2) and UG-IN of m2 at (3,2) are adjacent
        // tiles — surface flow continues across the seam.
    }

    #[test]
    fn quad_input_row_height_matches_row_kind() {
        let (_, h) = quad_input_row(
            "flying-robot-frame",
            "assembling-machine-3",
            3, 1, 0, 0,
            ("a", "b", "c", "d"),
            "flying-robot-frame",
            ("transport-belt", "transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false, false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(h, crate::bus::placer::RowKind::QuadInput.row_height());
    }

    // ---- fluid_input_row ----

    #[test]
    fn fluid_input_row_last_in_row_has_zero_extra_column_budget() {
        // 2 chemical-plants (port_dx=0, inserter_dx=1, free dx=2):
        // machine 0 is Interior (extra column available), machine 1 is
        // LastInRow (belt trimmed away under the free column, budget=0).
        // A rate that needs the extra column to avoid a shortfall (15.0/s
        // exceeds one stack inserter's 12.0/s ceiling) proves the
        // distinction: machine 0 gets 2 stack inserters, machine 1 stays
        // at 1 with an honest shortfall.
        let (entities, _, _) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            2,
            0,
            0,
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
            15.0, // solid_rate
            0.5,  // output_rate
            InserterTier::default(),
        );
        // Machine 0 (mx=0, Interior): 2 stack inserters at dx=1 (baseline)
        // and dx=2 (extra).
        let m0_stacks: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 3 && e.x < 3 && e.name == "stack-inserter")
            .collect();
        assert_eq!(m0_stacks.len(), 2, "Interior machine should use its extra column: {m0_stacks:?}");
        let m0_xs: Vec<i32> = m0_stacks.iter().map(|e| e.x).collect();
        assert!(m0_xs.contains(&1) && m0_xs.contains(&2), "{m0_xs:?}");

        // Machine 1 (mx=3, LastInRow): exactly 1 stack inserter — the
        // free column's belt tile is trimmed away, no extra available.
        let m1_stacks: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 3 && e.x >= 3 && e.name == "stack-inserter")
            .collect();
        assert_eq!(m1_stacks.len(), 1, "LastInRow has zero extra-column budget: {m1_stacks:?}");
        assert_eq!(m1_stacks[0].x, 4, "baseline dx=1 (mx+1=4)");
    }

    #[test]
    fn fluid_input_row_chemical_plant() {
        // T-shape layout: 2 chemical plants, y_offset=0
        //   y+0: T-junction pipe at x=0 (trunk row, bus tap-point)
        //   y+1: UG pipe IN at x=0 facing south
        //   y+2: solid belt
        //   y+3: UG pipe OUT at x=0 + inserter at x=1
        //   y+4..y+6: machine
        //   y+7: output inserter
        //   y+8: output belt
        let (entities, height, fluid_port_pipes) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            2,
            0,
            0,
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(height, 9); // msz + 6 = 3 + 6

        // fluid_port_pipes reports the TRUNK ROW position for ALL machines
        // (bus router chains horizontal PTG pairs to these T-junction pipe tiles)
        assert_eq!(fluid_port_pipes, vec![
            ("petroleum-gas".to_string(), 0, 0),
            ("petroleum-gas".to_string(), 3, 0),
        ]);

        // Machine 1: T-junction pipe at (0, 0) — trunk row
        let tj = assert_entity(&entities, 0, 0, "pipe");
        assert_eq!(tj.carries.as_deref(), Some("petroleum-gas"));

        // Machine 1: UG pipe IN at (0, 1) facing South, io=input
        let ptg_in = assert_entity(&entities, 0, 1, "pipe-to-ground");
        assert_eq!(ptg_in.direction, EntityDirection::South);
        assert_eq!(ptg_in.io_type.as_deref(), Some("input"));
        assert_eq!(ptg_in.carries.as_deref(), Some("petroleum-gas"));

        // Machine 1: solid belt at y=2
        for dx in 0..3_i32 {
            let b = assert_entity(&entities, dx, 2, "transport-belt");
            assert_eq!(b.carries.as_deref(), Some("coal"));
        }

        // Machine 1: UG pipe OUT at (0, 3) facing North (back toward input), io=output
        let ptg_out = assert_entity(&entities, 0, 3, "pipe-to-ground");
        assert_eq!(ptg_out.direction, EntityDirection::North);
        assert_eq!(ptg_out.io_type.as_deref(), Some("output"));
        assert_eq!(ptg_out.carries.as_deref(), Some("petroleum-gas"));

        // Machine 1: inserter at (1, 3) — different column from UG pipe
        let ins = assert_entity(&entities, 1, 3, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("coal"));

        // Machine 1 at (0, 4) NORTH
        let mach = assert_entity(&entities, 0, 4, "chemical-plant");
        assert_eq!(mach.direction, EntityDirection::North);

        // Machine 2: T-junction pipe at (3, 0)
        assert_entity(&entities, 3, 0, "pipe");

        // Machine 2: UG pipe IN at (3, 1) facing South
        let ptg2_in = assert_entity(&entities, 3, 1, "pipe-to-ground");
        assert_eq!(ptg2_in.direction, EntityDirection::South);
        assert_eq!(ptg2_in.io_type.as_deref(), Some("input"));

        // Machine 2: UG pipe OUT at (3, 3) facing North (back toward input)
        let ptg2_out = assert_entity(&entities, 3, 3, "pipe-to-ground");
        assert_eq!(ptg2_out.direction, EntityDirection::North);
        assert_eq!(ptg2_out.io_type.as_deref(), Some("output"));

        // Machine 2 at (3, 4)
        assert_entity(&entities, 3, 4, "chemical-plant");

        // Output inserter at y=7
        assert_entity(&entities, 1, 7, "inserter");
        // Output belt at y=8
        for dx in 0..3_i32 {
            let b = assert_entity(&entities, dx, 8, "transport-belt");
            assert_eq!(b.direction, EntityDirection::West);
        }
    }

    #[test]
    fn fluid_input_row_chemical_plant_row_height_matches_row_kind() {
        // Verify that the row_height returned by fluid_input_row matches
        // RowKind::FluidInput::row_height() for a 3x3 machine.
        let (_, height, _) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            1,
            0,
            0,
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            InserterTier::default(),
        );
        use crate::bus::placer::RowKind;
        assert_eq!(height, RowKind::FluidInput.row_height());
    }

    #[test]
    fn self_loop_row_fluid_height_matches_row_kind() {
        // Verify that the row_height returned by self_loop_row with a
        // fluid_input (1-item shape) matches
        // RowKind::SelfLoop{has_minor:false, has_fluid:true}.row_height().
        let (_, height, port_pipes) = self_loop_row(
            "pentapod-egg",
            "biochamber",
            3,
            1,
            0,
            0,
            "pentapod-egg",
            0.1333,
            0.2667,
            "nutrients",
            0.1333,
            None, // minor
            Some(("water", 8.0)),
            None, // max_belt_tier
            InserterTier::default(),
        );
        use crate::bus::placer::RowKind;
        let expected = RowKind::SelfLoop { has_minor: false, has_fluid: true }.row_height();
        assert_eq!(expected, 12, "row_height formula sanity: 11 (no-fluid base) + 1");
        assert_eq!(height, expected);
        assert_eq!(port_pipes.len(), 1, "expected 1 fluid tap point for 1 machine");
        assert_eq!(port_pipes[0].0, "water");
    }

    #[test]
    fn self_loop_row_no_fluid_no_minor_near_uses_extra_column() {
        // near_net_rate=15.0 exceeds one stack inserter's 12.0/s ceiling;
        // the uncontested extra column (dx=1 — major's own demand is
        // check-invisible, so near is credited it unconditionally, per
        // the census's documented "optimistic" caveat) lets a second
        // stack inserter cover it.
        let (entities, _, _) = self_loop_row(
            "iron-bacteria",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-bacteria",
            1.0,
            2.0,
            "bioflux",
            15.0,
            None,
            None,
            None,
            InserterTier::default(),
        );
        // PREFIX=3 dead columns west of machine 0 -> mx0 = x_offset + 3 = 3.
        let near_stacks: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 4 && e.name == "stack-inserter" && e.carries.as_deref() == Some("bioflux"))
            .collect();
        assert_eq!(near_stacks.len(), 2, "near should use its extra column: {near_stacks:?}");
        let xs: Vec<i32> = near_stacks.iter().map(|e| e.x).collect();
        assert!(xs.contains(&4) && xs.contains(&5), "baseline dx=2 (mx0+2=5) + extra dx=1 (mx0+1=4): {xs:?}");
        // Major's own LHI (check-invisible input demand) stays untouched.
        let major_lhi = assert_entity(&entities, 3, 4, "long-handed-inserter");
        assert_eq!(major_lhi.carries.as_deref(), Some("iron-bacteria"));
    }

    #[test]
    fn self_loop_row_has_fluid_near_is_lhi_hard_zero_budget() {
        // near_net_rate=15.0 would need 2 LHIs (>1.2/s) if any budget
        // existed, but the has-fluid shape's near row (PTG-in, LHI,
        // PTG-out) is 100% packed — exactly 1 LHI, best-effort +
        // shortfall, never a second.
        let (entities, _, _) = self_loop_row(
            "pentapod-egg",
            "biochamber",
            3,
            1,
            0,
            0,
            "pentapod-egg",
            0.1333,
            0.2667,
            "nutrients",
            15.0,
            None,
            Some(("water", 8.0)),
            None,
            InserterTier::default(),
        );
        let near_lhis: Vec<_> = entities
            .iter()
            .filter(|e| e.name == "long-handed-inserter" && e.carries.as_deref() == Some("nutrients"))
            .collect();
        assert_eq!(near_lhis.len(), 1, "hard 0 budget -- exactly 1 LHI even at a high rate: {near_lhis:?}");
    }

    #[test]
    fn self_loop_row_has_minor_output_contest_major_wins() {
        // major_produced_rate=15.0 (large relative shortfall against its
        // 12.0/s stack ceiling) vs minor_produced_rate=0.1 (well within
        // its 1.2/s LHI ceiling, zero shortfall) -> major wins the one
        // shared output column.
        let (entities, _, _) = self_loop_row(
            "major-item",
            "centrifuge",
            3,
            1,
            0,
            0,
            "major-item",
            1.0,
            15.0,
            "near-item",
            0.5,
            Some((0.2, 0.1)),
            None,
            None,
            InserterTier::default(),
        );
        // dy_out_ins for the has-minor shape = dy_machine(6) + msz(3) = 9.
        // PREFIX=3 dead columns west of machine 0 -> mx0 = x_offset + 3 = 3;
        // baseline dx=1 -> x=4, shared extra dx=2 -> x=5. Filter to
        // inserters only — the far/near2 belts also carry "major-item"/
        // "near-item" and pass through y=9 elsewhere in the row.
        let major_at_out: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 9 && e.name.contains("inserter") && e.carries.as_deref() == Some("major-item"))
            .collect();
        let major_xs: Vec<i32> = major_at_out.iter().map(|e| e.x).collect();
        assert!(
            major_xs.contains(&4) && major_xs.contains(&5),
            "major should win the contest and use the shared extra column (mx0+1=4, mx0+2=5): {major_xs:?}"
        );
        // Minor's export stays at its baseline dx=0 only (lost the contest).
        let minor_at_out: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 9 && e.name == "long-handed-inserter" && e.carries.as_deref() == Some("near-item"))
            .collect();
        assert_eq!(minor_at_out.len(), 1, "minor lost the contest, stays at baseline: {minor_at_out:?}");
        assert_eq!(minor_at_out[0].x, 3, "baseline dx=0 -> mx0+0=3");
    }

    #[test]
    #[should_panic(expected = "has_minor == false")]
    fn self_loop_row_fluid_input_with_minor_panics() {
        // fluid_input is only legal alongside has_minor == false — the
        // 2-item (kovarex) shape has no fluid-header row.
        self_loop_row(
            "some-recipe",
            "centrifuge",
            3,
            1,
            0,
            0,
            "major",
            1.0,
            2.0,
            "near",
            0.5,
            Some((0.5, 0.1)), // minor: has_minor == true
            Some(("water", 1.0)),
            None,
            InserterTier::default(),
        );
    }

    #[test]
    fn fluid_input_row_chemical_plant_ug_pair_alignment() {
        // UG pipe IN and UG pipe OUT must be in the same x column (port_dx=0),
        // and the inserter must be in a DIFFERENT column (x=1).
        let (entities, _, _) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            1,
            5, // y_offset=5
            10, // x_offset=10
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // T-junction pipe at (10, 5) — trunk row
        assert_entity(&entities, 10, 5, "pipe");
        // UG pipe IN at x=10+0=10, y=6 (trunk row + 1)
        let ptg_in = assert_entity(&entities, 10, 6, "pipe-to-ground");
        assert_eq!(ptg_in.io_type.as_deref(), Some("input"));
        // UG pipe OUT at x=10+0=10, y=8 (belt at y=7, UG OUT at y=8)
        let ptg_out = assert_entity(&entities, 10, 8, "pipe-to-ground");
        assert_eq!(ptg_out.io_type.as_deref(), Some("output"));
        // Same column
        assert_eq!(ptg_in.x, ptg_out.x, "UG pair must share the same x column");
        // Inserter at x=11 (different from UG column x=10)
        let ins = assert_entity(&entities, 11, 8, "inserter");
        assert_ne!(ins.x, ptg_in.x, "inserter must be in a different column from UG pipe");
        // Machine at (10, 9)
        assert_entity(&entities, 10, 9, "chemical-plant");
    }

    #[test]
    fn fluid_input_row_assembling_machine() {
        // AM2 uses the same T-junction pattern as chemical-plant now —
        // continuous pipe header at y+0, UG-in at y+1, solid belt at y+2,
        // UG-out + inserter at y+3. The only difference is
        // `port_dx == 1` (vs `port_dx == 0` for chemical-plant), so the
        // UG pair sits at the centre column and the inserter at
        // `mx + 0` (the first non-port column).
        let (entities, height, fluid_port_pipes) = fluid_input_row(
            "some-recipe",
            "assembling-machine-2",
            3,
            1,
            0,
            0,
            "solid-item",
            "fluid-item",
            "output-item",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // Unified T-junction height: msz + 6 = 9.
        assert_eq!(height, 9);
        // fluid_port_pipes reports a tile on the y+0 header row.
        assert_eq!(fluid_port_pipes, vec![("fluid-item".to_string(), 1, 0)]);

        // Continuous pipe row at y=0 spanning the full machine width.
        for x in 0..3_i32 {
            let pipe = assert_entity(&entities, x, 0, "pipe");
            assert_eq!(pipe.carries.as_deref(), Some("fluid-item"));
        }

        // UG-in South at (mx + port_dx = 1, y = 1) — tunnels under the
        // solid belt to reach the machine's fluid port.
        let ug_in = assert_entity(&entities, 1, 1, "pipe-to-ground");
        assert_eq!(ug_in.direction, EntityDirection::South);
        assert_eq!(ug_in.io_type.as_deref(), Some("input"));

        // UG-out North at (1, 3) — surface on south side, adjacent to
        // the machine's top-centre fluid port at (1, 4).
        let ug_out = assert_entity(&entities, 1, 3, "pipe-to-ground");
        assert_eq!(ug_out.direction, EntityDirection::North);
        assert_eq!(ug_out.io_type.as_deref(), Some("output"));

        // Input inserter at (mx + 0, y + 3) — `inserter_dx = 0` because
        // `port_dx = 1`. South-facing: picks from solid belt at y+2,
        // drops into machine at y+4.
        let ins = assert_entity(&entities, 0, 3, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("solid-item"));

        // Machine occupies rows 4..=6. Ports column is 1; top face is y=4.
        assert_entity(&entities, 0, 4, "assembling-machine-2");

        // Sanity: no entity overlap on y+1 or y+3 for x in 0..3.
        for y in [1_i32, 3] {
            for x in 0..3_i32 {
                let here: Vec<_> = entities.iter().filter(|e| e.x == x && e.y == y).collect();
                assert!(
                    here.len() <= 1,
                    "multiple entities at ({x}, {y}): {:?}",
                    here.iter().map(|e| e.name.as_str()).collect::<Vec<_>>()
                );
            }
        }
    }

    // ---- fluid_dual_input_row ----

    #[test]
    fn fluid_dual_input_row_ladder_hard_zero_budget_both_sides() {
        // Both solid sides have a hard 0 extra-column budget at every
        // position (2 solid inserters + the fluid PTG pack all 3 dx) —
        // high rates escalate TIER (in place) but never add a second
        // inserter, even at 2 machines (one Interior, one LastInRow).
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-solid-recipe",
            "chemical-plant",
            3,
            2,
            0,
            0,
            ("input1", "input2"),
            "fluid",
            "output",
            false, // output_is_fluid
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
            20.0, // far_rate -- would need 2 LHIs if a budget existed (>1.2)
            20.0, // near_rate -- would need 2 stacks if a budget existed (>12)
            0.5,
            InserterTier::default(),
        );
        // Far: long-handed, exactly 1 per machine (best-effort + shortfall,
        // never a second inserter).
        let far_lhis: Vec<_> = entities.iter().filter(|e| e.name == "long-handed-inserter").collect();
        assert_eq!(far_lhis.len(), 2, "1 far LHI per machine, no extra: {far_lhis:?}");
        // Near: stack tier (cheapest-sufficient ceiling below 20.0/s is
        // still stack — best-effort), exactly 1 per machine.
        let near_stacks: Vec<_> = entities.iter().filter(|e| e.name == "stack-inserter").collect();
        assert_eq!(near_stacks.len(), 2, "1 near stack per machine, no extra: {near_stacks:?}");
    }

    #[test]
    fn fluid_dual_input_row_solid_output() {
        let (entities, height, fluid_in_ports, fluid_out_ports) = fluid_dual_input_row(
            "some-solid-recipe",
            "chemical-plant",
            3,
            2,
            0,
            0,
            ("input1", "input2"),
            "fluid",
            "output",
            false, // output_is_fluid = false
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(height, 10);
        assert_eq!(fluid_in_ports, vec![("fluid".to_string(), 0, 0)]);
        assert!(fluid_out_ports.is_empty());

        // Fluid header at y=0, x=0..=last_mx+2 = 0..=3+2=5
        for x in 0..=5_i32 {
            let pipe = assert_entity(&entities, x, 0, "pipe");
            assert_eq!(pipe.carries.as_deref(), Some("fluid"));
        }

        // PTG input at (0+0, 1) = (0, 1) direction SOUTH for chemical-plant (port_dx=0)
        let ptg_in = assert_entity(&entities, 0, 1, "pipe-to-ground");
        assert_eq!(ptg_in.direction, EntityDirection::South);
        assert_eq!(ptg_in.io_type.as_deref(), Some("input"));

        // PTG output at (0+0, 4) = (0, 4) direction NORTH (faces input)
        let ptg_out = assert_entity(&entities, 0, 4, "pipe-to-ground");
        assert_eq!(ptg_out.direction, EntityDirection::North);
        assert_eq!(ptg_out.io_type.as_deref(), Some("output"));

        // Solid input belt 1 at y=2
        for dx in 0..3_i32 {
            assert_entity(&entities, dx, 2, "transport-belt");
        }
        // Solid input belt 2 at y=3
        for dx in 0..3_i32 {
            assert_entity(&entities, dx, 3, "transport-belt");
        }

        // Long-handed inserter at (1, 4) for chemical-plant (port_dx=0, long_x=mx+1=1)
        let lh = assert_entity(&entities, 1, 4, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        // Regular inserter at (2, 4)
        assert_entity(&entities, 2, 4, "inserter");

        // Machine at (0, 5)
        assert_entity(&entities, 0, 5, "chemical-plant");

        // Solid output: inserter at (1, 8), output belt at y=9
        assert_entity(&entities, 1, 8, "inserter");
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 9, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
        }
    }

    #[test]
    fn fluid_dual_input_row_fluid_output() {
        let (entities, height, fluid_in_ports, fluid_out_ports) = fluid_dual_input_row(
            "sulfuric-acid",
            "chemical-plant",
            3,
            1,
            0,
            0,
            ("iron-plate", "sulfur"),
            "water",
            "sulfuric-acid",
            true, // output_is_fluid = true
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(height, 10);
        assert_eq!(fluid_in_ports, vec![("water".to_string(), 0, 0)]);
        // 2 port positions reported (chemical-plant fluid output boxes at
        // dx=0 and dx=2) even though the emitted pipe row is continuous.
        assert_eq!(fluid_out_ports.len(), 2);
        assert!(fluid_out_ports.contains(&("sulfuric-acid".to_string(), 0, 8)));
        assert!(fluid_out_ports.contains(&("sulfuric-acid".to_string(), 2, 8)));

        // Continuous output pipe row at y=8 spanning x=0..=2
        assert_entity(&entities, 0, 8, "pipe");
        assert_entity(&entities, 1, 8, "pipe");
        assert_entity(&entities, 2, 8, "pipe");
    }

    #[test]
    fn fluid_dual_input_row_assembling_machine_inserter_positions() {
        // assembling-machine-2 has port_dx=1, so long_x=mx+2, reg_x=mx+0
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-recipe",
            "assembling-machine-2",
            3,
            1,
            0,
            0,
            ("input1", "input2"),
            "fluid",
            "output",
            false,
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // long-handed inserter at (2, 4), regular at (0, 4)
        let lh = assert_entity(&entities, 2, 4, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        let ri = assert_entity(&entities, 0, 4, "inserter");
        assert_eq!(ri.direction, EntityDirection::South);
    }

    #[test]
    fn fluid_dual_input_row_trim_chemical_plant() {
        // chemical-plant: port_dx=0 → long_dx=1, reg_dx=2.
        // Belt 1 (far, y+2) picked by long-hand at mx+1 → trim = msz-1-1 = 1.
        // Belt 2 (close, y+3) picked by regular at mx+2 → trim = 0.
        // Output (west-flow, y+9) drop at mx+1 → trim = 1.
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-solid-recipe",
            "chemical-plant",
            3,
            2,
            0,
            10, // x_offset so last_mx = 13
            ("input1", "input2"),
            "fluid",
            "output",
            false,
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false, // west-flow output
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // Machine 0 (mx=10) keeps all 3 tiles on every belt row.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 2),
                "belt 1 tile at (10+{dx}, 2) should exist on non-last machine");
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 3),
                "belt 2 tile at (10+{dx}, 3) should exist on non-last machine");
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 9),
                "output tile at (10+{dx}, 9) should exist on non-last machine");
        }
        // Machine 1 (last, mx=13): belt 1 trimmed at mx+2.
        assert!(entities.iter().any(|e| e.x == 13 && e.y == 2));
        assert!(entities.iter().any(|e| e.x == 14 && e.y == 2));
        assert!(entities.iter().find(|e| e.x == 15 && e.y == 2).is_none(),
            "belt 1 tile at (15, 2) should be trimmed (east of long-hand pickup at mx+1)");
        // Belt 2: regular at mx+2=15 → no trim, all 3 tiles remain.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 13 + dx && e.y == 3),
                "belt 2 tile at (13+{dx}, 3) should survive (no trim for reg_dx=2)");
        }
        // Output west-flow: drop at mx+1=14, tile at mx+2=15 is orphan.
        assert!(entities.iter().any(|e| e.x == 13 && e.y == 9));
        assert!(entities.iter().any(|e| e.x == 14 && e.y == 9));
        assert!(entities.iter().find(|e| e.x == 15 && e.y == 9).is_none(),
            "output tile at (15, 9) should be trimmed (west-flow, east of drop)");
    }

    #[test]
    fn fluid_dual_input_row_trim_assembling_machine() {
        // assembling-machine-3: port_dx=1 → long_dx=2, reg_dx=0.
        // Belt 1 (far, y+2) picked by long-hand at mx+2 → trim = 0.
        // Belt 2 (close, y+3) picked by regular at mx+0 → trim = msz-1-0 = 2.
        // Output (west-flow, y+9) drop at mx+1 → trim = 1.
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-recipe",
            "assembling-machine-3",
            3,
            2,
            0,
            10,
            ("input1", "input2"),
            "fluid",
            "output",
            false,
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
            0.5,
            0.5,
            0.5,
            InserterTier::default(),
        );
        // Non-last machine (mx=10) keeps all tiles.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 3),
                "belt 2 tile at (10+{dx}, 3) should exist on non-last machine");
        }
        // Last machine (mx=13): belt 1 all 3 tiles (no trim).
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 13 + dx && e.y == 2),
                "belt 1 tile at (13+{dx}, 2) should survive (no trim for long_dx=2)");
        }
        // Belt 2: only x=13 survives — reg inserter at mx+0=13 makes 14,15 orphan.
        assert!(entities.iter().any(|e| e.x == 13 && e.y == 3));
        assert!(entities.iter().find(|e| e.x == 14 && e.y == 3).is_none(),
            "belt 2 at (14, 3) should be trimmed (reg_dx=0, trim=2)");
        assert!(entities.iter().find(|e| e.x == 15 && e.y == 3).is_none(),
            "belt 2 at (15, 3) should be trimmed (reg_dx=0, trim=2)");
    }

    // ---- fluid_only_row ----

    // basic-oil-processing: 1 fluid input (crude-oil → input box 2, dx=3),
    // 1 fluid output (petroleum-gas → output box 3, dx=0).
    #[test]
    fn fluid_only_row_one_refinery_basic() {
        let (entities, height, fluid_in, fluid_out) = fluid_only_row(
            "basic-oil-processing",
            "oil-refinery",
            5,
            1,
            0,
            0,
            &[(3, "crude-oil")],
            &[(0, "petroleum-gas")],
        );
        assert_eq!(height, 7);
        // 1 reported input port (the dx=1 UG-bridge entry) + 1 output pipe
        assert_eq!(fluid_in.len(), 1);
        assert_eq!(fluid_out.len(), 1);
        // Input port reported at dx=1 (the leftmost UG-bridge entry).
        assert_eq!(fluid_in[0], ("crude-oil".to_string(), 1, 0));
        // Output pipe at dx=0 → (0, 6)
        assert_eq!(fluid_out[0], ("petroleum-gas".to_string(), 0, 6));

        // Input row uses the pole-gap UG bridge: pipe, UG-in, GAP, UG-out, pipe.
        let in_pipe_left = assert_entity(&entities, 0, 0, "pipe");
        assert_eq!(in_pipe_left.carries.as_deref(), Some("crude-oil"));
        let in_ug_left = assert_entity(&entities, 1, 0, "pipe-to-ground");
        assert_eq!(in_ug_left.direction, EntityDirection::East);
        assert_eq!(in_ug_left.io_type.as_deref(), Some("input"));
        let in_ug_right = assert_entity(&entities, 3, 0, "pipe-to-ground");
        assert_eq!(in_ug_right.direction, EntityDirection::West);
        assert_eq!(in_ug_right.io_type.as_deref(), Some("output"));
        // dx=2 left empty for `place_poles` to drop a medium-electric-pole.
        assert!(!entities.iter().any(|e| e.x == 2 && e.y == 0));

        // Refinery at (0, 1) NORTH mirrored
        let refinery = assert_entity(&entities, 0, 1, "oil-refinery");
        assert_eq!(refinery.direction, EntityDirection::North);
        assert!(refinery.mirror);
        assert_eq!(refinery.recipe.as_deref(), Some("basic-oil-processing"));

        // Output pipe at (0, 6)
        let out_pipe = assert_entity(&entities, 0, 6, "pipe");
        assert_eq!(out_pipe.carries.as_deref(), Some("petroleum-gas"));
    }

    // basic-oil-processing with two machines
    #[test]
    fn fluid_only_row_two_refineries_basic() {
        let (entities, _, fluid_in, fluid_out) = fluid_only_row(
            "basic-oil-processing",
            "oil-refinery",
            5,
            2,
            0,
            0,
            &[(3, "crude-oil")],
            &[(0, "petroleum-gas")],
        );
        // 2 machines × 1 reported input port (dx=1) + 2 machines × 1 output pipe
        assert_eq!(fluid_in.len(), 2);
        assert_eq!(fluid_out.len(), 2);

        // Second refinery at x=5 (pitch=5)
        assert_entity(&entities, 5, 1, "oil-refinery");
        // Second machine: input port reported at mx=5, dx=1 → (6, 0)
        assert_eq!(fluid_in[1], ("crude-oil".to_string(), 6, 0));
        // Second machine: output pipe at mx=5, dx=0 → (5, 6)
        assert_eq!(fluid_out[1], ("petroleum-gas".to_string(), 5, 6));
    }

    // advanced-oil-processing: 2 fluid inputs (water→dx=1, crude-oil→dx=3),
    // 3 fluid outputs (heavy-oil→dx=0, light-oil→dx=2, petroleum-gas→dx=4).
    // Uses the staggered staircase on BOTH input and output sides.
    #[test]
    fn fluid_only_row_one_refinery_advanced() {
        let (entities, height, fluid_in, fluid_out) = fluid_only_row(
            "advanced-oil-processing",
            "oil-refinery",
            5,
            1,
            0,
            0,
            &[(1, "water"), (3, "crude-oil")],
            &[(0, "heavy-oil"), (2, "light-oil"), (4, "petroleum-gas")],
        );
        // Layout (n_inputs=2, gap=1):
        //   y=0..4: input section (5 rows = n+2+gap)
        //   y=5..9: machine (5×5)
        //   y=10:   port row (output)
        //   y=11..13: output staircase (middle, east, west)
        // → height = 14 = msz + 5 + input_section(5) - 1 = ...
        //   actually y_west = port_row_y + 3 = 13, height = 13 - 0 + 1 = 14.
        assert_eq!(height, 14);
        assert_eq!(fluid_in.len(), 2);
        assert_eq!(fluid_out.len(), 3);

        // --- Input staircase (mirrored vertically vs output) ---
        // Outermost fluid (crude-oil at dx=3) anchors at y=0; water shifts down by 1+gap.
        //
        // Crude-oil trunk at y=0: UG-W at (2,0), pipe at (3,0), UG-E at (4,0).
        let c_l = assert_entity(&entities, 2, 0, "pipe-to-ground");
        assert_eq!(c_l.direction, EntityDirection::West);
        assert_eq!(c_l.carries.as_deref(), Some("crude-oil"));
        let c_t = assert_entity(&entities, 3, 0, "pipe");
        assert_eq!(c_t.carries.as_deref(), Some("crude-oil"));
        let c_r = assert_entity(&entities, 4, 0, "pipe-to-ground");
        assert_eq!(c_r.direction, EntityDirection::East);

        // Crude-oil drop UG-S at (3, 1).
        let c_drop = assert_entity(&entities, 3, 1, "pipe-to-ground");
        assert_eq!(c_drop.direction, EntityDirection::South);
        assert_eq!(c_drop.carries.as_deref(), Some("crude-oil"));

        // Water trunk at y=2: UG-W at (0,2), pipe at (1,2), UG-E at (2,2).
        let w_l_in = assert_entity(&entities, 0, 2, "pipe-to-ground");
        assert_eq!(w_l_in.direction, EntityDirection::West);
        assert_eq!(w_l_in.carries.as_deref(), Some("water"));
        let w_t_in = assert_entity(&entities, 1, 2, "pipe");
        assert_eq!(w_t_in.carries.as_deref(), Some("water"));
        let w_r_in = assert_entity(&entities, 2, 2, "pipe-to-ground");
        assert_eq!(w_r_in.direction, EntityDirection::East);

        // Water drop UG-S at (1, 3) — sits in the gap row.
        let w_drop_in = assert_entity(&entities, 1, 3, "pipe-to-ground");
        assert_eq!(w_drop_in.direction, EntityDirection::South);

        // UG-out row at y=4 (=machine_y-1): water at (1,4), crude-oil at (3,4),
        // both UG-N facing south to merge with machine ports.
        let w_out = assert_entity(&entities, 1, 4, "pipe-to-ground");
        assert_eq!(w_out.direction, EntityDirection::North);
        assert_eq!(w_out.carries.as_deref(), Some("water"));
        let c_out = assert_entity(&entities, 3, 4, "pipe-to-ground");
        assert_eq!(c_out.direction, EntityDirection::North);
        assert_eq!(c_out.carries.as_deref(), Some("crude-oil"));

        // Tap points report each input fluid's T-drop position.
        assert_eq!(fluid_in[0], ("water".to_string(), 1, 2));
        assert_eq!(fluid_in[1], ("crude-oil".to_string(), 3, 0));

        // --- Refinery at (0, 5) ---
        let refinery = assert_entity(&entities, 0, 5, "oil-refinery");
        assert_eq!(refinery.direction, EntityDirection::North);
        assert!(refinery.mirror);
        assert_eq!(refinery.recipe.as_deref(), Some("advanced-oil-processing"));

        // --- Output staircase ---
        // Port row (y=10): west UG-S/input at (0,10), middle pipe at (2,10),
        // east UG-S/input at (4,10). Each connects to its refinery output box.
        let w_port = assert_entity(&entities, 0, 10, "pipe-to-ground");
        assert_eq!(w_port.direction, EntityDirection::South);
        assert_eq!(w_port.io_type.as_deref(), Some("input"));
        assert_eq!(w_port.carries.as_deref(), Some("heavy-oil"));
        let m_port = assert_entity(&entities, 2, 10, "pipe");
        assert_eq!(m_port.carries.as_deref(), Some("light-oil"));
        let e_port = assert_entity(&entities, 4, 10, "pipe-to-ground");
        assert_eq!(e_port.direction, EntityDirection::South);
        assert_eq!(e_port.carries.as_deref(), Some("petroleum-gas"));

        // Middle trunk at y=11: L-flank (1,11), T-drop (2,11), R-flank (3,11).
        let m_l = assert_entity(&entities, 1, 11, "pipe-to-ground");
        assert_eq!(m_l.direction, EntityDirection::West);
        assert_eq!(m_l.io_type.as_deref(), Some("output"));
        let m_t = assert_entity(&entities, 2, 11, "pipe");
        assert_eq!(m_t.carries.as_deref(), Some("light-oil"));
        let m_r = assert_entity(&entities, 3, 11, "pipe-to-ground");
        assert_eq!(m_r.direction, EntityDirection::East);

        // East drop UG sits at (4,11) on the middle trunk row but on N-S axis
        // (F5a perpendicular to middle's E-W flanks — no fluid merge).
        let e_drop = assert_entity(&entities, 4, 11, "pipe-to-ground");
        assert_eq!(e_drop.direction, EntityDirection::North);
        assert_eq!(e_drop.carries.as_deref(), Some("petroleum-gas"));

        // East trunk at y=12.
        let e_l = assert_entity(&entities, 3, 12, "pipe-to-ground");
        assert_eq!(e_l.direction, EntityDirection::West);
        let e_t = assert_entity(&entities, 4, 12, "pipe");
        assert_eq!(e_t.carries.as_deref(), Some("petroleum-gas"));
        let e_r = assert_entity(&entities, 5, 12, "pipe-to-ground");
        assert_eq!(e_r.direction, EntityDirection::East);

        // West drop UG at (0,12) — N-S axis, doesn't conflict with east trunk
        // at y=12 (different columns).
        let w_drop = assert_entity(&entities, 0, 12, "pipe-to-ground");
        assert_eq!(w_drop.direction, EntityDirection::North);
        assert_eq!(w_drop.carries.as_deref(), Some("heavy-oil"));

        // West trunk at y=13.
        let w_l = assert_entity(&entities, -1, 13, "pipe-to-ground");
        assert_eq!(w_l.direction, EntityDirection::West);
        let w_t = assert_entity(&entities, 0, 13, "pipe");
        assert_eq!(w_t.carries.as_deref(), Some("heavy-oil"));
        let w_r = assert_entity(&entities, 1, 13, "pipe-to-ground");
        assert_eq!(w_r.direction, EntityDirection::East);

        // Tap points report each output fluid's T-drop pipe position.
        assert_eq!(fluid_out[0], ("heavy-oil".to_string(), 0, 13));
        assert_eq!(fluid_out[1], ("light-oil".to_string(), 2, 11));
        assert_eq!(fluid_out[2], ("petroleum-gas".to_string(), 4, 12));

        // No two entities share a tile (exclude machine anchor — 5×5 footprint
        // registered once at its origin).
        use std::collections::HashMap;
        let mut by_tile: HashMap<(i32, i32), Vec<&PlacedEntity>> = HashMap::new();
        for e in &entities {
            by_tile.entry((e.x, e.y)).or_default().push(e);
        }
        for (&(x, y), es) in &by_tile {
            let non_machine: Vec<_> = es.iter().filter(|e| e.name != "oil-refinery").collect();
            assert!(
                non_machine.len() <= 1,
                "tile ({x},{y}) has {} overlapping entities: {:?}",
                non_machine.len(),
                non_machine.iter().map(|e| &e.name).collect::<Vec<_>>(),
            );
        }
    }

    // Staggered 3-output now supports machine_count > 1 directly (issue
    // #277) instead of falling back to the non-staggered per-port-isolated
    // path. Regression guard: no panic, no two entities share a tile, and
    // the coordinate pair that used to collide at plain `machine_size`
    // pitch (machine N's east R-flank vs machine N+1's west drop UG) is
    // now exactly 1 tile apart thanks to `fluid_only_row_pitch`.
    #[test]
    fn fluid_only_row_advanced_multi_machine_staggered() {
        let (entities, row_height, in_ports, out_ports) = fluid_only_row(
            "advanced-oil-processing",
            "oil-refinery",
            5,
            2, // multi-machine — previously hit assert_eq!(machine_count, 1)
            0,
            0,
            &[(1, "water"), (3, "crude-oil")],
            &[(0, "heavy-oil"), (2, "light-oil"), (4, "petroleum-gas")],
        );
        // Staggered path, same row_height formula as the single-machine case
        // (row height doesn't grow with machine_count — only width does).
        assert_eq!(row_height, 14, "staggered multi-machine row height should match single-machine (msz+9)");

        // Pitch = msz+1 = 6 for the ≥3-distinct-output case, so machines land
        // at x=0 and x=6 (not x=0/x=5, which is where the collision was).
        let machine_xs: Vec<i32> = entities.iter()
            .filter(|e| e.name == "oil-refinery")
            .map(|e| e.x)
            .collect();
        assert!(machine_xs.contains(&0), "first machine at x=0; got {machine_xs:?}");
        assert!(machine_xs.contains(&6), "second machine at x=6 (pitch=msz+1); got {machine_xs:?}");

        // 3 distinct output fluids × 2 machines = 6 tap points; 2 distinct
        // input fluids × 2 machines = 4 tap points.
        assert_eq!(out_ports.len(), 6, "2 machines × 3 output ports = 6 out-port entries");
        assert_eq!(in_ports.len(), 4, "2 machines × 2 input ports = 4 in-port entries");

        // The previously-colliding tiles: machine 0's east R-flank UG (at
        // col_e0+1, y_east) and machine 1's west drop UG (at col_w1,
        // y_west-1 == y_east) are now 1 tile apart instead of identical.
        // col_e0 = 4 (dx=4), so east R-flank is at x=5; col_w1 = mx1+0 = 6.
        let y_east = 12; // port_row_y(10) + 2
        let east_r_flank = entities.iter()
            .find(|e| e.x == 5 && e.y == y_east && e.name == "pipe-to-ground" && e.direction == EntityDirection::East)
            .expect("machine 0 east R-flank UG at (5, y_east)");
        let west_drop = entities.iter()
            .find(|e| e.x == 6 && e.y == y_east && e.name == "pipe-to-ground" && e.direction == EntityDirection::North)
            .expect("machine 1 west drop UG at (6, y_east)");
        assert_ne!(
            (east_r_flank.x, east_r_flank.y), (west_drop.x, west_drop.y),
            "east R-flank and west drop UG must not share a tile",
        );
        assert_eq!(
            (west_drop.x - east_r_flank.x).abs(), 1,
            "east R-flank and west drop UG should be exactly 1 tile apart",
        );

        // No two entities share a tile (exclude machine anchors — 5×5
        // footprints registered once at their origin, one per machine).
        use std::collections::HashMap;
        let mut by_tile: HashMap<(i32, i32), Vec<&PlacedEntity>> = HashMap::new();
        for e in &entities {
            by_tile.entry((e.x, e.y)).or_default().push(e);
        }
        for (&(x, y), es) in &by_tile {
            let non_machine: Vec<_> = es.iter().filter(|e| e.name != "oil-refinery").collect();
            assert!(
                non_machine.len() <= 1,
                "tile ({x},{y}) has {} overlapping entities: {:?}",
                non_machine.len(),
                non_machine.iter().map(|e| &e.name).collect::<Vec<_>>(),
            );
        }
    }

    // (machine_xs helper deleted in the inline-bridge unification —
    // all templates now compute machine_xs directly with tight packing
    // or per-template Strategy-B gap insertion.)

    // ---- voider_row ----

    #[test]
    fn voider_row_ladder_hard_zero_budget_both_sides() {
        // 2-wide recycler footprint: near (dx=0) and far (dx=1) baselines
        // already claim both tiles -- high rates escalate TIER in place,
        // never add a second inserter.
        let (entities, _) = voider_row(
            "iron-scrap-recycling",
            "iron-plate",
            1,
            0,
            0,
            15.0, // near_rate_per_machine -- exceeds stack's 12.0/s
            5.0,  // far_rate_per_machine -- exceeds LHI's 1.2/s
            None,
            InserterTier::default(),
        );
        let near: Vec<_> = entities
            .iter()
            .filter(|e| e.name == "stack-inserter" && e.carries.as_deref() == Some("iron-plate"))
            .collect();
        assert_eq!(near.len(), 1, "exactly 1 near inserter, tier-upgraded: {near:?}");
        let far: Vec<_> = entities
            .iter()
            .filter(|e| e.name == "long-handed-inserter" && e.carries.as_deref() == Some("iron-plate"))
            .collect();
        assert_eq!(far.len(), 1, "exactly 1 far inserter, no extra column: {far:?}");
        // Adjacent columns (dx=0 near, dx=1 far), same row.
        assert_eq!(far[0].y, near[0].y);
        assert_eq!(far[0].x, near[0].x + 1);
    }

    // ---- fluid_multi_input_row ----

    #[test]
    fn fluid_multi_input_heavy_oil_cracking_geometry() {
        // heavy-oil-cracking: water (dx=0) + heavy-oil (dx=2), fluid output
        // light-oil. Single chemical-plant (3×3). x_offset=0, y_offset=0.
        let (entities, row_height, in_ports, out_ports) = fluid_multi_input_row(
            "heavy-oil-cracking",
            "chemical-plant",
            3, // msz
            1, // machine_count
            0, // y_offset
            0, // x_offset
            &[(0, "water"), (2, "heavy-oil")],
            None,
            &[(1, "light-oil")],
            None,
            false, // lane_split
            true,
            0.5,
            InserterTier::default(),
        );

        // Row layout for N=2, msz=3, fluid output, with always-gap=1:
        //   y=0: fluid 1 (heavy-oil) trunk at cols 1, 2, 3
        //   y=1: gap row (empty in our column; reserved for outer fluid bridging)
        //   y=2: fluid 0 (water) trunk at cols -1, 0, 1 + fluid 1 drop at col 2
        //   y=3: fluid 0 drop UG at col 0
        //   y=4: UG-out at cols 0 and 2
        //   y=5..7: machine
        //   y=8: output pipe row
        assert_eq!(row_height, 9, "expected row height 9 for pure-fluid 2-input 3×3 with gap=1");

        // Fluid 1 (heavy-oil, outermost) trunk at y=0, T-drop at col 2
        let left_flank_b = assert_entity(&entities, 1, 0, "pipe-to-ground");
        assert_eq!(left_flank_b.direction, EntityDirection::West);
        assert_eq!(left_flank_b.carries.as_deref(), Some("heavy-oil"));
        let t_drop_b = assert_entity(&entities, 2, 0, "pipe");
        assert_eq!(t_drop_b.carries.as_deref(), Some("heavy-oil"));
        let right_flank_b = assert_entity(&entities, 3, 0, "pipe-to-ground");
        assert_eq!(right_flank_b.direction, EntityDirection::East);

        // Fluid 0 (water, innermost) trunk at y=2 (was y=1 pre-gap), T-drop at col 0
        let left_flank_a = assert_entity(&entities, -1, 2, "pipe-to-ground");
        assert_eq!(left_flank_a.direction, EntityDirection::West);
        assert_eq!(left_flank_a.carries.as_deref(), Some("water"));
        let t_drop_a = assert_entity(&entities, 0, 2, "pipe");
        assert_eq!(t_drop_a.carries.as_deref(), Some("water"));
        let right_flank_a = assert_entity(&entities, 1, 2, "pipe-to-ground");
        assert_eq!(right_flank_a.direction, EntityDirection::East);

        // Fluid 1 drop UG at (col_B=2, y=1) direction=SOUTH (in gap row).
        let drop_b = assert_entity(&entities, 2, 1, "pipe-to-ground");
        assert_eq!(drop_b.direction, EntityDirection::South);
        assert_eq!(drop_b.carries.as_deref(), Some("heavy-oil"));

        // Fluid 0 drop UG at (col_A=0, y=3) direction=SOUTH
        let drop_a = assert_entity(&entities, 0, 3, "pipe-to-ground");
        assert_eq!(drop_a.direction, EntityDirection::South);
        assert_eq!(drop_a.carries.as_deref(), Some("water"));

        // UG-outs at y=4, facing NORTH
        let ug_out_a = assert_entity(&entities, 0, 4, "pipe-to-ground");
        assert_eq!(ug_out_a.direction, EntityDirection::North);
        assert_eq!(ug_out_a.carries.as_deref(), Some("water"));
        let ug_out_b = assert_entity(&entities, 2, 4, "pipe-to-ground");
        assert_eq!(ug_out_b.direction, EntityDirection::North);
        assert_eq!(ug_out_b.carries.as_deref(), Some("heavy-oil"));

        // Machine at y=5
        let m = assert_entity(&entities, 0, 5, "chemical-plant");
        assert_eq!(m.recipe.as_deref(), Some("heavy-oil-cracking"));

        // Output pipe row at y=8
        for dx in 0..3 {
            let p = assert_entity(&entities, dx, 8, "pipe");
            assert_eq!(p.carries.as_deref(), Some("light-oil"));
        }

        // Port tap points reported correctly
        assert!(in_ports.contains(&("water".to_string(), 0, 2)));
        assert!(in_ports.contains(&("heavy-oil".to_string(), 2, 0)));
        assert_eq!(out_ports, vec![("light-oil".to_string(), 1, 8)]);
    }

    #[test]
    fn fluid_multi_input_sulfur_solid_output() {
        // sulfur: water (dx=0) + petroleum-gas (dx=2), solid output (sulfur).
        let (entities, row_height, _, out_ports) = fluid_multi_input_row(
            "sulfur",
            "chemical-plant",
            3, 1, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            Some("sulfur"),
            &[],
            Some("transport-belt"),
            false, // lane_split
            false, // westward output
            0.5,
            InserterTier::default(),
        );

        // Row height = 10 with always-gap=1 (one extra for inserter + belt
        // vs pipe row, on top of the +1 gap row).
        assert_eq!(row_height, 10);

        // Output inserter at (mx+1=1, y=8), facing South
        let ins = assert_entity(&entities, 1, 8, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("sulfur"));

        // Output belt at y=9, direction West (output_east=false)
        for dx in 0..3 {
            let b = assert_entity(&entities, dx, 9, "transport-belt");
            assert_eq!(b.direction, EntityDirection::West);
            assert_eq!(b.carries.as_deref(), Some("sulfur"));
        }

        // Fluid output should be empty
        assert!(out_ports.is_empty());
    }

    #[test]
    fn fluid_multi_input_sulfur_output_uses_extra_column() {
        // output_rate=15.0 exceeds one stack inserter's 12.0/s ceiling;
        // this row never trims its output belt by position (no `is_last`
        // logic exists anywhere in the template), so both extra columns
        // (dx=0, dx=2) are available uncontested at every position.
        let (entities, _, _, _) = fluid_multi_input_row(
            "sulfur",
            "chemical-plant",
            3, 1, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            Some("sulfur"),
            &[],
            Some("transport-belt"),
            false, // lane_split
            false,
            15.0, // output_rate
            InserterTier::default(),
        );
        let stacks: Vec<_> = entities
            .iter()
            .filter(|e| e.y == 8 && e.name == "stack-inserter" && e.carries.as_deref() == Some("sulfur"))
            .collect();
        assert_eq!(stacks.len(), 2, "should use one extra column: {stacks:?}");
        let xs: Vec<i32> = stacks.iter().map(|e| e.x).collect();
        assert!(xs.contains(&1) && (xs.contains(&0) || xs.contains(&2)), "baseline dx=1 + one extra: {xs:?}");
    }

    #[test]
    fn fluid_multi_input_isolation_perpendicular_ugs() {
        // Proves the F5a isolation invariant: the tile where fluid 0's right
        // flank UG (east-west) meets fluid 1's drop UG (north-south) must
        // exist, and both must have perpendicular facing directions so their
        // shared edge has no surface on either side.
        let (entities, _, _, _) = fluid_multi_input_row(
            "heavy-oil-cracking",
            "chemical-plant",
            3, 1, 0, 0,
            &[(0, "water"), (2, "heavy-oil")],
            None,
            &[(1, "light-oil")],
            None,
            false, // lane_split
            true,
            0.5,
            InserterTier::default(),
        );

        // With always-gap=1, water (fi=0, inner) trunk_y=2; its right flank
        // is at (col_t+1, 2) = (1, 2). Heavy-oil (fi=1, outer) trunk_y=0;
        // its drop UG sits in the gap row at (col_t, trunk_y+1) = (2, 1).
        // The flank and drop UG are no longer at the same y, so the
        // perpendicular-edge isolation case from gap=0 doesn't arise.
        let right_flank = entities.iter().find(|e| e.x == 1 && e.y == 2 && e.name == "pipe-to-ground").expect("right flank UG exists");
        assert!(matches!(right_flank.direction, EntityDirection::East | EntityDirection::West));
        assert_eq!(right_flank.carries.as_deref(), Some("water"));

        // Heavy-oil drop UG at (2, 1) — north-south, sits in gap row.
        let drop_b = entities.iter().find(|e| e.x == 2 && e.y == 1 && e.name == "pipe-to-ground").expect("fluid 1 drop UG exists");
        assert!(matches!(drop_b.direction, EntityDirection::North | EntityDirection::South));
        assert_eq!(drop_b.carries.as_deref(), Some("heavy-oil"));

        // Different fluids carried.
        assert_ne!(drop_b.carries, right_flank.carries);
    }

    #[test]
    fn fluid_multi_input_adjacent_ports_inserts_gap() {
        // Sulfuric-acid-on-assembling-machine-3 shape: two fluid inputs
        // whose port_dx values differ by exactly 1 (petgas=1, water=2).
        // Without the gap, fluid 1's drop UG lands on fluid 0's left-flank
        // UG at the same tile — an object collision.
        //
        // The gap=1 mechanism now applies always for n≥2 fluids (also
        // serving the bus-router lane-adjacency case), so adjacent-port
        // and non-adjacent-port layouts share the same height. This test
        // verifies the previously-colliding adjacent-port shape no longer
        // overlaps and matches the always-on baseline.
        let (entities, row_height_adj, _, _) = fluid_multi_input_row(
            "sulfuric-acid",
            "assembling-machine-3",
            3, 1, 0, 0,
            &[(2, "water"), (1, "petroleum-gas")],
            None,
            &[(1, "sulfuric-acid")],
            None,
            false, // lane_split
            true,
            0.5,
            InserterTier::default(),
        );

        // Baseline layout (ports 2 apart) — same row_height under always-gap=1.
        let (_, row_height_base, _, _) = fluid_multi_input_row(
            "sulfuric-acid",
            "assembling-machine-3",
            3, 1, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            None,
            &[(1, "sulfuric-acid")],
            None,
            false, // lane_split
            true,
            0.5,
            InserterTier::default(),
        );
        assert_eq!(
            row_height_adj, row_height_base,
            "adjacent-port and non-adjacent-port layouts should match under always-gap=1"
        );

        // No two entities share a tile.
        use std::collections::HashMap;
        let mut by_tile: HashMap<(i32, i32), Vec<&PlacedEntity>> = HashMap::new();
        for e in &entities {
            by_tile.entry((e.x, e.y)).or_default().push(e);
        }
        // Machines occupy a 3×3 footprint but are registered once at their
        // anchor — ignore them for the "no duplicates" check.
        for (&(x, y), es) in &by_tile {
            let non_machine: Vec<_> = es.iter().filter(|e| e.name != "assembling-machine-3").collect();
            assert!(
                non_machine.len() <= 1,
                "tile ({x},{y}) has {} overlapping entities: {:?}",
                non_machine.len(),
                non_machine.iter().map(|e| &e.name).collect::<Vec<_>>(),
            );
        }

        // Petroleum-gas is fi=1 (outer) because fluid_inputs[0]=water,
        // fluid_inputs[1]=petgas. Outer stays at y_offset+0.
        let pg_t = assert_entity(&entities, 1, 0, "pipe");
        assert_eq!(pg_t.carries.as_deref(), Some("petroleum-gas"));

        // Petgas drop UG sits in the gap row at y=1, col_t=1.
        let pg_drop = assert_entity(&entities, 1, 1, "pipe-to-ground");
        assert_eq!(pg_drop.direction, EntityDirection::South);
        assert_eq!(pg_drop.carries.as_deref(), Some("petroleum-gas"));

        // Water is fi=0 (inner), shifted down by 1 → trunk at y=2.
        let water_t = assert_entity(&entities, 2, 2, "pipe");
        assert_eq!(water_t.carries.as_deref(), Some("water"));

        // Water's left-flank UG is at (1, 2) — no longer colliding with
        // petgas's drop UG (which is now at (1, 1)).
        let water_lf = assert_entity(&entities, 1, 2, "pipe-to-ground");
        assert_eq!(water_lf.direction, EntityDirection::West);
        assert_eq!(water_lf.carries.as_deref(), Some("water"));
    }

    #[test]
    fn fluid_multi_input_multi_machine_stamps_repeat() {
        let (entities, _, in_ports, _) = fluid_multi_input_row(
            "heavy-oil-cracking",
            "chemical-plant",
            3, 2, // 2 machines
            0, 0,
            &[(0, "water"), (2, "heavy-oil")],
            None,
            &[(1, "light-oil")],
            None,
            false, // lane_split
            true,
            0.5,
            InserterTier::default(),
        );

        // Second machine at mx=3. Its T-drop pipes should be at (3, 2) water (was y=1 pre-gap)
        // and (5, 0) heavy-oil (outermost trunk, unchanged).
        let water_b = assert_entity(&entities, 3, 2, "pipe");
        assert_eq!(water_b.carries.as_deref(), Some("water"));
        let ho_b = assert_entity(&entities, 5, 0, "pipe");
        assert_eq!(ho_b.carries.as_deref(), Some("heavy-oil"));

        // Second machine present at y=5 (was y=4 pre-gap)
        let m2 = assert_entity(&entities, 3, 5, "chemical-plant");
        assert_eq!(m2.recipe.as_deref(), Some("heavy-oil-cracking"));

        // 4 port tap points (2 fluids × 2 machines)
        assert_eq!(in_ports.len(), 4);
    }

    #[test]
    fn fluid_multi_input_lane_split_solid_output() {
        // Sulfur: 2 fluids in (water dx=0, petroleum-gas dx=2), solid output.
        // Strategy A (tight pack, inline shift): 2 machines at x=0, 3.
        // Anchor=0 → bridge cols [1, 4) on output rows (y=8, y=9).
        let (entities, _, _, _) = fluid_multi_input_row(
            "sulfur",
            "chemical-plant",
            3, 2, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            Some("sulfur"),
            &[],
            Some("transport-belt"),
            true,  // lane_split
            false, // west-flow output
            0.5,
            InserterTier::default(),
        );

        // Machines packed tight at x=0, 3. Machine y=5 (always-gap=1
        // pushes machines down by 1 vs pre-gap layout).
        assert_entity(&entities, 0, 5, "chemical-plant");
        assert_entity(&entities, 3, 5, "chemical-plant");

        // Water (innermost) trunk row at y=2. Machine 0's east UG-in
        // for water (port_dx=0) at (1, 2); machine 1's west UG-out at
        // (2, 2). Tight pack puts these adjacent — 0 hidden tiles, well
        // within fluid UG max.
        let water_east = assert_entity(&entities, 1, 2, "pipe-to-ground");
        assert_eq!(water_east.direction, EntityDirection::East);
        assert_eq!(water_east.carries.as_deref(), Some("water"));
        let water_west = assert_entity(&entities, 2, 2, "pipe-to-ground");
        assert_eq!(water_west.direction, EntityDirection::West);
        assert_eq!(water_west.carries.as_deref(), Some("water"));

        // Inline bridge at cols 1, 2, 3 (mx_anchor=0, bridge cols
        // [mx+1, mx+4)). Bridge_y=8, output_y=9.
        for &x in &[1, 2, 3] {
            assert_entity(&entities, x, 8, "transport-belt");
            assert_entity(&entities, x, 9, "transport-belt");
        }
    }

    // ---- scrap_recycling_row (RFP Fulgora Phase 3 sushi sorter) ----

    #[test]
    fn scrap_recycling_row_places_sorter_mechanism() {
        // 4 recyclers, 3 sorted items — enough to exercise the fan-out.
        let items = vec![
            ("iron-gear-wheel".to_string(), 2.0),
            ("stone".to_string(), 0.4),
            ("holmium-ore".to_string(), 0.1),
        ];
        let (entities, height, sorted_belts) = scrap_recycling_row(
            "scrap-recycling",
            "scrap",
            4,
            0,
            0,
            10.0,
            &items,
            Some("transport-belt"),
            InserterTier::default(),
        );

        // Four south-facing recyclers at pitch 2, dy=2.
        let recyclers: Vec<_> = entities.iter().filter(|e| e.name == "recycler").collect();
        assert_eq!(recyclers.len(), 4, "expected 4 recyclers");
        for r in &recyclers {
            assert_eq!(r.direction, EntityDirection::South);
            assert_eq!(r.recipe.as_deref(), Some("scrap-recycling"));
            assert_eq!(r.y, 2);
        }

        // Sushi belt tagged `:sushi:`, carries no single item.
        let sushi: Vec<_> = entities
            .iter()
            .filter(|e| e.segment_id.as_deref().is_some_and(|s| s.contains(":sushi:")))
            .collect();
        assert!(!sushi.is_empty(), "expected a :sushi: tagged belt run");
        assert!(sushi.iter().all(|e| e.carries.is_none()), "sushi tiles carry no single item");
        assert!(sushi.iter().all(|e| e.y == 6), "sushi belt at dy=6 (one tile past the eject edge)");

        // One filter inserter per item, each with the matching filter.
        for (item, _) in &items {
            let ins: Vec<_> = entities
                .iter()
                .filter(|e| {
                    e.name == "fast-inserter"
                        && e.segment_id.as_deref().is_some_and(|s| s.contains(":sushi-sort:"))
                        && e.filters == vec![item.clone()]
                })
                .collect();
            assert_eq!(ins.len(), 1, "expected exactly one sort inserter filtering {item}");
        }

        // Each item gets its own output belt at a distinct y (the fan-out).
        assert_eq!(sorted_belts.len(), 3);
        let ys: std::collections::BTreeSet<i32> = sorted_belts.iter().map(|(_, y)| *y).collect();
        assert_eq!(ys.len(), 3, "each sorted item exits on its own y");
        // Height covers scrap-in (0) .. last fan-out row.
        assert_eq!(height, 8 + 3);

        // No two entities share a tile (crossing-free fan-out).
        let mut seen: std::collections::HashSet<(i32, i32, String)> = Default::default();
        let mut occ: std::collections::HashMap<(i32, i32), usize> = Default::default();
        for e in &entities {
            *occ.entry((e.x, e.y)).or_default() += 1;
            seen.insert((e.x, e.y, e.name.clone()));
        }
        // Belts/inserters are 1×1; recyclers are 2×4 (handled separately).
        // Assert no two BELTS overlap (the fan-out staircase must not cross).
        let mut belt_occ: std::collections::HashMap<(i32, i32), usize> = Default::default();
        for e in &entities {
            if e.name.contains("belt") {
                *belt_occ.entry((e.x, e.y)).or_default() += 1;
            }
        }
        assert!(belt_occ.values().all(|&n| n == 1), "fan-out belts must not overlap");
    }
}
