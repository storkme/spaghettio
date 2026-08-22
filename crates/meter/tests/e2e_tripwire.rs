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
//! plan — an above-plan baseline can never manufacture a "regression" out
//! of a reading that improved toward or past plan (second-opinion review
//! on #693 round 1, finding #1 — this was a live false-alarm path, not
//! hypothetical: found against `sulfuric5-chem`'s then-committed +239%,
//! back when fluid targets were still written to the baseline — round 2
//! excluded them entirely, see below, so that specific row is no longer
//! an example of anything committed, only of a MEASURED reading).
//!
//! A convergence collapse (baseline converged, fresh does not) is a
//! below-plan-class failure precisely when the fresh reading is ALSO
//! materially worse than the baseline: a previously-settling factory that
//! no longer settles AND has genuinely gotten worse is exactly the shape
//! of the worst regression this tripwire exists to catch (a stall to ~0
//! — the meter's own convergence detector treats "producing nothing"
//! specially, see `producing_nothing_is_not_converged` in `factory.rs`),
//! and round 1's shape — treating ANY non-convergence as a graceful skip
//! — silently passed exactly that regression (second-opinion review
//! round 2, finding #1).
//!
//! **Getting the gate right took three rounds, and it's worth naming
//! why**: round 2/3 failed on ANY non-convergence, which also fails on
//! good news — the detector rejects any unstable trajectory, rising or
//! falling (`factory.rs`'s `a_filling_buffer_is_not_converged` covers the
//! rising case too), so an over-producing or still-settling factory would
//! fail exactly like a stall (round 4, finding #1). Round 4's fix (a raw
//! sign check, `deficit_pct < 0.0`, no tolerance) stopped failing on
//! ABOVE-plan improvement, but not on two more shapes: a below-plan
//! baseline IMPROVING while non-converged (blessed -25%, fresh -5% — -5%
//! is still negative, so it still failed, despite being much better) and
//! any sub-tolerance non-converged flap (a reading that would land in
//! `standing`, not fail, if it happened to converge) — because a raw sign
//! check has no tolerance at all, inconsistent with the compared branch's
//! `REGRESSION_TOLERANCE_PP` (round 5, finding #1). The collapse branch
//! now shares the EXACT SAME tolerance-gated clamped comparison
//! (`clamped_drop`) as the compared branch below; only the label differs
//! (COLLAPSE, so the reader knows convergence was also lost). A
//! non-converged reading that ISN'T materially worse than baseline is
//! `UNSETTLED` instead — reported, needs a stable re-run, never failed.
//!
//! Baseline-side non-convergence is different again and stays a skip
//! (see "Standing gaps" below) — the baseline itself was never a
//! trustworthy anchor to measure a regression FROM.
//!
//! **A baseline-non-converged fixture can never gain collapse coverage
//! while it stays blessed non-converged** (second-opinion review round
//! 3, finding #1). Two of the eleven blessed rows are affected today,
//! not just one (round 3's text named only the first and read as if it
//! were unique — round 4, finding #5): `ec30-am2-ore` (its 3
//! `belt-dead-end` validator errors) and `plastic5-chem-crude` (no
//! validator errors, and NOT a below-plan deficit either — it reads
//! +0.89%, above plan; round 5 nit — a prior revision of this comment
//! mischaracterized it as a "shortfall". The real issue is that its
//! reading simply never stabilizes within the warmup window). Both hit
//! the "baseline never converged" branch before the
//! collapse branch is ever reached, so a further regression on either is
//! structurally unrepresentable today. This is not a gap in the check's
//! design: an instrument cannot alarm on "worse than a baseline that was
//! already producing nothing" — there is no direction left to regress
//! FROM. It is inherent to any blessed-baseline design, not something a
//! smarter comparison formula fixes. `ec30-am2-ore`'s real fix is the
//! layout itself — queued campaign-side as the W1c contested sample;
//! `plastic5-chem-crude`'s is unexamined (out of scope for this
//! diagnostic PR to chase). The flip condition, so this is not
//! permanently invisible: once either fixture is fixed and RE-BLESSED
//! converged, collapse coverage arms itself automatically for it — no
//! code change needed, just a bless run over the repaired layout. Until
//! then, `check` prints a `RECOVERED` line (see the protocol section
//! below) if a fresh run ever converges again on a still-non-converged
//! baseline, so a fix landing is visible immediately rather than
//! silently waiting for someone to think to re-bless.
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
//!     never before the match. A missing planned rate (`no-plan` in the
//!     scoreboard's flag column) is likewise printed, not alarmed on, in
//!     this mode — by design, not an oversight (round 5 nit): report-only
//!     has nothing to protect by asserting on it, unlike `bless`/`check`,
//!     where a NaN deficit would corrupt what gets committed or compared.
//!   * `bless` — writes the current per-fixture readings (excluding fluid
//!     targets — see above) as the new committed baseline
//!     (`e2e_tripwire_baseline.json`), alongside a hash of the zone-cache
//!     PIN — captured before any fixture is built, not after (second-
//!     opinion review round 3, finding #2: building can append newly-
//!     solved zones to the cache file, so hashing after every fixture had
//!     already run described post-run bytes, not the committed pin that
//!     was actually consulted as the starting state).
//!   * `check` — compares fresh readings against the committed baseline.
//!     Every fixture lands in exactly one bucket, and the run prints an
//!     accounting line totalling them (second-opinion review round 2,
//!     finding #2 — a run where every row is skipped must not read as
//!     "clean", the same failure class `docs/validator-reporting.md`
//!     documents for check severity/count conflation):
//!       - **excluded** — a fluid target (see above); never gated.
//!       - **new** — no baseline row (bless it).
//!       - **target-changed** — the fixture's solved TARGET item differs
//!         from the baseline's; a different KIND of drift than a
//!         geometry change (a config edit under the same label, not the
//!         engine/cache producing a different layout), so its own
//!         accounting reason rather than folded into `stale`
//!         (`BaselineRow.target` was written at bless time but never
//!         read here until this — second-opinion review round 5).
//!       - **stale** — entity count changed vs the baseline; not
//!         comparable (the zone-cache hash recorded at bless time helps
//!         tell "the SAME pin produced a different count" — a genuine
//!         engine change — from "a DIFFERENT pin is in play" — geometry
//!         noise, not a finding).
//!       - **uncovered** — the BASELINE row itself never converged, so it
//!         was never a trustworthy anchor; a standing gap, printed
//!         (`UNCOVERED (baseline non-converged)`), not a fresh failure.
//!       - **recovered** — the BASELINE never converged, but the FRESH
//!         run does: printed (`RECOVERED — re-bless to arm coverage`),
//!         not a fresh failure, and not `judged` either (there is still
//!         no trustworthy number to compare against) — but visible,
//!         which `uncovered` alone would not make it (second-opinion
//!         review round 3, finding #1).
//!       - **collapsed** — baseline converged, fresh does not, AND the
//!         SAME tolerance-gated clamped comparison used for `compared`
//!         below reads the fresh side as materially worse: FAILS,
//!         prints `CONVERGENCE COLLAPSE` (see the hard-rule section
//!         above for why this took three rounds to get right).
//!       - **unsettled** — baseline converged, fresh does not, but that
//!         same comparison does NOT read it as materially worse: printed
//!         (`UNSETTLED`), not a failure and not `judged` either — a
//!         rising buffer, a still-settling improvement (even while
//!         remaining below plan), or plain jitter, none of which this
//!         gate can tell apart from each other, only from a genuine
//!         regression (second-opinion review rounds 4 and 5, finding #1
//!         both times — collapse used to fire on non-convergence alone,
//!         then on raw deficit sign with no tolerance; neither survived
//!         contact with a below-plan-but-improving or sub-tolerance-flap
//!         reading).
//!       - **compared** — both converged, entities match: the clamped
//!         below-plan comparison (`clamped_drop`) runs and lands in
//!         exactly one of three buckets (second-opinion review round 3,
//!         finding #3 — the previous single `compared` counter
//!         incremented BEFORE the regression check, so a row that FAILED
//!         still printed as "compared clean-or-standing" in the
//!         accounting, the exact count/severity conflation
//!         `docs/validator-reporting.md` polices): **at-or-above** (no
//!         below-plan reading at all — not named "clean": that's a
//!         clearance word, and an at-or-above reading is evidence of
//!         nothing either way, second-opinion review round 4, finding
//!         #2), or **standing** (below-plan but not worse than baseline
//!         — prints `STANDING BELOW-PLAN`, does not fail), or
//!         **regressed** (below-plan AND materially worse — FAILS,
//!         prints `BELOW-PLAN REGRESSION`).
//!
//!     `check` FAILS if nothing landed in `collapsed`, `regressed`,
//!     `standing`, or `at-or-above` — a check run that judged nothing
//!     must be distinguishable from one that judged everything and found
//!     no problems.
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
//! * A fixture blessed right at the convergence boundary — `ac5-am2-ore`
//!   at -0.2%, the closest-to-plan below-plan reading in the corpus —
//!   could in principle flap non-converged on a noisy host (CPU
//!   contention widening the tick-timing jitter the convergence detector
//!   samples) (second-opinion review round 3, nit). **Resolved by round
//!   5's fix, not still open**: the collapse branch now shares
//!   `compared`'s tolerance-gated `clamped_drop`, so a flap that stays
//!   within `REGRESSION_TOLERANCE_PP` of the blessed value lands in
//!   `UNSETTLED`, the same way it would land in `standing` if it
//!   happened to converge — it can no longer read as a false
//!   `CONVERGENCE COLLAPSE`. Left here, marked resolved, so the history
//!   stays traceable rather than the bullet just vanishing.
//! * Convergence (`factory.rs`'s checkpoints) samples TOTAL delivered
//!   counts across every item in the factory, while a solid target's own
//!   `deficit_pct` here watches PRODUCED for that target specifically
//!   (see `build_and_measure`) — two different signals (second-opinion
//!   review round 5, "absorb #3"). A genuine sink-side throttle
//!   (delivery destabilizing while the target's own production stays
//!   rock-steady at plan) would present as non-converged + at-or-above
//!   plan, landing in `UNSETTLED` — which this gate never fails, by
//!   design. Named, not fixed: closing it means watching the target's
//!   own delivered/produced series for convergence rather than the
//!   whole-factory delivered total, a bigger change than this
//!   diagnostic's scope.

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
/// ever compared. Justified on its own terms, NOT by analogy to
/// `crates/sim-harness/src/baseline.rs`'s 2% tolerance (round 3's doc
/// claimed the two "match"; they don't — theirs is a RELATIVE tolerance
/// on the blessed rate itself (`(got - want).abs() / want`), this is a
/// fixed PERCENTAGE-POINT tolerance on plan-relative deficit; the shared
/// numeral 2 is coincidence, not agreement — corrected round 4, finding
/// #3). 2.0pp is set to comfortably absorb the tick-window jitter
/// documented in docs/meter-divergence.md's window-quantization section
/// (sub-1pp effects from window-boundary quantization alone) while still
/// catching a real regression.
///
/// Trade-off, stated plainly (round 4, finding #4): the clamped
/// comparison (below) means a fixture blessed comfortably ABOVE plan
/// that later drops to JUST below plan — within this tolerance of zero —
/// will not alarm, because `min(fresh, 0)` is small in magnitude and the
/// "drop" from the baseline's clamped 0 doesn't clear the bar. Accepted:
/// the alternative (a zero-tolerance clamped comparison) would fail on
/// jitter alone for any fixture blessed near the boundary, which is
/// worse in practice than a several-run delay before a marginal
/// transition trips the gate.
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
    /// Machine production, the calibration metric for this meter-only
    /// stability tripwire. This intentionally differs from the CLI's gate
    /// metric for solid targets, which mirrors the sim verdict's delivered
    /// rate; fluid rows remain excluded from bless/check because they are not
    /// yet calibrated against a real Factorio baseline.
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
    // This tripwire watches the target machine's production series so a
    // downstream sink change does not masquerade as a meter regression. The
    // CLI gate uses delivered for solids to mirror the simulator verdict;
    // that is a deliberate decision boundary, not an accidental mismatch.
    let (measured, metric) = (
        report.produced_per_s.get(&target.item).copied().unwrap_or(0.0),
        "produced",
    );
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
/// doc's bless/check protocol section). Skips fluid targets (round 4,
/// finding #6) for consistency with their bless/check exclusion — a
/// fluid target's plan-relative number is never gated on anyway (see the
/// module doc's "Fluid targets are excluded" section), so this sanity
/// check has no business asserting on it either.
fn assert_has_plan(results: &[Measurement]) {
    let no_plan: Vec<&str> = results
        .iter()
        .filter(|m| !m.is_fluid && m.deficit_pct.is_nan())
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
    /// Both converged, entities matched, no below-plan reading at all
    /// (`deficit_pct >= 0`). Split out from `standing`/`regressed`
    /// (second-opinion review round 3, finding #3): the previous single
    /// `compared` counter incremented BEFORE the regression check ran, so
    /// a row that FAILED still printed as "compared clean-or-standing" —
    /// the exact count/severity conflation `docs/validator-reporting.md`
    /// polices against. Named `compared_at_or_above`, not `..._clean`
    /// (round 4, finding #2): "clean" is a clearance word, and an
    /// at-or-above-plan reading is evidence of nothing either way — see
    /// the module doc's hard-rule section.
    compared_at_or_above: usize,
    /// Both converged, entities matched, below plan but NOT materially
    /// worse than baseline — see `standing_below_plan` for the messages.
    /// A subset of "compared"; counted via `standing_below_plan.len()`.
    standing_below_plan: Vec<String>,
    /// Both converged, entities matched, below plan AND materially worse
    /// than baseline. Counted separately from `compared_at_or_above`/
    /// `standing_below_plan` so the accounting can never conflate a
    /// failing row with a clean one.
    regressed: usize,
    /// Baseline converged, fresh does NOT converge, AND `clamped_drop`
    /// (the SAME tolerance-gated comparison `regressed` uses) reads the
    /// fresh side as materially worse. Counted as "judged" (a verdict WAS
    /// rendered: FAIL) even though it is not `compared_at_or_above`,
    /// `standing`, or `regressed`.
    ///
    /// Getting this gate right took three rounds (see the module doc's
    /// hard-rule section for the blow-by-blow): round 2/3 failed on ANY
    /// non-convergence, including a RISING reading (the meter's
    /// `converged` flag is also false for an over-producing factory —
    /// see `factory.rs`'s `a_filling_buffer_is_not_converged`), which
    /// fails on good news (round 4, finding #1). Round 4's fix (a raw
    /// sign check) stopped that, but not a below-plan-but-improving
    /// reading or a sub-tolerance flap, because it had no tolerance at
    /// all — inconsistent with `regressed`'s (round 5, finding #1).
    /// Sharing `clamped_drop` makes both consistent by construction. See
    /// `unsettled` for the non-converged case this excludes.
    collapsed: usize,
    excluded_fluid: usize,
    skipped_new: usize,
    skipped_stale: usize,
    /// The fixture's solved TARGET changed vs the baseline (e.g. a
    /// fixture config edit under the same label) — a different KIND of
    /// drift than `skipped_stale`'s geometry mismatch, so its own
    /// accounting reason (round 5, "absorb #1": `BaselineRow.target` was
    /// written at bless time but never read in `check` before this).
    skipped_target_changed: usize,
    /// Labels whose BASELINE row never converged and whose FRESH run
    /// still doesn't either — a standing gap, not a fresh finding.
    uncovered: Vec<String>,
    /// Labels whose BASELINE row never converged but whose FRESH run now
    /// DOES — a fix may have landed, but there is still no trustworthy
    /// baseline number to compare against, so this is not `judged` either
    /// (second-opinion review round 3, finding #1). Distinct from
    /// `uncovered` so a repair is visible instead of looking identical to
    /// "still broken".
    recovered: Vec<String>,
    /// Labels where baseline converged but fresh does NOT, and
    /// `clamped_drop` does NOT read it as materially worse than the
    /// baseline — a rising buffer, an improvement still settling (even
    /// while remaining below plan), or plain jitter; not a below-plan
    /// finding. Not `collapsed` and not `judged`: there is no trustworthy
    /// number here either, just not a failing one. Needs a stable re-run
    /// (or a re-bless once it settles) before it can be judged either
    /// way.
    unsettled: Vec<String>,
}

impl CheckOutcome {
    /// At least one row got a real verdict (compared at-or-above,
    /// standing, regressed, OR collapsed to a failure). Distinguishes
    /// "verified clean" from "compared nothing" (second-opinion review
    /// round 2, finding #2) — "verified clean" here describes the CHECK
    /// RUN's own state (did it actually judge anything), not any single
    /// fixture's rate, so it is not the clearance wording finding #2
    /// targets.
    fn judged(&self) -> usize {
        self.compared_at_or_above + self.standing_below_plan.len() + self.regressed + self.collapsed
    }

    fn not_judged(&self) -> usize {
        self.skipped_new
            + self.skipped_stale
            + self.skipped_target_changed
            + self.uncovered.len()
            + self.recovered.len()
            + self.excluded_fluid
            + self.unsettled.len()
    }
}

/// The below-plan-portion "drop" between a baseline and fresh reading:
/// both clamped to a ceiling of 0 before differencing, so the result is
/// only positive when the FRESH reading is itself below plan (module
/// doc's hard-rule section). Shared by the collapse and compared
/// branches in `evaluate_check` (round 5, finding #1) — the collapse
/// branch used to gate on raw deficit SIGN with no tolerance at all,
/// which was inconsistent with this comparison and wrong on two counts
/// besides: it failed on a below-plan-but-IMPROVING reading (a baseline
/// blessed at -25% settling to a fresh -5% still read negative, so it
/// still failed, even though -5% is a big improvement) and on any
/// sub-tolerance non-converged flap (e.g. -0.5% against an at-plan
/// baseline — the same -0.5% would land in `standing`, not fail, if it
/// happened to converge). Sharing this function makes "the exact same
/// comparison" true by construction, not by convention.
fn clamped_drop(baseline_deficit_pct: f64, fresh_deficit_pct: f64) -> f64 {
    baseline_deficit_pct.min(0.0) - fresh_deficit_pct.min(0.0)
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
        if b.target != m.target {
            // `BaselineRow.target` was written at bless time but never
            // read here — round 5, finding "absorb #1": the same fixture
            // LABEL now solving a different item is exactly the "two
            // representations drift apart" class this field exists to
            // catch, and it went unchecked. Its own accounting reason,
            // not folded into `skipped_stale`: a target change is a
            // different KIND of drift than a geometry change (a fixture
            // config edit under the same label, not the engine/cache
            // producing a different layout for the same target).
            out.skipped_target_changed += 1;
            if verbose {
                eprintln!(
                    "{}: SKIPPED — target changed ({} -> {}); this fixture now solves a \
                     different item than its baseline recorded, so the two numbers are not \
                     comparable. Re-bless deliberately.",
                    m.label, b.target, m.target
                );
            }
            continue;
        }
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
            // (second-opinion review round 2, finding #1). Round 3 splits
            // this direction again: a fresh run that now converges is a
            // possible repair, not just more of the same standing gap
            // (finding #1) — see the module doc's dedicated section on
            // why this fixture can never gain collapse coverage while it
            // stays blessed non-converged, and why RECOVERED exists.
            if m.converged {
                out.recovered.push(m.label.to_string());
                if verbose {
                    eprintln!(
                        "{}: RECOVERED — baseline never converged, but this run does; \
                         re-bless to arm collapse coverage. Not yet judged: there is still no \
                         trustworthy baseline number to compare against.",
                        m.label
                    );
                }
            } else {
                out.uncovered.push(m.label.to_string());
                if verbose {
                    eprintln!(
                        "{}: UNCOVERED (baseline non-converged) — this fixture has never had \
                         a trustworthy baseline reading to compare against; a standing gap, \
                         not a fresh finding.",
                        m.label
                    );
                }
            }
            continue;
        }
        if !m.converged {
            // Baseline WAS a trustworthy anchor; fresh no longer
            // converges. Judged by the EXACT SAME tolerance-gated
            // clamped comparison as the converged case below — only the
            // LABEL differs (COLLAPSE, so the reader knows convergence
            // was also lost), per `clamped_drop`'s doc comment (round 5,
            // finding #1 — the kernel's final, unifying shape). A true
            // stall reads a clamped drop of roughly baseline-deficit
            // minus -100, far over tolerance, so it is still caught.
            let drop = clamped_drop(b.deficit_pct, m.deficit_pct);
            if drop > REGRESSION_TOLERANCE_PP {
                out.collapsed += 1;
                out.regressions.push(format!(
                    "{}: CONVERGENCE COLLAPSE — baseline {:.1}%, now {:.1}% ({:.1}pp worse, \
                     below-plan portion only) AND fresh no longer converges. A stalled/\
                     collapsed factory is exactly the regression class this guard exists to \
                     catch.",
                    m.label, b.deficit_pct, m.deficit_pct, drop
                ));
            } else {
                out.unsettled.push(m.label.to_string());
                if verbose {
                    eprintln!(
                        "{}: UNSETTLED — not converged, and not materially worse than the \
                         blessed baseline ({:.1}% -> {:.1}%, within {:.1}pp tolerance); could \
                         be a rising buffer, a still-settling improvement (even while \
                         remaining below plan), or jitter — not evidence of a below-plan \
                         regression. Needs a stable re-run (or a re-bless once stable) \
                         before this can be judged.",
                        m.label, b.deficit_pct, m.deficit_pct, REGRESSION_TOLERANCE_PP
                    );
                }
            }
            continue;
        }
        // The ONLY percentage-based alarm condition, per the module
        // docs' hard rule: compare only the BELOW-plan portion of each
        // reading (clamped to a ceiling of 0, via `clamped_drop`) so an
        // above-plan baseline can never manufacture a "regression" out
        // of a fresh reading that moved toward plan or into a
        // trustworthy below-plan range. `drop` is provably <= 0 whenever
        // the fresh reading is at-or-above plan, so this can only fire
        // when the CURRENT reading is itself below plan (second-opinion
        // review round 1, finding #1). The bucket a row lands in is
        // decided HERE, after the check, never before it (second-opinion
        // review round 3, finding #3 — the old `out.compared += 1` ran
        // unconditionally before this comparison, so a row that
        // regressed still counted as "compared clean-or-standing").
        let drop = clamped_drop(b.deficit_pct, m.deficit_pct);
        if drop > REGRESSION_TOLERANCE_PP {
            out.regressed += 1;
            out.regressions.push(format!(
                "{}: BELOW-PLAN REGRESSION — baseline {:.1}%, now {:.1}% ({:.1}pp worse, \
                 below-plan portion only)",
                m.label, b.deficit_pct, m.deficit_pct, drop
            ));
        } else if m.deficit_pct < 0.0 {
            // Standing, not-materially-worse deficit — reported so
            // "clean" never reads as "nothing here is wrong"
            // (second-opinion review round 2, finding #6). Worded as the
            // actual condition (within tolerance of the blessed value),
            // not "accepted at bless time as this fixture's zero point"
            // (round 4's wording — factually wrong when the baseline
            // itself was at-or-above plan and only the FRESH reading
            // dipped sub-tolerance below it; round 5, finding #2).
            out.standing_below_plan.push(format!(
                "{}: STANDING BELOW-PLAN — {:.1}%, within {:.1}pp tolerance of the blessed \
                 value ({:.1}%); not a new regression.",
                m.label, m.deficit_pct, REGRESSION_TOLERANCE_PP, b.deficit_pct
            ));
        } else {
            out.compared_at_or_above += 1;
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
    // Captured BEFORE building anything below, deliberately (second-
    // opinion review round 3, finding #2): `build_and_measure` can append
    // newly-solved zones to the cache file, so hashing after the build
    // loop had already run described post-run bytes, not the committed
    // pin that was actually consulted as this run's starting state. Both
    // `bless` and `check` reuse this ONE captured value rather than
    // re-hashing later.
    let pin_hash = hash_zone_cache();

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
                zone_cache_hash: pin_hash.clone(),
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
            if pin_hash != baseline.zone_cache_hash {
                eprintln!(
                    "NOTE: zone-cache hash differs from the blessed baseline's ({:?} vs \
                     {:?}) — entity-count mismatches below may be caused by this rather \
                     than a genuine engine change.",
                    baseline.zone_cache_hash, pin_hash
                );
            }

            let outcome = evaluate_check(&results, &baseline, true);
            eprintln!(
                "\naccounting: {}/{} judged ({} at-or-above, {} standing, {} regressed, {} \
                 collapsed), {}/{} not judged ({} new, {} stale, {} target-changed, {} \
                 uncovered [baseline never converged], {} recovered [re-bless to arm], {} \
                 unsettled [not converged, not worse than baseline], {} excluded [fluid, \
                 uncalibrated])",
                outcome.judged(),
                results.len(),
                outcome.compared_at_or_above,
                outcome.standing_below_plan.len(),
                outcome.regressed,
                outcome.collapsed,
                outcome.not_judged(),
                results.len(),
                outcome.skipped_new,
                outcome.skipped_stale,
                outcome.skipped_target_changed,
                outcome.uncovered.len(),
                outcome.recovered.len(),
                outcome.unsettled.len(),
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
        m.metric = "produced";
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

    /// Round 5's final, unifying shape: the collapse branch now uses the
    /// EXACT SAME tolerance-gated clamped comparison (`clamped_drop`) as
    /// the compared branch below — only the label differs. A true stall
    /// (measured ≈ 0, i.e. deficit_pct ≈ -100%) reads a clamped drop far
    /// beyond any reasonable tolerance regardless of the baseline's own
    /// value, so it is still caught (round 1's original bug: uniform
    /// non-convergence skipping let a throughput collapse to ~0 pass
    /// green).
    #[test]
    fn convergence_collapse_fails_on_a_true_stall() {
        let results = vec![measurement("x", -100.0, 100, false)];
        let base = baseline(vec![baseline_row("x", -2.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert_eq!(out.collapsed, 1, "must be counted as collapsed");
        assert_eq!(out.judged(), 1, "a collapse IS a rendered verdict");
        assert!(
            out.regressions.iter().any(|r| r.contains("COLLAPSE")),
            "expected a COLLAPSE regression, got {:?}",
            out.regressions
        );
        assert!(out.unsettled.is_empty());
    }

    /// Round 4, finding #1, first pass: baseline converged, fresh does
    /// NOT converge, but the fresh reading is AT-OR-ABOVE plan — a rising
    /// buffer or a still-settling improvement, not below-plan evidence.
    /// Must NOT fail: the round-2/3 collapse branch fired on
    /// non-convergence alone, which also covers this shape (`factory.rs`'s
    /// convergence detector fails on any unstable trajectory, not only a
    /// falling one), so it would have failed on good news — violating the
    /// below-plan-only hard rule as badly as round 1's bug violated it in
    /// the other direction.
    #[test]
    fn convergence_collapse_does_not_fire_on_above_plan_improvement() {
        let results = vec![measurement("x", 5.0, 100, false)];
        let base = baseline(vec![baseline_row("x", -2.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert!(
            out.regressions.is_empty(),
            "a non-converged AT-OR-ABOVE-plan reading must never fail: {:?}",
            out.regressions
        );
        assert_eq!(out.collapsed, 0, "must not be counted as a collapse");
        assert_eq!(out.unsettled, vec!["x".to_string()]);
        assert_eq!(out.judged(), 0, "unsettled is not yet judged either way");
    }

    /// Round 5, finding #1, second pass: round 4's fix (raw sign check,
    /// `deficit_pct < 0.0`, no tolerance) still false-failed two shapes
    /// this test and the next one pin. Here: a below-plan baseline
    /// IMPROVING while non-converged (blessed -25%, fresh -5%) — -5% is
    /// still negative, so round 4's gate still failed it, even though
    /// it's a big improvement over the baseline. Must NOT fail.
    #[test]
    fn convergence_collapse_does_not_fire_on_below_plan_improvement() {
        let results = vec![measurement("x", -5.0, 100, false)];
        let base = baseline(vec![baseline_row("x", -25.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert!(
            out.regressions.is_empty(),
            "an improving-while-still-below-plan reading must never fail: {:?}",
            out.regressions
        );
        assert_eq!(out.collapsed, 0);
        assert_eq!(out.unsettled, vec!["x".to_string()]);
    }

    /// Round 5, finding #1, second pass: a sub-tolerance non-converged
    /// flap (baseline at-plan, fresh -0.5%) must NOT fail — round 4's raw
    /// sign gate failed on ANY negative fresh deficit, even one well
    /// within `REGRESSION_TOLERANCE_PP` of the baseline, which was
    /// inconsistent with the compared branch (the same -0.5% would land
    /// in `standing`, not `regressed`, if it happened to converge).
    #[test]
    fn convergence_collapse_does_not_fire_on_sub_tolerance_flap() {
        let results = vec![measurement("x", -0.5, 100, false)];
        let base = baseline(vec![baseline_row("x", 0.0, 100, true)]);
        let out = evaluate_check(&results, &base, false);
        assert!(
            out.regressions.is_empty(),
            "a sub-tolerance non-converged flap must never fail: {:?}",
            out.regressions
        );
        assert_eq!(out.collapsed, 0);
        assert_eq!(out.unsettled, vec!["x".to_string()]);
    }

    /// Finding #1 (other direction): the baseline itself never converged
    /// and the fresh run still doesn't either — this stays a skip (an
    /// "uncovered" standing gap), never a failure.
    #[test]
    fn baseline_never_converged_is_uncovered_not_failed() {
        let results = vec![measurement("x", -1.0, 100, false)];
        let base = baseline(vec![baseline_row("x", -1.0, 100, false)]);
        let out = evaluate_check(&results, &base, false);
        assert!(out.regressions.is_empty(), "must not fail: {:?}", out.regressions);
        assert_eq!(out.uncovered, vec!["x".to_string()]);
        assert!(out.recovered.is_empty());
        assert_eq!(out.judged(), 0, "an uncovered row is not a rendered verdict");
    }

    /// Round 3, finding #1: `ec30-am2-ore`'s real shape — a baseline that
    /// never converged, now paired with a fresh run that DOES. This must
    /// print/count as RECOVERED, not silently fold into `uncovered`
    /// (which would make a real fix invisible) and must not fail (there
    /// is still no trustworthy baseline number to compare the fresh
    /// reading against — an instrument cannot alarm relative to a
    /// baseline that was already producing nothing).
    #[test]
    fn baseline_never_converged_recovers_when_fresh_converges() {
        let results = vec![measurement("ec30-am2-ore", 0.0, 100, true)];
        let base = baseline(vec![baseline_row("ec30-am2-ore", -100.0, 100, false)]);
        let out = evaluate_check(&results, &base, false);
        assert!(out.regressions.is_empty(), "must not fail: {:?}", out.regressions);
        assert_eq!(out.recovered, vec!["ec30-am2-ore".to_string()]);
        assert!(out.uncovered.is_empty(), "must not ALSO count as uncovered");
        assert_eq!(
            out.judged(),
            0,
            "recovered is not yet judged — still no trustworthy baseline number"
        );
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
    /// above-plan baseline (the shape sulfuric5-chem/lightoil5-chem-
    /// cracking were then-committed at, +239%/+200%, before round 2
    /// excluded fluid targets from the baseline entirely) moving to a
    /// near-plan fresh reading must NOT read as a regression — the exact
    /// false-alarm the unclamped `b.deficit_pct - m.deficit_pct` formula
    /// produced.
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
        assert_eq!(out.regressed, 0);
        // -1.0% is itself below plan, so it lands in `standing`, not
        // `compared_at_or_above` — both are non-failing buckets, but
        // conflating them is exactly what finding #3 (round 3) polices
        // against.
        assert_eq!(out.standing_below_plan.len(), 1);
        assert_eq!(out.compared_at_or_above, 0);
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
        assert_eq!(
            out.regressed, 1,
            "must be counted in the `regressed` bucket, not `compared_at_or_above`"
        );
        assert_eq!(out.compared_at_or_above, 0);
        assert!(out.standing_below_plan.is_empty());
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

    /// Round 5, "absorb #1": `BaselineRow.target` was written at bless
    /// time but never read in `check` — the same fixture LABEL now
    /// solving a different item must be its own skip reason, distinct
    /// from `skipped_stale`'s geometry-mismatch, and must never fail.
    #[test]
    fn target_changed_is_skipped_not_failed() {
        let mut m = measurement("x", -50.0, 100, true);
        m.target = "gizmo".to_string();
        let results = vec![m];
        let base = baseline(vec![baseline_row("x", 0.0, 100, true)]); // target: "widget"
        let out = evaluate_check(&results, &base, false);
        assert_eq!(out.skipped_target_changed, 1);
        assert_eq!(out.skipped_stale, 0, "must not ALSO count as a stale-geometry skip");
        assert!(out.regressions.is_empty());
        assert_eq!(out.judged(), 0);
    }
}
