//! Pole-to-pole copper wire graph — THE single source of truth for the
//! electric-network wiring a blueprint must encode.
//!
//! Factorio 2.0 stores copper connections between electric poles in a
//! blueprint-level `wires` array. Each wire is
//! `[entity_number_a, connector_id, entity_number_b, connector_id]`; for
//! pole-to-pole copper the connector id is [`POLE_COPPER`] = 5 (draftsman
//! `WireConnectorID.POLE_COPPER`; the old 1.x per-entity `neighbours` array is
//! not read by 2.0). WITHOUT this array, pasted poles are electrically
//! disconnected islands and the whole factory is power-dead — the bug this
//! module exists to prevent.
//!
//! [`compute_pole_wires`] is consumed by three call sites that MUST agree:
//! - `blueprint::export` — emits the `wires` array (entity_number = index+1).
//! - `validate::power::check_pole_network_connectivity` — asserts the emitted
//!   graph is one connected component (checks the artifact, not mere geometry).
//! - `bus::layout` / `blueprint_parser` — populate `LayoutResult::power_wires`
//!   for the web power-connectivity overlay.
//!
//! ## Density: connect all in-reach pairs (no cap)
//!
//! Factorio ≤1.x capped a pole at 5 copper connections; **2.0 removed that
//! cap** (FFF-379 "Abstract rewiring"), and draftsman's prototype data carries
//! no `maximum_wire_connections` field for any pole. So there is effectively no
//! count cap for copper, and we connect **every** pair of poles within wire
//! reach. This makes the emitted graph exactly the geometric adjacency graph
//! that `bus::layout::place_poles` + `repair_pole_connectivity` already
//! guarantee is connected — so a correctly-placed pole field always yields a
//! single connected wire network, and the validator can never be lied to.
//!
//! ## Reach: min-of-both, Euclidean between footprint centers
//!
//! Two poles wire iff the Euclidean distance between their footprint centers is
//! ≤ the *smaller* of the two poles' wire reaches (Factorio's min-of-both rule).
//! Reaches are draftsman `maximum_wire_distance`: medium 9, small 7.5,
//! substation 18, big 32. Footprint centers come from the shared
//! [`crate::common::entity_size`] (RFC power-arc Phase 3a-i), the same source
//! `blueprint::export` uses to place each entity — so the wire graph's reach
//! test uses the exact centers the blueprint encodes (a 2×2 substation at
//! `+1.0`, a 1×1 medium pole at `+0.5`).

use crate::common::entity_size;
use crate::models::PlacedEntity;

/// Blueprint `wires` connector id for pole-to-pole copper (draftsman
/// `WireConnectorID.POLE_COPPER`).
pub const POLE_COPPER: u32 = 5;

/// Copper wire reach (tile distance between pole CENTERS) for a pole entity
/// at a build-quality tier; `None` for anything that is not an electric pole.
/// Delegates to [`crate::common::pole_wire_reach`] — the ONE wire-reach table
/// this module, the validator, and `bus::layout::repair_pole_connectivity`
/// all read (base values are draftsman `maximum_wire_distance`; quality adds
/// +2 per level, rfc-build-quality). Do NOT confuse it with
/// [`crate::common::supply_area_distance`] (power COVERAGE radius: medium 3.5,
/// substation 9), which is a different quantity.
pub fn wire_reach(name: &str, quality: crate::common::QualityTier) -> Option<f64> {
    crate::common::pole_wire_reach(name, quality)
}

/// Whether `name` is an electric pole (a copper-wire node). Pole-ness is
/// quality-independent.
pub fn is_pole(name: &str) -> bool {
    wire_reach(name, crate::common::QualityTier::Normal).is_some()
}

/// Footprint center in continuous tile-space for a pole at top-left `(x, y)`,
/// from the shared [`entity_size`]: a substation is 2×2 (center `+1.0`); medium
/// and small poles are 1×1 (center `+0.5`). This is byte-identical to
/// `blueprint::export`'s position calc, so the reach test runs on the centers
/// the blueprint actually encodes.
pub fn pole_center(name: &str, x: i32, y: i32) -> (f64, f64) {
    let (w, h) = entity_size(name);
    (x as f64 + w as f64 / 2.0, y as f64 + h as f64 / 2.0)
}

/// Compute the pole-to-pole copper wire graph over `entities`.
///
/// Returns a deterministic, ascending list of `(a, b)` index pairs (`a < b`)
/// into `entities`, one per copper wire, where both endpoints are electric
/// poles within min-of-both wire reach. Connects every in-reach pair (no count
/// cap — see the module docs). Iteration is in entity order, so the result is
/// stable across runs given a stable entity list.
pub fn compute_pole_wires(entities: &[PlacedEntity]) -> Vec<(u32, u32)> {
    // (entity_index, center_x, center_y, wire_reach) for every pole, in order.
    let poles: Vec<(u32, f64, f64, f64)> = entities
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            // Per-entity quality (rfc-build-quality): a legendary medium
            // pole wires at 19, so quality layouts' sparser pole fields
            // still emit a fully-connected artifact.
            wire_reach(&e.name, e.quality.unwrap_or_default()).map(|r| {
                let (cx, cy) = pole_center(&e.name, e.x, e.y);
                (i as u32, cx, cy, r)
            })
        })
        .collect();

    let mut wires: Vec<(u32, u32)> = Vec::new();
    for a in 0..poles.len() {
        let (ia, ax, ay, ar) = poles[a];
        for &(ib, bx, by, br) in &poles[a + 1..] {
            let dx = ax - bx;
            let dy = ay - by;
            let reach = ar.min(br);
            // Squared compare avoids sqrt; `<=` makes exact-reach connect.
            if dx * dx + dy * dy <= reach * reach {
                wires.push((ia, ib));
            }
        }
    }
    wires
}

/// Count how many pole entities are NOT reachable from the first pole via the
/// emitted copper `wires`. `0` means the whole pole set is one connected
/// electric network. `wires` are index pairs into `entities` (as produced by
/// [`compute_pole_wires`] or parsed from a blueprint). Non-pole entities and
/// out-of-range endpoints are ignored defensively.
pub fn count_disconnected_poles(entities: &[PlacedEntity], wires: &[(u32, u32)]) -> usize {
    let pole_idxs: Vec<usize> = entities
        .iter()
        .enumerate()
        .filter(|(_, e)| is_pole(&e.name))
        .map(|(i, _)| i)
        .collect();
    if pole_idxs.len() <= 1 {
        return 0;
    }

    // Union-find over pole entity indices.
    let n = entities.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    for &(a, b) in wires {
        let (a, b) = (a as usize, b as usize);
        if a < n && b < n && is_pole(&entities[a].name) && is_pole(&entities[b].name) {
            let ra = find(&mut parent, a);
            let rb = find(&mut parent, b);
            if ra != rb {
                parent[ra] = rb;
            }
        }
    }

    let root0 = find(&mut parent, pole_idxs[0]);
    pole_idxs
        .iter()
        .filter(|&&i| find(&mut parent, i) != root0)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PlacedEntity;

    fn pole(name: &str, x: i32, y: i32) -> PlacedEntity {
        PlacedEntity { name: name.to_string(), x, y, ..Default::default() }
    }

    #[test]
    fn line_of_three_medium_poles_wires_in_reach_pairs() {
        // Centers 0.5, 7.5, 14.5 (spacing 7 ≤ 9). Adjacent pairs within reach:
        // (0,1) d=7, (1,2) d=7; (0,2) d=14 > 9 → no direct edge.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            pole("medium-electric-pole", 7, 0),
            pole("medium-electric-pole", 14, 0),
        ];
        let wires = compute_pole_wires(&ents);
        assert_eq!(wires, vec![(0, 1), (1, 2)]);
        // Transitively connected → nothing disconnected.
        assert_eq!(count_disconnected_poles(&ents, &wires), 0);
    }

    #[test]
    fn dense_cluster_connects_all_in_reach_pairs_not_just_a_tree() {
        // Three medium poles inside mutual reach → a triangle (3 edges), not a
        // 2-edge spanning tree. Confirms "all in-reach pairs", no cap.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            pole("medium-electric-pole", 5, 0),
            pole("medium-electric-pole", 5, 5),
        ];
        let wires = compute_pole_wires(&ents);
        assert_eq!(wires, vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn out_of_reach_poles_are_disconnected() {
        // Centers 0.5 and 10.5 → d=10 > 9. No wire.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            pole("medium-electric-pole", 10, 0),
        ];
        let wires = compute_pole_wires(&ents);
        assert!(wires.is_empty());
        assert_eq!(count_disconnected_poles(&ents, &wires), 1);
    }

    #[test]
    fn mixed_pole_types_use_min_of_both_reach() {
        // medium (9) + small (7.5) at centers 0.5 and 8.5 → d=8 > min(9,7.5)=7.5.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            pole("small-electric-pole", 8, 0),
        ];
        assert!(compute_pole_wires(&ents).is_empty());
    }

    #[test]
    fn substation_uses_2x2_center_and_18_reach() {
        // Substation top-left (0,0) → center (1,1) via entity_size 2×2. A medium
        // at (16,0) → center (16.5,0.5): dx=15.5,dy=0.5 → d²≈240.5; min(18,9)=9
        // → 81 → out of reach. A medium at (9,0) → center (9.5,0.5):
        // dx=8.5,dy=0.5 → d²≈72.5 ≤ 81 → wired.
        let far = vec![
            pole("substation", 0, 0),
            pole("medium-electric-pole", 16, 0),
        ];
        assert!(compute_pole_wires(&far).is_empty());
        let near = vec![
            pole("substation", 0, 0),
            pole("medium-electric-pole", 9, 0),
        ];
        assert_eq!(compute_pole_wires(&near), vec![(0, 1)]);
    }

    #[test]
    fn indices_are_absolute_entity_indices_skipping_non_poles() {
        // A belt sits between two poles; the wire must reference indices 0 and 2.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            PlacedEntity { name: "transport-belt".into(), x: 3, y: 0, ..Default::default() },
            pole("medium-electric-pole", 6, 0),
        ];
        assert_eq!(compute_pole_wires(&ents), vec![(0, 2)]);
    }

    #[test]
    fn count_disconnected_ignores_out_of_range_and_non_pole_endpoints() {
        // Defensive: a wire referencing a belt index or an out-of-bounds index
        // must not connect or panic; the second pole stays disconnected.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            PlacedEntity { name: "transport-belt".into(), x: 1, y: 0, ..Default::default() },
            pole("medium-electric-pole", 50, 0),
        ];
        let bogus = vec![(0u32, 1u32), (0u32, 99u32)];
        assert_eq!(count_disconnected_poles(&ents, &bogus), 1);
    }
}
