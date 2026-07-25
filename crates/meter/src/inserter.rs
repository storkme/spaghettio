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
    /// `deposit` inserts as much of the hand as fits, draining what it
    /// took, and returns the count accepted. A partial return means the
    /// inserter keeps the remainder and retries — which is what the game
    /// does, and is not the same as being blocked.
    pub fn tick<G, D>(&mut self, mut grab: G, mut deposit: D)
    where
        G: FnMut(u32, &mut Vec<ItemId>),
        D: FnMut(&mut Vec<ItemId>) -> usize,
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
            // ACCUMULATE, never assign. The previous cycle ended with the
            // timer slightly below zero; that overshoot is the fractional
            // part of the true period and must carry forward. A hard
            // assignment quantises the period up to `ceil(cycle_ticks)` —
            // for a regular inserter, 72 ticks instead of 71.43, i.e.
            // 0.8333/s against I8's 0.84/s. Same discipline as
            // `Lane::tick`, `Source::tick` and `Chest::tick`.
            self.cycle_timer += self.cycle_ticks();
            self.dropped = false;
            self.phase = Phase::Cycling;
            // Fall through: the grab tick is part of the cycle, not extra.
        }

        // Consume one tick of the cycle.
        self.cycle_timer -= 1.0;

        // The drop happens at the halfway crossing (pickup → drop is half
        // a turn), the return over the remaining half.
        if !self.dropped && self.cycle_timer <= self.cycle_ticks() / 2.0 {
            self.delivered += deposit(&mut self.hand) as u64;
            if self.hand.is_empty() {
                self.swings += 1;
                self.dropped = true;
            } else {
                // Nothing fit, or only part of the hand did. Hold position
                // with the remainder and retry next tick — the game's
                // partial-insert behaviour. Give back the tick so a
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
                |hand| { let n = hand.len(); hand.clear(); n },
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
            // 1%, not 5%. A 5% band hid a real defect: the cycle timer was
            // hard-assigned rather than accumulated, quantising a regular
            // inserter's period from 71.43 to 72 ticks (0.8333/s vs 0.84,
            // −0.79%) — invisible at 5%, caught by review instead of by
            // the test that exists to catch it.
            assert!(
                (got - want).abs() / want < 0.01,
                "{kind:?}: derived {got:.4}/s, I8 says {want}"
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
                |hand| { let n = hand.len(); hand.clear(); n },
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
            ins.tick(|_want, _hand| {}, |hand| { let n = hand.len(); hand.clear(); n });
        }
        assert_eq!(ins.delivered, 0);
        assert_eq!(ins.starved_ticks, ticks);
        assert!(ins.is_waiting_for_source());
    }

    /// The **full** research ladder, pinned level by level against
    /// `factorio-mechanics.md` I8b.
    ///
    /// Deliberately exhaustive rather than endpoint-sampled. An earlier
    /// version asserted only L0/L2/L7 and happened to pick endpoints that
    /// were right, so it pinned a mis-transcribed middle instead of
    /// catching it (review, PR #458): bulk read `2,4,5,6,7,9,11,12`
    /// against I8b's `2,3,4,5,6,8,10,12`, and non-bulk `1,1,1,2,3,3,3,3`
    /// against `1,1,2,2,2,2,2,3`. Sampling a table is not testing it.
    #[test]
    fn hand_size_ladder_matches_mechanics_i8b() {
        let ladder = |k: InserterKind| (0..=7).map(|l| k.hand_size(l)).collect::<Vec<_>>();

        // bulk chain: +1 at L1-4, +2 at L5-7
        assert_eq!(ladder(InserterKind::Bulk), vec![2, 3, 4, 5, 6, 8, 10, 12]);
        // stack = bulk + 4 built-in
        assert_eq!(ladder(InserterKind::Stack), vec![6, 7, 8, 9, 10, 12, 14, 16]);
        // non-bulk: +1 at L2 and L7 only
        for k in [
            InserterKind::Regular,
            InserterKind::LongHanded,
            InserterKind::Fast,
        ] {
            assert_eq!(ladder(k), vec![1, 1, 2, 2, 2, 2, 2, 3], "{k:?}");
        }
    }
}
