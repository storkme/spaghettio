//! `di-patterns` — mine direct-insertion (DI) patterns from a corpus of
//! community blueprints. This is the tool that produced RFC-053's evidence
//! base; it is tracked (rather than living in the gitignored
//! `crates/core/examples/`) so the RFC's central claims are reproducible.
//!
//! ```text
//! cargo run --release -p spaghettio_mining --bin di-patterns -- census <corpus-dir>
//! cargo run --release -p spaghettio_mining --bin di-patterns -- geometry <corpus-dir> <producer> <consumer>
//! ```
//!
//! A DI pair is an inserter whose pickup AND drop tiles both resolve to
//! machine tiles (the `classify.rs` convention, engine direction = drop
//! side). Belt-to-belt inserters are therefore *not* counted — including
//! the belt→belt "bridge" of PR #432, which is why the corpus shows zero
//! instances of that shape rather than a small minority.
//!
//! **Vintage caveat**: inserter *names* shifted between game versions
//! (1.1's `stack-inserter` is 2.0's `bulk-inserter`, and 2.0 added a new
//! belt-stacking `stack-inserter`). Mined *geometry* is trustworthy
//! immediately; mined *throughput* attribution is not, without version
//! gating. This tool reports names verbatim and does not infer rates.

use spaghettio_core::analysis;
use spaghettio_core::common::{
    dir_to_vec, entity_size, inserter_reach, is_inserter, is_machine_entity,
};
use std::collections::{BTreeMap, HashMap};

/// One mined DI observation, canonicalized.
struct Obs {
    producer: String,
    consumer: String,
    inserter: String,
    /// Empty tiles between the two machines' facing edges.
    gap: i32,
    /// Offset of the consumer's origin from the producer's, perpendicular
    /// to the insertion axis — the "straddle" signature.
    lateral: i32,
    vertical: bool,
}

fn mine(dir: &str) -> Vec<Obs> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        eprintln!("cannot read corpus dir: {dir}");
        return out;
    };
    for ent in rd.flatten() {
        let Ok(txt) = std::fs::read_to_string(ent.path()) else { continue };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { continue };
        let Some(bp) = v.get("blueprintString").and_then(|s| s.as_str()) else { continue };
        let Ok(analyses) = analysis::analyze_blueprint_string_any(bp) else { continue };
        for na in &analyses {
            let ents = &na.layout.entities;
            let mut occ: HashMap<(i32, i32), usize> = HashMap::new();
            for (i, m) in ents.iter().enumerate() {
                if !is_machine_entity(&m.name) {
                    continue;
                }
                let (w, h) = entity_size(&m.name);
                for dx in 0..w as i32 {
                    for dy in 0..h as i32 {
                        occ.insert((m.x + dx, m.y + dy), i);
                    }
                }
            }
            for ins in ents {
                if !is_inserter(&ins.name) {
                    continue;
                }
                let (dx, dy) = dir_to_vec(ins.direction);
                let r = inserter_reach(&ins.name);
                let (Some(&di), Some(&si)) = (
                    occ.get(&(ins.x + dx * r, ins.y + dy * r)),
                    occ.get(&(ins.x - dx * r, ins.y - dy * r)),
                ) else {
                    continue;
                };
                if di == si {
                    continue;
                }
                let (p, c) = (&ents[si], &ents[di]);
                let vertical = dy != 0;
                let (pw, ph) = entity_size(&p.name);
                let (cw, ch) = entity_size(&c.name);
                // Gap = distance between origins minus the span of whichever
                // machine sits "first" along the insertion axis.
                let gap = if vertical {
                    (c.y - p.y).abs() - if c.y > p.y { ph as i32 } else { ch as i32 }
                } else {
                    (c.x - p.x).abs() - if c.x > p.x { pw as i32 } else { cw as i32 }
                };
                let lateral = if vertical { c.x - p.x } else { c.y - p.y };
                out.push(Obs {
                    producer: p.recipe.clone().unwrap_or_else(|| p.name.clone()),
                    consumer: c.recipe.clone().unwrap_or_else(|| c.name.clone()),
                    inserter: ins.name.clone(),
                    gap,
                    lateral,
                    vertical,
                });
            }
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("census");
    let dir = args.get(2).cloned().unwrap_or_else(|| "scripts/blueprints".into());
    let obs = mine(&dir);
    if obs.is_empty() {
        eprintln!("no DI observations found in {dir}");
        std::process::exit(1);
    }

    match cmd {
        "geometry" => {
            let (want_p, want_c) = (
                args.get(3).cloned().unwrap_or_else(|| "copper-cable".into()),
                args.get(4).cloned().unwrap_or_else(|| "electronic-circuit".into()),
            );
            let mut g: BTreeMap<(String, i32, i32, bool), usize> = BTreeMap::new();
            for o in obs.iter().filter(|o| o.producer == want_p && o.consumer == want_c) {
                *g.entry((o.inserter.clone(), o.gap, o.lateral, o.vertical)).or_insert(0) += 1;
            }
            let mut v: Vec<_> = g.into_iter().collect();
            v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
            println!("{want_p} -> {want_c}: geometry distribution");
            println!("{:>7}  {:<22} {:>5} {:>9}  axis", "count", "inserter", "gap", "lateral");
            for ((ins, gap, lat, vert), n) in v.iter().take(25) {
                println!(
                    "{n:>7}  {ins:<22} {gap:>5} {lat:>9}  {}",
                    if *vert { "vertical" } else { "horizontal" }
                );
            }
        }
        _ => {
            let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
            for o in &obs {
                *pairs.entry((o.producer.clone(), o.consumer.clone())).or_insert(0) += 1;
            }
            let mut v: Vec<_> = pairs.into_iter().collect();
            v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
            println!("{} DI observations; top producer -> consumer pairs:", obs.len());
            for ((p, c), n) in v.iter().take(15) {
                println!("{n:>7}  {p} -> {c}");
            }
        }
    }
}
