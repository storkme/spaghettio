//! Pipe isolation and fluid port connectivity checks.
//!
//! Port of `src/validate.py` — `check_pipe_isolation` and
//! `check_fluid_port_connectivity`.

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{is_machine_entity, DIRECTIONS};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity};
use crate::recipe_db;

use super::{LayoutStyle, Severity, ValidationIssue};

// ---------------------------------------------------------------------------
// Entity-set constants (mirrors Python's module-level sets)
// ---------------------------------------------------------------------------

const PIPE_ENTITIES: &[&str] = &["pipe", "pipe-to-ground"];

// Machine set: the canonical `common::MACHINE_ENTITY_NAMES` via
// `is_machine_entity` (RFC `docs/rfc-power-supply.md` Phase 0b — no more
// hand-synced fluids-local list). Machines with no fluid ports fall through
// the `ports.is_empty()` guard below, so this checks exactly
// `canonical ∩ has-fluid-ports`: AM2/AM3, chemical-plant, oil-refinery,
// biochamber, foundry, cryogenic-plant, electromagnetic-plant. AM1,
// electric-furnace, centrifuge, and recycler have no fluid boxes and are
// skipped. See `machine_has_fluid_ports`.

// ---------------------------------------------------------------------------
// Fluid port data
// ---------------------------------------------------------------------------
//
// Port geometry lives in the shared `crate::fluid_ports` module (RFC
// `docs/rfc-power-supply.md` Phase 0e-i) so the bus templates and this
// validator read the SAME tables — the geometry dual of the Phase 0b machine
// list unification. `fluid_ports` is orientation-aware (mirror + direction);
// the call site below passes each entity's actual `mirror`/`direction`, which
// lets the check honor the East-rotated electromagnetic-plant and the mirrored
// foundry/cryogenic-plant.
use crate::fluid_ports::fluid_ports;

// ---------------------------------------------------------------------------
// check_pipe_isolation
// ---------------------------------------------------------------------------

fn opposite_direction(dir: EntityDirection) -> EntityDirection {
    match dir {
        EntityDirection::North => EntityDirection::South,
        EntityDirection::South => EntityDirection::North,
        EntityDirection::East => EntityDirection::West,
        EntityDirection::West => EntityDirection::East,
    }
}

/// For a pipe-to-ground entity, return the single surface-side neighbour tile.
///
/// Per F5 in `docs/factorio-mechanics.md`, every PTG has its surface
/// connection on the side **opposite** its facing direction, regardless of the
/// blueprint `type` (input/output) field — the type field does not affect
/// surface placement in Factorio's actual fluid simulation.
fn ptg_surface_neighbour(x: i32, y: i32, direction: EntityDirection) -> (i32, i32) {
    let (dx, dy) = match direction {
        EntityDirection::North => (0i32, 1i32),  // surface SOUTH
        EntityDirection::East => (-1, 0),         // surface WEST
        EntityDirection::South => (0, -1),        // surface NORTH
        EntityDirection::West => (1, 0),          // surface EAST
    };
    (x + dx, y + dy)
}

/// Check that adjacent pipes don't carry different fluids.
///
/// In Factorio, adjacent pipes automatically connect and merge their fluid
/// networks.  Two pipes carrying different fluids must not be connected on
/// the surface.
pub fn check_pipe_isolation(layout_result: &LayoutResult) -> Vec<ValidationIssue> {
    type PipeEntry<'a> = (Option<&'a str>, &'a str, EntityDirection);
    let mut pipe_map: FxHashMap<(i32, i32), PipeEntry<'_>> = FxHashMap::default();

    for e in &layout_result.entities {
        if PIPE_ENTITIES.contains(&e.name.as_str()) {
            pipe_map.insert(
                (e.x, e.y),
                (e.carries.as_deref(), e.name.as_str(), e.direction),
            );
        }
    }

    let mut issues = Vec::new();
    // Canonical pairs prevent double-reporting the same edge.
    let mut checked: FxHashSet<((i32, i32), (i32, i32))> = FxHashSet::default();

    for (&(px, py), &(carries, name, direction)) in &pipe_map {
        let carries = match carries {
            Some(c) => c,
            None => continue,
        };

        // Determine which neighbours to check: PTGs expose only one surface
        // side; regular pipes connect on all four sides.
        let ptg_nb;
        let neighbours: &[(i32, i32)] = if name == "pipe-to-ground" {
            ptg_nb = [ptg_surface_neighbour(px, py, direction)];
            &ptg_nb
        } else {
            &[(px + 1, py), (px - 1, py), (px, py + 1), (px, py - 1)]
        };

        for &nb in neighbours {
            let Some(&(nb_carries, nb_name, nb_direction)) = pipe_map.get(&nb) else {
                continue;
            };
            let nb_carries = match nb_carries {
                Some(c) => c,
                None => continue,
            };

            // If neighbour is a PTG, its surface side must face back at us
            if nb_name == "pipe-to-ground" {
                let nb_surface = ptg_surface_neighbour(nb.0, nb.1, nb_direction);
                if nb_surface != (px, py) {
                    continue;
                }
            }

            // Canonical pair to avoid double-reporting
            let pair = if (px, py) <= nb { ((px, py), nb) } else { (nb, (px, py)) };
            if !checked.insert(pair) {
                continue;
            }

            if nb_carries != carries {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "pipe-isolation",
                    format!(
                        "Adjacent pipes carry different fluids: ({px},{py}) carries {carries}, \
                         ({},{}) carries {nb_carries}",
                        nb.0, nb.1
                    ),
                    px,
                    py,
                ));
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// Helpers for check_fluid_port_connectivity
// ---------------------------------------------------------------------------

/// Find pipe-to-ground pairs: returns a bidirectional map `pos_a ↔ pos_b`.
///
/// Mirrors Factorio's pairing semantics (per F4): an input pairs with the
/// **nearest unpaired output** on the same axis whose direction is opposite,
/// within the max underground distance (vanilla pipe-to-ground: 10 tiles
/// gap between input and output, so a max axis distance of 11 between the
/// two entities). Iteration-order matching would cascade incorrect pairs
/// when entities are emitted out of y-order (e.g. junction-solver pipes
/// added after the main trunk emission).
fn find_ptg_pairs(layout_result: &LayoutResult) -> FxHashMap<(i32, i32), (i32, i32)> {
    // Per F4: vanilla pipe-to-ground has max underground distance of 10
    // tiles (gap), so the entity-to-entity distance cap is 11.
    const MAX_PIPE_PTG_DISTANCE: i32 = 11;

    // Collect inputs and outputs separately
    let mut inputs: Vec<&PlacedEntity> = Vec::new();
    let mut outputs: Vec<&PlacedEntity> = Vec::new();

    for e in &layout_result.entities {
        if e.name != "pipe-to-ground" {
            continue;
        }
        match e.io_type.as_deref() {
            Some("input") => inputs.push(e),
            Some("output") => outputs.push(e),
            _ => {}
        }
    }

    // Sort inputs by position so iteration order is deterministic and matches
    // Factorio's "input pairs with the nearest output along its facing
    // direction" semantics — for a row of aligned inputs and outputs, scanning
    // inputs in spatial order means the closest output is always still
    // available when its natural partner reaches the front of the queue.
    inputs.sort_by_key(|e| (e.x, e.y));

    let mut pairs: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    let mut taken: FxHashSet<(i32, i32)> = FxHashSet::default();

    for inp in &inputs {
        let expected_dir = opposite_direction(inp.direction);
        // Find the unpaired output along inp's direction at the smallest distance.
        let mut best: Option<(usize, i32)> = None;
        for (idx, out) in outputs.iter().enumerate() {
            if taken.contains(&(out.x, out.y)) {
                continue;
            }
            if out.direction != expected_dir {
                continue;
            }
            let dist = match inp.direction {
                EntityDirection::East => {
                    if out.y == inp.y && out.x > inp.x { Some(out.x - inp.x) } else { None }
                }
                EntityDirection::West => {
                    if out.y == inp.y && out.x < inp.x { Some(inp.x - out.x) } else { None }
                }
                EntityDirection::South => {
                    if out.x == inp.x && out.y > inp.y { Some(out.y - inp.y) } else { None }
                }
                EntityDirection::North => {
                    if out.x == inp.x && out.y < inp.y { Some(inp.y - out.y) } else { None }
                }
            };
            let Some(d) = dist else { continue };
            if d > MAX_PIPE_PTG_DISTANCE {
                continue;
            }
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((idx, d));
            }
        }
        if let Some((idx, _)) = best {
            let out = outputs[idx];
            let a = (inp.x, inp.y);
            let b = (out.x, out.y);
            pairs.insert(a, b);
            pairs.insert(b, a);
            taken.insert(b);
        }
    }

    pairs
}

/// Per-tile info about a pipe entity for connectivity walks.
///
/// `is_ptg` distinguishes pipe-to-ground (single surface side per F5/F5a) from
/// regular pipes (4-way connections). For PTGs, `direction` is required; for
/// regular pipes it's ignored.
#[derive(Copy, Clone)]
struct PipeInfo {
    is_ptg: bool,
    direction: EntityDirection,
}

fn build_pipe_info_map(
    layout_result: &LayoutResult,
) -> FxHashMap<(i32, i32), PipeInfo> {
    layout_result
        .entities
        .iter()
        .filter(|e| PIPE_ENTITIES.contains(&e.name.as_str()))
        .map(|e| {
            (
                (e.x, e.y),
                PipeInfo {
                    is_ptg: e.name == "pipe-to-ground",
                    direction: e.direction,
                },
            )
        })
        .collect()
}

/// BFS flood-fill through pipe tiles, honoring F5/F5a surface-side rules.
///
/// A regular pipe connects to all four neighbours that are themselves
/// surface-compatible. A PTG connects only on its single surface side, and
/// only if the neighbour at that tile is either a regular pipe or another PTG
/// whose surface points back. Underground tunnel jumps are followed via
/// `ptg_pairs`.
fn bfs_pipe_reach(
    start: (i32, i32),
    pipe_info: &FxHashMap<(i32, i32), PipeInfo>,
    ptg_pairs: &FxHashMap<(i32, i32), (i32, i32)>,
) -> FxHashSet<(i32, i32)> {
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    if !pipe_info.contains_key(&start) {
        return visited;
    }
    visited.insert(start);
    queue.push_back(start);

    while let Some(pos) = queue.pop_front() {
        let info = pipe_info[&pos];
        let (x, y) = pos;

        // Candidate surface neighbours: a PTG only exposes its single mouth
        // tile; a regular pipe exposes all 4 sides.
        let mut candidates: [(i32, i32); 4] = [(0, 0); 4];
        let n = if info.is_ptg {
            candidates[0] = ptg_surface_neighbour(x, y, info.direction);
            1
        } else {
            for (i, (dx, dy)) in DIRECTIONS.iter().enumerate() {
                candidates[i] = (x + dx, y + dy);
            }
            4
        };

        for nb in &candidates[..n] {
            let Some(nb_info) = pipe_info.get(nb).copied() else {
                continue;
            };
            // If the neighbour is a PTG, its mouth must point back at us.
            if nb_info.is_ptg
                && ptg_surface_neighbour(nb.0, nb.1, nb_info.direction) != pos
            {
                continue;
            }
            if visited.insert(*nb) {
                queue.push_back(*nb);
            }
        }

        // Underground tunnel jump (independent of surface mouth orientation).
        if let Some(&other) = ptg_pairs.get(&pos) {
            if visited.insert(other) {
                queue.push_back(other);
            }
        }
    }

    visited
}

/// Return `true` if `recipe_name` produces at least one fluid product.
fn recipe_has_fluid_output(recipe_name: &str) -> bool {
    if let Some(recipe) = recipe_db::db().recipes.get(recipe_name) {
        recipe.products.iter().any(|p| p.type_ == "fluid")
    } else {
        false
    }
}

/// Return `true` if `recipe_name` consumes at least one fluid ingredient.
///
/// The input-side dual of [`recipe_has_fluid_output`]: an input fluid port
/// is only *active* (and thus requires a connected pipe) when the recipe
/// actually consumes a fluid. This replaces the old machine-allowlist
/// "fluid boxes disabled" guard — recipe-driven gating is correct for every
/// machine, including foundry/cryogenic-plant recipes that produce a fluid
/// but consume none (`molten-iron`, `molten-copper`), where the old
/// "skip only if NO port has any pipe" guard would false-positive on the
/// idle input port while the output port carried a pipe.
fn recipe_has_fluid_input(recipe_name: &str) -> bool {
    if let Some(recipe) = recipe_db::db().recipes.get(recipe_name) {
        recipe.ingredients.iter().any(|i| i.type_ == "fluid")
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// check_fluid_port_connectivity
// ---------------------------------------------------------------------------

/// Check that every machine's fluid ports have connected pipes.
///
/// For each machine with fluid ports, verifies:
/// 1. At least one input port has an adjacent pipe.
/// 2. (`Bus` style only) At least one input pipe is reachable from the bus
///    via BFS.
/// 3. At least one output port has an adjacent pipe (only if the recipe
///    actually produces a fluid).
pub fn check_fluid_port_connectivity(
    layout_result: &LayoutResult,
    layout_style: LayoutStyle,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Build pipe info map (tracks PTG vs regular pipe + direction for F5/F5a).
    let pipe_info = build_pipe_info_map(layout_result);
    // Plain tile set for membership-only checks (port adjacency, bus filter).
    let pipe_tiles: FxHashSet<(i32, i32)> = pipe_info.keys().copied().collect();

    // Build PTG pair map for tunnel traversal
    let ptg_pairs = find_ptg_pairs(layout_result);

    // Find bus pipe positions (pipes west of the leftmost machine).
    // Only needed for Bus-mode connectivity checks.
    let bus_pipes: FxHashSet<(i32, i32)> = if layout_style == LayoutStyle::Bus && !pipe_tiles.is_empty() {
        let leftmost_machine_x = layout_result
            .entities
            .iter()
            .filter(|e| is_machine_entity(&e.name))
            .map(|e| e.x)
            .min();

        if let Some(leftmost) = leftmost_machine_x {
            let west_pipes: FxHashSet<_> =
                pipe_tiles.iter().copied().filter(|(x, _)| *x < leftmost).collect();
            if !west_pipes.is_empty() {
                west_pipes
            } else {
                // Fallback: leftmost pipe column
                let min_x = pipe_tiles.iter().map(|(x, _)| *x).min().unwrap();
                pipe_tiles.iter().copied().filter(|(x, _)| *x == min_x).collect()
            }
        } else {
            // No machines — fallback to leftmost column
            let min_x = pipe_tiles.iter().map(|(x, _)| *x).min().unwrap();
            pipe_tiles.iter().copied().filter(|(x, _)| *x == min_x).collect()
        }
    } else {
        FxHashSet::default()
    };

    for e in &layout_result.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        let recipe = match &e.recipe {
            Some(r) => r.as_str(),
            None => continue,
        };

        // Machines with no fluid ports (AM1, electric-furnace, centrifuge,
        // recycler) fall out here — this is the `∩ has-fluid-ports` filter.
        let ports = fluid_ports(e.name.as_str(), e.mirror, e.direction);
        if ports.is_empty() {
            continue;
        }

        // A fluid box is only *active* when the recipe actually uses that
        // fluid direction. Many machines carry fluid boxes that sit idle for
        // solid recipes — assembling-machine-{2,3} (most recipes), biochamber
        // (organic recipes are frequently pure-solid: iron/copper-bacteria-
        // cultivation, bioflux, carbon-fiber, …), foundry (belt/splitter
        // casting), cryogenic-plant (fusion-reactor, promethium-science). An
        // idle port has no pipe and must NOT be flagged.
        //
        // Recipe-driven gating (RFC `docs/rfc-power-supply.md` Phase 0b)
        // replaces the previous machine-allowlist guard: it is the input-side
        // dual of the long-standing `recipe_has_fluid_output` output gate, and
        // is correct for every machine — including foundry/cryogenic-plant
        // recipes that produce a fluid but consume none (`molten-iron`,
        // `molten-copper`), which the old "skip only if NO port has any pipe"
        // guard mishandled (the live output pipe kept it from skipping, so the
        // idle input port false-positived). chemical-plant and oil-refinery
        // recipes always consume a fluid, so their input check still always
        // fires — a missing pipe there is a genuine bug (see
        // `chemical_plant_no_input_pipe_error`).
        let has_fluid_input = recipe_has_fluid_input(recipe);
        let has_fluid_output = recipe_has_fluid_output(recipe);
        if !has_fluid_input && !has_fluid_output {
            continue;
        }

        let input_ports: Vec<(i32, i32)> = ports
            .iter()
            .filter(|(_, _, pt)| *pt == "input")
            .map(|(rx, ry, _)| (e.x + rx, e.y + ry))
            .collect();
        let output_ports: Vec<(i32, i32)> = ports
            .iter()
            .filter(|(_, _, pt)| *pt == "output")
            .map(|(rx, ry, _)| (e.x + rx, e.y + ry))
            .collect();

        // --- Input port checks (only when the recipe consumes a fluid) ---
        if has_fluid_input && !input_ports.is_empty() {
            let input_pipe_positions: Vec<(i32, i32)> = input_ports
                .iter()
                .copied()
                .filter(|pos| pipe_tiles.contains(pos))
                .collect();

            if input_pipe_positions.is_empty() {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "fluid-connectivity",
                    format!(
                        "{} at ({},{}): no input port has an adjacent pipe",
                        e.name, e.x, e.y
                    ),
                    e.x,
                    e.y,
                ));
            } else if layout_style == LayoutStyle::Bus && !bus_pipes.is_empty() {
                // Check at least one input pipe connects to the bus via BFS
                let any_connected = input_pipe_positions.iter().any(|&pos| {
                    !bfs_pipe_reach(pos, &pipe_info, &ptg_pairs)
                        .is_disjoint(&bus_pipes)
                });
                if !any_connected {
                    issues.push(ValidationIssue::with_pos(
                        Severity::Error,
                        "fluid-connectivity",
                        format!(
                            "{} at ({},{}): input pipes not connected to bus",
                            e.name, e.x, e.y
                        ),
                        e.x,
                        e.y,
                    ));
                }
            }
        }

        // --- Output port checks (only when the recipe produces a fluid) ---
        if has_fluid_output && !output_ports.is_empty() {
            let has_output_pipe = output_ports
                .iter()
                .any(|pos| pipe_tiles.contains(pos));
            if !has_output_pipe {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "fluid-connectivity",
                    format!(
                        "{} at ({},{}): no output port has an adjacent pipe",
                        e.name, e.x, e.y
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
// check_fluid_network_connectivity
// ---------------------------------------------------------------------------

/// Check that every pipe labeled as carrying a given fluid is connected to
/// every other pipe carrying that fluid via real surface + tunnel paths
/// (respecting F5/F5a).
///
/// Catches cases the older validators missed:
/// - Perpendicular UG/pipe adjacency that the layout treats as connected but
///   isn't (issue 1: a UG-S input on a tap row vs the horizontal branch one
///   tile to its east).
/// - Silent gap-fill skips that leave a physical break in a fluid trunk
///   (e.g. a UG bridge skipped because an intermediate anchor was blocked,
///   leaving two trunk segments labeled as the same fluid but disconnected).
///
/// One error is emitted per orphaned component, anchored at the
/// lexicographically smallest tile of that component for stable output.
pub fn check_fluid_network_connectivity(
    layout_result: &LayoutResult,
) -> Vec<ValidationIssue> {
    let pipe_info = build_pipe_info_map(layout_result);
    let ptg_pairs = find_ptg_pairs(layout_result);

    // Group pipe tiles by carried fluid.
    let mut by_fluid: FxHashMap<&str, Vec<(i32, i32)>> = FxHashMap::default();
    for e in &layout_result.entities {
        if !PIPE_ENTITIES.contains(&e.name.as_str()) {
            continue;
        }
        let Some(carries) = e.carries.as_deref() else {
            continue;
        };
        by_fluid.entry(carries).or_default().push((e.x, e.y));
    }

    let mut issues = Vec::new();

    // Stable iteration for stable error ordering.
    let mut fluids: Vec<(&str, Vec<(i32, i32)>)> = by_fluid.into_iter().collect();
    fluids.sort_by_key(|(name, _)| *name);

    for (fluid, mut tiles) in fluids {
        if tiles.len() < 2 {
            continue;
        }
        tiles.sort_unstable();
        let tile_set: FxHashSet<(i32, i32)> = tiles.iter().copied().collect();

        // Restrict the BFS to same-fluid tiles only — cross-fluid contamination
        // is reported separately by `check_pipe_isolation`; here we just want
        // to know whether all pipes carrying this fluid form one network.
        let fluid_pipe_info: FxHashMap<(i32, i32), PipeInfo> = pipe_info
            .iter()
            .filter(|(p, _)| tile_set.contains(p))
            .map(|(&p, &i)| (p, i))
            .collect();
        let fluid_ptg_pairs: FxHashMap<(i32, i32), (i32, i32)> = ptg_pairs
            .iter()
            .filter(|(a, b)| tile_set.contains(*a) && tile_set.contains(b))
            .map(|(&a, &b)| (a, b))
            .collect();

        let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut components: Vec<FxHashSet<(i32, i32)>> = Vec::new();
        for &start in &tiles {
            if visited.contains(&start) {
                continue;
            }
            let reached = bfs_pipe_reach(start, &fluid_pipe_info, &fluid_ptg_pairs);
            visited.extend(reached.iter().copied());
            components.push(reached);
        }

        if components.len() > 1 {
            // Sort components by their representative tile for stable output.
            let mut reps: Vec<(i32, i32)> = components
                .iter()
                .map(|c| c.iter().copied().min().unwrap_or((0, 0)))
                .collect();
            reps.sort_unstable();
            let n_components = components.len();
            for (x, y) in reps.iter().skip(1) {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "fluid-network",
                    format!(
                        "{fluid} pipe network is split into {n_components} disconnected components; orphan tile at ({x},{y})"
                    ),
                    *x,
                    *y,
                ));
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, LayoutResult, PlacedEntity};

    fn pipe(x: i32, y: i32, carries: Option<&str>) -> PlacedEntity {
        PlacedEntity {
            name: "pipe".to_string(),
            x,
            y,
            direction: EntityDirection::North,
            carries: carries.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn ptg(
        x: i32,
        y: i32,
        dir: EntityDirection,
        io_type: &str,
        carries: Option<&str>,
    ) -> PlacedEntity {
        PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x,
            y,
            direction: dir,
            io_type: Some(io_type.to_string()),
            carries: carries.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn machine(name: &str, x: i32, y: i32, recipe: &str, mirror: bool) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            recipe: Some(recipe.to_string()),
            mirror,
            ..Default::default()
        }
    }

    fn layout(entities: Vec<PlacedEntity>) -> LayoutResult {
        LayoutResult { entities, width: 20, height: 20, ..Default::default() }
    }

    // === check_pipe_isolation ===

    #[test]
    fn same_fluid_adjacent_ok() {
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 0, Some("water")),
            pipe(2, 0, Some("water")),
        ]);
        assert!(check_pipe_isolation(&lr).is_empty());
    }

    #[test]
    fn different_fluid_adjacent_error() {
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 0, Some("crude-oil")),
        ]);
        let issues = check_pipe_isolation(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].category, "pipe-isolation");
    }

    #[test]
    fn diagonal_pipes_ok() {
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 1, Some("crude-oil")),
        ]);
        assert!(check_pipe_isolation(&lr).is_empty());
    }

    #[test]
    fn untagged_pipes_ignored() {
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 0, None),
        ]);
        assert!(check_pipe_isolation(&lr).is_empty());
    }

    #[test]
    fn separated_pipes_ok() {
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(2, 0, Some("crude-oil")),
        ]);
        assert!(check_pipe_isolation(&lr).is_empty());
    }

    #[test]
    fn different_fluid_reported_once_not_twice() {
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 0, Some("petroleum-gas")),
        ]);
        assert_eq!(check_pipe_isolation(&lr).len(), 1);
    }

    #[test]
    fn ptg_input_surface_neighbour_check() {
        // PTG input facing EAST: surface side is WEST (behind direction)
        // So ptg at (3,0) facing EAST io=input → surface neighbour is (2,0)
        // pipe at (2,0) carries water, ptg carries crude-oil → isolation error
        let lr = layout(vec![
            pipe(2, 0, Some("water")),
            ptg(3, 0, EntityDirection::East, "input", Some("crude-oil")),
        ]);
        let issues = check_pipe_isolation(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "pipe-isolation");
    }

    #[test]
    fn ptg_wrong_side_not_checked() {
        // PTG input at (3,0) facing EAST: only connects to (2,0)
        // pipe at (4,0) is on the wrong side → not connected → no error
        let lr = layout(vec![
            pipe(4, 0, Some("crude-oil")),
            ptg(3, 0, EntityDirection::East, "input", Some("water")),
        ]);
        assert!(check_pipe_isolation(&lr).is_empty());
    }

    #[test]
    fn ptg_output_surface_opposite_direction() {
        // Per F5: output PTG surface side is OPPOSITE its facing direction
        // (same rule as input). Direction=North → surface SOUTH.
        // PTG output dir=North at (0, 1) has its mouth at (0, 2).
        // Pipe at (0, 2) carrying a different fluid → isolation error.
        let lr = layout(vec![
            ptg(0, 1, EntityDirection::North, "output", Some("water")),
            pipe(0, 2, Some("crude-oil")),
        ]);
        let issues = check_pipe_isolation(&lr);
        assert_eq!(issues.len(), 1, "expected isolation error from output mouth on south side");
        assert_eq!(issues[0].category, "pipe-isolation");
    }

    #[test]
    fn ptg_output_north_side_is_not_surface() {
        // Output dir=North surface is SOUTH, NOT north.
        // A pipe on the north side of an output dir=North PTG is NOT connected.
        let lr = layout(vec![
            pipe(0, 0, Some("crude-oil")),
            ptg(0, 1, EntityDirection::North, "output", Some("water")),
        ]);
        // No surface connection between (0,0) and (0,1) → no isolation error
        assert!(check_pipe_isolation(&lr).is_empty());
    }

    // === check_fluid_port_connectivity ===

    #[test]
    fn no_fluid_machines_no_issues() {
        let lr = layout(vec![
            machine("assembling-machine-1", 0, 0, "iron-gear-wheel", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        assert!(issues.is_empty());
    }

    #[test]
    fn assembling_machine_no_pipes_skipped() {
        // assembling-machine-2 without adjacent pipes → skipped (fluid_boxes_off)
        let lr = layout(vec![
            machine("assembling-machine-2", 0, 0, "iron-gear-wheel", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        assert!(issues.is_empty());
    }

    #[test]
    fn chemical_plant_no_input_pipe_error() {
        // chemical-plant at (0,0): input ports at (0,-1) and (2,-1)
        // No pipes placed → should error
        let lr = layout(vec![
            machine("chemical-plant", 0, 0, "plastic-bar", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty(), "expected fluid-connectivity error");
        assert!(errors.iter().all(|i| i.category == "fluid-connectivity"));
    }

    #[test]
    fn chemical_plant_with_input_pipe_ok_spaghetti() {
        // plastic-bar has no fluid output so only input check applies
        // chemical-plant at (0,0): input port at (0,-1)
        let lr = layout(vec![
            machine("chemical-plant", 0, 0, "plastic-bar", false),
            pipe(0, -1, Some("petroleum-gas")),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    #[test]
    fn oil_refinery_fluid_output_needs_pipe() {
        // basic-oil-processing produces fluid outputs
        // oil-refinery at (0,0): output ports at (0,-1),(2,-1),(4,-1)
        // Place input pipes but no output pipe → should error on output
        let lr = layout(vec![
            machine("oil-refinery", 0, 0, "basic-oil-processing", false),
            pipe(1, 5, Some("crude-oil")),  // input port 1
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        // Should have output-pipe-missing error
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|i| i.message.contains("output port")));
    }

    #[test]
    fn oil_refinery_mirror_ports_flipped() {
        // mirror=true: input ports move to (1,-1),(3,-1); outputs to (0,5),(2,5),(4,5)
        let lr = layout(vec![
            machine("oil-refinery", 0, 0, "basic-oil-processing", true),
            pipe(1, -1, Some("crude-oil")), // input port with mirror
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        // With mirror, input at (1,-1) should be adjacent → only output error remains
        let input_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("input") && i.severity == Severity::Error)
            .collect();
        assert!(input_errors.is_empty(), "unexpected input errors with mirror: {:?}", input_errors);
    }

    #[test]
    fn bus_mode_input_pipe_not_connected_to_bus_error() {
        // Bus mode: machine at x=5, bus pipe at x=0
        // Machine's input pipe at (5, 3) but not connected to bus
        let lr = layout(vec![
            machine("chemical-plant", 5, 4, "plastic-bar", false),
            // Input port at (5+0, 4-1) = (5,3)
            pipe(5, 3, Some("petroleum-gas")), // adjacent but not connected to bus
            // Bus pipe far to the left
            pipe(0, 3, Some("petroleum-gas")),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Bus);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty(), "expected bus connectivity error");
        assert!(errors.iter().any(|i| i.message.contains("not connected to bus")));
    }

    #[test]
    fn bus_mode_input_pipe_connected_via_ptg_to_bus_ok() {
        // Bus mode: machine at x=5, bus pipe at x=0
        // PTG tunnel bridges the gap
        let lr = layout(vec![
            machine("chemical-plant", 5, 4, "plastic-bar", false),
            // Input port at (5,3)
            pipe(5, 3, Some("petroleum-gas")),
            // PTG tunnel from x=4 to x=1 (WEST direction)
            ptg(4, 3, EntityDirection::West, "input", Some("petroleum-gas")),
            ptg(1, 3, EntityDirection::West, "output", Some("petroleum-gas")),
            // Bus pipe
            pipe(0, 3, Some("petroleum-gas")),
        ]);
        // Connect the chain: (5,3)-(4,3) adjacent, ptg tunnel (4,3)-(1,3), (1,3)-(0,3) adjacent
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Bus);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    }

    // === find_ptg_pairs helper ===

    #[test]
    fn ptg_pairs_east_direction() {
        let lr = layout(vec![
            ptg(0, 0, EntityDirection::East, "input", None),
            ptg(3, 0, EntityDirection::West, "output", None),
        ]);
        let pairs = find_ptg_pairs(&lr);
        assert_eq!(pairs.get(&(0, 0)), Some(&(3, 0)));
        assert_eq!(pairs.get(&(3, 0)), Some(&(0, 0)));
    }

    #[test]
    fn ptg_pairs_north_direction() {
        let lr = layout(vec![
            ptg(0, 3, EntityDirection::North, "input", None),
            ptg(0, 0, EntityDirection::South, "output", None),
        ]);
        let pairs = find_ptg_pairs(&lr);
        assert_eq!(pairs.get(&(0, 3)), Some(&(0, 0)));
        assert_eq!(pairs.get(&(0, 0)), Some(&(0, 3)));
    }

    #[test]
    fn ptg_pairs_wrong_direction_not_paired() {
        // Output faces same direction as input instead of opposite → no pairing
        let lr = layout(vec![
            ptg(3, 0, EntityDirection::East, "input", None),
            ptg(0, 0, EntityDirection::East, "output", None),
        ]);
        let pairs = find_ptg_pairs(&lr);
        assert!(pairs.is_empty());
    }

    // === bfs_pipe_reach ===

    fn regular_pipes_at(positions: &[(i32, i32)]) -> FxHashMap<(i32, i32), PipeInfo> {
        positions
            .iter()
            .map(|&p| {
                (
                    p,
                    PipeInfo {
                        is_ptg: false,
                        direction: EntityDirection::North,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn bfs_reaches_adjacent_tiles() {
        let info = regular_pipes_at(&[(0, 0), (1, 0), (2, 0)]);
        let ptg: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
        let reached = bfs_pipe_reach((0, 0), &info, &ptg);
        assert!(reached.contains(&(0, 0)));
        assert!(reached.contains(&(2, 0)));
    }

    #[test]
    fn bfs_traverses_ptg_tunnel() {
        // (0,0) regular pipe → (1,0) PTG (East input, surface (0,0)) →
        // tunnel → (5,0) PTG (East output, surface (6,0)) → (6,0) regular pipe.
        let mut info: FxHashMap<(i32, i32), PipeInfo> = FxHashMap::default();
        info.insert((0, 0), PipeInfo { is_ptg: false, direction: EntityDirection::North });
        info.insert((1, 0), PipeInfo { is_ptg: true, direction: EntityDirection::East });
        info.insert((5, 0), PipeInfo { is_ptg: true, direction: EntityDirection::West });
        info.insert((6, 0), PipeInfo { is_ptg: false, direction: EntityDirection::North });
        let mut ptg: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
        ptg.insert((1, 0), (5, 0));
        ptg.insert((5, 0), (1, 0));
        let reached = bfs_pipe_reach((0, 0), &info, &ptg);
        assert!(reached.contains(&(6, 0)));
    }

    #[test]
    fn bfs_perpendicular_to_ptg_not_reached() {
        // PTG dir=South (mouth NORTH at (0, -1)).
        // A regular pipe to the EAST of the PTG should NOT be reachable —
        // perpendicular sides have no surface connection (F5a).
        let mut info: FxHashMap<(i32, i32), PipeInfo> = FxHashMap::default();
        info.insert((0, 0), PipeInfo { is_ptg: true, direction: EntityDirection::South });
        info.insert((1, 0), PipeInfo { is_ptg: false, direction: EntityDirection::North });
        let ptg: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
        let reached = bfs_pipe_reach((0, 0), &info, &ptg);
        assert!(!reached.contains(&(1, 0)),
            "perpendicular pipe should not be surface-reachable from PTG");
    }

    // === check_fluid_network_connectivity ===

    #[test]
    fn fluid_network_single_connected_ok() {
        // Three pipes carrying water, all surface-adjacent → one network
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 0, Some("water")),
            pipe(2, 0, Some("water")),
        ]);
        assert!(check_fluid_network_connectivity(&lr).is_empty());
    }

    #[test]
    fn fluid_network_perpendicular_branch_to_ptg_orphan() {
        // The issue 1 shape: UG-S input on a tap row, regular pipe to its
        // east as a horizontal branch. Both labelled water but perpendicular,
        // so they form two disconnected components.
        let lr = layout(vec![
            ptg(0, 1, EntityDirection::South, "input", Some("water")),
            pipe(1, 1, Some("water")),
        ]);
        let issues = check_fluid_network_connectivity(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "fluid-network");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn fluid_network_gap_in_trunk_orphan() {
        // Two trunk segments labelled the same fluid but separated by an
        // empty tile — the silent-gap-fill case (bug 3) condensed.
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            // gap at (0, 1)
            pipe(0, 2, Some("water")),
        ]);
        let issues = check_fluid_network_connectivity(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "fluid-network");
    }

    #[test]
    fn fluid_network_through_ug_pair_ok() {
        // Pipe → UG-S input → tunnel → UG-N output → pipe. All same fluid,
        // all reachable as one component.
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            ptg(0, 1, EntityDirection::South, "input", Some("water")),
            // intervening tile (0, 2) skipped — UG tunnel
            ptg(0, 3, EntityDirection::North, "output", Some("water")),
            pipe(0, 4, Some("water")),
        ]);
        assert!(check_fluid_network_connectivity(&lr).is_empty(),
            "UG-paired same-fluid network should be one component");
    }

    #[test]
    fn fluid_network_different_fluids_independent() {
        // Two separate fluids each in their own connected network → no error
        let lr = layout(vec![
            pipe(0, 0, Some("water")),
            pipe(1, 0, Some("water")),
            pipe(0, 5, Some("crude-oil")),
            pipe(1, 5, Some("crude-oil")),
        ]);
        assert!(check_fluid_network_connectivity(&lr).is_empty());
    }

    #[test]
    fn bfs_two_ptgs_facing_each_other_reach() {
        // Two PTGs adjacent: (0,0) dir=East (mouth WEST at (-1,0))
        // and (1,0) dir=West (mouth EAST at (2,0)). They are tile-adjacent
        // but neither's mouth points at the other → no surface connection.
        let mut info: FxHashMap<(i32, i32), PipeInfo> = FxHashMap::default();
        info.insert((0, 0), PipeInfo { is_ptg: true, direction: EntityDirection::East });
        info.insert((1, 0), PipeInfo { is_ptg: true, direction: EntityDirection::West });
        let ptg: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
        let reached = bfs_pipe_reach((0, 0), &info, &ptg);
        assert!(!reached.contains(&(1, 0)),
            "PTGs whose mouths face away from each other don't surface-connect");
    }

    // === recipe_has_fluid_output ===

    #[test]
    fn plastic_bar_has_no_fluid_output() {
        assert!(!recipe_has_fluid_output("plastic-bar"));
    }

    #[test]
    fn basic_oil_processing_has_fluid_output() {
        assert!(recipe_has_fluid_output("basic-oil-processing"));
    }

    #[test]
    fn unknown_recipe_has_no_fluid_output() {
        assert!(!recipe_has_fluid_output("nonexistent-recipe"));
    }

    // === recipe_has_fluid_input gating ===

    #[test]
    fn recipe_fluid_input_detection() {
        // casting-iron consumes molten-iron (fluid), produces solid.
        assert!(recipe_has_fluid_input("casting-iron"));
        assert!(!recipe_has_fluid_output("casting-iron"));
        // molten-iron consumes solid ore, produces a fluid (no fluid input).
        assert!(!recipe_has_fluid_input("molten-iron"));
        assert!(recipe_has_fluid_output("molten-iron"));
        // A pure-solid belt cast on the foundry — neither direction is fluid.
        assert!(!recipe_has_fluid_input("transport-belt"));
        assert!(!recipe_has_fluid_output("transport-belt"));
    }

    #[test]
    fn foundry_fluid_output_only_recipe_no_false_input_error() {
        // molten-iron: fluid OUTPUT, no fluid input. The output pipe is
        // present but there is (correctly) no input pipe. The old
        // machine-allowlist guard would have false-positived on the idle
        // input port; recipe-driven gating must not.
        let lr = layout(vec![
            machine("foundry", 0, 0, "molten-iron", false),
            // output port at (1,-1) or (3,-1) — supply one so the output
            // check is satisfied and only the input-side behavior is tested.
            pipe(1, -1, Some("molten-iron")),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        let input_errs: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("input") && i.severity == Severity::Error)
            .collect();
        assert!(input_errs.is_empty(), "unexpected input error on fluid-output-only recipe: {input_errs:?}");
    }

    #[test]
    fn foundry_solid_recipe_skipped() {
        // transport-belt on a foundry uses no fluid at all — both fluid
        // boxes idle, no pipes, must produce no fluid-connectivity issue.
        let lr = layout(vec![
            machine("foundry", 0, 0, "transport-belt", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        assert!(issues.is_empty(), "solid foundry recipe should be skipped: {issues:?}");
    }

    #[test]
    fn foundry_casting_missing_input_pipe_error() {
        // casting-iron consumes molten-iron (fluid input) — a foundry with
        // no input pipe is a genuine missing-pipe bug and MUST error.
        let lr = layout(vec![
            machine("foundry", 0, 0, "casting-iron", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr, LayoutStyle::Spaghetti);
        let errs: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errs.is_empty(), "expected missing-input-pipe error");
        assert!(errs.iter().all(|i| i.category == "fluid-connectivity"));
        assert!(errs.iter().any(|i| i.message.contains("input")));
    }
}
