//! Walk a belt path downstream from a boundary feed, dumping per-tile
//! lane occupancy — the meter's tile-level debugger.
//!
//! ```bash
//! cargo run --release -p spaghettio_meter --example trace_belt -- chain-mil5plates-d0 coal
//! ```
//!
//! Reads the same fixtures as `measure`/`attribute`. Answers "where does
//! this item stop flowing", which is the question a head-full/tail-starved
//! gradient always poses.

use std::path::PathBuf;

use spaghettio_meter::network::{LaneMap, TileKind};
use spaghettio_meter::{Factory, Manifest};

fn main() {
    let mut args = std::env::args().skip(1);
    let label = args.next().unwrap_or_else(|| "chain-mil5plates-d0".into());
    let item_name = args.next().unwrap_or_else(|| "coal".into());

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("blueprint");
    let manifest =
        Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("manifest");
    let mut f = Factory::build(&bp, manifest).expect("build");
    f.run_for(60 * 60 * 3);

    let Some(feed) = f
        .feeds
        .iter()
        .find(|fd| f.items.name(fd.item) == item_name)
        .map(|fd| (fd.tile, fd.pos))
    else {
        eprintln!("no boundary feed for {item_name}");
        return;
    };

    println!("tracing {item_name} from {:?}\n", feed.1);
    println!(
        "{:>4}  {:<10} {:<6} {:<8} {:>5} {:>5}  {:<9} note",
        "step", "pos", "dir", "kind", "L0", "L1", "lanemap"
    );

    // Inserters that pick from each tile, so drop-offs along the path show up.
    let mut pickers: rustc_hash::FxHashMap<usize, usize> = Default::default();
    for w in &f.inserters {
        if let spaghettio_meter::factory::Endpoint::Belt(t) = w.pickup {
            *pickers.entry(t).or_insert(0) += 1;
        }
    }
    let mut droppers: rustc_hash::FxHashMap<usize, usize> = Default::default();
    for w in &f.inserters {
        if let spaghettio_meter::factory::Endpoint::Belt(t) = w.drop {
            *droppers.entry(t).or_insert(0) += 1;
        }
    }

    let mut tile = feed.0;
    let mut seen = rustc_hash::FxHashSet::default();
    for step in 0..400 {
        if !seen.insert(tile) {
            println!("  (loop back to a visited tile — stopping)");
            break;
        }
        let t = &f.net.tiles[tile];
        let kind = match t.kind {
            TileKind::Belt => "belt",
            TileKind::UgInput => "ug-in",
            TileKind::UgOutput => "ug-out",
            TileKind::Splitter { .. } => "splitter",
        };
        let lanemap = match t.downstream.map(|d| d.lanes) {
            Some(LaneMap::Straight) => "straight".to_string(),
            Some(LaneMap::OntoLane(l)) => format!("sideload{l}"),
            None => "-".to_string(),
        };
        let mut note = String::new();
        if let Some(n) = pickers.get(&tile) {
            note.push_str(&format!("{n} picker(s) "));
        }
        if let Some(n) = droppers.get(&tile) {
            note.push_str(&format!("{n} dropper(s) "));
        }
        if t.is_sink {
            note.push_str("SINK ");
        }
        println!(
            "{step:>4}  {:<10} {:<6} {:<8} {:>5} {:>5}  {:<9} {note}",
            format!("{:?}", t.pos),
            format!("{:?}", t.dir),
            kind,
            t.lanes[0].occupancy(),
            t.lanes[1].occupancy(),
            lanemap,
        );
        match t.downstream {
            Some(d) => tile = d.tile,
            None => {
                println!("  (no downstream — end of path)");
                break;
            }
        }
    }
}
