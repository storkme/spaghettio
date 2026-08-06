//! RFC-064 P1 calibration: [`spaghettio_core::objective`] against a real,
//! cheap fixture rather than hand-built geometry (see the unit tests inside
//! `objective.rs` itself for formula-level pins, including the RFC's own
//! `AR_score ≈ 0.9945` worked example).
//!
//! The RFC's Phase 0/1 calibration anchor is `chain-mil5ore`'s 3-fold
//! (`AR_score ≈ 0.9945`), but reproducing that fixture requires the
//! `SimFixture`/`MachinePalette` harness private to
//! `crates/core/tests/cell_composition.rs` — not reusable from a separate
//! integration test binary without duplicating that harness, and the fold
//! search itself is the multi-second-plus case this PR's brief flags as a
//! `#[ignore]` candidate. Per the brief's own fallback ("otherwise add a
//! faster always-on calibration case"), this file uses
//! `fold_layout_knob.rs`'s own `stress-ac-partitioned-5s-pooled` fixture —
//! already exercised un-ignored elsewhere in this suite, so it is known
//! cheap — as the real, on-by-default calibration case: RFC-064 Phase 1's
//! own admissible-fold finding (134x54 AR 2.48 -> 79x66 AR 1.22, +6.7%
//! entities) confirms this specific fold is a genuine, validated
//! improvement, which is exactly the "candidate ranks above native"
//! property the composite must reproduce.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy};
use spaghettio_core::objective::{measure, rank_admissible, score_vs_native, FLUID_WEIGHT};
use spaghettio_core::solver;

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Mirrors `fold_layout_knob.rs`'s `base_opts` exactly, so this reuses the
/// same fixture geometry that file's own admissible-fold test already
/// pins.
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

/// Calibration anchor (real fixture, cheap): a validated, AR-improving
/// fold must rank above its native incumbent under `Composite(L)` — the
/// property RFC-064 Phase 1's metric sanity check (gate step 3) requires
/// for `chain-mil5ore` and this test reproduces for a second, ordinary
/// row-bus fixture found admissible in the same phase.
#[test]
fn fold_ranks_above_native_on_stress_ac_partitioned() {
    let sr = solver::solve_with_exclusions(
        "advanced-circuit",
        5.0,
        &set(&["iron-plate", "copper-plate", "coal", "crude-oil", "water"]),
        "assembling-machine-2",
        &FxHashSet::default(),
    )
    .expect("solve should succeed");

    let native_layout = build_bus_layout(
        &sr,
        LayoutOptions { compact_layout: true, ..base_opts(None, LayoutStrategy::Pooled) },
    )
    .expect("native (compacted) build failed");
    let folded_layout = build_bus_layout(
        &sr,
        LayoutOptions { fold_layout: true, ..base_opts(None, LayoutStrategy::Pooled) },
    )
    .expect("folded build failed");

    // Sanity: this must actually be the admissible fold RFC-064 Phase 1
    // found (AR improves), or the rest of this test proves nothing.
    let aspect = |w: i32, h: i32| w.max(h) as f64 / w.min(h) as f64;
    assert!(
        aspect(folded_layout.width, folded_layout.height) < aspect(native_layout.width, native_layout.height),
        "fixture assumption violated: fold did not improve aspect ratio \
         (native {}x{}, folded {}x{}) — Phase 1's admissible-fold finding \
         may have drifted; re-pick a calibration fixture",
        native_layout.width,
        native_layout.height,
        folded_layout.width,
        folded_layout.height,
    );

    let native = measure(&native_layout, &sr).expect("measure(native) should succeed");
    let folded = measure(&folded_layout, &sr).expect("measure(folded) should succeed");

    let native_scores = score_vs_native(&native, &native);
    let folded_scores = score_vs_native(&folded, &native);

    assert_eq!(native_scores.composite, 0.0, "native scores 0 by construction (RFC-064 §(a)/(b))");
    assert!(folded_scores.ar_score > 0.0, "AR must improve: got {}", folded_scores.ar_score);
    assert!(
        folded_scores.composite > native_scores.composite,
        "fold must rank above native under Composite(L): native {:.4}, folded {:.4}",
        native_scores.composite,
        folded_scores.composite,
    );

    let ranked = rank_admissible(&[
        ("native".to_string(), native_scores, native.entity_count),
        ("folded".to_string(), folded_scores, folded.entity_count),
    ]);
    assert_eq!(ranked[0], "folded", "rank_admissible must place the fold first: {ranked:?}");
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
    let layout = build_bus_layout(
        &sr,
        LayoutOptions { compact_layout: true, ..base_opts(None, LayoutStrategy::Pooled) },
    )
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
