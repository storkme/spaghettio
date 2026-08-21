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
//! cause. The e2e harness therefore differs from production defaults on
//! TWO fields, not one, which is why `e2e-harness` below is its own
//! option set rather than a synonym for `cells-off`.
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
/// default()`, which means it is only as accurate as this list — a
/// future change to `run_e2e_inner`'s other pinned fields, or a default
/// flipping to match a harness value, would break the "this is what the
/// harness runs" claim with no corpus cell moving (#694 review round 2).
/// It cannot be asserted automatically: `run_e2e_inner` is a private fn
/// in a different test binary. So it is hand-verified instead, and here
/// is the receipt to re-check against — as of `a54b9a7a`,
/// `tests/e2e.rs:342-358` differs from the struct defaults on exactly
/// two fields, `cell_composition` (`Default::default()` → the enum's
/// `Off`) and `inserter_capacity: 0`; every other field it spells
/// (`max_inserter_tier`, `quality`, `wire_mode`, `merge_tap`,
/// `stacking`, `splitter_tap_spacers`) already equals its default.
/// Re-run that diff if either fossil is ever fixed.
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
fn outer_selection(events: &[TraceEvent]) -> (String, Option<(String, SelectionStage)>) {
    let rows: Vec<(&str, SelectionCandidateOutcome)> = events
        .iter()
        .filter_map(|e| match e {
            TraceEvent::SelectionCandidateEvaluated { name, outcome, .. } => {
                Some((name.as_str(), *outcome))
            }
            _ => None,
        })
        .collect();
    if rows.is_empty() {
        return (String::new(), None);
    }
    let tail = &rows[rows.len().saturating_sub(EXPECTED_ORDER.len())..];
    let names: Vec<&str> = tail.iter().map(|(n, _)| *n).collect();
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
        tail.iter().map(|(_, o)| outcome_name(*o)).collect::<Vec<_>>().join(",");
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
    (outcomes, decided)
}

fn run_cell(f: &Fixture, machine: &str, opts_label: &str, apply: fn(&mut LayoutOptions)) -> Cell {
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
        return Cell { status: "no-solve".into(), ..base };
    };

    let mut opts = LayoutOptions { max_belt_tier: f.belt.map(str::to_string), ..Default::default() };
    apply(&mut opts);

    let guard = trace::start_trace();
    let built = layout::build_bus_layout(&sr, opts);
    let events = trace::drain_events();
    drop(guard);

    let (outcomes, decided) = outer_selection(&events);
    match decided {
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
    }
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

#[test]
#[ignore = "RFC-070 Phase 0c corpus baseline — run with --ignored --nocapture; \
            see module docs for bless/check"]
fn parity_corpus() {
    let pin_hash = hash_zone_cache();
    let mut cells = Vec::new();
    for f in FIXTURES {
        for machine in f.machines {
            for (label, apply) in OPTION_SETS {
                cells.push(run_cell(f, machine, label, *apply));
            }
        }
    }

    print_grid(&cells);
    print_divergences(&cells);

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
