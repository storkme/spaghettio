//! Assembly row templates: patterns of belts, inserters, and machines.
//!
//! Every belt and inserter entity is tagged with `carries` so the validator
//! can trace item flow through the layout.
//!
//! Machines are packed with zero gap (3-tile pitch for 3×3 machines).
//! When lane splitting is active, machines are split into two groups with a
//! sideload bridge in between so both output belt lanes are utilised.
//!
//! Port of `src/bus/templates.py`.

use crate::models::{EntityDirection, PlacedEntity};

/// Gap between machine groups when lane-splitting output belts.
/// 3 tiles: 1 for sideload target filler, 1 for through-belt filler,
/// 1 for the NORTH lift from group 2.
pub const LANE_SPLIT_GAP: i32 = 3;

// Fluid port dx (relative to machine tile_position) for each machine type.
fn fluid_input_port_dx(machine_entity: &str) -> i32 {
    match machine_entity {
        "assembling-machine-2" | "assembling-machine-3" => 1,
        _ => 0,
    }
}

/// Map `output_east` flag to the corresponding belt direction.
fn output_dir(output_east: bool) -> EntityDirection {
    if output_east { EntityDirection::East } else { EntityDirection::West }
}

/// Return x-coordinates for each machine, accounting for lane-split gap.
fn machine_xs(x_offset: i32, machine_count: usize, pitch: i32, lane_split: bool) -> Vec<i32> {
    if !lane_split || machine_count < 2 {
        return (0..machine_count as i32)
            .map(|i| x_offset + i * pitch)
            .collect();
    }

    let g1 = machine_count / 2;
    let mut positions = Vec::with_capacity(machine_count);
    for i in 0..g1 {
        positions.push(x_offset + i as i32 * pitch);
    }
    for j in 0..(machine_count - g1) {
        positions.push(x_offset + g1 as i32 * pitch + LANE_SPLIT_GAP + j as i32 * pitch);
    }
    positions
}

/// Number of tiles to drop from the east end of the last machine's belt stamp.
///
/// A belt tile is "orphan" when it sits east of every inserter that picks from
/// (east-flow input) or drops onto (west-flow output) the belt: items never
/// flow through it, so it has no functional role. Skipping these tiles frees
/// corridor for the ghost router. `last_adjacency_dx` is the rightmost `dx`
/// of an inserter adjacency on the belt for one machine.
fn east_tail_skip(msz: i32, last_adjacency_dx: i32) -> i32 {
    (msz - 1 - last_adjacency_dx).max(0)
}

/// Generate the 6-entity sideload bridge between two machine groups.
///
/// `output_row_dy` is the output belt's offset from `y_offset`
/// (6 for `single_input_row`, 7 for `dual_input_row`).
///
/// When `output_east` is `true`, the bridge is mirrored: group 1 items
/// flow EAST across the bridge into group 2 (instead of group 2 → group 1).
fn sideload_bridge(
    gap_start_x: i32,
    y_offset: i32,
    output_row_dy: i32,
    belt: &str,
    item: &str,
    output_east: bool,
) -> Vec<PlacedEntity> {
    let bridge_y = y_offset + output_row_dy - 1;
    let output_y = y_offset + output_row_dy;

    let carries = Some(item.to_string());
    let belt = belt.to_string();

    if output_east {
        // EAST flow: group 1 → bridge EAST → group 2
        vec![
            // Bridge row
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: bridge_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: bridge_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: bridge_y,
                direction: EntityDirection::South,
                carries: carries.clone(),
                ..Default::default()
            },
            // Output belt row — gap tiles
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: output_y,
                direction: EntityDirection::North,
                carries: carries.clone(),
                ..Default::default()
            }, // lifts group1 items up to bridge
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: output_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                ..Default::default()
            }, // through-belt filler
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: output_y,
                direction: EntityDirection::East,
                carries: carries.clone(),
                ..Default::default()
            }, // sideload target (through-belt)
        ]
    } else {
        vec![
            // Bridge row (y+5 or y+6 depending on template)
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: bridge_y,
                direction: EntityDirection::South,
                carries: carries.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: bridge_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                ..Default::default()
            },
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: bridge_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                ..Default::default()
            },
            // Output belt row — gap tiles
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x,
                y: output_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                ..Default::default()
            }, // sideload target (through-belt)
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 1,
                y: output_y,
                direction: EntityDirection::West,
                carries: carries.clone(),
                ..Default::default()
            }, // through-belt filler
            PlacedEntity {
                name: belt.clone(),
                x: gap_start_x + 2,
                y: output_y,
                direction: EntityDirection::North,
                carries: carries.clone(),
                ..Default::default()
            }, // lifts group2 items up to bridge
        ]
    }
}

/// Description of one row that must be continued across a lane-split
/// gap. Templates pass a slice of these to `stamp_lane_split_gap`
/// describing every input belt / pipe header that cross-cuts the row.
pub(crate) struct GapRow<'a> {
    /// Row offset from `y_offset`.
    pub dy: i32,
    /// Entity name: `"transport-belt"` (any tier), `"pipe"`, etc.
    pub name: &'a str,
    /// Facing direction. Ignored for pipes (leave as `North` or any).
    pub direction: EntityDirection,
    /// Item/fluid carried.
    pub item: &'a str,
    /// Segment id for per-row provenance tracking.
    pub segment_id: Option<String>,
}

/// Stamp the `LANE_SPLIT_GAP` cross-row continuation tiles + the
/// sideload bridge at `output_row_dy`. Shared across every
/// lane-splitting row template so the gap geometry stays in one place.
/// Each template just declares *which* rows cross-cut its gap (input
/// belts, pipe headers) and hands them in `rows`.
pub(crate) fn stamp_lane_split_gap(
    entities: &mut Vec<PlacedEntity>,
    gap_start_x: i32,
    y_offset: i32,
    rows: &[GapRow<'_>],
    output_row_dy: i32,
    output_belt: &str,
    output_item: &str,
    output_east: bool,
) {
    for dx in 0..LANE_SPLIT_GAP {
        for row in rows {
            entities.push(PlacedEntity {
                name: row.name.to_string(),
                x: gap_start_x + dx,
                y: y_offset + row.dy,
                direction: row.direction,
                carries: Some(row.item.to_string()),
                segment_id: row.segment_id.clone(),
                ..Default::default()
            });
        }
    }
    entities.extend(sideload_bridge(
        gap_start_x,
        y_offset,
        output_row_dy,
        output_belt,
        output_item,
        output_east,
    ));
}

/// Row for a recipe with 1 solid input.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : input belt (EAST)
///   y+1 : input inserter (SOUTH)
///   y+2..y+2+msz-1 : machine (msz×msz)
///   y+2+msz : output inserter (SOUTH)
///   y+2+msz+1 : output belt (WEST -- toward bus)
/// ```
///
/// When `lane_split=true`, machines are split into two groups with a
/// sideload bridge between them so the output belt uses both lanes.
///
/// Returns `(entities, row_height)`.
pub fn single_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_item: &str,
    output_item: &str,
    input_belt: &str,
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32) {
    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 4;
    let mut entities = Vec::new();
    let belt_in_seg = Some(format!("row:{recipe}:belt-in:{input_item}"));
    let inserter_in_seg = Some(format!("row:{recipe}:inserter-in:{input_item}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    let lane_split = lane_split && machine_count >= 2;
    let mxs = machine_xs(x_offset, machine_count, pitch, lane_split);
    let g1 = if lane_split { machine_count / 2 } else { machine_count };
    let last_mx = *mxs.last().expect("machine_count >= 1");

    // Input belt: east-flow, inserter picks at mx+1 → last_adjacency_dx=1.
    // West-flow output belt: inserter drops at mx+1 → last_adjacency_dx=1.
    // East-flow output belt: tiles east of the drop carry items to the merger,
    // so no tail trim.
    let in_tail = east_tail_skip(msz, 1);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    for &mx in &mxs {
        let is_last = mx == last_mx;
        let in_stop = if is_last { msz - in_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Input belt (machine_size tiles wide, continuous with adjacent machines)
        for dx in 0..in_stop {
            entities.push(PlacedEntity {
                name: input_belt.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input_item.to_string()),
                segment_id: belt_in_seg.clone(),
                ..Default::default()
            });
        }

        // Input inserter
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 1,
            y: y_offset + 1,
            direction: EntityDirection::South,
            carries: Some(input_item.to_string()),
            segment_id: inserter_in_seg.clone(),
            ..Default::default()
        });

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 2,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output inserter
        let out_ins_y = y_offset + 2 + msz;
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 1,
            y: out_ins_y,
            direction: EntityDirection::South,
            carries: Some(output_item.to_string()),
            segment_id: inserter_out_seg.clone(),
            ..Default::default()
        });

        // Output belt (machine_size tiles wide)
        let out_belt_y = y_offset + 2 + msz + 1;
        let out_dir = output_dir(output_east);
        for dx in 0..out_stop {
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x: mx + dx,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }
    }

    if lane_split {
        let gap_start_x = x_offset + g1 as i32 * pitch;
        stamp_lane_split_gap(
            &mut entities,
            gap_start_x,
            y_offset,
            &[GapRow {
                dy: 0,
                name: input_belt,
                direction: EntityDirection::East,
                item: input_item,
                segment_id: belt_in_seg.clone(),
            }],
            2 + msz + 1, // output_row_dy
            output_belt,
            output_item,
            output_east,
        );
    }

    (entities, row_height)
}

/// Row for a recipe with 2 solid inputs.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : input belt 1 (EAST) -- far belt
///   y+1 : input belt 2 (EAST) -- close belt
///   y+2 : long-handed inserter (picks y+0) + inserter (picks y+1)
///   y+3..y+3+msz-1 : machine (msz×msz)
///   y+3+msz : output inserter (SOUTH)
///   y+3+msz+1 : output belt (WEST -- toward bus)
/// ```
///
/// When `lane_split=true`, machines are split into two groups with a
/// sideload bridge between them so the output belt uses both lanes.
///
/// Returns `(entities, row_height)`.
pub fn dual_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str),
    output_item: &str,
    input_belts: (&str, &str),
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32) {
    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 5;
    let mut entities = Vec::new();

    let (input1, input2) = input_items;
    let (belt1, belt2) = input_belts;
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    let lane_split = lane_split && machine_count >= 2;
    let mxs = machine_xs(x_offset, machine_count, pitch, lane_split);
    let g1 = if lane_split { machine_count / 2 } else { machine_count };
    let last_mx = *mxs.last().expect("machine_count >= 1");

    // Belt 1 (far): long-handed inserter picks at mx → dx=0, trim = msz-1.
    // Belt 2 (close): regular inserter picks at mx+2 → dx=2, trim = 0 for msz=3.
    // West-flow output belt: drop at mx+1 → dx=1, trim = 1 for msz=3.
    // East-flow output belt: no trim (tiles east of drop flow to merger).
    let in1_tail = east_tail_skip(msz, 0);
    let in2_tail = east_tail_skip(msz, 2);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    for &mx in &mxs {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Input belt 1 -- far belt
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }

        // Input belt 2 -- close belt
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: y_offset + 1,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Long-handed inserter (picks from far belt y+0, drops into machine y+3)
        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: mx,
            y: y_offset + 2,
            direction: EntityDirection::South,
            carries: Some(input1.to_string()),
            segment_id: inserter_in1_seg.clone(),
            ..Default::default()
        });

        // Regular inserter (picks from close belt y+1, drops into machine y+3)
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 2,
            y: y_offset + 2,
            direction: EntityDirection::South,
            carries: Some(input2.to_string()),
            segment_id: inserter_in2_seg.clone(),
            ..Default::default()
        });

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 3,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output inserter
        let out_ins_y = y_offset + 3 + msz;
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 1,
            y: out_ins_y,
            direction: EntityDirection::South,
            carries: Some(output_item.to_string()),
            segment_id: inserter_out_seg.clone(),
            ..Default::default()
        });

        // Output belt
        let out_belt_y = y_offset + 3 + msz + 1;
        let out_dir = output_dir(output_east);
        for dx in 0..out_stop {
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x: mx + dx,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }
    }

    if lane_split {
        let gap_start_x = x_offset + g1 as i32 * pitch;
        stamp_lane_split_gap(
            &mut entities,
            gap_start_x,
            y_offset,
            &[
                GapRow {
                    dy: 0,
                    name: belt1,
                    direction: EntityDirection::East,
                    item: input1,
                    segment_id: belt_in1_seg.clone(),
                },
                GapRow {
                    dy: 1,
                    name: belt2,
                    direction: EntityDirection::East,
                    item: input2,
                    segment_id: belt_in2_seg.clone(),
                },
            ],
            3 + msz + 1, // output_row_dy
            output_belt,
            output_item,
            output_east,
        );
    }

    (entities, row_height)
}

/// HorizontalStack variant of `dual_input_row` — see
/// `docs/rfp-horizontal-trunks.md`. One long row with `k_trunks`
/// stacked east-flowing input₀ trunks on top, a single continuous
/// input₁ belt, and machines packed into `block_size`-sized blocks
/// with a 1-tile gap between blocks for trunk dives + low-demand
/// E-axis UG cross.
///
/// Y layout (msz=3 example, K=2):
/// ```text
///   y+0..y+K-1 : K stacked east-flowing trunks of input₀ (high-demand)
///   y+K        : input₁ continuous east-flowing belt (low-demand,
///                E-axis UG cross at each block boundary)
///   y+K+1      : input₀ current-feed east-flowing belt (per-block
///                segments, fed by trunk dives)
///   y+K+2      : inserter row (long-handed picks input₁ at y+K,
///                regular/stack picks input₀ at y+K+1)
///   y+K+3..y+K+5 : 3×3 machine
///   y+K+6      : output inserter row
///   y+K+7      : output belt
/// ```
///
/// Phase 1 first cut: emits the row geometry without trunk-dive
/// connectivity — the K trunks and current-feed are disconnected
/// surface belts. The validator will flag the missing item source on
/// current-feed; the dive geometry will be added in subsequent
/// iterations once the visible row shape has been validated.
#[allow(clippy::too_many_arguments)]
pub fn dual_input_row_horizontal(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str),
    output_item: &str,
    input_belts: (&str, &str),
    output_belt: &str,
    k_trunks: usize,
    block_size: usize,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32) {
    let msz = machine_size as i32;
    let pitch = msz;
    let k = k_trunks.max(1) as i32;
    let block_size = block_size.max(1);
    let mut entities = Vec::new();

    let (input0, input1) = input_items;
    let (belt0, belt1) = input_belts;
    let trunk_seg = Some(format!("row:{recipe}:trunk:{input0}"));
    let belt1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let cf_seg = Some(format!("row:{recipe}:current-feed:{input0}"));
    let dive_seg = Some(format!("row:{recipe}:trunk-dive:{input0}"));
    let inserter_in0_seg = Some(format!("row:{recipe}:inserter-in:{input0}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Block layout: each block is `block_size` machines wide; a 1-tile
    // dive column precedes each block (including block 0). Trunk b
    // dives at `dive_xs[b]`, then block b's machines start at
    // `dive_xs[b] + 1` and span `block_size * msz` tiles.
    //
    // The leftmost dive sits at `x_offset + 1` (NOT `x_offset`) so the
    // iron-plate row's east-axis UG cross around it — UG INPUT at
    // `dive - 1`, UG OUTPUT at `dive + 1` — has both endpoints inside
    // the row template's iteration range. Without this offset the UG
    // INPUT would land at `x_offset - 1` (bus area, outside the
    // template), leaving the UG OUTPUT orphaned and the bus router's
    // straight iron-plate delivery sideloading onto the dive's
    // south-belt at `(x_offset, y+K)`.
    let n_blocks = machine_count.div_ceil(block_size);
    let dive_xs: Vec<i32> = (0..n_blocks)
        .map(|b| x_offset + 1 + (b as i32) * (block_size as i32 * pitch + 1))
        .collect();
    let mut machine_xs: Vec<i32> = Vec::with_capacity(machine_count);
    for (b, &dive_x) in dive_xs.iter().enumerate() {
        let block_count = if b == n_blocks - 1 {
            machine_count - b * block_size
        } else {
            block_size
        };
        for m in 0..block_count {
            machine_xs.push(dive_x + 1 + (m as i32) * pitch);
        }
    }
    let row_west_x = x_offset;
    let row_east_x_excl = machine_xs.last().copied().unwrap_or(x_offset) + pitch;

    // Y row indices.
    let trunk_y = |k_idx: i32| y_offset + k_idx;
    let belt1_y = y_offset + k;
    let cf_y = y_offset + k + 1;
    let inserter_in_y = y_offset + k + 2;
    let machine_y = y_offset + k + 3;
    let inserter_out_y = y_offset + k + 3 + msz;
    let out_belt_y = y_offset + k + 3 + msz + 1;
    let row_height = k + 8;

    let belt1_ug = ug_belt_name(belt1);

    let is_dive_col = |x: i32| dive_xs.contains(&x);

    // Lane assignment to trunk rows. We REVERSE the natural order so
    // the trunk closest to iron-plate (y+K-1) is lane 0 and dives at
    // the leftmost column. Each subsequent lane is one row higher and
    // dives one column further east. This means a trunk's dive only
    // ever has to bridge iron-plate — the trunks BELOW it have already
    // dove west (rows are empty), and the trunks ABOVE it are at
    // shallower rows that the dive's south-belts never touch.
    let lane_trunk_row = |b: usize| y_offset + (k - 1 - b as i32);

    // K stacked east-flowing trunks. Lane b's trunk lives at row
    // lane_trunk_row(b) from row_west_x up to dive_xs[b]; past that
    // column it's gone (the items have dove south). At intermediate
    // dive columns of *deeper* lanes (b' < b) the trunk continues
    // surface-east — those dives' south-belts are at rows below this
    // trunk, no entity conflict.
    for (b, &trunk_end_x) in dive_xs.iter().enumerate().take(k_trunks) {
        let trunk_row = lane_trunk_row(b);
        for x in row_west_x..trunk_end_x {
            entities.push(PlacedEntity {
                name: belt0.to_string(),
                x,
                y: trunk_row,
                direction: EntityDirection::East,
                carries: Some(input0.to_string()),
                segment_id: trunk_seg.clone(),
                ..Default::default()
            });
        }
    }

    // Iron-plate (low-demand input₁) continuous belt at y+K, with
    // east-axis UG crosses at each dive column.
    for x in row_west_x..row_east_x_excl {
        if is_dive_col(x) {
            continue;
        }
        let is_ug_input = dive_xs.iter().any(|&dx| x == dx - 1);
        let is_ug_output = dive_xs.iter().any(|&dx| x == dx + 1);
        let ent = if is_ug_input {
            PlacedEntity {
                name: belt1_ug.to_string(),
                x,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt1_seg.clone(),
                io_type: Some("input".to_string()),
                ..Default::default()
            }
        } else if is_ug_output {
            PlacedEntity {
                name: belt1_ug.to_string(),
                x,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt1_seg.clone(),
                io_type: Some("output".to_string()),
                ..Default::default()
            }
        } else {
            PlacedEntity {
                name: belt1.to_string(),
                x,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt1_seg.clone(),
                ..Default::default()
            }
        };
        entities.push(ent);
    }

    // Trunk dives. Lane b dives at column dive_xs[b], starting at its
    // trunk row `lane_trunk_row(b)` (= y+K-1-b) and descending through
    // any rows of deeper lanes (already empty — those lanes dove west)
    // plus iron-plate at y+K, landing on current-feed at y+K+1 via a
    // south→east corner.
    //   (dx, lane_trunk_row(b))           : SOUTH belt corner.
    //   (dx, lane_trunk_row(b)+1..y+K)    : SOUTH belts.
    //   (dx, y+K+1)                       : EAST belt corner — start of
    //                                       current-feed segment b.
    // Per B11 the south→east turn preserves both lanes, so the trunk's
    // full belt capacity feeds straight into current-feed.
    let _ = trunk_y;
    for (b, &dx) in dive_xs.iter().enumerate() {
        let trunk_row = lane_trunk_row(b);
        // South corner at the trunk's row.
        entities.push(PlacedEntity {
            name: belt0.to_string(),
            x: dx,
            y: trunk_row,
            direction: EntityDirection::South,
            carries: Some(input0.to_string()),
            segment_id: dive_seg.clone(),
            ..Default::default()
        });
        // South belts down to (and including) iron-plate row.
        for y in (trunk_row + 1)..=(y_offset + k) {
            entities.push(PlacedEntity {
                name: belt0.to_string(),
                x: dx,
                y,
                direction: EntityDirection::South,
                carries: Some(input0.to_string()),
                segment_id: dive_seg.clone(),
                ..Default::default()
            });
        }
        // East corner at current-feed row — first tile of segment b.
        entities.push(PlacedEntity {
            name: belt0.to_string(),
            x: dx,
            y: cf_y,
            direction: EntityDirection::East,
            carries: Some(input0.to_string()),
            segment_id: cf_seg.clone(),
            ..Default::default()
        });
    }

    // Current-feed segments per block. The east-corner above stamps
    // (dive_xs[b], cf_y); fill the rest of segment b only out to the
    // last machine's near-inserter column. Stopping at that column
    // (instead of continuing into the gap before the next dive)
    // prevents a straight-feed merge into the next dive's east-corner,
    // which would otherwise demote that corner's input from B7
    // straight-feed to B8 sideload (near lane only).
    for (b, &dx) in dive_xs.iter().enumerate() {
        let block_count = if b + 1 == n_blocks {
            machine_count - b * block_size
        } else {
            block_size
        };
        // Near-inserter (regular) sits at machine_x for each machine
        // (post-swap). The last current-feed tile we need is the column
        // of the last machine in this block.
        let last_machine_col = dx + 1 + (block_count as i32 - 1) * pitch;
        for x in (dx + 1)..=last_machine_col {
            entities.push(PlacedEntity {
                name: belt0.to_string(),
                x,
                y: cf_y,
                direction: EntityDirection::East,
                carries: Some(input0.to_string()),
                segment_id: cf_seg.clone(),
                ..Default::default()
            });
        }
    }

    // Inserters, machines, output inserters per machine.
    //
    // Inserter columns are SWAPPED relative to vertical-split's
    // `dual_input_row`: in horizontal-stack the high-demand input₀
    // sits on the near (current-feed) row at y+K+1 and the low-demand
    // input₁ sits on the far row at y+K. So the regular reach-1
    // inserter goes at machine_x (near, picks input₀) and the
    // long-handed reach-2 inserter goes at machine_x+2 (far, picks
    // input₁). This also keeps current-feed from needing to extend
    // past the last machine's column (see segment loop above).
    for &mx in &machine_xs {
        // Regular inserter at mx — reaches y+K+1 (current-feed, distance 1).
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx,
            y: inserter_in_y,
            direction: EntityDirection::South,
            carries: Some(input0.to_string()),
            segment_id: inserter_in0_seg.clone(),
            ..Default::default()
        });
        // Long-handed inserter at mx+2 — reaches y+K (iron-plate, distance 2).
        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: mx + 2,
            y: inserter_in_y,
            direction: EntityDirection::South,
            carries: Some(input1.to_string()),
            segment_id: inserter_in1_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 1,
            y: inserter_out_y,
            direction: EntityDirection::South,
            carries: Some(output_item.to_string()),
            segment_id: inserter_out_seg.clone(),
            ..Default::default()
        });
    }

    // Output belt — single continuous east- (or west-) flowing belt
    // across the whole row, including dive columns.
    let out_dir = output_dir(output_east);
    for x in row_west_x..row_east_x_excl {
        entities.push(PlacedEntity {
            name: output_belt.to_string(),
            x,
            y: out_belt_y,
            direction: out_dir,
            carries: Some(output_item.to_string()),
            segment_id: belt_out_seg.clone(),
            ..Default::default()
        });
    }

    (entities, row_height)
}

/// Map a transport-belt entity name to the matching underground-belt name.
fn ug_belt_name(belt: &str) -> &str {
    match belt {
        "fast-transport-belt" => "fast-underground-belt",
        "express-transport-belt" => "express-underground-belt",
        _ => "underground-belt",
    }
}

/// Row for a recipe with 3 solid inputs.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : input belt 1 (EAST) -- far belt (long-handed reach)
///   y+1 : input belt 2 (EAST) -- close belt (regular reach)
///   y+2 : long-handed-inserter at mx (picks y+0) + inserter at mx+2 (picks y+1)
///   y+3..y+3+msz-1 : machine (msz×msz)
///   y+3+msz : output inserter at mx+1 (SOUTH) + long-handed inserter at mx+2 (NORTH)
///   y+3+msz+1 : output belt (WEST or EAST)
///   y+3+msz+2 : input belt 3 (EAST) -- delivered from south side
/// ```
///
/// When `lane_split=true` (and `machine_count >= 2`), machines are
/// split into two groups with a sideload bridge between them so the
/// output belt uses both lanes. The gap continues all three input
/// belts (y+0, y+1, y+8) so items still reach the second group.
///
/// Returns `(entities, row_height)`.
pub fn triple_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    input_items: (&str, &str, &str),
    output_item: &str,
    input_belts: (&str, &str, &str),
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32) {
    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 6;
    let mut entities = Vec::new();

    let (input1, input2, input3) = input_items;
    let (belt1, belt2, belt3) = input_belts;
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let belt_in3_seg = Some(format!("row:{recipe}:belt-in:{input3}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let inserter_in3_seg = Some(format!("row:{recipe}:inserter-in:{input3}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Belt 1 (far, long-hand at mx): trim msz-1.
    // Belts 2 and 3 (regular at mx+2 / long-hand at mx+2 picking from south): trim 0 for msz=3.
    // West-flow output: trim 1 for msz=3; east-flow output: no trim.
    let in1_tail = east_tail_skip(msz, 0);
    let in2_tail = east_tail_skip(msz, 2);
    let in3_tail = east_tail_skip(msz, 2);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    let lane_split = lane_split && machine_count >= 2;
    let mxs = machine_xs(x_offset, machine_count, pitch, lane_split);
    let g1 = if lane_split { machine_count / 2 } else { machine_count };
    let last_mx = *mxs.last().expect("machine_count >= 1");

    for &mx in &mxs {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let in3_stop = if is_last { msz - in3_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Input belt 1 -- far belt (long-handed range)
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: y_offset,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }

        // Input belt 2 -- close belt (regular inserter range)
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: y_offset + 1,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Long-handed inserter: picks from y+0 (input1), drops into machine at y+3
        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: mx,
            y: y_offset + 2,
            direction: EntityDirection::South,
            carries: Some(input1.to_string()),
            segment_id: inserter_in1_seg.clone(),
            ..Default::default()
        });

        // Regular inserter: picks from y+1 (input2), drops into machine at y+3
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 2,
            y: y_offset + 2,
            direction: EntityDirection::South,
            carries: Some(input2.to_string()),
            segment_id: inserter_in2_seg.clone(),
            ..Default::default()
        });

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 3,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output inserter: picks from machine south face, drops to output belt
        let ins_y = y_offset + 3 + msz;
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: mx + 1,
            y: ins_y,
            direction: EntityDirection::South,
            carries: Some(output_item.to_string()),
            segment_id: inserter_out_seg.clone(),
            ..Default::default()
        });

        // Input3 long-handed inserter: picks from south belt, drops to machine south
        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: mx + 2,
            y: ins_y,
            direction: EntityDirection::North,
            carries: Some(input3.to_string()),
            segment_id: inserter_in3_seg.clone(),
            ..Default::default()
        });

        // Output belt
        let out_belt_y = y_offset + 3 + msz + 1;
        let out_dir = output_dir(output_east);
        for dx in 0..out_stop {
            entities.push(PlacedEntity {
                name: output_belt.to_string(),
                x: mx + dx,
                y: out_belt_y,
                direction: out_dir,
                carries: Some(output_item.to_string()),
                segment_id: belt_out_seg.clone(),
                ..Default::default()
            });
        }

        // Input belt 3 -- south-side belt (long-handed range from ins_y)
        let belt3_y = y_offset + 3 + msz + 2;
        for dx in 0..in3_stop {
            entities.push(PlacedEntity {
                name: belt3.to_string(),
                x: mx + dx,
                y: belt3_y,
                direction: EntityDirection::East,
                carries: Some(input3.to_string()),
                segment_id: belt_in3_seg.clone(),
                ..Default::default()
            });
        }
    }

    // Lane-split gap: continue all three input belts across the gap and
    // emit the sideload bridge at the output row. The two inserters at
    // y+(3+msz) (output + input3 long-hand) don't land in gap columns
    // because there are no machines there, so bridge_y is free.
    if lane_split {
        let gap_start_x = x_offset + g1 as i32 * pitch;
        let belt3_dy = 3 + msz + 2;
        stamp_lane_split_gap(
            &mut entities,
            gap_start_x,
            y_offset,
            &[
                GapRow {
                    dy: 0,
                    name: belt1,
                    direction: EntityDirection::East,
                    item: input1,
                    segment_id: belt_in1_seg.clone(),
                },
                GapRow {
                    dy: 1,
                    name: belt2,
                    direction: EntityDirection::East,
                    item: input2,
                    segment_id: belt_in2_seg.clone(),
                },
                GapRow {
                    dy: belt3_dy,
                    name: belt3,
                    direction: EntityDirection::East,
                    item: input3,
                    segment_id: belt_in3_seg.clone(),
                },
            ],
            3 + msz + 1, // output_row_dy
            output_belt,
            output_item,
            output_east,
        );
    }

    (entities, row_height)
}

/// Row for a recipe with 1 solid input + 1 fluid input.
///
/// For chemical-plant, uses the simple single-fluid pattern: a continuous
/// east-west pipe line on `y+0` spans the full machine width and carries
/// fluid to each machine's port via a short UG tunnel under the solid belt.
/// ```text
///   y+0 : pipe — pipe — pipe — ...            ← continuous east-west pipe row
///   y+1 : empty — UG pipe IN (SOUTH) — empty  ← dedicated UG-in row at port_x
///   y+2 : solid input belt (EAST, msz wide)
///   y+3 : UG pipe OUT (NORTH, port_x) + inserter (mx+1)
///   y+4..y+4+msz-1 : machine (msz×msz)
///   y+4+msz : output inserter (SOUTH)
///   y+4+msz+1 : output belt (WEST or EAST)
/// ```
/// The `y+0` pipe row gives machine-to-machine connectivity for free (all
/// machines share one fluid, so there's no isolation concern) and the bus
/// router just has to extend it west/east until it hits the fluid trunk
/// column. Multi-fluid-per-side rows need the richer UG-pipe-UG isolation
/// pattern; see `docs/archive/rfp-multi-fluid-rows.md` for that.
///
/// For other machines (assembling-machine-2/3 with fluid): uses a regular pipe
/// at the port position:
/// ```text
///   y+0 : solid input belt (EAST, msz wide)
///   y+1 : inserter (solid) + pipe at (mx+port_dx)
///   y+2..y+2+msz-1 : machine (msz×msz)
///   y+2+msz : output inserter (SOUTH)
///   y+2+msz+1 : output belt (WEST or EAST)
/// ```
///
/// Returns `(entities, row_height, fluid_port_pipes)` where
/// `fluid_port_pipes` is a list of `(x, y)` giving the bus tap-point for each
/// machine's fluid connection (a tile on the port-adjacent pipe row for
/// chemical-plant, the pipe tile for other machines).
pub fn fluid_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    solid_item: &str,
    fluid_item: &str,
    output_item: &str,
    input_belt: &str,
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>) {
    let msz = machine_size as i32;
    let pitch = msz;
    let port_dx = fluid_input_port_dx(machine_entity);
    let mut entities = Vec::new();
    let mut fluid_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let belt_in_seg = Some(format!("row:{recipe}:belt-in:{solid_item}"));
    let inserter_in_seg = Some(format!("row:{recipe}:inserter-in:{solid_item}"));
    let fluid_in_seg = Some(format!("row:{recipe}:belt-in:{fluid_item}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Solid input belt pickup at mx+1, west-flow output drop at mx+1 → trim 1 tile each.
    let in_tail = east_tail_skip(msz, 1);
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    // Lane-split: machines go in two groups with a LANE_SPLIT_GAP between them.
    // The sideload bridge in the gap flips some output onto the near lane so
    // both output-belt lanes run at ~full throughput.
    let lane_split = lane_split && machine_count >= 2;
    let mxs = machine_xs(x_offset, machine_count, pitch, lane_split);
    let g1 = if lane_split { machine_count / 2 } else { machine_count };
    let last_mx = *mxs.last().expect("machine_count >= 1");

    // Inserter column: must be distinct from the fluid-port column
    // (`port_dx`) so pipe and inserter never share a tile. chemical-plant
    // has `port_dx == 0` so the inserter sits at `mx + 1`; AM2/AM3 have
    // `port_dx == 1` so the inserter sits at `mx + 0`.
    let inserter_dx: i32 = if port_dx == 0 { 1 } else { 0 };
    // `in_tail` captures the current hard-coded assumption that the solid
    // belt is picked at `mx + 1`. Post-unification the actual pickup
    // column is `mx + inserter_dx`, so recompute the trim to match.
    let _ = in_tail;
    let in_tail = east_tail_skip(msz, inserter_dx);

    {
        // Unified T-junction pattern (formerly chemical-plant-only):
        // continuous east-west pipe line at y+0, a dedicated UG-in row at
        // y+1, the solid belt at y+2, and UG-out + inserter at y+3 adjacent
        // to the machine port. The same layout works for both chemical-
        // plant (`port_dx == 0`, inserter at `mx + 1`) and AM2/AM3
        // (`port_dx == 1`, inserter at `mx + 0`): the UG pair gives
        // machine-to-machine fluid connectivity via the y+0 header pipe
        // regardless of which column the fluid port sits on.
        //
        //   y+0: pipe row (msz tiles per machine, continuous across the row).
        //         Bus router extends this line west/east to the fluid trunk.
        //   y+1: UG pipe IN (SOUTH) at (port_x, y+1); rest of row empty.
        //   y+2: solid belt (crosses above the UG tunnel)
        //   y+3: UG pipe OUT (NORTH) at (port_x, y+3) + inserter at (mx+inserter_dx)
        //   y+4..y+6: machine (3×3)
        //   y+7: output inserter
        //   y+8: output belt
        let row_height = msz + 6;
        let belt_y = y_offset + 2;
        let interface_y = y_offset + 3; // UG pipe OUT + inserter row
        let machine_y = y_offset + 4;
        let out_ins_y = machine_y + msz;
        let out_belt_y = machine_y + msz + 1;
        let out_dir = output_dir(output_east);

        for &mx in &mxs {
            let is_last = mx == last_mx;
            let in_stop = if is_last { msz - in_tail } else { msz };
            let out_stop = if is_last { msz - out_tail } else { msz };

            // y+0: continuous east-west pipe line spanning the machine's
            // full width. Adjacent machines' pipes abut so the whole row
            // forms one pipe network, giving machine-to-machine fluid
            // connectivity. The bus router extends this line to the trunk.
            for dx in 0..msz {
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: y_offset,
                    carries: Some(fluid_item.to_string()),
                    segment_id: fluid_in_seg.clone(),
                    ..Default::default()
                });
            }

            // y+1: UG pipe IN facing south — enters tunnel, connects to the
            // T-junction pipe above and tunnels under the solid belt at y+2.
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: mx + port_dx,
                y: y_offset + 1,
                direction: EntityDirection::South,
                io_type: Some("input".to_string()),
                carries: Some(fluid_item.to_string()),
                segment_id: fluid_in_seg.clone(),
                ..Default::default()
            });

            // y+2: solid input belt (msz tiles wide)
            for dx in 0..in_stop {
                entities.push(PlacedEntity {
                    name: input_belt.to_string(),
                    x: mx + dx,
                    y: belt_y,
                    direction: EntityDirection::East,
                    carries: Some(solid_item.to_string()),
                    segment_id: belt_in_seg.clone(),
                    ..Default::default()
                });
            }

            // y+3: UG pipe OUT facing north (back toward input) adjacent to machine fluid port
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: mx + port_dx,
                y: interface_y,
                direction: EntityDirection::North,
                io_type: Some("output".to_string()),
                carries: Some(fluid_item.to_string()),
                segment_id: fluid_in_seg.clone(),
                ..Default::default()
            });
            // Inserter column: distinct from the fluid UG at `port_dx`.
            entities.push(PlacedEntity {
                name: "inserter".to_string(),
                x: mx + inserter_dx,
                y: interface_y,
                direction: EntityDirection::South,
                carries: Some(solid_item.to_string()),
                segment_id: inserter_in_seg.clone(),
                ..Default::default()
            });

            // Machine
            entities.push(PlacedEntity {
                name: machine_entity.to_string(),
                x: mx,
                y: machine_y,
                direction: EntityDirection::North,
                recipe: Some(recipe.to_string()),
                segment_id: machine_seg.clone(),
                ..Default::default()
            });

            // Output inserter
            entities.push(PlacedEntity {
                name: "inserter".to_string(),
                x: mx + 1,
                y: out_ins_y,
                direction: EntityDirection::South,
                carries: Some(output_item.to_string()),
                segment_id: inserter_out_seg.clone(),
                ..Default::default()
            });

            // Output belt
            for dx in 0..out_stop {
                entities.push(PlacedEntity {
                    name: output_belt.to_string(),
                    x: mx + dx,
                    y: out_belt_y,
                    direction: out_dir,
                    carries: Some(output_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }

            // Report a tile on the pipe row (y+0) as the bus tap-point for
            // each machine. The whole row at y+0 is pipe, so the bus router
            // can connect its horizontal branch to any x between mx and
            // mx+msz-1; we pick port_x for consistency with the UG column.
            fluid_port_pipes.push((fluid_item.to_string(), mx + port_dx, y_offset));
        }

        // Lane-split gap: continue the y+0 pipe header + y+2 solid input
        // belt across the `LANE_SPLIT_GAP` tiles so fluid + solid reach
        // group 2, then emit the sideload bridge at y+7 / y+8.
        if lane_split {
            let gap_start_x = x_offset + g1 as i32 * pitch;
            stamp_lane_split_gap(
                &mut entities,
                gap_start_x,
                y_offset,
                &[
                    GapRow {
                        dy: 0,
                        name: "pipe",
                        direction: EntityDirection::North,
                        item: fluid_item,
                        segment_id: fluid_in_seg.clone(),
                    },
                    GapRow {
                        dy: belt_y - y_offset,
                        name: input_belt,
                        direction: EntityDirection::East,
                        item: solid_item,
                        segment_id: belt_in_seg.clone(),
                    },
                ],
                out_belt_y - y_offset,
                output_belt,
                output_item,
                output_east,
            );
        }

        (entities, row_height, fluid_port_pipes)
    }
}

/// Row for a recipe with 2 solid inputs + 1 fluid input.
///
/// Fluid is delivered via a horizontal pipe header ABOVE the machine row,
/// with vertical pipe-to-ground tunnels per machine dropping fluid down to
/// the machine's fluid input port. This frees the inserter row for two inserters.
///
/// Layout per machine (`msz`-tile horizontal pitch, no gaps):
/// ```text
///   y+0 : horizontal fluid header (pipes carrying fluid_item)
///   y+1 : pipe-to-ground input at mx+port_dx (direction SOUTH)
///   y+2 : solid input belt 1 (EAST) -- far belt
///   y+3 : solid input belt 2 (EAST) -- close belt
///   y+4 : long-handed-inserter at mx+1 + inserter at mx+2 +
///           pipe-to-ground output at mx+port_dx (direction NORTH, faces input)
///   y+5..y+5+msz-1 : machine (msz×msz)
///   y+5+msz : fluid output pipes (if output_is_fluid) OR output inserter
///   y+5+msz+1 : output belt (solid output only)
/// ```
///
/// Returns `(entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)`.
pub fn fluid_dual_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    solid_items: (&str, &str),
    fluid_item: &str,
    output_item: &str,
    output_is_fluid: bool,
    input_belts: (&str, &str),
    output_belt: &str,
    lane_split: bool,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    let msz = machine_size as i32;
    let pitch = msz;
    // Fluid output occupies y+5+msz; add a trailing empty row so sub-row
    // stacking doesn't put output pipes adjacent to the next sub-row's
    // fluid header row (which would trip pipe-isolation).
    let row_height = msz + 7;
    let mut entities = Vec::new();

    let (input1, input2) = solid_items;
    let (belt1, belt2) = input_belts;
    let port_dx = fluid_input_port_dx(machine_entity);
    let fluid_in_seg = Some(format!("row:{recipe}:belt-in:{fluid_item}"));
    let belt_in1_seg = Some(format!("row:{recipe}:belt-in:{input1}"));
    let belt_in2_seg = Some(format!("row:{recipe}:belt-in:{input2}"));
    let inserter_in1_seg = Some(format!("row:{recipe}:inserter-in:{input1}"));
    let inserter_in2_seg = Some(format!("row:{recipe}:inserter-in:{input2}"));
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    let header_y = y_offset;
    let ptg_in_y = y_offset + 1;
    let belt1_y = y_offset + 2;
    let belt2_y = y_offset + 3;
    let inserter_y = y_offset + 4;
    let machine_y = y_offset + 5;
    let output_y = y_offset + 5 + msz;

    // Lane-split only makes sense when the output belt is actually a belt
    // (not a fluid pipe row). Fluid outputs go through the y+msz+5 pipe
    // row, which doesn't benefit from `sideload_bridge`.
    let lane_split = lane_split && machine_count >= 2 && !output_is_fluid;
    let mxs = machine_xs(x_offset, machine_count, pitch, lane_split);
    let g1 = if lane_split { machine_count / 2 } else { machine_count };
    let last_mx = *mxs.last().expect("machine_count >= 1");

    // Horizontal fluid header chain: spans x_offset .. last machine's mx+(msz-1).
    // With lane_split, `last_mx` already accounts for the `LANE_SPLIT_GAP`
    // offset on group 2, so this loop naturally fills the gap tiles with
    // continuous pipe — keeping the fluid network unbroken.
    let header_end_x = last_mx + msz - 1;
    for x in x_offset..=header_end_x {
        entities.push(PlacedEntity {
            name: "pipe".to_string(),
            x,
            y: header_y,
            carries: Some(fluid_item.to_string()),
            segment_id: fluid_in_seg.clone(),
            ..Default::default()
        });
    }

    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();

    // Inserter placement branches on port_dx:
    //   port_dx=0 (chemical-plant): long_x=mx+1, reg_x=mx+2.
    //   port_dx=1 (assembling-machine-2/3): long_x=mx+2, reg_x=mx.
    // Belt 1 (far, y+2) is picked by the long-handed inserter; belt 2 (close,
    // y+3) by the regular inserter.
    let (long_dx, reg_dx) = if port_dx == 1 { (2, 0) } else { (1, 2) };
    let in1_tail = east_tail_skip(msz, long_dx);
    let in2_tail = east_tail_skip(msz, reg_dx);
    // Solid output (output_is_fluid=false) has its drop at mx+1 → trim 1 for msz=3 west-flow.
    let out_tail = if output_east { 0 } else { east_tail_skip(msz, 1) };

    for &mx in &mxs {
        let is_last = mx == last_mx;
        let in1_stop = if is_last { msz - in1_tail } else { msz };
        let in2_stop = if is_last { msz - in2_tail } else { msz };
        let out_stop = if is_last { msz - out_tail } else { msz };

        // Vertical PTG pair: input at y+1 tunnels SOUTH to output at y+4
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: mx + port_dx,
            y: ptg_in_y,
            direction: EntityDirection::South,
            io_type: Some("input".to_string()),
            carries: Some(fluid_item.to_string()),
            segment_id: fluid_in_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "pipe-to-ground".to_string(),
            x: mx + port_dx,
            y: inserter_y,
            direction: EntityDirection::North,
            io_type: Some("output".to_string()),
            carries: Some(fluid_item.to_string()),
            segment_id: fluid_in_seg.clone(),
            ..Default::default()
        });

        // Solid input belts (machine_size tiles wide each)
        for dx in 0..in1_stop {
            entities.push(PlacedEntity {
                name: belt1.to_string(),
                x: mx + dx,
                y: belt1_y,
                direction: EntityDirection::East,
                carries: Some(input1.to_string()),
                segment_id: belt_in1_seg.clone(),
                ..Default::default()
            });
        }
        for dx in 0..in2_stop {
            entities.push(PlacedEntity {
                name: belt2.to_string(),
                x: mx + dx,
                y: belt2_y,
                direction: EntityDirection::East,
                carries: Some(input2.to_string()),
                segment_id: belt_in2_seg.clone(),
                ..Default::default()
            });
        }

        // Inserter placement depends on which column the fluid PTG occupies.
        // port_dx == 0 (chemical-plant): PTG at mx+0, inserters at mx+1 (long) and mx+2 (regular).
        // port_dx == 1 (assembling-machine-2/3): PTG at mx+1, so move the
        //   long-handed inserter to mx+2 and the regular inserter to mx+0.
        let long_x = mx + long_dx;
        let reg_x = mx + reg_dx;

        entities.push(PlacedEntity {
            name: "long-handed-inserter".to_string(),
            x: long_x,
            y: inserter_y,
            direction: EntityDirection::South,
            carries: Some(input1.to_string()),
            segment_id: inserter_in1_seg.clone(),
            ..Default::default()
        });
        entities.push(PlacedEntity {
            name: "inserter".to_string(),
            x: reg_x,
            y: inserter_y,
            direction: EntityDirection::South,
            carries: Some(input2.to_string()),
            segment_id: inserter_in2_seg.clone(),
            ..Default::default()
        });

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output row
        if output_is_fluid {
            // Continuous pipe row one tile south of the machine spanning the
            // full machine width. Chemical-plant's two output fluid boxes sit
            // at dx=0 and dx=2, both of which this pipe row covers. Adjacent
            // machines' rows abut, giving machine-to-machine connectivity and
            // a clean run out to the bus trunk for downstream consumers.
            for dx in 0..msz {
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: output_y,
                    carries: Some(output_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }
            fluid_output_port_pipes.push((output_item.to_string(), mx, output_y));
            fluid_output_port_pipes.push((output_item.to_string(), mx + 2, output_y));
        } else {
            // Solid output: inserter at output_y, belt at output_y+1
            entities.push(PlacedEntity {
                name: "inserter".to_string(),
                x: mx + 1,
                y: output_y,
                direction: EntityDirection::South,
                carries: Some(output_item.to_string()),
                segment_id: inserter_out_seg.clone(),
                ..Default::default()
            });
            let out_dir = output_dir(output_east);
            for dx in 0..out_stop {
                entities.push(PlacedEntity {
                    name: output_belt.to_string(),
                    x: mx + dx,
                    y: output_y + 1,
                    direction: out_dir,
                    carries: Some(output_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }
        }
    }

    // Lane-split gap: y+0 pipe header already extended naturally via the
    // `header_end_x` loop above. Continue both solid belts (y+2, y+3)
    // through the gap and emit the sideload bridge at the solid output
    // belt (y+5+msz+1). Only runs when output_is_fluid=false (guarded
    // by `lane_split` itself).
    if lane_split {
        let gap_start_x = x_offset + g1 as i32 * pitch;
        let output_row_dy = 5 + msz + 1;
        stamp_lane_split_gap(
            &mut entities,
            gap_start_x,
            y_offset,
            &[
                GapRow {
                    dy: belt1_y - y_offset,
                    name: belt1,
                    direction: EntityDirection::East,
                    item: input1,
                    segment_id: belt_in1_seg.clone(),
                },
                GapRow {
                    dy: belt2_y - y_offset,
                    name: belt2,
                    direction: EntityDirection::East,
                    item: input2,
                    segment_id: belt_in2_seg.clone(),
                },
            ],
            output_row_dy,
            output_belt,
            output_item,
            output_east,
        );
    }

    let fluid_input_port_pipes = vec![(fluid_item.to_string(), x_offset, header_y)];

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Staggered 3-output layout for advanced-oil-processing-style recipes on 5×5
/// refineries. Each output fluid gets its own full-width trunk at a distinct
/// y row so flanks and drop-UGs of adjacent fluids never share a tile.
///
/// Staircase ordering (from the user's sketch in `misc/Untitled.png`):
/// - **West** (smallest dx, fluid_outputs[0]) → **lowest** (largest y)
/// - **Middle** (middle dx, fluid_outputs[1]) → **highest** (smallest y,
///   adjacent to the port row)
/// - **East** (largest dx, fluid_outputs[2]) → middle y
///
/// For refinery ports at `dx = [0, 2, 4]`, y_offset = 0, and a single machine:
///
/// ```text
///   y+0          : fluid input pipes (per-port, unchanged)
///   y+1..y+5     : oil-refinery (mirror=true, dir=North)
///   y+6          : port row — west UG-S/in, middle pipe, east UG-S/in
///   y+7 (middle) : middle L-flank · T-drop · R-flank  plus east drop UG at col_E
///   y+8 (east)   : east L-flank · T-drop · R-flank    plus west drop UG at col_W
///   y+9 (west)   : west L-flank · T-drop · R-flank
/// ```
///
/// Perpendicular-axis PTG rule (F5a) keeps fluids isolated where a drop UG
/// passes the flank column of a neighbouring trunk. Tunnel pairs:
///
/// - West fluid: machine-side UG (col_W, y+6) dir=S ↔ drop UG (col_W, y+8) dir=N
/// - East fluid: machine-side UG (col_E, y+6) dir=S ↔ drop UG (col_E, y+7) dir=N
/// - Middle fluid: direct surface pipe from (col_M, y+6) to T-drop at (col_M, y+7)
///
/// Row height = `msz + 5` (3 extra rows for the staircase vs. the single-row
/// output of the 1-fluid path).
///
/// Currently requires `machine_count == 1`; the multi-machine case has a
/// known collision (east R-flank of machine N overlaps west drop UG of
/// machine N+1 at `(mx + msz, y_east)`) that needs a separate design pass.
fn fluid_only_row_staggered_3output(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    fluid_inputs: &[(i32, &str)],
    fluid_outputs: &[(i32, &str)],
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    assert_eq!(
        fluid_outputs.len(), 3,
        "fluid_only_row_staggered_3output requires exactly 3 output ports",
    );
    assert_eq!(
        machine_count, 1,
        "staggered 3-output layout doesn't yet support multi-machine — \
         east R-flank of machine N at (mx + msz, y_east) collides with west \
         drop UG of machine N+1 at the same tile",
    );

    let msz = machine_size as i32;
    assert!(msz >= 5, "staggered 3-output requires a 5×5+ machine");

    let mut entities = Vec::new();
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let machine_seg = Some(format!("row:{recipe}:machine"));

    let mx = x_offset;
    let input_y = y_offset;
    let port_row_y = y_offset + 1 + msz;

    // --- Input side (unchanged from the non-staggered path). ---
    let input_distinct: std::collections::BTreeSet<&str> =
        fluid_inputs.iter().map(|&(_, it)| it).collect();
    if input_distinct.len() <= 1 {
        if let Some(&(_, item)) = fluid_inputs.first() {
            let seg = Some(format!("row:{recipe}:belt-in:{item}"));
            for dx in 0..msz {
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx, y: input_y,
                    carries: Some(item.to_string()),
                    segment_id: seg.clone(),
                    ..Default::default()
                });
            }
            for &(dx, port_item) in fluid_inputs {
                fluid_input_port_pipes.push((port_item.to_string(), mx + dx, input_y));
            }
        }
    } else {
        for &(dx, item) in fluid_inputs {
            let seg = Some(format!("row:{recipe}:belt-in:{item}"));
            entities.push(PlacedEntity {
                name: "pipe".to_string(),
                x: mx + dx, y: input_y,
                carries: Some(item.to_string()),
                segment_id: seg,
                ..Default::default()
            });
            fluid_input_port_pipes.push((item.to_string(), mx + dx, input_y));
        }
    }

    // --- Machine. ---
    entities.push(PlacedEntity {
        name: machine_entity.to_string(),
        x: mx,
        y: y_offset + 1,
        direction: EntityDirection::North,
        recipe: Some(recipe.to_string()),
        mirror: true,
        segment_id: machine_seg,
        ..Default::default()
    });

    // --- Output side: staggered 3 trunks. ---
    // Order: fluid_outputs sorted by dx ascending — [0]=west, [1]=middle, [2]=east.
    // (Caller in placer.rs already passes them in this order for refinery outputs.)
    let (dx_w, item_w) = fluid_outputs[0];
    let (dx_m, item_m) = fluid_outputs[1];
    let (dx_e, item_e) = fluid_outputs[2];
    let col_w = mx + dx_w;
    let col_m = mx + dx_m;
    let col_e = mx + dx_e;

    let y_middle = port_row_y + 1;
    let y_east = port_row_y + 2;
    let y_west = port_row_y + 3;

    let seg_w = Some(format!("row:{recipe}:fluid-out:{item_w}"));
    let seg_m = Some(format!("row:{recipe}:fluid-out:{item_m}"));
    let seg_e = Some(format!("row:{recipe}:fluid-out:{item_e}"));

    // Port row — one tile per fluid, each connecting to the refinery's
    // corresponding fluid box. West and east use UG-S/input so the tunnel
    // continues south to the trunk's drop UG. Middle uses a regular pipe
    // since its trunk is directly below (0-tile gap).
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_w, y: port_row_y,
        direction: EntityDirection::South,
        io_type: Some("input".to_string()),
        carries: Some(item_w.to_string()),
        segment_id: seg_w.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe".to_string(),
        x: col_m, y: port_row_y,
        carries: Some(item_m.to_string()),
        segment_id: seg_m.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_e, y: port_row_y,
        direction: EntityDirection::South,
        io_type: Some("input".to_string()),
        carries: Some(item_e.to_string()),
        segment_id: seg_e.clone(),
        ..Default::default()
    });

    // Middle trunk at y_middle (adjacent to port row).
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_m - 1, y: y_middle,
        direction: EntityDirection::West,
        io_type: Some("output".to_string()),
        carries: Some(item_m.to_string()),
        segment_id: seg_m.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe".to_string(),
        x: col_m, y: y_middle,
        carries: Some(item_m.to_string()),
        segment_id: seg_m.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_m + 1, y: y_middle,
        direction: EntityDirection::East,
        io_type: Some("input".to_string()),
        carries: Some(item_m.to_string()),
        segment_id: seg_m.clone(),
        ..Default::default()
    });

    // East drop UG (at y_middle row, col_e) — pairs with machine-side UG at port row.
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_e, y: y_east - 1, // = y_middle
        direction: EntityDirection::North,
        io_type: Some("output".to_string()),
        carries: Some(item_e.to_string()),
        segment_id: seg_e.clone(),
        ..Default::default()
    });

    // East trunk at y_east.
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_e - 1, y: y_east,
        direction: EntityDirection::West,
        io_type: Some("output".to_string()),
        carries: Some(item_e.to_string()),
        segment_id: seg_e.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe".to_string(),
        x: col_e, y: y_east,
        carries: Some(item_e.to_string()),
        segment_id: seg_e.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_e + 1, y: y_east,
        direction: EntityDirection::East,
        io_type: Some("input".to_string()),
        carries: Some(item_e.to_string()),
        segment_id: seg_e.clone(),
        ..Default::default()
    });

    // West drop UG (at y_east row, col_w) — pairs with machine-side UG at port row,
    // tunnel spans y_middle in between (perpendicular to middle trunk's PTGs, F5a).
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_w, y: y_west - 1, // = y_east
        direction: EntityDirection::North,
        io_type: Some("output".to_string()),
        carries: Some(item_w.to_string()),
        segment_id: seg_w.clone(),
        ..Default::default()
    });

    // West trunk at y_west.
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_w - 1, y: y_west,
        direction: EntityDirection::West,
        io_type: Some("output".to_string()),
        carries: Some(item_w.to_string()),
        segment_id: seg_w.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe".to_string(),
        x: col_w, y: y_west,
        carries: Some(item_w.to_string()),
        segment_id: seg_w.clone(),
        ..Default::default()
    });
    entities.push(PlacedEntity {
        name: "pipe-to-ground".to_string(),
        x: col_w + 1, y: y_west,
        direction: EntityDirection::East,
        io_type: Some("input".to_string()),
        carries: Some(item_w.to_string()),
        segment_id: seg_w,
        ..Default::default()
    });

    // Tap points at each trunk's T-drop — the ghost router connects the bus
    // lanes for each output fluid to these positions.
    fluid_output_port_pipes.push((item_w.to_string(), col_w, y_west));
    fluid_output_port_pipes.push((item_m.to_string(), col_m, y_middle));
    fluid_output_port_pipes.push((item_e.to_string(), col_e, y_east));

    // Row spans y_offset (input row) down to y_west.
    let row_height = y_west - y_offset + 1; // = msz + 5

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Row for fluid-only recipes on large machines (5×5: oil-refinery, cryogenic-plant, foundry).
///
/// Machines are placed at `direction=NORTH` with `mirror=true` so
/// fluid inputs sit at the NORTH edge (matching the bus trunk-above
/// pattern) and fluid outputs sit at the SOUTH edge.
///
/// `fluid_inputs` and `fluid_outputs` specify the port assignments as
/// `(dx_from_machine_left_edge, item_name)` pairs.  For oil-refinery (mirrored):
/// - Input ports are at dx=1 (box 1) and dx=3 (box 2).
/// - Output ports are at dx=0 (box 3), dx=2 (box 4), and dx=4 (box 5).
///
/// ## Single-fluid-per-side (simple pattern)
///
/// When all inputs carry the same fluid (len ≤ 1) and all outputs carry the
/// same fluid (len ≤ 1), each side gets a continuous east-west pipe row
/// spanning the full machine width. Adjacent machines' rows abut, giving
/// machine-to-machine connectivity and a straight run to the bus trunk with
/// no per-port isolation needed.
///
/// ```text
///   y+0     : pipe ── pipe ── pipe ── ...        ← continuous input pipe row
///   y+1..y+msz : machine (msz×msz, mirrored)
///   y+msz+1 : pipe ── pipe ── pipe ── ...        ← continuous output pipe row
/// ```
///
/// ## Multi-fluid-per-side (isolated pipes)
///
/// When a side has ≥2 distinct fluids, we fall back to per-port isolated
/// pipes — a continuous row would merge them and violate F3 (fluid isolation).
/// The proper multi-fluid pattern is the stacked-T design (see
/// `docs/archive/rfp-multi-fluid-rows.md`); until that lands, these rows will not
/// connect to the bus.
///
/// Returns `(entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)`.
/// Port pipe lists have the form `(item, x, y)`.
pub fn fluid_only_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    fluid_inputs: &[(i32, &str)],   // (dx_from_machine_left, item_name) per input port
    fluid_outputs: &[(i32, &str)],  // (dx_from_machine_left, item_name) per output port
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    // Is this row carrying a single fluid on each side? The continuous-pipe
    // simplification only applies when a side has 0 or 1 fluid.
    let input_distinct_items: std::collections::BTreeSet<&str> =
        fluid_inputs.iter().map(|&(_, it)| it).collect();
    let output_distinct_items: std::collections::BTreeSet<&str> =
        fluid_outputs.iter().map(|&(_, it)| it).collect();

    // 3 distinct fluid outputs: switch to the staggered 3-trunk staircase
    // layout (advanced-oil-processing, coal-liquefaction). Each output gets
    // its own trunk row below the machine so flanks and drop-UGs don't
    // share tiles.
    if output_distinct_items.len() >= 3 {
        return fluid_only_row_staggered_3output(
            recipe, machine_entity, machine_size, machine_count,
            y_offset, x_offset, fluid_inputs, fluid_outputs,
        );
    }

    let msz = machine_size as i32;
    let pitch = msz;
    let row_height = msz + 2;
    let mut entities = Vec::new();
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let machine_seg = Some(format!("row:{recipe}:machine"));

    let single_input_fluid = input_distinct_items.len() <= 1;
    let single_output_fluid = output_distinct_items.len() <= 1;

    let input_y = y_offset;
    let output_y = y_offset + 1 + msz;

    for i in 0..machine_count {
        let mx = x_offset + i as i32 * pitch;

        if single_input_fluid {
            // Continuous input pipe row spanning the machine's full width.
            // Even when the recipe uses only one input port (e.g. dx=3 for
            // basic-oil-processing's crude-oil), the extra pipe tiles touch
            // inactive fluid boxes which Factorio simply ignores.
            //
            // For 5×5 oil-refinery rows the continuous strip leaves no free
            // tile at `input_y` for power poles — the only y where a
            // medium-electric-pole reaches the machine center. Bridge the
            // strip with a 1-tile UG pair at dx=1 and dx=3 (the two
            // physical fluid input port positions per `fluid_ports`),
            // leaving dx=2 free for `place_poles` to drop a pole that
            // covers each refinery's center. dx=0 and dx=4 stay as regular
            // pipes so adjacent machines' strips connect on the surface.
            if let Some(&(_, item)) = fluid_inputs.first() {
                let seg = Some(format!("row:{recipe}:belt-in:{item}"));
                let gap_dx: Option<i32> = if msz == 5 && machine_entity == "oil-refinery" {
                    Some(2)
                } else {
                    None
                };
                for dx in 0..msz {
                    if Some(dx) == gap_dx {
                        continue;
                    }
                    let (name, direction, io_type) = if Some(dx + 1) == gap_dx {
                        ("pipe-to-ground", EntityDirection::East, Some("input".to_string()))
                    } else if Some(dx - 1) == gap_dx {
                        ("pipe-to-ground", EntityDirection::West, Some("output".to_string()))
                    } else {
                        ("pipe", EntityDirection::North, None)
                    };
                    entities.push(PlacedEntity {
                        name: name.to_string(),
                        x: mx + dx,
                        y: input_y,
                        direction,
                        io_type,
                        carries: Some(item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                if gap_dx == Some(2) {
                    // Report only dx=1 (the leftmost UG-bridge port) so the
                    // pole-tap reservation around the lane→port path
                    // (`hi - 1` in `layout.rs`) doesn't claim the dx=2 gap
                    // tile before `place_poles` runs. The dx=3 port is
                    // still a `pipe-to-ground` so the validator sees an
                    // adjacent pipe at both physical input ports, and
                    // fluid reaches it via the underground tunnel.
                    fluid_input_port_pipes.push((item.to_string(), mx + 1, input_y));
                } else {
                    for &(dx, port_item) in fluid_inputs {
                        fluid_input_port_pipes.push((port_item.to_string(), mx + dx, input_y));
                    }
                }
            }
        } else {
            // Multi-fluid side: keep per-port isolated pipes until the
            // stacked-T multi-fluid pattern lands.
            for &(dx, item) in fluid_inputs {
                let seg = Some(format!("row:{recipe}:belt-in:{item}"));
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: input_y,
                    carries: Some(item.to_string()),
                    segment_id: seg,
                    ..Default::default()
                });
                fluid_input_port_pipes.push((item.to_string(), mx + dx, input_y));
            }
        }

        // Machine, mirrored so inputs face north, outputs face south
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + 1,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            mirror: true,
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        if single_output_fluid {
            if let Some(&(_, item)) = fluid_outputs.first() {
                let seg = Some(format!("row:{recipe}:belt-out:{item}"));
                for dx in 0..msz {
                    entities.push(PlacedEntity {
                        name: "pipe".to_string(),
                        x: mx + dx,
                        y: output_y,
                        carries: Some(item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                for &(dx, port_item) in fluid_outputs {
                    fluid_output_port_pipes.push((port_item.to_string(), mx + dx, output_y));
                }
            }
        } else {
            for &(dx, item) in fluid_outputs {
                let seg = Some(format!("row:{recipe}:belt-out:{item}"));
                entities.push(PlacedEntity {
                    name: "pipe".to_string(),
                    x: mx + dx,
                    y: output_y,
                    carries: Some(item.to_string()),
                    segment_id: seg,
                    ..Default::default()
                });
                fluid_output_port_pipes.push((item.to_string(), mx + dx, output_y));
            }
        }
    }

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

/// Multi-fluid input row: machines that consume ≥2 distinct fluids on the
/// same face. Uses the stacked-T pattern from `docs/archive/rfp-multi-fluid-rows.md`
/// with UG-pipe-UG isolation flanks.
///
/// Currently handles **2 fluid inputs on a 3×3 chemical-plant** with no solid
/// input (heavy-oil-cracking, light-oil-cracking, sulfur). Solid output (sulfur)
/// and fluid output (cracking) both supported.
///
/// ## Geometry for N=2 fluids, chemical-plant, ports at dx=0 (fluid 0) and dx=2 (fluid 1)
///
/// ```text
/// y+0:      UG(1)e  pipe(1)  UG(1)w                  ← fluid 1 trunk (outer; T-drop at mx+2)
/// y+1:  UG(0)e  pipe(0)  UG(0)w  UG(1)s              ← fluid 0 trunk + fluid 1 drop
/// y+2:         UG(0)s                                ← fluid 0 drop
/// y+3:         UG-out(0)n        UG-out(1)n          ← surface south, adjacent to machine port
/// y+4..y+6:    ▓ machine 3×3 ▓
/// y+7..       [output inserter + belt for solid output, or pipe row for fluid output]
/// ```
///
/// Direction suffixes: `e/w/n/s` = East/West/North/South. Trunk-row UG flanks
/// face toward their T-drop so their surface-sides merge with the pipe (same
/// fluid). Drop UGs face SOUTH (surface north joins T-drop above, tunnel south
/// to UG-out below). UG-outs face NORTH (surface south joins machine port).
///
/// ## Isolation
///
/// Per [F5a](../../../../docs/factorio-mechanics.md), PTG's perpendicular sides
/// have no surface. Fluid A's east-west-oriented right flank at `(col_0+1, y+1)`
/// meets fluid B's north-south-oriented drop UG at `(col_1, y+1)` edge-to-edge
/// with no surface on either side — no cross-fluid merge.
///
/// ## Trunk feed
///
/// Caller is responsible for routing the fluid trunk at `lane.x` across to the
/// left flank of each T-drop stamp (ghost_router step 3.6 + an additional
/// horizontal UG pair). This template returns per-fluid tap positions so the
/// router can target each fluid's trunk row at the correct y.
///
/// ## Multi-machine
///
/// For `machine_count > 1`, the stamp repeats per machine. Between machines the
/// trunk on each fluid's trunk row continues east via another UG pair.
///
/// Returns `(entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)`.
pub fn fluid_multi_input_row(
    recipe: &str,
    machine_entity: &str,
    machine_size: u32,
    machine_count: usize,
    y_offset: i32,
    x_offset: i32,
    fluid_inputs: &[(i32, &str)],     // (port_dx, item) — must have ≥2 distinct items
    solid_output_item: Option<&str>,
    fluid_outputs: &[(i32, &str)],    // (port_dx, item) for fluid output(s)
    output_belt: Option<&str>,
    lane_split: bool,
    output_east: bool,
) -> (Vec<PlacedEntity>, i32, Vec<(String, i32, i32)>, Vec<(String, i32, i32)>) {
    assert!(fluid_inputs.len() >= 2, "fluid_multi_input_row requires ≥2 fluid inputs");
    assert!(
        solid_output_item.is_some() || !fluid_outputs.is_empty(),
        "fluid_multi_input_row requires either a solid output or a fluid output",
    );

    let msz = machine_size as i32;
    let pitch = msz;
    let n = fluid_inputs.len() as i32;

    // Always-gap=1 for n≥2 fluids. Two structural reasons:
    //
    // 1. Adjacent-port collision: when two fluids' `port_dx` values differ
    //    by exactly 1, the outer fluid's drop UG at `(col_t_outer,
    //    y_offset+1)` lands on the inner fluid's flank UG at
    //    `(col_t_inner ± 1, y_offset+1)`. The gap row gives the outer
    //    drop UG somewhere to sit that isn't a flank.
    //
    // 2. Bus-router lane adjacency: when the outer fluid's bus column is
    //    adjacent to the inner fluid's bus column (outer.x = inner.x + 1),
    //    the bus router places the inner fluid's branch UG-East at
    //    (inner.x + 1, inner_trunk_y) — i.e. inside the outer fluid's
    //    column. Without a gap row above the inner trunk, the outer
    //    fluid has no clear tile in its column to bridge from its own
    //    tap row down to its UG-S input. Inserting a gap gives the
    //    outer fluid a clear bridging tile at (outer.x, y_offset + 1).
    //
    // We can't predict bus column adjacency at template-emission time
    // (lane_order runs after row placement), so we always pay the
    // 1-tile cost. The existing N≥3 path with adjacent port_dx still
    // needs cumulative gap accounting; that case is unreachable for
    // currently tested recipes.
    let gap: i32 = if n >= 2 {
        1
    } else {
        0
    };
    if n > 2
        && fluid_inputs
            .windows(2)
            .any(|w| (w[1].0 - w[0].0).abs() == 1)
    {
        todo!("3+ fluid inputs with adjacent port_dx — separate fix pending");
    }

    // Row layout (offsets from y_offset), with gap=1 case noted inline:
    //   0              : outermost trunk (fluid_inputs[n-1])
    //   1              : gap row when gap=1 (otherwise inner trunk starts here)
    //   1+gap..n+gap-1 : inner trunk rows (fluid_inputs[n-2..0])
    //   n+gap          : drop UG row for the innermost fluid
    //   n+1+gap        : UG-out row (adjacent to machine port)
    //   n+2+gap..      : machine (msz rows), then output inserter + belt
    //                    (solid out) or single output pipe row (fluid out).
    //
    // With gap=0 this matches the original dense layout. With gap=1 the
    // inner trunk and everything below slide down by one; the outer trunk
    // stays at y_offset+0. Visually equivalent to "move the outer T one
    // tile higher".
    let ug_out_row_idx = n + 1 + gap;
    let machine_row_idx = n + 2 + gap;
    let output_first_idx = machine_row_idx + msz;

    let row_height = if solid_output_item.is_some() {
        output_first_idx + 2   // inserter + belt
    } else {
        output_first_idx + 1   // pipe row only
    };

    // Per-fluid trunk y positions. The outermost (fi == n-1) stays
    // anchored at y_offset+0; inner fluids shift down by `gap` so the
    // outer fluid's drop UG lands in the empty gap row.
    let trunk_ys: Vec<i32> = (0..fluid_inputs.len())
        .map(|fi| {
            let fi = fi as i32;
            y_offset
                + (n - 1 - fi)
                + if fi < n - 1 { gap } else { 0 }
        })
        .collect();

    let mut entities = Vec::new();
    let mut fluid_input_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = Vec::new();
    let machine_seg = Some(format!("row:{recipe}:machine"));
    let inserter_out_seg = Some(format!("row:{recipe}:inserter-out"));
    let belt_out_seg = Some(format!("row:{recipe}:belt-out"));

    // Lane-split only applies to solid-output recipes (the fluid-output path
    // has no belt to sideload onto). The `LANE_SPLIT_GAP` (3) between groups
    // widens each fluid's per-machine trunk-row UG pair from its original
    // adjacent layout (distance 1) to distance `pitch + LANE_SPLIT_GAP - 2
    // = 4`, well under `FLUID_UG_MAX_DISTANCE` (10) — so the fluid trunks
    // tunnel across the gap with no additional gap-filling entities on the
    // trunk rows. The drop-UG / UG-out / machine rows have no inter-machine
    // entities, so the gap stays empty there too. The bridge goes on the
    // output belt row.
    let lane_split = lane_split && machine_count >= 2 && solid_output_item.is_some();
    let mxs = machine_xs(x_offset, machine_count, pitch, lane_split);
    let g1 = if lane_split { machine_count / 2 } else { machine_count };

    for &mx in &mxs {
        // Stamp each fluid's T-drop + trunk flanks + drop UG.
        //
        // Fluid index `fi` in `fluid_inputs`. fi=0 is innermost (closest to
        // machine, lowest trunk row = y_offset + n - 1); fi=n-1 is outermost
        // (topmost, y_offset + 0).
        for (fi, &(port_dx, item)) in fluid_inputs.iter().enumerate() {
            let trunk_y = trunk_ys[fi];
            let col_t = mx + port_dx;
            let fluid_seg = Some(format!("row:{recipe}:fluid-in:{item}"));

            // Trunk row stamp: UG, pipe, UG at (col_t - 1), (col_t), (col_t + 1).
            // Left flank faces WEST (tunnel west, surface east connects to T-drop).
            // Right flank faces EAST (tunnel east, surface west connects to T-drop).
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t - 1,
                y: trunk_y,
                direction: EntityDirection::West,
                io_type: Some("output".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                name: "pipe".to_string(),
                x: col_t,
                y: trunk_y,
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t + 1,
                y: trunk_y,
                direction: EntityDirection::East,
                io_type: Some("input".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });

            // Drop UG at one row below this fluid's trunk row, at col_t.
            // Direction=SOUTH so surface faces north (merges with T-drop pipe
            // above, same fluid) and tunnel goes south to the UG-out.
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t,
                y: trunk_y + 1,
                direction: EntityDirection::South,
                io_type: Some("input".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });

            // UG-out on the machine-adjacent row, direction=NORTH so surface
            // faces south (merges with machine port at (col_t, machine_y)).
            entities.push(PlacedEntity {
                name: "pipe-to-ground".to_string(),
                x: col_t,
                y: y_offset + ug_out_row_idx,
                direction: EntityDirection::North,
                io_type: Some("output".to_string()),
                carries: Some(item.to_string()),
                segment_id: fluid_seg.clone(),
                ..Default::default()
            });

            // Report the T-drop pipe as the tap point for ghost router step 3.6.
            // Router emits the horizontal feed UG pair from lane.x to col_t - 1.
            fluid_input_port_pipes.push((item.to_string(), col_t, trunk_y));
        }

        // Machine
        entities.push(PlacedEntity {
            name: machine_entity.to_string(),
            x: mx,
            y: y_offset + machine_row_idx,
            direction: EntityDirection::North,
            recipe: Some(recipe.to_string()),
            segment_id: machine_seg.clone(),
            ..Default::default()
        });

        // Output: solid output uses inserter + belt; fluid output uses a pipe row.
        if let Some(out_item) = solid_output_item {
            let belt_name = output_belt.unwrap_or("transport-belt");
            let ins_y = y_offset + output_first_idx;
            let belt_y = ins_y + 1;
            // Output inserter (centered on machine; msz=3 so at mx+1)
            entities.push(PlacedEntity {
                name: "inserter".to_string(),
                x: mx + 1,
                y: ins_y,
                direction: EntityDirection::South,
                carries: Some(out_item.to_string()),
                segment_id: inserter_out_seg.clone(),
                ..Default::default()
            });
            // Output belt row
            let out_dir = output_dir(output_east);
            for dx in 0..msz {
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x: mx + dx,
                    y: belt_y,
                    direction: out_dir,
                    carries: Some(out_item.to_string()),
                    segment_id: belt_out_seg.clone(),
                    ..Default::default()
                });
            }
        } else {
            // Fluid output: continuous pipe row (single-fluid output branch)
            let pipe_y = y_offset + output_first_idx;
            if let Some(&(_, out_item)) = fluid_outputs.first() {
                let seg = Some(format!("row:{recipe}:fluid-out:{out_item}"));
                for dx in 0..msz {
                    entities.push(PlacedEntity {
                        name: "pipe".to_string(),
                        x: mx + dx,
                        y: pipe_y,
                        carries: Some(out_item.to_string()),
                        segment_id: seg.clone(),
                        ..Default::default()
                    });
                }
                for &(dx, item) in fluid_outputs {
                    fluid_output_port_pipes.push((item.to_string(), mx + dx, pipe_y));
                }
            }
        }
    }

    // Lane-split bridge on the solid output belt. Trunk rows need no
    // gap-filler because each fluid's per-machine UG pair naturally
    // tunnels across the widened gap (pitch + LANE_SPLIT_GAP − 2 = 4
    // tiles between endpoints, well under the 10-tile max).
    if lane_split {
        let gap_start_x = x_offset + g1 as i32 * pitch;
        let belt_name = output_belt.unwrap_or("transport-belt");
        let solid_out = solid_output_item.expect("lane_split guard ensures solid output");
        let output_row_dy = output_first_idx + 1; // inserter at output_first_idx, belt one below
        entities.extend(sideload_bridge(
            gap_start_x,
            y_offset,
            output_row_dy,
            belt_name,
            solid_out,
            output_east,
        ));
    }

    (entities, row_height, fluid_input_port_pipes, fluid_output_port_pipes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: assert at least one entity at (x, y) with given name; returns the first match.
    fn assert_entity<'a>(entities: &'a [PlacedEntity], x: i32, y: i32, name: &str) -> &'a PlacedEntity {
        let found: Vec<_> = entities.iter().filter(|e| e.x == x && e.y == y).collect();
        assert!(!found.is_empty(), "No entity at ({x}, {y}), expected '{name}'");
        assert_eq!(found[0].name, name, "Wrong entity at ({x}, {y}): got '{}', expected '{name}'", found[0].name);
        found[0]
    }

    // ---- single_input_row ----

    #[test]
    fn single_input_row_basic_entity_count() {
        // 2 machines, west-flow output, no lane split. Machine 1 gets the full
        // 9 entities (3 input + 1 inserter + 1 machine + 1 out inserter + 3 output).
        // Machine 2 (last) drops one input belt tile (east-tail orphan at mx+2)
        // and one output belt tile (west-flow orphan at mx+2): 7 entities.
        let (entities, height) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3, // machine_size
            2,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
        );
        assert_eq!(height, 7);
        assert_eq!(entities.len(), 9 + 7);
    }

    #[test]
    fn single_input_row_one_machine_positions() {
        let (entities, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0, // y_offset
            0, // x_offset
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
        );

        // Input belts at y=0: x=0,1 (x=2 is orphan east-tail, trimmed).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 0, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
            assert_eq!(e.carries.as_deref(), Some("iron-plate"));
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none(),
            "input belt at (2, 0) should be trimmed (east of inserter pickup)");

        // Inserter at (1, 1) facing SOUTH
        let ins = assert_entity(&entities, 1, 1, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("iron-plate"));

        // Machine at (0, 2) facing NORTH
        let machine = assert_entity(&entities, 0, 2, "assembling-machine-3");
        assert_eq!(machine.direction, EntityDirection::North);
        assert_eq!(machine.recipe.as_deref(), Some("iron-gear-wheel"));

        // Output inserter at (1, 5) facing SOUTH
        let out_ins = assert_entity(&entities, 1, 5, "inserter");
        assert_eq!(out_ins.direction, EntityDirection::South);
        assert_eq!(out_ins.carries.as_deref(), Some("iron-gear-wheel"));

        // Output belts at y=6: x=0,1 (x=2 is orphan east-tail for west-flow output).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 6, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
            assert_eq!(e.carries.as_deref(), Some("iron-gear-wheel"));
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 6).is_none(),
            "output belt at (2, 6) should be trimmed");
    }

    #[test]
    fn single_input_row_x_y_offset() {
        // With x_offset=6, y_offset=10, first machine should be at (6, 12).
        let (entities, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            10,
            6,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
        );
        assert_entity(&entities, 6, 12, "assembling-machine-3");
    }

    #[test]
    fn single_input_row_output_east() {
        let (entities, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            true, // output_east
        );
        // Output belts at y=6 should face EAST
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 6, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
        }
    }

    #[test]
    fn single_input_row_lane_split_two_machines() {
        // 2 machines with lane_split: machines at x=0 and x=3+3=6 (g1=1, gap_start=3)
        let (entities, height) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            2,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            true, // lane_split
            false,
        );
        assert_eq!(height, 7);

        // Machine 1 at x=0
        assert_entity(&entities, 0, 2, "assembling-machine-3");
        // Machine 2 at x=6 (g1=1, gap_start=3, gap=3, so g2_start = 3+3=6)
        assert_entity(&entities, 6, 2, "assembling-machine-3");

        // Sideload bridge: 3 input belt tiles through gap at x=3,4,5 y=0
        for dx in 3..6_i32 {
            let e = assert_entity(&entities, dx, 0, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
        }

        // Bridge entities: 6 total at gap_start_x=3
        // bridge_y = y_offset + 6 - 1 = 5
        // output_y = y_offset + 6 = 6
        // West-flowing bridge (not output_east):
        // (3, 5) SOUTH, (4, 5) WEST, (5, 5) WEST
        let b0 = assert_entity(&entities, 3, 5, "transport-belt");
        assert_eq!(b0.direction, EntityDirection::South);
        let b1 = assert_entity(&entities, 4, 5, "transport-belt");
        assert_eq!(b1.direction, EntityDirection::West);
        let b2 = assert_entity(&entities, 5, 5, "transport-belt");
        assert_eq!(b2.direction, EntityDirection::West);
        // (3, 6) WEST, (4, 6) WEST, (5, 6) NORTH
        let b3 = assert_entity(&entities, 3, 6, "transport-belt");
        assert_eq!(b3.direction, EntityDirection::West);
        let b4 = assert_entity(&entities, 4, 6, "transport-belt");
        assert_eq!(b4.direction, EntityDirection::West);
        let b5 = assert_entity(&entities, 5, 6, "transport-belt");
        assert_eq!(b5.direction, EntityDirection::North);
    }

    #[test]
    fn single_input_row_lane_split_ignored_for_one_machine() {
        // lane_split with only 1 machine should be a no-op
        let (entities_split, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            true,
            false,
        );
        let (entities_no_split, _) = single_input_row(
            "iron-gear-wheel",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            "iron-plate",
            "iron-gear-wheel",
            "transport-belt",
            "transport-belt",
            false,
            false,
        );
        assert_eq!(entities_split.len(), entities_no_split.len());
    }

    // ---- dual_input_row ----

    #[test]
    fn dual_input_row_basic() {
        let (entities, height) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false,
        );
        assert_eq!(height, 8);

        // Input belt 1 (far, y=0): only x=0 survives — long-handed inserter
        // picks at mx, so both x=1 and x=2 are east-tail orphans.
        let e = assert_entity(&entities, 0, 0, "transport-belt");
        assert_eq!(e.direction, EntityDirection::East);
        assert_eq!(e.carries.as_deref(), Some("copper-cable"));
        assert!(entities.iter().find(|e| e.x == 1 && e.y == 0).is_none());
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none());

        // Input belt 2 (close, y=1): regular inserter picks at mx+2, so all 3
        // tiles remain (trim = 0 for msz=3).
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 1, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
            assert_eq!(e.carries.as_deref(), Some("iron-plate"));
        }

        // Long-handed inserter at (0, 2) SOUTH, carries copper-cable
        let lh = assert_entity(&entities, 0, 2, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        assert_eq!(lh.carries.as_deref(), Some("copper-cable"));

        // Regular inserter at (2, 2) SOUTH, carries iron-plate
        let ri = assert_entity(&entities, 2, 2, "inserter");
        assert_eq!(ri.direction, EntityDirection::South);
        assert_eq!(ri.carries.as_deref(), Some("iron-plate"));

        // Machine at (0, 3)
        assert_entity(&entities, 0, 3, "assembling-machine-3");

        // Output inserter at (1, 6) SOUTH
        let oi = assert_entity(&entities, 1, 6, "inserter");
        assert_eq!(oi.direction, EntityDirection::South);

        // Output belts at y=7: x=0,1 (x=2 is trimmed, west-flow tail past drop).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 7, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 7).is_none());
    }

    #[test]
    fn dual_input_row_lane_split_four_machines() {
        // 4 machines with lane_split: g1=2 at x=0,3; g2=2 at x=6+3=9, 9+3=12
        // gap_start_x = 0 + 2*3 = 6
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            4,
            0,
            0,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            true,
            false,
        );

        // Machines in group 1: x=0, x=3
        assert_entity(&entities, 0, 3, "assembling-machine-3");
        assert_entity(&entities, 3, 3, "assembling-machine-3");
        // Machines in group 2: x=9, x=12
        assert_entity(&entities, 9, 3, "assembling-machine-3");
        assert_entity(&entities, 12, 3, "assembling-machine-3");

        // Both input belts span the gap (x=6,7,8 for y=0 and y=1)
        for dx in 6..9_i32 {
            assert_entity(&entities, dx, 0, "transport-belt");
            assert_entity(&entities, dx, 1, "transport-belt");
        }

        // Bridge at gap_start_x=6, output_row_dy=7:
        // bridge_y = 0 + 7 - 1 = 6
        // output_y = 0 + 7 = 7
        // (6, 6) SOUTH, (7, 6) WEST, (8, 6) WEST
        let b0 = assert_entity(&entities, 6, 6, "transport-belt");
        assert_eq!(b0.direction, EntityDirection::South);
        let b3 = assert_entity(&entities, 6, 7, "transport-belt");
        assert_eq!(b3.direction, EntityDirection::West);
    }

    #[test]
    fn tail_orphans_trimmed_dual_input_row() {
        // Regression: belts east of the last inserter pickup/drop must not be
        // stamped. Dual-input row at mx=10, msz=3: long-handed picks at mx=10,
        // regular at mx+2=12, output drop at mx+1=11.
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,   // y_offset
            10,  // x_offset
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            false, // west-flow output
        );
        // Belt 1 (y=0) — only x=10 survives; x=11 and x=12 are orphan tail.
        assert!(entities.iter().any(|e| e.x == 10 && e.y == 0 && e.carries.as_deref() == Some("copper-cable")));
        assert!(entities.iter().find(|e| e.x == 11 && e.y == 0).is_none());
        assert!(entities.iter().find(|e| e.x == 12 && e.y == 0).is_none());
        // Belt 2 (y=1) — regular picks at mx+2=12, no trim.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 1 && e.carries.as_deref() == Some("iron-plate")));
        }
        // Output belt (y=7, west-flow) — drop at mx+1=11, x=12 trimmed.
        assert!(entities.iter().any(|e| e.x == 10 && e.y == 7));
        assert!(entities.iter().any(|e| e.x == 11 && e.y == 7));
        assert!(entities.iter().find(|e| e.x == 12 && e.y == 7).is_none());
    }

    #[test]
    fn east_flow_output_tail_preserved_dual_input_row() {
        // When output flows EAST, tiles east of the drop are on the path to
        // the output merger — they must NOT be trimmed.
        let (entities, _) = dual_input_row(
            "electronic-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            10,
            ("copper-cable", "iron-plate"),
            "electronic-circuit",
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false,
            true, // east-flow output
        );
        // Output belt at y=7, x=10,11,12 — all three preserved.
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, 10 + dx, 7, "transport-belt");
            assert_eq!(e.direction, EntityDirection::East);
        }
    }

    // ---- triple_input_row ----

    #[test]
    fn triple_input_row_basic() {
        let (entities, height) = triple_input_row(
            "advanced-circuit",
            "assembling-machine-3",
            3,
            1,
            0,
            0,
            ("copper-cable", "plastic-bar", "iron-plate"),
            "advanced-circuit",
            ("transport-belt", "transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
        );
        assert_eq!(height, 9);

        // Input belt 1 at y=0 (copper-cable): only x=0 — long-handed picks at mx.
        let e = assert_entity(&entities, 0, 0, "transport-belt");
        assert_eq!(e.carries.as_deref(), Some("copper-cable"));
        assert!(entities.iter().find(|e| e.x == 1 && e.y == 0).is_none());
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 0).is_none());
        // Input belt 2 at y=1 (plastic-bar): regular picks at mx+2, all 3 tiles.
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 1, "transport-belt");
            assert_eq!(e.carries.as_deref(), Some("plastic-bar"));
        }
        // Long-handed inserter at (0, 2) SOUTH
        let lh = assert_entity(&entities, 0, 2, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        // Regular inserter at (2, 2) SOUTH
        let ri = assert_entity(&entities, 2, 2, "inserter");
        assert_eq!(ri.direction, EntityDirection::South);
        // Machine at (0, 3)
        assert_entity(&entities, 0, 3, "assembling-machine-3");
        // Output inserter at (1, 6) SOUTH
        let oi = assert_entity(&entities, 1, 6, "inserter");
        assert_eq!(oi.direction, EntityDirection::South);
        // Long-handed inserter at (2, 6) NORTH (picks iron-plate from y+8)
        let lh3 = assert_entity(&entities, 2, 6, "long-handed-inserter");
        assert_eq!(lh3.direction, EntityDirection::North);
        assert_eq!(lh3.carries.as_deref(), Some("iron-plate"));
        // Output belt at y=7: west-flow, x=0,1 (x=2 is trimmed).
        for dx in 0..2_i32 {
            let e = assert_entity(&entities, dx, 7, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
        }
        assert!(entities.iter().find(|e| e.x == 2 && e.y == 7).is_none());
        // Input belt 3 at y=8 (iron-plate): long-handed picks at mx+2, all 3 tiles.
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 8, "transport-belt");
            assert_eq!(e.carries.as_deref(), Some("iron-plate"));
        }
    }

    // ---- fluid_input_row ----

    #[test]
    fn fluid_input_row_chemical_plant() {
        // T-shape layout: 2 chemical plants, y_offset=0
        //   y+0: T-junction pipe at x=0 (trunk row, bus tap-point)
        //   y+1: UG pipe IN at x=0 facing south
        //   y+2: solid belt
        //   y+3: UG pipe OUT at x=0 + inserter at x=1
        //   y+4..y+6: machine
        //   y+7: output inserter
        //   y+8: output belt
        let (entities, height, fluid_port_pipes) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            2,
            0,
            0,
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
        );
        assert_eq!(height, 9); // msz + 6 = 3 + 6

        // fluid_port_pipes reports the TRUNK ROW position for ALL machines
        // (bus router chains horizontal PTG pairs to these T-junction pipe tiles)
        assert_eq!(fluid_port_pipes, vec![
            ("petroleum-gas".to_string(), 0, 0),
            ("petroleum-gas".to_string(), 3, 0),
        ]);

        // Machine 1: T-junction pipe at (0, 0) — trunk row
        let tj = assert_entity(&entities, 0, 0, "pipe");
        assert_eq!(tj.carries.as_deref(), Some("petroleum-gas"));

        // Machine 1: UG pipe IN at (0, 1) facing South, io=input
        let ptg_in = assert_entity(&entities, 0, 1, "pipe-to-ground");
        assert_eq!(ptg_in.direction, EntityDirection::South);
        assert_eq!(ptg_in.io_type.as_deref(), Some("input"));
        assert_eq!(ptg_in.carries.as_deref(), Some("petroleum-gas"));

        // Machine 1: solid belt at y=2
        for dx in 0..3_i32 {
            let b = assert_entity(&entities, dx, 2, "transport-belt");
            assert_eq!(b.carries.as_deref(), Some("coal"));
        }

        // Machine 1: UG pipe OUT at (0, 3) facing North (back toward input), io=output
        let ptg_out = assert_entity(&entities, 0, 3, "pipe-to-ground");
        assert_eq!(ptg_out.direction, EntityDirection::North);
        assert_eq!(ptg_out.io_type.as_deref(), Some("output"));
        assert_eq!(ptg_out.carries.as_deref(), Some("petroleum-gas"));

        // Machine 1: inserter at (1, 3) — different column from UG pipe
        let ins = assert_entity(&entities, 1, 3, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("coal"));

        // Machine 1 at (0, 4) NORTH
        let mach = assert_entity(&entities, 0, 4, "chemical-plant");
        assert_eq!(mach.direction, EntityDirection::North);

        // Machine 2: T-junction pipe at (3, 0)
        assert_entity(&entities, 3, 0, "pipe");

        // Machine 2: UG pipe IN at (3, 1) facing South
        let ptg2_in = assert_entity(&entities, 3, 1, "pipe-to-ground");
        assert_eq!(ptg2_in.direction, EntityDirection::South);
        assert_eq!(ptg2_in.io_type.as_deref(), Some("input"));

        // Machine 2: UG pipe OUT at (3, 3) facing North (back toward input)
        let ptg2_out = assert_entity(&entities, 3, 3, "pipe-to-ground");
        assert_eq!(ptg2_out.direction, EntityDirection::North);
        assert_eq!(ptg2_out.io_type.as_deref(), Some("output"));

        // Machine 2 at (3, 4)
        assert_entity(&entities, 3, 4, "chemical-plant");

        // Output inserter at y=7
        assert_entity(&entities, 1, 7, "inserter");
        // Output belt at y=8
        for dx in 0..3_i32 {
            let b = assert_entity(&entities, dx, 8, "transport-belt");
            assert_eq!(b.direction, EntityDirection::West);
        }
    }

    #[test]
    fn fluid_input_row_chemical_plant_row_height_matches_row_kind() {
        // Verify that the row_height returned by fluid_input_row matches
        // RowKind::FluidInput::row_height() for a 3x3 machine.
        let (_, height, _) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            1,
            0,
            0,
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
        );
        use crate::bus::placer::RowKind;
        assert_eq!(height, RowKind::FluidInput.row_height());
    }

    #[test]
    fn fluid_input_row_chemical_plant_ug_pair_alignment() {
        // UG pipe IN and UG pipe OUT must be in the same x column (port_dx=0),
        // and the inserter must be in a DIFFERENT column (x=1).
        let (entities, _, _) = fluid_input_row(
            "plastic-bar",
            "chemical-plant",
            3,
            1,
            5, // y_offset=5
            10, // x_offset=10
            "coal",
            "petroleum-gas",
            "plastic-bar",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
        );
        // T-junction pipe at (10, 5) — trunk row
        assert_entity(&entities, 10, 5, "pipe");
        // UG pipe IN at x=10+0=10, y=6 (trunk row + 1)
        let ptg_in = assert_entity(&entities, 10, 6, "pipe-to-ground");
        assert_eq!(ptg_in.io_type.as_deref(), Some("input"));
        // UG pipe OUT at x=10+0=10, y=8 (belt at y=7, UG OUT at y=8)
        let ptg_out = assert_entity(&entities, 10, 8, "pipe-to-ground");
        assert_eq!(ptg_out.io_type.as_deref(), Some("output"));
        // Same column
        assert_eq!(ptg_in.x, ptg_out.x, "UG pair must share the same x column");
        // Inserter at x=11 (different from UG column x=10)
        let ins = assert_entity(&entities, 11, 8, "inserter");
        assert_ne!(ins.x, ptg_in.x, "inserter must be in a different column from UG pipe");
        // Machine at (10, 9)
        assert_entity(&entities, 10, 9, "chemical-plant");
    }

    #[test]
    fn fluid_input_row_assembling_machine() {
        // AM2 uses the same T-junction pattern as chemical-plant now —
        // continuous pipe header at y+0, UG-in at y+1, solid belt at y+2,
        // UG-out + inserter at y+3. The only difference is
        // `port_dx == 1` (vs `port_dx == 0` for chemical-plant), so the
        // UG pair sits at the centre column and the inserter at
        // `mx + 0` (the first non-port column).
        let (entities, height, fluid_port_pipes) = fluid_input_row(
            "some-recipe",
            "assembling-machine-2",
            3,
            1,
            0,
            0,
            "solid-item",
            "fluid-item",
            "output-item",
            "transport-belt",
            "transport-belt",
            false, // lane_split
            false,
        );
        // Unified T-junction height: msz + 6 = 9.
        assert_eq!(height, 9);
        // fluid_port_pipes reports a tile on the y+0 header row.
        assert_eq!(fluid_port_pipes, vec![("fluid-item".to_string(), 1, 0)]);

        // Continuous pipe row at y=0 spanning the full machine width.
        for x in 0..3_i32 {
            let pipe = assert_entity(&entities, x, 0, "pipe");
            assert_eq!(pipe.carries.as_deref(), Some("fluid-item"));
        }

        // UG-in South at (mx + port_dx = 1, y = 1) — tunnels under the
        // solid belt to reach the machine's fluid port.
        let ug_in = assert_entity(&entities, 1, 1, "pipe-to-ground");
        assert_eq!(ug_in.direction, EntityDirection::South);
        assert_eq!(ug_in.io_type.as_deref(), Some("input"));

        // UG-out North at (1, 3) — surface on south side, adjacent to
        // the machine's top-centre fluid port at (1, 4).
        let ug_out = assert_entity(&entities, 1, 3, "pipe-to-ground");
        assert_eq!(ug_out.direction, EntityDirection::North);
        assert_eq!(ug_out.io_type.as_deref(), Some("output"));

        // Input inserter at (mx + 0, y + 3) — `inserter_dx = 0` because
        // `port_dx = 1`. South-facing: picks from solid belt at y+2,
        // drops into machine at y+4.
        let ins = assert_entity(&entities, 0, 3, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("solid-item"));

        // Machine occupies rows 4..=6. Ports column is 1; top face is y=4.
        assert_entity(&entities, 0, 4, "assembling-machine-2");

        // Sanity: no entity overlap on y+1 or y+3 for x in 0..3.
        for y in [1_i32, 3] {
            for x in 0..3_i32 {
                let here: Vec<_> = entities.iter().filter(|e| e.x == x && e.y == y).collect();
                assert!(
                    here.len() <= 1,
                    "multiple entities at ({x}, {y}): {:?}",
                    here.iter().map(|e| e.name.as_str()).collect::<Vec<_>>()
                );
            }
        }
    }

    // ---- fluid_dual_input_row ----

    #[test]
    fn fluid_dual_input_row_solid_output() {
        let (entities, height, fluid_in_ports, fluid_out_ports) = fluid_dual_input_row(
            "some-solid-recipe",
            "chemical-plant",
            3,
            2,
            0,
            0,
            ("input1", "input2"),
            "fluid",
            "output",
            false, // output_is_fluid = false
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
        );
        assert_eq!(height, 10);
        assert_eq!(fluid_in_ports, vec![("fluid".to_string(), 0, 0)]);
        assert!(fluid_out_ports.is_empty());

        // Fluid header at y=0, x=0..=last_mx+2 = 0..=3+2=5
        for x in 0..=5_i32 {
            let pipe = assert_entity(&entities, x, 0, "pipe");
            assert_eq!(pipe.carries.as_deref(), Some("fluid"));
        }

        // PTG input at (0+0, 1) = (0, 1) direction SOUTH for chemical-plant (port_dx=0)
        let ptg_in = assert_entity(&entities, 0, 1, "pipe-to-ground");
        assert_eq!(ptg_in.direction, EntityDirection::South);
        assert_eq!(ptg_in.io_type.as_deref(), Some("input"));

        // PTG output at (0+0, 4) = (0, 4) direction NORTH (faces input)
        let ptg_out = assert_entity(&entities, 0, 4, "pipe-to-ground");
        assert_eq!(ptg_out.direction, EntityDirection::North);
        assert_eq!(ptg_out.io_type.as_deref(), Some("output"));

        // Solid input belt 1 at y=2
        for dx in 0..3_i32 {
            assert_entity(&entities, dx, 2, "transport-belt");
        }
        // Solid input belt 2 at y=3
        for dx in 0..3_i32 {
            assert_entity(&entities, dx, 3, "transport-belt");
        }

        // Long-handed inserter at (1, 4) for chemical-plant (port_dx=0, long_x=mx+1=1)
        let lh = assert_entity(&entities, 1, 4, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        // Regular inserter at (2, 4)
        assert_entity(&entities, 2, 4, "inserter");

        // Machine at (0, 5)
        assert_entity(&entities, 0, 5, "chemical-plant");

        // Solid output: inserter at (1, 8), output belt at y=9
        assert_entity(&entities, 1, 8, "inserter");
        for dx in 0..3_i32 {
            let e = assert_entity(&entities, dx, 9, "transport-belt");
            assert_eq!(e.direction, EntityDirection::West);
        }
    }

    #[test]
    fn fluid_dual_input_row_fluid_output() {
        let (entities, height, fluid_in_ports, fluid_out_ports) = fluid_dual_input_row(
            "sulfuric-acid",
            "chemical-plant",
            3,
            1,
            0,
            0,
            ("iron-plate", "sulfur"),
            "water",
            "sulfuric-acid",
            true, // output_is_fluid = true
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
        );
        assert_eq!(height, 10);
        assert_eq!(fluid_in_ports, vec![("water".to_string(), 0, 0)]);
        // 2 port positions reported (chemical-plant fluid output boxes at
        // dx=0 and dx=2) even though the emitted pipe row is continuous.
        assert_eq!(fluid_out_ports.len(), 2);
        assert!(fluid_out_ports.contains(&("sulfuric-acid".to_string(), 0, 8)));
        assert!(fluid_out_ports.contains(&("sulfuric-acid".to_string(), 2, 8)));

        // Continuous output pipe row at y=8 spanning x=0..=2
        assert_entity(&entities, 0, 8, "pipe");
        assert_entity(&entities, 1, 8, "pipe");
        assert_entity(&entities, 2, 8, "pipe");
    }

    #[test]
    fn fluid_dual_input_row_assembling_machine_inserter_positions() {
        // assembling-machine-2 has port_dx=1, so long_x=mx+2, reg_x=mx+0
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-recipe",
            "assembling-machine-2",
            3,
            1,
            0,
            0,
            ("input1", "input2"),
            "fluid",
            "output",
            false,
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
        );
        // long-handed inserter at (2, 4), regular at (0, 4)
        let lh = assert_entity(&entities, 2, 4, "long-handed-inserter");
        assert_eq!(lh.direction, EntityDirection::South);
        let ri = assert_entity(&entities, 0, 4, "inserter");
        assert_eq!(ri.direction, EntityDirection::South);
    }

    #[test]
    fn fluid_dual_input_row_trim_chemical_plant() {
        // chemical-plant: port_dx=0 → long_dx=1, reg_dx=2.
        // Belt 1 (far, y+2) picked by long-hand at mx+1 → trim = msz-1-1 = 1.
        // Belt 2 (close, y+3) picked by regular at mx+2 → trim = 0.
        // Output (west-flow, y+9) drop at mx+1 → trim = 1.
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-solid-recipe",
            "chemical-plant",
            3,
            2,
            0,
            10, // x_offset so last_mx = 13
            ("input1", "input2"),
            "fluid",
            "output",
            false,
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false, // west-flow output
        );
        // Machine 0 (mx=10) keeps all 3 tiles on every belt row.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 2),
                "belt 1 tile at (10+{dx}, 2) should exist on non-last machine");
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 3),
                "belt 2 tile at (10+{dx}, 3) should exist on non-last machine");
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 9),
                "output tile at (10+{dx}, 9) should exist on non-last machine");
        }
        // Machine 1 (last, mx=13): belt 1 trimmed at mx+2.
        assert!(entities.iter().any(|e| e.x == 13 && e.y == 2));
        assert!(entities.iter().any(|e| e.x == 14 && e.y == 2));
        assert!(entities.iter().find(|e| e.x == 15 && e.y == 2).is_none(),
            "belt 1 tile at (15, 2) should be trimmed (east of long-hand pickup at mx+1)");
        // Belt 2: regular at mx+2=15 → no trim, all 3 tiles remain.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 13 + dx && e.y == 3),
                "belt 2 tile at (13+{dx}, 3) should survive (no trim for reg_dx=2)");
        }
        // Output west-flow: drop at mx+1=14, tile at mx+2=15 is orphan.
        assert!(entities.iter().any(|e| e.x == 13 && e.y == 9));
        assert!(entities.iter().any(|e| e.x == 14 && e.y == 9));
        assert!(entities.iter().find(|e| e.x == 15 && e.y == 9).is_none(),
            "output tile at (15, 9) should be trimmed (west-flow, east of drop)");
    }

    #[test]
    fn fluid_dual_input_row_trim_assembling_machine() {
        // assembling-machine-3: port_dx=1 → long_dx=2, reg_dx=0.
        // Belt 1 (far, y+2) picked by long-hand at mx+2 → trim = 0.
        // Belt 2 (close, y+3) picked by regular at mx+0 → trim = msz-1-0 = 2.
        // Output (west-flow, y+9) drop at mx+1 → trim = 1.
        let (entities, _, _, _) = fluid_dual_input_row(
            "some-recipe",
            "assembling-machine-3",
            3,
            2,
            0,
            10,
            ("input1", "input2"),
            "fluid",
            "output",
            false,
            ("transport-belt", "transport-belt"),
            "transport-belt",
            false, // lane_split
            false,
        );
        // Non-last machine (mx=10) keeps all tiles.
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 10 + dx && e.y == 3),
                "belt 2 tile at (10+{dx}, 3) should exist on non-last machine");
        }
        // Last machine (mx=13): belt 1 all 3 tiles (no trim).
        for dx in 0..3_i32 {
            assert!(entities.iter().any(|e| e.x == 13 + dx && e.y == 2),
                "belt 1 tile at (13+{dx}, 2) should survive (no trim for long_dx=2)");
        }
        // Belt 2: only x=13 survives — reg inserter at mx+0=13 makes 14,15 orphan.
        assert!(entities.iter().any(|e| e.x == 13 && e.y == 3));
        assert!(entities.iter().find(|e| e.x == 14 && e.y == 3).is_none(),
            "belt 2 at (14, 3) should be trimmed (reg_dx=0, trim=2)");
        assert!(entities.iter().find(|e| e.x == 15 && e.y == 3).is_none(),
            "belt 2 at (15, 3) should be trimmed (reg_dx=0, trim=2)");
    }

    // ---- fluid_only_row ----

    // basic-oil-processing: 1 fluid input (crude-oil → input box 2, dx=3),
    // 1 fluid output (petroleum-gas → output box 3, dx=0).
    #[test]
    fn fluid_only_row_one_refinery_basic() {
        let (entities, height, fluid_in, fluid_out) = fluid_only_row(
            "basic-oil-processing",
            "oil-refinery",
            5,
            1,
            0,
            0,
            &[(3, "crude-oil")],
            &[(0, "petroleum-gas")],
        );
        assert_eq!(height, 7);
        // 1 reported input port (the dx=1 UG-bridge entry) + 1 output pipe
        assert_eq!(fluid_in.len(), 1);
        assert_eq!(fluid_out.len(), 1);
        // Input port reported at dx=1 (the leftmost UG-bridge entry).
        assert_eq!(fluid_in[0], ("crude-oil".to_string(), 1, 0));
        // Output pipe at dx=0 → (0, 6)
        assert_eq!(fluid_out[0], ("petroleum-gas".to_string(), 0, 6));

        // Input row uses the pole-gap UG bridge: pipe, UG-in, GAP, UG-out, pipe.
        let in_pipe_left = assert_entity(&entities, 0, 0, "pipe");
        assert_eq!(in_pipe_left.carries.as_deref(), Some("crude-oil"));
        let in_ug_left = assert_entity(&entities, 1, 0, "pipe-to-ground");
        assert_eq!(in_ug_left.direction, EntityDirection::East);
        assert_eq!(in_ug_left.io_type.as_deref(), Some("input"));
        let in_ug_right = assert_entity(&entities, 3, 0, "pipe-to-ground");
        assert_eq!(in_ug_right.direction, EntityDirection::West);
        assert_eq!(in_ug_right.io_type.as_deref(), Some("output"));
        // dx=2 left empty for `place_poles` to drop a medium-electric-pole.
        assert!(!entities.iter().any(|e| e.x == 2 && e.y == 0));

        // Refinery at (0, 1) NORTH mirrored
        let refinery = assert_entity(&entities, 0, 1, "oil-refinery");
        assert_eq!(refinery.direction, EntityDirection::North);
        assert!(refinery.mirror);
        assert_eq!(refinery.recipe.as_deref(), Some("basic-oil-processing"));

        // Output pipe at (0, 6)
        let out_pipe = assert_entity(&entities, 0, 6, "pipe");
        assert_eq!(out_pipe.carries.as_deref(), Some("petroleum-gas"));
    }

    // basic-oil-processing with two machines
    #[test]
    fn fluid_only_row_two_refineries_basic() {
        let (entities, _, fluid_in, fluid_out) = fluid_only_row(
            "basic-oil-processing",
            "oil-refinery",
            5,
            2,
            0,
            0,
            &[(3, "crude-oil")],
            &[(0, "petroleum-gas")],
        );
        // 2 machines × 1 reported input port (dx=1) + 2 machines × 1 output pipe
        assert_eq!(fluid_in.len(), 2);
        assert_eq!(fluid_out.len(), 2);

        // Second refinery at x=5 (pitch=5)
        assert_entity(&entities, 5, 1, "oil-refinery");
        // Second machine: input port reported at mx=5, dx=1 → (6, 0)
        assert_eq!(fluid_in[1], ("crude-oil".to_string(), 6, 0));
        // Second machine: output pipe at mx=5, dx=0 → (5, 6)
        assert_eq!(fluid_out[1], ("petroleum-gas".to_string(), 5, 6));
    }

    // advanced-oil-processing: 2 fluid inputs (water→dx=1, crude-oil→dx=3),
    // 3 fluid outputs (heavy-oil→dx=0, light-oil→dx=2, petroleum-gas→dx=4).
    // Uses the staggered 3-trunk staircase output layout.
    #[test]
    fn fluid_only_row_one_refinery_advanced() {
        let (entities, height, fluid_in, fluid_out) = fluid_only_row(
            "advanced-oil-processing",
            "oil-refinery",
            5,
            1,
            0,
            0,
            &[(1, "water"), (3, "crude-oil")],
            &[(0, "heavy-oil"), (2, "light-oil"), (4, "petroleum-gas")],
        );
        // Row height grows by 3 rows (msz+5 = 10) to accommodate the staircase.
        assert_eq!(height, 10);
        assert_eq!(fluid_in.len(), 2);
        assert_eq!(fluid_out.len(), 3);

        // Input pipes — unchanged from non-staggered path.
        assert_eq!(fluid_in[0], ("water".to_string(), 1, 0));
        assert_eq!(fluid_in[1], ("crude-oil".to_string(), 3, 0));
        let water_pipe = assert_entity(&entities, 1, 0, "pipe");
        assert_eq!(water_pipe.carries.as_deref(), Some("water"));
        let crude_pipe = assert_entity(&entities, 3, 0, "pipe");
        assert_eq!(crude_pipe.carries.as_deref(), Some("crude-oil"));

        // Refinery at (0, 1).
        let refinery = assert_entity(&entities, 0, 1, "oil-refinery");
        assert_eq!(refinery.direction, EntityDirection::North);
        assert!(refinery.mirror);
        assert_eq!(refinery.recipe.as_deref(), Some("advanced-oil-processing"));

        // Port row (y=6): west UG-S/input at (0,6), middle pipe at (2,6),
        // east UG-S/input at (4,6). Each connects to its refinery output box.
        let w_port = assert_entity(&entities, 0, 6, "pipe-to-ground");
        assert_eq!(w_port.direction, EntityDirection::South);
        assert_eq!(w_port.io_type.as_deref(), Some("input"));
        assert_eq!(w_port.carries.as_deref(), Some("heavy-oil"));
        let m_port = assert_entity(&entities, 2, 6, "pipe");
        assert_eq!(m_port.carries.as_deref(), Some("light-oil"));
        let e_port = assert_entity(&entities, 4, 6, "pipe-to-ground");
        assert_eq!(e_port.direction, EntityDirection::South);
        assert_eq!(e_port.carries.as_deref(), Some("petroleum-gas"));

        // Middle trunk at y=7: L-flank (1,7), T-drop (2,7), R-flank (3,7).
        let m_l = assert_entity(&entities, 1, 7, "pipe-to-ground");
        assert_eq!(m_l.direction, EntityDirection::West);
        assert_eq!(m_l.io_type.as_deref(), Some("output"));
        let m_t = assert_entity(&entities, 2, 7, "pipe");
        assert_eq!(m_t.carries.as_deref(), Some("light-oil"));
        let m_r = assert_entity(&entities, 3, 7, "pipe-to-ground");
        assert_eq!(m_r.direction, EntityDirection::East);

        // East drop UG sits at (4,7) on the middle trunk row but on N-S axis
        // (F5a perpendicular to middle's E-W flanks — no fluid merge).
        let e_drop = assert_entity(&entities, 4, 7, "pipe-to-ground");
        assert_eq!(e_drop.direction, EntityDirection::North);
        assert_eq!(e_drop.carries.as_deref(), Some("petroleum-gas"));

        // East trunk at y=8.
        let e_l = assert_entity(&entities, 3, 8, "pipe-to-ground");
        assert_eq!(e_l.direction, EntityDirection::West);
        let e_t = assert_entity(&entities, 4, 8, "pipe");
        assert_eq!(e_t.carries.as_deref(), Some("petroleum-gas"));
        let e_r = assert_entity(&entities, 5, 8, "pipe-to-ground");
        assert_eq!(e_r.direction, EntityDirection::East);

        // West drop UG at (0,8) — N-S axis, doesn't conflict with east trunk
        // at y=8 (different columns).
        let w_drop = assert_entity(&entities, 0, 8, "pipe-to-ground");
        assert_eq!(w_drop.direction, EntityDirection::North);
        assert_eq!(w_drop.carries.as_deref(), Some("heavy-oil"));

        // West trunk at y=9.
        let w_l = assert_entity(&entities, -1, 9, "pipe-to-ground");
        assert_eq!(w_l.direction, EntityDirection::West);
        let w_t = assert_entity(&entities, 0, 9, "pipe");
        assert_eq!(w_t.carries.as_deref(), Some("heavy-oil"));
        let w_r = assert_entity(&entities, 1, 9, "pipe-to-ground");
        assert_eq!(w_r.direction, EntityDirection::East);

        // Tap points report each fluid's T-drop pipe position.
        assert_eq!(fluid_out[0], ("heavy-oil".to_string(), 0, 9));
        assert_eq!(fluid_out[1], ("light-oil".to_string(), 2, 7));
        assert_eq!(fluid_out[2], ("petroleum-gas".to_string(), 4, 8));

        // No two entities share a tile (exclude machine anchor — 5×5 footprint
        // registered once at its origin).
        use std::collections::HashMap;
        let mut by_tile: HashMap<(i32, i32), Vec<&PlacedEntity>> = HashMap::new();
        for e in &entities {
            by_tile.entry((e.x, e.y)).or_default().push(e);
        }
        for (&(x, y), es) in &by_tile {
            let non_machine: Vec<_> = es.iter().filter(|e| e.name != "oil-refinery").collect();
            assert!(
                non_machine.len() <= 1,
                "tile ({x},{y}) has {} overlapping entities: {:?}",
                non_machine.len(),
                non_machine.iter().map(|e| &e.name).collect::<Vec<_>>(),
            );
        }
    }

    // Staggered 3-output with machine_count > 1 is not yet supported —
    // east R-flank at (mx+5, y_east) overlaps next machine's west drop UG.
    #[test]
    #[should_panic(expected = "multi-machine")]
    fn fluid_only_row_advanced_multi_machine_panics() {
        let _ = fluid_only_row(
            "advanced-oil-processing",
            "oil-refinery",
            5,
            2, // multi-machine
            0,
            0,
            &[(1, "water"), (3, "crude-oil")],
            &[(0, "heavy-oil"), (2, "light-oil"), (4, "petroleum-gas")],
        );
    }

    // ---- machine_xs ----

    #[test]
    fn machine_xs_no_split() {
        let xs = machine_xs(0, 3, 3, false);
        assert_eq!(xs, vec![0, 3, 6]);
    }

    #[test]
    fn machine_xs_split_four() {
        // 4 machines, lane_split: g1=2 at 0,3; g2=2 at 6+3=9, 12
        let xs = machine_xs(0, 4, 3, true);
        assert_eq!(xs, vec![0, 3, 9, 12]);
    }

    #[test]
    fn machine_xs_split_two() {
        // 2 machines, lane_split: g1=1 at 0; g2=1 at 3+3=6
        let xs = machine_xs(0, 2, 3, true);
        assert_eq!(xs, vec![0, 6]);
    }

    #[test]
    fn machine_xs_split_ignored_for_one() {
        let xs_split = machine_xs(0, 1, 3, true);
        let xs_no_split = machine_xs(0, 1, 3, false);
        assert_eq!(xs_split, xs_no_split);
    }

    // ---- fluid_multi_input_row ----

    #[test]
    fn fluid_multi_input_heavy_oil_cracking_geometry() {
        // heavy-oil-cracking: water (dx=0) + heavy-oil (dx=2), fluid output
        // light-oil. Single chemical-plant (3×3). x_offset=0, y_offset=0.
        let (entities, row_height, in_ports, out_ports) = fluid_multi_input_row(
            "heavy-oil-cracking",
            "chemical-plant",
            3, // msz
            1, // machine_count
            0, // y_offset
            0, // x_offset
            &[(0, "water"), (2, "heavy-oil")],
            None,
            &[(1, "light-oil")],
            None,
            false, // lane_split
            true,
        );

        // Row layout for N=2, msz=3, fluid output, with always-gap=1:
        //   y=0: fluid 1 (heavy-oil) trunk at cols 1, 2, 3
        //   y=1: gap row (empty in our column; reserved for outer fluid bridging)
        //   y=2: fluid 0 (water) trunk at cols -1, 0, 1 + fluid 1 drop at col 2
        //   y=3: fluid 0 drop UG at col 0
        //   y=4: UG-out at cols 0 and 2
        //   y=5..7: machine
        //   y=8: output pipe row
        assert_eq!(row_height, 9, "expected row height 9 for pure-fluid 2-input 3×3 with gap=1");

        // Fluid 1 (heavy-oil, outermost) trunk at y=0, T-drop at col 2
        let left_flank_b = assert_entity(&entities, 1, 0, "pipe-to-ground");
        assert_eq!(left_flank_b.direction, EntityDirection::West);
        assert_eq!(left_flank_b.carries.as_deref(), Some("heavy-oil"));
        let t_drop_b = assert_entity(&entities, 2, 0, "pipe");
        assert_eq!(t_drop_b.carries.as_deref(), Some("heavy-oil"));
        let right_flank_b = assert_entity(&entities, 3, 0, "pipe-to-ground");
        assert_eq!(right_flank_b.direction, EntityDirection::East);

        // Fluid 0 (water, innermost) trunk at y=2 (was y=1 pre-gap), T-drop at col 0
        let left_flank_a = assert_entity(&entities, -1, 2, "pipe-to-ground");
        assert_eq!(left_flank_a.direction, EntityDirection::West);
        assert_eq!(left_flank_a.carries.as_deref(), Some("water"));
        let t_drop_a = assert_entity(&entities, 0, 2, "pipe");
        assert_eq!(t_drop_a.carries.as_deref(), Some("water"));
        let right_flank_a = assert_entity(&entities, 1, 2, "pipe-to-ground");
        assert_eq!(right_flank_a.direction, EntityDirection::East);

        // Fluid 1 drop UG at (col_B=2, y=1) direction=SOUTH (in gap row).
        let drop_b = assert_entity(&entities, 2, 1, "pipe-to-ground");
        assert_eq!(drop_b.direction, EntityDirection::South);
        assert_eq!(drop_b.carries.as_deref(), Some("heavy-oil"));

        // Fluid 0 drop UG at (col_A=0, y=3) direction=SOUTH
        let drop_a = assert_entity(&entities, 0, 3, "pipe-to-ground");
        assert_eq!(drop_a.direction, EntityDirection::South);
        assert_eq!(drop_a.carries.as_deref(), Some("water"));

        // UG-outs at y=4, facing NORTH
        let ug_out_a = assert_entity(&entities, 0, 4, "pipe-to-ground");
        assert_eq!(ug_out_a.direction, EntityDirection::North);
        assert_eq!(ug_out_a.carries.as_deref(), Some("water"));
        let ug_out_b = assert_entity(&entities, 2, 4, "pipe-to-ground");
        assert_eq!(ug_out_b.direction, EntityDirection::North);
        assert_eq!(ug_out_b.carries.as_deref(), Some("heavy-oil"));

        // Machine at y=5
        let m = assert_entity(&entities, 0, 5, "chemical-plant");
        assert_eq!(m.recipe.as_deref(), Some("heavy-oil-cracking"));

        // Output pipe row at y=8
        for dx in 0..3 {
            let p = assert_entity(&entities, dx, 8, "pipe");
            assert_eq!(p.carries.as_deref(), Some("light-oil"));
        }

        // Port tap points reported correctly
        assert!(in_ports.contains(&("water".to_string(), 0, 2)));
        assert!(in_ports.contains(&("heavy-oil".to_string(), 2, 0)));
        assert_eq!(out_ports, vec![("light-oil".to_string(), 1, 8)]);
    }

    #[test]
    fn fluid_multi_input_sulfur_solid_output() {
        // sulfur: water (dx=0) + petroleum-gas (dx=2), solid output (sulfur).
        let (entities, row_height, _, out_ports) = fluid_multi_input_row(
            "sulfur",
            "chemical-plant",
            3, 1, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            Some("sulfur"),
            &[],
            Some("transport-belt"),
            false, // lane_split
            false, // westward output
        );

        // Row height = 10 with always-gap=1 (one extra for inserter + belt
        // vs pipe row, on top of the +1 gap row).
        assert_eq!(row_height, 10);

        // Output inserter at (mx+1=1, y=8), facing South
        let ins = assert_entity(&entities, 1, 8, "inserter");
        assert_eq!(ins.direction, EntityDirection::South);
        assert_eq!(ins.carries.as_deref(), Some("sulfur"));

        // Output belt at y=9, direction West (output_east=false)
        for dx in 0..3 {
            let b = assert_entity(&entities, dx, 9, "transport-belt");
            assert_eq!(b.direction, EntityDirection::West);
            assert_eq!(b.carries.as_deref(), Some("sulfur"));
        }

        // Fluid output should be empty
        assert!(out_ports.is_empty());
    }

    #[test]
    fn fluid_multi_input_isolation_perpendicular_ugs() {
        // Proves the F5a isolation invariant: the tile where fluid 0's right
        // flank UG (east-west) meets fluid 1's drop UG (north-south) must
        // exist, and both must have perpendicular facing directions so their
        // shared edge has no surface on either side.
        let (entities, _, _, _) = fluid_multi_input_row(
            "heavy-oil-cracking",
            "chemical-plant",
            3, 1, 0, 0,
            &[(0, "water"), (2, "heavy-oil")],
            None,
            &[(1, "light-oil")],
            None,
            false, // lane_split
            true,
        );

        // With always-gap=1, water (fi=0, inner) trunk_y=2; its right flank
        // is at (col_t+1, 2) = (1, 2). Heavy-oil (fi=1, outer) trunk_y=0;
        // its drop UG sits in the gap row at (col_t, trunk_y+1) = (2, 1).
        // The flank and drop UG are no longer at the same y, so the
        // perpendicular-edge isolation case from gap=0 doesn't arise.
        let right_flank = entities.iter().find(|e| e.x == 1 && e.y == 2 && e.name == "pipe-to-ground").expect("right flank UG exists");
        assert!(matches!(right_flank.direction, EntityDirection::East | EntityDirection::West));
        assert_eq!(right_flank.carries.as_deref(), Some("water"));

        // Heavy-oil drop UG at (2, 1) — north-south, sits in gap row.
        let drop_b = entities.iter().find(|e| e.x == 2 && e.y == 1 && e.name == "pipe-to-ground").expect("fluid 1 drop UG exists");
        assert!(matches!(drop_b.direction, EntityDirection::North | EntityDirection::South));
        assert_eq!(drop_b.carries.as_deref(), Some("heavy-oil"));

        // Different fluids carried.
        assert_ne!(drop_b.carries, right_flank.carries);
    }

    #[test]
    fn fluid_multi_input_adjacent_ports_inserts_gap() {
        // Sulfuric-acid-on-assembling-machine-3 shape: two fluid inputs
        // whose port_dx values differ by exactly 1 (petgas=1, water=2).
        // Without the gap, fluid 1's drop UG lands on fluid 0's left-flank
        // UG at the same tile — an object collision.
        //
        // The gap=1 mechanism now applies always for n≥2 fluids (also
        // serving the bus-router lane-adjacency case), so adjacent-port
        // and non-adjacent-port layouts share the same height. This test
        // verifies the previously-colliding adjacent-port shape no longer
        // overlaps and matches the always-on baseline.
        let (entities, row_height_adj, _, _) = fluid_multi_input_row(
            "sulfuric-acid",
            "assembling-machine-3",
            3, 1, 0, 0,
            &[(2, "water"), (1, "petroleum-gas")],
            None,
            &[(1, "sulfuric-acid")],
            None,
            false, // lane_split
            true,
        );

        // Baseline layout (ports 2 apart) — same row_height under always-gap=1.
        let (_, row_height_base, _, _) = fluid_multi_input_row(
            "sulfuric-acid",
            "assembling-machine-3",
            3, 1, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            None,
            &[(1, "sulfuric-acid")],
            None,
            false, // lane_split
            true,
        );
        assert_eq!(
            row_height_adj, row_height_base,
            "adjacent-port and non-adjacent-port layouts should match under always-gap=1"
        );

        // No two entities share a tile.
        use std::collections::HashMap;
        let mut by_tile: HashMap<(i32, i32), Vec<&PlacedEntity>> = HashMap::new();
        for e in &entities {
            by_tile.entry((e.x, e.y)).or_default().push(e);
        }
        // Machines occupy a 3×3 footprint but are registered once at their
        // anchor — ignore them for the "no duplicates" check.
        for (&(x, y), es) in &by_tile {
            let non_machine: Vec<_> = es.iter().filter(|e| e.name != "assembling-machine-3").collect();
            assert!(
                non_machine.len() <= 1,
                "tile ({x},{y}) has {} overlapping entities: {:?}",
                non_machine.len(),
                non_machine.iter().map(|e| &e.name).collect::<Vec<_>>(),
            );
        }

        // Petroleum-gas is fi=1 (outer) because fluid_inputs[0]=water,
        // fluid_inputs[1]=petgas. Outer stays at y_offset+0.
        let pg_t = assert_entity(&entities, 1, 0, "pipe");
        assert_eq!(pg_t.carries.as_deref(), Some("petroleum-gas"));

        // Petgas drop UG sits in the gap row at y=1, col_t=1.
        let pg_drop = assert_entity(&entities, 1, 1, "pipe-to-ground");
        assert_eq!(pg_drop.direction, EntityDirection::South);
        assert_eq!(pg_drop.carries.as_deref(), Some("petroleum-gas"));

        // Water is fi=0 (inner), shifted down by 1 → trunk at y=2.
        let water_t = assert_entity(&entities, 2, 2, "pipe");
        assert_eq!(water_t.carries.as_deref(), Some("water"));

        // Water's left-flank UG is at (1, 2) — no longer colliding with
        // petgas's drop UG (which is now at (1, 1)).
        let water_lf = assert_entity(&entities, 1, 2, "pipe-to-ground");
        assert_eq!(water_lf.direction, EntityDirection::West);
        assert_eq!(water_lf.carries.as_deref(), Some("water"));
    }

    #[test]
    fn fluid_multi_input_multi_machine_stamps_repeat() {
        let (entities, _, in_ports, _) = fluid_multi_input_row(
            "heavy-oil-cracking",
            "chemical-plant",
            3, 2, // 2 machines
            0, 0,
            &[(0, "water"), (2, "heavy-oil")],
            None,
            &[(1, "light-oil")],
            None,
            false, // lane_split
            true,
        );

        // Second machine at mx=3. Its T-drop pipes should be at (3, 2) water (was y=1 pre-gap)
        // and (5, 0) heavy-oil (outermost trunk, unchanged).
        let water_b = assert_entity(&entities, 3, 2, "pipe");
        assert_eq!(water_b.carries.as_deref(), Some("water"));
        let ho_b = assert_entity(&entities, 5, 0, "pipe");
        assert_eq!(ho_b.carries.as_deref(), Some("heavy-oil"));

        // Second machine present at y=5 (was y=4 pre-gap)
        let m2 = assert_entity(&entities, 3, 5, "chemical-plant");
        assert_eq!(m2.recipe.as_deref(), Some("heavy-oil-cracking"));

        // 4 port tap points (2 fluids × 2 machines)
        assert_eq!(in_ports.len(), 4);
    }

    #[test]
    fn fluid_multi_input_lane_split_solid_output() {
        // Sulfur: 2 fluids in (water dx=0, petroleum-gas dx=2), solid output.
        // With lane_split and 2 machines, group 1 has machine at mx=0, group 2
        // at mx = 3 + LANE_SPLIT_GAP = 6. Each fluid's trunk-row UG pair
        // naturally tunnels across the gap (distance 4, well under the
        // 10-tile fluid UG max). The sideload bridge goes on the output belt
        // row. No gap-filling pipes on trunk rows (tunnels handle it).
        let (entities, _, _, _) = fluid_multi_input_row(
            "sulfur",
            "chemical-plant",
            3, 2, 0, 0,
            &[(0, "water"), (2, "petroleum-gas")],
            Some("sulfur"),
            &[],
            Some("transport-belt"),
            true,  // lane_split
            false, // west-flow output
        );

        // Machine positions: group 1 at x=0, group 2 at x=6, machines at y=5 (was y=4 pre-gap).
        assert_entity(&entities, 0, 5, "chemical-plant");
        assert_entity(&entities, 6, 5, "chemical-plant");

        // Water (innermost) trunk row shifts from y=1 to y=2 with always-gap=1.
        // Machine 0's east UG-in for water (port_dx=0) at (1, 2); machine 1's
        // west UG-out for water at (5, 2). Distance 4 tiles — within max_reach.
        let water_east = assert_entity(&entities, 1, 2, "pipe-to-ground");
        assert_eq!(water_east.direction, EntityDirection::East);
        assert_eq!(water_east.carries.as_deref(), Some("water"));
        let water_west = assert_entity(&entities, 5, 2, "pipe-to-ground");
        assert_eq!(water_west.direction, EntityDirection::West);
        assert_eq!(water_west.carries.as_deref(), Some("water"));

        // No trunk-row surface pipes in the gap columns (2..=4) at y=2 —
        // the UG tunnel covers it.
        for x in 2..=4 {
            assert!(
                entities.iter().find(|e| e.x == x && e.y == 2 && e.name == "pipe").is_none(),
                "expected no surface pipe at ({x}, 2) in the gap — the UG pair tunnels through"
            );
        }

        // Sideload bridge at y=8 (output inserter row) / y=9 (output belt),
        // shifted down by 1 from pre-gap layout.
        let gap_start_x = 3;
        assert_entity(&entities, gap_start_x, 8, "transport-belt");
        assert_entity(&entities, gap_start_x + 1, 8, "transport-belt");
        assert_entity(&entities, gap_start_x + 2, 8, "transport-belt");
        assert_entity(&entities, gap_start_x, 9, "transport-belt");
        assert_entity(&entities, gap_start_x + 1, 9, "transport-belt");
        assert_entity(&entities, gap_start_x + 2, 9, "transport-belt");
    }
}
