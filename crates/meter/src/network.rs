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
use serde::Serialize;

use crate::belt::{ItemId, Lane};
use crate::blueprint_in::Dir;
use crate::entity_data::{
    BeltTier, INNER_TURN_SLOTS, OUTER_TURN_SLOTS, SIDELOAD_EARLY_POSITION, SIDELOAD_LATE_POSITION,
    SLOTS_PER_TILE,
};

const SPLITTER_BLOCK_MEMORY_ITEMS: u8 = 5;

type SplitterMemory = [[Option<(usize, u8)>; 2]; 2];
type LanePath = Vec<(usize, usize)>;
type DownstreamLines = [LanePath; 2];

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
    ///
    /// `partner` is `None` when the other half
    /// could not be created — its cell was already occupied — which is
    /// recorded as `TopologyNote::OrphanSplitterHalf`. Modelled as an
    /// `Option` rather than a sentinel index: the sentinel (`usize::MAX`)
    /// was indexed unguarded in `step_splitter_exit` and panicked the
    /// first time such a tile was stepped.
    Splitter {
        partner: Option<usize>,
        id: usize,
    },
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

/// Report-only counters for splitter routing decisions. These are reset with
/// the meter window and do not participate in movement decisions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SplitterStats {
    pub attempts: [u64; 2],
    pub first_blocked: [u64; 2],
    pub fallback_accepted: [u64; 2],
    pub first_accepted: [u64; 2],
    pub both_blocked: [u64; 2],
    pub remembered_accepted: [u64; 2],
    pub memory_started: [u64; 2],
    pub memory_expired: [u64; 2],
    pub half_attempts: [[u64; 2]; 2],
    pub half_fallback_accepted: [[u64; 2]; 2],
    pub half_both_blocked: [[u64; 2]; 2],
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
    /// Continuous coordinate of each tile lane within its connected
    /// transport line, measured in item spacings.  The coordinates are
    /// intentionally per lane: a sideload can join two upstream lanes while
    /// a straight belt keeps its two transport lines independent.
    lane_bases: Vec<[f64; 2]>,
    lane_components: Vec<[usize; 2]>,
    /// Precomputed forward line paths used by inserter collision checks.
    downstream_lines: Vec<DownstreamLines>,
    /// Whether a tile has an orthogonal straight feeder, i.e. is the curved
    /// leg of a turn-to-sideload merge. This is fixed by topology.
    turn_feeder: Vec<bool>,
    /// Independent round-robin state for each splitter input lane.
    splitter_toggle: Vec<[bool; 2]>,
    /// Per-half, per-input-lane memory of an output that was blocked. Factorio keeps
    /// up to five items for that side so a transiently blocked output catches
    /// up when it becomes available again (splitter mechanics, 0.3.0).
    splitter_memory: Vec<SplitterMemory>,
    /// Report-only routing counters, one record per splitter.
    pub splitter_stats: Vec<SplitterStats>,
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

    /// Continuous connected-line coordinate for diagnostics and probes.
    pub fn lane_segment(&self, tile: usize, lane: usize) -> Option<(usize, f64)> {
        Some((
            *self.lane_components.get(tile)?.get(lane)?,
            *self.lane_bases.get(tile)?.get(lane)?,
        ))
    }

    /// Total items currently on the network.
    pub fn item_count(&self) -> usize {
        self.tiles.iter().map(|t| t.occupancy()).sum()
    }

    /// Clear report-only splitter counters at a measurement-window boundary.
    pub fn reset_splitter_stats(&mut self) {
        self.splitter_stats.fill(SplitterStats::default());
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
    /// puts things. Vanilla inserters target the midpoint of the belt tile;
    /// the phase-aware projection is handled by [`Lane::try_insert_at`].
    /// Returns false when the local collision window is blocked.
    pub fn drop_onto_tile(&mut self, tile: usize, from: (i32, i32), item: ItemId) -> bool {
        self.drop_onto_tile_at(tile, from, 0.5, item)
    }

    /// Drop at a continuous local position on the target tile.
    pub fn drop_onto_tile_at(
        &mut self,
        tile: usize,
        from: (i32, i32),
        local_position: f64,
        item: ItemId,
    ) -> bool {
        // Far lane = the one on the opposite side from the dropper.
        let (dir, pos, tier) = (
            self.tiles[tile].dir,
            self.tiles[tile].pos,
            self.tiles[tile].tier,
        );
        let near = near_lane_from(dir, pos, from);
        let far = 1 - near;
        let base = self.lane_bases[tile][far];
        let progress_slots = self.progress[tier_ix(tier)];

        // A weak component is not a transport line: two feeder lanes can
        // share a component only after a sideload merge.  Walk forward from
        // this lane so an item on the other feeder is not treated as
        // colliding with a drop before the merge.  The target tile itself is
        // still handled by the slot projection below, preserving its proven
        // midpoint admission rule.
        for &(downstream_tile, downstream_lane) in
            self.downstream_line_nodes(tile, far).iter().skip(1)
        {
            let downstream_ref = &self.tiles[downstream_tile];
            let downstream_base = self.lane_bases[downstream_tile][downstream_lane];
            let phase = self.progress[tier_ix(downstream_ref.tier)];
            for (idx, slot) in downstream_ref.lanes[downstream_lane]
                .slots_debug()
                .iter()
                .enumerate()
            {
                if slot.is_some()
                    && (downstream_base
                        + idx as f64
                        + phase
                        + downstream_ref.lanes[downstream_lane].slot_offset(idx)
                        - (base + local_position / crate::entity_data::ITEM_SPACING_TILES))
                        .abs()
                        <= crate::belt::DROP_COLLISION_WINDOW_SLOTS + f64::EPSILON
                {
                    return false;
                }
            }
        }

        self.tiles[tile].lanes[far].try_insert_at_segment(
            local_position,
            progress_slots,
            base,
            item,
        )
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

    /// Return the precomputed forward transport-line nodes reachable from one lane.
    fn downstream_line_nodes(&self, tile: usize, lane: usize) -> &[(usize, usize)] {
        self.downstream_lines
            .get(tile)
            .and_then(|lanes| lanes.get(lane))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Compute forward transport-line nodes for every tile lane. A
    /// merge intentionally appears only after the two feeder paths converge;
    /// walking backwards through the weak component would incorrectly make
    /// the feeders share a line before that point.
    fn collect_downstream_line_nodes(&self, tile: usize, lane: usize) -> LanePath {
        let mut out = Vec::new();
        let mut seen: FxHashSet<(usize, usize)> = FxHashSet::default();
        let mut stack = vec![(tile, lane)];
        while let Some(node @ (id, lane_ix)) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }
            out.push(node);
            for downstream in self.downstream_edges(id) {
                let target_lane = match downstream.lanes {
                    LaneMap::Straight => lane_ix,
                    LaneMap::OntoLane(target) => target,
                };
                stack.push((downstream.tile, target_lane));
            }
        }
        out
    }

    fn compute_downstream_lines(net: &mut BeltNetwork) {
        let mut lines = vec![[Vec::new(), Vec::new()]; net.tiles.len()];
        for (tile, tile_lines) in lines.iter_mut().enumerate() {
            for (lane, path) in tile_lines.iter_mut().enumerate() {
                *path = net.collect_downstream_line_nodes(tile, lane);
            }
        }
        net.downstream_lines = lines;
    }

    fn downstream_edges(&self, id: usize) -> Vec<Downstream> {
        // A splitter's partner output is an alternate runtime route, not a
        // continuation of this lane's transport line. The splitter stepper
        // probes both outputs when it actually routes an item; the
        // pre-insertion collision walk must stay on the branch represented by
        // this physical half or an item on the sibling branch can reject a
        // drop on an otherwise free line.
        self.tiles[id].downstream.into_iter().collect()
    }

    fn step_plain_exit(&mut self, id: usize, downstream: Option<Downstream>) {
        for lane_ix in 0..2 {
            let Some((item, offset)) = self.tiles[id].lanes[lane_ix].peek_exit_with_offset() else {
                continue;
            };
            match downstream {
                None if self.tiles[id].is_sink => {
                    // A designated boundary output drains, so backpressure
                    // cannot falsify the measurement — the same reason the
                    // harness uses remove-mode chests.
                    self.tiles[id].lanes[lane_ix].take_exit_with_offset();
                    self.tiles[id].exited += 1;
                    self.exited_log.push((id, item));
                }
                // Interior dead end: hold. The lane backs up, which is what
                // lets it re-compress.
                None => {}
                Some(d) if self.try_insert_downstream(id, lane_ix, d, item, offset) => {
                    self.tiles[id].lanes[lane_ix].take_exit_with_offset();
                }
                Some(_) => {
                    // Downstream full — back up, which is the whole mechanism
                    // behind re-compression.
                }
            }
        }
    }

    /// Insert an item at the correct target-line point for a belt handoff.
    /// Straight feeds enter at the upstream edge. A sideload enters at one of
    /// the two measured internal positions within the target tile, rather
    /// than teleporting to its entry slot.
    fn try_insert_downstream(
        &mut self,
        source: usize,
        source_lane: usize,
        downstream: Downstream,
        item: ItemId,
        offset: f64,
    ) -> bool {
        let target_lane = match downstream.lanes {
            LaneMap::Straight => source_lane,
            LaneMap::OntoLane(lane) => lane,
        };
        match downstream.lanes {
            LaneMap::Straight => self.tiles[downstream.tile].lanes[target_lane]
                .try_insert_entry_with_offset(item, offset),
            LaneMap::OntoLane(_) => {
                let turn_merge = self.turn_feeder.get(source).copied().unwrap_or(false);
                let target = &self.tiles[downstream.tile];
                let local_position =
                    sideload_position(target.dir, target.pos, self.tiles[source].pos);
                let progress = self.progress[tier_ix(target.tier)];
                let inner_lane = turn_inner_lane(self.tiles[source].dir, target.dir);
                if turn_merge && inner_lane == Some(source_lane) {
                    self.tiles[downstream.tile].lanes[target_lane].try_insert_at_inner_turn_merge(
                        local_position,
                        progress,
                        0.0,
                        item,
                    )
                } else if turn_merge {
                    self.tiles[downstream.tile].lanes[target_lane].try_insert_at_turn_merge(
                        local_position,
                        progress,
                        0.0,
                        item,
                    )
                } else {
                    self.tiles[downstream.tile].lanes[target_lane].try_insert_at(
                        local_position,
                        progress,
                        item,
                    )
                }
            }
        }
    }

    fn compute_turn_feeders(net: &mut BeltNetwork) {
        let mut turn_feeder = vec![false; net.tiles.len()];
        for candidate in 0..net.tiles.len() {
            let Some(downstream) = net.tiles[candidate].downstream else {
                continue;
            };
            if matches!(downstream.lanes, LaneMap::Straight)
                && net.tiles[candidate].dir != net.tiles[downstream.tile].dir
            {
                turn_feeder[downstream.tile] = true;
            }
        }
        net.turn_feeder = turn_feeder;
    }

    /// Splitters distribute each input lane independently, preserving lanes
    /// and retaining bounded memory when an output is blocked
    /// (**S3**/**S4**/**S10**/**S11**). Output priority and filters are not
    /// modelled.
    fn step_splitter_exit(&mut self, id: usize, sid: usize) {
        let partner = match self.tiles[id].kind {
            TileKind::Splitter { partner, .. } => partner,
            _ => return,
        };
        let half_ix = usize::from(partner.is_some_and(|partner| id > partner));
        // An unpaired half has no second output. It still moves items —
        // degrading to plain-belt behaviour is closer to the truth than
        // either panicking or dropping the item on the floor — and the
        // topology note already tells the caller this tile's rates are
        // not to be trusted.
        let outs = [
            self.tiles[id].downstream,
            partner.map(|p| self.tiles[p].downstream).unwrap_or(None),
        ];
        for lane_ix in 0..2 {
            let Some((item, offset)) = self.tiles[id].lanes[lane_ix].peek_exit_with_offset() else {
                continue;
            };
            let remembered = self.splitter_memory[sid][half_ix][lane_ix];
            self.splitter_stats[sid].attempts[lane_ix] += 1;
            self.splitter_stats[sid].half_attempts[half_ix][lane_ix] += 1;
            let first = remembered
                .map(|(output, _)| output)
                .unwrap_or_else(|| usize::from(self.splitter_toggle[sid][lane_ix]));
            let mut placed = false;
            for probe in 0..2 {
                let which = (first + probe) % 2;
                match outs[which] {
                    None if self.tiles[id].is_sink => {
                        self.tiles[id].lanes[lane_ix].take_exit_with_offset();
                        self.tiles[id].exited += 1;
                        self.exited_log.push((id, item));
                        placed = true;
                    }
                    None => {}
                    Some(d) if self.try_insert_downstream(id, lane_ix, d, item, offset) => {
                        self.tiles[id].lanes[lane_ix].take_exit_with_offset();
                        placed = true;
                    }
                    Some(_) => {
                    }
                }
                if !placed && probe == 0 {
                    self.splitter_stats[sid].first_blocked[lane_ix] += 1;
                }
                if placed {
                    if which == first {
                        self.splitter_stats[sid].first_accepted[lane_ix] += 1;
                    } else {
                        self.splitter_stats[sid].fallback_accepted[lane_ix] += 1;
                        self.splitter_stats[sid].half_fallback_accepted[half_ix][lane_ix] += 1;
                    }
                    if let Some((remembered_output, remaining)) = remembered {
                        if which == remembered_output {
                            self.splitter_stats[sid].remembered_accepted[lane_ix] += 1;
                            self.splitter_memory[sid][half_ix][lane_ix] =
                                (remaining > 1).then_some((remembered_output, remaining - 1));
                            if remaining == 1 {
                                self.splitter_stats[sid].memory_expired[lane_ix] += 1;
                            }
                        }
                    } else if which != first && outs[first].is_some() {
                        // This branch is reached only when `remembered` was
                        // None. A fallback while an existing memory episode
                        // is still active must not refresh its five-item
                        // budget or inflate `memory_started`.
                        self.splitter_stats[sid].memory_started[lane_ix] += 1;
                        self.splitter_memory[sid][half_ix][lane_ix] =
                            Some((first, SPLITTER_BLOCK_MEMORY_ITEMS));
                    }
                    if self.splitter_memory[sid][half_ix][lane_ix].is_none() {
                        self.splitter_toggle[sid][lane_ix] = which == 0;
                    }
                    break;
                }
            }
            if !placed {
                self.splitter_stats[sid].both_blocked[lane_ix] += 1;
                self.splitter_stats[sid].half_both_blocked[half_ix][lane_ix] += 1;
            }
        }
    }
}

/// Which lane of a tile facing `dir` at `pos` is nearest to `from`.
/// 0 = left, 1 = right (**B3**).
///
/// Decided by the **sign of the perpendicular projection**, so it holds at
/// any distance. An earlier version tested `from == pos + left_of(dir)` —
/// exact equality against the single tile one step to the left — which is
/// only ever true for a reach-1 hand. A **long-handed inserter sits two
/// tiles away**, so the comparison failed unconditionally, `near` came back
/// 1 every time, and `far = 1 - near` put every reach-2 drop on lane 0
/// regardless of which side the inserter was actually on. Right by
/// coincidence on one side, wrong on the other, and invisible in aggregate
/// throughput while corrupting anything lane-sensitive (sideload
/// occupancy, splitter lane routing).
///
/// Strictly generalizes the old behaviour: at distance 1 on the left the
/// projection is `+1`, on the right `-1`, so belt-to-belt sideload callers
/// are unaffected.
fn near_lane_from(dir: Dir, pos: (i32, i32), from: (i32, i32)) -> usize {
    let (lx, ly) = left_of(dir).delta();
    let (dx, dy) = (from.0 - pos.0, from.1 - pos.1);
    if dx * lx + dy * ly > 0 {
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
                    partner: None,
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
                            partner: Some(made[1]),
                            id: sid,
                        };
                        net.tiles[made[1]].kind = TileKind::Splitter {
                            partner: Some(made[0]),
                            id: sid,
                        };
                    }
                } else {
                    pending_splitters.push((splitter_id, (e.x, e.y)));
                }
                splitter_id += 1;
                net.splitter_toggle.push([false; 2]);
                net.splitter_memory.push([[None; 2]; 2]);
                net.splitter_stats.push(SplitterStats::default());
            }
        }
        for (_, pos) in pending_splitters {
            net.notes.push(TopologyNote::OrphanSplitterHalf { pos });
        }

        // --- 2. Underground pairing (U1-U5) --------------------------------
        Self::pair_undergrounds(&mut net);

        // --- 3. Downstream links -------------------------------------------
        Self::link_downstream(&mut net);
        BeltNetwork::compute_turn_feeders(&mut net);
        BeltNetwork::compute_downstream_lines(&mut net);
        Self::configure_inner_turn_merge_lanes(&mut net);

        // --- 4. Update order ------------------------------------------------
        Self::compute_order(&mut net);
        Self::compute_lane_segments(&mut net);

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
                // U6 / B-straight: a straight feed loads both lanes.
                LaneMap::Straight
            } else if net.tiles[target].kind == TileKind::UgInput {
                // **U7**, and it is the opposite of B8: sideloading onto an
                // underground INPUT fills only the **far** lane. There is no
                // B11 curve exception here — U8 states that feeding straight
                // from behind is the *only* way to load both lanes of a UG
                // input, so a lone side-feeder does not render as a turn the
                // way it would onto plain belt.
                LaneMap::OntoLane(1 - near_lane_from(tdir, ahead, pos))
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
            net.tiles[id].downstream = Some(Downstream {
                tile: target,
                lanes,
            });
        }
    }

    /// The inner arc is a short continuous line (106 internal positions),
    /// but it does not expose two independently insertable quarter-grid
    /// slots when it immediately merges into a target.  Retaining one
    /// discrete slot here reproduces the measured steady-state asymmetry:
    /// the target gets three inner-lane items while one remains on the arc.
    /// This is scoped to turn-to-sideload topology; ordinary turns retain
    /// the four-slot tile representation.
    fn configure_inner_turn_merge_lanes(net: &mut BeltNetwork) {
        for source in 0..net.tiles.len() {
            let Some(curve) = net.tiles[source].downstream else {
                continue;
            };
            if !matches!(curve.lanes, LaneMap::Straight)
                || net.tiles[source].dir == net.tiles[curve.tile].dir
            {
                continue;
            }
            let Some(merge) = net.tiles[curve.tile].downstream else {
                continue;
            };
            if !matches!(merge.lanes, LaneMap::OntoLane(_)) {
                continue;
            }
            let Some(inner_lane) =
                turn_inner_lane(net.tiles[source].dir, net.tiles[curve.tile].dir)
            else {
                continue;
            };
            net.tiles[curve.tile].lanes[inner_lane] = Lane::new(INNER_TURN_SLOTS.floor() as usize);
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

    /// Assign continuous coordinates to the transport-line graph. A directed
    /// handoff advances along the actual connected line: straight neighbours
    /// differ by four item spacings, while a 90-degree curve uses Factorio's
    /// asymmetric inner/outer arc lengths. The graph is walked as undirected
    /// for the purpose of finding the connected line; this also joins both
    /// inputs of a sideload to the one target lane.
    fn compute_lane_segments(net: &mut BeltNetwork) {
        let node_count = net.tiles.len() * 2;
        let mut graph: Vec<Vec<(usize, f64)>> = vec![Vec::new(); node_count];
        let mut add_edge = |source: usize, downstream: Downstream| {
            for lane in 0..2 {
                let target_lane = match downstream.lanes {
                    LaneMap::Straight => lane,
                    LaneMap::OntoLane(target) => target,
                };
                let a = source * 2 + lane;
                let b = downstream.tile * 2 + target_lane;
                let delta = handoff_length_slots(
                    net.tiles[source].dir,
                    net.tiles[downstream.tile].dir,
                    downstream.lanes,
                    lane,
                );
                graph[a].push((b, delta));
                graph[b].push((a, -delta));
            }
        };

        for id in 0..net.tiles.len() {
            match net.tiles[id].kind {
                TileKind::Splitter { partner, .. } => {
                    if let Some(downstream) = net.tiles[id].downstream {
                        add_edge(id, downstream);
                    }
                    if let Some(partner) = partner {
                        if let Some(downstream) = net.tiles[partner].downstream {
                            add_edge(id, downstream);
                        }
                    }
                }
                _ => {
                    if let Some(downstream) = net.tiles[id].downstream {
                        add_edge(id, downstream);
                    }
                }
            }
        }

        let mut bases = vec![[0.0f64; 2]; net.tiles.len()];
        let mut components = vec![[0usize; 2]; net.tiles.len()];
        let mut seen = vec![false; node_count];
        let mut component = 0;
        for root in 0..node_count {
            if seen[root] {
                continue;
            }
            seen[root] = true;
            let mut stack = vec![root];
            while let Some(node) = stack.pop() {
                let tile = node / 2;
                let lane = node % 2;
                components[tile][lane] = component;
                let base = bases[tile][lane];
                for &(next, delta) in &graph[node] {
                    let next_tile = next / 2;
                    let next_lane = next % 2;
                    if !seen[next] {
                        seen[next] = true;
                        bases[next_tile][next_lane] = base + delta;
                        stack.push(next);
                    }
                }
            }
            component += 1;
        }
        net.lane_bases = bases;
        net.lane_components = components;
    }
}

/// Distance, in item spacings, between the local origins of two connected
/// lane segments. A single side-fed target is represented as a curve by
/// [`link_downstream`], so only that `Straight` handoff gets the asymmetric
/// inner/outer turn lengths.
fn handoff_length_slots(source_dir: Dir, target_dir: Dir, lanes: LaneMap, lane: usize) -> f64 {
    if !matches!(lanes, LaneMap::Straight) || source_dir == target_dir {
        return SLOTS_PER_TILE as f64;
    }

    let Some(inner_lane) = turn_inner_lane(source_dir, target_dir) else {
        return SLOTS_PER_TILE as f64;
    };
    if lane == inner_lane {
        INNER_TURN_SLOTS
    } else {
        OUTER_TURN_SLOTS
    }
}

/// Lane index occupied by the inside of a quarter-turn. Lane 0 is the
/// source belt's left lane and lane 1 its right lane.
fn turn_inner_lane(source: Dir, target: Dir) -> Option<usize> {
    match (source, target) {
        (Dir::North, Dir::West)
        | (Dir::West, Dir::South)
        | (Dir::South, Dir::East)
        | (Dir::East, Dir::North) => Some(0),
        (Dir::North, Dir::East)
        | (Dir::East, Dir::South)
        | (Dir::South, Dir::West)
        | (Dir::West, Dir::North) => Some(1),
        _ => None,
    }
}

/// Local position at which a perpendicular belt feed enters the target line.
/// The two sides of a target belt have the game's measured 68/188-position
/// entry geometry; lane 0 (the target's left side) is the late case and lane
/// 1 (the target's right side) the early case in the meter's lane convention.
fn sideload_position(target_dir: Dir, target_pos: (i32, i32), source_pos: (i32, i32)) -> f64 {
    match near_lane_from(target_dir, target_pos, source_pos) {
        0 => SIDELOAD_LATE_POSITION,
        _ => SIDELOAD_EARLY_POSITION,
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
            mirror: false,
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
            mirror: false,
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
                    TileKind::Splitter {
                        partner: pa,
                        id: ia,
                    },
                    TileKind::Splitter {
                        partner: pb,
                        id: ib,
                    },
                ) => {
                    assert_eq!(pa, Some(b), "{dir:?}: partner links must be mutual");
                    assert_eq!(pb, Some(a), "{dir:?}: partner links must be mutual");
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
        let half = net
            .tile_at((10, 5))
            .expect("splitter occupies its own tile");
        assert_eq!(
            net.tiles[feeder].downstream.map(|d| d.tile),
            Some(half),
            "the trunk belt must feed the splitter half beneath it"
        );
        for out in [(10, 6), (11, 6)] {
            let id = net.tile_at(out).unwrap();
            let fed = net
                .tiles
                .iter()
                .any(|t| t.downstream.is_some_and(|d| d.tile == id));
            assert!(fed, "belt at {out:?} must have an upstream feeder");
        }
    }

    #[test]
    fn orphan_splitter_half_does_not_panic_when_stepped() {
        // A splitter whose second cell is already occupied by a belt: the
        // second half is never created, so `partner` stays at its sentinel.
        let ents = vec![
            belt("transport-belt", 11, 5, Dir::South), // occupies the splitter's 2nd cell
            splitter(10, 5, Dir::South),
            belt("transport-belt", 10, 6, Dir::South),
        ];
        let mut net = NetworkBuilder::build(&ents);
        assert!(
            net.notes
                .iter()
                .any(|n| matches!(n, TopologyNote::OrphanSplitterHalf { .. })),
            "expected an orphan note, got {:?}",
            net.notes
        );
        // The real assertion: stepping must not panic.
        for _ in 0..10 {
            net.tick();
        }
    }

    #[test]
    fn splitter_remembers_a_blocked_output_per_lane() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            splitter(1, 0, Dir::East),
            belt("transport-belt", 2, 0, Dir::East),
            belt("transport-belt", 2, 1, Dir::East),
        ];
        let mut net = NetworkBuilder::build(&ents);
        let input = net.tile_at((1, 0)).unwrap();
        let output0 = net.tile_at((2, 0)).unwrap();
        let output1 = net.tile_at((2, 1)).unwrap();
        let sid = match net.tiles[input].kind {
            TileKind::Splitter { id, .. } => id,
            _ => panic!("expected splitter input half"),
        };

        for _ in 0..SLOTS_PER_TILE {
            assert!(net.tiles[output0].lanes[0].try_insert_anywhere(ItemId(9)));
        }
        assert!(net.tiles[input].lanes[0].try_insert_at(0.75, 0.0, ItemId(1)));
        net.step_splitter_exit(input, sid);
        assert_eq!(net.tiles[output1].lanes[0].occupancy(), 1);
        assert_eq!(net.splitter_memory[sid][0][0], Some((0, 5)));
        assert_eq!(net.splitter_stats[sid].attempts[0], 1);
        assert_eq!(net.splitter_stats[sid].first_blocked[0], 1);
        assert_eq!(net.splitter_stats[sid].fallback_accepted[0], 1);
        assert_eq!(net.splitter_stats[sid].memory_started[0], 1);

        // A second item also falls back while the remembered output remains
        // blocked. It must not restart the same memory episode or refresh its
        // budget; only acceptance by the remembered output consumes memory.
        let mut discarded_second = Vec::new();
        net.tiles[output1]
            .lanes[0]
            .take_all(99, &mut discarded_second);
        assert!(net.tiles[input].lanes[0].try_insert_at(0.75, 0.0, ItemId(1)));
        net.step_splitter_exit(input, sid);
        assert_eq!(net.tiles[output1].lanes[0].occupancy(), 1);
        assert_eq!(net.splitter_memory[sid][0][0], Some((0, 5)));
        assert_eq!(net.splitter_stats[sid].memory_started[0], 1);

        let mut discarded = Vec::new();
        net.tiles[output0].lanes[0].take_all(99, &mut discarded);
        assert!(net.tiles[input].lanes[0].try_insert_at(0.75, 0.0, ItemId(1)));
        net.step_splitter_exit(input, sid);
        assert_eq!(net.tiles[output0].lanes[0].occupancy(), 1);
        assert_eq!(net.splitter_memory[sid][0][0], Some((0, 4)));
        assert_eq!(net.splitter_stats[sid].attempts[0], 3);
        assert_eq!(net.splitter_stats[sid].fallback_accepted[0], 2);
        assert_eq!(net.splitter_stats[sid].remembered_accepted[0], 1);
    }

    #[test]
    fn splitter_blocked_memory_is_isolated_between_physical_halves() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            splitter(1, 0, Dir::East),
            belt("transport-belt", 2, 0, Dir::East),
            belt("transport-belt", 2, 1, Dir::East),
        ];
        let mut net = NetworkBuilder::build(&ents);
        let first_half = net.tile_at((1, 0)).unwrap();
        let second_half = net.tile_at((1, 1)).unwrap();
        let first_output = net.tile_at((2, 0)).unwrap();
        let second_output = net.tile_at((2, 1)).unwrap();
        let sid = match net.tiles[first_half].kind {
            TileKind::Splitter { id, .. } => id,
            _ => panic!("expected splitter input half"),
        };
        let first_half_ix = usize::from(first_half > second_half);
        let second_half_ix = 1 - first_half_ix;

        // Seed only the first physical half. The shared round-robin toggle is
        // deliberately set to the other output: a shared-memory
        // implementation would route the second half to `second_output`,
        // while the isolated half follows the toggle to `first_output`.
        net.splitter_memory[sid][first_half_ix][0] = Some((0, 5));
        net.splitter_toggle[sid][0] = true;
        assert!(net.tiles[second_half].lanes[0].try_insert_at(0.75, 0.0, ItemId(1)));

        net.step_splitter_exit(second_half, sid);

        assert_eq!(net.tiles[first_output].lanes[0].occupancy(), 1);
        assert_eq!(net.tiles[second_output].lanes[0].occupancy(), 0);
        assert_eq!(net.splitter_memory[sid][second_half_ix][0], None);
        assert_eq!(net.splitter_memory[sid][first_half_ix][0], Some((0, 5)));
    }

    #[test]
    fn downstream_line_stays_on_the_splitter_half_branch() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            splitter(1, 0, Dir::East),
            belt("transport-belt", 2, 0, Dir::East),
            belt("transport-belt", 2, 1, Dir::East),
        ];
        let net = NetworkBuilder::build(&ents);
        let input = net.tile_at((0, 0)).unwrap();
        let own_half = net.tile_at((1, 0)).unwrap();
        let own_output = net.tile_at((2, 0)).unwrap();
        let sibling_output = net.tile_at((2, 1)).unwrap();

        let path = net.downstream_line_nodes(input, 0);
        assert!(path.contains(&(own_half, 0)));
        assert!(path.contains(&(own_output, 0)));
        assert!(!path.contains(&(sibling_output, 0)));
    }

    /// **I5**: an inserter drops on the FAR lane — the one on the opposite
    /// side from where it stands. Asserted at reach 1 *and* reach 2,
    /// because the reach-2 case is where this was wrong: exact-tile
    /// equality against the one-step-left tile can only ever match a
    /// reach-1 hand, so every long-handed drop landed on lane 0 whichever
    /// side it came from.
    #[test]
    fn drops_land_on_the_far_lane_at_both_reaches() {
        for dist in [1, 2] {
            // Belt at (10,5) facing south. Left of south is east (+x),
            // so a dropper to the EAST is on the near side (lane 0) and
            // its item must land on lane 1, and vice versa.
            let mut net = NetworkBuilder::build(&[belt("transport-belt", 10, 5, Dir::South)]);
            let tile = net.tile_at((10, 5)).unwrap();
            let item = ItemId(1);

            assert!(
                net.drop_onto_tile(tile, (10 + dist, 5), item),
                "east drop, dist {dist}"
            );
            assert_eq!(
                net.tiles[tile].lanes[1].occupancy(),
                1,
                "dist {dist}: a dropper on the east (near) side must fill the FAR lane 1"
            );
            assert_eq!(net.tiles[tile].lanes[0].occupancy(), 0, "dist {dist}");

            let mut net = NetworkBuilder::build(&[belt("transport-belt", 10, 5, Dir::South)]);
            let tile = net.tile_at((10, 5)).unwrap();
            assert!(
                net.drop_onto_tile(tile, (10 - dist, 5), item),
                "west drop, dist {dist}"
            );
            assert_eq!(
                net.tiles[tile].lanes[0].occupancy(),
                1,
                "dist {dist}: a dropper on the west (far-side) must fill lane 0"
            );
            assert_eq!(net.tiles[tile].lanes[1].occupancy(), 0, "dist {dist}");
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
            mirror: false,
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
            belt("transport-belt", 0, 0, Dir::East), // feeds (1,0) from its west side
            belt("transport-belt", 1, 0, Dir::South), // turns south
        ];
        let net = NetworkBuilder::build(&ents);
        let d = net.tiles[net.tile_at((0, 0)).unwrap()].downstream.unwrap();
        assert_eq!(d.lanes, LaneMap::Straight);
    }

    /// The OPPOSITE chirality is also lane-for-lane. Documentation-level
    /// lock from the 2026-08-21 chirality adjudication (this model was
    /// CLEARED; the swap was core's bug, fixed in belt_flow on PR #683):
    /// honestly, this test is NON-discriminating today — link_downstream's
    /// only_feeder branch never consults chirality, so both arms take the
    /// same code path (#685 review). It pins that a future refactor which
    /// ADDS a chirality distinction here fails loudly; the discriminating
    /// lock (asymmetric lanes, both arms) lives in belt_flow's
    /// lane_transfer test on #683.
    #[test]
    fn lone_side_feed_opposite_chirality_also_curves() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East), // feeds (1,0)
            belt("transport-belt", 1, 0, Dir::North), // turns NORTH (other handedness)
        ];
        let net = NetworkBuilder::build(&ents);
        let d = net.tiles[net.tile_at((0, 0)).unwrap()].downstream.unwrap();
        assert_eq!(d.lanes, LaneMap::Straight, "B11: no chirality-dependent swap");
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
    fn sideload_enters_at_the_measured_side_dependent_position() {
        for (side_y, target_lane, expected_slot) in [
            (-1, 0, 3), // target-left side: late entry at 188/256 of the tile
            (1, 1, 1),  // target-right side: early entry at 68/256 of the tile
        ] {
            let ents = vec![
                belt("transport-belt", 0, 0, Dir::East),
                belt("transport-belt", 1, 0, Dir::East),
                belt(
                    "transport-belt",
                    1,
                    side_y,
                    if side_y < 0 { Dir::South } else { Dir::North },
                ),
            ];
            let mut net = NetworkBuilder::build(&ents);
            let side = net.tile_at((1, side_y)).unwrap();
            let target = net.tile_at((1, 0)).unwrap();
            assert!(net.tiles[side].lanes[0].try_insert(ItemId(7)));
            for _ in 0..(SLOTS_PER_TILE - 1) {
                net.tiles[side].lanes[0].shift_forward();
            }
            let downstream = net.tiles[side].downstream.unwrap();
            net.step_plain_exit(side, Some(downstream));

            assert_eq!(downstream.tile, target);
            assert_eq!(
                net.tiles[target].lanes[target_lane].slots_debug()[expected_slot],
                Some(ItemId(7))
            );
        }
    }

    /// **U7**: sideloading onto an underground INPUT fills the **far**
    /// lane, the opposite of B8's near-lane rule for plain belt.
    ///
    /// Both directions are asserted, because a single-side test passes
    /// under the old near-lane code for whichever side happens to match:
    /// the defect is a sign error, and a sign error is only visible from
    /// both sides. Feeding from the west and from the east must land on
    /// *different* lanes, and each must be the opposite of the plain-belt
    /// answer for the same geometry.
    #[test]
    fn sideload_onto_ug_input_fills_the_far_lane() {
        for (fx, fy, feeder_dir) in [(1, -1, Dir::South), (1, 1, Dir::North)] {
            let ents = vec![
                belt("transport-belt", 0, 0, Dir::East), // back feed into the UG input
                belt("transport-belt", fx, fy, feeder_dir), // side feed into it
                ug("underground-belt", 1, 0, Dir::East, "input"),
                ug("underground-belt", 4, 0, Dir::East, "output"),
            ];
            let net = NetworkBuilder::build(&ents);
            let side = net.tiles[net.tile_at((fx, fy)).unwrap()]
                .downstream
                .unwrap();
            let near = near_lane_from(Dir::East, (1, 0), (fx, fy));
            assert_eq!(
                side.lanes,
                LaneMap::OntoLane(1 - near),
                "U7: side feed from {:?} onto a UG input must fill the FAR lane \
                 (near={near}), got {:?}",
                (fx, fy),
                side.lanes
            );
        }
    }

    /// U8: a *lone* side feeder onto a UG input is still a far-lane
    /// sideload — the B11 curve rule does not apply, because feeding
    /// straight from behind is the only way to load both lanes.
    #[test]
    fn lone_side_feed_onto_ug_input_is_not_a_curve() {
        let ents = vec![
            belt("transport-belt", 1, -1, Dir::South),
            ug("underground-belt", 1, 0, Dir::East, "input"),
            ug("underground-belt", 4, 0, Dir::East, "output"),
        ];
        let net = NetworkBuilder::build(&ents);
        let side = net.tiles[net.tile_at((1, -1)).unwrap()].downstream.unwrap();
        assert_eq!(
            side.lanes,
            LaneMap::OntoLane(1 - near_lane_from(Dir::East, (1, 0), (1, -1))),
            "a lone side feed onto a UG input must NOT keep both lanes (U8)"
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

    #[test]
    fn connected_lane_segments_share_a_coordinate_system() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            belt("transport-belt", 1, 0, Dir::East),
        ];
        let net = NetworkBuilder::build(&ents);
        let a = net.tile_at((0, 0)).unwrap();
        let b = net.tile_at((1, 0)).unwrap();
        for lane in 0..2 {
            let (a_component, a_base) = net.lane_segment(a, lane).unwrap();
            let (b_component, b_base) = net.lane_segment(b, lane).unwrap();
            assert_eq!(a_component, b_component);
            assert!((b_base - a_base).abs() == SLOTS_PER_TILE as f64);
        }
    }

    #[test]
    fn curve_lane_segments_use_inner_and_outer_lengths() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            belt("transport-belt", 1, 0, Dir::South),
        ];
        let net = NetworkBuilder::build(&ents);
        let source = net.tile_at((0, 0)).unwrap();
        let target = net.tile_at((1, 0)).unwrap();

        for lane in 0..2 {
            let (source_component, source_base) = net.lane_segment(source, lane).unwrap();
            let (target_component, target_base) = net.lane_segment(target, lane).unwrap();
            assert_eq!(source_component, target_component);
            let expected = if lane == 1 {
                INNER_TURN_SLOTS
            } else {
                OUTER_TURN_SLOTS
            };
            assert!((target_base - source_base - expected).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn turn_merge_curve_fixture_fills_target_by_tick_twenty_two() {
        let ents = vec![
            belt("express-transport-belt", 0, 0, Dir::East),
            belt("express-transport-belt", 1, 0, Dir::South),
            belt("express-transport-belt", 1, 1, Dir::West),
            belt("express-transport-belt", 2, 1, Dir::West),
        ];
        let mut net = NetworkBuilder::build(&ents);
        let source = net.tile_at((0, 0)).unwrap();
        let back = net.tile_at((2, 1)).unwrap();
        let target = net.tile_at((1, 1)).unwrap();
        for _ in 0..4 {
            assert!(net.tiles[source].lanes[0].try_insert_anywhere(ItemId(1)));
            assert!(net.tiles[back].lanes[0].try_insert_anywhere(ItemId(2)));
        }

        for _ in 0..22 {
            net.tick();
        }

        assert_eq!(net.tiles[target].lanes[0].occupancy(), 4);
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 4);
    }

    #[test]
    fn inner_turn_merge_curve_fixture_keeps_one_item_on_the_curve() {
        let ents = vec![
            belt("express-transport-belt", 0, 0, Dir::East),
            belt("express-transport-belt", 1, 0, Dir::South),
            belt("express-transport-belt", 1, 1, Dir::West),
            belt("express-transport-belt", 2, 1, Dir::West),
        ];
        let mut net = NetworkBuilder::build(&ents);
        let source = net.tile_at((0, 0)).unwrap();
        let curve = net.tile_at((1, 0)).unwrap();
        let back = net.tile_at((2, 1)).unwrap();
        let target = net.tile_at((1, 1)).unwrap();
        for _ in 0..4 {
            assert!(net.tiles[source].lanes[1].try_insert_anywhere(ItemId(1)));
            assert!(net.tiles[back].lanes[0].try_insert_anywhere(ItemId(2)));
        }

        // The exact Factorio line-2 probe admits at ticks 5, 8, and 10.
        // The discrete inner merge is one tick behind the first and third
        // admissions, but preserves the measured three-on-target/one-on-arc
        // steady state without applying the outer-lane packing rule.
        for _ in 0..5 {
            net.tick();
        }
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 0);
        net.tick();
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 1);
        net.tick();
        net.tick();
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 2);
        net.tick();
        net.tick();
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 2);
        net.tick();
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 3);
        for _ in 0..19 {
            net.tick();
        }

        assert_eq!(net.tiles[target].lanes[0].occupancy(), 4);
        assert_eq!(net.tiles[target].lanes[1].occupancy(), 3);
        assert_eq!(net.tiles[curve].lanes[1].occupancy(), 1);
    }

    #[test]
    fn left_turn_merge_uses_the_orientation_aware_inner_lane() {
        let ents = vec![
            belt("express-transport-belt", 0, 0, Dir::East),
            belt("express-transport-belt", 1, 0, Dir::North),
            belt("express-transport-belt", 1, -1, Dir::West),
            belt("express-transport-belt", 2, -1, Dir::West),
        ];
        let mut net = NetworkBuilder::build(&ents);
        let source = net.tile_at((0, 0)).unwrap();
        let curve = net.tile_at((1, 0)).unwrap();
        let back = net.tile_at((2, -1)).unwrap();
        let target = net.tile_at((1, -1)).unwrap();

        // East -> North is the opposite chirality from the existing
        // East -> South fixture, so lane 0 is the inner lane here.
        assert_eq!(net.tiles[curve].lanes[0].slots_debug().len(), 1);
        assert_eq!(
            net.tiles[curve].lanes[1].slots_debug().len(),
            SLOTS_PER_TILE
        );
        for _ in 0..4 {
            assert!(net.tiles[source].lanes[0].try_insert_anywhere(ItemId(1)));
            assert!(net.tiles[back].lanes[1].try_insert_anywhere(ItemId(2)));
        }

        for _ in 0..30 {
            net.tick();
        }

        assert_eq!(net.tiles[target].lanes[1].occupancy(), 4);
        assert_eq!(net.tiles[target].lanes[0].occupancy(), 1);
        assert_eq!(net.tiles[curve].lanes[0].occupancy(), 1);
    }

    #[test]
    fn merged_feeders_share_a_component_but_not_a_premerge_line() {
        let ents = vec![
            belt("transport-belt", 0, 0, Dir::East),
            belt("transport-belt", 1, 0, Dir::East),
            belt("transport-belt", 2, 0, Dir::East),
            // A south-facing side feeder joins the target's left lane.
            belt("transport-belt", 1, -1, Dir::South),
        ];
        let net = NetworkBuilder::build(&ents);
        let main = net.tile_at((0, 0)).unwrap();
        let target = net.tile_at((1, 0)).unwrap();
        let side = net.tile_at((1, -1)).unwrap();
        let downstream = net.tile_at((2, 0)).unwrap();

        let (main_component, _) = net.lane_segment(main, 0).unwrap();
        let (side_component, _) = net.lane_segment(side, 0).unwrap();
        let (target_component, _) = net.lane_segment(target, 0).unwrap();
        assert_eq!(main_component, side_component);
        assert_eq!(side_component, target_component);

        let target_line = net.downstream_line_nodes(target, 0);
        assert!(target_line.contains(&(target, 0)));
        assert!(target_line.contains(&(downstream, 0)));
        assert!(!target_line.contains(&(main, 0)));
        assert!(!target_line.contains(&(side, 0)));
    }
}
