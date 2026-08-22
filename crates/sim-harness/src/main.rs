//! `spaghettio-sim` — RFC-050 headless simulation harness CLI.
//!
//! Subcommands: `fetch` (pinned Factorio download), `run` (measurement
//! pipeline), `check-data` (KC1 dump-data parity spot-check). See
//! `docs/rfc-050-headless-sim-harness.md`.

mod baseline;
mod checkdata;
mod fetch;
mod manifest;
mod meter_probe;
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
        Some("serve") => cmd_serve(&args[1..]),
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
                      [--warmup N] [--window N] [--fixed-window]
                      [--out report.json] [--timeout-secs N]
                      [--meter [--meter-warmup N] [--meter-window N]]
                      [--pickup-trace-only] [--drop-trace]
  spaghettio-sim serve --bp <file> --manifest <file> [--port 34197] [--speed 1]
                        [--warmup N]
  spaghettio-sim check-data
  spaghettio-sim bless --report <report.json> --baselines <dir> [--label <name>]
  spaghettio-sim check --report <report.json> --baselines <dir> [--tolerance 0.02]

ENV:
  SPAGHETTIO_FACTORIO_DIR   Override the install dir (default:
                            ~/.cache/spaghettio-sim/factorio-2.0.77)
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
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&b).map_err(|e| e.to_string())?,
    )
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
    let blessed_raw = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "no blessed baseline for '{}' at {} ({e})",
            fresh.label,
            path.display()
        )
    })?;
    let blessed: baseline::Baseline =
        serde_json::from_str(&blessed_raw).map_err(|e| e.to_string())?;
    let drifts = baseline::check_against(&blessed, &json, tolerance);
    if drifts.is_empty() {
        println!(
            "check: '{}' matches its blessed baseline (tolerance {:.1}%).",
            fresh.label,
            tolerance * 100.0
        );
        Ok(())
    } else {
        eprintln!(
            "check: '{}' DRIFTED from its blessed baseline:",
            fresh.label
        );
        for d in &drifts {
            eprintln!("  - {d}");
        }
        Err(format!(
            "{} drift(s); re-bless deliberately if intended",
            drifts.len()
        ))
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
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--ticks must be an integer, got '{s}'"))
        })
        .transpose()?;
    // Steady-state probe knob: delay measurement past slow buffer-fill
    // transients that the stability windows would misread as convergence.
    let warmup: Option<u32> = flag_value(args, "--warmup")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--warmup must be an integer, got '{s}'"))
        })
        .transpose()?;
    let window: Option<u32> = flag_value(args, "--window")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--window must be an integer, got '{s}'"))
        })
        .transpose()?;
    let speed: u32 = flag_value(args, "--speed")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--speed must be an integer, got '{s}'"))
        })
        .transpose()?
        .unwrap_or(16);
    let timeout_secs: Option<u64> = flag_value(args, "--timeout-secs")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--timeout-secs must be an integer, got '{s}'"))
        })
        .transpose()?;
    let out_path = flag_value(args, "--out");
    let meter_warmup: u64 = flag_value(args, "--meter-warmup")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--meter-warmup must be an integer, got '{s}'"))
        })
        .transpose()?
        .unwrap_or(meter_probe::DEFAULT_WARMUP_TICKS);
    let meter_window: u64 = flag_value(args, "--meter-window")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--meter-window must be an integer, got '{s}'"))
        })
        .transpose()?
        .unwrap_or(meter_probe::DEFAULT_WINDOW_TICKS);

    let install_dir = paths::resolve_existing_install()?;

    let bp = std::fs::read_to_string(bp_path)
        .map_err(|e| format!("reading blueprint file {bp_path}: {e}"))?
        .trim()
        .to_string();
    let manifest_str = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("reading manifest file {manifest_path}: {e}"))?;
    let manifest = manifest::Manifest::from_str(&manifest_str)?;

    let scenario_name = sanitize_scenario_name(&manifest.label);
    let mut params =
        scenario::RunParams::defaults_for(&manifest, scenario_name.clone(), speed, ticks);
    if let Some(w) = warmup {
        params = params.with_warmup(w);
    }
    if let Some(w) = window {
        if has_flag(args, "--fixed-window") {
            params = params.with_fixed_window(w);
        } else {
            return Err("--window requires --fixed-window".to_string());
        }
    } else if has_flag(args, "--fixed-window") {
        let window = params.window_ticks;
        params = params.with_fixed_window(window);
    }
    // Live per-window telemetry: stream the machine/item time-series to
    // script-output/timeseries.csv as the run progresses (not just into the
    // JSON at finalize), so a long/grinding run can be watched and scored in
    // real time. Measurement-safe (unlike `serve`'s operator QoL, it changes
    // no force bonuses and reveals no map).
    if has_flag(args, "--timeseries") {
        params = params.with_timeseries();
    }
    if has_flag(args, "--pickup-trace-only") {
        params = params.with_pickup_trace_only();
    }
    if has_flag(args, "--drop-trace") {
        params = params.with_drop_trace();
    }
    let meter = has_flag(args, "--meter").then(|| {
        println!(
            "Running report-only meter (warmup={} window={} ticks; it cannot alter the sim verdict)...",
            meter_warmup, meter_window
        );
        meter_probe::MeterProbe::run(&bp, &manifest_str, meter_warmup, meter_window)
    });
    let lua = scenario::build_control_lua(&manifest, &bp, &params);
    // Derived AFTER params so the wall-clock net always clears the run's
    // own tick budget — a timeout that fires first turns a non-converged
    // report into no report at all (#464 review).
    let timeout_secs = timeout_secs
        .unwrap_or_else(|| scenario::default_timeout_secs(params.end_tick, params.speed));

    let run_dir = orchestrate::prepare_run_dir(&install_dir, &scenario_name)?;
    orchestrate::write_scenario(&run_dir, &scenario_name, &lua)?;
    println!(
        "Launching scenario '{scenario_name}' (warmup={} window={} ceiling={} speed={} timeout={}s)...",
        params.warmup_ticks, params.window_ticks, params.end_tick, params.speed, timeout_secs
    );
    if params.write_timeseries {
        println!(
            "[timeseries] live CSV streaming to {}/script-output/timeseries.csv (watch: scripts/sim-watch.py {})",
            run_dir.display(),
            scenario_name
        );
    }
    let outcome =
        orchestrate::launch_and_wait(&install_dir, &run_dir, &scenario_name, timeout_secs)?;

    let rpt = report::compute(&manifest, &outcome.result);
    report::print_human(&rpt);
    if let Some(meter) = &meter {
        print_meter_probe(meter, &manifest);
    }

    if let Some(out_path) = out_path {
        let full = serde_json::json!({
            "report": rpt,
            "meter": meter,
            "raw_result": outcome.result,
            "sim_state": outcome.sim_state,
            "run_params": {
                "end_tick": params.end_tick,
                "speed": params.speed,
                "warmup_ticks": params.warmup_ticks,
                "window_ticks": params.window_ticks,
                "fixed_window": params.fixed_window,
                "scenario_name": params.scenario_name,
            },
            "game_version": paths::PINNED_VERSION,
        });
        std::fs::write(
            out_path,
            serde_json::to_string_pretty(&full).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("writing {out_path}: {e}"))?;
        println!("Full report written to {out_path}");
    }
    println!("(factorio log: {:?})", outcome.log_path);

    Ok(())
}

fn print_meter_probe(meter: &meter_probe::MeterProbe, manifest: &manifest::Manifest) {
    println!();
    println!(
        "=== report-only meter: warmup={} window={} ticks ===",
        meter.warmup_ticks, meter.window_ticks
    );
    if let Some(error) = &meter.error {
        println!("meter: ERROR — {error}");
        println!("meter: no sim verdict or gate decision was changed");
        return;
    }
    let Some(report) = &meter.report else {
        println!("meter: ERROR — no report returned");
        println!("meter: no sim verdict or gate decision was changed");
        return;
    };
    println!(
        "meter: converged={} boundary_refusals={} notes={}",
        report.converged,
        report.boundary_refusals,
        report.notes.len()
    );
    println!(
        "{:<24} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "target", "planned/s", "produced/s", "delivered/s", "gate/s", "gate d%"
    );
    for target in &manifest.targets {
        let planned = report
            .planned_per_s
            .get(&target.item)
            .copied()
            .unwrap_or(target.rate);
        let produced = report
            .produced_per_s
            .get(&target.item)
            .copied()
            .unwrap_or(0.0);
        let delivered = report
            .delivered_per_s
            .get(&target.item)
            .copied()
            .unwrap_or(0.0);
        // Match the sim harness's own target verdict: solid targets are
        // judged on delivered output, while fluid targets are judged on
        // production because the boundary drain is not meaningful there.
        let gate_rate = if target.is_fluid { produced } else { delivered };
        let delta = if planned > 0.0 {
            (gate_rate / planned - 1.0) * 100.0
        } else {
            f64::NAN
        };
        println!(
            "{:<24} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>+9.1}%",
            target.item, planned, produced, delivered, gate_rate, delta
        );
    }
    println!("meter: recipe attribution (measurement window)");
    for (recipe, a) in &report.recipe_attribution {
        println!(
            "  {recipe:<28} machines={:>3} crafts={:>6} working={:>7} output_blocked={:>7} inserter_blocked={:>7} item_shortage={:>7} fluid_shortage={:>7} supplied={:?} consumed={:?}",
            a.machines,
            a.crafts,
            a.working_ticks,
            a.output_blocked_ticks,
            a.output_inserter_blocked_ticks,
            a.item_shortage_ticks,
            a.fluid_shortage_ticks,
            a.fluid_supplied,
            a.fluid_consumed,
        );
    }
    println!(
        "meter: report-only; gate metric is delivered for solid targets and produced for fluids; an at-plan reading is not clearance"
    );
}

/// Run a fixture as a live, joinable Factorio server so a human can look
/// at it in a client.
///
/// The repo's verification protocol requires eyeballing a layout, not just
/// measuring it — a zero-warning layout that visibly has disconnected
/// belts is a validator bug, not a success. `run` cannot serve that: it
/// races at `game.speed = 16` and tears the world down the moment it has
/// its number.
///
/// Differences from `run`, all deliberate:
/// - `--speed 1` by default: real time, so a human can watch items move.
/// - A **fixed** port (34197, Factorio's default), because someone has to
///   type it into a client. `run` uses an ephemeral port for concurrency.
/// - No tick ceiling: the scenario's end tick is pushed far out, so the
///   scenario keeps MEASURING (and appending its time-series) instead of
///   finalizing at a ceiling minutes in. This does not by itself keep the
///   world alive — `finalize` also fires on convergence, and the ceiling
///   guard never covered that path (2026-08-07).
/// - [`scenario::RunParams::keep_alive`]: what actually keeps an inspected
///   world alive. It gates the boundary kit against finalize from EITHER
///   caller, so it alone is sufficient for aliveness; the ceiling above is
///   about how long measurement runs, not whether the factory keeps
///   running.
/// - The scratch run dir is left in place for the same reason.
///
/// The client's version must match the server's install exactly.
fn cmd_serve(args: &[String]) -> Result<(), String> {
    let bp_path = flag_value(args, "--bp").ok_or("serve requires --bp <file>")?;
    let manifest_path = flag_value(args, "--manifest").ok_or("serve requires --manifest <file>")?;
    let port: u16 = flag_value(args, "--port")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--port must be an integer, got '{s}'"))
        })
        .transpose()?
        .unwrap_or(34197);
    let speed: u32 = flag_value(args, "--speed")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--speed must be an integer, got '{s}'"))
        })
        .transpose()?
        .unwrap_or(1);
    let warmup: Option<u32> = flag_value(args, "--warmup")
        .map(|s| {
            s.parse()
                .map_err(|_| format!("--warmup must be an integer, got '{s}'"))
        })
        .transpose()?;

    let install_dir = paths::resolve_existing_install()?;
    let bp = std::fs::read_to_string(bp_path)
        .map_err(|e| format!("reading blueprint file {bp_path}: {e}"))?
        .trim()
        .to_string();
    let manifest_str = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("reading manifest file {manifest_path}: {e}"))?;
    let manifest = manifest::Manifest::from_str(&manifest_str)?;

    let scenario_name = sanitize_scenario_name(&manifest.label);
    // Roughly a week of game time at 60 UPS: far enough out that the
    // scenario never finalizes mid-inspection, without being a special
    // case in the scenario itself.
    const NO_CEILING: u32 = 36_000_000;
    let mut params = scenario::RunParams::defaults_for(
        &manifest,
        scenario_name.clone(),
        speed,
        Some(NO_CEILING),
    )
    .with_operator_qol()
    // This, not NO_CEILING above, is what keeps an inspected world alive:
    // `finalize` also fires on CONVERGENCE (minutes in), and that stops
    // the kit's feed/drain/power upkeep. Without it the operator inspects
    // a starved, stopped factory (2026-08-07).
    .with_keep_alive();
    if let Some(w) = warmup {
        params = params.with_warmup(w);
    }
    let lua = scenario::build_control_lua(&manifest, &bp, &params);

    let run_dir = orchestrate::prepare_run_dir(&install_dir, &scenario_name)?;
    orchestrate::write_scenario(&run_dir, &scenario_name, &lua)?;

    println!("=== spaghettio-sim serve: {} ===", manifest.label);
    println!("install : {}", install_dir.display());
    println!("scenario: {scenario_name} (speed {speed}x, no tick ceiling)");
    println!("run dir : {} (kept)", run_dir.display());
    // #537: the same per-checkpoint machine/item sampler that feeds
    // `run`'s JSON `timeseries` also appends CSV rows here, so a human
    // watching a `serve` session gets a machine-readable record of what
    // they watched instead of nothing — see docs/sim-harness.md "Reading
    // the time-series".
    println!(
        "timeseries: {} (CSV, appended each window until the scenario \
         finalizes — on convergence, or at the ceiling if it never \
         converges; the factory keeps running either way)",
        run_dir
            .join("script-output")
            .join("timeseries.csv")
            .display()
    );
    println!();
    println!("Connect a Factorio client of the SAME version via");
    println!("  Multiplayer -> Connect to address -> <host>:{port}");
    println!("Ctrl-C to stop the server.");
    println!();

    orchestrate::launch_server(&install_dir, &run_dir, &scenario_name, port)
}

fn cmd_check_data(_args: &[String]) -> Result<(), String> {
    let install_dir = paths::resolve_existing_install()?;
    let mismatches = checkdata::run(&install_dir)?;
    if mismatches.is_empty() {
        println!("check-data: OK — no mismatches between the pinned install's dumped data and recipes.json's baseline.");
        Ok(())
    } else {
        eprintln!(
            "check-data: {} mismatch(es) found (RFC-050 KC1):",
            mismatches.len()
        );
        for m in &mismatches {
            eprintln!("  - {m}");
        }
        Err(format!("{} KC1 mismatch(es); see above", mismatches.len()))
    }
}

fn sanitize_scenario_name(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let cleaned = if cleaned.is_empty() {
        "spaghettio-sim".to_string()
    } else {
        cleaned
    };
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
        let args = vec![
            "--bp".to_string(),
            "foo.txt".to_string(),
            "--speed".to_string(),
            "16".to_string(),
        ];
        assert_eq!(flag_value(&args, "--bp"), Some("foo.txt"));
        assert_eq!(flag_value(&args, "--speed"), Some("16"));
        assert_eq!(flag_value(&args, "--missing"), None);
    }
}
