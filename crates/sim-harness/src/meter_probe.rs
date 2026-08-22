//! Opt-in report-only execution of the fast meter beside a headless sim.
//!
//! This is deliberately a probe, not a gate. The meter's post-lift calibration
//! does not justify treating an at-plan reading as clearance, so a meter error
//! or low reading must never change the sim verdict.

use spaghettio_meter::{Factory, MeterReport};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub const DEFAULT_WARMUP_TICKS: u64 = 108_000;
pub const DEFAULT_WINDOW_TICKS: u64 = 216_000;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MeterProbe {
    pub warmup_ticks: u64,
    pub window_ticks: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<MeterReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl MeterProbe {
    pub fn run(bp: &str, manifest_json: &str, warmup_ticks: u64, window_ticks: u64) -> Self {
        let result = catch_unwind(AssertUnwindSafe(|| {
            (|| {
                let manifest = spaghettio_meter::Manifest::from_json(manifest_json)?;
                let mut factory = Factory::build(bp, manifest)?;
                Ok::<_, String>(factory.measure(warmup_ticks, window_ticks))
            })()
        }));

        match result {
            Ok(Ok(report)) => Self {
                warmup_ticks,
                window_ticks,
                report: Some(report),
                error: None,
            },
            Ok(Err(error)) => Self {
                warmup_ticks,
                window_ticks,
                report: None,
                error: Some(error),
            },
            Err(payload) => Self {
                warmup_ticks,
                window_ticks,
                report: None,
                error: Some(format!("meter panicked: {}", panic_text(&*payload))),
            },
        }
    }
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic payload".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_build_errors_are_reported_without_becoming_sim_failures() {
        let probe = MeterProbe::run("not-a-blueprint", "{not-json", 1, 1);
        assert!(probe.report.is_none());
        assert!(probe
            .error
            .as_deref()
            .is_some_and(|error| error.contains("manifest parse failed")));
        assert_eq!(probe.warmup_ticks, 1);
        assert_eq!(probe.window_ticks, 1);
    }
}
