//! RFC-064 follow-up: rotation-aware rigid-row packing.
//!
//! This is deliberately separate from RFC-058's horizontal band packer.  A
//! row macro owns the machines, inserters, pipes, and the straight local belt
//! runs between its inserters.  Placement may rotate that fixed geometry, and
//! routing may choose either end of each local belt run and orient the run to
//! match.  Everything outside those local runs is replaceable connection
//! fabric.
//!
//! The first consumer is an inert Science-2 probe.  Fluid rows, direct-
//! insertion compounds, local undergrounds, and splitters fail closed rather
//! than receiving guessed transforms.

use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use crate::bus::placer::RowSpan;
use crate::common::{entity_size, is_inserter, is_machine_entity, oriented_splitter_dims};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity, SolverResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuarterTurn {
    Zero,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

impl QuarterTurn {
    fn steps(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Clockwise270 => 3,
        }
    }

    fn swaps_axes(self) -> bool {
        self.steps() % 2 == 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunRole {
    Input,
    Output,
}

#[derive(Debug, Clone)]
struct LocalBeltRun {
    item: String,
    role: RunRole,
    tiles: Vec<(i32, i32)>,
}

#[derive(Debug, Clone)]
struct RowMacro {
    recipe: String,
    width: i32,
    height: i32,
    entities: Vec<PlacedEntity>,
    belt_runs: Vec<LocalBeltRun>,
}

#[derive(Debug, Clone)]
struct OrientedRun {
    item: String,
    role: RunRole,
    tiles: Vec<(i32, i32)>,
    endpoint_a: (i32, i32),
    endpoint_b: (i32, i32),
}

#[derive(Debug, Clone)]
struct OrientedMacro {
    width: i32,
    height: i32,
    entities: Vec<PlacedEntity>,
    belt_runs: Vec<OrientedRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RotationOrder {
    Source,
    HeightDescending,
    AreaDescending,
}

/// Stable coordinates for one member of the bounded rotation-aware search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotationSelection {
    /// Bit `i` rotates source row `i` 90 degrees clockwise.
    pub rotation_mask: u64,
    pub gap: i32,
    pub target_width: i32,
    pub order: RotationOrder,
    /// Optional commodity promoted ahead of the router's default edge order.
    pub route_priority: Option<String>,
}

#[derive(Debug, Clone)]
struct RotationPlan {
    rotations: Vec<QuarterTurn>,
    origins: Vec<(i32, i32)>,
    width: i32,
    height: i32,
    gap: i32,
    target_width: i32,
    order: RotationOrder,
    estimated_transit: f64,
}

#[derive(Debug, Clone, Copy)]
struct Port {
    /// First replaceable connection-fabric tile outside the fixed belt run.
    stub: (i32, i32),
    /// Required direction at `stub`.
    direction: EntityDirection,
    owner: usize,
}

#[derive(Debug, Clone)]
struct RoutedNet {
    item: String,
    rate: f64,
    sources: Vec<Port>,
    consumers: Vec<Port>,
    external_input: bool,
    external_output: bool,
}

#[derive(Debug, Clone)]
struct PlacedMacros {
    entities: Vec<PlacedEntity>,
    nets: Vec<RoutedNet>,
}

#[derive(Debug, Clone)]
struct RouteEdge {
    item: String,
    rate: f64,
    start: Port,
    end: Port,
}

#[derive(Debug, Clone)]
struct HubPlacement {
    entities: Vec<PlacedEntity>,
    input: Port,
    outputs: Vec<Port>,
    assignment: Vec<usize>,
    score: i32,
}

#[derive(Debug, Clone)]
struct RouteWork {
    entities: Vec<PlacedEntity>,
    edges: Vec<RouteEdge>,
    boundary_inputs: Vec<crate::models::BoundaryRecord>,
    boundary_outputs: Vec<crate::models::BoundaryRecord>,
    reserved: FxHashSet<(i32, i32)>,
}

fn direction_vector(direction: EntityDirection) -> (i32, i32) {
    match direction {
        EntityDirection::North => (0, -1),
        EntityDirection::East => (1, 0),
        EntityDirection::South => (0, 1),
        EntityDirection::West => (-1, 0),
    }
}

fn vector_direction(vector: (i32, i32)) -> Result<EntityDirection, String> {
    match vector {
        (0, -1) => Ok(EntityDirection::North),
        (1, 0) => Ok(EntityDirection::East),
        (0, 1) => Ok(EntityDirection::South),
        (-1, 0) => Ok(EntityDirection::West),
        _ => Err(format!("non-cardinal direction vector {vector:?}")),
    }
}

fn rotate_direction(mut direction: EntityDirection, rotation: QuarterTurn) -> EntityDirection {
    for _ in 0..rotation.steps() {
        direction = match direction {
            EntityDirection::North => EntityDirection::East,
            EntityDirection::East => EntityDirection::South,
            EntityDirection::South => EntityDirection::West,
            EntityDirection::West => EntityDirection::North,
        };
    }
    direction
}

fn entity_dims(entity: &PlacedEntity) -> (i32, i32) {
    let (mut width, mut height) = oriented_splitter_dims(&entity.name, entity.direction)
        .unwrap_or_else(|| entity_size(&entity.name));
    if matches!(
        entity.direction,
        EntityDirection::East | EntityDirection::West
    ) && width != height
        && oriented_splitter_dims(&entity.name, entity.direction).is_none()
    {
        std::mem::swap(&mut width, &mut height);
    }
    (width as i32, height as i32)
}

fn transform_tile(tile: (i32, i32), width: i32, height: i32, rotation: QuarterTurn) -> (i32, i32) {
    match rotation {
        QuarterTurn::Zero => tile,
        QuarterTurn::Clockwise90 => (height - 1 - tile.1, tile.0),
        QuarterTurn::Clockwise180 => (width - 1 - tile.0, height - 1 - tile.1),
        QuarterTurn::Clockwise270 => (tile.1, width - 1 - tile.0),
    }
}

fn transform_entity(
    entity: &PlacedEntity,
    width: i32,
    height: i32,
    rotation: QuarterTurn,
) -> PlacedEntity {
    let (entity_width, entity_height) = entity_dims(entity);
    let (x, y) = match rotation {
        QuarterTurn::Zero => (entity.x, entity.y),
        QuarterTurn::Clockwise90 => (height - entity.y - entity_height, entity.x),
        QuarterTurn::Clockwise180 => (
            width - entity.x - entity_width,
            height - entity.y - entity_height,
        ),
        QuarterTurn::Clockwise270 => (entity.y, width - entity.x - entity_width),
    };
    let mut transformed = entity.clone();
    transformed.x = x;
    transformed.y = y;
    transformed.direction = rotate_direction(entity.direction, rotation);
    transformed
}

fn is_transport(name: &str) -> bool {
    name.ends_with("transport-belt")
        || name.ends_with("underground-belt")
        || name.ends_with("splitter")
}

fn extract_row_macros(
    rows: &[RowSpan],
    entities: &[PlacedEntity],
) -> Result<Vec<RowMacro>, String> {
    let mut macros = Vec::with_capacity(rows.len());
    for (row_index, row) in rows.iter().enumerate() {
        if row
            .spec
            .inputs
            .iter()
            .chain(&row.spec.outputs)
            .any(|flow| flow.is_fluid)
        {
            return Err(format!(
                "rotation-refusal: row {} ({}) contains fluids",
                row_index, row.spec.recipe
            ));
        }
        if !row.di_input.is_empty()
            || row.secondary_output_belt.is_some()
            || !row.sorted_output_belts.is_empty()
        {
            return Err(format!(
                "rotation-refusal: row {} ({}) is a compound/DI row",
                row_index, row.spec.recipe
            ));
        }

        let structural: Vec<&PlacedEntity> = entities
            .iter()
            .filter(|entity| {
                (is_machine_entity(&entity.name) || is_inserter(&entity.name))
                    && entity.y >= row.y_start
                    && entity.y < row.y_end
            })
            .collect();
        if structural.is_empty() {
            return Err(format!(
                "rotation-refusal: row {} ({}) has no structural entities",
                row_index, row.spec.recipe
            ));
        }
        let min_x = structural.iter().map(|entity| entity.x).min().unwrap();
        let max_x = structural
            .iter()
            .map(|entity| entity.x + entity_dims(entity).0)
            .max()
            .unwrap();

        let mut run_specs = Vec::new();
        let solid_inputs: Vec<_> = row
            .spec
            .inputs
            .iter()
            .filter(|flow| !flow.is_fluid)
            .collect();
        if solid_inputs.len() != row.input_belt_y.len() {
            return Err(format!(
                "rotation-refusal: row {} ({}) input metadata differs: {} solid flows, {} belt rows",
                row_index,
                row.spec.recipe,
                solid_inputs.len(),
                row.input_belt_y.len()
            ));
        }
        for (flow, &y) in solid_inputs.into_iter().zip(&row.input_belt_y) {
            run_specs.push((flow.item.clone(), RunRole::Input, y));
        }
        let solid_outputs: Vec<_> = row
            .spec
            .outputs
            .iter()
            .filter(|flow| !flow.is_fluid)
            .collect();
        if solid_outputs.len() != 1 {
            return Err(format!(
                "rotation-refusal: row {} ({}) has {} solid outputs; the first spike requires one",
                row_index,
                row.spec.recipe,
                solid_outputs.len()
            ));
        }
        run_specs.push((
            solid_outputs[0].item.clone(),
            RunRole::Output,
            row.output_belt_y,
        ));

        let min_y = structural
            .iter()
            .map(|entity| entity.y)
            .chain(run_specs.iter().map(|spec| spec.2))
            .min()
            .unwrap();
        let max_y = structural
            .iter()
            .map(|entity| entity.y + entity_dims(entity).1)
            .chain(run_specs.iter().map(|spec| spec.2 + 1))
            .max()
            .unwrap();
        let width = max_x - min_x;
        let height = max_y - min_y;

        // The spike owns only the straight local rows that touch the row's
        // inserters.  Transport on another y inside the structural x-range
        // is native tap/merger fabric and is intentionally ripped up.  On a
        // declared local row, however, a splitter or underground would make
        // reversal non-trivial and therefore fails closed.
        let run_ys: FxHashSet<i32> = run_specs.iter().map(|spec| spec.2).collect();
        if let Some(unexpected) = entities.iter().find(|entity| {
            let is_surface_belt = entity.name.ends_with("transport-belt")
                && !entity.name.ends_with("underground-belt");
            entity.x >= min_x
                && entity.x < max_x
                && entity.y >= min_y
                && entity.y < max_y
                && is_transport(&entity.name)
                && run_ys.contains(&entity.y)
                && !is_surface_belt
        }) {
            return Err(format!(
                "rotation-refusal: row {} ({}) contains unsupported local transport {} at ({}, {})",
                row_index, row.spec.recipe, unexpected.name, unexpected.x, unexpected.y
            ));
        }

        let mut local_entities = Vec::new();
        for entity in entities.iter().filter(|entity| {
            entity.x >= min_x
                && entity.x < max_x
                && entity.y >= min_y
                && entity.y < max_y
                && !is_transport(&entity.name)
                && !entity.name.contains("electric-pole")
        }) {
            let mut local = entity.clone();
            local.x -= min_x;
            local.y -= min_y;
            local_entities.push(local);
        }

        let belt_runs = run_specs
            .into_iter()
            .map(|(item, role, y)| LocalBeltRun {
                item,
                role,
                tiles: (0..width).map(|x| (x, y - min_y)).collect(),
            })
            .collect();
        macros.push(RowMacro {
            recipe: row.spec.recipe.clone(),
            width,
            height,
            entities: local_entities,
            belt_runs,
        });
    }
    Ok(macros)
}

fn orient_macro(row: &RowMacro, rotation: QuarterTurn) -> OrientedMacro {
    let (width, height) = if rotation.swaps_axes() {
        (row.height, row.width)
    } else {
        (row.width, row.height)
    };
    let entities = row
        .entities
        .iter()
        .map(|entity| transform_entity(entity, row.width, row.height, rotation))
        .collect();
    let belt_runs = row
        .belt_runs
        .iter()
        .map(|run| {
            let mut tiles: Vec<_> = run
                .tiles
                .iter()
                .map(|&tile| transform_tile(tile, row.width, row.height, rotation))
                .collect();
            tiles.sort_unstable();
            let horizontal = tiles.iter().all(|tile| tile.1 == tiles[0].1);
            if horizontal {
                tiles.sort_by_key(|tile| tile.0);
            } else {
                tiles.sort_by_key(|tile| tile.1);
            }
            OrientedRun {
                item: run.item.clone(),
                role: run.role,
                endpoint_a: tiles[0],
                endpoint_b: *tiles.last().unwrap(),
                tiles,
            }
        })
        .collect();
    OrientedMacro {
        width,
        height,
        entities,
        belt_runs,
    }
}

fn shelf_place(
    oriented: &[OrientedMacro],
    target_width: i32,
    gap: i32,
    order: RotationOrder,
) -> (Vec<(i32, i32)>, i32, i32) {
    let mut indices: Vec<usize> = (0..oriented.len()).collect();
    match order {
        RotationOrder::Source => {}
        RotationOrder::HeightDescending => indices
            .sort_by_key(|&index| Reverse((oriented[index].height, oriented[index].width, index))),
        RotationOrder::AreaDescending => indices.sort_by_key(|&index| {
            Reverse((
                oriented[index].width * oriented[index].height,
                oriented[index].height,
                oriented[index].width,
                index,
            ))
        }),
    }
    let mut origins = vec![(0, 0); oriented.len()];
    let (mut x, mut y, mut shelf_height) = (0, 0, 0);
    for index in indices {
        let row = &oriented[index];
        if x > 0 && x + row.width > target_width {
            x = 0;
            y += shelf_height + gap;
            shelf_height = 0;
        }
        origins[index] = (x, y);
        x += row.width + gap;
        shelf_height = shelf_height.max(row.height);
    }
    let width = oriented
        .iter()
        .zip(&origins)
        .map(|(row, origin)| origin.0 + row.width)
        .max()
        .unwrap_or(0);
    let height = oriented
        .iter()
        .zip(&origins)
        .map(|(row, origin)| origin.1 + row.height)
        .max()
        .unwrap_or(0);
    (origins, width, height)
}

fn macro_centres(oriented: &[OrientedMacro], origins: &[(i32, i32)]) -> Vec<(i32, i32)> {
    oriented
        .iter()
        .zip(origins)
        .map(|(row, origin)| (origin.0 + row.width / 2, origin.1 + row.height / 2))
        .collect()
}

fn estimate_transit(
    rows: &[RowSpan],
    solver_result: &SolverResult,
    oriented: &[OrientedMacro],
    origins: &[(i32, i32)],
) -> f64 {
    let centres = macro_centres(oriented, origins);
    let mut producers: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    for (index, row) in rows.iter().enumerate() {
        for output in row.spec.outputs.iter().filter(|flow| !flow.is_fluid) {
            producers.entry(&output.item).or_default().push(index);
        }
    }
    let mut cost = 0.0;
    for (consumer, row) in rows.iter().enumerate() {
        for input in row.spec.inputs.iter().filter(|flow| !flow.is_fluid) {
            let rate = input.rate * row.machine_count as f64;
            if let Some(sources) = producers.get(input.item.as_str()) {
                let distance = sources
                    .iter()
                    .map(|&source| {
                        (centres[source].0 - centres[consumer].0).abs()
                            + (centres[source].1 - centres[consumer].1).abs()
                    })
                    .min()
                    .unwrap_or(0);
                cost += rate * distance as f64;
            } else {
                cost += rate * (centres[consumer].0 + 4) as f64;
            }
        }
    }
    for output in &solver_result.external_outputs {
        if let Some(sources) = producers.get(output.item.as_str()) {
            let distance = sources
                .iter()
                .map(|&source| centres[source].0 + 4)
                .min()
                .unwrap_or(0);
            cost += output.rate * distance as f64;
        }
    }
    cost
}

/// Hard bound on rows entering the rotation search.
///
/// [`enumerate_rotation_plans`] enumerates `2^count` rotation masks and
/// multiplies each by the gap (7), target-width and shelf-order loops, so the
/// row count is the sole exponential term. Two separate reasons to bound it:
///
///  1. **Overflow.** `1usize << count` is undefined for `count >= 64` — a
///     panic in debug, a silent wrap to `1usize << 0 == 1` in release, which
///     would quietly search exactly one mask and report a "complete" search.
///  2. **Tractability.** 64 does not bound anything useful; `2^64` masks are
///     unreachable by many orders of magnitude. Even 2^20 masks times the
///     inner loops is already far past practical. The bound below is a
///     tractability limit that happens to also make (1) unreachable.
///
/// The frozen Science-2 fixture this search is currently gated to runs well
/// under this; the bound exists so an unfrozen future caller fails loudly
/// instead of hanging or silently truncating its own search.
const MAX_ROTATION_ROWS: usize = 20;

/// Shared refusal for both [`build_rotation_aware_layout`] and
/// [`build_rotation_aware_layout_selected`]. Previously only the `_selected`
/// entry point checked this, and it checked `> u64::BITS` — off by one, so
/// exactly 64 rows passed the guard and then overflowed the shift. The
/// unselected path had no check at all. (PR #575 bot review.)
fn check_rotation_row_budget(rows: &[RowSpan]) -> Result<(), String> {
    if rows.len() > MAX_ROTATION_ROWS {
        return Err(format!(
            "rotation-refusal: {} rows exceed the {MAX_ROTATION_ROWS}-row rotation search bound",
            rows.len()
        ));
    }
    Ok(())
}

fn enumerate_rotation_plans(
    macros: &[RowMacro],
    rows: &[RowSpan],
    solver_result: &SolverResult,
) -> Vec<RotationPlan> {
    let count = macros.len();
    // Both callers gate on `check_rotation_row_budget` first. This catches a
    // future third caller that forgets to, in tests, rather than at 2^count.
    debug_assert!(
        count <= MAX_ROTATION_ROWS,
        "enumerate_rotation_plans called with {count} rows, above the \
         {MAX_ROTATION_ROWS}-row bound — caller skipped check_rotation_row_budget"
    );
    let mut plans = Vec::new();
    for mask in 0usize..(1usize << count) {
        let rotations: Vec<_> = (0..count)
            .map(|index| {
                if mask & (1 << index) == 0 {
                    QuarterTurn::Zero
                } else {
                    QuarterTurn::Clockwise90
                }
            })
            .collect();
        let oriented: Vec<_> = macros
            .iter()
            .zip(&rotations)
            .map(|(row, &rotation)| orient_macro(row, rotation))
            .collect();
        let widest = oriented.iter().map(|row| row.width).max().unwrap_or(1);
        let total_width: i32 =
            oriented.iter().map(|row| row.width).sum::<i32>() + 12 * count as i32;
        for gap in 4..=10 {
            let mut target_width = widest;
            while target_width <= total_width.min(96) {
                for order in [
                    RotationOrder::Source,
                    RotationOrder::HeightDescending,
                    RotationOrder::AreaDescending,
                ] {
                    let (origins, width, height) = shelf_place(&oriented, target_width, gap, order);
                    let aspect = width.max(height) as f64 / width.min(height).max(1) as f64;
                    if aspect > 1.75 {
                        continue;
                    }
                    plans.push(RotationPlan {
                        rotations: rotations.clone(),
                        estimated_transit: estimate_transit(
                            rows,
                            solver_result,
                            &oriented,
                            &origins,
                        ),
                        origins,
                        width,
                        height,
                        gap,
                        target_width,
                        order,
                    });
                }
                target_width += 2;
            }
        }
    }
    plans.sort_by(|a, b| {
        let a_aspect = a.width.max(a.height) as f64 / a.width.min(a.height).max(1) as f64;
        let b_aspect = b.width.max(b.height) as f64 / b.width.min(b.height).max(1) as f64;
        a_aspect
            .total_cmp(&b_aspect)
            .then_with(|| a.estimated_transit.total_cmp(&b.estimated_transit))
            .then_with(|| (a.width * a.height).cmp(&(b.width * b.height)))
            .then_with(|| a.gap.cmp(&b.gap))
            .then_with(|| a.target_width.cmp(&b.target_width))
            .then_with(|| a.order.cmp(&b.order))
    });
    plans.dedup_by(|a, b| a.rotations == b.rotations && a.origins == b.origins && a.gap == b.gap);
    plans
}

fn average_point(points: impl Iterator<Item = (i32, i32)>) -> Option<(i32, i32)> {
    let points: Vec<_> = points.collect();
    if points.is_empty() {
        None
    } else {
        Some((
            points.iter().map(|point| point.0).sum::<i32>() / points.len() as i32,
            points.iter().map(|point| point.1).sum::<i32>() / points.len() as i32,
        ))
    }
}

fn choose_run_port(
    run: &OrientedRun,
    origin: (i32, i32),
    desired: (i32, i32),
    owner: usize,
) -> Result<(Port, Vec<PlacedEntity>), String> {
    let endpoint_a = (origin.0 + run.endpoint_a.0, origin.1 + run.endpoint_a.1);
    let endpoint_b = (origin.0 + run.endpoint_b.0, origin.1 + run.endpoint_b.1);
    let distance = |point: (i32, i32)| (point.0 - desired.0).abs() + (point.1 - desired.1).abs();
    let choose_a = distance(endpoint_a) <= distance(endpoint_b);
    let chosen = if choose_a { endpoint_a } else { endpoint_b };
    let other = if choose_a { endpoint_b } else { endpoint_a };
    let toward_other = ((other.0 - chosen.0).signum(), (other.1 - chosen.1).signum());
    let flow_vector = match run.role {
        RunRole::Input => toward_other,
        RunRole::Output => (-toward_other.0, -toward_other.1),
    };
    let direction = vector_direction(flow_vector)?;
    let outward = match run.role {
        RunRole::Input => (-flow_vector.0, -flow_vector.1),
        RunRole::Output => flow_vector,
    };
    let port = Port {
        stub: (chosen.0 + outward.0, chosen.1 + outward.1),
        direction,
        owner,
    };
    let belts = run
        .tiles
        .iter()
        .map(|tile| PlacedEntity {
            name: "express-transport-belt".to_string(),
            x: origin.0 + tile.0,
            y: origin.1 + tile.1,
            direction,
            carries: Some(run.item.clone()),
            ..Default::default()
        })
        .collect();
    Ok((port, belts))
}

fn place_macros(
    macros: &[RowMacro],
    rows: &[RowSpan],
    solver_result: &SolverResult,
    plan: &RotationPlan,
) -> Result<PlacedMacros, String> {
    const MARGIN: i32 = 14;
    let oriented: Vec<_> = macros
        .iter()
        .zip(&plan.rotations)
        .map(|(row, &rotation)| orient_macro(row, rotation))
        .collect();
    let origins: Vec<_> = plan
        .origins
        .iter()
        .map(|origin| (origin.0 + MARGIN, origin.1 + MARGIN))
        .collect();
    let centres = macro_centres(&oriented, &origins);

    let mut producer_rows: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    let mut consumer_rows: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    let mut rates: FxHashMap<&str, f64> = FxHashMap::default();
    for (row_index, row) in rows.iter().enumerate() {
        for output in row.spec.outputs.iter().filter(|flow| !flow.is_fluid) {
            producer_rows
                .entry(&output.item)
                .or_default()
                .push(row_index);
            *rates.entry(&output.item).or_default() += output.rate * row.machine_count as f64;
        }
        for input in row.spec.inputs.iter().filter(|flow| !flow.is_fluid) {
            consumer_rows
                .entry(&input.item)
                .or_default()
                .push(row_index);
        }
    }
    for input in &solver_result.external_inputs {
        rates.entry(&input.item).or_insert(input.rate);
    }

    let mut entities = Vec::new();
    for (row, origin) in oriented.iter().zip(&origins) {
        for entity in &row.entities {
            let mut placed = entity.clone();
            placed.x += origin.0;
            placed.y += origin.1;
            entities.push(placed);
        }
    }

    let mut sources: BTreeMap<String, Vec<Port>> = BTreeMap::new();
    let mut consumers: BTreeMap<String, Vec<Port>> = BTreeMap::new();
    for (macro_index, (row, origin)) in oriented.iter().zip(&origins).enumerate() {
        for run in &row.belt_runs {
            let desired = match run.role {
                RunRole::Input => average_point(
                    producer_rows
                        .get(run.item.as_str())
                        .into_iter()
                        .flatten()
                        .map(|&index| centres[index]),
                ),
                RunRole::Output => average_point(
                    consumer_rows
                        .get(run.item.as_str())
                        .into_iter()
                        .flatten()
                        .map(|&index| centres[index]),
                ),
            }
            .unwrap_or((MARGIN - 8, centres[macro_index].1));
            let (port, belts) = choose_run_port(run, *origin, desired, macro_index)?;
            entities.extend(belts);
            match run.role {
                RunRole::Input => consumers.entry(run.item.clone()).or_default().push(port),
                RunRole::Output => sources.entry(run.item.clone()).or_default().push(port),
            }
        }
    }

    let external_inputs: FxHashSet<&str> = solver_result
        .external_inputs
        .iter()
        .map(|flow| flow.item.as_str())
        .collect();
    let external_outputs: FxHashSet<&str> = solver_result
        .external_outputs
        .iter()
        .map(|flow| flow.item.as_str())
        .collect();
    let mut items: FxHashSet<String> = sources.keys().cloned().collect();
    items.extend(consumers.keys().cloned());
    items.extend(external_inputs.iter().map(|item| (*item).to_string()));
    items.extend(external_outputs.iter().map(|item| (*item).to_string()));
    let mut items: Vec<_> = items.into_iter().collect();
    items.sort();
    let mut nets = Vec::new();
    for (owner, item) in items.into_iter().enumerate() {
        let mut item_sources = sources.remove(&item).unwrap_or_default();
        let mut item_consumers = consumers.remove(&item).unwrap_or_default();
        for port in item_sources.iter_mut().chain(&mut item_consumers) {
            port.owner = owner;
        }
        nets.push(RoutedNet {
            rate: *rates.get(item.as_str()).unwrap_or(&0.0),
            external_input: external_inputs.contains(item.as_str()),
            external_output: external_outputs.contains(item.as_str()),
            item,
            sources: item_sources,
            consumers: item_consumers,
        });
    }
    Ok(PlacedMacros { entities, nets })
}

fn rotate_template_tile(
    point: (i32, i32),
    width: i32,
    height: i32,
    direction: EntityDirection,
) -> (i32, i32) {
    match direction {
        EntityDirection::South => point,
        EntityDirection::North => (width - 1 - point.0, height - 1 - point.1),
        EntityDirection::East => (point.1, width - 1 - point.0),
        EntityDirection::West => (height - 1 - point.1, point.0),
    }
}

fn rotate_template_direction(direction: EntityDirection, flow: EntityDirection) -> EntityDirection {
    let value = direction as u8;
    let rotated = match flow {
        EntityDirection::South => value,
        EntityDirection::North => (value + 8) % 16,
        EntityDirection::East => (value + 12) % 16,
        EntityDirection::West => (value + 4) % 16,
    };
    match rotated {
        0 => EntityDirection::North,
        4 => EntityDirection::East,
        8 => EntityDirection::South,
        12 => EntityDirection::West,
        _ => unreachable!("cardinal rotation produced {rotated}"),
    }
}

fn stamp_rotated_balancer(
    template: &crate::bus::balancer_library::BalancerTemplate,
    origin: (i32, i32),
    flow: EntityDirection,
    item: &str,
) -> (Vec<PlacedEntity>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    use crate::bus::balancer::{splitter_for_belt, underground_for_belt};

    let width = template.width as i32;
    let height = template.height as i32;
    let mut entities = template.stamp(
        0,
        0,
        "express-transport-belt",
        splitter_for_belt("express-transport-belt"),
        underground_for_belt("express-transport-belt"),
        Some(item),
    );
    for entity in &mut entities {
        let (entity_width, entity_height) = entity_dims(entity);
        let mut rotated_tiles = Vec::new();
        for x in entity.x..entity.x + entity_width {
            for y in entity.y..entity.y + entity_height {
                rotated_tiles.push(rotate_template_tile((x, y), width, height, flow));
            }
        }
        entity.x = origin.0 + rotated_tiles.iter().map(|tile| tile.0).min().unwrap();
        entity.y = origin.1 + rotated_tiles.iter().map(|tile| tile.1).min().unwrap();
        entity.direction = rotate_template_direction(entity.direction, flow);
    }
    let rotate_ports = |ports: &[(i32, i32)]| {
        ports
            .iter()
            .map(|&point| {
                let point = rotate_template_tile(point, width, height, flow);
                (origin.0 + point.0, origin.1 + point.1)
            })
            .collect()
    };
    (
        entities,
        rotate_ports(template.input_tiles),
        rotate_ports(template.output_tiles),
    )
}

fn occupied_tiles(entities: &[PlacedEntity]) -> FxHashSet<(i32, i32)> {
    let mut occupied = FxHashSet::default();
    for entity in entities {
        let (width, height) = entity_dims(entity);
        for dx in 0..width {
            for dy in 0..height {
                occupied.insert((entity.x + dx, entity.y + dy));
            }
        }
    }
    occupied
}

fn permutations(count: usize) -> Vec<Vec<usize>> {
    fn visit(prefix: &mut Vec<usize>, remaining: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if remaining.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for index in 0..remaining.len() {
            let value = remaining.remove(index);
            prefix.push(value);
            visit(prefix, remaining, out);
            prefix.pop();
            remaining.insert(index, value);
        }
    }
    let mut out = Vec::new();
    visit(&mut Vec::new(), &mut (0..count).collect(), &mut out);
    out
}

fn place_fanout_hub(
    net: &RoutedNet,
    existing: &[PlacedEntity],
    reserved: &FxHashSet<(i32, i32)>,
) -> Result<HubPlacement, String> {
    if net.sources.len() != 1 || net.consumers.len() < 2 {
        return Err(format!(
            "net {}: fanout hub requires one source and at least two consumers",
            net.item
        ));
    }
    let count = net.consumers.len();
    let template = crate::bus::balancer_library::balancer_templates()
        .get(&(1, count as u32))
        .ok_or_else(|| format!("net {}: no 1x{count} balancer template", net.item))?;
    let occupied = occupied_tiles(existing);
    let min_x = existing.iter().map(|entity| entity.x).min().unwrap_or(0) - 8;
    let min_y = existing.iter().map(|entity| entity.y).min().unwrap_or(0) - 8;
    let max_x = existing
        .iter()
        .map(|entity| entity.x + entity_dims(entity).0)
        .max()
        .unwrap_or(0)
        + 8;
    let max_y = existing
        .iter()
        .map(|entity| entity.y + entity_dims(entity).1)
        .max()
        .unwrap_or(0)
        + 8;
    let assignments = permutations(count);
    let mut best: Option<HubPlacement> = None;
    for flow in [
        EntityDirection::North,
        EntityDirection::East,
        EntityDirection::South,
        EntityDirection::West,
    ] {
        let (hub_width, hub_height) =
            if matches!(flow, EntityDirection::East | EntityDirection::West) {
                (template.height as i32, template.width as i32)
            } else {
                (template.width as i32, template.height as i32)
            };
        for y in min_y..=max_y - hub_height {
            for x in min_x..=max_x - hub_width {
                // Treat the template bbox plus one tile as rigid.  This is
                // intentionally more conservative than its sparse entity
                // mask: a hub hidden in a machine's hollow bbox is not a
                // useful first-spike success.
                let clear = (x - 1..=x + hub_width).all(|tx| {
                    (y - 1..=y + hub_height)
                        .all(|ty| !occupied.contains(&(tx, ty)) && !reserved.contains(&(tx, ty)))
                });
                if !clear {
                    continue;
                }
                let (entities, inputs, outputs) =
                    stamp_rotated_balancer(template, (x, y), flow, &net.item);
                if inputs.len() != 1 || outputs.len() != count {
                    continue;
                }
                let vector = direction_vector(flow);
                let input = Port {
                    stub: (inputs[0].0 - vector.0, inputs[0].1 - vector.1),
                    direction: flow,
                    owner: net.sources[0].owner,
                };
                let output_ports: Vec<_> = outputs
                    .iter()
                    .map(|output| Port {
                        stub: (output.0 + vector.0, output.1 + vector.1),
                        direction: flow,
                        owner: net.sources[0].owner,
                    })
                    .collect();
                if occupied.contains(&input.stub)
                    || reserved.contains(&input.stub)
                    || output_ports
                        .iter()
                        .any(|port| occupied.contains(&port.stub) || reserved.contains(&port.stub))
                {
                    continue;
                }
                for assignment in &assignments {
                    let mut score = (net.sources[0].stub.0 - input.stub.0).abs()
                        + (net.sources[0].stub.1 - input.stub.1).abs();
                    for (output_index, &consumer_index) in assignment.iter().enumerate() {
                        score += (output_ports[output_index].stub.0
                            - net.consumers[consumer_index].stub.0)
                            .abs()
                            + (output_ports[output_index].stub.1
                                - net.consumers[consumer_index].stub.1)
                                .abs();
                    }
                    let candidate = HubPlacement {
                        entities: entities.clone(),
                        input,
                        outputs: output_ports.clone(),
                        assignment: assignment.clone(),
                        score,
                    };
                    if best.as_ref().is_none_or(|current| {
                        candidate.score < current.score
                            || (candidate.score == current.score
                                && (candidate.entities[0].y, candidate.entities[0].x)
                                    < (current.entities[0].y, current.entities[0].x))
                    }) {
                        best = Some(candidate);
                    }
                }
            }
        }
    }
    best.ok_or_else(|| format!("net {}: no collision-free fanout hub", net.item))
}

fn assemble_route_work(placed: PlacedMacros) -> Result<RouteWork, String> {
    let mut entities = placed.entities;
    let mut reserved: FxHashSet<(i32, i32)> = placed
        .nets
        .iter()
        .flat_map(|net| net.sources.iter().chain(&net.consumers))
        .map(|port| port.stub)
        .collect();
    let mut hubs: BTreeMap<String, HubPlacement> = BTreeMap::new();
    let mut fanouts: Vec<_> = placed
        .nets
        .iter()
        .filter(|net| net.consumers.len() >= 2)
        .collect();
    fanouts.sort_by(|a, b| {
        b.consumers
            .len()
            .cmp(&a.consumers.len())
            .then_with(|| b.rate.total_cmp(&a.rate))
            .then_with(|| a.item.cmp(&b.item))
    });
    for net in fanouts {
        if net.external_input || net.external_output {
            return Err(format!(
                "net {}: the first rotation spike does not combine fanout with an external boundary",
                net.item
            ));
        }
        let hub = place_fanout_hub(net, &entities, &reserved)?;
        reserved.insert(hub.input.stub);
        reserved.extend(hub.outputs.iter().map(|port| port.stub));
        entities.extend(hub.entities.clone());
        hubs.insert(net.item.clone(), hub);
    }

    let boundary_x = entities.iter().map(|entity| entity.x).min().unwrap_or(0) - 8;
    let mut edges = Vec::new();
    let mut boundary_inputs = Vec::new();
    let mut boundary_outputs = Vec::new();
    for net in &placed.nets {
        if net.sources.len() > 1 {
            return Err(format!(
                "net {}: {} producers require a collector; the first rotation spike supports one",
                net.item,
                net.sources.len()
            ));
        }
        if net.sources.is_empty() != net.external_input {
            return Err(format!(
                "net {}: producer/external-input contract is ambiguous",
                net.item
            ));
        }
        let source = if let Some(&source) = net.sources.first() {
            source
        } else {
            if net.consumers.len() != 1 {
                return Err(format!(
                    "net {}: external fanout is outside the first rotation spike",
                    net.item
                ));
            }
            let source = Port {
                stub: (boundary_x, net.consumers[0].stub.1),
                direction: EntityDirection::East,
                owner: net.consumers[0].owner,
            };
            reserved.insert(source.stub);
            boundary_inputs.push(crate::models::BoundaryRecord {
                item: net.item.clone(),
                x: source.stub.0,
                y: source.stub.1,
                direction: source.direction,
                is_fluid: false,
                entity: "express-transport-belt".to_string(),
            });
            source
        };

        if let Some(hub) = hubs.get(&net.item) {
            edges.push(RouteEdge {
                item: net.item.clone(),
                rate: net.rate,
                start: source,
                end: hub.input,
            });
            for (output_index, &consumer_index) in hub.assignment.iter().enumerate() {
                edges.push(RouteEdge {
                    item: net.item.clone(),
                    rate: net.rate / net.consumers.len() as f64,
                    start: hub.outputs[output_index],
                    end: net.consumers[consumer_index],
                });
            }
        } else if let Some(&consumer) = net.consumers.first() {
            edges.push(RouteEdge {
                item: net.item.clone(),
                rate: net.rate,
                start: source,
                end: consumer,
            });
        }

        if net.external_output {
            if !net.consumers.is_empty() {
                return Err(format!(
                    "net {}: simultaneous internal and external consumers are outside the first rotation spike",
                    net.item
                ));
            }
            let boundary = Port {
                stub: (boundary_x, source.stub.1),
                direction: EntityDirection::West,
                owner: source.owner,
            };
            reserved.insert(boundary.stub);
            boundary_outputs.push(crate::models::BoundaryRecord {
                item: net.item.clone(),
                x: boundary.stub.0,
                y: boundary.stub.1,
                direction: boundary.direction,
                is_fluid: false,
                entity: "express-transport-belt".to_string(),
            });
            edges.push(RouteEdge {
                item: net.item.clone(),
                rate: net.rate,
                start: source,
                end: boundary,
            });
        } else if net.consumers.is_empty() {
            return Err(format!("net {}: has no consumer", net.item));
        }
    }
    edges.sort_by(|a, b| {
        let distance = |edge: &RouteEdge| {
            (edge.start.stub.0 - edge.end.stub.0).abs()
                + (edge.start.stub.1 - edge.end.stub.1).abs()
        };
        distance(b)
            .cmp(&distance(a))
            .then_with(|| b.rate.total_cmp(&a.rate))
            .then_with(|| a.item.cmp(&b.item))
    });
    Ok(RouteWork {
        entities,
        edges,
        boundary_inputs,
        boundary_outputs,
        reserved,
    })
}

const OCC_BLOCK: u8 = 1;
const OCC_HORIZONTAL: u8 = 2;
const OCC_VERTICAL: u8 = 4;

fn transport_axis(direction: EntityDirection) -> u8 {
    match direction {
        EntityDirection::East | EntityDirection::West => OCC_HORIZONTAL,
        EntityDirection::North | EntityDirection::South => OCC_VERTICAL,
    }
}

fn build_occupancy(entities: &[PlacedEntity]) -> FxHashMap<(i32, i32), u8> {
    let mut occupancy = FxHashMap::default();
    for entity in entities {
        let (width, height) = entity_dims(entity);
        let is_plain_belt =
            entity.name.ends_with("transport-belt") && !entity.name.ends_with("underground-belt");
        let bits = if is_plain_belt {
            transport_axis(entity.direction)
        } else {
            OCC_BLOCK
        };
        for dx in 0..width {
            for dy in 0..height {
                *occupancy.entry((entity.x + dx, entity.y + dy)).or_insert(0) |= bits;
            }
        }
    }
    occupancy
}

fn direction_index(direction: EntityDirection) -> u8 {
    match direction {
        EntityDirection::North => 0,
        EntityDirection::East => 1,
        EntityDirection::South => 2,
        EntityDirection::West => 3,
    }
}

fn index_direction(index: u8) -> EntityDirection {
    match index {
        0 => EntityDirection::North,
        1 => EntityDirection::East,
        2 => EntityDirection::South,
        3 => EntityDirection::West,
        _ => unreachable!("direction index {index}"),
    }
}

fn route_edge_path(
    edge: &RouteEdge,
    occupancy: &FxHashMap<(i32, i32), u8>,
    belt_dirs: &FxHashMap<(i32, i32), (EntityDirection, Option<String>)>,
    reserved: &FxHashSet<(i32, i32)>,
    bounds: ((i32, i32), (i32, i32)),
) -> Result<Vec<(i32, i32)>, String> {
    type State = ((i32, i32), u8);
    let start = (edge.start.stub, direction_index(edge.start.direction));
    let target_direction = direction_index(edge.end.direction);
    let heuristic =
        |tile: (i32, i32)| (tile.0 - edge.end.stub.0).abs() + (tile.1 - edge.end.stub.1).abs();
    let mut open: BinaryHeap<Reverse<(i32, i32, (i32, i32), u8)>> = BinaryHeap::new();
    let mut best: FxHashMap<State, i32> = FxHashMap::default();
    let mut parent: FxHashMap<State, State> = FxHashMap::default();
    best.insert(start, 0);
    open.push(Reverse((heuristic(start.0), 0, start.0, start.1)));

    let passable = |tile: (i32, i32), direction: EntityDirection| {
        if tile.0 < bounds.0 .0
            || tile.1 < bounds.0 .1
            || tile.0 > bounds.1 .0
            || tile.1 > bounds.1 .1
        {
            return false;
        }
        if reserved.contains(&tile) && tile != edge.start.stub && tile != edge.end.stub {
            return false;
        }
        let bits = occupancy.get(&tile).copied().unwrap_or(0);
        if bits & OCC_BLOCK != 0 {
            return false;
        }
        let axis = transport_axis(direction);
        bits == 0 || bits & axis == 0
    };
    let fed_by_foreign = |tile: (i32, i32)| {
        [
            EntityDirection::North,
            EntityDirection::East,
            EntityDirection::South,
            EntityDirection::West,
        ]
        .iter()
        .any(|&direction| {
            let vector = direction_vector(direction);
            let neighbour = (tile.0 - vector.0, tile.1 - vector.1);
            belt_dirs
                .get(&neighbour)
                .is_some_and(|(belt_direction, item)| {
                    let belt_vector = direction_vector(*belt_direction);
                    (neighbour.0 + belt_vector.0, neighbour.1 + belt_vector.1) == tile
                        && item.as_deref().is_some_and(|item| item != edge.item)
                })
        })
    };

    let mut found = None;
    while let Some(Reverse((_, cost, tile, incoming))) = open.pop() {
        let state = (tile, incoming);
        if best.get(&state).copied().unwrap_or(i32::MAX) < cost {
            continue;
        }
        if tile == edge.end.stub && incoming == target_direction && tile != edge.start.stub {
            found = Some(state);
            break;
        }
        let current_bits = occupancy.get(&tile).copied().unwrap_or(0);
        let previous = parent.get(&state).map(|state| state.0);
        let previous_bits = previous
            .and_then(|tile| occupancy.get(&tile).copied())
            .unwrap_or(0);
        for next_direction in [
            EntityDirection::North,
            EntityDirection::East,
            EntityDirection::South,
            EntityDirection::West,
        ] {
            let next_index = direction_index(next_direction);
            // The source stub is itself a belt tile: its first step must
            // continue the selected local output direction.
            if tile == edge.start.stub && !parent.contains_key(&state) && next_index != incoming {
                continue;
            }
            // Crossed surface belts are represented by a straight UG pair.
            // Neither an occupied crossing tile nor its first free exit may
            // turn.
            if (current_bits != 0 || previous_bits != 0) && next_index != incoming {
                continue;
            }
            if direction_index(rotate_direction(
                index_direction(incoming),
                QuarterTurn::Clockwise180,
            )) == next_index
            {
                continue;
            }
            let vector = direction_vector(next_direction);
            let next = (tile.0 + vector.0, tile.1 + vector.1);
            let next_bits = occupancy.get(&next).copied().unwrap_or(0);
            // A free tile immediately after a crossing becomes the
            // underground output.  It cannot simultaneously be the input
            // to another crossing; require one ordinary surface belt between
            // consecutive underground pairs.
            if previous_bits != 0 && next_bits != 0 {
                continue;
            }
            if !passable(next, next_direction) {
                continue;
            }
            if occupancy.get(&next).copied().unwrap_or(0) == 0 && fed_by_foreign(next) {
                continue;
            }
            let next_state = (next, next_index);
            let turn_cost = i32::from(next_index != incoming);
            let next_cost = cost + 1 + turn_cost;
            if best.get(&next_state).copied().unwrap_or(i32::MAX) <= next_cost {
                continue;
            }
            best.insert(next_state, next_cost);
            parent.insert(next_state, state);
            open.push(Reverse((
                next_cost + heuristic(next),
                next_cost,
                next,
                next_index,
            )));
        }
    }
    let Some(mut state) = found else {
        return Err(format!(
            "net {}: no route from {:?} {:?} to {:?} {:?}",
            edge.item, edge.start.stub, edge.start.direction, edge.end.stub, edge.end.direction
        ));
    };
    let mut path = vec![state.0];
    while let Some(&previous) = parent.get(&state) {
        path.push(previous.0);
        state = previous;
    }
    path.reverse();
    Ok(path)
}

fn materialize_edge_path(
    edge: &RouteEdge,
    path: &[(i32, i32)],
    occupancy: &FxHashMap<(i32, i32), u8>,
) -> Result<Vec<PlacedEntity>, String> {
    if path.len() < 2 {
        return Err(format!("net {}: route has fewer than two tiles", edge.item));
    }
    let belt = "express-transport-belt";
    let underground = "express-underground-belt";
    let reach = crate::common::ug_max_reach(belt) as usize;
    let occupied: Vec<bool> = path
        .iter()
        .map(|tile| occupancy.get(tile).copied().unwrap_or(0) != 0)
        .collect();
    if occupied[0] || *occupied.last().unwrap() {
        return Err(format!("net {}: a route stub is occupied", edge.item));
    }
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut index = 0usize;
    while index < path.len() {
        if occupied[index] {
            let run_start = index;
            let mut after = index;
            while after < path.len() && occupied[after] {
                after += 1;
            }
            if run_start == 0 || after == path.len() {
                return Err(format!(
                    "net {}: crossing touches a route endpoint",
                    edge.item
                ));
            }
            if after - run_start + 1 > reach {
                return Err(format!(
                    "net {}: crossing span {} exceeds express underground reach",
                    edge.item,
                    after - run_start + 1
                ));
            }
            let entrance_tile = path[run_start - 1];
            let exit_tile = path[after];
            let vector = (
                (exit_tile.0 - entrance_tile.0).signum(),
                (exit_tile.1 - entrance_tile.1).signum(),
            );
            let direction = vector_direction(vector)?;
            let Some(entrance) = entities
                .iter_mut()
                .rev()
                .find(|entity| (entity.x, entity.y) == entrance_tile && entity.name == belt)
            else {
                return Err(format!(
                    "net {}: crossing entrance {:?} was not stamped",
                    edge.item, entrance_tile
                ));
            };
            entrance.name = underground.to_string();
            entrance.direction = direction;
            entrance.io_type = Some("input".to_string());
            entities.push(PlacedEntity {
                name: underground.to_string(),
                x: exit_tile.0,
                y: exit_tile.1,
                direction,
                io_type: Some("output".to_string()),
                carries: Some(edge.item.clone()),
                rate: Some(edge.rate),
                ..Default::default()
            });
            index = after + 1;
            continue;
        }
        let direction = if let Some(&next) = path.get(index + 1) {
            vector_direction((
                (next.0 - path[index].0).signum(),
                (next.1 - path[index].1).signum(),
            ))?
        } else {
            edge.end.direction
        };
        entities.push(PlacedEntity {
            name: belt.to_string(),
            x: path[index].0,
            y: path[index].1,
            direction,
            carries: Some(edge.item.clone()),
            rate: Some(edge.rate),
            ..Default::default()
        });
        index += 1;
    }
    Ok(entities)
}

fn route_work_once(mut work: RouteWork) -> Result<RouteWork, String> {
    let mut occupancy = build_occupancy(&work.entities);
    let mut belt_dirs: FxHashMap<(i32, i32), (EntityDirection, Option<String>)> =
        FxHashMap::default();
    for entity in work.entities.iter().filter(|entity| {
        entity.name.ends_with("transport-belt") && !entity.name.ends_with("underground-belt")
    }) {
        belt_dirs.insert(
            (entity.x, entity.y),
            (entity.direction, entity.carries.clone()),
        );
    }
    let mut points: Vec<_> = work
        .edges
        .iter()
        .flat_map(|edge| [edge.start.stub, edge.end.stub])
        .collect();
    points.extend(work.entities.iter().flat_map(|entity| {
        let dims = entity_dims(entity);
        [(entity.x, entity.y), (entity.x + dims.0, entity.y + dims.1)]
    }));
    let bounds = (
        (
            points.iter().map(|point| point.0).min().unwrap_or(0) - 12,
            points.iter().map(|point| point.1).min().unwrap_or(0) - 12,
        ),
        (
            points.iter().map(|point| point.0).max().unwrap_or(0) + 12,
            points.iter().map(|point| point.1).max().unwrap_or(0) + 12,
        ),
    );
    for edge in &work.edges {
        let path = route_edge_path(edge, &occupancy, &belt_dirs, &work.reserved, bounds)?;
        let routed = materialize_edge_path(edge, &path, &occupancy)?;
        let turns: FxHashSet<_> = path
            .windows(3)
            .filter_map(|window| {
                let incoming = (window[1].0 - window[0].0, window[1].1 - window[0].1);
                let outgoing = (window[2].0 - window[1].0, window[2].1 - window[1].1);
                (incoming != outgoing).then_some(window[1])
            })
            .collect();
        for entity in &routed {
            let bits = if entity.name.ends_with("underground-belt")
                || turns.contains(&(entity.x, entity.y))
            {
                OCC_BLOCK
            } else {
                transport_axis(entity.direction)
            };
            occupancy.insert((entity.x, entity.y), bits);
            if !entity.name.ends_with("underground-belt") {
                belt_dirs.insert(
                    (entity.x, entity.y),
                    (entity.direction, entity.carries.clone()),
                );
            }
        }
        work.entities.extend(routed);
    }
    Ok(work)
}

fn route_work_with_priority(
    work: RouteWork,
    mut priority: Vec<String>,
) -> Result<RouteWork, String> {
    let mut last_error = String::new();
    for _ in 0..=work.edges.len() {
        let mut attempt = work.clone();
        attempt.edges.sort_by_key(|edge| {
            priority
                .iter()
                .position(|item| item == &edge.item)
                .unwrap_or(priority.len())
        });
        match route_work_once(attempt) {
            Ok(routed) => return Ok(routed),
            Err(error) => {
                let failed = work
                    .edges
                    .iter()
                    .map(|edge| edge.item.clone())
                    .find(|item| error.contains(&format!("net {item}:")));
                last_error = error;
                match failed {
                    Some(item) if !priority.contains(&item) => priority.insert(0, item),
                    _ => break,
                }
            }
        }
    }
    Err(last_error)
}

fn add_power(entities: &mut Vec<PlacedEntity>) -> Result<(), String> {
    let occupied = occupied_tiles(entities);
    let machines: Vec<_> = entities
        .iter()
        .filter(|entity| entity.recipe.is_some())
        .map(|entity| {
            let (width, height) = entity_dims(entity);
            (entity.x + width / 2, entity.y, height)
        })
        .collect();
    let inserters: Vec<_> = entities
        .iter()
        .filter(|entity| is_inserter(&entity.name))
        .map(|entity| (entity.x, entity.y))
        .collect();
    let (poles, uncovered) = crate::bus::layout::place_poles(
        &machines,
        &inserters,
        &occupied,
        &[],
        crate::common::QualityTier::Normal,
    );
    if !uncovered.is_empty() {
        return Err(format!(
            "power placement left {} rotated-row subjects uncovered; sample {:?}",
            uncovered.len(),
            uncovered.iter().take(6).collect::<Vec<_>>()
        ));
    }
    entities.extend(poles);
    Ok(())
}

fn finish_layout(mut work: RouteWork, native: &LayoutResult) -> Result<LayoutResult, String> {
    add_power(&mut work.entities)?;
    let min_x = work
        .entities
        .iter()
        .map(|entity| entity.x)
        .min()
        .unwrap_or(0);
    let min_y = work
        .entities
        .iter()
        .map(|entity| entity.y)
        .min()
        .unwrap_or(0);
    for entity in &mut work.entities {
        entity.x -= min_x;
        entity.y -= min_y;
    }
    for record in work
        .boundary_inputs
        .iter_mut()
        .chain(&mut work.boundary_outputs)
    {
        record.x -= min_x;
        record.y -= min_y;
    }
    let width = work
        .entities
        .iter()
        .map(|entity| entity.x + entity_dims(entity).0)
        .max()
        .unwrap_or(0);
    let height = work
        .entities
        .iter()
        .map(|entity| entity.y + entity_dims(entity).1)
        .max()
        .unwrap_or(0);
    let mut layout = LayoutResult {
        entities: work.entities,
        width,
        height,
        boundary_inputs: work.boundary_inputs,
        boundary_outputs: work.boundary_outputs,
        // Rotation changes geometry, not the caller's declared planning
        // context.  Preserve the source pass metadata and rebuild the
        // stored wire graph under that same mode below.
        wire_mode: native.wire_mode,
        stacking: native.stacking,
        inserter_capacity: native.inserter_capacity,
        ..Default::default()
    };
    layout.power_wires = Some(crate::power_wires::compute_pole_wires(
        &layout.entities,
        layout.wire_mode,
    ));
    Ok(layout)
}

fn non_pole_bbox(layout: &LayoutResult) -> (i32, i32) {
    let mut bounds = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for entity in &layout.entities {
        if entity.name.contains("electric-pole") || entity.name == "substation" {
            continue;
        }
        let (width, height) = entity_dims(entity);
        bounds.0 = bounds.0.min(entity.x);
        bounds.1 = bounds.1.min(entity.y);
        bounds.2 = bounds.2.max(entity.x + width);
        bounds.3 = bounds.3.max(entity.y + height);
    }
    if bounds.0 == i32::MAX {
        (0, 0)
    } else {
        (bounds.2 - bounds.0, bounds.3 - bounds.1)
    }
}

fn aspect((width, height): (i32, i32)) -> f64 {
    width.max(height) as f64 / width.min(height).max(1) as f64
}

fn aspect_score(native: f64, candidate: f64) -> f64 {
    if (native - 1.0).abs() <= f64::EPSILON {
        f64::from((candidate - 1.0).abs() <= f64::EPSILON)
    } else {
        1.0 - (candidate - 1.0) / (native - 1.0)
    }
}

fn plan_rotation_mask(plan: &RotationPlan) -> u64 {
    plan.rotations
        .iter()
        .enumerate()
        .fold(0u64, |mask, (index, rotation)| {
            if rotation.swaps_axes() {
                mask | (1u64 << index)
            } else {
                mask
            }
        })
}

/// Materialize one stable member of the rotation-aware search.
///
/// The explicit form is the regression seam: it avoids paying the full search
/// cost in the ordinary test suite while still exercising real routing,
/// validation, and directed transit measurement.
pub(crate) fn build_rotation_aware_layout_selected(
    rows: &[RowSpan],
    native: &LayoutResult,
    solver_result: &SolverResult,
    selection: &RotationSelection,
) -> Result<LayoutResult, String> {
    use crate::bus::transit::measure_realized_transit;
    use crate::validate::{self, LayoutStyle, Severity};

    if !solver_result
        .external_outputs
        .iter()
        .any(|flow| flow.item == "logistic-science-pack")
    {
        return Err(
            "rotation-refusal: first slice is frozen to the solid Science-2 fixture".into(),
        );
    }
    check_rotation_row_budget(rows)?;
    let macros = extract_row_macros(rows, &native.entities)?;
    let plans = enumerate_rotation_plans(&macros, rows, solver_result);
    let plan = plans
        .iter()
        .find(|plan| {
            plan_rotation_mask(plan) == selection.rotation_mask
                && plan.gap == selection.gap
                && plan.target_width == selection.target_width
                && plan.order == selection.order
        })
        .ok_or_else(|| {
            format!(
                "rotation-refusal: selection mask={:#x} gap={} target={} order={:?} is not in this layout's search",
                selection.rotation_mask, selection.gap, selection.target_width, selection.order
            )
        })?;
    let placed = place_macros(&macros, rows, solver_result, plan)?;
    if let Some(priority) = selection.route_priority.as_deref() {
        if !placed.nets.iter().any(|net| net.item == priority) {
            return Err(format!(
                "rotation-refusal: route priority {priority} is not a Science-2 net"
            ));
        }
    }
    let work = assemble_route_work(placed)?;
    let priority = selection.route_priority.iter().cloned().collect();
    let work = route_work_with_priority(work, priority)?;
    let mut layout = finish_layout(work, native)?;
    let issues = match validate::validate(&layout, Some(solver_result), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(error) => error.issues,
    };
    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Warning)
        .count();
    if let Some(first) = issues.first() {
        return Err(format!(
            "rotation-refusal: selected artifact has {} validation issues ({error_count} Errors, \
             {warning_count} Warnings); first {} at ({:?}, {:?}): {}",
            issues.len(),
            first.category,
            first.x,
            first.y,
            first.message
        ));
    }
    let native_transit = measure_realized_transit(native, solver_result, 0.5)
        .map_err(|error| format!("rotation-refusal: native transit is not measurable: {error}"))?
        .total;
    let transit = measure_realized_transit(&layout, solver_result, 0.5)
        .map_err(|error| format!("rotation-refusal: selected transit is not measurable: {error}"))?
        .total;
    let native_aspect = aspect(non_pole_bbox(native));
    let bbox = non_pole_bbox(&layout);
    let ar_score = aspect_score(native_aspect, aspect(bbox));
    if ar_score < 0.5 {
        return Err(format!(
            "rotation-refusal: selected artifact misses the frozen shape bar: AR_score {ar_score:+.3} < +0.500"
        ));
    }
    let transit_score = 1.0 - transit / native_transit.max(f64::EPSILON);
    let composite = 0.5 * ar_score + 0.5 * transit_score;
    let rotated: Vec<_> = macros
        .iter()
        .zip(&plan.rotations)
        .filter(|(_, rotation)| rotation.swaps_axes())
        .map(|(row, _)| row.recipe.as_str())
        .collect();
    layout.warnings.push(format!(
        "RFC-064 rotation-aware row-macro spike: bbox={}x{}, transit={:.2}, \
         AR_score={:+.3}, Transit_score={:+.3}, composite={:+.3}; gap={}, target={}, \
         order={:?}; rotated={:?}; route_priority={:?}",
        bbox.0,
        bbox.1,
        transit,
        ar_score,
        transit_score,
        composite,
        plan.gap,
        plan.target_width,
        plan.order,
        rotated,
        selection.route_priority,
    ));
    Ok(layout)
}

/// Materialize the bounded Science-2 rotation-aware row-macro experiment.
///
/// This never participates in decomposition selection and has no option or
/// default-path call site.  Every returned artifact has zero validator issues
/// and a measurable directed realized-transit graph.  Refusals remain `Err`.
pub(crate) fn build_rotation_aware_layout(
    rows: &[RowSpan],
    native: &LayoutResult,
    solver_result: &SolverResult,
) -> Result<LayoutResult, String> {
    use crate::bus::transit::measure_realized_transit;
    use crate::validate::{self, LayoutStyle, Severity};

    if !solver_result
        .external_outputs
        .iter()
        .any(|flow| flow.item == "logistic-science-pack")
    {
        return Err(
            "rotation-refusal: first slice is frozen to the solid Science-2 fixture".into(),
        );
    }
    check_rotation_row_budget(rows)?;
    let macros = extract_row_macros(rows, &native.entities)?;
    let plans = enumerate_rotation_plans(&macros, rows, solver_result);
    if plans.is_empty() {
        return Err("rotation-refusal: orientation search produced no structural plans".into());
    }
    let native_bbox = non_pole_bbox(native);
    let native_aspect = aspect(native_bbox);
    let native_transit = measure_realized_transit(native, solver_result, 0.5)
        .map_err(|error| format!("rotation-refusal: native transit is not measurable: {error}"))?
        .total;

    let mut best: Option<(f64, f64, LayoutResult, &RotationPlan, Option<String>)> = None;
    let mut routed = 0usize;
    let mut validation_rejected = 0usize;
    let mut transit_rejected = 0usize;
    let mut last_error = String::new();
    let eligible = |plan: &&RotationPlan| {
        let structural_aspect =
            plan.width.max(plan.height) as f64 / plan.width.min(plan.height).max(1) as f64;
        macros
            .iter()
            .zip(&plan.rotations)
            .filter(|(row, _)| row.width >= 40)
            .all(|(_, rotation)| rotation.swaps_axes())
            && structural_aspect <= 1.15
    };
    let mut transit_frontier: Vec<_> = plans.iter().filter(eligible).collect();
    transit_frontier.sort_by(|a, b| {
        let aspect = |plan: &RotationPlan| {
            plan.width.max(plan.height) as f64 / plan.width.min(plan.height).max(1) as f64
        };
        let rotations = |plan: &RotationPlan| {
            plan.rotations
                .iter()
                .filter(|rotation| rotation.swaps_axes())
                .count()
        };
        a.estimated_transit
            .total_cmp(&b.estimated_transit)
            .then_with(|| aspect(a).total_cmp(&aspect(b)))
            .then_with(|| rotations(a).cmp(&rotations(b)))
            .then_with(|| (a.width * a.height).cmp(&(b.width * b.height)))
    });
    let mut candidate_plans: Vec<_> = transit_frontier.into_iter().take(64).collect();
    for plan in plans.iter().filter(eligible).take(64) {
        if !candidate_plans.iter().any(|candidate| {
            candidate.rotations == plan.rotations
                && candidate.origins == plan.origins
                && candidate.gap == plan.gap
        }) {
            candidate_plans.push(plan);
        }
    }
    let structural_candidate_count = candidate_plans.len();
    let mut route_order_count = 0usize;
    for (attempt_index, plan) in candidate_plans.into_iter().enumerate() {
        if let Ok(only) = std::env::var("SPAGHETTIO_ROTATION_ATTEMPT") {
            if only.parse::<usize>().ok() != Some(attempt_index) {
                continue;
            }
        }
        if std::env::var("SPAGHETTIO_ROTATION_DEBUG").is_ok() {
            eprintln!(
                "rotation candidate {attempt_index}: {}x{} gap={} target={} {:?} est={:.1}",
                plan.width,
                plan.height,
                plan.gap,
                plan.target_width,
                plan.order,
                plan.estimated_transit,
            );
        }
        let placed = match place_macros(&macros, rows, solver_result, plan) {
            Ok(placed) => placed,
            Err(error) => {
                last_error = error;
                continue;
            }
        };
        // Search the default route order AND every one-item promotion for
        // every structural plan.  A default refusal or warning is evidence
        // about that order only; it must not suppress the other deterministic
        // orderings of the same placement.
        let mut priority_items: Vec<_> = placed.nets.iter().map(|net| net.item.clone()).collect();
        priority_items.sort();
        priority_items.dedup();
        let route_orders: Vec<Option<String>> = std::iter::once(None)
            .chain(priority_items.into_iter().map(Some))
            .collect();
        for priority in route_orders {
            route_order_count += 1;
            let attempt = (|| {
                let work = assemble_route_work(placed.clone())?;
                let priority_vec = priority.iter().cloned().collect();
                let work = route_work_with_priority(work, priority_vec)?;
                finish_layout(work, native)
            })();
            let candidate = match attempt {
                Ok(candidate) => {
                    routed += 1;
                    candidate
                }
                Err(error) => {
                    if std::env::var("SPAGHETTIO_ROTATION_DEBUG").is_ok() {
                        eprintln!("  route/materialization refusal priority={priority:?}: {error}");
                    }
                    last_error = error.to_string();
                    continue;
                }
            };
            let issues = match validate::validate(&candidate, Some(solver_result), LayoutStyle::Bus)
            {
                Ok(issues) => issues,
                Err(error) => error.issues,
            };
            let error_count = issues
                .iter()
                .filter(|issue| issue.severity == Severity::Error)
                .count();
            let warning_count = issues
                .iter()
                .filter(|issue| issue.severity == Severity::Warning)
                .count();
            if !issues.is_empty() {
                validation_rejected += 1;
                if std::env::var("SPAGHETTIO_ROTATION_DEBUG").is_ok() {
                    eprintln!(
                        "  validation rejected priority={priority:?}: {} issues ({error_count} Errors, {warning_count} Warnings)",
                        issues.len()
                    );
                }
                let first = &issues[0];
                last_error = format!(
                    "{} validation issues ({error_count} Errors, {warning_count} Warnings); first: \
                     {:?} {} at ({:?}, {:?}): {}",
                    issues.len(),
                    first.severity,
                    first.category,
                    first.x,
                    first.y,
                    first.message
                );
                continue;
            }
            let transit = match measure_realized_transit(&candidate, solver_result, 0.5) {
                Ok(transit) => transit.total,
                Err(error) => {
                    transit_rejected += 1;
                    if std::env::var("SPAGHETTIO_ROTATION_DEBUG").is_ok() {
                        eprintln!("  transit rejected priority={priority:?}: {error}");
                    }
                    last_error = error.to_string();
                    continue;
                }
            };
            let candidate_aspect = aspect(non_pole_bbox(&candidate));
            let ar_score = aspect_score(native_aspect, candidate_aspect);
            let transit_score = 1.0 - transit / native_transit.max(f64::EPSILON);
            let composite = 0.5 * ar_score + 0.5 * transit_score;
            if std::env::var("SPAGHETTIO_ROTATION_DEBUG").is_ok() {
                eprintln!(
                    "  admissible priority={priority:?}: bbox={:?} transit={transit:.2} AR={ar_score:+.3} TX={transit_score:+.3}",
                    non_pole_bbox(&candidate)
                );
            }
            if ar_score < 0.5 {
                last_error = format!(
                    "candidate routed cleanly but missed AR_score: {ar_score:+.3} < +0.500"
                );
                continue;
            }
            if best.as_ref().is_none_or(|current| {
                composite > current.0 + 1e-12
                    || ((composite - current.0).abs() <= 1e-12 && transit < current.1)
            }) {
                best = Some((composite, transit, candidate, plan, priority));
            }
        }
    }
    let Some((composite, transit, mut layout, plan, selected_priority)) = best else {
        return Err(format!(
            "rotation-refusal: no shape-clearing, zero-issue, transit-measurable candidate among {structural_candidate_count} \
             structural plans / {route_order_count} route orders ({routed} routed, {validation_rejected} validation-rejected, \
             {transit_rejected} transit-rejected); last failure: {last_error}"
        ));
    };
    let bbox = non_pole_bbox(&layout);
    let rotated: Vec<_> = macros
        .iter()
        .zip(&plan.rotations)
        .filter(|(_, rotation)| rotation.swaps_axes())
        .map(|(row, _)| row.recipe.as_str())
        .collect();
    layout.warnings.push(format!(
        "RFC-064 rotation-aware row-macro spike: bbox={}x{}, transit={:.2}, \
         AR_score={:+.3}, Transit_score={:+.3}, composite={:+.3}; gap={}, target={}, \
         order={:?}; rotated={:?}; route_priority={:?}",
        bbox.0,
        bbox.1,
        transit,
        aspect_score(native_aspect, aspect(bbox)),
        1.0 - transit / native_transit.max(f64::EPSILON),
        composite,
        plan.gap,
        plan.target_width,
        plan.order,
        rotated,
        selected_priority,
    ));
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(name: &str, x: i32, y: i32, direction: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            direction,
            ..Default::default()
        }
    }

    #[test]
    fn quarter_turn_rotates_fixed_geometry_and_direction() {
        let machine = entity("assembling-machine-2", 1, 2, EntityDirection::North);
        let rotated = transform_entity(&machine, 8, 6, QuarterTurn::Clockwise90);
        assert_eq!((rotated.x, rotated.y), (1, 1));
        assert_eq!(rotated.direction, EntityDirection::East);
        assert_eq!(
            transform_tile((0, 4), 8, 6, QuarterTurn::Clockwise90),
            (1, 0)
        );
    }

    #[test]
    fn shelf_places_vertical_rows_without_changing_their_identity() {
        let rows = [
            RowMacro {
                recipe: "wide".into(),
                width: 12,
                height: 4,
                entities: Vec::new(),
                belt_runs: Vec::new(),
            },
            RowMacro {
                recipe: "small".into(),
                width: 5,
                height: 4,
                entities: Vec::new(),
                belt_runs: Vec::new(),
            },
        ];
        let oriented = vec![
            orient_macro(&rows[0], QuarterTurn::Clockwise90),
            orient_macro(&rows[1], QuarterTurn::Zero),
        ];
        let (origins, width, height) = shelf_place(&oriented, 10, 2, RotationOrder::Source);
        assert_eq!(origins, vec![(0, 0), (0, 14)]);
        assert_eq!((width, height), (5, 18));
        assert_eq!((oriented[0].width, oriented[0].height), (4, 12));
    }

    #[test]
    fn connection_target_selects_belt_end_and_flow_direction() {
        let run = OrientedRun {
            item: "iron-plate".into(),
            role: RunRole::Input,
            tiles: (0..5).map(|x| (x, 0)).collect(),
            endpoint_a: (0, 0),
            endpoint_b: (4, 0),
        };
        let (from_left, _) = choose_run_port(&run, (10, 10), (0, 10), 0).unwrap();
        let (from_right, _) = choose_run_port(&run, (10, 10), (30, 10), 0).unwrap();
        assert_eq!(from_left.stub, (9, 10));
        assert_eq!(from_left.direction, EntityDirection::East);
        assert_eq!(from_right.stub, (15, 10));
        assert_eq!(from_right.direction, EntityDirection::West);

        let output = OrientedRun {
            role: RunRole::Output,
            ..run
        };
        let (exit_left, _) = choose_run_port(&output, (10, 10), (0, 10), 0).unwrap();
        assert_eq!(exit_left.stub, (9, 10));
        assert_eq!(exit_left.direction, EntityDirection::West);
    }

    #[test]
    fn finish_layout_preserves_native_planning_metadata_and_rewires() {
        let native = LayoutResult {
            wire_mode: crate::power_wires::WireMode::Tree,
            stacking: 3,
            inserter_capacity: 7,
            ..Default::default()
        };
        let work = RouteWork {
            entities: vec![
                entity("medium-electric-pole", 0, 0, EntityDirection::North),
                entity("medium-electric-pole", 7, 0, EntityDirection::North),
            ],
            edges: Vec::new(),
            boundary_inputs: Vec::new(),
            boundary_outputs: Vec::new(),
            reserved: FxHashSet::default(),
        };

        let layout = finish_layout(work, &native).expect("metadata fixture should finish");
        assert_eq!(layout.wire_mode, native.wire_mode);
        assert_eq!(layout.stacking, native.stacking);
        assert_eq!(layout.inserter_capacity, native.inserter_capacity);
        assert_eq!(
            layout.power_wires,
            Some(crate::power_wires::compute_pole_wires(
                &layout.entities,
                crate::power_wires::WireMode::Tree,
            ))
        );
    }
}
