//! Density scoring for bus layouts.
//!
//! Given a [`LayoutResult`] and a target aspect ratio, compute how tightly the
//! layout packs its entities into an axis-aligned bounding rectangle whose
//! width:height matches the target ratio.
//!
//! Filled area counts every entity's full footprint (e.g. a 3×3 assembler
//! contributes 9 tiles). Only the entities themselves contribute — power-pole
//! coverage areas, wire connections, and `LayoutResult.connections` are
//! ignored. Overlap is not expected; if the sum of footprints exceeds the
//! rectangle area, [`DensityScore::filled_exceeds_rect`] is set so the caller
//! can surface the anomaly rather than silently clamp.

use crate::common::{machine_dims, is_machine_entity};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity};

/// Axis-aligned footprint (in tiles) of a single entity.
///
/// Splitters are 2×1 or 1×2 depending on their flow direction; machines use
/// [`machine_dims`]; beacons are 3×3; everything else is 1×1.
pub fn entity_footprint(entity: &PlacedEntity) -> (u32, u32) {
    let name = entity.name.as_str();
    if is_machine_entity(name) {
        return machine_dims(name);
    }
    if name == "beacon" {
        return (3, 3);
    }
    if matches!(name, "splitter" | "fast-splitter" | "express-splitter") {
        return match entity.direction {
            EntityDirection::North | EntityDirection::South => (2, 1),
            EntityDirection::East | EntityDirection::West => (1, 2),
        };
    }
    // Belts, underground belts, inserters, poles, pipes, pipe-to-ground, and
    // any other single-tile entity default to 1×1.
    (1, 1)
}

/// Result of scoring a layout against a target aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DensityScore {
    /// Width of the smallest aspect-ratio-matching rectangle that contains
    /// every entity footprint.
    pub rect_width: u32,
    /// Height of that rectangle.
    pub rect_height: u32,
    /// Total area of the rectangle (`rect_width * rect_height`).
    pub rect_area: u64,
    /// Sum of entity footprint areas (width × height per entity).
    pub filled_tiles: u64,
    /// `rect_area - filled_tiles` (saturating at zero if filled exceeds rect,
    /// which would indicate overlapping entities).
    pub empty_tiles: u64,
    /// `filled_tiles / rect_area` as a fraction in `[0.0, 1.0]` (or above 1.0
    /// if entities overlap).
    pub density: f64,
    /// Content bounding box width — the tight axis-aligned rectangle around
    /// the actual entity footprints, before aspect-ratio padding.
    pub content_bbox_width: u32,
    /// Content bounding box height (tight, pre-padding).
    pub content_bbox_height: u32,
    /// Set to `true` if `filled_tiles > rect_area`, which implies overlapping
    /// entity footprints — a bug in the layout engine worth surfacing.
    pub filled_exceeds_rect: bool,
}

impl DensityScore {
    /// Zero-entity layout score — all fields 0, density 0.
    pub const EMPTY: DensityScore = DensityScore {
        rect_width: 0,
        rect_height: 0,
        rect_area: 0,
        filled_tiles: 0,
        empty_tiles: 0,
        density: 0.0,
        content_bbox_width: 0,
        content_bbox_height: 0,
        filled_exceeds_rect: false,
    };
}

/// Score a layout's tile-packing density against a target aspect ratio.
///
/// The aspect ratio is given as `(width, height)` in integer units. Default
/// usage is `(1, 1)` for a square rectangle. Higher ratios like `(16, 9)`
/// stretch the rectangle horizontally.
///
/// Steps:
/// 1. Compute the tight content bbox over every entity footprint (each entity
///    contributes the tiles `[x, x + w) × [y, y + h)` from
///    [`entity_footprint`]).
/// 2. Pad one axis outward to match the aspect ratio. The padded side length
///    is `max(bbox_w * ratio_h, bbox_h * ratio_w)`, scaled back to per-axis
///    widths. For 1:1 this reduces to `max(bbox_w, bbox_h)` on both axes.
/// 3. Sum entity footprint areas for the filled count.
/// 4. Density = filled / rect_area.
///
/// Empty layouts yield [`DensityScore::EMPTY`].
pub fn score_density(layout: &LayoutResult, aspect_ratio: (u32, u32)) -> DensityScore {
    if layout.entities.is_empty() {
        return DensityScore::EMPTY;
    }

    // 1. Tight content bbox over entity footprints.
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut filled_tiles: u64 = 0;

    for e in &layout.entities {
        let (w, h) = entity_footprint(e);
        let ex0 = e.x;
        let ey0 = e.y;
        let ex1 = e.x + w as i32; // exclusive
        let ey1 = e.y + h as i32; // exclusive
        if ex0 < min_x { min_x = ex0; }
        if ey0 < min_y { min_y = ey0; }
        if ex1 > max_x { max_x = ex1; }
        if ey1 > max_y { max_y = ey1; }
        filled_tiles += w as u64 * h as u64;
    }

    let bbox_w = (max_x - min_x) as u32;
    let bbox_h = (max_y - min_y) as u32;

    // 2. Pad to match aspect ratio. Guard against a zero-denominator ratio by
    //    falling back to a 1:1 square.
    let (rw, rh) = aspect_ratio;
    let (rect_w, rect_h) = if rw == 0 || rh == 0 {
        let side = bbox_w.max(bbox_h);
        (side, side)
    } else {
        aspect_padded(bbox_w, bbox_h, rw, rh)
    };

    let rect_area = rect_w as u64 * rect_h as u64;
    let empty_tiles = rect_area.saturating_sub(filled_tiles);
    let density = if rect_area > 0 {
        filled_tiles as f64 / rect_area as f64
    } else {
        0.0
    };
    let filled_exceeds_rect = filled_tiles > rect_area;

    DensityScore {
        rect_width: rect_w,
        rect_height: rect_h,
        rect_area,
        filled_tiles,
        empty_tiles,
        density,
        content_bbox_width: bbox_w,
        content_bbox_height: bbox_h,
        filled_exceeds_rect,
    }
}

/// Expand `(bbox_w, bbox_h)` to the smallest rectangle with `rw:rh` aspect
/// that still contains the bbox.
fn aspect_padded(bbox_w: u32, bbox_h: u32, rw: u32, rh: u32) -> (u32, u32) {
    // We need w/h == rw/rh with w >= bbox_w and h >= bbox_h.
    // Let k = max(bbox_w / rw, bbox_h / rh) rounded up. Then w = k*rw, h = k*rh.
    let k_from_w = div_ceil(bbox_w, rw);
    let k_from_h = div_ceil(bbox_h, rh);
    let k = k_from_w.max(k_from_h);
    (k * rw, k * rh)
}

fn div_ceil(a: u32, b: u32) -> u32 {
    if b == 0 { 0 } else { a.div_ceil(b) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, LayoutResult, PlacedEntity};

    fn belt(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".into(),
            x,
            y,
            direction: EntityDirection::East,
            ..Default::default()
        }
    }

    fn assembler(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "assembling-machine-2".into(),
            x,
            y,
            direction: EntityDirection::North,
            ..Default::default()
        }
    }

    #[test]
    fn entity_footprint_recycler_is_non_square() {
        // Fulgora recycler: 2 wide × 4 tall (rfc-fulgora-scrap Phase 0).
        // A square-assuming footprint helper would return (4, 4) or (2, 2).
        let recycler = PlacedEntity {
            name: "recycler".into(),
            x: 0,
            y: 0,
            direction: EntityDirection::North,
            ..Default::default()
        };
        assert_eq!(entity_footprint(&recycler), (2, 4));
    }

    fn splitter_ns(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "splitter".into(),
            x,
            y,
            direction: EntityDirection::North,
            ..Default::default()
        }
    }

    #[test]
    fn empty_layout_returns_empty_score() {
        let layout = LayoutResult::default();
        let s = score_density(&layout, (1, 1));
        assert_eq!(s, DensityScore::EMPTY);
    }

    #[test]
    fn single_belt_is_1x1_square() {
        let layout = LayoutResult {
            entities: vec![belt(5, 7)],
            ..Default::default()
        };
        let s = score_density(&layout, (1, 1));
        assert_eq!(s.content_bbox_width, 1);
        assert_eq!(s.content_bbox_height, 1);
        assert_eq!(s.rect_width, 1);
        assert_eq!(s.rect_height, 1);
        assert_eq!(s.filled_tiles, 1);
        assert_eq!(s.empty_tiles, 0);
        assert!((s.density - 1.0).abs() < 1e-9);
        assert!(!s.filled_exceeds_rect);
    }

    #[test]
    fn assembler_contributes_full_3x3_footprint() {
        let layout = LayoutResult {
            entities: vec![assembler(0, 0)],
            ..Default::default()
        };
        let s = score_density(&layout, (1, 1));
        assert_eq!(s.filled_tiles, 9);
        assert_eq!(s.rect_width, 3);
        assert_eq!(s.rect_height, 3);
        assert!((s.density - 1.0).abs() < 1e-9);
    }

    #[test]
    fn splitter_north_is_2x1() {
        let layout = LayoutResult {
            entities: vec![splitter_ns(10, 20)],
            ..Default::default()
        };
        let s = score_density(&layout, (1, 1));
        assert_eq!(s.filled_tiles, 2);
        assert_eq!(s.content_bbox_width, 2);
        assert_eq!(s.content_bbox_height, 1);
        assert_eq!(s.rect_width, 2);
        assert_eq!(s.rect_height, 2); // padded to square
        assert_eq!(s.empty_tiles, 2);
    }

    #[test]
    fn wide_row_of_belts_pads_to_square_for_1to1() {
        // 10 belts in a row → bbox 10×1, square is 10×10, filled=10, density=0.1.
        let entities: Vec<PlacedEntity> = (0..10).map(|i| belt(i, 0)).collect();
        let layout = LayoutResult { entities, ..Default::default() };
        let s = score_density(&layout, (1, 1));
        assert_eq!(s.filled_tiles, 10);
        assert_eq!(s.rect_width, 10);
        assert_eq!(s.rect_height, 10);
        assert!((s.density - 0.1).abs() < 1e-9);
    }

    #[test]
    fn aspect_ratio_16_by_9() {
        // 32×9 bbox fits in 32×18 at 16:9 (ceil(32/16)=2 → 32×18).
        let entities: Vec<PlacedEntity> = (0..32)
            .flat_map(|x| (0..9).map(move |y| belt(x, y)))
            .collect();
        let layout = LayoutResult { entities, ..Default::default() };
        let s = score_density(&layout, (16, 9));
        assert_eq!(s.content_bbox_width, 32);
        assert_eq!(s.content_bbox_height, 9);
        assert_eq!(s.rect_width, 32);
        assert_eq!(s.rect_height, 18);
    }

    #[test]
    fn zero_aspect_ratio_falls_back_to_square() {
        let layout = LayoutResult {
            entities: vec![belt(0, 0), belt(2, 5)],
            ..Default::default()
        };
        let s = score_density(&layout, (0, 1));
        assert_eq!(s.rect_width, s.rect_height);
    }
}
