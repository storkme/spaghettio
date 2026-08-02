//! RFC-064 P2b: parity/behavior tests for `bus::candidate_runner` — the
//! general produce → transform → validate → measure → verdict → rank loop.
//!
//! This module ships nothing to any existing entry point: `build_bus_layout`
//! and `select_best_decomposition` are unchanged. These tests exist to prove
//! the new runner reproduces those two functions' behavior byte-for-byte
//! when `CompactTransform`/`FoldTransform` are asked to do the same work,
//! and that the runner's own scoring/verdict/ranking wiring behaves as
//! documented on synthetic transforms that a real pipeline would never
//! produce.

use spaghettio_core::bus::bands::PackObjective;
use spaghettio_core::bus::candidate_runner::{
    pack_candidate_plan, produce_plan, run_candidate_field, CandidateOutcome, CandidatePlan,
    CompactTransform, FoldTransform, FullSelectionCandidate, LayoutTransform, TransformOutcome,
};
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy};
use spaghettio_core::models::{LayoutResult, SolverResult};
use spaghettio_core::solver;
use spaghettio_core::verdict::{GatePolicy, MatchTier, Policy};

use rustc_hash::FxHashSet;

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Mirrors `fold_layout_knob.rs`'s `base_opts` exactly, so the same fixture
/// produces the same geometry across both test files.
fn base_opts(belt_tier: Option<&str>, strategy: LayoutStrategy) -> LayoutOptions {
    LayoutOptions {
        strategy,
        surplus_policy: Default::default(),
        max_belt_tier: belt_tier.map(|s| s.to_string()),
        row_layout: Default::default(),
        max_inserter_tier: Default::default(),
        quality: Default::default(),
        wire_mode: Default::default(),
        merge_tap: false,
        stacking: 1,
        inserter_capacity: 0,
        cell_composition: Default::default(),
        splitter_tap_spacers: false,
        horizontal_candidate: true,
        ..Default::default()
    }
}

fn solve(item: &str, rate: f64, inputs: &[&str], machine: &str) -> SolverResult {
    solver::solve_with_exclusions(item, rate, &set(inputs), machine, &FxHashSet::default())
        .unwrap_or_else(|e| panic!("solve {item}@{rate}/s ({machine}): {e}"))
}

fn json_of(l: &LayoutResult) -> serde_json::Value {
    serde_json::to_value(l).expect("LayoutResult must serialize")
}

/// The RFC-064 Phase 1 spike's own admissible-and-AR-improving fixture
/// (`fold_layout_knob.rs`'s `fold_layout_finds_admissible_fold_on_stress_ac_partitioned`):
/// advanced-circuit@5/s from plates, AM2, Pooled, no belt tier cap. Cheap
/// (a few hundred entities) and — unlike most fixtures — actually finds an
/// admissible fold, so it exercises `FoldTransform::apply`'s `Some(found)`
/// branch rather than only its fallback.
fn stress_ac_partitioned() -> SolverResult {
    solve(
        "advanced-circuit",
        5.0,
        &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
        "assembling-machine-2",
    )
}

// ---------------------------------------------------------------------------
// 1. Fold parity
// ---------------------------------------------------------------------------

#[test]
fn runner_fold_plan_matches_build_bus_layout_fold_layout_true() {
    let sr = stress_ac_partitioned();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let native_layout = build_bus_layout(
        &sr,
        LayoutOptions {
            fold_layout: true,
            ..opts.clone()
        },
    )
    .expect("build_bus_layout with fold_layout: true must succeed");

    let plan = CandidatePlan::new("compact-fold", FullSelectionCandidate)
        .with_transform(CompactTransform)
        .with_transform(FoldTransform::default());

    // `fold_layout: true` applies compact-then-fold UNCONDITIONALLY in
    // `build_bus_layout` — there is no "does this beat native" gate at that
    // call site, so parity means "what the CHAIN produces", not "what
    // `run_candidate_field`'s own objective-driven ranking would have
    // picked". `produce_plan` runs the chain with no competition, matching
    // that unconditional-apply semantics exactly.
    let produced = produce_plan(&plan, &sr, &opts).expect("produce_plan must succeed");
    assert_eq!(
        json_of(&produced),
        json_of(&native_layout),
        "runner's [compact, fold] plan must reproduce build_bus_layout(fold_layout: true) byte-for-byte"
    );

    // Bonus signal (not load-bearing for parity): on THIS fixture — the
    // spike's own admissible-and-AR-improving case — the fold is such a
    // clear aspect-ratio win that the runner's own objective ranking picks
    // it over the incumbent too, confirming the objective genuinely rewards
    // a fold that is known-good rather than merely being inert about it.
    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&plan),
        &Policy::fold(),
    )
    .expect("run_candidate_field must succeed");
    assert_eq!(
        result.winner_name, "compact-fold",
        "on this fixture the fold is a clear AR win, so the runner's own ranking should pick it"
    );
}

// ---------------------------------------------------------------------------
// 2. Compact parity
// ---------------------------------------------------------------------------

#[test]
fn runner_compact_plan_matches_build_bus_layout_compact_layout_true() {
    let sr = stress_ac_partitioned();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let native_layout = build_bus_layout(
        &sr,
        LayoutOptions {
            compact_layout: true,
            ..opts.clone()
        },
    )
    .expect("build_bus_layout with compact_layout: true must succeed");

    let plan =
        CandidatePlan::new("compact-only", FullSelectionCandidate).with_transform(CompactTransform);

    // Same reasoning as the fold test: `compact_layout: true` applies
    // unconditionally in `build_bus_layout`, so parity is about the CHAIN's
    // own output, via `produce_plan` — not about whether this fixture's
    // objective-score ranking happens to prefer the compacted geometry over
    // native (measured separately below: it does NOT always, since raw
    // compaction legally shrinks entity count within
    // `compact_validated_geometry`'s own error-count-only acceptance rule
    // without any guarantee of improving RFC-064's aspect-ratio/transit
    // metric — that is an honest, expected outcome, not a bug in either
    // compaction or the objective).
    let produced = produce_plan(&plan, &sr, &opts).expect("produce_plan must succeed");
    assert_eq!(
        json_of(&produced),
        json_of(&native_layout),
        "runner's [compact] plan must reproduce build_bus_layout(compact_layout: true) byte-for-byte"
    );

    // Exercise the full run_candidate_field pipeline too (validate/measure/
    // verdict), without asserting on who wins: the candidate must at least
    // evaluate cleanly and pass the fold-style verdict (compaction never
    // increases the raw issue count in any category, by
    // `compact_validated_geometry`'s own accept-if-no-worse discipline).
    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&plan),
        &Policy::fold(),
    )
    .expect("run_candidate_field must succeed");
    let evaluated = result
        .entries
        .iter()
        .find_map(|e| match e {
            CandidateOutcome::Evaluated(ec) if ec.name == "compact-only" => Some(ec),
            _ => None,
        })
        .expect("compact-only must have produced and evaluated");
    assert!(
        evaluated.verdict.pass,
        "compaction must never regress any issue category's raw count"
    );
}

// ---------------------------------------------------------------------------
// 3. Inertness
// ---------------------------------------------------------------------------

#[test]
fn runner_incumbent_only_field_matches_plain_build_bus_layout() {
    let sr = stress_ac_partitioned();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let native_layout =
        build_bus_layout(&sr, opts.clone()).expect("plain build_bus_layout must succeed");

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let result = run_candidate_field(&sr, &opts, &incumbent, &[], &Policy::fold())
        .expect("run_candidate_field must succeed with an empty field");

    assert_eq!(result.winner_name, "incumbent");
    assert_eq!(
        json_of(&result.winner),
        json_of(&native_layout),
        "an empty field must return the incumbent's layout byte-identical to plain build_bus_layout"
    );
    assert!(result.entries.is_empty());
}

// ---------------------------------------------------------------------------
// Synthetic test-local transforms for 4 and 5
// ---------------------------------------------------------------------------

/// Doubles the bbox by translating every other entity far to the east.
/// Deliberately crude — this is not meant to be a plausible transform, only
/// one whose objective score is unambiguously worse than doing nothing.
struct BboxDoublingTransform;

impl LayoutTransform for BboxDoublingTransform {
    fn name(&self) -> &str {
        "test-bbox-doubler"
    }

    fn admissible_input(&self, _layout: &LayoutResult) -> Result<(), String> {
        Ok(())
    }

    fn apply(
        &self,
        layout: &LayoutResult,
        _solver: &SolverResult,
        _opts: &LayoutOptions,
    ) -> Result<TransformOutcome, String> {
        let mut out = layout.clone();
        let shove = out.width.max(1) * 2;
        for (i, e) in out.entities.iter_mut().enumerate() {
            if i % 2 == 0 {
                e.x += shove;
            }
        }
        out.width += shove;
        Ok(TransformOutcome {
            layout: out,
            correspondence: None,
            tier: MatchTier::Count,
        })
    }
}

/// Moves the entity with the largest X coordinate onto the entity with the
/// smallest X coordinate. This (a) shrinks the bounding box — a genuine,
/// measurable objective improvement on any layout with more than one
/// distinct X value — and (b) deliberately creates exactly one
/// `entity-overlap` error, the gated regression test 5 exists to catch.
struct ShrinkOntoOverlapTransform;

impl LayoutTransform for ShrinkOntoOverlapTransform {
    fn name(&self) -> &str {
        "test-shrink-onto-overlap"
    }

    fn admissible_input(&self, _layout: &LayoutResult) -> Result<(), String> {
        Ok(())
    }

    fn apply(
        &self,
        layout: &LayoutResult,
        _solver: &SolverResult,
        _opts: &LayoutOptions,
    ) -> Result<TransformOutcome, String> {
        let mut out = layout.clone();
        let n = out.entities.len();
        let min_idx = (0..n)
            .min_by_key(|&i| out.entities[i].x)
            .expect("non-empty layout");
        let max_idx = (0..n)
            .max_by_key(|&i| out.entities[i].x)
            .expect("non-empty layout");
        assert_ne!(
            min_idx, max_idx,
            "fixture must have more than one distinct X coordinate for this transform to mean anything"
        );
        let (tx, ty) = (out.entities[min_idx].x, out.entities[min_idx].y);
        out.entities[max_idx].x = tx;
        out.entities[max_idx].y = ty;
        Ok(TransformOutcome {
            layout: out,
            correspondence: None,
            tier: MatchTier::Count,
        })
    }
}

fn tier1_gear_from_ore() -> SolverResult {
    solve(
        "iron-gear-wheel",
        10.0,
        &["iron-ore"],
        "assembling-machine-2",
    )
}

// ---------------------------------------------------------------------------
// 4. Native-wins-by-construction
// ---------------------------------------------------------------------------

#[test]
fn degrading_transform_loses_the_ranking_even_when_verdict_passes() {
    let sr = tier1_gear_from_ore();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let candidate = CandidatePlan::new("bbox-doubler", FullSelectionCandidate)
        .with_transform(BboxDoublingTransform);

    // ReportOnly: the point of this test is the SCORE-based rejection, not
    // the verdict-based one (that is test 5's job) — a policy that never
    // regresses isolates the mechanism under test.
    let policy = Policy::new(GatePolicy::ReportOnly);
    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&candidate),
        &policy,
    )
    .expect("run_candidate_field must succeed");

    let evaluated = result
        .entries
        .iter()
        .find_map(|e| match e {
            CandidateOutcome::Evaluated(ec) if ec.name == "bbox-doubler" => Some(ec),
            _ => None,
        })
        .expect("bbox-doubler must have produced and evaluated");
    assert!(
        evaluated.verdict.pass,
        "ReportOnly policy must never fail the verdict"
    );
    assert!(
        evaluated.scores.composite < 0.0,
        "a candidate with a far worse aspect ratio must score a negative composite, got {}",
        evaluated.scores.composite
    );
    assert_eq!(
        result.winner_name, "incumbent",
        "the incumbent (composite 0.0 by construction) must outrank a verdict-passing \
         candidate whose composite is negative"
    );
}

// ---------------------------------------------------------------------------
// 5. Verdict wiring
// ---------------------------------------------------------------------------

#[test]
fn new_gated_issue_excludes_a_candidate_even_with_a_better_composite() {
    let sr = tier1_gear_from_ore();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let candidate = CandidatePlan::new("shrink-overlap", FullSelectionCandidate)
        .with_transform(ShrinkOntoOverlapTransform);

    // Default policy: GateInstances on every category — a brand new
    // "entity-overlap" issue (native has none) must regress.
    let policy = Policy::new(GatePolicy::GateInstances);
    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&candidate),
        &policy,
    )
    .expect("run_candidate_field must succeed");

    let evaluated = result
        .entries
        .iter()
        .find_map(|e| match e {
            CandidateOutcome::Evaluated(ec) if ec.name == "shrink-overlap" => Some(ec),
            _ => None,
        })
        .expect("shrink-overlap must have produced and evaluated");

    assert!(
        evaluated.scores.composite > 0.0,
        "the shrink must genuinely improve the composite for this test to mean anything, got {}",
        evaluated.scores.composite
    );
    assert!(
        !evaluated.verdict.pass,
        "a brand new entity-overlap error must fail the verdict"
    );
    assert!(
        evaluated
            .verdict
            .categories
            .get("entity-overlap")
            .is_some_and(|o| o.regressed),
        "entity-overlap specifically must be the regressed category, got: {:?}",
        evaluated.verdict.categories.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        result.winner_name, "incumbent",
        "a verdict-failing candidate must be excluded from ranking regardless of composite"
    );
}

// ---------------------------------------------------------------------------
// 6. RFC-064 Phase 3: PackCandidate
// ---------------------------------------------------------------------------

/// electronic-circuit@1/s from ore, AM2, yellow belt tier, Pooled — small
/// (126 entities) and, unlike `stress_ac_partitioned`/`tier1_gear_from_ore`,
/// actually clears `bands::packing_refusal` (single-lane items throughout,
/// >= 3 bands: iron-plate, copper-plate, copper-cable, electronic-circuit).
fn ec_1_am2_yellow() -> SolverResult {
    solve(
        "electronic-circuit",
        1.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-2",
    )
}

/// Smoke test: `run_candidate_field` with a native incumbent and a
/// `pack_candidate_plan` field entry, on a fixture the packer builds
/// successfully. Asserts the pack candidate reaches evaluation (produces,
/// validates, measures, verdicts) and carries a genuine `Provenance`-tier
/// verdict from its paired `PackCorrespondenceTransform` — NOT who wins,
/// which is fixture-dependent and Unit B's adjudication to make.
#[test]
fn runner_pack_candidate_produces_measures_and_verdicts() {
    let sr = ec_1_am2_yellow();
    let opts = base_opts(Some("yellow"), LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let pack_plan = pack_candidate_plan("pack", PackObjective::MinAspectRatio);

    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&pack_plan),
        &Policy::fold(),
    )
    .expect("run_candidate_field must succeed");

    let evaluated = result
        .entries
        .iter()
        .find_map(|e| match e {
            CandidateOutcome::Evaluated(ec) if ec.name == "pack" => Some(ec),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "pack candidate must produce and evaluate on this fixture — entries: {:?}",
                result.entries.iter().map(CandidateOutcome::name).collect::<Vec<_>>()
            )
        });

    assert!(
        evaluated.scores.composite.is_finite(),
        "ObjectiveScores must be a real measurement, not NaN/inf"
    );
    assert_eq!(
        evaluated.verdict.tier,
        MatchTier::Provenance,
        "PackCorrespondenceTransform must supply a genuine Provenance-tier map on a fixture that packs with pure per-band translation"
    );
}

/// `PackCandidate` alone (no paired transform) must verdict at
/// `MatchTier::Count` — `compose_chain`'s own "empty chain" rule — proving
/// the `Provenance` tier above comes from the paired transform, not from
/// `PackCandidate` itself claiming precision it cannot support standalone.
#[test]
fn runner_pack_candidate_without_its_transform_is_count_tier() {
    use spaghettio_core::bus::candidate_runner::PackCandidate;

    let sr = ec_1_am2_yellow();
    let opts = base_opts(Some("yellow"), LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let bare_plan = CandidatePlan::new("pack-bare", PackCandidate::new(PackObjective::MinAspectRatio));

    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&bare_plan),
        &Policy::fold(),
    )
    .expect("run_candidate_field must succeed");

    let evaluated = result
        .entries
        .iter()
        .find_map(|e| match e {
            CandidateOutcome::Evaluated(ec) if ec.name == "pack-bare" => Some(ec),
            _ => None,
        })
        .expect("pack-bare must have produced and evaluated");
    assert_eq!(evaluated.verdict.tier, MatchTier::Count);
}

/// Refusals surface as a clean `CandidateOutcome::Refused` carrying the
/// packer's own reason — `stress_ac_partitioned` refuses on a multi-lane
/// copper-cable item (`PackRefusal::MultiLaneItem`), a DIFFERENT typed
/// refusal than the entity-count budget guard.
#[test]
fn runner_pack_candidate_refusal_surfaces_the_packers_own_reason() {
    let sr = stress_ac_partitioned();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let pack_plan = pack_candidate_plan("pack", PackObjective::MinAspectRatio);

    let result = run_candidate_field(
        &sr,
        &opts,
        &incumbent,
        std::slice::from_ref(&pack_plan),
        &Policy::fold(),
    )
    .expect("run_candidate_field must succeed — a refused field candidate must not fail the whole call");

    let reason = result
        .entries
        .iter()
        .find_map(|e| match e {
            CandidateOutcome::Refused { name, reason } if name == "pack" => Some(reason.clone()),
            _ => None,
        })
        .expect("pack must be Refused, not Evaluated, on this fixture");
    assert!(
        reason.contains("exceeds one") && reason.contains("lane"),
        "the refusal reason must be the packer's own MultiLaneItem message, got: {reason}"
    );
    assert_eq!(result.winner_name, "incumbent");
}
