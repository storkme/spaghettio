//! RFC-064 Phase 1: `LayoutOptions::fold_layout` — the fold-and-square
//! post-layout knob (`search_snake_fold`/`fold_snake`, RFC-057's
//! mechanism).
//!
//! The Phase 1 corpus-applicability spike (2026-08-01; results in session
//! artifacts, methodology reproduced from `probe_fold_corpus` at
//! `cell_composition.rs:4206`) measured admissibility — a fold is found,
//! validates no worse than the compacted baseline, and does not increase
//! input-rate-delivery warnings — at 21.4% literal / 14.3% AR-improving of
//! a 14-fixture corpus, both below the RFC's pre-registered 25% bar for
//! auto-selecting folding as a scored decomposition candidate. Per the
//! RFC's own pre-registered decision rule, folding therefore ships as this
//! plain user knob rather than a scored candidate — see
//! `docs/rfc-064-spaghetti-objective.md`'s decision log for the full
//! adjudication.
//!
//! Structural asserts only, per this PR's brief — no exact geometry/hash
//! pins against committed values (fold-search output is host- and
//! budget-relative, same discipline as every SAT/junction-solver test in
//! this suite).

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy};
use spaghettio_core::models::{LayoutResult, SolverResult};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity};

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Mirrors `e2e.rs`'s `run_e2e_inner` construction exactly (same field
/// values) so fixtures shared with that suite reproduce the same
/// geometry — in particular `inserter_capacity: 0`, which diverges from
/// `LayoutOptions::default()`'s own `DEFAULT_INSERTER_CAPACITY`.
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

fn solve(
    item: &str,
    rate: f64,
    inputs: &[&str],
    machine: &str,
) -> SolverResult {
    solver::solve_with_exclusions(item, rate, &set(inputs), machine, &FxHashSet::default())
        .unwrap_or_else(|e| panic!("solve {item}@{rate}/s ({machine}): {e}"))
}

fn json_of(l: &LayoutResult) -> serde_json::Value {
    serde_json::to_value(l).expect("LayoutResult must serialize")
}

/// Spec item 1: with `fold_layout` false — however a caller arrives at
/// that value — `build_bus_layout` must be byte-identical to today's
/// pipeline. Checked across three fixtures (tier1 gear, tier2 EC-from-ore,
/// the stress-ac-partitioned POOLED fixture the admissible-fold test below
/// also uses) and three constructions of the same `false` value: never
/// mentioning the field (relying on a convenience builder's own
/// `..Default::default()`), setting it explicitly `false`, and building
/// from a bare `LayoutOptions::default()` patched only with the fixture's
/// own belt tier / inserter capacity.
#[test]
fn fold_layout_default_off_is_byte_identical() {
    struct Fixture {
        name: &'static str,
        item: &'static str,
        rate: f64,
        machine: &'static str,
        belt: Option<&'static str>,
        inputs: &'static [&'static str],
    }
    let fixtures = [
        Fixture {
            name: "tier1-gear-from-ore",
            item: "iron-gear-wheel",
            rate: 10.0,
            machine: "assembling-machine-2",
            belt: None,
            inputs: &["iron-ore"],
        },
        Fixture {
            name: "tier2-ec-from-ore",
            item: "electronic-circuit",
            rate: 10.0,
            machine: "assembling-machine-1",
            belt: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
        },
        Fixture {
            name: "stress-ac-partitioned-5s-pooled",
            item: "advanced-circuit",
            rate: 5.0,
            machine: "assembling-machine-2",
            belt: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
        },
    ];

    for fx in fixtures {
        let sr = solve(fx.item, fx.rate, fx.inputs, fx.machine);

        // (a) field never mentioned — relies on `base_opts`'s own
        // `..Default::default()` spread to supply `false`.
        let via_base = base_opts(fx.belt, LayoutStrategy::Pooled);
        // (b) field explicitly `false`.
        let via_explicit_false = LayoutOptions {
            fold_layout: false,
            ..base_opts(fx.belt, LayoutStrategy::Pooled)
        };
        // (c) bare struct default, patched only with this fixture's belt
        // tier and `inserter_capacity: 0` (matching `base_opts`'s
        // deliberate divergence from the field default) — a construction
        // path independent of the `base_opts` helper entirely.
        let via_struct_default = LayoutOptions {
            max_belt_tier: fx.belt.map(|s| s.to_string()),
            inserter_capacity: 0,
            ..LayoutOptions::default()
        };

        let r_base = build_bus_layout(&sr, via_base)
            .unwrap_or_else(|e| panic!("{}: base build failed: {e}", fx.name));
        let r_false = build_bus_layout(&sr, via_explicit_false)
            .unwrap_or_else(|e| panic!("{}: explicit-false build failed: {e}", fx.name));
        let r_struct_default = build_bus_layout(&sr, via_struct_default)
            .unwrap_or_else(|e| panic!("{}: struct-default build failed: {e}", fx.name));

        assert_eq!(
            json_of(&r_base),
            json_of(&r_false),
            "{}: absent vs explicit-false fold_layout must be byte-identical",
            fx.name,
        );
        assert_eq!(
            json_of(&r_base),
            json_of(&r_struct_default),
            "{}: absent vs struct-default fold_layout must be byte-identical",
            fx.name,
        );
    }
}

/// Spec item 5b: knob-on for the stress-ac-partitioned-5s-from-plates
/// POOLED fixture — the spike's own admissible-and-AR-improving case
/// (134x54, AR 2.48 -> 79x66, AR 1.22, +6.7% entities). Assert a fold
/// actually fired, aspect ratio improved, entity growth stayed under the
/// spec's +20% bound, and the folded layout validates with zero errors.
#[test]
fn fold_layout_finds_admissible_fold_on_stress_ac_partitioned() {
    let sr = solve(
        "advanced-circuit",
        5.0,
        &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
        "assembling-machine-2",
    );

    // The baseline folding is measured against: compacted, unfolded.
    let native = build_bus_layout(
        &sr,
        LayoutOptions {
            compact_layout: true,
            ..base_opts(None, LayoutStrategy::Pooled)
        },
    )
    .expect("compacted baseline build failed");
    let folded = build_bus_layout(
        &sr,
        LayoutOptions {
            fold_layout: true,
            ..base_opts(None, LayoutStrategy::Pooled)
        },
    )
    .expect("fold_layout build failed");

    let aspect = |l: &LayoutResult| l.width.max(l.height) as f64 / l.width.min(l.height) as f64;
    let native_ar = aspect(&native);
    let folded_ar = aspect(&folded);

    assert_ne!(
        (folded.width, folded.height),
        (native.width, native.height),
        "a fold must actually have fired (geometry must differ from the \
         compacted baseline) — got {}x{} for both",
        native.width,
        native.height,
    );
    assert!(
        folded_ar < native_ar,
        "fold must improve aspect ratio: native {native_ar:.3} ({}x{}) \
         folded {folded_ar:.3} ({}x{})",
        native.width,
        native.height,
        folded.width,
        folded.height,
    );

    let growth =
        (folded.entities.len() as f64 - native.entities.len() as f64) / native.entities.len() as f64;
    assert!(
        growth < 0.20,
        "entity growth {:.1}% exceeds the spec's +20% bound (native {} -> folded {})",
        growth * 100.0,
        native.entities.len(),
        folded.entities.len(),
    );

    let issues = match validate::validate(&folded, Some(&sr), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    assert!(
        issues.iter().all(|i| i.severity != Severity::Error),
        "folded layout must validate with zero errors: {issues:?}",
    );
}

/// Spec item 5c: knob-on for a fixture where no fold is admissible
/// (tier2-ec-from-ore — the spike measured 44/44 geometrically-legal fold
/// candidates rejected by validation, zero geometric refusals). Assert a
/// clean fallback: `fold_layout: true` must produce exactly the same
/// layout `compact_layout: true` alone would, never an error and never
/// the raw uncompacted geometry.
#[test]
fn fold_layout_falls_back_cleanly_when_no_fold_is_admissible() {
    let sr = solve(
        "electronic-circuit",
        10.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-1",
    );
    let belt = Some("transport-belt");

    let knob_off = build_bus_layout(&sr, base_opts(belt, LayoutStrategy::Pooled))
        .expect("knob-off build failed");
    let knob_on = build_bus_layout(
        &sr,
        LayoutOptions {
            fold_layout: true,
            ..base_opts(belt, LayoutStrategy::Pooled)
        },
    )
    .expect("fold_layout build failed");
    let compacted_only = build_bus_layout(
        &sr,
        LayoutOptions {
            compact_layout: true,
            ..base_opts(belt, LayoutStrategy::Pooled)
        },
    )
    .expect("compacted-only build failed");

    assert_eq!(
        json_of(&knob_on),
        json_of(&compacted_only),
        "fold_layout must fall back to exactly the compacted-but-unfolded \
         layout when no fold is admissible",
    );
    // Compaction measurably changes this fixture (spike: raw 1110 entities
    // -> compacted 953), so the fallback must differ from the raw
    // knob-off layout — confirming the knob still compacts even when no
    // fold is found, rather than silently no-op'ing back to raw geometry.
    assert_ne!(
        json_of(&knob_on),
        json_of(&knob_off),
        "fold_layout's fallback must still be the COMPACTED geometry, not \
         a silent no-op back to the raw uncompacted layout",
    );
}

/// Spec item 5d: knob-on above the latency-guard entity threshold. Uses
/// electronic-circuit@80/s (AM2, red belt, from ore) — not part of the
/// spike corpus, chosen because it comfortably exceeds the guard's 6,000-
/// entity threshold (~6,558 compacted entities measured locally) while
/// staying cheap enough to build in a normal test run, unlike the spike's
/// own mega-chain fixtures (14k-19k entities, requiring the separate
/// chain-composition harness). Assert the search was skipped (the layout
/// warnings carry the "fold search skipped: layout too large" message)
/// rather than the user being silently handed a compacted-but-unfolded
/// layout with no explanation.
#[test]
#[ntest::timeout(180000)]
fn fold_layout_skips_search_above_entity_threshold_and_warns() {
    let sr = solve(
        "electronic-circuit",
        80.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-2",
    );

    let result = build_bus_layout(
        &sr,
        LayoutOptions {
            fold_layout: true,
            ..base_opts(Some("fast-transport-belt"), LayoutStrategy::Pooled)
        },
    )
    .expect("fold_layout build failed");

    assert!(
        result.entities.len() > 6000,
        "fixture must exceed the fold-search entity threshold \
         (FOLD_SEARCH_ENTITY_THRESHOLD in crates/core/src/bus/layout.rs) \
         for this test to mean anything: got {} entities",
        result.entities.len(),
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("fold search skipped") && w.contains("too large")),
        "expected a 'fold search skipped: layout too large' warning, got: {:?}",
        result.warnings,
    );
}
