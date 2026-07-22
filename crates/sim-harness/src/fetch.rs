//! `spaghettio-sim fetch` — download the PINNED headless build, extract
//! it, and stamp the config files the scenario/orchestrator require.
//!
//! Deliberately pinned: RFC-050 KC1 requires the harness to match
//! `crates/core/data/recipes.json`'s 2.0.76 baseline exactly, and the
//! discovery spike found `latest` had already drifted to 2.1.12 (new
//! `recycler` prototype module) by the time this RFC was written. NEVER
//! pass `latest` to the download URL.

use crate::paths::{self, PINNED_VERSION};
use std::path::Path;
use std::process::Command;

const DOWNLOAD_URL: &str = "https://factorio.com/get-download/2.0.76/headless/linux64";

/// The server settings the RFC's empirical base requires: `auto_pause`
/// false (dedicated servers otherwise never tick with no players
/// connected) and `autosave_interval` 0 (a multi-minute run has no use
/// for mid-run autosaves, and they cost wall-clock KC4 is tight on).
/// Layered onto the game's own `server-settings.example.json` shape so
/// unrelated fields keep sane defaults, `serde_json::Value`-merged so we
/// don't have to hand-maintain every field the game ships.
fn harness_server_settings() -> serde_json::Value {
    serde_json::json!({
        "name": "spaghettio-sim harness",
        "description": "RFC-050 headless measurement run",
        "tags": ["spaghettio-sim"],
        "max_players": 0,
        "visibility": {"public": false, "lan": false},
        "username": "",
        "password": "",
        "token": "",
        "game_password": "",
        "require_user_verification": false,
        "max_upload_in_kilobytes_per_second": 0,
        "max_upload_slots": 5,
        "minimum_latency_in_ticks": 0,
        "max_heartbeats_per_second": 60,
        "ignore_player_limit_for_returning_players": false,
        "allow_commands": "admins-only",
        "autosave_interval": 0,
        "autosave_slots": 1,
        "afk_autokick_interval": 0,
        "auto_pause": false,
        "auto_pause_when_players_connect": false,
        "only_admins_can_pause_the_game": true,
        "autosave_only_on_server": true,
        "non_blocking_saving": false,
    })
}

fn mod_list() -> serde_json::Value {
    serde_json::json!({
        "mods": [
            {"name": "base", "enabled": true},
            {"name": "elevated-rails", "enabled": true},
            {"name": "quality", "enabled": true},
            {"name": "space-age", "enabled": true},
        ]
    })
}

pub fn run(force: bool) -> Result<(), String> {
    let install_dir = paths::target_install_dir()?;
    let binary = paths::factorio_binary_path(&install_dir);

    if binary.is_file() && !force {
        println!("Factorio {PINNED_VERSION} already present at {install_dir:?} (skipping download; pass --force to re-fetch)");
    } else {
        download_and_extract(&install_dir)?;
    }

    stamp_config(&install_dir)?;
    std::fs::create_dir_all(paths::scenarios_dir(&install_dir))
        .map_err(|e| format!("creating scenarios dir: {e}"))?;
    std::fs::create_dir_all(paths::script_output_dir(&install_dir))
        .map_err(|e| format!("creating script-output dir: {e}"))?;

    println!("Factorio {PINNED_VERSION} ready at {install_dir:?}");
    Ok(())
}

fn download_and_extract(install_dir: &Path) -> Result<(), String> {
    let parent = install_dir
        .parent()
        .ok_or_else(|| format!("install dir {install_dir:?} has no parent"))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating cache dir {parent:?}: {e}"))?;

    let tmp_archive = parent.join(format!(".spaghettio-sim-download-{}.tar.xz", std::process::id()));
    let extract_root = parent.join(format!(".spaghettio-sim-extract-{}", std::process::id()));

    println!("Downloading pinned Factorio {PINNED_VERSION} from {DOWNLOAD_URL} ...");
    let status = Command::new("curl")
        .args(["-fL", "-o"])
        .arg(&tmp_archive)
        .arg(DOWNLOAD_URL)
        .status()
        .map_err(|e| format!("failed to spawn curl (is it installed?): {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp_archive);
        return Err(format!("curl exited with {status}"));
    }

    std::fs::create_dir_all(&extract_root).map_err(|e| format!("creating extract dir: {e}"))?;
    println!("Extracting ...");
    let status = Command::new("tar")
        .arg("-xJf")
        .arg(&tmp_archive)
        .arg("-C")
        .arg(&extract_root)
        .status()
        .map_err(|e| format!("failed to spawn tar (is it installed?): {e}"))?;
    let _ = std::fs::remove_file(&tmp_archive);
    if !status.success() {
        let _ = std::fs::remove_dir_all(&extract_root);
        return Err(format!("tar exited with {status}"));
    }

    // The headless tarball extracts a single top-level `factorio/` dir;
    // move it into place at the versioned target path.
    let extracted_factorio = extract_root.join("factorio");
    if !extracted_factorio.is_dir() {
        return Err(format!(
            "expected {extracted_factorio:?} after extraction, but it doesn't exist \
             (tarball layout may have changed)"
        ));
    }
    if install_dir.exists() {
        std::fs::remove_dir_all(install_dir).map_err(|e| format!("removing stale install dir: {e}"))?;
    }
    std::fs::rename(&extracted_factorio, install_dir)
        .map_err(|e| format!("moving extracted install into place: {e}"))?;
    let _ = std::fs::remove_dir_all(&extract_root);

    let binary = paths::factorio_binary_path(install_dir);
    if !binary.is_file() {
        return Err(format!("extraction completed but {binary:?} is missing"));
    }
    Ok(())
}

fn stamp_config(install_dir: &Path) -> Result<(), String> {
    let mod_list_path = paths::mod_list_path(install_dir);
    if let Some(parent) = mod_list_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("creating mods dir: {e}"))?;
    }
    write_json(&mod_list_path, &mod_list())?;

    let settings_path = paths::server_settings_path(install_dir);
    write_json(&settings_path, &harness_server_settings())?;
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let s = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, s).map_err(|e| format!("writing {path:?}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_settings_disable_auto_pause_and_autosave() {
        let v = harness_server_settings();
        assert_eq!(v["auto_pause"], false);
        assert_eq!(v["autosave_interval"], 0);
    }

    #[test]
    fn mod_list_enables_space_age_stack() {
        let v = mod_list();
        let names: Vec<&str> = v["mods"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["name"].as_str().unwrap())
            .collect();
        for expect in ["base", "space-age", "quality", "elevated-rails"] {
            assert!(names.contains(&expect), "missing mod {expect}");
        }
    }

    #[test]
    fn download_url_is_pinned_never_latest() {
        assert!(DOWNLOAD_URL.contains("2.0.76"));
        assert!(!DOWNLOAD_URL.contains("latest"));
    }
}
