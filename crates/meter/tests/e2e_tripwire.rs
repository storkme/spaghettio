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
//! word. `check` mode's only failure condition is "the CURRENT reading is
//! below plan, and materially worse than the committed baseline" — never
//! "the reading dropped from a higher baseline while staying non-negative
//! throughout" and never "at-plan, therefore fine". Enforced structurally,
//! not just by convention: the comparison clamps both readings to a
//! ceiling of 0 before differencing (`min(deficit_pct, 0.0)`), so the
//! result can only be positive (alarm-worthy) when the FRESH reading is
//! itself below plan — an above-plan baseline (sulfuric5-chem's committed
//! +239%, for instance) can never manufacture a "regression" out of a
//! reading that improved toward or past plan (second-opinion review on
//! #693, finding #1 — this was a live false-alarm path, not hypothetical).
//! Everything that ISN'T that one comparison — a stale baseline (entity
//! count moved), a reading that isn't converged on either side, or a
//! fixture with no baseline row at all — cannot be validly judged either
//! way, so it is printed as a skip and never fails the test; failing on
//! those would itself violate the below-plan-only contract (finding #2).
//!
//! # bless/check protocol (mirrors `SPAGHETTIO_STRESS_GOLDEN`'s shape,
//! but see the note below on why this one is NOT the same design)
//!
//! `SPAGHETTIO_METER_TRIPWIRE` selects the mode:
//!   * unset — **report-only** (the default). Prints the scoreboard,
//!     asserts only that every fixture built and measured something (a
//!     build regression must not hide as a silently-empty table); no
//!     plan-relative verdict, and no other assertion — the
//!     manifest-has-a-plan sanity check below only runs under `bless`/
//!     `check`, where a missing plan would otherwise corrupt what gets
//!     committed or compared.
//!   * `bless` — writes the current per-fixture readings as the new
//!     committed baseline (`e2e_tripwire_baseline.json`).
//!   * `check` — compares fresh readings against the committed baseline.
//!     **Fails only on a genuine below-plan regression**: the fresh
//!     reading is itself below plan AND its below-plan portion is
//!     materially worse than the baseline's (see the hard-rule section
//!     above for the clamped-comparison mechanism). Three things that
//!     cannot be validly judged are printed and SKIPPED rather than
//!     failed — none of them is a below-plan finding, so none may fail
//!     the test: a fixture missing from the baseline (new — bless it), a
//!     baseline stale for the current geometry (entity count changed —
//!     comparing them would be apples to oranges), and a reading that
//!     isn't converged on either side (the deficit number isn't
//!     trustworthy at all in that case, not just "trustworthy but
//!     different" — matches `corpus_replay.rs`'s own treatment of an
//!     unconverged reading as unusable, not merely suspect).
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
    entities: usize,
    planned: f64,
    measured: f64,
    /// `(measured/planned - 1) * 100`. NEGATIVE = below plan, and that is
    /// the only direction this file trusts. Non-negative is printed for
    /// visibility only — never treated as evidence the layout is correct.
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
        entities,
        planned,
        measured,
        deficit_pct,
        converged: report.converged,
    })
}

/// Print the scoreboard. See the module docs' hard rule: a non-negative
/// deficit gets a blank flag column, never a "PASS"/"OK"/"cleared" word.
fn print_scoreboard(results: &[Measurement]) {
    println!(
        "\n{:<24} {:>20} {:>10} {:>10} {:>10} {:>8} {:>10}  flag",
        "fixture", "target", "metric", "planned", "measured", "delta%", "converged"
    );
    for m in results {
        let flag = if m.deficit_pct.is_nan() {
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

fn baseline_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/e2e_tripwire_baseline.json")
}

fn load_baseline() -> Option<Vec<BaselineRow>> {
    let text = std::fs::read_to_string(baseline_path()).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_baseline(rows: &[BaselineRow]) {
    let json = serde_json::to_string_pretty(rows).expect("baseline rows serialize");
    std::fs::write(baseline_path(), json + "\n").expect("write baseline");
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
    assert!(
        build_failures.is_empty(),
        "fixture build/measure failed — a build regression, not a below-plan reading:\n{}",
        build_failures.join("\n")
    );

    print_scoreboard(&results);

    match std::env::var("SPAGHETTIO_METER_TRIPWIRE").as_deref() {
        Ok("bless") => {
            assert_has_plan(&results);
            let rows: Vec<BaselineRow> = results
                .iter()
                .map(|m| BaselineRow {
                    label: m.label.to_string(),
                    target: m.target.clone(),
                    entities: m.entities,
                    deficit_pct: m.deficit_pct,
                    converged: m.converged,
                })
                .collect();
            write_baseline(&rows);
            eprintln!("BLESSED {} fixture row(s) to {:?}", rows.len(), baseline_path());
        }
        Ok("check") => {
            assert_has_plan(&results);
            let baseline = load_baseline().expect(
                "SPAGHETTIO_METER_TRIPWIRE=check needs a committed baseline; bless one first",
            );
            let mut regressions = Vec::new();
            for m in &results {
                // Three "cannot judge this one" cases. None of these is a
                // below-plan finding, so per the below-plan-only contract
                // (module docs; second-opinion review on #693, finding
                // #2) none of them may fail the test — printed and
                // skipped only.
                let Some(b) = baseline.iter().find(|b| b.label == m.label) else {
                    eprintln!("{}: SKIPPED — no baseline row (new fixture — bless it)", m.label);
                    continue;
                };
                if b.entities != m.entities {
                    eprintln!(
                        "{}: SKIPPED — entity count changed ({} -> {}); geometry moved, \
                         baseline is stale and not comparable. Re-bless deliberately.",
                        m.label, b.entities, m.entities
                    );
                    continue;
                }
                if !b.converged || !m.converged {
                    // Either side, not just a flip (finding #3): both
                    // sides unconverged fell through this guard when it
                    // only checked `b.converged != m.converged`, despite
                    // the whole point being that an unconverged reading
                    // isn't trustworthy full stop — matches
                    // `corpus_replay.rs`'s own treatment (an unconverged
                    // solid config is a buffer-fill transient, not a
                    // rate, regardless of what it's being compared to).
                    eprintln!(
                        "{}: SKIPPED — not converged (baseline={}, fresh={}); the deficit \
                         reading is not trustworthy, so it is not compared either way.",
                        m.label, b.converged, m.converged
                    );
                    continue;
                }
                // The ONLY alarm condition, per the module docs' hard
                // rule: compare only the BELOW-plan portion of each
                // reading (clamped to a ceiling of 0) so an above-plan
                // baseline can never manufacture a "regression" out of a
                // fresh reading that moved toward plan or into a
                // trustworthy below-plan range. `drop` is provably <= 0
                // whenever the fresh reading is at-or-above plan
                // (`fresh_below` is then exactly 0, and `baseline_below`
                // is always <= 0), so this can only fire when the CURRENT
                // reading is itself below plan (second-opinion review on
                // #693, finding #1 — this exact shape, an above-plan
                // baseline moving toward plan, was a live false alarm
                // before this fix: sulfuric5-chem/lightoil5-chem-cracking
                // are committed at +239%/+200%).
                let baseline_below = b.deficit_pct.min(0.0);
                let fresh_below = m.deficit_pct.min(0.0);
                let drop = baseline_below - fresh_below;
                if drop > REGRESSION_TOLERANCE_PP {
                    regressions.push(format!(
                        "{}: BELOW-PLAN REGRESSION — baseline {:.1}%, now {:.1}% ({:.1}pp \
                         worse, below-plan portion only)",
                        m.label, b.deficit_pct, m.deficit_pct, drop
                    ));
                }
            }
            assert!(
                regressions.is_empty(),
                "meter tripwire: below-plan regression(s) vs the committed baseline:\n{}",
                regressions.join("\n")
            );
        }
        _ => {
            // Report-only default — see module docs. No plan-relative
            // assertion here, deliberately, and no OTHER assertion either
            // (the manifest-has-a-plan sanity check is bless/check-only —
            // second-opinion review on #693, finding #4: report-only
            // must not be able to fail a test over anything but the
            // build-failure check above).
        }
    }
}

/// Sanity, not a plan-relative verdict: every fixture here solves a
/// single declared target, so a missing planned rate is an export/
/// manifest bug, never a legitimate measurement. Gated to `bless`/`check`
/// only (called from within those match arms) — a NaN deficit would
/// otherwise corrupt what gets committed or compared, but report-only
/// mode has nothing to protect and must not assert on it (finding #4).
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
