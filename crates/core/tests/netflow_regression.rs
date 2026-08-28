//! Net-flow solver's own regression suite — formerly the netflow half of
//! `solver_netflow_parity.rs`, split out when #632 A1 (churn-reduction
//! campaign) deleted the legacy recursive tree walk. See
//! docs/rfc-solver-net-flow.md.
//!
//! Coverage: whole-graph flow conservation
//! (`netflow_flow_conservation_sweep` + `assert_conservation` — the only
//! whole-graph conservation coverage in the repo), full-sweep determinism
//! (`kc2_determinism_double_run`), golden optima (kovarex self-loop
//! netting, rocket-fuel free mode, utility-science-pack oil crediting,
//! epsilon sensitivity), the perf gate (`kc4_perf_sweep`, ignored — run
//! explicitly in release), and the Fulgora scrap-economy spike (report,
//! determinism, voider break-even regression).
//!
//! #632 A1's original deletion pass over-scoped this file as entirely
//! walk-vs-LP parity and deleted it whole; a second-opinion review caught
//! that only 3 of its 14 tests actually referenced the legacy walk. Those
//! three, and their walk-only helpers, are gone for good (the walk itself
//! no longer exists to compare against):
//!   - `kc1_pinned_parity_on_gated_corpus` (+ helpers `GatedConfig`,
//!     `GATED`, `compat_solve`, `walk_set`)
//!   - `golden_rocket_fuel_compat_credits_byproducts`
//!   - `report_unpinned_deltas`
//!
//! Everything else in this file is unchanged from the original — restored
//! in the same PR (#635) that caught the over-deletion.

use rustc_hash::FxHashSet;
use spaghettio_core::common;
use spaghettio_core::models::SolverResult;
use spaghettio_core::netflow::{solve_netflow, solve_netflow_with_options, CostTable, NetflowOptions, RecipeScope};
use spaghettio_core::recipe_db::{self, MachinePalette};
use spaghettio_core::solver::{self, SolverError};
use spaghettio_core::trace::{self, TraceEvent};

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Numeric tolerance for comparing two solves: |a−b| ≤ max(0.001, 0.1% ·
/// max(a,b)). Originally KC1's (deleted, #632 A1) walk-vs-LP parity
/// tolerance; still used by `golden_epsilon_sensitivity` below to check
/// ε-perturbation stability, which is why this helper survived the walk.
fn within_parity_tol(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::max(0.001, 0.001 * f64::max(a.abs(), b.abs()))
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
    // KC5 census, evaluated 2026-07-10 (see RFC decision log); re-checked
    // 2026-07-11 after solver-side self-loop netting landed (RFC Phase 2,
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
/// self-loop net flows (RFC Phase 2) instead of the typed refusal Phase 1
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

/// Hand-derived kovarex netting math (RFC Phase 2): restrict the LP to
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

/// Regression for #476: lubricant forces advanced oil processing into the
/// utility-science chain. Its co-products must displace basic oil
/// processing and be fully consumed by ordinary recipes; vanilla Factorio
/// has no ordinary fluid void.
#[test]
fn utility_science_credits_advanced_oil_petroleum_before_basic_oil() {
    let inputs = set(&["iron-ore", "copper-ore", "crude-oil", "water", "coal", "stone"]);
    let r = solve_netflow(
        "utility-science-pack",
        2.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("utility science solves");

    assert_conservation("utility-science-pack", 2.0, &r);
    let recipes: Vec<&str> = r.machines.iter().map(|m| m.recipe.as_str()).collect();
    assert!(recipes.contains(&"advanced-oil-processing"), "got {recipes:?}");
    assert!(
        !recipes.contains(&"basic-oil-processing"),
        "advanced oil's petroleum must displace basic oil; got {recipes:?}"
    );
    assert!(
        r.surplus_outputs.iter().all(|f| !f.is_fluid),
        "all fluid co-products must be consumed, not stranded: {:?}",
        r.surplus_outputs
    );
    assert!(
        recipes.contains(&"heavy-oil-cracking"),
        "heavy-oil excess must be consumed by a real recipe; got {recipes:?}"
    );
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

/// #461 part (a) follow-up — solver-level pin. The `recipe_db.rs` unit
/// test fixed alongside this only pins `machine_for_recipe`'s lookup
/// table; this pins the actual entry point `run_e2e` (and the web app)
/// call — `solver::solve_with_exclusions` — so a future palette or
/// category change that reintroduces a burner anywhere on the machine-
/// selection path trips a test against the real pipeline, not just the
/// table lookup.
///
/// Before #461 part (a), `organic-or-assembling` recipes resolved to
/// `biochamber` — a burner (fuel category `nutrients`) — and nothing in
/// the engine delivers burner fuel, so the layout validated clean and
/// produced 0/s (issue #461's rocket-fuel example: 0 errors, 0 warnings,
/// 0.00/s measured, census `no_fuel: 8`).
///
/// Two of the six `organic-or-assembling` recipes, each solved off a
/// single raw input so the LP has no other recipe to reach for:
/// - `rocket-fuel` — `light-oil` supplies both itself (direct, fluid
///   ingredient) and `solid-fuel` (via `solid-fuel-from-light-oil`,
///   category `chemistry` → chemical-plant, electric).
/// - `nutrients-from-spoilage` — targeted via its product `nutrients`
///   (the item name differs from the recipe name here, unlike
///   rocket-fuel).
///
/// **Both items have OTHER producer recipes outside the six, and free-mode
/// cost selection is not shy about reaching for them** — discovered
/// empirically writing this test, not something to leave implicit:
/// `rocket-fuel` is ALSO produced by `rocket-fuel-from-jelly` (pure
/// `organic` — biochamber; out of #461 part (a)'s scope, a separate task's
/// concern) and `ammonia-rocket-fuel` (`chemistry-or-cryogenics` —
/// chemical-plant); `nutrients` is ALSO produced by
/// `nutrients-from-yumako-mash` and `nutrients-from-bioflux` (both pure
/// `organic` — biochamber) besides the other two `-or-assembling` siblings
/// (`nutrients-from-fish`, `nutrients-from-biter-egg`). Root resources with
/// no recipe of their own (`jellynut`, `yumako`, `ammoniacal-solution`, …)
/// are automatically free supply in the LP regardless of what
/// `available_inputs` lists (`s_eligible` in netflow.rs — anything with no
/// in-closure producer is eligible), so a first attempt at this test that
/// supplied only `light-oil` and left `rocket-fuel-from-jelly` uncontested
/// actually solved via the biochamber path (cheaper here: fewer, faster
/// machines dominated the raw-input weight disadvantage) — the OPPOSITE
/// of what this test exists to catch. All non-`-or-assembling` producers
/// (and the `-or-assembling` siblings not under test) are excluded below
/// so the solve is forced through the recipe #461 part (a) actually fixed.
#[test]
fn issue_461_no_burner_machines_in_solver_result() {
    let assert_no_burners = |item: &str, inputs: &FxHashSet<String>, excluded: &FxHashSet<String>| {
        let sr = solver::solve_with_exclusions(item, 1.0, inputs, "assembling-machine-3", excluded)
            .unwrap_or_else(|e| panic!("{item} solves: {e}"));
        for m in &sr.machines {
            assert!(
                common::needs_electricity(&m.entity),
                "#461: solving {item} placed a burner machine ({} for recipe {}) — nothing \
                 in the engine delivers burner fuel, so this layout would validate clean \
                 and produce 0/s",
                m.entity,
                m.recipe,
            );
        }
        sr
    };

    let rocket_fuel_result = assert_no_burners(
        "rocket-fuel",
        &set(&["light-oil"]),
        &set(&["rocket-fuel-from-jelly", "ammonia-rocket-fuel"]),
    );
    assert_no_burners(
        "nutrients",
        &set(&["spoilage"]),
        &set(&[
            "nutrients-from-yumako-mash",
            "nutrients-from-bioflux",
            "nutrients-from-fish",
            "nutrients-from-biter-egg",
        ]),
    );

    // "Electric" alone isn't the whole claim — the placed machine must also
    // be one the recipe DB actually says can run this recipe (fluid boxes,
    // ingredient slots, category all agree), not just happen to need grid
    // power. Pins "electric" and "can craft it" together, next to
    // `recipe_db::tests::rocket_fuel_runs_on_am2_am3_not_am1`, which pins
    // the same fact from the other side (the lookup table, not a live
    // solve).
    let rocket_fuel_machine = rocket_fuel_result
        .machines
        .iter()
        .find(|m| m.recipe == "rocket-fuel")
        .expect("rocket-fuel recipe must appear in its own solve");
    let rocket_fuel_recipe =
        recipe_db::find_recipe_for_item("rocket-fuel").expect("rocket-fuel recipe exists");
    assert!(
        recipe_db::machine_can_run_recipe(&rocket_fuel_machine.entity, rocket_fuel_recipe).is_ok(),
        "#461: solver placed {} for rocket-fuel, but recipe_db::machine_can_run_recipe says \
         that machine can't actually run this recipe",
        rocket_fuel_machine.entity,
    );
}

/// #461 follow-up — the AM1 question, pinned rather than asserted.
/// `organic-or-assembling` now sits in `GENERAL_CATEGORIES` (`recipe_db.rs`)
/// alongside every other assembler-tier category, and every recipe in that
/// set is checked by the SAME function, `machine_can_run_recipe`, in the
/// SAME order: ingredient-slot count, then fluid support, then category.
/// There is no tier-bump/auto-uptier mechanism anywhere in netflow.rs or
/// solver.rs (checked): a machine-incompatible column is either a hard
/// error (when it is the target's only producer) or silently dropped (when
/// an alternative exists) — never routed to a bigger machine.
///
/// **Outcome class, pinned against `processing-unit`** (category
/// `electronics-with-fluid`, also a `GENERAL_CATEGORIES` member, needs
/// sulfuric-acid): both `processing-unit`@AM1 and `rocket-fuel`@AM1 are a
/// HARD `SolverError::IncompatibleMachine` — never `Ok`, never a
/// substituted machine. Under `RecipeScope::Free` (what
/// `solve_with_exclusions` always uses), an incompatible column that is
/// the target's ONLY producer surfaces as this hard error instead of being
/// silently dropped (netflow.rs's `dropped_incompat` bookkeeping) — true
/// for both recipes here, since every other ingredient is supplied
/// directly.
///
/// **The specific `reason` genuinely differs, and that is expected, not a
/// bug**: `processing-unit` needs 3 ingredients (electronic-circuit,
/// advanced-circuit, sulfuric-acid) against AM1's 2-slot limit, so the
/// SLOT-COUNT check (which runs first) fires — `TooManyIngredients`, the
/// fluid check never even runs. `rocket-fuel` needs exactly 2
/// (solid-fuel, light-oil), clears the slot check, and hits the FLUID
/// check instead — `FluidNotSupported`. Both are the same ordered gate in
/// the same shared function; which sub-check fires depends on each
/// recipe's own shape, not on any divergence in machine-selection logic.
/// (`advanced-circuit`'s own 3-ingredient recipe is excluded even though
/// it is supplied directly: free mode still considers it as a candidate
/// column, and `dropped_incompat` surfaces whichever incompatible column
/// was enumerated FIRST — not necessarily the target's own producer — so
/// leaving it in would report `advanced-circuit`'s own refusal instead of
/// `processing-unit`'s.)
#[test]
fn issue_461_am1_fluid_recipe_outcome_matches_processing_unit() {
    fn outcome_class(result: &Result<SolverResult, SolverError>) -> &'static str {
        match result {
            Ok(_) => "solved",
            Err(SolverError::IncompatibleMachine { .. }) => "IncompatibleMachine",
            Err(SolverError::MissingCraftingSpeed { .. }) => "MissingCraftingSpeed",
            Err(SolverError::UnsupportedSelfLoop { .. }) => "UnsupportedSelfLoop",
            Err(SolverError::UnsupportedCycle { .. }) => "UnsupportedCycle",
            Err(SolverError::LpFailed { .. }) => "LpFailed",
        }
    }

    let processing_unit_result = solver::solve_with_exclusions(
        "processing-unit",
        1.0,
        &set(&["electronic-circuit", "advanced-circuit", "sulfuric-acid"]),
        "assembling-machine-1",
        &set(&["advanced-circuit"]),
    );
    let rocket_fuel_result = solver::solve_with_exclusions(
        "rocket-fuel",
        1.0,
        &set(&["solid-fuel", "light-oil"]),
        "assembling-machine-1",
        // Same reasoning as `issue_461_no_burner_machines_in_solver_result`'s
        // doc comment: `rocket-fuel-from-jelly` / `ammonia-rocket-fuel` are
        // OTHER producers of `rocket-fuel`, and `solid-fuel-from-*` are
        // other producers of the directly-supplied `solid-fuel` — left in,
        // free mode can route around the AM1-incompatible `rocket-fuel`
        // column entirely (solving via the biochamber path) instead of
        // hitting the refusal this test exists to pin, or `dropped_incompat`
        // can surface one of these siblings' refusal instead of
        // `rocket-fuel`'s own. Excluding them forces `rocket-fuel` to be
        // the only candidate, exactly like `advanced-circuit` is excluded
        // for `processing-unit` above.
        &set(&[
            "rocket-fuel-from-jelly",
            "ammonia-rocket-fuel",
            "solid-fuel-from-light-oil",
            "solid-fuel-from-petroleum-gas",
            "solid-fuel-from-heavy-oil",
            "solid-fuel-from-ammonia",
        ]),
    );

    // Same outcome CLASS: both a hard refusal, neither a solve, neither
    // any other SolverError variant (in particular, no tier bump — that
    // would show up as `Ok` with a machine other than AM1, not as a
    // distinct error variant, since no such mechanism exists to produce
    // one).
    assert_eq!(
        outcome_class(&processing_unit_result),
        outcome_class(&rocket_fuel_result),
        "rocket-fuel@AM1 must produce the same outcome CLASS as processing-unit@AM1 — both \
         route through the same machine_can_run_recipe gate; got processing-unit={:?}, \
         rocket-fuel={:?}",
        processing_unit_result,
        rocket_fuel_result,
    );

    // The specific reason legitimately differs by recipe shape (see doc
    // comment) — pinned separately so a regression in either direction is
    // caught precisely rather than papered over by the coarser class check
    // above.
    assert!(
        matches!(
            processing_unit_result,
            Err(SolverError::IncompatibleMachine {
                reason: recipe_db::MachineIncompatibility::TooManyIngredients { limit: 2, .. },
                ..
            })
        ),
        "processing-unit@AM1 outcome changed — update this test's doc comment: {processing_unit_result:?}"
    );
    assert!(
        matches!(
            rocket_fuel_result,
            Err(SolverError::IncompatibleMachine {
                reason: recipe_db::MachineIncompatibility::FluidNotSupported { .. },
                ..
            })
        ),
        "rocket-fuel@AM1 outcome changed — update this test's doc comment: {rocket_fuel_result:?}"
    );
}

/// #461 part (a) round 5 — the production-path gap rounds 1-2's exclusion-
/// guarded tests couldn't see, fixed as a POST-SOLVE re-solve
/// (`solver::avoid_burner_recipes`), not the LP cost penalty round 3 tried
/// first (`BURNER_MACHINE_COST_FACTOR`, reverted): any coefficient change
/// on any LP column — even a losing one — can shift the simplex's
/// floating-point path for the WHOLE shared LP instance, and pure-`organic`
/// recipes (`bioplastic` → plastic-bar, `biosulfur` → sulfur, …) sit in far
/// more fixtures' demand closures than #461 is about, so the penalty moved
/// 11 calibration fixtures' manifest hashes by float noise even after
/// narrowing it to burner-only categories. Doing it strictly AFTER the LP
/// has run — as a policy decision over the output, re-solving only when a
/// burner actually appears — keeps every other fixture's LP untouched.
///
/// Correction to round 3's doc comment: `solver::solve` is NOT the wasm
/// `solve` binding's entry point — wasm's `solve`/`solve_with_palette`
/// actually call `solver::solve_with_palette_exclusions_quality_and_modules`.
/// Both families (and the multi-target one) share the same
/// `avoid_burner_recipes` post-solve step, so pinning it here via
/// `solver::solve` still exercises the real mechanism.
///
/// Found empirically: the SIX-ORE set (`iron-ore`, `copper-ore`, `coal`,
/// `stone`, `crude-oil`, `water` — what the web passes by default)
/// targeting `rocket-fuel` at AM1, with NO exclusions (real callers never
/// exclude anything), selects `rocket-fuel-from-jelly` on a biochamber in
/// its FIRST solve — AM1 can't run the direct `rocket-fuel` recipe (fluid
/// check), and the jellynut/yumako-rooted chain's raw-input cost undercuts
/// the electric `ammonia-rocket-fuel` alternative. `avoid_burner_recipes`
/// then excludes `rocket-fuel-from-jelly` (its product, `rocket-fuel`, has
/// other producers whose category has an electric machine) and re-solves
/// once, landing on the electric chain — burner-free, so round 6's
/// acceptance rule takes it. Asserts the outcome (no burner in the final
/// result) AND that the re-solve trace event fired with `accepted: true` —
/// confirming the mechanism both engaged AND kept its result, not that the
/// LP happened to avoid a burner on its own.
#[test]
fn issue_461_production_path_prefers_electric_rocket_fuel() {
    let _guard = trace::start_trace();
    let inputs = set(&["iron-ore", "copper-ore", "coal", "stone", "crude-oil", "water"]);
    let sr = solver::solve("rocket-fuel", 1.0, &inputs, "assembling-machine-1")
        .unwrap_or_else(|e| panic!("rocket-fuel solves: {e}"));
    for m in &sr.machines {
        assert!(
            common::needs_electricity(&m.entity),
            "#461: the production path (solver::solve, no exclusions) placed a burner \
             machine ({} for recipe {}) for rocket-fuel@AM1 off the six-ore set — the \
             post-solve re-solve should have steered to the electric ammonia-rocket-fuel \
             alternative instead",
            m.entity,
            m.recipe,
        );
    }
    let events = trace::drain_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, TraceEvent::BurnerRecipeExcluded { accepted: true, .. })),
        "expected an ACCEPTED BurnerRecipeExcluded trace event — the re-solve here is \
         burner-free and round 6's acceptance rule must keep it — got: {events:?}"
    );
}

/// #461 part (a) round 5, companion to the pin above and renamed to match
/// the mechanism (was `issue_461_burner_penalty_does_not_refuse_sole_
/// producer`, from the reverted LP-penalty design): `avoid_burner_recipes`
/// STEERS when an electric alternative exists, it does not REFUSE — and,
/// unlike a cost penalty, it does not even ATTEMPT a re-solve when none
/// does. `pentapod-egg` is biochamber-only in the game (pure `organic`
/// category, no assembler-tier alternative anywhere in the recipe graph),
/// so no OTHER producer of `pentapod-egg` clears the
/// `category_has_electric_machine` bar — nothing gets excluded, and the
/// FIRST solve's result is returned untouched. Asserted via the trace
/// event's ABSENCE, not just the outcome: a second solve that happened to
/// land on the same machine would pass the outcome check but not this one.
#[test]
fn issue_461_sole_burner_producer_skips_resolve() {
    let _guard = trace::start_trace();
    let inputs = set(&["nutrients", "water"]);
    let sr = solver::solve("pentapod-egg", 0.2, &inputs, "assembling-machine-3")
        .unwrap_or_else(|e| panic!("pentapod-egg solves: {e}"));
    let m = sr
        .machines
        .iter()
        .find(|m| m.recipe == "pentapod-egg")
        .expect("pentapod-egg recipe must appear in its own solve");
    assert_eq!(
        m.entity, "biochamber",
        "pentapod-egg is biochamber-only in the game; got {}",
        m.entity
    );
    let events = trace::drain_events();
    assert!(
        !events.iter().any(|e| matches!(e, TraceEvent::BurnerRecipeExcluded { .. })),
        "no re-solve should have been attempted — pentapod-egg has no electric alternative \
         producer — but got: {events:?}"
    );
}

/// #461 part (a) round 5: the common, unaffected case — a solve that never
/// chose a burner in the first place must perform NO re-solve at all.
/// `avoid_burner_recipes` is a no-op unless `result.machines` actually
/// contains a burner; electronic-circuit from ore (iron-ore, copper-ore,
/// coal) never touches `organic` or any other burner-only category, so
/// this pins the fast path: zero `BurnerRecipeExcluded` events, meaning
/// the netflow LP ran exactly once — the property every all-electric
/// calibration fixture depends on for its LP to stay bit-identical.
#[test]
fn issue_461_no_burner_no_resolve_for_electronic_circuit_from_ore() {
    let _guard = trace::start_trace();
    let inputs = set(&["iron-ore", "copper-ore", "coal"]);
    let sr = solver::solve("electronic-circuit", 10.0, &inputs, "assembling-machine-3")
        .unwrap_or_else(|e| panic!("electronic-circuit solves: {e}"));
    assert!(!sr.machines.is_empty(), "sanity: solve should place machines");
    let events = trace::drain_events();
    assert!(
        !events.iter().any(|e| matches!(e, TraceEvent::BurnerRecipeExcluded { .. })),
        "electronic-circuit from ore never touches a burner machine — no re-solve should \
         have been attempted, but got: {events:?}"
    );
}

/// #461 part (a) round 6, tightened in round 7, re-verified against round
/// 8's bounded fixpoint — the rejection path, and its ACTUAL attempt
/// sequence rather than an assumed one. `avoid_burner_recipes` attempts a
/// re-solve whenever a burner has a TIER-FEASIBLE electric alternative
/// producer, accepting only when `burners_after < burners_before`.
/// `lubricant` off only `{jelly}` is the case that motivated the rule:
/// `biolubricant` (`organic`, biochamber) is genuinely cost-optimal here,
/// and the `lubricant` recipe (`chemistry` → chemical-plant, AM3-feasible)
/// is a real alternative for the same item, so attempt 1 excludes
/// `biolubricant` and re-solves — but that re-solve doesn't land on the
/// simple chemistry chain; it wanders into an 11-machine-type plan
/// (coal-liquefaction, coal-synthesis, sulfuric-acid, …) carrying THREE
/// other biochamber recipes (burnt-spoilage, biosulfur, bioflux) — MORE
/// burners than the one attempt 1 started from (1 → 3), not fewer.
///
/// Per the fixpoint's own rule, a rejected attempt stops the WHOLE loop —
/// it does not examine that rejected re-solve's own burners for a follow-
/// up attempt (only an ACCEPTED attempt's result becomes the new "current
/// best" the next iteration examines). So this is a SINGLE-attempt
/// sequence: attempt 1 only, rejected, fixpoint stops there. Verified
/// empirically (not assumed) before writing this assertion — the
/// receipts, not a hoped-for outcome, are what's pinned here.
/// Consequently `crates/core/tests/e2e.rs`'s
/// `phase0e1_biolubricant_biochamber` fixture is UNCHANGED by round 8: it
/// still lands on biolubricant/biochamber and was not edited.
#[test]
fn issue_461_burner_resolve_rejected_when_still_burner() {
    let _guard = trace::start_trace();
    let inputs = set(&["jelly"]);
    let sr = solver::solve("lubricant", 5.0, &inputs, "assembling-machine-3")
        .unwrap_or_else(|e| panic!("lubricant solves: {e}"));
    let m = sr
        .machines
        .iter()
        .find(|m| m.recipe == "biolubricant")
        .expect("expected the ORIGINAL biolubricant/biochamber result — the re-solve should \
                 have been rejected as still-burner, not replaced this");
    assert_eq!(m.entity, "biochamber", "biolubricant always runs on a biochamber");
    let events = trace::drain_events();
    assert_eq!(
        events.len(),
        1,
        "expected exactly ONE attempt (rejected attempts stop the fixpoint immediately \
         rather than probing further) — got: {events:?}"
    );
    match &events[0] {
        TraceEvent::BurnerRecipeExcluded {
            accepted, attempt, burners_before, burners_after, machines_before, machines_after, ..
        } => {
            assert_eq!(*attempt, 1, "the sole attempt is attempt 1");
            assert!(!accepted, "expected the re-solve to be REJECTED, got accepted=true");
            assert_eq!(*burners_before, 1, "original result has 1 burner (biolubricant)");
            assert_eq!(
                *burners_after,
                Some(3),
                "the re-solve's plan has 3 burners (burnt-spoilage, biosulfur, bioflux) — \
                 MORE than the original, hence rejected"
            );
            assert_eq!(*machines_before, 1, "original plan is exactly the biolubricant row");
            assert_eq!(
                *machines_after,
                Some(12),
                "the rejected re-solve's plan has 12 total machine entries — a much bigger \
                 plan traded for a worse burner count, exactly why there's no cost arbiter \
                 here: it would have been the wrong signal either way"
            );
        }
        other => panic!("expected BurnerRecipeExcluded, got {other:?}"),
    }
}

/// #461 part (a) round 8 — the tier-feasibility check. An "electric
/// alternative" only counts if the caller's `default_machine` tier (for a
/// `GENERAL_CATEGORIES` recipe) or the category's own canonical machine
/// (for a specialised category) can actually run it —
/// `machine_can_run_recipe(machine_for_recipe(other, default_machine),
/// other).is_ok()`. Calls `solver::avoid_burner_recipes` DIRECTLY (it's
/// `pub` since round 8, for exactly this kind of isolated check, and for
/// `sim_export.rs`) with a hand-built `SolverResult` pinning
/// `rocket-fuel-from-jelly` on a biochamber as the sole machine, so the
/// test exercises ONLY the tier-feasibility decision, not the LP's own
/// cost-driven recipe selection (at AM3 the LP would never have chosen
/// this burner in the first place — see
/// `issue_461_no_burner_no_resolve_for_electronic_circuit_from_ore`'s
/// doc comment for the general "never chose one" case). `ammonia-
/// rocket-fuel` (`chemistry-or-cryogenics` → chemical-plant) is excluded
/// so the ONLY remaining candidate is the direct `rocket-fuel` recipe
/// itself (2 ingredients incl. a fluid, `organic-or-assembling` →
/// `GENERAL_CATEGORIES` → resolves to `default_machine`) — the recipe
/// whose OWN tier-feasibility genuinely differs between AM1 and AM3.
///
/// At AM1: `machine_for_recipe_with_palette` resolves the direct recipe
/// to AM1 (the default, empty-palette case), and AM1 has no fluid boxes
/// (`FluidNotSupported`) — not tier-feasible, so NO re-solve is attempted
/// (the `resolve` closure panics if called, proving it never is; zero
/// trace events). At AM3: the same recipe resolves to AM3, which has
/// fluid boxes — tier-feasible, so a re-solve IS attempted (`resolve` is
/// called; exactly one trace event fires). Both calls pass
/// `&MachinePalette::default()` — the companion pin
/// `issue_461_palette_entry_changes_tier_feasibility_verdict` covers the
/// case where a non-default palette entry is what flips the verdict.
#[test]
fn issue_461_tier_feasible_alternative_gates_resolve_attempt() {
    use spaghettio_core::models::{MachineSpec, SolverResult};

    let synthetic_burner_result = || SolverResult {
        machines: vec![MachineSpec {
            entity: "biochamber".to_string(),
            recipe: "rocket-fuel-from-jelly".to_string(),
            count: 1.0,
            ..Default::default()
        }],
        ..Default::default()
    };
    let excluded = set(&["ammonia-rocket-fuel"]);
    let palette = MachinePalette::default();

    let _guard = trace::start_trace();
    let am1_result = solver::avoid_burner_recipes(
        synthetic_burner_result(),
        "rocket-fuel",
        &excluded,
        &palette,
        "assembling-machine-1",
        |_excl| panic!("resolve must not be called at AM1 — the only remaining alternative \
                         (the direct rocket-fuel recipe) is FluidNotSupported on AM1"),
    );
    assert_eq!(
        am1_result.machines[0].entity, "biochamber",
        "no tier-feasible alternative at AM1 — the synthetic result must pass through unchanged"
    );
    let events = trace::drain_events();
    assert!(
        events.is_empty(),
        "expected NO BurnerRecipeExcluded event at AM1 (no re-solve attempted), got: {events:?}"
    );

    let _guard = trace::start_trace();
    let resolve_called = std::cell::Cell::new(false);
    let am3_result = solver::avoid_burner_recipes(
        synthetic_burner_result(),
        "rocket-fuel",
        &excluded,
        &palette,
        "assembling-machine-3",
        |_excl| {
            resolve_called.set(true);
            // The specific re-solve outcome isn't under test here — only
            // that `avoid_burner_recipes` decided to ATTEMPT one. Erroring
            // keeps this test independent of the real solve's numbers.
            Err(SolverError::LpFailed {
                target: "rocket-fuel".to_string(),
                detail: "test stub — intentionally not a real solve".to_string(),
            })
        },
    );
    assert!(
        resolve_called.get(),
        "expected resolve to be CALLED at AM3 — the direct rocket-fuel recipe is \
         tier-feasible there (AM3 has fluid boxes)"
    );
    assert_eq!(
        am3_result.machines[0].entity, "biochamber",
        "the stub resolve errored, so the original result must be what's returned"
    );
    let events = trace::drain_events();
    assert_eq!(
        events.len(),
        1,
        "expected exactly one BurnerRecipeExcluded event (attempt 1, errored -> rejected), \
         got: {events:?}"
    );
}

/// #461 part (a) round 9 — palette-aware feasibility. The gate must
/// resolve a candidate machine the SAME way the real re-solve would: via
/// `machine_for_recipe_with_palette`, honouring the caller's
/// [`MachinePalette`], not just `default_machine`. Same synthetic
/// `rocket-fuel-from-jelly`/biochamber setup as the pin above, `ammonia-
/// rocket-fuel` excluded so the direct `rocket-fuel` recipe (`organic-or-
/// assembling` → `GENERAL_CATEGORIES`) is the only remaining candidate —
/// but this time `default_machine` is AM3 (tier-feasible on its own,
/// per the pin above) while the PALETTE pins `crafting` to AM1. Since
/// `rocket-fuel`'s category is `organic-or-assembling`, not `crafting`,
/// a palette entry keyed on the wrong category must NOT change anything
/// (sanity half); a palette entry keyed on `organic-or-assembling`
/// itself — the category `GENERAL_CATEGORIES` actually dispatches this
/// recipe through — pinning it to AM1 must flip the verdict to
/// infeasible, even though `default_machine` alone says AM3.
#[test]
fn issue_461_palette_entry_changes_tier_feasibility_verdict() {
    use spaghettio_core::models::{MachineSpec, SolverResult};

    let synthetic_burner_result = || SolverResult {
        machines: vec![MachineSpec {
            entity: "biochamber".to_string(),
            recipe: "rocket-fuel-from-jelly".to_string(),
            count: 1.0,
            ..Default::default()
        }],
        ..Default::default()
    };
    let excluded = set(&["ammonia-rocket-fuel"]);

    // Sanity half: a palette entry for an unrelated category (`crafting`)
    // must not affect `organic-or-assembling`'s resolution — AM3 default
    // still applies, so a re-solve IS attempted.
    let mut unrelated_palette = MachinePalette::default();
    unrelated_palette.by_category.insert("crafting".to_string(), "assembling-machine-1".to_string());
    let _guard = trace::start_trace();
    let resolve_called = std::cell::Cell::new(false);
    solver::avoid_burner_recipes(
        synthetic_burner_result(),
        "rocket-fuel",
        &excluded,
        &unrelated_palette,
        "assembling-machine-3",
        |_excl| {
            resolve_called.set(true);
            Err(SolverError::LpFailed {
                target: "rocket-fuel".to_string(),
                detail: "test stub".to_string(),
            })
        },
    );
    assert!(
        resolve_called.get(),
        "a palette entry for `crafting` must not affect the `organic-or-assembling` \
         recipe's resolution — expected resolve to be called (AM3 default applies)"
    );

    // The actual verdict-changing half: pinning `organic-or-assembling`
    // itself to AM1 in the palette must make the gate see the SAME
    // FluidNotSupported infeasibility a plain AM1 default would, even
    // though `default_machine` here is AM3.
    let mut targeted_palette = MachinePalette::default();
    targeted_palette
        .by_category
        .insert("organic-or-assembling".to_string(), "assembling-machine-1".to_string());
    let _guard = trace::start_trace();
    solver::avoid_burner_recipes(
        synthetic_burner_result(),
        "rocket-fuel",
        &excluded,
        &targeted_palette,
        "assembling-machine-3",
        |_excl| panic!("resolve must not be called — the palette pins organic-or-assembling \
                         to AM1, which is FluidNotSupported for the direct rocket-fuel recipe, \
                         even though default_machine here is AM3"),
    );
    let events = trace::drain_events();
    assert!(
        events.is_empty(),
        "expected NO BurnerRecipeExcluded event — the palette entry must override \
         default_machine's AM3 and make the alternative tier-infeasible, got: {events:?}"
    );
}

/// #461 part (a) round 7 — the mixed steerable/unsteerable multi-target
/// case that motivated moving the acceptance rule from "burner-free" to
/// "strictly fewer burners". `solve_multi_with_palette_exclusions_quality_
/// and_modules` is the multi-target family's own entry point (RFC-062,
/// the wasm `solve_multi` boundary's choke point) — a THIRD real call
/// site into `avoid_burner_recipes`, alongside `solve`/`solve_with_
/// palette`/`solve_with_exclusions`'s family and the wasm-facing scalar
/// quality/module family.
///
/// Targets `[rocket-fuel, pentapod-egg]` together off the six-ore set at
/// AM1 (no exclusions): the FIRST solve mixes a STEERABLE burner
/// (`rocket-fuel-from-jelly` — `rocket-fuel` has electric alternatives)
/// with an UNSTEERABLE one (`pentapod-egg` — biochamber-only) in the SAME
/// result, plus whatever nutrient-supply burners pentapod-egg's own
/// demand pulls in. `avoid_burner_recipes` excludes only the recipes that
/// clear the `category_has_electric_machine` bar (`rocket-fuel-from-jelly`
/// among them; `pentapod-egg` itself never does) and re-solves — the
/// re-solve is neither identical nor burner-free (pentapod-egg is still
/// there), but it has STRICTLY FEWER burners than the original (verified:
/// 4 → 2), so round 6's stricter "== 0" rule would have wrongly rejected
/// this improvement; round 7's "< before" rule correctly accepts it.
#[test]
fn issue_461_multi_target_accepts_partial_burner_reduction() {
    let _guard = trace::start_trace();
    let inputs = set(&["iron-ore", "copper-ore", "coal", "stone", "crude-oil", "water"]);
    let targets = vec![("rocket-fuel".to_string(), 1.0), ("pentapod-egg".to_string(), 0.2)];
    let sr = solver::solve_multi_with_palette_exclusions_quality_and_modules(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-1",
        &FxHashSet::default(),
        common::QualityTier::Normal,
        spaghettio_core::module_policy::ModulePolicy::default(),
    )
    .unwrap_or_else(|e| panic!("[rocket-fuel, pentapod-egg] solves: {e}"));

    let rocket_fuel_machine = sr
        .machines
        .iter()
        .find(|m| m.recipe.contains("rocket-fuel"))
        .expect("some rocket-fuel-producing recipe must appear in the accepted result");
    assert!(
        common::needs_electricity(&rocket_fuel_machine.entity),
        "rocket-fuel must be on an electric machine after the accepted re-solve, got {} for {}",
        rocket_fuel_machine.entity,
        rocket_fuel_machine.recipe,
    );

    let pentapod_machine = sr
        .machines
        .iter()
        .find(|m| m.recipe == "pentapod-egg")
        .expect("pentapod-egg recipe must appear in the result");
    assert_eq!(
        pentapod_machine.entity, "biochamber",
        "pentapod-egg is biochamber-only in the game and has no electric alternative — the \
         re-solve must not (and cannot) move it"
    );

    let events = trace::drain_events();
    let event = events
        .iter()
        .find(|e| matches!(e, TraceEvent::BurnerRecipeExcluded { .. }))
        .unwrap_or_else(|| panic!("expected a BurnerRecipeExcluded trace event, got: {events:?}"));
    match event {
        TraceEvent::BurnerRecipeExcluded { accepted, burners_before, burners_after, .. } => {
            assert!(
                *accepted,
                "expected the re-solve to be ACCEPTED (fewer burners, even though not zero)"
            );
            let after = burners_after.unwrap_or_else(|| panic!("accepted implies Some(_)"));
            assert!(
                after < *burners_before,
                "expected burners_after ({after}) < burners_before ({burners_before})"
            );
        }
        other => panic!("expected BurnerRecipeExcluded, got {other:?}"),
    }
}

/// KILL CRITERION 4 — perf. Run explicitly in release:
/// `cargo test --release --manifest-path crates/core/Cargo.toml \
///    --test netflow_regression -- kc4 --ignored --nocapture`
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

// ============================================================================
// Fulgora scrap-economy spike (RFC decision log, 2026-07-11 entries).
// Everything below is additive and behind `NetflowOptions`, which defaults
// both flags to `false` — nothing above this point (or any other default
// solve path) is affected.
// ============================================================================

/// True if `recipe_name` is a "pure voider": exactly one ingredient and one
/// product, both the same item, net-negative. Mirrors netflow.rs's private
/// `is_pure_voider`, reimplemented here against public `recipe_db` data
/// since the original is crate-private and this is a report/test helper.
fn is_pure_voider_recipe(recipe_name: &str) -> bool {
    let Some(recipe) = recipe_db::db().recipes.get(recipe_name) else {
        return false;
    };
    if recipe.ingredients.len() != 1 || recipe.products.len() != 1 {
        return false;
    }
    let ing = &recipe.ingredients[0];
    let prod = &recipe.products[0];
    ing.name == prod.name && prod.amount * prod.probability - ing.amount < 0.0
}

/// Per-item conservation breakdown: produced / externally supplied /
/// consumed by ordinary machines / destroyed by voiders / left as surplus.
/// Report helper only — the invariant itself is `assert_conservation`.
fn print_item_breakdown(r: &SolverResult) {
    use std::collections::BTreeMap;
    #[derive(Default, Clone, Copy)]
    struct Row {
        produced: f64,
        consumed: f64,
        voided: f64,
    }
    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    for m in &r.machines {
        let voider = is_pure_voider_recipe(&m.recipe);
        for f in &m.outputs {
            rows.entry(f.item.clone()).or_default().produced += f.rate * m.count;
        }
        for f in &m.inputs {
            let row = rows.entry(f.item.clone()).or_default();
            if voider {
                row.voided += f.rate * m.count;
            } else {
                row.consumed += f.rate * m.count;
            }
        }
    }
    let external: FxHashSet<String> = r.external_inputs.iter().map(|f| f.item.clone()).collect();
    let surplus: FxHashSet<String> = r.surplus_outputs.iter().map(|f| f.item.clone()).collect();
    let mut items: Vec<String> = rows.keys().cloned().collect();
    for s in external.iter().chain(surplus.iter()) {
        if !items.contains(s) {
            items.push(s.clone());
        }
    }
    items.sort();
    println!(
        "  {:<28} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "item", "produced", "external", "consumed", "voided", "surplus"
    );
    for item in items {
        let row = rows.get(&item).copied().unwrap_or_default();
        let ext = r.external_inputs.iter().find(|f| f.item == item).map(|f| f.rate).unwrap_or(0.0);
        let sur = r.surplus_outputs.iter().find(|f| f.item == item).map(|f| f.rate).unwrap_or(0.0);
        println!(
            "  {item:<28} {:>10.4} {ext:>10.4} {:>10.4} {:>10.4} {sur:>10.4}",
            row.produced, row.consumed, row.voided,
        );
    }
}

fn print_machine_mix(r: &SolverResult) {
    for m in &r.machines {
        let tag = if is_pure_voider_recipe(&m.recipe) {
            " [VOIDER]"
        } else if m.entity == "recycler" {
            " [recycler]"
        } else {
            ""
        };
        println!("  {:>10.4}x {:<34} ({}){tag}", m.count, m.recipe, m.entity);
    }
}

/// Fulgora scrap-economy spike report (RFC decision log, 2026-07-11). Not a
/// kill-criterion gate — run explicitly:
///   cargo test --manifest-path crates/core/Cargo.toml --test netflow_regression \
///       report_fulgora_spike -- --ignored --nocapture
#[test]
#[ignore = "report only — run with --ignored --nocapture"]
fn report_fulgora_spike() {
    let inputs = set(&["scrap", "water"]);
    let default_costs = CostTable::default();
    let spike_costs_1e3 = CostTable { eps_surplus: 1e-3, ..CostTable::default() };
    let spike_costs_1e2 = CostTable { eps_surplus: 1e-2, ..CostTable::default() };
    let opts_novoid = NetflowOptions { allow_recycling: true, allow_voiding: false, ..Default::default() };
    let opts_void = NetflowOptions { allow_recycling: true, allow_voiding: true, ..Default::default() };

    // Item names verified directly against draftsman 3.3.0 / Space Age data
    // before writing this test (see the recipe_db exploration in the RFC
    // spike session) — all three exist under these exact slugs.
    let targets: &[(&str, f64)] = &[
        ("holmium-plate", 1.0),
        ("superconductor", 0.5),
        ("electromagnetic-science-pack", 1.0),
    ];

    for &(target, rate) in targets {
        println!("\n================ {target} @ {rate}/s ================");

        // --- frozen default cost table: surplus mode vs voiding-enabled ---
        let surplus_mode = solve_netflow_with_options(
            target, rate, &inputs, &MachinePalette::default(), "assembling-machine-3",
            &FxHashSet::default(), RecipeScope::Free, &default_costs, &opts_novoid,
        )
        .unwrap_or_else(|e| panic!("{target}: surplus-mode solve failed: {e}"));
        assert_conservation(target, rate, &surplus_mode);

        let voiding_mode = solve_netflow_with_options(
            target, rate, &inputs, &MachinePalette::default(), "assembling-machine-3",
            &FxHashSet::default(), RecipeScope::Free, &default_costs, &opts_void,
        )
        .unwrap_or_else(|e| panic!("{target}: voiding-mode solve failed: {e}"));
        assert_conservation(target, rate, &voiding_mode);

        println!("--- default costs (eps_surplus={}), surplus mode ---", default_costs.eps_surplus);
        print_machine_mix(&surplus_mode);
        print_item_breakdown(&surplus_mode);
        let scrap_rate =
            surplus_mode.external_inputs.iter().find(|f| f.item == "scrap").map(|f| f.rate).unwrap_or(0.0);
        println!("  scrap consumption: {scrap_rate:.4}/s");

        // KEY DELIVERABLE: with the frozen default CostTable, does
        // allow_voiding change anything? Per the RFC's cost-table design
        // (w_available > eps_machine·time > eps_surplus), it must not —
        // surplus stays strictly cheaper than running a voider machine.
        let voider_ran =
            voiding_mode.machines.iter().any(|m| is_pure_voider_recipe(&m.recipe) && m.count > 1e-9);
        println!(
            "--- default costs, voiding enabled: voider active = {voider_ran} \
             (expected: false — surplus beats voider machine cost) ---"
        );
        assert!(
            !voider_ran,
            "{target}: a voider ran under the FROZEN default cost table — eps_surplus is no longer \
             strictly cheaper than eps_machine·time; this is a real regression, not the spike finding"
        );
        assert_eq!(
            surplus_mode.dependency_order, voiding_mode.dependency_order,
            "{target}: allow_voiding changed the recipe set under default pricing — should be a no-op"
        );
        for wm in &surplus_mode.machines {
            let vm = voiding_mode.machines.iter().find(|m| m.recipe == wm.recipe).unwrap();
            assert!(
                (wm.count - vm.count).abs() < 1e-6,
                "{target}: {} machine count changed under default pricing: {} -> {}",
                wm.recipe, wm.count, vm.count
            );
        }
        assert_eq!(
            surplus_mode.surplus_outputs.len(),
            voiding_mode.surplus_outputs.len(),
            "{target}: surplus item count changed under default pricing"
        );

        // --- spike-only elevated eps_surplus: the pricing experiment ---
        for (label, costs) in [("1e-3", &spike_costs_1e3), ("1e-2", &spike_costs_1e2)] {
            let r = solve_netflow_with_options(
                target, rate, &inputs, &MachinePalette::default(), "assembling-machine-3",
                &FxHashSet::default(), RecipeScope::Free, costs, &opts_void,
            );
            match r {
                Ok(res) => {
                    assert_conservation(target, rate, &res);
                    let voider_ran =
                        res.machines.iter().any(|m| is_pure_voider_recipe(&m.recipe) && m.count > 1e-9);
                    println!(
                        "--- eps_surplus={label}: SOLVED, voider active={voider_ran}, surplus items={} ---",
                        res.surplus_outputs.len()
                    );
                    print_machine_mix(&res);
                }
                Err(SolverError::UnsupportedCycle { recipes }) => {
                    // Observed finding (see RFC decision log): admitting
                    // ~310 recycling recipes into the FULL free-selection
                    // graph, combined with a high enough eps_surplus,
                    // makes "craft an ordinary game entity purely to feed
                    // it into its OWN recycling recipe as a byproduct
                    // sink" look profitable — a NEW laundering shape, not
                    // involving voiders at all (reproduces with
                    // allow_voiding=false too). The existing multi-recipe
                    // cycle guard (find_active_cycle_indices,
                    // UnsupportedCycle) correctly refuses it rather than
                    // silently laundering. This is why the pricing
                    // experiment does not report clean full-graph voiding
                    // for these three real targets — see
                    // `voider_disposes_surplus_above_break_even_price` for
                    // proof the underlying LP mechanism is nonetheless
                    // correct once scoped away from this exploit family.
                    println!("--- eps_surplus={label}: REFUSED (UnsupportedCycle) — {recipes} ---");
                }
                Err(e) => panic!("{target} @ eps_surplus={label}: unexpected error: {e}"),
            }
        }
    }

    // --- hand-derived golden: holmium-plate@1/s scrap rate ---
    // holmium-plate needs 20 holmium-solution/craft (1 plate/craft);
    // holmium-solution needs 0.2 holmium-ore per 10 solution/craft
    // (= 0.02 ore/solution); scrap-recycling yields holmium-ore at
    // p=0.01/craft (1 scrap/craft, category recycling-or-hand-crafting).
    //   holmium-ore/s = 20 solution/s * 0.02 ore/solution = 0.4
    //   scrap/s = 0.4 / 0.01 = 40.0
    let r = solve_netflow_with_options(
        "holmium-plate", 1.0, &inputs, &MachinePalette::default(), "assembling-machine-3",
        &FxHashSet::default(), RecipeScope::Free, &default_costs, &opts_void,
    )
    .expect("holmium-plate solves");
    let scrap_rate = r.external_inputs.iter().find(|f| f.item == "scrap").expect("scrap external").rate;
    let hand_derived = 40.0;
    assert!(
        (scrap_rate - hand_derived).abs() / hand_derived < 0.01,
        "holmium-plate@1/s: scrap rate {scrap_rate} not within 1% of hand-derived {hand_derived}"
    );
    println!("\nGOLDEN: holmium-plate@1/s scrap rate = {scrap_rate:.4} (hand-derived: {hand_derived})");
}

/// Determinism (RFC Fulgora spike): double-run byte-compare, KC2's shape
/// scoped to the spike's actual solves (allow_recycling + allow_voiding,
/// all three report targets, default AND spike-priced cost tables).
#[test]
fn fulgora_spike_determinism_double_run() {
    let inputs = set(&["scrap", "water"]);
    let opts = NetflowOptions { allow_recycling: true, allow_voiding: true, ..Default::default() };
    let targets: &[(&str, f64)] =
        &[("holmium-plate", 1.0), ("superconductor", 0.5), ("electromagnetic-science-pack", 1.0)];
    let costs = [CostTable::default(), CostTable { eps_surplus: 1e-3, ..CostTable::default() }];
    let run = || -> String {
        let mut out = String::new();
        for &(target, rate) in targets {
            for c in &costs {
                let r = solve_netflow_with_options(
                    target, rate, &inputs, &MachinePalette::default(), "assembling-machine-3",
                    &FxHashSet::default(), RecipeScope::Free, c, &opts,
                );
                match r {
                    Ok(res) => out.push_str(&serde_json::to_string(&res).unwrap()),
                    Err(e) => out.push_str(&format!("ERR:{e}")),
                }
                out.push('\n');
            }
        }
        out
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "Fulgora spike solves are not deterministic");
}

/// Regression guard for the voiding mechanism itself, isolated from the
/// full-graph elevated-eps_surplus cycle exploit documented in
/// `report_fulgora_spike` by restricting scope to exactly two recipes:
/// scrap-recycling (produces steel-plate as one of its ~12 byproducts) and
/// steel-plate-recycling (a genuine pure voider: 1 steel-plate in, 0.25
/// out, net −0.75/craft — unlike iron-plate, which scrap-recycling never
/// produces, so it can't be used for this isolation).
///
/// Break-even eps_surplus, derived analytically: recycler crafting_speed
/// 0.5, steel-plate-recycling energy 1.0 → machine_time = 2s. Voider cost
/// per net-destroyed unit/s = eps_machine·machine_time / 0.75 =
/// 1e-6·2/0.75 ≈ 2.667e-6. Below that price, accepting surplus
/// (eps_surplus·rate) is cheaper; above it, voiding is cheaper. This test
/// samples one point below (1e-6) and one comfortably above (1e-4) —
/// neither is the frozen default (CostTable::default() uses 1e-8, used by
/// every other test in this file).
#[test]
fn voider_disposes_surplus_above_break_even_price() {
    let inputs = set(&["scrap", "water"]);
    let scope = set(&["scrap-recycling", "steel-plate-recycling"]);
    let opts = NetflowOptions { allow_recycling: true, allow_voiding: true, ..Default::default() };

    let below = solve_netflow_with_options(
        "iron-gear-wheel", 1.0, &inputs, &MachinePalette::default(), "assembling-machine-3",
        &FxHashSet::default(), RecipeScope::Restricted(&scope),
        &CostTable { eps_surplus: 1e-6, ..CostTable::default() }, &opts,
    )
    .expect("below break-even solves");
    assert_conservation("iron-gear-wheel", 1.0, &below);
    let steel_surplus_below =
        below.surplus_outputs.iter().find(|f| f.item == "steel-plate").map(|f| f.rate).unwrap_or(0.0);
    assert!(
        steel_surplus_below > 0.0,
        "expected steel-plate surplus below break-even: {:?}",
        below.surplus_outputs
    );
    assert!(
        !below.machines.iter().any(|m| m.recipe == "steel-plate-recycling" && m.count > 1e-9),
        "voider should be inactive below break-even: {:?}",
        below.machines
    );

    let above = solve_netflow_with_options(
        "iron-gear-wheel", 1.0, &inputs, &MachinePalette::default(), "assembling-machine-3",
        &FxHashSet::default(), RecipeScope::Restricted(&scope),
        &CostTable { eps_surplus: 1e-4, ..CostTable::default() }, &opts,
    )
    .expect("above break-even solves");
    assert_conservation("iron-gear-wheel", 1.0, &above);
    assert!(
        !above.surplus_outputs.iter().any(|f| f.item == "steel-plate"),
        "expected steel-plate fully voided above break-even: {:?}",
        above.surplus_outputs
    );
    let voider = above
        .machines
        .iter()
        .find(|m| m.recipe == "steel-plate-recycling")
        .expect("voider machine present above break-even");
    // Hand-derived: scrap-recycling runs at exactly 2.0 machines (5
    // crafts/s to hit iron-gear-wheel@1/s at p=0.2/craft), producing
    // steel-plate at 5 * 0.04 = 0.2/s. The voider nets −0.375/s per
    // machine (1*0.5 consumed − 1*0.25*0.5 produced), so it must run at
    // 0.2 / 0.375 = 8/15 machines to net exactly zero steel-plate.
    let hand_derived_voider_count = 8.0 / 15.0;
    assert!(
        (voider.count - hand_derived_voider_count).abs() < 1e-6,
        "voider machine count: {} (hand-derived: {hand_derived_voider_count})",
        voider.count
    );

    // No OTHER item's surplus, and no non-voider machine count, changed
    // between the two price points — the price change is scoped to the
    // voider's own decision, not a broader laundering resurgence.
    for f in &below.surplus_outputs {
        if f.item == "steel-plate" {
            continue;
        }
        let a = above.surplus_outputs.iter().find(|g| g.item == f.item).map(|g| g.rate);
        assert_eq!(a, Some(f.rate), "{}: surplus changed unexpectedly", f.item);
    }
    for wm in &below.machines {
        if wm.recipe == "steel-plate-recycling" {
            continue;
        }
        let am = above
            .machines
            .iter()
            .find(|m| m.recipe == wm.recipe)
            .unwrap_or_else(|| panic!("{}: missing above break-even", wm.recipe));
        assert!((wm.count - am.count).abs() < 1e-9, "{}: count changed", wm.recipe);
    }
}
