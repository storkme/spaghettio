//! Shared constants and utility functions for routing.
//!
//! Port of `src/routing/common.py`.

use rustc_hash::FxHashSet;

use crate::models::{EntityDirection, MachineSpec, PlacedEntity, SolverResult};

const DEFAULT_MACHINE_SIZE: u32 = 3;
const DEFAULT_MACHINE_DIMS: (u32, u32) = (DEFAULT_MACHINE_SIZE, DEFAULT_MACHINE_SIZE);

/// Known machine entity names (the set of all crafting machines the layout engine uses).
pub const MACHINE_ENTITY_NAMES: &[&str] = &[
    "assembling-machine-1",
    "assembling-machine-2",
    "assembling-machine-3",
    "chemical-plant",
    "electric-furnace",
    "oil-refinery",
    "electromagnetic-plant",
    "cryogenic-plant",
    "foundry",
    "biochamber",
    "centrifuge",
    "recycler",
];

/// Return `true` if `entity` is a known crafting machine.
pub fn is_machine_entity(entity: &str) -> bool {
    MACHINE_ENTITY_NAMES.contains(&entity)
}

/// Recycler direct-ejection tile (RFP Fulgora Phase 0 finding,
/// `docs/rfp-fulgora-scrap.md`): the ONE tile a recycler credits its
/// output onto directly, mining-drill-style — no output inserter, per
/// `vector_to_place_result`. Only NORTH and SOUTH facing are supported
/// (matches `templates::voider_row` and the RFP's documented E/W
/// export-centering caveat — blueprint export doesn't swap width/height
/// for rotated non-square machines). `(x, y)` is the entity's placement
/// anchor (top-left tile of its 2×4 footprint). `None` for unsupported
/// directions or non-recycler entities.
pub fn recycler_eject_tile(entity: &str, x: i32, y: i32, direction: EntityDirection) -> Option<(i32, i32)> {
    if entity != "recycler" {
        return None;
    }
    match direction {
        // West column (dx=0), one tile past the north edge (Phase 0:
        // vector_to_place_result lands 0.6 tiles beyond the north edge,
        // 0.5 tiles off-center toward -x).
        EntityDirection::North => Some((x, y - 1)),
        // 180° rotation of the North case: east column (dx=1), one tile
        // past the south edge (height=4).
        EntityDirection::South => Some((x + 1, y + 4)),
        EntityDirection::East | EntityDirection::West => None,
    }
}

/// Return the footprint `(width, height)` in tiles for the given entity name.
///
/// All machines today are square, except `recycler` (Fulgora scrap recycler:
/// `tile_width=2, tile_height=4`, per `docs/rfp-fulgora-scrap.md` Phase 0
/// findings). There is deliberately no square-assuming wrapper around this —
/// every call site must pick width or height explicitly so a non-square
/// machine can't silently grab the wrong axis.
pub fn machine_dims(entity: &str) -> (u32, u32) {
    match entity {
        "assembling-machine-1" | "assembling-machine-2" | "assembling-machine-3"
        | "chemical-plant" | "electric-furnace" | "biochamber" | "centrifuge" => (3, 3),
        "oil-refinery" | "cryogenic-plant" | "foundry" => (5, 5),
        "electromagnetic-plant" => (4, 4),
        "recycler" => (2, 4),
        _ => DEFAULT_MACHINE_DIMS,
    }
}

/// Footprint `(width, height)` in tiles for ANY placed entity — the single
/// source both the blueprint center math and the power validator's pole
/// geometry consult (RFP `docs/rfp-power-supply.md` Phase 3a-i). Machines defer
/// to [`machine_dims`]; the substation is 2×2; everything else (medium/small
/// poles, belts, inserters, pipes) is 1×1. Before this, `blueprint.rs` hard-
/// coded `(1,1)` for every non-machine, so a substation would have exported at
/// center `x+0.5` instead of `x+1.0`.
pub fn entity_size(entity: &str) -> (u32, u32) {
    if is_machine_entity(entity) {
        machine_dims(entity)
    } else if entity == "substation" {
        (2, 2)
    } else {
        (1, 1)
    }
}

/// Supply-area half-extent in tiles (Chebyshev, from the pole's center) for a
/// power-distribution entity — THE shared value for the power validator's
/// coverage check and `bus::layout::place_poles`, unifying the two formerly
/// duplicated `POLE_RANGE = 3` constants (RFP Phase 3a-i).
///
/// `medium-electric-pole` is 3 (7×7 supply). `substation` is 9 (18×18 supply
/// from its 2×2 center); because the footprint is even, an integer ±9 from the
/// center tile reaches one tile past the exact supply on the + axis — a
/// conservative bound for a coverage *guardrail*, and exact placement geometry
/// is Phase 3a-ii's concern. Any other name falls back to the medium value (the
/// coverage check only ever passes it a medium pole or a substation).
pub fn pole_supply_range(entity: &str) -> i32 {
    match entity {
        "substation" => 9,
        _ => 3,
    }
}

/// Whether `entity` draws electricity from the power grid.
///
/// The codebase's first energy-source fact (RFP `docs/rfp-power-supply.md`
/// Phase 0a). Every canonical crafting machine is electric **except**
/// `biochamber`, which is burner-fueled — draftsman ground truth:
/// `energy_source.type == "burner"`, `fuel_categories: ["nutrients"]`. A
/// biochamber draws zero grid power, so power-coverage validation must not
/// flag one as unpowered, and pole placement need not reserve a pole line
/// for a pure-biochamber row.
///
/// Inserters are the one non-crafting-machine class that also draws grid
/// power (RFP Phase 0f): every tier the engine places is electric, so they
/// are power-coverage subjects too. Folding them into this single predicate
/// (rather than a parallel `is_inserter` check in the validator) keeps
/// "draws grid power" in one place, drift-test covered.
///
/// All other non-machine entities (poles, belts, pipes) return `false`, and
/// `biochamber` deliberately falls into the same `_ => false` arm; the drift
/// test in this module pins that intent (biochamber must be in
/// `MACHINE_ENTITY_NAMES` *and* classified non-electric here) so the shared
/// arm can never silently swallow a newly-added machine.
pub fn needs_electricity(entity: &str) -> bool {
    if is_inserter(entity) {
        return true;
    }
    match entity {
        "assembling-machine-1"
        | "assembling-machine-2"
        | "assembling-machine-3"
        | "chemical-plant"
        | "electric-furnace"
        | "oil-refinery"
        | "electromagnetic-plant"
        | "cryogenic-plant"
        | "foundry"
        | "centrifuge"
        | "recycler" => true,
        // `biochamber` (burner-fueled — see doc comment) lands here with
        // every non-machine entity: none draw grid power.
        _ => false,
    }
}

/// Candidate pole rows for a machine row whose machines start at `top_y`
/// with footprint height `mh`: the row just above the machine (`top_y-1`,
/// the input-inserter band) and just below (`top_y+mh`, the output-inserter
/// band). `top_y-1` is dropped when it would be negative (the topmost row).
///
/// THE shared source of truth for where a row's poles sit (RFP
/// `docs/rfp-power-supply.md`). `bus::layout::place_poles` seeds each band's
/// outward search from these rows; Phase 1's fluid-row reservation reserves
/// gap tiles in exactly these rows — one function so the placer and the
/// reservation can never drift (the geometry dual of the Phase 0b name-list
/// unification).
pub fn pole_candidate_ys(top_y: i32, mh: i32) -> Vec<i32> {
    let mut ys = Vec::with_capacity(2);
    if top_y > 0 {
        ys.push(top_y - 1);
    }
    ys.push(top_y + mh);
    ys
}

/// All tile coordinates occupied by a machine at `(x, y)` with footprint `(w, h)`.
pub fn machine_tiles(x: i32, y: i32, w: u32, h: u32) -> Vec<(i32, i32)> {
    let (w, h) = (w as i32, h as i32);
    (0..w)
        .flat_map(move |dx| (0..h).map(move |dy| (x + dx, y + dy)))
        .collect()
}

/// Fraction of a full machine-slot the row's fractional `spec.count` actually
/// fills. `effective_count` is the whole number of physical machine slots the
/// row must place (at least 1); the returned utilization scales per-machine
/// rates down so a partial last machine isn't credited with a full machine's
/// throughput.
///
/// THE single source of this formula, shared by placement (`bus::placer`,
/// which sizes belts/inserters per machine) and validation (`validate::`,
/// which must reconstruct the same effective rate to avoid false-positive
/// starvation/overflow warnings on partitioned rows) — see
/// `docs/rfp-inserter-sizing.md`'s honest-zero gate.
pub fn utilization_for(spec: &MachineSpec) -> f64 {
    let effective_count = spec.count.ceil().max(1.0);
    (spec.count / effective_count).min(1.0)
}

/// Belt throughput tiers: (entity name, items-per-second capacity).
pub const BELT_TIERS: &[(&str, f64)] = &[
    ("transport-belt", 15.0),
    ("fast-transport-belt", 30.0),
    ("express-transport-belt", 45.0),
];

/// Underground belt max reach (tiles between entry and exit, exclusive).
pub fn ug_max_reach(belt: &str) -> u32 {
    match belt {
        "transport-belt" => 4,
        "fast-transport-belt" => 6,
        "express-transport-belt" => 8,
        _ => 4,
    }
}

/// Cost multiplier for underground belt tiles vs surface.
pub const UG_COST_MULTIPLIER: u32 = 5;

/// Pipe-to-ground max reach (tiles between entry and exit, exclusive).
pub const UG_PIPE_REACH: u32 = 10;

/// Full belt throughput (both lanes combined) for the given belt entity.
pub fn belt_throughput(belt: &str) -> f64 {
    BELT_TIERS
        .iter()
        .find(|(name, _)| *name == belt)
        .map(|(_, rate)| *rate)
        .unwrap_or(15.0)
}

/// Per-lane capacity (half of total belt throughput).
pub fn lane_capacity(belt: &str) -> f64 {
    belt_throughput(belt) / 2.0
}

// ---------------------------------------------------------------------------
// Entity classification helpers (shared across validation modules)
// ---------------------------------------------------------------------------

/// Surface (above-ground) belt entity names.
pub const SURFACE_BELT_ENTITIES: &[&str] =
    &["transport-belt", "fast-transport-belt", "express-transport-belt"];

/// Underground belt entity names.
pub const UG_BELT_ENTITIES: &[&str] = &[
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt",
];

/// Splitter entity names.
pub const SPLITTER_ENTITIES: &[&str] =
    &["splitter", "fast-splitter", "express-splitter"];

/// Inserter entity names.
pub const INSERTER_ENTITIES: &[&str] =
    &["inserter", "long-handed-inserter", "fast-inserter", "stack-inserter"];

/// Return `true` if `name` is a surface (above-ground) belt.
pub fn is_surface_belt(name: &str) -> bool {
    SURFACE_BELT_ENTITIES.contains(&name)
}

/// Return `true` if `name` is an underground belt.
pub fn is_ug_belt(name: &str) -> bool {
    UG_BELT_ENTITIES.contains(&name)
}

/// Return `true` if `name` is a splitter.
pub fn is_splitter(name: &str) -> bool {
    SPLITTER_ENTITIES.contains(&name)
}

/// Return `true` if `name` is any belt-type entity (surface, underground, or splitter).
pub fn is_belt_entity(name: &str) -> bool {
    is_surface_belt(name) || is_ug_belt(name) || is_splitter(name)
}

/// Return `true` if `name` is an inserter.
pub fn is_inserter(name: &str) -> bool {
    INSERTER_ENTITIES.contains(&name)
}

/// Return `true` if an entity with this name must stay `forbidden`
/// inside a junction SAT zone.
///
/// Surface belts are the only zone-permissive entity: SAT may lift and
/// re-stamp them as surface belts, undergrounds, or different
/// directions as long as item flow is preserved. Everything else
/// (splitters, UG entrances/exits, inserters, machines, poles, pipes)
/// must remain as-is — those tiles stay in `forbidden_tiles`. Unknown
/// names default to forbidden (conservative).
pub fn tile_is_forbidden_kind(name: &str) -> bool {
    !is_surface_belt(name)
}

/// Classify a balancer `segment_id` as a "simple" single-splitter block
/// (shapes 1x1, 1x2, 2x1, 2x2) that the junction solver may grow into
/// and re-route through, vs a multi-splitter block that must be treated
/// as a hard boundary.
///
/// The segment_id format is `balancer:{item}:{n}x{m}[:rest]`. Older
/// format `balancer:{item}` (no shape) or any parse failure returns
/// `false` — conservative: unknown balancers are treated as multi-
/// splitter, so the junction solver will route around them.
pub fn balancer_seg_is_simple(seg: &str) -> bool {
    let Some(rest) = seg.strip_prefix("balancer:") else {
        return false;
    };
    let mut parts = rest.splitn(3, ':');
    let _item = parts.next();
    let Some(shape) = parts.next() else {
        return false;
    };
    let Some((n, m)) = shape.split_once('x') else {
        return false;
    };
    let (Ok(n), Ok(m)) = (n.parse::<u32>(), m.parse::<u32>()) else {
        return false;
    };
    n <= 2 && m <= 2
}

/// Inserter reach: how many tiles away the pick-up / drop position is.
pub fn inserter_reach(name: &str) -> i32 {
    if name == "long-handed-inserter" {
        2
    } else {
        1
    }
}

/// Approximate steady-state throughput (items/second) of an inserter,
/// from `docs/factorio-mechanics.md` table I8.
///
/// Assumption: **no inserter-capacity / stack-bonus research** — these are
/// the base (unresearched) rates a fresh factory sees. Values are
/// `rotation_speed × 60 × items_per_swing`, adjusted for the observed
/// extension delay on the fast inserter (table I8 note). Bases:
/// regular ~0.84/s, long-handed ~1.2/s (faster per cycle than regular —
/// the "long arm = slow" intuition is wrong, per I8), fast ~2.31/s,
/// stack ~12/s base, bulk 2.4/s base. Unknown names fall back to the
/// regular rate (conservative — never over-credits an unknown inserter).
pub fn inserter_throughput(name: &str) -> f64 {
    match name {
        "inserter" => 0.84,
        "long-handed-inserter" => 1.2,
        "fast-inserter" => 2.31,
        "stack-inserter" => 12.0,
        "bulk-inserter" => 2.4,
        _ => 0.84,
    }
}

/// Map underground-belt entity name to its corresponding surface belt tier.
pub fn ug_to_surface_tier(ug_name: &str) -> &'static str {
    match ug_name {
        "underground-belt" => "transport-belt",
        "fast-underground-belt" => "fast-transport-belt",
        "express-underground-belt" => "express-transport-belt",
        _ => "transport-belt",
    }
}

/// Map splitter entity name to its corresponding surface belt tier.
pub fn splitter_to_surface_tier(splitter: &str) -> &'static str {
    match splitter {
        "splitter" => "transport-belt",
        "fast-splitter" => "fast-transport-belt",
        "express-splitter" => "express-transport-belt",
        _ => "transport-belt",
    }
}

// ---------------------------------------------------------------------------
// Tile helpers for entities that span multiple tiles
// ---------------------------------------------------------------------------

/// Second tile occupied by a splitter (perpendicular to flow direction).
pub fn splitter_second_tile(e: &PlacedEntity) -> (i32, i32) {
    match e.direction {
        EntityDirection::North | EntityDirection::South => (e.x + 1, e.y),
        _ => (e.x, e.y + 1),
    }
}

/// Collect the set of recipes whose inputs and outputs are all fluids (no solid items).
pub fn fluid_only_recipes(solver: Option<&SolverResult>) -> FxHashSet<String> {
    let mut out = rustc_hash::FxHashSet::default();
    if let Some(sr) = solver {
        for spec in &sr.machines {
            let has_solid = spec
                .inputs
                .iter()
                .chain(spec.outputs.iter())
                .any(|f| !f.is_fluid);
            if !has_solid {
                out.insert(spec.recipe.clone());
            }
        }
    }
    out
}

/// Pick the cheapest belt tier whose throughput is `>= rate`.
///
/// If `max_tier` is `Some(name)`, never select a higher tier than that.
pub fn belt_entity_for_rate(rate: f64, max_tier: Option<&str>) -> &'static str {
    let max_idx = if let Some(max) = max_tier {
        BELT_TIERS
            .iter()
            .position(|(name, _)| *name == max)
            .unwrap_or(BELT_TIERS.len() - 1)
    } else {
        BELT_TIERS.len() - 1
    };

    for (i, &(name, throughput)) in BELT_TIERS.iter().enumerate() {
        if i > max_idx {
            break;
        }
        if rate <= throughput {
            return name;
        }
    }
    BELT_TIERS[max_idx].0
}

/// Cardinal direction vectors `(dx, dy)` in order N, E, S, W.
pub const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Convert a `(dx, dy)` unit vector to an `EntityDirection`, or `None` for non-cardinal inputs.
pub fn dir_from_vec(dx: i32, dy: i32) -> Option<EntityDirection> {
    match (dx, dy) {
        (0, -1) => Some(EntityDirection::North),
        (1, 0) => Some(EntityDirection::East),
        (0, 1) => Some(EntityDirection::South),
        (-1, 0) => Some(EntityDirection::West),
        _ => None,
    }
}

/// Convert an `EntityDirection` to its `(dx, dy)` vector.
pub fn dir_to_vec(dir: EntityDirection) -> (i32, i32) {
    match dir {
        EntityDirection::North => (0, -1),
        EntityDirection::East => (1, 0),
        EntityDirection::South => (0, 1),
        EntityDirection::West => (-1, 0),
    }
}

/// Belt lane: left relative to belt travel direction.
pub const LANE_LEFT: &str = "left";

/// Belt lane: right relative to belt travel direction.
pub const LANE_RIGHT: &str = "right";

/// Segment-id marker for merge-and-tap **priority tap** branches
/// (RFP `docs/rfp-merge-tap-trunks.md` D4). An inline splitter on a shared
/// trunk whose feed branch's downstream belt carries a segment id containing
/// this tag is modeled by the lane-rate walkers with the priority rate law
/// (`loop_priority_rate`: the feed branch receives `min(total, cap)`, the
/// trunk continuation the remainder) instead of the generic even split,
/// mirroring the `:selfloop:` precedent.
///
/// Deliberately distinct from the ghost router's generic tap segment ids
/// (`ghost:tap:*`, `tapoff:*`): those are plain 50/50 splitters and must NOT
/// be re-modeled or flagged as priority taps — a bare `:tap:` substring would
/// collide with `ghost:tap:*` and break the fallback's inertness. Keep this
/// the single source of truth so the Checkpoint B stamper and the validators
/// cannot drift.
pub const MERGE_TAP_SEGMENT_TAG: &str = ":mergetap:";

/// Return which lane an inserter places items on (the far lane).
///
/// The inserter sits on one side of the belt (left or right relative to belt
/// direction); items land on the opposite (far) lane.
pub fn inserter_target_lane(
    ins_x: i32,
    ins_y: i32,
    belt_x: i32,
    belt_y: i32,
    belt_dir: EntityDirection,
) -> &'static str {
    let (dx, dy) = dir_to_vec(belt_dir);
    // Left perpendicular (CCW 90° of belt direction vector)
    let (left_dx, left_dy) = (-dy, dx);
    let dot = (ins_x - belt_x) * left_dx + (ins_y - belt_y) * left_dy;
    // Inserter on left side → items land on right (far) lane, and vice versa.
    // dot == 0 means directly in-line; default to left.
    if dot > 0 { LANE_RIGHT } else { LANE_LEFT }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_dims_assembling_3() {
        assert_eq!(machine_dims("assembling-machine-3"), (3, 3));
    }

    #[test]
    fn machine_dims_oil_refinery() {
        assert_eq!(machine_dims("oil-refinery"), (5, 5));
    }

    #[test]
    fn machine_dims_default() {
        assert_eq!(machine_dims("unknown-machine"), DEFAULT_MACHINE_DIMS);
    }

    #[test]
    fn machine_dims_space_age() {
        assert_eq!(machine_dims("electromagnetic-plant"), (4, 4));
        assert_eq!(machine_dims("cryogenic-plant"), (5, 5));
        assert_eq!(machine_dims("foundry"), (5, 5));
        assert_eq!(machine_dims("biochamber"), (3, 3));
    }

    #[test]
    fn machine_dims_recycler_non_square() {
        assert_eq!(machine_dims("recycler"), (2, 4));
    }

    #[test]
    fn space_age_machines_in_entity_list() {
        for name in ["electromagnetic-plant", "cryogenic-plant", "foundry", "biochamber"] {
            assert!(MACHINE_ENTITY_NAMES.contains(&name), "{name} missing from MACHINE_ENTITY_NAMES");
            assert!(is_machine_entity(name), "{name} not recognized by is_machine_entity");
        }
    }

    #[test]
    fn needs_electricity_biochamber_is_burner() {
        // Biochamber is burner-fueled (fuel category nutrients) — the one
        // canonical machine that draws no grid power.
        assert!(!needs_electricity("biochamber"));
        // Every other canonical machine is electric.
        for name in [
            "assembling-machine-1",
            "assembling-machine-2",
            "assembling-machine-3",
            "chemical-plant",
            "electric-furnace",
            "oil-refinery",
            "electromagnetic-plant",
            "cryogenic-plant",
            "foundry",
            "centrifuge",
            "recycler",
        ] {
            assert!(needs_electricity(name), "{name} should be electric");
        }
        // Non-machine entities draw no grid power.
        assert!(!needs_electricity("medium-electric-pole"));
        assert!(!needs_electricity("transport-belt"));
    }

    /// Drift regression (RFP `docs/rfp-power-supply.md` Phase 0b): every
    /// machine in the canonical `MACHINE_ENTITY_NAMES` list must be
    /// classified for BOTH energy source (`needs_electricity`) and fluid
    /// ports (`validate::fluids::machine_has_fluid_ports`). Adding a machine
    /// to the canonical list without adding it to `CLASSIFIED` below fails
    /// this test (set-equality check), forcing an explicit, ground-truthed
    /// decision on both facts rather than letting a new machine silently
    /// inherit a `_ => false` / empty-port-table default — which is exactly
    /// the twin validator blind spot this RFP closes.
    #[test]
    fn machine_classification_no_drift() {
        // (name, needs_electricity, has_fluid_ports) — ground-truthed from
        // game data (recipes.json `machines.*.fluid_boxes`; biochamber's
        // burner energy source is draftsman-verified).
        const CLASSIFIED: &[(&str, bool, bool)] = &[
            ("assembling-machine-1", true, false),
            ("assembling-machine-2", true, true),
            ("assembling-machine-3", true, true),
            ("chemical-plant", true, true),
            ("electric-furnace", true, false),
            ("oil-refinery", true, true),
            ("electromagnetic-plant", true, true),
            ("cryogenic-plant", true, true),
            ("foundry", true, true),
            ("biochamber", false, true),
            ("centrifuge", true, false),
            ("recycler", true, false),
        ];

        // 1. Set-equality: CLASSIFIED covers exactly the canonical list. A
        //    machine added to one but not the other trips here.
        let classified_names: FxHashSet<&str> =
            CLASSIFIED.iter().map(|(n, _, _)| *n).collect();
        let canonical_names: FxHashSet<&str> =
            MACHINE_ENTITY_NAMES.iter().copied().collect();
        assert_eq!(
            classified_names, canonical_names,
            "machine classification table out of sync with MACHINE_ENTITY_NAMES — \
             every canonical machine must be classified for energy AND fluid ports"
        );

        // 2. Each classification matches the real predicates.
        for &(name, expect_elec, expect_fluid) in CLASSIFIED {
            assert_eq!(
                needs_electricity(name),
                expect_elec,
                "{name}: needs_electricity disagrees with classification table"
            );
            assert_eq!(
                crate::validate::machine_has_fluid_ports(name),
                expect_fluid,
                "{name}: fluid-port presence disagrees with classification table"
            );
        }
    }

    #[test]
    fn inserters_are_electric_coverage_subjects() {
        // RFP Phase 0f: every inserter tier the engine places draws grid
        // power and is folded into `needs_electricity`. Adding an inserter
        // name to `INSERTER_ENTITIES` without it being electric would strand
        // it from power coverage silently — this pins the whole list.
        for &name in INSERTER_ENTITIES {
            assert!(
                needs_electricity(name),
                "{name} is an inserter and must be an electric coverage subject"
            );
            assert!(is_inserter(name), "{name} must be recognized by is_inserter");
        }
        // And a machine/belt must NOT be mistaken for an inserter here.
        assert!(!is_inserter("assembling-machine-1"));
        assert!(!is_inserter("transport-belt"));
    }

    #[test]
    fn machine_tiles_3x3() {
        let tiles = machine_tiles(0, 0, 3, 3);
        assert_eq!(tiles.len(), 9);
        assert!(tiles.contains(&(0, 0)));
        assert!(tiles.contains(&(2, 2)));
    }

    #[test]
    fn machine_tiles_2x4_non_square() {
        // Recycler footprint: 2 wide, 4 tall.
        let tiles = machine_tiles(0, 0, 2, 4);
        assert_eq!(tiles.len(), 8);
        for x in 0..2 {
            for y in 0..4 {
                assert!(tiles.contains(&(x, y)), "missing tile ({x}, {y})");
            }
        }
        assert!(!tiles.contains(&(2, 0)), "tile beyond width should not be included");
        assert!(!tiles.contains(&(0, 4)), "tile beyond height should not be included");
    }

    #[test]
    fn belt_entity_for_rate_low() {
        assert_eq!(belt_entity_for_rate(10.0, None), "transport-belt");
    }

    #[test]
    fn belt_entity_for_rate_exact_15() {
        assert_eq!(belt_entity_for_rate(15.0, None), "transport-belt");
    }

    #[test]
    fn belt_entity_for_rate_mid() {
        assert_eq!(belt_entity_for_rate(20.0, None), "fast-transport-belt");
    }

    #[test]
    fn belt_entity_for_rate_high() {
        assert_eq!(belt_entity_for_rate(40.0, None), "express-transport-belt");
    }

    #[test]
    fn belt_entity_for_rate_capped_by_max_tier() {
        assert_eq!(
            belt_entity_for_rate(40.0, Some("transport-belt")),
            "transport-belt"
        );
    }

    #[test]
    fn ug_max_reach_values() {
        assert_eq!(ug_max_reach("transport-belt"), 4);
        assert_eq!(ug_max_reach("fast-transport-belt"), 6);
        assert_eq!(ug_max_reach("express-transport-belt"), 8);
    }

    #[test]
    fn lane_capacity_values() {
        assert_eq!(lane_capacity("transport-belt"), 7.5);
        assert_eq!(lane_capacity("fast-transport-belt"), 15.0);
        assert_eq!(lane_capacity("express-transport-belt"), 22.5);
    }

    #[test]
    fn dir_roundtrip() {
        for dir in [
            EntityDirection::North,
            EntityDirection::East,
            EntityDirection::South,
            EntityDirection::West,
        ] {
            let (dx, dy) = dir_to_vec(dir);
            assert_eq!(dir_from_vec(dx, dy), Some(dir));
        }
    }

    #[test]
    fn inserter_target_lane_north_belt_inserter_left() {
        // North belt, inserter to the east (left side) → far lane is right.
        assert_eq!(inserter_target_lane(1, 0, 0, 0, EntityDirection::North), LANE_RIGHT);
    }

    #[test]
    fn inserter_target_lane_north_belt_inserter_right() {
        // North belt, inserter to the west (right side) → far lane is left.
        assert_eq!(inserter_target_lane(-1, 0, 0, 0, EntityDirection::North), LANE_LEFT);
    }

    #[test]
    fn inserter_target_lane_east_belt() {
        // East belt, inserter to the south (left side) → far lane is right.
        assert_eq!(inserter_target_lane(0, 1, 0, 0, EntityDirection::East), LANE_RIGHT);
    }

    #[test]
    fn inserter_target_lane_default_inline() {
        // Inserter directly in front of belt → defaults to left lane.
        assert_eq!(inserter_target_lane(0, -1, 0, 0, EntityDirection::North), LANE_LEFT);
    }

    #[test]
    fn inserter_target_lane_south_belt() {
        // South belt, inserter to the west (left side of southward travel) → far lane is right.
        assert_eq!(inserter_target_lane(-1, 0, 0, 0, EntityDirection::South), LANE_RIGHT);
        // South belt, inserter to the east (right side) → far lane is left.
        assert_eq!(inserter_target_lane(1, 0, 0, 0, EntityDirection::South), LANE_LEFT);
    }

    #[test]
    fn inserter_target_lane_west_belt() {
        // West belt, inserter to the north (left side of westward travel) → far lane is right.
        assert_eq!(inserter_target_lane(0, -1, 0, 0, EntityDirection::West), LANE_RIGHT);
        // West belt, inserter to the south (right side) → far lane is left.
        assert_eq!(inserter_target_lane(0, 1, 0, 0, EntityDirection::West), LANE_LEFT);
    }

    #[test]
    fn machine_dims_chemical_plant_and_furnace() {
        assert_eq!(machine_dims("chemical-plant"), (3, 3));
        assert_eq!(machine_dims("electric-furnace"), (3, 3));
    }

    #[test]
    fn machine_tiles_offset_origin() {
        // Tiles should be offset by (x, y), not always start at (0, 0).
        let tiles = machine_tiles(5, 3, 3, 3);
        assert_eq!(tiles.len(), 9);
        assert!(tiles.contains(&(5, 3)));
        assert!(tiles.contains(&(7, 5)));
        assert!(!tiles.contains(&(0, 0)));
    }

    #[test]
    fn belt_throughput_values() {
        assert_eq!(belt_throughput("transport-belt"), 15.0);
        assert_eq!(belt_throughput("fast-transport-belt"), 30.0);
        assert_eq!(belt_throughput("express-transport-belt"), 45.0);
        // Unknown belt falls back to 15.0
        assert_eq!(belt_throughput("unknown-belt"), 15.0);
    }

    #[test]
    fn dir_from_vec_diagonal_returns_none() {
        assert_eq!(dir_from_vec(1, 1), None);
        assert_eq!(dir_from_vec(0, 0), None);
    }
}

// ---- Module and beacon data ----

/// Speed and productivity bonuses for a module type.
#[derive(Debug, Clone, Copy)]
pub struct ModuleEffect {
    pub speed: f64,
    pub productivity: f64,
}

/// Look up module effects by entity name. Returns (0,0) for unknown modules.
pub fn module_effect(name: &str) -> ModuleEffect {
    match name {
        "speed-module" => ModuleEffect { speed: 0.2, productivity: 0.0 },
        "speed-module-2" => ModuleEffect { speed: 0.3, productivity: 0.0 },
        "speed-module-3" => ModuleEffect { speed: 0.5, productivity: 0.0 },
        "productivity-module" => ModuleEffect { speed: -0.05, productivity: 0.04 },
        "productivity-module-2" => ModuleEffect { speed: -0.10, productivity: 0.06 },
        "productivity-module-3" => ModuleEffect { speed: -0.15, productivity: 0.10 },
        "quality-module" => ModuleEffect { speed: -0.05, productivity: 0.0 },
        "quality-module-2" => ModuleEffect { speed: -0.05, productivity: 0.0 },
        "quality-module-3" => ModuleEffect { speed: -0.05, productivity: 0.0 },
        // Efficiency modules have no speed/productivity effect
        _ => ModuleEffect { speed: 0.0, productivity: 0.0 },
    }
}

/// Number of module slots for a machine entity.
pub fn module_slots(entity: &str) -> u32 {
    match entity {
        "assembling-machine-1" | "stone-furnace" | "steel-furnace" => 0,
        "assembling-machine-2" | "electric-furnace" | "centrifuge" | "crusher" | "lab" => 2,
        "chemical-plant" | "oil-refinery" => 3,
        "assembling-machine-3" | "rocket-silo" | "foundry" | "biochamber" | "biolab"
        | "recycler" => 4,
        "electromagnetic-plant" => 5,
        "cryogenic-plant" => 8,
        "beacon" => 2,
        _ => 0,
    }
}

/// Beacon supply area distance (tiles from edge of 3×3 beacon).
pub const BEACON_SUPPLY_DISTANCE: i32 = 3;

/// Beacon distribution effectivity in Factorio 2.0.
/// In 2.0 this is distance-based, but for tile-distance=0 it's 1.5.
/// For a simple model we use 0.5 (the Factorio 1.x value and a reasonable
/// average across distances in 2.0 with the profile falloff).
pub const BEACON_DISTRIBUTION_EFFECTIVITY: f64 = 0.5;
