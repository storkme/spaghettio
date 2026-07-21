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
//! RFC-044 stored-graph contract: the graph is computed ONCE
//! (`bus::layout::layout_pass`, in `LayoutOptions::wire_mode` — dense
//! mesh or deterministic spanning tree — recorded on
//! `LayoutResult::wire_mode`) and stored in `LayoutResult::power_wires`;
//! `blueprint::export` and
//! `validate::power::check_pole_network_connectivity` both consume the
//! STORED graph through [`wires_for`] (with a dense-derive fallback for
//! layouts that never computed wires), and `blueprint_parser` preserves
//! an imported blueprint's wires verbatim. The artifact, the validated
//! graph, and the web overlay are therefore the same object by
//! construction — the pre-RFC "N sites re-derive and must agree"
//! convention is retired.
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
use crate::models::{LayoutResult, PlacedEntity};
use std::borrow::Cow;

/// User-facing pole wiring mode (RFC-044). `Dense` connects every
/// in-reach pair (Factorio 2.0 has no copper connection cap) — the
/// maximally robust artifact, and the default. `Tree` emits a
/// deterministic minimum spanning forest over the SAME candidate set:
/// the fewest wires that keep each connected component connected —
/// visually clean, but deconstructing any one pole in-game splits the
/// network. Recorded on `LayoutResult.wire_mode` so post-layout
/// recomputes (the improve-region splice) honor the layout's policy.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WireMode {
    #[default]
    Dense,
    Tree,
}

impl WireMode {
    /// Parse the URL/wasm string form; unknown → `None` (callers
    /// default to `Dense`, the `max_inserter_tier` pattern).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "dense" => Some(WireMode::Dense),
            "tree" => Some(WireMode::Tree),
            _ => None,
        }
    }
}

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
pub fn compute_pole_wires(entities: &[PlacedEntity], mode: WireMode) -> Vec<(u32, u32)> {
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
    match mode {
        WireMode::Dense => wires,
        WireMode::Tree => minimum_spanning_forest(entities, &poles, wires),
    }
}

/// Deterministic minimum spanning forest over the dense candidate set
/// (RFC-044 §2). Kruskal with edges totally ordered by the PHYSICAL key
/// `(squared_center_distance, min_endpoint_anchor, max_endpoint_anchor)`
/// — anchors are the poles' integer `(x, y)` tile positions, so the
/// selected wire SET is a unique function of the physical layout,
/// invariant under entity-Vec reordering (the project's own
/// `golden_hash` treats entity order as non-canonical, so an
/// index-keyed tiebreak would let a legitimate pipeline reorder
/// silently reshape trees; grid layouts tie on distance constantly).
/// Distinct poles cannot share an anchor tile, so the key is a total
/// order and the Kruskal result is unique, ties or not. Disconnected
/// candidate graphs yield one tree per component — `Tree` never papers
/// over a genuine disconnection.
fn minimum_spanning_forest(
    entities: &[PlacedEntity],
    poles: &[(u32, f64, f64, f64)],
    candidates: Vec<(u32, u32)>,
) -> Vec<(u32, u32)> {
    use rustc_hash::FxHashMap;
    let centers: FxHashMap<u32, (f64, f64)> =
        poles.iter().map(|&(i, cx, cy, _)| (i, (cx, cy))).collect();
    let anchor = |i: u32| {
        let e = &entities[i as usize];
        (e.x, e.y)
    };

    let mut keyed: Vec<((f64, (i32, i32), (i32, i32)), (u32, u32))> = candidates
        .into_iter()
        .map(|(a, b)| {
            let (ax, ay) = centers[&a];
            let (bx, by) = centers[&b];
            let d2 = (ax - bx) * (ax - bx) + (ay - by) * (ay - by);
            let (pa, pb) = (anchor(a), anchor(b));
            let (lo, hi) = if pa <= pb { (pa, pb) } else { (pb, pa) };
            ((d2, lo, hi), (a, b))
        })
        .collect();
    // All d2 are finite squared grid distances — partial_cmp cannot fail.
    keyed.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

    let n = entities.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }

    let mut out: Vec<(u32, u32)> = Vec::new();
    for (_, (a, b)) in keyed {
        let (ra, rb) = (find(&mut parent, a as usize), find(&mut parent, b as usize));
        if ra != rb {
            parent[ra] = rb;
            out.push((a, b));
        }
    }
    // Same normalization/ordering convention as Dense output.
    out.sort_unstable();
    out
}

/// The stored-graph accessor (RFC-044 §1): `Some` is authoritative and
/// consumed verbatim (even `Some(vec![])`); `None` means the layout
/// never computed wires (hand-built `LayoutResult`s, pre-power-3c
/// snapshots) and falls back to a dense derivation — exactly the
/// pre-RFC behavior, so nothing that worked before can silently export
/// power-dead poles. Export and the connectivity validator BOTH read
/// through this single accessor, which is what makes the emitted
/// artifact and the validated graph the same object by construction.
pub fn wires_for(layout: &LayoutResult) -> Cow<'_, [(u32, u32)]> {
    match &layout.power_wires {
        Some(w) => Cow::Borrowed(w.as_slice()),
        None => Cow::Owned(compute_pole_wires(&layout.entities, WireMode::Dense)),
    }
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

    /// Kill 3 (RFC-044): tie-heavy determinism. An equidistant pole line
    /// ties every adjacent pair; the physical-key Kruskal must pick the
    /// SAME edge set every run, pinned exactly.
    #[test]
    fn tree_mode_is_deterministic_under_ties() {
        let ents: Vec<PlacedEntity> =
            (0..5).map(|i| pole("medium-electric-pole", i * 7, 0)).collect();
        let t1 = compute_pole_wires(&ents, WireMode::Tree);
        let t2 = compute_pole_wires(&ents, WireMode::Tree);
        assert_eq!(t1, t2);
        // 5 poles, one component → exactly 4 edges; the adjacent-pair MST.
        assert_eq!(t1, vec![(0, 1), (1, 2), (2, 3), (3, 4)]);
    }

    /// Kill 3 (RFC-044, review finding 6): the selected wire SET is a
    /// function of the PHYSICAL layout, not entity-Vec order — the same
    /// poles constructed in reversed order must yield the same physical
    /// edges (entity order is non-canonical per the project's own
    /// golden-hash convention).
    #[test]
    fn tree_mode_is_invariant_under_entity_reordering() {
        let fwd: Vec<PlacedEntity> = (0..6)
            .map(|i| pole("medium-electric-pole", (i % 3) * 7, (i / 3) * 8))
            .collect();
        let rev: Vec<PlacedEntity> = fwd.iter().rev().cloned().collect();

        let physical = |ents: &[PlacedEntity], wires: &[(u32, u32)]| {
            let mut set: Vec<((i32, i32), (i32, i32))> = wires
                .iter()
                .map(|&(a, b)| {
                    let pa = (ents[a as usize].x, ents[a as usize].y);
                    let pb = (ents[b as usize].x, ents[b as usize].y);
                    if pa <= pb { (pa, pb) } else { (pb, pa) }
                })
                .collect();
            set.sort_unstable();
            set
        };
        let t_fwd = compute_pole_wires(&fwd, WireMode::Tree);
        let t_rev = compute_pole_wires(&rev, WireMode::Tree);
        assert_eq!(physical(&fwd, &t_fwd), physical(&rev, &t_rev));
    }

    /// Kill 4 (RFC-044): tree property per component — `edges ==
    /// poles − components`, every tree edge is a dense-candidate edge,
    /// and the validator's actual scalar (`count_disconnected_poles`)
    /// is identical between modes. Two clusters deliberately out of
    /// reach of each other → a spanning FOREST that never papers over
    /// the genuine disconnection.
    #[test]
    fn tree_mode_spans_each_component_and_matches_validator_scalar() {
        let mut ents: Vec<PlacedEntity> =
            (0..3).map(|i| pole("medium-electric-pole", i * 7, 0)).collect();
        // Second cluster 100 tiles away — unreachable from the first.
        ents.extend((0..4).map(|i| pole("medium-electric-pole", 100 + i * 7, 0)));

        let dense = compute_pole_wires(&ents, WireMode::Dense);
        let tree = compute_pole_wires(&ents, WireMode::Tree);

        // 7 poles, 2 components → 5 edges.
        assert_eq!(tree.len(), 5);
        for e in &tree {
            assert!(dense.contains(e), "tree edge {e:?} not in dense candidate set");
        }
        assert_eq!(
            count_disconnected_poles(&ents, &tree),
            count_disconnected_poles(&ents, &dense),
            "validator scalar must be mode-invariant"
        );
    }

    /// RFC-044 kill 6 contract (core half): a recompute in the layout's
    /// OWN recorded `wire_mode` — the exact call
    /// `improve_region_streaming` makes after its entity-reordering
    /// splice — must preserve tree-ness. Simulates the splice as a
    /// reorder + recompute; the wasm site is three lines reading
    /// `layout.wire_mode`, review-verifiable against this pin.
    #[test]
    fn recompute_in_recorded_mode_preserves_tree() {
        let ents: Vec<PlacedEntity> =
            (0..4).map(|i| pole("medium-electric-pole", i * 7, 0)).collect();
        let mut layout = LayoutResult::default();
        layout.wire_mode = WireMode::Tree;
        layout.entities = ents;
        layout.power_wires = Some(compute_pole_wires(&layout.entities, layout.wire_mode));
        assert_eq!(layout.power_wires.as_deref().map(|w| w.len()), Some(3));

        // Splice-shaped mutation: rotate entity order, then recompute the
        // way the wasm site does — in layout.wire_mode, not hardcoded Dense.
        layout.entities.rotate_left(1);
        layout.power_wires =
            Some(compute_pole_wires(&layout.entities, layout.wire_mode));
        let w = layout.power_wires.as_deref().unwrap();
        assert_eq!(w.len(), 3, "tree-ness must survive the recompute: {w:?}");
    }

    /// RFC-044 kill 1: consuming the STORED dense graph must be
    /// byte-indistinguishable from the `None`-fallback derivation at
    /// export time.
    #[test]
    fn export_stored_dense_equals_fallback_derivation() {
        let ents: Vec<PlacedEntity> =
            (0..3).map(|i| pole("medium-electric-pole", i * 7, 0)).collect();
        let mut stored = LayoutResult::default();
        stored.entities = ents.clone();
        stored.power_wires = Some(compute_pole_wires(&ents, WireMode::Dense));
        let mut fallback = LayoutResult::default();
        fallback.entities = ents;
        fallback.power_wires = None;
        assert_eq!(
            crate::blueprint::export(&stored, "t"),
            crate::blueprint::export(&fallback, "t"),
            "stored-vs-derived export strings must be byte-identical at Dense"
        );
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
        let wires = compute_pole_wires(&ents, WireMode::Dense);
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
        let wires = compute_pole_wires(&ents, WireMode::Dense);
        assert_eq!(wires, vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn out_of_reach_poles_are_disconnected() {
        // Centers 0.5 and 10.5 → d=10 > 9. No wire.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            pole("medium-electric-pole", 10, 0),
        ];
        let wires = compute_pole_wires(&ents, WireMode::Dense);
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
        assert!(compute_pole_wires(&ents, WireMode::Dense).is_empty());
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
        assert!(compute_pole_wires(&far, WireMode::Dense).is_empty());
        let near = vec![
            pole("substation", 0, 0),
            pole("medium-electric-pole", 9, 0),
        ];
        assert_eq!(compute_pole_wires(&near, WireMode::Dense), vec![(0, 1)]);
    }

    #[test]
    fn indices_are_absolute_entity_indices_skipping_non_poles() {
        // A belt sits between two poles; the wire must reference indices 0 and 2.
        let ents = vec![
            pole("medium-electric-pole", 0, 0),
            PlacedEntity { name: "transport-belt".into(), x: 3, y: 0, ..Default::default() },
            pole("medium-electric-pole", 6, 0),
        ];
        assert_eq!(compute_pole_wires(&ents, WireMode::Dense), vec![(0, 2)]);
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
