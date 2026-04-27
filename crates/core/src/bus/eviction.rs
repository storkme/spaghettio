//! Junction-solver eviction strategy.
//!
//! Slots in after the SAT variants in `ghost_router.rs` and only fires
//! when every SAT variant has returned UNSAT for this iteration. The
//! strategy tries an ordered list of **eviction recipes**: each recipe
//! selects one or more participating specs, routes them around or
//! through the bbox via `ghost_astar` (or a geometric pre-pass for
//! `OppositePairUg`), then re-invokes SAT on the *remaining* spec set.
//! First success wins.
//!
//! The pipeline is always `evict N specs → route them → SAT-on-remainder`.
//! Eviction is never a complete solve on its own — if SAT comes back
//! UNSAT on the reduced problem we move to the next recipe.
//!
//! See `docs/rfp-junction-solver-capability.md` for the design rationale.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::astar::ghost_astar;
use crate::bus::junction::{BeltTier, Junction, Rect, SpecCrossing};
use crate::bus::junction_sat_strategy::SatStrategy;
use crate::bus::junction_solver::{
    GrowingRegion, JunctionSolution, JunctionStrategy, JunctionStrategyContext,
};
use crate::bus::trunk_renderer::render_path;
use crate::common::ug_max_reach;
use crate::models::{EntityDirection, PlacedEntity};
use crate::trace::{self, EvictionSpecMetric, TraceEvent};

/// Which spec selector + route emitter combo to try.
#[derive(Debug, Clone, Copy)]
pub enum EvictionRecipe {
    /// Geometric pre-pass: for every spec where entry/exit sit on
    /// opposite walls of the bbox in the same row/column AND the gap
    /// fits within `ug_max_reach(belt_tier) + 1`, emit a 2-entity UG
    /// pair and drop the spec from SAT input.
    OppositePairUg,
    /// Pick the `count` longest specs by Manhattan(entry, exit). Route
    /// each via `ghost_astar`. `ug_preferred = true` cranks
    /// `turn_penalty` from 8 to 100 so A* prefers straight runs.
    /// (Today's `render_path` only emits UG pairs when consecutive
    /// path tiles are non-adjacent — A* always emits adjacent steps,
    /// so `ug_preferred` produces all-surface paths until a
    /// path-compression post-pass lands. Kept as a knob for that
    /// follow-up.)
    AstarLongest { count: usize, ug_preferred: bool },
    /// Pick the `count` specs whose entry/exit straight line crosses
    /// the most other specs' lines (estimated via bbox intersection).
    AstarMostConflicting { count: usize },
    /// Opposite hypothesis to `AstarLongest`: short specs are easiest
    /// to route around, freeing SAT to focus on the harder ones.
    AstarShortest { count: usize },
    /// Pick the `count` specs whose `manhattan / direct_line` ratio is
    /// worst (proxy for "needs many turns").
    AstarTurnsRich { count: usize },
}

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

pub struct EvictionStrategy {
    recipes: Vec<EvictionRecipe>,
    total_budget_ms: u64,
}

impl EvictionStrategy {
    /// Default recipe set + budgets per the spike plan.
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
            total_budget_ms: 200,
        }
    }
}

impl JunctionStrategy for EvictionStrategy {
    fn name(&self) -> &'static str {
        "eviction"
    }

    fn try_solve(&self, ctx: &JunctionStrategyContext) -> Option<JunctionSolution> {
        let total_started = web_time::Instant::now();
        let total_deadline =
            total_started + std::time::Duration::from_millis(self.total_budget_ms);

        let n_participating = ctx.region.participating.len();
        if n_participating < 2 || ctx.region.tile_count() <= 1 {
            return None;
        }
        // Only fire on the last growth iteration. Earlier iterations
        // give the region a chance to grow into a SAT-solvable shape;
        // eviction is a fallback when growth has run out of room.
        // Gating prevents eviction from beating SAT on cases that would
        // have solved cleanly after one or two more growth steps.
        if ctx.growth_iter + 1 < crate::bus::junction_solver::MAX_GROWTH_ITERS {
            return None;
        }

        let sat_strategy = SatStrategy::unrestricted();
        let mut recipes_tried = 0usize;
        let seed = ctx.region.initial_tile;

        for recipe in &self.recipes {
            if web_time::Instant::now() >= total_deadline {
                break;
            }
            recipes_tried += 1;

            let selected = select_specs(recipe, ctx);
            if selected.is_empty() {
                continue;
            }
            // Don't evict every participating spec — SAT needs at least
            // one belt spec to have something to solve.
            if selected.len() >= n_participating {
                continue;
            }

            trace::emit(TraceEvent::EvictionAttempted {
                seed_x: seed.0,
                seed_y: seed.1,
                iter: ctx.growth_iter,
                recipe: recipe.name().to_string(),
                candidate_spec_keys: selected.clone(),
                region_tiles: ctx.region.tile_count(),
                boundary_count_before: ctx.junction.specs.len() * 2,
            });

            let route_started = web_time::Instant::now();
            let mut all_route_entities: Vec<PlacedEntity> = Vec::new();
            let mut metrics: Vec<EvictionSpecMetric> = Vec::new();
            let mut route_failed = false;
            for spec_key in &selected {
                let one_started = web_time::Instant::now();
                match route_evicted_spec(recipe, spec_key, ctx, &all_route_entities) {
                    Some((entities, metric)) => {
                        all_route_entities.extend(entities);
                        metrics.push(metric);
                    }
                    None => {
                        trace::emit(TraceEvent::EvictionRouteFailed {
                            seed_x: seed.0,
                            seed_y: seed.1,
                            recipe: recipe.name().to_string(),
                            spec_key: spec_key.clone(),
                            reason: "no_path".to_string(),
                            elapsed_us: one_started.elapsed().as_micros() as u64,
                        });
                        route_failed = true;
                        break;
                    }
                }
            }
            let route_us = route_started.elapsed().as_micros() as u64;
            if route_failed {
                continue;
            }

            // Build filtered Junction: drop evicted specs; mark their
            // path tiles inside bbox + the new route's tiles forbidden.
            let evicted_set: FxHashSet<&str> =
                selected.iter().map(|s| s.as_str()).collect();
            let mut filtered_specs: Vec<SpecCrossing> =
                Vec::with_capacity(ctx.junction.specs.len());
            for (i, sc) in ctx.junction.specs.iter().enumerate() {
                let evicted = i < n_participating
                    && evicted_set.contains(ctx.region.participating[i].as_str());
                if !evicted {
                    filtered_specs.push(sc.clone());
                }
            }
            let mut filtered_forbidden = ctx.junction.forbidden.clone();
            for spec_key in &selected {
                if let Some(path) = ctx.routed_paths.get(spec_key) {
                    for &t in path {
                        if ctx.junction.bbox.contains(t.0, t.1) {
                            filtered_forbidden.insert(t);
                        }
                    }
                }
            }
            for ent in &all_route_entities {
                filtered_forbidden.insert((ent.x, ent.y));
            }
            let filtered_junction = Junction {
                bbox: ctx.junction.bbox,
                forbidden: filtered_forbidden,
                specs: filtered_specs,
            };

            // Filtered region: clone, drop evicted keys from
            // `participating`. SAT only reads `tile_count` and
            // `participating`, so this is enough.
            let mut filtered_region: GrowingRegion = ctx.region.clone();
            filtered_region
                .participating
                .retain(|k| !evicted_set.contains(k.as_str()));

            let filtered_ctx = JunctionStrategyContext {
                junction: &filtered_junction,
                region: &filtered_region,
                growth_iter: ctx.growth_iter,
                growth_variant: ctx.growth_variant,
                routed_paths: ctx.routed_paths,
                hard_obstacles: ctx.hard_obstacles,
                strict_obstacles: ctx.strict_obstacles,
                placed_entities: ctx.placed_entities,
                unreleasable_obstacles: ctx.unreleasable_obstacles,
            };

            let sat_started = web_time::Instant::now();
            let sat_solution = sat_strategy.try_solve(&filtered_ctx);
            let sat_us = sat_started.elapsed().as_micros() as u64;

            let Some(sat_sol) = sat_solution else {
                trace::emit(TraceEvent::EvictionSatFailed {
                    seed_x: seed.0,
                    seed_y: seed.1,
                    recipe: recipe.name().to_string(),
                    evicted_spec_keys: selected.clone(),
                    elapsed_us: sat_us,
                });
                continue;
            };

            let mut merged_entities = sat_sol.entities;
            merged_entities.extend(all_route_entities);
            let mut merged_participating = sat_sol.participating.clone();
            for k in &selected {
                if !merged_participating.contains(k) {
                    merged_participating.push(k.clone());
                }
            }

            let total_us = total_started.elapsed().as_micros() as u64;
            trace::emit(TraceEvent::EvictionSucceeded {
                seed_x: seed.0,
                seed_y: seed.1,
                iter: ctx.growth_iter,
                recipe: recipe.name().to_string(),
                evicted_spec_keys: selected,
                boundary_count_after: filtered_junction.specs.len() * 2,
                sat_us,
                route_us,
                total_us,
                metrics,
            });

            return Some(JunctionSolution {
                entities: merged_entities,
                footprint: sat_sol.footprint,
                strategy_name: "eviction",
                participating: merged_participating,
                sat_zone: sat_sol.sat_zone,
            });
        }

        trace::emit(TraceEvent::EvictionBudgetExhausted {
            seed_x: seed.0,
            seed_y: seed.1,
            recipes_tried,
            total_us: total_started.elapsed().as_micros() as u64,
        });
        None
    }
}

// ---------------------------------------------------------------------------
// Spec selectors
// ---------------------------------------------------------------------------

fn manhattan(sc: &SpecCrossing) -> u32 {
    ((sc.exit.x - sc.entry.x).abs() + (sc.exit.y - sc.entry.y).abs()) as u32
}

fn select_specs(recipe: &EvictionRecipe, ctx: &JunctionStrategyContext) -> Vec<String> {
    let n = ctx.region.participating.len();
    let pairs: Vec<(&String, &SpecCrossing)> = ctx
        .region
        .participating
        .iter()
        .zip(ctx.junction.specs.iter().take(n))
        .collect();

    match recipe {
        EvictionRecipe::OppositePairUg => pairs
            .iter()
            .filter(|(_, sc)| opposite_pair_fits(sc, &ctx.junction.bbox))
            .map(|(k, _)| (*k).clone())
            .collect(),

        EvictionRecipe::AstarLongest { count, .. } => {
            let mut by_len: Vec<(&String, u32)> =
                pairs.iter().map(|(k, sc)| (*k, manhattan(sc))).collect();
            by_len.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            by_len.iter().take(*count).map(|(k, _)| (*k).clone()).collect()
        }

        EvictionRecipe::AstarShortest { count } => {
            let mut by_len: Vec<(&String, u32)> =
                pairs.iter().map(|(k, sc)| (*k, manhattan(sc))).collect();
            by_len.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));
            by_len.iter().take(*count).map(|(k, _)| (*k).clone()).collect()
        }

        EvictionRecipe::AstarMostConflicting { count } => {
            // Score = number of other specs whose entry-exit bbox
            // overlaps this one's.
            let mut scored: Vec<(&String, u32)> = pairs
                .iter()
                .map(|(k, sc)| (*k, count_conflicts(sc, &pairs)))
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            scored.iter().take(*count).map(|(k, _)| (*k).clone()).collect()
        }

        EvictionRecipe::AstarTurnsRich { count } => {
            // Higher manhattan-vs-direct deficit = more turns implied.
            let mut scored: Vec<(&String, u32)> = pairs
                .iter()
                .map(|(k, sc)| {
                    let dx = (sc.exit.x - sc.entry.x).unsigned_abs();
                    let dy = (sc.exit.y - sc.entry.y).unsigned_abs();
                    let direct = dx.max(dy);
                    let manh = dx + dy;
                    let score = manh.saturating_sub(direct);
                    (*k, score)
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            scored.iter().take(*count).map(|(k, _)| (*k).clone()).collect()
        }
    }
}

fn count_conflicts(sc: &SpecCrossing, all: &[(&String, &SpecCrossing)]) -> u32 {
    let mut count = 0u32;
    for (_, other) in all {
        if std::ptr::eq(*other, sc) {
            continue;
        }
        if line_bbox_overlaps(sc, other) {
            count += 1;
        }
    }
    count
}

fn line_bbox_overlaps(a: &SpecCrossing, b: &SpecCrossing) -> bool {
    let a_x_min = a.entry.x.min(a.exit.x);
    let a_x_max = a.entry.x.max(a.exit.x);
    let a_y_min = a.entry.y.min(a.exit.y);
    let a_y_max = a.entry.y.max(a.exit.y);
    let b_x_min = b.entry.x.min(b.exit.x);
    let b_x_max = b.entry.x.max(b.exit.x);
    let b_y_min = b.entry.y.min(b.exit.y);
    let b_y_max = b.entry.y.max(b.exit.y);
    a_x_min <= b_x_max && a_x_max >= b_x_min && a_y_min <= b_y_max && a_y_max >= b_y_min
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wall {
    N,
    S,
    E,
    W,
    Interior,
}

fn which_wall(x: i32, y: i32, bbox: &Rect) -> Wall {
    let max_x = bbox.x + bbox.w as i32 - 1;
    let max_y = bbox.y + bbox.h as i32 - 1;
    if y == bbox.y {
        Wall::N
    } else if y == max_y {
        Wall::S
    } else if x == bbox.x {
        Wall::W
    } else if x == max_x {
        Wall::E
    } else {
        Wall::Interior
    }
}

fn opposite_pair_fits(sc: &SpecCrossing, bbox: &Rect) -> bool {
    let entry_wall = which_wall(sc.entry.x, sc.entry.y, bbox);
    let exit_wall = which_wall(sc.exit.x, sc.exit.y, bbox);
    let opposite = matches!(
        (entry_wall, exit_wall),
        (Wall::N, Wall::S) | (Wall::S, Wall::N) | (Wall::E, Wall::W) | (Wall::W, Wall::E)
    );
    if !opposite {
        return false;
    }
    let same_axis = match entry_wall {
        Wall::N | Wall::S => sc.entry.x == sc.exit.x,
        Wall::E | Wall::W => sc.entry.y == sc.exit.y,
        Wall::Interior => false,
    };
    if !same_axis {
        return false;
    }
    let max_reach = ug_max_reach(sc.belt_tier.belt_name());
    let gap = manhattan(sc);
    // UG pair: entry and exit can be up to max_reach + 1 tiles apart
    // (max_reach hidden tiles in between).
    gap > 0 && gap <= max_reach + 1
}

// ---------------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------------

/// Route an evicted spec through the bbox. Returns the entities to
/// stamp + a metric snapshot. `already_routed` carries entities from
/// previously-evicted specs in the same recipe so this call's A* can
/// avoid them.
fn route_evicted_spec(
    recipe: &EvictionRecipe,
    spec_key: &str,
    ctx: &JunctionStrategyContext,
    already_routed: &[PlacedEntity],
) -> Option<(Vec<PlacedEntity>, EvictionSpecMetric)> {
    let idx = ctx.region.participating.iter().position(|k| k == spec_key)?;
    let sc = ctx.junction.specs.get(idx)?;
    let entry = (sc.entry.x, sc.entry.y);
    let exit = (sc.exit.x, sc.exit.y);
    let belt_name = sc.belt_tier.belt_name();
    let item = sc.item.clone();
    let bbox = ctx.junction.bbox;

    let path = match recipe {
        EvictionRecipe::OppositePairUg => {
            // 2-tile path triggers `render_path`'s UG-emission branch.
            vec![entry, exit]
        }
        EvictionRecipe::AstarLongest { ug_preferred, .. } => {
            astar_in_bbox(spec_key, sc, ctx, already_routed, if *ug_preferred { 100 } else { 8 })?
        }
        EvictionRecipe::AstarMostConflicting { .. }
        | EvictionRecipe::AstarShortest { .. }
        | EvictionRecipe::AstarTurnsRich { .. } => {
            astar_in_bbox(spec_key, sc, ctx, already_routed, 8)?
        }
    };

    let direction_hint = sc.exit.direction;
    let segment_id = Some(format!("crossing:{}:{}", bbox.x, bbox.y));
    let entities = render_path(&path, &item, belt_name, direction_hint, segment_id, None);
    if entities.is_empty() {
        return None;
    }

    let metric = EvictionSpecMetric {
        spec_key: spec_key.to_string(),
        item,
        belt_tier: belt_name.to_string(),
        manhattan_len: manhattan(sc),
        turn_count: count_turns(&path),
        entity_count: entities.len(),
    };
    Some((entities, metric))
}

/// Run `ghost_astar` constrained to inside the bbox. Tiles outside
/// `bbox` are added to the hard set so A* can't escape.
fn astar_in_bbox(
    spec_key: &str,
    sc: &SpecCrossing,
    ctx: &JunctionStrategyContext,
    already_routed: &[PlacedEntity],
    turn_penalty: u32,
) -> Option<Vec<(i32, i32)>> {
    let bbox = ctx.junction.bbox;
    let max_x = bbox.x + bbox.w as i32;
    let max_y = bbox.y + bbox.h as i32;

    let mut hard: FxHashSet<(i32, i32)> = FxHashSet::default();
    // Frame everything outside bbox.
    for x in 0..max_x {
        for y in 0..max_y {
            if !bbox.contains(x, y) {
                hard.insert((x, y));
            }
        }
    }
    // Existing junction-level forbidden (machines, pipes, etc).
    hard.extend(ctx.junction.forbidden.iter().copied());
    // Other participating specs' path tiles inside bbox.
    for other_key in ctx.region.participating.iter() {
        if other_key == spec_key {
            continue;
        }
        if let Some(path) = ctx.routed_paths.get(other_key) {
            for &t in path {
                if bbox.contains(t.0, t.1) {
                    hard.insert(t);
                }
            }
        }
    }
    // Already-routed entities from the same recipe.
    for ent in already_routed {
        hard.insert((ent.x, ent.y));
    }
    // Make sure entry / exit are reachable.
    let entry = (sc.entry.x, sc.entry.y);
    let exit = (sc.exit.x, sc.exit.y);
    hard.remove(&entry);
    hard.remove(&exit);

    let empty: FxHashSet<(i32, i32)> = FxHashSet::default();
    let empty_grid: FxHashMap<(i32, i32), (u32, u32)> = FxHashMap::default();
    let (path, _crossings) = ghost_astar(
        entry,
        exit,
        &hard,
        &empty,
        max_x,
        max_y,
        turn_penalty,
        &empty_grid,
    )?;
    Some(path)
}

fn count_turns(path: &[(i32, i32)]) -> u32 {
    if path.len() < 3 {
        return 0;
    }
    let mut turns = 0u32;
    let mut last = (path[1].0 - path[0].0, path[1].1 - path[0].1);
    for w in path.windows(2).skip(1) {
        let d = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        if d != last && d != (0, 0) {
            turns += 1;
            last = d;
        }
    }
    turns
}

// Suppress unused-import warning when feature flags strip a binding.
#[allow(dead_code)]
fn _unused_imports() {
    let _ = EntityDirection::East;
    let _ = BeltTier::Yellow;
}
