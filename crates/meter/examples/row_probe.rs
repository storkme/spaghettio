//! Probe: does a zero-margin shared input belt starve its tail?
//!
//! Run: `cargo run -p spaghettio_meter --example row_probe`
//!
//! This is the PR-1 instrument for the question PR 2 answers against real
//! Factorio. It prints, per margin, the per-consumer consumption rate,
//! ingredient buffer, and starved ticks — the three quantities #448's
//! per-machine dumps report.

use spaghettio_meter::{BeltTier, InserterKind, RowFixture};

const DEMAND: f64 = 7.5;
const CONSUMERS: usize = 6;
const WARMUP: u64 = 30_000;
const MEASURE: u64 = 60_000;

fn main() {
    println!("chain-ec15 shape: {CONSUMERS} consumers x {DEMAND}/s on express (45.0 nominal)");
    println!("stack inserters, capacity L2, smooth boundary supply\n");
    println!(
        "{:>7}  {:>8}  {:<44}  {:<28}  starved ticks",
        "margin", "supply/s", "consumption/s (head->tail)", "buffer"
    );

    for margin in [1.0, 1.02, 1.05, 1.10, 1.25, 1.50] {
        let mut fx = RowFixture::build(
            BeltTier::Blue,
            InserterKind::Stack,
            2,
            CONSUMERS,
            DEMAND,
            margin,
            "copper-cable",
        );
        fx.world.run_for(WARMUP);
        for &c in &fx.chests {
            fx.world.chests[c].consumed = 0;
            fx.world.chests[c].starved_ticks = 0;
        }
        let start = fx.world.ticks;
        fx.world.run_for(MEASURE);
        let elapsed = fx.world.ticks - start;

        let rates: Vec<String> = fx
            .chests
            .iter()
            .map(|&c| format!("{:.2}", fx.world.chests[c].consumption_rate(elapsed)))
            .collect();
        let buffers: Vec<String> = fx
            .chests
            .iter()
            .map(|&c| fx.world.chests[c].buffer.to_string())
            .collect();
        let starved: u64 = fx
            .chests
            .iter()
            .map(|&c| fx.world.chests[c].starved_ticks)
            .sum();

        println!(
            "{:>7.2}  {:>8.1}  {:<44}  {:<28}  {}",
            margin,
            DEMAND * CONSUMERS as f64 * margin,
            rates.join(" "),
            buffers.join(" "),
            starved
        );
    }

    println!("\nbelt occupancy per tile (both lanes, 8 slots max), margin 1.0:");
    let mut fx = RowFixture::build(
        BeltTier::Blue,
        InserterKind::Stack,
        2,
        CONSUMERS,
        DEMAND,
        1.0,
        "copper-cable",
    );
    fx.world.run_for(WARMUP);
    println!("  {:?}", fx.tile_occupancy());
    let src_rejected: u64 = fx.world.sources.iter().map(|s| s.rejected).sum();
    let src_injected: u64 = fx.world.sources.iter().map(|s| s.injected).sum();
    println!(
        "  source injected {src_injected}, rejected {src_rejected} \
         (rejected > 0 means the belt backed up to its head)"
    );
}
