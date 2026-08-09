//! #609 claim 1, completeness pass over LAYOUT MODES, with the correct
//! discriminator.
//!
//! A perpendicular feeder is NOT automatically a B8 sideload. Per
//! `factorio-mechanics.md`:
//!
//! - B11 TURN — sole perpendicular feeder, no straight feeder: BOTH lanes
//!   preserved. Harmless. The engine builds these constantly.
//! - B8 SIDELOAD — perpendicular feeder coexisting with a straight feeder into
//!   the same tile: NEAR LANE ONLY. This is the defect shape #609 wants to
//!   close.
//! - B10 — two opposing perpendicular feeders, no straight: both lanes.
//!
//! `belt_flow::lane_transfer` already makes exactly this distinction via
//! `to_has_straight_feeder`. A binary perpendicular/straight classifier does
//! not, and reports turns as if they were sideloads.
//!
//! Question: does ANY layout mode produce a true B8 sideload into a
//! `row:*:belt-in:*` run? Axes the original sweep did not vary: strategy,
//! row_layout, merge_tap, cell_composition, stacking.
//!
//! Prints incrementally — a killed run keeps everything already measured.
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::layout::{LayoutOptions, LayoutStrategy, RowLayout};
use spaghettio_core::common::{
    dir_to_vec, is_splitter, is_surface_belt, is_ug_belt, splitter_second_tile,
};
use spaghettio_core::models::EntityDirection;
use spaghettio_core::{bus::layout, solver};
use std::collections::HashMap;
use std::io::Write;

struct C(&'static str, &'static str, f64, &'static str, &'static [&'static str]);

/// One swept layout configuration:
/// `(belt tier, DI policy, strategy, row layout, merge_tap, cell composition, stacking)`.
type Combo = (
    Option<&'static str>,
    DirectInsertion,
    LayoutStrategy,
    RowLayout,
    bool,
    CellComposition,
    u8,
);

fn perp(a: EntityDirection, b: EntityDirection) -> bool {
    use EntityDirection::*;
    !matches!(
        (a, b),
        (North, North) | (South, South) | (East, East) | (West, West)
            | (North, South) | (South, North) | (East, West) | (West, East)
    )
}

fn main() {
    // Full cross-product tier.
    let tier_a = [
        C("ec10", "electronic-circuit", 10.0, "assembling-machine-2", &["iron-plate", "copper-plate"]),
        C("ec30", "electronic-circuit", 30.0, "assembling-machine-2", &["iron-plate", "copper-plate"]),
        C("gear10", "iron-gear-wheel", 10.0, "assembling-machine-2", &["iron-plate"]),
        C("gear45", "iron-gear-wheel", 45.0, "assembling-machine-3", &["iron-plate"]),
        C("ac10", "advanced-circuit", 10.0, "assembling-machine-2", &["iron-plate", "copper-plate", "coal", "crude-oil", "water"]),
        C("pu5", "processing-unit", 5.0, "assembling-machine-3", &["iron-plate", "copper-plate", "coal", "crude-oil", "water", "sulfur"]),
        C("lds2", "low-density-structure", 2.0, "assembling-machine-3", &["iron-plate", "copper-plate", "coal", "crude-oil", "petroleum-gas"]),
        C("sci3_5", "chemical-science-pack", 5.0, "assembling-machine-3", &["iron-plate", "copper-plate", "coal", "crude-oil", "water", "sulfur"]),
        C("battery5", "battery", 5.0, "chemical-plant", &["iron-plate", "copper-plate", "sulfuric-acid"]),
        C("sulf20", "sulfuric-acid", 20.0, "chemical-plant", &["iron-plate", "water", "sulfur"]),
    ];
    // Scale tier — megablock geometry, reduced axes (these layouts are slow).
    let tier_b = [
        C("ec100", "electronic-circuit", 100.0, "assembling-machine-3", &["iron-plate", "copper-plate"]),
        C("gear100", "iron-gear-wheel", 100.0, "assembling-machine-3", &["iron-plate"]),
        C("pu20", "processing-unit", 20.0, "assembling-machine-3", &["iron-plate", "copper-plate", "coal", "crude-oil", "water", "sulfur"]),
    ];

    let mut layouts = 0usize;
    let mut run_tiles = 0usize;
    let mut b8 = 0usize;
    let mut b10 = 0usize;
    let mut b11_turn = 0usize;
    let mut u7_into_ug_input = 0usize;
    let mut b8_item_mismatch = 0usize;
    let mut perp_multi = 0usize;
    let mut examples: Vec<String> = Vec::new();

    for (tier, cases) in [("A", &tier_a[..]), ("B", &tier_b[..])] {
        for C(name, item, rate, machine, inputs) in cases {
            let set: rustc_hash::FxHashSet<String> =
                inputs.iter().map(|s| s.to_string()).collect();
            let Ok(sr) = solver::solve(item, *rate, &set, machine) else {
                println!("{name}: solver refused");
                continue;
            };

            // Tier A gets the full cross-product; tier B only the axes most
            // likely to reshape tap geometry (megablocks are slow).
            let combos: Vec<Combo> =
                if tier == "A" {
                    let mut v = Vec::new();
                    for belt in [None, Some("fast-transport-belt"), Some("express-transport-belt")] {
                        for di in [DirectInsertion::Off, DirectInsertion::Forced, DirectInsertion::Candidate] {
                            for st in [LayoutStrategy::Pooled, LayoutStrategy::PartitionedDecomposed] {
                                for rl in [RowLayout::VerticalSplit, RowLayout::HorizontalStack] {
                                    for mt in [false, true] {
                                        for cc in [CellComposition::Off, CellComposition::Candidate] {
                                            for stk in [1u8, 4u8] {
                                                v.push((belt, di, st, rl, mt, cc, stk));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    v
                } else {
                    let mut v = Vec::new();
                    for di in [DirectInsertion::Off, DirectInsertion::Forced, DirectInsertion::Candidate] {
                        for st in [LayoutStrategy::Pooled, LayoutStrategy::PartitionedDecomposed] {
                            v.push((None, di, st, RowLayout::VerticalSplit, false, CellComposition::Off, 1u8));
                        }
                    }
                    v
                };

            let (mut n_ok, mut n_b8) = (0usize, 0usize);
            for (belt, di, strategy, row_layout, merge_tap, cell_composition, stacking) in combos {
                let opts = LayoutOptions {
                    direct_insertion: di,
                    max_belt_tier: belt.map(|s| s.to_string()),
                    strategy,
                    row_layout,
                    merge_tap,
                    cell_composition,
                    stacking,
                    ..Default::default()
                };
                let Ok(l) = layout::build_bus_layout(&sr, opts) else { continue };
                layouts += 1;
                n_ok += 1;

                // every belt tile, its direction and carried item
                let mut dir_of: HashMap<(i32, i32), EntityDirection> = HashMap::new();
                let mut carries_of: HashMap<(i32, i32), Option<String>> = HashMap::new();
                for e in &l.entities {
                    if is_surface_belt(&e.name) || is_ug_belt(&e.name) || is_splitter(&e.name) {
                        dir_of.insert((e.x, e.y), e.direction);
                        carries_of.insert((e.x, e.y), e.carries.clone());
                        // A splitter occupies TWO tiles. Registering only the
                        // origin makes the second invisible, so a run tile fed
                        // straight THROUGH it reads as having no straight feeder
                        // — which collapses a genuine B8 sideload into the benign
                        // B11-turn bucket. Under-counts precisely what this probe
                        // exists to find.
                        if is_splitter(&e.name) {
                            let second = splitter_second_tile(e);
                            dir_of.insert(second, e.direction);
                            carries_of.insert(second, e.carries.clone());
                        }
                    }
                }
                // the belt-in run tiles
                let run: Vec<&spaghettio_core::models::PlacedEntity> = l
                    .entities
                    .iter()
                    .filter(|e| {
                        e.segment_id.as_deref().is_some_and(|s| s.contains(":belt-in:"))
                            // Surface belts AND underground tiles. A UG *output*
                            // can be sideloaded into under normal B8 rules (U10);
                            // scanning only surface tiles silently excludes
                            // undergrounded stretches of a run from the question.
                            && (is_surface_belt(&e.name) || is_ug_belt(&e.name))
                    })
                    .collect();

                for t in &run {
                    run_tiles += 1;
                    let td = t.direction;
                    let titem = t.carries.as_deref();
                    let (mut straight, mut perps): (bool, Vec<(i32, i32)>) = (false, Vec::new());
                    let (mut straight_im, mut perps_im) = (false, 0usize);
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let f = (t.x + dx, t.y + dy);
                        let Some(&fd) = dir_of.get(&f) else { continue };
                        let (fdx, fdy) = dir_to_vec(fd);
                        if (f.0 + fdx, f.1 + fdy) != (t.x, t.y) {
                            continue; // not feeding this tile
                        }
                        let item_match = match (titem, carries_of.get(&f).and_then(|c| c.as_deref())) {
                            (Some(a), Some(b)) => a == b,
                            _ => true, // untagged counts, same as the check's guard
                        };
                        if fd == td {
                            straight = true;
                            if item_match {
                                straight_im = true;
                            }
                        } else if perp(fd, td) {
                            perps.push((dx, dy));
                            if item_match {
                                perps_im += 1;
                            }
                        }
                    }
                    if perps.is_empty() {
                        continue;
                    }
                    // A perpendicular feed onto a UG *input* is U7 (FAR lane),
                    // not B8 (near lane) — an inverted rule, so it must not be
                    // pooled with the B8 count either way.
                    if is_ug_belt(&t.name) && t.io_type.as_deref() == Some("input") {
                        u7_into_ug_input += 1;
                        continue;
                    }
                    // Require the ITEM MATCH for the headline count. Without it a
                    // perpendicular belt of an unrelated item crossing beside a
                    // straight feeder scores as a B8 defect that isn't one.
                    if straight && straight_im && perps_im > 0 {
                        b8 += 1;
                        n_b8 += 1;
                        if examples.len() < 25 {
                            examples.push(format!(
                                "{name} belt={} di={di:?} strat={strategy:?} rl={row_layout:?} mt={merge_tap} cc={cell_composition:?} st={stacking} tile=({},{}) seg={}",
                                belt.unwrap_or("yellow"), t.x, t.y,
                                t.segment_id.as_deref().unwrap_or("-")
                            ));
                        }
                    } else if straight {
                        // perp + straight, but the items differ — not a defect,
                        // recorded so it cannot hide inside the benign bucket.
                        b8_item_mismatch += 1;
                    } else if perps.len() >= 2
                        && perps.iter().any(|a| perps.iter().any(|b| a.0 == -b.0 && a.1 == -b.1))
                    {
                        b10 += 1;
                    } else if perps.len() == 1 {
                        b11_turn += 1;
                    } else {
                        // >=2 perpendicular feeders, none opposing. Not a turn and
                        // not B10; previously swept into `b11_turn` and labelled
                        // benign without justification.
                        perp_multi += 1;
                    }
                }
            }
            println!(
                "tier{tier} {name:<10} layouts={n_ok:<4} B8_here={n_b8:<4} | running: layouts={layouts} run_tiles={run_tiles} B8={b8} B10={b10} B11turn={b11_turn}"
            );
            std::io::stdout().flush().ok();
        }
    }

    println!("\n===== {layouts} layouts, {run_tiles} belt-in run tiles =====");
    // No "[item-matched: N]" qualifier: the B8 branch REQUIRES the item match,
    // so a separate count would always equal `b8` and imply a non-matched B8
    // category that cannot exist. Item-mismatched perp+straight is reported on
    // its own line below.
    println!("  B8 sideload into a belt-in tile (perp + straight):  {b8}");
    println!("  B10 opposing double sideload (both lanes):          {b10}");
    println!("  B11 turn, sole perpendicular (both lanes, benign):  {b11_turn}");
    println!("  U7 perpendicular onto a UG INPUT (far lane, inverted): {u7_into_ug_input}");
    println!("  perp + straight but ITEM MISMATCH (not a defect):      {b8_item_mismatch}");
    println!("  >=2 perpendicular, none opposing (uncategorised):      {perp_multi}");
    if b8 == 0 {
        println!("\nNO TRUE B8 SIDELOAD into any belt-in run, across every layout mode swept.");
    } else {
        println!("\n--- B8 examples (POPULATION FOR #609) ---");
        for e in &examples {
            println!("  {e}");
        }
    }
}
