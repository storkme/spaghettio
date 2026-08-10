//! K67-2's instrument: how honest is the interface-first preview?
//!
//! For every corpus entry that solves and lays out, compare the preview's
//! estimated total area against the realized layout's bounding-box area,
//! and print the per-fixture ratio plus the median absolute error. The RFC
//! adjudicates K67-2 on this output: median error > 30% ships the preview
//! disabled.
//!
//! Corpus: same entries as the Phase-0 probes. Inlined here until #617
//! lands on main and the branches converge on the shared
//! `examples/celldb/corpus.rs` (noted debt, not a fork — the list below is
//! the solver-relevant projection of that file at fcc40671).
use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::preview::preview_boxes;
use spaghettio_core::solver;

struct F(&'static str, f64, &'static str, Option<&'static str>, &'static [&'static str], &'static [&'static str]);

fn corpus() -> Vec<F> {
    vec![
        F("iron-gear-wheel", 10.0, "assembling-machine-1", None, &["iron-plate"], &[]),
        F("iron-gear-wheel", 10.0, "assembling-machine-2", None, &["iron-ore"], &[]),
        F("iron-gear-wheel", 20.0, "assembling-machine-2", None, &["iron-plate"], &[]),
        F("electronic-circuit", 10.0, "assembling-machine-2", None, &["iron-plate", "copper-plate"], &[]),
        F("electronic-circuit", 10.0, "assembling-machine-1", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 20.0, "assembling-machine-2", None, &["iron-ore", "copper-ore"], &[]),
        F("plastic-bar", 10.0, "chemical-plant", None, &["petroleum-gas", "coal"], &[]),
        F("plastic-bar", 10.0, "chemical-plant", None, &["crude-oil", "coal"], &[]),
        F("sulfuric-acid", 5.0, "chemical-plant", None, &["iron-plate", "sulfur", "water"], &[]),
        F("light-oil", 5.0, "chemical-plant", None, &["water", "heavy-oil"], &["advanced-oil-processing", "coal-liquefaction"]),
        F("petroleum-gas", 12.0, "oil-refinery", None, &["water", "crude-oil"], &[]),
        F("petroleum-gas", 24.0, "oil-refinery", None, &["water", "crude-oil"], &["basic-oil-processing", "coal-liquefaction"]),
        F("advanced-circuit", 1.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("advanced-circuit", 5.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], &[]),
        F("processing-unit", 2.0, "assembling-machine-3", Some("fast-transport-belt"), &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], &[]),
        F("uranium-235", 0.1, "assembling-machine-3", None, &["uranium-238"], &["uranium-processing"]),
        F("uranium-235", 0.05, "assembling-machine-3", None, &["uranium-ore"], &["kovarex-enrichment-process"]),
        F("pentapod-egg", 0.2, "assembling-machine-3", None, &["nutrients", "water"], &[]),
        F("raw-fish", 0.15, "assembling-machine-3", Some("fast-transport-belt"), &["nutrients", "water"], &[]),
        F("iron-bacteria", 1.0, "assembling-machine-3", None, &["bioflux"], &["iron-bacteria"]),
        F("electronic-circuit", 30.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("advanced-circuit", 45.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "plastic-bar"], &[]),
        F("advanced-circuit", 5.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("advanced-circuit", 4.0, "assembling-machine-2", None, &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("electronic-circuit", 60.0, "assembling-machine-2", Some("fast-transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 22.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 23.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 35.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 40.0, "assembling-machine-2", Some("transport-belt"), &["iron-ore", "copper-ore"], &[]),
    ]
}

fn main() {
    let mut errs: Vec<f64> = Vec::new();
    println!("{:<28} {:>10} {:>10} {:>8}", "fixture", "preview", "realized", "ratio");
    for F(item, rate, machine, belt, inputs, excluded) in corpus() {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let excl: FxHashSet<String> = excluded.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve_with_exclusions(item, rate, &input_set, machine, &excl)
        else {
            continue;
        };
        let p = preview_boxes(&sr, belt);
        let opts = LayoutOptions { max_belt_tier: belt.map(|s| s.to_string()), ..Default::default() };
        let Ok(l) = layout::build_bus_layout(&sr, opts) else {
            // A fixture whose realization fails is a MAXIMAL preview error,
            // not a skip — dropping it excluded exactly the hardest cases
            // from the K67-2 median and biased the gate toward PASS
            // (round-3 review on this PR).
            println!("{item}@{rate}: layout failed — counted as 100% error");
            errs.push(1.0);
            continue;
        };
        let pa = (p.width as f64) * (p.height as f64);
        let ra = (l.width as f64) * (l.height as f64);
        let ratio = pa / ra;
        errs.push((ratio - 1.0).abs());
        println!("{:<28} {:>10.0} {:>10.0} {:>8.2}", format!("{item}@{rate}"), pa, ra, ratio);
    }
    if errs.is_empty() {
        println!("\nK67-2: NO DATA — every corpus entry failed to solve or lay out.");
        return;
    }
    errs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = if errs.len() % 2 == 1 {
        errs[errs.len() / 2]
    } else {
        (errs[errs.len() / 2 - 1] + errs[errs.len() / 2]) / 2.0
    };
    println!(
        "\nK67-2: median |error| = {:.1}%  (n={}, threshold 30%) -> {}",
        100.0 * med,
        errs.len(),
        if med <= 0.30 { "PASS" } else { "FAIL — preview ships disabled" }
    );
}
