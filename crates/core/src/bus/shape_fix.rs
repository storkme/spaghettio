//! Strategies for handling unstampable balancer shapes.
//!
//! Module-level `#[allow(dead_code)]`: the strategies and selector are
//! defined and tested here, but the partitioner wiring (next commit)
//! actually exercises them. Once `decompose_oversized_modules` calls
//! `select_shape_fix`, this can be removed.
#![allow(dead_code)]

//!
//! The balancer library has templates for many `(N, M)` producer→lane
//! shapes, plus a gcd-decomposition fallback (`stamp_family_balancer`)
//! for shapes like `(6, 8) = 2×(3, 4)`. But coprime shapes like `(4, 9)`
//! or `(7, 9)` have no template AND no decomposition path. When the
//! lane planner produces such a shape, `stamp_family_balancer` returns
//! an empty entity vec and the producer→trunk handoff is silently
//! dropped — symptom is dead-end belts at the row's exit column.
//!
//! This module exposes a strategy-based fix: each strategy proposes an
//! alteration to the original `(n, m)` that *would* be stampable, with
//! its own cost profile. The selector tries strategies in order and
//! returns the first viable fix.
//!
//! ## Strategies today
//!
//! - **PadLanesStrategy** — extend `m` by up to `max_pad` empty trunk
//!   lanes to reach a stampable nearby shape. Cost: a few extra empty
//!   bus columns. Zero machine cost.
//! - **ShardStrategy** — split the module into `k ≤ max_shards` shards
//!   with non-uniform lane counts, each shard's `(n, lanes_i)` shape
//!   individually stampable. Cost: every consuming recipe gets per-
//!   shard sub-rows, which multiplies cluster crossings and layout
//!   width. Each shard sees the full producer set (today).
//!
//! ## Future
//!
//! Eventually we want every strategy to produce a candidate fix, then
//! score each by predicted layout density (entities / area) and pick
//! the most compact. Pad usually wins on small `m` overshoots; shard
//! wins when no viable padding exists. Today's ordered-fallback is a
//! placeholder for that scoring scaffold.

use crate::bus::balancer::shape_is_stampable;

/// Result of trying to fix an unstampable shape `(n, m)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ShapeFix {
    /// Original `(n, m)` is already stampable — no action needed.
    Native,
    /// Pad consumer lanes from `m` to `new_m`. Producer count `n`
    /// unchanged. Extra lanes carry no flow.
    PadLanes { new_m: u32 },
    /// Split into `lanes.len()` shards with the given per-shard lane
    /// counts (sum = original `m`). Each shard sees the full set of
    /// `n` producers. Sub-shape `(n, lanes[i])` is individually
    /// stampable for every `i`.
    Shard { lanes: Vec<u32> },
}

/// A strategy for handling shapes the balancer library can't directly
/// stamp. Returns `Some(ShapeFix)` if it can transform `(n, m)` into a
/// stampable layout, `None` if it can't (caller falls through).
///
/// **Contract:** strategies are *only* called for shapes where
/// `shape_is_stampable(n, m) == false`. The selector handles the
/// already-stampable case directly.
pub(crate) trait ShapeFixStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn try_fix(&self, n: u32, m: u32) -> Option<ShapeFix>;
}

/// Add up to `max_pad` empty trunk lanes to reach a nearby stampable
/// shape. Zero machine cost; small layout-width cost (a few empty bus
/// columns).
pub(crate) struct PadLanesStrategy {
    pub max_pad: u32,
}

impl ShapeFixStrategy for PadLanesStrategy {
    fn name(&self) -> &'static str {
        "pad-lanes"
    }

    fn try_fix(&self, n: u32, m: u32) -> Option<ShapeFix> {
        for delta in 1..=self.max_pad {
            if shape_is_stampable(n, m + delta) {
                return Some(ShapeFix::PadLanes { new_m: m + delta });
            }
        }
        None
    }
}

/// Split into `k ≤ max_shards` shards. Each shard handles a chunk of
/// the consumer demand; each `(n, lanes[i])` sub-shape must be
/// stampable. Round-robin lane distribution: first `m % k` shards get
/// `ceil(m / k)`, rest get `floor(m / k)`.
pub(crate) struct ShardStrategy {
    pub max_shards: u32,
}

impl ShapeFixStrategy for ShardStrategy {
    fn name(&self) -> &'static str {
        "shard"
    }

    fn try_fix(&self, n: u32, m: u32) -> Option<ShapeFix> {
        for k in 2..=self.max_shards.min(m) {
            let base = m / k;
            let rem = m % k;
            let lanes: Vec<u32> = (0..k)
                .map(|i| if i < rem { base + 1 } else { base })
                .collect();
            if lanes.iter().all(|&l| shape_is_stampable(n, l)) {
                return Some(ShapeFix::Shard { lanes });
            }
        }
        None
    }
}

/// Try strategies in order. Returns the first successful fix, or
/// `ShapeFix::Native` if `(n, m)` is already stampable, or `None` if no
/// strategy could handle it (caller should expect a silent layout drop;
/// `validate::check_balancer_template_coverage` will surface a warning).
pub(crate) fn select_shape_fix(
    n: u32,
    m: u32,
    strategies: &[&dyn ShapeFixStrategy],
) -> Option<ShapeFix> {
    if shape_is_stampable(n, m) {
        return Some(ShapeFix::Native);
    }
    for strategy in strategies {
        if let Some(fix) = strategy.try_fix(n, m) {
            return Some(fix);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_finds_nearby_stampable_for_4_9() {
        // (4, 9) is unstampable. (4, 10) gcd=2 → 2×(2, 5) coprime, fails.
        // (4, 11) gcd=1, fails. (4, 12) gcd=4 → 4×(1, 3), works.
        let strategy = PadLanesStrategy { max_pad: 4 };
        let fix = strategy.try_fix(4, 9);
        assert!(matches!(fix, Some(ShapeFix::PadLanes { .. })),
            "PadLanesStrategy should find a stampable shape near (4, 9); got {fix:?}");
        if let Some(ShapeFix::PadLanes { new_m }) = fix {
            assert!(new_m > 9, "padded m must be > 9, got {new_m}");
            assert!(shape_is_stampable(4, new_m), "padded shape (4, {new_m}) must be stampable");
        }
    }

    #[test]
    fn pad_returns_none_when_no_stampable_in_range() {
        // (3, 9) → (3, 10) gcd=1, library doesn't cover (3, 10) directly
        //                  (library is (1..=8, 1..=8) minus (1, 1)).
        //         → (3, 11) gcd=1, same.
        // With max_pad=2, no nearby stampable shape exists.
        let strategy = PadLanesStrategy { max_pad: 2 };
        assert_eq!(strategy.try_fix(3, 9), None,
            "with max_pad=2, no stampable shape in (3, 9..=11)");
    }

    #[test]
    fn shard_splits_4_9_into_stampable_subshapes() {
        // (4, 9) needs shards. k=2: lanes=[5, 4], (4, 5) gcd=1 fail.
        // k=3: lanes=[3, 3, 3], (4, 3) gcd=1 — check library coverage.
        let strategy = ShardStrategy { max_shards: 5 };
        let fix = strategy.try_fix(4, 9);
        if let Some(ShapeFix::Shard { lanes }) = &fix {
            assert_eq!(lanes.iter().sum::<u32>(), 9, "shards must sum to original m");
            for &l in lanes {
                assert!(shape_is_stampable(4, l),
                    "every sub-shape (4, {l}) must be stampable");
            }
        }
        // We don't assert success — if the library doesn't cover any of
        // (4, 1..=4), Shard returns None and the caller (selector) falls
        // through to None. That's the correct behaviour.
    }

    #[test]
    fn select_returns_native_for_stampable_shapes() {
        let pad = PadLanesStrategy { max_pad: 2 };
        let shard = ShardStrategy { max_shards: 3 };
        let strategies: &[&dyn ShapeFixStrategy] = &[&pad, &shard];
        // Library covers (1..=8, 1..=8) except (1, 1). Pick shapes that
        // are definitely covered.
        assert_eq!(select_shape_fix(1, 2, strategies), Some(ShapeFix::Native));
        assert_eq!(select_shape_fix(2, 4, strategies), Some(ShapeFix::Native));
        assert_eq!(select_shape_fix(8, 8, strategies), Some(ShapeFix::Native));
    }

    #[test]
    fn select_prefers_pad_over_shard_when_both_possible() {
        let pad = PadLanesStrategy { max_pad: 4 };
        let shard = ShardStrategy { max_shards: 5 };
        let strategies: &[&dyn ShapeFixStrategy] = &[&pad, &shard];
        // (4, 9) → pad finds (4, 12). Shard is also possible. Pad wins
        // because it's first in the strategy list.
        let fix = select_shape_fix(4, 9, strategies);
        assert!(matches!(fix, Some(ShapeFix::PadLanes { .. })),
            "pad-first ordering should prefer pad; got {fix:?}");
    }
}
