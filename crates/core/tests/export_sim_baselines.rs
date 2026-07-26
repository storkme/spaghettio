//! Export the five blessed sim baselines as reproducible fixtures.
//!
//! ```bash
//! cargo test --manifest-path crates/core/Cargo.toml \
//!     --test export_sim_baselines -- --ignored --nocapture
//! ```
//!
//! # Why this exists
//!
//! `crates/sim-harness/baselines/` holds five real-Factorio measurements
//! — gear10, ec10, automation/logistic/military science — but the
//! blueprints they were measured from were produced **ad hoc** and never
//! committed to a test. So from a clean checkout they cannot be
//! regenerated, which means RFC-054's corpus replay could not include
//! them and its kill criterion was evaluated over a corpus five configs
//! smaller than the one the repo actually owns.
//!
//! A corpus that silently shrinks is a kill criterion that silently
//! weakens. This closes that.
//!
//! # FINDING (2026-07-25): the blessed baselines are STALE
//!
//! Each baseline JSON records the `entities` count of the layout it was
//! measured on, so regenerating and comparing is a free check on whether
//! the config here is right. Run at the time of writing:
//!
//! ```text
//! gear10                   406 entities (baseline 428)
//! ec10                     671 entities (baseline 805)
//! automation-science-pack  281 entities (baseline 196)
//! logistic-science-pack    636 entities (baseline 593)
//! military-science-pack   1002 entities (baseline 919)
//! ```
//!
//! The configs are **right**: `docs/status.md`'s 2026-07-21 six-pack
//! gauntlet independently records automation at **281** and military at
//! **1002**, matching exactly. It is the *baselines* that no longer
//! describe what the engine builds — they were blessed 2026-07-22, and
//! RFC-044/046/047, #385, #431, #434 and #448 have all moved geometry
//! since.
//!
//! **This is silent drift of the same class RFC-051 built geometry-hashed
//! keys to prevent.** The cell-sim registry keys each verdict on a hash of
//! the generated entity list precisely so an engine change invalidates it
//! automatically; `crates/sim-harness/baselines/` has no such key, so its
//! numbers decayed without anyone noticing.
//!
//! Consequence for RFC-054: these five cannot extend the KC1 corpus until
//! they are **re-blessed against current geometry**, which needs Factorio.
//! The fixtures are still exported here — a layout whose plan rate is known
//! is a useful meter target even without a blessed measurement — but they
//! are NOT added to the replay corpus, because comparing today's meter
//! against a measurement of a different blueprint would be worse than
//! having no data at all.
//!
//! The comparison below therefore **reports** rather than asserts. Making
//! it a hard failure would leave a permanently-red test for a reason that
//! is not this test's fault and cannot be fixed from here.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout;
use spaghettio_core::{blueprint, solver};

struct Fixture {
    /// Must match the `label` in the baseline JSON.
    label: &'static str,
    target: &'static str,
    rate: f64,
    machine: &'static str,
    /// Entity count recorded in the blessed baseline.
    expect_entities: usize,
}

/// Raw Nauvis inputs, matching `science_gauntlet.rs`.
const NAUVIS_INPUTS: &[&str] = &[
    "iron-ore",
    "copper-ore",
    "coal",
    "stone",
    "crude-oil",
    "water",
];

/// Configs mirror `science_gauntlet.rs`'s case table (machine tiers
/// included — logistic and military are AM2 because AM1's two ingredient
/// slots cannot craft the Space Age inserter chain).
const FIXTURES: &[Fixture] = &[
    Fixture {
        label: "gear10",
        target: "iron-gear-wheel",
        rate: 10.0,
        machine: "assembling-machine-2",
        expect_entities: 428,
    },
    Fixture {
        label: "ec10",
        target: "electronic-circuit",
        rate: 10.0,
        machine: "assembling-machine-2",
        expect_entities: 805,
    },
    Fixture {
        label: "automation-science-pack",
        target: "automation-science-pack",
        rate: 1.0,
        machine: "assembling-machine-1",
        expect_entities: 196,
    },
    Fixture {
        label: "logistic-science-pack",
        target: "logistic-science-pack",
        rate: 1.0,
        machine: "assembling-machine-2",
        expect_entities: 593,
    },
    Fixture {
        label: "military-science-pack",
        target: "military-science-pack",
        rate: 1.0,
        machine: "assembling-machine-2",
        expect_entities: 919,
    },
];

#[test]
#[ignore = "fixture export; run explicitly"]
fn export_sim_baseline_fixtures() {
    std::fs::create_dir_all("target/tmp").unwrap();
    let inputs: FxHashSet<String> = NAUVIS_INPUTS.iter().map(|s| s.to_string()).collect();

    let mut mismatches = Vec::new();
    for f in FIXTURES {
        let sr = match solver::solve(f.target, f.rate, &inputs, f.machine) {
            Ok(sr) => sr,
            Err(e) => {
                mismatches.push(format!("{}: solve failed: {e:?}", f.label));
                continue;
            }
        };
        let lr = match layout::build_bus_layout(&sr, layout::LayoutOptions::default()) {
            Ok(lr) => lr,
            Err(e) => {
                mismatches.push(format!("{}: layout failed: {e:?}", f.label));
                continue;
            }
        };
        let (bp, manifest) = blueprint::export_with_manifest(&lr, &sr, f.label);
        std::fs::write(format!("target/tmp/{}.bp", f.label), &bp).unwrap();
        std::fs::write(
            format!("target/tmp/{}.manifest.json", f.label),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let got = lr.entities.len();
        let tag = if got == f.expect_entities { "ok" } else { "MISMATCH" };
        println!(
            "{:<26} {got:>5} entities (baseline {:>5}) {tag}",
            f.label, f.expect_entities
        );
        if got != f.expect_entities {
            mismatches.push(format!(
                "{}: {got} entities, baseline says {}",
                f.label, f.expect_entities
            ));
        }
    }

    if !mismatches.is_empty() {
        println!(
            "\nDRIFT vs blessed baselines (see module docs — the baselines are \
             stale, not these configs):\n  {}",
            mismatches.join("\n  ")
        );
    }
    // Assert only that every fixture actually built. Geometry drift is
    // reported above, deliberately not failed here.
    let failures: Vec<&String> = mismatches
        .iter()
        .filter(|m| m.contains("failed"))
        .collect();
    assert!(failures.is_empty(), "fixtures failed to build: {failures:?}");
}
