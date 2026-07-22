//! RFC-048 Phase 1: cell catalog + stamper + composition harness.
//!
//! Test-only (never wired into production layout paths — kill 4: cells
//! are produced BY the engine, cropped and composed here; there is one
//! layout stack). See `docs/rfc-048-cell-composition.md`.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout;
use spaghettio_core::common::QualityTier;
use spaghettio_core::models::LayoutResult;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;

/// Generate a single-recipe layout via the full engine pipeline — the
/// engine-as-cell-generator bootstrap path (RFC-048 Design).
fn generate_row_layout(
    item: &str,
    rate: f64,
    inputs: &[&str],
) -> (spaghettio_core::models::SolverResult, LayoutResult) {
    let inputs: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        item,
        rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap_or_else(|e| panic!("solve {item}: {e}"));
    let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
        .unwrap_or_else(|e| panic!("layout {item}: {e}"));
    (sr, l)
}

/// Exploration probe (run with --nocapture): geometry of the two
/// candidate cell source layouts, to design the crop + port extraction.
#[test]
#[ignore = "exploration probe, not a gate"]
fn probe_cell_source_geometry() {
    for (label, item, rate, inputs) in [
        ("cable", "copper-cable", 15.0, &["copper-plate"][..]),
        ("ec", "electronic-circuit", 5.0, &["iron-plate", "copper-cable"][..]),
    ] {
        let (sr, l) = generate_row_layout(item, rate, inputs);
        println!("== {label}: {}x{}, {} entities ==", l.width, l.height, l.entities.len());
        for m in &sr.machines {
            println!("   spec {} x{:.2}", m.recipe, m.count);
        }
        // Edge survey: entities in the outermost 2 columns/rows, belts only.
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


/// A cell port derived from a kept belt's terminal stub.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Port {
    /// "W" or "E" (Phase 1 composes horizontally-fed vertical stacks).
    edge: &'static str,
    /// y in cell-local coordinates.
    y: i32,
    item: String,
    /// true = flow into the cell.
    inbound: bool,
}

/// A pre-verified cell: engine-generated row machinery, cropped to the
/// `row:*` segment partition (+ machines and poles), normalized to
/// (0,0), with ports derived from belt terminal stubs.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Cell {
    entities: Vec<spaghettio_core::models::PlacedEntity>,
    width: i32,
    height: i32,
    ports: Vec<Port>,
}

/// Crop an engine layout to its cell interior: keep entities whose
/// segment is `row:*` or `pole`, plus machines (recipe carriers).
/// Everything trunk/tap/ghost/merger-shaped is bus machinery, shed.
fn extract_cell(l: &LayoutResult) -> Cell {
    use spaghettio_core::common::{is_belt_entity, is_machine_entity};
    let keep: Vec<_> = l
        .entities
        .iter()
        .filter(|e| {
            let seg = e.segment_id.as_deref().unwrap_or("");
            seg.starts_with("row:") || seg == "pole" || is_machine_entity(&e.name)
        })
        .cloned()
        .collect();
    let min_x = keep.iter().map(|e| e.x).min().unwrap_or(0);
    let min_y = keep.iter().map(|e| e.y).min().unwrap_or(0);
    let mut entities = keep;
    for e in &mut entities {
        e.x -= min_x;
        e.y -= min_y;
    }
    let width = entities.iter().map(|e| e.x).max().unwrap_or(0) + 1;
    let height = entities.iter().map(|e| e.y).max().unwrap_or(0) + 1;

    // Ports: for each row:*:belt-in/belt-out segment, the terminal stub
    // in its flow direction (in-belts: westmost tile; out-belts:
    // eastmost tile). Phase 1 rows all flow W->E.
    let mut ports: Vec<Port> = Vec::new();
    let mut segs: std::collections::BTreeMap<String, Vec<&spaghettio_core::models::PlacedEntity>> =
        Default::default();
    for e in &entities {
        if let Some(seg) = e.segment_id.as_deref() {
            if is_belt_entity(&e.name) && (seg.contains(":belt-in") || seg.contains(":belt-out")) {
                segs.entry(seg.to_string()).or_default().push(e);
            }
        }
    }
    for (seg, belts) in &segs {
        let inbound = seg.contains(":belt-in");
        let e = if inbound {
            belts.iter().min_by_key(|e| e.x).unwrap()
        } else {
            belts.iter().max_by_key(|e| e.x).unwrap()
        };
        ports.push(Port {
            edge: if inbound { "W" } else { "E" },
            y: e.y,
            item: e.carries.clone().unwrap_or_default(),
            inbound,
        });
    }
    Cell { entities, width, height, ports }
}

/// Probe: extracted cells' dimensions, ports, and full belt inventory.
#[test]
#[ignore = "exploration probe, not a gate"]
fn probe_extracted_cells() {
    for (label, item, rate, inputs) in [
        ("cable", "copper-cable", 15.0, &["copper-plate"][..]),
        ("ec", "electronic-circuit", 5.0, &["iron-plate", "copper-cable"][..]),
    ] {
        let (_sr, l) = generate_row_layout(item, rate, inputs);
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
