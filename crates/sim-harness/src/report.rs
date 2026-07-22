//! Report computation: planned vs measured per item, PASS/WARN/FAIL
//! verdict per RFC-050 KC2 (one-sided — overshoot is expected because
//! placed machine counts are ceil'd above the fractional plan, and is
//! reported informationally, never penalized).

use crate::manifest::Manifest;
use std::collections::BTreeMap;
use std::fmt;

/// KC2's PASS boundary, verbatim from the RFC: "Measured target rate >=
/// 0.98 x planned ... at steady state".
const PASS_RATIO: f64 = 0.98;
/// WARN/FAIL split. The RFC only pins the PASS boundary; this floor is a
/// resolved ambiguity (documented here rather than silently invented) —
/// below it the shortfall looks like more than measurement noise and
/// should read as a hard failure rather than "close but under".
const WARN_RATIO: f64 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
    /// No measurement was available to judge (e.g. the run never reached
    /// a stability checkpoint AND produced no samples either).
    NoData,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
            Verdict::NoData => "NO DATA",
        })
    }
}

fn verdict_for_ratio(ratio: Option<f64>) -> Verdict {
    match ratio {
        None => Verdict::NoData,
        Some(r) if r >= PASS_RATIO => Verdict::Pass,
        Some(r) if r >= WARN_RATIO => Verdict::Warn,
        _ => Verdict::Fail,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ItemReport {
    pub item: String,
    pub planned_rate: f64,
    pub measured_produced_rate: Option<f64>,
    pub measured_delivered_rate: Option<f64>,
    pub delta_pct_produced: Option<f64>,
    pub delta_pct_delivered: Option<f64>,
    pub is_target: bool,
    pub verdict: Option<Verdict>,
}

impl serde::Serialize for Verdict {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    pub label: String,
    pub items: Vec<ItemReport>,
    pub import_rc: i64,
    pub ghosts: u64,
    pub revived: u64,
    pub pole_networks: u64,
    pub factory_eeis: u64,
    pub proxies_fulfilled: u64,
    pub converged: bool,
    pub final_tick: u64,
    pub fluid_fed: bool,
    pub uncalibrated_direction: bool,
    pub fluid_errors: BTreeMap<String, String>,
    /// Boundary-kit self-audit failures (overlapping bank chests etc.).
    /// Non-empty means the kit itself is compromised — measured rates are
    /// meaningless (wrong-item cross-feeds poison the factory; #357) and
    /// the overall verdict is forced to NO DATA.
    pub kit_errors: Vec<String>,
    pub machine_census: BTreeMap<String, u64>,
    pub overall_verdict: Verdict,
    /// Manifest context (RFC-050: "config axes (quality/stacking/
    /// inserter-capacity)" and "external_inputs" are report context, not
    /// measurements — surfaced here rather than dropped).
    pub entities: usize,
    pub stacking: u8,
    pub inserter_capacity: u8,
    pub external_inputs: Vec<(String, f64, bool)>,
    /// Realized force capacity bonuses at finalize (tech-state parity,
    /// #370) — surfaced so the parity the rates were measured under is
    /// part of the report, not buried in raw_result. The scenario also
    /// self-audits the assignment into `kit_errors` at init.
    pub inserter_stack_size_bonus: f64,
    pub bulk_inserter_capacity_bonus: f64,
}

fn get_u64(v: &serde_json::Value, key: &str) -> u64 {
    v.get(key).and_then(|x| x.as_u64()).unwrap_or(0)
}
fn get_i64(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}
fn get_bool(v: &serde_json::Value, key: &str) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(false)
}

/// `(tick, produced_count_for_item)` pairs pulled out of the `samples`
/// array for a single item.
fn sample_series(result: &serde_json::Value, item: &str) -> Vec<(f64, f64)> {
    result
        .get("samples")
        .and_then(|s| s.as_array())
        .into_iter()
        .flatten()
        .filter_map(|s| {
            let tick = s.get("tick")?.as_f64()?;
            let produced = s.get("produced")?.get(item)?.as_f64()?;
            Some((tick, produced))
        })
        .collect()
}

/// Rate over the trailing measurement window: from the sample at or
/// before `window_start` (the second-to-last checkpoint's tick — the
/// same window the target rate is measured over) to the last sample.
///
/// Falls back to the last two samples when no checkpoint window exists.
/// The last two samples span only 20 game-seconds, which is badly
/// aliased for bursty intermediate producers: the #357 recon caught a
/// gear machine (crafting in bursts between plate deliveries) reported
/// at 0.40/s on the 20s snapshot vs 0.80/s over the real window.
fn rate_over_window(series: &[(f64, f64)], window_start: Option<f64>) -> Option<f64> {
    let (t1, v1) = *series.last()?;
    let (t0, v0) = match window_start {
        Some(ws) => series
            .iter()
            .rev()
            .find(|(t, _)| *t <= ws)
            .copied()
            .or_else(|| series.first().copied()),
        None => (series.len() >= 2).then(|| series[series.len() - 2]),
    }?;
    let dt = (t1 - t0) / 60.0;
    if dt <= 0.0 {
        None
    } else {
        Some((v1 - v0) / dt)
    }
}

fn delta_pct(measured: Option<f64>, planned: f64) -> Option<f64> {
    if planned <= 0.0 {
        return None;
    }
    measured.map(|m| (m - planned) / planned * 100.0)
}

pub fn compute(manifest: &Manifest, result: &serde_json::Value) -> Report {
    let target_items: Vec<&str> = manifest.targets.iter().map(|t| t.item.as_str()).collect();

    let checkpoints = result
        .get("checkpoints")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    let checkpoint_series: Vec<(f64, f64, f64)> = checkpoints
        .iter()
        .filter_map(|c| {
            Some((
                c.get("tick")?.as_f64()?,
                c.get("produced")?.as_f64()?,
                c.get("delivered")?.as_f64()?,
            ))
        })
        .collect();
    let (target_produced_rate, target_delivered_rate) = if checkpoint_series.len() >= 2 {
        let (t0, p0, d0) = checkpoint_series[checkpoint_series.len() - 2];
        let (t1, p1, d1) = checkpoint_series[checkpoint_series.len() - 1];
        let dt = (t1 - t0) / 60.0;
        if dt > 0.0 {
            (Some((p1 - p0) / dt), Some((d1 - d0) / dt))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    // Intermediates measure over the same trailing window as the target.
    let window_start = (checkpoint_series.len() >= 2)
        .then(|| checkpoint_series[checkpoint_series.len() - 2].0);

    let mut items = Vec::new();
    for (item, planned_rate) in &manifest.planned_rates {
        let is_target = target_items.contains(&item.as_str());
        let measured_produced_rate = if is_target && target_produced_rate.is_some() {
            target_produced_rate
        } else {
            rate_over_window(&sample_series(result, item), window_start)
        };
        let measured_delivered_rate = if is_target { target_delivered_rate } else { None };
        let verdict = if is_target {
            Some(verdict_for_ratio(measured_delivered_rate.map(|m| m / planned_rate)))
        } else {
            None
        };
        items.push(ItemReport {
            item: item.clone(),
            planned_rate: *planned_rate,
            measured_produced_rate,
            measured_delivered_rate,
            delta_pct_produced: delta_pct(measured_produced_rate, *planned_rate),
            delta_pct_delivered: delta_pct(measured_delivered_rate, *planned_rate),
            is_target,
            verdict,
        });
    }
    items.sort_by(|a, b| b.is_target.cmp(&a.is_target).then_with(|| a.item.cmp(&b.item)));

    let kit_errors: Vec<String> = result
        .get("kit_errors")
        .and_then(|k| k.as_array())
        .into_iter()
        .flatten()
        .filter_map(|e| e.as_str().map(str::to_string))
        .collect();

    let overall_verdict = if kit_errors.is_empty() {
        items
            .iter()
            .filter(|i| i.is_target)
            .map(|i| i.verdict.unwrap_or(Verdict::NoData))
            .fold(Verdict::Pass, worst)
    } else {
        Verdict::NoData
    };

    let mut machine_census = BTreeMap::new();
    if let Some(obj) = result.get("machine_census").and_then(|c| c.as_object()) {
        for (k, v) in obj {
            machine_census.insert(k.clone(), v.as_u64().unwrap_or(0));
        }
    }
    let mut fluid_errors = BTreeMap::new();
    if let Some(obj) = result.get("fluid_errors").and_then(|c| c.as_object()) {
        for (k, v) in obj {
            fluid_errors.insert(k.clone(), v.as_str().unwrap_or_default().to_string());
        }
    }

    Report {
        label: manifest.label.clone(),
        items,
        import_rc: get_i64(result, "import_rc"),
        ghosts: get_u64(result, "ghosts"),
        revived: get_u64(result, "revived"),
        pole_networks: get_u64(result, "pole_networks"),
        factory_eeis: get_u64(result, "factory_eeis"),
        proxies_fulfilled: get_u64(result, "proxies_fulfilled"),
        converged: get_bool(result, "converged"),
        final_tick: get_u64(result, "final_tick"),
        fluid_fed: manifest.has_fluid_boundary(),
        uncalibrated_direction: manifest.has_uncalibrated_direction(),
        fluid_errors,
        kit_errors,
        machine_census,
        overall_verdict,
        entities: manifest.entities,
        stacking: manifest.stacking,
        inserter_capacity: manifest.inserter_capacity,
        external_inputs: manifest
            .external_inputs
            .iter()
            .map(|i| (i.item.clone(), i.rate, i.is_fluid))
            .collect(),
        inserter_stack_size_bonus: result
            .get("inserter_stack_size_bonus")
            .and_then(|v| v.as_f64())
            .unwrap_or(-1.0),
        bulk_inserter_capacity_bonus: result
            .get("bulk_inserter_capacity_bonus")
            .and_then(|v| v.as_f64())
            .unwrap_or(-1.0),
    }
}

fn worst(a: Verdict, b: Verdict) -> Verdict {
    fn rank(v: Verdict) -> u8 {
        match v {
            Verdict::Pass => 0,
            Verdict::Warn => 1,
            Verdict::NoData => 2,
            Verdict::Fail => 3,
        }
    }
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

pub fn print_human(report: &Report) {
    println!("=== spaghettio-sim report: {} ===", report.label);
    println!(
        "import: rc={} ghosts={} revived={} (failed={})",
        report.import_rc,
        report.ghosts,
        report.revived,
        report.ghosts.saturating_sub(report.revived)
    );
    println!(
        "power: {} pole network(s), {} factory EEI(s)",
        report.pole_networks, report.factory_eeis
    );
    println!("module proxies fulfilled: {}", report.proxies_fulfilled);
    println!(
        "run: final_tick={} converged={}",
        report.final_tick, report.converged
    );
    println!(
        "layout: {} entities, stacking={} inserter_capacity={} (realized bonuses: nb={} bulk={})",
        report.entities,
        report.stacking,
        report.inserter_capacity,
        report.inserter_stack_size_bonus,
        report.bulk_inserter_capacity_bonus
    );
    if !report.external_inputs.is_empty() {
        let inputs: Vec<String> = report
            .external_inputs
            .iter()
            .map(|(item, rate, is_fluid)| format!("{item}@{rate:.1}/s{}", if *is_fluid { " (fluid)" } else { "" }))
            .collect();
        println!("external inputs: {}", inputs.join(", "));
    }
    if report.fluid_fed {
        println!("NOTE: this run has fluid boundaries — infinity-pipe feed/void paths are UNCALIBRATED (no fixture has exercised them; RFC-050 Phase 1).");
    }
    if report.uncalibrated_direction {
        println!("NOTE: at least one boundary is not south-facing — the jog geometry is a faithful vector generalization of the calibrated south-only prototype, but has never been measured live.");
    }
    if !report.fluid_errors.is_empty() {
        println!("fluid rig errors:");
        for (k, v) in &report.fluid_errors {
            println!("  {k}: {v}");
        }
    }
    if !report.kit_errors.is_empty() {
        println!("KIT ERRORS — boundary kit compromised, RUN INVALID (verdict forced NO DATA):");
        for e in &report.kit_errors {
            println!("  {e}");
        }
    }
    println!();
    println!(
        "{:<28} {:>10} {:>12} {:>10} {:>12} {:>10} {:>8}",
        "item", "planned/s", "produced/s", "d%", "delivered/s", "d%", "verdict"
    );
    for item in &report.items {
        println!(
            "{:<28} {:>10.2} {:>12} {:>10} {:>12} {:>10} {:>8}",
            item.item,
            item.planned_rate,
            fmt_opt(item.measured_produced_rate),
            fmt_pct(item.delta_pct_produced),
            fmt_opt(item.measured_delivered_rate),
            fmt_pct(item.delta_pct_delivered),
            item.verdict.map(|v| v.to_string()).unwrap_or_default(),
        );
    }
    println!();
    if !report.machine_census.is_empty() {
        println!("machine census:");
        for (status, count) in &report.machine_census {
            println!("  {status}: {count}");
        }
    }
    println!();
    println!("OVERALL: {}", report.overall_verdict);
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "-".to_string())
}
fn fmt_pct(v: Option<f64>) -> String {
    v.map(|x| format!("{x:+.1}%")).unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

    fn fixture_manifest() -> Manifest {
        Manifest::from_str(include_str!("../tests/fixtures/manifest_gear10.json")).unwrap()
    }

    #[test]
    fn pass_at_or_above_98_percent() {
        assert_eq!(verdict_for_ratio(Some(0.98)), Verdict::Pass);
        assert_eq!(verdict_for_ratio(Some(1.5)), Verdict::Pass, "overshoot is informational, still PASS");
        assert_eq!(verdict_for_ratio(Some(0.979999)), Verdict::Warn);
    }

    #[test]
    fn warn_band_between_90_and_98_percent() {
        assert_eq!(verdict_for_ratio(Some(0.95)), Verdict::Warn);
        assert_eq!(verdict_for_ratio(Some(0.90)), Verdict::Warn);
    }

    #[test]
    fn fail_below_90_percent() {
        assert_eq!(verdict_for_ratio(Some(0.89999)), Verdict::Fail);
        assert_eq!(verdict_for_ratio(Some(0.0)), Verdict::Fail);
    }

    #[test]
    fn no_data_when_ratio_missing() {
        assert_eq!(verdict_for_ratio(None), Verdict::NoData);
    }

    #[test]
    fn compute_reads_checkpoints_for_target_rate() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "import_rc": 0, "ghosts": 428, "revived": 428,
            "pole_networks": 1, "factory_eeis": 1, "proxies_fulfilled": 0,
            "converged": true, "final_tick": 12600,
            "checkpoints": [
                {"tick": 9000, "produced": 1000.0, "delivered": 980.0},
                {"tick": 10800, "produced": 1300.0, "delivered": 1276.0}
            ],
            "samples": [],
            "machine_census": {"full_output": 5}
        });
        let report = compute(&m, &result);
        let target = report.items.iter().find(|i| i.item == "iron-gear-wheel").unwrap();
        // (1300-1000)/((10800-9000)/60) = 300/30 = 10.0/s
        assert!((target.measured_produced_rate.unwrap() - 10.0).abs() < 1e-9);
        // (1276-980)/30 = 9.866...
        assert!((target.measured_delivered_rate.unwrap() - 9.8666666667).abs() < 1e-6);
        assert_eq!(target.verdict, Some(Verdict::Pass));
        assert_eq!(report.overall_verdict, Verdict::Pass);
    }

    #[test]
    fn intermediate_rates_use_the_checkpoint_window_not_the_last_sample_pair() {
        // Bursty producer: 30 items in the first 20s of the window, none
        // in the last 20s. Last-two-samples reads 0.0/s; the honest
        // window rate is 30 items / 60s = 0.5/s.
        let series = vec![(9000.0, 100.0), (10200.0, 130.0), (11400.0, 130.0), (12600.0, 130.0)];
        assert_eq!(rate_over_window(&series, Some(9000.0)), Some(30.0 / 60.0));
        // Fallback without a window: the old 20s snapshot behavior.
        assert_eq!(rate_over_window(&series, None), Some(0.0));
        // Window start before the first sample clamps to the first sample.
        assert_eq!(rate_over_window(&series, Some(0.0)), Some(30.0 / 60.0));
        assert_eq!(rate_over_window(&[], Some(9000.0)), None);
    }

    #[test]
    fn compute_flags_fail_when_delivered_short() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "import_rc": 0, "ghosts": 428, "revived": 428,
            "pole_networks": 1, "factory_eeis": 1, "proxies_fulfilled": 0,
            "converged": true, "final_tick": 12600,
            "checkpoints": [
                {"tick": 9000, "produced": 1000.0, "delivered": 700.0},
                {"tick": 10800, "produced": 1300.0, "delivered": 850.0}
            ],
            "samples": [],
            "machine_census": {}
        });
        let report = compute(&m, &result);
        let target = report.items.iter().find(|i| i.item == "iron-gear-wheel").unwrap();
        // (850-700)/30 = 5.0/s vs planned 10.0/s -> 50%, FAIL
        assert_eq!(target.verdict, Some(Verdict::Fail));
        assert_eq!(report.overall_verdict, Verdict::Fail);
    }

    #[test]
    fn no_fluid_or_uncalibrated_flags_on_gear_fixture() {
        let m = fixture_manifest();
        let result = serde_json::json!({"checkpoints": [], "samples": []});
        let report = compute(&m, &result);
        assert!(!report.fluid_fed);
        assert!(!report.uncalibrated_direction);
    }
}
