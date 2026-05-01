//! [`PlacementEngine`] — Stage 2 of the decoupled balancer pipeline.
//!
//! Mirrors the [`JunctionStrategy`](crate::bus::junction_solver) trait
//! pattern: each engine implements `place(request) -> result` independently,
//! and the bench harness / library generator runs them side by side.
//!
//! ## Engines (planned)
//!
//! - [`library_lookup`] — pulls existing entries from `bus::balancer_library`.
//!   No "placement" really; it's a known-good baseline for cross-checking
//!   engines that *do* place.
//! - **Factorio-SAT** — wraps the existing Python subprocess; same external
//!   tool that produced today's library entries. Subprocess + JSON.
//! - **CP-SAT** — Google OR-tools constraint solver (`cp_sat` Rust crate);
//!   no Python in the new path. `no_overlap_2d` + `circuit` + table
//!   constraints encode the spatial-routing problem natively.
//!
//! ## Trait shape
//!
//! [`PlacementRequest`] carries the abstract [`BalancerGraph`] (from
//! [`crate::balancer::synth`]) plus per-call hints (timeout, seed, grid
//! caps). Engines that ignore the graph and discover their own topology
//! (Factorio-SAT's net-free mode) are allowed to do so — the graph is
//! advisory at the trait level.
//!
//! [`PlacementResult`] is the owned counterpart of [`BalancerTemplate`]
//! plus engine metadata (id, wall-clock).

pub mod cp_sat;
pub mod library_lookup;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::balancer::graph::BalancerGraph;

/// Inputs to a placement engine.
#[derive(Debug, Clone)]
pub struct PlacementRequest<'a> {
    /// Abstract logical graph from synthesis. Engines may consume the
    /// graph as a fixed network or ignore it and discover their own.
    pub graph: &'a BalancerGraph,
    pub n: u32,
    pub m: u32,
    /// Wall-clock budget per call. Engines that can honor this should;
    /// those that can't are expected to return [`PlacementError::Timeout`]
    /// once the budget elapses.
    pub timeout: Duration,
    /// Optional deterministic seed. Honored by engines whose backend
    /// supports it (CP-SAT does; kissat does not).
    pub seed: Option<u64>,
}

/// One placed entity — owned counterpart of
/// [`crate::bus::balancer_library::BalancerTemplateEntity`]. Coordinates
/// are tile offsets from the template origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedTemplateEntity {
    pub name: String,
    pub x: i32,
    pub y: i32,
    /// Factorio 1.0 8-way direction: 0=N, 2=E, 4=S, 6=W.
    pub direction: u8,
    /// `Some("input")` or `Some("output")` for underground-belt, else `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_type: Option<String>,
}

/// Owned counterpart of [`crate::bus::balancer_library::BalancerTemplate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedTemplate {
    pub n_inputs: u32,
    pub n_outputs: u32,
    pub width: u32,
    pub height: u32,
    pub entities: Vec<PlacedTemplateEntity>,
    pub input_tiles: Vec<(i32, i32)>,
    pub output_tiles: Vec<(i32, i32)>,
    /// Original blueprint string when the engine emits one (Factorio-SAT
    /// does; CP-SAT may not).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_blueprint: Option<String>,
}

/// One successful placement.
#[derive(Debug, Clone)]
pub struct PlacementResult {
    pub template: PlacedTemplate,
    pub solve_wall_ms: u64,
    /// Stable engine id, e.g. `"library_lookup"`, `"factorio_sat"`,
    /// `"cp_sat"`. Used for bench reports.
    pub engine_id: &'static str,
}

#[derive(Debug, Error)]
pub enum PlacementError {
    /// Solver returned UNSAT (no valid placement under the given budget).
    #[error("UNSAT")]
    Unsat,
    /// Solver exceeded its wall-clock budget.
    #[error("timed out after {0:?}")]
    Timeout(Duration),
    /// Engine itself crashed or refused to run (e.g. subprocess died).
    #[error("engine error: {0}")]
    Engine(String),
    /// The supplied graph is incompatible with this engine. Engines that
    /// discover their own topology never return this; engines that consume
    /// the graph as fixed network may.
    #[error("graph incompatible: {0}")]
    GraphIncompatible(String),
    /// Shape `(n, m)` not in the engine's repertoire (e.g. library lookup
    /// missing a template).
    #[error("shape ({n}, {m}) not available")]
    ShapeNotAvailable { n: u32, m: u32 },
}

pub trait PlacementEngine {
    fn name(&self) -> &'static str;
    fn place(&self, req: &PlacementRequest<'_>) -> Result<PlacementResult, PlacementError>;
}
