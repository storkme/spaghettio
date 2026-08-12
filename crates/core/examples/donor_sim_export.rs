//! One-off: export the donor-stamped copper-plate layouts (counts 48, 52)
//! as sim-harness bp+manifest pairs, synthesizing the boundary records
//! from the celldb entry ports (TemplateCandidate::produce emits none).

use rustc_hash::FxHashSet;
use spaghettio_core::blueprint;
use spaghettio_core::bus::decomposition_search::DecompositionCandidate;
use spaghettio_core::bus::layout::LayoutOptions;
use spaghettio_core::bus::template_candidate::TemplateCandidate;
use spaghettio_core::models::{BoundaryRecord, EntityDirection};
use spaghettio_core::solver;

fn main() {
    let out_root = std::env::args().nth(1).expect("out dir");
    // (count, label, in-record, out-record)
    let cases = [
        (
            48u32,
            "donor48-f3r",
            BoundaryRecord {
                item: "copper-ore".into(),
                x: 0,
                y: 6,
                direction: EntityDirection::East,
                is_fluid: false,
                entity: "fast-transport-belt".into(),
            },
            BoundaryRecord {
                item: "copper-plate".into(),
                x: 73,
                y: 6,
                direction: EntityDirection::East,
                is_fluid: false,
                entity: "fast-transport-belt".into(),
            },
        ),
        (
            52u32,
            "donor52-on0",
            BoundaryRecord {
                item: "copper-ore".into(),
                x: 8,
                y: 56,
                direction: EntityDirection::North,
                is_fluid: false,
                entity: "express-transport-belt".into(),
            },
            BoundaryRecord {
                item: "copper-plate".into(),
                x: 9,
                y: 0,
                direction: EntityDirection::North,
                is_fluid: false,
                entity: "express-transport-belt".into(),
            },
        ),
    ];
    for (count, label, rec_in, rec_out) in cases {
        let inputs: FxHashSet<String> = ["copper-ore".to_string()].into_iter().collect();
        let probe = solver::solve("copper-plate", 1.0, &inputs, "electric-furnace").unwrap();
        let rate = (count as f64 - 0.01) / probe.machines[0].count;
        let sr = solver::solve("copper-plate", rate, &inputs, "electric-furnace").unwrap();
        let mut layout = TemplateCandidate.produce(&sr, &LayoutOptions::default()).unwrap();
        layout.boundary_inputs = vec![rec_in];
        layout.boundary_outputs = vec![rec_out];
        let (bp, manifest) = blueprint::export_with_manifest(&layout, &sr, label);
        let dir = format!("{out_root}/{label}");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/bp.txt"), &bp).unwrap();
        std::fs::write(
            format!("{dir}/manifest-real.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        println!("{label}: rate={rate:.3} -> {dir}");
    }
}
