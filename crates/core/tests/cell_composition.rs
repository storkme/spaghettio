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

use rustc_hash::{FxHashMap, FxHashSet};
use spaghettio_core::bus::cells::compose::{compose_pairs_calibrated, compose_plastic_calibrated};
use spaghettio_core::bus::cells::extract::{extract_cell, generate_cell_layout};
use spaghettio_core::bus::layout;
use spaghettio_core::common::QualityTier;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;

/// PERMANENT GATE (RFC-048 Phase 1; Phase-A parity pin): EC@15/s — the
/// config the bus engine refuses (#336) — composes from
/// engine-generated cells at 0 validation errors. The 6 carried
/// warnings measured harmless under the PRE-#378 harness (15/15
/// working, 15.0/s exact) — but that run realized researched inserter
/// bonuses; under tech-state parity (declared capacity 0) the same
/// warnings are REAL long-handed-inserter shortfalls (#383). The
/// warnings stay tolerated at the validator level; the fix is RFC-049
/// Phase 3 inserter sizing (#381), after which they should vanish.
/// Dims/entity count pinned to the sim artifact (110×22, 461 entities).
#[test]
fn cell_composed_ec15_zero_errors() {
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let (esr, l) = compose_pairs_calibrated(3);
    println!(
        "calibrated EC@15: {}x{} = {} tiles, {} entities",
        l.width,
        l.height,
        l.width * l.height,
        l.entities.len()
    );
    // Phase-A parity pins: the lift must reproduce the Phase-1 geometry
    // bit-for-bit (RFC-051 verification plan).
    assert_eq!(
        (l.width, l.height),
        (110, 22),
        "parity: sim-verified artifact dims"
    );
    assert_eq!(
        l.entities.len(),
        461,
        "parity: sim-verified artifact entity count"
    );
    let issues = validate::validate(&l, Some(&esr), LayoutStyle::Bus)
        .unwrap_or_else(|e| panic!("composed EC@15 must validate: {e}"));
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "composed EC@15 errors: {errors:?}");
    assert!(
        issues
            .iter()
            .all(|i| i.category == "inserter-item-throughput"),
        "only the sim-disproven attribution warnings are tolerated: {issues:?}"
    );
    // The 6 specific warnings were sim-adjudicated; more of the same
    // category would be NEW unadjudicated claims — trip on growth.
    assert!(
        issues.len() <= 6,
        "warning count grew past the adjudicated 6: {issues:?}"
    );
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
    println!(
        "composed plastic (calibrated): {}x{}, {} entities, {} issues",
        comp.width,
        comp.height,
        comp.entities.len(),
        issues.len()
    );
    assert!(issues.is_empty(), "composed plastic issues: {issues:?}");
}

/// Exploration probe (run with --nocapture): geometry of the two
/// candidate cell source layouts.
#[test]
#[ignore = "exploration probe, not a gate"]
fn probe_cell_source_geometry() {
    for (label, item, rate, inputs) in [
        ("cable", "copper-cable", 15.0, &["copper-plate"][..]),
        (
            "ec",
            "electronic-circuit",
            5.0,
            &["iron-plate", "copper-cable"][..],
        ),
    ] {
        let (sr, l) = generate_cell_layout(item, rate, inputs);
        println!(
            "== {label}: {}x{}, {} entities ==",
            l.width,
            l.height,
            l.entities.len()
        );
        for m in &sr.machines {
            println!("   spec {} x{:.2}", m.recipe, m.count);
        }
        for e in &l.entities {
            let edge = e.x <= 1 || e.x >= l.width - 2 || e.y <= 1 || e.y >= l.height - 2;
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
        (
            "ec",
            "electronic-circuit",
            5.0,
            &["iron-plate", "copper-cable"][..],
        ),
    ] {
        let (_sr, l) = generate_cell_layout(item, rate, inputs);
        let c = extract_cell(&l);
        println!(
            "== {label} cell: {}x{}, {} entities ==",
            c.width,
            c.height,
            c.entities.len()
        );
        for p in &c.ports {
            println!(
                "   port {} y={} {} {}",
                p.edge,
                p.y,
                p.item,
                if p.inbound { "IN" } else { "OUT" }
            );
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
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&l, &esr, "rfc048-ec15-composed");
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
        let inputs: FxHashSet<String> = ["iron-plate", "copper-cable"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            "electronic-circuit",
            5.0,
            &inputs,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default()).unwrap();
        let c = extract_cell(&l);
        println!(
            "== {machine}: cell {}x{}, {} entities ==",
            c.width,
            c.height,
            c.entities.len()
        );
        for m in &sr.machines {
            println!("   spec {} x{:.2}", m.recipe, m.count);
        }
        for p in &c.ports {
            println!(
                "   port {} ({},{}) {} {}",
                p.edge,
                p.x,
                p.y,
                p.item,
                if p.inbound { "IN" } else { "OUT" }
            );
        }
    }
}

/// Fluid-consumer probe: plastic-bar cell segment structure.
#[test]
#[ignore = "exploration probe"]
fn probe_fluid_cell_geometry() {
    let (sr, l) = generate_cell_layout("plastic-bar", 2.0, &["petroleum-gas", "coal"]);
    let c = extract_cell(&l);
    println!(
        "== plastic cell {}x{}, {} entities ==",
        c.width,
        c.height,
        c.entities.len()
    );
    for m in &sr.machines {
        println!("   spec {} x{:.2}", m.recipe, m.count);
    }
    for port in &c.ports {
        println!(
            "   port {} ({},{}) {} {}",
            port.edge,
            port.x,
            port.y,
            port.item,
            if port.inbound { "IN" } else { "OUT" }
        );
    }
    let mut segs: std::collections::BTreeSet<String> = Default::default();
    for e in &c.entities {
        if let Some(seg) = e.segment_id.as_deref() {
            segs.insert(format!("{seg} [{}]", e.name));
        }
    }
    for s in &segs {
        println!("   seg {s}");
    }
}

/// Artifact producer for the sim: composed plastic blueprint + manifest.
#[test]
#[ignore = "artifact producer — run explicitly when exporting for the sim"]
fn export_composed_plastic_for_sim() {
    let (sr, comp) = compose_plastic_calibrated();
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&comp, &sr, "rfc048-plastic-composed");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/rfc048-plastic.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/rfc048-plastic.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
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
            println!(
                "{} ({},{}) dir={:?} io={:?} seg={:?}",
                e.name, e.x, e.y, e.direction, e.io_type, e.segment_id
            );
        }
    }
}

/// Attribution control kept from the #364 arc: the ENGINE's own plastic
/// layout through the sim path.
#[test]
#[ignore = "artifact producer"]
fn export_engine_plastic_for_sim() {
    let (sr, l) = generate_cell_layout("plastic-bar", 2.0, &["petroleum-gas", "coal"]);
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&l, &sr, "rfc048-engine-plastic");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/rfc048-engine-plastic.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/rfc048-engine-plastic.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "wrote engine plastic artifacts ({} boundary in)",
        l.boundary_inputs.len()
    );
}

/// Phase-B dev probe: the chain auto-placer on the two dev fixtures.
#[test]
#[ignore = "exploration probe while the auto-placer stabilizes"]
fn probe_chain_autoplace() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    for (label, item, rate, inputs) in [
        (
            "ec15",
            "electronic-circuit",
            15.0,
            &["iron-plate", "copper-plate"][..],
        ),
        (
            "ac1",
            "advanced-circuit",
            1.0,
            &["iron-plate", "copper-plate", "plastic-bar"][..],
        ),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            rate,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        println!("== {label}: {} specs ==", sr.machines.len());
        for m in &sr.machines {
            println!(
                "   {} x{:.2} out {:.2}/s",
                m.recipe, m.count, m.outputs[0].rate
            );
        }
        match compose_chain(&sr) {
            Ok(l) => {
                println!(
                    "   composed {}x{} = {} tiles, {} entities",
                    l.width,
                    l.height,
                    l.width * l.height,
                    l.entities.len()
                );
                match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
                    Ok(issues) => {
                        let e = issues
                            .iter()
                            .filter(|i| i.severity == Severity::Error)
                            .count();
                        println!("   validation: {} errors / {} issues", e, issues.len());
                        for i in issues.iter().take(15) {
                            println!("     [{:?}] {} {}", i.severity, i.category, i.message);
                        }
                    }
                    Err(er) => {
                        for line in format!("{er}").lines().take(12) {
                            println!("     {line}");
                        }
                    }
                }
            }
            Err(e) => println!("   REFUSED: {e}"),
        }
    }
}

/// How a sim fixture's geometry is composed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Compose {
    /// `compose_chain_with_capacity` at the row's `geo_cap`.
    Chain,
    /// `compose_mega_calibrated` — inserter capacity is not a parameter
    /// of that path, so `geo_cap` does not apply.
    MegaCell,
}

/// **Every** sim fixture, as one source of truth.
///
/// `geo_cap` is the capacity the GEOMETRY is built at, and it is a
/// property of the frozen measurement rather than a knob. Entries
/// measured before #431 moved the engine default to L2 are L0 geometry
/// and must stay L0 for their committed numbers to mean anything;
/// `chem5` was blessed after the flip and is genuinely L2. Re-blessing
/// is what changes a row, never an ambient default moving underneath it.
///
/// `levels` are the DECLARED `-dN` worlds to export, which select the
/// harness world and not the geometry. Empty means the fixture exports
/// once under its bare label.
struct SimFixture {
    label: &'static str,
    target: &'static str,
    rate: f64,
    inputs: &'static [&'static str],
    compose: Compose,
    geo_cap: u8,
    levels: &'static [u8],
}

const SIM_FIXTURES: &[SimFixture] = &[
    SimFixture {
        label: "chain-ac1",
        target: "advanced-circuit",
        rate: 1.0,
        inputs: &["iron-plate", "copper-plate", "plastic-bar"],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[0],
    },
    SimFixture {
        label: "chain-ec15",
        target: "electronic-circuit",
        rate: 15.0,
        inputs: &["iron-plate", "copper-plate"],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[1, 2, 3, 5, 7],
    },
    SimFixture {
        label: "chain-ec30",
        target: "electronic-circuit",
        rate: 30.0,
        inputs: &["iron-plate", "copper-plate"],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[1, 2, 3, 5, 7],
    },
    SimFixture {
        label: "chain-mil5ore",
        target: "military-science-pack",
        rate: 5.0,
        inputs: &["iron-ore", "copper-ore", "stone", "coal"],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[0, 2, 3, 7],
    },
    SimFixture {
        label: "chain-mil5plates",
        target: "military-science-pack",
        rate: 5.0,
        inputs: &[
            "iron-plate",
            "copper-plate",
            "steel-plate",
            "stone-brick",
            "coal",
        ],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[0, 2],
    },
    // Mega chains. Same `compose_chain` path as the rows above, so they
    // carried the identical ambient-default defect; they export once
    // under a bare label rather than at declared levels.
    SimFixture {
        label: "mega-chain-ac2raw",
        target: "advanced-circuit",
        rate: 2.0,
        inputs: &["iron-ore", "copper-ore", "crude-oil", "water", "coal"],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[],
    },
    SimFixture {
        label: "mega-chain-chem5raw",
        target: "chemical-science-pack",
        rate: 5.0,
        inputs: &[
            "iron-ore",
            "copper-ore",
            "crude-oil",
            "water",
            "coal",
            "iron-plate",
            "copper-plate",
            "steel-plate",
        ],
        compose: Compose::Chain,
        geo_cap: 2,
        levels: &[],
    },
    // Not registry-blessed: their measurements live in #453 (USP@2,
    // -57.0%) and #437 (PU@4, -27.3%), both recorded before #431. Pinned
    // to L0 so those recorded numbers keep describing this geometry.
    SimFixture {
        label: "mega-chain-usp2raw",
        target: "utility-science-pack",
        rate: 2.0,
        inputs: &[
            "iron-ore",
            "copper-ore",
            "crude-oil",
            "water",
            "coal",
            "stone",
        ],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[],
    },
    SimFixture {
        label: "mega-chain-pu4raw",
        target: "processing-unit",
        rate: 4.0,
        inputs: &["iron-ore", "copper-ore", "crude-oil", "water", "coal"],
        compose: Compose::Chain,
        geo_cap: 0,
        levels: &[],
    },
    // Mega CELLS: a different composer, unaffected by the capacity
    // default, but covered here so the gate has no blind spot.
    SimFixture {
        label: "mega-plastic2",
        target: "plastic-bar",
        rate: 2.0,
        inputs: &["crude-oil", "water", "coal"],
        compose: Compose::MegaCell,
        geo_cap: 0,
        levels: &[],
    },
    SimFixture {
        label: "mega-sulfur2",
        target: "sulfur",
        rate: 2.0,
        inputs: &["crude-oil", "water"],
        compose: Compose::MegaCell,
        geo_cap: 0,
        levels: &[],
    },
];

impl SimFixture {
    /// Compose this fixture's geometry exactly as its exporter does.
    fn compose_layout(&self) -> spaghettio_core::models::LayoutResult {
        let inputs: FxHashSet<String> = self.inputs.iter().map(|s| s.to_string()).collect();
        match self.compose {
            Compose::MegaCell => {
                spaghettio_core::bus::cells::mega::compose_mega_calibrated(
                    self.target,
                    self.rate,
                    self.inputs,
                )
                .unwrap_or_else(|e| panic!("{}: mega cell must compose: {e}", self.label))
                .1
            }
            Compose::Chain => {
                let sr = solver::solve_with_palette_exclusions_and_quality(
                    self.target,
                    self.rate,
                    &inputs,
                    &MachinePalette::default(),
                    "assembling-machine-3",
                    &FxHashSet::default(),
                    QualityTier::Normal,
                )
                .unwrap_or_else(|e| panic!("{}: must solve: {e:?}", self.label));
                spaghettio_core::bus::cells::chain::compose_chain_with_capacity(&sr, self.geo_cap)
                    .unwrap_or_else(|e| panic!("{}: chain must compose: {e}", self.label))
            }
        }
    }

    fn find(label: &str) -> &'static SimFixture {
        SIM_FIXTURES
            .iter()
            .find(|f| f.label == label)
            .unwrap_or_else(|| panic!("no SIM_FIXTURES row for {label}"))
    }
}

/// PERMANENT GATE: the geometry the sim fixture EXPORTERS actually write
/// must match a registered hash.
///
/// `cell_registry_hashes_current` re-derives geometry through
/// `compose_chain_with_capacity` and so stayed green while the exporters
/// — a *separate* code path calling bare `compose_chain` — silently
/// drifted to the ambient default after #431. Two paths to one artifact,
/// one of them checked. This gate watches the path that produces the
/// bytes the sim and the meter actually consume.
///
/// **Every registry entry must have a [`SIM_FIXTURES`] row.** There is no
/// "no matching row, skip" arm on purpose: that arm is what let the four
/// mega-chain exporters keep the identical defect after the first version
/// of this gate landed, because a coverage gap was indistinguishable from
/// a deliberate exclusion.
/// Which registry pins have moved, all of them, without aborting on the first.
///
/// Deliberately a separate probe rather than a flag on the gate above: a real
/// gate that turns into a no-op when an environment variable is set is one
/// stray CI export away from protecting nothing.
#[test]
#[ignore = "survey — re-blessing aid, reports every moved pin"]
fn probe_registry_pin_survey() {
    use spaghettio_core::bus::cells::registry::{entries, geometry_hash};
    for e in entries() {
        let candidates: Vec<(&str, String)> = SIM_FIXTURES
            .iter()
            .filter(|f| f.target == e.target && (f.rate - e.rate).abs() < 1e-9)
            .map(|f| (f.label, format!("{:016x}", geometry_hash(&f.compose_layout()))))
            .collect();
        let ok = candidates.iter().any(|(_, h)| *h == e.geometry_hash);
        println!(
            "{:<24}@{:<5} {}  blessed={} fresh={:?}",
            e.target, e.rate, if ok { "OK   " } else { "MOVED" }, e.geometry_hash, candidates
        );
    }
}

#[test]
fn chain_fixture_geometry_matches_registry() {
    use spaghettio_core::bus::cells::registry::{entries, geometry_hash};
    // Direction is registry -> fixtures. Several fixtures share a
    // (target, rate) key with only one blessed (mil5 ore vs plates), so
    // "every fixture is registered" is false by construction; the real
    // invariant is that every registered geometry is still reproducible
    // by the table the exporters write from.
    for e in entries() {
        let candidates: Vec<(&str, String)> = SIM_FIXTURES
            .iter()
            .filter(|f| f.target == e.target && (f.rate - e.rate).abs() < 1e-9)
            .map(|f| {
                (
                    f.label,
                    format!("{:016x}", geometry_hash(&f.compose_layout())),
                )
            })
            .collect();
        assert!(
            !candidates.is_empty(),
            "{}@{}: registry entry has no SIM_FIXTURES row, so no gate covers the exporter \
             that writes it — add the row. A silent skip here is exactly how the mega-chain \
             exporters kept the ambient-default defect.",
            e.target,
            e.rate
        );
        assert!(
            candidates.iter().any(|(_, h)| *h == e.geometry_hash),
            "{}@{}: registered geometry {} is no longer produced by any sim fixture at its \
             blessed capacity (fresh: {:?}). The exporter would write a DIFFERENT factory \
             under the same label, and every sim/meter number taken against this baseline \
             would silently compare two layouts — re-bless deliberately, never ignore.",
            e.target,
            e.rate,
            e.geometry_hash,
            candidates
        );
    }
}

/// Artifact producers for the chain auto-placer's sim runs. Each
/// fixture exports at one or more DECLARED inserter-capacity levels
/// (the `-dN` suffix), which selects the world the parity harness
/// builds — NOT the geometry.
///
/// **Geometry is built at [`CHAIN_FIXTURE_CONFIGS`]'s blessed
/// capacity, never at the ambient engine default.** Until #431 the
/// chain path hardcoded L0, so `compose_chain` + a post-hoc
/// `inserter_capacity = lvl` stamp changed only the declaration and
/// the distinction did not exist. #431 moved the default to L2, which
/// silently began exporting L2 geometry under an L0 label: a fixture
/// whose inserters are placed for L2 bonuses but which the harness
/// then runs in an L0 world. `chain-mil5plates-d0` measured −40.7%
/// that way against a blessed −3.3%, and every meter-vs-Factorio
/// comparison built on these fixtures was between two different
/// factories. Re-derive from the shared table so this cannot drift
/// from `cell_registry_hashes_current` again.
#[test]
#[ignore = "artifact producer"]
fn export_chain_fixtures_for_sim() {
    for f in SIM_FIXTURES.iter().filter(|f| !f.levels.is_empty()) {
        let (label, sr) = (f.label, {
            let inputs_set: FxHashSet<String> = f.inputs.iter().map(|s| s.to_string()).collect();
            solver::solve_with_palette_exclusions_and_quality(
                f.target,
                f.rate,
                &inputs_set,
                &MachinePalette::default(),
                "assembling-machine-3",
                &FxHashSet::default(),
                QualityTier::Normal,
            )
            .unwrap()
        });
        for &lvl in f.levels {
            let mut l = f.compose_layout();
            l.inserter_capacity = lvl;
            let tag = format!("{label}-d{lvl}");
            let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&l, &sr, &tag);
            std::fs::create_dir_all("target/tmp").unwrap();
            std::fs::write(format!("target/tmp/{tag}.bp"), &bp).unwrap();
            std::fs::write(
                format!("target/tmp/{tag}.manifest.json"),
                serde_json::to_string_pretty(&manifest).unwrap(),
            )
            .unwrap();
            println!(
                "wrote target/tmp/{tag}.bp ({} boundary in / {} out)",
                l.boundary_inputs.len(),
                l.boundary_outputs.len()
            );
        }
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
        (
            "ec5",
            "electronic-circuit",
            5.0,
            &["iron-plate", "copper-plate"],
        ),
        (
            "ec15",
            "electronic-circuit",
            15.0,
            &["iron-plate", "copper-plate"],
        ),
        (
            "ec30",
            "electronic-circuit",
            30.0,
            &["iron-plate", "copper-plate"],
        ),
        (
            "ac1",
            "advanced-circuit",
            1.0,
            &["iron-plate", "copper-plate", "plastic-bar"],
        ),
        (
            "ac2",
            "advanced-circuit",
            2.0,
            &["iron-plate", "copper-plate", "plastic-bar"],
        ),
        // Package-2 targets: the scaling-wall class + from-ore chains
        // (furnace cells; fan-out >2 on shared plates).
        (
            "ec15-ore",
            "electronic-circuit",
            15.0,
            &["iron-ore", "copper-ore"],
        ),
        (
            "mil5-plates",
            "military-science-pack",
            5.0,
            &[
                "iron-plate",
                "copper-plate",
                "steel-plate",
                "stone-brick",
                "coal",
            ],
        ),
        (
            "mil5-ore",
            "military-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "stone", "coal"],
        ),
        (
            "ec60",
            "electronic-circuit",
            60.0,
            &["iron-plate", "copper-plate"],
        ),
    ];
    for (label, item, rate, inputs) in fixtures {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        // Explicit Off — the DEFAULT is Candidate post-flip, and the bus
        // column must measure the bus.
        let bus_opts = layout::LayoutOptions {
            cell_composition: spaghettio_core::bus::cells::CellComposition::Off,
            ..Default::default()
        };
        let bus = std::panic::catch_unwind(|| layout::build_bus_layout(&sr, bus_opts));
        let bus_desc = match &bus {
            Ok(Ok(l)) => match validate::validate(l, Some(&sr), LayoutStyle::Bus) {
                Ok(issues) => {
                    let e = issues
                        .iter()
                        .filter(|i| i.severity == Severity::Error)
                        .count();
                    format!(
                        "{}x{}={} tiles, {} errors / {} warnings",
                        l.width,
                        l.height,
                        l.width * l.height,
                        e,
                        issues.len() - e
                    )
                }
                Err(er) => format!(
                    "validate() Err: {}",
                    format!("{er}").lines().next().unwrap_or("")
                ),
            },
            Ok(Err(e)) => format!("REFUSED: {}", e.lines().next().unwrap_or("")),
            Err(_) => "PANICKED".into(),
        };
        let comp_desc = match chain_eligible(&sr) {
            Err(e) => format!("INELIGIBLE: {e}"),
            Ok(()) => match compose_chain(&sr) {
                Ok(l) => match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
                    Ok(issues) => {
                        let e = issues
                            .iter()
                            .filter(|i| i.severity == Severity::Error)
                            .count();
                        format!(
                            "{}x{}={} tiles, {} errors / {} warnings",
                            l.width,
                            l.height,
                            l.width * l.height,
                            e,
                            issues.len() - e
                        )
                    }
                    Err(er) => format!(
                        "validate() Err: {}",
                        format!("{er}").lines().next().unwrap_or("")
                    ),
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
/// only the sim-adjudicated warning categories. With the flag OFF
/// (default) the refusal stands (inertness: the bus path is untouched).
///
/// **2026-07-23 (#385 second half):** this candidate's geometry is a
/// SINGLE 6-machine electronic-circuit row (confirmed via a manual
/// entity dump: all 6 machines at y=7, one belt-out cluster at y=10/11
/// with a genuine midpoint sideload bridge) producing the full 15.0/s
/// demand onto one yellow belt-out — distinct from
/// `cell_composed_ec15_zero_errors`'s sim-verified geometry
/// (`compose_pairs_calibrated`'s 3 independent 5.0/s pairs, each well
/// under budget).
///
/// **2026-07-24 (#383/#431 recalibration):** the bridged budget this row
/// was warned against (1.733 × 7.5 = 13.0/s) was measured through an
/// input-bound cell — #431's level sweep shows bridged yellow delivering
/// the full 15.00/s exactly at L2+ with zero `full_output` at every
/// level. At the recalibrated `ROW_LANE_FACTOR_BRIDGED = 2.0` the budget
/// is exactly 15.0/s and the warning correctly no longer fires: the
/// finding this pin tolerated as plausible-but-unproven is now
/// measured-resolved in the geometry's favor.
#[test]
fn cell_candidate_resolves_ec15_refusal() {
    use spaghettio_core::bus::cells::CellComposition;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit",
        15.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();

    // Flag OFF (explicit — the DEFAULT is Candidate since the flip
    // decision): the bus refusal stands. DI is turned off too, because
    // since RFC-053 it is a SECOND refusal-resolving candidate — leaving
    // it on would let this arm pass for the wrong reason and stop
    // isolating cell-composition.
    let off_opts = layout::LayoutOptions {
        cell_composition: CellComposition::Off,
        direct_insertion: spaghettio_core::bus::di_cell::DirectInsertion::Off,
        // RFC-060 made horizontal-stack a THIRD refusal-resolving
        // candidate; off for the same isolation reason as DI above.
        horizontal_candidate: false,
        ..Default::default()
    };
    let off = layout::build_bus_layout(&sr, off_opts);
    assert!(off.is_err(), "flag-Off must preserve the bus refusal");

    // Cell-composition ON, DI OFF — the arm this fixture is ABOUT.
    // Isolating it keeps every adjudicated finding below meaningful;
    // under the true default DI wins this config outright (asserted at
    // the end), which would otherwise silently delete the measurements
    // this test exists to pin.
    let opts = layout::LayoutOptions {
        direct_insertion: spaghettio_core::bus::di_cell::DirectInsertion::Off,
        // Isolating the cell-composition arm (see off_opts note).
        horizontal_candidate: false,
        ..Default::default()
    };
    let l = layout::build_bus_layout(&sr, opts).expect("candidate must resolve the refusal");
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).unwrap();
    let errors = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    assert_eq!(errors, 0, "composed candidate errors: {issues:?}");
    // **2026-07-25 (#448):** `row-input-belt-margin` joins the tolerated
    // set, and this fixture is the check's own motivating measurement —
    // not a tolerated unknown. This row's copper-cable INPUT belt is 6
    // machines × 7.5/s = 45.00/s aggregate against an express belt whose
    // both-lane nominal is exactly 45.0/s. Per-machine sim dumps show the
    // row holding cable 42 → 34 → 20 → 6 → … → 2, the tail machine parked
    // in `item_ingredient_shortage` with 20 iron plates idle beside it,
    // and an upstream cable producer `full_output` with 32 cable stuck —
    // ~5.3% of the row's throughput lost, research-invariant. It is a
    // real defect this layout has; the warning is the point. Note the
    // symmetry with the OUTPUT-side note above: the belt-out at exactly
    // 15.0/s of a 15.0/s bridged budget is measured FINE (#431 sweep),
    // while the belt-in at exactly 45.0/s of 45.0/s is measured BROKEN —
    // the asymmetry is real (an output belt is filled by inserters that
    // simply stall when it is full; an input belt is drained head-first
    // by consumers who buffer).
    assert!(
        issues
            .iter()
            .all(|i| i.category == "inserter-item-throughput"
                || i.category == "row-input-belt-margin"),
        "only the adjudicated categories tolerated: {issues:?}"
    );
    assert_eq!(
        issues
            .iter()
            .filter(|i| i.category == "row-input-belt-margin")
            .count(),
        1,
        "expected exactly the one measured copper-cable input finding: {issues:?}"
    );
    // Post-#431 recalibration the row sits exactly at the bridged
    // budget (2.0 × 7.5 = 15.0/s) — any lane-budget warning here would
    // be a new, unadjudicated claim.
    assert_eq!(
        issues
            .iter()
            .filter(|i| i.category == "row-output-lane-budget")
            .count(),
        0,
        "row-output-lane-budget should not fire at the recalibrated budget: {issues:?}"
    );

    // Under the TRUE default all refusal-resolving candidates are live.
    // Succession on this config, each step strictly at-or-above the
    // last on both issue channels: composition (292 entities, 1
    // adjudicated warning) → DI (RFC-053, 2026-07-26: 272 entities,
    // 0/0, cable off the belts entirely) → horizontal-stack (RFC-060,
    // 2026-07-30: 252 entities, 0/0 — equal cleanliness, so the
    // error-free tier's density order decides; cable returns to belts
    // but on stacked input trunks that clear the margin). K60-3 keeps
    // this config on the sim-verification list precisely because the
    // winner changed.
    //
    // Pinned because it is the clearest evidence for the candidate
    // lifecycle: each successor must beat the incumbent on the
    // never-worse terms, and the floors below enforce exactly that
    // rather than any one winner's internal structure.
    let both = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
        .expect("the default must still resolve the refusal");
    let both_issues = validate::validate(&both, Some(&sr), LayoutStyle::Bus).unwrap();
    assert!(
        both_issues.iter().all(|i| i.severity != Severity::Error),
        "DI winner must be error-free: {both_issues:?}"
    );
    assert_eq!(
        both_issues.iter().filter(|i| i.category == "row-input-belt-margin").count(),
        0,
        "DI removes the cable input belt, so its margin finding must not fire: {both_issues:?}"
    );
    assert!(
        both.entities.len() < l.entities.len(),
        "the default winner must beat composition on entity count ({} vs composed {})",
        both.entities.len(),
        l.entities.len()
    );
    // Floor at DI's 272-entity resolution: a future winner may only
    // succeed by matching or beating the incumbent (the succession
    // note above). If this fires, a candidate won while being LARGER
    // than a clean predecessor — the error-free tier ordering broke.
    assert!(
        both.entities.len() <= 272,
        "default winner ({} entities) regressed past DI's 272-entity resolution",
        both.entities.len()
    );
}

/// Print geometry hashes for registry seeding.
#[test]
#[ignore = "registry seeding probe"]
fn probe_registry_hashes() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::bus::cells::registry::geometry_hash;
    for (label, item, rate, inputs) in [
        (
            "chain-ec15",
            "electronic-circuit",
            15.0,
            &["iron-plate", "copper-plate"][..],
        ),
        (
            "chain-ac1",
            "advanced-circuit",
            1.0,
            &["iron-plate", "copper-plate", "plastic-bar"][..],
        ),
        (
            "chain-ec30",
            "electronic-circuit",
            30.0,
            &["iron-plate", "copper-plate"][..],
        ),
        (
            "chain-mil5ore",
            "military-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "stone", "coal"][..],
        ),
        (
            "chain-mil5plates",
            "military-science-pack",
            5.0,
            &[
                "iron-plate",
                "copper-plate",
                "steel-plate",
                "stone-brick",
                "coal",
            ][..],
        ),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            rate,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let l = compose_chain(&sr).unwrap();
        println!("{label}: {:016x}", geometry_hash(&l));
    }
    for (label, item, rate, inputs) in [
        (
            "mega-plastic2",
            "plastic-bar",
            2.0,
            &["crude-oil", "water", "coal"][..],
        ),
        ("mega-sulfur2", "sulfur", 2.0, &["crude-oil", "water"][..]),
    ] {
        let (_sr, l) =
            spaghettio_core::bus::cells::mega::compose_mega_calibrated(item, rate, inputs).unwrap();
        println!("{label}: {:016x}", geometry_hash(&l));
    }
    {
        let inputs_set: FxHashSet<String> =
            ["iron-ore", "copper-ore", "crude-oil", "water", "coal"]
                .iter()
                .map(|s| s.to_string())
                .collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            "advanced-circuit",
            2.0,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        println!(
            "mega-chain-ac2raw: {:016x}",
            geometry_hash(&compose_chain(&sr).unwrap())
        );
    }
}

/// PERMANENT GATE (RFC-051 registry): every seeded sim-verified entry
/// still matches freshly composed geometry — iterated from the
/// registry itself, so adding an entry without a re-derivable fixture
/// config fails loudly here, and an engine change that alters any
/// registered cell fails here instead of silently decaying
/// "sim-verified" into a stale verdict (#375 review finding 1; world
/// axes added per #391 — declared fields are checked data, recorded at
/// measurement time). The fix when it fires: re-run the sim on the new
/// geometry, then update the hash + measurement in
/// cell-sim-registry.json.
/// #415: the declared inserter-capacity level must actually REACH the
/// composed cells' placer. L0 stays byte-identical to `compose_chain`
/// (the registry gate enforces that side); a nonzero level must change
/// the geometry — #381's ladder sizes hands differently — or the option
/// silently died on the way down again (#383's original symptom: d1/d7
/// fixtures with byte-identical blueprints).
#[test]
fn chain_capacity_reaches_the_placer() {
    use spaghettio_core::bus::cells::chain::{compose_chain, compose_chain_with_capacity};
    use spaghettio_core::bus::cells::registry::geometry_hash;
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit",
        15.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let plain = compose_chain(&sr).unwrap();
    let default_explicit =
        compose_chain_with_capacity(&sr, spaghettio_core::common::DEFAULT_INSERTER_CAPACITY)
            .unwrap();
    assert_eq!(
        geometry_hash(&plain),
        geometry_hash(&default_explicit),
        "plain compose_chain must equal the DEFAULT_INSERTER_CAPACITY build"
    );
    let l0 = compose_chain_with_capacity(&sr, 0).unwrap();
    let l7 = compose_chain_with_capacity(&sr, 7).unwrap();
    assert_eq!(
        l7.inserter_capacity, 7,
        "composed layout must declare its capacity"
    );
    assert_ne!(
        geometry_hash(&l0), geometry_hash(&l7),
        "declared L7 must change composed geometry (ladder-resized hands) —          identical hashes mean the option died on the way to the placer again (#383)"
    );
}

/// #383 (2026-07-24): the EC@15 chain — the canonical #383 fixture —
/// carries input-inserter-throughput warnings at the raw L0 world but
/// composes inserter-clean at the L2 engine default (the input bind
/// clears at non-bulk hand 2). Guards the default-level fix: sim-measured
/// at 13.8/s (−8%) at L0 vs 15.0/s (full plan) at L2.
#[test]
fn ec15_chain_inserter_clean_at_default_capacity() {
    use spaghettio_core::bus::cells::chain::{compose_chain, compose_chain_with_capacity};
    use spaghettio_core::bus::cells::registry::geometry_hash;
    use spaghettio_core::common::DEFAULT_INSERTER_CAPACITY;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit",
        15.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    // Geometry-regression lock on the L2 DEFAULT path — the geometry
    // production actually builds, which the registry gate (rebuilt at
    // explicit L0) does not cover (PR #431 review finding). Not sim-blessed
    // (there is no L2 sim baseline); a self-consistency golden. Re-bless
    // deliberately if a change legitimately reshapes the default geometry.
    assert_eq!(
        format!("{:016x}", geometry_hash(&compose_chain(&sr).unwrap())),
        "eb9e1796a1f53695",
        "EC@15 default (L2) geometry changed — re-verify and re-bless"
    );
    let inserter_warns = |cap: u8| -> usize {
        let l = compose_chain_with_capacity(&sr, cap).unwrap();
        let issues =
            validate::validate(&l, Some(&sr), LayoutStyle::Bus).unwrap_or_else(|e| e.issues);
        assert!(
            issues.iter().all(|i| i.severity != Severity::Error),
            "EC@15 L{cap} must have no errors: {issues:?}"
        );
        issues
            .iter()
            .filter(|i| i.category.contains("inserter") || i.category == "input-rate-delivery")
            .count()
    };
    // L0 (raw unresearched) still shows the #383 input bind — guards the
    // premise: if L0 stops warning, this test no longer proves anything.
    assert!(
        inserter_warns(0) > 0,
        "L0 must still exhibit the #383 input-inserter bind"
    );
    // The L2 engine default (red+green research) clears them all.
    assert_eq!(
        inserter_warns(DEFAULT_INSERTER_CAPACITY),
        0,
        "EC@15 chain must be inserter-clean at the engine default (#383 fix)"
    );
}

/// #383 / #415 (#422 landed): a mega-containing chain composes at its
/// DECLARED capacity. #422 threads capacity into the non-mega cells; the
/// mega INTERIOR bootstrap (`compose_mega_block`) stays conservatively L0
/// (it takes no capacity argument). No refusal, no whole-chain clamp.
/// History: hard `Err` refusal (pre-#383) → whole-chain L0 clamp (#383
/// initial) → dropped once #422 landed (PR #431 review coordination).
#[test]
fn mega_chain_composes_at_declared_capacity() {
    use spaghettio_core::bus::cells::chain::compose_chain_with_capacity;
    let inputs: FxHashSet<String> = ["iron-ore", "copper-ore", "crude-oil", "water", "coal"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "advanced-circuit",
        2.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    // L7 composes (no refusal, no clamp) and DECLARES its requested level —
    // the non-mega cells thread it; the mega interior stays L0 internally.
    let l7 = compose_chain_with_capacity(&sr, 7).expect("mega chain composes at L7");
    assert_eq!(
        l7.inserter_capacity, 7,
        "mega chain must declare its requested capacity (#422 threaded non-mega cells)"
    );
    let l0 = compose_chain_with_capacity(&sr, 0).expect("L0 mega chain composes");
    assert_eq!(l0.inserter_capacity, 0);
}

#[test]
fn cell_registry_hashes_current() {
    use spaghettio_core::bus::cells::chain::compose_chain_with_capacity;
    use spaghettio_core::bus::cells::registry::{entries, geometry_hash};
    // Fixture configs the registry may reference, keyed by (target,
    // rate); same-key configs (mil5 ore vs plates) disambiguate by
    // hash.
    // kind: "chain" re-derives via compose_chain; "mega" via the
    // RFC-052 uncropped mega-cell composer.
    //
    // The trailing `u8` is the CAPACITY THE GEOMETRY WAS BLESSED AT, and
    // it is per-config for a reason (2026-07-24): entries blessed before
    // the engine default moved to L2 (#431) are L0-GEOMETRY baselines —
    // the chain path hardcoded L0 pre-#422, so e.g. ec15's d1 and d7
    // entries share ONE hash, differing only in the harness WORLD
    // (`declared_inserter_capacity`), not the built geometry. Those must
    // still reproduce at L0. Entries blessed after the flip (chem5) are
    // real L2 geometry and must reproduce at L2. Re-deriving everything
    // at one capacity would falsely fail one group or the other.
    let configs: &[(&str, f64, &[&str], &str, u8)] = &[
        (
            "advanced-circuit",
            1.0,
            &["iron-plate", "copper-plate", "plastic-bar"],
            "chain",
            0,
        ),
        (
            "electronic-circuit",
            15.0,
            &["iron-plate", "copper-plate"],
            "chain",
            0,
        ),
        (
            "electronic-circuit",
            30.0,
            &["iron-plate", "copper-plate"],
            "chain",
            0,
        ),
        (
            "military-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "stone", "coal"],
            "chain",
            0,
        ),
        (
            "military-science-pack",
            5.0,
            &[
                "iron-plate",
                "copper-plate",
                "steel-plate",
                "stone-brick",
                "coal",
            ],
            "chain",
            0,
        ),
        (
            "plastic-bar",
            2.0,
            &["crude-oil", "water", "coal"],
            "mega",
            0,
        ),
        ("sulfur", 2.0, &["crude-oil", "water"], "mega", 0),
        (
            "advanced-circuit",
            2.0,
            &["iron-ore", "copper-ore", "crude-oil", "water", "coal"],
            "chain",
            0,
        ),
        // First post-#431 registration: blessed at the L2 default.
        (
            "chemical-science-pack",
            5.0,
            &[
                "iron-ore",
                "copper-ore",
                "crude-oil",
                "water",
                "coal",
                "iron-plate",
                "copper-plate",
                "steel-plate",
            ],
            "chain",
            2,
        ),
    ];
    assert!(!entries().is_empty(), "registry must not be empty");
    for e in entries() {
        let candidates: Vec<String> = configs
            .iter()
            .filter(|(t, r, _, _, _)| *t == e.target && (r - e.rate).abs() < 1e-9)
            .map(|(t, r, inputs, kind, blessed_capacity)| {
                let l = match *kind {
                    "mega" => {
                        spaghettio_core::bus::cells::mega::compose_mega_calibrated(t, *r, inputs)
                            .unwrap()
                            .1
                    }
                    _ => {
                        let inputs_set: FxHashSet<String> =
                            inputs.iter().map(|s| s.to_string()).collect();
                        let sr = solver::solve_with_palette_exclusions_and_quality(
                            t,
                            *r,
                            &inputs_set,
                            &MachinePalette::default(),
                            "assembling-machine-3",
                            &FxHashSet::default(),
                            QualityTier::Normal,
                        )
                        .unwrap();
                        // Re-derive at the capacity this config was BLESSED at
                        // (never the ambient engine default, which moves) — see
                        // the per-config `u8` above for why the two groups differ.
                        compose_chain_with_capacity(&sr, *blessed_capacity).unwrap()
                    }
                };
                format!("{:016x}", geometry_hash(&l))
            })
            .collect();
        assert!(
            !candidates.is_empty(),
            "{}@{}: registry entry has no re-derivable fixture config in this gate — add it",
            e.target,
            e.rate
        );
        assert!(candidates.contains(&e.geometry_hash),
            "{}@{} (declared capacity {}): composed geometry no longer matches the registered hash {} (fresh: {:?}) — re-verify in the sim and update cell-sim-registry.json",
            e.target, e.rate, e.declared_inserter_capacity, e.geometry_hash, candidates);
    }
}

/// PERMANENT GATE (RFC-051 K-quantization): the copy count is the
/// smallest K putting every produced item AND every external input at
/// ≤45/s (express capacity) per copy — a physical belt cap, not a
/// quality knob (a 15/s "measured-exact" quantum was falsified: the
/// Phase-1 exact measurement was pre-#378 harness tech state, and
/// under declared capacity small rows measure WORSE — #383). Pins the
/// ladder's K values: chains under the cap stay K=1 bit-identical
/// (the registered chain-ac1 hash depends on it); ec30/ec60 — the
/// pre-quantization corridor-cap refusals — now compose; K_MAX=12
/// refuses loudly.
#[test]
fn cell_quantization_copy_counts() {
    use spaghettio_core::bus::cells::chain::{chain_eligible, required_copies};
    for (label, item, rate, inputs, want_k) in [
        (
            "ec15",
            "electronic-circuit",
            15.0,
            &["iron-plate", "copper-plate"][..],
            1,
        ),
        (
            "ac1",
            "advanced-circuit",
            1.0,
            &["iron-plate", "copper-plate", "plastic-bar"][..],
            1,
        ),
        (
            "ec5",
            "electronic-circuit",
            5.0,
            &["iron-plate", "copper-plate"][..],
            1,
        ),
        // pre-quantization these two REFUSED on the 45/s corridor cap
        (
            "ec30",
            "electronic-circuit",
            30.0,
            &["iron-plate", "copper-plate"][..],
            2,
        ),
        (
            "ec60",
            "electronic-circuit",
            60.0,
            &["iron-plate", "copper-plate"][..],
            4,
        ),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            rate,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        assert_eq!(required_copies(&sr), want_k, "{label}: copy count");
        assert!(
            chain_eligible(&sr).is_ok(),
            "{label}: must be chain-eligible"
        );
    }
    // Past K_MAX=12 the chain refuses loudly (ec600 → cable 1800/s → K=40).
    let inputs_set: FxHashSet<String> = ["iron-plate", "copper-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit",
        600.0,
        &inputs_set,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let err = chain_eligible(&sr).unwrap_err();
    assert!(
        err.contains("quantized copies"),
        "K_MAX refusal, got: {err}"
    );
}

/// PERMANENT GATE (belt-tier constraint): composed corridors are
/// express-only, and belt tier is a USER constraint — under any lower
/// max_belt_tier the candidate must be INERT: whatever the bus does
/// (succeed, as it happens to here, or refuse), the Candidate flag
/// changes nothing, and no express entity ever appears. (Latent from
/// the flip until K-quantization surfaced it: an eligible chain whose
/// bus path fails under a sub-express cap would have won with express
/// corridors.)
#[test]
fn cell_candidate_respects_belt_tier_cap() {
    use spaghettio_core::bus::cells::registry::geometry_hash;
    use spaghettio_core::bus::cells::CellComposition;
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit",
        15.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    // EC@15 is chain-eligible, so only the tier guard keeps the
    // candidate out under a red cap.
    let build = |cc: CellComposition| {
        layout::build_bus_layout(
            &sr,
            layout::LayoutOptions {
                max_belt_tier: Some("fast-transport-belt".into()),
                cell_composition: cc,
                ..Default::default()
            },
        )
    };
    match (
        build(CellComposition::Candidate),
        build(CellComposition::Off),
    ) {
        (Ok(on), Ok(off)) => {
            assert_eq!(
                geometry_hash(&on),
                geometry_hash(&off),
                "sub-express cap: Candidate must be inert (bit-identical to Off)"
            );
            assert!(
                !on.entities.iter().any(|e| e.name.starts_with("express")),
                "sub-express cap: no express entity may appear"
            );
        }
        (on, off) => assert_eq!(
            on.is_err(),
            off.is_err(),
            "sub-express cap: Candidate must not flip a refusal"
        ),
    }
    // Express cap (explicit) still composes the #336 refusal.
    let opts = layout::LayoutOptions {
        max_belt_tier: Some("express-transport-belt".into()),
        ..Default::default()
    };
    layout::build_bus_layout(&sr, opts).expect("express-capped EC@15 must compose");
}

/// PERMANENT GATE (#384 review finding 4): the additive contract —
/// where the BUS succeeds, the bus wins; composition surfaces only on
/// refusals. This was empirically true (density margin 2–5×) but not
/// pinned, and the selection tie-break used to point at cell-composed.
/// Asserts native wins every chain-eligible fixture where both paths
/// succeed, via the observable marker: composed winners carry the
/// "cell-composed:" registry annotation in warnings; bus winners never
/// do.
#[test]
fn cell_candidate_never_displaces_a_succeeding_bus() {
    for (label, item, rate, inputs) in [
        ("gear15", "iron-gear-wheel", 15.0, &["iron-plate"][..]),
        (
            "ec5",
            "electronic-circuit",
            5.0,
            &["iron-plate", "copper-plate"][..],
        ),
        (
            "ac1",
            "advanced-circuit",
            1.0,
            &["iron-plate", "copper-plate", "plastic-bar"][..],
        ),
        (
            "ac2",
            "advanced-circuit",
            2.0,
            &["iron-plate", "copper-plate", "plastic-bar"][..],
        ),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            rate,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
            .unwrap_or_else(|e| panic!("{label}: bus-succeeding fixture must lay out: {e}"));
        assert!(
            !l.warnings.iter().any(|w| w.starts_with("cell-composed:")),
            "{label}: composition displaced a succeeding bus layout"
        );
    }
}

/// PERMANENT GATE (flipped 2026-07-23 — the capability win its
/// predecessor anticipated): mil5-ore COMPOSES. The Router's overlap
/// classes (boundary-blind hops, 1-pitch bypass rows/lanes) and the
/// silent east-only bypass assumption (reversed-dependency placement
/// can put consumers WEST of producers) are fixed, so the 9-spec
/// scaling-wall fixture the bus refuses (stone-brick lane capacity,
/// #336-class) now wins via composition at 0 validation errors. The
/// self-validation contract (#387 review) stands behind it: if this
/// geometry ever regresses to errors, the candidate refuses and this
/// gate fails on the refusal — never on a silently broken Ok.
#[test]
fn cell_candidate_composes_mil5_ore() {
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = ["iron-ore", "copper-ore", "stone", "coal"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "military-science-pack",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    // Bus-only arm still refuses — composition is doing the winning.
    let off = layout::build_bus_layout(
        &sr,
        layout::LayoutOptions {
            cell_composition: spaghettio_core::bus::cells::CellComposition::Off,
            ..Default::default()
        },
    );
    assert!(
        off.is_err(),
        "bus-only mil5-ore must still refuse (else move this fixture to the bus ladder)"
    );
    let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
        .expect("mil5-ore must compose");
    assert!(
        l.warnings.iter().any(|w| w.starts_with("cell-composed:")),
        "the composed candidate must be the winner"
    );
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).unwrap();
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "composed mil5-ore errors: {errors:?}");
}

/// PERMANENT GATE (#392): validation-tiered selection — when the
/// best-scoring accepted candidate hard-fails validation and a CLEAN
/// accepted sibling exists, the clean one wins. mil5-from-plates is
/// the live specimen: the native bus layout fails validation while the
/// composed candidate is 0 errors / 0 warnings; pre-#392 the search
/// returned the broken native as Ok.
#[test]
fn cell_candidate_wins_mil5_plates_over_broken_native() {
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = [
        "iron-plate",
        "copper-plate",
        "steel-plate",
        "stone-brick",
        "coal",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "military-science-pack",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
        .expect("mil5-plates must lay out");
    assert!(
        l.warnings.iter().any(|w| w.starts_with("cell-composed:")),
        "the clean composed candidate must win over the validation-broken native"
    );
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).unwrap();
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "winner must validate clean: {errors:?}");
}

/// PERMANENT GATE (#396 review, blocking finding): the selection
/// tier's validate() calls must never leak trace events into the
/// winner's replayed stream — the web timing log reads the FIRST
/// ValidationCompleted event, so a leaked loser-candidate validation
/// makes the browser report a broken layout for a clean one. Exactly
/// one ValidationCompleted may appear: the winning cells candidate's
/// own self-check replay.
#[test]
fn selection_tier_validation_never_leaks_trace_events() {
    let inputs: FxHashSet<String> = [
        "iron-plate",
        "copper-plate",
        "steel-plate",
        "stone-brick",
        "coal",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "military-science-pack",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = layout::build_bus_layout_traced(&sr, layout::LayoutOptions::default())
        .expect("mil5-plates must lay out");
    let n_validation_events = l
        .trace
        .as_ref()
        .expect("traced build carries trace")
        .iter()
        .filter(|e| {
            matches!(
                e,
                spaghettio_core::trace::TraceEvent::ValidationCompleted { .. }
            )
        })
        .count();
    assert_eq!(n_validation_events, 1,
        "only the winner's own validation may appear in the trace (leaked tier validations corrupt the web timing log)");
}

/// PERMANENT GATE (RFC-052 Phase A, gate (a) validator half): the oil
/// mega-cell — the UNCROPPED engine layout for a fluid subgraph,
/// boundary-adapted to the calibrated form by the generic re-pitching
/// adapter — composes at 0 errors / 0 warnings. plastic-from-crude is
/// the fixture (refinery → chem plant, one fluid intermediate; the
/// crop would sever its petroleum trunk, which is the whole reason
/// mega-cells are uncropped — RFC-052 decision log). The sim half of
/// gate (a) lives in the registry entry.
#[test]
fn mega_cell_plastic_from_crude_zero_issues() {
    use spaghettio_core::bus::cells::mega::compose_mega_calibrated;
    use spaghettio_core::validate::{self, LayoutStyle};
    // All three Phase-A fixtures gate here (#401 review note: probe-only
    // coverage of plastic@5/sulfur@2 wouldn't catch a regression).
    for (label, item, rate, inputs) in [
        (
            "plastic@2",
            "plastic-bar",
            2.0,
            &["crude-oil", "water", "coal"][..],
        ),
        (
            "plastic@5",
            "plastic-bar",
            5.0,
            &["crude-oil", "water", "coal"][..],
        ),
        ("sulfur@2", "sulfur", 2.0, &["crude-oil", "water"][..]),
    ] {
        let (sr, l) = compose_mega_calibrated(item, rate, inputs)
            .unwrap_or_else(|e| panic!("{label}: mega must compose: {e}"));
        // Kit-pitch invariant: boundary heads >= 4 apart, all at y=0,
        // sorted west→east (#363).
        let xs: Vec<i32> = l.boundary_inputs.iter().map(|b| b.x).collect();
        assert!(
            xs.windows(2).all(|w| w[1] - w[0] >= 4),
            "{label}: feed heads must sit at >=4 pitch: {xs:?}"
        );
        assert!(
            l.boundary_inputs.iter().all(|b| b.y == 0),
            "{label}: feed heads at the north edge"
        );
        let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus)
            .unwrap_or_else(|e| panic!("{label}: mega must validate: {e}"));
        assert!(issues.is_empty(), "{label}: mega issues: {issues:?}");
    }
}

/// Artifact producer for RFC-052 mega-cell sim runs (declared 0).
#[test]
#[ignore = "artifact producer"]
fn export_mega_fixtures_for_sim() {
    use spaghettio_core::bus::cells::mega::compose_mega_calibrated;
    for (label, item, rate, inputs) in [
        (
            "mega-plastic2",
            "plastic-bar",
            2.0,
            &["crude-oil", "water", "coal"][..],
        ),
        ("mega-sulfur2", "sulfur", 2.0, &["crude-oil", "water"][..]),
    ] {
        let (sr, l) = compose_mega_calibrated(item, rate, inputs).unwrap();
        let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&l, &sr, label);
        std::fs::create_dir_all("target/tmp").unwrap();
        std::fs::write(format!("target/tmp/{label}.bp"), &bp).unwrap();
        std::fs::write(
            format!("target/tmp/{label}.manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        println!(
            "wrote target/tmp/{label}.bp ({} in / {} out)",
            l.boundary_inputs.len(),
            l.boundary_outputs.len()
        );
    }
}

/// PERMANENT GATE (RFC-052 Phase B, gate (b) validator half): the
/// FLAGSHIP — advanced circuits from fully raw inputs (iron ore,
/// copper ore, crude oil, water, coal) — composes at 0 errors /
/// 0 warnings across the honest ladder (the plastic sub-solve outgrows
/// the engine's own oil layout above AC@4; the candidate self-refuses
/// there). The fluid subgraph (refinery + plastic chem) collapses into
/// one mega slot; the mega corridor rides its own bypass row from the
/// drain head to the AC cell.
#[test]
fn mega_chain_ac_from_raw_zero_issues() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    for rate in [1.0, 2.0, 4.0] {
        let inputs: FxHashSet<String> = ["iron-ore", "copper-ore", "crude-oil", "water", "coal"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            "advanced-circuit",
            rate,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let l =
            compose_chain(&sr).unwrap_or_else(|e| panic!("AC@{rate} from raw must compose: {e}"));
        let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus)
            .unwrap_or_else(|e| panic!("AC@{rate} from raw must validate: {e}"));
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "AC@{rate} from raw errors: {errors:?}");
        // AC@4's cable cell outputs 40/s. Under the old bridged express
        // budget (1.733 × 22.5 = 39.0/s) that was a 1/s shortfall and
        // raised one row-output-lane-budget warning; the 2026-07-24
        // #383/#431 recalibration (ROW_LANE_FACTOR_BRIDGED = 2.0 → 45.0/s
        // on express) clears it. Every rung now asserts a plain zero:
        // keeping the old tolerated-category branch would pass VACUOUSLY,
        // since the category it filters for can no longer fire here.
        assert!(issues.is_empty(), "AC@{rate} from raw issues: {issues:?}");
    }
}

/// PERMANENT GATE (RFC-052 kill-2): chemical-science-pack@5 from raw —
/// a config whose BUS layout hard-fails validation (junction-solver
/// crossing) — composes at 0 errors. The fluid subgraph (refinery +
/// plastic + sulfur, two exports) collapses into a mega block whose
/// packed feeds route via the PTG-tail joint planner; the chain
/// quantizes K=2 (cable 60/s), exercising per-copy mega replication.
/// Tolerated warning categories only: the #383-class inserter
/// attribution and the multi-block pole-network note.
#[test]
fn mega_chain_chem5_resolves_bus_failure() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = [
        "iron-ore",
        "copper-ore",
        "crude-oil",
        "water",
        "coal",
        "iron-plate",
        "copper-plate",
        "steel-plate",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "chemical-science-pack",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = compose_chain(&sr).expect("chem5 from raw must compose");
    let issues = validate::validate(&l, Some(&sr), LayoutStyle::Bus).expect("must validate");
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "chem5 errors: {errors:?}");
    assert!(
        issues
            .iter()
            .all(|i| matches!(i.category.as_str(), "inserter-item-throughput" | "power")),
        "only adjudicated categories tolerated: {issues:?}"
    );
}

/// RFC-052 increment 2 (chain-fed mega inputs): processing-unit@4
/// from raw — the fluid subgraph swallows the PU spec, which consumes
/// chain-produced EC/AC/iron-plate. The BUS path hard-fails here
/// (unresolved junctions); the chain must compose with zero errors,
/// tolerating only the adjudicated categories. (@2 is both-paths-clean
/// since #408's reach fix shifted junction geometry — the bus-refusal
/// win for this class lives at 4/s.)
#[test]
fn mega_chain_pu4_resolves_bus_failure() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = ["iron-ore", "copper-ore", "crude-oil", "water", "coal"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "processing-unit",
        4.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let plan = spaghettio_core::bus::cells::mega::mega_subgraph(&sr)
        .expect("subgraph")
        .expect("PU chain has a fluid subgraph");
    assert!(
        !plan.chain_fed.is_empty(),
        "PU class must exercise chain-fed inputs, got {:?}",
        plan.chain_fed
    );
    let l = compose_chain(&sr).expect("PU@4 from raw must compose");
    let issues = match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
        Ok(v) => v,
        Err(e) => e.issues,
    };
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "PU@4 errors: {errors:?}");
    assert!(
        issues.iter().all(|i| matches!(
            i.category.as_str(),
            "inserter-item-throughput" | "inserter-throughput" | "power" | "row-output-lane-budget"
        )),
        "only adjudicated categories tolerated: {issues:?}"
    );
}

/// RFC-052 Phase C flagship: utility-science-pack@2 from fully raw
/// inputs. The BUS hard-fails (belt-loop + underground-belt); the
/// mega swallows the ENTIRE oil complex — 10 members including BOTH
/// oil processings, cracking, and lubricant, with 4 solid exports
/// (multi-consumer fan-out) and 5 chain-fed inputs. Composes at ZERO
/// errors.
///
/// **2026-07-24 — moved to opt-in.** This is the single heaviest test
/// in the suite: >6 min of LIVE SAT solving because USP's 10-member
/// oil complex generates crossing zones absent from the baked cache
/// (`crates/core/data/sat-zones.bin`), so every zone re-solves each
/// run. Its sibling `mega_chain_chem5_resolves_bus_failure` does the
/// same class of work in 0.67s purely because ITS zones are cached —
/// i.e. the cost is uncached-zone artifact, not intrinsic. It held the
/// whole `cell_composition` binary at ~1378s. The in-loop mega gates
/// (chem5, pu4, ac-from-raw) keep the mega path covered on every run;
/// this flagship stays runnable opt-in. To restore it to the default
/// loop cheaply, bake its zones into the cache (the chem5 route) rather
/// than un-ignoring it as-is. See #433-adjacent perf note / RFC-052.
#[test]
#[ignore = "RFC-052 USP@2 mega gate: >6min live SAT (uncached zones); opt in with --ignored, or bake its zones into sat-zones.bin"]
fn mega_chain_usp2_resolves_bus_failure() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let inputs: FxHashSet<String> = [
        "iron-ore",
        "copper-ore",
        "crude-oil",
        "water",
        "coal",
        "stone",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "utility-science-pack",
        2.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let plan = spaghettio_core::bus::cells::mega::mega_subgraph(&sr)
        .expect("subgraph")
        .expect("USP chain has a fluid subgraph");
    assert!(
        plan.members.contains("advanced-oil-processing")
            && plan.members.iter().any(|r| r.contains("cracking")),
        "flagship must exercise the advanced complex, got {:?}",
        plan.members
    );
    assert!(
        plan.outputs.len() >= 2 && !plan.chain_fed.is_empty(),
        "flagship must exercise multi-export fan + chain-fed inputs"
    );
    let l = compose_chain(&sr).expect("USP@2 from raw must compose");
    let heavy_exit = l
        .surplus_exits
        .iter()
        .find(|(item, _, _)| item == "heavy-oil")
        .expect("advanced-only oil plan must route heavy-oil surplus to the perimeter");
    assert!(
        l.entities.iter().any(|e| {
            e.x == heavy_exit.1
                && e.y == heavy_exit.2
                && matches!(e.name.as_str(), "pipe" | "pipe-to-ground")
                && e.carries.as_deref() == Some("heavy-oil")
        }),
        "heavy-oil surplus exit must name a physical matching pipe: {heavy_exit:?}"
    );
    let issues = match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
        Ok(v) => v,
        Err(e) => e.issues,
    };
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "USP@2 errors: {errors:?}");
}

/// Artifact producer for the Phase C flagship sim run.
#[test]
#[ignore = "artifact producer"]
fn export_mega_usp_for_sim() {
    let inputs: FxHashSet<String> = [
        "iron-ore",
        "copper-ore",
        "crude-oil",
        "water",
        "coal",
        "stone",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "utility-science-pack",
        2.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = SimFixture::find("mega-chain-usp2raw").compose_layout();
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&l, &sr, "mega-chain-usp2raw");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/mega-chain-usp2raw.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/mega-chain-usp2raw.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "wrote mega-chain-usp2raw.bp ({} in / {} out)",
        l.boundary_inputs.len(),
        l.boundary_outputs.len()
    );
}

/// RFC-055 real-geometry experiment. Kept opt-in while compact ordering is
/// speculative; unlike the placement estimator, this composes and validates
/// both complete routed factories.
#[test]
#[ignore = "RFC-055 compact-order experiment"]
fn rfc055_compact_usp_real_geometry() {
    use spaghettio_core::bus::cells::chain::{compose_chain_compact, compose_chain_with_capacity};
    use spaghettio_core::validate::{self, LayoutStyle, Severity};

    let inputs: FxHashSet<String> = [
        "iron-ore",
        "copper-ore",
        "crude-oil",
        "water",
        "coal",
        "stone",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "utility-science-pack",
        2.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let control = compose_chain_with_capacity(&sr, 0).expect("control composes");
    let compact = compose_chain_compact(&sr, 0).expect("compact composes");
    let issues = match validate::validate(&compact, Some(&sr), LayoutStyle::Bus) {
        Ok(v) => v,
        Err(e) => e.issues,
    };
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "compact USP errors: {errors:?}");
    println!(
        "control={}x{} entities={} compact={}x{} entities={}",
        control.width,
        control.height,
        control.entities.len(),
        compact.width,
        compact.height,
        compact.entities.len()
    );
}

#[test]
#[ignore = "RFC-055 acceptance-corpus experiment"]
fn rfc055_compact_acceptance_corpus() {
    use spaghettio_core::bus::cells::chain::{compose_chain_compact, compose_chain_with_capacity};
    use spaghettio_core::validate::{self, LayoutStyle, Severity};

    for (label, target, rate, raw) in [
        (
            "usp2raw",
            "utility-science-pack",
            2.0,
            &[
                "iron-ore",
                "copper-ore",
                "crude-oil",
                "water",
                "coal",
                "stone",
            ][..],
        ),
        (
            "chem5raw",
            "chemical-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "crude-oil", "water", "coal"][..],
        ),
        (
            "pu4raw",
            "processing-unit",
            4.0,
            &["iron-ore", "copper-ore", "crude-oil", "water", "coal"][..],
        ),
        (
            "mil5ore",
            "military-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "stone", "coal"][..],
        ),
    ] {
        let inputs: FxHashSet<String> = raw.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            target,
            rate,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let control =
            compose_chain_with_capacity(&sr, 0).unwrap_or_else(|e| panic!("{label} control: {e}"));
        let compact =
            compose_chain_compact(&sr, 0).unwrap_or_else(|e| panic!("{label} compact: {e}"));
        let issues = match validate::validate(&compact, Some(&sr), LayoutStyle::Bus) {
            Ok(v) => v,
            Err(e) => e.issues,
        };
        let errors = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        assert_eq!(errors, 0, "{label} compact has errors: {issues:?}");
        let belts = |l: &spaghettio_core::models::LayoutResult| {
            l.entities
                .iter()
                .filter(|e| e.name.contains("transport-belt") || e.name.contains("splitter"))
                .count()
        };
        let corridors = |l: &spaghettio_core::models::LayoutResult| {
            l.entities
                .iter()
                .filter(|e| {
                    e.segment_id
                        .as_deref()
                        .is_some_and(|s| s.starts_with("corr:"))
                })
                .count()
        };
        println!("{label}: control={}x{} entities={} belts={} corr={} compact={}x{} entities={} belts={} corr={}",
            control.width, control.height, control.entities.len(), belts(&control), corridors(&control),
            compact.width, compact.height, compact.entities.len(), belts(&compact), corridors(&compact));
    }
}

#[test]
#[ignore = "RFC-055 Factorio artifact producer"]
fn export_rfc055_factorio_candidates() {
    use spaghettio_core::blueprint::export_with_manifest;
    use spaghettio_core::bus::cells::chain::{compose_chain_compact, compose_chain_with_capacity};

    std::fs::create_dir_all("target/tmp/rfc055").unwrap();
    for (label, target, rate, raw) in [
        (
            "usp2raw",
            "utility-science-pack",
            2.0,
            &[
                "iron-ore",
                "copper-ore",
                "crude-oil",
                "water",
                "coal",
                "stone",
            ][..],
        ),
        (
            "chem5raw",
            "chemical-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "crude-oil", "water", "coal"][..],
        ),
        (
            "pu4raw",
            "processing-unit",
            4.0,
            &["iron-ore", "copper-ore", "crude-oil", "water", "coal"][..],
        ),
    ] {
        let inputs: FxHashSet<String> = raw.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            target,
            rate,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        for (variant, layout) in [
            ("control", compose_chain_with_capacity(&sr, 0).unwrap()),
            ("compact", compose_chain_compact(&sr, 0).unwrap()),
        ] {
            let artifact = format!("rfc055-{label}-{variant}");
            let (bp, manifest) = export_with_manifest(&layout, &sr, &artifact);
            std::fs::write(format!("target/tmp/rfc055/{artifact}.bp"), bp).unwrap();
            std::fs::write(
                format!("target/tmp/rfc055/{artifact}.manifest.json"),
                serde_json::to_string_pretty(&manifest).unwrap(),
            )
            .unwrap();
            println!(
                "wrote {artifact}: {}x{}, {} entities",
                layout.width,
                layout.height,
                layout.entities.len()
            );
        }
    }
}

#[test]
#[ignore = "RFC-057 coarse machine compaction potential"]
fn rfc057_machine_constraint_baseline() {
    use spaghettio_core::bus::compaction::{
        blocks_overlap, build_local_manifold_graph, build_manifold_nets, compact_axis,
        compact_island_axis, compact_transport_geometry, estimated_manifold_wirelength,
        extract_rigid_islands, extract_route_nets, legalize_manifold_routes, machine_blocks,
        materialize_legalized_manifold_routes, occupied_bbox,
        place_distributed_local_manifold_nodes, place_recipe_clusters, plan_local_manifolds,
        route_local_manifold_edges, CompactAxis, CompactIr, PlacedMachineSignature,
        ProductionSignature, RouteTerminalKind,
    };
    use spaghettio_core::common::is_belt_entity;
    use spaghettio_core::density::{entity_footprint, score_density};
    use spaghettio_core::validate::{self, LayoutStyle, Severity};

    for label in [
        "mega-chain-usp2raw",
        "mega-chain-chem5raw",
        "mega-chain-pu4raw",
        "chain-mil5ore",
    ] {
        let fixture = SimFixture::find(label);
        let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            fixture.target,
            fixture.rate,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let layout = fixture.compose_layout();
        let production = ProductionSignature::from_solver(&sr).unwrap();
        let placed = PlacedMachineSignature::from_layout(&layout);
        let nets = extract_route_nets(&layout);
        let islands = extract_rigid_islands(&layout);
        assert!(!production.machines.is_empty());
        assert!(!placed.0.is_empty());
        for edge in production.edges.iter().filter(|edge| !edge.is_fluid) {
            assert!(
                nets.iter().any(|net| {
                    net.item == edge.item
                        && net.terminals.iter().any(|terminal| {
                            terminal.kind == RouteTerminalKind::ProducerDrop
                                && terminal
                                    .recipe
                                    .as_ref()
                                    .is_some_and(|recipe| edge.producer_recipes.contains(recipe))
                        })
                        && net.terminals.iter().any(|terminal| {
                            terminal.kind == RouteTerminalKind::ConsumerPickup
                                && terminal.recipe.as_deref() == Some(edge.consumer_recipe.as_str())
                        })
                }),
                "{label}: no route intent covers {edge:?}"
            );
        }

        let original = machine_blocks(&layout);
        let original_bbox = occupied_bbox(&original);
        let mut compacted = original.clone();
        for _ in 0..8 {
            compacted = compact_axis(&compacted, CompactAxis::X, 1);
            compacted = compact_axis(&compacted, CompactAxis::Y, 1);
        }
        for (i, a) in compacted.iter().enumerate() {
            for b in &compacted[i + 1..] {
                assert!(
                    !blocks_overlap(a, b),
                    "{label}: blocks {} and {} overlap",
                    a.id,
                    b.id
                );
            }
        }
        let compact_bbox = occupied_bbox(&compacted);
        let before = i64::from(original_bbox.0) * i64::from(original_bbox.1);
        let after = i64::from(compact_bbox.0) * i64::from(compact_bbox.1);
        println!(
            "{label}: machines={} machine-bbox={}x{} -> {}x{} ({:+.1}%)",
            original.len(),
            original_bbox.0,
            original_bbox.1,
            compact_bbox.0,
            compact_bbox.1,
            (after as f64 / before as f64 - 1.0) * 100.0,
        );

        let island_source = occupied_bbox(
            &islands
                .iter()
                .map(|island| island.block.clone())
                .collect::<Vec<_>>(),
        );
        let mut island_compacted = islands.clone();
        for _ in 0..8 {
            island_compacted = compact_island_axis(&island_compacted, CompactAxis::X, 1);
            island_compacted = compact_island_axis(&island_compacted, CompactAxis::Y, 1);
        }
        let island_after = occupied_bbox(
            &island_compacted
                .iter()
                .map(|island| island.block.clone())
                .collect::<Vec<_>>(),
        );
        let ir = CompactIr::from_source(&layout, &sr);
        assert_eq!(ir.islands, islands);
        assert_eq!(ir.route_nets, nets);
        let manifolds = build_manifold_nets(&ir, &island_compacted).unwrap();
        let (clustered_islands, clusters) = place_recipe_clusters(&ir, 1);
        let clustered_manifolds = build_manifold_nets(&ir, &clustered_islands).unwrap();
        let local_plans = plan_local_manifolds(&clustered_islands, &clustered_manifolds, 1);
        let local_graphs: Vec<_> = local_plans.iter().map(build_local_manifold_graph).collect();
        let placed_hubs =
            place_distributed_local_manifold_nodes(&clustered_islands, &local_graphs, 3).unwrap();
        if label == "chain-mil5ore" {
            let routed =
                route_local_manifold_edges(&clustered_islands, &local_graphs, &placed_hubs)
                    .unwrap();
            let edge_count: usize = local_graphs.iter().map(|graph| graph.edges.len()).sum();
            assert!(
                routed.unroutable.is_empty(),
                "{label}: {} manifold edges could not be routed",
                routed.unroutable.len(),
            );
            assert_eq!(routed.routes.len(), edge_count);
            assert!(routed.routes.iter().all(|route| route.path.len() >= 2));
            let legalized = legalize_manifold_routes(&routed.routes);
            assert_eq!(legalized.routes.len(), routed.routes.len());
            if legalized.unresolved_routes == 0 {
                assert!(materialize_legalized_manifold_routes(&legalized.routes).is_ok());
            } else {
                assert!(materialize_legalized_manifold_routes(&legalized.routes).is_err());
            }
            let mut crossed_merge_edges = 0usize;
            let mut crossed_distribute_edges = 0usize;
            for route in routed
                .routes
                .iter()
                .filter(|route| !route.crossings.is_empty())
            {
                let graph = local_graphs
                    .iter()
                    .find(|graph| graph.item == route.item)
                    .unwrap();
                let touches_role =
                    |endpoint: &spaghettio_core::bus::compaction::ManifoldEndpoint, role| {
                        match endpoint {
                            spaghettio_core::bus::compaction::ManifoldEndpoint::NodeInput {
                                node,
                                ..
                            }
                            | spaghettio_core::bus::compaction::ManifoldEndpoint::NodeOutput {
                                node,
                                ..
                            } => graph.nodes[*node].role == role,
                            _ => false,
                        }
                    };
                if touches_role(
                    &route.edge.from,
                    spaghettio_core::bus::compaction::BalancerNodeRole::Merge,
                ) || touches_role(
                    &route.edge.to,
                    spaghettio_core::bus::compaction::BalancerNodeRole::Merge,
                ) {
                    crossed_merge_edges += 1;
                }
                if touches_role(
                    &route.edge.from,
                    spaghettio_core::bus::compaction::BalancerNodeRole::Distribute,
                ) || touches_role(
                    &route.edge.to,
                    spaghettio_core::bus::compaction::BalancerNodeRole::Distribute,
                ) {
                    crossed_distribute_edges += 1;
                }
            }
            let mut occupied_axes = FxHashMap::<(i32, i32), u8>::default();
            let mut same_axis_tiles = 0usize;
            let mut perpendicular_tiles = 0usize;
            for route in &routed.routes {
                let mut route_axes = FxHashMap::<(i32, i32), u8>::default();
                for pair in route.path.windows(2) {
                    let axis = if pair[0].0 == pair[1].0 { 1 } else { 2 };
                    *route_axes.entry(pair[0]).or_default() |= axis;
                    *route_axes.entry(pair[1]).or_default() |= axis;
                }
                for (tile, axis) in route_axes {
                    if let Some(previous) = occupied_axes.get(&tile) {
                        if previous & axis != 0 {
                            same_axis_tiles += 1;
                        } else {
                            perpendicular_tiles += 1;
                        }
                    }
                    *occupied_axes.entry(tile).or_default() |= axis;
                }
            }
            println!(
                "{label}: routed {} manifold edges, {} paths cross prior paths, \
                 conflicts={same_axis_tiles} same-axis/{perpendicular_tiles} perpendicular tiles; \
                 legalization={} UG spans, {} residual routes/{} tiles; \
                 crossed roles={crossed_merge_edges} merge/{crossed_distribute_edges} distribute",
                routed.routes.len(),
                routed
                    .routes
                    .iter()
                    .filter(|route| !route.crossings.is_empty())
                    .count(),
                legalized.underground_spans,
                legalized.unresolved_routes,
                legalized.unresolved_tiles,
            );
        }
        let mut hub_tiles = FxHashSet::default();
        for hub in &placed_hubs {
            for entity in &hub.entities {
                let (width, height) = entity_footprint(entity);
                for x in entity.x..entity.x + width as i32 {
                    for y in entity.y..entity.y + height as i32 {
                        assert!(
                            hub_tiles.insert((x, y)),
                            "{label}: stamped hub entity overlap at ({x},{y})",
                        );
                    }
                }
            }
        }

        let clustered_bbox = occupied_bbox(
            &clustered_islands
                .iter()
                .map(|island| island.block.clone())
                .collect::<Vec<_>>(),
        );
        for (idx, island) in clustered_islands.iter().enumerate() {
            for other in &clustered_islands[idx + 1..] {
                assert!(
                    !blocks_overlap(&island.block, &other.block),
                    "{label}: clustered islands {} and {} overlap",
                    island.id,
                    other.id,
                );
            }
        }
        let mut non_monotone = Vec::new();
        for manifold in &manifolds {
            assert!(
                manifold.producers().next().is_some(),
                "{label}: {} manifold has no producer/input",
                manifold.item,
            );
            assert!(
                manifold.consumers().next().is_some(),
                "{label}: {} manifold has no consumer/output",
                manifold.item,
            );
            assert!(
                manifold.planned_rate > 0,
                "{label}: {} has no planned rate",
                manifold.item
            );
            let producer_max = manifold
                .producers()
                .filter(|terminal| terminal.island_id.is_some())
                .map(|terminal| terminal.x)
                .max();
            let consumer_min = manifold
                .consumers()
                .filter(|terminal| terminal.island_id.is_some())
                .map(|terminal| terminal.x)
                .min();
            if producer_max.zip(consumer_min).is_some_and(|(p, c)| p > c) {
                non_monotone.push(manifold.item.clone());
            }
        }
        let before = i64::from(island_source.0) * i64::from(island_source.1);
        let after = i64::from(island_after.0) * i64::from(island_after.1);
        println!(
            "{label}: islands={} terminals={} manifolds={} island-bbox={}x{} -> {}x{} ({:+.1}%)",
            islands.len(),
            islands
                .iter()
                .map(|island| island.terminals.len())
                .sum::<usize>(),
            manifolds.len(),
            island_source.0,
            island_source.1,
            island_after.0,
            island_after.1,
            (after as f64 / before as f64 - 1.0) * 100.0,
        );
        println!("{label}: non-monotone manifolds={non_monotone:?}");
        println!(
            "{label}: express manifold lanes total={} max={}",
            manifolds
                .iter()
                .map(|manifold| manifold.required_belts(45.0))
                .sum::<u32>(),
            manifolds
                .iter()
                .map(|manifold| manifold.required_belts(45.0))
                .max()
                .unwrap_or(0),
        );
        println!(
            "{label}: recipe clusters={} bbox={}x{}, weighted-wirelength={} -> {}",
            clusters.len(),
            clustered_bbox.0,
            clustered_bbox.1,
            estimated_manifold_wirelength(&manifolds),
            estimated_manifold_wirelength(&clustered_manifolds),
        );
        println!(
            "{label}: local hubs={} lanes={} merger-ready={} distributor-ready={}",
            local_plans.len(),
            local_plans.iter().map(|plan| plan.belt_count).sum::<u32>(),
            local_plans
                .iter()
                .filter(|plan| plan.all_mergers_stampable)
                .count(),
            local_plans
                .iter()
                .filter(|plan| plan.all_distributors_stampable)
                .count(),
        );
        println!(
            "{label}: stamped hub nodes={} entities={}",
            placed_hubs.iter().map(|hub| hub.nodes.len()).sum::<usize>(),
            placed_hubs
                .iter()
                .map(|hub| hub.entities.len())
                .sum::<usize>(),
        );
        assert_eq!(local_plans.len(), clustered_manifolds.len());
        for ((plan, graph), manifold) in local_plans
            .iter()
            .zip(&local_graphs)
            .zip(&clustered_manifolds)
        {
            assert!(
                plan.all_mergers_stampable,
                "{label}: {} merger hierarchy not stampable",
                plan.item
            );
            assert!(
                plan.all_distributors_stampable,
                "{label}: {} distributor hierarchy not stampable",
                plan.item
            );
            assert_eq!(
                plan.lane_groups
                    .iter()
                    .map(|group| group.producers.len())
                    .sum::<usize>(),
                manifold.producers().count(),
            );
            assert_eq!(
                plan.lane_groups
                    .iter()
                    .map(|group| group.consumers.len())
                    .sum::<usize>(),
                manifold.consumers().count(),
            );
            for terminal in manifold.producers() {
                assert!(graph.edges.iter().any(|edge| {
                    edge.from
                        == spaghettio_core::bus::compaction::ManifoldEndpoint::Terminal(
                            terminal.clone(),
                        )
                }));
            }
            for terminal in manifold.consumers() {
                assert!(graph.edges.iter().any(|edge| {
                    edge.to
                        == spaghettio_core::bus::compaction::ManifoldEndpoint::Terminal(
                            terminal.clone(),
                        )
                }));
            }
        }

        let runnable = compact_transport_geometry(&layout);
        assert_eq!(
            PlacedMachineSignature::from_layout(&runnable),
            placed,
            "{label}: runnable post-pass changed machines",
        );
        let issues = match validate::validate(&runnable, Some(&sr), LayoutStyle::Bus) {
            Ok(issues) => issues,
            Err(error) => error.issues,
        };
        let errors: Vec<_> = issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "{label}: runnable post-pass errors: {errors:?}"
        );
        let underground_warnings = issues
            .iter()
            .filter(|issue| {
                issue.severity != Severity::Error && issue.category == "underground-belt"
            })
            .count();
        let source_belts = layout
            .entities
            .iter()
            .filter(|entity| is_belt_entity(&entity.name))
            .count();
        let compact_belts = runnable
            .entities
            .iter()
            .filter(|entity| is_belt_entity(&entity.name))
            .count();
        println!(
            "{label}: runnable={}x{} -> {}x{}, belts={} -> {} ({:+.1}%)",
            layout.width,
            layout.height,
            runnable.width,
            runnable.height,
            source_belts,
            compact_belts,
            (compact_belts as f64 / source_belts as f64 - 1.0) * 100.0,
        );
        println!("{label}: underground warnings={underground_warnings}");
        let source_density = score_density(&layout, (1, 1));
        let compact_density = score_density(&runnable, (1, 1));
        println!(
            "{label}: occupied tiles={} -> {} ({:+.1}%), content area={} -> {}",
            source_density.filled_tiles,
            compact_density.filled_tiles,
            (compact_density.filled_tiles as f64 / source_density.filled_tiles as f64 - 1.0)
                * 100.0,
            u64::from(source_density.content_bbox_width)
                * u64::from(source_density.content_bbox_height),
            u64::from(compact_density.content_bbox_width)
                * u64::from(compact_density.content_bbox_height),
        );
    }
}

#[test]
#[ignore = "RFC-057 runnable whitespace-compaction baseline"]
fn rfc057_strip_empty_columns_mil5ore() {
    use spaghettio_core::bus::compaction::{
        compact_island_axis, compact_transport_geometry, compact_validated_geometry,
        extract_rigid_islands, extract_route_nets, occupied_bbox, strip_empty_columns, CompactAxis,
        PlacedMachineSignature, ProductionSignature, RouteTerminalKind,
    };
    use spaghettio_core::common::is_belt_entity;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};

    let fixture = SimFixture::find("chain-mil5ore");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let source = fixture.compose_layout();
    let compacted = strip_empty_columns(&source);
    let underground_compacted = compact_transport_geometry(&source);
    let cut_compacted = compact_validated_geometry(&source, &sr);
    let production = ProductionSignature::from_solver(&sr).unwrap();
    let nets = extract_route_nets(&source);
    let islands = extract_rigid_islands(&source);
    for edge in production.edges.iter().filter(|edge| !edge.is_fluid) {
        assert!(
            nets.iter().any(|net| {
                net.item == edge.item
                    && net.terminals.iter().any(|terminal| {
                        terminal.kind == RouteTerminalKind::ProducerDrop
                            && terminal
                                .recipe
                                .as_ref()
                                .is_some_and(|recipe| edge.producer_recipes.contains(recipe))
                    })
                    && net.terminals.iter().any(|terminal| {
                        terminal.kind == RouteTerminalKind::ConsumerPickup
                            && terminal.recipe.as_deref() == Some(edge.consumer_recipe.as_str())
                    })
            }),
            "no extracted route net covers edge {edge:?}"
        );
    }
    assert_eq!(
        PlacedMachineSignature::from_layout(&source),
        PlacedMachineSignature::from_layout(&compacted),
    );
    assert_eq!(
        PlacedMachineSignature::from_layout(&source),
        PlacedMachineSignature::from_layout(&underground_compacted),
    );
    assert_eq!(
        PlacedMachineSignature::from_layout(&source),
        PlacedMachineSignature::from_layout(&cut_compacted),
    );
    for (label, candidate) in [
        ("stripped", &compacted),
        ("underground-compacted", &underground_compacted),
        ("cut-compacted", &cut_compacted),
    ] {
        let issues = match validate::validate(candidate, Some(&sr), LayoutStyle::Bus) {
            Ok(v) => v,
            Err(e) => e.issues,
        };
        let errors: Vec<_> = issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "{label} candidate errors: {errors:?}");
    }
    std::fs::create_dir_all("target/tmp").unwrap();
    for (label, layout) in [
        ("rfc057-mil5ore-control", &source),
        ("rfc057-mil5ore-strip", &compacted),
        ("rfc057-mil5ore-underground", &underground_compacted),
        ("rfc057-mil5ore-cut", &cut_compacted),
    ] {
        let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(layout, &sr, label);
        std::fs::write(format!("target/tmp/{label}.bp"), bp).unwrap();
        std::fs::write(
            format!("target/tmp/{label}.manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }
    println!(
        "chain-mil5ore: {}x{} -> {}x{}; entities={}",
        source.width,
        source.height,
        compacted.width,
        compacted.height,
        compacted.entities.len(),
    );
    let source_belts = source
        .entities
        .iter()
        .filter(|entity| is_belt_entity(&entity.name))
        .count();
    let underground_belts = underground_compacted
        .entities
        .iter()
        .filter(|entity| is_belt_entity(&entity.name))
        .count();
    println!(
        "underground candidate: {}x{} entities={} belts={} ({:+.1}% belts)",
        underground_compacted.width,
        underground_compacted.height,
        underground_compacted.entities.len(),
        underground_belts,
        (underground_belts as f64 / source_belts as f64 - 1.0) * 100.0,
    );
    println!(
        "validated-cut candidate: {}x{} entities={}",
        cut_compacted.width,
        cut_compacted.height,
        cut_compacted.entities.len(),
    );
    println!("extracted {} replaceable route nets", nets.len());
    println!(
        "extracted {} rigid production islands: entities={} terminals={} largest={}",
        islands.len(),
        islands
            .iter()
            .map(|island| island.entity_indices.len())
            .sum::<usize>(),
        islands
            .iter()
            .map(|island| island.terminals.len())
            .sum::<usize>(),
        islands
            .iter()
            .map(|island| island.entity_indices.len())
            .max()
            .unwrap_or(0),
    );
    let source_island_bbox = occupied_bbox(
        &islands
            .iter()
            .map(|island| island.block.clone())
            .collect::<Vec<_>>(),
    );
    let mut placed_islands = islands.clone();
    for _ in 0..8 {
        placed_islands = compact_island_axis(&placed_islands, CompactAxis::X, 1);
        placed_islands = compact_island_axis(&placed_islands, CompactAxis::Y, 1);
    }
    let placed_island_bbox = occupied_bbox(
        &placed_islands
            .iter()
            .map(|island| island.block.clone())
            .collect::<Vec<_>>(),
    );
    println!(
        "rigid-island bbox: {}x{} -> {}x{}",
        source_island_bbox.0, source_island_bbox.1, placed_island_bbox.0, placed_island_bbox.1,
    );
    for net in nets.iter().take(12) {
        println!(
            "  net {}: segments={} entities={} terminals={}",
            net.item,
            net.segments.len(),
            net.entity_indices.len(),
            net.terminals.len(),
        );
    }
}

#[test]
#[ignore = "RFC-057 compacted artifact producer"]
fn export_rfc057_compacted_candidates() {
    use spaghettio_core::bus::compaction::compact_validated_geometry;

    std::fs::create_dir_all("target/tmp").unwrap();
    for fixture_label in [
        "mega-chain-chem5raw",
        "mega-chain-pu4raw",
        "mega-chain-usp2raw",
    ] {
        let fixture = SimFixture::find(fixture_label);
        let inputs: FxHashSet<String> =
            fixture.inputs.iter().map(|item| item.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            fixture.target,
            fixture.rate,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let control = fixture.compose_layout();
        let compacted = compact_validated_geometry(&control, &sr);
        for (variant, layout) in [("control", &control), ("compact", &compacted)] {
            let label = format!("rfc057-{fixture_label}-{variant}");
            let (bp, manifest) =
                spaghettio_core::blueprint::export_with_manifest(layout, &sr, &label);
            std::fs::write(format!("target/tmp/{label}.bp"), bp).unwrap();
            std::fs::write(
                format!("target/tmp/{label}.manifest.json"),
                serde_json::to_string_pretty(&manifest).unwrap(),
            )
            .unwrap();
        }
        println!(
            "{fixture_label}: {}x{} / {} entities -> {}x{} / {} entities",
            control.width,
            control.height,
            control.entities.len(),
            compacted.width,
            compacted.height,
            compacted.entities.len(),
        );
    }
}

/// Artifact producer for the increment-2 sim run.
#[test]
#[ignore = "artifact producer"]
fn export_mega_pu_for_sim() {
    let inputs: FxHashSet<String> = ["iron-ore", "copper-ore", "crude-oil", "water", "coal"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "processing-unit",
        4.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = SimFixture::find("mega-chain-pu4raw").compose_layout();
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&l, &sr, "mega-chain-pu4raw");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/mega-chain-pu4raw.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/mega-chain-pu4raw.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "wrote mega-chain-pu4raw.bp ({} in / {} out)",
        l.boundary_inputs.len(),
        l.boundary_outputs.len()
    );
}

/// Artifact producer for the kill-2 sim run.
#[test]
#[ignore = "artifact producer"]
fn export_mega_chem_for_sim() {
    let inputs: FxHashSet<String> = [
        "iron-ore",
        "copper-ore",
        "crude-oil",
        "water",
        "coal",
        "iron-plate",
        "copper-plate",
        "steel-plate",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "chemical-science-pack",
        5.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = SimFixture::find("mega-chain-chem5raw").compose_layout();
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&l, &sr, "mega-chain-chem5raw");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/mega-chain-chem5raw.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/mega-chain-chem5raw.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "wrote mega-chain-chem5raw.bp ({} in / {} out)",
        l.boundary_inputs.len(),
        l.boundary_outputs.len()
    );
}

/// Artifact producer for the Phase-B flagship sim run.
#[test]
#[ignore = "artifact producer"]
fn export_mega_chain_for_sim() {
    let inputs: FxHashSet<String> = ["iron-ore", "copper-ore", "crude-oil", "water", "coal"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "advanced-circuit",
        2.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = SimFixture::find("mega-chain-ac2raw").compose_layout();
    let (bp, manifest) =
        spaghettio_core::blueprint::export_with_manifest(&l, &sr, "mega-chain-ac2raw");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/mega-chain-ac2raw.bp", &bp).unwrap();
    std::fs::write(
        "target/tmp/mega-chain-ac2raw.manifest.json",
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    println!(
        "wrote target/tmp/mega-chain-ac2raw.bp ({} in / {} out)",
        l.boundary_inputs.len(),
        l.boundary_outputs.len()
    );
}

#[test]
#[ignore = "exploration probe"]
fn probe_mega_cells() {
    use spaghettio_core::bus::cells::mega::compose_mega_calibrated;
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    for (label, item, rate, inputs) in [
        (
            "plastic2",
            "plastic-bar",
            2.0,
            &["crude-oil", "water", "coal"][..],
        ),
        (
            "plastic5",
            "plastic-bar",
            5.0,
            &["crude-oil", "water", "coal"][..],
        ),
        ("sulfur2", "sulfur", 2.0, &["crude-oil", "water"][..]),
    ] {
        match compose_mega_calibrated(item, rate, inputs) {
            Ok((sr, l)) => {
                let d = validate::validate(&l, Some(&sr), LayoutStyle::Bus);
                match d {
                    Ok(is) => {
                        let e = is.iter().filter(|i| i.severity == Severity::Error).count();
                        println!(
                            "{label}: {}x{} {} entities, {} errors / {} warnings; feeds {:?}",
                            l.width,
                            l.height,
                            l.entities.len(),
                            e,
                            is.len() - e,
                            l.boundary_inputs
                                .iter()
                                .map(|b| (b.item.clone(), b.x))
                                .collect::<Vec<_>>()
                        );
                        for i in is.iter().take(8) {
                            println!("   [{:?}] {} {}", i.severity, i.category, i.message);
                        }
                    }
                    Err(er) => println!(
                        "{label}: validate ERR {}",
                        format!("{er}").lines().next().unwrap_or("")
                    ),
                }
            }
            Err(e) => println!("{label}: REFUSED {e}"),
        }
    }
}

#[test]
#[ignore = "debug probe"]
fn probe_mil5_errors() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, LayoutStyle};
    for (label, item, rate, inputs) in [
        (
            "mil5-ore",
            "military-science-pack",
            5.0,
            &["iron-ore", "copper-ore", "stone", "coal"][..],
        ),
        (
            "ec30",
            "electronic-circuit",
            30.0,
            &["iron-plate", "copper-plate"][..],
        ),
    ] {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            rate,
            &inputs_set,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        println!("== {label}: {} specs ==", sr.machines.len());
        match compose_chain(&sr) {
            Ok(l) => match validate::validate(&l, Some(&sr), LayoutStyle::Bus) {
                Ok(_) => println!("   validates OK"),
                Err(er) => {
                    for line in format!("{er}")
                        .lines()
                        .filter(|l| l.contains("error"))
                        .take(8)
                    {
                        println!("   {line}");
                    }
                }
            },
            Err(e) => println!("   REFUSED: {e}"),
        }
    }
}

/// Manhattan MST over a terminal set — an obstacle-free lower bound on the
/// belt tiles a perfect shared-trunk router would need for one commodity.
/// Prim's, O(n²); nets here are tens of terminals, not thousands.
fn manhattan_mst_cost(points: &[(i32, i32)]) -> i64 {
    if points.len() < 2 {
        return 0;
    }
    let n = points.len();
    let mut in_tree = vec![false; n];
    let mut best = vec![i64::MAX; n];
    best[0] = 0;
    let mut total = 0i64;
    for _ in 0..n {
        let mut pick = usize::MAX;
        for i in 0..n {
            if !in_tree[i] && (pick == usize::MAX || best[i] < best[pick]) {
                pick = i;
            }
        }
        in_tree[pick] = true;
        total += best[pick];
        let (px, py) = points[pick];
        for i in 0..n {
            if in_tree[i] {
                continue;
            }
            let (qx, qy) = points[i];
            let d = i64::from((px - qx).abs()) + i64::from((py - qy).abs());
            if d < best[i] {
                best[i] = d;
            }
        }
    }
    total
}

/// Map a point through the snake-fold coordinate transform.
///
/// `bounds` is the fold partition (`[0, f1, .., fk, width]`). Even segments
/// keep their X orientation, odd segments mirror; each segment drops by
/// `height + gap`. This is `fold_snake`'s transform with the U-turn and
/// reconnection machinery removed — geometry only.
///
/// `y_mirror` additionally flips odd segments vertically. `fold_snake` does
/// not do this, and it should matter: an X-only fold puts the *bottom* edge
/// of segment k against the *top* edge of segment k+1, so structures that sat
/// at the same height in the ribbon (a trunk, say) end up `height + gap`
/// apart. Flipping alternate segments in Y makes them meet instead — which is
/// what a two-sided main bus actually looks like.
fn fold_point(
    x: i32,
    y: i32,
    bounds: &[i32],
    height: i32,
    gap: i32,
    y_mirror: bool,
) -> (i32, i32) {
    let n_segs = bounds.len() - 1;
    let seg = (0..n_segs)
        .find(|&k| x >= bounds[k] && x < bounds[k + 1])
        .unwrap_or(n_segs - 1);
    let odd = seg % 2 == 1;
    let nx = if odd {
        bounds[seg + 1] - 1 - x
    } else {
        x - bounds[seg]
    };
    let local_y = if odd && y_mirror { height - 1 - y } else { y };
    (nx, local_y + (seg as i32) * (height + gap))
}

/// Does folding actually shorten the *routing problem*?
///
/// `probe_fold_search_mil5` measures fold-as-implemented, which can only add
/// entities (it preserves every belt and adds U-turns). That says nothing
/// about fold-as-a-stage followed by a re-router. This probe removes the
/// reconnection machinery entirely and asks the prior question: under a pure
/// fold coordinate transform, does the idealized cost of connecting each
/// commodity's terminals go down?
///
/// The metric is the sum over commodity nets of the Manhattan MST over that
/// net's terminals. It ignores obstacles, so it is a *lower bound* on what a
/// real router achieves — but it is measured identically before and after,
/// so the ratio is the honest screen. If MST does not improve, no downstream
/// shortcut search can pay for the fold.
///
/// Run over the whole RFC-057 corpus, because the fold's cost is fixed
/// (`height + gap` of vertical separation per fold) while its benefit scales
/// with width — so the answer should depend on aspect ratio, and mil5 is the
/// thinnest fixture of the four.
#[test]
#[ignore = "exploration probe — fold routing headroom"]
fn probe_fold_routing_headroom() {
    for label in [
        "chain-mil5ore",
        "mega-chain-chem5raw",
        "mega-chain-pu4raw",
        "mega-chain-usp2raw",
    ] {
        println!("\n========== {label} ==========");
        fold_routing_headroom_for(label);
    }
}

fn fold_routing_headroom_for(label: &str) {
    use spaghettio_core::bus::compaction::{
        build_manifold_nets, compact_validated_geometry, CompactIr, RouteTerminalKind,
    };

    let fixture = SimFixture::find(label);
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();

    let bus = fixture.compose_layout();
    let compact = compact_validated_geometry(&bus, &sr);
    let belt_entities = compact
        .entities
        .iter()
        .filter(|e| spaghettio_core::common::is_belt_entity(&e.name))
        .count();
    println!(
        "compact baseline: {}x{} = {} tiles, {} entities ({} belts)",
        compact.width,
        compact.height,
        compact.width * compact.height,
        compact.entities.len(),
        belt_entities,
    );

    // One net extraction, then two coordinate mappings — the terminal set is
    // identical in both cases, so the comparison is apples-to-apples.
    let ir = CompactIr::from_source(&compact, &sr);
    let nets = build_manifold_nets(&ir, &ir.islands).expect("identity placement must build nets");

    let w = compact.width;
    let h = compact.height;
    let gap = 2;

    // Two metrics, because they fail in opposite directions.
    //
    // MST forces every terminal of a commodity into ONE tree. Real delivery
    // is a forest: smelter bank A feeds the consumers beside A and never
    // connects to bank B. That is why MST here exceeds the actual belt count
    // — it pays for inter-cluster links the factory never builds. Folding
    // shortens exactly those links, so MST alone would flatter the result.
    //
    // `nearest-source` is the honest one: for every consuming terminal, the
    // distance to the closest producing terminal of that item. It is
    // forest-shaped like real delivery, and it measures the thing the
    // shortcut hypothesis actually claims — that folding puts consumers
    // nearer their producers.
    let cost_at = |bounds: &[i32], y_mirror: bool| -> (i64, i64, i64, i32, i32) {
        let mut mst_total = 0i64;
        let mut near_total = 0i64;
        let mut near_weighted = 0i64;
        let (mut max_x, mut max_y) = (0i32, 0i32);
        for net in &nets {
            let mut all = Vec::with_capacity(net.terminals.len());
            let mut sources = Vec::new();
            let mut sinks = Vec::new();
            for t in &net.terminals {
                let p = fold_point(t.x, t.y, bounds, h, gap, y_mirror);
                max_x = max_x.max(p.0);
                max_y = max_y.max(p.1);
                all.push(p);
                match t.kind {
                    RouteTerminalKind::ProducerDrop | RouteTerminalKind::BoundaryInput => {
                        sources.push(p)
                    }
                    RouteTerminalKind::ConsumerPickup | RouteTerminalKind::BoundaryOutput => {
                        sinks.push(p)
                    }
                }
            }
            mst_total += manhattan_mst_cost(&all);

            let mut net_near = 0i64;
            for &(sx, sy) in &sinks {
                let closest = sources
                    .iter()
                    .map(|&(px, py)| i64::from((sx - px).abs()) + i64::from((sy - py).abs()))
                    .min();
                if let Some(d) = closest {
                    net_near += d;
                }
            }
            near_total += net_near;
            // planned_rate is fixed-point; scale down so the weighted total
            // stays readable rather than exact.
            near_weighted += net_near * net.planned_rate.max(1) / 1000;
        }
        (mst_total, near_total, near_weighted, max_x + 1, max_y + 1)
    };

    let flat: Vec<i32> = vec![0, w];
    let (base_mst, base_near, base_weighted, base_w, base_h) = cost_at(&flat, false);
    let terminal_count: usize = nets.iter().map(|net| net.terminals.len()).sum();
    println!(
        "unfolded: MST={base_mst}  nearest-source={base_near}  \
         rate-weighted={base_weighted}  terminal-bbox={base_w}x{base_h}"
    );
    println!(
        "  ({} nets, {terminal_count} terminals, {belt_entities} actual belts)",
        nets.len()
    );

    for y_mirror in [false, true] {
        println!(
            "\n--- even folds ({}) ---",
            if y_mirror {
                "X-mirror + Y-mirror on odd segments"
            } else {
                "X-mirror only — what fold_snake does"
            }
        );
        for k in 1..=6usize {
            let mut bounds = vec![0];
            for i in 1..=k {
                bounds.push(w * i as i32 / (k + 1) as i32);
            }
            bounds.push(w);
            let (mst, near, weighted, fw, fh) = cost_at(&bounds, y_mirror);
            println!(
                "  {k} fold(s): MST {:+.1}%  nearest-source {:+.1}%  \
                 rate-weighted {:+.1}%  bbox={fw}x{fh} aspect={:.2}",
                (mst as f64 / base_mst as f64 - 1.0) * 100.0,
                (near as f64 / base_near as f64 - 1.0) * 100.0,
                (weighted as f64 / base_weighted as f64 - 1.0) * 100.0,
                if fh > fw {
                    fh as f64 / fw as f64
                } else {
                    fw as f64 / fh as f64
                },
            );
        }
    }

    // Even splits are an arbitrary choice of fold line, and the erratic
    // rate-weighted numbers above look like the consequence: a fold that
    // happens to cut between a high-rate producer and its consumer pays for
    // that pair, one that lands in a quiet seam does not. Choosing fold
    // columns to minimise the rate-weighted cost is the fair test of the
    // idea. Greedy, because this is a screen and not the final placer.
    // Rate-weighted nearest-source only — no MST. The greedy sweep evaluates
    // this thousands of times and MST is O(n²) per net, which is unaffordable
    // on the 1,000-plus-terminal fixtures.
    let weighted_only = |bounds: &[i32], y_mirror: bool| -> i64 {
        let mut total = 0i64;
        for net in &nets {
            let mut sources = Vec::new();
            let mut sinks = Vec::new();
            for t in &net.terminals {
                let p = fold_point(t.x, t.y, bounds, h, gap, y_mirror);
                match t.kind {
                    RouteTerminalKind::ProducerDrop | RouteTerminalKind::BoundaryInput => {
                        sources.push(p)
                    }
                    RouteTerminalKind::ConsumerPickup | RouteTerminalKind::BoundaryOutput => {
                        sinks.push(p)
                    }
                }
            }
            let mut net_near = 0i64;
            for &(sx, sy) in &sinks {
                if let Some(d) = sources
                    .iter()
                    .map(|&(px, py)| i64::from((sx - px).abs()) + i64::from((sy - py).abs()))
                    .min()
                {
                    net_near += d;
                }
            }
            total += net_near * net.planned_rate.max(1) / 1000;
        }
        total
    };

    println!("\n--- greedily placed folds (best of X-only / X+Y mirror) ---");
    // ~40 candidate columns regardless of fixture width, so the sweep costs
    // the same on a 550-wide layout and a 2,400-wide one.
    let step = ((w / 40).max(4)) as usize;
    for k in 1..=4usize {
        let mut chosen: Vec<i32> = Vec::new();
        let mut best_cost = base_weighted;
        let mut best_mirror = false;
        for _ in 0..k {
            let mut round: Option<(i64, i32, bool)> = None;
            for f in (10..w - 10).step_by(step) {
                if chosen.contains(&f) {
                    continue;
                }
                let mut trial = chosen.clone();
                trial.push(f);
                trial.sort();
                let mut bounds = vec![0];
                bounds.extend_from_slice(&trial);
                bounds.push(w);
                for y_mirror in [false, true] {
                    let weighted = weighted_only(&bounds, y_mirror);
                    if round.is_none() || weighted < round.unwrap().0 {
                        round = Some((weighted, f, y_mirror));
                    }
                }
            }
            let Some((cost, f, mirror)) = round else { break };
            chosen.push(f);
            chosen.sort();
            best_cost = cost;
            best_mirror = mirror;
        }
        println!(
            "  {k} fold(s) at {chosen:?}{}: rate-weighted {:+.1}%",
            if best_mirror { " (Y-mirrored)" } else { "" },
            (best_cost as f64 / base_weighted as f64 - 1.0) * 100.0,
        );
    }
}

/// Export the compacted control and its folded counterpart for the sim
/// harness. Belt lane semantics are the thing folding is most likely to
/// break — a corner that is a 90° turn carries both lanes, one that has
/// become a sideload carries one — and only Factorio adjudicates that.
#[test]
#[ignore = "sim export — run spaghettio-sim against the written artifacts"]
fn export_fold_pair_for_sim() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, fold_snake};

    let fixture = SimFixture::find("chain-mil5ore");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let compact = compact_validated_geometry(&fixture.compose_layout(), &sr);
    // Snap to a LEGAL column rather than the bare midpoint: `width / 2` is not
    // guaranteed cuttable and refused with CutsEntity once compaction changed
    // the width. The search snaps; this has to as well.
    let legal = spaghettio_core::bus::compaction::legal_fold_columns(&compact);
    let snap = |t: i32| {
        legal
            .iter()
            .copied()
            .min_by_key(|&f| (f - t).abs())
            .expect("layout must have a legal fold column")
    };
    let mid = snap(compact.width / 2);
    let folded = fold_snake(&compact, &[mid]).expect("midpoint fold must succeed");
    // The 3-fold the search picks (#492). Exported alongside the verified pair
    // so one sim run compares control / 1-fold / 3-fold on identical inputs.
    let fold3 = fold_snake(&compact, &[138, 276, 414]).expect("3-fold must succeed");

    std::fs::create_dir_all("target/tmp").unwrap();
    for (tag, l) in [
        ("mil5-compact", &compact),
        ("mil5-fold1", &folded),
        ("mil5-fold3", &fold3),
    ] {
        let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(l, &sr, tag);
        std::fs::write(format!("target/tmp/{tag}.bp"), &bp).unwrap();
        std::fs::write(
            format!("target/tmp/{tag}.manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        println!(
            "wrote target/tmp/{tag}.bp — {}x{}, {} entities, {} in / {} out",
            l.width,
            l.height,
            l.entities.len(),
            l.boundary_inputs.len(),
            l.boundary_outputs.len(),
        );
    }
}

/// Dump source-vs-folded geometry as JSON for the visual report.
///
/// One character per tile, classified by entity family, so the renderer can
/// colour belts, machines, pipes and the rest differently without shipping
/// per-entity records for tens of thousands of entities.
#[test]
#[ignore = "report export — feeds the fold visualisation"]
fn export_fold_report_json() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, search_snake_fold};
    use spaghettio_core::models::LayoutResult;

    fn class_of(name: &str) -> char {
        use spaghettio_core::common::*;
        if is_machine_entity(name) {
            'm'
        } else if is_splitter(name) {
            's'
        } else if is_ug_belt(name) {
            'u'
        } else if is_belt_entity(name) {
            'b'
        } else if is_inserter(name) {
            'i'
        } else if name.starts_with("pipe") {
            'p'
        } else if name.ends_with("electric-pole") || name == "substation" {
            'e'
        } else {
            'o'
        }
    }

    fn grid(l: &LayoutResult) -> String {
        let (w, h) = (l.width.max(1) as usize, l.height.max(1) as usize);
        let mut g = vec![b'.'; w * h];
        for e in &l.entities {
            let (ew, eh) = spaghettio_core::common::entity_size(&e.name);
            let c = class_of(&e.name) as u8;
            for dx in 0..(ew as i32).max(1) {
                for dy in 0..(eh as i32).max(1) {
                    let (x, y) = (e.x + dx, e.y + dy);
                    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                        let i = y as usize * w + x as usize;
                        // machines win over belts so footprints read clearly
                        if g[i] == b'.' || c == b'm' {
                            g[i] = c;
                        }
                    }
                }
            }
        }
        String::from_utf8(g).unwrap()
    }

    fn counts(l: &LayoutResult) -> String {
        let mut m = std::collections::BTreeMap::new();
        for e in &l.entities {
            *m.entry(class_of(&e.name)).or_insert(0usize) += 1;
        }
        m.iter()
            .map(|(k, v)| format!("\"{k}\":{v}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    struct Case {
        label: &'static str,
        arch: &'static str,
        item: &'static str,
        rate: f64,
        inputs: &'static [&'static str],
        machine: &'static str,
        cell: bool,
    }
    let cases = [
        Case { label: "chain-mil5ore", arch: "cell", item: "military-science-pack", rate: 5.0,
               inputs: &["iron-ore", "copper-ore", "stone", "coal"], machine: "assembling-machine-3", cell: true },
        Case { label: "gear15-ore", arch: "bus", item: "iron-gear-wheel", rate: 15.0,
               inputs: &["iron-ore"], machine: "assembling-machine-2", cell: false },
        Case { label: "ec10-ore", arch: "bus", item: "electronic-circuit", rate: 10.0,
               inputs: &["iron-ore", "copper-ore"], machine: "assembling-machine-1", cell: false },
    ];

    let mut out = String::from("[\n");
    for (n, c) in cases.iter().enumerate() {
        let inputs_set: FxHashSet<String> = c.inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            c.item, c.rate, &inputs_set, &MachinePalette::default(), c.machine,
            &FxHashSet::default(), QualityTier::Normal,
        )
        .unwrap();
        let base = if c.cell {
            SimFixture::find(c.label).compose_layout()
        } else {
            layout::build_bus_layout(&sr, layout::LayoutOptions::default()).unwrap()
        };
        let compact = compact_validated_geometry(&base, &sr);
        let search = search_snake_fold(&compact, &sr, 4);
        let Some(found) = &search.best else {
            println!("{}: no fold, skipped", c.label);
            continue;
        };
        let f = &found.layout;
        if n > 0 {
            out.push_str(",\n");
        }
        out.push_str(&format!(
            "{{\"label\":\"{}\",\"arch\":\"{}\",\"item\":\"{}\",\"rate\":{},\"folds\":{:?},\n\
             \"src\":{{\"w\":{},\"h\":{},\"n\":{},\"counts\":{{{}}},\"grid\":\"{}\"}},\n\
             \"fold\":{{\"w\":{},\"h\":{},\"n\":{},\"counts\":{{{}}},\"grid\":\"{}\"}}}}",
            c.label, c.arch, c.item, c.rate, found.folds,
            compact.width, compact.height, compact.entities.len(), counts(&compact), grid(&compact),
            f.width, f.height, f.entities.len(), counts(f), grid(f),
        ));
        println!(
            "{}: {}x{} -> {}x{} ({} -> {} entities)",
            c.label, compact.width, compact.height, f.width, f.height,
            compact.entities.len(), f.entities.len()
        );
    }
    out.push_str("\n]\n");
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write("target/tmp/fold-report.json", out).unwrap();
    println!("wrote target/tmp/fold-report.json");
}

/// Export `mega-chain-chem5raw` at its registry-declared capacity, for
/// re-blessing its pinned geometry after a change.
///
/// The registry pin carries a SIM-VERIFIED claim, so moving the hash without
/// re-measuring would attach yesterday's evidence to today's factory — the
/// exact failure its own assert message warns about.
#[test]
#[ignore = "sim export — re-bless the chem5 registry pin"]
fn export_chem5_for_rebless() {
    let fixture = SimFixture::find("mega-chain-chem5raw");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target, fixture.rate, &inputs, &MachinePalette::default(),
        "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
    )
    .unwrap();
    let mut l = fixture.compose_layout();
    // Registry entry declares inserter_capacity 2 / stacking 1.
    l.inserter_capacity = 2;
    let tag = "chem5-rebless";
    let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&l, &sr, tag);
    std::fs::create_dir_all("target/tmp").unwrap();
    std::fs::write(format!("target/tmp/{tag}.bp"), &bp).unwrap();
    std::fs::write(
        format!("target/tmp/{tag}.manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    ).unwrap();
    println!(
        "wrote target/tmp/{tag}.bp — {}x{}, {} entities, hash {:016x}",
        l.width, l.height, l.entities.len(),
        spaghettio_core::bus::cells::registry::geometry_hash(&l),
    );
}

/// How often does the NORMAL pipeline emit a fragmented pole network?
///
/// Decides whether `check_pole_network_connectivity` can be promoted from
/// Warning to Error: a factory whose poles form two islands does not run, but
/// promoting a check that already fires on ordinary output would just break
/// every build. Measures rather than assumes.
#[test]
#[ignore = "measurement probe — pole connectivity across the pipeline"]
fn probe_pole_connectivity_census() {
    use spaghettio_core::bus::compaction::compact_validated_geometry;
    use spaghettio_core::power_wires::{disconnected_poles, wires_for};

    let cases: &[(&str, &str, f64, &[&str], &str)] = &[
        ("gear15-ore", "iron-gear-wheel", 15.0, &["iron-ore"], "assembling-machine-2"),
        ("gear15-plate", "iron-gear-wheel", 15.0, &["iron-plate"], "assembling-machine-2"),
        ("ec10-ore", "electronic-circuit", 10.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("ec15-plate", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"], "assembling-machine-2"),
        ("belt5-ore", "transport-belt", 5.0, &["iron-ore"], "assembling-machine-2"),
        ("insert3-ore", "inserter", 3.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("plastic10", "plastic-bar", 10.0, &["coal", "petroleum-gas"], "chemical-plant"),
    ];

    let mut dirty_raw = 0;
    let mut dirty_compact = 0;
    let mut total = 0;
    for (label, item, rate, inputs, machine) in cases {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve_with_palette_exclusions_and_quality(
            item, *rate, &inputs_set, &MachinePalette::default(), machine,
            &FxHashSet::default(), QualityTier::Normal,
        ) else { println!("{label}: solver refused"); continue };
        let Ok(bus) = layout::build_bus_layout(&sr, layout::LayoutOptions::default()) else {
            println!("{label}: layout refused"); continue
        };
        let poles = |l: &spaghettio_core::models::LayoutResult| {
            let n = l.entities.iter()
                .filter(|e| e.name.ends_with("electric-pole") || e.name == "substation")
                .count();
            (n, disconnected_poles(&l.entities, &wires_for(l)).len())
        };
        let (bn, bd) = poles(&bus);
        let compact = compact_validated_geometry(&bus, &sr);
        let (cn, cd) = poles(&compact);
        total += 1;
        if bd > 0 { dirty_raw += 1; }
        if cd > 0 { dirty_compact += 1; }
        println!(
            "{label:<14} bus {bd:>3}/{bn:<4} disconnected   compacted {cd:>3}/{cn:<4}{}",
            if cd > bd { "   <-- COMPACTION MADE IT WORSE" } else { "" }
        );
    }
    println!("\n{dirty_raw}/{total} raw bus layouts have a fragmented pole network");
    println!("{dirty_compact}/{total} compacted layouts do");

    // Cell composition is the other producer of layouts, and the one whose
    // compacted mil5 control carries 2 disconnected poles.
    println!("\n--- cell-composed fixtures ---");
    for label in ["chain-mil5ore", "mega-chain-chem5raw", "mega-chain-pu4raw", "mega-chain-usp2raw"] {
        let fixture = SimFixture::find(label);
        let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve_with_palette_exclusions_and_quality(
            fixture.target, fixture.rate, &inputs, &MachinePalette::default(),
            "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
        ) else { continue };
        let base = fixture.compose_layout();
        let compact = compact_validated_geometry(&base, &sr);
        // `disconnected_poles` counts "unreachable from pole[0]", which is a
        // poor progress metric: merging two components that both exclude
        // pole[0] leaves the count unchanged while ADDING the bridge, so real
        // repair can read as regression. Component count is the honest one.
        let components = |l: &spaghettio_core::models::LayoutResult| -> usize {
            let poles: Vec<(f64, f64, f64)> = l
                .entities
                .iter()
                .filter_map(|e| {
                    spaghettio_core::power_wires::wire_reach(
                        &e.name,
                        e.quality.unwrap_or_default(),
                    )
                    .map(|r| {
                        let (cx, cy) = spaghettio_core::power_wires::pole_center(&e.name, e.x, e.y);
                        (cx, cy, r)
                    })
                })
                .collect();
            let n = poles.len();
            let mut parent: Vec<usize> = (0..n).collect();
            fn find(p: &mut [usize], mut x: usize) -> usize {
                while p[x] != x { p[x] = p[p[x]]; x = p[x]; }
                x
            }
            for i in 0..n {
                for j in (i + 1)..n {
                    let (ax, ay, ar) = poles[i];
                    let (bx, by, br) = poles[j];
                    let (dx, dy) = (ax - bx, ay - by);
                    let reach = ar.min(br);
                    if dx * dx + dy * dy <= reach * reach {
                        let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                        if ri != rj { parent[ri] = rj; }
                    }
                }
            }
            let mut roots = std::collections::BTreeSet::new();
            for i in 0..n { let r = find(&mut parent, i); roots.insert(r); }
            roots.len()
        };
        let count = |l: &spaghettio_core::models::LayoutResult| {
            let n = l.entities.iter()
                .filter(|e| e.name.ends_with("electric-pole") || e.name == "substation")
                .count();
            (n, disconnected_poles(&l.entities, &wires_for(l)).len())
        };
        let (bn, bd) = count(&base);
        let (cn, cd) = count(&compact);
        // Can the pipeline's own repair close the gap on composed output?
        let mut repaired = base.clone();
        let added = spaghettio_core::bus::layout::repair_pole_network(&mut repaired);
        let (rn, rd) = count(&repaired);
        println!(
            "{label:<22} composed {bd:>4}/{bn:<4} in {:>4} networks   \
             after repair {rd:>4}/{rn:<4} in {:>4} networks (+{added} bridges)",
            components(&base),
            components(&repaired),
        );
        let _ = (cd, cn);
    }
}

/// Does the fold work on BUS layouts, not just cell chains?
///
/// Every fold measurement so far has been on `compose_chain_*` output — the
/// cell architecture. The only wired-up consumer of this module,
/// `LayoutOptions::compact_layout`, sits on the bus path, so bus layouts are
/// the case with a real caller and no evidence.
#[test]
#[ignore = "exploration probe — fold search over bus layouts"]
fn probe_fold_bus_layouts() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, search_snake_fold};
    use spaghettio_core::validate::{self, LayoutStyle};
    use std::collections::BTreeMap;

    let cases: &[(&str, &str, f64, &[&str], &str)] = &[
        ("gear15-ore", "iron-gear-wheel", 15.0, &["iron-ore"], "assembling-machine-2"),
        ("ec10-ore", "electronic-circuit", 10.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("ec15-plate", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"], "assembling-machine-2"),
        ("belt5-ore", "transport-belt", 5.0, &["iron-ore"], "assembling-machine-2"),
        ("insert3-ore", "inserter", 3.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
    ];

    for (label, item, rate, inputs, machine) in cases {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        ) else {
            println!("{label}: solver refused");
            continue;
        };
        let Ok(bus) = layout::build_bus_layout(&sr, layout::LayoutOptions::default()) else {
            println!("{label}: bus layout refused");
            continue;
        };
        let compact = compact_validated_geometry(&bus, &sr);
        let asp = compact.width.max(compact.height) as f64 / compact.width.min(compact.height) as f64;

        let search = search_snake_fold(&compact, &sr, 3);
        let mut why: BTreeMap<String, usize> = BTreeMap::new();
        for (_, r) in &search.refusals {
            *why.entry(format!("{r:?}").split(' ').next().unwrap().into()).or_default() += 1;
        }
        match &search.best {
            Some(found) => {
                let l = &found.layout;
                let fasp = l.width.max(l.height) as f64 / l.width.min(l.height) as f64;
                println!(
                    "{label}: bus {}x{} ({asp:.1}:1, {} ent) -> folds={:?} {}x{} ({fasp:.2}:1, {} ent)",
                    compact.width, compact.height, compact.entities.len(),
                    found.folds, l.width, l.height, l.entities.len(),
                );
            }
            None => {
                println!(
                    "{label}: bus {}x{} ({asp:.1}:1) -> NO fold | legal={} refusals={why:?} rejected={}",
                    compact.width, compact.height, search.legal_columns,
                    search.rejected_by_validation
                );
                // A candidate that folds but validates worse is a different
                // animal from one that refuses; show what it broke.
                if search.rejected_by_validation > 0 {
                    let legal = spaghettio_core::bus::compaction::legal_fold_columns(&compact);
                    let mid = legal
                        .iter()
                        .copied()
                        .min_by_key(|f| (f - compact.width / 2).abs());
                    if let Some(f) = mid {
                        if let Ok(folded) =
                            spaghettio_core::bus::compaction::fold_snake(&compact, &[f])
                        {
                            let prof = |l: &spaghettio_core::models::LayoutResult| {
                                let mut m: BTreeMap<String, usize> = BTreeMap::new();
                                if let Ok(v) = validate::validate(l, Some(&sr), LayoutStyle::Bus) {
                                    for i in &v {
                                        *m.entry(i.category.clone()).or_default() += 1;
                                    }
                                }
                                m
                            };
                            let (a, b) = (prof(&compact), prof(&folded));
                            println!("     control={a:?}");
                            println!("     fold@{f}={b:?}");
                        }
                    }
                }
            }
        }
    }
}

/// Does the fold generalise beyond the fixture it was developed on?
/// Runs the validated search over the whole RFC-057 corpus and reports what
/// each fixture yields, plus why candidates were refused.
#[test]
#[ignore = "exploration probe — fold search across the corpus"]
fn probe_fold_corpus() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, search_snake_fold};
    use std::collections::BTreeMap;

    for label in [
        "chain-mil5ore",
        "mega-chain-chem5raw",
        "mega-chain-pu4raw",
        "mega-chain-usp2raw",
    ] {
        let fixture = SimFixture::find(label);
        let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            fixture.target,
            fixture.rate,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap();
        let compact = compact_validated_geometry(&fixture.compose_layout(), &sr);
        let src_aspect = compact.width as f64 / compact.height as f64;

        let search = search_snake_fold(&compact, &sr, 4);
        let mut why: BTreeMap<String, usize> = BTreeMap::new();
        for (_, r) in &search.refusals {
            *why.entry(format!("{r:?}").split(' ').next().unwrap().into())
                .or_default() += 1;
        }
        match &search.best {
            Some(found) => {
                let l = &found.layout;
                let asp = l.width.max(l.height) as f64 / l.width.min(l.height) as f64;
                println!(
                    "{label}: {}x{} ({:.1}:1, {} ent) -> folds={:?} {}x{} ({:.2}:1, {} ent)",
                    compact.width,
                    compact.height,
                    src_aspect,
                    compact.entities.len(),
                    found.folds,
                    l.width,
                    l.height,
                    asp,
                    l.entities.len(),
                );
                println!(
                    "   legal columns={}, refusals={why:?}, rejected-by-validation={}",
                    search.legal_columns, search.rejected_by_validation
                );
            }
            None => {
                println!(
                    "{label}: {}x{} ({:.1}:1) -> NO function-preserving fold",
                    compact.width, compact.height, src_aspect
                );
                println!(
                    "   legal columns={} of {}, refusals={why:?}, \
                     rejected-by-validation={}",
                    search.legal_columns,
                    compact.width - 1,
                    search.rejected_by_validation
                );
            }
        }
    }
}

/// What is actually blocking a fold? Fold `chain-mil5ore` at its midpoint
/// and group every validation issue by category, rather than sampling the
/// first few as the fold search does.
#[test]
#[ignore = "exploration probe — fold error breakdown"]
fn probe_fold_error_breakdown_mil5() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, fold_snake};
    use spaghettio_core::validate::{self, LayoutStyle};
    use std::collections::BTreeMap;

    let fixture = SimFixture::find("chain-mil5ore");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let compact = compact_validated_geometry(&fixture.compose_layout(), &sr);
    println!(
        "compact baseline: {}x{}, {} entities",
        compact.width,
        compact.height,
        compact.entities.len()
    );

    {
        println!("  --- boundaries: source vs folded ---");
        for b in &compact.boundary_inputs {
            println!("      IN  src {} ({},{}) dir={:?}", b.item, b.x, b.y, b.direction);
        }
        for b in &compact.boundary_outputs {
            println!("      OUT src {} ({},{}) dir={:?}", b.item, b.x, b.y, b.direction);
        }
        if let Ok(f) = spaghettio_core::bus::compaction::fold_snake(&compact, &[compact.width / 2]) {
            for b in &f.boundary_inputs {
                println!("      IN  fold {} ({},{}) dir={:?}", b.item, b.x, b.y, b.direction);
            }
            for b in &f.boundary_outputs {
                println!("      OUT fold {} ({},{}) dir={:?}", b.item, b.x, b.y, b.direction);
            }
        } else {
            println!("      (fold currently refused; boundaries unavailable)");
        }
        println!("  --- source around (489,8) ---");
        let mut near: Vec<_> = compact
            .entities
            .iter()
            .filter(|e| (e.x - 489).abs() <= 5 && (e.y - 8).abs() <= 2)
            .collect();
        near.sort_by_key(|e| (e.y, e.x));
        for e in near {
            println!(
                "      ({},{}) {} dir={:?} io={:?} carries={:?}",
                e.x, e.y, e.name, e.direction, e.io_type, e.carries
            );
        }
    }

    // Control: whatever the compacted source already warns about is not
    // something folding introduced.
    let base = validate::validate(&compact, Some(&sr), LayoutStyle::Bus).unwrap();
    let mut base_cat: BTreeMap<String, usize> = BTreeMap::new();
    for i in &base {
        *base_cat.entry(i.category.clone()).or_default() += 1;
    }
    println!("  unfolded control: {} issues {:?}", base.len(), base_cat);
    for i in &base {
        println!("      control: [{}] {}", i.category, i.message);
    }

    println!(
        "legal fold columns: {} of {}",
        spaghettio_core::bus::compaction::legal_fold_columns(&compact).len(),
        compact.width - 1
    );

    // The validated search: only folds that match the control's issue
    // profile are admitted.
    let search5 = spaghettio_core::bus::compaction::search_snake_fold(&compact, &sr, 5);
    match &search5.best {
        Some(found) => {
            let l = &found.layout;
            println!(
                "\nSEARCH WINNER: folds={:?} -> {}x{} (aspect {:.2}), {} entities                  [source {}x{} aspect {:.2}, {} entities]",
                found.folds,
                l.width,
                l.height,
                l.width.max(l.height) as f64 / l.width.min(l.height) as f64,
                l.entities.len(),
                compact.width,
                compact.height,
                compact.width as f64 / compact.height as f64,
                compact.entities.len(),
            );
            let mut why: BTreeMap<String, usize> = BTreeMap::new();
            for (_, reason) in &search5.refusals {
                *why.entry(format!("{reason:?}").split(' ').next().unwrap().to_string())
                    .or_default() += 1;
            }
            println!("  refusals during search: {why:?}");
        }
        None => println!("\nSEARCH: no fold preserved the control issue profile"),
    }

    let ladder: Vec<Vec<i32>> = (1..=5)
        .filter_map(|k| spaghettio_core::bus::compaction::even_legal_folds(&compact, k, 30))
        .collect();
    for folds in ladder {
        println!("\n=== folds={folds:?} ===");
        let folded = match fold_snake(&compact, &folds) {
            Ok(f) => f,
            Err(reason) => {
                println!("  fold_snake refused: {reason:?}");
                continue;
            }
        };
        println!(
            "  folded: {}x{}, {} entities",
            folded.width,
            folded.height,
            folded.entities.len()
        );
        let issues = match validate::validate(&folded, Some(&sr), LayoutStyle::Bus) {
            Ok(v) => v,
            Err(e) => {
                // A hard refusal carries its findings in the message body.
                let msg = format!("{e}");
                let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
                for line in msg.lines().skip(1) {
                    let kind = if line.contains("has no receiver") {
                        "dead-end belt"
                    } else if line.contains("but feeds into") {
                        "item mismatch on feed"
                    } else if line.contains("overlap") {
                        "entity overlap"
                    } else {
                        "other"
                    };
                    *by_kind.entry(kind.to_string()).or_default() += 1;
                }
                println!("  REFUSED, {} findings:", msg.lines().count() - 1);
                for (kind, n) in &by_kind {
                    println!("    {n:>5}  {kind}");
                }
                for line in msg.lines().skip(1).take(4) {
                    println!("    e.g. {}", line.trim());
                }
                // Dump the neighbourhood of the first reported coordinate so
                // the cause is visible instead of inferred.
                if let Some(coord) = msg
                    .lines()
                    .skip(1)
                    .find_map(|l| l.split(" at (").nth(1))
                    .and_then(|rest| rest.split(')').next())
                {
                    let nums: Vec<i32> = coord
                        .split(',')
                        .filter_map(|n| n.trim().parse().ok())
                        .collect();
                    if let [ex, ey] = nums[..] {
                        println!("    --- around ({ex},{ey}) ---");
                        let mut near: Vec<_> = folded
                            .entities
                            .iter()
                            .filter(|e| (e.x - ex).abs() <= 3 && (e.y - ey).abs() <= 3)
                            .collect();
                        near.sort_by_key(|e| (e.y, e.x));
                        for e in near {
                            println!(
                                "      ({},{}) {} dir={:?} io={:?} carries={:?}",
                                e.x, e.y, e.name, e.direction, e.io_type, e.carries
                            );
                        }
                    }
                }
                continue;
            }
        };
        let mut by_cat: BTreeMap<(String, String), usize> = BTreeMap::new();
        for i in &issues {
            *by_cat
                .entry((format!("{:?}", i.severity), i.category.clone()))
                .or_default() += 1;
        }
        println!("  {} issues:", issues.len());
        for ((sev, cat), n) in &by_cat {
            println!("    {n:>5}  [{sev}] {cat}");
        }
        if let Some(w) = issues
            .iter()
            .find(|i| i.category == "belt-flow-reachability")
        {
            if let Some(coord) = w.message.split(" at (").nth(1).and_then(|r| r.split(')').next()) {
                let nums: Vec<i32> = coord
                    .split(',')
                    .filter_map(|n| n.trim().parse().ok())
                    .collect();
                if let [ex, ey] = nums[..] {
                    println!("    --- feed side of ({ex},{ey}) ---");
                    let mut near: Vec<_> = folded
                        .entities
                        .iter()
                        .filter(|e| (e.x - ex).abs() <= 4 && (e.y - ey).abs() <= 4)
                        .collect();
                    near.sort_by_key(|e| (e.y, e.x));
                    for e in near {
                        println!(
                            "      ({},{}) {} dir={:?} io={:?} carries={:?}",
                            e.x, e.y, e.name, e.direction, e.io_type, e.carries
                        );
                    }
                }
            }
        }
        let mut shown: BTreeMap<String, usize> = BTreeMap::new();
        for i in &issues {
            let n = shown.entry(i.category.clone()).or_default();
            if *n < 2 {
                println!("    e.g. [{}] {}", i.category, i.message);
            }
            *n += 1;
        }
    }
}

/// Classify what actually kills a multi-fold candidate at the input pass.
///
/// #492 recorded the blocker as "two items share an input column, needs a B12
/// dive". Measuring says that is the SMALLEST of three classes. A gap takes
/// inputs from BOTH neighbouring segments, but lane rows are allocated purely
/// by column, so climbs from opposite sides sweep through each other's rows
/// with no shared column involved.
///
/// Read the classes off stderr with `SPAGHETTIO_FOLD_DEBUG=1`:
///   CROSS-SIDE     — row assignment ignores source side. No underground needed.
///   SAME-SIDE-TIE  — two same-side items on one column. Needs the dive.
///   PRE-EXISTING   — a lane run hit real machine/belt geometry.
///
/// Scoped to mil5 and 2 folds deliberately: the whole corpus at 4 folds takes
/// >10 minutes with debug on, and this is meant to be a fast feedback loop.
/// Counts are of FIRST clashes — a candidate dies at its first one, so this
/// does not measure latent conflicts hiding behind the one that fired.
#[test]
#[ignore = "exploration probe — input-clash classes (#492)"]
fn probe_input_clash_classes() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, search_snake_fold};
    use std::collections::BTreeMap;

    let fixture = SimFixture::find("chain-mil5ore");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let compact = compact_validated_geometry(&fixture.compose_layout(), &sr);

    let search = search_snake_fold(&compact, &sr, 2);
    let mut why: BTreeMap<String, usize> = BTreeMap::new();
    for (_, r) in &search.refusals {
        *why.entry(format!("{r:?}").split(' ').next().unwrap().into())
            .or_default() += 1;
    }
    println!(
        "mil5 @<=2 folds: legal_columns={} refusals={why:?} rejected_by_validation={} best={:?}",
        search.legal_columns,
        search.rejected_by_validation,
        search.best.as_ref().map(|f| f.folds.clone()),
    );
    // The bare rejection count cannot say which check the newly-buildable
    // layouts fail, which is the whole question once a geometry fix converts
    // refusals into validation rejections.
    println!("   validation regressions by category: {:?}", search.validation_regressions);
}

/// Take ONE 2-fold candidate that now builds and say exactly what it violates.
///
/// The side-partition fix (#492) took `GapLaneConflict` from 32 to 0 on mil5
/// while `rejected_by_validation` went 22 -> 54: the same candidates, failing
/// later. The aggregate categories are power / belt-flow-reachability /
/// orphan-belt-segment; this prints positions so the cause is a location, not
/// a category name.
#[test]
#[ignore = "exploration probe — one failing 2-fold candidate (#492)"]
fn probe_one_multifold_candidate() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, fold_snake};
    use spaghettio_core::validate::{self, LayoutStyle};
    use std::collections::BTreeMap;

    let fixture = SimFixture::find("chain-mil5ore");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let compact = compact_validated_geometry(&fixture.compose_layout(), &sr);

    let profile = |l: &spaghettio_core::models::LayoutResult| {
        let issues = match validate::validate(l, Some(&sr), LayoutStyle::Bus) {
            Ok(i) => i,
            Err(e) => e.issues,
        };
        let mut by: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for i in &issues {
            by.entry(i.category.clone())
                .or_default()
                .push(i.message.clone());
        }
        by
    };

    let base = profile(&compact);
    println!("source {}x{}", compact.width, compact.height);
    for (cat, msgs) in &base {
        println!("  base {cat}: {}", msgs.len());
    }

    // The comb the search would try for k=2 at delta=0.
    let legal = spaghettio_core::bus::compaction::legal_fold_columns(&compact);
    let snap = |t: i32| legal.iter().copied().min_by_key(|&f| (f - t).abs()).unwrap();
    let folds = vec![snap(compact.width / 3), snap(compact.width * 2 / 3)];
    println!("folds={folds:?}");

    match fold_snake(&compact, &folds) {
        Err(r) => println!("REFUSED: {r:?}"),
        Ok(folded) => {
            println!("folded {}x{}", folded.width, folded.height);
            // Are the gap-1 lanes actually fed? Print the gap rows and the
            // boundary-input records that should terminate them.
            println!("  --- boundary_inputs ({}) ---", folded.boundary_inputs.len());
            for b in &folded.boundary_inputs {
                println!("      in {:?} at ({},{}) dir={:?}", b.item, b.x, b.y, b.direction);
            }
            for row in 66..=72 {
                let mut row_ents: Vec<String> = folded
                    .entities
                    .iter()
                    .filter(|e| e.y == row)
                    .map(|e| format!("{}@{}({:?},{:?})", e.name.replace("transport-belt", "TB"), e.x, e.direction, e.carries))
                    .collect();
                row_ents.sort();
                println!("  row {row}: {} ents: {}", row_ents.len(),
                    row_ents.iter().take(8).cloned().collect::<Vec<_>>().join(" "));
            }
            let got = profile(&folded);
            for (cat, msgs) in &got {
                let b = base.get(cat).map(|v| v.len()).unwrap_or(0);
                let marker = if msgs.len() > b { " <-- REGRESSED" } else { "" };
                println!("  {cat}: {} (base {b}){marker}", msgs.len());
                if msgs.len() > b {
                    for m in msgs.iter().take(6) {
                        println!("      {m}");
                    }
                }
            }
        }
    }
}

/// mil5 must keep folding, stay near-square, and keep BOTH belt lanes.
///
/// One test rather than three because the expensive part — solve, compose,
/// compact, search — is shared; splitting it triples a 25-second cost.
///
/// The fold columns are DERIVED, not pinned. They were pinned initially, and
/// #502/#503 (concurrent work fixing undergroundification, which changed the
/// compacted source geometry) immediately falsified them: mil5 moved from
/// folding at [172,346] to [189,373] with legal columns 220 -> 252. The fold
/// itself was fine. A pin here tests "did upstream compaction change" — which
/// is not this test's job and produces a failure that reads as a fold
/// regression.
///
/// Three properties, and the third is the one a throughput sim cannot see:
///
/// 1. **A multi-fold still exists.** The transform's whole claim.
/// 2. **No validation category is worse than the source.** Deliberately a
///    comparison rather than a geometry assertion, because the validator
///    demonstrably catches this transform's real failures — the lane terminus
///    that never turned surfaced as 45 `belt-flow-reachability` + 4
///    `orphan-belt-segment`, the pole seam holes as 5 `power`.
/// 3. **Both belt lanes survive the gap corners.** A 90-degree corner preserves
///    both lanes (factorio-mechanics B11); a sideload dumps everything onto the
///    lane nearest the feeder (B8). `lane_transfer` models both exactly, so this
///    is statically decidable — and INVISIBLE to a sim, since mil5 needs 5/s of
///    science and a halved lane still delivers that.
///
/// The teeth assertion on (3) is load-bearing: "no lane-throughput errors" is
/// vacuous if every lane sits far below capacity, so the busiest lane must
/// exceed HALF the per-lane cap for a collapse to be detectable at all. mil5
/// supplies stone at 50/s over two boundaries, which is what qualifies it.
///
/// What none of this covers: anything the validator is blind to. RFC-057 carries
/// the case where this transform validated at exact control parity and produced
/// 0.00/s in Factorio, which is why the sim measurement lives in the decision
/// log rather than being replaced by a test.
#[test]
fn mil5_multifold_holds_and_preserves_lanes() {
    use spaghettio_core::bus::compaction::{compact_validated_geometry, search_snake_fold};
    use spaghettio_core::common::lane_capacity_stacked;
    use spaghettio_core::validate::belt_flow::{check_lane_throughput, compute_lane_rates};
    use spaghettio_core::validate::{self, LayoutStyle};
    use std::collections::BTreeMap;

    let fixture = SimFixture::find("chain-mil5ore");
    let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        fixture.target,
        fixture.rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let compact = compact_validated_geometry(&fixture.compose_layout(), &sr);

    let search = search_snake_fold(&compact, &sr, 4);
    let found = search.best.as_ref().unwrap_or_else(|| {
        let mut why: BTreeMap<String, usize> = BTreeMap::new();
        for (_, r) in &search.refusals {
            *why.entry(format!("{r:?}").split(' ').next().unwrap().into()).or_default() += 1;
        }
        panic!(
            "mil5 no longer folds at all: legal_columns={} refusals={why:?} \
             rejected_by_validation={} regressions={:?}",
            search.legal_columns, search.rejected_by_validation, search.validation_regressions
        )
    });
    let folded = &found.layout;

    assert!(
        found.folds.len() >= 2,
        "mil5 should reach a MULTI-fold, got {} fold(s) at {:?} -> {}x{}",
        found.folds.len(),
        found.folds,
        folded.width,
        folded.height,
    );

    let aspect =
        folded.width.max(folded.height) as f64 / folded.width.min(folded.height) as f64;
    assert!(
        aspect < 2.5,
        "fold should be much squarer than the {:.1}:1 source, got {}x{} ({aspect:.2}:1) at {:?}",
        compact.width as f64 / compact.height as f64,
        folded.width,
        folded.height,
        found.folds,
    );

    let profile = |l: &spaghettio_core::models::LayoutResult| -> BTreeMap<String, usize> {
        let issues = match validate::validate(l, Some(&sr), LayoutStyle::Bus) {
            Ok(i) => i,
            Err(e) => e.issues,
        };
        let mut by: BTreeMap<String, usize> = BTreeMap::new();
        for i in &issues {
            *by.entry(i.category.clone()).or_default() += 1;
        }
        by
    };
    let (base, got) = (profile(&compact), profile(folded));
    let regressed: Vec<String> = got
        .iter()
        .filter(|(cat, n)| base.get(*cat).copied().unwrap_or(0) < **n)
        .map(|(cat, n)| format!("{cat}: {} -> {n}", base.get(cat).copied().unwrap_or(0)))
        .collect();
    assert!(
        regressed.is_empty(),
        "fold regressed validation vs its source: {regressed:?}\nbase={base:?}\ngot={got:?}"
    );

    // Lane preservation, with the detectability check first.
    let cap = lane_capacity_stacked("express-transport-belt", 1);
    let peak = compute_lane_rates(folded, Some(&sr))
        .values()
        .map(|&[a, b]| a.max(b))
        .fold(0.0f64, f64::max);
    assert!(
        peak * 2.0 > cap,
        "lane check is vacuous here: busiest lane {peak:.2}/s doubles to {:.2}/s, under the \
         {cap}/s per-lane cap, so a sideload could not be detected",
        peak * 2.0
    );
    let lane_issues = check_lane_throughput(folded, Some(&sr));
    assert!(
        lane_issues.is_empty(),
        "fold collapsed a belt onto one lane ({} errors): {:?}",
        lane_issues.len(),
        lane_issues.iter().take(4).map(|i| &i.message).collect::<Vec<_>>()
    );
    assert!(
        check_lane_throughput(&compact, Some(&sr)).is_empty(),
        "control already has lane-throughput errors; the fold result is not attributable"
    );
}

/// Island placement must keep TERMINAL tiles disjoint, not just machine blocks.
///
/// A terminal is the belt tile an inserter reaches to, so it sits OUTSIDE the
/// island's block. Packing on blocks alone let one island's input terminal
/// land exactly on another island's output terminal — one tile asked to be the
/// delivery point for two different commodities, which is physically
/// impossible and which no router can repair, because the conflict is created
/// before routing starts.
///
/// This asserts the invariant by COUNT and reports every offending tile, not a
/// sample: the failure it guards was previously visible only as an aggregate
/// "N routes still contain surface conflicts", which said nothing about cause.
/// Cross-item shared terminals exactly equalled that unresolved-route count on
/// all six bus fixtures measured (2, 2, 44, 4, 8, 7).
#[test]
fn rfc057_island_placement_keeps_terminals_disjoint() {
    use spaghettio_core::bus::compaction::{
        build_manifold_nets, materialize_legalized_manifold_routes, place_recipe_clusters,
        CompactIr,
    };
    use std::collections::BTreeMap;

    let cases: &[(&str, &str, f64, &[&str], &str)] = &[
        ("gear5-plate", "iron-gear-wheel", 5.0, &["iron-plate"], "assembling-machine-1"),
        ("sci1-ore", "automation-science-pack", 1.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("ec15-plate", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"], "assembling-machine-2"),
        ("belt5-ore", "transport-belt", 5.0, &["iron-ore"], "assembling-machine-2"),
    ];

    let mut materialized = 0usize;
    for (label, item, rate, inputs, machine) in cases {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap_or_else(|e| panic!("{label}: solver refused: {e}"));
        let bus = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
            .unwrap_or_else(|e| panic!("{label}: layout refused: {e}"));
        let src = spaghettio_core::bus::compaction::compact_validated_geometry(&bus, &sr);

        let ir = CompactIr::from_source(&src, &sr);
        let (clustered, _) = place_recipe_clusters(&ir, 1);
        let manifolds = build_manifold_nets(&ir, &clustered)
            .unwrap_or_else(|e| panic!("{label}: manifold nets refused: {e}"));

        // One entry per tile, so a failure names every collision.
        let mut by_tile: BTreeMap<(i32, i32), Vec<&str>> = BTreeMap::new();
        for net in &manifolds {
            for terminal in &net.terminals {
                by_tile
                    .entry((terminal.x, terminal.y))
                    .or_default()
                    .push(net.item.as_str());
            }
        }
        let cross_item: Vec<_> = by_tile
            .iter()
            .filter(|(_, items)| items.iter().any(|i| *i != items[0]))
            .collect();
        assert!(
            cross_item.is_empty(),
            "{label}: {} tile(s) host terminals for more than one commodity: {cross_item:?}",
            cross_item.len(),
        );

        let inside: Vec<_> = manifolds
            .iter()
            .flat_map(|net| net.terminals.iter().map(move |t| (net.item.as_str(), t)))
            .filter(|(_, t)| {
                clustered.iter().enumerate().any(|(idx, island)| {
                    Some(idx) != t.island_id
                        && t.x >= island.block.x
                        && t.x < island.block.x + island.block.width
                        && t.y >= island.block.y
                        && t.y < island.block.y + island.block.height
                })
            })
            .map(|(item, t)| (item, t.x, t.y))
            .collect();
        assert!(
            inside.is_empty(),
            "{label}: {} terminal(s) land inside another island's footprint: {inside:?}",
            inside.len(),
        );

        // The point of the invariant: routing can now actually complete.
        let plans = spaghettio_core::bus::compaction::plan_local_manifolds(&clustered, &manifolds, 1);
        let graphs: Vec<_> = plans
            .iter()
            .map(spaghettio_core::bus::compaction::build_local_manifold_graph)
            .collect();
        let hubs = spaghettio_core::bus::compaction::place_distributed_local_manifold_nodes(
            &clustered, &graphs, 3,
        )
        .unwrap_or_else(|e| panic!("{label}: hub placement refused: {e}"));
        let routed =
            spaghettio_core::bus::compaction::route_local_manifold_edges(&clustered, &graphs, &hubs)
                .unwrap_or_else(|e| panic!("{label}: routing refused: {e}"));
        // An edge that fails all three routing passes lands in `unroutable`
        // and NEVER enters `routes` — so the legalization and materialization
        // gates below cannot see it. Without these two assertions they hold
        // trivially for every dropped edge, and vacuously in the limit where
        // nothing routes at all. Exactly the "a check going quiet is not
        // evidence" failure in CLAUDE.md's verification protocol.
        let edge_count: usize = graphs.iter().map(|graph| graph.edges.len()).sum();
        // …and `routes.len() == edge_count` is itself vacuous at zero, so the
        // corpus has to actually pose the problem.
        assert!(
            edge_count > 0,
            "{label}: no manifold edges to route — this fixture proves nothing",
        );
        assert!(
            routed.unroutable.is_empty(),
            "{label}: {} of {edge_count} manifold edge(s) could not be routed at all: {:?}",
            routed.unroutable.len(),
            routed.unroutable,
        );
        assert_eq!(
            routed.routes.len(),
            edge_count,
            "{label}: {} routes emitted for {edge_count} graph edges — every edge \
             must be accounted for before the gates below mean anything",
            routed.routes.len(),
        );
        let legalized =
            spaghettio_core::bus::compaction::legalize_manifold_routes(&routed.routes);
        assert_eq!(
            legalized.unresolved_routes, 0,
            "{label}: {} routes still conflict after terminal-disjoint placement \
             ({} tiles) — the placement invariant holds, so this is a genuine \
             routing failure and wants its own diagnosis",
            legalized.unresolved_routes, legalized.unresolved_tiles,
        );
        assert!(materialize_legalized_manifold_routes(&legalized.routes).is_ok());
        materialized += 1;
    }
    assert_eq!(materialized, cases.len());
}

// ---------------------------------------------------------------------------
// RFC-058 band helpers, shared by the phase-0 probes below.
//
// A band is a maximal run of rows containing machines or inserters. Trunk
// belts span all rows and are deliberately NOT band-forming — they are the
// transport being priced, not the structure being packed.
//
// `rfc058_band_packing_premise_holds` (the CI guard) intentionally does NOT
// use these helpers: it keeps its own self-contained copy so a defect
// introduced here cannot blind the guard that exists to catch drift.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Rfc058Band {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    recipes: Vec<String>,
}

impl Rfc058Band {
    fn cx(&self) -> f64 {
        self.x as f64 + self.w as f64 / 2.0
    }
    fn cy(&self) -> f64 {
        self.y as f64 + self.h as f64 / 2.0
    }
}

fn rfc058_extract_bands(l: &spaghettio_core::models::LayoutResult) -> Vec<Rfc058Band> {
    use spaghettio_core::common::{entity_size, is_machine_entity};
    let h = l.height.max(0) as usize;
    let mut structural = vec![false; h];
    for e in &l.entities {
        if is_machine_entity(&e.name) || e.name.contains("inserter") {
            let (_, eh) = entity_size(&e.name);
            for dy in 0..eh as i32 {
                let y = e.y + dy;
                if y >= 0 && (y as usize) < h {
                    structural[y as usize] = true;
                }
            }
        }
    }
    let mut bands = Vec::new();
    let mut y = 0usize;
    while y < h {
        if !structural[y] {
            y += 1;
            continue;
        }
        let start = y;
        while y < h && structural[y] {
            y += 1;
        }
        let end = y - 1;
        let (mut xmin, mut xmax) = (i32::MAX, i32::MIN);
        let mut recipes: FxHashSet<String> = FxHashSet::default();
        for e in &l.entities {
            if !(is_machine_entity(&e.name) || e.name.contains("inserter")) {
                continue;
            }
            if e.y < start as i32 || e.y > end as i32 {
                continue;
            }
            let (ew, _) = entity_size(&e.name);
            xmin = xmin.min(e.x);
            xmax = xmax.max(e.x + ew as i32 - 1);
            if let Some(r) = &e.recipe {
                recipes.insert(r.clone());
            }
        }
        if xmin > xmax {
            continue;
        }
        let mut recipes: Vec<String> = recipes.into_iter().collect();
        recipes.sort();
        bands.push(Rfc058Band {
            x: xmin,
            y: start as i32,
            w: xmax - xmin + 1,
            h: (end - start + 1) as i32,
            recipes,
        });
    }
    bands
}

fn rfc058_bbox(bands: &[Rfc058Band]) -> (i32, i32) {
    let w = bands.iter().map(|b| b.x + b.w).max().unwrap_or(0)
        - bands.iter().map(|b| b.x).min().unwrap_or(0);
    let h = bands.iter().map(|b| b.y + b.h).max().unwrap_or(0)
        - bands.iter().map(|b| b.y).min().unwrap_or(0);
    (w, h)
}

fn rfc058_shelf_pack(
    bands: &[Rfc058Band],
    target_w: i32,
    gap: i32,
    sort_desc: bool,
) -> Vec<Rfc058Band> {
    let mut idx: Vec<usize> = (0..bands.len()).collect();
    if sort_desc {
        idx.sort_by_key(|&i| std::cmp::Reverse((bands[i].h, bands[i].w, i)));
    }
    let mut out = bands.to_vec();
    let (mut x, mut y, mut shelf_h) = (0, 0, 0);
    for &i in &idx {
        let (w, h) = (bands[i].w, bands[i].h);
        if x > 0 && x + w > target_w {
            x = 0;
            y += shelf_h + gap;
            shelf_h = 0;
        }
        out[i].x = x;
        out[i].y = y;
        x += w + gap;
        shelf_h = shelf_h.max(h);
    }
    out
}

/// Best aspect-capped shelf packing: target width swept from the widest band
/// to twice the control width, both source and height-descending order, and
/// the minimum bounding-box area under `max_aspect` wins. Returns
/// `(area, w, h, packed)`, or None when no packing fits the cap — a
/// width-dominant band. Selection is by area alone with strict `<`, so the
/// first candidate at the minimum wins; iteration order is part of the
/// contract (the probe's published numbers depend on it).
fn rfc058_best_pack(
    bands: &[Rfc058Band],
    gap: i32,
    max_aspect: f64,
) -> Option<(i64, i32, i32, Vec<Rfc058Band>)> {
    let (cw, _) = rfc058_bbox(bands);
    let widest = bands.iter().map(|b| b.w).max().unwrap_or(1);
    let mut best: Option<(i64, i32, i32, Vec<Rfc058Band>)> = None;
    for sort_desc in [false, true] {
        let mut t = widest;
        while t <= cw.max(widest) * 2 {
            let packed = rfc058_shelf_pack(bands, t, gap, sort_desc);
            let (w, h) = rfc058_bbox(&packed);
            let aspect = w.max(h) as f64 / w.min(h).max(1) as f64;
            if aspect <= max_aspect {
                let area = (w as i64) * (h as i64);
                if best.as_ref().is_none_or(|(ba, _, _, _)| area < *ba) {
                    best = Some((area, w, h, packed));
                }
            }
            t += 2;
        }
    }
    best
}

/// RFC-058 Phase 0 instrument: band census + packing headroom.
///
/// This reproduces every number the RFC cites, so its premise is checkable by
/// anyone picking the work up rather than asserted from a probe that only ever
/// existed on one machine. It answers two things:
///
/// - **Kill criterion 2 (reach)** — how many fixtures have >=3 bands and no
///   width-dominant band. The RFC's 40-50% is over these ten hand-picked
///   fixtures, NOT the e2e corpus, and is explicitly not treated as settled.
/// - **The headroom estimate** — band-bbox and rate-weighted transport for the
///   best aspect-capped shelf packing against the as-placed control.
///
/// Both figures are obstacle-free and exclude trunk corridor space, so the
/// area saving is an upper bound. Closing that gap is what RFC-058's phase-3
/// trunk spike exists to do; nothing here can stand in for it.
///
/// `#[ignore]` because it builds ten layouts (~30s) and reports rather than
/// asserts — the same shape as `probe_fold_corpus`.
#[test]
#[ignore = "RFC-058 Phase 0 probe — run with --ignored --nocapture"]
fn probe_band_packing_headroom() {
    use spaghettio_core::models::SolverResult;

    // Rate-weighted transport: for every (band, consumed item) pair, Manhattan
    // distance to the nearest band producing it, times planned rate. External
    // inputs are priced from a spine at x=0.
    //
    // This is a PROXY and it has a known directional risk, recorded in the
    // RFC: real bus transport runs down a trunk then across to a row, which is
    // close to Manhattan for the stacked control but not necessarily for a
    // packed layout whose bands are no longer in one column. It may therefore
    // flatter packing. The area result does not depend on it.
    fn transport_cost(bands: &[Rfc058Band], sr: &SolverResult) -> f64 {
        let mut produced_by: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
        for spec in &sr.machines {
            for out in &spec.outputs {
                for (i, b) in bands.iter().enumerate() {
                    if b.recipes.iter().any(|r| r == &spec.recipe) {
                        produced_by.entry(out.item.as_str()).or_default().push(i);
                    }
                }
            }
        }
        let mut total = 0.0;
        for spec in &sr.machines {
            let consumers: Vec<usize> = bands
                .iter()
                .enumerate()
                .filter(|(_, b)| b.recipes.iter().any(|r| r == &spec.recipe))
                .map(|(i, _)| i)
                .collect();
            if consumers.is_empty() {
                continue;
            }
            for inp in &spec.inputs {
                let rate = inp.rate * spec.count;
                for &ci in &consumers {
                    let c = &bands[ci];
                    let d = match produced_by.get(inp.item.as_str()) {
                        Some(ps) if !ps.is_empty() => ps
                            .iter()
                            .map(|&pi| {
                                let p = &bands[pi];
                                (p.cx() - c.cx()).abs() + (p.cy() - c.cy()).abs()
                            })
                            .fold(f64::INFINITY, f64::min),
                        _ => c.cx(),
                    };
                    total += rate * d / consumers.len() as f64;
                }
            }
        }
        total
    }

    // Minimising bbox AREA alone drives every packing to a one-shelf ribbon
    // (all bands side by side). That is the shape this workstream exists to
    // remove — pu4-raw ships one at 2752x90 — so area is only a valid
    // objective under an aspect cap. 3:1 is the RFC's value; it is arbitrary
    // and consequential (insert3-ore packs at 3.16:1 and is refused), and
    // RFC-058 phase 2 owns sweeping it.
    const MAX_ASPECT: f64 = 3.0;
    const GAP: i32 = 2;

    let cases: &[(&str, &str, f64, &[&str], &str)] = &[
        ("gear15-ore", "iron-gear-wheel", 15.0, &["iron-ore"], "assembling-machine-2"),
        ("ec10-ore", "electronic-circuit", 10.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("ec15-plate", "electronic-circuit", 15.0, &["iron-plate", "copper-plate"], "assembling-machine-2"),
        ("belt5-ore", "transport-belt", 5.0, &["iron-ore"], "assembling-machine-2"),
        ("insert3-ore", "inserter", 3.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("gear5-plate", "iron-gear-wheel", 5.0, &["iron-plate"], "assembling-machine-1"),
        ("pu1-plate", "processing-unit", 1.0, &["iron-plate", "copper-plate", "sulfuric-acid"], "assembling-machine-2"),
        ("sci1-ore", "automation-science-pack", 1.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("lds2-plate", "low-density-structure", 2.0, &["iron-plate", "copper-plate", "plastic-bar"], "assembling-machine-2"),
    ];

    let (mut multi_band, mut packable) = (0usize, 0usize);
    let (mut corpus_ctrl, mut corpus_packed) = (0i64, 0i64);
    let (mut win_ctrl, mut win_packed) = (0i64, 0i64);
    let (mut win_tx_ctrl, mut win_tx_packed) = (0.0f64, 0.0f64);
    // Kill criterion 1's baseline is the aggregate over ONLY the three
    // fixtures that criterion names — not the four-fixture figure.
    let gate = ["sci1-ore", "sci2-ore", "pu1-plate"];
    let (mut gate_ctrl, mut gate_packed, mut gate_n) = (0i64, 0i64, 0usize);

    for (label, item, rate, inputs, machine) in cases {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        ) else {
            println!("{label:<13} solver refused");
            continue;
        };
        let Ok(l) = layout::build_bus_layout(&sr, layout::LayoutOptions::default()) else {
            println!("{label:<13} layout refused");
            continue;
        };
        let bands = rfc058_extract_bands(&l);
        let dims: Vec<(i32, i32)> = bands.iter().map(|b| (b.w, b.h)).collect();
        // Two gates, deliberately distinct.
        //
        // The CORPUS aggregate includes every fixture with >1 band, counting
        // unpackable ones at their control area — excluding them would bias
        // the headline toward winners, the exact defect review caught in the
        // RFC itself. That is what reproduces the RFC's −39.6%.
        //
        // KC2 REACH counts only fixtures with >=3 bands as candidates: two
        // bands admit at most two shelves, which is a stacking choice rather
        // than a packing problem, and counting them would inflate the figure
        // the criterion turns on.
        if bands.len() < 2 {
            println!("{label:<13} bands={} — nothing to pack {dims:?}", bands.len());
            continue;
        }
        let kc2_candidate = bands.len() >= 3;
        multi_band += 1;
        let (cw, ch) = rfc058_bbox(&bands);
        let ctrl_area = (cw as i64) * (ch as i64);
        let ctrl_tx = transport_cost(&bands, &sr);

        let best = rfc058_best_pack(&bands, GAP, MAX_ASPECT)
            .map(|(area, w, h, packed)| (area, transport_cost(&packed, &sr), w, h));

        corpus_ctrl += ctrl_area;
        match best {
            Some((barea, btx, bw, bh)) => {
                if kc2_candidate {
                    packable += 1;
                }
                corpus_packed += barea;
                win_ctrl += ctrl_area;
                win_packed += barea;
                win_tx_ctrl += ctrl_tx;
                win_tx_packed += btx;
                if gate.contains(label) {
                    gate_ctrl += ctrl_area;
                    gate_packed += barea;
                    gate_n += 1;
                }
                println!(
                    "{label:<13} bands={:<2} ctrl {cw}x{ch}={ctrl_area:<6} -> {bw}x{bh}={barea:<6} \
                     area {:+6.1}%  transport {:+7.1}%  {dims:?}",
                    bands.len(),
                    (barea as f64 - ctrl_area as f64) / ctrl_area as f64 * 100.0,
                    (btx - ctrl_tx) / ctrl_tx.max(1e-9) * 100.0,
                );
            }
            None => {
                // Unpackable fixtures stay at their control area in the corpus
                // aggregate — counting only the winners would be selection bias.
                corpus_packed += ctrl_area;
                println!(
                    "{label:<13} bands={:<2} ctrl {cw}x{ch}={ctrl_area:<6} -> no packing within \
                     {MAX_ASPECT}:1 (width-dominant band) {dims:?}",
                    bands.len(),
                );
            }
        }
    }

    let pct = |new: f64, old: f64| (new - old) / old * 100.0;
    println!("\n=== RFC-058 headroom ===");
    println!(
        "  corpus  ({multi_band} multi-band): band-bbox {corpus_ctrl} -> {corpus_packed} ({:+.1}%)",
        pct(corpus_packed as f64, corpus_ctrl as f64),
    );
    println!(
        "  winners ({packable} packable):    band-bbox {win_ctrl} -> {win_packed} ({:+.1}%), \
         transport {:.0} -> {:.0} ({:+.1}%)",
        pct(win_packed as f64, win_ctrl as f64),
        win_tx_ctrl,
        win_tx_packed,
        pct(win_tx_packed, win_tx_ctrl),
    );
    println!(
        "  KC1 baseline ({gate_n}/{} gate fixtures contributing): {gate_ctrl} -> {gate_packed} \
         ({:+.1}%), so the KC1 bar is half that: {:+.1}%{}",
        gate.len(),
        pct(gate_packed as f64, gate_ctrl as f64),
        pct(gate_packed as f64, gate_ctrl as f64) / 2.0,
        if gate_n == gate.len() { "" } else { "  <-- INCOMPLETE: a gate fixture no longer packs, so this baseline is not comparable to the RFC's" },
    );
    println!(
        "  KC2 reach: {packable}/{} fixtures packable ({:.0}%) — bar is 30%, but the \n\
         \x20            corpus that settles KC2 is the e2e corpus, not these ten.",
        cases.len(),
        packable as f64 / cases.len() as f64 * 100.0,
    );
}

/// RFC-058's premise must not evaporate silently.
///
/// `probe_band_packing_headroom` reports; this asserts. The RFC's design rests
/// on measured numbers, and a placer change could quietly invalidate them —
/// leaving an RFC that reads as evidence-backed while its evidence is stale.
/// That is the failure shape `docs/validator-reporting.md` records nine times:
/// a check going quiet is not the same as a problem being absent.
///
/// Bounds are deliberately loose. The point is to catch "the premise is gone",
/// not to pin exact geometry — pinning would break on every placer tweak and
/// get muted, which is worse than not asserting at all. Tight numbers live in
/// the probe's output and in the RFC's decision log.
#[test]
fn rfc058_band_packing_premise_holds() {
    use spaghettio_core::common::{entity_size, is_machine_entity};
    use spaghettio_core::models::LayoutResult;

    fn band_rects(l: &LayoutResult) -> Vec<(i32, i32, i32, i32)> {
        let h = l.height.max(0) as usize;
        let mut structural = vec![false; h];
        for e in &l.entities {
            if is_machine_entity(&e.name) || e.name.contains("inserter") {
                let (_, eh) = entity_size(&e.name);
                for dy in 0..eh as i32 {
                    let y = e.y + dy;
                    if y >= 0 && (y as usize) < h {
                        structural[y as usize] = true;
                    }
                }
            }
        }
        let mut out = Vec::new();
        let mut y = 0usize;
        while y < h {
            if !structural[y] {
                y += 1;
                continue;
            }
            let start = y;
            while y < h && structural[y] {
                y += 1;
            }
            let end = y - 1;
            let (mut xmin, mut xmax) = (i32::MAX, i32::MIN);
            for e in &l.entities {
                if !(is_machine_entity(&e.name) || e.name.contains("inserter")) {
                    continue;
                }
                if e.y < start as i32 || e.y > end as i32 {
                    continue;
                }
                let (ew, _) = entity_size(&e.name);
                xmin = xmin.min(e.x);
                xmax = xmax.max(e.x + ew as i32 - 1);
            }
            if xmin <= xmax {
                out.push((xmin, start as i32, xmax - xmin + 1, (end - start + 1) as i32));
            }
        }
        out
    }

    // The three fixtures kill criterion 1 names. Its baseline is the aggregate
    // over exactly these — not the four-fixture figure quoted in Motivation,
    // which includes the weakest packer and would set a more lenient bar.
    let cases: &[(&str, &str, f64, &[&str], &str)] = &[
        ("sci1-ore", "automation-science-pack", 1.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("pu1-plate", "processing-unit", 1.0, &["iron-plate", "copper-plate", "sulfuric-acid"], "assembling-machine-2"),
    ];

    let (mut ctrl_total, mut packed_total) = (0i64, 0i64);
    for (label, item, rate, inputs, machine) in cases {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap_or_else(|e| panic!("{label}: solver refused: {e}"));
        let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
            .unwrap_or_else(|e| panic!("{label}: layout refused: {e}"));
        let bands = band_rects(&l);
        assert!(
            bands.len() >= 3,
            "{label}: {} bands — RFC-058 needs >=3 to pack; if the placer now emits \
             fewer, the RFC's fixture choice is stale",
            bands.len(),
        );

        let cw = bands.iter().map(|b| b.0 + b.2).max().unwrap()
            - bands.iter().map(|b| b.0).min().unwrap();
        let ch = bands.iter().map(|b| b.1 + b.3).max().unwrap()
            - bands.iter().map(|b| b.1).min().unwrap();
        ctrl_total += (cw as i64) * (ch as i64);

        // Best aspect-capped shelf packing, same construction as the probe.
        let widest = bands.iter().map(|b| b.2).max().unwrap();
        let mut best = i64::MAX;
        for &sort_desc in &[false, true] {
            let mut order: Vec<usize> = (0..bands.len()).collect();
            if sort_desc {
                order.sort_by_key(|&i| std::cmp::Reverse((bands[i].3, bands[i].2, i)));
            }
            let mut t = widest;
            while t <= cw.max(widest) * 2 {
                let (mut x, mut y, mut shelf_h, mut mx, mut my) = (0, 0, 0, 0, 0);
                for &i in &order {
                    let (w, h) = (bands[i].2, bands[i].3);
                    if x > 0 && x + w > t {
                        x = 0;
                        y += shelf_h + 2;
                        shelf_h = 0;
                    }
                    x += w + 2;
                    shelf_h = shelf_h.max(h);
                    mx = mx.max(x - 2);
                    my = my.max(y + shelf_h);
                }
                let aspect = mx.max(my) as f64 / mx.min(my).max(1) as f64;
                if aspect <= 3.0 {
                    best = best.min((mx as i64) * (my as i64));
                }
                t += 2;
            }
        }
        assert!(best < i64::MAX, "{label}: no packing within 3:1 aspect");
        packed_total += best;
    }

    let saving = (ctrl_total - packed_total) as f64 / ctrl_total as f64 * 100.0;
    assert!(
        saving >= 50.0,
        "RFC-058's kill-criterion-1 baseline has drifted: band-bbox saving over the three \
         gate fixtures is {saving:.1}% ({ctrl_total} -> {packed_total}), against the {:.1}% \
         the RFC records. The criterion's -33.0% bar is half that baseline, so a materially \
         different number makes the gate wrong — update the RFC before relying on it.",
        66.1,
    );
}

/// RFC-058 Phase 0: band census over the e2e corpus — the reach measurement
/// kill criterion 2 turns on.
///
/// The corpus is every distinct production request (item, rate, machine,
/// belt, inputs, exclusions) exercised by a non-ignored test in
/// `crates/core/tests/e2e.rs`, transcribed 2026-07-31. Inclusion rule,
/// recorded so the denominator is arguable rather than mysterious:
/// strategy/row-layout variants are normalised to the default bus path,
/// duplicate tuples are listed once (`source` names one owning test), and
/// `#[ignore]`d tests are out — which excludes `measure_utility_10s_am3`,
/// `fixture_source_ec_15s_am1_yellow_from_ore`, the ignored stress trio
/// (`ac_partitioned_7s`, `ac_45s`, `pu_20s` — pu20 was an 85-band packable
/// winner, so this rule costs the numerator too) and, notably,
/// `pipe_belt_processing_unit_1s_routes` (pu1-plate): the KC1 gate fixture
/// is NOT part of KC2's denominator. Two non-ignored requests are also out
/// because this harness cannot express them —
/// `fulgora_scrap_sorter_mechanism_present` needs the recycling-aware
/// `solve_fulgora` path, and `cable13u_bridged_row_lane_throughput_clean`
/// solves at `QualityTier::Uncommon` — so "every distinct request" is
/// exact only over default-quality, non-recycling solves. (The Legendary
/// leg of `quality_differential_ec_normal_vs_legendary` is excluded the
/// same way; its Normal leg is the `ec4-plate-am3-red` row.)
///
/// KC2 asks what fraction of this corpus has >=3 bands and no
/// width-dominant band. The aspect cap is consequential (it alone refuses
/// insert3-ore at 3.16:1 in the probe corpus), so applicability is
/// reported as a function of the cap instead of inheriting 3:1.
///
/// Band structure is host-geometry-relative (RFC-058 decision log,
/// 2026-07-31: ec10-ore extracts 1 or 3 bands depending on the machine's
/// SAT-cache state). Treat these numbers like stress goldens — comparable
/// on one machine, not portable constants — and re-run on a second machine
/// before calling KC2 if the result lands within a few points of the 30%
/// bar.
#[test]
#[ignore = "RFC-058 Phase 0 census — run with --ignored --nocapture"]
fn probe_band_census_e2e_corpus() {
    struct Case {
        label: &'static str,
        source: &'static str,
        item: &'static str,
        rate: f64,
        machine: &'static str,
        belt: Option<&'static str>,
        inputs: &'static [&'static str],
        excluded: &'static [&'static str],
    }
    const A1: &str = "assembling-machine-1";
    const A2: &str = "assembling-machine-2";
    const A3: &str = "assembling-machine-3";
    const CHEM: &str = "chemical-plant";
    const REFINERY: &str = "oil-refinery";
    const YELLOW: Option<&'static str> = Some("transport-belt");
    const RED: Option<&'static str> = Some("fast-transport-belt");
    const ORES: &[&str] = &["iron-ore", "copper-ore"];
    const NAUVIS5: &[&str] = &["iron-plate", "copper-plate", "coal", "crude-oil", "water"];
    const ORES5: &[&str] = &["iron-ore", "copper-ore", "coal", "water", "crude-oil"];
    const PU9: &[&str] = &[
        "iron-plate", "copper-plate", "steel-plate", "stone", "coal",
        "water", "crude-oil", "iron-ore", "copper-ore",
    ];

    let cases: &[Case] = &[
        Case { label: "gear10-plate", source: "tier1_iron_gear_wheel", item: "iron-gear-wheel", rate: 10.0, machine: A1, belt: None, inputs: &["iron-plate"], excluded: &[] },
        Case { label: "gear10-ore", source: "tier1_iron_gear_wheel_from_ore", item: "iron-gear-wheel", rate: 10.0, machine: A2, belt: None, inputs: &["iron-ore"], excluded: &[] },
        Case { label: "gear20-plate", source: "tier1_iron_gear_wheel_20s", item: "iron-gear-wheel", rate: 20.0, machine: A2, belt: None, inputs: &["iron-plate"], excluded: &[] },
        Case { label: "ec10-plate", source: "tier2_electronic_circuit", item: "electronic-circuit", rate: 10.0, machine: A2, belt: None, inputs: &["iron-plate", "copper-plate"], excluded: &[] },
        Case { label: "ec10-plate-am1-red", source: "tier2_electronic_circuit_splitter_stamp_regression", item: "electronic-circuit", rate: 10.0, machine: A1, belt: RED, inputs: &["iron-plate", "copper-plate"], excluded: &[] },
        Case { label: "ec4-plate-am3-red", source: "quality_differential_ec_normal_vs_legendary (Normal leg)", item: "electronic-circuit", rate: 4.0, machine: A3, belt: RED, inputs: &["iron-plate", "copper-plate"], excluded: &[] },
        Case { label: "ec10-ore-yellow", source: "tier2_electronic_circuit_from_ore", item: "electronic-circuit", rate: 10.0, machine: A1, belt: YELLOW, inputs: ORES, excluded: &[] },
        Case { label: "ec20-ore", source: "tier2_electronic_circuit_20s_from_ore", item: "electronic-circuit", rate: 20.0, machine: A2, belt: None, inputs: ORES, excluded: &[] },
        Case { label: "plastic10-gas", source: "tier3_plastic_bar", item: "plastic-bar", rate: 10.0, machine: CHEM, belt: None, inputs: &["petroleum-gas", "coal"], excluded: &[] },
        Case { label: "plastic10-crude", source: "tier3_plastic_bar_from_crude", item: "plastic-bar", rate: 10.0, machine: CHEM, belt: None, inputs: &["crude-oil", "coal"], excluded: &[] },
        Case { label: "sulfuric5", source: "tier3_sulfuric_acid", item: "sulfuric-acid", rate: 5.0, machine: CHEM, belt: None, inputs: &["iron-plate", "sulfur", "water"], excluded: &[] },
        Case { label: "lightoil5-hoc", source: "tier3_heavy_oil_cracking", item: "light-oil", rate: 5.0, machine: CHEM, belt: None, inputs: &["water", "heavy-oil"], excluded: &["advanced-oil-processing", "coal-liquefaction"] },
        Case { label: "gas12-aop", source: "tier3_advanced_oil_processing_multi_machine", item: "petroleum-gas", rate: 12.0, machine: REFINERY, belt: None, inputs: &["water", "crude-oil"], excluded: &[] },
        Case { label: "gas24-aop", source: "tier3_advanced_oil_processing_forced_multi_machine_pipe_isolation", item: "petroleum-gas", rate: 24.0, machine: REFINERY, belt: None, inputs: &["water", "crude-oil"], excluded: &["basic-oil-processing", "coal-liquefaction"] },
        Case { label: "ac1-nauvis", source: "tier4_advanced_circuit_from_plates", item: "advanced-circuit", rate: 1.0, machine: A2, belt: None, inputs: NAUVIS5, excluded: &[] },
        Case { label: "ac4-nauvis", source: "stress_advanced_circuit_partitioned_4s_from_plates", item: "advanced-circuit", rate: 4.0, machine: A2, belt: None, inputs: NAUVIS5, excluded: &[] },
        Case { label: "ac5-nauvis", source: "stress_advanced_circuit_partitioned_5s_from_plates", item: "advanced-circuit", rate: 5.0, machine: A2, belt: None, inputs: NAUVIS5, excluded: &[] },
        Case { label: "ac7-nauvis-yellow", source: "tier4_advanced_circuit_7s_horizontal_stack_belt_pipe_crossing", item: "advanced-circuit", rate: 7.0, machine: A2, belt: YELLOW, inputs: NAUVIS5, excluded: &[] },
        Case { label: "ac5-ore-yellow", source: "tier4_advanced_circuit_from_ore_am2", item: "advanced-circuit", rate: 5.0, machine: A2, belt: YELLOW, inputs: ORES5, excluded: &[] },
        Case { label: "pu2-ore-red", source: "tier5_processing_unit_from_ore_am3", item: "processing-unit", rate: 2.0, machine: A3, belt: RED, inputs: ORES5, excluded: &[] },
        Case { label: "pu2-ore-hs", source: "tier5_processing_unit_2s_horizontal_stack_iron_ore_pipe_bypass", item: "processing-unit", rate: 2.0, machine: A3, belt: None, inputs: &["iron-ore", "copper-ore", "stone", "coal", "water", "crude-oil"], excluded: &[] },
        Case { label: "pu2.5-plates-hs", source: "tier5_processing_unit_25s_horizontal_stack_pole_coverage", item: "processing-unit", rate: 2.5, machine: A3, belt: None, inputs: PU9, excluded: &[] },
        Case { label: "pu2-am2-red", source: "processing_unit_2s_am2_fast_belts_validation_baseline", item: "processing-unit", rate: 2.0, machine: A2, belt: RED, inputs: PU9, excluded: &[] },
        Case { label: "u235-kovarex", source: "tier_kovarex_self_loop", item: "uranium-235", rate: 0.1, machine: A3, belt: None, inputs: &["uranium-238"], excluded: &["uranium-processing"] },
        Case { label: "u235-up", source: "tier_uranium_processing_surplus_export", item: "uranium-235", rate: 0.05, machine: A3, belt: None, inputs: &["uranium-ore"], excluded: &["kovarex-enrichment-process"] },
        Case { label: "pentapod0.2", source: "tier_pentapod_egg_self_loop", item: "pentapod-egg", rate: 0.2, machine: A3, belt: None, inputs: &["nutrients", "water"], excluded: &[] },
        Case { label: "fish0.15", source: "tier_fish_breeding_self_loop", item: "raw-fish", rate: 0.15, machine: A3, belt: RED, inputs: &["nutrients", "water"], excluded: &[] },
        Case { label: "bacteria1", source: "tier_bacteria_self_loop_regression", item: "iron-bacteria", rate: 1.0, machine: A3, belt: None, inputs: &["bioflux"], excluded: &["iron-bacteria"] },
        Case { label: "superconductor1", source: "phase0e1_superconductor_electromagnetic_plant", item: "superconductor", rate: 1.0, machine: A3, belt: None, inputs: &["holmium-plate", "copper-plate", "plastic-bar", "light-oil"], excluded: &[] },
        Case { label: "fusioncell1", source: "phase0e1_fusion_power_cell_cryogenic_plant", item: "fusion-power-cell", rate: 1.0, machine: A3, belt: None, inputs: &["lithium-plate", "holmium-plate", "ammonia"], excluded: &[] },
        Case { label: "molteniron5", source: "phase0e1_molten_iron_foundry", item: "molten-iron", rate: 5.0, machine: A3, belt: None, inputs: &["iron-ore", "calcite"], excluded: &[] },
        Case { label: "biolube5", source: "phase0e1_biolubricant_biochamber", item: "lubricant", rate: 5.0, machine: A3, belt: None, inputs: &["jelly"], excluded: &[] },
        Case { label: "ec22-ore-yellow", source: "stress_electronic_circuit_22s_from_ore", item: "electronic-circuit", rate: 22.0, machine: A2, belt: YELLOW, inputs: ORES, excluded: &[] },
        Case { label: "ec23-ore-yellow", source: "stress_electronic_circuit_23s_from_ore", item: "electronic-circuit", rate: 23.0, machine: A2, belt: YELLOW, inputs: ORES, excluded: &[] },
        Case { label: "ec30-ore-yellow", source: "stress_electronic_circuit_30s_from_ore", item: "electronic-circuit", rate: 30.0, machine: A2, belt: YELLOW, inputs: ORES, excluded: &[] },
        Case { label: "ec35-ore-yellow", source: "stress_electronic_circuit_35s_from_ore", item: "electronic-circuit", rate: 35.0, machine: A2, belt: YELLOW, inputs: ORES, excluded: &[] },
        Case { label: "ec40-ore-yellow", source: "stress_electronic_circuit_40s_from_ore", item: "electronic-circuit", rate: 40.0, machine: A2, belt: YELLOW, inputs: ORES, excluded: &[] },
        Case { label: "ec60-ore-red", source: "stress_electronic_circuit_60s_red_from_ore", item: "electronic-circuit", rate: 60.0, machine: A2, belt: RED, inputs: ORES, excluded: &[] },
    ];

    const GAP: i32 = 2;
    let caps = [3.0f64, 3.5, 4.0];

    let mut built = 0usize;
    let mut ge3 = 0usize;
    let mut packable = [0usize; 3];

    for c in cases {
        let inputs: FxHashSet<String> = c.inputs.iter().map(|s| s.to_string()).collect();
        let excluded: FxHashSet<String> = c.excluded.iter().map(|s| s.to_string()).collect();
        let sr = match solver::solve_with_exclusions(c.item, c.rate, &inputs, c.machine, &excluded) {
            Ok(sr) => sr,
            Err(e) => {
                println!("{:<20} SOLVER REFUSED ({}) — counted in the denominator: {e}", c.label, c.source);
                continue;
            }
        };
        let l = match layout::build_bus_layout(
            &sr,
            layout::LayoutOptions {
                max_belt_tier: c.belt.map(|s| s.to_string()),
                ..Default::default()
            },
        ) {
            Ok(l) => l,
            Err(e) => {
                println!("{:<20} LAYOUT REFUSED ({}) — counted in the denominator: {e}", c.label, c.source);
                continue;
            }
        };
        built += 1;
        let bands = rfc058_extract_bands(&l);
        let dims: Vec<(i32, i32)> = bands.iter().map(|b| (b.w, b.h)).collect();
        let (cw, ch) = rfc058_bbox(&bands);
        if bands.len() < 3 {
            println!(
                "{:<20} bands={:<2} ctrl {cw}x{ch} — below the 3-band floor  {dims:?}",
                c.label,
                bands.len(),
            );
            continue;
        }
        ge3 += 1;
        let ctrl_area = (cw as i64) * (ch as i64);
        let mut cells = String::new();
        for (i, &cap) in caps.iter().enumerate() {
            match rfc058_best_pack(&bands, GAP, cap) {
                Some((area, w, h, _)) => {
                    packable[i] += 1;
                    cells.push_str(&format!(
                        "  {cap}:1 {w}x{h} ({:+.0}%)",
                        (area - ctrl_area) as f64 / ctrl_area as f64 * 100.0,
                    ));
                }
                None => cells.push_str(&format!("  {cap}:1 —")),
            }
        }
        println!("{:<20} bands={:<2} ctrl {cw}x{ch}{cells}  {dims:?}", c.label, bands.len());
    }

    let n = cases.len();
    let pct = |k: usize| k as f64 / n as f64 * 100.0;
    println!("\n=== RFC-058 KC2: reach over the e2e corpus ===");
    println!("  corpus rows: {n} ({built} built; refusals stay in the denominator)");
    println!("  >=3 bands: {ge3}/{n} ({:.0}%)", pct(ge3));
    for (i, cap) in caps.iter().enumerate() {
        println!(
            "  >=3 bands AND packable at {cap}:1: {}/{n} ({:.1}%) — KC2 bar is 30%",
            packable[i],
            pct(packable[i]),
        );
    }
}

/// RFC-058 Phase 3: the trunk spike — kill criterion 1's gate, on all three
/// gate fixtures.
///
/// Phases 0–2 are cheap and prove nothing about whether trunks fit; this is
/// the deliberately-throwaway measurement that does. Bands are packed with
/// `rfc058_best_pack` (the phase-0 packer, 3:1 cap), then every band-to-band
/// item flow is routed as a real 1-tile corridor with A*:
///
/// - band rectangles are opaque obstacles;
/// - corridors may cross each other only PERPENDICULARLY (the underground
///   dive every real bus crossing uses); same-axis overlap is forbidden, and
///   corner tiles are hard-blocked for everyone (a UG cannot dive through a
///   turn);
/// - every band gets REAL full-width belt rows reserved before any routing:
///   `ceil(distinct inputs / 2)` feed rows above it (a belt carries two
///   lanes, so two items per feed row — the dual-input row template's
///   capability; fluids count as inputs too, a pipe row being no thinner)
///   and one output row below. This is the row's shared-belt structure the
///   RFC's premise rests on, and it is what a single-tile-termination model
///   would silently omit. Reserved rows count as transport tiles;
/// - a flow STARTS on its producer's output row and terminates on any tile
///   of its consumer's feed rows (a sideload/merge point). Inserter-column
///   specificity, splitters and poles are out of scope, and every flow gets
///   exactly ONE lane (all gate-fixture flows are far below one belt's
///   capacity);
/// - external inputs enter from the arrangement's west edge, the target item
///   exits east — matching the control bus's edge-fed shape;
/// - if any flow fails to route, or a band's reserved rows collide with a
///   neighbouring band, the whole arrangement is re-packed with a wider gap
///   (2 → 8) and every flow re-routed from scratch.
///
/// The score is the REAL bounding box — band rects plus every corridor tile —
/// against the control's as-placed band-bbox, the same quantity kill
/// criterion 1's 10,729-tile baseline is measured in. KC1: the three-fixture
/// aggregate saving must beat −33.0% (half the obstacle-free −66.1%).
///
/// Like every RFC-058 number, host-geometry-relative; the KC1 verdict is
/// recorded in the RFC decision log with the machine it was measured on.
#[test]
#[ignore = "RFC-058 Phase 3 trunk spike — run with --ignored --nocapture"]
fn probe_trunk_spike_gate_fixtures() {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    // One aggregated item flow between two bands (or an edge, when None).
    #[derive(Clone, Debug)]
    struct Flow {
        item: String,
        src: Option<usize>,
        dst: Option<usize>,
        rate: f64,
    }

    // Occupancy bits per tile.
    const BAND: u8 = 1;
    const HCOR: u8 = 2;
    const VCOR: u8 = 4;
    const TURN: u8 = 8;

    struct Grid {
        occ: FxHashMap<(i32, i32), u8>,
        min: (i32, i32),
        max: (i32, i32),
    }

    impl Grid {
        fn passable(&self, t: (i32, i32), horizontal: bool) -> bool {
            if t.0 < self.min.0 || t.0 > self.max.0 || t.1 < self.min.1 || t.1 > self.max.1 {
                return false;
            }
            let bits = self.occ.get(&t).copied().unwrap_or(0);
            if bits & (BAND | TURN) != 0 {
                return false;
            }
            let axis = if horizontal { HCOR } else { VCOR };
            bits & axis == 0
        }
    }

    // Multi-source, multi-target A* returning the path, or None. Visited
    // state is (tile, entry axis) because crossing legality depends on the
    // axis a corridor occupies a tile with.
    fn route(
        grid: &Grid,
        starts: &[(i32, i32)],
        targets: &FxHashSet<(i32, i32)>,
    ) -> Option<Vec<(i32, i32)>> {
        // Admissible heuristic: Manhattan distance to the targets' bounding
        // rectangle. Every target lies inside the rect, so this never
        // exceeds the true remaining cost and is 0 on every target tile —
        // a hint-point heuristic here was inadmissible (h > 0 on targets)
        // and could return non-shortest corridors.
        let tx0 = targets.iter().map(|t| t.0).min().unwrap_or(0);
        let tx1 = targets.iter().map(|t| t.0).max().unwrap_or(0);
        let ty0 = targets.iter().map(|t| t.1).min().unwrap_or(0);
        let ty1 = targets.iter().map(|t| t.1).max().unwrap_or(0);
        let h = move |t: (i32, i32)| {
            (tx0 - t.0).max(t.0 - tx1).max(0) + (ty0 - t.1).max(t.1 - ty1).max(0)
        };
        let mut open: BinaryHeap<Reverse<(i32, i32, (i32, i32), bool)>> = BinaryHeap::new();
        let mut best: FxHashMap<((i32, i32), bool), i32> = FxHashMap::default();
        let mut parent: FxHashMap<((i32, i32), bool), ((i32, i32), bool)> = FxHashMap::default();
        for &s in starts {
            for horizontal in [true, false] {
                if !grid.passable(s, horizontal) {
                    continue;
                }
                if best.get(&(s, horizontal)).is_none_or(|&c| c > 0) {
                    best.insert((s, horizontal), 0);
                    open.push(Reverse((h(s), 0, s, horizontal)));
                }
            }
        }
        while let Some(Reverse((_, cost, tile, horizontal))) = open.pop() {
            if best.get(&(tile, horizontal)).copied().unwrap_or(i32::MAX) < cost {
                continue;
            }
            if targets.contains(&tile) {
                let mut path = vec![tile];
                let mut cur = (tile, horizontal);
                while let Some(&p) = parent.get(&cur) {
                    path.push(p.0);
                    cur = p;
                }
                path.reverse();
                return Some(path);
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let next = (tile.0 + dx, tile.1 + dy);
                let next_h = dy == 0;
                if !grid.passable(next, next_h) {
                    continue;
                }
                // Changing axis makes `tile` a corner, which occupies BOTH
                // axes there — so the current tile must be free on the new
                // axis too. Without this, a corridor could turn on top of a
                // reserved belt row or another corridor's straight run it
                // was only ever allowed to CROSS.
                if next_h != horizontal && !grid.passable(tile, next_h) {
                    continue;
                }
                let ncost = cost + 1;
                if best.get(&(next, next_h)).copied().unwrap_or(i32::MAX) <= ncost {
                    continue;
                }
                best.insert((next, next_h), ncost);
                parent.insert((next, next_h), (tile, horizontal));
                open.push(Reverse((ncost + h(next), ncost, next, next_h)));
            }
        }
        None
    }

    // Stamp a routed path: each tile takes the axis it is traversed on;
    // a tile where the axis changes is a corner and blocks everything.
    fn stamp(grid: &mut Grid, path: &[(i32, i32)]) {
        for (i, &t) in path.iter().enumerate() {
            let axis_in = if i > 0 { Some(path[i - 1].1 == t.1) } else { None };
            let axis_out = if i + 1 < path.len() { Some(path[i + 1].1 == t.1) } else { None };
            let bits = grid.occ.entry(t).or_insert(0);
            match (axis_in, axis_out) {
                (Some(a), Some(b)) if a != b => *bits |= TURN,
                (Some(a), _) | (_, Some(a)) => *bits |= if a { HCOR } else { VCOR },
                (None, None) => {}
            }
        }
    }

    let gate: &[(&str, &str, f64, &[&str], &str)] = &[
        ("sci1-ore", "automation-science-pack", 1.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("pu1-plate", "processing-unit", 1.0, &["iron-plate", "copper-plate", "sulfuric-acid"], "assembling-machine-2"),
    ];

    let (mut agg_ctrl, mut agg_real) = (0i64, 0i64);

    for (label, item, rate, inputs, machine) in gate {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap_or_else(|e| panic!("{label}: solver refused: {e}"));
        let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
            .unwrap_or_else(|e| panic!("{label}: layout refused: {e}"));
        let bands = rfc058_extract_bands(&l);
        let (cw, ch) = rfc058_bbox(&bands);
        let ctrl_area = (cw as i64) * (ch as i64);

        // Aggregate flows at (src band, dst band, item) granularity from the
        // solver's machine specs, splitting a recipe's demand evenly across
        // the bands that carry it — the same convention as the probe's
        // transport proxy. Producer selection (nearest by centre) happens
        // per packing attempt, because "nearest" changes when bands move.
        let mut item_producers: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
        for spec in &sr.machines {
            for out in &spec.outputs {
                for (i, b) in bands.iter().enumerate() {
                    if b.recipes.iter().any(|r| r == &spec.recipe) {
                        item_producers.entry(out.item.as_str()).or_default().push(i);
                    }
                }
            }
        }
        let target_producers: Vec<usize> = sr
            .external_outputs
            .iter()
            .flat_map(|out| item_producers.get(out.item.as_str()).cloned().unwrap_or_default())
            .collect();

        let mut demand: Vec<(usize, String, f64)> = Vec::new();
        for spec in &sr.machines {
            let consumers: Vec<usize> = bands
                .iter()
                .enumerate()
                .filter(|(_, b)| b.recipes.iter().any(|r| r == &spec.recipe))
                .map(|(i, _)| i)
                .collect();
            if consumers.is_empty() {
                continue;
            }
            for inp in &spec.inputs {
                let per = inp.rate * spec.count / consumers.len() as f64;
                for &ci in &consumers {
                    demand.push((ci, inp.item.clone(), per));
                }
            }
        }

        // Widen the packing gap until every flow routes. GAP starts at the
        // probe's 2; each retry re-packs and re-routes everything.
        let mut solved: Option<(i32, i64, i64, usize, usize, usize)> = None;
        'gaps: for gap in 2..=8 {
            let Some((_, _, _, packed)) = rfc058_best_pack(&bands, gap, 3.0) else {
                continue;
            };

            let bmin_x = packed.iter().map(|b| b.x).min().unwrap();
            let bmin_y = packed.iter().map(|b| b.y).min().unwrap();
            let bmax_x = packed.iter().map(|b| b.x + b.w - 1).max().unwrap();
            let bmax_y = packed.iter().map(|b| b.y + b.h - 1).max().unwrap();
            let mut grid = Grid {
                occ: FxHashMap::default(),
                min: (bmin_x - 6, bmin_y - 6),
                max: (bmax_x + 6, bmax_y + 6),
            };
            for b in &packed {
                for x in b.x..b.x + b.w {
                    for y in b.y..b.y + b.h {
                        *grid.occ.entry((x, y)).or_insert(0) |= BAND;
                    }
                }
            }

            // Reserve each band's real belt rows BEFORE any through-routing:
            // ceil(distinct inputs / 2) full-width feed rows above (two lanes
            // per belt), one output row below. A reservation landing on a
            // band tile, or on another band's reservation, means this gap
            // cannot physically hold the rows the bands need — widen.
            let mut band_inputs: Vec<FxHashSet<&str>> = vec![FxHashSet::default(); packed.len()];
            for (ci, item, _) in &demand {
                band_inputs[*ci].insert(item.as_str());
            }
            let feed_row_count = |i: usize| band_inputs[i].len().div_ceil(2) as i32;
            let mut reserved_tiles = 0usize;
            let mut reserve_ok = true;
            'bands: for (i, b) in packed.iter().enumerate() {
                let mut rows: Vec<i32> = (1..=feed_row_count(i)).map(|k| b.y - k).collect();
                rows.push(b.y + b.h); // output row
                for row in rows {
                    for x in b.x..b.x + b.w {
                        let bits = grid.occ.entry((x, row)).or_insert(0);
                        if *bits & (BAND | HCOR) != 0 {
                            reserve_ok = false;
                            break 'bands;
                        }
                        *bits |= HCOR;
                        reserved_tiles += 1;
                    }
                }
            }
            if !reserve_ok {
                println!("{label}: gap {gap}: reserved belt rows collide — widening");
                continue 'gaps;
            }

            // A producer's flows leave from its output row (or a 2-tile
            // extension past either end — a belt extends); a consumer's
            // flows arrive by sideloading anywhere onto its feed rows.
            let out_row = |b: &Rfc058Band| -> Vec<(i32, i32)> {
                (b.x - 2..b.x + b.w + 2).map(|x| (x, b.y + b.h)).collect()
            };
            let feed_tiles = |i: usize, b: &Rfc058Band| -> Vec<(i32, i32)> {
                let mut v = Vec::new();
                for k in 1..=feed_row_count(i) {
                    for x in b.x..b.x + b.w {
                        v.push((x, b.y - k));
                    }
                }
                v
            };
            let centre = |b: &Rfc058Band| (b.x + b.w / 2, b.y + b.h / 2);

            // Nearest-producer selection on the PACKED geometry, then one
            // aggregated flow per (src, dst, item).
            let mut flows: FxHashMap<(Option<usize>, Option<usize>, String), f64> =
                FxHashMap::default();
            for (ci, item, per) in &demand {
                let src = item_producers.get(item.as_str()).and_then(|ps| {
                    ps.iter()
                        .filter(|&&pi| pi != *ci)
                        .min_by_key(|&&pi| {
                            let (px, py) = centre(&packed[pi]);
                            let (cx, cy) = centre(&packed[*ci]);
                            (px - cx).abs() + (py - cy).abs()
                        })
                        .copied()
                });
                if item_producers.contains_key(item.as_str()) && src.is_none() {
                    continue; // self-supplied within the band
                }
                *flows.entry((src, Some(*ci), item.clone())).or_default() += per;
            }
            for &pi in &target_producers {
                *flows
                    .entry((Some(pi), None, format!("OUT:{item}")))
                    .or_default() += *rate / target_producers.len().max(1) as f64;
            }
            let mut flows: Vec<Flow> = flows
                .into_iter()
                .map(|((src, dst, item), rate)| Flow { item, src, dst, rate })
                .collect();
            flows.sort_by(|a, b| {
                b.rate
                    .total_cmp(&a.rate)
                    .then_with(|| a.item.cmp(&b.item))
                    .then_with(|| a.dst.cmp(&b.dst))
                    .then_with(|| a.src.cmp(&b.src))
            });

            let west: Vec<(i32, i32)> = (grid.min.1..=grid.max.1).map(|y| (grid.min.0, y)).collect();
            let east: FxHashSet<(i32, i32)> =
                (grid.min.1..=grid.max.1).map(|y| (grid.max.0, y)).collect();

            let mut corridor_tiles = 0usize;
            let n_flows = flows.len();
            for f in &flows {
                let starts: Vec<(i32, i32)> = match f.src {
                    Some(pi) => out_row(&packed[pi]),
                    None => west.clone(),
                };
                let targets: FxHashSet<(i32, i32)> = match f.dst {
                    Some(ci) => feed_tiles(ci, &packed[ci]).into_iter().collect(),
                    None => east.clone(),
                };
                let Some(path) = route(&grid, &starts, &targets) else {
                    println!(
                        "{label}: gap {gap}: flow {} {:?}->{:?} failed to route — widening",
                        f.item, f.src, f.dst,
                    );
                    continue 'gaps;
                };
                corridor_tiles += path.len();
                stamp(&mut grid, &path);
            }

            // Real bbox: band rects plus every corridor tile.
            let (mut lo_x, mut lo_y, mut hi_x, mut hi_y) = (bmin_x, bmin_y, bmax_x, bmax_y);
            for (&(x, y), &bits) in &grid.occ {
                if bits & (HCOR | VCOR | TURN) != 0 {
                    lo_x = lo_x.min(x);
                    lo_y = lo_y.min(y);
                    hi_x = hi_x.max(x);
                    hi_y = hi_y.max(y);
                }
            }
            let real = ((hi_x - lo_x + 1) as i64) * ((hi_y - lo_y + 1) as i64);
            solved = Some((gap, real, ctrl_area, n_flows, corridor_tiles, reserved_tiles));
            break;
        }

        let Some((gap, real, ctrl, n_flows, corridor_tiles, reserved_tiles)) = solved else {
            panic!("{label}: no gap in 2..=8 routes all flows — KC1 cannot be evaluated");
        };
        agg_ctrl += ctrl;
        agg_real += real;
        println!(
            "{label:<10} ctrl {cw}x{ch}={ctrl}  packed+trunks {real} ({:+.1}%)  gap={gap} flows={n_flows} belt_rows={reserved_tiles} corridors={corridor_tiles}",
            (real - ctrl) as f64 / ctrl as f64 * 100.0,
        );
    }

    let saving = (agg_ctrl - agg_real) as f64 / agg_ctrl as f64 * 100.0;
    println!("\n=== RFC-058 KC1 (trunk spike, three-fixture aggregate) ===");
    println!(
        "  control {agg_ctrl} -> packed+trunks {agg_real}  saving {saving:.1}%  (bar: 33.0%, obstacle-free estimate: 66.1%)"
    );
    println!(
        "  KC1 {}",
        if saving >= 33.0 { "CLEARS" } else { "FAILS — stop; do not re-tune the packer" }
    );
}

/// RFC-058 phase 1 parity: the engine's placer-native band extraction
/// (`bus::bands::extract_bands`, grouped by `RowSpan`) must agree with the
/// phase-0 probe's deliberately decoupled y-projection on the layouts both
/// can see. The probe stays the oracle — its published numbers are the
/// RFC's evidence — so a disagreement here means the placer-native path is
/// wrong, not the probe.
///
/// Also pins phase-2 packing parity: the positions the engine records in
/// `BandPackingPlanned` must reproduce `rfc058_best_pack` on the same
/// bands (same sweep, same tie-break).
///
/// Runs with cell composition and DI forced OFF so the native pass — the
/// one that emits the trace event — is also the layout that wins, keeping
/// the comparison apples-to-apples.
#[test]
fn rfc058_placer_bands_match_y_projection() {
    use spaghettio_core::trace::{self, TraceEvent};

    let cases: &[(&str, &str, f64, &[&str], &str)] = &[
        ("sci1-ore", "automation-science-pack", 1.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("pu1-plate", "processing-unit", 1.0, &["iron-plate", "copper-plate", "sulfuric-acid"], "assembling-machine-2"),
        ("gear15-ore", "iron-gear-wheel", 15.0, &["iron-ore"], "assembling-machine-2"),
    ];

    for (label, item, rate, inputs, machine) in cases {
        let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve_with_palette_exclusions_and_quality(
            item,
            *rate,
            &inputs_set,
            &MachinePalette::default(),
            machine,
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap_or_else(|e| panic!("{label}: solver refused: {e}"));

        let opts = layout::LayoutOptions {
            band_packing: true,
            cell_composition: spaghettio_core::bus::cells::CellComposition::Off,
            direct_insertion: spaghettio_core::bus::di_cell::DirectInsertion::Off,
            ..Default::default()
        };
        let _guard = trace::start_trace();
        let l = layout::build_bus_layout(&sr, opts)
            .unwrap_or_else(|e| panic!("{label}: layout refused: {e}"));
        let events = trace::drain_events();
        drop(_guard);

        // Oracle: the probe's y-projection over the final layout.
        let oracle = rfc058_extract_bands(&l);
        let oracle_rects: Vec<(i32, i32, i32, i32)> =
            oracle.iter().map(|b| (b.x, b.y, b.w, b.h)).collect();

        let last_plan = events
            .iter()
            .rev()
            .find(|e| {
                matches!(
                    e,
                    TraceEvent::BandPackingPlanned { .. } | TraceEvent::BandPackingRefused { .. }
                )
            })
            .unwrap_or_else(|| panic!("{label}: no band-packing event emitted"));

        match last_plan {
            TraceEvent::BandPackingPlanned {
                band_rects,
                packed_w,
                packed_h,
                positions,
                ..
            } => {
                assert_eq!(
                    band_rects, &oracle_rects,
                    "{label}: placer-native band rects diverge from the y-projection oracle",
                );
                let oracle_pack = rfc058_best_pack(&oracle, 2, 3.0)
                    .unwrap_or_else(|| panic!("{label}: oracle packs but engine claims a plan?"));
                assert_eq!(
                    (*packed_w, *packed_h),
                    (oracle_pack.1, oracle_pack.2),
                    "{label}: packed dimensions diverge from the probe packer",
                );
                let oracle_positions: Vec<(i32, i32)> =
                    oracle_pack.3.iter().map(|b| (b.x, b.y)).collect();
                assert_eq!(
                    positions, &oracle_positions,
                    "{label}: planned positions diverge from the probe packer",
                );
            }
            TraceEvent::BandPackingRefused { bands, .. } => {
                // The oracle must agree there is nothing to pack here:
                // either too few bands, or no packing under the cap.
                assert_eq!(*bands, oracle.len(), "{label}: refusal band count diverges");
                assert!(
                    oracle.len() < 3 || rfc058_best_pack(&oracle, 2, 3.0).is_none(),
                    "{label}: engine refused but the oracle packs {} bands",
                    oracle.len(),
                );
            }
            _ => unreachable!(),
        }
    }
}
