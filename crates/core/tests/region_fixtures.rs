//! Region-solver fixture regression harness.
//!
//! Parallel to `sat_fixtures.rs` but exercises the full region solver
//! (`junction_solver::solve_crossing`) via the `fixture::replay_region_fixture`
//! helper, not just SAT in isolation. Each fixture captures the entire
//! solve-crossing argument set + an `expected.mode` outcome.
//!
//! Run with:
//!   cargo test --manifest-path crates/core/Cargo.toml --test region_fixtures
//!
//! See `tests/region_fixtures/README.md` for the schema, the capture
//! workflow (via `FUCKTORIO_DUMP_REGION_FIXTURE`), and the promote-to-
//! committed-fixture checklist.
//!
//! All fixture failures are accumulated and reported together at the end.

use fucktorio_core::fixture::{replay_region_fixture, RegionFixture};
use std::path::Path;

#[test]
fn region_fixtures() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("region_fixtures");

    // Skip gracefully if the directory isn't created yet — lets the
    // test compile and run immediately after adding the harness, before
    // any fixtures have been captured.
    if !fixtures_dir.exists() {
        eprintln!(
            "region_fixtures: directory {} does not exist, nothing to test",
            fixtures_dir.display()
        );
        return;
    }

    let mut fixture_paths: Vec<_> = std::fs::read_dir(&fixtures_dir)
        .unwrap_or_else(|e| {
            panic!("cannot read region_fixtures dir {}: {e}", fixtures_dir.display())
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    fixture_paths.sort();

    if fixture_paths.is_empty() {
        eprintln!("region_fixtures: no *.json files found, nothing to test");
        return;
    }

    let mut failures: Vec<String> = Vec::new();
    let mut passed = 0u32;

    for path in &fixture_paths {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

        let fixture: RegionFixture = serde_json::from_str(&raw).unwrap_or_else(|e| {
            panic!("cannot parse {}: {e}", path.display())
        });

        if fixture.version != 1 {
            failures.push(format!(
                "{}: unsupported fixture version {} (only version 1 supported)",
                fixture.name, fixture.version
            ));
            continue;
        }

        let result = replay_region_fixture(&fixture);

        match fixture.expected.mode.as_str() {
            "solve" => match result.cost {
                None => {
                    failures.push(format!(
                        "{} ({}): expected solve, got None{}",
                        fixture.name,
                        filename,
                        if result.capped { " (capped)" } else { "" }
                    ));
                }
                Some(cost) => {
                    if let Some(max_cost) = fixture.expected.max_cost {
                        if cost > max_cost {
                            failures.push(format!(
                                "{} ({}): solver cost {cost} exceeds fixture max_cost {max_cost}",
                                fixture.name, filename
                            ));
                            continue;
                        }
                    }
                    let gap_note = match fixture.expected.optimal_cost {
                        Some(opt) if cost > opt => {
                            format!(" / optimal {opt} / gap {}", cost - opt)
                        }
                        Some(opt) => format!(" / optimal {opt} / gap 0"),
                        None => String::new(),
                    };
                    eprintln!(
                        "  PASS  {} ({}) — solved with {} entities, cost {}{}",
                        fixture.name,
                        filename,
                        result.entities.len(),
                        cost,
                        gap_note,
                    );
                    passed += 1;
                }
            },
            "capped" => {
                if let Some(cost) = result.cost {
                    failures.push(format!(
                        "{} ({}): expected capped, got solution with cost {cost}",
                        fixture.name, filename,
                    ));
                } else if !result.capped {
                    failures.push(format!(
                        "{} ({}): expected capped, got None without any GrowthCapped event",
                        fixture.name, filename
                    ));
                } else {
                    eprintln!(
                        "  PASS  {} ({}) — correctly capped",
                        fixture.name, filename
                    );
                    passed += 1;
                }
            }
            "unsatisfiable" => {
                if let Some(cost) = result.cost {
                    failures.push(format!(
                        "{} ({}): expected unsatisfiable, got solution with cost {cost}",
                        fixture.name, filename,
                    ));
                } else if result.capped {
                    failures.push(format!(
                        "{} ({}): expected unsatisfiable, but growth-cap event fired",
                        fixture.name, filename
                    ));
                } else {
                    eprintln!(
                        "  PASS  {} ({}) — correctly unsatisfiable",
                        fixture.name, filename
                    );
                    passed += 1;
                }
            }
            other => {
                failures.push(format!(
                    "{} ({}): unknown expected.mode {:?} — must be \"solve\", \
                     \"capped\", or \"unsatisfiable\"",
                    fixture.name, filename, other
                ));
            }
        }
    }

    eprintln!(
        "\nregion_fixtures: {passed} passed, {} failed (from {} fixture files)",
        failures.len(),
        fixture_paths.len()
    );

    assert!(
        failures.is_empty(),
        "{} fixture failure(s):\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
