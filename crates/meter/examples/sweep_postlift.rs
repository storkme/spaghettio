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
/// provenance — but every `converged: true` run has at least 4 by construction
/// (runs continue to a tick ceiling; tier2-ec10-lift has 8), so the
/// gate rejected nothing the convergence check had not already caught: a check
/// that reads as protection and discriminates nothing, which is the exact
/// failure `docs/validator-reporting.md` catalogues. Raising it to 5 would be
/// worse, silently deleting five of the six banked rows. So the count is
/// **reported per fixture** and rows at the minimum are marked, leaving the
/// judgement with the reader instead of hiding it behind a threshold.
///
/// **Drift point.** This mirrors a value that lives in another crate and is
/// itself derived (`STABILITY_WINDOWS + 1`). If the harness raises
/// `STABILITY_WINDOWS`, this silently desynchronises and the consistency check
/// below starts rejecting legitimately-converged runs. It is not fixed "by
/// construction" across versions — only within one.
const HARNESS_MIN_CHECKPOINTS: usize = 4;

/// The warmup the 2026-08-07 bank was run at, and the figure both
/// `meter-divergence.md` and `status.md` quote as a credential.
///
/// **Gated**, unlike the checkpoint count. A short warmup makes a reading
/// invalid rather than merely weak — `CLAUDE.md` says it reads buffer fill as
/// throughput and must not be quoted — so a row below this is excluded, not
/// annotated. The warmup column in the provenance table is therefore always
/// `>= ` this value; it is printed so the credential is visible rather than
/// taken on trust.
const EXPECTED_WARMUP_TICKS: u64 = 432_000;

/// Per-fixture sim provenance, kept so the strength of each row is visible
/// rather than asserted in prose.
struct Provenance {
    fixture: String,
    checkpoints: usize,
    warmup_ticks: u64,
}

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

use sha2::{Digest, Sha256};

/// What a `calibration_matrix_export` bank declares about itself, read from
/// its `matrix.json`. Absent for the older ad-hoc banks.
struct MatrixIndex {
    /// Fixtures the corpus DECLARES — exported rows plus any that failed to
    /// build at export. The honest coverage denominator.
    corpus_size: usize,
    /// Rows recorded under `build_failures`: declared, never exported, so
    /// they have no directory and can never be measured from this bank.
    build_failures: usize,
    /// `label → blueprint_sha256` for every exported row.
    hashes: std::collections::BTreeMap<String, String>,
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
    let mut provenance: Vec<Provenance> = Vec::new();
    let mut dropped: Vec<(String, Vec<String>)> = Vec::new();

    let mut fixtures: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {dir}: {e}"))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    fixtures.sort();
    // A bank created by `calibration_matrix_export` carries an explicit list
    // of expected fixture labels. Validate it before measuring anything: a
    // missing directory otherwise turns a 35-row matrix into a plausible
    // 34-row one, which is exactly the silent coverage shrink this tool exists
    // to expose. Older ad-hoc banks deliberately have no such contract and
    // retain their directory-driven behavior.
    let matrix_path = std::path::Path::new(&dir).join("matrix.json");
    let matrix = if matrix_path.exists() {
        let raw = std::fs::read_to_string(&matrix_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", matrix_path.display()));
        let matrix: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("parse {}: {e}", matrix_path.display()));
        let mut hashes = std::collections::BTreeMap::new();
        for v in matrix["fixtures"]
            .as_array()
            .unwrap_or_else(|| panic!("{} has no fixtures array", matrix_path.display()))
        {
            let label = v["label"]
                .as_str()
                .unwrap_or_else(|| panic!("{} has fixture without label", matrix_path.display()));
            let hash = v["blueprint_sha256"].as_str().unwrap_or_else(|| {
                panic!("{} row {label} has no blueprint_sha256", matrix_path.display())
            });
            if hashes.insert(label.to_string(), hash.to_string()).is_some() {
                panic!("{} names {label} twice", matrix_path.display());
            }
        }
        let expected: std::collections::BTreeSet<String> = hashes.keys().cloned().collect();
        let declared = matrix["fixture_count"]
            .as_u64()
            .unwrap_or_else(|| panic!("{} has no fixture_count", matrix_path.display()))
            as usize;
        assert_eq!(
            expected.len(),
            declared,
            "{} declares {declared} fixtures but names {} unique labels",
            matrix_path.display(),
            expected.len()
        );
        let actual: std::collections::BTreeSet<String> = fixtures
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            actual, expected,
            "matrix directory set differs from {}; regenerate a fresh bank rather than mixing artifacts",
            matrix_path.display()
        );
        // Additive fields (second exporter revision): a bank from the first
        // revision has neither and declared exactly what it exported.
        let build_failures = matrix["build_failures"].as_array().map_or(0, Vec::len);
        let corpus_size = matrix["corpus_size"]
            .as_u64()
            .map_or(declared + build_failures, |n| n as usize);
        assert_eq!(
            corpus_size,
            declared + build_failures,
            "{} corpus_size must equal fixture_count + build_failures",
            matrix_path.display()
        );
        Some(MatrixIndex { corpus_size, build_failures, hashes })
    } else {
        None
    };

    for fp in fixtures {
        let fixture = fp.file_name().unwrap().to_string_lossy().to_string();
        let bp_path = fp.join("bp.txt");
        let mf_path = fp.join("manifest-real.json");
        let rp_path = fp.join("report.json");
        if !bp_path.exists() || !mf_path.exists() {
            excluded.push((fixture, "no bp.txt / manifest-real.json".into()));
            continue;
        }
        // Bank integrity before any report gate. The matrix fingerprint is
        // the only thing that binds report.json to THIS bp.txt — the label
        // matches whether or not someone re-exported underneath the report —
        // and a blueprint that no longer matches is a corrupted bank whatever
        // else is true of the row, so it is reported as that and not as the
        // first report gate it happens to fail. A fingerprint recorded but
        // never read lets a stale report through as vetted (#710 review).
        if let Some(index) = &matrix {
            let bytes = match std::fs::read(&bp_path) {
                Ok(b) => b,
                Err(e) => {
                    excluded.push((fixture, format!("bp.txt unreadable: {e}")));
                    continue;
                }
            };
            let actual = format!("{:x}", Sha256::digest(&bytes));
            match index.hashes.get(&fixture) {
                Some(recorded) if *recorded == actual => {}
                Some(_) => {
                    excluded.push((
                        fixture,
                        "bp.txt sha256 differs from matrix.json — report.json cannot be bound \
                         to this blueprint; regenerate a fresh bank"
                            .into(),
                    ));
                    continue;
                }
                None => {
                    excluded.push((fixture, "label absent from matrix.json fixtures".into()));
                    continue;
                }
            }
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
        let Some(label) = rep["label"].as_str() else {
            // Absent key rejects, like every sibling gate. An `if let Some`
            // here would quietly downgrade this guard to a no-op on exactly
            // the schema drift it exists to catch.
            excluded.push((fixture, "report has no `label` — cannot bind to its directory".into()));
            continue;
        };
        if label != fixture {
            excluded.push((
                fixture.clone(),
                format!("report label {label:?} does not match directory {fixture:?}"),
            ));
            continue;
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
        let Some(raw_checkpoints) = report["raw_result"]["checkpoints"]
            .as_array()
            .map(|a| a.len())
        else {
            excluded.push((fixture, "report has no `raw_result.checkpoints` array".into()));
            continue;
        };
        if raw_checkpoints < HARNESS_MIN_CHECKPOINTS {
            // Fewer checkpoints than the harness can emit for a converged run
            // means the report contradicts itself.
            excluded.push((
                fixture,
                format!("{raw_checkpoints} checkpoint(s) but converged=true — inconsistent report"),
            ));
            continue;
        }
        // The warmup the sim actually ran. Both `meter-divergence.md` and
        // `status.md` quote "warmup 432 000 on every one" as a credential, and
        // an unenforced credential drifts. `CLAUDE.md` is explicit that the
        // dim-scaled default is too short for deep chains and reads buffer
        // fill as throughput, so a short-warmup row would be exactly the class
        // that must not be quoted -- and it would otherwise pass every gate
        // here, since it can be kit-clean, converged and 4-checkpointed.
        let Some(pending_warmup) = report["run_params"]["warmup_ticks"].as_u64() else {
            excluded.push((fixture, "report has no `run_params.warmup_ticks` — cannot vet".into()));
            continue;
        };
        // GATED, unlike the checkpoint count, and the distinction is
        // deliberate: checkpoint depth is a strength gradient (a class-5c row
        // is weak but real), whereas a short warmup makes the reading INVALID
        // -- `CLAUDE.md` says such numbers read buffer fill as throughput and
        // must not be quoted. A validity bar excludes; a strength gradient
        // reports.
        if pending_warmup < EXPECTED_WARMUP_TICKS {
            excluded.push((
                fixture,
                format!(
                    "warmup {pending_warmup} < {EXPECTED_WARMUP_TICKS} — reads buffer fill as \
                     throughput, must not be quoted"
                ),
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
        let mut bad_schema = false;
        let mut dropped_items: Vec<String> = Vec::new();
        // Per-fixture counter rather than a `rows.len()` diff: the guard below
        // means "this fixture contributed nothing", and it should keep meaning
        // that if anything else ever touches `rows` in this loop.
        let mut added = 0usize;
        for item in items {
            // Empty/absent name would push a plausible-looking row whose meter
            // lookup can never match, and it would still count toward `added`,
            // so the "no usable items" guard would not catch it either.
            let Some(name) = item["item"].as_str().filter(|s| !s.is_empty()) else {
                dropped_items.push("<unnamed>".into());
                continue;
            };
            let name = name.to_string();
            // A malformed `is_target` defaulting to false would drop the target
            // from the classification AND from the NOT-COMPARABLE list —
            // vanishing without trace, and capable of turning "1/6 missed
            // defects" into "0/5" with no warning. Absent is legitimately
            // false; present-but-not-a-bool is schema drift and rejects.
            let is_target = match &item["is_target"] {
                serde_json::Value::Null => false,
                serde_json::Value::Bool(b) => *b,
                other => {
                    excluded.push((
                        fixture.clone(),
                        format!("item {name:?} has non-bool `is_target`: {other}"),
                    ));
                    bad_schema = true;
                    break;
                }
            };
            // A bare `continue` here would let a fixture whose TARGET lacks a
            // planned_rate still pass `added > 0` on its intermediates, and the
            // target would vanish from the table unannounced.
            let Some(planned) = item["planned_rate"].as_f64() else {
                dropped_items.push(format!("{name} (no planned_rate)"));
                continue;
            };
            if planned == 0.0 {
                dropped_items.push(format!("{name} (planned_rate = 0)"));
                continue;
            }
            rows.push(Row {
                fixture: fixture.clone(),
                item: name.clone(),
                is_target,
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
        if bad_schema {
            // Roll back this fixture's rows: a schema-drifted report must not
            // contribute a partial set that reads as a complete one.
            rows.truncate(rows.len() - added);
            continue;
        }
        if !dropped_items.is_empty() {
            dropped.push((fixture.clone(), dropped_items));
        }
        if added == 0 {
            excluded.push((fixture, "report has no items with a non-zero planned_rate".into()));
        } else {
            provenance.push(Provenance {
                fixture: fixture.clone(),
                checkpoints: pending_checkpoints,
                warmup_ticks: pending_warmup,
            });
        }
    }

    let f = |v: Option<f64>| v.map_or("n/a".into(), |x| format!("{x:.4}"));
    let p = |v: Option<f64>| v.map_or("n/a".into(), |x| format!("{x:.2}"));

    if let Some(index) = &matrix {
        // Four buckets that sum to the corpus the bank declares. A fixture
        // excluded for any reason other than "not measured yet" (kit errors,
        // non-convergence, short warmup, a hash mismatch above) is a coverage
        // shortfall and is counted as one — not folded into "awaiting", and
        // not dropped from the denominator.
        let awaiting = excluded
            .iter()
            .filter(|(_, why)| why == "no sim report.json — nothing to compare")
            .count();
        let other = excluded.len() - awaiting;
        let vetted = provenance.len();
        println!("=== MATRIX COVERAGE ===");
        println!(
            "  {vetted}/{} fixtures have a vetted sim report; {awaiting} awaiting measurement; \
             {other} excluded for another reason (see EXCLUDED); {} failed to build at export",
            index.corpus_size, index.build_failures,
        );
        let accounted = vetted + awaiting + other + index.build_failures;
        if accounted != index.corpus_size {
            println!(
                "  WARNING: buckets sum to {accounted}, corpus declares {} — a fixture was \
                 neither vetted nor excluded (a fixture directory that yielded no row?)",
                index.corpus_size
            );
        }
    }

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
    // Counts TARGET ROWS, not distinct fixtures. Each banked fixture declares
    // exactly one target today, so "N/6" reads as fixtures — but a multi-target
    // report (RFC-062 `--multi`) would inflate both the denominator and the
    // offender list without saying so. Flagged when it happens.
    let all_targets: Vec<&Row> = rows.iter().filter(|r| r.is_target).collect();
    {
        let mut seen = std::collections::BTreeMap::<&str, usize>::new();
        for r in &all_targets {
            *seen.entry(r.fixture.as_str()).or_default() += 1;
        }
        let multi: Vec<String> = seen
            .iter()
            .filter(|(_, n)| **n > 1)
            .map(|(f, n)| format!("{f} ({n} targets)"))
            .collect();
        if !multi.is_empty() {
            println!(
                "\nNOTE: gate denominators count target ROWS, and these fixtures contribute more \
                 than one: {}",
                multi.join(", ")
            );
        }
    }

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
    let at_min = provenance
        .iter()
        .filter(|p| p.checkpoints <= HARNESS_MIN_CHECKPOINTS)
        .count();
    println!("\n=== SIM PROVENANCE ===");
    println!("  {:<30} {:>11}  {:>7}", "fixture", "checkpoints", "warmup");
    for p in &provenance {
        let cp_flag = if p.checkpoints <= HARNESS_MIN_CHECKPOINTS {
            "  <- checkpoints AT/below harness minimum (forensics class 5c)"
        } else {
            ""
        };
        println!(
            "  {:<30} {:>11}  {:>7}{cp_flag}",
            p.fixture, p.checkpoints, p.warmup_ticks
        );
    }
    println!(
        "  {at_min}/{} at/below the checkpoint minimum of {HARNESS_MIN_CHECKPOINTS} (NOT a filter — \
         every converged run has >= {HARNESS_MIN_CHECKPOINTS} by construction). \
         Warmup is gated at {EXPECTED_WARMUP_TICKS}, so every row shown cleared it.",
        provenance.len()
    );

    if !dropped.is_empty() {
        println!("\n=== ITEMS DROPPED (fixture kept) ===");
        for (fx, items) in &dropped {
            println!("  {fx:<30} {}", items.join(", "));
        }
    }

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
