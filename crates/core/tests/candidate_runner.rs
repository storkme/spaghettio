//! RFC-064 P2b: parity/behavior tests for `bus::candidate_runner` — the
//! general produce → validate → measure → verdict → rank loop.
//!
//! This module ships nothing to any existing entry point: `build_bus_layout`
//! and `select_best_decomposition` are unchanged; `run_candidate_field` is a
//! parallel entry point consumed today by this crate's own tests and by the
//! live RFC-068 celldb campaign (`crates/core/tests/celldb_template.rs`).
//!
//! Originally this file also carried parity tests proving the runner's
//! `LayoutTransform` chain (`CompactTransform`/`FoldTransform`) reproduced
//! `build_bus_layout`'s `compact_layout`/`fold_layout` flags byte-for-byte,
//! plus two ranking/verdict tests built on synthetic `LayoutTransform` impls.
//! The transform mechanism and both flags were deleted 2026-08-14 (#632 A2,
//! owner call) after the underlying relocation research never shipped past
//! three falsified attempts (RFC-057/058/064-P3 decision logs) — the parity
//! tests went with it (there is nothing left to prove parity against; one of
//! them, `runner_fold_plan_matches_build_bus_layout_fold_layout_true`, was
//! also this suite's slowest test at >60s). The two ranking/verdict tests
//! (4 and 5 below) were PORTED rather than deleted: their synthetic
//! `LayoutTransform` impls became synthetic `DecompositionCandidate` base
//! producers instead, since what they actually exercise — "a candidate whose
//! score/verdict differs from the incumbent's" — never depended on the
//! transform abstraction specifically.

use spaghettio_core::bus::candidate_runner::{
    run_candidate_field, CandidateOutcome, CandidatePlan, FullSelectionCandidate,
};
use spaghettio_core::bus::decomposition_search::DecompositionCandidate;
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy};
use spaghettio_core::models::{LayoutResult, PlacedEntity, SolverResult};
use spaghettio_core::solver;
use spaghettio_core::verdict::{GatePolicy, Policy};

use rustc_hash::FxHashSet;

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Mirrors what used to be `fold_layout_knob.rs`'s `base_opts` (deleted
/// alongside `fold_layout` itself, #632 A2), kept here so this file's
/// fixtures stay self-contained.
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

// ---------------------------------------------------------------------------
// 1. Inertness
// ---------------------------------------------------------------------------

#[test]
fn runner_incumbent_only_field_matches_plain_build_bus_layout() {
    let sr = solve(
        "advanced-circuit",
        5.0,
        &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
        "assembling-machine-2",
    );
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
// Synthetic test-local base producers for 2 and 3
// ---------------------------------------------------------------------------

/// Doubles the bbox by translating one entity far to the east, after normal
/// production. Deliberately crude — this is not meant to be a plausible
/// strategy, only one whose objective score is unambiguously worse than
/// doing nothing.
struct BboxDoublingCandidate;

impl DecompositionCandidate for BboxDoublingCandidate {
    fn name(&self) -> &str {
        "test-bbox-doubler"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        let mut out = FullSelectionCandidate.produce(solver_result, opts)?;
        // Degrade the aspect ratio by EXTENDING the footprint, not by tearing
        // the network apart.
        //
        // Appending one isolated belt tile far to the east grows the non-pole
        // bbox — so `ar_score` genuinely worsens — while leaving every routed
        // path exactly as it was, so the candidate still measures and the
        // ranking has a real negative composite to reject.
        let shove = out.width.max(1) * 2;
        let far = out
            .entities
            .iter()
            .map(|e| e.x)
            .max()
            .unwrap_or(0)
            + shove;
        let anchor_y = out.entities.first().map(|e| e.y).unwrap_or(0);
        out.entities.push(PlacedEntity {
            name: "transport-belt".to_string(),
            x: far,
            y: anchor_y,
            ..Default::default()
        });
        out.width += shove;
        Ok(out)
    }
}

/// Moves the entity with the largest X coordinate onto the entity with the
/// smallest X coordinate, after normal production. This (a) shrinks the
/// bounding box — a genuine, measurable objective improvement on any layout
/// with more than one distinct X value — and (b) deliberately creates
/// exactly one `entity-overlap` error, the gated regression test 3 exists to
/// catch.
struct ShrinkOntoOverlapCandidate;

impl DecompositionCandidate for ShrinkOntoOverlapCandidate {
    fn name(&self) -> &str {
        "test-shrink-onto-overlap"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        let mut out = FullSelectionCandidate.produce(solver_result, opts)?;
        let n = out.entities.len();
        let min_idx = (0..n)
            .min_by_key(|&i| out.entities[i].x)
            .expect("non-empty layout");
        let max_idx = (0..n)
            .max_by_key(|&i| out.entities[i].x)
            .expect("non-empty layout");
        assert_ne!(
            min_idx, max_idx,
            "fixture must have more than one distinct X coordinate for this candidate to mean anything"
        );
        let (tx, ty) = (out.entities[min_idx].x, out.entities[min_idx].y);
        out.entities[max_idx].x = tx;
        out.entities[max_idx].y = ty;
        Ok(out)
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
// 2. Native-wins-by-construction
// ---------------------------------------------------------------------------

#[test]
fn degrading_candidate_loses_the_ranking_even_when_verdict_passes() {
    let sr = tier1_gear_from_ore();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let candidate = CandidatePlan::new("bbox-doubler", BboxDoublingCandidate);

    // ReportOnly: the point of this test is the SCORE-based rejection, not
    // the verdict-based one (that is test 3's job) — a policy that never
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

    // Requires the EVALUATED branch specifically. An earlier cut of this
    // accepted "Evaluated with a negative composite OR Refused", which made it
    // tautological — it passed whether the score-based rejection fired or the
    // transit-refusal path fired instead, and the score path is the one this
    // test is named for (PR #582 review, 3/3). The candidate now degrades the
    // bbox without severing any route, so it is measurable by construction
    // and a Refused outcome is a real failure worth surfacing.
    let evaluated = match result
        .entries
        .iter()
        .find(|e| matches!(e, CandidateOutcome::Evaluated(ec) if ec.name == "bbox-doubler")
            || matches!(e, CandidateOutcome::Refused { name, .. } if name == "bbox-doubler"))
        .expect("bbox-doubler must appear in the field")
    {
        CandidateOutcome::Evaluated(ec) => ec,
        CandidateOutcome::Refused { reason, .. } => panic!(
            "bbox-doubler must be MEASURABLE so the score path is exercised; it was \
             refused: {reason}"
        ),
    };
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
// 3. Verdict wiring
// ---------------------------------------------------------------------------

#[test]
fn new_gated_issue_excludes_a_candidate_even_with_a_better_composite() {
    let sr = tier1_gear_from_ore();
    let opts = base_opts(None, LayoutStrategy::Pooled);

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let candidate = CandidatePlan::new("shrink-overlap", ShrinkOntoOverlapCandidate);

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
        .unwrap_or_else(|| {
            // Not a bare `expect`: under the conforming metric a candidate can
            // now be REFUSED (unmeasurable transit) rather than evaluated, and
            // this fixture's measurability is an assumption, not a guarantee.
            // Say which it was, so a future fixture drift reads as a fixture
            // problem rather than a missing candidate (PR #582 review).
            let outcomes: Vec<String> = result
                .entries
                .iter()
                .map(|e| match e {
                    CandidateOutcome::Evaluated(ec) => format!("Evaluated({})", ec.name),
                    CandidateOutcome::Refused { name, reason } => {
                        format!("Refused({name}: {reason})")
                    }
                })
                .collect();
            panic!("shrink-overlap must have produced and evaluated; field was {outcomes:?}");
        });

    // PREMISE WEAKENED, deliberately and visibly. This used to assert
    // `composite > 0.0` — the candidate had to genuinely out-score the
    // incumbent for "the gate beats the score" to mean anything. Under the
    // conforming metric it no longer does: the old non-conforming measurement
    // credited this shrink with a transit improvement it does not physically
    // have. Rather than tune the fixture until the number comes back, the
    // premise is recorded as lost and the gate assertion below — which is what
    // this test is actually for — is kept intact.
    //
    // FOLLOW-UP: rebuild a synthetic candidate that both improves the
    // composite AND leaves the layout measurable, and restore the premise.
    // Until then this test proves the gate excludes a verdict-failing
    // candidate, but not that it does so *against* a winning score.
    let composite = evaluated.scores.composite;
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
        "a verdict-failing candidate must be excluded from ranking regardless of \
         composite (candidate scored {composite})"
    );
}

// ---------------------------------------------------------------------------
// 4. Duplicate plan names are refused (round-3 bot review, minor 5)
// ---------------------------------------------------------------------------

/// Winner resolution and event replay look candidates up by name; two
/// same-named plans would tie in ranking and replay the wrong plan's
/// events/layout. The runner must refuse the field up front.
#[test]
fn duplicate_plan_names_are_refused() {
    let sr = solve("iron-gear-wheel", 1.0, &["iron-plate"], "assembling-machine-2");
    let opts = base_opts(None, LayoutStrategy::Pooled);
    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let field = vec![
        CandidatePlan::new("dup", FullSelectionCandidate),
        CandidatePlan::new("dup", FullSelectionCandidate),
    ];
    match run_candidate_field(&sr, &opts, &incumbent, &field, &Policy::fold()) {
        Err(err) => assert!(
            err.contains("duplicate candidate plan name 'dup'"),
            "got: {err}"
        ),
        Ok(_) => panic!("same-named plans must be refused"),
    }
}

// ---------------------------------------------------------------------------
// 5. The incumbent-unmeasurable abort path (PR #582 review, 3/3, twice)
// ---------------------------------------------------------------------------

/// A base candidate that produces a layout with the right machines and NO
/// transport at all, so every production edge has an unreachable terminal.
struct UnmeasurableCandidate;

impl DecompositionCandidate for UnmeasurableCandidate {
    fn name(&self) -> &str {
        "unmeasurable-incumbent"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        let full = build_bus_layout(solver_result, opts.clone())?;
        // Strip every belt, underground, splitter, pipe and inserter: the
        // machines remain, so the production edges still exist, but nothing
        // can carry an item between them.
        let mut out = full.clone();
        out.entities.retain(|e| {
            let n = e.name.as_str();
            !(n.contains("transport-belt")
                || n.contains("underground-belt")
                || n.contains("splitter")
                || n.contains("pipe")
                || n.contains("inserter"))
        });
        Ok(out)
    }
}

/// Delegating §(b) to `bus::transit` made `measure` fallible for real, which
/// made this abort path reachable: `run_candidate_field` fails the WHOLE field
/// when the incumbent cannot be measured, rather than ranking candidates
/// against a partially-measured baseline.
///
/// That is the §(b) conformance being bought — scoring against a
/// partially-measured incumbent is exactly the silent-proxy behaviour the spec
/// forbids — but the decision log recorded it as "designed, not verified"
/// because no fixture exercised it. This is that fixture. Reviewers flagged
/// the gap twice at 3/3; an availability change that only exists in prose is
/// the kind of thing that gets rediscovered in anger.
#[test]
fn an_unmeasurable_incumbent_fails_the_field_with_a_named_cause() {
    let sr = tier1_gear_from_ore();
    let opts = base_opts(None, LayoutStrategy::Pooled);
    let incumbent = CandidatePlan::new("incumbent", UnmeasurableCandidate);
    let policy = Policy::decomposition();

    // `FieldResult` is not Debug, so match rather than `expect_err`.
    let err = match run_candidate_field(&sr, &opts, &incumbent, &[], &policy) {
        Err(e) => e,
        Ok(f) => panic!(
            "an incumbent that cannot be measured must fail the field, got winner {:?}",
            f.winner_name
        ),
    };

    assert!(
        err.contains("incumbent"),
        "the failure must say it was the INCUMBENT that could not be measured, \
         not merely that something failed: {err}"
    );
    assert!(
        err.contains("transit is not measurable"),
        "and it must carry bus::transit's cause through, so the operator can \
         see which edge broke: {err}"
    );
}
