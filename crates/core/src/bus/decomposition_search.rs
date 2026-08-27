//! Decomposition-search: pick the best layout among a set of
//! candidate decompositions.
//!
//! See `docs/rfc-decomposition-search.md`. The search layer sits above
//! the existing `LayoutStrategy` enum: each candidate produces a full
//! `LayoutResult` via the existing pipeline (with whatever per-strategy
//! and per-module shape-fix machinery applies), the layouts are measured
//! and scored, and the policy-driven selection loop returns the winner's
//! layout. Candidate production remains here; the decision stages live in
//! [`selection_policy`].

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
use super::selection_policy;
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
/// pre-RFC `build_bus_layout` (K-DS0-1 inertness gate).
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
/// (`docs/rfc-merge-tap-trunks.md`). This is the only place `merge_tap` is
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

/// RFC-053: the direct-insertion candidate. Builds the layout with DI
/// `Forced` so the policy can compare it with the DI-free native one.
///
/// Unlike ordinary ranked candidates, this one is registered with the
/// policy's component-wise issue-floor stage. The reason is specific and
/// measured:
/// `score_layout` is density-dominated and hard-gates only on
/// `missing-balancer-template`, while DI removes roughly a third of the
/// entities and is typically *denser*. It would therefore win the raw
/// score on layouts where it regresses warnings — which is exactly the
/// failure that defaulting DI to a bare `true` produced (an
/// `input-rate-delivery` warning on `tier2_electronic_circuit`, the
/// flagship DI pair).
///
/// `cell-composed` can safely ride the generic ranking because composed
/// density is empirically 1.5–3x WORSE, so it loses by construction.
/// DI has no such margin, so its safety has to be structural.
pub struct DirectInsertionCandidate;

impl DecompositionCandidate for DirectInsertionCandidate {
    fn name(&self) -> &str {
        "direct-insertion"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        if opts.direct_insertion != crate::bus::di_cell::DirectInsertion::Candidate {
            return Err("direct insertion is not in Candidate mode".to_string());
        }

        // RFC-059: the claim order is an internal SEARCH AXIS, not a policy.
        //
        // When a spec is eligible in two couplings, only one may fuse, and which
        // one wins is decided by the direction the dispatcher walks consumers.
        // The RFC set out to pick a direction and measured instead that
        // **neither dominates**: over every producible item at 1/5/20 per second
        // across three machine tiers, downstream-first ships a strictly better
        // layout on 6 targets and a strictly worse one on 2. Picking either
        // fixed direction forfeits the other's wins.
        //
        // What made the choice unnecessary is a second measurement: on **no**
        // target does any other assignment beat both static orders. Pinning each
        // contended coupling to claim first and rebuilding never found a third
        // answer. So the per-target optimum is always one of these two, and
        // trying both is not a heuristic — it is exhaustive over the reachable
        // set, which is why this RFC ships no estimator (P2) and no matching
        // solver (P3).
        //
        // Cost is one extra layout build on solves that have couplings, and it
        // buys a result that is never worse than either fixed order by
        // construction. That is a different trade from the one the RFC's Design
        // section rejected: there the cost was a build **per candidate
        // coupling**, unbounded in the coupling count; here it is a constant 2.
        let arm = |order: crate::bus::di_cell::DiClaimOrder| {
            let mut di_opts = opts.clone();
            di_opts.direct_insertion = crate::bus::di_cell::DirectInsertion::Forced;
            di_opts.di_claim_order = order;
            let l = run_layout_with_retry(solver_result, &di_opts)?;
            // Self-validate before competing, for the same reason
            // `CellComposedCandidate` does: `score_layout.accepted` never
            // runs the full validator, so an error-laden DI layout would
            // reach real callers as a silently broken `Ok`. Errors refuse;
            // warnings pass here and are weighed by the selection policy.
            let issues = crate::validate::validate(&l, Some(solver_result)).map_err(|e| {
                format!(
                    "direct insertion failed validation: {}",
                    e.to_string().lines().next().unwrap_or("")
                )
            })?;
            let n_err = issues
                .iter()
                .filter(|i| i.severity == crate::validate::Severity::Error)
                .count();
            if n_err > 0 {
                return Err(format!(
                    "direct insertion carries {n_err} validation errors (refusing a broken layout)"
                ));
            }
            let n_warn = crate::validate::selection_warning_count(&issues);
            Ok::<_, String>((l, n_warn))
        };

        // Two arms only under `Search`, which is NOT the default —
        // `Downstream` is (`DiClaimOrder`'s `#[default]`, reached here via
        // `LayoutOptions::default()`; RFC-059's sim close-out flipped it from
        // `Upstream`, and this comment said `Upstream` until 2026-08-21).
        // `Search` waits on #520: it is better on every validator channel and
        // ships a 0/s factory on one target. Every other value pins a single
        // arm, and that is also how the corpus sweep measures the search
        // against the pre-RFC status quo rather than asserting that picking
        // the better arm cannot be worse.
        if opts.di_claim_order != crate::bus::di_cell::DiClaimOrder::Search {
            return arm(opts.di_claim_order.clone()).map(|(l, _)| l);
        }
        let upstream = arm(crate::bus::di_cell::DiClaimOrder::Upstream);
        let downstream = arm(crate::bus::di_cell::DiClaimOrder::Downstream);
        match (upstream, downstream) {
            // Both built: keep the better on (validator warnings, then density).
            // Errors are already zero on both — an arm carrying any refuses
            // above — so warnings are the first channel that can separate them.
            // TIES GO TO UPSTREAM, which is the pre-RFC behaviour: a tie must
            // stay bit-identical to what shipped before, or every unaffected
            // target in the corpus becomes a diff to explain.
            (Ok((lu, wu)), Ok((ld, wd))) => {
                let downstream_wins = (wd, ld.warnings.len(), ld.entities.len())
                    < (wu, lu.warnings.len(), lu.entities.len());
                crate::trace::emit(crate::trace::TraceEvent::DiClaimOrderChosen {
                    order: if downstream_wins {
                        "downstream"
                    } else {
                        "upstream"
                    }
                    .to_string(),
                    upstream_entities: lu.entities.len(),
                    downstream_entities: ld.entities.len(),
                    upstream_warnings: wu,
                    downstream_warnings: wd,
                });
                Ok(if downstream_wins { ld } else { lu })
            }
            // One arm refused. This is the ordinary case, not an error: a claim
            // order that fuses the wrong pair produces a layout DI's own gate
            // rejects, and the other order's result is then the honest answer.
            (Ok((l, _)), Err(_)) | (Err(_), Ok((l, _))) => Ok(l),
            // Both refused — report upstream's reason, so the message a caller
            // sees is unchanged from before this candidate had two arms.
            (Err(e), Err(_)) => Err(e),
        }
    }
}

/// RFC-060: the horizontal-stack row-layout candidate. Builds the layout
/// with `RowLayout::HorizontalStack` so it can be compared against the
/// vertical-split native one.
///
/// Like `DirectInsertionCandidate` (and for the same measured reason),
/// this is registered with the policy's component-wise issue-floor stage:
/// horizontal rows are typically denser (sweep: +7–10pp density on the
/// AC/PU cases), so the density-dominated soft score would let it win
/// layouts where it regresses warnings. Its safety is structural: strict
/// improvement on every issue channel, with ties to native.
pub struct HorizontalStackCandidate;

impl DecompositionCandidate for HorizontalStackCandidate {
    fn name(&self) -> &str {
        "horizontal-stack"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        if !opts.horizontal_candidate {
            return Err("horizontal-stack candidate is disabled".to_string());
        }
        if !matches!(opts.row_layout, super::layout::RowLayout::VerticalSplit) {
            return Err("row layout already forced".to_string());
        }
        let mut hs_opts = opts.clone();
        hs_opts.row_layout = super::layout::RowLayout::HorizontalStack;
        let l = run_layout_with_retry(solver_result, &hs_opts)?;
        // Self-validate before competing, same as `DirectInsertionCandidate`:
        // `score_layout.accepted` never runs the full validator, so an
        // error-laden horizontal layout would reach real callers as a
        // silently broken `Ok`. Errors refuse; warnings pass here and are
        // weighed by the selection policy instead. (Conscious conservatism,
        // RFC-060 decision log: an E1 horizontal never displaces an E10
        // native — on sweep evidence horizontal's wins all land at E0, so
        // the forgone region is empty.)
        let issues = crate::validate::validate(&l, Some(solver_result)).map_err(|e| {
            format!(
                "horizontal-stack failed validation: {}",
                e.to_string().lines().next().unwrap_or("")
            )
        })?;
        let n_err = issues
            .iter()
            .filter(|i| i.severity == crate::validate::Severity::Error)
            .count();
        if n_err > 0 {
            return Err(format!(
                "horizontal-stack carries {n_err} validation errors (refusing a broken layout)"
            ));
        }
        Ok(l)
    }
}

/// RFC-051 Phase B: the cell-composition candidate. Runs only under
/// `LayoutOptions.cell_composition == Candidate` (default Off) on
/// chain-eligible solves (solid tree-with-fan-out; `cells::chain::
/// chain_eligible`). Deliberately UNBIASED: it competes on the same
/// score/acceptance machinery as every other candidate — if it only
/// wins where the bus engine refuses, that is the honest value
/// statement (RFC-051 kill 3).
pub struct CellComposedCandidate;

impl DecompositionCandidate for CellComposedCandidate {
    fn name(&self) -> &str {
        "cell-composed"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        if opts.cell_composition != crate::bus::cells::CellComposition::Candidate {
            return Err("cell composition is Off".to_string());
        }
        // Belt tier is a USER constraint, never a strategy knob: the
        // composed corridors are express-only (quantization caps them
        // at express capacity), so a lower tier cap refuses instead of
        // silently exceeding it. Tier-parameterized corridors (quantum
        // = allowed tier's capacity) are a followup.
        if let Some(t) = opts.max_belt_tier.as_deref() {
            if t != "express-transport-belt" {
                return Err(format!(
                    "cell composition uses express corridors, over the max belt tier {t}"
                ));
            }
        }
        let mut l = crate::bus::cells::chain::compose_chain_with_capacity(
            solver_result,
            opts.inserter_capacity,
        )?;
        // Self-validate before competing: `score_layout.accepted` never
        // runs the full validator, so an error-laden composition that
        // "wins" on a bus refusal would reach real callers as a
        // silently broken Ok (#387 review; mil5-ore's Router-overlap
        // class). Composition's contract is pre-verified cells +
        // template corridors — errors refuse, surfacing the bus
        // refusal instead. Warnings pass (the adjudicated categories).
        let issues = crate::validate::validate(&l, Some(solver_result)).map_err(|e| {
            format!(
                "cell composition failed validation: {}",
                e.to_string().lines().next().unwrap_or("")
            )
        })?;
        let n_err = issues
            .iter()
            .filter(|i| i.severity == crate::validate::Severity::Error)
            .count();
        if n_err > 0 {
            return Err(format!(
                "cell composition carries {n_err} validation errors (refusing a broken layout)"
            ));
        }
        // Tier-1 verification annotation (RFC-051 registry): sim-verified
        // geometries carry their measurement; unverified ones say so.
        // The note stays on `warnings` — selection counts that channel
        // and RFC-071 B3 reads the never-verified substring from it —
        // and is ALSO carried typed on the composition receipt (RFC-074
        // Unit 1) so a consumer can tell the receipt from a warning.
        if let Some(t) = solver_result.external_outputs.first() {
            let (note, verified, standing) =
                crate::bus::cells::registry::verification_status(&t.item, t.rate, &l);
            // The chain/grid composers always attach the receipt's shape;
            // a composed layout arriving here without one would ship an
            // empty-kind badge (#737 round 2) — loud in debug.
            debug_assert!(
                l.composition.as_ref().is_some_and(|c| !c.kind.is_empty()),
                "cell composition produced a layout without a composition receipt"
            );
            let receipt = l.composition.get_or_insert_with(Default::default);
            receipt.verification = note.clone();
            receipt.verified = verified;
            receipt.standing = standing.to_string();
            l.warnings.push(note);
        }
        Ok(l)
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
        // at 5s+). The (4, 9) coprime trap that motivated this RFC
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
            research_productivity: opts.research_productivity.clone(),
            max_belt_tier: opts.max_belt_tier.clone(),
            row_layout: opts.row_layout,
            surplus_policy: opts.surplus_policy,
            max_inserter_tier: opts.max_inserter_tier,
            quality: opts.quality,
            wire_mode: opts.wire_mode,
            merge_tap: opts.merge_tap,
            // Inherited (RFC-069): candidate variants must plan at the
            // same duty as the native pass or the comparison is skewed.
            planning_duty: opts.planning_duty,
            stacking: opts.stacking,
            inserter_capacity: opts.inserter_capacity,
            cell_composition: opts.cell_composition,
            splitter_tap_spacers: opts.splitter_tap_spacers,
            direct_insertion: opts.direct_insertion,
            // Inherited, not defaulted: a claim-order measurement that silently
            // reverted to P0 inside the partitioned path would report P0-vs-P0.
            di_claim_order: opts.di_claim_order.clone(),
            // Inert at this depth (run_layout_with_retry does not
            // re-enter the search); carried for faithfulness.
            horizontal_candidate: opts.horizontal_candidate,
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
        let Some(rest) = w.strip_prefix("No ") else {
            continue;
        };
        let Some((shape_str, item_part)) = rest.split_once(" balancer template for ") else {
            continue;
        };
        let Some((n_str, m_str)) = shape_str.split_once('\u{2192}') else {
            continue;
        };
        let Ok(n) = n_str.parse::<u32>() else {
            continue;
        };
        let Ok(m) = m_str.parse::<u32>() else {
            continue;
        };
        let item = match item_part.split_once(';') {
            Some((before_semi, _)) => before_semi.trim().to_string(),
            None => item_part.trim().to_string(),
        };
        out.push((item, n, m));
    }
    out
}

/// Consumer recipes of `item` with their total consumption rates, or
/// `None` when any consumption is fluid (pipes merge freely; belt
/// enrollment does not apply). Shared by the K=1 and multi-consumer
/// enrollment arms of `build_k1_enrollment_plan`. Note the guard is
/// item-wide: one fluid consumer stands the WHOLE item down, dropping
/// its solid consumers too — inherited from the K=1 predecessor,
/// stronger than strictly needed, recorded as latent hardening in the
/// RFC-069 log (#721 round 1; no live fixture mixes belt and pipe
/// consumption of one item).
fn consumers_by_recipe(
    item: &str,
    solver_result: &SolverResult,
) -> Option<rustc_hash::FxHashMap<String, f64>> {
    let mut by_recipe: rustc_hash::FxHashMap<String, f64> = rustc_hash::FxHashMap::default();
    for m in &solver_result.machines {
        for inp in &m.inputs {
            if inp.item == item {
                if inp.is_fluid {
                    return None;
                }
                *by_recipe.entry(m.recipe.clone()).or_insert(0.0) += inp.rate * m.count;
            }
        }
    }
    Some(by_recipe)
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
pub(crate) fn build_k1_enrollment_plan(
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

    // Base plan first; on PD it may already own a module for `item`
    // (then the shape-aware pass there is the authority and this pass
    // stands down). On Pooled the base plan is empty by construction.
    let mut plan =
        crate::trace::with_muted(|| plan_partitioning(solver_result, opts.strategy, max_belt_tier));

    let pad = PadLanesStrategy { max_pad: 4 };
    let shard = ShardStrategy { max_shards: 3 };
    let strategies: &[&dyn ShapeFixStrategy] = &[&pad, &shard];

    let mut enrolled_any = false;
    for (item, n, m) in warnings {
        if plan.modules.iter().any(|x| x.item == item) {
            continue; // the base plan owns this item's modules
        }
        let Some(by_recipe) = consumers_by_recipe(&item, solver_result) else {
            continue; // fluid consumption: pipes merge freely, out of scope
        };
        if by_recipe.len() == 1 {
            // K=1: pad from the warning's own (n, m). Kept exactly as
            // shipped by RFC-069 Phase A1 (#720) — the ec35 artifact and
            // its bank row are byte-stable against this arm.
            let (recipe, rate) = by_recipe.into_iter().next().expect("len checked");
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
        } else if by_recipe.len() >= 2 {
            // Multi-consumer (RFC-069's tier5 blocker): the item's one
            // pooled family produced an unstampable (n, m), so enroll it
            // the way `plan_partitioning` would under PD — one module
            // per consumer recipe, lane_count = ceil(rate / cap) — and
            // run the partitioner's own shape-fix pass over JUST these
            // new modules so an unstampable per-consumer shape gets the
            // same pad/shard treatment (single source; no second
            // implementation of the decision). tier5's three trapped
            // items (cable EC+AC, iron-plate EC+sulfuric-acid, EC
            // PU+AC) are this arm's motivating cases.
            let mut recipes: Vec<(String, f64)> = by_recipe.into_iter().collect();
            recipes.sort_by(|a, b| a.0.cmp(&b.0));
            let new_modules: Vec<ModuleAssignment> = recipes
                .into_iter()
                .enumerate()
                .map(|(module_id, (recipe, rate))| {
                    let lane_count = (rate / cap).ceil().max(1.0) as u32;
                    let per_lane_rate = rate / lane_count as f64;
                    ModuleAssignment {
                        item: item.clone(),
                        module_id: module_id as u32,
                        consumer_recipe: recipe,
                        rate,
                        lane_count,
                        utilization: per_lane_rate / utilization_cap.max(f64::EPSILON),
                    }
                })
                .collect();
            // Mirror plan_partitioning's Phase 2 → Phase 3 order for the
            // identical construction (#721 round 2): sub-shard oversized
            // modules first, then the shape-fix pass.
            let new_modules =
                super::partitioner::decompose_oversized_modules(new_modules, cap);
            let fixed =
                super::partitioner::apply_shape_fixes(new_modules, solver_result, cap);
            // DELIBERATELY no bail-out on shapes `select_shape_fix` has
            // no answer for — the asymmetry with the K=1 arm is
            // measured, not an oversight (#721 round 2 adjudication).
            // The K=1 arm's bail is sound for its arm: its enrolled
            // module's shape IS the warned shape, padded or nothing.
            // Here the per-consumer split changes every shape, and the
            // stamp path's capabilities (runtime generator, passthrough
            // rules) exceed `select_shape_fix`'s direct+gcd+pad+shard
            // model: a post-fix guard using that model was implemented
            // and VETOED tier5's working rescue (k1 flips Produced+
            // accepted → Refused with the guard in place). The
            // acceptance gate on the produced layout is the adjudicator
            // with ground truth; a model-based veto here is strictly
            // worse.
            // Utilization accounting, same as plan_partitioning's
            // construction: an over-committed module is enrolled (no
            // silent downgrade) but flagged.
            let n_estimate = super::partitioner::producer_count_estimate(solver_result, &item);
            for mo in &fixed {
                if mo.utilization > 1.0 {
                    crate::trace::emit(
                        crate::trace::TraceEvent::PartitionRejectedByUtilization {
                            item: mo.item.clone(),
                            module_id: mo.module_id,
                            lane_util: mo.utilization,
                            belt_tier: max_belt_tier
                                .unwrap_or("express-transport-belt")
                                .to_string(),
                        },
                    );
                    plan.utilization_violations.push(mo.clone());
                }
                crate::trace::emit(crate::trace::TraceEvent::K1ItemEnrolled {
                    item: mo.item.clone(),
                    consumer_recipe: mo.consumer_recipe.clone(),
                    // The same `n` the shape-fix decision used — not the
                    // warning's pooled count (#721 round 2).
                    n_producers: n_estimate,
                    lane_count: mo.lane_count,
                });
            }
            plan.modules.extend(fixed);
            enrolled_any = true;
        }
    }
    if enrolled_any {
        Some(plan)
    } else {
        None
    }
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

/// Contamination is weighted this many starvation units by the selection
/// policy's quality-key stage. `3` sits inside a robust `[3, 17]` window on
/// the merge-tap corpus (see the policy tests): electronic-circuit@35/s stays
/// native for any integer weight `> 2`, and utility-science-pack@10/s flips to
/// merge-tap for any weight `< 18`.
pub(crate) const KIND_CONTAMINATION_WEIGHT: usize = 3;

/// Classify a candidate layout's `Severity::Error` issues into the
/// policy's kind projections. This measurement is retained for the
/// scoreboard's `IssueProfile`; `selection_policy::decide` owns the
/// comparison.
///
/// Classification reads the POLICY's `error_kind_classes` table — not the
/// category consts directly (RFC-071 B2). Before this, the shipping path
/// classified through the consts while the policy carried a table nothing
/// on the shipping path consulted, so a "policy-table edit" did not
/// actually steer shipped selection — the exact policy-as-data promise
/// RFC-070 migrated for. The consts remain the SOURCE the table is built
/// from in `SelectionPolicy::current()`; this function makes the table
/// the single authority the shipping path reads.
fn classify_errors(
    layout: &LayoutResult,
    solver_result: &SolverResult,
    policy: &selection_policy::SelectionPolicy,
) -> selection_policy::ErrorKindCounts {
    let issues = match crate::validate::validate(layout, Some(solver_result)) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    let mut kinds = selection_policy::ErrorKindCounts::default();
    for i in issues
        .iter()
        .filter(|i| i.severity == crate::validate::Severity::Error)
    {
        match policy.kind_of(&i.category) {
            selection_policy::IssueKind::Contamination => kinds.contamination += 1,
            selection_policy::IssueKind::Structural => kinds.structural += 1,
            selection_policy::IssueKind::RouteSevered => kinds.route_severed += 1,
            selection_policy::IssueKind::Starvation => kinds.starvation += 1,
        }
    }
    kinds
}

/// The Error categories that classify as CONTAMINATION — a wrong item on
/// a trunk, which propagates downstream. Hoisted out of the `match` this
/// function used to spell inline so `bus::selection_policy` can build its
/// category→kind table from the same list rather than re-typing it beside
/// this one (#698 review round 2: the "two definitions that disagree"
/// class, where a category added here would silently fall to Starvation
/// there).
pub(crate) const CONTAMINATION_CATEGORIES: [&str; 5] = [
    "belt-item-isolation",
    "fluid-network",
    "pipe-isolation",
    "fluid-connectivity",
    "belt-junction",
];

/// The Error categories that classify as STRUCTURAL — the blueprint does
/// not import at all. Everything not in any list is STARVATION (the
/// `_` arm above).
pub(crate) const STRUCTURAL_CATEGORIES: [&str; 2] = ["entity-overlap", "pipe-to-ground"];

/// The Error categories that classify as ROUTE-SEVERED — a product tier
/// with no route to its consumer, a total stop rather than a throttle
/// (RFC-071 B2, #701). Membership is EVIDENCE, not intuition: on the
/// calibration matrix's full-strength table
/// (`docs/selection-policy-calibration-evidence.md`, 2026-08-23, 20/35
/// rows Factorio-vetted) these five appear exclusively on rows the sim
/// measures as broken (ac45, ec35, ec40 — 0/s, non-converged) and never
/// once on an AT-PLAN factory. Precision (#716 review): the claim is
/// "never on a working factory", NOT "any member implies exactly 0/s" —
/// the B2 flip itself ships an ec35 winner carrying two route-severed
/// errors at quarter rate, chosen because FEWER severed routes measured
/// strictly better than more. The class ranks lexicographically above
/// the weighted functional total in `ErrorKindCounts::quality_key`, so
/// 3 total-stops can no longer lose to 65 throttles — the exact ec30
/// shipping mechanism (#701, bisected).
///
/// Membership is observational, deliberately: semantically-similar
/// categories that have never fired on a calibration row
/// (`belt-connectivity` is the named example, #716 round 2) stay in
/// their default class until the table shows them — classifying on
/// meaning instead of measurement is the exact practice B2 replaces.
/// When one first fires, classify it THEN, with its row as the receipt.
pub(crate) const ROUTE_SEVERING_CATEGORIES: [&str; 5] = [
    "belt-dead-end",
    "belt-flow-path",
    "belt-flow-reachability",
    "orphan-belt-segment",
    "unresolved-junction",
];

/// Issue counts for the selection profile: validator errors,
/// validator warnings, and the SECOND issue channel
/// (`LayoutResult.warnings`, stamped by the layout pipeline itself and
/// never seen by `validate()`).
///
/// Both channels are counted deliberately. RFC-053 has already produced
/// one false "0 errors 0 warnings" claim by reading only the validator
/// (#462), and the layout channel is where ghost-router and
/// missing-balancer problems surface.
/// **Deliberately NOT `Ord`/`PartialOrd`.** A derived ordering is
/// LEXICOGRAPHIC — it compares the first differing field and stops — so
/// `(0 err, 0 warn, 12 layout_warn) < (0, 1, 0)` would be `true`, letting
/// a 12-layout-warning regression win because the validator warning count
/// improved. That silently turns the second and third channels into
/// tiebreakers when they are meant to be protected floors, which is the
/// opposite of this type's purpose (review finding on #474).
///
fn count_issues(
    layout: &LayoutResult,
    solver_result: &SolverResult,
) -> selection_policy::IssueCounts {
    let issues = match crate::validate::validate(layout, Some(solver_result)) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    selection_policy::IssueCounts {
        errors: issues
            .iter()
            .filter(|i| i.severity == crate::validate::Severity::Error)
            .count(),
        // Selection-scoped count (see `validate::selection_warning_count`).
        // NOTE 2026-08-07: the `input-rate-delivery` exemption described
        // below is LIFTED — that category now counts here. The paragraph
        // that follows is kept as the record of WHY it was excluded and
        // what it cost, not as a description of current behaviour.
        // The `SELECTION_EXCLUDED_WARNING_CATEGORIES` set remains
        // excluded — as of 2026-08-21 that is `belt-detour` ALONE. The
        // two #632 B6 demotions left the set by DELETION: PR #684
        // removed the inserter-throughput check pair, under the #675
        // off-path campaign's Tier 2 item 9. (Both numbers name the same
        // event — the PR that did it and the issue that tracked it — so
        // this comment and the constant's own, which cites #675, do not
        // disagree.) The "plus the two demotions" this line used to
        // claim described a set with three entries that has had one
        // since. Read the constant, not this comment. Receipts:
        // validator-trust.md hole 2.
        // Honest scope statement (review finding on #525 corrected an
        // earlier "bit-identical to pre-#519" overclaim here): the
        // category PRE-EXISTED with nonzero counts, so excluding it DOES
        // change selection on any config where candidates already
        // differed in input-rate-delivery — the observed flips
        // (stacking_fanin_wall_lift S=2, adjudicated in the fixture) are
        // exactly that. The exemption is a calibration firewall, not a
        // no-op: the #519 recalibration multiplied the category's counts
        // ~10x, and letting an unanchored model steer selection shipped
        // a physically over-stamped winner on stacking_ec_60s. Folding
        // flux into selection (the #520 teeth) is the recorded follow-up,
        // gated on sim-anchoring — decision log:
        // docs/rfc-lane-demand-flow.md.
        selection_warnings: crate::validate::selection_warning_count(&issues),
        layout_warnings: layout.warnings.len(),
    }
}

fn count_issues_with_source(
    layout: &LayoutResult,
    solver_result: &SolverResult,
) -> (selection_policy::IssueCounts, &'static str) {
    match crate::validate::validate(layout, Some(solver_result)) {
        Ok(issues) => (count_issue_channels(layout, &issues), "clean-flags"),
        Err(e) => (
            count_issue_channels(layout, &e.issues),
            "clean-flags(unclean)",
        ),
    }
}

fn count_issue_channels(
    layout: &LayoutResult,
    issues: &[crate::validate::ValidationIssue],
) -> selection_policy::IssueCounts {
    selection_policy::IssueCounts {
        errors: issues
            .iter()
            .filter(|i| i.severity == crate::validate::Severity::Error)
            .count(),
        selection_warnings: crate::validate::selection_warning_count(issues),
        layout_warnings: layout.warnings.len(),
    }
}

/// Score a candidate's layout. Returns the soft score plus the input
/// metrics so the trace event can report them. Hard constraint: layout
/// must have zero `missing-balancer-template` warnings (the (n, m)
/// coprime trap that motivated this RFC). Other validator categories
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
    /// The candidate's own name, captured at the call that ran it. A run
    /// is self-identifying so the index-keyed arrays below can be CHECKED
    /// against [`CANDIDATE_ORDER`] rather than trusted (#692 review round
    /// 2, 3/3: four hand-maintained same-order lists bound only by
    /// comments is the "two definitions that disagree" bug class, and a
    /// silent misalignment would attribute one candidate's verdicts to
    /// another with nothing failing).
    name: &'static str,
    outcome: Option<(LayoutResult, CandidateScore)>,
    /// The candidate's layout error when it produced no outcome — kept so
    /// the all-candidates-failed terminal message can say WHY instead of
    /// the unactionable "no decomposition candidate produced a layout"
    /// (observability gap found debugging rfc-build-quality Phase 2).
    error: Option<String>,
    /// `produce()` panicked and `catch_unwind` swallowed it. Read only by
    /// the RFC-070 scoreboard, which must not conflate a panic with an
    /// ordinary refusal — `error` carries a sentinel string in that case
    /// and string-matching it would be a latent bug the moment the tag is
    /// reworded.
    panicked: bool,
    events: Vec<crate::trace::TraceEvent>,
}

impl CandidateRun {
    /// A candidate that wasn't tried (e.g. gating predicate was false).
    /// No outcome, no events; the winner-selection code skips it.
    fn skipped(name: &'static str) -> Self {
        Self {
            name,
            outcome: None,
            events: Vec::new(),
            error: None,
            panicked: false,
        }
    }
}

/// The canonical candidate order. Every index-keyed structure in
/// [`select_best_decomposition`] — the `*_IDX` constants, the scoreboard
/// rows, `tier_outcomes` and `candidates` — is keyed by
/// THIS list, and each of those is checked against it at construction
/// using each run's own [`CandidateRun::name`]. Adding or reordering a
/// candidate without updating the list therefore fails loudly instead of
/// silently recording one candidate's verdicts against another's row
/// (#692 review round 2, 3/3).
///
/// The checks are `debug_assert`s: they compare compile-time constants,
/// so a violation cannot depend on the input — it is caught by the first
/// debug-built call, which every `cargo test` run and every CI job is
/// (tests build in debug), while release and WASM pay nothing and cannot
/// panic a browser solve over a code-level ordering mistake. Coverage is
/// unchanged; the blast radius is not (#692 review round 3, 1/3, which
/// noted the unconditional form inverted the degradation philosophy used
/// twenty lines away, where candidates run under `catch_unwind`).
pub(crate) const CANDIDATE_ORDER: [&str; 7] = [
    "native",
    "k1-shape-fix",
    "size-split-2",
    "merge-tap",
    "cell-composed",
    "direct-insertion",
    "horizontal-stack",
];

/// Indices into [`CANDIDATE_ORDER`]. Named because several decisions
/// below are scoped to specific candidates.
const NATIVE_IDX: usize = 0;
const MERGE_TAP_IDX: usize = 3;
const DI_IDX: usize = 5;
const H_IDX: usize = 6;

/// Run a candidate, score it, and capture every trace event it emitted.
/// The events are removed from the global collector so they don't bleed
/// into other candidates' runs or into the final result; the caller
/// replays only the winner's events at the end.
fn run_candidate<F>(name: &'static str, solver_result: &SolverResult, f: F) -> CandidateRun
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
    CandidateRun {
        name,
        outcome,
        events,
        error,
        panicked: false,
    }
}

/// Like `run_candidate` but wraps the produce call in `catch_unwind`.
/// Used for `ModuleSizeSplit`, whose transformed solver can land the
/// lane planner in panic territory (e.g. consumer-clamped fan-in for
/// configurations the multi-stage balancer doesn't yet handle). Captures
/// the panic so the search degrades to whichever earlier candidate had
/// a layout instead of bringing the whole call down.
fn run_candidate_catch_unwind<F>(
    name: &'static str,
    solver_result: &SolverResult,
    f: F,
) -> CandidateRun
where
    F: FnOnce() -> Result<LayoutResult, String> + std::panic::UnwindSafe,
{
    let start = crate::trace::peek_events_len();
    let result = std::panic::catch_unwind(f);
    let mut events = crate::trace::peek_events_since(start);
    crate::trace::truncate_events(start);
    let mut error = None;
    let mut panicked = false;
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
            panicked = true;
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
    CandidateRun {
        name,
        outcome,
        events,
        error,
        panicked,
    }
}

/// RFC-070 Phase 0b (#689 W1b): one candidate's row on the selection
/// scoreboard.
///
/// A row starts from the candidate's `CandidateRun` (outcome + soft score)
/// and is then FILLED IN by whichever measurement site touches that
/// candidate, **at the site where the selection path already computed the
/// number**. Nothing here recomputes: a scoreboard that re-ran
/// `validate()` on its own could disagree with the value the decision
/// actually used, and an oracle that disagrees with the thing it is
/// oracling is worse than no oracle.
///
/// The price of that discipline is `None`s — a candidate that no
/// comparison needed carries no issue counts at all, because on that call
/// nothing computed any. That is a genuine hole in what this can see, and
/// it is recorded as a hole rather than papered over with a fresh
/// validation pass.
struct CandidateVerdict {
    name: &'static str,
    outcome: crate::trace::SelectionCandidateOutcome,
    reason: Option<String>,
    score: Option<f64>,
    accepted: Option<bool>,
    accepted_reason: Option<String>,
    counts: Option<selection_policy::IssueCounts>,
    counts_source: Option<&'static str>,
    kinds: Option<selection_policy::ErrorKindCounts>,
    /// RFC-071 B3: the produced layout's warnings carry the RFC-051
    /// registry's never-verified note. Recorded by the shipping loop
    /// (which has the policy's substring); false until recorded.
    unverified: bool,
}

impl CandidateVerdict {
    /// The row's name comes from the RUN, not from a parallel list — see
    /// `CandidateRun::name`.
    fn from_run(run: &CandidateRun) -> Self {
        use crate::trace::SelectionCandidateOutcome as Outcome;
        // `panicked` is a field, not a string match on `error` — see its
        // doc on `CandidateRun`. Order matters: a panic also sets `error`.
        let outcome = match (&run.outcome, run.panicked, &run.error) {
            (Some(_), _, _) => Outcome::Produced,
            (None, true, _) => Outcome::Panicked,
            (None, false, Some(_)) => Outcome::Refused,
            (None, false, None) => Outcome::NotRun,
        };
        Self {
            name: run.name,
            outcome,
            reason: run.error.clone(),
            score: run.outcome.as_ref().map(|(_, s)| s.score),
            accepted: run.outcome.as_ref().map(|(_, s)| s.accepted),
            accepted_reason: run
                .outcome
                .as_ref()
                .and_then(|(_, s)| s.accepted_reason.clone()),
            counts: None,
            counts_source: None,
            kinds: None,
            unverified: false,
        }
    }
}

/// The seven candidate rows, indexed by [`CANDIDATE_ORDER`] — the same
/// keying the `*_IDX` constants and the `candidates` array use, so a
/// recorder can key off the index constants the decision uses.
struct Scoreboard([CandidateVerdict; 7]);

impl Scoreboard {
    /// Build the board from the runs, CHECKING each run's own name
    /// against its slot rather than trusting the literal order (#692
    /// review round 2, 3/3).
    fn from_runs(runs: [&CandidateRun; 7]) -> Self {
        Self(std::array::from_fn(|i| {
            debug_assert_eq!(
                runs[i].name, CANDIDATE_ORDER[i],
                "candidate slot {i} holds `{}` but CANDIDATE_ORDER says `{}` — a \
                 candidate was added or reordered without updating every \
                 index-keyed list in select_best_decomposition, which would \
                 record this candidate's verdicts against another's row",
                runs[i].name, CANDIDATE_ORDER[i]
            );
            CandidateVerdict::from_run(runs[i])
        }))
    }
    /// Record the issue-channel measurement a validation site computed.
    ///
    /// FIRST WRITE WINS, so `source` names the site that first measured
    /// this candidate. Repeated validation is deterministic; the label is
    /// provenance, while `SelectionDecided::stage` names the decision.
    fn record_counts(
        &mut self,
        idx: usize,
        counts: selection_policy::IssueCounts,
        source: &'static str,
    ) {
        let row = &mut self.0[idx];
        if row.counts.is_none() {
            row.counts = Some(counts);
            row.counts_source = Some(source);
        }
    }

    /// RFC-071 B3: record whether a produced layout's own warnings carry
    /// the registry's never-verified note. Recorded from the shipping
    /// loop (the policy's substring is in scope there), like the kind
    /// classes — a policy field only the measure path read would be the
    /// decorative-table class again. `|=` so a later caller cannot
    /// quietly CLEAR an observed flag; one site calls this today, so the
    /// discipline is aspiration, not exercised behaviour (#717 round 2)
    /// — a second measuring site must decide sticky-vs-refresh
    /// deliberately when it appears.
    fn record_unverified(&mut self, idx: usize, unverified: bool) {
        self.0[idx].unverified |= unverified;
    }

    /// RFC-071 B3: record every produced layout's geometry-verification
    /// standing (its own warnings vs the policy's never-verified
    /// substring). Extracted from the shipping loop so the mapping is
    /// unit-testable — the #717 review's point that with the loop inline,
    /// deleting it silently no-ops the whole #700 fix (every profile
    /// defaults to `unverified_geometry: false`) with all tests green.
    fn record_geometry_verification(
        &mut self,
        runs: &[&CandidateRun; 7],
        unverified_substring: &str,
    ) {
        for (idx, run) in runs.iter().enumerate() {
            if let Some((layout, _)) = run.outcome.as_ref() {
                let unverified = layout
                    .warnings
                    .iter()
                    .any(|w| w.contains(unverified_substring));
                self.record_unverified(idx, unverified);
            }
        }
    }

    /// Record the error-kind measurement. First write wins, for the same
    /// reason as `record_counts`.
    fn record_kinds(&mut self, idx: usize, kinds: selection_policy::ErrorKindCounts) {
        let row = &mut self.0[idx];
        if row.kinds.is_none() {
            row.kinds = Some(kinds);
        }
    }

    /// Emit one event per candidate — all seven, including the ones that
    /// never ran. "Which candidates were even eligible" is half the
    /// question RFC-070 asks, and a row that is absent from the stream
    /// cannot answer it (`docs/validator-reporting.md`: one positioned
    /// record per instance, never a count in a message).
    fn emit(&self) {
        for row in &self.0 {
            crate::trace::emit(crate::trace::TraceEvent::SelectionCandidateEvaluated {
                name: row.name.to_string(),
                outcome: row.outcome,
                reason: row.reason.clone(),
                score: row.score,
                accepted: row.accepted,
                accepted_reason: row.accepted_reason.clone(),
                errors: row.counts.map(|c| c.errors),
                selection_warnings: row.counts.map(|c| c.selection_warnings),
                layout_warnings: row.counts.map(|c| c.layout_warnings),
                counts_source: row.counts_source.map(str::to_string),
                contamination_errors: row.kinds.map(|k| k.contamination),
                route_severed_errors: row.kinds.map(|k| k.route_severed),
                starvation_errors: row.kinds.map(|k| k.starvation),
                structural_errors: row.kinds.map(|k| k.structural),
                unverified_geometry: row.unverified,
            });
        }
    }

    /// The v2 profile vector: one [`IssueProfile`] per candidate slot,
    /// derived from the rows this board already holds.
    ///
    /// **Nothing is re-validated, and that is the discipline, not an
    /// optimisation.** The rows record what each measurement site
    /// computed; re-running `validate()` here could disagree with the
    /// number a site supplied, and then a policy result would be
    /// uninterpretable. Measuring at the retained sites and projecting is
    /// also what preserves the
    /// intentional gaps: a candidate no site examined has no counts here,
    /// which is exactly the gap [`decide`] skips on.
    ///
    /// The production recorder intentionally does not call
    /// [`IssueProfile::measure`], so the
    /// produce-time `refuse_on_error` gate and the eager-measurement
    /// question are not under test here. They are covered by the unit
    /// tier in `bus::selection_policy`.
    fn v2_profiles(&self) -> Vec<selection_policy::IssueProfile> {
        self.0
            .iter()
            .map(|row| selection_policy::IssueProfile {
                outcome: Some(row.outcome),
                refusal_reason: row.reason.clone(),
                score: row.score,
                accepted: row.accepted,
                accepted_reason: row.accepted_reason.clone(),
                counts: row.counts,
                kinds: row.kinds,
                unverified_geometry: row.unverified,
            })
            .collect()
    }
}

/// Validate the index contract before the shipping decision is allowed to
/// select a candidate. `Decision::winner` is an index into the candidate
/// array, while the policy is keyed by registration order; a mismatch would
/// ship one candidate's layout under another candidate's policy.
fn validate_shipping_alignment(
    policy: &selection_policy::SelectionPolicy,
    profile_count: usize,
) -> Result<(), String> {
    if profile_count != policy.producers.len() {
        return Err(format!(
            "selection policy/profile misalignment: {profile_count} v2 profiles for {} policy producers",
            policy.producers.len()
        ));
    }

    let producer_order: Vec<&str> = policy.producers.iter().map(|p| p.name).collect();
    if producer_order.as_slice() != CANDIDATE_ORDER || policy.producers.len() != CANDIDATE_ORDER.len() {
        return Err(format!(
            "selection policy producer-order misalignment: expected CANDIDATE_ORDER {:?}, got {:?}",
            CANDIDATE_ORDER, producer_order
        ));
    }

    Ok(())
}

/// Run candidates and pick the winner.
///
/// Candidate production is sequential and each run's trace is captured;
/// the scoreboard retains the measurements, and [`selection_policy`]
/// supplies the single decision over those profiles.
pub fn select_best_decomposition(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String> {
    select_best_decomposition_with_policy(
        solver_result,
        opts,
        selection_policy::SelectionPolicy::current(),
    )
}

fn select_best_decomposition_with_policy(
    solver_result: &SolverResult,
    opts: LayoutOptions,
    policy: selection_policy::SelectionPolicy,
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

    // Cell-composition candidate (RFC-051 Phase B): flag-gated,
    // eligibility-gated, and catch_unwind — the composer's internal
    // asserts must degrade to the bus candidates, never abort the solve.
    let try_cells = opts.cell_composition == crate::bus::cells::CellComposition::Candidate
        // DI needs both rows in one place_rows call. Only `Forced`
        // actually puts DI in the native pass, so `Candidate` leaves
        // cell-composition free to compete as its own candidate — the
        // two are alternatives, not exclusions.
        && opts.direct_insertion != crate::bus::di_cell::DirectInsertion::Forced
        && opts
            .max_belt_tier
            .as_deref()
            .is_none_or(|t| t == "express-transport-belt")
        // Level-aware like the composer it gates (#733 round 6): a chain
        // grid-composable at the caller's level must be OFFERED at it.
        && crate::bus::cells::chain::chain_eligible_at(solver_result, opts.inserter_capacity).is_ok();
    let cells_run = if try_cells {
        run_candidate_catch_unwind("cell-composed", solver_result, || {
            CellComposedCandidate.produce(solver_result, &opts)
        })
    } else {
        CandidateRun::skipped("cell-composed")
    };

    // Direct-insertion candidate (RFC-053). Gated on the mode AND on the
    // solve actually having couplings — the second half is the cost
    // control: this candidate is a full extra layout pass, and the
    // search is otherwise deliberately short-circuiting because "the
    // stress test corpus busts the 600s timeout when partitioned cases
    // run two full layouts". catch_unwind for the same reason as cells.
    let try_di = opts.direct_insertion == crate::bus::di_cell::DirectInsertion::Candidate
        && !solver_result.di_couplings.is_empty();
    let di_run = if try_di {
        run_candidate_catch_unwind("direct-insertion", solver_result, || {
            DirectInsertionCandidate.produce(solver_result, &opts)
        })
    } else {
        CandidateRun::skipped("direct-insertion")
    };

    // Horizontal-stack candidate (RFC-060). Gated on the mode AND on the
    // solve actually having a `RowKind::DualInput` row — the only row
    // kind whose construction consults `RowLayout`, so "no dual-input
    // row" means the variant would be bit-identical and the extra full
    // layout pass is pure waste (same cost-control shape as `try_di`'s
    // `di_couplings` gate). catch_unwind because the horizontal template
    // path is the least-exercised in the pipeline.
    let try_horizontal = opts.horizontal_candidate
        && matches!(opts.row_layout, super::layout::RowLayout::VerticalSplit)
        // Forced DI is an explicit topology request (the A/B debug
        // control) — a competing variant must not displace it. Same
        // stand-down `try_cells` applies, and found the same way: the
        // `di_bridge_feeds_cable_only_at_high_research` unit test
        // asserts stamped DI under Forced, and the horizontal variant
        // won and returned a DI-free layout.
        && opts.direct_insertion != crate::bus::di_cell::DirectInsertion::Forced
        && super::placer::any_dual_input_row(&solver_result.machines);
    let horizontal_run = if try_horizontal {
        run_candidate_catch_unwind("horizontal-stack", solver_result, || {
            HorizontalStackCandidate.produce(solver_result, &opts)
        })
    } else {
        CandidateRun::skipped("horizontal-stack")
    };

    // K=1 shape-fix follow-up. When Native's layout has missing-balancer
    // warnings on K=1 items (the (4, 9) coprime trap on PU@3/s ore-red
    // copper-plate), enroll those items in the partition plan with a
    // padded `lane_count` and re-run. Surgical to the actual unstampable
    // shape — no producer-rate split, no machine-count multiplication.
    // Skipped when Native is already accepted — which keeps every
    // blessed clean-corpus fixture inert (their natives are accepted, so
    // this candidate is never built there).
    //
    // The `PartitionedDecomposed`-only strategy gate was REMOVED
    // 2026-08-24 (RFC-069 Phase A1): the coprime traps this candidate
    // was built for bite hardest on the POOLED default path — ec35/ec40
    // dead-end on copper-plate (4,9) and tier5-PU on three coprime
    // shapes at once, shipping 313/631-error merge-taps at 22.9%/18.5%
    // of plan while the rescue sat gated off the field. On Pooled the
    // enrollment plan builds from `plan_partitioning` over the same
    // solver result and produces a measured 0-error ec35 (RFC-069
    // decision log, 2026-08-24, receipts).
    // The Forced-DI stand-down mirrors the registration's clause (the
    // three-lists rule): an explicit topology request must not be
    // displaced by the rescue. Moot while k1 was PD-only; newly
    // reachable on Pooled (#720 review round 4).
    let try_k1_shape_fix = opts.direct_insertion != crate::bus::di_cell::DirectInsertion::Forced
        && native_run
            .outcome
            .as_ref()
            .is_some_and(|(_, score)| !score.accepted);

    let k1_run = if try_k1_shape_fix {
        let native_layout = &native_run.outcome.as_ref().unwrap().0;
        let maybe_plan = build_k1_enrollment_plan(native_layout, solver_result, &opts);
        run_candidate("k1-shape-fix", solver_result, |s| {
            match maybe_plan.as_ref() {
                Some(plan) => run_layout_with_explicit_plan(s, &opts, plan),
                None => Err("no k1 enrollment".to_string()),
            }
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

    // Merge-and-tap fallback candidate (`docs/rfc-merge-tap-trunks.md`).
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

    // The index-keyed list of runs. `tier_outcomes` and the RFC-070
    // scoreboard are both derived from this one array, and
    // `Scoreboard::from_runs` checks each run's own name against
    // `CANDIDATE_ORDER`, so the three lists cannot drift apart silently
    // (#692 review round 2, 3/3).
    let run_refs: [&CandidateRun; 7] = [
        &native_run,
        &k1_run,
        &split_run,
        &merge_tap_run,
        &cells_run,
        &di_run,
        &horizontal_run,
    ];

    // RFC-070 Phase 0b scoreboard. Built here — after every candidate has
    // run — so the measurement sites below can record their projections.
    let mut board = Scoreboard::from_runs(run_refs);

    // RFC-071 B3: record each produced layout's geometry-verification
    // standing (its own warnings vs the policy's never-verified
    // substring), which `verified_geometry_first` ranks on.
    board.record_geometry_verification(&run_refs, policy.unverified_geometry_substring);

    // Retain the quality-key measurement for the merge-tap profile. The
    // policy owns the comparison; this call only classifies the validation
    // errors and records the result used by that stage.
    if let Some((mt_layout, _)) = merge_tap_run.outcome.as_ref() {
        let start = crate::trace::peek_events_len();
        let mergetap_kinds = classify_errors(mt_layout, solver_result, &policy);
        let native_kinds = native_run
            .outcome
            .as_ref()
            .map(|(l, _)| classify_errors(l, solver_result, &policy));
        crate::trace::truncate_events(start);
        board.record_kinds(MERGE_TAP_IDX, mergetap_kinds);
        if let Some(n) = native_kinds {
            board.record_kinds(NATIVE_IDX, n);
        }
    }

    // Retain the pairwise issue-channel measurements. They feed the
    // policy's component-wise floor; no comparison or winner is computed
    // here. `validate()` emits a terminal event, so keep the established
    // peek/truncate discipline.
    if let (Some((di_layout, _)), Some((nat_layout, _))) =
        (di_run.outcome.as_ref(), native_run.outcome.as_ref())
    {
        let start = crate::trace::peek_events_len();
        let di_counts = count_issues(di_layout, solver_result);
        let nat_counts = count_issues(nat_layout, solver_result);
        crate::trace::truncate_events(start);
        board.record_counts(DI_IDX, di_counts, "di-vs-native");
        board.record_counts(NATIVE_IDX, nat_counts, "di-vs-native");
    }

    if let (Some((horizontal_layout, _)), Some((nat_layout, _))) =
        (horizontal_run.outcome.as_ref(), native_run.outcome.as_ref())
    {
        let start = crate::trace::peek_events_len();
        let horizontal_counts = count_issues(horizontal_layout, solver_result);
        let nat_counts = count_issues(nat_layout, solver_result);
        crate::trace::truncate_events(start);
        board.record_counts(H_IDX, horizontal_counts, "horizontal-vs-native");
        board.record_counts(NATIVE_IDX, nat_counts, "horizontal-vs-native");
    }

    // Preserve the old error-free-tier measurement boundary: below two
    // produced layouts no validation is needed for that tier. The bijection
    // is: for any candidate set with >1 produced layouts, an early
    // MergeTap/ScopedPairwise decision means clean-flags is not needed, and
    // any other preliminary result means it is needed. This is the old v1
    // laziness condition expressed through the policy's stage ordering;
    // `decide()` is pure and cheap over the already-recorded profiles, so
    // re-running it after filling the profiles is safe. See K70-3.
    let profiles = board.v2_profiles();
    validate_shipping_alignment(&policy, profiles.len())?;
    let preliminary = selection_policy::decide(&profiles, &policy);
    let early_stage_decided = matches!(
        preliminary.as_ref().map(|d| d.stage),
        Some(crate::trace::SelectionStage::MergeTap | crate::trace::SelectionStage::ScopedPairwise)
    );
    let tier_outcomes = run_refs.map(|r| r.outcome.as_ref());
    let n_layouts = tier_outcomes.iter().filter(|o| o.is_some()).count();
    // The old bijection ("early MergeTap/ScopedPairwise decision means
    // clean-flags is not needed") predates a Pooled `k1-shape-fix`: it
    // was true when an early decision implied every produced candidate
    // already carried counts from the pairwise sites. With the rescue on
    // the Pooled field (RFC-069 Phase A1), an early MergeTap decision
    // can coexist with a produced-but-unmeasured candidate that
    // `BestErrorFree` is entitled to rank. The widened condition is
    // honest about its population (#720 review round 1): merge-tap's
    // pre-decide site records KINDS, not counts, so every Pooled
    // unaccepted-native field measures here now — not only rescue-
    // bearing ones — and any candidate that measures error-free may
    // displace the held merge-tap, which is `BestErrorFree`'s job, not
    // a k1 special case. The loop skips rows that already carry counts,
    // so the K70-3 laziness survives everywhere it was valid (≤1
    // produced, or everyone measured) and each remaining validate()
    // runs once, on a field that is already mid-rescue and slow.
    // Scoped to candidates OUTSIDE the early decision's own pair (#720
    // review round 3): when only the incumbent and the quality-key rival
    // are unmeasured, no ranked stage can produce a different outcome —
    // an unaccepted incumbent cannot enter the accepted tiers, and the
    // rival's win is absorbed back to its pairwise tag whether or not it
    // carries counts — so measuring them buys nothing and the pre-A1
    // laziness stands on rescue-less broken fields (tier5/ac45-class,
    // the corpus's largest layouts). A produced third party (k1, split,
    // cells, DI, HS) is what the ranked stages are entitled to rank, and
    // is what triggers the measurement.
    let any_produced_unmeasured = tier_outcomes.iter().enumerate().any(|(idx, o)| {
        idx != NATIVE_IDX
            && idx != MERGE_TAP_IDX
            && o.is_some()
            && profiles[idx].counts.is_none()
    });
    if n_layouts > 1 && (!early_stage_decided || any_produced_unmeasured) {
        let start = crate::trace::peek_events_len();
        for (idx, outcome) in tier_outcomes.iter().enumerate() {
            if profiles[idx].counts.is_some() {
                continue; // first-write-wins anyway; skip the re-validate
            }
            if let Some((layout, _)) = outcome {
                let (counts, source) = count_issues_with_source(layout, solver_result);
                board.record_counts(idx, counts, source);
            }
        }
        crate::trace::truncate_events(start);
    }

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
        &cells_run.events,
        &di_run.events,
        &horizontal_run.events,
    ] {
        for ev in events {
            if matches!(
                ev,
                crate::trace::TraceEvent::DecompositionCandidateScored { .. }
            ) {
                crate::trace::emit(ev.clone());
            }
        }
    }

    // Refusal reasons for the all-candidates-failed message, taken from
    // the same checked list everything else is keyed by. Cloned here,
    // before `candidates` moves the runs.
    let refusal_reasons: [Option<String>; 7] = run_refs.map(|r| r.error.clone());
    let candidates: [(
        Option<(LayoutResult, CandidateScore)>,
        Vec<crate::trace::TraceEvent>,
        &str,
    ); 7] = [
        (native_run.outcome, native_run.events, native_run.name),
        (k1_run.outcome, k1_run.events, k1_run.name),
        (split_run.outcome, split_run.events, split_run.name),
        (
            merge_tap_run.outcome,
            merge_tap_run.events,
            merge_tap_run.name,
        ),
        (cells_run.outcome, cells_run.events, cells_run.name),
        (di_run.outcome, di_run.events, di_run.name),
        (
            horizontal_run.outcome,
            horizontal_run.events,
            horizontal_run.name,
        ),
    ];
    for (i, (_, _, name)) in candidates.iter().enumerate() {
        debug_assert_eq!(
            *name, CANDIDATE_ORDER[i],
            "candidates[{i}] holds `{name}` but CANDIDATE_ORDER says `{}` — the \
             winner index would name the wrong candidate",
            CANDIDATE_ORDER[i]
        );
    }

    // v2 is the sole shipped decision. It consumes the scoreboard
    // projections of the measurements already computed; it never
    // re-validates and there is no second winner to consult.
    let profiles = board.v2_profiles();
    validate_shipping_alignment(&policy, profiles.len())?;
    let v2_winner = selection_policy::decide(&profiles, &policy);
    let Some(v2_decision) = v2_winner else {
        // The final TieredRank stage always names an admitted produced
        // candidate whenever one exists. Therefore a v2 `None` alongside
        // a v1 winner is a v2-declined case, not an all-candidates refusal.
        // The scoreboard still goes out — WHY each candidate was declined
        // is exactly what a refusal investigation needs — but no
        // `SelectionDecided` follows it because there is no v2 winner.
        board.emit();
        // Both sides keyed by `CANDIDATE_ORDER`: the names from the list
        // itself and the reasons from `run_refs`, whose slots are checked
        // against it. Nothing here is positional against a separately
        // typed literal any more, so the message cannot glue one
        // candidate's reason onto another's name.
        let details: Vec<String> = CANDIDATE_ORDER
            .iter()
            .zip(refusal_reasons.iter())
            .map(|(name, err)| format!("{name}: {}", err.as_deref().unwrap_or("did not run")))
            .collect();
        // Phase 2c invariant: `decide` returns `None` iff no candidate produced a layout.
        return Err(format!(
            "selection policy declined every produced candidate — {}",
            details.join("; ")
        ));
    };

    let idx = v2_decision.winner;
    let stage = v2_decision.stage;

    // Move winning entry out of the array; replay its captured trace
    // events to the live sink and back into the collector so the only
    // entities the web UI / snapshot debugger see are the winner's.
    let mut candidates = candidates.into_iter().collect::<Vec<_>>();
    let (outcome, events, name) = candidates.swap_remove(idx);
    let (layout, score) = outcome.expect("v2 winner index must point to Some outcome");
    for ev in events {
        // Skip Score events — already replayed for telemetry above.
        if matches!(
            ev,
            crate::trace::TraceEvent::DecompositionCandidateScored { .. }
        ) {
            continue;
        }
        crate::trace::emit(ev);
    }
    crate::trace::emit(crate::trace::TraceEvent::DecompositionChosen {
        name: name.to_string(),
        score: score.score,
    });
    // RFC-070 Phase 0b: the scoreboard and its terminal event go out LAST
    // and ADJACENTLY, after the winner's replayed stream. A candidate's
    // `produce` can run its own nested selection whose events are replayed
    // above; emitting the outer block here keeps each selection's
    // candidates contiguous with their own `SelectionDecided`, so a reader
    // pairs them by flushing on the terminal event without a nested block
    // splicing into the outer one.
    board.emit();
    crate::trace::emit(crate::trace::TraceEvent::SelectionDecided {
        winner: name.to_string(),
        stage,
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
            research_productivity: Default::default(),
            width: 0,
            height: 0,
            boundary_inputs: vec![],
            boundary_outputs: vec![],
            warnings: vec![],
            regions: vec![],
            composition: None,
            trace: None,
            surplus_exits: vec![],
            voided_streams: vec![],
            effective_rows: vec![],
            power_wires: None,
            wire_mode: Default::default(),
            stacking: 1,
            inserter_capacity: 0,
        }
    }

    fn empty_solver() -> SolverResult {
        SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn overproduction_zero_when_production_matches_demand() {
        let mut solver = empty_solver();
        solver.machines.push(MachineSpec {
            entity: "assembling-machine-1".to_string(),
            recipe: "iron-gear-wheel".to_string(),
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
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
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
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
        assert!(
            (excess - 0.5).abs() < 1e-9,
            "expected 0.5/s excess, got {excess}"
        );
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
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
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
    fn shipping_refuses_a_misordered_policy_instead_of_shipping_a_layout() {
        let solver = empty_solver();
        let mut policy = selection_policy::SelectionPolicy::current();
        policy.producers.swap(0, 1);

        let err = select_best_decomposition_with_policy(&solver, LayoutOptions::default(), policy)
            .expect_err("a misordered shipping policy must refuse, never select by raw index");
        assert!(
            err.contains("selection policy producer-order misalignment"),
            "unexpected refusal: {err}"
        );
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

    /// RFC-071 B2 receipts producer (K71-2): for the shipped fixtures the
    /// RouteSevered class flips (ec35, ec40 — the only route-severed
    /// carriers on the calibration evidence table), select under the OLD
    /// policy (route-severing categories classed Starvation) and the NEW
    /// one, and export both winners for the meter/sim to judge. The flip
    /// ships only if every new winner measures ≥ its old winner.
    ///
    ///   B2_RECEIPTS_OUT=/tmp/b2-receipts \
    ///   SPAGHETTIO_ZONE_CACHE_PATH=<copy of the committed cache> \
    ///   cargo test --manifest-path crates/core/Cargo.toml --lib -- \
    ///     bus::decomposition_search::tests::b2_route_severed_flip_receipts \
    ///     --exact --ignored --nocapture
    #[test]
    #[ignore = "artifact producer — K71-2 receipts for the RouteSevered flip"]
    fn b2_route_severed_flip_receipts() {
        use rustc_hash::FxHashSet;

        let out_root = std::path::PathBuf::from(
            std::env::var("B2_RECEIPTS_OUT").expect("set B2_RECEIPTS_OUT to an output dir"),
        );
        for (tag, rate) in [("ec35", 35.0), ("ec40", 40.0)] {
            for policy_name in ["old", "new"] {
                // Not Clone (deliberately — one policy instance per
                // selection), so build each variant fresh.
                let mut policy = selection_policy::SelectionPolicy::current();
                if policy_name == "old" {
                    for c in ROUTE_SEVERING_CATEGORIES {
                        policy
                            .error_kind_classes
                            .insert(c.to_string(), selection_policy::IssueKind::Starvation);
                    }
                }
                let inputs: FxHashSet<String> = ["iron-ore", "copper-ore"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let sr = crate::solver::solve_with_exclusions(
                    "electronic-circuit",
                    rate,
                    &inputs,
                    "assembling-machine-2",
                    &FxHashSet::default(),
                )
                .expect("solve");
                let opts = LayoutOptions::from_groups(
                    crate::bus::layout::UserConstraints {
                        max_belt_tier: Some("transport-belt".into()),
                        ..Default::default()
                    },
                    crate::bus::layout::SearchAxes::default(),
                    crate::bus::layout::EngineTuning::default(),
                );
                let layout =
                    select_best_decomposition_with_policy(&sr, opts, policy).expect("select");
                let issues = crate::validate::validate(&layout, Some(&sr))
                    .unwrap_or_else(|e| e.issues);
                let errors = issues
                    .iter()
                    .filter(|i| i.severity == crate::validate::Severity::Error)
                    .count();
                let warnings = issues.len() - errors;
                let mut by_cat: std::collections::BTreeMap<&str, usize> = Default::default();
                for i in issues
                    .iter()
                    .filter(|i| i.severity == crate::validate::Severity::Error)
                {
                    *by_cat.entry(i.category.as_str()).or_default() += 1;
                }
                let label = format!("{tag}-{policy_name}");
                eprintln!("{label} warnings={warnings} errors by category: {by_cat:?}");
                let (bp, manifest) = crate::blueprint::export_with_manifest_validated(
                    &layout, &sr, &label, &issues,
                );
                let dir = out_root.join(&label);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join("bp.txt"), &bp).unwrap();
                std::fs::write(
                    dir.join("manifest-real.json"),
                    serde_json::to_string_pretty(&manifest).unwrap(),
                )
                .unwrap();
                eprintln!(
                    "{label:<10} entities={:<6} errors={errors:<5} -> {}",
                    layout.entities.len(),
                    dir.display()
                );
            }
        }
    }

    /// RFC-071 B3: the policy's never-verified substring matches the
    /// registry note's NO-MATCH tier only. A geometry sim-verified under
    /// a different declared world renders the "do NOT transfer across
    /// worlds" note — verified somewhere, shippable, visible — and must
    /// not read as unverified; the never-verified tier must. This pins
    /// the substring against BOTH real note shapes from
    /// `cells::registry::verification_note`, so a rewording of either
    /// note fails here instead of silently blinding (or over-triggering)
    /// the verified-first ordering.
    #[test]
    fn unverified_substring_matches_only_the_never_verified_tier() {
        let p = selection_policy::SelectionPolicy::current();
        let matches = |w: &str| w.contains(p.unverified_geometry_substring);
        // The LOAD-BEARING arm uses the REAL producer, not a transcription
        // (#717 review, 3/3): a rewording of the never-verified note in
        // `cells::registry::verification_note` must fail HERE, not
        // silently stop matching in production. No registry entry exists
        // for this target/rate, so the real no-match note is rendered.
        let real_note = crate::bus::cells::registry::verification_note(
            "no-such-target-for-this-test",
            999.25,
            &empty_layout(),
        );
        assert!(
            matches(&real_note),
            "the policy substring must match the REAL never-verified note; \
             producer says: {real_note:?}"
        );
        // The other two tiers need registry-hash collisions to render via
        // the real producer, so they stay pinned as labelled
        // transcriptions of `verification_note`'s other arms — update
        // them WITH that function.
        assert!(
            !matches(
                "cell-composed: geometry sim-verified at plan ONLY under declared capacity 1 / \
                 stacking 1 (PASS produced 2.00/s, 2026-07-23); this layout declares capacity 2 \
                 / stacking 1 — measurements do NOT transfer across worlds (#383)"
            ),
            "world-mismatch tier must not read as unverified"
        );
        assert!(
            !matches("cell-composed: geometry SIM-VERIFIED at plan (…)"),
            "verified tier must not read as unverified"
        );
        // #730 round 2 (3/3): the FAIL arms — both the full-world match
        // and the world-mismatch fallback — MUST carry the unverified
        // substring: a failed sim is the opposite of verification, and
        // the mismatch arm laundering a FAIL row into "sim-verified as
        // warned" ranked a proven-failing geometry above never-measured
        // ones. Transcriptions of `verification_note`'s FAIL arms —
        // update them WITH that function.
        assert!(
            matches(
                "cell-composed: geometry NOT sim-verified — the sim FAILED it at its declared                  world (scenario — FAIL produced 12.58/s at declared capacity 1, 2026-08-26)"
            ),
            "the full-world FAIL arm must read as unverified"
        );
        assert!(
            matches(
                "cell-composed: geometry NOT sim-verified — a hash-sharing build sim-FAILED in a DIFFERENT declared world (capacity 1 / stacking 1: FAIL produced 12.58/s, 2026-08-26); this layout declares capacity 2 / stacking 1, which was never measured"
            ),
            "the world-mismatch FAIL arm must read as unverified"
        );
    }

    /// RFC-071 B3, end-to-end over the shipping mapping (#717 review,
    /// 2/3): a produced run whose layout carries the REAL never-verified
    /// note must surface `unverified_geometry: true` in the v2 profile —
    /// through `Scoreboard::from_runs` + `record_geometry_verification` +
    /// `v2_profiles`, the exact pieces the shipping loop composes. If the
    /// recording call is deleted or its condition inverted, this fails.
    #[test]
    fn shipping_path_records_unverified_geometry_into_profiles() {
        let mut layout = empty_layout();
        let note = crate::bus::cells::registry::verification_note(
            "no-such-target-for-this-test",
            999.25,
            &layout,
        );
        layout.warnings.push(note);
        let produced_run = CandidateRun {
            name: CANDIDATE_ORDER[0],
            outcome: Some((
                layout,
                CandidateScore {
                    score: 1.0,
                    density: 0.1,
                    entity_count: 1,
                    overproduction: 0.0,
                    accepted: true,
                    accepted_reason: None,
                },
            )),
            events: Vec::new(),
            error: None,
            panicked: false,
        };
        let rest: Vec<CandidateRun> = CANDIDATE_ORDER
            .iter()
            .skip(1)
            .map(|n| CandidateRun::skipped(n))
            .collect();
        let runs: [&CandidateRun; 7] = [
            &produced_run,
            &rest[0],
            &rest[1],
            &rest[2],
            &rest[3],
            &rest[4],
            &rest[5],
        ];
        let mut board = Scoreboard::from_runs(runs);
        let policy = selection_policy::SelectionPolicy::current();
        board.record_geometry_verification(&runs, policy.unverified_geometry_substring);
        let profiles = board.v2_profiles();
        assert!(
            profiles[0].unverified_geometry,
            "the produced run's real note must map into the profile"
        );
        assert!(
            profiles[1..].iter().all(|p| !p.unverified_geometry),
            "non-produced rows stay unflagged by default"
        );
    }

    /// The verified direction (#717 round 2): a produced layout whose
    /// note is one of the VERIFIED tiers must map to
    /// `unverified_geometry: false` — the gear@20-keeps-its-win half of
    /// the rule, otherwise guarded only by the full e2e fixture.
    #[test]
    fn verified_note_maps_to_unflagged_profile() {
        let mut layout = empty_layout();
        // Labelled transcription of `verification_note`'s full-match arm
        // (the real arm needs a registry-hash collision to render).
        layout.warnings.push(
            "cell-composed: geometry SIM-VERIFIED at plan (spaghettio-sim gear20 cells-on — \
             PASS produced 20.00/s at declared capacity 2, 2026-08-23)"
                .into(),
        );
        let produced_run = CandidateRun {
            name: CANDIDATE_ORDER[0],
            outcome: Some((
                layout,
                CandidateScore {
                    score: 1.0,
                    density: 0.1,
                    entity_count: 1,
                    overproduction: 0.0,
                    accepted: true,
                    accepted_reason: None,
                },
            )),
            events: Vec::new(),
            error: None,
            panicked: false,
        };
        let rest: Vec<CandidateRun> = CANDIDATE_ORDER
            .iter()
            .skip(1)
            .map(|n| CandidateRun::skipped(n))
            .collect();
        let runs: [&CandidateRun; 7] = [
            &produced_run,
            &rest[0],
            &rest[1],
            &rest[2],
            &rest[3],
            &rest[4],
            &rest[5],
        ];
        let mut board = Scoreboard::from_runs(runs);
        let policy = selection_policy::SelectionPolicy::current();
        board.record_geometry_verification(&runs, policy.unverified_geometry_substring);
        assert!(
            !board.v2_profiles()[0].unverified_geometry,
            "a verified-tier note must not flag the profile"
        );
    }
}
