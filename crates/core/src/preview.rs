//! Interface-first preview (RFC-067, consumer 1): turn a `SolverResult`
//! into placed BOXES with ports — the shape of a layout before any interior
//! exists. No correctness stakes: the preview is an estimate by contract,
//! and its calibration against realized layouts is *measured* by
//! `examples/celldb_preview_calibration.rs` (kill criterion K67-2: >30%
//! median total-area error ships the preview disabled).
//!
//! Box sizing: a celldb entry for the motif gives exact recipe-specific
//! geometry (tiles and aspect from a real engine fragment, scaled by
//! machine count). Uncached motifs fall back to
//! `machine footprint × INTERIOR_FACTOR` — one constant, not a per-recipe
//! table, so the fallback cannot silently become a hand-maintained shadow
//! database. The layout mirrors the bus's shape: boxes stack vertically in
//! dependency order beside a trunk column whose width scales with the
//! number of distinct bussed items.
use crate::celldb::{self, Motif};
use crate::common::{belt_throughput, entity_size};
use crate::models::SolverResult;
use serde::{Deserialize, Serialize};

/// Interior tiles per machine-footprint tile for motifs with no DB entry.
/// Phase-0 measured 1.4–2.8 across the corpus (17.0–25.1 interior tiles on
/// 3×3 machines, 35 on 5×5 refineries); the calibration probe adjudicates
/// whether one constant suffices (K67-2), so tune it there, not here.
const INTERIOR_FACTOR: f64 = 2.2;

/// Vertical gap between boxes — the tap/inserter band the bus inserts
/// between rows.
const ROW_GAP: i32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewBox {
    pub recipe: String,
    pub machine: String,
    pub count: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// True when the geometry came from a celldb entry (scaled), false for
    /// the INTERIOR_FACTOR fallback — rendered differently so an estimate
    /// never masquerades as a measured shape.
    pub cached: bool,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewLayout {
    pub boxes: Vec<PreviewBox>,
    /// Trunk column width (tiles) reserved on the left edge.
    pub trunk_w: i32,
    pub width: i32,
    pub height: i32,
}

pub fn preview_boxes(sr: &SolverResult, max_belt_tier: Option<&str>) -> PreviewLayout {
    // Trunk lanes are PHYSICS, not one-per-item: an item flowing above its
    // belt tier's capacity needs ceil(rate/capacity) parallel lanes (plus
    // the balancers the engine will stamp to split them — approximated by
    // the same count). This is what the first calibration rounds missed
    // hardest: the belt-saturated fixtures need 3 lanes per item on yellow.
    let cap = belt_throughput(max_belt_tier.unwrap_or("transport-belt"));
    let mut item_rates: std::collections::BTreeMap<&str, f64> = Default::default();
    for f in sr.external_inputs.iter().filter(|f| !f.is_fluid) {
        *item_rates.entry(f.item.as_str()).or_default() += f.rate;
    }
    for m in &sr.machines {
        for o in m.outputs.iter().filter(|o| !o.is_fluid) {
            *item_rates.entry(o.item.as_str()).or_default() += o.rate;
        }
    }
    let lanes: i32 = item_rates.values().map(|r| (r / cap).ceil().max(1.0) as i32).sum();
    let trunk_w = 2 + lanes;

    let mut boxes = Vec::new();
    let mut y = 0i32;
    let mut max_w = 0i32;
    for m in &sr.machines {
        let count = m.count.ceil().max(1.0) as u32;
        let hits = celldb::query_unit(&m.recipe, &m.entity, 1, None);
        let (w, h, cached) = if let Some(e) = hits.first() {
            // Scale the entry's real geometry by machine count: keep its
            // per-machine tile cost and its height (rows grow horizontally
            // in the engine), derive width.
            let entry_count = match &e.motif {
                Motif::Unit { count, .. } => *count,
                Motif::Fused { count_a, count_b, .. } => count_a + count_b,
            }
            .max(1);
            let tiles =
                e.metrics.interior_tiles as f64 * (count as f64 / entry_count as f64);
            let h = e.metrics.bbox_h;
            (((tiles / h as f64).ceil() as i32).max(1), h, true)
        } else {
            let (mw, mh) = entity_size(&m.entity);
            let tiles = (mw * mh) as f64 * INTERIOR_FACTOR * count as f64;
            let h = mh as i32 + 2; // machine row + feed belt bands
            (((tiles / h as f64).ceil() as i32).max(1), h, false)
        };
        boxes.push(PreviewBox {
            recipe: m.recipe.clone(),
            machine: m.entity.clone(),
            count,
            x: trunk_w,
            y,
            w,
            h,
            cached,
            inputs: m.inputs.iter().map(|f| f.item.clone()).collect(),
            outputs: m.outputs.iter().map(|f| f.item.clone()).collect(),
        });
        max_w = max_w.max(w);
        y += h + ROW_GAP;
    }
    // Non-interior allowance. Boxes model interiors only; realized layouts
    // additionally carry fabric (Phase-0 median 17.8% of interior+fabric,
    // `probe_motif_cost`) and infra (~5%), so non-interior area is ~23% of
    // the realized total: scale by 1/(1-0.23) ≈ 1.30, uniform. A
    // rate-banded version was tried and made calibration WORSE (band
    // medians hide the >=20/s band's huge variance — see the calibration
    // probe's log in the RFC decision trail); the uniform factor is both
    // simpler and better, and its provenance is Phase-0's measurement, not
    // the calibration target. The +4/+2 margins are pole bands and
    // perimeter.
    const NON_INTERIOR_MULT: f64 = 1.30;
    let height = ((y - ROW_GAP).max(0) as f64 * NON_INTERIOR_MULT).ceil() as i32 + 4;
    PreviewLayout { boxes, trunk_w, width: trunk_w + max_w + 2, height }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashSet;

    #[test]
    fn preview_covers_every_machine_group_and_nests_boxes() {
        let inputs: FxHashSet<String> =
            ["iron-plate", "copper-plate"].iter().map(|s| s.to_string()).collect();
        let sr = crate::solver::solve("electronic-circuit", 10.0, &inputs, "assembling-machine-2")
            .expect("fixture solves");
        let p = preview_boxes(&sr, None);
        assert_eq!(p.boxes.len(), sr.machines.len());
        for b in &p.boxes {
            assert!(b.w > 0 && b.h > 0);
            assert!(b.x >= p.trunk_w);
            assert!(b.x + b.w <= p.width);
            assert!(b.y + b.h <= p.height);
        }
        // Seeded motifs must come from the DB, not the fallback.
        assert!(
            p.boxes
                .iter()
                .filter(|b| b.recipe == "electronic-circuit" || b.recipe == "copper-cable")
                .all(|b| b.cached),
            "seeded motifs should resolve from celldb"
        );
    }
}
