//! Power coverage and pole network connectivity validation.
//!
//! Port of `check_power_coverage` from `src/validate.py`.
//!
//! Checks that every machine is within range of a medium-electric-pole.
//! A medium electric pole has a 7×7 supply area (3 tiles in each direction
//! from the pole center).
//!
//! Also checks that all power poles form a single connected graph via copper
//! wire (P6). Disconnected poles are a Warning, not an Error — they still
//! function but require separate power sources.

use crate::common::needs_electricity;
use crate::models::LayoutResult;
use crate::validate::{Severity, ValidationIssue};

// Wire-reach values (medium 9, small 7.5, substation 18, big 32, all
// +2/quality level) live in `common::pole_wire_reach` — the shared table
// `power_wires::compute_pole_wires` (export + this validator + the web
// overlay) and `bus::layout::repair_pole_connectivity` all read, so
// placement-time repair, the emitted artifact, and validation can never
// disagree (rfc-build-quality Phase 2 review fix, merged with the
// power-3c wires arc).

/// Check that the EMITTED pole copper-wire graph is a single connected network.
///
/// This validates the ARTIFACT, not mere geometry: it recomputes the exact
/// `wires` array [`crate::blueprint::export`] encodes (via
/// [`crate::power_wires::compute_pole_wires`] — the single source of wire reach
/// and footprint centers) and asserts every pole is reachable from the first
/// pole through it. A geometry-only check could pass while the export omitted
/// the wires entirely, pasting poles as disconnected islands — the bug this
/// check now catches at the artifact level.
///
/// Wiring rule (in `compute_pole_wires`): two poles connect when the Euclidean
/// distance between their footprint centers is ≤ the *smaller* of the two
/// poles' wire reaches, with no per-pole connection cap (Factorio 2.0 removed
/// it). Returns a single `Warning` when any pole is unreachable.
pub fn check_pole_network_connectivity(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let wires = crate::power_wires::compute_pole_wires(&layout.entities);
    let disconnected = crate::power_wires::count_disconnected_poles(&layout.entities, &wires);
    if disconnected == 0 {
        return vec![];
    }

    vec![ValidationIssue::new(
        Severity::Warning,
        "power",
        format!(
            "{disconnected} power pole(s) are not connected to the main pole network via copper wire"
        ),
    )]
}

/// Check that every electric entity is within a power pole's supply area.
///
/// Returns a list of [`ValidationIssue`]s (all with severity `Warning`).
pub fn check_power_coverage(layout_result: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Supply sources: (center_x, center_y, supply_area_distance) in continuous
    // tile-space for every power-distribution entity — medium-electric-poles
    // (center +0.5, reach 3.5) and substations (center +1.0, reach 9.0), both
    // from the shared `entity_size` + `supply_area_distance` (RFC Phase 3a-i/
    // 3a-ii). Continuous coordinates make coverage EXACT for the even 2×2
    // substation, not the +1-tile false-accept the integer version gave.
    let poles: Vec<(f64, f64, f64)> = layout_result
        .entities
        .iter()
        .filter(|e| e.name == "medium-electric-pole" || e.name == "substation")
        .map(|e| {
            let (w, h) = crate::common::entity_size(&e.name);
            (
                e.x as f64 + w as f64 / 2.0,
                e.y as f64 + h as f64 / 2.0,
                crate::common::supply_area_distance(&e.name, e.quality.unwrap_or_default()),
            )
        })
        .collect();

    if poles.is_empty() {
        issues.push(ValidationIssue::new(
            Severity::Warning,
            "power",
            "No power poles in layout",
        ));
        return issues;
    }

    for e in &layout_result.entities {
        // Coverage subjects (RFC `docs/rfc-power-supply.md` Phase 0b + 0f):
        // everything that draws grid power, via the single `needs_electricity`
        // fact — electric crafting machines (checked at their footprint
        // center) and electric inserters (1×1, checked at their own tile).
        // A burner biochamber (needs_electricity false) and all non-powered
        // entities (belts, pipes, poles) are skipped. Phase 0b widened this
        // beyond the old 6-machine list; Phase 0f folded inserters in (before
        // it, only machine centers were checked, hiding the ~40-52% of
        // inserters one tile beyond a north-band pole's supply area).
        if !needs_electricity(&e.name) {
            continue;
        }
        // Subject center in continuous tile-space (index + size/2) from the
        // shared `entity_size`: a machine's footprint center, a 1×1 inserter's
        // tile center. Byte-identical to the old integer machine-center /
        // own-tile split once paired with the pole's +0.5 center below.
        let (w, h) = crate::common::entity_size(&e.name);
        let (scx, scy) = (e.x as f64 + w as f64 / 2.0, e.y as f64 + h as f64 / 2.0);

        let powered = poles
            .iter()
            .any(|&(pcx, pcy, d)| (scx - pcx).abs() <= d && (scy - pcy).abs() <= d);

        if !powered {
            issues.push(ValidationIssue::with_pos(
                Severity::Warning,
                "power",
                format!("{} at ({},{}): not in range of any power pole", e.name, e.x, e.y),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use crate::common::QualityTier;

    /// The RFC's pole-margin table as an invariant (rfc-build-quality;
    /// kill-1 verified in-game 2026-07-20): medium poles keep a strict
    /// 2-tile margin between supply DIAMETER and wire reach at every
    /// quality tier (7<9 ... 17<19), so coverage-spaced mediums can never
    /// strand — while the substation has ZERO margin at every tier
    /// (18=18 ... 28=28), so spacing logic may never assume its wire
    /// reach exceeds its supply diameter. Equality asserted exactly so a
    /// data-table typo can't hide.
    #[test]
    fn pole_margin_invariants_per_quality_tier() {
        for tier in QualityTier::ALL {
            let m_supply = 2.0 * crate::common::supply_area_distance("medium-electric-pole", tier);
            let m_wire = crate::common::pole_wire_reach("medium-electric-pole", tier).unwrap();
            assert_eq!(m_wire - m_supply, 2.0, "medium margin {tier:?}");
            let s_supply = 2.0 * crate::common::supply_area_distance("substation", tier);
            let s_wire = crate::common::pole_wire_reach("substation", tier).unwrap();
            assert_eq!(s_wire, s_supply, "substation zero-margin {tier:?}");
        }
    }

    use super::*;
    use crate::models::{EntityDirection, PlacedEntity};

    fn machine(name: &str, x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            direction: EntityDirection::North,
            recipe: Some("iron-gear-wheel".to_string()),
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn pole(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "medium-electric-pole".to_string(),
            x,
            y,
            direction: EntityDirection::North,
            recipe: None,
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn layout(entities: Vec<PlacedEntity>) -> LayoutResult {
        LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        }
    }

    // --- Substation exact-coverage boundary (RFC Phase 3a-ii carried
    // constraint: the continuous-coordinate check must be EXACT for the even
    // 2×2 footprint, not the +1-tile false-accept the integer version gave) ---

    fn inserter(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity { name: "inserter".to_string(), x, y, ..Default::default() }
    }

    #[test]
    fn substation_coverage_is_exact_at_the_even_footprint_edge() {
        // Substation top-left (0,0) → center (1.0,1.0), supply_area_distance 9.
        // A 1×1 inserter's center is index+0.5, so it is covered iff
        // |x+0.5 − 1| ≤ 9 → x ∈ [−8, 9]. The old integer ±9-from-center test
        // covered x=10 too — a false-accept this check must not repeat.
        let sub = PlacedEntity { name: "substation".to_string(), x: 0, y: 0, ..Default::default() };
        assert!(
            check_power_coverage(&layout(vec![sub.clone(), inserter(9, 1)])).is_empty(),
            "inserter at the +9 supply edge must be covered"
        );
        let past = check_power_coverage(&layout(vec![sub.clone(), inserter(10, 1)]));
        assert_eq!(past.len(), 1, "inserter one tile past the exact supply must be flagged");
    }

    // --- No poles at all ---

    #[test]
    fn no_poles_returns_single_warning() {
        let lr = layout(vec![machine("assembling-machine-1", 0, 0)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].category, "power");
        assert!(issues[0].message.contains("No power poles"));
    }

    #[test]
    fn no_poles_empty_layout_returns_single_warning() {
        let lr = layout(vec![]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].message, "No power poles in layout");
    }

    // --- Machine within range ---

    #[test]
    fn machine_within_range_no_issues() {
        // 3x3 machine at (0,0): center = (1,1); pole at (4,4): distance = (3,3) — exactly at edge
        let lr = layout(vec![machine("assembling-machine-1", 0, 0), pole(4, 4)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn machine_directly_under_pole_no_issues() {
        let lr = layout(vec![machine("assembling-machine-2", 0, 0), pole(1, 1)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 0);
    }

    // --- Machine out of range ---

    #[test]
    fn machine_out_of_range_returns_warning() {
        // 3x3 machine at (0,0): center = (1,1); pole at (10,10): clearly out of range
        let lr = layout(vec![machine("assembling-machine-1", 0, 0), pole(10, 10)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].category, "power");
        assert!(issues[0].message.contains("assembling-machine-1"));
        assert_eq!(issues[0].x, Some(0));
        assert_eq!(issues[0].y, Some(0));
    }

    #[test]
    fn machine_just_outside_range_returns_warning() {
        // 3x3 machine at (0,0): center = (1,1); pole at (5,5): distance = (4,4) > 3
        let lr = layout(vec![machine("assembling-machine-3", 0, 0), pole(5, 5)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
    }

    // --- Oil refinery (5x5) ---

    #[test]
    fn oil_refinery_center_computed_correctly() {
        // 5x5 oil-refinery at (0,0): center = (2,2); pole at (5,5): distance = (3,3) — at edge
        let lr = layout(vec![
            PlacedEntity {
                name: "oil-refinery".to_string(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("basic-oil-processing".to_string()),
                io_type: None,
                carries: None,
                mirror: false,
                segment_id: None,
                ..Default::default()
            },
            pole(5, 5),
        ]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 0, "oil-refinery center (2,2) should be within range of pole at (5,5)");
    }

    // --- Multiple machines, mixed coverage ---

    #[test]
    fn multiple_machines_only_uncovered_reported() {
        let lr = layout(vec![
            machine("assembling-machine-1", 0, 0),  // center (1,1), pole at (2,2) → in range
            machine("assembling-machine-2", 15, 15), // center (16,16), out of range
            pole(2, 2),
        ]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].x, Some(15));
    }

    #[test]
    fn multiple_poles_any_covers_machine() {
        // Machine center (1,1); no single pole within range, but two poles together cover it
        let lr = layout(vec![
            machine("assembling-machine-1", 0, 0),
            pole(1, 10), // out of range
            pole(1, 1),  // in range
        ]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 0);
    }

    // --- Non-machine entities are ignored ---

    #[test]
    fn non_machine_entities_ignored() {
        let belt = PlacedEntity {
            name: "transport-belt".to_string(),
            x: 0,
            y: 0,
            direction: EntityDirection::North,
            recipe: None,
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        };
        // No poles, but only a belt → the "No power poles" warning fires (not a per-entity warning)
        let lr = layout(vec![belt]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("No power poles"));
    }

    // --- All machine types covered ---

    #[test]
    fn all_machine_types_checked() {
        // Every electric machine (canonical ∩ needs_electricity) is a
        // power-coverage subject. Widened in RFC Phase 0b to add foundry,
        // centrifuge, recycler, cryogenic-plant, electromagnetic-plant.
        let machine_names = [
            "assembling-machine-1",
            "assembling-machine-2",
            "assembling-machine-3",
            "chemical-plant",
            "electric-furnace",
            "oil-refinery",
            "foundry",
            "centrifuge",
            "recycler",
            "cryogenic-plant",
            "electromagnetic-plant",
        ];
        for name in &machine_names {
            let lr = layout(vec![machine(name, 0, 0)]);
            // No poles → warning
            let issues = check_power_coverage(&lr);
            assert_eq!(issues.len(), 1, "{} should trigger 'No power poles' warning", name);
        }
    }

    #[test]
    fn biochamber_excluded_foundry_included_from_coverage() {
        // A biochamber far from any pole must NOT warn — it is burner-fueled
        // (needs_electricity false), so it draws no grid power. A foundry in
        // the same spot MUST warn (it is electric and out of range). This is
        // the Phase 0b widening's headline correctness property.
        let lr = layout(vec![machine("biochamber", 0, 0), pole(30, 30)]);
        let issues = check_power_coverage(&lr);
        assert!(
            issues.is_empty(),
            "biochamber is burner-fueled and must not be a power-coverage subject: {issues:?}"
        );

        let lr = layout(vec![machine("foundry", 0, 0), pole(30, 30)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1, "foundry is electric and out of range → must warn");
        assert!(issues[0].message.contains("foundry"));
    }

    // --- Done-when criterion: layout missing power reports uncovered machines ---

    #[test]
    fn layout_missing_power_reports_uncovered_machines() {
        // 3 machines, no poles → "No power poles" single warning
        let lr = layout(vec![
            machine("assembling-machine-1", 0, 0),
            machine("assembling-machine-2", 5, 0),
            machine("assembling-machine-3", 10, 0),
        ]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("No power poles"));
    }

    #[test]
    fn layout_with_full_coverage_reports_zero_issues() {
        // Pole at (1,1) covers machine at (0,0) with center (1,1) → distance 0
        let lr = layout(vec![machine("assembling-machine-1", 0, 0), pole(1, 1)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn pole_range_boundary_exact_3_tiles() {
        // Machine center at (0,0) (1x1 for simplicity — but our smallest is 3x3)
        // Use 3x3 at (-1,-1) so center = (0,0); pole at (3,0) → distance = exactly 3
        let lr = layout(vec![machine("assembling-machine-1", -1, -1), pole(3, 0)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 0, "distance of exactly 3 should be within range");
    }

    #[test]
    fn pole_range_boundary_4_tiles_out_of_range() {
        // 3x3 at (-1,-1) center (0,0); pole at (4,0) → distance = 4 > POLE_RANGE
        let lr = layout(vec![machine("assembling-machine-1", -1, -1), pole(4, 0)]);
        let issues = check_power_coverage(&lr);
        assert_eq!(issues.len(), 1, "distance of 4 should be out of range");
    }

    // -----------------------------------------------------------------------
    // check_pole_network_connectivity tests
    // -----------------------------------------------------------------------

    fn small_pole(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "small-electric-pole".to_string(),
            x,
            y,
            direction: EntityDirection::North,
            recipe: None,
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    #[test]
    fn single_pole_trivially_connected() {
        let lr = layout(vec![pole(0, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn no_poles_trivially_connected() {
        let lr = layout(vec![machine("assembling-machine-1", 0, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn two_medium_poles_within_wire_reach_connected() {
        // Medium pole wire reach = 9 tiles. Centers at (0.5, 0.5) and (8.5, 0.5) → dist = 8.0
        let lr = layout(vec![pole(0, 0), pole(8, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 0, "poles 8 tiles apart should be within medium-pole reach of 9");
    }

    #[test]
    fn two_medium_poles_exactly_at_wire_reach_connected() {
        // Centers at (0.5, 0.5) and (9.5, 0.5) → dist = 9.0 == reach → connected
        let lr = layout(vec![pole(0, 0), pole(9, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 0, "distance of exactly 9.0 should be within reach");
    }

    #[test]
    fn two_medium_poles_just_outside_wire_reach_disconnected() {
        // Centers at (0.5, 0.5) and (10.5, 0.5) → dist = 10.0 > 9.0
        let lr = layout(vec![pole(0, 0), pole(10, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].category, "power");
        assert!(issues[0].message.contains("not connected"));
    }

    #[test]
    fn connected_line_of_poles_no_issues() {
        // Three medium poles spaced 8 tiles apart: 0, 8, 16 — each adjacent pair within reach
        let lr = layout(vec![pole(0, 0), pole(8, 0), pole(16, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn disconnected_cluster_reports_warning() {
        // Two groups: poles at x=0 and x=20 (dist=20 > reach=9)
        let lr = layout(vec![pole(0, 0), pole(0, 8), pole(20, 0), pole(20, 8)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("2 power pole(s)"));
    }

    #[test]
    fn small_pole_uses_smaller_wire_reach() {
        // Small pole reach = 7.5. Centers at (0.5,0.5) and (8.5,0.5) → dist = 8.0 > 7.5
        let lr = layout(vec![small_pole(0, 0), small_pole(8, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 1, "small poles 8 tiles apart should be out of reach (7.5)");
    }

    #[test]
    fn mixed_pole_types_use_minimum_reach() {
        // medium (reach=9) + small (reach=7.5): min=7.5. dist between (0.5,0.5) and (8.5,0.5) = 8.0 > 7.5
        let lr = layout(vec![pole(0, 0), small_pole(8, 0)]);
        let issues = check_pole_network_connectivity(&lr);
        assert_eq!(issues.len(), 1, "mixed poles should use min reach (7.5), distance 8.0 > 7.5");
    }
}
