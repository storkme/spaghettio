// Shared demand corpus for the Phase-0 celldb probes (census + cost).
// Verbatim snapshot of `survey_fixtures()` from `crates/core/tests/e2e.rs`
// at cd78eed7, collapsed on the solver key — entries differing only by
// layout-strategy variant are one demand observation. Included via
// `include!` by both probes so the two corpora cannot drift (plain comments:
// inner doc comments are illegal under include!).
//
// Fields: item, rate, machine, belt tier (None = UNCAPPED — the engine
// escalates to the tier the rate demands, matching e2e's belt_tier:None
// semantics; it does NOT mean yellow), inputs, excluded recipes.
// Module assumption: bare machines (matches the fixtures).

pub struct F(
    pub &'static str,
    pub f64,
    pub &'static str,
    pub Option<&'static str>,
    pub &'static [&'static str],
    pub &'static [&'static str],
);

pub fn corpus() -> Vec<F> {
    vec![
        F("iron-gear-wheel", 10.0, "assembling-machine-1", None, &["iron-plate"], &[]),
        F("iron-gear-wheel", 10.0, "assembling-machine-2", None, &["iron-ore"], &[]),
        F("iron-gear-wheel", 20.0, "assembling-machine-2", None, &["iron-plate"], &[]),
        F("electronic-circuit", 10.0, "assembling-machine-2", None, &["iron-plate", "copper-plate"], &[]),
        F("electronic-circuit", 10.0, "assembling-machine-1", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 20.0, "assembling-machine-2", None, &["iron-ore", "copper-ore"], &[]),
        F("plastic-bar", 10.0, "chemical-plant", None, &["petroleum-gas", "coal"], &[]),
        F("plastic-bar", 10.0, "chemical-plant", None, &["crude-oil", "coal"], &[]),
        F("sulfuric-acid", 5.0, "chemical-plant", None, &["iron-plate", "sulfur", "water"], &[]),
        F("light-oil", 5.0, "chemical-plant", None, &["water", "heavy-oil"], &["advanced-oil-processing", "coal-liquefaction"]),
        F("petroleum-gas", 12.0, "oil-refinery", None, &["water", "crude-oil"], &[]),
        F("petroleum-gas", 24.0, "oil-refinery", None, &["water", "crude-oil"], &["basic-oil-processing", "coal-liquefaction"]),
        F("advanced-circuit", 1.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("advanced-circuit", 5.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], &[]),
        F("processing-unit", 2.0, "assembling-machine-3", Some("fast-transport-belt"), &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], &[]),
        F("uranium-235", 0.1, "assembling-machine-3", None, &["uranium-238"], &["uranium-processing"]),
        F("uranium-235", 0.05, "assembling-machine-3", None, &["uranium-ore"], &["kovarex-enrichment-process"]),
        F("pentapod-egg", 0.2, "assembling-machine-3", None, &["nutrients", "water"], &[]),
        F("raw-fish", 0.15, "assembling-machine-3", Some("fast-transport-belt"), &["nutrients", "water"], &[]),
        F("iron-bacteria", 1.0, "assembling-machine-3", None, &["bioflux"], &["iron-bacteria"]),
        F("electronic-circuit", 30.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("advanced-circuit", 45.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "plastic-bar"], &[]),
        F("advanced-circuit", 5.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("advanced-circuit", 4.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("electronic-circuit", 60.0, "assembling-machine-2", Some("fast-transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 22.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 23.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 35.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 40.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
    ]
}
