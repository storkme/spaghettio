//! RFC-067 Phase 3, K67-3's adjudicating harness: run the celldb template
//! candidate against the full-selection incumbent in `candidate_runner` on
//! single-motif fixtures, and RECORD the outcome. These tests assert the
//! machinery (the template produces, validates, gets verdicted); whether it
//! WINS is data printed for the RFC's decision log, deliberately not
//! asserted — K67-3 makes the null result an explicit acceptable outcome,
//! and an assertion here would be a thumb on that scale.
use rustc_hash::FxHashSet;
use spaghettio_core::bus::candidate_runner::{
    run_candidate_field, CandidateOutcome, CandidatePlan, FullSelectionCandidate,
};
use spaghettio_core::bus::layout::LayoutOptions;
use spaghettio_core::bus::template_candidate::TemplateCandidate;
use spaghettio_core::solver;
use spaghettio_core::verdict::Policy;

fn run_fixture(item: &str, rate: f64, machine: &str, inputs: &[&str]) {
    let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve(item, rate, &input_set, machine).expect("fixture must solve");
    assert_eq!(
        sr.machines.len(),
        1,
        "harness precondition: single-motif fixture (got {} groups)",
        sr.machines.len()
    );

    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let field = vec![CandidatePlan::new("celldb-template", TemplateCandidate)];
    let result = run_candidate_field(
        &sr,
        &LayoutOptions::default(),
        &incumbent,
        &field,
        &Policy::fold(),
    )
    .expect("incumbent must produce");

    let entry = result
        .entries
        .iter()
        .find(|e| e.name() == "celldb-template")
        .expect("template outcome recorded");
    match entry {
        CandidateOutcome::Refused { reason, .. } => {
            // Scope refusals (no matching entry) are expected; measurement
            // refusals from the runner ("transit is not measurable", etc.)
            // are DATA for K67-3, not test failures — panicking on them
            // hid outcomes (round-2 review). Only a stamping panic-class
            // reason (overlap/pole failure text) would indicate a bug, and
            // those surface as produce() errors with their own text.
            println!("K67-3 data: {item}@{rate}: REFUSED: {reason}");
        }
        CandidateOutcome::Evaluated(ev) => {
            println!(
                "K67-3 data: {item}@{rate}: verdict pass={} scores={:?} winner={}",
                ev.verdict.pass, ev.scores, result.winner_name
            );
        }
    }
}

#[test]
fn template_vs_incumbent_iron_plate_smelting() {
    run_fixture("iron-plate", 10.0, "electric-furnace", &["iron-ore"]);
}

#[test]
fn template_vs_incumbent_copper_cable_row() {
    run_fixture("copper-cable", 10.0, "assembling-machine-2", &["copper-plate"]);
}

#[test]
fn template_refuses_multi_group_solves_cleanly() {
    let input_set: FxHashSet<String> =
        ["iron-plate", "copper-plate"].iter().map(|s| s.to_string()).collect();
    let sr = solver::solve("electronic-circuit", 10.0, &input_set, "assembling-machine-2")
        .expect("fixture must solve");
    assert!(sr.machines.len() > 1);
    let incumbent = CandidatePlan::new("incumbent", FullSelectionCandidate);
    let field = vec![CandidatePlan::new("celldb-template", TemplateCandidate)];
    let result = run_candidate_field(
        &sr,
        &LayoutOptions::default(),
        &incumbent,
        &field,
        &Policy::fold(),
    )
    .expect("incumbent must produce");
    match result.entries.iter().find(|e| e.name() == "celldb-template").unwrap() {
        CandidateOutcome::Refused { reason, .. } => {
            assert!(reason.contains("single-group"), "wrong refusal: {reason}");
        }
        CandidateOutcome::Evaluated(_) => panic!("multi-group solve must refuse in v1"),
    }
}
