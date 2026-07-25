//! Inserter state machine.
//!
//! The point of modelling inserters as a **state machine over ticks**
//! rather than as a throughput number is that density-dependent
//! throughput becomes *derived* rather than asserted.
//!
//! A real inserter's hand arrives at the pickup tile on a fixed cycle. If
//! nothing is under it, that swing is lost. If it grabs multi-item hands,
//! it can only take what is physically present. So an inserter above a
//! gappy belt moves less than one above a compressed belt — with no
//! `machine_feed_rate` table anywhere in sight. That is precisely what
//! the engine's static model cannot express, and it is why KC4 forbids
//! importing that table.

use crate::belt::ItemId;
use crate::entity_data::InserterKind;

/// Where an inserter picks from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupTarget {
    /// A tile of a belt run. Inserters pick from **both lanes**
    /// (`factorio-mechanics.md` I6).
    BeltTile { run: usize, tile: usize },
}

/// Where an inserter drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropTarget {
    /// A counting container. PR 1 has no machines; drops land in chests
    /// so per-position extraction can be measured directly.
    Chest(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Hand empty, waiting for something to appear under the pickup tile.
    /// Every tick spent here is a lost swing — the starvation signal.
    WaitingForSource,
    /// Mid-cycle: carrying to the drop side, then returning empty.
    ///
    /// One phase rather than two, and one **fractional** timer rather than
    /// two rounded halves. Both details are load-bearing: an earlier draft
    /// used `Swinging`/`Returning` with `round()`ed half-cycles and lost a
    /// tick to the grab, which under-credited a fast inserter by ~8%
    /// (2.222/s against I8's 2.4). A model that mis-times inserters cannot
    /// measure a defect *about* inserter timing.
    Cycling,
}

#[derive(Debug, Clone)]
pub struct Inserter {
    pub kind: InserterKind,
    pub pickup: PickupTarget,
    pub drop: DropTarget,
    /// Declared inserter-capacity research level (0–7), a user-facing
    /// engine axis. The meter takes it as an input, never infers it.
    pub capacity_level: u8,
    phase: Phase,
    /// Ticks remaining in the current cycle. Fractional on purpose —
    /// `cycle_ticks` is 25 for fast, 71.43 for regular; rounding it
    /// quantises throughput.
    cycle_timer: f64,
    /// Whether this cycle's drop has already happened (at the halfway
    /// crossing).
    dropped: bool,
    hand: Vec<ItemId>,
    /// Ticks spent with an empty hand waiting for items.
    pub starved_ticks: u64,
    /// Items delivered over the run.
    pub delivered: u64,
    /// Completed swings, whether or not the hand was full.
    pub swings: u64,
    /// Items that could have been carried but were not, because the belt
    /// did not have a full hand's worth under the pickup tile. This is the
    /// density penalty, made countable.
    pub short_hand_items: u64,
}

impl Inserter {
    pub fn new(
        kind: InserterKind,
        pickup: PickupTarget,
        drop: DropTarget,
        capacity_level: u8,
    ) -> Self {
        Inserter {
            kind,
            pickup,
            drop,
            capacity_level,
            phase: Phase::WaitingForSource,
            cycle_timer: 0.0,
            dropped: false,
            hand: Vec::new(),
            starved_ticks: 0,
            delivered: 0,
            swings: 0,
            short_hand_items: 0,
        }
    }

    pub fn hand_size(&self) -> u32 {
        self.kind.hand_size(self.capacity_level)
    }

    /// Full pick→drop→return cycle, in ticks. Never rounded.
    fn cycle_ticks(&self) -> f64 {
        self.kind.cycle_ticks()
    }

    pub fn is_waiting_for_source(&self) -> bool {
        matches!(self.phase, Phase::WaitingForSource)
    }

    /// Effective throughput over a measured window, items/s.
    pub fn rate_per_second(&self, ticks: u64) -> f64 {
        if ticks == 0 {
            return 0.0;
        }
        self.delivered as f64 / (ticks as f64 / 60.0)
    }

    /// Called each tick with the items the pickup tile could supply.
    ///
    /// `grab` is invoked only at the moment the hand is actually over the
    /// source, which is what makes lost swings possible.
    pub fn tick<G, D>(&mut self, mut grab: G, mut deposit: D)
    where
        G: FnMut(u32, &mut Vec<ItemId>),
        D: FnMut(&mut Vec<ItemId>) -> bool,
    {
        if self.phase == Phase::WaitingForSource {
            let want = self.hand_size();
            grab(want, &mut self.hand);
            if self.hand.is_empty() {
                // Nothing under the pickup tile this tick. A real inserter
                // simply waits here; the swing is lost. This is the
                // starvation signal, and it is why an inserter above a
                // gappy belt moves less than one above a compressed belt
                // without any rate table saying so.
                self.starved_ticks += 1;
                return;
            }
            // Took a partial hand because the belt was not dense enough to
            // fill it. Count the shortfall — the density penalty the
            // engine's flat rate cannot express.
            self.short_hand_items += (want as u64).saturating_sub(self.hand.len() as u64);
            self.cycle_timer = self.cycle_ticks();
            self.dropped = false;
            self.phase = Phase::Cycling;
            // Fall through: the grab tick is part of the cycle, not extra.
        }

        // Consume one tick of the cycle.
        self.cycle_timer -= 1.0;

        // The drop happens at the halfway crossing (pickup → drop is half
        // a turn), the return over the remaining half.
        if !self.dropped && self.cycle_timer <= self.cycle_ticks() / 2.0 {
            if deposit(&mut self.hand) {
                self.delivered += self.hand.len() as u64;
                self.hand.clear();
                self.swings += 1;
                self.dropped = true;
            } else {
                // Destination full: hold position. Give back the tick so a
                // blocked inserter does not silently complete its cycle.
                self.cycle_timer += 1.0;
                return;
            }
        }

        if self.cycle_timer <= 0.0 {
            self.phase = Phase::WaitingForSource;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IRON: ItemId = ItemId(0);

    fn run_saturated(kind: InserterKind, level: u8, ticks: u64) -> Inserter {
        let mut ins = Inserter::new(
            kind,
            PickupTarget::BeltTile { run: 0, tile: 0 },
            DropTarget::Chest(0),
            level,
        );
        for _ in 0..ticks {
            ins.tick(
                |want, hand| {
                    for _ in 0..want {
                        hand.push(IRON);
                    }
                },
                |_hand| true,
            );
        }
        ins
    }

    /// With an infinitely dense source the meter must *derive* the
    /// mechanics doc's I8 throughputs. Chest-to-chest, hand 1.
    #[test]
    fn saturated_throughput_matches_mechanics_i8() {
        for (kind, want) in [
            (InserterKind::Regular, 0.84),
            (InserterKind::LongHanded, 1.20),
            (InserterKind::Fast, 2.40),
        ] {
            let ticks = 36_000;
            let ins = run_saturated(kind, 0, ticks);
            let got = ins.rate_per_second(ticks);
            assert!(
                (got - want).abs() / want < 0.05,
                "{kind:?}: derived {got:.3}/s, I8 says {want}"
            );
        }
    }

    /// A stack inserter's advantage is hand size, not swing speed.
    #[test]
    fn stack_inserter_moves_a_full_hand_per_swing() {
        let ticks = 36_000;
        let ins = run_saturated(InserterKind::Stack, 0, ticks);
        let fast = run_saturated(InserterKind::Fast, 0, ticks);
        assert_eq!(ins.hand_size(), 6, "stack base hand is 6 (game-true)");
        // Same rotation speed, 6× the hand.
        assert!(
            (ins.rate_per_second(ticks) / fast.rate_per_second(ticks) - 6.0).abs() < 0.3,
            "stack should be ~6x fast at equal rotation speed"
        );
    }

    /// The headline behaviour: a source that cannot fill the hand yields
    /// proportionally less, and the shortfall is *counted* rather than
    /// silently absorbed.
    #[test]
    fn partial_hands_reduce_throughput_and_are_counted() {
        let ticks = 18_000;
        let mut ins = Inserter::new(
            InserterKind::Stack,
            PickupTarget::BeltTile { run: 0, tile: 0 },
            DropTarget::Chest(0),
            0,
        );
        // Source can only ever supply 2 items per grab.
        for _ in 0..ticks {
            ins.tick(
                |want, hand| {
                    for _ in 0..want.min(2) {
                        hand.push(IRON);
                    }
                },
                |_hand| true,
            );
        }
        let full = run_saturated(InserterKind::Stack, 0, ticks);
        assert!(
            ins.rate_per_second(ticks) < full.rate_per_second(ticks) * 0.5,
            "a 2-of-6 hand must cost throughput"
        );
        assert!(
            ins.short_hand_items > 0,
            "the shortfall must be observable, not absorbed"
        );
    }

    /// An empty source produces lost swings, not a slower cycle.
    #[test]
    fn empty_source_starves_rather_than_slowing() {
        let ticks = 600;
        let mut ins = Inserter::new(
            InserterKind::Fast,
            PickupTarget::BeltTile { run: 0, tile: 0 },
            DropTarget::Chest(0),
            0,
        );
        for _ in 0..ticks {
            ins.tick(|_want, _hand| {}, |_hand| true);
        }
        assert_eq!(ins.delivered, 0);
        assert_eq!(ins.starved_ticks, ticks);
        assert!(ins.is_waiting_for_source());
    }

    /// Capacity research raises the hand for bulk-class inserters and
    /// barely moves the others (`factorio-mechanics.md` I8b).
    #[test]
    fn capacity_research_scales_bulk_class_only() {
        assert_eq!(InserterKind::Stack.hand_size(0), 6);
        assert_eq!(InserterKind::Stack.hand_size(2), 9);
        assert_eq!(InserterKind::Stack.hand_size(7), 16);
        assert_eq!(InserterKind::Bulk.hand_size(0), 2);
        assert_eq!(InserterKind::Bulk.hand_size(7), 12);
        // Non-bulk barely moves: 1 -> 3 across the whole ladder.
        assert_eq!(InserterKind::Fast.hand_size(0), 1);
        assert_eq!(InserterKind::Fast.hand_size(7), 3);
    }
}
