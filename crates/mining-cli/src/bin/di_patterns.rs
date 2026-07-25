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
use std::collections::{BTreeSet, BTreeMap, HashMap};

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
    /// Blueprint-member id, unique across the corpus scan. Machine
    /// indices are only meaningful within one member.
    member: u32,
    /// Index of the producer / consumer machine inside that member. These
    /// are what make fan-in, fan-out and chain depth computable — without
    /// them every observation is anonymous and only PAIRWISE geometry can
    /// be reported (which is all the `geometry` subcommand ever needed).
    p_idx: usize,
    c_idx: usize,
}

fn mine(dir: &str) -> Vec<Obs> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        eprintln!("cannot read corpus dir: {dir}");
        return out;
    };
    let mut member: u32 = 0;
    for ent in rd.flatten() {
        let Ok(txt) = std::fs::read_to_string(ent.path()) else { continue };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { continue };
        let Some(bp) = v.get("blueprintString").and_then(|s| s.as_str()) else { continue };
        let Ok(analyses) = analysis::analyze_blueprint_string_any(bp) else { continue };
        for na in &analyses {
            member += 1;
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
                    member,
                    p_idx: si,
                    c_idx: di,
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
        "faces" => {
            // RFC-053 Phase 2 evidence. The `geometry` view looks at a DI
            // pair in isolation; this asks the question Phase 2 actually
            // needs answered: for a machine that RECEIVES direct insertion,
            // where does everything ELSE it touches live?
            //
            // Reported per consumer machine: the side the DI band arrives
            // on, then every other inserter touching that machine — which
            // side, reach, whether it feeds or drains the machine, and
            // whether its far end is a belt or another machine.
            //
            // This is what tells us whether "second input belt below the
            // consumers" (the RFC's hand-drawn sketch) is what people
            // build, something rarer, or something nobody does.
            let (want_p, want_c) = (
                args.get(3).cloned().unwrap_or_else(|| "copper-cable".into()),
                args.get(4).cloned().unwrap_or_else(|| "electronic-circuit".into()),
            );
            let plans = mine_faces(&dir, &want_p, &want_c);
            let mut hist: BTreeMap<String, usize> = BTreeMap::new();
            let mut side_hist: BTreeMap<String, usize> = BTreeMap::new();
            let mut n = 0usize;
            for fp in &plans {
                n += 1;
                hist.entry(fp.signature()).and_modify(|c| *c += 1).or_insert(1);
                for f in &fp.others {
                    *side_hist
                        .entry(format!(
                            "{} {} r{} -> {}",
                            f.side,
                            if f.into_machine { "IN " } else { "OUT" },
                            f.reach,
                            f.far_kind
                        ))
                        .or_insert(0) += 1;
                }
            }
            println!("face plans for DI consumers of {want_p} -> {want_c}: {n} machines");
            println!();
            println!("  per-interface distribution (side is relative to the consumer machine):");
            let mut sv: Vec<_> = side_hist.into_iter().collect();
            sv.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
            for (k, c) in sv.iter().take(20) {
                println!("    {c:>6}  {k}");
            }
            println!();
            println!("  whole-machine face plans (DI side | other interfaces):");
            let mut hv: Vec<_> = hist.into_iter().collect();
            hv.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
            for (k, c) in hv.iter().take(20) {
                println!("    {c:>6}  {k}");
            }
        }
        "fan" => {
            // Multi-band evidence (RFC-053 Phase 3). Three questions the
            // pairwise `geometry` view structurally cannot answer:
            //   fan-out  — does one producer feed several consumers?
            //   fan-in   — does one consumer draw from several producers?
            //              (>2 is what `plan_straddle` refuses today)
            //   chain    — is a machine BOTH a consumer and a producer?
            //              that is the stacked-band shape itself.
            let filter: Option<(String, String)> = match (args.get(3), args.get(4)) {
                (Some(a), Some(b)) => Some((a.clone(), b.clone())),
                _ => None,
            };
            let sel: Vec<&Obs> = obs
                .iter()
                .filter(|o| {
                    filter
                        .as_ref()
                        .is_none_or(|(a, b)| &o.producer == a && &o.consumer == b)
                })
                .collect();
            let mut fan_out: BTreeMap<(u32, usize), BTreeSet<usize>> = BTreeMap::new();
            let mut fan_in: BTreeMap<(u32, usize), BTreeSet<usize>> = BTreeMap::new();
            let mut is_producer: BTreeSet<(u32, usize)> = BTreeSet::new();
            let mut is_consumer: BTreeSet<(u32, usize)> = BTreeSet::new();
            for o in &sel {
                fan_out.entry((o.member, o.p_idx)).or_default().insert(o.c_idx);
                fan_in.entry((o.member, o.c_idx)).or_default().insert(o.p_idx);
                is_producer.insert((o.member, o.p_idx));
                is_consumer.insert((o.member, o.c_idx));
            }
            let hist = |m: &BTreeMap<(u32, usize), BTreeSet<usize>>| {
                let mut h: BTreeMap<usize, usize> = BTreeMap::new();
                for v in m.values() {
                    *h.entry(v.len()).or_insert(0) += 1;
                }
                h
            };
            let label = filter
                .as_ref()
                .map(|(a, b)| format!("{a} -> {b}"))
                .unwrap_or_else(|| "ALL pairs".into());
            println!("fan analysis: {label}  ({} observations)", sel.len());
            println!("  fan-OUT (consumers fed by one producer machine):");
            for (k, n) in hist(&fan_out) {
                println!("    {k} consumer(s): {n} producer machines");
            }
            println!("  fan-IN (producers feeding one consumer machine):");
            for (k, n) in hist(&fan_in) {
                println!("    {k} producer(s): {n} consumer machines");
            }
            let chained: Vec<_> = is_producer.intersection(&is_consumer).collect();
            println!(
                "  CHAIN: {} machines are both a DI producer and a DI consumer \
                 (stacked bands) out of {} distinct machines",
                chained.len(),
                is_producer.union(&is_consumer).count()
            );
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


/// One non-DI interface on a machine that receives direct insertion.
struct Face {
    /// Side of the CONSUMER machine this inserter sits on.
    side: &'static str,
    /// True when the inserter drops INTO the machine (an input).
    into_machine: bool,
    reach: i32,
    /// What sits at the inserter's other end: belt, machine, or neither.
    far_kind: &'static str,
}

struct FacePlan {
    /// EVERY side DI arrives on. A single side would be wrong for the
    /// majority case: the fan analysis shows 1,405 of 2,039 cable->EC
    /// consumers straddle TWO producers, so they receive DI on two faces.
    di_sides: Vec<&'static str>,
    others: Vec<Face>,
}

impl FacePlan {
    /// Canonical string so identical arrangements aggregate.
    fn signature(&self) -> String {
        let mut parts: Vec<String> = self
            .others
            .iter()
            .map(|f| {
                format!(
                    "{}:{}{}->{}",
                    f.side,
                    if f.into_machine { "in" } else { "out" },
                    f.reach,
                    f.far_kind
                )
            })
            .collect();
        parts.sort();
        let mut ds = self.di_sides.clone();
        ds.sort_unstable();
        ds.dedup();
        format!("DI@{} | {}", ds.join("+"), parts.join(" "))
    }
}

/// Which side of the machine box at `(mx,my,w,h)` the tile `(tx,ty)` lies on.
fn side_of(mx: i32, my: i32, w: i32, h: i32, tx: i32, ty: i32) -> &'static str {
    if ty < my {
        "N"
    } else if ty >= my + h {
        "S"
    } else if tx < mx {
        "W"
    } else if tx >= mx + w {
        "E"
    } else {
        "?"
    }
}

fn mine_faces(dir: &str, want_p: &str, want_c: &str) -> Vec<FacePlan> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
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
            let mut belt: HashMap<(i32, i32), ()> = HashMap::new();
            for (i, m) in ents.iter().enumerate() {
                if m.name.contains("transport-belt") || m.name.contains("underground-belt") {
                    belt.insert((m.x, m.y), ());
                }
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
            // Consumer machine index -> the side its DI band arrives on.
            let mut di_consumers: HashMap<usize, Vec<&'static str>> = HashMap::new();
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
                let pn = p.recipe.clone().unwrap_or_else(|| p.name.clone());
                let cn = c.recipe.clone().unwrap_or_else(|| c.name.clone());
                if pn != want_p || cn != want_c {
                    continue;
                }
                let (cw, ch) = entity_size(&c.name);
                di_consumers
                    .entry(di)
                    .or_default()
                    .push(side_of(c.x, c.y, cw as i32, ch as i32, ins.x, ins.y));
            }
            // Second pass: every OTHER inserter touching those consumers.
            for (&ci, di_sides) in &di_consumers {
                let c = &ents[ci];
                let (cw, ch) = entity_size(&c.name);
                let mut others = Vec::new();
                for ins in ents {
                    if !is_inserter(&ins.name) {
                        continue;
                    }
                    let (dx, dy) = dir_to_vec(ins.direction);
                    let r = inserter_reach(&ins.name);
                    let drop = (ins.x + dx * r, ins.y + dy * r);
                    let pick = (ins.x - dx * r, ins.y - dy * r);
                    let drops_in = occ.get(&drop) == Some(&ci);
                    let picks_from = occ.get(&pick) == Some(&ci);
                    if !drops_in && !picks_from {
                        continue;
                    }
                    // Skip the DI band itself (both ends are machines).
                    if occ.contains_key(&drop) && occ.contains_key(&pick) {
                        continue;
                    }
                    let far = if drops_in { pick } else { drop };
                    let far_kind = if belt.contains_key(&far) {
                        "belt"
                    } else if occ.contains_key(&far) {
                        "machine"
                    } else {
                        "none"
                    };
                    others.push(Face {
                        side: side_of(c.x, c.y, cw as i32, ch as i32, ins.x, ins.y),
                        into_machine: drops_in,
                        reach: r,
                        far_kind,
                    });
                }
                out.push(FacePlan { di_sides: di_sides.clone(), others });
            }
        }
    }
    out
}
