//! `spaghettio-sim` — RFC-050 headless simulation harness CLI.
//!
//! Subcommands: `fetch` (pinned Factorio download), `run` (measurement
//! pipeline), `check-data` (KC1 dump-data parity spot-check). See
//! `docs/rfc-050-headless-sim-harness.md`.

mod baseline;
mod checkdata;
mod fetch;
mod manifest;
mod orchestrate;
mod paths;
mod report;
mod scenario;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sub = args.first().map(|s| s.as_str());

    let result = match sub {
        Some("fetch") => cmd_fetch(&args[1..]),
        Some("run") => cmd_run(&args[1..]),
        Some("check-data") => cmd_check_data(&args[1..]),
        Some("bless") => cmd_bless(&args[1..]),
        Some("check") => cmd_check(&args[1..]),
        Some("--help") | Some("-h") | None => {
            print_help();
            return ExitCode::SUCCESS;
        }
        Some(other) => Err(format!("unknown subcommand '{other}'; see --help")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    eprintln!(
        r#"spaghettio-sim — RFC-050 headless simulation harness

USAGE:
  spaghettio-sim fetch [--force]
  spaghettio-sim run --bp <file> --manifest <file> [--ticks N] [--speed N]
                      [--warmup N] [--out report.json] [--timeout-secs N]
  spaghettio-sim check-data
  spaghettio-sim bless --report <report.json> --baselines <dir> [--label <name>]
  spaghettio-sim check --report <report.json> --baselines <dir> [--tolerance 0.02]

ENV:
  SPAGHETTIO_FACTORIO_DIR   Override the install dir (default:
                            ~/.cache/spaghettio-sim/factorio-2.0.76)
"#
    );
}

fn cmd_bless(args: &[String]) -> Result<(), String> {
    let report_path = flag_value(args, "--report").ok_or("bless requires --report <file>")?;
    let dir = flag_value(args, "--baselines").ok_or("bless requires --baselines <dir>")?;
    let raw = std::fs::read_to_string(report_path).map_err(|e| format!("{report_path}: {e}"))?;
    let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut b = baseline::baseline_from_report(&json)?;
    // Optional relabel: report labels come from the exporter and may not
    // be unique per fixture (the baseline file is keyed on label).
    if let Some(label) = flag_value(args, "--label") {
        b.label = label.to_string();
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = baseline::baseline_path(std::path::Path::new(dir), &b.label);
    std::fs::write(&path, serde_json::to_string_pretty(&b).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    println!(
        "blessed '{}' ({} items produced, {} delivered, verdict {}) -> {}",
        b.label,
        b.produced.len(),
        b.delivered.len(),
        b.overall_verdict,
        path.display()
    );
    Ok(())
}

fn cmd_check(args: &[String]) -> Result<(), String> {
    let report_path = flag_value(args, "--report").ok_or("check requires --report <file>")?;
    let dir = flag_value(args, "--baselines").ok_or("check requires --baselines <dir>")?;
    let tolerance: f64 = flag_value(args, "--tolerance").map_or(Ok(0.02), |t| {
        t.parse().map_err(|_| format!("bad --tolerance '{t}'"))
    })?;
    let raw = std::fs::read_to_string(report_path).map_err(|e| format!("{report_path}: {e}"))?;
    let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let fresh = baseline::baseline_from_report(&json)?;
    let path = baseline::baseline_path(std::path::Path::new(dir), &fresh.label);
    let blessed_raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("no blessed baseline for '{}' at {} ({e})", fresh.label, path.display()))?;
    let blessed: baseline::Baseline =
        serde_json::from_str(&blessed_raw).map_err(|e| e.to_string())?;
    let drifts = baseline::check_against(&blessed, &json, tolerance);
    if drifts.is_empty() {
        println!("check: '{}' matches its blessed baseline (tolerance {:.1}%).", fresh.label, tolerance * 100.0);
        Ok(())
    } else {
        eprintln!("check: '{}' DRIFTED from its blessed baseline:", fresh.label);
        for d in &drifts {
            eprintln!("  - {d}");
        }
        Err(format!("{} drift(s); re-bless deliberately if intended", drifts.len()))
    }
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn cmd_fetch(args: &[String]) -> Result<(), String> {
    fetch::run(has_flag(args, "--force"))
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    let bp_path = flag_value(args, "--bp").ok_or("run requires --bp <file>")?;
    let manifest_path = flag_value(args, "--manifest").ok_or("run requires --manifest <file>")?;
    let ticks: Option<u32> = flag_value(args, "--ticks")
        .map(|s| s.parse().map_err(|_| format!("--ticks must be an integer, got '{s}'")))
        .transpose()?;
    // Steady-state probe knob: delay measurement past slow buffer-fill
    // transients that the stability windows would misread as convergence.
    let warmup: Option<u32> = flag_value(args, "--warmup")
        .map(|s| s.parse().map_err(|_| format!("--warmup must be an integer, got '{s}'")))
        .transpose()?;
    let speed: u32 = flag_value(args, "--speed")
        .map(|s| s.parse().map_err(|_| format!("--speed must be an integer, got '{s}'")))
        .transpose()?
        .unwrap_or(16);
    let timeout_secs: u64 = flag_value(args, "--timeout-secs")
        .map(|s| s.parse().map_err(|_| format!("--timeout-secs must be an integer, got '{s}'")))
        .transpose()?
        .unwrap_or(900);
    let out_path = flag_value(args, "--out");

    let install_dir = paths::resolve_existing_install()?;

    let bp = std::fs::read_to_string(bp_path)
        .map_err(|e| format!("reading blueprint file {bp_path}: {e}"))?
        .trim()
        .to_string();
    let manifest_str = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("reading manifest file {manifest_path}: {e}"))?;
    let manifest = manifest::Manifest::from_str(&manifest_str)?;

    let scenario_name = sanitize_scenario_name(&manifest.label);
    let mut params = scenario::RunParams::defaults_for(&manifest, scenario_name.clone(), speed, ticks);
    if let Some(w) = warmup {
        params = params.with_warmup(w);
    }
    let lua = scenario::build_control_lua(&manifest, &bp, &params);

    orchestrate::write_scenario(&install_dir, &scenario_name, &lua)?;
    println!(
        "Launching scenario '{scenario_name}' (warmup={} window={} ceiling={} speed={})...",
        params.warmup_ticks, params.window_ticks, params.end_tick, params.speed
    );
    let outcome = orchestrate::launch_and_wait(&install_dir, &scenario_name, timeout_secs)?;

    let rpt = report::compute(&manifest, &outcome.result);
    report::print_human(&rpt);

    if let Some(out_path) = out_path {
        let full = serde_json::json!({
            "report": rpt,
            "raw_result": outcome.result,
            "sim_state": outcome.sim_state,
            "run_params": {
                "end_tick": params.end_tick,
                "speed": params.speed,
                "warmup_ticks": params.warmup_ticks,
                "window_ticks": params.window_ticks,
                "scenario_name": params.scenario_name,
            },
            "game_version": paths::PINNED_VERSION,
        });
        std::fs::write(out_path, serde_json::to_string_pretty(&full).map_err(|e| e.to_string())?)
            .map_err(|e| format!("writing {out_path}: {e}"))?;
        println!("Full report written to {out_path}");
    }
    println!("(factorio log: {:?})", outcome.log_path);

    Ok(())
}

fn cmd_check_data(_args: &[String]) -> Result<(), String> {
    let install_dir = paths::resolve_existing_install()?;
    let mismatches = checkdata::run(&install_dir)?;
    if mismatches.is_empty() {
        println!("check-data: OK — no mismatches between the pinned install's dumped data and recipes.json's baseline.");
        Ok(())
    } else {
        eprintln!("check-data: {} mismatch(es) found (RFC-050 KC1):", mismatches.len());
        for m in &mismatches {
            eprintln!("  - {m}");
        }
        Err(format!("{} KC1 mismatch(es); see above", mismatches.len()))
    }
}

fn sanitize_scenario_name(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let cleaned = if cleaned.is_empty() { "spaghettio-sim".to_string() } else { cleaned };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("spaghettio-sim-{cleaned}-{ts}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_scenario_name_strips_bad_chars() {
        let n = sanitize_scenario_name("electronic circuit @10/s!");
        assert!(n.starts_with("spaghettio-sim-electronic-circuit"));
        assert!(!n.contains('/'));
        assert!(!n.contains('!'));
        assert!(!n.contains(' '));
        assert!(!n.contains('@'));
    }

    #[test]
    fn flag_value_parses_pairs() {
        let args = vec!["--bp".to_string(), "foo.txt".to_string(), "--speed".to_string(), "16".to_string()];
        assert_eq!(flag_value(&args, "--bp"), Some("foo.txt"));
        assert_eq!(flag_value(&args, "--speed"), Some("16"));
        assert_eq!(flag_value(&args, "--missing"), None);
    }
}
