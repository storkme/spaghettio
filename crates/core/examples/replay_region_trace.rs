//! Replay a single region fixture JSON and dump the growth-loop trace.
//!
//! Usage:
//!   cargo run --manifest-path crates/core/Cargo.toml --release \
//!     --example replay_region_trace -- <path_to_fixture.json>

use spaghettio_core::fixture::{replay_region_fixture, RegionFixture};
use spaghettio_core::trace::{self, TraceEvent};
use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: replay_region_trace <fixture.json>");
        std::process::exit(2);
    });

    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    let fixture: RegionFixture =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {path}: {e}"));

    println!("== replaying {} ==", fixture.name);
    println!("seeds: {:?}", fixture.seeds);
    println!("initial_specs: {:?}", fixture.initial_specs);
    println!();

    let _sink = trace::set_sink(Box::new(|ev: &TraceEvent| {
        match ev {
            TraceEvent::JunctionGrowthStarted { seed_x, seed_y, participating, nearby_stamped } => {
                println!("→ growth start @ ({seed_x},{seed_y}) — {} participating, {} nearby stamped",
                    participating.len(), nearby_stamped.len());
                for p in participating {
                    println!("    spec {} ({}): tile=({},{}) path_len={}",
                        p.key, p.item, p.initial_tile_x, p.initial_tile_y, p.path_len);
                }
            }
            TraceEvent::JunctionGrowthIteration {
                iter, variant, bbox_x, bbox_y, bbox_w, bbox_h, tiles, boundaries, participating, encountered, ..
            } => {
                let v = if variant.is_empty() { "primary" } else { variant.as_str() };
                println!("  iter {iter} [{v}] bbox=({bbox_x},{bbox_y}) {bbox_w}×{bbox_h} tiles={} participating={} encountered={} boundaries={}",
                    tiles.len(), participating.len(), encountered.len(), boundaries.len());
                for b in boundaries {
                    let io = if b.is_input { "IN" } else { "OUT" };
                    println!("      bnd {io} ({},{}) dir={} item={} spec={} origin={}{}",
                        b.x, b.y, b.direction, b.item, b.spec_key, b.origin,
                        if b.interior { " [interior]" } else { "" });
                }
            }
            TraceEvent::JunctionStrategyAttempt { iter, variant, strategy, outcome, detail, elapsed_us, .. } => {
                let v = if variant.is_empty() { "primary" } else { variant.as_str() };
                println!("    iter {iter} [{v}] strategy={strategy} outcome={outcome} detail=\"{detail}\" elapsed_us={elapsed_us}");
            }
            TraceEvent::SatInvocation { iter, variant, zone_x, zone_y, zone_w, zone_h, satisfied, variables, clauses, solve_time_us, entities_raw, proposed_entities, .. } => {
                let v = if variant.is_empty() { "primary" } else { variant.as_str() };
                println!("      SAT iter {iter} [{v}] zone=({zone_x},{zone_y}) {zone_w}×{zone_h} sat={satisfied} vars={variables} clauses={clauses} time_us={solve_time_us} entities_raw={entities_raw}");
                if *satisfied {
                    for pe in proposed_entities {
                        println!("         raw: {} @ ({},{}) dir={} carries={:?}{}",
                            pe.name, pe.x, pe.y, pe.direction, pe.carries,
                            pe.io_type.as_ref().map(|s| format!(" io={s}")).unwrap_or_default());
                    }
                }
            }
            TraceEvent::SatPruned { zone_x, zone_y, total, kept } => {
                println!("      SAT pruned @ ({zone_x},{zone_y}): {kept}/{total} kept");
            }
            TraceEvent::RegionWalkerVeto { growth_iter, variant, strategy, broken_segment, break_tile_x, break_tile_y, break_count, .. } => {
                let v = if variant.is_empty() { "primary" } else { variant.as_str() };
                println!("    ✗ walker veto iter {growth_iter} [{v}] strategy={strategy} break={broken_segment} at ({break_tile_x},{break_tile_y}) [{break_count} breaks]");
            }
            TraceEvent::JunctionCandidateSolved { growth_iter, variant, strategy, cost, .. } => {
                let v = if variant.is_empty() { "primary" } else { variant.as_str() };
                println!("    ✓ candidate iter {growth_iter} [{v}] strategy={strategy} cost={cost}");
            }
            TraceEvent::JunctionVariantChosen { iter, variant, cost, considered, .. } => {
                println!("    ★ variant chosen iter {iter} variant=\"{variant}\" cost={cost} considered={considered:?}");
            }
            TraceEvent::JunctionSolved { strategy, growth_iter, region_tiles, .. } => {
                println!("✅ solved strategy={strategy} iter={growth_iter} region_tiles={region_tiles}");
            }
            TraceEvent::JunctionGrowthCapped { iters, region_tiles, reason, .. } => {
                println!("💥 CAPPED iters={iters} region_tiles={region_tiles} reason=\"{reason}\"");
            }
            TraceEvent::JunctionTemplateRejected { tile_x, tile_y, bridge_dir, reason } => {
                println!("    template reject @ ({tile_x},{tile_y}) dir={bridge_dir} reason={reason}");
            }
            _ => {}
        }
    }));

    let result = replay_region_fixture(&fixture);
    drop(_sink);

    println!();
    match result.cost {
        Some(c) => println!("final: solved cost={c} entities={}", result.entities.len()),
        None => println!("final: {} (no solution)", if result.capped { "capped" } else { "unsat" }),
    }
}
