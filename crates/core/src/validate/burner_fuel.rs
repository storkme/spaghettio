//! Burner-fuel delivery validation.
//!
//! The layout engine currently routes recipe ingredients only. Burner-fuelled
//! machines therefore paste with no fuel and cannot run, even when every
//! ingredient belt is connected.

use crate::common::needs_electricity;
use crate::models::{LayoutResult, PlacedEntity};
use crate::validate::{Severity, ValidationIssue};

/// Whether the engine delivers fuel to a burner machine.
///
/// A future fuel-delivery feature satisfies this when an inserter drops a
/// burnable item into `machine`'s fuel inventory. The engine has no such
/// delivery concept today, so no placed burner machine has fuel delivery.
fn has_fuel_delivery(_layout: &LayoutResult, _machine: &PlacedEntity) -> bool {
    false
}

/// Whether this entity is a machine whose operation needs a recipe or mining
/// fuel. Crafting machines are identified by their stamped recipe rather than
/// a prototype-name list, so any future burner crafting machine is covered.
/// Mining drills have no recipe in Factorio; include them should the engine
/// begin placing one.
fn is_fuelled_machine(machine: &PlacedEntity) -> bool {
    machine.recipe.is_some() || machine.name.ends_with("mining-drill")
}

/// Report every placed burner machine with no fuel delivery.
///
/// Grid power and burner fuel are separate obligations. `needs_electricity`
/// deliberately exempts burners from power coverage; this check owns the
/// complementary delivery obligation until the engine can model fuel inserters.
pub fn check_burner_fuel(layout: &LayoutResult) -> Vec<ValidationIssue> {
    layout
        .entities
        .iter()
        .filter(|machine| is_fuelled_machine(machine))
        .filter(|machine| !needs_electricity(&machine.name))
        .filter(|machine| !has_fuel_delivery(layout, machine))
        .map(|machine| {
            let recipe = machine.recipe.as_deref().unwrap_or("mining");
            ValidationIssue::with_pos(
                Severity::Error,
                "burner-fuel",
                format!(
                    "burner-fuel: {} at ({}, {}) running {recipe} has no fuel delivery — the engine delivers no fuel (#461)",
                    machine.name, machine.x, machine.y,
                ),
                machine.x,
                machine.y,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, PlacedEntity};

    fn machine(name: &str, recipe: &str, x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            recipe: Some(recipe.to_string()),
            x,
            y,
            direction: EntityDirection::North,
            ..Default::default()
        }
    }

    #[test]
    fn fires_once_per_burner_machine() {
        let layout = LayoutResult {
            entities: vec![
                machine("biochamber", "bioflux", 1, 2),
                machine("biochamber", "bacteria-1", 5, 7),
                machine("biochamber", "bacteria-2", 9, 11),
            ],
            ..Default::default()
        };

        let issues = check_burner_fuel(&layout);
        assert_eq!(issues.len(), 3, "one issue per unfuelled biochamber");
        assert!(issues
            .iter()
            .all(|issue| { issue.severity == Severity::Error && issue.category == "burner-fuel" }));
    }

    #[test]
    fn does_not_fire_on_electric_machine() {
        let layout = LayoutResult {
            entities: vec![machine("assembling-machine-3", "iron-gear-wheel", 1, 2)],
            ..Default::default()
        };

        assert!(check_burner_fuel(&layout).is_empty());
    }

    #[test]
    fn issue_carries_the_machine_position() {
        let layout = LayoutResult {
            entities: vec![machine("biochamber", "bioflux", 17, -3)],
            ..Default::default()
        };

        let issues = check_burner_fuel(&layout);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].x, Some(17));
        assert_eq!(issues[0].y, Some(-3));
        assert!(issues[0].message.contains("running bioflux"));
    }
}
