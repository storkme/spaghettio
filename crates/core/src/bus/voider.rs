//! Voider row synthesis (RFC Fulgora Phase 2, `docs/rfc-fulgora-scrap.md`
//! D1).
//!
//! Under `LayoutOptions::surplus_policy == SurplusPolicy::Void`, solid
//! byproduct surplus (`SolverResult::surplus_outputs`) that resolves to a
//! recognized self-voider recipe (`<item>-recycling`: X -> fraction*X,
//! e.g. `uranium-238-recycling`: U-238 -> 0.25*U-238) gets a
//! layout-synthesized recycler bank instead of an export route to the
//! perimeter. This is deterministic arithmetic derived straight from
//! recipe data — no LP involvement. The solver keeps reporting surplus
//! honestly at the frozen cost table; the LAYOUT decides what to do with
//! it (export or void), the same way it already synthesizes balancers
//! and output mergers without solver involvement. See the RFC's D1
//! design rationale for why voiding lives here and not in the solver.
//!
//! Streams that don't resolve to a self-voider shape (multi-output
//! cascades like `electronic-circuit-recycling` -> iron-plate +
//! copper-cable, or items with no recycling recipe at all) are left in
//! `surplus_outputs` untouched, so they keep following the ordinary
//! `SurplusPolicy::Export` path — never silently dropped.

use crate::models::{ItemFlow, MachineSpec, SolverResult};
use crate::netflow::is_pure_voider;
use crate::recipe_db::{db, get_crafting_speed};

/// Recipe entity that hosts every recycling recipe.
pub const RECYCLER_ENTITY: &str = "recycler";

/// Machine sizing + rate math for a self-voider recipe, or `None` if
/// `item` doesn't resolve to a synthesizable voider shape (missing
/// `<item>-recycling` recipe, or not a pure X -> fraction*X self-voider
/// — e.g. a multi-output cascade hop).
///
/// Shared by [`synthesize_voiders`] (builds the `MachineSpec`),
/// `bus::placer` (re-derives the per-machine recirculated rate at
/// row-render time — see the module doc on why it's re-derived rather
/// than threaded through `MachineSpec`), and
/// `validate::check_stranded_byproducts` (independently re-derives the
/// expected machine count to cross-check the layout's `voided_streams`
/// ledger). Kept as ONE function so these call sites can't drift out of
/// sync.
#[derive(Debug, Clone, Copy)]
pub struct VoiderSizing {
    /// Fraction of the input item returned per craft (e.g. 0.25).
    pub fraction: f64,
    /// Gross per-recycler consumption rate (items/s) at 100% uptime.
    pub per_recycler_rate: f64,
    /// Recycler machine count needed to cover the gross throughput
    /// `rate / (1 - fraction)`.
    pub machines: usize,
}

/// Resolve `item` to a self-voider recipe and size a recycler bank for
/// destroying `rate` items/s of it. `rate` is the item's reported
/// surplus rate (or, for the placer's re-derivation, the reconstructed
/// equivalent — see call site).
pub fn size_self_voider(item: &str, rate: f64) -> Option<VoiderSizing> {
    let recipe_name = recycling_recipe_name(item);
    let recipe = db().recipes.get(&recipe_name)?;
    if !is_pure_voider(recipe) {
        return None;
    }
    let ingredient = recipe.ingredients.first()?;
    let product = recipe.products.first()?;
    if ingredient.amount <= 0.0 {
        return None;
    }
    let fraction = (product.amount * product.probability) / ingredient.amount;
    if !(0.0..1.0).contains(&fraction) {
        return None;
    }
    let crafting_speed = get_crafting_speed(RECYCLER_ENTITY);
    if crafting_speed <= 0.0 || recipe.energy <= 0.0 {
        return None;
    }
    let crafts_per_sec = crafting_speed / recipe.energy;
    let per_recycler_rate = ingredient.amount * crafts_per_sec;
    if per_recycler_rate <= 0.0 || rate <= 0.0 {
        return None;
    }
    let gross = rate / (1.0 - fraction);
    let machines = snap_machine_count(gross / per_recycler_rate);
    Some(VoiderSizing { fraction, per_recycler_rate, machines })
}

/// Canonical recycling recipe name for `item`.
pub fn recycling_recipe_name(item: &str) -> String {
    format!("{item}-recycling")
}

/// Per-machine recirculated (far-belt) intake rate for a voider bank —
/// `fraction * gross / machines`. `machines` recyclers physically placed
/// jointly consume `gross = rate / (1 - fraction)` items/s, of which
/// `fraction * gross` comes back around the recirc loop (the rest,
/// `rate`, is the external bus tap). Distributing evenly across the
/// placed machine count gives each recycler's own far-belt inserter
/// rate — matches `templates::self_loop_row`'s per-machine calling
/// convention (total = per-machine rate * machine count).
pub fn far_rate_per_machine(sizing: &VoiderSizing, rate: f64) -> f64 {
    let gross = rate / (1.0 - sizing.fraction);
    let far_total = sizing.fraction * gross;
    far_total / sizing.machines as f64
}

/// Snap-guard a float machine count to the nearest integer when within
/// float drift of one (mirrors `netflow.rs`'s snap-guard convention for
/// LP-derived counts), else ceil. Minimum 1.
fn snap_machine_count(raw: f64) -> usize {
    let snapped = if (raw - raw.round()).abs() < 1e-6 { raw.round() } else { raw.ceil() };
    (snapped as usize).max(1)
}

/// Clone `solver_result`, remove every solid surplus item that resolves
/// to a self-voider from `surplus_outputs`, and append a synthesized
/// `MachineSpec` (marked `voider: true`) + dependency-order entry for
/// each. Non-self-voider solid surplus is left untouched in
/// `surplus_outputs`, so it keeps following the ordinary
/// `SurplusPolicy::Export` path (`ghost_router` Step 7b, `lane_planner`
/// surplus handling — neither is touched by this function).
///
/// Emits `TraceEvent::VoiderSynthesized` per synthesized bank and
/// `TraceEvent::VoiderFallbackExport` per unresolved solid surplus item,
/// so nothing is silently dropped.
///
/// Fluid surplus is never touched — recycling takes items only.
pub fn synthesize_voiders(solver_result: &SolverResult) -> SolverResult {
    let mut out = solver_result.clone();

    let solid_surplus: Vec<ItemFlow> = solver_result
        .surplus_outputs
        .iter()
        .filter(|f| !f.is_fluid)
        .cloned()
        .collect();

    for f in &solid_surplus {
        match size_self_voider(&f.item, f.rate) {
            Some(sizing) => {
                let near_rate_per_machine = f.rate / sizing.machines as f64;
                let spec = MachineSpec {
                    entity: RECYCLER_ENTITY.to_string(),
                    recipe: recycling_recipe_name(&f.item),
                    count: sizing.machines as f64,
                    inputs: vec![ItemFlow {
                        item: f.item.clone(),
                        rate: near_rate_per_machine,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![],
                    self_loop: vec![],
                    voider: true,
                };
                out.dependency_order.push(spec.recipe.clone());
                out.machines.push(spec);
                out.surplus_outputs.retain(|s| s.item != f.item);

                crate::trace::emit(crate::trace::TraceEvent::VoiderSynthesized {
                    item: f.item.clone(),
                    rate: f.rate,
                    machines: sizing.machines,
                });
            }
            None => {
                let recipe_name = recycling_recipe_name(&f.item);
                let reason = if db().recipes.get(&recipe_name).is_none() {
                    format!("no `{recipe_name}` recipe in the recipe DB")
                } else {
                    format!(
                        "`{recipe_name}` isn't a pure self-voider (X -> fraction*X) shape \
                         — likely a multi-output cascade hop, not implemented in v1"
                    )
                };
                crate::trace::emit(crate::trace::TraceEvent::VoiderFallbackExport {
                    item: f.item.clone(),
                    reason,
                });
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uranium_238_is_a_self_voider() {
        let sizing = size_self_voider("uranium-238", 7.09)
            .expect("uranium-238-recycling should resolve as a self-voider");
        assert!((sizing.fraction - 0.25).abs() < 1e-9);
        // crafting_speed 0.5 / energy 0.03125 = 16 crafts/s = 16 items/s.
        assert!((sizing.per_recycler_rate - 16.0).abs() < 1e-9);
        // gross = 7.09 / 0.75 = 9.4533..; ceil(9.4533/16) = 1.
        assert_eq!(sizing.machines, 1);
    }

    #[test]
    fn steel_plate_is_a_self_voider() {
        let sizing = size_self_voider("steel-plate", 1.0)
            .expect("steel-plate-recycling should resolve as a self-voider");
        assert!((sizing.fraction - 0.25).abs() < 1e-9);
    }

    #[test]
    fn unknown_item_has_no_voider_recipe() {
        assert!(size_self_voider("totally-made-up-item", 1.0).is_none());
    }

    #[test]
    fn multi_output_cascade_is_not_a_self_voider() {
        // electronic-circuit-recycling: iron-plate + copper-cable, not X -> fraction*X.
        assert!(size_self_voider("electronic-circuit", 1.0).is_none());
    }

    #[test]
    fn synthesize_voiders_removes_self_voider_from_surplus_and_adds_machine() {
        let sr = SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![ItemFlow {
                item: "uranium-238".to_string(),
                rate: 7.09,
                is_fluid: false,
                module_id: 0,
            }],
            dependency_order: vec![],
        };
        let out = synthesize_voiders(&sr);
        assert!(out.surplus_outputs.iter().all(|f| f.item != "uranium-238"));
        let spec = out
            .machines
            .iter()
            .find(|m| m.voider)
            .expect("expected a synthesized voider MachineSpec");
        assert_eq!(spec.entity, "recycler");
        assert_eq!(spec.recipe, "uranium-238-recycling");
        assert_eq!(spec.count, 1.0);
        assert_eq!(spec.inputs.len(), 1);
        assert!((spec.inputs[0].rate - 7.09).abs() < 1e-9);
        assert!(spec.outputs.is_empty());
        assert!(out.dependency_order.contains(&"uranium-238-recycling".to_string()));
    }

    #[test]
    fn synthesize_voiders_leaves_unresolvable_surplus_untouched() {
        let sr = SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![ItemFlow {
                item: "totally-made-up-item".to_string(),
                rate: 3.0,
                is_fluid: false,
                module_id: 0,
            }],
            dependency_order: vec![],
        };
        let out = synthesize_voiders(&sr);
        assert_eq!(out.surplus_outputs.len(), 1);
        assert_eq!(out.surplus_outputs[0].item, "totally-made-up-item");
        assert!(out.machines.iter().all(|m| !m.voider));
    }

    #[test]
    fn synthesize_voiders_never_touches_fluid_surplus() {
        let sr = SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![ItemFlow {
                item: "petroleum-gas".to_string(),
                rate: 5.0,
                is_fluid: true,
                module_id: 0,
            }],
            dependency_order: vec![],
        };
        let out = synthesize_voiders(&sr);
        assert_eq!(out.surplus_outputs.len(), 1);
        assert!(out.machines.iter().all(|m| !m.voider));
    }
}
