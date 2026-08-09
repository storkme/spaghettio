//! #609 pre-RFC: of `belt_flow`'s over-cap tiles, how many are physically
//! impossible artifacts vs plausibly-real over-capacity?
//!
//! Discriminator needs no judgement: sum the total flow of an item that exists
//! in the factory at all — external imports plus machine production. A belt
//! LANE predicted to carry more than that is an artifact by conservation, full
//! stop. Anything at or below it is "plausible" and would need triage.
//!
//! Determines the shape of the arbitration RFC: mostly-artifact means fixing
//! the cycle bug shrinks the blast radius to something small; mostly-plausible
//! means thousands of new Errors need staged rollout.
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::common::{
    is_splitter, is_surface_belt, is_ug_belt, lane_capacity_stacked, splitter_second_tile,
    ug_to_surface_tier,
};
use spaghettio_core::{bus::layout, solver};
use std::collections::BTreeMap;

struct C(&'static str, &'static str, &'static [f64], &'static [&'static str], &'static [&'static str]);

fn main() {
    let cases = [
        C("gear", "iron-gear-wheel", &[5.0, 10.0, 20.0, 45.0], &["assembling-machine-1", "assembling-machine-2", "assembling-machine-3"], &["iron-plate"]),
        C("gear_ore", "iron-gear-wheel", &[10.0, 20.0], &["assembling-machine-2"], &["iron-ore"]),
        C("ec", "electronic-circuit", &[5.0, 10.0, 20.0, 30.0, 60.0], &["assembling-machine-1", "assembling-machine-2", "assembling-machine-3"], &["iron-plate", "copper-plate"]),
        C("ec_ore", "electronic-circuit", &[10.0, 20.0, 30.0], &["assembling-machine-1", "assembling-machine-2"], &["iron-ore", "copper-ore"]),
        C("cable", "copper-cable", &[15.0, 45.0], &["assembling-machine-2"], &["copper-plate"]),
        C("plastic", "plastic-bar", &[10.0, 20.0], &["chemical-plant"], &["petroleum-gas", "coal"]),
        C("ac", "advanced-circuit", &[1.0, 3.0, 5.0], &["assembling-machine-2", "assembling-machine-3"], &["iron-plate", "copper-plate", "coal", "crude-oil", "water"]),
        C("pu", "processing-unit", &[1.0, 2.0], &["assembling-machine-3"], &["iron-plate", "copper-plate", "coal", "crude-oil", "water", "sulfur"]),
        C("belt", "transport-belt", &[5.0, 15.0], &["assembling-machine-2"], &["iron-plate"]),
        C("inserter", "inserter", &[2.0, 8.0], &["assembling-machine-2"], &["iron-plate", "copper-plate"]),
        C("steel", "steel-plate", &[5.0], &["electric-furnace"], &["iron-plate"]),
        C("sci1", "automation-science-pack", &[2.0, 5.0], &["assembling-machine-2"], &["iron-plate", "copper-plate"]),
        C("sci2", "logistic-science-pack", &[2.0, 5.0], &["assembling-machine-2"], &["iron-plate", "copper-plate"]),
    ];
    let belt_tiers: [Option<&str>; 3] =
        [None, Some("fast-transport-belt"), Some("express-transport-belt")];

    let mut layouts_with_overcap = 0usize;
    let mut overcap_tiles = 0usize;
    let mut impossible = 0usize;
    let mut plausible = 0usize;
    let mut untagged = 0usize;
    let mut worst_ratio = 0.0f64;
    let mut worst = String::new();
    let mut plausible_examples: Vec<String> = Vec::new();
    let mut impossible_by_seg: BTreeMap<String, usize> = BTreeMap::new();
    let mut plausible_by_seg: BTreeMap<String, usize> = BTreeMap::new();

    for C(name, item, rates, machines, inputs) in cases {
        let set: rustc_hash::FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        for &rate in rates {
            for machine in machines {
                let Ok(sr) = solver::solve(item, rate, &set, machine) else { continue };

                // Total items/s of each item that exists in this factory at
                // all: imports + machine production. A lane cannot exceed it.
                let mut item_ceiling: BTreeMap<String, f64> = BTreeMap::new();
                for f in &sr.external_inputs {
                    if !f.is_fluid {
                        *item_ceiling.entry(f.item.clone()).or_default() += f.rate;
                    }
                }
                for m in &sr.machines {
                    for o in &m.outputs {
                        if !o.is_fluid {
                            *item_ceiling.entry(o.item.clone()).or_default() +=
                                o.rate * m.count.ceil();
                        }
                    }
                }

                for belt in belt_tiers {
                    for di in [DirectInsertion::Off, DirectInsertion::Forced, DirectInsertion::Candidate] {
                        let opts = layout::LayoutOptions {
                            direct_insertion: di,
                            max_belt_tier: belt.map(|s| s.to_string()),
                            ..Default::default()
                        };
                        let Ok(l) = layout::build_bus_layout(&sr, opts) else { continue };

                        let flow = spaghettio_core::validate::belt_flow::compute_lane_rates(&l, Some(&sr));
                        if flow.is_empty() {
                            continue;
                        }
                        let ctx = spaghettio_core::bus::stacking_ctx::StackingCtx::derive(&sr, l.stacking);

                        // tile -> (belt tier name, carried item, segment head)
                        let mut meta: BTreeMap<(i32, i32), (String, Option<String>, String)> =
                            BTreeMap::new();
                        for e in &l.entities {
                            let tier = if is_surface_belt(&e.name) {
                                Some(e.name.clone())
                            } else if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("output") {
                                Some(ug_to_surface_tier(&e.name).to_string())
                            } else if is_splitter(&e.name) {
                                Some(spaghettio_core::common::splitter_to_surface_tier(&e.name).to_string())
                            } else {
                                None
                            };
                            let Some(tier) = tier else { continue };
                            let seg = e
                                .segment_id
                                .as_deref()
                                .and_then(|s| s.split(':').next())
                                .unwrap_or("-")
                                .to_string();
                            meta.insert((e.x, e.y), (tier.clone(), e.carries.clone(), seg.clone()));
                            if is_splitter(&e.name) {
                                meta.insert(splitter_second_tile(e), (tier, e.carries.clone(), seg));
                            }
                        }

                        let mut any = false;
                        for (&pos, &lanes) in &flow {
                            let (tier, carries, seg) = match meta.get(&pos) {
                                Some(m) => m.clone(),
                                None => ("transport-belt".to_string(), None, "-".to_string()),
                            };
                            let stacking = carries
                                .as_deref()
                                .map(|i| ctx.for_item(i))
                                .unwrap_or(l.stacking);
                            let cap = lane_capacity_stacked(&tier, stacking);
                            for lane in lanes {
                                if lane <= cap + 0.01 {
                                    continue;
                                }
                                overcap_tiles += 1;
                                any = true;
                                let Some(carried) = carries.as_deref() else {
                                    untagged += 1;
                                    continue;
                                };
                                let ceiling = item_ceiling.get(carried).copied().unwrap_or(0.0);
                                if ceiling <= 0.0 {
                                    untagged += 1;
                                    continue;
                                }
                                let ratio = lane / ceiling;
                                if ratio > 1.0 {
                                    impossible += 1;
                                    *impossible_by_seg.entry(seg.clone()).or_default() += 1;
                                    if ratio > worst_ratio {
                                        worst_ratio = ratio;
                                        worst = format!(
                                            "{name}@{rate}/{machine}/{}/{di:?} {pos:?} seg={seg} {carried} lane={lane:.1}/s vs factory total {ceiling:.1}/s = {ratio:.0}x",
                                            belt.unwrap_or("yellow")
                                        );
                                    }
                                } else {
                                    plausible += 1;
                                    *plausible_by_seg.entry(seg.clone()).or_default() += 1;
                                    if plausible_examples.len() < 20 {
                                        plausible_examples.push(format!(
                                            "{name}@{rate}/{machine}/{}/{di:?} {pos:?} seg={seg} {carried} lane={lane:.2}/s cap={cap:.2} (factory total {ceiling:.1}/s)",
                                            belt.unwrap_or("yellow")
                                        ));
                                    }
                                }
                            }
                        }
                        if any {
                            layouts_with_overcap += 1;
                        }
                    }
                }
            }
        }
    }

    println!("\n===== belt_flow over-cap triage =====");
    println!("  layouts with >=1 over-cap tile: {layouts_with_overcap}");
    println!("  over-cap lane readings total:   {overcap_tiles}");
    println!("  PHYSICALLY IMPOSSIBLE (lane > whole factory's flow of that item): {impossible}");
    println!("  plausible (over cap, under factory total):                       {plausible}");
    println!("  unclassifiable (untagged carries / no ceiling):                  {untagged}");
    if overcap_tiles > 0 {
        println!(
            "\n  => {:.1}% of over-cap readings are impossible artifacts",
            100.0 * impossible as f64 / overcap_tiles as f64
        );
    }
    println!("\n  worst: {worst}");
    println!("\n--- impossible, by segment kind ---");
    for (s, n) in &impossible_by_seg {
        println!("  {s:<14} {n:>6}");
    }
    println!("\n--- plausible, by segment kind ---");
    for (s, n) in &plausible_by_seg {
        println!("  {s:<14} {n:>6}");
    }
    println!("\n--- plausible examples (these would need triage) ---");
    for e in &plausible_examples {
        println!("  {e}");
    }
}
