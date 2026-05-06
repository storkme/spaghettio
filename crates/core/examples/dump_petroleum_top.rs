//! Dump entities at the top of the processing-unit layout.
//! Used to inspect the petroleum trunk's connectivity around y=1-12.

use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::solver;
use rustc_hash::FxHashSet;

fn main() {
    let inputs: FxHashSet<String> = [
        "iron-plate", "copper-plate", "coal", "water", "crude-oil",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let solver_result =
        solver::solve("processing-unit", 2.0, &inputs, "assembling-machine-3")
            .unwrap();

    let layout = build_bus_layout(&solver_result, LayoutOptions::default()).unwrap();

    println!("Entities in 19 ≤ x ≤ 25, 0 ≤ y ≤ 14:");
    let mut filtered: Vec<_> = layout.entities.iter()
        .filter(|e| (19..=25).contains(&e.x) && (0..=14).contains(&e.y))
        .collect();
    filtered.sort_by_key(|e| (e.x, e.y));
    for e in filtered {
        let dir = format!("{:?}", e.direction);
        let io = e.io_type.as_deref().unwrap_or("");
        let carries = e.carries.as_deref().unwrap_or("");
        println!("  ({:>2},{:>2}) {:24} dir={:5} type={:6} carries={}",
            e.x, e.y, e.name, dir, io, carries);
    }
}
