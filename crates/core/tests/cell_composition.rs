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
    /// Terminal tile (cell-local): the westmost tile of an in-run /
    /// eastmost of an out-run — corridors attach at x±1.
    x: i32,
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
    // Contiguity grouping (decision log 2026-07-22): a segment can span
    // multiple internal rows (the placer splits machine groups), so one
    // port per CONNECTED RUN of same-segment belt tiles, at the run's
    // flow-direction terminal.
    for (seg, belts) in &segs {
        let inbound = seg.contains(":belt-in");
        let tiles: std::collections::BTreeSet<(i32, i32)> =
            belts.iter().map(|e| (e.x, e.y)).collect();
        let mut seen: std::collections::BTreeSet<(i32, i32)> = Default::default();
        for &start in &tiles {
            if seen.contains(&start) {
                continue;
            }
            // flood-fill this run (4-adjacency)
            let mut run = vec![start];
            let mut stack = vec![start];
            seen.insert(start);
            while let Some((x, y)) = stack.pop() {
                for n in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if tiles.contains(&n) && seen.insert(n) {
                        run.push(n);
                        stack.push(n);
                    }
                }
            }
            let &(px, py) = if inbound {
                run.iter().min_by_key(|(x, _)| *x).unwrap()
            } else {
                run.iter().max_by_key(|(x, _)| *x).unwrap()
            };
            let item = belts
                .iter()
                .find(|e| (e.x, e.y) == (px, py))
                .and_then(|e| e.carries.clone())
                .unwrap_or_default();
            let _ = seg;
            ports.push(Port {
                edge: if inbound { "W" } else { "E" },
                x: px,
                y: py,
                item,
                inbound,
            });
        }
    }
    ports.sort_by_key(|p| (p.edge, p.y));
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


/// Stamp a belt polyline along orthogonal waypoints. Directions follow
/// travel; corners are B11 (lane-preserving) by construction. The LAST
/// tile faces toward `final_dir` so it can feed the next segment.
fn stamp_path(
    out: &mut Vec<spaghettio_core::models::PlacedEntity>,
    waypoints: &[(i32, i32)],
    item: &str,
    belt: &str,
    seg: &str,
) {
    use spaghettio_core::models::{EntityDirection, PlacedEntity};
    let mut tiles: Vec<(i32, i32)> = Vec::new();
    for w in waypoints.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        assert!(x0 == x1 || y0 == y1, "orthogonal only: {w:?}");
        let (dx, dy) = ((x1 - x0).signum(), (y1 - y0).signum());
        let (mut x, mut y) = (x0, y0);
        while (x, y) != (x1, y1) {
            tiles.push((x, y));
            x += dx;
            y += dy;
        }
    }
    tiles.push(*waypoints.last().unwrap());
    for i in 0..tiles.len() {
        let (x, y) = tiles[i];
        let (nx, ny) = if i + 1 < tiles.len() { tiles[i + 1] } else {
            let (px, py) = tiles[i - 1];
            (x + (x - px), y + (y - py))
        };
        let dir = match (nx - x, ny - y) {
            (1, 0) => EntityDirection::East,
            (-1, 0) => EntityDirection::West,
            (0, 1) => EntityDirection::South,
            (0, -1) => EntityDirection::North,
            d => panic!("bad step {d:?}"),
        };
        out.push(PlacedEntity {
            name: belt.to_string(),
            x,
            y,
            direction: dir,
            carries: Some(item.to_string()),
            segment_id: Some(seg.to_string()),
            ..Default::default()
        });
    }
}

/// Compose one ratio pair: cable cell (west) -> corridor -> EC cell
/// (east), external plate/iron feeds extended to the west boundary.
/// Returns the composed LayoutResult (5 EC/s planned).
fn compose_ratio_pair() -> (spaghettio_core::models::SolverResult, LayoutResult) {
    compose_pairs(1)
}

/// Compose N ratio pairs stacked vertically (5·N EC/s planned): each
/// pair is the cable cell + corridor + EC cell from `compose_pairs`'s
/// single-pair geometry, at y offset k·PAIR_PITCH; feeds and outputs
/// run to the layout boundaries independently per pair (contract-clean;
/// a collection corridor is a later refinement, not a gate need).
fn compose_pairs(n: i32) -> (spaghettio_core::models::SolverResult, LayoutResult) {
    use spaghettio_core::models::PlacedEntity;
    let (_csr, cl) = generate_row_layout("copper-cable", 15.0, &["copper-plate"]);
    let (esr_one, el) = generate_row_layout("electronic-circuit", 5.0, &["iron-plate", "copper-cable"]);
    let cable = extract_cell(&cl);
    let ec = extract_cell(&el);
    // Solver context for the COMPOSED total (validation's demand model);
    // machine counts scale with n by construction of the ratio cell.
    let esr = if n == 1 {
        esr_one
    } else {
        let inputs: FxHashSet<String> =
            ["iron-plate", "copper-cable"].iter().map(|s| s.to_string()).collect();
        solver::solve_with_palette_exclusions_and_quality(
            "electronic-circuit",
            5.0 * n as f64,
            &inputs,
            &MachinePalette::default(),
            "assembling-machine-3",
            &FxHashSet::default(),
            QualityTier::Normal,
        )
        .unwrap()
    };
    const PAIR_PITCH: i32 = 16;

    // Placement: 4-tile W feed margin; cable cell at x=4; corridor gap 6
    // wide; EC cell east of it, vertically centered on the cable cell.
    let cable_x = 4;
    let corridor_x = cable_x + cable.width; // corridor occupies [corridor_x, corridor_x+5]
    let ec_x = corridor_x + 6;

    let mut entities: Vec<PlacedEntity> = Vec::new();
    for k in 0..n {
    let dy = k * PAIR_PITCH;
    let ec_y = 3 + dy;
    for e in &cable.entities {
        let mut e = e.clone();
        e.x += cable_x;
        e.y += dy;
        entities.push(e);
    }
    for e in &ec.entities {
        let mut e = e.clone();
        e.x += ec_x;
        e.y += ec_y;
        entities.push(e);
    }

    // External feeds attach at each inbound W port's terminal-1.
    // Cable cell feeds: straight runs from the boundary.
    for port in cable.ports.iter().filter(|p| p.inbound) {
        stamp_path(
            &mut entities,
            &[(0, port.y + dy), (cable_x + port.x - 1, port.y + dy)],
            &port.item,
            "express-transport-belt",
            &format!("corridor:feed:{}:{}", port.item, port.y + dy),
        );
    }
    // EC iron feed: UG hop under the cable cell (B12 weaving) — entry
    // west of the cell, exit east of it (express reach 8 covers the
    // 8-wide cell exactly), then East to the port terminal-1.
    {
        use spaghettio_core::models::{EntityDirection, PlacedEntity};
        let port = ec.ports.iter().find(|p| p.inbound && p.item == "iron-plate").unwrap();
        let y = ec_y + port.y;
        stamp_path(&mut entities, &[(0, y), (cable_x - 2, y)], "iron-plate",
            "express-transport-belt", &format!("corridor:feed:iron-plate:{k}"));
        for (x, io) in [(cable_x - 1, "input"), (cable_x + cable.width, "output")] {
            entities.push(PlacedEntity {
                name: "express-underground-belt".to_string(),
                x,
                y,
                direction: EntityDirection::East,
                io_type: Some(io.to_string()),
                carries: Some("iron-plate".to_string()),
                segment_id: Some(format!("corridor:feed:iron-plate:{k}")),
                ..Default::default()
            });
        }
        stamp_path(&mut entities, &[(cable_x + cable.width + 1, y), (ec_x + port.x - 1, y)],
            "iron-plate", "express-transport-belt", &format!("corridor:feed:iron-plate:{k}"));
    }

    // Cable corridor: two outs -> 2->1 splitter -> EC cable-in port.
    // outs at (corridor_x-1 is last cell tile; runs start at corridor_x)
    let outs: Vec<&Port> = cable.ports.iter().filter(|p| !p.inbound).collect();
    assert_eq!(outs.len(), 2, "cable cell exposes two outs");
    let (o1, o2) = (outs[0], outs[1]); // (x,y) terminals
    let ec_cable_port = ec.ports.iter().find(|p| p.inbound && p.item == "copper-cable").unwrap();
    let ec_cable_in_y = ec_y + ec_cable_port.y;
    // Splitter (ONE entity, 2-wide spanning y=o1.y, o1.y+1) at sx.
    let sx = corridor_x + 2;
    stamp_path(&mut entities, &[(cable_x + o1.x + 1, o1.y + dy), (sx - 1, o1.y + dy)],
        "copper-cable", "fast-transport-belt", &format!("corridor:cable:a:{k}"));
    stamp_path(
        &mut entities,
        &[(cable_x + o2.x + 1, o2.y + dy), (sx - 1, o2.y + dy), (sx - 1, o1.y + 1 + dy)],
        "copper-cable",
        "fast-transport-belt",
        &format!("corridor:cable:b:{k}"),
    );
    entities.push(spaghettio_core::models::PlacedEntity {
        name: "fast-splitter".to_string(),
        x: sx,
        y: o1.y + dy,
        direction: spaghettio_core::models::EntityDirection::East,
        carries: Some("copper-cable".to_string()),
        segment_id: Some("corridor:cable:merge".to_string()),
        ..Default::default()
    });
    // Merged run from splitter output to the EC cable-in terminal-1.
    stamp_path(
        &mut entities,
        &[(sx + 1, o1.y + dy), (sx + 2, o1.y + dy), (sx + 2, ec_cable_in_y), (ec_x + ec_cable_port.x - 1, ec_cable_in_y)],
        "copper-cable",
        "fast-transport-belt",
        &format!("corridor:cable:c:{k}"),
    );
    // Final product: extend the EC out port to the east boundary.
    let ec_out = ec.ports.iter().find(|p| !p.inbound).unwrap();
    let out_x0 = ec_x + ec_out.x + 1;
    let out_y = ec_y + ec_out.y;
    stamp_path(&mut entities, &[(out_x0, out_y), (out_x0 + 3, out_y)],
        "electronic-circuit", "transport-belt", &format!("corridor:out:ec:{k}"));

    // Corridor pole stitch: cells carry their own poles; add a pole
    // column in the corridor so the network is connected.
    for y in [dy, dy + 7, dy + 14] {
        entities.push(PlacedEntity {
            name: "medium-electric-pole".to_string(),
            x: sx + 4,
            y,
            direction: spaghettio_core::models::EntityDirection::North,
            segment_id: Some("pole".to_string()),
            ..Default::default()
        });
    }
    } // end pairs loop

    let max_y = entities.iter().map(|e| e.y).max().unwrap_or(0);
    let width = entities.iter().map(|e| e.x).max().unwrap() + 1;
    let height = max_y + 1;
    let mut l = LayoutResult::default();
    l.entities = entities;
    l.width = width;
    l.height = height;
    l.stacking = 1;
    (esr, l)
}

/// First composition gate probe: one ratio pair, validated.
#[test]
#[ignore = "exploration probe while the composer stabilizes"]
fn probe_compose_ratio_pair() {
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let (esr, l) = compose_ratio_pair();
    println!("composed pair: {}x{}, {} entities", l.width, l.height, l.entities.len());
    match validate::validate(&l, Some(&esr), LayoutStyle::Bus) {
        Ok(issues) => {
            let e = issues.iter().filter(|i| i.severity == Severity::Error).count();
            println!("validation Ok: {} errors / {} issues", e, issues.len());
            for i in issues.iter().take(12) {
                println!("  [{:?}] {} {}", i.severity, i.category, i.message);
            }
        }
        Err(er) => {
            let s = format!("{er}");
            for line in s.lines().take(15) {
                println!("  {line}");
            }
        }
    }
}


/// Gate probe: EC@15/s composed (3 ratio pairs), validated.
#[test]
#[ignore = "exploration probe while the composer stabilizes"]
fn probe_compose_ec15() {
    use spaghettio_core::validate::{self, LayoutStyle, Severity};
    let (esr, l) = compose_pairs(3);
    println!("composed EC@15: {}x{} = {} tiles, {} entities", l.width, l.height, l.width * l.height, l.entities.len());
    match validate::validate(&l, Some(&esr), LayoutStyle::Bus) {
        Ok(issues) => {
            let e = issues.iter().filter(|i| i.severity == Severity::Error).count();
            println!("validation Ok: {} errors / {} issues", e, issues.len());
            for i in issues.iter().take(12) {
                println!("  [{:?}] {} {}", i.severity, i.category, i.message);
            }
        }
        Err(er) => {
            let s = format!("{er}");
            println!("validation FAILED:");
            for line in s.lines().filter(|l| l.contains("[error]")).take(12) {
                println!("  {line}");
            }
        }
    }
}
