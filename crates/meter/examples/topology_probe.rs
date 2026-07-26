//! Probe: build the belt network from real exported blueprints and report
//! how well the topology rules cope.
//!
//! Run: `cargo run -p spaghettio_meter --example topology_probe`
//!
//! Regenerate fixtures first (no Factorio needed):
//! ```bash
//! cargo test --manifest-path crates/core/Cargo.toml \
//!     --test cell_composition -- --ignored export_chain_fixtures_for_sim
//! ```

use std::path::PathBuf;

use spaghettio_meter::network::{LaneMap, TileKind};
use spaghettio_meter::{blueprint_in, NetworkBuilder};

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("no fixtures at {dir:?} — generate them first (see module docs)");
        return;
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "bp"))
        .collect();
    files.sort();

    println!(
        "{:<28} {:>6} {:>7} {:>7} {:>8} {:>8} {:>6}",
        "fixture", "tiles", "linked", "edge", "straight", "sideload", "notes"
    );

    for path in files {
        let Ok(bp) = std::fs::read_to_string(&path) else {
            continue;
        };
        let label = path.file_stem().unwrap().to_string_lossy().to_string();
        let ents = match blueprint_in::decode(&bp) {
            Ok(e) => e,
            Err(e) => {
                println!("{label:<28} DECODE FAILED: {e}");
                continue;
            }
        };
        let net = NetworkBuilder::build(&ents);

        let linked = net.tiles.iter().filter(|t| t.downstream.is_some()).count();
        let edge = net.len() - linked;
        let straight = net
            .tiles
            .iter()
            .filter(|t| matches!(t.downstream.map(|d| d.lanes), Some(LaneMap::Straight)))
            .count();
        let sideload = net
            .tiles
            .iter()
            .filter(|t| matches!(t.downstream.map(|d| d.lanes), Some(LaneMap::OntoLane(_))))
            .count();

        println!(
            "{label:<28} {:>6} {:>7} {:>7} {:>8} {:>8} {:>6}",
            net.len(),
            linked,
            edge,
            straight,
            sideload,
            net.notes.len()
        );
        for n in &net.notes {
            println!("    note: {n:?}");
        }

        // Underground pairing detail — the rule most likely to be wrong.
        let ug_in = net
            .tiles
            .iter()
            .filter(|t| t.kind == TileKind::UgInput)
            .count();
        let ug_out = net
            .tiles
            .iter()
            .filter(|t| t.kind == TileKind::UgOutput)
            .count();
        if ug_in != ug_out {
            println!("    UG halves unbalanced: {ug_in} in / {ug_out} out");
        }
    }
}
