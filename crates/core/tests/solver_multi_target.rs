//! RFC-062 Phase 1: multi-seed solver.
//! See docs/rfc-062-multi-target-outputs.md — §Solver and kill criterion 2.
//!
//! The DI-coupling guard's dedicated unit test
//! (`di_coupling_guard_suppresses_target_item_coupling`) lives in
//! `crates/core/src/netflow.rs` itself, since it needs white-box access to
//! the private `Column`/`Items`/`detect_di_couplings` types — see that
//! module's `tests` block.

use rustc_hash::FxHashSet;
use spaghettio_core::models::{ItemFlow, SolverResult};
use spaghettio_core::netflow::{
    solve_netflow, solve_netflow_multi, CostTable, RecipeScope,
};
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver::SolverError;
use std::collections::HashMap;

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn machine_count(r: &SolverResult, recipe: &str) -> f64 {
    r.machines
        .iter()
        .find(|m| m.recipe == recipe)
        .unwrap_or_else(|| panic!("no {recipe} machines in result; machines present: {:?}",
            r.machines.iter().map(|m| &m.recipe).collect::<Vec<_>>()))
        .count
}

fn rate_of<'a>(flows: &'a [ItemFlow], item: &str) -> Option<&'a ItemFlow> {
    flows.iter().find(|f| f.item == item)
}

fn summed_rates(flows: &[ItemFlow]) -> HashMap<String, f64> {
    let mut out: HashMap<String, f64> = HashMap::new();
    for f in flows {
        *out.entry(f.item.clone()).or_insert(0.0) += f.rate;
    }
    out
}

/// KC2 (`docs/rfc-062-multi-target-outputs.md` kill criterion 2): the
/// multi-seed LP on the canonical `electronic-circuit@10/s` +
/// `advanced-circuit@3/s` (from ore, AM2) case must reproduce the
/// naive-concatenation per-item machine totals exactly — copper-cable
/// 20.0, NOT the hand-sum shortcut's 16.0 — for every item consumed by
/// more than one target's recipe tree. Numbers mirror the RFC's inlined
/// probe table (Motivation section) and the Phase 0 decision log's
/// re-derivation, confirmed independently here via crafting-speed/energy
/// arithmetic: AM2 speed 0.75, electric-furnace speed 2.0 (smelting is
/// hardcoded off the `default_machine` palette regardless of the AM2
/// passed in).
#[test]
fn kc2_ec_ac_shared_copper_cable_exact() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "water", "crude-oil"]);
    let targets = vec![
        ("electronic-circuit".to_string(), 10.0),
        ("advanced-circuit".to_string(), 3.0),
    ];
    let r = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("EC+AC from ore should solve");

    // Combined EC demand = 10 (own target) + 6 (AC's 2-per-craft ingredient
    // draw at AC's 3 crafts/s) = 16/s. AM2: 0.75/0.5 = 1.5 crafts/s/machine.
    let ec = machine_count(&r, "electronic-circuit");
    assert!(
        (ec - 16.0 / 1.5).abs() < 1e-9,
        "electronic-circuit machines: expected {}, got {ec}",
        16.0 / 1.5
    );

    // Combined copper-cable demand = EC's 3-per-craft * 16 crafts/s (48/s)
    // + AC's 4-per-craft * 3 crafts/s (12/s) = 60/s. AM2: 0.75/0.5 =
    // 1.5 crafts/s/machine, 2 cable/craft = 3/s/machine.
    let cable = machine_count(&r, "copper-cable");
    assert!(
        (cable - 20.0).abs() < 1e-9,
        "copper-cable machines: expected 20.0 (naive-concatenation total, NOT the \
         hand-sum shortcut's 16.0), got {cable}"
    );

    // iron-plate demand = EC's 1-per-craft * 16 crafts/s = 16/s.
    // electric-furnace: speed 2.0, energy 3.2 -> 0.625 crafts/s/machine.
    let iron_plate = machine_count(&r, "iron-plate");
    assert!(
        (iron_plate - 25.6).abs() < 1e-9,
        "iron-plate machines: expected 25.6, got {iron_plate}"
    );

    // copper-plate demand = copper-cable's 1-per-craft * 30 crafts/s (60/s
    // cable / 2 per craft) = 30/s. electric-furnace: 0.625 crafts/s/machine.
    let copper_plate = machine_count(&r, "copper-plate");
    assert!(
        (copper_plate - 48.0).abs() < 1e-9,
        "copper-plate machines: expected 48.0, got {copper_plate}"
    );
}

/// Dependency-order sanity for the coupled KC2 case: electronic-circuit
/// (the shared upstream item) must be resolved before advanced-circuit
/// (its consumer) in `dependency_order`, matching the DFS's target-order
/// seeding (EC requested first).
#[test]
fn kc2_dependency_order_ec_before_ac() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "water", "crude-oil"]);
    let targets = vec![
        ("electronic-circuit".to_string(), 10.0),
        ("advanced-circuit".to_string(), 3.0),
    ];
    let r = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("EC+AC from ore should solve");

    let ec_pos = r
        .dependency_order
        .iter()
        .position(|r| r == "electronic-circuit")
        .expect("electronic-circuit in dependency_order");
    let ac_pos = r
        .dependency_order
        .iter()
        .position(|r| r == "advanced-circuit")
        .expect("advanced-circuit in dependency_order");
    assert!(
        ec_pos < ac_pos,
        "expected electronic-circuit ({ec_pos}) before advanced-circuit ({ac_pos}): {:?}",
        r.dependency_order
    );

    // external_outputs carries both targets, each at its own requested
    // rate (RFC-062 Phase 1 semantics: not netted against each other).
    assert_eq!(r.external_outputs.len(), 2);
    assert_eq!(rate_of(&r.external_outputs, "electronic-circuit").unwrap().rate, 10.0);
    assert_eq!(rate_of(&r.external_outputs, "advanced-circuit").unwrap().rate, 3.0);
}

/// Two targets with non-overlapping recipe trees (`electronic-circuit` and
/// `iron-gear-wheel`, solved from plates so neither pulls in smelting):
/// the multi-target solve's machine mix must equal the union of the two
/// independent single-target solves, with identical per-recipe counts —
/// no merging should occur because nothing is actually shared upstream of
/// the raw external inputs.
#[test]
fn disjoint_targets_equal_independent_sums() {
    let inputs = set(&["iron-plate", "copper-plate"]);

    let ec_only = solve_netflow(
        "electronic-circuit",
        10.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("EC-only should solve");
    let gear_only = solve_netflow(
        "iron-gear-wheel",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("gear-only should solve");

    let targets = vec![
        ("electronic-circuit".to_string(), 10.0),
        ("iron-gear-wheel".to_string(), 5.0),
    ];
    let multi = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("disjoint multi-target should solve");

    // No recipe overlap between the two closures, so the multi-target
    // machine list is exactly the concatenation of the two independent
    // solves' machine lists (same recipes, same counts).
    assert_eq!(
        multi.machines.len(),
        ec_only.machines.len() + gear_only.machines.len(),
        "expected disjoint machine sets to concatenate with no merging"
    );
    for single in [&ec_only, &gear_only] {
        for m in &single.machines {
            let count = machine_count(&multi, &m.recipe);
            assert!(
                (count - m.count).abs() < 1e-9,
                "recipe {}: independent solve had {}, multi-target had {count}",
                m.recipe,
                m.count
            );
        }
    }

    // external_inputs: rates sum across both singles (iron-plate is drawn
    // by both EC and gear-wheel; copper-plate only by EC).
    let expected_inputs = {
        let mut m = summed_rates(&ec_only.external_inputs);
        for (k, v) in summed_rates(&gear_only.external_inputs) {
            *m.entry(k).or_insert(0.0) += v;
        }
        m
    };
    let actual_inputs = summed_rates(&multi.external_inputs);
    assert_eq!(
        expected_inputs.len(),
        actual_inputs.len(),
        "external_inputs item set mismatch: expected {expected_inputs:?}, got {actual_inputs:?}"
    );
    for (item, expected_rate) in &expected_inputs {
        let actual_rate = actual_inputs.get(item).unwrap_or_else(|| {
            panic!("missing external input {item} in multi-target result")
        });
        assert!(
            (actual_rate - expected_rate).abs() < 1e-9,
            "external input {item}: expected {expected_rate}, got {actual_rate}"
        );
    }

    // external_outputs: both targets present at their own requested rates.
    assert_eq!(multi.external_outputs.len(), 2);
    assert_eq!(rate_of(&multi.external_outputs, "electronic-circuit").unwrap().rate, 10.0);
    assert_eq!(rate_of(&multi.external_outputs, "iron-gear-wheel").unwrap().rate, 5.0);
}

/// N=1 equivalence (kill criterion 5): calling the new multi-target entry
/// point with a one-element slice must produce a field-level-identical
/// `SolverResult` to the old scalar `solve_netflow` path. This is a
/// construction guarantee (both bottom out in the same `solve_attempt`
/// call with a one-element `targets` slice), not a coincidence — this test
/// pins it, and the full existing suite passing with zero golden churn is
/// the systemic proof.
#[test]
fn n1_equivalence_multi_matches_scalar() {
    let inputs = set(&["iron-ore", "copper-ore"]);
    let scalar = solve_netflow(
        "electronic-circuit",
        10.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("scalar solve");

    let targets = vec![("electronic-circuit".to_string(), 10.0)];
    let multi = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("one-element multi-target solve");

    assert_solver_results_bit_identical(&scalar, &multi);
}

/// Same equivalence check on a coupled multi-recipe target (advanced
/// circuit from ore — exercises the DI-coupling path and self-loop-free
/// but multi-hop closure) to make sure N=1 identity holds beyond the
/// simplest single-recipe case.
#[test]
fn n1_equivalence_holds_on_multi_hop_target() {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "water", "crude-oil"]);
    let scalar = solve_netflow(
        "advanced-circuit",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("scalar solve");

    let targets = vec![("advanced-circuit".to_string(), 5.0)];
    let multi = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("one-element multi-target solve");

    assert_solver_results_bit_identical(&scalar, &multi);
}

/// Duplicate-item targets (RFC-062 Phase 1 decision log): requesting the
/// same item twice sums the rates into one demand row and one
/// `external_outputs` entry, rather than erroring or emitting two entries.
/// Splitting a single-target rate across two duplicate requests must
/// reproduce the equivalent single request exactly.
#[test]
fn duplicate_target_item_rates_are_summed() {
    let inputs = set(&["iron-plate", "copper-plate"]);
    let combined = solve_netflow(
        "electronic-circuit",
        10.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("combined-rate solve");

    let targets = vec![
        ("electronic-circuit".to_string(), 4.0),
        ("electronic-circuit".to_string(), 6.0),
    ];
    let split = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("split-rate solve");

    assert_eq!(
        split.external_outputs.len(),
        1,
        "duplicate target item must collapse to one external_outputs entry, got {:?}",
        split.external_outputs
    );
    assert_eq!(split.external_outputs[0].item, "electronic-circuit");
    assert_eq!(split.external_outputs[0].rate, 10.0);

    assert_solver_results_bit_identical(&combined, &split);
}

/// Typed-refusal review under multi-target: a machine-incompatibility
/// error for one of two targets must still surface as the same typed
/// error it would under single-target solving — not be masked or dropped
/// because the OTHER target is perfectly satisfiable. Mirrors
/// `solver::tests::am1_palette_for_advanced_circuit_returns_incompatible_error`,
/// extended with a second, unrelated, trivially-satisfiable target
/// (`iron-gear-wheel`).
#[test]
fn multi_target_incompatible_machine_error_not_masked_by_other_target() {
    let available = set(&[
        "iron-plate",
        "copper-plate",
        "plastic-bar",
        "electronic-circuit",
    ]);
    let mut palette = MachinePalette::default();
    palette.by_category.insert("electronics".into(), "assembling-machine-1".into());

    let targets = vec![
        ("advanced-circuit".to_string(), 1.0),
        ("iron-gear-wheel".to_string(), 5.0),
    ];
    let err = solve_netflow_multi(
        &targets,
        &available,
        &palette,
        "assembling-machine-3",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect_err("AM1-pinned advanced-circuit should still be rejected under multi-target");

    match err {
        SolverError::IncompatibleMachine { machine, .. } => {
            assert_eq!(machine, "assembling-machine-1");
        }
        other => panic!("expected IncompatibleMachine, got {other:?}"),
    }
}

/// Field-level equality between two `SolverResult`s (neither type derives
/// `PartialEq` — deliberately manual so f64 fields are compared bit-exact
/// via `to_bits()`, proving byte-identical output, not merely
/// approximately-equal).
fn assert_solver_results_bit_identical(a: &SolverResult, b: &SolverResult) {
    assert_eq!(a.machines.len(), b.machines.len(), "machines.len()");
    for (ma, mb) in a.machines.iter().zip(b.machines.iter()) {
        assert_eq!(ma.entity, mb.entity, "machine entity");
        assert_eq!(ma.recipe, mb.recipe, "machine recipe");
        assert_eq!(
            ma.count.to_bits(),
            mb.count.to_bits(),
            "machine {} count: {} vs {}",
            ma.recipe,
            ma.count,
            mb.count
        );
        assert_eq!(ma.voider, mb.voider, "machine {} voider", ma.recipe);
        assert_item_flows_bit_identical(&ma.inputs, &mb.inputs, &format!("{} inputs", ma.recipe));
        assert_item_flows_bit_identical(&ma.outputs, &mb.outputs, &format!("{} outputs", ma.recipe));
        assert_eq!(ma.self_loop.len(), mb.self_loop.len(), "machine {} self_loop len", ma.recipe);
        assert_eq!(ma.game_modules.len(), mb.game_modules.len(), "machine {} game_modules len", ma.recipe);
    }
    assert_item_flows_bit_identical(&a.external_inputs, &b.external_inputs, "external_inputs");
    assert_item_flows_bit_identical(&a.external_outputs, &b.external_outputs, "external_outputs");
    assert_item_flows_bit_identical(&a.surplus_outputs, &b.surplus_outputs, "surplus_outputs");
    assert_eq!(a.dependency_order, b.dependency_order, "dependency_order");
    assert_eq!(a.di_couplings.len(), b.di_couplings.len(), "di_couplings.len()");
    for (ca, cb) in a.di_couplings.iter().zip(b.di_couplings.iter()) {
        assert_eq!(ca.producer_recipe, cb.producer_recipe, "di_coupling producer_recipe");
        assert_eq!(ca.consumer_recipe, cb.consumer_recipe, "di_coupling consumer_recipe");
        assert_eq!(ca.item, cb.item, "di_coupling item");
        assert_eq!(
            ca.producer_count.to_bits(),
            cb.producer_count.to_bits(),
            "di_coupling {} producer_count",
            ca.item
        );
        assert_eq!(
            ca.consumer_count.to_bits(),
            cb.consumer_count.to_bits(),
            "di_coupling {} consumer_count",
            ca.item
        );
    }
}

fn assert_item_flows_bit_identical(a: &[ItemFlow], b: &[ItemFlow], label: &str) {
    assert_eq!(a.len(), b.len(), "{label}: item flow count differs ({a:?} vs {b:?})");
    for (fa, fb) in a.iter().zip(b.iter()) {
        assert_eq!(fa.item, fb.item, "{label}: item name");
        assert_eq!(
            fa.rate.to_bits(),
            fb.rate.to_bits(),
            "{label}: {} rate {} vs {}",
            fa.item,
            fa.rate,
            fb.rate
        );
        assert_eq!(fa.is_fluid, fb.is_fluid, "{label}: {} is_fluid", fa.item);
        assert_eq!(fa.module_id, fb.module_id, "{label}: {} module_id", fa.item);
    }
}
