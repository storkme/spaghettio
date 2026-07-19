//! Single source of truth for machine fluid-port geometry (RFP
//! `docs/rfp-power-supply.md` Phase 0e-i).
//!
//! The bus fluid-row templates and the fluid-connectivity validator must agree
//! on where a machine's pipes sit, exactly as the power/fluid validators and
//! the layout engine were unified behind [`crate::common::MACHINE_ENTITY_NAMES`]
//! in Phase 0b. Before this module the validator carried its own port tables
//! (`validate::fluids::fluid_ports`) while the templates re-derived port
//! positions by hand — the same drift class Phase 0 killed for the machine
//! lists. This module is now the one place both sides read.
//!
//! ## Coordinates
//!
//! Each port is `(dx, dy, production_type)`: the tile where an adjacent pipe
//! must sit to connect to that fluid box, **relative to the machine's top-left
//! footprint tile**, for the machine placed at the given orientation.
//! `production_type` is `"input"` or `"output"`.
//!
//! ## Provenance
//!
//! Base (north-facing, unmirrored) tables are ground-truthed from
//! `recipes.json` `fluid_boxes` and draftsman-verified by
//! `scripts/verify_fluid_ports_emag_cryo.py`. The orientation transforms
//! (mirror y-flip, East rotation) are the authoritative Factorio transforms,
//! verified by `scripts/verify_fluid_ports_transforms.py` against draftsman's
//! own `rotate_point` and against the in-game-validated oil-refinery mirror
//! precedent. For foundry and cryogenic-plant the mirror (y-flip) and a
//! 180-degree rotation produce identical port tiles (the ports are x-symmetric),
//! so the mirror tables are unambiguous — the script asserts that equivalence.

use crate::models::EntityDirection;

/// A fluid port: `(dx, dy, production_type)` relative to the machine top-left.
pub type FluidPort = (i32, i32, &'static str);

// --- 3x3 machines (center=1) --------------------------------------------

/// assembling-machine-2 / -3: input north, output south.
const AM2: &[FluidPort] = &[(1, -1, "input"), (1, 3, "output")];

/// chemical-plant / biochamber: inputs north, outputs south. biochamber's
/// fluid boxes are geometrically identical to chemical-plant's (verified in
/// recipes.json), so they share this table.
const CHEM: &[FluidPort] = &[
    (0, -1, "input"),
    (2, -1, "input"),
    (0, 3, "output"),
    (2, 3, "output"),
];

// --- 5x5 machines (center=2) --------------------------------------------

/// oil-refinery, north-facing unmirrored: inputs south, outputs north.
const OIL: &[FluidPort] = &[
    (1, 5, "input"),
    (3, 5, "input"),
    (0, -1, "output"),
    (2, -1, "output"),
    (4, -1, "output"),
];
/// oil-refinery mirrored (the placement the engine actually uses): the y-flip
/// swaps the input/output faces so inputs sit on the north edge under the bus
/// trunk. In-game-validated (the entire oil-processing tier ladder).
const OIL_MIRROR: &[FluidPort] = &[
    (1, -1, "input"),
    (3, -1, "input"),
    (0, 5, "output"),
    (2, 5, "output"),
    (4, 5, "output"),
];

/// foundry, north-facing unmirrored: inputs south, outputs north.
const FOUNDRY: &[FluidPort] = &[
    (1, 5, "input"),
    (3, 5, "input"),
    (1, -1, "output"),
    (3, -1, "output"),
];
/// foundry mirrored: inputs north, outputs south (same y-flip as oil-refinery;
/// x-symmetric so identical to a 180-degree rotation — see the provenance
/// script's assertion). Used for molten-metal casting, whose fluid *output*
/// then sits on the south face the dual-input row's output pipe row occupies.
const FOUNDRY_MIRROR: &[FluidPort] = &[
    (1, -1, "input"),
    (3, -1, "input"),
    (1, 5, "output"),
    (3, 5, "output"),
];

/// cryogenic-plant, north-facing unmirrored: inputs south, outputs north.
const CRYO: &[FluidPort] = &[
    (0, 5, "input"),
    (2, 5, "input"),
    (4, 5, "input"),
    (0, -1, "output"),
    (2, -1, "output"),
    (4, -1, "output"),
];
/// cryogenic-plant mirrored: inputs north, outputs south (y-flip; x-symmetric,
/// == 180-degree rotation). Brings the fluid inputs onto the north face the bus
/// fluid-row templates deliver to.
const CRYO_MIRROR: &[FluidPort] = &[
    (0, -1, "input"),
    (2, -1, "input"),
    (4, -1, "input"),
    (0, 5, "output"),
    (2, 5, "output"),
    (4, 5, "output"),
];

// --- 4x4 machine (center=2, even) ---------------------------------------

/// electromagnetic-plant, north-facing unmirrored. Its fluid ports face
/// EAST/WEST as well as N/S, so no mirror brings an input onto the north face —
/// only rotation does. Inputs west/east, outputs south/north.
const EMAG: &[FluidPort] = &[
    (-1, 2, "input"),
    (4, 1, "input"),
    (2, 4, "output"),
    (1, -1, "output"),
];
/// electromagnetic-plant rotated East (direction=4, +90 deg): the west input
/// rotates onto the NORTH face at dx=1, so a solid-output emag recipe's single
/// fluid input (e.g. superconductor's light-oil) can be delivered by the bus
/// fluid-row's north pipe. Outputs rotate to west/east — usable only when the
/// recipe's output is solid (fluid-output emag recipes stay unsupported and
/// fail loud; see the validator). Verified against draftsman's `rotate_point`.
const EMAG_EAST: &[FluidPort] = &[
    (1, -1, "input"),
    (2, 4, "input"),
    (-1, 2, "output"),
    (4, 1, "output"),
];

/// Fluid ports for `entity` placed at the given `mirror` / `direction`,
/// relative to its top-left tile. Empty for machines with no fluid boxes
/// (assembling-machine-1, electric-furnace, centrifuge, recycler).
///
/// Only the orientations the layout engine actually places are distinguished:
/// oil/foundry/cryo honor `mirror` (north-vs-south input face); emag honors
/// `direction` (North vs the East-rotation used for solid-output fluid recipes).
/// All other machines return their single base table regardless of orientation.
pub fn fluid_ports(entity: &str, mirror: bool, direction: EntityDirection) -> &'static [FluidPort] {
    match entity {
        "assembling-machine-2" | "assembling-machine-3" => AM2,
        "chemical-plant" | "biochamber" => CHEM,
        "oil-refinery" => {
            if mirror {
                OIL_MIRROR
            } else {
                OIL
            }
        }
        "foundry" => {
            if mirror {
                FOUNDRY_MIRROR
            } else {
                FOUNDRY
            }
        }
        "cryogenic-plant" => {
            if mirror {
                CRYO_MIRROR
            } else {
                CRYO
            }
        }
        "electromagnetic-plant" => {
            if direction == EntityDirection::East {
                EMAG_EAST
            } else {
                EMAG
            }
        }
        _ => &[],
    }
}

/// The `dx` offsets of the output-port pipes on the SOUTH face
/// (`dy == mh`, the machine's height), for `entity` at the given orientation.
/// This is what the single-/dual-input row templates' output pipe row must
/// register so the ghost router taps the right columns. Sorted ascending.
pub fn south_output_dxs(entity: &str, mirror: bool, direction: EntityDirection, mh: i32) -> Vec<i32> {
    let mut dxs: Vec<i32> = fluid_ports(entity, mirror, direction)
        .iter()
        .filter(|(_, dy, pt)| *dy == mh && *pt == "output")
        .map(|(dx, _, _)| *dx)
        .collect();
    dxs.sort_unstable();
    dxs
}

/// Whether `entity`'s fluid output ports (at the given orientation) all sit on
/// the SOUTH face (`dy == mh`) — the face the single-/dual-input row templates
/// emit their fluid-output pipe row on. `false` when the machine has no output
/// port there (e.g. an unmirrored foundry, whose outputs face north). Lets the
/// placer gate "solid-in → fluid-out" rows onto that template branch without
/// hard-coding machine names (chemical-plant, biochamber today).
pub fn output_ports_all_south(entity: &str, mirror: bool, direction: EntityDirection, mh: i32) -> bool {
    let outs: Vec<&FluidPort> = fluid_ports(entity, mirror, direction)
        .iter()
        .filter(|(_, _, pt)| *pt == "output")
        .collect();
    !outs.is_empty() && outs.iter().all(|(_, dy, _)| *dy == mh)
}

/// Whether `entity` has any fluid ports (non-empty base table). Test-only: the
/// live validator uses the `fluid_ports(...).is_empty()` guard inline (it needs
/// the port list, not just the presence bit); this accessor exists for the
/// `common::machine_classification_no_drift` regression.
#[cfg(test)]
pub fn machine_has_fluid_ports(entity: &str) -> bool {
    !fluid_ports(entity, false, EntityDirection::North).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_tables_match_draftsman_ground_truth() {
        // Sanity anchors reproduced by scripts/verify_fluid_ports_transforms.py.
        assert_eq!(
            fluid_ports("cryogenic-plant", false, EntityDirection::North),
            &[
                (0, 5, "input"),
                (2, 5, "input"),
                (4, 5, "input"),
                (0, -1, "output"),
                (2, -1, "output"),
                (4, -1, "output"),
            ]
        );
        assert_eq!(
            fluid_ports("electromagnetic-plant", false, EntityDirection::North),
            &[(-1, 2, "input"), (4, 1, "input"), (2, 4, "output"), (1, -1, "output")]
        );
    }

    #[test]
    fn mirror_moves_5x5_inputs_to_north() {
        // foundry + cryo mirror swaps the input face south -> north.
        assert_eq!(
            fluid_ports("foundry", true, EntityDirection::North),
            &[(1, -1, "input"), (3, -1, "input"), (1, 5, "output"), (3, 5, "output")]
        );
        assert_eq!(
            fluid_ports("cryogenic-plant", true, EntityDirection::North),
            &[
                (0, -1, "input"),
                (2, -1, "input"),
                (4, -1, "input"),
                (0, 5, "output"),
                (2, 5, "output"),
                (4, 5, "output"),
            ]
        );
    }

    #[test]
    fn emag_east_rotation_puts_an_input_on_the_north_face() {
        // The West input rotates onto the north face at dx=1.
        let east = fluid_ports("electromagnetic-plant", false, EntityDirection::East);
        assert!(east.contains(&(1, -1, "input")));
        // Unrotated emag has NO north input (both inputs face east/west).
        let north = fluid_ports("electromagnetic-plant", false, EntityDirection::North);
        assert!(!north.iter().any(|(_, dy, pt)| *dy == -1 && *pt == "input"));
    }

    #[test]
    fn non_fluid_machines_have_no_ports() {
        for entity in ["assembling-machine-1", "electric-furnace", "centrifuge", "recycler"] {
            assert!(!machine_has_fluid_ports(entity), "{entity} should have no fluid ports");
        }
    }
}
