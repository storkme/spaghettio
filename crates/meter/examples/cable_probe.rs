//! Where does copper-cable sit along its belt, and who can reach it?
//!
//! Usage: `cable_probe [label] [warmup_ticks] [measure_ticks]`
//!
//! Membership of the ITEM-scoped sections is sampled at several spaced instants
//! and unioned; the per-tile occupancy NUMBERS are still instantaneous, read at
//! window end. A tile can therefore be listed at 0/4 — that is the union
//! working, not a contradiction.
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

    // Membership is sampled at SAMPLES spaced instants across the window and
    // UNIONED, not read once at the end. A single end-of-window sample is what
    // made every earlier version of this probe under-select: a tile is only
    // "carrying ITEM" if an item happens to occupy a slot at the instant we
    // look, and on a tight chain with low belt residency — the zero-headroom
    // population this probe exists to study — a producer's drop tile is empty
    // most ticks by design. Sampling once there drops real taps and reports the
    // remainder as if complete.
    const SAMPLES: usize = 8;
    let chunk = (measure / SAMPLES as u64).max(1);
    let mut seen: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    let mut ticked = 0u64;
    for k in 0..SAMPLES {
        // Clamp every iteration to what is left, not just the last one. With
        // `chunk` floored at 1, a `measure` smaller than SAMPLES would
        // otherwise run one tick per iteration and OVERSHOOT the requested
        // window (measure=5 ran 7 ticks). Unreachable at the defaults, wrong
        // regardless.
        let remaining = measure.saturating_sub(ticked);
        let run = if k == SAMPLES - 1 { remaining } else { chunk.min(remaining) };
        f.run_for(run);
        ticked += run;
        for (i, t) in f.net.tiles.iter().enumerate() {
            let holds = t.lanes.iter().any(|lane| {
                lane.slots_debug().iter().flatten().any(|it| f.items.name(*it) == ITEM)
            });
            if holds {
                seen.insert(i);
            }
        }
    }

    println!("window: {warmup} warmup + {ticked} measured ticks ({SAMPLES} membership samples)");

    // machines by recipe, with crafts. `crafts` is a window total; `state` is
    // a single instantaneous sample taken at window end, NOT a window
    // aggregate — a machine that worked throughout can print `ItemShortage`
    // because of where the last tick fell. Read them differently.
    println!("\nmachines (recipe, pos, state AT END OF WINDOW, crafts over window):");
    let mut ms: Vec<_> = f.machines.iter().collect();
    ms.sort_by_key(|m| (m.recipe.clone(), m.pos.0, m.pos.1));
    for mc in &ms {
        println!("  {:<20} at {:?}  {:?}  crafts={}", mc.recipe, mc.pos, mc.state, mc.crafts);
    }

    // Every tile that held ITEM at ANY of the sampled instants. Still a lower
    // bound — a tile empty at all eight would drop out — but no longer a
    // single-instant artifact. Two things to keep straight when reading the
    // output below: MEMBERSHIP is unioned across the window, while the
    // occupancy NUMBERS printed per tile are instantaneous, read at the end.
    // So a tile can appear in the list at 0/4 on both lanes; that is the
    // sampling working, not a contradiction.
    let item_tiles: Vec<usize> = seen.iter().copied().collect();

    // Membership test against the set, not a linear scan of the Vec — the
    // union is larger than the old single sample and this is inside a loop
    // over every inserter.
    let touches_item = |e: &Endpoint| match e {
        Endpoint::Belt(t) => seen.contains(t),
        _ => false,
    };

    println!(
        "\n{ITEM} inserters (machine->belt = producer tap, belt->machine = consumer tap, belt->belt = transfer):"
    );
    println!("  counters are window-scoped (warmup subtracted)");
    // No `belt_to_belt` predicate here, deliberately. The original probe
    // required BOTH endpoints to be `Endpoint::Belt`, which excludes exactly
    // the two things the header names — a producer tap is Machine->Belt and a
    // consumer tap is Belt->Machine, so both were silently dropped and the
    // section reported only bus transfer inserters. `touches_item` alone is
    // the right filter: it catches a producer's drop tile and a consumer's
    // pickup tile as well as belt->belt.
    let mut printed = 0usize;
    // Counted as two classes, not one. A combined `machine_taps` counter is
    // satisfied by EITHER class surviving, so producer taps could vanish
    // entirely while consumer taps kept the count positive and no guard fired
    // — losing exactly the producer-saturation signal this probe is for.
    let mut producer_taps = 0usize; // machine -> belt
    let mut consumer_taps = 0usize; // belt -> machine
    for (w, (d0, s0)) in f.inserters.iter().zip(&base) {
        if !(touches_item(&w.pickup) || touches_item(&w.drop)) {
            continue;
        }
        printed += 1;
        if matches!(w.pickup, Endpoint::Machine(_)) && touches_item(&w.drop) {
            producer_taps += 1;
        }
        if matches!(w.drop, Endpoint::Machine(_)) && touches_item(&w.pickup) {
            consumer_taps += 1;
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
    // Non-vacuity guard. Lesson 6 of the handoff note committed alongside this
    // file: "a check that stops discriminating is worse than one that fails",
    // and any scoping change needs a guard asserting the probe still has
    // something to inspect. Every section below the machines listing is scoped
    // to ITEM via `item_tiles`, and an empty scope looks identical to a healthy
    // factory with nothing to report.
    //
    // Four failure levels. The partial ones are the dangerous ones, and they
    // are named separately because a single combined counter hides them: with
    // membership derived from belt contents, a whole tap CLASS can drop out
    // while the other keeps any combined count positive, leaving output that
    // looks complete. Union sampling above makes this much less likely; it
    // does not make it impossible, so the guard still checks.
    if item_tiles.is_empty() {
        eprintln!(
            "\n!! VACUOUS: no belt tile held {ITEM} at sample time. Every {ITEM}-scoped \
             section (the inserter list and both occupancy blocks — NOT the machines \
             listing, which is unscoped) is empty by construction, not by finding. Wrong \
             fixture, wrong item, or a window that ended mid-gap."
        );
    } else if printed == 0 {
        eprintln!(
            "\n!! VACUOUS: {} tiles hold {ITEM} but no inserter touches any of them. \
             Nothing was filtered out — there is genuinely nothing to report, which for \
             this probe is itself the finding.",
            item_tiles.len()
        );
    } else {
        if producer_taps == 0 {
            eprintln!(
                "\n!! PARTIALLY VACUOUS: {printed} inserters touch {ITEM} but NONE is a \
                 producer tap (machine -> {ITEM} belt). Producer-side saturation is this \
                 probe's headline output, so treat it as absent, not as zero. Either no \
                 machine produces {ITEM} in this fixture, or every producer drop tile was \
                 empty at all {SAMPLES} samples."
            );
        }
        if consumer_taps == 0 {
            eprintln!(
                "\n!! PARTIALLY VACUOUS: {printed} inserters touch {ITEM} but NONE is a \
                 consumer tap ({ITEM} belt -> machine). Nothing downstream is reading this \
                 belt, which is either the finding or a sampling miss."
            );
        }
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

    println!("\nbelt tiles carrying {ITEM}:");
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
        // Denominator derived, not hardcoded. This printed `occ/8`, right only
        // because SLOTS_PER_TILE is 4 and there are two lanes — while the
        // per-lane block above already derived its own from `slots.len()`. The
        // same quantity, computed two ways in one file, is how a constant
        // change silently desyncs one of them.
        let cap: usize = t.lanes.iter().map(|l| l.slots_debug().len()).sum();
        println!("  {:?} occ {}/{cap}  {:?}", t.pos, t.occupancy(), names);
    }
}
