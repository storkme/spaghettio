//! Export the current-generation calibration matrix for Factorio and meter.
//!
//! ```text
//! cargo run --release -p spaghettio_core --example calibration_matrix_export -- \
//!   /path/to/new-bank
//! scripts/run-calibration-matrix.sh /path/to/new-bank
//! cargo run --release -p spaghettio_meter --example sweep_postlift -- \
//!   /path/to/new-bank /tmp/meter-vs-sim.csv
//! ```
//!
//! The output directory must be new or empty.  A calibration report only
//! means anything when its Factorio result belongs to the same blueprint, so
//! this exporter refuses to overwrite an existing bank.  Generate a new bank
//! after an engine change; do not replace `bp.txt` underneath an old report.
//!
//! A fixture that fails to build is recorded under `build_failures` in
//! `matrix.json` and the export continues: an engine regression is exactly
//! when the other rows' measurements are wanted, so one broken fixture must
//! not cost the bank.  The process still exits non-zero so a script notices.
//!
//! `matrix.json` schema versions (the sweep reads both):
//!   1 — first revision: `fixture_count`, `fixtures[]` with `blueprint_sha256`.
//!   2 — adds `corpus_size`, `build_failures[]`, and `manifest_sha256` per
//!       row, so the immutable bp.txt/manifest-real.json PAIR is fingerprinted.

use sha2::{Digest, Sha256};
use spaghettio_core::blueprint;
use spaghettio_core::calibration_matrix::{build, fixtures, CalibrationFixture, FixtureVariant};
use spaghettio_core::validate::Severity;

const SCHEMA_VERSION: u64 = 2;

fn usage() -> ! {
    eprintln!("usage: calibration_matrix_export <new-or-empty-bank-dir>");
    std::process::exit(2);
}

/// Machine-readable variant tag. A strategy row carries its discriminant —
/// the pooled/partitioned A/B pairs in the corpus are otherwise identical
/// rows distinguishable only by label.
fn variant_name(variant: FixtureVariant) -> String {
    match variant {
        FixtureVariant::Plain => "plain".into(),
        FixtureVariant::Strategy(s) => format!("strategy:{s:?}"),
        FixtureVariant::Excluded => "excluded".into(),
        FixtureVariant::ExcludedVoid => "excluded-void".into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn entry(
    fixture: &CalibrationFixture,
    built: &spaghettio_core::calibration_matrix::BuiltFixture,
    bp: &str,
    manifest_json: &str,
) -> serde_json::Value {
    let errors = built
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    let warnings = built
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .count();
    serde_json::json!({
        "label": fixture.name,
        "target": fixture.item,
        "rate": fixture.rate,
        "machine": fixture.machine,
        "belt_tier": fixture.belt_tier,
        "inputs": fixture.inputs,
        "excluded_recipes": fixture.excluded,
        "variant": variant_name(fixture.variant),
        "blueprint_sha256": sha256_hex(bp.as_bytes()),
        "manifest_sha256": sha256_hex(manifest_json.as_bytes()),
        "entities": built.layout.entities.len(),
        "dimensions": [built.layout.width, built.layout.height],
        "validator": { "errors": errors, "warnings": warnings },
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        usage();
    }
    let root = std::path::Path::new(&args[1]);
    if root.exists() {
        let non_empty = std::fs::read_dir(root)
            .unwrap_or_else(|e| panic!("read {}: {e}", root.display()))
            .next()
            .is_some();
        if non_empty {
            panic!(
                "refusing to overwrite non-empty calibration bank {}; choose a new directory",
                root.display()
            );
        }
    } else {
        std::fs::create_dir_all(root).unwrap_or_else(|e| panic!("create {}: {e}", root.display()));
    }

    let corpus = fixtures();
    let mut entries = Vec::new();
    let mut failures = Vec::new();
    for fixture in &corpus {
        let built = match build(fixture) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{:<62} FAILED: {e}", fixture.name);
                failures.push(serde_json::json!({ "label": fixture.name, "error": e }));
                continue;
            }
        };
        let (bp, manifest) = blueprint::export_with_manifest_validated(
            &built.layout,
            &built.solver_result,
            fixture.name,
            &built.issues,
        );
        // Hashed from the same string that is written, so a reader hashing
        // the file bytes reproduces it exactly.
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("manifest serializes");
        let dir = root.join(fixture.name);
        std::fs::create_dir(&dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
        std::fs::write(dir.join("bp.txt"), &bp)
            .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("bp.txt").display()));
        std::fs::write(dir.join("manifest-real.json"), &manifest_json)
            .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("manifest-real.json").display()));
        println!(
            "{:<62} {:>5} entities {:>4}x{:<4}",
            fixture.name,
            built.layout.entities.len(),
            built.layout.width,
            built.layout.height,
        );
        entries.push(entry(fixture, &built, &bp, &manifest_json));
    }
    let exported = entries.len();
    let failed = failures.len();
    // `fixture_count` is the number of exported rows (what the sweep checks
    // the directory set against); `corpus_size` is what the corpus declares.
    // They differ by exactly `build_failures.len()`.
    let index = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "purpose": "current-generation meter-vs-Factorio calibration matrix",
        "corpus_size": corpus.len(),
        "fixture_count": exported,
        "fixtures": entries,
        "build_failures": failures,
    });
    let index_path = root.join("matrix.json");
    std::fs::write(
        &index_path,
        serde_json::to_string_pretty(&index).expect("matrix serializes"),
    )
    .unwrap_or_else(|e| panic!("write {}: {e}", index_path.display()));
    println!(
        "wrote {exported} of {} fixtures ({failed} failed to build) and {}",
        corpus.len(),
        index_path.display()
    );
    if failed > 0 {
        eprintln!(
            "{failed} fixture(s) failed to build; the bank is usable but incomplete — \
             see `build_failures` in {}",
            index_path.display()
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spaghettio_core::bus::layout::LayoutStrategy;

    #[test]
    fn variants_have_stable_machine_readable_names() {
        assert_eq!(variant_name(FixtureVariant::Plain), "plain");
        assert_eq!(variant_name(FixtureVariant::Excluded), "excluded");
        assert_eq!(variant_name(FixtureVariant::ExcludedVoid), "excluded-void");
        assert_eq!(
            variant_name(FixtureVariant::Strategy(LayoutStrategy::Pooled)),
            "strategy:Pooled"
        );
        assert_eq!(
            variant_name(FixtureVariant::Strategy(LayoutStrategy::PartitionedDecomposed)),
            "strategy:PartitionedDecomposed"
        );
    }

    /// The sweep's reader keys its compat branch on this number; bumping it
    /// without teaching the reader is the failure this pin makes visible.
    #[test]
    fn schema_version_is_the_one_the_sweep_reads() {
        assert_eq!(SCHEMA_VERSION, 2);
    }
}
