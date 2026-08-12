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

/// Rate that demands EXACTLY `target_count` machines: machine count is
/// linear in rate, so one probe solve at 1.0/s calibrates it. Demand-
/// matching makes K67-3 adjudicate fragment QUALITY — an unscaled stamp
/// against a smaller demand scored overproduction, not the fragment, and
/// the verdicts were foregone (round-3 review, both earlier rounds flagged
/// the same shape).
fn rate_for_count(item: &str, machine: &str, inputs: &FxHashSet<String>, target_count: u32) -> f64 {
    let probe = solver::solve(item, 1.0, inputs, machine).expect("probe solve");
    let count_per_rate = probe.machines[0].count;
    // Nudge below the exact boundary so ceil() lands on target_count.
    (target_count as f64 - 0.01) / count_per_rate
}

fn run_fixture(item: &str, entry_count: u32, machine: &str, inputs: &[&str]) {
    let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let rate = rate_for_count(item, machine, &input_set, entry_count);
    let sr = solver::solve(item, rate, &input_set, machine).expect("fixture must solve");
    assert_eq!(
        sr.machines[0].count.ceil() as u32,
        entry_count,
        "demand-matching failed: harness must request exactly the seeded count"
    );
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
            // These fixtures are demand-matched to seeded entries — a
            // refusal here is a REGRESSION, not data: the entry exists,
            // the count matches, so produce() must reach evaluation. A
            // harness that println!s every outcome and passes is the
            // check-going-quiet failure validator-reporting.md documents
            // (round-5 review, 2/2). Scope refusals remain legitimate only
            // in the multi-group test below.
            panic!("demand-matched {item} (count={entry_count}) refused: {reason}");
        }
        CandidateOutcome::Evaluated(ev) => {
            println!(
                "K67-3 data (demand-matched, count={entry_count}): {item}@{rate:.2}: verdict pass={} scores={:?} winner={}",
                ev.verdict.pass, ev.scores, result.winner_name
            );
        }
    }
}

#[test]
fn template_vs_incumbent_iron_plate_smelting_demand_matched() {
    // 32 = the seeded iron-plate entry's machine count.
    run_fixture("iron-plate", 32, "electric-furnace", &["iron-ore"]);
}

#[test]
fn template_vs_incumbent_copper_cable_row_demand_matched() {
    // 20 = the seeded copper-cable entry's machine count.
    run_fixture("copper-cable", 20, "assembling-machine-2", &["copper-plate"]);
}

#[test]
fn template_vs_incumbent_copper_plate_smelting_demand_matched() {
    // 48 = the seeded copper-plate entry's machine count. Added after a
    // review round showed the K67-3 discharge skipped a realizable motif:
    // of the five seeds, exactly three (iron-plate, copper-plate,
    // copper-cable) admit single-group demands — ec/ac always co-solve
    // with their cable/circuit inputs and refuse by v1 design.
    //
    // Since the donor probe (RFC-067 decision log 2026-08-12), count 48 is
    // shared with community donor -OL38TX27JmPIivo#8, which wins the
    // (count, interior_tiles) sort at 753 < 817 — so THIS fixture stamps
    // the DONOR, per the pre-registered collision rule. The engine seed's
    // own tie verdict stays recorded in the decision log; re-deriving it
    // requires a store without the donor row.
    run_fixture("copper-plate", 48, "electric-furnace", &["copper-ore"]);
}

#[test]
fn template_vs_incumbent_copper_plate_donor_lxr_demand_matched() {
    // 50 = community donor -Lxr8KxgJup5AsKyTygI#6 ("Mass Plate
    // Production", RFC-067 donor probe): the only count-50 entry, so the
    // stamp is the donor by construction.
    run_fixture("copper-plate", 50, "electric-furnace", &["copper-ore"]);
}

#[test]
fn template_vs_incumbent_copper_plate_donor_on0_demand_matched() {
    // 52 = community donor -ON0-7RmLQJjDfgq7s_W#2 (vertical express
    // column cell, RFC-067 donor probe): the only count-52 entry.
    run_fixture("copper-plate", 52, "electric-furnace", &["copper-ore"]);
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
