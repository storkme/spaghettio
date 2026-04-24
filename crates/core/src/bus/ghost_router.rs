//! Ghost A* bus router — Phases 2+3 of the ghost-cluster routing rewrite.
//!
//! Algorithm overview:
//! 1. Build a hard-obstacle set from row_entities (machine footprints, poles, etc.)
//!    and fluid lane tile reservations.
//! 2. Stamp splitters and balancer entities.
//! 3. Pre-stamp trunk belts (South-facing) directly as Permanent entities.
//!    Synthetic column paths are recorded in `trunk_synth_paths` and injected
//!    into `routed_paths` after A* so the junction solver can classify
//!    trunk/tap crossings correctly.
//! 4. Route each connecting-belt spec (tap-offs, returns, feeders) via
//!    `ghost_astar`. Trunk tiles are passable so A* ghosts through them and
//!    records crossing tiles for the junction resolver.
//! 5. Negotiate lane conflicts iteratively; adopt best routing.
//! 6. Resolve crossings: perpendicular template first, SAT fallback.
//! 7. Merge output rows via the existing `merge_output_rows` helper.
//!
//! Returns a `GhostRouteResult` containing all placed entities, ghost crossing
//! tiles, cluster info, and layout dimensions.
//!
//! See `docs/archive/rfp-ghost-cluster-routing.md` for the full design.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::astar::ghost_astar;
use crate::bus::balancer::{balancer_origin_x, splitter_for_belt, stamp_family_balancer};
use crate::bus::lane_planner::{BusLane, LaneFamily, MACHINE_ENTITIES};
use crate::bus::output_merger::merge_output_rows;
use crate::bus::trunk_renderer::{is_intermediate, render_path, trunk_segments};
use crate::bus::junction::{BeltTier, Rect};
use crate::bus::junction_sat_strategy::SatStrategy;
use crate::bus::junction_solver::{
    self, JunctionSolution, JunctionStrategy, JunctionStrategyContext,
};
use crate::bus::placer::RowSpan;
use crate::common::{belt_entity_for_rate, machine_size, machine_tiles, ug_max_reach};
use crate::models::{EntityDirection, LayoutRegion, PlacedEntity, SolverResult};
// sat.rs is retained in the tree as a standalone library; route_bus_ghost
// no longer uses it after the per-tile "unresolved" rewrite. The junction
// solver (docs/archive/rfp-junction-solver.md) may reintroduce it as a T4
// fallback strategy, in which case reimport at that point.
use crate::trace;

const TURN_PENALTY: u32 = 8;

/// Output of the ghost router.
pub struct GhostRouteResult {
    pub entities: Vec<PlacedEntity>,
    /// All tiles where two or more routed paths overlap.
    pub ghost_crossing_tiles: FxHashSet<(i32, i32)>,
    /// Number of union-find clusters among the ghost crossings.
    pub cluster_count: usize,
    /// Tile count of the largest cluster.
    pub max_cluster_tiles: usize,
    /// Specs that could not be routed (no path found).
    pub unroutable_specs: Vec<String>,
    /// Total layout height (y extent).
    pub max_y: i32,
    /// Maximum x used by output mergers.
    pub merge_max_x: i32,
    /// Layout regions (empty for Phase 2; SAT fills these in Phase 3).
    pub regions: Vec<LayoutRegion>,
    /// Non-fatal warnings (direct/bare modes produce these for cases
    /// that would hard-error in the default pipeline).
    pub warnings: Vec<String>,
}

/// A spec for one connecting belt run.
struct BeltSpec {
    key: String,
    start: (i32, i32),
    goal: (i32, i32),
    item: String,
    belt_name: &'static str,
    /// Explicit exit direction for the final belt on this path. Set when the
    /// planner knows the spec's topology — producer-row orientation, trunk
    /// axis — at emission time. When `Some`, render_path uses this for
    /// single-tile paths and the last tile of multi-tile paths instead of
    /// inferring direction from start/goal coordinate comparisons (which is
    /// ambiguous for degenerate start == goal specs and length-1 blocked A*
    /// fallbacks). See plan file abundant-gliding-turing.md for context.
    exit_dir: Option<EntityDirection>,
}

/// Route all bus belts using the ghost A* approach.
#[allow(clippy::too_many_arguments)]
pub fn route_bus_ghost(
    lanes: &[BusLane],
    row_spans: &[RowSpan],
    total_height: i32,
    bw: i32,
    max_belt_tier: Option<&str>,
    solver_result: &SolverResult,
    families: &[LaneFamily],
    row_entities: &[PlacedEntity],
) -> Result<GhostRouteResult, String> {
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut max_y = total_height;
    let mut merge_max_x = 0i32;

    let width = (bw + 200).max(200);
    let height = (total_height + 50).max(200);

    // -------------------------------------------------------------------------
    // Step 1: Build hard obstacle set from row_entities
    // -------------------------------------------------------------------------
    let mut hard: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut existing_belts: FxHashSet<(i32, i32)> = FxHashSet::default();
    // Tracks belt positions that existed before ghost routing (row templates,
    // trunks, splitters, balancers).  Crossings against these are not
    // ghost-vs-ghost conflicts and are filtered out of the crossing set.
    let mut pre_ghost_belts: FxHashSet<(i32, i32)> = FxHashSet::default();

    for e in row_entities {
        if is_belt_like(&e.name) {
            existing_belts.insert((e.x, e.y));
            pre_ghost_belts.insert((e.x, e.y));
        } else if MACHINE_ENTITIES.contains(&e.name.as_str()) {
            let sz = machine_size(&e.name);
            for t in machine_tiles(e.x, e.y, sz) {
                hard.insert(t);
            }
        } else {
            hard.insert((e.x, e.y));
        }
    }

    // Reserve fluid lane tiles as hard obstacles (same logic as pole placer
    // in layout.rs: fluid lanes reserve the column from source_y to last tap_y).
    // Tracked in `fluid_reservations` too so the step-3.6 trunk emitter can
    // distinguish its own reservations from real obstacles (e.g. neighbouring
    // solid lanes' splitter secondary tiles) when deciding where to UG-bridge.
    let mut fluid_reservations: FxHashSet<(i32, i32)> = FxHashSet::default();
    for lane in lanes {
        if lane.is_fluid {
            let end_y = lane.tap_off_ys.iter().copied().max().unwrap_or(lane.source_y);
            for y in lane.source_y..=end_y {
                hard.insert((lane.x, y));
                fluid_reservations.insert((lane.x, y));
            }
        }
    }

    // -------------------------------------------------------------------------
    // Step 2: Place splitter stamps as hard obstacles.
    // -------------------------------------------------------------------------
    for lane in lanes {
        if lane.is_fluid {
            continue;
        }

        let x = lane.x;
        let belt_name = belt_entity_for_rate(lane.rate * 2.0, max_belt_tier);
        let trunk_seg_id = Some(format!("trunk:{}", lane.item));
        let last_tap_y = lane.tap_off_ys.iter().copied().max();

        // Place splitter stamps for non-last tap-offs
        if lane.tap_off_ys.len() > 1 {
            let splitter_name = splitter_for_belt(belt_name);
            let tapoff_seg_id = Some(format!("tapoff:{}", lane.item));
            for &tap_y in &lane.tap_off_ys {
                if Some(tap_y) == last_tap_y {
                    continue;
                }
                // Splitter at (x, tap_y-1), East belt at (x+1, tap_y-1)
                // Trunk-continue belt at (x, tap_y)
                entities.push(PlacedEntity {
                    name: splitter_name.to_string(),
                    x,
                    y: tap_y - 1,
                    direction: EntityDirection::South,
                    carries: Some(lane.item.clone()),
                    segment_id: tapoff_seg_id.clone(),
                    rate: Some(lane.rate),
                    ..Default::default()
                });
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x,
                    y: tap_y,
                    direction: EntityDirection::South,
                    carries: Some(lane.item.clone()),
                    segment_id: trunk_seg_id.clone(),
                    rate: Some(lane.rate),
                    ..Default::default()
                });
                // The splitter occupies 2 tiles (x, tap_y-1) and (x+1, tap_y-1)
                hard.insert((x, tap_y - 1));
                hard.insert((x + 1, tap_y - 1));
                hard.insert((x, tap_y));
                // If the secondary tile lands in a fluid lane's column, the
                // fluid trunk emitter must know to UG-bridge past it instead
                // of stamping a pipe on top of the splitter. Drop the tile
                // from `fluid_reservations` so the step-3.6 check sees it as
                // foreign-claimed.
                fluid_reservations.remove(&(x + 1, tap_y - 1));
                existing_belts.insert((x, tap_y - 1));
                existing_belts.insert((x, tap_y));
                pre_ghost_belts.insert((x, tap_y - 1));
                pre_ghost_belts.insert((x, tap_y));
            }
        }
    }

    // -------------------------------------------------------------------------
    // Step 3: Stamp balancer blocks as hard obstacles
    // -------------------------------------------------------------------------
    for fam in families {
        let balancer_ents = stamp_family_balancer(fam, max_belt_tier)
            .map_err(|e| format!("ghost router: balancer stamp failed for {:?}: {}", fam.shape, e))?;
        crate::trace::emit(crate::trace::TraceEvent::BalancerStamped {
            item: fam.item.clone(),
            shape: fam.shape,
            y_start: fam.balancer_y_start,
            y_end: fam.balancer_y_end,
            template_found: !balancer_ents.is_empty(),
        });
        for ent in &balancer_ents {
            if is_belt_like(&ent.name) {
                hard.insert((ent.x, ent.y));
                existing_belts.insert((ent.x, ent.y));
                pre_ghost_belts.insert((ent.x, ent.y));
            } else {
                hard.insert((ent.x, ent.y));
            }
            fluid_reservations.remove(&(ent.x, ent.y));
            // Splitters occupy two tiles. Without this the second tile
            // would be invisible to the obstacle set — it wouldn't be
            // classified as a splitter body by the junction solver's
            // forbidden pass, so SAT could try to stamp something there
            // and the splitter-topology synthesis wouldn't see the tile
            // as in-bbox+forbidden.
            if crate::common::is_splitter(&ent.name) {
                let (sx, sy) = crate::common::splitter_second_tile(ent);
                hard.insert((sx, sy));
                fluid_reservations.remove(&(sx, sy));
            }
        }
        entities.extend(balancer_ents);
    }

    // -------------------------------------------------------------------------
    // Step 3.5: Stamp trunk belts directly (no A*).
    //
    // Each trunk segment is stamped as a South-facing Permanent entity before
    // Occupancy construction and before A* runs. This replaces the old approach
    // of routing trunks through ghost_astar. Benefits:
    //   - 1-tile trunk stubs (start == goal) are unconstructable as degenerate
    //     A* paths; a single South-facing entity is stamped directly instead.
    //   - Trunk direction is always exactly South — no A* bending.
    //   - Trunk entities land in `permanent_inits` before Occupancy::new, so
    //     the Permanent claim is present during all downstream steps.
    //   - Trunk keys are absent from `routed_paths`, so the junction solver
    //     only sees crossing specs that are genuinely ghost-routed.
    //
    // Trunks stay in `existing_belts` (transparent to A*) so tap-offs and
    // returns can still route in a straight line through trunk columns. They are
    // NOT added to `pre_ghost_belts` — instead their tile→item pairs are
    // collected into `trunk_tile_items` and injected into `ghost_item_at` after
    // the materialisation reset. This preserves the OLD crossing-detection
    // mechanism: the `ghost_item_at` filter drops tap entities that land on a
    // trunk tile, and the `all_ghost_crossings` check still fires for different-
    // item overlaps so the junction solver can bridge them.
    // -------------------------------------------------------------------------
    let mut trunk_tile_items: FxHashMap<(i32, i32), String> = FxHashMap::default();
    // Synthetic column paths for each trunk lane, keyed by "trunk:{item}:{x}".
    // Keyed per-column (not just per-item) because multi-lane items like a
    // split copper-cable trunk have multiple vertical columns — merging them
    // into one path produces bogus horizontal dx between (x,y) and (x+1,y)
    // tiles at the same y, which mis-classifies the trunk axis.
    // Injected into `routed_paths` after routing so classify_crossing and
    // the junction solver can see trunk specs at crossing tiles — the same
    // way they saw them in the old code when trunks were BeltSpecs routed
    // through A*.
    let mut trunk_synth_paths: FxHashMap<String, Vec<(i32, i32)>> = FxHashMap::default();
    for lane in lanes {
        if lane.is_fluid {
            continue;
        }
        let x = lane.x;
        let belt_name = belt_entity_for_rate(lane.rate * 2.0, max_belt_tier);
        let trunk_seg_id = Some(format!("trunk:{}", lane.item));
        let last_tap_y = lane.tap_off_ys.iter().copied().max();

        let mut producer_ys: FxHashSet<i32> = FxHashSet::default();
        if let Some(pr) = lane.producer_row {
            if pr < row_spans.len() {
                producer_ys.insert(row_spans[pr].output_belt_y);
            }
        }
        for &pri in &lane.extra_producer_rows {
            if pri < row_spans.len() {
                producer_ys.insert(row_spans[pri].output_belt_y);
            }
        }

        // skip_ys mirrors the old trunk-BeltSpec logic exactly: skip all
        // tap_off_ys (non-last handled by step 2 splitter/continue-belt
        // stamps; last_tap_y left for the tap spec to stamp an East-facing
        // belt) and all balancer rows (step 3 already stamped those).
        let mut skip_ys: FxHashSet<i32> = lane.tap_off_ys.iter().copied().collect();
        for &ty in &lane.tap_off_ys {
            if lane.tap_off_ys.len() > 1 && Some(ty) != last_tap_y {
                skip_ys.insert(ty - 1);
            }
        }
        if let Some(by) = lane.balancer_y {
            skip_ys.insert(by);
        }
        if let Some((by_start, by_end)) = lane.family_balancer_range {
            for y in by_start..=by_end {
                skip_ys.insert(y);
            }
        }

        let mut all_ys: Vec<i32> = lane.tap_off_ys.clone();
        all_ys.extend(producer_ys.iter().copied());
        let start_y = lane.source_y;
        let end_y = all_ys.iter().copied().max().unwrap_or(start_y);
        let end_y = if let Some(by) = lane.balancer_y {
            end_y.max(by + 1)
        } else {
            end_y
        };

        for (seg_start, seg_end) in trunk_segments(start_y, end_y, &skip_ys) {
            for y in seg_start..=seg_end {
                let tile = (x, y);
                if hard.contains(&tile) || existing_belts.contains(&tile) {
                    continue;
                }
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x,
                    y,
                    direction: EntityDirection::South,
                    carries: Some(lane.item.clone()),
                    segment_id: trunk_seg_id.clone(),
                    ..Default::default()
                });
                // Passable to A* (existing_belts). Not in pre_ghost_belts —
                // the ghost_item_at mechanism handles dropping conflicting
                // tap/ret entities at trunk tiles, and preserves crossing
                // detection so the junction solver can bridge them.
                existing_belts.insert(tile);
                trunk_tile_items.insert(tile, lane.item.clone());
                trunk_synth_paths
                    .entry(format!("trunk:{}:{}", lane.item, x))
                    .or_default()
                    .push(tile);
            }
        }
    }
    // Sort each synth path so tiles are ordered top-to-bottom (ascending y).
    for path in trunk_synth_paths.values_mut() {
        path.sort_by_key(|&(_, y)| y);
    }

    // -------------------------------------------------------------------------
    // Step 3.6: Stamp fluid trunks + horizontal branches to port rows.
    //
    // For each fluid lane, stamp a vertical column of `"pipe"` at lane.x from
    // source_y to the last port y. Where the column is blocked by another
    // lane's splitter/balancer/machine (typical collision: a solid lane's
    // splitter secondary tile landing in the first fluid lane's column), use
    // a pipe-to-ground pair to tunnel under the obstacle run so trunk
    // connectivity holds.
    //
    // Then, for each consumer/producer port row, stamp a horizontal pipe run
    // from (lane.x + 1, port_y) east until it meets the row template's pipe
    // line (at row.x_offset for the leftmost machine). `optimize_lane_order`
    // places fluid lanes east of solids, so horizontal branches don't have
    // to cross foreign solid trunks — straight surface pipes suffice.
    // -------------------------------------------------------------------------
    // Max distance between a UG-in and its partner UG-out (per F4, vanilla).
    const FLUID_UG_MAX_DISTANCE: i32 = 10;
    // Max spacing between surface pipes on a fluid trunk. A UG pair spans
    // at most `FLUID_UG_MAX_DISTANCE` tiles of trunk; between two surface
    // pipes we need room for the pair plus the surface tiles themselves,
    // so maximum surface-to-surface spacing is `max_distance + 1` (e.g.
    // pipe@y=0, UG-in@y=1, UG-out@y=10, pipe@y=11 fits for distance 10).
    const FLUID_TRUNK_SURFACE_SPACING: i32 = FLUID_UG_MAX_DISTANCE + 1;

    for lane in lanes {
        if !lane.is_fluid {
            continue;
        }
        let x = lane.x;
        let trunk_seg_id = Some(format!("trunk:{}", lane.item));

        let mut end_y = lane.source_y;
        for &(_ri, _px, py) in &lane.fluid_port_positions {
            end_y = end_y.max(py);
        }
        for &(_ri, _px, py) in &lane.fluid_output_port_positions {
            end_y = end_y.max(py);
        }
        let start_y = lane.source_y;

        // Fluid trunks surface at every "anchor" row: start_y (top entry),
        // every tap_y, end_y, and breathing points every
        // FLUID_TRUNK_SURFACE_SPACING tiles. Between anchors, UG pairs fill
        // the gap (for gap ≥3); short gaps just use a straight surface-pipe
        // chain. This keeps the trunk surface-sparse in the long stretches
        // between taps while still giving the top-of-layout tile a concrete
        // entity so blueprint users have something to connect their external
        // pipe to.
        //
        // Adjacent fluid lanes stay isolated where UGs are present (F5a:
        // east/west faces closed). Where two adjacent lanes BOTH have surface
        // pipes at the same y (e.g. both tap at y=1), the row templates are
        // expected to stagger tap ys so only one lane surfaces on any given
        // row (multi-fluid template stacks tap rows one per fluid).
        let mut tap_ys: Vec<i32> = Vec::new();
        for &(_ri, _px, py) in &lane.fluid_port_positions {
            tap_ys.push(py);
        }
        for &(_ri, _px, py) in &lane.fluid_output_port_positions {
            tap_ys.push(py);
        }

        let mut anchors: Vec<i32> = vec![start_y];
        anchors.extend(tap_ys.iter().copied());
        anchors.push(end_y);
        anchors.sort_unstable();
        anchors.dedup();

        // Insert breathing anchors so no gap exceeds the max-reach-plus-one
        // spacing (UG pair plus two surface-pipe ends fits `max_distance + 1`
        // tiles of trunk).
        let mut refined: Vec<i32> = Vec::new();
        for &sp in &anchors {
            if let Some(&prev) = refined.last() {
                let mut cur = prev;
                while sp - cur > FLUID_TRUNK_SURFACE_SPACING {
                    cur += FLUID_TRUNK_SURFACE_SPACING;
                    refined.push(cur);
                }
            }
            refined.push(sp);
        }

        let reservations = &fluid_reservations;
        let is_blocked = |y: i32, existing_belts: &FxHashSet<(i32, i32)>, hard: &FxHashSet<(i32, i32)>| -> bool {
            let tile = (x, y);
            existing_belts.contains(&tile)
                || (hard.contains(&tile) && !reservations.contains(&tile))
        };

        // The entry tile (start_y) gets a UG-in when the gap to the next
        // anchor is ≥ 2, so an adjacent fluid lane's surface pipes don't
        // cross-merge with our trunk at the entry row. When the first tap
        // is directly adjacent (gap ≤ 1), we fall back to a surface pipe
        // (UG pair can't fit, per user's short-trunk entry preference).
        let entry_is_ug = refined.len() >= 2 && refined[1] - refined[0] >= 2;

        for (idx, &sp) in refined.iter().enumerate() {
            if is_blocked(sp, &existing_belts, &hard) {
                continue;
            }
            if idx == 0 && entry_is_ug {
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x,
                    y: sp,
                    direction: EntityDirection::South,
                    io_type: Some("input".to_string()),
                    carries: Some(lane.item.clone()),
                    segment_id: trunk_seg_id.clone(),
                    ..Default::default()
                });
            } else {
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x,
                    y: sp,
                    carries: Some(lane.item.clone()),
                    segment_id: trunk_seg_id.clone(),
                    ..Default::default()
                });
            }
            existing_belts.insert((x, sp));
            hard.insert((x, sp));
        }

        // Fill the gaps between consecutive anchors.
        for (idx, pair) in refined.windows(2).enumerate() {
            let (y0, y1) = (pair[0], pair[1]);
            let gap = y1 - y0;
            if gap <= 1 {
                continue;
            }
            let first_pair = idx == 0;
            if gap == 2 {
                if first_pair && entry_is_ug {
                    // UG-in already at y0; partner UG-out at y0+1 completes
                    // the pair (tunnel distance 1, valid).
                    let ug_out_y = y0 + 1;
                    if !is_blocked(ug_out_y, &existing_belts, &hard) {
                        entities.push(PlacedEntity {
                            name: "pipe-to-ground".to_string(),
                            x,
                            y: ug_out_y,
                            direction: EntityDirection::North,
                            io_type: Some("output".to_string()),
                            carries: Some(lane.item.clone()),
                            segment_id: trunk_seg_id.clone(),
                            ..Default::default()
                        });
                        existing_belts.insert((x, ug_out_y));
                        hard.insert((x, ug_out_y));
                    }
                } else {
                    // Surface pipe middle.
                    let mid = y0 + 1;
                    if !is_blocked(mid, &existing_belts, &hard) {
                        entities.push(PlacedEntity {
                            name: "pipe".to_string(),
                            x,
                            y: mid,
                            carries: Some(lane.item.clone()),
                            segment_id: trunk_seg_id.clone(),
                            ..Default::default()
                        });
                        existing_belts.insert((x, mid));
                        hard.insert((x, mid));
                    }
                }
                continue;
            }
            // gap ≥ 3. UG-in/UG-out pair inside the gap. If the first anchor
            // is a UG-in entry (y0 == start_y), its partner UG-out lives at
            // `first_surface - 1` so fluid exits adjacent to the next tap.
            let ug_in_y = if first_pair && entry_is_ug { y0 } else { y0 + 1 };
            let ug_out_y = y1 - 1;
            if first_pair && entry_is_ug {
                // UG-in already emitted at y0; just emit UG-out.
                if !is_blocked(ug_out_y, &existing_belts, &hard) {
                    entities.push(PlacedEntity {
                        name: "pipe-to-ground".to_string(),
                        x,
                        y: ug_out_y,
                        direction: EntityDirection::North,
                        io_type: Some("output".to_string()),
                        carries: Some(lane.item.clone()),
                        segment_id: trunk_seg_id.clone(),
                        ..Default::default()
                    });
                    existing_belts.insert((x, ug_out_y));
                    hard.insert((x, ug_out_y));
                }
            } else if !is_blocked(ug_in_y, &existing_belts, &hard)
                && !is_blocked(ug_out_y, &existing_belts, &hard)
            {
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x,
                    y: ug_in_y,
                    direction: EntityDirection::South,
                    io_type: Some("input".to_string()),
                    carries: Some(lane.item.clone()),
                    segment_id: trunk_seg_id.clone(),
                    ..Default::default()
                });
                existing_belts.insert((x, ug_in_y));
                hard.insert((x, ug_in_y));
                entities.push(PlacedEntity {
                    name: "pipe-to-ground".to_string(),
                    x,
                    y: ug_out_y,
                    direction: EntityDirection::North,
                    io_type: Some("output".to_string()),
                    carries: Some(lane.item.clone()),
                    segment_id: trunk_seg_id.clone(),
                    ..Default::default()
                });
                existing_belts.insert((x, ug_out_y));
                hard.insert((x, ug_out_y));
            }
        }

        // Horizontal branches: one per (row, port_y) pair. Multiple machines
        // in the same row share the same port_y; we stamp to the leftmost px
        // and let the row template's own pipe line propagate east.
        //
        // If the template placed a UG direction=West carrying this fluid at
        // (min_px - 1, py), that's the multi-fluid stacked-T left flank ready
        // to be a UG partner. Emit a single east-facing UG at (x+1, py) and
        // leave the intermediate tiles empty — the tunnel carries fluid across
        // without surface pipes that could cross-merge with foreign trunks on
        // adjacent rows. Otherwise fall back to continuous surface pipes.
        let mut branch_targets: Vec<(i32, i32)> = Vec::new();
        for &(_ri, px, py) in &lane.fluid_port_positions {
            branch_targets.push((px, py));
        }
        for &(_ri, px, py) in &lane.fluid_output_port_positions {
            branch_targets.push((px, py));
        }
        let mut by_py: FxHashMap<i32, i32> = FxHashMap::default();
        for (px, py) in branch_targets {
            by_py.entry(py).and_modify(|min_px| { if px < *min_px { *min_px = px; } }).or_insert(px);
        }
        for (py, min_px) in by_py {
            // Check if the template placed a UG partner at (min_px - 1, py).
            let partner_tile = (min_px - 1, py);
            let has_ug_partner = row_entities.iter().any(|e| {
                e.x == partner_tile.0
                    && e.y == partner_tile.1
                    && e.name == "pipe-to-ground"
                    && e.direction == EntityDirection::West
                    && e.carries.as_deref() == Some(lane.item.as_str())
            });

            if has_ug_partner && min_px - (x + 1) <= 10 {
                // Multi-fluid template: emit one UG at (x+1, py) direction=East,
                // partnered with the template's left flank UG. Reach cap 10 per F4.
                let ug_tile = (x + 1, py);
                if !(hard.contains(&ug_tile) || existing_belts.contains(&ug_tile)) {
                    entities.push(PlacedEntity {
                        name: "pipe-to-ground".to_string(),
                        x: x + 1,
                        y: py,
                        direction: EntityDirection::East,
                        io_type: Some("input".to_string()),
                        carries: Some(lane.item.clone()),
                        segment_id: trunk_seg_id.clone(),
                        ..Default::default()
                    });
                    existing_belts.insert(ug_tile);
                    hard.insert(ug_tile);
                }
            } else {
                // Single-fluid path: continuous surface pipes from (x+1, py)
                // to (min_px - 1, py). Connects to the template's continuous
                // pipe row starting at min_px.
                for bx in (x + 1)..min_px {
                    let tile = (bx, py);
                    if hard.contains(&tile) || existing_belts.contains(&tile) {
                        continue;
                    }
                    entities.push(PlacedEntity {
                        name: "pipe".to_string(),
                        x: bx,
                        y: py,
                        carries: Some(lane.item.clone()),
                        segment_id: trunk_seg_id.clone(),
                        ..Default::default()
                    });
                    existing_belts.insert(tile);
                    hard.insert(tile);
                }
            }
        }
    }

    // Fluid-trunk tile bookkeeping: step 3.6 pushes `pipe` / `pipe-to-ground`
    // entities into `entities` and marks their tiles in `existing_belts` and
    // `hard`, but never registers them in `trunk_tile_items`. Downstream the
    // survivor filter (`!ghost_item_at.contains_key`) and the crossings
    // filter both consult `ghost_item_at`, which is seeded from
    // `trunk_tile_items` at the start of the routing loop — so without this
    // catch-up pass, A* routes belts straight across fluid-trunk pipe
    // columns, the survivor filter fails to drop them, and `Occupancy::place`
    // panics on the `Permanent` claim at the pipe tile. Register every
    // pipe / PTG tile whose `segment_id` is `trunk:*` so both the filter and
    // `all_ghost_crossings` see it as a foreign-item crossing and hand it to
    // the junction solver.
    for ent in &entities {
        let seg = ent.segment_id.as_deref().unwrap_or("");
        if !seg.starts_with("trunk:") {
            continue;
        }
        if ent.name != "pipe" && ent.name != "pipe-to-ground" {
            continue;
        }
        if let Some(item) = &ent.carries {
            trunk_tile_items.insert((ent.x, ent.y), item.clone());
        }
    }

    // -------------------------------------------------------------------------
    // Occupancy refactor (Steps 2-3.5): construct the parallel `Occupancy` from
    // the inputs to steps 1-3 of this function. Step 3 of the rollout uses it
    // to mirror materialisation writes; Step 4+ will switch the template and
    // SAT phases over to it as the source of obstacle truth. See
    // `docs/archive/rfp-ghost-occupancy-refactor.md`.
    // -------------------------------------------------------------------------
    // Row template entities split by permeability: belts are
    // `RowEntity` (boundary ports may land on them); machines,
    // inserters, poles, and pipes are `Permanent` (real obstacles).
    let (row_belts, row_non_belts): (Vec<PlacedEntity>, Vec<PlacedEntity>) = row_entities
        .iter()
        .cloned()
        .partition(|e| is_belt_like(&e.name));
    let mut permanent_inits = row_non_belts;
    permanent_inits.extend(entities.iter().cloned());
    // Drop fluid-reservation tiles that ended up as empty tunnel-through
    // space before handing `hard` to `Occupancy`. Those tiles must remain
    // placeable for belts that route over the PTG tunnel (per F7). The
    // non-empty reservation tiles are already represented by the UG-in /
    // UG-out / pipe entities in `permanent_inits`.
    let occupied_for_occ: FxHashSet<(i32, i32)> = row_entities
        .iter()
        .chain(entities.iter())
        .flat_map(|e| {
            if MACHINE_ENTITIES.contains(&e.name.as_str()) {
                machine_tiles(e.x, e.y, machine_size(&e.name))
            } else {
                vec![(e.x, e.y)]
            }
        })
        .collect();
    let occupancy_hard: FxHashSet<(i32, i32)> = hard
        .iter()
        .filter(|t| !fluid_reservations.contains(t) || occupied_for_occ.contains(t))
        .copied()
        .collect();
    let mut occupancy = crate::bus::ghost_occupancy::Occupancy::new(
        occupancy_hard,
        row_belts,
        permanent_inits,
    );

    #[cfg(debug_assertions)]
    {
        for &tile in &hard {
            // Reservation-only tiles (empty tunnel-through space on a fluid
            // trunk) were intentionally excluded from `occupancy_hard` so
            // belts can route over the PTG tunnel.
            if fluid_reservations.contains(&tile) && !occupied_for_occ.contains(&tile) {
                continue;
            }
            debug_assert!(
                occupancy.is_claimed(tile),
                "occupancy refactor: hard tile {:?} not claimed in parallel Occupancy",
                tile,
            );
        }
        for &tile in &pre_ghost_belts {
            // Row template belts now sit in `Occupancy` as
            // `RowEntity` claims (not `Permanent`), to mirror
            // today's `pre_existing_positions` semantics. They
            // should still be `is_claimed`.
            debug_assert!(
                occupancy.is_claimed(tile),
                "occupancy refactor: pre_ghost_belts tile {:?} not claimed in parallel Occupancy",
                tile,
            );
        }
    }

    // -------------------------------------------------------------------------
    // Step 4: Build connecting-belt spec list
    // -------------------------------------------------------------------------
    let mut specs: Vec<BeltSpec> = Vec::new();

    // Helper: compute the row-exit origin tile for a ret/feeder spec based on
    // the producer row's orientation. For westward rows (intermediate
    // producers feeding a trunk column to their left) items exit at
    // (output_belt_x_min - 1, out_y) flowing West. For eastward rows (final
    // output rows) items exit at (output_belt_x_max + 1, out_y) flowing East.
    // The ret/feeder spec always walks back horizontally toward the trunk,
    // so the *direction* of the bend at the origin tile is always "toward the
    // trunk column": West for westward rows (trunk is west of the exit), West
    // for eastward rows too (trunk is west of the row start at x = bus_width,
    // and the ret walks back west to it).
    let row_exit_origin = |row: &RowSpan| -> (i32, i32) {
        let out_y = row.output_belt_y;
        if row.output_east {
            (row.output_belt_x_max + 1, out_y)
        } else {
            (row.output_belt_x_min - 1, out_y)
        }
    };

    for lane in lanes {
        if lane.is_fluid {
            continue;
        }
        let x = lane.x;
        let has_consumers = !lane.consumer_rows.is_empty();
        let has_producers = lane.producer_row.is_some() || !lane.extra_producer_rows.is_empty();
        let last_tap_y = lane.tap_off_ys.iter().copied().max();
        let horiz_belt = belt_entity_for_rate(lane.rate * 2.0, max_belt_tier);

        // Tap-off specs
        if has_consumers {
            for &tap_y in &lane.tap_off_ys {
                let is_last = Some(tap_y) == last_tap_y;
                // Non-last: start from (x+1, tap_y) (splitter right output)
                // Last: start from (x, tap_y) (trunk terminates here)
                let start_x = if is_last { x } else { x + 1 };
                // Goal: right edge of the bus
                let goal_x = bw - 1;
                let tap_key = format!("tap:{}:{}:{}", lane.item, x, tap_y);
                specs.push(BeltSpec {
                    key: tap_key,
                    start: (start_x, tap_y),
                    goal: (goal_x, tap_y),
                    item: lane.item.clone(),
                    belt_name: horiz_belt,
                    exit_dir: Some(EntityDirection::East),
                });
            }
        }

        // Return specs for intermediate lanes (no family balancer).
        // Producers feed the trunk from the west side of the bus; start tile
        // is orientation-aware (westward rows exit left of the row, eastward
        // rows exit right of the row). Items always walk West back to the
        // trunk, so exit_dir is West — this makes the bend belt that lands
        // at (x+1, out_y) face West regardless of path length, so it sideloads
        // correctly into the South-facing trunk at (x, out_y).
        if is_intermediate(lane) && lane.family_balancer_range.is_none() {
            let mut all_producers = Vec::new();
            if let Some(pr) = lane.producer_row {
                all_producers.push(pr);
            }
            all_producers.extend(&lane.extra_producer_rows);

            for &pri in &all_producers {
                if pri >= row_spans.len() {
                    continue;
                }
                let row = &row_spans[pri];
                let (start_x, out_y) = row_exit_origin(row);
                let goal_x = x + 1;
                // Skip the spec entirely when the exit lands west of (or at)
                // the goal — the row's own exit belt already covers it and
                // no additional ret belt is needed. This can happen when a
                // westward row's output_belt_x_min is adjacent to the trunk.
                if start_x < goal_x {
                    continue;
                }
                // Phase 3 of `docs/rfp-unified-belt-specs.md`: ret specs are
                // already one-spec-per-physical-belt (there is no spec
                // handoff internal to a return path), so "unification" is
                // cosmetic — rename the key to the unified `flow:` prefix
                // so downstream naming is consistent. The `:ret:` infix
                // preserves debuggability (we can still tell at a glance
                // this is a return) and keeps the key unambiguous vs the
                // trunk+last-tap `flow:{item}:{x}` keys (different field
                // count).
                let ret_key = format!("flow:{}:{}:ret:{}", lane.item, x, out_y);
                specs.push(BeltSpec {
                    key: ret_key,
                    start: (start_x, out_y),
                    goal: (goal_x, out_y),
                    item: lane.item.clone(),
                    belt_name: horiz_belt,
                    exit_dir: Some(EntityDirection::West),
                });
            }
        }

        // Collector lanes (producers only, no consumers): ret specs
        if has_producers && !has_consumers && lane.family_balancer_range.is_none() {
            let mut all_producers = Vec::new();
            if let Some(pr) = lane.producer_row {
                all_producers.push(pr);
            }
            all_producers.extend(&lane.extra_producer_rows);

            for &pri in &all_producers {
                if pri >= row_spans.len() {
                    continue;
                }
                let row = &row_spans[pri];
                let (start_x, out_y) = row_exit_origin(row);
                let goal_x = x + 1;
                if start_x < goal_x {
                    continue;
                }
                // Phase 3 of `docs/rfp-unified-belt-specs.md`: ret specs are
                // already one-spec-per-physical-belt (there is no spec
                // handoff internal to a return path), so "unification" is
                // cosmetic — rename the key to the unified `flow:` prefix
                // so downstream naming is consistent. The `:ret:` infix
                // preserves debuggability (we can still tell at a glance
                // this is a return) and keeps the key unambiguous vs the
                // trunk+last-tap `flow:{item}:{x}` keys (different field
                // count).
                let ret_key = format!("flow:{}:{}:ret:{}", lane.item, x, out_y);
                specs.push(BeltSpec {
                    key: ret_key,
                    start: (start_x, out_y),
                    goal: (goal_x, out_y),
                    item: lane.item.clone(),
                    belt_name: horiz_belt,
                    exit_dir: Some(EntityDirection::West),
                });
            }
        }

        // Feeder specs for family balancer lanes
        // Feeder specs for family-balanced lanes — generate once per family
        // (when this lane is the leftmost of its family). Producers are stored
        // on `LaneFamily.producer_rows`, not on the lane itself.
        if let Some(family_id) = lane.family_id {
            if let Some(fam) = families.get(family_id) {
                let is_first_lane_in_family = fam
                    .lane_xs
                    .iter()
                    .copied()
                    .min()
                    .map(|min_x| min_x == lane.x)
                    .unwrap_or(false);
                if is_first_lane_in_family {
                    let templates = crate::bus::balancer_library::balancer_templates();
                    let (n, m) = (fam.shape.0 as u32, fam.shape.1 as u32);
                    if let Some(template) = templates.get(&(n, m)) {
                        // Must match the origin used by stamp_family_balancer
                        // (see balancer::balancer_origin_x). Feeder goals
                        // are template.input_tiles offsets added to this
                        // origin; if it diverges from the stamper's origin
                        // the feeder belts aim at the wrong tiles.
                        let origin_x = if fam.lane_xs.is_empty() {
                            x
                        } else {
                            balancer_origin_x(&fam.lane_xs, template.output_tiles)
                        };
                        let origin_y = fam.balancer_y_start;
                        let mut inputs: Vec<(i32, i32)> = template.input_tiles.to_vec();
                        inputs.sort_by_key(|t| t.0);
                        let feeder_belt = belt_entity_for_rate(fam.total_rate, max_belt_tier);

                        for (i, &pri) in fam.producer_rows.iter().enumerate() {
                            if pri >= row_spans.len() {
                                continue;
                            }
                            let row = &row_spans[pri];
                            let (start_x, out_y) = row_exit_origin(row);
                            if let Some(&(input_x_rel, _input_y_rel)) = inputs.get(i) {
                                let input_x = origin_x + input_x_rel;
                                let input_y = origin_y;
                                let feeder_key =
                                    format!("feeder:{}:{}:{}", lane.item, input_x, out_y);
                                // Feeders walk from the row exit to a
                                // balancer input tile. exit_dir aims at the
                                // balancer — South when the input is below
                                // the row (typical), West otherwise.
                                let feeder_exit_dir = if input_y > out_y {
                                    EntityDirection::South
                                } else if input_y < out_y {
                                    EntityDirection::North
                                } else if input_x < start_x {
                                    EntityDirection::West
                                } else {
                                    EntityDirection::East
                                };
                                specs.push(BeltSpec {
                                    key: feeder_key,
                                    start: (start_x, out_y),
                                    goal: (input_x, input_y),
                                    item: lane.item.clone(),
                                    belt_name: feeder_belt,
                                    exit_dir: Some(feeder_exit_dir),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Step 5: Route each spec with ghost_astar
    // -------------------------------------------------------------------------
    let count_turns = |path: &[(i32, i32)]| -> usize {
        let mut t = 0;
        for w in path.windows(3) {
            let d1 = (w[1].0 - w[0].0, w[1].1 - w[0].1);
            let d2 = (w[2].0 - w[1].0, w[2].1 - w[1].1);
            if d1 != d2 {
                t += 1;
            }
        }
        t
    };

    #[allow(clippy::needless_late_init)]
    let mut routed_paths: FxHashMap<String, Vec<(i32, i32)>>;
    let mut all_ghost_crossings: Vec<(i32, i32)> = Vec::new();
    #[allow(clippy::needless_late_init)]
    let unroutable_specs: Vec<String>;
    // Tracks which item each ghost-routed tile carries, so we can distinguish
    // same-item overlaps (not conflicts) from different-item overlaps (real).
    let mut ghost_item_at: FxHashMap<(i32, i32), String> = FxHashMap::default();

    // All remaining specs (taps, returns, feeders) — no ordering constraint
    // since trunks are now stamped as hard obstacles before A* runs.
    let ordered_specs: Vec<&BeltSpec> = specs.iter().collect();

    // -------------------------------------------------------------------------
    // Step 5: Negotiation loop — route all specs, measure same-axis conflicts,
    // bump per-tile per-axis cost, re-route. Converges when no improvement.
    // -------------------------------------------------------------------------
    // Snapshot pre-routing state so each iteration starts from the same place.
    let pre_routing_existing_belts = existing_belts.clone();

    // Pipe tiles are "soft" obstacles: A* may route belts through them (the
    // tile shows up as a crossing, and the junction solver handles the
    // resulting belt×pipe intersection). `astar_hard` excludes pipe tiles
    // from the obstacle set while `hard` keeps them for internal stamping
    // guards (step 3.6 self-protection, junction solver invariants, etc.).
    //
    // Also excluded: fluid_reservations tiles that were reserved in step 1.5
    // but ended up as empty tunnel-through tiles (between a UG-in and UG-out
    // on the fluid trunk). Belts can cross over PTG tunnels per F7, so these
    // empty reserved tiles must not block A* routing.
    const PIPE_NAMES: &[&str] = &["pipe", "pipe-to-ground"];
    let pipe_tiles: FxHashSet<(i32, i32)> = row_entities
        .iter()
        .chain(entities.iter())
        .filter(|e| PIPE_NAMES.contains(&e.name.as_str()))
        .map(|e| (e.x, e.y))
        .collect();
    let occupied_tiles: FxHashSet<(i32, i32)> = row_entities
        .iter()
        .chain(entities.iter())
        .flat_map(|e| {
            if MACHINE_ENTITIES.contains(&e.name.as_str()) {
                let sz = machine_size(&e.name);
                machine_tiles(e.x, e.y, sz)
            } else {
                vec![(e.x, e.y)]
            }
        })
        .collect();
    let astar_hard: FxHashSet<(i32, i32)> = hard
        .iter()
        .filter(|t| !pipe_tiles.contains(t))
        .filter(|t| !fluid_reservations.contains(t) || occupied_tiles.contains(t))
        .copied()
        .collect();

    const MAX_NEGOTIATION_ITERATIONS: u32 = 8;
    // History penalty: accumulated across iterations on tiles that had
    // same-axis conflicts in previous iterations.
    const HISTORY_PENALTY_K: u32 = 4;
    // Present penalty: bumped per spec INSIDE an iteration. Each spec's
    // routing pays a per-tile cost based on how many already-routed specs
    // in the current iteration used that tile in the same axis.
    const PRESENT_PENALTY_K: u32 = 6;
    const MAX_NO_IMPROVEMENT: u32 = 2;

    let mut history_cost_grid: FxHashMap<(i32, i32), (u32, u32)> = FxHashMap::default();
    let mut best_paths: FxHashMap<String, Vec<(i32, i32)>> = FxHashMap::default();
    let mut best_unroutable: Vec<String> = Vec::new();
    let mut best_same_axis: u32 = u32::MAX;
    let mut no_improvement_streak: u32 = 0;

    for iter in 0..MAX_NEGOTIATION_ITERATIONS {
        // Reset per-iteration routing state.
        let mut iter_routed: FxHashMap<String, Vec<(i32, i32)>> = FxHashMap::default();
        let mut iter_existing = pre_routing_existing_belts.clone();
        let mut iter_unroutable: Vec<String> = Vec::new();
        // Per-iteration cost grid = history (carried across iters) + present
        // (rebuilt each iter, bumped after each spec routes).
        let mut iter_cost_grid: FxHashMap<(i32, i32), (u32, u32)> =
            history_cost_grid.clone();

        for spec in ordered_specs.iter().copied() {
            match ghost_astar(
                spec.start,
                spec.goal,
                &astar_hard,
                &iter_existing,
                width,
                height,
                TURN_PENALTY,
                &iter_cost_grid,
            ) {
                Some((path, crossings)) => {
                    // Stream the per-spec route so the browser overlay can
                    // watch negotiation happen instead of seeing everything
                    // flash in at the end of routing. Path tiles + crossings
                    // computed above are the same shape the final post-
                    // negotiation `GhostSpecRouted` event carries.
                    trace::emit(trace::TraceEvent::GhostSpecRouted {
                        spec_key: spec.key.clone(),
                        path_len: path.len(),
                        crossings: crossings.len(),
                        turns: count_turns(&path),
                        tiles: path.clone(),
                        crossing_tiles: crossings.clone(),
                    });
                    // Incrementally bump the present cost for tiles used by
                    // this spec, in the spec's axis at each tile. Subsequent
                    // specs in this iteration will pay the bumped cost.
                    if path.len() >= 2 {
                        let last_idx = path.len() - 1;
                        for (i, &tile) in path.iter().enumerate() {
                            let (dx, dy) = if i < last_idx {
                                (path[i + 1].0 - tile.0, path[i + 1].1 - tile.1)
                            } else {
                                (tile.0 - path[i - 1].0, tile.1 - path[i - 1].1)
                            };
                            let entry = iter_cost_grid.entry(tile).or_insert((0, 0));
                            if dx == 0 && dy != 0 {
                                entry.0 += PRESENT_PENALTY_K;
                            } else if dy == 0 && dx != 0 {
                                entry.1 += PRESENT_PENALTY_K;
                            }
                        }
                    }
                    for &tile in &path {
                        iter_existing.insert(tile);
                    }
                    iter_routed.insert(spec.key.clone(), path);
                }
                None => {
                    iter_unroutable.push(spec.key.clone());
                }
            }
        }

        // Compute axis counts for this iteration.
        let mut axis_counts: FxHashMap<(i32, i32), (u32, u32)> = FxHashMap::default();
        for path in iter_routed.values() {
            if path.len() < 2 {
                continue;
            }
            let last_idx = path.len() - 1;
            for (i, &tile) in path.iter().enumerate() {
                let (dx, dy) = if i < last_idx {
                    (path[i + 1].0 - tile.0, path[i + 1].1 - tile.1)
                } else {
                    (tile.0 - path[i - 1].0, tile.1 - path[i - 1].1)
                };
                let entry = axis_counts.entry(tile).or_insert((0, 0));
                if dx == 0 && dy != 0 {
                    entry.0 += 1;
                } else if dy == 0 && dx != 0 {
                    entry.1 += 1;
                }
            }
        }

        let mut iter_same_axis: u32 = 0;
        let mut iter_perp: u32 = 0;
        for &(v, h) in axis_counts.values() {
            if v >= 2 || h >= 2 {
                iter_same_axis += 1;
            }
            if v >= 1 && h >= 1 {
                iter_perp += 1;
            }
        }

        trace::emit(trace::TraceEvent::GhostNegotiationIteration {
            iter,
            same_axis_conflict_count: iter_same_axis,
            perpendicular_crossing_count: iter_perp,
            unroutable_count: iter_unroutable.len() as u32,
            cost_grid_size: history_cost_grid.len() as u32,
        });

        // Track the best routing across iterations.
        if iter_same_axis < best_same_axis {
            best_same_axis = iter_same_axis;
            best_paths = iter_routed;
            best_unroutable = iter_unroutable;
            no_improvement_streak = 0;
        } else {
            no_improvement_streak += 1;
        }

        // Stop conditions.
        if iter_same_axis == 0 {
            break;
        }
        if no_improvement_streak >= MAX_NO_IMPROVEMENT {
            break;
        }

        // Bump the HISTORY cost grid for tiles with same-axis conflicts.
        // Per-axis: only the over-crowded axis gets a higher penalty, leaving
        // the other axis free to keep using the tile. This carries across
        // iterations to discourage repeat conflicts at the same tiles.
        for (&tile, &(v, h)) in &axis_counts {
            if v >= 2 {
                let entry = history_cost_grid.entry(tile).or_insert((0, 0));
                entry.0 += HISTORY_PENALTY_K * (v - 1);
            }
            if h >= 2 {
                let entry = history_cost_grid.entry(tile).or_insert((0, 0));
                entry.1 += HISTORY_PENALTY_K * (h - 1);
            }
        }
    }

    // Adopt the best routing as the canonical one.
    routed_paths = best_paths;
    unroutable_specs = best_unroutable;
    // Inject synthetic trunk column paths so classify_crossing and the
    // junction solver find trunk specs at crossing tiles (same role they
    // played when trunks were BeltSpecs routed through A*).
    for (key, path) in &trunk_synth_paths {
        routed_paths.insert(key.clone(), path.clone());
    }

    // -------------------------------------------------------------------------
    // Materialize entities from the converged routed_paths.
    // Replays the per-spec materialization logic in spec order so that
    // existing_belts/ghost_item_at/all_ghost_crossings end up in the same
    // shape they had before the negotiation refactor.
    // -------------------------------------------------------------------------
    existing_belts = pre_routing_existing_belts;
    ghost_item_at.clear();
    // Pre-load trunk tile → item mappings. In the old code, trunk specs
    // materialised first and populated ghost_item_at. Now trunks are
    // pre-stamped and have no routed_paths entry, so we inject them here.
    // This ensures: (a) tap entities on trunk tiles are dropped by the
    // ghost_item_at filter, and (b) different-item crossings at trunk tiles
    // are still added to all_ghost_crossings for the junction solver.
    for (&tile, item) in &trunk_tile_items {
        ghost_item_at.insert(tile, item.clone());
    }

    for spec in ordered_specs.iter().copied() {
        if let Some(path) = routed_paths.get(&spec.key).cloned() {
            // Recompute crossings from the final state of existing_belts so
            // they reflect the converged routing order.
            let crossings: Vec<(i32, i32)> = path
                .iter()
                .copied()
                .filter(|t| existing_belts.contains(t))
                .collect();

            let turns = count_turns(&path);
            trace::emit(trace::TraceEvent::GhostSpecRouted {
                spec_key: spec.key.clone(),
                path_len: path.len(),
                crossings: crossings.len(),
                turns,
                tiles: path.clone(),
                crossing_tiles: crossings.clone(),
            });

            // Emit entities via render_path. When the planner set an explicit
            // exit_dir on the spec (orientation-aware ret/feeder, vertical
            // trunks), use it directly — this is the only reliable direction
            // source for length-1 paths (start == goal, or A* blocked fallback
            // to goal tile only). When unset, infer from start/goal coordinate
            // comparison (legacy behaviour for multi-tile horizontal/vertical
            // specs where the path deltas in render_path carry the direction).
            let direction_hint = if let Some(d) = spec.exit_dir {
                d
            } else if spec.start.1 != spec.goal.1 && spec.start.0 == spec.goal.0 {
                if spec.goal.1 > spec.start.1 {
                    EntityDirection::South
                } else {
                    EntityDirection::North
                }
            } else if spec.start.0 <= spec.goal.0 {
                EntityDirection::East
            } else {
                EntityDirection::West
            };
            // All specs materialised here are tap/ret/feeder flows — trunks
            // are stamped directly by step 3.5 and never reach A*. Post-
            // Phases 1-3 of `rfp-unified-belt-specs.md`, no spec key has a
            // `trunk:` prefix, so the segment id is always `ghost:...`.
            let spec_seg_id = Some(format!("ghost:{}", spec.key));
            let path_ents = render_path(
                &path,
                &spec.item,
                spec.belt_name,
                direction_hint,
                spec_seg_id,
                None,
            );
            // Materialise the path into entities + Occupancy.
            //
            // Claim kind: trunk specs become load-bearing vertical bus
            // belts (`Permanent` claims). All other specs (tap-offs,
            // returns, horizontals) get `GhostSurface` claims which
            // templates and SAT may replace. This mirrors the old
            // `pre_existing_positions` filter semantics — non-ghost
            // segment IDs (trunks) were kept as obstacles; ghost
            // segment IDs were skipped.
            //
            // Filter: drop path tiles that already hold a pre-existing
            // belt (from row templates / step 2-3 setup), already
            // carry another ghost item (first-spec-wins), or overlap a
            // hard obstacle. The hard-obstacle filter protects against
            // `ghost_astar:695` which allows goal tiles on hard
            // obstacles (and silently also start tiles — no check at
            // `astar.rs:658`). Dropping those entities prevents
            // entity-overlap validator errors on fluid-lane
            // reservations and machine anchors.
            // All materialised specs are horizontals/returns/feeders (see
            // comment at `spec_seg_id` above); post-Phases 1-3, none of them
            // are `trunk:*`. Claim kind is always `GhostSurface` — templates
            // and SAT may replace these tiles.
            let claim_kind = crate::bus::ghost_occupancy::ClaimKindTag::GhostSurface;
            let surviving_ents: Vec<PlacedEntity> = path_ents
                .into_iter()
                .filter(|e| {
                    // Use `astar_hard` so belts can survive at pipe tiles
                    // (soft obstacles) and at reserved-but-empty fluid-column
                    // tiles (the tunnel-through area between a UG-in and
                    // UG-out on a fluid trunk — belts can cross over PTG
                    // tunnels per F7).
                    !pre_ghost_belts.contains(&(e.x, e.y))
                        && !ghost_item_at.contains_key(&(e.x, e.y))
                        && !astar_hard.contains(&(e.x, e.y))
                })
                .collect();
            // Stream the materialised entities so a live renderer can
            // swap its per-tile ghost-belt placeholders for the real
            // turn-aware / UG-aware shapes. Fires once per spec after
            // `GhostSpecRouted` for the same spec.
            //
            // Emitted AFTER the survivors filter so the streaming UI
            // never commits a feeder-belt at a tile already owned by
            // a balancer/row/permanent pre-stamp. Emitting unfiltered
            // path_ents here used to leave West-facing feeder ghosts
            // rendered over South-facing balancer tiles — same final
            // layout entity, wrong visual.
            trace::emit(trace::TraceEvent::GhostSpecCommitted {
                spec_key: spec.key.clone(),
                entities: surviving_ents.clone(),
            });
            for ent in &surviving_ents {
                occupancy
                    .place(ent.clone(), claim_kind)
                    .unwrap_or_else(|err| {
                        panic!(
                            "occupancy refactor: place failed for spec {} at ({},{}): {:?}",
                            spec.key, ent.x, ent.y, err
                        );
                    });
            }
            entities.extend(surviving_ents);

            for &tile in &path {
                existing_belts.insert(tile);
            }

            all_ghost_crossings.extend(crossings.into_iter().filter(|t| {
                if pre_ghost_belts.contains(t) {
                    return false;
                }
                match ghost_item_at.get(t) {
                    Some(existing_item) => *existing_item != spec.item,
                    None => false,
                }
            }));

            for &tile in &path {
                ghost_item_at.entry(tile).or_insert_with(|| spec.item.clone());
            }
        }
    }

    // Step 3 of the occupancy refactor: verify that every tile in the
    // post-materialisation `existing_belts` set has a corresponding claim
    // in Occupancy. Reverse direction does not hold — Occupancy includes
    // machines, poles, and fluid-lane reservations that `existing_belts`
    // does not.
    #[cfg(debug_assertions)]
    {
        for &tile in &existing_belts {
            debug_assert!(
                occupancy.is_claimed(tile),
                "occupancy refactor: existing_belts tile {:?} not claimed in Occupancy after materialisation",
                tile,
            );
        }
    }

    // Emit GhostSpecFailed events for specs that didn't route.
    for failed_key in &unroutable_specs {
        if let Some(spec) = ordered_specs.iter().find(|s| &s.key == failed_key) {
            trace::emit(trace::TraceEvent::GhostSpecFailed {
                spec_key: failed_key.clone(),
                from_x: spec.start.0,
                from_y: spec.start.1,
                to_x: spec.goal.0,
                to_y: spec.goal.1,
            });
        }
    }

    // -------------------------------------------------------------------------
    // Phase-1 instrumentation: per-tile axis occupancy
    // -------------------------------------------------------------------------
    // For each tile in routed_paths, determine the spec's outgoing axis
    // (vertical N/S or horizontal E/W). Last tile uses incoming direction.
    // Aggregate counts per tile and emit a summary trace event so the web
    // overlay can visualize same-axis conflicts vs perpendicular crossings.
    {
        use crate::trace::GhostAxisOccupancyTile;

        let mut axis_counts: FxHashMap<(i32, i32), (u32, u32)> = FxHashMap::default();
        for path in routed_paths.values() {
            if path.len() < 2 {
                continue;
            }
            let last_idx = path.len() - 1;
            for (i, &tile) in path.iter().enumerate() {
                let (dx, dy) = if i < last_idx {
                    (path[i + 1].0 - tile.0, path[i + 1].1 - tile.1)
                } else {
                    (tile.0 - path[i - 1].0, tile.1 - path[i - 1].1)
                };
                let entry = axis_counts.entry(tile).or_insert((0, 0));
                if dx == 0 && dy != 0 {
                    entry.0 += 1; // vertical
                } else if dy == 0 && dx != 0 {
                    entry.1 += 1; // horizontal
                }
            }
        }

        let mut tiles: Vec<GhostAxisOccupancyTile> = Vec::new();
        let mut same_axis_conflict_count: u32 = 0;
        let mut perpendicular_crossing_count: u32 = 0;
        for (&(x, y), &(v, h)) in &axis_counts {
            let same_axis = v >= 2 || h >= 2;
            let perp = v >= 1 && h >= 1;
            if same_axis {
                same_axis_conflict_count += 1;
            }
            if perp {
                perpendicular_crossing_count += 1;
            }
            if same_axis || perp {
                tiles.push(GhostAxisOccupancyTile {
                    x,
                    y,
                    vert_count: v,
                    horiz_count: h,
                });
            }
        }
        tiles.sort_by_key(|t| (t.y, t.x));

        trace::emit(trace::TraceEvent::GhostAxisOccupancy {
            tiles,
            same_axis_conflict_count,
            perpendicular_crossing_count,
        });
    }

    // -------------------------------------------------------------------------
    // Step 6: Resolve ghost crossings — templates first, SAT fallback
    // -------------------------------------------------------------------------
    let crossing_set: FxHashSet<(i32, i32)> = all_ghost_crossings.iter().copied().collect();

    // Running count of templates emitted in step 6a. Started at 0
    // because the corridor-template pre-pass was removed; all
    // crossings now flow through the junction-solver cluster loop
    // below, which increments this counter for each solved cluster.
    // See `docs/archive/rfp-remove-corridor-template.md` for the rationale.
    let mut template_count: usize = 0;
    let mut template_regions: Vec<LayoutRegion> = Vec::new();
    let mut remaining_crossings: FxHashSet<(i32, i32)> = FxHashSet::default();
    // Tracks crossings already resolved by an earlier cluster's
    // solution footprint. Populated inside the cluster loop so a grown
    // SAT zone that spans multiple original crossings doesn't produce
    // a second, partial solution for the same tiles. Name kept
    // historical — previously also held corridor-template stamps.
    let mut corridor_handled: FxHashSet<(i32, i32)> = FxHashSet::default();
    // Maximum underground-belt distance in tiles for this belt tier.
    // Used downstream by the UG-pair interior calculation (`dist` loop
    // below); was previously defined inline in the corridor-template
    // block.
    let max_reach = ug_max_reach(max_belt_tier.unwrap_or("transport-belt")) as i32;

    // Step 6a: Per-tile crossing resolution via the junction-solver
    // growth loop. The loop seeds a `GrowingRegion` from each remaining
    // crossing tile, runs the registered strategies, and grows the
    // region's participating-spec frontier when none succeed. Today
    // the only strategy is `PerpendicularTemplateStrategy`, a wrapper
    // around the existing per-tile template — so behaviour matches the
    // old direct-call path for every crossing the old code solved.
    // Growth-aware strategies land on top of this scaffold.

    let mut spec_belt_tiers: FxHashMap<String, BeltTier> = specs
        .iter()
        .map(|s| {
            (
                s.key.clone(),
                BeltTier::from_name(s.belt_name).unwrap_or(BeltTier::Yellow),
            )
        })
        .collect();
    let mut spec_items: FxHashMap<String, String> = specs
        .iter()
        .map(|s| (s.key.clone(), s.item.clone()))
        .collect();
    let mut spec_exit_dirs: FxHashMap<String, EntityDirection> = specs
        .iter()
        .filter_map(|s| s.exit_dir.map(|d| (s.key.clone(), d)))
        .collect();
    // Extend with synthetic trunk entries so classify_crossing can resolve
    // item name and belt tier for trunk keys found in routed_paths. Keys
    // match the per-column format used when populating `trunk_synth_paths`
    // ("trunk:{item}:{x}") so multi-lane items register distinct keys per
    // column.
    for lane in lanes {
        if lane.is_fluid {
            continue;
        }
        let key = format!("trunk:{}:{}", lane.item, lane.x);
        spec_items.insert(key.clone(), lane.item.clone());
        spec_belt_tiers.insert(
            key,
            BeltTier::from_name(belt_entity_for_rate(lane.rate * 2.0, max_belt_tier))
                .unwrap_or(BeltTier::Yellow),
        );
    }

    // -------------------------------------------------------------------------
    // Phases 1+2 of `docs/rfp-unified-belt-specs.md`: unify trunk+last-tap
    // flows into one `flow:{item}:{x}` entry in routed_paths and the three
    // spec_* maps. Presents the junction solver with a single coherent
    // flow rather than two specs that pin the handoff tile from both
    // sides — the root cause of `advanced_circuit_iron_plate_trio_capped`
    // and all single-pin variants.
    //
    // Phase 1 (landed 2026-04-21) handled `tap_off_ys.len() == 1`. Phase 2
    // extends the same post-routing pass to multi-tap lanes: the trunk
    // column and the *last* tap are fused (they already form one continuous
    // bent belt), while non-last taps keep their standalone `tap:` specs
    // because each is its own physical belt fed by a splitter output.
    // Internal gaps in the trunk path (at non-last tap rows and their
    // splitter rows) are preserved in the unified Vec; `direction_at`
    // handles jumps correctly (same-axis steps still derive the right
    // flow direction from non-adjacent indices).
    //
    // Materialisation already ran (line 794-ish) using the original
    // trunk/tap keys, so entity stamping is unaffected. The corridor
    // template also already ran and sees the decomposed view; only the
    // cluster-formation + junction-solve phase downstream sees the
    // unified keys.
    for lane in lanes {
        if lane.is_fluid || lane.tap_off_ys.is_empty() {
            continue;
        }
        let x = lane.x;
        let last_tap_y = lane
            .tap_off_ys
            .iter()
            .copied()
            .max()
            .expect("tap_off_ys non-empty by guard above");
        let trunk_key = format!("trunk:{}:{}", lane.item, x);
        let last_tap_key = format!("tap:{}:{}:{}", lane.item, x, last_tap_y);

        let Some(trunk_path) = routed_paths.get(&trunk_key).cloned() else {
            continue;
        };
        let Some(last_tap_path) = routed_paths.get(&last_tap_key).cloned() else {
            continue;
        };

        // Trunk ends at (x, last_tap_y - 1) (last_tap_y is in skip_ys).
        // Last tap starts at (x, last_tap_y) because `is_last = true`
        // forces start_x = x. The two sequences are adjacent with no
        // overlap, so direct concatenation produces a bent-belt path
        // that carries the item from the trunk top all the way to the
        // last consumer. Non-last taps are *separate* physical belts
        // fed by splitter east-outputs — they stay as their own `tap:`
        // specs and are NOT folded in here.
        let mut unified_path = trunk_path;
        unified_path.extend(last_tap_path);

        let unified_key = format!("flow:{}:{}", lane.item, x);
        routed_paths.insert(unified_key.clone(), unified_path);
        routed_paths.remove(&trunk_key);
        routed_paths.remove(&last_tap_key);

        let tier = spec_belt_tiers
            .remove(&last_tap_key)
            .or_else(|| spec_belt_tiers.remove(&trunk_key))
            .unwrap_or(BeltTier::Yellow);
        spec_belt_tiers.remove(&trunk_key);
        spec_belt_tiers.insert(unified_key.clone(), tier);

        spec_items.remove(&trunk_key);
        spec_items.remove(&last_tap_key);
        spec_items.insert(unified_key.clone(), lane.item.clone());

        // exit_dir propagates from the last tap (east at the bus-edge
        // terminus) since that's where the unified flow exits the
        // region. The trunk had no exit_dir of its own.
        if let Some(dir) = spec_exit_dirs.remove(&last_tap_key) {
            spec_exit_dirs.insert(unified_key, dir);
        }
        spec_exit_dirs.remove(&trunk_key);
    }

    // Build the obstacle set seen by junction strategies. The narrow
    // `hard` set only covers row-template machines and fluid lanes;
    // SAT (and any future strategy) needs the full picture so it
    // doesn't stamp belts on trunks, tap-off splitters, prior template
    // output, or row belts. Pulled from Occupancy at this point in
    // the pipeline — covers everything except `GhostSurface`, which
    // strategies are allowed to replace.
    let junction_hard: FxHashSet<(i32, i32)> = occupancy.snapshot_junction_obstacles();
    // Subset of `junction_hard` whose claims would panic if perp-template
    // stamped over them. `release_for_pertile_template` clears trunks and
    // tapoffs inside the footprint, and the post-place loop
    // (ghost_router.rs:1356) benignly skips `Template` and `RowEntity`
    // collisions. What remains, and what panics, is `Permanent` claims
    // whose segment id is NOT trunk/tapoff/row — balancer belts,
    // corridor-perp re-adds, merger chains — plus hard obstacles.
    // `PerpendicularTemplateStrategy` consults this narrower set so it
    // returns `None` instead of producing a panicking solution.
    let unreleasable_obstacles: FxHashSet<(i32, i32)> = entities
        .iter()
        .filter(|e| {
            let seg = e.segment_id.as_deref().unwrap_or("");
            !seg.starts_with("trunk:")
                && !seg.starts_with("tapoff:")
                && !seg.starts_with("ghost:")
                && !seg.starts_with("row:")
                && !seg.starts_with("junction:")
                && !seg.starts_with("corridor:")
                && !seg.starts_with("crossing:")
        })
        .map(|e| (e.x, e.y))
        .chain(hard.iter().copied())
        .collect();
    let perp_strategy = PerpendicularTemplateStrategy;
    let sat_surface = SatStrategy::surface_only();
    let sat_1ug = SatStrategy::with(
        "sat-1ug",
        crate::bus::junction_sat_strategy::SatConstraints::max_ug_ins(1),
    );
    let sat_2ug = SatStrategy::with(
        "sat-2ug",
        crate::bus::junction_sat_strategy::SatConstraints::max_ug_ins(2),
    );
    let sat_full = SatStrategy::unrestricted();
    // Strategy order = priority. Walker vetoes bad proposals from any
    // of them; escalation happens naturally by falling through to the
    // next strategy in the list.
    //   1. cheap templates (fixed footprint, no search)
    //   2. surface-only SAT — simplest layout, no UG at all
    //   3-4. SAT with an increasing UG budget — the solver has to
    //        justify each corridor by infeasibility at the previous
    //        cap, so it won't spend UG pairs on items that surface
    //        could route (e.g. a straight iron trunk next to a genuine
    //        copper-cable crossing).
    //   5. SAT unrestricted — final fallback for layouts that need
    //      more than 2 UG corridors in one junction.
    let strategies: [&dyn JunctionStrategy; 5] =
        [&perp_strategy, &sat_surface, &sat_1ug, &sat_2ug, &sat_full];

    // Group adjacent crossings that share a spec into a single cluster
    // and solve each cluster jointly. A single crossing is still a
    // valid 1-tile cluster. Clustering prevents the old failure mode
    // where N identical adjacent crossings each grew a 9×9 zone
    // independently, overlapped heavily, and corrupted one another.
    // Cluster only the crossings the corridor template didn't resolve.
    // Mixing handled tiles into clusters was a silent bug under the
    // Manhattan-2 rule: a cluster could span a resolved run (e.g.
    // plastic-bar 25-26) plus an unresolved one (e.g. iron-plate 21-23)
    // via a shared horizontal spec, and the `any(is_handled)` check
    // below would then discard the whole cluster — leaving the
    // unresolved crossings unsolved with no error.
    let unhandled_crossings: FxHashSet<(i32, i32)> = crossing_set
        .iter()
        .filter(|t| !corridor_handled.contains(t))
        .copied()
        .collect();
    let clusters = cluster_adjacent_crossings(
        &unhandled_crossings,
        &routed_paths,
        &spec_belt_tiers,
    );

    for cluster in &clusters {
        // Cluster is already guaranteed unhandled by the filter above;
        // keep the check as a cheap invariant in debug builds.
        debug_assert!(
            !cluster.iter().any(|t| corridor_handled.contains(t)),
            "cluster contains a corridor-handled tile despite pre-cluster filter"
        );
        // classify_crossing gates on "exactly two specs with a valid
        // direction at the tile" — require it of every cluster member.
        // If any member is degenerate the whole cluster defers to
        // unresolved, matching today's per-tile conservatism.
        if cluster.iter().any(|&t| {
            classify_crossing(t, &routed_paths, &specs, &spec_items, &spec_belt_tiers)
                .is_none()
        }) {
            for &t in cluster {
                remaining_crossings.insert(t);
            }
            continue;
        }
        // Union of spec keys across every cluster tile. Scan
        // routed_paths directly so synthetic trunk keys (injected
        // above) are included alongside regular BeltSpec keys.
        let cluster_tiles: FxHashSet<(i32, i32)> = cluster.iter().copied().collect();
        let keys_at_tile: Vec<&str> = routed_paths
            .iter()
            .filter(|(_, path)| path.iter().any(|t| cluster_tiles.contains(t)))
            .map(|(key, _)| key.as_str())
            .collect();

        // Pending crossings for the DeferredExit check: the subset of
        // `crossing_set` whose cluster hasn't committed yet. Excluding
        // `corridor_handled` avoids false deferrals when a zone's exit
        // lands on a tile that was already solved by a prior cluster —
        // that tile now has a committed entity, it's not a pending
        // collision anymore.
        let pending_crossings: FxHashSet<(i32, i32)> = crossing_set
            .iter()
            .filter(|t| !corridor_handled.contains(t))
            .copied()
            .collect();

        // Optional capture: dump a region-solver fixture on this
        // invocation. Gated by `FUCKTORIO_DUMP_REGION_FIXTURE=<dir>`;
        // narrowable with `FUCKTORIO_DUMP_REGION_FIXTURE_SEED="x,y"` to
        // match a specific cluster seed. Off by default, zero-cost when
        // unset. See `crates/core/tests/region_fixtures/README.md`.
        #[cfg(not(target_arch = "wasm32"))]
        dump_region_fixture(
            cluster.as_slice(),
            &keys_at_tile,
            &routed_paths,
            &hard,
            &junction_hard,
            &unreleasable_obstacles,
            &spec_belt_tiers,
            &spec_items,
            &spec_exit_dirs,
            &entities,
            &pending_crossings,
        );

        let Some(sol) = junction_solver::solve_crossing(
            cluster.as_slice(),
            &keys_at_tile,
            &routed_paths,
            &hard,
            &junction_hard,
            &unreleasable_obstacles,
            &spec_belt_tiers,
            &spec_items,
            &spec_exit_dirs,
            &entities,
            &strategies,
            &pending_crossings,
        ) else {
            for &t in cluster {
                remaining_crossings.insert(t);
            }
            continue;
        };

        // Every cluster member is now handled, regardless of whether
        // it sits inside the solution footprint.
        for &t in cluster {
            corridor_handled.insert(t);
        }

        let footprint = sol.footprint;
        // Mark any other crossing tiles inside this zone's footprint as
        // handled. A grown zone (e.g. a 4-tile wide SAT solution) may span
        // several original crossing tiles; if we let the loop visit them
        // independently the solver produces a second, broken solution that
        // only partially overlaps the first (e.g. a UG output with no input).
        for &ct in &crossing_set {
            if ct.0 >= footprint.x
                && ct.0 < footprint.x + footprint.w as i32
                && ct.1 >= footprint.y
                && ct.1 < footprint.y + footprint.h as i32
            {
                corridor_handled.insert(ct);
            }
        }
        trace::emit(trace::TraceEvent::GhostClusterSolved {
            cluster_id: template_count,
            zone_x: footprint.x,
            zone_y: footprint.y,
            zone_w: footprint.w,
            zone_h: footprint.h,
            boundary_count: 4,
            variables: 0,
            clauses: 0,
            solve_time_us: 0,
        });
        template_regions.push(match &sol.sat_zone {
            Some(snap) => {
                use crate::models::{PortIo, PortPoint, RegionKind, RegionPort};
                let ports: Vec<RegionPort> = snap
                    .boundaries
                    .iter()
                    .map(|b| RegionPort {
                        point: PortPoint {
                            x: b.x,
                            y: b.y,
                            direction: b.direction,
                        },
                        io: if b.is_input {
                            PortIo::Input
                        } else {
                            PortIo::Output
                        },
                        item: Some(b.item.clone()),
                        interior: b.interior,
                    })
                    .collect();
                LayoutRegion {
                    id: 0,
                    kind: RegionKind::CrossingZone,
                    x: footprint.x,
                    y: footprint.y,
                    width: footprint.w as i32,
                    height: footprint.h as i32,
                    ports,
                    forced_empty: snap.forced_empty.clone(),
                    belt_tier: Some(snap.belt_tier.clone()),
                    max_ug_reach: Some(snap.max_ug_reach),
                }
            }
            None => LayoutRegion {
                id: 0,
                kind: crate::models::RegionKind::JunctionTemplate,
                x: footprint.x,
                y: footprint.y,
                width: footprint.w as i32,
                height: footprint.h as i32,
                ports: Vec::new(),
                forced_empty: Vec::new(),
                belt_tier: None,
                max_ug_reach: None,
            },
        });

        let release_rect = crate::bus::ghost_occupancy::Rect {
            x: footprint.x,
            y: footprint.y,
            w: footprint.w,
            h: footprint.h,
        };
        // Preserve every tile in the footprint that the strategy is
        // NOT explicitly stamping a new entity at. The strategy's
        // solution is authoritative *exactly* over its proposed
        // tiles; every other tile's existing claim (trunk belt,
        // balancer splitter, non-participating tap, whatever) must
        // remain intact so the chain around the crossing stays
        // connected.
        //
        // This is the minimum-authority rule: we only touch what
        // the solver explicitly promises to replace. Without it,
        // uniformly-grown bboxes wipe out unrelated trunk/splitter
        // entities just because they sit inside the rectangle.
        let mut proposed_tiles: rustc_hash::FxHashSet<(i32, i32)> =
            sol.entities.iter().map(|e| (e.x, e.y)).collect();
        // Expand `proposed_tiles` to include the interior tiles of every
        // UG pair in the solution. SAT places an entity at the UG-in
        // and UG-out endpoints only — the tiles between them are
        // "tunneled" through underground, so any pre-existing trunk or
        // ghost-stamped surface belt at those interior tiles is now
        // dead geometry and must be released by the cleanup below.
        // Without this, those interior tiles land in `preserve_trunk_tiles`
        // (because SAT didn't touch them explicitly) and SAT's tunnel
        // co-exists with leftover surface belts — "floating belts" that
        // the validator catches as adjacent-item mismatches.
        let ug_pair_interiors: Vec<(i32, i32)> = sol
            .entities
            .iter()
            .filter(|e| e.io_type.as_deref() == Some("input"))
            .flat_map(|ug_in| {
                let (dx, dy) = match ug_in.direction {
                    EntityDirection::North => (0i32, -1i32),
                    EntityDirection::East => (1, 0),
                    EntityDirection::South => (0, 1),
                    EntityDirection::West => (-1, 0),
                };
                let mut interior: Vec<(i32, i32)> = Vec::new();
                for dist in 1..=max_reach {
                    let (ox, oy) = (ug_in.x + dx * dist, ug_in.y + dy * dist);
                    let paired = sol.entities.iter().any(|e| {
                        e.x == ox
                            && e.y == oy
                            && e.io_type.as_deref() == Some("output")
                            && e.direction == ug_in.direction
                    });
                    if paired {
                        for d in 1..dist {
                            interior.push((ug_in.x + dx * d, ug_in.y + dy * d));
                        }
                        break;
                    }
                }
                interior
            })
            .collect();
        proposed_tiles.extend(ug_pair_interiors);
        let preserve_trunk_tiles: rustc_hash::FxHashSet<(i32, i32)> =
            (0..release_rect.h as i32)
                .flat_map(|dy| {
                    (0..release_rect.w as i32)
                        .map(move |dx| (release_rect.x + dx, release_rect.y + dy))
                })
                .filter(|t| !proposed_tiles.contains(t))
                .collect();
        // Only release ghost surface entities that lie on a participating
        // spec path. Ghost entities belonging to non-participating specs
        // (e.g. a copper-cable tap whose path runs through the zone bbox
        // but is NOT being rerouted) must stay so the belt chain is intact.
        //
        // The authoritative participating set comes from the solver on
        // `sol.participating`, not from `keys_at_tile` — the latter is
        // built from specs touching the original cluster seeds, so after
        // region growth it can miss participating specs whose path only
        // enters the footprint via the grown bbox. Using the solver's
        // list closes that gap and keeps I3's minimum-authority rule
        // honest.
        let participating_keys: rustc_hash::FxHashSet<&str> = sol
            .participating
            .iter()
            .map(String::as_str)
            .collect();
        let releasable_ghost_tiles: rustc_hash::FxHashSet<(i32, i32)> = routed_paths
            .iter()
            .filter(|(k, _)| participating_keys.contains(k.as_str()))
            .flat_map(|(_, path)| path.iter().copied())
            .filter(|&t| release_rect.contains(t.0, t.1))
            .collect();
        let released_count = occupancy.release_for_pertile_template(
            &release_rect,
            Some(&releasable_ghost_tiles),
            Some(&preserve_trunk_tiles),
        );
        trace::emit(trace::TraceEvent::GhostResidueCleared {
            zone_x: release_rect.x,
            zone_y: release_rect.y,
            zone_w: release_rect.w,
            zone_h: release_rect.h,
            participating_count: participating_keys.len(),
            released_count,
        });
        let mut committed_entities: Vec<crate::models::PlacedEntity> = Vec::new();
        for ent in sol.entities {
            let tile = (ent.x, ent.y);
            if occupancy.is_hard_obstacle(tile) {
                continue;
            }
            // Two per-tile templates with overlapping footprints —
            // the second one's stamp is skipped to match the
            // legacy post-hoc `occupied` filter behaviour. We also
            // skip the entity from the entity list entirely, otherwise
            // it leaks an orphan belt at a tile that's already claimed
            // by an earlier template, and the validator flags it as
            // an entity-overlap with the earlier entity.
            if matches!(
                occupancy.claim_at(tile),
                Some(crate::bus::ghost_occupancy::Claim::Template { .. })
                    | Some(crate::bus::ghost_occupancy::Claim::RowEntity { .. })
            ) {
                continue;
            }
            occupancy
                .place(
                    ent.clone(),
                    crate::bus::ghost_occupancy::ClaimKindTag::Template,
                )
                .unwrap_or_else(|err| {
                    panic!(
                        "occupancy refactor: template place failed at ({},{}): {:?}",
                        tile.0, tile.1, err
                    );
                });
            committed_entities.push(ent.clone());
            entities.push(ent);
        }
        // Stream the solved cluster's entities + the spec keys whose
        // ghost-routed belts inside the footprint were just invalidated.
        // A live renderer uses this to fade out per-tile ghost belts in
        // the zone and fade in the real SAT output.
        trace::emit(trace::TraceEvent::JunctionCommitted {
            cluster_id: template_count,
            zone_x: footprint.x,
            zone_y: footprint.y,
            zone_w: footprint.w,
            zone_h: footprint.h,
            entities: committed_entities,
            participating: participating_keys.iter().map(|s| s.to_string()).collect(),
        });
        // Sync-gap assertion. Any ghost entity for a participating spec
        // that still holds a GhostSurface claim inside the footprint is
        // a leak — `releasable_ghost_tiles` should have covered every
        // tile on every participating path. Non-participating ghost
        // entities are expected to persist by design (foreign trunks),
        // so they don't count.
        let leaked_tiles: Vec<(i32, i32)> = entities
            .iter()
            .filter(|e| {
                let Some(seg) = e.segment_id.as_deref() else { return false; };
                let Some(spec_key) = seg.strip_prefix("ghost:") else { return false; };
                if !release_rect.contains(e.x, e.y) { return false; }
                if !participating_keys.contains(spec_key) { return false; }
                matches!(
                    occupancy.claim_at((e.x, e.y)),
                    Some(crate::bus::ghost_occupancy::Claim::GhostSurface { .. })
                )
            })
            .map(|e| (e.x, e.y))
            .collect();
        if !leaked_tiles.is_empty() {
            trace::emit(trace::TraceEvent::GhostResidueLeaked {
                zone_x: release_rect.x,
                zone_y: release_rect.y,
                leaked_tiles,
            });
        }
        template_count += 1;
    }

    // Step 6b: Emit per-tile "unresolved" regions for crossings that
    // no template could handle. This replaces the old SAT cluster
    // pipeline — the padded-bbox + union-find + varisat approach was
    // producing broken output on every cluster in real layouts (see
    // docs/archive/sat-band-investigation.md).
    //
    // Each unresolved crossing becomes a 1×1 mini-junction whose specs
    // we record as input/output port pairs on a LayoutRegion with
    // `kind = "unresolved"`. Downstream (UI + diagnostic) can render
    // these as "here's where junction-solver work is needed". No
    // entities are emitted — the layout renders with visible gaps
    // where crossings weren't solved.
    let cluster_count = template_count + remaining_crossings.len();
    let max_cluster_tiles = if remaining_crossings.is_empty() { 0 } else { 1 };
    let mut unresolved_regions = emit_unresolved_junctions(
        &remaining_crossings,
        &routed_paths,
        &specs,
        &spec_items,
        &spec_belt_tiers,
        &ghost_item_at,
    );

    let mut regions = template_regions;
    regions.append(&mut unresolved_regions);

    if !remaining_crossings.is_empty() {
        warnings.push(format!(
            "ghost router: {} unresolved crossings (junction solver not yet implemented)",
            remaining_crossings.len()
        ));
    }

    // Step 6: sync `entities` to Occupancy's released state.
    //
    // Templates and SAT write to both `entities` and Occupancy during
    // their phases. When they release/replace a prior claim (a ghost
    // surface belt, a trunk, or a tapoff) via `release_ghost_surface_in`
    // or `release_for_pertile_template`, Occupancy is updated but the
    // old entity stays in the local `entities` Vec. This pass drops
    // any ghost/trunk/tapoff entity whose Occupancy claim no longer
    // matches — i.e., where a later phase stamped over it.
    //
    // Other entity kinds (row templates, step 2/3 entities, templates,
    // SAT solutions, corridor-perp re-adds) are always kept.
    //
    // The post-hoc add loop that previously re-added template/SAT
    // entities via `solved_zones` is gone — Steps 4-5 push those
    // entities directly into the Vec at the moment they're generated.
    entities.retain(|e| {
        let seg = e.segment_id.as_deref().unwrap_or("");
        let occ_claim = occupancy.claim_at((e.x, e.y));
        if seg.starts_with("ghost:") {
            // Keep only if Occupancy still holds a GhostSurface claim
            // at this tile. If the claim was released by a template
            // or SAT solution, drop the entity.
            return matches!(
                occ_claim,
                Some(crate::bus::ghost_occupancy::Claim::GhostSurface { .. })
            );
        }
        if seg.starts_with("trunk:") || seg.starts_with("tapoff:") {
            // Keep only if Occupancy still holds a Permanent claim.
            // A per-tile template that stamped over the trunk will
            // have released the Permanent claim and replaced it with
            // a Template claim.
            return matches!(
                occ_claim,
                Some(crate::bus::ghost_occupancy::Claim::Permanent { .. })
            );
        }
        if seg.starts_with("balancer:") {
            // Keep only if Occupancy still holds a Permanent claim.
            // A junction crossing template that stamped over a simple
            // balancer belt releases its Permanent claim; drop the
            // stale entity so it doesn't overlap the crossing entity.
            return matches!(
                occ_claim,
                Some(crate::bus::ghost_occupancy::Claim::Permanent { .. })
            );
        }
        true
    });

    // -------------------------------------------------------------------------
    // Step 7: Merge output rows for final products
    // -------------------------------------------------------------------------
    let output_items: FxHashSet<String> = solver_result
        .external_outputs
        .iter()
        .filter(|ext| !ext.is_fluid)
        .map(|ext| ext.item.clone())
        .collect();

    for item in &output_items {
        let output_rows: Vec<usize> = row_spans
            .iter()
            .enumerate()
            .filter(|(_, rs)| rs.spec.outputs.iter().any(|o| &o.item == item && !o.is_fluid))
            .map(|(i, _)| i)
            .collect();

        if !output_rows.is_empty() {
            let (merge_ents, merge_end_y, item_merge_x) =
                merge_output_rows(&output_rows, item, row_spans, max_y, max_belt_tier);
            crate::trace::emit(crate::trace::TraceEvent::OutputMerged {
                item: item.clone(),
                rows: output_rows.clone(),
                merge_y: max_y,
            });
            entities.extend(merge_ents);
            max_y = max_y.max(merge_end_y);
            merge_max_x = merge_max_x.max(item_merge_x);
        }
    }

    // -------------------------------------------------------------------------
    // Emit summary trace event
    // -------------------------------------------------------------------------
    trace::emit(trace::TraceEvent::GhostRoutingComplete {
        entity_count: entities.len(),
        cluster_count,
        max_cluster_tiles,
        unroutable_count: unroutable_specs.len(),
    });

    Ok(GhostRouteResult {
        entities,
        ghost_crossing_tiles: crossing_set,
        cluster_count,
        max_cluster_tiles,
        unroutable_specs,
        max_y,
        merge_max_x,
        regions,
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Cluster zone + per-tile template support (used by Step 6a templates)
// ---------------------------------------------------------------------------

/// Bounding box for a ghost cluster zone (padded by 1 tile on each side).
#[derive(Clone, Copy)]
struct ClusterZone {
    /// Padded bbox left
    x: i32,
    /// Padded bbox top
    y: i32,
    /// Padded bbox width
    w: u32,
    /// Padded bbox height
    h: u32,
}

/// Direction from a (dx, dy) step.
fn step_direction(dx: i32, dy: i32) -> EntityDirection {
    if dx > 0 {
        EntityDirection::East
    } else if dx < 0 {
        EntityDirection::West
    } else if dy > 0 {
        EntityDirection::South
    } else {
        EntityDirection::North
    }
}

/// Classify a cluster's crossing pattern by examining which paths pass
/// through its tiles and their directions.
struct CrossingInfo {
    /// The single crossing tile (only set for single-tile clusters).
    tile: (i32, i32),
    /// The two specs that cross, with their direction at the crossing tile.
    spec_a: (String, EntityDirection), // (item, direction)
    spec_b: (String, EntityDirection),
    /// Belt name for each spec (for entity construction).
    belt_a: &'static str,
    belt_b: &'static str,
}

/// Check if two directions are perpendicular.
fn is_perpendicular(a: EntityDirection, b: EntityDirection) -> bool {
    matches!(
        (a, b),
        (EntityDirection::East | EntityDirection::West, EntityDirection::North | EntityDirection::South)
        | (EntityDirection::North | EntityDirection::South, EntityDirection::East | EntityDirection::West)
    )
}

fn is_horizontal(d: EntityDirection) -> bool {
    matches!(d, EntityDirection::East | EntityDirection::West)
}

/// Group crossing tiles into clusters. Two tiles belong to the same
/// cluster iff they share at least one routed spec AND either
///   (a) they are within Manhattan-2 of each other, OR
///   (b) they sit within UG-reach (`max_reach + 1` tiles for the spec's
///       belt tier) along a shared spec's path.
///
/// The UG-reach rule (b) guards against the "adjacent committed UG"
/// failure mode: when cluster A is solved first and stamps a UG pair on
/// a shared spec, a later cluster B within reach of that UG cannot solve
/// without disturbing it — the walker correctly vetoes every SAT output
/// because the UG-pair invariant breaks. Merging A and B into one
/// cluster lets SAT see both ends of the shared spec together and place
/// a single coherent UG layout.
///
/// A `HULL_BUDGET` gate stops pathological chains (e.g. N evenly-spaced
/// crossings each within reach of the next) from fusing into one zone
/// that would exceed `MAX_REGION_TILES`. When the guardrail trips, the
/// chain falls back to separate clusters — same worst case as today.
///
/// Deterministic output: each cluster is sorted `(y, x)` internally,
/// and the outer Vec is sorted by its first tile.
fn cluster_adjacent_crossings(
    crossing_set: &FxHashSet<(i32, i32)>,
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
) -> Vec<Vec<(i32, i32)>> {
    if crossing_set.is_empty() {
        return Vec::new();
    }
    let tiles: Vec<(i32, i32)> = {
        let mut v: Vec<(i32, i32)> = crossing_set.iter().copied().collect();
        v.sort_unstable_by_key(|&(x, y)| (y, x));
        v
    };
    let index_of: FxHashMap<(i32, i32), usize> = tiles
        .iter()
        .enumerate()
        .map(|(i, &t)| (t, i))
        .collect();

    // tile → set of spec keys whose path passes through it.
    let mut tile_specs: Vec<FxHashSet<&str>> =
        vec![FxHashSet::default(); tiles.len()];
    for (key, path) in routed_paths {
        for t in path {
            if let Some(&i) = index_of.get(t) {
                tile_specs[i].insert(key.as_str());
            }
        }
    }

    // Union-find with path compression.
    let mut parent: Vec<usize> = (0..tiles.len()).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    // Merge any two crossings within Manhattan distance 2 that share at
    // least one spec. The outer loop visits every tile, so we only need
    // to emit half the offsets — the other half is covered by symmetry
    // (e.g. (−1, 0) from tile B is (1, 0) from tile A).
    //
    // Why Manhattan 2, not strict orthogonal adjacency: in dense junction
    // regions (e.g. advanced-circuit's output-merger taps) crossings are
    // often 2 tiles apart along a shared spec path. Strict |dx|+|dy|=1
    // left them as separate clusters, forcing the region solver's growth
    // loop to stitch them back together one tile at a time — which blew
    // through MAX_REGION_TILES before a solution could emerge. Manhattan 2
    // captures these near-misses while the shared-spec gate (below) keeps
    // unrelated crossings apart.
    const OFFSETS: &[(i32, i32)] = &[
        (1, 0), (0, 1),       // Manhattan 1 orthogonal
        (2, 0), (0, 2),       // Manhattan 2 orthogonal
        (1, 1), (1, -1),      // Manhattan 2 diagonals
    ];
    for (i, &(x, y)) in tiles.iter().enumerate() {
        for &(dx, dy) in OFFSETS {
            let Some(&j) = index_of.get(&(x + dx, y + dy)) else {
                continue;
            };
            if tile_specs[i].is_disjoint(&tile_specs[j]) {
                continue;
            }
            let ri = find(&mut parent, i);
            let rj = find(&mut parent, j);
            if ri != rj {
                parent[ri] = rj;
            }
        }
    }

    // Second pass: within-UG-reach along a shared spec's path, gated
    // to **single-crossing rescue** — only merges when at least one of
    // the two parties is currently a single-crossing cluster. This is
    // the specific failure mode we fix: an isolated crossing wedged
    // next to a multi-crossing cluster whose earlier solve committed a
    // UG pair in the isolate's growth envelope. Multi-on-multi merges
    // are regressive (they fuse clusters that would otherwise solve
    // independently into super-zones the solver hasn't been tuned for),
    // so we skip them.
    //
    // The HULL_BUDGET cap is still honoured: even a single-to-many
    // rescue won't proceed if the merged hull can't fit growth.
    const HULL_BUDGET: usize = 48;

    // Cluster sizes from the union-find state after the Manhattan-2
    // pass. Kept current as unions happen in this second pass so the
    // "single-crossing" gate sees the right size.
    let mut cluster_size = vec![0usize; tiles.len()];
    for i in 0..tiles.len() {
        let r = find(&mut parent, i);
        cluster_size[r] += 1;
    }

    for (key, path) in routed_paths {
        // Crossings that lie on this spec's path, in path order.
        let mut on_path: Vec<(usize, usize)> = Vec::new();
        for (idx, t) in path.iter().enumerate() {
            if let Some(&i) = index_of.get(t) {
                on_path.push((i, idx));
            }
        }
        if on_path.len() < 2 {
            continue;
        }
        let tier = spec_belt_tiers
            .get(key.as_str())
            .copied()
            .unwrap_or(BeltTier::Yellow);
        let max_reach = ug_max_reach(belt_name_for_tier(tier)) as usize;
        // UG-pair span = max_reach + 1 positions (max_reach hidden-
        // middle tiles). See `sat.rs::encode_underground` and memory
        // `project_prune_first_suspect`.
        let reach_threshold = max_reach + 1;
        for w in on_path.windows(2) {
            let (i, p_i) = w[0];
            let (j, p_j) = w[1];
            if p_j - p_i > reach_threshold {
                continue;
            }
            let ri = find(&mut parent, i);
            let rj = find(&mut parent, j);
            if ri == rj {
                continue;
            }
            // Single-crossing rescue gate: at least one party must be
            // a lone-crossing cluster. Skip multi-to-multi merges —
            // they regress clusters that solve fine independently.
            if cluster_size[ri] != 1 && cluster_size[rj] != 1 {
                continue;
            }
            // Compute hull of the proposed merged cluster. O(n) per
            // check; n = total crossings in the layout (small, <~100
            // in practice).
            let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
            let (mut max_x, mut max_y) = (i32::MIN, i32::MIN);
            for (k, &(tx, ty)) in tiles.iter().enumerate() {
                let rk = find(&mut parent, k);
                if rk == ri || rk == rj {
                    min_x = min_x.min(tx);
                    max_x = max_x.max(tx);
                    min_y = min_y.min(ty);
                    max_y = max_y.max(ty);
                }
            }
            let hull = ((max_x - min_x + 1) as usize)
                * ((max_y - min_y + 1) as usize);
            if hull > HULL_BUDGET {
                continue;
            }
            let merged_size = cluster_size[ri] + cluster_size[rj];
            parent[ri] = rj;
            cluster_size[rj] = merged_size;
            cluster_size[ri] = 0;
        }
    }

    let mut groups: FxHashMap<usize, Vec<(i32, i32)>> = FxHashMap::default();
    for (i, &tile) in tiles.iter().enumerate() {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(tile);
    }
    let mut clusters: Vec<Vec<(i32, i32)>> = groups.into_values().collect();
    for c in &mut clusters {
        c.sort_unstable_by_key(|&(x, y)| (y, x));
    }
    clusters.sort_unstable_by_key(|c| (c[0].1, c[0].0));
    clusters
}

/// Try to classify a single crossing tile as a 2-path crossing.
/// Returns CrossingInfo if exactly 2 different-item specs cross at this tile.
///
/// `spec_items` and `spec_belt_tiers` are consulted as a fallback for keys
/// (e.g. synthetic trunk paths) that don't have a corresponding `BeltSpec`
/// in `specs`.
fn classify_crossing(
    tile: (i32, i32),
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    specs: &[BeltSpec],
    spec_items: &FxHashMap<String, String>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
) -> Option<CrossingInfo> {
    let (cx, cy) = tile;

    let spec_map: FxHashMap<&str, &BeltSpec> = specs.iter().map(|s| (s.key.as_str(), s)).collect();
    // Each entry: (item, belt_name, direction)
    let mut crossing_specs: Vec<(String, &'static str, EntityDirection)> = Vec::new();

    for (key, path) in routed_paths {
        // Derive item and belt_name from BeltSpec when available; fall back
        // to the supplementary maps for synthetic trunk paths.
        let (item, belt_name, exit_dir) = if let Some(spec) = spec_map.get(key.as_str()) {
            (spec.item.clone(), spec.belt_name, spec.exit_dir)
        } else if let Some(it) = spec_items.get(key.as_str()) {
            let tier = spec_belt_tiers
                .get(key.as_str())
                .copied()
                .unwrap_or(BeltTier::Yellow);
            (it.clone(), belt_name_for_tier(tier), None)
        } else {
            continue;
        };

        for (i, &(px, py)) in path.iter().enumerate() {
            if px == cx && py == cy {
                let dir = if i + 1 < path.len() {
                    let (nx, ny) = path[i + 1];
                    step_direction(nx - px, ny - py)
                } else if i > 0 {
                    let (px2, py2) = path[i - 1];
                    step_direction(px - px2, py - py2)
                } else if let Some(d) = exit_dir {
                    // 1-tile path: no neighbour to derive direction from.
                    // Use the explicit exit_dir set at emission time.
                    d
                } else {
                    continue;
                };
                crossing_specs.push((item, belt_name, dir));
                break;
            }
        }
    }

    if crossing_specs.len() != 2 {
        return None;
    }
    let (ref item_a, belt_a, da) = crossing_specs[0];
    let (ref item_b, belt_b, db) = crossing_specs[1];

    Some(CrossingInfo {
        tile,
        spec_a: (item_a.clone(), da),
        spec_b: (item_b.clone(), db),
        belt_a,
        belt_b,
    })
}

/// Build one `LayoutRegion { kind: Unresolved }` per remaining crossing
/// tile. Each region is lowered from a `Junction` — a 1×1 mini-junction
/// with one `SpecCrossing` per crossing spec — so the long-term junction
/// solver pass can consume the same internal shape.
///
/// See `docs/archive/rfp-junction-solver.md` for the target replacement.
fn emit_unresolved_junctions(
    remaining: &FxHashSet<(i32, i32)>,
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    specs: &[BeltSpec],
    spec_items: &FxHashMap<String, String>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
    ghost_item_at: &FxHashMap<(i32, i32), String>,
) -> Vec<LayoutRegion> {
    use crate::bus::junction::{Junction, Rect, SpecCrossing, SpecOrigin};
    use crate::models::{PortPoint, RegionKind};

    let _ = ghost_item_at;

    let mut out: Vec<LayoutRegion> = Vec::with_capacity(remaining.len());

    // Sort tiles for deterministic output (the diagnostic expects stable
    // order across runs).
    let mut tiles: Vec<(i32, i32)> = remaining.iter().copied().collect();
    tiles.sort();

    for (tx, ty) in tiles {
        let bbox = Rect { x: tx, y: ty, w: 1, h: 1 };
        let junction_specs: Vec<SpecCrossing> =
            classify_crossing((tx, ty), routed_paths, specs, spec_items, spec_belt_tiers)
            .map(|info| {
                // 1×1 bbox: entry and exit sit on the same tile; direction
                // encodes the flow. The lowering in `Junction::to_layout_region`
                // picks the correct edges from `(io, direction)`.
                let make = |item: String, dir: EntityDirection, belt: &str| SpecCrossing {
                    item,
                    belt_tier: BeltTier::from_name(belt).unwrap_or(BeltTier::Yellow),
                    entry: PortPoint { x: tx, y: ty, direction: dir },
                    exit: PortPoint { x: tx, y: ty, direction: dir },
                    origin: SpecOrigin::Participating,
                };
                vec![
                    make(info.spec_a.0, info.spec_a.1, info.belt_a),
                    make(info.spec_b.0, info.spec_b.1, info.belt_b),
                ]
            })
            .unwrap_or_default();

        let junction = Junction {
            bbox,
            forbidden: FxHashSet::default(),
            specs: junction_specs,
        };
        out.push(junction.to_layout_region(RegionKind::Unresolved));
    }

    out
}

fn ug_for_belt(belt: &str) -> &'static str {
    match belt {
        "fast-transport-belt" => "fast-underground-belt",
        "express-transport-belt" => "express-underground-belt",
        _ => "underground-belt",
    }
}

/// Returns true if any spec in `routed_paths` has a turn (different incoming
/// and outgoing directions) at the given tile. Used to reject per-tile UG
/// templates: a UG-in/out can't sit on a tile where another spec is turning,
/// because the turning spec would sideload onto the UG and items would be
/// dropped (UG belts only accept items entering from behind in their
/// facing direction, not from a sideload).
fn any_spec_turns_at(
    tile: (i32, i32),
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
) -> bool {
    for path in routed_paths.values() {
        for (i, &t) in path.iter().enumerate() {
            if t != tile {
                continue;
            }
            if i == 0 || i + 1 >= path.len() {
                break;
            }
            let dx_in = t.0 - path[i - 1].0;
            let dy_in = t.1 - path[i - 1].1;
            let dx_out = path[i + 1].0 - t.0;
            let dy_out = path[i + 1].1 - t.1;
            if (dx_in, dy_in) != (dx_out, dy_out) {
                return true;
            }
            break;
        }
    }
    false
}

/// Returns true if a UG-in/out at `tile` facing `bridge_dir` would receive
/// a sideload from a perpendicular spec — either because the tile itself has
/// a perpendicular spec passing through, or because an adjacent SIDE tile
/// (perpendicular to the bridge axis) has a belt flowing INTO this tile.
///
/// In Factorio, sideloads onto UG-input belts only fill the far lane and
/// can dump items wrong; we reject any template that would create one.
///
/// The check considers both (a) in-flight routed ghost specs and (b) already-
/// placed physical entities (splitters, row belts, trunks placed in Step 2-3).
/// Without (b), a splitter whose output drops straight into this tile from a
/// perpendicular direction slips through — the splitter is stamped before
/// ghost routing and never enters `routed_paths`.
///
/// Returns `None` if the tile is fine, or `Some(reason)` identifying the
/// first conflict found. The caller tags the reason with the endpoint
/// it was checking (UG-in vs UG-out).
fn ug_endpoint_conflicts(
    tile: (i32, i32),
    bridge_dir: EntityDirection,
    bridge_spec_key: &str,
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    placed_entities: &[PlacedEntity],
) -> Option<&'static str> {
    let bridge_axis_vert = matches!(bridge_dir, EntityDirection::North | EntityDirection::South);

    // 1. The tile itself: if any other spec has it in its path AND its axis
    //    at the tile differs from the bridge axis, we'd have two
    //    perpendicular belts at the same tile — conflict.
    for (key, path) in routed_paths {
        if key == bridge_spec_key {
            continue;
        }
        for (i, &t) in path.iter().enumerate() {
            if t != tile {
                continue;
            }
            let last_idx = path.len() - 1;
            let (_dx, dy) = if i < last_idx {
                (path[i + 1].0 - t.0, path[i + 1].1 - t.1)
            } else if i > 0 {
                (t.0 - path[i - 1].0, t.1 - path[i - 1].1)
            } else {
                continue;
            };
            let spec_axis_vert = dy != 0;
            if spec_axis_vert != bridge_axis_vert {
                return Some("axis_conflict");
            }
            break;
        }
    }

    // 2. Adjacent SIDE tiles flowing INTO this tile (sideload). For a
    //    horizontal bridge, the sides are above/below. For a vertical
    //    bridge, the sides are left/right.
    let side_offsets: &[(i32, i32, EntityDirection)] = if bridge_axis_vert {
        // Vertical bridge: sides are east/west; a sideload comes from
        // (x-1, y) facing East, or (x+1, y) facing West.
        &[(-1, 0, EntityDirection::East), (1, 0, EntityDirection::West)]
    } else {
        // Horizontal bridge: sides are north/south; a sideload comes from
        // (x, y-1) facing South, or (x, y+1) facing North.
        &[(0, -1, EntityDirection::South), (0, 1, EntityDirection::North)]
    };
    for &(dx, dy, expected_dir) in side_offsets {
        let side = (tile.0 + dx, tile.1 + dy);
        // 2a. Routed-path sideloads (original check).
        for path in routed_paths.values() {
            for (i, &t) in path.iter().enumerate() {
                if t != side {
                    continue;
                }
                // Compute the spec's direction at this side tile.
                let last_idx = path.len() - 1;
                let (sdx, sdy) = if i < last_idx {
                    (path[i + 1].0 - t.0, path[i + 1].1 - t.1)
                } else if i > 0 {
                    (t.0 - path[i - 1].0, t.1 - path[i - 1].1)
                } else {
                    continue;
                };
                let spec_dir = step_direction(sdx, sdy);
                if spec_dir == expected_dir {
                    return Some("sideload");
                }
                break;
            }
        }
        // 2b. Pre-routing splitters (Step 2-3) dropping items into the
        //     UG endpoint tile from a perpendicular direction. Splitters
        //     never enter `routed_paths` so 2a misses them. Splitters are
        //     two tiles wide perpendicular to their facing direction, so
        //     the `side` tile may be the right half of a splitter placed
        //     one column west.
        for ent in placed_entities {
            let is_splitter = matches!(
                ent.name.as_str(),
                "splitter" | "fast-splitter" | "express-splitter"
            );
            if !is_splitter {
                continue;
            }
            if ent.direction != expected_dir {
                continue;
            }
            let second = match ent.direction {
                EntityDirection::North | EntityDirection::South => (ent.x + 1, ent.y),
                EntityDirection::East | EntityDirection::West => (ent.x, ent.y + 1),
            };
            if (ent.x, ent.y) == side || second == side {
                return Some("splitter_sideload");
            }
        }
    }

    None
}

/// Solve a perpendicular crossing with a deterministic template.
///
/// One path stays on the surface, the other goes underground via a UG pair.
/// Prefers bridging the vertical path so horizontal connections to row inputs
/// stay on the surface.
fn solve_perpendicular_template(
    info: &CrossingInfo,
    hard_obstacles: &FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &FxHashSet<(i32, i32)>,
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    placed_entities: &[PlacedEntity],
) -> Option<(Vec<PlacedEntity>, ClusterZone)> {
    let perpendicular = is_perpendicular(info.spec_a.1, info.spec_b.1);
    if !perpendicular {
        // Same-direction crossings — single attempt, bridge spec_b arbitrarily.
        return try_bridge(
            info.tile,
            (&info.spec_a.0, info.spec_a.1, info.belt_a),
            (&info.spec_b.0, info.spec_b.1, info.belt_b),
            hard_obstacles,
            unreleasable_obstacles,
            routed_paths,
            placed_entities,
        );
    }

    // Perpendicular: try BOTH bridge directions and pick the first that works.
    // Prefer bridging the vertical first (keeps horizontal connections on the
    // surface for row inputs), but fall back to bridging the horizontal if
    // the vertical option is blocked or has UG-position turn conflicts.
    let (h_spec, v_spec) = if is_horizontal(info.spec_a.1) {
        (&info.spec_a, &info.spec_b)
    } else {
        (&info.spec_b, &info.spec_a)
    };

    let bridge_vertical_first = try_bridge(
        info.tile,
        (&h_spec.0, h_spec.1, if std::ptr::eq(h_spec, &info.spec_a) { info.belt_a } else { info.belt_b }),
        (&v_spec.0, v_spec.1, if std::ptr::eq(v_spec, &info.spec_a) { info.belt_a } else { info.belt_b }),
        hard_obstacles,
        unreleasable_obstacles,
        routed_paths,
        placed_entities,
    );
    if bridge_vertical_first.is_some() {
        return bridge_vertical_first;
    }

    // Fall back to bridging the horizontal (vertical stays on surface).
    try_bridge(
        info.tile,
        (&v_spec.0, v_spec.1, if std::ptr::eq(v_spec, &info.spec_a) { info.belt_a } else { info.belt_b }),
        (&h_spec.0, h_spec.1, if std::ptr::eq(h_spec, &info.spec_a) { info.belt_a } else { info.belt_b }),
        hard_obstacles,
        unreleasable_obstacles,
        routed_paths,
        placed_entities,
    )
}

/// Try to place a UG bridge for the second `(item, dir, belt)` triple over
/// the first one staying on the surface at `crossing`. Returns `None` if the
/// UG positions are obstructed or would receive a sideload.
fn try_bridge(
    crossing: (i32, i32),
    surface: (&String, EntityDirection, &'static str),
    bridge: (&String, EntityDirection, &'static str),
    hard_obstacles: &FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &FxHashSet<(i32, i32)>,
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    placed_entities: &[PlacedEntity],
) -> Option<(Vec<PlacedEntity>, ClusterZone)> {
    let (cx, cy) = crossing;
    let (surface_item, surface_dir, surface_belt) = surface;
    let (bridge_item, bridge_dir, bridge_belt) = bridge;

    let (dx, dy) = match bridge_dir {
        EntityDirection::North => (0, -1),
        EntityDirection::South => (0, 1),
        EntityDirection::East => (1, 0),
        EntityDirection::West => (-1, 0),
    };
    let ug_in = (cx - dx, cy - dy);
    let ug_out = (cx + dx, cy + dy);

    let bridge_axis_label: &'static str = match bridge_dir {
        EntityDirection::North | EntityDirection::South => "vertical",
        EntityDirection::East | EntityDirection::West => "horizontal",
    };
    let reject = |reason: &'static str| -> Option<(Vec<PlacedEntity>, ClusterZone)> {
        trace::emit(trace::TraceEvent::JunctionTemplateRejected {
            tile_x: cx,
            tile_y: cy,
            bridge_dir: bridge_axis_label.to_string(),
            reason: reason.to_string(),
        });
        None
    };

    if hard_obstacles.contains(&ug_in) {
        return reject("hard_obstacle_ug_in");
    }
    // `release_for_pertile_template` clears trunks/tapoffs inside the 3-tile
    // footprint, so those don't block us. But it leaves Permanent claims with
    // any other segment id — balancer belts, corridor-perp re-adds, row
    // templates, prior stamped templates — alone. Stamping a UG endpoint OR
    // the crossing-tile surface belt on top of one of those panics in
    // `place`. Reject here so the growth loop in `solve_crossing` moves on
    // to SatStrategy at the next iteration.
    if unreleasable_obstacles.contains(&ug_in) {
        return reject("unreleasable_obstacle_ug_in");
    }
    if unreleasable_obstacles.contains(&ug_out) {
        return reject("unreleasable_obstacle_ug_out");
    }
    if unreleasable_obstacles.contains(&(cx, cy)) {
        return reject("unreleasable_obstacle_crossing");
    }
    if hard_obstacles.contains(&ug_out) {
        return reject("hard_obstacle_ug_out");
    }

    // Reject if any spec turns at the UG-in/out tile, or if a perpendicular
    // belt would sideload into them. Sideloads onto UG belts only fill the
    // far lane and dump items.
    if any_spec_turns_at(ug_in, routed_paths) {
        return reject("turn_at_ug_in");
    }
    if any_spec_turns_at(ug_out, routed_paths) {
        return reject("turn_at_ug_out");
    }
    // Find a representative key for the bridged spec to exclude from the
    // conflict check. Any path containing the crossing tile in the bridge
    // direction at that tile counts as the bridged spec.
    let bridge_key = routed_paths
        .iter()
        .find(|(_, path)| path.contains(&crossing))
        .map(|(k, _)| k.as_str())
        .unwrap_or("");
    if let Some(sub) = ug_endpoint_conflicts(ug_in, bridge_dir, bridge_key, routed_paths, placed_entities) {
        return reject(match sub {
            "axis_conflict" => "ug_in_axis_conflict",
            "sideload" => "ug_in_sideload",
            _ => "ug_in_conflict",
        });
    }
    if let Some(sub) = ug_endpoint_conflicts(ug_out, bridge_dir, bridge_key, routed_paths, placed_entities) {
        return reject(match sub {
            "axis_conflict" => "ug_out_axis_conflict",
            "sideload" => "ug_out_sideload",
            _ => "ug_out_conflict",
        });
    }

    let ug_name = ug_for_belt(bridge_belt);
    let seg = Some(format!("junction:{}:{},{}", bridge_item, cx, cy));

    let entities = vec![
        PlacedEntity {
            name: surface_belt.to_string(),
            x: cx,
            y: cy,
            direction: surface_dir,
            carries: Some(surface_item.clone()),
            segment_id: seg.clone(),
            ..Default::default()
        },
        PlacedEntity {
            name: ug_name.to_string(),
            x: ug_in.0,
            y: ug_in.1,
            direction: bridge_dir,
            io_type: Some("input".to_string()),
            carries: Some(bridge_item.clone()),
            segment_id: seg.clone(),
            ..Default::default()
        },
        PlacedEntity {
            name: ug_name.to_string(),
            x: ug_out.0,
            y: ug_out.1,
            direction: bridge_dir,
            io_type: Some("output".to_string()),
            carries: Some(bridge_item.clone()),
            segment_id: seg.clone(),
            ..Default::default()
        },
    ];

    let zone = ClusterZone {
        x: cx.min(ug_in.0).min(ug_out.0),
        y: cy.min(ug_in.1).min(ug_out.1),
        w: ((cx - ug_in.0).abs().max((cx - ug_out.0).abs()) * 2 + 1) as u32,
        h: ((cy - ug_in.1).abs().max((cy - ug_out.1).abs()) * 2 + 1) as u32,
    };

    Some((entities, zone))
}

/// Belt entity name for a junction-solver `BeltTier`. Matches the
/// surface belt names that `BeltSpec::belt_name` holds.
fn belt_name_for_tier(tier: BeltTier) -> &'static str {
    match tier {
        BeltTier::Yellow => "transport-belt",
        BeltTier::Red => "fast-transport-belt",
        BeltTier::Blue => "express-transport-belt",
    }
}

/// The first (and currently only) `JunctionStrategy` wired into the
/// growth loop: a thin wrapper around the existing
/// `solve_perpendicular_template`. Only activates when the junction
/// has exactly two specs with perpendicular directions at the initial
/// crossing tile. Ignores region growth entirely — the underlying
/// template operates on a fixed 3-tile footprint. Real growth-aware
/// strategies will land alongside this one.
pub(crate) struct PerpendicularTemplateStrategy;

/// Construct a boxed `PerpendicularTemplateStrategy`. Exposed for the
/// fixture-replay helper in `crate::fixture` so it can build the same
/// strategy slice production uses without this type becoming part of the
/// crate's public surface.
pub(crate) fn perpendicular_template_strategy(
) -> Box<dyn crate::bus::junction_solver::JunctionStrategy> {
    Box::new(PerpendicularTemplateStrategy)
}

impl JunctionStrategy for PerpendicularTemplateStrategy {
    fn name(&self) -> &'static str {
        "perpendicular_template"
    }

    fn try_solve(&self, ctx: &JunctionStrategyContext) -> Option<JunctionSolution> {
        // The underlying per-tile template operates on a fixed 3-tile
        // footprint around the original crossing and has no concept of
        // a grown region. If it fails on the initial 1×1 region, it
        // will fail the same way on every grown iteration — skip
        // subsequent attempts to avoid noisy duplicate trace events.
        if ctx.region.tile_count() > 1 {
            return None;
        }
        if ctx.junction.specs.len() != 2 {
            return None;
        }
        let sa = &ctx.junction.specs[0];
        let sb = &ctx.junction.specs[1];
        let da = sa.entry.direction;
        let db = sb.entry.direction;
        if !is_perpendicular(da, db) {
            return None;
        }
        let info = CrossingInfo {
            tile: ctx.region.initial_tile,
            spec_a: (sa.item.clone(), da),
            spec_b: (sb.item.clone(), db),
            belt_a: belt_name_for_tier(sa.belt_tier),
            belt_b: belt_name_for_tier(sb.belt_tier),
        };
        let (entities, zone) =
            solve_perpendicular_template(
                &info,
                ctx.hard_obstacles,
                ctx.unreleasable_obstacles,
                ctx.routed_paths,
                ctx.placed_entities,
            )?;
        Some(JunctionSolution {
            entities,
            footprint: Rect {
                x: zone.x,
                y: zone.y,
                w: zone.w,
                h: zone.h,
            },
            strategy_name: self.name(),
            participating: ctx.region.participating.clone(),
            sat_zone: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Region-solver fixture capture (debug-only)
// ---------------------------------------------------------------------------
//
// Writes a `RegionFixture` JSON for each `solve_crossing` call when the
// `FUCKTORIO_DUMP_REGION_FIXTURE` env var names a directory. Optional
// `FUCKTORIO_DUMP_REGION_FIXTURE_SEED="x,y"` restricts capture to
// clusters containing that seed. Off by default; the probe returns
// immediately when the env var is unset.
//
// The captured JSON has `expected.mode = "solve"` as a placeholder —
// the dev promoting a capture to a committed fixture sets the correct
// mode and `max_cost` by hand, matching the sat-fixture workflow.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn dump_region_fixture(
    seeds: &[(i32, i32)],
    initial_specs: &[&str],
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    hard_obstacles: &FxHashSet<(i32, i32)>,
    strict_obstacles: &FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &FxHashSet<(i32, i32)>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
    spec_items: &FxHashMap<String, String>,
    spec_exit_dirs: &FxHashMap<String, EntityDirection>,
    placed_entities: &[PlacedEntity],
    pending_crossings: &FxHashSet<(i32, i32)>,
) {
    let Ok(dir) = std::env::var("FUCKTORIO_DUMP_REGION_FIXTURE") else {
        return;
    };
    if dir.is_empty() {
        return;
    }

    // Optional seed filter: only dump clusters whose seeds include the
    // requested tile. Useful when a layout produces dozens of clusters
    // and you want just one.
    if let Ok(seed_str) = std::env::var("FUCKTORIO_DUMP_REGION_FIXTURE_SEED") {
        let parts: Vec<&str> = seed_str.split(',').collect();
        if parts.len() != 2 {
            eprintln!(
                "FUCKTORIO_DUMP_REGION_FIXTURE_SEED: expected \"x,y\", got {:?}",
                seed_str
            );
            return;
        }
        let Ok(sx) = parts[0].trim().parse::<i32>() else { return; };
        let Ok(sy) = parts[1].trim().parse::<i32>() else { return; };
        if !seeds.contains(&(sx, sy)) {
            return;
        }
    }

    // Everything captured in the fixture is filtered to a generous
    // radius around the cluster's tiles. Far-away obstacles / belts /
    // paths can't influence solve_crossing's outcome (it only reads
    // tiles inside its growing region + perimeter); dumping them would
    // bloat fixtures by orders of magnitude with no test value.
    //
    // Radius 20 is wider than any realistic growth bbox (cap is 64
    // tiles area, e.g. 8×8) so participating spec paths and their UG
    // pair mates stay in the shadow view.
    const RADIUS: i32 = 20;
    let (min_sx, min_sy, max_sx, max_sy) = seeds.iter().fold(
        (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
        |(lx, ly, hx, hy), &(x, y)| (lx.min(x), ly.min(y), hx.max(x), hy.max(y)),
    );
    let in_radius =
        |x: i32, y: i32| -> bool {
            x >= min_sx - RADIUS
                && x <= max_sx + RADIUS
                && y >= min_sy - RADIUS
                && y <= max_sy + RADIUS
        };

    // Keep routed_paths whose tile sequence touches the radius window —
    // those are the specs the region solver might interact with. Keeping
    // the full path (not just the in-radius portion) so frontier tracking
    // in solve_crossing sees the same sequence it would in production.
    let kept_keys: std::collections::BTreeSet<String> = routed_paths
        .iter()
        .filter(|(_, path)| path.iter().any(|&(x, y)| in_radius(x, y)))
        .map(|(k, _)| k.clone())
        .collect();

    let filter_xy = |v: &FxHashSet<(i32, i32)>| -> Vec<(i32, i32)> {
        let mut out: Vec<(i32, i32)> = v
            .iter()
            .filter(|&&(x, y)| in_radius(x, y))
            .copied()
            .collect();
        out.sort_unstable_by_key(|&(x, y)| (y, x));
        out
    };

    use std::collections::BTreeMap;
    let fixture = crate::fixture::RegionFixture {
        version: 1,
        name: format!("seed_{}_{}", seeds[0].0, seeds[0].1),
        notes: String::from(
            "Captured via FUCKTORIO_DUMP_REGION_FIXTURE. Review expected.mode before committing.",
        ),
        source_url: None,
        seeds: seeds.to_vec(),
        initial_specs: initial_specs.iter().map(|s| s.to_string()).collect(),
        routed_paths: routed_paths
            .iter()
            .filter(|(k, _)| kept_keys.contains(k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<BTreeMap<_, _>>(),
        hard_obstacles: filter_xy(hard_obstacles),
        strict_obstacles: filter_xy(strict_obstacles),
        unreleasable_obstacles: filter_xy(unreleasable_obstacles),
        spec_belt_tiers: spec_belt_tiers
            .iter()
            .filter(|(k, _)| kept_keys.contains(k.as_str()))
            .map(|(k, &v)| (k.clone(), v))
            .collect::<BTreeMap<_, _>>(),
        spec_items: spec_items
            .iter()
            .filter(|(k, _)| kept_keys.contains(k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<BTreeMap<_, _>>(),
        spec_exit_dirs: spec_exit_dirs
            .iter()
            .filter(|(k, _)| kept_keys.contains(k.as_str()))
            .map(|(k, &v)| (k.clone(), v))
            .collect::<BTreeMap<_, _>>(),
        placed_entities: placed_entities
            .iter()
            .filter(|e| in_radius(e.x, e.y))
            .cloned()
            .collect(),
        pending_crossings: filter_xy(pending_crossings),
        expected: crate::fixture::RegionExpected {
            mode: "solve".to_string(),
            max_cost: None,
            optimal_cost: None,
            required_entities: Vec::new(),
        },
    };

    let path = format!("{}/seed_{}_{}.json", dir, seeds[0].0, seeds[0].1);
    match serde_json::to_string_pretty(&fixture) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("dump_region_fixture: failed to write {}: {e}", path);
            } else {
                eprintln!("dump_region_fixture: wrote {}", path);
            }
        }
        Err(e) => {
            eprintln!("dump_region_fixture: serialization failed: {e}");
        }
    }
}

fn is_belt_like(name: &str) -> bool {
    matches!(
        name,
        "transport-belt"
            | "fast-transport-belt"
            | "express-transport-belt"
            | "underground-belt"
            | "fast-underground-belt"
            | "express-underground-belt"
            | "splitter"
            | "fast-splitter"
            | "express-splitter"
    )
}

#[cfg(test)]
mod cluster_adjacent_crossings_tests {
    use super::*;

    fn paths(entries: &[(&str, &[(i32, i32)])]) -> FxHashMap<String, Vec<(i32, i32)>> {
        entries
            .iter()
            .map(|(k, ts)| ((*k).to_string(), ts.to_vec()))
            .collect()
    }

    fn crossings(tiles: &[(i32, i32)]) -> FxHashSet<(i32, i32)> {
        tiles.iter().copied().collect()
    }

    fn no_tiers() -> FxHashMap<String, BeltTier> {
        // All tests assume yellow belts; empty map makes
        // `cluster_adjacent_crossings` fall back to Yellow per spec.
        FxHashMap::default()
    }

    #[test]
    fn empty_input_returns_empty() {
        let cs = FxHashSet::default();
        let rp = FxHashMap::default();
        assert!(cluster_adjacent_crossings(&cs, &rp, &no_tiers()).is_empty());
    }

    #[test]
    fn single_tile_becomes_single_cluster() {
        let cs = crossings(&[(5, 5)]);
        let rp = paths(&[("tap:iron-plate", &[(5, 5)])]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters, vec![vec![(5, 5)]]);
    }

    #[test]
    fn adjacent_sharing_spec_merge() {
        // Five adjacent crossings on row 90, all sharing the same tap
        // (iron-plate row 90) plus individual trunk columns.
        let cs = crossings(&[(13, 90), (14, 90), (15, 90), (16, 90), (17, 90)]);
        let rp = paths(&[
            ("tap:iron-plate", &[
                (13, 90), (14, 90), (15, 90), (16, 90), (17, 90),
            ]),
            ("trunk:copper-cable:13", &[(13, 89), (13, 90), (13, 91)]),
            ("trunk:copper-cable:14", &[(14, 89), (14, 90), (14, 91)]),
            ("trunk:copper-cable:15", &[(15, 89), (15, 90), (15, 91)]),
            ("trunk:copper-cable:16", &[(16, 89), (16, 90), (16, 91)]),
            ("trunk:copper-cable:17", &[(17, 89), (17, 90), (17, 91)]),
        ]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters.len(), 1);
        assert_eq!(
            clusters[0],
            vec![(13, 90), (14, 90), (15, 90), (16, 90), (17, 90)]
        );
    }

    #[test]
    fn adjacent_without_shared_spec_stay_separate() {
        // Two 4-adjacent crossings but with DISJOINT spec sets — must
        // NOT merge. Different two-item crossings that happen to touch.
        let cs = crossings(&[(10, 10), (11, 10)]);
        let rp = paths(&[
            ("tap:a", &[(10, 10)]),
            ("trunk:b", &[(10, 9), (10, 10), (10, 11)]),
            ("tap:c", &[(11, 10)]),
            ("trunk:d", &[(11, 9), (11, 10), (11, 11)]),
        ]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters, vec![vec![(10, 10)], vec![(11, 10)]]);
    }

    #[test]
    fn manhattan_3_sharing_spec_merges_via_ug_reach() {
        // Two crossings 3 tiles apart along a shared spec. Outside
        // Manhattan-2 but inside yellow UG-reach (5), so the UG-reach
        // pass merges them. Guards the failure mode at advanced-circuit
        // (22,143)+(25,142), where the earlier cluster stamps a UG pair
        // whose mate sits in the later cluster's expansion envelope —
        // forcing a walker veto on every SAT output until growth caps.
        let cs = crossings(&[(5, 5), (8, 5)]);
        let rp = paths(&[
            ("tap:iron-plate", &[(5, 5), (6, 5), (7, 5), (8, 5)]),
            ("trunk:a", &[(5, 4), (5, 5), (5, 6)]),
            ("trunk:b", &[(8, 4), (8, 5), (8, 6)]),
        ]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters.len(), 1);
    }

    #[test]
    fn ug_reach_merge_respects_hull_budget() {
        // Long chain of crossings 4 tiles apart along a shared spec.
        // Each consecutive pair is within UG-reach, so naive transitive
        // union-find would fuse them all — but once the running hull
        // would exceed HULL_BUDGET=48 tiles the guardrail must stop
        // merging. 20 crossings × 4 tile spacing → hull 77×1 = 77 tiles,
        // well over the cap, so we must end up with >1 cluster.
        let tiles: Vec<(i32, i32)> = (0..20).map(|k| (k * 4, 5)).collect();
        let cs = crossings(&tiles);
        let path: Vec<(i32, i32)> = (0..=20 * 4).map(|x| (x, 5)).collect();
        let rp = paths(&[("tap:shared", path.as_slice())]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert!(
            clusters.len() > 1,
            "expected hull-budget to split the chain; got {} cluster(s)",
            clusters.len()
        );
    }

    #[test]
    fn manhattan_2_straight_sharing_spec_merges() {
        // Two crossings 2 tiles apart along a shared spec's path. The
        // real-world motivator: at (21,161) in advanced-circuit the cable
        // ret touches crossings at (23,161) and (25,161) with a clean
        // belt between them — these should cluster into one zone.
        let cs = crossings(&[(5, 5), (7, 5)]);
        let rp = paths(&[
            ("ret:shared", &[(5, 5), (6, 5), (7, 5)]),
            ("trunk:a", &[(5, 4), (5, 5), (5, 6)]),
            ("trunk:b", &[(7, 4), (7, 5), (7, 6)]),
        ]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0], vec![(5, 5), (7, 5)]);
    }

    #[test]
    fn diagonal_adjacency_sharing_spec_merges() {
        // Manhattan-2 diagonals also merge when they share a spec.
        // Matches the (23,161)/(21,163) vertical neighbourhood that
        // motivated relaxing from strict orthogonal adjacency.
        let cs = crossings(&[(5, 5), (6, 6)]);
        let rp = paths(&[
            ("tap:x", &[(5, 5), (6, 6)]),
            ("trunk:a", &[(5, 4), (5, 5), (5, 6)]),
            ("trunk:b", &[(6, 5), (6, 6), (6, 7)]),
        ]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters.len(), 1);
    }

    #[test]
    fn manhattan_2_without_shared_spec_stays_separate() {
        // Bounded relaxation: Manhattan 2 *without* a shared spec must
        // still not merge — the shared-spec gate is the safety net.
        let cs = crossings(&[(5, 5), (7, 5)]);
        let rp = paths(&[
            ("tap:a", &[(5, 5)]),
            ("tap:b", &[(7, 5)]),
            ("trunk:x", &[(5, 4), (5, 5), (5, 6)]),
            ("trunk:y", &[(7, 4), (7, 5), (7, 6)]),
        ]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers());
        assert_eq!(clusters.len(), 2);
    }
}
