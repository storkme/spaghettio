//! SAT-based junction strategy — wraps `crate::sat::solve_crossing_zone`
//! over a grown region.
//!
//! Only fires on regions that have grown past the initial single-tile
//! crossing: a 1×1 zone has entry==exit for every spec, which is not a
//! valid `CrossingZone`. Once the growth loop has walked each
//! participating spec's path at least one step outward, the bbox is
//! large enough that the spec entries and exits sit at distinct
//! boundary tiles, and the SAT encoder can route the interior.
//!
//! The mapping is mechanical:
//!
//! - `Junction.bbox`          → `CrossingZone { x, y, width, height }`
//! - `SpecCrossing.entry`     → `ZoneBoundary { ..., is_input: true, belt_tier: None, channel_id: 0 }`
//! - `SpecCrossing.exit`      → `ZoneBoundary { ..., is_input: false, belt_tier: None, channel_id: 0 }`
//! - `Junction.forbidden`     → `CrossingZone.forced_empty`
//!
//! Belt tier + UG max reach are picked from the dominant (highest-rank)
//! tier across the participating specs. If the region mixes tiers the
//! SAT solution uses the fastest belts everywhere — fine for
//! correctness, possibly wasteful for throughput-limited downstream
//! checks. Revisit if mixed-tier junctions turn out to be common.

use rustc_hash::FxHashSet;

use crate::bus::junction::{BeltTier, Rect, SpecCrossing, SpecKind};
use crate::bus::junction_solver::{JunctionSolution, JunctionStrategy, JunctionStrategyContext};
use crate::common::{is_splitter, is_surface_belt, is_ug_belt, splitter_second_tile, ug_max_reach};
use crate::models::{EntityDirection, PlacedEntity};
use crate::sat::{CrossingZone, ZoneBoundary};
use crate::trace::{self, BoundarySnapshot, ExternalFeederSnapshot, SatProposedEntity, TraceEvent};

/// A feeder/consumer tile candidate found adjacent to a spec entry/exit.
struct FeederHit {
    /// The tile of the Permanent entity that physically interacts with
    /// the boundary (for splitters, the specific one of two tiles).
    entity_tile: (i32, i32),
    /// The Permanent entity's facing direction.
    entity_direction: EntityDirection,
    /// The Permanent entity's name — used by `topology_boundaries` to
    /// stamp the per-boundary `belt_tier` metadata.
    entity_name: String,
}

/// Canonical surface-belt name for an entity's tier — `"transport-belt"`
/// / `"fast-transport-belt"` / `"express-transport-belt"`. Accepts belts,
/// undergrounds and splitters. Returns `None` only for entities that
/// aren't in the belt family (shouldn't happen at boundary construction
/// sites, where the entity is always a belt/UG/splitter).
fn tier_name_for(entity_name: &str) -> Option<String> {
    BeltTier::from_name(entity_name).map(|t| t.belt_name().to_string())
}

/// Name of the entity occupying `tile`, if any. Splitters occupy two
/// tiles and will match on either. Used by `topology_boundaries` to
/// look up the external receiver at an OUT boundary so its tier gets
/// recorded on the boundary instead of the in-zone entity's tier.
fn entity_name_at(placed_entities: &[PlacedEntity], tile: (i32, i32)) -> Option<String> {
    for e in placed_entities {
        if (e.x, e.y) == tile {
            return Some(e.name.clone());
        }
        if is_splitter(&e.name) {
            let (sx, sy) = splitter_second_tile(e);
            if (sx, sy) == tile {
                return Some(e.name.clone());
            }
        }
    }
    None
}

/// How to pick per-channel UG-reach caps when solving a zone.
///
/// **Native** — each channel's reach equals its declared belt tier
/// (yellow=5, red=7, blue=9). Forces SAT to pick UG pairs that fit
/// the tier of the external flow, so the solve-time entity names are
/// automatically correct — no post-pass retyping needed. Fails on
/// zones that genuinely need longer-than-native UG spans to route.
///
/// **Relaxed** — every channel uses the zone's max-tier reach, same
/// as the pre-refactor behaviour. More flexible (any channel can use
/// blue's 9-tile UG) but loses per-tier correctness: a yellow flow
/// routed through a 9-tile UG gets stamped as blue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReachMode {
    Native,
    Relaxed,
}

/// Knobs the outer loop tunes when asking SAT to solve a junction.
/// Strategies in priority order: the first config that satisfies AND
/// passes the walker wins. New cost dimensions (entity budget, specific
/// per-item constraints, …) should land here so the outer loop stays a
/// plain list of configs without ballooning into per-dimension strategy
/// structs.
#[derive(Debug, Clone, Copy)]
pub struct SatConstraints {
    /// Maximum number of underground-belt input tiles. Each UG-in
    /// pairs with exactly one UG-out, so this effectively caps the
    /// number of UG corridors in the zone.
    /// - `None`: unlimited — the original SAT behaviour.
    /// - `Some(0)`: surface-only. Forbids UG entirely.
    /// - `Some(k)`: at most `k` corridors. Used to spend the UG budget
    ///   only where surface routing is genuinely infeasible (real
    ///   crossings, tight turns), keeping everything else on the
    ///   surface.
    pub max_ug_ins: Option<u32>,
    /// Max iterations of the post-solve cost-descent loop. After SAT
    /// finds a first layout with cost C, the loop re-solves with
    /// `cost ≤ C-1` up to this many times. 0 disables descent.
    pub cost_descent_max_iters: u8,
    /// Wall-clock budget (ms) for the descent loop, checked between
    /// iterations. Prevents pathological zones from blocking the
    /// solver for too long.
    pub cost_descent_budget_ms: u32,
    /// Whether to use per-channel native reaches (tight) or the
    /// zone's max tier reach (loose) for UG pairing.
    pub reach_mode: ReachMode,
}

impl SatConstraints {
    /// Unrestricted — matches the original SAT behaviour.
    pub const fn unrestricted() -> Self {
        Self {
            max_ug_ins: None,
            cost_descent_max_iters: 4,
            cost_descent_budget_ms: 50,
            reach_mode: ReachMode::Relaxed,
        }
    }

    /// Hard-forbid underground-belt entities.
    pub const fn surface_only() -> Self {
        Self {
            max_ug_ins: Some(0),
            cost_descent_max_iters: 4,
            cost_descent_budget_ms: 50,
            reach_mode: ReachMode::Relaxed,
        }
    }

    /// Cap the number of UG corridors at `n`.
    pub const fn max_ug_ins(n: u32) -> Self {
        Self {
            max_ug_ins: Some(n),
            cost_descent_max_iters: 4,
            cost_descent_budget_ms: 50,
            reach_mode: ReachMode::Relaxed,
        }
    }

    /// Same as `max_ug_ins(n)` but with per-channel native reach — each
    /// UG pair's run length is bounded by the channel's declared tier.
    /// The native-reach rungs run before the relaxed ladder so mixed-
    /// tier zones get tier-correct UG lengths when possible.
    pub const fn max_ug_ins_native(n: u32) -> Self {
        Self {
            max_ug_ins: Some(n),
            cost_descent_max_iters: 4,
            cost_descent_budget_ms: 50,
            reach_mode: ReachMode::Native,
        }
    }

    /// Unrestricted UG budget with per-channel native reach. Final
    /// native-mode fallback before the ladder falls through to the
    /// relaxed-reach rungs.
    pub const fn unrestricted_native() -> Self {
        Self {
            max_ug_ins: None,
            cost_descent_max_iters: 4,
            cost_descent_budget_ms: 50,
            reach_mode: ReachMode::Native,
        }
    }
}

impl Default for SatConstraints {
    fn default() -> Self {
        Self::unrestricted()
    }
}

pub struct SatStrategy {
    name: &'static str,
    constraints: SatConstraints,
}

impl SatStrategy {
    /// Custom-named strategy with arbitrary constraints. Used when the
    /// pre-canned variants below aren't enough.
    pub const fn with(name: &'static str, constraints: SatConstraints) -> Self {
        Self { name, constraints }
    }

    /// Surface-only pass. Tried first so SAT only reaches for UG when
    /// it genuinely can't route on the surface.
    pub const fn surface_only() -> Self {
        Self::with("sat-surface", SatConstraints::surface_only())
    }

    /// Unrestricted pass. The original SAT behaviour — used as the
    /// fallback when budgeted passes fail.
    pub const fn unrestricted() -> Self {
        Self::with("sat", SatConstraints::unrestricted())
    }
}

/// Direction vector for N/E/S/W.
fn dir_delta(d: EntityDirection) -> (i32, i32) {
    match d {
        EntityDirection::North => (0, -1),
        EntityDirection::East => (1, 0),
        EntityDirection::South => (0, 1),
        EntityDirection::West => (-1, 0),
    }
}

/// Human-readable direction label for trace events.
fn dir_label(d: EntityDirection) -> String {
    match d {
        EntityDirection::North => "North",
        EntityDirection::East => "East",
        EntityDirection::South => "South",
        EntityDirection::West => "West",
    }
    .to_string()
}

/// Find any entity in `placed` whose output lands on `tile`, for use in
/// BoundarySnapshot.external_feeder.
fn find_external_feeder(
    tile: (i32, i32),
    placed: &[PlacedEntity],
) -> Option<ExternalFeederSnapshot> {
    for e in placed {
        if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input") {
            continue;
        }
        let emits = is_surface_belt(&e.name)
            || is_splitter(&e.name)
            || (is_ug_belt(&e.name) && e.io_type.as_deref() == Some("output"));
        if !emits {
            continue;
        }
        let (dx, dy) = dir_delta(e.direction);
        let lands = if is_splitter(&e.name) {
            let (s2x, s2y) = splitter_second_tile(e);
            (e.x + dx, e.y + dy) == tile || (s2x + dx, s2y + dy) == tile
        } else {
            (e.x + dx, e.y + dy) == tile
        };
        if lands {
            return Some(ExternalFeederSnapshot {
                entity_name: e.name.clone(),
                entity_x: e.x,
                entity_y: e.y,
                direction: dir_label(e.direction),
            });
        }
    }
    None
}


/// Resolve the (x, y, direction) at which to synthesize a perimeter IN
/// boundary for a participating spec whose topology-based IN was
/// missed. Returns the tile of the chain head (the first upstream tile
/// with no in-bbox belt feeder) and the flow direction of the belt at
/// that tile.
///
/// Strategy: start at the spec's entry tile. Repeatedly find the belt
/// that physically feeds the current tile (`physical_feeder_hit`, which
/// inspects all 4 neighbours — crucial at turn/join tiles where the
/// feeder is perpendicular to the local path direction). If the feeder
/// is inside the bbox, advance to it. Stop when the feeder is outside
/// the bbox (use feeder's direction) or no feeder exists (chain head;
/// use the current tile's belt direction). Cycle-safe via visited set.
///
/// Why not just use `spec.entry.direction`: at segment joins (e.g. a
/// `ret:*` merging into a `flow:*`), the flow spec's path-derived
/// direction is a DEPARTURE direction at the join tile, while the
/// actual upstream flow arrives from a perpendicular direction. SAT
/// needs the IN direction to match the feeder's flow direction.
fn resolve_chain_head(
    sc: &SpecCrossing,
    placed_entities: &[PlacedEntity],
    bbox: &Rect,
) -> (i32, i32, EntityDirection) {
    let mut cur = (sc.entry.x, sc.entry.y);
    let mut cur_dir = sc.entry.direction;
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    loop {
        if !visited.insert(cur) {
            return (cur.0, cur.1, cur_dir);
        }
        // Pick up the direction of the in-place belt at `cur`, if any —
        // `sc.entry.direction` is a path-derived approximation.
        if let Some(e) = placed_entities.iter().find(|e| {
            e.x == cur.0
                && e.y == cur.1
                && e.carries.as_deref() == Some(sc.item.as_str())
                && (is_surface_belt(&e.name) || is_ug_belt(&e.name))
        }) {
            cur_dir = e.direction;
        }
        let Some(hit) = physical_feeder_hit(cur, placed_entities, &sc.item) else {
            // No belt feeder anywhere — chain head.
            return (cur.0, cur.1, cur_dir);
        };
        if !bbox.contains(hit.entity_tile.0, hit.entity_tile.1) {
            // Feeder is outside the bbox — emit perimeter IN at `cur`
            // with the feeder's flow direction.
            return (cur.0, cur.1, hit.entity_direction);
        }
        // Feeder is another FREE in-bbox belt. Walk to it.
        cur = hit.entity_tile;
        cur_dir = hit.entity_direction;
    }
}

/// Find a Permanent entity (splitter / belt / UG-out) whose output lands
/// on `tile` AND carries `item`. Returns the *specific* feeder tile and
/// direction.
///
/// The item filter matters: physically, a belt carrying item B that
/// outputs onto a tile carrying item A *does* geometrically drop items
/// (and in Factorio would produce a mixed belt). But for boundary
/// derivation we only care about feeders that are part of the same
/// item's flow graph. Without this filter, `topology_boundaries` creates
/// phantom boundaries labelled with the in-bbox entity's item while the
/// physical feeder carries a different item — SAT then tries to route
/// a flow that has no real source, and the walker vetoes every solution
/// because the "source" belts are in a different item's graph. Any
/// genuine cross-item belt facing is a ghost-routing bug upstream; the
/// fix there belongs in the router, not here.
///
/// For splitters, returns the one of the two tiles that physically emits
/// onto `tile` (the tile from which `tile = feeder_tile + dir_delta(dir)`).
fn physical_feeder_hit(
    tile: (i32, i32),
    placed_entities: &[PlacedEntity],
    item: &str,
) -> Option<FeederHit> {
    for e in placed_entities {
        // UG-ins consume; they don't emit onto the surface.
        if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input") {
            continue;
        }
        if e.carries.as_deref() != Some(item) {
            continue;
        }
        let emits = is_surface_belt(&e.name)
            || is_splitter(&e.name)
            || (is_ug_belt(&e.name) && e.io_type.as_deref() == Some("output"));
        if !emits {
            continue;
        }
        let (dx, dy) = dir_delta(e.direction);
        if is_splitter(&e.name) {
            let (sx, sy) = splitter_second_tile(e);
            if (e.x + dx, e.y + dy) == tile {
                return Some(FeederHit {
                    entity_tile: (e.x, e.y),
                    entity_direction: e.direction,
                    entity_name: e.name.clone(),
                });
            }
            if (sx + dx, sy + dy) == tile {
                return Some(FeederHit {
                    entity_tile: (sx, sy),
                    entity_direction: e.direction,
                    entity_name: e.name.clone(),
                });
            }
        } else if (e.x + dx, e.y + dy) == tile {
            return Some(FeederHit {
                entity_tile: (e.x, e.y),
                entity_direction: e.direction,
                entity_name: e.name.clone(),
            });
        }
    }
    None
}

/// Single-pass topology walk that discovers all SAT boundaries by classifying
/// every tile in the bbox as FREE (surface belt not in `forbidden`, SAT can
/// re-stamp) or FIXED (anything in `forbidden`: UG belts, splitters, machines,
/// etc.). Boundaries exist only where item flow crosses between a FREE tile
/// and the outside world or a FIXED tile. FIXED-to-FIXED and FIXED-to-outside
/// connections are pre-routed and invisible to SAT.
///
/// Replaces `walk_entry_to_perimeter` + `splitter_topology_boundaries` +
/// `belt_topology_boundaries` + the dedup step with a unified walk that
/// produces no duplicates by construction.
///
/// **Reliance on ghost-routed layout**: unlike the previous design, this walk
/// does not consult the `SpecCrossing` list directly. It assumes ghost routing
/// has already stamped entities along every spec's path *or* that neighbouring
/// stamped entities implicitly constrain the uncovered tiles (e.g. a belt
/// flowing north into a FREE spec-exit tile at y−1 forces SAT to stay
/// compatible with that flow even if the exit tile itself has no entity).
/// An experimentally-added debug-assert confirmed both modes occur in the
/// active e2e corpus: sometimes the stamped entity is at the spec tile,
/// sometimes it's one step away and SAT bridges the gap via adjacency. If a
/// future change introduces a participating spec whose entire FREE-in-bbox
/// path is unstamped, SAT will silently solve that region without constraint
/// and produce a layout that ignores the spec. Consider re-adding the
/// debug-assert (previously at the call site, removed once it proved too
/// strict for the current corpus) if you suspect this class of bug.
fn topology_boundaries(
    placed_entities: &[PlacedEntity],
    bbox: &Rect,
    forbidden: &FxHashSet<(i32, i32)>,
) -> Vec<ZoneBoundary> {
    let mut boundaries: Vec<ZoneBoundary> = Vec::new();

    // Phase 1: Walk every entity whose tile is inside bbox and FREE.
    for e in placed_entities {
        // Determine all tiles this entity occupies (1 for most, 2 for splitters).
        let entity_tiles: Vec<(i32, i32)> = if is_splitter(&e.name) {
            let (sx, sy) = splitter_second_tile(e);
            vec![(e.x, e.y), (sx, sy)]
        } else {
            vec![(e.x, e.y)]
        };

        for &(tx, ty) in &entity_tiles {
            if !bbox.contains(tx, ty) {
                continue;
            }
            // FREE = not in forbidden. FIXED tiles are skipped.
            if forbidden.contains(&(tx, ty)) {
                continue;
            }
            let Some(item) = e.carries.as_deref() else {
                continue;
            };

            // -- Output check: where does this belt/UG output? --
            let (dx, dy) = dir_delta(e.direction);
            let target = (tx + dx, ty + dy);

            // For OUT boundaries we want the EXTERNAL receiver's tier —
            // the entity sitting at `target` on the far side of the
            // boundary — not the in-zone entity `e`'s tier. Fall back to
            // `e`'s tier if nothing's there yet (rare: should only
            // happen if ghost-routing hasn't finalised the downstream
            // trunk, which shouldn't occur by the time SAT runs).
            let external_tier = entity_name_at(placed_entities, target)
                .as_deref()
                .and_then(tier_name_for)
                .or_else(|| tier_name_for(&e.name));
            if !bbox.contains(target.0, target.1) {
                // Target outside bbox: perimeter OUT.
                boundaries.push(ZoneBoundary {
                    x: tx,
                    y: ty,
                    direction: e.direction,
                    item: item.to_string(),
                    is_input: false,
                    interior: false,
                    belt_tier: external_tier,
                    channel_id: 0,
                });
            } else if forbidden.contains(&target) {
                // Target is FIXED: interior OUT at the FIXED tile.
                boundaries.push(ZoneBoundary {
                    x: target.0,
                    y: target.1,
                    direction: e.direction,
                    item: item.to_string(),
                    is_input: false,
                    interior: true,
                    belt_tier: external_tier,
                    channel_id: 0,
                });
            }
            // else: target is FREE, both SAT-routable → no boundary.

            // -- Input check: does anything feed this tile from outside/FIXED? --
            if let Some(hit) = physical_feeder_hit((tx, ty), placed_entities, item) {
                let feeder_belt_tier = tier_name_for(&hit.entity_name);
                if !bbox.contains(hit.entity_tile.0, hit.entity_tile.1) {
                    // Feeder outside bbox: perimeter IN.
                    boundaries.push(ZoneBoundary {
                        x: tx,
                        y: ty,
                        direction: hit.entity_direction,
                        item: item.to_string(),
                        is_input: true,
                        interior: false,
                        belt_tier: feeder_belt_tier,
                        channel_id: 0,
                    });
                } else if forbidden.contains(&hit.entity_tile) {
                    // Feeder is FIXED: interior IN at the FIXED tile.
                    boundaries.push(ZoneBoundary {
                        x: hit.entity_tile.0,
                        y: hit.entity_tile.1,
                        direction: hit.entity_direction,
                        item: item.to_string(),
                        is_input: true,
                        interior: true,
                        belt_tier: feeder_belt_tier,
                        channel_id: 0,
                    });
                }
                // else: feeder is FREE → no boundary.
            }
        }
    }

    // Phase 2: Splitter topology. Splitters are FIXED (in forbidden) but
    // are active flow devices. For each splitter tile inside bbox, emit
    // interior boundaries for lanes connected to FREE tiles.
    for e in placed_entities {
        if !is_splitter(&e.name) {
            continue;
        }
        let Some(item) = e.carries.as_deref() else {
            continue;
        };
        let tiles = [(e.x, e.y), splitter_second_tile(e)];
        let (dx, dy) = dir_delta(e.direction);
        for &(sx, sy) in &tiles {
            if !bbox.contains(sx, sy) {
                continue;
            }
            let input_nb = (sx - dx, sy - dy);
            let output_nb = (sx + dx, sy + dy);

            // Input side: splitter receives from a FREE tile.
            let input_wired = placed_entities.iter().any(|n| {
                if n.carries.as_deref() != Some(item) {
                    return false;
                }
                if is_ug_belt(&n.name) && n.io_type.as_deref() != Some("output") {
                    return false;
                }
                if !(is_surface_belt(&n.name) || is_splitter(&n.name) || is_ug_belt(&n.name)) {
                    return false;
                }
                let (ndx, ndy) = dir_delta(n.direction);
                if is_splitter(&n.name) {
                    let (nsx, nsy) = splitter_second_tile(n);
                    (n.x + ndx, n.y + ndy) == (sx, sy)
                        || (nsx + ndx, nsy + ndy) == (sx, sy)
                } else {
                    (n.x == input_nb.0 && n.y == input_nb.1)
                        && (n.x + ndx, n.y + ndy) == (sx, sy)
                }
            });
            if input_wired {
                // Only emit if the neighbor is FREE (not FIXED-to-FIXED).
                if !forbidden.contains(&input_nb) {
                    boundaries.push(ZoneBoundary {
                        x: sx,
                        y: sy,
                        direction: e.direction,
                        item: item.to_string(),
                        is_input: false, // splitter input-side = zone OUT
                        interior: true,
                        belt_tier: tier_name_for(&e.name),
                        channel_id: 0,
                    });
                }
            }

            // Output side: splitter emits to a FREE tile.
            let output_wired = placed_entities.iter().any(|n| {
                if n.carries.as_deref() != Some(item) {
                    return false;
                }
                if !(is_surface_belt(&n.name) || is_splitter(&n.name) || is_ug_belt(&n.name)) {
                    return false;
                }
                if is_splitter(&n.name) {
                    let (nsx, nsy) = splitter_second_tile(n);
                    (n.x, n.y) == output_nb || (nsx, nsy) == output_nb
                } else {
                    (n.x, n.y) == output_nb
                }
            });
            if output_wired && !forbidden.contains(&output_nb) {
                boundaries.push(ZoneBoundary {
                    x: sx,
                    y: sy,
                    direction: e.direction,
                    item: item.to_string(),
                    is_input: true, // splitter output-side = zone IN
                    interior: true,
                    belt_tier: tier_name_for(&e.name),
                    channel_id: 0,
                });
            }
        }
    }

    boundaries
}

/// Assign `channel_id` to every boundary in `boundaries`.
///
/// Boundaries that share `(item, belt_tier)` get the same channel id.
/// Different tiers of the same item get different ids — that's what
/// enforces tier-aware pairing at the SAT level (boundaries in
/// different channels can't route into each other's flows).
///
/// Returns a `channel_id → (item, belt_tier)` table, indexed by
/// channel id. The encoder needs the tier per channel for UG-reach
/// constraints; the entity stamper needs the item + tier to write
/// `carries` and `name` on each placed entity.
///
/// Deterministic: buckets are sorted by `(item, tier)` so the same
/// boundary set always produces the same assignment.
pub(crate) fn assign_channels(
    boundaries: &mut [ZoneBoundary],
) -> Vec<(String, Option<String>)> {
    use std::collections::BTreeMap;

    // BTreeMap for deterministic iteration order.
    let mut bucket_ids: BTreeMap<(String, Option<String>), u32> = BTreeMap::new();
    for b in boundaries.iter() {
        let key = (b.item.clone(), b.belt_tier.clone());
        let next = bucket_ids.len() as u32;
        bucket_ids.entry(key).or_insert(next);
    }
    for b in boundaries.iter_mut() {
        let key = (b.item.clone(), b.belt_tier.clone());
        b.channel_id = bucket_ids[&key];
    }
    // Invert the map into a Vec indexed by channel_id.
    let mut table = vec![(String::new(), None); bucket_ids.len()];
    for (key, id) in bucket_ids {
        table[id as usize] = key;
    }
    table
}


impl JunctionStrategy for SatStrategy {
    fn name(&self) -> &'static str {
        self.name
    }

    fn try_solve(&self, ctx: &JunctionStrategyContext) -> Option<JunctionSolution> {
        // SAT cannot solve a 1-tile zone: entry and exit for each spec
        // would collapse to the same tile, which is not a valid
        // `CrossingZone`. Wait for the growth loop to expand the
        // frontier at least once.
        if ctx.region.tile_count() <= 1 {
            return None;
        }
        if ctx.junction.specs.is_empty() {
            return None;
        }
        // SAT's encoder treats every spec as a belt — it has no notion
        // of a fixed-surface pipe that must not be routed over. If the
        // junction carries any pipe-kind spec, defer to higher-level
        // strategies (`bridge_belt_over_pipe` in the perpendicular
        // template, or an Unresolved region) rather than emitting a
        // SAT model that would stamp belts on top of the pipe.
        // Pipe-aware SAT is Phase 3 in `docs/rfp-pipe-belt-junctions.md`.
        if ctx.junction.specs.iter().any(|s| s.kind == SpecKind::Pipe) {
            return None;
        }

        // Dominant belt tier across participating specs. If a junction
        // carries both yellow and red specs we use red (faster) so the
        // solver has the widest UG reach to work with.
        let belt_tier: BeltTier = ctx
            .junction
            .specs
            .iter()
            .map(|s| s.belt_tier)
            .max_by_key(|t| t.rank())
            .unwrap_or(BeltTier::Yellow);
        let belt_name = belt_tier.belt_name();
        let max_reach = ug_max_reach(belt_name);

        // Topology-based boundary discovery: walk every physical tile in the
        // bbox, classify as FREE (surface belt, SAT can re-stamp) or FIXED
        // (anything in forbidden), and emit boundaries wherever flow crosses
        // between FREE and outside/FIXED. No dedup needed — each boundary
        // position is unique by construction.
        let mut boundaries = topology_boundaries(
            ctx.placed_entities,
            &ctx.junction.bbox,
            &ctx.junction.forbidden,
        );
        let mut origins: Vec<String> = boundaries
            .iter()
            .map(|_| "topology".to_string())
            .collect();

        // Chain-head augmentation: every participating spec contracts to
        // route one item into the zone at `entry`. The topology walk only
        // emits an IN when a *belt* feeds the tile — it misses the case
        // where the chain's first in-bbox belt is fed by an inserter from
        // a machine output (no belt upstream). Without this fix, SAT
        // receives no input constraint for the spec, solves the zone
        // however it likes, and silently drops the item's flow entirely.
        // The walker doesn't catch this either because it only checks
        // continuity of already-stamped paths, not missing ones.
        //
        // For each participating spec, if no IN boundary was emitted for
        // its item anywhere in the zone, synthesize one at the true flow
        // source. The source is found by following the spec's path
        // upstream of its entry until a tile with an actual belt feeder
        // is found, or until a tile with no feeder is reached (the chain
        // head — typically inserter-fed from a machine output).
        for sc in ctx.junction.specs.iter() {
            let has_in_for_item = boundaries
                .iter()
                .any(|b| b.is_input && b.item == sc.item);
            if has_in_for_item {
                continue;
            }
            // Find a concrete entity at the entry tile (or walk upstream
            // to find one) and use its direction as the IN direction.
            // Using `entry.direction` directly is wrong at spec joins —
            // the copper-cable trunk at (3,7) has entry dir South (path
            // departure) but the real flow arrives West from (4,7) via a
            // ret segment. Without a direction that matches the feeder,
            // SAT routes incorrectly or UNSATs.
            let (in_x, in_y, in_dir) =
                resolve_chain_head(sc, ctx.placed_entities, &ctx.junction.bbox);
            boundaries.push(ZoneBoundary {
                x: in_x,
                y: in_y,
                direction: in_dir,
                item: sc.item.clone(),
                is_input: true,
                interior: false,
                belt_tier: Some(sc.belt_tier.belt_name().to_string()),
                channel_id: 0,
            });
            origins.push("spec-chain-head".to_string());
        }

        // Assign channel ids now that the boundary set is final. Buckets
        // by (item, belt_tier); ids are deterministic. `channel_table[i]`
        // gives `(item, belt_tier)` for channel id `i` — used below to
        // compute per-channel UG reaches under `ReachMode::Native`.
        let channel_table = assign_channels(&mut boundaries);

        // Per-channel UG reaches. Native mode: each channel's reach
        // derives from its declared belt tier (fall back to the zone's
        // dominant tier when a channel's tier is unknown). Relaxed
        // mode: every channel shares the dominant tier's reach —
        // matches pre-refactor behaviour. Native is tried first in the
        // ladder (see `ghost_router.rs`) so mixed-tier zones get
        // tier-correct UG lengths when feasible, then falls through to
        // relaxed for zones that genuinely can't route under tight
        // reach.
        let channel_reaches: Vec<u32> = match self.constraints.reach_mode {
            ReachMode::Native => channel_table
                .iter()
                .map(|(_, tier)| {
                    tier.as_deref()
                        .and_then(BeltTier::from_name)
                        .map(|t| ug_max_reach(t.belt_name()))
                        .unwrap_or(max_reach)
                })
                .collect(),
            ReachMode::Relaxed => vec![max_reach; channel_table.len().max(1)],
        };

        let forced_empty: Vec<(i32, i32)> =
            ctx.junction.forbidden.iter().copied().collect();

        // Build snapshots: origin was tracked in parallel above.
        let boundary_snapshots: Vec<BoundarySnapshot> = boundaries
            .iter()
            .zip(&origins)
            .map(|(b, origin)| {
                let feeder = if b.is_input {
                    find_external_feeder((b.x, b.y), ctx.placed_entities)
                } else {
                    None
                };
                BoundarySnapshot {
                    x: b.x,
                    y: b.y,
                    direction: dir_label(b.direction),
                    item: b.item.clone(),
                    is_input: b.is_input,
                    interior: b.interior,
                    spec_key: String::new(),
                    origin: origin.clone(),
                    external_feeder: feeder,
                    belt_tier: b.belt_tier.clone(),
                    channel_id: b.channel_id,
                }
            })
            .collect();

        let zone = CrossingZone {
            x: ctx.junction.bbox.x,
            y: ctx.junction.bbox.y,
            width: ctx.junction.bbox.w,
            height: ctx.junction.bbox.h,
            boundaries: boundaries.clone(),
            forced_empty: forced_empty.clone(),
        };

        let (seed_x, seed_y) = ctx.region.initial_tile;
        let iter = ctx.growth_iter;

        let (entities_opt, stats) = crate::sat::solve_crossing_zone_per_channel(
            &zone,
            &channel_reaches,
            belt_name,
            self.constraints.max_ug_ins,
        );
        let satisfied = entities_opt.is_some();
        let entities_raw = entities_opt.as_ref().map(|e| e.len()).unwrap_or(0);
        let proposed_entities: Vec<SatProposedEntity> = entities_opt
            .as_ref()
            .map(|es| {
                es.iter()
                    .map(|e| SatProposedEntity {
                        x: e.x,
                        y: e.y,
                        name: e.name.clone(),
                        direction: dir_label(e.direction),
                        carries: e.carries.clone(),
                        io_type: e.io_type.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let initial_cost = entities_opt
            .as_ref()
            .map(|es| crate::bus::junction_cost::solution_cost(es));
        trace::emit(TraceEvent::SatInvocation {
            seed_x,
            seed_y,
            iter,
            variant: ctx.growth_variant.to_string(),
            zone_x: zone.x,
            zone_y: zone.y,
            zone_w: zone.width,
            zone_h: zone.height,
            boundaries: boundary_snapshots,
            forced_empty,
            belt_tier: belt_name.to_string(),
            max_reach,
            satisfied,
            variables: stats.variables,
            clauses: stats.clauses,
            solve_time_us: stats.solve_time_us,
            entities_raw,
            initial_cost,
            proposed_entities,
        });
        let mut best = entities_opt?;
        let mut best_cost =
            initial_cost.expect("entities_opt is Some here, so initial_cost is Some");

        // Cost descent: re-solve with a tighter cost cap until either
        // UNSAT (current best is optimal at this cap), wall-clock
        // budget runs out, or iter limit. Descent operates on RAW
        // SAT output so the cap we compute lines up with what the
        // encoder sees; pruning happens once at the end.
        let deadline = web_time::Instant::now()
            + std::time::Duration::from_millis(self.constraints.cost_descent_budget_ms as u64);

        for descent_iter in 0..self.constraints.cost_descent_max_iters {
            if web_time::Instant::now() >= deadline {
                break;
            }
            let Some(cap) = best_cost.checked_sub(1) else {
                break; // cost already zero — nothing to tighten
            };
            let (next_opt, next_stats) =
                crate::sat::solve_crossing_zone_per_channel_with_cost_cap(
                    &zone,
                    &channel_reaches,
                    belt_name,
                    self.constraints.max_ug_ins,
                    Some(cap),
                );
            let next_sat = next_opt.is_some();
            let next_cost = next_opt
                .as_ref()
                .map(|es| crate::bus::junction_cost::solution_cost(es));
            let cost_after = match next_cost {
                Some(c) if c < best_cost => Some(c),
                _ => None,
            };
            trace::emit(TraceEvent::SatCostDescent {
                seed_x,
                seed_y,
                iter,
                variant: ctx.growth_variant.to_string(),
                descent_iter,
                cap,
                satisfied: next_sat,
                solve_time_us: next_stats.solve_time_us,
                cost_after,
            });
            match (next_opt, next_cost) {
                (Some(ents), Some(c)) if c < best_cost => {
                    best = ents;
                    best_cost = c;
                }
                (Some(_), _) => {
                    // Encoder said SAT but cost didn't drop — safety
                    // bail. Shouldn't happen if weights are in sync
                    // with `junction_cost::solution_cost`.
                    break;
                }
                (None, _) => break,
            }
        }

        let pruned = prune_dangling_sat_entities(
            best,
            &boundaries,
            max_reach,
            zone.x,
            zone.y,
        );

        Some(JunctionSolution {
            entities: pruned,
            footprint: Rect {
                x: zone.x,
                y: zone.y,
                w: zone.width,
                h: zone.height,
            },
            strategy_name: self.name(),
            participating: ctx.region.participating.clone(),
            sat_zone: Some(crate::bus::junction_solver::SatZoneSnapshot {
                boundaries: boundaries.clone(),
                forced_empty: zone.forced_empty.clone(),
                belt_tier: belt_name.to_string(),
                max_ug_reach: max_reach,
            }),
        })
    }
}

// ---------------------------------------------------------------------------
// Dangling belt pruning
// ---------------------------------------------------------------------------

fn opposite(dir: EntityDirection) -> EntityDirection {
    match dir {
        EntityDirection::North => EntityDirection::South,
        EntityDirection::East  => EntityDirection::West,
        EntityDirection::South => EntityDirection::North,
        EntityDirection::West  => EntityDirection::East,
    }
}

/// Remove SAT-placed belt entities that are not on any path from an input
/// boundary to an output boundary.  Orphaned tiles arise from near-miss SAT
/// assignments where a variable is set true but the resulting entity is
/// unreachable in the final flow graph.
///
/// Algorithm: downstream BFS from all input boundaries ∩ upstream BFS from
/// all output boundaries.  Keep only entities in both reachable sets.
///
/// For interior boundaries the boundary tile itself is `forced_empty`
/// (no SAT entity), so the BFS seeds from the in-zone *neighbor* — the
/// tile the encoder's interior arm actually constrains. For an interior
/// input the neighbor is `boundary + dir_delta(direction)`; for an
/// interior output it's `boundary + dir_delta(opposite(direction))`.
pub(crate) fn prune_dangling_sat_entities(
    entities: Vec<PlacedEntity>,
    boundaries: &[ZoneBoundary],
    max_reach: u32,
    zone_x: i32,
    zone_y: i32,
) -> Vec<PlacedEntity> {
    use std::collections::{HashMap, VecDeque};

    let by_tile: HashMap<(i32, i32), usize> = entities
        .iter()
        .enumerate()
        .map(|(i, e)| ((e.x, e.y), i))
        .collect();

    // Map a boundary to the actual in-zone tile that holds the SAT
    // entity feeding (for inputs) or sinking (for outputs) the spec's
    // flow. Perimeter boundaries: that's the boundary tile itself.
    // Interior boundaries: the in-zone neighbor along the flow axis.
    let bfs_start = |b: &ZoneBoundary| -> (i32, i32) {
        if b.interior {
            let (dx, dy) = if b.is_input {
                dir_delta(b.direction)
            } else {
                dir_delta(opposite(b.direction))
            };
            (b.x + dx, b.y + dy)
        } else {
            (b.x, b.y)
        }
    };

    // ---- downstream BFS (input → output direction) ----

    let mut reachable_from_input: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();

    for b in boundaries.iter().filter(|b| b.is_input) {
        let t = bfs_start(b);
        if reachable_from_input.insert(t) {
            queue.push_back(t);
        }
    }

    while let Some(t) = queue.pop_front() {
        let Some(&idx) = by_tile.get(&t) else { continue };
        let e = &entities[idx];
        let next_tiles = next_downstream(&entities, &by_tile, e, max_reach);
        for n in next_tiles {
            if reachable_from_input.insert(n) {
                queue.push_back(n);
            }
        }
    }

    // ---- upstream BFS (output → input direction) ----

    let mut reachable_to_output: FxHashSet<(i32, i32)> = FxHashSet::default();

    for b in boundaries.iter().filter(|b| !b.is_input) {
        let t = bfs_start(b);
        if reachable_to_output.insert(t) {
            queue.push_back(t);
        }
    }

    while let Some(t) = queue.pop_front() {
        let Some(&idx) = by_tile.get(&t) else { continue };
        let e = &entities[idx];
        let prev_tiles = next_upstream(&entities, &by_tile, e, max_reach);
        for n in prev_tiles {
            if reachable_to_output.insert(n) {
                queue.push_back(n);
            }
        }
    }

    // ---- keep intersection ----

    let total = entities.len();
    let pruned: Vec<PlacedEntity> = entities
        .into_iter()
        .filter(|e| {
            let t = (e.x, e.y);
            reachable_from_input.contains(&t) && reachable_to_output.contains(&t)
        })
        .collect();
    let kept = pruned.len();

    if kept < total {
        trace::emit(trace::TraceEvent::SatPruned { zone_x, zone_y, total, kept });
    }

    pruned
}

/// Tiles reachable downstream from entity `e` in one step (or one UG pair).
fn next_downstream(
    entities: &[PlacedEntity],
    by_tile: &std::collections::HashMap<(i32, i32), usize>,
    e: &PlacedEntity,
    max_reach: u32,
) -> Vec<(i32, i32)> {
    match e.io_type.as_deref() {
        Some("input") => {
            // UG-in: scan forward up to max_reach tiles to find the paired UG-out.
            let (dx, dy) = dir_delta(e.direction);
            let mut results = Vec::new();
            // Scan up to `max_reach + 1` tiles away — the SAT encoder
            // allows a UG pair to span `max_reach + 1` positions apart
            // (up to `max_reach` consecutive hidden-middle tiles; see
            // `sat.rs::encode_underground`'s max-reach clause). Scanning
            // only `max_reach` misses the paired endpoint when SAT picks
            // a maximum-span UG, which silently prunes the whole spec's
            // placement and manifests as "SAT solved but one item is
            // missing from the final model."
            for dist in 1..=(max_reach as i32 + 1) {
                let nx = e.x + dx * dist;
                let ny = e.y + dy * dist;
                if let Some(&ni) = by_tile.get(&(nx, ny)) {
                    let n = &entities[ni];
                    if n.io_type.as_deref() == Some("output") && n.direction == e.direction {
                        results.push((nx, ny));
                        break;
                    }
                }
            }
            results
        }
        _ => {
            // Belt or UG-out: next tile in output direction.
            let (dx, dy) = dir_delta(e.direction);
            vec![(e.x + dx, e.y + dy)]
        }
    }
}

/// Tiles reachable upstream from entity `e` in one step (or one UG pair).
fn next_upstream(
    entities: &[PlacedEntity],
    by_tile: &std::collections::HashMap<(i32, i32), usize>,
    e: &PlacedEntity,
    max_reach: u32,
) -> Vec<(i32, i32)> {
    match e.io_type.as_deref() {
        Some("output") => {
            // UG-out: scan backward to find the paired UG-in.
            let (dx, dy) = dir_delta(opposite(e.direction));
            let mut results = Vec::new();
            // Scan up to `max_reach + 1` tiles away — the SAT encoder
            // allows a UG pair to span `max_reach + 1` positions apart
            // (up to `max_reach` consecutive hidden-middle tiles; see
            // `sat.rs::encode_underground`'s max-reach clause). Scanning
            // only `max_reach` misses the paired endpoint when SAT picks
            // a maximum-span UG, which silently prunes the whole spec's
            // placement and manifests as "SAT solved but one item is
            // missing from the final model."
            for dist in 1..=(max_reach as i32 + 1) {
                let nx = e.x + dx * dist;
                let ny = e.y + dy * dist;
                if let Some(&ni) = by_tile.get(&(nx, ny)) {
                    let n = &entities[ni];
                    if n.io_type.as_deref() == Some("input") && n.direction == e.direction {
                        results.push((nx, ny));
                        break;
                    }
                }
            }
            results
        }
        _ => {
            // Belt or UG-in: the tile that outputs toward us.
            // Check all 4 neighbors; keep those whose entity outputs into `e`.
            let mut results = Vec::new();
            for &dir in &[
                EntityDirection::North,
                EntityDirection::East,
                EntityDirection::South,
                EntityDirection::West,
            ] {
                let (dx, dy) = dir_delta(dir);
                let nx = e.x + dx;
                let ny = e.y + dy;
                if let Some(&ni) = by_tile.get(&(nx, ny)) {
                    let n = &entities[ni];
                    // n must output in direction opposite(dir) to feed into e
                    if n.io_type.as_deref() != Some("input") && n.direction == opposite(dir) {
                        results.push((nx, ny));
                    }
                }
            }
            results
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityDirection;

    fn make_belt(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".into(),
            x,
            y,
            direction: dir,
            carries: Some(item.into()),
            ..Default::default()
        }
    }

    #[test]
    fn test_prune_removes_orphan_belt() {
        // Layout: input at (0,0) East, output at (2,0) East.
        // Valid path: (0,0)→(1,0)→(2,0) all facing East.
        // Orphan: (1,1) facing East — not connected to anything.
        let entities = vec![
            make_belt(0, 0, EntityDirection::East, "iron-plate"),
            make_belt(1, 0, EntityDirection::East, "iron-plate"),
            make_belt(2, 0, EntityDirection::East, "iron-plate"),
            make_belt(1, 1, EntityDirection::East, "iron-plate"), // orphan
        ];
        let boundaries = vec![
            ZoneBoundary { x: 0, y: 0, direction: EntityDirection::East, item: "iron-plate".into(), is_input: true, interior: false, belt_tier: None, channel_id: 0 },
            ZoneBoundary { x: 2, y: 0, direction: EntityDirection::East, item: "iron-plate".into(), is_input: false, interior: false, belt_tier: None, channel_id: 0 },
        ];
        let result = prune_dangling_sat_entities(entities, &boundaries, 4, 0, 0);
        assert_eq!(result.len(), 3, "orphan at (1,1) should be pruned");
        assert!(result.iter().all(|e| e.y == 0), "only y=0 row survives");
    }

    #[test]
    fn test_prune_keeps_full_path() {
        // Single straight path, nothing to prune.
        let entities = vec![
            make_belt(0, 0, EntityDirection::East, "copper-plate"),
            make_belt(1, 0, EntityDirection::East, "copper-plate"),
        ];
        let boundaries = vec![
            ZoneBoundary { x: 0, y: 0, direction: EntityDirection::East, item: "copper-plate".into(), is_input: true, interior: false, belt_tier: None, channel_id: 0 },
            ZoneBoundary { x: 1, y: 0, direction: EntityDirection::East, item: "copper-plate".into(), is_input: false, interior: false, belt_tier: None, channel_id: 0 },
        ];
        let result = prune_dangling_sat_entities(entities, &boundaries, 4, 0, 0);
        assert_eq!(result.len(), 2, "full path should be untouched");
    }

    // -- Prune behaviour with interior boundaries ---------------------------

    fn ug_in(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "fast-underground-belt".into(),
            x,
            y,
            direction: dir,
            carries: Some(item.into()),
            io_type: Some("input".into()),
            ..Default::default()
        }
    }

    fn ug_out(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "fast-underground-belt".into(),
            x,
            y,
            direction: dir,
            carries: Some(item.into()),
            io_type: Some("output".into()),
            ..Default::default()
        }
    }

    /// Reproduces the iter-2 tier2 SAT solution and pipes it through
    /// `prune_dangling_sat_entities` exactly as the strategy does.
    /// The boundaries are interior on both inputs (iron-plate at (2,9),
    /// copper-cable at (3,9)) — their tiles are forced_empty, so a
    /// naive BFS that starts at `(b.x, b.y)` never advances and ALL
    /// entities get pruned even though they form valid input→output
    /// flows via UG corridors. The fix is to seed the BFS from the
    /// in-zone neighbour for interior boundaries.
    #[test]
    fn test_prune_keeps_interior_boundary_paths() {
        let entities = vec![
            ug_in(2, 10, EntityDirection::East, "iron-plate"),
            ug_out(5, 10, EntityDirection::East, "iron-plate"),
            ug_in(3, 10, EntityDirection::South, "copper-cable"),
            ug_out(3, 12, EntityDirection::South, "copper-cable"),
        ];
        let boundaries = vec![
            ZoneBoundary {
                x: 3, y: 9,
                direction: EntityDirection::South,
                item: "copper-cable".into(),
                is_input: true,
                interior: true,
                belt_tier: None,
                channel_id: 0,
            },
            ZoneBoundary {
                x: 3, y: 12,
                direction: EntityDirection::South,
                item: "copper-cable".into(),
                is_input: false,
                interior: false,
                belt_tier: None,
                channel_id: 0,
            },
            ZoneBoundary {
                x: 2, y: 9,
                direction: EntityDirection::South,
                item: "iron-plate".into(),
                is_input: true,
                interior: true,
                belt_tier: None,
                channel_id: 0,
            },
            ZoneBoundary {
                x: 5, y: 10,
                direction: EntityDirection::East,
                item: "iron-plate".into(),
                is_input: false,
                interior: false,
                belt_tier: None,
                channel_id: 0,
            },
        ];

        let pruned = prune_dangling_sat_entities(entities.clone(), &boundaries, 6, 1, 8);
        // All 4 UG endpoints must survive — they're the SAT-resolved
        // crossing for both specs and form valid input→output paths.
        assert_eq!(
            pruned.len(),
            4,
            "interior-boundary specs should retain their UG endpoints; got {pruned:#?}"
        );
    }

    // -- Splitter topology helpers ------------------------------------------

    fn make_surface_belt(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "fast-transport-belt".into(),
            x,
            y,
            direction: dir,
            carries: Some(item.into()),
            ..Default::default()
        }
    }

    // -- Topology boundary tests ------------------------------------------------

    #[test]
    fn test_topology_boundaries_ug_crossing_zone() {
        // Bbox: x:2-5, y:17-19 (4x3)
        //
        // y\x  2         3           4         5
        // 17   .         UG↓in(cc)   belt↓(cc) .
        // 18   belt→(ip) UG→in(ip)   belt↓(cc) UG→out(ip)
        // 19   .         UG↓out(cc)  belt→(cc) belt→(cc)
        //
        // FREE: (2,18), (4,17), (4,18), (4,19), (5,19) — surface belts
        // FIXED: (3,17), (3,18), (3,19), (5,18) — UG belts
        let bbox = Rect { x: 2, y: 17, w: 4, h: 3 };

        let placed = vec![
            // External feeders (outside bbox)
            make_surface_belt(1, 18, EntityDirection::East, "iron-plate"),  // feeds (2,18)
            make_surface_belt(4, 16, EntityDirection::South, "copper-cable"), // feeds (4,17)
            // Iron-plate path
            make_surface_belt(2, 18, EntityDirection::East, "iron-plate"),
            ug_in(3, 18, EntityDirection::East, "iron-plate"),
            ug_out(5, 18, EntityDirection::East, "iron-plate"),
            // Copper-cable UG tunnel (column 3)
            ug_in(3, 17, EntityDirection::South, "copper-cable"),
            ug_out(3, 19, EntityDirection::South, "copper-cable"),
            // Copper-cable surface chain (column 4 → 5)
            make_surface_belt(4, 17, EntityDirection::South, "copper-cable"),
            make_surface_belt(4, 18, EntityDirection::South, "copper-cable"),
            make_surface_belt(4, 19, EntityDirection::East, "copper-cable"),
            make_surface_belt(5, 19, EntityDirection::East, "copper-cable"),
        ];

        let forbidden: FxHashSet<(i32, i32)> = [
            (3, 17), (3, 18), (3, 19), (5, 18),
        ].into_iter().collect();

        let bounds = topology_boundaries(&placed, &bbox, &forbidden);

        let ins: Vec<_> = bounds.iter().filter(|b| b.is_input).collect();
        let outs: Vec<_> = bounds.iter().filter(|b| !b.is_input).collect();

        // Expected 4 boundaries:
        //   IN:  (2,18) East iron-plate perimeter, (4,17) South copper-cable perimeter
        //   OUT: (3,18) East iron-plate interior, (5,19) East copper-cable perimeter
        assert_eq!(ins.len(), 2, "IN boundaries: {ins:#?}");
        assert_eq!(outs.len(), 2, "OUT boundaries: {outs:#?}");

        // IN (2,18) East iron-plate, perimeter
        assert!(
            ins.iter().any(|b| (b.x, b.y) == (2, 18)
                && b.direction == EntityDirection::East
                && b.item == "iron-plate"
                && !b.interior),
            "missing perimeter IN (2,18) East iron-plate"
        );

        // IN (4,17) South copper-cable, perimeter
        assert!(
            ins.iter().any(|b| (b.x, b.y) == (4, 17)
                && b.direction == EntityDirection::South
                && b.item == "copper-cable"
                && !b.interior),
            "missing perimeter IN (4,17) South copper-cable"
        );

        // OUT (3,18) East iron-plate, interior (belt outputs to FIXED UG-in)
        assert!(
            outs.iter().any(|b| (b.x, b.y) == (3, 18)
                && b.direction == EntityDirection::East
                && b.item == "iron-plate"
                && b.interior),
            "missing interior OUT (3,18) East iron-plate"
        );

        // OUT (5,19) East copper-cable, perimeter
        assert!(
            outs.iter().any(|b| (b.x, b.y) == (5, 19)
                && b.direction == EntityDirection::East
                && b.item == "copper-cable"
                && !b.interior),
            "missing perimeter OUT (5,19) East copper-cable"
        );
    }

    /// Regression: `physical_feeder_hit` must ignore adjacent belts that
    /// carry a different item. Before the filter, an east-bound iron-ore
    /// belt at (5,10) inside the bbox with a south-facing *copper-plate*
    /// belt at (5,9) outside the bbox produced a phantom
    /// `iron-ore IN at (5,10) South` boundary (item copied from the
    /// in-bbox tile). The SAT walker would then veto every solution
    /// because the tap's real approach from the west had no place in
    /// the iron-ore flow graph. Observed in the ac-5/from-ore layout at
    /// seed (10,136) — see the trace with break_segment=tap:iron-ore:4:136
    /// in the fixture notes of ac_seed_10_136 if you need to reproduce.
    #[test]
    fn test_topology_boundaries_filters_cross_item_feeder() {
        // Bbox: x:4-6, y:10-10 (3x1).
        // Inside: belt→iron-ore at (4..6, 10).
        // External east feeder for tap: belt→iron-ore at (3,10).
        // External south "feeder" at (5,9) carrying copper-plate — this
        // physically drops onto (5,10) but must NOT create an iron-ore
        // IN boundary.
        let bbox = Rect { x: 4, y: 10, w: 3, h: 1 };
        let placed = vec![
            make_surface_belt(3, 10, EntityDirection::East, "iron-ore"), // west approach
            make_surface_belt(5, 9, EntityDirection::South, "copper-plate"), // cross-item
            make_surface_belt(4, 10, EntityDirection::East, "iron-ore"),
            make_surface_belt(5, 10, EntityDirection::East, "iron-ore"),
            make_surface_belt(6, 10, EntityDirection::East, "iron-ore"),
        ];
        let forbidden: FxHashSet<(i32, i32)> = FxHashSet::default();

        let bounds = topology_boundaries(&placed, &bbox, &forbidden);

        let ins: Vec<_> = bounds.iter().filter(|b| b.is_input).collect();
        let outs: Vec<_> = bounds.iter().filter(|b| !b.is_input).collect();

        // Exactly one IN (west feeder, iron-ore, East) and one OUT
        // (east exit, iron-ore, East). No phantom iron-ore IN from the
        // copper-plate belt at (5,9).
        assert_eq!(ins.len(), 1, "IN boundaries (expected 1 iron-ore): {ins:#?}");
        assert_eq!(outs.len(), 1, "OUT boundaries (expected 1): {outs:#?}");
        assert!(
            ins.iter().any(|b| (b.x, b.y) == (4, 10)
                && b.direction == EntityDirection::East
                && b.item == "iron-ore"),
            "missing perimeter IN (4,10) East iron-ore"
        );
        assert!(
            !ins.iter().any(|b| (b.x, b.y) == (5, 10)
                && b.direction == EntityDirection::South),
            "phantom south-IN boundary at (5,10) should be filtered by item mismatch"
        );
    }

}
