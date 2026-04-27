//! WIP: junction-solver eviction strategy.
//!
//! See `docs/rfp-junction-solver-capability.md` for the design. This file
//! is a scaffold — the recipes and routing logic are not yet implemented.
//! The strategy is wired in but `try_solve` returns `None` unconditionally,
//! so behaviour is unchanged from main.
//!
//! Plan summary (from the spike write-up — full text in
//! `/root/.claude/plans/try-them-all-cosmic-cat.md` on the original
//! environment):
//!
//! - Slot `EvictionStrategy` after the SAT variants in
//!   `ghost_router.rs:1978`. It only fires when SAT has returned UNSAT.
//! - Each recipe = `(spec selector, route emitter)`. Apply, then
//!   re-invoke `SatStrategy::unrestricted()` on the filtered junction.
//! - Filtered junction: clone `ctx.junction`, drop the evicted specs
//!   from `specs`, union the evicted specs' `routed_paths` tiles inside
//!   the bbox AND the new route's tiles into `forbidden`.
//! - Filtered region: clone `ctx.region`, drop evicted keys from
//!   `participating`. Build a stand-in `JunctionStrategyContext` and
//!   call `SatStrategy::try_solve` on it.
//! - Merge entities: SAT entities + A*-routed entities. Include evicted
//!   spec keys in `JunctionSolution.participating` so the framework's
//!   release-and-stamp pass clears their ghost belts inside the bbox.
//! - Per-recipe budget 50ms, total budget 200ms.
//! - Recipe order: `OppositePairUg` (geometric), `AstarLongest{1,2}`,
//!   `AstarMostConflicting{1}`, `AstarShortest{1}`, `AstarTurnsRich{1}`.
//!   UG-preferred routing (high `turn_penalty`) is recipes 4-5 of the
//!   plan but render_path emits UGs only on `dist > 1` between
//!   consecutive path tiles, so default A* produces all-surface paths.
//!   To get UGs, add a path-compression post-pass that replaces
//!   straight runs with single hops; out of scope for the first draft.
//!
//! Trace events (added in `trace.rs`): `EvictionAttempted`,
//! `EvictionRouteFailed`, `EvictionSatFailed`, `EvictionSucceeded`,
//! `EvictionBudgetExhausted`. Diagnostic helpers go in
//! `tests/e2e.rs::diag_eviction_recipe_grid`.

use crate::bus::junction_solver::{JunctionSolution, JunctionStrategy, JunctionStrategyContext};

/// Which spec selector + route emitter combo to try.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum EvictionRecipe {
    /// Geometric pre-pass: for every spec where entry/exit sit on
    /// opposite walls of the bbox in the same row/column AND the gap
    /// fits within `ug_max_reach(belt_tier) + 1`, emit a 2-entity UG
    /// pair and drop the spec from SAT input.
    OppositePairUg,
    /// Pick the `count` longest specs by Manhattan(entry, exit), route
    /// each via `ghost_astar` with default `turn_penalty = 8`.
    /// `ug_preferred = true` cranks `turn_penalty` to ~100 so the path
    /// is mostly straight runs that a future path-compression pass can
    /// turn into UG hops (today's `render_path` produces all-surface).
    AstarLongest { count: usize, ug_preferred: bool },
    /// Pick the `count` specs whose straight-line bbox overlaps the
    /// most other specs' lines. Hypothesis: high-conflict specs are
    /// the ones SAT struggles to fit; evicting them simplifies the
    /// remaining encoding.
    AstarMostConflicting { count: usize },
    /// Opposite hypothesis to longest: short specs are the easiest to
    /// route around; evicting them leaves the harder ones to SAT.
    AstarShortest { count: usize },
    /// Pick the spec with the worst Manhattan-vs-direct-line ratio
    /// (proxy for "needs many turns"). Evict one.
    AstarTurnsRich { count: usize },
}

#[allow(dead_code)]
impl EvictionRecipe {
    pub fn name(&self) -> &'static str {
        match self {
            Self::OppositePairUg => "OppositePairUg",
            Self::AstarLongest { ug_preferred: false, .. } => "AstarLongest",
            Self::AstarLongest { ug_preferred: true, .. } => "AstarLongestUgPreferred",
            Self::AstarMostConflicting { .. } => "AstarMostConflicting",
            Self::AstarShortest { .. } => "AstarShortest",
            Self::AstarTurnsRich { .. } => "AstarTurnsRich",
        }
    }
}

/// Eviction strategy. WIP — currently a no-op that always returns `None`.
pub struct EvictionStrategy {
    #[allow(dead_code)]
    recipes: Vec<EvictionRecipe>,
    #[allow(dead_code)]
    per_recipe_budget_ms: u64,
    #[allow(dead_code)]
    total_budget_ms: u64,
}

impl EvictionStrategy {
    /// Default recipe set + budgets per the spike plan. Trying each in
    /// order, tightest geometric pre-pass first.
    pub fn default_recipes() -> Self {
        Self {
            recipes: vec![
                EvictionRecipe::OppositePairUg,
                EvictionRecipe::AstarLongest { count: 1, ug_preferred: false },
                EvictionRecipe::AstarLongest { count: 2, ug_preferred: false },
                EvictionRecipe::AstarLongest { count: 1, ug_preferred: true },
                EvictionRecipe::AstarMostConflicting { count: 1 },
                EvictionRecipe::AstarShortest { count: 1 },
                EvictionRecipe::AstarTurnsRich { count: 1 },
            ],
            per_recipe_budget_ms: 50,
            total_budget_ms: 200,
        }
    }
}

impl JunctionStrategy for EvictionStrategy {
    fn name(&self) -> &'static str {
        "eviction"
    }

    fn try_solve(&self, _ctx: &JunctionStrategyContext) -> Option<JunctionSolution> {
        // TODO(eviction):
        //   1. Iterate `self.recipes` under `total_budget_ms` cap.
        //   2. For each recipe: select specs, emit
        //      `TraceEvent::EvictionAttempted`, route them
        //      (OppositePairUg = geometric 2-tile UG pair via
        //      `render_path`; A* recipes = `crate::astar::ghost_astar`
        //      → `render_path`).
        //   3. Build filtered Junction (clone, drop evicted specs from
        //      `specs`, union evicted-spec path tiles in bbox + new
        //      route tiles into `forbidden`). Build filtered
        //      GrowingRegion (clone, drop evicted keys from
        //      `participating`).
        //   4. Construct a fresh `JunctionStrategyContext` and call
        //      `SatStrategy::unrestricted().try_solve(&filtered_ctx)`.
        //   5. On SAT success: merge entities (SAT + evicted routes),
        //      append evicted keys to `JunctionSolution.participating`,
        //      emit `EvictionSucceeded`, return.
        //   6. On SAT failure: emit `EvictionSatFailed`, try next recipe.
        //   7. After all recipes: emit `EvictionBudgetExhausted`,
        //      return `None`.
        //
        // Call sites needed:
        //   - `crate::astar::ghost_astar` (in `astar.rs`).
        //   - `crate::bus::trunk_renderer::render_path` (currently
        //     `pub(crate)`).
        //   - `crate::common::ug_max_reach`, `BeltTier::belt_name`.
        //   - `crate::bus::junction_sat_strategy::SatStrategy::unrestricted()`.
        //
        // See `docs/rfp-junction-solver-capability.md` and the WIP plan
        // file for the full design rationale.
        None
    }
}
