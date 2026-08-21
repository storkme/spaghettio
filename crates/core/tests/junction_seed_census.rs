//! G1 (offpath campaign, #689 W1d) — the production census of junction
//! seeds that `docs/offpath-code-followups.md`'s G1 entry named as the
//! one remaining follow-up. #687 established the ~795-line perpendicular-
//! template rung (`crates/core/src/bus/ghost_router.rs`,
//! `solve_perpendicular_template` area) appears production-unreachable on
//! BOTH its shapes: belt×belt two-item crossings die on
//! `junction_solver`'s item-conflict gate before the rung's single-tile
//! window, and pipe×belt crossings never seed junctions because
//! `keys_at_tile` filters `SpecKind::Pipe` out of seeding entirely. The
//! ONE remaining reachability hypothesis: same-item belt crossings — do
//! they ever seed junctions? This census answers exactly that question,
//! nothing else — CENSUS-ONLY, no gate/behavior change regardless of the
//! result (see the G1 entry for what a zero count would and would not
//! authorize).
//!
//! Instrumentation: `TraceEvent::JunctionSeedCensus`
//! (`crates/core/src/trace.rs`), emitted once per junction seed in
//! `ghost_router`'s cluster loop — every `keys_at_tile` computation that
//! survives the `corridor_handled`/`any_undecidable` skips and reaches
//! `junction_solver::solve_crossing` (`crates/core/src/bus/ghost_router.rs`,
//! right after `keys_at_tile` is built). Purely observational — the emit
//! site is gated behind `SPAGHETTIO_JUNCTION_SEED_CENSUS` (checked once
//! per `route_bus_ghost` call, off by default), so this test must set
//! that env var, and production pays nothing without it (round 1 review
//! finding: the shipped web path always has a trace collector *and* sink
//! active, so gating on "is tracing active" alone would not have made
//! this zero-cost there). The gate is on the ENV VAR only, so a caller
//! that sets it without attaching a collector still pays the full
//! per-cluster payload cost — the `resolve_item` fold and the
//! `routed_paths` scan both run before `trace::emit` discovers there is
//! nothing to emit into (round 6 review, #691).
//!
//! **A NECESSARY (not exact) predicate for the rung's firing** (round 4
//! review finding: the previous wording overclaimed precision).
//! `PerpendicularTemplateStrategy::try_solve` (ghost_router.rs
//! 6177/6183/6186) actually gates on THREE things: `region.tile_count()
//! == 1`, `junction.specs.len() == 2`, AND `is_perpendicular(da, db)` on
//! the two specs' directions — this census records neither direction,
//! so it cannot evaluate the third condition, and `specs.len()` at the
//! moment `try_solve` runs may include specs the growth loop
//! *encountered* after this seed's `keys_at_tile` was captured, not only
//! the original seed set. So `cluster_tile_count == 1 && n_specs == 2 &&
//! n_distinct_items == 1` is a bucket the rung's firing conditions are
//! a SUBSET of — not the predicate itself. That's still enough to
//! support the zero conclusion: any actual firing must land in this
//! bucket, so an empty bucket means zero firings, full stop; it just
//! isn't the tightest possible bucket. (Round 2 review separately fixed
//! a real superset bug in an earlier pass — flagging ANY single-tile
//! seed with `n_specs > n_distinct_items`, which would have counted a
//! single-tile 3+-spec cluster as a "hit" even though the rung hard-
//! refuses whenever `specs.len() != 2`; this corpus has zero such
//! clusters, so the number didn't change, but the metric needed
//! tightening regardless.)
//!
//! **Methodology note (round 1 review finding):** `n_specs`/
//! `n_distinct_items` are CLUSTER-WIDE — the union of every spec whose
//! path touches ANY tile in the seed's cluster, which can already span
//! multiple tiles before `solve_crossing`'s own growth. `cluster_tile_count`
//! disambiguates: when it's `1` the union IS that one tile's spec set;
//! when it's `>1`, "some tile in this cluster" is not "this tile" — the
//! rung structurally cannot ever run on a `tile_count > 1` region (see
//! above), so a per-tile breakdown of multi-tile clusters is not needed
//! to answer the reachability question — those clusters are provably
//! outside the rung's domain regardless of what any one tile inside them
//! looks like. This census still reports the multi-tile same-item count
//! as a separate, explicitly weaker signal (round 2 review: likely
//! dominated by the mundane case of a trunk column and its own non-last
//! tap-off sharing an item, per `docs/rfc-unified-belt-specs.md` Phase 2
//! — that's a parent/child relationship within ONE logical flow, not two
//! independent specs crossing, so treat the multi-tile count as an upper
//! bound on anything resembling a same-item crossing, not evidence of
//! one).
//!
//! **Scope note (round 2 review, CORRECTED round 5):** `build_bus_layout`
//! runs the full candidate search + junction-cap retry machinery
//! (`decomposition_search::select_best_decomposition` →
//! `run_layout_with_retry_inner`), each of which can invoke
//! `route_bus_ghost` more than once per fixture (once per evaluated
//! candidate variant, plus a pass-2 retry when pass 1 caps). An earlier
//! version of this note claimed the totals count "every ... invocation
//! ... not deduplicated" — that overstates it: when a candidate's pass 1
//! caps, `run_layout_with_retry_inner` calls
//! `trace::truncate_events(trace_start)` to discard that candidate's
//! pass-1 events (including any `JunctionSeedCensus` it emitted) before
//! pass 2 runs, specifically so a streaming consumer never sees an
//! abandoned pass. So this census counts the SHIPPED (surviving) pass
//! per candidate evaluation — a retried candidate's pass-1 seeds are
//! silently dropped from what this test observes, not double-counted.
//!
//! **Corrected round 6 (#691):** this census does NOT observe every
//! candidate. `decomposition_search::run_candidate` captures each
//! candidate's events and `truncate_events` them out of the collector,
//! replaying only the WINNER's at the end — so a LOSING candidate's
//! junction seeds never reach this test at all. The totals are therefore
//! the winner's surviving pass per selection call (plus any nested
//! selection inside the winner), not one entry per evaluated candidate
//! variant. The earlier wording ("each candidate's own final pass counts
//! separately") described a stream the production loop deliberately
//! edits. This does not move the zero conclusion — losing candidates'
//! seeds being invisible can only REMOVE observations, never fabricate a
//! same-item hit — but it does mean the corpus is narrower than the
//! candidate count suggests, and it widens the same lower-bound caveat
//! the truncation note below already carries.
//!
//! A seed whose geometry is untouched across passes within one candidate
//! can still appear at most once per candidate (pass 1 is gone if pass 2
//! ran). This test also tallies `TraceEvent::LayoutRetried` occurrences
//! (emitted right after the truncate, so it survives into what this test
//! observes) as a directly-measured count of how many truncation
//! episodes happened — printed next to the summary line. **This cannot
//! flip the zero conclusion to a false positive**: truncation only
//! REMOVES seed observations this census would otherwise have counted;
//! it can never fabricate a same-item hit that didn't occur. It CAN hide
//! a true positive that occurred only in a truncated pass-1 — the zero
//! reported below is therefore a lower bound on this corpus's true
//! same-item-crossing count, not a certainty that no such pass-1 ever
//! existed. `SPAGHETTIO_JUNCTION_SEED_CENSUS`, once wired to fire from
//! inside `layout_pass` before the truncate point (out of scope for this
//! CENSUS-ONLY PR), would close this gap.
//!
//! Corpus (stated exactly, per the W1d brief): the same six tier-ladder
//! fixtures `check_firing_census.rs` uses (G2's hardcoded slice — kept
//! identical so the two censuses describe the same baseline corpus), plus
//! the six explicit "from-ore" fixtures defined in `tests/e2e.rs`
//! (`tier1_iron_gear_wheel_from_ore`, `tier2_electronic_circuit_from_ore`,
//! `tier2_electronic_circuit_20s_from_ore`, `tier3_plastic_bar_from_crude`,
//! `tier4_advanced_circuit_from_ore_am2`, `tier5_processing_unit_from_ore_am3`)
//! that differ from the tier-ladder slice in belt-tier constraint and/or
//! rate/machine (both listed so a reader can see what widened the corpus
//! over G2's). Each fixture builds ONCE under `LayoutOptions::default()`
//! (plus the fixture's own belt-tier constraint) — this census is about
//! whether the rung is reachable from the shape the engine actually
//! ships, not about candidate-search variants (G2's concern).
//!
//! Determinism: like every SAT-backed layout run, seed shapes depend on
//! which zone solutions the cache replays. Run pinned, per
//! `docs/offpath-code-followups.md`'s method section and CLAUDE.md's
//! verification protocol:
//!
//! ```text
//! SPAGHETTIO_ZONE_CACHE_PATH=$(pwd)/crates/core/data/sat-zones-ci.bin \
//!   cargo test --manifest-path crates/core/Cargo.toml \
//!   --test junction_seed_census -- --ignored --nocapture
//! git checkout -- crates/core/data/sat-zones-ci.bin   # ALWAYS, every run —
//! # the cache file is written to by the run above; it must never appear
//! # in a commit.
//! ```

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::solver;
use spaghettio_core::trace::{self, TraceEvent};

/// RAII guard: sets an env var for the guard's lifetime and restores
/// whatever value (or absence) it had before, on drop — including on
/// panic unwind, so an assertion failure mid-test doesn't leak the
/// mutation into any test that runs afterward in this process (round 3
/// review finding: this test's own env mutation was never restored).
struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prev = std::env::var(key).ok();
        // No `unsafe` here: `crates/core/Cargo.toml` pins `edition =
        // "2021"` (verified directly, not guessed — round 4 review),
        // and `std::env::set_var`/`remove_var` are plain safe functions
        // at edition 2021 on this toolchain (rustc 1.95.0) — confirmed
        // by compiling a bare, unwrapped call with `--edition 2021`:
        // zero errors, zero warnings. (Round 3's claim that the wrapper
        // WAS required was based on a flawed test: wrapping the call in
        // `unsafe {}` also produces no `unused_unsafe` warning at this
        // edition, which looks like "required" but isn't — these two
        // functions carry a `#[rustc_deprecated_safe_2024]`-style
        // edition-conditional attribute that specifically suppresses
        // `unused_unsafe` for them, precisely so pre-emptive wrapping
        // doesn't warn during the safe→unsafe migration window. The
        // decisive test is whether the BARE call compiles, not whether
        // the wrapped call warns: at edition 2024 the bare call fails
        // with E0133 "call to unsafe function"; at edition 2021 it
        // compiles clean. This crate is 2021, so the wrapper is
        // genuinely unneeded today — re-add it if this crate ever bumps
        // to edition 2024.) This test binary is single-threaded for env
        // mutations — no other thread reads/writes process env
        // concurrently here.
        std::env::set_var(key, value);
        EnvVarGuard { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // See `set` above — no `unsafe` needed at this crate's edition.
        match &self.prev {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
#[ignore = "G1 diagnostic census — run with --ignored --nocapture; see module doc for the zone-cache pin"]
fn junction_seed_census() {
    // The emit site is gated behind this env var (off by default in
    // production — round 1 review finding). Set it for the duration of
    // this test only; restored on drop regardless of how the test exits.
    let _census_env_guard = EnvVarGuard::set("SPAGHETTIO_JUNCTION_SEED_CENSUS", "1");

    // (label, item, rate, machine, belt_tier, inputs)
    #[allow(clippy::type_complexity)]
    let fixtures: &[(&str, &str, f64, &str, Option<&str>, &[&str])] = &[
        // --- tier-ladder slice (identical to check_firing_census.rs's six,
        // so both censuses describe the same baseline corpus) ---
        ("tier1_gear_am1", "iron-gear-wheel", 10.0, "assembling-machine-1", None, &["iron-plate"]),
        ("tier2_ec_am1_10_ore", "electronic-circuit", 10.0, "assembling-machine-1", None, &["iron-ore", "copper-ore"]),
        ("tier2_ec_am2_30_ore", "electronic-circuit", 30.0, "assembling-machine-2", None, &["iron-ore", "copper-ore"]),
        ("tier3_plastic_cp_5", "plastic-bar", 5.0, "chemical-plant", None, &["coal", "water", "crude-oil"]),
        (
            "tier4_ac_am2_5_unconstrained",
            "advanced-circuit",
            5.0,
            "assembling-machine-2",
            None,
            &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        ),
        (
            "tier5_pu_am3_2_unconstrained",
            "processing-unit",
            2.0,
            "assembling-machine-3",
            None,
            &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        ),
        // --- e2e "from-ore" fixtures, each distinct from the slice above in
        // belt-tier constraint and/or rate/machine (widening the corpus) ---
        (
            "e2e_tier1_iron_gear_wheel_from_ore",
            "iron-gear-wheel",
            10.0,
            "assembling-machine-2",
            None,
            &["iron-ore"],
        ),
        (
            "e2e_tier2_electronic_circuit_from_ore",
            "electronic-circuit",
            10.0,
            "assembling-machine-1",
            Some("transport-belt"),
            &["iron-ore", "copper-ore"],
        ),
        (
            "e2e_tier2_electronic_circuit_20s_from_ore",
            "electronic-circuit",
            20.0,
            "assembling-machine-2",
            None,
            &["iron-ore", "copper-ore"],
        ),
        (
            "e2e_tier3_plastic_bar_from_crude",
            "plastic-bar",
            10.0,
            "chemical-plant",
            None,
            &["crude-oil", "coal"],
        ),
        (
            "e2e_tier4_advanced_circuit_from_ore_am2",
            "advanced-circuit",
            5.0,
            "assembling-machine-2",
            Some("transport-belt"),
            &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        ),
        (
            "e2e_tier5_processing_unit_from_ore_am3",
            "processing-unit",
            2.0,
            "assembling-machine-3",
            Some("fast-transport-belt"),
            &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        ),
    ];

    // (cluster_tile_count, n_specs, n_distinct_items) -> occurrence count.
    let mut table: std::collections::BTreeMap<(usize, usize, usize), usize> = Default::default();
    let mut total_seeds = 0usize;
    let mut pipe_tagged_seeds = 0usize;
    // Cross-check (round 2 review): does "pipe-tagged" actually coincide
    // with "single-spec seed", as the prose has claimed? Measure it
    // instead of asserting it.
    let mut pipe_tagged_and_single_spec = 0usize;
    let mut pipe_tagged_but_not_single_spec = 0usize;
    let mut single_spec_but_not_pipe_tagged = 0usize;

    // Every seed falls into exactly one of these six buckets — printed
    // total must equal `total_seeds` (checked below via assert_eq!, not
    // just asserted in prose).
    let mut zero_specs_after_filter = 0usize; // n_specs=0 (round 3 review finding: reachable — e.g. a pipe-only seed where keys_at_tile filters out the sole participant — and precisely the kind of case this census exists to surface, never to crash on)
    let mut single_tile_bypass = 0usize; // tiles=1, n_specs=1 (the belt-over-forbidden-tile bypass)
    let mut single_tile_diff_item = 0usize; // tiles=1, n_specs=2, distinct=2
    let mut single_tile_same_item = 0usize; // tiles=1, n_specs=2, distinct=1 — the rung's EXACT predicate shape
    let mut single_tile_gt2_specs = 0usize; // tiles=1, n_specs>2 (outside the rung's specs.len()==2 gate regardless of item-sharing)
    let mut multi_tile_all_distinct = 0usize; // tiles>1, n_specs==n_distinct_items, n_specs>0
    let mut multi_tile_same_item = 0usize; // tiles>1, n_specs>n_distinct_items — weak signal, see module doc

    let mut same_item_single_tile: Vec<(String, i32, i32)> = Vec::new();
    let mut same_item_multi_tile_union: Vec<(String, i32, i32, usize, usize, usize)> = Vec::new();
    let mut builds_ok = 0usize;
    let mut builds_refused = 0usize;
    let mut solver_skipped = 0usize;
    // Round 5 review: directly measure truncation episodes rather than
    // just caveating them in prose. `LayoutRetried` is emitted AFTER
    // `run_layout_with_retry_inner`'s `trace::truncate_events` call (see
    // the module doc's scope note), so it survives into what this test
    // observes and counts exactly how many times a candidate's pass-1
    // census events were discarded before this test ever saw them.
    let mut retried_episodes = 0usize;

    for &(label, item, rate, machine, belt_tier, inputs) in fixtures {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = match solver::solve(item, rate, &input_set, machine) {
            Ok(sr) => sr,
            Err(e) => {
                eprintln!("SKIP (no solve): {label}: {e}");
                solver_skipped += 1;
                continue;
            }
        };
        let opts = LayoutOptions {
            max_belt_tier: belt_tier.map(|s| s.to_string()),
            ..LayoutOptions::default()
        };
        let _guard = trace::start_trace();
        let build_result = layout::build_bus_layout(&sr, opts);
        let events = trace::drain_events();
        match build_result {
            Ok(_) => builds_ok += 1,
            Err(e) => {
                // Still tabulate whatever seeds fired before the refusal —
                // route_bus_ghost's cluster loop emits the census event
                // per seed as it goes, independent of whether the overall
                // layout pass later refuses for an unrelated reason.
                eprintln!("REFUSED: {label}: {e}");
                builds_refused += 1;
            }
        }
        for ev in &events {
            if matches!(ev, TraceEvent::LayoutRetried { .. }) {
                retried_episodes += 1;
            }
            if let TraceEvent::JunctionSeedCensus {
                seed_x,
                seed_y,
                cluster_tile_count,
                n_specs,
                n_distinct_items,
                has_pipe,
            } = ev
            {
                total_seeds += 1;
                *table.entry((*cluster_tile_count, *n_specs, *n_distinct_items)).or_insert(0) += 1;

                let is_single_spec = *n_specs == 1;
                if *has_pipe {
                    pipe_tagged_seeds += 1;
                    if is_single_spec {
                        pipe_tagged_and_single_spec += 1;
                    } else {
                        pipe_tagged_but_not_single_spec += 1;
                    }
                } else if is_single_spec {
                    single_spec_but_not_pipe_tagged += 1;
                }

                // Round 3 review finding: n_specs == 0 IS reachable (a
                // single-tile — or, in principle, multi-tile — cluster
                // whose only participant(s) were pipes, all filtered out
                // of keys_at_tile) and is exactly the kind of edge case
                // this census exists to surface, so it gets its own
                // bucket ahead of everything else — never a crash. Every
                // other arm below is now genuinely exhaustive without a
                // data-dependent catch-all: once n_specs == 0 is handled
                // first, n_distinct_items is provably in 1..=n_specs for
                // every remaining case (a non-empty HashSet's size can't
                // exceed its input count or be zero), so the trailing
                // `_ => unreachable!()` only catches a violated counting
                // invariant, not a real corpus shape.
                match (*cluster_tile_count == 1, *n_specs, *n_distinct_items) {
                    (_, 0, _) => zero_specs_after_filter += 1,
                    (true, 1, _) => single_tile_bypass += 1,
                    (true, 2, 2) => single_tile_diff_item += 1,
                    (true, 2, 1) => {
                        single_tile_same_item += 1;
                        same_item_single_tile.push((label.to_string(), *seed_x, *seed_y));
                    }
                    (true, n, _) if n > 2 => single_tile_gt2_specs += 1,
                    (false, n, d) if n == d => multi_tile_all_distinct += 1,
                    (false, n, d) if n > d => {
                        multi_tile_same_item += 1;
                        same_item_multi_tile_union.push((
                            label.to_string(),
                            *seed_x,
                            *seed_y,
                            *cluster_tile_count,
                            *n_specs,
                            *n_distinct_items,
                        ));
                    }
                    _ => unreachable!(
                        "counting invariant violated: n_distinct_items ({n_distinct_items}) \
                         must be in 1..=n_specs ({n_specs}) once n_specs > 0"
                    ),
                }
            }
        }
    }

    println!(
        "\n=== junction seed census: {total_seeds} seeds across {} fixtures ({builds_ok} built, {builds_refused} refused, {solver_skipped} solver-skipped) ===",
        fixtures.len()
    );
    // Round 5 review: the seed counts above are partial in two distinct
    // ways, both DECREASING what this census can observe (never
    // inflating it — see the module doc's scope note for why that means
    // the zero conclusion below is a lower bound, not a certainty):
    // (1) a REFUSED build's counted seeds are only whatever fired before
    // the failure — the layout never finished, so there may have been
    // more; (2) a build that internally retried had its pass-1 seeds
    // discarded by the engine itself (`trace::truncate_events`) before
    // this test could ever see them — `retried_episodes` below is a
    // direct, measured count of how many such discards happened, not a
    // guess.
    println!(
        "(seed counts are PARTIAL, never inflated: {builds_refused} build(s) refused \
         partway through, and {retried_episodes} candidate-evaluation(s) internally \
         retried — each retry's pass-1 census events were discarded by \
         `trace::truncate_events` before this test observed them; see the \
         module doc's scope note)"
    );
    println!("{:>7} {:>8} {:>9} {:>7}", "tiles", "n_specs", "distinct", "count");
    for ((tiles, n_specs, n_distinct), count) in &table {
        println!("{tiles:>7} {n_specs:>8} {n_distinct:>9} {count:>7}");
    }

    // Reconciliation: every seed must land in exactly one bucket. Assert
    // it rather than trust hand-transcribed prose (round 2 review) — this
    // is an arithmetic invariant true by construction (the match above is
    // exhaustive over the same fields the table sums), not a data-
    // dependent claim, so it can't spuriously fail from corpus drift.
    let bucket_sum = zero_specs_after_filter
        + single_tile_bypass
        + single_tile_diff_item
        + single_tile_same_item
        + single_tile_gt2_specs
        + multi_tile_all_distinct
        + multi_tile_same_item;
    assert_eq!(bucket_sum, total_seeds, "seed buckets must partition total_seeds exactly");

    println!(
        "\n--- bucket breakdown (sums to {total_seeds}) ---\n\
         n_specs == 0 after the pipe filter (never a crash — see module doc): {zero_specs_after_filter}\n\
         single-tile, 1 spec (belt-over-forbidden-tile bypass): {single_tile_bypass}\n\
         single-tile, 2 specs, different items:                 {single_tile_diff_item}\n\
         single-tile, 2 specs, SAME item (necessary-superset):  {single_tile_same_item}\n\
         single-tile, >2 specs (outside rung's specs.len()==2): {single_tile_gt2_specs}\n\
         multi-tile,  all participants distinct items:          {multi_tile_all_distinct}\n\
         multi-tile,  item-sharing pair somewhere in union:     {multi_tile_same_item}"
    );

    // The DISCOVERY DUMPS run BEFORE the assertion below (round 6
    // review, #691): the assert is what fires when the conclusion
    // changes, and the seeds printed here are the only record of WHICH
    // seeds changed it. Printing them afterwards meant a failing run
    // reported the count and swallowed the addresses — the reader would
    // have had to comment out the assert to see the finding it was
    // announcing.
    println!(
        "\nsame-item seeds matching the census's necessary-superset bucket \
         for the rung's predicate (cluster_tile_count == 1 AND n_specs == 2 \
         AND n_distinct_items == 1 — see the module doc's precision note; \
         zero here proves zero actual firings, but the bucket may be \
         broader than the rung's true firing set): {}",
        same_item_single_tile.len()
    );
    for (label, x, y) in &same_item_single_tile {
        println!("  {label} @ ({x},{y})");
    }
    println!(
        "\nsame-item MULTI-TILE cluster-wide seeds (cluster_tile_count > 1 \
         AND the cluster's participant UNION has an item-sharing pair \
         somewhere in it — a weak, likely trunk/tap-contaminated signal; \
         see the module doc's methodology note. NOT evidence of a same-item \
         crossing, and not further examined per-tile because the rung \
         structurally cannot act on tile_count > 1 regardless): {}",
        same_item_multi_tile_union.len()
    );
    for (label, x, y, tiles, n_specs, n_distinct) in &same_item_multi_tile_union {
        println!("  {label} @ ({x},{y}): {tiles} tiles, {n_specs} specs, {n_distinct} distinct items");
    }

    // Round 5 review: this is THE conclusion the census exists to check
    // — not an exploratory tally where an unexpected value is itself
    // informative (that's what the demoted checks below are for; round
    // 4's "never abort on discovery" call was about those, not this).
    // Deliberately a hard `assert_eq!`: this diagnostic is `#[ignore]`d,
    // so it only runs when someone deliberately re-executes the
    // evidence behind a deletion or reachability decision — exactly the
    // moment a changed conclusion should be loud, not a silently-updated
    // number in a printed line nobody re-reads. If this ever fires, DO
    // NOT proceed with any deletion of the perpendicular-template rung
    // (ghost_router.rs, `solve_perpendicular_template`/`try_bridge`/
    // `bridge_belt_over_pipe`) on the strength of this census — the
    // reachability conclusion has changed, and `docs/offpath-code-
    // followups.md`'s G1 entry must be updated with the new finding
    // before anyone trusts a deletion call built on the old "zero"
    // result.
    //
    // The message says "re-examine", not "the rung is reachable" (round
    // 6 review, #691). The bucket is a NECESSARY superset, not the
    // rung's predicate — it records neither spec direction, so it cannot
    // evaluate `is_perpendicular` — and a non-zero bucket therefore
    // establishes only that the cheap proof of unreachability is gone,
    // never that the rung fires. Wording it as a falsified conclusion
    // would have handed the next reader a stronger finding than the
    // instrument can support.
    assert_eq!(
        single_tile_same_item, 0,
        "necessary-superset bucket non-zero — re-examine perpendicularity \
         before trusting the conclusion. Same-item seed(s) landed in the \
         rung's necessary-superset bucket (cluster_tile_count == 1, \
         n_specs == 2, n_distinct_items == 1), listed above. That bucket \
         does NOT check `is_perpendicular`, so this is not proof the rung \
         fires — it is proof the zero-reachability argument no longer \
         holds cheaply on this corpus. Check the printed seeds' spec \
         directions, then update docs/offpath-code-followups.md's G1 \
         entry BEFORE trusting any deletion decision that cited the old \
         zero."
    );

    println!(
        "\npipe-tagged seeds (measured on the RAW spec set at each cluster's \
         tiles, BEFORE keys_at_tile's SpecKind::Pipe filter runs): \
         {pipe_tagged_seeds} — expected to equal the single-spec bypass \
         count ({single_tile_bypass} + {zero_specs_after_filter} zero-spec, \
         since a zero-spec seed's sole participant was necessarily a \
         filtered-out pipe); any EXCESS beyond that is the signal — a \
         multi-spec seed also touched by a pipe, which #687's pipe×belt \
         finding does not rule out and this census has not previously \
         checked for."
    );
    // Round 5 review: this corroboration, and the same_item_single_tile
    // zero conclusion above, both inherit an acknowledged residual risk
    // from n_distinct_items's item-resolution path in ghost_router.rs —
    // TWO independent biases, and both bias toward HIDING a same-item
    // pair rather than fabricating one: (1) the fluid catch-up sweep
    // only tags a synth key as Pipe when its path sits ENTIRELY on pipe
    // tiles, so a fluid synth key touching any non-pipe tile falls
    // through to `resolve_item`'s prefix-recovery path instead of being
    // excluded as a pipe outright; (2) `resolve_item`'s absolute-last-
    // resort (no `spec_items` entry, no recognized key prefix) treats
    // the raw key as a unique pseudo-item, which can make two specs that
    // truly share an item read as "distinct" if their key format is one
    // this census doesn't yet recognize. Neither has been observed
    // firing in any run to date (this census would need dedicated
    // instrumentation on `resolve_item` itself to prove that beyond
    // "not observed so far"), so the zero conclusion above is honest but
    // not airtight: a same-item pair COULD be hiding behind either bias.
    // Any deletion follow-up that cites this census's zero should say so.
    println!(
        "(both the corroboration above and the zero same-item conclusion \
         printed earlier share an acknowledged residual risk: ghost_router.rs's \
         item-resolution fallback path can HIDE a true same-item pair — \
         never fabricate one — under an unrecognized key format; not \
         observed in any run to date, but not proven absent either. Cite \
         this caveat alongside the zero in any deletion follow-up.)"
    );
    // Round 4 review finding: these two correlation checks were hard
    // `assert_eq!`s in an earlier pass, but a legitimate new seed shape
    // (round 3's zero_specs_after_filter bucket: has_pipe=true,
    // n_specs=0, so NOT single-spec) trips `pipe_tagged_but_not_single_
    // spec` by construction — the census would abort exactly when it
    // finds something worth reporting. A census must never crash on
    // discovery; demoted to printed diagnostics with a loud marker.
    //
    // TWO hard assertions survive, both ABOVE this point (round 6
    // review, #691 — an earlier version of this comment said "one",
    // "below", and named only the first): `bucket_sum == total_seeds`,
    // a true tautology the exhaustive match cannot fail on real data,
    // and `single_tile_same_item == 0`, which is data-dependent and
    // deliberately so — it is the conclusion the census exists to
    // defend, not a correlation.
    if pipe_tagged_but_not_single_spec > 0 {
        println!(
            "  ⚠ UNEXPECTED: {pipe_tagged_but_not_single_spec} pipe-tagged \
             seed(s) are NOT single-spec (n_specs > 1 with a pipe also \
             touching the cluster) — investigate before trusting the \
             pipe-bypass narrative for these"
        );
    }
    if single_spec_but_not_pipe_tagged > 0 {
        println!(
            "  ⚠ UNEXPECTED: {single_spec_but_not_pipe_tagged} single-spec \
             seed(s) are NOT pipe-tagged — a bypass reason other than the \
             belt-crosses-a-placed-pipe case may exist"
        );
    }
    println!(
        "cross-check (printed, not asserted — see round 4 review note \
         above): pipe-tagged & single-spec = {pipe_tagged_and_single_spec}, \
         pipe-tagged but NOT single-spec = {pipe_tagged_but_not_single_spec}, \
         single-spec but NOT pipe-tagged = {single_spec_but_not_pipe_tagged}"
    );
}
