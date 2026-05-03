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

use fucktorio_core::bus::layout;
use fucktorio_core::density;
use fucktorio_core::solver;
use fucktorio_core::trace;
use fucktorio_core::validate::{self, LayoutStyle, Severity};
use rustc_hash::FxHashSet;
use std::collections::BTreeMap;

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
    lr: fucktorio_core::models::LayoutResult,
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
                Err(e) => return Err(Outcome::SolverErr(format!("{e}"))),
            };

            let lr = match layout::build_bus_layout(&sr, layout::LayoutOptions::default()) {
                Ok(lr) => lr,
                Err(e) => return Err(Outcome::LayoutErr(format!("{e}"))),
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
