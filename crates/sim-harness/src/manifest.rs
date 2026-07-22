//! Types for the RFC-050 verification manifest emitted by
//! `spaghettio_core::blueprint::export_with_manifest` (branch
//! `rfc050-phase0-manifest`, commit `3030855`).
//!
//! This crate does NOT depend on `spaghettio_core` (RFC-050 constraint:
//! "consumes the manifest schema only") — these types are a hand-written
//! mirror of the JSON the engine emits, kept honest by the fixture in
//! `tests/fixtures/manifest_gear10.json` (a hand-transcribed instance of
//! the real schema, not the pre-Phase-0 ad hoc `feeds`/`drain` shape that
//! circulated during the discovery spike).
//!
//! Field-by-field provenance (from reading `export_with_manifest` directly,
//! per the task brief — NOT from the RFC prose, which promises a couple of
//! fields the landed Phase 0 code doesn't actually emit yet; see the
//! `validator_errors`/`validator_warnings` note below):
//!
//! - `label`, `targets`, `external_inputs`, `planned_rates`,
//!   `boundary_inputs`, `boundary_outputs`, `surplus_exits`, `bbox_min`,
//!   `dims`, `entities`, `stacking`, `inserter_capacity` are all emitted
//!   verbatim by the `serde_json::json!` call in `export_with_manifest`.
//! - The RFC's Design section says the manifest carries "validator
//!   error/warning counts at export time" — the actual Phase 0 commit's
//!   `export_with_manifest` does NOT include such fields. Resolved as: treat
//!   them as optional/absent-tolerant (`validator_errors`/
//!   `validator_warnings` are not modeled at all here; nothing in Phase 0/1
//!   reads them). If a later Phase 0 revision adds them before merge, this
//!   module only needs a new optional field, not a rewrite.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Factorio's 4-way direction constants (the engine only ever emits one of
/// these four; `EntityDirection` on the core side is `#[repr(u8)]` and
/// serializes to the same numbers via `serde`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn from_u8(v: u8) -> Option<Direction> {
        match v {
            0 => Some(Direction::North),
            4 => Some(Direction::East),
            8 => Some(Direction::South),
            12 => Some(Direction::West),
            _ => None,
        }
    }

    /// Unit vector this direction moves an item: `(dx, dy)` in layout/world
    /// tile coordinates, where +y is south (down) — matches Factorio's and
    /// the layout engine's shared convention.
    pub fn vector(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }

}

/// Rotate a unit vector 90 degrees to get a lateral (perpendicular) axis.
/// Verified against the calibrated south-facing prototype
/// (`gen_harness_scenario.py`'s drain rig): for south `(0,1)` this yields
/// `(1,0)` (east), and the prototype's `side=-1` (west offset, direction
/// east pickup) / `side=1` (east offset, direction west pickup) fall out
/// exactly from `offset = lateral*side`, `pickup_dir = -offset`.
pub fn rot90((dx, dy): (i32, i32)) -> (i32, i32) {
    (dy, -dx)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemRate {
    pub item: String,
    pub rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalInput {
    pub item: String,
    pub rate: f64,
    #[serde(default)]
    pub is_fluid: bool,
}

/// One boundary point: `direction` is the DIRECTION FIELD AS EXPORTED —
/// deliberately not translated by the artifact-boundary inserter-direction
/// flip (that flip only applies to `entities[].direction` in the blueprint
/// JSON itself; the manifest's `BoundaryRecord::direction` is a plain
/// `EntityDirection as u8` cast in `export_with_manifest`, i.e. the
/// engine's own drop-side/flow convention, matching Factorio's own belt
/// `direction` semantics 1:1 — belts (unlike inserters) don't have a
/// pickup/drop-side ambiguity).
#[derive(Debug, Clone, Deserialize)]
pub struct BoundaryRecord {
    pub item: String,
    pub x: i32,
    pub y: i32,
    pub direction: u8,
    #[serde(default)]
    pub is_fluid: bool,
    pub entity: String,
}

impl BoundaryRecord {
    pub fn direction(&self) -> Direction {
        Direction::from_u8(self.direction).unwrap_or_else(|| {
            panic!(
                "boundary record for {} at ({},{}) has non-cardinal direction {}",
                self.item, self.x, self.y, self.direction
            )
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub label: String,
    #[serde(default)]
    pub targets: Vec<ItemRate>,
    #[serde(default)]
    pub external_inputs: Vec<ExternalInput>,
    #[serde(default)]
    pub planned_rates: BTreeMap<String, f64>,
    #[serde(default)]
    pub boundary_inputs: Vec<BoundaryRecord>,
    #[serde(default)]
    pub boundary_outputs: Vec<BoundaryRecord>,
    /// `(item, x, y)` — matches `LayoutResult::surplus_exits`'s tuple
    /// shape, which serde serializes as a 3-element JSON array.
    #[serde(default)]
    pub surplus_exits: Vec<(String, i32, i32)>,
    pub bbox_min: [i32; 2],
    pub dims: [i32; 2],
    #[serde(default)]
    pub entities: usize,
    #[serde(default)]
    pub stacking: u8,
    #[serde(default)]
    pub inserter_capacity: u8,
}

impl Manifest {
    pub fn from_str(s: &str) -> Result<Manifest, String> {
        serde_json::from_str(s).map_err(|e| format!("manifest parse error: {e}"))
    }

    /// True if any boundary (input, output, or surplus exit) is fluid —
    /// the RFC requires fluid-fed runs to be flagged UNCALIBRATED in the
    /// report (no fixture has exercised the infinity-pipe feed/void paths).
    pub fn has_fluid_boundary(&self) -> bool {
        self.boundary_inputs.iter().any(|b| b.is_fluid)
            || self.boundary_outputs.iter().any(|b| b.is_fluid)
            || !self.surplus_exits.is_empty()
    }

    /// True if every boundary direction is one this harness has live
    /// calibration evidence for. The #345 dogfood + gear10 PASS artifact
    /// only ever exercised south-facing inputs into a top-fed bus; the
    /// vector-generalized jog (see `scenario::rot90`) is a faithful,
    /// low-risk extension of that exact mechanism to the other three
    /// cardinal directions, but has never been measured against a live
    /// server. Callers should surface this as an UNCALIBRATED flag, not
    /// silently treat every direction as equally trustworthy.
    pub fn has_uncalibrated_direction(&self) -> bool {
        self.boundary_inputs
            .iter()
            .any(|b| b.direction().vector() != (0, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_json() -> &'static str {
        include_str!("../tests/fixtures/manifest_gear10.json")
    }

    #[test]
    fn parses_real_schema_fixture() {
        let m = Manifest::from_str(fixture_json()).expect("parse");
        assert_eq!(m.label, "gear");
        assert_eq!(m.targets.len(), 1);
        assert_eq!(m.targets[0].item, "iron-gear-wheel");
        assert_eq!(m.boundary_inputs.len(), 2);
        assert_eq!(m.boundary_outputs.len(), 1);
        assert_eq!(m.bbox_min, [1, 0]);
        assert_eq!(m.planned_rates["iron-gear-wheel"], 10.0);
    }

    #[test]
    fn boundary_direction_decodes() {
        let m = Manifest::from_str(fixture_json()).expect("parse");
        for b in &m.boundary_inputs {
            assert_eq!(b.direction(), Direction::South);
        }
        assert!(!m.has_uncalibrated_direction());
    }

    #[test]
    fn rot90_matches_calibrated_drain_convention() {
        // south (0,1) -> east (1,0): side=-1 offset is west, pickup faces
        // east (toward the belt column) exactly as the calibrated
        // gen_harness_scenario.py drain rig hardcodes.
        assert_eq!(rot90((0, 1)), (1, 0));
        // east (1,0) -> south (0,-1)... sanity check the rotation is a
        // consistent quarter-turn in one direction for every axis.
        assert_eq!(rot90((1, 0)), (0, -1));
        assert_eq!(rot90((0, -1)), (-1, 0));
        assert_eq!(rot90((-1, 0)), (0, 1));
    }

    #[test]
    fn no_fluid_boundary_in_gear_fixture() {
        let m = Manifest::from_str(fixture_json()).expect("parse");
        assert!(!m.has_fluid_boundary());
    }
}
