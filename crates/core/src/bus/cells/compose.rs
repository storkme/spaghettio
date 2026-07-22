//! Cell composition (RFC-051 Phase A): corridor stamping + the Phase-1
//! composers, lifted verbatim from `tests/cell_composition.rs` (parity
//! gate — see `extract.rs` module docs). The calibrated composers keep
//! the sim-kit boundary geometry (#363 rules: north feeds at 4-tile rig
//! pitch, south drains, west→east record order); the production
//! boundary-attachment form and the linear-chain auto-placer land in
//! Phase B, under the calibrated-twin invariant (RFC-051 Design).

use rustc_hash::FxHashSet;
use crate::common::QualityTier;
use crate::models::LayoutResult;
use crate::recipe_db::MachinePalette;
use crate::solver;
use super::extract::{extract_cell, generate_cell_layout, Port};

/// Stamp a belt polyline along orthogonal waypoints. Directions follow
/// travel; corners are B11 (lane-preserving) by construction. The LAST
/// tile faces toward `final_dir` so it can feed the next segment.
pub fn stamp_path(
    out: &mut Vec<crate::models::PlacedEntity>,
    waypoints: &[(i32, i32)],
    item: &str,
    belt: &str,
    seg: &str,
) {
    use crate::models::{EntityDirection, PlacedEntity};
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

/// Compose the plastic cell with boundary feeds (coal belt + petroleum
/// pipe) — the fluid-boundary calibration geometry.
pub fn compose_plastic_calibrated() -> (crate::models::SolverResult, LayoutResult) {
    use crate::models::{BoundaryRecord, EntityDirection, PlacedEntity};
    let (sr, l) = generate_cell_layout("plastic-bar", 2.0, &["petroleum-gas", "coal"]);
    let cell = extract_cell(&l);
    let cx = 4;
    let cy = 3;
    let mut entities: Vec<PlacedEntity> = Vec::new();
    for e in &cell.entities {
        let mut e = e.clone();
        e.x += cx;
        e.y += cy;
        entities.push(e);
    }
    let mut b_in: Vec<BoundaryRecord> = Vec::new();
    let mut b_out: Vec<BoundaryRecord> = Vec::new();
    // Coal: north-edge column (calibrated South direction), corner East.
    let coal = cell.ports.iter().find(|p| p.inbound && p.item == "coal").unwrap();
    stamp_path(&mut entities, &[(1, 0), (1, cy + coal.y), (cx + coal.x - 1, cy + coal.y)],
        "coal", "express-transport-belt", "feed:coal");
    b_in.push(BoundaryRecord { item: "coal".into(), x: 1, y: 0,
        direction: EntityDirection::South, is_fluid: false, entity: "express-transport-belt".into() });
    // Petroleum: pipe column from the north edge, ending one tile above
    // the cell's eastmost petroleum pipe (pipes connect omnidirectionally,
    // so adjacency joins them). The column x is DERIVED from the terminal
    // — a hardcoded x connected only by coincidence, and the fluid checks
    // can't catch a drifted column (corridor pipe runs are outside their
    // model; only the sim could, and it's blocked by #364).
    let pipe_terminal = cell.entities.iter()
        .filter(|e| (e.name == "pipe" || e.name == "pipe-to-ground")
            && e.segment_id.as_deref().map(|s| s.contains("petroleum")).unwrap_or(false))
        .max_by_key(|e| e.x).unwrap();
    let pt_x = pipe_terminal.x + cx;
    let pt_y = pipe_terminal.y + cy;
    assert!(pt_y > 0, "petroleum terminal must sit below the north edge");
    for y in 0..pt_y {
        entities.push(PlacedEntity { name: "pipe".into(), x: pt_x, y,
            direction: EntityDirection::North,
            segment_id: Some("feed:petroleum".into()),
            carries: Some("petroleum-gas".into()), ..Default::default() });
    }
    // Guard the adjacency the geometry relies on: the column's last tile
    // sits directly north of a petroleum-segment pipe tile of the cell.
    assert!(entities.iter().any(|e| e.x == pt_x && e.y == pt_y
            && (e.name == "pipe" || e.name == "pipe-to-ground")
            && e.segment_id.as_deref().map(|s| s.contains("petroleum")).unwrap_or(false)),
        "petroleum feed column must terminate adjacent to the cell's petroleum pipe");
    b_in.push(BoundaryRecord { item: "petroleum-gas".into(), x: pt_x, y: 0,
        direction: EntityDirection::South, is_fluid: true, entity: "pipe".into() });
    // Output: corner South to the bottom edge.
    let out = cell.ports.iter().find(|p| !p.inbound).unwrap();
    let (ox, oy) = (cx + out.x + 1, cy + out.y);
    let bottom = oy + 4;
    stamp_path(&mut entities, &[(ox, oy), (ox + 1, oy), (ox + 1, bottom)],
        "plastic-bar", "transport-belt", "out:plastic");
    b_out.push(BoundaryRecord { item: "plastic-bar".into(), x: ox + 1, y: bottom,
        direction: EntityDirection::South, is_fluid: false, entity: "transport-belt".into() });

    let width = entities.iter().map(|e| e.x).max().unwrap() + 1;
    let height = (entities.iter().map(|e| e.y).max().unwrap() + 1).max(bottom + 1);
    let comp = LayoutResult {
        entities,
        width,
        height,
        stacking: 1,
        boundary_inputs: b_in,
        boundary_outputs: b_out,
        ..Default::default()
    };

    (sr, comp)
}

/// Compose N ratio pairs stacked HORIZONTALLY with north-edge feeds and
/// south-edge drains — the sim harness's CALIBRATED boundary directions
/// (scenario.rs: only south-facing feed rigs are calibrated; the
/// east-facing attempt misassembled — #363). Per pair: three feed
/// columns at the pair's west margin drop South and corner East into
/// the ports (inner column corners first — no crossings); the EC out
/// corners South to the bottom edge.
pub fn compose_pairs_calibrated(n: i32) -> (crate::models::SolverResult, LayoutResult) {
    use crate::models::{BoundaryRecord, EntityDirection, PlacedEntity};
    let (_csr, cl) = generate_cell_layout("copper-cable", 15.0, &["copper-plate"]);
    let (esr_one, el) = generate_cell_layout("electronic-circuit", 5.0, &["iron-plate", "copper-cable"]);
    let cable = extract_cell(&cl);
    let ec = extract_cell(&el);
    let esr = if n == 1 { esr_one } else {
        let inputs: FxHashSet<String> =
            ["iron-plate", "copper-cable"].iter().map(|s| s.to_string()).collect();
        solver::solve_with_palette_exclusions_and_quality(
            "electronic-circuit", 5.0 * n as f64, &inputs, &MachinePalette::default(),
            "assembling-machine-3", &FxHashSet::default(), QualityTier::Normal,
        ).unwrap()
    };

    // Vertical geometry inside a pair (shared by all pairs): feeds enter
    // at y=0; cells start at y=FEED_MARGIN.
    const FEED_MARGIN: i32 = 2;
    // Feed columns need >=4 tiles of lateral separation: the sim kit's
    // feed rig (chest + flanking stack inserters) is ~4 wide, and
    // adjacent rigs collide at construction (observed live: only the
    // last of three 1-tile-apart rigs survived; lanes 0/1 never fed).
    const FEED_PITCH: i32 = 4;
    let cell_y = FEED_MARGIN + 1;
    let ec_y = cell_y + 3;
    let feed_w = FEED_PITCH * 2 + 1; // three columns at 0, 4, 8
    let pair_w = feed_w + 2 + cable.width + 6 /*corridor*/ + ec.width + 5 /*out+gap*/;
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut b_in: Vec<BoundaryRecord> = Vec::new();
    let mut b_out: Vec<BoundaryRecord> = Vec::new();
    let mut bottom = 0;

    for k in 0..n {
        let px = k * pair_w; // pair origin x
        let cable_x = px + feed_w + 2;
        let corridor_x = cable_x + cable.width;
        let ec_x = corridor_x + 6;

        // cells
        for e in &cable.entities {
            let mut e = e.clone();
            e.x += cable_x;
            e.y += cell_y;
            entities.push(e);
        }
        for e in &ec.entities {
            let mut e = e.clone();
            e.x += ec_x;
            e.y += ec_y;
            entities.push(e);
        }

        // Feeds: three ports, sorted by target y ASC; inner (eastmost)
        // column serves the topmost port.
        let mut targets: Vec<(String, i32, i32)> = Vec::new(); // (item, port_terminal_x, y)
        for port in cable.ports.iter().filter(|p| p.inbound) {
            targets.push((port.item.clone(), cable_x + port.x, cell_y + port.y));
        }
        let iron = ec.ports.iter().find(|p| p.inbound && p.item == "iron-plate").unwrap();
        targets.push((iron.item.clone(), ec_x + iron.x, ec_y + iron.y));
        targets.sort_by_key(|t| t.2);
        for (i, (item, tx, ty)) in targets.iter().enumerate() {
            let col_x = px + feed_w - 1 - FEED_PITCH * i as i32; // inner column first (i=0 → px+8)
            if item == "iron-plate" {
                // Corner row passes through the cable cell — UG hop
                // under it (express reach 8 == cell width, exactly).
                stamp_path(&mut entities, &[(col_x, 0), (col_x, *ty), (cable_x - 2, *ty)],
                    item, "express-transport-belt", &format!("corridor:feed:{item}:{k}:{i}"));
                for (x, io) in [(cable_x - 1, "input"), (cable_x + cable.width, "output")] {
                    entities.push(PlacedEntity {
                        name: "express-underground-belt".into(), x, y: *ty,
                        direction: EntityDirection::East,
                        io_type: Some(io.to_string()),
                        carries: Some(item.clone()),
                        segment_id: Some(format!("corridor:feed:{item}:{k}:{i}")),
                        ..Default::default()
                    });
                }
                stamp_path(&mut entities, &[(cable_x + cable.width + 1, *ty), (tx - 1, *ty)],
                    item, "express-transport-belt", &format!("corridor:feed:{item}:{k}:{i}b"));
            } else {
                stamp_path(
                    &mut entities,
                    &[(col_x, 0), (col_x, *ty), (tx - 1, *ty)],
                    item,
                    "express-transport-belt",
                    &format!("corridor:feed:{item}:{k}:{i}"),
                );
            }
            b_in.push(BoundaryRecord {
                item: item.clone(), x: col_x, y: 0,
                direction: EntityDirection::South, is_fluid: false,
                entity: "express-transport-belt".into(),
            });
        }

        // Cable corridor: identical to the validated pair geometry.
        let outs: Vec<&Port> = cable.ports.iter().filter(|p| !p.inbound).collect();
        let (o1, o2) = (outs[0], outs[1]);
        let ec_cable_port = ec.ports.iter().find(|p| p.inbound && p.item == "copper-cable").unwrap();
        let ec_cable_in_y = ec_y + ec_cable_port.y;
        let sx = corridor_x + 2;
        stamp_path(&mut entities, &[(cable_x + o1.x + 1, cell_y + o1.y), (sx - 1, cell_y + o1.y)],
            "copper-cable", "fast-transport-belt", &format!("cc:a:{k}"));
        // b-run must approach the splitter's south half EASTWARD: ending
        // the northward column at o1.y+1 facing north would sideload into
        // the a-run's tail instead of feeding the splitter (found by
        // tile-level review of the sim artifact). Column stops one tile
        // below the splitter row; a single east-facing belt there has
        // exactly one (perpendicular) input, so it's a lane-preserving
        // corner feeding the splitter — not a sideload.
        assert!(o2.y > o1.y + 1, "b-run approaches from below: {} vs {}", o2.y, o1.y);
        stamp_path(&mut entities,
            &[(cable_x + o2.x + 1, cell_y + o2.y), (sx - 1, cell_y + o2.y), (sx - 1, cell_y + o1.y + 2)],
            "copper-cable", "fast-transport-belt", &format!("cc:b:{k}"));
        entities.push(PlacedEntity {
            name: "fast-transport-belt".into(), x: sx - 1, y: cell_y + o1.y + 1,
            direction: EntityDirection::East,
            carries: Some("copper-cable".into()),
            segment_id: Some(format!("cc:b:{k}")), ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "fast-splitter".into(), x: sx, y: cell_y + o1.y,
            direction: EntityDirection::East,
            carries: Some("copper-cable".into()),
            segment_id: Some(format!("cc:m:{k}")), ..Default::default()
        });
        stamp_path(&mut entities,
            &[(sx + 1, cell_y + o1.y), (sx + 2, cell_y + o1.y), (sx + 2, ec_cable_in_y), (ec_x + ec_cable_port.x - 1, ec_cable_in_y)],
            "copper-cable", "fast-transport-belt", &format!("cc:c:{k}"));

        // Output: corner SOUTH to the bottom edge (calibrated drain dir).
        let ec_out = ec.ports.iter().find(|p| !p.inbound).unwrap();
        let ox = ec_x + ec_out.x + 1;
        let oy = ec_y + ec_out.y;
        let out_bottom = cell_y + cable.height + 4;
        stamp_path(&mut entities,
            &[(ox, oy), (ox + 1, oy), (ox + 1, out_bottom)],
            "electronic-circuit", "transport-belt", &format!("out:{k}"));
        b_out.push(BoundaryRecord {
            item: "electronic-circuit".into(), x: ox + 1, y: out_bottom,
            direction: EntityDirection::South, is_fluid: false,
            entity: "transport-belt".into(),
        });
        bottom = bottom.max(out_bottom);

        // Pole stitch along the corridor.
        for y in [cell_y, cell_y + 7, cell_y + 14] {
            entities.push(PlacedEntity {
                name: "medium-electric-pole".into(), x: sx + 4, y,
                direction: EntityDirection::North,
                segment_id: Some("pole".into()), ..Default::default()
            });
        }
    }

    let width = entities.iter().map(|e| e.x).max().unwrap() + 1;
    let height = (entities.iter().map(|e| e.y).max().unwrap() + 1).max(bottom + 1);
    // Spanning pole line (wire reach 9 > pitch 8) so per-pair pole
    // islands join one network.
    {
        let y = bottom;
        let mut x = 1;
        while x < width {
            // Nudge to the nearest free tile rather than skipping — a
            // skipped pole breaks the chain (wire reach 9, pitch 8).
            if let Some(px) = (0..=4).map(|d| x + d).find(|&cx| {
                cx < width && !entities.iter().any(|e| e.x == cx && e.y == y)
            }) {
                entities.push(PlacedEntity {
                    name: "medium-electric-pole".into(), x: px, y,
                    direction: EntityDirection::North,
                    segment_id: Some("pole".into()), ..Default::default()
                });
            }
            x += 8;
        }
    }
    // Sim-kit invariant (found live, see #363 comment): feed-rig depth
    // grows with record ORDER and each rig's chest jog runs 12–18 tiles
    // WEST at its depth row — a shallow rig's jog crosses the outward
    // column of any head west of it. Ordering records west→east makes
    // every deeper jog pass below all shorter columns.
    b_in.sort_by_key(|r| r.x);
    let l = LayoutResult {
        entities,
        width,
        height,
        stacking: 1,
        boundary_inputs: b_in,
        boundary_outputs: b_out,
        ..Default::default()
    };
    (esr, l)
}
