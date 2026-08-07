//! Calibrate the fast meter against sim baselines measured on **post-lift**
//! layouts — the population a meter-backed gate would actually run on.
//!
//! Sibling of [`sweep_corpus`], and deliberately a separate driver rather than
//! a flag on it, because the two answer different questions:
//!
//! - `sweep_corpus` sweeps the Job-2 bank (`<fixture>/<variant>/…` plus
//!   `sim/<fixture>__<variant>/report.json`), frozen 2026-08-01. Every layout
//!   in it was selected *before* #605 lifted the `input-rate-delivery`
//!   exemption, so it characterises the meter on layouts the engine no longer
//!   picks.
//! - this driver sweeps a flat `<fixture>/{bp.txt,manifest-real.json,
//!   report.json}` dir of layouts selected on current `main`.
//!
//! That distinction is the whole point. `meter-divergence.md` established the
//! meter as a safe **floor** on the corpus — "meter says below plan ⇒ believe
//! it", with the dangerous quadrant empty at every tolerance ≥90%. That result
//! is a property of the corpus population and does **not** transfer to
//! post-lift layouts, which is what this driver measures.
//!
//! Usage:
//!   cargo run --release --manifest-path crates/meter/Cargo.toml \
//!     --example sweep_postlift -- <dir> [out.csv]
//!
//! A fixture whose report carries `kit_errors` is **reported and excluded**,
//! never silently dropped: a non-empty kit means the sim's boundary was
//! compromised and its rates are not comparable against any plan
//! (`docs/sim-harness.md`). Dropping such a row quietly is how a sweep comes
//! to claim coverage it does not have.

use spaghettio_meter::factory::Factory;
use spaghettio_meter::manifest::Manifest;

/// Thresholds at which each fixture is classified at-plan vs below-plan, for
/// both instruments. These are the gate tolerances `meter-divergence.md`
/// checked the corpus at, kept identical so the two results are comparable.
const THRESHOLDS: [f64; 4] = [90.0, 95.0, 98.0, 99.0];

struct Row {
    fixture: String,
    item: String,
    is_target: bool,
    planned: f64,
    meter: Option<f64>,
    sim: Option<f64>,
}

impl Row {
    fn meter_pct(&self) -> Option<f64> {
        self.meter.map(|m| m / self.planned * 100.0)
    }
    fn sim_pct(&self) -> Option<f64> {
        self.sim.map(|s| s / self.planned * 100.0)
    }
    fn delta_pp(&self) -> Option<f64> {
        Some(self.meter_pct()? - self.sim_pct()?)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args.get(1).cloned().unwrap_or_else(|| {
        eprintln!("usage: sweep_postlift <dir> [out.csv]");
        std::process::exit(2);
    });
    let out = args.get(2).cloned();

    let mut rows: Vec<Row> = Vec::new();
    let mut excluded: Vec<(String, String)> = Vec::new();

    let mut fixtures: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {dir}: {e}"))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    fixtures.sort();

    for fp in fixtures {
        let fixture = fp.file_name().unwrap().to_string_lossy().to_string();
        let bp_path = fp.join("bp.txt");
        let mf_path = fp.join("manifest-real.json");
        let rp_path = fp.join("report.json");
        if !bp_path.exists() || !mf_path.exists() {
            excluded.push((fixture, "no bp.txt / manifest-real.json".into()));
            continue;
        }
        if !rp_path.exists() {
            excluded.push((fixture, "no sim report.json — nothing to compare".into()));
            continue;
        }

        let report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&rp_path).unwrap()).unwrap();
        let rep = &report["report"];

        // Provenance gates, in the order that makes a bad row cheapest to
        // reject. `kit_errors` first: it invalidates the run outright.
        let kit = rep["kit_errors"].as_array().cloned().unwrap_or_default();
        if !kit.is_empty() {
            let first = kit[0].as_str().unwrap_or("").to_string();
            excluded.push((fixture, format!("kit_errors ({}): {first}", kit.len())));
            continue;
        }
        if rep["converged"].as_bool() != Some(true) {
            excluded.push((fixture, "sim run did not converge".into()));
            continue;
        }

        let bp = std::fs::read_to_string(&bp_path).unwrap();
        let manifest = match Manifest::from_path(&mf_path) {
            Ok(m) => m,
            Err(e) => {
                excluded.push((fixture, format!("manifest: {e}")));
                continue;
            }
        };
        let mut factory = match Factory::build(&bp, manifest) {
            Ok(f) => f,
            Err(e) => {
                excluded.push((fixture, format!("meter build: {e}")));
                continue;
            }
        };
        // Same window both drivers use. `meter-divergence.md` measured the
        // meter's own convergence floor at ~20-40k ticks on the deepest corpus
        // fixture, so 108k carries a 3-5x margin; it is NOT the sim's warmup
        // and does not inherit the "default warmup is too short" caveat, which
        // is a property of headless Factorio rather than of this simulator.
        let meter = factory.measure(108_000, 216_000);

        for item in rep["items"].as_array().cloned().unwrap_or_default() {
            let name = item["item"].as_str().unwrap_or("").to_string();
            let Some(planned) = item["planned_rate"].as_f64() else {
                continue;
            };
            if planned == 0.0 {
                continue;
            }
            rows.push(Row {
                fixture: fixture.clone(),
                item: name.clone(),
                is_target: item["is_target"].as_bool().unwrap_or(false),
                planned,
                meter: meter.produced_per_s.get(&name).copied(),
                sim: item["measured_produced_rate"].as_f64(),
            });
        }
    }

    // --- per-item table ---------------------------------------------------
    println!(
        "{:<30}{:<22}{:>10}{:>10}{:>10}{:>9}{:>9}{:>9}",
        "fixture", "item", "plan", "meter", "sim", "meter%", "sim%", "Dpp"
    );
    println!("{}", "-".repeat(109));
    let f = |v: Option<f64>| v.map_or("n/a".into(), |x| format!("{x:.4}"));
    let p = |v: Option<f64>| v.map_or("n/a".into(), |x| format!("{x:.2}"));
    for r in &rows {
        println!(
            "{:<30}{:<22}{:>10.4}{:>10}{:>10}{:>9}{:>9}{:>9}{}",
            r.fixture,
            r.item,
            r.planned,
            f(r.meter),
            f(r.sim),
            p(r.meter_pct()),
            p(r.sim_pct()),
            p(r.delta_pp()),
            if r.is_target { " *" } else { "" },
        );
    }

    // --- target-only summary ---------------------------------------------
    let targets: Vec<&Row> = rows
        .iter()
        .filter(|r| r.is_target && r.delta_pp().is_some())
        .collect();
    println!("\n=== TARGET ITEMS ({} compared) ===", targets.len());
    for r in &targets {
        println!(
            "{:<30}{:<22}{:>9.2}{:>9.2}{:>+9.2}",
            r.fixture,
            r.item,
            r.meter_pct().unwrap(),
            r.sim_pct().unwrap(),
            r.delta_pp().unwrap()
        );
    }
    if !targets.is_empty() {
        let d: Vec<f64> = targets.iter().map(|r| r.delta_pp().unwrap()).collect();
        let mean = d.iter().sum::<f64>() / d.len() as f64;
        let worst_opt = d.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let worst_pess = d.iter().cloned().fold(f64::INFINITY, f64::min);
        println!(
            "\nmean {mean:+.2}pp | worst optimistic {worst_opt:+.2}pp | worst pessimistic {worst_pess:+.2}pp"
        );
    }

    // --- the gate question ------------------------------------------------
    //
    // Two quadrants, and they are not the same risk:
    //   MISSED DEFECT    meter says at-plan, sim says below  -> report-only
    //                    is worth less than claimed (it stays silent on a
    //                    real deficit).
    //   FALSE ACCUSATION meter says below-plan, sim says at   -> a BLOCKING
    //                    gate rejects a good layout.
    println!("\n=== GATE CLASSIFICATION (target items) ===");
    println!(
        "{:>10}{:>18}{:>20}   offenders",
        "threshold", "missed defects", "false accusations"
    );
    for t in THRESHOLDS {
        let mut missed = Vec::new();
        let mut false_acc = Vec::new();
        for r in &targets {
            let (m, s) = (r.meter_pct().unwrap(), r.sim_pct().unwrap());
            if m >= t && s < t {
                missed.push(format!("MISS:{}", r.fixture));
            }
            if m < t && s >= t {
                false_acc.push(format!("FALSE:{}", r.fixture));
            }
        }
        let mut names = missed.clone();
        names.extend(false_acc.clone());
        println!(
            "{:>9.0}%{:>18}{:>20}   {}",
            t,
            format!("{}/{}", missed.len(), targets.len()),
            format!("{}/{}", false_acc.len(), targets.len()),
            names.join(" ")
        );
    }

    if !excluded.is_empty() {
        println!("\n=== EXCLUDED ({}) ===", excluded.len());
        for (fx, why) in &excluded {
            println!("  {fx:<30} {why}");
        }
    }

    if let Some(out) = out {
        let mut csv =
            String::from("fixture,item,is_target,planned,meter,sim,meter_pct,sim_pct,delta_pp\n");
        for r in &rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                r.fixture,
                r.item,
                r.is_target,
                r.planned,
                f(r.meter),
                f(r.sim),
                p(r.meter_pct()),
                p(r.sim_pct()),
                p(r.delta_pp()),
            ));
        }
        std::fs::write(&out, csv).unwrap();
        println!("\nwrote {out}");
    }
}
