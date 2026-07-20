//! Decomposition-search: pick the best layout among a set of
//! candidate decompositions.
//!
//! See `docs/rfp-decomposition-search.md`. The search layer sits above
//! the existing `LayoutStrategy` enum: each candidate produces a full
//! `LayoutResult` via the existing pipeline (with whatever per-strategy
//! and per-module shape-fix machinery applies), the layouts are scored,
//! and the winner's layout is returned.
//!
//! ## Phase 0 status
//!
//! Only `NativeCandidate` exists — wraps today's dispatch. With one
//! candidate the search is a no-op pass-through (K-DS0-1 inertness
//! gate); the abstraction is in place so non-Native candidates can land
//! in Phase 1+ without refactoring `build_bus_layout`.

use crate::density;
use crate::models::{LayoutResult, SolverResult};

use super::balancer::shape_is_stampable;
use super::layout::{
    run_layout_with_explicit_plan, run_layout_with_retry, LayoutOptions, LayoutStrategy,
};
use super::partitioner::{
    apply_cap_driven_split, apply_partition_plan, apply_size_split, plan_partitioning,
    ModuleAssignment, PartitionPlan,
};
use super::shape_fix::{
    select_shape_fix, PadLanesStrategy, ShapeFix, ShapeFixStrategy, ShardStrategy,
};

/// Soft-score weights. Frozen until Phase 1 introduces a second
/// candidate — with one candidate, ordering is trivial and these
/// values do not affect output. Phase 1 will calibrate against a
/// corpus where `NativeCandidate` and `ModuleSizeSplit` produce
/// distinguishable scores on the motivating PU@3/s ore-red case.
///
/// Magnitudes:
/// - `density` ∈ [0, 1]      → α weight ≈ 1.0 dominates the "good" axis
/// - `overproduction` ≥ 0     → β small; typical values are fractions of items/sec
/// - `entity_count` ≥ 0       → γ tiny; entity counts are 100s–1000s
const ALPHA_DENSITY: f64 = 1.0;
const BETA_OVERPRODUCTION: f64 = 0.001;
const GAMMA_ENTITY_COUNT: f64 = 0.0001;

/// One scored candidate. `accepted == false` means hard constraints
/// failed (demand not met, unstampable shapes left over) and the
/// candidate is dropped from the ranking regardless of `score`.
#[derive(Debug, Clone)]
pub struct CandidateScore {
    pub score: f64,
    pub density: f64,
    pub entity_count: usize,
    pub overproduction: f64,
    pub accepted: bool,
    /// Short tag explaining `accepted == false`, if applicable.
    pub accepted_reason: Option<String>,
}

/// A single candidate decomposition strategy. `produce` is a full
/// layout call — same level of abstraction as today's `build_bus_layout`
/// — so each candidate can apply whatever pre-pipeline transformations
/// it needs (partitioning, splitting, producer round-up) and then run
/// the rest of the engine unchanged.
pub trait DecompositionCandidate {
    fn name(&self) -> &str;
    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String>;
}

/// The Phase 0 / no-op candidate: delegates to today's strategy
/// dispatch. With this as the only candidate in the catalogue,
/// `select_best_decomposition` returns byte-identical layouts to the
/// pre-RFP `build_bus_layout` (K-DS0-1 inertness gate).
pub struct NativeCandidate;

impl DecompositionCandidate for NativeCandidate {
    fn name(&self) -> &str {
        "native"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        run_layout_with_retry(solver_result, opts)
    }
}

/// Pooled candidate: retire the unstampable (n, m) balancer a family would
/// otherwise get to `K = ceil(rate / belt_cap)` shared trunks — producers
/// merge in via splitter merge-trees, consumers tap with priority splitters
/// (`docs/rfp-merge-tap-trunks.md`). This is the only place `merge_tap` is
/// turned on: it flips the runtime flag and re-runs the ordinary pipeline.
///
/// Pooled-only. Under any other strategy the merge-tap fallback either
/// re-merges siblings away (`Pooled` is where the shared trunk makes sense)
/// or fights the partitioner's module IDs, so `produce` returns an error and
/// the selector falls through to `NativeCandidate`.
pub struct MergeTapCandidate;

impl DecompositionCandidate for MergeTapCandidate {
    fn name(&self) -> &str {
        "merge-tap"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        if !matches!(opts.strategy, LayoutStrategy::Pooled) {
            return Err("merge-tap candidate is Pooled-only".to_string());
        }
        let mut mt_opts = opts.clone();
        mt_opts.merge_tap = true;
        run_layout_with_retry(solver_result, &mt_opts)
    }
}

/// Phase 1 candidate: split each multi-producer module into `k` sibling
/// sub-modules, each with halved rate and independent bus presence.
/// Targets coprime balancer shapes like `(4, 9)` on PU@3/s ore-red
/// copper-plate — splitting into `2 × (2, 5)` gives two natively
/// stampable shapes instead of one unstampable one.
///
/// Only meaningful under `LayoutStrategy::PartitionedDecomposed` —
/// `Pooled` re-merges sibling producers into one balancer regardless,
/// so the split has no effect there. When invoked on `Pooled`,
/// `produce` returns an error and the selector falls through to
/// `NativeCandidate`.
///
/// Pipeline shape:
/// 1. `plan_partitioning(strategy=PartitionedDecomposed)` — baseline plan
/// 2. `apply_size_split(plan, k)` — augment with k-way splits
/// 3. `apply_partition_plan(solver, augmented)` — bake module IDs into
///    the SolverResult's `ItemFlow.module_id` fields
/// 4. `run_layout_with_retry(transformed, opts.with(strategy=Pooled))` —
///    `Pooled` skips the strategy-dispatch re-partitioning so the pre-
///    applied plan survives intact.
pub struct ModuleSizeSplit {
    pub k: u32,
}

impl DecompositionCandidate for ModuleSizeSplit {
    fn name(&self) -> &str {
        "size-split-2"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        if !matches!(opts.strategy, LayoutStrategy::PartitionedDecomposed) {
            return Err(
                "ModuleSizeSplit only applies to PartitionedDecomposed strategy".to_string(),
            );
        }
        let max_belt_tier = opts.max_belt_tier.as_deref();

        // Mute trace events from the interior plan_partitioning call.
        // Without this, every layout call would emit duplicate
        // PartitionRejectedByUtilization / ModulePartitioned / etc.
        // events (once for ModuleSizeSplit's plan, once for Native's
        // plan via the layout_pass strategy dispatch). Tests that
        // count those events (e.g. K1-3 partition_rejected baselines)
        // expect one set per layout call. Phase 1b candidate-event
        // capture-and-replay would supersede this, but for Phase 1a
        // suppression keeps the test corpus stable.
        let plan = crate::trace::with_muted(|| {
            plan_partitioning(solver_result, opts.strategy, max_belt_tier)
        });
        if plan.is_empty() {
            return Err(
                "ModuleSizeSplit cannot apply: partition plan is empty (no multi-consumer items)"
                    .to_string(),
            );
        }

        // Runtime guard (Phase 1a): only proceed if at least one
        // module's `(n, m)` is unstampable. Splitting a stampable
        // module doubles layout work without a shape-fix benefit, and
        // the doubled work busts the stress test time budget on
        // big partitioned cases (advanced-circuit / processing-unit
        // at 5s+). The (4, 9) coprime trap that motivated this RFP
        // *is* unstampable, so the guard fires the split exactly where
        // it matters. Phase 1b will lift this guard once the per-
        // candidate event capture lands and runtime budget is
        // actively measured.
        let any_unstampable = plan.modules.iter().any(|m| {
            let n = estimate_producer_count(m, solver_result, &plan);
            !shape_is_stampable(n, m.lane_count)
        });
        if !any_unstampable {
            return Err(
                "ModuleSizeSplit not applicable: all module shapes already stampable".to_string(),
            );
        }

        // Two-stage augmentation. First the k-way size split (the
        // (4, 9) coprime fix). Then a cap-driven split for any module
        // whose post-size-split rate still exceeds full belt capacity
        // — without this, e.g. PU@3/s ore-red lands a 40/s EC module
        // on a 30/s red trunk, and the lane planner's consumer-clamp
        // path returns Err. Cap-driven split inside ModuleSizeSplit
        // (rather than as an unconditional partitioner phase) keeps
        // existing PartitionedDecomposed cases byte-equal — only
        // candidates that opt into more partitioning pay the
        // multiply-modules cost.
        let augmented = crate::trace::with_muted(|| {
            let size_split = apply_size_split(plan, self.k, max_belt_tier);
            super::partitioner::PartitionPlan {
                modules: apply_cap_driven_split(size_split.modules, max_belt_tier),
                utilization_violations: size_split.utilization_violations,
            }
        });
        let transformed = apply_partition_plan(solver_result, &augmented);

        // Use Pooled in the inner call so the strategy-dispatch in
        // `layout_pass` doesn't re-partition (which would overwrite our
        // module IDs with a fresh plan). The transformed solver already
        // has module_ids baked into ItemFlow fields — Pooled passes
        // through unchanged and the lane planner picks them up via its
        // existing `(item, module_id)` keying.
        let inner_opts = LayoutOptions {
            strategy: LayoutStrategy::Pooled,
            max_belt_tier: opts.max_belt_tier.clone(),
            row_layout: opts.row_layout,
            surplus_policy: opts.surplus_policy,
            max_inserter_tier: opts.max_inserter_tier,
            quality: opts.quality,
            merge_tap: opts.merge_tap,
        };
        run_layout_with_retry(&transformed, &inner_opts)
    }
}

/// Estimate the producer-row count `n` for a partition module. Used by
/// `ModuleSizeSplit`'s pre-layout shape-stampability guard.
///
/// `n_for_recipe` = sum of `MachineSpec.count` over all machines that
/// output this module's item (typically one recipe). Each module gets a
/// rate-proportional share of those producers.
///
/// Returns `≥ 1`. Approximates how many producer rows the placer will
/// emit for this module: with `r_module / r_total = 0.5` and 4 total
/// producers, this module gets 2. Matches `apply_partition_plan`'s
/// share formula at `partitioner.rs:686`.
fn estimate_producer_count(
    module: &ModuleAssignment,
    solver_result: &SolverResult,
    plan: &super::partitioner::PartitionPlan,
) -> u32 {
    let total_producers: f64 = solver_result
        .machines
        .iter()
        .filter(|m| m.outputs.iter().any(|o| o.item == module.item))
        .map(|m| m.count)
        .sum();
    let total_module_rate: f64 = plan
        .modules
        .iter()
        .filter(|m| m.item == module.item)
        .map(|m| m.rate)
        .sum();
    let share = if total_module_rate > 0.0 {
        module.rate / total_module_rate
    } else {
        1.0
    };
    ((total_producers * share).ceil() as u32).max(1)
}

/// Parse Native's `layout.warnings` for missing-balancer-template
/// strings into structured `(item, n, m)` tuples. The warning format
/// is the one emitted by `bus::layout::layout_pass`:
/// `"No {n}→{m} balancer template for {item}; producer outputs are disconnected"`.
///
/// Used by `try_k1_shape_fix` to identify which K=1 items had their
/// producer→trunk handoff dropped at balancer-stamp time, so we can
/// enroll them in a follow-up partition plan with `apply_shape_fixes`-
/// computed `lane_count` overrides.
fn parse_unstampable_warnings(layout: &LayoutResult) -> Vec<(String, u32, u32)> {
    let mut out = Vec::new();
    for w in &layout.warnings {
        let Some(rest) = w.strip_prefix("No ") else { continue };
        let Some((shape_str, item_part)) = rest.split_once(" balancer template for ") else {
            continue;
        };
        let Some((n_str, m_str)) = shape_str.split_once('\u{2192}') else {
            continue;
        };
        let Ok(n) = n_str.parse::<u32>() else { continue };
        let Ok(m) = m_str.parse::<u32>() else { continue };
        let item = match item_part.split_once(';') {
            Some((before_semi, _)) => before_semi.trim().to_string(),
            None => item_part.trim().to_string(),
        };
        out.push((item, n, m));
    }
    out
}

/// Find the single consumer recipe for a K=1 item. Returns the recipe
/// name and total consumption rate, or `None` if `item` is consumed by
/// zero or 2+ recipes (in which case enrollment doesn't make sense —
/// K=1 path doesn't apply).
fn k1_consumer_for_item(
    item: &str,
    solver_result: &SolverResult,
) -> Option<(String, f64)> {
    let mut by_recipe: rustc_hash::FxHashMap<String, f64> =
        rustc_hash::FxHashMap::default();
    let mut found_fluid = false;
    for m in &solver_result.machines {
        for inp in &m.inputs {
            if inp.item == item {
                if inp.is_fluid {
                    found_fluid = true;
                }
                *by_recipe.entry(m.recipe.clone()).or_insert(0.0) += inp.rate * m.count;
            }
        }
    }
    if found_fluid || by_recipe.len() != 1 {
        return None;
    }
    by_recipe.into_iter().next()
}

/// Build a partition plan that overlays K=1 enrollments onto the
/// strategy-driven base plan. For each `(item, n, m)` from Native's
/// missing-balancer warnings:
///   * Skip if `item` is already in the base plan (multi-consumer K≥2
///     case — Phase 3 `apply_shape_fixes` already had a chance).
///   * Compute `select_shape_fix(n, m)` with the same strategies the
///     existing `apply_shape_fixes` uses (pad first, shard fallback).
///   * On a `PadLanes { new_m }` fix: enroll item with `module_id=0`,
///     `lane_count = new_m`. The lane planner picks this up via
///     `plan.lane_count_override` and pads the family.
///   * `Shard` fix is unsupported here (would require splitting the
///     producer rate, which interacts with the existing partition plan
///     in non-obvious ways) — fall through.
///
/// Returns `None` if no enrollments would apply (e.g. all warnings are
/// for K≥2 items or shard-only fixes), so the caller can skip the
/// follow-up layout pass.
fn build_k1_enrollment_plan(
    native_layout: &LayoutResult,
    solver_result: &SolverResult,
    opts: &LayoutOptions,
) -> Option<PartitionPlan> {
    let warnings = parse_unstampable_warnings(native_layout);
    if warnings.is_empty() {
        return None;
    }

    let max_belt_tier = opts.max_belt_tier.as_deref();
    let cap = super::partitioner::lane_capacity(max_belt_tier);
    let utilization_cap = cap * super::partitioner::UTILIZATION_CEILING;

    // Base plan first; if `item` already has a module here it is K≥2 and
    // out of scope for this pass.
    let mut plan = crate::trace::with_muted(|| {
        plan_partitioning(solver_result, opts.strategy, max_belt_tier)
    });

    let pad = PadLanesStrategy { max_pad: 4 };
    let shard = ShardStrategy { max_shards: 3 };
    let strategies: &[&dyn ShapeFixStrategy] = &[&pad, &shard];

    let mut enrolled_any = false;
    for (item, n, m) in warnings {
        if plan.modules.iter().any(|x| x.item == item) {
            continue; // K≥2; not our case
        }
        let Some((recipe, rate)) = k1_consumer_for_item(&item, solver_result) else {
            continue;
        };
        let new_m = match select_shape_fix(n, m, strategies) {
            Some(ShapeFix::PadLanes { new_m }) => new_m,
            // Native shouldn't reach here (the family wouldn't have
            // dropped if it were stampable), but bail safely.
            Some(ShapeFix::Native) => continue,
            // Shard for K=1 needs producer-rate splitting; leave for
            // a follow-up. Pad already covers the (4, 9) motivating case.
            Some(ShapeFix::Shard { .. }) | None => continue,
        };
        let per_lane_rate = rate / new_m as f64;
        crate::trace::emit(crate::trace::TraceEvent::K1ItemEnrolled {
            item: item.clone(),
            consumer_recipe: recipe.clone(),
            n_producers: n,
            lane_count: new_m,
        });
        plan.modules.push(ModuleAssignment {
            item,
            module_id: 0,
            consumer_recipe: recipe,
            rate,
            lane_count: new_m,
            utilization: per_lane_rate / utilization_cap.max(f64::EPSILON),
        });
        enrolled_any = true;
    }
    if enrolled_any { Some(plan) } else { None }
}

/// Sum of `max(0, production - demand)` across external output items.
/// Captures the cost of strategies that overshoot demand (e.g.
/// `ProducerCountRoundUp`). Native overshoots only by the solver's
/// `ceil(rate / machine_speed)` rounding.
fn compute_overproduction(solver_result: &SolverResult) -> f64 {
    use rustc_hash::FxHashMap;

    let mut produced: FxHashMap<&str, f64> = FxHashMap::default();
    for m in &solver_result.machines {
        for out in &m.outputs {
            *produced.entry(out.item.as_str()).or_insert(0.0) += m.count * out.rate;
        }
    }

    let mut total = 0.0;
    for ext in &solver_result.external_outputs {
        let prod = produced.get(ext.item.as_str()).copied().unwrap_or(0.0);
        let excess = prod - ext.rate;
        if excess > 0.0 {
            total += excess;
        }
    }
    total
}

/// Contamination is weighted this many starvation units in the merge-tap-vs-
/// native decision (see [`ErrorKinds`]). `3` sits inside a robust `[3, 17]`
/// window on the merge-tap corpus (see `contamination_weight_window` test):
/// electronic-circuit@35/s stays native for any integer weight `> 2`, and
/// utility-science-pack@10/s flips to merge-tap for any weight `< 18`.
const KIND_CONTAMINATION_WEIGHT: usize = 3;

/// `Severity::Error` count from a full validation pass, split by in-game
/// severity CLASS rather than counted flat. The classes are ranked by how the
/// defect behaves in a running factory, not by how many there are:
///
/// - **structural** (`entity-overlap`, `pipe-to-ground`) — the exported
///   blueprint is invalid and will not import at all. Categorically worse than
///   any number of functional defects, so it dominates the comparison
///   lexicographically (a candidate with one structural error loses to one
///   with fifty functional ones — the latter at least imports and can be
///   patched).
/// - **contamination** (`belt-item-isolation`, `fluid-network`,
///   `pipe-isolation`, `fluid-connectivity`, `underground-belt-sideload`,
///   `belt-junction`) — a wrong item/fluid reaches a shared belt/pipe. It jams
///   and PROPAGATES downstream, poisoning the sink; a single one can stall a
///   whole branch. Weighted [`KIND_CONTAMINATION_WEIGHT`]× starvation.
/// - **starvation** (everything else: `belt-dead-end`, `lane-throughput`,
///   `unresolved-junction`, `input-rate-delivery`, `belt-flow-reachability`,
///   inserter/power) — a LOCAL underdelivery that stays put and is recoverable
///   (add a belt, widen a lane). Weight 1.
///
/// This taxonomy was fixed on Factorio propagation semantics BEFORE it was
/// measured against the fixtures it decides — it is not tuned to pass them.
/// The decision it drives: on electronic-circuit@35/s the merge-tap candidate
/// trades 2 starvation dead-ends for 1 contamination (copper-plate sideloaded
/// onto the iron trunk), so it has fewer errors by COUNT (3 < 4) but is worse
/// by KIND, and native is kept; on utility-science-pack@10/s merge-tap's
/// contamination is dwarfed by native's error mass and it wins under both.
///
/// Deliberately kept out of the common `score_layout` path — every accepted
/// layout (all goldens, the bulk of the stress corpus) then pays no extra
/// validation cost, and `validate()`'s terminal `ValidationCompleted` trace
/// event never perturbs those already-blessed trace streams.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ErrorKinds {
    contamination: usize,
    starvation: usize,
    structural: usize,
}

impl ErrorKinds {
    /// Weighted functional-error total at a given contamination weight.
    /// Excludes structural (handled lexicographically in `quality_key`).
    fn weighted_functional(&self, contamination_weight: usize) -> usize {
        contamination_weight * self.contamination + self.starvation
    }

    /// Lexicographic quality key, lower is better: structural dominates (an
    /// unimportable blueprint is worse than any functional defect), then the
    /// weighted functional total breaks ties within equal structural.
    fn quality_key(&self) -> (usize, usize) {
        (
            self.structural,
            self.weighted_functional(KIND_CONTAMINATION_WEIGHT),
        )
    }
}

/// Classify a candidate layout's `Severity::Error` issues into the three
/// [`ErrorKinds`] classes. Used only by the scoped Pooled merge-tap decision
/// in `select_best_decomposition`: when native leaves an unstampable shape,
/// native and the merge-tap fallback are compared by `quality_key` and the
/// strictly-lower one wins (ties favour native).
fn classify_errors(layout: &LayoutResult, solver_result: &SolverResult) -> ErrorKinds {
    let issues = match crate::validate::validate(
        layout,
        Some(solver_result),
        crate::validate::LayoutStyle::Bus,
    ) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    let mut kinds = ErrorKinds::default();
    for i in issues
        .iter()
        .filter(|i| i.severity == crate::validate::Severity::Error)
    {
        match i.category.as_str() {
            "belt-item-isolation" | "fluid-network" | "pipe-isolation"
            | "fluid-connectivity" | "underground-belt-sideload" | "belt-junction" => {
                kinds.contamination += 1
            }
            "entity-overlap" | "pipe-to-ground" => kinds.structural += 1,
            _ => kinds.starvation += 1,
        }
    }
    kinds
}

/// Score a candidate's layout. Returns the soft score plus the input
/// metrics so the trace event can report them. Hard constraint: layout
/// must have zero `missing-balancer-template` warnings (the (n, m)
/// coprime trap that motivated this RFP). Other validator categories
/// are intentionally not gated here — they fire on harmless edge cases
/// (pole connectivity, inserter chain near edges) and would break
/// inertness on clean tier 1-3 layouts.
pub fn score_layout(layout: &LayoutResult, solver_result: &SolverResult) -> CandidateScore {
    let density_score = density::score_density(layout, (1, 1));
    let density_val = density_score.density;
    let entity_count = layout.entities.len();
    let overproduction = compute_overproduction(solver_result);

    let score = ALPHA_DENSITY * density_val
        - BETA_OVERPRODUCTION * overproduction
        - GAMMA_ENTITY_COUNT * (entity_count as f64);

    let missing_balancer = crate::validate::count_missing_balancer_template_warnings(layout);
    let (accepted, accepted_reason) = if missing_balancer > 0 {
        (
            false,
            Some(format!(
                "{missing_balancer} missing-balancer-template warning(s)"
            )),
        )
    } else {
        (true, None)
    };

    CandidateScore {
        score,
        density: density_val,
        entity_count,
        overproduction,
        accepted,
        accepted_reason,
    }
}

/// One candidate's outcome plus the trace events it emitted. Captured
/// per-candidate so only the winning candidate's events are replayed
/// into the global trace stream — losing candidates' events are
/// dropped instead of overlapping the winner's in the web UI's live
/// renderer (which surfaces the streaming sink).
struct CandidateRun {
    outcome: Option<(LayoutResult, CandidateScore)>,
    /// The candidate's layout error when it produced no outcome — kept so
    /// the all-candidates-failed terminal message can say WHY instead of
    /// the unactionable "no decomposition candidate produced a layout"
    /// (observability gap found debugging rfp-build-quality Phase 2).
    error: Option<String>,
    events: Vec<crate::trace::TraceEvent>,
}

impl CandidateRun {
    /// A candidate that wasn't tried (e.g. gating predicate was false).
    /// No outcome, no events; the winner-selection code skips it.
    fn skipped(_name: &str) -> Self {
        Self { outcome: None, events: Vec::new(), error: None }
    }
}

/// Run a candidate, score it, and capture every trace event it emitted.
/// The events are removed from the global collector so they don't bleed
/// into other candidates' runs or into the final result; the caller
/// replays only the winner's events at the end.
fn run_candidate<F>(name: &str, solver_result: &SolverResult, f: F) -> CandidateRun
where
    F: FnOnce(&SolverResult) -> Result<LayoutResult, String>,
{
    let start = crate::trace::peek_events_len();
    let result = f(solver_result);
    let mut events = crate::trace::peek_events_since(start);
    crate::trace::truncate_events(start);
    let mut error = None;
    let outcome = match result {
        Ok(layout) => {
            let score = score_layout(&layout, solver_result);
            // The Score event lives with the candidate's events (so the
            // winner-replay step keeps it alongside the rest of the
            // stream). For losing candidates, the caller separately
            // filters out and re-emits Score events for telemetry.
            events.push(crate::trace::TraceEvent::DecompositionCandidateScored {
                name: name.to_string(),
                density: score.density,
                overproduction: score.overproduction,
                entity_count: score.entity_count,
                score: score.score,
                accepted: score.accepted,
                accepted_reason: score.accepted_reason.clone(),
            });
            Some((layout, score))
        }
        Err(e) => {
            error = Some(e);
            None
        }
    };
    CandidateRun { outcome, events, error }
}

/// Like `run_candidate` but wraps the produce call in `catch_unwind`.
/// Used for `ModuleSizeSplit`, whose transformed solver can land the
/// lane planner in panic territory (e.g. consumer-clamped fan-in for
/// configurations the multi-stage balancer doesn't yet handle). Captures
/// the panic so the search degrades to whichever earlier candidate had
/// a layout instead of bringing the whole call down.
fn run_candidate_catch_unwind<F>(name: &str, solver_result: &SolverResult, f: F) -> CandidateRun
where
    F: FnOnce() -> Result<LayoutResult, String> + std::panic::UnwindSafe,
{
    let start = crate::trace::peek_events_len();
    let result = std::panic::catch_unwind(f);
    let mut events = crate::trace::peek_events_since(start);
    crate::trace::truncate_events(start);
    let mut error = None;
    let outcome = match result {
        Ok(Ok(layout)) => {
            let score = score_layout(&layout, solver_result);
            events.push(crate::trace::TraceEvent::DecompositionCandidateScored {
                name: name.to_string(),
                density: score.density,
                overproduction: score.overproduction,
                entity_count: score.entity_count,
                score: score.score,
                accepted: score.accepted,
                accepted_reason: score.accepted_reason.clone(),
            });
            Some((layout, score))
        }
        Ok(Err(e)) => {
            error = Some(e);
            None
        }
        Err(_) => {
            error = Some("panicked (caught)".to_string());
            events.push(crate::trace::TraceEvent::DecompositionCandidateScored {
                name: name.to_string(),
                density: 0.0,
                overproduction: 0.0,
                entity_count: 0,
                score: f64::NEG_INFINITY,
                accepted: false,
                accepted_reason: Some("panic in produce()".to_string()),
            });
            None
        }
    };
    CandidateRun { outcome, events, error }
}

/// Run candidates and pick the winner.
///
/// The dispatch is **sequential, not parallel**: Native runs first; if
/// its layout has zero `missing-balancer-template` warnings, the search
/// stops there and Native wins. Only when Native produces an
/// unstampable shape does the search fall through to `ModuleSizeSplit`.
/// This avoids the runtime cost of laying out every candidate when
/// Native is already clean (the common case across tier 1-3 and most
/// stress tests).
///
/// Trade-off: gives up the "score every candidate, pick best by
/// density" framing the RFP committed to. The motivation: budget. The
/// stress test corpus busts the 600s timeout when partitioned cases
/// run two full layouts. A predict-shape-before-layout guard would
/// preserve the parallel framing but is harder to get right than this
/// post-Native check (see decision log entry 2026-04-30 in the RFP).
pub fn select_best_decomposition(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String> {
    // Per-candidate run + captured trace events. Detach the sink for the
    // duration so the streaming web UI doesn't render every candidate's
    // entities live (which produced the visual stack-up of two layouts on
    // top of each other before this fix). Capture each candidate's events
    // into a side buffer and truncate them out of the collector; at the
    // end, only the winner's events get replayed to the sink and back
    // into the collector.
    let original_sink = crate::trace::swap_sink(None);

    let native_run = run_candidate("native", solver_result, |s| {
        NativeCandidate.produce(s, &opts)
    });

    // K=1 shape-fix follow-up. When Native's layout has missing-balancer
    // warnings on K=1 items (the (4, 9) coprime trap on PU@3/s ore-red
    // copper-plate), enroll those items in the partition plan with a
    // padded `lane_count` and re-run. Surgical to the actual unstampable
    // shape — no producer-rate split, no machine-count multiplication.
    // Skipped on `Pooled` and when Native is already accepted.
    let try_k1_shape_fix = matches!(opts.strategy, LayoutStrategy::PartitionedDecomposed)
        && native_run
            .outcome
            .as_ref()
            .is_some_and(|(_, score)| !score.accepted);

    let k1_run = if try_k1_shape_fix {
        let native_layout = &native_run.outcome.as_ref().unwrap().0;
        let maybe_plan = build_k1_enrollment_plan(native_layout, solver_result, &opts);
        run_candidate("k1-shape-fix", solver_result, |s| match maybe_plan.as_ref() {
            Some(plan) => run_layout_with_explicit_plan(s, &opts, plan),
            None => Err("no k1 enrollment".to_string()),
        })
    } else {
        CandidateRun::skipped("k1-shape-fix")
    };

    // `ModuleSizeSplit` is the heavy fallback. Same gating as before but
    // also gated on the cheaper K=1 fix not landing.
    let try_size_split = matches!(opts.strategy, LayoutStrategy::PartitionedDecomposed)
        && native_run
            .outcome
            .as_ref()
            .is_none_or(|(_, score)| !score.accepted)
        && k1_run
            .outcome
            .as_ref()
            .is_none_or(|(_, score)| !score.accepted);

    let split_run = if try_size_split {
        run_candidate_catch_unwind("size-split-2", solver_result, || {
            ModuleSizeSplit { k: 2 }.produce(solver_result, &opts)
        })
    } else {
        CandidateRun::skipped("size-split-2")
    };

    // Merge-and-tap fallback candidate (`docs/rfp-merge-tap-trunks.md`).
    // Pooled-only, and only when Native left an unstampable shape — Native's
    // `accepted == false` is exactly the missing-balancer-template gate. This
    // construction gate is what keeps every currently-blessed Pooled golden
    // inert: they all validate with zero missing-balancer warnings, so Native
    // is `accepted` and this candidate is never even built. `catch_unwind`
    // because the merge-tree is the riskiest transform in the pipeline —
    // a panic degrades the whole solve to Native rather than aborting.
    let try_merge_tap = matches!(opts.strategy, LayoutStrategy::Pooled)
        && native_run
            .outcome
            .as_ref()
            .is_some_and(|(_, score)| !score.accepted);

    let merge_tap_run = if try_merge_tap {
        run_candidate_catch_unwind("merge-tap", solver_result, || {
            MergeTapCandidate.produce(solver_result, &opts)
        })
    } else {
        CandidateRun::skipped("merge-tap")
    };

    // Scoped Native-vs-merge-tap decision, resolved here while the sink is
    // still detached. Metric: kind-weighted error quality (`ErrorKinds::
    // quality_key`), not a flat count — a merge-tap layout with FEWER total
    // errors than native can still lose if the difference is contamination
    // (which propagates) traded for starvation (which stays local). Ties
    // favour Native (`NATIVE_IDX`). This is deliberately *not* the accepted-
    // by-score path below — an accepted merge-tap layout that is worse by kind
    // than an unaccepted Native still loses. `classify_errors` runs
    // `validate()`, which emits a `ValidationCompleted` event per call;
    // peek/truncate drops both so they never reach the winner's replayed
    // stream. `None` when merge-tap didn't run — the generic selection then
    // applies unchanged (every non-Pooled and every Native-clean case).
    const NATIVE_IDX: usize = 0;
    const MERGE_TAP_IDX: usize = 3;
    let merge_tap_choice: Option<usize> = merge_tap_run.outcome.as_ref().map(|(mt_layout, _)| {
        let start = crate::trace::peek_events_len();
        let mergetap_kinds = classify_errors(mt_layout, solver_result);
        let native_kinds = native_run
            .outcome
            .as_ref()
            .map(|(l, _)| classify_errors(l, solver_result));
        crate::trace::truncate_events(start);
        match native_kinds {
            Some(n) if mergetap_kinds.quality_key() < n.quality_key() => MERGE_TAP_IDX,
            Some(_) => NATIVE_IDX,
            // The gate above requires an unstampable *Native layout*, so this
            // arm is unreachable in practice; if Native somehow produced
            // nothing, merge-tap is the only layout we have.
            None => MERGE_TAP_IDX,
        }
    });

    // Re-attach the sink before replaying the winner's events. Score
    // events for *every* candidate that actually ran are emitted (so
    // telemetry/snapshot debugger see what was tried), then the winner's
    // full event stream, then `DecompositionChosen`.
    if let Some(sink) = original_sink {
        crate::trace::swap_sink(Some(sink));
    }

    // Re-emit each candidate's `DecompositionCandidateScored` event for
    // telemetry. Filtering each candidate's captured events for the
    // single Score line is cheap (≤1 hit per candidate).
    for events in [
        &native_run.events,
        &k1_run.events,
        &split_run.events,
        &merge_tap_run.events,
    ] {
        for ev in events {
            if matches!(ev, crate::trace::TraceEvent::DecompositionCandidateScored { .. }) {
                crate::trace::emit(ev.clone());
            }
        }
    }

    // Pick winner: best accepted candidate by score; otherwise best
    // unaccepted candidate (degraded path so the user still sees a
    // layout — same behaviour as today's pipeline when shape-fix can't
    // resolve a (n, m) trap).
    // Index order MUST match NATIVE_IDX (0) / MERGE_TAP_IDX (3) above.
    let (native_err, k1_err, split_err, merge_tap_err) = (
        native_run.error.clone(),
        k1_run.error.clone(),
        split_run.error.clone(),
        merge_tap_run.error.clone(),
    );
    let candidates: [(Option<(LayoutResult, CandidateScore)>, Vec<crate::trace::TraceEvent>, &str); 4] = [
        (native_run.outcome, native_run.events, "native"),
        (k1_run.outcome, k1_run.events, "k1-shape-fix"),
        (split_run.outcome, split_run.events, "size-split-2"),
        (merge_tap_run.outcome, merge_tap_run.events, "merge-tap"),
    ];

    // Find best accepted candidate (highest score).
    let best_accepted_idx = candidates
        .iter()
        .enumerate()
        .filter_map(|(i, (outcome, _, _))| {
            outcome.as_ref().and_then(|(_, score)| {
                if score.accepted {
                    Some((i, score.score))
                } else {
                    None
                }
            })
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i);

    // The scoped Pooled merge-tap decision (error-count metric, ties → Native)
    // overrides the generic accepted-by-score pick when it ran; otherwise fall
    // back to best-accepted, then to the first candidate that produced a
    // layout (Native preferred — earliest in the array). Same degraded
    // behaviour as today's pipeline when shape-fix can't resolve a (n, m) trap.
    let winner_idx = merge_tap_choice
        .or(best_accepted_idx)
        .or_else(|| candidates.iter().position(|(o, _, _)| o.is_some()));

    let Some(idx) = winner_idx else {
        let details: Vec<String> = candidates
            .iter()
            .map(|(_, _, name)| name.to_string())
            .zip([&native_err, &k1_err, &split_err, &merge_tap_err])
            .map(|(name, err)| {
                format!("{name}: {}", err.as_deref().unwrap_or("did not run"))
            })
            .collect();
        return Err(format!(
            "no decomposition candidate produced a layout — {}",
            details.join("; ")
        ));
    };

    // Move winning entry out of the array; replay its captured trace
    // events to the live sink and back into the collector so the only
    // entities the web UI / snapshot debugger see are the winner's.
    let mut candidates = candidates.into_iter().collect::<Vec<_>>();
    let (outcome, events, name) = candidates.swap_remove(idx);
    let (layout, score) = outcome.expect("winner_idx must point to Some outcome");
    for ev in events {
        // Skip Score events — already replayed for telemetry above.
        if matches!(ev, crate::trace::TraceEvent::DecompositionCandidateScored { .. }) {
            continue;
        }
        crate::trace::emit(ev);
    }
    crate::trace::emit(crate::trace::TraceEvent::DecompositionChosen {
        name: name.to_string(),
        score: score.score,
    });
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemFlow, MachineSpec};

    fn empty_layout() -> LayoutResult {
        LayoutResult {
            entities: vec![],
            width: 0,
            height: 0,
            warnings: vec![],
            regions: vec![],
            trace: None,
            surplus_exits: vec![],
            voided_streams: vec![],
            effective_rows: vec![],
        }
    }

    fn empty_solver() -> SolverResult {
        SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        }
    }

    #[test]
    fn overproduction_zero_when_production_matches_demand() {
        let mut solver = empty_solver();
        solver.machines.push(MachineSpec {
            entity: "assembling-machine-1".to_string(),
            recipe: "iron-gear-wheel".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![],
            outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        });
        solver.external_outputs.push(ItemFlow {
            item: "iron-gear-wheel".to_string(),
            rate: 1.0,
            is_fluid: false,
            module_id: 0,
        });
        assert!((compute_overproduction(&solver)).abs() < 1e-9);
    }

    #[test]
    fn overproduction_picks_up_excess_for_external_outputs() {
        let mut solver = empty_solver();
        solver.machines.push(MachineSpec {
            entity: "assembling-machine-1".to_string(),
            recipe: "iron-gear-wheel".to_string(),
            self_loop: vec![], voider: false,
            count: 2.0, // 2 machines × 1/s = 2/s production
            inputs: vec![],
            outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        });
        solver.external_outputs.push(ItemFlow {
            item: "iron-gear-wheel".to_string(),
            rate: 1.5, // demand 1.5/s, produce 2/s → excess 0.5/s
            is_fluid: false,
            module_id: 0,
        });
        let excess = compute_overproduction(&solver);
        assert!((excess - 0.5).abs() < 1e-9, "expected 0.5/s excess, got {excess}");
    }

    #[test]
    fn overproduction_only_counts_external_outputs() {
        // Internal items (produced and consumed within the factory)
        // shouldn't count against overproduction — only items the user
        // asked for at the boundary.
        let mut solver = empty_solver();
        solver.machines.push(MachineSpec {
            entity: "electric-furnace".to_string(),
            recipe: "iron-plate".to_string(),
            self_loop: vec![], voider: false,
            count: 5.0, // big internal item — not external, doesn't count
            inputs: vec![],
            outputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        });
        // No external_outputs entries → no overproduction reported.
        assert!((compute_overproduction(&solver)).abs() < 1e-9);
    }

    #[test]
    fn score_layout_basic_sanity() {
        let layout = empty_layout();
        let solver = empty_solver();
        let score = score_layout(&layout, &solver);
        // Empty layout: density 0, no entities, no overproduction.
        assert_eq!(score.entity_count, 0);
        assert!((score.overproduction).abs() < 1e-9);
        assert!(score.accepted, "Phase 0 stub: always accepts");
    }

    #[test]
    fn native_candidate_name() {
        assert_eq!(NativeCandidate.name(), "native");
    }

    #[test]
    fn merge_tap_candidate_name() {
        assert_eq!(MergeTapCandidate.name(), "merge-tap");
    }

    #[test]
    fn merge_tap_candidate_is_pooled_only() {
        // The Pooled-only guard short-circuits before any layout work, so an
        // empty solver is fine here — we only exercise the strategy gate that
        // makes `select_best_decomposition` fall through to Native on any
        // non-Pooled strategy.
        let solver = empty_solver();
        let opts = LayoutOptions {
            strategy: LayoutStrategy::PartitionedDecomposed,
            ..Default::default()
        };
        let out = MergeTapCandidate.produce(&solver, &opts);
        assert!(
            out.is_err(),
            "merge-tap candidate must refuse non-Pooled strategies; got {out:?}"
        );
    }

    // ---- kind-weighted merge-tap-vs-native decision -------------------------
    // These exercise the pure comparison (`ErrorKinds::quality_key`) with the
    // real fixtures' measured kind splits as synthetic inputs, so they carry no
    // slow layout work. The merge-tap winner gate is a strict `<` with ties to
    // native, so "native selected" == `!(mergetap.quality_key() < native.…)`.

    /// electronic-circuit@35/s-from-ore with STEP B: native is 4 starvation
    /// dead-ends; merge-tap trades 2 of them for 1 contamination (copper-plate
    /// sideloaded onto the iron trunk). Count picks merge-tap (3 < 4); kind
    /// keeps native, because contamination propagates and dead-ends stay local.
    #[test]
    fn kind_keeps_ec35_native_despite_lower_count() {
        let native = ErrorKinds { contamination: 0, starvation: 4, structural: 0 };
        let mergetap = ErrorKinds { contamination: 1, starvation: 2, structural: 0 };
        // Merge-tap is cheaper by flat count (1+2 = 3 < 4) — the old metric.
        assert!(
            mergetap.contamination + mergetap.starvation
                < native.contamination + native.starvation
        );
        // …but worse by kind, so native must win.
        assert!(
            native.quality_key() < mergetap.quality_key(),
            "native must win by kind (native {:?} vs merge-tap {:?})",
            native.quality_key(),
            mergetap.quality_key(),
        );
    }

    /// utility-science-pack@10/s: native ~175 total errors, merge-tap 46
    /// (8 contamination from the balancer/trunk interleave + 38 starvation).
    /// Merge-tap wins under both count and kind — native's error mass dwarfs
    /// the contamination penalty.
    #[test]
    fn kind_flips_utility_to_merge_tap() {
        let native = ErrorKinds { contamination: 0, starvation: 175, structural: 0 };
        let mergetap = ErrorKinds { contamination: 8, starvation: 38, structural: 0 };
        assert!(
            mergetap.quality_key() < native.quality_key(),
            "merge-tap must win by kind (merge-tap {:?} vs native {:?})",
            mergetap.quality_key(),
            native.quality_key(),
        );
    }

    /// A structural error (invalid, unimportable blueprint) loses to any number
    /// of functional errors — the lexicographic structural term dominates.
    #[test]
    fn structural_dominates_functional() {
        let importable = ErrorKinds { contamination: 50, starvation: 50, structural: 0 };
        let unimportable = ErrorKinds { contamination: 0, starvation: 0, structural: 1 };
        assert!(importable.quality_key() < unimportable.quality_key());
    }

    /// The contamination weight must land inside `[3, 17]`: below 3, EC@35s
    /// stops being a strict native win (k=2 ties); at/above 18, utility flips
    /// back to native. Both bounds are checked directly against the fixtures'
    /// weighted functional totals.
    #[test]
    fn contamination_weight_window() {
        let ec_native = ErrorKinds { contamination: 0, starvation: 4, structural: 0 };
        let ec_mergetap = ErrorKinds { contamination: 1, starvation: 2, structural: 0 };
        let util_native = ErrorKinds { contamination: 0, starvation: 175, structural: 0 };
        let util_mergetap = ErrorKinds { contamination: 8, starvation: 38, structural: 0 };

        // k=2: EC ties (both 4) → native only by the strict-`<` tiebreak.
        assert_eq!(ec_native.weighted_functional(2), ec_mergetap.weighted_functional(2));
        // k=3 (lower window bound): EC native strictly wins.
        assert!(ec_native.weighted_functional(3) < ec_mergetap.weighted_functional(3));
        // k=17 (upper window bound): utility still merge-tap (174 < 175)…
        assert!(util_mergetap.weighted_functional(17) < util_native.weighted_functional(17));
        // …k=18 flips it back to native (182 > 175), so 17 is the last good weight.
        assert!(util_mergetap.weighted_functional(18) > util_native.weighted_functional(18));
        // The production weight is inside the window.
        assert!((3..=17).contains(&KIND_CONTAMINATION_WEIGHT));
    }
}
