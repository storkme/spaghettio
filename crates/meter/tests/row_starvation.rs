//! Row-level behaviour on synthetic fixtures — and **the PR-1 negative
//! result**.
//!
//! # What these tests establish
//!
//! The physics core reproduces every *belt-level* property RFC-054 needs
//! (see `src/belt.rs` and `src/inserter.rs` unit tests): gaps do not heal,
//! dead ends back up, compressed lanes move at full speed, throughput and
//! inserter rates are derived rather than tabulated, and a sparse source
//! costs an inserter throughput through partial hands and lost swings.
//!
//! # What they do NOT establish
//!
//! **The model does not currently reproduce #448.** With a *smooth*
//! boundary supply at exactly aggregate demand and bounded consumer
//! buffers, a 6-consumer express row delivers `[7.50 × 6]` — no tail
//! starvation at all. The user's conservation intuition holds in this
//! configuration: supply equals demand, buffers are finite, and it works
//! out.
//!
//! Worse, a margin sweep is **non-monotonic** — margin 1.02 starves where
//! 1.00 does not, recovering by 1.25 (`cargo run -p spaghettio_meter
//! --example row_probe`). In a fully deterministic simulator with periodic
//! sources and periodic inserter swings, that is the signature of *phase
//! aliasing* between the two cadences, not a physical effect.
//!
//! # Why this is landed rather than fixed
//!
//! The fixture idealizes both ends of the real row. A real row-input belt
//! is fed by a producer cell's output **inserters** — bursty, discrete
//! drops onto particular tiles — not a smooth boundary source; and real
//! consumers draw in discrete craft batches, not a smooth trickle. Adding
//! burstiness would very likely produce starvation, which is exactly why
//! it must not be added *here*: choosing mechanisms until the answer
//! matches is how an instrument acquires the quirks it is supposed to
//! detect.
//!
//! The anchored margin sweep against real Factorio is **PR 2**, and this
//! is its first and most important question. Landing the negative result
//! at ~1k LOC — rather than after machines, boundary and convergence are
//! built on top — is precisely what the RFC's 4-PR split was designed to
//! buy.
//!
//! Tracking: #457. Divergences belong in `docs/meter-divergence.md`.

use spaghettio_meter::{BeltTier, InserterKind, RowFixture};

const DEMAND: f64 = 7.5;
const CONSUMERS: usize = 6;
const WARMUP: u64 = 30_000;
const MEASURE: u64 = 60_000;

struct Measured {
    rates: Vec<f64>,
    starved_ticks: Vec<u64>,
}

fn measure(consumers: usize, margin: f64, level: u8) -> Measured {
    let mut fx = RowFixture::build(
        BeltTier::Blue,
        InserterKind::Stack,
        level,
        consumers,
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

    Measured {
        rates: fx
            .chests
            .iter()
            .map(|&c| fx.world.chests[c].consumption_rate(elapsed))
            .collect(),
        starved_ticks: fx
            .chests
            .iter()
            .map(|&c| fx.world.chests[c].starved_ticks)
            .collect(),
    }
}

/// **The negative result, pinned.**
///
/// This test asserts the model's *current* behaviour so that a later
/// change which makes the row starve cannot land silently — it will fail
/// here and force a decision-log entry saying what changed and why.
///
/// It is deliberately phrased as "this is what we measure", not "this is
/// correct". If PR 2's anchored sweep shows real Factorio starving this
/// configuration, this test is the thing that must change, and its failure
/// is the signal that the belt model needs work.
#[test]
fn smooth_supply_at_zero_margin_does_not_starve_the_row() {
    let m = measure(CONSUMERS, 1.0, 2);

    for (i, &r) in m.rates.iter().enumerate() {
        assert!(
            (r - DEMAND).abs() < 0.05,
            "position {i} delivered {r:.2}/s against {DEMAND}/s demand — the \
             model's smooth-supply behaviour changed. If this is intended, \
             record it in the RFC-054 decision log and in \
             docs/meter-divergence.md. Full row: {:?}",
            m.rates
        );
    }
    assert_eq!(
        m.starved_ticks.iter().sum::<u64>(),
        0,
        "no consumer should starve under smooth supply at exactly demand; \
         got {:?}",
        m.starved_ticks
    );
}

/// Ample surplus must also be clean. Together with the test above this
/// brackets the sweep: both ends behave, and the disorder PR 2 must
/// explain lives in between (margin 1.02–1.10).
#[test]
fn generous_margin_serves_every_consumer() {
    let m = measure(CONSUMERS, 1.5, 2);
    for (i, &r) in m.rates.iter().enumerate() {
        assert!(
            (r - DEMAND).abs() < 0.05,
            "position {i} delivered {r:.2}/s at 1.5x margin; row {:?}",
            m.rates
        );
    }
}

/// A single consumer has no head and no tail. #448 scoped its check to
/// rows of ≥2 for exactly this reason, and the floor should hold whatever
/// happens to the multi-consumer case.
#[test]
fn a_single_consumer_is_not_starved() {
    let m = measure(1, 1.0, 2);
    assert!(
        m.rates[0] > DEMAND * 0.98,
        "one consumer at its own demand should be served, got {:.2}",
        m.rates[0]
    );
}

/// Supply genuinely below demand *must* under-deliver. This is the
/// sanity floor: an instrument that reports full delivery on a starved row
/// is measuring nothing, and would pass the two tests above vacuously.
#[test]
fn genuine_undersupply_is_visible() {
    let m = measure(CONSUMERS, 0.8, 2);
    let total: f64 = m.rates.iter().sum();
    let planned = DEMAND * CONSUMERS as f64;
    assert!(
        total < planned * 0.95,
        "80% supply must show as under-delivery; got {total:.2}/s against \
         {planned:.2}/s planned (row {:?})",
        m.rates
    );
    assert!(
        m.starved_ticks.iter().sum::<u64>() > 0,
        "undersupply must produce starved ticks; got {:?}",
        m.starved_ticks
    );
}

/// The belt cannot carry more than its physical capacity, and the excess
/// must show up as *refused injections* rather than being quietly
/// absorbed. Silent absorption would let the meter over-report delivery.
#[test]
fn oversupply_is_refused_at_the_belt_head_not_absorbed() {
    let mut fx = RowFixture::build(
        BeltTier::Blue,
        InserterKind::Stack,
        2,
        CONSUMERS,
        DEMAND,
        3.0, // 135/s offered onto a 45/s belt
        "copper-cable",
    );
    fx.world.run_for(WARMUP);
    let rejected: u64 = fx.world.sources.iter().map(|s| s.rejected).sum();
    assert!(
        rejected > 0,
        "offering 3x a belt's capacity must be refused at the head — \
         otherwise the model creates throughput that does not exist"
    );
}
