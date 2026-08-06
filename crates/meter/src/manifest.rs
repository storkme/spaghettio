//! The verification manifest — the meter reads the same file
//! `spaghettio-sim` does.
//!
//! Deliberately a **separate, minimal** deserializer rather than a
//! dependency on `spaghettio_sim_harness::manifest`: the meter needs only
//! the boundary records, the plan and the declared research/stacking
//! levels, and coupling the two tools would make a harness refactor able
//! to change what the meter measures. Fields the meter does not model are
//! simply ignored by serde, so a manifest gaining a field never breaks it.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BoundaryRecord {
    pub item: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub direction: u8,
    #[serde(default)]
    pub is_fluid: bool,
    #[serde(default)]
    pub entity: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ItemRate {
    pub item: String,
    pub rate: f64,
    #[serde(default)]
    pub is_fluid: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Manifest {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub targets: Vec<ItemRate>,
    #[serde(default)]
    pub external_inputs: Vec<ItemRate>,
    /// Per-item planned rate — what the layout is *supposed* to do.
    #[serde(default)]
    pub planned_rates: BTreeMap<String, f64>,
    #[serde(default)]
    pub boundary_inputs: Vec<BoundaryRecord>,
    #[serde(default)]
    pub boundary_outputs: Vec<BoundaryRecord>,
    /// Declared inserter-capacity research level (0–7). A user-facing
    /// engine axis; the meter takes it as an input and never infers it.
    #[serde(default)]
    pub inserter_capacity: u8,
    /// Declared belt stack size (1–4).
    #[serde(default = "one")]
    pub stacking: u8,
    /// Declared research-sourced productivity, per recipe (e.g.
    /// `{"processing-unit": 0.10}`).
    ///
    /// Same contract as the two axes above: an input the meter takes and
    /// never infers. Empty = none, which is what every manifest written
    /// before this field existed deserializes to, so their measurements are
    /// unchanged.
    ///
    /// This exists because the sim runs `research_all_technologies()` and so
    /// carried productivity that neither the solver nor this meter modelled —
    /// the meter then under-produced by exactly that factor and the gap was
    /// chased for three sessions as a belt/distribution defect. Declaring it
    /// makes the two worlds match by construction rather than by luck. See
    /// `docs/meter-divergence.md`.
    #[serde(default)]
    pub research_productivity: BTreeMap<String, f64>,
    #[serde(default)]
    pub entities: usize,
}

fn one() -> u8 {
    1
}

impl Manifest {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("manifest parse failed: {e}"))
    }

    pub fn from_path(p: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let text = std::fs::read_to_string(p.as_ref())
            .map_err(|e| format!("cannot read manifest {:?}: {e}", p.as_ref()))?;
        Self::from_json(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_shaped_manifest() {
        let json = r#"{
            "label": "chain-ec15-d2",
            "targets": [{"item": "electronic-circuit", "rate": 15.0, "is_fluid": false}],
            "external_inputs": [{"item": "iron-plate", "rate": 15.0, "is_fluid": false}],
            "planned_rates": {"copper-cable": 45.0, "electronic-circuit": 15.0},
            "boundary_inputs": [
                {"item": "copper-plate", "x": 1, "y": 0, "direction": 8,
                 "is_fluid": false, "entity": "express-transport-belt"}
            ],
            "boundary_outputs": [
                {"item": "electronic-circuit", "x": 64, "y": 20, "direction": 8,
                 "is_fluid": false, "entity": "transport-belt"}
            ],
            "inserter_capacity": 2,
            "stacking": 1,
            "entities": 292,
            "some_future_field": [1,2,3]
        }"#;
        let m = Manifest::from_json(json).expect("parse");
        assert_eq!(m.label, "chain-ec15-d2");
        assert_eq!(m.inserter_capacity, 2);
        assert_eq!(m.stacking, 1);
        assert_eq!(m.planned_rates.get("electronic-circuit"), Some(&15.0));
        assert_eq!(m.boundary_inputs.len(), 1);
        assert_eq!(m.boundary_outputs[0].x, 64);
    }

    /// Unknown fields must not break ingestion — the harness owns this
    /// format and will add to it.
    #[test]
    fn tolerates_unknown_fields_and_missing_optionals() {
        let m = Manifest::from_json(r#"{"label":"x","brand_new":true}"#).expect("parse");
        assert_eq!(m.label, "x");
        assert_eq!(m.stacking, 1, "stacking defaults to 1, not 0");
        assert!(m.boundary_inputs.is_empty());
    }

    #[test]
    fn malformed_json_is_an_error() {
        assert!(Manifest::from_json("{not json").is_err());
    }
}
