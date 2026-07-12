//! Validate the user's processing-unit @ 2/s URL config (vertical-split,
//! NOT horizontal-stack) and dump all issues by category.

use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity};
use rustc_hash::FxHashSet;
use std::collections::BTreeMap;

fn main() {
    let inputs: FxHashSet<String> = [
        "iron-plate","copper-plate","steel-plate","stone","coal",
        "water","crude-oil","iron-ore","copper-ore",
    ].iter().map(|s| s.to_string()).collect();
    let sr = solver::solve("processing-unit", 2.0, &inputs, "assembling-machine-2").unwrap();
    let opts = LayoutOptions {
        surplus_policy: Default::default(),
        max_belt_tier: Some("fast-transport-belt".to_string()),
        max_inserter_tier: Default::default(),
        ..Default::default()
    };
    let layout = build_bus_layout(&sr, opts).unwrap();

    let issues = match validate::validate(&layout, Some(&sr), LayoutStyle::Bus) {
        Ok(v) => v,
        Err(e) => e.issues,
    };

    let mut errs = 0;
    let mut warns = 0;
    let mut by_cat: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for i in &issues {
        if matches!(i.severity, Severity::Error) { errs += 1; }
        else { warns += 1; }
        let prefix = match i.severity {
            Severity::Error => "ERR ",
            Severity::Warning => "WARN",
        };
        by_cat.entry(i.category.clone()).or_default()
            .push(format!("[{prefix}] {}", i.message));
    }
    println!("Total: {} errors, {} warnings\n", errs, warns);
    for (cat, msgs) in &by_cat {
        println!("== {} ({}) ==", cat, msgs.len());
        for m in msgs.iter().take(20) {
            println!("  {}", m);
        }
        if msgs.len() > 20 {
            println!("  ... +{} more", msgs.len() - 20);
        }
        println!();
    }
}
