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

use sha2::{Digest, Sha256};
use spaghettio_core::blueprint;
use spaghettio_core::calibration_matrix::{build, fixtures, CalibrationFixture, FixtureVariant};
use spaghettio_core::validate::Severity;

fn usage() -> ! {
    eprintln!("usage: calibration_matrix_export <new-or-empty-bank-dir>");
    std::process::exit(2);
}

fn variant_name(variant: FixtureVariant) -> &'static str {
    match variant {
        FixtureVariant::Plain => "plain",
        FixtureVariant::Strategy(_) => "strategy",
        FixtureVariant::Excluded => "excluded",
        FixtureVariant::ExcludedVoid => "excluded-void",
    }
}

fn entry(
    fixture: &CalibrationFixture,
    built: &spaghettio_core::calibration_matrix::BuiltFixture,
    bp: &str,
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
    let hash = format!("{:x}", Sha256::digest(bp.as_bytes()));
    serde_json::json!({
        "label": fixture.name,
        "target": fixture.item,
        "rate": fixture.rate,
        "machine": fixture.machine,
        "belt_tier": fixture.belt_tier,
        "inputs": fixture.inputs,
        "excluded_recipes": fixture.excluded,
        "variant": variant_name(fixture.variant),
        "blueprint_sha256": hash,
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

    let mut entries = Vec::new();
    for fixture in fixtures() {
        let built = build(&fixture).unwrap_or_else(|e| panic!("{}: {e}", fixture.name));
        let (bp, manifest) = blueprint::export_with_manifest_validated(
            &built.layout,
            &built.solver_result,
            fixture.name,
            &built.issues,
        );
        let dir = root.join(fixture.name);
        std::fs::create_dir(&dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
        std::fs::write(dir.join("bp.txt"), &bp)
            .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("bp.txt").display()));
        std::fs::write(
            dir.join("manifest-real.json"),
            serde_json::to_string_pretty(&manifest).expect("manifest serializes"),
        )
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("manifest-real.json").display()));
        println!(
            "{:<62} {:>5} entities {:>4}x{:<4}",
            fixture.name,
            built.layout.entities.len(),
            built.layout.width,
            built.layout.height,
        );
        entries.push(entry(&fixture, &built, &bp));
    }
    let index = serde_json::json!({
        "schema_version": 1,
        "purpose": "current-generation meter-vs-Factorio calibration matrix",
        "fixture_count": entries.len(),
        "fixtures": entries,
    });
    let index_path = root.join("matrix.json");
    std::fs::write(
        &index_path,
        serde_json::to_string_pretty(&index).expect("matrix serializes"),
    )
    .unwrap_or_else(|e| panic!("write {}: {e}", index_path.display()));
    println!(
        "wrote {} fixtures and {}",
        fixtures().len(),
        index_path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_have_stable_machine_readable_names() {
        assert_eq!(variant_name(FixtureVariant::Plain), "plain");
        assert_eq!(variant_name(FixtureVariant::Excluded), "excluded");
        assert_eq!(variant_name(FixtureVariant::ExcludedVoid), "excluded-void");
    }
}
