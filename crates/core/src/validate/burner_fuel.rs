//! Burner-fuel delivery validation.
//!
//! The layout engine currently routes recipe ingredients only. Burner-fuelled
//! machines therefore paste with no fuel and cannot run, even when every
//! ingredient belt is connected.

use crate::common::is_burner_machine;
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

/// Whether this entity is a burner machine this check can classify: one with
/// a stamped recipe, or a mining drill (mining drills have no recipe in
/// Factorio; include them should the engine begin placing one). Recipe-less
/// burner entities (`boiler`, `burner-inserter`) are out of this check's
/// scope — the engine places none of them.
fn is_fuelled_machine(machine: &PlacedEntity) -> bool {
    machine.recipe.is_some() || machine.name.ends_with("mining-drill")
}

/// Report every placed burner machine with no fuel delivery.
///
/// Grid power and burner fuel are separate obligations. `needs_electricity`
/// deliberately exempts burners from power coverage; this check owns the
/// complementary delivery obligation until the engine can model fuel inserters.
///
/// Classified by [`is_burner_machine`]'s EXPLICIT name list, not
/// `!needs_electricity(name)`: `needs_electricity` is itself an allow-list of
/// known-electric machines falling through to `_ => false`, so its negation
/// means "not a machine this codebase recognizes as electric" — true of
/// `electric-mining-drill`/`big-mining-drill` (real electric machines with no
/// arm in that function) and of any future prototype neither function has
/// seen yet. An unrecognized machine is never assumed to be a burner.
pub fn check_burner_fuel(layout: &LayoutResult) -> Vec<ValidationIssue> {
    layout
        .entities
        .iter()
        .filter(|machine| is_fuelled_machine(machine))
        .filter(|machine| is_burner_machine(&machine.name))
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
    fn does_not_fire_on_electric_mining_drill() {
        // Regression: `electric-mining-drill` has no recipe, so it passed
        // `is_fuelled_machine`'s mining-drill branch, and `needs_electricity`
        // has no arm for it either — `!needs_electricity(name)` would have
        // wrongly condemned it as a burner. `is_burner_machine` is an
        // explicit list, so an unrecognized (but real, electric) machine is
        // never assumed to be a burner.
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "electric-mining-drill".to_string(),
                recipe: None,
                x: 1,
                y: 2,
                direction: EntityDirection::North,
                ..Default::default()
            }],
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
