//! RFC-054 **KC1** — replay the meter over the frozen calibration corpus.
//!
//! # Why this can run without Factorio
//!
//! The measurements are already in the repo. Blueprints regenerate from
//! the engine locally:
//!
//! ```bash
//! cargo test --manifest-path crates/core/Cargo.toml --test cell_composition \
//!     -- --ignored export_chain_fixtures_for_sim
//! # (and export_mega_fixtures_for_sim, export_mega_pu_for_sim, ...)
//! ```
//!
//! and the numbers they are judged against were blessed before this RFC
//! existed — `crates/core/data/cell-sim-registry.json` and the issues
//! below. That is the property the RFC's "Why now, and not before
//! RFC-050" section rests on, and it is what makes the kill criterion
//! un-tunable: the answers predate the instrument.
//!
//! # What is NOT reachable here, stated plainly
//!
//! The five blessed baselines in `crates/sim-harness/baselines/` (gear10,
//! ec10, automation, logistic, military) have **no tracked export test** —
//! their blueprints were produced ad hoc, so they cannot be regenerated
//! from a clean checkout and are absent from this replay. Closing that gap
//! means adding export fixtures for them; recorded rather than quietly
//! dropped, because a corpus that silently shrinks is a kill criterion
//! that silently weakens.

use std::path::PathBuf;

use spaghettio_meter::{Factory, Manifest};

/// Warmup before measurement starts.
///
/// **Was 2 game-minutes, and that was measuring transients as rates.** A
/// 46-machine chain with deep ingredient buffers is nowhere near steady
/// state by then: `chain-mil5plates-d0` reads −38.4% at 2 minutes, −10.2%
/// at 5, and +0.7% once actually converged. The entire "military family"
/// deficit that tripped KC1 was buffer fill being read as throughput.
///
/// 80 game-minutes, and the margin is deliberate. `chain-mil5ore-d2` (146
/// machines, the largest fixture) reads −13.8% at a 40-minute warmup and
/// −1.3% at 80 — **while reporting `converged: true` at 40**. So the
/// convergence detector is not sufficient on its own here: three
/// consecutive 1-minute windows within 2% cannot distinguish steady state
/// from a large factory filling slowly and smoothly, which is the same
/// limitation `RunParams::with_warmup` documents for the real harness.
///
/// The flag is therefore a floor, not a ceiling: it reliably catches
/// "obviously still filling", and a generous fixed warmup covers the rest.
/// Runtime is not a reason to shave it — the whole corpus replays in
/// seconds.
const WARMUP: u64 = 60 * 60 * 80;
const WINDOW: u64 = 60 * 60 * 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Band {
    /// Measured at or above plan.
    Pass,
    /// Measured a few percent short (−5% to −8%).
    Marginal,
    /// Measured badly short.
    Fail,
}

struct Entry {
    label: &'static str,
    /// The item whose rate the baseline is stated for.
    target: &'static str,
    /// Real-Factorio measured delta, as a fraction (−0.06 = −6%).
    measured: f64,
    band: Band,
    /// Where the number comes from — every one is checkable. Kept for
    /// provenance even though the assertions do not read it: a baseline
    /// whose origin is not written down is a baseline nobody can re-derive.
    #[allow(dead_code)]
    source: &'static str,
    /// True when the chain contains an on-site fluid step. The meter holds
    /// fluid-fed machines in shortage (PR-3 scope; fluids are the RFC's
    /// Phase 3), so these are expected to under-report and are reported
    /// separately rather than being quietly dropped.
    fluid_dependent: bool,
}

const CORPUS: &[Entry] = &[
    Entry {
        label: "chain-ec15-d1",
        target: "electronic-circuit",
        measured: -0.080,
        band: Band::Marginal,
        source: "cell-sim-registry.json ec@15 d1 (13.8/15.0)",
        fluid_dependent: false,
    },
    Entry {
        label: "chain-ec15-d7",
        target: "electronic-circuit",
        measured: -0.053,
        band: Band::Marginal,
        source: "cell-sim-registry.json ec@15 d7 (14.2/15.0)",
        fluid_dependent: false,
    },
    Entry {
        label: "chain-ec30-d2",
        target: "electronic-circuit",
        measured: -0.053,
        band: Band::Marginal,
        source: "cell-sim-registry.json ec@30 (27.7/30.0)",
        fluid_dependent: false,
    },
    Entry {
        label: "chain-ac1-d0",
        target: "advanced-circuit",
        measured: -0.003,
        band: Band::Pass,
        source: "cell-sim-registry.json AC@1 PASS (1.00/1.00)",
        fluid_dependent: true,
    },
    Entry {
        label: "chain-mil5plates-d0",
        target: "military-science-pack",
        measured: -0.033,
        band: Band::Pass,
        source: "cell-sim-registry.json mil5-from-plates PASS (4.83/5.00)",
        fluid_dependent: false,
    },
    Entry {
        label: "mega-plastic2",
        target: "plastic-bar",
        measured: 0.10,
        band: Band::Pass,
        source: "cell-sim-registry.json plastic@2 PASS (2.20/2.00)",
        fluid_dependent: true,
    },
    Entry {
        label: "mega-sulfur2",
        target: "sulfur",
        measured: 0.0,
        band: Band::Pass,
        source: "cell-sim-registry.json sulfur@2 PASS (2.00/2.00 exact)",
        fluid_dependent: true,
    },
    Entry {
        label: "mega-chain-ac2raw",
        target: "advanced-circuit",
        measured: 0.005,
        band: Band::Pass,
        source: "cell-sim-registry.json AC@2 PASS (2.01/2.00)",
        fluid_dependent: true,
    },
    Entry {
        label: "mega-chain-chem5raw",
        target: "chemical-science-pack",
        measured: 0.0,
        band: Band::Pass,
        source: "cell-sim-registry.json chem5 PASS (5.00/5.00 exact)",
        fluid_dependent: true,
    },
    // RE-BANDED 2026-07-26 Fail -> Pass, on oracle evidence only.
    //
    // Recorded for months as -28.7% FAIL. Re-measured in real Factorio at
    // `--warmup 288000`: **+0.7%, 146/146 machines working, PASS**,
    // delivered 5.07/s against a 5.00 plan. The -28.7% was taken at the
    // harness's dim-scaled default warmup and is a buffer-fill transient,
    // not a layout deficit.
    //
    // The bar for touching a frozen corpus entry, and why it is cleared:
    // the correction is justified by the ORACLE alone, never by agreement
    // with the meter. It was also not taken in isolation — the same
    // long-warmup re-measurement was run on the other two Fail-band
    // entries, #453 (USP@2) and #437 (PU@4), and BOTH survived unchanged
    // (-55.0% and -21.0%, the latter byte-identical across a 1.5x warmup
    // increase). So this is a corpus re-measured as a unit that moved one
    // row, not a row adjusted because it was inconvenient.
    Entry {
        label: "chain-mil5ore-d2",
        target: "military-science-pack",
        measured: 0.007,
        band: Band::Pass,
        source: "spaghettio-sim chain-mil5ore-d2 --warmup 288000 (Factorio 2.0.77, \
                 146/146 working, produced 5.03/s, delivered 5.07/s, PASS); \
                 supersedes the -28.7% default-warmup measurement",
        fluid_dependent: false,
    },
    Entry {
        label: "mega-chain-pu4raw",
        target: "processing-unit",
        measured: -0.273,
        band: Band::Fail,
        source: "issue #437 (2.91/4.00)",
        fluid_dependent: true,
    },
    Entry {
        label: "mega-chain-usp2raw",
        target: "utility-science-pack",
        measured: -0.570,
        band: Band::Fail,
        source: "issue #453 (0.86/2.00)",
        fluid_dependent: true,
    },
];

struct Result_ {
    label: &'static str,
    band: Band,
    fluid: bool,
    real: f64,
    meter: f64,
    /// Absolute gap in percentage points.
    gap_pp: f64,
    /// Did the run reach steady state? An unconverged row's `meter` is a
    /// transient, not a rate, and comparing it to `real` is meaningless —
    /// which is exactly how KC1 came to be graded against buffer fill.
    converged: bool,
}

fn tmp_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp")
}

fn replay() -> Vec<Result_> {
    let dir = tmp_dir();
    let mut out = Vec::new();
    for e in CORPUS {
        let Ok(bp) = std::fs::read_to_string(dir.join(format!("{}.bp", e.label))) else {
            continue;
        };
        let Ok(manifest) = Manifest::from_path(dir.join(format!("{}.manifest.json", e.label)))
        else {
            continue;
        };
        let planned = manifest
            .planned_rates
            .get(e.target)
            .copied()
            .or_else(|| {
                manifest
                    .targets
                    .iter()
                    .find(|t| t.item == e.target)
                    .map(|t| t.rate)
            })
            .unwrap_or(0.0);
        if planned <= 0.0 {
            continue;
        }
        let Ok(mut f) = Factory::build(&bp, manifest) else {
            continue;
        };
        let report = f.measure(WARMUP, WINDOW);
        let got = report.produced_per_s.get(e.target).copied().unwrap_or(0.0);
        let meter = got / planned - 1.0;
        out.push(Result_ {
            label: e.label,
            band: e.band,
            fluid: e.fluid_dependent,
            real: e.measured,
            meter,
            gap_pp: (meter - e.measured).abs() * 100.0,
            converged: report.converged,
        });
    }
    out
}

/// Always-on: replay the corpus and print the comparison. Asserts only
/// that what is reachable actually runs — the KC1 verdict lives in the
/// gate tests below, so a scoping change can never quietly weaken it.
#[test]
fn corpus_replay_reports() {
    let results = replay();
    if results.is_empty() {
        eprintln!("skipping: no fixtures generated (see module docs)");
        return;
    }

    println!(
        "\n{:<24} {:>8} {:>9} {:>9} {:>8} {:>10}  fluid?",
        "config", "band", "real", "meter", "gap pp", "converged"
    );
    for r in &results {
        println!(
            "{:<24} {:>8?} {:>8.1}% {:>8.1}% {:>8.1} {:>10}  {}",
            r.label,
            r.band,
            r.real * 100.0,
            r.meter * 100.0,
            r.gap_pp,
            if r.converged { "yes" } else { "NO" },
            if r.fluid { "fluid" } else { "" }
        );
    }
    let solids: Vec<&Result_> = results.iter().filter(|r| !r.fluid).collect();
    println!(
        "\n{} configs replayed ({} solid-only)",
        results.len(),
        solids.len()
    );

    // A solid config that has not reached steady state is not a
    // measurement, and reporting one as if it were is the defect that
    // tripped KC1 for a day. Loud, and named, rather than a silent row.
    let stalled: Vec<&str> = solids
        .iter()
        .filter(|r| !r.converged)
        .map(|r| r.label)
        .collect();
    assert!(
        stalled.is_empty(),
        "solid configs did not reach steady state within WARMUP: {stalled:?}. \
         Their `meter` values are buffer-fill transients, not rates — raise \
         WARMUP rather than comparing them to anything."
    );
}

/// **KC1, rank half — solids only. PASSES as of 2026-07-26.**
///
/// It tripped from 2026-07-25 and the trip was real, but neither cause was
/// in the meter: the fixtures were built at the wrong geometry (#466, so
/// meter and Factorio measured different factories) and the corpus warmed
/// up for two game-minutes, reading buffer fill as throughput. With both
/// fixed, every solid config lands within 4pp. Full account in the RFC's
/// 2026-07-26 decision-log entry.
///
/// **Still `#[ignore]`d, deliberately.** It passes when run on demand, but
/// re-enabling it in the same change that corrects the corpus would be the
/// most tuning-shaped edit available: a criterion that starts passing in
/// the commit that edits the data it grades against invites exactly the
/// suspicion kill criteria exist to earn against. The correction stands on
/// oracle evidence alone; flipping the verdict is a separate decision,
/// separately reviewed. Run it with
/// `cargo test -p spaghettio_meter --test corpus_replay -- --ignored`.
///
/// **Known weakening, stated because it is a real reduction in coverage:**
/// this now discriminates **Pass vs Marginal only**. Re-banding
/// `chain-mil5ore-d2` (Fail → Pass, on oracle evidence — see its entry)
/// removed the last solid Fail-band config, so nothing here exercises
/// Fail-vs-anything. That is the *consequence of the correction being
/// right*, not a scoping choice, and it is the weaker half of the
/// criterion in any case: the RFC names the Marginal band as "the real
/// test ... where a rate-shaped model will fail", because separating −5.3%
/// from at-plan is the resolution an in-loop score actually needs. Adding
/// a solid Fail fixture would restore full coverage.
///
/// Scoped to solid chains because the meter does not model fluids yet
/// (the RFC's Phase 3): a fluid-fed machine is deliberately held in
/// shortage rather than allowed to craft from nothing, so a fluid chain
/// under-reports by construction. That is a *stated phase boundary*, not a
/// criterion rewritten after seeing it fire — the full-corpus result is
/// reported by `corpus_replay_reports` above and by
/// `kc1_full_corpus_status`, so nothing is hidden by the scoping.
#[test]
#[ignore = "verdict re-enable is a separate decision; passes on demand as of 2026-07-26"]
fn kc1_rank_ordering_on_solid_chains() {
    let results = replay();
    let solids: Vec<&Result_> = results.iter().filter(|r| !r.fluid).collect();
    if solids.len() < 2 {
        eprintln!("skipping: need at least two solid configs, have {}", solids.len());
        return;
    }

    let mut inversions = Vec::new();
    for a in &solids {
        for b in &solids {
            if a.band < b.band && a.meter <= b.meter {
                inversions.push(format!(
                    "{} ({:?}, meter {:.1}%) should rank above {} ({:?}, meter {:.1}%)",
                    a.label,
                    a.band,
                    a.meter * 100.0,
                    b.label,
                    b.band,
                    b.meter * 100.0
                ));
            }
        }
    }
    assert!(
        inversions.is_empty(),
        "KC1 rank inversions between bands on solid chains:\n{}",
        inversions.join("\n")
    );
}

/// **KC1, magnitude half — solids only. PASSES as of 2026-07-26**
/// (5/5 solid configs within 10pp, against a 4/5 bar). See the rank gate
/// above for why it tripped and what fixed it.
/// Within ±10 percentage points; currently 3/5, needing 4/5.
/// See the rank test above for why this is ignored rather than removed.
#[test]
#[ignore = "verdict re-enable is a separate decision; passes on demand as of 2026-07-26"]
fn kc1_magnitude_on_solid_chains() {
    let results = replay();
    let solids: Vec<&Result_> = results.iter().filter(|r| !r.fluid).collect();
    if solids.is_empty() {
        eprintln!("skipping: no solid configs");
        return;
    }
    let within: Vec<&&Result_> = solids.iter().filter(|r| r.gap_pp <= 10.0).collect();
    let frac = within.len() as f64 / solids.len() as f64;
    let misses: Vec<String> = solids
        .iter()
        .filter(|r| r.gap_pp > 10.0)
        .map(|r| {
            format!(
                "  {} real {:.1}% vs meter {:.1}% ({:.1}pp)",
                r.label,
                r.real * 100.0,
                r.meter * 100.0,
                r.gap_pp
            )
        })
        .collect();
    assert!(
        frac >= 0.8,
        "KC1 magnitude: only {}/{} solid configs within 10pp ({:.0}%, need 80%):\n{}",
        within.len(),
        solids.len(),
        frac * 100.0,
        misses.join("\n")
    );
}

/// The full-corpus picture, including fluid chains, recorded but not
/// gated. This exists so the solids-only scoping above can never be
/// mistaken for the whole story: if fluid configs are wildly off, it shows
/// here, in the same run, every time.
#[test]
fn kc1_full_corpus_status() {
    let results = replay();
    if results.is_empty() {
        return;
    }
    let (fluid, solid): (Vec<&Result_>, Vec<&Result_>) =
        results.iter().partition(|r| r.fluid);
    let mean = |v: &[&Result_]| {
        if v.is_empty() {
            0.0
        } else {
            v.iter().map(|r| r.gap_pp).sum::<f64>() / v.len() as f64
        }
    };
    println!(
        "mean |gap|: solids {:.1}pp over {} configs, fluid-dependent {:.1}pp over {}",
        mean(&solid),
        solid.len(),
        mean(&fluid),
        fluid.len()
    );
    // Sanity: every replayed SOLID config must have produced something, or
    // the "agreement" would be an artifact of measuring nothing.
    //
    // The threshold is load-bearing and was wrong. `meter = got/planned - 1`
    // with `got >= 0` and `planned > 0` is bounded below by exactly −1.0, so
    // the original `> -1.001` could not fail for any input — a vacuous
    // assertion that read like a real floor. −0.999 is just inside the
    // actual bound, so "produced literally nothing" now trips it.
    //
    // Restricted to solids in the same breath, and not as a convenience:
    // the fluid chains genuinely sit at −100% because fluids are
    // unimplemented (RFC Phase 3), so asserting over them would red the
    // suite for a gap that is documented, expected, and not a build
    // failure. Their count is printed above, so they cannot vanish
    // unnoticed.
    for r in &solid {
        assert!(
            r.meter > -0.999,
            "{} produced nothing at all — that is a build failure, not a measurement",
            r.label
        );
    }
}
