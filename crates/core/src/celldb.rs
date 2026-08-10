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

use crate::common::{is_machine_entity, is_splitter, is_surface_belt, is_ug_belt, oriented_entity_dims};
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
    /// Not consumed by `query_unit` here — its consumer is the template
    /// candidate's hard belt-cap filter (RFC-067 P3, the stacked PR), which
    /// maps these to surface tiers and refuses unknown names. Kept on this
    /// type because deriving it belongs with the vocabulary derivation.
    pub belt_tiers: BTreeSet<String>,
}

impl DerivedConstraints {
    pub fn of(entry: &CellEntry) -> Self {
        let mut vocabulary = BTreeSet::new();
        let mut belt_tiers = BTreeSet::new();
        for e in &entry.entities {
            vocabulary.insert(e.name.clone());
            if is_surface_belt(&e.name) || is_ug_belt(&e.name) || is_splitter(&e.name) {
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

/// The seed source fixtures — the single authority consumed by BOTH the
/// seed tool and the drift regression test, so the test's coverage cannot
/// diverge from what the tool seeds (round-2 review: only 1 of 5 entries
/// was drift-protected when the lists lived apart).
pub fn seed_sources() -> Vec<(&'static str, f64, &'static str, Vec<&'static str>, Vec<(&'static str, &'static str)>)> {
    vec![
        (
            "electronic-circuit",
            20.0,
            "assembling-machine-2",
            vec!["iron-ore", "copper-ore"],
            vec![
                ("copper-plate", "electric-furnace"),
                ("iron-plate", "electric-furnace"),
                ("copper-cable", "assembling-machine-2"),
                ("electronic-circuit", "assembling-machine-2"),
            ],
        ),
        (
            "advanced-circuit",
            4.0,
            "assembling-machine-2",
            vec!["iron-plate", "copper-plate", "plastic-bar"],
            vec![("advanced-circuit", "assembling-machine-2")],
        ),
    ]
}

/// Extract a unit-motif fragment from a laid-out entity list: the recipe's
/// machines plus every `row:{recipe}:*` entity, origin-normalized, with
/// ports DERIVED from the belt-in/belt-out/fluid-in segment runs. Returns
/// the entry plus any derivation warnings — a warning is an escape hatch
/// under RFC-067 K67-1, so the seed tool prints them and the drift test
/// asserts there are none.
///
/// Lives in the library (not the seed example) so the regression test can
/// re-extract from a freshly built layout and diff against the stored
/// entry — stored-vs-stored testing cannot see engine drift.
pub fn extract_unit(
    entities: &[PlacedEntity],
    recipe: &str,
    machine: &str,
    provenance: &str,
) -> (Option<CellEntry>, Vec<String>) {
    use crate::common::{dir_to_vec, is_machine_entity};
    let mut warnings = Vec::new();
    let frag: Vec<PlacedEntity> = entities
        .iter()
        .filter(|e| {
            e.recipe.as_deref() == Some(recipe) && is_machine_entity(&e.name)
                || e
                    .segment_id
                    .as_deref()
                    .is_some_and(|s| s.starts_with(&format!("row:{recipe}:")))
        })
        .cloned()
        .collect();
    if frag.is_empty() {
        warnings.push(format!("no entities for {recipe}"));
        return (None, warnings);
    }
    let min_x = frag.iter().map(|e| e.x).min().unwrap();
    let min_y = frag.iter().map(|e| e.y).min().unwrap();
    let mut frag: Vec<PlacedEntity> = frag
        .into_iter()
        .map(|mut e| {
            e.x -= min_x;
            e.y -= min_y;
            // Counts stored, rates derived — enforced at extraction: rate
            // stamps are fixture-specific flow aggregates and committing
            // them is the stale-declared-data failure the schema bans.
            // Field taxonomy, decided not improvised (round-3 review):
            // STRIPPED = per-run measurements (rate). STRUCTURAL, kept =
            // io_type (UG halves), filters/priorities (splitter function),
            // carries (port-check referee), items/quality — module and
            // quality configs are part of the IMPLEMENTATION (they change
            // derived rates via the solver, not stored ones). v1 seeds are
            // bare-machine Normal; a moduled seed is a different entry, not
            // a stripped one.
            e.rate = None;
            e
        })
        .collect();
    frag.sort_by_key(|e| (e.y, e.x, e.name.clone()));

    let count = frag
        .iter()
        .filter(|e| e.recipe.as_deref() == Some(recipe) && is_machine_entity(&e.name))
        .count() as u32;

    let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut tiles = 0u32;
    let (mut max_x, mut max_y) = (0, 0);
    for e in &frag {
        let (w, h) = oriented_entity_dims(&e.name, e.direction);
        for dx in 0..w {
            for dy in 0..h {
                occupied.insert((e.x + dx, e.y + dy));
                max_x = max_x.max(e.x + dx);
                max_y = max_y.max(e.y + dy);
                tiles += 1;
            }
        }
    }

    let mut ports: Vec<Port> = Vec::new();
    let mut seg_items: Vec<(String, String)> = Vec::new();
    // Segment kinds that are structure, not ports. Anything outside this
    // set AND outside the port kinds below warns — a new engine segment
    // kind must be classified deliberately, never silently skipped
    // (round-3 review: fluid-out fell through both lists with no port and
    // no warning, the exact degraded-store case K67-1 exists to block).
    const STRUCTURAL_KINDS: &[&str] = &[
        "machine",
        "inserter-in",
        "inserter-out",
        "trunk",
        "trunk-dive",
        "current-feed",
    ];
    for e in &frag {
        if let Some(s) = e.segment_id.as_deref() {
            let parts: Vec<&str> = s.split(':').collect();
            let kind = parts.get(2).copied().unwrap_or("");
            if parts.len() >= 4
                && (kind == "belt-in" || kind == "fluid-in" || kind == "fluid-out")
            {
                let key = (kind.to_string(), parts[3].to_string());
                if !seg_items.contains(&key) {
                    seg_items.push(key);
                }
            } else if parts.len() >= 3 && kind == "belt-out" {
                // 3-part belt-out defaults its item to the recipe name —
                // correct for every current row template (the product IS
                // the recipe item); check_entry's carries-vs-item port
                // check catches a future mislabel on tagged belts.
                let item = parts.get(3).unwrap_or(&recipe).to_string();
                let key = ("belt-out".to_string(), item);
                if !seg_items.contains(&key) {
                    seg_items.push(key);
                }
            } else if !STRUCTURAL_KINDS.contains(&kind) {
                let w = format!("unhandled segment kind '{kind}' in {s}");
                if !warnings.contains(&w) {
                    warnings.push(w);
                }
            }
        }
    }
    for (kind, item) in &seg_items {
        let run: Vec<&PlacedEntity> = frag
            .iter()
            .filter(|e| {
                e.segment_id.as_deref().is_some_and(|s| {
                    let p: Vec<&str> = s.split(':').collect();
                    p.get(2).is_some_and(|k| k == kind)
                        && (p.get(3).is_some_and(|i| i == item)
                            || (kind == "belt-out" && p.len() == 3))
                })
            })
            .collect();
        let candidates: Vec<(i32, i32)> = match kind.as_str() {
            "belt-in" => run
                .iter()
                .filter(|t| {
                    !run.iter().any(|f| {
                        let (dx, dy) = dir_to_vec(f.direction);
                        (f.x + dx, f.y + dy) == (t.x, t.y)
                    })
                })
                .map(|t| (t.x, t.y))
                .collect(),
            "belt-out" => run
                .iter()
                .filter(|t| {
                    let (dx, dy) = dir_to_vec(t.direction);
                    !occupied.contains(&(t.x + dx, t.y + dy))
                })
                .map(|t| (t.x, t.y))
                .collect(),
            _ => run
                .iter()
                .filter(|t| t.x == 0 || t.x == max_x || t.y == 0 || t.y == max_y)
                .map(|t| (t.x, t.y))
                .collect(),
        };
        // Multiple candidates are genuine (one port per split-row half);
        // only ZERO is an escape hatch.
        if candidates.is_empty() {
            warnings.push(format!("{kind}:{item} derived no port tiles"));
            continue;
        }
        let pk = match kind.as_str() {
            "belt-in" => PortKind::BeltIn,
            "belt-out" => PortKind::BeltOut,
            "fluid-in" => PortKind::PipeIn,
            "fluid-out" => PortKind::PipeOut,
            other => {
                warnings.push(format!("no PortKind mapping for segment kind '{other}'"));
                continue;
            }
        };
        let mut sorted = candidates.clone();
        sorted.sort();
        for (dx, dy) in sorted {
            ports.push(Port { dx, dy, kind: pk, item: item.clone() });
        }
    }

    let entry = CellEntry {
        motif: Motif::Unit {
            recipe: recipe.to_string(),
            machine: machine.to_string(),
            count,
        },
        metrics: Metrics {
            bbox_w: max_x + 1,
            bbox_h: max_y + 1,
            interior_tiles: tiles,
            entity_count: frag.len() as u32,
        },
        entities: frag,
        ports,
        provenance: provenance.to_string(),
        sim_anchor: "unanchored".to_string(),
    };
    (Some(entry), warnings)
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
        let (w, h) = oriented_entity_dims(&e.name, e.direction);
        for dx in 0..w {
            for dy in 0..h {
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
    // Counts stored, rates derived — both halves enforced. A stored rate is
    // stale-by-construction data; a motif count diverging from the fragment
    // silently mis-ranks every query while tests stay green (round-1
    // review, both flagged).
    for e in &entry.entities {
        if e.rate.is_some() {
            issues.push(format!("entity {} at ({},{}) stores a rate", e.name, e.x, e.y));
        }
    }
    if let Motif::Unit { recipe, machine, count } = &entry.motif {
        let derived = entry
            .entities
            .iter()
            .filter(|e| e.recipe.as_deref() == Some(recipe.as_str()) && &e.name == machine)
            .count() as u32;
        if derived != *count {
            issues.push(format!(
                "motif count {count} != {derived} matching machines in fragment"
            ));
        }
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
        // A port must be a real transport tile carrying the declared item —
        // an unoccupied or wrong-item port composes into a dead feed.
        let holder = entry.entities.iter().find(|e| {
            let (w, h) = oriented_entity_dims(&e.name, e.direction);
            p.dx >= e.x && p.dx < e.x + w && p.dy >= e.y && p.dy < e.y + h
        });
        match holder {
            None => issues.push(format!("port at ({},{}) is an empty tile", p.dx, p.dy)),
            Some(e) => {
                if is_machine_entity(&e.name) {
                    issues.push(format!(
                        "port at ({},{}) sits on machine {}, not transport",
                        p.dx, p.dy, e.name
                    ));
                } else if let Some(c) = e.carries.as_deref() {
                    if c != p.item {
                        issues.push(format!(
                            "port at ({},{}) declares {} but tile carries {c}",
                            p.dx, p.dy, p.item
                        ));
                    }
                }
            }
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

    /// The drift test: stored-vs-CURRENT-ENGINE for EVERY seeded entry, over
    /// the same `seed_sources()` the tool consumes, with ELEMENT-WISE
    /// geometry comparison (serialized fragments) — aggregate comparisons
    /// pass a relocated belt or swapped inserter clean, and a
    /// one-fixture test left 4 of 5 entries unprotected (round-2 review,
    /// both 3/3).
    #[test]
    fn every_stored_entry_matches_fresh_engine_extraction() {
        use crate::bus::layout::{self, LayoutOptions};
        let mut checked = 0usize;
        for (item, rate, machine, inputs, targets) in seed_sources() {
            let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
            let sr = crate::solver::solve(item, rate, &input_set, machine)
                .expect("seed source solves");
            let l = layout::build_bus_layout(&sr, LayoutOptions::default())
                .expect("seed source lays out");
            for (recipe, target_machine) in targets {
                let (fresh, warnings) =
                    extract_unit(&l.entities, recipe, target_machine, "test");
                assert!(
                    warnings.is_empty(),
                    "escape hatches re-extracting {recipe}: {warnings:?}"
                );
                let fresh = fresh.expect("extraction succeeds");
                let stored = celldb()
                    .entries
                    .iter()
                    .find(|e| {
                        matches!(&e.motif, Motif::Unit { recipe: r, machine: m, .. }
                                 if r == recipe && m == target_machine)
                    })
                    .unwrap_or_else(|| panic!("{recipe} entry is seeded"));
                assert_eq!(stored.motif, fresh.motif, "{recipe}: motif drifted");
                assert_eq!(stored.metrics, fresh.metrics, "{recipe}: metrics drifted");
                assert_eq!(stored.ports, fresh.ports, "{recipe}: ports drifted");
                // Element-wise geometry: both fragments are sorted by
                // (y, x, name) at extraction, so serialized equality is
                // positional equality of every entity field.
                assert_eq!(
                    serde_json::to_string(&stored.entities).unwrap(),
                    serde_json::to_string(&fresh.entities).unwrap(),
                    "{recipe}: fragment geometry drifted from engine output"
                );
                checked += 1;
            }
        }
        // Scope: ENGINE-seeded entries only. community:/hand entries are
        // first-class per the provenance taxonomy and cannot be reproduced
        // from seed_sources by definition (round-3 review — the unscoped
        // assert would have banned them).
        let engine_entries = celldb()
            .entries
            .iter()
            .filter(|e| e.provenance.starts_with("engine@"))
            .count();
        assert_eq!(
            checked, engine_entries,
            "every ENGINE-seeded entry must be drift-checked (seed_sources out of sync?)"
        );
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
