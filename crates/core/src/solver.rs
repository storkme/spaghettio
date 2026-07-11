//! Recursive recipe solver: target item/rate → machine counts & flows.

use crate::models::{ItemFlow, MachineSpec, SolverResult};
use crate::recipe_db::{
    find_recipe_for_item_excluding, get_crafting_speed, machine_can_run_recipe,
    machine_for_recipe_with_palette, MachineIncompatibility, MachinePalette,
};
use rustc_hash::FxHashSet;

/// Marker prefix carried in `IncompatibleMachine` error strings across the
/// WASM boundary. The web sidebar splits on this to route the message to
/// the dedicated config-error banner instead of the generic solver-error
/// region. Keep in sync with `INCOMPATIBLE_MACHINE_PREFIX` in the web layer.
pub const INCOMPATIBLE_MACHINE_PREFIX: &str = "[INCOMPATIBLE_MACHINE] ";

#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    #[error("recipe {recipe} produces 0 of {item}")]
    ZeroProduct { recipe: String, item: String },
    #[error("no crafting speed for entity {entity}")]
    MissingCraftingSpeed { entity: String },
    /// Pre-flight rejection: the machine the palette resolved to can't run
    /// this recipe. The Display impl prefixes the message with
    /// [`INCOMPATIBLE_MACHINE_PREFIX`] so web callers can route it to the
    /// dedicated config-error banner.
    #[error("{}{machine} can't make {recipe}: {reason}", INCOMPATIBLE_MACHINE_PREFIX)]
    IncompatibleMachine {
        recipe: String,
        machine: String,
        reason: MachineIncompatibility,
    },
    /// The optimal plan uses a self-loop recipe (an item on both sides)
    /// outside v1's supported shapes (RFP Phase 2, "Cycle policy"; extended
    /// for the fluid-ingredient row variant): a fluid self-loop item
    /// (coal-liquefaction's heavy-oil — no template recirculates a fluid),
    /// more than one non-self-loop fluid ingredient, any non-self-loop
    /// fluid product, a non-self-loop fluid ingredient paired with the
    /// two-item (kovarex-shape) self-loop, more than two self-loop items,
    /// or — for exactly two self-loop items — same-sign net flow.
    /// Pure-solid self-loops (kovarex: U-235 +1/craft, U-238 −3/craft;
    /// bacteria cultivations: single net-positive item) and 1-item
    /// self-loops with a single fluid ingredient (pentapod-egg,
    /// fish-breeding — water alongside the net-positive solid self-loop
    /// item) solve via net flows instead of hitting this refusal.
    #[error("recipe {recipe} feeds its own output back as an ingredient — self-loop rows are not supported yet")]
    UnsupportedSelfLoop { recipe: String },
    /// The optimal plan contains a multi-recipe cycle (e.g. the
    /// carbon ↔ coal-synthesis loop). Cross-row feedback routing is out of
    /// scope for the net-flow RFP.
    #[error("recipes form a production cycle ({recipes}) — cyclic chains are not supported")]
    UnsupportedCycle { recipes: String },
    /// The LP itself failed (infeasible/unbounded/internal). Should not
    /// happen for well-formed inputs — external-supply eligibility
    /// guarantees feasibility — so this indicates a bug worth reporting.
    #[error("net-flow solve failed for {target}: {detail}")]
    LpFailed { target: String, detail: String },
}

struct SolveState {
    machines: Vec<MachineSpec>,
    external_inputs: Vec<ItemFlow>, // keep insertion order, small N
    dependency_order: Vec<String>,
    resolving: FxHashSet<String>,
}

impl SolveState {
    fn add_external(&mut self, item: &str, rate: f64, is_fluid: bool) {
        if let Some(flow) = self.external_inputs.iter_mut().find(|f| f.item == item) {
            flow.rate += rate;
            flow.is_fluid = is_fluid;
        } else {
            self.external_inputs.push(ItemFlow {
                item: item.to_string(),
                rate,
                is_fluid,
                module_id: 0,
            });
        }
    }
}

/// Compute machines needed to produce `target_item` at `target_rate` items/sec.
///
/// Recursively resolves intermediate recipes until hitting items in
/// `available_inputs` (which the caller must supply externally).
pub fn solve(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    machine_entity: &str,
) -> Result<SolverResult, SolverError> {
    solve_with_palette_and_exclusions(
        target_item,
        target_rate,
        available_inputs,
        &MachinePalette::default(),
        machine_entity,
        &FxHashSet::default(),
    )
}

/// Like [`solve`] but consults a per-category [`MachinePalette`] before
/// falling back to the hardcoded category mapping and `default_machine`.
pub fn solve_with_palette(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
) -> Result<SolverResult, SolverError> {
    solve_with_palette_and_exclusions(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        &FxHashSet::default(),
    )
}

/// Like [`solve`] but skips recipes listed in `excluded_recipes`.
///
/// Useful when several recipes produce the same item and the caller wants to
/// steer the solver away from some of them (e.g. exclude `coal-liquefaction`
/// to avoid pulling in the whole oil chain for `plastic-bar`).
pub fn solve_with_exclusions(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    machine_entity: &str,
    excluded_recipes: &FxHashSet<String>,
) -> Result<SolverResult, SolverError> {
    solve_with_palette_and_exclusions(
        target_item,
        target_rate,
        available_inputs,
        &MachinePalette::default(),
        machine_entity,
        excluded_recipes,
    )
}

/// Combined variant: per-category palette + recipe exclusions.
///
/// Since Phase 1 of docs/rfp-solver-net-flow.md this routes through the
/// net-flow LP in **compatibility mode**: the legacy tree walk runs first
/// to pick the recipe set (JSON-first per item, exclusions honored), then
/// the LP re-derives flows over exactly that set — fixing byproduct
/// crediting and fleet double-counting without changing recipe selection.
/// Cycle-shaped selections return typed errors instead of the walk's
/// silent nonsense externals.
///
/// Phase 3 (docs/rfp-solver-net-flow.md): free cost-based recipe
/// selection is the default. All non-excluded recipes are candidate LP
/// columns; the frozen cost table picks the mix — raw-input efficiency
/// first, so e.g. advanced-oil-processing + cracking replaces
/// basic-oil-processing wherever byproducts can be credited, typically
/// with zero surplus. Byproduct surplus and fluid targets route to the
/// layout perimeter (Phase 2). Unsupported cycles return typed errors.
pub fn solve_with_palette_and_exclusions(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
) -> Result<SolverResult, SolverError> {
    solve_free_with_palette_and_exclusions(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
    )
}

/// Compatibility mode (Phase 1 behavior): the legacy tree walk picks the
/// recipe set (JSON-first per item), then the LP re-derives flows over
/// exactly that set. Kept for A/B comparison and the parity harness.
pub fn solve_compat_with_palette_and_exclusions(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
) -> Result<SolverResult, SolverError> {
    let walk = solve_tree_walk_with_palette_and_exclusions(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
    )?;
    let recipe_set: FxHashSet<String> = walk.dependency_order.iter().cloned().collect();
    crate::netflow::solve_netflow(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        crate::netflow::RecipeScope::Restricted(&recipe_set),
        &crate::netflow::CostTable::default(),
    )
}

/// Phase 3 free cost-based recipe selection: all non-excluded recipes are
/// candidate columns and the frozen cost table picks the mix (raw-input
/// efficiency first — e.g. advanced-oil-processing + cracking replaces
/// basic-oil-processing wherever byproducts can be credited, typically
/// with zero surplus). Solver-level behavior is fully verified (parity
/// harness); the LAYOUT of dense oil complexes still has a known fluid-
/// lane stagger gap, so this is opt-in until that closes.
pub fn solve_free_with_palette_and_exclusions(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
) -> Result<SolverResult, SolverError> {
    crate::netflow::solve_netflow(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        crate::netflow::RecipeScope::Free,
        &crate::netflow::CostTable::default(),
    )
}

/// The legacy recursive tree walk. Kept as the recipe-*selection* oracle
/// for compatibility mode and as the parity-harness reference. Known-wrong
/// flow accounting (no byproduct crediting, `resolving`-guard cycle punts,
/// fleet double-counting) — do not add new callers; use [`solve`] /
/// [`solve_with_palette_and_exclusions`] instead.
pub fn solve_tree_walk_with_palette_and_exclusions(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
) -> Result<SolverResult, SolverError> {
    let mut state = SolveState {
        machines: Vec::new(),
        external_inputs: Vec::new(),
        dependency_order: Vec::new(),
        resolving: FxHashSet::default(),
    };

    resolve(
        target_item,
        target_rate,
        false,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        &mut state,
    )?;

    Ok(SolverResult {
        machines: state.machines,
        external_inputs: state.external_inputs,
        external_outputs: vec![ItemFlow {
            item: target_item.to_string(),
            rate: target_rate,
            is_fluid: false,
            module_id: 0,
        }],
        surplus_outputs: Vec::new(),
        dependency_order: state.dependency_order,
    })
}

#[allow(clippy::too_many_arguments)]
fn resolve(
    item: &str,
    rate: f64,
    is_fluid: bool,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    state: &mut SolveState,
) -> Result<(), SolverError> {
    if available_inputs.contains(item) {
        state.add_external(item, rate, is_fluid);
        return Ok(());
    }

    let recipe = match find_recipe_for_item_excluding(item, excluded_recipes) {
        Some(r) => r,
        None => {
            state.add_external(item, rate, is_fluid);
            return Ok(());
        }
    };

    if state.resolving.contains(item) {
        state.add_external(item, rate, is_fluid);
        return Ok(());
    }

    state.resolving.insert(item.to_string());

    let entity = machine_for_recipe_with_palette(recipe, palette, default_machine);
    if let Err(reason) = machine_can_run_recipe(&entity, recipe) {
        return Err(SolverError::IncompatibleMachine {
            recipe: recipe.name.clone(),
            machine: entity.clone(),
            reason,
        });
    }
    let crafting_speed = get_crafting_speed(&entity);
    if crafting_speed <= 0.0 {
        return Err(SolverError::MissingCraftingSpeed {
            entity: entity.clone(),
        });
    }

    let products_per_craft: f64 = recipe
        .products
        .iter()
        .filter(|p| p.name == item)
        .map(|p| p.amount * p.probability)
        .sum();

    if products_per_craft <= 0.0 {
        return Err(SolverError::ZeroProduct {
            recipe: recipe.name.clone(),
            item: item.to_string(),
        });
    }

    let crafts_per_sec = crafting_speed / recipe.energy;
    let items_per_sec_per_machine = crafts_per_sec * products_per_craft;
    let count = rate / items_per_sec_per_machine;

    let input_flows: Vec<ItemFlow> = recipe
        .ingredients
        .iter()
        .map(|ing| ItemFlow {
            item: ing.name.clone(),
            rate: ing.amount * crafts_per_sec,
            is_fluid: ing.type_ == "fluid",
            module_id: 0,
        })
        .collect();

    let output_flows: Vec<ItemFlow> = recipe
        .products
        .iter()
        .map(|prod| ItemFlow {
            item: prod.name.clone(),
            rate: prod.amount * prod.probability * crafts_per_sec,
            is_fluid: prod.type_ == "fluid",
            module_id: 0,
        })
        .collect();

    if let Some(existing) = state.machines.iter_mut().find(|m| m.recipe == recipe.name) {
        existing.count += count;
    } else {
        state.machines.push(MachineSpec {
            entity,
            recipe: recipe.name.clone(),
            self_loop: vec![], voider: false,
            count,
            inputs: input_flows,
            outputs: output_flows,
        });
        state.dependency_order.push(recipe.name.clone());
    }

    for ing in &recipe.ingredients {
        let ingredient_rate = ing.amount * crafts_per_sec * count;
        let ing_is_fluid = ing.type_ == "fluid";
        resolve(
            &ing.name,
            ingredient_rate,
            ing_is_fluid,
            available_inputs,
            palette,
            default_machine,
            excluded_recipes,
            state,
        )?;
    }

    state.resolving.remove(item);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs_of(items: &[&str]) -> FxHashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn solves_iron_gear_wheel() {
        let available = inputs_of(&["iron-plate"]);
        let result = solve("iron-gear-wheel", 10.0, &available, "assembling-machine-3").unwrap();

        assert_eq!(result.machines.len(), 1);
        let m = &result.machines[0];
        assert_eq!(m.recipe, "iron-gear-wheel");
        // asm3 crafting_speed=1.25, recipe energy=0.5 → 2.5 crafts/s/machine
        // 10/s target ÷ 2.5 = 4.0 machines
        assert!(
            (m.count - 4.0).abs() < 0.01,
            "expected count ≈ 4.0, got {}",
            m.count
        );

        let iron = result
            .external_inputs
            .iter()
            .find(|f| f.item == "iron-plate")
            .expect("iron-plate in external inputs");
        assert!(
            (iron.rate - 20.0).abs() < 0.01,
            "expected iron-plate rate ≈ 20.0, got {}",
            iron.rate
        );

        assert_eq!(result.external_outputs.len(), 1);
        assert_eq!(result.external_outputs[0].item, "iron-gear-wheel");
        assert_eq!(result.external_outputs[0].rate, 10.0);
    }

    #[test]
    fn am1_palette_for_advanced_circuit_returns_incompatible_error() {
        // advanced-circuit has 3 ingredients in `electronics` category. Pin
        // electronics → AM1 in the palette and expect a typed
        // IncompatibleMachine error rather than a silent half-broken layout.
        let available = inputs_of(&[
            "iron-plate",
            "copper-plate",
            "plastic-bar",
            "electronic-circuit",
        ]);
        let mut palette = MachinePalette::default();
        palette
            .by_category
            .insert("electronics".into(), "assembling-machine-1".into());
        let err = solve_with_palette(
            "advanced-circuit",
            1.0,
            &available,
            &palette,
            "assembling-machine-3",
        )
        .expect_err("AM1 should be rejected for advanced-circuit");
        match err {
            SolverError::IncompatibleMachine { machine, reason, .. } => {
                assert_eq!(machine, "assembling-machine-1");
                assert!(matches!(
                    reason,
                    MachineIncompatibility::TooManyIngredients { limit: 2, .. }
                ));
            }
            other => panic!("expected IncompatibleMachine, got {other:?}"),
        }
    }

    #[test]
    fn incompatible_machine_error_message_carries_marker_prefix() {
        // The web layer relies on the marker prefix to route this error to
        // the dedicated config-error banner. Lock the contract.
        let err = SolverError::IncompatibleMachine {
            recipe: "advanced-circuit".into(),
            machine: "assembling-machine-1".into(),
            reason: MachineIncompatibility::TooManyIngredients { limit: 2, got: 3 },
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with(INCOMPATIBLE_MACHINE_PREFIX),
            "expected leading marker, got: {msg}"
        );
    }

    #[test]
    fn palette_overrides_electronics_machine_end_to_end() {
        // electronic-circuit and copper-cable are both `electronics` category
        // (a fall-through, not hardcoded). With palette {electronics: AM1},
        // both production steps should land on AM1 regardless of `default`.
        let available = inputs_of(&["iron-plate", "copper-plate"]);
        let mut palette = MachinePalette::default();
        palette
            .by_category
            .insert("electronics".into(), "assembling-machine-1".into());
        let result = solve_with_palette(
            "electronic-circuit",
            5.0,
            &available,
            &palette,
            "assembling-machine-3",
        )
        .expect("solver runs");

        assert!(!result.machines.is_empty());
        for m in &result.machines {
            assert_eq!(
                m.entity, "assembling-machine-1",
                "recipe {} ended up on {}, expected AM1",
                m.recipe, m.entity
            );
        }
    }
}
