//! Boundary records must describe the direction of the physical bus flow.

use spaghettio_core::calibration_matrix::{self, CalibrationFixture};
use spaghettio_core::models::EntityDirection;

fn fixture(name: &str) -> CalibrationFixture {
    calibration_matrix::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("missing calibration fixture {name}"))
}

#[test]
fn fluid_boundary_inputs_use_lane_flow_and_preserve_pipe_to_ground_direction() {
    let fixtures = [
        fixture("tier3_plastic_bar"),
        fixture("tier3_sulfuric_acid"),
        fixture("tier3_heavy_oil_cracking"),
    ];
    let mut bare_pipe_count = 0;
    let mut pipe_to_ground_count = 0;

    for fixture in fixtures {
        let built = calibration_matrix::build(&fixture)
            .unwrap_or_else(|e| panic!("{} should build: {e}", fixture.name));
        for record in built.layout.boundary_inputs.iter().filter(|r| r.is_fluid) {
            let head = built
                .layout
                .entities
                .iter()
                .find(|e| e.x == record.x && e.y == record.y)
                .unwrap_or_else(|| {
                    panic!(
                        "{}: missing boundary head at ({}, {})",
                        fixture.name, record.x, record.y
                    )
                });
            assert_eq!(head.name, record.entity, "{}: boundary head mismatch", fixture.name);

            match record.entity.as_str() {
                "pipe" => {
                    bare_pipe_count += 1;
                    assert_eq!(
                        record.direction,
                        EntityDirection::South,
                        "{}: bare fluid pipe must record the southbound lane flow",
                        fixture.name
                    );
                }
                "pipe-to-ground" => {
                    pipe_to_ground_count += 1;
                    assert_eq!(
                        record.direction, head.direction,
                        "{}: pipe-to-ground boundary direction must remain the head direction",
                        fixture.name
                    );
                    assert_eq!(
                        record.direction,
                        EntityDirection::South,
                        "{}: pipe-to-ground input head must flow south",
                        fixture.name
                    );
                }
                other => panic!("{}: unexpected fluid boundary head {other}", fixture.name),
            }
        }
    }

    assert!(bare_pipe_count > 0, "affected fixtures must exercise a bare pipe head");
    assert!(
        pipe_to_ground_count > 0,
        "affected fixtures must exercise a pipe-to-ground head"
    );
}
