//! `spaghettio-sim` — RFC-050 headless simulation harness CLI.
//!
//! Subcommands: `fetch` (pinned Factorio download), `run` (measurement
//! pipeline), `check-data` (KC1 dump-data parity spot-check). See
//! `docs/rfc-050-headless-sim-harness.md`.

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
                      [--out report.json] [--timeout-secs N]
  spaghettio-sim check-data

ENV:
  SPAGHETTIO_FACTORIO_DIR   Override the install dir (default:
                            ~/.cache/spaghettio-sim/factorio-2.0.76)
"#
    );
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
    let params = scenario::RunParams::defaults_for(&manifest, scenario_name.clone(), speed, ticks);
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
