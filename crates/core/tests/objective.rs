//! RFC-064 P1 calibration: [`spaghettio_core::objective`] against a real,
//! cheap fixture rather than hand-built geometry (see the unit tests inside
//! `objective.rs` itself for formula-level pins, including the RFC's own
//! `AR_score ≈ 0.9945` worked example).
//!
//! This file used to also carry `fold_ranks_above_native_on_stress_ac_partitioned`
//! — a calibration anchor proving a validated, AR-improving `fold_layout: true`
//! candidate ranked above its native incumbent under `Composite(L)`. Deleted
//! 2026-08-14 (#632 A2, owner call) along with `fold_layout`/`compact_layout`
//! and the `bus::compaction` module they called: the underlying relocation
//! research never shipped past three falsified attempts (RFC-057/058/064-P3
//! decision logs). `measurement_is_total_or_it_refuses` below is unaffected —
//! it pins `objective::measure`'s §(b) totality invariant on an ordinary
//! (uncompacted) layout, which never depended on the fold/compact transforms.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy};
use spaghettio_core::objective::{measure, FLUID_WEIGHT};
use spaghettio_core::solver;

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Shared fixture options for this file's one remaining test.
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

/// RFC-064 §(b) makes a measurement TOTAL: "Any other unreachable terminal
/// makes the metric unmeasurable and the candidate inadmissible — never
/// silently fall back to Manhattan for a broken routed edge."
///
/// This pins that invariant at the API boundary. It replaces a test that
/// counted *partial* attribution, which was meaningful only while this module
/// carried its own non-conforming measurement: that one averaged over whichever
/// producer ports it could reach and reported the mean as if it were the whole
/// edge. On this very fixture it reported 1 unattributed and 2 partially
/// attributed edges out of 5. The conforming implementation measures all five.
///
/// There is deliberately no "partial" case to assert any more — that is the
/// point of the change, and a test asserting one would re-introduce the state
/// the spec forbids.
#[test]
fn measurement_is_total_or_it_refuses() {
    let sr = solver::solve_with_exclusions(
        "advanced-circuit",
        5.0,
        &set(&["iron-plate", "copper-plate", "coal", "crude-oil", "water"]),
        "assembling-machine-2",
        &FxHashSet::default(),
    )
    .expect("solve should succeed");
    let layout = build_bus_layout(&sr, base_opts(None, LayoutStrategy::Pooled))
        .expect("layout should build");
    let m = measure(&layout, &sr).expect("measure should succeed");

    assert!(!m.edges.is_empty(), "fixture must have production edges to measure");
    for e in &m.edges {
        assert!(
            e.path_length.is_finite() && e.path_length >= 0.0,
            "{} -> {}: every edge of a successful measure carries a real length",
            e.item,
            e.consumer_recipe
        );
        assert!(
            e.consumer_terminals > 0,
            "{} -> {}: §(b) means over consumer terminals, so there must be at least one",
            e.item,
            e.consumer_recipe
        );
    }

    // The reported total is the sum of its parts, and the medium split adds up.
    let summed: f64 = m
        .edges
        .iter()
        .map(|e| e.rate * if e.is_fluid { FLUID_WEIGHT } else { 1.0 } * e.path_length)
        .sum();
    assert!(
        (summed - m.transit).abs() < 1e-6,
        "transit {} disagrees with the sum over its own edges {summed}",
        m.transit
    );
    assert!(
        (m.solid_transit + m.fluid_transit - m.transit).abs() < 1e-6,
        "solid+fluid split must reconstruct the total"
    );
}
