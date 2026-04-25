//! Demand-partitioning of multi-consumer items.
//!
//! Implements the outer pass of `LayoutStrategy::PartitionedPerConsumer`
//! from `docs/rfp-modular-production.md`. PR1 of Phase 1 introduced the
//! dispatcher + utilization helpers; PR2 (this file's full algorithm)
//! actually splits multi-consumer items into K modules at the
//! `SolverResult` level, before placement and lane planning.
//!
//! Granularity note. The RFP defines a "consumer" as one consuming
//! recipe-row. PR2 partitions at *recipe* granularity instead — if a
//! recipe is split across multiple rows by the placer's throughput
//! heuristic, all those rows share the same module. This is a
//! conservative under-counting of K (never over-partitioning) and is
//! sufficient for the motivating `advanced_circuit` case (K=2 for
//! copper-cable: one module for `electronic-circuit`, one for
//! `advanced-circuit`). True per-row K can be a follow-up if any
//! stress case needs it.
//!
//! Fluids carve-out. Pipe networks merge freely; per-lane identity
//! doesn't apply. The partitioner skips items with `is_fluid = true`
//! (RFP "Fluids" section).
//!
//! See K0-1, K1-1, K1-2, K1-3, K1-4 in the RFP for the gates this code
//! upholds.

use rustc_hash::FxHashMap;

use crate::models::{ItemFlow, MachineSpec, SolverResult};
use crate::trace::{self, TraceEvent};

/// Per-belt-tier per-lane (belt-side) capacity in items/s. Mirrors
/// `LANE_CAPACITY_TABLE` in `lane_planner.rs` — these are *per-side*
/// rates, not full-belt rates. A yellow belt has two sides each
/// carrying 7.5/s; the bus engine treats each `BusLane` as one
/// side, so the partitioner must use the same per-side numbers
/// when sizing modules. Single source of truth is awkward today
/// because the planner's table is private; the partitioner only
/// needs the numeric ceilings.
const PER_LANE_CAPACITY: &[(&str, f64)] = &[
    ("transport-belt", 7.5),
    ("fast-transport-belt", 15.0),
    ("express-transport-belt", 22.5),
];

/// The 75% over-provisioning ceiling from the RFP's "Load-bearing
/// assumption" section. Lanes above this fraction of belt capacity are
/// assumed to suffer from per-machine timing jitter, and the
/// partitioner refuses to allocate them.
pub const UTILIZATION_CEILING: f64 = 0.75;

/// Per-lane (belt-side) capacity for the chosen belt tier. Falls back
/// to the fastest tier when `max_belt_tier` is `None` (matches the
/// lane planner's behaviour). Per-side, not full-belt: a yellow belt
/// has two sides each carrying 7.5/s.
pub fn lane_capacity(max_belt_tier: Option<&str>) -> f64 {
    let default_cap = PER_LANE_CAPACITY.last().map(|(_, c)| *c).unwrap_or(22.5);
    match max_belt_tier {
        Some(tier) => PER_LANE_CAPACITY
            .iter()
            .find(|(name, _)| *name == tier)
            .map(|(_, c)| *c)
            .unwrap_or(default_cap),
        None => default_cap,
    }
}

/// `lane_rate / (capacity * 0.75)` — the saturation fraction relative
/// to the utilization ceiling. Returns `> 1.0` when the lane busts the
/// 75% gate. Phase 1 partitioner emits
/// `TraceEvent::PartitionRejectedByUtilization` and produces an invalid
/// layout when this happens, instead of silently falling back to
/// `Pooled`.
pub fn lane_utilization(lane_rate: f64, max_belt_tier: Option<&str>) -> f64 {
    let cap = lane_capacity(max_belt_tier);
    if cap <= 0.0 {
        return f64::INFINITY;
    }
    lane_rate / (cap * UTILIZATION_CEILING)
}

/// Per-item count of consuming recipe-rows. The partitioner uses this
/// to decide which items need K>1 modules under
/// `PartitionedPerConsumer`. Iteration over `solver_result.machines` is
/// the right unit because the placer turns each `MachineSpec` into one
/// or more `RowSpan` instances; the count of unique consuming
/// recipes is a conservative under-estimate of consuming rows
/// (post-`placer.rs` row-splitting can multiply this further). For PR1's
/// K=1-vs-K>1 fan-out check, the conservative count is sufficient
/// because it can only over-detect K=1 cases (false negatives for
/// partitioning), never the other way around.
pub fn consumers_per_item(solver_result: &SolverResult) -> FxHashMap<String, u32> {
    let mut counts: FxHashMap<String, u32> = FxHashMap::default();
    for m in &solver_result.machines {
        for inp in &m.inputs {
            *counts.entry(inp.item.clone()).or_insert(0) += 1;
        }
    }
    counts
}

/// Items that the partitioner would produce K>1 modules for. Under
/// `PartitionedPerConsumer`, these are the items whose handling diverges
/// from `Pooled`; PR1 of Phase 1 does not yet support them.
pub fn multi_consumer_items(solver_result: &SolverResult) -> Vec<String> {
    let mut items: Vec<String> = consumers_per_item(solver_result)
        .into_iter()
        .filter(|(_, k)| *k > 1)
        .map(|(item, _)| item)
        .collect();
    items.sort();
    items
}

/// Per-recipe partition assignment. Under `PartitionedPerConsumer`,
/// item X with K consuming recipes gets K modules, indexed by the
/// recipe name they serve.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleAssignment {
    pub item: String,
    pub module_id: u32,
    /// Recipe consuming from this module. One module per consuming
    /// recipe under PR2's recipe-level granularity.
    pub consumer_recipe: String,
    /// Total flow rate of `item` produced by (and consumed from) this
    /// module, in items/s.
    pub rate: f64,
    /// Lane count required by this module: `ceil(rate / per_lane_cap)`.
    /// Driven by `max_belt_tier`.
    pub lane_count: u32,
    /// Per-lane saturation fraction relative to the 75% utilization
    /// ceiling: `(rate / lane_count) / (per_lane_cap * 0.75)`. The
    /// rate is divided across `lane_count` belt-sides first, then
    /// compared to the ceiling on a single side. `> 1.0` means at
    /// least one side busts the gate; the partitioner emits
    /// `PartitionRejectedByUtilization` and produces an invalid
    /// layout. See K1-2 in the RFP.
    pub utilization: f64,
}

/// Materialised partition plan for a `SolverResult`. Empty under
/// `LayoutStrategy::Pooled`; populated under `PartitionedPerConsumer`
/// when there is at least one item with K ≥ 2 consuming recipes.
#[derive(Debug, Clone, Default)]
pub struct PartitionPlan {
    pub modules: Vec<ModuleAssignment>,
    /// Items where the 75% utilization gate tripped. Layout still
    /// produces, but the warning surfaces in trace output and a
    /// validator-style message gets attached to `LayoutResult.warnings`.
    pub utilization_violations: Vec<ModuleAssignment>,
}

impl PartitionPlan {
    /// Returns the empty plan — equivalent to `Pooled` semantics.
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Look up a module by `(item, consumer_recipe)`. Returns `None`
    /// if the item is not partitioned (i.e. K=1 or fluid).
    pub fn module_for_consumer(&self, item: &str, consumer_recipe: &str) -> Option<&ModuleAssignment> {
        self.modules
            .iter()
            .find(|m| m.item == item && m.consumer_recipe == consumer_recipe)
    }

    /// Number of modules allocated for `item`. `0` when not
    /// partitioned (treat as Pooled / module_id 0).
    pub fn modules_for_item(&self, item: &str) -> u32 {
        self.modules.iter().filter(|m| m.item == item).count() as u32
    }
}

/// Build a partition plan for the given solver result. Empty under
/// `Pooled`; populated under `PartitionedPerConsumer`.
///
/// Algorithm (PR2 of Phase 1):
///   1. For each output item across `solver_result.machines`, count
///      distinct consuming recipes (excluding fluids).
///   2. For items with K ≥ 2 consumers, allocate one module per
///      consumer. Modules are deterministic: sorted by consumer
///      recipe name, indexed `0..K`.
///   3. Each module's rate = the consumer recipe's per-machine
///      consumption × consumer machine count.
///   4. Lane count = ceil(rate / per_lane_capacity(belt_tier)).
///   5. Utilization = rate / (lane_count * per_lane_capacity * 0.75).
///      The partitioner does *not* veto on overshoot — it produces
///      the plan and surfaces a `PartitionRejectedByUtilization`
///      trace event so the user sees the strategy doesn't fit (RFP's
///      "no silent downgrade" stance).
pub fn plan_partitioning(
    solver_result: &SolverResult,
    strategy: crate::bus::layout::LayoutStrategy,
    max_belt_tier: Option<&str>,
) -> PartitionPlan {
    use crate::bus::layout::LayoutStrategy;
    if !matches!(strategy, LayoutStrategy::PartitionedPerConsumer | LayoutStrategy::PartitionedDecomposed) {
        return PartitionPlan::empty();
    }

    // Map each output item to the list of consumer (recipe, rate) pairs.
    // Rate is the consumer's *consumption* of this item, not the
    // consumer's production rate.
    let mut consumers_of: FxHashMap<String, Vec<(String, f64, bool)>> = FxHashMap::default();
    for m in &solver_result.machines {
        for inp in &m.inputs {
            consumers_of
                .entry(inp.item.clone())
                .or_default()
                .push((m.recipe.clone(), inp.rate * m.count, inp.is_fluid));
        }
    }

    let cap = lane_capacity(max_belt_tier);
    let belt_tier_label = max_belt_tier.unwrap_or("express-transport-belt").to_string();

    // Collect items in a deterministic order so module_id assignment
    // is reproducible across builds.
    let mut items: Vec<&String> = consumers_of.keys().collect();
    items.sort();

    let mut plan = PartitionPlan::empty();
    for item in items {
        let consumers = &consumers_of[item];
        // Fluids carve-out: pipe networks merge freely.
        if consumers.first().is_some_and(|(_, _, is_fluid)| *is_fluid) {
            continue;
        }
        // Aggregate by recipe name (a single recipe may appear in
        // multiple `MachineSpec`s in odd cases; treat them as one
        // consumer). PR2's granularity is recipe-level.
        let mut by_recipe: FxHashMap<String, f64> = FxHashMap::default();
        for (recipe, rate, _) in consumers {
            *by_recipe.entry(recipe.clone()).or_insert(0.0) += rate;
        }
        if by_recipe.len() < 2 {
            continue; // K=1 or K=0 → Pooled-equivalent.
        }

        // Sort recipes for deterministic module_id assignment.
        let mut recipes: Vec<(String, f64)> = by_recipe.into_iter().collect();
        recipes.sort_by(|a, b| a.0.cmp(&b.0));

        for (module_id, (recipe, rate)) in recipes.into_iter().enumerate() {
            let lane_count = (rate / cap).ceil().max(1.0) as u32;
            let per_lane_rate = rate / lane_count as f64;
            let utilization = per_lane_rate / (cap * UTILIZATION_CEILING);
            let module = ModuleAssignment {
                item: item.clone(),
                module_id: module_id as u32,
                consumer_recipe: recipe,
                rate,
                lane_count,
                utilization,
            };
            if utilization > 1.0 {
                trace::emit(TraceEvent::PartitionRejectedByUtilization {
                    item: module.item.clone(),
                    module_id: module.module_id,
                    lane_util: utilization,
                    belt_tier: belt_tier_label.clone(),
                });
                plan.utilization_violations.push(module.clone());
            }
            plan.modules.push(module);
        }

        // Emit one ModulePartitioned per partitioned item. The trace
        // captures the per-module lane-count vector so the snapshot
        // debugger can correlate module_ids to lanes.
        let lanes_per_module: Vec<usize> = plan
            .modules
            .iter()
            .filter(|m| m.item == *item)
            .map(|m| m.lane_count as usize)
            .collect();
        trace::emit(TraceEvent::ModulePartitioned {
            item: item.clone(),
            modules: lanes_per_module.len() as u32,
            lanes_per_module,
        });
    }

    plan
}

/// Apply a partition plan to a solver result, producing a transformed
/// copy where:
///   - The producer `MachineSpec` for each partitioned item is split
///     into K sibling specs (one per module), each with its
///     `outputs[0].module_id` set to the module index. Each sibling's
///     machine count is proportional to its module's rate.
///   - Consumer `MachineSpec`s have their `inputs[i].module_id` set
///     to the module they tap from.
///
/// Items not in the plan pass through unchanged (module_id = 0).
pub fn apply_partition_plan(solver_result: &SolverResult, plan: &PartitionPlan) -> SolverResult {
    if plan.is_empty() {
        return solver_result.clone();
    }

    // Build a quick `(item, recipe) → module_id` index for consumer-side rewriting.
    let mut consumer_module: FxHashMap<(String, String), u32> = FxHashMap::default();
    for m in &plan.modules {
        consumer_module.insert((m.item.clone(), m.consumer_recipe.clone()), m.module_id);
    }
    // And `(item) → list-of-modules` for producer-side splitting.
    let mut item_modules: FxHashMap<String, Vec<&ModuleAssignment>> = FxHashMap::default();
    for m in &plan.modules {
        item_modules.entry(m.item.clone()).or_default().push(m);
    }

    let mut new_machines: Vec<MachineSpec> = Vec::with_capacity(solver_result.machines.len());
    for spec in &solver_result.machines {
        // Determine which output item, if any, gets partitioned. PR2
        // assumes a single non-fluid product per recipe (the codebase's
        // existing assumption); fluid by-products are unaffected.
        let primary_solid_idx = spec
            .outputs
            .iter()
            .position(|o| !o.is_fluid && item_modules.contains_key(&o.item));

        let modules = primary_solid_idx
            .and_then(|idx| item_modules.get(&spec.outputs[idx].item))
            .cloned()
            .unwrap_or_default();

        if modules.len() < 2 {
            // Not partitioned. Rewrite consumer-side input module_ids in place.
            let mut new_spec = spec.clone();
            for inp in &mut new_spec.inputs {
                if let Some(&module_id) = consumer_module.get(&(inp.item.clone(), spec.recipe.clone())) {
                    inp.module_id = module_id;
                }
            }
            new_machines.push(new_spec);
            continue;
        }

        // Partitioned. Split this spec into one sibling per module,
        // proportional to module rate.
        let total_rate: f64 = modules.iter().map(|m| m.rate).sum();
        let n_modules = modules.len();
        for module in modules {
            let share = if total_rate > 0.0 { module.rate / total_rate } else { 1.0 / n_modules as f64 };
            let mut sibling = spec.clone();
            sibling.count = spec.count * share;
            // Tag every output (the partitioned product + any fluid
            // by-products) with this module_id so downstream identity
            // is consistent.
            for out in &mut sibling.outputs {
                out.module_id = module.module_id;
                // per-machine `out.rate` unchanged; the sibling's
                // share comes from `count`.
            }
            // Inputs get their module_id rewritten too — for the
            // consumer-side lookup, but here it represents the
            // producer's own consumption of upstream ingredients.
            // Recursive partitioning (RFP step 6) is out of scope
            // for PR2; upstream ingredients stay at module_id=0.
            for inp in &mut sibling.inputs {
                inp.module_id = 0;
            }
            new_machines.push(sibling);
        }
    }

    SolverResult {
        machines: new_machines,
        external_inputs: solver_result.external_inputs.clone(),
        external_outputs: solver_result.external_outputs.clone(),
        dependency_order: solver_result.dependency_order.clone(),
    }
}

/// Convenience helper: an `ItemFlow` constructor that fills in `module_id: 0`.
/// Used by call sites that don't care about partitioning.
pub fn pooled_flow(item: impl Into<String>, rate: f64, is_fluid: bool) -> ItemFlow {
    ItemFlow { item: item.into(), rate, is_fluid, module_id: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::layout::LayoutStrategy;
    use crate::models::MachineSpec;

    #[test]
    fn lane_capacity_falls_back_to_fastest_tier() {
        // Per-side rates (matches `lane_planner::LANE_CAPACITY_TABLE`).
        assert_eq!(lane_capacity(None), 22.5);
        assert_eq!(lane_capacity(Some("transport-belt")), 7.5);
        assert_eq!(lane_capacity(Some("fast-transport-belt")), 15.0);
        assert_eq!(lane_capacity(Some("express-transport-belt")), 22.5);
        // Unknown tier → fastest.
        assert_eq!(lane_capacity(Some("not-a-belt")), 22.5);
    }

    #[test]
    fn utilization_ceiling_at_75_percent() {
        // Yellow belt: 7.5/s per side × 0.75 = 5.625 ceiling.
        assert!((lane_utilization(5.625, Some("transport-belt")) - 1.0).abs() < 1e-9);
        // Just over → > 1.0.
        assert!(lane_utilization(5.7, Some("transport-belt")) > 1.0);
        // Just under → < 1.0.
        assert!(lane_utilization(5.5, Some("transport-belt")) < 1.0);
    }

    fn flow(item: &str, rate: f64) -> ItemFlow {
        ItemFlow { item: item.to_string(), rate, is_fluid: false, module_id: 0 }
    }

    fn fluid(item: &str, rate: f64) -> ItemFlow {
        ItemFlow { item: item.to_string(), rate, is_fluid: true, module_id: 0 }
    }

    fn machine(recipe: &str, count: f64, inputs: Vec<ItemFlow>, outputs: Vec<ItemFlow>) -> MachineSpec {
        MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: recipe.to_string(),
            count,
            inputs,
            outputs,
        }
    }

    /// Pooled strategy → empty plan regardless of consumer count.
    #[test]
    fn pooled_strategy_returns_empty_plan() {
        let solver_result = SolverResult {
            machines: vec![
                machine("copper-cable", 4.0, vec![flow("copper-plate", 1.0)], vec![flow("copper-cable", 2.0)]),
                machine("electronic-circuit", 2.0, vec![flow("copper-cable", 3.0), flow("iron-plate", 1.0)], vec![flow("electronic-circuit", 1.0)]),
                machine("advanced-circuit", 1.0, vec![flow("copper-cable", 4.0), flow("electronic-circuit", 2.0)], vec![flow("advanced-circuit", 1.0)]),
            ],
            external_inputs: vec![flow("copper-plate", 4.0), flow("iron-plate", 2.0)],
            external_outputs: vec![flow("advanced-circuit", 1.0)],
            dependency_order: vec!["copper-cable".to_string(), "electronic-circuit".to_string(), "advanced-circuit".to_string()],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::Pooled, None);
        assert!(plan.is_empty());
    }

    /// K=1 case (single consumer of every intermediate) → empty plan.
    #[test]
    fn k_eq_1_returns_empty_plan() {
        let solver_result = SolverResult {
            machines: vec![
                machine("iron-gear-wheel", 5.0, vec![flow("iron-plate", 2.0)], vec![flow("iron-gear-wheel", 1.0)]),
            ],
            external_inputs: vec![flow("iron-plate", 10.0)],
            external_outputs: vec![flow("iron-gear-wheel", 5.0)],
            dependency_order: vec!["iron-gear-wheel".to_string()],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedPerConsumer, None);
        assert!(plan.is_empty(), "K=1 should not partition");
    }

    /// K=2 case (advanced_circuit copper-cable shape) → 2 modules,
    /// indexed by sorted recipe name.
    #[test]
    fn advanced_circuit_copper_cable_partitions_into_two_modules() {
        // copper-cable consumed by both electronic-circuit (3.0/s) and
        // advanced-circuit (4.0/s). Two modules expected.
        let solver_result = SolverResult {
            machines: vec![
                machine("copper-cable", 4.0, vec![flow("copper-plate", 1.0)], vec![flow("copper-cable", 2.0)]),
                machine("electronic-circuit", 3.0, vec![flow("copper-cable", 3.0), flow("iron-plate", 1.0)], vec![flow("electronic-circuit", 1.0)]),
                machine("advanced-circuit", 2.0, vec![flow("copper-cable", 4.0), flow("electronic-circuit", 2.0)], vec![flow("advanced-circuit", 1.0)]),
            ],
            external_inputs: vec![flow("copper-plate", 4.0), flow("iron-plate", 3.0)],
            external_outputs: vec![flow("advanced-circuit", 2.0)],
            dependency_order: vec!["copper-cable".to_string(), "electronic-circuit".to_string(), "advanced-circuit".to_string()],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedPerConsumer, Some("transport-belt"));
        let cu_modules: Vec<_> = plan.modules.iter().filter(|m| m.item == "copper-cable").collect();
        assert_eq!(cu_modules.len(), 2, "expected 2 modules for copper-cable");
        // Sorted by recipe name: advanced-circuit first, electronic-circuit second.
        assert_eq!(cu_modules[0].consumer_recipe, "advanced-circuit");
        assert_eq!(cu_modules[0].module_id, 0);
        assert_eq!(cu_modules[1].consumer_recipe, "electronic-circuit");
        assert_eq!(cu_modules[1].module_id, 1);
        // Rates: AC = 4 × 2 = 8/s, EC = 3 × 3 = 9/s.
        assert!((cu_modules[0].rate - 8.0).abs() < 1e-9);
        assert!((cu_modules[1].rate - 9.0).abs() < 1e-9);
        // electronic-circuit and advanced-circuit themselves have K=1 → not partitioned.
        assert!(plan.modules.iter().all(|m| m.item == "copper-cable"));
    }

    /// Fluid items stay pooled even with K ≥ 2 consumers.
    #[test]
    fn fluids_stay_pooled() {
        let solver_result = SolverResult {
            machines: vec![
                machine("plastic-bar", 4.0, vec![flow("coal", 1.0), fluid("petroleum-gas", 20.0)], vec![flow("plastic-bar", 2.0)]),
                machine("sulfuric-acid", 2.0, vec![flow("iron-plate", 1.0), flow("sulfur", 5.0), fluid("water", 100.0)], vec![fluid("sulfuric-acid", 50.0)]),
                machine("explosives", 1.0, vec![flow("coal", 1.0), flow("sulfur", 1.0), fluid("water", 10.0)], vec![flow("explosives", 1.0)]),
            ],
            external_inputs: vec![fluid("petroleum-gas", 80.0), fluid("water", 210.0), flow("coal", 5.0), flow("sulfur", 11.0), flow("iron-plate", 2.0)],
            external_outputs: vec![flow("plastic-bar", 8.0), fluid("sulfuric-acid", 100.0), flow("explosives", 1.0)],
            dependency_order: vec![],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedPerConsumer, None);
        // water has 2 consumers but is fluid → not partitioned.
        assert!(plan.modules.iter().all(|m| m.item != "water"));
        // coal has 2 consumers (plastic-bar, explosives) and is solid → partitioned.
        let coal_modules: Vec<_> = plan.modules.iter().filter(|m| m.item == "coal").collect();
        assert_eq!(coal_modules.len(), 2);
    }

    /// `apply_partition_plan` splits the producer MachineSpec
    /// proportionally and tags consumer inputs with module_id.
    #[test]
    fn apply_plan_splits_producer_and_tags_consumer_inputs() {
        let solver_result = SolverResult {
            machines: vec![
                machine("copper-cable", 4.0, vec![flow("copper-plate", 1.0)], vec![flow("copper-cable", 2.0)]),
                machine("electronic-circuit", 3.0, vec![flow("copper-cable", 3.0), flow("iron-plate", 1.0)], vec![flow("electronic-circuit", 1.0)]),
                machine("advanced-circuit", 2.0, vec![flow("copper-cable", 4.0), flow("electronic-circuit", 2.0)], vec![flow("advanced-circuit", 1.0)]),
            ],
            external_inputs: vec![flow("copper-plate", 4.0), flow("iron-plate", 3.0)],
            external_outputs: vec![flow("advanced-circuit", 2.0)],
            dependency_order: vec![],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedPerConsumer, Some("transport-belt"));
        let partitioned = apply_partition_plan(&solver_result, &plan);

        // Copper-cable producer was 1 spec; should now be 2 (one per module).
        let cc_specs: Vec<&MachineSpec> = partitioned.machines.iter().filter(|m| m.recipe == "copper-cable").collect();
        assert_eq!(cc_specs.len(), 2);
        // Each split has module_id on its output.
        let mut module_ids: Vec<u32> = cc_specs.iter().map(|s| s.outputs[0].module_id).collect();
        module_ids.sort();
        assert_eq!(module_ids, vec![0, 1]);
        // Counts are proportional to module rate (AC=8/s, EC=9/s; total=17). Total preserved.
        let total_count: f64 = cc_specs.iter().map(|s| s.count).sum();
        assert!((total_count - 4.0).abs() < 1e-9);

        // electronic-circuit consumer input.copper-cable.module_id should match the EC module's id (1).
        let ec_spec = partitioned.machines.iter().find(|m| m.recipe == "electronic-circuit").unwrap();
        let cc_input = ec_spec.inputs.iter().find(|i| i.item == "copper-cable").unwrap();
        assert_eq!(cc_input.module_id, 1, "EC input.copper-cable.module_id should be 1");

        // advanced-circuit consumer input.copper-cable.module_id should be 0.
        let ac_spec = partitioned.machines.iter().find(|m| m.recipe == "advanced-circuit").unwrap();
        let cc_input = ac_spec.inputs.iter().find(|i| i.item == "copper-cable").unwrap();
        assert_eq!(cc_input.module_id, 0);
    }
}
