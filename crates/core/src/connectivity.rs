//! RFC-065 Phase 0: the connectivity IR — a **derived** topology lens over a
//! [`LayoutResult`].
//!
//! `LayoutResult.entities` is a flat list; whether a belt feeds the next
//! belt, an inserter reaches a machine, or an underground pair matches is
//! re-derived from tile adjacency independently by every consumer. This
//! module gives that re-derivation one canonical, testable home:
//!
//! - [`derive_connectivity`] — pure function from a layout to a directed
//!   entity-level flow graph ([`ConnectivityGraph`]). Nothing is stored in
//!   the artifact (derive-don't-store: a stored graph could go stale, which
//!   is the exact failure class this RFC exists to kill).
//! - [`diff`] — set difference between two graphs. Node identity is the
//!   entity **index**, and edges carry no coordinates, so the edge set is
//!   invariant under any rigid motion that preserves the entity list. This
//!   is the primitive a "topology-preserving" transform can finally assert
//!   preservation with.
//! - [`check_record_integrity`] — cross-checks the positional *records*
//!   riding in `LayoutResult` (`effective_rows` bands, `power_wires`
//!   indices) against the geometry they claim to describe. Catches the
//!   "validates clean but the ledger lies" class (see RFC-065 § Motivation).
//!   A third family, `segment_id` coordinate anchors, was built and then
//!   REMOVED when adversarial review tripped K65-1: the embedded
//!   coordinates are uniqueness keys the router does not maintain as
//!   positions (non-last tap runs deliberately start one tile east of the
//!   embedded x — `ghost_router.rs` `start_x = x + 1`; junction zones
//!   re-stamp absorbed entities with bbox-corner `crossing:` ids), and no
//!   consumer reads them as coordinates. See the RFC decision log,
//!   2026-08-04.
//! - [`scan_graph_anomalies`] — structural sanity over the derived graph
//!   (unpaired underground halves, unbound inserter hands, head-on belt
//!   contacts). On a validator-green layout this must be empty; that parity
//!   is kill criterion K65-1.
//!
//! Since Phase 1 this module is the **home** of the shared derivation
//! primitives, not a consumer of them: [`build_ug_pairs`] and
//! [`build_splitter_siblings`] are canonical here and `validate::belt_flow`
//! delegates back (its previous copy, a private duplicate in
//! `belt_structural`, and `check_underground_belt_pairs`'s inline loop all
//! collapsed onto these). UG pairing is name-filtered — nearest ahead,
//! same-direction, SAME-NAME, distance > 1 — matching
//! `check_underground_belt_pairs` and game rule U5; Phase 0's canonical
//! primitive lacked the name filter (a review-recorded fidelity gap, now
//! closed). Geometry vocabulary (`dir_to_vec`, `inserter_reach`,
//! `splitter_second_tile`, footprints) comes from `common`. Inserter
//! convention matches `belt_structural`/`belt_detour`: pickup =
//! `pos − dir·reach`, drop = `pos + dir·reach`.
//!
//! Known fidelity gaps, recorded for Phase 1 (all empirically inert on the
//! green corpus): head-on conflicts are carries-blind where the validator's
//! head-on error is same-carries-only; a perpendicular `Sideload` edge onto
//! a `UgExit` receiver models a transfer the game does not perform.
//!
//! Phase 0 scope: solid transport + inserters + machines. Pipes are a mesh,
//! not a flow lattice, and stay with `validate::fluids` until a later phase;
//! poles stay with `power_wires` (only their index integrity is checked
//! here). Nothing in this module is wired into the `validate()` dispatch —
//! kill criterion K65-4 requires Phase 0 to be additive.

use rustc_hash::FxHashMap;

use crate::common::{
    dir_to_vec, entity_size, inserter_reach, is_inserter, is_machine_entity, is_splitter,
    is_surface_belt, is_ug_belt, oriented_splitter_dims, splitter_second_tile,
};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity};
use crate::validate::{Severity, ValidationIssue};

/// Canonical underground-belt pairing (Phase 1 home; the validators
/// delegate here). An entrance pairs with the NEAREST unused exit strictly
/// ahead of it that shares its direction AND its name (game rule U5 — a
/// yellow entrance never pairs a red exit), at distance > 1; greedy in
/// entity order. Returns a bidirectional tile map (entrance ↔ exit).
pub fn build_ug_pairs(entities: &[PlacedEntity]) -> FxHashMap<(i32, i32), (i32, i32)> {
    let mut ug_inputs: Vec<&PlacedEntity> = Vec::new();
    let mut ug_outputs: Vec<&PlacedEntity> = Vec::new();
    for e in entities {
        if is_ug_belt(&e.name) {
            match e.io_type.as_deref() {
                Some("input") => ug_inputs.push(e),
                Some("output") => ug_outputs.push(e),
                _ => {}
            }
        }
    }

    let mut pairs: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    let mut used_outputs: rustc_hash::FxHashSet<(i32, i32)> = rustc_hash::FxHashSet::default();

    for inp in &ug_inputs {
        let (dx, dy) = dir_to_vec(inp.direction);
        let mut best_out: Option<&PlacedEntity> = None;
        let mut best_dist = i32::MAX;

        for out in &ug_outputs {
            if used_outputs.contains(&(out.x, out.y)) {
                continue;
            }
            if out.direction != inp.direction || out.name != inp.name {
                continue;
            }
            let rx = out.x - inp.x;
            let ry = out.y - inp.y;
            let dist = if dx != 0 {
                if ry != 0 || (rx > 0) != (dx > 0) {
                    continue;
                }
                rx.abs()
            } else {
                if rx != 0 || (ry > 0) != (dy > 0) {
                    continue;
                }
                ry.abs()
            };
            if dist > 1 && dist < best_dist {
                best_dist = dist;
                best_out = Some(out);
            }
        }

        if let Some(out) = best_out {
            pairs.insert((inp.x, inp.y), (out.x, out.y));
            pairs.insert((out.x, out.y), (inp.x, inp.y));
            used_outputs.insert((out.x, out.y));
        }
    }
    pairs
}

/// Canonical splitter footprint-sibling map (Phase 1 home): each of a
/// splitter's two tiles maps to the other.
pub fn build_splitter_siblings(entities: &[PlacedEntity]) -> FxHashMap<(i32, i32), (i32, i32)> {
    let mut siblings: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    for e in entities {
        if !is_splitter(&e.name) {
            continue;
        }
        let second = splitter_second_tile(e);
        siblings.insert((e.x, e.y), second);
        siblings.insert(second, (e.x, e.y));
    }
    siblings
}

/// Coarse per-entity role in the flow graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeClass {
    SurfaceBelt,
    /// Underground belt with `io_type == "input"`.
    UgEntrance,
    /// Underground belt with `io_type == "output"`.
    UgExit,
    Splitter,
    Inserter,
    Machine,
    /// Everything else (poles, pipes, untagged undergrounds, …). Never a
    /// flow participant in Phase 0. Untagged undergrounds land here
    /// deliberately: the engine always tags `io_type`, and guessing a role
    /// for a hand-built entity risks false findings (K65-1).
    Other,
}

/// How flow crosses from `src` to `dst`. Directed: items move src → dst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeKind {
    /// Surface flow onto a belt-like entity, arriving in line with the
    /// receiver's direction.
    BeltFlow,
    /// Surface flow arriving perpendicular to the receiver's direction.
    /// Deliberately covers both a Factorio "curve" (sole side input) and a
    /// true sideload merge: the distinction is lane-level, owned by the
    /// `belt_flow` lane walkers — the flow graph only needs the edge.
    Sideload,
    /// Underground span: a paired entrance to its exit
    /// (`build_ug_pairs` semantics).
    UgSpan,
    /// Belt-like entity feeding a splitter, aligned with the splitter's
    /// direction (Factorio splitters only accept aligned rear input).
    SplitterIn,
    /// Splitter footprint tile feeding whatever sits ahead of it.
    SplitterOut,
    /// What an inserter picks from (src = belt/machine, dst = inserter).
    InserterPickup,
    /// What an inserter drops to (src = inserter, dst = belt/machine).
    InserterDrop,
}

/// One directed flow edge between two entity indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub src: usize,
    pub dst: usize,
    pub kind: EdgeKind,
}

/// Non-flowing structural contact worth representing so a diff can see it
/// appear or vanish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConflictKind {
    /// Two belt-like entities pointing directly at each other. The game
    /// transfers nothing there; `check_belt_junctions` errors it.
    HeadOn,
}

/// A conflict between two entities, canonicalized `a < b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Conflict {
    pub a: usize,
    pub b: usize,
    pub kind: ConflictKind,
}

/// The derived flow graph. Edges and conflicts are sorted and deduped, so
/// two graphs over the same entity list compare structurally.
#[derive(Debug, Clone, Default)]
pub struct ConnectivityGraph {
    pub edges: Vec<Edge>,
    pub conflicts: Vec<Conflict>,
    /// One class per entity, index-aligned with `layout.entities`.
    pub classes: Vec<NodeClass>,
    occupancy: FxHashMap<(i32, i32), usize>,
}

impl ConnectivityGraph {
    /// Entity index occupying `tile`, if any (multi-tile footprints cover
    /// every tile they span).
    pub fn occupant(&self, tile: (i32, i32)) -> Option<usize> {
        self.occupancy.get(&tile).copied()
    }
}

/// Direction-aware footprint. Mirrors `bus::compaction::entity_dims` (the
/// splitter table first, then the E/W width/height swap for non-square
/// entities — the recycler is 2×4 and rotates); unification into `common`
/// is Phase 1 material.
fn oriented_dims(name: &str, direction: EntityDirection) -> (i32, i32) {
    if let Some((w, h)) = oriented_splitter_dims(name, direction) {
        return (w as i32, h as i32);
    }
    let (mut w, mut h) = entity_size(name);
    if matches!(direction, EntityDirection::East | EntityDirection::West) && w != h {
        std::mem::swap(&mut w, &mut h);
    }
    (w as i32, h as i32)
}

fn classify(e: &PlacedEntity) -> NodeClass {
    if is_surface_belt(&e.name) {
        NodeClass::SurfaceBelt
    } else if is_ug_belt(&e.name) {
        match e.io_type.as_deref() {
            Some("input") => NodeClass::UgEntrance,
            Some("output") => NodeClass::UgExit,
            _ => NodeClass::Other,
        }
    } else if is_splitter(&e.name) {
        NodeClass::Splitter
    } else if is_inserter(&e.name) {
        NodeClass::Inserter
    } else if is_machine_entity(&e.name) {
        NodeClass::Machine
    } else {
        NodeClass::Other
    }
}

fn opposite(d: EntityDirection) -> EntityDirection {
    match d {
        EntityDirection::North => EntityDirection::South,
        EntityDirection::East => EntityDirection::West,
        EntityDirection::South => EntityDirection::North,
        EntityDirection::West => EntityDirection::East,
    }
}

/// Derive the flow graph. Pure; `O(entities)` with hash-map tile lookups.
pub fn derive_connectivity(layout: &LayoutResult) -> ConnectivityGraph {
    let entities = &layout.entities;
    let mut classes = Vec::with_capacity(entities.len());
    let mut occupancy: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    for (i, e) in entities.iter().enumerate() {
        classes.push(classify(e));
        if is_splitter(&e.name) {
            occupancy.insert((e.x, e.y), i);
            occupancy.insert(splitter_second_tile(e), i);
        } else {
            let (w, h) = oriented_dims(&e.name, e.direction);
            for dx in 0..w.max(1) {
                for dy in 0..h.max(1) {
                    occupancy.insert((e.x + dx, e.y + dy), i);
                }
            }
        }
    }

    let ug_pairs = build_ug_pairs(&layout.entities);

    let mut edges: Vec<Edge> = Vec::new();
    let mut conflicts: Vec<Conflict> = Vec::new();

    // Surface flow from `src` leaving `from` in direction `d`. Kind
    // precedence: a splitter receiver wins (SplitterIn), then a splitter
    // source (SplitterOut), then plain geometry (BeltFlow / Sideload).
    let surface_flow = |src: usize,
                            from: (i32, i32),
                            d: EntityDirection,
                            src_is_splitter: bool,
                            edges: &mut Vec<Edge>,
                            conflicts: &mut Vec<Conflict>| {
        let (dx, dy) = dir_to_vec(d);
        let target = (from.0 + dx, from.1 + dy);
        let Some(&dst) = occupancy.get(&target) else {
            return;
        };
        if dst == src {
            return;
        }
        let receiver_dir = entities[dst].direction;
        let head_on = receiver_dir == opposite(d);
        match classes[dst] {
            NodeClass::Splitter => {
                // Splitters accept aligned rear input only; a perpendicular
                // or head-on feed transfers nothing (a stall the dead-end
                // check owns, not a merge).
                if receiver_dir == d {
                    edges.push(Edge { src, dst, kind: EdgeKind::SplitterIn });
                }
            }
            NodeClass::SurfaceBelt | NodeClass::UgEntrance => {
                if head_on {
                    conflicts.push(Conflict {
                        a: src.min(dst),
                        b: src.max(dst),
                        kind: ConflictKind::HeadOn,
                    });
                } else if src_is_splitter {
                    edges.push(Edge { src, dst, kind: EdgeKind::SplitterOut });
                } else if receiver_dir == d {
                    edges.push(Edge { src, dst, kind: EdgeKind::BeltFlow });
                } else {
                    edges.push(Edge { src, dst, kind: EdgeKind::Sideload });
                }
            }
            NodeClass::UgExit => {
                // The exit's rear is its underground mouth: an aligned
                // surface feed from behind transfers nothing. Perpendicular
                // sideloads onto the exit tile are ordinary belt physics.
                if head_on {
                    conflicts.push(Conflict {
                        a: src.min(dst),
                        b: src.max(dst),
                        kind: ConflictKind::HeadOn,
                    });
                } else if receiver_dir != d {
                    edges.push(Edge {
                        src,
                        dst,
                        kind: if src_is_splitter { EdgeKind::SplitterOut } else { EdgeKind::Sideload },
                    });
                }
            }
            // Belts do not feed machines/inserters/poles directly.
            NodeClass::Inserter | NodeClass::Machine | NodeClass::Other => {}
        }
    };

    for (i, e) in entities.iter().enumerate() {
        match classes[i] {
            NodeClass::SurfaceBelt | NodeClass::UgExit => {
                surface_flow(i, (e.x, e.y), e.direction, false, &mut edges, &mut conflicts);
            }
            NodeClass::Splitter => {
                for tile in [(e.x, e.y), splitter_second_tile(e)] {
                    surface_flow(i, tile, e.direction, true, &mut edges, &mut conflicts);
                }
            }
            NodeClass::UgEntrance => {
                // Entrances flow underground only; an unpaired entrance
                // simply has no out-edge (the absence is the signal
                // `scan_graph_anomalies` reads).
                if let Some(&exit_tile) = ug_pairs.get(&(e.x, e.y)) {
                    if let Some(&dst) = occupancy.get(&exit_tile) {
                        edges.push(Edge { src: i, dst, kind: EdgeKind::UgSpan });
                    }
                }
            }
            NodeClass::Inserter => {
                let (dx, dy) = dir_to_vec(e.direction);
                let reach = inserter_reach(&e.name);
                let pickup = (e.x - dx * reach, e.y - dy * reach);
                let drop = (e.x + dx * reach, e.y + dy * reach);
                // Splitter tiles are deliberately NOT valid hand endpoints:
                // in-game an inserter cannot pick from or drop onto a
                // splitter's footprint (same exclusion `belt_detour` makes
                // for its anchors). Binding one would bless a game-dead
                // inserter with a healthy edge; leaving it unbound makes it
                // a `scan_graph_anomalies` finding instead. The engine
                // never places this geometry (0 instances corpus-wide).
                let hand_target = |idx: usize| {
                    matches!(
                        classes[idx],
                        NodeClass::SurfaceBelt
                            | NodeClass::UgEntrance
                            | NodeClass::UgExit
                            | NodeClass::Machine
                    )
                };
                if let Some(&j) = occupancy.get(&pickup) {
                    if hand_target(j) {
                        edges.push(Edge { src: j, dst: i, kind: EdgeKind::InserterPickup });
                    }
                }
                if let Some(&k) = occupancy.get(&drop) {
                    if hand_target(k) {
                        edges.push(Edge { src: i, dst: k, kind: EdgeKind::InserterDrop });
                    }
                }
            }
            NodeClass::Machine | NodeClass::Other => {}
        }
    }

    edges.sort_unstable();
    edges.dedup();
    conflicts.sort_unstable();
    conflicts.dedup();

    ConnectivityGraph { edges, conflicts, classes, occupancy }
}

/// Set difference between two derived graphs. Meaningful when the entity
/// list is index-stable between the two derivations (rigid motions, in-place
/// rewrites); entity insertions/removals surface as edge churn localized to
/// the touched indices.
#[derive(Debug, Clone, Default)]
pub struct TopologyDiff {
    pub added_edges: Vec<Edge>,
    pub removed_edges: Vec<Edge>,
    pub added_conflicts: Vec<Conflict>,
    pub removed_conflicts: Vec<Conflict>,
}

impl TopologyDiff {
    pub fn is_empty(&self) -> bool {
        self.added_edges.is_empty()
            && self.removed_edges.is_empty()
            && self.added_conflicts.is_empty()
            && self.removed_conflicts.is_empty()
    }
}

fn sorted_diff<T: Ord + Copy>(before: &[T], after: &[T]) -> (Vec<T>, Vec<T>) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < before.len() || j < after.len() {
        match (before.get(i), after.get(j)) {
            (Some(&b), Some(&a)) if b == a => {
                i += 1;
                j += 1;
            }
            (Some(&b), Some(&a)) if b < a => {
                removed.push(b);
                i += 1;
            }
            (Some(_), Some(&a)) => {
                added.push(a);
                j += 1;
            }
            (Some(&b), None) => {
                removed.push(b);
                i += 1;
            }
            (None, Some(&a)) => {
                added.push(a);
                j += 1;
            }
            (None, None) => unreachable!(),
        }
    }
    (added, removed)
}

/// Diff two graphs (see [`TopologyDiff`] for the identity caveat).
pub fn diff(before: &ConnectivityGraph, after: &ConnectivityGraph) -> TopologyDiff {
    let (added_edges, removed_edges) = sorted_diff(&before.edges, &after.edges);
    let (added_conflicts, removed_conflicts) = sorted_diff(&before.conflicts, &after.conflicts);
    TopologyDiff { added_edges, removed_edges, added_conflicts, removed_conflicts }
}

/// Structural sanity over the derived graph. On a validator-green layout
/// this must return nothing (K65-1); each finding is one positioned issue
/// per instance (`docs/validator-reporting.md` rule 1). Not wired into
/// `validate()` in Phase 0.
pub fn scan_graph_anomalies(
    graph: &ConnectivityGraph,
    layout: &LayoutResult,
) -> Vec<ValidationIssue> {
    let n = layout.entities.len();
    let mut has_ug_out = vec![false; n];
    let mut has_ug_in = vec![false; n];
    let mut ins_pickup = vec![false; n];
    let mut ins_drop = vec![false; n];
    for edge in &graph.edges {
        match edge.kind {
            EdgeKind::UgSpan => {
                has_ug_out[edge.src] = true;
                has_ug_in[edge.dst] = true;
            }
            EdgeKind::InserterPickup => ins_pickup[edge.dst] = true,
            EdgeKind::InserterDrop => ins_drop[edge.src] = true,
            _ => {}
        }
    }

    let mut issues = Vec::new();
    for (i, e) in layout.entities.iter().enumerate() {
        match graph.classes[i] {
            NodeClass::UgEntrance if !has_ug_out[i] => {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "connectivity-anomaly",
                    format!(
                        "underground entrance at ({},{}) facing {:?} has no paired exit in the derived graph",
                        e.x, e.y, e.direction
                    ),
                    e.x,
                    e.y,
                ));
            }
            NodeClass::UgExit if !has_ug_in[i] => {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "connectivity-anomaly",
                    format!(
                        "underground exit at ({},{}) facing {:?} is claimed by no entrance in the derived graph",
                        e.x, e.y, e.direction
                    ),
                    e.x,
                    e.y,
                ));
            }
            NodeClass::Inserter if !ins_pickup[i] || !ins_drop[i] => {
                let missing = match (ins_pickup[i], ins_drop[i]) {
                    (false, false) => "pickup and drop",
                    (false, true) => "pickup",
                    _ => "drop",
                };
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "connectivity-anomaly",
                    format!(
                        "inserter at ({},{}) facing {:?} has no {} binding in the derived graph",
                        e.x, e.y, e.direction, missing
                    ),
                    e.x,
                    e.y,
                ));
            }
            _ => {}
        }
    }
    for c in &graph.conflicts {
        let e = &layout.entities[c.a];
        issues.push(ValidationIssue::with_pos(
            Severity::Error,
            "connectivity-anomaly",
            format!(
                "head-on belt contact between entities at ({},{}) and ({},{}) — no flow crosses there",
                e.x, e.y, layout.entities[c.b].x, layout.entities[c.b].y
            ),
            e.x,
            e.y,
        ));
    }
    issues
}

/// Cross-check `LayoutResult`'s positional records against its geometry.
/// Two record families (RFC-065 § Design; a third, `segment_id` anchor
/// coverage, was removed when review falsified its invariant — see the
/// module doc and the RFC decision log):
///
/// - `record-effective-rows` — every machine whose recipe has bands must
///   have its full footprint inside one (harm-calibrated: see the comment
///   at the check), and every band must contain a machine of its recipe.
/// - `record-power-wires` — stored wire endpoints must be in-bounds pole
///   entities (reach/coverage stay owned by `validate::power`).
///
/// Boundary records are deliberately not re-checked here —
/// `validate::check_boundary_record_integrity` already owns them.
pub fn check_record_integrity(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // RI-1: effective_rows bands vs machine geometry.
    //
    // Calibrated to HARMFUL staleness, by construction of the consumer:
    // `resolve_row_spec_banded` mis-attributes exactly when a machine's `y`
    // resolves to a different band than the one that placed it — i.e. when
    // the machine has crossed into a foreign-recipe band or exited every
    // own-recipe band. A ledger that is off by less than the row's internal
    // margins still resolves to the same spec and is functionally inert, so
    // membership alone would under-detect (the first draft did) and
    // edge-exactness would over-detect. The invariant with teeth, true by
    // construction for every placed row: a machine's FULL footprint lies
    // inside one own-recipe band, and bands are disjoint — so any harmful
    // shift shows up as a straddle, an exit, or a foreign-band landing.
    let mut bands_by_recipe: FxHashMap<&str, Vec<(i32, i32)>> = FxHashMap::default();
    for row in &layout.effective_rows {
        bands_by_recipe
            .entry(row.spec.recipe.as_str())
            .or_default()
            .push((row.y_start, row.y_end));
    }
    // Machines collected ONCE (entity index per recipe): both RI-1
    // directions read from this instead of rescanning the full entity list
    // per band — the naive form cost ~11 ms/call at 20k entities × 50
    // bands, a real tax on the fold search's per-candidate validate().
    let mut machines_by_recipe: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    for (i, e) in layout.entities.iter().enumerate() {
        if is_machine_entity(&e.name) {
            if let Some(recipe) = e.recipe.as_deref() {
                machines_by_recipe.entry(recipe).or_default().push(i);
            }
        }
    }
    for (&recipe, machine_indices) in {
        let mut keys: Vec<_> = machines_by_recipe.iter().collect();
        keys.sort_by_key(|(r, _)| **r);
        keys
    } {
        let Some(bands) = bands_by_recipe.get(recipe) else {
            // No ledger entry for this recipe: attribution falls back to the
            // recipe-global spec by design — not an integrity violation.
            continue;
        };
        for &i in machine_indices {
            let e = &layout.entities[i];
            let (_, mh) = oriented_dims(&e.name, e.direction);
            if bands.iter().any(|&(y0, y1)| e.y >= y0 && e.y + mh <= y1) {
                continue;
            }
            let foreign = layout.effective_rows.iter().find(|row| {
                row.spec.recipe != recipe && e.y >= row.y_start && e.y < row.y_end
            });
            let shape = if let Some(f) = foreign {
                format!(
                    "resolves into the {} band [{},{}) instead",
                    f.spec.recipe, f.y_start, f.y_end
                )
            } else if bands.iter().any(|&(y0, y1)| e.y >= y0 && e.y < y1) {
                "straddles its band's edge".to_string()
            } else {
                "sits outside every band for that recipe".to_string()
            };
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "record-effective-rows",
                format!(
                    "machine {} for {} at ({},{}) {} — banded spec attribution no longer \
                     matches the geometry that placed it",
                    e.name, recipe, e.x, e.y, shape
                ),
                e.x,
                e.y,
            ));
        }
    }
    for row in &layout.effective_rows {
        let populated = machines_by_recipe
            .get(row.spec.recipe.as_str())
            .is_some_and(|idxs| {
                idxs.iter().any(|&i| {
                    let y = layout.entities[i].y;
                    y >= row.y_start && y < row.y_end
                })
            });
        if !populated {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "record-effective-rows",
                format!(
                    "effective_rows band [{},{}) for {} contains no machine of that recipe — \
                     the ledger describes geometry that is not there",
                    row.y_start, row.y_end, row.spec.recipe
                ),
                0,
                row.y_start,
            ));
        }
    }

    // RI-3: power_wires index sanity. Pole-ness comes from the canonical
    // `power_wires::is_pole` — duplicating the name list here would be the
    // exact parallel-derivation smell this RFC campaigns against.
    if let Some(wires) = &layout.power_wires {
        for &(a, b) in wires {
            for idx in [a, b] {
                match layout.entities.get(idx as usize) {
                    None => issues.push(ValidationIssue::with_pos(
                        Severity::Error,
                        "record-power-wires",
                        format!(
                            "power wire endpoint {} is out of bounds ({} entities)",
                            idx,
                            layout.entities.len()
                        ),
                        0,
                        0,
                    )),
                    Some(e) if !crate::power_wires::is_pole(&e.name) => {
                        issues.push(ValidationIssue::with_pos(
                            Severity::Error,
                            "record-power-wires",
                            format!(
                                "power wire endpoint {} is a {} at ({},{}), not a pole — the \
                                 stored graph indexes a reordered entity list",
                                idx, e.name, e.x, e.y
                            ),
                            e.x,
                            e.y,
                        ));
                    }
                    Some(_) => {}
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EffectiveRow, MachineSpec};

    fn belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".to_string(),
            x,
            y,
            direction: dir,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        }
    }

    fn ug(x: i32, y: i32, dir: EntityDirection, io: &str) -> PlacedEntity {
        PlacedEntity {
            name: "underground-belt".to_string(),
            x,
            y,
            direction: dir,
            io_type: Some(io.to_string()),
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        }
    }

    fn inserter(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity { name: "inserter".to_string(), x, y, direction: dir, ..Default::default() }
    }

    fn machine(x: i32, y: i32, recipe: &str) -> PlacedEntity {
        PlacedEntity {
            name: "assembling-machine-2".to_string(),
            x,
            y,
            recipe: Some(recipe.to_string()),
            ..Default::default()
        }
    }

    fn layout(entities: Vec<PlacedEntity>) -> LayoutResult {
        LayoutResult { entities, width: 40, height: 40, ..Default::default() }
    }

    fn edge(g: &ConnectivityGraph, src: usize, dst: usize, kind: EdgeKind) -> bool {
        g.edges.contains(&Edge { src, dst, kind })
    }

    #[test]
    fn inline_flow_and_perpendicular_arrival() {
        use EntityDirection::{East, North};
        // (0,0)E → (1,0)E → (2,0)N: inline then perpendicular arrival.
        let lr = layout(vec![belt(0, 0, East), belt(1, 0, East), belt(2, 0, North)]);
        let g = derive_connectivity(&lr);
        assert!(edge(&g, 0, 1, EdgeKind::BeltFlow), "{:?}", g.edges);
        assert!(edge(&g, 1, 2, EdgeKind::Sideload), "{:?}", g.edges);
        assert!(g.conflicts.is_empty());
    }

    #[test]
    fn head_on_is_conflict_not_flow() {
        use EntityDirection::{East, West};
        let lr = layout(vec![belt(0, 0, East), belt(1, 0, West)]);
        let g = derive_connectivity(&lr);
        assert!(g.edges.is_empty(), "{:?}", g.edges);
        assert_eq!(
            g.conflicts,
            vec![Conflict { a: 0, b: 1, kind: ConflictKind::HeadOn }]
        );
    }

    #[test]
    fn ug_span_and_surface_resume() {
        use EntityDirection::East;
        let lr = layout(vec![
            belt(0, 0, East),
            ug(1, 0, East, "input"),
            ug(4, 0, East, "output"),
            belt(5, 0, East),
        ]);
        let g = derive_connectivity(&lr);
        assert!(edge(&g, 0, 1, EdgeKind::BeltFlow));
        assert!(edge(&g, 1, 2, EdgeKind::UgSpan));
        assert!(edge(&g, 2, 3, EdgeKind::BeltFlow));
        assert!(scan_graph_anomalies(&g, &lr).is_empty());
    }

    /// U5 pin (Phase 1 unification): a yellow entrance never pairs a red
    /// exit, even perfectly aligned — the canonical pairing is
    /// name-filtered, matching `check_underground_belt_pairs` and the
    /// game. Same-tier control pairs fine.
    #[test]
    fn ug_pairing_is_name_filtered() {
        use EntityDirection::East;
        let mixed = layout(vec![ug(0, 0, East, "input"), {
            let mut e = ug(4, 0, East, "output");
            e.name = "fast-underground-belt".to_string();
            e
        }]);
        let g = derive_connectivity(&mixed);
        assert!(
            !g.edges.iter().any(|e| e.kind == EdgeKind::UgSpan),
            "cross-tier pair must not form: {:?}",
            g.edges
        );
        assert_eq!(scan_graph_anomalies(&g, &mixed).len(), 2, "both halves orphaned");

        let same = layout(vec![ug(0, 0, East, "input"), ug(4, 0, East, "output")]);
        let g2 = derive_connectivity(&same);
        assert!(edge(&g2, 0, 1, EdgeKind::UgSpan), "{:?}", g2.edges);
    }

    #[test]
    fn unpaired_ug_halves_are_anomalies() {
        use EntityDirection::East;
        let lr = layout(vec![ug(1, 0, East, "input"), ug(10, 5, East, "output")]);
        // Pairing requires same axis; these are on different rows, so both
        // halves are orphans.
        let g = derive_connectivity(&lr);
        let anomalies = scan_graph_anomalies(&g, &lr);
        assert_eq!(anomalies.len(), 2, "{anomalies:#?}");
        assert!(anomalies.iter().all(|i| i.category == "connectivity-anomaly"));
    }

    #[test]
    fn splitter_in_and_out() {
        use EntityDirection::East;
        // Feeders into both footprint tiles, receivers off both.
        let lr = layout(vec![
            belt(0, 0, East),
            belt(0, 1, East),
            PlacedEntity {
                name: "splitter".to_string(),
                x: 1,
                y: 0,
                direction: East,
                ..Default::default()
            },
            belt(2, 0, East),
            belt(2, 1, East),
        ]);
        let g = derive_connectivity(&lr);
        assert!(edge(&g, 0, 2, EdgeKind::SplitterIn));
        assert!(edge(&g, 1, 2, EdgeKind::SplitterIn));
        assert!(edge(&g, 2, 3, EdgeKind::SplitterOut));
        assert!(edge(&g, 2, 4, EdgeKind::SplitterOut));
    }

    #[test]
    fn inserter_binds_machine_and_belt() {
        use EntityDirection::East;
        // 3×3 machine at (0,0); inserter at (3,1) picks from (2,1) inside
        // the footprint and drops onto the belt at (4,1).
        let lr = layout(vec![
            machine(0, 0, "iron-gear-wheel"),
            inserter(3, 1, East),
            belt(4, 1, East),
        ]);
        let g = derive_connectivity(&lr);
        assert!(edge(&g, 0, 1, EdgeKind::InserterPickup));
        assert!(edge(&g, 1, 2, EdgeKind::InserterDrop));
        assert!(scan_graph_anomalies(&g, &lr).is_empty());
    }

    #[test]
    fn long_handed_inserter_reaches_two() {
        use EntityDirection::East;
        let lr = layout(vec![
            machine(0, 0, "iron-gear-wheel"),
            PlacedEntity {
                name: "long-handed-inserter".to_string(),
                x: 4,
                y: 1,
                direction: East,
                ..Default::default()
            },
            belt(6, 1, East),
        ]);
        let g = derive_connectivity(&lr);
        assert!(edge(&g, 0, 1, EdgeKind::InserterPickup), "{:?}", g.edges);
        assert!(edge(&g, 1, 2, EdgeKind::InserterDrop), "{:?}", g.edges);
    }

    #[test]
    fn recycler_footprint_is_oriented() {
        use EntityDirection::{East, North};
        // A recycler is 2×4 north-facing, 4×2 east-facing. The inserter at
        // (4,1) facing East picks from (3,1) — covered only under the
        // oriented footprint.
        let lr = layout(vec![
            PlacedEntity {
                name: "recycler".to_string(),
                x: 0,
                y: 0,
                direction: East,
                recipe: Some("iron-gear-wheel-recycling".to_string()),
                ..Default::default()
            },
            inserter(4, 1, East),
            belt(5, 1, East),
        ]);
        let g = derive_connectivity(&lr);
        assert!(edge(&g, 0, 1, EdgeKind::InserterPickup), "{:?}", g.edges);
        // North-facing control: (3,1) is outside a 2×4 footprint at (0,0).
        let mut north = lr.clone();
        north.entities[0].direction = North;
        let g2 = derive_connectivity(&north);
        assert!(!edge(&g2, 0, 1, EdgeKind::InserterPickup), "{:?}", g2.edges);
    }

    #[test]
    fn diff_is_invariant_under_translation() {
        use EntityDirection::East;
        let base = vec![
            belt(0, 0, East),
            ug(1, 0, East, "input"),
            ug(4, 0, East, "output"),
            belt(5, 0, East),
            inserter(6, 0, East),
            machine(7, -1, "iron-gear-wheel"),
        ];
        let lr = layout(base.clone());
        let mut moved = lr.clone();
        for e in &mut moved.entities {
            e.x += 11;
            e.y += 7;
        }
        let d = diff(&derive_connectivity(&lr), &derive_connectivity(&moved));
        assert!(d.is_empty(), "{d:#?}");
    }

    #[test]
    fn diff_sees_a_severed_link() {
        use EntityDirection::East;
        let lr = layout(vec![belt(0, 0, East), belt(1, 0, East), belt(2, 0, East)]);
        let mut broken = lr.clone();
        broken.entities[1].y += 5; // pull the middle belt out of the run
        let d = diff(&derive_connectivity(&lr), &derive_connectivity(&broken));
        assert!(!d.is_empty());
        assert_eq!(d.removed_edges.len(), 2, "{d:#?}");
    }

    #[test]
    fn integrity_effective_rows_fires_both_directions() {
        let spec = MachineSpec {
            recipe: "iron-gear-wheel".to_string(),
            entity: "assembling-machine-2".to_string(),
            ..Default::default()
        };
        let mut lr = layout(vec![machine(0, 10, "iron-gear-wheel")]);
        lr.effective_rows =
            vec![EffectiveRow { y_start: 0, y_end: 5, spec }];
        let issues = check_record_integrity(&lr);
        // Machine outside every band + band with no machine.
        assert_eq!(issues.len(), 2, "{issues:#?}");
        assert!(issues.iter().all(|i| i.category == "record-effective-rows"));

        // Aligned: clean.
        lr.effective_rows[0].y_start = 8;
        lr.effective_rows[0].y_end = 14;
        assert!(check_record_integrity(&lr).is_empty());
    }

    /// Review nit 4: an inserter over a splitter tile is game-dead — the
    /// hand must stay UNBOUND (an anomaly), never blessed with an edge.
    #[test]
    fn inserter_over_splitter_tile_is_unbound() {
        use EntityDirection::East;
        let lr = layout(vec![
            PlacedEntity {
                name: "splitter".to_string(),
                x: 0,
                y: 0,
                direction: East,
                ..Default::default()
            },
            inserter(1, 0, East), // pickup (0,0) = splitter tile
            belt(2, 0, East),
        ]);
        let g = derive_connectivity(&lr);
        assert!(
            !g.edges.iter().any(|e| e.kind == EdgeKind::InserterPickup),
            "{:?}",
            g.edges
        );
        let anomalies = scan_graph_anomalies(&g, &lr);
        assert!(
            anomalies.iter().any(|i| i.message.contains("pickup")),
            "{anomalies:#?}"
        );
    }

    #[test]
    fn integrity_power_wires_index_sanity() {
        let mut lr = layout(vec![
            PlacedEntity {
                name: "medium-electric-pole".to_string(),
                x: 0,
                y: 0,
                ..Default::default()
            },
            belt(1, 0, EntityDirection::East),
        ]);
        lr.power_wires = Some(vec![(0, 1), (0, 9)]);
        let issues = check_record_integrity(&lr);
        // (0,1): endpoint 1 is a belt; (0,9): endpoint 9 out of bounds.
        assert_eq!(issues.len(), 2, "{issues:#?}");
        assert!(issues.iter().all(|i| i.category == "record-power-wires"));
    }

    /// K65-3 measurement instrument, not a CI gate: a synthetic serpentine
    /// at mega-chain scale (~20k belt entities), timed in whatever profile
    /// the test runs under. Run manually with
    /// `cargo test --release -p spaghettio_core connectivity::tests::bench -- --ignored --nocapture`
    /// and record the number in RFC-065's decision log.
    #[test]
    #[ignore]
    fn bench_derive_and_diff_at_mega_chain_scale() {
        use EntityDirection::{East, South, West};
        let mut entities = Vec::new();
        let cols = 200;
        let rows = 100;
        for r in 0..rows {
            let dir = if r % 2 == 0 { East } else { West };
            for c in 0..cols {
                entities.push(belt(c, r * 2, dir));
            }
            let turn_x = if r % 2 == 0 { cols - 1 } else { 0 };
            entities.push(belt(turn_x, r * 2 + 1, South));
        }
        let lr = LayoutResult {
            entities,
            width: cols + 2,
            height: rows * 2 + 2,
            ..Default::default()
        };
        let t0 = std::time::Instant::now();
        let g1 = derive_connectivity(&lr);
        let derive_ms = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = std::time::Instant::now();
        let d = diff(&g1, &derive_connectivity(&lr));
        let diff_ms = t1.elapsed().as_secs_f64() * 1e3;
        assert!(d.is_empty());
        eprintln!(
            "derive_connectivity over {} entities: {:.2} ms; derive+diff: {:.2} ms",
            lr.entities.len(),
            derive_ms,
            derive_ms + diff_ms,
        );
    }
}
