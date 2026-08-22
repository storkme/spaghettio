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

use rustc_hash::FxHashMap;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::belt::ItemId;
use crate::blueprint_in::{self, Dir, RawEntity};
use crate::entity_data::{self, InserterKind};
use crate::fluid::{self, MachPort};
use crate::inserter::Inserter;
use crate::machine::{Machine, MachineState, DEFAULT_BUFFER_CRAFTS};
use crate::manifest::Manifest;
use crate::network::{BeltNetwork, NetworkBuilder, SplitterStats, TopologyNote};
use crate::world::ItemInterner;
use spaghettio_core::recipe_db;

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
    /// Products emitted per second, measured over the trailing window. This
    /// includes both solid products and fluid products, before transport
    /// delivery.
    pub produced_per_s: std::collections::BTreeMap<String, f64>,
    /// Target items reaching the layout edge, per second.
    pub delivered_per_s: std::collections::BTreeMap<String, f64>,
    /// Plan from the manifest, for comparison. The meter reports both and
    /// judges neither — a verdict is the caller's business.
    pub planned_per_s: std::collections::BTreeMap<String, f64>,
    pub machine_census: std::collections::BTreeMap<String, usize>,
    /// Window-scoped attribution by recipe. This is diagnostic evidence, not
    /// a second verdict: it says where the meter spent time and fluid, so a
    /// meter/sim disagreement can be traced to a stage instead of guessed
    /// from the target rate alone.
    pub recipe_attribution: BTreeMap<String, RecipeAttribution>,
    /// Window-scoped splitter routing evidence, in topology order. This is
    /// diagnostic only, but makes branch-distribution measurements
    /// reproducible from the saved report.
    pub splitter_stats: Vec<SplitterStats>,
    pub converged: bool,
    /// Boundary feeds that could not push — a starved rig, not a starved
    /// factory. Surfaced so the two are never confused.
    pub boundary_refusals: u64,
    /// Things the build could not model faithfully.
    pub notes: Vec<String>,
}

/// Per-recipe evidence collected over the measurement window.
#[derive(Debug, Default, Clone, Serialize)]
pub struct RecipeAttribution {
    pub machines: usize,
    pub crafts: u64,
    pub working_ticks: u64,
    pub output_blocked_ticks: u64,
    pub output_inserter_blocked_ticks: u64,
    pub item_shortage_ticks: u64,
    pub fluid_shortage_ticks: u64,
    pub fluid_supplied: BTreeMap<String, u64>,
    pub fluid_consumed: BTreeMap<String, u64>,
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
    /// Cumulative crafted counts, by item.
    crafted: FxHashMap<u16, u64>,
    /// Cumulative items reaching a sink.
    delivered: FxHashMap<u16, u64>,
    pub notes: Vec<String>,
    /// `(tick, total delivered)` sampled every [`CHECKPOINT_TICKS`] since
    /// the last counter reset. Feeds the convergence test in [`Self::report`].
    checkpoints: Vec<(u64, u64)>,
    /// Fluid pipe topology (RFC-054 Phase B): connected pipe + port + feed
    /// components that `tick_fluids` routes through.
    pub fluids: crate::fluid::FluidSystem,
}

/// Convergence sampling cadence, matching `spaghettio-sim`'s 3600-tick
/// (one game-minute) checkpoints so the two instruments' `converged`
/// flags answer the same question.
const CHECKPOINT_TICKS: u64 = 3600;

/// A run is converged when the last [`CONVERGENCE_WINDOWS`] checkpoint
/// deltas differ from their mean by less than this. 2% is the sim
/// harness's own steady-state tolerance.
const CONVERGENCE_TOLERANCE: f64 = 0.02;
const CONVERGENCE_WINDOWS: usize = 3;

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
        let mut machine_ports: Vec<MachPort> = Vec::new();
        let db = recipe_db::db();
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
                // Declared, not inferred: absent from the manifest means no
                // research productivity, which is what every manifest written
                // before this axis existed says.
                manifest
                    .research_productivity
                    .get(recipe)
                    .copied()
                    .unwrap_or(0.0),
            ) {
                Some(m) => {
                    // Collect this machine's fluid ports, bound to the recipe's
                    // fluid items (RFC-054 Phase B). Port fluid binding is
                    // x-ascending over each IO face EXCEPT on the machines the
                    // engine mirrors (oil-refinery, foundry, cryogenic-plant),
                    // whose exported orientation binds recipe fluids
                    // x-descending (the measured rule in `spaghettio_core::
                    // fluid_ports`). The decoder parses the blueprint
                    // `mirror` flag since 2026-08-21 (offpath B2), but this
                    // site deliberately does NOT consume it: for the three
                    // engine-mirrored machines the name heuristic already
                    // applies the binding flip (their mirror is
                    // tile-identical, so the flag adds nothing), and for any
                    // OTHER machine an explicit mirror:true also MOVES port
                    // tiles — honoring it as a binding flip alone would
                    // mis-place ports on asymmetric machines (#685 review).
                    // The complete general fix is a reflect_port
                    // (w-1-dx on port x) plus the flag; until built, the
                    // flag is decoder knowledge only, and the engine-form
                    // undecidable (unmirrored community instance of the
                    // three, multi-fluid face) stays documented in
                    // meter-divergence.md rather than guessed.
                    let mirrored = matches!(
                        e.name.as_str(),
                        "oil-refinery" | "foundry" | "cryogenic-plant"
                    );
                    let mi = machines.len();
                    if let Some(rdb) = db.recipes.get(recipe) {
                        let mut in_fluids: Vec<String> = Vec::new();
                        for ing in &rdb.ingredients {
                            if ing.type_ == "fluid" {
                                in_fluids.push(ing.name.clone());
                            }
                        }
                        let mut out_fluids: Vec<String> = Vec::new();
                        for p in &rdb.products {
                            if p.type_ == "fluid" {
                                out_fluids.push(p.name.clone());
                            }
                        }
                        let base = entity_data::base_fluid_ports(&e.name);
                        let w = size.0 as i32;
                        // inputs
                        let mut in_ports: Vec<(i32, i32)> = Vec::new();
                        for &(dx, dy, io) in base {
                            if io == entity_data::PortIO::Input {
                                in_ports.push(entity_data::rotate_port(e.direction, dx, dy, w));
                            }
                        }
                        in_ports.sort_by_key(|p| p.0);
                        // Mirror the engine's `port_fluid_assignment` exactly:
                        // only the first n = fluids.len() ports are used, and a
                        // mirrored machine binds fluid fluids[n-1-k] to port k
                        // (i.e. fluid at recipe-index k -> port index n-1-k).
                        let n = in_fluids.len();
                        for (k, name) in in_fluids.iter().enumerate() {
                            let pk = if mirrored { n - 1 - k } else { k };
                            if let Some(&(px, py)) = in_ports.get(pk) {
                                machine_ports.push(MachPort {
                                    machine: mi,
                                    x: e.x + px,
                                    y: e.y + py,
                                    item: items.intern(name).0,
                                    is_input: true,
                                });
                            }
                        }
                        // outputs
                        let mut out_ports: Vec<(i32, i32)> = Vec::new();
                        for &(dx, dy, io) in base {
                            if io == entity_data::PortIO::Output {
                                out_ports.push(entity_data::rotate_port(e.direction, dx, dy, w));
                            }
                        }
                        out_ports.sort_by_key(|p| p.0);
                        let n = out_fluids.len();
                        for (k, name) in out_fluids.iter().enumerate() {
                            let pk = if mirrored { n - 1 - k } else { k };
                            if let Some(&(px, py)) = out_ports.get(pk) {
                                machine_ports.push(MachPort {
                                    machine: mi,
                                    x: e.x + px,
                                    y: e.y + py,
                                    item: items.intern(name).0,
                                    is_input: false,
                                });
                            }
                        }
                    }
                    machines.push(m);
                }
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
            // Gate on the NAME first, mirroring the machine loop's
            // `is_crafting_machine`. `entities` is every decoded entity —
            // belts, poles, pipes, machines — so testing
            // `from_entity_name` alone would note ~250 non-inserters on a
            // 292-entity fixture and bury the signal below.
            if !e.name.ends_with("inserter") {
                continue;
            }
            let Some(kind) = InserterKind::from_entity_name(&e.name) else {
                // A real inserter this constructor does not know. It MUST be
                // said: `footprint_checked`'s generic `*-inserter` arm lets
                // burner and filter variants through `decode`'s otherwise-hard
                // unknown-entity gate, so a silent `continue` under-reports
                // throughput with an empty `notes` — the one failure mode a
                // meter must never have.
                notes.push(format!("{} at ({},{}) not modelled", e.name, e.x, e.y));
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
        let mut fluid_feed_tiles: Vec<(u16, (i32, i32))> = Vec::new();
        for b in &manifest.boundary_inputs {
            if b.is_fluid {
                let id = items.intern(&b.item).0;
                fluid_feed_tiles.push((id, (b.x, b.y)));
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
        for b in &manifest.boundary_outputs {
            match net.tile_at((b.x, b.y)) {
                Some(t) => {
                    net.tiles[t].is_sink = true;
                }
                None => notes.push(format!(
                    "boundary output for {} at ({},{}) is not a belt tile",
                    b.item, b.x, b.y
                )),
            }
        }

        // --- fluid pipe network (RFC-054 Phase B) ------------------------
        let mut pipe_entities: Vec<(i32, i32, &str, Dir)> = Vec::new();
        for e in entities {
            if e.name == "pipe" || e.name == "pipe-to-ground" || e.name == "pump" {
                pipe_entities.push((e.x, e.y, &e.name, e.direction));
            }
        }
        let fluids_system =
            fluid::build_networks(&pipe_entities, &machine_ports, &fluid_feed_tiles);
        // Fluid output only creates machine backpressure when this exact
        // fluid has a consumer on the same connected component. Unconnected
        // and boundary-only products retain the meter's report-only drain
        // semantics; multi-output recipes are handled per fluid id.
        for net in &fluids_system.networks {
            for producer in net.ports.iter().filter(|p| !p.is_input) {
                if net
                    .ports
                    .iter()
                    .any(|p| p.is_input && p.item == producer.item)
                {
                    machines[producer.machine]
                        .bounded_fluid_outputs
                        .insert(producer.item);
                }
            }
        }
        for (item, (x, y)) in &fluids_system.unconnected_feeds {
            notes.push(format!(
                "fluid boundary input for {} at ({},{}) touches no pipe network",
                items.name(ItemId(*item)),
                x,
                y
            ));
        }

        Ok(Factory {
            checkpoints: vec![(0, 0)],
            net,
            machines,
            inserters,
            feeds,
            items,
            manifest,
            ticks: 0,
            crafted: FxHashMap::default(),
            delivered: FxHashMap::default(),
            notes,
            fluids: fluids_system,
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
        self.tick_fluids();
        self.net.tick();
        self.drain_sinks();
        self.ticks += 1;
    }

    /// RFC-054 Phase B: route fluid through the pipe networks. Each connected
    /// component's boundary feeds (infinite standing sources, matching the
    /// saturated input rig) and producer `fluid_output` are pooled per item
    /// and drawn by the component's consumers up to their per-craft need —
    /// pipe-fast, so a petroleum→plastic→AC chain is not throttled to one
    /// unit a tick the way Phase A's port-adjacency was. Surplus is retained
    /// when a fluid has attached consumers, allowing downstream full buffers
    /// to back up the producer; a component with no consumer still drains as
    /// delivered, preserving the target-fluid boundary behavior.
    fn tick_fluids(&mut self) {
        let mut drained: FxHashMap<u16, u64> = FxHashMap::default();
        for net in &self.fluids.networks {
            // Group ports by item.
            let mut by_item: FxHashMap<u16, (Vec<usize>, Vec<usize>)> = FxHashMap::default();
            for p in &net.ports {
                let e = by_item.entry(p.item).or_default();
                if p.is_input {
                    e.1.push(p.machine);
                } else {
                    e.0.push(p.machine);
                }
            }
            for (item, (producers, consumers)) in by_item {
                // Boundary standing source: infinite (saturated rig).
                let boundary = net.boundary.contains(&item);
                // Snapshot each producer's on-hand amount for this item.
                let total_held: u64 = producers
                    .iter()
                    .map(|&mi| {
                        self.machines[mi]
                            .fluid_output
                            .get(&item)
                            .copied()
                            .unwrap_or(0) as u64
                    })
                    .sum();
                // Deliver the producer pool fairly across consumers. A greedy
                // always-serve-the-lowest-index-first allocation would starve
                // the last consumer when supply is tight (a real pipe feeds
                // every attached consumer in parallel and shares scarcity).
                // Split each consumer's buffer room, then allocate the pool
                // proportional to room, handing the fractional remainder out
                // one unit at a time (deterministic, largest-remainder first).
                let mut rooms: Vec<(usize, u32)> = Vec::new();
                let mut total_room: u64 = 0;
                for &ci in &consumers {
                    let room = self.machines[ci].fluid_room_for(ItemId(item));
                    if room > 0 {
                        rooms.push((ci, room));
                        total_room += room as u64;
                    }
                }
                let mut pool_alloc: Vec<(usize, u32)> =
                    rooms.iter().map(|&(c, _)| (c, 0)).collect();
                if !rooms.is_empty() {
                    if total_held >= total_room {
                        for (i, &(c, room)) in rooms.iter().enumerate() {
                            pool_alloc[i] = (c, room);
                        }
                    } else {
                        // deficient supply: proportional share, floor, then
                        // hand leftover to the largest fractional remainders.
                        let mut allocated: u64 = 0;
                        let mut rem: Vec<(usize, u64)> = Vec::new();
                        for (i, &(c, room)) in rooms.iter().enumerate() {
                            let share = (room as u64 * total_held) / total_room;
                            pool_alloc[i] = (c, share as u32);
                            allocated += share;
                            rem.push((i, (room as u64 * total_held) % total_room));
                        }
                        rem.sort_by_key(|&(_, r)| std::cmp::Reverse(r));
                        let mut left = total_held - allocated;
                        for (i, _) in rem {
                            if left == 0 {
                                break;
                            }
                            if pool_alloc[i].1 < rooms[i].1 {
                                pool_alloc[i].1 += 1;
                                left -= 1;
                            }
                        }
                    }
                }

                // A fluid output with attached consumers is a bounded
                // machine output, not an implicit world drain. Retain the
                // producer surplus when all consumer buffers are full; the
                // next machine tick will then see `FullOutput` once the
                // output cap is reached. This is the coupling the report-only
                // meter needs for a downstream solid shortage to propagate
                // back through a pipe-fed intermediate (e.g. sulfuric acid
                // into processing units). A component with no consumer keeps
                // the established drain philosophy for genuine byproducts /
                // declared fluid outputs.
                let producer_used: u64 = pool_alloc.iter().map(|&(_, n)| n as u64).sum();
                if consumers.is_empty() && total_held > 0 {
                    *drained.entry(item).or_insert(0) += total_held;
                }
                // Remove what was consumed by attached consumers, or the
                // whole pool when this is a genuine boundary drain. The
                // latter is important: counting a producer buffer as
                // delivered without emptying it re-counts the same fluid on
                // every tick and turns a steady output into a triangular
                // delivery curve.
                let mut remaining_used = if consumers.is_empty() {
                    total_held
                } else {
                    producer_used
                };
                for &pj in &producers {
                    if remaining_used == 0 {
                        break;
                    }
                    if let Some(v) = self.machines[pj].fluid_output.get_mut(&item) {
                        let take = (*v as u64).min(remaining_used) as u32;
                        *v -= take;
                        remaining_used -= take as u64;
                    }
                }
                // Credit consumers: their pool allocation, topped up from the
                // (infinite) boundary source if any. Separate loop to avoid
                // overlapping mutable borrows of `self.machines`.
                for (ci, amt) in pool_alloc {
                    let mut total = amt;
                    if boundary {
                        let room = self.machines[ci].fluid_room_for(ItemId(item));
                        total += room.saturating_sub(total);
                    }
                    if total > 0 {
                        self.machines[ci].insert_fluid(ItemId(item), total);
                    }
                }
            }
        }
        for (item, n) in drained {
            *self.delivered.entry(item).or_insert(0) += n;
        }
    }

    pub fn run_for(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.tick();
            if self.ticks.is_multiple_of(CHECKPOINT_TICKS) {
                let total = self.delivered.values().sum();
                self.checkpoints.push((self.ticks, total));
            }
        }
    }

    /// Has throughput stopped moving?
    ///
    /// Compares the last [`CONVERGENCE_WINDOWS`] per-checkpoint deltas
    /// against their mean. Deliberately measured on **delivered**, not on
    /// buffer levels: a factory filling its buffers has rising deltas and
    /// must not read as converged, which is the artifact class
    /// `sim-harness-forensics.md` records the real harness learning the
    /// hard way.
    ///
    /// Too few checkpoints to judge is reported as NOT converged — the
    /// honest direction for a field a caller may gate on.
    fn detect_converged(&self) -> bool {
        converged_from(&self.checkpoints)
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
            m.tick();
            // Credit what the machine actually emitted, never a re-derivation
            // from `products` — those are fractional expectations, and casting
            // one to an integer here truncated 0.25 to 0 and 1.5 to 1 while
            // the machine's own carry got it right, so `produced_per_s`
            // disagreed with belt-delivered throughput in the same report.
            for (id, n) in &m.emitted_this_tick {
                *self.crafted.entry(*id).or_insert(0) += *n as u64;
            }
            for (id, n) in &m.fluid_emitted_this_tick {
                *self.crafted.entry(*id).or_insert(0) += *n as u64;
            }
        }
    }

    /// Count everything that left the network this tick.
    ///
    /// No sink filter here, and deliberately: `exited_log` is only ever
    /// appended to from the two `is_sink` arms in `BeltNetwork`, so every
    /// entry is already a declared boundary output. An earlier version
    /// re-checked membership against a separate `sinks` set, which read
    /// like a meaningful filter — including an `if sinks.is_empty()`
    /// fallback suggesting "count everything when the manifest declares no
    /// outputs". Both were unreachable: with no sinks, nothing sets
    /// `is_sink`, so `exited_log` stays empty. Two representations of one
    /// fact that could drift apart, and a dead branch a later reader would
    /// have trusted.
    fn drain_sinks(&mut self) {
        for (_tile, item) in self.net.exited_log.drain(..) {
            *self.delivered.entry(item.0).or_insert(0) += 1;
        }
    }

    /// Reset the measurement counters, keeping the simulation state — this
    /// is how a warmup is excluded from the window.
    pub fn reset_counters(&mut self) {
        self.crafted.clear();
        self.delivered.clear();
        self.net.reset_splitter_stats();
        // Checkpoints describe the window we just discarded; keeping them
        // would let a pre-warmup transient decide `converged`. The window's
        // OWN start is a sample though — cumulative zero at tick zero — and
        // seeding it is what makes an N-checkpoint window yield N deltas
        // rather than N-1.
        self.checkpoints.clear();
        self.checkpoints.push((0, 0));
        for m in &mut self.machines {
            m.reset_counters();
        }
        for wired in &mut self.inserters {
            wired.core.deposit_blocked_ticks = 0;
        }
        for f in &mut self.feeds {
            f.refused = 0;
            f.offered = 0;
            f.injected = 0;
        }
        self.ticks = 0;
    }

    /// Aggregate each machine's window-scoped counters by recipe. Machine
    /// indices are intentionally not exposed here: recipe-level attribution
    /// is stable across blueprint entity ordering and is what calibration
    /// comparisons need.
    fn recipe_attribution(&self) -> BTreeMap<String, RecipeAttribution> {
        let mut out: BTreeMap<String, RecipeAttribution> = BTreeMap::new();
        for machine in &self.machines {
            let entry = out.entry(machine.recipe.clone()).or_default();
            entry.machines += 1;
            entry.crafts += machine.crafts;
            entry.working_ticks += machine.working_ticks;
            entry.output_blocked_ticks += machine.output_blocked_ticks;
            entry.item_shortage_ticks += machine.item_shortage_ticks;
            entry.fluid_shortage_ticks += machine.fluid_shortage_ticks;
            for (id, amount) in &machine.fluid_supplied {
                *entry
                    .fluid_supplied
                    .entry(self.items.name(ItemId(*id)).to_string())
                    .or_insert(0) += amount;
            }
            for (id, amount) in &machine.fluid_consumed {
                *entry
                    .fluid_consumed
                    .entry(self.items.name(ItemId(*id)).to_string())
                    .or_insert(0) += amount;
            }
        }
        for wired in &self.inserters {
            let Endpoint::Machine(machine) = wired.pickup else {
                continue;
            };
            if !matches!(wired.drop, Endpoint::Belt(_)) {
                continue;
            }
            out.entry(self.machines[machine].recipe.clone())
                .or_default()
                .output_inserter_blocked_ticks += wired.core.deposit_blocked_ticks;
        }
        out
    }

    pub fn census(&self) -> std::collections::BTreeMap<String, usize> {
        let mut c = std::collections::BTreeMap::new();
        for state in [
            MachineState::Working,
            MachineState::FullOutput,
            MachineState::ItemIngredientShortage,
            MachineState::FluidIngredientShortage,
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
            recipe_attribution: self.recipe_attribution(),
            splitter_stats: self.net.splitter_stats.clone(),
            converged: self.detect_converged(),
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

/// Steady-state test over `(tick, cumulative delivered)` checkpoints.
///
/// Split out from [`Factory::detect_converged`] so the decision rule is
/// testable without standing up a whole factory — the rule is the part
/// that can be subtly wrong.
fn converged_from(checkpoints: &[(u64, u64)]) -> bool {
    if checkpoints.len() < CONVERGENCE_WINDOWS + 1 {
        return false;
    }
    let tail = &checkpoints[checkpoints.len() - (CONVERGENCE_WINDOWS + 1)..];
    let deltas: Vec<f64> = tail
        .windows(2)
        .map(|w| (w[1].1.saturating_sub(w[0].1)) as f64)
        .collect();
    let mean = deltas.iter().sum::<f64>() / deltas.len() as f64;
    if mean <= 0.0 {
        // Nothing delivered at all. Stable, but not a steady state any
        // caller should treat as a converged measurement.
        return false;
    }
    deltas
        .iter()
        .all(|d| (d - mean).abs() / mean < CONVERGENCE_TOLERANCE)
}

#[cfg(test)]
mod convergence_tests {
    use super::*;

    fn ticks(counts: &[u64]) -> Vec<(u64, u64)> {
        counts
            .iter()
            .enumerate()
            .map(|(i, &c)| (i as u64 * CHECKPOINT_TICKS, c))
            .collect()
    }

    #[test]
    fn flat_delivery_is_converged() {
        // +180 every window, exactly the shape a steady factory shows.
        assert!(converged_from(&ticks(&[0, 180, 360, 540, 720])));
    }

    #[test]
    fn a_filling_buffer_is_not_converged() {
        // Rising deltas: 100, 140, 180, 220. This is THE artifact class
        // the harness records learning the hard way — a transient that
        // reads as production. It must not pass.
        assert!(!converged_from(&ticks(&[0, 100, 240, 420, 640])));
    }

    #[test]
    fn producing_nothing_is_not_converged() {
        // Perfectly stable at zero. Stable is not the question.
        assert!(!converged_from(&ticks(&[0, 0, 0, 0, 0])));
    }

    #[test]
    fn too_few_checkpoints_is_not_converged() {
        // Under-report honestly: no evidence is not evidence of steadiness.
        assert!(!converged_from(&ticks(&[0, 180, 360])));
        assert!(!converged_from(&[]));
    }

    #[test]
    fn small_jitter_is_still_converged() {
        // 180/181/179/180 — within the 2% tolerance.
        assert!(converged_from(&ticks(&[0, 180, 361, 540, 720])));
    }

    /// The detector must be able to fire under the window the callers
    /// ACTUALLY ship, not merely under hand-built checkpoint arrays.
    ///
    /// The first version of this detector required 4 checkpoints while
    /// `corpus_replay` and `examples/measure` both use a 3-game-minute
    /// window, which produces exactly 3 — so `converged` was
    /// unconditionally false, a hardcoded `true` swapped for an
    /// effectively hardcoded `false`. Every unit test above still passed,
    /// because they construct checkpoints directly and never exercise the
    /// schedule `run_for` generates. Same lesson this RFC already learned
    /// twice: a test that samples the rule is not testing the wiring.
    #[test]
    fn the_shipped_measurement_window_can_converge() {
        const WINDOW: u64 = 60 * 60 * 3; // corpus_replay.rs / measure.rs
                                         // Replay exactly what reset_counters + run_for record for a
                                         // perfectly steady factory.
        let mut cps = vec![(0u64, 0u64)];
        let mut delivered = 0u64;
        for t in 1..=WINDOW {
            delivered += 3;
            if t.is_multiple_of(CHECKPOINT_TICKS) {
                cps.push((t, delivered));
            }
        }
        assert!(
            cps.len() > CONVERGENCE_WINDOWS,
            "the shipped {WINDOW}-tick window yields {} checkpoints; the detector needs {}",
            cps.len(),
            CONVERGENCE_WINDOWS + 1
        );
        assert!(
            converged_from(&cps),
            "a perfectly steady factory must read as converged in the shipped window"
        );
    }
}

#[cfg(test)]
mod note_tests {
    use super::*;
    use crate::blueprint_in::Dir;

    fn ent(name: &str, x: i32, y: i32) -> RawEntity {
        RawEntity {
            name: name.into(),
            x,
            y,
            direction: Dir::North,
            recipe: None,
            io_type: None,
            mirror: false,
        }
    }

    /// Only genuine inserters may produce an "unmodelled inserter" note.
    ///
    /// The build loop walks EVERY decoded entity, so testing
    /// `InserterKind::from_entity_name` without a name gate first noted
    /// every belt, pole and pipe in the blueprint — roughly 250 spurious
    /// notes on a 292-entity fixture, burying the one signal the note
    /// exists to carry. A diagnostic that fires on everything is worse
    /// than no diagnostic.
    #[test]
    fn ordinary_entities_do_not_produce_unmodelled_notes() {
        let ents = vec![
            ent("transport-belt", 0, 0),
            ent("express-transport-belt", 1, 0),
            ent("underground-belt", 2, 0),
            ent("splitter", 3, 0),
            ent("medium-electric-pole", 4, 0),
            ent("inserter", 5, 0),
            ent("long-handed-inserter", 6, 0),
        ];
        let f = Factory::from_entities(&ents, Manifest::default()).expect("builds");
        let spurious: Vec<&String> = f
            .notes
            .iter()
            .filter(|n| n.contains("not modelled"))
            .collect();
        assert!(
            spurious.is_empty(),
            "no ordinary entity may be reported as an unmodelled inserter, got {spurious:?}"
        );
    }

    /// ...but a real inserter this crate cannot model MUST be reported.
    /// `burner-inserter` passes `decode`'s unknown-entity gate through
    /// `footprint_checked`'s generic `*-inserter` arm, so without a note
    /// its transfers vanish silently and throughput under-reports.
    #[test]
    fn an_unmodelled_inserter_is_reported() {
        let ents = vec![ent("burner-inserter", 0, 0)];
        let f = Factory::from_entities(&ents, Manifest::default()).expect("builds");
        assert!(
            f.notes
                .iter()
                .any(|n| n.contains("burner-inserter") && n.contains("not modelled")),
            "an inserter variant the meter cannot model must be noted, got {:?}",
            f.notes
        );
    }

    #[test]
    fn fluid_emission_is_reported_before_pipe_delivery() {
        let entity = RawEntity {
            name: "chemical-plant".into(),
            x: 0,
            y: 0,
            direction: Dir::North,
            recipe: Some("sulfuric-acid".into()),
            io_type: None,
            mirror: false,
        };
        let output_pipe = RawEntity {
            name: "pipe".into(),
            x: 0,
            y: 3,
            direction: Dir::North,
            recipe: None,
            io_type: None,
            mirror: false,
        };
        let mut f =
            Factory::from_entities(&[entity, output_pipe], Manifest::default()).expect("builds");
        let iron = f.items.intern("iron-plate");
        let sulfur = f.items.intern("sulfur");
        let water = f.items.intern("water");
        f.machines[0].insert(iron, 100);
        f.machines[0].insert(sulfur, 100);
        f.machines[0].insert_fluid(water, 100);

        f.run_for(1_200);
        let report = f.report();
        assert!(
            report
                .produced_per_s
                .get("sulfuric-acid")
                .copied()
                .unwrap_or(0.0)
                > 0.0,
            "fluid production must be present even when delivery is a separate pipe concern"
        );
        assert_eq!(
            report.produced_per_s.get("sulfuric-acid"),
            report.delivered_per_s.get("sulfuric-acid"),
            "a no-consumer fluid network must drain each emitted unit once"
        );
    }
}
