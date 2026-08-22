//! W1c (RFC-070 campaign, tracking issue #689): the **parity corpus** and
//! its committed baseline — Phase 0c.
//!
//! K70-2 is not re-runnable until the corpus it measures over is NAMED
//! (RFC-070 §Kill criteria). This file is that name, expressed as code:
//! an explicit fixture × machine-tier × OPTION-SET grid, each cell
//! recording who won `select_best_decomposition` and which precedence
//! stage decided it. `parity_corpus_baseline.json` is the committed
//! result; Phase 2a's shadow loop diffs against it under the
//! divergence-equivalence rule in the RFC's Verification plan.
//!
//! # Why the option-set axis exists
//!
//! W1b found that `run_e2e` does not run production's candidate set
//! (RFC-070 decision log, 2026-08-21): it spells
//! `cell_composition: Default::default()`, the ENUM default `Off`, next
//! to a `..Default::default()` that would have given the STRUCT default
//! `Candidate`. So "same fixture" does not pin the candidate field, and
//! a corpus indexed by fixture alone would record winners for a
//! configuration nothing ships.
//!
//! This file found the same fossil a second time, in the same function:
//! `run_e2e_inner` also pins `inserter_capacity: 0`
//! (`tests/e2e.rs:354`). That line was correct when it was written
//! (`40fd48dc`, RFC-049 Phase 1, 2026-07-22 — the struct default WAS 0),
//! and went stale two days later when #383 flipped the default to
//! `common::DEFAULT_INSERTER_CAPACITY` = 2. Identical shape, identical
//! cause. The e2e harness therefore differed from production defaults on
//! TWO fields, not one, which is why `e2e-harness` below is its own
//! option set rather than a synonym for `cells-off`.
//!
//! Past tense as of 2026-08-21: #689 track W2c killed both fossils and
//! the harness now runs the `default` column. `e2e-harness` stays as the
//! historical record the W2c re-blesses were adjudicated against — see
//! the `OPTION_SETS` doc below.
//!
//! # What a cell records, and what it does not
//!
//! Winner + deciding stage + the seven candidate outcomes. Nothing else:
//! the verdict NUMBERS (`IssueCounts`, `ErrorKinds`) are structurally
//! full of holes by construction — see the RFC's "Phase-0b oracle gaps"
//! entry — so a baseline pinning them would pin gaps as facts. The
//! diagnostic that PRINTS those numbers is
//! `check_firing_census.rs::selection_scoreboard_census`; this one
//! commits only what a shadow loop can be held to.
//!
//! # Corpus scope
//!
//! Fixture list is the #691 corpus verbatim (the G2 census's six
//! tier-ladder fixtures plus the six explicit e2e "from-ore" ones), so
//! all three censuses — check-firing, junction-seed, and this — describe
//! the same solves. Widened here by machine tier (`assembling-machine-1
//! /-2/-3` for assembler targets) and by option set.
//!
//! Deliberately EXCLUDED, and why: the meter tripwire's two fluid-target
//! fixtures (`sulfuric5-chem`, `lightoil5-chem-cracking`). The machine
//! axis is vacuous for them (no chemical-plant tier ladder) and
//! `lightoil5` needs `solve_with_exclusions`; neither buys a candidate
//! configuration the twelve below do not already reach. Chemical-plant
//! fixtures that ARE here carry a one-entry machine list for the same
//! reason.
//!
//! A cell whose machine cannot run the recipe (`assembling-machine-1`
//! has `ingredient_slots: 2` and no fluid boxes) is recorded as
//! `no-solve`, not dropped. The corpus states its own holes.
//!
//! # Running it
//!
//! ```text
//! SPAGHETTIO_ZONE_CACHE_PATH=$(pwd)/crates/core/data/sat-zones-ci.bin \
//!   cargo test --manifest-path crates/core/Cargo.toml \
//!   --test parity_corpus -- --ignored --nocapture
//! git checkout -- crates/core/data/sat-zones-ci.bin   # ALWAYS, every run
//! ```
//!
//! # Reproducibility is of the WHOLE RUN, in order — not of one cell
//!
//! A trap worth knowing before Phase 2a re-runs a divergent cell on its
//! own (#694 review round 6). `zone_cache::lookup_table()` is a
//! process-wide `OnceLock`, seeded once from disk and then **mutated in
//! memory** as solves append newly-found zones. All 160 cells run in one
//! process, so cell N is solved against a map that cells 1..N-1 have
//! already grown. The committed hash describes the run's STARTING disk
//! state, which is what makes the full sweep reproducible — measured
//! seven times, byte-identical every time — but it says nothing about
//! what any individual cell saw.
//!
//! Consequence: **re-running one fixture in isolation is not guaranteed
//! to reproduce its committed cell.** To adjudicate a divergence, re-run
//! the whole corpus and read that cell out of it, or accept an isolated
//! re-run as indicative rather than as the baseline's own value.
//!
//! `SPAGHETTIO_PARITY_CORPUS=bless` rewrites the committed baseline;
//! `=check` fails on any cell that differs from it. `bless` REFUSES if
//! this run's zone cache is not the one the committed baseline was
//! blessed against (or if there is no resolvable cache at all) — the
//! corpus is cache-relative and blessing also rewrites the recorded
//! hash, so a mis-pinned bless would leave 160 unreproducible rows with
//! nothing to notice it by. `=bless-repin` is the escape hatch for
//! deliberately re-taking the baseline on a new cache.
//!
//! **Not CI-gated, deliberately, and the reason is scheduling rather
//! than cost** (#694 review, finding 2). Gating this against production
//! today would only assert "production has not changed", which every
//! engine PR legitimately falsifies — a re-bless treadmill with no
//! reader. The consumer that makes a gate meaningful is Phase 2a's
//! shadow loop, and the RFC already commits to gating THAT
//! (Verification plan item 2: "winner mismatch fails the check"). Until
//! then the guard on the instrument is the three non-ignored contract
//! tests in `check_firing_census.rs`; the guard on the DATA is that the
//! next phase re-takes it. Same posture as the stress goldens for the
//! same underlying reason: the layouts depend on which zone solutions
//! the pinned cache replays.

use std::collections::BTreeMap;

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::bus::selection_policy::{
    decide, ErrorKindCounts, IssueCounts, IssueProfile, SelectionPolicy,
};
use spaghettio_core::solver;
use spaghettio_core::trace::{self, SelectionCandidateOutcome, SelectionStage, TraceEvent};

/// The candidate slot order, spelled longhand rather than imported from
/// `decomposition_search::CANDIDATE_ORDER` (which is private anyway).
/// Same argument as `selection_scoreboard_contract`'s copy: a list that
/// reads the constant the code reads cannot detect a wrong reorder.
const EXPECTED_ORDER: [&str; 7] = [
    "native",
    "k1-shape-fix",
    "size-split-2",
    "merge-tap",
    "cell-composed",
    "direct-insertion",
    "horizontal-stack",
];

struct Fixture {
    label: &'static str,
    item: &'static str,
    rate: f64,
    belt: Option<&'static str>,
    inputs: &'static [&'static str],
    /// Machine tiers swept for this fixture. Assembler targets get all
    /// three; a chemical-plant target gets the one machine that exists.
    machines: &'static [&'static str],
}

const ASSEMBLERS: &[&str] =
    &["assembling-machine-1", "assembling-machine-2", "assembling-machine-3"];
const CHEM: &[&str] = &["chemical-plant"];

/// The #691 corpus, verbatim (labels included, so the three censuses can
/// be read against each other row by row). The machine named in each
/// #691 label is now one point on the swept axis, not the whole cell —
/// e.g. `tier1_gear_am1` contributes three cells per option set.
const FIXTURES: &[Fixture] = &[
    // --- tier-ladder slice (== check_firing_census.rs's six) ---
    Fixture {
        label: "tier1_gear_am1",
        item: "iron-gear-wheel",
        rate: 10.0,
        belt: None,
        inputs: &["iron-plate"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "tier2_ec_am1_10_ore",
        item: "electronic-circuit",
        rate: 10.0,
        belt: None,
        inputs: &["iron-ore", "copper-ore"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "tier2_ec_am2_30_ore",
        item: "electronic-circuit",
        rate: 30.0,
        belt: None,
        inputs: &["iron-ore", "copper-ore"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "tier3_plastic_cp_5",
        item: "plastic-bar",
        rate: 5.0,
        belt: None,
        inputs: &["coal", "water", "crude-oil"],
        machines: CHEM,
    },
    Fixture {
        label: "tier4_ac_am2_5_unconstrained",
        item: "advanced-circuit",
        rate: 5.0,
        belt: None,
        inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "tier5_pu_am3_2_unconstrained",
        item: "processing-unit",
        rate: 2.0,
        belt: None,
        inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        machines: ASSEMBLERS,
    },
    // --- e2e "from-ore" fixtures (distinct in belt tier and/or rate) ---
    Fixture {
        label: "e2e_tier1_iron_gear_wheel_from_ore",
        item: "iron-gear-wheel",
        rate: 10.0,
        belt: None,
        inputs: &["iron-ore"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "e2e_tier2_electronic_circuit_from_ore",
        item: "electronic-circuit",
        rate: 10.0,
        belt: Some("transport-belt"),
        inputs: &["iron-ore", "copper-ore"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "e2e_tier2_electronic_circuit_20s_from_ore",
        item: "electronic-circuit",
        rate: 20.0,
        belt: None,
        inputs: &["iron-ore", "copper-ore"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "e2e_tier3_plastic_bar_from_crude",
        item: "plastic-bar",
        rate: 10.0,
        belt: None,
        inputs: &["crude-oil", "coal"],
        machines: CHEM,
    },
    Fixture {
        label: "e2e_tier4_advanced_circuit_from_ore_am2",
        item: "advanced-circuit",
        rate: 5.0,
        belt: Some("transport-belt"),
        inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        machines: ASSEMBLERS,
    },
    Fixture {
        label: "e2e_tier5_processing_unit_from_ore_am3",
        item: "processing-unit",
        rate: 2.0,
        belt: Some("fast-transport-belt"),
        inputs: &["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
        machines: ASSEMBLERS,
    },
];

/// The option-set axis. `default` is what production ships; the rest are
/// the configurations a shadow loop has to reproduce too.
///
/// `cells-off` and `e2e-harness` are NOT the same cell: see the module
/// doc's second-fossil note. Keeping both is what lets a divergence be
/// attributed to the cell-composed arm rather than to the inserter
/// ladder.
///
/// `e2e-harness` encodes the harness as a DELTA from `LayoutOptions::
/// default()`. **It no longer describes a LIVE configuration.** Both
/// fossils it encodes were killed on 2026-08-21 (#689 track W2c): the
/// harness now builds its options through `LayoutOptions::from_groups`
/// and runs the `default` column. What this label names is the
/// HISTORICAL configuration the committed baseline was taken under —
/// which is exactly what made it the prediction W2c's re-blesses were
/// adjudicated against. It is kept, not deleted: dropping it would
/// silently re-take 32 cells and destroy the only record of what the
/// fossilized suite decided. The baseline itself needed no re-bless —
/// this file builds its option sets as closures over
/// `LayoutOptions::default()` (below), never through `run_e2e`, so a
/// change to the harness cannot move a cell here (verified empirically:
/// `SPAGHETTIO_PARITY_CORPUS=check` passed 160/160 after W2c).
///
/// **SUPERSEDED — the hand-verification receipt, kept for provenance.**
/// Everything in this paragraph describes the pre-W2c harness and is
/// false of the current one; it is here because it is the evidence the
/// committed cells were taken under, not as a description of today.
/// *"…it is only as accurate as this list — a future change to
/// `run_e2e_inner`'s other pinned fields, or a default flipping to match
/// a harness value, would break the 'this is what the harness runs'
/// claim with no corpus cell moving (#694 review round 2). It cannot be
/// asserted automatically: `run_e2e_inner` is a private fn in a
/// different test binary. So it is hand-verified instead, and here is
/// the receipt to re-check against — as of `a54b9a7a`,
/// `tests/e2e.rs:342-358` differs from the struct defaults on exactly
/// two fields, `cell_composition` (`Default::default()` → the enum's
/// `Off`) and `inserter_capacity: 0`; every other field it spells
/// (`max_inserter_tier`, `quality`, `wire_mode`, `merge_tap`,
/// `stacking`, `splitter_tap_spacers`) already equals its default."*
/// That struct literal no longer exists, and the harness spells none of
/// those fields any more (#699 review round 3 — the correction used to
/// be appended AFTER the receipt, leaving two mutually exclusive
/// descriptions in one docblock).
///
/// The label names an OPTION SET, not a cell the harness runs (#694
/// review round 3). It is applied across the whole machine sweep, so
/// most `e2e-harness` cells combine the harness's option configuration
/// with a tier the harness never invokes for that fixture — which is the
/// POINT of the axis, not an overclaim, but only the (fixture, machine)
/// pairs `e2e.rs` actually calls are "what the suite runs". For the
/// tier-ladder six those are the machines their #691 labels name
/// (`tier1_gear_am1` → am1, `tier2_ec_am2_30_ore` → am2, …).
type OptionSet = (&'static str, fn(&mut LayoutOptions));

const OPTION_SETS: &[OptionSet] = &[
    ("default", |_| {}),
    ("cells-off", |o| o.cell_composition = CellComposition::Off),
    ("e2e-harness", |o| {
        o.cell_composition = CellComposition::Off;
        o.inserter_capacity = 0;
    }),
    ("di-off", |o| o.direct_insertion = DirectInsertion::Off),
    ("hs-off", |o| o.horizontal_candidate = false),
];

/// One grid cell's outcome. `status` is the coarse verdict, so a reader
/// (and `check`) never has to infer one from a `None`. The complete set
/// the code can emit, which is the set a baseline reader should expect
/// (#694 review, finding 3 — an earlier version of this line listed a
/// `refused` that no branch produces):
///
/// - `decided` — the search picked a winner and the build returned it.
/// - `decided-then-refused` — the search picked a winner but
///   `build_bus_layout` returned `Err` afterwards. **Unreached on the
///   current corpus** (zero cells in the committed baseline); kept
///   because the pick is not the last thing the build does, and
///   collapsing it into `decided` would silently mislabel a refusal.
/// - `no-winner` — every candidate failed: rows emitted, no terminal.
/// - `no-selection` — the build never reached the search at all.
/// - `no-solve` — the solver refused this (fixture, machine) pair.
///
/// `no-winner` and `no-selection` do not consult `built`, and do not need
/// to: both IMPLY it errored. A stream with rows but no terminal is
/// `select_best_decomposition`'s all-candidates-failed path, which
/// returns `Err` by construction; a stream with no rows at all means the
/// build refused before reaching the search. So "search ran, all
/// refused" and "build failed after the search ran" are the same fact
/// here, not two collapsed ones (#694 review round 2, adjudicated as
/// designed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Cell {
    fixture: String,
    machine: String,
    options: String,
    status: String,
    winner: Option<String>,
    stage: Option<String>,
    /// Outcome per candidate SLOT, comma-joined in `EXPECTED_ORDER`.
    /// Empty when no selection ran. Deliberately one string, not seven
    /// JSON entries: it keeps a cell to one line, so a baseline diff
    /// shows a changed candidate field as one changed line rather than
    /// a re-indent. NOT part of the equivalence rule — recorded so a
    /// divergence can be adjudicated without re-taking the baseline.
    outcomes: String,
}

impl Cell {
    fn key(&self) -> (&str, &str, &str) {
        (self.fixture.as_str(), self.machine.as_str(), self.options.as_str())
    }
    /// The equivalence rule's comparison surface: winner name and
    /// deciding stage (plus status, which is what makes "no winner" and
    /// "winner unrecorded" different facts). See RFC-070's Verification
    /// plan §"Divergence-equivalence rule".
    fn verdict(&self) -> (&str, Option<&str>, Option<&str>) {
        (self.status.as_str(), self.winner.as_deref(), self.stage.as_deref())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Baseline {
    zone_cache_hash: Option<String>,
    cells: Vec<Cell>,
}

fn baseline_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/parity_corpus_baseline.json")
}

/// Mirrors `zone_cache::resolve_cache_path`'s fallback chain, which is
/// private to the crate — same reimplementation every other diagnostic
/// here makes (`e2e_tripwire.rs`, `e2e.rs`'s histogram diagnostics).
fn resolve_zone_cache_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("SPAGHETTIO_ZONE_CACHE_PATH") {
        return std::path::PathBuf::from(p);
    }
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| std::path::PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| std::path::PathBuf::from(".cache"));
    base.join("spaghettio").join("sat-zones.bin")
}

/// Content hash of the pinned cache, captured BEFORE any build — a run
/// appends newly-solved zones, so hashing afterwards would describe
/// post-run bytes rather than the pin actually consulted (the mistake
/// #693 round 3 caught in the meter tripwire).
///
/// **SHA-256, not `DefaultHasher`** (#694 review round 2). This value is
/// COMMITTED and then compared against on every later run, so it has to
/// be an identity that outlives the toolchain: `DefaultHasher`'s
/// algorithm is explicitly documented as unspecified and unstable across
/// releases, which would make the next Rust bump refuse to bless the
/// very cache it had blessed against, and flag every `check` as a
/// provenance mismatch. `sha2` is already a dev-dependency and `e2e.rs`
/// already hashes content with it.
///
/// Covers the zone sources a NATIVE run actually consults, and only
/// those. `lookup_table()` (`zone_cache.rs:1404-1412`) seeds its map
/// with:
///
/// ```text
/// #[cfg(not(target_arch = "wasm32"))] load_existing_jsonl(&mut map);
/// #[cfg(target_arch = "wasm32")]      install_prebaked_into(&mut map, EMBEDDED_CACHE);
/// ```
///
/// so on native — which is how this corpus runs — the sources are
/// exactly what `load_existing_jsonl` reads: the **pin** and the
/// **legacy `.jsonl` beside it**. The compiled-in `EMBEDDED_CACHE`
/// (`include_bytes!("../data/sat-zones.bin")`) is **WASM-only** and is
/// deliberately NOT hashed: including it would make a wasm-only change
/// to that file hard-fail every native `check` and refuse every plain
/// `bless` while the native zone set was byte-identical, and would
/// refuse outright if a file this run never reads went missing.
///
/// The `.jsonl` is optional — absent contributes nothing to the map and
/// nothing here, so a file APPEARING moves the hash, which is the
/// direction that matters.
///
/// **Three review rounds went into that list, and it was WRONG in two of
/// them** — first missing the `.jsonl`, then over-corrected into hashing
/// a wasm-only file. The durable lesson is not the list: it is that *a
/// provenance hash is worth exactly what its source list is, a source
/// list goes stale, and "which sources" is a `#[cfg]` question that
/// cannot be answered by reading a function name.* Re-derive it from
/// `lookup_table()` rather than from this comment if it ever matters.
fn hash_zone_cache() -> Option<String> {
    use sha2::{Digest, Sha256};
    let pin_path = resolve_zone_cache_path();
    let pin = std::fs::read(&pin_path).ok()?;
    let legacy = std::fs::read(pin_path.with_extension("jsonl")).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&pin);
    h.update(&legacy);
    Some(format!("{:x}", h.finalize()))
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

fn outcome_name(o: SelectionCandidateOutcome) -> &'static str {
    match o {
        SelectionCandidateOutcome::Produced => "produced",
        SelectionCandidateOutcome::Refused => "refused",
        SelectionCandidateOutcome::Panicked => "PANICKED",
        SelectionCandidateOutcome::NotRun => "not-run",
    }
}

/// One build's outer selection: what the cell records, plus the
/// per-candidate rows themselves (which `policy_replay` feeds through
/// the v2 comparators — see its own docs for why they cannot come from
/// the committed baseline).
#[derive(Default)]
struct OuterSelection {
    outcomes: String,
    decided: Option<(String, SelectionStage)>,
    /// The seven `SelectionCandidateEvaluated` events of the outer
    /// block, in canonical slot order. Empty when no selection ran.
    rows: Vec<TraceEvent>,
    /// RFC-070 Phase 2a: the live shadow comparison the engine emitted
    /// for this selection. `None` when no selection ran at all — and
    /// that is the ONLY reason it may be absent, which is why the
    /// harnesses below assert its presence on every decided cell rather
    /// than skipping a missing one (a skipped comparison reads as clean,
    /// the #693 shape this campaign keeps re-closing).
    shadow: Option<ShadowOutcome>,
}

/// One `SelectionShadowCompared` event, flattened for reporting.
#[derive(Debug, Clone)]
struct ShadowOutcome {
    agree: bool,
    v1: (Option<String>, Option<SelectionStage>),
    v2: (Option<String>, Option<SelectionStage>),
    gate_disagreements: Vec<String>,
}

impl ShadowOutcome {
    /// Everything an adjudicator needs on one line: both verdicts and
    /// any gate disagreement, NAMED. A disagreement is a campaign-level
    /// finding, so the message has to carry enough to act on without
    /// re-running.
    fn describe(&self) -> String {
        let side = |(w, s): &(Option<String>, Option<SelectionStage>)| {
            format!(
                "{}/{}",
                w.as_deref().unwrap_or("<none>"),
                s.map(stage_name).unwrap_or("<none>")
            )
        };
        let gates = if self.gate_disagreements.is_empty() {
            String::new()
        } else {
            format!("  gates: [{}]", self.gate_disagreements.join("; "))
        };
        format!("v1 {} -> v2 {}{gates}", side(&self.v1), side(&self.v2))
    }
    /// Everything wrong with this comparison, or `None`.
    ///
    /// **Deliberately does NOT trust the engine's `agree` bit** (#703
    /// review round 1, the 1/3-pass finding and the most useful one):
    /// a harness that reads a boolean the thing under test computed is
    /// asserting "it says it agrees", not "the two programs agree". The
    /// event carries all four sides, so the comparison is redone here
    /// from the data — and the engine's own bit is then checked against
    /// that recomputation, which turns the self-referential read into a
    /// second independent surface.
    ///
    /// Four surfaces, each failing separately:
    ///
    /// 1. **The recomputed verdict.** `(v1_winner, v1_stage)` vs
    ///    `(v2_winner, v2_stage)`, compared here.
    /// 2. **The engine's `agree` bit** against (1) — a stuck-true bit,
    ///    or a comparison written against the wrong pair, shows up as a
    ///    mismatch rather than as silence.
    /// 3. **v1's side against the cell's OWN record**, which came from
    ///    the independent `SelectionDecided` event. This catches a
    ///    mis-indexed winner name (`CANDIDATE_ORDER[idx]` vs the name
    ///    the winner replay actually used) and a shadow event paired
    ///    with the wrong block.
    /// 4. **The producer gates.** A gate disagreement that does not
    ///    happen to move the winner on this solve is still a
    ///    mis-transcription, and letting it ride on the verdict alone
    ///    hides exactly the class the gate comparison exists to catch.
    ///
    /// What none of this can reach, stated so the coverage is not
    /// overread: a policy error faithfully reproduced by BOTH the
    /// registration and v1 — the shadow's gate half checks v2's clauses
    /// against v1's actual dispatch, so a mis-transcription is caught
    /// wherever the corpus exercises it, but a clause that is wrong and
    /// behaviourally identical on all 160 cells is a coverage limit, not
    /// something an assertion can close.
    fn faults(&self, cell: &Cell) -> Vec<String> {
        let mut out = Vec::new();
        let recomputed = self.v1.0 == self.v2.0 && self.v1.1 == self.v2.1;
        if !recomputed {
            out.push("v1 and v2 named different verdicts".to_string());
        }
        if self.agree != recomputed {
            out.push(format!(
                "the engine's own `agree` bit says {} where the four recorded fields say \
                 {recomputed} — the shadow's comparison disagrees with its own data",
                self.agree
            ));
        }
        let cell_side = (cell.winner.clone(), cell.stage.clone());
        let shadow_side = (self.v1.0.clone(), self.v1.1.map(|s| stage_name(s).to_string()));
        if cell_side != shadow_side {
            out.push(format!(
                "the shadow's v1 side {shadow_side:?} is not what `SelectionDecided` \
                 recorded for this cell ({cell_side:?}) — the two events are from \
                 different selections"
            ));
        }
        if !self.gate_disagreements.is_empty() {
            out.push(format!("gates: [{}]", self.gate_disagreements.join("; ")));
        }
        out
    }
}

/// The shadow-agreement tally over a sweep of cells, and the failure
/// message it produces.
///
/// **Why this gate is CI-shaped where the baseline is not.** Five review
/// rounds across #694/#698 refused "CI-gate the corpus", and correctly:
/// a baseline gate asserts "production has not changed", which every
/// engine PR legitimately falsifies, and the record is cache-relative
/// besides. The shadow is neither. It compares two dispatches on ONE
/// solve, so it says nothing about which layout was produced and
/// everything about whether the two programs answer alike — a fixture
/// whose winner legitimately moved still has a well-defined shadow
/// verdict, and a host with a different zone cache computes the same
/// one. That is what makes the smoke tier below runnable on every push
/// (RFC-070 Verification plan item 2, whose promise the earlier
/// refusals kept pointing at).
///
/// **Scoped precisely** (#703 review round 1): what is cache- and
/// layout-independent is the VERDICT COMPARISON. The fixture list is
/// still a list — a smoke cell that stops reaching the search has
/// nothing to shadow and fails the count check below, which is a
/// maintenance obligation like any hand-written fixture list. The claim
/// is "no re-bless treadmill for the DECISION", not "no fixture can
/// ever need replacing".
#[derive(Default)]
struct ShadowReport {
    /// Cells where BOTH programs named a winner and agreed.
    agreed_decided: usize,
    /// Cells where both programs named NO winner. Also an agreement,
    /// and counted apart from the one above so the headline figure
    /// cannot quietly mean something wider than "decided cells agree"
    /// (#703 review round 1: today's corpus has zero of these, so the
    /// two numbers coincide — the split is what keeps that checkable
    /// rather than assumed).
    agreed_no_winner: usize,
    disagreements: Vec<String>,
    /// Cells whose selection RAN but emitted no shadow event. A missing
    /// comparison must never read as agreement — that is the "compared
    /// nothing reads as clean" shape (#693) this campaign has now closed
    /// in four places.
    missing: Vec<String>,
}

impl ShadowReport {
    fn compared(&self) -> usize {
        self.agreed_decided + self.agreed_no_winner + self.disagreements.len()
    }

    fn absorb(&mut self, key: &str, run: &CellRun) {
        match (&run.shadow, run.cell.status.as_str()) {
            (Some(s), _) => {
                let faults = s.faults(&run.cell);
                if faults.is_empty() {
                    if run.cell.winner.is_some() {
                        self.agreed_decided += 1;
                    } else {
                        self.agreed_no_winner += 1;
                    }
                } else {
                    self.disagreements
                        .push(format!("  {key}: {}\n      {}", s.describe(), faults.join("\n      ")));
                }
            }
            // The two statuses where there is genuinely nothing to
            // shadow, because no selection ran at all: `no-solve` (the
            // solver refused this fixture×machine pair) and
            // `no-selection` (`build_bus_layout` refused BEFORE reaching
            // the search — the cell emits no scoreboard rows either).
            //
            // `no-selection` was hard-failing here in the first draft,
            // on a comment claiming "every other status means the search
            // ran" that is simply false of it (#703 review round 1). It
            // is latent today (zero such cells) but it would have turned
            // a legitimately-refusing cell into a campaign-finding-style
            // failure with no divergence — a criterion the baseline
            // comparison never had and this gate has no business adding.
            (None, "no-solve" | "no-selection") => {}
            (None, status) => self.missing.push(format!("  {key}: status {status}")),
        }
    }

    fn print(&self) {
        println!("\n=== RFC-070 Phase 2a shadow ===");
        println!(
            "cells compared: {}  (agreed: {} decided + {} no-winner)  |  disagreements: {}  \
             |  missing comparisons: {}",
            self.compared(),
            self.agreed_decided,
            self.agreed_no_winner,
            self.disagreements.len(),
            self.missing.len()
        );
        for d in self.disagreements.iter().chain(self.missing.iter()) {
            println!("{d}");
        }
    }

    /// Fails the sweep on a disagreement, a missing comparison, or an
    /// empty tally.
    fn assert_clean(&self) {
        assert!(
            self.compared() > 0,
            "the shadow compared NOTHING across the whole sweep — the comparison is not \
             running, which is not the same fact as it agreeing"
        );
        assert!(
            self.missing.is_empty(),
            "{} cell(s) ran a selection but emitted no `SelectionShadowCompared`. A \
             missing comparison is a HOLE, not an agreement:\n{}",
            self.missing.len(),
            self.missing.join("\n")
        );
        assert!(
            self.disagreements.is_empty(),
            "the v2 shadow disagreed with production on {} of {} cell(s). **THIS IS A \
             CAMPAIGN-LEVEL FINDING (RFC-070 K70-1 / K70-2), not a test bug.** Record the \
             cell and the mechanism and take it to the campaign lead. A transcription bug \
             may be fixed with receipts; a semantic one MUST be reported — do not tune \
             policy data until the numbers line up.\n{}",
            self.disagreements.len(),
            self.compared(),
            self.disagreements.join("\n")
        );
    }
}

/// Pull the outer selection's shadow event out of one build's stream.
/// Same "last one wins" rule as the rows: a nested selection's shadow is
/// emitted inside that candidate's captured events and is replayed
/// (winner only) BEFORE the outer board, so the last event is the outer
/// one.
fn outer_shadow(events: &[TraceEvent]) -> Option<ShadowOutcome> {
    events.iter().rev().find_map(|e| match e {
        TraceEvent::SelectionShadowCompared {
            v1_winner,
            v1_stage,
            v2_winner,
            v2_stage,
            agree,
            gate_disagreements,
        } => Some(ShadowOutcome {
            agree: *agree,
            v1: (v1_winner.clone(), *v1_stage),
            v2: (v2_winner.clone(), *v2_stage),
            gate_disagreements: gate_disagreements.clone(),
        }),
        _ => None,
    })
}

/// Pull the OUTER selection out of one build's event stream.
///
/// The census walks blocks to render every nested selection; this needs
/// only the outer one, and gets it by a rule that cannot mis-group:
/// `Scoreboard::emit` writes all seven rows contiguously immediately
/// before its terminal, and a nested selection completes strictly before
/// its parent's board is emitted, so the outer block is the LAST seven
/// rows in the stream. That the last seven really are the seven slots in
/// canonical order is ASSERTED, not assumed — if the emission contract
/// ever changes, the baseline run fails instead of committing rows
/// attributed to the wrong candidates.
fn outer_selection(events: &[TraceEvent]) -> OuterSelection {
    let rows: Vec<&TraceEvent> = events
        .iter()
        .filter(|e| matches!(e, TraceEvent::SelectionCandidateEvaluated { .. }))
        .collect();
    if rows.is_empty() {
        return OuterSelection::default();
    }
    let tail = &rows[rows.len().saturating_sub(EXPECTED_ORDER.len())..];
    let row_fields: Vec<(&str, SelectionCandidateOutcome)> = tail
        .iter()
        .map(|e| match e {
            TraceEvent::SelectionCandidateEvaluated { name, outcome, .. } => {
                (name.as_str(), *outcome)
            }
            _ => unreachable!("filtered to scoreboard rows above"),
        })
        .collect();
    let names: Vec<&str> = row_fields.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names, EXPECTED_ORDER,
        "the last {} scoreboard rows must be the outer selection's seven slots in \
         canonical order; got {names:?}. TWO readings: (1) the ENGINE gained, lost or \
         reordered a candidate — update `EXPECTED_ORDER` here and RE-TAKE the baseline, \
         since the candidate field moved; (2) the EMISSION contract changed — rows no \
         longer arrive contiguously before their terminal — in which case this extractor \
         is picking up a nested block's tail and every recorded winner is suspect. \
         `check_firing_census.rs::selection_scoreboard_contract` discriminates: it pins \
         row-before-terminal ordering on a single-selection fixture",
        EXPECTED_ORDER.len()
    );
    let outcomes =
        row_fields.iter().map(|(_, o)| outcome_name(*o)).collect::<Vec<_>>().join(",");
    let terminal = events
        .iter()
        .rposition(|e| matches!(e, TraceEvent::SelectionDecided { .. }));
    // The pairing this whole extractor rests on: the last terminal must
    // follow the last row, or those rows and that verdict belong to
    // different selections and the cell would record one block's winner
    // against another's candidates. Today this cannot happen — the
    // all-candidates-failed path emits a board with no terminal and
    // replays nobody's events, so a nested terminal can only reach the
    // stream inside a WINNER's replay, which precedes the outer board.
    // Asserted anyway, because "cannot happen" is what the mis-pairing
    // bug class always says first.
    if let Some(t) = terminal {
        let last_row = events
            .iter()
            .rposition(|e| matches!(e, TraceEvent::SelectionCandidateEvaluated { .. }))
            .expect("rows non-empty here");
        assert!(
            last_row < t,
            "the last `SelectionDecided` (index {t}) precedes the last scoreboard row \
             (index {last_row}) — the outer rows and the recorded verdict are from \
             different selections, so this cell would attribute one block's winner to \
             another block's candidates. Do not bless this baseline"
        );
    }
    let decided = terminal.and_then(|t| match &events[t] {
        TraceEvent::SelectionDecided { winner, stage } => Some((winner.clone(), *stage)),
        _ => None,
    });
    // Membership, on top of the timing guard above (#694 review round 2).
    // The timing check catches a terminal that arrives out of order; it
    // does NOT catch a terminal that arrives in order but names a
    // candidate from a different block. That cell would look
    // self-consistent — plausible winner, plausible stage, seven
    // plausible outcomes — and `check` would re-verify it green forever,
    // because `check` reads through this same extractor. This is the
    // same invariant `assert_scoreboard_contract` pins on its three
    // fixtures; here it covers all 160.
    if let Some((winner, _)) = &decided {
        assert!(
            names.contains(&winner.as_str()),
            "the outer selection's winner `{winner}` is not among the seven rows this \
             cell recorded ({names:?}) — the verdict and the candidate outcomes are from \
             different blocks. Do not bless this baseline"
        );
    }
    OuterSelection {
        outcomes,
        decided,
        rows: tail.iter().map(|e| (*e).clone()).collect(),
        shadow: outer_shadow(events),
    }
}

/// One cell's result: the committed record, the scoreboard rows
/// `policy_replay` replays, and the live shadow comparison the engine
/// emitted alongside them.
///
/// `rows` and `shadow` are deliberately NOT fields of [`Cell`]: `Cell`
/// is what gets serialised into the committed baseline, and adding to it
/// would force a re-bless of 160 rows for data the equivalence rule does
/// not compare. The baseline stays byte-identical across this phase,
/// which is itself part of the evidence that nothing moved.
struct CellRun {
    cell: Cell,
    rows: Vec<TraceEvent>,
    shadow: Option<ShadowOutcome>,
}

/// One cell: the committed record, the scoreboard rows `policy_replay`
/// replays, and the shadow comparison. One solve, three consumers —
/// `policy_replay`'s shape, widened by Phase 2a.
fn run_cell(
    f: &Fixture,
    machine: &str,
    opts_label: &str,
    apply: fn(&mut LayoutOptions),
) -> CellRun {
    let base = Cell {
        fixture: f.label.to_string(),
        machine: machine.to_string(),
        options: opts_label.to_string(),
        status: String::new(),
        winner: None,
        stage: None,
        outcomes: String::new(),
    };
    let inputs: FxHashSet<String> = f.inputs.iter().map(|s| s.to_string()).collect();
    let Ok(sr) = solver::solve(f.item, f.rate, &inputs, machine) else {
        return CellRun {
            cell: Cell { status: "no-solve".into(), ..base },
            rows: Vec::new(),
            shadow: None,
        };
    };

    let mut opts = LayoutOptions { max_belt_tier: f.belt.map(str::to_string), ..Default::default() };
    apply(&mut opts);

    let guard = trace::start_trace();
    let built = layout::build_bus_layout(&sr, opts);
    let events = trace::drain_events();
    drop(guard);

    let OuterSelection { outcomes, decided, rows, shadow } = outer_selection(&events);
    let cell = match decided {
        Some((winner, stage)) => Cell {
            // A build can refuse AFTER the search picked a winner (the
            // pick is not the last thing `build_bus_layout` does), so
            // status distinguishes them rather than collapsing both into
            // "decided".
            status: if built.is_ok() { "decided".into() } else { "decided-then-refused".into() },
            winner: Some(winner),
            stage: Some(stage_name(stage).to_string()),
            outcomes,
            ..base
        },
        None if outcomes.is_empty() => Cell { status: "no-selection".into(), ..base },
        None => Cell { status: "no-winner".into(), outcomes, ..base },
    };
    CellRun { cell, rows, shadow }
}

/// Print the corpus as the campaign's key table: rows are
/// fixture×machine, columns are option sets, and a cell that differs
/// from that row's `default` is marked so the winner-changes are
/// readable at a glance.
fn print_grid(cells: &[Cell]) {
    let mut by_row: BTreeMap<(&str, &str), BTreeMap<&str, &Cell>> = BTreeMap::new();
    for c in cells {
        by_row
            .entry((c.fixture.as_str(), c.machine.as_str()))
            .or_default()
            .insert(c.options.as_str(), c);
    }
    let cell_text = |c: &Cell| match c.status.as_str() {
        "decided" | "decided-then-refused" => format!(
            "{}/{}",
            c.winner.as_deref().unwrap_or("?"),
            c.stage.as_deref().unwrap_or("?")
        ),
        other => other.to_string(),
    };

    println!("\n=== RFC-070 parity corpus: {} cells ===", cells.len());
    println!(
        "(cell = winner/deciding-stage. `!=` marks an option set whose verdict differs \
         from that row's `default` — the claim surface Phase 2a's shadow loop must \
         reproduce)"
    );
    let mut changed_rows = 0usize;
    for ((fixture, machine), sets) in &by_row {
        println!("\n-- {fixture}  [{machine}]");
        let default_verdict = sets.get("default").map(|c| c.verdict());
        let mut row_changed = false;
        for (label, _) in OPTION_SETS {
            let Some(c) = sets.get(label) else { continue };
            let differs = default_verdict.is_some_and(|d| d != c.verdict()) && *label != "default";
            if differs {
                row_changed = true;
            }
            println!("   {:<12} {:<2} {}", label, if differs { "!=" } else { "" }, cell_text(c));
        }
        if row_changed {
            changed_rows += 1;
        }
    }
    println!(
        "\n{changed_rows}/{} fixture×machine rows change verdict across the option-set axis.",
        by_row.len()
    );

    // Stage distribution — the column K70-1 turns on.
    let mut stages: BTreeMap<&str, usize> = BTreeMap::new();
    let mut statuses: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cells {
        *statuses.entry(c.status.as_str()).or_default() += 1;
        if let Some(s) = c.stage.as_deref() {
            *stages.entry(s).or_default() += 1;
        }
    }
    println!("\nstatus distribution: {statuses:?}");
    println!("deciding-stage distribution: {stages:?}");
}

/// Report the option-set winner changes as an explicit list — the table
/// above shows WHERE, this shows WHAT, which is what the RFC's
/// divergence-equivalence rule is written against.
fn print_divergences(cells: &[Cell]) {
    let mut by_row: BTreeMap<(&str, &str), BTreeMap<&str, &Cell>> = BTreeMap::new();
    for c in cells {
        by_row
            .entry((c.fixture.as_str(), c.machine.as_str()))
            .or_default()
            .insert(c.options.as_str(), c);
    }
    println!("\n=== option-set verdict changes (vs that row's `default`) ===");
    let mut major = 0usize;
    let mut minor = 0usize;
    for ((fixture, machine), sets) in &by_row {
        let Some(d) = sets.get("default") else { continue };
        for (label, _) in OPTION_SETS {
            if *label == "default" {
                continue;
            }
            let Some(c) = sets.get(label) else { continue };
            if c.verdict() == d.verdict() {
                continue;
            }
            let kind = if c.winner == d.winner && c.status == d.status {
                minor += 1;
                "MINOR (stage only)"
            } else {
                major += 1;
                "MAJOR (winner/status)"
            };
            // Variant FIRST, default second — the order the sentence
            // reads in. (Written the other way round first, which
            // printed every divergence with its two sides swapped: the
            // `tier1_gear_am1`/`cells-off` row claimed default decided
            // at `best-accepted` when W1b had measured it at
            // `best-error-free`. Caught by that known value, which is
            // the argument for keeping a hand-checked datapoint around.)
            println!(
                "  {kind:<21} {fixture} [{machine}] {label} = {:?}/{:?}/{:?}   default = \
                 {:?}/{:?}/{:?}",
                c.status, c.winner, c.stage, d.status, d.winner, d.stage
            );
        }
    }
    println!("  ({major} major, {minor} minor)");
}

// =====================================================================
// RFC-070 Phase 1b (#689 W2b): the `policy_replay` acceptance harness
// =====================================================================

/// Turn one recorded scoreboard row into the profile the v2 comparators
/// consume. Gaps stay gaps: a `None` count means no mechanism computed
/// one on that call, and the stage that would have read it skips —
/// which is precisely how v1's lazy sites behave.
fn profile_from_row(ev: &TraceEvent) -> IssueProfile {
    let TraceEvent::SelectionCandidateEvaluated {
        outcome,
        reason,
        score,
        accepted,
        accepted_reason,
        errors,
        selection_warnings,
        layout_warnings,
        contamination_errors,
        starvation_errors,
        structural_errors,
        ..
    } = ev
    else {
        panic!("profile_from_row wants a scoreboard row, got {ev:?}");
    };
    // All three channels are written together by `record_counts`, and
    // all three kind fields by `record_kinds`. A partial would mean the
    // instrument changed under us, and silently reading the present half
    // would fabricate the absent one.
    let counts_present =
        [errors.is_some(), selection_warnings.is_some(), layout_warnings.is_some()];
    assert!(
        counts_present.iter().all(|p| *p) || counts_present.iter().all(|p| !*p),
        "a scoreboard row carries a PARTIAL count triple {counts_present:?} — the three \
         channels are written together, so this is an instrument change, not a gap"
    );
    let kinds_present = [
        contamination_errors.is_some(),
        starvation_errors.is_some(),
        structural_errors.is_some(),
    ];
    assert!(
        kinds_present.iter().all(|p| *p) || kinds_present.iter().all(|p| !*p),
        "a scoreboard row carries a PARTIAL kind triple {kinds_present:?}"
    );
    IssueProfile {
        outcome: Some(*outcome),
        refusal_reason: reason.clone(),
        score: *score,
        accepted: *accepted,
        accepted_reason: accepted_reason.clone(),
        counts: match (errors, selection_warnings, layout_warnings) {
            (Some(e), Some(w), Some(lw)) => Some(IssueCounts {
                errors: *e,
                selection_warnings: *w,
                layout_warnings: *lw,
            }),
            _ => None,
        },
        kinds: match (contamination_errors, starvation_errors, structural_errors) {
            (Some(c), Some(s), Some(st)) => Some(ErrorKindCounts {
                contamination: *c,
                starvation: *s,
                structural: *st,
            }),
            _ => None,
        },
    }
}

/// **The Phase-1b acceptance bar** (RFC-070 §"The Phase-1b acceptance
/// harness"): the offline precursor to K70-1.
///
/// One live corpus run, two consumers. v1 decides each cell as normal;
/// the harness captures that cell's emitted per-candidate profiles
/// in-process, feeds them through `selection_policy::decide`, and
/// requires v2's winner AND deciding stage to match **both** the live v1
/// decision and the committed #694 baseline, on all 140 decided cells.
///
/// Why the profiles are captured live rather than read from the
/// baseline: the committed baseline deliberately stores only
/// `(status, winner, stage, outcomes)`. The verdict NUMBERS are
/// structurally holed (RFC-070's Phase-0b oracle gaps) and a baseline
/// pinning them would pin gaps as facts. So the profiles exist only in
/// the live `SelectionCandidateEvaluated` events. There is no second
/// layout pass per cell — the "replay" is over captured profiles, not
/// re-produced layouts. The live shadow against freshly produced layouts
/// is Phase 2a's job, and K70-1 is adjudicated there.
///
/// Same cache-relative posture as `parity_corpus` itself: run with the
/// zone-cache pin, and read a divergence out of a whole-corpus re-run
/// rather than an isolated one (see the module doc's reproducibility
/// note).
///
/// A failure here is a CAMPAIGN-LEVEL finding, not a test bug: it means
/// today's decisions are not expressible as policy data over the
/// recorded measurements, which is K70-1's precursor firing.
///
/// # What this does NOT cover — read before quoting "140/140"
///
/// - **CI never runs it.** `#[ignore]`d like its sibling, for the same
///   cache-relative reason. "The parity harness passes" always refers to
///   a hand-run sweep with the pin, never to a green CI badge. The
///   always-on gate is the comparator unit tier in
///   `bus::selection_policy`.
/// - **Zero gate coverage.** `decide()` consumes already-produced
///   profiles and never evaluates a `ProducerGate`, so every clause of
///   every producer's eligibility gate is untested by this harness; the
///   only gate coverage is two unit tests. A mis-transcribed gate is
///   invisible here and would first surface at the Phase-2a shadow,
///   where it changes the candidate SET rather than the ranking.
/// - **Two of the three comparators are unexercised by the corpus** —
///   the component-wise floor's non-lexicographic-ness and the #474
///   non-shadowing rule both survive being broken with 140/140 intact.
///   Measured, not assumed; see the RFC decision log.
#[test]
#[ignore = "RFC-070 Phase 1b policy replay — runs the full #694 corpus; \
            run with --ignored --nocapture and the zone-cache pin"]
fn policy_replay() {
    let policy = SelectionPolicy::current();
    // The profile vector is keyed by registration order, so a
    // registration that does not line up with the recorded slot order
    // would rank one candidate's measurement under another's policy.
    // Spelled longhand against this file's own EXPECTED_ORDER for the
    // same reason that list exists at all.
    assert_eq!(
        policy.producers.iter().map(|p| p.name).collect::<Vec<_>>(),
        EXPECTED_ORDER,
        "SelectionPolicy::current() does not register the seven producers in the slot \
         order the scoreboard records"
    );

    let pin_hash = hash_zone_cache();
    // REQUIRED, not optional. An absent baseline used to leave
    // `baseline_cells` empty, which silently turned every
    // v2-vs-baseline comparison into a no-op while the run still
    // reported green — the "compared nothing reads as clean" shape #693
    // closed elsewhere (#698 review round 1 named the neighbouring case;
    // this disjunct is the same hazard one path over).
    let text = std::fs::read_to_string(baseline_path())
        .expect("policy_replay compares against the committed #694 baseline; it must exist");
    let baseline: Baseline = serde_json::from_str(&text).expect("committed baseline parses");
    let baseline_cells: BTreeMap<(&str, &str, &str), &Cell> =
        baseline.cells.iter().map(|c| (c.key(), c)).collect();
    // The count of DECIDED cells comes from the committed record, not a
    // hand-typed literal that can go stale against it (#698 review round
    // 1, finding 4). A legitimate corpus widening re-blesses the
    // baseline and this follows; a candidate field that moved without a
    // re-bless still fails, which is the case the guard is for.
    let expected_decided = baseline.cells.iter().filter(|c| c.stage.is_some()).count();
    let provenance_ok =
        baseline.zone_cache_hash.is_some() && baseline.zone_cache_hash == pin_hash;

    let mut decided = 0usize;
    let mut replay_diffs: Vec<String> = Vec::new();
    let mut baseline_diffs: Vec<String> = Vec::new();
    let mut stage_hits: BTreeMap<&str, usize> = BTreeMap::new();

    for f in FIXTURES {
        for machine in f.machines {
            for (label, apply) in OPTION_SETS {
                let CellRun { cell, rows, .. } = run_cell(f, machine, label, *apply);
                let key = format!("{}[{machine}]/{label}", f.label);

                // v1 decided this cell against the committed record.
                // Reported separately from the replay result: a live-vs-
                // baseline difference is an ENGINE or provenance change,
                // and reading it as a policy failure would misattribute
                // the finding.
                let baseline_cell = baseline_cells.get(&cell.key());
                let drifted = baseline_cell.is_some_and(|b| b.verdict() != cell.verdict());
                if drifted {
                    let b = baseline_cell.expect("drifted implies a baseline cell");
                    baseline_diffs.push(format!(
                        "  {key}: baseline {:?} -> live {:?}",
                        b.verdict(),
                        cell.verdict()
                    ));
                }

                let Some(stage) = cell.stage.as_deref() else {
                    // `no-solve` / `no-selection` / `no-winner`: v1 named
                    // no winner, so there is nothing for the program to
                    // reproduce. The 20 `no-solve` cells live here.
                    assert!(
                        rows.is_empty() || cell.status == "no-winner",
                        "cell {key} has scoreboard rows but no stage and status {:?}",
                        cell.status
                    );
                    continue;
                };
                decided += 1;
                *stage_hits.entry(stage_label(stage)).or_default() += 1;

                assert_eq!(
                    rows.len(),
                    EXPECTED_ORDER.len(),
                    "cell {key} decided but recorded {} rows",
                    rows.len()
                );
                let profiles: Vec<IssueProfile> = rows.iter().map(profile_from_row).collect();
                let v2 = decide(&profiles, &policy);
                let v2_verdict = v2.map(|d| {
                    (policy.producers[d.winner].name.to_string(), stage_name(d.stage).to_string())
                });
                let v1_verdict = Some((
                    cell.winner.clone().expect("a decided cell names a winner"),
                    stage.to_string(),
                ));
                if v2_verdict != v1_verdict {
                    replay_diffs.push(format!(
                        "  {key}: v1 {v1_verdict:?} -> policy {v2_verdict:?}\n      \
                         outcomes: {}",
                        cell.outcomes
                    ));
                    continue;
                }
                // …and against the committed baseline, which is the
                // record Phase 2a will diff against.
                //
                // SKIPPED on a drifted cell, and that guard is
                // load-bearing (#698 review round 1, finding 1). Where
                // v1 itself has moved off the baseline, "policy == live
                // v1" is the correct result and "policy != baseline"
                // follows from the drift alone — pushing it into
                // `replay_diffs` would fire the campaign-finding
                // assertion at the bottom against exactly the cells the
                // NOTE above tells the reader to treat as an ENGINE
                // change. That is a manufactured K70-1 finding, and the
                // conditions that produce it (a stale baseline, a
                // mis-pinned cache) are the ones where a reader is most
                // primed to believe it.
                if !drifted {
                    if let Some(b) = baseline_cell {
                        let b_verdict = Some((
                            b.winner.clone().unwrap_or_default(),
                            b.stage.clone().unwrap_or_default(),
                        ));
                        if v2_verdict != b_verdict {
                            replay_diffs.push(format!(
                                "  {key}: baseline {b_verdict:?} -> policy {v2_verdict:?}"
                            ));
                        }
                    }
                }
            }
        }
    }

    eprintln!("\n=== RFC-070 policy replay ===");
    eprintln!("decided cells replayed: {decided}");
    eprintln!("deciding-stage distribution: {stage_hits:?}");
    if !baseline_diffs.is_empty() {
        eprintln!(
            "\nNOTE: v1 itself diverged from the committed baseline on {} cell(s). The \
             replay below is still meaningful (it compares against the LIVE v1 decision \
             too), but these cells are an engine or provenance change, not a policy \
             finding:\n{}",
            baseline_diffs.len(),
            baseline_diffs.join("\n")
        );
    }
    // A provenance mismatch FAILS. It used to print a NOTE and pass —
    // which is the "compared nothing reads as clean" shape #693 closed
    // and #694 round 3 closed again in this very file's `check` mode,
    // reappearing one path over (#698 review round 5, the one genuinely
    // new argument in five rounds of raising this test's coverage).
    // Under an unidentifiable cache a green replay is not evidence that
    // the committed record reproduces; it is evidence about a corpus
    // nobody else can re-take.
    let provenance_escape =
        std::env::var("SPAGHETTIO_POLICY_REPLAY").as_deref() == Ok("any-cache");
    assert!(
        provenance_ok || provenance_escape,
        "PROVENANCE MISMATCH: this run's zone cache is {pin_hash:?}, the baseline was \
         blessed against {:?}. (A `null` on EITHER side counts as a mismatch — an \
         unidentified cache is not the same cache, it is an unknown one.) The corpus is \
         cache-relative, so a replay taken here is not evidence about the committed \
         record. Re-run with \
         SPAGHETTIO_ZONE_CACHE_PATH=crates/core/data/sat-zones-ci.bin, or pass \
         SPAGHETTIO_POLICY_REPLAY=any-cache if comparing across caches is what you meant.",
        baseline.zone_cache_hash
    );

    // "Verified clean" must be distinguishable from "compared nothing"
    // (the #693 lesson). The expected count is the committed record's
    // own, so a deliberate corpus widening travels with its re-bless
    // while a candidate field that moved under a stale baseline still
    // fails here.
    assert!(expected_decided > 0, "the committed baseline records no decided cells at all");
    assert_eq!(
        decided, expected_decided,
        "the corpus decided {decided} cells, not the {expected_decided} the committed \
         baseline records — the candidate field moved, and a replay over a different cell \
         set is not evidence about this baseline"
    );
    assert!(
        replay_diffs.is_empty(),
        "SelectionPolicy::decide did not reproduce {} of {decided} decided cells. THIS IS \
         A CAMPAIGN-LEVEL FINDING, not a test bug: it means today's selection is not \
         expressible as policy data over the recorded measurements — K70-1's precursor. \
         Report it; do not add candidate-name-keyed logic to make it pass.\n{}",
        replay_diffs.len(),
        replay_diffs.join("\n")
    );
}

/// `stage_name`'s inverse-ish: the label a `Cell` stores, normalised for
/// the histogram. A stage the corpus records but this file does not know
/// is a loud `unknown:` row rather than a silent bucket.
fn stage_label(stage: &str) -> &'static str {
    match stage {
        "merge-tap" => "merge-tap",
        "scoped-pairwise" => "scoped-pairwise",
        "best-error-free" => "best-error-free",
        "best-accepted" => "best-accepted",
        "first-produced" => "first-produced",
        _ => "unknown",
    }
}

#[test]
#[ignore = "RFC-070 Phase 0c corpus baseline — run with --ignored --nocapture; \
            see module docs for bless/check"]
fn parity_corpus() {
    let pin_hash = hash_zone_cache();
    let mut cells = Vec::new();
    let mut shadow_report = ShadowReport::default();
    for f in FIXTURES {
        for machine in f.machines {
            for (label, apply) in OPTION_SETS {
                let run = run_cell(f, machine, label, *apply);
                shadow_report.absorb(&format!("{}[{machine}]/{label}", f.label), &run);
                cells.push(run.cell);
            }
        }
    }

    print_grid(&cells);
    print_divergences(&cells);
    shadow_report.print();

    match std::env::var("SPAGHETTIO_PARITY_CORPUS").as_deref() {
        Ok(mode @ ("bless" | "bless-repin")) => {
            // Blessing DELIBERATELY overwrites verdicts — re-taking the
            // baseline after an intentional engine change is what bless
            // is for, and refusing on divergence would make it unusable
            // (#694 review, finding 1, half-absorbed).
            //
            // The PROVENANCE is different. This baseline is pinned-cache-
            // relative, so a bless taken against a different cache — or
            // against none — writes 160 rows nobody can reproduce, and
            // the mismatch is invisible afterwards because bless also
            // rewrites the recorded hash. That half of the finding is
            // real, so it is now a refusal with a named escape hatch.
            // "File missing" and "file present but corrupt" must not
            // collapse into the same `None` (#694 review round 3 — the
            // same disjunct-disarms-the-guard class round 2 retired in
            // the null-hash path). A baseline left truncated by a killed
            // bless would otherwise skip the hash check entirely and get
            // silently overwritten by the next plain `bless`, which is
            // precisely the run whose provenance nobody can check.
            if mode == "bless" {
                let prior: Option<Baseline> = match std::fs::read_to_string(baseline_path()) {
                    // ONLY not-found means "first bless". A permission or
                    // I/O error is a baseline that exists and cannot be
                    // read, which must not be treated as one that is not
                    // there — same conflation as the parse arm below,
                    // caught one round later (#694 round 4).
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(e) => panic!(
                        "the committed baseline exists but could not be read ({e}) — \
                         refusing to bless over it, because its zone-cache hash is what \
                         this run must be checked against. Fix the read error and re-run, \
                         or pass SPAGHETTIO_PARITY_CORPUS=bless-repin to overwrite it \
                         deliberately"
                    ),
                    Ok(text) => Some(serde_json::from_str::<Baseline>(&text).unwrap_or_else(
                        |e| {
                            panic!(
                                "the committed baseline exists but does not parse ({e}) — \
                                 refusing to bless over it, because a corrupt file cannot \
                                 supply the zone-cache hash this run must be checked \
                                 against. Restore it (`git checkout -- {:?}`) and re-run, \
                                 or pass SPAGHETTIO_PARITY_CORPUS=bless-repin to overwrite \
                                 it deliberately",
                                baseline_path()
                            )
                        },
                    )),
                };
                assert!(
                    pin_hash.is_some(),
                    "refusing to bless with no resolvable zone cache: this baseline is \
                     cache-relative and a null `zone_cache_hash` cannot be checked by \
                     anyone later. Set SPAGHETTIO_ZONE_CACHE_PATH to \
                     crates/core/data/sat-zones-ci.bin, or pass \
                     SPAGHETTIO_PARITY_CORPUS=bless-repin if you really mean it"
                );
                if let Some(p) = &prior {
                    // NOT `is_none() || ==`. A prior baseline recording
                    // no hash must fail the guard too, not satisfy it:
                    // one `bless-repin` with no resolvable cache would
                    // otherwise write `null` and permanently disarm every
                    // later plain `bless` through that disjunct (#694
                    // review round 2). "No prior baseline" (handled by
                    // the `if let`) and "prior baseline recorded no hash"
                    // are different facts, and only the first is benign.
                    assert!(
                        p.zone_cache_hash == pin_hash,
                        "refusing to bless against a DIFFERENT zone cache than the \
                         committed baseline's: baseline {:?}, this run {pin_hash:?}. The \
                         cells below may differ for provenance reasons rather than engine \
                         ones, and blessing would overwrite the old hash so nobody could \
                         tell afterwards. Re-run with the committed pin, or pass \
                         SPAGHETTIO_PARITY_CORPUS=bless-repin to deliberately re-take the \
                         baseline on a new cache",
                        p.zone_cache_hash
                    );
                }
            }
            let baseline = Baseline { zone_cache_hash: pin_hash.clone(), cells };
            let json = serde_json::to_string_pretty(&baseline).expect("baseline serializes");
            std::fs::write(baseline_path(), json + "\n").expect("write baseline");
            eprintln!(
                "BLESSED ({mode}) {} cell(s) to {:?} (zone-cache hash: {pin_hash:?})",
                baseline.cells.len(),
                baseline_path()
            );
        }
        Ok(mode @ ("check" | "check-any-cache")) => {
            // RFC-070 Phase 2a: the shadow gate, asserted BEFORE the
            // baseline comparison and deliberately so. The two failures
            // are independent and answer different questions, but only
            // one of them is interpretable under a mis-pinned cache: a
            // baseline divergence may be provenance, while a shadow
            // disagreement is two programs answering differently about
            // the same solve — true regardless of which layout that
            // solve produced. So the interpretable failure leads.
            shadow_report.assert_clean();
            let text = std::fs::read_to_string(baseline_path())
                .expect("SPAGHETTIO_PARITY_CORPUS=check needs a committed baseline");
            let baseline: Baseline = serde_json::from_str(&text).expect("baseline parses");
            // A mismatched (or absent) pin does not stop the comparison —
            // a clean result under a different cache is still worth
            // knowing — but it must LEAD the failure if there is one,
            // rather than trailing it as a NOTE the reader met before the
            // diffs and has forgotten by the time they matter (#694
            // review round 1, finding 4).
            //
            // And it must FAIL, even when the cells all match (#694 review
            // round 3). Round 1's version printed the mismatch and passed
            // green, which is the same shape as "compared nothing, read as
            // clean" that #693 closed: a green `check` under a cache
            // nobody can identify is not evidence the baseline
            // reproduces. `check-any-cache` is the named escape for
            // deliberately comparing across caches.
            //
            // `None == None` is a MISMATCH here, not a match (#694 review
            // round 5). `bless-repin` may legitimately write a null hash,
            // and a later `check` on a cache-less host would then compare
            // null to null, find no mismatch, and green-check 160 rows
            // nobody can reproduce — the same "compared nothing reads as
            // clean" shape round 3 closed on the Some-vs-None pair, with
            // the None-vs-None pair still getting through. Both sides
            // must be `Some` AND equal.
            let provenance = if pin_hash.is_none()
                || baseline.zone_cache_hash.is_none()
                || pin_hash != baseline.zone_cache_hash
            {
                let msg = format!(
                    "PROVENANCE MISMATCH: this run's zone cache is {pin_hash:?}, the \
                     baseline was blessed against {:?}. (A `null` on EITHER side counts as \
                     a mismatch — an unidentified cache is not the same cache, it is an \
                     unknown one.) This corpus is cache-relative, so the divergences below \
                     may be provenance rather than an engine change — re-run with the \
                     committed pin \
                     (SPAGHETTIO_ZONE_CACHE_PATH=crates/core/data/sat-zones-ci.bin) before \
                     reading them as findings.",
                    baseline.zone_cache_hash
                );
                eprintln!("\n{msg}");
                Some(msg)
            } else {
                None
            };
            let old: BTreeMap<_, _> = baseline.cells.iter().map(|c| (c.key(), c)).collect();
            let new: BTreeMap<_, _> = cells.iter().map(|c| (c.key(), c)).collect();
            let mut diffs = Vec::new();
            // Reported, never failed: a cell whose CANDIDATE FIELD moved
            // while the verdict held — an arm that used to produce now
            // refuses, say. The equivalence rule is winner+stage, so
            // failing here would be inventing a stricter contract than
            // the RFC states; but staying silent would let the field
            // shift under a green check, which is the shape this repo
            // keeps getting bitten by. So it prints.
            let mut field_shifts = Vec::new();
            for (k, c) in &new {
                match old.get(k) {
                    // Equality is the RFC's rule, not `Cell::eq`:
                    // `outcomes` is recorded for adjudication and is
                    // deliberately NOT part of the comparison.
                    Some(b) if b.verdict() == c.verdict() => {
                        if b.outcomes != c.outcomes {
                            field_shifts.push(format!(
                                "  {k:?}: {} -> {}",
                                b.outcomes, c.outcomes
                            ));
                        }
                    }
                    Some(b) => diffs.push(format!(
                        "  DIVERGED {k:?}: baseline {:?} -> now {:?}",
                        b.verdict(),
                        c.verdict()
                    )),
                    None => diffs.push(format!("  NEW CELL {k:?}: {:?}", c.verdict())),
                }
            }
            if !field_shifts.is_empty() {
                eprintln!(
                    "\nNOTE: {} cell(s) kept their verdict but changed which candidates \
                     produced/refused. Not a divergence under the equivalence rule; still \
                     worth reading before trusting a green check:\n{}",
                    field_shifts.len(),
                    field_shifts.join("\n")
                );
            }
            for k in old.keys() {
                if !new.contains_key(k) {
                    diffs.push(format!("  MISSING CELL {k:?} (in baseline, not produced now)"));
                }
            }
            // "Verified clean" must be distinguishable from "compared
            // nothing" — the #693 lesson. An empty baseline would
            // otherwise pass silently.
            assert!(
                !baseline.cells.is_empty() && !new.is_empty(),
                "check compared NOTHING: baseline has {} cells, this run produced {}",
                baseline.cells.len(),
                new.len()
            );
            assert!(
                diffs.is_empty(),
                "{}parity corpus diverged from the committed baseline ({} cell(s)):\n{}",
                // Provenance FIRST when it applies: a mis-pinned run's
                // diffs are uninterpretable, and a failure message that
                // opens with the diff list invites reading them as
                // findings anyway.
                provenance.as_deref().map(|p| format!("{p}\n\n")).unwrap_or_default(),
                diffs.len(),
                diffs.join("\n")
            );
            // Only reached when every cell matched. A green result under
            // an unidentifiable cache still must not read as clean.
            assert!(
                provenance.is_none() || mode == "check-any-cache",
                "{}\n\nEvery cell matched, but under a DIFFERENT zone cache than the \
                 baseline was blessed against — so this run is not evidence the committed \
                 baseline reproduces. Re-run with the committed pin, or pass \
                 SPAGHETTIO_PARITY_CORPUS=check-any-cache if comparing across caches is \
                 what you meant.",
                provenance.as_deref().unwrap_or_default()
            );
            eprintln!(
                "checked {} cell(s) against the committed baseline{}",
                new.len(),
                if provenance.is_some() {
                    " (check-any-cache: UNDER A DIFFERENT ZONE CACHE — see above)"
                } else {
                    ""
                }
            );
        }
        _ => {
            // Report-only default: prints, asserts nothing.
        }
    }
}

// =====================================================================
// RFC-070 Phase 2a (#689 W3a): the non-ignored shadow smoke tier
// =====================================================================

/// The six census fixtures at the machine tier their #691 labels name,
/// under production defaults. Spelled longhand rather than sliced off
/// `FIXTURES` — a list derived from the thing it checks cannot notice
/// that thing changing, the same argument `EXPECTED_ORDER` rests on.
///
/// These are the six that `check_firing_census.rs` and the junction-seed
/// census also describe, so a shadow finding here is readable against
/// both of the campaign's other instruments row by row.
const SMOKE_CELLS: &[(&str, &str)] = &[
    ("tier1_gear_am1", "assembling-machine-1"),
    ("tier2_ec_am1_10_ore", "assembling-machine-1"),
    ("tier2_ec_am2_30_ore", "assembling-machine-2"),
    ("tier3_plastic_cp_5", "chemical-plant"),
    ("tier4_ac_am2_5_unconstrained", "assembling-machine-2"),
    ("tier5_pu_am3_2_unconstrained", "assembling-machine-3"),
];

/// **The parity CI gate.** Not `#[ignore]`d: this is what runs on every
/// push, and it is the first assertion in this campaign that can.
///
/// Its two ignored siblings are cache-relative — they compare a live run
/// against a COMMITTED record, so a host with a different zone cache
/// gets different layouts and a meaningless diff, and every legitimate
/// engine change falsifies them. This test compares the v1 and v2
/// dispatches **on the same solve**: whatever layout each fixture
/// produces on this host with this cache, both programs see the same
/// scoreboard and must reach the same `(winner, stage)`. Nothing about
/// the assertion depends on WHICH layout that was, so there is no
/// re-bless treadmill and no pin to get wrong.
///
/// What it therefore covers, and what it does not: six of the corpus's
/// 160 cells, all at the `default` option set. The option-set axis
/// carries this corpus's claim surface (RFC-070 decision log), so the
/// full 160-cell sweep under `SPAGHETTIO_PARITY_CORPUS=check` remains
/// the real gate for a phase boundary — this is the tripwire between
/// them, sized so it can run unconditionally.
/// Cost: ~15s local warm (16 threads, pinned cache) and **30.9s
/// measured on the CI runner** (PR #703 head `12b1ea87`, job
/// 97039777698) — a 2x host penalty rather than the 8-18x the cold-cache
/// note in `.config/nextest.toml` records, because the rust job pins
/// `SPAGHETTIO_ZONE_CACHE_PATH`. The 300s ceiling is therefore a ~10x
/// margin over an observed number, not an extrapolation. It also carries
/// a `threads-required` override there, because six SAT-heavy solves in
/// one test on a 4-thread runner is the shape that blew
/// `tier4_..._belt_pipe_crossing`'s ceiling when parallelism was first
/// tried. The ntest ceiling is the real hang detector; nextest'''s kill
/// sits above it.
#[test]
#[ntest::timeout(300_000)]
fn shadow_agrees_with_production_on_the_census_fixtures() {
    // The third parallel candidate list, bound in CI (#698 rounds 9-10
    // carry-over (e) — the unit tier binds the other two). A shadow
    // whose profile vector is keyed differently from the scoreboard
    // would compare mis-keyed slots and could agree by luck.
    assert_eq!(
        SelectionPolicy::current().producers.iter().map(|p| p.name).collect::<Vec<_>>(),
        EXPECTED_ORDER,
        "SelectionPolicy::current() does not register the seven producers in the slot \
         order this file expects"
    );

    let mut report = ShadowReport::default();
    for (label, machine) in SMOKE_CELLS {
        let f = FIXTURES
            .iter()
            .find(|f| f.label == *label)
            .unwrap_or_else(|| panic!("smoke cell {label} is not a corpus fixture"));
        assert!(
            f.machines.contains(machine),
            "smoke cell {label} names machine {machine}, which is not on that fixture's \
             swept axis"
        );
        let run = run_cell(f, machine, "default", |_| {});
        report.absorb(&format!("{label}[{machine}]"), &run);
    }
    report.print();
    // The ONLY fixture-shaped requirement: each smoke cell must reach
    // the search, so there is something to compare. Nothing here pins
    // WHICH winner, WHICH stage, or even that the build succeeded.
    //
    // The first draft asserted `status == "decided"` per cell, and that
    // was the wrong pin (#703 review round 1, the major): the shadow's
    // verdict comparison is layout-independent, but "decided" is not —
    // a host whose zone cache differs can legitimately land a cell in
    // `decided-then-refused` (the search picked a winner and a LATER
    // build step refused, which `run_cell` distinguishes on purpose),
    // and that would red the one always-on gate with no v1/v2
    // divergence anywhere. The shadow is emitted on that path too, so
    // the comparison is still available and still meaningful; only the
    // over-tight assertion had to go.
    assert_eq!(
        report.compared(),
        SMOKE_CELLS.len(),
        "every smoke cell must reach the selection and emit a comparison; {} of {} did. \
         A cell that stopped reaching the search has nothing to shadow, so this gate \
         would be silently measuring less than it claims — replace the fixture rather \
         than letting the count drift",
        report.compared(),
        SMOKE_CELLS.len()
    );
    report.assert_clean();
}
