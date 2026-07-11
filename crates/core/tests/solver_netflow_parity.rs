//! Phase 0 parity harness for the net-flow solver.
//! See docs/rfp-solver-net-flow.md — kill criteria 1–5 are evaluated here.

use rustc_hash::FxHashSet;
use spaghettio_core::netflow::{solve_netflow, CostTable, RecipeScope};
use spaghettio_core::recipe_db::{self, MachinePalette};
use spaghettio_core::solver::{self, SolverError};

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// One gated e2e solve config (mirrors crates/core/tests/e2e.rs — the
/// solver-relevant tuple only; belt tier / layout strategy don't reach the
/// solver). Update alongside e2e.rs when gated configs change.
struct GatedConfig {
    name: &'static str,
    item: &'static str,
    rate: f64,
    machine: &'static str,
    inputs: &'static [&'static str],
    excluded: &'static [&'static str],
}

const GATED: &[GatedConfig] = &[
    GatedConfig { name: "tier1_iron_gear_wheel", item: "iron-gear-wheel", rate: 10.0, machine: "assembling-machine-1", inputs: &["iron-plate"], excluded: &[] },
    GatedConfig { name: "tier1_iron_gear_wheel_from_ore", item: "iron-gear-wheel", rate: 10.0, machine: "assembling-machine-2", inputs: &["iron-ore"], excluded: &[] },
    GatedConfig { name: "tier1_iron_gear_wheel_20s", item: "iron-gear-wheel", rate: 20.0, machine: "assembling-machine-2", inputs: &["iron-plate"], excluded: &[] },
    GatedConfig { name: "tier2_electronic_circuit", item: "electronic-circuit", rate: 10.0, machine: "assembling-machine-2", inputs: &["iron-plate", "copper-plate"], excluded: &[] },
    GatedConfig { name: "tier2_electronic_circuit_from_ore", item: "electronic-circuit", rate: 10.0, machine: "assembling-machine-1", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "tier2_electronic_circuit_20s_from_ore", item: "electronic-circuit", rate: 20.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "tier3_plastic_bar", item: "plastic-bar", rate: 10.0, machine: "chemical-plant", inputs: &["petroleum-gas", "coal"], excluded: &[] },
    GatedConfig { name: "tier3_plastic_bar_from_crude", item: "plastic-bar", rate: 10.0, machine: "chemical-plant", inputs: &["crude-oil", "coal"], excluded: &[] },
    GatedConfig { name: "tier3_sulfuric_acid", item: "sulfuric-acid", rate: 5.0, machine: "chemical-plant", inputs: &["iron-plate", "sulfur", "water"], excluded: &[] },
    GatedConfig { name: "tier3_heavy_oil_cracking", item: "light-oil", rate: 5.0, machine: "chemical-plant", inputs: &["water", "heavy-oil"], excluded: &["advanced-oil-processing", "coal-liquefaction"] },
    GatedConfig { name: "tier3_advanced_oil_processing_multi_machine", item: "petroleum-gas", rate: 12.0, machine: "oil-refinery", inputs: &["water", "crude-oil"], excluded: &[] },
    GatedConfig { name: "tier4_advanced_circuit_from_plates", item: "advanced-circuit", rate: 1.0, machine: "assembling-machine-2", inputs: &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], excluded: &[] },
    GatedConfig { name: "tier4_advanced_circuit_from_ore_am2", item: "advanced-circuit", rate: 5.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], excluded: &[] },
    GatedConfig { name: "tier5_processing_unit_from_ore_am3", item: "processing-unit", rate: 2.0, machine: "assembling-machine-3", inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], excluded: &[] },
    GatedConfig { name: "processing_unit_2s_am2_fast_belts", item: "processing-unit", rate: 2.0, machine: "assembling-machine-2", inputs: &["iron-plate", "copper-plate", "steel-plate", "stone", "coal", "water", "crude-oil", "iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ec_22s", item: "electronic-circuit", rate: 22.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ec_23s", item: "electronic-circuit", rate: 23.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ec_30s", item: "electronic-circuit", rate: 30.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ec_35s", item: "electronic-circuit", rate: 35.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ec_40s", item: "electronic-circuit", rate: 40.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ec_60s", item: "electronic-circuit", rate: 60.0, machine: "assembling-machine-2", inputs: &["iron-ore", "copper-ore"], excluded: &[] },
    GatedConfig { name: "stress_ac_partitioned_4s", item: "advanced-circuit", rate: 4.0, machine: "assembling-machine-2", inputs: &["iron-plate", "copper-plate", "plastic-bar"], excluded: &[] },
    GatedConfig { name: "stress_ac_partitioned_5s", item: "advanced-circuit", rate: 5.0, machine: "assembling-machine-2", inputs: &["iron-plate", "copper-plate", "plastic-bar"], excluded: &[] },
];

/// Tree-walk recipe set for a config (compat-mode column restriction).
fn walk_set(result: &spaghettio_core::models::SolverResult) -> FxHashSet<String> {
    result.dependency_order.iter().cloned().collect()
}

fn compat_solve(
    cfg: &GatedConfig,
) -> (
    spaghettio_core::models::SolverResult,
    spaghettio_core::models::SolverResult,
) {
    let inputs = set(cfg.inputs);
    let excluded = set(cfg.excluded);
    let walk = solver::solve_tree_walk_with_palette_and_exclusions(
        cfg.item,
        cfg.rate,
        &inputs,
        &MachinePalette::default(),
        cfg.machine,
        &excluded,
    )
    .unwrap_or_else(|e| panic!("{}: tree walk failed: {e}", cfg.name));
    let recipes = walk_set(&walk);
    let lp = solve_netflow(
        cfg.item,
        cfg.rate,
        &inputs,
        &MachinePalette::default(),
        cfg.machine,
        &excluded,
        RecipeScope::Restricted(&recipes),
        &CostTable::default(),
    )
    .unwrap_or_else(|e| panic!("{}: netflow failed: {e}", cfg.name));
    (walk, lp)
}

/// Kill criterion 1 tolerance: |a−b| ≤ max(0.001, 0.1% · max(a,b)).
fn within_parity_tol(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::max(0.001, 0.001 * f64::max(a.abs(), b.abs()))
}

/// KILL CRITERION 1 — pinned (compatibility-mode) parity on the gated corpus.
#[test]
fn kc1_pinned_parity_on_gated_corpus() {
    for cfg in GATED {
        let (walk, lp) = compat_solve(cfg);

        // Same recipe set, same traversal order (golden-hash stability).
        assert_eq!(
            walk.dependency_order, lp.dependency_order,
            "{}: dependency_order diverged",
            cfg.name
        );
        assert_eq!(
            walk.external_inputs.iter().map(|f| &f.item).collect::<Vec<_>>(),
            lp.external_inputs.iter().map(|f| &f.item).collect::<Vec<_>>(),
            "{}: external input order diverged",
            cfg.name
        );

        // Per-recipe machine counts within tolerance.
        for wm in &walk.machines {
            let lm = lp
                .machines
                .iter()
                .find(|m| m.recipe == wm.recipe)
                .unwrap_or_else(|| panic!("{}: recipe {} missing from netflow", cfg.name, wm.recipe));
            assert!(
                within_parity_tol(wm.count, lm.count),
                "{}: machine count for {} diverged: walk={} lp={}",
                cfg.name,
                wm.recipe,
                wm.count,
                lm.count
            );
            assert_eq!(wm.entity, lm.entity, "{}: machine entity diverged", cfg.name);
        }

        // Per-item external input rates within tolerance.
        for wf in &walk.external_inputs {
            let lf = lp
                .external_inputs
                .iter()
                .find(|f| f.item == wf.item)
                .unwrap_or_else(|| panic!("{}: external {} missing from netflow", cfg.name, wf.item));
            assert!(
                within_parity_tol(wf.rate, lf.rate),
                "{}: external rate for {} diverged: walk={} lp={}",
                cfg.name,
                wf.item,
                wf.rate,
                lf.rate
            );
        }

        // Analysis said the gated corpus produces no surplus under compat
        // mode. A surplus entry here means that analysis (or the compat
        // restriction) is wrong — kill criterion 1 fires.
        assert!(
            lp.surplus_outputs.is_empty(),
            "{}: unexpected surplus on gated config: {:?}",
            cfg.name,
            lp.surplus_outputs
                .iter()
                .map(|f| (f.item.clone(), f.rate))
                .collect::<Vec<_>>()
        );
    }
}

/// Flow conservation over every producible item (free mode, gauntlet-style
/// raw inputs). The tree walk FAILS this on oil/uranium chains — that is the
/// motivating bug; here we assert the LP never does. Typed cycle refusals
/// are acceptable outcomes, recorded and bounded below.
#[test]
fn netflow_flow_conservation_sweep() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "stone", "crude-oil", "water"]);
    let mut refusals: Vec<(String, String)> = Vec::new();
    let mut solved = 0usize;

    for item in recipe_db::all_producible_items() {
        let r = solve_netflow(
            &item,
            1.0,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            RecipeScope::Free,
            &CostTable::default(),
        );
        match r {
            Ok(result) => {
                solved += 1;
                assert_conservation(&item, 1.0, &result);
            }
            Err(SolverError::UnsupportedSelfLoop { recipe }) => {
                refusals.push((item.clone(), format!("self-loop {recipe}")));
            }
            Err(SolverError::UnsupportedCycle { recipes }) => {
                refusals.push((item.clone(), format!("cycle {recipes}")));
            }
            Err(SolverError::IncompatibleMachine { recipe, .. }) => {
                // Unsupported machine categories (rocket-building,
                // captive-spawner-process) surface as typed errors under
                // free selection instead of the walk's silent wrong-machine
                // or import-from-nowhere output.
                refusals.push((item.clone(), format!("incompatible {recipe}")));
            }
            Err(e) => panic!("{item}: unexpected solver error: {e}"),
        }
    }

    println!("solved {solved} items; {} typed refusals: {refusals:?}", refusals.len());
    // KC5 census, evaluated 2026-07-10 (see RFP decision log); re-checked
    // 2026-07-11 after solver-side self-loop netting landed (RFP Phase 2,
    // "Cycle policy"). Two known refusal families remain, both of which the
    // tree walk today "solves" with physically-broken output (nonsense
    // externals / stranded byproducts):
    //   1. Gleba forced self-loops: pentapod-egg (60 water/craft) and
    //      fish-breeding (100 water/craft) each have a fluid ingredient
    //      alongside their solid self-loop item, so — per
    //      `classify_self_loop` in netflow.rs — they fall outside v1's
    //      pure-solid self-loop support and stay refused. Phase 2's
    //      self-loop row template needs fluid support before these solve.
    //   2. The Aquilo fluoroketone coolant loop (fresh fluoroketone is
    //      produced HOT; the only cold producer is the cooler, so the
    //      loop is mandatory) — needs the forced-surplus edge-cut planned
    //      for Phase 2, or multi-row cycle routing (out of scope).
    // Pure-solid self-loops (kovarex-enrichment-process,
    // iron/copper-bacteria-cultivation — verified via recipes.json: no
    // fluid ingredients or products) now solve via net flows and are
    // correctly absent from this list; they were never forced under this
    // sweep's default inputs anyway (the plain, non-self-loop producing
    // recipes are always cheaper here), so their absence isn't new. Forcing
    // kovarex (excluding uranium-processing) is covered by the dedicated
    // golden tests below instead.
    for (item, why) in &refusals {
        let known = why.starts_with("self-loop pentapod-egg")
            || why.starts_with("self-loop fish-breeding")
            || why.contains("fluoroketone-cooling")
            || why.starts_with("incompatible");
        assert!(
            known,
            "KC5: refusal outside the reviewed census for {item}: {why}"
        );
    }
    assert!(
        refusals.len() <= 24,
        "KC5: refusal list grew beyond the reviewed census: {refusals:?}"
    );
}

/// Net production + externals − surplus must equal the target, per item.
fn assert_conservation(target: &str, rate: f64, r: &spaghettio_core::models::SolverResult) {
    use std::collections::HashMap;
    let mut net: HashMap<&str, f64> = HashMap::new();
    for m in &r.machines {
        for f in &m.outputs {
            *net.entry(f.item.as_str()).or_default() += f.rate * m.count;
        }
        for f in &m.inputs {
            *net.entry(f.item.as_str()).or_default() -= f.rate * m.count;
        }
    }
    for f in &r.external_inputs {
        *net.entry(f.item.as_str()).or_default() += f.rate;
    }
    for f in &r.surplus_outputs {
        *net.entry(f.item.as_str()).or_default() -= f.rate;
    }
    for (item, v) in net {
        let expected = if item == target { rate } else { 0.0 };
        assert!(
            (v - expected).abs() < 1e-6,
            "{target}: conservation violated for {item}: net {v}, expected {expected}"
        );
    }
}

/// KILL CRITERION 2 — determinism: two full sweeps must serialize
/// byte-identically.
#[test]
fn kc2_determinism_double_run() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "stone", "crude-oil", "water"]);
    let run = || -> String {
        let mut out = String::new();
        for item in recipe_db::all_producible_items() {
            let r = solve_netflow(
                &item,
                1.0,
                &inputs,
                &MachinePalette::default(),
                "assembling-machine-3",
                &FxHashSet::default(),
                RecipeScope::Free,
                &CostTable::default(),
            );
            match r {
                Ok(res) => out.push_str(&serde_json::to_string(&res).unwrap()),
                Err(e) => out.push_str(&format!("ERR:{e}")),
            }
            out.push('\n');
        }
        out
    };
    let a = run();
    let b = run();
    assert!(a == b, "KC2: netflow sweep is not deterministic");
}

/// Golden: kovarex forced (exclude uranium-processing) now SOLVES via
/// self-loop net flows (RFP Phase 2) instead of the typed refusal Phase 1
/// shipped. Free selection is also free to route the U-238 deficit through
/// any other in-closure producer (here: nuclear-fuel-reprocessing from
/// depleted-uranium-fuel-cell, which the LP finds cheaper than the tree
/// walk's silent nonsense externals) — this test only asserts kovarex
/// itself is active and the whole plan conserves; the isolated netting
/// arithmetic is nailed down by `kovarex_self_loop_net_flows_hand_derived`
/// below, which restricts the scope to kovarex alone.
#[test]
fn golden_kovarex_solves_as_self_loop() {
    let inputs = set(&["uranium-ore", "water"]);
    let excluded = set(&["uranium-processing"]);
    let r = solve_netflow(
        "uranium-235",
        1.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &excluded,
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("kovarex must solve now that Phase 2 self-loop netting has landed");
    assert!(
        r.dependency_order.contains(&"kovarex-enrichment-process".to_string()),
        "expected kovarex in the plan: {:?}",
        r.dependency_order
    );
    let kovarex = r
        .machines
        .iter()
        .find(|m| m.recipe == "kovarex-enrichment-process")
        .expect("kovarex machine spec present");
    assert_eq!(kovarex.entity, "centrifuge");
    // Net flows only: the self-referencing items must not leak into
    // ordinary inputs/outputs alongside their netted entry.
    assert_eq!(kovarex.self_loop.len(), 2);
    assert!(!kovarex.inputs.iter().any(|f| f.item == "uranium-235"));
    assert!(!kovarex.outputs.iter().any(|f| f.item == "uranium-238"));
    assert_conservation("uranium-235", 1.0, &r);
}

/// Hand-derived kovarex netting math (RFP Phase 2): restrict the LP to
/// kovarex alone (excluding uranium-processing AND any other producer of
/// uranium-238, so there is no alternative for free selection to route
/// through) so the only unknowns under test are the netting arithmetic
/// itself — machine count, net inputs/outputs, self_loop entries, and the
/// external supply of the net-consumed item.
#[test]
fn kovarex_self_loop_net_flows_hand_derived() {
    let inputs = set(&["uranium-ore", "water"]);
    let excluded = set(&["uranium-processing"]);
    let scope_set = set(&["kovarex-enrichment-process"]);
    let r = solve_netflow(
        "uranium-235",
        0.1,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &excluded,
        RecipeScope::Restricted(&scope_set),
        &CostTable::default(),
    )
    .expect("kovarex-only scope solves");

    assert_eq!(
        r.machines.len(),
        1,
        "expected only the kovarex column: {:?}",
        r.machines
    );
    let m = &r.machines[0];
    assert_eq!(m.recipe, "kovarex-enrichment-process");
    assert_eq!(m.entity, "centrifuge");
    // x[kovarex] = target_rate / net(U-235)/craft = 0.1 / 1 = 0.1 crafts/s.
    // count = crafts/s / (crafting_speed/energy) = 0.1 / (1.0/60) = 6.0.
    assert!((m.count - 6.0).abs() < 1e-9, "count: {}", m.count);

    assert_eq!(m.inputs.len(), 1, "inputs: {:?}", m.inputs);
    assert_eq!(m.inputs[0].item, "uranium-238");
    assert!(!m.inputs[0].is_fluid);
    assert!(
        (m.inputs[0].rate - 3.0 / 60.0).abs() < 1e-9,
        "input rate: {}",
        m.inputs[0].rate
    );

    assert_eq!(m.outputs.len(), 1, "outputs: {:?}", m.outputs);
    assert_eq!(m.outputs[0].item, "uranium-235");
    assert!(!m.outputs[0].is_fluid);
    assert!(
        (m.outputs[0].rate - 1.0 / 60.0).abs() < 1e-9,
        "output rate: {}",
        m.outputs[0].rate
    );

    assert_eq!(m.self_loop.len(), 2, "self_loop: {:?}", m.self_loop);
    let u235 = m
        .self_loop
        .iter()
        .find(|f| f.item == "uranium-235")
        .expect("uranium-235 self-loop entry");
    assert!(!u235.is_fluid);
    assert!((u235.consumed_rate - 40.0 / 60.0).abs() < 1e-9, "{u235:?}");
    assert!((u235.produced_rate - 41.0 / 60.0).abs() < 1e-9, "{u235:?}");
    assert!((u235.net_rate - 1.0 / 60.0).abs() < 1e-9, "{u235:?}");

    let u238 = m
        .self_loop
        .iter()
        .find(|f| f.item == "uranium-238")
        .expect("uranium-238 self-loop entry");
    assert!(!u238.is_fluid);
    assert!((u238.consumed_rate - 5.0 / 60.0).abs() < 1e-9, "{u238:?}");
    assert!((u238.produced_rate - 2.0 / 60.0).abs() < 1e-9, "{u238:?}");
    assert!((u238.net_rate - (-3.0 / 60.0)).abs() < 1e-9, "{u238:?}");

    assert_eq!(
        r.external_inputs.len(),
        1,
        "unexpected externals: {:?}",
        r.external_inputs
    );
    assert_eq!(r.external_inputs[0].item, "uranium-238");
    assert!(
        (r.external_inputs[0].rate - 0.3).abs() < 1e-9,
        "external u238 rate: {}",
        r.external_inputs[0].rate
    );

    assert_conservation("uranium-235", 0.1, &r);
}

/// Golden: rocket-fuel free mode — the reviewer-verified optimum. AOP alone
/// on the refinery side (no basic-oil blend), zero surplus (all three
/// co-products consumed via the solid-fuel split + direct light-oil use).
#[test]
fn golden_rocket_fuel_free_mode_zero_surplus() {
    let inputs = set(&["crude-oil", "water"]);
    let r = solve_netflow(
        "rocket-fuel",
        1.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("rocket-fuel solves");
    assert_conservation("rocket-fuel", 1.0, &r);
    assert!(
        r.surplus_outputs.is_empty(),
        "expected zero surplus, got {:?}",
        r.surplus_outputs.iter().map(|f| (&f.item, f.rate)).collect::<Vec<_>>()
    );
    let recipes: Vec<&str> = r.machines.iter().map(|m| m.recipe.as_str()).collect();
    assert!(
        !recipes.contains(&"basic-oil-processing"),
        "AOP should fully replace basic-oil at the optimum; got {recipes:?}"
    );
    assert!(recipes.contains(&"advanced-oil-processing"), "got {recipes:?}");
}

/// Golden: rocket-fuel compat mode — byproduct crediting within the tree
/// walk's own recipe set. AOP is pinned in by the walk's light-oil choice;
/// its gas byproduct must offset basic-oil production (strictly fewer
/// basic-oil machines than the walk built), with zero *fluid* stranding
/// hidden: any remaining imbalance must be explicit surplus.
#[test]
fn golden_rocket_fuel_compat_credits_byproducts() {
    let inputs = set(&["crude-oil", "water"]);
    let excluded = FxHashSet::default();
    let walk = solver::solve_tree_walk_with_palette_and_exclusions(
        "rocket-fuel",
        1.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &excluded,
    )
    .expect("walk solves");
    let recipes = walk_set(&walk);
    let lp = solve_netflow(
        "rocket-fuel",
        1.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &excluded,
        RecipeScope::Restricted(&recipes),
        &CostTable::default(),
    )
    .expect("netflow solves");
    assert_conservation("rocket-fuel", 1.0, &lp);

    let count = |r: &spaghettio_core::models::SolverResult, name: &str| {
        r.machines.iter().find(|m| m.recipe == name).map(|m| m.count)
    };
    if let (Some(w), Some(l)) = (count(&walk, "basic-oil-processing"), count(&lp, "basic-oil-processing")) {
        assert!(
            l < w - 0.001,
            "crediting should shrink basic-oil: walk={w} lp={l}"
        );
    }
    // The walk silently dropped AOP's byproducts; the LP must account for
    // every co-product — either consumed (crediting) or explicit surplus.
    // (This is exactly the class of delta kill criterion 1 exempts on
    // non-gated probes.)
}

/// ε sensitivity — 10× and 100× perturbations of the tiebreaker weights
/// must not change any golden solution's active recipe set or counts.
#[test]
fn golden_epsilon_sensitivity() {
    let inputs = set(&["crude-oil", "water"]);
    let solve_with = |costs: &CostTable| {
        solve_netflow(
            "rocket-fuel",
            1.0,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            RecipeScope::Free,
            costs,
        )
        .expect("rocket-fuel solves")
    };
    let base = solve_with(&CostTable::default());
    for (eo_mul, em_mul) in [(10.0, 10.0), (100.0, 100.0), (10.0, 100.0), (100.0, 10.0)] {
        let costs = CostTable {
            eps_surplus: CostTable::default().eps_surplus * eo_mul,
            eps_machine: CostTable::default().eps_machine * em_mul,
            ..CostTable::default()
        };
        let alt = solve_with(&costs);
        let base_recipes: Vec<(&str, f64)> =
            base.machines.iter().map(|m| (m.recipe.as_str(), m.count)).collect();
        let alt_recipes: Vec<(&str, f64)> =
            alt.machines.iter().map(|m| (m.recipe.as_str(), m.count)).collect();
        assert_eq!(
            base_recipes.len(),
            alt_recipes.len(),
            "ε ({eo_mul},{em_mul}): active set changed"
        );
        for ((br, bc), (ar, ac)) in base_recipes.iter().zip(alt_recipes.iter()) {
            assert_eq!(br, ar, "ε ({eo_mul},{em_mul}): recipe set changed");
            assert!(
                within_parity_tol(*bc, *ac),
                "ε ({eo_mul},{em_mul}): count changed for {br}: {bc} vs {ac}"
            );
        }
    }
}

/// KILL CRITERION 4 — perf. Run explicitly in release:
/// `cargo test --release --manifest-path crates/core/Cargo.toml \
///    --test solver_netflow_parity -- kc4 --ignored --nocapture`
#[test]
#[ignore = "perf gate — run in release mode per kill criterion 4"]
fn kc4_perf_sweep() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "stone", "crude-oil", "water"]);
    let items = recipe_db::all_producible_items();
    let mut times_us: Vec<u128> = Vec::with_capacity(items.len());
    for item in &items {
        let t0 = std::time::Instant::now();
        let _ = solve_netflow(
            item,
            1.0,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            RecipeScope::Free,
            &CostTable::default(),
        );
        times_us.push(t0.elapsed().as_micros());
    }
    times_us.sort_unstable();
    let median = times_us[times_us.len() / 2];
    let max = *times_us.last().unwrap();
    println!(
        "KC4: n={} median={}µs p90={}µs max={}µs",
        times_us.len(),
        median,
        times_us[times_us.len() * 9 / 10],
        max
    );
    assert!(median <= 2_000, "KC4: median {median}µs > 2ms");
    assert!(max <= 10_000, "KC4: max {max}µs > 10ms");
}

/// Phase 0 report (not a gate): where free selection diverges from the tree
/// walk. Evidence base for the Phase 3 flip and kill criterion 5's census.
#[test]
#[ignore = "report only — run with --ignored --nocapture"]
fn report_unpinned_deltas() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "stone", "crude-oil", "water"]);
    let mut changed = 0usize;
    for item in recipe_db::all_producible_items() {
        let excluded = FxHashSet::default();
        let walk = match solver::solve_tree_walk_with_palette_and_exclusions(
            &item,
            1.0,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &excluded,
        ) {
            Ok(w) => w,
            Err(e) => {
                println!("{item}: walk error: {e}");
                continue;
            }
        };
        let free = match solve_netflow(
            &item,
            1.0,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &excluded,
            RecipeScope::Free,
            &CostTable::default(),
        ) {
            Ok(f) => f,
            Err(e) => {
                println!("{item}: netflow refusal: {e}");
                continue;
            }
        };
        let ws: FxHashSet<&str> = walk.dependency_order.iter().map(|s| s.as_str()).collect();
        let fs: FxHashSet<&str> = free.dependency_order.iter().map(|s| s.as_str()).collect();
        if ws != fs {
            changed += 1;
            let added: Vec<&&str> = fs.difference(&ws).collect();
            let removed: Vec<&&str> = ws.difference(&fs).collect();
            println!("{item}: +{added:?} -{removed:?} surplus={:?}",
                free.surplus_outputs.iter().map(|f| (&f.item, f.rate)).collect::<Vec<_>>());
        }
    }
    println!("recipe-set deltas under free selection: {changed} items");
}
