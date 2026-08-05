//! RFC-065 Phase 0 gates: K65-1 (parity — zero IR findings on green
//! layouts) and K65-2 (detection — reconstructed historical failure classes
//! fire), plus the regression pin for the `effective_rows` compaction fix.
//!
//! Parity fixtures are built through the real pipeline (`solver` →
//! `build_bus_layout`) and asserted validator-green first, so a red parity
//! assertion means the IR disagrees with the engine's semantics — the K65-1
//! stop signal — not that the fixture drifted.

use rustc_hash::FxHashSet;

use spaghettio_core::bus::compaction::compact_validated_geometry;
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::connectivity::{
    check_record_integrity, derive_connectivity, diff, scan_graph_anomalies,
};
use spaghettio_core::models::{LayoutResult, SolverResult};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity, ValidationIssue};

fn build(
    item: &str,
    rate: f64,
    machine: &str,
    inputs: &[&str],
    opts: LayoutOptions,
) -> (SolverResult, LayoutResult) {
    build_with_exclusions(item, rate, machine, inputs, &[], opts)
}

fn build_with_exclusions(
    item: &str,
    rate: f64,
    machine: &str,
    inputs: &[&str],
    excluded: &[&str],
    opts: LayoutOptions,
) -> (SolverResult, LayoutResult) {
    let inputs: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let excluded: FxHashSet<String> = excluded.iter().map(|s| s.to_string()).collect();
    let solver_result = solver::solve_with_exclusions(item, rate, &inputs, machine, &excluded)
        .unwrap_or_else(|e| panic!("solve {item}: {e:?}"));
    let layout = build_bus_layout(&solver_result, opts)
        .unwrap_or_else(|e| panic!("layout {item}: {e}"));
    (solver_result, layout)
}

fn issues_of(layout: &LayoutResult, solver_result: &SolverResult) -> Vec<ValidationIssue> {
    match validate::validate(layout, Some(solver_result), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(error) => error.issues,
    }
}

/// Assert the fixture is validator-green (no errors; residual warnings are
/// corpus-normal), then assert the IR sees nothing either: no graph
/// anomalies, no record-integrity findings.
fn assert_green_and_ir_parity(name: &str, layout: &LayoutResult, solver_result: &SolverResult) {
    let errors: Vec<_> = issues_of(layout, solver_result)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "{name}: fixture is not validator-green — parity is only meaningful on green layouts: {errors:#?}"
    );

    let graph = derive_connectivity(layout);
    let anomalies = scan_graph_anomalies(&graph, layout);
    assert!(
        anomalies.is_empty(),
        "{name}: K65-1 — graph anomalies on a validator-green layout: {anomalies:#?}"
    );
    let integrity = check_record_integrity(layout);
    assert!(
        integrity.is_empty(),
        "{name}: K65-1 — record-integrity findings on a validator-green layout: {integrity:#?}"
    );
    // Positive structural floor (PR #574 bot review: absence-only parity
    // is the check-went-quiet shape). Every engine fixture moves items on
    // belts via inserters, so a derivation that silently drops or
    // mis-classifies these edge kinds must fail here, not pass quietly.
    for kind in [
        spaghettio_core::connectivity::EdgeKind::BeltFlow,
        spaghettio_core::connectivity::EdgeKind::InserterPickup,
        spaghettio_core::connectivity::EdgeKind::InserterDrop,
    ] {
        assert!(
            graph.edges.iter().any(|e| e.kind == kind),
            "{name}: derived graph has zero {kind:?} edges — derivation went quiet"
        );
    }
    // Exact count invariant (bot round 5: presence alone tolerates
    // over-emission): on a validator-green layout every inserter has
    // exactly one pickup and one drop binding, so each hand-edge kind
    // counts exactly the inserter population — over- or under-emission
    // of either kind fails here.
    let inserter_count = layout
        .entities
        .iter()
        .filter(|e| spaghettio_core::common::is_inserter(&e.name))
        .count();
    for kind in [
        spaghettio_core::connectivity::EdgeKind::InserterPickup,
        spaghettio_core::connectivity::EdgeKind::InserterDrop,
    ] {
        let n = graph.edges.iter().filter(|e| e.kind == kind).count();
        assert_eq!(
            n, inserter_count,
            "{name}: {kind:?} edge count must equal the inserter population"
        );
    }
}

#[test]
fn parity_tier1_gear_from_plates() {
    let (sr, layout) = build(
        "iron-gear-wheel",
        1.0,
        "assembling-machine-2",
        &["iron-plate"],
        LayoutOptions::from_belt_tier(None),
    );
    assert_green_and_ir_parity("tier1-gear", &layout, &sr);
}

#[test]
fn parity_tier2_electronic_circuit_from_ore() {
    let (sr, layout) = build(
        "electronic-circuit",
        2.0,
        "assembling-machine-2",
        &["iron-ore", "copper-ore"],
        LayoutOptions::from_belt_tier(None),
    );
    assert_green_and_ir_parity("tier2-ec-from-ore", &layout, &sr);
}

#[test]
fn parity_fluid_plastic_bar() {
    let (sr, layout) = build(
        "plastic-bar",
        2.0,
        "assembling-machine-2",
        &["coal", "petroleum-gas"],
        LayoutOptions::from_belt_tier(None),
    );
    assert_green_and_ir_parity("fluid-plastic", &layout, &sr);
}

/// Review defect 3: the original 4-fixture set could not discriminate on
/// junction zones or multi-tap lanes — the geometry that falsified the
/// removed RI-2. These two fixtures carry both (and one is an in-corpus
/// tier regression fixture), so the K65-1 gate now covers the shapes that
/// actually bit.
#[test]
fn parity_tier4_ac7_horizontal_stack() {
    let (sr, layout) = build(
        "advanced-circuit",
        7.0,
        "assembling-machine-2",
        &["iron-plate", "copper-plate", "coal", "water", "crude-oil"],
        LayoutOptions {
            max_belt_tier: Some("transport-belt".to_string()),
            row_layout: spaghettio_core::bus::layout::RowLayout::HorizontalStack,
            ..LayoutOptions::default()
        },
    );
    assert_green_and_ir_parity("tier4-ac7-hs", &layout, &sr);
}

#[test]
fn parity_tier4_ac5_from_ore_default() {
    let (sr, layout) = build(
        "advanced-circuit",
        5.0,
        "assembling-machine-2",
        &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        LayoutOptions::from_belt_tier(Some("transport-belt")),
    );
    assert_green_and_ir_parity("tier4-ac5-ore-default", &layout, &sr);
}

#[test]
fn parity_direct_insertion_forced_cable_ec() {
    let (sr, layout) = build(
        "electronic-circuit",
        3.0,
        "assembling-machine-2",
        &["iron-plate", "copper-plate"],
        LayoutOptions {
            direct_insertion: spaghettio_core::bus::di_cell::DirectInsertion::Forced,
            ..LayoutOptions::default()
        },
    );
    assert_green_and_ir_parity("di-forced-cable-ec", &layout, &sr);
}

/// The whole-pipeline diff invariance check: translating every entity is a
/// rigid motion, so the derived edge set must not change. This is the
/// property no code could state before this module existed.
#[test]
fn diff_invariance_on_real_layout() {
    let (_sr, layout) = build(
        "iron-gear-wheel",
        1.0,
        "assembling-machine-2",
        &["iron-plate"],
        LayoutOptions::from_belt_tier(None),
    );
    let mut moved = layout.clone();
    for e in &mut moved.entities {
        e.x += 13;
        e.y += 6;
    }
    let d = diff(&derive_connectivity(&layout), &derive_connectivity(&moved));
    assert!(d.is_empty(), "rigid translation changed the derived topology: {d:#?}");
}

/// K65-2 detection (a): the fold-class record failure. A transform moves
/// the geometry and forgets the `effective_rows` ledger — exactly what the
/// pre-fix compact path shipped. Geometry-only validation stayed quiet on
/// this class; the integrity pass must not.
#[test]
fn detection_stale_effective_rows_fires() {
    let (_sr, layout) = build(
        "iron-gear-wheel",
        1.0,
        "assembling-machine-2",
        &["iron-plate"],
        LayoutOptions::from_belt_tier(None),
    );
    assert!(
        !layout.effective_rows.is_empty(),
        "fixture must carry an effective_rows ledger for this test to discriminate"
    );
    // A fold-class displacement: whole segments move, not single tiles —
    // the 0.00/s fold relocated geometry by segment heights. Gross shift
    // guarantees every machine exits its stale band.
    let mut stale = layout.clone();
    for e in &mut stale.entities {
        e.y += 50;
    }
    let hits: Vec<_> = check_record_integrity(&stale)
        .into_iter()
        .filter(|i| i.category == "record-effective-rows")
        .collect();
    assert!(
        !hits.is_empty(),
        "K65-2: shifted geometry with an untouched effective_rows ledger produced no finding"
    );
}

/// K65-2 detection (b): entity-list surgery with a stale `power_wires`
/// graph — the documented historical class (`bus/compaction.rs`'s
/// undergroundify pass shipped exactly this before its recompute fix: index
/// pairs naming different entities after a splice). Reconstruct it by
/// removing one entity without recomputing wires.
#[test]
fn detection_stale_power_wires_fire() {
    let (_sr, layout) = build(
        "iron-gear-wheel",
        1.0,
        "assembling-machine-2",
        &["iron-plate"],
        LayoutOptions::from_belt_tier(None),
    );
    let wires = layout.power_wires.as_ref().expect("engine layouts store a wire graph");
    assert!(!wires.is_empty(), "fixture must have at least one wire to discriminate");

    let mut spliced = layout.clone();
    // Remove the first pole: every index at or past it now names a
    // different (or missing) entity.
    let first_pole = spliced
        .entities
        .iter()
        .position(|e| spaghettio_core::power_wires::is_pole(&e.name))
        .expect("fixture has poles");
    spliced.entities.remove(first_pole);
    let hits: Vec<_> = check_record_integrity(&spliced)
        .into_iter()
        .filter(|i| i.category == "record-power-wires")
        .collect();
    assert!(
        !hits.is_empty(),
        "K65-2: spliced entity list with untouched power_wires produced no finding"
    );
}

/// Phase 1 dispatch pin: record integrity now runs inside `validate()`
/// itself, so a stale-ledger artifact fails validation with no special
/// tooling — the property that makes every future transform's admission
/// loop guard the records automatically.
#[test]
fn dispatched_validate_catches_stale_ledger() {
    let (sr, layout) = build(
        "iron-gear-wheel",
        1.0,
        "assembling-machine-2",
        &["iron-plate"],
        LayoutOptions::from_belt_tier(None),
    );
    let mut stale = layout.clone();
    for e in &mut stale.entities {
        e.y += 50;
    }
    let issues = issues_of(&stale, &sr);
    assert!(
        issues.iter().any(|i| i.category == "record-effective-rows"),
        "validate() must now carry record-integrity findings for a stale ledger: {:#?}",
        issues.iter().map(|i| &i.category).collect::<Vec<_>>()
    );
}

/// Regression pin for the RFC-065 compaction fix: after
/// `compact_validated_geometry`, the `effective_rows` ledger must describe
/// the compacted geometry (RI-1 clean). The second half reconstructs the
/// pre-fix bug — the compacted geometry wearing the ORIGINAL ledger — and
/// requires the integrity pass to catch it, guarded on the fixture actually
/// having moved bands (otherwise this test cannot discriminate and must say
/// so rather than pass vacuously).
#[test]
fn compaction_remaps_effective_rows() {
    // Multi-recipe chain: strip shifts accumulate down the row stack, so
    // deeper rows move far enough for stale bands to cross recipe
    // boundaries — the harmful class RI-1 is calibrated to.
    let (sr, layout) = build(
        "electronic-circuit",
        2.0,
        "assembling-machine-2",
        &["iron-ore", "copper-ore"],
        LayoutOptions::from_belt_tier(None),
    );
    let compacted = compact_validated_geometry(&layout, &sr);

    // Un-filtered on purpose (review defect 3): the compacted artifact must
    // be clean across EVERY record family — the cut passes recompute
    // power_wires and this RFC's fix remaps effective_rows, so nothing may
    // fire.
    let ri = check_record_integrity(&compacted);
    assert!(
        ri.is_empty(),
        "compacted layout's records do not match its geometry: {ri:#?}"
    );

    let bands = |l: &LayoutResult| -> Vec<(i32, i32)> {
        l.effective_rows.iter().map(|r| (r.y_start, r.y_end)).collect()
    };
    if bands(&compacted) != bands(&layout) {
        // Compaction moved rows on this fixture — reconstruct the pre-fix
        // artifact (compacted geometry, original ledger) and require a hit.
        let mut prefix_bug = compacted.clone();
        prefix_bug.effective_rows = layout.effective_rows.clone();
        let hits: Vec<_> = check_record_integrity(&prefix_bug)
            .into_iter()
            .filter(|i| i.category == "record-effective-rows")
            .collect();
        assert!(
            !hits.is_empty(),
            "pre-fix reconstruction (stale ledger on compacted geometry) produced no finding"
        );
    } else {
        // PR #574 bot review: a silent skip here is the check-went-quiet
        // shape — if compaction stops moving bands on this fixture, the
        // test's discriminating power is GONE and it must say so as a
        // failure, so the fixture gets re-chosen instead of the gate
        // rotting.
        panic!(
            "compaction moved no effective_rows bands on this fixture — the pre-fix \
             reconstruction half cannot discriminate; pick a fixture where vertical \
             compaction moves rows"
        );
    }
}

/// PR #574 bot round 3 (the "single biggest unclosed risk"): now that
/// record integrity runs inside every `validate()`, the shapes most likely
/// to hide an RI-1 false positive are the exotic row kinds — the kovarex
/// self-loop and the voider recycler bank (recyclers are 2×4 and rotate).
/// Committed here so K65-1 is pinned on them, not review-session folklore.
#[test]
fn parity_kovarex_self_loop() {
    let (sr, layout) = build_with_exclusions(
        "uranium-235",
        0.1,
        "assembling-machine-3",
        &["uranium-238"],
        &["uranium-processing"],
        LayoutOptions::default(),
    );
    assert_green_and_ir_parity("kovarex-self-loop", &layout, &sr);
}

#[test]
fn parity_uranium_voider() {
    let (sr, layout) = build_with_exclusions(
        "uranium-235",
        0.05,
        "assembling-machine-3",
        &["uranium-ore"],
        &["kovarex-enrichment-process"],
        LayoutOptions {
            surplus_policy: spaghettio_core::bus::layout::SurplusPolicy::Void,
            ..LayoutOptions::default()
        },
    );
    assert_green_and_ir_parity("uranium-voider", &layout, &sr);
}

/// Second in-corpus parity fixture from the RFC's K65-1 corpus definition
/// (EC@20-from-ore was one of the shapes that falsified the removed RI-2).
#[test]
fn parity_ec20_from_ore_default() {
    let (sr, layout) = build(
        "electronic-circuit",
        20.0,
        "assembling-machine-2",
        &["iron-ore", "copper-ore"],
        LayoutOptions::from_belt_tier(None),
    );
    assert_green_and_ir_parity("ec20-ore-default", &layout, &sr);
}
