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
//! `SPAGHETTIO_PARITY_CORPUS=bless` rewrites the committed baseline;
//! `=check` fails on any cell that differs from it. Neither is CI-gated
//! (the test is `#[ignore]`d) — same posture as the stress goldens,
//! which are host-cache-relative for the same reason: the layouts depend
//! on which zone solutions the pinned cache replays.

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
/// (and `check`) never has to infer one from a `None`:
/// `decided` / `no-winner` / `refused` / `no-solve` / `no-selection`.
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

/// Cheap content hash of the pinned cache, captured BEFORE any build —
/// a run appends newly-solved zones, so hashing afterwards would describe
/// post-run bytes rather than the pin actually consulted (the mistake
/// #693 round 3 caught in the meter tripwire).
fn hash_zone_cache() -> Option<String> {
    use std::hash::{Hash, Hasher};
    let bytes = std::fs::read(resolve_zone_cache_path()).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    Some(format!("{:016x}", hasher.finish()))
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
        Ok("bless") => {
            let baseline = Baseline { zone_cache_hash: pin_hash.clone(), cells };
            let json = serde_json::to_string_pretty(&baseline).expect("baseline serializes");
            std::fs::write(baseline_path(), json + "\n").expect("write baseline");
            eprintln!(
                "BLESSED {} cell(s) to {:?} (zone-cache hash: {pin_hash:?})",
                baseline.cells.len(),
                baseline_path()
            );
        }
        Ok("check") => {
            let text = std::fs::read_to_string(baseline_path())
                .expect("SPAGHETTIO_PARITY_CORPUS=check needs a committed baseline");
            let baseline: Baseline = serde_json::from_str(&text).expect("baseline parses");
            if pin_hash != baseline.zone_cache_hash {
                eprintln!(
                    "NOTE: zone-cache hash differs from the blessed baseline's ({:?} vs \
                     {pin_hash:?}) — divergences below may be cache provenance rather \
                     than an engine change.",
                    baseline.zone_cache_hash
                );
            }
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
                "parity corpus diverged from the committed baseline ({} cell(s)):\n{}",
                diffs.len(),
                diffs.join("\n")
            );
            eprintln!("checked {} cell(s) against the committed baseline", new.len());
        }
        _ => {
            // Report-only default: prints, asserts nothing.
        }
    }
}
