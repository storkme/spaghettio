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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ToOwnedTemplateError {
    #[error("entity {idx} has unknown name {name:?} (expected one of: {expected:?})")]
    UnknownName {
        idx: usize,
        name: String,
        expected: &'static [&'static str],
    },
    #[error("entity {idx} has unknown io_type {io_type:?} (expected \"input\" or \"output\")")]
    UnknownIoType { idx: usize, io_type: String },
}

impl PlacedTemplate {
    /// Convert to [`crate::bus::balancer_generate::OwnedTemplate`] so the
    /// template can be classified by [`crate::bus::balancer_classify`]
    /// (which requires `&'static str` names from a fixed set). Maps each
    /// entity's owned [`String`] name to the matching static literal;
    /// returns an error for unrecognized names.
    pub fn into_owned_template(
        self,
    ) -> Result<crate::bus::balancer_generate::OwnedTemplate, ToOwnedTemplateError> {
        use crate::bus::balancer_library::BalancerTemplateEntity;

        let mut entities = Vec::with_capacity(self.entities.len());
        for (idx, e) in self.entities.into_iter().enumerate() {
            let name: &'static str = match e.name.as_str() {
                "transport-belt" => "transport-belt",
                "splitter" => "splitter",
                "underground-belt" => "underground-belt",
                _ => {
                    return Err(ToOwnedTemplateError::UnknownName {
                        idx,
                        name: e.name,
                        expected: &["transport-belt", "splitter", "underground-belt"],
                    });
                }
            };
            let io_type: Option<&'static str> = match e.io_type.as_deref() {
                None => None,
                Some("input") => Some("input"),
                Some("output") => Some("output"),
                Some(other) => {
                    return Err(ToOwnedTemplateError::UnknownIoType {
                        idx,
                        io_type: other.to_string(),
                    });
                }
            };
            entities.push(BalancerTemplateEntity {
                name,
                x: e.x,
                y: e.y,
                direction: e.direction,
                io_type,
                input_priority: None,
                output_priority: None,
            });
        }
        Ok(crate::bus::balancer_generate::OwnedTemplate {
            n_inputs: self.n_inputs,
            n_outputs: self.n_outputs,
            width: self.width,
            height: self.height,
            entities,
            input_tiles: self.input_tiles,
            output_tiles: self.output_tiles,
        })
    }
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
