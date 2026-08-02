//! RFC-064 Phase 3, Unit B — pre-registered gate adjudication for the
//! AR-rescored band packer (`PackObjective::MinAspectRatio`, Unit A, commit
//! b7db3631), on RFC-058's own gate and holdout fixtures.
//!
//! See `docs/rfc-064-spaghetti-objective.md`, "Phase 3 — row-granularity
//! rigid packing" (~lines 491-566) for the gate text and kill criterion, and
//! its "Metrics" section (~lines 204-330) for `AR_score`/`Transit_score`/
//! `ΔEntities%` definitions. This driver measures against those bars; it
//! does not tune anything to clear them — a miss here is a valid,
//! pre-registered result (the RFC's own "new, stronger falsification"), and
//! this file must not be edited to change the outcome.
//!
//! **Fixture specs.** Recovered from `crates/core/tests/cell_composition.rs`
//! — the ten-fixture `(label, item, rate, inputs, machine)` array in
//! `probe_band_packing_headroom` (~line 5197) and `rfc058_band_packing_
//! premise_holds` (~line 5395), both of which solve via
//! `solver::solve_with_palette_exclusions_and_quality(item, rate, inputs,
//! &MachinePalette::default(), machine, &FxHashSet::default(),
//! QualityTier::Normal)` and lay out via `layout::build_bus_layout(&sr,
//! LayoutOptions::default())` — reused verbatim below. The three GATE
//! fixtures are named explicitly at cell_composition.rs:5216
//! (`let gate = ["sci1-ore", "sci2-ore", "pu1-plate"];`) and again at
//! cell_composition.rs:5395-5399. The four HOLDOUT fixtures
//! (`belt5-ore`, `insert3-ore`, `gear15-ore`, `lds2-plate`) are the
//! remainder of that same ten-fixture array, matching the candidate list
//! named in this unit's brief; `ec10-ore`/`ec15-plate`/`gear5-plate` (the
//! other three entries in that array) are NOT part of either named set and
//! are excluded here.
//!
//! **Incumbent definition.** Per the RFC's Metrics section: "the layout the
//! existing decomposition search would otherwise produce for the same
//! solve, at `compact_layout: false` and no folding/row-flip/bidirectional
//! candidate selected" — exactly `CandidatePlan::new(_, FullSelectionCandidate)`
//! run under `LayoutOptions::default()` (`compact_layout: false, fold_layout:
//! false, band_packing: false` — see `bus::layout::LayoutOptions::default`),
//! no transforms.
//!
//! **Verdict policy.** The gate text requires "sim-anchored never-worse
//! (#520) on every fixture the rescored candidate ships on" as a SEPARATE,
//! out-of-scope bar (this driver is a dry, validator-only adjudication) —
//! but it also implies validation must never-worse. `Policy::fold()` (the
//! `search_snake_fold` preset) is deliberately NOT used here: it gates at
//! `MatchTier::Count` with raw-count comparison, which cannot see churn
//! (`docs/validator-reporting.md`'s nine-time failure mode: N resolved + N
//! new in one category nets to zero). This driver uses
//! `Policy::new(GatePolicy::GateInstances)` — the strictest policy
//! `verdict::never_worse` supports — paired with the `MatchTier::Provenance`
//! tier `pack_candidate_plan`'s paired `PackCorrespondenceTransform`
//! supplies, so a positioned regression is never netted away by an
//! unrelated resolved issue elsewhere in the same category.
//!
//! Run: `cargo test --manifest-path crates/core/Cargo.toml --release --test rfc064_phase3_gate -- --ignored --nocapture`

use rustc_hash::FxHashSet;

use spaghettio_core::bus::bands::PackObjective;
use spaghettio_core::bus::candidate_runner::{
    pack_candidate_plan, run_candidate_field, CandidateOutcome, CandidatePlan, FullSelectionCandidate,
};
use spaghettio_core::bus::layout::LayoutOptions;
use spaghettio_core::common::QualityTier;
use spaghettio_core::objective;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;
use spaghettio_core::verdict::{GatePolicy, Policy};

struct Fixture {
    label: &'static str,
    item: &'static str,
    rate: f64,
    inputs: &'static [&'static str],
    machine: &'static str,
    is_gate: bool,
}

/// The three gate fixtures (cell_composition.rs:5216) plus the four holdout
/// fixtures named in this unit's brief — both subsets of the same
/// ten-fixture array `probe_band_packing_headroom` defines. Specs verbatim
/// from that array; see module docs for the exact source lines.
const FIXTURES: &[Fixture] = &[
    Fixture {
        label: "sci1-ore",
        item: "automation-science-pack",
        rate: 1.0,
        inputs: &["iron-ore", "copper-ore"],
        machine: "assembling-machine-1",
        is_gate: true,
    },
    Fixture {
        label: "sci2-ore",
        item: "logistic-science-pack",
        rate: 2.0,
        inputs: &["iron-ore", "copper-ore"],
        machine: "assembling-machine-2",
        is_gate: true,
    },
    Fixture {
        label: "pu1-plate",
        item: "processing-unit",
        rate: 1.0,
        inputs: &["iron-plate", "copper-plate", "sulfuric-acid"],
        machine: "assembling-machine-2",
        is_gate: true,
    },
    Fixture {
        label: "belt5-ore",
        item: "transport-belt",
        rate: 5.0,
        inputs: &["iron-ore"],
        machine: "assembling-machine-2",
        is_gate: false,
    },
    Fixture {
        label: "insert3-ore",
        item: "inserter",
        rate: 3.0,
        inputs: &["iron-ore", "copper-ore"],
        machine: "assembling-machine-2",
        is_gate: false,
    },
    Fixture {
        label: "gear15-ore",
        item: "iron-gear-wheel",
        rate: 15.0,
        inputs: &["iron-ore"],
        machine: "assembling-machine-2",
        is_gate: false,
    },
    Fixture {
        label: "lds2-plate",
        item: "low-density-structure",
        rate: 2.0,
        inputs: &["iron-plate", "copper-plate", "plastic-bar"],
        machine: "assembling-machine-2",
        is_gate: false,
    },
];

enum FixtureOutcome {
    /// Solver refusal, incumbent-production failure, or a `PackRefusal`
    /// (verbatim reason string) — never dropped silently.
    Refused { reason: String },
    Evaluated {
        native_w: i32,
        native_h: i32,
        native_ar: f64,
        packed_w: i32,
        packed_h: i32,
        packed_ar: f64,
        ar_score: f64,
        transit_native: f64,
        transit_packed: f64,
        transit_score: f64,
        unattributed_native: usize,
        unattributed_packed: usize,
        delta_entities_pct: f64,
        verdict_pass: bool,
        regressed_categories: Vec<String>,
        composite_winner: String,
    },
}

#[test]
#[ignore = "RFC-064 Phase 3 gate adjudication — run with --release --ignored --nocapture"]
fn rfc064_phase3_gate_adjudication() {
    // Strictest policy `verdict::never_worse` supports — see module docs.
    let policy = Policy::new(GatePolicy::GateInstances);
    let mut results: Vec<(&Fixture, FixtureOutcome)> = Vec::new();

    for fx in FIXTURES {
        let inputs_set: FxHashSet<String> = fx.inputs.iter().map(|s| s.to_string()).collect();
        let sr = match solver::solve_with_palette_exclusions_and_quality(
            fx.item,
            fx.rate,
            &inputs_set,
            &MachinePalette::default(),
            fx.machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        ) {
            Ok(sr) => sr,
            Err(e) => {
                results.push((fx, FixtureOutcome::Refused { reason: format!("solver refused: {e}") }));
                continue;
            }
        };

        // Native incumbent: full decomposition search, compact_layout:
        // false, no fold, no pack — LayoutOptions::default() is exactly
        // that (see module docs).
        let opts = LayoutOptions::default();
        let incumbent = CandidatePlan::new("native", FullSelectionCandidate);
        let pack_plan = pack_candidate_plan("pack", PackObjective::MinAspectRatio);

        let field = match run_candidate_field(
            &sr,
            &opts,
            &incumbent,
            std::slice::from_ref(&pack_plan),
            &policy,
        ) {
            Ok(f) => f,
            Err(e) => {
                results.push((fx, FixtureOutcome::Refused { reason: format!("incumbent failed to produce/measure: {e}") }));
                continue;
            }
        };

        let pack_entry = field.entries.iter().find(|e| e.name() == "pack");
        match pack_entry {
            Some(CandidateOutcome::Refused { reason, .. }) => {
                results.push((fx, FixtureOutcome::Refused { reason: reason.clone() }));
            }
            Some(CandidateOutcome::Evaluated(ec)) => {
                let native_measure =
                    objective::measure(&field.incumbent, &sr).expect("native layout must measure");
                let packed_measure =
                    objective::measure(&ec.layout, &sr).expect("packed layout must measure");
                let regressed: Vec<String> =
                    ec.verdict.regressed_categories().map(|s| s.to_string()).collect();
                results.push((
                    fx,
                    FixtureOutcome::Evaluated {
                        native_w: native_measure.bbox_width,
                        native_h: native_measure.bbox_height,
                        native_ar: native_measure.aspect_ratio,
                        packed_w: packed_measure.bbox_width,
                        packed_h: packed_measure.bbox_height,
                        packed_ar: packed_measure.aspect_ratio,
                        ar_score: ec.scores.ar_score,
                        transit_native: native_measure.transit,
                        transit_packed: packed_measure.transit,
                        transit_score: ec.scores.transit_score,
                        unattributed_native: native_measure.unattributed_edge_count,
                        unattributed_packed: packed_measure.unattributed_edge_count,
                        delta_entities_pct: ec.scores.delta_entities_pct,
                        verdict_pass: ec.verdict.pass,
                        regressed_categories: regressed,
                        composite_winner: field.winner_name.clone(),
                    },
                ));
            }
            None => {
                results.push((
                    fx,
                    FixtureOutcome::Refused {
                        reason: "pack candidate absent from field entries (unexpected — run_candidate_field bug?)"
                            .to_string(),
                    },
                ));
            }
        }
    }

    // -----------------------------------------------------------------
    // Per-fixture table
    // -----------------------------------------------------------------
    println!("\n=== RFC-064 Phase 3 gate adjudication ===\n");
    println!(
        "| fixture | gate | native w×h (AR) | packed w×h (AR) | AR_score | Transit(native) | Transit(packed) | Transit_score | unattrib N/P | ΔEntities% | verdict | regressed | winner |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for (fx, outcome) in &results {
        match outcome {
            FixtureOutcome::Refused { reason } => {
                println!(
                    "| {} | {} | REFUSED: {reason} | | | | | | | | | | |",
                    fx.label,
                    if fx.is_gate { "yes" } else { "no" }
                );
            }
            FixtureOutcome::Evaluated {
                native_w,
                native_h,
                native_ar,
                packed_w,
                packed_h,
                packed_ar,
                ar_score,
                transit_native,
                transit_packed,
                transit_score,
                unattributed_native,
                unattributed_packed,
                delta_entities_pct,
                verdict_pass,
                regressed_categories,
                composite_winner,
            } => {
                println!(
                    "| {} | {} | {native_w}×{native_h} ({native_ar:.2}) | {packed_w}×{packed_h} ({packed_ar:.2}) | {ar_score:.4} | {transit_native:.1} | {transit_packed:.1} | {transit_score:+.4} | {unattributed_native}/{unattributed_packed} | {:+.1}% | {} | {} | {composite_winner} |",
                    fx.label,
                    if fx.is_gate { "yes" } else { "no" },
                    delta_entities_pct * 100.0,
                    if *verdict_pass { "PASS" } else { "FAIL" },
                    if regressed_categories.is_empty() {
                        "-".to_string()
                    } else {
                        regressed_categories.join(",")
                    },
                );
            }
        }
    }

    // -----------------------------------------------------------------
    // Adjudication — pre-registered, from the RFC's own text. Nothing
    // below may be adjusted to change the outcome.
    // -----------------------------------------------------------------
    println!("\n=== Adjudication ===\n");

    println!(
        "Bar 3 (sim-anchored never-worse, #520): OUT OF SCOPE for this dry, validator-only \
         adjudication. Nothing ships from this unit regardless of Bar 1/2 outcome."
    );

    // Bar 1: AR_score >= 0.5, per fixture, on gate AND holdout (the RFC's
    // gate text names both sets for this bar).
    println!("\nBar 1 — AR_score >= 0.5, per fixture (no summing across fixtures):");
    let mut gate_refused: Vec<&str> = Vec::new();
    let mut gate_clear = true;
    for (fx, outcome) in &results {
        match outcome {
            FixtureOutcome::Refused { reason } => {
                println!("  {:<13} REFUSED ({reason}) — no AR_score", fx.label);
                if fx.is_gate {
                    gate_refused.push(fx.label);
                    gate_clear = false;
                }
            }
            FixtureOutcome::Evaluated { ar_score, .. } => {
                let clears = *ar_score >= 0.5;
                println!(
                    "  {:<13} AR_score = {ar_score:.4}  [{}]",
                    fx.label,
                    if clears { "CLEAR" } else { "MISS" }
                );
                if fx.is_gate && !clears {
                    gate_clear = false;
                }
            }
        }
    }
    if !gate_refused.is_empty() {
        println!(
            "\n  gate incomplete: {} refuses — the bar CANNOT be cleared on the full gate set.",
            gate_refused.join(", ")
        );
    } else {
        println!(
            "\n  gate set (sci1-ore, sci2-ore, pu1-plate) complete — Bar 1 on the 3 named gate fixtures: {}",
            if gate_clear { "CLEAR (all three >= 0.5)" } else { "MISS (at least one gate fixture < 0.5)" }
        );
    }

    // Bar 2: net Transit_score across fixtures with fully-attributed
    // transit on both sides.
    println!(
        "\nBar 2 — net Transit_score not negative across fixtures. Formula: \
         net = 1 - (sum Transit(packed)) / (sum Transit(native)), over fixtures where BOTH \
         native and packed have unattributed_edge_count == 0 (a partially-unattributed \
         Transit number is weaker evidence and is excluded from the sum, not silently \
         included):"
    );
    let mut sum_native = 0.0f64;
    let mut sum_packed = 0.0f64;
    let mut included: Vec<&str> = Vec::new();
    let mut excluded: Vec<&str> = Vec::new();
    for (fx, outcome) in &results {
        match outcome {
            FixtureOutcome::Evaluated {
                transit_native,
                transit_packed,
                transit_score,
                unattributed_native,
                unattributed_packed,
                ..
            } => {
                println!(
                    "  {:<13} Transit_score = {transit_score:+.4}  (unattributed edges: native={unattributed_native}, packed={unattributed_packed})",
                    fx.label,
                );
                if *unattributed_native == 0 && *unattributed_packed == 0 {
                    sum_native += transit_native;
                    sum_packed += transit_packed;
                    included.push(fx.label);
                } else {
                    excluded.push(fx.label);
                }
            }
            FixtureOutcome::Refused { .. } => {
                excluded.push(fx.label);
            }
        }
    }
    if sum_native > 0.0 {
        let net = 1.0 - sum_packed / sum_native;
        println!(
            "\n  net Transit_score across {included:?} = {net:+.4}  [{}]",
            if net >= 0.0 { "CLEAR (not negative)" } else { "MISS (negative net)" }
        );
    } else {
        println!("\n  net Transit_score: no fixtures with fully-attributed transit on both sides — cannot compute.");
    }
    if !excluded.is_empty() {
        println!("  excluded from the net (refused, or has unattributed edges on either side): {excluded:?}");
    }

    println!(
        "\nNote: no bar above is tuned or adjusted based on this run's outcome — a miss is a \
         valid, pre-registered result per this unit's brief."
    );
}
