//! The simulated world: belt runs, inserters, containers, boundary sources.
//!
//! PR 1 carries the physics core only — belts, inserters, chests. Machines,
//! blueprint ingestion, convergence detection and `MeterReport` land with
//! PR 3 (see the RFC's phasing). What is here is exactly what PR 2's margin
//! sweep needs, and it is deliberately the smallest thing that can falsify
//! the RFC's central premise.

use rustc_hash::FxHashMap;

use crate::belt::{BeltRun, ItemId, RunEnd};
use crate::entity_data::{BeltTier, InserterKind};
use crate::inserter::{DropTarget, Inserter, PickupTarget};

/// Interns item names so the hot loop never touches strings.
#[derive(Debug, Default, Clone)]
pub struct ItemInterner {
    names: Vec<String>,
    ids: FxHashMap<String, u16>,
}

impl ItemInterner {
    pub fn intern(&mut self, name: &str) -> ItemId {
        if let Some(&id) = self.ids.get(name) {
            return ItemId(id);
        }
        let id = self.names.len() as u16;
        self.names.push(name.to_string());
        self.ids.insert(name.to_string(), id);
        ItemId(id)
    }

    pub fn name(&self, id: ItemId) -> &str {
        &self.names[id.0 as usize]
    }
}

/// A container with bounded intake — a machine's ingredient side, minus
/// the crafting.
///
/// **The bound is not optional decoration.** An unbounded sink lets the
/// head consumer of a row pull at its inserter's full swing rate forever
/// (measured: 16.2/s against a 7.5/s demand), which both overstates head
/// hogging and makes added margin *worse*, since the head simply hogs
/// more. Real machines fill an ingredient buffer to a limit and then draw
/// only at their consumption rate, which is what lets surplus reach the
/// tail. #448's dumps show exactly this: head machines holding 42/34/20
/// items — buffered to a cap, not draining without limit.
///
/// Full machines (craft timers, recipes, outputs) land in PR 3; this is
/// the smallest faithful stand-in for the input side.
#[derive(Debug, Default, Clone)]
pub struct Chest {
    pub received: u64,
    pub by_item: FxHashMap<u16, u64>,
    /// Buffer limit. `None` = infinite sink (a drain, or the world edge).
    pub capacity: Option<u32>,
    /// Steady consumption, items/s. `0.0` = never drains.
    pub demand_per_second: f64,
    pub buffer: u32,
    drain_accumulator: f64,
    /// Ticks the consumer wanted an item and had none — the starvation
    /// signal that maps onto the harness's `item_ingredient_shortage`.
    pub starved_ticks: u64,
    pub consumed: u64,
}

impl Chest {
    /// A bounded consumer: buffers up to `capacity`, draws `demand` items/s.
    pub fn consumer(capacity: u32, demand_per_second: f64) -> Self {
        Chest {
            capacity: Some(capacity),
            demand_per_second,
            ..Default::default()
        }
    }

    /// Insert as much of `hand` as fits, draining what was taken and
    /// leaving the remainder. Returns the number accepted.
    ///
    /// **Partial insert is the game's behaviour, and modelling it
    /// all-or-nothing was a real defect** (review, PR #458). A Factorio
    /// inserter transfers as many items as the destination accepts and
    /// keeps the rest in hand, stalling fully only when *nothing* fits.
    /// Rejecting a whole hand whenever it doesn't entirely fit made
    /// consumers block and release in whole-hand quanta once `buffer` came
    /// within `hand_size` of `capacity` — a quantised cadence capable of
    /// beating against the source period, i.e. a candidate cause of the
    /// non-monotonic margin behaviour this crate reports.
    fn accept(&mut self, hand: &mut Vec<ItemId>) -> usize {
        let space = match self.capacity {
            Some(cap) => (cap.saturating_sub(self.buffer)) as usize,
            None => hand.len(),
        };
        let n = space.min(hand.len());
        if n == 0 {
            return 0;
        }
        for it in hand.drain(..n) {
            *self.by_item.entry(it.0).or_insert(0) += 1;
        }
        self.buffer += n as u32;
        self.received += n as u64;
        n
    }

    fn tick(&mut self) {
        if self.demand_per_second <= 0.0 {
            return;
        }
        self.drain_accumulator += self.demand_per_second / 60.0;
        while self.drain_accumulator >= 1.0 {
            self.drain_accumulator -= 1.0;
            if self.buffer > 0 {
                self.buffer -= 1;
                self.consumed += 1;
            } else {
                self.starved_ticks += 1;
            }
        }
    }

    /// Items/s actually consumed over `ticks` — the delivered rate that
    /// matters, as opposed to what an inserter managed to pick up.
    pub fn consumption_rate(&self, ticks: u64) -> f64 {
        if ticks == 0 {
            return 0.0;
        }
        self.consumed as f64 / (ticks as f64 / 60.0)
    }
}

/// A boundary feed: injects one item type onto one lane at a fixed rate.
///
/// Rate is the *offered* rate. What actually lands is limited by whether
/// slot 0 is free — which is how a saturated belt makes its producer go
/// `full_output`, and is therefore a measured output, not an assumption.
#[derive(Debug, Clone)]
pub struct Source {
    pub run: usize,
    pub lane: usize,
    pub item: ItemId,
    pub rate_per_second: f64,
    accumulator: f64,
    /// Items the source wanted to inject but could not, because the lane
    /// was backed up to its head.
    pub rejected: u64,
    pub injected: u64,
}

impl Source {
    pub fn new(run: usize, lane: usize, item: ItemId, rate_per_second: f64) -> Self {
        Source {
            run,
            lane,
            item,
            rate_per_second,
            accumulator: 0.0,
            rejected: 0,
            injected: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct World {
    pub runs: Vec<BeltRun>,
    pub chests: Vec<Chest>,
    pub inserters: Vec<Inserter>,
    pub sources: Vec<Source>,
    pub items: ItemInterner,
    pub ticks: u64,
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    pub fn add_run(&mut self, run: BeltRun) -> usize {
        self.runs.push(run);
        self.runs.len() - 1
    }

    /// An unbounded sink (a drain, or the world edge).
    pub fn add_chest(&mut self) -> usize {
        self.chests.push(Chest::default());
        self.chests.len() - 1
    }

    /// A bounded consumer — see [`Chest::consumer`].
    pub fn add_consumer(&mut self, capacity: u32, demand_per_second: f64) -> usize {
        self.chests.push(Chest::consumer(capacity, demand_per_second));
        self.chests.len() - 1
    }

    pub fn add_inserter(&mut self, ins: Inserter) -> usize {
        self.inserters.push(ins);
        self.inserters.len() - 1
    }

    pub fn add_source(&mut self, src: Source) -> usize {
        self.sources.push(src);
        self.sources.len() - 1
    }

    /// Advance one tick.
    ///
    /// **Update order: sources → inserters → belts.** Stated because it is
    /// an approximation of Factorio's own entity update order and is
    /// therefore a candidate divergence. Inserters are given their chance
    /// to grab *before* the belt advances, so an item that arrives under a
    /// pickup tile this tick can be taken this tick rather than sliding
    /// past. Any measured disagreement traceable to this ordering belongs
    /// in `docs/meter-divergence.md`.
    pub fn tick(&mut self) {
        self.tick_sources();
        self.tick_inserters();
        for run in &mut self.runs {
            run.tick();
        }
        for chest in &mut self.chests {
            chest.tick();
        }
        self.ticks += 1;
    }

    pub fn run_for(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    fn tick_sources(&mut self) {
        for src in &mut self.sources {
            src.accumulator += src.rate_per_second / 60.0;
            while src.accumulator >= 1.0 {
                src.accumulator -= 1.0;
                let lane = &mut self.runs[src.run].lanes[src.lane];
                if lane.try_insert(src.item) {
                    src.injected += 1;
                } else {
                    // Belt backed up to its head: the offered item is
                    // refused. Upstream, this is a producer going
                    // `full_output`.
                    src.rejected += 1;
                }
            }
        }
    }

    fn tick_inserters(&mut self) {
        // Take the vec so the closures can borrow runs/chests disjointly.
        let mut inserters = std::mem::take(&mut self.inserters);
        for ins in &mut inserters {
            let runs = &mut self.runs;
            let chests = &mut self.chests;
            let pickup = ins.pickup;
            let drop = ins.drop;
            ins.tick(
                |want, hand| match pickup {
                    PickupTarget::BeltTile { run, tile } => {
                        let run = &mut runs[run];
                        // Both lanes (factorio-mechanics.md I6).
                        let mut remaining = want;
                        for lane in run.lanes.iter_mut() {
                            if remaining == 0 {
                                break;
                            }
                            let before = hand.len();
                            lane.take_from_tile(tile, remaining, hand);
                            remaining -= (hand.len() - before) as u32;
                        }
                    }
                },
                |hand| match drop {
                    DropTarget::Chest(idx) => chests[idx].accept(hand),
                },
            );
        }
        self.inserters = inserters;
    }

    /// Items/s delivered by one inserter over the run so far.
    pub fn inserter_rate(&self, idx: usize) -> f64 {
        self.inserters[idx].rate_per_second(self.ticks)
    }
}

/// The #448 fixture: N consumers along one shared input belt.
///
/// A producer feeds the head of a dead-ended belt at `supply_per_second`;
/// `consumers` inserters sit along it, one per tile, each pulling into its
/// own chest. This is the exact shape of a bus row's input belt, and the
/// smallest configuration in which head-buffers-starve-tail can occur.
///
/// `margin` is the ratio of supplied rate to aggregate demand: **1.0 is
/// the zero-margin case #448 measured failing**, and the axis PR 2 sweeps
/// to replace that check's admitted lower bound with a measured number.
///
/// Supply and demand are separate parameters on purpose. Deriving demand
/// from supply would make margin inexpressible — the bug that made the
/// first draft of `margin_resolves_the_starvation` unsatisfiable.
pub struct RowFixture {
    pub world: World,
    pub run: usize,
    /// Inserter index per consumer position, head first.
    pub consumers: Vec<usize>,
    /// Chest index per consumer position, head first.
    pub chests: Vec<usize>,
    pub item: ItemId,
}

/// Ingredient-buffer depth per consumer, in items.
///
/// Chosen to sit in the range #448's dumps actually show — head machines
/// holding 42 and 34 copper-cable. It is a stated approximation of
/// Factorio's per-recipe insertion limit, not a measured constant, and is
/// therefore a candidate entry for `docs/meter-divergence.md`; PR 3
/// replaces it with the real per-recipe rule when machines land.
const BUFFER_CAP: u32 = 40;

impl RowFixture {
    pub fn build(
        tier: BeltTier,
        kind: InserterKind,
        capacity_level: u8,
        consumers: usize,
        demand_per_consumer: f64,
        margin: f64,
        item_name: &str,
    ) -> Self {
        let mut world = World::new();
        let item = world.items.intern(item_name);
        let supply_per_second = demand_per_consumer * consumers as f64 * margin;

        // One tile per consumer, laid out west→east, dead-ended.
        let tiles: Vec<(i32, i32)> = (0..consumers as i32).map(|x| (x, 0)).collect();
        let run = world.add_run(BeltRun::new(tier, tiles, RunEnd::DeadEnd));

        let mut consumer_ids = Vec::new();
        let mut chest_ids = Vec::new();
        for tile in 0..consumers {
            let chest = world.add_consumer(BUFFER_CAP, demand_per_consumer);
            let ins = world.add_inserter(Inserter::new(
                kind,
                PickupTarget::BeltTile { run, tile },
                DropTarget::Chest(chest),
                capacity_level,
            ));
            consumer_ids.push(ins);
            chest_ids.push(chest);
        }

        // Feed both lanes at the head, splitting the offered rate.
        for lane in 0..2 {
            world.add_source(Source::new(run, lane, item, supply_per_second / 2.0));
        }

        RowFixture {
            world,
            run,
            consumers: consumer_ids,
            chests: chest_ids,
            item,
        }
    }

    /// Delivered rate per consumer position, head first.
    pub fn rates(&self) -> Vec<f64> {
        self.consumers
            .iter()
            .map(|&i| self.world.inserter_rate(i))
            .collect()
    }

    /// Belt occupancy per tile (both lanes), head first — the gradient.
    pub fn tile_occupancy(&self) -> Vec<usize> {
        let run = &self.world.runs[self.run];
        (0..run.tiles.len())
            .map(|t| run.occupancy_in_tile(t))
            .collect()
    }
}
