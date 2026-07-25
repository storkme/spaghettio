//! Measure a real exported factory end to end.
//!
//! ```bash
//! cargo run --release -p spaghettio_meter --example measure -- chain-ec15-d2
//! cargo run --release -p spaghettio_meter --example measure          # all
//! ```
//!
//! Fixtures come from the engine, no Factorio needed:
//! ```bash
//! cargo test --manifest-path crates/core/Cargo.toml \
//!     --test cell_composition -- --ignored export_chain_fixtures_for_sim
//! ```

use std::path::PathBuf;
use std::time::Instant;

use spaghettio_meter::{Factory, Manifest};

const WARMUP: u64 = 60 * 60 * 2; // 2 game-minutes
const WINDOW: u64 = 60 * 60 * 3; // 3 game-minutes

fn main() {
    let want = std::env::args().nth(1);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("no fixtures at {dir:?} — generate them first (see module docs)");
        return;
    };
    let mut labels: Vec<String> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "bp"))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    labels.sort();
    if let Some(w) = &want {
        labels.retain(|l| l == w);
    }

    for label in labels {
        let bp = match std::fs::read_to_string(dir.join(format!("{label}.bp"))) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let manifest = match Manifest::from_path(dir.join(format!("{label}.manifest.json"))) {
            Ok(m) => m,
            Err(e) => {
                println!("{label}: no manifest ({e})");
                continue;
            }
        };

        let started = Instant::now();
        let mut f = match Factory::build(&bp, manifest) {
            Ok(f) => f,
            Err(e) => {
                println!("{label}: BUILD FAILED: {e}");
                continue;
            }
        };
        let entities = f.net.len() + f.machines.len() + f.inserters.len();
        let report = f.measure(WARMUP, WINDOW);
        let wall = started.elapsed();

        println!(
            "\n=== {label}  ({} belt tiles, {} machines, {} inserters)  {:.2}s wall for {} ticks",
            f.net.len(),
            f.machines.len(),
            f.inserters.len(),
            wall.as_secs_f64(),
            WARMUP + WINDOW
        );
        let _ = entities;

        let mut rows: Vec<(String, f64, f64, f64)> = report
            .planned_per_s
            .iter()
            .map(|(item, planned)| {
                let got = report.produced_per_s.get(item).copied().unwrap_or(0.0);
                let delta = if *planned > 0.0 {
                    (got / planned - 1.0) * 100.0
                } else {
                    0.0
                };
                (item.clone(), *planned, got, delta)
            })
            .collect();
        rows.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        println!("  {:<26} {:>9} {:>9} {:>8}", "item", "planned/s", "meter/s", "d%");
        for (item, planned, got, delta) in &rows {
            println!("  {item:<26} {planned:>9.2} {got:>9.2} {delta:>7.1}%");
        }
        for (item, rate) in &report.delivered_per_s {
            println!("  delivered: {item} {rate:.2}/s");
        }
        println!("  census: {:?}", report.machine_census);
        if report.boundary_refusals > 0 {
            println!("  boundary refusals: {}", report.boundary_refusals);
        }
        for n in report.notes.iter().take(6) {
            println!("  note: {n}");
        }
        if report.notes.len() > 6 {
            println!("  ... and {} more notes", report.notes.len() - 6);
        }
    }
}
