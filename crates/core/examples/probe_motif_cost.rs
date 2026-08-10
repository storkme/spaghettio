//! Phase-0 probes 2+3 for the cell-interface DB idea, one shared layout
//! sweep over the same demand corpus as the census (include!d, cannot
//! drift).
//!
//! Probe 2 — COST BASELINE: what does each demand motif cost in the current
//! engine's output? Attribution is by segment id, written down here rather
//! than improvised: `row:{recipe}:*` and machine entities attribute to that
//! recipe; `di-row:{a}:{b}` attributes to the fused pair (reported
//! separately — the census's edge motifs predicted these).
//!
//! Probe 3 — FABRIC SHARE: the pre-registered RFC-057 kill criterion. Every
//! placed entity is classified interior / fabric / infra / other:
//!   interior: segment `row:*` or `di-row:*` (machines, feed belts,
//!             inserters, row pipes)
//!   fabric:   segment `trunk:*`, `tap*`, `ghost:*`, `balancer:*`
//!             (the inter-row transport the RFC-057 corpse warns about)
//!   infra:    electric poles (placed last, never router obstacles)
//!   other:    anything else — PRINTED PER NAME, never silently pooled
//! Area is entity footprint tiles (`common::entity_size`), so overlapping
//! row bboxes cannot double-count. If fabric dominates at target rates,
//! neat cached interiors cannot move the headline number and the DB pivots
//! to preview + fabric-side motifs; the threshold argument belongs to the
//! RFC, the measurement to this probe.
//!
//! Layout options: engine defaults + the fixture's belt tier — deliberately
//! the vanilla path (no DI forcing, no cell composition), because the
//! question is what the DEFAULT pipeline spends, not what a tuned one can.
use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::common::entity_size;
use spaghettio_core::solver;
use std::collections::BTreeMap;

include!("celldb/corpus.rs");

fn class_of(seg: Option<&str>, name: &str) -> &'static str {
    if name.contains("electric-pole") || name == "substation" {
        return "infra";
    }
    match seg {
        // RFC-061 per-block trunk columns live under the row: prefix but
        // are INTER-ROW transport — `row:{r}:trunk:*`, `row:{r}:trunk-dive:*`,
        // `row:{r}:current-feed:*` (templates.rs:1180-1183). Counting them
        // as interior understated fabric exactly in the belt-saturated
        // regime where the verdict is closest (round-3 review, the one
        // finding that touched the published conclusion).
        Some(s)
            if s.starts_with("row:")
                && s.split(':').nth(2).is_some_and(|k| {
                    k == "trunk" || k == "trunk-dive" || k == "current-feed"
                }) =>
        {
            "fabric"
        }
        Some(s) if s.starts_with("row:") || s.starts_with("di-row:") => "interior",
        Some(s)
            if s.starts_with("trunk:")
                || s.starts_with("tap")
                || s.starts_with("ghost:")
                || s.starts_with("balancer:") =>
        {
            "fabric"
        }
        // Segmentless transport is stamped fabric, not a mystery: balancer
        // stamps carry segment_id: None by construction
        // (balancer_library.rs:84). (Merge-tap branches are NOT segmentless
        // — they emit tap-prefixed ids, ghost_router.rs, and classify as
        // fabric above; an earlier comment here claimed otherwise.) Counted
        // as its own class so the attribution stays visible, but it is
        // fabric for the share headline.
        _ if name.contains("transport-belt")
            || name.contains("underground-belt")
            || name.contains("splitter") =>
        {
            "fabric-stamp"
        }
        _ => "other",
    }
}

fn main() {
    let corpus = corpus();
    let mut seen: FxHashSet<String> = FxHashSet::default();

    // per-motif: (attributed tiles, placed machine entities)
    let mut motif: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    let mut other_names: BTreeMap<String, usize> = BTreeMap::new();
    // name, rate, [interior, fabric, fabric-stamp, infra, other]
    let mut per_layout: Vec<(String, f64, [f64; 5])> = Vec::new();
    let (mut built, mut failed) = (0usize, 0usize);

    for F(item, rate, machine, belt, inputs, excluded) in &corpus {
        // Unlike the census (solver-only; belt is irrelevant there), this
        // probe's entire output depends on the belt tier — it MUST be in
        // the dedupe key or two entries differing only by tier would get
        // one layout silently dropped.
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

        let mut cls = [0f64; 5]; // interior, fabric, fabric-stamp, infra, other
        for e in &l.entities {
            let (w, h) = entity_size(&e.name);
            let a = (w * h) as f64;
            let seg = e.segment_id.as_deref();
            match class_of(seg, &e.name) {
                "interior" => {
                    cls[0] += a;
                    // Attribute to the recipe parsed from the segment id.
                    if let Some(s) = seg {
                        let parts: Vec<&str> = s.split(':').collect();
                        let m = if s.starts_with("di-row:") && parts.len() >= 3 {
                            format!("di:{}+{}", parts[1], parts[2])
                        } else {
                            parts.get(1).unwrap_or(&"?").to_string()
                        };
                        let ent = motif.entry(m).or_default();
                        ent.0 += a;
                        if e.recipe.is_some() {
                            ent.1 += 1;
                        }
                    }
                }
                "fabric" => cls[1] += a,
                "fabric-stamp" => cls[2] += a,
                "infra" => cls[3] += a,
                _ => {
                    cls[4] += a;
                    *other_names.entry(e.name.clone()).or_default() += 1;
                }
            }
        }
        per_layout.push((format!("{item}@{rate}"), *rate, cls));
        // Percentages on the SAME base the headline medians use
        // (interior+fabric; infra excluded) — the row dump and the quoted
        // medians must reconcile without a footnote (round-3 review).
        let ifab = cls[0] + cls[1] + cls[2];
        let fab = cls[1] + cls[2];
        println!(
            "{item}@{rate:<6} interior {:>5.0} ({:>4.1}%)  fabric {:>5.0} ({:>4.1}%, stamps {:>4.0})  infra {:>4.0}  other {:>4.0}   [% of interior+fabric]",
            cls[0], 100.0 * cls[0] / ifab, fab, 100.0 * fab / ifab, cls[2], cls[3], cls[4],
        );
    }

    println!("\n===== {built} layouts built, {failed} failed =====");

    // "Zero unexplained tiles" is ENFORCED, not asserted in prose: any
    // OTHER tile falls outside every share denominator, so a non-empty
    // bucket poisons the headline silently (round-3 review). Refuse.
    let other_total: f64 = per_layout.iter().map(|(_, _, c)| c[4]).sum();
    if other_total > 0.0 {
        println!(
            "ERROR: {other_total:.0} tiles classified OTHER — share denominators exclude"
        );
        println!("       them; fix the classifier before quoting ANY figure below.");
    }
    if per_layout.is_empty() {
        println!("no layouts built — nothing to summarize");
        return;
    }

    // Probe 3 headline: fabric share of the transport-relevant area
    // (interior + fabric incl. stamps; infra excluded from both sides,
    // disclosed here).
    let mut shares: Vec<f64> = per_layout
        .iter()
        .map(|(_, _, c)| 100.0 * (c[1] + c[2]) / (c[0] + c[1] + c[2]))
        .collect();
    shares.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = if shares.len() % 2 == 1 {
        shares[shares.len() / 2]
    } else {
        (shares[shares.len() / 2 - 1] + shares[shares.len() / 2]) / 2.0
    };
    println!(
        "fabric share of interior+fabric area: min {:.1}%  median {:.1}%  max {:.1}%",
        shares.first().unwrap(),
        med,
        shares.last().unwrap()
    );
    // Split by target-rate band — censoring for scale effects.
    for (label, lo, hi) in [("rate <5", 0.0, 5.0), ("5-20", 5.0, 20.0), (">=20", 20.0, f64::MAX)] {
        let mut v: Vec<f64> = per_layout
            .iter()
            .filter(|(_, r, _)| *r >= lo && *r < hi)
            .map(|(_, _, c)| 100.0 * (c[1] + c[2]) / (c[0] + c[1] + c[2]))
            .collect();
        if v.is_empty() {
            continue;
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let m = if v.len() % 2 == 1 {
            v[v.len() / 2]
        } else {
            (v[v.len() / 2 - 1] + v[v.len() / 2]) / 2.0
        };
        println!("  {label:<8} n={:<3} median {m:.1}%  max {:.1}%", v.len(), v.last().unwrap());
    }

    // Probe 2: per-motif attributed interior cost.
    println!("\n--- interior tiles by motif (join against the census's mass shares) ---");
    let mut rows: Vec<(&String, &(f64, usize))> = motif.iter().collect();
    rows.sort_by(|a, b| b.1 .0.partial_cmp(&a.1 .0).unwrap());
    println!("{:<40} {:>10} {:>9} {:>12}", "motif", "tiles", "machines", "tiles/machine");
    for (m, (tiles, machines)) in rows {
        println!(
            "{m:<40} {tiles:>10.0} {machines:>9} {:>12.1}",
            if *machines > 0 { tiles / *machines as f64 } else { f64::NAN }
        );
    }

    if !other_names.is_empty() {
        println!("\n--- OTHER (unclassified — every name listed, nothing pooled) ---");
        for (n, c) in &other_names {
            println!("  {c:>5}  {n}");
        }
    }
}
