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

use crate::entity_data::{BeltTier, ITEM_SPACING_TILES, SLOTS_PER_TILE};

/// Width of the continuous inserter collision window, in item spacings.
/// One item occupies one spacing, so the valid center-to-center tolerance is
/// half a spacing.
pub(crate) const DROP_COLLISION_WINDOW_SLOTS: f64 = 0.5;

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
    /// Fractional position carried by each item relative to the lane's slot
    /// grid.  A normal belt transfer preserves this value across tile
    /// boundaries; an inserter drop uses it to retain the part of the
    /// requested continuous position that did not land exactly on a slot.
    offsets: Vec<f64>,
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
            offsets: vec![0.0; slot_count],
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
                self.offsets[0] = 0.0;
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
                self.offsets[idx] = 0.0;
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

    /// Read-only view of the raw slots, for diagnostics.
    pub fn slots_debug(&self) -> &[Option<ItemId>] {
        &self.slots
    }

    /// Fractional residual for a slot, used by connected-line probes.
    pub fn slot_offset(&self, index: usize) -> f64 {
        self.offsets.get(index).copied().unwrap_or(0.0)
    }

    /// The exit slot's contents, if any (the downstream end).
    pub fn peek_exit(&self) -> Option<ItemId> {
        self.slots.last().copied().flatten()
    }

    /// Remove and return the exit slot's item.
    pub fn take_exit(&mut self) -> Option<ItemId> {
        self.take_exit_with_offset().map(|(item, _)| item)
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
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(item);
                self.offsets[idx] = 0.0;
                return true;
            }
        }
        false
    }

    /// Insert an item at a continuous position within this tile.
    ///
    /// The tile lanes are intentionally still represented as four slots, but
    /// the slot chosen for an inserter drop depends on the belt's fractional
    /// progress. `progress_slots` is the fraction of one item spacing already
    /// travelled since the last whole-slot shift. A slot is therefore at
    /// `slot + progress_slots` in spacing units, and the nearest free slot is
    /// the discrete projection of the requested position.
    ///
    /// The half-spacing bound is the collision window: a free slot farther
    /// from the requested point is not a valid approximation of a real
    /// continuous insertion and the drop remains blocked. This is deliberately
    /// separate from [`Lane::try_insert_anywhere`], which remains for
    /// boundary/legacy insertion paths that have no drop coordinate.
    pub fn try_insert_at(
        &mut self,
        local_position: f64,
        progress_slots: f64,
        item: ItemId,
    ) -> bool {
        self.try_insert_at_segment_mode(local_position, progress_slots, 0.0, item, false, false)
    }

    /// Insert at a local position whose lane has a continuous segment base.
    /// The base affects diagnostics and the caller's connected-line
    /// collision query; the residual stored on this tile remains local.
    pub fn try_insert_at_segment(
        &mut self,
        local_position: f64,
        progress_slots: f64,
        segment_base: f64,
        item: ItemId,
    ) -> bool {
        self.try_insert_at_segment_mode(
            local_position,
            progress_slots,
            segment_base,
            item,
            false,
            false,
        )
    }

    /// Insert at a turn-to-merge boundary where the engine may finish a
    /// compressed target lane by using its upstream gap. This is deliberately
    /// opt-in; ordinary inserter drops and sideloads retain the stricter
    /// collision rule above.
    pub fn try_insert_at_turn_merge(
        &mut self,
        local_position: f64,
        progress_slots: f64,
        segment_base: f64,
        item: ItemId,
    ) -> bool {
        self.try_insert_at_segment_mode(local_position, progress_slots, segment_base, item, true, false)
    }

    /// Insert at the inner turn lane's merge boundary. The inner path's
    /// measured admission does not use the upstream-gap completion that the
    /// outer path does, so a contiguous downstream block remains blocking.
    pub fn try_insert_at_inner_turn_merge(
        &mut self,
        local_position: f64,
        progress_slots: f64,
        segment_base: f64,
        item: ItemId,
    ) -> bool {
        self.try_insert_at_segment_mode(local_position, progress_slots, segment_base, item, false, true)
    }

    fn try_insert_at_segment_mode(
        &mut self,
        local_position: f64,
        progress_slots: f64,
        segment_base: f64,
        item: ItemId,
        allow_turn_merge_pack: bool,
        reject_inner_turn_pack: bool,
    ) -> bool {
        if self.slots.is_empty() || !local_position.is_finite() || !progress_slots.is_finite() {
            return false;
        }

        let phase = progress_slots.clamp(0.0, 1.0);
        let position =
            segment_base + local_position.clamp(0.0, 1.0 - f64::EPSILON) / ITEM_SPACING_TILES;

        // Factorio does not treat the collision window as a static circle.
        // If one item is near the requested point, the engine may enlarge
        // the one-sided gap by nudging that item (and any contiguous items
        // behind it) away from the insertion point.  It will not perform
        // that rearrangement when both sides are occupied, or when an item
        // is exactly at the requested point.  Count the nearby occupants
        // before choosing a free slot so the discrete model preserves that
        // distinction.
        let nearby: Vec<(usize, f64)> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| {
                slot.as_ref().map(|_| {
                    (
                        idx,
                        (segment_base + idx as f64 + phase + self.offsets[idx] - position).abs(),
                    )
                })
            })
            .filter(|(_, distance)| *distance <= DROP_COLLISION_WINDOW_SLOTS + f64::EPSILON)
            .collect();
        if nearby.len() > 1 {
            return false;
        }

        let mut best: Option<(usize, f64)> = None;
        for (idx, slot) in self.slots.iter().enumerate() {
            if slot.is_some() {
                continue;
            }
            let distance = ((segment_base + idx as f64 + phase) - position).abs();
            if distance > DROP_COLLISION_WINDOW_SLOTS + f64::EPSILON {
                continue;
            }
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((idx, distance));
            }
        }

        if let Some((idx, _)) = best {
            self.slots[idx] = Some(item);
            // Preserve the residual between the requested continuous position
            // and the selected slot.  On a subsequent whole-slot shift this
            // residual moves with the item, including when it crosses a bend or
            // sideload into another tile.
            self.offsets[idx] = position - (segment_base + idx as f64 + phase);
            return true;
        }

        // No free slot is close enough.  A single non-exact nearby item can
        // still be admitted by shifting the contiguous block on the side of
        // the gap.  This is the discrete equivalent of the engine's gap
        // enlargement; an exact occupant has no gap to enlarge.
        let Some(&(occupied_idx, distance)) = nearby.first() else {
            return false;
        };
        if distance <= f64::EPSILON {
            return false;
        }
        let occupied_position =
            segment_base + occupied_idx as f64 + phase + self.offsets[occupied_idx];
        if reject_inner_turn_pack
            && occupied_idx > 0
            && self.slots[..occupied_idx].iter().any(Option::is_none)
            && self.slots[occupied_idx..].iter().all(Option::is_some)
        {
            return false;
        }
        if occupied_position < position {
            let Some(free_idx) = (0..occupied_idx)
                .rev()
                .find(|&idx| self.slots[idx].is_none())
            else {
                return false;
            };
            for idx in free_idx..occupied_idx {
                self.slots[idx] = self.slots[idx + 1].take();
                self.offsets[idx] = self.offsets[idx + 1];
                self.offsets[idx + 1] = 0.0;
            }
            self.slots[occupied_idx] = Some(item);
            self.offsets[occupied_idx] = position - (segment_base + occupied_idx as f64 + phase);
            return true;
        }

        let Some(free_idx) =
            ((occupied_idx + 1)..self.slots.len()).find(|&idx| self.slots[idx].is_none())
        else {
            if allow_turn_merge_pack
                && occupied_idx > 0
                && self.slots[..occupied_idx].iter().any(Option::is_none)
                && self.slots[occupied_idx..].iter().all(Option::is_some)
            {
                let free_idx = (0..occupied_idx)
                    .rev()
                    .find(|&idx| self.slots[idx].is_none())
                    .expect("checked for an upstream gap");
                self.slots[free_idx] = Some(item);
                self.offsets[free_idx] = position - (segment_base + free_idx as f64 + phase);
                return true;
            }
            return false;
        };
        for idx in ((occupied_idx + 1)..=free_idx).rev() {
            self.slots[idx] = self.slots[idx - 1].take();
            self.offsets[idx] = self.offsets[idx - 1];
            self.offsets[idx - 1] = 0.0;
        }
        self.slots[occupied_idx] = Some(item);
        self.offsets[occupied_idx] = position - (segment_base + occupied_idx as f64 + phase);
        true
    }

    /// Take up to `max` items the predicate accepts, most-downstream
    /// first. Items it rejects are left in place — an inserter that will
    /// not take an item does not disturb it (mechanics I11).
    pub fn take_matching<F>(&mut self, max: u32, accept: &mut F, out: &mut Vec<ItemId>)
    where
        F: FnMut(ItemId) -> bool,
    {
        let mut taken = 0u32;
        for idx in (0..self.slots.len()).rev() {
            if taken >= max {
                break;
            }
            let Some(item) = self.slots[idx] else {
                continue;
            };
            if !accept(item) {
                continue;
            }
            self.slots[idx] = None;
            self.offsets[idx] = 0.0;
            out.push(item);
            taken += 1;
        }
    }

    /// Take up to `max` items, most-downstream first.
    pub fn take_all(&mut self, max: u32, out: &mut Vec<ItemId>) {
        let mut taken = 0u32;
        for idx in (0..self.slots.len()).rev() {
            if taken >= max {
                break;
            }
            if let Some(item) = self.slots[idx].take() {
                self.offsets[idx] = 0.0;
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
                self.offsets[idx] = self.offsets[idx - 1];
                self.offsets[idx - 1] = 0.0;
            }
        }
    }

    /// Read the exit item and its continuous residual without removing it.
    pub fn peek_exit_with_offset(&self) -> Option<(ItemId, f64)> {
        let idx = self.slots.len().checked_sub(1)?;
        self.slots[idx].map(|item| (item, self.offsets[idx]))
    }

    /// Remove the exit item and preserve its continuous residual for a
    /// downstream tile handoff.
    pub fn take_exit_with_offset(&mut self) -> Option<(ItemId, f64)> {
        let idx = self.slots.len().checked_sub(1)?;
        let item = self.slots[idx].take()?;
        let offset = self.offsets[idx];
        self.offsets[idx] = 0.0;
        Some((item, offset))
    }

    /// Insert an item at the entry while retaining a residual from the
    /// preceding tile's continuous position.
    pub fn try_insert_entry_with_offset(&mut self, item: ItemId, offset: f64) -> bool {
        match self.slots.first_mut() {
            Some(slot @ None) => {
                *slot = Some(item);
                self.offsets[0] = offset;
                true
            }
            _ => false,
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
            self.take_exit_with_offset();
            self.exited += 1;
        }

        // 2. Shift the rest forward into any free slot ahead.
        self.shift_forward();
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
        assert!(
            y < r && r < b,
            "expected yellow < red < blue, got {y} {r} {b}"
        );
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

    #[test]
    fn continuous_drop_tracks_fractional_belt_progress() {
        let mut lane = Lane::new(SLOTS_PER_TILE);

        // At 0.8 of a slot into the tile, local .5 is closest to slot 1,
        // not the static midpoint slot 2.
        assert!(lane.try_insert_at(0.5, 0.8, IRON));
        assert!(lane.slots[1].is_some());

        let mut lane = Lane::new(SLOTS_PER_TILE);
        // An exact occupant still blocks the drop; gap enlargement does not
        // move an item that is already at the requested point.
        lane.slots[1] = Some(IRON);
        lane.slots[2] = Some(IRON);
        assert!(!lane.try_insert_at(0.7, 0.8, ItemId(2)));
    }

    #[test]
    fn continuous_drop_enlarges_one_sided_gap() {
        let mut lane = Lane::new(SLOTS_PER_TILE);
        // The engine admits a drop at .5 when the only nearby item is just
        // to its left: that item is nudged upstream to make room.
        assert!(lane.try_insert_at(0.4375, 0.0, IRON));
        assert!(lane.try_insert_at(0.5, 0.0, ItemId(2)));
        assert_eq!(lane.occupancy(), 2);
        assert!(lane.slots[1].is_some());
        assert!(lane.slots[2].is_some());
    }

    #[test]
    fn continuous_drop_rejects_exact_or_two_sided_occupancy() {
        let mut exact = Lane::new(SLOTS_PER_TILE);
        assert!(exact.try_insert_at(0.5, 0.0, IRON));
        assert!(!exact.try_insert_at(0.5, 0.0, ItemId(2)));

        let mut bracketed = Lane::new(SLOTS_PER_TILE);
        assert!(bracketed.try_insert_at(0.375, 0.0, IRON));
        assert!(bracketed.try_insert_at(0.625, 0.0, IRON));
        assert!(!bracketed.try_insert_at(0.5, 0.0, ItemId(2)));
    }

    #[test]
    fn turn_merge_drop_can_finish_an_upstream_gap() {
        let mut lane = Lane::new(SLOTS_PER_TILE);
        lane.slots[1] = Some(IRON);
        lane.slots[2] = Some(IRON);
        lane.slots[3] = Some(IRON);

        // A turn-to-merge handoff can fill the remaining upstream gap once
        // the downstream block is contiguous; an ordinary drop must remain
        // strict when the nearby item sits downstream of the requested point.
        assert!(!lane.try_insert_at(0.15, 0.0, ItemId(2)));
        assert!(lane.try_insert_at_turn_merge(
            0.15,
            0.0,
            0.0,
            ItemId(2)
        ));
        assert_eq!(lane.occupancy(), 4);
        assert_eq!(lane.slots[0], Some(ItemId(2)));
    }

    #[test]
    fn inner_turn_merge_keeps_a_contiguous_block_blocked() {
        let mut lane = Lane::new(SLOTS_PER_TILE);
        lane.slots[1] = Some(IRON);
        lane.slots[2] = Some(IRON);
        lane.slots[3] = Some(IRON);

        assert!(!lane.try_insert_at_inner_turn_merge(0.15, 0.0, 0.0, ItemId(2)));
        assert_eq!(lane.occupancy(), 3);
    }

    #[test]
    fn fractional_drop_residual_moves_with_the_item() {
        let mut lane = Lane::new(SLOTS_PER_TILE);
        assert!(lane.try_insert_at(0.5, 0.8, IRON));
        let residual = lane.offsets[1];
        assert!((residual - 0.2).abs() < 1e-9);

        // The item remains at the same continuous phase when the discrete
        // grid advances: it changes slot, but not its residual position.
        lane.shift_forward();
        assert!(lane.slots[2].is_some());
        assert!((lane.offsets[2] - residual).abs() < 1e-9);
    }

    #[test]
    fn whole_step_clears_exit_offset_and_moves_residual() {
        let mut lane = Lane::new(SLOTS_PER_TILE);
        lane.slots[0] = Some(IRON);
        lane.offsets[0] = 0.25;
        lane.step(RunEnd::DeadEnd);
        assert_eq!(lane.slots[1], Some(IRON));
        assert!((lane.offsets[1] - 0.25).abs() < 1e-9);
        assert_eq!(lane.offsets[0], 0.0);

        let last = lane.slots.len() - 1;
        lane.slots[last] = Some(ItemId(2));
        lane.offsets[last] = -0.125;
        lane.step(RunEnd::Sink);
        assert_eq!(lane.exited, 1);
        assert_eq!(lane.slots[last], None);
        assert_eq!(lane.offsets[last], 0.0);
    }
}
