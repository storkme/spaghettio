//! Demand-partitioning of multi-consumer items.
//!
//! Implements the outer pass of `LayoutStrategy::PartitionedDecomposed`
//! from `docs/rfp-modular-production.md`. PR1 of Phase 1 introduced the
//! dispatcher + utilization helpers; PR2 (this file's full algorithm)
//! actually splits multi-consumer items into K modules at the
//! `SolverResult` level, before placement and lane planning. Phase 2's
//! decomposition pass adds subtree sharding when a single module still
//! exceeds the 8-lane cap.
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

use rustc_hash::{FxHashMap, FxHashSet};

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
/// `PartitionedDecomposed`. Iteration over `solver_result.machines` is
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
/// `PartitionedDecomposed`, these are the items whose handling diverges
/// from `Pooled`.
pub fn multi_consumer_items(solver_result: &SolverResult) -> Vec<String> {
    let mut items: Vec<String> = consumers_per_item(solver_result)
        .into_iter()
        .filter(|(_, k)| *k > 1)
        .map(|(item, _)| item)
        .collect();
    items.sort();
    items
}

/// Per-recipe partition assignment. Under `PartitionedDecomposed`,
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
/// `LayoutStrategy::Pooled`; populated under `PartitionedDecomposed`
/// when there is at least one item with K ≥ 2 consuming recipes (or a
/// K=1 item whose demand exceeds the 8-lane cap).
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
/// `Pooled`; populated under `PartitionedDecomposed`.
///
/// Algorithm (PR2 of Phase 1, plus Phase 2 decomposition):
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
    if !matches!(strategy, LayoutStrategy::PartitionedDecomposed) {
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

    // Phase 2 decomposition pass. Under `PartitionedDecomposed`
    // (the only strategy reaching this point):
    //   1. Sub-shard any existing module where lane_count > 8 into
    //      ⌈lane_count/8⌉ sub-modules of proportional rate.
    //   2. Add modules for K=1 items where total demand exceeds 8
    //      lanes — these are the structural growth blocker (one
    //      consumer demanding >8 lanes by itself).
    plan.modules = decompose_oversized_modules(plan.modules, cap);
    decompose_single_consumer_items(&mut plan, &consumers_of, cap);

    // Phase 3 (shape-aware): for any remaining module whose `(n, m)` shape
    // is not stampable (no direct template, no gcd-decomposition path —
    // the coprime trap, e.g. (4, 9) for copper-plate on PU@3/s ore red),
    // try strategies in order: pad lanes first (cheap layout-side cost),
    // shard as fallback (multiplies consumer rows). If neither works,
    // module stays as-is and the missing-balancer-template validator
    // warning surfaces.
    plan.modules = apply_shape_fixes(plan.modules, solver_result, cap);

    plan
}

/// Lane count below which we skip Phase 2 sharding even though the
/// module is over the 8-lane cap. The bus engine's downstream balancer
/// `decompose` fallback handles 9-10 lane modules cleanly via existing
/// templates (e.g. (10, 10) decomposes as 2 × (5, 5)), so sharding
/// these adds spec-multiplication overhead without a meaningful SAT-
/// zone-shrink win — and PU@2/s ore red specifically shows a regression
/// vs Phase 1 when 10-lane modules get sharded. Sharding kicks in at
/// 11+ lanes where the balancer library starts to thin out and the
/// SAT-zone shrink is more substantial. Tuned empirically against the
/// stress corpus; revisit if new stress cases shift the trade-off.
const SHARD_THRESHOLD_LANES: u32 = 10;

/// Cost-benefit ceiling on shard count. Sharding a module into N pieces
/// multiplies every consuming recipe-row by N (each consumer sub-row
/// taps one specific shard). For modules whose lane_count would
/// produce ≥ 4 shards, the consumer-side cost (more rows → more
/// junction crossings → more solver-failed clusters) routinely
/// dominates the producer-side benefit (smaller balancer SAT zones,
/// access to narrower library templates).
///
/// Bounding shards at 3 keeps the wins on EC@30/s ores yellow
/// (12 lanes → 2 shards, 1 extra consumer row) and EC@45/s ores
/// yellow (18 lanes → 3 shards, 2 extra rows) but blocks the
/// regression on PU@3/s plates yellow (29-lane copper-cable would be
/// 4 shards, 3 extra EC-consumer rows + 8× cluster multiplication
/// from Cartesian split with consumer-side EC partitioning).
///
/// Tuning rationale: above 3, the unsolved-junction count on the
/// scoreboard's plates-yellow cases climbed faster than the
/// balancer-template-availability won back. See PR #238 thread for
/// the data; revisit when junction-solver capability work (RFP #241)
/// changes the baseline.
const MAX_SHARDS_PER_MODULE: u32 = 3;

/// Phase 2 sub-pass: replace modules with `lane_count > SHARD_THRESHOLD_LANES`
/// with N proportional shards. Module IDs are reassigned dense per
/// item (0..N across all modules combined). Emits `ShardSplit` traces.
fn decompose_oversized_modules(
    modules: Vec<ModuleAssignment>,
    cap: f64,
) -> Vec<ModuleAssignment> {
    let mut by_item: FxHashMap<String, Vec<ModuleAssignment>> = FxHashMap::default();
    for m in modules {
        by_item.entry(m.item.clone()).or_default().push(m);
    }
    let mut out: Vec<ModuleAssignment> = Vec::new();
    let mut item_keys: Vec<String> = by_item.keys().cloned().collect();
    item_keys.sort();
    for item in item_keys {
        let item_modules = by_item.remove(&item).unwrap_or_default();
        let mut next_module_id: u32 = 0;
        for m in item_modules {
            if m.lane_count <= SHARD_THRESHOLD_LANES {
                let mut keep = m;
                keep.module_id = next_module_id;
                next_module_id += 1;
                out.push(keep);
                continue;
            }
            let would_be_shards = m.lane_count.div_ceil(8);
            if would_be_shards > MAX_SHARDS_PER_MODULE {
                trace::emit(TraceEvent::ShardSkipped {
                    item: item.clone(),
                    consumer_recipe: m.consumer_recipe.clone(),
                    lane_count: m.lane_count,
                    would_be_shards,
                    max_shards: MAX_SHARDS_PER_MODULE,
                });
                let mut keep = m;
                keep.module_id = next_module_id;
                next_module_id += 1;
                out.push(keep);
                continue;
            }
            let n_shards = would_be_shards as usize;
            let lanes_per_shard: Vec<usize> = (0..n_shards)
                .map(|i| {
                    let total = m.lane_count as usize;
                    let base = total / n_shards;
                    let rem = total % n_shards;
                    if i < rem { base + 1 } else { base }
                })
                .collect();
            trace::emit(TraceEvent::ShardSplit {
                item: item.clone(),
                consumer_recipe: m.consumer_recipe.clone(),
                original_lane_count: m.lane_count,
                shards: n_shards as u32,
                lanes_per_shard: lanes_per_shard.clone(),
            });
            for &shard_lanes in &lanes_per_shard {
                let shard_rate = m.rate * shard_lanes as f64 / m.lane_count as f64;
                let per_lane_rate = shard_rate / shard_lanes as f64;
                out.push(ModuleAssignment {
                    item: item.clone(),
                    module_id: next_module_id,
                    consumer_recipe: m.consumer_recipe.clone(),
                    rate: shard_rate,
                    lane_count: shard_lanes as u32,
                    utilization: per_lane_rate / (cap * UTILIZATION_CEILING),
                });
                next_module_id += 1;
            }
        }
    }
    out
}

/// Phase 2 sub-pass: shard K=1 items where demand exceeds 8 lanes.
/// Phase 1 leaves these as Pooled-equivalent (no module); Phase 2
/// shards them so no bus segment is ever wider than 8 lanes — the
/// growth-blocker case `processing-unit @ 5/s ore red` exemplifies.
fn decompose_single_consumer_items(
    plan: &mut PartitionPlan,
    consumers_of: &FxHashMap<String, Vec<(String, f64, bool)>>,
    cap: f64,
) {
    // Items already in the plan from Phase 1 (multi-consumer); skip.
    let already_planned: FxHashSet<String> =
        plan.modules.iter().map(|m| m.item.clone()).collect();
    let mut items: Vec<&String> = consumers_of.keys().collect();
    items.sort();
    for item in items {
        if already_planned.contains(item) {
            continue;
        }
        let consumers = &consumers_of[item];
        if consumers.first().is_some_and(|(_, _, is_fluid)| *is_fluid) {
            continue; // fluid carve-out
        }
        // K=1 only — by construction here (K≥2 would be in plan already).
        let mut by_recipe: FxHashMap<String, f64> = FxHashMap::default();
        for (recipe, rate, _) in consumers {
            *by_recipe.entry(recipe.clone()).or_insert(0.0) += rate;
        }
        if by_recipe.len() != 1 {
            continue;
        }
        let (recipe, rate) = by_recipe.into_iter().next().unwrap();
        let lane_count = (rate / cap).ceil().max(1.0) as u32;
        if lane_count <= SHARD_THRESHOLD_LANES {
            continue; // fits without sharding (see SHARD_THRESHOLD_LANES doc)
        }
        let would_be_shards = lane_count.div_ceil(8);
        if would_be_shards > MAX_SHARDS_PER_MODULE {
            trace::emit(TraceEvent::ShardSkipped {
                item: item.clone(),
                consumer_recipe: recipe.clone(),
                lane_count,
                would_be_shards,
                max_shards: MAX_SHARDS_PER_MODULE,
            });
            // K=1 item left out of the plan entirely — falls back to
            // Pooled-equivalent for this item (one wide module).
            continue;
        }
        let n_shards = would_be_shards as usize;
        let lanes_per_shard: Vec<usize> = (0..n_shards)
            .map(|i| {
                let total = lane_count as usize;
                let base = total / n_shards;
                let rem = total % n_shards;
                if i < rem { base + 1 } else { base }
            })
            .collect();
        trace::emit(TraceEvent::ShardSplit {
            item: item.clone(),
            consumer_recipe: recipe.clone(),
            original_lane_count: lane_count,
            shards: n_shards as u32,
            lanes_per_shard: lanes_per_shard.clone(),
        });
        for (shard_id, &shard_lanes) in lanes_per_shard.iter().enumerate() {
            let shard_rate = rate * shard_lanes as f64 / lane_count as f64;
            let per_lane_rate = shard_rate / shard_lanes as f64;
            plan.modules.push(ModuleAssignment {
                item: item.clone(),
                module_id: shard_id as u32,
                consumer_recipe: recipe.clone(),
                rate: shard_rate,
                lane_count: shard_lanes as u32,
                utilization: per_lane_rate / (cap * UTILIZATION_CEILING),
            });
        }
    }
}

/// Estimate the producer-row count for a given item from the solver
/// result. Used as the `n` input to shape-aware sharding decisions.
///
/// Returns the sum of `count` across all machines whose recipe outputs
/// the given (non-fluid) item. **This is an approximation:** the placer
/// row-splits machines based on output-belt capacity (`max_per_row` in
/// `placer.rs`), so the actual N at lane-planner time can be higher than
/// the raw machine count when a recipe needs more rows than its machine
/// count to fit the belt. For most cases the estimate matches; the
/// validator's `missing-balancer-template` warning catches the
/// divergent cases at layout time.
fn estimate_producer_count(solver: &SolverResult, item: &str) -> u32 {
    solver
        .machines
        .iter()
        .filter(|m| {
            m.outputs
                .iter()
                .any(|o| o.item == item && !o.is_fluid)
        })
        .map(|m| m.count as u32)
        .sum()
}

/// Phase 3 shape-aware fix pass. Walks every module and asks the strategy
/// selector if the module's `(n, m)` shape is stampable. If not, applies
/// the chosen fix (pad lanes or shard). Strategies tried in order: pad
/// first (cheap layout cost), shard fallback.
///
/// Returns the transformed module list. Module IDs are reassigned dense
/// per item so downstream code (apply_partition_plan) sees contiguous IDs.
fn apply_shape_fixes(
    modules: Vec<ModuleAssignment>,
    solver_result: &SolverResult,
    cap: f64,
) -> Vec<ModuleAssignment> {
    use crate::bus::shape_fix::{
        select_shape_fix, PadLanesStrategy, ShapeFix, ShapeFixStrategy, ShardStrategy,
    };

    let pad = PadLanesStrategy { max_pad: 4 };
    let shard = ShardStrategy {
        max_shards: MAX_SHARDS_PER_MODULE,
    };
    let strategies: &[&dyn ShapeFixStrategy] = &[&pad, &shard];

    // Group by item so we can reassign module_ids densely after fixing.
    let mut by_item: FxHashMap<String, Vec<ModuleAssignment>> = FxHashMap::default();
    for m in modules {
        by_item.entry(m.item.clone()).or_default().push(m);
    }

    let mut item_keys: Vec<String> = by_item.keys().cloned().collect();
    item_keys.sort();

    let mut out: Vec<ModuleAssignment> = Vec::new();
    for item in item_keys {
        let item_modules = by_item.remove(&item).unwrap_or_default();
        let n_estimate = estimate_producer_count(solver_result, &item);
        let mut next_module_id: u32 = 0;

        for m in item_modules {
            let m_lanes = m.lane_count;
            let fix = select_shape_fix(n_estimate, m_lanes, strategies);

            match fix {
                Some(ShapeFix::Native) | None => {
                    // Native: shape is already stampable. Keep as-is.
                    // None: no strategy could fix; keep as-is so the
                    // validator surfaces the missing-template warning
                    // and the layout dead-ends loudly.
                    let mut keep = m;
                    keep.module_id = next_module_id;
                    next_module_id += 1;
                    out.push(keep);
                }
                Some(ShapeFix::PadLanes { new_m }) => {
                    trace::emit(TraceEvent::ShapeFixApplied {
                        item: item.clone(),
                        consumer_recipe: m.consumer_recipe.clone(),
                        n: n_estimate,
                        original_m: m_lanes,
                        strategy: "pad-lanes".to_string(),
                        kind: "pad-lanes".to_string(),
                        new_total_lanes: new_m,
                    });
                    let mut keep = m;
                    // Bump lane_count; rate stays the same so per-lane
                    // utilisation drops (the extra lanes are empty).
                    keep.lane_count = new_m;
                    keep.utilization = (keep.rate / new_m as f64) / (cap * UTILIZATION_CEILING);
                    keep.module_id = next_module_id;
                    next_module_id += 1;
                    out.push(keep);
                }
                Some(ShapeFix::Shard { lanes }) => {
                    let total_lanes: u32 = lanes.iter().sum();
                    trace::emit(TraceEvent::ShapeFixApplied {
                        item: item.clone(),
                        consumer_recipe: m.consumer_recipe.clone(),
                        n: n_estimate,
                        original_m: m_lanes,
                        strategy: "shard".to_string(),
                        kind: "shard".to_string(),
                        new_total_lanes: total_lanes,
                    });
                    // Split rate proportionally to lane allocation.
                    let total_rate = m.rate;
                    for shard_lanes in &lanes {
                        let shard_rate = total_rate * (*shard_lanes as f64) / (m_lanes as f64);
                        let per_lane_rate = shard_rate / *shard_lanes as f64;
                        out.push(ModuleAssignment {
                            item: m.item.clone(),
                            module_id: next_module_id,
                            consumer_recipe: m.consumer_recipe.clone(),
                            rate: shard_rate,
                            lane_count: *shard_lanes,
                            utilization: per_lane_rate / (cap * UTILIZATION_CEILING),
                        });
                        next_module_id += 1;
                    }
                }
            }
        }
    }

    out
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

    // `(item, recipe) → list of module_ids` for consumer-side split.
    // Multi-module entries arise from Phase 2 sharding (one consumer
    // recipe served by multiple shards of the same item).
    let mut consumer_modules: FxHashMap<(String, String), Vec<u32>> = FxHashMap::default();
    for m in &plan.modules {
        consumer_modules
            .entry((m.item.clone(), m.consumer_recipe.clone()))
            .or_default()
            .push(m.module_id);
    }
    for ids in consumer_modules.values_mut() {
        ids.sort();
    }
    // `item → list-of-modules` for producer-side splitting. Sorted by
    // module_id for deterministic sibling order.
    let mut item_modules: FxHashMap<String, Vec<&ModuleAssignment>> = FxHashMap::default();
    for m in &plan.modules {
        item_modules.entry(m.item.clone()).or_default().push(m);
    }
    for v in item_modules.values_mut() {
        v.sort_by_key(|m| m.module_id);
    }

    let mut new_machines: Vec<MachineSpec> = Vec::with_capacity(solver_result.machines.len());
    for spec in &solver_result.machines {
        // Determine which output item, if any, gets partitioned.
        // Single non-fluid product per recipe (existing codebase
        // assumption); fluid by-products are unaffected.
        let primary_solid_idx = spec
            .outputs
            .iter()
            .position(|o| !o.is_fluid && item_modules.contains_key(&o.item));

        let producer_modules = primary_solid_idx
            .and_then(|idx| item_modules.get(&spec.outputs[idx].item))
            .cloned()
            .unwrap_or_default();

        // Producer-side: emit one sibling per module (multi-module
        // happens for Phase 1 K≥2 partitions and Phase 2 shards alike).
        let producer_siblings: Vec<MachineSpec> = if producer_modules.len() < 2 {
            vec![spec.clone()]
        } else {
            let total_rate: f64 = producer_modules.iter().map(|m| m.rate).sum();
            let n_modules = producer_modules.len();
            producer_modules
                .iter()
                .map(|module| {
                    let share = if total_rate > 0.0 {
                        module.rate / total_rate
                    } else {
                        1.0 / n_modules as f64
                    };
                    let mut sibling = spec.clone();
                    sibling.count = spec.count * share;
                    for out in &mut sibling.outputs {
                        out.module_id = module.module_id;
                    }
                    // Recursive partitioning is out of scope; upstream
                    // ingredients consumed by the producer stay at
                    // module_id=0.
                    for inp in &mut sibling.inputs {
                        inp.module_id = 0;
                    }
                    sibling
                })
                .collect()
        };

        // Consumer-side: for each producer sibling, build a Cartesian
        // product of its inputs across each input's shards. Single-
        // shard inputs collapse to length-1 in the product.
        for mut sib in producer_siblings {
            // Collect per-input module-id lists.
            let input_shards: Vec<Vec<u32>> = sib
                .inputs
                .iter()
                .map(|inp| {
                    consumer_modules
                        .get(&(inp.item.clone(), sib.recipe.clone()))
                        .cloned()
                        .unwrap_or_else(|| vec![inp.module_id])
                })
                .collect();
            let cart_size: usize = input_shards.iter().map(|v| v.len()).product();
            if cart_size <= 1 {
                // No Cartesian split needed. Apply single-shard inputs.
                for (i, inp) in sib.inputs.iter_mut().enumerate() {
                    if let Some(id) = input_shards[i].first() {
                        inp.module_id = *id;
                    }
                }
                new_machines.push(sib);
                continue;
            }
            // Cartesian split. cart_size sub-specs, each with
            // count/cart_size and one specific (input_idx → module_id)
            // assignment. Index unrolling: for cart_idx in 0..cart_size,
            // input_i's shard index = (cart_idx / div_i) % len_i.
            let count_per = sib.count / cart_size as f64;
            for cart_idx in 0..cart_size {
                let mut sub = sib.clone();
                sub.count = count_per;
                let mut div = 1usize;
                for (i, ids) in input_shards.iter().enumerate() {
                    let n = ids.len();
                    let shard_idx = (cart_idx / div) % n;
                    sub.inputs[i].module_id = ids[shard_idx];
                    div *= n;
                }
                new_machines.push(sub);
            }
        }
    }

    SolverResult {
        machines: new_machines,
        external_inputs: solver_result.external_inputs.clone(),
        external_outputs: solver_result.external_outputs.clone(),
        dependency_order: solver_result.dependency_order.clone(),
    }
}

/// Replace each module in `plan` with `k` sibling modules each carrying
/// `1/k` of the original rate. Module IDs are densely reassigned per item
/// so downstream `apply_partition_plan` keying on `(item, module_id)`
/// remains contiguous.
///
/// Used by `bus::decomposition_search::ModuleSizeSplit` (see
/// `docs/rfp-decomposition-search.md`). Splitting `(item=copper-plate,
/// recipe=electronic-circuit, rate=R, lane_count=L)` into `k=2` gives
/// two modules with `rate=R/2` and `lane_count=ceil(R/2 / per_lane_cap)`,
/// both serving the same consumer recipe. Producer-side, the existing
/// `apply_partition_plan` machine-share formula (line ~688) divides
/// `MachineSpec.count` by `k` automatically, since each new sibling has
/// rate `R/k` against original total `R`. Consumer-side, the Cartesian
/// unrolling splits consuming machines across the `k` modules.
///
/// Lane-count rounding: `ceil(rate / k / per_lane_cap)` per sibling. Total
/// across siblings may slightly overshoot the original `lane_count`
/// (e.g. `(4, 9) → 2 × (2, 5)` = 10 lanes for what was 9), which is the
/// expected layout-width cost of the split. The rate-per-lane stays the
/// same; sub-saturation isn't introduced.
///
/// Skips modules with `lane_count <= 1` — splitting a single-lane module
/// produces two modules each rounded back up to one lane, doubling the
/// lane count without any shape benefit.
///
/// Trace event: emits `ModuleSizeSplitApplied` per resulting sibling.
/// Read by tests / scoreboards to verify the candidate exercised on a
/// given case.
pub(crate) fn apply_size_split(
    plan: PartitionPlan,
    k: u32,
    max_belt_tier: Option<&str>,
) -> PartitionPlan {
    if k <= 1 {
        return plan;
    }
    let cap = lane_capacity(max_belt_tier);
    let utilization_cap = cap * UTILIZATION_CEILING;

    let mut by_item: FxHashMap<String, Vec<ModuleAssignment>> = FxHashMap::default();
    for module in plan.modules {
        by_item.entry(module.item.clone()).or_default().push(module);
    }

    let mut new_modules: Vec<ModuleAssignment> = Vec::new();
    for (_item, modules) in by_item.iter_mut() {
        // Process in deterministic order so module_id reassignment is stable.
        modules.sort_by_key(|m| m.module_id);
        let mut next_id: u32 = 0;
        for original in modules.drain(..) {
            // Trivial module — splitting yields two 1-lane siblings; no
            // shape-fix benefit, just lane-count waste. Keep as-is.
            if original.lane_count <= 1 {
                let mut kept = original.clone();
                kept.module_id = next_id;
                next_id += 1;
                new_modules.push(kept);
                continue;
            }

            let original_module_id = original.module_id;
            let split_rate = original.rate / k as f64;
            let split_lane_count = ((split_rate / cap).ceil() as u32).max(1);
            let split_utilization = if utilization_cap > 0.0 && split_lane_count > 0 {
                (split_rate / split_lane_count as f64) / utilization_cap
            } else {
                f64::INFINITY
            };

            for _ in 0..k {
                let new_module_id = next_id;
                next_id += 1;
                let sibling = ModuleAssignment {
                    item: original.item.clone(),
                    module_id: new_module_id,
                    consumer_recipe: original.consumer_recipe.clone(),
                    rate: split_rate,
                    lane_count: split_lane_count,
                    utilization: split_utilization,
                };
                trace::emit(TraceEvent::ModuleSizeSplitApplied {
                    item: sibling.item.clone(),
                    consumer_recipe: sibling.consumer_recipe.clone(),
                    original_module_id,
                    k_splits: k,
                    new_module_id,
                    rate: split_rate,
                    lane_count: split_lane_count,
                });
                new_modules.push(sibling);
            }
        }
    }

    // Sort for deterministic output (apply_partition_plan does its own
    // sorts but we preserve ordering by (item, module_id) for tests).
    new_modules.sort_by(|a, b| a.item.cmp(&b.item).then(a.module_id.cmp(&b.module_id)));

    PartitionPlan {
        modules: new_modules,
        utilization_violations: Vec::new(),
    }
}

/// Replace any module whose rate exceeds full-belt capacity with K
/// sibling sub-modules, where `K = ceil(rate / full_belt_cap)` and
/// `full_belt_cap = lane_capacity(belt_tier) * 2`. Each sub-module's
/// rate ≤ `full_belt_cap`, so the lane planner's consumer-clamp path
/// (`split_overflowing_lanes`) won't be asked to fit more flow than a
/// single physical belt can carry.
///
/// Module IDs are densely reassigned per item; existing
/// `apply_partition_plan` Cartesian unrolling splits consuming
/// machines across the new sub-modules naturally. Producer-side, the
/// rate-share formula gives each sub-module its proportional share of
/// the upstream machines.
///
/// Modules already at-or-below `full_belt_cap` are passed through
/// unchanged with their `module_id` densely re-numbered.
///
/// Trace event: `ModuleCapSplitApplied` per resulting sub-module of a
/// split (not for unchanged modules).
pub(crate) fn apply_cap_driven_split(
    modules: Vec<ModuleAssignment>,
    max_belt_tier: Option<&str>,
) -> Vec<ModuleAssignment> {
    let cap = lane_capacity(max_belt_tier);
    let full_belt_cap = cap * 2.0;
    if full_belt_cap <= 0.0 {
        return modules;
    }
    let utilization_cap = cap * UTILIZATION_CEILING;

    let mut by_item: FxHashMap<String, Vec<ModuleAssignment>> = FxHashMap::default();
    for module in modules {
        by_item.entry(module.item.clone()).or_default().push(module);
    }

    let mut new_modules: Vec<ModuleAssignment> = Vec::new();
    for (_item, item_modules) in by_item.iter_mut() {
        // Process in deterministic order so module_id reassignment is stable.
        item_modules.sort_by_key(|m| m.module_id);
        let mut next_id: u32 = 0;
        for original in item_modules.drain(..) {
            // No split needed — rate fits on a single full-belt trunk.
            // Renumber densely and pass through.
            if original.rate <= full_belt_cap {
                let mut kept = original.clone();
                kept.module_id = next_id;
                next_id += 1;
                new_modules.push(kept);
                continue;
            }

            // Compute K. Float-safe ceil — `(rate / cap).ceil()` rounds
            // up exactly when rate > cap.
            let k = (original.rate / full_belt_cap).ceil() as u32;
            let k = k.max(2); // already > cap, so K is at least 2
            let original_module_id = original.module_id;
            let split_rate = original.rate / k as f64;
            let split_lane_count = ((split_rate / cap).ceil() as u32).max(1);
            let split_utilization = if utilization_cap > 0.0 {
                (split_rate / split_lane_count as f64) / utilization_cap
            } else {
                f64::INFINITY
            };

            for _ in 0..k {
                let new_module_id = next_id;
                next_id += 1;
                let sibling = ModuleAssignment {
                    item: original.item.clone(),
                    module_id: new_module_id,
                    consumer_recipe: original.consumer_recipe.clone(),
                    rate: split_rate,
                    lane_count: split_lane_count,
                    utilization: split_utilization,
                };
                trace::emit(TraceEvent::ModuleCapSplitApplied {
                    item: sibling.item.clone(),
                    consumer_recipe: sibling.consumer_recipe.clone(),
                    original_module_id,
                    k_splits: k,
                    new_module_id,
                    original_rate: original.rate,
                    new_rate: split_rate,
                    full_belt_cap,
                });
                new_modules.push(sibling);
            }
        }
    }

    // Sort for deterministic output (apply_partition_plan does its own
    // sorts but we preserve ordering by (item, module_id) for tests).
    new_modules.sort_by(|a, b| a.item.cmp(&b.item).then(a.module_id.cmp(&b.module_id)));

    new_modules
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
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, None);
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
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, Some("transport-belt"));
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
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, None);
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
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, Some("transport-belt"));
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

    /// Phase 2: a K=1 item with > 8 lanes of demand gets sharded
    /// under PartitionedDecomposed.
    #[test]
    fn phase2_shards_k1_oversized_item() {
        // Single iron-gear-wheel consumer at high rate. iron-plate has K=1.
        // 60 IGW/s × 2 plate = 120/s on yellow (cap 7.5) = 16 lanes (over 8 by 8).
        let solver_result = SolverResult {
            machines: vec![
                machine("iron-gear-wheel", 60.0, vec![flow("iron-plate", 2.0)], vec![flow("iron-gear-wheel", 1.0)]),
            ],
            external_inputs: vec![flow("iron-plate", 120.0)],
            external_outputs: vec![flow("iron-gear-wheel", 60.0)],
            dependency_order: vec!["iron-gear-wheel".to_string()],
        };
        // PartitionedDecomposed: K=1 with 16 lanes → shard into 2 of 8.
        let p2 = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, Some("transport-belt"));
        let plate_modules: Vec<_> = p2.modules.iter().filter(|m| m.item == "iron-plate").collect();
        assert_eq!(plate_modules.len(), 2, "expected 2 shards for 16-lane K=1 item");
        assert_eq!(plate_modules[0].lane_count, 8);
        assert_eq!(plate_modules[1].lane_count, 8);
        assert_eq!(plate_modules[0].consumer_recipe, "iron-gear-wheel");
        assert_eq!(plate_modules[1].consumer_recipe, "iron-gear-wheel");
        // Module IDs are 0 and 1 within iron-plate.
        let mut ids: Vec<u32> = plate_modules.iter().map(|m| m.module_id).collect();
        ids.sort();
        assert_eq!(ids, vec![0, 1]);
    }

    /// Phase 2: a K≥2 item where one module is itself > 8 lanes gets
    /// further sub-sharded (the core motivating case).
    #[test]
    fn phase2_subshards_oversized_module() {
        // Two consumers of copper-cable; the EC consumer has very high
        // demand (180/s = 24 lanes on yellow @ 7.5/s), AC moderate (30/s = 4 lanes).
        let solver_result = SolverResult {
            machines: vec![
                machine("copper-cable", 100.0, vec![flow("copper-plate", 1.0)], vec![flow("copper-cable", 2.0)]),
                machine("electronic-circuit", 60.0, vec![flow("copper-cable", 3.0), flow("iron-plate", 1.0)], vec![flow("electronic-circuit", 1.0)]),
                machine("advanced-circuit", 7.5, vec![flow("copper-cable", 4.0), flow("electronic-circuit", 2.0)], vec![flow("advanced-circuit", 1.0)]),
            ],
            external_inputs: vec![flow("copper-plate", 100.0), flow("iron-plate", 60.0)],
            external_outputs: vec![flow("advanced-circuit", 7.5)],
            dependency_order: vec!["copper-cable".to_string(), "electronic-circuit".to_string(), "advanced-circuit".to_string()],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, Some("transport-belt"));
        let cu_modules: Vec<_> = plan.modules.iter().filter(|m| m.item == "copper-cable").collect();
        // EC consumer demand: 60 × 3 = 180/s = 24 lanes → 3 shards of 8.
        // AC consumer demand: 7.5 × 4 = 30/s = 4 lanes → 1 module (no shard).
        // Total: 3 + 1 = 4 modules for copper-cable.
        assert_eq!(cu_modules.len(), 4, "expected 3 EC shards + 1 AC = 4 cable modules; got {}", cu_modules.len());
        let ec_shards: Vec<_> = cu_modules.iter().filter(|m| m.consumer_recipe == "electronic-circuit").collect();
        let ac_modules: Vec<_> = cu_modules.iter().filter(|m| m.consumer_recipe == "advanced-circuit").collect();
        assert_eq!(ec_shards.len(), 3, "EC should be 3 shards");
        assert_eq!(ac_modules.len(), 1, "AC should stay as 1 module (4 lanes ≤ 8)");
        // Each EC shard ≤ 8 lanes.
        for s in &ec_shards {
            assert!(s.lane_count <= 8, "EC shard lane_count = {} should be ≤ 8", s.lane_count);
        }
        // Module IDs unique within copper-cable.
        let mut ids: Vec<u32> = cu_modules.iter().map(|m| m.module_id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 4, "module IDs should be unique within an item");
    }

    /// `apply_partition_plan` Cartesian-splits a consumer when one of its
    /// inputs is multi-shard.
    #[test]
    fn phase2_apply_plan_cartesian_splits_consumer() {
        // Same shape as `phase2_subshards_oversized_module`. We expect EC's
        // machinespec to split into 3 sub-specs (one per cable shard),
        // each tapping a different module_id.
        let solver_result = SolverResult {
            machines: vec![
                machine("copper-cable", 100.0, vec![flow("copper-plate", 1.0)], vec![flow("copper-cable", 2.0)]),
                machine("electronic-circuit", 60.0, vec![flow("copper-cable", 3.0), flow("iron-plate", 1.0)], vec![flow("electronic-circuit", 1.0)]),
                machine("advanced-circuit", 7.5, vec![flow("copper-cable", 4.0), flow("electronic-circuit", 2.0)], vec![flow("advanced-circuit", 1.0)]),
            ],
            external_inputs: vec![flow("copper-plate", 100.0), flow("iron-plate", 60.0)],
            external_outputs: vec![flow("advanced-circuit", 7.5)],
            dependency_order: vec![],
        };
        let plan = plan_partitioning(&solver_result, LayoutStrategy::PartitionedDecomposed, Some("transport-belt"));
        let partitioned = apply_partition_plan(&solver_result, &plan);

        // EC consumer should split into 3 sub-specs (one per cable shard).
        // iron-plate is K=1 with 60/s = 8 lanes (right at cap, no shard).
        let ec_specs: Vec<&MachineSpec> = partitioned.machines.iter().filter(|m| m.recipe == "electronic-circuit").collect();
        assert_eq!(ec_specs.len(), 3, "EC should split into 3 (one per cable shard)");
        let mut cable_ids: Vec<u32> = ec_specs.iter()
            .map(|s| s.inputs.iter().find(|i| i.item == "copper-cable").unwrap().module_id)
            .collect();
        cable_ids.sort();
        cable_ids.dedup();
        assert_eq!(cable_ids.len(), 3, "EC sub-specs should each tap a different cable shard");
        // Each EC sub-spec count = original / 3.
        let total: f64 = ec_specs.iter().map(|s| s.count).sum();
        assert!((total - 60.0).abs() < 1e-9, "EC total count should preserve");
    }

    fn module(item: &str, recipe: &str, module_id: u32, rate: f64, lane_count: u32) -> ModuleAssignment {
        ModuleAssignment {
            item: item.to_string(),
            module_id,
            consumer_recipe: recipe.to_string(),
            rate,
            lane_count,
            utilization: 0.5,
        }
    }

    #[test]
    fn apply_size_split_doubles_modules_with_halved_rate() {
        let plan = PartitionPlan {
            modules: vec![
                module("copper-plate", "electronic-circuit", 0, 60.0, 9),
            ],
            utilization_violations: vec![],
        };
        let split = apply_size_split(plan, 2, Some("transport-belt"));
        assert_eq!(split.modules.len(), 2, "k=2 should produce two sibling modules");
        let total_rate: f64 = split.modules.iter().map(|m| m.rate).sum();
        assert!((total_rate - 60.0).abs() < 1e-9, "total rate must be preserved");
        // Module IDs are densely re-numbered per item starting at 0.
        let mut ids: Vec<u32> = split.modules.iter().map(|m| m.module_id).collect();
        ids.sort();
        assert_eq!(ids, vec![0, 1]);
        // Both siblings serve the same consumer recipe.
        for m in &split.modules {
            assert_eq!(m.consumer_recipe, "electronic-circuit");
            assert_eq!(m.item, "copper-plate");
        }
    }

    #[test]
    fn apply_size_split_skips_lane_count_one_modules() {
        let plan = PartitionPlan {
            modules: vec![
                module("rare-metal", "advanced-circuit", 0, 5.0, 1),
            ],
            utilization_violations: vec![],
        };
        let split = apply_size_split(plan, 2, Some("transport-belt"));
        // Single-lane modules can't usefully halve — splitting yields
        // two 1-lane siblings (lane_count rounds back up). We keep the
        // original to avoid the lane-count waste.
        assert_eq!(split.modules.len(), 1, "single-lane modules stay intact");
        assert_eq!(split.modules[0].lane_count, 1);
        assert!((split.modules[0].rate - 5.0).abs() < 1e-9);
    }

    #[test]
    fn apply_size_split_no_op_for_k_le_1() {
        let plan = PartitionPlan {
            modules: vec![
                module("copper-plate", "electronic-circuit", 0, 60.0, 9),
                module("copper-plate", "advanced-circuit", 1, 30.0, 4),
            ],
            utilization_violations: vec![],
        };
        let original_count = plan.modules.len();
        let result = apply_size_split(plan, 1, Some("transport-belt"));
        assert_eq!(result.modules.len(), original_count, "k=1 is a no-op");
    }

    #[test]
    fn apply_size_split_renumbers_per_item_densely() {
        // Two items each with two modules. After k=2 split, each item
        // should have 4 modules with module_ids 0..4 (dense per-item).
        let plan = PartitionPlan {
            modules: vec![
                module("copper-plate", "ec", 0, 30.0, 4),
                module("copper-plate", "ac", 1, 30.0, 4),
                module("iron-plate", "ec", 0, 20.0, 3),
                module("iron-plate", "ac", 1, 20.0, 3),
            ],
            utilization_violations: vec![],
        };
        let split = apply_size_split(plan, 2, Some("transport-belt"));
        let mut copper_ids: Vec<u32> = split.modules.iter()
            .filter(|m| m.item == "copper-plate")
            .map(|m| m.module_id)
            .collect();
        copper_ids.sort();
        assert_eq!(copper_ids, vec![0, 1, 2, 3]);
        let mut iron_ids: Vec<u32> = split.modules.iter()
            .filter(|m| m.item == "iron-plate")
            .map(|m| m.module_id)
            .collect();
        iron_ids.sort();
        assert_eq!(iron_ids, vec![0, 1, 2, 3]);
    }
}
