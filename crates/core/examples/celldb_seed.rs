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
    let sha = std::env::var("SEED_SHA").unwrap_or_else(|_| "worktree".into());
    let mut entries: Vec<CellEntry> = Vec::new();

    let sources: Vec<(&str, f64, &str, Vec<&str>, Vec<(&str, &str)>)> = vec![
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
    ];

    let mut total_warnings = 0usize;
    for (item, rate, machine, inputs, targets) in sources {
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

    let db = CellDb { version: 1, entries };
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/celldb.json");
    std::fs::write(path, serde_json::to_string_pretty(&db).unwrap()).unwrap();
    println!(
        "wrote {} entries to {path}  (escape hatches: {total_warnings} — K67-1 trips above 1)",
        db.entries.len()
    );
}
