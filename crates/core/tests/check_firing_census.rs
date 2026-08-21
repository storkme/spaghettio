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

use std::collections::BTreeSet;

use rustc_hash::{FxHashMap, FxHashSet};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions, LayoutStrategy};
use spaghettio_core::{solver, validate};

/// Per-category census row. `winner` / `variants` / `err_variants` are kept
/// separate rather than folded into one flag because candidate refusal
/// (`decomposition_search.rs`) keys on `Severity::Error` only — a category
/// that fires Warnings on a non-default variant but never an Error is
/// invisible to selection, and must not be conflated with one that does.
#[derive(Default)]
struct CatRow {
    /// Fired (any severity) on the "default" variant — the one the engine
    /// actually ships.
    winner: bool,
    /// Every variant name this category fired on, "default" included.
    variants: BTreeSet<&'static str>,
    /// Variant names where this category fired at `Severity::Error`.
    err_variants: BTreeSet<&'static str>,
    /// Total issue count across all builds.
    count: usize,
}

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

    let mut census: FxHashMap<String, CatRow> = FxHashMap::default();
    let mut builds = 0usize;
    let mut refusals_by_variant: FxHashMap<&'static str, usize> = FxHashMap::default();

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
                    *refusals_by_variant.entry(*vname).or_insert(0) += 1;
                    continue;
                }
            };
            builds += 1;
            let issues = match validate::validate(&l, Some(&sr)) {
                Ok(i) => i,
                Err(e) => e.issues,
            };
            for i in &issues {
                let row = census.entry(i.category.clone()).or_default();
                row.variants.insert(*vname);
                row.count += 1;
                if *vname == "default" {
                    row.winner = true;
                }
                if i.severity == validate::Severity::Error {
                    row.err_variants.insert(*vname);
                }
            }
        }
    }

    let mut rows: Vec<_> = census.into_iter().collect();
    rows.sort_by(|a, b| b.1.count.cmp(&a.1.count).then_with(|| a.0.cmp(&b.0)));

    let refusal_summary = if refusals_by_variant.is_empty() {
        "none".to_string()
    } else {
        let mut entries: Vec<_> = refusals_by_variant.iter().collect();
        entries.sort_by_key(|(name, _)| **name);
        entries.iter().map(|(name, n)| format!("{name}={n}")).collect::<Vec<_>>().join(" ")
    };
    println!("\n=== check-firing census: {builds} builds, refusals: {refusal_summary} ===");
    println!(
        "{:<32} {:>7} {:>10} {:>9} {:>6}  {}",
        "category", "winner", "loser-only", "err-loser", "count", "variants"
    );
    for (cat, row) in &rows {
        // "loser-only" and "err-loser" both read as "fired on a non-default
        // variant, never on default" — the difference is which severity is
        // required on that non-default firing. `winner` already tracks
        // "fired on default at any severity", so `!winner` alone covers the
        // "never on default" half for both columns.
        let loser_only = !row.winner;
        let err_loser = !row.winner && !row.err_variants.is_empty();
        let variants_str = row.variants.iter().copied().collect::<Vec<_>>().join(",");
        println!(
            "{:<32} {:>7} {:>10} {:>9} {:>6}  {}",
            cat,
            if row.winner { "yes" } else { "-" },
            if loser_only { "YES" } else { "-" },
            if err_loser { "YES" } else { "-" },
            row.count,
            variants_str
        );
    }
    println!(
        "\nInterpretation: candidate refusal keys on Severity::Error only \
         (decomposition_search refuses Error-carrying candidates; warnings \
         pass), so only err-loser=YES categories are evidence of invisible \
         selection work. loser-only=YES with warnings alone means the \
         category fires on shapes the corpus never ships — useful for \
         coverage, silent on selection. The variants column says WHICH \
         shapes: cells-off/partitioned are user-elected or alternate-path \
         builds the native search does not enumerate (v1 approximation, \
         module doc), so firings confined to them are 'fired on a \
         non-default pipeline', not 'refused within the search'. \
         Categories absent entirely fired on NOTHING evaluated here (v1 \
         scope caveats in the module doc apply before concluding they are \
         inert)."
    );
}
