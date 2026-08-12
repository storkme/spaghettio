//! Seed tool for the cell-interface DB (RFC-067 P1). Regenerates
//! `crates/core/data/celldb.json` from ENGINE OUTPUT — metrics are recorded
//! by the extraction, never typed by hand, and provenance names the fixture
//! and SHA so drift is attributable.
//!
//! The extraction itself lives in `spaghettio_core::celldb::extract_unit`
//! (so the drift regression test can re-extract from a fresh layout and
//! diff against the store); this tool is a thin driver that picks the
//! source fixtures and writes the JSON. Every extraction warning prints as
//! SEED-WARN — each one is an escape hatch under RFC-067 K67-1.
//!
//! Top-5 motifs (Phase-0 census) via two fixtures:
//!   electronic-circuit@20 from ore  -> copper-plate, iron-plate,
//!                                      copper-cable, electronic-circuit
//!   advanced-circuit@4 from plates  -> advanced-circuit
use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::celldb::{extract_unit, CellDb, CellEntry, Motif};
use spaghettio_core::solver;

fn main() {
    // Provenance SHA: explicit SEED_SHA, else the actual HEAD — committing
    // "engine@worktree" identified no real commit, defeating the very
    // attributability provenance exists for (round-2 review, 3/3). A dirty
    // tree is marked as such.
    let sha = std::env::var("SEED_SHA").unwrap_or_else(|_| {
        let head = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "unknown".into());
        let dirty = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .is_some_and(|o| !o.stdout.is_empty());
        if dirty { format!("{head}-dirty") } else { head }
    });
    let mut entries: Vec<CellEntry> = Vec::new();

    let mut total_warnings = 0usize;
    for (item, rate, machine, inputs, targets) in spaghettio_core::celldb::seed_sources() {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve(item, rate, &input_set, machine).expect("seed fixture must solve");
        let l = layout::build_bus_layout(&sr, LayoutOptions::default())
            .expect("seed fixture must lay out");
        let prov = format!("engine@{sha} fixture={item}@{rate}");
        for (recipe, target_machine) in targets {
            let (entry, warnings) = extract_unit(&l.entities, recipe, target_machine, &prov);
            for w in &warnings {
                println!("SEED-WARN {recipe}: {w}");
            }
            total_warnings += warnings.len();
            if let Some(e) = entry {
                println!(
                    "seeded {recipe:<24} count={:<3} bbox={}x{} tiles={} ports={}",
                    match &e.motif {
                        Motif::Unit { count, .. } => *count,
                        _ => 0,
                    },
                    e.metrics.bbox_w,
                    e.metrics.bbox_h,
                    e.metrics.interior_tiles,
                    e.ports.len()
                );
                entries.push(e);
            }
        }
    }

    // K67-1 enforced at the tool, not narrated: a degraded seed must not
    // ship (round-2 review — the gate was announced but the tool wrote and
    // exited 0 regardless).
    if total_warnings > 0 {
        eprintln!(
            "ERROR: {total_warnings} extraction warning(s) — refusing to write a degraded store."
        );
        std::process::exit(2);
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/celldb.json");
    // This tool owns engine@ rows ONLY. Donor entries (community:/hand
    // provenance, RFC-067 donor probe) pass through a re-seed untouched —
    // rebuilding the store from seed_sources() used to silently drop them.
    if let Ok(existing) = std::fs::read_to_string(path) {
        let existing: CellDb = serde_json::from_str(&existing).expect("celldb.json parses");
        entries.extend(
            existing
                .entries
                .into_iter()
                .filter(|e| !e.provenance.starts_with("engine@")),
        );
    }
    let db = CellDb { version: 1, entries };
    std::fs::write(path, serde_json::to_string_pretty(&db).unwrap()).unwrap();
    println!(
        "wrote {} entries to {path}  (escape hatches: 0 — K67-1 clean)",
        db.entries.len()
    );
}
