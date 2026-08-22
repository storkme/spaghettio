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
//! FIVE tests live here. Two are `#[ignore]`d diagnostics over the same
//! `FIXTURES` slice, running in opposite directions; the other THREE are
//! NOT ignored and are what a build depends on — they share
//! `assert_scoreboard_contract` and pin the Phase-0b oracle's contract
//! (every candidate slot emits a row, in canonical order, before its
//! terminal event; the winner is one of its own block's rows; the
//! deciding stage is the one this fixture reaches) so that a broken stage
//! tag or a row that stops being emitted fails CI instead of quietly
//! printing wrong output to a human who may never run it. One fixture
//! each for `best-error-free`, `merge-tap` and `scoped-pairwise`, because
//! a contract pinned on ONE stage cannot tell a broken stage tag from a
//! stage that never fires (W1c, #689).
//!
//! Of the two diagnostics: `check_firing_census` (this one) approximates
//! the candidate field from OUTSIDE via option toggles and reports
//! validator CATEGORIES. `selection_scoreboard_census` (RFC-070 Phase 0b,
//! bottom of the file) reads the REAL internal loop under default options
//! and reports candidates, verdicts and the deciding precedence stage —
//! it closes the "which candidate" approximation named just above while
//! saying nothing about categories, so neither replaces the other.
//!
//! Run: cargo test --test check_firing_census -- --ignored --nocapture

use std::collections::BTreeSet;

use rustc_hash::{FxHashMap, FxHashSet};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions, LayoutStrategy};
use spaghettio_core::models::LayoutResult;
use spaghettio_core::{solver, validate};

/// A structural signature of a built layout's entities, sorted so
/// emission order doesn't matter. Used only to
/// detect when a variant's build is identical to the same fixture's
/// default (round 6, #686): `cells-off`/`hs-off` are gated internally
/// (`decomposition_search.rs`'s `try_cells`/`try_horizontal`) on
/// conditions this hardcoded fixture list may not satisfy for every
/// fixture (chain eligibility, a `DualInput` row), so a variant can be a
/// silent no-op — producing the same layout as default — for some or all
/// fixtures, which would otherwise look like a genuinely evaluated,
/// merely-quiet candidate. What "identical" means here is defined
/// precisely below, and it is NOT "validates the same".
///
/// Field order: name, x, y, direction, recipe, carries, mirror, rate-bits.
/// The last three joined in as the #675 follow-up recorded on #686's
/// closing comment, and they are NOT equally load-bearing:
///
/// - `carries` and `mirror` are read by the validator directly
///   (`validate/*.rs` reads `e.carries` across nine check modules;
///   `fluids.rs` passes `e.mirror` into `fluid_ports`), so two layouts
///   differing only there genuinely validate differently — omitting them
///   let the "identical" label mean less than it claimed.
/// - `rate` is NOT read by any validator or engine decision. Receipts,
///   since two reviews have now claimed opposite things: all 21 `.rate`
///   reads across `validate/` and `connectivity.rs` are on solver
///   `ItemFlow`s (`o`/`i`/`f`/`out`/`inp`/`sur`/`flow`/`ext`), none on a
///   `PlacedEntity`; the three `belt_flow.rs` lines #686 round 7 cited as
///   proof (`:684`, `:1675`, `:3286`) read `e.carries`, `e.carries` and
///   `build_ug_pairs` respectively. `docs/rate-stamp-semantics.md` and
///   the `PlacedEntity::rate` doc say the same. It is in the signature
///   anyway because a differing stamp means the pipeline made a different
///   lane-family decision on the way to the same tiles — which is worth
///   not calling "identical" — and that choice REDEFINES the label:
///
/// **A "no-op" here means tiles AND stamps identical, not
/// validator-identical.** The two differ only if `rate` can vary
/// independently of geometry; where it does, this reports the variant as
/// doing something when the validator would not care (#692 review round
/// 2, 1/3). Measured on the current fixture set the redefinition changes
/// nothing — the ratios are identical to the pre-`rate` run (di-off 5/6,
/// di-forced 5/6, cells-off 6/6, hs-off 5/6, partitioned 4/6) — so
/// nothing here is currently reported noisy on a stamp alone.
///
/// `f64` has no `Ord`, so the rate travels as `to_bits`: exact equality
/// is all this needs, and the sort only wants a total order, not a
/// numerically meaningful one.
type EntitySignature = Vec<(String, i32, i32, u8, Option<String>, Option<String>, bool, Option<u64>)>;

fn layout_signature(l: &LayoutResult) -> EntitySignature {
    let mut sig: Vec<_> = l
        .entities
        .iter()
        .map(|e| {
            (
                e.name.clone(),
                e.x,
                e.y,
                e.direction as u8,
                e.recipe.clone(),
                e.carries.clone(),
                e.mirror,
                e.rate.map(f64::to_bits),
            )
        })
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

/// The hardcoded tier-ladder slice both diagnostics in this file run
/// over. Shared so the check-firing census and the RFC-070 selection
/// scoreboard describe THE SAME six solves — a scoreboard taken over a
/// different fixture set could not be read against the census rows.
/// Fields: item, rate, machine, external inputs.
const FIXTURES: &[(&str, f64, &str, &[&str])] = &[
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

#[test]
#[ignore = "G2 diagnostic census — run with --ignored --nocapture"]
fn check_firing_census() {
    let fixtures = FIXTURES;
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
    // search.rs), so a variant can come out identical to default — in the
    // label this file uses, TILES AND STAMPS identical (see
    // `EntitySignature`), which is stricter than validator-identical and
    // is not the same claim as "bit-identical" — for some or all fixtures
    // without that being visible anywhere in the table above.
    // Per-variant: how many non-default builds were COMPARABLE (their
    // fixture's own default also built), and how many of those were
    // structurally identical to it.
    //
    // #675 follow-up (round-7 minor, recorded on #686's closing comment):
    // the denominator used to count every successful non-default build,
    // including ones from fixtures whose default REFUSED. Those can never
    // be flagged identical — there is nothing to compare against, and the
    // code scores "unknown" as "not identical" — so they silently pushed
    // the printed ratio toward 0/N and invited a skimmer to read a real
    // difference where there was only a missing baseline. Non-comparable
    // builds are counted separately and printed as their own column.
    let mut comparable_builds: FxHashMap<&'static str, usize> = FxHashMap::default();
    let mut noncomparable_builds: FxHashMap<&'static str, usize> = FxHashMap::default();
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
                match default_sig.as_ref() {
                    Some(d) => {
                        *comparable_builds.entry(*vname).or_insert(0) += 1;
                        if *d == layout_signature(&l) {
                            *noop_count.entry(*vname).or_insert(0) += 1;
                        }
                    }
                    // This fixture's default refused (or the variant order
                    // ever changes so default isn't first): no baseline,
                    // so this build is not evidence either way and stays
                    // out of the ratio.
                    None => *noncomparable_builds.entry(*vname).or_insert(0) += 1,
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
        "\n=== per-variant no-op check (entities identical to that \
         fixture's own default; round 6, #686) ==="
    );
    println!(
        "  (denominator counts COMPARABLE builds only: ones that SUCCEEDED \
         and whose fixture's own default also built. \"+N with no default\" \
         counts successful builds whose fixture's default did NOT build — \
         they have no baseline, so they are neither identical nor \
         different. It is NOT a count of builds with a failing default in \
         general: a variant's OWN refusals never reach this tally at all, \
         they are in the `refusals:` summary at the top; #675 follow-up, \
         wording per #692 review round 3)"
    );
    println!(
        "  (\"identical\" = tiles AND stamps — name/x/y/direction/recipe/\
         carries/mirror/rate. `rate` is not validator-visible, so this is \
         a stricter test than \"validates the same\": a variant differing \
         only in its rate stamp counts as NOT identical)"
    );
    for &(vname, _, native_adjacent) in variants.iter().filter(|t| t.0 != "default") {
        let total = comparable_builds.get(vname).copied().unwrap_or(0);
        let noop = noop_count.get(vname).copied().unwrap_or(0);
        let uncomparable = noncomparable_builds.get(vname).copied().unwrap_or(0);
        let tag = if native_adjacent { "native-adjacent" } else { "user-elected" };
        println!(
            "  {vname:<12} {noop}/{total} comparable builds identical to default \
             (+{uncomparable} with no default to compare)  ({tag})"
        );
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
         nothing here — see the per-variant no-op check above, and note \
         that its \"identical\" is tiles-and-stamps, a STRICTER test than \
         validator-identical, so it can only under-report no-ops, never \
         over-report them. That rule \
         is enforced by the PAIRING, not by the no-op flag: a genuinely \
         identical variant validates identically, so its firings also \
         land on the same fixture's default side and cannot set either \
         flag. The flag is a label on the table, and the signature it \
         rests on now covers carries/mirror/rate so the label is worth \
         roughly what it claims."
    );
}

// ---------------------------------------------------------------------------
// RFC-070 Phase 0b (#689 W1b): the selection scoreboard
// ---------------------------------------------------------------------------

/// Render `select_best_decomposition`'s own scoreboard for each fixture:
/// which of the seven candidates were evaluated, what each of the THREE
/// verdict mechanisms said about them, who won, and which precedence
/// stage did the deciding.
///
/// Relationship to the census above: same fixtures, opposite direction.
/// The census approximates the candidate field from OUTSIDE, by
/// re-running the whole pipeline under option toggles (its stated v1
/// scope: "k1-shape-fix and size-split members only exist inside
/// `select_best_decomposition` and are not re-enacted here"). This one
/// reads the real internal loop, once, under DEFAULT options — the
/// production configuration, not a toggled one — and reports what it
/// actually did. It therefore closes the census's "which candidate" gap
/// while saying nothing about categories, and the census still closes
/// the "which categories fire" gap this says nothing about.
///
/// What it CANNOT show, by construction: the instrumentation records only
/// what the decision path already computed, never a fresh `validate()`
/// call. A candidate that no comparison mechanism needed carries no issue
/// counts at all, and those blanks print as `-`. A `-` is "nothing
/// computed this", NOT "zero" — reading it as zero is the `unwrap_or(0)`
/// mistake this instrument exists to avoid making at scale.
///
/// Run: cargo test --test check_firing_census -- --ignored --nocapture
#[test]
#[ignore = "RFC-070 Phase 0b diagnostic — run with --ignored --nocapture"]
fn selection_scoreboard_census() {
    use spaghettio_core::trace::{self, SelectionCandidateOutcome, SelectionStage, TraceEvent};

    fn outcome_name(o: SelectionCandidateOutcome) -> &'static str {
        // Exhaustive on purpose: the enum is closed, so a new outcome
        // fails to compile here rather than printing as something else.
        match o {
            SelectionCandidateOutcome::Produced => "produced",
            SelectionCandidateOutcome::Refused => "refused",
            SelectionCandidateOutcome::Panicked => "PANICKED",
            SelectionCandidateOutcome::NotRun => "not-run",
        }
    }

    fn stage_name(s: SelectionStage) -> &'static str {
        match s {
            SelectionStage::MergeTap => "merge-tap",
            SelectionStage::ScopedPairwise => "scoped-pairwise",
            SelectionStage::BestErrorFree => "best-error-free",
            SelectionStage::BestAccepted => "best-accepted",
            SelectionStage::FirstProduced => "first-produced",
        }
    }

    println!("\n=== RFC-070 selection scoreboard (default options, one build per fixture) ===");
    println!(
        "(err/selw/laww are counts a decision mechanism computed; `-` means \
         no mechanism computed one — a gap, not a zero. `from` names the \
         site that FIRST computed them, which is not necessarily the site \
         that decided: recording is first-write-wins and the value is the \
         same either way, so read the deciding stage below the table for \
         WHO decided. `reason` is produce()'s refusal text, or the \
         accepted=no tag for a candidate that built but failed the hard \
         gate.)"
    );

    let mut stage_tally: Vec<(String, String, String)> = Vec::new();
    for &(item, rate, machine, inputs) in FIXTURES {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let fixture_key = format!("{item}@{rate}:{machine}");
        // Every fixture contributes exactly one summary row, including the
        // ones that never reach a decision (#692 review, 2/3: the header
        // claimed "one row per fixture" while a no-winner or unsolvable
        // fixture silently contributed none — a summary that drops its
        // failures is the shape this repo keeps getting bitten by).
        let Ok(sr) = solver::solve(item, rate, &input_set, machine) else {
            println!("\n-- {fixture_key}\n   SKIP (no solve)");
            stage_tally.push((fixture_key, "—".to_string(), "SKIP (no solve)".to_string()));
            continue;
        };
        // Collect this fixture's whole event stream, then walk it. The
        // guard must outlive the drain.
        let guard = trace::start_trace();
        let built = layout::build_bus_layout(&sr, LayoutOptions::default());
        let events = trace::drain_events();
        drop(guard);

        println!("\n-- {fixture_key}");
        if let Err(e) = &built {
            println!("   build REFUSED: {e}");
        }

        // Pair candidates with their terminal event by flushing on
        // `SelectionDecided`. A candidate's `produce` can run its own
        // nested selection (cell composition builds per-cell layouts),
        // and each such block is contiguous and complete, so this
        // separates them instead of merging them into the outer one.
        //
        // Only the WINNER's nested blocks are visible: `run_candidate`
        // truncates every candidate's events out of the collector and
        // replays only the winner's, so a losing candidate's inner
        // selection is dropped before anything can read it. Absence of a
        // nested block is therefore not evidence that none ran.
        /// One selection's candidate rows plus its terminal verdict —
        /// `None` when the selection ended with every candidate failing,
        /// which emits rows but no `SelectionDecided`.
        type SelectionBlock<'a> = (Vec<&'a TraceEvent>, Option<(&'a str, SelectionStage)>);
        let mut blocks: Vec<SelectionBlock> = Vec::new();
        let mut pending: Vec<&TraceEvent> = Vec::new();
        for ev in &events {
            match ev {
                TraceEvent::SelectionCandidateEvaluated { name, .. } => {
                    // A block that ended WITHOUT a `SelectionDecided` (the
                    // all-candidates-failed path) would otherwise stay open
                    // and absorb the next block's rows — flushing only on
                    // the terminal event has no defense against that
                    // (#692 review, 3/3). `Scoreboard::emit` walks its rows
                    // in index order and index 0 is always `native`, so a
                    // `native` row arriving with rows already pending marks
                    // the start of a new block, terminated or not.
                    if name == "native" && !pending.is_empty() {
                        blocks.push((std::mem::take(&mut pending), None));
                    }
                    pending.push(ev);
                }
                TraceEvent::SelectionDecided { winner, stage } => {
                    blocks.push((std::mem::take(&mut pending), Some((winner.as_str(), *stage))));
                }
                _ => {}
            }
        }
        if !pending.is_empty() {
            // Candidates with no terminal event: the all-refused path.
            blocks.push((pending, None));
        }
        if blocks.is_empty() {
            println!("   (no selection events — build never reached the search)");
            stage_tally.push((
                fixture_key,
                "—".to_string(),
                "no selection events".to_string(),
            ));
            continue;
        }

        for (n, (rows, decided)) in blocks.iter().enumerate() {
            let is_outer = n == blocks.len() - 1;
            if blocks.len() > 1 && !is_outer {
                // An inner block belongs to a candidate whose `produce`
                // recursed. The OUTER block is the last one and must not
                // wear this banner — it is the row a reader is sent to
                // (#692 review round 2, 1/3: the banner previously
                // labelled "selection 2/2" as nested, which is exactly
                // the load-bearing one).
                println!(
                    "   [selection {}/{} — nested, from a candidate's own \
                     search; the outer selection is {}/{}]",
                    n + 1,
                    blocks.len(),
                    blocks.len(),
                    blocks.len()
                );
            }
            println!(
                "   {:<18} {:<9} {:>9} {:>4} {:>5} {:>5} {:>5}  {:<21} {:<9} reason",
                "candidate", "outcome", "score", "acc", "err", "selw", "laww", "from", "kinds"
            );
            for ev in rows {
                let TraceEvent::SelectionCandidateEvaluated {
                    name,
                    outcome,
                    reason,
                    score,
                    accepted,
                    accepted_reason,
                    errors,
                    selection_warnings,
                    layout_warnings,
                    counts_source,
                    contamination_errors,
                    starvation_errors,
                    structural_errors,
                    ..
                } = ev
                else {
                    continue;
                };
                let num = |v: &Option<usize>| v.map_or("-".to_string(), |n| n.to_string());
                let kinds = match (contamination_errors, starvation_errors, structural_errors) {
                    (Some(c), Some(s), Some(x)) => format!("c{c}/s{s}/x{x}"),
                    _ => "-".to_string(),
                };
                let won = decided.is_some_and(|(w, _)| w == name);
                println!(
                    "  {}{:<18} {:<9} {:>9} {:>4} {:>5} {:>5} {:>5}  {:<21} {:<9} {}",
                    if won { "*" } else { " " },
                    name,
                    outcome_name(*outcome),
                    score.map_or("-".to_string(), |s| format!("{s:.4}")),
                    accepted.map_or("-", |a| if a { "yes" } else { "no" }),
                    num(errors),
                    num(selection_warnings),
                    num(layout_warnings),
                    counts_source.as_deref().unwrap_or("-"),
                    kinds,
                    // `produce()`'s refusal text, or — for a candidate
                    // that produced but failed the hard gate — the
                    // `accepted=no` tag, which is the only place the
                    // missing-balancer-template count surfaces.
                    reason.as_deref().or(accepted_reason.as_deref()).unwrap_or("-"),
                );
            }
            match decided {
                Some((winner, stage)) => {
                    println!("   => winner: {winner}   deciding stage: {}", stage_name(*stage));
                    // The block-pairing invariant, checked rather than
                    // assumed (#692 review round 2, 3/3): a selection's
                    // winner must be one of ITS OWN rows. If grouping ever
                    // breaks — a reorder, or a nested selection emitted
                    // after the outer terminal — this fires instead of the
                    // table quietly attributing the wrong block's verdicts.
                    // A diagnostic prints its failures loudly rather than
                    // panicking; the CI-enforced version of this invariant
                    // is `selection_scoreboard_contract` below.
                    let winner_in_block = rows.iter().any(|ev| {
                        matches!(ev, TraceEvent::SelectionCandidateEvaluated { name, .. }
                                 if name == winner)
                    });
                    if !winner_in_block {
                        println!(
                            "   !! BLOCK PAIRING BROKEN: winner `{winner}` is not among \
                             this block's {} rows — the summary row below is not \
                             trustworthy",
                            rows.len()
                        );
                    }
                    if is_outer {
                        // The OUTER selection is the last block: nested
                        // ones close while the outer is still running.
                        stage_tally.push((
                            fixture_key.clone(),
                            (*winner).to_string(),
                            stage_name(*stage).to_string(),
                        ));
                    }
                }
                None => {
                    println!("   => NO WINNER (every candidate failed; see reasons above)");
                    if is_outer {
                        stage_tally.push((
                            fixture_key.clone(),
                            "—".to_string(),
                            "NO WINNER (all candidates failed)".to_string(),
                        ));
                    }
                }
            }
        }
    }

    println!(
        "\n=== outer-selection summary (one row per fixture, including the \
         ones that reached no decision) ==="
    );
    println!("{:<44} {:<18} deciding stage", "fixture", "winner");
    for (fixture, winner, stage) in &stage_tally {
        println!("{fixture:<44} {winner:<18} {stage}");
    }
    println!(
        "\nInterpretation: the three verdict mechanisms are not \
         commensurable and the columns must not be read as one ranking. \
         `score`/`acc` is the soft score (`score_layout`), whose `acc` \
         carries ONLY the missing-balancer-template hard gate and is not \
         a validation verdict. `err`/`selw`/`laww` are the component-wise \
         `IssueCounts` floor used by the DI and horizontal pairwise \
         comparisons — never lexicographic, so a better `selw` does NOT \
         buy a worse `laww`. `kinds` is the lexicographic `ErrorKinds` \
         key, computed only by the Pooled merge-tap decision. A blank \
         column is a candidate NO mechanism examined, and the blanks are \
         structural rather than incidental: a merge-tap decision \
         short-circuits the `clean_flags` tier entirely, so the only \
         counts such a fixture can show are ones a scoped pairwise \
         already computed — and where DI and horizontal both refused, \
         that is none, leaving the kinds key as the whole of what the \
         decision looked at and the whole of what this can report. That \
         is also why the deciding STAGE is the load-bearing column: it \
         says which question was actually asked, where the counts only \
         say what the answer was made of."
    );
}

// ---------------------------------------------------------------------------
// RFC-070 Phase 0b: the CI contract (NOT #[ignore]d)
// ---------------------------------------------------------------------------

/// What `assert_scoreboard_contract` hands back: the two facts each
/// caller pins for its own fixture, plus the per-slot outcomes so a
/// caller can also pin the candidate FIELD it decided among.
struct ScoreboardFacts {
    winner: String,
    stage: spaghettio_core::trace::SelectionStage,
    outcomes: Vec<(String, spaghettio_core::trace::SelectionCandidateOutcome)>,
}

impl ScoreboardFacts {
    fn outcome_of(&self, name: &str) -> spaghettio_core::trace::SelectionCandidateOutcome {
        self.outcomes
            .iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("no row for {name}"))
            .1
    }
}

/// Run one fixture under `LayoutOptions::default()` and assert the
/// scoreboard's STRUCTURAL contract, returning the winner and the stage
/// that decided — so each caller only has to state the fact that is
/// specific to its own fixture.
///
/// The three callers below cover three different deciding stages
/// (`best-error-free`, `merge-tap`, `scoped-pairwise`) because a contract
/// pinned on one stage cannot tell a broken stage TAG from a stage that
/// simply never fires: #692 landed with only the error-free tier
/// asserted, and the RFC-070 W1c corpus then measured four of the five
/// stages live. Mis-tagging `merge-tap` as `scoped-pairwise` would have
/// been invisible to a single-fixture pin.
///
/// The expected candidate order is written out longhand rather than
/// imported from `CANDIDATE_ORDER`: a test that reads the same constant
/// the code reads cannot detect a wrong reorder, because both move
/// together. This list is the independent second opinion.
///
/// **Zone cache**: this pins NO cache, while `parity_corpus.rs` insists
/// its baseline is cache-relative. The two postures are reconciled, not
/// in tension (#694 review round 2). CI pins the cache for these at the
/// JOB level — `ci.yml`'s `cargo nextest run -p spaghettio_core` sets
/// `SPAGHETTIO_ZONE_CACHE_PATH` to the committed
/// `crates/core/data/sat-zones-ci.bin` — so the CI path IS pinned and
/// reproducible. A local unpinned `cargo test` replays the developer's
/// own `~/.cache/spaghettio/sat-zones.bin` instead. Measured 2026-08-21:
/// all three pass under BOTH caches (pinned 1.10s, unpinned 2.66s), so
/// these three verdicts are stable across two different zone sets. That
/// is a datapoint, not a proof of cache-independence — **if one of these
/// three ever fails on a local unpinned run, re-run it with the pin
/// before believing it.** The corpus pins harder because it commits 160
/// rows as data; a test asserting three verdicts does not.
fn assert_scoreboard_contract(
    item: &str,
    rate: f64,
    machine: &str,
    inputs: &[&str],
) -> ScoreboardFacts {
    use spaghettio_core::trace::{self, SelectionCandidateOutcome, TraceEvent};

    /// Slot order asserted independently of the engine's own constant.
    const EXPECTED_ORDER: [&str; 7] = [
        "native",
        "k1-shape-fix",
        "size-split-2",
        "merge-tap",
        "cell-composed",
        "direct-insertion",
        "horizontal-stack",
    ];

    let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve(item, rate, &input_set, machine)
        .unwrap_or_else(|e| panic!("{item}@{rate} on {machine} must solve: {e}"));

    let guard = trace::start_trace();
    let built = layout::build_bus_layout(&sr, LayoutOptions::default());
    let events = trace::drain_events();
    drop(guard);
    built.unwrap_or_else(|e| panic!("{item}@{rate} on {machine} must build: {e}"));

    // --- every slot emits a row, in order, before the terminal event ---
    let rows: Vec<(&str, SelectionCandidateOutcome)> = events
        .iter()
        .filter_map(|e| match e {
            TraceEvent::SelectionCandidateEvaluated { name, outcome, .. } => {
                Some((name.as_str(), *outcome))
            }
            _ => None,
        })
        .collect();
    let names: Vec<&str> = rows.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names, EXPECTED_ORDER,
        "the scoreboard must emit one row per candidate SLOT, in canonical order, for \
         {item}@{rate} on {machine}. TWO readings, and they need different fixes: (1) the \
         ENGINE legitimately gained, lost or renamed a candidate — check `CANDIDATE_ORDER` \
         in decomposition_search.rs; if it changed, this list is merely stale, update it \
         and re-take the RFC-070 Phase-0 baseline, since the candidate field moved. (2) The \
         INSTRUMENTATION broke — `CANDIDATE_ORDER` is unchanged but `Scoreboard::emit` \
         stopped emitting a slot or the index alignment slipped, in which case every \
         recorded verdict is now attributed to the wrong candidate and the fix is here, not \
         in the baseline"
    );

    let decided: Vec<(&str, spaghettio_core::trace::SelectionStage)> = events
        .iter()
        .filter_map(|e| match e {
            TraceEvent::SelectionDecided { winner, stage } => Some((winner.as_str(), *stage)),
            _ => None,
        })
        .collect();
    assert_eq!(
        decided.len(),
        1,
        "exactly one selection must terminate for {item}@{rate} on {machine}; got \
         {decided:?}. TWO readings: (1) the ENGINE legitimately nested a selection — some \
         candidate's `produce` now runs its own search on this fixture, which is a correct \
         engine that this pin does not cover; confirm by checking whether the extra terminal \
         is preceded by its own full block of seven rows (the census's block walker renders \
         exactly that) and, if so, re-point this helper at the OUTER block instead of \
         assuming one. (2) The INSTRUMENTATION broke — terminals are duplicated or emitted \
         somewhere other than once per selection, which would regroup every census table"
    );

    let last_row = events
        .iter()
        .rposition(|e| matches!(e, TraceEvent::SelectionCandidateEvaluated { .. }))
        .expect("rows asserted present above");
    let terminal = events
        .iter()
        .position(|e| matches!(e, TraceEvent::SelectionDecided { .. }))
        .expect("terminal asserted present above");
    assert!(
        last_row < terminal,
        "all rows must precede their `SelectionDecided` — the census pairs a block \
         to its verdict by flushing on the terminal event, so a row after it \
         silently regroups the table. TWO readings: (1) the ENGINE nested a \
         selection whose block now lands after the outer terminal (the ordering \
         contract in `select_best_decomposition` puts the board immediately before \
         `SelectionDecided`, so this means that ordering changed and the walker \
         needs revisiting); (2) the INSTRUMENTATION broke — `board.emit()` moved \
         relative to the terminal event. `decided.len()` above discriminates: a \
         nested selection adds a terminal, a misplaced emit does not"
    );

    // --- the winner is one of this block's own rows, and it produced ---
    let (winner, stage) = decided[0];
    let winner_row = rows
        .iter()
        .find(|(n, _)| *n == winner)
        .unwrap_or_else(|| panic!("winner `{winner}` is not among the emitted rows {names:?}"));
    assert_eq!(
        winner_row.1,
        SelectionCandidateOutcome::Produced,
        "the winner's own row must say it produced a layout"
    );
    ScoreboardFacts {
        winner: winner.to_string(),
        stage,
        outcomes: rows.iter().map(|(n, o)| ((*n).to_string(), *o)).collect(),
    }
}

/// The non-ignored tests in this file are this one and its two siblings
/// below, and deliberately so.
///
/// Everything else here is a print-only diagnostic a human runs with
/// `--ignored`. That is fine for a table nobody's build depends on — but
/// the Phase-0b scoreboard is the oracle every later RFC-070 phase diffs
/// its shadow loop against, and an oracle whose only reader is a human
/// running a diagnostic by hand has no failure mode: a broken stage tag,
/// a row that stops being emitted, or an `from_run` outcome deduced
/// backwards would all ship green (#692 review round 2, 3/3). The repo's
/// own doctrine names this exactly — "a check going quiet is not evidence
/// the problem is fixed" (`docs/validator-reporting.md`).
///
/// So this pins the CONTRACT, not the layout: every candidate slot emits
/// a row, the rows arrive in the canonical order and before the terminal
/// event, the winner is one of that block's own rows, and the deciding
/// stage is the one this fixture actually reaches. Tile geometry is the
/// golden-hash tests' job and is deliberately not asserted here.
///
/// The structural half lives in `assert_scoreboard_contract`; this test
/// adds only the facts specific to the tier-1 fixture — which candidates
/// ran, who won, and which stage decided.
#[test]
#[ntest::timeout(120_000)]
fn selection_scoreboard_contract() {
    use spaghettio_core::trace::{SelectionCandidateOutcome, SelectionStage};

    let facts = assert_scoreboard_contract(
        "iron-gear-wheel",
        10.0,
        "assembling-machine-1",
        &["iron-plate"],
    );

    // --- `from_run`'s outcome deduction, across two live variants ---
    assert_eq!(
        facts.outcome_of("cell-composed"),
        SelectionCandidateOutcome::Produced,
        "cell-composition is a live candidate under `LayoutOptions::default()` \
         (`cell_composition: Candidate`) and produces on this chain-eligible fixture; \
         if this flips, the candidate FIELD changed, and the stage assertion below \
         will move with it"
    );
    assert_eq!(
        facts.outcome_of("k1-shape-fix"),
        SelectionCandidateOutcome::NotRun,
        "k1-shape-fix is gated on PartitionedDecomposed + an unaccepted native, \
         neither true here — `not-run` and `refused` are different facts and must \
         not collapse"
    );

    // --- the deciding stage ---
    assert_eq!(
        facts.winner, "native",
        "native must win this clean tier-1 fixture; got {}",
        facts.winner
    );
    assert_eq!(
        facts.stage,
        SelectionStage::BestErrorFree,
        "expected the error-free tier to decide: native and cell-composed both \
         produce here, so `clean_flags` runs (its gate is `n_layouts > 1`) and the \
         validation tier picks before `best-accepted` is reached. If this fails, \
         read it as one of TWO different things — the stage TAGGING broke (an \
         instrumentation bug, fix here), or the CANDIDATE SET for this fixture \
         changed so only one candidate now produces (an engine change, in which \
         case the expected stage is `best-accepted` and the RFC-070 Phase-0 \
         baseline needs re-taking). The `cell-composed` assertion above \
         discriminates between them."
    );
}

/// Third deciding stage: `scoped-pairwise`, the component-wise
/// `IssueCounts` floor. This is the production-default ec@30/am2 receipt
/// that was deliberately expected to flip when the belt jam was fixed
/// (#694 review round 3). The validated (3,2) restore makes the native
/// copper-cable feeders stampable; merge-tap no longer gates, and the
/// clean horizontal-stack candidate wins through scoped-pairwise. The
/// matching parity cells are re-blessed in `parity_corpus_baseline.json`.
#[test]
#[ntest::timeout(180_000)]
fn selection_scoreboard_contract_ec30_scoped_pairwise_stage() {
    use spaghettio_core::trace::SelectionStage;

    let facts = assert_scoreboard_contract(
        "electronic-circuit",
        30.0,
        "assembling-machine-2",
        &["iron-ore", "copper-ore"],
    );
    assert_eq!(
        facts.winner, "horizontal-stack",
        "the fixed ec@30/am2 fixture should select horizontal-stack through \
         scoped-pairwise; got {}",
        facts.winner
    );
    assert_eq!(
        facts.stage,
        SelectionStage::ScopedPairwise,
        "expected scoped-pairwise to decide the fixed ec@30/am2 fixture; got {:?}. \
         If this moves again, compare it with the pinned tier2_ec_am2_30_ore \
         parity row before changing the contract",
        facts.stage
    );
}

/// Third deciding stage: `scoped-pairwise`, the component-wise
/// `IssueCounts` floor — the mechanism that is deliberately NOT
/// lexicographic. Here horizontal-stack displaces native, so this also
/// pins that a scoped pairwise CAN name a non-native winner, which the
/// merge-tap fixture above cannot show.
#[test]
#[ntest::timeout(180_000)]
fn selection_scoreboard_contract_scoped_pairwise_stage() {
    use spaghettio_core::trace::SelectionStage;

    let facts = assert_scoreboard_contract(
        "advanced-circuit",
        5.0,
        "assembling-machine-2",
        &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
    );
    assert_eq!(
        facts.winner, "horizontal-stack",
        "horizontal-stack strictly improves native's issue channels on ac@5/am2 and \
         displaces it; got {}. This is a knife-edge by construction — the pairwise floor \
         is component-wise, so one channel moving either way flips it, and the corpus \
         records `hs-off` on this same fixture landing on native/best-error-free. A red \
         here is therefore as likely to be a legitimate engine shift as a bug: re-take the \
         parity baseline (SPAGHETTIO_PARITY_CORPUS=bless), check whether the fixture \
         SIM-anchors better or worse than the +0.6%-of-plan recorded in RFC-070's decision \
         log, and update this expectation rather than reverting to reach it",
        facts.winner
    );
    assert_eq!(
        facts.stage,
        SelectionStage::ScopedPairwise,
        "expected `horizontal_choice`'s pairwise comparison to decide ac@5/am2. TWO \
         readings: (1) the stage TAGGING broke — fix here; (2) the ENGINE changed so \
         horizontal no longer strictly improves, in which case the winner assertion \
         above fires first and the RFC-070 parity corpus needs re-taking (this fixture \
         is `tier4_ac_am2_5_unconstrained` there; the baseline records \
         horizontal-stack/scoped-pairwise under four option sets and \
         native/best-error-free under `hs-off`)"
    );
}
