//! Global module policy → per-machine loadouts and effect factors
//! (RFC-044 Phase 3).
//!
//! One policy applies to every machine whose (machine, recipe) pair is
//! eligible; ineligible pairs get NO modules (the productivity policy
//! falls back to empty, not to speed). The resolved
//! [`MachineModuleEffects`] is the single source consumed by the netflow
//! solver (speed factor at the column crafting-speed site; productivity
//! at the three result-scaling sites) and by the layout's stamping
//! post-pass (via `MachineSpec::game_modules`).
//!
//! Naming note: "module" in this file always means the GAME item
//! (speed/prod modules). Partition modules (`ItemFlow::module_id`) are a
//! different concept — new identifiers here use `game_module`/
//! `ModulePolicy` vocabulary, per the RFC-044 hazard note.

use crate::common::{module_effect, module_slots_known, QualityTier};
use crate::models::ModuleItem;
use crate::recipe_db::{db, Recipe};

/// Which module family the policy fills eligible machines with.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ModulePolicyKind {
    #[default]
    None,
    Speed,
    Productivity,
}

/// The user-facing global module policy: family, tier (1–3), and the
/// quality of the modules themselves.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ModulePolicy {
    pub kind: ModulePolicyKind,
    /// Module tier 1–3 (clamped; only meaningful when `kind != None`).
    pub tier: u8,
    /// Quality of the modules (scales beneficial effects only).
    pub quality: QualityTier,
}

/// Resolved per-(machine, recipe) module effects.
#[derive(Debug, Clone, PartialEq)]
pub struct MachineModuleEffects {
    /// `max(0.2, 1 + Σspeed)` — exactly `1.0` when no modules apply
    /// (KC1: the no-op path performs no float arithmetic).
    pub speed_multiplier: f64,
    /// Σ module productivity + machine `base_effect` productivity, both
    /// gated on the recipe's `allow_productivity`. `0.0` when none.
    pub prod_bonus: f64,
    /// The modules to stamp into placed machines (`PlacedEntity.items`).
    pub loadout: Vec<ModuleItem>,
}

impl MachineModuleEffects {
    /// The exact no-op: no modules, no factors.
    pub fn none() -> Self {
        MachineModuleEffects {
            speed_multiplier: 1.0,
            prod_bonus: 0.0,
            loadout: Vec::new(),
        }
    }
}

/// Quality scaling for a BENEFICIAL module effect: ×(1 + 0.3·level),
/// floored to 1% steps (the game's rule for non-quality modules —
/// RFC-044 game-rule model; wiki-anchored, not prototype data). The tiny
/// epsilon absorbs f64 representation error on products that land
/// exactly on a step (steps are a full 1.0 apart in percent space, so it
/// can never jump one).
fn quality_scaled(base: f64, quality: QualityTier) -> f64 {
    (base * (1.0 + 0.3 * quality.level() as f64) * 100.0 + 1e-9).floor() / 100.0
}

/// Prototype name for a family + tier (tier 1 has no suffix).
fn game_module_name(kind: ModulePolicyKind, tier: u8) -> Option<String> {
    let family = match kind {
        ModulePolicyKind::None => return None,
        ModulePolicyKind::Speed => "speed-module",
        ModulePolicyKind::Productivity => "productivity-module",
    };
    Some(match tier.clamp(1, 3) {
        1 => family.to_string(),
        t => format!("{family}-{t}"),
    })
}

/// Resolve the policy for one (machine, recipe) pair.
///
/// Eligibility (all data-driven from recipes.json, RFC-044 Phase 0c):
/// - the machine must have known module slots (> 0);
/// - the policy family's beneficial effect must be in the machine's
///   `allowed_effects` (absent list = unrestricted);
/// - productivity additionally requires the recipe's
///   `allow_productivity` — which also gates the machine's built-in
///   `base_effect` productivity (game rule: an ineligible recipe shows
///   no productivity bar at all, e.g. foundry-cast pipes get no +50%).
///
/// Ineligible pairs return [`MachineModuleEffects::none`] — the exact
/// no-op, bit-identical to a `None` policy (KC1).
pub fn resolve_machine_modules(
    policy: &ModulePolicy,
    machine: &str,
    recipe: &Recipe,
) -> MachineModuleEffects {
    let Some(name) = game_module_name(policy.kind, policy.tier) else {
        return MachineModuleEffects::none();
    };
    let Some(slots) = module_slots_known(machine).filter(|s| *s > 0) else {
        return MachineModuleEffects::none();
    };

    let machine_data = db().machines.get(machine);
    let allows = |effect: &str| -> bool {
        machine_data
            .and_then(|m| m.allowed_effects.as_ref())
            .map(|list| list.iter().any(|e| e == effect))
            .unwrap_or(true)
    };
    let base_effect = machine_data.map(|m| m.base_effect_productivity).unwrap_or(0.0);

    let eligible = match policy.kind {
        ModulePolicyKind::None => false,
        ModulePolicyKind::Speed => allows("speed"),
        ModulePolicyKind::Productivity => allows("productivity") && recipe.allow_productivity,
    };
    if !eligible {
        return MachineModuleEffects::none();
    }

    let base = module_effect(&name);
    let n = slots as f64;
    let (speed_sum, prod_sum) = match policy.kind {
        ModulePolicyKind::None => unreachable!("handled above"),
        // Positive speed scales with quality; no productivity.
        ModulePolicyKind::Speed => (n * quality_scaled(base.speed, policy.quality), 0.0),
        // Positive productivity scales with quality; the speed PENALTY
        // does not (harmful effects are quality-flat).
        ModulePolicyKind::Productivity => {
            (n * base.speed, n * quality_scaled(base.productivity, policy.quality))
        }
    };

    // Recipe-gated built-in productivity joins the module sum. For
    // productivity policies `recipe.allow_productivity` already held;
    // for speed policies it gates here.
    let prod_bonus = if recipe.allow_productivity {
        prod_sum + base_effect
    } else {
        prod_sum
    };

    MachineModuleEffects {
        // The 20% floor: a full prod-3 loadout in an 8-slot cryo plant
        // is −120% speed — negative without this (review MAJOR finding
        // on RFC rev 1).
        speed_multiplier: (1.0 + speed_sum).max(0.2),
        prod_bonus,
        loadout: vec![ModuleItem {
            item: name,
            count: slots,
            quality: (policy.quality != QualityTier::Normal).then_some(policy.quality),
        }],
    }
}

/// Effective per-craft amount of one product under a productivity bonus:
/// the catalyst portion (`ignored_by_productivity`) is exempt.
/// `prod_bonus == 0.0` returns `amount` UNCHANGED (bit-exact — the
/// subtract-and-re-add path is not an IEEE identity, so KC1 forbids it
/// on the no-op path).
pub fn effective_product_amount(amount: f64, ignored_by_productivity: f64, prod_bonus: f64) -> f64 {
    if prod_bonus == 0.0 {
        return amount;
    }
    let ignored = ignored_by_productivity.clamp(0.0, amount);
    (amount - ignored) * (1.0 + prod_bonus) + ignored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(name: &str) -> &'static Recipe {
        &db().recipes[name]
    }

    fn policy(kind: ModulePolicyKind, tier: u8, quality: QualityTier) -> ModulePolicy {
        ModulePolicy { kind, tier, quality }
    }

    #[test]
    fn none_policy_is_exact_noop() {
        let e = resolve_machine_modules(
            &ModulePolicy::default(),
            "assembling-machine-3",
            recipe("iron-gear-wheel"),
        );
        assert_eq!(e, MachineModuleEffects::none());
        assert_eq!(e.speed_multiplier.to_bits(), 1.0f64.to_bits());
        assert_eq!(e.prod_bonus.to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn kc5_am3_four_prod3() {
        // RFC verification plan: AM3 + 4× prod-3 → speed ×0.4 (−15% each),
        // products ×1.4.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
            "assembling-machine-3",
            recipe("iron-gear-wheel"),
        );
        assert!((e.speed_multiplier - 0.4).abs() < 1e-12);
        assert!((e.prod_bonus - 0.4).abs() < 1e-12);
        assert_eq!(e.loadout[0].item, "productivity-module-3");
        assert_eq!(e.loadout[0].count, 4);
    }

    #[test]
    fn kc5_cryo_speed_floor_and_base_effect_absence() {
        // RFC verification plan: cryo + 8× prod-3 → 1 − 1.2 floors to
        // 0.2; cryo has no base_effect, so prod = +0.8 exactly.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
            "cryogenic-plant",
            recipe("fluoroketone"),
        );
        assert_eq!(e.speed_multiplier, 0.2);
        assert!((e.prod_bonus - 0.8).abs() < 1e-12);
    }

    #[test]
    fn foundry_base_effect_joins_prod() {
        // Foundry: 4 slots, base_effect 0.5 → 4×0.10 + 0.5 = 0.9.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
            "foundry",
            recipe("casting-iron"),
        );
        assert!((e.prod_bonus - 0.9).abs() < 1e-12);
    }

    #[test]
    fn foundry_base_effect_gated_by_recipe_eligibility() {
        // casting-pipe is allow_productivity=false: no modules under a
        // prod policy AND no base_effect credit — the game shows no
        // productivity bar at all for ineligible recipes.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
            "foundry",
            recipe("casting-pipe"),
        );
        assert_eq!(e, MachineModuleEffects::none());

        // Speed policy on the same pair: modules apply, base_effect
        // still gated off.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Speed, 3, QualityTier::Normal),
            "foundry",
            recipe("casting-pipe"),
        );
        assert_eq!(e.prod_bonus, 0.0);
        assert!((e.speed_multiplier - 3.0).abs() < 1e-12); // 1 + 4×0.5
    }

    #[test]
    fn speed_policy_foundry_eligible_recipe_credits_base_effect() {
        // Speed policy, prod-eligible recipe: base_effect rides along.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Speed, 1, QualityTier::Normal),
            "foundry",
            recipe("casting-iron"),
        );
        assert!((e.prod_bonus - 0.5).abs() < 1e-12);
        assert!((e.speed_multiplier - 1.8).abs() < 1e-12); // 1 + 4×0.2
    }

    #[test]
    fn quality_scaling_floors_to_percent_steps() {
        // Wiki-anchored values (RFC game-rule model): prod-1 uncommon is
        // +5% (not 5.2), prod-2 rare is +9% (not 9.6); legendary lands
        // on integers (prod-3 +25%, speed-3 +125%).
        assert_eq!(quality_scaled(0.04, QualityTier::Uncommon), 0.05);
        assert_eq!(quality_scaled(0.06, QualityTier::Rare), 0.09);
        assert_eq!(quality_scaled(0.10, QualityTier::Legendary), 0.25);
        assert_eq!(quality_scaled(0.50, QualityTier::Legendary), 1.25);
        assert_eq!(quality_scaled(0.10, QualityTier::Normal), 0.10);
    }

    #[test]
    fn prod_speed_penalty_is_quality_flat() {
        // Legendary prod-3: productivity scales ×2.5 but the −15% speed
        // penalty stays flat.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Productivity, 3, QualityTier::Legendary),
            "assembling-machine-3",
            recipe("iron-gear-wheel"),
        );
        assert!((e.prod_bonus - 1.0).abs() < 1e-12); // 4 × 0.25
        assert!((e.speed_multiplier - 0.4).abs() < 1e-12); // 1 − 4×0.15
        assert_eq!(e.loadout[0].quality, Some(QualityTier::Legendary));
    }

    #[test]
    fn prod_in_recycler_falls_back_to_no_modules() {
        // Machine-level allowed_effects: recycler forbids productivity.
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
            "recycler",
            recipe("iron-gear-wheel-recycling"),
        );
        assert_eq!(e, MachineModuleEffects::none());
    }

    #[test]
    fn am1_zero_slots_no_modules() {
        let e = resolve_machine_modules(
            &policy(ModulePolicyKind::Speed, 3, QualityTier::Normal),
            "assembling-machine-1",
            recipe("iron-gear-wheel"),
        );
        assert_eq!(e, MachineModuleEffects::none());
    }

    #[test]
    fn kc5_solve_level_machine_count_is_explainable() {
        // KC5(a): EC from plates, AM3, 4× prod-3 normal. Per machine:
        // crafts/s = 1.25 × 0.4 / 0.5 = 1.0; output = 1 × 1.4 = 1.4/s.
        // 10/s target → fractional count 10/1.4 (snap-exact).
        use rustc_hash::FxHashSet;
        let inputs: FxHashSet<String> =
            ["iron-plate", "copper-plate"].iter().map(|s| s.to_string()).collect();
        let result = crate::solver::solve_with_palette_exclusions_quality_and_modules(
            "electronic-circuit",
            10.0,
            &inputs,
            &crate::recipe_db::MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
            policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
        )
        .unwrap();
        let ec = result
            .machines
            .iter()
            .find(|m| m.recipe == "electronic-circuit")
            .unwrap();
        assert!((ec.count - 10.0 / 1.4).abs() < 1e-9, "count {}", ec.count);
        let out = ec.outputs.iter().find(|o| o.item == "electronic-circuit").unwrap();
        assert!((out.rate - 1.4).abs() < 1e-12, "per-machine rate {}", out.rate);
        // Ingredients per craft unchanged: 3 cable × 1.0 craft/s.
        let cable = ec.inputs.iter().find(|i| i.item == "copper-cable").unwrap();
        assert!((cable.rate - 3.0).abs() < 1e-12);
        // Loadout rides the spec for the layout stamp pass.
        assert_eq!(ec.game_modules.len(), 1);
        assert_eq!(ec.game_modules[0].item, "productivity-module-3");
        assert_eq!(ec.game_modules[0].count, 4);
        // Upstream cable machines are prod'd too (2 cable per craft → 2.8).
        let cc = result.machines.iter().find(|m| m.recipe == "copper-cable").unwrap();
        assert_eq!(cc.game_modules.len(), 1);
    }

    #[test]
    fn kc1_none_policy_solve_is_bit_identical() {
        // The default-policy entry point and the quality entry point must
        // produce bit-identical results (same code path, exact no-op).
        use rustc_hash::FxHashSet;
        let inputs: FxHashSet<String> =
            ["iron-plate", "copper-plate"].iter().map(|s| s.to_string()).collect();
        let base = crate::solver::solve_with_palette_exclusions_and_quality(
            "electronic-circuit",
            7.5,
            &inputs,
            &crate::recipe_db::MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let with_none = crate::solver::solve_with_palette_exclusions_quality_and_modules(
            "electronic-circuit",
            7.5,
            &inputs,
            &crate::recipe_db::MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
            ModulePolicy::default(),
        )
        .unwrap();
        assert_eq!(base.machines.len(), with_none.machines.len());
        for (a, b) in base.machines.iter().zip(&with_none.machines) {
            assert_eq!(a.count.to_bits(), b.count.to_bits(), "{}", a.recipe);
            assert!(b.game_modules.is_empty());
            for (fa, fb) in a.inputs.iter().zip(&b.inputs) {
                assert_eq!(fa.rate.to_bits(), fb.rate.to_bits());
            }
            for (fa, fb) in a.outputs.iter().zip(&b.outputs) {
                assert_eq!(fa.rate.to_bits(), fb.rate.to_bits());
            }
        }
    }

    #[test]
    fn kovarex_solve_level_prod_boosts_net_only() {
        // Kovarex under prod-3: centrifuge is prod-eligible; the U-235
        // self-loop's produced side rises by prod on the net +1 only.
        use rustc_hash::FxHashSet;
        let inputs: FxHashSet<String> =
            ["uranium-238"].iter().map(|s| s.to_string()).collect();
        let result = crate::solver::solve_with_palette_exclusions_quality_and_modules(
            "uranium-235",
            0.1,
            &inputs,
            &crate::recipe_db::MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
            policy(ModulePolicyKind::Productivity, 3, QualityTier::Normal),
        )
        .unwrap();
        let kov = result
            .machines
            .iter()
            .find(|m| m.recipe == "kovarex-enrichment-process")
            .expect("kovarex spec");
        let u235 = kov.self_loop.iter().find(|s| s.item == "uranium-235").unwrap();
        // Per craft: consumed 40, produced 41 + 0.4 (prod on net 1 only;
        // centrifuge has 2 slots → 2 × 0.10 = +20%? No: count-based —
        // 2 slots × prod-3 (+10% each) = +0.2. Produced = 41.2.
        let crafts = u235.consumed_rate / 40.0;
        assert!(
            (u235.produced_rate / crafts - 41.2).abs() < 1e-9,
            "produced per craft {}",
            u235.produced_rate / crafts
        );
        assert_eq!(kov.game_modules[0].count, 2);
    }

    #[test]
    fn effective_amount_kovarex_catalyst_exempt() {
        // Kovarex U-235: 41 out, 40 ignored → +25% prod boosts only the
        // net +1: 41 + 0.25.
        let a = effective_product_amount(41.0, 40.0, 0.25);
        assert!((a - 41.25).abs() < 1e-12);
        // U-238 return: fully ignored, unchanged.
        assert_eq!(effective_product_amount(2.0, 2.0, 0.25), 2.0);
        // No-op path is bit-exact.
        assert_eq!(effective_product_amount(0.1, 0.07, 0.0).to_bits(), 0.1f64.to_bits());
    }
}
