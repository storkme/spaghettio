//! Per-machine attribution for one fixture — the meter's own
//! `sim-capture-state.sh`.
//!
//! ```bash
//! cargo run --release -p spaghettio_meter --example attribute -- chain-mil5plates-d0
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;

use spaghettio_meter::factory::Endpoint;
use spaghettio_meter::{Factory, Manifest, MachineState};

fn main() {
    let label = std::env::args().nth(1).unwrap_or_else(|| "chain-mil5plates-d0".into());
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("blueprint");
    let manifest = Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("manifest");
    let mut f = Factory::build(&bp, manifest).expect("build");

    f.run_for(60 * 60 * 2);
    f.reset_counters();
    f.run_for(60 * 60 * 3);

    // --- per-recipe census + what the starved ones are missing ----------
    let mut by_recipe: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    for m in &f.machines {
        let e = by_recipe.entry(m.recipe.clone()).or_insert((0, 0, 0, 0));
        e.0 += 1;
        match m.state {
            MachineState::Working => e.1 += 1,
            MachineState::FullOutput => e.2 += 1,
            MachineState::ItemIngredientShortage => e.3 += 1,
        }
    }
    println!("{:<28} {:>5} {:>8} {:>7} {:>9}", "recipe", "n", "working", "full", "starved");
    for (recipe, (n, w, full, starved)) in &by_recipe {
        println!("{recipe:<28} {n:>5} {w:>8} {full:>7} {starved:>9}");
    }

    println!("\nstarved machines — what they hold vs what they need:");
    let mut shown = 0;
    for m in &f.machines {
        if m.state != MachineState::ItemIngredientShortage || shown >= 12 {
            continue;
        }
        shown += 1;
        let need: Vec<String> = m
            .ingredients
            .iter()
            .map(|(id, amt)| {
                let have = m.input.get(&id.0).copied().unwrap_or(0);
                format!("{}={have}/{amt}", f.items.name(*id))
            })
            .collect();
        println!("  {:<24} at {:?}  {}", m.recipe, m.pos, need.join(" "));
    }

    // --- inserter wiring census ----------------------------------------
    let mut nothing_pick = 0;
    let mut nothing_drop = 0;
    let mut belt_to_machine = 0;
    let mut machine_to_belt = 0;
    let mut machine_to_machine = 0;
    let mut belt_to_belt = 0;
    for w in &f.inserters {
        match (w.pickup, w.drop) {
            (Endpoint::Nothing, _) => nothing_pick += 1,
            (_, Endpoint::Nothing) => nothing_drop += 1,
            (Endpoint::Belt(_), Endpoint::Machine(_)) => belt_to_machine += 1,
            (Endpoint::Machine(_), Endpoint::Belt(_)) => machine_to_belt += 1,
            (Endpoint::Machine(_), Endpoint::Machine(_)) => machine_to_machine += 1,
            (Endpoint::Belt(_), Endpoint::Belt(_)) => belt_to_belt += 1,
        }
    }
    println!(
        "\ninserters: belt->machine {belt_to_machine}, machine->belt {machine_to_belt}, \
         machine->machine {machine_to_machine}, belt->belt {belt_to_belt}, \
         pickup-nothing {nothing_pick}, drop-nothing {nothing_drop}"
    );

    // --- starved inserters ----------------------------------------------
    let mut starving: Vec<(&str, u64)> = Vec::new();
    for w in &f.inserters {
        if w.core.starved_ticks > 0 {
            starving.push(("", w.core.starved_ticks));
        }
    }
    println!(
        "inserters that starved at all: {}/{}",
        starving.len(),
        f.inserters.len()
    );

    println!("\nboundary feeds:");
    for feed in &f.feeds {
        println!(
            "  {:<14} at {:?}: injected {:.2}/s  (belt full {:.0}% of ticks)",
            f.items.name(feed.item),
            feed.pos,
            feed.injected as f64 / (f.ticks as f64 / 60.0),
            feed.refused as f64 / feed.offered.max(1) as f64 * 100.0
        );
    }
    for n in f.notes.iter().take(10) {
        println!("  note: {n}");
    }
}
