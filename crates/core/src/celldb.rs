//! The cell-interface database (RFC-067): an in-repo store of production
//! subtree implementations keyed by demand motif.
//!
//! Schema commitments (argued in the RFC, enforced here):
//! - **Counts stored, rates derived.** An entry never records a rate; the
//!   current solver derives rates from counts at lookup time, so the store
//!   cannot go stale when the rate model changes.
//! - **Constraints derived, never declared.** An entry's requirements
//!   (entity vocabulary, belt tiers) are computed from its own entities by
//!   [`DerivedConstraints::of`]. A declared field can lie; a derived one
//!   cannot outlive its state.
//! - **Metrics recorded by the seed tool** (`examples/celldb_seed.rs`),
//!   not typed by hand; the regression test re-derives them and diffs.
//!
//! The store ships embedded (`data/celldb.json`, `include_str!`) on the
//! `recipes.json` / balancer-library pattern: versioned, diffable, no
//! infrastructure.

use crate::common::{entity_size, is_surface_belt, is_ug_belt};
use crate::models::PlacedEntity;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Motif {
    /// One recipe on one machine type, `count` machines.
    Unit { recipe: String, machine: String, count: u32 },
    /// A fused producer→consumer pair (the DI-cell shape).
    Fused { recipe_a: String, recipe_b: String, count_a: u32, count_b: u32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PortKind {
    BeltIn,
    BeltOut,
    PipeIn,
    PipeOut,
}

/// A declared flow port on the fragment boundary. Tiles are relative to the
/// fragment origin (its min-x/min-y corner after normalization).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Port {
    pub dx: i32,
    pub dy: i32,
    pub kind: PortKind,
    pub item: String,
}

/// Recorded by the seed tool; re-derived and diffed by the regression test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metrics {
    pub bbox_w: i32,
    pub bbox_h: i32,
    pub interior_tiles: u32,
    pub entity_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellEntry {
    pub motif: Motif,
    /// Fragment entities, coordinates relative to the fragment origin.
    pub entities: Vec<PlacedEntity>,
    pub ports: Vec<Port>,
    pub metrics: Metrics,
    /// `engine@<sha> fixture=<name>` | `community:<source>` | `hand`.
    pub provenance: String,
    /// `unanchored` until a sim run blesses it (`anchored@<sha> <rate>/s`).
    pub sim_anchor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellDb {
    pub version: u32,
    pub entries: Vec<CellEntry>,
}

/// What an entry NEEDS — computed from its entities, never stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedConstraints {
    /// Every entity prototype the fragment places. "Tech level" is exactly
    /// this set: filter with `is_satisfied_by`.
    pub vocabulary: BTreeSet<String>,
    /// Belt tiers used (prototype names of surface/UG belts and splitters).
    pub belt_tiers: BTreeSet<String>,
}

impl DerivedConstraints {
    pub fn of(entry: &CellEntry) -> Self {
        let mut vocabulary = BTreeSet::new();
        let mut belt_tiers = BTreeSet::new();
        for e in &entry.entities {
            vocabulary.insert(e.name.clone());
            if is_surface_belt(&e.name) || is_ug_belt(&e.name) || e.name.contains("splitter") {
                belt_tiers.insert(e.name.clone());
            }
        }
        DerivedConstraints { vocabulary, belt_tiers }
    }

    /// Dominance test: every entity the fragment places is allowed.
    pub fn is_satisfied_by(&self, allowed: &FxHashSet<String>) -> bool {
        self.vocabulary.iter().all(|v| allowed.contains(v))
    }
}

pub fn celldb() -> &'static CellDb {
    static DB: OnceLock<CellDb> = OnceLock::new();
    DB.get_or_init(|| {
        serde_json::from_str(include_str!("../data/celldb.json"))
            .expect("embedded celldb.json must parse — the seed tool wrote it")
    })
}

/// Unit-motif lookup: entries for `recipe` on `machine` with at least
/// `min_count` machines, whose derived vocabulary is allowed (pass `None`
/// to skip the constraint filter). Ranked smallest-sufficient-first:
/// ascending count, then ascending interior tiles.
pub fn query_unit<'a>(
    recipe: &str,
    machine: &str,
    min_count: u32,
    allowed: Option<&FxHashSet<String>>,
) -> Vec<&'a CellEntry> {
    let mut hits: Vec<&CellEntry> = celldb()
        .entries
        .iter()
        .filter(|e| match &e.motif {
            Motif::Unit { recipe: r, machine: m, count } => {
                r == recipe && m == machine && *count >= min_count
            }
            Motif::Fused { .. } => false,
        })
        .filter(|e| match allowed {
            Some(a) => DerivedConstraints::of(e).is_satisfied_by(a),
            None => true,
        })
        .collect();
    hits.sort_by_key(|e| {
        let c = match &e.motif {
            Motif::Unit { count, .. } => *count,
            Motif::Fused { count_a, count_b, .. } => count_a + count_b,
        };
        (c, e.metrics.interior_tiles)
    });
    hits
}

/// Structural invariants every entry must hold. Returns human-readable
/// violations; the test suite asserts the list is empty for every embedded
/// entry, so a bad seed is a build failure, not a runtime surprise.
pub fn check_entry(entry: &CellEntry) -> Vec<String> {
    let mut issues = Vec::new();
    if entry.entities.is_empty() {
        issues.push("entry has no entities".into());
        return issues;
    }
    // Footprint overlap + bbox from entity sizes.
    let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    let mut tiles = 0u32;
    for e in &entry.entities {
        let (w, h) = entity_size(&e.name);
        for dx in 0..w as i32 {
            for dy in 0..h as i32 {
                let t = (e.x + dx, e.y + dy);
                if !occupied.insert(t) {
                    issues.push(format!("overlap at {t:?} ({})", e.name));
                }
                min_x = min_x.min(t.0);
                min_y = min_y.min(t.1);
                max_x = max_x.max(t.0);
                max_y = max_y.max(t.1);
                tiles += 1;
            }
        }
    }
    if (min_x, min_y) != (0, 0) {
        issues.push(format!("fragment not normalized to origin (min = {min_x},{min_y})"));
    }
    let (bw, bh) = (max_x - min_x + 1, max_y - min_y + 1);
    if (bw, bh) != (entry.metrics.bbox_w, entry.metrics.bbox_h) {
        issues.push(format!(
            "metrics bbox {}x{} != derived {bw}x{bh}",
            entry.metrics.bbox_w, entry.metrics.bbox_h
        ));
    }
    if tiles != entry.metrics.interior_tiles {
        issues.push(format!(
            "metrics interior_tiles {} != derived {tiles}",
            entry.metrics.interior_tiles
        ));
    }
    if entry.entities.len() as u32 != entry.metrics.entity_count {
        issues.push(format!(
            "metrics entity_count {} != derived {}",
            entry.metrics.entity_count,
            entry.entities.len()
        ));
    }
    if entry.ports.is_empty() {
        issues.push("entry declares no ports".into());
    }
    for p in &entry.ports {
        let on_edge =
            p.dx == min_x || p.dx == max_x || p.dy == min_y || p.dy == max_y;
        if !on_edge {
            issues.push(format!("port at ({},{}) not on fragment boundary", p.dx, p.dy));
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_db_parses_and_entries_hold_invariants() {
        let db = celldb();
        assert!(db.version >= 1);
        for (i, e) in db.entries.iter().enumerate() {
            let issues = check_entry(e);
            assert!(
                issues.is_empty(),
                "entry {i} ({:?}) violates invariants: {issues:?}",
                e.motif
            );
            let dc = DerivedConstraints::of(e);
            assert!(!dc.vocabulary.is_empty());
        }
    }

    #[test]
    fn query_ranks_smallest_sufficient_first() {
        let db = celldb();
        // Property over whatever is seeded: results are count-ascending and
        // every result satisfies the min_count bound.
        for e in &db.entries {
            if let Motif::Unit { recipe, machine, count } = &e.motif {
                let hits = query_unit(recipe, machine, *count, None);
                assert!(!hits.is_empty());
                let counts: Vec<u32> = hits
                    .iter()
                    .map(|h| match &h.motif {
                        Motif::Unit { count, .. } => *count,
                        Motif::Fused { count_a, count_b, .. } => count_a + count_b,
                    })
                    .collect();
                assert!(counts.windows(2).all(|w| w[0] <= w[1]));
                assert!(counts.iter().all(|c| c >= count));
            }
        }
    }

    #[test]
    fn constraint_filter_excludes_disallowed_vocabulary() {
        let db = celldb();
        let Some(e) = db.entries.iter().find_map(|e| match &e.motif {
            Motif::Unit { recipe, machine, count } => Some((recipe, machine, *count)),
            _ => None,
        }) else {
            return; // empty DB: nothing to test yet, invariant test covers shape
        };
        let empty: FxHashSet<String> = FxHashSet::default();
        assert!(query_unit(e.0, e.1, e.2, Some(&empty)).is_empty());
    }
}
