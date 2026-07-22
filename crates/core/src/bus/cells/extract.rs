//! Cell extraction (RFC-051 Phase A, lifted from the Phase-1 harness):
//! the engine-as-cell-generator bootstrap plus the segment-crop +
//! contiguity-port derivation, verbatim from
//! `tests/cell_composition.rs` at the #365 merge (parity is the Phase-A
//! gate — geometry changes are NOT allowed in the lift).
//!
//! Panic policy: `generate_cell_layout` panics on solve/layout failure,
//! matching the harness it was lifted from. Phase B (the
//! `CellComposedCandidate`) converts these to `Result` at the candidate
//! boundary — candidates must fail soft in the decomposition search.

use rustc_hash::FxHashSet;
use crate::bus::layout;
use crate::common::QualityTier;
use crate::models::LayoutResult;
use crate::recipe_db::MachinePalette;
use crate::solver;

/// Generate a single-recipe layout via the full engine pipeline — the
/// engine-as-cell-generator bootstrap path (RFC-048 Design).
pub fn generate_cell_layout(
    item: &str,
    rate: f64,
    inputs: &[&str],
) -> (crate::models::SolverResult, LayoutResult) {
    let inputs: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve_with_palette_exclusions_and_quality(
        item,
        rate,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap_or_else(|e| panic!("solve {item}: {e}"));
    let l = layout::build_bus_layout(&sr, layout::LayoutOptions::default())
        .unwrap_or_else(|e| panic!("layout {item}: {e}"));
    (sr, l)
}

/// A cell port derived from a kept belt's terminal stub.
#[derive(Debug, Clone)]
pub struct Port {
    /// "W" or "E" (Phase 1 composes horizontally-fed vertical stacks).
    pub edge: &'static str,
    /// Terminal tile (cell-local): the westmost tile of an in-run /
    /// eastmost of an out-run — corridors attach at x±1.
    pub x: i32,
    /// y in cell-local coordinates.
    pub y: i32,
    pub item: String,
    /// true = flow into the cell.
    pub inbound: bool,
}

/// A pre-verified cell: engine-generated row machinery, cropped to the
/// `row:*` segment partition (+ machines and poles), normalized to
/// (0,0), with ports derived from belt terminal stubs.
#[derive(Debug, Clone)]
pub struct Cell {
    pub entities: Vec<crate::models::PlacedEntity>,
    pub width: i32,
    pub height: i32,
    pub ports: Vec<Port>,
}

/// Crop an engine layout to its cell interior: keep entities whose
/// segment is `row:*` or `pole`, plus machines (recipe carriers).
/// Everything trunk/tap/ghost/merger-shaped is bus machinery, shed.
pub fn extract_cell(l: &LayoutResult) -> Cell {
    use crate::common::{is_belt_entity, is_machine_entity};
    let keep: Vec<_> = l
        .entities
        .iter()
        .filter(|e| {
            let seg = e.segment_id.as_deref().unwrap_or("");
            seg.starts_with("row:") || seg == "pole" || is_machine_entity(&e.name)
        })
        .cloned()
        .collect();
    let min_x = keep.iter().map(|e| e.x).min().unwrap_or(0);
    let min_y = keep.iter().map(|e| e.y).min().unwrap_or(0);
    let mut entities = keep;
    for e in &mut entities {
        e.x -= min_x;
        e.y -= min_y;
    }
    let width = entities.iter().map(|e| e.x).max().unwrap_or(0) + 1;
    let height = entities.iter().map(|e| e.y).max().unwrap_or(0) + 1;

    // Ports: for each row:*:belt-in/belt-out segment, the terminal stub
    // in its flow direction (in-belts: westmost tile; out-belts:
    // eastmost tile). Phase 1 rows all flow W->E.
    let mut ports: Vec<Port> = Vec::new();
    let mut segs: std::collections::BTreeMap<String, Vec<&crate::models::PlacedEntity>> =
        Default::default();
    for e in &entities {
        if let Some(seg) = e.segment_id.as_deref() {
            let is_pipe = e.name == "pipe" || e.name == "pipe-to-ground";
            if (is_belt_entity(&e.name) || is_pipe)
                && (seg.contains(":belt-in") || seg.contains(":belt-out"))
            {
                segs.entry(seg.to_string()).or_default().push(e);
            }
        }
    }
    // Contiguity grouping (decision log 2026-07-22): a segment can span
    // multiple internal rows (the placer splits machine groups), so one
    // port per CONNECTED RUN of same-segment belt tiles, at the run's
    // flow-direction terminal.
    for (seg, belts) in &segs {
        let inbound = seg.contains(":belt-in");
        let tiles: std::collections::BTreeSet<(i32, i32)> =
            belts.iter().map(|e| (e.x, e.y)).collect();
        let mut seen: std::collections::BTreeSet<(i32, i32)> = Default::default();
        for &start in &tiles {
            if seen.contains(&start) {
                continue;
            }
            // flood-fill this run (4-adjacency)
            let mut run = vec![start];
            let mut stack = vec![start];
            seen.insert(start);
            while let Some((x, y)) = stack.pop() {
                for n in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if tiles.contains(&n) && seen.insert(n) {
                        run.push(n);
                        stack.push(n);
                    }
                }
            }
            let &(px, py) = if inbound {
                run.iter().min_by_key(|(x, _)| *x).unwrap()
            } else {
                run.iter().max_by_key(|(x, _)| *x).unwrap()
            };
            let item = belts
                .iter()
                .find(|e| (e.x, e.y) == (px, py))
                .and_then(|e| e.carries.clone())
                .unwrap_or_default();
            let _ = seg;
            ports.push(Port {
                edge: if inbound { "W" } else { "E" },
                x: px,
                y: py,
                item,
                inbound,
            });
        }
    }
    ports.sort_by_key(|p| (p.edge, p.y));
    Cell { entities, width, height, ports }
}
