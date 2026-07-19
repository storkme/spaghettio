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

/// Name the binding constraint behind a CAPPED side plan (RFP
/// validation-explainability D2 — the `limit` field of the
/// `InserterSideCapped` trace event). Valid only when
/// `plan.shortfall.is_some()`: a capped plan is best-effort, so it used
/// every slot at the richest allowed tier — which makes both the budget
/// (`count - 1` extra columns) and the reach (`LONG_HANDED` ⇔ far) and
/// the allowed tier ceiling (`plan.entity`) recoverable from the plan
/// itself, keeping the ~30 template emit sites mechanical.
///
/// Precedence, most-actionable first:
/// 1. `"tier-cap"` — near side, and stack tier at the SAME budget would
///    cover: the user's `max_inserter_tier` is the binding constraint.
/// 2. `"column-contest"` — the caller says this side lost the shared
///    near/far column, and the counterfactual (budget + 1 at the same
///    tier ceiling) would cover: the contest is the binding constraint.
/// 3. `"geometry"` — the row shape offers no slot that would help.
pub fn capped_limit(required: f64, plan: &SidePlan, lost_contest: bool) -> &'static str {
    debug_assert!(plan.shortfall.is_some(), "capped_limit is only defined for capped plans");
    let budget = plan.count.saturating_sub(1);
    let (reach, tier_ceiling) = match plan.entity {
        LONG_HANDED => (Reach::Far, InserterTier::Stack), // far ignores tier
        FAST => (Reach::Near, InserterTier::Fast),
        REGULAR => (Reach::Near, InserterTier::Regular),
        _ => (Reach::Near, InserterTier::Stack),
    };
    if reach == Reach::Near
        && plan.entity != STACK
        && size_side(required, Reach::Near, budget, InserterTier::Stack).shortfall.is_none()
    {
        return "tier-cap";
    }
    if lost_contest && size_side(required, reach, budget + 1, tier_ceiling).shortfall.is_none() {
        return "column-contest";
    }
    "geometry"
}

/// Ingredient-to-belt assignment for `dual_input_row`/`triple_input_row`'s
/// near/far pair (`docs/rfp-inserter-sizing.md`, lever (b)): the item with
/// the higher per-machine rate goes to the NEAR (reach-1) belt, where the
/// full regular→fast→stack ladder applies — no fast/stack long-handed
/// inserter exists, so the FAR (reach-2) belt stays low-ceiling regardless,
/// making it worth keeping the hungrier ingredient off it. Ties (and
/// `rate0 <= rate1`) preserve the caller's structural default (`item0`
/// stays far, `item1` stays near) — deterministic, minimizes golden churn.
/// Returns `(far, near)`, each paired with its rate. Mirrors the census's
/// `reassign_near_far` exactly.
pub fn reassign_near_far<T>(item0: T, rate0: f64, item1: T, rate1: f64) -> ((T, f64), (T, f64)) {
    if rate0 > rate1 + EPS {
        ((item1, rate1), (item0, rate0))
    } else {
        ((item0, rate0), (item1, rate1))
    }
}

/// Resolve an extra inserter column shared by two competing machine
/// sides — dual/triple's near-vs-far input pair, triple's
/// input3-vs-output tile, quad's south-input-vs-output tile
/// (`docs/rfp-inserter-sizing.md`). Larger RELATIVE shortfall, measured
/// against each side's own single-slot top-tier ceiling (stack for
/// `Reach::Near`, one long-handed for `Reach::Far`), wins the shared
/// slot; ties favor the far/reach-2 side. `far_eligible` is `false` when
/// the position's own geometry has already excluded far from the tile
/// (e.g. `dual_input_row` at `LastInRow`, where the far belt itself is
/// trimmed away, or a bridge-collapsed position where the tile doesn't
/// exist for anyone) — near then wins (or, when the tile doesn't exist
/// for near either, the caller simply never offers it a budget). Mirrors
/// `examples/census_inserter_sizing_v2.rs`'s `resolve_contests` exactly —
/// single source of truth per KC6's audit discipline. This function only
/// decides who is ENTITLED to the slot; whether the winner's own ladder
/// is threaded yet (near always is; far's reach-2 ladder is Phase 3) is
/// the caller's concern — a far win with no far ladder simply leaves the
/// slot unused rather than handing it to near.
pub fn contest_favors_far(near_required: f64, far_required: f64, far_eligible: bool) -> bool {
    if !far_eligible {
        return false;
    }
    let near_ceiling = inserter_throughput(STACK);
    let far_ceiling = inserter_throughput(LONG_HANDED);
    let near_shortfall = (near_required - near_ceiling).max(0.0);
    let far_shortfall = (far_required - far_ceiling).max(0.0);
    let near_rel = if near_required > 0.0 { near_shortfall / near_required } else { 0.0 };
    let far_rel = if far_required > 0.0 { far_shortfall / far_required } else { 0.0 };
    far_rel >= near_rel
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

    // ── capped_limit derivation (RFP validation-explainability D2) ──────

    /// The anchor mechanism: EC@35s far side needs 1.4583/s, one LHI moves
    /// 1.2/s. Lost the shared column → the budget+1 counterfactual (2.4/s)
    /// covers → "column-contest"; same plan without a lost contest is
    /// honest "geometry".
    #[test]
    fn capped_limit_column_contest_matches_anchor() {
        let required = 1.4583333333333333;
        let plan = size_side(required, Reach::Far, 0, InserterTier::Stack);
        assert!(plan.shortfall.is_some());
        assert_eq!(capped_limit(required, &plan, true), "column-contest");
        assert_eq!(capped_limit(required, &plan, false), "geometry");
    }

    /// Near side capped at Regular where a stack inserter at the SAME
    /// budget would cover → the user's max_inserter_tier is binding.
    #[test]
    fn capped_limit_tier_cap() {
        let required = 2.0; // > 0.84 regular, < 12.0 stack
        let plan = size_side(required, Reach::Near, 0, InserterTier::Regular);
        assert!(plan.shortfall.is_some());
        assert_eq!(plan.entity, REGULAR);
        assert_eq!(capped_limit(required, &plan, false), "tier-cap");
        // tier-cap outranks contest: even a lost contest reports the
        // user-controllable knob first.
        assert_eq!(capped_limit(required, &plan, true), "tier-cap");
    }

    /// Demand beyond any single-column relief or tier upgrade → geometry,
    /// even when a contest was lost (the counterfactual doesn't cover).
    #[test]
    fn capped_limit_geometry_when_nothing_helps() {
        let required = 100.0;
        let plan = size_side(required, Reach::Near, 0, InserterTier::Stack);
        assert!(plan.shortfall.is_some());
        assert_eq!(plan.entity, STACK);
        assert_eq!(capped_limit(required, &plan, true), "geometry");
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

    // ── reassign_near_far (ingredient-to-belt assignment, lever (b)) ────

    #[test]
    fn reassign_item0_hungrier_swaps_to_near() {
        // item0 (iron-plate, 5.0) is hungrier than item1 (copper-plate,
        // 1.5) -> item0 swaps onto near, item1 becomes far.
        let (far, near) = reassign_near_far("iron-plate", 5.0, "copper-plate", 1.5);
        assert_eq!(far, ("copper-plate", 1.5));
        assert_eq!(near, ("iron-plate", 5.0));
    }

    #[test]
    fn reassign_item1_hungrier_keeps_default() {
        // item1 (iron-plate, 5.0) is hungrier than item0 (copper-plate,
        // 1.5) -> already in the default far/near slots, no swap.
        let (far, near) = reassign_near_far("copper-plate", 1.5, "iron-plate", 5.0);
        assert_eq!(far, ("copper-plate", 1.5));
        assert_eq!(near, ("iron-plate", 5.0));
    }

    #[test]
    fn reassign_tie_keeps_default_order() {
        let (far, near) = reassign_near_far("iron-plate", 3.0, "copper-plate", 3.0);
        assert_eq!(far, ("iron-plate", 3.0));
        assert_eq!(near, ("copper-plate", 3.0));
    }

    // ── contest_favors_far (shared-column resolution) ────────────────────

    #[test]
    fn contest_far_wins_when_far_shortfall_larger() {
        // far: 5.0/s vs its 1.2/s ceiling -> large relative shortfall.
        // near: 2.0/s, comfortably under the 12.0/s stack ceiling -> zero.
        assert!(contest_favors_far(2.0, 5.0, true));
    }

    #[test]
    fn contest_near_wins_when_near_shortfall_larger() {
        // near: 20.0/s vs its 12.0/s ceiling -> large relative shortfall.
        // far: 1.0/s, under the 1.2/s ceiling -> zero.
        assert!(!contest_favors_far(20.0, 1.0, true));
    }

    #[test]
    fn contest_tie_favors_far() {
        // Equal relative shortfall (both zero, well under their own
        // ceilings) -> tie breaks to far.
        assert!(contest_favors_far(0.5, 0.5, true));
    }

    #[test]
    fn contest_far_ineligible_near_wins_unconditionally() {
        // Even a huge far requirement can't win when the position's own
        // geometry has excluded far from the tile (e.g. LastInRow).
        assert!(!contest_favors_far(0.1, 100.0, false));
    }
}
