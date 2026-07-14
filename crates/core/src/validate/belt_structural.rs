//! Belt-structural validation checks.
//!
//! Port of the belt-related checks from `src/validate.py`:
//! - `check_belt_loops`
//! - `check_belt_dead_ends`
//! - `check_belt_item_isolation`
//! - `check_belt_inserter_conflict`
//! - `check_belt_throughput`
//! - `check_lane_throughput` (with `compute_lane_rates`)
//! - `check_output_belt_coverage`

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{
    belt_throughput, dir_to_vec, inserter_reach, inserter_target_lane, is_belt_entity,
    is_inserter, is_machine_entity, is_splitter, is_surface_belt, is_ug_belt,
    splitter_second_tile, splitter_to_surface_tier, ug_to_surface_tier, lane_capacity,
    machine_dims, machine_tiles, utilization_for, LANE_LEFT, LANE_RIGHT,
    MERGE_TAP_SEGMENT_TAG,
};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity, SolverResult};

use super::{Severity, ValidationIssue};

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build a map `(x, y) → EntityDirection` for all belt entities (expanding
/// splitters to both tiles).
fn belt_dir_map(entities: &[PlacedEntity]) -> FxHashMap<(i32, i32), EntityDirection> {
    let mut map = FxHashMap::default();
    for e in entities {
        if !is_belt_entity(&e.name) {
            continue;
        }
        map.insert((e.x, e.y), e.direction);
        if is_splitter(&e.name) {
            map.insert(splitter_second_tile(e), e.direction);
        }
    }
    map
}

/// All tiles occupied by belt entities (expanding splitters).
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

/// Build underground belt pair map: entry ↔ exit (bidirectional).
fn build_ug_pairs(entities: &[PlacedEntity]) -> FxHashMap<(i32, i32), (i32, i32)> {
    let ug_inputs: Vec<&PlacedEntity> = entities
        .iter()
        .filter(|e| is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input"))
        .collect();
    let ug_outputs: Vec<&PlacedEntity> = entities
        .iter()
        .filter(|e| is_ug_belt(&e.name) && e.io_type.as_deref() == Some("output"))
        .collect();

    let mut pairs = FxHashMap::default();
    let mut used_outputs: FxHashSet<(i32, i32)> = FxHashSet::default();

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

/// All tiles occupied by machines.
fn build_machine_tile_set(entities: &[PlacedEntity]) -> FxHashSet<(i32, i32)> {
    let mut tiles = FxHashSet::default();
    for e in entities {
        if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    tiles.insert((e.x + dx, e.y + dy));
                }
            }
        }
    }
    tiles
}

// ---------------------------------------------------------------------------
// check_belt_loops
// ---------------------------------------------------------------------------

/// Check for belt loops (cycles where items circulate forever).
pub fn check_belt_loops(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let bdm = belt_dir_map(&layout.entities);

    // Balancer templates legitimately contain loops (splitter recirculation),
    // and so do self-loop rows (e.g. kovarex enrichment): a priority
    // splitter deliberately recirculates its loop-back branch. Voider
    // rows (RFP Fulgora Phase 2, `docs/rfp-fulgora-scrap.md` D1) are the
    // same shape without a splitter — a recycler bank recirculates its
    // own ejected output 100% back into its own input, a genuine
    // physical cycle. Exclude tiles belonging to any of the three from
    // loop detection.
    let balancer_tiles: FxHashSet<(i32, i32)> = layout.entities.iter()
        .filter(|e| e.segment_id.as_deref().is_some_and(|s| {
            s.starts_with("balancer:") || s.contains(":selfloop:") || s.contains(":voider:")
        }))
        .map(|e| (e.x, e.y))
        .collect();

    // UG-belt tunnels: when items hit a UG-input they jump to the paired
    // UG-output and continue from (output + direction step). Walking
    // surface-only treats UG-inputs as if items step to the next surface
    // tile, which fabricates loops when an unrelated belt happens to be
    // at the UG-input's "ahead" tile and chains back around. Build the
    // pair map so the walker can jump.
    let ug_pairs = build_ug_pairs(&layout.entities);
    let ug_inputs: FxHashSet<(i32, i32)> = layout.entities.iter()
        .filter(|e| is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input"))
        .map(|e| (e.x, e.y))
        .collect();

    let mut issues = Vec::new();
    let mut confirmed: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut reported_loops: FxHashSet<Vec<(i32, i32)>> = FxHashSet::default();

    for &start in bdm.keys() {
        if confirmed.contains(&start) || balancer_tiles.contains(&start) {
            continue;
        }

        let mut visited_order: Vec<(i32, i32)> = Vec::new();
        let mut visited_set: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut cur = start;

        while bdm.contains_key(&cur) && !visited_set.contains(&cur) {
            visited_set.insert(cur);
            visited_order.push(cur);
            // If `cur` is a UG-input, jump the tunnel: items emerge at the
            // paired UG-output, then step forward from there.
            if ug_inputs.contains(&cur) {
                if let Some(&peer) = ug_pairs.get(&cur) {
                    if visited_set.contains(&peer) {
                        cur = peer;
                        break;
                    }
                    visited_set.insert(peer);
                    visited_order.push(peer);
                    let d = bdm.get(&peer).copied().unwrap_or(bdm[&cur]);
                    let (dx, dy) = dir_to_vec(d);
                    cur = (peer.0 + dx, peer.1 + dy);
                    continue;
                }
                // Unpaired UG-input — items go nowhere; treat as walked-off.
                break;
            }
            let d = bdm[&cur];
            let (dx, dy) = dir_to_vec(d);
            cur = (cur.0 + dx, cur.1 + dy);
        }

        if visited_set.contains(&cur) {
            let cycle_start_idx = visited_order.iter().position(|&t| t == cur).unwrap_or(0);
            let mut loop_tiles: Vec<(i32, i32)> = visited_order[cycle_start_idx..].to_vec();
            loop_tiles.sort();

            // Skip loops that include balancer tiles — splitter recirculation is valid.
            let involves_balancer = loop_tiles.iter().any(|t| balancer_tiles.contains(t));
            if !involves_balancer && !reported_loops.contains(&loop_tiles) {
                reported_loops.insert(loop_tiles.clone());
                let rep = loop_tiles[0];
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "belt-loop",
                    format!(
                        "Belt loop detected: {} tiles form a cycle near ({},{})",
                        loop_tiles.len(),
                        rep.0,
                        rep.1
                    ),
                    rep.0,
                    rep.1,
                ));
            }
        }

        confirmed.extend(visited_set);
    }

    issues
}

// ---------------------------------------------------------------------------
// check_tap_splitter_priority
// ---------------------------------------------------------------------------

/// The physical output tile a splitter's `output_priority` designates, or
/// `None` if no priority is set. `output_priority` is `"left"` / `"right"`
/// relative to the belt's travel direction; with `+y` = south this maps to
/// the two occupied tiles by the left-hand vector `(dy, -dx)` — matching the
/// lane orientation table in `docs/factorio-mechanics.md` B3 (e.g. facing
/// East, "left" is the north tile, "right" the south tile). Verified against
/// the in-game self-loop template (`bus/templates.rs`: an East splitter with
/// its loop-back output on the south tile carries `output_priority: "right"`).
fn priority_output_tile(e: &PlacedEntity) -> Option<(i32, i32)> {
    let op = e.output_priority.as_deref()?;
    let (dx, dy) = dir_to_vec(e.direction);
    let (lx, ly) = (dy, -dx); // left-hand vector when facing (dx, dy)
    let m = (e.x, e.y);
    let s = splitter_second_tile(e);
    let delta = (s.0 - m.0, s.1 - m.1);
    let s_is_left = delta.0 * lx + delta.1 * ly > 0;
    let (left_tile, right_tile) = if s_is_left { (s, m) } else { (m, s) };
    match op {
        LANE_LEFT => Some(left_tile),
        LANE_RIGHT => Some(right_tile),
        _ => None,
    }
}

/// Structural direction check for merge-and-tap **priority taps** (RFP
/// `docs/rfp-merge-tap-trunks.md` D4). An inline tap splitter on a shared
/// trunk sends one output to the consumer row (the *feed* branch, whose
/// downstream belt is tagged [`MERGE_TAP_SEGMENT_TAG`]) and continues the bus
/// on the other. Factorio 2.0 `output_priority` must point at the feed branch
/// so consumers are fed first and the surplus flows on — the "consumers fed
/// first, surplus continues" bus discipline the lane-rate walkers model via
/// `loop_priority_rate`.
///
/// This closes the D4 validation gap: the walkers key the priority rate law
/// off the segment tag alone, so a **backwards-stamped tap** (priority on the
/// trunk continuation instead of the feed) or a tap with no priority set at
/// all would be re-modeled as priority-fed by the validator while the exported
/// splitter behaves as an even 50/50 in game — a silent false PASS. Emitted as
/// errors:
/// - priority points at the trunk continuation (backwards tap);
/// - a tap splitter with no `output_priority` set;
/// - both outputs tagged as feed branches (ambiguous — the walker cannot
///   assign a single priority branch, so the tag would re-model a splitter
///   with no well-defined priority; containment discipline, mirroring the
///   sushi-boundary precedent that the tag only re-models what it validly
///   marks).
///
/// Purely structural (no `SolverResult`), and inert on every layout without a
/// `MERGE_TAP_SEGMENT_TAG` segment — nothing emits that tag before Checkpoint
/// B, and the ghost router's generic `ghost:tap:*` / `tapoff:*` taps do not
/// contain it.
pub fn check_tap_splitter_priority(layout: &LayoutResult) -> Vec<ValidationIssue> {
    // Segment id of every belt tile (both splitter tiles expanded), so we can
    // read the segment of the tile immediately downstream of each output.
    let mut belt_segment: FxHashMap<(i32, i32), Option<&str>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_segment.insert((e.x, e.y), e.segment_id.as_deref());
            if is_splitter(&e.name) {
                belt_segment.insert(splitter_second_tile(e), e.segment_id.as_deref());
            }
        }
    }

    let is_tap = |tile: (i32, i32)| -> bool {
        belt_segment
            .get(&tile)
            .copied()
            .flatten()
            .is_some_and(|s| s.contains(MERGE_TAP_SEGMENT_TAG))
    };

    let mut issues = Vec::new();
    for e in &layout.entities {
        if !is_splitter(&e.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(e.direction);
        let m = (e.x, e.y);
        let s = splitter_second_tile(e);
        let m_feed = is_tap((m.0 + dx, m.1 + dy));
        let s_feed = is_tap((s.0 + dx, s.1 + dy));

        // Not a priority tap (no tagged branch): the generic even-split model
        // owns it — leave it alone.
        if !m_feed && !s_feed {
            continue;
        }

        if m_feed && s_feed {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "tap-priority",
                format!(
                    "Tap splitter at ({},{}) has both outputs tagged as feed \
                     branches — a priority tap needs exactly one feed branch and \
                     one trunk continuation so the walker can assign a single \
                     priority output",
                    e.x, e.y,
                ),
                e.x,
                e.y,
            ));
            continue;
        }

        let feed_tile = if m_feed { m } else { s };
        match priority_output_tile(e) {
            Some(p) if p == feed_tile => {}
            Some(_) => issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "tap-priority",
                format!(
                    "Tap splitter at ({},{}) has output_priority pointing at the \
                     trunk continuation, not the feed branch — a backwards-stamped \
                     tap starves the consumer row (priority must point at the \
                     tapped output)",
                    e.x, e.y,
                ),
                e.x,
                e.y,
            )),
            None => issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "tap-priority",
                format!(
                    "Tap splitter at ({},{}) has no output_priority set — a \
                     priority tap must set output_priority toward the feed branch, \
                     else the exported splitter splits 50/50 in game while the \
                     validator models it as priority-fed",
                    e.x, e.y,
                ),
                e.x,
                e.y,
            )),
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// check_belt_dead_ends
// ---------------------------------------------------------------------------

/// Check for surface belts whose output tile is empty and interior to the layout.
///
/// Belts flowing off the layout edge are OK (external I/O points).
pub fn check_belt_dead_ends(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Tiles that can receive belt output: belts, splitters, inserter pickups.
    let mut receiver_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            receiver_tiles.insert((e.x, e.y));
            if is_splitter(&e.name) {
                let dir = e.direction;
                if dir == EntityDirection::North || dir == EntityDirection::South {
                    receiver_tiles.insert((e.x + 1, e.y));
                } else {
                    receiver_tiles.insert((e.x, e.y + 1));
                }
            }
        } else if is_inserter(&e.name) {
            let d = dir_to_vec(e.direction);
            let reach = inserter_reach(&e.name);
            receiver_tiles.insert((e.x - d.0 * reach, e.y - d.1 * reach));
        }
    }

    let mut belt_at: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    for e in &layout.entities {
        if is_surface_belt(&e.name) {
            belt_at.insert((e.x, e.y), e);
        }
    }

    let mut pickup_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if is_inserter(&e.name) {
            let d = dir_to_vec(e.direction);
            let reach = inserter_reach(&e.name);
            pickup_tiles.insert((e.x - d.0 * reach, e.y - d.1 * reach));
        }
    }

    // Tiles where items are deposited by something *other than* a same-direction
    // belt chain — i.e. an actual flow source: inserter drops, splitter outputs,
    // underground-belt outputs. Used to distinguish a real dead-end (items pile
    // up because there's a producer upstream but no receiver downstream) from
    // an orphan segment (no source AND no receiver — dead concrete).
    let mut supplier_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if is_inserter(&e.name) {
            let d = dir_to_vec(e.direction);
            let reach = inserter_reach(&e.name);
            supplier_tiles.insert((e.x + d.0 * reach, e.y + d.1 * reach));
        } else if is_splitter(&e.name) {
            let d = dir_to_vec(e.direction);
            let (sx2, sy2) = splitter_second_tile(e);
            supplier_tiles.insert((e.x + d.0, e.y + d.1));
            supplier_tiles.insert((sx2 + d.0, sy2 + d.1));
        } else if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("output") {
            let d = dir_to_vec(e.direction);
            supplier_tiles.insert((e.x + d.0, e.y + d.1));
        }
    }

    let w = layout.width;
    let h = layout.height;

    for e in &layout.entities {
        if !is_surface_belt(&e.name) {
            continue;
        }
        let d = dir_to_vec(e.direction);
        let out_x = e.x + d.0;
        let out_y = e.y + d.1;
        if out_x < 0 || out_x >= w || out_y < 0 || out_y >= h {
            continue;
        }
        if receiver_tiles.contains(&(out_x, out_y)) {
            continue;
        }
        if chain_has_pickup((e.x, e.y), e.direction, &belt_at, &pickup_tiles) {
            continue;
        }
        let has_source = chain_has_source((e.x, e.y), e.direction, &belt_at, &supplier_tiles);
        if has_source {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "belt-dead-end",
                format!(
                    "Belt at ({},{}) facing {:?} has no receiver at output tile ({},{}) \
                     — items accumulate with nowhere to go",
                    e.x,
                    e.y,
                    e.direction,
                    out_x,
                    out_y
                ),
                e.x,
                e.y,
            ));
        } else {
            issues.push(ValidationIssue::with_pos(
                Severity::Warning,
                "orphan-belt-segment",
                format!(
                    "Belt at ({},{}) facing {:?} is part of an orphan segment — \
                     no producer upstream and no receiver downstream",
                    e.x, e.y, e.direction,
                ),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

/// Walk upstream from `tail_tile` along same-direction belts; return true if
/// any tile in the chain has a non-chain flow source — an inserter drop, a
/// splitter output, an underground-belt output, or a perpendicular belt that
/// sideloads onto the chain. Returns false when the chain terminates with no
/// such source (orphan segment).
fn chain_has_source(
    tail_tile: (i32, i32),
    direction: EntityDirection,
    belt_at: &FxHashMap<(i32, i32), &PlacedEntity>,
    supplier_tiles: &FxHashSet<(i32, i32)>,
) -> bool {
    let d = dir_to_vec(direction);
    let mut cur = tail_tile;
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    for _ in 0..200 {
        if !visited.insert(cur) {
            break;
        }
        if supplier_tiles.contains(&cur) {
            return true;
        }
        for (ndx, ndy) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
            let n = (cur.0 + ndx, cur.1 + ndy);
            if let Some(nb) = belt_at.get(&n) {
                let nd = dir_to_vec(nb.direction);
                if (n.0 + nd.0, n.1 + nd.1) == cur && nb.direction != direction {
                    return true;
                }
            }
        }
        let upstream = (cur.0 - d.0, cur.1 - d.1);
        match belt_at.get(&upstream) {
            Some(b) if b.direction == direction => cur = upstream,
            _ => break,
        }
    }
    false
}

/// Walk upstream from `tail_tile` along same-direction belts; return true if
/// any tile is an inserter pickup (items from that inserter drain the chain).
fn chain_has_pickup(
    tail_tile: (i32, i32),
    direction: EntityDirection,
    belt_at: &FxHashMap<(i32, i32), &PlacedEntity>,
    pickup_tiles: &FxHashSet<(i32, i32)>,
) -> bool {
    let d = dir_to_vec(direction);
    let mut cur = tail_tile;
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    for _ in 0..200 {
        if !visited.insert(cur) {
            break;
        }
        if pickup_tiles.contains(&cur) {
            return true;
        }
        let upstream = (cur.0 - d.0, cur.1 - d.1);
        match belt_at.get(&upstream) {
            Some(b) if b.direction == direction => cur = upstream,
            _ => break,
        }
    }
    false
}

// ---------------------------------------------------------------------------
// check_belt_item_isolation
// ---------------------------------------------------------------------------

/// Check that belts carrying different items don't feed into each other.
///
/// Belt pairs where either tile sits inside a `RegionKind::Unresolved`
/// region are skipped — those are orphan ghost belts left behind by a
/// `JunctionGrowthCapped` failure, and the unresolved-junction check
/// already reports the underlying cause.
pub fn check_belt_item_isolation(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut belt_dir: FxHashMap<(i32, i32), EntityDirection> = FxHashMap::default();
    let mut belt_carry: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    let mut ug_inputs: FxHashSet<(i32, i32)> = FxHashSet::default();
    // Sushi (mixed-item) belt tiles (RFP Fulgora Phase 3). A sushi↔sushi
    // adjacency legitimately carries multiple items and is exempt here; the
    // sushi boundary + saturation checks own it. This is a purely additive
    // skip — ordinary (non-sushi) adjacencies are unaffected (KC5).
    let mut sushi_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();

    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_dir.insert((e.x, e.y), e.direction);
            belt_carry.insert((e.x, e.y), e.carries.clone());
            let sushi = super::sushi::is_sushi_segment(e.segment_id.as_deref());
            if sushi {
                sushi_tiles.insert((e.x, e.y));
            }
            if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input") {
                ug_inputs.insert((e.x, e.y));
            }
            if is_splitter(&e.name) {
                let second = splitter_second_tile(e);
                belt_dir.insert(second, e.direction);
                belt_carry.insert(second, e.carries.clone());
                if sushi {
                    sushi_tiles.insert(second);
                }
            }
        }
    }

    let unresolved = super::unresolved_region_tiles(layout);
    let mut issues = Vec::new();
    let mut seen: FxHashSet<((i32, i32), (i32, i32))> = FxHashSet::default();

    for (&(ax, ay), &ad) in &belt_dir {
        // UG belt entries carry items underground, not to the adjacent surface tile.
        if ug_inputs.contains(&(ax, ay)) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ad);
        let b = (ax + dx, ay + dy);
        if !belt_dir.contains_key(&b) {
            continue;
        }
        if unresolved.contains(&(ax, ay)) || unresolved.contains(&b) {
            continue;
        }
        // Sushi↔sushi adjacency is exempt (both carry the mixed set);
        // sushi→ordinary is caught by `check_sushi_boundary`, not here.
        if sushi_tiles.contains(&(ax, ay)) && sushi_tiles.contains(&b) {
            continue;
        }
        let ac = belt_carry.get(&(ax, ay)).and_then(|c| c.as_deref());
        let bc = belt_carry.get(&b).and_then(|c| c.as_deref());
        if let (Some(ac), Some(bc)) = (ac, bc) {
            if ac != bc && seen.insert(((ax, ay), b)) {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "belt-item-isolation",
                    format!(
                        "Belt at ({},{}) carries {} but feeds into ({},{}) which carries {}",
                        ax, ay, ac, b.0, b.1, bc
                    ),
                    ax,
                    ay,
                ));
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// check_belt_inserter_conflict
// ---------------------------------------------------------------------------

/// Check that multiple inserters don't drop different items onto the same belt tile.
pub fn check_belt_inserter_conflict(layout: &LayoutResult) -> Vec<ValidationIssue> {
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
        let dv = dir_to_vec(e.direction);
        let reach = inserter_reach(&e.name);
        let drop = (e.x + dv.0 * reach, e.y + dv.1 * reach);
        if belt_tiles.contains(&drop) {
            drop_map.entry(drop).or_default().push(carries);
        }
    }

    let mut issues = Vec::new();
    for ((bx, by), items) in &drop_map {
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
                *bx,
                *by,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// check_belt_throughput
// ---------------------------------------------------------------------------

/// Check that belt tiles don't have multiple overlapping route entities.
pub fn check_belt_throughput(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut tile_counts: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    let mut tile_names: FxHashMap<(i32, i32), &str> = FxHashMap::default();

    for e in &layout.entities {
        if !is_belt_entity(&e.name) {
            continue;
        }
        *tile_counts.entry((e.x, e.y)).or_insert(0) += 1;
        tile_names.insert((e.x, e.y), &e.name);
    }

    let mut issues = Vec::new();
    for ((x, y), &count) in &tile_counts {
        if count > 1 {
            let belt_name = tile_names[&(*x, *y)];
            let max_throughput = belt_throughput(belt_name);
            issues.push(ValidationIssue::with_pos(
                Severity::Warning,
                "belt-throughput",
                format!(
                    "Belt at ({},{}): {} overlapping routes on {} (max {}/s)",
                    x, y, count, belt_name, max_throughput
                ),
                *x,
                *y,
            ));
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// check_output_belt_coverage
// ---------------------------------------------------------------------------

/// Check that every machine with solid outputs has an output inserter with a belt.
pub fn check_output_belt_coverage(
    layout: &LayoutResult,
    solver_result: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let mut fluid_output_recipes: FxHashSet<String> = FxHashSet::default();
    if let Some(sr) = solver_result {
        for spec in &sr.machines {
            if !spec.outputs.iter().any(|f| !f.is_fluid) {
                fluid_output_recipes.insert(spec.recipe.clone());
            }
        }
    }

    let machine_tiles = build_machine_tile_set(&layout.entities);
    let belt_tiles = build_belt_tile_set(&layout.entities);

    let mut issues = Vec::new();
    let mut checked: FxHashSet<(i32, i32)> = FxHashSet::default();

    for e in &layout.entities {
        if !is_machine_entity(&e.name) || !checked.insert((e.x, e.y)) {
            continue;
        }
        if let Some(recipe) = &e.recipe {
            if fluid_output_recipes.contains(recipe) {
                continue;
            }
        }

        let (mw, mh) = machine_dims(&e.name);
        let (mw, mh) = (mw as i32, mh as i32);
        let my_tiles: FxHashSet<(i32, i32)> =
            (0..mw).flat_map(|dx| (0..mh).map(move |dy| (e.x + dx, e.y + dy))).collect();

        // Recyclers (RFP Fulgora Phase 0/2, `docs/rfp-fulgora-scrap.md`)
        // eject directly onto a belt tile, mining-drill-style — no
        // output inserter, and none is wanted (Phase 0 physicals
        // finding). `recycler_eject_tile` returns `None` for the
        // unsupported E/W directions, correctly falling through to "no
        // coverage" for those (nothing in this codebase places a
        // recycler facing E/W today).
        if let Some((ex, ey)) = crate::common::recycler_eject_tile(&e.name, e.x, e.y, e.direction) {
            let has_eject_belt = belt_tiles.contains(&(ex, ey));
            if !has_eject_belt {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "output-belt",
                    format!(
                        "{} at ({},{}): no belt at the direct-ejection tile ({},{})",
                        e.name, e.x, e.y, ex, ey
                    ),
                    e.x,
                    e.y,
                ));
            }
            continue;
        }

        let has_output_belt = layout.entities.iter().any(|ins| {
            if !is_inserter(&ins.name) {
                return false;
            }
            let dv = dir_to_vec(ins.direction);
            let reach = inserter_reach(&ins.name);
            let pickup = (ins.x - dv.0 * reach, ins.y - dv.1 * reach);
            let drop = (ins.x + dv.0 * reach, ins.y + dv.1 * reach);
            my_tiles.contains(&pickup) && !machine_tiles.contains(&drop) && belt_tiles.contains(&drop)
        });

        if !has_output_belt {
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
// compute_lane_rates + check_lane_throughput
// ---------------------------------------------------------------------------

/// Classify feeder type for a belt tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeedType {
    Straight,
    SideloadLeft,
    SideloadRight,
}

/// For each belt tile, classify all upstream feeders and their feed type.
fn classify_belt_feeders(
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    ug_output_tiles: &FxHashSet<(i32, i32)>,
) -> FxHashMap<(i32, i32), Vec<((i32, i32), FeedType)>> {
    let mut feeders: FxHashMap<(i32, i32), Vec<((i32, i32), FeedType)>> = FxHashMap::default();

    for (&(bx, by), &belt_d) in belt_dir_map {
        // UG outputs receive rates from the tunnel; their in-degree is handled separately.
        if ug_output_tiles.contains(&(bx, by)) {
            continue;
        }
        let (bdx, bdy) = dir_to_vec(belt_d);
        let (left_dx, left_dy) = (-bdy, bdx);

        let mut tile_feeders: Vec<((i32, i32), FeedType)> = Vec::new();
        for (ddx, ddy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (bx + ddx, by + ddy);
            let nd = match belt_dir_map.get(&(nx, ny)) {
                Some(&d) => d,
                None => continue,
            };
            let (ndx, ndy) = dir_to_vec(nd);
            if (nx + ndx, ny + ndy) != (bx, by) {
                continue;
            }
            if nd == belt_d {
                tile_feeders.push(((nx, ny), FeedType::Straight));
            } else {
                let dot = (nx - bx) * left_dx + (ny - by) * left_dy;
                if dot > 0 {
                    tile_feeders.push(((nx, ny), FeedType::SideloadLeft));
                } else {
                    tile_feeders.push(((nx, ny), FeedType::SideloadRight));
                }
            }
        }
        if !tile_feeders.is_empty() {
            feeders.insert((bx, by), tile_feeders);
        }
    }

    feeders
}

/// Per-lane rates: `{ (x,y): (left_rate, right_rate) }`.
pub type LaneRates = FxHashMap<(i32, i32), (f64, f64)>;

/// Compute the pair of per-lane output rates `(pos_out, sib_out)` for a
/// splitter's two output tiles, given the accumulated per-lane input rates
/// at each tile (`pos_rates`, `sib_rates`).
///
/// Default behavior mirrors a plain splitter: the combined left-lane total
/// is split 50/50 across both output tiles, and likewise for the right
/// lane (both tiles end up with the same `(left, right)` tuple).
///
/// When `loop_priority_rate` is `Some(cap)` and exactly one of
/// `pos_is_loop_branch` / `sib_is_loop_branch` is `true` (identifying
/// which output tile feeds a tagged priority-branch segment — a self-loop
/// recirculation or a merge-and-tap consumer tap), this instead models a
/// priority-output splitter: the priority branch receives
/// `min(total, cap)` and the other branch receives the remainder,
/// preserving the input left/right ratio within each branch. Falls back
/// to the symmetric split if `loop_priority_rate` is `None`, or if the
/// priority branch can't be determined unambiguously (neither or both
/// flagged) — see docs/rfp-solver-net-flow.md Phase 2(c) and
/// docs/rfp-merge-tap-trunks.md D4.
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

/// Compute per-lane throughput for every surface belt / UG-output tile via
/// topological propagation of output-inserter injection rates.
pub fn compute_lane_rates(layout: &LayoutResult, solver_result: &SolverResult) -> LaneRates {
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
            match e.direction {
                EntityDirection::North | EntityDirection::South => {
                    belt_dir_map.insert((e.x + 1, e.y), e.direction);
                }
                _ => {
                    belt_dir_map.insert((e.x, e.y + 1), e.direction);
                }
            }
        }
    }

    if belt_dir_map.is_empty() {
        return FxHashMap::default();
    }

    let ug_pairs = build_ug_pairs(&layout.entities);
    for (&pos, &other) in &ug_pairs {
        if ug_input_dir.contains_key(&pos) {
            ug_output_to_input.insert(other, pos);
        }
    }

    let machine_tiles = build_machine_tile_set(&layout.entities);
    let mut machine_by_tile: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    let mut machine_entity_map: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            machine_entity_map.insert((e.x, e.y), e);
            let (w, h) = machine_dims(&e.name);
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    machine_by_tile.insert((e.x + dx, e.y + dy), (e.x, e.y));
                }
            }
        }
    }

    let recipe_to_spec: FxHashMap<&str, &_> =
        solver_result.machines.iter().map(|s| (s.recipe.as_str(), s)).collect();

    let mut belt_carries: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    for e in &layout.entities {
        if is_belt_entity(&e.name) {
            belt_carries.insert((e.x, e.y), e.carries.clone());
            if is_splitter(&e.name) {
                belt_carries.insert(splitter_second_tile(e), e.carries.clone());
            }
        }
    }

    // Seed lane injection rates from output inserters.
    let mut lane_injections: FxHashMap<(i32, i32), (f64, f64)> = FxHashMap::default();
    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let dv = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dv.0 * reach, ins.y + dv.1 * reach);
        let pickup_pos = (ins.x - dv.0 * reach, ins.y - dv.1 * reach);

        if !machine_tiles.contains(&pickup_pos) || !belt_dir_map.contains_key(&drop_pos) {
            continue;
        }
        let mpos = match machine_by_tile.get(&pickup_pos) {
            Some(&p) => p,
            None => continue,
        };
        let me = match machine_entity_map.get(&mpos) {
            Some(&e) => e,
            None => continue,
        };
        let recipe = match &me.recipe {
            Some(r) => r.as_str(),
            None => continue,
        };
        let fallback_spec = match recipe_to_spec.get(recipe) {
            Some(&s) => s,
            None => continue,
        };
        // Position-resolved via `effective_rows` — see
        // `super::resolve_row_spec`'s doc comment for the
        // partition-sibling rationale (`docs/rfp-inserter-sizing.md`
        // Phase 1 finding).
        let spec = super::resolve_row_spec(layout, recipe, me.y, fallback_spec);
        let carried_item = match belt_carries.get(&drop_pos).and_then(|c| c.as_deref()) {
            Some(i) => i,
            None => continue,
        };
        // spec.outputs[].rate is the per-machine rate at full utilization;
        // the layout places ceil(spec.count) machines each running at
        // count/ceil(count) utilization. Scale like the input-rate-delivery
        // check does, or a fast machine at fractional count overstates the
        // lane rate (a 0.06-count foundry pressing transport-belt at 16/s
        // nominal would seed 16/s onto a lane that actually carries 1/s).
        let utilization = utilization_for(spec);
        let rate = spec.outputs.iter().find(|o| o.item == carried_item).map(|o| o.rate * utilization).unwrap_or(0.0);
        if rate <= 0.0 {
            continue;
        }
        let belt_d = belt_dir_map[&drop_pos];
        let lane = inserter_target_lane(ins.x, ins.y, drop_pos.0, drop_pos.1, belt_d);
        let entry = lane_injections.entry(drop_pos).or_insert((0.0, 0.0));
        if lane == "left" { entry.0 += rate; } else { entry.1 += rate; }
    }

    let feeders = classify_belt_feeders(&belt_dir_map, &ug_output_tiles);

    let mut in_degree: FxHashMap<(i32, i32), i32> =
        belt_dir_map.keys().map(|&p| (p, 0)).collect();
    for (pos, tile_feeders) in &feeders {
        in_degree.insert(*pos, tile_feeders.len() as i32);
    }

    let mut lane_rates: FxHashMap<(i32, i32), (f64, f64)> = belt_dir_map
        .keys()
        .map(|&p| (p, lane_injections.get(&p).copied().unwrap_or((0.0, 0.0))))
        .collect();

    let mut splitter_sibling: FxHashMap<(i32, i32), (i32, i32)> = FxHashMap::default();
    // Owning splitter entity for each of its two tiles, so the priority-loop
    // model in `splitter_output_rates` can look up `loop_priority_rate`.
    let mut splitter_entity: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    for e in &layout.entities {
        if is_splitter(&e.name) {
            let second = splitter_second_tile(e);
            splitter_entity.insert((e.x, e.y), e);
            splitter_entity.insert(second, e);
            match e.direction {
                EntityDirection::North | EntityDirection::South => {
                    splitter_sibling.insert((e.x, e.y), (e.x + 1, e.y));
                    splitter_sibling.insert((e.x + 1, e.y), (e.x, e.y));
                }
                _ => {
                    splitter_sibling.insert((e.x, e.y), (e.x, e.y + 1));
                    splitter_sibling.insert((e.x, e.y + 1), (e.x, e.y));
                }
            }
        }
    }

    // Segment id per belt tile (surface belts, UG in/out, splitters), used to
    // find a self-loop-tagged tile immediately downstream of a splitter
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

    let mut queue: VecDeque<(i32, i32)> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&p, _)| p)
        .collect();

    let mut processed: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut splitter_input_ready: FxHashSet<(i32, i32)> = FxHashSet::default();
    // UG output tiles wait for their paired input's feeder; guard against an
    // unresolvable upstream (a cycle) by giving up after a few retries. Splitter
    // tiles instead PARK on an unready sibling and are force-resolved by the
    // outer loop below, so they need no retry budget of their own — a fixed
    // budget starved long tap chains (each tapoff depends on the previous
    // splitter's continuation, so an N-deep chain needs O(N) passes).
    let mut ug_retries: FxHashMap<(i32, i32), u32> = FxHashMap::default();
    const MAX_RETRIES: u32 = 3;

    // Drain the topological queue. When it empties, any splitter tile still
    // parked on a sibling that never became ready sits in an upstream cycle;
    // force-resolve the lowest-coordinate one (deterministic so cyclic results
    // are reproducible) with its current rates and re-drain. This outer loop is
    // the only remaining give-up path, and it is reached only for true cycles.
    loop {
        let pos = match queue.pop_front() {
            Some(p) => p,
            None => {
                match splitter_input_ready
                    .iter()
                    .filter(|p| !processed.contains(p))
                    .min()
                    .copied()
                {
                    Some(stuck) => {
                        processed.insert(stuck);
                        do_propagate(stuck, &belt_dir_map, &mut lane_rates, &mut in_degree, &mut queue, &feeders);
                        continue;
                    }
                    None => break,
                }
            }
        };
        if processed.contains(&pos) {
            continue;
        }

        // UG output: inherit rates from the belt that feeds the paired UG input.
        if ug_output_tiles.contains(&pos) {
            if let Some(&paired_input) = ug_output_to_input.get(&pos) {
                if let Some(&inp_d) = ug_input_dir.get(&paired_input) {
                    let (idx, idy) = dir_to_vec(inp_d);
                    let behind = (paired_input.0 - idx, paired_input.1 - idy);
                    if belt_dir_map.contains_key(&behind) && !processed.contains(&behind) {
                        let retry = ug_retries.entry(pos).or_insert(0);
                        if *retry < MAX_RETRIES {
                            *retry += 1;
                            queue.push_back(pos);
                            continue;
                        }
                        // Gave up — fall through with rate 0 for the UG tunnel.
                    }
                    if let Some(&behind_rates) = lane_rates.get(&behind) {
                        let entry = lane_rates.entry(pos).or_insert((0.0, 0.0));
                        entry.0 += behind_rates.0;
                        entry.1 += behind_rates.1;
                    }
                }
            }
        }

        // Splitter tiles must wait for their sibling before redistributing 50/50.
        if let Some(&sib) = splitter_sibling.get(&pos) {
            if !processed.contains(&sib) {
                splitter_input_ready.insert(pos);
                if !splitter_input_ready.contains(&sib) {
                    // Sibling's inputs aren't resolved yet. `pos` was just marked
                    // input-ready above, so when the sibling is popped it fires
                    // both tiles through `splitter_output_rates`. Park here rather
                    // than re-enqueue: in a DAG the sibling always resolves, and
                    // parking preserves dependency order for chains of any depth.
                    // A genuinely unresolvable sibling (cycle) is handled by the
                    // outer loop's force-resolve after the main drain.
                    continue;
                } else {
                    let pos_rates = lane_rates.get(&pos).copied().unwrap_or((0.0, 0.0));
                    let sib_rates = lane_rates.get(&sib).copied().unwrap_or((0.0, 0.0));
                    let loop_priority_rate =
                        splitter_entity.get(&pos).and_then(|e| e.loop_priority_rate);
                    let dir = belt_dir_map[&pos];
                    let (dx, dy) = dir_to_vec(dir);
                    // Priority branch = downstream tagged `:selfloop:` or a
                    // merge-and-tap priority tap. Same rate law for both.
                    let pos_is_priority_branch = super::segment_is_priority_branch(
                        belt_segment.get(&(pos.0 + dx, pos.1 + dy)).copied().flatten(),
                    );
                    let sib_is_priority_branch = super::segment_is_priority_branch(
                        belt_segment.get(&(sib.0 + dx, sib.1 + dy)).copied().flatten(),
                    );
                    let (pos_out, sib_out) = splitter_output_rates(
                        pos_rates,
                        sib_rates,
                        loop_priority_rate,
                        pos_is_priority_branch,
                        sib_is_priority_branch,
                    );
                    lane_rates.insert(pos, pos_out);
                    lane_rates.insert(sib, sib_out);
                    for &tile in &[sib, pos] {
                        processed.insert(tile);
                        do_propagate(tile, &belt_dir_map, &mut lane_rates, &mut in_degree, &mut queue, &feeders);
                    }
                    continue;
                }
            }
        }

        processed.insert(pos);
        do_propagate(pos, &belt_dir_map, &mut lane_rates, &mut in_degree, &mut queue, &feeders);
    }

    lane_rates
}

/// Propagate one tile's lane rates to its downstream neighbour and decrement in-degree.
fn do_propagate(
    tile: (i32, i32),
    belt_dir_map: &FxHashMap<(i32, i32), EntityDirection>,
    lane_rates: &mut FxHashMap<(i32, i32), (f64, f64)>,
    in_degree: &mut FxHashMap<(i32, i32), i32>,
    queue: &mut VecDeque<(i32, i32)>,
    feeders: &FxHashMap<(i32, i32), Vec<((i32, i32), FeedType)>>,
) {
    let d = match belt_dir_map.get(&tile) {
        Some(&d) => d,
        None => return,
    };
    let (ddx, ddy) = dir_to_vec(d);
    let downstream = (tile.0 + ddx, tile.1 + ddy);

    let downstream_d = match belt_dir_map.get(&downstream) {
        Some(&d) => d,
        None => return,
    };
    let (downstream_dx, downstream_dy) = dir_to_vec(downstream_d);
    let (left_dx, left_dy) = (-downstream_dy, downstream_dx);

    let my_rates = match lane_rates.get(&tile).copied() {
        Some(r) => r,
        None => return,
    };

    let ds_entry = lane_rates.entry(downstream).or_insert((0.0, 0.0));

    if d == downstream_d || (ddx, ddy) == (downstream_dx, downstream_dy) {
        ds_entry.0 += my_rates.0;
        ds_entry.1 += my_rates.1;
    } else {
        let behind_downstream = (downstream.0 - downstream_dx, downstream.1 - downstream_dy);
        if tile == behind_downstream {
            ds_entry.0 += my_rates.0;
            ds_entry.1 += my_rates.1;
        } else {
            let has_straight = feeders
                .get(&downstream)
                .map(|v| v.iter().any(|(_, ft)| *ft == FeedType::Straight))
                .unwrap_or(false);

            if has_straight {
                // Sideload: all items land on the near lane.
                let dot = (tile.0 - downstream.0) * left_dx + (tile.1 - downstream.1) * left_dy;
                let total = my_rates.0 + my_rates.1;
                if dot > 0 { ds_entry.0 += total; } else { ds_entry.1 += total; }
            } else {
                // 90-degree turn: cross > 0 → clockwise (left↔right swap).
                let cross = ddx * downstream_dy - ddy * downstream_dx;
                if cross > 0 {
                    ds_entry.1 += my_rates.0;
                    ds_entry.0 += my_rates.1;
                } else {
                    ds_entry.0 += my_rates.0;
                    ds_entry.1 += my_rates.1;
                }
            }
        }
    }

    let deg = in_degree.entry(downstream).or_insert(0);
    *deg -= 1;
    if *deg <= 0 {
        queue.push_back(downstream);
    }
}

// ---------------------------------------------------------------------------
// check_lane_throughput
// ---------------------------------------------------------------------------

/// Check per-lane belt throughput using full lane simulation.
pub fn check_lane_throughput(
    layout: &LayoutResult,
    solver_result: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let sr = match solver_result {
        Some(sr) => sr,
        None => return vec![],
    };

    let lane_rates = compute_lane_rates(layout, sr);
    if lane_rates.is_empty() {
        return vec![];
    }

    let mut belt_name_map: FxHashMap<(i32, i32), &str> = FxHashMap::default();
    for e in &layout.entities {
        if is_surface_belt(&e.name) {
            belt_name_map.insert((e.x, e.y), &e.name);
        } else if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("output") {
            belt_name_map.insert((e.x, e.y), ug_to_surface_tier(&e.name));
        } else if is_splitter(&e.name) {
            let tier = splitter_to_surface_tier(&e.name);
            belt_name_map.insert((e.x, e.y), tier);
            belt_name_map.insert(splitter_second_tile(e), tier);
        }
    }

    let mut issues = Vec::new();
    for ((x, y), (left, right)) in &lane_rates {
        let belt_name = belt_name_map.get(&(*x, *y)).copied().unwrap_or("transport-belt");
        let cap = lane_capacity(belt_name);
        for (lane_name, rate) in [("left", *left), ("right", *right)] {
            if rate > cap + 0.01 {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "lane-throughput",
                    format!(
                        "Belt at ({},{}): {} lane {:.1}/s exceeds {} per-lane capacity {}/s",
                        x, y, lane_name, rate, belt_name, cap
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
    use crate::models::{ItemFlow, MachineSpec, PlacedEntity, SolverResult};

    fn make_entity(name: &str, x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            direction: dir,
            ..Default::default()
        }
    }

    fn belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        make_entity("transport-belt", x, y, dir)
    }

    fn belt_carrying(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity { carries: Some(item.to_string()), ..belt(x, y, dir) }
    }

    fn inserter(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        make_entity("inserter", x, y, dir)
    }

    fn inserter_carrying(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity { carries: Some(item.to_string()), ..inserter(x, y, dir) }
    }

    fn machine(name: &str, x: i32, y: i32, recipe: &str) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            recipe: Some(recipe.to_string()),
            ..Default::default()
        }
    }

    fn layout(entities: Vec<PlacedEntity>) -> LayoutResult {
        LayoutResult { entities, width: 50, height: 50, ..Default::default() }
    }

    // -----------------------------------------------------------------------
    // check_belt_loops
    // -----------------------------------------------------------------------

    #[test]
    fn belt_loop_no_loop_passes() {
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::East),
            belt(2, 0, EntityDirection::East),
        ]);
        assert!(check_belt_loops(&lr).is_empty());
    }

    #[test]
    fn belt_loop_square_loop_detected() {
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::South),
            belt(1, 1, EntityDirection::West),
            belt(0, 1, EntityDirection::North),
        ]);
        let issues = check_belt_loops(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "belt-loop");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn belt_loop_reported_once() {
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::South),
            belt(1, 1, EntityDirection::West),
            belt(0, 1, EntityDirection::North),
        ]);
        let issues = check_belt_loops(&lr);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn belt_loop_chain_leading_into_loop() {
        let lr = layout(vec![
            belt(-2, 0, EntityDirection::East),
            belt(-1, 0, EntityDirection::East),
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::South),
            belt(1, 1, EntityDirection::West),
            belt(0, 1, EntityDirection::North),
        ]);
        let issues = check_belt_loops(&lr);
        assert_eq!(issues.len(), 1);
    }

    // -----------------------------------------------------------------------
    // check_belt_dead_ends
    // -----------------------------------------------------------------------

    #[test]
    fn dead_end_belt_flowing_off_edge_ok() {
        // Belt at x=49 facing East → output tile (50,0) is off-edge (width=50).
        let lr = layout(vec![
            belt(48, 0, EntityDirection::East),
            belt(49, 0, EntityDirection::East),
        ]);
        assert!(check_belt_dead_ends(&lr).is_empty());
    }

    #[test]
    fn dead_end_belt_into_another_belt_ok() {
        // Belt at (48,0) → (49,0) which has a belt → (50,0) off-edge.
        let lr = layout(vec![
            belt(48, 0, EntityDirection::East),
            belt(49, 0, EntityDirection::East),
        ]);
        assert!(check_belt_dead_ends(&lr).is_empty());
    }

    #[test]
    fn dead_end_isolated_belt_is_orphan_warning() {
        // No producer upstream, no receiver downstream — dead concrete, not
        // a flow that's piling up. Reclassified to Warning orphan-belt-segment.
        let lr = layout(vec![belt(5, 5, EntityDirection::East)]);
        let issues = check_belt_dead_ends(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "orphan-belt-segment");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn dead_end_with_inserter_source_is_real_error() {
        // Inserter drops onto belt, belt has no receiver — items genuinely
        // pile up. Stays an Error.
        let inserter_drop = inserter(7, 5, EntityDirection::West); // drops at (6,5)
        let lr = layout(vec![belt(6, 5, EntityDirection::East), inserter_drop]);
        let issues = check_belt_dead_ends(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "belt-dead-end");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn dead_end_belt_into_inserter_pickup_ok() {
        // inserter at (2,0) facing EAST → pickup at (1,0); belt at (0,0) East → (1,0).
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            inserter(2, 0, EntityDirection::East),
        ]);
        assert!(check_belt_dead_ends(&lr).is_empty());
    }

    #[test]
    fn dead_end_belt_into_ug_input_ok() {
        let ug_input = PlacedEntity {
            name: "underground-belt".to_string(),
            x: 1,
            y: 0,
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            ..Default::default()
        };
        let lr = layout(vec![belt(0, 0, EntityDirection::East), ug_input]);
        assert!(check_belt_dead_ends(&lr).is_empty());
    }

    // -----------------------------------------------------------------------
    // check_belt_item_isolation
    // -----------------------------------------------------------------------

    #[test]
    fn item_isolation_same_item_ok() {
        let lr = layout(vec![
            belt_carrying(0, 0, EntityDirection::East, "iron-plate"),
            belt_carrying(1, 0, EntityDirection::East, "iron-plate"),
        ]);
        assert!(check_belt_item_isolation(&lr).is_empty());
    }

    #[test]
    fn item_isolation_different_items_error() {
        let lr = layout(vec![
            belt_carrying(0, 0, EntityDirection::East, "iron-plate"),
            belt_carrying(1, 0, EntityDirection::East, "copper-plate"),
        ]);
        let issues = check_belt_item_isolation(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "belt-item-isolation");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn item_isolation_ug_input_skipped() {
        let ug_input = PlacedEntity {
            name: "underground-belt".to_string(),
            x: 0,
            y: 0,
            direction: EntityDirection::East,
            io_type: Some("input".to_string()),
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        };
        let lr = layout(vec![ug_input, belt_carrying(1, 0, EntityDirection::East, "copper-plate")]);
        assert!(check_belt_item_isolation(&lr).is_empty());
    }

    #[test]
    fn item_isolation_no_carry_no_error() {
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt_carrying(1, 0, EntityDirection::East, "copper-plate"),
        ]);
        assert!(check_belt_item_isolation(&lr).is_empty());
    }

    // -----------------------------------------------------------------------
    // check_belt_inserter_conflict
    // -----------------------------------------------------------------------

    #[test]
    fn inserter_conflict_same_item_ok() {
        let lr = layout(vec![
            belt(0, 5, EntityDirection::East),
            inserter_carrying(0, 4, EntityDirection::South, "iron-plate"),
            inserter_carrying(0, 6, EntityDirection::North, "iron-plate"),
        ]);
        assert!(check_belt_inserter_conflict(&lr).is_empty());
    }

    #[test]
    fn inserter_conflict_different_items_error() {
        let lr = layout(vec![
            belt(0, 5, EntityDirection::East),
            inserter_carrying(0, 4, EntityDirection::South, "iron-plate"),
            inserter_carrying(0, 6, EntityDirection::North, "copper-plate"),
        ]);
        let issues = check_belt_inserter_conflict(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "belt-item-isolation");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn inserter_conflict_no_carry_ignored() {
        let lr = layout(vec![
            belt(0, 5, EntityDirection::East),
            inserter(0, 4, EntityDirection::South),
            inserter_carrying(0, 6, EntityDirection::North, "iron-plate"),
        ]);
        assert!(check_belt_inserter_conflict(&lr).is_empty());
    }

    // -----------------------------------------------------------------------
    // check_belt_throughput
    // -----------------------------------------------------------------------

    #[test]
    fn throughput_no_overlap_ok() {
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::East),
        ]);
        assert!(check_belt_throughput(&lr).is_empty());
    }

    #[test]
    fn throughput_overlapping_entities_warns() {
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(0, 0, EntityDirection::South),
        ]);
        let issues = check_belt_throughput(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].category, "belt-throughput");
        assert!(issues[0].message.contains("2 overlapping"));
    }

    // -----------------------------------------------------------------------
    // check_output_belt_coverage
    // -----------------------------------------------------------------------

    #[test]
    fn output_coverage_machine_with_output_belt_ok() {
        // Machine at (0,0) 3×3; inserter at (1,3) facing South picks from (1,2) inside
        // machine and drops onto belt at (1,4).
        let lr = layout(vec![
            machine("assembling-machine-1", 0, 0, "iron-gear-wheel"),
            inserter(1, 3, EntityDirection::South),
            belt(1, 4, EntityDirection::East),
        ]);
        assert!(check_output_belt_coverage(&lr, None).is_empty());
    }

    #[test]
    fn output_coverage_machine_without_output_belt_error() {
        let lr = layout(vec![machine("assembling-machine-1", 0, 0, "iron-gear-wheel")]);
        let issues = check_output_belt_coverage(&lr, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "output-belt");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn output_coverage_fluid_output_recipe_skipped() {
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "fluid-recipe".to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![],
                outputs: vec![ItemFlow { item: "water".to_string(), rate: 10.0, is_fluid: true, module_id: 0 }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let lr = layout(vec![machine("assembling-machine-3", 0, 0, "fluid-recipe")]);
        assert!(check_output_belt_coverage(&lr, Some(&sr)).is_empty());
    }

    // -----------------------------------------------------------------------
    // check_lane_throughput
    // -----------------------------------------------------------------------

    #[test]
    fn lane_throughput_no_solver_result_returns_empty() {
        let lr = layout(vec![belt(0, 0, EntityDirection::East)]);
        assert!(check_lane_throughput(&lr, None).is_empty());
    }

    #[test]
    fn lane_throughput_single_inserter_within_capacity() {
        // One inserter at 2.5/s on yellow belt (7.5/s lane cap) — should pass.
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-gear-wheel".to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![ItemFlow { item: "iron-plate".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
                outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 2.5,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let entities = vec![
            machine("assembling-machine-3", 3, 0, "iron-gear-wheel"),
            // inserter at (4,3) South: pickup at (4,2) inside machine, drop at (4,4)
            inserter(4, 3, EntityDirection::South),
            belt_carrying(4, 4, EntityDirection::East, "iron-gear-wheel"),
            belt_carrying(5, 4, EntityDirection::East, "iron-gear-wheel"),
        ];
        let lr = layout(entities);
        assert!(check_lane_throughput(&lr, Some(&sr)).is_empty());
    }

    #[test]
    fn lane_throughput_same_side_inserters_overload() {
        // Two machines each dropping 5.0/s on the same lane → 10.0/s > 7.5/s cap.
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-gear-wheel".to_string(),
                self_loop: vec![], voider: false,
                count: 2.0,
                inputs: vec![ItemFlow { item: "iron-plate".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
                outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".to_string(),
                    rate: 5.0,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let mut entities = vec![
            machine("assembling-machine-3", 0, 0, "iron-gear-wheel"),
            machine("assembling-machine-3", 7, 0, "iron-gear-wheel"),
            inserter(1, 3, EntityDirection::South),
            inserter(8, 3, EntityDirection::South),
        ];
        for x in 1..=9 {
            entities.push(belt_carrying(x, 4, EntityDirection::East, "iron-gear-wheel"));
        }
        let lr = layout(entities);
        let issues = check_lane_throughput(&lr, Some(&sr));
        assert!(issues.iter().any(|i| i.category.contains("lane")));
    }

    #[test]
    fn lane_throughput_fractional_count_scales_to_utilization() {
        // Regression: logistic-science-pack@1/s picked a foundry for
        // transport-belt (category "pressing") — 16/s per machine nominal,
        // but solver count 0.06 → actual sustained flow ~1/s. Seeding the
        // nominal per-machine rate flagged the whole trunk over the lane
        // cap. The injection must scale by count/ceil(count) utilization,
        // matching the input-rate-delivery check's convention.
        //
        // Same geometry both times: count 1.0 (full utilization, 16/s)
        // must flag — proving the setup actually seeds — and count 0.06
        // (0.96/s effective) must not.
        let run = |count: f64| {
            let sr = SolverResult {
                machines: vec![MachineSpec {
                    entity: "assembling-machine-3".to_string(),
                    recipe: "transport-belt".to_string(),
                    self_loop: vec![], voider: false,
                    count,
                    inputs: vec![],
                    outputs: vec![ItemFlow {
                        item: "transport-belt".to_string(),
                        rate: 16.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                }],
                external_inputs: vec![],
                external_outputs: vec![],
                surplus_outputs: vec![],
                dependency_order: vec![],
            };
            let entities = vec![
                machine("assembling-machine-3", 3, 0, "transport-belt"),
                // inserter at (4,3) South: pickup (4,2) inside machine, drop (4,4)
                inserter(4, 3, EntityDirection::South),
                // yellow belt: 7.5/s lane cap
                belt_carrying(4, 4, EntityDirection::East, "transport-belt"),
                belt_carrying(5, 4, EntityDirection::East, "transport-belt"),
            ];
            check_lane_throughput(&layout(entities), Some(&sr))
        };

        let full = run(1.0);
        assert!(
            full.iter().any(|i| i.category == "lane-throughput"),
            "count=1.0 at 16/s on yellow must flag (geometry sanity check), got: {:?}",
            full.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
        let fractional = run(0.06);
        assert!(
            fractional.is_empty(),
            "count=0.06 must seed utilization-scaled 0.96/s, got: {:?}",
            fractional.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn lane_rates_priority_splitter_loop_branch_gets_priority_share() {
        // Priority-output splitter (self-loop row, e.g. kovarex): the loop
        // branch should receive min(total, loop_priority_rate) and the
        // export branch the remainder, NOT a symmetric 50/50 split.
        //
        // Machine feeds 4.1/s onto the splitter's "pos" tile (10,0) via a
        // direct inserter injection; the splitter's sibling tile (10,1)
        // gets nothing. Downstream of (10,0) is tagged as the self-loop
        // segment; downstream of (10,1) is a plain export belt.
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "kovarex-enrichment-process".to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![],
                outputs: vec![ItemFlow {
                    item: "uranium-235".to_string(),
                    rate: 4.1,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let splitter_entity = PlacedEntity {
            loop_priority_rate: Some(4.0),
            carries: Some("uranium-235".to_string()),
            ..make_entity("splitter", 10, 0, EntityDirection::East)
        };
        let entities = vec![
            machine("assembling-machine-3", 8, -4, "kovarex-enrichment-process"),
            // inserter at (10,-1) South: pickup (10,-2) inside machine, drop
            // (10,0) — the splitter's "pos" tile.
            inserter(10, -1, EntityDirection::South),
            splitter_entity,
            // pos (10,0) -> downstream (11,0): tagged self-loop segment.
            PlacedEntity {
                segment_id: Some(
                    "row:kovarex-enrichment-process:selfloop:uranium-235".to_string(),
                ),
                ..belt_carrying(11, 0, EntityDirection::East, "uranium-235")
            },
            // sib (10,1) -> downstream (11,1): plain export belt, no tag.
            belt_carrying(11, 1, EntityDirection::East, "uranium-235"),
        ];
        let lr = layout(entities);
        let rates = compute_lane_rates(&lr, &sr);

        let pos_total = rates.get(&(10, 0)).map(|&(l, r)| l + r).unwrap_or(0.0);
        let sib_total = rates.get(&(10, 1)).map(|&(l, r)| l + r).unwrap_or(0.0);
        assert!(
            (pos_total - 4.0).abs() < 0.01,
            "loop branch should get ~4.0/s, got {pos_total}"
        );
        assert!(
            (sib_total - 0.1).abs() < 0.01,
            "export branch should get ~0.1/s (not 2.05/2.05 symmetric), got {sib_total}"
        );
    }

    #[test]
    fn belt_loop_selfloop_tagged_segment_exempted() {
        // Same 4-tile square cycle as `belt_loop_square_loop_detected`, but
        // one tile carries a self-loop row segment tag (priority-splitter
        // recirculation is intentional, not a bug) — no error expected.
        let lr = layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::South),
            PlacedEntity {
                segment_id: Some(
                    "row:kovarex-enrichment-process:selfloop:uranium-235".to_string(),
                ),
                ..belt(1, 1, EntityDirection::West)
            },
            belt(0, 1, EntityDirection::North),
        ]);
        assert!(check_belt_loops(&lr).is_empty());
    }

    // -----------------------------------------------------------------------
    // check_tap_splitter_priority (RFP merge-tap-trunks D4)
    // -----------------------------------------------------------------------

    /// Build a tagged merge-tap feed belt one tile downstream of `(x,y)`.
    fn tap_belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            segment_id: Some(format!("family:copper-cable{MERGE_TAP_SEGMENT_TAG}0")),
            ..belt(x, y, dir)
        }
    }

    #[test]
    fn priority_output_tile_matches_lane_orientation() {
        // docs/factorio-mechanics.md B3: left/right of a splitter relative to
        // travel direction. Splitter at (3,3); main=(3,3), and second is
        // (3,4) for E/W, (4,3) for N/S.
        let at = |dir, op: &str| {
            priority_output_tile(&PlacedEntity {
                output_priority: Some(op.to_string()),
                ..make_entity("splitter", 3, 3, dir)
            })
        };
        use EntityDirection::*;
        // East: left = north (main), right = south (second).
        assert_eq!(at(East, "left"), Some((3, 3)));
        assert_eq!(at(East, "right"), Some((3, 4)));
        // West: left = south (second), right = north (main).
        assert_eq!(at(West, "left"), Some((3, 4)));
        assert_eq!(at(West, "right"), Some((3, 3)));
        // North: left = west (main), right = east (second).
        assert_eq!(at(North, "left"), Some((3, 3)));
        assert_eq!(at(North, "right"), Some((4, 3)));
        // South: left = east (second), right = west (main).
        assert_eq!(at(South, "left"), Some((4, 3)));
        assert_eq!(at(South, "right"), Some((3, 3)));
        // No priority set.
        assert_eq!(
            priority_output_tile(&make_entity("splitter", 3, 3, East)),
            None
        );
    }

    #[test]
    fn tap_splitter_correct_priority_passes() {
        // East tap splitter: south output (second tile) feeds the row, north
        // output continues the trunk. Priority "right" = south = feed branch.
        let entities = vec![
            PlacedEntity {
                output_priority: Some("right".to_string()),
                ..make_entity("splitter", 10, 0, EntityDirection::East)
            },
            tap_belt(11, 1, EntityDirection::East), // south output → feed
            belt(11, 0, EntityDirection::East),     // north output → trunk
        ];
        let lr = layout(entities);
        assert!(
            check_tap_splitter_priority(&lr).is_empty(),
            "correct tap should pass: {:?}",
            check_tap_splitter_priority(&lr)
        );
    }

    #[test]
    fn tap_splitter_south_facing_correct_priority_passes() {
        // Standard bus tap-off (mechanics S9): South splitter, east tile's
        // output feeds the row, west tile continues south. For South facing,
        // "left" is the east tile — priority must be "left".
        let entities = vec![
            PlacedEntity {
                output_priority: Some("left".to_string()),
                ..make_entity("splitter", 5, 5, EntityDirection::South)
            },
            tap_belt(6, 6, EntityDirection::South), // east output → feed
            belt(5, 6, EntityDirection::South),     // west output → trunk
        ];
        let lr = layout(entities);
        assert!(
            check_tap_splitter_priority(&lr).is_empty(),
            "south tap with priority at east(left) feed should pass: {:?}",
            check_tap_splitter_priority(&lr)
        );
    }

    #[test]
    fn tap_splitter_backwards_priority_errors() {
        // Same East tap, but priority "left" points at the north (trunk)
        // continuation, not the south feed — the backwards-stamped tap.
        let entities = vec![
            PlacedEntity {
                output_priority: Some("left".to_string()),
                ..make_entity("splitter", 10, 0, EntityDirection::East)
            },
            tap_belt(11, 1, EntityDirection::East),
            belt(11, 0, EntityDirection::East),
        ];
        let lr = layout(entities);
        let issues = check_tap_splitter_priority(&lr);
        assert_eq!(issues.len(), 1, "backwards tap must error: {issues:?}");
        assert_eq!(issues[0].category, "tap-priority");
        assert!(issues[0].message.contains("continuation"));
    }

    #[test]
    fn tap_splitter_missing_priority_errors() {
        // Tap splitter with no output_priority set at all: the exported
        // splitter splits 50/50 while the walker models it as priority-fed.
        let entities = vec![
            make_entity("splitter", 10, 0, EntityDirection::East),
            tap_belt(11, 1, EntityDirection::East),
            belt(11, 0, EntityDirection::East),
        ];
        let lr = layout(entities);
        let issues = check_tap_splitter_priority(&lr);
        assert_eq!(issues.len(), 1, "missing priority must error: {issues:?}");
        assert_eq!(issues[0].category, "tap-priority");
        assert!(issues[0].message.contains("no output_priority"));
    }

    #[test]
    fn tap_splitter_both_branches_tapped_errors() {
        // Both outputs tagged as feed branches — ambiguous; the walker cannot
        // assign a single priority output (containment discipline).
        let entities = vec![
            PlacedEntity {
                output_priority: Some("right".to_string()),
                ..make_entity("splitter", 10, 0, EntityDirection::East)
            },
            tap_belt(11, 0, EntityDirection::East),
            tap_belt(11, 1, EntityDirection::East),
        ];
        let lr = layout(entities);
        let issues = check_tap_splitter_priority(&lr);
        assert_eq!(issues.len(), 1, "double-tap must error: {issues:?}");
        assert!(issues[0].message.contains("both outputs"));
    }

    #[test]
    fn selfloop_splitter_not_flagged_as_tap() {
        // A :selfloop: priority splitter is not a :mergetap: tap — the tap
        // check must leave it entirely alone (containment discipline).
        let entities = vec![
            PlacedEntity {
                output_priority: Some("right".to_string()),
                loop_priority_rate: Some(4.0),
                ..make_entity("splitter", 10, 0, EntityDirection::East)
            },
            PlacedEntity {
                segment_id: Some(
                    "row:kovarex-enrichment-process:selfloop:uranium-235".to_string(),
                ),
                ..belt(11, 1, EntityDirection::East)
            },
            belt(11, 0, EntityDirection::East),
        ];
        let lr = layout(entities);
        assert!(check_tap_splitter_priority(&lr).is_empty());
    }

    #[test]
    fn generic_ghost_tap_not_flagged_as_priority_tap() {
        // Regression / inertness guard (KC2): the ghost router's generic tap
        // segments (`ghost:tap:*`) contain the substring `:tap:` but are plain
        // 50/50 splitters — they must NOT be mistaken for merge-and-tap
        // priority taps, which is why the marker is the distinct `:mergetap:`.
        let entities = vec![
            make_entity("splitter", 10, 0, EntityDirection::East),
            PlacedEntity {
                segment_id: Some("ghost:tap:iron-ore:3:143".to_string()),
                ..belt(11, 1, EntityDirection::East)
            },
            belt(11, 0, EntityDirection::East),
        ];
        let lr = layout(entities);
        assert!(
            check_tap_splitter_priority(&lr).is_empty(),
            "ghost:tap:* is a generic tap, not a priority tap"
        );
    }

    #[test]
    fn plain_splitter_not_flagged_as_tap() {
        // Untagged splitter: the generic even-split model owns it.
        let entities = vec![
            make_entity("splitter", 10, 0, EntityDirection::East),
            belt(11, 0, EntityDirection::East),
            belt(11, 1, EntityDirection::East),
        ];
        let lr = layout(entities);
        assert!(check_tap_splitter_priority(&lr).is_empty());
    }

    #[test]
    fn lane_rates_tap_branch_gets_priority_share() {
        // Merge-and-tap priority tap: the feed branch (downstream tagged
        // MERGE_TAP_SEGMENT_TAG) receives min(total, loop_priority_rate) and
        // the trunk continuation the remainder — the same rate law as
        // :selfloop:, now lit up by the tap tag. Mirrors
        // `lane_rates_priority_splitter_loop_branch_gets_priority_share`.
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "kovarex-enrichment-process".to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![],
                outputs: vec![ItemFlow {
                    item: "uranium-235".to_string(),
                    rate: 4.1,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let splitter_entity = PlacedEntity {
            loop_priority_rate: Some(4.0),
            carries: Some("uranium-235".to_string()),
            ..make_entity("splitter", 10, 0, EntityDirection::East)
        };
        let entities = vec![
            machine("assembling-machine-3", 8, -4, "kovarex-enrichment-process"),
            inserter(10, -1, EntityDirection::South),
            splitter_entity,
            // pos (10,0) → downstream (11,0): tagged merge-tap feed branch.
            PlacedEntity {
                segment_id: Some(format!("family:uranium-235{MERGE_TAP_SEGMENT_TAG}0")),
                ..belt_carrying(11, 0, EntityDirection::East, "uranium-235")
            },
            // sib (10,1) → downstream (11,1): trunk continuation, no tag.
            belt_carrying(11, 1, EntityDirection::East, "uranium-235"),
        ];
        let lr = layout(entities);
        let rates = compute_lane_rates(&lr, &sr);
        let feed_total = rates.get(&(10, 0)).map(|&(l, r)| l + r).unwrap_or(0.0);
        let cont_total = rates.get(&(10, 1)).map(|&(l, r)| l + r).unwrap_or(0.0);
        assert!(
            (feed_total - 4.0).abs() < 0.01,
            "feed branch should get ~4.0/s, got {feed_total}"
        );
        assert!(
            (cont_total - 0.1).abs() < 0.01,
            "continuation should get ~0.1/s (not symmetric), got {cont_total}"
        );
    }
}

// ---------------------------------------------------------------------------
// check_entity_overlaps
// ---------------------------------------------------------------------------

/// Collect all tiles occupied by an entity. Splitters occupy 2 tiles;
/// machines occupy an N×N footprint; everything else occupies 1 tile.
fn entity_tiles(e: &PlacedEntity) -> Vec<(i32, i32)> {
    if is_splitter(&e.name) {
        return vec![(e.x, e.y), splitter_second_tile(e)];
    }
    if is_machine_entity(&e.name) {
        let (w, h) = machine_dims(&e.name);
        return machine_tiles(e.x, e.y, w, h);
    }
    vec![(e.x, e.y)]
}

/// Check for entities that physically overlap on the same tile.
///
/// Builds a tile → entity-index occupancy map, then reports any tile claimed
/// by two or more entities.
pub fn check_entity_overlaps(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut occupancy: FxHashMap<(i32, i32), Vec<usize>> = FxHashMap::default();
    for (idx, e) in layout.entities.iter().enumerate() {
        for tile in entity_tiles(e) {
            occupancy.entry(tile).or_default().push(idx);
        }
    }

    let mut overlap_tiles: Vec<(i32, i32)> = occupancy
        .iter()
        .filter(|(_, idxs)| idxs.len() >= 2)
        .map(|(&tile, _)| tile)
        .collect();
    overlap_tiles.sort_unstable();

    overlap_tiles
        .into_iter()
        .map(|(tx, ty)| {
            let idxs = &occupancy[&(tx, ty)];
            let names: Vec<&str> =
                idxs.iter().map(|&i| layout.entities[i].name.as_str()).collect();
            ValidationIssue::with_pos(
                Severity::Error,
                "entity-overlap",
                format!("Entities overlap at ({tx},{ty}): {}", names.join(", ")),
                tx,
                ty,
            )
        })
        .collect()
}

#[cfg(test)]
mod overlap_tests {
    use super::*;
    use crate::models::{EntityDirection, LayoutResult, PlacedEntity};

    fn make_layout(entities: Vec<PlacedEntity>) -> LayoutResult {
        LayoutResult { entities, width: 20, height: 20, ..Default::default() }
    }

    fn belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".to_string(),
            x,
            y,
            direction: dir,
            ..Default::default()
        }
    }

    fn splitter_ent(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "splitter".to_string(),
            x,
            y,
            direction: dir,
            ..Default::default()
        }
    }

    fn machine_ent(name: &str, x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            direction: EntityDirection::North,
            ..Default::default()
        }
    }

    #[test]
    fn no_overlap_returns_empty() {
        let lr = make_layout(vec![
            belt(0, 0, EntityDirection::East),
            belt(1, 0, EntityDirection::East),
        ]);
        assert!(check_entity_overlaps(&lr).is_empty());
    }

    #[test]
    fn two_belts_same_tile_reports_error() {
        let lr = make_layout(vec![
            belt(3, 3, EntityDirection::East),
            belt(3, 3, EntityDirection::North),
        ]);
        let issues = check_entity_overlaps(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].category, "entity-overlap");
        assert_eq!(issues[0].x, Some(3));
        assert_eq!(issues[0].y, Some(3));
    }

    #[test]
    fn splitter_companion_tile_detected() {
        // Splitter at (0,0) facing North/South occupies (0,0) and (1,0).
        // A belt at (1,0) overlaps the companion tile.
        let lr = make_layout(vec![
            splitter_ent(0, 0, EntityDirection::North),
            belt(1, 0, EntityDirection::East),
        ]);
        let issues = check_entity_overlaps(&lr);
        assert_eq!(issues.len(), 1, "Expected 1 overlap at (1,0); got: {issues:?}");
        assert_eq!(issues[0].x, Some(1));
        assert_eq!(issues[0].y, Some(0));
    }

    #[test]
    fn two_splitters_same_anchor_overlap() {
        // Two splitters at the same anchor — both (0,0) and (1,0) are doubly claimed.
        let lr = make_layout(vec![
            splitter_ent(0, 0, EntityDirection::North),
            splitter_ent(0, 0, EntityDirection::North),
        ]);
        let issues = check_entity_overlaps(&lr);
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn machine_footprint_overlap_with_belt() {
        // assembling-machine-1 at (0,0) occupies tiles (0..2, 0..2).
        // A belt at (2,2) sits on the last tile of the 3×3 footprint.
        let lr = make_layout(vec![
            machine_ent("assembling-machine-1", 0, 0),
            belt(2, 2, EntityDirection::East),
        ]);
        let issues = check_entity_overlaps(&lr);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].x, Some(2));
        assert_eq!(issues[0].y, Some(2));
    }

    #[test]
    fn machine_no_overlap_when_adjacent() {
        // Machine at (0,0) occupies (0..2, 0..2). Belt at (3,0) is just outside.
        let lr = make_layout(vec![
            machine_ent("assembling-machine-1", 0, 0),
            belt(3, 0, EntityDirection::East),
        ]);
        assert!(check_entity_overlaps(&lr).is_empty());
    }

    #[test]
    fn empty_layout_passes() {
        let lr = make_layout(vec![]);
        assert!(check_entity_overlaps(&lr).is_empty());
    }
}
