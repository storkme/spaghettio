//! RFC-048 Phase 1: cell catalog + stamper + composition harness.
//!
//! Test-only (never wired into production layout paths — kill 4: cells
//! are produced BY the engine, cropped and composed here; there is one
//! layout stack). See `docs/rfc-048-cell-composition.md`.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::layout;
use spaghettio_core::common::QualityTier;
use spaghettio_core::models::LayoutResult;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;

/// Generate a single-recipe layout via the full engine pipeline — the
/// engine-as-cell-generator bootstrap path (RFC-048 Design).
fn generate_row_layout(
    item: &str,
    rate: f64,
    inputs: &[&str],
) -> (spaghettio_core::models::SolverResult, LayoutResult) {
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

/// Exploration probe (run with --nocapture): geometry of the two
/// candidate cell source layouts, to design the crop + port extraction.
#[test]
#[ignore = "exploration probe, not a gate"]
fn probe_cell_source_geometry() {
    for (label, item, rate, inputs) in [
        ("cable", "copper-cable", 15.0, &["copper-plate"][..]),
        ("ec", "electronic-circuit", 5.0, &["iron-plate", "copper-cable"][..]),
    ] {
        let (sr, l) = generate_row_layout(item, rate, inputs);
        println!("== {label}: {}x{}, {} entities ==", l.width, l.height, l.entities.len());
        for m in &sr.machines {
            println!("   spec {} x{:.2}", m.recipe, m.count);
        }
        // Edge survey: entities in the outermost 2 columns/rows, belts only.
        for e in &l.entities {
            let edge = e.x <= 1
                || e.x >= l.width - 2
                || e.y <= 1
                || e.y >= l.height - 2;
            if edge && spaghettio_core::common::is_belt_entity(&e.name) {
                println!(
                    "   edge belt ({},{}) {} dir={:?} carries={:?} seg={:?}",
                    e.x, e.y, e.name, e.direction, e.carries, e.segment_id
                );
            }
        }
    }
}
