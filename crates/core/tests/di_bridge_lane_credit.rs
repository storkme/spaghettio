//! #607: a `stamp_di_bridge`-fed row input belt loads the FAR LANE ONLY, and
//! `check_row_input_belt_margin` must credit it one lane rather than two.
//!
//! These assert on REAL ENGINE OUTPUT, not a hand-built fixture. The unit
//! tests in `validate::inserters` pin the classifier's two branches; this
//! pins that the engine actually produces the shape, that the check fires on
//! it, and that candidate selection consequently ships the layout that
//! measures at plan. Review of PR #608 correctly flagged that without this
//! the fix was unproven on anything the engine builds.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity};

fn solve_ec10_from_plates() -> spaghettio_core::models::SolverResult {
    let inputs: FxHashSet<String> =
        ["iron-plate", "copper-plate"].iter().map(|s| s.to_string()).collect();
    solver::solve("electronic-circuit", 10.0, &inputs, "assembling-machine-2")
        .expect("ec@10 from plates must solve")
}

fn build(di: DirectInsertion) -> spaghettio_core::models::LayoutResult {
    layout::build_bus_layout(
        &solve_ec10_from_plates(),
        LayoutOptions { direct_insertion: di, ..Default::default() },
    )
    .expect("layout must build")
}

/// The DI variant really does bridge two belts with an inserter bank — the
/// construction the check exists to price. Guards against the test below
/// passing vacuously if the placer ever stops emitting the bridge.
#[test]
fn di_forced_ec10_still_builds_the_belt_to_belt_bridge() {
    let l = build(DirectInsertion::Forced);
    let bridge = l
        .entities
        .iter()
        .filter(|e| e.segment_id.as_deref().is_some_and(|s| s.starts_with("di-bridge:")))
        .count();
    assert!(
        bridge > 0,
        "di=Forced must still stamp a di-bridge for this fixture, else the \
         lane-credit assertion below is vacuous"
    );
}

/// The motivating case. Pre-fix this layout validated at ZERO issues while
/// the sim measured 90.9% of plan.
#[test]
fn di_bridge_fed_input_belt_warns_on_single_lane_credit() {
    let sr = solve_ec10_from_plates();
    let l = build(DirectInsertion::Forced);
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).expect("validate");
    let warns: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Warning && i.category == "row-input-belt-margin")
        .collect();
    assert!(
        !warns.is_empty(),
        "the di-bridge-fed copper-cable input belt must be priced at one lane; \
         got no row-input-belt-margin warning at all: {issues:?}"
    );
    assert!(
        warns.iter().any(|w| w.message.contains("fed by inserter drops (far lane only)")),
        "the warning must name the feed that caused it: {warns:?}"
    );
}

/// The bus-lane variant is straight-fed, so it must NOT pick up the warning —
/// the regression guard for the other branch on real output.
#[test]
fn bus_lane_variant_stays_clean() {
    let sr = solve_ec10_from_plates();
    let l = build(DirectInsertion::Off);
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).expect("validate");
    let warns: Vec<_> = issues
        .iter()
        .filter(|i| i.category == "row-input-belt-margin")
        .collect();
    assert!(warns.is_empty(), "straight-fed input belts keep both-lane credit: {warns:?}");
}

/// End of the causal chain: with the DI variant now carrying an issue the
/// native does not, `di_choice` stops preferring it on density. The shipped
/// layout becomes the bus-lane one, which sims at 100.0% of plan (PR #608)
/// against the bridge's 90.9%.
#[test]
fn default_candidate_selection_ships_the_bus_lane_variant() {
    let native = build(DirectInsertion::Off).entities.len();
    let bridged = build(DirectInsertion::Forced).entities.len();
    let shipped = build(DirectInsertion::Candidate).entities.len();
    assert_ne!(native, bridged, "the two variants must differ for this to mean anything");
    assert_eq!(
        shipped, native,
        "default selection must ship the straight-fed variant ({native} entities), \
         not the di-bridge one ({bridged})"
    );
}
