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
//! right after `keys_at_tile` is built). Purely observational —
//! `trace::emit` is a no-op unless a collector/sink is active on the
//! thread, so adding it cannot change routing.
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

#[test]
#[ignore = "G1 diagnostic census — run with --ignored --nocapture; see module doc for the zone-cache pin"]
fn junction_seed_census() {
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

    // (n_specs, n_distinct_items) -> occurrence count across the corpus.
    let mut table: std::collections::BTreeMap<(usize, usize), usize> = Default::default();
    let mut total_seeds = 0usize;
    let mut pipe_tagged_seeds = 0usize;
    let mut same_item_seeds: Vec<(String, i32, i32, usize, usize)> = Vec::new();
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
            if let TraceEvent::JunctionSeedCensus { seed_x, seed_y, n_specs, n_distinct_items, has_pipe } = ev
            {
                total_seeds += 1;
                *table.entry((*n_specs, *n_distinct_items)).or_insert(0) += 1;
                if *has_pipe {
                    pipe_tagged_seeds += 1;
                }
                if *n_specs > *n_distinct_items {
                    same_item_seeds.push((label.to_string(), *seed_x, *seed_y, *n_specs, *n_distinct_items));
                }
            }
        }
    }

    println!(
        "\n=== junction seed census: {total_seeds} seeds across {} fixtures ({builds_ok} built, {builds_refused} refused, {solver_skipped} solver-skipped) ===",
        fixtures.len()
    );
    println!("{:>8} {:>9} {:>7}", "n_specs", "distinct", "count");
    for ((n_specs, n_distinct), count) in &table {
        println!("{n_specs:>8} {n_distinct:>9} {count:>7}");
    }
    println!(
        "\npipe-tagged seeds (measured on the RAW spec set at each cluster's \
         tiles, BEFORE keys_at_tile's SpecKind::Pipe filter runs — expected \
         0 by construction if #687's pipe×belt finding holds; a nonzero \
         count here would mean pipes DO reach a cluster tile candidate, \
         just always get filtered before solve_crossing): {pipe_tagged_seeds}"
    );
    println!(
        "\nsame-item seeds (n_specs > n_distinct_items — the G1 open \
         question, same-item belt crossings seeding junctions): {}",
        same_item_seeds.len()
    );
    for (label, x, y, n_specs, n_distinct) in &same_item_seeds {
        println!("  {label} @ ({x},{y}): {n_specs} specs, {n_distinct} distinct items");
    }
}
