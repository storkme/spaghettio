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
        // The hand-keyed records are verified against the layout produce()
        // actually returned: each port tile must hold a transport entity
        // whose SURFACE tier matches the record's belt prototype. The
        // record deliberately names the surface belt, never a splitter —
        // the sim kit builds its own rig belts from `entity`, and a
        // splitter prototype there would stamp splitters down the rig.
        for rec in [&rec_in, &rec_out] {
            let holder = layout
                .entities
                .iter()
                .find(|e| {
                    let (w, h) =
                        spaghettio_core::common::oriented_entity_dims(&e.name, e.direction);
                    rec.x >= e.x && rec.x < e.x + w && rec.y >= e.y && rec.y < e.y + h
                })
                .unwrap_or_else(|| panic!("port tile ({},{}) is empty", rec.x, rec.y));
            let surface = holder
                .name
                .strip_suffix("-splitter")
                .map(|b| format!("{b}-transport-belt"))
                .unwrap_or_else(|| holder.name.clone());
            assert_eq!(
                surface, rec.entity,
                "boundary record at ({},{}) names {} but the tile holds {}",
                rec.x, rec.y, rec.entity, holder.name
            );
        }
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
