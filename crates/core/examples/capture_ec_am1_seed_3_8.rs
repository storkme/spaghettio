//! One-shot capture script for the electronic-circuit @ 10/s AM1 layout
//! that exposes a `topology_boundaries` chain-walk gap at seed (3,8).
//!
//! URL:
//!   ?item=electronic-circuit&rate=10&machine=assembling-machine-1&
//!    in=iron-plate,copper-plate,steel-plate,stone,coal,water,crude-oil,
//!       iron-ore,copper-ore
//!
//! Run:
//!   FUCKTORIO_DUMP_REGION_FIXTURE=/tmp/rfx_ec \
//!   FUCKTORIO_DUMP_REGION_FIXTURE_SEED="3,8" \
//!     cargo run --manifest-path crates/core/Cargo.toml \
//!     --example capture_ec_am1_seed_3_8 --release

use fucktorio_core::bus::layout::{build_bus_layout, LayoutOptions};
use fucktorio_core::solver;
use rustc_hash::FxHashSet;

fn main() {
    let inputs: FxHashSet<String> = [
        "iron-plate",
        "copper-plate",
        "steel-plate",
        "stone",
        "coal",
        "water",
        "crude-oil",
        "iron-ore",
        "copper-ore",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let solver_result =
        solver::solve("electronic-circuit", 10.0, &inputs, "assembling-machine-1")
            .unwrap_or_else(|e| {
                eprintln!("solver failed: {e}");
                std::process::exit(1);
            });

    let layout = build_bus_layout(&solver_result, LayoutOptions::default())
        .unwrap_or_else(|e| {
            eprintln!("layout failed: {e}");
            std::process::exit(1);
        });

    println!(
        "layout: {}x{} ({} entities)",
        layout.width,
        layout.height,
        layout.entities.len()
    );
}
