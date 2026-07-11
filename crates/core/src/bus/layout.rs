//! Bus layout orchestrator: rows + bus lanes + poles -> LayoutResult.
//!
//! Entry point: [`build_bus_layout`]. Calls `place_rows` to stack
//! assembly rows, `plan_bus_lanes` to decide which items need which
//! trunks, and `route_bus_ghost` to materialise every connecting belt
//! via the ghost-routing pipeline. See `docs/ghost-pipeline-contracts.md`
//! for the phase-by-phase invariants the router promises.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::models::{EntityDirection, LayoutResult, PlacedEntity, SolverResult};
use crate::bus::lane_planner::{
    plan_bus_lanes, bus_width_for_lanes, BusLane, LaneFamily, MACHINE_ENTITIES,
};
use crate::bus::placer::{place_rows, RowSpan};

/// Layout strategy. Selects the shape of the bus the engine produces.
/// See `docs/rfp-modular-production.md` for the rationale.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayoutStrategy {
    /// One shared lane family per item, single balancer at the producer
    /// row. Capped at 8 lanes per item.
    #[default]
    Pooled,
    /// One lane family per consuming recipe-row, sized to that
    /// consumer's exact demand, no pool-balancer; plus subtree sharding
    /// when a single module's widest upstream recipe still exceeds 8
    /// lanes. The Phase 1 + Phase 2 strategy from the RFP, merged into
    /// a single variant after Phase 1's per-consumer-only mode was
    /// removed (it was strictly dominated by the decomposed pass across
    /// the diag corpus).
    PartitionedDecomposed,
}

/// Per-recipe row geometry. See `docs/rfp-horizontal-trunks.md`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RowLayout {
    /// Today's behaviour: input-bottlenecked recipes split vertically
    /// into many short rows reconciled by an N→M balancer family.
    #[default]
    VerticalSplit,
    /// One long row with K stacked input belts at the top, each
    /// terminating in a south-axis dive that feeds a sub-row block of
    /// machines. Output is a single full-capacity east-flowing belt.
    /// Phase 1: dual-input solid recipes only; other row kinds fall
    /// back to `VerticalSplit` silently.
    HorizontalStack,
}

/// What the layout does with solid byproduct surplus (`SolverResult::
/// surplus_outputs`). See `docs/rfp-fulgora-scrap.md` D1 — voiding is a
/// layout policy, not a solver objective, so this lives on
/// `LayoutOptions` rather than anywhere in the solver.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SurplusPolicy {
    /// Today's behaviour: solid surplus routes to the perimeter/merger
    /// (RFP Fulgora D2a/D2b). Byte-identical to pre-Phase-2 output.
    #[default]
    Export,
    /// Solid surplus that resolves to a recognized self-voider recipe
    /// (`<item>-recycling`: X → fraction·X) is consumed by a
    /// layout-synthesized recycler bank instead of exported. Streams
    /// that don't resolve (multi-output cascades, missing recipe) fall
    /// back to `Export` with a `VoiderFallbackExport` trace event —
    /// never silently dropped. Fluid surplus is never voided (recycling
    /// takes items only) and always routes via `Export` regardless of
    /// this setting.
    Void,
}

/// Per-call options for `build_bus_layout`. New struct; absorbs the
/// previous `max_belt_tier` parameter so future per-call options
/// (strategy, escargio fold parameters, …) attach as additional fields.
#[derive(Clone, Debug, Default)]
pub struct LayoutOptions {
    pub strategy: LayoutStrategy,
    pub max_belt_tier: Option<String>,
    pub row_layout: RowLayout,
    pub surplus_policy: SurplusPolicy,
}

impl LayoutOptions {
    /// Convenience: keep today's call shape working for tests / examples
    /// that only care about the belt tier.
    pub fn from_belt_tier(max_belt_tier: Option<&str>) -> Self {
        Self {
            strategy: LayoutStrategy::default(),
            max_belt_tier: max_belt_tier.map(|s| s.to_string()),
            row_layout: RowLayout::default(),
            surplus_policy: SurplusPolicy::default(),
        }
    }
}

/// Convert a SolverResult into a bus-style LayoutResult.
///
/// Returns a LayoutResult with:
/// - entities: all belts, inserters, machines, power poles
/// - width: maximum x dimension used
/// - height: maximum y dimension used
///
/// Delegates to the decomposition-search layer
/// (`crate::bus::decomposition_search::select_best_decomposition`),
/// which evaluates each `DecompositionCandidate` against a scoring
/// function and returns the winner. With Phase 0's single
/// `NativeCandidate`, output is byte-identical to direct dispatch
/// (K-DS0-1 inertness gate). See `docs/rfp-decomposition-search.md`.
pub fn build_bus_layout(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String> {
    crate::bus::decomposition_search::select_best_decomposition(solver_result, opts)
}

/// Today's `build_bus_layout` body — the retry orchestrator that
/// invokes `layout_pass`, scans for `JunctionGrowthCapped` events,
/// computes retry gaps, and runs a second pass if needed. Extracted
/// from `build_bus_layout` so `NativeCandidate::produce` can call it
/// directly without recursing through the search layer. See
/// `docs/rfp-decomposition-search.md` §Design.
pub(crate) fn run_layout_with_retry(
    solver_result: &SolverResult,
    opts: &LayoutOptions,
) -> Result<LayoutResult, String> {
    run_layout_with_retry_inner(solver_result, opts, None)
}

/// Variant of `run_layout_with_retry` that bypasses the strategy-driven
/// `plan_partitioning` call and uses the caller's `explicit_plan`
/// directly. Used by candidate decompositions
/// (`bus::decomposition_search::K1ShapeFix`) that want to overlay
/// per-`(item, module_id)` `lane_count` overrides onto the partition
/// plan — `plan_pad_floor` in `lane_planner::split_overflowing_lanes`
/// reads those overrides as a lower bound on `effective_n_splits`.
pub(crate) fn run_layout_with_explicit_plan(
    solver_result: &SolverResult,
    opts: &LayoutOptions,
    plan: &crate::bus::partitioner::PartitionPlan,
) -> Result<LayoutResult, String> {
    run_layout_with_retry_inner(solver_result, opts, Some(plan))
}

fn run_layout_with_retry_inner(
    solver_result: &SolverResult,
    opts: &LayoutOptions,
    explicit_plan: Option<&crate::bus::partitioner::PartitionPlan>,
) -> Result<LayoutResult, String> {
    // Snapshot the trace collector before the first pass so we can
    // detect `JunctionGrowthCapped` events emitted by *this* layout
    // call (and not whatever the caller already had collected).
    let trace_start = crate::trace::peek_events_len();

    // Detach the active sink (if any) for pass 1. The collector still
    // sees every event — that's what we need for retry detection. The
    // sink is reinstalled (or replayed-into) below depending on whether
    // pass 1 caps. This keeps the streaming consumer from seeing
    // events from a layout pass that gets abandoned by retry.
    let original_sink = crate::trace::swap_sink(None);

    let pass_1 = layout_pass(solver_result, opts, None, explicit_plan);
    let (result_1, row_spans_1) = pass_1?;

    // Scan only events emitted by this layout call.
    let new_events = crate::trace::peek_events_since(trace_start);
    let cap_coords: Vec<(i32, i32)> = new_events
        .iter()
        .filter_map(|e| match e {
            crate::trace::TraceEvent::JunctionGrowthCapped { tile_x, tile_y, .. } => {
                Some((*tile_x, *tile_y))
            }
            _ => None,
        })
        .collect();

    let retry_gaps = if cap_coords.is_empty() {
        FxHashMap::default()
    } else {
        compute_retry_gaps(&cap_coords, &row_spans_1)
    };

    if retry_gaps.is_empty() {
        // No retry — replay pass-1 events from the collector to the
        // original sink so the streaming consumer sees the same events
        // it would have seen without the silent-pass wrapper.
        if let Some(mut sink) = original_sink {
            for evt in &new_events {
                sink(evt);
            }
            crate::trace::swap_sink(Some(sink));
        }
        return Ok(result_1);
    }

    // Discard pass-1 events from the collector so `result.trace`
    // reflects only the retried (final) pass. The sink never saw them
    // (because we detached it above), so the streaming consumer
    // doesn't see the abandoned pass 1 either.
    crate::trace::truncate_events(trace_start);

    // Reinstall the original sink so pass-2 events stream live.
    if let Some(sink) = original_sink {
        crate::trace::swap_sink(Some(sink));
    }

    let mut gaps_vec: Vec<(usize, i32)> = retry_gaps.iter().map(|(k, v)| (*k, *v)).collect();
    gaps_vec.sort_by_key(|(k, _)| *k);
    let recipes: Vec<String> = gaps_vec
        .iter()
        .map(|(idx, _)| row_spans_1[*idx].spec.recipe.clone())
        .collect();
    crate::trace::emit(crate::trace::TraceEvent::LayoutRetried {
        gaps: gaps_vec,
        caps_before: cap_coords.len(),
        recipes,
    });

    let (result_2, _) = layout_pass(solver_result, opts, Some(&retry_gaps), explicit_plan)?;
    Ok(result_2)
}

/// Map `JunctionGrowthCapped` (x, y) coordinates to the row indices
/// whose *successor* gap should be widened. A cap that fires inside
/// row `i`'s span (or in the open band immediately below it) means the
/// junction couldn't pack the geometry around row `i`'s tap-offs.
/// Widening the gap *before* row `i` (i.e. after row `i-1`) shifts row
/// `i` and everything below it down by one tile, giving the junction
/// solver an extra row of vertical room to land its entries/exits.
fn compute_retry_gaps(
    cap_coords: &[(i32, i32)],
    row_spans: &[RowSpan],
) -> FxHashMap<usize, i32> {
    let mut out: FxHashMap<usize, i32> = FxHashMap::default();
    for &(_x, y) in cap_coords {
        // Find the row whose span contains y, or the first row that
        // starts at-or-below y if y is in an inter-row gap.
        let target = row_spans.iter().position(|span| y <= span.y_end);
        let Some(target) = target else { continue };
        // Widen the gap *before* `target` (i.e. after row `target - 1`).
        // No-op if target is row 0 — there's no preceding row to widen.
        if target > 0 {
            let widen_after = target - 1;
            out.entry(widen_after)
                .and_modify(|v| *v = (*v).max(1))
                .or_insert(1);
        }
    }
    out
}

/// One layout attempt — the body of the original `build_bus_layout`.
/// Takes an optional `retry_extra_gaps` map (row index → extra tiles)
/// that the retry loop in `build_bus_layout` uses to widen specific
/// row boundaries on a second pass. `None` on the first pass; `Some`
/// on the retry. Returns the layout plus the final `row_spans`, which
/// `build_bus_layout` needs to map cap coordinates back to row indices.
fn layout_pass(
    solver_result: &SolverResult,
    opts: &LayoutOptions,
    retry_extra_gaps: Option<&FxHashMap<usize, i32>>,
    explicit_plan: Option<&crate::bus::partitioner::PartitionPlan>,
) -> Result<(LayoutResult, Vec<RowSpan>), String> {
    let max_belt_tier = opts.max_belt_tier.as_deref();

    // RFP Fulgora Phase 2 (docs/rfp-fulgora-scrap.md D1): under
    // `SurplusPolicy::Void`, synthesize recycler-bank voider rows for
    // solid surplus that resolves to a self-voider recipe BEFORE any
    // other pipeline stage runs, so `place_rows`/`plan_bus_lanes`/
    // `route_bus_ghost` all see the voider `MachineSpec`s as ordinary
    // rows and the item removed from `surplus_outputs` as ordinary
    // export/lane machinery would expect. No-op (same reference, zero
    // clone) under `SurplusPolicy::Export` — KC4's byte-identical
    // guarantee for the default policy.
    let voided_solver_result;
    let solver_result: &SolverResult = if opts.surplus_policy == SurplusPolicy::Void {
        voided_solver_result = crate::bus::voider::synthesize_voiders(solver_result);
        &voided_solver_result
    } else {
        solver_result
    };

    // Plan source. If the caller has pre-built a plan (candidate
    // decompositions in `bus::decomposition_search`), use it directly
    // and skip the strategy-driven `plan_partitioning` call. Otherwise
    // dispatch on `opts.strategy`: `Pooled` passes through unchanged;
    // `PartitionedDecomposed` runs `plan_partitioning` + `apply_partition_plan`
    // up-front so the rest of the pipeline picks up the per-`(item,
    // module_id)` flow tagging via `ItemFlow.module_id`. Empty plan
    // (K=1 everywhere) → byte-identical to `Pooled`.
    let owned_solver_result;
    let owned_plan;
    let (solver_result, plan_ref): (&SolverResult, Option<&crate::bus::partitioner::PartitionPlan>) =
        match (explicit_plan, opts.strategy) {
            (Some(plan), _) => {
                owned_solver_result =
                    crate::bus::partitioner::apply_partition_plan(solver_result, plan);
                (&owned_solver_result, Some(plan))
            }
            (None, LayoutStrategy::Pooled) => (solver_result, None),
            (None, LayoutStrategy::PartitionedDecomposed) => {
                let plan = crate::bus::partitioner::plan_partitioning(
                    solver_result,
                    opts.strategy,
                    max_belt_tier,
                );
                if plan.is_empty() {
                    (solver_result, None)
                } else {
                    owned_solver_result =
                        crate::bus::partitioner::apply_partition_plan(solver_result, &plan);
                    owned_plan = plan;
                    (&owned_solver_result, Some(&owned_plan))
                }
            }
        };
    // Final product items get EAST-flowing output belts (merge at right side)
    let final_output_items: FxHashSet<String> = solver_result
        .external_outputs
        .iter()
        .filter(|ext| !ext.is_fluid)
        .map(|ext| ext.item.clone())
        .collect();

    let bus_header = 1;

    crate::trace::emit(crate::trace::TraceEvent::SolverCompleted {
        recipe_count: solver_result.machines.len(),
        machine_count: solver_result.machines.iter().map(|m| m.count.ceil() as usize).sum(),
        external_input_count: solver_result.external_inputs.len(),
        external_output_count: solver_result.external_outputs.len(),
        machines: solver_result.machines.iter().map(|m| crate::trace::MachineTrace {
            recipe: m.recipe.clone(),
            machine: m.entity.clone(),
            count: m.count,
            rate: m.outputs.iter().map(|o| o.rate).sum::<f64>() * m.count,
        }).collect(),
    });

    // Pass 1: place rows with an estimated bus width, then plan lanes
    // so we know the real bus width and any balancer blocks that need
    // vertical gaps between producer rows.
    let temp_bw = estimate_bus_width(solver_result);
    let t_place1 = web_time::Instant::now();
    let (row_entities_1, row_spans_1, _row_width_1, total_height_1) = place_rows(
        &solver_result.machines,
        &solver_result.dependency_order,
        temp_bw,
        bus_header,
        max_belt_tier,
        Some(&final_output_items),
        retry_extra_gaps,
        opts.row_layout,
    );
    crate::trace::emit(crate::trace::TraceEvent::PhaseTime {
        phase: "place_rows_1".to_string(),
        duration_ms: t_place1.elapsed().as_millis() as u64,
    });
    let t_plan1 = web_time::Instant::now();
    let (lanes_1, families_1) =
        plan_bus_lanes(solver_result, &row_spans_1, max_belt_tier, plan_ref, total_height_1)?;
    crate::trace::emit(crate::trace::TraceEvent::PhaseTime {
        phase: "plan_bus_lanes_1".to_string(),
        duration_ms: t_plan1.elapsed().as_millis() as u64,
    });
    let actual_bw = bus_width_for_lanes(&lanes_1);
    let balancer_gaps = compute_extra_gaps(&families_1);

    // Pass 2: re-place rows with the real bus width + any balancer
    // gaps. Retry gaps were already applied in pass 1, so they don't
    // gate pass 2 — but if pass 2 runs anyway, both sets are merged so
    // the second placement keeps the retry slack.
    let (row_entities, row_spans, row_width, total_height, lanes, families) =
        if actual_bw == temp_bw && balancer_gaps.is_empty() {
            (row_entities_1, row_spans_1, _row_width_1, total_height_1, lanes_1, families_1)
        } else {
            let merged_gaps: FxHashMap<usize, i32> = match retry_extra_gaps {
                None => balancer_gaps,
                Some(retry) => {
                    let mut merged = balancer_gaps;
                    for (k, v) in retry {
                        merged.entry(*k).and_modify(|cur| *cur += *v).or_insert(*v);
                    }
                    merged
                }
            };
            let t_place2 = web_time::Instant::now();
            let (re, rs, rw, th) = place_rows(
                &solver_result.machines,
                &solver_result.dependency_order,
                actual_bw,
                bus_header,
                max_belt_tier,
                Some(&final_output_items),
                Some(&merged_gaps),
                opts.row_layout,
            );
            crate::trace::emit(crate::trace::TraceEvent::PhaseTime {
                phase: "place_rows_2".to_string(),
                duration_ms: t_place2.elapsed().as_millis() as u64,
            });
            let t_plan2 = web_time::Instant::now();
            let (nl, nf) = plan_bus_lanes(solver_result, &rs, max_belt_tier, plan_ref, th)?;
            crate::trace::emit(crate::trace::TraceEvent::PhaseTime {
                phase: "plan_bus_lanes_2".to_string(),
                duration_ms: t_plan2.elapsed().as_millis() as u64,
            });
            (re, rs, rw, th, nl, nf)
        };

    crate::trace::emit(crate::trace::TraceEvent::PhaseComplete {
        phase: "rows_placed".into(),
        entity_count: row_entities.len(),
    });
    if crate::trace::is_active() {
        crate::trace::emit(crate::trace::TraceEvent::PhaseSnapshot {
            phase: "rows_placed".into(),
            entities: row_entities.clone(),
            width: row_width.max(actual_bw),
            height: total_height,
        });
    }
    crate::trace::emit(crate::trace::TraceEvent::PhaseComplete {
        phase: "lanes_planned".into(),
        entity_count: row_entities.len(),
    });
    if crate::trace::is_active() {
        crate::trace::emit(crate::trace::TraceEvent::PhaseSnapshot {
            phase: "lanes_planned".into(),
            entities: row_entities.clone(),
            width: row_width.max(actual_bw),
            height: total_height,
        });
    }

    // DIAGNOSTIC: dump the solver/row/lane fingerprint so native and
    // WASM runs can be compared side-by-side. Captures dependency_order
    // (solver output), row y-spans (placer output), and lane x-columns
    // (lane_planner output). If any of these diverge between targets,
    // everything downstream (ghost routing, junctions) will too.
    crate::trace::emit(crate::trace::TraceEvent::PipelineDiagnostics {
        dep_order: solver_result.dependency_order.clone(),
        rows: row_spans
            .iter()
            .map(|r| format!("{},{},{}", r.spec.recipe, r.y_start, r.y_end))
            .collect(),
        lanes: lanes
            .iter()
            .map(|l| format!("{},{},{:.2},{}", l.item, l.x, l.rate, l.is_fluid))
            .collect(),
    });

    // Place power poles from machine positions before routing so the router
    // sees them as hard obstacles. The occupied set reserves row-entity tiles
    // AND planned fluid-lane columns so poles don't land where the router
    // will later place pipe/PTG entities.
    let pole_entities: Vec<PlacedEntity> = {
        let mut row_occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut machines_for_poles: Vec<(i32, i32, i32)> = Vec::new();
        for ent in &row_entities {
            if MACHINE_ENTITIES.contains(&ent.name.as_str()) {
                let (mw, mh) = crate::common::machine_dims(&ent.name);
                let (mw, mh) = (mw as i32, mh as i32);
                for dx in 0..mw {
                    for dy in 0..mh {
                        row_occupied.insert((ent.x + dx, ent.y + dy));
                    }
                }
                // place_poles groups by row height and offsets the fallback
                // pole row by that height (below the machine row), so the
                // third field is height; the center x uses width.
                machines_for_poles.push((ent.x + mw / 2, ent.y, mh));
            } else {
                row_occupied.insert((ent.x, ent.y));
            }
        }
        for lane in &lanes {
            if !lane.is_fluid {
                continue;
            }
            let mut trunk_ys: Vec<i32> = vec![lane.source_y];
            trunk_ys.extend(lane.tap_off_ys.iter().copied());
            for &(_ri, _px, py) in &lane.fluid_output_port_positions {
                trunk_ys.push(py);
            }
            trunk_ys.sort_unstable();
            trunk_ys.dedup();
            for &y in &trunk_ys {
                row_occupied.insert((lane.x, y));
            }
            for pair in trunk_ys.windows(2) {
                let (y0, y1) = (pair[0], pair[1]);
                if y1 - y0 > 1 {
                    row_occupied.insert((lane.x, y0 + 1));
                }
                if y1 - y0 > 2 {
                    row_occupied.insert((lane.x, y1 - 1));
                }
            }
            let all_ports = lane.fluid_port_positions.iter()
                .chain(lane.fluid_output_port_positions.iter());
            for &(_ri, port_x, port_y) in all_ports {
                row_occupied.insert((port_x, port_y));
                row_occupied.insert((lane.x, port_y));
                let (lo, hi) = if port_x < lane.x {
                    (port_x, lane.x)
                } else {
                    (lane.x, port_x)
                };
                if hi - lo > 1 {
                    row_occupied.insert((lo + 1, port_y));
                }
                if hi - lo > 2 {
                    row_occupied.insert((hi - 1, port_y));
                }
            }
        }
        let pole_strategy = if machines_for_poles.is_empty() { "empty" } else { "rows" };
        let poles = place_poles(&machines_for_poles, &row_occupied);
        crate::trace::emit(crate::trace::TraceEvent::PolesPlaced {
            count: poles.len(),
            strategy: pole_strategy.to_string(),
        });
        // Stream sibling of PolesPlaced — carries the pole entity batch so
        // the live renderer can reveal them progressively instead of dumping
        // them via the poles_placed PhaseSnapshot safety net.
        if !poles.is_empty() {
            crate::trace::emit(crate::trace::TraceEvent::PolesCommitted {
                entities: poles.clone(),
            });
        }
        crate::trace::emit(crate::trace::TraceEvent::PhaseComplete {
            phase: "poles_placed".into(),
            entity_count: poles.len(),
        });
        poles
    };

    // Route all connecting belts via the ghost routing pipeline.
    // See `docs/ghost-pipeline-contracts.md` for the phase-by-phase
    // contract and `ghost_router.rs` for the implementation.
    let t_ghost = web_time::Instant::now();
    let ghost_result = crate::bus::ghost_router::route_bus_ghost(
        &lanes,
        &row_spans,
        total_height,
        actual_bw,
        max_belt_tier,
        solver_result,
        &families,
        &row_entities,
        &pole_entities,
    )?;
    let bus_entities = ghost_result.entities;
    let max_y = ghost_result.max_y;
    let merge_max_x = ghost_result.merge_max_x;
    let mut regions = ghost_result.regions;
    for (idx, region) in regions.iter_mut().enumerate() {
        region.id = idx as u32;
    }
    let ghost_warnings = ghost_result.warnings;
    let surplus_exits = ghost_result.surplus_exits;
    crate::trace::emit(crate::trace::TraceEvent::PhaseTime {
        phase: "ghost_routing".to_string(),
        duration_ms: t_ghost.elapsed().as_millis() as u64,
    });
    crate::trace::emit(crate::trace::TraceEvent::PhaseComplete {
        phase: "bus_routed".into(),
        entity_count: bus_entities.len(),
    });
    emit_inter_row_bands(&row_spans, &lanes);
    if crate::trace::is_active() {
        let mut snap_entities = row_entities.clone();
        snap_entities.extend(bus_entities.clone());
        crate::trace::emit(crate::trace::TraceEvent::PhaseSnapshot {
            phase: "bus_routed".into(),
            entities: snap_entities,
            width: row_width.max(actual_bw).max(merge_max_x),
            height: max_y,
        });
    }

    // Remove row entities that overlap with bus splitters
    let splitter_names: FxHashSet<&str> = ["splitter", "fast-splitter", "express-splitter"]
        .iter()
        .copied()
        .collect();
    let mut bus_occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
    for ent in &bus_entities {
        if splitter_names.contains(ent.name.as_str()) {
            bus_occupied.insert((ent.x, ent.y));
            if matches!(ent.direction, EntityDirection::West | EntityDirection::East) {
                bus_occupied.insert((ent.x, ent.y + 1));
            } else {
                bus_occupied.insert((ent.x + 1, ent.y));
            }
        }
    }
    let row_entities: Vec<PlacedEntity> = if bus_occupied.is_empty() {
        row_entities
    } else {
        row_entities.into_iter().filter(|e| !bus_occupied.contains(&(e.x, e.y))).collect()
    };

    let width = row_width.max(actual_bw).max(merge_max_x);

    // Emit a post-routing snapshot showing poles already placed before routing.
    if crate::trace::is_active() {
        let mut snap_entities = row_entities.clone();
        snap_entities.extend(bus_entities.clone());
        snap_entities.extend(pole_entities.clone());
        crate::trace::emit(crate::trace::TraceEvent::PhaseSnapshot {
            phase: "poles_placed".into(),
            entities: snap_entities,
            width,
            height: max_y,
        });
    }

    // Check for missing balancer templates and collect warnings
    let mut warnings = ghost_warnings;
    let templates = crate::bus::balancer_library::balancer_templates();
    for fam in &families {
        let (n, m) = (fam.shape.0 as u32, fam.shape.1 as u32);
        let has_direct = templates.contains_key(&(n, m));
        let has_decomp = (1..=n).rev().any(|g| {
            n % g == 0 && m % g == 0 && templates.contains_key(&(n / g, m / g))
        });
        if !has_direct && !has_decomp {
            warnings.push(format!(
                "No {}→{} balancer template for {}; producer outputs are disconnected",
                n, m, fam.item
            ));
        }
    }

    // Combine all entities: row_entities + bus_entities + pole_entities
    let mut all_entities = Vec::new();
    all_entities.extend(row_entities);
    all_entities.extend(bus_entities);
    all_entities.extend(pole_entities);

    // First-class, trace-independent ledger of voided solid surplus
    // (RFP Fulgora Phase 2, D1/D6) — mirrors `surplus_exits`.
    // `check_stranded_byproducts` cross-checks each entry against real
    // recycler entities rather than trusting this alone. Reconstructed
    // from the (possibly voider-synthesized) `solver_result` used by
    // this pass: `MachineSpec.inputs[0].rate` is the PER-MACHINE tap
    // rate (see `bus::voider::synthesize_voiders`), so `rate * count`
    // recovers the original surplus rate.
    let voided_streams: Vec<crate::models::VoidedStream> = solver_result
        .machines
        .iter()
        .filter(|m| m.voider)
        .filter_map(|m| {
            let inp = m.inputs.first()?;
            Some(crate::models::VoidedStream {
                item: inp.item.clone(),
                rate: inp.rate * m.count,
                machines: m.count.round().max(1.0) as usize,
                recipe: m.recipe.clone(),
            })
        })
        .collect();

    Ok((
        LayoutResult {
            entities: all_entities,
            width,
            height: max_y,
            warnings,
            regions,
            trace: None,
            surplus_exits,
            voided_streams,
        },
        row_spans,
    ))
}

/// Traced variant of [`build_bus_layout`].
///
/// Collects structured trace events through all pipeline phases and returns
/// them in `LayoutResult.trace`. Zero overhead when using the non-traced entry point.
pub fn build_bus_layout_traced(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String> {
    let _guard = crate::trace::start_trace();
    let mut result = build_bus_layout(solver_result, opts)?;
    result.trace = Some(crate::trace::drain_events());
    Ok(result)
}

/// Streaming variant — mirrors every emitted `TraceEvent` to `on_event` as it
/// happens, and also returns them in `LayoutResult.trace` at the end. Used by
/// the web app to render pipeline progress live while the engine runs.
pub fn build_bus_layout_streaming(
    solver_result: &SolverResult,
    opts: LayoutOptions,
    mut on_event: Box<dyn FnMut(&crate::trace::TraceEvent)>,
) -> Result<LayoutResult, String> {
    let _collector_guard = crate::trace::start_trace();
    let _sink_guard = crate::trace::set_sink(Box::new(move |evt| on_event(evt)));
    let mut result = build_bus_layout(solver_result, opts)?;
    result.trace = Some(crate::trace::drain_events());
    Ok(result)
}

/// Estimate bus width before full lane planning.
fn estimate_bus_width(solver_result: &SolverResult) -> i32 {
    // Count external solid inputs
    let n_external = solver_result
        .external_inputs
        .iter()
        .filter(|f| !f.is_fluid)
        .count() as i32;

    // Count intermediate items (items produced and consumed internally)
    let mut produced = FxHashSet::default();
    let mut consumed = FxHashSet::default();

    for m in &solver_result.machines {
        for out in &m.outputs {
            if !out.is_fluid {
                produced.insert(out.item.clone());
            }
        }
        for inp in &m.inputs {
            if !inp.is_fluid {
                consumed.insert(inp.item.clone());
            }
        }
    }

    let n_intermediate = produced.intersection(&consumed).count() as i32;
    let n_lanes = n_external + n_intermediate;
    (2).max(n_lanes * 2 + 1)
}

/// Compute extra gaps needed for balancer blocks.
fn compute_extra_gaps(families: &[LaneFamily]) -> FxHashMap<usize, i32> {
    let mut extra: FxHashMap<usize, i32> = FxHashMap::default();

    for fam in families {
        if fam.producer_rows.is_empty() {
            continue;
        }

        let n_producers = fam.shape.0;
        // Get template height from balancer library
        let (n, m) = (fam.shape.0 as u32, fam.shape.1 as u32);
        let templates = crate::bus::balancer_library::balancer_templates();
        let template_height = templates.get(&(n, m)).map(|t| t.height as i32)
            .or_else(|| {
                // Decomposition: find divisor g where (n/g, m/g) has a template.
                (1..=n).rev().find_map(|g| {
                    if n % g == 0 && m % g == 0 {
                        templates.get(&(n / g, m / g)).map(|t| t.height as i32)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(3);

        let needed = if n_producers == 1 {
            (template_height - 3).max(0)
        } else {
            (template_height - 2).max(0)
        };

        if needed == 0 {
            continue;
        }

        let last_producer = *fam.producer_rows.iter().max().unwrap();
        extra
            .entry(last_producer)
            .and_modify(|v| *v = (*v).max(needed))
            .or_insert(needed);
    }

    extra
}

/// Place medium electric poles for power coverage.
///
/// Strategy: one horizontal pole line per machine row. Within a line, poles
/// are placed by greedy forward sweep — for each machine not yet covered, we
/// choose the rightmost pole position that still covers it, then advance past
/// every machine the new pole reaches. This guarantees edge machines are
/// covered (which a fixed-stride approach cannot) while still producing
/// regularly-spaced poles.
///
/// Pole y for a row is `machine_row_y - 1` (one tile above the machine tops).
/// With a 3-tile supply range that covers machine centers one tile below the
/// pole line comfortably. The tile above the machine row is typically the
/// inserter row, which has gaps every ~3 tiles between inserters — the probe
/// finds those gaps.
///
/// Connectivity is guaranteed by construction:
/// - Within a line: consecutive pole x-distance <= 6 < `WIRE_REACH` (9).
/// - Between lines: row cycle (row height + gap) is typically ~7 tiles <
///   wire-reach, so pole lines above consecutive rows connect vertically.
///
/// The old greedy + centroid-bridge implementation produced clumpy, order-
/// dependent output; this approach is deterministic, regular, and matches the
/// row-based structure of the bus layout.
/// `machines` entries are `(center_x, top_y, height)` — height (not width)
/// because every use below (row grouping, below-row fallback y) is a
/// vertical offset from the machine row.
fn place_poles(
    machines: &[(i32, i32, i32)],
    occupied: &FxHashSet<(i32, i32)>,
) -> Vec<PlacedEntity> {
    /// Supply range of a medium-electric-pole (Chebyshev, tiles).
    const POLE_RANGE: i32 = 3;
    /// Max X offset to probe when the ideal pole position is occupied.
    /// Set to `2 * POLE_RANGE` so the search covers the full supply range
    /// either side of the machine center: the rightmost-first ordering
    /// keeps forward reach, but when every position from `ideal_px` down
    /// to `ideal_px - POLE_RANGE` is blocked, we keep probing leftward
    /// to `ideal_px - 2*POLE_RANGE = cx - POLE_RANGE`. Without this, a
    /// tight row whose right side is full of bridge belts (sideload
    /// balancer below the machine row) leaves the corresponding machine
    /// center uncovered even though a free tile exists to its left
    /// inside the supply range — the pre-fix algorithm gave up at d=3.
    const POLE_PROBE_X: i32 = POLE_RANGE * 2;

    if machines.is_empty() {
        return Vec::new();
    }

    // Group by (top_y, height). Rows of different-height machines get their
    // own pole lines because the pole y needs to match the machine footprint.
    let mut by_row: FxHashMap<(i32, i32), Vec<i32>> = FxHashMap::default();
    for &(cx, top_y, mh) in machines {
        by_row.entry((top_y, mh)).or_default().push(cx);
    }
    for xs in by_row.values_mut() {
        xs.sort_unstable();
    }

    // Process rows top-to-bottom for determinism.
    let mut keys: Vec<(i32, i32)> = by_row.keys().copied().collect();
    keys.sort_unstable();

    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut placed: FxHashSet<(i32, i32)> = FxHashSet::default();

    for key in keys {
        let (top_y, mh) = key;
        let cxs = &by_row[&key];

        // Two candidate pole rows: one above the machine row (preferred —
        // keeps poles in a single visible band) and one below as fallback
        // for dense templates (HS / TripleInput / FluidDualInput) where
        // the above-row is jammed with inserters/pipes. Both rows are
        // within POLE_RANGE of the machine center on the y axis.
        let mut candidate_ys: Vec<i32> = Vec::with_capacity(2);
        if top_y > 0 {
            candidate_ys.push(top_y - 1);
        }
        candidate_ys.push(top_y + mh);

        let mut i = 0;
        while i < cxs.len() {
            // Aim for the rightmost position that still covers cxs[i] — this
            // maximises forward reach and keeps the line sparse. Probing
            // searches nearby tiles if the ideal one is occupied, always
            // staying within POLE_RANGE of the target machine.
            let target_cx = cxs[i];
            let ideal_px = target_cx + POLE_RANGE;
            let mut placed_at: Option<(i32, i32)> = None;
            'outer: for &py in &candidate_ys {
                for d in 0..=POLE_PROBE_X {
                    let offsets: &[i32] = if d == 0 { &[0] } else { &[-d, d] };
                    for &off in offsets {
                        let px = ideal_px + off;
                        if (px - target_cx).abs() > POLE_RANGE {
                            continue; // stepped outside range of the target machine
                        }
                        if occupied.contains(&(px, py)) || placed.contains(&(px, py)) {
                            continue;
                        }
                        placed_at = Some((px, py));
                        break 'outer;
                    }
                }
            }

            match placed_at {
                Some((px, py)) => {
                    entities.push(make_pole(px, py));
                    placed.insert((px, py));
                    // Advance past every machine this pole covers. POLE_RANGE
                    // is Chebyshev — both candidate y rows are within range
                    // of the machine center, so the x check alone is enough.
                    i += 1;
                    while i < cxs.len() && (cxs[i] - px).abs() <= POLE_RANGE {
                        i += 1;
                    }
                }
                None => {
                    // Couldn't place a pole covering cxs[i]. Skip it to avoid an
                    // infinite loop — power validator will flag the gap.
                    i += 1;
                }
            }
        }
    }

    repair_pole_connectivity(&mut entities, &placed, occupied);
    entities
}

/// After the row lines are placed, bridge any remaining disconnected pole
/// clusters. This only fires when two machine rows are further apart in Y
/// than `WIRE_REACH` (e.g. oil-refinery row above a chemical-plant row with
/// a pipe-routing gap between them). We walk intermediate poles down a
/// free column between the two nearest clusters.
fn repair_pole_connectivity(
    entities: &mut Vec<PlacedEntity>,
    placed: &FxHashSet<(i32, i32)>,
    occupied: &FxHashSet<(i32, i32)>,
) {
    /// Wire reach in tiles. Matches `validate::power::MEDIUM_POLE_WIRE_REACH`.
    /// Two poles are connected iff `dx² + dy² ≤ WIRE_REACH²` (Euclidean) —
    /// using Chebyshev here as a proxy is a near-miss bug: poles 7 right and
    /// 6 down are Chebyshev=7 (looks connected) but Euclidean≈9.22 (actually
    /// disconnected per the validator).
    const WIRE_REACH: i32 = 9;
    const WIRE_REACH_SQ: i32 = WIRE_REACH * WIRE_REACH;

    let mut all_occupied: FxHashSet<(i32, i32)> = occupied.iter().copied().collect();
    for &p in placed {
        all_occupied.insert(p);
    }

    for _ in 0..20 {
        let positions: Vec<(i32, i32)> = entities.iter().map(|e| (e.x, e.y)).collect();
        if positions.len() <= 1 {
            return;
        }

        // Union-find under Chebyshev distance <= WIRE_REACH.
        let n = positions.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut [usize], mut x: usize) -> usize {
            while p[x] != x {
                p[x] = p[p[x]];
                x = p[x];
            }
            x
        }
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                if dx * dx + dy * dy <= WIRE_REACH_SQ {
                    let ri = find(&mut parent, i);
                    let rj = find(&mut parent, j);
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
            }
        }

        // Group by root component.
        let mut by_comp: FxHashMap<usize, Vec<(i32, i32)>> = FxHashMap::default();
        for (idx, &pos) in positions.iter().enumerate() {
            let root = find(&mut parent, idx);
            by_comp.entry(root).or_default().push(pos);
        }
        if by_comp.len() == 1 {
            return;
        }

        // Find the closest inter-component pole pair.
        let comps: Vec<&Vec<(i32, i32)>> = by_comp.values().collect();
        // Squared Euclidean for closest-pair selection — order is identical to
        // Euclidean and we avoid sqrt.
        let mut best: Option<((i32, i32), (i32, i32), i32)> = None;
        for a in 0..comps.len() {
            for b in (a + 1)..comps.len() {
                for &pa in comps[a] {
                    for &pb in comps[b] {
                        let dx = pa.0 - pb.0;
                        let dy = pa.1 - pb.1;
                        let d_sq = dx * dx + dy * dy;
                        if best.is_none_or(|(_, _, bd)| d_sq < bd) {
                            best = Some((pa, pb, d_sq));
                        }
                    }
                }
            }
        }
        let Some((pa, pb, _)) = best else {
            return;
        };

        // Pick a midpoint and walk outward in a small neighbourhood looking
        // for a free tile to place a bridge pole. The search radius must
        // reach at least `WIRE_REACH` so that for component pairs whose
        // midpoint is more than `WIRE_REACH` away from both endpoints
        // (i.e. when the gap between components is wider than `2 *
        // WIRE_REACH`), the scan can step *back* toward an endpoint and
        // find a tile that's both free and within wire reach of `pa` or
        // `pb`. With radius 6, gaps wider than 12 left the loop unable
        // to drop a first bridge pole and the components stayed
        // disconnected — see `tier4_advanced_circuit_from_ore_am2`,
        // where the pa↔pb gap is ~32 tiles.
        let mid = ((pa.0 + pb.0) / 2, (pa.1 + pb.1) / 2);
        let mut bridge: Option<(i32, i32)> = None;
        'scan: for r in 0i32..=WIRE_REACH {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue; // only examine the ring at radius r
                    }
                    let p = (mid.0 + dx, mid.1 + dy);
                    if all_occupied.contains(&p) {
                        continue;
                    }
                    // Must be within wire-reach of pa or pb for it to actually bridge.
                    let near = |q: (i32, i32)| -> bool {
                        let dx = p.0 - q.0;
                        let dy = p.1 - q.1;
                        dx * dx + dy * dy <= WIRE_REACH_SQ
                    };
                    if near(pa) || near(pb) {
                        bridge = Some(p);
                        break 'scan;
                    }
                }
            }
        }

        let Some(p) = bridge else { return };
        entities.push(make_pole(p.0, p.1));
        all_occupied.insert(p);
    }
}


/// Create a pole entity at the given position.
fn emit_inter_row_bands(row_spans: &[RowSpan], lanes: &[BusLane]) {
    if row_spans.len() < 2 {
        return;
    }
    let lane_extents: Vec<(i32, i32)> = lanes
        .iter()
        .map(|l| {
            let mut y_min = l.source_y;
            let mut y_max = l.source_y;
            for &ty in &l.tap_off_ys {
                y_min = y_min.min(ty);
                y_max = y_max.max(ty);
            }
            for &cr in &l.consumer_rows {
                if let Some(rs) = row_spans.get(cr) {
                    y_min = y_min.min(rs.y_start);
                    y_max = y_max.max(rs.y_end - 1);
                }
            }
            (y_min, y_max)
        })
        .collect();

    for i in 0..row_spans.len() - 1 {
        let upper = &row_spans[i];
        let lower = &row_spans[i + 1];
        // y_end is exclusive, so y_end is the first tile of the gap.
        let band_y_start = upper.y_end;
        let band_y_end = lower.y_start - 1;
        if band_y_end < band_y_start {
            continue;
        }
        let mut trunk_count = 0usize;
        let mut items: FxHashSet<&str> = FxHashSet::default();
        for (lane, &(y_min, y_max)) in lanes.iter().zip(lane_extents.iter()) {
            if y_min <= band_y_start && y_max >= band_y_end {
                trunk_count += 1;
                items.insert(lane.item.as_str());
            }
        }
        crate::trace::emit(crate::trace::TraceEvent::InterRowBand {
            upper_row_idx: i,
            lower_row_idx: i + 1,
            band_y_start,
            band_y_end,
            gap_height: band_y_end - band_y_start + 1,
            trunk_count,
            distinct_items: items.len(),
        });
    }
}

fn make_pole(x: i32, y: i32) -> PlacedEntity {
    PlacedEntity {
        name: "medium-electric-pole".to_string(),
        x,
        y,
        direction: EntityDirection::North,
        recipe: None,
        io_type: None,
        carries: None,
        mirror: false,
        segment_id: Some("pole".to_string()),
        ..Default::default()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_bus_width_empty() {
        let sr = SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let bw = estimate_bus_width(&sr);
        assert!(bw >= 2);
    }

    #[test]
    fn test_compute_extra_gaps_empty() {
        let extras = compute_extra_gaps(&[]);
        assert!(extras.is_empty());
    }

    /// D2a (RFP Fulgora, `docs/rfp-fulgora-scrap.md`): a solid surplus
    /// item whose producing row's FIRST (and only) solid output IS the
    /// surplus — distinct from D2b's secondary-belt shape
    /// (uranium-processing, `tier_uranium_processing_surplus_export` in
    /// `tests/e2e.rs`), which needs a second solid output on the SAME
    /// row. No organic e2e fixture exercises D2a in isolation today —
    /// Phase 3's scrap sorter is the first natural source of a
    /// same-recipe-family surplus without a second output slot — so
    /// this synthetic `SolverResult` is the intended coverage per the
    /// D2a/D2b PR review. Two independent SingleInput rows: one
    /// producing the target (`widget`), one producing a surplus with no
    /// consumer (`gadget-scrap`) as its ONLY output.
    #[test]
    fn d2a_solid_surplus_merges_without_overlapping_target() {
        use crate::models::{ItemFlow, MachineSpec};

        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-3".to_string(),
                    recipe: "widget".to_string(),
                    self_loop: vec![], voider: false,
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 2.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "widget".to_string(),
                        rate: 1.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "assembling-machine-3".to_string(),
                    recipe: "gadget-scrap".to_string(),
                    self_loop: vec![], voider: false,
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 2.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "gadget-scrap".to_string(),
                        rate: 3.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
            ],
            external_inputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 4.0,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![ItemFlow {
                item: "widget".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            surplus_outputs: vec![ItemFlow {
                item: "gadget-scrap".to_string(),
                rate: 3.0,
                is_fluid: false,
                module_id: 0,
            }],
            dependency_order: vec!["widget".to_string(), "gadget-scrap".to_string()],
        };

        let layout = build_bus_layout(&sr, LayoutOptions::default())
            .expect("D2a synthetic layout should build");

        // Surplus recorded and cross-checked against a real belt entity —
        // mirrors `check_stranded_byproducts`'s own acceptance logic.
        let exit = layout
            .surplus_exits
            .iter()
            .find(|(item, _, _)| item == "gadget-scrap");
        assert!(
            exit.is_some(),
            "expected a gadget-scrap surplus_exits entry, got {:?}",
            layout.surplus_exits
        );
        let &(_, ex, ey) = exit.unwrap();
        assert!(
            layout.entities.iter().any(|e| e.x == ex
                && e.y == ey
                && e.carries.as_deref() == Some("gadget-scrap")
                && crate::common::is_belt_entity(&e.name)),
            "expected a belt/splitter entity carrying gadget-scrap at the recorded exit tile ({ex},{ey})"
        );

        // No overlap between the target's own merge block and the
        // surplus merge block — the whole point of threading
        // merge_x_cursor/blocked_columns through Step 7b. Mirrors
        // `output_merger::test_two_items_merge_blocks_do_not_overlap`
        // at the full-layout level.
        let target_tiles: FxHashSet<(i32, i32)> = layout
            .entities
            .iter()
            .filter(|e| e.segment_id.as_deref() == Some("merger:widget"))
            .map(|e| (e.x, e.y))
            .collect();
        let surplus_tiles: FxHashSet<(i32, i32)> = layout
            .entities
            .iter()
            .filter(|e| e.segment_id.as_deref() == Some("merger:gadget-scrap"))
            .map(|e| (e.x, e.y))
            .collect();
        assert!(!target_tiles.is_empty(), "expected target merger tiles");
        assert!(!surplus_tiles.is_empty(), "expected surplus merger tiles");
        let overlap: Vec<_> = target_tiles.intersection(&surplus_tiles).collect();
        assert!(
            overlap.is_empty(),
            "target and surplus merge blocks overlap at {overlap:?}"
        );
    }

    /// Build a synthetic `RowSpan` with just the y-coordinates that
    /// `compute_retry_gaps` looks at. Other fields use minimal defaults.
    fn dummy_row_span(recipe: &str, y_start: i32, y_end: i32) -> RowSpan {
        use crate::models::MachineSpec;
        RowSpan {
            y_start,
            y_end,
            spec: MachineSpec {
                entity: "assembling-machine-1".to_string(),
                recipe: recipe.to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![],
                outputs: vec![],
            },
            machine_count: 1,
            module_id: 0,
            input_belt_y: vec![],
            output_belt_y: y_end,
            row_width: 0,
            fluid_port_ys: vec![],
            fluid_port_pipes: vec![],
            fluid_output_port_pipes: vec![],
            output_east: false,
            output_belt_x_min: 0,
            output_belt_x_max: 0,
            horizontal_stack: None,
            secondary_output_belt: None,
            sorted_output_belts: Vec::new(),
        }
    }

    fn three_row_layout() -> Vec<RowSpan> {
        vec![
            dummy_row_span("copper-plate", 1, 8),
            dummy_row_span("iron-plate", 15, 22),
            dummy_row_span("electronic-circuit", 30, 38),
        ]
    }

    #[test]
    fn compute_retry_gaps_no_caps_is_empty() {
        let spans = three_row_layout();
        let gaps = compute_retry_gaps(&[], &spans);
        assert!(gaps.is_empty());
    }

    #[test]
    fn compute_retry_gaps_cap_inside_row_widens_predecessor_gap() {
        let spans = three_row_layout();
        // Cap at y=31 lands inside electronic-circuit (row 2). The
        // heuristic widens the gap *before* row 2, i.e. after row 1.
        let gaps = compute_retry_gaps(&[(10, 31)], &spans);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps.get(&1), Some(&1));
    }

    #[test]
    fn compute_retry_gaps_cap_in_inter_row_band_attributes_to_next_row() {
        let spans = three_row_layout();
        // Cap at y=25 lands in the gap between row 1 (ends at 22) and
        // row 2 (starts at 30). `position(|s| y <= s.y_end)` matches
        // row 2 (since 25 <= 38), so we widen the gap before row 2.
        let gaps = compute_retry_gaps(&[(10, 25)], &spans);
        assert_eq!(gaps.get(&1), Some(&1));
    }

    #[test]
    fn compute_retry_gaps_cap_in_first_row_skips() {
        let spans = three_row_layout();
        // Cap at y=5 lands inside row 0 — there's no preceding row to
        // widen, so the cap is silently ignored.
        let gaps = compute_retry_gaps(&[(10, 5)], &spans);
        assert!(gaps.is_empty());
    }

    #[test]
    fn compute_retry_gaps_cap_below_last_row_skips() {
        let spans = three_row_layout();
        // Cap at y=100 falls past every row's y_end. No row matches
        // `y <= y_end`; the cap is silently ignored.
        let gaps = compute_retry_gaps(&[(10, 100)], &spans);
        assert!(gaps.is_empty());
    }

    #[test]
    fn compute_retry_gaps_multiple_caps_same_row_collapse_to_single_widen() {
        let spans = three_row_layout();
        // Two caps both inside row 2. Both widen the same predecessor
        // gap; the resulting map has one entry with value 1 (max, not
        // sum — caps can fire multiple times for one geometry issue).
        let gaps = compute_retry_gaps(&[(5, 31), (12, 35)], &spans);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps.get(&1), Some(&1));
    }

    #[test]
    fn compute_retry_gaps_caps_in_different_rows_widen_each_predecessor() {
        let spans = three_row_layout();
        // One cap in row 1, one in row 2. Each widens its own
        // predecessor gap (rows 0 and 1 respectively).
        let gaps = compute_retry_gaps(&[(5, 18), (12, 35)], &spans);
        assert_eq!(gaps.len(), 2);
        assert_eq!(gaps.get(&0), Some(&1));
        assert_eq!(gaps.get(&1), Some(&1));
    }

    #[test]
    fn compute_retry_gaps_empty_row_spans_returns_empty() {
        let gaps = compute_retry_gaps(&[(10, 20)], &[]);
        assert!(gaps.is_empty());
    }

}
