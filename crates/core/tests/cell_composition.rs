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
/// warnings measured harmless under the PRE-#378 harness (15/15
/// working, 15.0/s exact) — but that run realized researched inserter
/// bonuses; under tech-state parity (declared capacity 0) the same
/// warnings are REAL long-handed-inserter shortfalls (#383). The
/// warnings stay tolerated at the validator level; the fix is RFC-049
/// Phase 3 inserter sizing (#381), after which they should vanish.
/// Dims/entity count pinned to the sim artifact (110×22, 461 entities).
#[test]
fn cell_composed_ec15_zero_errors() {
    use spaghettio_core::validate::{self, Severity};
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
    let issues = validate::validate(&l, Some(&esr))
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
    use spaghettio_core::validate::{self};
    let (sr, comp) = compose_plastic_calibrated();
    let issues = validate::validate(&comp, Some(&sr))
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
    use spaghettio_core::validate::{self, Severity};
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
                match validate::validate(&l, Some(&sr)) {
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
        // RFC-071 B3: the shipped gear@20/am2 cell-composed winner — the
        // only production cell win — earning its registry entry after
        // #715 fixed its drain and the sim measured it AT PLAN
        // (20.0/20.0 produced and delivered, converged, kit-clean).
        // geo_cap 2 = the post-#431 engine default the production
        // candidate composes at; the e2e golden
        // (tier1_iron_gear_wheel_20s, 105 entities) is the proof this
        // row reproduces the shipped geometry — if it ever stops
        // matching, the verification gate refuses the candidate and
        // that golden fails loudly.
        label: "chain-gear20",
        target: "iron-gear-wheel",
        rate: 20.0,
        inputs: &["iron-plate"],
        compose: Compose::Chain,
        geo_cap: 2,
        levels: &[2],
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
    use spaghettio_core::validate::{self, Severity};
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
            Ok(Ok(l)) => match validate::validate(l, Some(&sr)) {
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
                Ok(l) => match validate::validate(&l, Some(&sr)) {
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
    use spaghettio_core::validate::{self, Severity};
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
    let issues = validate::validate(&l, Some(&sr)).unwrap();
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
    // #519 (2026-07-31): input-rate-delivery joins the adjudicated set —
    // the consumption-decremented walker turns the row-input-belt-margin
    // observation above into its concrete per-machine number (tail cable
    // 5.0/s vs 7.5/s on the same zero-margin belt), and the ec15 sim
    // measured the effect at −3.6% delivered (RFC-060 K60-3).
    assert!(
        issues
            .iter()
            .all(|i| i.category == "inserter-item-throughput"
                || i.category == "row-input-belt-margin"
                || i.category == "input-rate-delivery"),
        "only the adjudicated categories tolerated: {issues:?}"
    );
    // Quantum 40 (RFC-072 P2 unit 1): the 2-copy chain runs cable at
    // 22.5/s per copy — the zero-margin 45-on-45 input belt this pin
    // adjudicated (with its measured −3.6% effect) no longer exists on
    // this fixture. The finding's absence IS the improvement; a
    // reappearance would be a regression.
    assert_eq!(
        issues
            .iter()
            .filter(|i| i.category == "row-input-belt-margin")
            .count(),
        0,
        "the zero-margin cable input was dissolved by quantum 40: {issues:?}"
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
    let both_issues = validate::validate(&both, Some(&sr)).unwrap();
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

/// #433: an INTERNAL (producer-cell → consumer-cell) corridor targets
/// exactly one inbound port per ingredient today. A multi-row consumer
/// cell exposes one port PER ROW for the same item (extract.rs's
/// one-port-per-connected-run grouping) — pre-fix, the corridor wired
/// only the first row's port, leaving every row past it silently
/// starved: `compose_chain` returned `Ok` with a broken layout (rows
/// ≥2 read `item-ingredient-shortage` in-sim, per #433's own evidence).
/// Post-fix it refuses loudly instead.
///
/// Forcing the split: EC's OWN cell generation naturally splits into 2
/// rows once its rate crosses ~16/s (empirically probed). But at a
/// REAL solved rate, copper-cable's total output (3x EC's, from the
/// recipe ratio) hits the 45/s ratio-quantization ceiling first and
/// forces K=2 copies — which halves the per-copy EC rate straight back
/// under the row-split threshold. That's exactly why the path is
/// latent on every registered chain fixture (#383's decision log).
/// To force the split without the quantizer intervening, this test
/// takes a real solved `SolverResult` and overrides ONLY the
/// copper-cable spec's `count` (test-only, not flow-conserving) to
/// keep its total output under the 45/s quantum while leaving the EC
/// spec's own rate untouched — `required_copies` stays at K=1, EC's
/// cell still splits into 2 rows, and the internal corridor is
/// exercised exactly as `compose_chain` would drive it for any future
/// recipe/rate combination whose consumer cell splits under its own
/// steam.
#[test]
fn chain_refuses_multirow_internal_corridor() {
    use spaghettio_core::bus::cells::chain::compose_chain_with_capacity;
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut sr = solver::solve_with_palette_exclusions_and_quality(
        "electronic-circuit",
        16.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    for m in sr.machines.iter_mut() {
        if m.recipe == "copper-cable" {
            // 5.0/s per machine * 8 = 40.0/s, under the 45/s quantum
            // (the unmodified solve's 9.6 machines = 48.0/s would push
            // required_copies to K=2 and mask the bug — see doc comment).
            m.count = 8.0;
        }
    }
    let err = compose_chain_with_capacity(&sr, 2).expect_err(
        "a multi-row consumer cell must refuse the internal corridor, not silently wire row 1 and starve the rest",
    );
    assert!(
        err.contains("electronic-circuit") && err.contains("in-ports for copper-cable") && err.contains("#433"),
        "unexpected refusal message: {err}"
    );
}

/// #433 review round 2 (false-positive class). `single_inbound_port`'s
/// first cut refused on raw port COUNT > 1, y-blind — but extract.rs
/// gives a fresh port to each 4-adjacency-CONNECTED RUN of a row's
/// belt-in segment, and UG gaps punched into a SINGLE row's segment
/// (`quad_input_row`'s belt3: per-machine UG-IN at `mx` / gap / UG-OUT
/// at `mx+2`, still one logical segment) break 4-adjacency and split it
/// into several same-y runs. That is NOT the multi-row case #433 is
/// about — it is a same-row artifact of how the segment is drawn.
///
/// Review-measured false positive: assembling-machine-2@1 (raw
/// steel-plate as an external input, so the solve doesn't route it
/// through its own sub-chain; am3) has its OWN iron-gear-wheel consumer
/// ports at (0,2) and (2,2) — same y=2, UG-split, not a second row. The
/// y-blind check refused this chain outright.
///
/// Exercises `compose_chain_with_capacity` directly (not the full
/// decomposition-search path — see
/// `chain_am2_default_options_ships_cell_composed_rescue` below for
/// that) against this exact scenario: must return `Ok`. Confirmed
/// FAILS (refuses) on PR #533's round-1 head (commit a6f21b25) and
/// passes after the y-discrimination fix.
///
/// History: this originally also pinned dims + `geometry_hash`
/// byte-identical to base cc9fb340 (proving the min-x tie-break
/// reproduced `.find`'s westmost pick — verified during the #533
/// review round and recorded there). The pins were retired after the
/// sibling test's native baseline flaked on PR #541's CI run —
/// layout geometry is host-relative through the SAT crossing-zone
/// solver (same class as the stress goldens, which are env-gated for
/// exactly this reason; see docs/status.md). The refusal-regression
/// teeth live in the `Ok` assert, not the pins.
#[test]
fn chain_am2_from_plate_steel_composes_ok_not_false_positive() {
    use spaghettio_core::bus::cells::chain::compose_chain_with_capacity;
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate", "steel-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "assembling-machine-2",
        1.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    let l = compose_chain_with_capacity(&sr, spaghettio_core::common::DEFAULT_INSERTER_CAPACITY)
        .unwrap_or_else(|e| {
            panic!("same-row UG-split ports must NOT trigger the multi-row refusal: {e}")
        });
    assert!(
        l.entities.len() > 400,
        "composed chain should be the full multi-cell layout (got {} entities) — a near-empty result means the compose path degenerated",
        l.entities.len()
    );
}

/// #433 review round 2 companion to the test above: the END-TO-END
/// symptom the reviewer actually measured. `assembling-machine-2@1`
/// (raw steel-plate input, am3) is a case where NATIVE fails
/// acceptance and `CellComposedCandidate` — live under
/// `LayoutOptions::default()` (`cell_composition: Candidate` since
/// RFC-051) — RESCUES it. The round-1 false positive silently starved
/// that rescue: `CellComposedCandidate` errored, so
/// `select_best_decomposition` fell through to native's own inferior
/// result. This asserts the DEFAULT-options end-to-end path ships the
/// rescued cell-composed layout again: it must differ from what
/// `CellComposition::Off` (native-only) produces and be substantially
/// larger — the exact symptom shape of the regression (a silent
/// fallback matches native). Formerly also pinned dims +
/// `geometry_hash` byte-identical to base cc9fb340; retired after the
/// native pin flaked on PR #541's CI run (host-relative SAT
/// crossing-zone geometry — the stress-golden class, docs/status.md).
#[test]
fn chain_am2_default_options_ships_cell_composed_rescue() {
    use spaghettio_core::bus::cells::registry::geometry_hash;
    use spaghettio_core::bus::cells::CellComposition;
    use spaghettio_core::bus::layout::{self, LayoutOptions};
    let inputs: FxHashSet<String> = ["iron-plate", "copper-plate", "steel-plate"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        "assembling-machine-2",
        1.0,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap();
    // RFC-072 Phase 1 (#727 r5): the native-only build for this config
    // is now a NAMED REFUSAL — the restored layout was probed and is
    // genuinely broken (an Error-severity belt-dead-end plus seven
    // reachability findings: four machines unfed, four whose output
    // cannot leave), so "native must build" was a stale premise green
    // over a disconnected layout. The test's real object (its own
    // comment below) is that default options do NOT silently fall back
    // to native's inferior result — which the second build still pins.
    let native = layout::build_bus_layout(
        &sr,
        LayoutOptions {
            cell_composition: CellComposition::Off,
            ..Default::default()
        },
    );
    match &native {
        Err(e) => assert!(
            e.contains("tap assignment unrepairable"),
            "native-only must fail as the NAMED refusal class — got: {e}"
        ),
        Ok(_) => {
            // If a future repair heals this config natively, that is
            // strictly better — the refusal assert stands down.
        }
    }
    let default = layout::build_bus_layout(&sr, LayoutOptions::default())
        .expect("default options must compose (cell-composed rescue must not be silently lost)");
    // No exact dims/hash pins here: layout geometry is host-relative
    // through the SAT crossing-zone solver (this test's native pin came
    // back (23, 50, 424) vs (23, 51, 430) on PR #541's CI run). The
    // round-1 regression shape was "default silently falls back to
    // native's inferior result", which these two asserts pin exactly.
    // With native refused, "default does not silently fall back to
    // native" holds trivially — there is no native result to fall back
    // to. The comparative asserts run only if a future repair heals
    // native.
    if let Ok(native) = &native {
        assert_ne!(
            geometry_hash(&default),
            geometry_hash(native),
            "the rescue must actually differ from native-only — this is the whole point of the candidate"
        );
        assert!(
            default.entities.len() > native.entities.len() * 3 / 2,
            "default options must ship the (much larger) cell-composed rescue, not a near-native fallback: default={} native={}",
            default.entities.len(),
            native.entities.len()
        );
    }
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
    use spaghettio_core::validate::{self, Severity};
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
        "e442f54fb0dfc6cc",
        "EC@15 default (L2) geometry changed — re-verify and re-bless"
    );
    // RE-INSTRUMENTED 2026-08-21 (offpath item 9): this test's original
    // proof — L0 warns / L2 is clean, counted from the inserter-throughput
    // check pair — died when the owner deleted that never-sim-anchored
    // pair. What remains provable without it: the capacity axis still
    // SIZES differently (the #383 fix's mechanism is ladder sizing, and
    // L0-vs-L2 geometry divergence is its observable), and both builds
    // stay Error-free. The premise-guard survives in geometry form: if
    // these hashes ever converge, capacity stopped affecting the chain
    // and this test no longer proves anything — investigate, don't bless.
    let build = |cap: u8| {
        let l = compose_chain_with_capacity(&sr, cap).unwrap();
        let issues =
            validate::validate(&l, Some(&sr)).unwrap_or_else(|e| e.issues);
        assert!(
            issues.iter().all(|i| i.severity != Severity::Error),
            "EC@15 L{cap} must have no errors: {issues:?}"
        );
        format!("{:016x}", geometry_hash(&l))
    };
    assert_ne!(
        build(0),
        build(DEFAULT_INSERTER_CAPACITY),
        "L0 and the L2 default must size differently (#383 capacity axis)"
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
        // RFC-071 B3 (2026-08-23): the shipped gear@20 production cell
        // win, registered at plan after #715 — blessed at the L2 default
        // like chem5.
        ("iron-gear-wheel", 20.0, &["iron-plate"], "chain", 2),
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
            // Quantum 40 (RFC-072 P2 unit 1): ec15's cable runs 45/s,
            // one over the quantum — 2 copies now (was 1 at quantum 45).
            "ec15",
            "electronic-circuit",
            15.0,
            &["iron-plate", "copper-plate"][..],
            2,
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
        // pre-quantization these two REFUSED on the 45/s corridor cap;
        // at quantum 40 (RFC-072 P2 unit 1): cable 90 → 3, cable 180 → 5
        (
            "ec30",
            "electronic-circuit",
            30.0,
            &["iron-plate", "copper-plate"][..],
            3,
        ),
        (
            "ec60",
            "electronic-circuit",
            60.0,
            &["iron-plate", "copper-plate"][..],
            5,
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
    // Past K_MAX=12 the chain refuses loudly (ec600 → cable 1800/s → K=45 at quantum 40).
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
    use spaghettio_core::validate::{self, Severity};
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
    let issues = validate::validate(&l, Some(&sr)).unwrap();
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
    use spaghettio_core::validate::{self, Severity};
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
    let issues = validate::validate(&l, Some(&sr)).unwrap();
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
    use spaghettio_core::validate::{self};
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
        let issues = validate::validate(&l, Some(&sr))
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
    use spaghettio_core::validate::{self, Severity};
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
        let issues = validate::validate(&l, Some(&sr))
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
///
/// **2026-08-01 belt-detour survey finding** (`docs/status.md` "Open
/// tracking issues" — not yet root-caused): this mega-chain composition
/// ships belt runs the new `belt-detour` check flags as genuine detours —
/// 115/125-tile runs for 44/42-tile endpoint separations (2.6x/3.0x,
/// well past both the check's ratio and excess floors, replicated at
/// every mega-block copy). Tolerated here rather than silently allowed —
/// see the check's PR for the full corpus survey this was found against —
/// because fixing it is out of this check's scope; the gate's job is
/// "compose at zero errors," and this is a reported warning, not one.
#[test]
fn mega_chain_chem5_resolves_bus_failure() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, Severity};
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
    let issues = validate::validate(&l, Some(&sr)).expect("must validate");
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "chem5 errors: {errors:?}");
    assert!(
        issues.iter().all(|i| matches!(
            i.category.as_str(),
            "inserter-item-throughput" | "power" | "belt-detour"
        )),
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
///
/// **2026-08-01 belt-detour survey finding** (`docs/status.md` "Open
/// tracking issues" — not yet root-caused): same mega-chain-composition
/// pathology as `mega_chain_chem5_resolves_bus_failure` — 224-tile belt
/// runs for 87-tile endpoint separations (2.6x, well past both the new
/// `belt-detour` check's floors, replicated per mega-block copy).
/// Tolerated explicitly rather than silently allowed.
#[test]
fn mega_chain_pu4_resolves_bus_failure() {
    use spaghettio_core::bus::cells::chain::compose_chain;
    use spaghettio_core::validate::{self, Severity};
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
    let issues = match validate::validate(&l, Some(&sr)) {
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
            "inserter-item-throughput"
                | "inserter-throughput"
                | "power"
                | "row-output-lane-budget"
                | "belt-detour"
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
    use spaghettio_core::validate::{self, Severity};
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
    let issues = match validate::validate(&l, Some(&sr)) {
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

// The three RFC-055 compact-order experiments (real-geometry, acceptance
// corpus, Factorio artifact producer) were deleted 2026-08-20 with
// cells/placement.rs and ChainOrder::Compact — owner extended the #632 A2
// precedent (offpath Tier 2, #675); RFC-055 decision log has the record.

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
    use spaghettio_core::validate::{self, Severity};
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
                let d = validate::validate(&l, Some(&sr));
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
    use spaghettio_core::validate::{self};
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
            Ok(l) => match validate::validate(&l, Some(&sr)) {
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
///
/// Trimmed from its original form (second-opinion review on PR #645, #632
/// A2): the original also compared each case's raw-bus pole count against a
/// `compact_validated_geometry`-compacted variant ("COMPACTION MADE IT
/// WORSE"), and — for the cell-composed fixtures below — computed a compacted
/// variant's pole count that was never even printed (`let _ = (cd, cn);` in
/// the original). Both uses were incidental to this probe's actual purpose
/// (measuring the NORMAL pipeline, per the doc comment above) and depended on
/// the now-deleted `bus::compaction` module, so they're cut rather than
/// ported; the surviving normal-pipeline and composed/repaired measurements
/// below are unchanged from the original.
#[test]
#[ignore = "measurement probe — pole connectivity across the pipeline"]
fn probe_pole_connectivity_census() {
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
        total += 1;
        if bd > 0 { dirty_raw += 1; }
        println!("{label:<14} bus {bd:>3}/{bn:<4} disconnected");
    }
    println!("\n{dirty_raw}/{total} raw bus layouts have a fragmented pole network");

    // Cell composition is the other producer of layouts, and the one whose
    // compacted mil5 control carries 2 disconnected poles.
    println!("\n--- cell-composed fixtures ---");
    for label in ["chain-mil5ore", "mega-chain-chem5raw", "mega-chain-pu4raw", "mega-chain-usp2raw"] {
        let fixture = SimFixture::find(label);
        let inputs: FxHashSet<String> = fixture.inputs.iter().map(|s| s.to_string()).collect();
        // Solve-and-discard: the solve itself is the refusal gate (skip
        // fixtures the solver won't accept), unused otherwise now that the
        // compacted-variant comparison (the solve's only consumer) is gone.
        let Ok(_sr) = solver::solve_with_palette_exclusions_and_quality(
            fixture.target, fixture.rate, &inputs, &MachinePalette::default(),
            "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
        ) else { continue };
        let base = fixture.compose_layout();
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
    }
}

// The RFC-058 / RFC-064-P3 band-packing probe section that closed this
// file (~2,190 lines: shared helpers, phase-0/3/4/5 probes, the premise
// tripwire, and the placer-parity gate) was deleted 2026-08-20 with
// bus::bands itself — owner extended the #632 A2 precedent
// (offpath-code-followups Tier 2; falsification record: RFC-058 decision
// log).
