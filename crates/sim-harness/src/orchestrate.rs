//! Process orchestration: prepare a per-run write dir, write the
//! scenario, launch headless Factorio, poll for the result, and tear the
//! server down.
//!
//! We hold the `Child` handle directly and call `child.kill()` to tear
//! the server down — this sidesteps the RFC brief's "pkill patterns must
//! not match your own process" footgun entirely (that concern only
//! applies when a shell script backgrounds the process and kills it by
//! pattern-matching `ps`/`pgrep` output later; owning the exact PID via
//! `std::process::Child` makes pattern matching unnecessary).
//!
//! ## Per-run write dirs (concurrency)
//!
//! Factorio takes an exclusive lock on its write directory and drops
//! results at fixed filenames under `script-output/`, so two runs
//! sharing a write dir either die at startup or read each other's
//! reports. Every `run` therefore gets a scratch write dir under the OS
//! temp dir, wired up via `--config` with a generated `config.ini`:
//! `read-data` points at the shared install's `data/` (never written),
//! `write-data` at the scratch dir (lock, `scenarios/`, `script-output/`,
//! logs — all per-run). Concurrent runs against one install just work;
//! only `fetch` and `check-data` still write into the install itself.

use crate::paths;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct RunOutcome {
    pub result: serde_json::Value,
    pub sim_state: Option<serde_json::Value>,
    pub log_path: PathBuf,
}

/// Grab an OS-assigned free port, then release it immediately. There is
/// an inherent (small) race between releasing and Factorio binding it;
/// acceptable for an offline engineer tool run a handful of times, and
/// this is the standard "ephemeral port" pattern absent a way to hand an
/// already-open socket to a non-Rust child process.
fn free_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("binding ephemeral port: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);
    Ok(port)
}

/// Create the per-run scratch write dir and populate it: `config.ini`
/// (read-data → shared install, write-data → here), `mods/mod-list.json`
/// (copied so the SA mod set is enabled without sharing a writable mods
/// dir), and empty `scenarios/` + `script-output/`. The scenario name
/// already carries a unix-seconds suffix; the PID suffix here
/// disambiguates same-second launches.
pub fn prepare_run_dir(install_dir: &Path, scenario_name: &str) -> Result<PathBuf, String> {
    let run_dir = std::env::temp_dir()
        .join("spaghettio-sim-runs")
        .join(format!("{scenario_name}-{}", std::process::id()));
    for sub in ["mods", "scenarios", "script-output"] {
        let d = run_dir.join(sub);
        std::fs::create_dir_all(&d).map_err(|e| format!("creating run dir {d:?}: {e}"))?;
    }

    let config = run_config_ini(install_dir, &run_dir);
    std::fs::write(run_dir.join("config.ini"), config)
        .map_err(|e| format!("writing per-run config.ini: {e}"))?;

    let mod_list_src = paths::mod_list_path(install_dir);
    std::fs::copy(&mod_list_src, run_dir.join("mods").join("mod-list.json"))
        .map_err(|e| format!("copying {mod_list_src:?} into run dir: {e}"))?;

    Ok(run_dir)
}

/// The generated per-run `config.ini`. Only `[path]` matters: everything
/// else falls back to Factorio's defaults, same as the portable
/// install's own config.
fn run_config_ini(install_dir: &Path, run_dir: &Path) -> String {
    format!(
        "[path]\nread-data={}\nwrite-data={}\n",
        install_dir.join("data").display(),
        run_dir.display()
    )
}

pub fn write_scenario(run_dir: &Path, scenario_name: &str, control_lua: &str) -> Result<(), String> {
    let dir = run_dir.join("scenarios").join(scenario_name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating scenario dir {dir:?}: {e}"))?;
    std::fs::write(dir.join("control.lua"), control_lua)
        .map_err(|e| format!("writing control.lua: {e}"))
}

/// Launch the scenario, poll for `harness-result.json` in the run dir's
/// `script-output/`, then kill the server. `timeout_secs` bounds
/// wall-clock (KC4: a USP-scale cycle should stay under ~6 min; the
/// default here is deliberately generous to cover slower dev machines /
/// cold caches, per the RFC's own "rescope to nightly batch" escape
/// hatch above that). On success the scratch run dir is removed; on
/// failure it is kept and its path reported for forensics.
pub fn launch_and_wait(
    install_dir: &Path,
    run_dir: &Path,
    scenario_name: &str,
    timeout_secs: u64,
) -> Result<RunOutcome, String> {
    let binary = paths::factorio_binary_path(install_dir);
    let settings = paths::server_settings_path(install_dir);
    let port = free_port()?;

    let log_path = std::env::temp_dir().join(format!("spaghettio-sim-{scenario_name}.log"));
    let log_file = std::fs::File::create(&log_path).map_err(|e| format!("creating log file {log_path:?}: {e}"))?;
    let log_file_err = log_file.try_clone().map_err(|e| e.to_string())?;

    let mut child: Child = Command::new(&binary)
        .current_dir(install_dir)
        .arg("--config")
        .arg(run_dir.join("config.ini"))
        .arg("--start-server-load-scenario")
        .arg(scenario_name)
        .arg("--server-settings")
        .arg(&settings)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_err))
        .spawn()
        .map_err(|e| format!("failed to launch {binary:?}: {e}"))?;

    let result_path = run_dir.join("script-output").join("harness-result.json");
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let outcome = loop {
        if result_path.is_file() {
            break Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            break Err(format!(
                "factorio exited early (status {status}) before writing a result; \
                 see log at {log_path:?} and run dir {run_dir:?}"
            ));
        }
        if Instant::now() >= deadline {
            break Err(format!(
                "timed out after {timeout_secs}s waiting for {result_path:?}; \
                 see log at {log_path:?} and run dir {run_dir:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(1000));
    };

    // Always tear the server down, whether we succeeded or timed out.
    let _ = child.kill();
    let _ = child.wait();

    outcome?;

    let result = read_json(&result_path)?;
    let sim_state_path = run_dir.join("script-output").join("sim-state.json");
    let sim_state = if sim_state_path.is_file() {
        Some(read_json(&sim_state_path)?)
    } else {
        None
    };

    // Scratch dir served its purpose; failures return above and keep it.
    let _ = std::fs::remove_dir_all(run_dir);

    Ok(RunOutcome { result, sim_state, log_path })
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("opening {path:?}: {e}"))?;
    let mut s = String::new();
    f.read_to_string(&mut s).map_err(|e| format!("reading {path:?}: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("parsing {path:?} as JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_port_is_nonzero_and_reusable_ish() {
        let p1 = free_port().unwrap();
        assert!(p1 > 0);
    }

    #[test]
    fn run_config_ini_points_read_at_install_and_write_at_run_dir() {
        let ini = run_config_ini(Path::new("/opt/factorio"), Path::new("/tmp/run-1"));
        assert_eq!(
            ini,
            "[path]\nread-data=/opt/factorio/data\nwrite-data=/tmp/run-1\n"
        );
    }

    #[test]
    fn prepare_run_dir_builds_isolated_layout() {
        // A fake install: only mods/mod-list.json is required by prepare.
        let base = std::env::temp_dir().join(format!("spaghettio-sim-test-{}", std::process::id()));
        let install = base.join("install");
        std::fs::create_dir_all(install.join("mods")).unwrap();
        std::fs::write(install.join("mods").join("mod-list.json"), "{\"mods\":[]}").unwrap();

        let run_dir = prepare_run_dir(&install, "spaghettio-sim-test-scn-0").unwrap();
        assert!(run_dir.join("config.ini").is_file());
        assert!(run_dir.join("mods").join("mod-list.json").is_file());
        assert!(run_dir.join("scenarios").is_dir());
        assert!(run_dir.join("script-output").is_dir());
        let ini = std::fs::read_to_string(run_dir.join("config.ini")).unwrap();
        assert!(ini.contains(&format!("write-data={}", run_dir.display())));
        assert!(ini.contains(&format!("read-data={}", install.join("data").display())));

        // Distinct scenario names → distinct run dirs (concurrency).
        let run_dir2 = prepare_run_dir(&install, "spaghettio-sim-test-scn-1").unwrap();
        assert_ne!(run_dir, run_dir2);

        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = std::fs::remove_dir_all(&run_dir2);
    }
}
