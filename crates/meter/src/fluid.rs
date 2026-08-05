//! Fluid pipe networks (RFC-054 Phase B).
//!
//! Phase A delivered fluid by **port-adjacency**: a consumer pulled from a
//! directly-adjacent producer or a boundary standing source. That throttled
//! petroleum→plastic→AC→PU to ~20% because in a real blueprint the producer
//! and consumer are separated by pipes (possibly underground), never
//! adjacent — so the petroleum drained as "delivered" instead of reaching
//! the plastic chemical-plant.
//!
//! Phase B replaces that with a **pipe network**: connected components of
//! pipe tiles (including `pipe-to-ground` underground pairs) plus the
//! machine fluid ports and boundary feeds that touch them. Within one
//! component, fluid of a given type flows freely from boundary sources and
//! producer outputs to consumer inputs — pipe-fast, and balanced across
//! multi-output recipes (every byproduct ends up in the component's pool and
//! is drawn by whoever consumes it).
//!
//! ## Topology rules honoured (from `docs/factorio-mechanics.md`)
//!
//! - **F4**: a pipe-to-ground pairs underground with the nearest opposite-
//!   facing pipe-to-ground on the same axis, entity-to-entity distance ≤ 10.
//! - **F5**: the blueprint-JSON `direction` is the **surface-opening** side;
//!   the underground side is opposite.
//! - **F5a**: a pipe-to-ground's two perpendicular sides have no surface
//!   connection (keeps stacked fluid trunk rows isolated).
//!
//! ## Flow model
//!
//! Each tick the `Factory` routes fluid per component, per item: the
//! component's boundary feeds (infinite standing sources, matching the
//! saturated input rig) and producer `fluid_output` are pooled and drawn by
//! the component's consumers up to their per-craft need. Anything a consumer
//! does not take drains as delivered — the same behaviour as Phase A, so
//! target-fluids that exit a layout (e.g. `tier3_advanced_oil_processing`)
//! keep measuring as the sim measures them.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::blueprint_in::Dir;

/// Entity-to-entity max for a pipe-to-ground pair (F4): gap ≤ 9.
pub const PTG_MAX_UNDERGROUND: i32 = 10;

/// A machine fluid port bound to a specific item, in absolute tile coords.
#[derive(Debug, Clone, Copy)]
pub struct MachPort {
    pub machine: usize,
    pub x: i32,
    pub y: i32,
    pub item: u16,
    pub is_input: bool,
}

/// One connected component of the fluid topology.
#[derive(Debug, Default)]
pub struct FluidNetwork {
    pub id: usize,
    /// Machine ports bound to this component.
    pub ports: Vec<MachPort>,
    /// Fluid ids available here as infinite boundary standing sources.
    pub boundary: FxHashSet<u16>,
}

/// The built fluid topology.
#[derive(Debug, Default)]
pub struct FluidSystem {
    pub networks: Vec<FluidNetwork>,
    /// Declared fluid boundary inputs whose tile touched no pipe (would
    /// otherwise be silently dropped).
    pub unconnected_feeds: Vec<(u16, (i32, i32))>,
}

/// A tile-level "node" of the fluid graph before network assignment.
struct Node {
    /// is this a pipe-to-ground (versus a plain pipe / pump)?
    ptg: bool,
    /// blueprint-JSON direction of the surface opening (only for ptg).
    dir: Option<Dir>,
}

/// Build the fluid pipe networks from pipe entities plus machine ports and
/// boundary fluid feeds.
///
/// `ports` and `feeds` must already carry their absolute tile positions.
pub fn build_networks(
    pipe_entities: &[(i32, i32, &str, Dir)],
    ports: &[MachPort],
    feeds: &[(u16, (i32, i32))],
) -> FluidSystem {
    // --- collect nodes: pipe tiles + port tiles + feed tiles -------------
    let mut pipe: FxHashMap<(i32, i32), Node> = FxHashMap::default();
    for &(x, y, name, dir) in pipe_entities {
        pipe.insert(
            (x, y),
            Node {
                ptg: name == "pipe-to-ground",
                dir: (name == "pipe-to-ground").then_some(dir),
            },
        );
    }
    let mut all: FxHashSet<(i32, i32)> = pipe.keys().copied().collect();
    for p in ports {
        all.insert((p.x, p.y));
    }
    for &(_, t) in feeds {
        all.insert(t);
    }

    // --- union-find --------------------------------------------------------
    let mut parent: FxHashMap<(i32, i32), (i32, i32)> = all.iter().map(|&t| (t, t)).collect();
    let mut rank: FxHashMap<(i32, i32), u8> = all.iter().map(|&t| (t, 0)).collect();
    fn find(parent: &mut FxHashMap<(i32, i32), (i32, i32)>, t: (i32, i32)) -> (i32, i32) {
        let p = parent[&t];
        if p == t {
            return t;
        }
        let r = find(parent, p);
        parent.insert(t, r);
        r
    }
    fn union(
        parent: &mut FxHashMap<(i32, i32), (i32, i32)>,
        rank: &mut FxHashMap<(i32, i32), u8>,
        a: (i32, i32),
        b: (i32, i32),
    ) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra == rb {
            return;
        }
        let (ra, rb) = match rank[&ra].cmp(&rank[&rb]) {
            std::cmp::Ordering::Less => (rb, ra),
            _ => (ra, rb),
        };
        parent.insert(rb, ra);
        if rank[&ra] == rank[&rb] {
            rank.insert(ra, rank[&ra] + 1);
        }
    }

    let delta = |d: Dir| d.delta();
    let is_pipe = |t: (i32, i32)| pipe.contains_key(&t);

    // Surface connections between adjacent pipes (F5/F5a).
    //
    // A regular pipe connects to all four orthogonal pipe neighbours — EXCEPT
    // that a neighbouring pipe-to-ground only joins on the side that is the
    // PTG's surface mouth (a PTG's perpendicular sides are closed, F5a; this
    // is what keeps stacked / crossing trunk lines isolated).
    let surface_delta = |p: (i32, i32)| -> (i32, i32) {
        let d = pipe[&p].dir.unwrap();
        (p.0 + d.delta().0, p.1 + d.delta().1)
    };
    for &(x, y) in pipe.keys() {
        let n = &pipe[&(x, y)];
        if !n.ptg {
            for d in [Dir::North, Dir::East, Dir::South, Dir::West] {
                let nb = (x + d.delta().0, y + d.delta().1);
                if !is_pipe(nb) {
                    continue;
                }
                if pipe[&nb].ptg {
                    // join only if this PTG's surface mouth faces us
                    if surface_delta(nb) == (x, y) {
                        union(&mut parent, &mut rank, (x, y), nb);
                    }
                } else {
                    union(&mut parent, &mut rank, (x, y), nb);
                }
            }
            continue;
        }
        // pipe-to-ground: connect only on its surface side (F5/F5a);
        // underground pairing handled by F4 below.
        let dir = n.dir.unwrap();
        let surf = (x + dir.delta().0, y + dir.delta().1);
        match pipe.get(&surf) {
            // A regular pipe opens on every side, so it joins the mouth.
            Some(p) if !p.ptg => union(&mut parent, &mut rank, (x, y), surf),
            // F5b: a PTG whose surface mouth faces back toward this PTG merges
            // across their shared edge.
            Some(p) if p.ptg && p.dir == Some(Dir::opp_dir(dir)) => {
                union(&mut parent, &mut rank, (x, y), surf)
            }
            // A PTG that does NOT open back (e.g. a same-facing stacked PTG)
            // does not connect to this mouth (F5a): the lines stay isolated.
            _ => {}
        }
    }

    // F4: underground pairing — a ptg connects to the nearest opposite-facing
    // ptg along its underground axis within PTG_MAX_UNDERGROUND.
    for &(x, y) in pipe.keys() {
        let n = &pipe[&(x, y)];
        if !n.ptg {
            continue;
        }
        let dir = n.dir.unwrap();
        // Underground direction is opposite the surface opening (F5).
        let under = delta(Dir::opp_dir(dir));
        for k in 1..=PTG_MAX_UNDERGROUND {
            let t = (x + under.0 * k, y + under.1 * k);
            match pipe.get(&t) {
                Some(other)
                    if other.ptg && other.dir.map(|d| d == Dir::opp_dir(dir)).unwrap_or(false) =>
                {
                    union(&mut parent, &mut rank, (x, y), t);
                    break;
                }
                _ => {}
            }
        }
    }

    // Machine ports + boundary feeds connect to a pipe on their own tile or
    // any orthogonally adjacent pipe tile. A PTG neighbour only joins when its
    // surface mouth is this tile (F5a). Join EVERY eligible neighbour — a port
    // that touches two components must not be arbitrarily pegged to just one
    // (that would isolate it from the other and mis-route its fluid).
    let mut connect_to_pipe = |t: (i32, i32)| -> bool {
        let mut connected = false;
        for c in std::iter::once(t).chain(
            [Dir::North, Dir::East, Dir::South, Dir::West]
                .into_iter()
                .map(|d| (t.0 + delta(d).0, t.1 + delta(d).1)),
        ) {
            if !pipe.contains_key(&c) {
                continue;
            }
            if c != t && pipe[&c].ptg && surface_delta(c) != t {
                continue;
            }
            union(&mut parent, &mut rank, t, c);
            connected = true;
        }
        connected
    };
    for p in ports {
        connect_to_pipe((p.x, p.y));
    }
    for &(_, t) in feeds {
        connect_to_pipe(t);
    }

    // --- flood-fill into networks ------------------------------------------
    let mut net_ids: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    let mut networks: Vec<FluidNetwork> = Vec::new();
    let mut members: FxHashMap<(i32, i32), Vec<(i32, i32)>> = FxHashMap::default();
    for &t in &all {
        let r = find(&mut parent, t);
        members.entry(r).or_default().push(t);
    }
    let mut roots_sorted: Vec<(i32, i32)> = members.keys().copied().collect();
    // sort by min tile for determinism
    roots_sorted.sort_by_key(|&r| members[&r].iter().min().copied().unwrap_or((0, 0)));
    for (i, r) in roots_sorted.iter().enumerate() {
        for &t in &members[r] {
            net_ids.insert(t, i);
        }
        networks.push(FluidNetwork {
            id: i,
            ports: Vec::new(),
            boundary: FxHashSet::default(),
        });
    }

    for p in ports {
        if let Some(&nid) = net_ids.get(&(p.x, p.y)) {
            networks[nid].ports.push(*p);
        }
    }
    let mut unconnected_feeds: Vec<(u16, (i32, i32))> = Vec::new();
    for &(item, t) in feeds {
        if let Some(&nid) = net_ids.get(&t) {
            networks[nid].boundary.insert(item);
        } else {
            // A declared fluid boundary input that touches no pipe is a silent
            // data-loss: report it so the caller can surface a note.
            unconnected_feeds.push((item, t));
        }
    }
    FluidSystem {
        networks,
        unconnected_feeds,
    }
}

impl Dir {
    fn opp_dir(d: Dir) -> Dir {
        match d {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::East => Dir::West,
            Dir::West => Dir::East,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pipe chain with an underground pair must connect a producer port to a
    /// consumer port across the underground gap (the AC/petroleum case).
    #[test]
    fn underground_pipe_chain_connects_ports() {
        // producer port (0,0) -> pipes -> ptg(0,2) face N -> ptg(0,4) face S ->
        // pipe -> consumer port (0,6)
        let pipes = vec![
            (0i32, 2i32, "pipe-to-ground", Dir::North),
            (0i32, 4i32, "pipe-to-ground", Dir::South),
            (0i32, 1i32, "pipe", Dir::North),
            (0i32, 5i32, "pipe", Dir::North),
        ];
        let ports = vec![
            MachPort {
                machine: 0,
                x: 0,
                y: 0,
                item: 10,
                is_input: false,
            },
            MachPort {
                machine: 1,
                x: 0,
                y: 6,
                item: 10,
                is_input: true,
            },
        ];
        let feeds = vec![(20u16, (0i32, 0i32))];
        let sys = build_networks(&pipes, &ports, &feeds);
        // both ports share one network
        let nets: Vec<&FluidNetwork> = sys
            .networks
            .iter()
            .filter(|n| !n.ports.is_empty())
            .collect();
        assert_eq!(
            nets.len(),
            1,
            "producer and consumer should merge via the UG pair"
        );
        assert_eq!(nets[0].ports.len(), 2);
        assert!(
            nets[0].boundary.contains(&20),
            "boundary feed joins the chain"
        );
    }

    /// F5a: two pipe-to-grounds placed side by side — adjacent on their
    /// PERPENDICULAR axis (both facing North, touching East–West) — must NOT
    /// merge, even though the tiles touch. This is non-vacuous: it would fail
    /// if the builder joined orthogonally-adjacent PTGs regardless of facing.
    #[test]
    fn perpendicular_ptg_neighbours_stay_isolated() {
        // Two PTGs at (0,1) and (1,1), both facing North: their surface mouths
        // are at (0,0) and (1,0), each feeding one machine port. The tiles
        // (0,1)-(1,1) are perpendicular (East-West) to their N-S facing.
        let pipes = vec![
            (0i32, 1i32, "pipe-to-ground", Dir::North),
            (1i32, 1i32, "pipe-to-ground", Dir::North),
        ];
        let ports = vec![
            MachPort {
                machine: 0,
                x: 0,
                y: 0,
                item: 1,
                is_input: true,
            },
            MachPort {
                machine: 1,
                x: 1,
                y: 0,
                item: 2,
                is_input: true,
            },
        ];
        let sys = build_networks(&pipes, &ports, &[]);
        // each PTG+port is its own component; they must not have merged.
        let mut ids: Vec<usize> = sys
            .networks
            .iter()
            .filter(|n| !n.ports.is_empty())
            .map(|n| n.id)
            .collect();
        assert_eq!(ids.len(), 2, "saw {:?}", ids);
        ids.sort_unstable();
        assert_ne!(
            ids[0], ids[1],
            "perpendicular-adjacent PTGs must not merge (F5a)"
        );
    }

    /// F5a/F5b: a PTG whose surface mouth rests against ANOTHER PTG that does
    /// not open back must not merge — the mouth only joins a regular pipe or a
    /// back-facing PTG (F5b). Pins the stacked-trunk isolation that a plain
    /// "union whatever is on the mouth tile" would break. (No underground
    /// partner is present, so F4 pairing stays out of the way.)
    #[test]
    fn stacked_same_facing_ptgs_stay_isolated() {
        // B at (0,0) faces North (mouth (0,-1), port there). A at (0,1) also
        // faces North, so A's mouth (0,0) rests on B's back — B does not open
        // toward A. If A's mouth wrongly joined B, they would be ONE network.
        let pipes = vec![
            (0i32, 0i32, "pipe-to-ground", Dir::North),
            (0i32, 1i32, "pipe-to-ground", Dir::North),
        ];
        let ports = vec![MachPort {
            machine: 0,
            x: 0,
            y: -1,
            item: 1,
            is_input: true,
        }];
        let sys = build_networks(&pipes, &ports, &[]);
        assert_eq!(
            sys.networks.len(),
            2,
            "B's pipe and A's pipe must remain separate components"
        );
    }
}
