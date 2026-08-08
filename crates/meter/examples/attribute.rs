//! Per-machine attribution for one fixture — the meter's own
//! `sim-capture-state.sh`.
//!
//! ```bash
//! cargo run --release -p spaghettio_meter --example attribute -- chain-mil5plates-d0
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;

use spaghettio_meter::belt::ItemId;
use spaghettio_meter::factory::Endpoint;
use spaghettio_meter::{Factory, MachineState, Manifest};

fn main() {
    let label = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "chain-mil5plates-d0".into());
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("blueprint");
    let manifest =
        Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("manifest");
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
            MachineState::ItemIngredientShortage | MachineState::FluidIngredientShortage => {
                e.3 += 1
            }
        }
    }
    println!(
        "{:<28} {:>5} {:>8} {:>7} {:>9}",
        "recipe", "n", "working", "full", "starved"
    );
    for (recipe, (n, w, full, starved)) in &by_recipe {
        println!("{recipe:<28} {n:>5} {w:>8} {full:>7} {starved:>9}");
    }

    // Both shortage states, and each row says which one it is. Reporting only
    // `ItemIngredientShortage` here meant a fluid-starved stage showed a
    // non-zero `starved` count in the census above and then never appeared in
    // this list — so the reader inferred "no solid shortage" from an absence
    // and had to reach the fluid explanation by elimination. That is the
    // failure mode `docs/validator-reporting.md` is about: the probe must emit
    // a positive signal for the case it is meant to find. Found 2026-08-08
    // while attributing pu1-lift's −24.2pp, whose whole chain hangs off two
    // petroleum-starved plastic-bar machines that this list did not print.
    // Per-KIND caps, not one shared budget. A shared cap of 12 lets a fixture
    // with 12+ solid-starved machines push every fluid-starved one off the
    // list — recreating, through the back door, the exact "absence reads as no
    // shortage" failure this block was changed to fix.
    const PER_KIND_CAP: usize = 12;
    println!("\nstarved machines — what they hold vs what they need:");
    let (mut shown_item, mut shown_fluid) = (0usize, 0usize);
    let (mut hidden_item, mut hidden_fluid) = (0usize, 0usize);
    for m in &f.machines {
        let kind = match m.state {
            MachineState::ItemIngredientShortage => "item",
            MachineState::FluidIngredientShortage => "fluid",
            _ => continue,
        };
        let over = if kind == "item" {
            shown_item += 1;
            if shown_item > PER_KIND_CAP {
                hidden_item += 1;
                true
            } else {
                false
            }
        } else {
            shown_fluid += 1;
            if shown_fluid > PER_KIND_CAP {
                hidden_fluid += 1;
                true
            } else {
                false
            }
        };
        if over {
            continue;
        }
        let mut need: Vec<String> = m
            .ingredients
            .iter()
            .map(|(id, amt)| {
                let have = m.input.get(&id.0).copied().unwrap_or(0);
                format!("{}={have}/{amt}", f.items.name(*id))
            })
            .collect();
        need.extend(m.fluid_needs.iter().map(|(id, amt)| {
            let have = m.fluid_input.get(id).copied().unwrap_or(0);
            format!("{}={have}/{amt} (fluid)", f.items.name(ItemId(*id)))
        }));
        println!(
            "  [{kind:>5}] {:<24} at {:?}  {}",
            m.recipe,
            m.pos,
            need.join(" ")
        );
    }
    // Truncation is stated, per kind. A capped list that does not say it was
    // capped is a list the reader will treat as exhaustive.
    if hidden_item > 0 || hidden_fluid > 0 {
        println!(
            "  ... {hidden_item} more item-starved and {hidden_fluid} more fluid-starved machine(s) not shown (cap {PER_KIND_CAP}/kind)"
        );
    }

    // --- what feeds the starved machines ---------------------------------
    println!("\ninput inserters of starved machines — pickup tile contents:");
    let mut shown2 = 0;
    for (mi, m) in f.machines.iter().enumerate() {
        if m.state != MachineState::ItemIngredientShortage || shown2 >= 6 {
            continue;
        }
        shown2 += 1;
        println!("  {} at {:?}:", m.recipe, m.pos);
        for w in &f.inserters {
            if w.drop != Endpoint::Machine(mi) {
                continue;
            }
            let what = match w.pickup {
                Endpoint::Belt(t) => {
                    let tile = &f.net.tiles[t];
                    let mut items: Vec<String> = Vec::new();
                    for lane in &tile.lanes {
                        for it in lane.slots_debug().iter().flatten() {
                            let n = f.items.name(*it).to_string();
                            if !items.contains(&n) {
                                items.push(n);
                            }
                        }
                    }
                    format!(
                        "belt {:?} occ {}/{} carrying {:?}",
                        tile.pos,
                        tile.occupancy(),
                        8,
                        items
                    )
                }
                Endpoint::Machine(o) => format!(
                    "machine {} at {:?}",
                    f.machines[o].recipe, f.machines[o].pos
                ),
                Endpoint::Nothing => "NOTHING".to_string(),
            };
            println!(
                "    {:?} inserter at {:?} <- {what}  (starved {} ticks, delivered {})",
                w.core.kind, w.pos, w.core.starved_ticks, w.core.delivered
            );
        }
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

    // Which belt tiles have NO upstream feeder at all?
    let mut has_upstream = vec![false; f.net.tiles.len()];
    for t in &f.net.tiles {
        if let Some(d) = t.downstream {
            has_upstream[d.tile] = true;
        }
    }
    // A tile is also fed if an inserter drops onto it.
    for w in &f.inserters {
        if let Endpoint::Belt(t) = w.drop {
            has_upstream[t] = true;
        }
    }
    for fd in &f.feeds {
        has_upstream[fd.tile] = true;
    }
    let orphans: Vec<(i32, i32)> = f
        .net
        .tiles
        .iter()
        .enumerate()
        .filter(|(i, _)| !has_upstream[*i])
        .map(|(_, t)| t.pos)
        .collect();
    println!(
        "\nbelt tiles with NO upstream feeder: {}/{}",
        orphans.len(),
        f.net.tiles.len()
    );
    for p in orphans.iter().take(12) {
        println!("    orphan head {p:?}");
    }

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
