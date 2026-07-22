//! Process orchestration: write the scenario, launch headless Factorio,
//! poll `script-output/` for the result, and tear the server down.
//!
//! We hold the `Child` handle directly and call `child.kill()` to tear
//! the server down — this sidesteps the RFC brief's "pkill patterns must
//! not match your own process" footgun entirely (that concern only
//! applies when a shell script backgrounds the process and kills it by
//! pattern-matching `ps`/`pgrep` output later; owning the exact PID via
//! `std::process::Child` makes pattern matching unnecessary).

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

fn clear_stale_outputs(install_dir: &Path, scenario_name: &str) {
    let dir = paths::script_output_dir(install_dir);
    for name in ["harness-result.json", "sim-state.json"] {
        let _ = std::fs::remove_file(dir.join(name));
    }
    let _ = scenario_name;
}

pub fn write_scenario(install_dir: &Path, scenario_name: &str, control_lua: &str) -> Result<(), String> {
    let dir = paths::scenarios_dir(install_dir).join(scenario_name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("creating scenario dir {dir:?}: {e}"))?;
    std::fs::write(dir.join("control.lua"), control_lua)
        .map_err(|e| format!("writing control.lua: {e}"))
}

/// Launch the scenario, poll for `harness-result.json`, then kill the
/// server. `timeout_secs` bounds wall-clock (KC4: a USP-scale cycle
/// should stay under ~6 min; the default here is deliberately generous
/// to cover slower dev machines / cold caches, per the RFC's own
/// "rescope to nightly batch" escape hatch above that).
pub fn launch_and_wait(
    install_dir: &Path,
    scenario_name: &str,
    timeout_secs: u64,
) -> Result<RunOutcome, String> {
    clear_stale_outputs(install_dir, scenario_name);

    let binary = paths::factorio_binary_path(install_dir);
    let settings = paths::server_settings_path(install_dir);
    let port = free_port()?;

    let log_path = std::env::temp_dir().join(format!("spaghettio-sim-{scenario_name}.log"));
    let log_file = std::fs::File::create(&log_path).map_err(|e| format!("creating log file {log_path:?}: {e}"))?;
    let log_file_err = log_file.try_clone().map_err(|e| e.to_string())?;

    let mut child: Child = Command::new(&binary)
        .current_dir(install_dir)
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

    let result_path = paths::script_output_dir(install_dir).join("harness-result.json");
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let outcome = loop {
        if result_path.is_file() {
            break Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            break Err(format!(
                "factorio exited early (status {status}) before writing a result; see log at {log_path:?}"
            ));
        }
        if Instant::now() >= deadline {
            break Err(format!(
                "timed out after {timeout_secs}s waiting for {result_path:?}; see log at {log_path:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(1000));
    };

    // Always tear the server down, whether we succeeded or timed out.
    let _ = child.kill();
    let _ = child.wait();

    outcome?;

    let result = read_json(&result_path)?;
    let sim_state_path = paths::script_output_dir(install_dir).join("sim-state.json");
    let sim_state = if sim_state_path.is_file() {
        Some(read_json(&sim_state_path)?)
    } else {
        None
    };

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
}
