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

/// The harness's own convergence minimum (`scenario.rs`:
/// `MIN_CHECKPOINTS = STABILITY_WINDOWS + 1`). A row at exactly this count
/// converged at the earliest opportunity, which `sim-harness-forensics.md`
/// class 5c flags as needing longer-warmup confirmation before it is trusted
/// as the asymptote.
///
/// **This is NOT a filter.** An earlier revision gated on `>= 4` and called it
/// provenance — but every `converged: true` run has 4 by construction, so the
/// gate rejected nothing the convergence check had not already caught: a check
/// that reads as protection and discriminates nothing, which is the exact
/// failure `docs/validator-reporting.md` catalogues. Raising it to 5 would be
/// worse, silently deleting five of the six banked rows. So the count is
/// **reported per fixture** and rows at the minimum are marked, leaving the
/// judgement with the reader instead of hiding it behind a threshold.
const HARNESS_MIN_CHECKPOINTS: usize = 4;

/// Which rate a row is being compared on.
///
/// Both are carried because they answer different questions and the answers
/// differ. **Calibration** wants like-for-like: `sweep_corpus` compares
/// `produced` for solid targets, and matching it keeps the two sweeps
/// commensurable. **A gate** wants the number it would threshold on, and the
/// sim harness verdicts a solid target on `measured_delivered_rate`
/// (`crates/sim-harness/src/report.rs`, `verdict` for `!is_fluid_target`) — so
/// classifying on `produced` would grade the meter against a quantity no gate
/// consults. Reporting one and calling it "the" answer hides that seam; this
/// driver reports both and lets them disagree in public.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Metric {
    Produced,
    Delivered,
}

impl Metric {
    fn label(self) -> &'static str {
        match self {
            Metric::Produced => "produced",
            Metric::Delivered => "delivered",
        }
    }
}

struct Row {
    fixture: String,
    item: String,
    is_target: bool,
    planned: f64,
    /// Sim checkpoint depth for this fixture's run. Carried onto the row so
    /// the provenance distribution is reproducible from the CSV alone rather
    /// than only from stdout.
    checkpoints: usize,
    meter_produced: Option<f64>,
    meter_delivered: Option<f64>,
    sim_produced: Option<f64>,
    sim_delivered: Option<f64>,
}

impl Row {
    fn meter(&self, m: Metric) -> Option<f64> {
        match m {
            Metric::Produced => self.meter_produced,
            Metric::Delivered => self.meter_delivered,
        }
    }
    fn sim(&self, m: Metric) -> Option<f64> {
        match m {
            Metric::Produced => self.sim_produced,
            Metric::Delivered => self.sim_delivered,
        }
    }
    fn meter_pct(&self, m: Metric) -> Option<f64> {
        self.meter(m).map(|v| v / self.planned * 100.0)
    }
    fn sim_pct(&self, m: Metric) -> Option<f64> {
        self.sim(m).map(|v| v / self.planned * 100.0)
    }
    /// Planned-relative: `(meter - sim) / planned * 100`, i.e. the gap in
    /// **percentage points of plan**. This is the unit a gate reasons in,
    /// because a gate thresholds "% of plan" — so the classification below
    /// must use it.
    fn delta_pp(&self, m: Metric) -> Option<f64> {
        Some(self.meter_pct(m)? - self.sim_pct(m)?)
    }

    /// Sim-relative: `(meter - sim) / sim * 100`, i.e. the meter's **percent
    /// error against its reference**.
    ///
    /// Carried because this — NOT `delta_pp` — is what `sweep_corpus` reports,
    /// and the corpus's headline bounds ("every optimistic error is <= +1.3",
    /// "pessimistic errors run to -13.6") are in these units despite the log
    /// calling them "pp". The two agree only where sim ~= plan, which is
    /// exactly where a below-plan fixture is not. Comparing a planned-relative
    /// number against those bounds silently mixes units; both are emitted here
    /// so a cross-sweep claim can be made in the corpus's own terms.
    fn delta_vs_sim_pct(&self, m: Metric) -> Option<f64> {
        let (meter, sim) = (self.meter(m)?, self.sim(m)?);
        if sim == 0.0 {
            return None;
        }
        Some((meter - sim) / sim * 100.0)
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
    let mut checkpoint_counts: Vec<(String, usize)> = Vec::new();

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

        // A corrupt or truncated report must cost its own row, not the run.
        // Panicking here means one bad file — sorted early — takes down a whole
        // calibration sweep, which is the same "silent" failure wearing a
        // louder coat: the other fixtures never get measured at all.
        let raw = match std::fs::read_to_string(&rp_path) {
            Ok(s) => s,
            Err(e) => {
                excluded.push((fixture, format!("report.json unreadable: {e}")));
                continue;
            }
        };
        let report: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                excluded.push((fixture, format!("report.json is not valid JSON: {e}")));
                continue;
            }
        };
        let rep = &report["report"];

        // Schema check BEFORE the provenance gates. A report.json missing its
        // `report` wrapper — schema drift, or a partial/corrupt write — makes
        // every field below read as `Null`, and the convergence gate would then
        // reject it as "did not converge": a malformed file masquerading as a
        // legitimately non-converged run. Name the real reason instead.
        if !rep.is_object() {
            excluded.push((fixture, "report.json has no `report` object — unparseable".into()));
            continue;
        }

        // Bind the report to the directory it was filed under. Without this the
        // sweep pairs bp.txt and report.json by folder alone, so a report
        // copied into the wrong dir would attribute meter(layout A) to
        // sim(layout B) — a confidently wrong calibration row, which is the one
        // failure this whole exercise cannot afford.
        //
        // Honest about its limits: this catches a MISFILED report, not a STALE
        // one. A bp.txt re-exported without a fresh sim run keeps a matching
        // label and would still pass. Binding that properly needs a blueprint
        // hash recorded at run time, which the harness does not emit today.
        if let Some(label) = rep["label"].as_str() {
            if label != fixture {
                excluded.push((
                    fixture.clone(),
                    format!("report label {label:?} does not match directory {fixture:?}"),
                ));
                continue;
            }
        }

        // Provenance gates, in the order that makes a bad row cheapest to
        // reject. `kit_errors` first: it invalidates the run outright.
        //
        // A MISSING or non-array key is rejected, not defaulted to empty. The
        // permissive reading is the inverse silent drop: a schema-drifted
        // report would silently *gain* coverage by looking clean.
        let Some(kit) = rep["kit_errors"].as_array() else {
            excluded.push((fixture, "report has no `kit_errors` array — cannot vet".into()));
            continue;
        };
        if !kit.is_empty() {
            let first = kit[0].as_str().unwrap_or("").to_string();
            excluded.push((fixture, format!("kit_errors ({}): {first}", kit.len())));
            continue;
        }
        // Same treatment for fluids. The provenance claim in
        // `meter-divergence.md` says these rows are fluid-clean; a claim no
        // code enforces is a claim that drifts.
        let fluid_len = match &rep["fluid_errors"] {
            serde_json::Value::Array(a) => a.len(),
            serde_json::Value::Object(o) => o.len(),
            _ => {
                excluded.push((fixture, "report has no `fluid_errors` — cannot vet".into()));
                continue;
            }
        };
        if fluid_len > 0 {
            excluded.push((fixture, format!("fluid_errors ({fluid_len})")));
            continue;
        }
        if rep["converged"].as_bool() != Some(true) {
            excluded.push((fixture, "sim run did not converge".into()));
            continue;
        }
        // Reported, not gated — see HARNESS_MIN_CHECKPOINTS. A missing key is
        // still rejected, because that is schema drift rather than a weak run.
        //
        // Prefer `report.measurement.checkpoints`, the harness's OWN parsed
        // count (`checkpoint_series.len()` — entries with a usable
        // tick/produced/delivered triple). Counting `raw_result.checkpoints`
        // instead counts unparsed entries too, so a malformed one would
        // overstate convergence depth — and this number is exactly what marks
        // a row "at the harness minimum" vs "best-provenanced", which the
        // divergence log's conclusions lean on.
        let Some(checkpoints) = rep["measurement"]["checkpoints"]
            .as_u64()
            .map(|n| n as usize)
            .or_else(|| report["raw_result"]["checkpoints"].as_array().map(|a| a.len()))
        else {
            excluded.push((fixture, "report has no checkpoint count".into()));
            continue;
        };
        // Consistency check on the RAW count, reporting on the parsed one.
        // `HARNESS_MIN_CHECKPOINTS` is a bound on how many checkpoints the
        // harness writes; `measurement.checkpoints` counts only those with a
        // usable triple, so a converged run whose first entry lacked one would
        // be wrongly called inconsistent if this compared the parsed number.
        let raw_checkpoints = report["raw_result"]["checkpoints"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(checkpoints);
        if raw_checkpoints < HARNESS_MIN_CHECKPOINTS {
            // Fewer checkpoints than the harness can emit for a converged run
            // means the report contradicts itself.
            excluded.push((
                fixture,
                format!("{raw_checkpoints} checkpoint(s) but converged=true — inconsistent report"),
            ));
            continue;
        }
        // Recorded, but only committed once the fixture actually contributes a
        // row (below). Pushing here would let a fixture that passes every gate
        // and then yields nothing still inflate the provenance denominator —
        // the shrinking-denominator failure this file guards against, running
        // in the opposite direction.
        let pending_checkpoints = checkpoints;

        // Same rule as report.json: one unreadable file costs its own row, not
        // the run. `exists()` above only proves the path is there.
        let bp = match std::fs::read_to_string(&bp_path) {
            Ok(s) => s,
            Err(e) => {
                excluded.push((fixture, format!("bp.txt unreadable: {e}")));
                continue;
            }
        };
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

        let items = rep["items"].as_array().cloned().unwrap_or_default();
        // Per-fixture counter rather than a `rows.len()` diff: the guard below
        // means "this fixture contributed nothing", and it should keep meaning
        // that if anything else ever touches `rows` in this loop.
        let mut added = 0usize;
        for item in items {
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
                checkpoints,
                meter_produced: meter.produced_per_s.get(&name).copied(),
                meter_delivered: meter.delivered_per_s.get(&name).copied(),
                sim_produced: item["measured_produced_rate"].as_f64(),
                sim_delivered: item["measured_delivered_rate"].as_f64(),
            });
            added += 1;
        }
        // A kit-clean, converged report that yields no usable row would
        // otherwise vanish from the table AND from the exclusion list — the
        // silent drop this file's header warns against, arriving through the
        // one door the `kit_errors` gate does not cover.
        if added == 0 {
            excluded.push((fixture, "report has no items with a non-zero planned_rate".into()));
        } else {
            checkpoint_counts.push((fixture, pending_checkpoints));
        }
    }

    let f = |v: Option<f64>| v.map_or("n/a".into(), |x| format!("{x:.4}"));
    let p = |v: Option<f64>| v.map_or("n/a".into(), |x| format!("{x:.2}"));

    // --- per-item table ---------------------------------------------------
    // `tgt` is a declared column, not a bare marker appended past the last
    // header field: an unlabelled 9th column on some rows only is exactly the
    // kind of output a reader misreads.
    println!("--- per-item table (PRODUCED rate; delivered is in the target sections below) ---");
    println!(
        "{:<30}{:<22}{:>10}{:>10}{:>10}{:>9}{:>9}{:>10}{:>6}",
        "fixture", "item", "plan", "meter_prod", "sim_prod", "meter%", "sim%", "Dpp", "tgt"
    );
    println!("{}", "-".repeat(116));
    for r in &rows {
        // The per-item table shows `produced` — the like-for-like quantity,
        // matching `sweep_corpus`. The gate table below re-does the target
        // rows on `delivered`.
        println!(
            "{:<30}{:<22}{:>10.4}{:>10}{:>10}{:>9}{:>9}{:>10}{:>6}",
            r.fixture,
            r.item,
            r.planned,
            f(r.meter(Metric::Produced)),
            f(r.sim(Metric::Produced)),
            p(r.meter_pct(Metric::Produced)),
            p(r.sim_pct(Metric::Produced)),
            r.delta_pp(Metric::Produced)
                .map_or("n/a".into(), |x| format!("{x:+.2}")),
            if r.is_target { "*" } else { "" },
        );
    }

    // --- target summary + gate classification, on BOTH metrics -------------
    let all_targets: Vec<&Row> = rows.iter().filter(|r| r.is_target).collect();

    for metric in [Metric::Produced, Metric::Delivered] {
        let targets: Vec<&&Row> = all_targets
            .iter()
            .filter(|r| r.delta_pp(metric).is_some())
            .collect();
        // Never let the denominator shrink in silence. A target the meter did
        // not produce, or one the sim reports no rate for, must be named --
        // otherwise "1/6" is read as coverage that was never there.
        let uncomparable: Vec<&&Row> = all_targets
            .iter()
            .filter(|r| r.delta_pp(metric).is_none())
            .collect();

        println!(
            "\n=== TARGET ITEMS on {} ({} of {} comparable) ===",
            metric.label(),
            targets.len(),
            all_targets.len()
        );
        for r in &targets {
            println!(
                "{:<30}{:<22}{:>9.2}{:>9.2}{:>+9.2}",
                r.fixture,
                r.item,
                r.meter_pct(metric).unwrap(),
                r.sim_pct(metric).unwrap(),
                r.delta_pp(metric).unwrap()
            );
        }
        for r in &uncomparable {
            println!(
                "{:<30}{:<22}  NOT COMPARABLE (meter {}, sim {})",
                r.fixture,
                r.item,
                f(r.meter(metric)),
                f(r.sim(metric))
            );
        }
        if !targets.is_empty() {
            let d: Vec<f64> = targets.iter().map(|r| r.delta_pp(metric).unwrap()).collect();
            let mean = d.iter().sum::<f64>() / d.len() as f64;
            let worst_opt = d.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let worst_pess = d.iter().cloned().fold(f64::INFINITY, f64::min);
            println!(
                "planned-relative : mean {mean:+.2}pp | worst optimistic {worst_opt:+.2}pp | worst pessimistic {worst_pess:+.2}pp"
            );
            // Same rows in sweep_corpus's units, so the corpus bounds can be
            // quoted against these without mixing denominators.
            let e: Vec<f64> = targets
                .iter()
                .filter_map(|r| r.delta_vs_sim_pct(metric))
                .collect();
            if !e.is_empty() {
                let emean = e.iter().sum::<f64>() / e.len() as f64;
                let eopt = e.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let epess = e.iter().cloned().fold(f64::INFINITY, f64::min);
                // Sim-relative is undefined at sim == 0, so its denominator can
                // be smaller than the planned-relative one. Say so rather than
                // letting the two summaries silently describe different sets.
                let dropped = targets.len() - e.len();
                let note = if dropped > 0 {
                    format!("   [{dropped} row(s) omitted: sim rate is 0]")
                } else {
                    String::new()
                };
                println!(
                    "sim-relative     : mean {emean:+.2}% | worst optimistic {eopt:+.2}% | worst pessimistic {epess:+.2}%   <- sweep_corpus units{note}"
                );
            }
        }

        // Two quadrants, and they are not the same risk:
        //   MISSED DEFECT    meter says at-plan, sim says below  -> report-only
        //                    is worth less than claimed (it stays silent on a
        //                    real deficit). This is the FLOOR property.
        //   FALSE ACCUSATION meter says below-plan, sim says at   -> a BLOCKING
        //                    gate rejects a good layout.
        println!("\n--- gate classification on {} ---", metric.label());
        println!(
            "{:>10}{:>18}{:>20}   offenders",
            "threshold", "missed defects", "false accusations"
        );
        for t in THRESHOLDS {
            let mut names = Vec::new();
            let (mut missed, mut false_acc) = (0, 0);
            for r in &targets {
                let (m, s) = (r.meter_pct(metric).unwrap(), r.sim_pct(metric).unwrap());
                if m >= t && s < t {
                    missed += 1;
                    names.push(format!("MISS:{}", r.fixture));
                }
                if m < t && s >= t {
                    false_acc += 1;
                    names.push(format!("FALSE:{}", r.fixture));
                }
            }
            println!(
                "{:>9.0}%{:>18}{:>20}   {}",
                t,
                format!("{}/{}", missed, targets.len()),
                format!("{}/{}", false_acc, targets.len()),
                names.join(" ")
            );
        }
    }

    // Provenance strength, stated rather than gated. A row at the harness
    // minimum converged at the earliest opportunity and carries the class-5c
    // caution; one well above it does not. This is the difference between
    // "vetted" and "vetted, and here is how strongly".
    let at_min = checkpoint_counts
        .iter()
        .filter(|(_, n)| *n == HARNESS_MIN_CHECKPOINTS)
        .count();
    println!("\n=== SIM PROVENANCE (checkpoints per fixture) ===");
    for (fx, n) in &checkpoint_counts {
        let flag = if *n == HARNESS_MIN_CHECKPOINTS {
            "  <- AT harness minimum (forensics class 5c: confirm with a longer warmup)"
        } else {
            ""
        };
        println!("  {fx:<30} {n}{flag}");
    }
    println!(
        "  {at_min}/{} at the minimum of {HARNESS_MIN_CHECKPOINTS}. This is NOT a filter — every \
         converged run has >= {HARNESS_MIN_CHECKPOINTS} by construction.",
        checkpoint_counts.len()
    );

    if !excluded.is_empty() {
        println!("\n=== EXCLUDED ({}) ===", excluded.len());
        for (fx, why) in &excluded {
            println!("  {fx:<30} {why}");
        }
    }

    if let Some(out) = out {
        // Missing values are written as EMPTY fields, not the literal "n/a":
        // the columns are numeric, and a float parser reading "n/a" either
        // throws or -- worse -- coerces. The console table can afford a word;
        // a CSV consumed by something else cannot.
        let c = |v: Option<f64>| v.map_or(String::new(), |x| format!("{x:.4}"));
        let cp = |v: Option<f64>| v.map_or(String::new(), |x| format!("{x:.2}"));
        // Both units per metric. The sim-relative columns are the ONLY ones
        // comparable against `sweep_corpus`'s bounds, so omitting them would
        // leave the doc's headline multipliers unreproducible from the
        // machine-readable artifact.
        let mut csv = String::from(
            "fixture,item,is_target,planned,sim_checkpoints,\
             meter_produced,sim_produced,meter_produced_pct,sim_produced_pct,delta_pp_produced,delta_vs_sim_pct_produced,\
             meter_delivered,sim_delivered,meter_delivered_pct,sim_delivered_pct,delta_pp_delivered,delta_vs_sim_pct_delivered\n",
        );
        for r in &rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                r.fixture,
                r.item,
                r.is_target,
                r.planned,
                r.checkpoints,
                c(r.meter(Metric::Produced)),
                c(r.sim(Metric::Produced)),
                cp(r.meter_pct(Metric::Produced)),
                cp(r.sim_pct(Metric::Produced)),
                cp(r.delta_pp(Metric::Produced)),
                cp(r.delta_vs_sim_pct(Metric::Produced)),
                c(r.meter(Metric::Delivered)),
                c(r.sim(Metric::Delivered)),
                cp(r.meter_pct(Metric::Delivered)),
                cp(r.sim_pct(Metric::Delivered)),
                cp(r.delta_pp(Metric::Delivered)),
                cp(r.delta_vs_sim_pct(Metric::Delivered)),
            ));
        }
        std::fs::write(&out, csv).unwrap();
        println!("\nwrote {out}");
    }
}
