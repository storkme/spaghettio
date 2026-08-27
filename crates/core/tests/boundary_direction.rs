//! Boundary records must describe the direction of the physical flow INTO
//! the layout — the sim harness rigs the outside world at the opposite
//! side of the head tile, so a wrong direction builds the rig on top of
//! the factory (#732: bare `pipe` heads carried the export default, North).
//!
//! Two layers (#736 review): the INVARIANT, checked on every calibration
//! fixture that builds — one step from the head in the recorded direction
//! is occupied (the lane continues into the layout) and one step the other
//! way is empty (the outside, where the rig goes) — and the three fixtures
//! that carried the bug, kept as named regression witnesses for the
//! bare-pipe class (existence checks, not exact counts).

use rustc_hash::FxHashSet;
use spaghettio_core::calibration_matrix::{self, CalibrationFixture};
use spaghettio_core::models::{BoundaryRecord, EntityDirection, LayoutResult};

fn step(d: EntityDirection) -> (i32, i32) {
    match d {
        EntityDirection::North => (0, -1),
        EntityDirection::East => (1, 0),
        EntityDirection::South => (0, 1),
        EntityDirection::West => (-1, 0),
    }
}

/// Every fluid boundary-input record points into the layout: the lane
/// continues along `direction` — the next tile for a bare `pipe`, any
/// tile within the underground reach (10 tiles, so the paired mouth is
/// at most 11 away) for a `pipe-to-ground` head whose span is surface-
/// empty by design — and the tile the other way carries nothing (the
/// outside, where the rig goes). Returns the violations (empty = holds).
fn into_layout_violations(layout: &LayoutResult) -> Vec<String> {
    let occupied: FxHashSet<(i32, i32)> = layout.entities.iter().map(|e| (e.x, e.y)).collect();
    layout
        .boundary_inputs
        .iter()
        .filter(|r| r.is_fluid)
        .filter_map(|r: &BoundaryRecord| {
            let (dx, dy) = step(r.direction);
            let reach = if r.entity == "pipe-to-ground" { 11 } else { 1 };
            let inside = (1..=reach).any(|k| occupied.contains(&(r.x + dx * k, r.y + dy * k)));
            let outside_clear = !occupied.contains(&(r.x - dx, r.y - dy));
            if inside && outside_clear {
                None
            } else {
                Some(format!(
                    "{} head {} at ({}, {}) records {:?}: inside occupied={inside}, outside clear={outside_clear}",
                    r.item, r.entity, r.x, r.y, r.direction
                ))
            }
        })
        .collect()
}

#[test]
fn fluid_boundary_inputs_point_into_the_layout_on_every_calibration_fixture() {
    const FLUIDS: [&str; 8] = [
        "crude-oil",
        "water",
        "heavy-oil",
        "light-oil",
        "petroleum-gas",
        "lubricant",
        "sulfuric-acid",
        "steam",
    ];
    let mut checked = 0usize;
    let mut violations = Vec::new();
    // Only fixtures with a fluid external input can have a fluid head;
    // skipping the solid-only ones keeps this sweep to the builds it
    // needs (a full build of every fixture is ~2.5 minutes in debug).
    for fixture in calibration_matrix::fixtures().into_iter().filter(|f| f.inputs.iter().any(|i| FLUIDS.contains(i))) {
        let Ok(built) = calibration_matrix::build(&fixture) else { continue };
        if built.layout.boundary_inputs.iter().any(|r| r.is_fluid) {
            checked += 1;
        }
        for v in into_layout_violations(&built.layout) {
            violations.push(format!("{}: {v}", fixture.name));
        }
    }
    assert!(checked >= 3, "the corpus must exercise fluid heads (checked {checked})");
    assert!(violations.is_empty(), "fluid heads pointing out of the layout:\n{}", violations.join("\n"));
}

fn fixture(name: &str) -> CalibrationFixture {
    calibration_matrix::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("missing calibration fixture {name}"))
}

/// The #732 witnesses: the three bank rows whose bare-pipe heads recorded
/// North and were rigged into the layout. Each must now record the flow
/// (South, by the invariant above) and the set must still exercise both
/// head classes — a bare `pipe` and a `pipe-to-ground` whose placed
/// direction the record keeps.
#[test]
fn the_732_witnesses_record_the_flow_for_both_head_classes() {
    let mut bare_pipe = 0usize;
    let mut pipe_to_ground = 0usize;
    for fixture in [
        fixture("tier3_plastic_bar"),
        fixture("tier3_sulfuric_acid"),
        fixture("tier3_heavy_oil_cracking"),
    ] {
        let built = calibration_matrix::build(&fixture)
            .unwrap_or_else(|e| panic!("{} should build: {e}", fixture.name));
        assert!(into_layout_violations(&built.layout).is_empty(), "{}", fixture.name);
        for record in built.layout.boundary_inputs.iter().filter(|r| r.is_fluid) {
            let head = built
                .layout
                .entities
                .iter()
                .find(|e| e.x == record.x && e.y == record.y)
                .unwrap_or_else(|| panic!("{}: no head at ({}, {})", fixture.name, record.x, record.y));
            assert_eq!(head.name, record.entity, "{}: head/record entity mismatch", fixture.name);
            match record.entity.as_str() {
                "pipe" => {
                    bare_pipe += 1;
                    assert_ne!(
                        record.direction,
                        EntityDirection::North,
                        "{}: a bare pipe's record must not be the export default",
                        fixture.name
                    );
                }
                "pipe-to-ground" => {
                    pipe_to_ground += 1;
                    assert_eq!(record.direction, head.direction, "{}: a pipe-to-ground head keeps its placed direction", fixture.name);
                }
                other => panic!("{}: unexpected fluid boundary head {other}", fixture.name),
            }
        }
    }
    assert!(bare_pipe > 0, "the witnesses must exercise a bare pipe head");
    assert!(pipe_to_ground > 0, "the witnesses must exercise a pipe-to-ground head");
}
