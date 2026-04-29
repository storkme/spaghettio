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

use super::layout::{run_layout_with_retry, LayoutOptions};

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
/// metrics so the trace event can report them. Phase 0 stub: hard
/// constraints always pass (`accepted: true`); Phase 1+ will check
/// demand-met and balancer-shape resolvability.
pub fn score_layout(layout: &LayoutResult, solver_result: &SolverResult) -> CandidateScore {
    let density_score = density::score_density(layout, (1, 1));
    let density_val = density_score.density;
    let entity_count = layout.entities.len();
    let overproduction = compute_overproduction(solver_result);

    let score = ALPHA_DENSITY * density_val
        - BETA_OVERPRODUCTION * overproduction
        - GAMMA_ENTITY_COUNT * (entity_count as f64);

    // Phase 0 stub. See module docs / RFP §Hard constraints. Promotes
    // to a real check (demand met, all `(n, m)` shapes resolvable) in
    // Phase 1 once a non-Native candidate exists to be rejected.
    let accepted = true;

    CandidateScore {
        score,
        density: density_val,
        entity_count,
        overproduction,
        accepted,
    }
}

/// Run every candidate, score each, pick the winner. Returns the
/// winner's layout. Errors only if no candidate produces an accepted
/// layout — which under Phase 0 (only `NativeCandidate`, always
/// accepted) means today's `build_bus_layout` itself errored, and the
/// error is propagated unchanged.
pub fn select_best_decomposition(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String> {
    // Phase 0: single-candidate catalogue. Add new impls here as they
    // land per `docs/rfp-decomposition-search.md` Phasing.
    let candidates: [&dyn DecompositionCandidate; 1] = [&NativeCandidate];

    let mut last_err: Option<String> = None;
    let mut best: Option<(LayoutResult, CandidateScore, String)> = None;

    for candidate in candidates {
        let name = candidate.name().to_string();
        match candidate.produce(solver_result, &opts) {
            Ok(layout) => {
                let score = score_layout(&layout, solver_result);
                crate::trace::emit(crate::trace::TraceEvent::DecompositionCandidateScored {
                    name: name.clone(),
                    density: score.density,
                    overproduction: score.overproduction,
                    entity_count: score.entity_count,
                    score: score.score,
                    accepted: score.accepted,
                });
                if !score.accepted {
                    continue;
                }
                let take = match &best {
                    None => true,
                    Some((_, best_score, _)) => score.score > best_score.score,
                };
                if take {
                    best = Some((layout, score, name));
                }
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    match best {
        Some((layout, score, name)) => {
            crate::trace::emit(crate::trace::TraceEvent::DecompositionChosen {
                name,
                score: score.score,
            });
            Ok(layout)
        }
        None => Err(last_err.unwrap_or_else(|| {
            "no decomposition candidate produced an acceptable layout".to_string()
        })),
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
