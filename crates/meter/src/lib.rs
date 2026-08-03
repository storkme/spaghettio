//! `spaghettio_meter` — RFC-054's fast meter.
//!
//! A native item-level discrete simulator. It is **a meter, not a
//! validator**: it emits numbers (rates, occupancy, machine census), never
//! `Issue`s. A validator returns a verdict, which you cannot optimize
//! against; a meter returns a measurement, which you can.
//!
//! ## Why it exists
//!
//! The engine's validators model flow conservation on a static graph. In
//! that language `supply == demand` is *correct* — and yet
//! [#448](https://github.com/storkme/spaghettio/issues/448) measured rows
//! where an item is simultaneously backed up at its producers and absent
//! at its tail consumers. The mechanism is time-domain (gap propagation,
//! buffer depth, inserter burst size, consumer order along a belt), so it
//! is not a missing check but a dimension the static model does not have.
//!
//! ## Integrity boundary (KC4)
//!
//! This crate may use `spaghettio_core` for **data** — the blueprint
//! parser and the recipe database. It must never import the engine's
//! *derived rate model*: `machine_feed_rate`, `belt_drop_rate`,
//! `lane_capacity*`, `utilization_for`, `LANE_UTILIZATION`,
//! `ROW_LANE_FACTOR_*`. Those are hand-calibrated estimates; importing one
//! would make the meter reproduce the engine's belief instead of measuring
//! it, and its agreement would be circular. Enforced by
//! `tests/kc4_independence.rs`.
//!
//! ## Status
//!
//! PR 1 of 4: physics core (belts, lanes, inserters, containers) for
//! **linear** belt runs, plus the KC4 guard. Blueprint ingestion, machines,
//! convergence detection and `MeterReport` land with PR 3; the corpus
//! replay that actually evaluates KC1–KC3 is PR 4. Tracking: #457.

pub mod belt;
pub mod blueprint_in;
pub mod entity_data;
pub mod factory;
pub mod fluid;
pub mod inserter;
pub mod machine;
pub mod manifest;
pub mod network;
pub mod world;

pub use belt::{BeltRun, ItemId, Lane, RunEnd};
pub use blueprint_in::{decode, Dir, RawEntity};
pub use entity_data::{BeltTier, InserterKind};
pub use factory::{Factory, MeterReport};
pub use inserter::{DropTarget, Inserter, PickupTarget};
pub use machine::{Machine, MachineState};
pub use manifest::Manifest;
pub use network::{BeltNetwork, NetworkBuilder, TileKind, TopologyNote};
pub use world::{Chest, ItemInterner, RowFixture, Source, World};
