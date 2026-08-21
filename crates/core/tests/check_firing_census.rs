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
//! appear inside refused candidates. This caveat is repeated in the
//! printed header (round 3, #686) so a table-skimmer sees it without
//! reading down to the Interpretation paragraph.
//!
//! Run: cargo test --test check_firing_census -- --ignored --nocapture

use std::collections::BTreeSet;

use rustc_hash::{FxHashMap, FxHashSet};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions, LayoutStrategy};
use spaghettio_core::models::LayoutResult;
use spaghettio_core::{solver, validate};

/// A structural signature of a built layout's entities — name/x/y/
/// direction/recipe, sorted so emission order doesn't matter. Used only to
/// detect when a variant's build is bit-identical to the same fixture's
/// default (round 6, #686): `cells-off`/`hs-off` are gated internally
/// (`decomposition_search.rs`'s `try_cells`/`try_horizontal`) on
/// conditions this hardcoded fixture list may not satisfy for every
/// fixture (chain eligibility, a `DualInput` row), so a variant can be a
/// silent no-op — bit-identical to default — for some or all fixtures,
/// which would otherwise look like a genuinely evaluated, merely-quiet
/// candidate.
type EntitySignature = Vec<(String, i32, i32, u8, Option<String>)>;

fn layout_signature(l: &LayoutResult) -> EntitySignature {
    let mut sig: Vec<_> = l
        .entities
        .iter()
        .map(|e| (e.name.clone(), e.x, e.y, e.direction as u8, e.recipe.clone()))
        .collect();
    sig.sort();
    sig
}

/// Per-category census row.
///
/// Four independent things go wrong if this is read naively, found across
/// #686 rounds 2 through 5:
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
///    carry a `Severity::Error`, so `err-loser`/`loser-only` must not be
///    gated on the any-severity `winner` flag — the two are unrelated
///    facts.
/// 2. Round 2's fix aggregated `winner` and the err sets ACROSS all six
///    fixtures. A category that Error-fires on `di-off` for fixture A but
///    only Warns on `default` for unrelated fixture B would read
///    `winner=true` from B and mask the fixture-A finding.
///    `fixtures_{err,any}_{default,nondefault}` key on fixture identity so
///    both `err-loser` and `loser-only` are computed per-fixture and then
///    unioned, not aggregated blind across fixtures.
/// 3. Per-fixture pairing alone still isn't enough: a fixture whose
///    `default` variant itself REFUSED (returned `Err`, so nothing is in
///    `fixtures_err_default`/`fixtures_any_default` for it) is
///    indistinguishable from one whose default built clean — both show
///    nothing in the "default" set for that fixture. Reading the former
///    as "clean default, so the non-default firing is invisible work"
///    would be wrong: there was no winner for that fixture to hide
///    anything from. `defaults_built` (checked in `err_loser`/
///    `loser_only` below) gates the predicate on the default having
///    actually produced a layout for that fixture.
/// 4. `di-forced` and `partitioned` are user-elected topologies the native
///    `Candidate` search never evaluates on its own (round 3's receipts,
///    reaffirmed round 5) — a category Error-firing ONLY on `partitioned`
///    says nothing about a candidate the search silently refused, because
///    the search never tried that shape at all. `fixtures_any_nondefault`/
///    `fixtures_err_nondefault` therefore only accumulate fixture keys from
///    variants tagged `native_adjacent: true` in the `variants` table below
///    (`di-off`/`cells-off`/`hs-off`, round 6 — the flag lives in the same
///    tuple the loop iterates, so it cannot desync from that table the way
///    a separately-declared parallel list could); `di-forced`/`partitioned`
///    firings still show up in `variants`/`err_variants` and in `count` —
///    visible, just not counted as loser-only/err-loser evidence.
#[derive(Default)]
struct CatRow {
    /// Fired (any severity) on SOME fixture's "default"-options build. NOT
    /// "the layout the engine ships" — `select_best_decomposition` can
    /// still ship an Error-carrying candidate when nothing validates
    /// clean; see point 1 above. Informational column only — no longer
    /// used to derive `loser-only`/`err-loser` (see points 2-3).
    winner: bool,
    /// Every variant name this category fired on, "default" included,
    /// across all fixtures. Unrestricted — includes `di-forced`/
    /// `partitioned` for display, even though those don't feed
    /// `loser-only`/`err-loser` (point 4).
    variants: BTreeSet<&'static str>,
    /// Every variant name where this category fired at `Severity::Error`,
    /// across all fixtures. Informational provenance only, unrestricted
    /// like `variants` above — NOT the err-loser predicate itself, which
    /// needs fixture pairing (point 2), gated on default-build success
    /// (point 3) and restricted to native-adjacent variants (point 4)
    /// rather than a flat union.
    err_variants: BTreeSet<&'static str>,
    /// Fixture keys (`"{item}@{rate}:{machine}"`) where this category
    /// fired (any severity) on that fixture's OWN "default" build.
    fixtures_any_default: BTreeSet<String>,
    /// Fixture keys where this category fired (any severity) on some
    /// NON-default, NATIVE-ADJACENT variant of that fixture (point 4) —
    /// `di-forced`/`partitioned` firings are excluded from this set even
    /// though they're still recorded in `variants` above.
    fixtures_any_nondefault: BTreeSet<String>,
    /// Fixture keys where this category fired at `Severity::Error` on
    /// that fixture's OWN "default" build.
    fixtures_err_default: BTreeSet<String>,
    /// Fixture keys where this category fired at `Severity::Error` on
    /// some NON-default, NATIVE-ADJACENT variant of that fixture (point 4)
    /// — same exclusion as `fixtures_any_nondefault` above.
    fixtures_err_nondefault: BTreeSet<String>,
    /// Total issue count across all builds (all variants, all severities
    /// — including `di-forced`/`partitioned`; see the printed legend).
    count: usize,
}

impl CatRow {
    /// True iff some fixture fired this category (any severity) on a
    /// non-default, native-adjacent variant (point 4, struct doc — the
    /// `fixtures_any_nondefault` input set already excludes `di-forced`/
    /// `partitioned`, gated at insertion time by each variant's own
    /// `native_adjacent` tuple field) while that SAME fixture's own default
    /// build (1) actually produced a layout AND (2) never fired it.
    /// `defaults_built` is the global set of fixture keys whose "default"
    /// variant built successfully — without that gate, a fixture whose
    /// default REFUSED would look identical to one whose default built
    /// clean and simply didn't fire, which is not evidence of anything
    /// (point 3, struct doc).
    fn loser_only(&self, defaults_built: &FxHashSet<String>) -> bool {
        self.fixtures_any_nondefault
            .iter()
            .any(|f| defaults_built.contains(f) && !self.fixtures_any_default.contains(f))
    }

    /// Same shape as `loser_only`, restricted to `Severity::Error` — the
    /// severity candidate refusal actually keys on (point 1, struct doc).
    fn err_loser(&self, defaults_built: &FxHashSet<String>) -> bool {
        self.fixtures_err_nondefault
            .iter()
            .any(|f| defaults_built.contains(f) && !self.fixtures_err_default.contains(f))
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
    // Third field: `native_adjacent` — is this a shape the native
    // `Candidate`-mode search evaluates as one of its own baseline/
    // competing arms (round 3's receipts: decomposition_search.rs's
    // `try_cells`/`try_horizontal`/`try_di` gates, `DirectInsertion::Off`'s
    // doc comment)? `di-forced`/`partitioned` are `false` — genuinely
    // separate, user-elected topologies the default search never tries.
    // Carried in the tuple itself (round 6, #686) rather than a
    // separately-declared list, so the flag cannot desync from the table
    // the loop below actually iterates.
    type Variant = (&'static str, fn(&mut LayoutOptions), bool);
    let variants: &[Variant] = &[
        ("default", |_| {}, false),
        ("di-off", |o| o.direct_insertion = DirectInsertion::Off, true),
        ("di-forced", |o| o.direct_insertion = DirectInsertion::Forced, false),
        ("cells-off", |o| o.cell_composition = CellComposition::Off, true),
        ("hs-off", |o| o.horizontal_candidate = false, true),
        ("partitioned", |o| o.strategy = LayoutStrategy::PartitionedDecomposed, false),
    ];
    let native_adjacent_names: Vec<&str> = variants
        .iter()
        .filter(|(_, _, native_adjacent)| *native_adjacent)
        .map(|(name, _, _)| *name)
        .collect();

    let mut census: FxHashMap<String, CatRow> = FxHashMap::default();
    let mut builds = 0usize;
    let mut refusals_by_variant: FxHashMap<&'static str, usize> = FxHashMap::default();
    // Fixture keys whose "default" variant actually produced a layout —
    // see struct doc point 3. Global (not per-category): whether a
    // fixture's default build succeeded doesn't depend on which category
    // we're looking at.
    let mut defaults_built: FxHashSet<String> = FxHashSet::default();
    // Round 6, #686: mechanical no-op detection. `cells-off`/`hs-off` are
    // gated internally on conditions this hardcoded fixture list may not
    // satisfy for every fixture (`try_cells`'s chain-eligibility check,
    // `try_horizontal`'s DualInput-row requirement — decomposition_
    // search.rs), so a variant can be bit-identical to default for some or
    // all fixtures without that being visible anywhere in the table above.
    // Per-variant: how many non-default builds succeeded, and how many of
    // those were structurally identical to that SAME fixture's default.
    let mut nondefault_builds: FxHashMap<&'static str, usize> = FxHashMap::default();
    let mut noop_count: FxHashMap<&'static str, usize> = FxHashMap::default();

    for &(item, rate, machine, inputs) in fixtures {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve(item, rate, &input_set, machine) else {
            eprintln!("SKIP (no solve): {item}@{rate}");
            continue;
        };
        // Machine tier included (round 4, #686): "{item}@{rate}" alone
        // collides if the fixture list ever gains a second tier at the
        // same item/rate (the module doc invites exactly that extension),
        // which would silently merge two fixtures' per-fixture state and
        // corrupt the err_loser/loser_only pairing this file exists to
        // protect.
        let fixture_key = format!("{item}@{rate}:{machine}");
        // Reset per fixture — no-op comparison is always against THIS
        // fixture's own default, never another fixture's.
        let mut default_sig: Option<EntitySignature> = None;
        for (vname, tweak, native_adjacent) in variants {
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
            if *vname == "default" {
                defaults_built.insert(fixture_key.clone());
                default_sig = Some(layout_signature(&l));
            } else {
                *nondefault_builds.entry(*vname).or_insert(0) += 1;
                let is_noop = default_sig.as_ref().is_some_and(|d| *d == layout_signature(&l));
                if is_noop {
                    *noop_count.entry(*vname).or_insert(0) += 1;
                }
            }
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
                    row.fixtures_any_default.insert(fixture_key.clone());
                } else if *native_adjacent {
                    // di-forced/partitioned still land in `variants` above
                    // (display/provenance) but are excluded here — they
                    // are user-elected topologies the native search never
                    // tries, so firing only there is not loser-only
                    // evidence (point 4, struct doc).
                    row.fixtures_any_nondefault.insert(fixture_key.clone());
                }
                if i.severity == validate::Severity::Error {
                    row.err_variants.insert(*vname);
                    if *vname == "default" {
                        row.fixtures_err_default.insert(fixture_key.clone());
                    } else if *native_adjacent {
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
        "(a category ABSENT below was never observed on a built layout \
         here — indistinguishable from 'only fires on refused \
         candidates'; see Interpretation at the bottom)"
    );
    println!(
        "(count sums every issue across all variants and both severities \
         for that category — it is not per-variant or per-severity; use \
         err-variants/variants for provenance)"
    );
    println!(
        "(loser-only/err-loser flags consider native-adjacent variants \
         only: {}. User-elected topologies still show up in \
         err-variants/variants and count, but are not counted as \
         selection evidence)",
        native_adjacent_names.join(", ")
    );
    println!(
        "{:<32} {:>7} {:>10} {:>9} {:>6}  {:<24}  variants",
        "category", "winner", "loser-only", "err-loser", "count", "err-variants"
    );
    for (cat, row) in &rows {
        // `winner` is informational only (struct doc point 1) — neither
        // column below is `!winner`. Both are computed per fixture, gated
        // on that fixture's default having actually built (points 2-3).
        let loser_only = row.loser_only(&defaults_built);
        let err_loser = row.err_loser(&defaults_built);
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
        "\n=== per-variant no-op check (bit-identical entities vs that \
         fixture's own default; round 6, #686) ==="
    );
    for &(vname, _, native_adjacent) in variants.iter().filter(|t| t.0 != "default") {
        let total = nondefault_builds.get(vname).copied().unwrap_or(0);
        let noop = noop_count.get(vname).copied().unwrap_or(0);
        let tag = if native_adjacent { "native-adjacent" } else { "user-elected" };
        println!("  {vname:<12} {noop}/{total} builds identical to default  ({tag})");
    }
    println!(
        "\nInterpretation: `loser-only`/`err-loser` are computed per \
         fixture, not from the aggregate `winner` flag (round 3, #686): \
         for a fixture whose default build actually succeeded, did a \
         non-default variant of THAT fixture fire (any severity for \
         loser-only, Severity::Error for err-loser) while that fixture's \
         own default did not — unioned across fixtures. A fixture whose \
         default REFUSED contributes nothing to either predicate, because \
         a refused default has no result to have stayed quiet: its \
         absence must not be read as 'clean'. Of the option-toggle \
         variants, cells-off/di-off/hs-off are shapes the native \
         Candidate-mode search already evaluates internally as its own \
         baseline — `decomposition_search.rs`'s NativeCandidate always \
         runs DI-free/cells-free/vertical under the default `Candidate` \
         settings (see the `DirectInsertion::Off` and `try_horizontal` \
         doc comments), so firings confined to those three are \
         native-adjacent, not outside the search. `di-forced` is \
         different: setting `direct_insertion` to `Forced` at the \
         top level (rather than letting `Candidate` mode use `Forced` \
         internally to build ONE competing candidate) stands down the \
         cells/horizontal/DI-candidate arms entirely (`decomposition_\
         search.rs`'s `try_cells`/`try_horizontal`/`try_di` gates all \
         require the outer mode to be non-Forced or exactly Candidate) \
         and bakes DI directly via a path the default search's own \
         DI-candidate arm does not reproduce — so `di-forced` is a \
         user-elected/forced topology, not a shape the native search \
         tries on its own. `partitioned` \
         (LayoutStrategy::PartitionedDecomposed) remains a genuinely \
         separate, user-elected top-level strategy the native search \
         never runs. Since round 5, `loser-only`/`err-loser` are computed \
         ONLY from di-off/cells-off/hs-off firings — `di-forced`/ \
         `partitioned` firings still show up in err-variants/variants/ \
         count but no longer set either flag, because the search never \
         evaluated those shapes to refuse in the first place. Refused \
         builds (Err from build_bus_layout) produce \
         no layout to validate, so no category attribution is possible \
         for them — 'fired on NOTHING evaluated here' cannot distinguish \
         a genuinely inert category from one that only ever appears \
         inside refused candidates (v1 scope caveats in the module doc \
         apply before concluding a quiet category is inert). A firing on \
         a variant identical to default carries no selection evidence, \
         and a variant that is no-op across all fixtures measures \
         nothing here — see the per-variant no-op check above."
    );
}
