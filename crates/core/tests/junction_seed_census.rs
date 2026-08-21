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
//! this zero-cost there).
//!
//! **The rung's EXACT firing predicate** (`PerpendicularTemplateStrategy::
//! try_solve`, ghost_router.rs 6177/6183/6186): `region.tile_count() == 1`
//! AND `junction.specs.len() == 2`. This census's "true rung shape"
//! bucket below requires both — `cluster_tile_count == 1`,
//! `n_specs == 2`, `n_distinct_items == 1` (round 2 review finding: an
//! earlier version of this test flagged ANY single-tile seed with
//! `n_specs > n_distinct_items`, which is a superset of the rung's
//! predicate — a single-tile 3+-spec cluster with an item-sharing pair
//! would have counted as a "hit" even though the rung can never fire on
//! it, since it hard-refuses whenever `specs.len() != 2`). This corpus
//! happens to have zero single-tile clusters with more than 2 specs, so
//! the superset bug didn't change the reported number, but the metric
//! itself needed tightening.
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
//! **Scope note (round 2 review):** `build_bus_layout` runs the full
//! candidate search + junction-cap retry machinery
//! (`decomposition_search::select_best_decomposition` →
//! `run_layout_with_retry_inner`), each of which can invoke
//! `route_bus_ghost` more than once per fixture (once per evaluated
//! candidate variant, plus a pass-2 retry when pass 1 caps). This
//! census's totals are seed-OBSERVATIONS across every such invocation
//! during a fixture's full build, not deduplicated by physical
//! coordinate — a seed whose geometry is untouched by a retry can appear
//! more than once. This is intentional (every invocation is a real
//! `route_bus_ghost` call the shipped pipeline actually makes), but it
//! means "111 seeds" is not "111 distinct physical crossings in the
//! final shipped layout".
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
        // SAFETY: `std::env::set_var` requires `unsafe` on this toolchain
        // at edition 2021 (verified: rustc 1.95.0 flags an unnecessary
        // `unsafe` block under `#[deny(unused_unsafe)]` for an ordinary
        // safe call — e.g. `unsafe { println!(...) }` — but does NOT flag
        // this one, confirming the wrapper is required, not vestigial;
        // round 3 review's claim that it's "unnecessary under edition
        // 2021" does not hold on this toolchain). This test binary is
        // single-threaded for env mutations — no other thread reads/
        // writes process env concurrently here.
        unsafe {
            std::env::set_var(key, value);
        }
        EnvVarGuard { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: same as `set` above.
        unsafe {
            match &self.prev {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
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
         single-tile, 2 specs, SAME item (rung's exact shape):  {single_tile_same_item}\n\
         single-tile, >2 specs (outside rung's specs.len()==2): {single_tile_gt2_specs}\n\
         multi-tile,  all participants distinct items:          {multi_tile_all_distinct}\n\
         multi-tile,  item-sharing pair somewhere in union:     {multi_tile_same_item}"
    );

    println!(
        "\npipe-tagged seeds (measured on the RAW spec set at each cluster's \
         tiles, BEFORE keys_at_tile's SpecKind::Pipe filter runs — expected \
         0 by construction if #687's pipe×belt finding holds; a nonzero \
         count here would mean pipes DO reach a cluster tile candidate, \
         just always get filtered before solve_crossing): {pipe_tagged_seeds}"
    );
    // Round 3 review finding: enforce the "pipe-tagged == single-spec"
    // identity with assert_eq!, not just prose — if the corpus ever
    // drifts so a pipe-tagged seed isn't single-spec (or vice versa),
    // this test fails loudly instead of the doc silently going stale.
    assert_eq!(
        pipe_tagged_but_not_single_spec, 0,
        "expected every pipe-tagged seed to be single-spec (the belt-crosses-a-placed-pipe bypass) — found one that wasn't"
    );
    assert_eq!(
        single_spec_but_not_pipe_tagged, 0,
        "expected every single-spec seed to be pipe-tagged — found one that wasn't"
    );
    println!(
        "cross-check (assert_eq!-enforced above, not just prose): \
         pipe-tagged & single-spec = {pipe_tagged_and_single_spec}, \
         pipe-tagged but NOT single-spec = {pipe_tagged_but_not_single_spec}, \
         single-spec but NOT pipe-tagged = {single_spec_but_not_pipe_tagged}"
    );

    println!(
        "\nsame-item seeds matching the rung's EXACT predicate \
         (cluster_tile_count == 1 AND n_specs == 2 AND n_distinct_items == 1): {}",
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
}
