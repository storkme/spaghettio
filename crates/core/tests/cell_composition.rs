//! RFC-051 Phase A: consumers of the production cell-composition path
//! (`spaghettio_core::bus::cells`), which was lifted verbatim from this
//! file's Phase-1 harness (PR #365). The gates now double as the
//! Phase-A PARITY proof: dimensions and entity counts are pinned to the
//! post-review-fold Phase-1 results — a lift that changes geometry
//! fails here, by design.
//!
//! The superseded east-feed composer and its probes (pre-#363
//! orientation) were dropped in the lift; their findings live in the
//! RFC-048 decision log.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::cells::compose::{compose_pairs_calibrated, compose_plastic_calibrated};
use spaghettio_core::bus::cells::extract::{extract_cell, generate_cell_layout};
use spaghettio_core::bus::layout;
use spaghettio_core::common::QualityTier;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;

/// PERMANENT GATE (RFC-048 Phase 1; Phase-A parity pin): EC@15/s — the
/// config the bus engine refuses (#336) — composes from
/// engine-generated cells at 0 validation errors. The 6 carried
/// warnings are validator-attribution conservatism DISPROVEN by
/// measurement (sim: 15/15 machines working, produced exactly 15.0/s —
/// see the RFC decision log 2026-07-22). Dims/entity count pinned to
/// the sim-verified artifact (110×22, 461 entities).
#[test]
fn cell_composed_ec15_zero_errors() {
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let (esr, l) = compose_pairs_calibrated(3);
    println!("calibrated EC@15: {}x{} = {} tiles, {} entities", l.width, l.height, l.width * l.height, l.entities.len());
    // Phase-A parity pins: the lift must reproduce the Phase-1 geometry
    // bit-for-bit (RFC-051 verification plan).
    assert_eq!((l.width, l.height), (110, 22), "parity: sim-verified artifact dims");
    assert_eq!(l.entities.len(), 461, "parity: sim-verified artifact entity count");
    let issues = validate::validate(&l, Some(&esr), LayoutStyle::Bus)
        .unwrap_or_else(|e| panic!("composed EC@15 must validate: {e}"));
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
    assert!(errors.is_empty(), "composed EC@15 errors: {errors:?}");
    assert!(
        issues.iter().all(|i| i.category == "inserter-item-throughput"),
        "only the sim-disproven attribution warnings are tolerated: {issues:?}"
    );
    // The 6 specific warnings were sim-adjudicated; more of the same
    // category would be NEW unadjudicated claims — trip on growth.
    assert!(issues.len() <= 6, "warning count grew past the adjudicated 6: {issues:?}");
}

/// PERMANENT GATE (RFC-048 Phase 1; Phase-A parity pin): the
/// fluid-consumer cell composes at 0 errors AND 0 warnings. Sim
/// verification PASSED post-#373 (produced 2.20/s vs 2.00 planned —
/// RFC-048 gate (a) closed in full).
#[test]
fn cell_composed_plastic_zero_issues() {
    use spaghettio_core::validate::{self, LayoutStyle};
    let (sr, comp) = compose_plastic_calibrated();
    let issues = validate::validate(&comp, Some(&sr), LayoutStyle::Bus)
        .unwrap_or_else(|e| panic!("composed plastic must validate: {e}"));
    println!("composed plastic (calibrated): {}x{}, {} entities, {} issues",
        comp.width, comp.height, comp.entities.len(), issues.len());
    assert!(issues.is_empty(), "composed plastic issues: {issues:?}");
}

/// Exploration probe (run with --nocapture): geometry of the two
/// candidate cell source layouts.
#[test]
#[ignore = "exploration probe, not a gate"]
fn probe_cell_source_geometry() {
    for (label, item, rate, inputs) in [
        ("cable", "copper-cable", 15.0, &["copper-plate"][..]),
        ("ec", "electronic-circuit", 5.0, &["iron-plate", "copper-cable"][..]),
    ] {
        let (sr, l) = generate_cell_layout(item, rate, inputs);
        println!("== {label}: {}x{}, {} entities ==", l.width, l.height, l.entities.len());
        for m in &sr.machines {
            println!("   spec {} x{:.2}", m.recipe, m.count);
        }
        for e in &l.entities {
            let edge = e.x <= 1
                || e.x >= l.width - 2
                || e.y <= 1
                || e.y >= l.height - 2;
            if edge && spaghettio_core::common::is_belt_entity(&e.name) {
                println!(
                    "   edge belt ({},{}) {} dir={:?} carries={:?} seg={:?}",
                    e.x, e.y, e.name, e.direction, e.carries, e.segment_id
                );
            }
        }
    }
}

/// Probe: extracted cells' dimensions, ports, and full belt inventory.
#[test]
#[ignore = "exploration probe, not a gate"]
fn probe_extracted_cells() {
    for (label, item, rate, inputs) in [
        ("cable", "copper-cable", 15.0, &["copper-plate"][..]),
        ("ec", "electronic-circuit", 5.0, &["iron-plate", "copper-cable"][..]),
    ] {
        let (_sr, l) = generate_cell_layout(item, rate, inputs);
        let c = extract_cell(&l);
        println!("== {label} cell: {}x{}, {} entities ==", c.width, c.height, c.entities.len());
        for p in &c.ports {
            println!("   port {} y={} {} {}", p.edge, p.y, p.item, if p.inbound { "IN" } else { "OUT" });
        }
        for e in &c.entities {
            if spaghettio_core::common::is_belt_entity(&e.name) {
                println!(
                    "   belt ({},{}) {:?} carries={:?} seg={:?}",
                    e.x, e.y, e.direction, e.carries, e.segment_id
                );
            }
        }
    }
}

/// Export the composed EC@15 layout + manifest for spaghettio-sim.
#[test]
#[ignore = "artifact producer for the sim step"]
fn export_composed_ec15_for_sim() {
    let (esr, l) = compose_pairs_calibrated(3);
    let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&l, &esr, "rfc048-ec15-composed");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/rfc048-ec15.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/rfc048-ec15.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "wrote target/tmp/rfc048-ec15.bp ({} chars) + manifest ({} boundary in / {} out)",
        bp.len(),
        l.boundary_inputs.len(),
        l.boundary_outputs.len()
    );
}

/// Gate (c): config-axis growth measurement — the EC cell at two
/// machine tiers (RFC-048 Phase-1 gate; the plan-or-hope number).
#[test]
#[ignore = "measurement probe"]
fn probe_axis_growth_machine_tier() {
    for machine in ["assembling-machine-2", "assembling-machine-3"] {
        let inputs: FxHashSet<String> =
            ["iron-plate", "copper-cable"].iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            "electronic-circuit", 5.0, &inputs, &MachinePalette::default(),
            machine, &FxHashSet::default(), QualityTier::Normal,
        ).unwrap();
        let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default()).unwrap();
        let c = extract_cell(&l);
        println!("== {machine}: cell {}x{}, {} entities ==", c.width, c.height, c.entities.len());
        for m in &sr.machines { println!("   spec {} x{:.2}", m.recipe, m.count); }
        for p in &c.ports {
            println!("   port {} ({},{}) {} {}", p.edge, p.x, p.y, p.item, if p.inbound { "IN" } else { "OUT" });
        }
    }
}

/// Fluid-consumer probe: plastic-bar cell segment structure.
#[test]
#[ignore = "exploration probe"]
fn probe_fluid_cell_geometry() {
    let (sr, l) = generate_cell_layout("plastic-bar", 2.0, &["petroleum-gas", "coal"]);
    let c = extract_cell(&l);
    println!("== plastic cell {}x{}, {} entities ==", c.width, c.height, c.entities.len());
    for m in &sr.machines { println!("   spec {} x{:.2}", m.recipe, m.count); }
    for port in &c.ports { println!("   port {} ({},{}) {} {}", port.edge, port.x, port.y, port.item, if port.inbound { "IN" } else { "OUT" }); }
    let mut segs: std::collections::BTreeSet<String> = Default::default();
    for e in &c.entities {
        if let Some(seg) = e.segment_id.as_deref() { segs.insert(format!("{seg} [{}]", e.name)); }
    }
    for s in &segs { println!("   seg {s}"); }
}

/// Artifact producer for the sim: composed plastic blueprint + manifest.
#[test]
#[ignore = "artifact producer — run explicitly when exporting for the sim"]
fn export_composed_plastic_for_sim() {
    let (sr, comp) = compose_plastic_calibrated();
    let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&comp, &sr, "rfc048-plastic-composed");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/rfc048-plastic.bp", &bp).unwrap();
    std::fs::write("target/tmp/rfc048-plastic.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    println!("wrote target/tmp/rfc048-plastic.bp + manifest");
}

#[test]
#[ignore = "probe"]
fn probe_pole_positions() {
    let (_sr, l) = compose_pairs_calibrated(3);
    for e in &l.entities {
        if e.name.contains("pole") {
            println!("pole ({},{})", e.x, e.y);
        }
    }
}

#[test]
#[ignore = "probe"]
fn probe_plastic_pipes() {
    let (_sr, l) = generate_cell_layout("plastic-bar", 2.0, &["petroleum-gas", "coal"]);
    let c = extract_cell(&l);
    for e in &c.entities {
        if e.name.contains("pipe") {
            println!("{} ({},{}) dir={:?} io={:?} seg={:?}", e.name, e.x, e.y, e.direction, e.io_type, e.segment_id);
        }
    }
}

/// Attribution control kept from the #364 arc: the ENGINE's own plastic
/// layout through the sim path.
#[test]
#[ignore = "artifact producer"]
fn export_engine_plastic_for_sim() {
    let (sr, l) = generate_cell_layout("plastic-bar", 2.0, &["petroleum-gas", "coal"]);
    let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&l, &sr, "rfc048-engine-plastic");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/rfc048-engine-plastic.bp", &bp).unwrap();
    std::fs::write("target/tmp/rfc048-engine-plastic.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    println!("wrote engine plastic artifacts ({} boundary in)", l.boundary_inputs.len());
}

/// Phase-B dev probe: the chain auto-placer on the two dev fixtures.
#[test]
#[ignore = "exploration probe while the auto-placer stabilizes"]
fn probe_chain_autoplace() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    for (label, item, rate, inputs) in [
        ("ec15", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"][..]),
        ("ac1", "advanced-circuit", 1.0, &["iron-plate", "copper-plate", "plastic-bar"][..]),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item, rate, &inputs_set, &MachinePalette::default(),
            "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
        ).unwrap();
        println!("== {label}: {} specs ==", sr.machines.len());
        for m in &sr.machines { println!("   {} x{:.2} out {:.2}/s", m.recipe, m.count, m.outputs[0].rate); }
        match compose_chain(&sr) {
            Ok(l) => {
                println!("   composed {}x{} = {} tiles, {} entities", l.width, l.height, l.width * l.height, l.entities.len());
                match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
                    Ok(issues) => {
                        let e = issues.iter().filter(|i| i.severity == Severity::Error).count();
                        println!("   validation: {} errors / {} issues", e, issues.len());
                        for i in issues.iter().take(15) {
                            println!("     [{:?}] {} {}", i.severity, i.category, i.message);
                        }
                    }
                    Err(er) => {
                        for line in format!("{er}").lines().take(12) { println!("     {line}"); }
                    }
                }
            }
            Err(e) => println!("   REFUSED: {e}"),
        }
    }
}

/// Artifact producers for the chain auto-placer's sim runs (Phase B/C).
#[test]
#[ignore = "artifact producer"]
fn export_chain_fixtures_for_sim() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    for (label, item, rate, inputs) in [
        ("chain-ec15", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"][..]),
        ("chain-ac1", "advanced-circuit", 1.0, &["iron-plate", "copper-plate", "plastic-bar"][..]),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item, rate, &inputs_set, &MachinePalette::default(),
            "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
        ).unwrap();
        let l = compose_chain(&sr).unwrap();
        let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&l, &sr, label);
        std::fs::create_dir_all("target/tmp").unwrap();
        std::fs::write(format!("target/tmp/{label}.bp"), &bp).unwrap();
        std::fs::write(format!("target/tmp/{label}.manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
        println!("wrote target/tmp/{label}.bp ({} boundary in / {} out)", l.boundary_inputs.len(), l.boundary_outputs.len());
    }
}

/// Phase-B differential scoreboard (kill-3 evidence): composed vs bus
/// on every chain-eligible ladder fixture. Prints errors / warnings /
/// area / refusals per path.
#[test]
#[ignore = "measurement probe — output goes to the RFC decision log"]
fn probe_differential_scoreboard() {
    use spaghettio_core::bus::cells::chain::{chain_eligible, compose_chain};
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let fixtures: &[(&str, &str, f64, &[&str])] = &[
        ("gear15", "iron-gear-wheel", 15.0, &["iron-plate"]),
        ("ec5", "electronic-circuit", 5.0, &["iron-plate", "copper-plate"]),
        ("ec15", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"]),
        ("ec30", "electronic-circuit", 30.0, &["iron-plate", "copper-plate"]),
        ("ac1", "advanced-circuit", 1.0, &["iron-plate", "copper-plate", "plastic-bar"]),
        ("ac2", "advanced-circuit", 2.0, &["iron-plate", "copper-plate", "plastic-bar"]),
    ];
    for (label, item, rate, inputs) in fixtures {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item, *rate, &inputs_set, &MachinePalette::default(),
            "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
        ).unwrap();
        let bus = std::panic::catch_unwind(|| layout::build_bus_layout(&sr, layout::LayoutOptions::default()));
        let bus_desc = match &bus {
            Ok(Ok(l)) => match validate::validate(l, Some(&sr), LayoutStyle::Bus) {
                Ok(issues) => {
                    let e = issues.iter().filter(|i| i.severity == Severity::Error).count();
                    format!("{}x{}={} tiles, {} errors / {} warnings", l.width, l.height, l.width * l.height, e, issues.len() - e)
                }
                Err(er) => format!("validate() Err: {}", format!("{er}").lines().next().unwrap_or("")),
            },
            Ok(Err(e)) => format!("REFUSED: {}", e.lines().next().unwrap_or("")),
            Err(_) => "PANICKED".into(),
        };
        let comp_desc = match chain_eligible(&sr) {
            Err(e) => format!("INELIGIBLE: {e}"),
            Ok(()) => match compose_chain(&sr) {
                Ok(l) => match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
                    Ok(issues) => {
                        let e = issues.iter().filter(|i| i.severity == Severity::Error).count();
                        format!("{}x{}={} tiles, {} errors / {} warnings", l.width, l.height, l.width * l.height, e, issues.len() - e)
                    }
                    Err(er) => format!("validate() Err: {}", format!("{er}").lines().next().unwrap_or("")),
                },
                Err(e) => format!("REFUSED: {e}"),
            },
        };
        println!("{label:8} bus:      {bus_desc}");
        println!("{label:8} composed: {comp_desc}");
    }
}

/// PERMANENT GATE (RFC-051 Phase B): with the flag ON, the decomposition
/// search resolves EC@15-from-plates — the config the bus engine refuses
/// outright (#336) — via the cell-composed candidate, at 0 errors with
/// only the sim-adjudicated warning category. With the flag OFF
/// (default) the refusal stands (inertness: the bus path is untouched).
#[test]
fn cell_candidate_resolves_ec15_refusal() {
    use spaghettio_core::bus::cells::CellComposition;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> =
        ["iron-plate", "copper-plate"].iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit", 15.0, &inputs, &MachinePalette::default(),
        "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
    ).unwrap();

    // Flag OFF: the refusal stands.
    let off = layout::build_bus_layout(&sr, layout::LayoutOptions::default());
    assert!(off.is_err(), "flag-Off must preserve the bus refusal");

    // Flag ON: the composed candidate wins and validates clean.
    let opts = layout::LayoutOptions {
        cell_composition: CellComposition::Candidate,
        ..Default::default()
    };
    let l = layout::build_bus_layout(&sr, opts).expect("candidate must resolve the refusal");
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).unwrap();
    let errors = issues.iter().filter(|i| i.severity == Severity::Error).count();
    assert_eq!(errors, 0, "composed candidate errors: {issues:?}");
    assert!(issues.iter().all(|i| i.category == "inserter-item-throughput"),
        "only the adjudicated category tolerated: {issues:?}");
}
