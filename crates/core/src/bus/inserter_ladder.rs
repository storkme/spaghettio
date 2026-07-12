//! Per-side inserter sizing ladder.
//!
//! `docs/rfp-inserter-sizing.md` — every row template used to place
//! exactly one regular inserter (~0.84/s) per machine side regardless of
//! the planned rate, making machines inserter-bound engine-wide. This
//! module picks the cheapest-sufficient inserter tier/count for a side's
//! planned rate: in-place tier upgrade first (regular → fast → stack),
//! then extra inserters into the position's real free-column budget, then
//! best-effort + honest shortfall if the position can't cover the rate
//! even at the top tier.
//!
//! Mirrors `examples/census_inserter_sizing_v2.rs`'s `place_ladder` (the
//! frozen Phase 0v2 evidence base) exactly for `max_tier ==
//! InserterTier::Stack`; generalizes it for lower caps so
//! `max_inserter_tier` capping is meaningful. Rates come from
//! `common::inserter_throughput` — the SAME table
//! `validate::inserters::check_inserter_throughput` reads — so the fix
//! and the check can never disagree on what an inserter moves (see the
//! `constants_identity` test below).

use crate::common::inserter_throughput;

/// Inserter entity names the ladder places, by tier.
pub const REGULAR: &str = "inserter";
pub const FAST: &str = "fast-inserter";
pub const STACK: &str = "stack-inserter";
pub const LONG_HANDED: &str = "long-handed-inserter";

const EPS: f64 = 1e-9;

/// User-facing hard cap on the near-reach ladder, mirroring
/// `LayoutOptions::max_belt_tier` semantics: the ladder never places an
/// inserter above this tier, even when that leaves a side under-
/// provisioned (best-effort + shortfall, never a layout failure).
///
/// Reach-2 (far) sides are unaffected — long-handed is the only inserter
/// that reaches 2 tiles, so there is no tier choice to cap there.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Default, Hash)]
pub enum InserterTier {
    Regular,
    Fast,
    #[default]
    Stack,
}

impl InserterTier {
    /// Entity names this cap allows, cheapest first — the order `size_side`
    /// tries them in.
    fn allowed_near_entities(self) -> &'static [&'static str] {
        match self {
            InserterTier::Regular => &[REGULAR],
            InserterTier::Fast => &[REGULAR, FAST],
            InserterTier::Stack => &[REGULAR, FAST, STACK],
        }
    }
}

/// How far the inserter must reach to touch both the machine and the
/// belt: `Near` (1 tile) gets the full regular/fast/stack ladder; `Far`
/// (2 tiles) can only ever be a long-handed inserter — no fast or stack
/// long-handed exists in vanilla Factorio — so it gets a count-ladder at
/// the single long-handed rate instead of a tier ladder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reach {
    Near,
    Far,
}

/// Result of sizing one machine side: place `count` inserters of
/// `entity`, cheapest-sufficient. `shortfall` is `Some(rate)` when even
/// the richest achievable placement (every free column used, tier capped
/// at `max_tier`) still falls short of `required` — the honest residual
/// `check_inserter_throughput` will keep warning about. Sizing never
/// fails a layout; a shortfall is recorded, not refused.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidePlan {
    pub entity: &'static str,
    pub count: usize,
    pub shortfall: Option<f64>,
}

/// Size one machine side.
///
/// `required` is the per-machine, utilization-scaled rate the side must
/// move (items/s). `position_budget` is the number of EXTRA columns the
/// caller's template geometry can spare beyond the one baseline slot
/// every side already has (0 at the tightest positions, e.g. a
/// sideload-bridge anchor's output face). `max_tier` caps the near-reach
/// ladder; it has no effect on `Reach::Far` (see [`Reach`]).
pub fn size_side(required: f64, reach: Reach, position_budget: usize, max_tier: InserterTier) -> SidePlan {
    let total_slots = position_budget + 1;
    match reach {
        Reach::Far => {
            let rate = inserter_throughput(LONG_HANDED);
            for n in 1..=total_slots {
                let placed = n as f64 * rate;
                if required <= placed + EPS {
                    return SidePlan { entity: LONG_HANDED, count: n, shortfall: None };
                }
            }
            let placed = total_slots as f64 * rate;
            SidePlan {
                entity: LONG_HANDED,
                count: total_slots,
                shortfall: Some((required - placed).max(0.0)),
            }
        }
        Reach::Near => {
            let tiers = max_tier.allowed_near_entities();

            // Rung 0/1: one inserter, cheapest tier that suffices
            // ("in-place tier swap" — zero extra columns spent).
            for &entity in tiers {
                let rate = inserter_throughput(entity);
                if required <= rate + EPS {
                    return SidePlan { entity, count: 1, shortfall: None };
                }
            }

            // Rung 2+: increase count into the free columns, cheapest
            // tier's count-ladder exhausted before moving to the next
            // tier up (matches the census: multi-fast fully tried before
            // any multi-stack).
            if total_slots > 1 {
                for &entity in tiers {
                    let rate = inserter_throughput(entity);
                    for n in 2..=total_slots {
                        let placed = n as f64 * rate;
                        if required <= placed + EPS {
                            return SidePlan { entity, count: n, shortfall: None };
                        }
                    }
                }
            }

            // Best effort: every free column, richest tier the cap
            // allows, honest shortfall recorded.
            let entity = tiers.last().copied().unwrap_or(REGULAR);
            let rate = inserter_throughput(entity);
            let placed = total_slots as f64 * rate;
            SidePlan {
                entity,
                count: total_slots,
                shortfall: Some((required - placed).max(0.0)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── constants identity ──────────────────────────────────────────────
    // The ladder must source every rate from `common::inserter_throughput`
    // — the same table the validator reads — never a duplicated literal.
    // This test pins the values that table currently returns; if it drifts,
    // this fails loudly instead of the ladder and the check silently
    // disagreeing.

    #[test]
    fn constants_identity() {
        assert_eq!(inserter_throughput(REGULAR), 0.84);
        assert_eq!(inserter_throughput(FAST), 2.31);
        assert_eq!(inserter_throughput(STACK), 12.0);
        assert_eq!(inserter_throughput(LONG_HANDED), 1.2);
    }

    // ── rung boundaries (Near, Stack cap, single slot) ──────────────────

    #[test]
    fn rung0_regular_at_boundary() {
        let plan = size_side(0.84, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: REGULAR, count: 1, shortfall: None });
    }

    #[test]
    fn rung0_regular_below_boundary() {
        let plan = size_side(0.5, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: REGULAR, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_fast_just_above_regular() {
        // Just over the regular ceiling — must NOT jump to stack.
        let plan = size_side(0.8401, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: FAST, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_fast_at_boundary() {
        let plan = size_side(2.31, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: FAST, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_stack_just_above_fast() {
        // Just over the fast ceiling — must NOT skip straight past
        // without trying fast first (cheapest-sufficient).
        let plan = size_side(2.311, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: STACK, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_stack_at_boundary() {
        let plan = size_side(12.0, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: STACK, count: 1, shortfall: None });
    }

    // ── cheapest-sufficient (KC6): never stack where fast suffices ─────

    #[test]
    fn cheapest_sufficient_never_stack_when_fast_covers() {
        for required in [0.85, 1.0, 1.5, 2.0, 2.31] {
            let plan = size_side(required, Reach::Near, 0, InserterTier::Stack);
            assert_ne!(
                plan.entity, STACK,
                "required={required}: fast (2.31/s) covers this, must not place stack"
            );
        }
    }

    #[test]
    fn cheapest_sufficient_never_fast_when_regular_covers() {
        for required in [0.1, 0.5, 0.84] {
            let plan = size_side(required, Reach::Near, 0, InserterTier::Stack);
            assert_eq!(plan.entity, REGULAR, "required={required}: regular alone covers this");
        }
    }

    // ── extra-column rung (count-ladder within budget) ──────────────────

    #[test]
    fn rung2_multi_stack_within_budget() {
        // 13/s exceeds one stack inserter (12/s) but two cover it (24/s),
        // and a free column is available.
        let plan = size_side(13.0, Reach::Near, 1, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: STACK, count: 2, shortfall: None });
    }

    #[test]
    fn rung2_prefers_fast_count_over_stack_count() {
        // A single stack inserter (12/s) covers anything up to 12/s at
        // rung 0/1, so rung 2+ is only reachable above that — and once
        // there, the cheaper tier's own count-ladder (fast) is exhausted
        // before any stack count is tried, even when fewer, pricier
        // inserters would also fit the budget. 13.0/s exceeds 1 stack
        // (12/s) and needs 6 fast (13.86/s) within a 6-slot budget; 2
        // stack (24/s) would also fit but must not be reached first.
        let plan = size_side(13.0, Reach::Near, 5, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: FAST, count: 6, shortfall: None });
    }

    // ── max_inserter_tier capping ────────────────────────────────────────

    #[test]
    fn fast_cap_reproduces_v1_single_slot_ceiling() {
        // v1's bridge-anchor output ceiling (no fast/stack rung available
        // beyond fast, zero extra columns) was 2.31/s exactly — Fast-capped
        // v2 must reproduce that number.
        let plan = size_side(2.31, Reach::Near, 0, InserterTier::Fast);
        assert_eq!(plan, SidePlan { entity: FAST, count: 1, shortfall: None });

        let plan = size_side(2.32, Reach::Near, 0, InserterTier::Fast);
        assert_eq!(plan.entity, FAST);
        assert_eq!(plan.count, 1);
        assert!(plan.shortfall.is_some(), "above the fast-capped single-slot ceiling must shortfall, not escalate to stack");
    }

    #[test]
    fn regular_cap_never_places_fast_or_stack() {
        for required in [0.5, 2.0, 5.0, 20.0] {
            let plan = size_side(required, Reach::Near, 2, InserterTier::Regular);
            assert_eq!(plan.entity, REGULAR, "required={required}: Regular cap must never place fast/stack");
        }
    }

    #[test]
    fn regular_cap_uses_extra_columns_before_shortfalling() {
        // 1.5/s exceeds one regular (0.84) but two regular (1.68) covers
        // it, and Regular-capped still gets to use its free column.
        let plan = size_side(1.5, Reach::Near, 1, InserterTier::Regular);
        assert_eq!(plan, SidePlan { entity: REGULAR, count: 2, shortfall: None });
    }

    // ── reach-2 count-laddering ──────────────────────────────────────────

    #[test]
    fn far_single_long_handed_suffices() {
        let plan = size_side(1.0, Reach::Far, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: LONG_HANDED, count: 1, shortfall: None });
    }

    #[test]
    fn far_boundary_exact() {
        let plan = size_side(1.2, Reach::Far, 0, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: LONG_HANDED, count: 1, shortfall: None });
    }

    #[test]
    fn far_count_ladder_within_budget() {
        let plan = size_side(1.3, Reach::Far, 1, InserterTier::Stack);
        assert_eq!(plan, SidePlan { entity: LONG_HANDED, count: 2, shortfall: None });
    }

    #[test]
    fn far_ignores_max_tier_cap() {
        // No fast/stack long-handed exists — max_tier must not change the
        // far-reach outcome at all.
        let regular_capped = size_side(1.3, Reach::Far, 1, InserterTier::Regular);
        let stack_capped = size_side(1.3, Reach::Far, 1, InserterTier::Stack);
        assert_eq!(regular_capped, stack_capped);
    }

    // ── best-effort shortfall path ───────────────────────────────────────

    #[test]
    fn near_shortfall_beyond_budget_and_cap() {
        let plan = size_side(50.0, Reach::Near, 0, InserterTier::Stack);
        assert_eq!(plan.entity, STACK);
        assert_eq!(plan.count, 1);
        assert_eq!(plan.shortfall, Some(38.0));
    }

    #[test]
    fn far_shortfall_beyond_budget() {
        let plan = size_side(100.0, Reach::Far, 1, InserterTier::Stack);
        assert_eq!(plan.entity, LONG_HANDED);
        assert_eq!(plan.count, 2);
        assert_eq!(plan.shortfall, Some(100.0 - 2.4));
    }

    #[test]
    fn shortfall_none_when_sufficient() {
        let plan = size_side(0.5, Reach::Near, 0, InserterTier::Stack);
        assert!(plan.shortfall.is_none());
    }

    // ── determinism ──────────────────────────────────────────────────────

    #[test]
    fn determinism_repeated_calls_identical() {
        // 13.0/s exceeds a single stack inserter (12/s), so this only
        // resolves at rung 2+ (fast's count-ladder, per
        // `rung2_prefers_fast_count_over_stack_count`).
        for _ in 0..5 {
            let plan = size_side(13.0, Reach::Near, 5, InserterTier::Stack);
            assert_eq!(plan, SidePlan { entity: FAST, count: 6, shortfall: None });
        }
    }

    #[test]
    fn default_tier_is_stack() {
        assert_eq!(InserterTier::default(), InserterTier::Stack);
    }
}
