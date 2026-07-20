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

use std::collections::VecDeque;

use crate::common::{is_machine_entity, machine_dims, needs_electricity};
use crate::models::LayoutResult;
use crate::validate::{Severity, ValidationIssue};

/// Wire reach (tile distance between centers) for each pole type.
const MEDIUM_POLE_WIRE_REACH: f64 = 9.0;
const SMALL_POLE_WIRE_REACH: f64 = 7.5;
/// Substation-to-substation copper wire reach (RFP Phase 3a-i; draftsman
/// `maximum_wire_distance = 18`). A substation-to-medium-pole connection uses
/// `min(18, 9) = 9` via the `ar.min(br)` rule in the connectivity walk.
const SUBSTATION_WIRE_REACH: f64 = 18.0;

/// Center in tile-space of a power-distribution entity's footprint. medium /
/// small-electric-poles are 1×1 (center +0.5); a substation is 2×2 (center
/// +1.0). Size from the shared [`entity_size`] (RFP Phase 3a-i — the old
/// hard-coded +0.5, and its comment calling the 1×1 medium pole "2×2", were
/// both wrong for the substation).
fn pole_center(name: &str, x: i32, y: i32) -> (f64, f64) {
    let (w, h) = crate::common::entity_size(name);
    (x as f64 + w as f64 / 2.0, y as f64 + h as f64 / 2.0)
}

/// Wire reach for a given pole name; returns `None` for unknown pole types.
fn wire_reach(name: &str) -> Option<f64> {
    match name {
        "medium-electric-pole" => Some(MEDIUM_POLE_WIRE_REACH),
        "small-electric-pole" => Some(SMALL_POLE_WIRE_REACH),
        "substation" => Some(SUBSTATION_WIRE_REACH),
        _ => None,
    }
}

/// Check that all power poles form a single connected graph via copper wire.
///
/// Two poles are connected when the Euclidean distance between their centers
/// is ≤ the wire reach of the *smaller* reach of the two poles (Factorio uses
/// the minimum of both poles' wire reaches).
///
/// Returns a single `Warning` issue if any pole is unreachable from the first
/// pole in the layout.
pub fn check_pole_network_connectivity(layout: &LayoutResult) -> Vec<ValidationIssue> {
    // Collect (center_x, center_y, wire_reach) for every known pole type.
    let poles: Vec<(f64, f64, f64)> = layout
        .entities
        .iter()
        .filter_map(|e| {
            wire_reach(&e.name).map(|r| {
                let (cx, cy) = pole_center(&e.name, e.x, e.y);
                (cx, cy, r)
            })
        })
        .collect();

    if poles.len() <= 1 {
        // 0 or 1 pole — trivially connected (or no poles, covered by check_power_coverage).
        return vec![];
    }

    let mut visited = vec![false; poles.len()];
    let mut queue = VecDeque::new();
    queue.push_back(0usize);
    visited[0] = true;

    while let Some(i) = queue.pop_front() {
        let (acx, acy, ar) = poles[i];
        for (j, &(bcx, bcy, br)) in poles.iter().enumerate() {
            if visited[j] {
                continue;
            }
            let dx = acx - bcx;
            let dy = acy - bcy;
            let reach = ar.min(br);
            // Compare squared distances to avoid sqrt.
            if dx * dx + dy * dy <= reach * reach {
                visited[j] = true;
                queue.push_back(j);
            }
        }
    }

    let disconnected: usize = visited.iter().filter(|&&v| !v).count();
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

    // Supply sources: (center_x, center_y, supply_range) for every power-
    // distribution entity — medium-electric-poles (range 3) and substations
    // (range 9), both from the shared `pole_supply_range` (RFP Phase 3a-i,
    // unifying the former `POLE_RANGE = 3` constant with its layout.rs twin).
    // Substations don't appear in generated layouts yet (3a-i is non-layout-
    // moving), so this is byte-identical on the corpus.
    let pole_positions: Vec<(i32, i32, i32)> = layout_result
        .entities
        .iter()
        .filter(|e| e.name == "medium-electric-pole" || e.name == "substation")
        .map(|e| {
            let (w, h) = crate::common::entity_size(&e.name);
            (
                e.x + w as i32 / 2,
                e.y + h as i32 / 2,
                crate::common::pole_supply_range(&e.name),
            )
        })
        .collect();

    if pole_positions.is_empty() {
        issues.push(ValidationIssue::new(
            Severity::Warning,
            "power",
            "No power poles in layout",
        ));
        return issues;
    }

    for e in &layout_result.entities {
        // Coverage subjects (RFP `docs/rfp-power-supply.md` Phase 0b + 0f):
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
        let (cx, cy) = if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            (e.x + w as i32 / 2, e.y + h as i32 / 2)
        } else {
            // Electric inserter — 1×1, powered from its own tile.
            (e.x, e.y)
        };

        let powered = pole_positions
            .iter()
            .any(|&(pcx, pcy, r)| (cx - pcx).abs() <= r && (cy - pcy).abs() <= r);

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
        // power-coverage subject. Widened in RFP Phase 0b to add foundry,
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
