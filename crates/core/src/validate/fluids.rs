//! Pipe isolation and fluid port connectivity checks.
//!
//! Port of `src/validate.py` — `check_pipe_isolation` and
//! `check_fluid_port_connectivity`.

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{is_machine_entity, DIRECTIONS};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity};
use crate::recipe_db;

use super::{Severity, ValidationIssue};

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
pub(crate) fn ptg_surface_neighbour(x: i32, y: i32, direction: EntityDirection) -> (i32, i32) {
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
/// within the max underground distance. Vanilla pipe-to-ground has
/// `max_underground_distance: 10`, which — like the belt `max_distance`
/// family — caps the ENTITY-TO-ENTITY axis distance at 10 (gap ≤ 9).
/// The old model here said gap 10 / distance 11; the first fluid-chain
/// sim measurement falsified it (an 11-apart trunk pair carried
/// nothing — #407). Iteration-order matching would cascade incorrect pairs
/// when entities are emitted out of y-order (e.g. junction-solver pipes
/// added after the main trunk emission).
pub(crate) fn find_ptg_pairs(layout_result: &LayoutResult) -> FxHashMap<(i32, i32), (i32, i32)> {
    // Game-measured (#407): entities at most 10 apart connect; 11 does
    // not. Mirrors belt max_distance semantics (yellow UG belt
    // max_distance=5 = gap 4 = our ug_max_reach).
    const MAX_PIPE_PTG_DISTANCE: i32 = 10;

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

/// Fluid SUPPLY tiles: pipes sitting on a fluid boundary-input record
/// (external supply by definition) or on a machine's fluid OUTPUT port
/// (internal production). Shared by the connectivity check (as bus
/// members) and the split-network check (a component containing one is
/// independently supplied — multi-copy composed layouts legitimately
/// run one network per copy; only UNSUPPLIED fragments are severed
/// trunks). RFC-052 Phase-B increment.
fn fluid_supply_tiles(
    layout_result: &LayoutResult,
    pipe_tiles: &FxHashSet<(i32, i32)>,
) -> FxHashSet<(i32, i32)> {
    let mut supply: FxHashSet<(i32, i32)> = FxHashSet::default();
    for r in layout_result.boundary_inputs.iter().filter(|r| r.is_fluid) {
        if pipe_tiles.contains(&(r.x, r.y)) {
            supply.insert((r.x, r.y));
        }
    }
    for e in &layout_result.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        let Some(recipe) = e.recipe.as_deref() else { continue };
        if !recipe_has_fluid_output(recipe) {
            continue;
        }
        for &(rx, ry, pt) in fluid_ports(e.name.as_str(), e.mirror, e.direction) {
            if pt == "output" && pipe_tiles.contains(&(e.x + rx, e.y + ry)) {
                supply.insert((e.x + rx, e.y + ry));
            }
        }
    }
    supply
}

/// Check that every machine's fluid ports have connected pipes.
///
/// For each machine with fluid ports, verifies:
/// 1. At least one input port has an adjacent pipe.
/// 2. At least one input pipe is reachable from the bus
///    via BFS.
/// 3. At least one output port has an adjacent pipe (only if the recipe
///    actually produces a fluid).
pub fn check_fluid_port_connectivity(
    layout_result: &LayoutResult,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Build pipe info map (tracks PTG vs regular pipe + direction for F5/F5a).
    let pipe_info = build_pipe_info_map(layout_result);
    // Plain tile set for membership-only checks (port adjacency, bus filter).
    let pipe_tiles: FxHashSet<(i32, i32)> = pipe_info.keys().copied().collect();

    // Build PTG pair map for tunnel traversal
    let ptg_pairs = find_ptg_pairs(layout_result);

    // Find bus pipe positions (pipes west of the leftmost machine).
    // (Unconditional since LayoutStyle's 2026-08-20 deletion — every
    // production layout was Bus; empty pipe_tiles short-circuits below.)
    let bus_pipes: FxHashSet<(i32, i32)> = if !pipe_tiles.is_empty() {
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
    // The west-of-leftmost-machine heuristic above assumes a single
    // west-edge bus, which is false for multi-copy composed layouts
    // (RFC-052: a K=2 chain's second copy has its feed heads and
    // internal producers east of the first copy's machines, and every
    // fluid machine in it flunked connectivity despite fully-connected
    // pipes). Two purely-additive supply sources widen the set:
    // (1) pipes ON a fluid boundary-input record — external supply by
    // definition; (2) pipes ON a machine's fluid OUTPUT port — an
    // input network containing a connected producer is supplied
    // internally (petroleum has no boundary record anywhere; the old
    // check only passed such machines when a trunk happened to poke
    // west of the leftmost machine).
    let bus_pipes: FxHashSet<(i32, i32)> = {
        let mut b = bus_pipes;
        b.extend(fluid_supply_tiles(layout_result, &pipe_tiles));
        b
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
            } else if !bus_pipes.is_empty() {
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
    solver: Option<&crate::models::SolverResult>,
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
            // Independently SUPPLIED components are legitimate — a K>1
            // composed chain runs one network per copy (RFC-052). Only
            // unsupplied fragments are severed trunks. When every
            // component is unsupplied (no records, no producers — the
            // synthetic-fixture case), keep the historical all-but-one
            // reporting.
            let supply = fluid_supply_tiles(layout_result, &tile_set);
            let unsupplied: Vec<&FxHashSet<(i32, i32)>> = components
                .iter()
                .filter(|c| c.is_disjoint(&supply))
                .collect();
            let all_unsupplied = unsupplied.len() == components.len();
            let flagged: Vec<&FxHashSet<(i32, i32)>> = if all_unsupplied {
                components.iter().skip(1).collect()
            } else {
                unsupplied
            };
            let mut reps: Vec<(i32, i32)> = flagged
                .iter()
                .map(|c| c.iter().copied().min().unwrap_or((0, 0)))
                .collect();
            reps.sort_unstable();
            let n_components = components.len();
            for (x, y) in reps.iter() {
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

            // #409 tightening: a SUPPLIED component can still be a severed
            // fragment — having *a* producer or boundary tile says nothing
            // about having *enough*. With the solver's rates, flag any
            // component whose attached consumer demand exceeds the supply
            // reachable inside it (producer machines' fluid output rate +
            // this fluid's share of external input, split evenly across
            // its boundary tiles like belt_flow's external seeding).
            // Independently-supplied balanced components — one network per
            // copy in K>1 composed chains (RFC-052) — stay legitimate:
            // their books balance. Skipped on the historical all-unsupplied
            // path (synthetic fixtures; already fully reported above) and
            // without a solver (no rates to account with).
            if let Some(sr) = solver {
                if !all_unsupplied {
                    let recipe_to_spec: FxHashMap<&str, &crate::models::MachineSpec> =
                        sr.machines.iter().map(|s| (s.recipe.as_str(), s)).collect();
                    let external_rate: f64 = sr
                        .external_inputs
                        .iter()
                        .find(|f| f.is_fluid && f.item == fluid)
                        .map(|f| f.rate)
                        .unwrap_or(0.0);
                    let boundary_tiles: Vec<(i32, i32)> = layout_result
                        .boundary_inputs
                        .iter()
                        .filter(|r| r.is_fluid && r.item == fluid && tile_set.contains(&(r.x, r.y)))
                        .map(|r| (r.x, r.y))
                        .collect();
                    let per_boundary_rate = if boundary_tiles.is_empty() {
                        0.0
                    } else {
                        external_rate / boundary_tiles.len() as f64
                    };
                    let comp_of =
                        |t: (i32, i32)| components.iter().position(|c| c.contains(&t));

                    let mut supply_rates = vec![0.0f64; components.len()];
                    let mut demand_rates = vec![0.0f64; components.len()];
                    for &bt in &boundary_tiles {
                        if let Some(i) = comp_of(bt) {
                            supply_rates[i] += per_boundary_rate;
                        }
                    }
                    for e in &layout_result.entities {
                        if !is_machine_entity(&e.name) {
                            continue;
                        }
                        let Some(recipe) = e.recipe.as_deref() else { continue };
                        let Some(&fallback_spec) = recipe_to_spec.get(recipe) else { continue };
                        // Position-resolved spec: composed/partitioned rows
                        // carry per-copy specs in `effective_rows`, and the
                        // whole-solve spec massively misattributes rates per
                        // copy (a K=8 chain read 8x the per-copy water
                        // demand before this). Same convention as every
                        // other rate check — see `resolve_row_spec`.
                        let spec = super::resolve_row_spec(layout_result, recipe, e.y, fallback_spec);
                        let utilization = crate::common::utilization_for(spec);
                        let out_rate = spec
                            .outputs
                            .iter()
                            .find(|o| o.is_fluid && o.item == fluid)
                            .map(|o| o.rate * utilization)
                            .unwrap_or(0.0);
                        let in_rate = spec
                            .inputs
                            .iter()
                            .find(|i| i.is_fluid && i.item == fluid)
                            .map(|i| i.rate * utilization)
                            .unwrap_or(0.0);
                        if out_rate <= 0.0 && in_rate <= 0.0 {
                            continue;
                        }
                        // A producer's output ports may touch several
                        // components (severed at the machine) — share its
                        // rate across them rather than double-count. A
                        // consumer draws its full need from the first
                        // component holding one of its input ports.
                        let mut out_comps: Vec<usize> = Vec::new();
                        let mut in_comp: Option<usize> = None;
                        for &(rx, ry, pt) in fluid_ports(e.name.as_str(), e.mirror, e.direction)
                        {
                            let t = (e.x + rx, e.y + ry);
                            if !tile_set.contains(&t) {
                                continue;
                            }
                            match pt {
                                "output" if out_rate > 0.0 => {
                                    if let Some(i) = comp_of(t) {
                                        if !out_comps.contains(&i) {
                                            out_comps.push(i);
                                        }
                                    }
                                }
                                "input" if in_rate > 0.0 && in_comp.is_none() => {
                                    in_comp = comp_of(t);
                                }
                                _ => {}
                            }
                        }
                        if !out_comps.is_empty() {
                            let share = out_rate / out_comps.len() as f64;
                            for i in out_comps {
                                supply_rates[i] += share;
                            }
                        }
                        if let Some(i) = in_comp {
                            demand_rates[i] += in_rate;
                        }
                    }

                    // Normalize both books to the SOLVER's totals before
                    // comparing. Placement rounds machine counts up per
                    // copy (K-quantization: 8 copies x ceil(22/8) = 24
                    // machines for a 22-count spec), and mega-cell
                    // interiors aren't in `effective_rows`, so per-machine
                    // attribution systematically overstates totals. The
                    // solver's books are the ground truth; scaling each
                    // side to them cancels uniform rounding while
                    // preserving exactly what a real sever creates:
                    // ASYMMETRY between components (a fragment stranded
                    // without its producer stays near zero supply under
                    // any uniform scale).
                    let solver_demand_total: f64 = sr
                        .machines
                        .iter()
                        .flat_map(|m| m.inputs.iter().map(move |i| (i, m.count)))
                        .filter(|(i, _)| i.is_fluid && i.item == fluid)
                        .map(|(i, count)| i.rate * count)
                        .sum();
                    let solver_supply_total: f64 = sr
                        .machines
                        .iter()
                        .flat_map(|m| m.outputs.iter().map(move |o| (o, m.count)))
                        .filter(|(o, _)| o.is_fluid && o.item == fluid)
                        .map(|(o, count)| o.rate * count)
                        .sum::<f64>()
                        + external_rate;
                    let attributed_demand: f64 = demand_rates.iter().sum();
                    let attributed_supply: f64 = supply_rates.iter().sum();
                    if attributed_demand > 0.0 && solver_demand_total > 0.0 {
                        let k = solver_demand_total / attributed_demand;
                        for d in demand_rates.iter_mut() {
                            *d *= k;
                        }
                    }
                    if attributed_supply > 0.0 && solver_supply_total > 0.0 {
                        let k = solver_supply_total / attributed_supply;
                        for v in supply_rates.iter_mut() {
                            *v *= k;
                        }
                    }

                    let already: FxHashSet<(i32, i32)> = reps.iter().copied().collect();
                    for (i, c) in components.iter().enumerate() {
                        let rep = c.iter().copied().min().unwrap_or((0, 0));
                        if already.contains(&rep) {
                            continue; // unsupplied — reported above
                        }
                        let demand = demand_rates[i];
                        let supply = supply_rates[i];
                        // 1% + absolute epsilon: rate-model tolerance, so
                        // this stays an honest structural signal and not a
                        // rounding trap (#404's lesson on Error-severity
                        // rate checks).
                        if demand > supply + (demand * 0.01).max(0.01) {
                            issues.push(ValidationIssue::with_pos(
                                Severity::Error,
                                "fluid-network",
                                format!(
                                    "{fluid} pipe network is split into {n_components} components and the component at ({},{}) is under-supplied: attached consumers need {demand:.2}/s but reachable supply is {supply:.2}/s — likely a severed trunk (#409)",
                                    rep.0, rep.1
                                ),
                                rep.0,
                                rep.1,
                            ));
                        }
                    }
                }
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
        let issues = check_fluid_port_connectivity(&lr);
        assert!(issues.is_empty());
    }

    #[test]
    fn assembling_machine_no_pipes_skipped() {
        // assembling-machine-2 without adjacent pipes → skipped (fluid_boxes_off)
        let lr = layout(vec![
            machine("assembling-machine-2", 0, 0, "iron-gear-wheel", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr);
        assert!(issues.is_empty());
    }

    #[test]
    fn chemical_plant_no_input_pipe_error() {
        // chemical-plant at (0,0): input ports at (0,-1) and (2,-1)
        // No pipes placed → should error
        let lr = layout(vec![
            machine("chemical-plant", 0, 0, "plastic-bar", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty(), "expected fluid-connectivity error");
        assert!(errors.iter().all(|i| i.category == "fluid-connectivity"));
    }

    #[test]
    fn chemical_plant_with_input_pipe_ok() {
        // plastic-bar has no fluid output so only input check applies
        // chemical-plant at (0,0): input port at (0,-1)
        let lr = layout(vec![
            machine("chemical-plant", 0, 0, "plastic-bar", false),
            pipe(0, -1, Some("petroleum-gas")),
        ]);
        let issues = check_fluid_port_connectivity(&lr);
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
        let issues = check_fluid_port_connectivity(&lr);
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
        let issues = check_fluid_port_connectivity(&lr);
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
        let issues = check_fluid_port_connectivity(&lr);
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
        let issues = check_fluid_port_connectivity(&lr);
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
        assert!(check_fluid_network_connectivity(&lr, None).is_empty());
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
        let issues = check_fluid_network_connectivity(&lr, None);
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
        let issues = check_fluid_network_connectivity(&lr, None);
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
        assert!(check_fluid_network_connectivity(&lr, None).is_empty(),
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
        assert!(check_fluid_network_connectivity(&lr, None).is_empty());
    }

    /// #409 fixture builder: fragment A = real producer (sulfuric-acid
    /// recipe → structural supply marker holds) piped to one consumer;
    /// fragment B = boundary-supplied pipe column to another consumer.
    /// Both fragments are structurally SUPPLIED, so pre-#409 the split
    /// passed silently regardless of rates. Port tiles are computed from
    /// the shared `fluid_ports` table, not hardcoded.
    fn severed_supplied_fixture(external_acid_rate: f64) -> (LayoutResult, crate::models::SolverResult) {
        use crate::models::{BoundaryRecord, ItemFlow, MachineSpec, SolverResult};
        let ports = fluid_ports("chemical-plant", false, EntityDirection::North);
        let &(ox, oy, _) = ports.iter().find(|p| p.2 == "output").expect("chem output port");
        let &(ix, iy, _) = ports.iter().find(|p| p.2 == "input").expect("chem input port");

        let acid = "sulfuric-acid";
        let mut ents = Vec::new();
        // Fragment A: producer at (0,0); consumer aligned so its input
        // port shares the producer output port's column.
        ents.push(machine("chemical-plant", 0, 0, "sulfuric-acid", false));
        let ca_x = ox - ix;
        ents.push(machine("chemical-plant", ca_x, 8, "battery", false));
        let a_col = ox;
        let a_top = oy; // producer output port tile y (machine at y=0)
        let a_bot = 8 + iy; // consumer input port tile y
        for y in a_top..=a_bot {
            ents.push(pipe(a_col, y, Some(acid)));
        }
        // Fragment B: boundary-supplied column at x=10 to a consumer.
        let cb_x = 10 - ix;
        ents.push(machine("chemical-plant", cb_x, 8, "battery", false));
        let b_bot = 8 + iy;
        for y in 0..=b_bot {
            ents.push(pipe(10, y, Some(acid)));
        }
        let mut lr = layout(ents);
        lr.boundary_inputs = vec![BoundaryRecord {
            item: acid.to_string(),
            x: 10,
            y: 0,
            direction: EntityDirection::North,
            is_fluid: true,
            entity: "pipe".to_string(),
        }];

        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "chemical-plant".to_string(),
                    recipe: "sulfuric-acid".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 1.0,
                    inputs: vec![],
                    outputs: vec![ItemFlow {
                        item: acid.to_string(), rate: 10.0, is_fluid: true, module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "chemical-plant".to_string(),
                    recipe: "battery".to_string(),
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
                    count: 2.0,
                    inputs: vec![ItemFlow {
                        item: acid.to_string(), rate: 5.0, is_fluid: true, module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "battery".to_string(), rate: 1.0, is_fluid: false, module_id: 0,
                    }],
                },
            ],
            external_inputs: vec![ItemFlow {
                item: acid.to_string(), rate: external_acid_rate, is_fluid: true, module_id: 0,
            }],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        (lr, sr)
    }

    #[test]
    fn fluid_network_supplied_but_undersupplied_fragment_flags() {
        // Boundary feeds fragment B only 4/s against its consumer's 5/s
        // — a severed trunk whose fragments each "have a supply" but
        // whose books don't balance. Pre-#409: silent pass.
        let (lr, sr) = severed_supplied_fixture(4.0);
        let issues = check_fluid_network_connectivity(&lr, Some(&sr));
        assert_eq!(
            issues.len(), 1,
            "exactly the under-supplied fragment must flag: {:?}",
            issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
        assert!(
            issues[0].message.contains("under-supplied"),
            "message must carry the accounting: {}",
            issues[0].message
        );
    }

    #[test]
    fn fluid_network_balanced_split_components_pass() {
        // Same severed topology, but every fragment's books balance
        // (boundary 6/s ≥ 5/s demand) — the legitimate one-network-per-
        // copy shape (K>1 composed chains) must stay clean.
        let (lr, sr) = severed_supplied_fixture(6.0);
        let issues = check_fluid_network_connectivity(&lr, Some(&sr));
        assert!(
            issues.is_empty(),
            "balanced independent components are legitimate: {:?}",
            issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
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
        let issues = check_fluid_port_connectivity(&lr);
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
        let issues = check_fluid_port_connectivity(&lr);
        assert!(issues.is_empty(), "solid foundry recipe should be skipped: {issues:?}");
    }

    #[test]
    fn foundry_casting_missing_input_pipe_error() {
        // casting-iron consumes molten-iron (fluid input) — a foundry with
        // no input pipe is a genuine missing-pipe bug and MUST error.
        let lr = layout(vec![
            machine("foundry", 0, 0, "casting-iron", false),
        ]);
        let issues = check_fluid_port_connectivity(&lr);
        let errs: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errs.is_empty(), "expected missing-input-pipe error");
        assert!(errs.iter().all(|i| i.category == "fluid-connectivity"));
        assert!(errs.iter().any(|i| i.message.contains("input")));
    }
}
