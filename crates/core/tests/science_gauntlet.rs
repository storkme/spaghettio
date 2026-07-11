//! Science Gauntlet — milestone scoreboard for the 6 Nauvis Space Age
//! science packs.
//!
//! Each case targets one science pack at 1/s, fed from raw Nauvis inputs
//! (ore + crude oil + water + coal + stone), with the assembler tier
//! roughly matched to recipe depth (AM1 for early packs, AM3 for late).
//! Belt tier is auto-picked from rate.
//!
//! This is a *measurement* test, not a regression gate — it never fails on
//! validation warnings or errors, only on panics or solver/layout pipeline
//! crashes. The scoreboard is the value. Promote individual packs to
//! `e2e.rs` regression tests once they stabilize.
//!
//! Run with:
//!   cargo test --manifest-path crates/core/Cargo.toml --test science_gauntlet \
//!       -- --ignored --nocapture

use spaghettio_core::bus::layout;
use spaghettio_core::density;
use spaghettio_core::solver;
use spaghettio_core::trace;
use spaghettio_core::validate::{self, LayoutStyle, Severity};
use spaghettio_core::zone_cache;
use rustc_hash::FxHashSet;
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

struct Case {
    pack: &'static str,
    rate: f64,
    machine: &'static str,
    extra_inputs: &'static [&'static str],
}

#[derive(Debug)]
enum Outcome {
    Pass {
        entities: usize,
        w: i32,
        h: i32,
        density: f64,
    },
    Warn {
        entities: usize,
        w: i32,
        h: i32,
        density: f64,
        warnings: usize,
        by_cat: Vec<(String, usize)>,
    },
    Fail {
        entities: usize,
        w: i32,
        h: i32,
        density: f64,
        errors: usize,
        warnings: usize,
        err_by_cat: Vec<(String, usize)>,
        warn_by_cat: Vec<(String, usize)>,
    },
    SolverErr(String),
    LayoutErr(String),
    Panic(String),
}

struct SolverLayoutOk {
    lr: spaghettio_core::models::LayoutResult,
}

const NAUVIS_INPUTS: &[&str] = &[
    "iron-ore",
    "copper-ore",
    "coal",
    "stone",
    "crude-oil",
    "water",
];

fn run_case(case: &Case) -> Outcome {
    let mut inputs: FxHashSet<String> = NAUVIS_INPUTS.iter().map(|s| s.to_string()).collect();
    for extra in case.extra_inputs {
        inputs.insert((*extra).to_string());
    }

    // Trace guard so any internal trace::record calls during layout don't
    // hit a missing thread-local. We don't read the events back.
    // Layout internals can panic on unsupported geometries (e.g. staggered
    // multi-output template assertions). Catch so one bad case doesn't take
    // the whole gauntlet down. AssertUnwindSafe is fine here — we discard
    // any state we touched on the panic path.
    use std::panic::{self, AssertUnwindSafe};

    let pack = case.pack;
    let rate = case.rate;
    let machine = case.machine;
    let inputs_ref = &inputs;

    // Silence the default panic stderr ("thread 'science_gauntlet' panicked
    // at ..." + backtrace) for the duration of this case — we surface the
    // payload ourselves below.
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result: Result<Result<(SolverLayoutOk, Vec<validate::ValidationIssue>), Outcome>, _> =
        panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = trace::start_trace();

            let sr = match solver::solve(pack, rate, inputs_ref, machine) {
                Ok(sr) => sr,
                Err(e) => return Err(Outcome::SolverErr(e.to_string())),
            };

            let lr = match layout::build_bus_layout(&sr, layout::LayoutOptions::default()) {
                Ok(lr) => lr,
                Err(e) => return Err(Outcome::LayoutErr(e.to_string())),
            };

            let issues = match validate::validate(&lr, Some(&sr), LayoutStyle::Bus) {
                Ok(i) => i,
                Err(e) => e.issues,
            };
            Ok((SolverLayoutOk { lr }, issues))
        }));
    panic::set_hook(prev_hook);

    let (ok, issues) = match result {
        Ok(Ok(v)) => v,
        Ok(Err(outcome)) => return outcome,
        Err(panic_payload) => {
            let msg = panic_payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "(non-string panic payload)".to_string());
            return Outcome::Panic(msg);
        }
    };
    let lr = ok.lr;

    let mut err_by_cat: BTreeMap<String, usize> = BTreeMap::new();
    let mut warn_by_cat: BTreeMap<String, usize> = BTreeMap::new();
    for i in &issues {
        match i.severity {
            Severity::Error => *err_by_cat.entry(i.category.clone()).or_default() += 1,
            Severity::Warning => *warn_by_cat.entry(i.category.clone()).or_default() += 1,
        }
    }
    let errors: usize = err_by_cat.values().sum();
    let warnings: usize = warn_by_cat.values().sum();

    let entities = lr.entities.len();
    let w = lr.width;
    let h = lr.height;
    let density = density::score_density(&lr, (1, 1)).density;

    if errors > 0 {
        Outcome::Fail {
            entities,
            w,
            h,
            density,
            errors,
            warnings,
            err_by_cat: err_by_cat.into_iter().collect(),
            warn_by_cat: warn_by_cat.into_iter().collect(),
        }
    } else if warnings > 0 {
        Outcome::Warn {
            entities,
            w,
            h,
            density,
            warnings,
            by_cat: warn_by_cat.into_iter().collect(),
        }
    } else {
        Outcome::Pass {
            entities,
            w,
            h,
            density,
        }
    }
}

#[test]
#[ignore = "milestone scoreboard — run with --ignored --nocapture"]
fn science_gauntlet() {
    // Assembler tier roughly matches recipe depth.
    // AM1: simple, no fluids needed.
    // AM2: handles fluid recipes; intermediate complexity.
    // AM3: deepest chains (production / utility).
    let cases: &[Case] = &[
        Case {
            pack: "automation-science-pack",
            rate: 1.0,
            machine: "assembling-machine-1",
            extra_inputs: &[],
        },
        // Bumped to AM2: AM1 has 2 ingredient slots and inserter has 3
        // ingredients in Space Age (iron-plate + iron-gear-wheel + EC),
        // so AM1 cannot craft the chain.
        Case {
            pack: "logistic-science-pack",
            rate: 1.0,
            machine: "assembling-machine-2",
            extra_inputs: &[],
        },
        Case {
            pack: "military-science-pack",
            rate: 1.0,
            machine: "assembling-machine-2",
            extra_inputs: &[],
        },
        Case {
            pack: "chemical-science-pack",
            rate: 1.0,
            machine: "assembling-machine-2",
            extra_inputs: &[],
        },
        Case {
            pack: "production-science-pack",
            rate: 1.0,
            machine: "assembling-machine-3",
            extra_inputs: &[],
        },
        Case {
            pack: "utility-science-pack",
            rate: 1.0,
            machine: "assembling-machine-3",
            extra_inputs: &[],
        },
    ];

    // Reset hit-rate counters so we only measure this run's workload.
    zone_cache::reset_cache_stats();

    eprintln!();
    eprintln!(
        "Science Gauntlet — Nauvis recipes, 1/s, auto belt tier, ore + crude/water/coal/stone inputs"
    );
    let sep: String = "-".repeat(102);
    eprintln!("{sep}");
    eprintln!(
        "  {:<26} {:<24} {:>9} {:>10} {:>7} {:>11}",
        "pack", "machine", "size", "entities", "dens%", "result"
    );
    eprintln!("{sep}");

    let mut summary = Summary::default();
    for case in cases {
        let outcome = run_case(case);
        report(case, &outcome, &mut summary);
        // Flush newly-solved SAT zones after each case so they accumulate in
        // the runtime cache file even if a later case panics.
        zone_cache::flush();
    }
    eprintln!("{sep}");
    eprintln!(
        "  total: {} pass, {} warn, {} fail, {} unsolved (of {})",
        summary.pass,
        summary.warn,
        summary.fail,
        summary.unsolved,
        summary.pass + summary.warn + summary.fail + summary.unsolved,
    );

    print_cache_footer();
}

// -----------------------------------------------------------------------
// SAT zone-cache hit-rate watchdog — shared by both gauntlets.
// -----------------------------------------------------------------------
fn print_cache_footer() {
    // cache_stats_extended returns (total, hits_all, hits_unsat, hits_timeout, misses).
    // hits_all already includes unsat+timeout; compute solved as the remainder.
    let (total_lookups, hits_all, hits_unsat, hits_timeout, misses) =
        zone_cache::cache_stats_extended();
    let hits = hits_all; // already the total hit count
    let hits_solved = hits_all.saturating_sub(hits_unsat + hits_timeout);
    if total_lookups > 0 {
        let hit_pct = hits as f64 / total_lookups as f64 * 100.0;
        eprintln!(
            "cache hit rate: {:.1}% ({} hits / {} lookups, {} misses) \
             [solved={} unsat={} timeout={}]",
            hit_pct, hits, total_lookups, misses, hits_solved, hits_unsat, hits_timeout,
        );
        // Only warn when the sample is large enough to be meaningful.
        if total_lookups > 1000 && hit_pct < 60.0 {
            eprintln!(
                "WARN: solve cache hit rate below 60% — corpus may need refresh. \
                 Run: rm -f ~/.cache/spaghettio/sat-zones.bin && \
                 cargo test --manifest-path crates/core/Cargo.toml && \
                 cargo test --manifest-path crates/core/Cargo.toml \
                 --test science_gauntlet -- --ignored --nocapture && \
                 cargo run --manifest-path crates/core/Cargo.toml \
                 --example promote_runtime_cache"
            );
        }
    } else {
        eprintln!("cache hit rate: n/a (0 lookups)");
    }
}

#[derive(Default)]
struct Summary {
    pass: usize,
    warn: usize,
    fail: usize,
    unsolved: usize,
}

fn report(case: &Case, outcome: &Outcome, summary: &mut Summary) {
    match outcome {
        Outcome::Pass {
            entities,
            w,
            h,
            density,
        } => {
            eprintln!(
                "  {:<26} {:<24} {:>4}x{:<4} {:>10} {:>6.1}% {:>11}",
                case.pack,
                case.machine,
                w,
                h,
                entities,
                density * 100.0,
                "PASS",
            );
            summary.pass += 1;
        }
        Outcome::Warn {
            entities,
            w,
            h,
            density,
            warnings,
            by_cat,
        } => {
            eprintln!(
                "  {:<26} {:<24} {:>4}x{:<4} {:>10} {:>6.1}% {:>11}",
                case.pack,
                case.machine,
                w,
                h,
                entities,
                density * 100.0,
                format!("WARN×{warnings}"),
            );
            for (cat, n) in by_cat {
                eprintln!("      warn: {cat} × {n}");
            }
            summary.warn += 1;
        }
        Outcome::Fail {
            entities,
            w,
            h,
            density,
            errors,
            warnings,
            err_by_cat,
            warn_by_cat,
        } => {
            eprintln!(
                "  {:<26} {:<24} {:>4}x{:<4} {:>10} {:>6.1}% {:>11}",
                case.pack,
                case.machine,
                w,
                h,
                entities,
                density * 100.0,
                format!("FAIL×{errors}"),
            );
            for (cat, n) in err_by_cat {
                eprintln!("      err:  {cat} × {n}");
            }
            if *warnings > 0 {
                for (cat, n) in warn_by_cat {
                    eprintln!("      warn: {cat} × {n}");
                }
            }
            summary.fail += 1;
        }
        Outcome::SolverErr(e) => {
            eprintln!(
                "  {:<26} {:<24} {:>9} {:>10} {:>7} {:>11}",
                case.pack, case.machine, "—", "—", "—", "UNSOLVED",
            );
            eprintln!("      solver: {e}");
            summary.unsolved += 1;
        }
        Outcome::LayoutErr(e) => {
            eprintln!(
                "  {:<26} {:<24} {:>9} {:>10} {:>7} {:>11}",
                case.pack, case.machine, "—", "—", "—", "UNSOLVED",
            );
            eprintln!("      layout: {e}");
            summary.unsolved += 1;
        }
        Outcome::Panic(msg) => {
            eprintln!(
                "  {:<26} {:<24} {:>9} {:>10} {:>7} {:>11}",
                case.pack, case.machine, "—", "—", "—", "PANIC",
            );
            for line in msg.lines() {
                eprintln!("      panic: {line}");
            }
            summary.fail += 1;
        }
    }
}

// =============================================================================
// Science Scaling Gauntlet — rate-scaling scoreboard for the 6 Nauvis packs.
// =============================================================================
//
// Same packs/machine tiers as `science_gauntlet`, but swept across
// 1, 2, 5, 10 /s to find where each pack's layout stops being clean.
// Reuses `Case`, `run_case`, `Outcome`, `Summary`/`report`-style formatting,
// and the zone_cache flush/footer machinery from the 1/s gauntlet above.
//
// Each cell gets a wall-clock budget (`SCALING_CELL_BUDGET`) and runs on a
// dedicated worker thread with a generous stack (deep recursive solving /
// routing on large recipe graphs can blow the default thread stack, and a
// stack overflow aborts the whole process — no catch_unwind can save us from
// that). If a cell doesn't report back within the budget we record TIMEOUT
// and move on, *deliberately leaking the worker thread* rather than trying
// to kill it — this is an ignored scoreboard, correctness of the table
// matters more than tidiness here.
//
// Run with (release recommended — USP@1/s is already ~6.6k entities in
// debug and 10/s can be an order of magnitude bigger):
//   cargo test --manifest-path crates/core/Cargo.toml --test science_gauntlet \
//       --release -- --ignored --nocapture science_scaling_gauntlet

const SCALING_RATES: &[f64] = &[1.0, 2.0, 5.0, 10.0];
const SCALING_CELL_BUDGET: Duration = Duration::from_secs(180);
const WORKER_STACK_BYTES: usize = 256 * 1024 * 1024;

struct PackTemplate {
    pack: &'static str,
    machine: &'static str,
}

const PACK_TEMPLATES: &[PackTemplate] = &[
    PackTemplate {
        pack: "automation-science-pack",
        machine: "assembling-machine-1",
    },
    PackTemplate {
        pack: "logistic-science-pack",
        machine: "assembling-machine-2",
    },
    PackTemplate {
        pack: "military-science-pack",
        machine: "assembling-machine-2",
    },
    PackTemplate {
        pack: "chemical-science-pack",
        machine: "assembling-machine-2",
    },
    PackTemplate {
        pack: "production-science-pack",
        machine: "assembling-machine-3",
    },
    PackTemplate {
        pack: "utility-science-pack",
        machine: "assembling-machine-3",
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Pass,
    Warn,
    Fail,
    SolverErr,
    LayoutErr,
    Panic,
    Timeout,
}

struct CellRecord {
    pack: &'static str,
    rate: f64,
    kind: CellKind,
    label: String,
    dims: Option<(i32, i32)>,
    entities: Option<usize>,
    wall_secs: f64,
    err_by_cat: Vec<(String, usize)>,
    warn_by_cat: Vec<(String, usize)>,
    /// First line of a solver/layout error or panic message, for
    /// spectacular-failure callouts. Not populated for Pass/Warn/Fail.
    detail: Option<String>,
}

enum CellOutcome {
    Ran(Outcome),
    Timeout,
}

/// Run one case on a dedicated worker thread with a wall-clock budget.
/// `run_case` already wraps the solve/layout/validate pipeline in its own
/// `catch_unwind`, so the only failure mode this adds is the cell simply
/// taking too long (or the worker dying without going through that
/// catch_unwind at all, e.g. a stack overflow abort).
fn run_case_with_budget(case: &Case, budget: Duration) -> (CellOutcome, Duration) {
    // Case is just 'static refs + an f64 — rebuild by value so the worker
    // closure can be 'static without needing Case: Clone.
    let case_owned = Case {
        pack: case.pack,
        rate: case.rate,
        machine: case.machine,
        extra_inputs: case.extra_inputs,
    };
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let handle = thread::Builder::new()
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            let outcome = run_case(&case_owned);
            // Ignore send failure: it only happens if the receiver already
            // timed out and walked away, which is exactly the case we're
            // deliberately tolerating here.
            let _ = tx.send(outcome);
        })
        .expect("failed to spawn scoreboard worker thread");

    match rx.recv_timeout(budget) {
        Ok(outcome) => {
            let _ = handle.join();
            (CellOutcome::Ran(outcome), start.elapsed())
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Deliberately don't join or abort `handle` — let it run to
            // completion (or forever) in the background. See module doc.
            (CellOutcome::Timeout, start.elapsed())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Sender dropped without sending — the worker died without
            // going through run_case's own panic handling (e.g. a stack
            // overflow, which aborts before any Rust-level catch runs).
            // Surface it as a panic outcome so it still shows up as a fix
            // target instead of silently vanishing from the table.
            (
                CellOutcome::Ran(Outcome::Panic(
                    "worker thread died without reporting (stack overflow / abort?)".to_string(),
                )),
                start.elapsed(),
            )
        }
    }
}

fn build_cell_record(pack: &'static str, rate: f64, outcome: CellOutcome, wall: Duration) -> CellRecord {
    let wall_secs = wall.as_secs_f64();
    match outcome {
        CellOutcome::Timeout => CellRecord {
            pack,
            rate,
            kind: CellKind::Timeout,
            label: "TIMEOUT".to_string(),
            dims: None,
            entities: None,
            wall_secs,
            err_by_cat: Vec::new(),
            warn_by_cat: Vec::new(),
            detail: None,
        },
        CellOutcome::Ran(Outcome::Pass {
            entities, w, h, ..
        }) => CellRecord {
            pack,
            rate,
            kind: CellKind::Pass,
            label: "PASS".to_string(),
            dims: Some((w, h)),
            entities: Some(entities),
            wall_secs,
            err_by_cat: Vec::new(),
            warn_by_cat: Vec::new(),
            detail: None,
        },
        CellOutcome::Ran(Outcome::Warn {
            entities,
            w,
            h,
            warnings,
            by_cat,
            ..
        }) => CellRecord {
            pack,
            rate,
            kind: CellKind::Warn,
            label: format!("WARN×{warnings}"),
            dims: Some((w, h)),
            entities: Some(entities),
            wall_secs,
            err_by_cat: Vec::new(),
            warn_by_cat: by_cat,
            detail: None,
        },
        CellOutcome::Ran(Outcome::Fail {
            entities,
            w,
            h,
            errors,
            err_by_cat,
            warn_by_cat,
            ..
        }) => CellRecord {
            pack,
            rate,
            kind: CellKind::Fail,
            label: format!("FAIL×{errors}"),
            dims: Some((w, h)),
            entities: Some(entities),
            wall_secs,
            err_by_cat,
            warn_by_cat,
            detail: None,
        },
        CellOutcome::Ran(Outcome::SolverErr(e)) => CellRecord {
            pack,
            rate,
            kind: CellKind::SolverErr,
            label: "SOLVER-ERR".to_string(),
            dims: None,
            entities: None,
            wall_secs,
            err_by_cat: Vec::new(),
            warn_by_cat: Vec::new(),
            detail: Some(e),
        },
        CellOutcome::Ran(Outcome::LayoutErr(e)) => CellRecord {
            pack,
            rate,
            kind: CellKind::LayoutErr,
            label: "LAYOUT-ERR".to_string(),
            dims: None,
            entities: None,
            wall_secs,
            err_by_cat: Vec::new(),
            warn_by_cat: Vec::new(),
            detail: Some(e),
        },
        CellOutcome::Ran(Outcome::Panic(msg)) => {
            let first_line = msg.lines().next().unwrap_or("").to_string();
            CellRecord {
                pack,
                rate,
                kind: CellKind::Panic,
                label: "PANIC".to_string(),
                dims: None,
                entities: None,
                wall_secs,
                err_by_cat: Vec::new(),
                warn_by_cat: Vec::new(),
                detail: Some(first_line),
            }
        }
    }
}

#[test]
#[ignore = "milestone scoreboard — run with --ignored --nocapture"]
fn science_scaling_gauntlet() {
    zone_cache::reset_cache_stats();

    eprintln!();
    eprintln!(
        "Science Scaling Gauntlet — Nauvis recipes × {{1,2,5,10}}/s, auto belt tier, ore + crude/water/coal/stone inputs"
    );
    eprintln!(
        "per-cell wall-clock budget: {}s on a dedicated worker thread; exceeding it is recorded as TIMEOUT and the sweep continues",
        SCALING_CELL_BUDGET.as_secs()
    );
    let sep: String = "-".repeat(102);
    eprintln!("{sep}");
    eprintln!(
        "  {:<26} {:>6} {:>10} {:>10} {:>9} {:>11}",
        "pack", "rate", "size", "entities", "wall(s)", "result"
    );
    eprintln!("{sep}");

    let mut records: Vec<CellRecord> = Vec::with_capacity(PACK_TEMPLATES.len() * SCALING_RATES.len());

    for tmpl in PACK_TEMPLATES {
        for &rate in SCALING_RATES {
            let case = Case {
                pack: tmpl.pack,
                rate,
                machine: tmpl.machine,
                extra_inputs: &[],
            };
            let (outcome, wall) = run_case_with_budget(&case, SCALING_CELL_BUDGET);
            let rec = build_cell_record(tmpl.pack, rate, outcome, wall);

            eprintln!(
                "  {:<26} {:>5.0}/s {:>10} {:>10} {:>8.1}s {:>11}",
                rec.pack,
                rec.rate,
                rec.dims
                    .map(|(w, h)| format!("{w}x{h}"))
                    .unwrap_or_else(|| "—".to_string()),
                rec.entities
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "—".to_string()),
                rec.wall_secs,
                rec.label,
            );
            for (cat, n) in &rec.err_by_cat {
                eprintln!("      err:  {cat} × {n}");
            }
            for (cat, n) in &rec.warn_by_cat {
                eprintln!("      warn: {cat} × {n}");
            }
            if let Some(d) = &rec.detail {
                eprintln!("      note: {d}");
            }

            records.push(rec);
            // Flush newly-solved SAT zones after each case so they accumulate
            // in the runtime cache file even if a later case panics/times out.
            zone_cache::flush();
        }
    }
    eprintln!("{sep}");

    print_scaling_matrix(&records);
    print_category_tally(&records);
    print_walls(&records);
    print_cache_footer();
}

/// 1. Per-cell matrix: rows = pack, columns = rate.
fn print_scaling_matrix(records: &[CellRecord]) {
    const COL_W: usize = 12;
    eprintln!();
    eprintln!("=== 1. Result matrix (rows = pack, columns = rate) ===");
    eprint!("  {:<26}", "pack");
    for &rate in SCALING_RATES {
        eprint!("{:>COL_W$}", format!("{rate:.0}/s"));
    }
    eprintln!();
    for tmpl in PACK_TEMPLATES {
        eprint!("  {:<26}", tmpl.pack);
        for &rate in SCALING_RATES {
            let rec = records
                .iter()
                .find(|r| r.pack == tmpl.pack && (r.rate - rate).abs() < f64::EPSILON)
                .expect("every pack×rate cell was run");
            eprint!("{:>COL_W$}", rec.label);
        }
        eprintln!();
    }
}

/// Category tally across the whole matrix (section 2 of the report), sorted
/// by frequency, split by severity. Only validated cells (Pass/Warn/Fail)
/// contribute categories; non-validated outcomes (solver/layout errors,
/// panics, timeouts) are tallied separately by kind since they never reach
/// the validator.
fn print_category_tally(records: &[CellRecord]) {
    eprintln!();
    eprintln!("=== 2. Category tally (sorted by frequency, split by severity) ===");

    let mut tally: BTreeMap<(String, &'static str), usize> = BTreeMap::new();
    for rec in records {
        for (cat, n) in &rec.err_by_cat {
            *tally.entry((cat.clone(), "error")).or_default() += n;
        }
        for (cat, n) in &rec.warn_by_cat {
            *tally.entry((cat.clone(), "warning")).or_default() += n;
        }
    }
    let mut rows: Vec<((String, &str), usize)> = tally.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    if rows.is_empty() {
        eprintln!("  (no validation issues recorded)");
    } else {
        eprintln!("  {:<8} {:<32} {:>6}", "sev", "category", "count");
        for ((cat, sev), n) in &rows {
            eprintln!("  {sev:<8} {cat:<32} {n:>6}");
        }
    }

    let mut kind_tally: BTreeMap<&'static str, usize> = BTreeMap::new();
    for rec in records {
        let kind = match rec.kind {
            CellKind::SolverErr => Some("SOLVER-ERR"),
            CellKind::LayoutErr => Some("LAYOUT-ERR"),
            CellKind::Panic => Some("PANIC"),
            CellKind::Timeout => Some("TIMEOUT"),
            CellKind::Pass | CellKind::Warn | CellKind::Fail => None,
        };
        if let Some(kind) = kind {
            *kind_tally.entry(kind).or_default() += 1;
        }
    }
    if !kind_tally.is_empty() {
        eprintln!();
        eprintln!("  non-validation outcomes (no issue category — pipeline never reached the validator):");
        let mut kinds: Vec<(&str, usize)> = kind_tally.into_iter().collect();
        kinds.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        for (kind, n) in kinds {
            eprintln!("    {kind:<12} × {n}");
        }
    }
}

/// 3. Walls: for each pack, the first rate at which it stops being PASS.
fn print_walls(records: &[CellRecord]) {
    eprintln!();
    eprintln!("=== 3. Walls — first rate per pack that stops being PASS ===");
    for tmpl in PACK_TEMPLATES {
        let mut cells: Vec<&CellRecord> = records.iter().filter(|r| r.pack == tmpl.pack).collect();
        cells.sort_by(|a, b| a.rate.partial_cmp(&b.rate).unwrap());

        match cells.iter().find(|r| r.kind != CellKind::Pass) {
            Some(rec) => {
                let mut parts: Vec<String> = rec
                    .err_by_cat
                    .iter()
                    .chain(rec.warn_by_cat.iter())
                    .map(|(c, n)| format!("{c}×{n}"))
                    .collect();
                if parts.is_empty() {
                    if let Some(d) = &rec.detail {
                        parts.push(d.clone());
                    }
                }
                let suffix = if parts.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", parts.join(", "))
                };
                eprintln!(
                    "  {:<26} wall at {:>4.0}/s: {}{}",
                    tmpl.pack, rec.rate, rec.label, suffix
                );
            }
            None => {
                let last = SCALING_RATES.last().copied().unwrap_or(0.0);
                eprintln!(
                    "  {:<26} no wall within tested rates (PASS through {:.0}/s)",
                    tmpl.pack, last
                );
            }
        }
    }
}
