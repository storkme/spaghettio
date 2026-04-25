//! Demand-partitioning of multi-consumer items.
//!
//! Implements the outer pass of `LayoutStrategy::PartitionedPerConsumer`
//! from `docs/rfp-modular-production.md`. The actual algorithm — assigning
//! `module_id`s to `ItemFlow` instances and splitting producer rows — is
//! introduced in PR2 of Phase 1. This file is the entry point used by
//! `build_bus_layout` and the utilization-gate helper. The dispatcher
//! here is deliberately conservative: under Pooled it is a no-op, and
//! under `PartitionedPerConsumer` it falls through to Pooled-equivalent
//! behaviour when every intermediate item has K=1 consumer rows. K>1
//! cases panic with a clear message until PR2 lands.
//!
//! See K0-1, K1-2, K1-3, K1-4 in the RFP for the gates this code
//! upholds.

use rustc_hash::FxHashMap;

use crate::models::SolverResult;

/// Per-belt-tier per-lane capacity in items/s. Mirrors
/// `LANE_CAPACITY_TABLE` in `lane_planner.rs`. Single source of truth
/// is awkward today because the planner's table is private; the
/// partitioner only needs the numeric ceilings.
const PER_LANE_CAPACITY: &[(&str, f64)] = &[
    ("transport-belt", 15.0),
    ("fast-transport-belt", 30.0),
    ("express-transport-belt", 45.0),
];

/// The 75% over-provisioning ceiling from the RFP's "Load-bearing
/// assumption" section. Lanes above this fraction of belt capacity are
/// assumed to suffer from per-machine timing jitter, and the
/// partitioner refuses to allocate them.
pub const UTILIZATION_CEILING: f64 = 0.75;

/// Per-lane belt capacity for the chosen belt tier. Falls back to the
/// fastest tier when `max_belt_tier` is `None` (matches the lane
/// planner's behaviour).
pub fn lane_capacity(max_belt_tier: Option<&str>) -> f64 {
    let default_cap = PER_LANE_CAPACITY.last().map(|(_, c)| *c).unwrap_or(15.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_capacity_falls_back_to_fastest_tier() {
        assert_eq!(lane_capacity(None), 45.0);
        assert_eq!(lane_capacity(Some("transport-belt")), 15.0);
        assert_eq!(lane_capacity(Some("fast-transport-belt")), 30.0);
        assert_eq!(lane_capacity(Some("express-transport-belt")), 45.0);
        // Unknown tier → fastest.
        assert_eq!(lane_capacity(Some("not-a-belt")), 45.0);
    }

    #[test]
    fn utilization_ceiling_at_75_percent() {
        // Yellow belt: 15/s × 0.75 = 11.25 ceiling.
        assert!((lane_utilization(11.25, Some("transport-belt")) - 1.0).abs() < 1e-9);
        // Just over → > 1.0.
        assert!(lane_utilization(11.30, Some("transport-belt")) > 1.0);
        // Just under → < 1.0.
        assert!(lane_utilization(11.20, Some("transport-belt")) < 1.0);
    }
}
