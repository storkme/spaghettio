//! Install-directory resolution shared by `fetch`, `run`, and `check-data`.

use std::path::PathBuf;

pub const PINNED_VERSION: &str = "2.0.76";

/// Default cache dir: `~/.cache/spaghettio-sim/factorio-2.0.76`.
pub fn default_install_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set; cannot resolve the default cache dir (set SPAGHETTIO_FACTORIO_DIR instead)".to_string())?;
    Ok(PathBuf::from(home)
        .join(".cache")
        .join("spaghettio-sim")
        .join(format!("factorio-{PINNED_VERSION}")))
}

/// The directory `fetch` targets: `SPAGHETTIO_FACTORIO_DIR` if set
/// (even if it doesn't exist yet — `fetch` will create it), else the
/// default cache path.
pub fn target_install_dir() -> Result<PathBuf, String> {
    match std::env::var_os("SPAGHETTIO_FACTORIO_DIR") {
        Some(v) => Ok(PathBuf::from(v)),
        None => default_install_dir(),
    }
}

/// Resolve an EXISTING install for `run`/`check-data`. Errors cleanly
/// (rather than panicking) when nothing is found at either location —
/// per the RFC-050 brief's constraint that these subcommands "error
/// cleanly when SPAGHETTIO_FACTORIO_DIR is absent". Resolution: absence
/// of the env var doesn't error by itself as long as a prior `fetch` has
/// already populated the default cache dir; what must error cleanly is
/// having NO usable install by either route.
pub fn resolve_existing_install() -> Result<PathBuf, String> {
    let candidate = target_install_dir()?;
    let binary = factorio_binary_path(&candidate);
    if binary.is_file() {
        return Ok(candidate);
    }
    Err(format!(
        "no Factorio install found at {candidate:?} (looked for {binary:?}).\n\
         Run `spaghettio-sim fetch` first, or set SPAGHETTIO_FACTORIO_DIR to an existing install."
    ))
}

pub fn factorio_binary_path(install_dir: &std::path::Path) -> PathBuf {
    install_dir.join("bin").join("x64").join("factorio")
}

pub fn server_settings_path(install_dir: &std::path::Path) -> PathBuf {
    install_dir.join("harness-server-settings.json")
}

pub fn mod_list_path(install_dir: &std::path::Path) -> PathBuf {
    install_dir.join("mods").join("mod-list.json")
}

pub fn scenarios_dir(install_dir: &std::path::Path) -> PathBuf {
    install_dir.join("scenarios")
}

pub fn script_output_dir(install_dir: &std::path::Path) -> PathBuf {
    install_dir.join("script-output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_path_matches_headless_layout() {
        let p = factorio_binary_path(std::path::Path::new("/foo/factorio-2.0.76"));
        assert_eq!(p, PathBuf::from("/foo/factorio-2.0.76/bin/x64/factorio"));
    }
}
