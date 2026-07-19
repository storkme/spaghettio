//! `spaghettio_core` — shared pipeline logic for the Factorio blueprint generator.
//!
//! Consumed by the WASM web app (`crates/wasm-bindings`). Enable the
//! `wasm` feature to gate in `tsify-next`/`wasm-bindgen` derives;
//! otherwise the crate is pure Rust.
//!
//! Pipeline: solver → bus layout → blueprint export → validation.
//! See `CLAUDE.md` and `docs/ghost-pipeline-contracts.md` for the full picture.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod analysis;
pub mod astar;
pub mod balancer;
pub mod blueprint;
pub mod blueprint_parser;
pub mod bus;
pub mod common;
pub mod density;
pub mod fixture;
pub mod fluid_ports;
pub mod models;
pub mod netflow;
pub mod recipe_db;
pub mod sat;
pub mod short_ids;
pub mod snapshot;
pub mod solver;
pub mod trace;
pub mod validate;
pub mod zone_cache;
