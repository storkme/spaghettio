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
//!   error/warning counts at export time". Phase 0 shipped without them and
//!   this module resolved that as optional/absent-tolerant. **Delivered
//!   2026-08-09** (`validator-trust.md` hole 3): `export_with_manifest` now
//!   emits a `validator` object, modeled here as `Option<ValidatorSummary>`.
//!   It is per-category rather than the flat `validator_errors`/
//!   `validator_warnings` the RFC prose named, because a bare total cannot
//!   tell 2 from 218 (`validator-reporting.md`). The field stays optional so
//!   manifests written before it keep parsing — and `None` renders as `?`,
//!   never as clean.

use serde::{Deserialize, Serialize};
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
    /// Fluid targets have no drain rig (their surplus voids are
    /// uncounted), so the report verdicts them on PRODUCED rate.
    /// Defaults false — every pre-existing manifest is a solid target.
    #[serde(default)]
    pub is_fluid: bool,
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
    /// Declared research-sourced productivity, per recipe. The engine plans
    /// and the meter measures at this; the scenario checks the sim's realized
    /// bonus against it and refuses to let a mismatch pass silently.
    ///
    /// Unlike `stacking` and `inserter_capacity`, this is **checked, not
    /// assigned**: recipe productivity is derived from researched technologies
    /// rather than a settable force field, so the scenario cannot simply pin
    /// it the way it pins the inserter and belt bonuses. Detecting the
    /// disagreement is what stops a run being compared against a plan built in
    /// a different world — which is exactly what RFC-064 Phase 2 item 7 was.
    #[serde(default)]
    pub research_productivity: std::collections::BTreeMap<String, f64>,
    /// Validator state of the exact layout this manifest describes, as of
    /// export. Closes hole 3 in `docs/validator-trust.md`.
    ///
    /// `Option`, and absent-tolerant, because manifests written before this
    /// field existed must keep parsing — a pre-existing `.json` on disk is
    /// how most sim fixtures are replayed. **`None` means "this manifest
    /// predates the field", which is not the same as "clean"**; the report
    /// renders it as `?`, never as an all-clear. Distinguishing those two is
    /// the entire point — reading absence as clearance is the failure mode
    /// `validator-reporting.md` catalogues.
    #[serde(default)]
    pub validator: Option<ValidatorSummary>,
}

/// Mirror of `spaghettio_core::validate::ValidatorSummary`.
///
/// Hand-mirrored rather than imported: this crate deliberately does not
/// depend on `spaghettio_core` (RFC-050 constraint, "consumes the manifest
/// schema only"), the same reason every other type in this module is a
/// mirror.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ValidatorSummary {
    #[serde(default)]
    pub errors: usize,
    #[serde(default)]
    pub warnings: usize,
    /// `category -> {errors, warnings}`. Per-category rather than totals
    /// because a bare total cannot tell 2 from 218 — see
    /// `docs/validator-reporting.md`.
    #[serde(default)]
    pub by_category: BTreeMap<String, CategoryCount>,
    /// Pipeline-stamped `LayoutResult::warnings`, which the validator never
    /// sees. Counted apart because reading only the validator has already
    /// produced one false "0 errors 0 warnings" claim.
    #[serde(default)]
    pub layout_warnings: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct CategoryCount {
    #[serde(default)]
    pub errors: usize,
    #[serde(default)]
    pub warnings: usize,
}

impl ValidatorSummary {
    pub fn is_clean(&self) -> bool {
        self.errors == 0 && self.warnings == 0 && self.layout_warnings == 0
    }

    /// Compact badge for report tables: `clean`, or e.g. `2E/5W/1L`.
    pub fn badge(&self) -> String {
        if self.is_clean() {
            return "clean".to_string();
        }
        let mut parts = Vec::new();
        if self.errors > 0 {
            parts.push(format!("{}E", self.errors));
        }
        if self.warnings > 0 {
            parts.push(format!("{}W", self.warnings));
        }
        if self.layout_warnings > 0 {
            parts.push(format!("{}L", self.layout_warnings));
        }
        parts.join("/")
    }

    /// Categories with at least one issue, worst-first, for the report's
    /// one-line explanation of *what* was flagged.
    ///
    /// Can legitimately be empty even when the summary is not clean: a layout
    /// carrying only pipeline-stamped `layout_warnings` has no validator
    /// category to name. Callers must not print a separator unconditionally.
    pub fn top_categories(&self, limit: usize) -> Vec<String> {
        let mut v: Vec<(&String, &CategoryCount)> = self.by_category.iter().collect();
        v.sort_by(|a, b| {
            (b.1.errors, b.1.warnings)
                .cmp(&(a.1.errors, a.1.warnings))
                .then(a.0.cmp(b.0))
        });
        v.into_iter()
            .take(limit)
            .map(|(k, c)| {
                let n = c.errors + c.warnings;
                format!("{k}×{n}")
            })
            .collect()
    }
}

/// How a measured rate should be read given the validator state of the
/// layout it was measured on.
///
/// The distinction the 2026-08-07 PU@1/s incident lacked: a number measured
/// on a layout the validator has already condemned is a fact about *that
/// layout*, not evidence about the pipeline that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeasurementStanding {
    /// Validator was silent. The number measures the pipeline — subject to
    /// the standing caveat that validator silence is not proof of anything.
    Unflagged,
    /// The layout carried warnings. The number measures this layout.
    Warned,
    /// The layout carried errors. It should arguably not have been simmed.
    Condemned,
    /// The manifest predates the validator field — standing is unknown.
    Unknown,
}

impl MeasurementStanding {
    pub fn of(summary: Option<&ValidatorSummary>) -> MeasurementStanding {
        match summary {
            None => MeasurementStanding::Unknown,
            Some(s) if s.errors > 0 => MeasurementStanding::Condemned,
            Some(s) if !s.is_clean() => MeasurementStanding::Warned,
            Some(_) => MeasurementStanding::Unflagged,
        }
    }

    /// One-line caveat to print beside a rate, or `None` when unflagged.
    pub fn caveat(self) -> Option<&'static str> {
        match self {
            MeasurementStanding::Unflagged => None,
            // Deliberately says "warnings", not "validator warnings": this
            // state is also reached by a layout whose only entries are
            // pipeline-stamped `layout_warnings`, which are explicitly NOT
            // validator output (#462). Naming the wrong source would send a
            // reader to the wrong place.
            MeasurementStanding::Warned => Some(
                "measured on a layout carrying warnings (validator and/or \
                 pipeline) — this measures the layout, not the pipeline",
            ),
            MeasurementStanding::Condemned => Some(
                "measured on a layout carrying validator ERRORS — \
                 this measures the layout, not the pipeline",
            ),
            MeasurementStanding::Unknown => Some(
                "manifest predates the validator field — validator state \
                 unknown, not clean",
            ),
        }
    }
}

impl Manifest {
    pub fn from_str(s: &str) -> Result<Manifest, String> {
        serde_json::from_str(s).map_err(|e| format!("manifest parse error: {e}"))
    }

    /// True if any boundary (input, output, or surplus exit) is fluid.
    /// Surfaced in the report as context (`Report::fluid_fed`), not as a
    /// calibration warning: the infinity-pipe feed/void path was flagged
    /// UNCALIBRATED through RFC-050 Phase 1, but #373 found and fixed the
    /// actual defect (an exporter pipe-to-ground direction inversion), and
    /// `plastic-bar` (crude-oil), `sulfur` (water + crude-oil, two fluid
    /// boundaries on one edge), and `land-mine` (mixed fluid+item boundary)
    /// have all since exercised it and PASSed — see #537. A run with a
    /// non-south fluid boundary is still covered by
    /// `has_uncalibrated_direction` below, which remains live.
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

    /// The load-bearing distinction. A manifest written before the validator
    /// field existed must NOT read as clean — that conflation is how a
    /// condemned layout gets quoted as a parity number.
    #[test]
    fn absent_validator_is_unknown_not_clean() {
        let m = Manifest::from_str(fixture_json()).expect("parse");
        assert!(m.validator.is_none(), "gear fixture predates the field");
        let standing = MeasurementStanding::of(m.validator.as_ref());
        assert_eq!(standing, MeasurementStanding::Unknown);
        assert_ne!(standing, MeasurementStanding::Unflagged);
        assert!(
            standing.caveat().is_some(),
            "an unknown-state run must still print a caveat"
        );
    }

    #[test]
    fn standing_separates_clean_warned_and_condemned() {
        let clean = ValidatorSummary::default();
        assert_eq!(
            MeasurementStanding::of(Some(&clean)),
            MeasurementStanding::Unflagged
        );
        assert!(MeasurementStanding::of(Some(&clean)).caveat().is_none());

        let warned = ValidatorSummary {
            warnings: 3,
            ..Default::default()
        };
        assert_eq!(
            MeasurementStanding::of(Some(&warned)),
            MeasurementStanding::Warned
        );

        let condemned = ValidatorSummary {
            errors: 1,
            warnings: 3,
            ..Default::default()
        };
        assert_eq!(
            MeasurementStanding::of(Some(&condemned)),
            MeasurementStanding::Condemned
        );

        // A layout with only pipeline-stamped warnings is still not clean —
        // reading the validator alone produced a false "0 errors 0 warnings"
        // claim once already (#462).
        let layout_only = ValidatorSummary {
            layout_warnings: 2,
            ..Default::default()
        };
        assert!(!layout_only.is_clean());
        assert_eq!(
            MeasurementStanding::of(Some(&layout_only)),
            MeasurementStanding::Warned
        );
    }

    #[test]
    fn badge_and_categories_are_per_category_not_totals() {
        let v = ValidatorSummary {
            errors: 1,
            warnings: 5,
            layout_warnings: 2,
            by_category: BTreeMap::from([
                (
                    "input-rate-delivery".to_string(),
                    CategoryCount {
                        errors: 0,
                        warnings: 4,
                    },
                ),
                (
                    "entity-overlap".to_string(),
                    CategoryCount {
                        errors: 1,
                        warnings: 1,
                    },
                ),
            ]),
        };
        assert_eq!(v.badge(), "1E/5W/2L");
        // Errors sort ahead of warnings, so the overlap error leads even
        // though input-rate-delivery has more total issues. A reader needs
        // to know 4 delivery warnings is not 1 — the whole reason this is
        // per-category (`validator-reporting.md`).
        let cats = v.top_categories(4);
        assert_eq!(cats, vec!["entity-overlap×2", "input-rate-delivery×4"]);
    }

    #[test]
    fn validator_object_parses_when_present() {
        let json = r#"{
            "label": "t", "bbox_min": [0,0], "dims": [4,4],
            "validator": {
              "errors": 2, "warnings": 1, "layout_warnings": 0,
              "by_category": { "pipe-isolation": {"errors": 2, "warnings": 1} }
            }
        }"#;
        let m = Manifest::from_str(json).expect("parse");
        let v = m.validator.expect("validator present");
        assert_eq!(v.errors, 2);
        assert_eq!(v.by_category["pipe-isolation"].errors, 2);
        assert_eq!(
            MeasurementStanding::of(Some(&v)),
            MeasurementStanding::Condemned
        );
    }
}
