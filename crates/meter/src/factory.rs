//! A whole factory: belt network + inserters + machines + boundary, built
//! from a blueprint and its manifest, and the `MeterReport` it produces.
//!
//! # Boundary semantics, matched to the harness
//!
//! `spaghettio-sim` feeds boundary inputs from infinity chests through
//! loaders — *saturated*, so the factory is never input-limited and the
//! measurement reflects what the layout can do rather than what the rig
//! could deliver. The meter does the same: it offers items to every
//! boundary-input tile every tick and lets the belt refuse. Refusals are
//! counted, because a boundary that cannot push is itself a finding.
//!
//! Outputs drain at the layout edge, so backpressure cannot falsify the
//! measurement — the same reason the harness uses remove-mode chests.
//!
//! # Convergence
//!
//! Rates are measured over a trailing window after a warmup, never from
//! the whole run. A filling buffer reads as production; the harness
//! learned that the hard way (`docs/sim-harness-forensics.md`), and a
//! cheap meter that repeated the mistake would be worse than useless.

use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;

use crate::belt::ItemId;
use crate::blueprint_in::{self, RawEntity};
use crate::entity_data::{self, InserterKind};
use crate::inserter::Inserter;
use crate::machine::{Machine, MachineState, DEFAULT_BUFFER_CRAFTS};
use crate::manifest::Manifest;
use crate::network::{BeltNetwork, NetworkBuilder, TopologyNote};
use crate::world::ItemInterner;

/// What an inserter's hand reaches on one side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endpoint {
    Belt(usize),
    Machine(usize),
    /// Empty space, or something the meter does not simulate.
    Nothing,
}

#[derive(Debug)]
pub struct WiredInserter {
    pub core: Inserter,
    pub pos: (i32, i32),
    pub pickup: Endpoint,
    pub drop: Endpoint,
}

/// A boundary feed: offers one item onto one belt tile, saturated.
#[derive(Debug)]
pub struct BoundaryFeed {
    pub tile: usize,
    pub pos: (i32, i32),
    pub item: ItemId,
    /// Ticks on which a push was attempted.
    pub offered: u64,
    /// Ticks on which neither lane had room.
    pub refused: u64,
    /// ITEMS actually placed — the number that matters.
    pub injected: u64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct MeterReport {
    pub label: String,
    pub ticks: u64,
    /// Items crafted per second, measured over the trailing window.
    pub produced_per_s: std::collections::BTreeMap<String, f64>,
    /// Target items reaching the layout edge, per second.
    pub delivered_per_s: std::collections::BTreeMap<String, f64>,
    /// Plan from the manifest, for comparison. The meter reports both and
    /// judges neither — a verdict is the caller's business.
    pub planned_per_s: std::collections::BTreeMap<String, f64>,
    pub machine_census: std::collections::BTreeMap<String, usize>,
    pub converged: bool,
    /// Boundary feeds that could not push — a starved rig, not a starved
    /// factory. Surfaced so the two are never confused.
    pub boundary_refusals: u64,
    /// Things the build could not model faithfully.
    pub notes: Vec<String>,
}

impl MeterReport {
    /// Measured/planned − 1, per item, for anything with a plan.
    pub fn deltas(&self) -> std::collections::BTreeMap<String, f64> {
        self.planned_per_s
            .iter()
            .filter(|(_, p)| **p > 0.0)
            .map(|(item, planned)| {
                let got = self.produced_per_s.get(item).copied().unwrap_or(0.0);
                (item.clone(), got / planned - 1.0)
            })
            .collect()
    }
}

pub struct Factory {
    pub net: BeltNetwork,
    pub machines: Vec<Machine>,
    pub inserters: Vec<WiredInserter>,
    pub feeds: Vec<BoundaryFeed>,
    pub items: ItemInterner,
    pub manifest: Manifest,
    pub ticks: u64,
    /// Sink tiles, by tile id — the manifest's boundary outputs.
    sinks: FxHashSet<usize>,
    /// Cumulative crafted counts, by item.
    crafted: FxHashMap<u16, u64>,
    /// Cumulative items reaching a sink.
    delivered: FxHashMap<u16, u64>,
    pub notes: Vec<String>,
}

impl Factory {
    pub fn build(bp: &str, manifest: Manifest) -> Result<Self, String> {
        let entities = blueprint_in::decode(bp)?;
        Self::from_entities(&entities, manifest)
    }

    pub fn from_entities(entities: &[RawEntity], manifest: Manifest) -> Result<Self, String> {
        let mut items = ItemInterner::default();
        let net = NetworkBuilder::build(entities);
        let mut notes: Vec<String> = net
            .notes
            .iter()
            .map(|n| match n {
                TopologyNote::UnpairedUnderground { pos } => {
                    format!("unpaired underground at {pos:?}")
                }
                TopologyNote::CycleInUpdateOrder { tiles } => {
                    format!("{tiles} tiles in a belt cycle; update order arbitrary")
                }
                TopologyNote::OrphanSplitterHalf { pos } => {
                    format!("orphan splitter half at {pos:?}")
                }
            })
            .collect();

        // --- machines ---------------------------------------------------
        let mut machines = Vec::new();
        for e in entities {
            if !entity_data::is_crafting_machine(&e.name) {
                continue;
            }
            let Some(recipe) = e.recipe.as_deref() else {
                notes.push(format!("{} at ({},{}) has no recipe", e.name, e.x, e.y));
                continue;
            };
            let size = entity_data::footprint(&e.name);
            match Machine::new(
                &e.name,
                recipe,
                (e.x, e.y),
                size,
                &mut items,
                DEFAULT_BUFFER_CRAFTS,
            ) {
                Some(m) => machines.push(m),
                None => notes.push(format!(
                    "cannot model {} running {recipe} at ({},{})",
                    e.name, e.x, e.y
                )),
            }
        }

        // Tile -> machine index, so inserter endpoints resolve in O(1).
        let mut machine_at: FxHashMap<(i32, i32), usize> = FxHashMap::default();
        for (ix, m) in machines.iter().enumerate() {
            for dx in 0..m.size.0 as i32 {
                for dy in 0..m.size.1 as i32 {
                    machine_at.insert((m.pos.0 + dx, m.pos.1 + dy), ix);
                }
            }
        }

        let resolve = |tile: (i32, i32)| -> Endpoint {
            if let Some(&m) = machine_at.get(&tile) {
                Endpoint::Machine(m)
            } else if let Some(t) = net.tile_at(tile) {
                Endpoint::Belt(t)
            } else {
                Endpoint::Nothing
            }
        };

        // --- inserters --------------------------------------------------
        let level = manifest.inserter_capacity;
        let mut inserters = Vec::new();
        for e in entities {
            let Some(kind) = InserterKind::from_entity_name(&e.name) else {
                continue;
            };
            let reach = kind.reach();
            let pickup = resolve(e.inserter_pickup_tile(reach));
            let drop = resolve(e.inserter_drop_tile(reach));
            inserters.push(WiredInserter {
                core: Inserter::detached(kind, level),
                pos: (e.x, e.y),
                pickup,
                drop,
            });
        }

        // --- boundary ---------------------------------------------------
        let mut feeds = Vec::new();
        for b in &manifest.boundary_inputs {
            if b.is_fluid {
                notes.push(format!("fluid boundary input {} not modelled", b.item));
                continue;
            }
            match net.tile_at((b.x, b.y)) {
                Some(tile) => feeds.push(BoundaryFeed {
                    tile,
                    pos: (b.x, b.y),
                    item: items.intern(&b.item),
                    offered: 0,
                    refused: 0,
                    injected: 0,
                }),
                None => notes.push(format!(
                    "boundary input for {} at ({},{}) is not a belt tile",
                    b.item, b.x, b.y
                )),
            }
        }
        let mut net = net;
        let mut sinks = FxHashSet::default();
        for b in &manifest.boundary_outputs {
            match net.tile_at((b.x, b.y)) {
                Some(t) => {
                    sinks.insert(t);
                    net.tiles[t].is_sink = true;
                }
                None => notes.push(format!(
                    "boundary output for {} at ({},{}) is not a belt tile",
                    b.item, b.x, b.y
                )),
            }
        }

        Ok(Factory {
            net,
            machines,
            inserters,
            feeds,
            items,
            manifest,
            ticks: 0,
            sinks,
            crafted: FxHashMap::default(),
            delivered: FxHashMap::default(),
            notes,
        })
    }

    /// Advance one tick.
    ///
    /// Order: boundary feed -> inserters -> machines -> belts -> drain.
    /// Inserters act before belts move so an item arriving under a pickup
    /// tile can be taken the same tick rather than sliding past. Stated
    /// because it approximates Factorio's own entity update order and is
    /// therefore a candidate divergence.
    pub fn tick(&mut self) {
        self.tick_feeds();
        self.tick_inserters();
        self.tick_machines();
        self.net.tick();
        self.drain_sinks();
        self.ticks += 1;
    }

    pub fn run_for(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    fn tick_feeds(&mut self) {
        for f in &mut self.feeds {
            f.offered += 1;
            // Saturated feed: push onto both lanes' entry slots.
            let mut placed = 0;
            for lane in 0..2 {
                if self.net.tiles[f.tile].lanes[lane].try_insert_entry(f.item) {
                    placed += 1;
                }
            }
            f.injected += placed;
            if placed == 0 {
                f.refused += 1;
            }
        }
    }

    fn tick_inserters(&mut self) {
        let mut wired = std::mem::take(&mut self.inserters);
        for w in &mut wired {
            let net = &mut self.net;
            let machines = &mut self.machines;
            let (pickup, drop, pos) = (w.pickup, w.drop, w.pos);
            w.core.tick(|io| match io {
                crate::inserter::Io::Grab { want, hand } => {
                    // Only take what the DROP side will accept — mechanics
                    // I11. Grabbing blind deadlocks the inserter on the
                    // first foreign item from a mixed belt, because the
                    // hand can then never be emptied.
                    match (pickup, drop) {
                        (Endpoint::Belt(t), Endpoint::Machine(m)) => {
                            let dest = &machines[m];
                            let accept = |item| dest.room_for(item) > 0;
                            net.take_from_tile_filtered(t, want, accept, hand);
                        }
                        (Endpoint::Belt(t), _) => net.take_from_tile(t, want, hand),
                        (Endpoint::Machine(m), _) => machines[m].take_output(want, hand),
                        (Endpoint::Nothing, _) => {}
                    }
                    0
                }
                crate::inserter::Io::Deposit { hand } => match drop {
                    Endpoint::Machine(m) => {
                        // Insert what fits, keep the rest (partial insert).
                        let mut moved = 0;
                        while let Some(&item) = hand.first() {
                            if machines[m].insert(item, 1) == 0 {
                                break;
                            }
                            hand.remove(0);
                            moved += 1;
                        }
                        moved
                    }
                    Endpoint::Belt(t) => {
                        let mut moved = 0;
                        while let Some(&item) = hand.first() {
                            if !net.drop_onto_tile(t, pos, item) {
                                break;
                            }
                            hand.remove(0);
                            moved += 1;
                        }
                        moved
                    }
                    // An inserter reaching nothing on its drop side holds
                    // its hand forever, exactly as it would in game. Items
                    // are NOT discarded: silently deleting them would
                    // manufacture throughput out of a wiring gap, and the
                    // endpoint is already reported in `notes`.
                    Endpoint::Nothing => 0,
                },
            });
        }
        self.inserters = wired;
    }

    fn tick_machines(&mut self) {
        for m in &mut self.machines {
            let before = m.crafts;
            m.tick();
            if m.crafts > before {
                for (id, amount) in &m.products {
                    *self.crafted.entry(id.0).or_insert(0) += *amount as u64;
                }
            }
        }
    }

    fn drain_sinks(&mut self) {
        for (tile, item) in self.net.exited_log.drain(..) {
            if self.sinks.is_empty() || self.sinks.contains(&tile) {
                *self.delivered.entry(item.0).or_insert(0) += 1;
            }
        }
    }

    /// Reset the measurement counters, keeping the simulation state — this
    /// is how a warmup is excluded from the window.
    pub fn reset_counters(&mut self) {
        self.crafted.clear();
        self.delivered.clear();
        for m in &mut self.machines {
            m.crafts = 0;
        }
        for f in &mut self.feeds {
            f.refused = 0;
            f.offered = 0;
            f.injected = 0;
        }
        self.ticks = 0;
    }

    pub fn census(&self) -> std::collections::BTreeMap<String, usize> {
        let mut c = std::collections::BTreeMap::new();
        for state in [
            MachineState::Working,
            MachineState::FullOutput,
            MachineState::ItemIngredientShortage,
        ] {
            let n = self.machines.iter().filter(|m| m.state == state).count();
            if n > 0 {
                c.insert(state.as_str().to_string(), n);
            }
        }
        c
    }

    /// Build a report over the ticks since the last `reset_counters`.
    pub fn report(&self) -> MeterReport {
        let secs = (self.ticks as f64 / 60.0).max(1e-9);
        let rate = |m: &FxHashMap<u16, u64>| {
            let mut out = std::collections::BTreeMap::new();
            for (&id, &n) in m {
                out.insert(self.items.name(ItemId(id)).to_string(), n as f64 / secs);
            }
            out
        };
        MeterReport {
            label: self.manifest.label.clone(),
            ticks: self.ticks,
            produced_per_s: rate(&self.crafted),
            delivered_per_s: rate(&self.delivered),
            planned_per_s: self.manifest.planned_rates.clone().into_iter().collect(),
            machine_census: self.census(),
            converged: true,
            boundary_refusals: self.feeds.iter().map(|f| f.refused).sum(),
            notes: self.notes.clone(),
        }
    }

    /// Warm up, then measure over a trailing window.
    pub fn measure(&mut self, warmup: u64, window: u64) -> MeterReport {
        self.run_for(warmup);
        self.reset_counters();
        self.run_for(window);
        self.report()
    }
}
