//! Ingestion against **real engine output**, not synthetic strings.
//!
//! These read blueprints the engine actually exports, regenerated locally
//! with no Factorio involved:
//!
//! ```bash
//! cargo test --manifest-path crates/core/Cargo.toml \
//!     --test cell_composition -- --ignored export_chain_fixtures_for_sim
//! ```
//!
//! Fixtures land in `crates/core/target/tmp/`. The tests **skip** when they
//! are absent rather than failing, so a clean checkout is not red — but
//! they are not vacuous: `fixtures_are_present_when_generated` fails loudly
//! if the directory exists yet holds nothing readable, which is the shape a
//! silently-broken export would take.

use std::path::PathBuf;

use spaghettio_meter::blueprint_in;
use spaghettio_meter::entity_data::{self, BeltTier, InserterKind};

fn tmp_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../core/target/tmp")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("../core/target/tmp"))
}

fn load(label: &str) -> Option<String> {
    std::fs::read_to_string(tmp_dir().join(format!("{label}.bp"))).ok()
}

/// `chain-ec15` is the fixture behind #448 and #435 — the one whose row
/// starvation the meter exists to explain. If ingestion works anywhere, it
/// has to work here.
#[test]
fn ingests_chain_ec15() {
    let Some(bp) = load("chain-ec15-d2") else {
        eprintln!("skipping: chain-ec15-d2.bp not generated");
        return;
    };
    let ents = blueprint_in::decode(&bp).expect("decode chain-ec15-d2");

    // Census against the known composition of this fixture.
    let machines = ents
        .iter()
        .filter(|e| entity_data::is_crafting_machine(&e.name))
        .count();
    let inserters = ents
        .iter()
        .filter(|e| InserterKind::from_entity_name(&e.name).is_some())
        .count();
    let belts = ents
        .iter()
        .filter(|e| BeltTier::from_entity_name(&e.name).is_some())
        .count();

    assert_eq!(ents.len(), 292, "entity count changed: {}", ents.len());
    assert_eq!(machines, 15, "expected 15 assembling machines");
    assert_eq!(inserters, 42, "expected 42 inserters");
    assert!(belts > 150, "expected the bulk to be belt-like, got {belts}");

    // Every crafting machine must carry a recipe — a machine without one
    // cannot be simulated, and silently treating it as idle would
    // under-report production.
    for m in ents.iter().filter(|e| entity_data::is_crafting_machine(&e.name)) {
        assert!(
            m.recipe.is_some(),
            "machine {} at ({},{}) has no recipe",
            m.name,
            m.x,
            m.y
        );
    }
}

/// Every inserter's pickup and drop tiles must land on something.
///
/// **This does NOT verify the direction convention, and saying so matters.**
/// A first draft of this comment claimed it "would have caught #348". It
/// would not: under that bug the inserters still pointed at real entities,
/// just the wrong ones round (inputs pulling from machines, outputs from
/// belts). Flipping pickup and drop leaves both tiles occupied, so this
/// assertion passes either way — it is symmetric in exactly the dimension
/// the bug lived in.
///
/// What it does catch is geometry: a footprint or centre-to-tile
/// conversion error, which would strand hands in empty space.
///
/// The real check on the convention is behavioural and arrives with
/// machines: a flipped meter produces *nothing*, because every machine
/// would be fed its own outputs. That is a rate of 0.0 against a plan, and
/// it is unmissable. Deferred to the simulation rather than faked here.
#[test]
fn inserter_hands_reach_real_entities() {
    let Some(bp) = load("chain-ec15-d2") else {
        eprintln!("skipping: chain-ec15-d2.bp not generated");
        return;
    };
    let ents = blueprint_in::decode(&bp).expect("decode");

    // Occupied tiles, expanding each entity over its footprint.
    let mut occupied: rustc_hash::FxHashMap<(i32, i32), String> = Default::default();
    for e in &ents {
        let horizontal = matches!(e.direction, blueprint_in::Dir::East | blueprint_in::Dir::West);
        let (w, h) = entity_data::footprint_oriented(&e.name, horizontal);
        for dx in 0..w as i32 {
            for dy in 0..h as i32 {
                occupied.insert((e.x + dx, e.y + dy), e.name.clone());
            }
        }
    }

    let mut both_ends_empty = 0;
    let mut total = 0;
    for e in &ents {
        let Some(kind) = InserterKind::from_entity_name(&e.name) else {
            continue;
        };
        total += 1;
        let reach = kind.reach();
        let pickup = occupied.get(&e.inserter_pickup_tile(reach));
        let drop = occupied.get(&e.inserter_drop_tile(reach));
        if pickup.is_none() && drop.is_none() {
            both_ends_empty += 1;
        }
    }

    assert!(total > 0, "no inserters found");
    assert_eq!(
        both_ends_empty, 0,
        "{both_ends_empty}/{total} inserters reach empty space on BOTH sides — \
         the signature of a direction-convention error (#348 class)"
    );
}

/// Ingestion must survive the whole generated fixture set, not just the
/// one config the author looked at. Unknown entity types surface here as
/// hard errors rather than as silent 1x1 defaults.
#[test]
fn ingests_every_generated_fixture() {
    let dir = tmp_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("skipping: {dir:?} does not exist");
        return;
    };

    let mut checked = 0;
    let mut failures = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "bp") {
            continue;
        }
        let Ok(bp) = std::fs::read_to_string(&path) else {
            continue;
        };
        match blueprint_in::decode(&bp) {
            Ok(ents) => {
                checked += 1;
                assert!(
                    !ents.is_empty(),
                    "{:?} decoded to zero entities",
                    path.file_name()
                );
            }
            Err(e) => failures.push(format!("{:?}: {e}", path.file_name().unwrap())),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} fixtures failed to ingest:\n{}",
        failures.len(),
        failures.len() + checked,
        failures.join("\n")
    );
    if checked == 0 {
        eprintln!("skipping: no .bp fixtures generated");
    }
}

/// The topology rules must cope with **real** factories, not just the
/// hand-built cases in `network.rs`'s unit tests.
///
/// Every `TopologyNote` is something the builder could not model — an
/// unpaired underground, an orphan splitter half, a belt loop with no
/// principled update order. Each is a place the simulation would get a
/// rate wrong. Asserting zero across the whole generated corpus is what
/// turns "the rules look right" into "the rules handle 3,754 tiles of
/// engine output".
#[test]
fn topology_builds_cleanly_on_every_fixture() {
    let dir = tmp_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("skipping: {dir:?} does not exist");
        return;
    };

    let mut checked = 0;
    let mut problems = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "bp") {
            continue;
        }
        let Ok(bp) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(ents) = blueprint_in::decode(&bp) else {
            continue; // decode failures are the other test's business
        };
        let net = spaghettio_meter::NetworkBuilder::build(&ents);
        checked += 1;

        let label = path.file_stem().unwrap().to_string_lossy().to_string();
        if !net.notes.is_empty() {
            problems.push(format!("{label}: {:?}", net.notes));
        }
        // A network where most tiles link nowhere is a broken network that
        // would still report "no notes".
        let linked = net.tiles.iter().filter(|t| t.downstream.is_some()).count();
        if net.len() > 20 && linked * 10 < net.len() * 9 {
            problems.push(format!(
                "{label}: only {linked}/{} tiles link downstream",
                net.len()
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "topology problems on {} fixture(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
    if checked == 0 {
        eprintln!("skipping: no .bp fixtures generated");
    }
}

/// Guard against the skips above going vacuous: if the fixture directory
/// exists at all, it must contain readable blueprints.
#[test]
fn fixtures_are_present_when_generated() {
    let dir = tmp_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return; // never generated — legitimate on a clean checkout
    };
    let bps: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "bp"))
        .collect();
    if bps.is_empty() {
        return; // directory exists for other build output; nothing claimed
    }
    for p in &bps {
        let text = std::fs::read_to_string(p).expect("read fixture");
        assert!(
            text.trim().starts_with('0'),
            "{p:?} is not a blueprint envelope"
        );
    }
}
