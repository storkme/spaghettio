//! Companion to the RFC-064 sim sweep: run the fast meter (RFC-054) over the
//! Job-2 corpus layouts and compare each meter `produced_per_s` to the measured
//! headless-Factorio sim value. Calibration of "how close is the meter" —
//! the whole point of the meter's KC1 (±10pp).
//!
//! Usage:
//!   cargo run --release --manifest-path crates/meter/Cargo.toml \
//!     --example sweep_corpus -- <corpus-date-dir> <out.csv>
//! The corpus dir is the dated Job-2 dir (e.g. ~/spaghettio-corpora/job2-sim-baselines/2026-08-01),
//! holding <fixture>/<variant>/{bp.txt,manifest-real.json} and sim/<fixture>__<variant>/report.json.

use spaghettio_meter::factory::Factory;
use spaghettio_meter::manifest::Manifest;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: sweep_corpus <corpus-date-dir> <out.csv>");
        std::process::exit(2);
    }
    let corpus = &args[1];
    let out = &args[2];

    let mut csv = String::from("fixture,variant,item,planned,meter_produced,meter_delivered,sim_produced,sim_delivered,meter_vs_sim_pp\n");
    let mut rows = 0usize;
    let mut compared = 0usize;

    let entries = std::fs::read_dir(corpus).unwrap();
    for e in entries.flatten() {
        let fp = e.path();
        if !fp.is_dir() {
            continue;
        }
        let fixture = fp.file_name().unwrap().to_string_lossy().to_string();
        for variant in ["native", "compact"] {
            let dir = fp.join(variant);
            let bp_path = dir.join("bp.txt");
            let mf_path = dir.join("manifest-real.json");
            if !bp_path.exists() || !mf_path.exists() {
                continue;
            }
            let bp = std::fs::read_to_string(&bp_path).unwrap();
            let manifest = match Manifest::from_path(&mf_path) {
                Ok(m) => m,
                Err(e) => {
                    csv.push_str(&format!("{fixture},{variant},,,,ERR:manifest {e},,,,\n"));
                    continue;
                }
            };
            let mut factory = match Factory::build(&bp, manifest.clone()) {
                Ok(f) => f,
                Err(e) => {
                    csv.push_str(&format!("{fixture},{variant},,,,ERR:build {e},,,,\n"));
                    continue;
                }
            };
            // generous steady-state window (meter is native/fast)
            let rep = factory.measure(108_000, 216_000);
            rows += 1;

            // sim measurement from the corpus report.json
            let sim_report = format!("{corpus}/sim/{fixture}__{variant}/report.json");
            let (sim_prod, sim_del) = sim_target(&sim_report);

            // target item = the manifest's declared target (NOT the first
            // planned item — planned_per_s is alphabetically sorted, so for a
            // multi-item layout that would wrongly pick e.g. copper-cable).
            let target = manifest.targets.first().map(|t| t.item.clone());
            let Some(target) = target else { continue };
            let planned = rep.planned_per_s.get(&target).copied().unwrap_or(0.0);
            let m_prod = rep.produced_per_s.get(&target).copied().unwrap_or(0.0);
            let m_del = rep.delivered_per_s.get(&target).copied().unwrap_or(0.0);
            let delta = if sim_prod.is_some() && sim_prod.unwrap() > 0.0 {
                compared += 1;
                (m_prod - sim_prod.unwrap()) / sim_prod.unwrap() * 100.0
            } else {
                f64::NAN
            };
            let sp = sim_prod.map(|v| format!("{v:.3}")).unwrap_or_else(|| "NA".into());
            let sd = sim_del.map(|v| format!("{v:.3}")).unwrap_or_else(|| "NA".into());
            csv.push_str(&format!(
                "{fixture},{variant},{target},{planned:.3},{m_prod:.3},{m_del:.3},{sp},{sd},{delta:.1}\n"
            ));
        }
    }
    std::fs::write(out, &csv).unwrap();
    eprintln!("meter sweep: {rows} layouts measured, {compared} compared->{out}");
}

fn sim_target(report_path: &str) -> (Option<f64>, Option<f64>) {
    let raw = match std::fs::read_to_string(report_path) {
        Ok(s) => s,
        Err(_) => return (None, None),
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };
    let items = v.pointer("/report/items").and_then(|i| i.as_array());
    let Some(items) = items else { return (None, None) };
    let mut prod = None;
    let mut del = None;
    for it in items {
        let is_t = it.get("is_target").and_then(|b| b.as_bool()).unwrap_or(false);
        if is_t {
            prod = it.get("measured_produced_rate").and_then(|x| x.as_f64());
            del = it.get("measured_delivered_rate").and_then(|x| x.as_f64());
        }
    }
    (prod, del)
}
