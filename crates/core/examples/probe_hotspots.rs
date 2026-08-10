//! Hotspot probe (RFC-067 follow-up): WHERE does layout area actually go,
//! and which motifs/fabric classes hold the biggest reclaimable prize?
//!
//! Same demand corpus as the Phase-0 probes (include!d, cannot drift), same
//! segment-id attribution rules as probe_motif_cost — but a different
//! question. The cost probe measured interior-vs-fabric *shares*; this one
//! builds the ranked target list for donor ingestion (the K67-3 reopening
//! path: the DB only pays if it holds cells the engine cannot produce, so
//! which cell is worth hand-crafting first?).
//!
//! Area budget per layout, every bbox tile in exactly one class:
//!   machines   — crafting-machine footprints (the incompressible floor)
//!   overhead   — interior minus machines (row belts, inserters, row pipes)
//!   fabric     — inter-row transport, reported BY KIND (trunk, tap, ghost,
//!                balancer, merger, crossing, junction, feed, row-trunk,
//!                segmentless stamps)
//!   infra      — electric poles
//!   whitespace — bbox minus occupied (measured nowhere else in the repo),
//!                decomposed per scanline into stripe / gutter / ragged /
//!                ug-shadow / hole (definitions at the decomposition site)
//!
//! "Prize" columns rank, they do not promise: overhead = attributed
//! interior tiles − machine tiles. A real cell still needs belts and
//! inserters, so the prize is an upper bound on what a perfect donor could
//! reclaim — how much IS reclaimable is the donor's job to prove under the
//! never-worse + sim gates (candidate_runner), not this probe's.
//!
//! Layout options: engine defaults + the fixture's belt tier — the vanilla
//! path, same rationale as probe_motif_cost.
//!
//! Citable output + prior-adjudication map (read before proposing a fix
//! for the void this measures): docs/hotspot-scoreboard-2026-08.md.
use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::common::{is_machine_entity, oriented_entity_dims};
use spaghettio_core::solver;
use std::collections::BTreeMap;

include!("celldb/corpus.rs");

/// Attribution: identical decision tree to probe_motif_cost::class_of
/// (round-3/4 review lessons baked in), refined to return the fabric KIND
/// instead of pooling everything into one "fabric" bucket.
fn class_of(seg: Option<&str>, name: &str) -> &'static str {
    if name.contains("electric-pole") || name == "substation" {
        return "infra";
    }
    match seg {
        // RFC-061 per-block trunk columns: row:-prefixed but inter-row.
        Some(s)
            if s.starts_with("row:")
                && s.split(':').nth(2).is_some_and(|k| {
                    k == "trunk" || k == "trunk-dive" || k == "current-feed"
                }) =>
        {
            "fab:row-trunk"
        }
        Some(s)
            if s.starts_with("row:")
                || s.starts_with("di-row:")
                || s.starts_with("di-cell:")
                || s.starts_with("di-bridge:") =>
        {
            "interior"
        }
        Some(s) if s.starts_with("trunk:") => "fab:trunk",
        Some(s) if s.starts_with("tap") => "fab:tap",
        Some(s) if s.starts_with("ghost:") => "fab:ghost",
        Some(s) if s.starts_with("balancer:") => "fab:balancer",
        Some(s) if s.starts_with("merger:") => "fab:merger",
        Some(s) if s.starts_with("crossing:") => "fab:crossing",
        Some(s) if s.starts_with("junction:") => "fab:junction",
        Some(s) if s.starts_with("feed:") || s.starts_with("feeder:") => "fab:feed",
        Some(s) if s.starts_with("fan") => "fab:fan",
        // Cell-composition export drain (cells/chain.rs `out:{seg}` — the
        // RFC-051 candidate path, which can win corpus layouts): the
        // final-product run to the drain row. Export transport = fabric.
        // Previously absorbed as fab:stamp by the segment-blind belt
        // fallback; surfaced the moment that arm went None-only.
        Some(s) if s.starts_with("out:") => "fab:out",
        // Segmentless transport = balancer-library stamps (segment_id: None
        // by construction, balancer_library.rs:84). None-only on purpose: a
        // belt under an UNRECOGNIZED prefix must fall to "other" and refuse,
        // not be silently absorbed as a stamp (review finding — the same
        // trap probe_motif_cost's round-4 fixed; inactive prefixes like
        // `cc:b:`/`corridor:` exist in-tree and would otherwise mislabel).
        None if name.contains("transport-belt")
            || name.contains("underground-belt")
            || name.contains("splitter") =>
        {
            "fab:stamp"
        }
        _ => "other",
    }
}

fn motif_key(seg: &str) -> String {
    let parts: Vec<&str> = seg.split(':').collect();
    if (seg.starts_with("di-row:") || seg.starts_with("di-cell:") || seg.starts_with("di-bridge:"))
        && parts.len() >= 3
    {
        format!("di:{}+{}", parts[1], parts[2])
    } else {
        parts.get(1).unwrap_or(&"?").to_string()
    }
}

fn median(v: &mut Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() {
        return f64::NAN;
    }
    if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        (v[v.len() / 2 - 1] + v[v.len() / 2]) / 2.0
    }
}

#[derive(Default, Clone)]
struct MotifAgg {
    tiles: f64,        // all interior tiles attributed to the motif
    machine_tiles: f64, // crafting-machine footprint within those
    machines: usize,
    layouts: usize,
}

fn main() {
    let corpus = corpus();
    let mut seen: FxHashSet<String> = FxHashSet::default();

    let mut motif: BTreeMap<String, MotifAgg> = BTreeMap::new();
    let mut fabric_kind: BTreeMap<&'static str, f64> = BTreeMap::new();
    let mut other_names: BTreeMap<String, usize> = BTreeMap::new();
    // name, bbox, machines, overhead, fabric, infra, whitespace,
    // [stripe, gutter, ragged, ug, hole]
    let mut per_layout: Vec<(String, f64, f64, f64, f64, f64, f64, [f64; 5])> = Vec::new();
    let (mut built, mut failed) = (0usize, 0usize);
    let mut other_total = 0f64;
    // ug-shadow's denominator, printed so a 0.0% share is checkable
    // against "how many shadow tiles exist at all" rather than mysterious.
    let (mut ug_all, mut ug_empty) = (0usize, 0usize);

    for F(item, rate, machine, belt, inputs, excluded) in &corpus {
        // Belt tier changes the layout — it MUST be in the dedupe key
        // (probe_motif_cost round-1 lesson).
        let key = format!("{item}|{rate}|{machine}|{belt:?}|{inputs:?}|{excluded:?}");
        if !seen.insert(key) {
            continue;
        }
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let excl: FxHashSet<String> = excluded.iter().map(|s| s.to_string()).collect();
        let Ok(sr) = solver::solve_with_exclusions(item, *rate, &input_set, machine, &excl)
        else {
            println!("SOLVE-REFUSED {item}@{rate}");
            failed += 1;
            continue;
        };
        let opts = LayoutOptions {
            max_belt_tier: belt.map(|s| s.to_string()),
            ..Default::default()
        };
        let l = match layout::build_bus_layout(&sr, opts) {
            Ok(l) => l,
            Err(e) => {
                println!("LAYOUT-FAILED {item}@{rate}: {e:?}");
                failed += 1;
                continue;
            }
        };
        built += 1;

        // Rasterize occupied tiles (oriented dims — same convention as
        // celldb::extract_unit) so whitespace is bbox − occupied, not a
        // sum-of-footprints guess.
        let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut interior_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        let mut cls: BTreeMap<&'static str, f64> = BTreeMap::new();
        let mut layout_motifs: FxHashSet<String> = FxHashSet::default();
        for e in &l.entities {
            let (w, h) = oriented_entity_dims(&e.name, e.direction);
            let is_interior = class_of(e.segment_id.as_deref(), &e.name) == "interior";
            for dx in 0..w {
                for dy in 0..h {
                    occupied.insert((e.x + dx, e.y + dy));
                    if is_interior {
                        interior_tiles.insert((e.x + dx, e.y + dy));
                    }
                }
            }
            min_x = min_x.min(e.x);
            min_y = min_y.min(e.y);
            max_x = max_x.max(e.x + w - 1);
            max_y = max_y.max(e.y + h - 1);
            let a = (w * h) as f64;
            let seg = e.segment_id.as_deref();
            let c = class_of(seg, &e.name);
            *cls.entry(c).or_default() += a;
            if c == "interior" {
                if let Some(s) = seg {
                    let m = motif.entry(motif_key(s)).or_default();
                    m.tiles += a;
                    if is_machine_entity(&e.name) && e.recipe.is_some() {
                        m.machine_tiles += a;
                        m.machines += 1;
                    }
                    layout_motifs.insert(motif_key(s));
                }
            } else if c == "other" {
                *other_names
                    .entry(format!("{} seg={:?}", e.name, e.segment_id))
                    .or_default() += 1;
                other_total += a;
            } else if c != "infra" {
                *fabric_kind.entry(c).or_default() += a;
            }
        }
        for mk in &layout_motifs {
            motif.get_mut(mk).unwrap().layouts += 1;
        }

        let bbox = ((max_x - min_x + 1) as f64) * ((max_y - min_y + 1) as f64);
        let white = bbox - occupied.len() as f64;
        // The doc's reconciliation claim is enforced here, not narrated:
        // class totals must equal the occupied-tile union exactly, or an
        // entity overlap is silently inflating every published share
        // (review finding, 3/3 — the assertion the doc cited did not
        // previously cover the classes).
        let class_sum: f64 = cls.values().sum();
        assert!(
            (class_sum - occupied.len() as f64).abs() < 0.5,
            "class totals ({class_sum}) != occupied union ({}) — overlapping footprints",
            occupied.len()
        );
        // UG hidden segments: the tiles between a paired entrance/exit are
        // NOT placeable in Factorio, so an empty tile there is not a
        // packable hole. Counted as their own kind (review finding).
        let mut ug_shadow: FxHashSet<(i32, i32)> = FxHashSet::default();
        for (a, b) in spaghettio_core::connectivity::build_ug_pairs(&l.entities) {
            if a.1 == b.1 {
                for x in a.0.min(b.0) + 1..a.0.max(b.0) {
                    ug_shadow.insert((x, a.1));
                }
            } else if a.0 == b.0 {
                for y in a.1.min(b.1) + 1..a.1.max(b.1) {
                    ug_shadow.insert((a.0, y));
                }
            }
        }
        ug_all += ug_shadow.len();
        ug_empty += ug_shadow.iter().filter(|t| !occupied.contains(*t)).count();
        // Decompose whitespace by scanline — distinct kinds imply distinct
        // fixes, so pooling them would hide which one applies:
        //   stripe — fully-empty scanline (vertical squeeze)
        //   gutter — scanline occupied ONLY by fabric/infra, no interior
        //            (inter-band corridor; a trunk-only line is this, not
        //            ragged — review finding, 2/3)
        //   ragged — outside an interior-bearing scanline's occupied span
        //            (row-width variance)
        //   ug     — inside the span, empty, under a UG hidden segment
        //            (not placeable — not a packable hole)
        //   hole   — inside the span, empty, placeable. Approximation
        //            disclosed in the doc: the single-span model folds
        //            gaps between same-line clusters into this kind.
        let (mut w_stripe, mut w_gutter, mut w_ragged, mut w_ug, mut w_hole) =
            (0f64, 0f64, 0f64, 0f64, 0f64);
        let bw = (max_x - min_x + 1) as f64;
        for y in min_y..=max_y {
            let xs: Vec<i32> = (min_x..=max_x).filter(|x| occupied.contains(&(*x, y))).collect();
            let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) else {
                w_stripe += bw;
                continue;
            };
            if !(min_x..=max_x).any(|x| interior_tiles.contains(&(x, y))) {
                w_gutter += bw - xs.len() as f64;
                continue;
            }
            w_ragged += bw - (hi - lo + 1) as f64;
            for x in lo..=hi {
                if !occupied.contains(&(x, y)) {
                    if ug_shadow.contains(&(x, y)) {
                        w_ug += 1.0;
                    } else {
                        w_hole += 1.0;
                    }
                }
            }
        }
        assert!(
            (w_stripe + w_gutter + w_ragged + w_ug + w_hole - white).abs() < 0.5,
            "whitespace decomposition must reconcile"
        );
        // HOTSPOT_PROFILE=item@rate — eyes-on check for the ragged claim:
        // one bar per 4 scanlines, span vs fill, so "the bbox is set by one
        // wide row" is something you can SEE, not just a percentage.
        if std::env::var("HOTSPOT_PROFILE").as_deref() == Ok(&format!("{item}@{rate}")) {
            println!("--- fill profile {item}@{rate} (bbox {}x{}) ---", bw as i64, max_y - min_y + 1);
            for y in (min_y..=max_y).step_by(4) {
                let xs: Vec<i32> =
                    (min_x..=max_x).filter(|x| occupied.contains(&(*x, y))).collect();
                let (span, fill) = match (xs.first(), xs.last()) {
                    (Some(lo), Some(hi)) => ((hi - lo + 1) as usize, xs.len()),
                    _ => (0, 0),
                };
                let scale = |v: usize| (v as f64 / bw * 100.0).round() as usize;
                let bar: String = (0..100)
                    .map(|i| {
                        if i < scale(fill) { '#' } else if i < scale(span) { '.' } else { ' ' }
                    })
                    .collect();
                println!("y{:>4} |{bar}| span {span} fill {fill}", y - min_y);
            }
        }
        let interior = cls.get("interior").copied().unwrap_or(0.0);
        let mach: f64 = l
            .entities
            .iter()
            .filter(|e| {
                is_machine_entity(&e.name)
                    && e.recipe.is_some()
                    && class_of(e.segment_id.as_deref(), &e.name) == "interior"
            })
            .map(|e| {
                let (w, h) = oriented_entity_dims(&e.name, e.direction);
                (w * h) as f64
            })
            .sum();
        let fab: f64 = cls
            .iter()
            .filter(|(k, _)| k.starts_with("fab:"))
            .map(|(_, v)| v)
            .sum();
        let infra = cls.get("infra").copied().unwrap_or(0.0);
        per_layout.push((
            format!("{item}@{rate}"),
            bbox,
            mach,
            interior - mach,
            fab,
            infra,
            white,
            [w_stripe, w_gutter, w_ragged, w_ug, w_hole],
        ));
        println!(
            "{item}@{rate:<6} bbox {bbox:>6.0}  machines {:>4.1}%  overhead {:>4.1}%  fabric {:>4.1}%  infra {:>3.1}%  whitespace {:>4.1}%",
            100.0 * mach / bbox,
            100.0 * (interior - mach) / bbox,
            100.0 * fab / bbox,
            100.0 * infra / bbox,
            100.0 * white / bbox,
        );
    }

    println!("\n===== {built} layouts built, {failed} failed =====");

    // Same refusal contract as probe_motif_cost: an OTHER tile falls outside
    // every denominator, so refuse to print quotable figures — names first.
    if other_total > 0.0 {
        println!("ERROR: {other_total:.0} tiles classified OTHER — fix the classifier.");
        for (n, c) in &other_names {
            println!("  {c:>5}  {n}");
        }
        std::process::exit(2);
    }
    if per_layout.is_empty() {
        println!("no layouts built — nothing to summarize");
        return;
    }

    // ---- Pooled area budget (sum of tiles / sum of bbox across corpus) ----
    let tb: f64 = per_layout.iter().map(|r| r.1).sum();
    let (tm, tov, tf, ti, tw) = per_layout.iter().fold((0.0, 0.0, 0.0, 0.0, 0.0), |a, r| {
        (a.0 + r.2, a.1 + r.3, a.2 + r.4, a.3 + r.5, a.4 + r.6)
    });
    println!("\n--- pooled area budget ({} layouts, {tb:.0} bbox tiles) ---", per_layout.len());
    println!("machines   {tm:>8.0}  ({:>4.1}%)", 100.0 * tm / tb);
    println!("overhead   {tov:>8.0}  ({:>4.1}%)   [interior non-machine]", 100.0 * tov / tb);
    println!("fabric     {tf:>8.0}  ({:>4.1}%)", 100.0 * tf / tb);
    println!("infra      {ti:>8.0}  ({:>4.1}%)", 100.0 * ti / tb);
    println!("whitespace {tw:>8.0}  ({:>4.1}%)", 100.0 * tw / tb);
    let mut wsh: Vec<f64> = per_layout.iter().map(|r| 100.0 * r.6 / r.1).collect();
    println!("whitespace share per layout: median {:.1}%", median(&mut wsh));
    let kinds = per_layout.iter().fold([0.0f64; 5], |mut a, r| {
        for (acc, v) in a.iter_mut().zip(r.7.iter()) {
            *acc += v;
        }
        a
    });
    println!(
        "whitespace kind (pooled): stripe {:.1}%  gutter {:.1}%  ragged {:.1}%  ug-shadow {:.1}%  hole {:.1}%   [of whitespace]",
        100.0 * kinds[0] / tw,
        100.0 * kinds[1] / tw,
        100.0 * kinds[2] / tw,
        100.0 * kinds[3] / tw,
        100.0 * kinds[4] / tw
    );
    println!(
        "ug hidden-segment tiles: {ug_all} total, {ug_empty} unoccupied ({:.0} counted ug-shadow in interior spans; the rest lie in gutter/outside-span lines)",
        kinds[3]
    );

    // ---- Hotspot table 1: per-motif prize (pooled) ----
    println!("\n--- motif prize table (overhead = interior − machines; upper bound, see header) ---");
    let mut rows: Vec<(&String, &MotifAgg)> = motif.iter().collect();
    rows.sort_by(|a, b| {
        (b.1.tiles - b.1.machine_tiles)
            .partial_cmp(&(a.1.tiles - a.1.machine_tiles))
            .unwrap()
    });
    // "mach-tiles" not "machines": the column is the FOOTPRINT in tiles;
    // the entity count only appears via ovh/m (review finding — a reader
    // comparing 4,500 against ovh/m 16.1 would misread it as a count).
    println!(
        "{:<34} {:>8} {:>10} {:>9} {:>7} {:>8} {:>8}",
        "motif", "tiles", "mach-tiles", "overhead", "ovh/m", "ovh%", "layouts"
    );
    for (m, a) in rows {
        let ovh = a.tiles - a.machine_tiles;
        println!(
            "{m:<34} {:>8.0} {:>10.0} {:>9.0} {:>7.1} {:>7.1}% {:>8}",
            a.tiles,
            a.machine_tiles,
            ovh,
            if a.machines > 0 { ovh / a.machines as f64 } else { f64::NAN },
            100.0 * ovh / a.tiles,
            a.layouts,
        );
    }

    // ---- Hotspot table 2: fabric by kind (pooled) ----
    println!("\n--- fabric by kind (pooled) ---");
    let mut fk: Vec<(&&str, &f64)> = fabric_kind.iter().collect();
    fk.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    for (k, v) in fk {
        println!("{k:<16} {v:>8.0}  ({:>4.1}% of fabric)", 100.0 * *v / tf);
    }

    // ---- Hotspot table 3: worst layouts ----
    println!("\n--- worst layouts by whitespace share ---");
    let mut byw = per_layout.clone();
    byw.sort_by(|a, b| (b.6 / b.1).partial_cmp(&(a.6 / a.1)).unwrap());
    for r in byw.iter().take(8) {
        println!("{:<28} bbox {:>6.0}  whitespace {:>4.1}%", r.0, r.1, 100.0 * r.6 / r.1);
    }
    println!("\n--- worst layouts by fabric share of bbox ---");
    let mut byf = per_layout.clone();
    byf.sort_by(|a, b| (b.4 / b.1).partial_cmp(&(a.4 / a.1)).unwrap());
    for r in byf.iter().take(8) {
        println!("{:<28} bbox {:>6.0}  fabric {:>4.1}%", r.0, r.1, 100.0 * r.4 / r.1);
    }
}
