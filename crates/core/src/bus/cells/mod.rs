//! RFC-051: the cell-composition production path (flag-gated).
//!
//! Phase A: the Phase-1 harness lifted behind `CellComposition::Off`
//! (the default — the bus path is bit-identical while the flag is Off,
//! which is what lets the goldens stand untouched). Nothing reads the
//! flag until Phase B wires `CellComposedCandidate` into the
//! decomposition search.

pub mod chain;
pub mod compose;
pub mod extract;
pub mod mega;
// `placement` (RFC-057's linear/fold placement estimator, kept for the
// RFC-055 compact-order experiments) was deleted 2026-08-20 with
// ChainOrder::Compact — owner extended #632 A2 (offpath Tier 2, #675).
pub mod registry;

/// Cell-composition mode (RFC-051). `Off` = production pipeline
/// untouched (default). `Candidate` = eligible solves also produce a
/// `CellComposedCandidate` in the decomposition search (Phase B; the
/// variant exists from Phase A so options plumbing is stable).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellComposition {
    #[default]
    Off,
    Candidate,
}
