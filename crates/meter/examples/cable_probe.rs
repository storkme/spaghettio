//! Where does copper-cable sit along its belt, and who can reach it?
use std::path::PathBuf;
use spaghettio_meter::factory::Endpoint;
use spaghettio_meter::{Factory, Manifest, MachineState};

fn main() {
    let label = std::env::args().nth(1).unwrap_or_else(|| "tier2-ec10-lift".into());
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("bp");
    let m = Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("m");
    let mut f = Factory::build(&bp, m).expect("build");
    f.run_for(60 * 60 * 2);
    f.reset_counters();
    f.run_for(60 * 60 * 3);

    // machines by recipe, with crafts
    println!("machines (recipe, pos, state, crafts over window):");
    let mut ms: Vec<_> = f.machines.iter().collect();
    ms.sort_by_key(|m| (m.recipe.clone(), m.pos.0, m.pos.1));
    for mc in &ms {
        println!("  {:<20} at {:?}  {:?}  crafts={}", mc.recipe, mc.pos, mc.state, mc.crafts);
    }

    // who drops cable onto which tile, who picks it up
    println!("\ncable inserters (machine->belt = producer tap, belt->machine = consumer tap):");
    for w in &f.inserters {
        let p = match w.pickup { Endpoint::Belt(t) => format!("belt{:?}", f.net.tiles[t].pos),
                                 Endpoint::Machine(i) => format!("mach {}{:?}", f.machines[i].recipe, f.machines[i].pos),
                                 Endpoint::Nothing => "-".into() };
        let d = match w.drop { Endpoint::Belt(t) => format!("belt{:?}", f.net.tiles[t].pos),
                               Endpoint::Machine(i) => format!("mach {}{:?}", f.machines[i].recipe, f.machines[i].pos),
                               Endpoint::Nothing => "-".into() };
        let belt_to_belt = matches!(w.pickup, Endpoint::Belt(_)) && matches!(w.drop, Endpoint::Belt(_));
        if belt_to_belt {
            println!("  {:<12} {:?} {p} -> {d}  delivered={} starved={}",
                format!("{:?}", w.core.kind), w.pos, w.core.delivered, w.core.starved_ticks);
        }
    }

    // occupancy of every tile carrying cable
    println!("\n=== y=11 drop belt, PER-LANE slot occupancy ===");
    let mut t11: Vec<_> = f.net.tiles.iter().filter(|t| t.pos.1 == 11).collect();
    t11.sort_by_key(|t| t.pos.0);
    for t in t11 {
        let lanes: Vec<String> = t.lanes.iter().map(|l| {
            let slots = l.slots_debug();
            let filled = slots.iter().filter(|s| s.is_some()).count();
            format!("{filled}/{}", slots.len())
        }).collect();
        println!("  x={:<3} lanes {:?}", t.pos.0, lanes);
    }
    println!("\nbelt tiles carrying copper-cable (occ/8):");
    let mut tiles: Vec<_> = f.net.tiles.iter().enumerate().collect();
    tiles.sort_by_key(|(_, t)| (t.pos.1, t.pos.0));
    for (_i, t) in tiles {
        let mut names: Vec<String> = Vec::new();
        for lane in &t.lanes {
            for it in lane.slots_debug().iter().flatten() {
                let n = f.items.name(*it).to_string();
                if !names.contains(&n) { names.push(n); }
            }
        }
        if names.iter().any(|n| n == "copper-cable") {
            println!("  {:?} occ {}/8  {:?}", t.pos, t.occupancy(), names);
        }
    }
    let _ = MachineState::Working;
}
