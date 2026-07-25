//! Belt topology as a tile graph.
//!
//! PR 1 simulated a *linear* run, which was enough to test the slot model
//! but cannot represent a real factory. This builds the general form: every
//! belt-like tile is a node, each holds two lanes of slots, and each links
//! to whatever it feeds.
//!
//! The slot physics are unchanged — items advance only into a free slot
//! ahead, so gaps still fail to heal and compression still happens only
//! against a blockage. What changes is *what* is downstream.
//!
//! # Connection rules (`factorio-mechanics.md`)
//!
//! A tile facing `d` outputs to the tile at `pos + d`. How the lanes map
//! depends on where it meets the target:
//!
//! - **Back feed** (**B6**/**B7**) — feeding the target's rear: both lanes
//!   transfer straight through, lane-for-lane.
//! - **Side feed** (**B8**) — feeding the target's flank: fills **only the
//!   near lane**, the one on the feeder's side.
//! - **Curve** (**B11**) — a side feed that is the target's *only* input
//!   renders as a 90° turn and preserves both lanes.
//!
//! The curve carve-out matters: treating every side feed as a sideload
//! would halve throughput around every corner, a systematic error in a
//! router that turns constantly.
//!
//! # Deliberately not modelled yet
//!
//! Splitter output priority and filters (**S6**–**S8**); belt loops get an
//! arbitrary update order rather than a principled one. Both are recorded
//! as `TopologyNote`s on the built network rather than being silent.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::belt::{ItemId, Lane};
use crate::blueprint_in::Dir;
use crate::entity_data::{BeltTier, SLOTS_PER_TILE};

/// Rotate a facing 90° counter-clockwise — the "left" side of a belt
/// (`factorio-mechanics.md` **B3**: a north-facing belt's left lane is on
/// its west side).
fn left_of(d: Dir) -> Dir {
    match d {
        Dir::North => Dir::West,
        Dir::West => Dir::South,
        Dir::South => Dir::East,
        Dir::East => Dir::North,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Belt,
    /// Underground entrance — its output emerges at the paired exit.
    UgInput,
    /// Underground exit.
    UgOutput,
    /// One half of a splitter (each splitter occupies two tiles).
    Splitter { partner: usize, id: usize },
}

/// How lanes map when items cross from one tile to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneMap {
    /// Lane-for-lane (back feed, curve, underground).
    Straight,
    /// Sideload: everything lands on one lane of the target (**B8**).
    OntoLane(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Downstream {
    pub tile: usize,
    pub lanes: LaneMap,
}

#[derive(Debug, Clone)]
pub struct BeltTile {
    pub pos: (i32, i32),
    pub dir: Dir,
    pub tier: BeltTier,
    pub kind: TileKind,
    pub lanes: [Lane; 2],
    pub downstream: Option<Downstream>,
    /// Items that left this tile with nowhere to go (layout edge).
    pub exited: u64,
    /// True only for tiles the manifest names as boundary outputs.
    ///
    /// **A belt with no downstream is a DEAD END, not a drain.** Draining
    /// every unlinked tile silently deletes items, which manufactures
    /// throughput and — worse — removes the backpressure that makes a dead
    /// end re-compress its lane. That backpressure is the exact mechanism
    /// #448 turns on, so getting this wrong would have hidden the
    /// phenomenon the meter exists to measure. Found by the first
    /// end-to-end run: copper-cable read 45.00/s at plan while
    /// electronic-circuit sat at -57.8%, because cable was falling off an
    /// interior belt end instead of backing up.
    pub is_sink: bool,
}

impl BeltTile {
    pub fn occupancy(&self) -> usize {
        self.lanes[0].occupancy() + self.lanes[1].occupancy()
    }
}

/// Something the topology builder could not model faithfully. Surfaced
/// rather than swallowed — an unmodelled connection is a rate the meter
/// would get wrong without saying so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyNote {
    /// An underground entrance with no matching exit.
    UnpairedUnderground { pos: (i32, i32) },
    /// Belt tiles form a cycle; update order within it is arbitrary.
    CycleInUpdateOrder { tiles: usize },
    /// A splitter whose two halves could not be paired.
    OrphanSplitterHalf { pos: (i32, i32) },
}

#[derive(Debug, Default)]
pub struct BeltNetwork {
    pub tiles: Vec<BeltTile>,
    index: FxHashMap<(i32, i32), usize>,
    /// Downstream-first update order.
    order: Vec<usize>,
    /// Sub-slot advance carried per tier. Tiles of a tier step in lockstep,
    /// because belt speed is a property of the tier, not of the tile.
    progress: [f64; 4],
    /// Round-robin state per splitter id.
    splitter_toggle: Vec<bool>,
    /// Items that left the network at a tile with no downstream, since the
    /// caller last drained this. Boundary outputs are counted from here.
    pub exited_log: Vec<(usize, ItemId)>,
    pub notes: Vec<TopologyNote>,
}

fn tier_ix(t: BeltTier) -> usize {
    match t {
        BeltTier::Yellow => 0,
        BeltTier::Red => 1,
        BeltTier::Blue => 2,
        BeltTier::Turbo => 3,
    }
}

impl BeltNetwork {
    pub fn tile_at(&self, pos: (i32, i32)) -> Option<usize> {
        self.index.get(&pos).copied()
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Total items currently on the network.
    pub fn item_count(&self) -> usize {
        self.tiles.iter().map(|t| t.occupancy()).sum()
    }

    /// Take up to `max` items from a tile, both lanes (**I6**).
    pub fn take_from_tile(&mut self, tile: usize, max: u32, out: &mut Vec<ItemId>) {
        self.take_from_tile_filtered(tile, max, |_| true, out)
    }

    /// Take up to `max` items the predicate accepts, both lanes.
    ///
    /// **The filter is not an optimisation — it is mechanics I11.** A real
    /// inserter checks its destination before swinging and refuses items
    /// the destination cannot accept. Modelling it as "grab whatever is
    /// under the hand" deadlocks the inserter the first time a mixed belt
    /// presents a foreign item: the hand can never be emptied, so the
    /// inserter stops forever.
    ///
    /// That is not a hypothetical. It is what made `chain-mil5plates`
    /// read −61.1% against a real-measured PASS: grenade machines sat with
    /// iron-plate buffers capped full and coal at 5/10, on a coal belt that
    /// was full 78% of the time. Found by per-machine attribution, PR 4.
    pub fn take_from_tile_filtered<F>(
        &mut self,
        tile: usize,
        max: u32,
        mut accept: F,
        out: &mut Vec<ItemId>,
    ) where
        F: FnMut(ItemId) -> bool,
    {
        let mut remaining = max;
        let t = &mut self.tiles[tile];
        for lane in t.lanes.iter_mut() {
            if remaining == 0 {
                break;
            }
            let before = out.len();
            lane.take_matching(remaining, &mut accept, out);
            remaining -= (out.len() - before) as u32;
        }
    }

    /// Drop an item onto a tile's far lane (**I5**) — where an inserter
    /// puts things. Returns false when that lane's slots are full.
    pub fn drop_onto_tile(&mut self, tile: usize, from: (i32, i32), item: ItemId) -> bool {
        let t = &mut self.tiles[tile];
        // Far lane = the one on the opposite side from the dropper.
        let near = near_lane_from(t.dir, t.pos, from);
        let far = 1 - near;
        t.lanes[far].try_insert_anywhere(item)
    }

    /// Advance the whole network one tick.
    pub fn tick(&mut self) {
        // Which tiers step this tick.
        let mut stepping = [false; 4];
        for (ix, tier) in [
            BeltTier::Yellow,
            BeltTier::Red,
            BeltTier::Blue,
            BeltTier::Turbo,
        ]
        .into_iter()
        .enumerate()
        {
            self.progress[ix] += tier.slots_per_tick();
            if self.progress[ix] >= 1.0 {
                self.progress[ix] -= 1.0;
                stepping[ix] = true;
            }
        }
        if !stepping.iter().any(|s| *s) {
            return;
        }

        // Downstream-first, so a tile may move into the slot its successor
        // vacated in this same step. Upstream-first would cap a compressed
        // line at one item per gap per step, which is not how belts work.
        for i in 0..self.order.len() {
            let id = self.order[i];
            if !stepping[tier_ix(self.tiles[id].tier)] {
                continue;
            }
            self.step_tile(id);
        }
    }

    fn step_tile(&mut self, id: usize) {
        // 1. Hand off the exit slots to whatever is downstream.
        let (kind, downstream) = (self.tiles[id].kind, self.tiles[id].downstream);
        match kind {
            TileKind::Splitter { id: sid, .. } => self.step_splitter_exit(id, sid),
            _ => self.step_plain_exit(id, downstream),
        }
        // 2. Shift the rest of this tile forward.
        for lane in self.tiles[id].lanes.iter_mut() {
            lane.shift_forward();
        }
    }

    fn step_plain_exit(&mut self, id: usize, downstream: Option<Downstream>) {
        for lane_ix in 0..2 {
            let Some(item) = self.tiles[id].lanes[lane_ix].peek_exit() else {
                continue;
            };
            match downstream {
                None if self.tiles[id].is_sink => {
                    // A designated boundary output drains, so backpressure
                    // cannot falsify the measurement — the same reason the
                    // harness uses remove-mode chests.
                    self.tiles[id].lanes[lane_ix].take_exit();
                    self.tiles[id].exited += 1;
                    self.exited_log.push((id, item));
                }
                // Interior dead end: hold. The lane backs up, which is what
                // lets it re-compress.
                None => {}
                Some(d) => {
                    let target_lane = match d.lanes {
                        LaneMap::Straight => lane_ix,
                        LaneMap::OntoLane(l) => l,
                    };
                    if self.tiles[d.tile].lanes[target_lane].try_insert_entry(item) {
                        self.tiles[id].lanes[lane_ix].take_exit();
                    }
                    // else: downstream full — back up, which is the whole
                    // mechanism behind re-compression.
                }
            }
        }
    }

    /// Splitters alternate between their two outputs and preserve lanes
    /// (**S3**/**S4**). Output priority and filters are not modelled.
    fn step_splitter_exit(&mut self, id: usize, sid: usize) {
        let partner = match self.tiles[id].kind {
            TileKind::Splitter { partner, .. } => partner,
            _ => return,
        };
        let outs = [self.tiles[id].downstream, self.tiles[partner].downstream];
        for lane_ix in 0..2 {
            let Some(item) = self.tiles[id].lanes[lane_ix].peek_exit() else {
                continue;
            };
            let first = usize::from(self.splitter_toggle[sid]);
            let mut placed = false;
            for probe in 0..2 {
                let which = (first + probe) % 2;
                match outs[which] {
                    None if self.tiles[id].is_sink => {
                        self.tiles[id].lanes[lane_ix].take_exit();
                        self.tiles[id].exited += 1;
                        self.exited_log.push((id, item));
                        placed = true;
                    }
                    None => {}
                    Some(d) => {
                        let target_lane = match d.lanes {
                            LaneMap::Straight => lane_ix,
                            LaneMap::OntoLane(l) => l,
                        };
                        if self.tiles[d.tile].lanes[target_lane].try_insert_entry(item) {
                            self.tiles[id].lanes[lane_ix].take_exit();
                            placed = true;
                        }
                    }
                }
                if placed {
                    self.splitter_toggle[sid] = which == 0;
                    break;
                }
            }
        }
    }
}

/// Which lane of a tile facing `dir` at `pos` is nearest to `from`.
/// 0 = left, 1 = right (**B3**).
fn near_lane_from(dir: Dir, pos: (i32, i32), from: (i32, i32)) -> usize {
    let (lx, ly) = left_of(dir).delta();
    let left_tile = (pos.0 + lx, pos.1 + ly);
    if from == left_tile {
        0
    } else {
        1
    }
}

/// Build a belt network from decoded blueprint entities.
pub struct NetworkBuilder;

impl NetworkBuilder {
    pub fn build(entities: &[crate::blueprint_in::RawEntity]) -> BeltNetwork {
        let mut net = BeltNetwork::default();

        // --- 1. Create a node per belt-like tile ---------------------------
        let mut splitter_id = 0usize;
        let mut pending_splitters: Vec<(usize, (i32, i32))> = Vec::new();
        for e in entities {
            let Some(tier) = BeltTier::from_entity_name(&e.name) else {
                continue;
            };
            let is_ug = e.name.ends_with("underground-belt");
            let is_splitter = e.name.ends_with("splitter");

            // A splitter covers two tiles perpendicular to its facing.
            let cells: Vec<(i32, i32)> = if is_splitter {
                // `blueprint_in::decode` already resolved the ORIENTED
                // footprint to a top-left corner, so the second cell is
                // unconditionally +1 along the wide axis: (x+1,y) for a
                // north/south splitter, (x,y+1) for east/west.
                //
                // An earlier version derived it from `left_of(direction)`,
                // which flips sign between north and south — so SOUTH and
                // WEST splitters claimed a cell one tile the wrong way,
                // outside their own footprint, and never registered the
                // tile they actually occupy. Bus trunks run **south**, so
                // this silently unlinked tap-off branches across every bus
                // layout: 11 orphan belt heads in `logistic-science-pack`,
                // whose gear machine then read `iron-plate=0/2` forever
                // while the far-side belts beside it were full.
                if matches!(e.direction, Dir::North | Dir::South) {
                    vec![(e.x, e.y), (e.x + 1, e.y)]
                } else {
                    vec![(e.x, e.y), (e.x, e.y + 1)]
                }
            } else {
                vec![(e.x, e.y)]
            };

            let kind = if is_ug {
                match e.io_type.as_deref() {
                    Some("output") => TileKind::UgOutput,
                    _ => TileKind::UgInput,
                }
            } else if is_splitter {
                TileKind::Splitter {
                    partner: usize::MAX,
                    id: splitter_id,
                }
            } else {
                TileKind::Belt
            };

            let mut made = Vec::new();
            for pos in cells {
                if net.index.contains_key(&pos) {
                    continue;
                }
                let id = net.tiles.len();
                net.tiles.push(BeltTile {
                    pos,
                    dir: e.direction,
                    tier,
                    kind,
                    lanes: [Lane::new(SLOTS_PER_TILE), Lane::new(SLOTS_PER_TILE)],
                    downstream: None,
                    exited: 0,
                    is_sink: false,
                });
                net.index.insert(pos, id);
                made.push(id);
            }
            if is_splitter {
                if made.len() == 2 {
                    if let TileKind::Splitter { id: sid, .. } = kind {
                        net.tiles[made[0]].kind = TileKind::Splitter {
                            partner: made[1],
                            id: sid,
                        };
                        net.tiles[made[1]].kind = TileKind::Splitter {
                            partner: made[0],
                            id: sid,
                        };
                    }
                } else {
                    pending_splitters.push((splitter_id, (e.x, e.y)));
                }
                splitter_id += 1;
                net.splitter_toggle.push(false);
            }
        }
        for (_, pos) in pending_splitters {
            net.notes.push(TopologyNote::OrphanSplitterHalf { pos });
        }

        // --- 2. Underground pairing (U1-U5) --------------------------------
        Self::pair_undergrounds(&mut net);

        // --- 3. Downstream links -------------------------------------------
        Self::link_downstream(&mut net);

        // --- 4. Update order ------------------------------------------------
        Self::compute_order(&mut net);

        net
    }

    fn pair_undergrounds(net: &mut BeltNetwork) {
        let inputs: Vec<usize> = (0..net.tiles.len())
            .filter(|&i| net.tiles[i].kind == TileKind::UgInput)
            .collect();
        let mut claimed: FxHashSet<usize> = FxHashSet::default();

        for id in inputs {
            let (dir, tier, pos) = (net.tiles[id].dir, net.tiles[id].tier, net.tiles[id].pos);
            let reach = ug_max_gap(tier);
            let (dx, dy) = dir.delta();
            let mut found = None;
            // Nearest unclaimed exit of matching tier on the same axis,
            // within reach (U3/U5).
            for step in 1..=(reach + 1) {
                let probe = (pos.0 + dx * step, pos.1 + dy * step);
                let Some(&cand) = net.index.get(&probe) else {
                    continue;
                };
                let t = &net.tiles[cand];
                if t.kind == TileKind::UgOutput
                    && t.tier == tier
                    && t.dir == dir
                    && !claimed.contains(&cand)
                {
                    found = Some(cand);
                    break;
                }
            }
            match found {
                Some(exit) => {
                    claimed.insert(exit);
                    // The entrance feeds the exit directly, lanes intact (U9).
                    net.tiles[id].downstream = Some(Downstream {
                        tile: exit,
                        lanes: LaneMap::Straight,
                    });
                }
                None => net.notes.push(TopologyNote::UnpairedUnderground { pos }),
            }
        }
    }

    fn link_downstream(net: &mut BeltNetwork) {
        // Count feeders per tile so curves (B11) can be told from
        // sideloads (B8).
        let mut feeders: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        for id in 0..net.tiles.len() {
            if net.tiles[id].downstream.is_some() {
                continue; // underground entrance, already linked
            }
            let (dx, dy) = net.tiles[id].dir.delta();
            let ahead = (net.tiles[id].pos.0 + dx, net.tiles[id].pos.1 + dy);
            if let Some(&target) = net.index.get(&ahead) {
                feeders.entry(target).or_default().push(id);
            }
        }

        for id in 0..net.tiles.len() {
            if net.tiles[id].downstream.is_some() {
                continue;
            }
            let (pos, dir) = (net.tiles[id].pos, net.tiles[id].dir);
            let (dx, dy) = dir.delta();
            let ahead = (pos.0 + dx, pos.1 + dy);
            let Some(&target) = net.index.get(&ahead) else {
                continue; // layout edge
            };
            // An underground ENTRANCE only accepts from its back; its
            // forward tile is underground, not a surface neighbour.
            if net.tiles[target].kind == TileKind::UgOutput
                && net.tiles[id].kind != TileKind::UgInput
            {
                // Feeding the face of an exit is not a connection.
                if !is_back_feed(net.tiles[target].dir, net.tiles[target].pos, pos) {
                    continue;
                }
            }
            let tdir = net.tiles[target].dir;
            let lanes = if is_back_feed(tdir, ahead, pos) {
                LaneMap::Straight
            } else {
                let only_feeder = feeders.get(&target).map(|v| v.len() == 1).unwrap_or(false);
                if only_feeder {
                    // Curve: single side input renders as a turn and keeps
                    // both lanes (B11).
                    LaneMap::Straight
                } else {
                    LaneMap::OntoLane(near_lane_from(tdir, ahead, pos))
                }
            };
            net.tiles[id].downstream = Some(Downstream { tile: target, lanes });
        }
    }

    /// Kahn's algorithm over the downstream graph, sinks first. Any tiles
    /// left over sit in a cycle (a belt loop) and get an arbitrary order,
    /// which is recorded rather than hidden.
    fn compute_order(net: &mut BeltNetwork) {
        let n = net.tiles.len();
        let mut out_degree = vec![0usize; n];
        let mut upstream: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (id, deg) in out_degree.iter_mut().enumerate() {
            if let Some(d) = net.tiles[id].downstream {
                *deg = 1;
                upstream[d.tile].push(id);
            }
        }
        let mut queue: Vec<usize> = (0..n).filter(|&i| out_degree[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        let mut seen = vec![false; n];
        while let Some(id) = queue.pop() {
            if seen[id] {
                continue;
            }
            seen[id] = true;
            order.push(id);
            for &up in &upstream[id] {
                if out_degree[up] > 0 {
                    out_degree[up] -= 1;
                    if out_degree[up] == 0 {
                        queue.push(up);
                    }
                }
            }
        }
        let leftover: Vec<usize> = (0..n).filter(|&i| !seen[i]).collect();
        if !leftover.is_empty() {
            net.notes.push(TopologyNote::CycleInUpdateOrder {
                tiles: leftover.len(),
            });
            order.extend(leftover);
        }
        net.order = order;
    }
}

/// True when `from` sits directly behind a tile facing `dir` at `pos`.
fn is_back_feed(dir: Dir, pos: (i32, i32), from: (i32, i32)) -> bool {
    let (bx, by) = dir.opposite().delta();
    from == (pos.0 + bx, pos.1 + by)
}

/// Maximum gap (exclusive) between underground halves, per tier (**U3**).
fn ug_max_gap(tier: BeltTier) -> i32 {
    match tier {
        BeltTier::Yellow => 4,
        BeltTier::Red => 6,
        BeltTier::Blue => 8,
        BeltTier::Turbo => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint_in::RawEntity;

    fn belt(name: &str, x: i32, y: i32, dir: Dir) -> RawEntity {
        RawEntity {
            name: name.into(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: None,
        }
    }

    fn splitter(x: i32, y: i32, dir: Dir) -> RawEntity {
        RawEntity {
            name: "splitter".into(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: None,
        }
    }

    /// A splitter's two halves must be the tile it was decoded to plus the
    /// next one along its **wide** axis, in every direction.
    ///
    /// This is the shape of a live bug (fixed 2026-07-25): the second cell
    /// was derived from `left_of(direction)`, which flips sign between
    /// north and south, so SOUTH and WEST splitters claimed a cell one tile
    /// outside their own footprint. That tile then existed in the network
    /// with no upstream, while the tile the splitter really occupied did
    /// not exist at all — silently unlinking every downstream branch.
    /// Bus trunks run south, so it hit essentially every bus layout.
    ///
    /// Asserted per-direction rather than on a single case, because the
    /// bug was invisible on north/east and only appeared on the other two.
    #[test]
    fn splitter_claims_both_cells_of_its_own_footprint() {
        for (dir, expected_second) in [
            (Dir::North, (11, 5)),
            (Dir::South, (11, 5)),
            (Dir::East, (10, 6)),
            (Dir::West, (10, 6)),
        ] {
            let net = NetworkBuilder::build(&[splitter(10, 5, dir)]);
            assert_eq!(net.len(), 2, "{dir:?}: a splitter is two tiles");
            assert!(
                net.notes.is_empty(),
                "{dir:?}: halves must pair — notes {:?}",
                net.notes
            );
            let a = net.tile_at((10, 5)).expect("{dir:?}: top-left cell");
            let b = net
                .tile_at(expected_second)
                .unwrap_or_else(|| panic!("{dir:?}: second cell at {expected_second:?}"));
            match (net.tiles[a].kind, net.tiles[b].kind) {
                (
                    TileKind::Splitter { partner: pa, id: ia },
                    TileKind::Splitter { partner: pb, id: ib },
                ) => {
                    assert_eq!(pa, b, "{dir:?}: partner links must be mutual");
                    assert_eq!(pb, a, "{dir:?}: partner links must be mutual");
                    assert_eq!(ia, ib, "{dir:?}: both halves share one splitter id");
                }
                other => panic!("{dir:?}: expected two splitter halves, got {other:?}"),
            }
        }
    }

    /// The behavioural consequence: a belt feeding a south-facing splitter
    /// must reach it. Under the sign bug the splitter's real tile was
    /// absent from the network, so the belt above it linked to nothing and
    /// the branch below it became an orphan head with no upstream — items
    /// stopped dead at the trunk.
    #[test]
    fn belt_links_through_a_south_facing_splitter() {
        let ents = vec![
            belt("transport-belt", 10, 4, Dir::South),
            splitter(10, 5, Dir::South),
            belt("transport-belt", 10, 6, Dir::South),
            belt("transport-belt", 11, 6, Dir::South),
        ];
        let net = NetworkBuilder::build(&ents);
        let feeder = net.tile_at((10, 4)).unwrap();
        let half = net.tile_at((10, 5)).expect("splitter occupies its own tile");
        assert_eq!(
            net.tiles[feeder].downstream.map(|d| d.tile),
            Some(half),
            "the trunk belt must feed the splitter half beneath it"
        );
        for out in [(10, 6), (11, 6)] {
            let id = net.tile_at(out).unwrap();
            let fed = net.tiles.iter().any(|t| t.downstream.is_some_and(|d| d.tile == id));
            assert!(fed, "belt at {out:?} must have an upstream feeder");
        }
    }

    fn ug(name: &str, x: i32, y: i32, dir: Dir, io: &str) -> RawEntity {
        RawEntity {
            name: name.into(),
            x,
            y,
            direction: dir,
            recipe: None,
            io_type: Some(io.into()),
        }
    }

    #[test]
    fn straight_run_links_head_to_tail() {
        let ents: Vec<_> = (0..5)
            .map(|x| belt("transport-belt", x, 0, Dir::East))
            .collect();
        let net = NetworkBuilder::build(&ents);
        assert_eq!(net.len(), 5);
        for x in 0..4 {
            let id = net.tile_at((x, 0)).unwrap();
            let d = net.tiles[id].downstream.expect("linked");
            assert_eq!(net.tiles[d.tile].pos, (x + 1, 0));
            assert_eq!(d.lanes, LaneMap::Straight, "back feed preserves lanes");
        }
        assert!(net.tiles[net.tile_at((4, 0)).unwrap()].downstream.is_none());
        assert!(net.notes.is_empty(), "notes: {:?}", net.notes);
    }

    /// A single side input is a curve and must keep both lanes (B11).
    /// Treating it as a sideload would halve throughput at every corner.
    #[test]
    fn lone_side_feed_is_a_curve_not_a_sideload() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),  // feeds (1,0) from its west side
            belt("transport-belt", 1, 0, Dir::South), // turns south
        ];
        let net = NetworkBuilder::build(&ents);
        let d = net.tiles[net.tile_at((0, 0)).unwrap()].downstream.unwrap();
        assert_eq!(d.lanes, LaneMap::Straight);
    }

    /// With a back feed present too, the side feeder becomes a genuine
    /// sideload onto the near lane only (B8).
    #[test]
    fn side_feed_alongside_a_back_feed_is_a_sideload() {
        let ents = vec![
            belt("transport-belt", 1, 1, Dir::North), // back feed into (1,0)
            belt("transport-belt", 0, 0, Dir::East),  // side feed into (1,0)
            belt("transport-belt", 1, 0, Dir::North),
        ];
        let net = NetworkBuilder::build(&ents);
        let side = net.tiles[net.tile_at((0, 0)).unwrap()].downstream.unwrap();
        let back = net.tiles[net.tile_at((1, 1)).unwrap()].downstream.unwrap();
        assert_eq!(back.lanes, LaneMap::Straight, "back feed keeps both lanes");
        assert!(
            matches!(side.lanes, LaneMap::OntoLane(_)),
            "side feed must sideload, got {:?}",
            side.lanes
        );
    }

    #[test]
    fn underground_pairs_within_reach_and_notes_orphans() {
        // Yellow reaches a gap of 4, so entrance at x=0 pairs with exit at x=5.
        let ents = vec![
            ug("underground-belt", 0, 0, Dir::East, "input"),
            ug("underground-belt", 5, 0, Dir::East, "output"),
        ];
        let net = NetworkBuilder::build(&ents);
        let entrance = net.tile_at((0, 0)).unwrap();
        let d = net.tiles[entrance].downstream.expect("paired");
        assert_eq!(net.tiles[d.tile].pos, (5, 0));
        assert!(net.notes.is_empty());

        // Beyond reach: unpaired, and said so.
        let far = vec![
            ug("underground-belt", 0, 0, Dir::East, "input"),
            ug("underground-belt", 9, 0, Dir::East, "output"),
        ];
        let net = NetworkBuilder::build(&far);
        assert!(net
            .notes
            .contains(&TopologyNote::UnpairedUnderground { pos: (0, 0) }));
    }

    /// Express reaches further than yellow — the tier table must be live,
    /// not decorative.
    #[test]
    fn underground_reach_scales_with_tier() {
        let ents = vec![
            ug("express-underground-belt", 0, 0, Dir::East, "input"),
            ug("express-underground-belt", 9, 0, Dir::East, "output"),
        ];
        let net = NetworkBuilder::build(&ents);
        assert!(
            net.notes.is_empty(),
            "express should reach a gap of 8: {:?}",
            net.notes
        );
    }

    #[test]
    fn belt_loop_is_ordered_and_recorded() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            belt("transport-belt", 1, 0, Dir::South),
            belt("transport-belt", 1, 1, Dir::West),
            belt("transport-belt", 0, 1, Dir::North),
        ];
        let net = NetworkBuilder::build(&ents);
        assert_eq!(net.order.len(), 4, "every tile must be in the update order");
        assert!(
            net.notes
                .iter()
                .any(|n| matches!(n, TopologyNote::CycleInUpdateOrder { .. })),
            "a loop must be recorded, not silently ordered"
        );
    }
}
