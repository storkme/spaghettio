//! RFC-051: the sim-verified registry (Tier-1 verification cache).
//!
//! Key = chain config + a GEOMETRY HASH of the composed layout's
//! entities (#375 review, finding 1): cells regenerate from the live
//! engine, so a config-only key would let row-template/placer/inserter
//! changes silently decay "sim-verified" into "unverified with a stale
//! verdict". The hash is self-maintaining: any engine change that
//! alters the composed geometry changes the hash, the lookup misses,
//! and the layout is annotated NOT-verified until someone re-runs the
//! sim and re-registers. `cell_registry_hashes_current` (tests) turns a
//! silent miss into a loud one for the seeded entries.
//!
//! Data: `crates/core/data/cell-sim-registry.json`, embedded via
//! `include_str!` like the recipe DB. Grown deliberately — an entry is
//! added only with a real sim PASS behind it (scenario + date recorded).

use std::sync::OnceLock;

use crate::models::LayoutResult;
use serde::Deserialize;

/// One sim-verified composed geometry.
#[derive(Debug, Clone, Deserialize)]
pub struct RegistryEntry {
    pub target: String,
    pub rate: f64,
    /// Hex FNV-1a-64 of the composed layout's entity list.
    pub geometry_hash: String,
    pub verdict: String,
    pub produced_per_s: f64,
    pub scenario: String,
    pub date: String,
}

fn registry() -> &'static [RegistryEntry] {
    static REG: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
    REG.get_or_init(|| {
        serde_json::from_str(include_str!("../../../data/cell-sim-registry.json"))
            .expect("cell-sim-registry.json must parse")
    })
}

/// Stable FNV-1a-64 over the sorted entity list. Deliberately NOT the
/// std hasher (its output is not guaranteed stable across toolchains,
/// and the registry is checked-in data).
pub fn geometry_hash(l: &LayoutResult) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut tuples: Vec<String> = l
        .entities
        .iter()
        .map(|e| {
            format!(
                "{}|{}|{}|{}|{}|{}",
                e.name,
                e.x,
                e.y,
                e.direction as u8,
                e.io_type.as_deref().unwrap_or(""),
                e.carries.as_deref().unwrap_or("")
            )
        })
        .collect();
    tuples.sort_unstable();
    let mut h = OFFSET;
    for t in &tuples {
        for b in t.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(PRIME);
        }
        h ^= 0x1f;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Look up a composed layout's verification status.
pub fn lookup(target: &str, rate: f64, hash: u64) -> Option<&'static RegistryEntry> {
    let hex = format!("{hash:016x}");
    registry().iter().find(|e| {
        e.target == target && (e.rate - rate).abs() < 1e-9 && e.geometry_hash == hex
    })
}

/// The annotation `CellComposedCandidate` attaches to its layouts.
pub fn verification_note(target: &str, rate: f64, l: &LayoutResult) -> String {
    let hash = geometry_hash(l);
    match lookup(target, rate, hash) {
        Some(e) => format!(
            "cell-composed: geometry SIM-VERIFIED at plan ({} — {} produced {:.2}/s, {})",
            e.scenario, e.verdict, e.produced_per_s, e.date
        ),
        None => format!(
            "cell-composed: geometry NOT sim-verified (hash {hash:016x}) — run spaghettio-sim and add the entry to cell-sim-registry.json"
        ),
    }
}
