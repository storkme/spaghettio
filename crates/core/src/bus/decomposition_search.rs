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
    let native = NativeCandidate;
    let native_name = native.name().to_string();

    let native_outcome = match native.produce(solver_result, &opts) {
        Ok(layout) => {
            let score = score_layout(&layout, solver_result);
            crate::trace::emit(crate::trace::TraceEvent::DecompositionCandidateScored {
                name: native_name.clone(),
                density: score.density,
                overproduction: score.overproduction,
                entity_count: score.entity_count,
                score: score.score,
                accepted: score.accepted,
                accepted_reason: score.accepted_reason.clone(),
            });
            Some((layout, score))
        }
        Err(_) => None,
    };

    // K=1 shape-fix follow-up. When Native's layout has missing-balancer
    // warnings on K=1 items (the (4, 9) coprime trap on PU@3/s ore-red
    // copper-plate), enroll those items in the partition plan with a
    // padded `lane_count` and re-run. This is much cheaper than
    // `ModuleSizeSplit` (no producer-rate split, no machine-count
    // multiplication) and surgical to the actual unstampable shape.
    // Skipped on `Pooled` (no per-`(item, module_id)` partitioning) and
    // when Native is already accepted.
    let try_k1_shape_fix = matches!(opts.strategy, LayoutStrategy::PartitionedDecomposed)
        && native_outcome
            .as_ref()
            .is_some_and(|(_, score)| !score.accepted);

    let k1_outcome = if try_k1_shape_fix {
        let native_layout = &native_outcome.as_ref().unwrap().0;
        match build_k1_enrollment_plan(native_layout, solver_result, &opts) {
            Some(plan) => {
                match run_layout_with_explicit_plan(solver_result, &opts, &plan) {
                    Ok(layout) => {
                        let score = score_layout(&layout, solver_result);
                        crate::trace::emit(crate::trace::TraceEvent::DecompositionCandidateScored {
                            name: "k1-shape-fix".to_string(),
                            density: score.density,
                            overproduction: score.overproduction,
                            entity_count: score.entity_count,
                            score: score.score,
                            accepted: score.accepted,
                            accepted_reason: score.accepted_reason.clone(),
                        });
                        Some((layout, score))
                    }
                    Err(_) => None,
                }
            }
            None => None,
        }
    } else {
        None
    };

    // Decide whether to also try `ModuleSizeSplit`. Only relevant under
    // `PartitionedDecomposed` (Pooled re-merges sibling producers, so
    // the split has no effect). And only when neither Native nor the
    // cheaper K=1 shape-fix produced an accepted layout — otherwise the
    // split costs density without a benefit and we shouldn't pay the
    // runtime.
    let try_size_split = matches!(opts.strategy, LayoutStrategy::PartitionedDecomposed)
        && native_outcome
            .as_ref()
            .is_none_or(|(_, score)| !score.accepted)
        && k1_outcome
            .as_ref()
            .is_none_or(|(_, score)| !score.accepted);

    let split_outcome = if try_size_split {
        let candidate = ModuleSizeSplit { k: 2 };
        let name = candidate.name().to_string();
        // `ModuleSizeSplit`'s transformed solver can land the lane
        // planner on `todo!()` paths (e.g. `lane_planner.rs:571`
        // consumer-clamped fan-in for items where the split creates a
        // configuration the multi-stage balancer hasn't been wired for
        // yet). Catch panics so the search degrades gracefully back to
        // Native's layout instead of bringing down the whole call —
        // same user-visible behaviour as today's pipeline (Native's
        // result with whatever errors it has). Fixing the underlying
        // `todo!()` is downstream work; tracked in the RFP decision
        // log.
        let produce_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            candidate.produce(solver_result, &opts)
        }));
        match produce_result {
            Ok(Ok(layout)) => {
                let score = score_layout(&layout, solver_result);
                crate::trace::emit(crate::trace::TraceEvent::DecompositionCandidateScored {
                    name: name.clone(),
                    density: score.density,
                    overproduction: score.overproduction,
                    entity_count: score.entity_count,
                    score: score.score,
                    accepted: score.accepted,
                    accepted_reason: score.accepted_reason.clone(),
                });
                Some((layout, score))
            }
            Ok(Err(_)) => None,
            Err(_) => {
                crate::trace::emit(crate::trace::TraceEvent::DecompositionCandidateScored {
                    name: name.clone(),
                    density: 0.0,
                    overproduction: 0.0,
                    entity_count: 0,
                    score: f64::NEG_INFINITY,
                    accepted: false,
                    accepted_reason: Some("panic in produce()".to_string()),
                });
                None
            }
        }
    } else {
        None
    };

    // Pick winner: best accepted candidate by score; otherwise best
    // unaccepted candidate (degraded path so the user still sees a
    // layout — same behaviour as today's pipeline when shape-fix can't
    // resolve a (n, m) trap).
    let candidates: [(Option<(LayoutResult, CandidateScore)>, &str); 3] = [
        (native_outcome, "native"),
        (k1_outcome, "k1-shape-fix"),
        (split_outcome, "size-split-2"),
    ];

    let accepted: Option<(LayoutResult, CandidateScore, &str)> = candidates
        .iter()
        .filter_map(|(outcome, name)| {
            outcome.as_ref().and_then(|(layout, score)| {
                if score.accepted {
                    Some((layout.clone(), score.clone(), *name))
                } else {
                    None
                }
            })
        })
        .max_by(|(_, a, _), (_, b, _)| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

    if let Some((layout, score, name)) = accepted {
        crate::trace::emit(crate::trace::TraceEvent::DecompositionChosen {
            name: name.to_string(),
            score: score.score,
        });
        return Ok(layout);
    }

    // No accepted candidate. Fall back to whichever candidate produced
    // a layout (Native preferred — first listed). If neither produced,
    // propagate the error.
    let fallback: Option<(LayoutResult, CandidateScore, &str)> = candidates
        .into_iter()
        .find_map(|(outcome, name)| outcome.map(|(l, s)| (l, s, name)));

    match fallback {
        Some((layout, score, name)) => {
            crate::trace::emit(crate::trace::TraceEvent::DecompositionChosen {
                name: name.to_string(),
                score: score.score,
            });
            Ok(layout)
        }
        None => Err("no decomposition candidate produced a layout".to_string()),
    }
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
        }
    }

    fn empty_solver() -> SolverResult {
        SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            dependency_order: vec![],
        }
    }

    #[test]
    fn overproduction_zero_when_production_matches_demand() {
        let mut solver = empty_solver();
        solver.machines.push(MachineSpec {
            entity: "assembling-machine-1".to_string(),
            recipe: "iron-gear-wheel".to_string(),
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
}
