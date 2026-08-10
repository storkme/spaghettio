//! Where does copper-cable sit along its belt, and who can reach it?
//!
//! Usage: `cable_probe [label] [warmup_ticks] [measure_ticks]`
//!
//! Defaults match `check_one.rs`'s calibrated window (108k warmup / 216k
//! measure). The short windows this probe originally shipped with (7.2k/10.8k)
//! read buffer fill as throughput on multi-stage chains — the same trap
//! `CLAUDE.md` flags for the sim harness's dim-scaled default warmup. Override
//! only when you want a deliberate cold-start capture, and say so if you quote
//! the numbers.
use spaghettio_meter::factory::Endpoint;
use spaghettio_meter::{Factory, Manifest};
use std::path::PathBuf;

const ITEM: &str = "copper-cable";

fn main() {
    let mut args = std::env::args().skip(1);
    let label = args.next().unwrap_or_else(|| "tier2-ec10-lift".into());
    let warmup: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(108_000);
    let measure: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(216_000);

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("bp");
    let m = Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("m");
    let mut f = Factory::build(&bp, m).expect("build");

    f.run_for(warmup);
    f.reset_counters();
    // `reset_counters` clears the factory-level maps, `machines[].crafts` and
    // `feeds[].*`, but NOT the per-inserter counters — `Inserter::delivered`
    // and `starved_ticks` are documented as totals "over the run" and keep
    // accumulating across the warmup. Snapshot them here and subtract below,
    // or every printed `starved=` carries the cold-start transient during
    // which belts are still filling and every pickup inserter legitimately
    // starves. That would invert the exact signal this probe exists to read.
    let base: Vec<(u64, u64)> = f
        .inserters
        .iter()
        .map(|w| (w.core.delivered, w.core.starved_ticks))
        .collect();

    f.run_for(measure);

    println!("window: {warmup} warmup + {measure} measured ticks");

    // machines by recipe, with crafts
    println!("\nmachines (recipe, pos, state, crafts over window):");
    let mut ms: Vec<_> = f.machines.iter().collect();
    ms.sort_by_key(|m| (m.recipe.clone(), m.pos.0, m.pos.1));
    for mc in &ms {
        println!("  {:<20} at {:?}  {:?}  crafts={}", mc.recipe, mc.pos, mc.state, mc.crafts);
    }

    // Tiles holding the item at end of window. Used both to report occupancy
    // and to decide which inserters are worth printing. Caveat: a belt that is
    // genuinely empty at this instant drops out, so this under-selects rather
    // than over-selects — fine for "who can reach it", wrong for a census.
    let item_tiles: Vec<usize> = f
        .net
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            t.lanes.iter().any(|lane| {
                lane.slots_debug().iter().flatten().any(|it| f.items.name(*it) == ITEM)
            })
        })
        .map(|(i, _)| i)
        .collect();

    let touches_item = |e: &Endpoint| match e {
        Endpoint::Belt(t) => item_tiles.contains(t),
        _ => false,
    };

    println!(
        "\n{ITEM} belt<->belt inserters (machine->belt = producer tap, belt->machine = consumer tap):"
    );
    println!("  counters are window-scoped (warmup subtracted)");
    for (w, (d0, s0)) in f.inserters.iter().zip(&base) {
        let belt_to_belt =
            matches!(w.pickup, Endpoint::Belt(_)) && matches!(w.drop, Endpoint::Belt(_));
        if !belt_to_belt || !(touches_item(&w.pickup) || touches_item(&w.drop)) {
            continue;
        }
        let ep = |e: &Endpoint| match e {
            Endpoint::Belt(t) => format!("belt{:?}", f.net.tiles[*t].pos),
            Endpoint::Machine(i) => {
                format!("mach {}{:?}", f.machines[*i].recipe, f.machines[*i].pos)
            }
            Endpoint::Nothing => "-".into(),
        };
        println!(
            "  {:<12} {:?} {} -> {}  delivered={} starved={}",
            format!("{:?}", w.core.kind),
            w.pos,
            ep(&w.pickup),
            ep(&w.drop),
            w.core.delivered.saturating_sub(*d0),
            w.core.starved_ticks.saturating_sub(*s0),
        );
    }

    // Per-lane occupancy along each row that carries the item. Rows are
    // derived, not hardcoded — the original probe pinned y=11, which was only
    // ever right for the fixture it was written against.
    let mut rows: Vec<i32> = item_tiles.iter().map(|&i| f.net.tiles[i].pos.1).collect();
    rows.sort_unstable();
    rows.dedup();
    for row in rows {
        println!("\n=== y={row}, PER-LANE slot occupancy (whole row, all items) ===");
        let mut in_row: Vec<_> = f.net.tiles.iter().filter(|t| t.pos.1 == row).collect();
        in_row.sort_by_key(|t| t.pos.0);
        for t in in_row {
            let lanes: Vec<String> = t
                .lanes
                .iter()
                .map(|l| {
                    let slots = l.slots_debug();
                    let filled = slots.iter().filter(|s| s.is_some()).count();
                    format!("{filled}/{}", slots.len())
                })
                .collect();
            println!("  x={:<3} lanes {:?}", t.pos.0, lanes);
        }
    }

    println!("\nbelt tiles carrying {ITEM} (occ/8):");
    let mut sorted = item_tiles.clone();
    sorted.sort_by_key(|&i| (f.net.tiles[i].pos.1, f.net.tiles[i].pos.0));
    for i in sorted {
        let t = &f.net.tiles[i];
        let mut names: Vec<String> = Vec::new();
        for lane in &t.lanes {
            for it in lane.slots_debug().iter().flatten() {
                let n = f.items.name(*it).to_string();
                if !names.contains(&n) {
                    names.push(n);
                }
            }
        }
        println!("  {:?} occ {}/8  {:?}", t.pos, t.occupancy(), names);
    }
}
