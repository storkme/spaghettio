//! Per-side inserter sizing ladder.
//!
//! `docs/rfc-inserter-sizing.md` — every row template used to place
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

use crate::common::{belt_drop_rate, inserter_throughput, QualityTier};

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

/// Single-tier count ladder: place 1..=`total_slots` copies of `entity`
/// (each moving `rate` items/s), cheapest count that covers `required`;
/// best-effort with an honest shortfall when even every slot falls short.
/// The shared core of the reach-2 (long-handed) ladder and the S>1
/// stack-forced belt-drop ladder.
fn count_ladder(entity: &'static str, rate: f64, required: f64, total_slots: usize) -> SidePlan {
    for n in 1..=total_slots {
        if required <= n as f64 * rate + EPS {
            return SidePlan { entity, count: n, shortfall: None };
        }
    }
    let placed = total_slots as f64 * rate;
    SidePlan { entity, count: total_slots, shortfall: Some((required - placed).max(0.0)) }
}

/// Size one machine side against an arbitrary per-entity `rate` table —
/// the rate-parametric core of [`size_side`] (flat machine-drop) and the
/// belt-drop entry points ([`size_belt_drop_side`] / [`size_side_output`],
/// which pass `common::belt_drop_rate`). `rate(name)` returns the items/s
/// one inserter of `name` moves on this side. Behavior is otherwise
/// identical to the historical `size_side`; passing the flat
/// `inserter_throughput` closure reproduces it bit-for-bit (kill 1).
fn size_side_rated(
    required: f64,
    reach: Reach,
    position_budget: usize,
    max_tier: InserterTier,
    rate: impl Fn(&str) -> f64,
) -> SidePlan {
    let total_slots = position_budget + 1;
    match reach {
        Reach::Far => count_ladder(LONG_HANDED, rate(LONG_HANDED), required, total_slots),
        Reach::Near => {
            let tiers = max_tier.allowed_near_entities();

            // Rung 0/1: one inserter, cheapest tier that suffices
            // ("in-place tier swap" — zero extra columns spent).
            for &entity in tiers {
                if required <= rate(entity) + EPS {
                    return SidePlan { entity, count: 1, shortfall: None };
                }
            }

            // Rung 2+: increase count into the free columns, cheapest
            // tier's count-ladder exhausted before moving to the next
            // tier up (matches the census: multi-fast fully tried before
            // any multi-stack).
            if total_slots > 1 {
                for &entity in tiers {
                    let r = rate(entity);
                    for n in 2..=total_slots {
                        if required <= n as f64 * r + EPS {
                            return SidePlan { entity, count: n, shortfall: None };
                        }
                    }
                }
            }

            // Best effort: every free column, richest tier the cap
            // allows, honest shortfall recorded.
            let entity = tiers.last().copied().unwrap_or(REGULAR);
            let placed = total_slots as f64 * rate(entity);
            SidePlan {
                entity,
                count: total_slots,
                shortfall: Some((required - placed).max(0.0)),
            }
        }
    }
}

/// Size one machine side (belt-pickup → machine-drop INPUT sides, and any
/// side with no belt-drop research/stacking scaling).
///
/// `required` is the per-machine, utilization-scaled rate the side must
/// move (items/s). `position_budget` is the number of EXTRA columns the
/// caller's template geometry can spare beyond the one baseline slot
/// every side already has (0 at the tightest positions, e.g. a
/// sideload-bridge anchor's output face). `max_tier` caps the near-reach
/// ladder; it has no effect on `Reach::Far` (see [`Reach`]).
///
/// Machine-drop (input) sides stay flat at every research level (RFC-049
/// kill 2 — no linear extrapolation on belt-pickup sides without measured
/// data), so this never takes a research `level`.
pub fn size_side(
    required: f64,
    reach: Reach,
    position_budget: usize,
    max_tier: InserterTier,
    quality: QualityTier,
) -> SidePlan {
    size_side_rated(required, reach, position_budget, max_tier, |name| {
        inserter_throughput(name, quality)
    })
}

/// Size a machine side that drops onto a **belt** (RFC-046 stacking +
/// RFC-049 inserter-capacity research).
///
/// At `stacking ≤ 1 && level == 0` this is exactly [`size_side`] —
/// bit-identical pre-RFC behavior (kill 1). Otherwise the per-tier rate
/// comes from `common::belt_drop_rate` (the SAME table the validator's
/// `belt_drop_throughput` reads):
///
/// - `Reach::Near` at **S > 1** is **forced to stack-inserter** (BS2: only
///   stack inserters create belt stacks; the ×S capacity credit on the fed
///   belt is honest only if every loader stacks). Count-laddered at
///   `belt_drop_rate(stack, quality, S, level)` = `swings ×
///   stack_inserter_belt_hand_at(level, S)` — note the mod-S hand dip
///   (S=4/L0 = 9.6/s; healed to 38.4/s at L7 where 16 ≡ 0 mod 4, but
///   REAPPEARING at L3/L4/L6, RFC-049). Layout entry already refused
///   `stacking > 1` with `max_inserter_tier < Stack`, so the forcing never
///   violates the user's cap (debug-asserted).
/// - `Reach::Near` at **S ≤ 1 && level > 0**: no belt stacking to force —
///   cheapest-sufficient tier ladder, but each tier's belt-drop rate is
///   research-scaled (`belt_drop_rate` at `stacking = 1`).
/// - `Reach::Far` cannot stack (BS5: no reach-2 stacking inserter), but its
///   long-handed belt-drop ceiling still rises with research (hand 1→2→4),
///   so it routes through the research-scaled rate too. Callers only pass
///   `Far` for stacking-exempt lanes; at level 0 this collapses back to
///   flat `size_side` behavior.
#[allow(clippy::too_many_arguments)]
pub fn size_belt_drop_side(
    required: f64,
    reach: Reach,
    position_budget: usize,
    max_tier: InserterTier,
    quality: QualityTier,
    stacking: u8,
    level: u8,
) -> SidePlan {
    // Kill 1: no stacking, no research → exactly the flat ladder.
    if stacking <= 1 && level == 0 {
        return size_side(required, reach, position_budget, max_tier, quality);
    }
    let rate = |name: &str| belt_drop_rate(name, quality, stacking, level);
    match reach {
        // Reach-2 can't stack (BS5); long-handed only, research-scaled.
        Reach::Far => size_side_rated(required, Reach::Far, position_budget, max_tier, rate),
        // S>1 forcing: only stack inserters create belt stacks.
        Reach::Near if stacking > 1 => {
            debug_assert!(
                max_tier == InserterTier::Stack,
                "layout entry must refuse stacking>1 with max_inserter_tier < Stack"
            );
            count_ladder(STACK, rate(STACK), required, position_budget + 1)
        }
        // S≤1, L>0: no forcing; cheapest-sufficient tier ladder at the
        // research-scaled belt-drop rates.
        Reach::Near => size_side_rated(required, Reach::Near, position_budget, max_tier, rate),
    }
}

/// Size a **stacking-exempt** belt-drop OUTPUT side (RFC-049 class (c):
/// Fulgora D2b secondary outputs, self-loop major/minor outputs — sites
/// that deliberately stay on the flat ladder under RFC-046 because their
/// lane family plans unstacked, so they must NOT be stack-forced).
///
/// The belt-drop hand still scales with inserter-capacity research (the
/// far long-handed output ceiling rises 1→2→4; near tiers scale linearly
/// too), so this is the cheapest-sufficient tier/count ladder at the
/// **unstacked** belt-drop rate (`belt_drop_rate` with `stacking = 1`) —
/// no stack-forcing, exemption intact. At `level == 0` it is bit-identical
/// to [`size_side`] (kill 1); a stacking value is never taken because the
/// family is exempt by construction.
pub fn size_side_output(
    required: f64,
    reach: Reach,
    position_budget: usize,
    max_tier: InserterTier,
    quality: QualityTier,
    level: u8,
) -> SidePlan {
    if level == 0 {
        return size_side(required, reach, position_budget, max_tier, quality);
    }
    size_side_rated(required, reach, position_budget, max_tier, |name| {
        belt_drop_rate(name, quality, 1, level)
    })
}

/// Name the binding constraint behind a CAPPED side plan (RFC
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
pub fn capped_limit(
    required: f64,
    plan: &SidePlan,
    lost_contest: bool,
    quality: QualityTier,
) -> &'static str {
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
        && size_side(required, Reach::Near, budget, InserterTier::Stack, quality).shortfall.is_none()
    {
        return "tier-cap";
    }
    if lost_contest && size_side(required, reach, budget + 1, tier_ceiling, quality).shortfall.is_none() {
        return "column-contest";
    }
    "geometry"
}

/// Ingredient-to-belt assignment for `dual_input_row`/`triple_input_row`'s
/// near/far pair (`docs/rfc-inserter-sizing.md`, lever (b)): the item with
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
/// (`docs/rfc-inserter-sizing.md`). Larger RELATIVE shortfall, measured
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
pub fn contest_favors_far(
    near_required: f64,
    far_required: f64,
    far_eligible: bool,
    quality: QualityTier,
) -> bool {
    if !far_eligible {
        return false;
    }
    let near_ceiling = inserter_throughput(STACK, quality);
    let far_ceiling = inserter_throughput(LONG_HANDED, quality);
    let near_shortfall = (near_required - near_ceiling).max(0.0);
    let far_shortfall = (far_required - far_ceiling).max(0.0);
    let near_rel = if near_required > 0.0 { near_shortfall / near_required } else { 0.0 };
    let far_rel = if far_required > 0.0 { far_shortfall / far_required } else { 0.0 };
    far_rel >= near_rel
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── size_belt_drop_side (RFC-046) ───────────────────────────────────

    /// S ≤ 1 AND level 0 is definitionally `size_side` — sweep rates,
    /// reaches, budgets, caps (kill 1: bit-identity floor across the
    /// stacking sweep at zero research).
    #[test]
    fn belt_drop_identity_at_s1_l0() {
        let q = QualityTier::Normal;
        for &s in &[0u8, 1u8] {
            for &required in &[0.3, 0.9, 2.5, 8.0, 13.0, 30.0] {
                for &reach in &[Reach::Near, Reach::Far] {
                    for budget in 0..=2usize {
                        for &cap in
                            &[InserterTier::Regular, InserterTier::Fast, InserterTier::Stack]
                        {
                            assert_eq!(
                                size_belt_drop_side(required, reach, budget, cap, q, s, 0),
                                size_side(required, reach, budget, cap, q),
                                "required={required} reach={reach:?} budget={budget} cap={cap:?} s={s}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// S > 1 near sides force stack-inserter even at rates a regular
    /// inserter would cover — that is the point (only stack inserters
    /// create stacks, BS2).
    #[test]
    fn belt_drop_forces_stack_above_s1() {
        let q = QualityTier::Normal;
        let plan = size_belt_drop_side(0.5, Reach::Near, 0, InserterTier::Stack, q, 2, 0);
        assert_eq!(plan.entity, STACK);
        assert_eq!(plan.count, 1);
        assert_eq!(plan.shortfall, None);
    }

    /// Counts follow the belt-drop decomposition, including the S=4 hand
    /// dip: 20/s needs 2 stack inserters at S=3 (14.4/s each) but 3 at
    /// S=4 (9.6/s each).
    #[test]
    fn belt_drop_counts_track_hand_dip() {
        let q = QualityTier::Normal;
        let s3 = size_belt_drop_side(20.0, Reach::Near, 2, InserterTier::Stack, q, 3, 0);
        assert_eq!((s3.entity, s3.count, s3.shortfall), (STACK, 2, None));
        let s4 = size_belt_drop_side(20.0, Reach::Near, 2, InserterTier::Stack, q, 4, 0);
        assert_eq!((s4.entity, s4.count, s4.shortfall), (STACK, 3, None));
        // Budget exhausted → honest shortfall at the belt-drop rate.
        let capped = size_belt_drop_side(20.0, Reach::Near, 0, InserterTier::Stack, q, 4, 0);
        assert_eq!(capped.entity, STACK);
        assert_eq!(capped.count, 1);
        let sf = capped.shortfall.expect("must record shortfall");
        assert!((sf - (20.0 - 9.6)).abs() < 1e-9);
    }

    /// Far sides can never stack (BS5): passthrough to the long-handed
    /// count ladder at any S.
    #[test]
    fn belt_drop_far_passthrough() {
        let q = QualityTier::Normal;
        assert_eq!(
            size_belt_drop_side(1.0, Reach::Far, 1, InserterTier::Stack, q, 4, 0),
            size_side(1.0, Reach::Far, 1, InserterTier::Stack, q),
        );
    }

    /// Quality scales swings on the belt-drop path (BS7): legendary at
    /// S=4 is 24/s per inserter.
    #[test]
    fn belt_drop_quality_scaling() {
        let plan = size_belt_drop_side(
            24.0,
            Reach::Near,
            0,
            InserterTier::Stack,
            QualityTier::Legendary,
            4,
            0,
        );
        assert_eq!((plan.entity, plan.count, plan.shortfall), (STACK, 1, None));
    }

    /// RFC-049 non-monotonicity pin: at S=4 the researched belt-hand
    /// rounds DOWN to a multiple of 4 (`stack_inserter_belt_hand_at`), so
    /// L2/L3/L4 all plateau at hand 8 (19.2/s) — the naive "each research
    /// level adds capacity" model is WRONG here — then L5 jumps to hand 12
    /// (28.8/s). An intermediate level therefore places MORE inserters than
    /// a higher one at equal rate: at 20/s, L4 needs 2 stack inserters where
    /// L5 needs 1, and L4 == L2 (the dip/plateau, not just endpoints).
    #[test]
    fn belt_drop_intermediate_dip_non_monotonic_s4() {
        let q = QualityTier::Normal;
        let count_at = |level: u8| {
            size_belt_drop_side(20.0, Reach::Near, 2, InserterTier::Stack, q, 4, level).count
        };
        let (l2, l4, l5) = (count_at(2), count_at(4), count_at(5));
        assert_eq!(l4, 2, "L4/S=4 belt-hand floors to 8 (19.2/s) → 2 inserters for 20/s");
        assert_eq!(l5, 1, "L5/S=4 belt-hand is 12 (28.8/s) → 1 inserter for 20/s");
        assert!(l4 > l5, "the mod-4 dip: L4 places MORE than L5 at equal rate (2 > 1)");
        assert_eq!(l4, l2, "plateau: L2=L3=L4 (hands 8/9/10 all floor to 8) — research gives no benefit until L5");
    }

    // ── size_side_output (RFC-049 class (c): stacking-exempt outputs) ────

    /// Level 0 is definitionally `size_side` (kill 1) — sweep rates,
    /// reaches, budgets, caps.
    #[test]
    fn size_side_output_identity_at_l0() {
        let q = QualityTier::Normal;
        for &required in &[0.3, 0.9, 2.5, 8.0, 13.0, 30.0] {
            for &reach in &[Reach::Near, Reach::Far] {
                for budget in 0..=2usize {
                    for &cap in &[InserterTier::Regular, InserterTier::Fast, InserterTier::Stack] {
                        assert_eq!(
                            size_side_output(required, reach, budget, cap, q, 0),
                            size_side(required, reach, budget, cap, q),
                            "required={required} reach={reach:?} budget={budget} cap={cap:?}"
                        );
                    }
                }
            }
        }
    }

    /// The far (long-handed) output ceiling genuinely rises with research
    /// (hand 1→2→4): 4.0/s shortfalls on one LHI at L0 (1.2/s) but a
    /// single LHI covers it at L7 (4.8/s) — no belt stacking involved.
    #[test]
    fn size_side_output_far_ceiling_rises_with_research() {
        let q = QualityTier::Normal;
        let l0 = size_side_output(4.0, Reach::Far, 0, InserterTier::Stack, q, 0);
        assert_eq!(l0.entity, LONG_HANDED);
        assert!(l0.shortfall.is_some(), "4.0/s exceeds one LHI (1.2/s) at L0");
        let l7 = size_side_output(4.0, Reach::Far, 0, InserterTier::Stack, q, 7);
        assert_eq!((l7.entity, l7.count, l7.shortfall), (LONG_HANDED, 1, None));
    }

    /// Exemption intact: a near output side is NEVER stack-forced (unlike
    /// `size_belt_drop_side` at S>1) — a low rate still gets the cheapest
    /// tier even at max research.
    #[test]
    fn size_side_output_near_never_forces_stack() {
        let q = QualityTier::Normal;
        let plan = size_side_output(0.5, Reach::Near, 0, InserterTier::Stack, q, 7);
        assert_eq!(plan.entity, REGULAR, "0.5/s is covered by a regular inserter; exempt side must not force stack");
        assert_eq!(plan.count, 1);
    }

    // ── constants identity ──────────────────────────────────────────────
    // The ladder must source every rate from `common::inserter_throughput`
    // — the same table the validator reads — never a duplicated literal.
    // This test pins the values that table currently returns; if it drifts,
    // this fails loudly instead of the ladder and the check silently
    // disagreeing.

    #[test]
    fn constants_identity() {
        assert_eq!(inserter_throughput(REGULAR, QualityTier::Normal), 0.84);
        assert_eq!(inserter_throughput(FAST, QualityTier::Normal), 2.31);
        assert_eq!(inserter_throughput(STACK, QualityTier::Normal), 12.0);
        assert_eq!(inserter_throughput(LONG_HANDED, QualityTier::Normal), 1.2);
        // Per-tier scaling (rfc-build-quality kill 2a, inserter third):
        // Normal is the bit-exact base above; legendary is ×2.5 exactly
        // (0.3×5 = 1.5 is exact in f64). Mid-tiers approx-checked.
        assert_eq!(inserter_throughput(STACK, QualityTier::Legendary), 30.0);
        assert_eq!(inserter_throughput(LONG_HANDED, QualityTier::Legendary), 3.0);
        assert_eq!(inserter_throughput(REGULAR, QualityTier::Legendary), 2.1);
        for tier in QualityTier::ALL {
            for name in [REGULAR, FAST, STACK, LONG_HANDED] {
                let expect = inserter_throughput(name, QualityTier::Normal) * tier.multiplier();
                assert!(
                    (inserter_throughput(name, tier) - expect).abs() < 1e-12,
                    "{name} {tier:?}"
                );
            }
        }
    }

    /// The belt-drop path (RFC-046 stacking + RFC-049 research) must source
    /// its per-inserter rate from `common::belt_drop_rate` — the SAME
    /// function the validator's `belt_drop_throughput` reads — so the
    /// ladder and the check can never disagree. This pins the values that
    /// function currently returns at the load-bearing (name, S, level)
    /// points; drift fails loudly here rather than silently splitting the
    /// fix from the check.
    #[test]
    fn belt_drop_constants_identity() {
        use crate::common::belt_drop_rate;
        let q = QualityTier::Normal;
        // L0 collapses to the flat I8 constant at every S≤1 (kill 1) and to
        // the RFC-046 stack decomposition at S>1 (level-0 sibling baseline).
        assert_eq!(belt_drop_rate(STACK, q, 1, 0), 12.0);
        assert_eq!(belt_drop_rate(FAST, q, 1, 0), 2.31);
        assert_eq!(belt_drop_rate(REGULAR, q, 1, 0), 0.84);
        assert!((belt_drop_rate(STACK, q, 2, 0) - 14.4).abs() < 1e-9);
        assert!((belt_drop_rate(STACK, q, 4, 0) - 9.6).abs() < 1e-9); // S=4 dip
        // Research scales the stack belt-hand: L7/S=4 heals (16 ≡ 0 mod 4)
        // to 38.4/s = 2.4 × 16; the intermediate dip levels plateau at
        // 19.2/s (L2=L3=L4, hands 8/9/10 all floor to 8) then jump at L5.
        assert!((belt_drop_rate(STACK, q, 4, 7) - 38.4).abs() < 1e-9);
        assert!((belt_drop_rate(STACK, q, 4, 2) - 19.2).abs() < 1e-9);
        assert!((belt_drop_rate(STACK, q, 4, 3) - 19.2).abs() < 1e-9);
        assert!((belt_drop_rate(STACK, q, 4, 4) - 19.2).abs() < 1e-9);
        assert!((belt_drop_rate(STACK, q, 4, 5) - 28.8).abs() < 1e-9);
        // Non-bulk output ceilings rise linearly by hand (1→2→4): far
        // long-handed goes 1.2 → 4.8 at L7; regular/fast scale likewise.
        assert!((belt_drop_rate(LONG_HANDED, q, 1, 7) - 1.2 * 4.0).abs() < 1e-9);
        assert!((belt_drop_rate(FAST, q, 1, 7) - 2.31 * 4.0).abs() < 1e-9);
        assert!((belt_drop_rate(REGULAR, q, 1, 2) - 0.84 * 2.0).abs() < 1e-9);
        // Bulk stays flat at every level (never placed; conservative floor).
        assert_eq!(belt_drop_rate("bulk-inserter", q, 1, 7), inserter_throughput("bulk-inserter", q));
    }

    // ── capped_limit derivation (RFC validation-explainability D2) ──────

    /// The anchor mechanism: EC@35s far side needs 1.4583/s, one LHI moves
    /// 1.2/s. Lost the shared column → the budget+1 counterfactual (2.4/s)
    /// covers → "column-contest"; same plan without a lost contest is
    /// honest "geometry".
    #[test]
    fn capped_limit_column_contest_matches_anchor() {
        let required = 1.4583333333333333;
        let plan = size_side(required, Reach::Far, 0, InserterTier::Stack, QualityTier::Normal);
        assert!(plan.shortfall.is_some());
        assert_eq!(capped_limit(required, &plan, true, QualityTier::Normal), "column-contest");
        assert_eq!(capped_limit(required, &plan, false, QualityTier::Normal), "geometry");
    }

    /// Near side capped at Regular where a stack inserter at the SAME
    /// budget would cover → the user's max_inserter_tier is binding.
    #[test]
    fn capped_limit_tier_cap() {
        let required = 2.0; // > 0.84 regular, < 12.0 stack
        let plan = size_side(required, Reach::Near, 0, InserterTier::Regular, QualityTier::Normal);
        assert!(plan.shortfall.is_some());
        assert_eq!(plan.entity, REGULAR);
        assert_eq!(capped_limit(required, &plan, false, QualityTier::Normal), "tier-cap");
        // tier-cap outranks contest: even a lost contest reports the
        // user-controllable knob first.
        assert_eq!(capped_limit(required, &plan, true, QualityTier::Normal), "tier-cap");
    }

    /// Demand beyond any single-column relief or tier upgrade → geometry,
    /// even when a contest was lost (the counterfactual doesn't cover).
    #[test]
    fn capped_limit_geometry_when_nothing_helps() {
        let required = 100.0;
        let plan = size_side(required, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert!(plan.shortfall.is_some());
        assert_eq!(plan.entity, STACK);
        assert_eq!(capped_limit(required, &plan, true, QualityTier::Normal), "geometry");
    }

    // ── rung boundaries (Near, Stack cap, single slot) ──────────────────

    #[test]
    fn rung0_regular_at_boundary() {
        let plan = size_side(0.84, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: REGULAR, count: 1, shortfall: None });
    }

    #[test]
    fn rung0_regular_below_boundary() {
        let plan = size_side(0.5, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: REGULAR, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_fast_just_above_regular() {
        // Just over the regular ceiling — must NOT jump to stack.
        let plan = size_side(0.8401, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: FAST, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_fast_at_boundary() {
        let plan = size_side(2.31, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: FAST, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_stack_just_above_fast() {
        // Just over the fast ceiling — must NOT skip straight past
        // without trying fast first (cheapest-sufficient).
        let plan = size_side(2.311, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: STACK, count: 1, shortfall: None });
    }

    #[test]
    fn rung1_stack_at_boundary() {
        let plan = size_side(12.0, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: STACK, count: 1, shortfall: None });
    }

    // ── cheapest-sufficient (KC6): never stack where fast suffices ─────

    #[test]
    fn cheapest_sufficient_never_stack_when_fast_covers() {
        for required in [0.85, 1.0, 1.5, 2.0, 2.31] {
            let plan = size_side(required, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
            assert_ne!(
                plan.entity, STACK,
                "required={required}: fast (2.31/s) covers this, must not place stack"
            );
        }
    }

    #[test]
    fn cheapest_sufficient_never_fast_when_regular_covers() {
        for required in [0.1, 0.5, 0.84] {
            let plan = size_side(required, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
            assert_eq!(plan.entity, REGULAR, "required={required}: regular alone covers this");
        }
    }

    // ── extra-column rung (count-ladder within budget) ──────────────────

    #[test]
    fn rung2_multi_stack_within_budget() {
        // 13/s exceeds one stack inserter (12/s) but two cover it (24/s),
        // and a free column is available.
        let plan = size_side(13.0, Reach::Near, 1, InserterTier::Stack, QualityTier::Normal);
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
        let plan = size_side(13.0, Reach::Near, 5, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: FAST, count: 6, shortfall: None });
    }

    // ── max_inserter_tier capping ────────────────────────────────────────

    #[test]
    fn fast_cap_reproduces_v1_single_slot_ceiling() {
        // v1's bridge-anchor output ceiling (no fast/stack rung available
        // beyond fast, zero extra columns) was 2.31/s exactly — Fast-capped
        // v2 must reproduce that number.
        let plan = size_side(2.31, Reach::Near, 0, InserterTier::Fast, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: FAST, count: 1, shortfall: None });

        let plan = size_side(2.32, Reach::Near, 0, InserterTier::Fast, QualityTier::Normal);
        assert_eq!(plan.entity, FAST);
        assert_eq!(plan.count, 1);
        assert!(plan.shortfall.is_some(), "above the fast-capped single-slot ceiling must shortfall, not escalate to stack");
    }

    #[test]
    fn regular_cap_never_places_fast_or_stack() {
        for required in [0.5, 2.0, 5.0, 20.0] {
            let plan = size_side(required, Reach::Near, 2, InserterTier::Regular, QualityTier::Normal);
            assert_eq!(plan.entity, REGULAR, "required={required}: Regular cap must never place fast/stack");
        }
    }

    #[test]
    fn regular_cap_uses_extra_columns_before_shortfalling() {
        // 1.5/s exceeds one regular (0.84) but two regular (1.68) covers
        // it, and Regular-capped still gets to use its free column.
        let plan = size_side(1.5, Reach::Near, 1, InserterTier::Regular, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: REGULAR, count: 2, shortfall: None });
    }

    // ── reach-2 count-laddering ──────────────────────────────────────────

    #[test]
    fn far_single_long_handed_suffices() {
        let plan = size_side(1.0, Reach::Far, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: LONG_HANDED, count: 1, shortfall: None });
    }

    #[test]
    fn far_boundary_exact() {
        let plan = size_side(1.2, Reach::Far, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: LONG_HANDED, count: 1, shortfall: None });
    }

    #[test]
    fn far_count_ladder_within_budget() {
        let plan = size_side(1.3, Reach::Far, 1, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan, SidePlan { entity: LONG_HANDED, count: 2, shortfall: None });
    }

    #[test]
    fn far_ignores_max_tier_cap() {
        // No fast/stack long-handed exists — max_tier must not change the
        // far-reach outcome at all.
        let regular_capped = size_side(1.3, Reach::Far, 1, InserterTier::Regular, QualityTier::Normal);
        let stack_capped = size_side(1.3, Reach::Far, 1, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(regular_capped, stack_capped);
    }

    // ── best-effort shortfall path ───────────────────────────────────────

    #[test]
    fn near_shortfall_beyond_budget_and_cap() {
        let plan = size_side(50.0, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan.entity, STACK);
        assert_eq!(plan.count, 1);
        assert_eq!(plan.shortfall, Some(38.0));
    }

    #[test]
    fn far_shortfall_beyond_budget() {
        let plan = size_side(100.0, Reach::Far, 1, InserterTier::Stack, QualityTier::Normal);
        assert_eq!(plan.entity, LONG_HANDED);
        assert_eq!(plan.count, 2);
        assert_eq!(plan.shortfall, Some(100.0 - 2.4));
    }

    #[test]
    fn shortfall_none_when_sufficient() {
        let plan = size_side(0.5, Reach::Near, 0, InserterTier::Stack, QualityTier::Normal);
        assert!(plan.shortfall.is_none());
    }

    // ── determinism ──────────────────────────────────────────────────────

    #[test]
    fn determinism_repeated_calls_identical() {
        // 13.0/s exceeds a single stack inserter (12/s), so this only
        // resolves at rung 2+ (fast's count-ladder, per
        // `rung2_prefers_fast_count_over_stack_count`).
        for _ in 0..5 {
            let plan = size_side(13.0, Reach::Near, 5, InserterTier::Stack, QualityTier::Normal);
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
        assert!(contest_favors_far(2.0, 5.0, true, QualityTier::Normal));
    }

    #[test]
    fn contest_near_wins_when_near_shortfall_larger() {
        // near: 20.0/s vs its 12.0/s ceiling -> large relative shortfall.
        // far: 1.0/s, under the 1.2/s ceiling -> zero.
        assert!(!contest_favors_far(20.0, 1.0, true, QualityTier::Normal));
    }

    #[test]
    fn contest_tie_favors_far() {
        // Equal relative shortfall (both zero, well under their own
        // ceilings) -> tie breaks to far.
        assert!(contest_favors_far(0.5, 0.5, true, QualityTier::Normal));
    }

    #[test]
    fn contest_far_ineligible_near_wins_unconditionally() {
        // Even a huge far requirement can't win when the position's own
        // geometry has excluded far from the tile (e.g. LastInRow).
        assert!(!contest_favors_far(0.1, 100.0, false, QualityTier::Normal));
    }
}
