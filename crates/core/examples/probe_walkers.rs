//! #609 / handoff §2d: how far apart are the two `compute_lane_rates`?
//!
//! `validate/mod.rs` dispatches `belt_structural::compute_lane_rates`;
//! `bus/template_validate.rs` uses `belt_flow::compute_lane_rates`. Both are
//! independent Python→Rust ports of the same walk. `belt_flow` has the #519
//! consumption decrement and an iterative convergence pass; `belt_structural`
//! has neither. Neither has ever been arbitrated.
//!
//! This measures the disagreement: per-tile lane-rate deltas, and — the part
//! that matters — how many `lane-throughput` ERRORS each model would raise.
//! That check is Severity::Error and participates in candidate selection.
use spaghettio_core::bus::di_cell::DirectInsertion;

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

    let mut layouts = 0usize;
    let mut tiles_total = 0usize;
    let mut tiles_only_flow = 0usize;
    let mut tiles_only_struct = 0usize;
    let mut tiles_agree = 0usize;
    let mut tiles_differ = 0usize;
    let mut max_delta = 0.0f64;
    let mut worst = String::new();
    // over-cap tiles under each model (what `lane-throughput` fires on)
    let mut overcap_flow = 0usize;
    let mut overcap_struct = 0usize;
    let mut layouts_overcap_flow = 0usize;
    let mut layouts_overcap_struct = 0usize;
    let mut per_case: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    // tiles where one model reads ~0 while the other sees real flow
    let mut blind_struct: BTreeMap<String, usize> = BTreeMap::new();
    let mut blind_flow: BTreeMap<String, usize> = BTreeMap::new();

    for C(name, item, rates, machines, inputs) in cases {
        let set: rustc_hash::FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        for &rate in rates {
            for machine in machines {
                let Ok(sr) = solver::solve(item, rate, &set, machine) else { continue };
                for belt in belt_tiers {
                    for di in [DirectInsertion::Off, DirectInsertion::Forced, DirectInsertion::Candidate] {
                        let opts = layout::LayoutOptions {
                            direct_insertion: di,
                            max_belt_tier: belt.map(|s| s.to_string()),
                            ..Default::default()
                        };
                        let Ok(l) = layout::build_bus_layout(&sr, opts) else { continue };
                        layouts += 1;
                        let tag = format!("{name}@{rate}/{machine}/{}/{di:?}", belt.unwrap_or("yellow"));

                        let flow = spaghettio_core::validate::belt_flow::compute_lane_rates(&l, Some(&sr));
                        let strct = spaghettio_core::validate::belt_structural::compute_lane_rates(&l, &sr);

                        // The REAL checks, not a re-derivation: the dispatched
                        // one (belt_structural, via `validate`) and the
                        // non-dispatched one (belt_flow), on the same layout.
                        // `validate` returns Err CARRYING the issues whenever any
                        // fire — an `unwrap_or(0)` here silently reports zero for
                        // every layout that has findings, which is most of them.
                        let all = match spaghettio_core::validate::validate(
                            &l,
                            Some(&sr),
                            spaghettio_core::validate::LayoutStyle::Bus,
                        ) {
                            Ok(is) => is,
                            Err(e) => e.issues,
                        };
                        let dispatched =
                            all.iter().filter(|i| i.category == "lane-throughput").count();
                        let shadow =
                            spaghettio_core::validate::belt_flow::check_lane_throughput(&l, Some(&sr))
                                .len();

                        let mut keys: Vec<(i32, i32)> = flow.keys().copied().collect();
                        for k in strct.keys() {
                            if !flow.contains_key(k) {
                                keys.push(*k);
                            }
                        }
                        keys.sort_unstable();
                        keys.dedup();

                        let seg_of: BTreeMap<(i32, i32), String> = l
                            .entities
                            .iter()
                            .filter_map(|e| {
                                e.segment_id.as_deref().map(|s| {
                                    let head = s.split(':').next().unwrap_or("-").to_string();
                                    ((e.x, e.y), head)
                                })
                            })
                            .collect();

                        let (oc_f, oc_s) = (shadow, dispatched);
                        for k in keys {
                            tiles_total += 1;
                            let f = flow.get(&k).copied();
                            let s = strct.get(&k).copied().map(|(a, b)| [a, b]);
                            let seg = seg_of.get(&k).cloned().unwrap_or_else(|| "-".into());
                            match (f, s) {
                                (Some(f), Some(s)) => {
                                    let d = (f[0] - s[0]).abs().max((f[1] - s[1]).abs());
                                    let (ft, st) = (f[0] + f[1], s[0] + s[1]);
                                    // Which model reads ZERO where the other sees flow?
                                    if st <= 0.01 && ft > 0.1 {
                                        *blind_struct.entry(seg.clone()).or_default() += 1;
                                    } else if ft <= 0.01 && st > 0.1 {
                                        *blind_flow.entry(seg.clone()).or_default() += 1;
                                    }
                                    if d > 0.01 {
                                        tiles_differ += 1;
                                        if d > max_delta {
                                            max_delta = d;
                                            worst = format!(
                                                "{tag} tile{k:?} flow={f:?} struct={s:?}"
                                            );
                                        }
                                    } else {
                                        tiles_agree += 1;
                                    }
                                }
                                (Some(_), None) => tiles_only_flow += 1,
                                (None, Some(_)) => tiles_only_struct += 1,
                                (None, None) => {}
                            }
                        }
                        overcap_flow += oc_f;
                        overcap_struct += oc_s;
                        if oc_f > 0 {
                            layouts_overcap_flow += 1;
                        }
                        if oc_s > 0 {
                            layouts_overcap_struct += 1;
                        }
                        if oc_f > 0 || oc_s > 0 {
                            let e = per_case.entry(tag).or_default();
                            e.0 += oc_f;
                            e.1 += oc_s;
                        }
                    }
                }
            }
        }
    }

    println!("\n===== {layouts} layouts, {tiles_total} tile slots =====");
    println!("  agree (<=0.01/s):        {tiles_agree}");
    println!("  differ (>0.01/s):        {tiles_differ}");
    println!("  only belt_flow has:      {tiles_only_flow}");
    println!("  only belt_structural:    {tiles_only_struct}");
    println!("  worst delta: {max_delta:.2}/s  @ {worst}");
    println!("\n--- over-cap tiles (what `lane-throughput` Errors on) ---");
    println!("  belt_flow (NOT dispatched):      {overcap_flow} tiles in {layouts_overcap_flow} layouts");
    println!("  belt_structural (DISPATCHED):    {overcap_struct} tiles in {layouts_overcap_struct} layouts");
    let tot = |m: &BTreeMap<String, usize>| m.values().sum::<usize>();
    println!(
        "\n--- blind spots: belt_structural reads ~0 where belt_flow sees flow ({} tiles) ---",
        tot(&blind_struct)
    );
    let mut bs: Vec<(&String, &usize)> = blind_struct.iter().collect();
    bs.sort_by(|a, b| b.1.cmp(a.1));
    for (seg, n) in bs.iter().take(15) {
        println!("  {seg:<20} {n:>7}");
    }
    println!(
        "\n--- blind spots: belt_flow reads ~0 where belt_structural sees flow ({} tiles) ---",
        tot(&blind_flow)
    );
    let mut bf: Vec<(&String, &usize)> = blind_flow.iter().collect();
    bf.sort_by(|a, b| b.1.cmp(a.1));
    for (seg, n) in bf.iter().take(15) {
        println!("  {seg:<20} {n:>7}");
    }
    println!("\n--- per-config over-cap (flow, struct) ---");
    for (tag, (f, s)) in &per_case {
        println!("  {tag:<52} flow={f:<5} struct={s}");
    }
}
