//! Run advanced-circuit @5/s AM3 from ores + fluids through the full
//! pipeline and report validator error/warning counts. Used to verify
//! the clustering change doesn't regress the AM3 layout.

use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity};
use rustc_hash::FxHashSet;

fn main() {
    let inputs: FxHashSet<String> = [
        "iron-ore", "copper-ore", "coal", "water", "crude-oil",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let solver_result =
        solver::solve("advanced-circuit", 5.0, &inputs, "assembling-machine-3")
            .unwrap();

    let layout = build_bus_layout(&solver_result, LayoutOptions::from_belt_tier(Some("transport-belt"))).unwrap();

    let issues = match validate::validate(&layout, Some(&solver_result), LayoutStyle::Bus) {
        Ok(v) => v,
        Err(e) => {
            let v = e.issues.clone();
            println!("validator returned Err ({} issues)", v.len());
            v
        }
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
