//! Bus layout orchestrator: rows + bus lanes + poles -> LayoutResult.
//!
//! Entry point: [`build_bus_layout`]. Calls `place_rows` to stack
//! assembly rows, `plan_bus_lanes` to decide which items need which
//! trunks, and `route_bus_ghost` to materialise every connecting belt
//! via the ghost-routing pipeline. See `docs/ghost-pipeline-contracts.md`
//! for the phase-by-phase invariants the router promises.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::models::{EntityDirection, LayoutResult, PlacedEntity, SolverResult};
use crate::bus::inserter_ladder::InserterTier;
use crate::bus::lane_planner::{
    plan_bus_lanes, bus_width_for_lanes, BusLane, LaneFamily,
};
use crate::common::{is_inserter, is_machine_entity};
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
    /// Hard cap on inserter tier the sizing ladder may place
    /// (`docs/rfp-inserter-sizing.md`), mirroring `max_belt_tier`
    /// semantics. Default `Stack`. Core field only in Phase 1 — not yet
    /// plumbed through wasm-bindings or the web UI (Phase 4).
    pub max_inserter_tier: InserterTier,
    /// Build quality of the entities the engine places
    /// (`docs/rfp-build-quality.md` Phase 2): scales the sizing ladder's
    /// inserter ceilings and pole geometry, and is stamped onto
    /// functional entities (machines/inserters/poles) for export.
    /// Default `Normal` — a bit-exact no-op (kill criterion 2).
    pub quality: crate::common::QualityTier,
    /// Enable the merge-and-tap trunk fallback for unstampable
    /// multi-producer/multi-consumer families (`docs/rfp-merge-tap-trunks.md`).
    /// Default `false` (byte-identical to pre-fallback layouts). Set only by
    /// `bus::decomposition_search::MergeTapCandidate`; never by the default
    /// `NativeCandidate`. Replaces the retired `MERGE_TAP_FALLBACK_ENABLED`
    /// compile-time const.
    pub merge_tap: bool,
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
            max_inserter_tier: InserterTier::default(),
            quality: crate::common::QualityTier::default(),
            merge_tap: false,
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
/// invokes `layout_pass`, reads the junction-cap tiles it returns,
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
    // Snapshot the trace collector length before the first pass. This is
    // now used ONLY to present a clean event stream to a streaming consumer
    // (replay pass-1 events if we don't retry, truncate them if we do) —
    // retry DETECTION reads cap coords from layout_pass's return, not the
    // collector, so it works whether or not tracing is active.
    let trace_start = crate::trace::peek_events_len();

    // Detach the active sink (if any) for pass 1, so a streaming consumer
    // doesn't see events from a layout pass that gets abandoned by retry.
    // Reinstalled (or replayed-into) below depending on whether we retry.
    let original_sink = crate::trace::swap_sink(None);

    // Cap coordinates come back as DATA from layout_pass (the ghost router's
    // GhostRouteResult.cap_coords), not scraped from the trace collector.
    // The old code read `JunctionGrowthCapped` out of `peek_events_since`,
    // which only sees events when a trace collector is active — so retries
    // fired only under a trace guard and `build_bus_layout` was not a pure
    // function of its arguments (package #3). The event is still emitted for
    // the snapshot debugger; this reads the control-flow copy instead.
    let (result_1, row_spans_1, cap_coords, uncovered_1) =
        layout_pass(solver_result, opts, None, None, 0, explicit_plan)?;

    // Two independent gap sources feed ONE pass-2 re-run (RFP Phase 3a-ii).
    // Junction-cap gaps (existing) widen rows the junction solver couldn't pack;
    // substation bands (new) widen the cycle boundary above a deep-packed
    // inserter row so a substation can reach it. Both derive from pass-1 data
    // and merge into the single row→extra-tiles map `place_rows` consumes; the
    // substation bands are ALSO threaded to pass 2's `place_poles` (typed) so it
    // powers them with a substation, never medium poles.
    let junction_gaps = if cap_coords.is_empty() {
        FxHashMap::default()
    } else {
        compute_retry_gaps(&cap_coords, &row_spans_1)
    };
    let substation_bands = compute_substation_bands(&uncovered_1, &row_spans_1);

    if junction_gaps.is_empty() && substation_bands.is_empty() {
        // No retry — replay pass-1 events from the collector to the
        // original sink so the streaming consumer sees the same events
        // it would have seen without the silent-pass wrapper. Untraced,
        // the sink is None and this block is a no-op.
        if let Some(mut sink) = original_sink {
            for evt in &crate::trace::peek_events_since(trace_start) {
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

    // Merge both INTERIOR gap sources into the single map. A row that needs both
    // takes the larger widening — a substation band's 2 tiles already give the
    // junction solver its ≥1 tile of extra room, so `max` (not sum) keeps the
    // freed band tight while satisfying both. Top-edge bands (RFP Phase 3b) go
    // through a separate channel — they have no `extra_gap_after_row` predecessor
    // to widen, so they bump `layout_pass`'s y-offset instead (below).
    let mut merged_gaps = junction_gaps.clone();
    for b in substation_bands.iter().filter(|b| !b.top_edge) {
        merged_gaps
            .entry(b.row_after)
            .and_modify(|v| *v = (*v).max(b.extra))
            .or_insert(b.extra);
    }
    // Top-edge widen (RFP Phase 3b): the largest top-edge band's extra rows are
    // inserted before row 0 as a y-offset bump. `max` because every top-edge
    // band anchors at row 0 (the only row with no predecessor); a single freed
    // band above the layout serves them all.
    let top_widen: i32 = substation_bands
        .iter()
        .filter(|b| b.top_edge)
        .map(|b| b.extra)
        .max()
        .unwrap_or(0);

    let mut gaps_vec: Vec<(usize, i32)> = merged_gaps.iter().map(|(k, v)| (*k, *v)).collect();
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

    let subs = (!substation_bands.is_empty()).then_some(substation_bands.as_slice());
    let (result_2, _, _, uncovered_2) =
        layout_pass(solver_result, opts, Some(&merged_gaps), subs, top_widen, explicit_plan)?;

    // Convergence guard (RFP `docs/rfp-power-reservation.md` Phase 3a-ii review
    // followup). The reactive pass widened every starved band; if `place_poles`
    // STILL gives up on any electric inserter, the repair did NOT converge and
    // this layout ships power-broken. Every corpus fixture converges (the four
    // gating pins + kovarex + USP all reach zero uncovered — the pins lock
    // that), so a non-empty set here is a genuinely-new starved geometry with no
    // pinning fixture. Emit a loud, release-surviving signal: a trace event
    // (lands in snapshots / drives a scoreboard) plus an env-gated eprintln for
    // local runs. NOT a `debug_assert` — release builds skip those and would
    // ship the break silently, exactly the "new starved case ships uncovered
    // without an alarm" hole the review flagged. This block is skipped whenever
    // the set is empty (the converging path, i.e. the entire corpus), so it adds
    // zero entities and leaves goldens byte-identical.
    if !uncovered_2.is_empty() {
        let mut sample = uncovered_2.clone();
        sample.sort_unstable();
        sample.truncate(16);
        crate::trace::emit(crate::trace::TraceEvent::ReactivePassNotConverged {
            uncovered_count: uncovered_2.len(),
            sample,
        });
        if std::env::var("SPAGHETTIO_WARN_ON_STDERR").is_ok() {
            eprintln!(
                "spaghettio: reactive power-repair pass did NOT converge — {} electric \
                 inserter(s) still uncovered after widening. This layout ships \
                 power-broken; a new starved geometry needs a pinning fixture. See \
                 docs/rfp-power-reservation.md Phase 3c. sample={:?}",
                uncovered_2.len(),
                uncovered_2.iter().take(8).collect::<Vec<_>>(),
            );
        }
    }
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

/// A pass-2 substation band (RFP `docs/rfp-power-reservation.md` Phase 3a-ii):
/// widen the gap AFTER row `row_after` (row_spans index) by `extra` tiles so a
/// substation can be dropped into the freed space to reach the deep-packed
/// inserter row that follows — a row whose input inserters sit at distance ≥4
/// from every physically possible medium-pole position (the packed-cycle wall
/// Phase 0f proved). This is a distinct TYPE from the junction-retry gaps: both
/// feed the single row→extra-tiles map `place_rows` consumes, but only these
/// are threaded to `place_poles` (via `SubstationTarget`) so the freed band is
/// powered by a substation's 18×18 supply, never medium poles — the distinction
/// is carried, never inferred from a coordinate coincidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SubstationBand {
    /// Row-spans index this band is anchored to. For an interior band
    /// (`top_edge == false`, RFP Phase 3a-ii) it is the PREDECESSOR row whose
    /// successor gap is widened: the freed rows land between `row_after` and
    /// `row_after + 1`, powering `row_after + 1`'s deep input inserters. For a
    /// top-edge band (`top_edge == true`, RFP Phase 3b — the kovarex self-loop)
    /// it is the STARVED row itself (row 0): the widen has no predecessor gap to
    /// consume, so it is applied as a y-offset bump and the freed rows land
    /// between the bus header and `row_after`'s top, powering `row_after`'s own
    /// input inserters.
    row_after: usize,
    extra: i32,
    /// True when this band widens the STARVED row's own top edge (row 0, no
    /// usable predecessor cycle). Threaded to `layout_pass` as a y-offset bump
    /// rather than an `extra_gap_after_row` entry, and resolved to a
    /// `SubstationTarget` against the row's own input-inserter band (which sits
    /// deeper than the RFP-assumed `y_start..y_start+4` — the self-loop stacks
    /// 5 belt/corridor rows above its inserters, so only a substation's ±9
    /// supply reaches down from the top freed band).
    top_edge: bool,
}

/// A widened substation band resolved to pass-2 coordinates and the inserter
/// row it must power. Built in the pole block (which holds the pass-2
/// `row_spans`) and handed to `place_poles`. `[band_y0, band_y1]` are the freed
/// rows (inclusive); `inserters` is the deep input-inserter band of the row
/// below, which the substation's supply must cover under the exact continuous
/// check.
struct SubstationTarget {
    band_y0: i32,
    band_y1: i32,
    inserters: Vec<(i32, i32)>,
}

/// Map `place_poles`' pass-1 uncovered-inserter set to the cycle boundaries
/// whose gap a substation band must widen (RFP Phase 3a-ii + 3b). For each
/// uncovered inserter, find the row that contains it and flag the gap BEFORE
/// that row: inserting free rows there lifts a band above the row's input belts
/// WITHOUT breaking pick adjacency (the free space lands at the cycle boundary,
/// never between belts and inserters — inserting there would sever the inserter
/// from the belt it picks from).
///
/// Two variants (RFP `docs/rfp-power-reservation.md`):
/// - **Interior** (3a-ii): `target > 0` — widen the gap after `target-1`
///   (`SubstationBand { row_after: target-1, top_edge: false }`), lifting the
///   band above `target`'s input belts. Powers `target`'s deep inserters.
/// - **Top edge** (3b, kovarex self-loop): `target == 0` — the starved row has
///   no predecessor gap. Widen the row's OWN top edge
///   (`SubstationBand { row_after: 0, top_edge: true }`): a y-offset bump frees
///   rows between the bus header and row 0, and a substation dropped there
///   reaches DOWN over the row's input inserters (which the self-loop stacks
///   5 belt/corridor rows below the top edge — beyond a medium pole's ±3, so
///   only the substation's ±9 supply reaches them).
///
/// One band per starved row; deduped and sorted. Empty for every layout the
/// two-band + mop-up covers, so non-starved layouts never enter pass 2 on this
/// account.
fn compute_substation_bands(
    uncovered: &[(i32, i32)],
    row_spans: &[RowSpan],
) -> Vec<SubstationBand> {
    // A substation is 2×2; two freed rows above the input belts give it a
    // footprint that routing rarely fully consumes, while keeping vertical
    // supply reach (±9 from center) comfortably over the input-inserter row a
    // few tiles below. Held small on purpose — the freed rows are pure
    // y-translation cost (movement-budget criterion). Pinned by the four gating
    // fixtures + the kovarex self-loop.
    //
    // ZERO-MARGIN WARNING (RFP Phase 3a-ii close-out): this +2 widen was tuned
    // so the four interior gating fixtures clear via ordinary MEDIUM poles, not
    // substations — the freed band lands its covering pole at medium distance
    // EXACTLY 3 (the electronic-circuit dual-input row has 2 belt rows, not the
    // RFP-assumed 3, so +2 is just enough). That is edge-tight with ZERO margin:
    // a template author who adds a belt row to a dual-input row, or shifts an
    // inserter one tile deeper, tips distance 3→4 and re-uncovers those inserters
    // — and 4 is outside the medium ±3, so the medium mop-up can't recover it.
    // The only guard is the four `assert_warnings_exactly([(power, 0)])` pins
    // (which flip loudly) plus the substation FALLBACK below (which fires only
    // for inserters STILL 0/49-free after widening). If you change belt-row count
    // or inserter depth in a dual-input template, re-run those pins and expect to
    // re-derive this constant. Do NOT raise it blindly to buy margin: every extra
    // tile is pure y-cost paid by the four fixtures whether or not they need it.
    const SUBSTATION_BAND_TILES: i32 = 2;
    let mut interior_rows_after: FxHashSet<usize> = FxHashSet::default();
    let mut top_edge_rows: FxHashSet<usize> = FxHashSet::default();
    for &(_x, y) in uncovered {
        // The row that physically contains this inserter.
        let Some(target) = row_spans.iter().position(|s| y >= s.y_start && y < s.y_end) else {
            continue;
        };
        if target > 0 {
            // Interior: widen the gap before `target` (after `target-1`).
            interior_rows_after.insert(target - 1);
        } else {
            // Top edge (RFP Phase 3b): row 0 has no predecessor gap — widen its
            // own top edge instead of skipping it (which is what left the
            // kovarex self-loop's 16 recirc inserters uncovered through 3a-ii).
            top_edge_rows.insert(target);
        }
    }
    let mut bands: Vec<SubstationBand> = interior_rows_after
        .into_iter()
        .map(|row_after| SubstationBand { row_after, extra: SUBSTATION_BAND_TILES, top_edge: false })
        .chain(
            top_edge_rows
                .into_iter()
                .map(|row_after| SubstationBand { row_after, extra: SUBSTATION_BAND_TILES, top_edge: true }),
        )
        .collect();
    // Sort by anchor row, top-edge bands last on ties (they use a distinct
    // widen channel, so co-anchoring is harmless — deterministic ordering only).
    bands.sort_by_key(|b| (b.row_after, b.top_edge));
    bands
}

/// One layout attempt — the body of the original `build_bus_layout`.
/// Takes an optional `retry_extra_gaps` map (row index → extra tiles)
/// that the retry loop in `build_bus_layout` uses to widen specific
/// row boundaries on a second pass. `None` on the first pass; `Some`
/// on the retry. `top_widen` (RFP Phase 3b) inserts that many free rows BEFORE
/// row 0 by bumping the row y-offset — the top-edge substation band's channel,
/// separate from the interior `extra_gap_after_row` map (row 0 has no
/// predecessor gap to widen). `0` on the first pass and for every layout without
/// a top-edge starved row. Returns
/// `(layout, row_spans, cap_coords, uncovered_inserters)`:
/// `uncovered_inserters` is place_poles' give-up set, the RFP Phase 3a-ii
/// reactive-substation trigger (empty for every non-starved layout). The retry
/// loop maps `cap_coords` (junction-solver cap tiles, carried as data
/// rather than scraped from the trace stream) back to row indices via
/// `row_spans` to build the gap map.
fn layout_pass(
    solver_result: &SolverResult,
    opts: &LayoutOptions,
    retry_extra_gaps: Option<&FxHashMap<usize, i32>>,
    substation_bands: Option<&[SubstationBand]>,
    top_widen: i32,
    explicit_plan: Option<&crate::bus::partitioner::PartitionPlan>,
) -> Result<(LayoutResult, Vec<RowSpan>, Vec<(i32, i32)>, Vec<(i32, i32)>), String> {
    let max_belt_tier = opts.max_belt_tier.as_deref();
    let max_inserter_tier = opts.max_inserter_tier;

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
    // Row placement y-origin. On the top-edge substation retry (RFP Phase 3b)
    // `top_widen > 0` bumps it below `bus_header`, freeing `top_widen` rows
    // between the bus header and row 0 for a substation that reaches down over
    // the row's packed input inserters. `bus_header` itself stays the freed
    // band's top edge (`band_y0`) for the target-resolution below.
    let row_y_origin = bus_header + top_widen;

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
    // Marks the start of pass-1 template events. If the width-corrected
    // pass 2 runs, pass 1's `InserterSideCapped` events are scrubbed —
    // their machine coordinates describe the discarded placement and
    // would mis-anchor the per-tile attribution join.
    let pass1_events_start = crate::trace::peek_events_len();
    let t_place1 = web_time::Instant::now();
    let (row_entities_1, row_spans_1, _row_width_1, total_height_1) = place_rows(
        &solver_result.machines,
        &solver_result.dependency_order,
        temp_bw,
        row_y_origin,
        max_belt_tier,
        max_inserter_tier,
        opts.quality,
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
        plan_bus_lanes(solver_result, &row_spans_1, max_belt_tier, plan_ref, total_height_1, opts.merge_tap)?;
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
            // Pass 2 replaces pass 1's placement wholesale; drop pass 1's
            // capped-side events so the trace only anchors at machines
            // that exist (other pass-1 events deliberately remain — the
            // phase timeline shows both passes).
            crate::trace::remove_capped_events_since(pass1_events_start);
            let t_place2 = web_time::Instant::now();
            let (re, rs, rw, th) = place_rows(
                &solver_result.machines,
                &solver_result.dependency_order,
                actual_bw,
                row_y_origin,
                max_belt_tier,
                max_inserter_tier,
                opts.quality,
                Some(&final_output_items),
                Some(&merged_gaps),
                opts.row_layout,
            );
            crate::trace::emit(crate::trace::TraceEvent::PhaseTime {
                phase: "place_rows_2".to_string(),
                duration_ms: t_place2.elapsed().as_millis() as u64,
            });
            let t_plan2 = web_time::Instant::now();
            // Pass 2 is the real plan; pass 1 (always run) already recorded the
            // pass-invariant `MergeTapFallback` event, so suppress it here to
            // dedup the double-emit while keeping pass 2's other events.
            let (nl, nf) = crate::trace::with_merge_tap_fallback_suppressed(|| {
                plan_bus_lanes(solver_result, &rs, max_belt_tier, plan_ref, th, opts.merge_tap)
            })?;
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

    // Power poles are placed LAST — after ghost routing — so they occupy only
    // tiles left free by the FINAL routed layout and can never obstruct a belt
    // or pipe (RFP `docs/rfp-power-supply.md`: "poles are placed last and live
    // off leftover tiles"). Phase 0f restored this invariant: pole-first meant
    // the doubled inserter-coverage poles won tile contests against the router
    // and broke belt routing (census_logistic_science_pack). See the
    // post-routing block below.

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
    // Junction-cap seed tiles, carried as data (not scraped from the trace
    // stream) so `run_layout_with_retry_inner` detects caps whether or not a
    // trace collector is active — see that function and package #3.
    let cap_coords = ghost_result.cap_coords;
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

    // Poles LAST: place them on tiles free of the FINAL routed layout — row
    // entities + the routed bus (belts, undergrounds, pipes, splitters) — so a
    // pole can never sit under a belt/pipe or force the router around it. The
    // mop-up now sees the true final occupancy, so coverage and hardness are
    // consistent by construction (RFP Phase 0f fix — restores the poles-last
    // invariant the pipeline had been violating).
    // The pole block also yields the uncovered electric inserters place_poles
    // gave up on (RFP Phase 3a-ii trigger), propagated out of layout_pass so
    // the reactive substation pass can widen starved bands. Empty for every
    // layout the two-band + mop-up covers.
    let (pole_entities, uncovered_inserters_out): (Vec<PlacedEntity>, Vec<(i32, i32)>) = {
        let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut machines_for_poles: Vec<(i32, i32, i32)> = Vec::new();
        let mut inserters_for_poles: Vec<(i32, i32)> = Vec::new();
        // Machines + electric inserters are the coverage subjects; biochamber
        // (burner) rows stay pole-covered for their electric inserters (Phase
        // 0c). Footprints go into `occupied` so poles never overlap them.
        for ent in &row_entities {
            if is_machine_entity(&ent.name) {
                let (mw, mh) = crate::common::machine_dims(&ent.name);
                let (mw, mh) = (mw as i32, mh as i32);
                for dx in 0..mw {
                    for dy in 0..mh {
                        occupied.insert((ent.x + dx, ent.y + dy));
                    }
                }
                machines_for_poles.push((ent.x + mw / 2, ent.y, mh));
            } else {
                occupied.insert((ent.x, ent.y));
                if is_inserter(&ent.name) {
                    inserters_for_poles.push((ent.x, ent.y));
                }
            }
        }
        // The routed bus is now REAL occupancy — this replaces the old
        // pre-routing fluid-lane reservation (routed pipes ARE the reservation)
        // and adds the solid belt/tap corridors that were never reserved
        // before (the pole-first obstruction bug).
        for ent in &bus_entities {
            occupied.insert((ent.x, ent.y));
            if crate::common::is_splitter(&ent.name) {
                let (sx, sy) = crate::common::splitter_second_tile(ent);
                occupied.insert((sx, sy));
            }
        }
        // Resolve each pass-2 substation band to concrete tile coordinates and
        // the deep input-inserter band it must power (RFP Phase 3a-ii + 3b).
        // Empty for every non-starved layout, so `place_poles` stays
        // byte-identical there.
        // A tile is genuinely packed (medium-unreachable) iff its whole 7×7 is
        // occupied against the routed layout — the RFP Phase 0f hardness
        // signature. A substation is placed only for inserters STILL unreachable
        // after the boundary is widened.
        let is_packed = |ix: i32, iy: i32| -> bool {
            (-3i32..=3).all(|dy| (-3i32..=3).all(|dx| occupied.contains(&(ix + dx, iy + dy))))
        };
        // Topmost machine row (row 0's machine top). Bounds the input-inserter
        // band for a top-edge band — inserters above it are inputs, at/below it
        // are the machines and their output inserters (already reachable from the
        // south band). `None` only when there are no machines (place_poles then
        // early-returns anyway).
        let machine_top = machines_for_poles.iter().map(|&(_, ty, _)| ty).min();
        let substation_targets: Vec<SubstationTarget> = match substation_bands {
            None => Vec::new(),
            Some(bands) => bands
                .iter()
                .filter_map(|b| {
                    if b.top_edge {
                        // Top-edge band (RFP Phase 3b, kovarex self-loop): the
                        // freed rows sit between the bus header and the starved
                        // row's top; the substation drops there and reaches DOWN
                        // over the row's OWN input inserters (which live ~5 rows
                        // below the top edge — beyond a medium pole's ±3, exactly
                        // why the top-edge variant needs the ±9 substation).
                        let row = row_spans.get(b.row_after)?;
                        let machine_top = machine_top?;
                        let band_y0 = bus_header;
                        let band_y1 = row.y_start - 1;
                        if band_y1 < band_y0 {
                            return None;
                        }
                        let inserters: Vec<(i32, i32)> = inserters_for_poles
                            .iter()
                            .copied()
                            // Every input inserter of the starved row: above the
                            // machine row, below the freed band. The self-loop's
                            // recirc inserters span two of these rows (top_y-1/-2),
                            // deeper than the interior band's y_start..+4 window.
                            .filter(|&(_x, y)| y >= row.y_start && y < machine_top)
                            .filter(|&(ix, iy)| is_packed(ix, iy))
                            .collect();
                        if inserters.is_empty() {
                            return None;
                        }
                        Some(SubstationTarget { band_y0, band_y1, inserters })
                    } else {
                        // Interior band (RFP Phase 3a-ii): the freed gap between
                        // row `row_after` and its successor; the target inserters
                        // are the successor row's top inserter band (y_start..+4).
                        // On the current corpus this yields zero targets — the +2
                        // widen lands the freed rows exactly 3 tiles above the deep
                        // inserters (the dual-input belt bundle is 2 rows, not the
                        // RFP-assumed 3), inside a medium pole's ±3, so the 7×7
                        // gains a free tile and the medium mop-up covers them. The
                        // substation branch is the VERIFIED-CORRECT FALLBACK for
                        // genuinely deeper geometry where an inserter stays >3 from
                        // every free tile even after widening.
                        let after = row_spans.get(b.row_after)?;
                        let below = row_spans.get(b.row_after + 1)?;
                        let band_y0 = after.y_end;
                        let band_y1 = below.y_start - 1;
                        if band_y1 < band_y0 {
                            return None;
                        }
                        let inserters: Vec<(i32, i32)> = inserters_for_poles
                            .iter()
                            .copied()
                            .filter(|&(_x, y)| y >= below.y_start && y < below.y_start + 4)
                            .filter(|&(ix, iy)| is_packed(ix, iy))
                            .collect();
                        if inserters.is_empty() {
                            return None;
                        }
                        Some(SubstationTarget { band_y0, band_y1, inserters })
                    }
                })
                .collect(),
        };
        let pole_strategy = if machines_for_poles.is_empty() { "empty" } else { "rows" };
        let (poles, uncovered) = place_poles(
            &machines_for_poles,
            &inserters_for_poles,
            &occupied,
            &substation_targets,
            opts.quality,
        );
        crate::trace::emit(crate::trace::TraceEvent::PolesPlaced {
            count: poles.len(),
            strategy: pole_strategy.to_string(),
        });
        if !poles.is_empty() {
            crate::trace::emit(crate::trace::TraceEvent::PolesCommitted {
                entities: poles.clone(),
            });
        }
        crate::trace::emit(crate::trace::TraceEvent::PhaseComplete {
            phase: "poles_placed".into(),
            entity_count: poles.len(),
        });
        (poles, uncovered)
    };

    let width = row_width.max(actual_bw).max(merge_max_x);

    // Post-routing snapshot with the poles now placed on leftover tiles.
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
        // Merge-tap families are stamped by the splitter merge-tree
        // (`stamp_merge_tap_family`), not the balancer library, so the library
        // lookup below is the wrong question for them — every K=1 sub-family
        // has shape (1, 1), which the library has no entry for, producing a
        // phantom "No 1→1 balancer template" warning even though merge_tree(1)
        // (a passthrough belt) stamped fine. Mirror the `if fam.merge_tap`
        // dispatch in ghost_router step 3. A GENUINE merge-tap stamp failure
        // (empty lane_xs → empty entity vec) is NOT hidden by this skip: it
        // still surfaces via `BalancerStamped { template_found: false }` and
        // `check_balancer_template_coverage`, the trace-based path.
        if fam.merge_tap {
            continue;
        }
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

    // First-class per-row spec attribution (see `models::EffectiveRow`) —
    // built from the same `row_spans` this pass actually placed, so it
    // reflects the (possibly voider-synthesized, possibly partition-
    // split) `solver_result` in scope here rather than whatever
    // pre-transform SolverResult a caller happens to pass to `validate`.
    let effective_rows: Vec<crate::models::EffectiveRow> = row_spans
        .iter()
        .map(|rs| crate::models::EffectiveRow {
            y_start: rs.y_start,
            y_end: rs.y_end,
            spec: rs.spec.clone(),
        })
        .collect();

    // Stamp build quality on functional entities (rfp-build-quality
    // Phase 2, functional-only stamping): machines, inserters, poles get
    // the planning tier; logistics stay `None`. Skipped entirely at
    // Normal — absent means normal everywhere downstream (export omits
    // the field, validators default absent → Normal), so the default
    // path is untouched (kill criterion 2). One post-pass here rather
    // than at the ~400 construction sites: single place to audit, and
    // Phase 3 per-class overrides become a per-class map lookup here.
    if opts.quality != crate::common::QualityTier::Normal {
        for e in &mut all_entities {
            if crate::common::quality_affects_entity(&e.name) {
                e.quality = Some(opts.quality);
            }
        }
    }

    // Pole copper wire graph for the web overlay — the SAME graph
    // `blueprint::export` re-derives and encodes in the blueprint `wires`
    // array. Computed from the final entity order so the `(a, b)` index pairs
    // stay valid — and AFTER the quality stamp pass above, because wire
    // reach is per-entity quality-aware (rfp-build-quality merge with the
    // power-3c arc). See `crate::power_wires`.
    let power_wires = crate::power_wires::compute_pole_wires(&all_entities);

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
            effective_rows,
            power_wires,
        },
        row_spans,
        cap_coords,
        uncovered_inserters_out,
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

/// Place power poles for grid coverage. Runs LAST in `build_bus_layout`, after
/// `route_bus_ghost` — poles live on leftover tiles and are never router
/// obstacles (invariant restored in Phase 0f, `docs/rfp-power-supply.md`).
///
/// For any `substation_targets` (RFP `docs/rfp-power-reservation.md` Phase
/// 3a-ii/3b) a substation is dropped into each widened band FIRST, so its 18×18
/// supply reaches the deep-packed inserter rows no medium pole can; every other
/// row is covered by medium-electric-poles. `substation_targets` is empty for
/// every layout the two-band + mop-up covers, keeping those byte-identical.
///
/// Medium-pole strategy (post-0f): for each machine row, seed TWO candidate
/// bands from the shared `common::pole_candidate_ys` — a north band on the
/// input-inserter row (covers the machine's north face) and a south band on the
/// output-inserter row (south face) — and search each outward, AWAY from the
/// machine, so a saturated inserter/belt band is still covered from the first
/// free row beyond it. A single band would leave the opposite inserter row at
/// Chebyshev distance `mh+1` (=4 for a 3×3 machine, one tile past the ±3 supply
/// area) — the systemic 40-52% uncovered-inserter signature Phase 0c measured,
/// now that electric inserters (not just machine centers) are coverage
/// subjects. Within a band a greedy forward sweep places a pole in `cx-2..cx+2`
/// (rightmost-first, covering the whole inserter footprint) with a center-only
/// fallback at `cx±pole_range`, then advances past every machine the pole still
/// covers.
///
/// Connectivity is guaranteed by construction:
/// - Within a line: consecutive pole x-distance <= 6 < the medium wire reach (9 at Normal).
/// - Between lines: row cycle (row height + gap) is typically ~7 tiles <
///   wire-reach, so pole lines above consecutive rows connect vertically.
///
/// The old greedy + centroid-bridge implementation produced clumpy, order-
/// dependent output; this approach is deterministic, regular, and matches the
/// row-based structure of the bus layout.
/// `machines` entries are `(center_x, top_y, height)` — height (not width)
/// because every use below (row grouping, below-row fallback y) is a vertical
/// offset from the machine row. Returns `(poles, uncovered_inserters)` — the
/// second element is the set of electric inserters NEITHER a substation NOR a
/// medium pole could reach (the `give_up` set), the reactive substation pass's
/// trigger; a final `repair_pole_connectivity` bridges any disconnected pole
/// clusters.
fn place_poles(
    machines: &[(i32, i32, i32)],
    inserters: &[(i32, i32)],
    occupied: &FxHashSet<(i32, i32)>,
    substation_targets: &[SubstationTarget],
    quality: crate::common::QualityTier,
) -> (Vec<PlacedEntity>, Vec<(i32, i32)>) {
    // Medium-pole supply half-extent on the tile grid, floored from the shared
    // `supply_area_distance` (RFP Phase 3a-i/3a-ii) so this placement radius and
    // the power validator's coverage radius can never drift. place_poles places
    // only medium poles here, so the value is unchanged (floor(3.5) = 3).
    let pole_range: i32 =
        crate::common::supply_area_distance("medium-electric-pole", quality).floor() as i32;

    if machines.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut placed: FxHashSet<(i32, i32)> = FxHashSet::default();

    // Working occupancy carrying the substation footprints we place first, so
    // the medium band lines and mop-up never collide with a 2×2. Equal to
    // `occupied` (zero substations) for every non-starved layout, keeping them
    // byte-identical. `placed` stays medium-only — the mop-up's Chebyshev-3
    // coverage test must never treat a substation tile as a medium pole.
    let mut occ: FxHashSet<(i32, i32)> = occupied.clone();
    // Inserters a substation already covers under the EXACT continuous check
    // (its 9-tile supply, not the medium 3), so the mop-up set-cover skips them.
    let mut sub_covered: FxHashSet<(i32, i32)> = FxHashSet::default();

    // === Substations first (RFP Phase 3a-ii) ===
    // A substation top-left (sx,sy) → center (sx+1,sy+1); an inserter (ix,iy) →
    // center (ix+0.5,iy+0.5) is powered iff |ix+0.5−(sx+1)| ≤ 9 and likewise in
    // y, i.e. ix ∈ [sx−8, sx+9] and iy ∈ [sy−8, sy+9]. Identical to the
    // validator's exact even-footprint check — placement guarantees real
    // coverage, never leaning on the validator's word (the 3a-i carried
    // constraint).
    // Quality-derived (rfp-build-quality Phase 2): supply distance d =
    // 9 + level (always integral for the substation), giving ix ∈
    // [sx−(d−1), sx+d]. At Normal d=9 → the original 8/9 constants,
    // bit-identical placement (kill criterion 2).
    let sub_d = crate::common::supply_area_distance("substation", quality) as i32;
    let (sub_lo, sub_hi) = (sub_d - 1, sub_d);
    let sub_covers = move |sx: i32, sy: i32, ix: i32, iy: i32| -> bool {
        ix >= sx - sub_lo && ix <= sx + sub_hi && iy >= sy - sub_lo && iy <= sy + sub_hi
    };
    for target in substation_targets {
        // Greedy set-cover: drop the fewest 2×2 substations into the widened
        // band that together cover every target inserter. Each candidate must
        // have a fully free 2×2 footprint inside the band's freed rows.
        let mut remaining: Vec<(i32, i32)> =
            target.inserters.iter().copied().filter(|p| !sub_covered.contains(p)).collect();
        while !remaining.is_empty() {
            let sy_lo = target.band_y0;
            let sy_hi = target.band_y1 - 1; // rows sy and sy+1 must both fit the band
            let rx_min = remaining.iter().map(|&(x, _)| x).min().unwrap();
            let rx_max = remaining.iter().map(|&(x, _)| x).max().unwrap();
            let mut best: Option<((i32, i32), usize)> = None;
            let mut sy = sy_lo;
            while sy <= sy_hi {
                // Any sx whose 2×2 could touch a remaining inserter's reach.
                for sx in (rx_min - sub_hi)..=(rx_max + sub_lo) {
                    let foot = [(sx, sy), (sx + 1, sy), (sx, sy + 1), (sx + 1, sy + 1)];
                    if foot.iter().any(|t| occ.contains(t) || placed.contains(t)) {
                        continue;
                    }
                    let n =
                        remaining.iter().filter(|&&(ix, iy)| sub_covers(sx, sy, ix, iy)).count();
                    // Strict `>` keeps the first (smallest sy, then sx) best on
                    // ties — deterministic.
                    if n > 0 && best.is_none_or(|(_, bn)| n > bn) {
                        best = Some(((sx, sy), n));
                    }
                }
                sy += 1;
            }
            let Some(((sx, sy), _)) = best else { break };
            entities.push(make_substation(sx, sy));
            for t in [(sx, sy), (sx + 1, sy), (sx, sy + 1), (sx + 1, sy + 1)] {
                occ.insert(t);
            }
            for &(ix, iy) in &target.inserters {
                if sub_covers(sx, sy, ix, iy) {
                    sub_covered.insert((ix, iy));
                }
            }
            remaining.retain(|&(ix, iy)| !sub_covers(sx, sy, ix, iy));
        }
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

    for key in keys {
        let (top_y, mh) = key;
        let cxs = &by_row[&key];

        // Phase 0f (RFP `docs/rfp-power-supply.md`): place a pole line in
        // BOTH candidate bands, not just the preferred one. The north band
        // (top_y-1) sits on the row's input-inserter row and covers the
        // machine's north face; the south band (top_y+mh) sits on the
        // output-inserter row and covers the machine's south face. A single
        // band leaves the opposite inserter row at Chebyshev distance mh+1
        // (=4 for a 3×3 machine — one tile past the ±3 supply area), which
        // was the systemic uncovered-inserter signature Phase 0c measured
        // (40-52% of electric inserters). Both inserter rows are now
        // coverage subjects (validate/power.rs), so both bands must carry a
        // line. Redundant machine-center coverage from the two lines is
        // fine; `repair_pole_connectivity` still bridges the row cluster.
        // Each band targets one inserter row and may place the pole up to
        // pole_range tiles FURTHER from the machine than that row, so a
        // saturated inserter/belt band (high-throughput rows fill every
        // column of the input-inserter row and the belt rows above it) can
        // still be covered from the first free row beyond it. Ordered from
        // the inserter row outward: the near tile also covers the machine,
        // and one of the two bands always lands close enough to keep the
        // machine center in range.
        // Seed each band from the shared `pole_candidate_ys` (the row Phase 1
        // reserves gap tiles in) and search outward from it, AWAY from the
        // machine — north band (candidate above the machine) upward past the
        // belts, south band (below) downward — so a saturated inserter/belt
        // band is still covered from the first free row beyond it.
        let band_y_lists: Vec<Vec<i32>> = crate::common::pole_candidate_ys(top_y, mh)
            .into_iter()
            .map(|cy| {
                let dir = if cy < top_y { -1 } else { 1 };
                (0..=pole_range)
                    .map(|d| cy + dir * d)
                    .filter(|&y| y >= 0)
                    .collect()
            })
            .collect();

        for band_ys in &band_y_lists {
            let mut i = 0;
            while i < cxs.len() {
                // A 3×3 machine's inserters span cx-1..cx+1, so a pole in
                // cx-2..cx+2 (rightmost-first, for forward reach) covers the
                // machine's whole inserter footprint. Try each candidate y
                // (inserter row first), then the same y's with a center-only
                // fallback tile (cx±pole_range) if the inserter-covering
                // window is full. The old `cx + pole_range` alone left the
                // machine's own left inserter at distance 4.
                let target_cx = cxs[i];
                let mut placed_at: Option<(i32, i32)> = None;
                'find: for &py in band_ys {
                    for px in (target_cx - 2..=target_cx + 2).rev() {
                        if !occ.contains(&(px, py)) && !placed.contains(&(px, py)) {
                            placed_at = Some((px, py));
                            break 'find;
                        }
                    }
                }
                if placed_at.is_none() {
                    'fallback: for &py in band_ys {
                        for px in [target_cx + pole_range, target_cx - pole_range] {
                            if !occ.contains(&(px, py)) && !placed.contains(&(px, py)) {
                                placed_at = Some((px, py));
                                break 'fallback;
                            }
                        }
                    }
                }

                match placed_at {
                    Some((px, py)) => {
                        entities.push(make_pole(px, py));
                        placed.insert((px, py));
                        // Advance past every machine whose center this pole
                        // still covers (Chebyshev, x only).
                        i += 1;
                        while i < cxs.len() && (cxs[i] - px).abs() <= pole_range {
                            i += 1;
                        }
                    }
                    None => {
                        // No free tile covers cxs[i] in this band — skip to
                        // avoid an infinite loop; the power validator flags
                        // the resulting inserter/machine coverage gap.
                        i += 1;
                    }
                }
            }
        }
    }

    // Coverage-driven mop-up (RFP `docs/rfp-power-supply.md` Phase 0f): the
    // band lines above cover the standard input/output inserter rows, but
    // tall / HorizontalStack rows place inserters at offsets the bands don't
    // reach. For any electric inserter still beyond pole_range of every
    // placed pole, place a pole at the free tile in its Chebyshev
    // neighbourhood that covers the most still-uncovered inserters (a greedy
    // set-cover step). Where no free tile exists in range, the inserter is
    // left uncovered and `check_power_coverage` flags it — the
    // "coverage can't fit the row pitch" kill-criterion signal.
    let covered = |x: i32, y: i32, placed: &FxHashSet<(i32, i32)>| {
        placed
            .iter()
            .any(|(px, py)| (x - px).abs() <= pole_range && (y - py).abs() <= pole_range)
    };
    let mut give_up: FxHashSet<(i32, i32)> = FxHashSet::default();
    loop {
        // Inserters a substation already reaches are skipped — they are covered
        // by its 9-tile supply, not the medium `covered` closure's Chebyshev-3.
        let uncovered: Vec<(i32, i32)> = inserters
            .iter()
            .copied()
            .filter(|&(ix, iy)| {
                !covered(ix, iy, &placed)
                    && !sub_covered.contains(&(ix, iy))
                    && !give_up.contains(&(ix, iy))
            })
            .collect();
        let Some(&(ix, iy)) = uncovered.first() else {
            break;
        };
        // Best free tile within pole_range of the target inserter, ranked by
        // how many still-uncovered inserters it would also cover.
        let mut best: Option<((i32, i32), usize)> = None;
        for dy in -pole_range..=pole_range {
            for dx in -pole_range..=pole_range {
                let (px, py) = (ix + dx, iy + dy);
                if occ.contains(&(px, py)) || placed.contains(&(px, py)) {
                    continue;
                }
                let n = uncovered
                    .iter()
                    .filter(|&&(ux, uy)| (ux - px).abs() <= pole_range && (uy - py).abs() <= pole_range)
                    .count();
                if best.is_none_or(|(_, bn)| n > bn) {
                    best = Some(((px, py), n));
                }
            }
        }
        match best {
            Some(((px, py), _)) => {
                entities.push(make_pole(px, py));
                placed.insert((px, py));
            }
            None => {
                give_up.insert((ix, iy));
            }
        }
    }

    // Interconnect (RFP Phase 3a-ii): substations are already in `entities`, so
    // the connectivity repair treats them as wire nodes and bridges any that
    // land >9 tiles (min(18,9) substation↔medium reach) from the medium network
    // with connector poles — counted in the stated pole cost. `occ` carries the
    // substation footprints so bridges never overlap a 2×2.
    repair_pole_connectivity(&mut entities, &placed, &occ, quality);

    // Phase 2 (RFP `docs/rfp-power-supply.md`): live slack instrumentation.
    // Emit each pole's free-alternative count so the stress scoreboard tallies
    // zero-slack / median / total-pole lines — a guardrail that makes any future
    // densification change pay its power cost visibly in the golden diff.
    // Zero-slack poles are "works but fragile"; the scoreboard also carries the
    // Phase 3 triggers (solid-row zero-slack = trigger (a); per-case pole total
    // vs the census baseline = trigger (b)).
    //
    // Measured on the FINAL pole set (after band placement + mop-up + repair) in
    // the census's `local_alternatives` window — same y, x within ±pole_range,
    // excluding the pole's own tile and every other pole. This deviates from the
    // RFP brief's "at each pole's decision instant" framing on purpose: the
    // ground-truth census computes slack post-hoc over the *complete* pole set,
    // so a per-decision measure (which can't see poles the mop-up/repair place
    // later) would diverge from it and fail the ±1 fixed-point criterion. This
    // pass is still fully live — in-engine, on the real pole positions, no probe
    // replay — so it matches the census by construction. Emitted per pole ENTITY
    // (not per unique tile) so `total poles` is the true count and a future
    // overlapping-pole regression can't hide inside a deduped set.
    // One PoleSlack per power ENTITY, substations included (they are in
    // `entities`), so the scoreboard's `total poles` line is the true
    // power-entity count — the census-baseline interconnect guardrail reads it.
    let all_poles: FxHashSet<(i32, i32)> = entities.iter().map(|e| (e.x, e.y)).collect();
    for e in &entities {
        let (px, py) = (e.x, e.y);
        let alternatives = (px - pole_range..=px + pole_range)
            .filter(|&x| x != px && !occ.contains(&(x, py)) && !all_poles.contains(&(x, py)))
            .count() as i32;
        crate::trace::emit(crate::trace::TraceEvent::PoleSlack { x: px, y: py, alternatives });
    }

    let mut uncovered: Vec<(i32, i32)> = give_up.into_iter().collect();
    uncovered.sort_unstable();
    (entities, uncovered)
}

/// After the row lines are placed, bridge any remaining disconnected pole
/// clusters with medium-electric-poles. This fires when two pole clusters land
/// further apart than their wire reach (e.g. an oil-refinery row above a
/// chemical-plant row with a pipe-routing gap between them, or a substation
/// that lands outside the medium network). We walk bridge poles between the two
/// nearest clusters until the whole pole field is one connected component.
///
/// The connectivity metric MUST match the artifact the blueprint emits and the
/// validator checks. [`crate::power_wires`] wires two poles iff the Euclidean
/// distance between their footprint CENTERS is ≤ the *smaller* of the two poles'
/// per-type wire reaches (min-of-both). Integer top-left deltas with one
/// hardcoded reach diverged for substation↔medium pairs — a 2×2 substation's
/// center is +1.0 and its reach 18, a medium's are +0.5 and 9 — so repair could
/// call a pair connected that the emitted `wires` array left as two power-dead
/// islands. Consuming `power_wires::{pole_center, wire_reach}` here makes repair
/// and the emitted wire graph agree by construction.
///
/// Quality (rfp-build-quality): reaches are evaluated at the build-quality
/// tier passed in — the same tier the functional stamp pass later writes onto
/// every pole, so post-stamp `compute_pole_wires` (which reads per-entity
/// `quality`) sees exactly the graph repair reasoned about. At legendary the
/// medium reach is 19, so repair neither mis-clusters nor over-bridges.
fn repair_pole_connectivity(
    entities: &mut Vec<PlacedEntity>,
    placed: &FxHashSet<(i32, i32)>,
    occupied: &FxHashSet<(i32, i32)>,
    quality: crate::common::QualityTier,
) {
    use crate::power_wires::{pole_center, wire_reach};

    // The bridge poles we drop are medium-electric-poles (stamped at the build
    // quality), so every reach test against a candidate bridge is
    // min(medium@quality, endpoint@quality).
    let medium_reach =
        wire_reach("medium-electric-pole", quality).expect("medium pole has a wire reach");
    // Search radius (in tiles) for a free bridge tile around the midpoint. Must
    // reach at least a medium's wire reach so that when the gap between
    // components is wider than `2 * reach`, the scan can step *back* toward an
    // endpoint and find a tile that's both free and within wire reach of `pa`
    // or `pb`. With radius 6, gaps wider than 12 left the loop unable to drop a
    // first bridge pole and the components stayed disconnected — see
    // `tier4_advanced_circuit_from_ore_am2`, where the pa↔pb gap is ~32 tiles.
    let scan_radius = medium_reach.ceil() as i32;

    let mut all_occupied: FxHashSet<(i32, i32)> = occupied.iter().copied().collect();
    for &p in placed {
        all_occupied.insert(p);
    }

    for _ in 0..20 {
        let n = entities.len();
        if n <= 1 {
            return;
        }
        // Per-pole geometry: top-left (the bridge-midpoint seed) and the
        // footprint center + per-type wire reach (the connectivity metric,
        // identical to `power_wires::compute_pole_wires`).
        let tops: Vec<(i32, i32)> = entities.iter().map(|e| (e.x, e.y)).collect();
        let nodes: Vec<(f64, f64, f64)> = entities
            .iter()
            .map(|e| {
                let (cx, cy) = pole_center(&e.name, e.x, e.y);
                (cx, cy, wire_reach(&e.name, quality).unwrap_or(medium_reach))
            })
            .collect();

        // Union-find over Euclidean center distance ≤ min-of-both reach.
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
                let (ax, ay, ar) = nodes[i];
                let (bx, by, br) = nodes[j];
                let (dx, dy) = (ax - bx, ay - by);
                let reach = ar.min(br);
                if dx * dx + dy * dy <= reach * reach {
                    let ri = find(&mut parent, i);
                    let rj = find(&mut parent, j);
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
            }
        }

        // Group pole indices by root component.
        let mut by_comp: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        for idx in 0..n {
            let root = find(&mut parent, idx);
            by_comp.entry(root).or_default().push(idx);
        }
        if by_comp.len() == 1 {
            return;
        }

        // Closest inter-component pole pair, by center distance (squared —
        // order is identical to Euclidean and we avoid sqrt).
        let comps: Vec<&Vec<usize>> = by_comp.values().collect();
        let mut best: Option<(usize, usize, f64)> = None;
        for a in 0..comps.len() {
            for b in (a + 1)..comps.len() {
                for &ia in comps[a] {
                    for &ib in comps[b] {
                        let (ax, ay, _) = nodes[ia];
                        let (bx, by, _) = nodes[ib];
                        let (dx, dy) = (ax - bx, ay - by);
                        let d_sq = dx * dx + dy * dy;
                        if best.is_none_or(|(_, _, bd)| d_sq < bd) {
                            best = Some((ia, ib, d_sq));
                        }
                    }
                }
            }
        }
        let Some((ia, ib, _)) = best else {
            return;
        };
        let (pa, pb) = (tops[ia], tops[ib]);

        // Seed the search at the top-left midpoint, then walk outward looking
        // for a free tile within wire reach of one endpoint. A bridge is a
        // medium pole at top-left `p` (center via `pole_center`, reach medium),
        // so it wires to an endpoint iff their center distance ≤ min(medium,
        // endpoint reach) — the exact test `power_wires` will apply.
        let mid = ((pa.0 + pb.0) / 2, (pa.1 + pb.1) / 2);
        let mut bridge: Option<(i32, i32)> = None;
        'scan: for r in 0i32..=scan_radius {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue; // only examine the ring at radius r
                    }
                    let p = (mid.0 + dx, mid.1 + dy);
                    if all_occupied.contains(&p) {
                        continue;
                    }
                    let (bcx, bcy) = pole_center("medium-electric-pole", p.0, p.1);
                    let near = |node: (f64, f64, f64)| -> bool {
                        let (nx, ny, nr) = node;
                        let (dx, dy) = (bcx - nx, bcy - ny);
                        let reach = medium_reach.min(nr);
                        dx * dx + dy * dy <= reach * reach
                    };
                    if near(nodes[ia]) || near(nodes[ib]) {
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

/// Create a substation entity (2×2, 18×18 supply) at top-left `(x, y)` (RFP
/// `docs/rfp-power-reservation.md` Phase 3a-ii). Shares the `"pole"` segment tag
/// so it groups with the power network in analysis/rendering; export centers it
/// at `x+1.0` via the shared `entity_size` 2×2 entry.
fn make_substation(x: i32, y: i32) -> PlacedEntity {
    PlacedEntity {
        name: "substation".to_string(),
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

    // --- Reactive substation repair (RFP Phase 3a-ii + 3b) ---

    #[test]
    fn compute_substation_bands_flags_predecessor_gap_of_the_starved_row() {
        // Rows: 0=[1,8) 1=[15,22) 2=[30,38). An uncovered inserter inside row 1
        // widens the gap BEFORE row 1 (after row 0) — an INTERIOR band.
        let rows = three_row_layout();
        let bands = compute_substation_bands(&[(5, 17)], &rows);
        assert_eq!(bands, vec![SubstationBand { row_after: 0, extra: 2, top_edge: false }]);
        // Two inserters in the same row collapse to one band; sorted by row.
        assert_eq!(
            compute_substation_bands(&[(5, 17), (7, 18), (2, 32)], &rows),
            vec![
                SubstationBand { row_after: 0, extra: 2, top_edge: false },
                SubstationBand { row_after: 1, extra: 2, top_edge: false },
            ]
        );
    }

    #[test]
    fn compute_substation_bands_flags_top_edge_for_row_zero_starvation() {
        // RFP Phase 3b (kovarex self-loop): an uncovered inserter inside row 0
        // has no predecessor gap to widen, so 3a-ii SKIPPED it (leaving the
        // recirc inserters starved). 3b flags the row's OWN top edge instead — a
        // TOP-EDGE band anchored at row 0, threaded to `layout_pass` as a
        // y-offset bump.
        let rows = three_row_layout();
        assert_eq!(
            compute_substation_bands(&[(5, 3)], &rows),
            vec![SubstationBand { row_after: 0, extra: 2, top_edge: true }],
        );
        // Row-0 starvation AND a row-1 boundary: one top-edge band + one interior
        // band, distinct channels (the top-edge sorts after the interior on the
        // shared row-0 anchor).
        assert_eq!(
            compute_substation_bands(&[(5, 3), (5, 17)], &rows),
            vec![
                SubstationBand { row_after: 0, extra: 2, top_edge: false },
                SubstationBand { row_after: 0, extra: 2, top_edge: true },
            ],
        );
    }

    /// The substation FALLBACK branch: when an electric inserter is genuinely
    /// unreachable by any medium pole (its 7×7 is fully occupied) and a
    /// substation band is supplied, `place_poles` drops a substation into the
    /// freed band and the inserter is covered under the exact continuous check —
    /// with give-up empty. The current corpus never reaches this branch (widening
    /// alone lands the freed rows within a medium pole's ±3), but a deeper
    /// geometry would, so it is verified here by construction.
    #[test]
    fn place_poles_substation_covers_an_inserter_no_medium_pole_can_reach() {
        // One machine so place_poles doesn't early-return.
        let machines = [(5, 0, 3)];
        // A single deep inserter whose whole 7×7 is occupied → medium can't fit.
        let ins = (40, 40);
        let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        for dy in -3..=3 {
            for dx in -3..=3 {
                occupied.insert((ins.0 + dx, ins.1 + dy));
            }
        }
        // A free 2-row band 8 tiles above the inserter — outside medium reach
        // (>3), inside a substation's 9-tile supply. band top sy=31 → covers
        // iy ∈ [23,40] ⊇ 40.
        let targets = [SubstationTarget { band_y0: 31, band_y1: 32, inserters: vec![ins] }];

        let (entities, uncovered) =
            place_poles(&machines, &[ins], &occupied, &targets, crate::common::QualityTier::Normal);
        let subs: Vec<&PlacedEntity> = entities.iter().filter(|e| e.name == "substation").collect();
        assert_eq!(subs.len(), 1, "exactly one substation should cover the deep inserter");
        let s = subs[0];
        // Exact continuous coverage: ix ∈ [sx-8, sx+9], iy ∈ [sy-8, sy+9].
        assert!(
            ins.0 >= s.x - 8 && ins.0 <= s.x + 9 && ins.1 >= s.y - 8 && ins.1 <= s.y + 9,
            "substation at ({},{}) must exactly cover inserter {:?}", s.x, s.y, ins
        );
        assert!(uncovered.is_empty(), "the substation must clear the give-up set");

        // Without the band, the same inserter is unreachable — proving the
        // substation, not medium, is what covered it.
        let (ent2, unc2) =
            place_poles(&machines, &[ins], &occupied, &[], crate::common::QualityTier::Normal);
        assert!(ent2.iter().all(|e| e.name != "substation"));
        assert_eq!(unc2, vec![ins], "no band ⇒ the deep inserter stays uncovered");
    }

    /// F2 (arc review): `repair_pole_connectivity` must judge connectivity by
    /// the SAME metric the blueprint emits and the validator checks —
    /// `power_wires` (footprint centers, per-type min-of-both reach) — not
    /// integer top-left deltas with one hardcoded reach. The divergent case: a
    /// substation top-left (0,0) [center (1,1), reach 18] and a medium top-left
    /// (-9,0) [center (-8.5,0.5), reach 9]. Center distance² = 9.5²+0.5² = 90.5
    /// > min(18,9)² = 81, so the EMITTED wire graph does NOT directly connect
    /// them (old repair's top-left d²=81 ≤ 81 wrongly called them connected and
    /// left two power-dead islands). Repair must drop a bridge pole so the
    /// emitted graph is one component. (Reaches quoted at Normal quality.)
    #[test]
    fn repair_bridges_substation_medium_at_wire_boundary() {
        use crate::power_wires::{compute_pole_wires, count_disconnected_poles};

        let sub = make_substation(0, 0);
        let med = make_pole(-9, 0);
        let mut entities = vec![sub.clone(), med.clone()];

        // Pre-repair: the artifact-level wire graph leaves the boundary pair as
        // two separate islands — exactly what the old top-left metric missed.
        let wires = compute_pole_wires(&entities);
        assert!(wires.is_empty(), "boundary pair must not directly wire; got {wires:?}");
        assert_eq!(count_disconnected_poles(&entities, &wires), 1);

        // `occupied` carries the substation's 2×2 footprint so a bridge never
        // lands on it; `placed` carries the medium (mirrors the real caller).
        let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        for dx in 0..2 {
            for dy in 0..2 {
                occupied.insert((sub.x + dx, sub.y + dy));
            }
        }
        let placed: FxHashSet<(i32, i32)> = [(med.x, med.y)].into_iter().collect();

        repair_pole_connectivity(
            &mut entities,
            &placed,
            &occupied,
            crate::common::QualityTier::Normal,
        );

        // Post-repair: a bridge pole was added and the EMITTED wire graph is now
        // a single connected component — repair and artifact agree.
        assert!(entities.len() > 2, "repair must add at least one bridge pole");
        let wires2 = compute_pole_wires(&entities);
        assert_eq!(
            count_disconnected_poles(&entities, &wires2),
            0,
            "after repair the emitted wire graph must be one connected component"
        );
    }

    /// Phase 2 adversarial-review regression (rfp-build-quality decision
    /// log 2026-07-20): `repair_pole_connectivity` hardcoded the
    /// Normal-tier medium wire reach (9), so at legendary it
    /// mis-clustered poles genuinely within the real 19-tile reach and
    /// inserted needless bridge poles. Two machine rows whose pole bands
    /// sit ~12 tiles apart: beyond Normal reach (bridges required),
    /// within Legendary reach (bridges are pure waste). Also proves the
    /// legendary result is genuinely connected under the validator's own
    /// per-entity quality walk, not just "fewer poles".
    #[test]
    fn repair_pole_connectivity_uses_quality_wire_reach() {
        use crate::common::QualityTier;
        let machines = [(0, 0, 3), (0, 16, 3)];
        let occupied: FxHashSet<(i32, i32)> = FxHashSet::default();

        let (normal_poles, _) =
            place_poles(&machines, &[], &occupied, &[], QualityTier::Normal);
        let (leg_poles, _) =
            place_poles(&machines, &[], &occupied, &[], QualityTier::Legendary);

        let bridge_count = |poles: &[PlacedEntity]| {
            poles.iter().filter(|e| e.y > 4 && e.y < 15).count()
        };
        assert!(
            bridge_count(&normal_poles) > 0,
            "Normal reach (9) must bridge the ~12-tile band gap"
        );
        assert_eq!(
            bridge_count(&leg_poles),
            0,
            "Legendary reach (19) covers the gap — bridge poles are waste: {:?}",
            leg_poles.iter().map(|e| (e.x, e.y)).collect::<Vec<_>>()
        );
        assert!(leg_poles.len() < normal_poles.len());

        // The sparser legendary set must still validate as one connected
        // network under the per-entity quality walk (stamp what the
        // layout_pass stamp pass would).
        let mut lr = crate::models::LayoutResult::default();
        lr.entities = leg_poles;
        for e in &mut lr.entities {
            e.quality = Some(QualityTier::Legendary);
        }
        let issues = crate::validate::power::check_pole_network_connectivity(&lr);
        assert!(issues.is_empty(), "legendary pole net must be connected: {issues:?}");
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

    /// Package #3 mechanism: cap detection and retry-gap computation must not
    /// depend on the trace collector.
    ///
    /// The retry loop used to learn which junctions capped by scraping
    /// `JunctionGrowthCapped` events out of the thread-local collector, which
    /// only records while a trace guard is active — so untraced callers (the
    /// wasm `layout()` entry point) silently skipped every retry. Package #3
    /// returns the capped tiles as data on `layout_pass`'s tuple, so the retry
    /// decision reads a real channel. This asserts that channel carries the
    /// caps with NO trace guard on the thread, and that `compute_retry_gaps`
    /// derives gaps from it — the whole retry-triggering path, untraced.
    ///
    /// The merge-tap layout of electronic-circuit@35/s from ore caps the
    /// junction solver (`MergeTapCandidate::produce` sets `merge_tap`, then
    /// runs exactly this `layout_pass`; 11 caps observed). This is the pass-1
    /// call it makes. Pre-#3 this test could not be written at all —
    /// `layout_pass` returned no cap tiles.
    #[test]
    fn cap_detection_and_retry_gaps_are_trace_independent() {
        // No trace guard on this thread — the untraced regime.
        assert!(!crate::trace::is_active(), "test must run untraced");

        let inputs: FxHashSet<String> =
            ["iron-ore", "copper-ore"].iter().map(|s| s.to_string()).collect();
        let sr = crate::solver::solve_with_exclusions(
            "electronic-circuit",
            35.0,
            &inputs,
            "assembling-machine-2",
            &FxHashSet::default(),
        )
        .expect("solve electronic-circuit@35/s");
        let opts = LayoutOptions {
            strategy: LayoutStrategy::Pooled,
            max_belt_tier: Some("transport-belt".to_string()),
            merge_tap: true,
            ..Default::default()
        };

        let (_layout, row_spans, cap_coords, _uncovered) =
            layout_pass(&sr, &opts, None, None, 0, None).expect("layout_pass");

        assert!(
            !cap_coords.is_empty(),
            "layout_pass returned no cap tiles untraced — cap detection still \
             depends on a trace guard",
        );
        let retry_gaps = compute_retry_gaps(&cap_coords, &row_spans);
        assert!(
            !retry_gaps.is_empty(),
            "no retry gaps computed from untraced cap tiles — the retry would \
             not fire without a trace guard",
        );
    }

}
