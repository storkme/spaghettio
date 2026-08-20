//! G2 (offpath campaign, #675): the check-firing census — which validator
//! categories fire on layouts the engine EVALUATES, not just the winners
//! it ships. The deletion audit's key negative result was that corpus
//! quietness proves nothing for structural checks, because candidate
//! selection refuses Error-carrying candidates invisibly; this instrument
//! makes that work observable, so a future "can we ditch check X?"
//! question starts from data.
//!
//! v1 SCOPE, stated honestly: the candidate field is approximated by the
//! public option-toggle family (native / DI-off / cells-off /
//! HS-off / DI-forced / partitioned), not by the search's internal
//! candidate list — k1-shape-fix and size-split members only exist inside
//! `select_best_decomposition` on native failure and are not re-enacted
//! here. If selection-unification lands (the audit's refactor #1), this
//! census should move onto the real candidate loop and drop the
//! approximation. Fixture list is a hardcoded tier-ladder slice for the
//! same reason; extend as needed.
//!
//! Run: cargo test --test check_firing_census -- --ignored --nocapture

use rustc_hash::{FxHashMap, FxHashSet};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions, LayoutStrategy};
use spaghettio_core::{solver, validate};

#[test]
#[ignore = "G2 diagnostic census — run with --ignored --nocapture"]
fn check_firing_census() {
    let fixtures: &[(&str, f64, &str, &[&str])] = &[
        ("iron-gear-wheel", 10.0, "assembling-machine-1", &["iron-plate"]),
        ("electronic-circuit", 10.0, "assembling-machine-1", &["iron-ore", "copper-ore"]),
        ("electronic-circuit", 30.0, "assembling-machine-2", &["iron-ore", "copper-ore"]),
        ("plastic-bar", 5.0, "chemical-plant", &["coal", "water", "crude-oil"]),
        (
            "advanced-circuit",
            5.0,
            "assembling-machine-2",
            &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        ),
        (
            "processing-unit",
            2.0,
            "assembling-machine-3",
            &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        ),
    ];
    let variants: &[(&str, fn(&mut LayoutOptions))] = &[
        ("default", |_| {}),
        ("di-off", |o| o.direct_insertion = DirectInsertion::Off),
        ("di-forced", |o| o.direct_insertion = DirectInsertion::Forced),
        ("cells-off", |o| o.cell_composition = CellComposition::Off),
        ("hs-off", |o| o.horizontal_candidate = false),
        ("partitioned", |o| o.strategy = LayoutStrategy::PartitionedDecomposed),
    ];

    // category -> (fired on the default/winner build, fired on any
    // non-default variant, total issue count across all builds)
    let mut census: FxHashMap<String, (bool, bool, usize)> = FxHashMap::default();
    let mut builds = 0usize;
    let mut refusals = 0usize;

    for &(item, rate, machine, inputs) in fixtures {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve(item, rate, &input_set, machine) else {
            eprintln!("SKIP (no solve): {item}@{rate}");
            continue;
        };
        for (vname, tweak) in variants {
            let mut opts = LayoutOptions::default();
            tweak(&mut opts);
            let l = match layout::build_bus_layout(&sr, opts) {
                Ok(l) => l,
                Err(_) => {
                    refusals += 1;
                    continue;
                }
            };
            builds += 1;
            let issues = match validate::validate(&l, Some(&sr)) {
                Ok(i) => i,
                Err(e) => e.issues,
            };
            for i in &issues {
                let e = census.entry(i.category.clone()).or_insert((false, false, 0));
                if *vname == "default" {
                    e.0 = true;
                } else {
                    e.1 = true;
                }
                e.2 += 1;
            }
        }
    }

    let mut rows: Vec<_> = census.into_iter().collect();
    rows.sort_by(|a, b| b.1 .2.cmp(&a.1 .2));
    println!("\n=== check-firing census: {builds} builds, {refusals} refusals ===");
    println!("{:<32} {:>7} {:>10} {:>6}", "category", "winner", "loser-only", "count");
    for (cat, (on_default, on_variant, n)) in &rows {
        let loser_only = *on_variant && !*on_default;
        println!(
            "{:<32} {:>7} {:>10} {:>6}",
            cat,
            if *on_default { "yes" } else { "-" },
            if loser_only { "YES" } else { "-" },
            n
        );
    }
    println!(
        "\nInterpretation: a category with loser-only=YES does invisible \
         selection work — quietness on the shipped corpus proves nothing \
         about it. Categories absent entirely fired on NOTHING evaluated \
         here (v1 scope caveats in the module doc apply before concluding \
         they are inert)."
    );
}
