//! Whole-map per-lane occupancy heatmap after warmup — the instrument for
//! "which lane is actually full" questions that trace_belt (boundary-feed
//! walker) cannot answer for internally produced items.
//!
//! ```bash
//! cargo run --release -p spaghettio_meter --example lane_heatmap -- <label> [warmup_ticks]
//! ```
//! Reads `crates/core/target/tmp/<label>.bp` + `<label>.manifest.json`.
//! Grid legend: belts print two hex digits (lane0, lane1 occupancy 0-4);
//! machines print the first two letters of their recipe; inserters `i↕`;
//! splitters `SS`; empty `··`.

use std::path::PathBuf;

use spaghettio_meter::factory::Endpoint;
use spaghettio_meter::network::TileKind;
use spaghettio_meter::{Factory, Manifest};

fn main() {
    let mut args = std::env::args().skip(1);
    let label = args.next().expect("label");
    let warmup: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(60 * 60 * 3);

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("blueprint");
    let manifest =
        Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("manifest");
    let mut f = Factory::build(&bp, manifest).expect("build");
    f.run_for(warmup);

    let mut cells: std::collections::BTreeMap<(i32, i32), String> = Default::default();
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    let mut touch = |p: (i32, i32)| {
        min_x = min_x.min(p.0);
        max_x = max_x.max(p.0);
        min_y = min_y.min(p.1);
        max_y = max_y.max(p.1);
    };

    for m in &f.machines {
        let tag: String = m.recipe.chars().take(2).collect();
        for dy in 0..m.size.1 as i32 {
            for dx in 0..m.size.0 as i32 {
                let p = (m.pos.0 + dx, m.pos.1 + dy);
                touch(p);
                cells.insert(p, tag.clone());
            }
        }
    }
    for t in &f.net.tiles {
        touch(t.pos);
        let s = match t.kind {
            TileKind::Splitter { .. } => "SS".to_string(),
            _ => format!(
                "{:x}{:x}",
                t.lanes[0].occupancy().min(15),
                t.lanes[1].occupancy().min(15)
            ),
        };
        cells.insert(t.pos, s);
    }
    for w in &f.inserters {
        touch(w.pos);
        cells.insert(w.pos, "i.".to_string());
    }

    println!("{label} after {warmup} ticks — grid ({min_x},{min_y})..({max_x},{max_y})");
    for y in min_y..=max_y {
        let mut row = String::new();
        for x in min_x..=max_x {
            row.push_str(cells.get(&(x, y)).map(String::as_str).unwrap_or("··"));
        }
        println!("{y:>3} {row}");
    }

    println!("\nboundary feeds:");
    for fd in &f.feeds {
        println!(
            "  {:?} {} injected={} refused={}",
            fd.pos,
            f.items.name(fd.item),
            fd.injected,
            fd.refused
        );
    }

    // Splitter routing counters over a post-warmup window: where flow
    // throttles, both_blocked climbs; an idle input shows low attempts.
    f.net.reset_splitter_stats();
    f.run_for(60 * 60);
    println!("\nsplitter stats over a 3600-tick window (per input lane):");
    for (i, t) in f.net.tiles.iter().enumerate() {
        if let TileKind::Splitter { id, .. } = t.kind {
            let s = &f.net.splitter_stats[id];
            // Print once per splitter, from its id-carrying tile only.
            let _ = i;
            println!(
                "  sid={id} {:?} {:?} attempts={:?} first_acc={:?} fallback_acc={:?} both_blocked={:?}",
                t.pos, t.dir, s.attempts, s.first_accepted, s.fallback_accepted, s.both_blocked
            );
        }
    }

    // Optional focused path dump: RECT=x0,y0,x1,y1 prints every network
    // tile in the rectangle with dir, kind, downstream lane mapping, and
    // occupancies — the "where does lane 1 dead-end" instrument.
    if let Ok(rect) = std::env::var("RECT") {
        let v: Vec<i32> = rect.split(',').filter_map(|s| s.parse().ok()).collect();
        if let [x0, y0, x1, y1] = v[..] {
            println!("\ntiles in ({x0},{y0})..({x1},{y1}):");
            let mut tiles: Vec<&spaghettio_meter::network::BeltTile> = f
                .net
                .tiles
                .iter()
                .filter(|t| t.pos.0 >= x0 && t.pos.0 <= x1 && t.pos.1 >= y0 && t.pos.1 <= y1)
                .collect();
            tiles.sort_by_key(|t| (t.pos.1, t.pos.0));
            for t in tiles {
                let lanemap = match t.downstream.map(|d| d.lanes) {
                    Some(spaghettio_meter::network::LaneMap::Straight) => "straight".to_string(),
                    Some(spaghettio_meter::network::LaneMap::OntoLane(l)) => {
                        format!("SIDELOAD->lane{l}")
                    }
                    None => "NO-DOWNSTREAM".to_string(),
                };
                println!(
                    "  {:?} {:?} {:?} L0={} L1={} {} sink={}",
                    t.pos,
                    t.dir,
                    t.kind,
                    t.lanes[0].occupancy(),
                    t.lanes[1].occupancy(),
                    lanemap,
                    t.is_sink,
                );
            }
        }
    }

    // Belt tiles that inserters drop onto (producer output runs), with lanes.
    println!("\nbelt tiles receiving inserter drops (pos, dir, L0, L1):");
    let mut drops: Vec<usize> = f
        .inserters
        .iter()
        .filter_map(|w| match w.drop {
            Endpoint::Belt(t) => Some(t),
            _ => None,
        })
        .collect();
    drops.sort_unstable();
    drops.dedup();
    for t in drops {
        let tile = &f.net.tiles[t];
        println!(
            "  {:?} {:?} L0={} L1={}",
            tile.pos,
            tile.dir,
            tile.lanes[0].occupancy(),
            tile.lanes[1].occupancy()
        );
    }
}
