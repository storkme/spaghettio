//! Module loadout checks (RFC-044 Phase 1).
//!
//! Two checks over `PlacedEntity.items`:
//!
//! - **module-slots** — the module count must fit the entity's slot count
//!   (`common::module_slots`), and module-shaped names we don't recognize
//!   (modded tiers) warn as unratable.
//! - **module-eligibility** — productivity modules require the recipe's
//!   `allow_productivity`, and a module's BENEFICIAL effect must be in
//!   the machine's `allowed_effects` (recycler forbids productivity,
//!   oil-refinery forbids quality, ...). Harmful side-effects never gate
//!   — see `module_gating_effect`.
//!
//! Both emit WARNINGS, not errors: an in-game paste doesn't fail on an
//! invalid loadout — the offending module requests are silently never
//! fulfilled, which is exactly the silent-failure class these checks
//! surface. Keeping them non-fatal also means imported community
//! blueprints can't be blocked from rendering by their module quirks.
//!
//! `items` carries ALL item requests (fuel, ammo, modules); only
//! module-shaped names participate here — a coal request on a furnace is
//! legitimate and ignored.

use crate::common::module_slots_known;
use crate::models::LayoutResult;
use crate::recipe_db::db;
use crate::validate::{Severity, ValidationIssue};

/// The GATING effect of a module family. The game gates insertion on the
/// module's BENEFICIAL effect only — harmful side-effects never gate
/// (draftsman 2.0.76: speed modules carry a quality malus, yet
/// speed-in-beacon is legal even though beacon's `allowed_effects` has no
/// "quality"). So each family reduces to the one effect that must be in
/// the machine's `allowed_effects`.
fn module_gating_effect(family: &str) -> Option<&'static str> {
    match family {
        "speed" => Some("speed"),
        "productivity" => Some("productivity"),
        "efficiency" => Some("consumption"),
        "quality" => Some("quality"),
        _ => None,
    }
}

/// Classify an item-request name: `Some(family)` for known module
/// prototypes, `None` for non-module requests (fuel, ammo, ...).
/// The `effectivity-module*` names are the pre-2.0 spelling of
/// `efficiency-module*` — the game migrates them on paste, and the
/// community corpus is full of them, so they alias into the efficiency
/// family rather than warning as unknown. Module-shaped names outside
/// the known set (modded tiers like `speed-module-4`) return
/// `Some("unknown")`; the shape test strips a trailing tier number and
/// requires a `-module` SUFFIX so real items like `empty-module-slot`
/// never match.
fn module_family(name: &str) -> Option<&'static str> {
    crate::common::game_module_family(name)
}

/// `allowed_effects` for module hosts that aren't in the machines dict,
/// hand-tabled from draftsman 2.0.76. Labs and mining drills are
/// deliberately ABSENT — their prototype `allowed_effects` is nil, which
/// means "all effects allowed", so skipping them is exact, not a gap.
fn fallback_allowed_effects(entity: &str) -> Option<&'static [&'static str]> {
    match entity {
        // Forbids productivity and quality.
        "beacon" => Some(&["speed", "consumption", "pollution"]),
        // Forbid quality only.
        "rocket-silo" | "pumpjack" => {
            Some(&["speed", "consumption", "pollution", "productivity"])
        }
        _ => None,
    }
}

/// module-slots: per entity, Σ module counts ≤ `module_slots(entity)`;
/// unknown module tiers warn (they can't be rated by the effect tables).
pub fn check_module_slots(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for ent in &layout.entities {
        if ent.items.is_empty() {
            continue;
        }
        let mut module_count: u32 = 0;
        for it in &ent.items {
            match module_family(&it.item) {
                None => {}
                Some("unknown") if it.count > 0 => {
                    issues.push(ValidationIssue::with_pos(
                        Severity::Warning,
                        "module-slots",
                        format!(
                            "{} at ({}, {}): unknown module '{}' — not in the \
                             known module tables, effects can't be rated",
                            ent.name, ent.x, ent.y, it.item
                        ),
                        ent.x,
                        ent.y,
                    ));
                    module_count += it.count;
                }
                Some("unknown") => {}
                Some(_) => module_count += it.count,
            }
        }
        // Unknown (modded) entities carry no slot claim we can check —
        // asserting "0 slots" about se-recycling-facility would be wrong.
        let Some(slots) = module_slots_known(&ent.name) else {
            continue;
        };
        if module_count > slots {
            // No `.with_detail` here: IssueDetail is for rate-shaped
            // issues, and the web starvation heatmap reads any detail
            // pair category-blind — a slot overflow is not "80% starved".
            issues.push(ValidationIssue::with_pos(
                Severity::Warning,
                "module-slots",
                format!(
                    "{} at ({}, {}): {} modules requested but the machine \
                     has {} module slot{} — the surplus requests are never \
                     fulfilled in game",
                    ent.name,
                    ent.x,
                    ent.y,
                    module_count,
                    slots,
                    if slots == 1 { "" } else { "s" }
                ),
                ent.x,
                ent.y,
            ));
        }
    }
    issues
}

/// module-eligibility: productivity needs the recipe whitelist, and every
/// module effect must be allowed by the machine.
pub fn check_module_eligibility(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for ent in &layout.entities {
        if ent.items.is_empty() {
            continue;
        }
        let machine = db().machines.get(ent.name.as_str());
        let allowed: Option<Vec<&str>> = machine
            .and_then(|m| m.allowed_effects.as_ref())
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .or_else(|| fallback_allowed_effects(&ent.name).map(|v| v.to_vec()));

        for it in &ent.items {
            let family = match module_family(&it.item) {
                Some(f) if f != "unknown" => f,
                _ => continue,
            };

            // Machine-level: the module's beneficial (gating) effect must
            // be in the machine's allowed_effects.
            if let (Some(allowed), Some(gating)) = (&allowed, module_gating_effect(family)) {
                if !allowed.contains(&gating) {
                    issues.push(ValidationIssue::with_pos(
                        Severity::Warning,
                        "module-eligibility",
                        format!(
                            "{} at ({}, {}): {} not insertable — machine \
                             forbids the '{}' effect",
                            ent.name, ent.x, ent.y, it.item, gating
                        ),
                        ent.x,
                        ent.y,
                    ));
                    continue; // recipe-level check would double-report
                }
            }

            // Recipe-level: productivity modules only on whitelisted recipes.
            if family == "productivity" {
                if let Some(recipe_name) = &ent.recipe {
                    if let Some(recipe) = db().recipes.get(recipe_name.as_str()) {
                        if !recipe.allow_productivity {
                            issues.push(ValidationIssue::with_pos(
                                Severity::Warning,
                                "module-eligibility",
                                format!(
                                    "{} at ({}, {}): {} not insertable — recipe \
                                     '{}' does not allow productivity",
                                    ent.name, ent.x, ent.y, it.item, recipe_name
                                ),
                                ent.x,
                                ent.y,
                            ));
                        }
                    }
                }
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, ModuleItem, PlacedEntity};

    fn entity(name: &str, recipe: Option<&str>, items: Vec<(&str, u32)>) -> PlacedEntity {
        PlacedEntity {
            name: name.into(),
            x: 0,
            y: 0,
            direction: EntityDirection::North,
            recipe: recipe.map(|r| r.into()),
            items: items
                .into_iter()
                .map(|(item, count)| ModuleItem {
                    item: item.into(),
                    count,
                    quality: None,
                })
                .collect(),
            ..Default::default()
        }
    }

    fn layout(entities: Vec<PlacedEntity>) -> LayoutResult {
        LayoutResult {
            entities,
            width: 20,
            height: 10,
            ..Default::default()
        }
    }

    #[test]
    fn overstuffed_machine_warns() {
        // AM3 has 4 slots; 5 modules requested.
        let l = layout(vec![entity(
            "assembling-machine-3",
            Some("iron-gear-wheel"),
            vec![("productivity-module-3", 3), ("speed-module", 2)],
        )]);
        let issues = check_module_slots(&l);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "module-slots");
        assert!(issues[0].message.contains("5 modules"));
        assert!(issues[0].message.contains("4 module slots"));
    }

    #[test]
    fn zero_slot_machine_with_module_warns() {
        // AM1 has zero module slots.
        let l = layout(vec![entity(
            "assembling-machine-1",
            Some("iron-gear-wheel"),
            vec![("speed-module", 1)],
        )]);
        assert_eq!(check_module_slots(&l).len(), 1);
    }

    #[test]
    fn fuel_and_ammo_requests_are_not_modules() {
        // Non-module item requests (fuel etc.) must not count toward
        // slots or trip eligibility.
        let l = layout(vec![entity("stone-furnace", None, vec![("coal", 50)])]);
        assert!(check_module_slots(&l).is_empty());
        assert!(check_module_eligibility(&l).is_empty());
    }

    #[test]
    fn unknown_module_tier_warns_but_counts() {
        let l = layout(vec![entity(
            "assembling-machine-3",
            None,
            vec![("speed-module-4", 4)],
        )]);
        let issues = check_module_slots(&l);
        // One unknown-module warning; count 4 still fits the 4 slots.
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("unknown module"));
    }

    #[test]
    fn prod_in_recycler_warns_machine_level() {
        let l = layout(vec![entity(
            "recycler",
            None,
            vec![("productivity-module-3", 1)],
        )]);
        let issues = check_module_eligibility(&l);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("productivity"));
    }

    #[test]
    fn quality_module_in_refinery_warns() {
        let l = layout(vec![entity(
            "oil-refinery",
            Some("advanced-oil-processing"),
            vec![("quality-module-2", 1)],
        )]);
        let issues = check_module_eligibility(&l);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("quality"));
    }

    #[test]
    fn prod_in_beacon_warns_via_fallback_table() {
        let l = layout(vec![entity(
            "beacon",
            None,
            vec![("productivity-module", 2)],
        )]);
        assert_eq!(check_module_eligibility(&l).len(), 1);
    }

    #[test]
    fn speed_in_beacon_is_clean() {
        let l = layout(vec![entity("beacon", None, vec![("speed-module-3", 2)])]);
        assert!(check_module_eligibility(&l).is_empty());
    }

    #[test]
    fn prod_on_non_eligible_recipe_warns_recipe_level() {
        // AM3 allows the productivity effect; iron-chest is not on the
        // recipe whitelist — isolates the recipe-level rule.
        let l = layout(vec![entity(
            "assembling-machine-3",
            Some("iron-chest"),
            vec![("productivity-module-3", 1)],
        )]);
        let issues = check_module_eligibility(&l);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("iron-chest"));
    }

    #[test]
    fn prod_on_eligible_recipe_is_clean() {
        let l = layout(vec![entity(
            "assembling-machine-3",
            Some("iron-gear-wheel"),
            vec![("productivity-module-3", 4)],
        )]);
        assert!(check_module_eligibility(&l).is_empty());
        assert!(check_module_slots(&l).is_empty());
    }

    #[test]
    fn kovarex_prod_is_clean() {
        // Kovarex is prod-eligible in 2.0 (catalyst-exempt crediting is
        // solver business, not eligibility business).
        let l = layout(vec![entity(
            "centrifuge",
            Some("kovarex-enrichment-process"),
            vec![("productivity-module-3", 2)],
        )]);
        assert!(check_module_eligibility(&l).is_empty());
    }

    #[test]
    fn effectivity_modules_alias_to_efficiency() {
        // Pre-2.0 spelling, migrated by the game on paste; the corpus is
        // full of it (review MAJOR-1: 94 false unknown-module warnings).
        let l = layout(vec![entity(
            "assembling-machine-2",
            Some("iron-gear-wheel"),
            vec![("effectivity-module", 2)],
        )]);
        assert!(check_module_slots(&l).is_empty());
        assert!(check_module_eligibility(&l).is_empty());
    }

    #[test]
    fn unknown_modded_entity_skips_slot_claim() {
        // Review MAJOR-2: we know nothing about se-recycling-facility's
        // slots — no overflow warning may be asserted.
        let l = layout(vec![entity(
            "se-recycling-facility",
            None,
            vec![("productivity-module-3", 4)],
        )]);
        assert!(check_module_slots(&l).is_empty());
    }

    #[test]
    fn pumpjack_slots_and_quality_restriction() {
        // Review MAJOR-3 + MINOR-6: pumpjack has 2 slots and forbids
        // quality (draftsman 2.0.76).
        let over = layout(vec![entity("pumpjack", None, vec![("speed-module-3", 3)])]);
        assert_eq!(check_module_slots(&over).len(), 1);
        let ok = layout(vec![entity("pumpjack", None, vec![("speed-module-3", 2)])]);
        assert!(check_module_slots(&ok).is_empty());
        let q = layout(vec![entity("pumpjack", None, vec![("quality-module", 1)])]);
        assert_eq!(check_module_eligibility(&q).len(), 1);
    }

    #[test]
    fn quality_in_rocket_silo_warns_via_fallback() {
        let l = layout(vec![entity(
            "rocket-silo",
            None,
            vec![("quality-module-3", 1)],
        )]);
        assert_eq!(check_module_eligibility(&l).len(), 1);
    }

    #[test]
    fn empty_module_slot_item_is_not_a_module() {
        // Review NIT-7: real hidden 2.0 item whose name contains
        // "-module" but is not module-shaped (no `-module` suffix after
        // tier-stripping).
        let l = layout(vec![entity(
            "assembling-machine-3",
            None,
            vec![("empty-module-slot", 1)],
        )]);
        assert!(check_module_slots(&l).is_empty());
        assert!(check_module_eligibility(&l).is_empty());
    }

    #[test]
    fn overflow_warning_carries_no_rate_detail() {
        // Review MINOR-5: IssueDetail feeds the category-blind starvation
        // heatmap; a slot overflow must not tint as starvation.
        let l = layout(vec![entity(
            "assembling-machine-3",
            None,
            vec![("speed-module", 5)],
        )]);
        let issues = check_module_slots(&l);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].detail.is_none());
    }

    #[test]
    fn drill_slot_counts_are_present() {
        // Phase 1 prerequisite: drills in the slots table (electric 3,
        // big 4) — moduled drills are ubiquitous in community imports.
        let over = layout(vec![entity(
            "electric-mining-drill",
            None,
            vec![("efficiency-module", 4)],
        )]);
        assert_eq!(check_module_slots(&over).len(), 1);
        let ok = layout(vec![entity(
            "big-mining-drill",
            None,
            vec![("efficiency-module", 4)],
        )]);
        assert!(check_module_slots(&ok).is_empty());
    }
}
