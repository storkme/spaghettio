//! RFC-054 **kill criterion 4**: the meter must not inherit the engine's
//! beliefs.
//!
//! The meter's whole value is being an *independent* instrument. If it
//! imported the engine's hand-calibrated rate model, its agreement with
//! the engine would be circular — which is exactly how `carries` labels
//! became worthless as ground truth (architecture audit §3.3 #17), and how
//! the backwards-inserter export bug survived the project's entire history
//! behind three artifacts that all shared the engine's direction
//! convention.
//!
//! This test lands in PR 1 on purpose. A guard added after the fact only
//! proves the code passes it *today*; a guard present from the first
//! commit means the boundary was never crossed.
//!
//! **If passing KC1 ever appears to require importing one of these, that
//! is the kill criterion firing — stop and record it, do not widen the
//! list.**

use std::fs;
use std::path::Path;

/// Symbols from `spaghettio_core::common` that encode the engine's
/// *estimates* rather than Factorio's facts.
const BANNED: &[&str] = &[
    "machine_feed_rate",
    "belt_drop_rate",
    "lane_capacity",
    "utilization_for",
    "LANE_UTILIZATION",
    "ROW_LANE_FACTOR",
    "belt_entity_for_rate",
];

fn rust_sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn meter_does_not_import_the_engine_rate_model() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src, &mut files);
    assert!(!files.is_empty(), "no sources found under {src:?}");

    let mut violations = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("read source");
        for (lineno, line) in text.lines().enumerate() {
            // Doc comments naming the ban are the point of the ban, not a
            // breach of it. Only real code counts.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for banned in BANNED {
                if line.contains(banned) {
                    violations.push(format!(
                        "{}:{}: references `{banned}`\n    {}",
                        file.display(),
                        lineno + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "RFC-054 KC4 breach — the meter must measure these, not import them:\n{}",
        violations.join("\n")
    );
}

/// The ban is on the *rate model*, not on `spaghettio_core` wholesale:
/// the blueprint parser and recipe DB are data and are explicitly allowed.
/// This test pins that distinction so nobody "fixes" KC4 by severing the
/// dependency the RFC relies on.
#[test]
fn data_dependencies_remain_permitted() {
    let manifest = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .expect("read manifest");
    assert!(
        manifest.contains("spaghettio_core"),
        "the meter is expected to depend on core for blueprint parsing and \
         recipe data; only the derived rate model is banned"
    );
}
