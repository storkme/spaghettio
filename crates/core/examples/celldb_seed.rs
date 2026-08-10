//! Seed tool for the cell-interface DB (RFC-067 P1). Regenerates
//! `crates/core/data/celldb.json` from ENGINE OUTPUT — metrics are recorded
//! by this tool, never typed by hand, and provenance names the fixture and
//! SHA so drift is attributable.
//!
//! Extraction: build a source fixture's full layout, then for each target
//! recipe take its machines plus every entity whose segment id starts
//! `row:{recipe}:` — the same attribution rule the Phase-0 cost probe used.
//! Fragments normalize to origin. Ports are DERIVED, not declared: a
//! belt-in run's port is the tile no fragment tile feeds; a belt-out run's
//! port is the tile whose successor lies outside the fragment. Ambiguity
//! (0 or >1 candidates) prints a loud PORT-WARN — K67-1's escape-hatch
//! count is exactly the number of those lines.
//!
//! Top-5 motifs (Phase-0 census) via two fixtures:
//!   electronic-circuit@20 from ore  -> copper-plate, iron-plate,
//!                                      copper-cable, electronic-circuit
//!   advanced-circuit@4 from plates  -> advanced-circuit
use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::celldb::{CellDb, CellEntry, Metrics, Motif, Port, PortKind};
use spaghettio_core::common::{dir_to_vec, entity_size, is_machine_entity};
use spaghettio_core::models::PlacedEntity;
use spaghettio_core::solver;

fn extract(
    entities: &[PlacedEntity],
    recipe: &str,
    machine: &str,
    provenance: &str,
) -> Option<CellEntry> {
    let frag: Vec<PlacedEntity> = entities
        .iter()
        .filter(|e| {
            e.recipe.as_deref() == Some(recipe) && is_machine_entity(&e.name)
                || e
                    .segment_id
                    .as_deref()
                    .is_some_and(|s| s.starts_with(&format!("row:{recipe}:")))
        })
        .cloned()
        .collect();
    if frag.is_empty() {
        println!("EXTRACT-WARN: no entities for {recipe}");
        return None;
    }
    // Normalize to origin.
    let min_x = frag.iter().map(|e| e.x).min().unwrap();
    let min_y = frag.iter().map(|e| e.y).min().unwrap();
    let mut frag: Vec<PlacedEntity> = frag
        .into_iter()
        .map(|mut e| {
            e.x -= min_x;
            e.y -= min_y;
            e
        })
        .collect();
    frag.sort_by_key(|e| (e.y, e.x, e.name.clone()));

    let count = frag
        .iter()
        .filter(|e| e.recipe.as_deref() == Some(recipe) && is_machine_entity(&e.name))
        .count() as u32;

    // Occupied tile set for successor/feeder tests.
    let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut tiles = 0u32;
    let (mut max_x, mut max_y) = (0, 0);
    for e in &frag {
        let (w, h) = entity_size(&e.name);
        for dx in 0..w as i32 {
            for dy in 0..h as i32 {
                occupied.insert((e.x + dx, e.y + dy));
                max_x = max_x.max(e.x + dx);
                max_y = max_y.max(e.y + dy);
                tiles += 1;
            }
        }
    }

    // Derive ports from the belt-in / belt-out / fluid-in segment runs.
    let mut ports: Vec<Port> = Vec::new();
    let mut seg_items: Vec<(String, String)> = Vec::new(); // (seg kind, item)
    for e in &frag {
        if let Some(s) = e.segment_id.as_deref() {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() >= 4 && (parts[2] == "belt-in" || parts[2] == "fluid-in") {
                let key = (parts[2].to_string(), parts[3].to_string());
                if !seg_items.contains(&key) {
                    seg_items.push(key);
                }
            } else if parts.len() >= 3 && parts[2] == "belt-out" {
                let item = parts.get(3).unwrap_or(&recipe).to_string();
                let key = ("belt-out".to_string(), item);
                if !seg_items.contains(&key) {
                    seg_items.push(key);
                }
            }
        }
    }
    for (kind, item) in &seg_items {
        let run: Vec<&PlacedEntity> = frag
            .iter()
            .filter(|e| {
                e.segment_id.as_deref().is_some_and(|s| {
                    let p: Vec<&str> = s.split(':').collect();
                    p.get(2).is_some_and(|k| k == kind)
                        && (p.get(3).is_some_and(|i| i == item) || (kind == "belt-out" && p.len() == 3))
                })
            })
            .collect();
        let candidates: Vec<(i32, i32)> = match kind.as_str() {
            "belt-in" => run
                .iter()
                .filter(|t| {
                    !run.iter().any(|f| {
                        let (dx, dy) = dir_to_vec(f.direction);
                        (f.x + dx, f.y + dy) == (t.x, t.y)
                    })
                })
                .map(|t| (t.x, t.y))
                .collect(),
            "belt-out" => run
                .iter()
                .filter(|t| {
                    let (dx, dy) = dir_to_vec(t.direction);
                    !occupied.contains(&(t.x + dx, t.y + dy))
                })
                .map(|t| (t.x, t.y))
                .collect(),
            _ => run
                .iter()
                .filter(|t| t.x == 0 || t.x == max_x || t.y == 0 || t.y == max_y)
                .map(|t| (t.x, t.y))
                .collect(),
        };
        if candidates.len() != 1 {
            println!(
                "PORT-WARN {recipe}: {kind}:{item} has {} candidate tiles {:?} — picking min",
                candidates.len(),
                candidates
            );
        }
        let Some(&(dx, dy)) = candidates.iter().min() else {
            continue;
        };
        let pk = match kind.as_str() {
            "belt-in" => PortKind::BeltIn,
            "belt-out" => PortKind::BeltOut,
            _ => PortKind::PipeIn,
        };
        ports.push(Port { dx, dy, kind: pk, item: item.clone() });
    }

    Some(CellEntry {
        motif: Motif::Unit {
            recipe: recipe.to_string(),
            machine: machine.to_string(),
            count,
        },
        metrics: Metrics {
            bbox_w: max_x + 1,
            bbox_h: max_y + 1,
            interior_tiles: tiles,
            entity_count: frag.len() as u32,
        },
        entities: frag,
        ports,
        provenance: provenance.to_string(),
        sim_anchor: "unanchored".to_string(),
    })
}

fn main() {
    let sha = std::env::var("SEED_SHA").unwrap_or_else(|_| "worktree".into());
    let mut entries: Vec<CellEntry> = Vec::new();

    let sources: Vec<(&str, f64, &str, Vec<&str>, Vec<(&str, &str)>)> = vec![
        (
            "electronic-circuit",
            20.0,
            "assembling-machine-2",
            vec!["iron-ore", "copper-ore"],
            vec![
                ("copper-plate", "electric-furnace"),
                ("iron-plate", "electric-furnace"),
                ("copper-cable", "assembling-machine-2"),
                ("electronic-circuit", "assembling-machine-2"),
            ],
        ),
        (
            "advanced-circuit",
            4.0,
            "assembling-machine-2",
            vec!["iron-plate", "copper-plate", "plastic-bar"],
            vec![("advanced-circuit", "assembling-machine-2")],
        ),
    ];

    for (item, rate, machine, inputs, targets) in sources {
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let sr = solver::solve(item, rate, &input_set, machine).expect("seed fixture must solve");
        let l = layout::build_bus_layout(&sr, LayoutOptions::default())
            .expect("seed fixture must lay out");
        let prov = format!("engine@{sha} fixture={item}@{rate}");
        for (recipe, target_machine) in targets {
            if let Some(e) = extract(&l.entities, recipe, target_machine, &prov) {
                println!(
                    "seeded {recipe:<24} count={:<3} bbox={}x{} tiles={} ports={}",
                    match &e.motif {
                        Motif::Unit { count, .. } => *count,
                        _ => 0,
                    },
                    e.metrics.bbox_w,
                    e.metrics.bbox_h,
                    e.metrics.interior_tiles,
                    e.ports.len()
                );
                entries.push(e);
            }
        }
    }

    let db = CellDb { version: 1, entries };
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/celldb.json");
    std::fs::write(path, serde_json::to_string_pretty(&db).unwrap()).unwrap();
    println!("wrote {} entries to {path}", db.entries.len());
}
