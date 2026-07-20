//! Junction solver: region-growth outer loop + strategy framework.
//!
//! The caller hands us a crossing tile and the set of specs that cross
//! at that tile (usually two, from `classify_crossing`). We build a
//! `GrowingRegion` around the crossing, then iterate: try each
//! registered `JunctionStrategy` on the current region, and if none
//! succeed, walk each participating spec's path one step outward and
//! try again. The loop terminates on success, on a tile-count cap, or
//! when every frontier is exhausted.
//!
//! `GrowingRegion` tracks two things per spec:
//! - a **frontier** `(start_idx, end_idx)` into the spec's routed path,
//!   representing the range of path tiles currently in the region,
//! - and a cached tile set so "is this tile in the region?" is O(1).
//!
//! Each iteration of `grow()` advances both ends of each frontier by
//! one step. The bbox is the tight enclosing rectangle of the accumulated
//! tile set. Tiles inside the bbox that belong to non-participating
//! specs' paths are marked forbidden — strategies must avoid them. If
//! a future strategy wants to also rewrite one of those specs, it can
//! be promoted to participating in a later pass (not yet implemented —
//! single-pass for the scaffold).
//!
//! Strategies consume a `Junction` snapshot (the existing
//! `bus::junction::Junction` type, which is already documented and is
//! the long-term template input). `GrowingRegion::to_junction` builds
//! the snapshot on each strategy call. Strategies also receive the
//! initial crossing tile, the current growth iteration, and references
//! to `routed_paths` + `hard_obstacles` in a context struct so new
//! fields can be added without breaking every impl.
//!
//! This module intentionally knows nothing about `BeltSpec` or any
//! ghost-router internals — strategies live in `ghost_router.rs` where
//! those types are in scope and can drive the existing templates.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::bus::junction::{BeltTier, Junction, Rect, SpecCrossing, SpecKind, SpecOrigin};
use crate::bus::region_walker::{walk_affected, AffectedPath, ShadowView, WalkResult};
use crate::common::{
    balancer_seg_is_simple, is_machine_entity, is_splitter, is_surface_belt, is_ug_belt,
    machine_dims, machine_tiles, splitter_second_tile, tile_is_forbidden_kind,
};
use crate::models::{EntityDirection, PlacedEntity, PortPoint};
use crate::trace::{
    self, BoundarySnapshot, ExternalFeederSnapshot, ParticipatingSpec, StampedNeighbor,
    TraceEvent,
};

/// Growth budget. Small on purpose — this runs per crossing tile and
/// bad inputs shouldn't melt the pipeline. Revisit once templates that
/// exploit growth are in place.
pub const MAX_GROWTH_ITERS: usize = 5;
/// Hard cap on region size. 8×8 = 64 tiles is roughly the largest
/// junction any per-tile template could reasonably stamp. Bigger than
/// this and we're in spec-run overlap territory (Sample C/D/E in the
/// RFC) which needs a different solver.
pub const MAX_REGION_TILES: usize = 64;

/// Mutable state threaded through the growth loop. Not consumed by
/// strategies directly — they see a `Junction` snapshot built via
/// `to_junction`.
#[derive(Clone)]
pub struct GrowingRegion {
    /// The crossing tile that seeded this region. Kept for trace events
    /// and strategies that want to know where the "original" problem was.
    pub initial_tile: (i32, i32),
    /// Spec keys whose paths are in the region and may be rewritten.
    pub participating: Vec<String>,
    /// Non-participating spec keys whose paths intersect the current
    /// bbox. Their tiles are in `forbidden_tiles` and strategies must
    /// treat them as obstacles.
    pub encountered: Vec<String>,
    /// All tiles currently in the region (union of every participating
    /// spec's frontier range).
    pub tiles: FxHashSet<(i32, i32)>,
    /// Tiles in the bbox that a strategy must not place belts on —
    /// either non-participating spec paths or hard obstacles (machines,
    /// poles, row template belts, etc.) that the caller passed in.
    pub forbidden_tiles: FxHashSet<(i32, i32)>,
    /// Tight enclosing rectangle around `tiles`.
    pub bbox: Rect,
    /// Per-spec path-index range currently included. Inclusive on both
    /// ends.
    frontiers: FxHashMap<String, (usize, usize)>,
}

impl GrowingRegion {
    /// Seed the region from a cluster of adjacent crossing tiles. The
    /// initial bbox is the min-rect containing every seed (filled into
    /// `tiles` to match `expand_bbox`'s rectangle invariant). Each
    /// spec's frontier is seeded at the first tile in its path that
    /// falls inside the initial bbox; specs whose path doesn't enter
    /// the bbox are silently skipped.
    ///
    /// `seeds` must be non-empty. `initial_tile` is set to `seeds[0]`
    /// for trace-event and `pending_crossings` deferred-exit purposes —
    /// cluster members other than the first are still recognized via
    /// `pending_crossings` lookups, which don't require the seed to be the
    /// representative tile.
    pub fn from_crossings(
        seeds: &[(i32, i32)],
        initial_specs: &[&str],
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        hard_obstacles: &FxHashSet<(i32, i32)>,
        strict_obstacles: &FxHashSet<(i32, i32)>,
        placed_entities: &[PlacedEntity],
        spec_kinds: &FxHashMap<String, SpecKind>,
    ) -> Self {
        assert!(!seeds.is_empty(), "from_crossings: seeds must be non-empty");
        let min_x = seeds.iter().map(|(x, _)| *x).min().unwrap();
        let max_x = seeds.iter().map(|(x, _)| *x).max().unwrap();
        let min_y = seeds.iter().map(|(_, y)| *y).min().unwrap();
        let max_y = seeds.iter().map(|(_, y)| *y).max().unwrap();
        let bbox = Rect {
            x: min_x,
            y: min_y,
            w: (max_x - min_x + 1) as u32,
            h: (max_y - min_y + 1) as u32,
        };

        let mut tiles = FxHashSet::default();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                tiles.insert((x, y));
            }
        }

        let mut frontiers = FxHashMap::default();
        let mut participating = Vec::new();
        for &key in initial_specs {
            let Some(path) = routed_paths.get(key) else {
                continue;
            };
            let Some(idx) = path.iter().position(|&t| bbox.contains(t.0, t.1)) else {
                continue;
            };
            frontiers.insert(key.to_string(), (idx, idx));
            participating.push(key.to_string());
        }
        let mut region = Self {
            initial_tile: seeds[0],
            participating,
            encountered: Vec::new(),
            tiles,
            forbidden_tiles: FxHashSet::default(),
            bbox,
            frontiers,
        };
        region.refresh_forbidden(
            routed_paths,
            hard_obstacles,
            strict_obstacles,
            placed_entities,
        );
        region.promote_blocked_encountered(routed_paths, spec_kinds);
        region
    }

    /// Promote any belt-kind encountered spec whose path crosses a
    /// forbidden interior tile within the current bbox.
    ///
    /// Without this, the SAT strategy ladder picks the cheapest
    /// satisfying strategy (sat-1ug-native before sat-2ug-native).
    /// When a participating spec can be solved with 1 UG pair AND an
    /// encountered spec's path passes through a forbidden tile, SAT
    /// believes it has solved the zone — but only the participating
    /// flow actually gets a bridge. The encountered flow's interior
    /// goes through unrouted, leaving its boundary belts dangling
    /// (validator catches this as belt-dead-end).
    ///
    /// Promoting forces SAT to model the encountered's frontier as
    /// participating, so the strategy ladder must produce a solution
    /// that bridges all flows that need bridging.
    ///
    /// `SpecKind::Pipe` is excluded — pipe paths ARE the obstacles
    /// (PTGs); SAT can't re-route them.
    fn promote_blocked_encountered(
        &mut self,
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        spec_kinds: &FxHashMap<String, SpecKind>,
    ) {
        let to_promote: Vec<String> = self
            .encountered
            .iter()
            .filter(|key| {
                if matches!(spec_kinds.get(key.as_str()), Some(SpecKind::Pipe)) {
                    return false;
                }
                let Some(path) = routed_paths.get(key.as_str()) else {
                    return false;
                };
                let Some((start, end)) = path_bbox_range(path, &self.bbox) else {
                    return false;
                };
                // Need at least one interior tile strictly between the
                // boundary ports (path[start] and path[end] are exempt
                // from forbidden checks).
                (start + 1..end).any(|i| self.forbidden_tiles.contains(&path[i]))
            })
            .cloned()
            .collect();
        if to_promote.is_empty() {
            return;
        }
        for key in &to_promote {
            if let Some(path) = routed_paths.get(key.as_str()) {
                if let Some((start, end)) = path_bbox_range(path, &self.bbox) {
                    self.frontiers.insert(key.clone(), (start, end));
                }
            }
            self.participating.push(key.clone());
        }
        self.encountered.retain(|k| !to_promote.contains(k));
    }

    /// Expand the region's bbox by the given per-side deltas, absorb
    /// every tile inside the new rectangle into the region, and
    /// recompute the frontiers of already-participating specs. A
    /// non-participating spec is *only* promoted when its path
    /// genuinely crosses another participating spec inside the new
    /// bbox (i.e. they share an interior tile). Specs that just happen
    /// to pass through the bbox without touching any participating
    /// path are left as obstacles — promoting them would over-constrain
    /// the SAT encoder with specs that aren't part of the actual
    /// crossing being resolved.
    ///
    /// This is the growth primitive used by the CEGAR loop: unlike
    /// `grow()` (which walks each spec's frontier by one step along
    /// its own axis), `expand_bbox` grows perpendicular to spec axes
    /// and can absorb perpendicular trunks the seed crossing had never
    /// heard of — the copper-cable column-4 trunk in the
    /// tier2_electronic_circuit case being the canonical example.
    ///
    /// Returns `true` if the bbox changed.
    pub fn expand_bbox(
        &mut self,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        hard_obstacles: &FxHashSet<(i32, i32)>,
        strict_obstacles: &FxHashSet<(i32, i32)>,
        placed_entities: &[PlacedEntity],
        protected_balancer_tiles: &FxHashSet<(i32, i32)>,
        spec_kinds: &FxHashMap<String, SpecKind>,
    ) -> bool {
        // Clamp each side so growth never crosses into a multi-splitter
        // balancer bbox. A side is zeroed if any tile in its new strip
        // overlaps a protected tile — the balancer becomes a hard wall
        // for the junction region. Simple single-splitter balancers are
        // not in the protected set so growth still flows through them.
        let left = clamp_side_against_protected(
            left.max(0),
            &self.bbox,
            ExpandSide::Left,
            protected_balancer_tiles,
        );
        let top = clamp_side_against_protected(
            top.max(0),
            &self.bbox,
            ExpandSide::Top,
            protected_balancer_tiles,
        );
        let right = clamp_side_against_protected(
            right.max(0),
            &self.bbox,
            ExpandSide::Right,
            protected_balancer_tiles,
        );
        let bottom = clamp_side_against_protected(
            bottom.max(0),
            &self.bbox,
            ExpandSide::Bottom,
            protected_balancer_tiles,
        );
        if left == 0 && top == 0 && right == 0 && bottom == 0 {
            return false;
        }
        let new_x = self.bbox.x - left;
        let new_y = self.bbox.y - top;
        let new_w = self.bbox.w + (left + right) as u32;
        let new_h = self.bbox.h + (top + bottom) as u32;
        self.bbox = Rect {
            x: new_x,
            y: new_y,
            w: new_w,
            h: new_h,
        };

        // Absorb every tile in the new bbox into the region's tile set.
        for dy in 0..new_h as i32 {
            for dx in 0..new_w as i32 {
                self.tiles.insert((new_x + dx, new_y + dy));
            }
        }

        let bbox = Rect {
            x: new_x,
            y: new_y,
            w: new_w,
            h: new_h,
        };
        let in_bbox = |tx: i32, ty: i32| -> bool { bbox.contains(tx, ty) };
        // Both loops below want "genuine" crossings (≥2 distinct
        // in-bbox tiles), so apply the strict `s < e` filter inline.
        let strict_range = |path: &[(i32, i32)]| -> Option<(usize, usize)> {
            path_bbox_range(path, &bbox).filter(|(s, e)| s < e)
        };

        // 1. Update frontiers for existing participating specs. Keep
        //    any spec whose in-bbox range is at least 2 tiles and has
        //    non-collapsed start < end.
        let mut new_frontiers: FxHashMap<String, (usize, usize)> = FxHashMap::default();
        let mut kept: Vec<String> = Vec::new();
        for key in &self.participating {
            if let Some(path) = routed_paths.get(key) {
                if let Some(range) = strict_range(path) {
                    new_frontiers.insert(key.clone(), range);
                    kept.push(key.clone());
                    continue;
                }
            }
            // Spec lost its in-bbox presence (shouldn't happen during
            // monotonic growth, but be defensive): drop it.
        }

        // 2. Promote a non-participating spec only if its path shares
        //    at least one in-bbox tile with an existing participating
        //    spec — that tile is the genuine crossing we want SAT to
        //    solve jointly. A spec whose path merely passes through
        //    the bbox without touching any participating path is left
        //    alone.
        let kept_tiles: FxHashSet<(i32, i32)> = kept
            .iter()
            .filter_map(|k| routed_paths.get(k))
            .flat_map(|p| {
                p.iter().copied().filter(|&(tx, ty)| in_bbox(tx, ty))
            })
            .collect();
        let kept_set: FxHashSet<&str> = kept.iter().map(|s| s.as_str()).collect();
        let mut promoted: Vec<String> = Vec::new();
        for (key, path) in routed_paths {
            if kept_set.contains(key.as_str()) {
                continue;
            }
            // Pipes never participate. See the matching filter at the
            // cluster-construction site in ghost_router.rs.
            if matches!(spec_kinds.get(key.as_str()), Some(SpecKind::Pipe)) {
                continue;
            }
            let Some(range) = strict_range(path) else {
                continue;
            };
            let (start, end) = range;
            let touches_participating = (start..=end)
                .any(|i| kept_tiles.contains(&path[i]));
            if !touches_participating {
                continue;
            }
            new_frontiers.insert(key.clone(), range);
            promoted.push(key.clone());
        }

        self.participating = kept;
        for key in promoted {
            self.participating.push(key);
        }
        self.frontiers = new_frontiers;
        self.encountered.retain(|k| !self.participating.contains(k));

        self.refresh_forbidden(
            routed_paths,
            hard_obstacles,
            strict_obstacles,
            placed_entities,
        );
        self.promote_blocked_encountered(routed_paths, spec_kinds);
        true
    }

    /// Advance each participating spec's frontier by one step in each
    /// direction along its path. Updates the bbox, the tile set, and
    /// the forbidden cache. Returns `true` if any new tile entered the
    /// region.
    #[allow(dead_code)]
    pub fn grow(
        &mut self,
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        hard_obstacles: &FxHashSet<(i32, i32)>,
        strict_obstacles: &FxHashSet<(i32, i32)>,
        placed_entities: &[PlacedEntity],
    ) -> bool {
        let mut added_any = false;
        let keys: Vec<String> = self.participating.clone();
        for key in &keys {
            let Some(path) = routed_paths.get(key) else {
                continue;
            };
            let Some(&(start, end)) = self.frontiers.get(key) else {
                continue;
            };
            let mut new_start = start;
            let mut new_end = end;
            if start > 0 {
                new_start = start - 1;
                let t = path[new_start];
                if self.tiles.insert(t) {
                    added_any = true;
                }
            }
            if end + 1 < path.len() {
                new_end = end + 1;
                let t = path[new_end];
                if self.tiles.insert(t) {
                    added_any = true;
                }
            }
            self.frontiers.insert(key.clone(), (new_start, new_end));
        }
        if added_any {
            self.recompute_bbox();
            self.refresh_forbidden(
                routed_paths,
                hard_obstacles,
                strict_obstacles,
                placed_entities,
            );
        }
        added_any
    }

    /// Number of tiles currently in the region. Checked against
    /// `MAX_REGION_TILES` by the outer loop.
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    fn recompute_bbox(&mut self) {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for &(x, y) in &self.tiles {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }
        self.bbox = Rect {
            x: min_x,
            y: min_y,
            w: (max_x - min_x + 1) as u32,
            h: (max_y - min_y + 1) as u32,
        };
    }

    fn refresh_forbidden(
        &mut self,
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        hard_obstacles: &FxHashSet<(i32, i32)>,
        strict_obstacles: &FxHashSet<(i32, i32)>,
        placed_entities: &[PlacedEntity],
    ) {
        self.forbidden_tiles.clear();
        self.encountered.clear();

        // Build a tile → entity-name index from the current layout.
        // Used to classify strict-obstacle tiles: simple surface belts
        // are *permissive* inside a SAT zone (SAT is free to lift and
        // re-stamp them). Splitters, UG entrances/exits, inserters,
        // machines, poles, pipes — all forbidden.
        let tile_kind = build_tile_kind_map(placed_entities, &self.bbox);

        // First pass: determine encountered specs (non-participating
        // specs whose path crosses the current bbox). Each contributes
        // entry + exit endpoints to the port-exemption set, so that
        // SAT can place boundary-port entities there.
        let participating: FxHashSet<&str> =
            self.participating.iter().map(|s| s.as_str()).collect();
        for (key, path) in routed_paths {
            if participating.contains(key.as_str()) {
                continue;
            }
            if path_bbox_range(path, &self.bbox).is_some()
                && !self.encountered.iter().any(|k| k == key)
            {
                self.encountered.push(key.clone());
            }
        }

        // Port tiles are exempt from forbidden. Participating frontiers
        // contribute their current entry+exit; encountered specs
        // contribute their first+last in-bbox tiles. Without this, SAT
        // rejects zones whose boundary ports land on tiles occupied by
        // Permanent feeders.
        //
        // Tiles occupied by pipe / pipe-to-ground entities are *not*
        // exempted. SAT doesn't model fluid flow, so a pipe tile is
        // never a valid port — it's an obstacle the belt must route
        // around. Without this filter, a participating fluid trunk's
        // in-bbox tiles bleed into `port_tiles`, get skipped by the
        // obstacle pass, and end up looking like flow boundaries that
        // SAT can't satisfy. The user-visible symptom is a SAT zone
        // failing UNSAT with bogus fluid boundaries instead of treating
        // the pipe as a forbidden tile to tunnel under.
        let pipe_tiles: FxHashSet<(i32, i32)> = placed_entities
            .iter()
            .filter(|e| matches!(e.name.as_str(), "pipe" | "pipe-to-ground"))
            .map(|e| (e.x, e.y))
            .collect();
        let mut port_tiles: FxHashSet<(i32, i32)> = self
            .frontiers
            .iter()
            .filter_map(|(key, &(start, end))| {
                routed_paths.get(key).map(|p| (p, start, end))
            })
            .flat_map(|(p, start, end)| [p[start], p[end]])
            .filter(|t| !pipe_tiles.contains(t))
            .collect();
        for key in &self.encountered {
            if let Some(path) = routed_paths.get(key) {
                if let Some((start, end)) = path_bbox_range(path, &self.bbox) {
                    if !pipe_tiles.contains(&path[start]) {
                        port_tiles.insert(path[start]);
                    }
                    if !pipe_tiles.contains(&path[end]) {
                        port_tiles.insert(path[end]);
                    }
                }
            }
        }

        // Walk every tile in the bbox and flag obstacles. A tile in
        // either obstacle set is forbidden iff the entity occupying it
        // is NOT a simple surface belt. Surface belts become
        // SAT-routable: SAT may re-stamp them as belts, UGs, or
        // different directions. Both `hard_obstacles` and
        // `strict_obstacles` can contain belt tiles — the ghost router
        // dumps splitter-output belts, balancer belts, and row-template
        // belts into one or the other — so we apply the same filter to
        // both. Port tiles are always exempt (a boundary port landing
        // on a Permanent would otherwise be infeasible). Tiles in an
        // obstacle set with NO matching entity in `tile_kind` default
        // forbidden (conservative).
        for y in self.bbox.y..self.bbox.y + self.bbox.h as i32 {
            for x in self.bbox.x..self.bbox.x + self.bbox.w as i32 {
                let t = (x, y);
                if port_tiles.contains(&t) {
                    continue;
                }
                let in_any_obstacle =
                    hard_obstacles.contains(&t) || strict_obstacles.contains(&t);
                if !in_any_obstacle {
                    continue;
                }
                let forbidden_kind = tile_kind
                    .get(&t)
                    .map(|n| tile_is_forbidden_kind(n))
                    .unwrap_or(true);
                if forbidden_kind {
                    self.forbidden_tiles.insert(t);
                }
            }
        }
    }

    /// Promote any encountered spec whose *entire* routed path is already
    /// inside the current tile set. These specs are "fully engulfed" —
    /// every tile they occupy is within the zone bbox — and must be routed
    /// by the SAT solver or they become orphaned crossing specs.
    ///
    /// Typical case: a 1-tile trunk column that sits directly on the path
    /// of a longer tap. The tap's crossing zone grows to include that tile,
    /// which causes the trunk to appear in `encountered`. Without promotion
    /// the SAT doesn't know about the trunk and places an entity that
    /// conflicts with it.
    #[allow(dead_code)]
    pub fn promote_fully_enclosed(
        &mut self,
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        hard_obstacles: &FxHashSet<(i32, i32)>,
        strict_obstacles: &FxHashSet<(i32, i32)>,
        placed_entities: &[PlacedEntity],
    ) {
        let to_promote: Vec<String> = self
            .encountered
            .iter()
            .filter(|key| {
                routed_paths
                    .get(key.as_str())
                    .is_some_and(|path| path.iter().all(|t| self.tiles.contains(t)))
            })
            .cloned()
            .collect();
        if to_promote.is_empty() {
            return;
        }
        for key in &to_promote {
            if let Some(path) = routed_paths.get(key.as_str()) {
                let start = 0;
                let end = path.len().saturating_sub(1);
                self.frontiers.insert(key.clone(), (start, end));
            }
            self.participating.push(key.clone());
        }
        self.encountered.retain(|k| !to_promote.contains(k));
        self.refresh_forbidden(
            routed_paths,
            hard_obstacles,
            strict_obstacles,
            placed_entities,
        );
    }

    /// Materialize a `Junction` snapshot suitable for strategy input.
    /// Entry/exit points for each participating spec are the first and
    /// last tiles of its current frontier range, with directions taken
    /// from the adjacent path step. For specs whose entire path is in
    /// the region, we fall back to the in-path step direction at the
    /// endpoint.
    pub fn to_junction(
        &self,
        routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
        spec_belt_tiers: &FxHashMap<String, BeltTier>,
        spec_items: &FxHashMap<String, String>,
        spec_exit_dirs: &FxHashMap<String, EntityDirection>,
        spec_kinds: &FxHashMap<String, SpecKind>,
    ) -> Junction {
        let mut specs: Vec<SpecCrossing> = Vec::with_capacity(self.participating.len());
        for key in &self.participating {
            let Some(path) = routed_paths.get(key) else {
                continue;
            };
            let Some(&(start, end)) = self.frontiers.get(key) else {
                continue;
            };
            let dir_hint = spec_exit_dirs.get(key).copied();
            // Use arrival direction for the entry tile: items are already
            // traveling in path[start-1]→path[start] direction when they
            // reach the zone boundary. Departure (path[start]→path[start+1])
            // is wrong at corners — e.g. a splitter above the entry tile
            // outputs South but departure would say East, causing SAT to
            // place a UG-East-In that the splitter can't feed correctly.
            let entry_dir = if start > 0 {
                direction_at(path, start - 1, dir_hint)
            } else {
                direction_at(path, start, dir_hint)
            };
            let exit_dir = direction_at(path, end, dir_hint);
            let entry = PortPoint {
                x: path[start].0,
                y: path[start].1,
                direction: entry_dir,
            };
            let exit = PortPoint {
                x: path[end].0,
                y: path[end].1,
                direction: exit_dir,
            };
            let item = spec_items
                .get(key)
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            let belt_tier = spec_belt_tiers
                .get(key)
                .copied()
                .unwrap_or(BeltTier::Yellow);
            let kind = spec_kinds.get(key).copied().unwrap_or(SpecKind::Belt);
            specs.push(SpecCrossing {
                item,
                belt_tier,
                entry,
                exit,
                origin: SpecOrigin::Participating,
                kind,
            });
        }
        // Encountered specs: paths that cross the bbox but didn't seed
        // this cluster. Each gets a pair of SAT boundaries (IN where
        // the path first enters the bbox, OUT where it last exits) so
        // SAT can route the item through — instead of treating the
        // path tiles as forbidden obstacles. Note: if the path merely
        // touches the bbox at a single tile we skip it (no meaningful
        // crossing to route). Paths that zig-zag in and out of the
        // bbox get one IN/OUT pair spanning first→last in-bbox
        // indices; intermediate excursions are collapsed (acceptable —
        // SAT isn't obliged to mirror the ghost-router path exactly).
        for key in &self.encountered {
            // Pipes never become SAT-routed flows — their tiles sit in
            // `forbidden_tiles` for the strategies to plan around. Skip
            // them here so junction.specs holds only belt specs.
            if matches!(spec_kinds.get(key), Some(SpecKind::Pipe)) {
                continue;
            }
            let Some(path) = routed_paths.get(key) else {
                continue;
            };
            let Some((start, end)) = path_bbox_range(path, &self.bbox) else {
                continue;
            };
            if start >= end {
                continue;
            }
            let dir_hint = spec_exit_dirs.get(key).copied();
            let entry_dir = if start > 0 {
                direction_at(path, start - 1, dir_hint)
            } else {
                direction_at(path, start, dir_hint)
            };
            let exit_dir = direction_at(path, end, dir_hint);
            let item = spec_items
                .get(key)
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            let belt_tier = spec_belt_tiers
                .get(key)
                .copied()
                .unwrap_or(BeltTier::Yellow);
            let kind = spec_kinds.get(key).copied().unwrap_or(SpecKind::Belt);
            specs.push(SpecCrossing {
                item,
                belt_tier,
                entry: PortPoint {
                    x: path[start].0,
                    y: path[start].1,
                    direction: entry_dir,
                },
                exit: PortPoint {
                    x: path[end].0,
                    y: path[end].1,
                    direction: exit_dir,
                },
                origin: SpecOrigin::Encountered,
                kind,
            });
        }
        Junction {
            bbox: self.bbox,
            forbidden: self.forbidden_tiles.clone(),
            specs,
        }
    }
}

/// Build a sparse (tile → entity-name) lookup covering every tile in
/// `bbox` that a `PlacedEntity` from `placed_entities` occupies.
/// Splitters contribute both tiles; machines contribute every tile in
/// their footprint; other entities occupy a single tile. Tiles with
/// no matching entity are absent from the map — callers treat those
/// as either "no obstacle" or "unknown default-forbidden" per context.
/// Which side of a bbox an `expand_bbox` request is growing into.
#[derive(Clone, Copy)]
enum ExpandSide {
    Left,
    Top,
    Right,
    Bottom,
}

/// Reduce `amount` until the new strip of tiles on `side` of `bbox`
/// contains no tile from `protected`. The strip grows outward from the
/// current bbox edge; if the tile immediately outside the edge is
/// protected, the side is clamped to 0. Amount is clamped rather than
/// shrunk in steps because expansion is usually `1` per call.
fn clamp_side_against_protected(
    amount: i32,
    bbox: &Rect,
    side: ExpandSide,
    protected: &FxHashSet<(i32, i32)>,
) -> i32 {
    if amount <= 0 || protected.is_empty() {
        return amount.max(0);
    }
    let (x0, y0, w, h) = (bbox.x, bbox.y, bbox.w as i32, bbox.h as i32);
    for step in 1..=amount {
        let hit = match side {
            ExpandSide::Left => {
                let x = x0 - step;
                (y0..y0 + h).any(|y| protected.contains(&(x, y)))
            }
            ExpandSide::Right => {
                let x = x0 + w + (step - 1);
                (y0..y0 + h).any(|y| protected.contains(&(x, y)))
            }
            ExpandSide::Top => {
                let y = y0 - step;
                (x0..x0 + w).any(|x| protected.contains(&(x, y)))
            }
            ExpandSide::Bottom => {
                let y = y0 + h + (step - 1);
                (x0..x0 + w).any(|x| protected.contains(&(x, y)))
            }
        };
        if hit {
            return step - 1;
        }
    }
    amount
}

/// Collect every tile occupied by a multi-splitter balancer (shape
/// where n>2 or m>2 in `balancer:{item}:{n}x{m}`). These tiles are
/// treated as hard boundaries by the junction growth loop so SAT never
/// gets to re-route the balancer internals. Simple single-splitter
/// balancers (1x1 / 1x2 / 2x1 / 2x2) are omitted — growth is still
/// allowed to absorb them.
pub(crate) fn build_protected_balancer_tiles(
    placed_entities: &[PlacedEntity],
) -> FxHashSet<(i32, i32)> {
    let mut out: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in placed_entities {
        let Some(seg) = e.segment_id.as_deref() else {
            continue;
        };
        // Multi-tile bus structures a crossing zone must route AROUND, never
        // through: balancer templates (`balancer:`) and merge-and-tap merge
        // trees (`mergetree:`). Without this the SAT solve grows a zone over a
        // merge-tree's splitters/belts and its committed template collides with
        // the merge-tree's Permanent claims (the `AlreadyClaimed` panic in
        // `route_bus_ghost`'s cluster-commit loop). Simple passthrough
        // balancers can be routed through; merge trees are never "simple".
        let is_balancer = seg.starts_with("balancer:");
        let is_mergetree = seg.starts_with("mergetree:");
        if !is_balancer && !is_mergetree {
            continue;
        }
        if is_balancer && balancer_seg_is_simple(seg) {
            continue;
        }
        if is_splitter(&e.name) {
            let (sx, sy) = splitter_second_tile(e);
            out.insert((e.x, e.y));
            out.insert((sx, sy));
        } else if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            for t in machine_tiles(e.x, e.y, w, h) {
                out.insert(t);
            }
        } else {
            out.insert((e.x, e.y));
        }
    }
    out
}

pub(crate) fn build_tile_kind_map<'a>(
    placed_entities: &'a [PlacedEntity],
    bbox: &Rect,
) -> FxHashMap<(i32, i32), &'a str> {
    let mut tile_kind: FxHashMap<(i32, i32), &'a str> = FxHashMap::default();
    let mut insert = |t: (i32, i32), name: &'a str| {
        if bbox.contains(t.0, t.1) {
            tile_kind.insert(t, name);
        }
    };
    for e in placed_entities {
        let name = e.name.as_str();
        if is_splitter(name) {
            let (sx, sy) = splitter_second_tile(e);
            insert((e.x, e.y), name);
            insert((sx, sy), name);
        } else if is_machine_entity(name) {
            // Assembling machines / furnaces / chemical plants / etc.
            // expand to their full footprint. machine_dims has a
            // defaults-to-(3,3) fallback for unknown names, so guard with
            // is_machine_entity to avoid treating belts as 3x3.
            let (w, h) = machine_dims(name);
            for t in machine_tiles(e.x, e.y, w, h) {
                insert(t, name);
            }
        } else {
            // Single-tile entities: belts, UG-in/out, inserters, poles,
            // pipes, beacons, etc.
            insert((e.x, e.y), name);
        }
    }
    tile_kind
}

/// First and last indices where `path` intersects `bbox`. Returns
/// `None` if the path never enters the bbox. Includes single-tile
/// crossings (start == end); callers that need strict crossings (at
/// least two distinct in-bbox tiles) should check `end > start`
/// themselves.
pub(crate) fn path_bbox_range(
    path: &[(i32, i32)],
    bbox: &Rect,
) -> Option<(usize, usize)> {
    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for (i, &(tx, ty)) in path.iter().enumerate() {
        if bbox.contains(tx, ty) {
            if first.is_none() {
                first = Some(i);
            }
            last = Some(i);
        }
    }
    match (first, last) {
        (Some(s), Some(e)) => Some((s, e)),
        _ => None,
    }
}

/// Direction of flow at path index `idx`. Looks at the next step when
/// possible, falling back to the previous step at the tail of the path.
/// `fallback` is used when the path is a single tile (no neighbours to
/// derive direction from) — callers should pass the spec's `exit_dir`.
fn direction_at(
    path: &[(i32, i32)],
    idx: usize,
    fallback: Option<EntityDirection>,
) -> EntityDirection {
    if idx + 1 < path.len() {
        let (x0, y0) = path[idx];
        let (x1, y1) = path[idx + 1];
        step_direction(x1 - x0, y1 - y0)
    } else if idx > 0 {
        let (x0, y0) = path[idx - 1];
        let (x1, y1) = path[idx];
        step_direction(x1 - x0, y1 - y0)
    } else {
        fallback.unwrap_or(EntityDirection::East)
    }
}

fn step_direction(dx: i32, dy: i32) -> EntityDirection {
    if dx > 0 {
        EntityDirection::East
    } else if dx < 0 {
        EntityDirection::West
    } else if dy > 0 {
        EntityDirection::South
    } else {
        EntityDirection::North
    }
}

/// A placed-entity list + the bbox it occupies. Returned by strategies
/// on success.
pub struct JunctionSolution {
    pub entities: Vec<PlacedEntity>,
    pub footprint: Rect,
    /// Kept for future trace instrumentation / diagnostics. Not yet
    /// consumed by the call site — the outer loop already emits a
    /// `JunctionSolved` trace event with the strategy name.
    #[allow(dead_code)]
    pub strategy_name: &'static str,
    /// Spec keys whose paths this strategy claims authority over. The
    /// ghost-router uses this to release every tile of every participating
    /// spec that falls inside `footprint`, so stale ghost-surface belts
    /// from the pre-solve A* don't leak into the final entity list.
    /// Empty for strategies whose footprint is narrow enough that residue
    /// can't form (e.g. the perpendicular 1×3 template).
    pub participating: Vec<String>,
    /// Snapshot of the SAT-zone spec when the strategy was `SatStrategy`.
    /// Populated so the ghost router can label the emitted `LayoutRegion`
    /// as `RegionKind::CrossingZone` and persist the boundaries / forced
    /// tiles / belt tier / max UG reach — everything needed to rebuild
    /// the zone for an interactive re-solve. `None` for non-SAT strategies.
    pub sat_zone: Option<SatZoneSnapshot>,
}

/// Serialisable snapshot of a SAT-solved crossing zone. Lives on
/// `JunctionSolution` so the ghost router can lower it into a
/// `LayoutRegion` without reaching back into `crate::sat`.
pub struct SatZoneSnapshot {
    pub boundaries: Vec<crate::sat::ZoneBoundary>,
    pub forced_empty: Vec<(i32, i32)>,
    pub belt_tier: String,
    pub max_ug_reach: u32,
}

/// Context passed to every strategy call. A struct so fields can be
/// added without breaking every `impl JunctionStrategy`.
pub struct JunctionStrategyContext<'a> {
    pub junction: &'a Junction,
    pub region: &'a GrowingRegion,
    /// Current region-growth iteration. 0 = initial single-tile
    /// crossing. Strategies that want cheap-first, expensive-later
    /// escalation read this; the scaffold wrapper ignores it.
    #[allow(dead_code)]
    pub growth_iter: usize,
    /// Variant label for the current attempt. Empty string for the
    /// primary attempt on the current region; non-empty for the
    /// speculative one-side expansion variants
    /// ("variant-west"/"-north"/"-east"/"-south"). Strategies forward
    /// this to their `SatInvocation` trace event so the debugger can
    /// disambiguate variant attempts that share an iter number.
    pub growth_variant: &'a str,
    pub routed_paths: &'a FxHashMap<String, Vec<(i32, i32)>>,
    pub hard_obstacles: &'a FxHashSet<(i32, i32)>,
    /// Tiles outside the narrow `hard_obstacles` set that strategies
    /// stamping interior belts (currently SAT) must also avoid: trunk
    /// columns, tap-off splitters, prior template output, row-template
    /// belts. Built from `Occupancy::snapshot_junction_obstacles`. The
    /// perpendicular-template strategy ignores this — it relies on the
    /// `release_for_pertile_template` path to clear trunks/tap-offs out
    /// of its 1×3 footprint and would refuse to fire if it saw them as
    /// obstacles.
    #[allow(dead_code)]
    pub strict_obstacles: &'a FxHashSet<(i32, i32)>,
    /// Entities already placed in Steps 2-5 (row templates, splitter
    /// stamps, balancer blocks, ghost-routed belts). Strategies that
    /// place UG inputs consult this to detect perpendicular sideloads
    /// from splitters or belts whose flow would drop items into the UG
    /// input tile from the wrong side — these sources live in
    /// `placed_entities` but never enter `routed_paths`.
    pub placed_entities: &'a [crate::models::PlacedEntity],
    /// Tiles holding a Permanent / Template / RowEntity / HardObstacle
    /// claim whose segment id is NOT `trunk:*` or `tapoff:*`. These are
    /// the claims `release_for_pertile_template` refuses to clear, so
    /// the perpendicular-template strategy must treat them as obstacles
    /// even though the comment on `strict_obstacles` says the strategy
    /// relies on release-in-footprint for trunk/tapoff cleanup. Computed
    /// once per `solve_crossing` call by the caller.
    pub unreleasable_obstacles: &'a FxHashSet<(i32, i32)>,
}

/// A strategy that attempts to produce a `JunctionSolution` for a
/// given region state. Return `None` to pass to the next strategy or
/// the next growth iteration.
pub trait JunctionStrategy {
    fn name(&self) -> &'static str;
    fn try_solve(&self, ctx: &JunctionStrategyContext) -> Option<JunctionSolution>;
}

/// Outer loop. Builds a `GrowingRegion` from a cluster of one or more
/// crossing tiles and iterates: try every strategy on the current
/// region, grow, repeat. Returns the first successful solution, or
/// `None` if every strategy failed within the growth budget.
///
/// `seeds` must be non-empty. The first seed is used as the
/// representative tile for trace events and the `pending_crossings`
/// deferred-exit check; other seeds are included in the initial
/// bbox but do not receive special treatment.
#[allow(clippy::too_many_arguments)]
pub fn solve_crossing(
    seeds: &[(i32, i32)],
    initial_specs: &[&str],
    routed_paths: &FxHashMap<String, Vec<(i32, i32)>>,
    hard_obstacles: &FxHashSet<(i32, i32)>,
    strict_obstacles: &FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &FxHashSet<(i32, i32)>,
    spec_belt_tiers: &FxHashMap<String, BeltTier>,
    spec_items: &FxHashMap<String, String>,
    spec_exit_dirs: &FxHashMap<String, EntityDirection>,
    spec_kinds: &FxHashMap<String, SpecKind>,
    placed_entities: &[crate::models::PlacedEntity],
    strategies: &[&dyn JunctionStrategy],
    // Crossing tiles that haven't been solved yet. Used to detect when
    // a spec's frontier exit lands on a *pending* crossing — in that
    // case the zone defers (grows one more step) so the solution exits
    // beyond all consecutive crossings rather than stopping mid-run.
    //
    // Important: this must exclude tiles belonging to clusters that
    // already committed their solutions. A solved crossing's tiles have
    // real entities; treating them as still-pending causes spurious
    // deferrals for zones whose exits happen to land on them.
    pending_crossings: &FxHashSet<(i32, i32)>,
) -> Option<JunctionSolution> {
    assert!(!seeds.is_empty(), "solve_crossing: seeds must be non-empty");
    let initial_tile = seeds[0];
    let mut region = GrowingRegion::from_crossings(
        seeds,
        initial_specs,
        routed_paths,
        hard_obstacles,
        strict_obstacles,
        placed_entities,
        spec_kinds,
    );

    // Emit start-of-solve snapshot: seed, participating specs, and
    // stamped entities within the seed's 1-tile perimeter. This gives
    // the replay tool everything needed to understand the initial
    // conditions before any growth happens.
    {
        let participating: Vec<ParticipatingSpec> = region
            .participating
            .iter()
            .filter_map(|key| {
                let path = routed_paths.get(key)?;
                let (start, end) = *region.frontiers.get(key)?;
                let (ix, iy) = path[start];
                let item = spec_items
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string());
                Some(ParticipatingSpec {
                    key: key.clone(),
                    item,
                    initial_tile_x: ix,
                    initial_tile_y: iy,
                    path_len: path.len(),
                    initial_start: start,
                    initial_end: end,
                })
            })
            .collect();
        let nearby_stamped = collect_nearby_stamped(initial_tile, placed_entities);
        trace::emit(TraceEvent::JunctionGrowthStarted {
            seed_x: initial_tile.0,
            seed_y: initial_tile.1,
            participating,
            nearby_stamped,
        });
    }

    let protected_balancer_tiles = build_protected_balancer_tiles(placed_entities);
    let cluster_seeds: FxHashSet<(i32, i32)> = seeds.iter().copied().collect();
    let solve_ctx = SolveCtx {
        initial_tile,
        routed_paths,
        spec_belt_tiers,
        spec_items,
        spec_exit_dirs,
        spec_kinds,
        hard_obstacles,
        strict_obstacles,
        unreleasable_obstacles,
        placed_entities,
        strategies,
        pending_crossings,
        cluster_seeds: &cluster_seeds,
        protected_balancer_tiles: &protected_balancer_tiles,
    };

    for iter in 0..MAX_GROWTH_ITERS {
        // Collect every walker-valid candidate across the primary
        // region and the four single-side expansions. The cheapest by
        // `junction_cost::solution_cost` wins — no early-return on
        // first-success, so a primary-region UG-heavy layout can be
        // beaten by a cheaper variant-east surface layout. Only if
        // every variant fails do we fall through to uniform growth.
        //
        // Variants run sequentially (SAT on these zones is low-ms) and
        // order is fixed so the trace is deterministic.
        let mut candidates: Vec<(u32, JunctionSolution, String)> = Vec::new();
        // Union of walker break tiles across every strategy × variant
        // on this iter. Drives veto-directed growth when no candidate
        // is accepted. See `docs/archive/rfc-veto-directed-growth.md`.
        let mut veto_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();

        match try_solve_on_region(&region, iter, None, &solve_ctx) {
            TryOutcome::Solved(sol) => {
                let c = crate::bus::junction_cost::solution_cost(&sol.entities);
                candidates.push((c, sol, String::new()));
            }
            TryOutcome::Continue { veto_tiles: vt } => {
                veto_tiles.extend(vt);
            }
        }

        if region.tile_count() >= MAX_REGION_TILES {
            if let Some(best) = pick_cheapest_candidate(
                &mut candidates,
                initial_tile,
                iter,
                region.tile_count(),
            ) {
                return Some(best);
            }
            trace::emit(TraceEvent::JunctionGrowthCapped {
                tile_x: initial_tile.0,
                tile_y: initial_tile.1,
                iters: iter,
                region_tiles: region.tile_count(),
                reason: "tile_cap".to_string(),
            });
            return None;
        }

        for (label, (left, top, right, bottom)) in SINGLE_SIDE_VARIANTS {
            let mut variant = region.clone();
            let changed = variant.expand_bbox(
                *left,
                *top,
                *right,
                *bottom,
                routed_paths,
                hard_obstacles,
                strict_obstacles,
                placed_entities,
                solve_ctx.protected_balancer_tiles,
                solve_ctx.spec_kinds,
            );
            if !changed {
                continue;
            }
            if variant.tile_count() > MAX_REGION_TILES {
                continue;
            }
            match try_solve_on_region(&variant, iter, Some(label), &solve_ctx) {
                TryOutcome::Solved(sol) => {
                    let c = crate::bus::junction_cost::solution_cost(&sol.entities);
                    candidates.push((c, sol, (*label).to_string()));
                }
                TryOutcome::Continue { veto_tiles: vt } => {
                    veto_tiles.extend(vt);
                }
            }
        }

        if let Some(best) = pick_cheapest_candidate(
            &mut candidates,
            initial_tile,
            iter,
            region.tile_count(),
        ) {
            return Some(best);
        }

        // Veto-directed growth: if the walker flagged tiles outside the
        // current bbox, expand the bbox just enough to cover them. The
        // first time we ran a fixture through this, ~13 of 14 distinct
        // break tiles sat outside the bbox, so this is almost always
        // productive. When every flagged tile is already inside (or no
        // strategy invoked the walker at all — SAT unsat with no
        // breaks) we fall back to uniform +1 so the region still makes
        // forward progress.
        //
        // Cap-aware special case ("growth monster"): when uniform +1
        // would push us past `MAX_REGION_TILES` AND the participating
        // set has 1-vs-N spec asymmetry along an axis (canonical case:
        // a horizontal return belt crossing N parallel vertical trunks
        // — see docs/factorio-mechanics.md and the user's processing-
        // unit @ 2/s case), grow only along the axis aligned with the
        // lone spec's direction. Doubling tile count per uniform iter
        // is what makes that case Cap with no SAT attempt at a smaller
        // bbox; halving the growth lets at least one more iter run.
        let grew = match compute_absorb_deltas(&region.bbox, &veto_tiles) {
            Some((left, top, right, bottom)) => region.expand_bbox(
                left,
                top,
                right,
                bottom,
                routed_paths,
                hard_obstacles,
                strict_obstacles,
                placed_entities,
                solve_ctx.protected_balancer_tiles,
                solve_ctx.spec_kinds,
            ),
            None => {
                let after_uniform =
                    (region.bbox.w + 2) as usize * (region.bbox.h + 2) as usize;
                let fallback = if after_uniform > MAX_REGION_TILES {
                    asymmetric_axis_deltas(&region.participating, solve_ctx.spec_exit_dirs)
                        .unwrap_or((1, 1, 1, 1))
                } else {
                    (1, 1, 1, 1)
                };
                region.expand_bbox(
                    fallback.0,
                    fallback.1,
                    fallback.2,
                    fallback.3,
                    routed_paths,
                    hard_obstacles,
                    strict_obstacles,
                    placed_entities,
                    solve_ctx.protected_balancer_tiles,
                    solve_ctx.spec_kinds,
                )
            }
        };
        if !grew {
            trace::emit(TraceEvent::JunctionGrowthCapped {
                tile_x: initial_tile.0,
                tile_y: initial_tile.1,
                iters: iter,
                region_tiles: region.tile_count(),
                reason: "bbox_expand_failed".to_string(),
            });
            return None;
        }
    }

    trace::emit(TraceEvent::JunctionGrowthCapped {
        tile_x: initial_tile.0,
        tile_y: initial_tile.1,
        iters: MAX_GROWTH_ITERS,
        region_tiles: region.tile_count(),
        reason: "iter_cap".to_string(),
    });
    None
}

/// Four single-side +1 expansions. Each tuple is `(label, (left, top,
/// right, bottom))`. Applied in a fixed order so the trace event's
/// `considered` list is deterministic — ties in cost resolve to the
/// earliest candidate to appear.
const SINGLE_SIDE_VARIANTS: &[(&str, (i32, i32, i32, i32))] = &[
    ("variant-west", (1, 0, 0, 0)),
    ("variant-north", (0, 1, 0, 0)),
    ("variant-east", (0, 0, 1, 0)),
    ("variant-south", (0, 0, 0, 1)),
];

/// Detect the "growth monster" geometry — one spec on one axis vs N≥3
/// parallel specs on the perpendicular axis — and return the axis-
/// symmetric expansion that grows along the lone spec's direction.
///
/// For a horizontal-belt × N-vertical-trunks junction the lone belt
/// extends along X; growing X (left+right) gives SAT room for an N-
/// trunk-spanning UG bridge without doubling the tile count via Y
/// growth that would just absorb more trunk path. Symmetric for the
/// transposed case (1 vertical × N horizontals → grow Y).
///
/// Returns `None` when the participating set isn't 1-vs-N — the caller
/// keeps the existing uniform-growth behavior. Specs with no entry-
/// direction recorded in `spec_exit_dirs` are ignored (they can't be
/// classified into an axis).
fn asymmetric_axis_deltas(
    participating: &[String],
    spec_exit_dirs: &FxHashMap<String, EntityDirection>,
) -> Option<(i32, i32, i32, i32)> {
    let mut horiz = 0u32;
    let mut vert = 0u32;
    for key in participating {
        match spec_exit_dirs.get(key) {
            Some(EntityDirection::East) | Some(EntityDirection::West) => horiz += 1,
            Some(EntityDirection::North) | Some(EntityDirection::South) => vert += 1,
            _ => {}
        }
    }
    // 1 horizontal + ≥3 vertical → grow X (left+right).
    // 1 vertical + ≥3 horizontal → grow Y (top+bottom).
    if horiz == 1 && vert >= 3 {
        Some((1, 0, 1, 0))
    } else if vert == 1 && horiz >= 3 {
        Some((0, 1, 0, 1))
    } else {
        None
    }
}

/// Compute the `(left, top, right, bottom)` deltas needed for
/// `expand_bbox` to absorb the **single closest** target outside the
/// bbox. Returns `None` when every target is already inside `bbox` —
/// the caller then falls back to uniform growth.
///
/// We pick one target at a time (Chebyshev-closest tile breaks ties by
/// `(y, x)` lexicographic) instead of enclosing all of them: naive
/// "enclose all" pulls the bbox wide in every direction a veto tile
/// ever appeared, blowing past `MAX_REGION_TILES` well before the
/// region reaches a satisfiable shape. One-at-a-time growth lets each
/// iteration re-run the walker on the new region, which often removes
/// several other far-flung veto tiles from consideration (the flagged
/// specs get different BFS outcomes once a new tile absorbs their
/// immediate blocker).
fn compute_absorb_deltas(
    bbox: &Rect,
    targets: &FxHashSet<(i32, i32)>,
) -> Option<(i32, i32, i32, i32)> {
    if targets.is_empty() {
        return None;
    }
    let min_x = bbox.x;
    let max_x = bbox.x + bbox.w as i32 - 1;
    let min_y = bbox.y;
    let max_y = bbox.y + bbox.h as i32 - 1;

    // Chebyshev distance from bbox edge; tiles inside have distance 0.
    let distance = |(x, y): (i32, i32)| -> i32 {
        let dx = if x < min_x {
            min_x - x
        } else if x > max_x {
            x - max_x
        } else {
            0
        };
        let dy = if y < min_y {
            min_y - y
        } else if y > max_y {
            y - max_y
        } else {
            0
        };
        dx.max(dy)
    };

    let closest = targets
        .iter()
        .copied()
        .filter(|&t| distance(t) > 0)
        .min_by_key(|&t| (distance(t), t.1, t.0))?;

    let (x, y) = closest;
    let left = if x < min_x { min_x - x } else { 0 };
    let right = if x > max_x { x - max_x } else { 0 };
    let top = if y < min_y { min_y - y } else { 0 };
    let bottom = if y > max_y { y - max_y } else { 0 };
    Some((left, top, right, bottom))
}

/// Pop the cheapest `JunctionSolution` from `candidates` (if any),
/// emit the terminal trace events for the winner, and return it. On
/// ties (same cost), the earliest candidate to have been pushed wins.
/// Returns `None` when the candidate list is empty.
fn pick_cheapest_candidate(
    candidates: &mut Vec<(u32, JunctionSolution, String)>,
    initial_tile: (i32, i32),
    iter: usize,
    region_tiles: usize,
) -> Option<JunctionSolution> {
    if candidates.is_empty() {
        return None;
    }
    let considered: Vec<(String, u32)> = candidates
        .iter()
        .map(|(c, _, label)| (label.clone(), *c))
        .collect();
    // `min_by_key` is stable in std: on equal keys the first one wins.
    let winner_idx = candidates
        .iter()
        .enumerate()
        .min_by_key(|(_, (c, _, _))| *c)
        .map(|(i, _)| i)?;
    let (winner_cost, winner_sol, winner_label) = candidates.swap_remove(winner_idx);
    trace::emit(TraceEvent::JunctionVariantChosen {
        tile_x: initial_tile.0,
        tile_y: initial_tile.1,
        iter,
        variant: winner_label.clone(),
        cost: winner_cost,
        considered,
    });
    trace::emit(TraceEvent::JunctionSolved {
        tile_x: initial_tile.0,
        tile_y: initial_tile.1,
        strategy: winner_sol.strategy_name.to_string(),
        growth_iter: iter,
        region_tiles,
    });
    Some(winner_sol)
}

/// Immutable references threaded through the growth loop. Extracted so
/// `try_solve_on_region` can be called per-iter and per-variant without
/// dragging a dozen parameters through each call.
struct SolveCtx<'a> {
    initial_tile: (i32, i32),
    routed_paths: &'a FxHashMap<String, Vec<(i32, i32)>>,
    spec_belt_tiers: &'a FxHashMap<String, BeltTier>,
    spec_items: &'a FxHashMap<String, String>,
    spec_exit_dirs: &'a FxHashMap<String, EntityDirection>,
    spec_kinds: &'a FxHashMap<String, SpecKind>,
    hard_obstacles: &'a FxHashSet<(i32, i32)>,
    strict_obstacles: &'a FxHashSet<(i32, i32)>,
    unreleasable_obstacles: &'a FxHashSet<(i32, i32)>,
    placed_entities: &'a [crate::models::PlacedEntity],
    strategies: &'a [&'a dyn JunctionStrategy],
    pending_crossings: &'a FxHashSet<(i32, i32)>,
    /// All crossing tiles that seed *this* cluster. The DeferredExit
    /// check must exclude these — a spec's exit landing on one of our
    /// own seeds is not an unresolved external crossing, it's a tile we
    /// are in the middle of solving.
    cluster_seeds: &'a FxHashSet<(i32, i32)>,
    protected_balancer_tiles: &'a FxHashSet<(i32, i32)>,
}

/// Outcome of one strategy attempt on a given region state.
enum TryOutcome {
    Solved(JunctionSolution),
    /// No strategy produced an accepted solution. `veto_tiles` contains
    /// the union of walker break tiles (and/or item-conflict tiles)
    /// collected across all strategies tried on this region. Empty when
    /// every strategy was skipped/unsat with no walker involvement.
    /// Used by `solve_crossing` to direct the next bbox expansion
    /// toward the tiles the walker actually cared about.
    Continue { veto_tiles: Vec<(i32, i32)> },
}

/// Scan the boundary list for a tile that carries more than one
/// distinct item. Such a tile is provably unroutable — one belt holds
/// one item type — so the region has to grow until the conflict sits
/// in the interior rather than on the perimeter/at a boundary. Returns
/// the offending tile coords and the sorted unique items at it.
///
/// Cheap: single pass, allocates one small map. Runs once per
/// iter/variant before we consider any strategy.
fn find_item_conflict(
    boundaries: &[crate::trace::BoundarySnapshot],
    pipe_items: &FxHashSet<String>,
) -> Option<(i32, i32, Vec<String>)> {
    use rustc_hash::FxHashMap;
    // A belt×pipe crossing puts the belt's item and the pipe's fluid on
    // the same boundary tile — structurally distinct items, but *not* a
    // belt-on-belt conflict that growth can fix. The pipe is a fixed
    // surface entity (`bridge_belt_over_pipe` handles the UG-bypass
    // directly at the 1×1 region); treating it as a conflict makes the
    // item-conflict fast-fail skip perp-template every time and grow
    // the region into a multi-spec blob that SAT then guards against.
    // Filter pipe items out before counting distinct items per tile.
    let mut by_tile: FxHashMap<(i32, i32), Vec<String>> = FxHashMap::default();
    for b in boundaries {
        if pipe_items.contains(&b.item) {
            continue;
        }
        let items = by_tile.entry((b.x, b.y)).or_default();
        if !items.iter().any(|i| i == &b.item) {
            items.push(b.item.clone());
        }
    }
    for ((x, y), mut items) in by_tile {
        if items.len() > 1 {
            items.sort();
            return Some((x, y, items));
        }
    }
    None
}

/// Snapshot the iteration, run every strategy on the region, and apply
/// the walker veto. Mirrors the original in-loop body so the outer
/// growth loop can call it per-iter *and* per-variant.
///
/// `variant` identifies the specific shape being tried when multiple
/// candidates share an `iter` number (for trace disambiguation). `None`
/// means "this is the primary attempt for the iter, no variant".
fn try_solve_on_region(
    region: &GrowingRegion,
    iter: usize,
    variant: Option<&str>,
    ctx: &SolveCtx,
) -> TryOutcome {
    let junction = region.to_junction(
        ctx.routed_paths,
        ctx.spec_belt_tiers,
        ctx.spec_items,
        ctx.spec_exit_dirs,
        ctx.spec_kinds,
    );

    let boundaries = junction_boundaries_to_snapshots(
        &junction,
        &region.participating,
        &region.encountered,
        ctx.placed_entities,
    );
    // Item-conflict fast-fail: a single boundary tile can carry at
    // most one item (a belt holds one item type). If two distinct
    // specs land boundaries on the same (x,y) with different items,
    // SAT is provably UNSAT regardless of strategy. Skip every strategy
    // for this iter and let growth expand outward to put the conflict
    // in the interior (where SAT can route around it via UG tunnels).
    // Exclude pipe-kind items — a fluid trunk sharing a boundary tile
    // with a belt isn't an unroutable conflict; `bridge_belt_over_pipe`
    // handles that shape.
    let pipe_items: FxHashSet<String> = junction
        .specs
        .iter()
        .filter(|s| s.kind == SpecKind::Pipe)
        .map(|s| s.item.clone())
        .collect();
    let conflict = find_item_conflict(&boundaries, &pipe_items);
    let mut tiles: Vec<(i32, i32)> = region.tiles.iter().copied().collect();
    tiles.sort();
    let mut forbidden: Vec<(i32, i32)> = region.forbidden_tiles.iter().copied().collect();
    forbidden.sort();
    // Variant tag for trace events. Empty string = primary attempt on
    // the current region; non-empty = speculative single-side expansion.
    // The web debugger groups per-iter state keyed by (iter, variant).
    let variant_tag = variant.unwrap_or("").to_string();

    trace::emit(TraceEvent::JunctionGrowthIteration {
        seed_x: ctx.initial_tile.0,
        seed_y: ctx.initial_tile.1,
        iter,
        variant: variant_tag.clone(),
        bbox_x: region.bbox.x,
        bbox_y: region.bbox.y,
        bbox_w: region.bbox.w,
        bbox_h: region.bbox.h,
        tiles,
        forbidden_tiles: forbidden,
        boundaries,
        participating: region.participating.clone(),
        encountered: region.encountered.clone(),
    });

    // Collected across every strategy attempt on this (iter, variant).
    // Forms the growth signal if no strategy succeeds. Ordered for
    // determinism, but the caller unions them with the other variants'
    // sets and order is lost there — so we don't rely on it.
    let mut veto_tiles: Vec<(i32, i32)> = Vec::new();

    if let Some((cx, cy, items)) = conflict {
        trace::emit(TraceEvent::JunctionStrategyAttempt {
            seed_x: ctx.initial_tile.0,
            seed_y: ctx.initial_tile.1,
            iter,
            variant: variant_tag.clone(),
            strategy: "item-conflict-check".to_string(),
            outcome: "Skipped".to_string(),
            detail: format!("({cx},{cy}) carries [{}]", items.join(", ")),
            elapsed_us: 0,
        });
        // Growth-signal: the conflict tile itself. Absorbing it pushes
        // the conflict into the bbox interior where SAT can route UGs
        // around it.
        veto_tiles.push((cx, cy));
        return TryOutcome::Continue { veto_tiles };
    }

    let strategy_ctx = JunctionStrategyContext {
        junction: &junction,
        region,
        growth_iter: iter,
        growth_variant: &variant_tag,
        routed_paths: ctx.routed_paths,
        hard_obstacles: ctx.hard_obstacles,
        strict_obstacles: ctx.strict_obstacles,
        placed_entities: ctx.placed_entities,
        unreleasable_obstacles: ctx.unreleasable_obstacles,
    };

    for strategy in ctx.strategies {
        let strategy_started = web_time::Instant::now();
        let result = strategy.try_solve(&strategy_ctx);
        let elapsed_us = strategy_started.elapsed().as_micros() as u64;
        let Some(sol) = result else {
            trace::emit(TraceEvent::JunctionStrategyAttempt {
                seed_x: ctx.initial_tile.0,
                seed_y: ctx.initial_tile.1,
                iter,
                variant: variant_tag.clone(),
                strategy: strategy.name().to_string(),
                outcome: "Unsatisfiable".to_string(),
                detail: String::new(),
                elapsed_us,
            });
            continue;
        };

        // Deferred-exit: a participating spec's frontier exits on
        // another unresolved crossing belonging to a DIFFERENT cluster
        // (either the exit tile itself, or the very next tile in the
        // exit direction — the spec would land there on the first
        // post-zone belt step). Skip this attempt so the growth loop
        // can push the frontier past all consecutive crossings before
        // committing.
        //
        // The "next tile" check matters when a UG-out at the bbox
        // edge would dump items directly into another cluster's
        // pipe-tile crossing: without it, this cluster commits a
        // surface belt run that the next cluster's solve has to
        // bridge over after the fact, and the second commit ends up
        // overwriting tiles the first one had already claimed.
        let exits_at_crossing = strategy_ctx.junction.specs.iter().any(|s| {
            should_defer_on_exit(s.exit, ctx.pending_crossings, ctx.cluster_seeds)
        });
        if exits_at_crossing {
            trace::emit(TraceEvent::JunctionStrategyAttempt {
                seed_x: ctx.initial_tile.0,
                seed_y: ctx.initial_tile.1,
                iter,
                variant: variant_tag.clone(),
                strategy: strategy.name().to_string(),
                outcome: "DeferredExit".to_string(),
                detail: "spec exits at another unresolved crossing".to_string(),
                elapsed_us,
            });
            break; // skip this iter's solution; fall through to grow
        }

        // Walker veto: shadow-view simulate the routed paths with
        // SAT's proposed placements and reject if any routed path
        // breaks. Catches locally-valid SAT solutions that break a
        // perpendicular trunk.
        //
        // For affected paths, baseline-subtract pre-existing pipe-tile
        // breaks so a belt×pipe bypass isn't vetoed by a pipe at a
        // neighbour tile that's itself a pending crossing (the walker's
        // trim_path_near_bbox range routinely includes neighbour pipe
        // columns that a later belt×pipe solve will bridge on its own).
        // Other pre-existing breaks aren't subtracted: a cluster is
        // responsible for leaving every flow it touches connected.
        let bbox = region.bbox;
        let released: FxHashSet<(i32, i32)> =
            sol.entities.iter().map(|e| (e.x, e.y)).collect();
        let affected: Vec<AffectedPath<'_>> = ctx
            .routed_paths
            .iter()
            .filter_map(|(seg, tiles)| {
                let trimmed = trim_path_near_bbox(tiles, bbox)?;
                let item = ctx.spec_items.get(seg).map(|s| s.as_str()).unwrap_or("");
                Some(AffectedPath {
                    segment_id: seg.as_str(),
                    tiles: trimmed,
                    item,
                })
            })
            .collect();
        // Belt flows crossing a pipe column are broken *in the routed
        // path* at the pipe tile (the survivor filter dropped the belt
        // there, leaving a gap). A neighbouring cluster's solve sees
        // that pre-existing break when the walker's trim range runs
        // through the pipe. Skip breaks on segments whose trimmed slice
        // runs through at least one pipe-kind spec tile — those flows
        // will be re-connected by the belt×pipe solve on that tile,
        // not by the cluster we're currently verifying.
        let pipe_tiles: FxHashSet<(i32, i32)> = ctx
            .routed_paths
            .iter()
            .filter(|(seg, _)| {
                matches!(
                    ctx.spec_kinds.get(seg.as_str()),
                    Some(SpecKind::Pipe)
                )
            })
            .flat_map(|(_, tiles)| tiles.iter().copied())
            .collect();
        let pipe_crossing_segments: FxHashSet<&str> = affected
            .iter()
            .filter(|p| p.tiles.iter().any(|t| pipe_tiles.contains(t)))
            .map(|p| p.segment_id)
            .collect();
        let shadow = ShadowView::build(ctx.placed_entities, &released, &sol.entities);
        if let WalkResult::Broken { breaks: all_breaks } = walk_affected(&affected, &shadow) {
            let breaks: Vec<_> = all_breaks
                .into_iter()
                .filter(|b| !pipe_crossing_segments.contains(b.segment_id.as_str()))
                .collect();
            if breaks.is_empty() {
                // Every reported break is on a segment crossing a pipe
                // tile — owned by the belt×pipe solve, not this one.
            } else {
            dump_walker_veto(
                ctx,
                &region.bbox,
                iter,
                &variant_tag,
                strategy.name(),
                &sol.entities,
                &affected,
                &shadow,
                &breaks,
            );
            let detail = if let Some(first) = breaks.first() {
                trace::emit(TraceEvent::RegionWalkerVeto {
                    tile_x: ctx.initial_tile.0,
                    tile_y: ctx.initial_tile.1,
                    strategy: strategy.name().to_string(),
                    growth_iter: iter,
                    variant: variant_tag.clone(),
                    broken_segment: first.segment_id.clone(),
                    break_tile_x: first.tile.0,
                    break_tile_y: first.tile.1,
                    break_count: breaks.len(),
                });
                format!(
                    "segment={} at ({},{}) breaks={}",
                    first.segment_id, first.tile.0, first.tile.1, breaks.len()
                )
            } else {
                String::new()
            };
            trace::emit(TraceEvent::JunctionStrategyAttempt {
                seed_x: ctx.initial_tile.0,
                seed_y: ctx.initial_tile.1,
                iter,
                variant: variant_tag.clone(),
                strategy: strategy.name().to_string(),
                outcome: "Vetoed".to_string(),
                detail,
                elapsed_us,
            });
            // Growth signal: remember every tile the walker flagged.
            // The outer loop unions these across strategies and
            // variants to pick the next expansion direction.
            veto_tiles.extend(breaks.iter().map(|b| b.tile));
            continue;
            }
        }

        trace::emit(TraceEvent::JunctionStrategyAttempt {
            seed_x: ctx.initial_tile.0,
            seed_y: ctx.initial_tile.1,
            iter,
            variant: variant_tag.clone(),
            strategy: strategy.name().to_string(),
            outcome: "Solved".to_string(),
            detail: format!("{} entities placed", sol.entities.len()),
            elapsed_us,
        });
        // Candidate accepted for this variant — the caller compares
        // it against the other variants' candidates by cost and emits
        // the terminal `JunctionSolved` for the winner only.
        let cost = crate::bus::junction_cost::solution_cost(&sol.entities);
        trace::emit(TraceEvent::JunctionCandidateSolved {
            tile_x: ctx.initial_tile.0,
            tile_y: ctx.initial_tile.1,
            strategy: strategy.name().to_string(),
            growth_iter: iter,
            variant: variant_tag.clone(),
            region_tiles: region.tile_count(),
            cost,
        });
        return TryOutcome::Solved(sol);
    }

    TryOutcome::Continue { veto_tiles }
}

/// Every tile inside `bbox` (inclusive on the min side, exclusive on the
/// max side, per `Rect`'s convention). Kept as a helper in case future
/// strategies want to release the whole footprint — the current walker
/// wiring releases only SAT-proposed tiles instead.
#[allow(dead_code)]
fn bbox_tiles_set(bbox: Rect) -> FxHashSet<(i32, i32)> {
    let mut out = FxHashSet::default();
    for dy in 0..bbox.h as i32 {
        for dx in 0..bbox.w as i32 {
            out.insert((bbox.x + dx, bbox.y + dy));
        }
    }
    out
}

/// True iff `(x, y)` falls inside `bbox` expanded by one tile on each
/// side. The walker uses this to decide which routed paths to check —
/// anything one tile beyond the bbox can still interact with the
/// region's boundary entities (e.g. a sideload onto a UG input).
fn near_bbox(bbox: Rect, (x, y): (i32, i32)) -> bool {
    let min_x = bbox.x - 1;
    let min_y = bbox.y - 1;
    let max_x = bbox.x + bbox.w as i32; // inclusive upper bound with +1 perimeter
    let max_y = bbox.y + bbox.h as i32;
    x >= min_x && x <= max_x && y >= min_y && y <= max_y
}

/// Should the strategy defer because `exit` (or the tile immediately
/// past it in the exit direction) lands on an unresolved crossing
/// belonging to a *different* cluster?
///
/// Returns true iff a candidate tile is in `pending_crossings` AND not
/// in `cluster_seeds`. Excluding the current cluster's own seeds is
/// critical — a spec's frontier exit landing on one of our seeds means
/// it's landing on a tile we are in the middle of solving, not an
/// external crossing to coordinate with.
///
/// Two candidates are checked:
///
/// 1. The exit tile itself. Original behaviour.
/// 2. The next tile in the exit direction (e.g. for a West-flowing
///    exit at (30, 123) the candidate is (29, 123)). This catches the
///    "exit dumps into another cluster's pipe column" case: without
///    deferring, this cluster commits a surface belt at the exit and
///    the next cluster's pipe×belt solve has to bridge over already-
///    permanent tiles, often producing an orphan UG-out the walker
///    can't pair.
fn should_defer_on_exit(
    exit: PortPoint,
    pending_crossings: &FxHashSet<(i32, i32)>,
    cluster_seeds: &FxHashSet<(i32, i32)>,
) -> bool {
    let here = (exit.x, exit.y);
    if !cluster_seeds.contains(&here) && pending_crossings.contains(&here) {
        return true;
    }
    let (dx, dy) = dir_delta(exit.direction);
    let after = (exit.x + dx, exit.y + dy);
    !cluster_seeds.contains(&after) && pending_crossings.contains(&after)
}

/// Trim a routed path down to the contiguous bbox-adjacent span.
///
/// Why trim: the walker's job is to confirm SAT's proposed placement
/// doesn't break the path's transit *through the current zone*. If the
/// path crosses other unresolved zones elsewhere in the bus (e.g. a
/// plastic-bar trunk sits on the path further along, waiting for its own
/// SAT pass), BFS from `path.first()` to `path.last()` hits that foreign
/// belt, fails, and the walker blames SAT — a false positive.
///
/// The returned slice spans `[first_near ..= last_near]` — no runway on
/// either side. Runway padding sounds safe but actively backfires: if
/// the path is about to enter another unresolved crossing, the padded
/// tail pulls a foreign belt into the walker's *target*, making end
/// unreachable regardless of what SAT emitted inside the bbox. Starting
/// at the first near_bbox tile is fine because `near_bbox` already
/// includes a 1-tile perimeter around the bbox — BFS has the zone-edge
/// tile it needs as an anchor.
///
/// Returns `None` if the path has no tile near the bbox.
fn trim_path_near_bbox(tiles: &[(i32, i32)], bbox: Rect) -> Option<&[(i32, i32)]> {
    let first = tiles.iter().position(|&t| near_bbox(bbox, t))?;
    let last = tiles.iter().rposition(|&t| near_bbox(bbox, t))?;
    Some(&tiles[first..=last])
}

// ---------------------------------------------------------------------------
// Walker-veto debug dump
// ---------------------------------------------------------------------------

/// Print the exact input the walker was given when it vetoes a SAT
/// solution. Enabled by setting `SPAGHETTIO_DUMP_WALKER_VETO=1` (optionally
/// `=seed:10,197` to filter by seed, or `=tile:11,196` to filter by
/// break tile). Off by default — adds nothing to the trace when
/// disabled.
///
/// This is a debug-only path; on wasm32 the env var is unreadable so it
/// becomes a no-op.
#[allow(clippy::too_many_arguments)]
fn dump_walker_veto(
    ctx: &SolveCtx,
    bbox: &Rect,
    iter: usize,
    variant: &str,
    strategy: &str,
    proposed: &[PlacedEntity],
    affected: &[AffectedPath<'_>],
    shadow: &crate::bus::region_walker::ShadowView,
    breaks: &[crate::bus::region_walker::WalkBreak],
) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (ctx, bbox, iter, variant, strategy, proposed, affected, shadow, breaks);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let filter = match std::env::var("SPAGHETTIO_DUMP_WALKER_VETO") {
            Ok(v) if !v.is_empty() && v != "0" => v,
            _ => return,
        };
        // Optional filters: "seed:X,Y" or "tile:X,Y". Any other non-empty
        // value enables unfiltered dumping.
        let (want_seed, want_tile) = parse_dump_filter(&filter);
        if let Some((sx, sy)) = want_seed {
            if (ctx.initial_tile.0, ctx.initial_tile.1) != (sx, sy) {
                return;
            }
        }
        if let Some((tx, ty)) = want_tile {
            if !breaks.iter().any(|b| b.tile == (tx, ty)) {
                return;
            }
        }

        eprintln!();
        eprintln!("=== walker veto dump ===");
        eprintln!(
            "  seed=({},{})  iter={}  variant={:?}  strategy={}",
            ctx.initial_tile.0, ctx.initial_tile.1, iter, variant, strategy
        );
        eprintln!(
            "  bbox=(x={}, y={}, w={}, h={})",
            bbox.x, bbox.y, bbox.w, bbox.h
        );

        eprintln!("  proposed entities ({}):", proposed.len());
        for e in proposed {
            eprintln!(
                "    ({:>3},{:>3}) {} dir={:?} carries={:?} io={:?}",
                e.x, e.y, e.name, e.direction, e.carries, e.io_type
            );
        }

        eprintln!("  affected paths ({}):", affected.len());
        for p in affected {
            let head: Vec<_> = p.tiles.iter().take(3).copied().collect();
            let tail: Vec<_> = p.tiles.iter().rev().take(3).rev().copied().collect();
            eprintln!(
                "    seg={:<40} item={:<20} len={:>3} head={:?} tail={:?}",
                p.segment_id, p.item, p.tiles.len(), head, tail
            );
        }

        eprintln!("  breaks ({}):", breaks.len());
        for b in breaks {
            eprintln!(
                "    segment={} first_bad=({},{}) reason={:?}",
                b.segment_id, b.tile.0, b.tile.1, b.reason
            );
        }

        // Shadow window around each break tile: ±2 tiles, just the
        // entities that exist in the shadow view.
        for b in breaks {
            eprintln!(
                "  shadow window around break ({},{}):",
                b.tile.0, b.tile.1
            );
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let t = (b.tile.0 + dx, b.tile.1 + dy);
                    if let Some(e) = shadow.get(t) {
                        eprintln!(
                            "    ({:>3},{:>3}) {} dir={:?} carries={:?} io={:?} seg={:?}",
                            e.x, e.y, e.name, e.direction, e.carries, e.io_type, e.segment_id
                        );
                    }
                }
            }
        }

        // For each broken path, walk every path tile and show its
        // shadow state. The first tile where the shadow entity either
        // (a) is missing, (b) carries a different item, or (c) faces a
        // direction that doesn't step toward the next path tile is the
        // real culprit.
        for b in breaks {
            let Some(p) = affected.iter().find(|p| p.segment_id == b.segment_id) else {
                continue;
            };
            eprintln!(
                "  path tile walk for {} (len={}):",
                p.segment_id, p.tiles.len()
            );
            for (i, &t) in p.tiles.iter().enumerate() {
                let next = p.tiles.get(i + 1).copied();
                match shadow.get(t) {
                    None => {
                        eprintln!(
                            "    [{:>3}] ({:>3},{:>3}) <MISSING from shadow> next={:?}",
                            i, t.0, t.1, next
                        );
                    }
                    Some(e) => {
                        let item_ok = e.carries.as_deref() == Some(p.item);
                        let step_ok = if let Some(n) = next {
                            let (dx, dy) = match e.direction {
                                EntityDirection::North => (0, -1),
                                EntityDirection::East => (1, 0),
                                EntityDirection::South => (0, 1),
                                EntityDirection::West => (-1, 0),
                            };
                            (t.0 + dx, t.1 + dy) == n
                        } else {
                            true
                        };
                        let flags = format!(
                            "item_ok={} step_ok={}",
                            if item_ok { "Y" } else { "N" },
                            if step_ok { "Y" } else { "N" },
                        );
                        eprintln!(
                            "    [{:>3}] ({:>3},{:>3}) {} dir={:?} carries={:?} seg={:?} {}",
                            i, t.0, t.1, e.name, e.direction, e.carries, e.segment_id, flags
                        );
                    }
                }
            }
        }
        eprintln!("=== end walker veto dump ===");
        eprintln!();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_dump_filter(v: &str) -> (Option<(i32, i32)>, Option<(i32, i32)>) {
    let mut seed = None;
    let mut tile = None;
    for part in v.split(';') {
        if let Some(rest) = part.strip_prefix("seed:") {
            if let Some((sx, sy)) = parse_pair(rest) {
                seed = Some((sx, sy));
            }
        } else if let Some(rest) = part.strip_prefix("tile:") {
            if let Some((tx, ty)) = parse_pair(rest) {
                tile = Some((tx, ty));
            }
        }
    }
    (seed, tile)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let (a, b) = s.split_once(',')?;
    let a = a.trim().parse().ok()?;
    let b = b.trim().parse().ok()?;
    Some((a, b))
}

// ---------------------------------------------------------------------------
// Trace helpers
// ---------------------------------------------------------------------------

fn dir_label(d: EntityDirection) -> String {
    match d {
        EntityDirection::North => "North",
        EntityDirection::East => "East",
        EntityDirection::South => "South",
        EntityDirection::West => "West",
    }
    .to_string()
}

fn dir_delta(d: EntityDirection) -> (i32, i32) {
    match d {
        EntityDirection::North => (0, -1),
        EntityDirection::East => (1, 0),
        EntityDirection::South => (0, 1),
        EntityDirection::West => (-1, 0),
    }
}

/// Entities whose footprint (1-tile for belts / UG, 2-tile for
/// splitters) lies within ±2 tiles of the seed. Includes a
/// `feeds_seed_area` hint so the replay tool can highlight likely
/// perpendicular feeders.
fn collect_nearby_stamped(seed: (i32, i32), placed: &[PlacedEntity]) -> Vec<StampedNeighbor> {
    let (sx, sy) = seed;
    let mut out = Vec::new();
    for e in placed {
        // Coarse cheap filter: within ±2 of seed.
        if (e.x - sx).abs() > 2 || (e.y - sy).abs() > 2 {
            continue;
        }
        let feeds = entity_feeds_seed_area(e, seed);
        out.push(StampedNeighbor {
            x: e.x,
            y: e.y,
            name: e.name.clone(),
            direction: dir_label(e.direction),
            carries: e.carries.clone(),
            segment_id: e.segment_id.clone(),
            feeds_seed_area: feeds,
        });
    }
    out
}

/// True iff `entity`'s output lands on the seed tile or any of its 4
/// direct neighbors. Used by `collect_nearby_stamped` to hint at likely
/// sources of external feeds.
fn entity_feeds_seed_area(entity: &PlacedEntity, seed: (i32, i32)) -> bool {
    let (sx, sy) = seed;
    // UG-ins consume; they don't emit onto the surface.
    if is_ug_belt(&entity.name) && entity.io_type.as_deref() == Some("input") {
        return false;
    }
    if !(is_surface_belt(&entity.name)
        || is_splitter(&entity.name)
        || (is_ug_belt(&entity.name) && entity.io_type.as_deref() == Some("output")))
    {
        return false;
    }
    let (dx, dy) = dir_delta(entity.direction);
    let mut targets: Vec<(i32, i32)> = Vec::with_capacity(2);
    if is_splitter(&entity.name) {
        let (s2x, s2y) = splitter_second_tile(entity);
        targets.push((entity.x + dx, entity.y + dy));
        targets.push((s2x + dx, s2y + dy));
    } else {
        targets.push((entity.x + dx, entity.y + dy));
    }
    for (tx, ty) in targets {
        if tx == sx && ty == sy {
            return true;
        }
        if (tx - sx).abs() + (ty - sy).abs() == 1 {
            return true;
        }
    }
    false
}

/// Build per-boundary snapshots from a `Junction` + participating +
/// encountered spec keys. Each SpecCrossing yields two boundaries
/// (entry + exit). The external-feeder annotation is derived by
/// scanning `placed_entities` for anything that outputs onto the entry
/// tile — same logic as the SAT strategy's `physical_feeder_direction`.
/// Participating specs fill the first N slots (by their order in
/// `participating_keys`); encountered specs follow, keyed by
/// `encountered_keys`.
fn junction_boundaries_to_snapshots(
    junction: &Junction,
    participating_keys: &[String],
    encountered_keys: &[String],
    placed: &[PlacedEntity],
) -> Vec<BoundarySnapshot> {
    let mut out = Vec::with_capacity(junction.specs.len() * 2);
    let mut p_idx = 0;
    let mut e_idx = 0;
    for sc in &junction.specs {
        let key = match sc.origin {
            SpecOrigin::Participating => {
                let k = participating_keys
                    .get(p_idx)
                    .cloned()
                    .unwrap_or_else(|| String::from("?"));
                p_idx += 1;
                k
            }
            SpecOrigin::Encountered => {
                let k = encountered_keys
                    .get(e_idx)
                    .cloned()
                    .unwrap_or_else(|| String::from("?"));
                e_idx += 1;
                k
            }
        };
        let origin = match sc.origin {
            SpecOrigin::Participating => "participating".to_string(),
            SpecOrigin::Encountered => "encountered".to_string(),
        };
        let entry_feeder = find_external_feeder((sc.entry.x, sc.entry.y), placed, &sc.item);
        let spec_tier = Some(sc.belt_tier.belt_name().to_string());
        out.push(BoundarySnapshot {
            x: sc.entry.x,
            y: sc.entry.y,
            direction: dir_label(sc.entry.direction),
            item: sc.item.clone(),
            is_input: true,
            interior: false,
            spec_key: key.clone(),
            origin: origin.clone(),
            external_feeder: entry_feeder,
            belt_tier: spec_tier.clone(),
            // Spec-level snapshot happens before SAT's channel
            // assignment. Debug consumers that want the final
            // channel_id should look at the SAT strategy's
            // BoundarySnapshot instead.
            channel_id: 0,
        });
        out.push(BoundarySnapshot {
            x: sc.exit.x,
            y: sc.exit.y,
            direction: dir_label(sc.exit.direction),
            item: sc.item.clone(),
            is_input: false,
            interior: false,
            spec_key: key,
            origin,
            external_feeder: None,
            belt_tier: spec_tier,
            channel_id: 0,
        });
    }
    out
}

/// Same item-filter rationale as `physical_feeder_hit` in
/// junction_sat_strategy: without matching on `carries`, an adjacent
/// belt of a different item reports as the feeder and the debug dump
/// shows a misleading "feeder" string (e.g. iron-ore tap approach belt
/// cited as the feeder for a copper-cable trunk whose real upstream
/// column is two tiles away). Cosmetic for display, but the confusion
/// it causes during debugging is worth eliminating.
fn find_external_feeder(
    tile: (i32, i32),
    placed: &[PlacedEntity],
    item: &str,
) -> Option<ExternalFeederSnapshot> {
    for e in placed {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::region_walker::{walk_affected, AffectedPath, ShadowView, WalkResult};
    use crate::models::PlacedEntity;

    fn belt(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".into(),
            x,
            y,
            direction: dir,
            carries: Some(item.into()),
            ..Default::default()
        }
    }

    /// Regression: `DeferredExit` was firing against the cluster's OWN
    /// seeds. At seed (25,196) with cluster seeds [(25,196), (25,197)],
    /// plastic-bar trunk's frontier exit landed on (25,197) — which is
    /// a seed of *this* cluster, not an external crossing — but defer
    /// fired anyway because the check only excluded `seeds[0]`.
    ///
    /// Fix: exclude all cluster seeds, not just the first.
    fn pp(x: i32, y: i32) -> PortPoint {
        PortPoint { x, y, direction: EntityDirection::North }
    }

    #[test]
    fn should_not_defer_on_own_cluster_seed() {
        let pending: FxHashSet<(i32, i32)> = [(25, 196), (25, 197), (40, 100)]
            .into_iter()
            .collect();
        let seeds: FxHashSet<(i32, i32)> = [(25, 196), (25, 197)].into_iter().collect();
        assert!(
            !should_defer_on_exit(pp(25, 197), &pending, &seeds),
            "exit on our own seed must not defer"
        );
        assert!(
            !should_defer_on_exit(pp(25, 196), &pending, &seeds),
            "exit on our initial seed must not defer"
        );
    }

    #[test]
    fn should_defer_on_external_pending_crossing() {
        let pending: FxHashSet<(i32, i32)> = [(25, 196), (40, 100)].into_iter().collect();
        let seeds: FxHashSet<(i32, i32)> = [(25, 196)].into_iter().collect();
        assert!(
            should_defer_on_exit(pp(40, 100), &pending, &seeds),
            "exit on a pending crossing in a different cluster must defer"
        );
    }

    #[test]
    fn should_not_defer_on_non_crossing_tile() {
        let pending: FxHashSet<(i32, i32)> = [(25, 196)].into_iter().collect();
        let seeds: FxHashSet<(i32, i32)> = [(25, 196)].into_iter().collect();
        assert!(
            !should_defer_on_exit(pp(99, 99), &pending, &seeds),
            "exit on a non-crossing tile must not defer"
        );
    }

    #[test]
    fn should_defer_on_pending_immediately_past_exit() {
        // Exit at (30, 123) flowing East; (31, 123) is a pending crossing
        // in another cluster (e.g. a fluid trunk's belt-pipe crossing).
        // The post-exit tile is the very next belt step, so committing
        // here would dump items into a tile the other cluster will
        // forbid. Defer so growth absorbs the conflict.
        let pending: FxHashSet<(i32, i32)> = [(31, 123)].into_iter().collect();
        let seeds: FxHashSet<(i32, i32)> = FxHashSet::default();
        let exit = PortPoint { x: 30, y: 123, direction: EntityDirection::East };
        assert!(
            should_defer_on_exit(exit, &pending, &seeds),
            "exit one tile west of an external pending crossing must defer"
        );
    }

    #[test]
    fn trim_path_keeps_bbox_span_only() {
        // Path runs east across 20 tiles; bbox covers x=10..=12. Trim
        // should return exactly the near_bbox span (bbox + 1-tile
        // perimeter), no runway on either side.
        let tiles: Vec<(i32, i32)> = (0..20).map(|x| (x, 5)).collect();
        let bbox = Rect { x: 10, y: 5, w: 3, h: 1 };
        let trimmed = trim_path_near_bbox(&tiles, bbox).expect("in-range path");
        // near_bbox covers x:9..=13 (bbox + 1-tile perimeter).
        assert_eq!(trimmed.first(), Some(&(9, 5)));
        assert_eq!(trimmed.last(), Some(&(13, 5)));
    }

    #[test]
    fn trim_path_returns_none_when_far_from_bbox() {
        let tiles: Vec<(i32, i32)> = (0..5).map(|x| (x, 5)).collect();
        let bbox = Rect { x: 100, y: 100, w: 3, h: 3 };
        assert!(trim_path_near_bbox(&tiles, bbox).is_none());
    }

    #[test]
    fn trim_path_clamps_to_path_bounds() {
        // Path is only 3 tiles long; bbox covers the whole thing.
        let tiles = vec![(10, 5), (11, 5), (12, 5)];
        let bbox = Rect { x: 10, y: 5, w: 3, h: 1 };
        let trimmed = trim_path_near_bbox(&tiles, bbox).expect("overlapping");
        assert_eq!(trimmed.first(), Some(&(10, 5)));
        assert_eq!(trimmed.last(), Some(&(12, 5)));
    }

    /// Regression for the walker false-positive at advanced-circuit @5/s
    /// from ores, seed (22,143):
    ///
    /// SAT proposed a correct UG tunnel for iron-ore at (21,143)→(23,143)
    /// with iron-plate crossing at (22,143) on the surface. One tile past
    /// the zone, at (25,143), sat an unresolved plastic-bar crossing —
    /// a different trunk that will be SAT-solved later. Earlier trim
    /// logic padded the tail by 2 tiles, so the trimmed path ended at
    /// (26,143) in plastic-bar territory. The walker's target tile
    /// carried the wrong item, so BFS reported Unreachable no matter
    /// what SAT proposed inside the bbox.
    ///
    /// Fix: no tail padding — target is the last bbox-adjacent tile.
    #[test]
    fn trim_targets_last_bbox_tile_not_foreign_neighbour() {
        // Iron-ore tap runs east through row 143. SAT zone is a 3x4 at
        // (21,141), its east perimeter sits at x=24. One tile further
        // east, at (25,143), an unresolved plastic-bar crossing has
        // stamped a south-flowing belt onto the path. The walker must
        // trim the path so its target is (24,143) (in-zone perimeter,
        // carrying iron-ore) — not (25,143) or (26,143) which were the
        // runway that pulled plastic-bar into view.
        let existing: Vec<PlacedEntity> = [
            belt(18, 143, EntityDirection::East, "iron-ore"),
            belt(19, 143, EntityDirection::East, "iron-ore"),
            belt(20, 143, EntityDirection::East, "iron-ore"),
            belt(21, 143, EntityDirection::East, "iron-ore"),
            belt(22, 143, EntityDirection::East, "iron-ore"),
            belt(23, 143, EntityDirection::East, "iron-ore"),
            belt(24, 143, EntityDirection::East, "iron-ore"),
            // Foreign belt from another unresolved crossing.
            belt(25, 143, EntityDirection::South, "plastic-bar"),
            belt(26, 143, EntityDirection::South, "plastic-bar"),
        ]
        .into();

        // SAT's proposal inside the 3x4 bbox — valid and complete.
        let proposed = vec![
            belt(21, 143, EntityDirection::East, "iron-ore"),
            belt(22, 143, EntityDirection::East, "iron-ore"),
            belt(23, 143, EntityDirection::East, "iron-ore"),
        ];
        let released: FxHashSet<(i32, i32)> =
            proposed.iter().map(|e| (e.x, e.y)).collect();
        let shadow = ShadowView::build(&existing, &released, &proposed);

        // Full routed path, (18,143) through (26,143).
        let full_path: Vec<(i32, i32)> = (18..=26).map(|x| (x, 143)).collect();
        let bbox = Rect { x: 21, y: 141, w: 3, h: 4 };

        let trimmed = trim_path_near_bbox(&full_path, bbox).expect("overlaps bbox");
        // near_bbox covers x:20..=24 for this bbox — trim must end at
        // (24,143), not extend into (25,143)/(26,143).
        assert_eq!(trimmed.last(), Some(&(24, 143)));

        let affected = AffectedPath {
            segment_id: "tap:iron-ore",
            tiles: trimmed,
            item: "iron-ore",
        };
        assert!(
            matches!(
                walk_affected(&[affected], &shadow),
                WalkResult::Passed
            ),
            "trimmed walker must accept SAT's local solution — the foreign belt at (25,143) is a different zone's problem"
        );
    }

    /// Regression for the walker false-positive at advanced-circuit @5/s
    /// from ores, seed (10,197):
    ///
    /// Walker was vetoing every SAT candidate at one crossing because
    /// the tap's routed_path extended east into *another* unresolved
    /// crossing (a plastic-bar trunk south-bound, 14 tiles away). BFS
    /// couldn't cross the alien belt and blamed the local SAT.
    ///
    /// The fix: trim the path to a local window around the SAT bbox
    /// before passing to the walker. The foreign belt is no longer on
    /// the trimmed path, so the walker verifies only SAT's local
    /// behaviour.
    #[test]
    fn trim_avoids_false_positive_from_foreign_belt_down_the_path() {
        // Zone is a 1×1 at (10,5). SAT is solving a trunk tile that
        // sits *adjacent* to the circuit tap (not on it) — tap runs
        // east along y=6, bbox is at y=5. The tap remains untouched by
        // the SAT proposal; it passes near the bbox perimeter but not
        // through it.
        //
        // A plastic-bar south belt sits at (18,6), which the circuit
        // tap's full path must cross 8 tiles east of the bbox. That
        // tile represents an unresolved foreign crossing the tap will
        // need to SAT-route in a later pass. If the walker checks the
        // tap's full path it will stall at (18,6) and blame the local
        // SAT — exactly the failure mode `trim_path_near_bbox` fixes.
        let mut existing: Vec<PlacedEntity> = (0..21)
            .filter(|x| *x != 18)
            .map(|x| belt(x, 6, EntityDirection::East, "circuit"))
            .collect();
        existing.push(belt(18, 6, EntityDirection::South, "plastic-bar"));

        // SAT proposal: single belt at (10,5), untouched row y=6.
        let proposed = vec![belt(10, 5, EntityDirection::South, "cable")];
        let released: FxHashSet<(i32, i32)> =
            proposed.iter().map(|e| (e.x, e.y)).collect();
        let shadow = ShadowView::build(&existing, &released, &proposed);

        // The circuit tap's full routed path, from (0,6) to (20,6).
        let full_path: Vec<(i32, i32)> = (0..21).map(|x| (x, 6)).collect();
        let bbox = Rect { x: 10, y: 5, w: 1, h: 1 };

        // Without trim: walker sees the entire path, BFS stalls at
        // (18,6)'s plastic-bar belt, reports Unreachable.
        let full_affected = AffectedPath {
            segment_id: "tap:circuit",
            tiles: &full_path,
            item: "circuit",
        };
        assert!(
            matches!(
                walk_affected(&[full_affected], &shadow),
                WalkResult::Broken { .. }
            ),
            "control: untrimmed walker must fail so the trim regression is meaningful"
        );

        // With trim: walker sees only the local window and the foreign
        // belt is no longer on the path.
        let trimmed = trim_path_near_bbox(&full_path, bbox).expect("overlaps bbox");
        let trimmed_affected = AffectedPath {
            segment_id: "tap:circuit",
            tiles: trimmed,
            item: "circuit",
        };
        assert!(
            matches!(
                walk_affected(&[trimmed_affected], &shadow),
                WalkResult::Passed
            ),
            "trim must drop the foreign belt, allowing the walker to pass"
        );
    }
}
