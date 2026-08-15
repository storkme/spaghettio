//! Belt connectivity, flow paths, reachability, network topology, junctions.
//!
//! Port of the belt-check functions from `src/validate.py`:
//! - `check_belt_connectivity`
//! - `check_belt_flow_path`
//! - `check_belt_network_topology`
//! - `check_belt_junctions`
//! - `check_belt_flow_reachability`
//!   Plus underground-belt helpers used by those checks.

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{
    belt_throughput_stacked, dir_to_vec, fluid_only_recipes, inserter_reach,
    inserter_target_lane, is_belt_entity, is_inserter, is_machine_entity, is_splitter,
    is_surface_belt, is_ug_belt, lane_capacity_stacked, machine_dims, machine_tiles,
    splitter_second_tile, splitter_to_surface_tier, ug_max_reach, ug_to_surface_tier,
    utilization_for, LANE_LEFT,
};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity, SolverResult};

use super::{LayoutStyle, Severity, ValidationIssue};

// ---------------------------------------------------------------------------
// Belt direction map (including splitter expansion)
// ---------------------------------------------------------------------------

// `pub(crate)` on these three: `belt_detour` reuses them rather than
// re-deriving belt adjacency / UG pairing / splitter geometry (the UG
// pairing search in particular is subtle — reach limits, interception
// checks — and this codebase's own `docs/validator-reporting.md` history is
// full of checks that quietly drifted from the canonical adjacency rules).
pub(crate) fn belt_dir_map_from(entities: &[PlacedEntity]) -> FxHashMap<(i32, i32), EntityDirection> {
    belt_dir_map_filtered(entities, false)
}

fn belt_dir_map_filtered(entities: &[PlacedEntity], skip_balancers: bool) -> FxHashMap<(i32, i32), EntityDirection> {
    let mut bdm = FxHashMap::default();
    for e in entities {
        if !is_belt_entity(&e.name) {
            continue;
        }
        if skip_balancers {
            if let Some(ref seg) = e.segment_id {
                // Sushi (mixed-item) belts (RFC Fulgora Phase 3) are not
                // single-item lanes — the sushi saturation check owns their
                // throughput; the per-item lanes downstream of the filter
                // inserters walk normally.
                if seg.starts_with("balancer:") || seg.contains(":sushi:") {
                    continue;
                }
            }
        }
        bdm.insert((e.x, e.y), e.direction);
        if is_splitter(&e.name) {
            let second = splitter_second_tile(e);
            bdm.insert(second, e.direction);
        }
    }
    bdm
}

// ---------------------------------------------------------------------------
// Belt tile set (including splitter expansion)
// ---------------------------------------------------------------------------

fn build_belt_tile_set(entities: &[PlacedEntity]) -> FxHashSet<(i32, i32)> {
    let mut tiles = FxHashSet::default();
    for e in entities {
        if is_belt_entity(&e.name) {
            tiles.insert((e.x, e.y));
            if is_splitter(&e.name) {
                tiles.insert(splitter_second_tile(e));
            }
        }
    }
    tiles
}

// ---------------------------------------------------------------------------
// Underground belt pair map
// ---------------------------------------------------------------------------

/// Delegates to the canonical [`crate::connectivity::build_ug_pairs`]
/// (RFC-065 Phase 1). NOTE a deliberate semantic tightening at the
/// hand-off: the old local copy paired by direction alone; the canonical
/// pairing is additionally NAME-FILTERED (game rule U5, matching
/// `check_underground_belt_pairs`'s own loop and `belt_structural`'s
/// now-deleted private duplicate). Divergence is possible only for
/// interleaved mixed-tier undergrounds on one axis, which the engine never
/// emits (corpus-censused; see the RFC decision log, 2026-08-04 Phase 1).
pub(crate) fn build_ug_pairs(layout: &LayoutResult) -> FxHashMap<(i32, i32), (i32, i32)> {
    crate::connectivity::build_ug_pairs(&layout.entities)
}

// ---------------------------------------------------------------------------
// Splitter sibling map
// ---------------------------------------------------------------------------

/// Delegates to the canonical [`crate::connectivity::build_splitter_siblings`]
/// (RFC-065 Phase 1). Pure code motion — bit-identical behavior.
pub(crate) fn build_splitter_siblings(layout: &LayoutResult) -> FxHashMap<(i32, i32), (i32, i32)> {
    crate::connectivity::build_splitter_siblings(&layout.entities)
}

// ---------------------------------------------------------------------------
// BFS helpers
// ---------------------------------------------------------------------------

fn bfs_belt_reach(
    starts: &FxHashSet<(i32, i32)>,
    belt_tiles: &FxHashSet<(i32, i32)>,
    ug_pairs: Option<&FxHashMap<(i32, i32), (i32, i32)>>,
) -> FxHashSet<(i32, i32)> {
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    for &s in starts {
        if visited.insert(s) {
            queue.push_back(s);
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nb = (x + dx, y + dy);
            if belt_tiles.contains(&nb) && visited.insert(nb) {
                queue.push_back(nb);
            }
        }
        if let Some(pairs) = ug_pairs {
            if let Some(&paired) = pairs.get(&(x, y)) {
                if belt_tiles.contains(&paired) && visited.insert(paired) {
                    queue.push_back(paired);
                }
            }
        }
    }
    visited
}

fn bfs_belt_downstream(
    starts: &FxHashSet<(i32, i32)>,
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    ug_pairs: Option<&FxHashMap<(i32, i32), (i32, i32)>>,
    splitter_siblings: Option<&FxHashMap<(i32, i32), (i32, i32)>>,
) -> FxHashSet<(i32, i32)> {
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    for &s in starts {
        if belt_dir_map.contains_key(&s) && visited.insert(s) {
            queue.push_back(s);
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        if let Some(&d) = belt_dir_map.get(&(x, y)) {
            let (dx, dy) = dir_to_vec(d);
            let nb = (x + dx, y + dy);
            if belt_dir_map.contains_key(&nb) && visited.insert(nb) {
                queue.push_back(nb);
            }
        }
        if let Some(pairs) = ug_pairs {
            if let Some(&paired) = pairs.get(&(x, y)) {
                if belt_dir_map.contains_key(&paired) && visited.insert(paired) {
                    queue.push_back(paired);
                }
            }
        }
        if let Some(siblings) = splitter_siblings {
            if let Some(&sib) = siblings.get(&(x, y)) {
                if belt_dir_map.contains_key(&sib) && visited.insert(sib) {
                    queue.push_back(sib);
                }
            }
        }
    }
    visited
}

/// One step of belt flow, mirroring `bfs_belt_downstream`'s three rules
/// (direction, underground tunnel, splitter sibling). Used to seed a
/// STRICTLY-DOWNSTREAM closure: `bfs_belt_downstream` includes its own starts,
/// so seeding it with the sources would let a tile count as its own supply.
fn belt_step_successors(
    t: (i32, i32),
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    ug_pairs: &FxHashMap<(i32, i32), (i32, i32)>,
    splitter_siblings: &FxHashMap<(i32, i32), (i32, i32)>,
) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    if let Some(&d) = belt_dir_map.get(&t) {
        let (dx, dy) = dir_to_vec(d);
        let nb = (t.0 + dx, t.1 + dy);
        if belt_dir_map.contains_key(&nb) {
            out.push(nb);
        }
    }
    if let Some(&p) = ug_pairs.get(&t) {
        if belt_dir_map.contains_key(&p) {
            out.push(p);
        }
    }
    if let Some(&sib) = splitter_siblings.get(&t) {
        if belt_dir_map.contains_key(&sib) {
            out.push(sib);
        }
    }
    out
}

/// Mirror of [`belt_step_successors`] for the upstream direction.
fn belt_step_predecessors(
    t: (i32, i32),
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    ug_pairs: &FxHashMap<(i32, i32), (i32, i32)>,
    splitter_siblings: &FxHashMap<(i32, i32), (i32, i32)>,
) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for (ddx, ddy) in [(1, 0i32), (-1, 0), (0, 1), (0, -1)] {
        let n = (t.0 + ddx, t.1 + ddy);
        if let Some(&nd) = belt_dir_map.get(&n) {
            let (ndx, ndy) = dir_to_vec(nd);
            if (n.0 + ndx, n.1 + ndy) == t {
                out.push(n);
            }
        }
    }
    if let Some(&p) = ug_pairs.get(&t) {
        if belt_dir_map.contains_key(&p) {
            out.push(p);
        }
    }
    if let Some(&sib) = splitter_siblings.get(&t) {
        if belt_dir_map.contains_key(&sib) {
            out.push(sib);
        }
    }
    out
}

fn bfs_belt_upstream(
    starts: &FxHashSet<(i32, i32)>,
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    ug_pairs: Option<&FxHashMap<(i32, i32), (i32, i32)>>,
    splitter_siblings: Option<&FxHashMap<(i32, i32), (i32, i32)>>,
) -> FxHashSet<(i32, i32)> {
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    for &s in starts {
        if belt_dir_map.contains_key(&s) && visited.insert(s) {
            queue.push_back(s);
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        // Underground tunnel jump (reverse)
        if let Some(pairs) = ug_pairs {
            if let Some(&paired) = pairs.get(&(x, y)) {
                if belt_dir_map.contains_key(&paired) && visited.insert(paired) {
                    queue.push_back(paired);
                }
            }
        }
        // Splitter sibling
        if let Some(siblings) = splitter_siblings {
            if let Some(&sib) = siblings.get(&(x, y)) {
                if belt_dir_map.contains_key(&sib) && visited.insert(sib) {
                    queue.push_back(sib);
                }
            }
        }
        // Upstream neighbours: tiles whose direction points at (x, y)
        for (ddx, ddy) in [(1, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + ddx, y + ddy);
            if let Some(&nd) = belt_dir_map.get(&(nx, ny)) {
                let (ndx, ndy) = dir_to_vec(nd);
                if (nx + ndx, ny + ndy) == (x, y) && visited.insert((nx, ny)) {
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    visited
}

// ---------------------------------------------------------------------------
// Machine tile helpers
// ---------------------------------------------------------------------------

fn build_machine_tile_set(layout: &LayoutResult) -> FxHashSet<(i32, i32)> {
    let mut tiles = FxHashSet::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            for t in machine_tiles(e.x, e.y, w, h) {
                tiles.insert(t);
            }
        }
    }
    tiles
}

/// Map each machine tile → machine origin `(e.x, e.y)`.
fn build_machine_by_tile(layout: &LayoutResult) -> FxHashMap<(i32, i32), (i32, i32)> {
    let mut by_tile = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            for t in machine_tiles(e.x, e.y, w, h) {
                by_tile.insert(t, (e.x, e.y));
            }
        }
    }
    by_tile
}

// ---------------------------------------------------------------------------
// 1. check_belt_connectivity
// ---------------------------------------------------------------------------

pub fn check_belt_connectivity(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let fluid_only = fluid_only_recipes(solver);
    let fluid_fed = crate::common::fluid_input_only_recipes(solver);
    let belt_tiles = build_belt_tile_set(&layout.entities);
    let ug_pairs = build_ug_pairs(layout);
    let inserter_positions: FxHashSet<(i32, i32)> = layout
        .entities
        .iter()
        .filter(|e| is_inserter(&e.name))
        .map(|e| (e.x, e.y))
        .collect();
    // Inserters that belong to a direct-insertion cell. An adjacent one is
    // proof a machine's product has somewhere to go without a belt.
    let coupler_positions: FxHashSet<(i32, i32)> = layout
        .entities
        .iter()
        .filter(|e| is_inserter(&e.name) && super::is_di_cell_entity(e.segment_id.as_deref()))
        .map(|e| (e.x, e.y))
        .collect();

    if belt_tiles.is_empty() {
        let has_solid = layout.entities.iter().any(|e| {
            is_machine_entity(&e.name)
                && e.recipe
                    .as_deref()
                    .is_none_or(|r| !fluid_only.contains(r))
        });
        if has_solid {
            issues.push(ValidationIssue::new(
                Severity::Error,
                "belt-connectivity",
                "No belts in layout but machines require solid item transport",
            ));
        }
        return issues;
    }

    let mut checked: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        if !checked.insert((e.x, e.y)) {
            continue;
        }
        if e.recipe.as_deref().is_some_and(|r| fluid_only.contains(r)) {
            continue;
        }

        let (mw, mh) = machine_dims(&e.name);
        let (mw, mh) = (mw as i32, mh as i32);
        let my_tiles: FxHashSet<(i32, i32)> = (0..mw)
            .flat_map(|dx| (0..mh).map(move |dy| (e.x + dx, e.y + dy)))
            .collect();

        // Adjacent inserters
        let mut adjacent_inserters: Vec<(i32, i32)> = Vec::new();
        for dx in -1..=mw {
            for dy in -1..=mh {
                let pos = (e.x + dx, e.y + dy);
                if inserter_positions.contains(&pos) && !my_tiles.contains(&pos) {
                    adjacent_inserters.push(pos);
                }
            }
        }

        // A fluid-fed producer inside a direct-insertion cell takes its
        // ingredients through a pipe and hands its solid product straight
        // to the neighbouring machine, so no inserter of its ever touches a
        // belt (RFC-053 pipe cut). Deliberately narrow: it needs a coupler
        // adjacent (proof the product has a route) AND no solid ingredient
        // (nothing left that a belt would have had to deliver), so a cell
        // machine that fails to get a real belt is still caught.
        if crate::validate::is_di_cell_entity(e.segment_id.as_deref())
            && e.recipe.as_deref().is_some_and(|r| fluid_fed.contains(r))
            && adjacent_inserters.iter().any(|p| coupler_positions.contains(p))
        {
            continue;
        }

        // Check if any inserter has a belt on its non-machine side
        let mut has_belt_connection = false;
        'outer: for (ix, iy) in &adjacent_inserters {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nb = (ix + dx, iy + dy);
                if belt_tiles.contains(&nb) && !my_tiles.contains(&nb) {
                    has_belt_connection = true;
                    break 'outer;
                }
            }
        }

        if !has_belt_connection {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "belt-connectivity",
                format!(
                    "{} at ({},{}): no inserter connects to a belt \
                     (inserters exist but none touch a belt tile)",
                    e.name, e.x, e.y
                ),
                e.x,
                e.y,
            ));
            continue;
        }

        // Collect starting belt tiles from inserters adjacent to this machine
        let mut start_belt_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
        for (ix, iy) in &adjacent_inserters {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nb = (ix + dx, iy + dy);
                if belt_tiles.contains(&nb) && !my_tiles.contains(&nb) {
                    start_belt_tiles.insert(nb);
                }
            }
        }

        let belt_network = bfs_belt_reach(&start_belt_tiles, &belt_tiles, Some(&ug_pairs));
        if belt_network.len() <= 1 {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "belt-connectivity",
                format!(
                    "{} at ({},{}): belt adjacent to inserter is isolated (single tile, not connected to anything)",
                    e.name, e.x, e.y
                ),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 2. check_belt_flow_path
// ---------------------------------------------------------------------------

pub fn check_belt_flow_path(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
    style: LayoutStyle,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let fluid_only = fluid_only_recipes(solver);
    let ug_pairs = build_ug_pairs(layout);
    let belt_tiles = build_belt_tile_set(&layout.entities);

    if belt_tiles.is_empty() {
        return issues;
    }

    let mut inserter_entities: Vec<&PlacedEntity> = Vec::new();
    let mut inserter_positions: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if is_inserter(&e.name) {
            inserter_entities.push(e);
            inserter_positions.insert((e.x, e.y));
        }
    }

    let all_machine_tiles = build_machine_tile_set(layout);

    // Classify inserters as input (drops into machine) or output (picks from machine)
    let mut input_inserter_positions: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut output_inserter_positions: FxHashSet<(i32, i32)> = FxHashSet::default();
    for ins in &inserter_entities {
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);
        if all_machine_tiles.contains(&drop_pos) {
            input_inserter_positions.insert((ins.x, ins.y));
        }
        if all_machine_tiles.contains(&pickup_pos) {
            output_inserter_positions.insert((ins.x, ins.y));
        }
    }

    // Layout boundary from belt positions
    let all_xs: Vec<i32> = belt_tiles.iter().map(|&(x, _)| x).collect();
    let all_ys: Vec<i32> = belt_tiles.iter().map(|&(_, y)| y).collect();
    let min_bx = *all_xs.iter().min().unwrap();
    let max_bx = *all_xs.iter().max().unwrap();
    let min_by = *all_ys.iter().min().unwrap();
    let max_by = *all_ys.iter().max().unwrap();

    let on_boundary = |bx: i32, by: i32| -> bool {
        bx == min_bx || bx == max_bx || by == min_by || by == max_by
    };
    let network_reaches_boundary = |network: &FxHashSet<(i32, i32)>| -> bool {
        network.len() >= 3 && network.iter().any(|&(bx, by)| on_boundary(bx, by))
    };

    // Recipes with solid outputs
    let mut solid_output_recipes: FxHashSet<String> = FxHashSet::default();
    if let Some(sr) = solver {
        for ms in &sr.machines {
            if ms.outputs.iter().any(|o| !o.is_fluid) {
                solid_output_recipes.insert(ms.recipe.clone());
            }
        }
    }

    let severity = if style == LayoutStyle::Spaghetti {
        Severity::Error
    } else {
        Severity::Warning
    };

    let mut checked: FxHashSet<(i32, i32)> = FxHashSet::default();
    let machine_entities: Vec<&PlacedEntity> = layout
        .entities
        .iter()
        .filter(|e| is_machine_entity(&e.name))
        .collect();

    for e in &machine_entities {
        if !checked.insert((e.x, e.y)) {
            continue;
        }
        if e.recipe.as_deref().is_some_and(|r| fluid_only.contains(r)) {
            continue;
        }

        let (mw, mh) = machine_dims(&e.name);
        let (mw, mh) = (mw as i32, mh as i32);
        let my_tiles: FxHashSet<(i32, i32)> = (0..mw)
            .flat_map(|dx| (0..mh).map(move |dy| (e.x + dx, e.y + dy)))
            .collect();

        // Helper: belt tiles adjacent to this machine's inserters of a given type
        let belt_tiles_near_inserters = |target: &FxHashSet<(i32, i32)>| -> FxHashSet<(i32, i32)> {
            let mut result = FxHashSet::default();
            for dx in -1..=mw {
                for dy in -1..=mh {
                    let ipos = (e.x + dx, e.y + dy);
                    if !target.contains(&ipos) || my_tiles.contains(&ipos) {
                        continue;
                    }
                    for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let nb = (ipos.0 + ddx, ipos.1 + ddy);
                        if belt_tiles.contains(&nb) && !my_tiles.contains(&nb) {
                            result.insert(nb);
                        }
                    }
                }
            }
            result
        };

        // --- Input path check ---
        let input_belt_starts = belt_tiles_near_inserters(&input_inserter_positions);
        if !input_belt_starts.is_empty() {
            let network = bfs_belt_reach(&input_belt_starts, &belt_tiles, Some(&ug_pairs));
            let mut reaches_source = false;
            'outer: for &(bx, by) in &network {
                for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let adj = (bx + ddx, by + ddy);
                    if inserter_positions.contains(&adj)
                        && !my_tiles.contains(&adj)
                        && !input_inserter_positions.contains(&adj)
                    {
                        reaches_source = true;
                        break 'outer;
                    }
                    if inserter_positions.contains(&adj) && !my_tiles.contains(&adj) {
                        for (ddx2, ddy2) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            let adj2 = (adj.0 + ddx2, adj.1 + ddy2);
                            if all_machine_tiles.contains(&adj2) && !my_tiles.contains(&adj2) {
                                reaches_source = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
            if !reaches_source && !network_reaches_boundary(&network) {
                issues.push(ValidationIssue::with_pos(
                    severity,
                    "belt-flow-path",
                    format!(
                        "{} at ({},{}): input belt network ({} tiles) \
                         doesn't reach any source (other machine or layout boundary)",
                        e.name,
                        e.x,
                        e.y,
                        network.len()
                    ),
                    e.x,
                    e.y,
                ));
            }
        }

        // --- Output path check ---
        let has_solid_output = solver.is_none_or(|_| {
            e.recipe
                .as_deref()
                .is_some_and(|r| solid_output_recipes.contains(r))
        });
        if !has_solid_output {
            continue;
        }
        let output_belt_starts = belt_tiles_near_inserters(&output_inserter_positions);
        if output_belt_starts.is_empty() {
            continue;
        }
        let network = bfs_belt_reach(&output_belt_starts, &belt_tiles, Some(&ug_pairs));

        let mut reaches_sink = false;
        'outer2: for &(bx, by) in &network {
            for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let adj = (bx + ddx, by + ddy);
                if input_inserter_positions.contains(&adj) && !my_tiles.contains(&adj) {
                    reaches_sink = true;
                    break 'outer2;
                }
            }
        }

        if !reaches_sink && !network_reaches_boundary(&network) {
            issues.push(ValidationIssue::with_pos(
                severity,
                "belt-flow-path",
                format!(
                    "{} at ({},{}): output belt network ({} tiles) \
                     doesn't reach any sink (other machine or layout boundary)",
                    e.name,
                    e.x,
                    e.y,
                    network.len()
                ),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 3. check_belt_network_topology
// ---------------------------------------------------------------------------

pub fn check_belt_network_topology(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let sr = match solver {
        Some(s) => s,
        None => return issues,
    };

    // Build belt tile set with carries annotation, expanding splitters
    let mut belt_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut belt_carries: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_tiles.insert((e.x, e.y));
            belt_carries.insert((e.x, e.y), e.carries.clone());
            if is_splitter(&e.name) {
                let second = splitter_second_tile(e);
                belt_tiles.insert(second);
                belt_carries.insert(second, e.carries.clone());
            }
        }
    }
    if belt_tiles.is_empty() {
        return issues;
    }

    let machine_tiles_set = build_machine_tile_set(layout);
    let machine_by_tile = build_machine_by_tile(layout);

    // Per-machine belt tiles for input/output inserters
    let mut input_inserter_belt_tiles: FxHashMap<(i32, i32), Vec<(i32, i32)>> =
        FxHashMap::default();
    let mut output_inserter_belt_tiles: FxHashMap<(i32, i32), Vec<(i32, i32)>> =
        FxHashMap::default();

    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);

        if machine_tiles_set.contains(&drop_pos) && belt_tiles.contains(&pickup_pos) {
            if let Some(&mpos) = machine_by_tile.get(&drop_pos) {
                input_inserter_belt_tiles
                    .entry(mpos)
                    .or_default()
                    .push(pickup_pos);
            }
        } else if machine_tiles_set.contains(&pickup_pos) && belt_tiles.contains(&drop_pos) {
            if let Some(&mpos) = machine_by_tile.get(&pickup_pos) {
                output_inserter_belt_tiles
                    .entry(mpos)
                    .or_default()
                    .push(drop_pos);
            }
        }
    }

    let ug_pairs = build_ug_pairs(layout);

    // Layout boundary
    let all_xs: Vec<i32> = belt_tiles.iter().map(|&(x, _)| x).collect();
    let all_ys: Vec<i32> = belt_tiles.iter().map(|&(_, y)| y).collect();
    let min_bx = *all_xs.iter().min().unwrap();
    let max_bx = *all_xs.iter().max().unwrap();
    let min_by = *all_ys.iter().min().unwrap();
    let max_by = *all_ys.iter().max().unwrap();

    let on_boundary = |(x, y): (i32, i32)| -> bool {
        x == min_bx || x == max_bx || y == min_by || y == max_by
    };

    // Group machines by recipe
    let mut recipe_machines: FxHashMap<&str, Vec<(i32, i32)>> = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            if let Some(r) = e.recipe.as_deref() {
                recipe_machines.entry(r).or_default().push((e.x, e.y));
            }
        }
    }

    let external_input_items: FxHashSet<&str> = sr
        .external_inputs
        .iter()
        .filter(|f| !f.is_fluid)
        .map(|f| f.item.as_str())
        .collect();
    let external_output_items: FxHashSet<&str> = sr
        .external_outputs
        .iter()
        .filter(|f| !f.is_fluid)
        .map(|f| f.item.as_str())
        .collect();

    // item → consumer recipes (external inputs)
    let mut item_to_consumer_recipes: FxHashMap<&str, FxHashSet<&str>> = FxHashMap::default();
    for spec in &sr.machines {
        for inp in &spec.inputs {
            if external_input_items.contains(inp.item.as_str()) && !inp.is_fluid {
                item_to_consumer_recipes
                    .entry(&inp.item)
                    .or_default()
                    .insert(&spec.recipe);
            }
        }
    }

    // item → producer recipes (external outputs)
    let mut item_to_producer_recipes: FxHashMap<&str, FxHashSet<&str>> = FxHashMap::default();
    for spec in &sr.machines {
        for out in &spec.outputs {
            if external_output_items.contains(out.item.as_str()) && !out.is_fluid {
                item_to_producer_recipes
                    .entry(&out.item)
                    .or_default()
                    .insert(&spec.recipe);
            }
        }
    }

    // Inner check function
    let mut check_network = |item: &str,
                              direction: &str,
                              belt_starts: &Vec<(i32, i32)>,
                              machine_list: &Vec<(i32, i32)>| {
        if belt_starts.is_empty() {
            return;
        }
        // Filter belt tiles to only those carrying this item
        let item_belt_tiles: FxHashSet<(i32, i32)> = belt_tiles
            .iter()
            .filter(|&&pos| belt_carries.get(&pos).and_then(|c| c.as_deref()) == Some(item))
            .copied()
            .collect();

        let starts_set: FxHashSet<(i32, i32)> = belt_starts.iter().copied().collect();
        let full_network = bfs_belt_reach(&starts_set, &item_belt_tiles, Some(&ug_pairs));

        // Check connectivity.
        //
        // One positioned issue per disconnected network, not one issue naming
        // a count. An aggregate is invisible to any consumer comparing issue
        // counts by category: `{"belt-topology": 1}` reads the same whether an
        // item's belts are in two fragments or twenty. The identical shape in
        // `check_pole_network_connectivity` let a layout transform go from 2
        // to 89 disconnected poles and pass its admission gate as "no worse
        // than source", shipping a factory that pasted as two dead halves.
        //
        // Networks are also grouped properly rather than measured from
        // `belt_starts[0]`. That element is the arbitrary first entry of a Vec
        // built off hash iteration order, so "unreachable from it" is not a
        // stable magnitude — the same flaw in `power_wires::disconnected_poles`
        // made a genuine repair (49 components down to 11) report as a
        // regression, because merging components that exclude element 0 adds
        // nodes still unreachable from element 0.
        if belt_starts.len() > 1 {
            // Partition the starts into connected components.
            let mut components: Vec<Vec<(i32, i32)>> = Vec::new();
            let mut assigned: FxHashSet<(i32, i32)> = FxHashSet::default();
            for &start in belt_starts.iter() {
                if assigned.contains(&start) {
                    continue;
                }
                let seed: FxHashSet<(i32, i32)> = std::iter::once(start).collect();
                let reach = bfs_belt_reach(&seed, &item_belt_tiles, Some(&ug_pairs));
                let mut member: Vec<(i32, i32)> = belt_starts
                    .iter()
                    .filter(|&&bt| bt == start || reach.contains(&bt))
                    .copied()
                    .collect();
                member.sort();
                for &m in &member {
                    assigned.insert(m);
                }
                components.push(member);
            }
            if components.len() > 1 {
                // Deterministic order regardless of hash iteration.
                components.sort();
                let total = components.len();
                for (n, member) in components.iter().enumerate() {
                    let anchor = member[0];
                    issues.push(ValidationIssue::with_pos(
                        Severity::Error,
                        "belt-topology",
                        format!(
                            "{item} {direction}: belt network {}/{total} is isolated \
                             ({} feed point(s), serving {} machine(s)) — should be a \
                             single connected network",
                            n + 1,
                            member.len(),
                            machine_list.len()
                        ),
                        anchor.0,
                        anchor.1,
                    ));
                }
                return;
            }
        }

        let boundary_tiles: Vec<(i32, i32)> = full_network
            .iter()
            .filter(|&&t| on_boundary(t))
            .copied()
            .collect();

        if boundary_tiles.is_empty() {
            issues.push(ValidationIssue::new(
                Severity::Error,
                "belt-topology",
                format!(
                    "{} {}: belt network ({} tiles) doesn't reach layout boundary",
                    item,
                    direction,
                    full_network.len()
                ),
            ));
            return;
        }

        // Check boundary tiles are contiguous
        let boundary_set: FxHashSet<(i32, i32)> = boundary_tiles.iter().copied().collect();
        let mut bfs_visited: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut bfs_queue: VecDeque<(i32, i32)> = VecDeque::new();
        bfs_queue.push_back(boundary_tiles[0]);
        bfs_visited.insert(boundary_tiles[0]);
        while let Some((bx, by)) = bfs_queue.pop_front() {
            for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nb = (bx + ddx, by + ddy);
                if boundary_set.contains(&nb) && bfs_visited.insert(nb) {
                    bfs_queue.push_back(nb);
                }
            }
        }
        if bfs_visited.len() < boundary_set.len() {
            issues.push(ValidationIssue::new(
                Severity::Warning,
                "belt-topology",
                format!(
                    "{} {}: belt network reaches layout boundary at multiple \
                     separate locations (ideally one contiguous entry/exit point)",
                    item, direction
                ),
            ));
        }
    };

    // Check input networks
    for (item, recipes) in &item_to_consumer_recipes {
        let mut input_belt_starts: Vec<(i32, i32)> = Vec::new();
        let mut consuming_machines: Vec<(i32, i32)> = Vec::new();
        for &recipe in recipes {
            for &mpos in recipe_machines.get(recipe).unwrap_or(&vec![]) {
                if let Some(bt_list) = input_inserter_belt_tiles.get(&mpos) {
                    let matched: Vec<(i32, i32)> = bt_list
                        .iter()
                        .filter(|&&pos| {
                            belt_carries
                                .get(&pos)
                                .and_then(|c| c.as_deref())
                                == Some(*item)
                        })
                        .copied()
                        .collect();
                    if !matched.is_empty() {
                        input_belt_starts.extend_from_slice(&matched);
                        consuming_machines.push(mpos);
                    }
                }
            }
        }
        check_network(item, "input", &input_belt_starts, &consuming_machines);
    }

    // Check output networks
    for (item, recipes) in &item_to_producer_recipes {
        let mut output_belt_starts: Vec<(i32, i32)> = Vec::new();
        let mut producing_machines: Vec<(i32, i32)> = Vec::new();
        for &recipe in recipes {
            for &mpos in recipe_machines.get(recipe).unwrap_or(&vec![]) {
                if let Some(bt_list) = output_inserter_belt_tiles.get(&mpos) {
                    let matched: Vec<(i32, i32)> = bt_list
                        .iter()
                        .filter(|&&pos| {
                            belt_carries
                                .get(&pos)
                                .and_then(|c| c.as_deref())
                                == Some(*item)
                        })
                        .copied()
                        .collect();
                    if !matched.is_empty() {
                        output_belt_starts.extend_from_slice(&matched);
                        producing_machines.push(mpos);
                    }
                }
            }
        }
        check_network(item, "output", &output_belt_starts, &producing_machines);
    }

    issues
}

// ---------------------------------------------------------------------------
// 5. check_belt_junctions
// ---------------------------------------------------------------------------

pub fn check_belt_junctions(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut belt_dir: FxHashMap<(i32, i32), EntityDirection> = FxHashMap::default();
    let mut belt_carry: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_dir.insert((e.x, e.y), e.direction);
            belt_carry.insert((e.x, e.y), e.carries.clone());
            if is_splitter(&e.name) {
                let second = splitter_second_tile(e);
                belt_dir.insert(second, e.direction);
                belt_carry.insert(second, e.carries.clone());
            }
        }
    }

    for (&(x, y), &direction) in &belt_dir {
        let (dx, dy) = dir_to_vec(direction);

        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if !belt_dir.contains_key(&(nx, ny)) {
                continue;
            }
            // Only check same-item feeders
            if belt_carry.get(&(nx, ny)) != belt_carry.get(&(x, y)) {
                continue;
            }
            let nd = belt_dir[&(nx, ny)];
            let (ndx, ndy) = dir_to_vec(nd);
            // Does this neighbour point at (x, y)?
            if (nx + ndx, ny + ndy) != (x, y) {
                continue;
            }

            let is_perpendicular = ndx * dx + ndy * dy == 0;
            let is_from_behind = ndx == dx && ndy == dy;
            if is_from_behind {
                continue;
            }
            if !is_perpendicular {
                let is_head_on = ndx == -dx && ndy == -dy;
                issues.push(ValidationIssue::with_pos(
                    if is_head_on {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                    "belt-junction",
                    if is_head_on {
                        format!("Belt at ({},{}) feeds HEAD-ON into ({},{})", nx, ny, x, y)
                    } else {
                        format!(
                            "Belt at ({},{}) feeds into ({},{}) from an invalid angle (not perpendicular)",
                            nx, ny, x, y
                        )
                    },
                    x,
                    y,
                ));
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 6. check_belt_flow_reachability
// ---------------------------------------------------------------------------

pub fn check_belt_flow_reachability(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
    style: LayoutStyle,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if solver.is_none() {
        return issues;
    }

    let fluid_only = fluid_only_recipes(solver);
    let belt_dir_map = belt_dir_map_from(&layout.entities);
    if belt_dir_map.is_empty() {
        return issues;
    }

    let ug_pairs = build_ug_pairs(layout);
    let splitter_siblings = build_splitter_siblings(layout);
    let machine_tiles_set = build_machine_tile_set(layout);
    let machine_by_tile = build_machine_by_tile(layout);

    let mut input_belt_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut output_belt_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut machine_input_belts: FxHashMap<(i32, i32), Vec<(i32, i32)>> = FxHashMap::default();
    let mut machine_output_belts: FxHashMap<(i32, i32), Vec<(i32, i32)>> = FxHashMap::default();

    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);

        if machine_tiles_set.contains(&drop_pos) && belt_dir_map.contains_key(&pickup_pos) {
            if let Some(&mpos) = machine_by_tile.get(&drop_pos) {
                input_belt_tiles.insert(pickup_pos);
                machine_input_belts.entry(mpos).or_default().push(pickup_pos);
            }
        } else if machine_tiles_set.contains(&pickup_pos) && belt_dir_map.contains_key(&drop_pos) {
            if let Some(&mpos) = machine_by_tile.get(&pickup_pos) {
                output_belt_tiles.insert(drop_pos);
                machine_output_belts.entry(mpos).or_default().push(drop_pos);
            }
        }
    }

    // Boundary from belt positions
    let all_xs: Vec<i32> = belt_dir_map.keys().map(|&(x, _)| x).collect();
    let all_ys: Vec<i32> = belt_dir_map.keys().map(|&(_, y)| y).collect();
    let min_bx = *all_xs.iter().min().unwrap();
    let max_bx = *all_xs.iter().max().unwrap();
    let min_by = *all_ys.iter().min().unwrap();
    let max_by = *all_ys.iter().max().unwrap();

    let on_boundary = |(x, y): (i32, i32)| -> bool {
        x == min_bx || x == max_bx || y == min_by || y == max_by
    };

    let severity = if style == LayoutStyle::Spaghetti {
        Severity::Error
    } else {
        Severity::Warning
    };

    // BELT-TO-BELT LIFT inserters: pick off one belt and drop onto another.
    // Invisible to the classification above, which only recognises
    // machine<->belt, and that blind spot is half of #520 — a lift's drop was
    // not counted as a source of its belt, and its own pickup was never
    // verified, so a lift feeding off permanently empty belt looked fine.
    //
    // Modelled as BOTH: its drop tile is a source (items do arrive there) and
    // its pickup tile is a sink that must itself be fed. That is what makes the
    // check transitive, and it localises the fault at the lift's pickup rather
    // than at the machine three rows downstream that starves because of it.
    let mut lift_drop_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut lift_pickup_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut lift_by_pickup: FxHashMap<(i32, i32), (String, i32, i32, Option<String>)> =
        FxHashMap::default();
    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);
        if belt_dir_map.contains_key(&drop_pos) && belt_dir_map.contains_key(&pickup_pos) {
            lift_drop_tiles.insert(drop_pos);
            lift_pickup_tiles.insert(pickup_pos);
            lift_by_pickup.insert(
                pickup_pos,
                (ins.name.clone(), ins.x, ins.y, ins.carries.clone()),
            );
        }
    }

    // DI bridge pickup tiles: a direct-insertion bridge inserter lifts the
    // coupled item off the producer's output belt (its pickup tile) and
    // carries it to the consumer — a valid sink for that belt, even though
    // the items never flow onward to a boundary or a machine's input belt.
    let di_bridge_pickup_tiles: FxHashSet<(i32, i32)> = layout
        .entities
        .iter()
        .filter(|e| is_inserter(&e.name) && super::is_di_bridge_inserter(e.segment_id.as_deref()))
        .map(|e| {
            let (dx, dy) = dir_to_vec(e.direction);
            let reach = inserter_reach(&e.name);
            (e.x - dx * reach, e.y - dy * reach)
        })
        .collect();

    // ONE forward sweep from every source, and one backward sweep from every
    // sink, instead of a BFS per machine.
    //
    // This is the other half of #520, and it is a reporting defect rather than a
    // missing rule. The check used to seed the BFS with ALL of a machine's input
    // belts at once and ask whether the UNION reached a source, so on
    // display-panel (iron-plate + electronic-circuit) the iron-plate belt's path
    // back to the furnaces satisfied the test and the electronic-circuit belt —
    // which no source fed — was never examined. A per-machine question cannot
    // distinguish "every input is fed" from "some input is fed"; the same shape
    // `validator-reporting.md` catalogues for counts that cannot tell 2 from
    // 218. Per-TILE membership answers it exactly, and costs one sweep instead
    // of one per machine.
    let mut source_tiles: FxHashSet<(i32, i32)> = belt_dir_map
        .keys()
        .copied()
        .filter(|&t| on_boundary(t))
        .collect();
    source_tiles.extend(output_belt_tiles.iter().copied());
    source_tiles.extend(lift_drop_tiles.iter().copied());
    // Seeded one step IN from each source, not with the sources themselves:
    // the old per-machine form asked about `upstream \ own_belts`, so a tile was
    // never its own supply. Seeding `bfs_belt_downstream` directly would have
    // made a boundary output tile drain into itself — caught by this check's own
    // `flow_reachability_output_dead_end_fails` unit test, which is the only
    // reason the regression did not ship.
    let mut fed_seed: FxHashSet<(i32, i32)> = FxHashSet::default();
    for &t in &source_tiles {
        fed_seed.extend(belt_step_successors(
            t,
            &belt_dir_map,
            &ug_pairs,
            &splitter_siblings,
        ));
    }
    let fed = bfs_belt_downstream(
        &fed_seed,
        &belt_dir_map,
        Some(&ug_pairs),
        Some(&splitter_siblings),
    );

    let mut sink_tiles: FxHashSet<(i32, i32)> = belt_dir_map
        .keys()
        .copied()
        .filter(|&t| on_boundary(t))
        .collect();
    sink_tiles.extend(input_belt_tiles.iter().copied());
    sink_tiles.extend(di_bridge_pickup_tiles.iter().copied());
    sink_tiles.extend(lift_pickup_tiles.iter().copied());
    let mut drain_seed: FxHashSet<(i32, i32)> = FxHashSet::default();
    for &t in &sink_tiles {
        drain_seed.extend(belt_step_predecessors(
            t,
            &belt_dir_map,
            &ug_pairs,
            &splitter_siblings,
        ));
    }
    let drains = bfs_belt_upstream(
        &drain_seed,
        &belt_dir_map,
        Some(&ug_pairs),
        Some(&splitter_siblings),
    );

    // Input check — one issue per unfed pickup TILE, positioned at the
    // consumer, so two starved inputs on one machine report as two.
    let mut machines_by_input: FxHashMap<(i32, i32), Vec<(String, i32, i32)>> =
        FxHashMap::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        if e.recipe.as_deref().is_some_and(|r| fluid_only.contains(r)) {
            continue;
        }
        if let Some(belts) = machine_input_belts.get(&(e.x, e.y)) {
            for &b in belts {
                machines_by_input
                    .entry(b)
                    .or_default()
                    .push((e.name.clone(), e.x, e.y));
            }
        }
    }
    let mut reported: FxHashSet<(i32, i32)> = FxHashSet::default();
    // A tile that is BOTH a drop and a pickup is fed by the drop, even though
    // the one-step-in seeding cannot see it: the seeding exists to stop a tile
    // supplying itself, and a drop AT the tile is a different thing from the
    // tile being its own upstream. Caught in review of this change — it is the
    // common shape, since `stamp_di_bridge`'s pickup column and a producer's own
    // output-drop column both land at `mx+1` under the default row geometry.
    //
    // Deliberately NOT `!source_tiles.contains(&t)`: `source_tiles` also holds
    // every boundary tile, and letting a pickup satisfy itself by sitting on the
    // boundary is the laxity the seeding was added to remove. Only an actual
    // drop at the tile counts.
    let mut unfed: Vec<(i32, i32)> = machines_by_input
        .keys()
        .chain(lift_pickup_tiles.iter())
        .filter(|&&t| {
            !fed.contains(&t)
                && !output_belt_tiles.contains(&t)
                && !lift_drop_tiles.contains(&t)
        })
        .copied()
        .collect();
    unfed.sort();
    for t in unfed {
        if !reported.insert(t) {
            continue;
        }
        let who = if let Some(ms) = machines_by_input.get(&t) {
            let (n, mx, my) = &ms[0];
            format!("{n} at ({mx},{my})")
        } else if let Some((n, ix, iy, carry)) = lift_by_pickup.get(&t) {
            format!(
                "belt-to-belt {n} at ({ix},{iy}){}",
                carry
                    .as_deref()
                    .map(|c| format!(" carrying {c}"))
                    .unwrap_or_default()
            )
        } else {
            "consumer".to_string()
        };
        issues.push(ValidationIssue::with_pos(
            severity,
            "belt-flow-reachability",
            format!(
                "{who}: nothing feeds its pickup belt at ({},{}) — no upstream path \
                 from the boundary, a machine's output, or another belt",
                t.0, t.1
            ),
            t.0,
            t.1,
        ));
    }

    // Output check — one issue per output TILE whose items cannot leave. Also
    // per-tile for the same reason: a machine with two output belts, one of
    // which dead-ends, used to pass on the other.
    let mut machines_by_output: FxHashMap<(i32, i32), Vec<(String, i32, i32)>> =
        FxHashMap::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        if e.recipe.as_deref().is_some_and(|r| fluid_only.contains(r)) {
            continue;
        }
        if let Some(belts) = machine_output_belts.get(&(e.x, e.y)) {
            for &b in belts {
                machines_by_output
                    .entry(b)
                    .or_default()
                    .push((e.name.clone(), e.x, e.y));
            }
        }
    }
    // Mirror of the input-side allowance: items leave a tile that something
    // picks FROM, even when that tile is the machine's own drop tile. The old
    // code encoded this only for DI bridges, by testing the INCLUSIVE
    // `downstream` set against `di_bridge_pickup_tiles` while using the strict
    // `downstream_beyond` for every other arm. That asymmetry was load-bearing
    // and undocumented; it is stated here and extended to the other pickup
    // kinds, which have the same physics.
    let mut stuck: Vec<(i32, i32)> = machines_by_output
        .keys()
        .filter(|&&t| {
            !drains.contains(&t)
                && !input_belt_tiles.contains(&t)
                && !di_bridge_pickup_tiles.contains(&t)
                && !lift_pickup_tiles.contains(&t)
        })
        .copied()
        .collect();
    stuck.sort();
    for t in stuck {
        let (n, mx, my) = &machines_by_output[&t][0];
        issues.push(ValidationIssue::with_pos(
            severity,
            "belt-flow-reachability",
            format!(
                "{n} at ({mx},{my}): items dropped on ({},{}) cannot leave \
                 (no downstream path to the boundary, a machine's input, or another belt)",
                t.0, t.1
            ),
            t.0,
            t.1,
        ));
    }

    issues
}

// ---------------------------------------------------------------------------
// 7. check_belt_throughput
// ---------------------------------------------------------------------------

pub fn check_belt_throughput(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut tile_counts: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    let mut tile_names: FxHashMap<(i32, i32), &str> = FxHashMap::default();

    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            let pos = (e.x, e.y);
            *tile_counts.entry(pos).or_insert(0) += 1;
            tile_names.insert(pos, &e.name);
        }
    }

    for (&pos, &count) in &tile_counts {
        if count > 1 {
            let belt_name = tile_names.get(&pos).copied().unwrap_or("transport-belt");
            let max_throughput = match belt_name {
                "transport-belt" | "underground-belt" => 15.0_f64,
                "fast-transport-belt" | "fast-underground-belt" => 30.0,
                "express-transport-belt" | "express-underground-belt" => 45.0,
                _ => 15.0,
            };
            issues.push(ValidationIssue::with_pos(
                Severity::Warning,
                "belt-throughput",
                format!(
                    "Belt at ({},{}): {} overlapping routes on {} (max {}/s)",
                    pos.0, pos.1, count, belt_name, max_throughput
                ),
                pos.0,
                pos.1,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 8. check_output_belt_coverage
// ---------------------------------------------------------------------------

pub fn check_output_belt_coverage(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut fluid_output_recipes: FxHashSet<String> = FxHashSet::default();
    if let Some(sr) = solver {
        for spec in &sr.machines {
            if !spec.outputs.iter().any(|f| !f.is_fluid) {
                fluid_output_recipes.insert(spec.recipe.clone());
            }
        }
    }

    let machine_tiles_set = build_machine_tile_set(layout);
    let belt_tiles = build_belt_tile_set(&layout.entities);

    let mut checked: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        if !checked.insert((e.x, e.y)) {
            continue;
        }
        if e.recipe
            .as_deref()
            .is_some_and(|r| fluid_output_recipes.contains(r))
        {
            continue;
        }

        let (mw, mh) = machine_dims(&e.name);
        let (mw, mh) = (mw as i32, mh as i32);
        let my_tiles: FxHashSet<(i32, i32)> = (0..mw)
            .flat_map(|dx| (0..mh).map(move |dy| (e.x + dx, e.y + dy)))
            .collect();

        let mut has_output_belt = false;
        'outer: for ins in &layout.entities {
            if !is_inserter(&ins.name) {
                continue;
            }
            let (dx, dy) = dir_to_vec(ins.direction);
            let reach = inserter_reach(&ins.name);
            let (odx, ody) = (-dx, -dy);
            let pickup_pos = (ins.x + odx * reach, ins.y + ody * reach);
            let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);

            if my_tiles.contains(&pickup_pos)
                && !machine_tiles_set.contains(&drop_pos)
                && belt_tiles.contains(&drop_pos)
            {
                has_output_belt = true;
                break 'outer;
            }
        }

        // RFC-053: a DI-cell producer's output leaves by inserter into the
        // consumer machine, never onto a belt. See `is_di_cell_entity`.
        // Both ends are tested. Picking from this machine is not enough:
        // the cell tags EVERY entity it stamps, including the consumer's
        // own output inserter, which also picks from inside its machine —
        // but drops onto a real belt and must stay under this check.
        // Requiring the drop tile to be a machine too is what distinguishes
        // a coupler from an ordinary output inserter living in a cell.
        let served_by_di_cell = layout.entities.iter().any(|ins| {
            is_inserter(&ins.name)
                && super::is_di_cell_entity(ins.segment_id.as_deref())
                && {
                    let (dx, dy) = dir_to_vec(ins.direction);
                    let reach = inserter_reach(&ins.name);
                    let pick = (ins.x - dx * reach, ins.y - dy * reach);
                    let drop = (ins.x + dx * reach, ins.y + dy * reach);
                    my_tiles.contains(&pick) && machine_tiles_set.contains(&drop)
                }
        });

        if !has_output_belt && !served_by_di_cell {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "output-belt",
                format!(
                    "{} at ({},{}): no output inserter has a belt at its drop position",
                    e.name, e.x, e.y
                ),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 9. check_underground_belt_pairs
// ---------------------------------------------------------------------------

pub fn check_underground_belt_pairs(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut ug_inputs: Vec<&PlacedEntity> = Vec::new();
    let mut ug_outputs: Vec<&PlacedEntity> = Vec::new();
    let mut all_ug: Vec<&PlacedEntity> = Vec::new();
    for e in &layout.entities {
        if is_ug_belt(&e.name) {
            all_ug.push(e);
            match e.io_type.as_deref() {
                Some("input") => ug_inputs.push(e),
                Some("output") => ug_outputs.push(e),
                _ => {}
            }
        }
    }

    // RFC-065 Phase 1: the pairing itself comes from the canonical
    // derivation — this check's previous inline loop was byte-equivalent
    // (same-name + same-direction + nearest ahead + dist > 1, greedy in
    // entity order) and is deleted rather than kept as a fourth copy. This
    // check keeps what it alone owns: reach, interception, and orphan
    // REPORTING over those pairs.
    let pairs = crate::connectivity::build_ug_pairs(&layout.entities);

    for inp in &ug_inputs {
        let (dx, dy) = dir_to_vec(inp.direction);
        let surface_tier = ug_to_surface_tier(&inp.name);
        let max_reach = ug_max_reach(surface_tier) as i32;

        let Some(&(out_x, out_y)) = pairs.get(&(inp.x, inp.y)) else {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "underground-belt",
                format!(
                    "Unpaired underground belt input at ({},{}) facing {:?}: no matching output found",
                    inp.x, inp.y, inp.direction
                ),
                inp.x,
                inp.y,
            ));
            continue;
        };
        // Pairs are axis-aligned, so the axis delta is the pair distance
        // the old loop tracked as `best_dist`.
        let best_dist = (out_x - inp.x).abs() + (out_y - inp.y).abs();

        if best_dist > max_reach + 1 {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "underground-belt",
                format!(
                    "Underground belt pair ({},{})->({},{}) distance {} exceeds max reach {} for {}",
                    inp.x, inp.y, out_x, out_y, best_dist, max_reach, surface_tier
                ),
                inp.x,
                inp.y,
            ));
        }

        // Check for intercepting UG belts
        for ug in &all_ug {
            if (ug.x, ug.y) == (inp.x, inp.y) || (ug.x, ug.y) == (out_x, out_y) {
                continue;
            }
            if ug.name != inp.name || ug.direction != inp.direction {
                continue;
            }
            let rx = ug.x - inp.x;
            let ry = ug.y - inp.y;
            let udist = if dx != 0 {
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
            if udist > 0 && udist < best_dist {
                issues.push(ValidationIssue::with_pos(
                    Severity::Warning,
                    "underground-belt",
                    format!(
                        "Underground belt at ({},{}) intercepts pair ({},{})->({},{})",
                        ug.x, ug.y, inp.x, inp.y, out_x, out_y
                    ),
                    ug.x,
                    ug.y,
                ));
            }
        }
    }

    // Unpaired outputs: the bidirectional pair map contains an output's
    // tile iff some input claimed it.
    for out in &ug_outputs {
        if !pairs.contains_key(&(out.x, out.y)) {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "underground-belt",
                format!(
                    "Unpaired underground belt output at ({},{}) facing {:?}: no matching input found",
                    out.x, out.y, out.direction
                ),
                out.x,
                out.y,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 10. check_underground_belt_sideloading
// ---------------------------------------------------------------------------

pub fn check_underground_belt_sideloading(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut belt_dir: FxHashMap<(i32, i32), EntityDirection> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_dir.insert((e.x, e.y), e.direction);
            if is_splitter(&e.name) {
                belt_dir.insert(splitter_second_tile(e), e.direction);
            }
        }
    }

    for e in &layout.entities {
        if !is_ug_belt(&e.name) || e.io_type.as_deref() != Some("output") {
            continue;
        }
        let (dx, dy) = dir_to_vec(e.direction);
        let exit_tile = (e.x + dx, e.y + dy);
        if let Some(&target_dir) = belt_dir.get(&exit_tile) {
            let (tdx, tdy) = dir_to_vec(target_dir);
            let dot = dx * tdx + dy * tdy;
            if dot < 0 {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "underground-belt",
                    format!(
                        "Underground belt exit at ({},{}) facing {:?} collides head-on with belt at ({},{}) facing {:?}",
                        e.x, e.y, e.direction, exit_tile.0, exit_tile.1, target_dir
                    ),
                    e.x,
                    e.y,
                ));
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 11. check_underground_belt_entry_sideload
// ---------------------------------------------------------------------------

pub fn check_underground_belt_entry_sideload(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut belt_dir: FxHashMap<(i32, i32), EntityDirection> = FxHashMap::default();
    // Segment id per tile, to recognize INTRA-BALANCER wiring (RFC-061
    // Phase 1.5): SAT-baked library templates are lane-verified by the
    // bake pipeline (rfc-balancer-bake-lane-validation), and their
    // internals legitimately use single-lane tricks this check exists to
    // forbid in ROUTED belts — the (6,6) template's internal splitter
    // half feeds a west UG entrance from the north by design. Exempt a
    // sideload only when BOTH tiles carry the SAME `balancer:` segment;
    // a routed belt entering a balancer from the side still warns.
    let mut seg_by_tile: FxHashMap<(i32, i32), &str> = FxHashMap::default();
    let mut ug_inputs: Vec<&PlacedEntity> = Vec::new();

    for e in &layout.entities {
        if is_surface_belt(&e.name) || is_splitter(&e.name) {
            belt_dir.insert((e.x, e.y), e.direction);
            if let Some(seg) = e.segment_id.as_deref() {
                seg_by_tile.insert((e.x, e.y), seg);
            }
            if is_splitter(&e.name) {
                belt_dir.insert(splitter_second_tile(e), e.direction);
                if let Some(seg) = e.segment_id.as_deref() {
                    seg_by_tile.insert(splitter_second_tile(e), seg);
                }
            }
        } else if is_ug_belt(&e.name) {
            match e.io_type.as_deref() {
                Some("output") => {
                    belt_dir.insert((e.x, e.y), e.direction);
                }
                Some("input") => ug_inputs.push(e),
                _ => {}
            }
        }
    }

    let same_balancer = |a: (i32, i32), b: Option<&str>| -> bool {
        match (seg_by_tile.get(&a), b) {
            (Some(sa), Some(sb)) => sa.starts_with("balancer:") && *sa == sb,
            _ => false,
        }
    };

    for ug in &ug_inputs {
        let (ug_dx, ug_dy) = dir_to_vec(ug.direction);
        for (ndx, ndy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = (ug.x + ndx, ug.y + ndy);
            if same_balancer((nx, ny), ug.segment_id.as_deref()) {
                continue;
            }
            if let Some(&n_dir) = belt_dir.get(&(nx, ny)) {
                let (n_dx, n_dy) = dir_to_vec(n_dir);
                if (nx + n_dx, ny + n_dy) != (ug.x, ug.y) {
                    continue;
                }
                let dot = n_dx * ug_dx + n_dy * ug_dy;
                if dot == 0 {
                    issues.push(ValidationIssue::with_pos(
                        Severity::Warning,
                        "underground-belt",
                        format!(
                            "Belt at ({},{}) facing {:?} sideloads into underground input at ({},{}) facing {:?} \
                             — only one lane loaded, must feed UG inputs straight",
                            nx, ny, n_dir, ug.x, ug.y, ug.direction
                        ),
                        ug.x,
                        ug.y,
                    ));
                }
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 15. check_belt_inserter_conflict
// ---------------------------------------------------------------------------

pub fn check_belt_inserter_conflict(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let belt_tiles = build_belt_tile_set(&layout.entities);
    let mut drop_map: FxHashMap<(i32, i32), Vec<String>> = FxHashMap::default();

    for e in &layout.entities {
        if !is_inserter(&e.name) {
            continue;
        }
        let carries = match &e.carries {
            Some(c) => c.clone(),
            None => continue,
        };
        let (dx, dy) = dir_to_vec(e.direction);
        let reach = inserter_reach(&e.name);
        let drop = (e.x + dx * reach, e.y + dy * reach);
        if belt_tiles.contains(&drop) {
            drop_map.entry(drop).or_default().push(carries);
        }
    }

    for (&(bx, by), items) in &drop_map {
        let unique: FxHashSet<&str> = items.iter().map(|s| s.as_str()).collect();
        if unique.len() >= 2 {
            let mut sorted: Vec<&str> = unique.into_iter().collect();
            sorted.sort();
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "belt-item-isolation",
                format!(
                    "Belt at ({},{}): inserters drop conflicting items {:?} and {:?}",
                    bx, by, sorted[0], sorted[1]
                ),
                bx,
                by,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// 16. compute_lane_rates + check_lane_throughput
// ---------------------------------------------------------------------------

/// Compute the pair of per-lane output rates `(pos_out, sib_out)` for a
/// splitter's two output tiles, given the accumulated per-lane input rates
/// at each tile (`pos_rates`, `sib_rates`).
///
/// Used by the topo-sort and cycle-breaker phases of
/// [`compute_lane_rates_impl`], which preserve the left/right lane
/// identity through a splitter (only the sibling total is split 50/50).
/// See the `_mixed` sibling below for the full-lane-mixing model used by
/// this same function's iterate-to-convergence phase.
///
/// Default behavior: the combined left-lane total is split 50/50 across
/// both output tiles, and likewise for the right lane (both tiles end up
/// with the same `(left, right)` tuple). When `loop_priority_rate` is
/// `Some(cap)` and exactly one of `pos_is_loop_branch` /
/// `sib_is_loop_branch` is `true` (the output tile feeding a tagged
/// priority-branch segment — a self-loop recirculation or a merge-and-tap
/// consumer tap), the priority branch instead receives `min(total, cap)`
/// and the other branch the remainder, preserving the input left/right
/// ratio within each branch. Falls back to the symmetric split if
/// `loop_priority_rate` is `None`, or the priority branch is ambiguous
/// (neither or both flagged) — see docs/rfc-solver-net-flow.md Phase 2(c)
/// and docs/rfc-merge-tap-trunks.md D4.
fn splitter_output_rates(
    pos_rates: (f64, f64),
    sib_rates: (f64, f64),
    loop_priority_rate: Option<f64>,
    pos_is_loop_branch: bool,
    sib_is_loop_branch: bool,
) -> ((f64, f64), (f64, f64)) {
    if let Some(cap) = loop_priority_rate {
        if pos_is_loop_branch != sib_is_loop_branch {
            let total_left = pos_rates.0 + sib_rates.0;
            let total_right = pos_rates.1 + sib_rates.1;
            let total = total_left + total_right;
            let loop_share = total.min(cap.max(0.0));
            let export_share = (total - loop_share).max(0.0);
            let (loop_ratio, export_ratio) = if total > f64::EPSILON {
                (loop_share / total, export_share / total)
            } else {
                (0.0, 0.0)
            };
            let loop_out = (total_left * loop_ratio, total_right * loop_ratio);
            let export_out = (total_left * export_ratio, total_right * export_ratio);
            return if pos_is_loop_branch {
                (loop_out, export_out)
            } else {
                (export_out, loop_out)
            };
        }
    }
    let half_left = (pos_rates.0 + sib_rates.0) / 2.0;
    let half_right = (pos_rates.1 + sib_rates.1) / 2.0;
    ((half_left, half_right), (half_left, half_right))
}

#[allow(clippy::too_many_arguments)]
/// Convergence-phase splitter model (RFC-047 Phase 0, Leg A).
///
/// LANE-PRESERVING: real splitters never mix lanes (mechanics rule S4)
/// — left-lane items stay left across whichever output they reach. The
/// predecessor (`splitter_output_rates_mixed`) pooled both lanes into
/// one scalar and re-split evenly, silently acting as a free lane
/// rebalancer at every splitter and masking genuine lane starvation
/// downstream (RFC-047 ground truth 6b). Now each lane's pooled total
/// is demand-allocated independently. The priority-loop branch computes
/// its loop/export ratio from the two-lane total (the loop cap is a
/// branch-level demand ceiling, not per-lane physics) and applies it to
/// each lane's own pool. Known simplification, recorded in the RFC's
/// decision log: no per-lane demand signal exists yet, so both lanes
/// share the same `demand_a/demand_b` ratio.
fn splitter_output_rates_convergence(
    a_rates: (f64, f64),
    b_rates: (f64, f64),
    loop_priority_rate: Option<f64>,
    a_is_loop_branch: bool,
    b_is_loop_branch: bool,
    demand_a: f64,
    demand_b: f64,
    cap: f64,
) -> ([f64; 2], [f64; 2]) {
    if let Some(loop_cap) = loop_priority_rate {
        if a_is_loop_branch != b_is_loop_branch {
            let total_left = a_rates.0 + b_rates.0;
            let total_right = a_rates.1 + b_rates.1;
            let total = total_left + total_right;
            let loop_share = total.min(loop_cap.max(0.0));
            let export_share = (total - loop_share).max(0.0);
            let (loop_ratio, export_ratio) = if total > f64::EPSILON {
                (loop_share / total, export_share / total)
            } else {
                (0.0, 0.0)
            };
            let loop_out = [total_left * loop_ratio, total_right * loop_ratio];
            let export_out = [total_left * export_ratio, total_right * export_ratio];
            return if a_is_loop_branch {
                (loop_out, export_out)
            } else {
                (export_out, loop_out)
            };
        }
    }
    let lane_cap = cap / 2.0;
    let (left_a, left_b) =
        allocate_by_demand(a_rates.0 + b_rates.0, demand_a, demand_b, lane_cap);
    let (right_a, right_b) =
        allocate_by_demand(a_rates.1 + b_rates.1, demand_a, demand_b, lane_cap);
    ([left_a, right_a], [left_b, right_b])
}

/// Allocate a splitter's total throughput `total` between its two output
/// tiles by downstream demand (RFC `rfc-lane-demand-flow.md` Phase 1 Branch
/// A). Returns `(out_a, out_b)` with `out_a + out_b == total` except when
/// the input genuinely exceeds `2 × cap` (over-capacity, surfaced by the
/// lane-throughput check).
///
/// Real splitters redistribute under backpressure: an output whose consumer
/// draws faster keeps pulling while a backed-up output spills to the other.
/// This models the steady state by splitting `total` **in proportion to
/// downstream demand** — when supply meets aggregate demand
/// (`total == demand_a + demand_b`) each output receives exactly its demand;
/// on undersupply both starve proportionally (so a truly under-fed consumer
/// still surfaces as a shortfall); on oversupply both scale up together, then
/// each is clamped to belt capacity `cap` with the overflow spilled to the
/// other output. A symmetric or absent demand signal (`demand_a ≈ demand_b`,
/// or both zero) is an exact even split, byte-identical to the legacy 50/50
/// model.
///
/// The allocation is deliberately **smooth in `total`** (a single linear
/// ramp, not the piecewise meet-demand/spill split an earlier draft used):
/// the demands are static, but `total` oscillates across iterations inside
/// balancer feedback loops, and a kink at `total == demand_sum` there turns
/// the forward fixed point into a limit cycle that never converges (observed
/// on processing-unit@2/s — RFC kill-criterion-2 probe). Proportional
/// splitting keeps the per-iteration map non-expansive, so the loop
/// converges at the same rate as the legacy even-split model.
fn allocate_by_demand(total: f64, demand_a: f64, demand_b: f64, cap: f64) -> (f64, f64) {
    const DEMAND_EPS: f64 = 1e-6;
    let demand_sum = demand_a + demand_b;
    if demand_sum <= DEMAND_EPS || (demand_a - demand_b).abs() <= DEMAND_EPS {
        let half = total / 2.0;
        return (half, half);
    }
    // Proportional to demand (continuous in `total`).
    let mut a = total * demand_a / demand_sum;
    let mut b = total - a;
    // Clamp each output to belt capacity, spilling the overflow to the other
    // (whose consumer can still draw it). If both would exceed `cap` the input
    // is over 2× belt capacity — a real over-capacity the lane-throughput
    // check surfaces; here we just clamp and leave the impossible surplus off.
    if a > cap {
        b += a - cap;
        a = cap;
    }
    if b > cap {
        a = (a + (b - cap)).min(cap);
        b = cap;
    }
    (a, b)
}

/// Backward demand propagation for the lane-rate walker (RFC
/// `rfc-lane-demand-flow.md` Phase 1). Returns, per belt tile, the total
/// downstream machine-input demand reachable by flowing forward from that
/// tile — the weight [`allocate_by_demand`] uses to route splitter output.
///
/// `base_demand` seeds each machine-input-inserter pickup tile with its
/// share of the (utilization-scaled) required rate. Demand flows upstream
/// over the reverse of the forward feeder graph (`demand_feeders`, which
/// mirrors `feeders` plus the underground `behind → ug-output` edge).
/// Splitters **pool**: the demand at a pair's two output tiles is summed
/// and distributed across all the pair's input feeders, so a multi-stage
/// balancer routes correctly at every stage, not just the last.
///
/// Ordering is a reverse-topological Gauss-Seidel sweep (consumers before
/// feeders) so a straight run's demand propagates its full length in a
/// single sweep instead of one tile per iteration — the same reason the
/// forward pass primes with a Kahn sort before iterating. Belt cycles
/// (balancer feedback) that the ordering can't place are appended and
/// resolved by repeating the sweep up to `budget` times; their demand is
/// symmetric anyway, so it converges fast and only ever feeds the even
/// fallback. Returns `(demand, sweeps_used)`.
fn compute_demand(
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    feeders: &FxHashMap<(i32, i32), Vec<((i32, i32), u8)>>,
    splitter_sibling: &FxHashMap<(i32, i32), (i32, i32)>,
    ug_output_tiles: &FxHashSet<(i32, i32)>,
    ug_output_to_input: &FxHashMap<(i32, i32), (i32, i32)>,
    ug_input_dir: &FxHashMap<(i32, i32), EntityDirection>,
    base_demand: &FxHashMap<(i32, i32), f64>,
    budget: usize,
) -> (FxHashMap<(i32, i32), f64>, usize) {
    // Upstream feeders per tile (positions only) plus the UG tunnel edge.
    let mut demand_feeders: FxHashMap<(i32, i32), Vec<(i32, i32)>> = FxHashMap::default();
    for (&v, fs) in feeders {
        demand_feeders.insert(v, fs.iter().map(|&(fp, _)| fp).collect());
    }
    for &ug_out in ug_output_tiles {
        if let Some(&pin) = ug_output_to_input.get(&ug_out) {
            if let Some(&idir) = ug_input_dir.get(&pin) {
                let (idx, idy) = dir_to_vec(idir);
                let behind = (pin.0 - idx, pin.1 - idy);
                if belt_dir_map.contains_key(&behind) {
                    demand_feeders.entry(ug_out).or_default().push(behind);
                }
            }
        }
    }

    let feeder_count: FxHashMap<(i32, i32), usize> = belt_dir_map
        .keys()
        .map(|&t| (t, demand_feeders.get(&t).map_or(0, |f| f.len())))
        .collect();
    // consumers[u] = tiles u feeds (inverse of demand_feeders).
    let mut consumers: FxHashMap<(i32, i32), Vec<(i32, i32)>> = FxHashMap::default();
    for (&v, fs) in &demand_feeders {
        for &fp in fs {
            consumers.entry(fp).or_default().push(v);
        }
    }

    // Reverse-topological order (consumers before feeders) via Kahn on the
    // reverse graph. Tiles left in belt cycles are appended afterwards.
    let mut out_degree: FxHashMap<(i32, i32), usize> = belt_dir_map
        .keys()
        .map(|&t| (t, consumers.get(&t).map_or(0, |c| c.len())))
        .collect();
    let mut order: Vec<(i32, i32)> = Vec::with_capacity(belt_dir_map.len());
    let mut placed: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut q: VecDeque<(i32, i32)> = out_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&t, _)| t)
        .collect();
    while let Some(u) = q.pop_front() {
        if !placed.insert(u) {
            continue;
        }
        order.push(u);
        if let Some(fs) = demand_feeders.get(&u) {
            for &fp in fs {
                if let Some(d) = out_degree.get_mut(&fp) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        q.push_back(fp);
                    }
                }
            }
        }
    }
    for &t in belt_dir_map.keys() {
        if placed.insert(t) {
            order.push(t);
        }
    }

    let pull_from = |v: (i32, i32), demand: &FxHashMap<(i32, i32), f64>| -> f64 {
        if let Some(&sib) = splitter_sibling.get(&v) {
            // Splitter output: pool the pair's demand across all its feeders.
            let pooled =
                demand.get(&v).copied().unwrap_or(0.0) + demand.get(&sib).copied().unwrap_or(0.0);
            let cnt = feeder_count.get(&v).copied().unwrap_or(0)
                + feeder_count.get(&sib).copied().unwrap_or(0);
            if cnt > 0 {
                pooled / cnt as f64
            } else {
                0.0
            }
        } else {
            let cnt = feeder_count.get(&v).copied().unwrap_or(0);
            if cnt > 0 {
                demand.get(&v).copied().unwrap_or(0.0) / cnt as f64
            } else {
                0.0
            }
        }
    };

    const EPS: f64 = 1e-5;
    let mut demand: FxHashMap<(i32, i32), f64> =
        belt_dir_map.keys().map(|&t| (t, 0.0)).collect();
    let mut sweeps = 0usize;
    for _ in 0..budget.max(1) {
        sweeps += 1;
        let mut max_change: f64 = 0.0;
        // Gauss-Seidel: update in place in reverse-topo order so acyclic
        // demand settles in one sweep.
        for &u in &order {
            let mut val = base_demand.get(&u).copied().unwrap_or(0.0);
            if let Some(cons) = consumers.get(&u) {
                for &v in cons {
                    val += pull_from(v, &demand);
                }
            }
            let prev = demand.insert(u, val).unwrap_or(0.0);
            let change = (val - prev).abs();
            if change > max_change {
                max_change = change;
            }
        }
        if max_change < EPS {
            break;
        }
    }
    (demand, sweeps)
}

pub fn compute_lane_rates(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> FxHashMap<(i32, i32), [f64; 2]> {
    compute_lane_rates_impl(layout, solver)
}

pub fn check_lane_throughput(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    check_lane_throughput_with(layout, solver, &compute_lane_rates_impl(layout, solver))
}

/// [`check_lane_throughput`] against a caller-supplied rate map — the
/// dispatch computes the walker ONCE per `validate()` and shares it with
/// [`check_input_rate_delivery_with`] (#632 B5 step 3; the walk is the
/// most expensive part of both checks).
pub fn check_lane_throughput_with(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
    lane_rates: &FxHashMap<(i32, i32), [f64; 2]>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if lane_rates.is_empty() {
        return issues;
    }
    // Non-empty lane_rates implies `solver` was Some (the impl returns
    // empty without it).
    let Some(sr) = solver else { return issues };

    // RFC-046: caps are per-ITEM stacking-aware, not blanket ×S — the
    // validator re-derives the family exemption independently instead of
    // trusting the planner's discipline (code-review finding, 2026-07-21).
    let stacking_ctx = crate::bus::stacking_ctx::StackingCtx::derive(sr, layout.stacking);
    // NOTE: splitter tiles + BOTH UG halves are covered here — gaps the
    // deleted belt_structural twin had (#632 B5; deleted 2026-08-15).
    let mut belt_name_map: FxHashMap<(i32, i32), &str> = FxHashMap::default();
    let mut carries_map: FxHashMap<(i32, i32), &str> = FxHashMap::default();
    for e in &layout.entities {
        if is_surface_belt(&e.name) {
            belt_name_map.insert((e.x, e.y), &e.name);
        } else if is_ug_belt(&e.name) {
            // BOTH halves of a UG pair, not just the output: UG-in tiles
            // carry rates via the walker's deliberate fix-up (inserter
            // pickups resolve there), and rating them at the yellow
            // fallback flagged fast UG entries lawfully carrying their
            // own tier's flow — the stacking family's "30.0/s exceeds
            // transport-belt 15/s" false positives (#632 B5; 30.0 is
            // exactly AT the fast S=2 per-lane cap).
            belt_name_map.insert((e.x, e.y), ug_to_surface_tier(&e.name));
        } else if is_splitter(&e.name) {
            // Both splitter tiles rate at the splitter's own tier. This
            // arm was MISSING here while present in belt_structural's
            // twin (#632 B5): splitter tiles fell through to the
            // "transport-belt" default below, so a fast/express
            // splitter lawfully carrying more than 7.5/s per lane was
            // falsely flagged over-cap — the class that refused the
            // sim-measured 1.10/s big-electric-pole layout (its
            // iron-plate 1x2 balancer runs 12.2-12.8/s per lane
            // through a FAST splitter) and would have shipped the
            // 0.51/s twin on dispatch swap.
            belt_name_map.insert((e.x, e.y), splitter_to_surface_tier(&e.name));
            belt_name_map.insert(splitter_second_tile(e), splitter_to_surface_tier(&e.name));
            if let Some(item) = e.carries.as_deref() {
                carries_map.insert(splitter_second_tile(e), item);
            }
        } else {
            continue;
        }
        if let Some(item) = e.carries.as_deref() {
            carries_map.insert((e.x, e.y), item);
        }
    }

    for (&pos, &[left, right]) in lane_rates {
        let belt_name = belt_name_map.get(&pos).copied().unwrap_or("transport-belt");
        // Tiles without a `carries` attribution fall back to the layout-
        // wide value (pre-review behavior); engine-stamped belts all carry.
        let tile_stacking = carries_map
            .get(&pos)
            .map(|item| stacking_ctx.for_item(item))
            .unwrap_or(layout.stacking);
        let cap = lane_capacity_stacked(belt_name, tile_stacking);
        for (lane_name, rate) in [("left", left), ("right", right)] {
            if rate > cap + 0.01 {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "lane-throughput",
                    format!(
                        "Belt at ({},{}): {} lane {:.1}/s exceeds {} per-lane capacity {}/s",
                        pos.0, pos.1, lane_name, rate, belt_name, cap
                    ),
                    pos.0,
                    pos.1,
                ));
            }
        }
    }

    issues
}

fn compute_lane_rates_impl(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> FxHashMap<(i32, i32), [f64; 2]> {
    let sr = match solver {
        Some(s) => s,
        None => return FxHashMap::default(),
    };
    // RFC-046: item-effective stacking for exemption-aware splitter caps.
    let rates_stacking_ctx = crate::bus::stacking_ctx::StackingCtx::derive(sr, layout.stacking);

    let mut belt_dir_map: FxHashMap<(i32, i32), EntityDirection> = FxHashMap::default();
    let mut ug_output_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut ug_output_to_input: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    let mut ug_input_dir: FxHashMap<(i32, i32), EntityDirection> = FxHashMap::default();

    for e in &layout.entities {
        if is_surface_belt(&e.name) {
            belt_dir_map.insert((e.x, e.y), e.direction);
        } else if is_ug_belt(&e.name) {
            match e.io_type.as_deref() {
                Some("output") => {
                    belt_dir_map.insert((e.x, e.y), e.direction);
                    ug_output_tiles.insert((e.x, e.y));
                }
                Some("input") => {
                    ug_input_dir.insert((e.x, e.y), e.direction);
                }
                _ => {}
            }
        } else if is_splitter(&e.name) {
            belt_dir_map.insert((e.x, e.y), e.direction);
            let second = splitter_second_tile(e);
            belt_dir_map.insert(second, e.direction);
        }
    }
    if belt_dir_map.is_empty() {
        return FxHashMap::default();
    }

    let ug_pairs = build_ug_pairs(layout);
    for (&(ix, iy), &(ox, oy)) in &ug_pairs {
        if ug_input_dir.contains_key(&(ix, iy)) {
            ug_output_to_input.insert((ox, oy), (ix, iy));
        }
    }

    let machine_tiles_set = build_machine_tile_set(layout);
    let machine_by_tile = build_machine_by_tile(layout);

    let mut belt_carries: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_carries.insert((e.x, e.y), e.carries.clone());
            if is_splitter(&e.name) {
                belt_carries.insert(splitter_second_tile(e), e.carries.clone());
            }
        }
    }

    let recipe_to_spec: FxHashMap<&str, &crate::models::MachineSpec> = sr
        .machines
        .iter()
        .map(|s| (s.recipe.as_str(), s))
        .collect();
    let mut machine_entity: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    // Physical machine positions per recipe, for `physical_utilization` —
    // but ONLY for recipes owned by a single MachineSpec. Voider/self-loop
    // siblings share a recipe name across specs (recipe_to_spec collapses
    // them last-wins), and pooling every physical machine against one
    // sibling's count wrecks the ratio (uranium fixtures regressed until
    // this guard).
    let mut spec_count_by_recipe: FxHashMap<&str, usize> = FxHashMap::default();
    for spec in &sr.machines {
        *spec_count_by_recipe.entry(spec.recipe.as_str()).or_insert(0) += 1;
    }
    let mut machine_ys_by_recipe: FxHashMap<&str, Vec<i32>> = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            machine_entity.insert((e.x, e.y), e);
            if let Some(r) = e.recipe.as_deref() {
                if spec_count_by_recipe.get(r).copied().unwrap_or(0) == 1 {
                    machine_ys_by_recipe.entry(r).or_default().push(e.y);
                }
            }
        }
    }
    // Recipes whose effective_rows bands carry REAL per-row counts (they
    // sum to the recipe total, within rounding) — see
    // `physical_utilization`'s scope rule.
    let per_row_count_recipes: FxHashSet<&str> = {
        let mut band_count_sum: FxHashMap<&str, f64> = FxHashMap::default();
        for row in &layout.effective_rows {
            *band_count_sum.entry(row.spec.recipe.as_str()).or_insert(0.0) += row.spec.count;
        }
        band_count_sum
            .into_iter()
            .filter(|(r, sum)| {
                recipe_to_spec
                    .get(*r)
                    .is_some_and(|s| (s.count - sum).abs() < 1e-6)
            })
            .map(|(r, _)| r)
            .collect()
    };

    // Pass 1: qualifying output inserters, keyed by (machine origin, item).
    // A machine's production is SHARED by all its output inserters — the
    // sizing ladder adds a second hand when one can't keep up (common at
    // low capacity research / high quality). Injecting the full per-machine
    // rate once per inserter double-counts such machines: #404's "bridged
    // row" false positives were really a 2-inserter machine seeding 2×6.5/s
    // onto a 13/s row (the bridge model itself was and is correct). Collect
    // first, then inject rate / n_inserters.
    let mut qualifying: Vec<((i32, i32), &str, ((i32, i32), &str), f64)> = Vec::new();
    let mut inserters_per_output: FxHashMap<((i32, i32), &str), u32> = FxHashMap::default();
    let mut lane_injections: FxHashMap<(i32, i32), [f64; 2]> = FxHashMap::default();
    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);
        if !machine_tiles_set.contains(&pickup_pos) || !belt_dir_map.contains_key(&drop_pos) {
            continue;
        }
        let mpos = match machine_by_tile.get(&pickup_pos) {
            Some(&p) => p,
            None => continue,
        };
        let me = match machine_entity.get(&mpos) {
            Some(e) => e,
            None => continue,
        };
        let recipe = match me.recipe.as_deref() {
            Some(r) => r,
            None => continue,
        };
        let fallback_spec = match recipe_to_spec.get(recipe) {
            Some(s) => *s,
            None => continue,
        };
        // Position-resolved via `effective_rows` — see
        // `super::resolve_row_spec`'s doc comment for the
        // partition-sibling rationale (`docs/rfc-inserter-sizing.md`
        // Phase 1 finding).
        let (spec, band) = super::resolve_row_spec_banded(layout, recipe, me.y, fallback_spec);
        let carried_item = match belt_carries.get(&drop_pos).and_then(|c| c.as_deref()) {
            Some(i) => i,
            None => continue,
        };
        // spec.outputs[].rate is the per-machine output rate at full
        // utilization. The layout places ceil(spec.count) physical machines,
        // each running at spec.count / ceil(spec.count) utilization — scale
        // the injected rate the same way the input-rate-delivery check
        // scales demand, or a fast machine at fractional count overstates
        // the lane rate (e.g. a 0.06-count foundry pressing transport-belt
        // at 16/s nominal seeds 16/s onto a lane that actually carries 1/s).
        let utilization = physical_utilization(spec, fallback_spec, band, &machine_ys_by_recipe, &per_row_count_recipes);
        let rate = spec
            .outputs
            .iter()
            .find(|o| o.item == carried_item)
            .map(|o| o.rate * utilization)
            .unwrap_or(0.0);
        if rate <= 0.0 {
            continue;
        }
        let belt_d = belt_dir_map[&drop_pos];
        let lane = inserter_target_lane(ins.x, ins.y, drop_pos.0, drop_pos.1, belt_d);
        let key = (mpos, carried_item);
        *inserters_per_output.entry(key).or_insert(0) += 1;
        qualifying.push((drop_pos, lane, key, rate));
    }
    // Pass 2: each machine's per-item output rate is split evenly across
    // its qualifying output inserters (identical hands drain a shared
    // buffer — the even split is the steady state the planner sizes for).
    // ASSUMPTION GUARD (#414 review): relies on the sizer emitting ONE
    // uniform plan per side and output templates confining a machine's
    // hands to one belt run; if the placer ever mixes hand tiers per
    // (machine, item) or fans hands across separate runs, revisit —
    // even-split would then under-state the faster hand's lane.
    for (drop_pos, lane, key, rate) in qualifying {
        let n = inserters_per_output[&key] as f64;
        let entry = lane_injections.entry(drop_pos).or_insert([0.0, 0.0]);
        if lane == LANE_LEFT {
            entry[0] += rate / n;
        } else {
            entry[1] += rate / n;
        }
    }

    // Build feeder map
    let mut feeders: FxHashMap<(i32, i32), Vec<((i32, i32), u8)>> = FxHashMap::default();
    for (&(bx, by), &belt_d) in &belt_dir_map {
        if ug_output_tiles.contains(&(bx, by)) {
            continue;
        }
        let (left_dx, left_dy) = { let (bdx, bdy) = dir_to_vec(belt_d); (-bdy, bdx) };
        let mut tile_feeders: Vec<((i32, i32), u8)> = Vec::new();
        for (ddx, ddy) in [(1, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (bx + ddx, by + ddy);
            if let Some(&nd) = belt_dir_map.get(&(nx, ny)) {
                let (ndx, ndy) = dir_to_vec(nd);
                if (nx + ndx, ny + ndy) != (bx, by) {
                    continue;
                }
                let feed_type = if nd == belt_d {
                    0u8
                } else {
                    let dot = (nx - bx) * left_dx + (ny - by) * left_dy;
                    // dot == 0 ⟺ the neighbor flows into this belt's FRONT
                    // face (opposite direction, head-on). Items enter a
                    // belt from behind or the sides, never the front — two
                    // facing belts jam at the interface and transfer
                    // NOTHING (factorio-mechanics.md). Classifying this as
                    // a sideload made the two belts mutual feeders: a
                    // gain-1 cycle the Jacobi pass grew by one seed per
                    // sweep up to the iteration budget — the impossible
                    // 645/4,192-per-s lane readings on DI fixtures whose
                    // output runs abut a ghost return belt (#632 B5).
                    if dot == 0 {
                        continue;
                    }
                    if dot > 0 { 1u8 } else { 2u8 }
                };
                tile_feeders.push(((nx, ny), feed_type));
            }
        }
        if !tile_feeders.is_empty() {
            feeders.insert((bx, by), tile_feeders);
        }
    }

    let mut splitter_sibling: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    // Owning splitter entity for each of its two tiles, so the
    // priority-loop model in `splitter_output_rates`/`splitter_output_rates_convergence`
    // can look up `loop_priority_rate`.
    let mut splitter_entity: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    for e in &layout.entities {
        if is_splitter(&e.name) {
            let second = splitter_second_tile(e);
            splitter_sibling.insert((e.x, e.y), second);
            splitter_sibling.insert(second, (e.x, e.y));
            splitter_entity.insert((e.x, e.y), e);
            splitter_entity.insert(second, e);
        }
    }

    // Segment id per belt tile (surface belts, UG in/out, splitters), used
    // to find a self-loop-tagged tile immediately downstream of a splitter
    // output — see `splitter_output_rates`.
    let mut belt_segment: FxHashMap<(i32, i32), Option<&str>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_segment.insert((e.x, e.y), e.segment_id.as_deref());
            if is_splitter(&e.name) {
                belt_segment.insert(splitter_second_tile(e), e.segment_id.as_deref());
            }
        }
    }

    // Whether `tile` is the priority (feed / loop-back) branch of its splitter.
    // Preferred signal is the entity's real Factorio field `output_priority`,
    // resolved geometrically by the shared mapping the structural tap check
    // owns — robust where the feed's first tile is overlaid by a trunk or
    // crossing belt and the downstream `:selfloop:` / `:mergetap:` segment tag
    // is absent. The tag (downstream of `tile`) is a fallback only for entities
    // with no `output_priority`; self-loop and merge-tap taps both set it, so a
    // priority splitter reaching the fallback is a stamper bug (debug-asserted).
    let is_priority_branch = |tile: (i32, i32)| -> bool {
        if let Some(&e) = splitter_entity.get(&tile) {
            if let Some(pt) = super::belt_structural::priority_output_tile(e) {
                return pt == tile;
            }
            debug_assert!(
                e.loop_priority_rate.is_none(),
                "priority splitter at {tile:?} has loop_priority_rate but no output_priority"
            );
        }
        let Some(&dir) = belt_dir_map.get(&tile) else {
            return false;
        };
        let (dx, dy) = dir_to_vec(dir);
        super::segment_is_priority_branch(
            belt_segment.get(&(tile.0 + dx, tile.1 + dy)).copied().flatten(),
        )
    };

    let mut in_degree: FxHashMap<(i32, i32), i32> =
        belt_dir_map.keys().map(|&p| (p, 0)).collect();
    for (&pos, tile_feeders) in &feeders {
        in_degree.insert(pos, tile_feeders.len() as i32);
    }
    // Unify splitter pair in-degrees: both tiles share the sum of both sides'
    // feeders. Without this, a 1→2 splitter whose "empty" side has no feeders
    // enters the queue immediately (in_degree=0) and exhausts retries waiting
    // for its sibling, propagating with stale [0,0] rates. See belt-flow bug
    // where tier2 copper-cable trunks fed from a single-input splitter all
    // delivered 0/s to downstream rows.
    let mut visited_pairs: FxHashSet<((i32, i32), (i32, i32))> = FxHashSet::default();
    for (&a, &b) in &splitter_sibling {
        let key = if a < b { (a, b) } else { (b, a) };
        if !visited_pairs.insert(key) {
            continue;
        }
        let total = in_degree.get(&a).copied().unwrap_or(0)
            + in_degree.get(&b).copied().unwrap_or(0);
        in_degree.insert(a, total);
        in_degree.insert(b, total);
    }

    // Virtual dependency: each UG-output inherits its rate from the surface
    // tile "behind" its paired UG-input. Track those dependencies as an
    // additional in_degree bump so UG-outputs don't dequeue before their
    // source is ready. `behind_to_ug_outputs` lets us decrement the UG-output's
    // counter once the "behind" tile is processed.
    let mut behind_to_ug_outputs: FxHashMap<(i32, i32), Vec<(i32, i32)>> =
        FxHashMap::default();
    for &ug_out in &ug_output_tiles {
        if let Some(&paired_input) = ug_output_to_input.get(&ug_out) {
            if let Some(&inp_d) = ug_input_dir.get(&paired_input) {
                let (idx, idy) = dir_to_vec(inp_d);
                let behind = (paired_input.0 - idx, paired_input.1 - idy);
                if belt_dir_map.contains_key(&behind) && behind != ug_out {
                    behind_to_ug_outputs
                        .entry(behind)
                        .or_default()
                        .push(ug_out);
                    *in_degree.entry(ug_out).or_insert(0) += 1;
                }
            }
        }
    }

    // Input-inserter pickup demand per belt tile, serving TWO roles:
    // (1) the backward demand pass (RFC rfc-lane-demand-flow.md Phase 1
    //     Branch A) that steers splitter allocation toward demand, and
    // (2) the forward CONSUMPTION DECREMENT (#519): each tile's outflow
    //     to its downstream neighbor is its arriving rate minus this
    //     tile's pickup demand, so a shared row belt depletes machine by
    //     machine and the tail sees leftovers, not the head supply. The
    //     walker previously propagated undecremented rates, which let
    //     `input-rate-delivery` compare every inserter against the full
    //     row supply — tail starvation (ac5-on simming at 75% of plan
    //     while E0/W0) was structurally invisible.
    let mut input_ins_count: FxHashMap<((i32, i32), String), usize> = FxHashMap::default();
    let mut input_ins: Vec<((i32, i32), (i32, i32), String)> = Vec::new();
    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);
        if !machine_tiles_set.contains(&drop_pos) || !belt_dir_map.contains_key(&pickup_pos) {
            continue;
        }

        let mpos = match machine_by_tile.get(&drop_pos) {
            Some(&p) => p,
            None => continue,
        };
        let item = match belt_carries.get(&pickup_pos).and_then(|c| c.as_deref()) {
            Some(i) => i.to_string(),
            None => continue,
        };
        *input_ins_count.entry((mpos, item.clone())).or_insert(0) += 1;
        input_ins.push((pickup_pos, mpos, item));
    }
    let mut base_demand: FxHashMap<(i32, i32), f64> = FxHashMap::default();
    for (pickup, mpos, item) in &input_ins {
        let me = match machine_entity.get(mpos) {
            Some(e) => e,
            None => continue,
        };
        let recipe = match me.recipe.as_deref() {
            Some(r) => r,
            None => continue,
        };
        let fallback_spec = match recipe_to_spec.get(recipe) {
            Some(s) => *s,
            None => continue,
        };
        // Self-loop draws are EXEMPT from consumption decrement (#519): a
        // loop's standing inventory circulates — the machine draws from and
        // re-feeds the same ring, and modeling the draw without the ring's
        // closed-inventory semantics collapses the modeled flow to zero
        // (kovarex's legendary arm read u-238 at 0.0/s on a loop the game
        // sustains indefinitely). Scoped by the spec's own self_loop
        // declaration, so a DIFFERENT consumer of the looped item (e.g. the
        // voider row drawing u-238) still depletes normally.
        if fallback_spec.self_loop.iter().any(|f| &f.item == item) {
            continue;
        }
        // Same position-resolved attribution as the injection loop above —
        // see `super::resolve_row_spec`'s doc comment.
        let (spec, band) = super::resolve_row_spec_banded(layout, recipe, me.y, fallback_spec);
        let utilization = physical_utilization(spec, fallback_spec, band, &machine_ys_by_recipe, &per_row_count_recipes);
        let required = spec
            .inputs
            .iter()
            .find(|i| &i.item == item)
            .map(|i| i.rate * utilization)
            .unwrap_or(0.0);
        if required <= 0.0 {
            continue;
        }
        let count = input_ins_count
            .get(&(*mpos, item.clone()))
            .copied()
            .unwrap_or(1);
        *base_demand.entry(*pickup).or_insert(0.0) += required / count as f64;
    }

    let mut lane_rates: FxHashMap<(i32, i32), [f64; 2]> = belt_dir_map
        .keys()
        .map(|&p| (p, lane_injections.get(&p).copied().unwrap_or([0.0, 0.0])))
        .collect();

    // Convergence budget (RFC kill criterion 2): a HARD `3 × segment_count`,
    // bounding both the demand pass and the forward fixed-point pass. Here a
    // belt "segment" is one tile of the belt graph — the walker's actual
    // propagation unit. This is the reading under which the hard budget
    // accommodates the walker's *pre-existing*, demand-independent balancer
    // convergence: a bare (3, 3) library template (33 tiles) needs 90
    // even-split iterations, which `3 × distinct-segment-ids` (= 3) cannot
    // cover but `3 × belt_tiles` (= 99) does — and it stays meaningful, since
    // a well-conditioned fixed point converges in O(tiles) and only genuine
    // oscillation/divergence exceeds 3× that. Measured max across the corpus
    // is 313 iters on the 5118-tile utility layout (0.02 × its budget); the
    // kill criterion fires nowhere. See docs/rfc-lane-demand-flow.md.
    // (Computed BEFORE the external seeding since #519: the seeds are
    // demand-weighted, so the backward demand pass must already have run.)
    let segment_count = belt_dir_map.len().max(1);
    let budget = 3 * segment_count;

    let (demand, demand_sweeps) = compute_demand(
        &belt_dir_map,
        &feeders,
        &splitter_sibling,
        &ug_output_tiles,
        &ug_output_to_input,
        &ug_input_dir,
        &base_demand,
        budget,
    );

    // Seed graph-source belts that carry external input items. External inputs
    // come from outside the layout and have no upstream producer in the belt
    // graph — without this seeding, rate propagation starts at 0 and every
    // downstream consumer of an external input is incorrectly flagged as
    // starved.
    //
    // Seeds are DEMAND-WEIGHTED (#519): a boundary trunk physically supplies
    // whatever its consumers pull (up to belt capacity) — the solver total is
    // an aggregate, not a per-trunk allotment. The old even split
    // under-supplied high-demand trunks and over-supplied low-demand ones
    // (chem5: iron-plate 35/s over six trunks = 5.83 each against one row's
    // 6.25/s draw — a fabricated 0.42/s tail deficit on a sim-verified
    // layout). Each source now seeds its own backward-propagated downstream
    // demand, scaled down proportionally if the demands exceed the solver
    // total (a genuine aggregate shortfall stays visible), and falling back
    // to the even split when no demand is attributable (unmodeled consumers).
    let external_rates: FxHashMap<&str, f64> = sr
        .external_inputs
        .iter()
        .filter(|f| !f.is_fluid)
        .map(|f| (f.item.as_str(), f.rate))
        .collect();
    if !external_rates.is_empty() {
        // First pass: group source tiles by the item they carry. A "source" is a
        // belt tile that has no upstream feeder in the surface belt graph. We
        // include UG outputs here too: although they inherit rate via the topo
        // sort's UG special case, that inheritance relies on the "behind the UG
        // input" surface tile being correctly seeded — for external inputs it's
        // simpler and safer to seed every graph source independently.
        let mut sources_by_item: FxHashMap<&str, Vec<(i32, i32)>> = FxHashMap::default();
        for &pos in belt_dir_map.keys() {
            if feeders.contains_key(&pos) {
                continue; // has upstream feeders, not a source
            }
            // A splitter pair is ONE machine: if either half has feeders,
            // the pair is fed and the unfed half is not a graph source.
            // Counting it as one fabricated a phantom source on every
            // inline splitter whose second tile sits beside the fed lane
            // (#624: 26 phantoms removed on the ON0 donor — 30 sources →
            // 4 — Σ attributed demand 1.5× the solver total, even-split
            // fallback distorting every real seed).
            //
            // KNOWN LIMIT of this rule (bot review on the fix PR): a
            // splitter half where external flow genuinely arrives while
            // its sibling is INDEPENDENTLY belt-fed (e.g. an entry
            // splitter that also receives an internal recirculation loop)
            // would be wrongly skipped — the walker cannot distinguish
            // "fed pair" from "fed sibling + external entry half" by
            // graph shape alone. No engine layout, celldb entry, or
            // balancer template has that topology today (both donor entry
            // splitters have BOTH halves unfed and still seed); if one
            // appears, seed-stats (`SPAGHETTIO_LANE_WALK_STATS=1`) will
            // show the missing source.
            if let Some(&sib) = splitter_sibling.get(&pos) {
                if feeders.contains_key(&sib) {
                    continue;
                }
            }
            // A UG output whose paired entrance is fed from a graph tile
            // is NOT a source: the topo sort's UG special case inherits
            // the behind-the-entrance rates onto it, so seeding it too
            // double-counts — and the phantom source breaks the item's
            // demand attribution (Σ ≠ solver total), demoting every real
            // trunk to the even-split fallback (#644: stress-ec30's two
            // crossing exits read 18/s out of a 9/s tunnel while the real
            // trunks under-seeded at 9/s instead of 15/s). Only an
            // ORPHANED exit — no paired entrance, or nothing behind it —
            // genuinely admits flow into the graph and keeps its seed.
            // Mirrors the topo sort's inheritance WIRING condition
            // (behind ∈ belt_dir_map, behind != this exit — see
            // `behind_to_ug_outputs`); the inheritance PROCESSING
            // block itself carries no behind!=pos clause, but
            // `build_ug_pairs` geometry keeps behind==pos unreachable
            // (the exit sits strictly ahead of the entrance). The skip
            // is sound only when inheritance will actually deliver.
            // Structural presence is the shared predicate by design —
            // neither side checks that `behind` is fed or carries the
            // tunnel's item, so tightening only one side would create
            // the de-seed-without-inherit divergence this guard
            // exists to prevent.
            if ug_output_tiles.contains(&pos) {
                if let Some(&paired_input) = ug_output_to_input.get(&pos) {
                    if let Some(&inp_d) = ug_input_dir.get(&paired_input) {
                        let (idx, idy) = dir_to_vec(inp_d);
                        let behind = (paired_input.0 - idx, paired_input.1 - idy);
                        if belt_dir_map.contains_key(&behind) && behind != pos {
                            continue;
                        }
                    }
                }
            }
            if let Some(Some(item)) = belt_carries.get(&pos) {
                if external_rates.contains_key(item.as_str()) {
                    sources_by_item
                        .entry(external_rates.get_key_value(item.as_str()).unwrap().0)
                        .or_default()
                        .push(pos);
                }
            }
        }
        // Second pass: seed each source tile, split evenly across the belt's
        // two lanes.
        //
        // Seeds are demand-weighted ONLY when the backward demand map is
        // SELF-CONSISTENT for the item (Σ attributed ≈ solver total): a
        // boundary trunk physically supplies what its consumers pull, and
        // when attribution reconciles with the plan, weighting fixes the
        // even split's misallocation (chem5: iron 35/s over six trunks =
        // 5.83 each against one row's 6.25 draw — a fabricated tail
        // deficit on a sim-verified layout; Σd = 35.000 exactly).
        //
        // When it does NOT reconcile, the map is untrustworthy in absolute
        // terms — compute_demand propagates the full downstream demand up
        // EVERY branch of a merge (correct as an upper bound for splitter
        // RATIOS, its original job), so merge-heavy layouts over-attribute
        // (ec45: three sources each claiming the same 33.75/s row, Σ=1.5×;
        // mil5 snake-fold: enough compounded merges that per-source
        // optimistic seeding modeled 57/s lanes on express and threw 406
        // false lane-cap errors). Fall back to the conservation-obeying
        // even split — the pre-#519 behavior — rather than guess.
        // Attribution across merges is the open follow-up on #519.
        for (item, sources) in &sources_by_item {
            let total = external_rates[item];
            let demands: Vec<f64> = sources
                .iter()
                .map(|pos| demand.get(pos).copied().unwrap_or(0.0))
                .collect();
            let demand_sum: f64 = demands.iter().sum();
            let attribution_consistent =
                demand_sum > 0.0 && (demand_sum - total).abs() <= 0.05 * total.max(1e-9);
            let even = total / sources.len() as f64;
            if std::env::var("SPAGHETTIO_LANE_WALK_STATS").is_ok() {
                eprintln!(
                    "seed-stats item={item} total={total:.3} demand_sum={demand_sum:.3} \
                     consistent={attribution_consistent} sources={:?}",
                    sources.iter().zip(&demands).collect::<Vec<_>>()
                );
            }
            for (&pos, &d) in sources.iter().zip(&demands) {
                // Deliberately UNCAPPED: a genuinely over-committed trunk
                // (the #311/#312 class, pinned by stacking_fanin_wall_lift)
                // must keep modeling above physical capacity so
                // lane-throughput can see it; the 90/s-on-yellow incident
                // was a candidate-selection flip (fixed by the scoped
                // ranking count), not a seeding overflow.
                let per_tile = if attribution_consistent { d * (total / demand_sum) } else { even };
                let entry = lane_rates.entry(pos).or_insert([0.0, 0.0]);
                entry[0] += per_tile / 2.0;
                entry[1] += per_tile / 2.0;
            }
        }
    }

    // Snapshot the seed rates (lane_injections + external-source seeds) before
    // the topo-sort mutates lane_rates. The iterative convergence pass below
    // uses these as the immutable "always-present" base for non-splitter tiles:
    // each iteration recomputes `next[pos] = seed_rates[pos] + feeder_sum(pos)`,
    // and seed values would otherwise be lost after the first iteration since
    // feeders accumulate on top of whatever's there.
    let seed_rates: FxHashMap<(i32, i32), [f64; 2]> = lane_rates.clone();

    let mut processed: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut splitter_input_ready: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut queue: VecDeque<(i32, i32)> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&p, _)| p)
        .collect();

    // Splitter sibling waits are still retry-driven (the pair is serialized,
    // so whichever half dequeues second triggers the averaging path).
    let mut splitter_retries: FxHashMap<(i32, i32), u32> = FxHashMap::default();
    const MAX_RETRIES: u32 = 3;

    // Helper: after marking `tile` processed, decrement in_degree for any
    // UG-output tiles that depended on `tile` as their "behind" source, and
    // enqueue them if ready.
    let notify_ug_deps = |tile: (i32, i32),
                          in_degree: &mut FxHashMap<(i32, i32), i32>,
                          queue: &mut VecDeque<(i32, i32)>| {
        if let Some(deps) = behind_to_ug_outputs.get(&tile) {
            for &ug_out in deps {
                let d = in_degree.entry(ug_out).or_insert(0);
                *d -= 1;
                if *d <= 0 {
                    queue.push_back(ug_out);
                }
            }
        }
    };

    while let Some(pos) = queue.pop_front() {
        if processed.contains(&pos) {
            continue;
        }

        // Underground output: inherit from behind paired input. The topo-sort
        // dependency is encoded via `behind_to_ug_outputs` — this tile won't
        // be dequeued until `behind` has been processed, so we can read its
        // rates directly.
        if ug_output_tiles.contains(&pos) {
            if let Some(&paired_input) = ug_output_to_input.get(&pos) {
                if let Some(&inp_d) = ug_input_dir.get(&paired_input) {
                    let (idx, idy) = dir_to_vec(inp_d);
                    let behind = (paired_input.0 - idx, paired_input.1 - idy);
                    if let Some(&behind_rates) = lane_rates.get(&behind) {
                        // Pickups at `behind` eat before the tunnel (#519).
                        let behind_rates = outflow_after_pickup(
                            behind_rates,
                            base_demand.get(&behind).copied().unwrap_or(0.0),
                        );
                        let rates = lane_rates.entry(pos).or_insert([0.0, 0.0]);
                        rates[0] += behind_rates[0];
                        rates[1] += behind_rates[1];
                    }
                }
            }
        }

        // Splitter: wait for sibling
        if let Some(&sib) = splitter_sibling.get(&pos) {
            if !processed.contains(&sib) {
                splitter_input_ready.insert(pos);
                if !splitter_input_ready.contains(&sib) {
                    let retry = splitter_retries.entry(pos).or_insert(0);
                    if *retry < MAX_RETRIES {
                        *retry += 1;
                        queue.push_back(pos);
                        continue;
                    }
                    // Gave up waiting for sibling — mark processed with current
                    // rates and skip averaging to avoid silently wrong numbers.
                    processed.insert(pos);
                    do_propagate(pos, &belt_dir_map, &feeders, &splitter_sibling, &mut in_degree, &mut queue, &mut lane_rates, &base_demand);
                    notify_ug_deps(pos, &mut in_degree, &mut queue);
                    continue;
                } else {
                    let pos_rates = lane_rates.get(&pos).copied().unwrap_or([0.0, 0.0]);
                    let sib_rates = lane_rates.get(&sib).copied().unwrap_or([0.0, 0.0]);
                    let loop_priority_rate =
                        splitter_entity.get(&pos).and_then(|e| e.loop_priority_rate);
                    let (pos_out, sib_out) = splitter_output_rates(
                        (pos_rates[0], pos_rates[1]),
                        (sib_rates[0], sib_rates[1]),
                        loop_priority_rate,
                        is_priority_branch(pos),
                        is_priority_branch(sib),
                    );
                    lane_rates.insert(pos, [pos_out.0, pos_out.1]);
                    lane_rates.insert(sib, [sib_out.0, sib_out.1]);
                    for &tile in &[sib, pos] {
                        processed.insert(tile);
                        do_propagate(tile, &belt_dir_map, &feeders, &splitter_sibling, &mut in_degree, &mut queue, &mut lane_rates, &base_demand);
                        notify_ug_deps(tile, &mut in_degree, &mut queue);
                    }
                    continue;
                }
            }
        }

        processed.insert(pos);
        do_propagate(pos, &belt_dir_map, &feeders, &splitter_sibling, &mut in_degree, &mut queue, &mut lane_rates, &base_demand);
        notify_ug_deps(pos, &mut in_degree, &mut queue);
    }

    // Cycle-breaker pass: tiles that are part of belt loops (e.g. internal
    // feedback paths inside N-to-M balancer templates) never reach in_degree==0
    // in the main topo-sort above because each tile waits for its predecessor,
    // which in turn waits for it.  After the main queue drains, force-process
    // any remaining tile whose *explicit* feeders (as recorded in `feeders`) are
    // all already done.  Those tiles were blocked only by the splitter-sibling
    // unified in_degree or a UG virtual dep that will never fire, not by a real
    // missing input.  Iterate until no further progress is made.
    loop {
        // Tier-1 freed: tiles where both own feeders AND sibling's feeders are all
        // processed (or the sibling is already done).  These are safe to process
        // immediately — the sibling will have real rates when we average.
        let tier1: Vec<(i32, i32)> = belt_dir_map
            .keys()
            .filter(|&&p| {
                !processed.contains(&p)
                    && !ug_output_tiles.contains(&p)
                    && feeders
                        .get(&p)
                        .is_none_or(|fs| fs.iter().all(|(fp, _)| processed.contains(fp)))
                    && splitter_sibling
                        .get(&p)
                        .is_none_or(|&sib| {
                            processed.contains(&sib)
                                || feeders
                                    .get(&sib)
                                    .is_none_or(|fs| fs.iter().all(|(fp, _)| processed.contains(fp)))
                        })
            })
            .copied()
            .collect();

        if !tier1.is_empty() {
            // Safe batch: process and let notify propagate before the next iteration.
            for pos in tier1 {
                in_degree.insert(pos, 0);
                queue.push_back(pos);
            }
        } else {
            // No fully-safe tile exists.  Fall back to forcing ONE tile from the
            // broader "own feeders all processed" set and draining the queue fully.
            // This breaks the deadlock at the cost of possibly averaging with a
            // [0,0] cycle tile — acceptable for a feedback loop that has a real
            // input on at least one side (the cycle tile inherits the same rate).
            //
            // Prefer tiles that already have non-zero lane_rates (they carry real
            // throughput) over tiles with all-zero rates (pure cycle tiles). This
            // ensures the "real-input" side of a splitter pair is freed first so
            // the averaging uses the actual rate rather than [0,0].
            let candidates: Vec<(i32, i32)> = belt_dir_map
                .keys()
                .filter(|&&p| {
                    !processed.contains(&p)
                        && !ug_output_tiles.contains(&p)
                        && feeders
                            .get(&p)
                            .is_none_or(|fs| fs.iter().all(|(fp, _)| processed.contains(fp)))
                })
                .copied()
                .collect();
            let fallback: Option<(i32, i32)> = candidates
                .iter()
                .find(|&&p| {
                    lane_rates.get(&p).is_some_and(|&r| r[0] > 0.0 || r[1] > 0.0)
                })
                .or_else(|| candidates.first())
                .copied();
            match fallback {
                None => break,
                Some(pos) => {
                    in_degree.insert(pos, 0);
                    queue.push_back(pos);
                }
            }
        }
        while let Some(pos) = queue.pop_front() {
            if processed.contains(&pos) {
                continue;
            }
            if ug_output_tiles.contains(&pos) {
                if let Some(&paired_input) = ug_output_to_input.get(&pos) {
                    if let Some(&inp_d) = ug_input_dir.get(&paired_input) {
                        let (idx, idy) = dir_to_vec(inp_d);
                        let behind = (paired_input.0 - idx, paired_input.1 - idy);
                        if let Some(&behind_rates) = lane_rates.get(&behind) {
                            // Pickups at `behind` eat before the tunnel (#519).
                            let behind_rates = outflow_after_pickup(
                                behind_rates,
                                base_demand.get(&behind).copied().unwrap_or(0.0),
                            );
                            let rates = lane_rates.entry(pos).or_insert([0.0, 0.0]);
                            rates[0] += behind_rates[0];
                            rates[1] += behind_rates[1];
                        }
                    }
                }
            }
            if let Some(&sib) = splitter_sibling.get(&pos) {
                if !processed.contains(&sib) {
                    splitter_input_ready.insert(pos);
                    // Also force-free the sibling so the averaging path can fire.
                    in_degree.insert(sib, 0);
                    queue.push_back(sib);
                    if !splitter_input_ready.contains(&sib) {
                        let retry = splitter_retries.entry(pos).or_insert(0);
                        if *retry < MAX_RETRIES {
                            *retry += 1;
                            queue.push_back(pos);
                            continue;
                        }
                        processed.insert(pos);
                        do_propagate(pos, &belt_dir_map, &feeders, &splitter_sibling, &mut in_degree, &mut queue, &mut lane_rates, &base_demand);
                        notify_ug_deps(pos, &mut in_degree, &mut queue);
                        continue;
                    } else {
                        let pos_rates = lane_rates.get(&pos).copied().unwrap_or([0.0, 0.0]);
                        let sib_rates = lane_rates.get(&sib).copied().unwrap_or([0.0, 0.0]);
                        // Use the combined rate of both halves and distribute
                        // equally. This correctly models 1→2 balanced splitting
                        // (one half has the feeder rate, the other has 0) as well
                        // as 2→2 splits and feedback-loop steady states — in all
                        // cases the splitter gives each output half the total input.
                        // The old "propagate non-zero to both" rule inflated rates
                        // 2× for the stuck-secondary case, causing false lane-
                        // throughput errors in the template audit.
                        let (eff_pos, eff_sib) = (pos_rates, sib_rates);
                        let loop_priority_rate =
                            splitter_entity.get(&pos).and_then(|e| e.loop_priority_rate);
                        let (pos_out, sib_out) = splitter_output_rates(
                            (eff_pos[0], eff_pos[1]),
                            (eff_sib[0], eff_sib[1]),
                            loop_priority_rate,
                            is_priority_branch(pos),
                            is_priority_branch(sib),
                        );
                        lane_rates.insert(pos, [pos_out.0, pos_out.1]);
                        lane_rates.insert(sib, [sib_out.0, sib_out.1]);
                        for &tile in &[sib, pos] {
                            processed.insert(tile);
                            do_propagate(tile, &belt_dir_map, &feeders, &splitter_sibling, &mut in_degree, &mut queue, &mut lane_rates, &base_demand);
                            notify_ug_deps(tile, &mut in_degree, &mut queue);
                        }
                        continue;
                    }
                }
            }
            processed.insert(pos);
            do_propagate(pos, &belt_dir_map, &feeders, &splitter_sibling, &mut in_degree, &mut queue, &mut lane_rates, &base_demand);
            notify_ug_deps(pos, &mut in_degree, &mut queue);
        }
    }

    // Underground-belt input → its paired output (tunnel exit). A splitter
    // whose output feeds a UG-input has its immediate downstream *inside* the
    // tunnel — a tile that isn't in `belt_dir_map` — so the demand behind the
    // tunnel must be read at the exit, or proportional splitting would route
    // zero flow across every UG hop (observed as 0.0/s starvation on
    // processing-unit@2/s).
    let ug_input_to_output: FxHashMap<(i32, i32), (i32, i32)> = ug_output_to_input
        .iter()
        .map(|(&out, &inp)| (inp, out))
        .collect();

    // Total downstream demand reachable through a splitter's output tile
    // `ds`. When `ds` is a splitter tile, use the pooled demand of its pair
    // (the flow entering `ds` is re-split there, so the whole downstream
    // sub-tree's demand is what this branch must feed). When `ds` is a
    // UG-input, resolve across the tunnel to the exit's demand.
    let resolve_demand = |t: (i32, i32)| -> Option<f64> {
        if !belt_dir_map.contains_key(&t) {
            return None;
        }
        let base = demand.get(&t).copied().unwrap_or(0.0);
        let extra = splitter_sibling
            .get(&t)
            .map(|&sib| demand.get(&sib).copied().unwrap_or(0.0))
            .unwrap_or(0.0);
        Some(base + extra)
    };
    let downstream_demand = |ds: (i32, i32)| -> f64 {
        if let Some(d) = resolve_demand(ds) {
            d
        } else if let Some(&ug_out) = ug_input_to_output.get(&ds) {
            resolve_demand(ug_out).unwrap_or(0.0)
        } else {
            // Off-map (external export) or otherwise no known consumer.
            0.0
        }
    };

    // Iterate-to-convergence pass. The Kahn topo-sort + cycle-breaker above
    // gives correct rates for acyclic belt sub-graphs but settles for whatever
    // it produces on the first reach into balancer-internal feedback loops —
    // splitter pairs in those loops can end up with unbalanced halves (one
    // half picks up its feeder rate, the other half's feedback hasn't been
    // computed yet). This pass treats the rate map as a fixed point of a
    // linear transfer function `T(x) = x` and iterates Jacobi-style until it
    // converges.  Splitters dampen cycle gain by 0.5 per pass, so feedback
    // error decays geometrically; ~14 iterations suffice to drop a 15/s seed
    // below 1e-3.
    let mut forward_iters = 0usize;
    let mut forward_converged = false;
    {
        let max_iter = budget;
        const EPS: f64 = 1e-5;

        // Pre-collect splitter pairs (canonical order) so we visit each once.
        let mut pair_set: Vec<((i32, i32), (i32, i32))> = Vec::new();
        let mut seen_pair: FxHashSet<((i32, i32), (i32, i32))> = FxHashSet::default();
        for (&a, &b) in &splitter_sibling {
            let key = if a < b { (a, b) } else { (b, a) };
            if seen_pair.insert(key) {
                pair_set.push(key);
            }
        }

        for _iter in 0..max_iter {
            forward_iters += 1;
            let prev = lane_rates.clone();
            let mut next: FxHashMap<(i32, i32), [f64; 2]> = FxHashMap::default();

            // Phase 1: non-splitter, non-UG-output tiles.
            // rate = seed (injections + external sources) + sum of feeder contributions.
            for &pos in belt_dir_map.keys() {
                if splitter_sibling.contains_key(&pos) || ug_output_tiles.contains(&pos) {
                    continue;
                }
                let seed = seed_rates.get(&pos).copied().unwrap_or([0.0, 0.0]);
                let fc = feeder_contributions_for_tile(pos, &prev, &feeders, &belt_dir_map, &base_demand);
                next.insert(pos, [seed[0] + fc[0], seed[1] + fc[1]]);
            }

            // Phase 2: splitter pairs. LANE-PRESERVING (RFC-047 Phase 0):
            // real Factorio splitters never mix lanes (S4) — the previous
            // comment here asserted the opposite as fact and the old model
            // implemented it, silently re-balancing lane-imbalanced
            // upstream flow at every splitter. Each lane's pooled feeder
            // contribution is now distributed independently, so genuine
            // one-lane skew propagates and can be caught downstream.
            //
            // Priority splitters (`loop_priority_rate` set) break that
            // symmetry: the loop-back branch draws `min(total, cap)` and
            // the export branch gets the remainder, via
            // `splitter_output_rates_convergence` (per-lane, RFC-047).
            for &(a, b) in &pair_set {
                // Seeds land on splitter tiles too — an entry splitter fed
                // from the layout edge carries an external-source seed with
                // zero feeders, and phase 1 folds `seed_rates` into every
                // non-splitter tile. Omitting it here erased the seed on
                // the first iteration and converged the entire downstream
                // graph to 0/s (#624: every feeder of the RFC-067 donor
                // cells read "delivers 0.0/s" while seed-stats showed the
                // seeding itself was correct). NOTE `seed_rates` is
                // injections + external seeds, so this line also repairs
                // the sibling defect for INSERTER DROPS onto splitter
                // tiles (previously equally erased) — no engine layout
                // exercises that today (the attributed-footprint census
                // would have shown it), but it is the same fix, claimed
                // deliberately rather than fixed by accident.
                let a_seed = seed_rates.get(&a).copied().unwrap_or([0.0, 0.0]);
                let b_seed = seed_rates.get(&b).copied().unwrap_or([0.0, 0.0]);
                let a_fc = feeder_contributions_for_tile(a, &prev, &feeders, &belt_dir_map, &base_demand);
                let b_fc = feeder_contributions_for_tile(b, &prev, &feeders, &belt_dir_map, &base_demand);
                let a_fc = [a_seed[0] + a_fc[0], a_seed[1] + a_fc[1]];
                let b_fc = [b_seed[0] + b_fc[0], b_seed[1] + b_fc[1]];
                let loop_priority_rate =
                    splitter_entity.get(&a).and_then(|e| e.loop_priority_rate);
                // Demand at each output tile's downstream, and the per-output
                // belt-capacity cap (full belt throughput of the splitter tier).
                let a_ds = {
                    let (adx, ady) = dir_to_vec(belt_dir_map[&a]);
                    (a.0 + adx, a.1 + ady)
                };
                let b_ds = {
                    let (bdx, bdy) = dir_to_vec(belt_dir_map[&b]);
                    (b.0 + bdx, b.1 + bdy)
                };
                let cap = splitter_entity
                    .get(&a)
                    .map(|e| {
                        // RFC-046: splitters are stack-preserving (BS4) —
                        // per-output cap scales with the ITEM-effective S
                        // (exemption-aware, like the lane caps; falls back
                        // to the layout-wide value without a `carries`).
                        let s = e
                            .carries
                            .as_deref()
                            .map(|item| rates_stacking_ctx.for_item(item))
                            .unwrap_or(layout.stacking);
                        belt_throughput_stacked(splitter_to_surface_tier(&e.name), s)
                    })
                    .unwrap_or(15.0);
                let (a_out, b_out) = splitter_output_rates_convergence(
                    (a_fc[0], a_fc[1]),
                    (b_fc[0], b_fc[1]),
                    loop_priority_rate,
                    is_priority_branch(a),
                    is_priority_branch(b),
                    downstream_demand(a_ds),
                    downstream_demand(b_ds),
                    cap,
                );
                next.insert(a, a_out);
                next.insert(b, b_out);
            }

            // Phase 3: UG-output tiles inherit from the surface tile behind
            // their paired UG-input. Use `next` (already updated in phase 1/2)
            // when available, else fall back to `prev`. The walker ADDs behind
            // to any seed (e.g. an inserter dropping onto the UG-output's
            // surface tile contributes alongside the underground throughput),
            // so we mirror that here — REPLACE would silently drop injected
            // rate.
            for &ug_out in &ug_output_tiles {
                let Some(&paired_input) = ug_output_to_input.get(&ug_out) else {
                    continue;
                };
                let Some(&inp_d) = ug_input_dir.get(&paired_input) else {
                    continue;
                };
                let (idx, idy) = dir_to_vec(inp_d);
                let behind = (paired_input.0 - idx, paired_input.1 - idy);
                let behind_rates = outflow_after_pickup(
                    next.get(&behind)
                        .copied()
                        .or_else(|| prev.get(&behind).copied())
                        .unwrap_or([0.0, 0.0]),
                    // Pickups at `behind` eat before the tunnel (#519).
                    base_demand.get(&behind).copied().unwrap_or(0.0),
                );
                let seed = seed_rates.get(&ug_out).copied().unwrap_or([0.0, 0.0]);
                next.insert(ug_out, [seed[0] + behind_rates[0], seed[1] + behind_rates[1]]);
            }

            // Convergence check: max per-lane absolute difference across all tiles.
            let mut max_change: f64 = 0.0;
            for (pos, &[nl, nr]) in &next {
                let &[pl, pr] = prev.get(pos).unwrap_or(&[0.0, 0.0]);
                let dl = (nl - pl).abs();
                let dr = (nr - pr).abs();
                if dl > max_change {
                    max_change = dl;
                }
                if dr > max_change {
                    max_change = dr;
                }
            }
            lane_rates = next;
            if max_change < EPS {
                forward_converged = true;
                break;
            }
        }
    }

    // Instrumentation for RFC kill criterion 2. The demand-pull fixed point
    // must converge within the `3 × segment_count` budget on every corpus
    // layout; if the forward pass ever exhausts it without converging, the
    // iterative model is wrong (STOP and report — do not widen the budget).
    if std::env::var("SPAGHETTIO_LANE_WALK_STATS").is_ok() {
        let splitter_pairs = {
            let mut seen: FxHashSet<((i32, i32), (i32, i32))> = FxHashSet::default();
            for (&a, &b) in &splitter_sibling {
                seen.insert(if a < b { (a, b) } else { (b, a) });
            }
            seen.len()
        };
        eprintln!(
            "lane-walk-stats forward_iters={forward_iters} forward_converged={forward_converged} \
             demand_sweeps={demand_sweeps} segment_count={segment_count} budget={budget} \
             splitter_pairs={splitter_pairs} belt_tiles={}",
            belt_dir_map.len()
        );
    }

    // Post-pass: surface UG-input tiles inherit their upstream surface
    // belt's lane rates. Inserters can pick from both lanes of any
    // belt's surface, UG entries and exits included (rule I6). UG-out
    // tiles are already in `belt_dir_map` and pick up rates via normal
    // propagation; UG-in tiles aren't (their forward flow goes
    // underground, not to a surface neighbour) so the topo sort skips
    // them and `lane_rates` stays empty there. The input-rate-delivery
    // check looks up the inserter's pickup tile in `lane_rates` and
    // treats `None` as 0/s — without this fix-up, every long-handed
    // inserter picking across a UG-in fires a false-positive "input
    // belt delivers 0/s" warning.
    let ug_input_tiles: Vec<((i32, i32), EntityDirection)> = ug_input_dir
        .iter()
        .map(|(&pos, &dir)| (pos, dir))
        .collect();
    for ((ix, iy), dir) in ug_input_tiles {
        let (dx, dy) = dir_to_vec(dir);
        let upstream = (ix - dx, iy - dy);
        if let Some(&upstream_rates) = lane_rates.get(&upstream) {
            // What reaches the UG-in surface is upstream's outflow (#519).
            let upstream_rates = outflow_after_pickup(
                upstream_rates,
                base_demand.get(&upstream).copied().unwrap_or(0.0),
            );
            lane_rates.insert((ix, iy), upstream_rates);
        }
    }

    lane_rates
}

/// Compute the per-lane contribution from a tile flowing into a downstream
/// tile, applying Factorio's belt-mixing rules. Single source of truth for
/// the four lane-transfer cases used by both [`do_propagate`] (push) and
/// [`feeder_contribution`] (pull).
///
/// Cases (in order):
/// - **Same direction** → straight pass-through, lanes preserved.
/// - **`from` directly behind `to`** → also straight (e.g. UG-output feeding
///   a belt that turns).
/// - **`to` has a straight feeder** (`to_has_straight_feeder=true`) →
///   sideload: all flow goes onto the lane closest to `from`.
/// - **Otherwise** → 90-degree turn: lanes swap on CW, preserve on CCW.
fn lane_transfer(
    from_pos: (i32, i32),
    from_dir: EntityDirection,
    from_rates: [f64; 2],
    to_pos: (i32, i32),
    to_dir: EntityDirection,
    to_has_straight_feeder: bool,
) -> [f64; 2] {
    if from_dir == to_dir {
        return from_rates;
    }
    let (fdx, fdy) = dir_to_vec(from_dir);
    let (tdx, tdy) = dir_to_vec(to_dir);
    let behind_to = (to_pos.0 - tdx, to_pos.1 - tdy);
    if from_pos == behind_to {
        return from_rates;
    }

    if to_has_straight_feeder {
        let (left_dx, left_dy) = (-tdy, tdx);
        let rel_x = from_pos.0 - to_pos.0;
        let rel_y = from_pos.1 - to_pos.1;
        let dot = rel_x * left_dx + rel_y * left_dy;
        let total = from_rates[0] + from_rates[1];
        if dot > 0 {
            [total, 0.0]
        } else {
            [0.0, total]
        }
    } else {
        let cross = fdx * tdy - fdy * tdx;
        if cross > 0 {
            [from_rates[1], from_rates[0]]
        } else {
            [from_rates[0], from_rates[1]]
        }
    }
}

/// Per-machine utilization derived from the machines the layout PHYSICALLY
/// placed for `spec`'s scope (#519 fallout). `utilization_for` divides by
/// `ceil(spec.count)`, but chain/mega replication quantizes per COPY:
/// chem5's copper-cable count 15 places 8+8 = 16 machines across K=2
/// copies, so per-machine rates scaled by count/ceil(count) = 1.0 overstate
/// both the demand each machine draws and the output each machine injects
/// by 1/16. Undecremented propagation never noticed; the consumption
/// decrement made the overstatement eat 20/s from an 18.75/s belt and
/// fabricate a tail deficit on a sim-verified-at-plan layout.
///
/// Scope rule: `effective_rows` bands carry PER-ROW counts for partition
/// siblings (band counts sum to the recipe total — count within the band)
/// but DUPLICATED GLOBAL counts for throughput-split rows (each band
/// repeats the total — count across the whole layout, or a 2-row split
/// would cap utilization at 1.0 and lose the fractional machine: uranium
/// 85.71/86 regressed to 1.0). Recipes shared by multiple specs
/// (voider/self-loop siblings) are excluded upstream and fall back to
/// `utilization_for`.
/// FRACTIONAL-DUTY FLOOR (2026-08-07). `spec` here is the *band-resolved*
/// spec from `effective_rows`, whose count is the count the ROW PLACED — so
/// for a sub-one-machine plan it reads 1 while the solver planned 0.667, and
/// `spec.count / n` collapses to 1/1 = 1.0, silently discarding exactly the
/// scaling this function exists to apply. `plan_spec` is the solver's spec,
/// and `utilization_for(plan_spec)` is `count / ceil(count)` — the true duty
/// whenever the row places `ceil(count)` machines, which is the normal case.
/// Taking the min restores it and is a no-op for integral counts.
///
/// Measured: `big-electric-pole@1` on am2 plans 0.667 machines, the row
/// places 1, and the bus correctly delivers 8.0/s of iron-stick (= 0.667 ×
/// 12.0). Without the floor the check demanded a fully saturated machine's
/// 12.0/s and fired two warnings on a layout the sim measures at **110% of
/// plan** — false positives that inverted candidate ranking the moment
/// `input-rate-delivery` was allowed to steer selection (it preferred a
/// layout measured at 0.51/s against a 1.00/s plan). See
/// `docs/validator-trust.md` hole 2.
fn physical_utilization(
    spec: &crate::models::MachineSpec,
    plan_spec: &crate::models::MachineSpec,
    band: Option<(i32, i32)>,
    machine_ys_by_recipe: &FxHashMap<&str, Vec<i32>>,
    per_row_count_recipes: &FxHashSet<&str>,
) -> f64 {
    let plan_duty = utilization_for(plan_spec);
    let Some(ys) = machine_ys_by_recipe.get(spec.recipe.as_str()) else {
        return utilization_for(spec).min(plan_duty);
    };
    let n = match band {
        Some((y0, y1)) if per_row_count_recipes.contains(spec.recipe.as_str()) => {
            ys.iter().filter(|&&y| y >= y0 && y < y1).count()
        }
        _ => ys.len(),
    };
    if n == 0 {
        return utilization_for(spec).min(plan_duty);
    }
    (spec.count / n as f64).min(1.0).min(plan_duty)
}

/// A tile's outflow after its own input-inserter pickups take their share
/// (#519): `consumed` — the tile's summed per-inserter demand from
/// `base_demand` — is subtracted proportionally across the two lanes,
/// clamped at zero. This is what flows PAST the tile to its downstream
/// neighbor, so a shared row belt depletes machine by machine and the
/// row's tail sees head supply minus every upstream draw. The stored
/// `lane_rates` value for a tile remains its ARRIVING rate (what its own
/// pickups can see); the subtraction applies only on the way out.
fn outflow_after_pickup(rates: [f64; 2], consumed: f64) -> [f64; 2] {
    let total = rates[0] + rates[1];
    if consumed <= 0.0 || total <= 0.0 {
        return rates;
    }
    let scale = ((total - consumed).max(0.0)) / total;
    [rates[0] * scale, rates[1] * scale]
}

/// Whether `pos`'s feeder list contains a straight feeder (ft == 0). Used
/// by [`lane_transfer`] callers to disambiguate sideload from turn.
fn has_straight_feeder(
    pos: (i32, i32),
    feeders: &FxHashMap<(i32, i32), Vec<((i32, i32), u8)>>,
) -> bool {
    feeders
        .get(&pos)
        .is_some_and(|fs| fs.iter().any(|(_, ft)| *ft == 0))
}

/// Pull-direction lane transfer: given a feeder at `fp` with rates `fr`
/// flowing into receiver `pos`, return the per-lane contribution that lands
/// on `pos`. Used by the iterative convergence pass in
/// [`compute_lane_rates_impl`] which needs to recompute each tile's rate
/// from current upstream rates without the side effects of [`do_propagate`].
fn feeder_contribution(
    fp: (i32, i32),
    pos: (i32, i32),
    fr: [f64; 2],
    feeders: &FxHashMap<(i32, i32), Vec<((i32, i32), u8)>>,
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    consumption: &FxHashMap<(i32, i32), f64>,
) -> [f64; 2] {
    let fd = match belt_dir_map.get(&fp) {
        Some(&d) => d,
        None => return [0.0, 0.0],
    };
    let pd = match belt_dir_map.get(&pos) {
        Some(&d) => d,
        None => return [0.0, 0.0],
    };
    // The feeder's own pickups eat before anything leaves it (#519).
    let fr = outflow_after_pickup(fr, consumption.get(&fp).copied().unwrap_or(0.0));
    lane_transfer(fp, fd, fr, pos, pd, has_straight_feeder(pos, feeders))
}

/// Sum every feeder's contribution into `pos`.  Wrapper over
/// [`feeder_contribution`] that walks `pos`'s feeder list.
fn feeder_contributions_for_tile(
    pos: (i32, i32),
    rates: &FxHashMap<(i32, i32), [f64; 2]>,
    feeders: &FxHashMap<(i32, i32), Vec<((i32, i32), u8)>>,
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    consumption: &FxHashMap<(i32, i32), f64>,
) -> [f64; 2] {
    let Some(my_feeders) = feeders.get(&pos) else {
        return [0.0, 0.0];
    };
    let mut total = [0.0, 0.0];
    for &(fp, _ft) in my_feeders {
        let fr = rates.get(&fp).copied().unwrap_or([0.0, 0.0]);
        let contrib = feeder_contribution(fp, pos, fr, feeders, belt_dir_map, consumption);
        total[0] += contrib[0];
        total[1] += contrib[1];
    }
    total
}

fn do_propagate(
    tile: (i32, i32),
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    feeders: &FxHashMap<(i32, i32), Vec<((i32, i32), u8)>>,
    splitter_sibling: &FxHashMap<(i32, i32), (i32, i32)>,
    in_degree: &mut FxHashMap<(i32, i32), i32>,
    queue: &mut VecDeque<(i32, i32)>,
    lane_rates: &mut FxHashMap<(i32, i32), [f64; 2]>,
    consumption: &FxHashMap<(i32, i32), f64>,
) {
    let d = match belt_dir_map.get(&tile) {
        Some(&d) => d,
        None => return,
    };
    let (ddx, ddy) = dir_to_vec(d);
    let downstream = (tile.0 + ddx, tile.1 + ddy);
    if !belt_dir_map.contains_key(&downstream) {
        return;
    }

    // This tile's pickups eat before anything moves on (#519).
    let my_rates = outflow_after_pickup(
        *lane_rates.get(&tile).unwrap_or(&[0.0, 0.0]),
        consumption.get(&tile).copied().unwrap_or(0.0),
    );
    let ds_d = belt_dir_map[&downstream];
    // Head-on: the downstream tile faces us — items cannot enter a belt's
    // front, so nothing transfers (mirror of the feeder-builder guard,
    // #632 B5). The Jacobi convergence pass would overwrite the push-side
    // rates anyway, but leaving the push path head-on-blind was a latent
    // landmine for any future flow that skips that pass (bot review).
    // Returning BEFORE the in_degree/sibling bookkeeping below is the
    // consistent half of the pair: the feeder map (and therefore
    // `in_degree`) no longer counts head-on edges, so decrementing here
    // would over-decrement an edge that was never counted and dequeue
    // the downstream tile early.
    {
        let (dsx, dsy) = dir_to_vec(ds_d);
        if (dsx, dsy) == (-ddx, -ddy) {
            return;
        }
    }
    let contrib = lane_transfer(
        tile,
        d,
        my_rates,
        downstream,
        ds_d,
        has_straight_feeder(downstream, feeders),
    );
    let ds_rates = lane_rates.entry(downstream).or_insert([0.0, 0.0]);
    ds_rates[0] += contrib[0];
    ds_rates[1] += contrib[1];

    let deg = in_degree.entry(downstream).or_insert(0);
    *deg -= 1;
    let ready = *deg <= 0;
    if ready {
        queue.push_back(downstream);
    }
    // If downstream is one half of a splitter, also decrement its sibling so
    // both tiles reach zero in lockstep (see in_degree unification above).
    if let Some(&sib) = splitter_sibling.get(&downstream) {
        let sib_deg = in_degree.entry(sib).or_insert(0);
        *sib_deg -= 1;
        if *sib_deg <= 0 {
            queue.push_back(sib);
        }
    }
}

// ---------------------------------------------------------------------------
/// Check that the belt rate arriving at each consumer's input inserter pickup
/// point meets the machine's required input rate.
///
/// Uses the same lane rate propagation as `check_lane_throughput` (topological
/// sort with splitter 50/50 handling) but instead of checking capacity, checks
/// that the delivered rate matches what the machine needs.
pub fn check_input_rate_delivery(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    check_input_rate_delivery_with(layout, solver, &compute_lane_rates_impl(layout, solver))
}

/// [`check_input_rate_delivery`] against a caller-supplied rate map — see
/// [`check_lane_throughput_with`].
pub fn check_input_rate_delivery_with(
    layout: &LayoutResult,
    solver: Option<&SolverResult>,
    lane_rates: &FxHashMap<(i32, i32), [f64; 2]>,
) -> Vec<ValidationIssue> {
    let sr = match solver {
        Some(s) => s,
        None => return Vec::new(),
    };

    if lane_rates.is_empty() {
        return Vec::new();
    }

    let machine_tiles_set = build_machine_tile_set(layout);
    let machine_by_tile = build_machine_by_tile(layout);

    let recipe_to_spec: FxHashMap<&str, &crate::models::MachineSpec> = sr
        .machines
        .iter()
        .map(|s| (s.recipe.as_str(), s))
        .collect();
    let mut machine_entity: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    // Physical machine positions per recipe, for `physical_utilization` —
    // but ONLY for recipes owned by a single MachineSpec. Voider/self-loop
    // siblings share a recipe name across specs (recipe_to_spec collapses
    // them last-wins), and pooling every physical machine against one
    // sibling's count wrecks the ratio (uranium fixtures regressed until
    // this guard).
    let mut spec_count_by_recipe: FxHashMap<&str, usize> = FxHashMap::default();
    for spec in &sr.machines {
        *spec_count_by_recipe.entry(spec.recipe.as_str()).or_insert(0) += 1;
    }
    let mut machine_ys_by_recipe: FxHashMap<&str, Vec<i32>> = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            machine_entity.insert((e.x, e.y), e);
            if let Some(r) = e.recipe.as_deref() {
                if spec_count_by_recipe.get(r).copied().unwrap_or(0) == 1 {
                    machine_ys_by_recipe.entry(r).or_default().push(e.y);
                }
            }
        }
    }
    // Recipes whose effective_rows bands carry REAL per-row counts (they
    // sum to the recipe total, within rounding) — see
    // `physical_utilization`'s scope rule.
    let per_row_count_recipes: FxHashSet<&str> = {
        let mut band_count_sum: FxHashMap<&str, f64> = FxHashMap::default();
        for row in &layout.effective_rows {
            *band_count_sum.entry(row.spec.recipe.as_str()).or_insert(0.0) += row.spec.count;
        }
        band_count_sum
            .into_iter()
            .filter(|(r, sum)| {
                recipe_to_spec
                    .get(*r)
                    .is_some_and(|s| (s.count - sum).abs() < 1e-6)
            })
            .map(|(r, _)| r)
            .collect()
    };

    let mut belt_carries: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_carries.insert((e.x, e.y), e.carries.clone());
            if is_splitter(&e.name) {
                belt_carries.insert(splitter_second_tile(e), e.carries.clone());
            }
        }
    }

    let mut issues = Vec::new();

    // First pass: collect input inserters and count how many feed each (machine, item) pair.
    struct InputInserter {
        pickup_pos: (i32, i32),
        machine_pos: (i32, i32),
        carried_item: String,
    }
    let mut inserters: Vec<InputInserter> = Vec::new();
    let mut inserter_count: FxHashMap<((i32, i32), String), usize> = FxHashMap::default();

    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);

        if !machine_tiles_set.contains(&drop_pos) {
            continue;
        }
        let mpos = match machine_by_tile.get(&drop_pos) {
            Some(&p) => p,
            None => continue,
        };
        let carried_item = match belt_carries.get(&pickup_pos).and_then(|c| c.as_deref()) {
            Some(i) => i.to_string(),
            None => continue,
        };
        *inserter_count.entry((mpos, carried_item.clone())).or_insert(0) += 1;
        inserters.push(InputInserter {
            pickup_pos,
            machine_pos: mpos,
            carried_item,
        });
    }

    // DI bridge deliveries: a direct-insertion bridge inserter drops the
    // coupled item onto the consumer's input belt in place of a bus lane.
    // Credit its REAL throughput (at the declared capacity) so the belt is
    // not seen as unfed — but at the actual rate, so an under-provisioned
    // bridge (a single reach-2 long-handed inserter cannot sustain a
    // high-rate coupling like copper-cable) still warns honestly instead of
    // being rubber-stamped.
    // The credit follows the BELT, not just the drop tile: a bridge drops
    // upstream of the consumer inserters' pickup tiles and the belt carries
    // the items down to them (on the cable→EC row the bridge drops at x=5
    // on an east-flowing belt whose pickups are x=6,9,…). So each bridge
    // credits every tile downstream of its drop. Approximation, stated:
    // consumption by an upstream machine is not subtracted, so this is an
    // upper bound on what arrives at a downstream tile — the same
    // simplification the surrounding lane-rate comparison makes.
    let mut di_bridge_delivery: FxHashMap<((i32, i32), String), f64> = FxHashMap::default();
    {
        let di_bridges: Vec<&PlacedEntity> = layout
            .entities
            .iter()
            .filter(|e| {
                is_inserter(&e.name) && super::is_di_bridge_inserter(e.segment_id.as_deref())
            })
            .collect();
        if !di_bridges.is_empty() {
            let belt_dir_map = belt_dir_map_from(&layout.entities);
            let ug_pairs = build_ug_pairs(layout);
            let splitter_siblings = build_splitter_siblings(layout);
            for ins in di_bridges {
                let Some(item) = ins.carries.clone() else {
                    continue;
                };
                let (dx, dy) = dir_to_vec(ins.direction);
                let reach = inserter_reach(&ins.name);
                let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
                let rate = crate::common::machine_feed_rate(
                    &ins.name,
                    ins.quality.unwrap_or_default(),
                    layout.inserter_capacity,
                );
                let starts: FxHashSet<(i32, i32)> = std::iter::once(drop_pos).collect();
                let reached = bfs_belt_downstream(
                    &starts,
                    &belt_dir_map,
                    Some(&ug_pairs),
                    Some(&splitter_siblings),
                );
                // `bfs_belt_downstream` already includes the start tile when
                // it is a belt; when the drop is not onto a belt at all the
                // set is empty and this bridge credits no belt (correct).
                for tile in &reached {
                    *di_bridge_delivery.entry((*tile, item.clone())).or_insert(0.0) += rate;
                }
            }
        }
    }

    // Second pass: check each inserter's available rate vs its share of the required rate.
    let pickup_splitter_siblings = build_splitter_siblings(layout);
    for ins in &inserters {
        let me = match machine_entity.get(&ins.machine_pos) {
            Some(e) => e,
            None => continue,
        };
        let recipe = match me.recipe.as_deref() {
            Some(r) => r,
            None => continue,
        };
        let fallback_spec = match recipe_to_spec.get(recipe) {
            Some(s) => *s,
            None => continue,
        };
        // Position-resolved via `effective_rows` — see
        // `super::resolve_row_spec`'s doc comment for the
        // partition-sibling rationale (`docs/rfc-inserter-sizing.md`
        // Phase 1 finding).
        let (spec, band) = super::resolve_row_spec_banded(layout, recipe, me.y, fallback_spec);
        // spec.inputs[].rate is the per-machine input rate at full
        // utilization; scale by the PHYSICAL utilization (planned count over
        // machines actually placed in the spec's scope) or the check is too
        // strict — up to 10× for a fractional machine (sulfuric-acid at 5/s
        // wants 0.1 machines running at 10% speed), 1/16 for chain-replicated
        // rows (see `physical_utilization`).
        let utilization = physical_utilization(spec, fallback_spec, band, &machine_ys_by_recipe, &per_row_count_recipes);
        let required_rate = spec
            .inputs
            .iter()
            .find(|i| i.item == ins.carried_item)
            .map(|i| i.rate * utilization)
            .unwrap_or(0.0);
        if required_rate <= 0.0 {
            continue;
        }
        let count = inserter_count.get(&(ins.machine_pos, ins.carried_item.clone())).copied().unwrap_or(1);
        let per_inserter_rate = required_rate / count as f64;

        let mut available = match lane_rates.get(&ins.pickup_pos) {
            Some(&[left, right]) => left + right,
            None => 0.0,
        };
        // A pickup ON a splitter tile draws from the pair's whole stream:
        // the walker's demand-aware allocation can route ~all flow to the
        // other half (an output blocked by a pole models ≈0 on this half),
        // but the inserter physically picks items traversing the splitter
        // regardless of exit branch (#624 residue: 26 long-handed feeders
        // read 0.0/s on a donor cell the sim measured at full plan). Both
        // walker phases preserve pair-sum == pooled input, so the pooled
        // read is convention-consistent. Known optimism, recorded: the
        // pair's own pickups are not debited from branch flows — the same
        // upper-bound class as the DI-bridge credit below.
        if let Some(&sib) = pickup_splitter_siblings.get(&ins.pickup_pos) {
            if let Some(&[l2, r2]) = lane_rates.get(&sib) {
                available += l2 + r2;
            }
        }
        // A DI bridge feeding this input belt adds its delivery on top of any
        // bus-lane rate (usually there is no lane — the DI'd item skips the
        // bus). Under-provisioned bridges still fall short and warn.
        if let Some(&r) = di_bridge_delivery.get(&(ins.pickup_pos, ins.carried_item.clone())) {
            available += r;
        }

        if available < per_inserter_rate - 0.02 {
            issues.push(ValidationIssue::with_pos(
                Severity::Warning,
                "input-rate-delivery",
                format!(
                    "Input belt at ({},{}) delivers {:.1}/s but machine needs {:.1}/s of {} (across {} inserter{})",
                    ins.pickup_pos.0, ins.pickup_pos.1, available, required_rate, ins.carried_item,
                    count, if count > 1 { "s" } else { "" }
                ),
                ins.pickup_pos.0,
                ins.pickup_pos.1,
            )
            // The check compares per-inserter, so that's the structured pair
            // (the prose prints the machine-total `required_rate` instead).
            .with_detail(available, per_inserter_rate));
        }
    }

    issues
}

// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, ItemFlow, LayoutResult, MachineSpec, PlacedEntity,
                       SolverResult};

    fn belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".to_string(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn belt_carries(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".to_string(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: None,
            carries: Some(item.to_string()),
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn inserter(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "inserter".to_string(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn machine(x: i32, y: i32, recipe: &str) -> PlacedEntity {
        PlacedEntity {
            name: "assembling-machine-1".to_string(),
            x,
            y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            io_type: None,
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn ug_belt(x: i32, y: i32, dir: EntityDirection, io_type: &str) -> PlacedEntity {
        PlacedEntity {
            name: "underground-belt".to_string(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: Some(io_type.to_string()),
            carries: None,
            mirror: false,
            segment_id: None,
            ..Default::default()
        }
    }

    fn simple_solver(input_rate: f64, output_rate: f64) -> SolverResult {
        SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-gear-wheel".to_string(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: 1.0,
                inputs: vec![ItemFlow {
                    item: "iron-plate".to_string(),
                    rate: input_rate,
                    is_fluid: false,
                    module_id: 0,
                }],
                outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: output_rate,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: input_rate,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: output_rate,
                is_fluid: false,
                module_id: 0,
            }],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-gear-wheel".to_string()],
            ..Default::default()
        }
    }

    // --- belt_dir_map_from ---

    #[test]
    fn belt_dir_map_surface_belt() {
        let e = belt(3, 5, EntityDirection::East);
        let map = belt_dir_map_from(&[e]);
        assert_eq!(map.get(&(3, 5)), Some(&EntityDirection::East));
    }

    #[test]
    fn belt_dir_map_splitter_expands() {
        let sp = PlacedEntity {
            name: "splitter".to_string(),
            x: 2,
            y: 4,
            direction: EntityDirection::North,
            ..Default::default()
        };
        let map = belt_dir_map_from(&[sp]);
        assert!(map.contains_key(&(2, 4)));
        assert!(map.contains_key(&(3, 4))); // second tile (North/South → x+1)
    }

    // --- head-on belts (B5, #632) ---

    /// Two belts facing each other transfer NOTHING — items cannot enter
    /// a belt's front face. The feeder builder used to classify a
    /// head-on neighbor as a sideload (its left-vector dot is 0), making
    /// the pair MUTUAL feeders: a gain-1 cycle the Jacobi convergence
    /// pass grew by one seed per sweep up to the iteration budget —
    /// 4,192/s lane readings on a 4/s seed in production DI fixtures.
    /// This pins both halves: no amplification on the seeded run, zero
    /// flow on the opposing belt.
    #[test]
    fn head_on_belts_are_not_feeders() {
        let layout = LayoutResult {
            entities: vec![
                machine(0, 0, "iron-gear-wheel"),
                inserter(3, 1, EntityDirection::East),
                belt_carries(4, 1, EntityDirection::East, "iron-gear-wheel"),
                belt_carries(5, 1, EntityDirection::East, "iron-gear-wheel"),
                // Head-on partner: faces WEST, directly abutting the
                // eastbound run's front — the production shape was a DI
                // cell's output run meeting a ghost return belt.
                belt_carries(6, 1, EntityDirection::West, "iron-gear-wheel"),
            ],
            width: 10,
            height: 5,
            ..Default::default()
        };
        let sr = simple_solver(2.0, 10.0);
        let rates = compute_lane_rates(&layout, Some(&sr));

        let max_lane = rates
            .values()
            .flat_map(|r| r.iter().copied())
            .fold(0.0f64, f64::max);
        assert!(
            max_lane <= 10.0 + 1e-6,
            "head-on adjacency amplified flow: max lane {max_lane:.1}/s on a 10.0/s seed \
             (pre-fix this read seed x iteration-budget)"
        );
        let opposing = rates.get(&(6, 1)).copied().unwrap_or([0.0, 0.0]);
        assert!(
            opposing[0].abs() < 1e-6 && opposing[1].abs() < 1e-6,
            "the head-on opposing belt must receive nothing, got {opposing:?}"
        );
        let dropped: f64 = rates.get(&(4, 1)).map(|r| r[0] + r[1]).unwrap_or(0.0);
        assert!(
            (dropped - 10.0).abs() < 1e-6,
            "the seeded drop tile must still carry the full injection, got {dropped:.2}"
        );
    }

    /// The cap lookup must rate splitter tiles at the SPLITTER's tier.
    /// The splitter arm was missing from check_lane_throughput's
    /// belt_name_map (present in belt_structural's twin), so both tiles
    /// of any splitter fell back to the yellow 7.5/s default — a fast
    /// splitter lawfully carrying 12/s per lane read as over-cap. That
    /// false positive refused the sim-measured 1.10/s big-electric-pole
    /// layout on dispatch swap (#632 B5). Both directions pinned: the
    /// fast splitter at 12/s per lane is clean, and the same flow onto
    /// a genuine yellow belt still flags.
    #[test]
    fn splitter_tiles_rate_at_splitter_tier() {
        let mk = |splitter: bool| {
            let mut ents = vec![
                machine(0, 0, "iron-gear-wheel"),
                inserter(3, 1, EntityDirection::East),
            ];
            if splitter {
                ents.push(PlacedEntity {
                    name: "fast-splitter".to_string(),
                    x: 4,
                    y: 1,
                    direction: EntityDirection::East,
                    carries: Some("iron-gear-wheel".to_string()),
                    ..Default::default()
                });
            } else {
                ents.push(belt_carries(4, 1, EntityDirection::East, "iron-gear-wheel"));
            }
            LayoutResult { entities: ents, width: 10, height: 5, ..Default::default() }
        };
        // A single inserter drops onto ONE lane. Splitter tiles record
        // POST-SPLIT rates (16/s in -> 8/8 across the two branches on
        // that lane), so 8/s per splitter tile: legal on fast hardware
        // (15/s per lane), over-cap on a yellow fallback (7.5/s). On
        // the plain-belt variant the full 16/s sits on one yellow lane.
        let sr = simple_solver(2.0, 16.0);

        let clean = check_lane_throughput(&mk(true), Some(&sr));
        assert!(
            clean.is_empty(),
            "8/s post-split per lane through a FAST splitter is legal; got {clean:?}"
        );
        let flagged = check_lane_throughput(&mk(false), Some(&sr));
        assert!(
            !flagged.is_empty(),
            "16/s on one YELLOW lane must still flag over-cap"
        );
    }

    /// UG INPUT tiles must rate at the UG's surface tier too. The walker
    /// deliberately copies upstream rates onto UG-in tiles (so inserter
    /// pickups resolve there), but the cap map covered UG OUTPUTS only —
    /// a fast UG entry lawfully carrying 12/s per lane rated against the
    /// yellow 7.5/s fallback (#632 B5, the stacking family's
    /// 30.0-vs-15 false positives).
    #[test]
    fn ug_input_tiles_rate_at_ug_tier() {
        let mk = |ug_name: &str| LayoutResult {
            entities: vec![
                machine(0, 0, "iron-gear-wheel"),
                inserter(3, 1, EntityDirection::East),
                {
                    let mut b = belt_carries(4, 1, EntityDirection::East, "iron-gear-wheel");
                    b.name = "fast-transport-belt".to_string();
                    b
                },
                PlacedEntity {
                    name: ug_name.to_string(),
                    x: 5,
                    y: 1,
                    direction: EntityDirection::East,
                    io_type: Some("input".to_string()),
                    carries: Some("iron-gear-wheel".to_string()),
                    ..Default::default()
                },
                PlacedEntity {
                    name: ug_name.to_string(),
                    x: 8,
                    y: 1,
                    direction: EntityDirection::East,
                    io_type: Some("output".to_string()),
                    carries: Some("iron-gear-wheel".to_string()),
                    ..Default::default()
                },
            ],
            width: 12,
            height: 5,
            ..Default::default()
        };
        // 12/s on one lane: legal on fast hardware, over the yellow cap.
        let sr = simple_solver(2.0, 12.0);
        let clean = check_lane_throughput(&mk("fast-underground-belt"), Some(&sr));
        assert!(
            clean.is_empty(),
            "12/s per lane through a FAST UG entry is legal; got {clean:?}"
        );
        let flagged = check_lane_throughput(&mk("underground-belt"), Some(&sr));
        assert!(
            !flagged.is_empty(),
            "12/s per lane through a YELLOW UG must still flag over-cap"
        );
    }

    /// A UG OUTPUT whose pair inherits flow must not ALSO count as an
    /// external-input graph source. It has no surface feeder, so the
    /// source scan picked it up; the phantom source then (a) broke the
    /// item's demand attribution (Σ over the real trunks + phantoms no
    /// longer reconciles with the solver total, forcing the even-split
    /// fallback that under-seeds every real trunk) and (b) double-counted
    /// at the exit — seed + pair inheritance (#644: two crossing exits on
    /// stress-ec30 read 18/s out of a 9/s-in tunnel, and every tap/row
    /// tile downstream of a crossing flagged over-cap on a layout whose
    /// true lane rates sit exactly AT the yellow cap).
    #[test]
    fn ug_exit_with_fed_pair_is_not_an_external_source() {
        let mut ug_in = ug_belt(3, 0, EntityDirection::East, "input");
        ug_in.carries = Some("iron-plate".to_string());
        let mut ug_out = ug_belt(6, 0, EntityDirection::East, "output");
        ug_out.carries = Some("iron-plate".to_string());
        let layout = LayoutResult {
            entities: vec![
                belt_carries(0, 0, EntityDirection::East, "iron-plate"),
                belt_carries(1, 0, EntityDirection::East, "iron-plate"),
                belt_carries(2, 0, EntityDirection::East, "iron-plate"),
                ug_in,
                ug_out,
                belt_carries(7, 0, EntityDirection::East, "iron-plate"),
                belt_carries(8, 0, EntityDirection::East, "iron-plate"),
            ],
            width: 12,
            height: 3,
            ..Default::default()
        };
        // simple_solver's external input is iron-plate at `input_rate`.
        let sr = simple_solver(12.0, 1.0);
        let rates = compute_lane_rates(&layout, Some(&sr));

        let total = |pos: (i32, i32)| rates.get(&pos).map(|r| r[0] + r[1]).unwrap_or(0.0);
        // The run's head must carry the FULL external rate: with the
        // phantom exit source, the even split halved it (6.0 here).
        assert!(
            (total((0, 0)) - 12.0).abs() < 1e-6,
            "head of the external run must seed the full 12/s, got {:.2}",
            total((0, 0))
        );
        // Conservation through the pair: no pickups anywhere, so the
        // exit-side tiles must carry exactly what the head carries.
        assert!(
            (total((8, 0)) - total((0, 0))).abs() < 1e-6,
            "flow must be conserved through a straight UG pair: head {:.2} vs tail {:.2}",
            total((0, 0)),
            total((8, 0))
        );
    }

    /// The guard's boundary: an ORPHANED UG exit (no paired entrance in
    /// the graph — flow genuinely enters the layout underground) has no
    /// inheritance to double-count and MUST keep seeding as a source.
    #[test]
    fn orphan_ug_exit_still_seeds_as_external_source() {
        let mut ug_out = ug_belt(6, 0, EntityDirection::East, "output");
        ug_out.carries = Some("iron-plate".to_string());
        let layout = LayoutResult {
            entities: vec![
                ug_out,
                belt_carries(7, 0, EntityDirection::East, "iron-plate"),
                belt_carries(8, 0, EntityDirection::East, "iron-plate"),
            ],
            width: 12,
            height: 3,
            ..Default::default()
        };
        let sr = simple_solver(12.0, 1.0);
        let rates = compute_lane_rates(&layout, Some(&sr));
        let total = |pos: (i32, i32)| rates.get(&pos).map(|r| r[0] + r[1]).unwrap_or(0.0);
        assert!(
            (total((8, 0)) - 12.0).abs() < 1e-6,
            "an orphan UG exit is the run's only source and must seed the full 12/s, got {:.2}",
            total((8, 0))
        );
    }

    // --- build_ug_pairs ---

    #[test]
    fn ug_pairs_basic_east() {
        let layout = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(3, 0, EntityDirection::East, "output"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let pairs = build_ug_pairs(&layout);
        assert_eq!(pairs.get(&(0, 0)), Some(&(3, 0)));
        assert_eq!(pairs.get(&(3, 0)), Some(&(0, 0)));
    }

    #[test]
    fn ug_pairs_no_match_different_direction() {
        let layout = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(3, 0, EntityDirection::West, "output"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let pairs = build_ug_pairs(&layout);
        assert!(pairs.is_empty());
    }

    // --- bfs_belt_reach ---

    #[test]
    fn bfs_belt_reach_connected() {
        let tiles: FxHashSet<(i32, i32)> =
            [(0, 0), (1, 0), (2, 0)].iter().copied().collect();
        let starts: FxHashSet<(i32, i32)> = [(0, 0)].iter().copied().collect();
        let reached = bfs_belt_reach(&starts, &tiles, None);
        assert_eq!(reached.len(), 3);
    }

    #[test]
    fn bfs_belt_reach_with_ug_jump() {
        let tiles: FxHashSet<(i32, i32)> =
            [(0, 0), (5, 0)].iter().copied().collect();
        let mut ug: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
        ug.insert((0, 0), (5, 0));
        ug.insert((5, 0), (0, 0));
        let starts: FxHashSet<(i32, i32)> = [(0, 0)].iter().copied().collect();
        let reached = bfs_belt_reach(&starts, &tiles, Some(&ug));
        assert_eq!(reached.len(), 2);
    }

    // --- check_belt_connectivity ---

    #[test]
    fn belt_connectivity_inserter_with_belt_ok() {
        // 3x3 machine at (0,0), inserter at (1,-1) SOUTH, belt at (1,-2) extended
        let lr = LayoutResult {
            entities: vec![
                machine(0, 0, "iron-gear-wheel"),
                inserter(1, -1, EntityDirection::South),
                belt_carries(1, -2, EntityDirection::East, "iron-plate"),
                belt_carries(2, -2, EntityDirection::East, "iron-plate"),
            ],
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_connectivity(&lr, None);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    #[test]
    fn belt_connectivity_no_belts_with_machine_error() {
        let lr = LayoutResult {
            entities: vec![machine(0, 0, "iron-gear-wheel")],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_connectivity(&lr, None);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty());
    }

    #[test]
    fn belt_connectivity_inserter_without_belt_error() {
        let lr = LayoutResult {
            entities: vec![
                machine(0, 0, "iron-gear-wheel"),
                inserter(1, -1, EntityDirection::South),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_connectivity(&lr, None);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].category, "belt-connectivity");
    }

    #[test]
    fn belt_connectivity_isolated_single_belt_error() {
        let lr = LayoutResult {
            entities: vec![
                machine(0, 0, "iron-gear-wheel"),
                inserter(1, -1, EntityDirection::South),
                belt(1, -2, EntityDirection::East),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_connectivity(&lr, None);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("isolated"));
    }

    // --- check_belt_flow_path ---

    #[test]
    fn belt_flow_path_connected_to_boundary_ok() {
        let lr = LayoutResult {
            entities: vec![
                machine(5, 5, "iron-gear-wheel"),
                inserter(6, 4, EntityDirection::South),
                belt(6, 3, EntityDirection::East),
                belt(5, 3, EntityDirection::East),
                belt(4, 3, EntityDirection::East),
                belt(3, 3, EntityDirection::East),
            ],
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_flow_path(&lr, None, LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    #[test]
    fn belt_flow_path_disconnected_input_error() {
        let lr = LayoutResult {
            entities: vec![
                machine(10, 10, "iron-gear-wheel"),
                inserter(11, 9, EntityDirection::South),
                belt(11, 8, EntityDirection::East),
                belt(12, 8, EntityDirection::East),
                // Push boundary far
                belt(0, 0, EntityDirection::East),
                belt(30, 30, EntityDirection::East),
            ],
            width: 50,
            height: 50,
            ..Default::default()
        };
        let issues = check_belt_flow_path(&lr, None, LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error && i.category == "belt-flow-path")
            .collect();
        assert_eq!(errors.len(), 1);
    }

    // --- check_belt_throughput ---

    #[test]
    fn belt_throughput_no_overlap_ok() {
        let lr = LayoutResult {
            entities: vec![
                belt(0, 0, EntityDirection::East),
                belt(1, 0, EntityDirection::East),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_belt_throughput(&lr).is_empty());
    }

    #[test]
    fn belt_throughput_overlapping_warning() {
        let lr = LayoutResult {
            entities: vec![
                belt(0, 0, EntityDirection::East),
                belt(0, 0, EntityDirection::South),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_throughput(&lr);
        let warnings: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].category, "belt-throughput");
        assert!(warnings[0].message.contains("2 overlapping"));
    }

    // --- check_belt_junctions ---

    #[test]
    fn belt_junctions_head_on_is_error() {
        let lr = LayoutResult {
            entities: vec![
                belt_carries(0, 0, EntityDirection::East, "iron-plate"),
                belt_carries(1, 0, EntityDirection::West, "iron-plate"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_junctions(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("HEAD-ON")));
    }

    #[test]
    fn belt_junctions_perpendicular_sideload_ok() {
        let lr = LayoutResult {
            entities: vec![
                belt_carries(0, 0, EntityDirection::East, "iron-plate"),
                belt_carries(0, 1, EntityDirection::North, "iron-plate"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_junctions(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn belt_junctions_same_direction_ok() {
        let lr = LayoutResult {
            entities: vec![
                belt_carries(0, 0, EntityDirection::East, "iron-plate"),
                belt_carries(1, 0, EntityDirection::East, "iron-plate"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_belt_junctions(&lr).is_empty());
    }

    #[test]
    fn belt_junctions_different_items_not_checked() {
        let lr = LayoutResult {
            entities: vec![
                belt_carries(0, 0, EntityDirection::East, "iron-plate"),
                belt_carries(1, 0, EntityDirection::West, "copper-plate"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_belt_junctions(&lr).is_empty());
    }

    // --- check_belt_flow_reachability ---

    #[test]
    fn flow_reachability_straight_east_ok() {
        let sr = simple_solver(5.0, 2.5);
        let mut entities = vec![
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            inserter(4, -1, EntityDirection::South),
        ];
        for x in 0..5 {
            entities.push(belt(x, -2, EntityDirection::East));
        }
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_flow_reachability(&lr, Some(&sr), LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("nothing feeds its pickup belt"))
            .collect();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    #[test]
    fn flow_reachability_reversed_belt_fails() {
        let sr = simple_solver(5.0, 2.5);
        let mut entities = vec![
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            inserter(4, -1, EntityDirection::South),
        ];
        for x in 0..5 {
            entities.push(belt(x, -2, EntityDirection::West)); // reversed
        }
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_flow_reachability(&lr, Some(&sr), LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("nothing feeds its pickup belt"))
            .collect();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn flow_reachability_output_dead_end_fails() {
        let sr = simple_solver(5.0, 2.5);
        let mut entities = vec![
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            inserter(4, -1, EntityDirection::South),
        ];
        for x in 0..5 {
            entities.push(belt(x, -2, EntityDirection::East));
        }
        // Output inserter drops onto a NORTH-facing belt (dead-end)
        entities.push(inserter(4, 3, EntityDirection::South));
        entities.push(belt(4, 4, EntityDirection::North));
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_flow_reachability(&lr, Some(&sr), LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("cannot leave"))
            .collect();
        assert_eq!(errors.len(), 1);
    }

    /// Regression for the same-tile false positive on the OUTPUT side (#524
    /// review): a DI bridge picks straight off the producer's own drop tile.
    ///
    /// The tile is in `sink_tiles` but never in the strictly-one-step-upstream
    /// `drains` closure, so a membership test alone reported "items cannot
    /// leave" on a working layout. This is the COMMON shape rather than a corner
    /// case — `stamp_di_bridge`'s pickup column and the producer's own
    /// output-drop column both land at `mx + 1` under the default row geometry —
    /// and because `di_choice` gates on being better on every channel, the
    /// spurious warning would have suppressed correct DI layouts.
    #[test]
    fn flow_reachability_di_bridge_on_own_drop_tile_is_not_stuck() {
        let sr = simple_solver(5.0, 2.5);
        let mut entities = vec![
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            inserter(4, -1, EntityDirection::South), // input: picks (4,-2)
        ];
        for x in 0..5 {
            entities.push(belt(x, -2, EntityDirection::East));
        }
        // Output inserter drops on (4,4); that belt flows nowhere...
        entities.push(inserter(4, 3, EntityDirection::South));
        entities.push(belt(4, 4, EntityDirection::North));
        // ...because a DI bridge lifts straight off it. South-facing at (4,5)
        // picks from (4,4) and carries to the consumer.
        let mut bridge = inserter(4, 5, EntityDirection::South);
        bridge.segment_id = Some("di-bridge:iron-gear-wheel".to_string());
        entities.push(bridge);
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_flow_reachability(&lr, Some(&sr), LayoutStyle::Spaghetti);
        let stuck: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("cannot leave"))
            .collect();
        assert!(
            stuck.is_empty(),
            "a DI bridge picking off the producer's own drop tile is a valid \
             exit: {stuck:?}"
        );
    }

    /// Mirror of the above on the INPUT side: a belt-to-belt lift drops onto the
    /// exact tile another inserter picks from. That tile is a genuine, active
    /// source, but the one-step-in seeding excluded it from `fed`.
    ///
    /// The geometry mirrors `flow_reachability_output_dead_end_fails`
    /// deliberately. A first draft put the machine where its input inserter's
    /// drop tile missed the machine footprint, so the pickup was never
    /// classified as a machine input and never checked — the test passed with
    /// the fix AND with it sabotaged, i.e. it asserted nothing.
    #[test]
    fn flow_reachability_lift_drop_on_pickup_tile_is_fed() {
        let sr = simple_solver(5.0, 2.5);
        let mut entities = vec![
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            // Machine input: picks from (4,-2).
            inserter(4, -1, EntityDirection::South),
        ];
        // (4,-2) is isolated: nothing FLOWS into it. Its only supply is the lift.
        entities.push(belt(4, -2, EntityDirection::South));
        // A fed run along y = -4, reaching the boundary at x = 0.
        for x in 0..5 {
            entities.push(belt(x, -4, EntityDirection::East));
        }
        // Lift at (4,-3) facing South: picks (4,-4) off the fed run, drops (4,-2)
        // — the machine's pickup tile.
        entities.push(inserter(4, -3, EntityDirection::South));
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_belt_flow_reachability(&lr, Some(&sr), LayoutStyle::Spaghetti);
        let unfed: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("nothing feeds its pickup belt at (4,-2)"))
            .collect();
        assert!(
            unfed.is_empty(),
            "a lift dropping onto the pickup tile feeds it: {unfed:?}"
        );
    }

    // --- check_belt_dead_ends: UG output ---
    //
    // Ported to validate::belt_structural::tests (ug_output_dead_end_detected,
    // ug_output_with_receiver_ok) — issue #488. This module's check_belt_dead_ends
    // was a shadowed duplicate (never dispatched from validate/mod.rs) and has
    // been deleted; belt_structural::check_belt_dead_ends is the live check.

    // --- check_underground_belt_pairs ---

    #[test]
    fn ug_pairs_valid_east() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(3, 0, EntityDirection::East, "output"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_underground_belt_pairs(&lr).is_empty());
    }

    #[test]
    fn ug_pairs_unpaired_input_error() {
        let lr = LayoutResult {
            entities: vec![ug_belt(0, 0, EntityDirection::East, "input")],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_pairs(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Unpaired"));
    }

    #[test]
    fn ug_pairs_unpaired_output_error() {
        let lr = LayoutResult {
            entities: vec![ug_belt(5, 0, EntityDirection::East, "output")],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_pairs(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Unpaired"));
    }

    #[test]
    fn ug_pairs_over_range_error() {
        // transport-belt max reach 4, distance 6 → error
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(6, 0, EntityDirection::East, "output"),
            ],
            width: 20,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_pairs(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.iter().any(|e| e.message.contains("exceeds max reach")));
    }

    #[test]
    fn ug_pairs_at_max_range_ok() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(4, 0, EntityDirection::East, "output"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_pairs(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn ug_pairs_wrong_direction_not_paired() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(3, 0, EntityDirection::West, "output"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_pairs(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn ug_pairs_intercepting_warning() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                ug_belt(2, 0, EntityDirection::East, "input"),
                ug_belt(3, 0, EntityDirection::East, "output"),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_pairs(&lr);
        let warnings: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();
        assert!(warnings.iter().any(|w| w.message.contains("intercepts")));
    }

    // --- check_underground_belt_sideloading ---

    #[test]
    fn ug_sideload_same_direction_ok() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "output"),
                belt(1, 0, EntityDirection::East),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_underground_belt_sideloading(&lr).is_empty());
    }

    #[test]
    fn ug_sideload_head_on_error() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "output"),
                belt(1, 0, EntityDirection::West),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_sideloading(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("head-on"));
    }

    #[test]
    fn ug_sideload_perpendicular_ok() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "output"),
                belt(1, 0, EntityDirection::North),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_underground_belt_sideloading(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn ug_sideload_input_ignored() {
        let lr = LayoutResult {
            entities: vec![
                ug_belt(0, 0, EntityDirection::East, "input"),
                belt(1, 0, EntityDirection::West),
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_underground_belt_sideloading(&lr).is_empty());
    }

    // --- check_belt_loops and check_belt_item_isolation ---
    //
    // Both were shadowed duplicates of validate::belt_structural's live
    // checks (never dispatched from validate/mod.rs) — deleted per issue
    // #488's cleanup; belt_structural::tests carries the equivalent coverage
    // (belt_loop_* / item_isolation_* tests).

    // --- check_belt_inserter_conflict ---

    #[test]
    fn inserter_conflict_same_item_ok() {
        let belt_e = PlacedEntity {
            name: "transport-belt".to_string(),
            x: 0,
            y: 1,
            direction: EntityDirection::East,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        };
        let ins = PlacedEntity {
            name: "inserter".to_string(),
            x: 0,
            y: 0,
            direction: EntityDirection::South,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        };
        let lr = LayoutResult {
            entities: vec![belt_e, ins],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_belt_inserter_conflict(&lr).is_empty());
    }

    #[test]
    fn inserter_conflict_different_items_error() {
        // Two inserters dropping onto the same belt tile with different items
        let belt_e = PlacedEntity {
            name: "transport-belt".to_string(),
            x: 0,
            y: 1,
            direction: EntityDirection::East,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        };
        let ins1 = PlacedEntity {
            name: "inserter".to_string(),
            x: 0,
            y: 0,
            direction: EntityDirection::South,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        };
        let ins2 = PlacedEntity {
            name: "inserter".to_string(),
            x: 1,
            y: 1,
            direction: EntityDirection::West,
            carries: Some("copper-plate".to_string()),
            ..Default::default()
        };
        let lr = LayoutResult {
            entities: vec![belt_e, ins1, ins2],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_belt_inserter_conflict(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty());
    }

    // --- check_lane_throughput ---

    #[test]
    fn lane_throughput_single_inserter_within_capacity() {
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-gear-wheel".to_string(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: 1.0,
                inputs: vec![ItemFlow {
                    item: "iron-plate".to_string(),
                    rate: 5.0,
                    is_fluid: false,
                    module_id: 0,
                }],
                outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 2.5,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 5.0,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 2.5,
                is_fluid: false,
                module_id: 0,
            }],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-gear-wheel".to_string()],
            ..Default::default()
        };

        let entities = vec![
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 4,
                y: 3,
                direction: EntityDirection::South,
                ..Default::default()
            },
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 4,
                y: 4,
                direction: EntityDirection::East,
                carries: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 5,
                y: 4,
                direction: EntityDirection::East,
                carries: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
        ];
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_lane_throughput(&lr, Some(&sr));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn lane_throughput_no_solver_returns_empty() {
        let lr = LayoutResult {
            entities: vec![],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_lane_throughput(&lr, None).is_empty());
    }

    // --- check_input_rate_delivery ---

    #[test]
    fn input_rate_delivery_no_solver_returns_empty() {
        let lr = LayoutResult {
            entities: vec![],
            width: 10,
            height: 10,
            ..Default::default()
        };
        assert!(check_input_rate_delivery(&lr, None).is_empty());
    }

    #[test]
    fn input_rate_delivery_sufficient_rate_ok() {
        // Two machines: producer (iron-plate) → belt chain → consumer (iron-gear-wheel).
        // Producer outputs 5/s iron-plate via inserter onto belt.
        // Consumer needs 5/s iron-plate — exactly matched.
        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "electric-furnace".to_string(),
                    recipe: "iron-plate".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-ore".to_string(),
                        rate: 5.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 5.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "assembling-machine-3".to_string(),
                    recipe: "iron-gear-wheel".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 5.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "iron-gear-wheel".to_string(),
                        rate: 2.5,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
            ],
            external_inputs: vec![ItemFlow {
                item: "iron-ore".to_string(),
                rate: 5.0,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 2.5,
                is_fluid: false,
                module_id: 0,
            }],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()],
            ..Default::default()
        };

        // Layout:
        //   Furnace at (0,0), output inserter at (1,3) South, drops onto belt at (1,4).
        //   Belt chain: (1,4) East → (2,4) East → (3,4) East.
        //   Assembler at (5,5), input inserter at (6,5) South picks from belt at (6,4) East.
        //   Wait — need the belt to flow from producer output to consumer input.
        //   Let's keep it simple: output inserter drops at (1,4), belt goes East to (3,4).
        //   But consumer inserter needs to pick from a belt tile in the chain.
        //
        // Simpler: producer at (0,0). Output inserter (1,3) South drops iron-plate at (1,4) East.
        // Belt (1,4)→(2,4)→(3,4) all East carrying iron-plate.
        // Consumer at (5,0). Input inserter at (6,3) South picks from belt at (6,4).
        // Hmm, the belt chain needs to reach (6,4).
        //
        // Even simpler: just one belt tile shared between output and input inserters.
        // Producer output inserter drops at (1,4). Consumer input inserter picks from (1,4).
        // But that's the same tile — inserter drops and picks from same belt.
        //
        // Simplest correct layout:
        // Output inserter at (1,3) South → drops onto belt (1,4) East
        // Belt chain (1,4) → (2,4) → (3,4) all East
        // Input inserter at (2,3) South ← picks from belt (2,4) East
        //   Wait, inserter at (2,3) South picks from (2,3-1)=(2,2) not (2,4).
        //   Inserter at (2,3) South: picks from (2,2), drops to (2,4). That's wrong direction.
        //   For input inserter: picks from belt, drops to machine.
        //   Inserter at (2,5) North: picks from (2,6), drops to (2,4). Picks from belt at (2,6).
        //
        // Let me use the standard bus template pattern:
        // Belt at y=0 East (input belt for assembler row).
        // Inserter at (1,1) South picks from belt at (1,0) drops to machine at (1,2).
        // Machine at (0,2) assembling-machine-3.
        // For the rate to be seeded, we need a PRODUCER output inserter dropping onto (1,0).
        // Producer machine at (0,-3), output inserter at (1,-1) South, drops at (1,0).
        // But the belt at (1,0) is EAST. Output inserter at (1,-1) South drops to (1,0). OK.
        // compute_lane_rates seeds 5/s at (1,0) left lane (inserter is inline with belt).
        // Input inserter at (1,1) South picks from (1,0). Needs 5/s. Available 5/s. OK.
        // Furnace at (0,-4), output inserter at (1,-1) South: drops to (1,0).
        // Single belt at (1,0) East (no chain → in_degree=0, seeded by injection).
        // Input inserter at (1,1) South: picks from (1,0), drops to (1,2).
        // Assembler at (0,2).
        let entities = vec![
            PlacedEntity {
                name: "electric-furnace".to_string(),
                x: 0,
                y: -4,
                direction: EntityDirection::North,
                recipe: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: -1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 1,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: 1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 0,
                y: 2,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: 5,
                direction: EntityDirection::South,
                carries: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 1,
                y: 6,
                direction: EntityDirection::East,
                carries: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
        ];
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_input_rate_delivery(&lr, Some(&sr));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn input_rate_delivery_insufficient_rate_warns() {
        // Same layout but producer outputs 5/s, consumer needs 20/s.
        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "electric-furnace".to_string(),
                    recipe: "iron-plate".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-ore".to_string(),
                        rate: 5.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 5.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "assembling-machine-3".to_string(),
                    recipe: "iron-gear-wheel".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 20.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "iron-gear-wheel".to_string(),
                        rate: 10.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
            ],
            external_inputs: vec![],
            external_outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 10.0,
                is_fluid: false,
                module_id: 0,
            }],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()],
            ..Default::default()
        };

        let entities = vec![
            PlacedEntity {
                name: "electric-furnace".to_string(),
                x: 0,
                y: -4,
                direction: EntityDirection::North,
                recipe: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: -1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 1,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: 1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 0,
                y: 2,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: 5,
                direction: EntityDirection::South,
                carries: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 1,
                y: 6,
                direction: EntityDirection::East,
                carries: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
        ];
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_input_rate_delivery(&lr, Some(&sr));
        assert!(!issues.is_empty(), "expected warning for insufficient rate");
        assert!(issues.iter().any(|i| i.category == "input-rate-delivery"),
            "expected input-rate-delivery issue, got: {:?}", issues);
        // RFC validation-explainability D1: the warning carries the exact
        // compared pair as structured numbers (delivered < needed).
        let detail = issues
            .iter()
            .find(|i| i.category == "input-rate-delivery")
            .and_then(|i| i.detail.as_ref())
            .expect("input-rate-delivery must carry IssueDetail");
        assert!(
            detail.delivered < detail.needed,
            "detail must reflect the failing comparison: {detail:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // Iterative walker / splitter math regression tests
    //
    // Three focused tests covering the math the iterative walker is supposed to
    // produce. These are contracts: each documents what a specific case should
    // compute, separate from the audit count which is end-to-end.
    // ---------------------------------------------------------------------------

    /// Splitter receiving rate on one half only must produce balanced, lane-
    /// mixed output across both halves. Pre-fix behaviour was "propagate non-
    /// zero half to both" which gave 2× the correct rate at the stuck
    /// secondary; current behaviour averages and lane-mixes via the iterative
    /// pass.
    ///
    /// 1→2 split: feeder full belt `[L=7.5, R=7.5]` → splitter pair → both
    /// halves at `[3.75, 3.75]` (belt total 7.5/s = half of input). Total mass
    /// conserved: 15/s in, 15/s out across two output belts.
    ///
    /// Under the demand-pull model (RFC rfc-lane-demand-flow.md) the outputs
    /// here are bare belts with no downstream machine demand, so the split is
    /// the exact-even *symmetric-residual fallback* — the same `[3.75, 3.75]`
    /// the legacy 50/50 model produced. This pins that the fallback is
    /// byte-identical when demand is absent.
    #[test]
    fn splitter_one_feeder_outputs_balanced_halves() {
        use EntityDirection::*;
        let item = "iron-plate";
        // Source belt at (0, 0) carrying the external input. Splitter at
        // (0, 1)/(1, 1) south-facing. Output belts at (0, 2) and (1, 2).
        let layout = LayoutResult {
            entities: vec![
                belt_carries(0, 0, South, item),
                PlacedEntity {
                    name: "splitter".to_string(),
                    x: 0,
                    y: 1,
                    direction: South,
                    ..Default::default()
                },
                belt(0, 2, South),
                belt(1, 2, South),
            ],
            width: 4,
            height: 4,
            ..Default::default()
        };
        let solver = SolverResult {
            machines: vec![],
            external_inputs: vec![ItemFlow {
                item: item.to_string(),
                rate: 15.0,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        let rates = compute_lane_rates(&layout, Some(&solver));
        let r0 = rates.get(&(0, 1)).copied().unwrap_or([0.0, 0.0]);
        let r1 = rates.get(&(1, 1)).copied().unwrap_or([0.0, 0.0]);
        assert!(
            (r0[0] - 3.75).abs() < 0.01
                && (r0[1] - 3.75).abs() < 0.01
                && (r1[0] - 3.75).abs() < 0.01
                && (r1[1] - 3.75).abs() < 0.01,
            "splitter halves expected [3.75, 3.75] each, got pos={r0:?} sib={r1:?}"
        );
        // Outputs inherit the splitter rate.
        let o0 = rates.get(&(0, 2)).copied().unwrap_or([0.0, 0.0]);
        let o1 = rates.get(&(1, 2)).copied().unwrap_or([0.0, 0.0]);
        assert!(
            (o0[0] + o0[1] + o1[0] + o1[1] - 15.0).abs() < 0.01,
            "total output mass should equal input 15/s, got {o0:?} + {o1:?}"
        );
    }

    /// Priority-output splitter (self-loop row, e.g. kovarex-enrichment-
    /// process): the loop-back branch receives `min(total, loop_priority_rate)`
    /// and the export branch the remainder, NOT the symmetric 50/50 split
    /// that `splitter_one_feeder_outputs_balanced_halves` exercises above.
    ///
    /// Same 1-feeder-into-a-splitter shape as that test (source belt feeds
    /// only the `(0,1)` half; `(1,1)` gets nothing directly), but the
    /// splitter carries `loop_priority_rate: Some(4.0)` and `(0,1)`'s
    /// downstream belt is tagged as the self-loop segment. Total input is
    /// 4.1/s, so the loop branch should settle at ~4.0/s and the export
    /// branch at ~0.1/s — not 2.05/2.05.
    #[test]
    fn splitter_priority_loop_branch_gets_priority_share() {
        use EntityDirection::*;
        let item = "uranium-235";
        let layout = LayoutResult {
            entities: vec![
                belt_carries(0, 0, South, item),
                PlacedEntity {
                    name: "splitter".to_string(),
                    x: 0,
                    y: 1,
                    direction: South,
                    loop_priority_rate: Some(4.0),
                    // Real priority splitters set output_priority at the priority
                    // branch; South splitter with priority on the (0,1) tile →
                    // LANE_RIGHT. The walker now reads this field, not the tag.
                    output_priority: Some(crate::common::LANE_RIGHT.to_string()),
                    ..Default::default()
                },
                // (0,1)'s downstream: tagged self-loop segment.
                PlacedEntity {
                    segment_id: Some(
                        "row:kovarex-enrichment-process:selfloop:uranium-235".to_string(),
                    ),
                    ..belt_carries(0, 2, South, item)
                },
                // (1,1)'s downstream: plain export belt, no tag.
                belt_carries(1, 2, South, item),
            ],
            width: 4,
            height: 4,
            ..Default::default()
        };
        let solver = SolverResult {
            machines: vec![],
            external_inputs: vec![ItemFlow {
                item: item.to_string(),
                rate: 4.1,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        let rates = compute_lane_rates(&layout, Some(&solver));
        let loop_total: f64 = rates.get(&(0, 1)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        let export_total: f64 = rates.get(&(1, 1)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        assert!(
            (loop_total - 4.0).abs() < 0.01,
            "loop branch should get ~4.0/s, got {loop_total}"
        );
        assert!(
            (export_total - 0.1).abs() < 0.01,
            "export branch should get ~0.1/s (not 2.05/2.05 symmetric), got {export_total}"
        );
    }

    /// Merge-and-tap priority tap (RFC `docs/rfc-merge-tap-trunks.md` D4): the
    /// feed branch (downstream tagged `MERGE_TAP_SEGMENT_TAG`) receives
    /// `min(total, loop_priority_rate)` and the trunk continuation the
    /// remainder — the same rate law the self-loop test above exercises, now
    /// lit up by the tap tag instead of `:selfloop:`. Confirms the generalized
    /// priority-branch predicate covers taps in the demand-pull walker.
    #[test]
    fn splitter_tap_branch_gets_priority_share() {
        use crate::common::MERGE_TAP_SEGMENT_TAG;
        use EntityDirection::*;
        let item = "uranium-235";
        let layout = LayoutResult {
            entities: vec![
                belt_carries(0, 0, South, item),
                PlacedEntity {
                    name: "splitter".to_string(),
                    x: 0,
                    y: 1,
                    direction: South,
                    loop_priority_rate: Some(4.0),
                    // Real priority splitters set output_priority at the priority
                    // branch; South splitter with priority on the (0,1) tile →
                    // LANE_RIGHT. The walker now reads this field, not the tag.
                    output_priority: Some(crate::common::LANE_RIGHT.to_string()),
                    ..Default::default()
                },
                // (0,1)'s downstream: tagged merge-tap feed branch.
                PlacedEntity {
                    segment_id: Some(format!("family:uranium-235{MERGE_TAP_SEGMENT_TAG}0")),
                    ..belt_carries(0, 2, South, item)
                },
                // (1,1)'s downstream: trunk continuation, no tag.
                belt_carries(1, 2, South, item),
            ],
            width: 4,
            height: 4,
            ..Default::default()
        };
        let solver = SolverResult {
            machines: vec![],
            external_inputs: vec![ItemFlow {
                item: item.to_string(),
                rate: 4.1,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        let rates = compute_lane_rates(&layout, Some(&solver));
        let feed_total: f64 = rates.get(&(0, 1)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        let cont_total: f64 = rates.get(&(1, 1)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        assert!(
            (feed_total - 4.0).abs() < 0.01,
            "feed branch should get ~4.0/s, got {feed_total}"
        );
        assert!(
            (cont_total - 0.1).abs() < 0.01,
            "continuation should get ~0.1/s (not 2.05/2.05 symmetric), got {cont_total}"
        );
    }

    /// (3, 3) library template at full saturation should converge to exactly
    /// `[7.5, 7.5]` per lane on every internal belt. Pre-iterative-walker, the
    /// internal feedback splitter at `(0, 5)/(1, 5)` settled at `[3.75, 3.75]`
    /// vs `[9.375, 9.375]` (1.25× capacity) because the single-pass walker
    /// hit the feedback loop before the upstream half had stabilised.
    ///
    /// This is the headline regression case for the iterative pass.
    #[test]
    fn iterative_walker_balances_3_3_template() {
        use crate::bus::balancer_classify::BalancerTemplateRef;
        use crate::bus::balancer_library::balancer_templates;
        use crate::bus::template_validate::compute_template_lane_rates;

        let templates = balancer_templates();
        let t = templates
            .get(&(3, 3))
            .expect("(3, 3) template missing from library");
        let rates = compute_template_lane_rates(BalancerTemplateRef::from(t));

        for (&pos, &[l, r]) in &rates {
            assert!(
                (l - 7.5).abs() < 0.01 && (r - 7.5).abs() < 0.01,
                "(3, 3) tile {pos:?} expected [7.5, 7.5], got [{l:.4}, {r:.4}]"
            );
        }
    }

    /// UG-output rate inherits from the surface tile behind the paired UG-
    /// input AND preserves any inserter injection on its own surface tile.
    /// Bug caught during dev: the iterative pass was REPLACING `next[ug_out]`
    /// with `behind`, dropping any `seed_rates[ug_out]` from inserter drops.
    /// Fix: `next[ug_out] = seed[ug_out] + behind`.
    ///
    /// Setup: an inserter drops 1.0/s of `iron-plate` onto the surface of a
    /// UG-output that's also carrying behind's rate via its underground feed.
    /// The UG-output's effective rate must include both contributions.
    #[test]
    fn ug_output_preserves_inserter_injection() {
        use EntityDirection::*;
        let item = "iron-plate";
        // Layout: source belt (0, 0) feeds UG-input (0, 1) south; pairs to
        // UG-output at (0, 4) south; output belt (0, 5). Machine at (2, 4)
        // making `iron-gear-wheel` from the picked-up plates; inserter at
        // (1, 4) drops gears onto the UG-output's surface (0, 4).
        //
        // Wait — we need the inserter to drop onto the UG-out, but the
        // injection is keyed off `belt_carries.get(drop_pos)` matching the
        // machine's *output* item. Use a single item `iron-plate` for both
        // the surface flow AND the inserter drop to keep the test focused on
        // UG-out accumulation rather than item-mixing.
        //
        // The simplest faithful setup: build a `lane_injections` directly via
        // the inserter+machine plumbing in `compute_lane_rates_impl`. Source
        // belt carries `iron-plate` at 7.5/s (half belt to leave headroom for
        // injection). Inserter long-handed into UG-out from a machine that
        // outputs `iron-plate`.
        // UG-out must have `carries` set for the inserter-drop logic to
        // recognise the drop_pos as carrying the right item; without it the
        // injection is silently skipped.
        let ug_out_with_item = PlacedEntity {
            name: "underground-belt".to_string(),
            x: 0,
            y: 4,
            direction: South,
            io_type: Some("output".to_string()),
            carries: Some(item.to_string()),
            ..Default::default()
        };
        let layout = LayoutResult {
            entities: vec![
                belt_carries(0, 0, South, item),
                ug_belt(0, 1, South, "input"),
                ug_out_with_item,
                belt_carries(0, 5, South, item),
                // Machine producing iron-plate; inserter long-handed onto UG-out.
                machine(2, 4, "iron-plate-recycle"),
                inserter(1, 4, West),
            ],
            width: 4,
            height: 7,
            ..Default::default()
        };
        let solver = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-plate-recycle".to_string(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: 1.0,
                inputs: vec![],
                outputs: vec![ItemFlow {
                    item: item.to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![ItemFlow {
                item: item.to_string(),
                rate: 7.5,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-plate-recycle".to_string()],
            ..Default::default()
        };
        let rates = compute_lane_rates(&layout, Some(&solver));
        let ug_out = rates.get(&(0, 4)).copied().unwrap_or([0.0, 0.0]);
        let total = ug_out[0] + ug_out[1];
        // Surface inherited rate (3.75 per lane = 7.5/belt) + inserter
        // injection (1.0/s on whichever lane) = 8.5/belt total. Allow some
        // float slack and tolerate the lane the inserter targets.
        assert!(
            total > 8.0 && total < 9.0,
            "UG-out should carry inherited 7.5 + injected 1.0 ≈ 8.5, got {total} ({ug_out:?})"
        );
    }

    // ---------------------------------------------------------------------------
    // Demand-pull splitter model (RFC rfc-lane-demand-flow.md Phase 1 Branch A)
    // ---------------------------------------------------------------------------

    /// Core allocation math. Pins each branch of [`allocate_by_demand`]: the
    /// symmetric/zero-demand fallback (exact even split, byte-identical to the
    /// legacy 50/50 model), demand-met, proportional undersupply, oversupply
    /// spill, and the per-output capacity cap.
    #[test]
    fn allocate_by_demand_branches() {
        let approx = |(a, b): (f64, f64), (ea, eb): (f64, f64)| {
            assert!(
                (a - ea).abs() < 1e-6 && (b - eb).abs() < 1e-6,
                "got ({a}, {b}), expected ({ea}, {eb})"
            );
        };
        // No demand / symmetric demand → exact even split (legacy 50/50).
        approx(allocate_by_demand(4.0, 0.0, 0.0, 15.0), (2.0, 2.0));
        approx(allocate_by_demand(4.0, 2.0, 2.0, 15.0), (2.0, 2.0));
        // Exactly enough to meet both → each gets its demand.
        approx(allocate_by_demand(4.0, 3.0, 1.0, 15.0), (3.0, 1.0));
        // Undersupply → both starve in proportion to demand.
        approx(allocate_by_demand(2.0, 3.0, 1.0, 15.0), (1.5, 0.5));
        // Oversupply → meet both, spill the surplus across remaining room.
        let (oa, ob) = allocate_by_demand(10.0, 3.0, 1.0, 15.0);
        assert!((oa + ob - 10.0).abs() < 1e-6, "surplus conserved: {oa}+{ob}");
        assert!(oa >= 3.0 && ob >= 1.0, "each output keeps at least its demand");
        // Cap binds: with cap 5 and input 20, each output is clamped to 5 and
        // the 10/s over 2×cap is left unrouted (a lane-throughput concern).
        approx(allocate_by_demand(20.0, 3.0, 1.0, 5.0), (5.0, 5.0));
    }

    /// Build a 1→2 splitter feeding two consumer rows with independent
    /// per-machine input demands. Source belt (0,0) → splitter (0,1)/(1,1) →
    /// row A belt (0,2) (machine draws `demand_a`) and row B belt (1,2)
    /// (machine draws `demand_b`). External `iron-plate` supply is `supply`.
    fn two_row_split(supply: f64, demand_a: f64, demand_b: f64) -> (LayoutResult, SolverResult) {
        use EntityDirection::*;
        let item = "iron-plate";
        let layout = LayoutResult {
            entities: vec![
                belt_carries(0, 0, South, item),
                PlacedEntity {
                    name: "splitter".to_string(),
                    x: 0,
                    y: 1,
                    direction: South,
                    ..Default::default()
                },
                belt_carries(0, 2, South, item),
                belt_carries(1, 2, South, item),
                // Machine A (west of row A) + input inserter picking from (0,2).
                machine(-4, 1, "recipe-a"),
                inserter(-1, 2, West),
                // Machine B (east of row B) + input inserter picking from (1,2).
                machine(3, 1, "recipe-b"),
                inserter(2, 2, East),
            ],
            width: 40,
            height: 40,
            ..Default::default()
        };
        let mk = |recipe: &str, rate: f64| MachineSpec {
            entity: "assembling-machine-1".to_string(),
            recipe: recipe.to_string(),
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
            count: 1.0,
            inputs: vec![ItemFlow {
                item: item.to_string(),
                rate,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: format!("out-{recipe}"),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        let solver = SolverResult {
            machines: vec![mk("recipe-a", demand_a), mk("recipe-b", demand_b)],
            external_inputs: vec![ItemFlow {
                item: item.to_string(),
                rate: supply,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        (layout, solver)
    }

    /// Redistributes under backpressure: an even 1→2 balancer feeding rows that
    /// draw 3.0/s and 1.0/s, with aggregate supply (4.0/s) meeting aggregate
    /// demand, routes 3.0/s to the hungry row (not the even-split 2.0/s) and
    /// clears the input-rate-delivery warning that the legacy 50/50 model
    /// raised. This is the headline logistic@1/s false-positive.
    #[test]
    fn demand_pull_redistributes_under_backpressure() {
        let (layout, solver) = two_row_split(4.0, 3.0, 1.0);
        let rates = compute_lane_rates(&layout, Some(&solver));
        let row_a: f64 = rates.get(&(0, 2)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        let row_b: f64 = rates.get(&(1, 2)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        assert!(
            (row_a - 3.0).abs() < 0.05,
            "hungry row should get its 3.0/s demand (not even-split 2.0/s), got {row_a}"
        );
        assert!(
            (row_b - 1.0).abs() < 0.05,
            "low-draw row should get its 1.0/s demand, got {row_b}"
        );
        let warns = check_input_rate_delivery(&layout, Some(&solver));
        assert!(
            warns.is_empty(),
            "backpressure meets both demands → no input-rate-delivery warning: {warns:?}"
        );
    }

    /// True positives survive: when aggregate supply (2.0/s) is genuinely below
    /// aggregate demand (4.0/s), demand-pull starves both rows in proportion and
    /// the input-rate-delivery check still warns — the model doesn't paper over
    /// a real shortfall.
    #[test]
    fn demand_pull_true_starvation_still_warns() {
        let (layout, solver) = two_row_split(2.0, 3.0, 1.0);
        let rates = compute_lane_rates(&layout, Some(&solver));
        let row_a: f64 = rates.get(&(0, 2)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        // 2.0 total, demand 3:1 → 1.5 to row A.
        assert!(
            (row_a - 1.5).abs() < 0.05,
            "under-supplied hungry row gets its proportional 1.5/s, got {row_a}"
        );
        let warns = check_input_rate_delivery(&layout, Some(&solver));
        assert!(
            warns.iter().any(|w| w.category == "input-rate-delivery"),
            "genuine undersupply must still warn, got {warns:?}"
        );
    }

    /// Symmetric residual: two rows with equal demand get an exact even split
    /// (the fallback path), and both are satisfied.
    #[test]
    fn demand_pull_symmetric_rows_split_evenly() {
        let (layout, solver) = two_row_split(4.0, 2.0, 2.0);
        let rates = compute_lane_rates(&layout, Some(&solver));
        let row_a: f64 = rates.get(&(0, 2)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        let row_b: f64 = rates.get(&(1, 2)).copied().unwrap_or([0.0, 0.0]).iter().sum();
        assert!(
            (row_a - 2.0).abs() < 0.05 && (row_b - 2.0).abs() < 0.05,
            "equal-demand rows split evenly, got a={row_a} b={row_b}"
        );
        assert!(check_input_rate_delivery(&layout, Some(&solver)).is_empty());
    }

    /// #519: consumption decrement along a shared row belt. One producer
    /// seeds 6/s of iron-plate onto a belt serving THREE consumers that
    /// need 2.5/s each (7.5 total). The walker must deplete the belt
    /// machine by machine — the first two pickups see enough, the tail
    /// sees 6.0 − 5.0 = 1.0/s and warns. Before the decrement the walker
    /// propagated the undecremented 6.0 to every pickup and per-inserter
    /// comparison passed all three (the ac5-on flux blind spot).
    #[test]
    fn input_rate_delivery_row_tail_starvation_warns() {
        fn sr_with_supply(supply: f64) -> SolverResult {
            SolverResult {
                machines: vec![
                    MachineSpec {
                        entity: "electric-furnace".to_string(),
                        recipe: "iron-plate".to_string(),
                        self_loop: vec![], voider: false, game_modules: Vec::new(),
                        count: 1.0,
                        inputs: vec![],
                        outputs: vec![ItemFlow {
                            item: "iron-plate".to_string(),
                            rate: supply,
                            is_fluid: false,
                            module_id: 0,
                        }],
                    },
                    MachineSpec {
                        entity: "assembling-machine-3".to_string(),
                        recipe: "iron-gear-wheel".to_string(),
                        self_loop: vec![], voider: false, game_modules: Vec::new(),
                        count: 3.0,
                        inputs: vec![ItemFlow {
                            item: "iron-plate".to_string(),
                            rate: 2.5,
                            is_fluid: false,
                            module_id: 0,
                        }],
                        outputs: vec![ItemFlow {
                            item: "iron-gear-wheel".to_string(),
                            rate: 1.25,
                            is_fluid: false,
                            module_id: 0,
                        }],
                    },
                ],
                external_inputs: vec![],
                external_outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 3.75,
                    is_fluid: false,
                    module_id: 0,
                }],
                surplus_outputs: vec![],
                dependency_order: vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()],
                ..Default::default()
            }
        }
        // Producer furnace at (0,-4); output inserter (1,-1)S drops onto the
        // belt head (1,0)E. Belt runs east (1,0)..(7,0). Three consumers at
        // x=0,3,6 (3x3, y=2..4), each with an input inserter at (x+1, 1)S
        // picking from (x+1, 0).
        let mut entities = vec![
            PlacedEntity {
                name: "electric-furnace".to_string(),
                x: 0,
                y: -4,
                direction: EntityDirection::North,
                recipe: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 1,
                y: -1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
        ];
        for x in 1..=7 {
            entities.push(PlacedEntity {
                name: "transport-belt".to_string(),
                x,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            });
        }
        for mx in [0, 3, 6] {
            entities.push(PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: mx,
                y: 2,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                name: "inserter".to_string(),
                x: mx + 1,
                y: 1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            });
        }
        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            ..Default::default()
        };

        // Under-supplied (6.0 vs 7.5): exactly ONE warning, at the TAIL
        // pickup (7,0) — the head machines eat first.
        let issues = check_input_rate_delivery(&lr, Some(&sr_with_supply(6.0)));
        assert_eq!(
            issues.len(),
            1,
            "expected exactly the tail machine to warn: {issues:?}"
        );
        assert_eq!((issues[0].x, issues[0].y), (Some(7), Some(0)), "{issues:?}");

        // Rates themselves deplete along the row: 6.0 at the head pickup,
        // 3.5 at the second, 1.0 arriving at the tail.
        let rates = compute_lane_rates(&lr, Some(&sr_with_supply(6.0)));
        let total = |p: (i32, i32)| rates.get(&p).map(|r| r[0] + r[1]).unwrap_or(0.0);
        assert!((total((1, 0)) - 6.0).abs() < 1e-6, "head {:?}", total((1, 0)));
        assert!((total((4, 0)) - 3.5).abs() < 1e-6, "mid {:?}", total((4, 0)));
        assert!((total((7, 0)) - 1.0).abs() < 1e-6, "tail {:?}", total((7, 0)));

        // Exact supply (7.5 == demand): the knife-edge is CLEAN — the tail
        // receives exactly its share and nothing warns.
        let issues = check_input_rate_delivery(&lr, Some(&sr_with_supply(7.5)));
        assert!(issues.is_empty(), "exact supply must not warn: {issues:?}");
    }

    /// #624 pooled-pair pickup read: the negative case must survive the
    /// credit. A pickup ON a splitter tile reads the pair's pooled
    /// stream — that must CLEAR a pickup whose demand the pooled stream
    /// covers (even when its own half's branch share alone would not),
    /// and must STILL WARN when the pooled stream genuinely falls short.
    #[test]
    fn input_rate_delivery_splitter_pickup_pooled_but_not_rubber_stamped() {
        fn sr_with_supply(supply: f64) -> SolverResult {
            SolverResult {
                machines: vec![MachineSpec {
                    entity: "assembling-machine-3".to_string(),
                    recipe: "iron-gear-wheel".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 2.5,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "iron-gear-wheel".to_string(),
                        rate: 1.25,
                        is_fluid: false,
                        module_id: 0,
                    }],
                }],
                external_inputs: vec![ItemFlow {
                    item: "iron-plate".to_string(),
                    rate: supply,
                    is_fluid: false,
                    module_id: 0,
                }],
                external_outputs: vec![],
                surplus_outputs: vec![],
                dependency_order: vec!["iron-gear-wheel".to_string()],
                ..Default::default()
            }
        }
        // External iron-plate belt (0,0)E feeds a splitter (1,0)+(1,1)E;
        // the input inserter at (2,1)E picks FROM the splitter's second
        // tile (1,1) and drops into the machine at (3,0)..(5,2). The
        // splitter's outputs lead nowhere, so demand-aware allocation has
        // no branch preference and the seed's path to the pickup is the
        // pooled pair itself. (This does NOT pin the phantom-source
        // filter — post-fix, phantom seeding is mass-conserving in a
        // single-source chain, so this fixture's verdicts are identical
        // with or without it; see the dedicated test below.)
        let entities = vec![
            PlacedEntity {
                name: "transport-belt".to_string(),
                x: 0,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "splitter".to_string(),
                x: 1,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 2,
                y: 1,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 3,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
        ];
        let lr = LayoutResult {
            entities,
            width: 10,
            height: 10,
            ..Default::default()
        };

        // Fed (2.5 == demand): clean — and ONLY because of the pooled
        // read: the pickup tile's own half carries strictly less than the
        // requirement.
        let issues = check_input_rate_delivery(&lr, Some(&sr_with_supply(2.5)));
        assert!(issues.is_empty(), "pooled read must clear a fed pickup: {issues:?}");
        let rates = compute_lane_rates(&lr, Some(&sr_with_supply(2.5)));
        let half = rates.get(&(1, 1)).map(|r| r[0] + r[1]).unwrap_or(0.0);
        assert!(
            half < 2.5 - 0.02,
            "test premise: the branch share alone must NOT cover the demand \
             (got {half}); otherwise this test no longer exercises the pooled read"
        );

        // Genuinely under-fed (1.0 vs 2.5): the pooled credit must NOT
        // rubber-stamp it — the pickup still warns.
        let issues = check_input_rate_delivery(&lr, Some(&sr_with_supply(1.0)));
        assert_eq!(
            issues.len(),
            1,
            "under-fed splitter pickup must warn despite pooling: {issues:?}"
        );
        assert_eq!((issues[0].x, issues[0].y), (Some(1), Some(1)), "{issues:?}");
    }

    /// #624 phantom-source filter regression net. The filter's observable
    /// is seed PLACEMENT, not mass (post-fix, a phantom seed on a
    /// splitter tile is no longer erased, so mass conserves in simple
    /// chains — bot review caught the first attempt at this net testing
    /// nothing). The discriminating topology: an upstream consumer
    /// BETWEEN the real source and an inline splitter. If the splitter's
    /// unfed half is wrongly seeded, part of the external supply enters
    /// AT the splitter — bypassing the upstream consumer, whose belt
    /// then models below full supply.
    #[test]
    fn phantom_splitter_source_does_not_bypass_upstream_consumers() {
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-gear-wheel".to_string(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: 2.0,
                inputs: vec![ItemFlow {
                    item: "iron-plate".to_string(),
                    rate: 2.0,
                    is_fluid: false,
                    module_id: 0,
                }],
                outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 4.0,
                is_fluid: false,
                module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-gear-wheel".to_string()],
            ..Default::default()
        };
        // (0,0)..(2,0)E belt; upstream machine picks at (2,0) via
        // inserter (2,1)N (drop (2,2) into machine at (0,2)..(2,4));
        // belt continues (3,0)E into splitter (4,0)+(4,1)E; downstream
        // machine picks at (4,0) via inserter (5,0)E dropping into the
        // machine at (6,0)..(8,2). The splitter half (4,1) is unfed with
        // a fed sibling — the phantom candidate.
        let mut entities = vec![
            PlacedEntity {
                name: "splitter".to_string(),
                x: 4,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 2,
                y: 1,
                direction: EntityDirection::South,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 0,
                y: 2,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".to_string(),
                x: 5,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            },
            PlacedEntity {
                name: "assembling-machine-3".to_string(),
                x: 6,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                ..Default::default()
            },
        ];
        for x in 0..=3 {
            entities.push(PlacedEntity {
                name: "transport-belt".to_string(),
                x,
                y: 0,
                direction: EntityDirection::East,
                carries: Some("iron-plate".to_string()),
                ..Default::default()
            });
        }
        let lr = LayoutResult {
            entities,
            width: 12,
            height: 8,
            ..Default::default()
        };
        let rates = compute_lane_rates(&lr, Some(&sr));
        let total = |p: (i32, i32)| rates.get(&p).map(|r| r[0] + r[1]).unwrap_or(0.0);
        // The ENTIRE external supply must arrive at the belt head and
        // traverse the upstream pickup tile. A phantom seed on (4,1)
        // diverts part of the supply to enter at the splitter, and this
        // reads below 4.0 (with the filter regressed and even-split
        // seeding, (0,0) models 2.0).
        assert!(
            (total((0, 0)) - 4.0).abs() < 1e-6,
            "external supply must enter at the run head, not at the inline \
             splitter's phantom half: head carries {:?}",
            total((0, 0))
        );
        assert!(
            (total((2, 0)) - 4.0).abs() < 1e-6,
            "upstream pickup tile must see the full supply: {:?}",
            total((2, 0))
        );
    }
}
