//! W1a (RFC-070 campaign, tracking issue #689): a report-only meter
//! tripwire over a slice of the e2e/census fixture corpus.
//!
//! `crates/meter/` measures a real blueprint in seconds and is calibrated
//! **asymmetrically** (docs/meter-divergence.md): "meter says below plan
//! ⇒ believe it"; "meter says at-or-above plan" is evidence of NOTHING
//! (the floor property does not hold post-lift — see that doc's
//! 2026-08-08 section). Before this file, nothing ran the meter as an
//! automated signal at all. This wires it into a diagnostic so an engine
//! change gets a seconds-scale below-plan regression signal instead of
//! waiting on a headless-Factorio sim run.
//!
//! # Scope (first cut, deliberately modest)
//!
//! 13 fixtures: the G2 census's 6 (`crates/core/tests/
//! check_firing_census.rs`'s `fixtures` table) plus 7 more from the e2e
//! tier ladder (`crates/core/tests/e2e.rs`), covering every tier1-3
//! config buildable without CLI-only machinery this test doesn't use.
//! This test calls `solver`/`layout`/`validate`/`blueprint` **directly**
//! rather than shelling out to `crates/core/examples/sim_export.rs` — the
//! `examples/` dir is gitignored (see CLAUDE.md), so a NEW example file
//! there would not exist on a fresh checkout; calling the same public
//! library API in-process needs no example file at all, tracked or not.
//! That also means the `--exclude`-only `tier3_heavy_oil_cracking` case
//! IS reachable here (`solver::solve_with_exclusions` takes exclusions
//! directly), even though `sim_export.rs`'s CLI has no `--exclude` flag.
//!
//! Left out of this first cut, and why:
//! * The mirror-flag fluid-port gap (`reflect_port` unimplemented,
//!   `meter/factory.rs` ~192-210) is known-open and explicitly OUT of
//!   scope — none of these fixtures exercise `mirror: true` ports.
//! * The full stress/mega corpus (dozens of fixtures, `cell_composition.rs`) —
//!   future work; this is the "manageable slice" the task called for.
//! * Multi-target (`--multi`) fixtures — none of the census/tier-ladder
//!   configs need them.
//!
//! # Hard rule: only BELOW-plan ever alarms
//!
//! Per the meter's calibrated asymmetry, this file must never treat an
//! at-or-above-plan reading as clearance, in code, comments, or printed
//! text. The scoreboard tags a negative deficit `BELOW PLAN`; a
//! non-negative one gets a blank column, never a `PASS`/`OK`/`cleared`
//! word. `check` mode's only failure conditions are (a) "the CURRENT
//! reading is below plan, and materially worse than the committed
//! baseline" and (b) a convergence collapse (below) — never "the reading
//! dropped from a higher baseline while staying non-negative throughout"
//! and never "at-plan, therefore fine". (a) is enforced structurally, not
//! just by convention: the comparison clamps both readings to a ceiling
//! of 0 before differencing (`min(deficit_pct, 0.0)`), so the result can
//! only be positive (alarm-worthy) when the FRESH reading is itself below
//! plan — an above-plan baseline (sulfuric5-chem's committed +239%, for
//! instance) can never manufacture a "regression" out of a reading that
//! improved toward or past plan (second-opinion review on #693 round 1,
//! finding #1 — this was a live false-alarm path, not hypothetical).
//!
//! A convergence collapse (baseline converged, fresh does not) is judged
//! qualitatively as its own below-plan-class failure, not folded into the
//! percentage comparison: a previously-settling factory that no longer
//! settles is exactly the shape of the worst regression this tripwire
//! exists to catch (a stall to ~0 — the meter's own convergence detector
//! treats "producing nothing" specially, see `producing_nothing_is_not_
//! converged` in `factory.rs`), and round 1's shape — treating ANY
//! non-convergence as a graceful skip — silently passed exactly that
//! regression (second-opinion review round 2, finding #1). This does not
//! violate the below-plan-only rule above: a stall is never an
//! at-or-above-plan condition, it just cannot be quantified into a
//! percentage, so it is judged as pass/fail rather than by magnitude.
//! Baseline-side non-convergence is different and stays a skip (see
//! "Standing gaps" below) — the baseline itself was never a trustworthy
//! anchor to measure a regression FROM.
//!
//! # Fluid targets are excluded from bless/check, never from the report
//!
//! `sulfuric5-chem` and `lightoil5-chem-cracking` (the corpus's only two
//! fluid-target fixtures) are measured and printed exactly like every
//! other fixture, but are never written into the committed baseline and
//! never judged in `check` mode (second-opinion review round 2, finding
//! #4). This is NOT because the meter fails to model fluid delivery — it
//! does (`fluid.rs`'s Phase B pipe network; `delivered_per_s` is
//! populated with real, non-zero numbers for both fixtures) — it is
//! because that number has never been calibrated against a real Factorio
//! measurement for these targets at all. docs/meter-divergence.md is
//! explicit on both points: *"Fluid targets are untested. All 4
//! fluid-target fixtures... have no sim baseline in this corpus. The
//! floor verdict is a solid-target-only result,"* and separately
//! documents a DELIBERATE modeling choice that biases fluid delivery
//! upward — *"the meter drains every unconsumed producer fluid unit as
//! delivered and never stalls a producer on a full fluid output"* (the
//! byproduct-backpressure entry) — which is exactly the mechanism behind
//! these two fixtures' measured +239%/+200%: a fractional solved machine
//! count (`count: 0.1`/`0.333`) rounds up to one physical machine whose
//! ingredient delivery isn't duty-cycled down, so it runs faster than
//! planned, and the meter drains all of it as "delivered" rather than
//! modeling the backpressure a real factory would apply. So the
//! calibrated asymmetry this whole file rests on has never been
//! established for these two targets in EITHER direction — gating on
//! them would apply a calibration to a population it was never measured
//! against. `check` prints an explicit `EXCLUDED` line for each, and the
//! scoreboard's `flag` column marks them `fluid (uncalibrated)`
//! regardless of sign.
//!
//! # The committed baseline is this instrument's ZERO, not a live floor
//!
//! `check` never alarms on absolute below-plan — only on a below-plan
//! reading that is MATERIALLY WORSE than what was blessed. A fixture
//! blessed already below plan (`gear20-am2-plate` at -25.0%, for
//! instance) stays clean in `check` forever, until someone deliberately
//! re-blesses a tighter number. This is deliberate (second-opinion review
//! round 2, finding #6, correcting round 1's docs for overselling it) —
//! exactly how `e2e.rs`'s `StressBaseline` ceilings work: a bar to
//! ratchet DOWN as fixes land, not a live absolute threshold. Failing on
//! any below-plan reading regardless of history would make this tripwire
//! permanently red from the moment it is blessed (several fixtures are
//! already below plan today) and useless until every pre-existing deficit
//! is independently fixed first — backwards for a regression gate, which
//! has to be able to bootstrap on a corpus with known, un-actioned
//! issues. Standing below-plan rows are not hidden, though: `check` lists
//! them explicitly (`STANDING BELOW-PLAN`, reported, never failed) so
//! "clean" never reads as "nothing here is wrong."
//!
//! # bless/check protocol (mirrors `SPAGHETTIO_STRESS_GOLDEN`'s shape,
//! but see the note below on why this one is NOT the same design)
//!
//! `SPAGHETTIO_METER_TRIPWIRE` selects the mode:
//!   * unset — **report-only** (the default). Prints the scoreboard and,
//!     loudly, any build failures — and asserts NOTHING. Every assertion
//!     in this file (build failures, the has-a-plan sanity check, the
//!     below-plan comparison itself) is scoped to `bless`/`check` only.
//!     Round 1 already made this promise for the has-a-plan check but
//!     left the build-failure assert unconditional; second-opinion review
//!     round 2 (finding #5) caught the gap. If a new assertion is ever
//!     added to this file, it belongs inside the `bless`/`check` arms,
//!     never before the match.
//!   * `bless` — writes the current per-fixture readings (excluding fluid
//!     targets — see above) as the new committed baseline
//!     (`e2e_tripwire_baseline.json`), alongside a hash of the zone-cache
//!     file used to build them.
//!   * `check` — compares fresh readings against the committed baseline.
//!     Every fixture lands in exactly one bucket, and the run prints an
//!     accounting line totalling them (second-opinion review round 2,
//!     finding #2 — a run where every row is skipped must not read as
//!     "clean", the same failure class `docs/validator-reporting.md`
//!     documents for check severity/count conflation):
//!       - **excluded** — a fluid target (see above); never gated.
//!       - **new** — no baseline row (bless it).
//!       - **stale** — entity count changed vs the baseline; not
//!         comparable (the zone-cache hash recorded at bless time helps
//!         tell "the SAME pin produced a different count" — a genuine
//!         engine change — from "a DIFFERENT pin is in play" — geometry
//!         noise, not a finding).
//!       - **uncovered** — the BASELINE row itself never converged, so it
//!         was never a trustworthy anchor; a standing gap, printed
//!         (`UNCOVERED (baseline non-converged)`), not a fresh failure.
//!       - **collapsed** — baseline converged, fresh does not: FAILS (see
//!         the hard-rule section above).
//!       - **compared** — both converged, entities match: the clamped
//!         below-plan comparison runs; a below-plan reading that's simply
//!         standing (not worse than baseline) prints `STANDING BELOW-PLAN`
//!         and does not fail.
//!
//!     `check` FAILS if nothing landed in `collapsed` or `compared` —
//!     "verified clean" must be distinguishable from "compared nothing".
//!
//! **Why report-only stays the default, and this is NOT wired into CI as
//! a gate**: the committed-golden `check`/`bless` flow that used to sit
//! behind `SPAGHETTIO_STRESS_GOLDEN` was deleted 2026-08-15 (#632 B7) —
//! it was host-cache-relative (crossing-zone geometry, hence entity
//! counts AND throughput, depends on the SAT zone cache's warm/cold
//! state) and nobody ran it, so every committed golden went stale within
//! weeks and produced only false drift signals when finally consulted.
//! This baseline was blessed with the CI zone-cache pin
//! (`SPAGHETTIO_ZONE_CACHE_PATH=$PWD/crates/core/data/sat-zones-ci.bin`)
//! so it is reproducible across hosts, but it is still a NEW instrument
//! with zero track record — gating merges on it now would repeat #632
//! B7's mistake at PR 1. Promoting `check` to a required gate is future
//! work once it has actually been run for a while.
//!
//! Reproduce the committed numbers with:
//! ```text
//! SPAGHETTIO_ZONE_CACHE_PATH=$PWD/crates/core/data/sat-zones-ci.bin \
//!   SPAGHETTIO_METER_TRIPWIRE=check \
//!   cargo test -p spaghettio_meter --test e2e_tripwire -- --ignored --nocapture
//! ```
//! `git checkout -- crates/core/data/sat-zones-ci.bin` afterward — a run
//! against the pin can append newly-solved zones to it, and the pin file
//! must never carry those into a commit.
//!
//! # `PlacedEntity::rate` is not read here
//!
//! This file never reads a layout's per-entity `rate` stamp (always an
//! aggregate, never per-tile flow — docs/rate-stamp-semantics.md). Entity
//! count is used only as a cheap geometry-identity check, and the actual
//! throughput numbers come entirely from the meter's own tick simulation.
//!
//! # Known limitations not (yet) worth more machinery
//! (second-opinion review round 2, findings #7-9 — recorded rather than
//! fixed further, per that review's own "your call, state it" latitude)
//!
//! * The stale-baseline guard keys on entity COUNT only, not a structural
//!   signature — two different layouts with the same entity count would
//!   pass this guard undetected. `golden_hash`-style structural hashing
//!   (`e2e.rs`) would close this; not done here to keep this PR's diff
//!   modest, and entity count plus the zone-cache hash above already
//!   catches the overwhelmingly common cases (added/removed machines,
//!   different cache state).
//! * The baseline records no engine/git-commit provenance beyond the
//!   zone-cache hash — unlike `crates/sim-harness/src/baseline.rs`'s
//!   `game_version`/`mods`/`tech_state` key, there is no "which engine
//!   commit was this blessed against" field. Low value here: unlike the
//!   sim harness (which compares against an external, independently
//!   versioned game binary), this baseline is always compared against
//!   whatever engine commit is checked out, so the comparison is
//!   inherently commit-relative already.

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::{blueprint, solver, validate};
use spaghettio_meter::{Factory, Manifest};

/// Warmup before measurement starts, and the measurement window after.
/// Mirrors `corpus_replay.rs`'s generous fixed warmup: the meter is
/// native/fast, so runtime is not a reason to shave it, and a shorter one
/// reads buffer fill as throughput on anything but the shallowest chains
/// (CLAUDE.md, "Default warmup is too short for deep chains").
const WARMUP: u64 = 60 * 60 * 80;
const WINDOW: u64 = 60 * 60 * 3;

/// How much worse than the committed baseline the BELOW-plan portion of
/// a reading must get before `check` mode alarms — see the comparison
/// itself (in `e2e_tripwire`) for why only the below-plan portion is
/// ever compared. 2.0pp matches the sim-harness's own baseline-check
/// tolerance (`crates/sim-harness/src/baseline.rs`) and comfortably
/// absorbs the tick-window jitter noted in docs/meter-divergence.md's
/// window-quantization section.
const REGRESSION_TOLERANCE_PP: f64 = 2.0;

struct Fixture {
    label: &'static str,
    item: &'static str,
    rate: f64,
    machine: &'static str,
    belt: Option<&'static str>,
    inputs: &'static [&'static str],
    excluded: &'static [&'static str],
}

const NO_EXCLUSIONS: &[&str] = &[];

/// The G2 census's 6 (`check_firing_census.rs`) plus the e2e tier ladder
/// (`crates/core/tests/e2e.rs`), deduped where a tier-ladder config is
/// identical to a census one (`tier1_iron_gear_wheel` == `gear10-am1-plate`
/// below, so it is not repeated).
const FIXTURES: &[Fixture] = &[
    // --- G2 census (check_firing_census.rs's `fixtures` table) ---
    Fixture {
        label: "gear10-am1-plate",
        item: "iron-gear-wheel",
        rate: 10.0,
        machine: "assembling-machine-1",
        belt: None,
        inputs: &["iron-plate"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "ec10-am1-ore",
        item: "electronic-circuit",
        rate: 10.0,
        machine: "assembling-machine-1",
        belt: None,
        inputs: &["iron-ore", "copper-ore"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "ec30-am2-ore",
        item: "electronic-circuit",
        rate: 30.0,
        machine: "assembling-machine-2",
        belt: None,
        inputs: &["iron-ore", "copper-ore"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "plastic5-chem-crude",
        item: "plastic-bar",
        rate: 5.0,
        machine: "chemical-plant",
        belt: None,
        inputs: &["coal", "water", "crude-oil"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "ac5-am2-ore",
        item: "advanced-circuit",
        rate: 5.0,
        machine: "assembling-machine-2",
        belt: None,
        inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "pu2-am3-ore",
        item: "processing-unit",
        rate: 2.0,
        machine: "assembling-machine-3",
        belt: None,
        inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        excluded: NO_EXCLUSIONS,
    },
    // --- e2e tier ladder additions (crates/core/tests/e2e.rs) ---
    Fixture {
        label: "gear10-am2-ore",
        item: "iron-gear-wheel",
        rate: 10.0,
        machine: "assembling-machine-2",
        belt: None,
        inputs: &["iron-ore"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "gear20-am2-plate",
        item: "iron-gear-wheel",
        rate: 20.0,
        machine: "assembling-machine-2",
        belt: None,
        inputs: &["iron-plate"],
        excluded: NO_EXCLUSIONS,
    },
    // Forced yellow belt (`Some("transport-belt")`) — distinct from
    // `ec10-am1-ore` above, which lets the engine mix tiers. Matches
    // `tier2_electronic_circuit_from_ore`'s own comment on why it forces
    // yellow.
    Fixture {
        label: "ec10-am1-ore-yellow",
        item: "electronic-circuit",
        rate: 10.0,
        machine: "assembling-machine-1",
        belt: Some("transport-belt"),
        inputs: &["iron-ore", "copper-ore"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "ec20-am2-ore",
        item: "electronic-circuit",
        rate: 20.0,
        machine: "assembling-machine-2",
        belt: None,
        inputs: &["iron-ore", "copper-ore"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "plastic10-chem-petro",
        item: "plastic-bar",
        rate: 10.0,
        machine: "chemical-plant",
        belt: None,
        inputs: &["petroleum-gas", "coal"],
        excluded: NO_EXCLUSIONS,
    },
    Fixture {
        label: "sulfuric5-chem",
        item: "sulfuric-acid",
        rate: 5.0,
        machine: "chemical-plant",
        belt: None,
        inputs: &["iron-plate", "sulfur", "water"],
        excluded: NO_EXCLUSIONS,
    },
    // tier3_heavy_oil_cracking: exclusions force heavy-oil-cracking (not
    // advanced-oil-processing) as the light-oil producer.
    Fixture {
        label: "lightoil5-chem-cracking",
        item: "light-oil",
        rate: 5.0,
        machine: "chemical-plant",
        belt: None,
        inputs: &["water", "heavy-oil"],
        excluded: &["advanced-oil-processing", "coal-liquefaction"],
    },
];

/// One fixture's meter reading against its plan.
struct Measurement {
    label: &'static str,
    target: String,
    /// "produced" or "delivered" — selected by the target's `is_fluid`
    /// flag (fluids never enter `produced_per_s`; matches
    /// `sweep_corpus.rs`'s metric-matching rule).
    metric: &'static str,
    /// True when the target is a fluid. Excluded from bless/check — see
    /// the module doc's "Fluid targets are excluded" section — but still
    /// measured and reported.
    is_fluid: bool,
    entities: usize,
    planned: f64,
    measured: f64,
    /// `(measured/planned - 1) * 100`. NEGATIVE = below plan, and that is
    /// the only direction this file trusts (for solid targets — see
    /// `is_fluid`). Non-negative is printed for visibility only — never
    /// treated as evidence the layout is correct.
    deficit_pct: f64,
    converged: bool,
}

/// Build a fixture through the engine's own pipeline (solve → layout →
/// validate → export), then hand the resulting blueprint + manifest to
/// the meter exactly as `crates/meter/examples/check_one.rs` does for a
/// disk fixture. The engine calls here construct the artifact under
/// test; they are not part of the measurement, which is entirely
/// `Factory::measure`'s job.
fn build_and_measure(f: &Fixture) -> Result<Measurement, String> {
    let inputs: FxHashSet<String> = f.inputs.iter().map(|s| s.to_string()).collect();
    let excluded: FxHashSet<String> = f.excluded.iter().map(|s| s.to_string()).collect();

    let sr = solver::solve_with_exclusions(f.item, f.rate, &inputs, f.machine, &excluded)
        .map_err(|e| format!("{}: solve failed: {e}", f.label))?;

    let opts = LayoutOptions {
        max_belt_tier: f.belt.map(str::to_string),
        ..Default::default()
    };
    let lay = build_bus_layout(&sr, opts).map_err(|e| format!("{}: layout failed: {e}", f.label))?;

    let issues = validate::validate(&lay, Some(&sr)).unwrap_or_else(|e| e.issues);
    let (bp, manifest_json) = blueprint::export_with_manifest_validated(&lay, &sr, f.label, &issues);

    // Round-trip through JSON, deliberately: the meter's `Manifest` is an
    // independent deserializer by design (KC4 — see `manifest.rs`'s doc
    // comment), so it must be fed the same string a disk fixture would
    // carry, never the engine's in-memory `serde_json::Value`.
    let manifest_str = serde_json::to_string(&manifest_json)
        .map_err(|e| format!("{}: manifest serialize: {e}", f.label))?;
    let manifest =
        Manifest::from_json(&manifest_str).map_err(|e| format!("{}: manifest parse: {e}", f.label))?;
    let target = manifest
        .targets
        .first()
        .cloned()
        .ok_or_else(|| format!("{}: manifest has no targets", f.label))?;
    let entities = lay.entities.len();

    let mut factory =
        Factory::build(&bp, manifest).map_err(|e| format!("{}: factory build failed: {e}", f.label))?;
    let report = factory.measure(WARMUP, WINDOW);

    let planned = report.planned_per_s.get(&target.item).copied().unwrap_or(0.0);
    let (measured, metric) = if target.is_fluid {
        (
            report.delivered_per_s.get(&target.item).copied().unwrap_or(0.0),
            "delivered",
        )
    } else {
        (
            report.produced_per_s.get(&target.item).copied().unwrap_or(0.0),
            "produced",
        )
    };
    let deficit_pct = if planned > 0.0 {
        (measured / planned - 1.0) * 100.0
    } else {
        f64::NAN
    };

    Ok(Measurement {
        label: f.label,
        target: target.item,
        metric,
        is_fluid: target.is_fluid,
        entities,
        planned,
        measured,
        deficit_pct,
        converged: report.converged,
    })
}

/// Print the scoreboard. See the module docs' hard rule: a non-negative
/// deficit gets a blank flag column, never a "PASS"/"OK"/"cleared" word.
/// A fluid target's flag is always `fluid (uncalibrated)` regardless of
/// sign — see the module doc's "Fluid targets are excluded" section.
fn print_scoreboard(results: &[Measurement]) {
    println!(
        "\n{:<24} {:>20} {:>10} {:>10} {:>10} {:>8} {:>10}  flag",
        "fixture", "target", "metric", "planned", "measured", "delta%", "converged"
    );
    for m in results {
        let flag = if m.is_fluid {
            "fluid (uncalibrated)"
        } else if m.deficit_pct.is_nan() {
            "no-plan"
        } else if m.deficit_pct < 0.0 {
            "BELOW PLAN"
        } else {
            ""
        };
        println!(
            "{:<24} {:>20} {:>10} {:>10.3} {:>10.3} {:>+8.1} {:>10}  {}",
            m.label,
            m.target,
            m.metric,
            m.planned,
            m.measured,
            m.deficit_pct,
            if m.converged { "yes" } else { "NO" },
            flag
        );
    }
}

/// One committed baseline row. `entities` is a cheap geometry-identity
/// guard (mirrors `crates/sim-harness/src/baseline.rs`'s game_version/
/// tech_state key): a baseline is only comparable to a fresh run over the
/// SAME layout, and an entity-count mismatch means the engine (or the
/// zone-cache state) produced a different factory, not a real deficit
/// change.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaselineRow {
    label: String,
    target: String,
    entities: usize,
    deficit_pct: f64,
    converged: bool,
}

/// The committed baseline: per-fixture rows plus the zone-cache file's
/// content hash at bless time (second-opinion review round 2, findings
/// #7-9) — lets `check` distinguish "the SAME cache pin produced a
/// different entity count" (a genuine engine change) from "a DIFFERENT
/// cache pin is in play" (the entity mismatch is provenance noise, not a
/// finding). `Option` because a bless run with no resolvable cache file
/// still has to produce something loadable.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Baseline {
    zone_cache_hash: Option<String>,
    rows: Vec<BaselineRow>,
}

fn baseline_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/e2e_tripwire_baseline.json")
}

fn load_baseline() -> Option<Baseline> {
    let text = std::fs::read_to_string(baseline_path()).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_baseline(baseline: &Baseline) {
    let json = serde_json::to_string_pretty(baseline).expect("baseline serializes");
    std::fs::write(baseline_path(), json + "\n").expect("write baseline");
}

/// Mirrors `spaghettio_core::zone_cache::resolve_cache_path`'s exact
/// fallback chain. That function is private to the core crate, so
/// diagnostics elsewhere in the repo that need the same path (e.g.
/// `e2e.rs`'s `diag_sat_zone_histogram`, `diag_decomposition_potential`)
/// each reimplement it rather than share it; this is no exception.
fn resolve_zone_cache_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("SPAGHETTIO_ZONE_CACHE_PATH") {
        return std::path::PathBuf::from(p);
    }
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".cache"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from(".cache"));
    base.join("spaghettio").join("sat-zones.bin")
}

/// A cheap (non-cryptographic) content hash of the zone-cache file, if it
/// exists. Not a security primitive — just a "did this change" signal so
/// `check` can tell a genuine engine change apart from a different cache
/// pin (second-opinion review round 2, findings #7-9).
fn hash_zone_cache() -> Option<String> {
    use std::hash::{Hash, Hasher};
    let bytes = std::fs::read(resolve_zone_cache_path()).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    Some(format!("{:016x}", hasher.finish()))
}

/// Sanity, not a plan-relative verdict: every fixture here solves a
/// single declared target, so a missing planned rate is an export/
/// manifest bug, never a legitimate measurement. Called only from
/// `bless`/`check` (report-only asserts nothing at all — see the module
/// doc's bless/check protocol section).
fn assert_has_plan(results: &[Measurement]) {
    let no_plan: Vec<&str> = results
        .iter()
        .filter(|m| m.deficit_pct.is_nan())
        .map(|m| m.label)
        .collect();
    assert!(
        no_plan.is_empty(),
        "no planned rate found for target on: {no_plan:?} — export/manifest bug"
    );
}

/// A build/measure failure is a pipeline break, not a below-plan reading
/// — but per report-only's "asserts nothing" contract (second-opinion
/// review round 2, finding #5), this is only ever an `assert!` inside
/// `bless`/`check`. Report-only prints failures via `eprintln!` instead,
/// at the call site.
fn assert_no_build_failures(build_failures: &[String]) {
    assert!(
        build_failures.is_empty(),
        "fixture build/measure failed — a build regression, not a below-plan reading:\n{}",
        build_failures.join("\n")
    );
}

/// The verdict `check` mode renders — pulled out of `e2e_tripwire` into a
/// pure function (second-opinion review round 2: findings #1 and #2
/// asked for the discrimination checks to be EXECUTED, not reasoned
/// about, and a live convergence flip cannot be manufactured by mutating
/// the committed baseline file the way round 1's false-alarm scenario
/// could — it needs a synthetic `Measurement`, which this function
/// accepts directly). See `mod tests` below for the executed proofs.
#[derive(Debug, Default)]
struct CheckOutcome {
    /// Below-plan-class failures: a genuine regression (the clamped
    /// comparison) or a convergence collapse. Non-empty ⇒ `check` fails.
    regressions: Vec<String>,
    /// Both converged, entities matched, no regression — includes
    /// standing below-plan rows (see `standing_below_plan`).
    compared: usize,
    /// Baseline converged, fresh did not — counted as "judged" (a
    /// verdict WAS rendered: FAIL) even though it lands in
    /// `regressions`, not `compared`.
    collapsed: usize,
    excluded_fluid: usize,
    skipped_new: usize,
    skipped_stale: usize,
    /// Labels whose BASELINE row never converged — a standing gap, not a
    /// fresh finding.
    uncovered: Vec<String>,
    /// Below-plan readings that are standing (not worse than baseline) —
    /// reported, never failed. See the module doc's "instrument's ZERO"
    /// section.
    standing_below_plan: Vec<String>,
}

impl CheckOutcome {
    /// At least one row got a real verdict (compared clean/standing, OR
    /// collapsed to a failure). Distinguishes "verified clean" from
    /// "compared nothing" (second-opinion review round 2, finding #2).
    fn judged(&self) -> usize {
        self.compared + self.collapsed
    }

    fn not_judged(&self) -> usize {
        self.skipped_new + self.skipped_stale + self.uncovered.len() + self.excluded_fluid
    }
}

/// Compare fresh `results` against the committed `baseline`, printing
/// each row's disposition and returning the aggregate verdict. `verbose`
/// gates the `eprintln!` calls — off for the synthetic unit tests below,
/// on for the real `check` run (`e2e_tripwire`), so a unit test run
/// doesn't spam CI output with fixture labels that don't exist.
fn evaluate_check(results: &[Measurement], baseline: &Baseline, verbose: bool) -> CheckOutcome {
    let mut out = CheckOutcome::default();
    for m in results {
        // Fluid targets are structurally excluded regardless of
        // baseline/convergence state — checked first so they never get
        // misreported as "new fixture" once the baseline no longer
        // carries a row for them.
        if m.is_fluid {
            out.excluded_fluid += 1;
            if verbose {
                eprintln!(
                    "{}: EXCLUDED — fluid target, uncalibrated for gating (no sim baseline \
                     exists for this target; see docs/meter-divergence.md). Reported above, \
                     never gated.",
                    m.label
                );
            }
            continue;
        }
        let Some(b) = baseline.rows.iter().find(|b| b.label == m.label) else {
            out.skipped_new += 1;
            if verbose {
                eprintln!("{}: SKIPPED — no baseline row (new fixture — bless it)", m.label);
            }
            continue;
        };
        if b.entities != m.entities {
            out.skipped_stale += 1;
            if verbose {
                eprintln!(
                    "{}: SKIPPED — entity count changed ({} -> {}); geometry moved, baseline \
                     is stale and not comparable. Re-bless deliberately.",
                    m.label, b.entities, m.entities
                );
            }
            continue;
        }
        if !b.converged {
            // The baseline itself was never a trustworthy anchor — round
            // 1 treated this identically to a fresh-side collapse (both
            // were "skip"); round 2 splits them, because only THIS
            // direction is a standing gap rather than a fresh finding
            // (second-opinion review round 2, finding #1).
            out.uncovered.push(m.label.to_string());
            if verbose {
                eprintln!(
                    "{}: UNCOVERED (baseline non-converged) — this fixture has never had a \
                     trustworthy baseline reading to compare against; a standing gap, not a \
                     fresh finding.",
                    m.label
                );
            }
            continue;
        }
        if !m.converged {
            // Baseline WAS a trustworthy anchor; fresh no longer
            // converges. Treated as its own below-plan-class failure —
            // see the module doc's hard-rule section.
            out.collapsed += 1;
            out.regressions.push(format!(
                "{}: CONVERGENCE COLLAPSE — baseline converged at {:.1}%, fresh no longer \
                 converges. A stalled/collapsed factory is exactly the regression class this \
                 guard exists to catch.",
                m.label, b.deficit_pct
            ));
            continue;
        }
        out.compared += 1;
        // The ONLY percentage-based alarm condition, per the module
        // docs' hard rule: compare only the BELOW-plan portion of each
        // reading (clamped to a ceiling of 0) so an above-plan baseline
        // can never manufacture a "regression" out of a fresh reading
        // that moved toward plan or into a trustworthy below-plan range.
        // `drop` is provably <= 0 whenever the fresh reading is
        // at-or-above plan, so this can only fire when the CURRENT
        // reading is itself below plan (second-opinion review round 1,
        // finding #1).
        let baseline_below = b.deficit_pct.min(0.0);
        let fresh_below = m.deficit_pct.min(0.0);
        let drop = baseline_below - fresh_below;
        if drop > REGRESSION_TOLERANCE_PP {
            out.regressions.push(format!(
                "{}: BELOW-PLAN REGRESSION — baseline {:.1}%, now {:.1}% ({:.1}pp worse, \
                 below-plan portion only)",
                m.label, b.deficit_pct, m.deficit_pct, drop
            ));
        } else if m.deficit_pct < 0.0 {
            // Standing, accepted deficit — the blessed state IS this
            // instrument's zero (module doc section above). Reported so
            // "clean" never reads as "nothing here is wrong"
            // (second-opinion review round 2, finding #6).
            out.standing_below_plan.push(format!(
                "{}: STANDING BELOW-PLAN — {:.1}% (baseline {:.1}%), not a new regression; \
                 accepted at bless time as this fixture's zero point.",
                m.label, m.deficit_pct, b.deficit_pct
            ));
        }
    }
    if verbose {
        for line in &out.standing_below_plan {
            eprintln!("{line}");
        }
    }
    out
}

/// W1a: the tripwire itself. See the module docs for the three modes.
#[test]
#[ignore = "diagnostic tripwire — run with --ignored; see module docs for bless/check"]
fn e2e_tripwire() {
    let mut results = Vec::new();
    let mut build_failures = Vec::new();
    for f in FIXTURES {
        match build_and_measure(f) {
            Ok(m) => results.push(m),
            Err(e) => build_failures.push(e),
        }
    }

    if !build_failures.is_empty() {
        eprintln!(
            "fixture build/measure failed — a build regression, not a below-plan \
             reading:\n{}",
            build_failures.join("\n")
        );
    }

    print_scoreboard(&results);

    match std::env::var("SPAGHETTIO_METER_TRIPWIRE").as_deref() {
        Ok("bless") => {
            assert_no_build_failures(&build_failures);
            assert_has_plan(&results);
            let rows: Vec<BaselineRow> = results
                .iter()
                // Fluid targets are never blessed — see the module doc's
                // "Fluid targets are excluded" section.
                .filter(|m| !m.is_fluid)
                .map(|m| BaselineRow {
                    label: m.label.to_string(),
                    target: m.target.clone(),
                    entities: m.entities,
                    deficit_pct: m.deficit_pct,
                    converged: m.converged,
                })
                .collect();
            let baseline = Baseline {
                zone_cache_hash: hash_zone_cache(),
                rows,
            };
            write_baseline(&baseline);
            eprintln!(
                "BLESSED {} fixture row(s) to {:?} (zone-cache hash: {:?})",
                baseline.rows.len(),
                baseline_path(),
                baseline.zone_cache_hash
            );
        }
        Ok("check") => {
            assert_no_build_failures(&build_failures);
            assert_has_plan(&results);
            let baseline = load_baseline().expect(
                "SPAGHETTIO_METER_TRIPWIRE=check needs a committed baseline; bless one first",
            );
            let fresh_hash = hash_zone_cache();
            if fresh_hash != baseline.zone_cache_hash {
                eprintln!(
                    "NOTE: zone-cache hash differs from the blessed baseline's ({:?} vs \
                     {:?}) — entity-count mismatches below may be caused by this rather \
                     than a genuine engine change.",
                    baseline.zone_cache_hash, fresh_hash
                );
            }

            let outcome = evaluate_check(&results, &baseline, true);
            eprintln!(
                "\naccounting: {}/{} judged ({} compared clean-or-standing, {} \
                 collapsed-to-failure), {}/{} not judged ({} new, {} stale, {} uncovered \
                 [baseline never converged], {} excluded [fluid, uncalibrated])",
                outcome.judged(),
                results.len(),
                outcome.compared,
                outcome.collapsed,
                outcome.not_judged(),
                results.len(),
                outcome.skipped_new,
                outcome.skipped_stale,
                outcome.uncovered.len(),
                outcome.excluded_fluid
            );

            // "Verified clean" must be distinguishable from "compared
            // nothing" (second-opinion review round 2, finding #2) — a
            // missing/wrong zone-cache pin can make every row
            // entity-mismatch, and that must not print a green check.
            assert!(
                outcome.judged() > 0,
                "check compared NOTHING — every row was skipped, excluded, or uncovered. \
                 This is indistinguishable from a wrong or missing SPAGHETTIO_ZONE_CACHE_PATH \
                 pin (every row would then entity-mismatch) and must not read as 'clean'. \
                 See the accounting line above for which reason dominated."
            );
            assert!(
                outcome.regressions.is_empty(),
                "meter tripwire: below-plan regression(s) vs the committed baseline:\n{}",
                outcome.regressions.join("\n")
            );
        }
        _ => {
            // Report-only default — see module docs. No assertion of any
            // kind here, deliberately (second-opinion review round 2,
            // finding #5): build failures above are printed, not
            // asserted, and neither `assert_has_plan` nor the below-plan
            // comparison ever runs in this branch.
        }
    }
}

/// Executed discrimination proofs for `evaluate_check` (second-opinion
/// review round 2 on #693: "execute the discrimination checks... don't
/// reason it"). Round 1's proofs mutated the committed baseline JSON and
/// re-ran the full solve→layout→meter pipeline — that works when only
/// the BASELINE side of a scenario needs to change, but a live
/// convergence flip is a property of a real meter run and cannot be
/// manufactured that way. Pulling the comparison into `evaluate_check`
/// makes both kinds of scenario directly constructible with synthetic
/// data, at unit-test speed.
#[cfg(test)]
mod tests {
    use super::*;

    fn measurement(label: &'static str, deficit_pct: f64, entities: usize, converged: bool) -> Measurement {
        Measurement {
            label,
            target: "widget".to_string(),
            metric: "produced",
            is_fluid: false,
            entities,
            planned: 10.0,
            measured: 10.0 * (1.0 + deficit_pct / 100.0),
            deficit_pct,
            converged,
        }
    }

    fn fluid_measurement(label: &'static str, deficit_pct: f64) -> Measurement {
        let mut m = measurement(label, deficit_pct, 100, true);
        m.is_fluid = true;
        m.metric = "delivered";
        m
    }

    fn baseline_row(label: &str, deficit_pct: f64, entities: usize, converged: bool) -> BaselineRow {
        BaselineRow {
            label: label.to_string(),
            target: "widget".to_string(),
            entities,
            deficit_pct,
            converged,
        }
    }

    fn baseline(rows: Vec<BaselineRow>) -> Baseline {
        Baseline {
            zone_cache_hash: Some("test-hash".to_string()),
            rows,
        }
    }

    /// Finding #1: baseline converged, fresh does not — must FAIL as a
    /// collapse, not silently skip (round 1's bug: uniform non-convergence
    /// skipping let a throughput collapse to ~0 pass green).
    #[test]
    fn convergence_collapse_fails() {
        let results = vec![measurement("x", -1.0, 100, false)];
        let base = baseline(vec![baseline_row("x", -1.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert_eq!(out.collapsed, 1, "must be counted as collapsed");
        assert_eq!(out.judged(), 1, "a collapse IS a rendered verdict");
        assert!(
            out.regressions.iter().any(|r| r.contains("COLLAPSE")),
            "expected a COLLAPSE regression, got {:?}",
            out.regressions
        );
    }

    /// Finding #1 (other direction): the baseline itself never converged
    /// — this stays a skip (an "uncovered" standing gap), never a
    /// failure, regardless of what the fresh run does.
    #[test]
    fn baseline_never_converged_is_uncovered_not_failed() {
        for fresh_converged in [true, false] {
            let results = vec![measurement("x", -1.0, 100, fresh_converged)];
            let base = baseline(vec![baseline_row("x", -1.0, 100, false)]);
            let out = evaluate_check(&results, &base, false);
            assert!(out.regressions.is_empty(), "must not fail: {:?}", out.regressions);
            assert_eq!(out.uncovered, vec!["x".to_string()]);
            assert_eq!(out.judged(), 0, "an uncovered row is not a rendered verdict");
        }
    }

    /// Finding #2: every row entity-mismatches (e.g. a wrong/missing
    /// zone-cache pin) — `judged()` must read 0, which is exactly the
    /// condition `e2e_tripwire`'s own `assert!(outcome.judged() > 0, ...)`
    /// gates on. This is the discrimination check for "verified clean"
    /// vs "compared nothing".
    #[test]
    fn all_rows_stale_yields_zero_judged() {
        let results = vec![
            measurement("a", -1.0, 200, true),
            measurement("b", 0.0, 300, true),
        ];
        let base = baseline(vec![
            baseline_row("a", -1.0, 999, true),
            baseline_row("b", 0.0, 999, true),
        ]);
        let out = evaluate_check(&results, &base, false);
        assert_eq!(out.judged(), 0, "every row should have entity-mismatched");
        assert_eq!(out.skipped_stale, 2);
        assert!(out.regressions.is_empty(), "a stale baseline must not itself fail the test");
    }

    /// Round 1, finding #1, re-verified after the round-2 refactor: an
    /// above-plan baseline (sulfuric5-chem/lightoil5-chem-cracking's
    /// committed +239%/+200% shape) moving to a near-plan fresh reading
    /// must NOT read as a regression — the exact false-alarm the
    /// unclamped `b.deficit_pct - m.deficit_pct` formula produced.
    #[test]
    fn above_plan_baseline_moving_to_near_plan_does_not_alarm() {
        let results = vec![measurement("x", -1.0, 100, true)];
        let base = baseline(vec![baseline_row("x", 200.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert!(
            out.regressions.is_empty(),
            "an improving reading must never alarm: {:?}",
            out.regressions
        );
        assert_eq!(out.compared, 1);
    }

    /// A genuine below-plan regression must still fail after the
    /// clamped-comparison fix — the fix must narrow the false-positive
    /// window, not remove true positives.
    #[test]
    fn genuine_below_plan_regression_still_fails() {
        let results = vec![measurement("x", -25.0, 100, true)];
        let base = baseline(vec![baseline_row("x", -2.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert!(
            out.regressions.iter().any(|r| r.contains("REGRESSION")),
            "expected a below-plan regression, got {:?}",
            out.regressions
        );
    }

    /// Finding #6: a standing below-plan reading (unchanged from
    /// baseline) is reported, never failed — the blessed state is this
    /// instrument's zero, not a live absolute floor.
    #[test]
    fn standing_below_plan_is_reported_not_failed() {
        let results = vec![measurement("x", -25.0, 100, true)];
        let base = baseline(vec![baseline_row("x", -25.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert!(out.regressions.is_empty());
        assert_eq!(out.standing_below_plan.len(), 1);
        assert!(out.standing_below_plan[0].contains("STANDING BELOW-PLAN"));
    }

    /// Finding #4: a fluid target is excluded from the verdict entirely
    /// — even one with a huge apparent below-plan swing must not fail,
    /// because the meter's fluid-delivery model has no sim-baseline
    /// calibration to trust in either direction.
    #[test]
    fn fluid_targets_are_excluded_from_verdict() {
        let results = vec![fluid_measurement("x", -90.0)];
        let base = baseline(vec![baseline_row("x", 0.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert_eq!(out.excluded_fluid, 1);
        assert_eq!(out.judged(), 0);
        assert!(out.regressions.is_empty());
    }

    /// A fixture with no baseline row at all is a "new fixture — bless
    /// it" skip, never a failure.
    #[test]
    fn new_fixture_without_baseline_is_skipped_not_failed() {
        let results = vec![measurement("brand-new", -50.0, 100, true)];
        let base = baseline(vec![baseline_row("other", 0.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert_eq!(out.skipped_new, 1);
        assert!(out.regressions.is_empty());
    }
}
