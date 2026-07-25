//! Item-level belt model.
//!
//! **The load-bearing design decision of RFC-054: items, not rates.** A
//! rate-based belt would reproduce the validator's blindness by
//! construction. Here a lane is a sequence of discrete slots at Factorio's
//! item spacing, and an item advances only when the slot ahead is free.
//!
//! Three properties fall out of that, unprogrammed, and they are the whole
//! reason the meter can see the #448 class:
//!
//! 1. **Compression happens only against a blockage.** Items on a
//!    free-flowing lane all advance together, so spacing is preserved.
//! 2. **Gaps do not heal.** An item removed mid-run leaves a hole that
//!    travels downstream forever unless something upstream is backed up.
//! 3. **Order is FIFO.**
//!
//! Nothing below special-cases tail starvation. It is a consequence.
//!
//! ## PR 1 scope: linear runs only
//!
//! This lands the physics for a *linear* belt run — the topology the #448
//! mechanism lives in and the one PR 2's margin sweep needs. Branching
//! (splitters, sideloads, underground pairs linking separate runs) is
//! **refused loudly** rather than silently approximated; see
//! [`crate::world::World::from_blueprint`]. Generalisation lands with PR 3.

use crate::entity_data::{BeltTier, SLOTS_PER_TILE};

/// Interned item identifier. The meter never reasons about item names in
/// the hot loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub u16);

/// What happens to an item that reaches the downstream end of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// Nothing consumes it: items pile up and the run backs up.
    ///
    /// This is the ordinary shape of a bus row's input belt, and it is
    /// central to #448: a dead end *would* re-compress the lane if surplus
    /// ever reached it, but a starving tail consumes everything that
    /// arrives, so it never backs up and the upstream gaps never heal.
    DeadEnd,
    /// Items are removed and counted (a drain, or the world edge).
    Sink,
}

/// One lane of a linear belt run.
#[derive(Debug, Clone)]
pub struct Lane {
    /// Slot 0 is the upstream (entry) end; the last slot is the exit.
    slots: Vec<Option<ItemId>>,
    /// Sub-slot advance carried between ticks. Belt speeds are not
    /// integer slots-per-tick (express is 2.67 ticks/slot), so the
    /// fractional remainder must persist or the model would quantise
    /// every tier down to the same speed.
    progress: f64,
    /// Items that left this lane through the run end, by tick of exit.
    pub exited: u64,
}

impl Lane {
    pub fn new(slot_count: usize) -> Self {
        Lane {
            slots: vec![None; slot_count],
            progress: 0.0,
            exited: 0,
        }
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn occupancy(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Fraction of slots occupied — the density that governs how much an
    /// inserter above this lane can actually pick up.
    pub fn density(&self) -> f64 {
        if self.slots.is_empty() {
            return 0.0;
        }
        self.occupancy() as f64 / self.slots.len() as f64
    }

    /// Occupied slot count within one tile of the run.
    pub fn occupancy_in_tile(&self, tile_index: usize) -> usize {
        let lo = tile_index * SLOTS_PER_TILE;
        let hi = (lo + SLOTS_PER_TILE).min(self.slots.len());
        if lo >= hi {
            return 0;
        }
        self.slots[lo..hi].iter().filter(|s| s.is_some()).count()
    }

    /// Try to place an item at the upstream end. Returns false when slot 0
    /// is occupied — i.e. the lane is backed up all the way to its source,
    /// which is what makes a producer go `full_output`.
    pub fn try_insert(&mut self, item: ItemId) -> bool {
        match self.slots.first_mut() {
            Some(slot @ None) => {
                *slot = Some(item);
                true
            }
            _ => false,
        }
    }

    /// Remove up to `max` items from `tile_index`, most-downstream first.
    ///
    /// Downstream-first matters: it is what leaves the *upstream* part of
    /// the tile empty for the next arrival, and it is how a real inserter
    /// grabbing from a belt behaves.
    pub fn take_from_tile(&mut self, tile_index: usize, max: u32, out: &mut Vec<ItemId>) {
        if max == 0 {
            return;
        }
        let lo = tile_index * SLOTS_PER_TILE;
        let hi = (lo + SLOTS_PER_TILE).min(self.slots.len());
        if lo >= hi {
            return;
        }
        let mut taken = 0u32;
        for idx in (lo..hi).rev() {
            if taken >= max {
                break;
            }
            if let Some(item) = self.slots[idx].take() {
                out.push(item);
                taken += 1;
            }
        }
    }

    // --- Tile-graph API (crate::network) --------------------------------
    //
    // The linear `BeltRun` above treats a lane as one long strip; the tile
    // graph treats each tile's lane as its own short strip and moves items
    // across tile boundaries explicitly. Same slot physics, different
    // granularity — deliberately one `Lane` type rather than two, since
    // duplicated state is this repo's named recurring hazard.

    /// The exit slot's contents, if any (the downstream end).
    pub fn peek_exit(&self) -> Option<ItemId> {
        self.slots.last().copied().flatten()
    }

    /// Remove and return the exit slot's item.
    pub fn take_exit(&mut self) -> Option<ItemId> {
        self.slots.last_mut().and_then(|s| s.take())
    }

    /// Insert at the entry (upstream) end. False when it is occupied,
    /// which is how a backed-up lane refuses its feeder.
    pub fn try_insert_entry(&mut self, item: ItemId) -> bool {
        self.try_insert(item)
    }

    /// Insert into any free slot, preferring the entry end.
    ///
    /// Used for inserter drops. A real inserter drops at a specific point
    /// on the tile; approximating that as "the first free slot" is a stated
    /// simplification — it can place an item marginally further along than
    /// the game would, which slightly *favours* throughput. Candidate
    /// `docs/meter-divergence.md` entry.
    pub fn try_insert_anywhere(&mut self, item: ItemId) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(item);
                return true;
            }
        }
        false
    }

    /// Take up to `max` items, most-downstream first.
    pub fn take_all(&mut self, max: u32, out: &mut Vec<ItemId>) {
        let mut taken = 0u32;
        for idx in (0..self.slots.len()).rev() {
            if taken >= max {
                break;
            }
            if let Some(item) = self.slots[idx].take() {
                out.push(item);
                taken += 1;
            }
        }
    }

    /// Shift items one slot toward the exit, downstream-first, without
    /// touching the exit slot itself (the caller hands that off first).
    pub fn shift_forward(&mut self) {
        if self.slots.is_empty() {
            return;
        }
        for idx in (1..self.slots.len()).rev() {
            if self.slots[idx].is_none() {
                self.slots[idx] = self.slots[idx - 1].take();
            }
        }
    }

    /// Advance one tick. Returns the number of whole-slot steps applied.
    fn tick(&mut self, tier: BeltTier, end: RunEnd) -> u32 {
        self.progress += tier.slots_per_tick();
        let mut steps = 0;
        while self.progress >= 1.0 {
            self.progress -= 1.0;
            self.step(end);
            steps += 1;
        }
        steps
    }

    /// One whole-slot advance, processed **downstream-first**.
    ///
    /// Downstream-first is what lets a fully compressed lane move at full
    /// speed: each item may enter the slot its neighbour vacated in this
    /// same step. Processing upstream-first would cap a compressed lane at
    /// one item per step per gap, which is not how belts work.
    fn step(&mut self, end: RunEnd) {
        if self.slots.is_empty() {
            return;
        }
        let last = self.slots.len() - 1;

        // 1. The exit slot: eject if the run end accepts, else it stays put
        //    and everything behind it backs up.
        if self.slots[last].is_some() && end == RunEnd::Sink {
            self.slots[last] = None;
            self.exited += 1;
        }

        // 2. Shift the rest forward into any free slot ahead.
        for idx in (1..=last).rev() {
            if self.slots[idx].is_none() {
                self.slots[idx] = self.slots[idx - 1].take();
            }
        }
    }
}

/// A linear run of belt tiles, both lanes.
#[derive(Debug, Clone)]
pub struct BeltRun {
    pub tier: BeltTier,
    pub end: RunEnd,
    /// Tile coordinates in flow order; slot `s` of a lane sits in tile
    /// `s / SLOTS_PER_TILE`.
    pub tiles: Vec<(i32, i32)>,
    /// Index 0 = left lane, 1 = right lane (`factorio-mechanics.md` B2/B3).
    pub lanes: [Lane; 2],
}

impl BeltRun {
    pub fn new(tier: BeltTier, tiles: Vec<(i32, i32)>, end: RunEnd) -> Self {
        let slot_count = tiles.len() * SLOTS_PER_TILE;
        BeltRun {
            tier,
            end,
            tiles,
            lanes: [Lane::new(slot_count), Lane::new(slot_count)],
        }
    }

    pub fn tile_index(&self, pos: (i32, i32)) -> Option<usize> {
        self.tiles.iter().position(|&t| t == pos)
    }

    pub fn tick(&mut self) {
        let (tier, end) = (self.tier, self.end);
        for lane in &mut self.lanes {
            lane.tick(tier, end);
        }
    }

    /// Total items currently on the run.
    pub fn item_count(&self) -> usize {
        self.lanes.iter().map(|l| l.occupancy()).sum()
    }

    /// Mean occupancy across both lanes, 0.0..=1.0.
    pub fn density(&self) -> f64 {
        (self.lanes[0].density() + self.lanes[1].density()) / 2.0
    }

    /// Occupied slots in one tile across both lanes — what an inserter
    /// above that tile can see. Inserters pick from **both lanes**
    /// (`factorio-mechanics.md` I6).
    pub fn occupancy_in_tile(&self, tile_index: usize) -> usize {
        self.lanes
            .iter()
            .map(|l| l.occupancy_in_tile(tile_index))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IRON: ItemId = ItemId(0);

    fn straight(tier: BeltTier, len: usize, end: RunEnd) -> BeltRun {
        BeltRun::new(tier, (0..len as i32).map(|x| (x, 0)).collect(), end)
    }

    /// The defining property: on a free-flowing lane, a hole punched in
    /// the middle travels downstream and never closes.
    #[test]
    fn gaps_do_not_heal_on_a_moving_lane() {
        let mut run = straight(BeltTier::Yellow, 6, RunEnd::Sink);
        // Fill the lane solid.
        for _ in 0..run.lanes[0].slot_count() {
            for _ in 0..8 {
                run.tick();
            }
            run.lanes[0].try_insert(IRON);
        }
        // Punch a hole in the middle by removing one item.
        let mut taken = Vec::new();
        run.lanes[0].take_from_tile(3, 1, &mut taken);
        assert_eq!(taken.len(), 1);
        let before = run.lanes[0].occupancy();

        // Advance without feeding. The hole must persist: occupancy falls
        // only because items leave at the sink, never because the gap closed.
        for _ in 0..8 {
            run.tick();
        }
        let exited = run.lanes[0].exited as usize;
        assert_eq!(
            run.lanes[0].occupancy(),
            before - exited,
            "items vanished or were created — the gap must simply travel"
        );
    }

    /// A compressed lane must move at full belt speed, not one item per
    /// step. This is what downstream-first ordering buys.
    #[test]
    fn compressed_lane_moves_at_full_speed() {
        let mut run = straight(BeltTier::Yellow, 4, RunEnd::Sink);
        for _ in 0..run.lanes[0].slot_count() {
            run.lanes[0].try_insert(IRON);
            // Advance exactly one slot (yellow = 8 ticks/slot).
            for _ in 0..8 {
                run.tick();
            }
        }
        // 16 slots at 8 ticks each: a solid lane should drain steadily.
        let start = run.lanes[0].exited;
        for _ in 0..80 {
            run.tick();
        }
        let moved = run.lanes[0].exited - start;
        assert!(
            moved >= 9,
            "compressed yellow lane should eject ~10 items in 80 ticks, got {moved}"
        );
    }

    /// A dead end backs up; a sink does not. #448 turns on this difference.
    #[test]
    fn dead_end_backs_up_and_refuses_further_input() {
        let mut run = straight(BeltTier::Yellow, 2, RunEnd::DeadEnd);
        let capacity = run.lanes[0].slot_count();
        let mut accepted = 0;
        for _ in 0..(capacity * 20) {
            if run.lanes[0].try_insert(IRON) {
                accepted += 1;
            }
            run.tick();
        }
        assert_eq!(
            accepted, capacity,
            "a dead-ended lane must accept exactly its slot count, then refuse"
        );
        assert_eq!(run.lanes[0].exited, 0, "a dead end must not eject items");
    }

    /// Faster tiers must actually move faster — the sub-slot accumulator
    /// exists so express (2.67 ticks/slot) is not quantised to red's 4.
    #[test]
    fn tiers_move_at_distinct_speeds() {
        let drained = |tier: BeltTier| {
            let mut run = straight(tier, 4, RunEnd::Sink);
            for _ in 0..600 {
                run.lanes[0].try_insert(IRON);
                run.tick();
            }
            run.lanes[0].exited
        };
        let (y, r, b) = (
            drained(BeltTier::Yellow),
            drained(BeltTier::Red),
            drained(BeltTier::Blue),
        );
        assert!(y < r && r < b, "expected yellow < red < blue, got {y} {r} {b}");
    }

    /// Throughput must *emerge* at the documented per-lane rate rather
    /// than being read from a table (B5: yellow 7.5/s per lane).
    #[test]
    fn saturated_lane_throughput_matches_mechanics_b5() {
        for (tier, want_per_lane) in [
            (BeltTier::Yellow, 7.5),
            (BeltTier::Red, 15.0),
            (BeltTier::Blue, 22.5),
        ] {
            let mut run = straight(tier, 8, RunEnd::Sink);
            // Warm up past the fill transient before measuring. Without
            // this the run under-reads by exactly its slot count (32),
            // which is the "buffer fill reads as convergence" artifact
            // class the sim harness already learned the hard way — see
            // docs/sim-harness-forensics.md.
            for _ in 0..1200 {
                run.lanes[0].try_insert(IRON);
                run.tick();
            }
            run.lanes[0].exited = 0;
            let ticks = 3600;
            for _ in 0..ticks {
                run.lanes[0].try_insert(IRON);
                run.tick();
            }
            let per_s = run.lanes[0].exited as f64 / (ticks as f64 / 60.0);
            assert!(
                (per_s - want_per_lane).abs() / want_per_lane < 0.02,
                "{tier:?}: derived {per_s:.2}/s per lane, mechanics B5 says {want_per_lane}"
            );
        }
    }

    #[test]
    fn take_from_tile_prefers_downstream_slots() {
        let mut run = straight(BeltTier::Yellow, 2, RunEnd::DeadEnd);
        for i in 0..SLOTS_PER_TILE {
            run.lanes[0].slots[i] = Some(ItemId(i as u16));
        }
        let mut out = Vec::new();
        run.lanes[0].take_from_tile(0, 2, &mut out);
        assert_eq!(out, vec![ItemId(3), ItemId(2)]);
    }
}
