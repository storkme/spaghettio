//! RFC-058 band extraction and packing (phases 1–2).
//!
//! A band is a maximal run of rows containing machines or inserters — one
//! recipe's machine row plus the inserter rows serving it. Belt rows are
//! deliberately not band structure: they are the transport RFC-058 re-plans.
//!
//! Extraction is placer-native (phase 1): the placer's `RowSpan`s are the
//! grouping authority — each band records which spans contribute to it,
//! the linkage phase 4's lane planner needs — while geometry is measured
//! from the spans' own machine and inserter entities, footprint-aware.
//! `probe_band_packing_headroom` (tests/cell_composition.rs) keeps its
//! deliberately decoupled y-projection copy as the oracle; the parity test
//! `rfc058_placer_bands_match_y_projection` pins the two against each
//! other, and the CI premise guard keeps a third self-contained copy so a
//! defect here cannot blind it. The duplication is intentional.
//!
//! Packing (phase 2) is the phase-0 probe's shelf packer, ported verbatim:
//! target width swept from the widest band to twice the control width,
//! source and height-descending order, minimum bounding-box area under an
//! aspect cap, strict `<` so the first minimum wins. It emits positions
//! only — `layout_pass` records them in a `BandPackingPlanned` trace event
//! and NOTHING consumes them yet. Kill criterion 1 cleared its spike on
//! 2026-07-31 at −35.9% against a −33.0% bar and stays armed through
//! phase 4; this module deliberately cannot move an entity.

use crate::bus::placer::RowSpan;
use crate::common::{entity_size, is_machine_entity};
use crate::models::PlacedEntity;
use rustc_hash::{FxHashMap, FxHashSet};

/// The probe's aspect cap. Phase 0 swept 3.0–4.0 and the applicable-fixture
/// count did not move, so 3:1 stands (RFC-058 decision log, 2026-07-31).
pub const MAX_ASPECT: f64 = 3.0;
/// Inter-band gap in tiles, both axes — the phase-0 probe's value. The
/// phase-3 spike measured real fixtures needing up to 6 once per-band belt
/// rows are reserved; the gap becomes a planner concern in phase 4.
pub const GAP: i32 = 2;

/// One band: a maximal machine+inserter row run, with its placer linkage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Band {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// Indices into the placer's `RowSpan` list that contribute structural
    /// rows to this band. Usually one; more when rows fuse with no belt
    /// row between them (direct-insertion bridges).
    pub row_indices: Vec<usize>,
    /// Sorted, deduplicated recipes of the contributing spans.
    pub recipes: Vec<String>,
}

fn is_structural(e: &PlacedEntity) -> bool {
    is_machine_entity(&e.name) || e.name.contains("inserter")
}

/// Placer-native band extraction (RFC-058 phase 1).
///
/// Structural rows are computed footprint-aware from machine and inserter
/// entities; contiguous structural runs are collected *within each span's
/// y-range* and runs that touch across span boundaries merge into one
/// band. Restricting runs to span ranges is what makes the spans the
/// grouping authority; the merge step preserves the y-projection's maximal
/// runs where DI has fused adjacent rows.
pub fn extract_bands(rows: &[RowSpan], entities: &[PlacedEntity]) -> Vec<Band> {
    let mut structural_ys: FxHashSet<i32> = FxHashSet::default();
    for e in entities {
        if is_structural(e) {
            let (_, eh) = entity_size(&e.name);
            for dy in 0..eh as i32 {
                structural_ys.insert(e.y + dy);
            }
        }
    }

    // Contiguous structural runs, per span.
    struct Run {
        y0: i32,
        y1: i32, // inclusive
        span: usize,
    }
    let mut runs: Vec<Run> = Vec::new();
    for (i, rs) in rows.iter().enumerate() {
        let mut y = rs.y_start;
        while y < rs.y_end {
            if !structural_ys.contains(&y) {
                y += 1;
                continue;
            }
            let y0 = y;
            while y < rs.y_end && structural_ys.contains(&y) {
                y += 1;
            }
            runs.push(Run { y0, y1: y - 1, span: i });
        }
    }
    runs.sort_by_key(|r| r.y0);

    // Merge runs that touch or overlap across span boundaries.
    let mut merged: Vec<(i32, i32, Vec<usize>)> = Vec::new();
    for r in runs {
        if let Some(last) = merged.last_mut() {
            if r.y0 <= last.1 + 1 {
                last.1 = last.1.max(r.y1);
                last.2.push(r.span);
                continue;
            }
        }
        merged.push((r.y0, r.y1, vec![r.span]));
    }

    // Geometry per merged run: x-extent of the structural entities whose
    // anchor row falls inside it (same predicate as the probe, so the
    // parity test compares like with like).
    let mut bands = Vec::new();
    for (y0, y1, spans) in merged {
        let (mut xmin, mut xmax) = (i32::MAX, i32::MIN);
        for e in entities {
            if !is_structural(e) || e.y < y0 || e.y > y1 {
                continue;
            }
            let (ew, _) = entity_size(&e.name);
            xmin = xmin.min(e.x);
            xmax = xmax.max(e.x + ew as i32 - 1);
        }
        if xmin > xmax {
            continue;
        }
        let mut recipes: Vec<String> =
            spans.iter().map(|&i| rows[i].spec.recipe.clone()).collect();
        recipes.sort();
        recipes.dedup();
        bands.push(Band {
            x: xmin,
            y: y0,
            w: xmax - xmin + 1,
            h: y1 - y0 + 1,
            row_indices: spans,
            recipes,
        });
    }
    bands
}

/// Bounding box (w, h) over band rectangles.
pub fn bbox(bands: &[Band]) -> (i32, i32) {
    let w = bands.iter().map(|b| b.x + b.w).max().unwrap_or(0)
        - bands.iter().map(|b| b.x).min().unwrap_or(0);
    let h = bands.iter().map(|b| b.y + b.h).max().unwrap_or(0)
        - bands.iter().map(|b| b.y).min().unwrap_or(0);
    (w, h)
}

/// One shelf-packing pass at a fixed target width. Identical construction
/// to the phase-0 probe's — see the module docs for why the probe keeps
/// its own copy.
fn shelf_pack(bands: &[Band], target_w: i32, gap: i32, sort_desc: bool) -> Vec<Band> {
    let mut idx: Vec<usize> = (0..bands.len()).collect();
    if sort_desc {
        idx.sort_by_key(|&i| std::cmp::Reverse((bands[i].h, bands[i].w, i)));
    }
    let mut out = bands.to_vec();
    let (mut x, mut y, mut shelf_h) = (0, 0, 0);
    for &i in &idx {
        let (w, h) = (bands[i].w, bands[i].h);
        if x > 0 && x + w > target_w {
            x = 0;
            y += shelf_h + gap;
            shelf_h = 0;
        }
        out[i].x = x;
        out[i].y = y;
        x += w + gap;
        shelf_h = shelf_h.max(h);
    }
    out
}

/// A packing the aspect cap admits.
#[derive(Debug, Clone)]
pub struct PackedPlan {
    pub w: i32,
    pub h: i32,
    /// Planned (x, y) origin per band, index-aligned with the extraction
    /// order of the `Band` slice the plan was computed from.
    pub positions: Vec<(i32, i32)>,
}

/// Best aspect-capped shelf packing: minimum bounding-box area under
/// `max_aspect`, target width swept from the widest band to twice the
/// control width, both orders, strict `<` so the first minimum wins
/// (iteration order is part of the published-numbers contract). `None`
/// when nothing fits the cap — a width-dominant band.
pub fn best_pack(bands: &[Band], gap: i32, max_aspect: f64) -> Option<PackedPlan> {
    let (cw, _) = bbox(bands);
    let widest = bands.iter().map(|b| b.w).max().unwrap_or(1);
    let mut best: Option<(i64, PackedPlan)> = None;
    for sort_desc in [false, true] {
        let mut t = widest;
        while t <= cw.max(widest) * 2 {
            let packed = shelf_pack(bands, t, gap, sort_desc);
            let (w, h) = bbox(&packed);
            let aspect = w.max(h) as f64 / w.min(h).max(1) as f64;
            if aspect <= max_aspect {
                let area = (w as i64) * (h as i64);
                if best.as_ref().is_none_or(|(ba, _)| area < *ba) {
                    best = Some((
                        area,
                        PackedPlan {
                            w,
                            h,
                            positions: packed.iter().map(|b| (b.x, b.y)).collect(),
                        },
                    ));
                }
            }
            t += 2;
        }
    }
    best.map(|(_, plan)| plan)
}

/// RFC-058 phase 2 entry point, called from `layout_pass` when
/// `LayoutOptions.band_packing` is on: extract, pack, and record the plan
/// as a trace event. Deliberately returns nothing — positions live in the
/// trace and nowhere else until phase 4 exists.
pub fn plan_band_packing(rows: &[RowSpan], entities: &[PlacedEntity]) {
    let bands = extract_bands(rows, entities);
    let widest = bands.iter().map(|b| b.w).max().unwrap_or(0);
    if bands.len() < 3 {
        crate::trace::emit(crate::trace::TraceEvent::BandPackingRefused {
            bands: bands.len(),
            widest_band: widest,
            reason: "fewer than 3 bands — nothing to pack".into(),
        });
        return;
    }
    let (control_w, control_h) = bbox(&bands);
    match best_pack(&bands, GAP, MAX_ASPECT) {
        Some(plan) => {
            let aspect10 =
                (plan.w.max(plan.h) as i64 * 10) / (plan.w.min(plan.h).max(1) as i64);
            crate::trace::emit(crate::trace::TraceEvent::BandPackingPlanned {
                band_rects: bands.iter().map(|b| (b.x, b.y, b.w, b.h)).collect(),
                control_w,
                control_h,
                packed_w: plan.w,
                packed_h: plan.h,
                aspect10,
                positions: plan.positions,
            });
        }
        None => {
            crate::trace::emit(crate::trace::TraceEvent::BandPackingRefused {
                bands: bands.len(),
                widest_band: widest,
                reason: format!(
                    "no target width packs within {MAX_ASPECT}:1 — width-dominant band"
                ),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 4: packed layout construction. Design and scope: the "Phase 4
// design" section of docs/rfc-058-band-packing.md. The packed candidate
// REFUSES anything it does not model and the native bus ships instead —
// the refusals below are the complete list, each typed so an abstention
// names its cause (the FoldRefusal discipline).
// ---------------------------------------------------------------------------

/// Why the packed builder abstained. Emitted in the `BandPackingRefused`
/// trace event's reason; the native pipeline runs unchanged afterward.
#[derive(Debug, Clone, PartialEq)]
pub enum PackRefusal {
    /// Any item's aggregate rate exceeds one lane of the belt tier — the
    /// linear bus handles this with balancer families, which are a
    /// 1D-trunk concept the packed planner deliberately does not model.
    MultiLaneItem { item: String, rate: f64, lane_cap: f64 },
    /// `RowLayout::HorizontalStack` rows carry K stacked trunks with
    /// their own lane-planner contract; out of scope.
    HorizontalStackRow { row: usize },
    /// Partitioned modules (`module_id > 0`) key lanes per module; the
    /// packed planner models one net per item only.
    PartitionedModule { item: String },
}

impl std::fmt::Display for PackRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackRefusal::MultiLaneItem { item, rate, lane_cap } => write!(
                f,
                "item {item} at {rate:.2}/s exceeds one {lane_cap:.1}/s lane — balancer families are out of packed scope"
            ),
            PackRefusal::HorizontalStackRow { row } => {
                write!(f, "row {row} uses RowLayout::HorizontalStack — out of packed scope")
            }
            PackRefusal::PartitionedModule { item } => {
                write!(f, "item {item} is partitioned (module_id > 0) — out of packed scope")
            }
        }
    }
}

/// The complete refusal check for the packed path, run before any
/// geometry work. Returns the FIRST refusal in a deterministic order
/// (rows, then items alphabetically) or `None` when the packed planner
/// models everything present.
pub fn packing_refusal(
    rows: &[RowSpan],
    solver_result: &crate::models::SolverResult,
    max_belt_tier: Option<&str>,
) -> Option<PackRefusal> {
    for (i, rs) in rows.iter().enumerate() {
        if rs.horizontal_stack.is_some() {
            return Some(PackRefusal::HorizontalStackRow { row: i });
        }
    }
    let mut items: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    // External inputs need a lane each too — an over-capacity external
    // would need the same balancer families produced items would.
    for ext in &solver_result.external_inputs {
        if !ext.is_fluid {
            *items.entry(ext.item.clone()).or_default() += ext.rate;
        }
    }
    for rs in rows {
        for io in rs.spec.inputs.iter().chain(rs.spec.outputs.iter()) {
            if io.module_id != 0 {
                return Some(PackRefusal::PartitionedModule { item: io.item.clone() });
            }
        }
        for out in &rs.spec.outputs {
            if !out.is_fluid {
                *items.entry(out.item.clone()).or_default() +=
                    out.rate * rs.machine_count as f64;
            }
        }
    }
    let default_cap = crate::bus::lane_planner::LANE_CAPACITY_TABLE
        .last()
        .map(|(_, c)| *c)
        .unwrap_or(15.0);
    let lane_cap = max_belt_tier
        .and_then(|tier| {
            crate::bus::lane_planner::LANE_CAPACITY_TABLE
                .iter()
                .find(|(name, _)| *name == tier)
                .map(|(_, cap)| *cap)
        })
        .unwrap_or(default_cap);
    for (item, rate) in items {
        if rate > lane_cap {
            return Some(PackRefusal::MultiLaneItem { item, rate, lane_cap });
        }
    }
    None
}

/// The rigid content of one band for phase-4 translation: every source
/// row its spans carry EXCEPT belt rows (transport, re-planned) — pipe
/// rows and any other non-belt span content travel with the band. See
/// the RFC's "content rect" implementation note (2026-07-31).
#[derive(Debug, Clone)]
pub struct BandContent {
    /// Source rows (ys) that translate rigidly with this band.
    pub content_ys: Vec<i32>,
    /// Content rect in source coordinates: (x, y, w, h) over non-belt,
    /// non-pole entities anchored in `content_ys`.
    pub rect: (i32, i32, i32, i32),
    pub row_indices: Vec<usize>,
}

fn is_transport(name: &str) -> bool {
    name.ends_with("transport-belt")
        || name.ends_with("underground-belt")
        || name.ends_with("splitter")
}

/// Compute each band's rigid content. A span's non-belt rows are
/// assigned to the nearest of the span's own bands (a span can split
/// into several bands when a belt row divides its structural runs).
pub fn band_contents(
    bands: &[Band],
    rows: &[RowSpan],
    entities: &[PlacedEntity],
) -> Vec<BandContent> {
    let mut belt_ys_per_span: Vec<FxHashSet<i32>> = Vec::with_capacity(rows.len());
    for rs in rows {
        let mut s: FxHashSet<i32> = rs.input_belt_y.iter().copied().collect();
        s.insert(rs.output_belt_y);
        if let Some((_, y)) = &rs.secondary_output_belt {
            s.insert(*y);
        }
        for (_, y) in &rs.sorted_output_belts {
            s.insert(*y);
        }
        belt_ys_per_span.push(s);
    }

    // span index -> bands that contain it, for nearest-band assignment.
    let mut span_bands: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
    for (bi, b) in bands.iter().enumerate() {
        for &si in &b.row_indices {
            span_bands[si].push(bi);
        }
    }

    let mut content_ys: Vec<Vec<i32>> = vec![Vec::new(); bands.len()];
    for (si, rs) in rows.iter().enumerate() {
        if span_bands[si].is_empty() {
            continue;
        }
        for y in rs.y_start..rs.y_end {
            if belt_ys_per_span[si].contains(&y) {
                continue;
            }
            let owner = span_bands[si]
                .iter()
                .copied()
                .min_by_key(|&bi| {
                    let b = &bands[bi];
                    (b.y - y).max(y - (b.y + b.h - 1)).max(0)
                })
                .unwrap();
            content_ys[owner].push(y);
        }
    }

    bands
        .iter()
        .enumerate()
        .map(|(bi, b)| {
            let mut ys = std::mem::take(&mut content_ys[bi]);
            ys.sort_unstable();
            ys.dedup();
            let yset: FxHashSet<i32> = ys.iter().copied().collect();
            let (mut xmin, mut xmax, mut ymin, mut ymax) =
                (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
            for e in entities {
                if is_transport(&e.name) || e.name.contains("electric-pole") {
                    continue;
                }
                if !yset.contains(&e.y) {
                    continue;
                }
                let (ew, eh) = entity_size(&e.name);
                xmin = xmin.min(e.x);
                xmax = xmax.max(e.x + ew as i32 - 1);
                ymin = ymin.min(e.y);
                ymax = ymax.max(e.y + eh as i32 - 1);
            }
            if xmin > xmax {
                // Degenerate (no entities): fall back to the structural rect.
                (xmin, xmax, ymin, ymax) = (b.x, b.x + b.w - 1, b.y, b.y + b.h - 1);
            }
            BandContent {
                content_ys: ys,
                rect: (xmin, ymin, xmax - xmin + 1, ymax - ymin + 1),
                row_indices: b.row_indices.clone(),
            }
        })
        .collect()
}

/// Rigidly translate each band's content entities so content-rect origins
/// land on `origins[i]` (packed coordinates). Transport and poles are NOT
/// carried — belts are re-planned and poles re-placed by the packed
/// builder. Returns the translated entities.
pub fn translate_band_contents(
    contents: &[BandContent],
    origins: &[(i32, i32)],
    entities: &[PlacedEntity],
) -> Vec<PlacedEntity> {
    let mut y_to_band: FxHashMap<i32, usize> = FxHashMap::default();
    for (bi, c) in contents.iter().enumerate() {
        for &y in &c.content_ys {
            y_to_band.insert(y, bi);
        }
    }
    let mut out = Vec::new();
    for e in entities {
        if is_transport(&e.name) || e.name.contains("electric-pole") {
            continue;
        }
        let Some(&bi) = y_to_band.get(&e.y) else { continue };
        let (rx, ry, _, _) = contents[bi].rect;
        let (ox, oy) = origins[bi];
        let mut moved = e.clone();
        moved.x = e.x - rx + ox;
        moved.y = e.y - ry + oy;
        out.push(moved);
    }
    out
}
