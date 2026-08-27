//! Solver public API: target item/rate → machine counts & flows. Every
//! entry point routes to the net-flow LP in `netflow.rs` (default since
//! Phase 3, 2026-07 — see `docs/rfc-solver-net-flow.md`). The legacy
//! recursive tree walk that used to live here (and its compat-mode A/B
//! path) was deleted 2026-08-14 (#632 A1) once the parity suite ran green
//! one final time — see the RFC's decision log for the deletion receipt.

use crate::common::needs_electricity;
use crate::models::SolverResult;
use crate::recipe_db::{
    category_has_electric_machine, db, is_excluded_recipe, MachineIncompatibility, MachinePalette,
};
use crate::trace::{self, TraceEvent};
use rustc_hash::FxHashSet;

/// Marker prefix carried in `IncompatibleMachine` error strings across the
/// WASM boundary. The web sidebar splits on this to route the message to
/// the dedicated config-error banner instead of the generic solver-error
/// region. Keep in sync with `INCOMPATIBLE_MACHINE_PREFIX` in the web layer.
pub const INCOMPATIBLE_MACHINE_PREFIX: &str = "[INCOMPATIBLE_MACHINE] ";

#[derive(Debug, thiserror::Error)]
pub enum SolverError {
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
    /// outside v1's supported shapes (RFC Phase 2, "Cycle policy"; extended
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
    /// scope for the net-flow RFC.
    #[error("recipes form a production cycle ({recipes}) — cyclic chains are not supported")]
    UnsupportedCycle { recipes: String },
    /// The LP itself failed (infeasible/unbounded/internal). Should not
    /// happen for well-formed inputs — external-supply eligibility
    /// guarantees feasibility — so this indicates a bug worth reporting.
    #[error("net-flow solve failed for {target}: {detail}")]
    LpFailed { target: String, detail: String },
}

/// #461 part (a)'s production-path fix. The engine cannot fuel a burner
/// (fuel category `nutrients` and no engine mechanism delivers it), so a
/// recipe that lands on one is a last resort — but pushing that preference
/// INTO the net-flow LP's cost model (an earlier version of this fix,
/// this PR's history) moved 11 calibration fixtures' manifest hashes by
/// float noise: any coefficient change on any column, even one that never
/// wins, can shift the simplex's floating-point path for the WHOLE shared
/// LP instance, and pure-`organic` recipes (`bioplastic` → plastic-bar,
/// `biosulfur` → sulfur, …) sit in the demand closure of far more fixtures
/// than #461 is about. Doing it here instead — strictly AFTER the LP has
/// already run, as a policy decision over its OUTPUT — keeps the LP for
/// every solve that never chose a burner bit-identical to before this fix
/// existed: this function is a no-op unless `result` actually contains a
/// burner machine.
///
/// Called by every real solver entry point (`solve`/`solve_with_palette`/
/// `solve_with_exclusions` via [`solve_free_with_palette_and_exclusions`];
/// the wasm-facing quality/module family via
/// [`solve_with_palette_exclusions_quality_and_modules`]; the multi-target
/// family via
/// [`solve_multi_with_palette_exclusions_quality_and_modules`]) right
/// after its own netflow call succeeds, via `resolve` — a closure that
/// re-invokes that SAME netflow call with an expanded exclusion set, so
/// the actual decision logic lives here exactly once rather than being
/// copied at each entry point.
///
/// Rule: for every machine in `result` where [`needs_electricity`] is
/// false, look up its recipe and ask whether ANY of its products has at
/// least one OTHER (non-excluded) recipe whose category
/// [`category_has_electric_machine`] says can run electric. If so, that
/// burner recipe is a genuine last resort — not the only way to make this
/// item — so it's added to the exclusion set. If NO burner recipe in the
/// result clears that bar (e.g. `pentapod-egg`, biochamber-only with no
/// assembler-tier alternative anywhere in the recipe graph), nothing is
/// excluded and `result` is returned untouched: no re-solve is attempted.
///
/// `resolve` is then called exactly ONCE with the expanded exclusion set.
/// The re-solve is only WORTH taking if it actually gets rid of the
/// burner(s): a second plan that still contains any `!needs_electricity`
/// machine is strictly worse than the first (more machines, still
/// unfuelled — #461 part (b)'s `burner-fuel` validator check makes that
/// loud either way, so there is nothing to gain by swapping one unfuelled
/// plan for a bigger one). So the re-solve's result is accepted ONLY when
/// it both succeeds AND comes back fully burner-free; otherwise the
/// ORIGINAL result is returned untouched. A trace event fires whenever a
/// re-solve is ATTEMPTED (naming the excluded recipes), with `accepted`
/// carrying which of the two results actually got used — see
/// [`crate::trace::TraceEvent::BurnerRecipeExcluded`].
fn avoid_burner_recipes(
    result: SolverResult,
    target_item: &str,
    excluded_recipes: &FxHashSet<String>,
    resolve: impl FnOnce(&FxHashSet<String>) -> Result<SolverResult, SolverError>,
) -> SolverResult {
    let mut expanded: Option<FxHashSet<String>> = None;
    for m in &result.machines {
        if needs_electricity(&m.entity) {
            continue;
        }
        let Some(recipe) = db().recipes.get(&m.recipe) else {
            continue;
        };
        let has_electric_alternative = recipe.products.iter().any(|product| {
            db().recipes.iter().any(|(other_name, other)| {
                other_name != &m.recipe
                    && !excluded_recipes.contains(other_name)
                    && !is_excluded_recipe(other)
                    && other.products.iter().any(|p| p.name == product.name)
                    && category_has_electric_machine(&other.category)
            })
        });
        if has_electric_alternative {
            expanded
                .get_or_insert_with(|| excluded_recipes.clone())
                .insert(m.recipe.clone());
        }
    }
    let Some(expanded) = expanded else {
        return result;
    };
    let mut newly_excluded: Vec<String> =
        expanded.difference(excluded_recipes).cloned().collect();
    newly_excluded.sort();
    // Accept the re-solve only when it succeeded AND is itself burner-free
    // — a re-solve that still contains a burner is strictly worse (more
    // machines, still unfuelled) than the original, so it's discarded.
    let accepted_result = resolve(&expanded)
        .ok()
        .filter(|r| r.machines.iter().all(|m| needs_electricity(&m.entity)));
    trace::emit(TraceEvent::BurnerRecipeExcluded {
        target_item: target_item.to_string(),
        excluded_recipes: newly_excluded,
        accepted: accepted_result.is_some(),
    });
    accepted_result.unwrap_or(result)
}

/// Compute machines needed to produce `target_item` at `target_rate` items/sec.
///
/// Solves via the net-flow LP with free cost-based recipe selection (the
/// default since Phase 3, 2026-07 — see `docs/rfc-solver-net-flow.md`);
/// items in `available_inputs` are treated as externally supplied.
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
/// steer the solver away from some of them (e.g. exclude
/// `basic-oil-processing` to force the advanced-oil-processing + cracking
/// chain for `plastic-bar`).
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
/// Routes through the net-flow LP with free cost-based recipe selection
/// (docs/rfc-solver-net-flow.md Phase 3, the default since 2026-07). All
/// non-excluded recipes are candidate LP columns; the frozen cost table
/// picks the mix — raw-input efficiency first, so e.g.
/// advanced-oil-processing + cracking replaces basic-oil-processing
/// wherever byproducts can be credited, typically with zero surplus.
/// Byproduct surplus and fluid targets route to the layout perimeter
/// (Phase 2). Unsupported cycles return typed errors.
///
/// (Phase 1's compatibility mode — restricting the LP to the recipe set a
/// legacy recursive tree walk would have picked — was removed 2026-08-14
/// (#632 A1) once the parity suite proved free-mode selection matches;
/// see the RFC's decision log.)
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

/// Phase 3 free cost-based recipe selection: all non-excluded recipes are
/// candidate columns and the frozen cost table picks the mix (raw-input
/// efficiency first — e.g. advanced-oil-processing + cracking replaces
/// basic-oil-processing wherever byproducts can be credited, typically
/// with zero surplus). This is the default path every public entry point
/// routes through — the only path, since #632 A1 deleted the compat
/// (tree-walk-selected) A/B mode. The LAYOUT of dense oil complexes still
/// has a known fluid-lane stagger gap.
pub fn solve_free_with_palette_and_exclusions(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
) -> Result<SolverResult, SolverError> {
    let result = crate::netflow::solve_netflow(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        crate::netflow::RecipeScope::Free,
        &crate::netflow::CostTable::default(),
    )?;
    Ok(avoid_burner_recipes(result, target_item, excluded_recipes, |excl| {
        crate::netflow::solve_netflow(
            target_item,
            target_rate,
            available_inputs,
            palette,
            default_machine,
            excl,
            crate::netflow::RecipeScope::Free,
            &crate::netflow::CostTable::default(),
        )
    }))
}

/// Like [`solve_with_palette_and_exclusions`] with a build-quality tier
/// (`docs/rfc-build-quality.md` Phase 1): machine counts shrink by the
/// quality crafting-speed multiplier. `Normal` is bit-identical to the
/// plain entry points (same code path — the multiplier rides through
/// `NetflowOptions`, whose default is `Normal`).
pub fn solve_with_palette_exclusions_and_quality(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    quality: crate::common::QualityTier,
) -> Result<SolverResult, SolverError> {
    solve_with_palette_exclusions_quality_and_modules(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        quality,
        crate::module_policy::ModulePolicy::default(),
    )
}

/// Like [`solve_with_palette_exclusions_and_quality`] with a global
/// module policy (RFC-044 Phase 3): eligible machines are planned with
/// full loadouts — speed/productivity factors flow through machine
/// counts and rates, and the loadout rides `MachineSpec::game_modules`
/// to the layout stamp pass. `ModulePolicy::default()` (kind `None`) is
/// bit-identical to the plain entry points (KC1 — the no-op path
/// multiplies by exactly 1.0 and rewrites nothing).
#[allow(clippy::too_many_arguments)]
pub fn solve_with_palette_exclusions_quality_and_modules(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    quality: crate::common::QualityTier,
    module_policy: crate::module_policy::ModulePolicy,
) -> Result<SolverResult, SolverError> {
    let options =
        crate::netflow::NetflowOptions { quality, module_policy, ..Default::default() };
    let result = crate::netflow::solve_netflow_with_options(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        crate::netflow::RecipeScope::Free,
        &crate::netflow::CostTable::default(),
        &options,
    )?;
    Ok(avoid_burner_recipes(result, target_item, excluded_recipes, |excl| {
        crate::netflow::solve_netflow_with_options(
            target_item,
            target_rate,
            available_inputs,
            palette,
            default_machine,
            excl,
            crate::netflow::RecipeScope::Free,
            &crate::netflow::CostTable::default(),
            &options,
        )
    }))
}

/// RFC-062 Phase 3: the multi-target counterpart of
/// [`solve_with_palette_exclusions_quality_and_modules`], for callers that
/// need N >= 1 simultaneous targets at this richness (palette, exclusions,
/// quality, modules) rather than the bare `targets: &[(String, f64)]`
/// entry points in `netflow.rs`. Choke point for the wasm `solve_multi`
/// boundary, so it gets the same recipe-selection/quality/module
/// behavior the scalar wasm `solve` already has.
///
/// NOTE `sim_export` no longer routes through here: it needs to declare a
/// research-productivity axis this signature cannot carry, so it calls
/// `netflow::solve_netflow_multi_with_options` directly with identical values
/// for every other field. That makes two `NetflowOptions` construction sites
/// where this was meant to be one — the honest fix is to give this wrapper
/// the field and bring `sim_export` back, which is a follow-up rather than a
/// silent divergence (PR #591 review) — and, by the same
/// one-element-slice construction `netflow.rs`'s Phase 1 established, N=1
/// here is bit-identical to [`solve_with_palette_exclusions_quality_and_modules`]
/// (that function is now a thin `targets: &[(target_item.to_string(),
/// target_rate)]` call into [`crate::netflow::solve_netflow_with_options`],
/// which itself forwards to the same multi entry point this calls).
///
/// Deliberately does NOT validate `targets` (empty list, non-positive
/// rates, unknown items) — same posture as every other entry point in this
/// file. Callers that need typed, pre-flight validation for a
/// user/API-facing boundary (e.g. the wasm `solve_multi` binding) do it at
/// their own boundary, not here: this module's job is to solve a
/// well-formed request, not to be every caller's input sanitizer, and an
/// internal caller (an already-validated URL, an example script's CLI
/// parse) shouldn't pay for a second validation pass it doesn't need.
#[allow(clippy::too_many_arguments)]
pub fn solve_multi_with_palette_exclusions_quality_and_modules(
    targets: &[(String, f64)],
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    quality: crate::common::QualityTier,
    module_policy: crate::module_policy::ModulePolicy,
) -> Result<SolverResult, SolverError> {
    let options =
        crate::netflow::NetflowOptions { quality, module_policy, ..Default::default() };
    let result = crate::netflow::solve_netflow_multi_with_options(
        targets,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        crate::netflow::RecipeScope::Free,
        &crate::netflow::CostTable::default(),
        &options,
    )?;
    // `target_item` is a trace-only label (not part of the returned
    // `SolverResult`, so this has no bearing on the N=1 bit-identity
    // invariant above) — join every requested item, same convention as
    // netflow.rs's own `target_label`.
    let label = targets.iter().map(|(item, _)| item.as_str()).collect::<Vec<_>>().join("+");
    Ok(avoid_burner_recipes(result, &label, excluded_recipes, |excl| {
        crate::netflow::solve_netflow_multi_with_options(
            targets,
            available_inputs,
            palette,
            default_machine,
            excl,
            crate::netflow::RecipeScope::Free,
            &crate::netflow::CostTable::default(),
            &options,
        )
    }))
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

    /// Kill criterion 2a (rfc-build-quality): the quality entry point at
    /// `Normal` must be bit-identical to the plain entry point — same code
    /// path, `×1.0` multiplier. Swept across rates adjacent to
    /// whole-machine boundaries (EC on AM3: 2.5/s per machine), where any
    /// rounding drift in the count math would surface first.
    #[test]
    fn quality_identity_at_normal_boundary_sweep() {
        use crate::common::QualityTier;
        let available = inputs_of(&["iron-plate", "copper-plate"]);
        let boundaries = [2.5, 5.0, 45.0, 60.0];
        let eps = [0.0, 1e-9, -1e-9, 1e-3, -1e-3];
        for b in boundaries {
            for e in eps {
                let rate = b + e;
                if rate <= 0.0 {
                    continue;
                }
                let plain =
                    solve("electronic-circuit", rate, &available, "assembling-machine-3").unwrap();
                let quality = solve_with_palette_exclusions_and_quality(
                    "electronic-circuit",
                    rate,
                    &available,
                    &MachinePalette::default(),
                    "assembling-machine-3",
                    &FxHashSet::default(),
                    QualityTier::Normal,
                )
                .unwrap();
                assert_eq!(plain.machines.len(), quality.machines.len(), "rate {rate}");
                for (p, q) in plain.machines.iter().zip(quality.machines.iter()) {
                    assert_eq!(p.recipe, q.recipe, "rate {rate}");
                    assert_eq!(p.entity, q.entity, "rate {rate}");
                    assert_eq!(
                        p.count.to_bits(),
                        q.count.to_bits(),
                        "rate {rate} recipe {}: {} vs {} not bit-identical",
                        p.recipe,
                        p.count,
                        q.count
                    );
                }
            }
        }
    }

    /// Per-tier machine counts on the RFC's hand-computed cases:
    /// EC@60/s on AM3 (Normal 2.5/s → 24 machines; Legendary 6.25/s →
    /// 9.6) with cable scaling alongside (Normal 5/s → 36; Legendary
    /// 12.5/s → 14.4), and iron smelting on electric furnaces (Normal
    /// 0.625/s → 96; Legendary 1.5625/s → 38.4).
    #[test]
    fn quality_scales_machine_counts() {
        use crate::common::QualityTier;

        let count_of = |result: &SolverResult, recipe: &str| -> f64 {
            result
                .machines
                .iter()
                .find(|m| m.recipe == recipe)
                .unwrap_or_else(|| panic!("no {recipe} machines"))
                .count
        };
        let solve_q = |item: &str, rate: f64, inputs: &FxHashSet<String>, machine: &str, q| {
            solve_with_palette_exclusions_and_quality(
                item,
                rate,
                inputs,
                &MachinePalette::default(),
                machine,
                &FxHashSet::default(),
                q,
            )
            .unwrap()
        };

        let ec_inputs = inputs_of(&["iron-plate", "copper-plate"]);
        for (tier, ec_expected, cable_expected) in [
            (QualityTier::Normal, 24.0, 36.0),
            (QualityTier::Uncommon, 24.0 / 1.3, 36.0 / 1.3),
            (QualityTier::Rare, 24.0 / 1.6, 36.0 / 1.6),
            (QualityTier::Epic, 24.0 / 1.9, 36.0 / 1.9),
            (QualityTier::Legendary, 9.6, 14.4),
        ] {
            let r = solve_q("electronic-circuit", 60.0, &ec_inputs, "assembling-machine-3", tier);
            let ec = count_of(&r, "electronic-circuit");
            let cable = count_of(&r, "copper-cable");
            assert!(
                (ec - ec_expected).abs() < 1e-9,
                "{tier:?}: EC count {ec} vs {ec_expected}"
            );
            assert!(
                (cable - cable_expected).abs() < 1e-9,
                "{tier:?}: cable count {cable} vs {cable_expected}"
            );
        }

        let ore_inputs = inputs_of(&["iron-ore"]);
        let normal = solve_q("iron-plate", 60.0, &ore_inputs, "electric-furnace", QualityTier::Normal);
        let legendary =
            solve_q("iron-plate", 60.0, &ore_inputs, "electric-furnace", QualityTier::Legendary);
        assert!((count_of(&normal, "iron-plate") - 96.0).abs() < 1e-9);
        assert!((count_of(&legendary, "iron-plate") - 38.4).abs() < 1e-9);
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

    // ── Direct-insertion coupling detection (RFC decomposition-search
    //    Phase 3) ──────────────────────────────────────────────────────

    /// Helper: find a coupling by (producer, consumer, item).
    fn find_coupling<'a>(
        result: &'a SolverResult,
        producer: &str,
        consumer: &str,
        item: &str,
    ) -> Option<&'a crate::models::DICoupling> {
        result.di_couplings.iter().find(|c| {
            c.producer_recipe == producer
                && c.consumer_recipe == consumer
                && c.item == item
        })
    }

    /// The canonical DI pair: copper-cable → electronic-circuit. When
    /// solving EC from plates, cable is produced internally and consumed
    /// solely by EC — exactly one producer, one consumer, no surplus.
    #[test]
    fn di_cable_to_ec_detected() {
        let available = inputs_of(&["iron-plate", "copper-plate"]);
        let result = solve("electronic-circuit", 10.0, &available, "assembling-machine-3")
            .unwrap();
        let c = find_coupling(&result, "copper-cable", "electronic-circuit", "copper-cable")
            .expect("cable→EC DI coupling should be detected");
        assert!(c.producer_count > 0.0, "producer count should be positive");
        assert!(c.consumer_count > 0.0, "consumer count should be positive");
        // The ratio depends on recipe amounts + crafting speeds; just
        // check it matches what the machine specs say.
        let cable = result.machines.iter().find(|m| m.recipe == "copper-cable").unwrap();
        let ec = result.machines.iter().find(|m| m.recipe == "electronic-circuit").unwrap();
        assert!((c.producer_count - cable.count).abs() < 0.01);
        assert!((c.consumer_count - ec.count).abs() < 0.01);
    }

    /// When the intermediate IS the target, it has no internal consumer →
    /// no coupling. Solving for copper-cable directly means cable is the
    /// external output, not consumed by anything internally.
    #[test]
    fn di_no_coupling_for_target_item() {
        let available = inputs_of(&["copper-plate"]);
        let result = solve("copper-cable", 10.0, &available, "assembling-machine-3").unwrap();
        assert!(
            result.di_couplings.is_empty(),
            "target item should not produce DI couplings, got {:?}",
            result.di_couplings
        );
    }

    /// Solving inserter from plates needs three internally-produced
    /// intermediates: copper-cable (→ EC), iron-gear-wheel (→ inserter),
    /// and electronic-circuit (→ inserter). Each has exactly one producer
    /// and one consumer — all three should be detected.
    #[test]
    fn di_multiple_couplings_detected() {
        let available = inputs_of(&["iron-plate", "copper-plate"]);
        let result = solve("inserter", 10.0, &available, "assembling-machine-3").unwrap();
        assert!(
            find_coupling(&result, "copper-cable", "electronic-circuit", "copper-cable")
                .is_some(),
            "cable→EC coupling missing"
        );
        assert!(
            find_coupling(&result, "iron-gear-wheel", "inserter", "iron-gear-wheel")
                .is_some(),
            "gear→inserter coupling missing"
        );
        assert!(
            find_coupling(&result, "electronic-circuit", "inserter", "electronic-circuit")
                .is_some(),
            "EC→inserter coupling missing"
        );
    }

    /// The full pipeline: solving EC from plates with
    /// `direct_insertion: true` produces a DI coupling for cable→EC, and
    /// `order_specs` co-locates them (cable immediately before EC, no
    /// other recipe between them).
    #[test]
    fn di_order_specs_co_locates_cable_and_ec() {
        use crate::bus::placer::order_specs;
        let available = inputs_of(&["iron-plate", "copper-plate"]);
        let result = solve("electronic-circuit", 10.0, &available, "assembling-machine-3")
            .unwrap();
        assert!(
            result
                .di_couplings
                .iter()
                .any(|c| c.producer_recipe == "copper-cable"
                    && c.consumer_recipe == "electronic-circuit"),
            "solver should detect cable→EC coupling"
        );
        let ordered = order_specs(&result.machines, &result.dependency_order, &result.di_couplings);
        let recipes: Vec<&str> = ordered.iter().map(|m| m.recipe.as_str()).collect();
        let cc = recipes.iter().position(|&r| r == "copper-cable");
        let ec = recipes.iter().position(|&r| r == "electronic-circuit");
        assert!(cc.is_some() && ec.is_some(), "both cable and EC should be present");
        assert_eq!(
            ec.unwrap(),
            cc.unwrap() + 1,
            "EC should be immediately after cable (DI co-location)"
        );
    }

    /// Fluid intermediates are excluded from DI (inserter DI only; pipe
    /// adjacency is a separate concern). Solving plastic-bar from crude
    /// has petroleum-gas as an intermediate — if it's a single-consumer
    /// pair, it should NOT be coupled because PG is a fluid.
    #[test]
    fn di_fluid_intermediate_not_coupled() {
        // Solve plastic-bar from petroleum-gas (fluid input).
        let available = inputs_of(&["coal", "petroleum-gas"]);
        let result = solve("plastic-bar", 10.0, &available, "assembling-machine-3").unwrap();
        // If any coupling exists, none should be for a fluid item.
        for c in &result.di_couplings {
            // plastic-bar's only intermediate is... well, plastic-bar
            // itself is the target. The coal→plastic-bar recipe uses PG
            // directly. If there's a coupling, check it's not a fluid.
            let recipe = crate::recipe_db::db()
                .recipes
                .get(c.producer_recipe.as_str());
            if let Some(r) = recipe {
                let is_fluid = r.products.iter().any(|p| p.name == c.item && p.type_ == "fluid");
                assert!(!is_fluid, "fluid item {} should not be DI-coupled", c.item);
            }
        }
    }
}
