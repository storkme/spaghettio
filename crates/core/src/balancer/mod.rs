//! Balancer graph synthesis and verification — Stage 1 of the decoupled
//! balancer pipeline.
//!
//! Pure-logic operations on abstract splitter networks, independent of grid
//! placement. See `docs/refs/2404.05472v2.pdf` (Couëtoux, Gastaldi, Naves
//! 2024) for the steady-state model and `docs/refs/1209.0724v1.pdf` (Zhou,
//! Chen, Bruck 2012) for the synthesis construction.
//!
//! The verifier is **bake-time only** — it allocates a dense matrix sized by
//! arc count, fine for ≤100-arc balancers but unfit for the WASM hot path.
//! Don't reach it from `bus::balancer` or anything called by `solve`/`layout`.

pub mod graph;
pub mod verify;

pub use graph::{Arc, BalancerGraph, GraphError, Sink, Source};
pub use verify::{verify_balancer, verify_balancer_with_tolerance, VerifyError, VerifyOutcome};
