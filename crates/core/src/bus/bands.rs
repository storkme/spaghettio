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
//! **RFC-058 concluded 2026-07-31 — kill criterion 1 fired.** The phase-4
//! packed builder below is a FALSIFICATION RECORD: flag-gated, default
//! off, never shipped. #523's review found three further latent defects,
//! annotated at their sites and deliberately left unfixed (fixing them
//! cannot change the conclusion — each makes routing stricter or restores
//! dropped transport, moving density further below the bar): the splitter
//! carve can no-op when a later branch re-selects a carved junction; the
//! collector loop skips crossing/UG handling and the foreign-feed filter;
//! and secondary/sorted output-belt rows are not re-stamped, dropping
//! those products' transport (no gate fixture has them).
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

/// One item flow the packed builder must route: a source anchor and one
/// tap per consumer band. Built from the same aggregation facts the bus
/// planner uses (consumers per item, DI-filtered), in packed coordinates.
#[derive(Debug, Clone)]
pub struct PackedNet {
    pub item: String,
    pub is_fluid: bool,
    pub rate: f64,
    /// Producing band indices (all of them — every producer's output must
    /// be collected), or empty for external inputs (west edge).
    pub src_bands: Vec<usize>,
    /// Consuming band indices, deduplicated.
    pub dst_bands: Vec<usize>,
}

/// Aggregate per-item nets over the packed bands. DI-fed inputs are
/// skipped exactly as `plan_bus_lanes` skips them (`RowSpan::di_input`).
pub fn build_packed_nets(
    rows: &[RowSpan],
    contents: &[BandContent],
    solver_result: &crate::models::SolverResult,
) -> Vec<PackedNet> {
    let mut span_to_band: FxHashMap<usize, usize> = FxHashMap::default();
    for (bi, c) in contents.iter().enumerate() {
        for &si in &c.row_indices {
            span_to_band.insert(si, bi);
        }
    }
    let mut producers: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    let mut rates: FxHashMap<&str, f64> = FxHashMap::default();
    let mut fluid: FxHashMap<&str, bool> = FxHashMap::default();
    for (si, rs) in rows.iter().enumerate() {
        for out in &rs.spec.outputs {
            if let Some(&bi) = span_to_band.get(&si) {
                let v = producers.entry(out.item.as_str()).or_default();
                if !v.contains(&bi) {
                    v.push(bi);
                }
            }
            *rates.entry(out.item.as_str()).or_default() +=
                out.rate * rs.machine_count as f64;
            fluid.insert(out.item.as_str(), out.is_fluid);
        }
    }
    for ext in &solver_result.external_inputs {
        rates.entry(ext.item.as_str()).or_insert(ext.rate);
        fluid.entry(ext.item.as_str()).or_insert(ext.is_fluid);
    }
    let mut nets: std::collections::BTreeMap<String, PackedNet> = Default::default();
    for (si, rs) in rows.iter().enumerate() {
        let Some(&bi) = span_to_band.get(&si) else { continue };
        for inp in &rs.spec.inputs {
            if rs.di_input.iter().any(|(item, _)| item == &inp.item) {
                continue;
            }
            let src = producers.get(inp.item.as_str()).cloned().unwrap_or_default();
            if src == vec![bi] {
                continue; // fed within its own band
            }
            let net = nets.entry(inp.item.clone()).or_insert_with(|| PackedNet {
                item: inp.item.clone(),
                is_fluid: *fluid.get(inp.item.as_str()).unwrap_or(&inp.is_fluid),
                rate: *rates.get(inp.item.as_str()).unwrap_or(&0.0),
                src_bands: src.into_iter().filter(|&p| p != bi).collect(),
                dst_bands: Vec::new(),
            });
            if !net.dst_bands.contains(&bi) {
                net.dst_bands.push(bi);
            }
        }
    }
    let mut nets: Vec<PackedNet> = nets.into_values().collect();
    // External outputs get an edge net (empty dst_bands = route to the
    // west edge): the target's output row must physically leave the
    // arrangement, or its west end is a validator-visible dead-end and a
    // sim drain has nothing to collect from.
    for out in &solver_result.external_outputs {
        if let Some(bis) = producers.get(out.item.as_str()) {
            nets.push(PackedNet {
                item: out.item.clone(),
                is_fluid: out.is_fluid,
                rate: out.rate,
                src_bands: bis.clone(),
                dst_bands: Vec::new(),
            });
        }
    }
    nets
}

/// Stamp each band's own belt rows at their TRANSLATED positions: the
/// row templates' inserters pick from the original `input_belt_y` /
/// `output_belt_y` tiles, so the packed feed and output belts must land
/// exactly where those rows translate to — not at rect-relative guesses.
/// Input rows flow EAST (inserters pick anywhere along them), output
/// rows flow WEST toward the corridor pickup at the band's west edge.
pub fn stamp_band_belt_rows(
    rows: &[RowSpan],
    contents: &[BandContent],
    origins: &[(i32, i32)],
    belt_name: &str,
) -> Vec<PlacedEntity> {
    use crate::models::EntityDirection;
    let mut out = Vec::new();
    for (bi, c) in contents.iter().enumerate() {
        let (_rx, ry, rw, _) = c.rect;
        let (ox, oy) = origins[bi];
        // Adjacent spans in one band can SHARE a source belt row (the
        // upper span's output is the lower span's input); collect rows
        // once per y, output taking precedence — inserters pick from the
        // row regardless of its flow direction, and the west flow is what
        // the corridor pickup needs.
        let mut row_dirs: std::collections::BTreeMap<i32, EntityDirection> =
            std::collections::BTreeMap::new();
        for &si in &c.row_indices {
            let rs = &rows[si];
            for &iy in &rs.input_belt_y {
                row_dirs.entry(iy).or_insert(EntityDirection::East);
            }
            row_dirs.insert(rs.output_belt_y, EntityDirection::West);
        }
        for (src_y, dir) in row_dirs {
            if std::env::var("SPAGHETTIO_BANDS_DEBUG").is_ok() {
                eprintln!(
                    "band {bi}: src_y {src_y} -> y {} dir {dir:?} x {}..{}",
                    src_y - ry + oy,
                    ox,
                    ox + rw - 1
                );
            }
            for dx in 0..rw {
                out.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x: ox + dx,
                    y: src_y - ry + oy,
                    direction: dir,
                    ..Default::default()
                });
            }
        }
    }
    out
}

/// Route every packed net as a real corridor: plain belts along the path,
/// an underground pair wherever the path crosses an existing perpendicular
/// belt, sideload termination onto the consumer's feed row. Sources are
/// the producer band's output-row west end, or the arrangement's west
/// edge for externals. Deterministic order: rate descending, then item.
///
/// This is the phase-3 spike's turn-legal, admissible router made real —
/// same crossing rules, but emitting entities instead of counting tiles.
/// Correctness hardening (lane semantics, congestion re-pack) belongs to
/// the layout_pass wiring increment, where the validator sees the result.
pub fn route_packed_nets(
    nets: &[PackedNet],
    rows: &[RowSpan],
    contents: &[BandContent],
    origins: &[(i32, i32)],
    existing: &[PlacedEntity],
    belt_name: &str,
    priority: &[String],
) -> Result<Vec<PlacedEntity>, String> {
    use crate::models::EntityDirection as D;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    const BAND: u8 = 1;
    const H: u8 = 2;
    const V: u8 = 4;
    const TURN: u8 = 8;

    // Occupancy from ENTITY footprints, not band rects: a band's interior
    // belt rows (dual-input templates put one between machine runs) are
    // corridor TARGETS, and rect-blocking made them unreachable — the
    // no-corridor-at-any-gap failure the parity test caught on sci2-ore.
    let mut occ: FxHashMap<(i32, i32), u8> = FxHashMap::default();
    let (mut lo, mut hi) = ((i32::MAX, i32::MAX), (i32::MIN, i32::MIN));
    for e in existing {
        if is_transport(&e.name) {
            let axis = match e.direction {
                D::East | D::West => H,
                _ => V,
            };
            *occ.entry((e.x, e.y)).or_insert(0) |= axis;
        } else {
            let (w, h) = entity_size(&e.name);
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    *occ.entry((e.x + dx, e.y + dy)).or_insert(0) |= BAND;
                }
            }
        }
        lo = (lo.0.min(e.x), lo.1.min(e.y));
        hi = (hi.0.max(e.x), hi.1.max(e.y));
    }
    let (min, max) = ((lo.0 - 6, lo.1 - 6), (hi.0 + 6, hi.1 + 6));

    // Translated belt-row helper.
    let row_y = |bi: usize, src_y: i32| src_y - contents[bi].rect.1 + origins[bi].1;

    // Each band's output-row CONTINUATION stub — (ox-1, y) and (ox-2, y) —
    // is reserved for that band's own nets before any routing: the pickup
    // must physically continue the west-flowing row, and an earlier
    // corridor turning on the stub previously left zero seedable starts.
    let mut reserved: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    for (bi, c) in contents.iter().enumerate() {
        let ox = origins[bi].0;
        for &si in &c.row_indices {
            let y = row_y(bi, rows[si].output_belt_y);
            reserved.insert((ox - 1, y), bi);
            reserved.insert((ox - 2, y), bi);
        }
    }
    let passable = |occ: &FxHashMap<(i32, i32), u8>,
                    t: (i32, i32),
                    horiz: bool,
                    me: Option<usize>| -> bool {
        if t.0 < min.0 || t.0 > max.0 || t.1 < min.1 || t.1 > max.1 {
            return false;
        }
        if let Some(&owner) = reserved.get(&t) {
            if me != Some(owner) {
                return false;
            }
        }
        let b = occ.get(&t).copied().unwrap_or(0);
        b & (BAND | TURN) == 0 && b & (if horiz { H } else { V }) == 0
    };

    // Every belt's direction and cargo, kept current as corridors stamp:
    // a tile a FOREIGN-carrying belt points into is not routable — placing
    // a belt there chains two items head-to-tail (the diagnosed
    // item-isolation class: an ore corridor feeding a plate UG entrance).
    let mut belt_dirs: FxHashMap<(i32, i32), (D, Option<String>)> = existing
        .iter()
        .filter(|e| is_transport(&e.name))
        .map(|e| ((e.x, e.y), (e.direction, e.carries.clone())))
        .collect();

    let mut ordered: Vec<&PackedNet> = nets.iter().collect();
    ordered.sort_by(|a, b| b.rate.total_cmp(&a.rate).then_with(|| a.item.cmp(&b.item)));
    // Negotiated ordering: promoted items route FIRST (the ghost router's
    // rip-up discipline in miniature) — a net that failed last attempt
    // claims its corridor before the nets that walled it in.
    for promo in priority.iter().rev() {
        if let Some(pos) = ordered.iter().position(|n| &n.item == promo) {
            let n = ordered.remove(pos);
            ordered.insert(0, n);
        }
    }

    let mut out: Vec<PlacedEntity> = Vec::new();
    for net in ordered {
        // Starts are the output row's CONTINUATION tiles — reserved for
        // this band above, so they are always seedable and the corridor's
        // first belt physically receives the row's west flow.
        let stubs = |bi: usize| -> Vec<(i32, i32)> {
            let ox = origins[bi].0;
            contents[bi]
                .row_indices
                .iter()
                .flat_map(|&si| {
                    let y = row_y(bi, rows[si].output_belt_y);
                    // ONLY the immediate continuation: seeding at ox-2
                    // left (ox-1, y) unstamped and the row dead-ended
                    // into the hole. ox-2 stays reserved as elbow room.
                    [(ox - 1, y)]
                })
                .collect()
        };
        let primary = net.src_bands.first().copied();
        let starts: Vec<(i32, i32)> = match primary {
            Some(bi) => stubs(bi),
            None => (min.1..=max.1).map(|y| (min.0, y)).collect(),
        };
        // Corridor TREE state: plain straight belts of this net's earlier
        // corridors are junction candidates — a later consumer's branch
        // starts at a splitter carved into the trunk (decision log,
        // 2026-07-31 tree-router entry).
        let mut net_belts: Vec<((i32, i32), D)> = Vec::new();
        let splitter_name = match belt_name {
            "express-transport-belt" => "express-splitter",
            "fast-transport-belt" => "fast-splitter",
            _ => "splitter",
        };
        // An empty dst set is an EDGE net: one corridor to the west edge.
        let dsts: Vec<Option<usize>> = if net.dst_bands.is_empty() {
            vec![None]
        } else {
            net.dst_bands.iter().map(|&d| Some(d)).collect()
        };
        for dst_opt in dsts {
            let mut targets: FxHashSet<(i32, i32)> = FxHashSet::default();
            let dst = match dst_opt {
                None => {
                    for y in min.1..=max.1 {
                        targets.insert((min.0, y));
                    }
                    usize::MAX
                }
                Some(d) => d,
            };
            if dst != usize::MAX {
            for &si in &contents[dst].row_indices {
                for &iy in &rows[si].input_belt_y {
                    let ty = row_y(dst, iy);
                    let (ox, _) = origins[dst];
                    for x in ox..ox + contents[dst].rect.2 {
                        targets.insert((x, ty));
                    }
                    // West CONTINUATION of the east-flowing feed row —
                    // how a real bus feeds a row: the belt starts west of
                    // it. Fresh approach tiles outside the crowded gap
                    // interior; sci2's copper-cable net walled out of
                    // band 5's feed row at every gap without these.
                    for x in ox - 4..ox {
                        targets.insert((x, ty));
                    }
                }
            }
            }
            if targets.is_empty() {
                return Err(format!("net {}: consumer band {dst} has no input rows", net.item));
            }
            let (tx0, tx1) = (
                targets.iter().map(|t| t.0).min().unwrap(),
                targets.iter().map(|t| t.0).max().unwrap(),
            );
            let (ty0, ty1) = (
                targets.iter().map(|t| t.1).min().unwrap(),
                targets.iter().map(|t| t.1).max().unwrap(),
            );
            let hfn = |t: (i32, i32)| {
                (tx0 - t.0).max(t.0 - tx1).max(0) + (ty0 - t.1).max(t.1 - ty1).max(0)
            };
            // Branch entries: for each straight trunk tile j (flow d) with
            // free (j+p, j+p+d) on a perpendicular side p, the entry tile
            // j+p+d may seed a branch — materialization then carves a
            // splitter into the trunk at j covering (j, j+p).
            let mut branch_from: FxHashMap<(i32, i32), ((i32, i32), (i32, i32), D)> =
                FxHashMap::default();
            let mut starts = starts.clone();
            for &(j, d) in &net_belts {
                let dv = match d {
                    D::East => (1, 0),
                    D::West => (-1, 0),
                    D::South => (0, 1),
                    D::North => (0, -1),
                };
                let straight = net_belts.iter().any(|&(t, td)| {
                    t == (j.0 + dv.0, j.1 + dv.1) && td == d
                });
                if !straight {
                    continue;
                }
                for p in [(dv.1, dv.0), (-dv.1, -dv.0)] {
                    let side = (j.0 + p.0, j.1 + p.1);
                    let entry = (side.0 + dv.0, side.1 + dv.1);
                    if passable(&occ, side, true, primary)
                        && passable(&occ, side, false, primary)
                        && passable(&occ, entry, true, primary)
                        && passable(&occ, entry, false, primary)
                    {
                        branch_from.insert(entry, (j, side, d));
                        starts.push(entry);
                    }
                }
            }
            let fed_by_foreign = |t: (i32, i32)| -> bool {
                [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                    let n = (t.0 + dx, t.1 + dy);
                    belt_dirs.get(&n).is_some_and(|(dir, carry)| {
                        let v = match dir {
                            D::East => (1, 0),
                            D::West => (-1, 0),
                            D::South => (0, 1),
                            D::North => (0, -1),
                        };
                        (n.0 + v.0, n.1 + v.1) == t
                            && carry.as_deref().is_some_and(|c| c != net.item)
                    })
                })
            };
            let mut open: BinaryHeap<Reverse<(i32, i32, (i32, i32), bool)>> = BinaryHeap::new();
            let mut best: FxHashMap<((i32, i32), bool), i32> = FxHashMap::default();
            let mut parent: FxHashMap<((i32, i32), bool), ((i32, i32), bool)> =
                FxHashMap::default();
            for &s in &starts {
                for horiz in [true, false] {
                    if passable(&occ, s, horiz, primary) && !fed_by_foreign(s) {
                        best.insert((s, horiz), 0);
                        open.push(Reverse((hfn(s), 0, s, horiz)));
                    }
                }
            }
            let mut found: Option<Vec<(i32, i32)>> = None;
            while let Some(Reverse((_, cost, tile, horiz))) = open.pop() {
                if best.get(&(tile, horiz)).copied().unwrap_or(i32::MAX) < cost {
                    continue;
                }
                // A goal must be belt-free or carry THIS net's item: a
                // foreign-occupied continuation tile can be LANDED on as a
                // crossing, and accepting it as a goal orphaned the branch
                // stub behind a trailing run materialization then skipped
                // (the four sci2 dead-ends).
                let goal_ok = |t: (i32, i32)| {
                    belt_dirs
                        .get(&t)
                        .is_none_or(|(_, c)| c.as_deref() == Some(net.item.as_str()) || c.is_none())
                };
                if targets.contains(&tile) && goal_ok(tile) {
                    let mut path = vec![tile];
                    let mut cur = (tile, horiz);
                    while let Some(&p) = parent.get(&cur) {
                        path.push(p.0);
                        cur = p;
                    }
                    path.reverse();
                    found = Some(path);
                    break;
                }
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nxt = (tile.0 + dx, tile.1 + dy);
                    let nh = dy == 0;
                    if !passable(&occ, nxt, nh, primary) || fed_by_foreign(nxt) {
                        continue;
                    }
                    if nh != horiz && !passable(&occ, tile, nh, primary) {
                        continue;
                    }
                    let nc = cost + 1;
                    if best.get(&(nxt, nh)).copied().unwrap_or(i32::MAX) <= nc {
                        continue;
                    }
                    best.insert((nxt, nh), nc);
                    parent.insert((nxt, nh), (tile, horiz));
                    open.push(Reverse((nc + hfn(nxt), nc, nxt, nh)));
                }
            }
            let Some(path) = found else {
                let seedable = starts
                    .iter()
                    .filter(|&&s| {
                        passable(&occ, s, true, primary)
                            || passable(&occ, s, false, primary)
                    })
                    .count();
                let sample: Vec<_> = starts.iter().take(3).collect();
                return Err(format!(
                    "net {}: no corridor from {:?} to band {dst} \
                     (starts {} seedable of {} {:?}, targets {})",
                    net.item,
                    net.src_bands,
                    seedable,
                    starts.len(),
                    sample,
                    targets.len(),
                ));
            };
            // Materialize with real crossings. A path tile already carrying
            // a belt is never double-stamped: at the FIRST tile that is the
            // pickup adjacency, at the LAST it is the sideload into the
            // feed row, and interior runs are bridged by an underground
            // pair whose entrance/exit replace the plain belts on either
            // side. Turn-legality guarantees crossings are straight, and a
            // run longer than the tier's reach fails the net (gap widens).
            if let Some(&(j, side, d)) = path.first().and_then(|t| branch_from.get(t)) {
                // Carve the junction: the trunk belt at j becomes a
                // splitter whose 2-tile footprint is EXACTLY {j, side} —
                // anchor derived from the oriented dims, and every book
                // (occ, belt_dirs, belt_tiles-equivalent via belt_dirs)
                // updated for BOTH tiles. The earlier anchor-min carve
                // left footprints misaligned and the books stale, which
                // is what orphaned the four sci2 branch stubs: later
                // nets sideloaded into tiles that were physically empty.
                let anchor = match d {
                    D::East | D::West => (j.0, j.1.min(side.1)),
                    _ => (j.0.min(side.0), j.1),
                };
                if let Some(tb) = out
                    .iter_mut()
                    .find(|e| (e.x, e.y) == j && e.name == belt_name)
                {
                    tb.name = splitter_name.to_string();
                    tb.x = anchor.0;
                    tb.y = anchor.1;
                    tb.direction = d;
                }
                for t in [j, side] {
                    *occ.entry(t).or_insert(0) |= BAND;
                    belt_dirs.insert(t, (d, Some(net.item.clone())));
                }
            }
            let mut belt_tiles: FxHashSet<(i32, i32)> = existing
                .iter()
                .chain(out.iter())
                .filter(|e| is_transport(&e.name))
                .map(|e| (e.x, e.y))
                .collect();
            let occupied: Vec<bool> = path.iter().map(|t| belt_tiles.contains(t)).collect();
            let ug_name = match belt_name {
                "express-transport-belt" => "express-underground-belt",
                "fast-transport-belt" => "fast-underground-belt",
                _ => "underground-belt",
            };
            let reach = crate::common::ug_max_reach(belt_name) as usize;
            if std::env::var("SPAGHETTIO_BANDS_DEBUG").is_ok() {
                eprintln!(
                    "route {} -> {:?}: path {:?} occupied {:?}",
                    net.item,
                    dst_opt,
                    path,
                    occupied
                        .iter()
                        .enumerate()
                        .filter(|(_, &o)| o)
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>()
                );
            }
            let mut skip_until = 0usize;
            for (i, &t) in path.iter().enumerate() {
                if i < skip_until {
                    continue;
                }
                if occupied[i] {
                    let run_start = i;
                    let mut j = i;
                    while j < path.len() && occupied[j] {
                        j += 1;
                    }
                    if run_start > 0 && j < path.len() {
                        // Interior run: bridge with a UG pair — entrance
                        // replaces the plain belt just pushed, exit lands
                        // on the first free tile past the run.
                        if j - run_start + 1 > reach {
                            return Err(format!(
                                "net {}: crossing run of {} exceeds {belt_name} reach",
                                net.item,
                                j - run_start
                            ));
                        }
                        // The entrance must be THE belt at path[run_start-1]
                        // — out.last_mut() converted whatever was pushed
                        // most recently, which corrupts an unrelated belt
                        // whenever the predecessor tile was itself skipped
                        // (sideload-through). If the predecessor was never
                        // stamped, this path shape cannot be materialized.
                        let prev_t = path[run_start - 1];
                        let Some(prev) = out
                            .iter_mut()
                            .rev()
                            .find(|e| (e.x, e.y) == prev_t && e.name == belt_name)
                        else {
                            return Err(format!(
                                "net {}: crossing at {:?} follows an unstamped tile",
                                net.item, path[run_start]
                            ));
                        };
                        prev.name = ug_name.to_string();
                        prev.io_type = Some("input".into());
                        let t2 = path[j];
                        let p = path[j - 1];
                        let dir = match (t2.0 - p.0, t2.1 - p.1) {
                            (1, 0) => D::East,
                            (-1, 0) => D::West,
                            (0, 1) => D::South,
                            _ => D::North,
                        };
                        belt_tiles.insert(t2);
                        belt_dirs.insert(t2, (dir, Some(net.item.clone())));
                        out.push(PlacedEntity {
                            name: ug_name.to_string(),
                            x: t2.0,
                            y: t2.1,
                            direction: dir,
                            io_type: Some("output".into()),
                            carries: Some(net.item.clone()),
                            ..Default::default()
                        });
                        skip_until = j + 1;
                    } else {
                        // Boundary run: the first tiles are the pickup
                        // adjacency, the last the sideload — the existing
                        // belt IS the connection; stamp nothing.
                        skip_until = j;
                    }
                    continue;
                }
                belt_tiles.insert(t);
                let dir = if i + 1 < path.len() {
                    let n = path[i + 1];
                    match (n.0 - t.0, n.1 - t.1) {
                        (1, 0) => D::East,
                        (-1, 0) => D::West,
                        (0, 1) => D::South,
                        _ => D::North,
                    }
                } else if i > 0 {
                    let p = path[i - 1];
                    match (t.0 - p.0, t.1 - p.1) {
                        (1, 0) => D::East,
                        (-1, 0) => D::West,
                        (0, 1) => D::South,
                        _ => D::North,
                    }
                } else {
                    D::East
                };
                let axis_in = (i > 0).then(|| path[i - 1].1 == t.1);
                let axis_out = (i + 1 < path.len()).then(|| path[i + 1].1 == t.1);
                let bits = occ.entry(t).or_insert(0);
                match (axis_in, axis_out) {
                    (Some(a), Some(b)) if a != b => *bits |= TURN,
                    (Some(a), _) | (_, Some(a)) => *bits |= if a { H } else { V },
                    _ => {}
                }
                belt_dirs.insert(t, (dir, Some(net.item.clone())));
                net_belts.push((t, dir));
                out.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x: t.0,
                    y: t.1,
                    direction: dir,
                    carries: Some(net.item.clone()),
                    ..Default::default()
                });
            }
        }

        // Collector runs: every ADDITIONAL producer's output merges into
        // the net's trunk (same-item sideload — legal), or its whole
        // output row strands as a dead-end while consumers starve.
        for &extra in net.src_bands.iter().skip(1) {
            let starts = stubs(extra);
            let targets: FxHashSet<(i32, i32)> =
                net_belts.iter().map(|&(t, _)| t).collect();
            if targets.is_empty() {
                continue;
            }
            let (tx0, tx1) = (
                targets.iter().map(|t| t.0).min().unwrap(),
                targets.iter().map(|t| t.0).max().unwrap(),
            );
            let (ty0, ty1) = (
                targets.iter().map(|t| t.1).min().unwrap(),
                targets.iter().map(|t| t.1).max().unwrap(),
            );
            let hfn = |t: (i32, i32)| {
                (tx0 - t.0).max(t.0 - tx1).max(0) + (ty0 - t.1).max(t.1 - ty1).max(0)
            };
            let mut open: BinaryHeap<Reverse<(i32, i32, (i32, i32), bool)>> = BinaryHeap::new();
            let mut best: FxHashMap<((i32, i32), bool), i32> = FxHashMap::default();
            let mut parent: FxHashMap<((i32, i32), bool), ((i32, i32), bool)> =
                FxHashMap::default();
            for &st in &starts {
                for horiz in [true, false] {
                    if passable(&occ, st, horiz, Some(extra)) {
                        best.insert((st, horiz), 0);
                        open.push(Reverse((hfn(st), 0, st, horiz)));
                    }
                }
            }
            let mut found: Option<Vec<(i32, i32)>> = None;
            while let Some(Reverse((_, cost, tile, horiz))) = open.pop() {
                if best.get(&(tile, horiz)).copied().unwrap_or(i32::MAX) < cost {
                    continue;
                }
                if targets.contains(&tile) {
                    let mut path = vec![tile];
                    let mut cur = (tile, horiz);
                    while let Some(&pr) = parent.get(&cur) {
                        path.push(pr.0);
                        cur = pr;
                    }
                    path.reverse();
                    found = Some(path);
                    break;
                }
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nxt = (tile.0 + dx, tile.1 + dy);
                    let nh = dy == 0;
                    if !passable(&occ, nxt, nh, Some(extra)) {
                        continue;
                    }
                    if nh != horiz && !passable(&occ, tile, nh, Some(extra)) {
                        continue;
                    }
                    let nc = cost + 1;
                    if best.get(&(nxt, nh)).copied().unwrap_or(i32::MAX) <= nc {
                        continue;
                    }
                    best.insert((nxt, nh), nc);
                    parent.insert((nxt, nh), (tile, horiz));
                    open.push(Reverse((nc + hfn(nxt), nc, nxt, nh)));
                }
            }
            let Some(path) = found else {
                return Err(format!(
                    "net {}: collector from band {extra} cannot reach the trunk",
                    net.item
                ));
            };
            for (i, &t) in path.iter().enumerate() {
                if i + 1 == path.len() {
                    break; // last tile is ON the trunk — sideload, no stamp
                }
                let n = path[i + 1];
                let dir = match (n.0 - t.0, n.1 - t.1) {
                    (1, 0) => D::East,
                    (-1, 0) => D::West,
                    (0, 1) => D::South,
                    _ => D::North,
                };
                let axis_in = (i > 0).then(|| path[i - 1].1 == t.1);
                let axis_out = Some(n.1 == t.1);
                let bits = occ.entry(t).or_insert(0);
                match (axis_in, axis_out) {
                    (Some(a), Some(b)) if a != b => *bits |= TURN,
                    (Some(a), _) | (_, Some(a)) => *bits |= if a { H } else { V },
                    _ => {}
                }
                belt_dirs.insert(t, (dir, Some(net.item.clone())));
                net_belts.push((t, dir));
                out.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x: t.0,
                    y: t.1,
                    direction: dir,
                    carries: Some(net.item.clone()),
                    ..Default::default()
                });
            }
        }
    }
    Ok(out)
}

/// Phase-4 orchestrator: refuse or build the packed layout. Packs CONTENT
/// rects (pipe rows travel with their band, per the RFC's implementation
/// note), translates band cargo, stamps inserter-aligned belt rows,
/// routes every net, and lays a simple 7-step pole grid (medium poles:
/// 7x7 supply, wire reach 9 > 7 keeps the grid one network). Boundary
/// records: external inputs at each corridor's west-edge entry; the
/// target's record stays on its producer's west-flowing output row —
/// where items physically arrive (the fold's 0.00/s lesson: records must
/// describe real flow, and this one does).
///
/// NOT yet called from layout_pass — the call-site flip is the next
/// increment, together with the phase-2 inertness test's update to the
/// phase-4 contract (flag on = packed layout or typed refusal).
pub fn build_packed_layout(
    rows: &[RowSpan],
    row_entities: &[PlacedEntity],
    solver_result: &crate::models::SolverResult,
    max_belt_tier: Option<&str>,
) -> Result<crate::models::LayoutResult, String> {
    if let Some(r) = packing_refusal(rows, solver_result, max_belt_tier) {
        return Err(format!("packed-refusal: {r}"));
    }
    let bands = extract_bands(rows, row_entities);
    if bands.len() < 3 {
        return Err(format!("packed-refusal: {} bands — below the 3-band floor", bands.len()));
    }
    let contents = band_contents(&bands, rows, row_entities);
    // A band's belt rows can OVERHANG its content rect (an input row two
    // above the top inserter row); pack the FULL footprint — rect grown by
    // each band's top/bottom overhang — or stacked shelves collide exactly
    // where the overlap diagnosis found 21 double-stamped tiles on sci1.
    let overhang: Vec<(i32, i32)> = contents
        .iter()
        .map(|c| {
            let (mut top, mut bot) = (0i32, 0i32);
            for &si in &c.row_indices {
                let rs = &rows[si];
                for &y in rs.input_belt_y.iter().chain([rs.output_belt_y].iter()) {
                    top = top.max(c.rect.1 - y);
                    bot = bot.max(y - (c.rect.1 + c.rect.3 - 1));
                }
            }
            (top, bot)
        })
        .collect();
    let pseudo: Vec<Band> = contents
        .iter()
        .enumerate()
        .map(|(i, c)| Band {
            x: c.rect.0,
            y: c.rect.1,
            w: c.rect.2,
            h: c.rect.3 + overhang[i].0 + overhang[i].1,
            row_indices: c.row_indices.clone(),
            recipes: bands[i].recipes.clone(),
        })
        .collect();
    // The spike's measured lesson (RFC-058 decision log): real fixtures
    // need gap widening once per-band belt rows are reserved — sci2-ore
    // routed only at gap 6. Same loop here: pack, translate, route; any
    // routing failure re-packs the whole arrangement one gap wider.
    let belt = crate::common::belt_entity_for_rate(f64::INFINITY, max_belt_tier);
    let nets = build_packed_nets(rows, &contents, solver_result);
    let mut built: Option<Vec<PlacedEntity>> = None;
    let mut last_err = String::new();
    let mut used_origins: Vec<(i32, i32)> = Vec::new();
    for gap in GAP..=8 {
        let Some(plan) = best_pack(&pseudo, gap, MAX_ASPECT) else {
            last_err = format!("no packing within the aspect cap at gap {gap}");
            continue;
        };
        // Origins place the CONTENT rect: shift each band down by its own
        // top overhang (the packed slot already reserves it), plus a
        // global margin so the top shelf's rows stay in-bounds.
        let origins: Vec<(i32, i32)> = plan
            .positions
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| (x + 2, y + 3 + overhang[i].0))
            .collect();
        let base_entities = translate_band_contents(&contents, &origins, row_entities);
        let mut base = base_entities.clone();
        base.extend(stamp_band_belt_rows(rows, &contents, &origins, belt));
        // Negotiation: on failure, promote the failing net and retry this
        // gap — up to one promotion per net — before widening.
        let mut priority: Vec<String> = Vec::new();
        let mut done = false;
        for _attempt in 0..=nets.len() {
            match route_packed_nets(&nets, rows, &contents, &origins, &base, belt, &priority) {
                Ok(corridors) => {
                    let mut entities = base.clone();
                    entities.extend(corridors);
                    built = Some(entities);
                    used_origins = origins.clone();
                    done = true;
                    break;
                }
                Err(e) => {
                    last_err = format!("gap {gap}: {e}");
                    let failed = nets
                        .iter()
                        .map(|n| n.item.clone())
                        .find(|it| e.contains(&format!("net {it}:")));
                    match failed {
                        Some(it) if !priority.contains(&it) => priority.insert(0, it),
                        _ => break,
                    }
                }
            }
        }
        if done {
            break;
        }
    }
    let Some(mut entities) = built else {
        return Err(format!("packed-refusal: no gap in 2..=8 routes all nets — {last_err}"));
    };
    let origins = used_origins;
    let _ = &mut entities;

    // Pole grid over the arrangement extent, on free tiles only.
    let occupied: FxHashSet<(i32, i32)> = entities
        .iter()
        .flat_map(|e| {
            let (w, h) = entity_size(&e.name);
            (0..w as i32)
                .flat_map(move |dx| (0..h as i32).map(move |dy| (e.x + dx, e.y + dy)))
                .collect::<Vec<_>>()
        })
        .collect();
    let max_x = entities.iter().map(|e| e.x).max().unwrap_or(0);
    let max_y = entities.iter().map(|e| e.y).max().unwrap_or(0);
    let mut poles = Vec::new();
    // Poles stay INSIDE the pre-pole extent: the grid running to +3 with a
    // 3-tile free-tile drift let poles land up to ~6 tiles past the last
    // real entity, inflating the bbox KC1 is measured from, and two
    // drifted neighbours could exceed wire reach (#523 review, both
    // findings). Clamping bounds the drift and the bbox; reach can still
    // be exceeded pathologically — the record's pole grid stays a sketch.
    for gy in (0..=max_y).step_by(7) {
        for gx in (0..=max_x).step_by(7) {
            if let Some(free) = (0..4)
                .flat_map(|dy| (0..4).map(move |dx| (gx + dx, gy + dy)))
                .filter(|&(x, y)| x <= max_x && y <= max_y)
                .find(|t| !occupied.contains(t))
            {
                poles.push(PlacedEntity {
                    name: "medium-electric-pole".to_string(),
                    x: free.0,
                    y: free.1,
                    ..Default::default()
                });
            }
        }
    }
    entities.extend(poles);

    // Footprint-inclusive, origin-normalised dimensions: anchor-only max
    // with no min understated the bbox (review finding on #523) — and this
    // number feeds KC1, so it must be the honest one. West-edge corridors
    // can sit at negative x; shift everything to a 0-origin first.
    let min_x = entities.iter().map(|e| e.x).min().unwrap_or(0);
    let min_y = entities.iter().map(|e| e.y).min().unwrap_or(0);
    if min_x != 0 || min_y != 0 {
        for e in &mut entities {
            e.x -= min_x;
            e.y -= min_y;
        }
    }
    let width = entities
        .iter()
        .map(|e| e.x + entity_size(&e.name).0 as i32)
        .max()
        .unwrap_or(0);
    let height = entities
        .iter()
        .map(|e| e.y + entity_size(&e.name).1 as i32)
        .max()
        .unwrap_or(0);
    let mut boundary_inputs = Vec::new();
    for net in &nets {
        if net.src_bands.is_empty() {
            if let Some(e) = entities
                .iter()
                .filter(|e| e.carries.as_deref() == Some(net.item.as_str()))
                .min_by_key(|e| e.x)
            {
                boundary_inputs.push(crate::models::BoundaryRecord {
                    item: net.item.clone(),
                    x: e.x,
                    y: e.y,
                    direction: e.direction,
                    is_fluid: net.is_fluid,
                    entity: e.name.clone(),
                });
            }
        }
    }
    let mut boundary_outputs = Vec::new();
    for out in &solver_result.external_outputs {
        for (bi, c) in contents.iter().enumerate() {
            for &si in &c.row_indices {
                if rows[si].spec.outputs.iter().any(|o| o.item == out.item) {
                    let y = rows[si].output_belt_y - c.rect.1 + origins[bi].1 - min_y;
                    boundary_outputs.push(crate::models::BoundaryRecord {
                        item: out.item.clone(),
                        x: origins[bi].0 - min_x,
                        y,
                        direction: crate::models::EntityDirection::West,
                        is_fluid: out.is_fluid,
                        entity: belt.to_string(),
                    });
                }
            }
        }
    }

    Ok(crate::models::LayoutResult {
        entities,
        width,
        height,
        boundary_inputs,
        boundary_outputs,
        ..Default::default()
    })
}
