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
//! A second, orthogonal gap (#686 round 2): a variant whose
//! `build_bus_layout` call returns `Err` (refused) produces no layout at
//! all, so nothing gets validated and no category can be attributed to
//! it. Refusal counts are tracked per-variant (see the printed summary),
//! but WHICH categories would have fired on a refused candidate is
//! invisible at this layer — the real fix is the search's internal
//! candidate loop (same refactor #1 above), not anything this census can
//! do post-hoc. "Fired on NOTHING evaluated here" therefore means "not
//! observed," not "inert" — it is silent on categories that only ever
//! appear inside refused candidates.
//!
//! Run: cargo test --test check_firing_census -- --ignored --nocapture

use std::collections::BTreeSet;

use rustc_hash::{FxHashMap, FxHashSet};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions, LayoutStrategy};
use spaghettio_core::{solver, validate};

/// Per-category census row.
///
/// Two independent things go wrong if this is read naively, both found in
/// #686 round 2 review:
///
/// 1. Candidate refusal (`decomposition_search.rs`) is supposed to key on
///    `Severity::Error` only, so a Warning-only firing on a non-default
///    variant is not by itself evidence of invisible selection work — but
///    `select_best_decomposition`'s error-free tier (`best_error_free_idx`)
///    only WINS the pick when some candidate validates clean; if the whole
///    ranking is error-laden, selection falls through to the best-accepted
///    candidate and ships it anyway (see the comment above
///    `best_error_free_idx` in `decomposition_search.rs`: "still returns
///    the error-laden best rather than refusing"). A default build CAN
///    carry a `Severity::Error`, so `err-loser` must not be gated on the
///    any-severity `winner` flag — the two are unrelated facts.
/// 2. `winner` and a naive `err_variants.is_empty()` check are both
///    accumulated ACROSS all six fixtures. A category that Error-fires on
///    `di-off` for fixture A but only Warns on `default` for unrelated
///    fixture B would read `winner=true` from B and mask the fixture-A
///    finding. `fixtures_err_default` / `fixtures_err_nondefault` key on
///    fixture identity so `err-loser` is computed per-fixture and then
///    unioned, not aggregated blind across fixtures.
#[derive(Default)]
struct CatRow {
    /// Fired (any severity) on SOME fixture's "default"-options build. NOT
    /// "the layout the engine ships" — `select_best_decomposition` can
    /// still ship an Error-carrying candidate when nothing validates
    /// clean; see point 1 above.
    winner: bool,
    /// Every variant name this category fired on, "default" included,
    /// across all fixtures.
    variants: BTreeSet<&'static str>,
    /// Every variant name where this category fired at `Severity::Error`,
    /// across all fixtures. Informational provenance only — NOT the
    /// err-loser predicate itself, which needs fixture pairing (point 2
    /// above) rather than a flat union.
    err_variants: BTreeSet<&'static str>,
    /// Fixture keys (`"{item}@{rate}"`) where this category fired at
    /// `Severity::Error` on that fixture's OWN "default" build.
    fixtures_err_default: BTreeSet<String>,
    /// Fixture keys where this category fired at `Severity::Error` on
    /// some NON-default variant of that fixture.
    fixtures_err_nondefault: BTreeSet<String>,
    /// Total issue count across all builds.
    count: usize,
}

impl CatRow {
    /// True iff some fixture Error-fired this category on a non-default
    /// variant while that SAME fixture's own default build did not —
    /// i.e. invisible-to-selection Error work that a different fixture's
    /// quiet default cannot mask.
    fn err_loser(&self) -> bool {
        self.fixtures_err_nondefault.iter().any(|f| !self.fixtures_err_default.contains(f))
    }
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
        let fixture_key = format!("{item}@{rate}");
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
                    if *vname == "default" {
                        row.fixtures_err_default.insert(fixture_key.clone());
                    } else {
                        row.fixtures_err_nondefault.insert(fixture_key.clone());
                    }
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
        "{:<32} {:>7} {:>10} {:>9} {:>6}  {:<24}  {}",
        "category", "winner", "loser-only", "err-loser", "count", "err-variants", "variants"
    );
    for (cat, row) in &rows {
        // "loser-only" = fired (any severity) on a non-default variant,
        // never on default; `winner` already tracks "fired on SOME
        // fixture's default at any severity", so `!winner` covers the
        // "never on default" half. "err-loser" is NOT `!winner` — see
        // `CatRow::err_loser` and the struct doc: it is computed per
        // fixture so a quiet-on-B default can't mask an Error found on A.
        let loser_only = !row.winner;
        let err_loser = row.err_loser();
        let err_variants_str = row.err_variants.iter().copied().collect::<Vec<_>>().join(",");
        let err_variants_disp = if err_variants_str.is_empty() { "-" } else { &err_variants_str };
        let variants_str = row.variants.iter().copied().collect::<Vec<_>>().join(",");
        println!(
            "{:<32} {:>7} {:>10} {:>9} {:>6}  {:<24}  {}",
            cat,
            if row.winner { "yes" } else { "-" },
            if loser_only { "YES" } else { "-" },
            if err_loser { "YES" } else { "-" },
            row.count,
            err_variants_disp,
            variants_str
        );
    }
    println!(
        "\nInterpretation: default builds are NOT guaranteed error-free — \
         select_best_decomposition ships the best-scoring candidate even \
         when none validates clean, so gating err-loser on the \
         any-severity `winner` flag was wrong (round 2, #686). err-loser \
         is now computed per fixture: YES iff some fixture Error-fired \
         this category on a non-default variant while that SAME \
         fixture's own default build did not. cell_composition and \
         direct_insertion both default to Candidate, so the native \
         search ALREADY tries the cells-off / DI-free shape internally \
         as its own baseline candidate — firings confined to \
         cells-off/di-off/di-forced/hs-off are native-adjacent, not \
         outside the search. Only `partitioned` \
         (LayoutStrategy::PartitionedDecomposed) is a genuinely \
         separate, user-elected top-level strategy the native search \
         never runs. Refused builds (Err from build_bus_layout) produce \
         no layout to validate, so no category attribution is possible \
         for them — 'fired on NOTHING evaluated here' cannot distinguish \
         a genuinely inert category from one that only ever appears \
         inside refused candidates (v1 scope caveats in the module doc \
         apply before concluding a quiet category is inert)."
    );
}
