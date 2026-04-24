//! One-shot capture script for advanced-circuit @5/s AM3 from ores +
//! fluids. Mirrors the URL:
//!   ?item=advanced-circuit&rate=5&machine=assembling-machine-3&
//!    in=coal,water,crude-oil,iron-ore,copper-ore&belt=transport-belt
//!
//! Run with:
//!   FUCKTORIO_DUMP_REGION_FIXTURE=/tmp/rfx \
//!     cargo run --manifest-path crates/core/Cargo.toml \
//!     --example capture_ac_am3_ore --release
//!
//! Prints the cluster seeds the region solver encounters + writes one
//! fixture per cluster to `$FUCKTORIO_DUMP_REGION_FIXTURE` (when set).

use fucktorio_core::bus::layout::{build_bus_layout, LayoutOptions};
use fucktorio_core::solver;
use rustc_hash::FxHashSet;

fn main() {
    let inputs: FxHashSet<String> = [
        "iron-ore",
        "copper-ore",
        "coal",
        "water",
        "crude-oil",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let solver_result =
        solver::solve("advanced-circuit", 5.0, &inputs, "assembling-machine-3")
            .unwrap_or_else(|e| {
                eprintln!("solver failed: {e}");
                std::process::exit(1);
            });

    let layout = build_bus_layout(&solver_result, LayoutOptions::from_belt_tier(Some("transport-belt")))
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
