//! Blessed measured baselines (RFC-050 Phase 3): `bless` freezes a run
//! report's measured rates as the expected values for a fixture; `check`
//! compares a fresh report against the blessed file and fails on drift.
//!
//! Factorio's sim is deterministic, so identical (blueprint, manifest,
//! game version, mods, tech) runs produce identical counts — but the
//! measurement WINDOW can land on a different tick boundary when the
//! loop-until-stable warmup converges at a different point, so a small
//! tolerance (default 2%, matching the stability tolerance) guards
//! window-phase jitter rather than real drift. Baselines are keyed on
//! (label, game_version) and record the mod set + tech state per the RFC
//! ("shareable — keyed on version+mods+tech, not per-host").

use std::collections::BTreeMap;
use std::path::Path;

/// One blessed fixture baseline, stored as `<dir>/<label>.json`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Baseline {
    pub label: String,
    pub game_version: String,
    /// Fixed for the harness today; recorded so a future change breaks
    /// the key loudly instead of silently comparing across worlds.
    pub mods: Vec<String>,
    pub tech_state: String,
    pub entities: usize,
    /// item → measured produced rate (items/s).
    pub produced: BTreeMap<String, f64>,
    /// target item → measured delivered rate (items/s).
    pub delivered: BTreeMap<String, f64>,
    pub overall_verdict: String,
}

pub const HARNESS_MODS: [&str; 4] = ["base", "space-age", "quality", "elevated-rails"];
pub const HARNESS_TECH_STATE: &str = "research_all_technologies";

/// Extract a [`Baseline`] from a full `run --out` report JSON value.
pub fn baseline_from_report(report: &serde_json::Value) -> Result<Baseline, String> {
    let r = report
        .get("report")
        .ok_or("not a spaghettio-sim report (missing `report` key)")?;
    let label = r
        .get("label")
        .and_then(|v| v.as_str())
        .ok_or("report.label missing")?
        .to_string();
    let game_version = report
        .get("game_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let mut produced = BTreeMap::new();
    let mut delivered = BTreeMap::new();
    for item in r.get("items").and_then(|v| v.as_array()).into_iter().flatten() {
        let name = item
            .get("item")
            .and_then(|v| v.as_str())
            .ok_or("item entry missing name")?
            .to_string();
        if let Some(p) = item.get("measured_produced_rate").and_then(|v| v.as_f64()) {
            produced.insert(name.clone(), p);
        }
        if let Some(d) = item.get("measured_delivered_rate").and_then(|v| v.as_f64()) {
            delivered.insert(name, d);
        }
    }
    if produced.is_empty() {
        return Err("report has no measured rates to bless".into());
    }
    Ok(Baseline {
        label,
        game_version,
        mods: HARNESS_MODS.iter().map(|s| s.to_string()).collect(),
        tech_state: HARNESS_TECH_STATE.into(),
        entities: r.get("entities").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        produced,
        delivered,
        overall_verdict: r
            .get("overall_verdict")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
    })
}

/// Compare a fresh report against a blessed baseline. Returns drift
/// descriptions (empty = check passes).
pub fn check_against(baseline: &Baseline, report: &serde_json::Value, tolerance: f64) -> Vec<String> {
    let mut drifts = Vec::new();
    let fresh = match baseline_from_report(report) {
        Ok(b) => b,
        Err(e) => return vec![format!("cannot read report: {e}")],
    };
    if fresh.game_version != baseline.game_version {
        drifts.push(format!(
            "game version {} != blessed {} — re-bless deliberately after a pin bump",
            fresh.game_version, baseline.game_version
        ));
        return drifts;
    }
    if fresh.mods != baseline.mods || fresh.tech_state != baseline.tech_state {
        drifts.push("mod set / tech state differs from blessed baseline".into());
        return drifts;
    }
    let compare = |kind: &str,
                   blessed: &BTreeMap<String, f64>,
                   fresh: &BTreeMap<String, f64>,
                   drifts: &mut Vec<String>| {
        for (item, want) in blessed {
            match fresh.get(item) {
                None => drifts.push(format!("{kind} {item}: missing from fresh report")),
                Some(got) => {
                    let denom = want.abs().max(1e-9);
                    let rel = (got - want).abs() / denom;
                    if rel > tolerance {
                        drifts.push(format!(
                            "{kind} {item}: {got:.2}/s vs blessed {want:.2}/s ({:+.1}%)",
                            (got - want) / denom * 100.0
                        ));
                    }
                }
            }
        }
        for item in fresh.keys() {
            if !blessed.contains_key(item) {
                drifts.push(format!("{kind} {item}: new item not in blessed baseline"));
            }
        }
    };
    compare("produced", &baseline.produced, &fresh.produced, &mut drifts);
    compare("delivered", &baseline.delivered, &fresh.delivered, &mut drifts);
    drifts
}

pub fn baseline_path(dir: &Path, label: &str) -> std::path::PathBuf {
    dir.join(format!("{label}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(label: &str, produced: f64, delivered: f64, version: &str) -> serde_json::Value {
        serde_json::json!({
            "game_version": version,
            "report": {
                "label": label,
                "entities": 428,
                "overall_verdict": "PASS",
                "items": [{
                    "item": "iron-gear-wheel",
                    "measured_produced_rate": produced,
                    "measured_delivered_rate": delivered,
                }],
            },
        })
    }

    #[test]
    fn bless_then_check_identical_passes() {
        let r = report("gear", 10.0, 10.13, "2.0.76");
        let b = baseline_from_report(&r).unwrap();
        assert!(check_against(&b, &r, 0.02).is_empty());
    }

    #[test]
    fn drift_beyond_tolerance_fails() {
        let b = baseline_from_report(&report("gear", 10.0, 10.13, "2.0.76")).unwrap();
        let drifted = report("gear", 9.0, 9.1, "2.0.76");
        let drifts = check_against(&b, &drifted, 0.02);
        assert_eq!(drifts.len(), 2, "{drifts:?}");
        assert!(drifts[0].contains("-10.0%"));
    }

    #[test]
    fn window_jitter_within_tolerance_passes() {
        let b = baseline_from_report(&report("gear", 10.0, 10.13, "2.0.76")).unwrap();
        let jitter = report("gear", 10.05, 10.0, "2.0.76");
        assert!(check_against(&b, &jitter, 0.02).is_empty());
    }

    #[test]
    fn version_mismatch_is_a_loud_single_error() {
        let b = baseline_from_report(&report("gear", 10.0, 10.13, "2.0.76")).unwrap();
        let bumped = report("gear", 10.0, 10.13, "2.1.12");
        let drifts = check_against(&b, &bumped, 0.02);
        assert_eq!(drifts.len(), 1);
        assert!(drifts[0].contains("re-bless deliberately"));
    }
}
