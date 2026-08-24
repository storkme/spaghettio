//! Canonical current-generation fixtures for meter-vs-Factorio calibration.
//!
//! The e2e suite and the Factorio calibration bank must exercise the same
//! factories.  Keeping this list in a test-only helper previously made the
//! latter a manually-maintained, incomplete subset.  The exporter in
//! `examples/calibration_matrix_export.rs` consumes this module to create a
//! fresh, immutable bank for `spaghettio-sim` and the meter sweep.

use rustc_hash::FxHashSet;

use crate::bus::layout::{self, LayoutOptions, LayoutStrategy, SurplusPolicy};
use crate::models::{LayoutResult, SolverResult};
use crate::solver;
use crate::validate::{self, ValidationIssue};

/// The one deliberate layout variation a calibration fixture may request.
#[derive(Clone, Copy, Debug)]
pub enum FixtureVariant {
    /// Production defaults.
    Plain,
    /// A named production strategy rather than the default candidate choice.
    Strategy(LayoutStrategy),
    /// Exclude recipes before solving.
    Excluded,
    /// Exclude recipes and consume recyclable solid surplus.
    ExcludedVoid,
}

/// A representative, current-production factory configuration.
///
/// This is not a claim to enumerate every possible generator input.  It is
/// the maintained regression corpus: every row builds in e2e, and every row
/// is eligible for an independently measured Factorio result.
#[derive(Clone, Copy, Debug)]
pub struct CalibrationFixture {
    pub name: &'static str,
    pub item: &'static str,
    pub rate: f64,
    pub machine: &'static str,
    pub belt_tier: Option<&'static str>,
    pub inputs: &'static [&'static str],
    pub excluded: &'static [&'static str],
    pub variant: FixtureVariant,
}

/// Machine-readable variant tag.  A strategy row carries its discriminant —
/// the pooled/partitioned A/B pairs in the corpus are otherwise identical
/// rows distinguishable only by label.
pub fn variant_name(variant: FixtureVariant) -> String {
    match variant {
        FixtureVariant::Plain => "plain".into(),
        FixtureVariant::Strategy(s) => format!("strategy:{s:?}"),
        FixtureVariant::Excluded => "excluded".into(),
        FixtureVariant::ExcludedVoid => "excluded-void".into(),
    }
}

/// The ordered corpus-definition fields whose SHA-256 is a schema-2
/// `matrix.json`'s `corpus_sha256`.  Each fixture contributes these fields
/// in order, each on its own line: name, item, rate, machine, belt tier,
/// inputs, exclusions, and variant tag; inputs and exclusions are
/// comma-joined and a missing belt tier is empty.  This lives in the
/// library so the exporter and the CI fingerprint probe serialize the SAME
/// fields; the hashing stays in the callers because `sha2` is deliberately
/// a dev-dependency (the library never hashes, and WASM would carry it).
pub fn corpus_fingerprint_fields(corpus: &[CalibrationFixture]) -> String {
    let mut fields = Vec::with_capacity(corpus.len() * 8);
    for fixture in corpus {
        fields.extend([
            fixture.name.to_owned(),
            fixture.item.to_owned(),
            fixture.rate.to_string(),
            fixture.machine.to_owned(),
            fixture.belt_tier.unwrap_or_default().to_owned(),
            fixture.inputs.join(","),
            fixture.excluded.join(","),
            variant_name(fixture.variant),
        ]);
    }
    fields.join("\n")
}

/// The shared current-generation corpus.  Add a fixture here when adding a
/// materially new generator shape; the e2e differential and calibration
/// exporter then gain it together.
pub fn fixtures() -> Vec<CalibrationFixture> {
    use FixtureVariant::*;

    vec![
        CalibrationFixture {
            name: "tier1_iron_gear_wheel",
            item: "iron-gear-wheel",
            rate: 10.0,
            machine: "assembling-machine-1",
            belt_tier: None,
            inputs: &["iron-plate"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier1_iron_gear_wheel_from_ore",
            item: "iron-gear-wheel",
            rate: 10.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier1_iron_gear_wheel_20s",
            item: "iron-gear-wheel",
            rate: 20.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier2_electronic_circuit",
            item: "electronic-circuit",
            rate: 10.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier2_electronic_circuit_from_ore",
            item: "electronic-circuit",
            rate: 10.0,
            machine: "assembling-machine-1",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier2_electronic_circuit_20s_from_ore",
            item: "electronic-circuit",
            rate: 20.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier3_plastic_bar",
            item: "plastic-bar",
            rate: 10.0,
            machine: "chemical-plant",
            belt_tier: None,
            inputs: &["petroleum-gas", "coal"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier3_plastic_bar_from_crude",
            item: "plastic-bar",
            rate: 10.0,
            machine: "chemical-plant",
            belt_tier: None,
            inputs: &["crude-oil", "coal"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier3_sulfuric_acid",
            item: "sulfuric-acid",
            rate: 5.0,
            machine: "chemical-plant",
            belt_tier: None,
            inputs: &["iron-plate", "sulfur", "water"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier3_heavy_oil_cracking",
            item: "light-oil",
            rate: 5.0,
            machine: "chemical-plant",
            belt_tier: None,
            inputs: &["water", "heavy-oil"],
            excluded: &["advanced-oil-processing", "coal-liquefaction"],
            variant: Excluded,
        },
        CalibrationFixture {
            name: "tier3_advanced_oil_processing_multi_machine",
            item: "petroleum-gas",
            rate: 12.0,
            machine: "oil-refinery",
            belt_tier: None,
            inputs: &["water", "crude-oil"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier3_advanced_oil_processing_forced_multi_machine_pipe_isolation",
            item: "petroleum-gas",
            rate: 24.0,
            machine: "oil-refinery",
            belt_tier: None,
            inputs: &["water", "crude-oil"],
            excluded: &["basic-oil-processing", "coal-liquefaction"],
            variant: Excluded,
        },
        CalibrationFixture {
            name: "tier4_advanced_circuit_from_plates",
            item: "advanced-circuit",
            rate: 1.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier4_advanced_circuit_partitioned",
            item: "advanced-circuit",
            rate: 1.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::PartitionedDecomposed),
        },
        CalibrationFixture {
            name: "tier4_advanced_circuit_from_ore_am2",
            item: "advanced-circuit",
            rate: 5.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier5_processing_unit_from_ore_am3",
            item: "processing-unit",
            rate: 2.0,
            machine: "assembling-machine-3",
            belt_tier: Some("fast-transport-belt"),
            inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier_kovarex_self_loop",
            item: "uranium-235",
            rate: 0.1,
            machine: "assembling-machine-3",
            belt_tier: None,
            inputs: &["uranium-238"],
            excluded: &["uranium-processing"],
            variant: Excluded,
        },
        CalibrationFixture {
            name: "tier_uranium_processing_surplus_export",
            item: "uranium-235",
            rate: 0.05,
            machine: "assembling-machine-3",
            belt_tier: None,
            inputs: &["uranium-ore"],
            excluded: &["kovarex-enrichment-process"],
            variant: Excluded,
        },
        CalibrationFixture {
            name: "tier_uranium_processing_voider",
            item: "uranium-235",
            rate: 0.05,
            machine: "assembling-machine-3",
            belt_tier: None,
            inputs: &["uranium-ore"],
            excluded: &["kovarex-enrichment-process"],
            variant: ExcludedVoid,
        },
        CalibrationFixture {
            name: "tier_pentapod_egg_self_loop",
            item: "pentapod-egg",
            rate: 0.2,
            machine: "assembling-machine-3",
            belt_tier: None,
            inputs: &["nutrients", "water"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier_fish_breeding_self_loop",
            item: "raw-fish",
            rate: 0.15,
            machine: "assembling-machine-3",
            belt_tier: Some("fast-transport-belt"),
            inputs: &["nutrients", "water"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "tier_bacteria_self_loop_regression",
            item: "iron-bacteria",
            rate: 1.0,
            machine: "assembling-machine-3",
            belt_tier: None,
            inputs: &["bioflux"],
            excluded: &["iron-bacteria"],
            variant: Excluded,
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_30s_from_ore",
            item: "electronic-circuit",
            rate: 30.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "stress_advanced_circuit_45s_from_plates",
            item: "advanced-circuit",
            rate: 45.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "plastic-bar"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "stress_advanced_circuit_partitioned_5s_from_plates_pooled",
            item: "advanced-circuit",
            rate: 5.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::Pooled),
        },
        CalibrationFixture {
            name: "stress_advanced_circuit_partitioned_5s_from_plates_partitioned",
            item: "advanced-circuit",
            rate: 5.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::PartitionedDecomposed),
        },
        CalibrationFixture {
            name: "stress_advanced_circuit_partitioned_4s_from_plates_pooled",
            item: "advanced-circuit",
            rate: 4.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::Pooled),
        },
        CalibrationFixture {
            name: "stress_advanced_circuit_partitioned_4s_from_plates_partitioned",
            item: "advanced-circuit",
            rate: 4.0,
            machine: "assembling-machine-2",
            belt_tier: None,
            inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::PartitionedDecomposed),
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_30s_decomposed_pooled",
            item: "electronic-circuit",
            rate: 30.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::Pooled),
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_30s_decomposed_partitioned",
            item: "electronic-circuit",
            rate: 30.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Strategy(LayoutStrategy::PartitionedDecomposed),
        },
        // `stress_processing_unit_20s_from_plates` (processing-unit at 20/s,
        // AM3) is deliberately NOT a row: its balancer-shape SAT search ran
        // >20 min without finishing (belt-detour survey driver run,
        // 2026-08-01), far outside any corpus budget. Its e2e test is
        // `#[ignore]` for the same reason. Listed here so the omission reads
        // as a decision rather than an oversight; add it back only with a
        // measured build time.
        CalibrationFixture {
            name: "stress_electronic_circuit_60s_red_from_ore",
            item: "electronic-circuit",
            rate: 60.0,
            machine: "assembling-machine-2",
            belt_tier: Some("fast-transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_22s_from_ore",
            item: "electronic-circuit",
            rate: 22.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_23s_from_ore",
            item: "electronic-circuit",
            rate: 23.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_35s_from_ore",
            item: "electronic-circuit",
            rate: 35.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
        CalibrationFixture {
            name: "stress_electronic_circuit_40s_from_ore",
            item: "electronic-circuit",
            rate: 40.0,
            machine: "assembling-machine-2",
            belt_tier: Some("transport-belt"),
            inputs: &["iron-ore", "copper-ore"],
            excluded: &[],
            variant: Plain,
        },
    ]
}

/// Build the exact production-default option set with only this fixture's
/// declared axes overridden.  Group defaults avoid the historic e2e fossils
/// where a field type's own `Default` differed from the engine default.
pub fn layout_options(fixture: &CalibrationFixture) -> LayoutOptions {
    let mut constraints = layout::UserConstraints {
        max_belt_tier: fixture.belt_tier.map(str::to_owned),
        ..Default::default()
    };
    let mut axes = layout::SearchAxes::default();
    match fixture.variant {
        FixtureVariant::Plain | FixtureVariant::Excluded => {}
        FixtureVariant::Strategy(strategy) => axes.strategy = strategy,
        FixtureVariant::ExcludedVoid => constraints.surplus_policy = SurplusPolicy::Void,
    }
    LayoutOptions::from_groups(constraints, axes, layout::EngineTuning::default())
}

/// A generated fixture ready to export and measure.
pub struct BuiltFixture {
    pub solver_result: SolverResult,
    pub layout: LayoutResult,
    pub issues: Vec<ValidationIssue>,
}

/// Solve, lay out, and validate one fixture using the same axes as e2e.
pub fn build(fixture: &CalibrationFixture) -> Result<BuiltFixture, String> {
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| (*s).to_string()).collect();
    let excluded: FxHashSet<String> = fixture.excluded.iter().map(|s| (*s).to_string()).collect();
    let solver_result = solver::solve_with_exclusions(
        fixture.item,
        fixture.rate,
        &inputs,
        fixture.machine,
        &excluded,
    )
    .map_err(|e| format!("solver: {e}"))?;
    let layout = layout::build_bus_layout(&solver_result, layout_options(fixture))
        .map_err(|e| format!("layout: {e}"))?;
    let issues = validate::validate(&layout, Some(&solver_result)).unwrap_or_else(|e| e.issues);
    Ok(BuiltFixture {
        solver_result,
        layout,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_corpus_has_unique_fixture_labels() {
        let fixtures = fixtures();
        let labels: std::collections::BTreeSet<_> = fixtures.iter().map(|f| f.name).collect();
        assert_eq!(
            labels.len(),
            fixtures.len(),
            "calibration fixture labels must be unique"
        );
        assert_eq!(
            fixtures.len(),
            35,
            "corpus changes must be deliberate and visible"
        );
    }

    #[test]
    fn plain_fixture_uses_shipped_defaults() {
        let fixture = fixtures()
            .into_iter()
            .find(|f| matches!(f.variant, FixtureVariant::Plain) && f.belt_tier.is_none())
            .expect("plain fixture");
        assert_eq!(
            layout_options(&fixture).constraints(),
            LayoutOptions::default().constraints()
        );
        assert_eq!(
            layout_options(&fixture).axes(),
            LayoutOptions::default().axes()
        );
    }
}
