//! Run processing-unit @ 2/s AM3 with the user's exact URL config and
//! print all validator findings. Used to verify the
//! `check_fluid_network_connectivity` validator catches the broken trunks.

use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity};
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

    let issues = match validate::validate(&layout, Some(&solver_result), LayoutStyle::Bus) {
        Ok(v) => v,
        Err(e) => e.issues,
    };
    let errors = issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings = issues.iter().filter(|i| i.severity == Severity::Warning).count();

    println!("layout: {}x{} ({} entities)",
        layout.width, layout.height, layout.entities.len());
    println!("validator: {errors} errors, {warnings} warnings");
    for issue in &issues {
        let sev = issue.severity.as_str();
        let loc = match (issue.x, issue.y) {
            (Some(x), Some(y)) => format!(" @ ({x},{y})"),
            _ => String::new(),
        };
        println!("  [{sev}] {}{loc} — {}", issue.category, issue.message);
    }
}
