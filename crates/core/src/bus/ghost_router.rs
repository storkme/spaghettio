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
use crate::bus::balancer::{balancer_origin_x, splitter_for_belt, stamp_family_balancer, underground_for_belt};
use crate::bus::lane_planner::{BusLane, LaneFamily, MACHINE_ENTITIES};
use crate::bus::output_merger::merge_output_rows;
use crate::bus::trunk_renderer::{is_intermediate, render_path, trunk_segments};
use crate::bus::junction::{BeltTier, Rect};
use crate::bus::eviction::EvictionStrategy;
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
    /// Module the spec's lane belongs to. Under `LayoutStrategy::Pooled`
    /// always 0; under `PartitionedDecomposed` a
    /// single item can have multiple sibling families with the same item
    /// name but different module ids. The crossing-detection filter
    /// (`ghost_item_at`) keys on `(item, module_id)` so same-item-
    /// different-module paths get bridged via UG instead of silently
    /// merging — see comment at the filter site for the symptom this
    /// guards against.
    module_id: u32,
    belt_name: &'static str,
    /// Explicit exit direction for the final belt on this path. Set when the
    /// planner knows the spec's topology — producer-row orientation, trunk
    /// axis — at emission time. When `Some`, render_path uses this for
    /// single-tile paths and the last tile of multi-tile paths instead of
    /// inferring direction from start/goal coordinate comparisons (which is
    /// ambiguous for degenerate start == goal specs and length-1 blocked A*
    /// fallbacks). See plan file abundant-gliding-turing.md for context.
    exit_dir: Option<EntityDirection>,
    /// X-column of the lane's own south-flowing trunk, when the spec
    /// originates from a tap-off or producer-return on a specific lane.
    /// A* hard-blocks `(lane_trunk_col, y)` for every `y` during this
    /// spec's routing call, so the path cannot detour through its own
    /// trunk tiles — flowing items north against the trunk's south-only
    /// flow doesn't physically work, and the surviving-tile filter that
    /// strips the overlap leaves a fragmented path with a head-on belt-
    /// junction at the start tile (PU@3/s ore-red copper-cable @ x=1,
    /// y=292 was the motivating bug). `None` for specs that don't have
    /// a single owning trunk (feeders into a balancer input).
    lane_trunk_col: Option<i32>,
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
    pole_entities: &[PlacedEntity],
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
    // Poles are placed before ghost routing but otherwise live outside the
    // row_entities flow. Inject their 1×1 tiles into the hard set so SAT,
    // ghost A*, and the junction solver all treat them as obstacles. Without
    // this, SAT can stamp belts/UGs on top of a pole when it solves a
    // junction whose grown bbox happens to overlap a pole tile.
    for e in pole_entities {
        hard.insert((e.x, e.y));
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

        let lane_start = entities.len();
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

        // Stream the per-lane tap-off batch so the live renderer can reveal
        // it progressively instead of dumping it via the bus_routed safety net.
        if entities.len() > lane_start {
            crate::trace::emit(crate::trace::TraceEvent::TrunkBeltCommitted {
                item: lane.item.clone(),
                lane_x: lane.x,
                is_fluid: false,
                entities: entities[lane_start..].to_vec(),
            });
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
        // Stream sibling of BalancerStamped — carries the entity batch so
        // the live renderer can reveal the cascade progressively. Emitted
        // before extend() so the event clones the entity list once and the
        // extend consumes the original.
        if !balancer_ents.is_empty() {
            crate::trace::emit(crate::trace::TraceEvent::BalancerCommitted {
                item: fam.item.clone(),
                shape: fam.shape,
                entities: balancer_ents.clone(),
            });
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
    // Tile → `(item, module_id)`. Module id is needed alongside item so the
    // crossing filter at the materialisation step can distinguish same-item
    // sibling families (under Phase 2 the same item can have multiple
    // independent flows that must be physically separated via UG bridges).
    let mut trunk_tile_items: FxHashMap<(i32, i32), (String, u32)> = FxHashMap::default();
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
        let lane_start = entities.len();
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
                trunk_tile_items.insert(tile, (lane.item.clone(), lane.module_id));
                trunk_synth_paths
                    .entry(format!("trunk:{}:{}", lane.item, x))
                    .or_default()
                    .push(tile);
            }
        }
        // Stream the per-lane trunk-segment batch.
        if entities.len() > lane_start {
            crate::trace::emit(crate::trace::TraceEvent::TrunkBeltCommitted {
                item: lane.item.clone(),
                lane_x: lane.x,
                is_fluid: false,
                entities: entities[lane_start..].to_vec(),
            });
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

    // Per-lane tap_ys (consumer + producer port rows). Used by the gap-fill
    // UG endpoint picker for cross-lane "foreign tap" avoidance: a UG mouth
    // landing on a foreign tap row would surface-merge with that lane's
    // branch tile (cross-fluid contamination), so we slide endpoints off
    // foreign tap rows when possible.
    let mut lane_tap_ys: Vec<FxHashSet<i32>> = Vec::with_capacity(lanes.len());
    for lane in lanes {
        let mut taps = FxHashSet::default();
        if lane.is_fluid {
            for &(_ri, _px, py) in &lane.fluid_port_positions {
                taps.insert(py);
            }
            for &(_ri, _px, py) in &lane.fluid_output_port_positions {
                taps.insert(py);
            }
        }
        lane_tap_ys.push(taps);
    }

    for (lane_idx, lane) in lanes.iter().enumerate() {
        if !lane.is_fluid {
            continue;
        }
        let lane_start = entities.len();
        let x = lane.x;
        let trunk_seg_id = Some(format!("trunk:{}", lane.item));

        // Foreign taps: union of all OTHER fluid lanes' tap_ys. We avoid
        // landing UG mouths on these rows where possible (cross-fluid
        // surface merge).
        let mut foreign_tap_ys: FxHashSet<i32> = FxHashSet::default();
        for (other_idx, other_taps) in lane_tap_ys.iter().enumerate() {
            if other_idx == lane_idx {
                continue;
            }
            foreign_tap_ys.extend(other_taps.iter().copied());
        }

        let mut end_y = lane.source_y;
        for &(_ri, _px, py) in &lane.fluid_port_positions {
            end_y = end_y.max(py);
        }
        for &(_ri, _px, py) in &lane.fluid_output_port_positions {
            end_y = end_y.max(py);
        }
        let start_y = lane.source_y;

        // Fluid trunks surface only at functional anchor rows: start_y
        // (top entry, where the user attaches an external feed), every
        // tap_y (consumer/producer port row, where a horizontal branch
        // peels off east), and end_y (bottom). Between anchors, the trunk
        // runs underground via a chain of back-to-back pipe-to-ground
        // pairs (F5b: adjacent PTGs with surface mouths facing each other
        // merge into one fluid network across their shared edge — no
        // surface pipe is needed between consecutive tunnel segments).
        //
        // Adjacent fluid lanes stay isolated by F5a: a UG's perpendicular
        // (east/west) faces have no surface connection, so a parallel
        // fluid trunk on the next column merges nothing. The only place
        // surface pipes coexist between adjacent lanes is at tap rows,
        // where multi-fluid row templates already stagger tap_ys so only
        // one lane surfaces per row.
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

        let reservations = &fluid_reservations;
        let is_blocked = |y: i32, existing_belts: &FxHashSet<(i32, i32)>, hard: &FxHashSet<(i32, i32)>| -> bool {
            let tile = (x, y);
            existing_belts.contains(&tile)
                || (hard.contains(&tile) && !reservations.contains(&tile))
        };

        // The entry tile (anchors[0]) gets a UG-S input when the gap to
        // the next anchor is ≥ 2 and the entry is not itself a tap row.
        // The UG-S input's perpendicular faces are closed (F5a),
        // protecting against adjacent fluid lanes' surface pipes
        // cross-merging at the entry row. When anchors[0] is a tap row,
        // we must emit a surface pipe — the horizontal branch needs an
        // east-facing fluid box, which a UG-S input does not have on its
        // perpendicular east face. The gap-fill chain then emits a UG-S
        // input one tile south automatically.
        let tap_set: FxHashSet<i32> = tap_ys.iter().copied().collect();
        let entry_is_ug = anchors.len() >= 2
            && anchors[1] - anchors[0] >= 2
            && !tap_set.contains(&anchors[0]);

        for (idx, &sp) in anchors.iter().enumerate() {
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

        // Picking a UG endpoint y: prefer ys that are unblocked AND both
        // the endpoint tile and its mouth tile are not in a foreign tap
        // row. The mouth tile is at y-1 for UG-S input (mouth NORTH per
        // F5) and y+1 for UG-N output (mouth SOUTH). If no such y exists
        // in the candidate range, fall back to the first unblocked y —
        // the validator will surface any resulting cross-fluid merge.
        let pick_endpoint_y = |
            range: &[i32],
            mouth_offset: i32,
            existing_belts: &FxHashSet<(i32, i32)>,
            hard: &FxHashSet<(i32, i32)>,
        | -> Option<i32> {
            let mut fallback: Option<i32> = None;
            for &cand in range {
                if is_blocked(cand, existing_belts, hard) {
                    continue;
                }
                let endpoint_clear = !foreign_tap_ys.contains(&cand);
                let mouth_clear = !foreign_tap_ys.contains(&(cand + mouth_offset));
                if endpoint_clear && mouth_clear {
                    return Some(cand);
                }
                if fallback.is_none() {
                    fallback = Some(cand);
                }
            }
            fallback
        };

        // Fill the gap between each consecutive anchor pair with a chain
        // of back-to-back pipe-to-ground pairs.
        //
        // For gap == 2, a single surface pipe at y0+1 (or the partner
        // UG-N output, when y0 is the entry UG-S input) closes the gap.
        //
        // For gap ≥ 3, chain UG pairs. The first UG-S input lands at
        // y0+1 (or y0 itself when entry_is_ug — the entry UG was already
        // emitted as the surface anchor). Each UG-N output lands at
        // min(in_y + FLUID_UG_MAX_DISTANCE, y1-1). If the UG-N output
        // doesn't reach y1-1, the next UG-S input lands at out_y+1
        // (back-to-back, F5b merge across the shared mouth edge). When
        // the last UG-N output lands at exactly y1-1, its mouth at y1
        // merges (F5) with the surface anchor at y1, closing the chain.
        for (idx, pair) in anchors.windows(2).enumerate() {
            let (y0, y1) = (pair[0], pair[1]);
            let gap = y1 - y0;
            if gap <= 1 {
                continue;
            }
            let first_pair = idx == 0;

            if gap == 2 {
                if first_pair && entry_is_ug {
                    // UG-S input already at y0; partner UG-N output at
                    // y0+1 completes the pair (tunnel distance 1, valid).
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
                    } else {
                        crate::trace::emit(crate::trace::TraceEvent::FluidTrunkBreak {
                            item: lane.item.clone(),
                            trunk_x: x,
                            y_start: y0,
                            y_end: y1,
                            reason: format!(
                                "gap=2 entry UG-out tile ({x},{ug_out_y}) blocked"
                            ),
                        });
                    }
                } else {
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
                    } else {
                        crate::trace::emit(crate::trace::TraceEvent::FluidTrunkBreak {
                            item: lane.item.clone(),
                            trunk_x: x,
                            y_start: y0,
                            y_end: y1,
                            reason: format!(
                                "gap=2 surface tile ({x},{mid}) blocked"
                            ),
                        });
                    }
                }
                continue;
            }

            // gap ≥ 3. Chain back-to-back UG pairs from y0 toward y1.
            let mut head_in_y: Option<i32> = if first_pair && entry_is_ug {
                // Entry UG-S input was already emitted at y0; chain
                // starts there.
                Some(y0)
            } else {
                // First UG-S input at y0+1 (mouth at y0 merges F5 with
                // the surface anchor). Slide forward if blocked or if its
                // mouth would land on a foreign tap row.
                let cand: Vec<i32> = ((y0 + 1)..=(y1 - 1)).collect();
                match pick_endpoint_y(&cand, -1, &existing_belts, &hard) {
                    Some(in_y) => {
                        // When the UG-S slides past y0+1, the tiles between
                        // the anchor at y0 and the UG-S mouth at (in_y-1)
                        // are unowned — without filling them the chain
                        // breaks and the surface anchor connects to nothing.
                        // Fill with surface pipes (same fluid as the anchor;
                        // adjacent columns are typically clear because the
                        // mouth_clear heuristic only slides past *foreign-
                        // tap* rows, where foreign trunks branch east via
                        // perpendicular UGs with no surface fluid).
                        for fill_y in (y0 + 1)..in_y {
                            if is_blocked(fill_y, &existing_belts, &hard) {
                                crate::trace::emit(crate::trace::TraceEvent::FluidTrunkBreak {
                                    item: lane.item.clone(),
                                    trunk_x: x,
                                    y_start: y0,
                                    y_end: y1,
                                    reason: format!(
                                        "anchor-to-UG-S bridge tile ({x},{fill_y}) blocked"
                                    ),
                                });
                                continue;
                            }
                            entities.push(PlacedEntity {
                                name: "pipe".to_string(),
                                x,
                                y: fill_y,
                                carries: Some(lane.item.clone()),
                                segment_id: trunk_seg_id.clone(),
                                ..Default::default()
                            });
                            existing_belts.insert((x, fill_y));
                            hard.insert((x, fill_y));
                        }
                        entities.push(PlacedEntity {
                            name: "pipe-to-ground".to_string(),
                            x,
                            y: in_y,
                            direction: EntityDirection::South,
                            io_type: Some("input".to_string()),
                            carries: Some(lane.item.clone()),
                            segment_id: trunk_seg_id.clone(),
                            ..Default::default()
                        });
                        existing_belts.insert((x, in_y));
                        hard.insert((x, in_y));
                        Some(in_y)
                    }
                    None => {
                        crate::trace::emit(crate::trace::TraceEvent::FluidTrunkBreak {
                            item: lane.item.clone(),
                            trunk_x: x,
                            y_start: y0,
                            y_end: y1,
                            reason: "no unblocked UG-in position in gap".to_string(),
                        });
                        None
                    }
                }
            };

            while let Some(in_y) = head_in_y {
                // UG-N output: prefer the farthest reach within the gap.
                // FLUID_UG_MAX_DISTANCE is the inclusive count of tiles
                // between UG-in and UG-out (F4), so the farthest valid
                // UG-out sits at in_y + max_distance + 1. Two cases:
                //   - Max reach lands at or past y1-1: aim for y1-1 so
                //     the chain closes this pair (mouth at y1 merges F5
                //     with the next surface anchor).
                //   - Max reach falls short: cap at y1-3 so the next
                //     pair fits (next UG-S at out_y+1, next UG-N at any
                //     tile through y1-1, minimum distance 0).
                // Capping at y1-3 prevents the greedy chain from
                // landing UG-N at y1-2 (which would leave a single tile
                // for the next UG-S at y1-1 with no room for its
                // partner UG-N).
                let max_reach = in_y + FLUID_UG_MAX_DISTANCE + 1;
                let upper = if max_reach >= y1 - 1 {
                    y1 - 1
                } else {
                    max_reach.min(y1 - 3)
                };
                let cand: Vec<i32> = ((in_y + 1)..=upper).rev().collect();
                match pick_endpoint_y(&cand, 1, &existing_belts, &hard) {
                    Some(out_y) => {
                        entities.push(PlacedEntity {
                            name: "pipe-to-ground".to_string(),
                            x,
                            y: out_y,
                            direction: EntityDirection::North,
                            io_type: Some("output".to_string()),
                            carries: Some(lane.item.clone()),
                            segment_id: trunk_seg_id.clone(),
                            ..Default::default()
                        });
                        existing_belts.insert((x, out_y));
                        hard.insert((x, out_y));

                        if out_y >= y1 - 1 {
                            // Last UG-N's mouth at y1 merges F5 with the
                            // surface anchor; chain done.
                            head_in_y = None;
                        } else {
                            // Chain next UG-S input back-to-back at
                            // out_y+1 (mouth at out_y shares the F5b
                            // edge with this UG-N's mouth).
                            let next_in = out_y + 1;
                            if is_blocked(next_in, &existing_belts, &hard) {
                                crate::trace::emit(crate::trace::TraceEvent::FluidTrunkBreak {
                                    item: lane.item.clone(),
                                    trunk_x: x,
                                    y_start: y0,
                                    y_end: y1,
                                    reason: format!(
                                        "chain UG-in tile ({x},{next_in}) blocked"
                                    ),
                                });
                                head_in_y = None;
                            } else {
                                entities.push(PlacedEntity {
                                    name: "pipe-to-ground".to_string(),
                                    x,
                                    y: next_in,
                                    direction: EntityDirection::South,
                                    io_type: Some("input".to_string()),
                                    carries: Some(lane.item.clone()),
                                    segment_id: trunk_seg_id.clone(),
                                    ..Default::default()
                                });
                                existing_belts.insert((x, next_in));
                                hard.insert((x, next_in));
                                head_in_y = Some(next_in);
                            }
                        }
                    }
                    None => {
                        crate::trace::emit(crate::trace::TraceEvent::FluidTrunkBreak {
                            item: lane.item.clone(),
                            trunk_x: x,
                            y_start: y0,
                            y_end: y1,
                            reason: "no unblocked UG-out position within max reach".to_string(),
                        });
                        head_in_y = None;
                    }
                }
            }
        }
        // Stream the per-lane fluid-trunk batch so the live renderer can
        // reveal trunks progressively. Branches emit separately in pass 2.
        if entities.len() > lane_start {
            crate::trace::emit(crate::trace::TraceEvent::TrunkBeltCommitted {
                item: lane.item.clone(),
                lane_x: lane.x,
                is_fluid: true,
                entities: entities[lane_start..].to_vec(),
            });
        }
    }

    // Step 3.7: Horizontal branches per fluid lane.
    //
    // Emitted in a SECOND pass over fluid lanes so every lane's trunk
    // endpoints are already in `existing_belts` by the time any branch
    // chooses tiles. Without this two-pass split, an early lane's branch
    // could land on a tile a later lane's trunk endpoint will need —
    // forcing the later trunk's UG to slide and the slid UG's mouth to
    // surface-merge with the early branch (cross-fluid contamination).
    //
    // For each (row, port_y) pair, multiple machines share `port_y`; we
    // stamp to the leftmost px and let the row template's own pipe line
    // propagate east.
    //
    // If the template placed a UG direction=West carrying this fluid at
    // (min_px - 1, py), that's the multi-fluid stacked-T left flank ready
    // to be a UG partner. Emit a single east-facing UG at (x+1, py) and
    // leave the intermediate tiles empty — the tunnel carries fluid across
    // without surface pipes that could cross-merge with foreign trunks on
    // adjacent rows. Otherwise fall back to continuous surface pipes.
    for lane in lanes {
        if !lane.is_fluid {
            continue;
        }
        let lane_start = entities.len();
        let x = lane.x;
        let trunk_seg_id = Some(format!("trunk:{}", lane.item));

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

            // A branch tile is blocked iff it holds an actual emitted entity
            // OR it's a non-fluid hard obstacle (machine, pole, splitter
            // stamp). Foreign fluid lanes' column reservations
            // (`fluid_reservations`) cover tunnel-interior tiles — placing
            // a perpendicular pipe there is safe per F5a (no surface merge
            // with the foreign trunk's UG endpoints), and after pass 1 the
            // foreign trunk's actual endpoint tiles are already in
            // `existing_belts` and properly excluded.

            if has_ug_partner && min_px - (x + 1) <= 10 {
                // Multi-fluid template: emit one UG at (x+1, py) direction=East,
                // partnered with the template's left flank UG. Reach cap 10 per F4.
                let ug_tile = (x + 1, py);
                let blocked = existing_belts.contains(&ug_tile)
                    || (hard.contains(&ug_tile) && !fluid_reservations.contains(&ug_tile));
                if !blocked {
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
                    let blocked = existing_belts.contains(&tile)
                        || (hard.contains(&tile) && !fluid_reservations.contains(&tile));
                    if blocked {
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
        // Stream the per-lane fluid-trunk batch.
        if entities.len() > lane_start {
            crate::trace::emit(crate::trace::TraceEvent::TrunkBeltCommitted {
                item: lane.item.clone(),
                lane_x: lane.x,
                is_fluid: true,
                entities: entities[lane_start..].to_vec(),
            });
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
            // Fluid trunks stay pooled (RFP "Fluids" carve-out), so module_id
            // is always 0 here. The `(item, 0)` tuple still distinguishes
            // them from any solid-trunk sibling at the same tile under the
            // crossing filter.
            trunk_tile_items.insert((ent.x, ent.y), (item.clone(), 0));
            // Also inject a synthetic fluid-trunk path so classify_crossing
            // sees the pipe column as a second spec at belt×pipe crossing
            // tiles. Key format mirrors the solid-trunk synth path format
            // (`trunk:{item}:{x}`) — downstream lookups in spec_kinds
            // branch on pipe-vs-belt, not on the key shape.
            trunk_synth_paths
                .entry(format!("trunk:{}:{}", item, ent.x))
                .or_default()
                .push((ent.x, ent.y));
        }
    }
    // Re-sort after the fluid catch-up: solid-lane tiles were already
    // sorted, fluid-lane tiles need to be merged in.
    for path in trunk_synth_paths.values_mut() {
        path.sort_by_key(|&(_, y)| y);
        path.dedup();
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
                    module_id: lane.module_id,
                    belt_name: horiz_belt,
                    exit_dir: Some(EntityDirection::East),
                    lane_trunk_col: Some(x),
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
                    module_id: lane.module_id,
                    belt_name: horiz_belt,
                    exit_dir: Some(EntityDirection::West),
                    lane_trunk_col: Some(x),
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
                    module_id: lane.module_id,
                    belt_name: horiz_belt,
                    exit_dir: Some(EntityDirection::West),
                    lane_trunk_col: Some(x),
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

                    // Collect all input tile absolute x-coords across the
                    // family's stamps. For a direct (n, m) template match
                    // there's a single stamp; under the decomposition
                    // fallback (mirroring `stamp_family_balancer` in
                    // `balancer.rs`) the family is realised as g sibling
                    // sub-stamps each at their own origin, with one set of
                    // input tiles per sub-stamp. Without this fallback
                    // path, decomposed families (e.g. iron-plate (2, 10)
                    // → 2 × (1, 5)) had zero feeders generated and the
                    // producer rows dead-ended.
                    //
                    // The two paths must agree with the stamper on origin
                    // selection (`balancer_origin_x` on the relevant lane
                    // chunk) and on the geometric guard
                    // `sub_template.width <= sub_m` — otherwise feeders
                    // aim at tiles where no balancer was actually stamped.
                    let mut input_xs: Vec<i32> = Vec::new();
                    if crate::bus::balancer::is_passthrough_shape(n, m) {
                        // Passthrough: each producer feeds the matching
                        // output column directly (issue #268). No template
                        // origin offset — input_xs == lane_xs.
                        input_xs = fam.lane_xs.clone();
                    } else if let Some(template) = templates.get(&(n, m)) {
                        let origin_x = if fam.lane_xs.is_empty() {
                            x
                        } else {
                            balancer_origin_x(&fam.lane_xs, template.output_tiles)
                        };
                        let mut rel_inputs: Vec<i32> =
                            template.input_tiles.iter().map(|t| t.0).collect();
                        rel_inputs.sort();
                        for r in rel_inputs {
                            input_xs.push(origin_x + r);
                        }
                    } else {
                        // Decomposition fallback: must match
                        // `stamp_family_balancer`'s search order
                        // (largest g first) and skip-rule
                        // (sub_template.width > sub_m).
                        for g in (1..=n).rev() {
                            if n % g != 0 || m % g != 0 {
                                continue;
                            }
                            let sub_n = n / g;
                            let sub_m = m / g;
                            let Some(sub_template) = templates.get(&(sub_n, sub_m)) else {
                                continue;
                            };
                            if sub_template.width > sub_m {
                                continue;
                            }
                            let lanes_per_group = sub_m as usize;
                            for gi in 0..(g as usize) {
                                let lane_start = gi * lanes_per_group;
                                let lane_end = (lane_start + lanes_per_group)
                                    .min(fam.lane_xs.len());
                                let lane_chunk = &fam.lane_xs[lane_start..lane_end];
                                if lane_chunk.is_empty() {
                                    continue;
                                }
                                let sub_origin_x = balancer_origin_x(
                                    lane_chunk, sub_template.output_tiles,
                                );
                                let mut rel_inputs: Vec<i32> =
                                    sub_template.input_tiles.iter().map(|t| t.0).collect();
                                rel_inputs.sort();
                                for r in rel_inputs {
                                    input_xs.push(sub_origin_x + r);
                                }
                            }
                            break;
                        }
                    }

                    if !input_xs.is_empty() {
                        // Sort across all sub-stamps so the leftmost
                        // producer row maps to the leftmost balancer
                        // input — same convention as the direct-template
                        // path used (`sort_by_key(|t| t.0)`), now applied
                        // across the union of sub-stamp inputs.
                        input_xs.sort();
                        let origin_y = fam.balancer_y_start;
                        let feeder_belt = belt_entity_for_rate(fam.total_rate, max_belt_tier);

                        for (i, &pri) in fam.producer_rows.iter().enumerate() {
                            if pri >= row_spans.len() {
                                continue;
                            }
                            let row = &row_spans[pri];
                            let (start_x, out_y) = row_exit_origin(row);
                            if let Some(&input_x) = input_xs.get(i) {
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
                                    module_id: lane.module_id,
                                    belt_name: feeder_belt,
                                    exit_dir: Some(feeder_exit_dir),
                                    // Feeders walk into a balancer input,
                                    // not down a single trunk column —
                                    // own-trunk hard-blocking doesn't apply.
                                    lane_trunk_col: None,
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
    // Tracks `(item, module_id)` at each ghost-routed tile so the crossing
    // filter can distinguish three cases: (1) same-item-same-module
    // overlaps (not conflicts — the original "two belts converging"
    // semantics), (2) different-item overlaps (real crossings), and (3)
    // same-item-different-module overlaps (also real crossings under
    // Phase 2 — sibling families must stay physically separate). Pre-Phase 2
    // this was a `FxHashMap<_, String>` and case (3) silently merged into
    // case (1), so mod0 taps would disappear into mod1 trunks at every
    // east-tap row pitch.
    let mut ghost_item_at: FxHashMap<(i32, i32), (String, u32)> = FxHashMap::default();

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
            // Augment hard-obstacles with the spec's own trunk column
            // so A* can't detour through its own south-flowing trunk
            // tiles (the (1, 292) head-on bug). The full column gets
            // hard-blocked, including tiles south of `spec.start.y`
            // that aren't physically trunks today — those are simply
            // empty bus tiles and blocking them just confines the path
            // to genuinely-relevant directions. `goal_on_obstacle`
            // semantics still let the path land on the goal even if it
            // sits in a blocked column (no current spec does, but the
            // semantics are stable). For specs without a single owning
            // trunk (feeders), `lane_trunk_col` is `None` and we reuse
            // the shared `astar_hard` directly.
            let hard_with_own_trunk;
            let hard_ref: &FxHashSet<(i32, i32)> = match spec.lane_trunk_col {
                Some(col) => {
                    let mut h = astar_hard.clone();
                    for y in 0..height {
                        if (col, y) == spec.start || (col, y) == spec.goal {
                            continue;
                        }
                        h.insert((col, y));
                    }
                    hard_with_own_trunk = h;
                    &hard_with_own_trunk
                }
                None => &astar_hard,
            };
            match ghost_astar(
                spec.start,
                spec.goal,
                hard_ref,
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
                    // Real crossing iff item OR module_id differs. Same-item-
                    // same-module overlaps remain silent (the original
                    // "two converging belts" case under Pool); same-item-
                    // different-module gets bridged via the junction solver
                    // — without this guard, mod0 east taps merged silently
                    // into mod1 south trunks at every consumer-row pitch
                    // (y=144/152/160/168/176 on EC@30/s decomposed).
                    Some((existing_item, existing_mod)) => {
                        existing_item != &spec.item || *existing_mod != spec.module_id
                    }
                    None => false,
                }
            }));

            for &tile in &path {
                ghost_item_at
                    .entry(tile)
                    .or_insert_with(|| (spec.item.clone(), spec.module_id));
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
    // Parallel to spec_items / spec_belt_tiers. Distinguishes belt specs
    // (default) from fluid-trunk synth paths so downstream templates can
    // branch: a belt×pipe crossing must UG-bridge the belt without
    // stamping anything on the pipe tile. See RFP `docs/rfp-pipe-belt-junctions.md`.
    let mut spec_kinds: FxHashMap<String, crate::bus::junction::SpecKind> = specs
        .iter()
        .map(|s| (s.key.clone(), crate::bus::junction::SpecKind::Belt))
        .collect();
    // Extend with synthetic trunk entries so classify_crossing can resolve
    // item name and belt tier for trunk keys found in routed_paths. Keys
    // match the per-column format used when populating `trunk_synth_paths`
    // ("trunk:{item}:{x}") so multi-lane items register distinct keys per
    // column. Solid trunks get Belt kind, fluid trunks get Pipe.
    for lane in lanes {
        let key = format!("trunk:{}:{}", lane.item, lane.x);
        spec_items.insert(key.clone(), lane.item.clone());
        if lane.is_fluid {
            // Fluid trunks: tier is irrelevant (pipes don't have a tier),
            // but the map needs an entry so classify_crossing doesn't
            // bail with `continue`. Yellow is a harmless default.
            spec_belt_tiers.insert(key.clone(), BeltTier::Yellow);
            spec_kinds.insert(key, crate::bus::junction::SpecKind::Pipe);
        } else {
            spec_belt_tiers.insert(
                key.clone(),
                BeltTier::from_name(belt_entity_for_rate(lane.rate * 2.0, max_belt_tier))
                    .unwrap_or(BeltTier::Yellow),
            );
            spec_kinds.insert(key, crate::bus::junction::SpecKind::Belt);
        }
    }
    // Fluid catch-up: `trunk_synth_paths` may contain keys for adjacent
    // pipe columns (e.g. `trunk:petroleum-gas:20` next to a primary lane
    // at column 19) that the per-lane loop above never names. Without
    // explicit spec_kinds entries these default to `Belt`, slipping
    // through every pipe filter (cluster construction, expand_bbox
    // promotion, walker veto exclusion). Sweep every synth-path key
    // whose path tiles all sit on pipe / pipe-to-ground entities and
    // tag them as Pipe + Yellow tier + their item.
    let pipe_tile_set: FxHashSet<(i32, i32)> = entities
        .iter()
        .filter(|e| matches!(e.name.as_str(), "pipe" | "pipe-to-ground"))
        .map(|e| (e.x, e.y))
        .collect();
    for (key, path) in &trunk_synth_paths {
        if spec_kinds.contains_key(key) {
            continue;
        }
        let all_pipe = !path.is_empty()
            && path.iter().all(|t| pipe_tile_set.contains(t));
        if !all_pipe {
            continue;
        }
        // Recover the item name from the key: "trunk:{item}:{x}".
        let item = key
            .strip_prefix("trunk:")
            .and_then(|rest| rest.rsplit_once(':').map(|(item, _x)| item.to_string()))
            .unwrap_or_default();
        if item.is_empty() {
            continue;
        }
        spec_items.insert(key.clone(), item);
        spec_belt_tiers.insert(key.clone(), BeltTier::Yellow);
        spec_kinds.insert(key.clone(), crate::bus::junction::SpecKind::Pipe);
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

        spec_kinds.remove(&trunk_key);
        spec_kinds.remove(&last_tap_key);
        spec_kinds.insert(unified_key.clone(), crate::bus::junction::SpecKind::Belt);

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
    // Filter the wide `hard` set the same way `occupancy_hard` was filtered:
    // drop empty fluid-reservation tiles (those between a PTG-in and PTG-out
    // on a fluid trunk — physically empty, belts can cross over per F7) but
    // keep pipe tiles (which the junction solver must UG-over). Without this,
    // `refresh_forbidden`'s `hard_obstacles || strict_obstacles` OR
    // re-introduces empty-reservation tiles that Occupancy already excluded,
    // and SAT mistakenly treats them as forbidden. The matching defence
    // against the old "encountered flow loses its bridge when sat-1ug-native
    // becomes satisfiable" failure mode lives in
    // `GrowingRegion::promote_blocked_encountered`, which forces SAT to
    // model encountered flows as participating when their path crosses a
    // forbidden interior tile.
    let pipe_tile_set: FxHashSet<(i32, i32)> = row_entities
        .iter()
        .chain(entities.iter())
        .filter(|e| matches!(e.name.as_str(), "pipe" | "pipe-to-ground"))
        .map(|e| (e.x, e.y))
        .collect();
    let hard_for_junction: FxHashSet<(i32, i32)> = hard
        .iter()
        .filter(|t| !fluid_reservations.contains(t) || pipe_tile_set.contains(t))
        .copied()
        .collect();
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
    // Native-reach rungs: each channel's UG reach equals its declared
    // belt tier. Tier-correct UG pair lengths — the SAT solver finds
    // chained-UG solutions when a single UG can't reach.
    let sat_1ug_native = SatStrategy::with(
        "sat-1ug-native",
        crate::bus::junction_sat_strategy::SatConstraints::max_ug_ins_native(1),
    );
    let sat_2ug_native = SatStrategy::with(
        "sat-2ug-native",
        crate::bus::junction_sat_strategy::SatConstraints::max_ug_ins_native(2),
    );
    let sat_full_native = SatStrategy::with(
        "sat-native",
        crate::bus::junction_sat_strategy::SatConstraints::unrestricted_native(),
    );
    // Auto-upgrade rungs: only included when the user did NOT pin a
    // `max_belt_tier`. When the tier is auto, the engine is free to
    // promote a low-rate channel's UG to the zone's dominant tier so
    // a yellow channel needing more reach than yellow can do gets a
    // red UG (which the SatStrategy post-processes to keep the entity
    // tier in sync with the relaxed reach the solver used). When the
    // user pinned a tier, we deliberately DO NOT include these — an
    // unsolvable zone surfaces as `unresolved-junction`, signalling
    // "your geometry needs a wider belt" rather than silently mixing
    // tiers behind the user's back.
    let sat_1ug_upgrade = SatStrategy::with(
        "sat-1ug-upgrade",
        crate::bus::junction_sat_strategy::SatConstraints::max_ug_ins(1),
    );
    let sat_2ug_upgrade = SatStrategy::with(
        "sat-2ug-upgrade",
        crate::bus::junction_sat_strategy::SatConstraints::max_ug_ins(2),
    );
    let sat_full_upgrade = SatStrategy::unrestricted();
    // Strategy order = priority. Walker vetoes bad proposals from any
    // of them; escalation happens naturally by falling through to the
    // next strategy in the list.
    //   1. cheap templates (fixed footprint, no search)
    //   2. surface-only SAT — simplest layout, no UG at all
    //   3-5. SAT with increasing UG budget at NATIVE reach — tier-
    //        correct UG lengths, including chained-UG solutions.
    //   6.   (auto only) eviction — pulls one or more participating
    //        specs out of the SAT problem (geometrically or via A*),
    //        then re-invokes SAT on the reduced spec set. Gated to
    //        auto-tier alongside the AutoUpgrade rungs because pinned-
    //        tier mode is contractually strict (see f04152d): an
    //        unsolvable zone should fail loudly with `unresolved-
    //        junction` rather than silently re-route specs around it,
    //        even though eviction's per-channel reach stays tier-correct
    //        unlike Relaxed.
    //   7-9. (auto only) AutoUpgrade rungs — Relaxed reach with UG
    //        entity-tier promoted to the zone's dominant tier.
    let eviction_strategy = EvictionStrategy::default_recipes();
    let mut strategies: Vec<&dyn JunctionStrategy> = vec![
        &perp_strategy,
        &sat_surface,
        &sat_1ug_native,
        &sat_2ug_native,
        &sat_full_native,
    ];
    if max_belt_tier.is_none() {
        strategies.push(&eviction_strategy);
        strategies.push(&sat_1ug_upgrade);
        strategies.push(&sat_2ug_upgrade);
        strategies.push(&sat_full_upgrade);
    }
    let strategies: &[&dyn JunctionStrategy] = &strategies;

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
        &spec_kinds,
    );

    for cluster in &clusters {
        // `corridor_handled` grows during this loop — a prior cluster's
        // SAT footprint may have absorbed tiles that belong to this
        // cluster (the unhandled_crossings filter above runs once,
        // before any solve). Skip if any tile is now handled to avoid
        // double-stamping. (Previously a debug_assert that held only
        // because the classify_crossing gate filtered out the
        // single-spec clusters that now make this case observable.)
        if cluster.iter().any(|t| corridor_handled.contains(t)) {
            continue;
        }
        // classify_crossing gates on "exactly two specs with a valid
        // direction at the tile". For belt×belt that's the right shape.
        // For belt×forbidden-tile (canonical case: a belt routed through
        // a fluid-trunk pipe via the soft-pipe astar_hard policy), only
        // one spec is in routed_paths — but the growth loop + SAT can
        // still solve it, because pipes are auto-forbidden in
        // refresh_forbidden and SAT supports UG bypasses.
        //
        // Defer to unresolved only when classify_crossing AND the
        // 1-spec-on-forbidden-tile shape both miss. This preserves the
        // original conservatism for genuinely degenerate clusters while
        // letting pipe×belt (and any future single-spec bypass case)
        // reach the solver.
        let any_undecidable = cluster.iter().any(|&t| {
            if classify_crossing(t, &routed_paths, &specs, &spec_items, &spec_belt_tiers, &spec_kinds).is_some() {
                return false;
            }
            let spec_count_at_tile = routed_paths
                .values()
                .filter(|path| path.contains(&t))
                .count();
            let tile_is_forbidden = entities.iter().any(|e| {
                (e.x, e.y) == t && crate::common::tile_is_forbidden_kind(&e.name)
            });
            !(spec_count_at_tile >= 1 && tile_is_forbidden)
        });
        if any_undecidable {
            for &t in cluster {
                remaining_crossings.insert(t);
            }
            continue;
        }
        // Union of spec keys across every cluster tile. Scan
        // routed_paths directly so synthetic trunk keys (injected
        // above) are included alongside regular BeltSpec keys.
        // Pipe specs are filtered out — the SAT solver doesn't model
        // fluid flow, so pipes participate as forbidden tiles only
        // (their entities are placed and become hard obstacles via
        // `refresh_forbidden`'s obstacle pass). Including them here
        // makes their in-bbox tiles get exempted as boundary ports
        // and emitted as fluid flow boundaries SAT can't satisfy.
        let cluster_tiles: FxHashSet<(i32, i32)> = cluster.iter().copied().collect();
        let keys_at_tile: Vec<&str> = routed_paths
            .iter()
            .filter(|(key, path)| {
                path.iter().any(|t| cluster_tiles.contains(t))
                    && !matches!(
                        spec_kinds.get(key.as_str()),
                        Some(crate::bus::junction::SpecKind::Pipe)
                    )
            })
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
        // invocation. Gated by `SPAGHETTIO_DUMP_REGION_FIXTURE=<dir>`;
        // narrowable with `SPAGHETTIO_DUMP_REGION_FIXTURE_SEED="x,y"` to
        // match a specific cluster seed. Off by default, zero-cost when
        // unset. See `crates/core/tests/region_fixtures/README.md`.
        #[cfg(not(target_arch = "wasm32"))]
        dump_region_fixture(
            cluster.as_slice(),
            &keys_at_tile,
            &routed_paths,
            &hard_for_junction,
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
            &hard_for_junction,
            &junction_hard,
            &unreleasable_obstacles,
            &spec_belt_tiers,
            &spec_items,
            &spec_exit_dirs,
            &spec_kinds,
            &entities,
            strategies,
            &pending_crossings,
        ) else {
            // Diagnostic: when SPAGHETTIO_BLAME_JUNCTIONS=1 is set,
            // identify which spec's removal would let the cluster
            // solve. Helps narrow "what shape of crossing keeps
            // failing" without instrumenting the solver itself.
            if std::env::var("SPAGHETTIO_BLAME_JUNCTIONS").is_ok() {
                blame_unsolvable_cluster(
                    cluster.as_slice(),
                    &keys_at_tile,
                    &routed_paths,
                    &hard,
                    &junction_hard,
                    &unreleasable_obstacles,
                    &spec_belt_tiers,
                    &spec_items,
                    &spec_exit_dirs,
                    &spec_kinds,
                    &entities,
                    strategies,
                    &pending_crossings,
                );
            }
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
                        belt_tier: b.belt_tier.clone(),
                        channel_id: b.channel_id,
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
        // Clean-slate release: clear every releasable claim inside the SAT
        // zone — ghost surface belts AND main-bus structure (trunk/tapoff/
        // balancer Permanent claims) — regardless of which spec they belong
        // to. The previous "minimum-authority" rule (preserve any tile SAT
        // didn't explicitly stamp) created orphan trunk stubs when SAT
        // routed a participating item through a different column from the
        // pre-stamped trunk (issue #243). The release function already
        // preserves non-bus structure (machines, poles, pipes, row
        // entities), so this is safe for those.
        //
        // Exception: tiles SAT marks as `forced_empty` are bus-structure
        // tiles SAT relies on as fixed obstacles (e.g. a balancer splitter
        // it routes around via interior boundaries). Releasing them and
        // letting Step 6 drop the entity leaves SAT's solution feeding
        // belts into thin air (issue #295). Preserve those Permanent
        // claims via `preserve_trunk_tiles`.
        //
        // Non-participating "encountered" specs are handled via the SAT
        // boundary mechanism — `topology_boundaries` emits port boundaries
        // for them so the SAT solution covers their flow through the zone.
        let participating_keys: rustc_hash::FxHashSet<&str> = sol
            .participating
            .iter()
            .map(String::as_str)
            .collect();
        // Narrow preserve set to balancer-segment Permanent claims only.
        // Trunks and tapoffs in `forced_empty` are still released — SAT may
        // route around them via UG arcs and Step 6 will drop the orphan
        // surface stubs (issue #243's original reason for clean-slate
        // release). Splitters are different: SAT models them as fixed
        // structure via interior boundaries and never re-stamps them, so
        // they must survive (issue #295).
        let forced_empty_set: rustc_hash::FxHashSet<(i32, i32)> = sol
            .sat_zone
            .as_ref()
            .map(|snap| snap.forced_empty.iter().copied().collect())
            .unwrap_or_default();
        let preserve_balancer_tiles: rustc_hash::FxHashSet<(i32, i32)> =
            forced_empty_set
                .iter()
                .filter(|tile| {
                    matches!(
                        occupancy.claim_at(**tile),
                        Some(crate::bus::ghost_occupancy::Claim::Permanent { entity_idx })
                            if occupancy
                                .entity_at(**tile)
                                .and_then(|e| e.segment_id.as_deref())
                                .is_some_and(|seg| seg.starts_with("balancer:"))
                                && { let _ = entity_idx; true }
                    )
                })
                .copied()
                .collect();
        let preserve_ref = if preserve_balancer_tiles.is_empty() {
            None
        } else {
            Some(&preserve_balancer_tiles)
        };
        let released_count = occupancy.release_for_pertile_template(
            &release_rect,
            None,
            preserve_ref,
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
        &spec_kinds,
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
    // Deterministic Vec order (not a set): with multiple output items the
    // iteration order decides merge-column positions, and FxHashSet order
    // is arbitrary. Dedup by seen-set, first occurrence wins.
    let mut seen_output_items: FxHashSet<&str> = FxHashSet::default();
    let output_items: Vec<String> = solver_result
        .external_outputs
        .iter()
        .filter(|ext| !ext.is_fluid && seen_output_items.insert(ext.item.as_str()))
        .map(|ext| ext.item.clone())
        .collect();

    // Running x-cursor so successive items' merge blocks tile east instead
    // of overlapping. Start east of EVERY row (not just the participating
    // ones) so south columns never clip a wider foreign row, and record
    // placed column x-positions so later items' east extension runs can
    // UG-hop across them.
    let mut merge_x_cursor: i32 = if output_items.len() > 1 {
        row_spans.iter().map(|rs| rs.row_width).max().unwrap_or(0) + 1
    } else {
        0
    };
    let mut blocked_columns: Vec<i32> = Vec::new();
    for item in &output_items {
        let output_rows: Vec<usize> = row_spans
            .iter()
            .enumerate()
            .filter(|(_, rs)| rs.spec.outputs.iter().any(|o| &o.item == item && !o.is_fluid))
            .map(|(i, _)| i)
            .collect();

        if !output_rows.is_empty() {
            let (merge_ents, merge_end_y, item_merge_x) = merge_output_rows(
                &output_rows,
                item,
                row_spans,
                max_y,
                max_belt_tier,
                merge_x_cursor,
                &blocked_columns,
            );
            blocked_columns.extend((item_merge_x - output_rows.len() as i32)..item_merge_x);
            merge_x_cursor = item_merge_x + 1;
            crate::trace::emit(crate::trace::TraceEvent::OutputMerged {
                item: item.clone(),
                rows: output_rows.clone(),
                merge_y: max_y,
            });
            // Stream sibling of OutputMerged — carries the merger entity batch
            // so the live renderer can reveal them progressively.
            if !merge_ents.is_empty() {
                crate::trace::emit(crate::trace::TraceEvent::OutputMergerCommitted {
                    item: item.clone(),
                    entities: merge_ents.clone(),
                });
            }
            entities.extend(merge_ents);
            max_y = max_y.max(merge_end_y);
            merge_max_x = merge_max_x.max(item_merge_x);
        }
    }

    // -------------------------------------------------------------------------
    // Display-tier pass: upgrade ghost-routed horizontal belts to max tier
    // -------------------------------------------------------------------------
    // Tap, ret, and feeder specs are sized for their actual rate during
    // routing — `lane.rate * 2.0` for taps/returns, `fam.total_rate` for
    // feeders — because the junction solver consults entity tier names to
    // size UG-reach budgets and assign SAT channels (junction_sat_strategy
    // line 492+, line 810+). Bumping the routing tier broke dense
    // processing-unit / partitioned-AC corridors with 60s+ timeouts.
    //
    // But the user-visible artefact is `transport-belt` next to
    // `express-transport-belt` at row-gap y-coords: the row's input belt
    // is always max tier (`row_input_belt` in placer.rs), so a tap-off
    // sized for less than max tier displays as a 1-tile downshift right at
    // the seam. Trunks (`trunk:` segment) and tap-off splitters
    // (`tapoff:` segment) keep their routing tier — they don't abut row
    // inputs.
    let display_belt = belt_entity_for_rate(f64::INFINITY, max_belt_tier);
    let display_ug = underground_for_belt(display_belt);
    for ent in &mut entities {
        let Some(seg) = ent.segment_id.as_deref() else {
            continue;
        };
        if !seg.starts_with("ghost:") {
            continue;
        }
        if crate::common::is_surface_belt(&ent.name) {
            ent.name = display_belt.to_string();
        } else if crate::common::is_ug_belt(&ent.name) {
            ent.name = display_ug.to_string();
        }
    }

    // -------------------------------------------------------------------------
    // Family-trunk pass: lift trunk/tapoff tiers to match the family balancer
    // -------------------------------------------------------------------------
    // When a lane belongs to a family, the family balancer above is sized for
    // `family.total_rate` while the trunk below is sized for `lane.rate * 2.0`
    // (the per-output rate). When the per-output rate falls just under a tier
    // threshold (e.g. 30/s split four ways = 7.5/s per lane → `lane.rate * 2.0
    // = 15` lands exactly on yellow's 15/s threshold), the trunk renders
    // yellow under a fast balancer. This isn't just cosmetic: the balancer's
    // outputs concentrate flow on one lane (a single-side feed pattern), so
    // the trunk's left lane sees up to `lane.rate * 2.0`, exceeding yellow's
    // 7.5/s per-lane cap and tripping the `lane-throughput` validator
    // (issue #267).
    //
    // The fix runs as a post-routing rename so the junction solver and SAT
    // see routing-tier entities throughout (the same reason the ghost: pass
    // upgrades only after routing). We build a `(item, x) → family_tier`
    // lookup and rename trunk-segment entities — surface belts, UGs, and
    // splitters — to the family tier when applicable.
    let mut family_tier_for: FxHashMap<(String, i32), &'static str> = FxHashMap::default();
    for lane in lanes {
        if lane.is_fluid {
            continue;
        }
        let Some(fid) = lane.family_id else { continue };
        let Some(fam) = families.get(fid) else { continue };
        let tier = belt_entity_for_rate(fam.total_rate, max_belt_tier);
        let lane_tier = belt_entity_for_rate(lane.rate * 2.0, max_belt_tier);
        if tier == lane_tier {
            continue; // Already at the right tier; skip the lookup insert.
        }
        // The splitter at non-last taps occupies (lane.x, ty-1) and
        // (lane.x+1, ty-1); register both columns so the splitter's
        // secondary tile gets renamed too.
        family_tier_for.insert((lane.item.clone(), lane.x), tier);
        family_tier_for.insert((lane.item.clone(), lane.x + 1), tier);
    }
    if !family_tier_for.is_empty() {
        for ent in &mut entities {
            let Some(item) = ent.carries.as_deref() else {
                continue;
            };
            let Some(&tier) = family_tier_for.get(&(item.to_string(), ent.x)) else {
                continue;
            };
            // Trunk verticals + tap-off splitters (the original stamps),
            // plus junction-crossing replacements (SAT / eviction) that
            // sit on a trunk column carrying the family item — those
            // entities continue the trunk's vertical flow through the
            // bbox interior, so they need the family tier too.
            let Some(seg) = ent.segment_id.as_deref() else {
                continue;
            };
            let upgradeable = seg.starts_with("trunk:")
                || seg.starts_with("tapoff:")
                || seg.starts_with("crossing:")
                || seg.starts_with("junction:");
            if !upgradeable {
                continue;
            }
            if crate::common::is_surface_belt(&ent.name) {
                ent.name = tier.to_string();
            } else if crate::common::is_ug_belt(&ent.name) {
                ent.name = underground_for_belt(tier).to_string();
            } else if crate::common::is_splitter(&ent.name) {
                ent.name = splitter_for_belt(tier).to_string();
            }
        }

        // UG pair equalization: the rename above is keyed by (item, x-column).
        // A horizontal UG INPUT upgraded on the trunk column leaves its OUTPUT
        // (at a different x, outside the trunk columns) at the original tier,
        // creating a mismatched pair that fails the underground-belt validator.
        // Fix: for every crossing:/junction: UG input that was upgraded, find
        // its matching output (same segment_id, item, direction) and set it to
        // the same UG tier.
        //
        // Collect upgraded crossing/junction UG inputs: (seg, item, dir_u8) → new_ug_name
        // EntityDirection is repr(u8) but doesn't impl Hash, so cast to u8 for the key.
        let mut upgraded_ug_inputs: FxHashMap<(String, String, u8), String> =
            FxHashMap::default();
        for ent in entities.iter() {
            if !crate::common::is_ug_belt(&ent.name) {
                continue;
            }
            if ent.io_type.as_deref() != Some("input") {
                continue;
            }
            let Some(seg) = ent.segment_id.as_deref() else {
                continue;
            };
            if !seg.starts_with("crossing:") && !seg.starts_with("junction:") {
                continue;
            }
            let Some(item) = ent.carries.as_deref() else {
                continue;
            };
            // Only record if this input was actually upgraded (i.e. it hit a
            // family_tier_for entry at its column).
            if !family_tier_for.contains_key(&(item.to_string(), ent.x)) {
                continue;
            }
            upgraded_ug_inputs.insert(
                (seg.to_string(), item.to_string(), ent.direction as u8),
                ent.name.clone(),
            );
        }
        // Apply the same UG name to the matching outputs.
        if !upgraded_ug_inputs.is_empty() {
            for ent in &mut entities {
                if !crate::common::is_ug_belt(&ent.name) {
                    continue;
                }
                if ent.io_type.as_deref() != Some("output") {
                    continue;
                }
                let Some(seg) = ent.segment_id.as_deref() else {
                    continue;
                };
                if !seg.starts_with("crossing:") && !seg.starts_with("junction:") {
                    continue;
                }
                let Some(item) = ent.carries.as_deref() else {
                    continue;
                };
                let key = (seg.to_string(), item.to_string(), ent.direction as u8);
                if let Some(ug_name) = upgraded_ug_inputs.get(&key) {
                    ent.name = ug_name.clone();
                }
            }
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
    /// Belt name for each spec (for entity construction). Unused when the
    /// corresponding kind is `Pipe` — pipe specs are fixed in place and
    /// don't get belt-family entities stamped.
    belt_a: &'static str,
    belt_b: &'static str,
    /// Transport kind per spec. Belt×Belt crossings go through the
    /// existing `try_bridge`; Belt×Pipe short-circuits to
    /// `bridge_belt_over_pipe`, which never stamps over the pipe tile.
    kind_a: crate::bus::junction::SpecKind,
    kind_b: crate::bus::junction::SpecKind,
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
    spec_kinds: &FxHashMap<String, crate::bus::junction::SpecKind>,
) -> Vec<Vec<(i32, i32)>> {
    use crate::bus::junction::SpecKind;
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

    // Tiles that sit on a pipe-kind spec's path. These are belt×pipe
    // crossings and MUST stay as singleton clusters — the perpendicular
    // template's `bridge_belt_over_pipe` handles the 2-spec case directly.
    // Merging them with belt×belt neighbours produces multi-spec clusters
    // that neither the template (requires exactly 2 specs) nor SAT (guards
    // against pipe-kind specs per Phase 3 in the RFP) can solve.
    let mut pipe_tiles: FxHashSet<usize> = FxHashSet::default();
    for (key, path) in routed_paths {
        if !matches!(spec_kinds.get(key.as_str()), Some(SpecKind::Pipe)) {
            continue;
        }
        for t in path {
            if let Some(&i) = index_of.get(t) {
                pipe_tiles.insert(i);
            }
        }
    }

    // tile → set of spec keys whose path passes through it.
    // Pipe-kind specs (fluid-trunk synth paths) are excluded: they're
    // stationary carriers, not flows that need joint routing across
    // crossings. Including them would merge every belt×pipe crossing
    // sharing a pipe column into one giant multi-spec cluster.
    let mut tile_specs: Vec<FxHashSet<&str>> =
        vec![FxHashSet::default(); tiles.len()];
    for (key, path) in routed_paths {
        if matches!(spec_kinds.get(key.as_str()), Some(SpecKind::Pipe)) {
            continue;
        }
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
        // Belt×pipe crossings stay singleton — don't consider them as
        // merge candidates from either side.
        if pipe_tiles.contains(&i) {
            continue;
        }
        for &(dx, dy) in OFFSETS {
            let Some(&j) = index_of.get(&(x + dx, y + dy)) else {
                continue;
            };
            if pipe_tiles.contains(&j) {
                continue;
            }
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
        let _tier = spec_belt_tiers
            .get(key.as_str())
            .copied()
            .unwrap_or(BeltTier::Yellow);
        // Tighter reach threshold (was `max_reach + 1` ≡ 5 yellow, 7
        // fast, 9 express): the wide threshold pulled distant
        // independently-soluble crossings into super-clusters when
        // they merely shared a long trunk path. The rescue scenario
        // we still want to catch (an isolated tap-off jammed up
        // against a multi-cluster's committed UG envelope) lives at
        // path-distance ≤ 3; anything further is over-aggressive.
        let reach_threshold = 3;
        for w in on_path.windows(2) {
            let (i, p_i) = w[0];
            let (j, p_j) = w[1];
            if p_j - p_i > reach_threshold {
                continue;
            }
            // Belt×pipe crossings stay singleton; never pull them into
            // or rescue them via UG-reach merges.
            if pipe_tiles.contains(&i) || pipe_tiles.contains(&j) {
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
/// Diagnostic for `SPAGHETTIO_BLAME_JUNCTIONS=1`: when a cluster fails
/// to solve, retry with each participating spec removed in turn. Any
/// removal that lets the cluster solve points at a "blamed" spec —
/// the kind of crossing this solver is failing on. Emits one
/// `JunctionBlamedSpec` per such removal.
///
/// Cost: up to N extra `solve_crossing` calls per failed cluster (N =
/// participating spec count). Gated on env var because each retry can
/// take seconds; keep it off in CI. Trace events from the retries are
/// muted via `trace::with_muted` so they don't pollute the real
/// event stream.
#[allow(clippy::too_many_arguments)]
fn blame_unsolvable_cluster(
    cluster: &[(i32, i32)],
    initial_specs: &[&str],
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    hard: &FxHashSet<(i32, i32)>,
    junction_hard: &FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &FxHashSet<(i32, i32)>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
    spec_items: &FxHashMap<String, String>,
    spec_exit_dirs: &FxHashMap<String, EntityDirection>,
    spec_kinds: &FxHashMap<String, crate::bus::junction::SpecKind>,
    entities: &[PlacedEntity],
    strategies: &[&dyn JunctionStrategy],
    pending_crossings: &FxHashSet<(i32, i32)>,
) {
    if initial_specs.len() < 2 {
        return; // can't blame a single spec — nothing to remove
    }
    let cluster_seed = cluster[0];
    for &removed in initial_specs {
        let kept_specs: Vec<&str> = initial_specs
            .iter()
            .copied()
            .filter(|&k| k != removed)
            .collect();
        let mut kept_paths = routed_paths.clone();
        kept_paths.remove(removed);
        let solved = trace::with_muted(|| {
            junction_solver::solve_crossing(
                cluster,
                &kept_specs,
                &kept_paths,
                hard,
                junction_hard,
                unreleasable_obstacles,
                spec_belt_tiers,
                spec_items,
                spec_exit_dirs,
                spec_kinds,
                entities,
                strategies,
                pending_crossings,
            )
            .is_some()
        });
        if !solved {
            continue;
        }
        let item = spec_items.get(removed).cloned().unwrap_or_default();
        let dir = spec_exit_dirs
            .get(removed)
            .map(|d| format!("{d:?}"))
            .unwrap_or_default();
        trace::emit(trace::TraceEvent::JunctionBlamedSpec {
            cluster_x: cluster_seed.0,
            cluster_y: cluster_seed.1,
            participating: initial_specs.len(),
            spec_key: removed.to_string(),
            spec_item: item,
            spec_direction: dir,
        });
    }
}

/// (e.g. synthetic trunk paths) that don't have a corresponding `BeltSpec`
/// in `specs`.
fn classify_crossing(
    tile: (i32, i32),
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    specs: &[BeltSpec],
    spec_items: &FxHashMap<String, String>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
    spec_kinds: &FxHashMap<String, crate::bus::junction::SpecKind>,
) -> Option<CrossingInfo> {
    use crate::bus::junction::SpecKind;
    let (cx, cy) = tile;

    let spec_map: FxHashMap<&str, &BeltSpec> = specs.iter().map(|s| (s.key.as_str(), s)).collect();
    // Each entry: (item, belt_name, direction, kind)
    let mut crossing_specs: Vec<(String, &'static str, EntityDirection, SpecKind)> = Vec::new();

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
        let kind = spec_kinds.get(key.as_str()).copied().unwrap_or(SpecKind::Belt);

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
                crossing_specs.push((item, belt_name, dir, kind));
                break;
            }
        }
    }

    if crossing_specs.len() != 2 {
        return None;
    }
    let (ref item_a, belt_a, da, ka) = crossing_specs[0];
    let (ref item_b, belt_b, db, kb) = crossing_specs[1];

    Some(CrossingInfo {
        tile,
        spec_a: (item_a.clone(), da),
        spec_b: (item_b.clone(), db),
        belt_a,
        belt_b,
        kind_a: ka,
        kind_b: kb,
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
    spec_kinds: &FxHashMap<String, crate::bus::junction::SpecKind>,
    ghost_item_at: &FxHashMap<(i32, i32), (String, u32)>,
) -> Vec<LayoutRegion> {
    use crate::bus::junction::{Junction, Rect, SpecCrossing, SpecKind, SpecOrigin};
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
            classify_crossing((tx, ty), routed_paths, specs, spec_items, spec_belt_tiers, spec_kinds)
            .map(|info| {
                // 1×1 bbox: entry and exit sit on the same tile; direction
                // encodes the flow. The lowering in `Junction::to_layout_region`
                // picks the correct edges from `(io, direction)`.
                let make = |item: String, dir: EntityDirection, belt: &str, kind: SpecKind| SpecCrossing {
                    item,
                    belt_tier: BeltTier::from_name(belt).unwrap_or(BeltTier::Yellow),
                    entry: PortPoint { x: tx, y: ty, direction: dir },
                    exit: PortPoint { x: tx, y: ty, direction: dir },
                    origin: SpecOrigin::Participating,
                    kind,
                };
                vec![
                    make(info.spec_a.0, info.spec_a.1, info.belt_a, info.kind_a),
                    make(info.spec_b.0, info.spec_b.1, info.belt_b, info.kind_b),
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
    use crate::bus::junction::SpecKind;
    // Pipe×belt short-circuit. The pipe is a fixed-surface entity that
    // belongs to a fluid-trunk column; the belt must UG-bypass it
    // without anything getting stamped on the pipe tile itself. See
    // `docs/rfp-pipe-belt-junctions.md` for the full story.
    match (info.kind_a, info.kind_b) {
        (SpecKind::Pipe, SpecKind::Pipe) => return None,
        (SpecKind::Pipe, SpecKind::Belt) => {
            return bridge_belt_over_pipe(
                info,
                /* pipe_is_a = */ true,
                hard_obstacles,
                unreleasable_obstacles,
                routed_paths,
                placed_entities,
            );
        }
        (SpecKind::Belt, SpecKind::Pipe) => {
            return bridge_belt_over_pipe(
                info,
                /* pipe_is_a = */ false,
                hard_obstacles,
                unreleasable_obstacles,
                routed_paths,
                placed_entities,
            );
        }
        (SpecKind::Belt, SpecKind::Belt) => {}
    }

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

/// UG-bridge a belt spec around a pipe spec at the crossing tile.
///
/// The pipe is a fixed-surface entity belonging to a fluid-trunk column —
/// it stays put, and nothing gets stamped on `(cx, cy)`. The belt enters
/// a UG-in on one side of the pipe and exits a UG-out on the other.
/// Obstacle / turn / sideload checks on the UG endpoints mirror
/// `try_bridge`'s belt-side handling.
///
/// `pipe_is_a` selects which entry in `CrossingInfo` is the pipe — the
/// other is the belt we're bridging. Returns `None` if any belt-side UG
/// endpoint is obstructed, turns, or receives a sideload; the caller
/// falls through to the SAT strategies (which currently defer on
/// pipe-kind specs via `SatStrategy`).
fn bridge_belt_over_pipe(
    info: &CrossingInfo,
    pipe_is_a: bool,
    hard_obstacles: &FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &FxHashSet<(i32, i32)>,
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    placed_entities: &[PlacedEntity],
) -> Option<(Vec<PlacedEntity>, ClusterZone)> {
    let (cx, cy) = info.tile;
    let (belt_item, belt_dir, belt_name) = if pipe_is_a {
        (&info.spec_b.0, info.spec_b.1, info.belt_b)
    } else {
        (&info.spec_a.0, info.spec_a.1, info.belt_a)
    };

    let (dx, dy) = match belt_dir {
        EntityDirection::North => (0, -1),
        EntityDirection::South => (0, 1),
        EntityDirection::East => (1, 0),
        EntityDirection::West => (-1, 0),
    };

    let bridge_axis_label: &'static str = match belt_dir {
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

    // Pipe-run discovery. The bridge must span every pipe touching the
    // belt's axis at this row/column without surfacing between them, or
    // a stranded pipe between two narrow bridges produces a dead-end.
    // Adjacent fluid-trunk columns (crude-oil/water/petroleum-gas at
    // x=26..28 in processing-unit @ 2/s) give the canonical multi-pipe
    // case. Reservations and ghost belts BETWEEN pipes are swallowed
    // into the run so the resulting UG tunnel passes beneath all of
    // them; we only stop walking when no further pipe lies within
    // `max_reach` tiles ahead.
    let is_pipe_at = |t: (i32, i32)| -> bool {
        placed_entities
            .iter()
            .any(|e| (e.x, e.y) == t && (e.name == "pipe" || e.name == "pipe-to-ground"))
    };
    // A tile is a "blocker" for UG-endpoint placement when it carries an
    // unreleasable entity (machine, pole, row template, etc.) or a pipe.
    // Releasable entities (ghost belts, trunks, tap-offs, prior junction
    // outputs, simple balancers) at the tile are fine — they get released
    // by the per-tile-template release pass before the new UG is stamped.
    // Reservation-only tiles (in `hard` but with no actual entity) are
    // allowed: nothing's actually there to conflict with.
    let blocked_at = |t: (i32, i32)| -> bool {
        placed_entities.iter().any(|e| {
            if (e.x, e.y) != t {
                return false;
            }
            if e.name == "pipe" || e.name == "pipe-to-ground" {
                return true;
            }
            let seg = e.segment_id.as_deref().unwrap_or("");
            !(seg.starts_with("ghost:")
                || seg.starts_with("trunk:")
                || seg.starts_with("tapoff:")
                || seg.starts_with("junction:")
                || seg.starts_with("corridor:")
                || seg.starts_with("crossing:"))
        })
    };
    // Non-pipe blocker check used by the walk: any unreleasable
    // entity at the tile (machines, poles, row templates, splitters,
    // multi-block balancers, etc.) that the bridge can NOT tunnel
    // through. Pipes are NOT counted as blockers here — the walk
    // EXTENDS through them, since the whole point of the bridge is to
    // bury the belt under pipe runs. Releasable kinds (ghost belts,
    // trunks, tap-offs, prior junctions) are also fine: they get
    // released by the per-tile-template release pass.
    let non_pipe_blocker_at = |t: (i32, i32)| -> bool {
        placed_entities.iter().any(|e| {
            if (e.x, e.y) != t {
                return false;
            }
            if e.name == "pipe" || e.name == "pipe-to-ground" {
                return false;
            }
            let seg = e.segment_id.as_deref().unwrap_or("");
            !(seg.starts_with("ghost:")
                || seg.starts_with("trunk:")
                || seg.starts_with("tapoff:")
                || seg.starts_with("junction:")
                || seg.starts_with("corridor:")
                || seg.starts_with("crossing:"))
        })
    };
    let max_reach = ug_max_reach(belt_name) as i32;
    // Walk along the axis from `start` in `(step_dx, step_dy)`, finding
    // the FARTHEST pipe such that every gap between pipes can be
    // jumped underground (i.e. no two pipes are more than `max_reach`
    // tiles apart, and no blocker entity sits in the gap). Returns the
    // last pipe in the run (or `start` if no pipes are reached).
    let walk_run = |start: (i32, i32), step_dx: i32, step_dy: i32| -> (i32, i32) {
        let mut last_pipe = start;
        let mut cur = start;
        loop {
            // Lookahead: any pipe within `max_reach` tiles? Advance
            // through pipes, ghost belts, reservations, and empty tiles.
            // Stop on a non-pipe unreleasable entity — can't tunnel
            // through it.
            let mut found_pipe_ahead: Option<i32> = None;
            for i in 1..=max_reach {
                let probe = (cur.0 + step_dx * i, cur.1 + step_dy * i);
                if is_pipe_at(probe) {
                    found_pipe_ahead = Some(i);
                    break;
                }
                if non_pipe_blocker_at(probe) {
                    break;
                }
            }
            match found_pipe_ahead {
                Some(steps) => {
                    cur = (cur.0 + step_dx * steps, cur.1 + step_dy * steps);
                    last_pipe = cur;
                }
                None => return last_pipe,
            }
        }
    };
    // Backward = upstream (UG-IN side), forward = downstream (UG-OUT
    // side). Walk in both directions so a pipe tile sandwiched between
    // pipes is found regardless of which one was the seed.
    let upstream_end = walk_run((cx, cy), -dx, -dy);
    let downstream_end = walk_run((cx, cy), dx, dy);
    let ug_in = (upstream_end.0 - dx, upstream_end.1 - dy);
    let ug_out = (downstream_end.0 + dx, downstream_end.1 + dy);

    // Underground span between UG-IN and UG-OUT: count of buried tiles.
    // For a single-pipe bridge that's 1; for an N-pipe run it's the
    // distance from upstream_end to downstream_end (inclusive of any
    // ghost-belt/reservation gaps the walk swallowed). Reject if the
    // span exceeds the belt tier's max reach.
    let span = match belt_dir {
        EntityDirection::North | EntityDirection::South => (ug_out.1 - ug_in.1).abs() - 1,
        EntityDirection::East | EntityDirection::West => (ug_out.0 - ug_in.0).abs() - 1,
    };
    if span > max_reach {
        return reject("pipe_run_exceeds_ug_reach");
    }

    // Belt-side endpoint checks. The pipe tiles inside the run are
    // left alone — no stamp goes on them, so we don't consult the
    // obstacle sets for tiles inside the run.
    //
    // We use `blocked_at` (which inspects actual entities at the tile)
    // rather than `hard_obstacles` because the narrow `hard` set
    // includes fluid-trunk reservations — column tiles reserved for
    // future fluid placement but with no actual entity. UG-IN/OUT can
    // sit on a reserved-but-empty tile because the perpendicular fluid
    // UG buried beneath does not interfere (per F4/F7).
    if blocked_at(ug_in) {
        return reject("blocked_ug_in");
    }
    if blocked_at(ug_out) {
        return reject("blocked_ug_out");
    }
    if any_spec_turns_at(ug_in, routed_paths) {
        return reject("turn_at_ug_in");
    }
    if any_spec_turns_at(ug_out, routed_paths) {
        return reject("turn_at_ug_out");
    }
    // The legacy `hard_obstacles` / `unreleasable_obstacles` params are
    // retained on the signature for parity with `bridge_belt_over_pipe`'s
    // siblings; the multi-pipe walk supersedes them by inspecting actual
    // entities at each tile (`blocked_at`, `non_pipe_blocker_at`).
    let _ = hard_obstacles;
    let _ = unreleasable_obstacles;

    // Exclude the belt's own path key from the conflict check. Match by
    // item name in the key (e.g. `flow:iron-plate:21`, `tap:iron-plate:...`)
    // — the pipe's synth key (`trunk:sulfuric-acid:21`) won't match.
    let belt_item_str = belt_item.as_str();
    let bridge_key = routed_paths
        .iter()
        .find(|(k, path)| path.contains(&info.tile) && k.contains(belt_item_str))
        .map(|(k, _)| k.as_str())
        .unwrap_or("");
    if let Some(sub) = ug_endpoint_conflicts(ug_in, belt_dir, bridge_key, routed_paths, placed_entities) {
        return reject(match sub {
            "axis_conflict" => "ug_in_axis_conflict",
            "sideload" => "ug_in_sideload",
            _ => "ug_in_conflict",
        });
    }
    if let Some(sub) = ug_endpoint_conflicts(ug_out, belt_dir, bridge_key, routed_paths, placed_entities) {
        return reject(match sub {
            "axis_conflict" => "ug_out_axis_conflict",
            "sideload" => "ug_out_sideload",
            _ => "ug_out_conflict",
        });
    }

    let ug_name = ug_for_belt(belt_name);
    let seg = Some(format!("junction:{}:{},{}", belt_item, cx, cy));

    // Sanity: at least one pipe must actually exist at the seed tile,
    // otherwise we have no business bridging here.
    if !placed_entities
        .iter()
        .any(|e| (e.x, e.y) == (cx, cy))
    {
        return reject("pipe_tile_missing");
    }

    // Pipes themselves are NOT re-emitted: their Permanent claims survive
    // the release pass (`release_for_pertile_template` skips pipe entities
    // — see `bus/ghost_occupancy.rs`), and trying to stamp a Template
    // entity over an existing Permanent tile would panic in the post-place
    // loop. We only emit the belt-side UG endpoints; the underground
    // tunnel passes beneath every pipe in the run untouched (per F4/U4).
    let entities = vec![
        PlacedEntity {
            name: ug_name.to_string(),
            x: ug_in.0,
            y: ug_in.1,
            direction: belt_dir,
            io_type: Some("input".to_string()),
            carries: Some(belt_item.clone()),
            segment_id: seg.clone(),
            ..Default::default()
        },
        PlacedEntity {
            name: ug_name.to_string(),
            x: ug_out.0,
            y: ug_out.1,
            direction: belt_dir,
            io_type: Some("output".to_string()),
            carries: Some(belt_item.clone()),
            segment_id: seg.clone(),
            ..Default::default()
        },
    ];

    // Zone covers the full UG-IN..UG-OUT span (inclusive). For the
    // single-pipe legacy case this is 3×1 / 1×3; for an N-pipe run it's
    // (N+2)×1 / 1×(N+2). Width is along the belt's axis.
    let (min_x, max_x) = (ug_in.0.min(ug_out.0), ug_in.0.max(ug_out.0));
    let (min_y, max_y) = (ug_in.1.min(ug_out.1), ug_in.1.max(ug_out.1));
    let zone = ClusterZone {
        x: min_x,
        y: min_y,
        w: (max_x - min_x + 1) as u32,
        h: (max_y - min_y + 1) as u32,
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
            kind_a: sa.kind,
            kind_b: sb.kind,
        };
        let (entities, zone) = solve_perpendicular_template(
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
// `SPAGHETTIO_DUMP_REGION_FIXTURE` env var names a directory. Optional
// `SPAGHETTIO_DUMP_REGION_FIXTURE_SEED="x,y"` restricts capture to
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
    let Ok(dir) = std::env::var("SPAGHETTIO_DUMP_REGION_FIXTURE") else {
        return;
    };
    if dir.is_empty() {
        return;
    }

    // Optional seed filter: only dump clusters whose seeds include the
    // requested tile. Useful when a layout produces dozens of clusters
    // and you want just one.
    if let Ok(seed_str) = std::env::var("SPAGHETTIO_DUMP_REGION_FIXTURE_SEED") {
        let parts: Vec<&str> = seed_str.split(',').collect();
        if parts.len() != 2 {
            eprintln!(
                "SPAGHETTIO_DUMP_REGION_FIXTURE_SEED: expected \"x,y\", got {:?}",
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
            "Captured via SPAGHETTIO_DUMP_REGION_FIXTURE. Review expected.mode before committing.",
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

    fn no_kinds() -> FxHashMap<String, crate::bus::junction::SpecKind> {
        // All unit tests model belt×belt clusters; empty map defaults
        // to SpecKind::Belt for every key.
        FxHashMap::default()
    }

    #[test]
    fn empty_input_returns_empty() {
        let cs = FxHashSet::default();
        let rp = FxHashMap::default();
        assert!(cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds()).is_empty());
    }

    #[test]
    fn single_tile_becomes_single_cluster() {
        let cs = crossings(&[(5, 5)]);
        let rp = paths(&[("tap:iron-plate", &[(5, 5)])]);
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
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
        let clusters = cluster_adjacent_crossings(&cs, &rp, &no_tiers(), &no_kinds());
        assert_eq!(clusters.len(), 2);
    }
}
