//! RFC-057 topology-preserving dense repacking foundation.
//!
//! This module freezes the logical production graph and the placed machine
//! multiset before any geometric search.  The first placement primitive is an
//! exact per-axis constraint-graph compactor: for a fixed relative order it
//! computes the minimum legal coordinates by longest paths in a DAG.

use std::collections::BTreeMap;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{
    dir_to_vec, entity_size, inserter_reach, is_belt_entity, is_inserter, is_surface_belt,
    is_ug_belt, oriented_splitter_dims, ug_max_reach, ug_to_surface_tier, QualityTier,
};
use crate::models::{EntityDirection, LayoutResult, ModuleItem, PlacedEntity, SolverResult};
use crate::verdict::CorrespondenceMap;

pub const RATE_SCALE: f64 = 1_000_000_000.0;

fn fixed_rate(rate: f64) -> i64 {
    (rate * RATE_SCALE).round() as i64
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductionMachine {
    pub recipe: String,
    pub entity: String,
    pub count: i64,
    pub modules: Vec<(String, u32, Option<QualityTier>)>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductionEdge {
    /// Canonical set of recipes capable of supplying this item in the solved
    /// graph. Oil co-products and cracking legitimately create more than one.
    pub producer_recipes: Vec<String>,
    pub item: String,
    pub consumer_recipe: String,
    pub rate: i64,
    pub is_fluid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductionBoundary {
    pub item: String,
    pub rate: i64,
    pub is_fluid: bool,
}

/// Canonical logical topology. Rates and fractional counts use fixed-point
/// nanounits so equality and hashing do not depend on `f64` ordering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionSignature {
    pub machines: Vec<ProductionMachine>,
    pub edges: Vec<ProductionEdge>,
    pub external_inputs: Vec<ProductionBoundary>,
    pub target_outputs: Vec<ProductionBoundary>,
    pub surplus_outputs: Vec<ProductionBoundary>,
}

impl ProductionSignature {
    pub fn from_solver(sr: &SolverResult) -> Result<Self, String> {
        let mut producers_of: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for machine in &sr.machines {
            for output in &machine.outputs {
                let producers = producers_of.entry(output.item.as_str()).or_default();
                if !producers.contains(&machine.recipe) {
                    producers.push(machine.recipe.clone());
                }
            }
        }
        for producers in producers_of.values_mut() {
            producers.sort();
        }

        let mut machines: Vec<_> = sr
            .machines
            .iter()
            .map(|machine| ProductionMachine {
                recipe: machine.recipe.clone(),
                entity: machine.entity.clone(),
                count: fixed_rate(machine.count),
                modules: canonical_modules(&machine.game_modules),
            })
            .collect();
        machines.sort();

        let mut edges = Vec::new();
        for consumer in &sr.machines {
            for input in &consumer.inputs {
                let Some(producer_recipes) = producers_of.get(input.item.as_str()) else {
                    continue;
                };
                if producer_recipes.len() == 1 && producer_recipes[0] == consumer.recipe {
                    continue;
                }
                edges.push(ProductionEdge {
                    producer_recipes: producer_recipes.clone(),
                    item: input.item.clone(),
                    consumer_recipe: consumer.recipe.clone(),
                    rate: fixed_rate(input.rate * consumer.count),
                    is_fluid: input.is_fluid,
                });
            }
        }
        edges.sort();

        Ok(Self {
            machines,
            edges,
            external_inputs: canonical_boundaries(&sr.external_inputs),
            target_outputs: canonical_boundaries(&sr.external_outputs),
            surplus_outputs: canonical_boundaries(&sr.surplus_outputs),
        })
    }
}

fn canonical_modules(modules: &[ModuleItem]) -> Vec<(String, u32, Option<QualityTier>)> {
    let mut result: Vec<_> = modules
        .iter()
        .map(|module| (module.item.clone(), module.count, module.quality))
        .collect();
    result.sort_by(|a, b| {
        (&a.0, a.1, a.2.map(|q| q.level())).cmp(&(&b.0, b.1, b.2.map(|q| q.level())))
    });
    result
}

fn canonical_boundaries(flows: &[crate::models::ItemFlow]) -> Vec<ProductionBoundary> {
    let mut result: Vec<_> = flows
        .iter()
        .map(|flow| ProductionBoundary {
            item: flow.item.clone(),
            rate: fixed_rate(flow.rate),
            is_fluid: flow.is_fluid,
        })
        .collect();
    result.sort();
    result
}

/// Exact placed-machine multiset. Unlike the logical signature this records
/// integer entities after capacity quantization, so a candidate can prove it
/// only shuffled machines rather than silently adding/removing one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedMachineSignature(pub Vec<PlacedMachine>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacedMachine {
    pub recipe: String,
    pub entity: String,
    pub quality: Option<QualityTier>,
    pub modules: Vec<(String, u32, Option<QualityTier>)>,
}

impl PlacedMachineSignature {
    pub fn from_layout(layout: &LayoutResult) -> Self {
        let mut machines: Vec<_> = layout
            .entities
            .iter()
            .filter_map(|entity| {
                Some(PlacedMachine {
                    recipe: entity.recipe.clone()?,
                    entity: entity.name.clone(),
                    quality: entity.quality,
                    modules: canonical_modules(&entity.items),
                })
            })
            .collect();
        machines.sort();
        Self(machines)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactAxis {
    X,
    Y,
}

/// One movable rectangle in the coarse compaction model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactBlock {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CompactBlock {
    fn axis_start(&self, axis: CompactAxis) -> i32 {
        match axis {
            CompactAxis::X => self.x,
            CompactAxis::Y => self.y,
        }
    }

    fn axis_size(&self, axis: CompactAxis) -> i32 {
        match axis {
            CompactAxis::X => self.width,
            CompactAxis::Y => self.height,
        }
    }

    fn overlaps_cross_axis(&self, other: &Self, axis: CompactAxis) -> bool {
        let (a0, a1, b0, b1) = match axis {
            CompactAxis::X => (
                self.y,
                self.y + self.height,
                other.y,
                other.y + other.height,
            ),
            CompactAxis::Y => (self.x, self.x + self.width, other.x, other.x + other.width),
        };
        a0 < b1 && b0 < a1
    }
}

/// Extract individual placed machines as the first coarse movable-block set.
/// Inserters and routes enter the later `CompactIR` slices; this function is a
/// metrics/constraint baseline and does not itself mutate a layout.
pub fn machine_blocks(layout: &LayoutResult) -> Vec<CompactBlock> {
    layout
        .entities
        .iter()
        .enumerate()
        .filter_map(|(id, entity)| {
            entity.recipe.as_ref()?;
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
            Some(CompactBlock {
                id,
                x: entity.x,
                y: entity.y,
                width: width as i32,
                height: height as i32,
            })
        })
        .collect()
}

/// Compact one axis while preserving the source order of every pair whose
/// cross-axis footprints overlap. This is longest-path placement on the
/// induced separation DAG.
pub fn compact_axis(
    blocks: &[CompactBlock],
    axis: CompactAxis,
    clearance: i32,
) -> Vec<CompactBlock> {
    let mut order: Vec<usize> = (0..blocks.len()).collect();
    order.sort_by_key(|&idx| (blocks[idx].axis_start(axis), blocks[idx].id));

    let mut coordinate = vec![0i32; blocks.len()];
    for (position, &idx) in order.iter().enumerate() {
        let mut lower_bound = 0;
        for &previous in &order[..position] {
            if blocks[previous].overlaps_cross_axis(&blocks[idx], axis) {
                lower_bound = lower_bound.max(
                    coordinate[previous] + blocks[previous].axis_size(axis) + clearance.max(0),
                );
            }
        }
        coordinate[idx] = lower_bound;
    }

    blocks
        .iter()
        .enumerate()
        .map(|(idx, block)| {
            let mut compacted = block.clone();
            match axis {
                CompactAxis::X => compacted.x = coordinate[idx],
                CompactAxis::Y => compacted.y = coordinate[idx],
            }
            compacted
        })
        .collect()
}

pub fn occupied_bbox(blocks: &[CompactBlock]) -> (i32, i32) {
    (
        blocks.iter().map(|b| b.x + b.width).max().unwrap_or(0)
            - blocks.iter().map(|b| b.x).min().unwrap_or(0),
        blocks.iter().map(|b| b.y + b.height).max().unwrap_or(0)
            - blocks.iter().map(|b| b.y).min().unwrap_or(0),
    )
}

pub fn blocks_overlap(a: &CompactBlock, b: &CompactBlock) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

/// Remove globally empty tile columns while preserving entity order and
/// shortening horizontal underground spans. This is the first runnable
/// constraint-compaction baseline: it never deletes or rotates an entity.
///
/// The result deliberately drops region/trace metadata because their
/// coordinate-rich solver provenance describes the source embedding, not the
/// compacted artifact. Functional boundary/effective-row records are remapped.
pub fn strip_empty_columns(layout: &LayoutResult) -> LayoutResult {
    if layout.width <= 0 {
        return layout.clone();
    }
    let mut occupied = vec![false; layout.width as usize];
    for entity in &layout.entities {
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
        let _ = height;
        for x in entity.x.max(0)..(entity.x + width as i32).min(layout.width) {
            occupied[x as usize] = true;
        }
    }
    for boundary in layout
        .boundary_inputs
        .iter()
        .chain(layout.boundary_outputs.iter())
    {
        if (0..layout.width).contains(&boundary.x) {
            occupied[boundary.x as usize] = true;
        }
    }
    for (_, x, _) in &layout.surplus_exits {
        if (0..layout.width).contains(x) {
            occupied[*x as usize] = true;
        }
    }

    let mut removed_before = vec![0i32; layout.width as usize + 1];
    for x in 0..layout.width as usize {
        removed_before[x + 1] = removed_before[x] + i32::from(!occupied[x]);
    }
    let remap_x = |x: i32| -> i32 {
        if x <= 0 {
            x
        } else if x >= layout.width {
            x - removed_before[layout.width as usize]
        } else {
            x - removed_before[x as usize]
        }
    };

    let mut compacted = layout.clone();
    for entity in &mut compacted.entities {
        entity.x = remap_x(entity.x);
    }
    for boundary in compacted
        .boundary_inputs
        .iter_mut()
        .chain(compacted.boundary_outputs.iter_mut())
    {
        boundary.x = remap_x(boundary.x);
    }
    for (_, x, _) in &mut compacted.surplus_exits {
        *x = remap_x(*x);
    }
    // Coordinate removal can collapse a valid underground pair to adjacent
    // entities. At distance one the equivalent topology is surface belts.
    normalize_adjacent_undergrounds(&mut compacted);
    compacted.width -= removed_before[layout.width as usize];
    compacted.regions.clear();
    compacted.trace = None;
    compacted
}

/// Row-axis counterpart of [`strip_empty_columns`].
pub fn strip_empty_rows(layout: &LayoutResult) -> LayoutResult {
    if layout.height <= 0 {
        return layout.clone();
    }
    let mut occupied = vec![false; layout.height as usize];
    for entity in &layout.entities {
        let (width, height) = entity_dims(&entity.name, entity.direction);
        let _ = width;
        for y in entity.y.max(0)..(entity.y + height).min(layout.height) {
            occupied[y as usize] = true;
        }
    }
    for boundary in layout
        .boundary_inputs
        .iter()
        .chain(layout.boundary_outputs.iter())
    {
        if (0..layout.height).contains(&boundary.y) {
            occupied[boundary.y as usize] = true;
        }
    }
    for (_, _, y) in &layout.surplus_exits {
        if (0..layout.height).contains(y) {
            occupied[*y as usize] = true;
        }
    }

    let mut removed_before = vec![0i32; layout.height as usize + 1];
    for y in 0..layout.height as usize {
        removed_before[y + 1] = removed_before[y] + i32::from(!occupied[y]);
    }
    let remap_y = |y: i32| -> i32 {
        if y <= 0 {
            y
        } else if y >= layout.height {
            y - removed_before[layout.height as usize]
        } else {
            y - removed_before[y as usize]
        }
    };

    let mut compacted = layout.clone();
    for entity in &mut compacted.entities {
        entity.y = remap_y(entity.y);
    }
    for boundary in compacted
        .boundary_inputs
        .iter_mut()
        .chain(compacted.boundary_outputs.iter_mut())
    {
        boundary.y = remap_y(boundary.y);
    }
    for (_, _, y) in &mut compacted.surplus_exits {
        *y = remap_y(*y);
    }
    normalize_adjacent_undergrounds(&mut compacted);
    compacted.height -= removed_before[layout.height as usize];
    compacted.regions.clear();
    compacted.trace = None;
    compacted
}

fn normalize_adjacent_undergrounds(layout: &mut LayoutResult) {
    let entity_at: BTreeMap<(i32, i32), usize> = layout
        .entities
        .iter()
        .enumerate()
        .map(|(idx, entity)| ((entity.x, entity.y), idx))
        .collect();
    let mut surface_pairs = Vec::new();
    for (idx, entity) in layout.entities.iter().enumerate() {
        if !is_ug_belt(&entity.name) || entity.io_type.as_deref() != Some("input") {
            continue;
        }
        let (dx, dy) = dir_to_vec(entity.direction);
        let Some(&output_idx) = entity_at.get(&(entity.x + dx, entity.y + dy)) else {
            continue;
        };
        let output = &layout.entities[output_idx];
        if output.name == entity.name
            && output.direction == entity.direction
            && output.io_type.as_deref() == Some("output")
        {
            surface_pairs.push((idx, output_idx));
        }
    }
    for (input_idx, output_idx) in surface_pairs {
        for idx in [input_idx, output_idx] {
            let entity = &mut layout.entities[idx];
            entity.name = ug_to_surface_tier(&entity.name).to_string();
            entity.io_type = None;
        }
    }
}

/// Run the safe transport compactor to a small fixed point.
pub fn compact_transport_geometry(layout: &LayoutResult) -> LayoutResult {
    let mut current = layout.clone();
    for _ in 0..3 {
        let next = strip_empty_rows(&strip_empty_columns(&undergroundify_straight_belts(
            &current,
        )));
        if next.width == current.width
            && next.height == current.height
            && next.entities.len() == current.entities.len()
        {
            return next;
        }
        current = next;
    }
    current
}

/// Transactionally delete vertical coordinate cuts while preserving a fully
/// valid factory. This is the exact #456-style post-pass baseline: everything
/// on the right of a cut moves together, equivalent seam belts coalesce, and
/// the move is committed only when full validation remains error-free.
/// Apply a transform only if it validates no worse than its input.
///
/// `compact_transport_geometry` was applied unguarded at the head of all three
/// compaction entry points, and the failure mode is self-reinforcing: if it
/// introduces an error, every subsequent coordinate-cut candidate inherits
/// that error, so every candidate is rejected, no cut commits, and the
/// function returns exactly the broken unvalidated layout it started from.
/// The per-cut validation below can never catch it, because it only ever
/// examines candidates derived FROM it.
///
/// Compaction is an optimisation. Declining to compact is always acceptable;
/// returning a broken factory is not.
fn accept_if_no_worse(
    input: &LayoutResult,
    candidate: LayoutResult,
    solver: &SolverResult,
) -> LayoutResult {
    use crate::validate::{self, LayoutStyle, Severity};
    let errors = |l: &LayoutResult| -> usize {
        match validate::validate(l, Some(solver), LayoutStyle::Bus) {
            Ok(issues) => issues,
            Err(error) => error.issues,
        }
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count()
    };
    if errors(&candidate) > errors(input) {
        return input.clone();
    }
    candidate
}

pub fn compact_validated_columns(
    layout: &LayoutResult,
    solver: &SolverResult,
    max_commits: usize,
) -> LayoutResult {
    use crate::validate::{self, LayoutStyle, Severity};

    let mut current = accept_if_no_worse(layout, compact_transport_geometry(layout), solver);
    let mut commits = 0;
    let mut cut = 1;
    while cut < current.width && commits < max_commits {
        let Some(candidate) = collapse_vertical_cut(&current, cut) else {
            cut += 1;
            continue;
        };
        let issues = match validate::validate(&candidate, Some(solver), LayoutStyle::Bus) {
            Ok(issues) => issues,
            Err(error) => error.issues,
        };
        if issues.iter().any(|issue| issue.severity == Severity::Error) {
            cut += 1;
            continue;
        }
        current = candidate;
        commits += 1;
        // Retry the same coordinate: several redundant columns may be
        // adjacent, and accepting a move changes every later cut.
    }
    current
}

/// Row-axis transactional coordinate compactor.
pub fn compact_validated_rows(
    layout: &LayoutResult,
    solver: &SolverResult,
    max_commits: usize,
) -> LayoutResult {
    use crate::validate::{self, LayoutStyle, Severity};

    let mut current = accept_if_no_worse(layout, compact_transport_geometry(layout), solver);
    let mut commits = 0;
    let mut cut = 1;
    while cut < current.height && commits < max_commits {
        let Some(candidate) = collapse_horizontal_cut(&current, cut) else {
            cut += 1;
            continue;
        };
        let issues = match validate::validate(&candidate, Some(solver), LayoutStyle::Bus) {
            Ok(issues) => issues,
            Err(error) => error.issues,
        };
        if issues.iter().any(|issue| issue.severity == Severity::Error) {
            cut += 1;
            continue;
        }
        current = candidate;
        commits += 1;
    }
    current
}

/// Full safe compaction entry point: transport resynthesis followed by
/// alternating validated X/Y coordinate cuts to a small fixed point.
pub fn compact_validated_geometry(layout: &LayoutResult, solver: &SolverResult) -> LayoutResult {
    let mut current = accept_if_no_worse(layout, compact_transport_geometry(layout), solver);
    for _ in 0..3 {
        let columns = compact_validated_columns(&current, solver, usize::MAX);
        let next = compact_validated_rows(&columns, solver, usize::MAX);
        if next.width == current.width
            && next.height == current.height
            && next.entities.len() == current.entities.len()
        {
            return next;
        }
        current = next;
    }
    current
}

fn collapse_vertical_cut(layout: &LayoutResult, cut: i32) -> Option<LayoutResult> {
    if cut <= 0 || cut >= layout.width {
        return None;
    }
    // A cut may not pass through the interior of a multi-tile footprint.
    if layout.entities.iter().any(|entity| {
        let (width, _) = entity_dims(&entity.name, entity.direction);
        entity.x < cut && entity.x + width > cut
    }) {
        return None;
    }

    let mut candidate = layout.clone();
    for entity in &mut candidate.entities {
        if entity.x >= cut {
            entity.x -= 1;
        }
    }
    for boundary in candidate
        .boundary_inputs
        .iter_mut()
        .chain(candidate.boundary_outputs.iter_mut())
    {
        if boundary.x >= cut {
            boundary.x -= 1;
        }
    }
    for (_, x, _) in &mut candidate.surplus_exits {
        if *x >= cut {
            *x -= 1;
        }
    }

    // Coalesce the one expected seam collision: two consecutive tiles of the
    // same surfaced route. All other footprint collisions refuse the cut.
    let mut anchor_at: BTreeMap<(i32, i32), usize> = BTreeMap::new();
    let mut remove = vec![false; candidate.entities.len()];
    for (idx, entity) in candidate.entities.iter().enumerate() {
        if let Some(&other_idx) = anchor_at.get(&(entity.x, entity.y)) {
            let other = &candidate.entities[other_idx];
            if is_surface_belt(&entity.name)
                && is_surface_belt(&other.name)
                && entity.name == other.name
                && entity.direction == other.direction
                && entity.carries == other.carries
            {
                remove[idx] = true;
                continue;
            }
            return None;
        }
        anchor_at.insert((entity.x, entity.y), idx);
    }
    candidate.entities = candidate
        .entities
        .into_iter()
        .enumerate()
        .filter_map(|(idx, entity)| (!remove[idx]).then_some(entity))
        .collect();

    let mut occupied = BTreeMap::new();
    for (idx, entity) in candidate.entities.iter().enumerate() {
        let (width, height) = entity_dims(&entity.name, entity.direction);
        for x in entity.x..entity.x + width {
            for y in entity.y..entity.y + height {
                if occupied.insert((x, y), idx).is_some() {
                    return None;
                }
            }
        }
    }
    candidate.width -= 1;
    candidate.regions.clear();
    candidate.trace = None;
    normalize_adjacent_undergrounds(&mut candidate);
    candidate.power_wires = Some(crate::power_wires::compute_pole_wires(
        &candidate.entities,
        candidate.wire_mode,
    ));
    Some(candidate)
}

fn collapse_horizontal_cut(layout: &LayoutResult, cut: i32) -> Option<LayoutResult> {
    if cut <= 0 || cut >= layout.height {
        return None;
    }
    if layout.entities.iter().any(|entity| {
        let (_, height) = entity_dims(&entity.name, entity.direction);
        entity.y < cut && entity.y + height > cut
    }) {
        return None;
    }

    let mut candidate = layout.clone();
    for entity in &mut candidate.entities {
        if entity.y >= cut {
            entity.y -= 1;
        }
    }
    for boundary in candidate
        .boundary_inputs
        .iter_mut()
        .chain(candidate.boundary_outputs.iter_mut())
    {
        if boundary.y >= cut {
            boundary.y -= 1;
        }
    }
    for (_, _, y) in &mut candidate.surplus_exits {
        if *y >= cut {
            *y -= 1;
        }
    }

    let mut anchor_at: BTreeMap<(i32, i32), usize> = BTreeMap::new();
    let mut remove = vec![false; candidate.entities.len()];
    for (idx, entity) in candidate.entities.iter().enumerate() {
        if let Some(&other_idx) = anchor_at.get(&(entity.x, entity.y)) {
            let other = &candidate.entities[other_idx];
            if is_surface_belt(&entity.name)
                && is_surface_belt(&other.name)
                && entity.name == other.name
                && entity.direction == other.direction
                && entity.carries == other.carries
            {
                remove[idx] = true;
                continue;
            }
            return None;
        }
        anchor_at.insert((entity.x, entity.y), idx);
    }
    candidate.entities = candidate
        .entities
        .into_iter()
        .enumerate()
        .filter_map(|(idx, entity)| (!remove[idx]).then_some(entity))
        .collect();

    let mut occupied = BTreeMap::new();
    for (idx, entity) in candidate.entities.iter().enumerate() {
        let (width, height) = entity_dims(&entity.name, entity.direction);
        for x in entity.x..entity.x + width {
            for y in entity.y..entity.y + height {
                if occupied.insert((x, y), idx).is_some() {
                    return None;
                }
            }
        }
    }
    candidate.height -= 1;
    candidate.regions.clear();
    candidate.trace = None;
    normalize_adjacent_undergrounds(&mut candidate);
    candidate.power_wires = Some(crate::power_wires::compute_pole_wires(
        &candidate.entities,
        candidate.wire_mode,
    ));
    Some(candidate)
}

/// Replace safe uninterrupted straight surface-belt spans with maximal
/// underground hops. Tiles addressed by inserters or boundaries are retained
/// on the surface. Segment, item, tier and direction boundaries split runs.
pub fn undergroundify_straight_belts(layout: &LayoutResult) -> LayoutResult {
    use crate::bus::balancer::underground_for_belt;
    use std::collections::BTreeSet;

    let mut protected_horizontal = BTreeSet::new();
    let mut protected_vertical = BTreeSet::new();
    let mut protect_both = |tile| {
        protected_horizontal.insert(tile);
        protected_vertical.insert(tile);
    };
    for inserter in layout
        .entities
        .iter()
        .filter(|entity| is_inserter(&entity.name))
    {
        let (dx, dy) = dir_to_vec(inserter.direction);
        let reach = inserter_reach(&inserter.name);
        protect_both((inserter.x - dx * reach, inserter.y - dy * reach));
        protect_both((inserter.x + dx * reach, inserter.y + dy * reach));
    }
    for boundary in layout
        .boundary_inputs
        .iter()
        .chain(layout.boundary_outputs.iter())
    {
        protect_both((boundary.x, boundary.y));
    }
    for (_, x, y) in &layout.surplus_exits {
        protect_both((*x, *y));
    }
    // `protect_both` mutably borrows both sets; NLL ends that borrow here at
    // its last use, so the sets are free for the direct inserts below. (An
    // explicit `drop` would be a `clippy::drop_non_drop` error — a closure
    // does not implement `Drop`.)

    // A perpendicular belt feeding a straight tile is a side-load/turn
    // junction. Splitters and existing undergrounds reserve their vicinity
    // on both axes.
    for entity in layout
        .entities
        .iter()
        .filter(|entity| is_belt_entity(&entity.name))
    {
        let (dx, dy) = dir_to_vec(entity.direction);
        if dx == 0 {
            protected_horizontal.insert((entity.x + dx, entity.y + dy));
        } else {
            protected_vertical.insert((entity.x + dx, entity.y + dy));
        }
        if !is_surface_belt(&entity.name) {
            for x in entity.x - 1..=entity.x + 2 {
                for y in entity.y - 1..=entity.y + 2 {
                    protected_horizontal.insert((x, y));
                    protected_vertical.insert((x, y));
                }
            }
        }
    }

    // (horizontal, positive direction, fixed cross-axis coordinate, ...)
    type RunKey = (bool, bool, i32, String, Option<String>, Option<String>);
    let mut runs: BTreeMap<RunKey, Vec<(i32, usize)>> = BTreeMap::new();
    for (idx, entity) in layout.entities.iter().enumerate() {
        if !is_surface_belt(&entity.name) {
            continue;
        }
        let horizontal = matches!(
            entity.direction,
            EntityDirection::East | EntityDirection::West
        );
        let protected = if horizontal {
            &protected_horizontal
        } else {
            &protected_vertical
        };
        if protected.contains(&(entity.x, entity.y)) {
            continue;
        }
        let positive = matches!(
            entity.direction,
            EntityDirection::East | EntityDirection::South
        );
        let (fixed, coordinate) = if horizontal {
            (entity.y, entity.x)
        } else {
            (entity.x, entity.y)
        };
        runs.entry((
            horizontal,
            positive,
            fixed,
            entity.name.clone(),
            entity.carries.clone(),
            entity.segment_id.clone(),
        ))
        .or_default()
        .push((coordinate, idx));
    }

    let mut remove = vec![false; layout.entities.len()];
    let mut replacements = BTreeMap::new();
    for ((_, positive, _, belt_name, _, _), mut tiles) in runs {
        tiles.sort_by_key(|(coordinate, _)| *coordinate);
        if !positive {
            tiles.reverse();
        }
        let step = if positive { 1 } else { -1 };
        let mut run_start = 0;
        while run_start < tiles.len() {
            let mut run_end = run_start + 1;
            while run_end < tiles.len() && tiles[run_end].0 - tiles[run_end - 1].0 == step {
                run_end += 1;
            }
            let run = &tiles[run_start..run_end];
            if run.len() < 4 {
                run_start = run_end;
                continue;
            }
            let max_distance = ug_max_reach(&belt_name) as usize + 1;
            let mut start = 0;
            while run.len() - start >= 4 {
                let end = (start + max_distance).min(run.len() - 1);
                if end - start < 3 {
                    break;
                }
                let start_idx = run[start].1;
                let end_idx = run[end].1;
                let mut entrance = layout.entities[start_idx].clone();
                entrance.name = underground_for_belt(&belt_name).to_string();
                entrance.io_type = Some("input".into());
                let mut exit = layout.entities[end_idx].clone();
                exit.name = underground_for_belt(&belt_name).to_string();
                exit.io_type = Some("output".into());
                replacements.insert(start_idx, entrance);
                replacements.insert(end_idx, exit);
                for &(_, idx) in &run[start + 1..end] {
                    remove[idx] = true;
                }
                start = end + 1;
            }
            run_start = run_end;
        }
    }

    let mut result = layout.clone();
    result.entities = layout
        .entities
        .iter()
        .enumerate()
        .filter_map(|(idx, entity)| {
            if remove[idx] {
                None
            } else {
                Some(
                    replacements
                        .get(&idx)
                        .cloned()
                        .unwrap_or_else(|| entity.clone()),
                )
            }
        })
        .collect();
    result.regions.clear();
    result.trace = None;
    // `power_wires` are index pairs into `entities`, and the rebuild above
    // deletes the interior of every undergrounded run — so every stored pair
    // now names a different entity. Left stale, `wires_for` hands the
    // connectivity check a garbage graph and it reports a fragmented pole
    // network that does not exist. Both coordinate-cut passes already
    // recompute here; this one did not, and because the transport stage runs
    // FIRST and is gated on total error count, the phantom errors rejected the
    // whole stage on every bus layout measured.
    result.power_wires = Some(crate::power_wires::compute_pole_wires(
        &result.entities,
        result.wire_mode,
    ));
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RouteTerminalKind {
    ProducerDrop,
    ConsumerPickup,
    BoundaryInput,
    BoundaryOutput,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteTerminal {
    pub kind: RouteTerminalKind,
    pub x: i32,
    pub y: i32,
    pub recipe: Option<String>,
}

/// Replaceable logistics net recovered from the current embedding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteNet {
    pub item: String,
    pub segments: Vec<String>,
    pub entity_indices: Vec<usize>,
    pub terminals: Vec<RouteTerminal>,
}

/// A machine or direct-insertion cluster that must move as one unit.
///
/// `entity_indices` contains recipe-bearing machines and every inserter that
/// touches one of them. An inserter touching two machines unions those
/// machines into the same island, preserving direct insertion exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigidIsland {
    pub id: usize,
    pub entity_indices: Vec<usize>,
    pub recipes: Vec<String>,
    pub block: CompactBlock,
    pub terminals: Vec<IslandTerminal>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IslandTerminal {
    pub item: String,
    pub kind: RouteTerminalKind,
    /// Belt contact relative to the island block's top-left corner.
    pub dx: i32,
    pub dy: i32,
    /// Inserter entity that establishes this terminal.
    pub inserter_entity_index: usize,
}

/// The geometry/logistics split consumed by the RFC-057 search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactIr {
    pub islands: Vec<RigidIsland>,
    pub route_nets: Vec<RouteNet>,
    pub commodity_flows: Vec<CommodityFlow>,
}

impl CompactIr {
    pub fn from_layout(layout: &LayoutResult) -> Self {
        Self {
            islands: extract_rigid_islands(layout),
            route_nets: extract_route_nets(layout),
            commodity_flows: Vec::new(),
        }
    }

    pub fn from_source(layout: &LayoutResult, solver: &SolverResult) -> Self {
        Self {
            islands: extract_rigid_islands(layout),
            route_nets: extract_route_nets(layout),
            commodity_flows: commodity_flows(solver),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommodityFlow {
    pub item: String,
    pub rate: i64,
    pub is_fluid: bool,
}

fn commodity_flows(solver: &SolverResult) -> Vec<CommodityFlow> {
    let mut totals: BTreeMap<String, (f64, f64, bool)> = BTreeMap::new();
    for machine in &solver.machines {
        for input in &machine.inputs {
            let total = totals
                .entry(input.item.clone())
                .or_insert((0.0, 0.0, input.is_fluid));
            total.0 += input.rate * machine.count;
            total.2 |= input.is_fluid;
        }
        for output in &machine.outputs {
            let total = totals
                .entry(output.item.clone())
                .or_insert((0.0, 0.0, output.is_fluid));
            total.1 += output.rate * machine.count;
            total.2 |= output.is_fluid;
        }
    }
    totals
        .into_iter()
        .map(|(item, (consumed, produced, is_fluid))| CommodityFlow {
            item,
            rate: fixed_rate(consumed.max(produced)),
            is_fluid,
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ManifoldTerminal {
    pub kind: RouteTerminalKind,
    pub x: i32,
    pub y: i32,
    pub island_id: Option<usize>,
    pub inserter_entity_index: Option<usize>,
}

/// One multi-terminal commodity-routing problem after island placement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifoldNet {
    pub item: String,
    pub planned_rate: i64,
    pub terminals: Vec<ManifoldTerminal>,
}

impl ManifoldNet {
    pub fn required_belts(&self, belt_capacity: f64) -> u32 {
        if self.planned_rate <= 0 || belt_capacity <= 0.0 {
            return 1;
        }
        ((self.planned_rate as f64 / RATE_SCALE) / belt_capacity)
            .ceil()
            .max(1.0) as u32
    }

    pub fn producers(&self) -> impl Iterator<Item = &ManifoldTerminal> {
        self.terminals.iter().filter(|terminal| {
            matches!(
                terminal.kind,
                RouteTerminalKind::ProducerDrop | RouteTerminalKind::BoundaryInput
            )
        })
    }

    pub fn consumers(&self) -> impl Iterator<Item = &ManifoldTerminal> {
        self.terminals.iter().filter(|terminal| {
            matches!(
                terminal.kind,
                RouteTerminalKind::ConsumerPickup | RouteTerminalKind::BoundaryOutput
            )
        })
    }
}

/// Materialise route terminals at a proposed island placement.
///
/// Machine terminals move with their islands. External boundaries remain
/// relocatable interfaces at this stage: their source coordinates are kept as
/// an incumbent, but the manifold renderer may move them to its perimeter.
pub fn build_manifold_nets(
    ir: &CompactIr,
    placed_islands: &[RigidIsland],
) -> Result<Vec<ManifoldNet>, String> {
    if ir.islands.len() != placed_islands.len() {
        return Err("CompactIR and placed island counts differ".into());
    }
    let mut by_item: BTreeMap<String, Vec<ManifoldTerminal>> = ir
        .route_nets
        .iter()
        .map(|net| (net.item.clone(), Vec::new()))
        .collect();
    let planned_rate_by_item: BTreeMap<_, _> = ir
        .commodity_flows
        .iter()
        .filter(|flow| !flow.is_fluid)
        .map(|flow| (flow.item.as_str(), flow.rate))
        .collect();

    for (source, placed) in ir.islands.iter().zip(placed_islands) {
        if source.id != placed.id || source.entity_indices != placed.entity_indices {
            return Err(format!("island identity changed at {}", source.id));
        }
        for terminal in &source.terminals {
            let Some(terminals) = by_item.get_mut(&terminal.item) else {
                return Err(format!(
                    "island {} terminal item {} has no route net",
                    source.id, terminal.item
                ));
            };
            terminals.push(ManifoldTerminal {
                kind: terminal.kind,
                x: placed.block.x + terminal.dx,
                y: placed.block.y + terminal.dy,
                island_id: Some(placed.id),
                inserter_entity_index: Some(terminal.inserter_entity_index),
            });
        }
    }

    // Boundary terminals have no island-relative representation, so island
    // placement cannot carry them. Keeping their SOURCE coordinates is only
    // safe while placement preserves those coordinates; once islands are
    // repacked in 2D the stale tile is arbitrary, and it can land exactly on
    // a relocated machine terminal. After terminal-aware packing removed the
    // island-vs-island collisions, this was the entire remaining residual on
    // three of six bus fixtures.
    //
    // Relocate them onto the perimeter of the placed extent instead: inputs
    // on the row above, outputs on the row below, which also agrees with the
    // south-flowing orientation the balancer library is stamped for. Order is
    // deterministic and the stride is 2, so no two boundary belts ever touch.
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for island in placed_islands {
        let (offset_dx, offset_dy, _, height) = terminal_inclusive_extent(island);
        min_x = min_x.min(island.block.x + offset_dx);
        min_y = min_y.min(island.block.y + offset_dy);
        max_y = max_y.max(island.block.y + offset_dy + height - 1);
    }
    if min_x == i32::MAX {
        min_x = 0;
        min_y = 0;
        max_y = 0;
    }

    let mut boundaries: Vec<(RouteTerminalKind, &str, i32, i32)> = Vec::new();
    for net in &ir.route_nets {
        for terminal in &net.terminals {
            if matches!(
                terminal.kind,
                RouteTerminalKind::BoundaryInput | RouteTerminalKind::BoundaryOutput
            ) {
                boundaries.push((terminal.kind, net.item.as_str(), terminal.x, terminal.y));
            }
        }
    }
    boundaries.sort();
    boundaries.dedup();
    let (mut next_input, mut next_output) = (0i32, 0i32);
    for (kind, item, _, _) in boundaries {
        let (x, y) = if matches!(kind, RouteTerminalKind::BoundaryInput) {
            let x = min_x + next_input * 2;
            next_input += 1;
            (x, min_y - 1)
        } else {
            let x = min_x + next_output * 2;
            next_output += 1;
            (x, max_y + 1)
        };
        by_item.get_mut(item).unwrap().push(ManifoldTerminal {
            kind,
            x,
            y,
            island_id: None,
            inserter_entity_index: None,
        });
    }

    let mut result = Vec::with_capacity(by_item.len());
    for (item, mut terminals) in by_item {
        terminals.sort();
        terminals.dedup();
        if terminals.is_empty() {
            return Err(format!("route net {item} has no terminals"));
        }
        result.push(ManifoldNet {
            planned_rate: planned_rate_by_item
                .get(item.as_str())
                .copied()
                .unwrap_or(0),
            item,
            terminals,
        });
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeClusterPlacement {
    pub recipes: Vec<String>,
    pub island_ids: Vec<usize>,
    pub block: CompactBlock,
}

/// Island footprint grown to contain every terminal tile, as
/// `(offset_dx, offset_dy, width, height)` where `offset_*` (always `<= 0`)
/// says where `block` sits inside the padded extent.
///
/// A terminal is the belt tile an inserter reaches to, so it lies OUTSIDE
/// `block`. Packing on `block` alone guarantees only that machines do not
/// overlap — it freely lets one island's input terminal land exactly on
/// another island's output terminal. That is a physically impossible belt
/// tile (one tile cannot be the delivery point for two commodities), and no
/// router can repair it, because the conflict is created at placement time,
/// before routing starts.
///
/// Measured across six bus fixtures, cross-item shared terminal tiles exactly
/// equalled the manifold router's unresolved-route count — 2, 2, 44, 4, 8, 7
/// — i.e. this was the entire residual keeping candidates non-exportable.
/// Negotiated-congestion rerouting could not touch it: contested tile counts
/// fell over 24 rounds but the contested ROUTE count never moved.
fn terminal_inclusive_extent(island: &RigidIsland) -> (i32, i32, i32, i32) {
    let mut min_dx = 0;
    let mut min_dy = 0;
    let mut max_dx = island.block.width - 1;
    let mut max_dy = island.block.height - 1;
    for terminal in &island.terminals {
        min_dx = min_dx.min(terminal.dx);
        min_dy = min_dy.min(terminal.dy);
        max_dx = max_dx.max(terminal.dx);
        max_dy = max_dy.max(terminal.dy);
    }
    (min_dx, min_dy, max_dx - min_dx + 1, max_dy - min_dy + 1)
}

/// Repack rigid islands into recipe banks, then place those banks in 2D by
/// solver-rate-weighted production adjacency. This is the first RFC-057
/// placement that does not preserve incumbent row/corridor coordinates.
pub fn place_recipe_clusters(
    ir: &CompactIr,
    clearance: i32,
) -> (Vec<RigidIsland>, Vec<RecipeClusterPlacement>) {
    let clearance = clearance.max(0);
    let mut groups: BTreeMap<Vec<String>, Vec<usize>> = BTreeMap::new();
    for island in &ir.islands {
        groups
            .entry(island.recipes.clone())
            .or_default()
            .push(island.id);
    }

    let mut islands = ir.islands.clone();
    let mut clusters = Vec::new();
    for (recipes, mut island_ids) in groups {
        island_ids.sort_by_key(|&id| {
            let block = &ir.islands[id].block;
            std::cmp::Reverse((block.height, block.width, id))
        });
        // Pack on the terminal-inclusive extent, not the bare machine block —
        // see `terminal_inclusive_extent`. The shelf coordinates below address
        // that extent; `block` is then offset back inside it, so the emitted
        // machine geometry keeps its shape and only its origin moves.
        let area: i64 = island_ids
            .iter()
            .map(|&id| {
                let (_, _, width, height) = terminal_inclusive_extent(&ir.islands[id]);
                i64::from(width + clearance) * i64::from(height + clearance)
            })
            .sum();
        let target_width = (area as f64).sqrt().ceil() as i32;
        let mut x = 0;
        let mut y = 0;
        let mut row_height = 0;
        let mut max_x = 0;
        let mut max_y = 0;
        for &id in &island_ids {
            let (offset_dx, offset_dy, width, height) = terminal_inclusive_extent(&islands[id]);
            if x > 0 && x + width > target_width {
                x = 0;
                y += row_height + clearance;
                row_height = 0;
            }
            islands[id].block.x = x - offset_dx;
            islands[id].block.y = y - offset_dy;
            x += width + clearance;
            row_height = row_height.max(height);
            max_x = max_x.max(x - clearance);
            max_y = max_y.max(y + height);
        }
        let id = clusters.len();
        clusters.push(RecipeClusterPlacement {
            recipes,
            island_ids,
            block: CompactBlock {
                id,
                x: 0,
                y: 0,
                width: max_x,
                height: max_y,
            },
        });
    }

    let mut cluster_of = vec![0usize; islands.len()];
    for (cluster_idx, cluster) in clusters.iter().enumerate() {
        for &island_id in &cluster.island_ids {
            cluster_of[island_id] = cluster_idx;
        }
    }
    let mut weights: BTreeMap<(usize, usize), i64> = BTreeMap::new();
    let flow_rate: BTreeMap<&str, i64> = ir
        .commodity_flows
        .iter()
        .map(|flow| (flow.item.as_str(), flow.rate))
        .collect();
    for net in &ir.route_nets {
        let mut producers = Vec::new();
        let mut consumers = Vec::new();
        for island in &ir.islands {
            for terminal in island
                .terminals
                .iter()
                .filter(|terminal| terminal.item == net.item)
            {
                match terminal.kind {
                    RouteTerminalKind::ProducerDrop => producers.push(cluster_of[island.id]),
                    RouteTerminalKind::ConsumerPickup => consumers.push(cluster_of[island.id]),
                    _ => {}
                }
            }
        }
        producers.sort_unstable();
        producers.dedup();
        consumers.sort_unstable();
        consumers.dedup();
        let rate = flow_rate.get(net.item.as_str()).copied().unwrap_or(1);
        for &producer in &producers {
            for &consumer in &consumers {
                if producer == consumer {
                    continue;
                }
                let edge = if producer < consumer {
                    (producer, consumer)
                } else {
                    (consumer, producer)
                };
                *weights.entry(edge).or_default() += rate;
            }
        }
    }

    let mut degree = vec![0i64; clusters.len()];
    for (&(a, b), &weight) in &weights {
        degree[a] += weight;
        degree[b] += weight;
    }
    let mut order: Vec<usize> = (0..clusters.len()).collect();
    order.sort_by_key(|&idx| std::cmp::Reverse((degree[idx], clusters[idx].block.width, idx)));
    let mut placed = Vec::<usize>::new();
    for &cluster_idx in &order {
        if placed.is_empty() {
            clusters[cluster_idx].block.x = 0;
            clusters[cluster_idx].block.y = 0;
            placed.push(cluster_idx);
            continue;
        }
        let mut candidates = Vec::new();
        for &other_idx in &placed {
            let other = &clusters[other_idx].block;
            let block = &clusters[cluster_idx].block;
            candidates.extend([
                (other.x + other.width + clearance, other.y),
                (other.x - block.width - clearance, other.y),
                (other.x, other.y + other.height + clearance),
                (other.x, other.y - block.height - clearance),
            ]);
        }
        candidates.sort();
        candidates.dedup();
        let best = candidates
            .into_iter()
            .filter(|&(x, y)| {
                let mut candidate = clusters[cluster_idx].block.clone();
                candidate.x = x;
                candidate.y = y;
                placed
                    .iter()
                    .all(|&other| !blocks_overlap(&candidate, &clusters[other].block))
            })
            .min_by_key(|&(x, y)| {
                let center_x = x + clusters[cluster_idx].block.width / 2;
                let center_y = y + clusters[cluster_idx].block.height / 2;
                let wire_cost: i128 = placed
                    .iter()
                    .map(|&other| {
                        let key = if cluster_idx < other {
                            (cluster_idx, other)
                        } else {
                            (other, cluster_idx)
                        };
                        let weight = weights.get(&key).copied().unwrap_or(0) as i128;
                        let other_block = &clusters[other].block;
                        let distance = (center_x - (other_block.x + other_block.width / 2)).abs()
                            + (center_y - (other_block.y + other_block.height / 2)).abs();
                        weight * i128::from(distance)
                    })
                    .sum();
                (wire_cost, x.abs() + y.abs(), x, y)
            })
            .unwrap_or_else(|| {
                let right = placed
                    .iter()
                    .map(|&idx| clusters[idx].block.x + clusters[idx].block.width)
                    .max()
                    .unwrap_or(0);
                (right + clearance, 0)
            });
        clusters[cluster_idx].block.x = best.0;
        clusters[cluster_idx].block.y = best.1;
        placed.push(cluster_idx);
    }

    let min_x = clusters
        .iter()
        .map(|cluster| cluster.block.x)
        .min()
        .unwrap_or(0);
    let min_y = clusters
        .iter()
        .map(|cluster| cluster.block.y)
        .min()
        .unwrap_or(0);
    for cluster in &mut clusters {
        cluster.block.x -= min_x;
        cluster.block.y -= min_y;
        for &island_id in &cluster.island_ids {
            islands[island_id].block.x += cluster.block.x;
            islands[island_id].block.y += cluster.block.y;
        }
    }
    (islands, clusters)
}

pub fn estimated_manifold_wirelength(manifolds: &[ManifoldNet]) -> i128 {
    manifolds
        .iter()
        .map(|net| {
            let min_x = net
                .terminals
                .iter()
                .map(|terminal| terminal.x)
                .min()
                .unwrap_or(0);
            let max_x = net
                .terminals
                .iter()
                .map(|terminal| terminal.x)
                .max()
                .unwrap_or(0);
            let min_y = net
                .terminals
                .iter()
                .map(|terminal| terminal.y)
                .min()
                .unwrap_or(0);
            let max_y = net
                .terminals
                .iter()
                .map(|terminal| terminal.y)
                .max()
                .unwrap_or(0);
            i128::from(net.planned_rate.max(1)) * i128::from((max_x - min_x) + (max_y - min_y))
        })
        .sum()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifoldLaneGroup {
    pub lane: u32,
    pub producers: Vec<ManifoldTerminal>,
    pub consumers: Vec<ManifoldTerminal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalancerStage {
    pub n: u32,
    pub m: u32,
    pub copies: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalManifoldPlan {
    pub item: String,
    pub planned_rate: i64,
    pub belt_count: u32,
    pub hub: CompactBlock,
    pub lane_groups: Vec<ManifoldLaneGroup>,
    pub producer_stages: Vec<Vec<BalancerStage>>,
    pub consumer_stages: Vec<Vec<BalancerStage>>,
    pub all_mergers_stampable: bool,
    pub all_distributors_stampable: bool,
}

/// Plan one local, capacity-sized hub per solid commodity. Machine terminals
/// are partitioned evenly across the minimum number of express belts; each
/// lane receives an `(producers,1)` merger and `(1,consumers)` distributor.
///
/// Hubs are reserved near the terminal median and shifted to the nearest
/// collision-free tile. Rendering the merger trees and branch routes is a
/// separate transactional phase.
pub fn plan_local_manifolds(
    islands: &[RigidIsland],
    manifolds: &[ManifoldNet],
    clearance: i32,
) -> Vec<LocalManifoldPlan> {
    let clearance = clearance.max(0);
    let mut plans = Vec::new();
    for (plan_id, manifold) in manifolds.iter().enumerate() {
        let belt_count = manifold.required_belts(45.0).max(1);
        let mut producers: Vec<_> = manifold.producers().cloned().collect();
        let mut consumers: Vec<_> = manifold.consumers().cloned().collect();
        producers.sort_by_key(|terminal| (terminal.x, terminal.y, terminal.kind));
        consumers.sort_by_key(|terminal| (terminal.x, terminal.y, terminal.kind));
        let mut lane_groups: Vec<_> = (0..belt_count)
            .map(|lane| ManifoldLaneGroup {
                lane,
                producers: Vec::new(),
                consumers: Vec::new(),
            })
            .collect();
        for (idx, terminal) in producers.into_iter().enumerate() {
            lane_groups[idx % belt_count as usize]
                .producers
                .push(terminal);
        }
        for (idx, terminal) in consumers.into_iter().enumerate() {
            lane_groups[idx % belt_count as usize]
                .consumers
                .push(terminal);
        }

        let mut xs: Vec<_> = manifold
            .terminals
            .iter()
            .map(|terminal| terminal.x)
            .collect();
        let mut ys: Vec<_> = manifold
            .terminals
            .iter()
            .map(|terminal| terminal.y)
            .collect();
        xs.sort_unstable();
        ys.sort_unstable();
        let center_x = xs.get(xs.len() / 2).copied().unwrap_or(0);
        let center_y = ys.get(ys.len() / 2).copied().unwrap_or(0);
        let max_group = lane_groups
            .iter()
            .map(|group| group.producers.len().max(group.consumers.len()))
            .max()
            .unwrap_or(1) as i32;
        let depth = if max_group <= 1 {
            1
        } else {
            (max_group as f64).log2().ceil() as i32 + 1
        };
        let width = (belt_count as i32 * 3).max(3);
        let height = (depth * 2 + 1).max(3);
        let base_x = center_x - width / 2;
        let base_y = center_y - height / 2;
        let mut chosen = None;
        'radius: for radius in 0..=512 {
            for dx in -radius..=radius {
                for dy in [-radius, radius] {
                    let candidate = CompactBlock {
                        id: plan_id,
                        x: base_x + dx,
                        y: base_y + dy,
                        width,
                        height,
                    };
                    if hub_is_free(&candidate, islands, &plans, clearance) {
                        chosen = Some(candidate);
                        break 'radius;
                    }
                }
            }
            for dy in (-radius + 1)..radius {
                for dx in [-radius, radius] {
                    let candidate = CompactBlock {
                        id: plan_id,
                        x: base_x + dx,
                        y: base_y + dy,
                        width,
                        height,
                    };
                    if hub_is_free(&candidate, islands, &plans, clearance) {
                        chosen = Some(candidate);
                        break 'radius;
                    }
                }
            }
        }
        let hub = chosen.unwrap_or(CompactBlock {
            id: plan_id,
            x: base_x,
            y: base_y,
            width,
            height,
        });
        let all_mergers_stampable = lane_groups.iter().all(|group| {
            merger_stages(group.producers.len() as u32)
                .iter()
                .all(|stage| crate::bus::balancer::shape_is_stampable(stage.n, stage.m))
        });
        let all_distributors_stampable = lane_groups.iter().all(|group| {
            distributor_stages(group.consumers.len() as u32)
                .iter()
                .all(|stage| crate::bus::balancer::shape_is_stampable(stage.n, stage.m))
        });
        let producer_stages = lane_groups
            .iter()
            .map(|group| merger_stages(group.producers.len() as u32))
            .collect();
        let consumer_stages = lane_groups
            .iter()
            .map(|group| distributor_stages(group.consumers.len() as u32))
            .collect();
        plans.push(LocalManifoldPlan {
            item: manifold.item.clone(),
            planned_rate: manifold.planned_rate,
            belt_count,
            hub,
            lane_groups,
            producer_stages,
            consumer_stages,
            all_mergers_stampable,
            all_distributors_stampable,
        });
    }
    plans
}

const LOCAL_BALANCER_FAN: u32 = 4;

/// Predict the merge tree `build_local_manifold_graph` will actually build.
///
/// It must mirror that builder's chunking exactly, because
/// `all_mergers_stampable` and `producer_stages` are derived from this and
/// never cross-checked against the graph. The previous formulation folded the
/// remainder back into the next level's input count instead of emitting it as
/// its own node, so the two diverged whenever `inputs % 4` was 2 or 3 — at 6
/// producers it predicted 2 nodes against the builder's 3, and likewise at 7
/// and 10. Ordinary counts, not edge cases, and the stampability guarantee was
/// being checked against a tree that was never built.
fn merger_stages(inputs: u32) -> Vec<BalancerStage> {
    let mut stages = Vec::new();
    let mut frontier = inputs;
    while frontier > 1 {
        // The builder walks the frontier in chunks of `LOCAL_BALANCER_FAN`:
        // each full chunk becomes an (n,1) node, a trailing chunk of 2 or more
        // becomes its own smaller node, and a lone leftover passes through
        // untouched to the next level.
        let full = frontier / LOCAL_BALANCER_FAN;
        let remainder = frontier % LOCAL_BALANCER_FAN;
        let mut next = 0;
        if full > 0 {
            stages.push(BalancerStage {
                n: LOCAL_BALANCER_FAN,
                m: 1,
                copies: full,
            });
            next += full;
        }
        if remainder >= 2 {
            stages.push(BalancerStage {
                n: remainder,
                m: 1,
                copies: 1,
            });
            next += 1;
        } else {
            next += remainder;
        }
        frontier = next;
    }
    stages
}

fn distributor_stages(outputs: u32) -> Vec<BalancerStage> {
    if outputs <= 1 {
        return Vec::new();
    }
    // Grow a possibly ragged high-fanout tree to exactly the requested leaves.
    let mut stages = Vec::new();
    let mut leaves = 1;
    while leaves < outputs {
        let fan = (outputs - leaves + 1).min(LOCAL_BALANCER_FAN);
        stages.push(BalancerStage {
            n: 1,
            m: fan,
            copies: 1,
        });
        leaves += fan - 1;
    }
    stages
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManifoldEndpoint {
    Terminal(ManifoldTerminal),
    NodeInput { node: usize, port: u32 },
    NodeOutput { node: usize, port: u32 },
    LaneInput(u32),
    LaneOutput(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BalancerNodeRole {
    Merge,
    Distribute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifoldBalancerNode {
    pub id: usize,
    pub lane: u32,
    pub role: BalancerNodeRole,
    pub n: u32,
    pub m: u32,
    pub level: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ManifoldGraphEdge {
    pub from: ManifoldEndpoint,
    pub to: ManifoldEndpoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalManifoldGraph {
    pub item: String,
    pub belt_count: u32,
    pub hub: CompactBlock,
    pub nodes: Vec<ManifoldBalancerNode>,
    pub edges: Vec<ManifoldGraphEdge>,
}

/// Expand a local manifold plan into exact balancer ports and routing edges.
pub fn build_local_manifold_graph(plan: &LocalManifoldPlan) -> LocalManifoldGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for group in &plan.lane_groups {
        let mut merge_frontier: Vec<_> = group
            .producers
            .iter()
            .cloned()
            .map(ManifoldEndpoint::Terminal)
            .collect();
        let mut level = 0;
        while merge_frontier.len() > 1 {
            let mut next = Vec::new();
            for chunk in merge_frontier.chunks(LOCAL_BALANCER_FAN as usize) {
                if chunk.len() == 1 {
                    next.push(chunk[0].clone());
                    continue;
                }
                let node = nodes.len();
                nodes.push(ManifoldBalancerNode {
                    id: node,
                    lane: group.lane,
                    role: BalancerNodeRole::Merge,
                    n: chunk.len() as u32,
                    m: 1,
                    level,
                });
                for (port, source) in chunk.iter().cloned().enumerate() {
                    edges.push(ManifoldGraphEdge {
                        from: source,
                        to: ManifoldEndpoint::NodeInput {
                            node,
                            port: port as u32,
                        },
                    });
                }
                next.push(ManifoldEndpoint::NodeOutput { node, port: 0 });
            }
            merge_frontier = next;
            level += 1;
        }
        if let Some(root) = merge_frontier.pop() {
            edges.push(ManifoldGraphEdge {
                from: root,
                to: ManifoldEndpoint::LaneInput(group.lane),
            });
        }

        let consumer_count = group.consumers.len();
        let mut distribute_frontier = vec![(ManifoldEndpoint::LaneOutput(group.lane), 0u32)];
        while distribute_frontier.len() < consumer_count {
            let (source, level) = distribute_frontier.remove(0);
            let fan = (consumer_count - distribute_frontier.len()).min(LOCAL_BALANCER_FAN as usize)
                as u32;
            let node = nodes.len();
            nodes.push(ManifoldBalancerNode {
                id: node,
                lane: group.lane,
                role: BalancerNodeRole::Distribute,
                n: 1,
                m: fan,
                level,
            });
            edges.push(ManifoldGraphEdge {
                from: source,
                to: ManifoldEndpoint::NodeInput { node, port: 0 },
            });
            for port in 0..fan {
                distribute_frontier.push((ManifoldEndpoint::NodeOutput { node, port }, level + 1));
            }
        }
        distribute_frontier.sort_by(|a, b| a.0.cmp(&b.0));
        for (source, terminal) in distribute_frontier
            .into_iter()
            .zip(group.consumers.iter().cloned())
        {
            edges.push(ManifoldGraphEdge {
                from: source.0,
                to: ManifoldEndpoint::Terminal(terminal),
            });
        }
    }
    edges.sort();
    LocalManifoldGraph {
        item: plan.item.clone(),
        belt_count: plan.belt_count,
        hub: plan.hub.clone(),
        nodes,
        edges,
    }
}

#[derive(Clone, Debug)]
pub struct PlacedManifoldNode {
    pub node_id: usize,
    pub origin: (i32, i32),
    pub direction: EntityDirection,
    pub input_ports: Vec<(i32, i32)>,
    pub output_ports: Vec<(i32, i32)>,
}

#[derive(Clone, Debug)]
pub struct PlacedLocalManifold {
    pub item: String,
    pub hub: CompactBlock,
    pub nodes: Vec<PlacedManifoldNode>,
    pub lane_inputs: Vec<(i32, i32)>,
    pub lane_outputs: Vec<(i32, i32)>,
    pub entities: Vec<PlacedEntity>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutedManifoldEdge {
    pub item: String,
    pub edge: ManifoldGraphEdge,
    /// Manhattan-adjacent tiles, including both graph endpoints.
    pub path: Vec<(i32, i32)>,
    /// Tiles already claimed by an earlier edge. These require a shared
    /// segment, an underground crossing, or negotiated rerouting before the
    /// path can be materialised.
    pub crossings: Vec<(i32, i32)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManifoldRoutingResult {
    pub routes: Vec<RoutedManifoldEdge>,
    pub unroutable: Vec<(String, ManifoldGraphEdge)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegalizedManifoldRoute {
    pub item: String,
    pub edge: ManifoldGraphEdge,
    /// Adjacent surface steps plus legal underground jumps.
    pub path: Vec<(i32, i32)>,
    /// Surface tiles still claimed by an earlier route.
    pub unresolved_tiles: Vec<(i32, i32)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManifoldLegalizationResult {
    pub routes: Vec<LegalizedManifoldRoute>,
    pub unresolved_routes: usize,
    pub unresolved_tiles: usize,
    pub underground_spans: usize,
}

/// Convert a fully legalised route set into belt entities.
///
/// Fixed node/lane endpoint belts are already present in the stamped hubs and
/// are omitted here. Any unresolved surface claim rejects the whole
/// transaction.
pub fn materialize_legalized_manifold_routes(
    routes: &[LegalizedManifoldRoute],
) -> Result<Vec<PlacedEntity>, String> {
    use crate::bus::trunk_renderer::render_path;

    let unresolved_routes = routes
        .iter()
        .filter(|route| !route.unresolved_tiles.is_empty())
        .count();
    if unresolved_routes > 0 {
        return Err(format!(
            "{unresolved_routes} manifold routes still contain surface conflicts"
        ));
    }

    let mut entities = Vec::new();
    let mut occupied = FxHashSet::default();
    for (route_idx, route) in routes.iter().enumerate() {
        if route.path.len() < 2 {
            return Err(format!(
                "manifold route {route_idx} has fewer than two points"
            ));
        }
        let direction = path_exit_direction(&route.path)
            .ok_or_else(|| format!("manifold route {route_idx} has no cardinal exit direction"))?;
        let mut rendered = render_path(
            &route.path,
            &route.item,
            "express-transport-belt",
            direction,
            Some(format!("compact-route:{}:{route_idx}", route.item)),
            None,
        );
        let omit_start =
            endpoint_has_fixed_port_direction(&route.edge.from).then_some(route.path[0]);
        let omit_end = endpoint_has_fixed_port_direction(&route.edge.to)
            .then_some(*route.path.last().unwrap());
        rendered.retain(|entity| {
            Some((entity.x, entity.y)) != omit_start && Some((entity.x, entity.y)) != omit_end
        });
        for entity in &rendered {
            let (width, height) = entity_dims(&entity.name, entity.direction);
            for x in entity.x..entity.x + width {
                for y in entity.y..entity.y + height {
                    if !occupied.insert((x, y)) {
                        return Err(format!("materialized manifold routes overlap at ({x},{y})"));
                    }
                }
            }
        }
        entities.extend(rendered);
    }
    Ok(entities)
}

fn path_exit_direction(path: &[(i32, i32)]) -> Option<EntityDirection> {
    for pair in path.windows(2).rev() {
        let dx = (pair[1].0 - pair[0].0).signum();
        let dy = (pair[1].1 - pair[0].1).signum();
        match (dx, dy) {
            (1, 0) => return Some(EntityDirection::East),
            (-1, 0) => return Some(EntityDirection::West),
            (0, 1) => return Some(EntityDirection::South),
            (0, -1) => return Some(EntityDirection::North),
            _ => {}
        }
    }
    None
}

/// Route every explicit edge in the local manifold graphs.
///
/// This is deliberately a topology result, not yet belt geometry: paths may
/// overlap earlier paths and report those tiles as crossings. A later
/// transactional materialiser must resolve each crossing before committing
/// entities. Keeping this boundary explicit prevents provisional ghost belts
/// from masquerading as a valid factory.
pub fn route_local_manifold_edges(
    islands: &[RigidIsland],
    graphs: &[LocalManifoldGraph],
    hubs: &[PlacedLocalManifold],
) -> Result<ManifoldRoutingResult, String> {
    if graphs.len() != hubs.len() {
        return Err(format!(
            "manifold graph/hub count differs: {} != {}",
            graphs.len(),
            hubs.len()
        ));
    }

    let mut points = Vec::new();
    for island in islands {
        points.push((island.block.x, island.block.y));
        points.push((
            island.block.x + island.block.width - 1,
            island.block.y + island.block.height - 1,
        ));
    }
    for hub in hubs {
        points.push((hub.hub.x, hub.hub.y));
        points.push((
            hub.hub.x + hub.hub.width - 1,
            hub.hub.y + hub.hub.height - 1,
        ));
    }
    for (graph, hub) in graphs.iter().zip(hubs) {
        if graph.item != hub.item {
            return Err(format!(
                "manifold graph/hub item differs: {} != {}",
                graph.item, hub.item
            ));
        }
        for edge in &graph.edges {
            points.push(manifold_endpoint_point(&edge.from, hub)?);
            points.push(manifold_endpoint_point(&edge.to, hub)?);
        }
    }
    if points.is_empty() {
        return Ok(ManifoldRoutingResult::default());
    }

    const PADDING: i32 = 8;
    let min_x = points.iter().map(|point| point.0).min().unwrap() - PADDING;
    let min_y = points.iter().map(|point| point.1).min().unwrap() - PADDING;
    let max_x = points.iter().map(|point| point.0).max().unwrap() + PADDING;
    let max_y = points.iter().map(|point| point.1).max().unwrap() + PADDING;
    let shift = (-min_x, -min_y);
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let shifted = |point: (i32, i32)| (point.0 + shift.0, point.1 + shift.1);

    // Island blocks are conservative hard reservations: they include the
    // machine/inserter geometry and the deliberately retained local space
    // between them. Hub reservations use exact stamped entity footprints so
    // paths can use the otherwise empty interior of a large hub block.
    let mut hard = FxHashSet::default();
    for island in islands {
        for x in island.block.x..island.block.x + island.block.width {
            for y in island.block.y..island.block.y + island.block.height {
                hard.insert(shifted((x, y)));
            }
        }
    }
    for hub in hubs {
        for entity in &hub.entities {
            let (entity_width, entity_height) = entity_dims(&entity.name, entity.direction);
            for x in entity.x..entity.x + entity_width {
                for y in entity.y..entity.y + entity_height {
                    hard.insert(shifted((x, y)));
                }
            }
        }
    }
    let mut exclusive_hard = hard.clone();
    // Keep every terminal and fixed-port access socket available until its
    // own edge is routed. Underground interiors may pass beneath these
    // reservations, but no unrelated surface endpoint may consume them.
    for (graph, hub) in graphs.iter().zip(hubs) {
        for edge in &graph.edges {
            for (endpoint, output_side) in [(&edge.from, true), (&edge.to, false)] {
                let socket = manifold_endpoint_socket(endpoint, hub, output_side)?;
                exclusive_hard.insert(shifted(socket));
            }
        }
    }
    let mut claimed = FxHashSet::default();
    let mut result = ManifoldRoutingResult::default();
    let mut tasks = Vec::new();
    for (graph_idx, graph) in graphs.iter().enumerate() {
        for edge in &graph.edges {
            let hub = &hubs[graph_idx];
            let start = manifold_endpoint_point(&edge.from, hub)?;
            let goal = manifold_endpoint_point(&edge.to, hub)?;
            let span = (start.0 - goal.0).abs() + (start.1 - goal.1).abs();
            tasks.push((span, graph_idx, edge));
        }
    }
    tasks.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(b.2)));
    let mut fallback = Vec::new();
    for &(_, graph_idx, edge) in &tasks {
        let graph = &graphs[graph_idx];
        let hub = &hubs[graph_idx];
        let start = manifold_endpoint_point(&edge.from, hub)?;
        let goal = manifold_endpoint_point(&edge.to, hub)?;
        // Library balancers and capacity lanes are SOUTH-oriented.
        // Route from the tile immediately below an output and into the
        // tile immediately above an input; the stamped endpoint belt then
        // supplies the final directed step. Terminals are newly emitted
        // belts and therefore impose no pre-existing direction.
        let route_start = manifold_endpoint_socket(&edge.from, hub, true)?;
        let route_goal = manifold_endpoint_socket(&edge.to, hub, false)?;
        let shifted_start = shifted(route_start);
        let shifted_goal = shifted(route_goal);
        let mut edge_hard = exclusive_hard.clone();
        edge_hard.remove(&shifted_start);
        edge_hard.remove(&shifted_goal);
        let Some(path) = compact_belt_astar(
            shifted_start,
            shifted_goal,
            &edge_hard,
            &claimed,
            width,
            height,
        ) else {
            fallback.push((graph_idx, edge));
            continue;
        };
        let mut path: Vec<_> = path
            .into_iter()
            .map(|point| (point.0 - shift.0, point.1 - shift.1))
            .collect();
        if route_start != start {
            path.insert(0, start);
        }
        if route_goal != goal {
            path.push(goal);
        }
        let crossings = Vec::new();
        let claim_start = usize::from(route_start != start);
        let claim_end = path.len() - usize::from(route_goal != goal);
        claimed.extend(path[claim_start..claim_end].iter().copied().map(shifted));
        result.routes.push(RoutedManifoldEdge {
            item: graph.item.clone(),
            edge: edge.clone(),
            path,
            crossings,
        });
    }
    let mut retry_hard = hard.clone();
    for &(graph_idx, edge) in &fallback {
        let hub = &hubs[graph_idx];
        retry_hard.insert(shifted(manifold_endpoint_socket(&edge.from, hub, true)?));
        retry_hard.insert(shifted(manifold_endpoint_socket(&edge.to, hub, false)?));
    }
    let mut ghost_fallback = Vec::new();
    for (graph_idx, edge) in fallback {
        let graph = &graphs[graph_idx];
        let hub = &hubs[graph_idx];
        let start = manifold_endpoint_point(&edge.from, hub)?;
        let goal = manifold_endpoint_point(&edge.to, hub)?;
        let route_start = manifold_endpoint_socket(&edge.from, hub, true)?;
        let route_goal = manifold_endpoint_socket(&edge.to, hub, false)?;
        let shifted_start = shifted(route_start);
        let shifted_goal = shifted(route_goal);
        let mut edge_hard = retry_hard.clone();
        edge_hard.remove(&shifted_start);
        edge_hard.remove(&shifted_goal);
        let Some(path) = compact_belt_astar(
            shifted_start,
            shifted_goal,
            &edge_hard,
            &claimed,
            width,
            height,
        ) else {
            ghost_fallback.push((graph_idx, edge));
            continue;
        };
        let mut path: Vec<_> = path
            .into_iter()
            .map(|point| (point.0 - shift.0, point.1 - shift.1))
            .collect();
        if route_start != start {
            path.insert(0, start);
        }
        if route_goal != goal {
            path.push(goal);
        }
        let claim_start = usize::from(route_start != start);
        let claim_end = path.len() - usize::from(route_goal != goal);
        claimed.extend(path[claim_start..claim_end].iter().copied().map(shifted));
        result.routes.push(RoutedManifoldEdge {
            item: graph.item.clone(),
            edge: edge.clone(),
            path,
            crossings: Vec::new(),
        });
    }
    // Preserve progress from the exclusive underground-aware pass, then use
    // ghost paths only for the residual edges. These routes remain explicit
    // legalization work instead of causing the entire candidate to vanish.
    let axis_costs = FxHashMap::default();
    for (graph_idx, edge) in ghost_fallback {
        let graph = &graphs[graph_idx];
        let hub = &hubs[graph_idx];
        let start = manifold_endpoint_point(&edge.from, hub)?;
        let goal = manifold_endpoint_point(&edge.to, hub)?;
        let route_start = manifold_endpoint_socket(&edge.from, hub, true)?;
        let route_goal = manifold_endpoint_socket(&edge.to, hub, false)?;
        let shifted_start = shifted(route_start);
        let shifted_goal = shifted(route_goal);
        let mut edge_hard = hard.clone();
        edge_hard.remove(&shifted_start);
        edge_hard.remove(&shifted_goal);
        let Some((path, crossings)) = crate::astar::ghost_astar(
            shifted_start,
            shifted_goal,
            &edge_hard,
            &claimed,
            width,
            height,
            2,
            &axis_costs,
        ) else {
            result.unroutable.push((graph.item.clone(), edge.clone()));
            continue;
        };
        let mut path: Vec<_> = path
            .into_iter()
            .map(|point| (point.0 - shift.0, point.1 - shift.1))
            .collect();
        if route_start != start {
            path.insert(0, start);
        }
        if route_goal != goal {
            path.push(goal);
        }
        let crossings: Vec<_> = crossings
            .into_iter()
            .map(|point| (point.0 - shift.0, point.1 - shift.1))
            .collect();
        let claim_start = usize::from(route_start != start);
        let claim_end = path.len() - usize::from(route_goal != goal);
        claimed.extend(path[claim_start..claim_end].iter().copied().map(shifted));
        result.routes.push(RoutedManifoldEdge {
            item: graph.item.clone(),
            edge: edge.clone(),
            path,
            crossings,
        });
    }
    Ok(result)
}

/// Entity-cost A* for dense solid routing.
///
/// Surface moves require a free destination. Underground moves may cross any
/// hard or previously routed interior tile, but their endpoints must be free.
/// A jump costs two entities regardless of distance, so the search naturally
/// prefers maximal legal underground spans without a separate compression
/// pass.
fn compact_belt_astar(
    start: (i32, i32),
    goal: (i32, i32),
    hard: &FxHashSet<(i32, i32)>,
    occupied: &FxHashSet<(i32, i32)>,
    width: i32,
    height: i32,
) -> Option<Vec<(i32, i32)>> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    struct State {
        x: i32,
        y: i32,
        dir: i8,
    }

    const DIRECTIONS: [(i32, i32, i8); 4] = [(1, 0, 0), (0, 1, 1), (-1, 0, 2), (0, -1, 3)];
    if occupied.contains(&start) || occupied.contains(&goal) {
        return None;
    }
    let start_state = State {
        x: start.0,
        y: start.1,
        dir: -1,
    };
    let mut queue = BinaryHeap::new();
    let mut distance: FxHashMap<State, u32> = FxHashMap::default();
    let mut parent: FxHashMap<State, State> = FxHashMap::default();
    let max_jump = ug_max_reach("express-transport-belt") as i32 + 1;
    let axis_lower_bound = |distance: i32| {
        let distance = distance.unsigned_abs();
        let full = distance / max_jump as u32;
        let remainder = distance % max_jump as u32;
        full * 2
            + match remainder {
                0 => 0,
                1 => 1,
                _ => 2,
            }
    };
    let heuristic =
        |point: (i32, i32)| axis_lower_bound(goal.0 - point.0) + axis_lower_bound(goal.1 - point.1);
    queue.push(Reverse((heuristic(start), 0u32, start_state)));
    distance.insert(start_state, 0u32);

    while let Some(Reverse((_, cost, state))) = queue.pop() {
        if cost != distance.get(&state).copied().unwrap_or(u32::MAX) {
            continue;
        }
        if (state.x, state.y) == goal {
            let mut path = vec![goal];
            let mut cursor = state;
            while let Some(&previous) = parent.get(&cursor) {
                path.push((previous.x, previous.y));
                cursor = previous;
            }
            path.reverse();
            return Some(path);
        }
        for &(dx, dy, dir) in &DIRECTIONS {
            let turn_cost = u32::from(state.dir != -1 && state.dir != dir) * 2;
            for jump in 1..=max_jump {
                // A distance-two underground pair is never cheaper than two
                // surface belts. Keep it available only when the intervening
                // tile is blocked.
                if jump == 2 {
                    let middle = (state.x + dx, state.y + dy);
                    if !hard.contains(&middle) && !occupied.contains(&middle) {
                        continue;
                    }
                }
                let next = (state.x + dx * jump, state.y + dy * jump);
                if next.0 < 0 || next.0 >= width || next.1 < 0 || next.1 >= height {
                    break;
                }
                if occupied.contains(&next) || (next != goal && hard.contains(&next)) {
                    continue;
                }
                let entity_cost = if jump == 1 { 1 } else { 2 };
                let next_cost = cost + entity_cost + turn_cost;
                let next_state = State {
                    x: next.0,
                    y: next.1,
                    dir,
                };
                if next_cost < distance.get(&next_state).copied().unwrap_or(u32::MAX) {
                    distance.insert(next_state, next_cost);
                    parent.insert(next_state, state);
                    queue.push(Reverse((
                        next_cost + heuristic(next),
                        next_cost,
                        next_state,
                    )));
                }
            }
        }
    }
    None
}

/// Hide bounded conflicting runs beneath earlier routes.
///
/// Express underground belts may bridge at most eight hidden tiles, so their
/// endpoints may be nine tiles apart. This pass greedily replaces a straight
/// adjacent subpath with such a jump whenever its interior contains an
/// already claimed surface tile and both endpoints remain free. Fixed
/// balancer/lane ports cannot themselves become underground endpoints.
///
/// Residual overlaps remain explicit and make the candidate non-exportable.
pub fn legalize_manifold_routes(routes: &[RoutedManifoldEdge]) -> ManifoldLegalizationResult {
    let max_distance = ug_max_reach("express-transport-belt") as usize + 1;
    let mut occupied = FxHashSet::default();
    let mut result = ManifoldLegalizationResult::default();
    for route in routes {
        if route.path.is_empty() {
            result.unresolved_routes += 1;
            result.routes.push(LegalizedManifoldRoute {
                item: route.item.clone(),
                edge: route.edge.clone(),
                path: Vec::new(),
                unresolved_tiles: Vec::new(),
            });
            continue;
        }
        let source_fixed = endpoint_has_fixed_port_direction(&route.edge.from);
        let target_fixed = endpoint_has_fixed_port_direction(&route.edge.to);
        let mut path = Vec::with_capacity(route.path.len());
        let mut i = 0;
        path.push(route.path[0]);
        while i + 1 < route.path.len() {
            let mut jump = None;
            let max_j = (i + max_distance).min(route.path.len() - 1);
            for j in ((i + 2)..=max_j).rev() {
                if (i == 0 && source_fixed) || (j + 1 == route.path.len() && target_fixed) {
                    continue;
                }
                if occupied.contains(&route.path[i]) || occupied.contains(&route.path[j]) {
                    continue;
                }
                let Some(step) = straight_adjacent_step(&route.path[i..=j]) else {
                    continue;
                };
                if step == (0, 0) {
                    continue;
                }
                if route.path[i + 1..j]
                    .iter()
                    .any(|tile| occupied.contains(tile))
                {
                    jump = Some(j);
                    break;
                }
            }
            if let Some(j) = jump {
                path.push(route.path[j]);
                result.underground_spans += 1;
                i = j;
            } else {
                i += 1;
                path.push(route.path[i]);
            }
        }

        let claim_start = usize::from(source_fixed);
        let claim_end = path.len() - usize::from(target_fixed);
        let unresolved_tiles: Vec<_> = path[claim_start..claim_end]
            .iter()
            .copied()
            .filter(|tile| occupied.contains(tile))
            .collect();
        if !unresolved_tiles.is_empty() {
            result.unresolved_routes += 1;
            result.unresolved_tiles += unresolved_tiles.len();
        }
        occupied.extend(path[claim_start..claim_end].iter().copied());
        result.routes.push(LegalizedManifoldRoute {
            item: route.item.clone(),
            edge: route.edge.clone(),
            path,
            unresolved_tiles,
        });
    }
    result
}

fn straight_adjacent_step(path: &[(i32, i32)]) -> Option<(i32, i32)> {
    let first = *path.first()?;
    let second = *path.get(1)?;
    let step = (second.0 - first.0, second.1 - first.1);
    if step.0.abs() + step.1.abs() != 1 {
        return None;
    }
    path.windows(2)
        .all(|pair| (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1) == step)
        .then_some(step)
}

fn endpoint_has_fixed_port_direction(endpoint: &ManifoldEndpoint) -> bool {
    !matches!(endpoint, ManifoldEndpoint::Terminal(_))
}

fn manifold_endpoint_point(
    endpoint: &ManifoldEndpoint,
    hub: &PlacedLocalManifold,
) -> Result<(i32, i32), String> {
    match endpoint {
        ManifoldEndpoint::Terminal(terminal) => Ok((terminal.x, terminal.y)),
        ManifoldEndpoint::NodeInput { node, port } => hub
            .nodes
            .get(*node)
            .and_then(|placed| placed.input_ports.get(*port as usize))
            .copied()
            .ok_or_else(|| format!("{} node {node} input port {port} is missing", hub.item)),
        ManifoldEndpoint::NodeOutput { node, port } => hub
            .nodes
            .get(*node)
            .and_then(|placed| placed.output_ports.get(*port as usize))
            .copied()
            .ok_or_else(|| format!("{} node {node} output port {port} is missing", hub.item)),
        ManifoldEndpoint::LaneInput(lane) => hub
            .lane_inputs
            .get(*lane as usize)
            .copied()
            .ok_or_else(|| format!("{} lane {lane} input is missing", hub.item)),
        ManifoldEndpoint::LaneOutput(lane) => hub
            .lane_outputs
            .get(*lane as usize)
            .copied()
            .ok_or_else(|| format!("{} lane {lane} output is missing", hub.item)),
    }
}

fn manifold_endpoint_socket(
    endpoint: &ManifoldEndpoint,
    hub: &PlacedLocalManifold,
    output_side: bool,
) -> Result<(i32, i32), String> {
    let point = manifold_endpoint_point(endpoint, hub)?;
    let direction = match endpoint {
        ManifoldEndpoint::Terminal(_) => return Ok(point),
        ManifoldEndpoint::NodeInput { node, .. } | ManifoldEndpoint::NodeOutput { node, .. } => hub
            .nodes
            .get(*node)
            .map(|placed| placed.direction)
            .ok_or_else(|| format!("{} node {node} direction is missing", hub.item))?,
        ManifoldEndpoint::LaneInput(_) | ManifoldEndpoint::LaneOutput(_) => EntityDirection::South,
    };
    let (dx, dy) = dir_to_vec(direction);
    if output_side {
        Ok((point.0 + dx, point.1 + dy))
    } else {
        Ok((point.0 - dx, point.1 - dy))
    }
}

/// Stamp the largest available `(n,1)` / `(1,m)` nodes into collision-free
/// local hubs.
/// Inter-node and terminal edges remain explicit in [`LocalManifoldGraph`]
/// for the negotiated router.
pub fn place_local_manifold_nodes(
    islands: &[RigidIsland],
    graphs: &[LocalManifoldGraph],
    clearance: i32,
) -> Vec<PlacedLocalManifold> {
    use crate::bus::balancer::{splitter_for_belt, underground_for_belt};
    use crate::bus::balancer_library::balancer_templates;

    let templates = balancer_templates();
    let clearance = clearance.max(0);
    let mut result: Vec<PlacedLocalManifold> = Vec::new();
    for (graph_idx, graph) in graphs.iter().enumerate() {
        let mut level_dims = BTreeMap::<(u32, u8, u32), (i32, i32)>::new();
        for node in &graph.nodes {
            let role = match node.role {
                BalancerNodeRole::Merge => 0,
                BalancerNodeRole::Distribute => 1,
            };
            let template = templates
                .get(&(node.n, node.m))
                .expect("planned local balancer template missing");
            let dims = level_dims.entry((node.lane, role, node.level)).or_default();
            if dims.0 > 0 {
                dims.0 += clearance;
            }
            dims.0 += template.width as i32;
            dims.1 = dims.1.max(template.height as i32);
        }
        let mut lane_widths = Vec::new();
        let mut merge_height = 0;
        let mut distribute_height = 0;
        for lane in 0..graph.belt_count {
            lane_widths.push(
                level_dims
                    .iter()
                    .filter(|((node_lane, _, _), _)| *node_lane == lane)
                    .map(|(_, dims)| dims.0)
                    .max()
                    .unwrap_or(1)
                    .max(1),
            );
            let role_height = |role: u8| {
                let levels: Vec<_> = level_dims
                    .iter()
                    .filter(|((node_lane, node_role, _), _)| {
                        *node_lane == lane && *node_role == role
                    })
                    .map(|((_, _, level), (_, height))| (*level, *height))
                    .collect();
                let count = levels.len();
                levels.into_iter().map(|(_, height)| height).sum::<i32>()
                    + clearance * count.saturating_sub(1) as i32
            };
            merge_height = merge_height.max(role_height(0));
            distribute_height = distribute_height.max(role_height(1));
        }
        let width = lane_widths.iter().sum::<i32>()
            + clearance * (lane_widths.len().saturating_sub(1) as i32);
        let lane_y = merge_height;
        let distribute_y = lane_y + 2;
        let height = distribute_y + distribute_height.max(1);
        let preferred_x = graph.hub.x + graph.hub.width / 2 - width / 2;
        let preferred_y = graph.hub.y + graph.hub.height / 2 - height / 2;
        let mut hub = None;
        'radius: for radius in 0..=1024 {
            for dx in -radius..=radius {
                for dy in [-radius, radius] {
                    let candidate = CompactBlock {
                        id: graph_idx,
                        x: preferred_x + dx,
                        y: preferred_y + dy,
                        width,
                        height,
                    };
                    if placed_hub_is_free(&candidate, islands, &result, clearance) {
                        hub = Some(candidate);
                        break 'radius;
                    }
                }
            }
            for dy in (-radius + 1)..radius {
                for dx in [-radius, radius] {
                    let candidate = CompactBlock {
                        id: graph_idx,
                        x: preferred_x + dx,
                        y: preferred_y + dy,
                        width,
                        height,
                    };
                    if placed_hub_is_free(&candidate, islands, &result, clearance) {
                        hub = Some(candidate);
                        break 'radius;
                    }
                }
            }
        }
        let hub = hub.unwrap_or(CompactBlock {
            id: graph_idx,
            x: preferred_x,
            y: preferred_y,
            width,
            height,
        });

        let mut lane_starts = Vec::new();
        let mut cursor = hub.x;
        for lane_width in &lane_widths {
            lane_starts.push(cursor);
            cursor += *lane_width + clearance;
        }
        let mut level_x_offsets = BTreeMap::<(u32, u8, u32), i32>::new();
        let mut placed_nodes = Vec::new();
        let mut entities = Vec::new();
        for node in &graph.nodes {
            let role = match node.role {
                BalancerNodeRole::Merge => 0,
                BalancerNodeRole::Distribute => 1,
            };
            let template = templates
                .get(&(node.n, node.m))
                .expect("planned local balancer template missing");
            let x_offset = level_x_offsets
                .entry((node.lane, role, node.level))
                .or_default();
            let origin_x = lane_starts[node.lane as usize] + *x_offset;
            let origin_y = hub.y
                + if node.role == BalancerNodeRole::Merge {
                    level_dims
                        .iter()
                        .filter(|((lane, node_role, level), _)| {
                            *lane == node.lane && *node_role == role && *level < node.level
                        })
                        .map(|(_, (_, height))| *height + clearance)
                        .sum::<i32>()
                } else {
                    distribute_y
                        + level_dims
                            .iter()
                            .filter(|((lane, node_role, level), _)| {
                                *lane == node.lane && *node_role == role && *level < node.level
                            })
                            .map(|(_, (_, height))| *height + clearance)
                            .sum::<i32>()
                };
            *x_offset += template.width as i32 + clearance;
            let mut stamped = template.stamp(
                origin_x,
                origin_y,
                "express-transport-belt",
                splitter_for_belt("express-transport-belt"),
                underground_for_belt("express-transport-belt"),
                Some(&graph.item),
            );
            for entity in &mut stamped {
                entity.segment_id = Some(format!(
                    "compact-hub:{}:{}:{}",
                    graph.item, node.lane, node.id
                ));
            }
            entities.extend(stamped);
            placed_nodes.push(PlacedManifoldNode {
                node_id: node.id,
                origin: (origin_x, origin_y),
                direction: EntityDirection::South,
                input_ports: template
                    .input_tiles
                    .iter()
                    .map(|&(x, y)| (origin_x + x, origin_y + y))
                    .collect(),
                output_ports: template
                    .output_tiles
                    .iter()
                    .map(|&(x, y)| (origin_x + x, origin_y + y))
                    .collect(),
            });
        }
        placed_nodes.sort_by_key(|node| node.node_id);
        let lane_inputs: Vec<_> = lane_starts
            .iter()
            .zip(&lane_widths)
            .map(|(&x, &lane_width)| (x + lane_width / 2, hub.y + lane_y))
            .collect();
        let lane_outputs: Vec<_> = lane_inputs
            .iter()
            .map(|&(x, _)| (x, hub.y + distribute_y - 1))
            .collect();
        for (lane, (&input, &output)) in lane_inputs.iter().zip(&lane_outputs).enumerate() {
            for y in input.1..=output.1 {
                entities.push(PlacedEntity {
                    name: "express-transport-belt".into(),
                    x: input.0,
                    y,
                    direction: EntityDirection::South,
                    carries: Some(graph.item.clone()),
                    segment_id: Some(format!("compact-lane:{}:{lane}", graph.item)),
                    ..Default::default()
                });
            }
        }
        result.push(PlacedLocalManifold {
            item: graph.item.clone(),
            hub,
            nodes: placed_nodes,
            lane_inputs,
            lane_outputs,
            entities,
        });
    }
    result
}

/// Place manifold primitives near the terminals they aggregate instead of in
/// one commodity-wide rectangle. Leaf mergers sit near their producer group;
/// distributors sit near the consumers reachable from their outputs. Only
/// the capacity lanes remain at the commodity median.
pub fn place_distributed_local_manifold_nodes(
    islands: &[RigidIsland],
    graphs: &[LocalManifoldGraph],
    clearance: i32,
) -> Result<Vec<PlacedLocalManifold>, String> {
    use crate::bus::balancer_library::balancer_templates;

    let templates = balancer_templates();
    let clearance = clearance.max(0);
    let mut reserved: Vec<CompactBlock> =
        islands.iter().map(|island| island.block.clone()).collect();
    let mut result = Vec::new();
    for (graph_idx, graph) in graphs.iter().enumerate() {
        let reservation_start = reserved.len();
        let terminals: Vec<_> = graph
            .edges
            .iter()
            .flat_map(|edge| [&edge.from, &edge.to])
            .filter_map(|endpoint| match endpoint {
                ManifoldEndpoint::Terminal(terminal) => Some((terminal.x, terminal.y)),
                _ => None,
            })
            .collect();
        let median = coordinate_median(&terminals);
        let lane_width = graph.belt_count.max(1) as i32;
        let lane_block = nearest_free_block(
            CompactBlock {
                id: graph_idx,
                x: median.0 - lane_width / 2,
                y: median.1 - 1,
                width: lane_width,
                height: 2,
            },
            &reserved,
            clearance,
            1024,
        )
        .ok_or_else(|| format!("{} has no collision-free lane block", graph.item))?;
        reserved.push(lane_block.clone());
        let lane_inputs: Vec<_> = (0..graph.belt_count)
            .map(|lane| (lane_block.x + lane as i32, lane_block.y))
            .collect();
        let lane_outputs: Vec<_> = lane_inputs.iter().map(|&(x, y)| (x, y + 1)).collect();
        let mut entities = Vec::new();
        for (lane, (&input, &output)) in lane_inputs.iter().zip(&lane_outputs).enumerate() {
            for y in input.1..=output.1 {
                entities.push(PlacedEntity {
                    name: "express-transport-belt".into(),
                    x: input.0,
                    y,
                    direction: EntityDirection::South,
                    carries: Some(graph.item.clone()),
                    segment_id: Some(format!("compact-lane:{}:{lane}", graph.item)),
                    ..Default::default()
                });
            }
        }

        let mut placed_nodes: Vec<PlacedManifoldNode> = Vec::new();
        for node in &graph.nodes {
            let template = templates
                .get(&(node.n, node.m))
                .ok_or_else(|| format!("missing local template ({},{})", node.n, node.m))?;
            let desired_points = match node.role {
                BalancerNodeRole::Merge => graph
                    .edges
                    .iter()
                    .filter_map(|edge| match &edge.to {
                        ManifoldEndpoint::NodeInput { node: target, .. } if *target == node.id => {
                            partial_endpoint_point(
                                &edge.from,
                                &placed_nodes,
                                &lane_inputs,
                                &lane_outputs,
                            )
                        }
                        _ => None,
                    })
                    .collect(),
                BalancerNodeRole::Distribute => descendant_terminal_points(graph, node.id),
            };
            let center = if desired_points.is_empty() {
                median
            } else {
                coordinate_median(&desired_points)
            };
            let (flow_from, flow_to) = match node.role {
                BalancerNodeRole::Merge => (center, lane_inputs[node.lane as usize]),
                BalancerNodeRole::Distribute => (lane_outputs[node.lane as usize], center),
            };
            let direction = preferred_cardinal_direction(flow_from, flow_to);
            let (template_width, template_height) = rotated_template_dimensions(
                template.width as i32,
                template.height as i32,
                direction,
            );
            let block = nearest_free_block(
                CompactBlock {
                    id: node.id,
                    x: center.0 - template_width / 2,
                    y: center.1 - template_height / 2,
                    width: template_width,
                    height: template_height,
                },
                &reserved,
                clearance,
                1024,
            )
            .ok_or_else(|| {
                format!(
                    "{} node {} ({},{}) has no collision-free placement",
                    graph.item, node.id, node.n, node.m
                )
            })?;
            reserved.push(block.clone());
            let (mut stamped, input_ports, output_ports) =
                stamp_rotated_balancer(template, block.x, block.y, direction, &graph.item);
            for entity in &mut stamped {
                entity.segment_id = Some(format!(
                    "compact-hub:{}:{}:{}",
                    graph.item, node.lane, node.id
                ));
            }
            entities.extend(stamped);
            placed_nodes.push(PlacedManifoldNode {
                node_id: node.id,
                origin: (block.x, block.y),
                direction,
                input_ports,
                output_ports,
            });
        }
        placed_nodes.sort_by_key(|node| node.node_id);
        let manifold_blocks = &reserved[reservation_start..];
        let min_x = manifold_blocks.iter().map(|block| block.x).min().unwrap();
        let min_y = manifold_blocks.iter().map(|block| block.y).min().unwrap();
        let max_x = manifold_blocks
            .iter()
            .map(|block| block.x + block.width)
            .max()
            .unwrap();
        let max_y = manifold_blocks
            .iter()
            .map(|block| block.y + block.height)
            .max()
            .unwrap();
        result.push(PlacedLocalManifold {
            item: graph.item.clone(),
            hub: CompactBlock {
                id: graph_idx,
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            },
            nodes: placed_nodes,
            lane_inputs,
            lane_outputs,
            entities,
        });
    }
    Ok(result)
}

fn preferred_cardinal_direction(from: (i32, i32), to: (i32, i32)) -> EntityDirection {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    if dx.abs() > dy.abs() {
        if dx >= 0 {
            EntityDirection::East
        } else {
            EntityDirection::West
        }
    } else if dy < 0 {
        EntityDirection::North
    } else {
        EntityDirection::South
    }
}

fn rotated_template_dimensions(width: i32, height: i32, direction: EntityDirection) -> (i32, i32) {
    if matches!(direction, EntityDirection::East | EntityDirection::West) {
        (height, width)
    } else {
        (width, height)
    }
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
    origin_x: i32,
    origin_y: i32,
    direction: EntityDirection,
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
        let (entity_width, entity_height) = entity_dims(&entity.name, entity.direction);
        let mut rotated_tiles = Vec::new();
        for x in entity.x..entity.x + entity_width {
            for y in entity.y..entity.y + entity_height {
                rotated_tiles.push(rotate_template_tile((x, y), width, height, direction));
            }
        }
        entity.x = origin_x + rotated_tiles.iter().map(|tile| tile.0).min().unwrap();
        entity.y = origin_y + rotated_tiles.iter().map(|tile| tile.1).min().unwrap();
        entity.direction = rotate_template_direction(entity.direction, direction);
    }
    let rotate_ports = |ports: &[(i32, i32)]| {
        ports
            .iter()
            .map(|&point| {
                let point = rotate_template_tile(point, width, height, direction);
                (origin_x + point.0, origin_y + point.1)
            })
            .collect()
    };
    (
        entities,
        rotate_ports(template.input_tiles),
        rotate_ports(template.output_tiles),
    )
}

fn partial_endpoint_point(
    endpoint: &ManifoldEndpoint,
    nodes: &[PlacedManifoldNode],
    lane_inputs: &[(i32, i32)],
    lane_outputs: &[(i32, i32)],
) -> Option<(i32, i32)> {
    match endpoint {
        ManifoldEndpoint::Terminal(terminal) => Some((terminal.x, terminal.y)),
        ManifoldEndpoint::NodeInput { node, port } => nodes
            .get(*node)
            .and_then(|placed| placed.input_ports.get(*port as usize))
            .copied(),
        ManifoldEndpoint::NodeOutput { node, port } => nodes
            .get(*node)
            .and_then(|placed| placed.output_ports.get(*port as usize))
            .copied(),
        ManifoldEndpoint::LaneInput(lane) => lane_inputs.get(*lane as usize).copied(),
        ManifoldEndpoint::LaneOutput(lane) => lane_outputs.get(*lane as usize).copied(),
    }
}

fn descendant_terminal_points(graph: &LocalManifoldGraph, node: usize) -> Vec<(i32, i32)> {
    let mut frontier: Vec<_> = graph
        .nodes
        .get(node)
        .into_iter()
        .flat_map(|placed| {
            (0..placed.m).map(move |port| ManifoldEndpoint::NodeOutput { node, port })
        })
        .collect();
    let mut visited = Vec::new();
    let mut terminals = Vec::new();
    while let Some(endpoint) = frontier.pop() {
        if visited.contains(&endpoint) {
            continue;
        }
        visited.push(endpoint.clone());
        for edge in graph.edges.iter().filter(|edge| edge.from == endpoint) {
            match &edge.to {
                ManifoldEndpoint::Terminal(terminal) => terminals.push((terminal.x, terminal.y)),
                ManifoldEndpoint::NodeInput { node, .. } => {
                    if let Some(next) = graph.nodes.get(*node) {
                        frontier.extend(
                            (0..next.m)
                                .map(|port| ManifoldEndpoint::NodeOutput { node: *node, port }),
                        );
                    }
                }
                _ => frontier.push(edge.to.clone()),
            }
        }
    }
    terminals
}

fn coordinate_median(points: &[(i32, i32)]) -> (i32, i32) {
    if points.is_empty() {
        return (0, 0);
    }
    let mut xs: Vec<_> = points.iter().map(|point| point.0).collect();
    let mut ys: Vec<_> = points.iter().map(|point| point.1).collect();
    xs.sort_unstable();
    ys.sort_unstable();
    (xs[xs.len() / 2], ys[ys.len() / 2])
}

fn nearest_free_block(
    preferred: CompactBlock,
    reserved: &[CompactBlock],
    clearance: i32,
    max_radius: i32,
) -> Option<CompactBlock> {
    for radius in 0..=max_radius {
        for dx in -radius..=radius {
            for dy in [-radius, radius] {
                let mut candidate = preferred.clone();
                candidate.x += dx;
                candidate.y += dy;
                if block_clears_all(&candidate, reserved, clearance) {
                    return Some(candidate);
                }
            }
        }
        for dy in (-radius + 1)..radius {
            for dx in [-radius, radius] {
                let mut candidate = preferred.clone();
                candidate.x += dx;
                candidate.y += dy;
                if block_clears_all(&candidate, reserved, clearance) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn block_clears_all(block: &CompactBlock, reserved: &[CompactBlock], clearance: i32) -> bool {
    let expanded = CompactBlock {
        id: block.id,
        x: block.x - clearance,
        y: block.y - clearance,
        width: block.width + clearance * 2,
        height: block.height + clearance * 2,
    };
    reserved
        .iter()
        .all(|other| !blocks_overlap(&expanded, other))
}

fn placed_hub_is_free(
    candidate: &CompactBlock,
    islands: &[RigidIsland],
    hubs: &[PlacedLocalManifold],
    clearance: i32,
) -> bool {
    let expanded = CompactBlock {
        id: candidate.id,
        x: candidate.x - clearance,
        y: candidate.y - clearance,
        width: candidate.width + clearance * 2,
        height: candidate.height + clearance * 2,
    };
    islands
        .iter()
        .all(|island| !blocks_overlap(&expanded, &island.block))
        && hubs.iter().all(|hub| !blocks_overlap(&expanded, &hub.hub))
}

fn hub_is_free(
    candidate: &CompactBlock,
    islands: &[RigidIsland],
    plans: &[LocalManifoldPlan],
    clearance: i32,
) -> bool {
    let expanded = CompactBlock {
        id: candidate.id,
        x: candidate.x - clearance,
        y: candidate.y - clearance,
        width: candidate.width + clearance * 2,
        height: candidate.height + clearance * 2,
    };
    islands
        .iter()
        .all(|island| !blocks_overlap(&expanded, &island.block))
        && plans
            .iter()
            .all(|plan| !blocks_overlap(&expanded, &plan.hub))
}

pub fn entity_dims(name: &str, direction: EntityDirection) -> (i32, i32) {
    let (mut width, mut height) =
        oriented_splitter_dims(name, direction).unwrap_or_else(|| entity_size(name));
    if matches!(direction, EntityDirection::East | EntityDirection::West)
        && width != height
        && oriented_splitter_dims(name, direction).is_none()
    {
        std::mem::swap(&mut width, &mut height);
    }
    (width as i32, height as i32)
}

/// Recover the rigid production islands used by RFC-057's placement search.
///
/// Belt routes and power are intentionally excluded. Their contacts are
/// recorded as relative terminals so they can be regenerated after an island
/// moves. Direct machine-to-machine inserters are retained inside an island
/// and create no route terminal.
pub fn extract_rigid_islands(layout: &LayoutResult) -> Vec<RigidIsland> {
    let machine_indices: Vec<usize> = layout
        .entities
        .iter()
        .enumerate()
        .filter_map(|(idx, entity)| entity.recipe.as_ref().map(|_| idx))
        .collect();
    let mut machine_ordinal = BTreeMap::new();
    let mut machine_at = BTreeMap::new();
    for (ordinal, &entity_idx) in machine_indices.iter().enumerate() {
        machine_ordinal.insert(entity_idx, ordinal);
        let entity = &layout.entities[entity_idx];
        let (width, height) = entity_dims(&entity.name, entity.direction);
        for x in entity.x..entity.x + width {
            for y in entity.y..entity.y + height {
                machine_at.insert((x, y), entity_idx);
            }
        }
    }

    let mut parent: Vec<usize> = (0..machine_indices.len()).collect();
    fn root(parent: &mut [usize], mut node: usize) -> usize {
        while parent[node] != node {
            parent[node] = parent[parent[node]];
            node = parent[node];
        }
        node
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = root(parent, a);
        let rb = root(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }

    struct InserterContact {
        entity_idx: usize,
        pickup: (i32, i32),
        drop: (i32, i32),
        pickup_machine: Option<usize>,
        drop_machine: Option<usize>,
    }
    let mut contacts = Vec::new();
    for (entity_idx, inserter) in layout.entities.iter().enumerate() {
        if !is_inserter(&inserter.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(inserter.direction);
        let reach = inserter_reach(&inserter.name);
        let pickup = (inserter.x - dx * reach, inserter.y - dy * reach);
        let drop = (inserter.x + dx * reach, inserter.y + dy * reach);
        let pickup_machine = machine_at.get(&pickup).copied();
        let drop_machine = machine_at.get(&drop).copied();
        if let (Some(a), Some(b)) = (pickup_machine, drop_machine) {
            union(&mut parent, machine_ordinal[&a], machine_ordinal[&b]);
        }
        if pickup_machine.is_some() || drop_machine.is_some() {
            contacts.push(InserterContact {
                entity_idx,
                pickup,
                drop,
                pickup_machine,
                drop_machine,
            });
        }
    }

    let mut belt_item_at = BTreeMap::new();
    for entity in &layout.entities {
        if !is_belt_entity(&entity.name) {
            continue;
        }
        let Some(item) = entity.carries.as_ref() else {
            continue;
        };
        let (width, height) = entity_dims(&entity.name, entity.direction);
        for x in entity.x..entity.x + width {
            for y in entity.y..entity.y + height {
                belt_item_at.insert((x, y), item.clone());
            }
        }
    }

    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (ordinal, &entity_idx) in machine_indices.iter().enumerate() {
        let group = root(&mut parent, ordinal);
        groups.entry(group).or_default().push(entity_idx);
    }

    let mut islands = Vec::new();
    for machine_entities in groups.into_values() {
        let mut entity_indices = machine_entities.clone();
        let mut terminals = Vec::new();
        for contact in &contacts {
            let belongs = contact
                .pickup_machine
                .into_iter()
                .chain(contact.drop_machine)
                .any(|idx| machine_entities.contains(&idx));
            if !belongs {
                continue;
            }
            entity_indices.push(contact.entity_idx);
            // Both ends on machines means direct insertion, already captured
            // by the rigid geometry.
            if contact.pickup_machine.is_some() && contact.drop_machine.is_some() {
                continue;
            }
            if contact.pickup_machine.is_some() {
                if let Some(item) = belt_item_at.get(&contact.drop) {
                    terminals.push((
                        item.clone(),
                        RouteTerminalKind::ProducerDrop,
                        contact.drop,
                        contact.entity_idx,
                    ));
                }
            } else if contact.drop_machine.is_some() {
                if let Some(item) = belt_item_at.get(&contact.pickup) {
                    terminals.push((
                        item.clone(),
                        RouteTerminalKind::ConsumerPickup,
                        contact.pickup,
                        contact.entity_idx,
                    ));
                }
            }
        }
        entity_indices.sort_unstable();
        entity_indices.dedup();

        let min_x = entity_indices
            .iter()
            .map(|&idx| layout.entities[idx].x)
            .min()
            .unwrap();
        let min_y = entity_indices
            .iter()
            .map(|&idx| layout.entities[idx].y)
            .min()
            .unwrap();
        let max_x = entity_indices
            .iter()
            .map(|&idx| {
                let entity = &layout.entities[idx];
                entity.x + entity_dims(&entity.name, entity.direction).0
            })
            .max()
            .unwrap();
        let max_y = entity_indices
            .iter()
            .map(|&idx| {
                let entity = &layout.entities[idx];
                entity.y + entity_dims(&entity.name, entity.direction).1
            })
            .max()
            .unwrap();
        let mut recipes: Vec<_> = machine_entities
            .iter()
            .filter_map(|&idx| layout.entities[idx].recipe.clone())
            .collect();
        recipes.sort();
        terminals.sort();
        let terminals = terminals
            .into_iter()
            .map(
                |(item, kind, point, inserter_entity_index)| IslandTerminal {
                    item,
                    kind,
                    dx: point.0 - min_x,
                    dy: point.1 - min_y,
                    inserter_entity_index,
                },
            )
            .collect();
        let id = islands.len();
        islands.push(RigidIsland {
            id,
            entity_indices,
            recipes,
            block: CompactBlock {
                id,
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            },
            terminals,
        });
    }
    islands
}

/// Compact rigid islands along one axis and carry their relative terminals
/// with them. This only computes a placement; it does not retain or reroute
/// the incumbent belts.
pub fn compact_island_axis(
    islands: &[RigidIsland],
    axis: CompactAxis,
    clearance: i32,
) -> Vec<RigidIsland> {
    let blocks: Vec<_> = islands.iter().map(|island| island.block.clone()).collect();
    let compacted = compact_axis(&blocks, axis, clearance);
    islands
        .iter()
        .zip(compacted)
        .map(|(island, block)| {
            let mut moved = island.clone();
            moved.block = block;
            moved
        })
        .collect()
}

/// Apply an island placement to its rigid entities. Replaceable logistics is
/// left untouched; callers must rip it up and reroute before validating the
/// result as a factory.
pub fn apply_island_placement(
    layout: &LayoutResult,
    source_islands: &[RigidIsland],
    placed_islands: &[RigidIsland],
) -> Result<LayoutResult, String> {
    if source_islands.len() != placed_islands.len() {
        return Err("source and placed island counts differ".into());
    }
    let mut result = layout.clone();
    for (source, placed) in source_islands.iter().zip(placed_islands) {
        if source.id != placed.id || source.entity_indices != placed.entity_indices {
            return Err(format!("island identity changed at {}", source.id));
        }
        let dx = placed.block.x - source.block.x;
        let dy = placed.block.y - source.block.y;
        for &entity_idx in &source.entity_indices {
            let Some(entity) = result.entities.get_mut(entity_idx) else {
                return Err(format!(
                    "island {} entity {entity_idx} is missing",
                    source.id
                ));
            };
            entity.x += dx;
            entity.y += dy;
        }
    }
    result.regions.clear();
    result.trace = None;
    Ok(result)
}

/// Recover belt nets and their machine/boundary terminals from a validated
/// layout. This is routing intent for rubber-band re-embedding, not part of
/// the production invariant: later phases may merge or split these nets while
/// preserving [`ProductionSignature`].
pub fn extract_route_nets(layout: &LayoutResult) -> Vec<RouteNet> {
    let mut nets: Vec<RouteNet> = Vec::new();
    let mut net_by_item: BTreeMap<String, usize> = BTreeMap::new();
    let mut entity_net: BTreeMap<usize, usize> = BTreeMap::new();

    for (entity_idx, entity) in layout.entities.iter().enumerate() {
        if !is_belt_entity(&entity.name) {
            continue;
        }
        let Some(segment) = entity.segment_id.clone() else {
            continue;
        };
        let Some(item) = entity.carries.clone() else {
            continue;
        };
        let net_idx = *net_by_item.entry(item.clone()).or_insert_with(|| {
            let idx = nets.len();
            nets.push(RouteNet {
                item,
                segments: Vec::new(),
                entity_indices: Vec::new(),
                terminals: Vec::new(),
            });
            idx
        });
        if !nets[net_idx].segments.contains(&segment) {
            nets[net_idx].segments.push(segment);
        }
        nets[net_idx].entity_indices.push(entity_idx);
        entity_net.insert(entity_idx, net_idx);
    }

    let mut machine_at: BTreeMap<(i32, i32), String> = BTreeMap::new();
    for entity in &layout.entities {
        let Some(recipe) = entity.recipe.as_ref() else {
            continue;
        };
        let (mut width, mut height) = entity_size(&entity.name);
        if matches!(
            entity.direction,
            EntityDirection::East | EntityDirection::West
        ) && width != height
        {
            std::mem::swap(&mut width, &mut height);
        }
        for x in entity.x..entity.x + width as i32 {
            for y in entity.y..entity.y + height as i32 {
                machine_at.insert((x, y), recipe.clone());
            }
        }
    }

    // Map every surface tile occupied by a belt-like entity to its net. A
    // splitter has a two-tile footprint and an inserter may address either
    // half.
    let mut belt_net_at: BTreeMap<(i32, i32), usize> = BTreeMap::new();
    for (&entity_idx, &net_idx) in &entity_net {
        let entity = &layout.entities[entity_idx];
        let (width, height) =
            oriented_splitter_dims(&entity.name, entity.direction).unwrap_or((1, 1));
        for x in entity.x..entity.x + width as i32 {
            for y in entity.y..entity.y + height as i32 {
                belt_net_at.insert((x, y), net_idx);
            }
        }
    }

    for inserter in layout.entities.iter().filter(|e| is_inserter(&e.name)) {
        let (dx, dy) = dir_to_vec(inserter.direction);
        let reach = inserter_reach(&inserter.name);
        let pickup = (inserter.x - dx * reach, inserter.y - dy * reach);
        let drop = (inserter.x + dx * reach, inserter.y + dy * reach);
        if let (Some(&net_idx), Some(recipe)) = (belt_net_at.get(&drop), machine_at.get(&pickup)) {
            nets[net_idx].terminals.push(RouteTerminal {
                kind: RouteTerminalKind::ProducerDrop,
                x: drop.0,
                y: drop.1,
                recipe: Some(recipe.clone()),
            });
        }
        if let (Some(&net_idx), Some(recipe)) = (belt_net_at.get(&pickup), machine_at.get(&drop)) {
            nets[net_idx].terminals.push(RouteTerminal {
                kind: RouteTerminalKind::ConsumerPickup,
                x: pickup.0,
                y: pickup.1,
                recipe: Some(recipe.clone()),
            });
        }
    }

    for (kind, boundaries) in [
        (RouteTerminalKind::BoundaryInput, &layout.boundary_inputs),
        (RouteTerminalKind::BoundaryOutput, &layout.boundary_outputs),
    ] {
        for boundary in boundaries {
            if let Some(&net_idx) = belt_net_at.get(&(boundary.x, boundary.y)) {
                nets[net_idx].terminals.push(RouteTerminal {
                    kind,
                    x: boundary.x,
                    y: boundary.y,
                    recipe: None,
                });
            }
        }
    }

    for net in &mut nets {
        net.entity_indices.sort_unstable();
        net.segments.sort();
        net.terminals.sort();
        net.terminals.dedup();
    }
    nets.sort_by(|a, b| a.item.cmp(&b.item));
    nets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemFlow, MachineSpec};

    #[test]
    fn production_signature_is_order_independent() {
        let plate = MachineSpec {
            entity: "electric-furnace".into(),
            recipe: "iron-plate".into(),
            count: 2.0,
            outputs: vec![ItemFlow {
                item: "iron-plate".into(),
                rate: 1.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let gear = MachineSpec {
            entity: "assembling-machine-3".into(),
            recipe: "iron-gear-wheel".into(),
            count: 3.0,
            inputs: vec![ItemFlow {
                item: "iron-plate".into(),
                rate: 2.0,
                ..Default::default()
            }],
            outputs: vec![ItemFlow {
                item: "iron-gear-wheel".into(),
                rate: 1.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let a = SolverResult {
            machines: vec![plate.clone(), gear.clone()],
            ..Default::default()
        };
        let b = SolverResult {
            machines: vec![gear, plate],
            ..Default::default()
        };
        assert_eq!(
            ProductionSignature::from_solver(&a).unwrap(),
            ProductionSignature::from_solver(&b).unwrap()
        );
    }

    #[test]
    fn x_compaction_is_exact_for_fixed_overlap_order() {
        let blocks = vec![
            CompactBlock {
                id: 0,
                x: 10,
                y: 0,
                width: 3,
                height: 3,
            },
            CompactBlock {
                id: 1,
                x: 30,
                y: 1,
                width: 4,
                height: 2,
            },
            CompactBlock {
                id: 2,
                x: 50,
                y: 8,
                width: 5,
                height: 2,
            },
        ];
        let compacted = compact_axis(&blocks, CompactAxis::X, 1);
        assert_eq!(compacted[0].x, 0);
        assert_eq!(compacted[1].x, 4);
        // Cross-axis-disjoint block has no ordering constraint.
        assert_eq!(compacted[2].x, 0);
        assert_eq!(occupied_bbox(&compacted), (8, 10));
        assert!(!blocks_overlap(&compacted[0], &compacted[1]));
    }

    #[test]
    fn placed_machine_signature_ignores_geometry_and_entity_order() {
        let machine = crate::models::PlacedEntity {
            name: "assembling-machine-3".into(),
            recipe: Some("iron-gear-wheel".into()),
            x: 10,
            y: 20,
            ..Default::default()
        };
        let belt = crate::models::PlacedEntity {
            name: "transport-belt".into(),
            x: 2,
            y: 3,
            ..Default::default()
        };
        let a = LayoutResult {
            entities: vec![machine.clone(), belt.clone()],
            ..Default::default()
        };
        let mut moved = machine;
        moved.x = -7;
        moved.y = 4;
        let b = LayoutResult {
            entities: vec![belt, moved],
            ..Default::default()
        };
        assert_eq!(
            PlacedMachineSignature::from_layout(&a),
            PlacedMachineSignature::from_layout(&b)
        );
    }

    #[test]
    fn empty_column_strip_preserves_entities_and_shortens_geometry() {
        let mut layout = LayoutResult {
            width: 6,
            height: 1,
            entities: vec![
                crate::models::PlacedEntity {
                    name: "express-underground-belt".into(),
                    x: 0,
                    direction: EntityDirection::East,
                    io_type: Some("input".into()),
                    ..Default::default()
                },
                crate::models::PlacedEntity {
                    name: "express-underground-belt".into(),
                    x: 5,
                    direction: EntityDirection::East,
                    io_type: Some("output".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let signature = PlacedMachineSignature::from_layout(&layout);
        let compacted = strip_empty_columns(&layout);
        assert_eq!(compacted.width, 2);
        assert_eq!(compacted.entities[0].x, 0);
        assert_eq!(compacted.entities[1].x, 1);
        assert_eq!(PlacedMachineSignature::from_layout(&compacted), signature);
        layout.entities.reverse();
        assert_eq!(strip_empty_columns(&layout).width, 2);
    }

    #[test]
    fn straight_belts_become_maximal_underground_hops() {
        let layout = LayoutResult {
            width: 10,
            height: 1,
            entities: (0..10)
                .map(|x| crate::models::PlacedEntity {
                    name: "express-transport-belt".into(),
                    x,
                    direction: EntityDirection::East,
                    carries: Some("plate".into()),
                    segment_id: Some("test".into()),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let compacted = undergroundify_straight_belts(&layout);
        assert_eq!(compacted.entities.len(), 2);
        assert_eq!(compacted.entities[0].name, "express-underground-belt");
        assert_eq!(compacted.entities[0].io_type.as_deref(), Some("input"));
        assert_eq!(compacted.entities[1].x, 9);
        assert_eq!(compacted.entities[1].io_type.as_deref(), Some("output"));
    }

    #[test]
    fn vertical_belts_and_empty_rows_compact_symmetrically() {
        let layout = LayoutResult {
            width: 1,
            height: 10,
            entities: (0..10)
                .map(|y| crate::models::PlacedEntity {
                    name: "express-transport-belt".into(),
                    y,
                    direction: EntityDirection::South,
                    carries: Some("plate".into()),
                    segment_id: Some("test".into()),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let underground = undergroundify_straight_belts(&layout);
        assert_eq!(underground.entities.len(), 2);
        assert_eq!(underground.entities[0].name, "express-underground-belt");
        assert_eq!(underground.entities[1].y, 9);
        let compacted = strip_empty_rows(&underground);
        assert_eq!(compacted.height, 2);
        assert!(compacted
            .entities
            .iter()
            .all(|entity| entity.name == "express-transport-belt"));
    }

    #[test]
    fn vertical_cut_coalesces_equivalent_seam_belts() {
        let belt = |x, segment: &str| crate::models::PlacedEntity {
            name: "transport-belt".into(),
            x,
            direction: EntityDirection::East,
            carries: Some("plate".into()),
            segment_id: Some(segment.into()),
            ..Default::default()
        };
        let layout = LayoutResult {
            width: 4,
            height: 1,
            entities: vec![belt(0, "row"), belt(1, "corridor"), belt(3, "tail")],
            ..Default::default()
        };
        let collapsed = collapse_vertical_cut(&layout, 1).unwrap();
        assert_eq!(collapsed.width, 3);
        assert_eq!(collapsed.entities.len(), 2);
        assert_eq!(collapsed.entities[0].x, 0);
        assert_eq!(collapsed.entities[1].x, 2);
    }

    #[test]
    fn horizontal_cut_coalesces_equivalent_seam_belts() {
        let belt = |y| crate::models::PlacedEntity {
            name: "transport-belt".into(),
            y,
            direction: EntityDirection::South,
            carries: Some("plate".into()),
            segment_id: Some("route".into()),
            ..Default::default()
        };
        let layout = LayoutResult {
            width: 1,
            height: 4,
            entities: vec![belt(0), belt(1), belt(3)],
            ..Default::default()
        };
        let collapsed = collapse_horizontal_cut(&layout, 1).unwrap();
        assert_eq!(collapsed.height, 3);
        assert_eq!(collapsed.entities.len(), 2);
        assert_eq!(collapsed.entities[0].y, 0);
        assert_eq!(collapsed.entities[1].y, 2);
    }

    #[test]
    fn bounded_balancer_hierarchies_cover_arbitrary_terminal_counts() {
        for count in 0..=129 {
            let mergers = merger_stages(count);
            let mut remaining = count;
            for stage in &mergers {
                assert_eq!(stage.m, 1);
                assert!((2..=LOCAL_BALANCER_FAN).contains(&stage.n));
                remaining = remaining - stage.copies * stage.n + stage.copies * stage.m;
            }
            assert_eq!(remaining, count.min(1));

            let distributors = distributor_stages(count);
            let leaves = distributors.iter().fold(1u32, |leaves, stage| {
                assert_eq!(stage.n, 1);
                assert!((2..=LOCAL_BALANCER_FAN).contains(&stage.m));
                leaves - stage.copies * stage.n + stage.copies * stage.m
            });
            assert_eq!(leaves, count.max(1));
            assert!(mergers
                .iter()
                .chain(distributors.iter())
                .all(|stage| crate::bus::balancer::shape_is_stampable(stage.n, stage.m)));
        }
    }

    #[test]
    fn legalized_manifold_routes_materialize_underground_jumps() {
        let terminal = |x| {
            ManifoldEndpoint::Terminal(ManifoldTerminal {
                kind: RouteTerminalKind::ProducerDrop,
                x,
                y: 0,
                island_id: None,
                inserter_entity_index: None,
            })
        };
        let routes = vec![LegalizedManifoldRoute {
            item: "iron-plate".into(),
            edge: ManifoldGraphEdge {
                from: terminal(0),
                to: terminal(6),
            },
            path: vec![(0, 0), (6, 0)],
            unresolved_tiles: Vec::new(),
        }];
        let entities = materialize_legalized_manifold_routes(&routes).unwrap();
        assert_eq!(entities.len(), 2);
        assert!(entities.iter().all(|entity| is_ug_belt(&entity.name)));
        assert_eq!(entities[0].io_type.as_deref(), Some("input"));
        assert_eq!(entities[1].io_type.as_deref(), Some("output"));
    }

    #[test]
    fn unresolved_manifold_routes_are_not_materialized() {
        let terminal = ManifoldEndpoint::Terminal(ManifoldTerminal {
            kind: RouteTerminalKind::ProducerDrop,
            x: 0,
            y: 0,
            island_id: None,
            inserter_entity_index: None,
        });
        let routes = vec![LegalizedManifoldRoute {
            item: "iron-plate".into(),
            edge: ManifoldGraphEdge {
                from: terminal.clone(),
                to: terminal,
            },
            path: vec![(0, 0), (1, 0)],
            unresolved_tiles: vec![(1, 0)],
        }];
        assert!(materialize_legalized_manifold_routes(&routes).is_err());
    }

    #[test]
    fn balancer_rotation_preserves_ports_and_footprint() {
        let template = crate::bus::balancer_library::balancer_templates()
            .get(&(2, 1))
            .unwrap();
        let (entities, mut inputs, outputs) =
            stamp_rotated_balancer(template, 10, 20, EntityDirection::East, "iron-plate");
        inputs.sort();
        assert_eq!(inputs, vec![(10, 20), (10, 21)]);
        assert_eq!(outputs, vec![(12, 20)]);
        for entity in entities {
            let (width, height) = entity_dims(&entity.name, entity.direction);
            assert!(entity.x >= 10 && entity.x + width <= 13);
            assert!(entity.y >= 20 && entity.y + height <= 22);
        }
    }

    #[test]
    fn route_net_extraction_finds_machine_endpoints() {
        let layout = LayoutResult {
            entities: vec![
                crate::models::PlacedEntity {
                    name: "assembling-machine-3".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("plate".into()),
                    ..Default::default()
                },
                crate::models::PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: 3,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
                crate::models::PlacedEntity {
                    name: "transport-belt".into(),
                    x: 1,
                    y: 4,
                    direction: EntityDirection::East,
                    carries: Some("plate".into()),
                    segment_id: Some("corr:plate".into()),
                    ..Default::default()
                },
            ],
            width: 3,
            height: 5,
            ..Default::default()
        };
        let nets = extract_route_nets(&layout);
        assert_eq!(nets.len(), 1);
        assert_eq!(nets[0].item, "plate");
        assert_eq!(nets[0].segments, vec!["corr:plate"]);
        assert_eq!(
            nets[0].terminals,
            vec![RouteTerminal {
                kind: RouteTerminalKind::ProducerDrop,
                x: 1,
                y: 4,
                recipe: Some("plate".into()),
            }]
        );
    }

    /// `merger_stages` predicts the merge tree; `build_local_manifold_graph`
    /// builds it. They must agree, because `all_mergers_stampable` and
    /// `producer_stages` are derived from the prediction and never
    /// cross-checked against the graph — so a divergence means the
    /// stampability guarantee is being checked against a tree nobody builds.
    /// They diverged at 6, 7 and 10 producers: ordinary counts, not edges.
    #[test]
    fn merger_stage_prediction_matches_the_built_graph() {
        for inputs in 2u32..40 {
            let predicted: u32 = merger_stages(inputs).iter().map(|s| s.copies).sum();

            // Mirror of the builder's frontier chunking: walk in chunks of
            // LOCAL_BALANCER_FAN,each chunk of 2+ becomes a node, a lone
            // leftover passes through.
            let mut frontier = inputs as usize;
            let mut built = 0u32;
            while frontier > 1 {
                let mut next = 0usize;
                let mut i = 0usize;
                while i < frontier {
                    let chunk = (frontier - i).min(LOCAL_BALANCER_FAN as usize);
                    if chunk > 1 {
                        built += 1;
                    }
                    next += 1;
                    i += chunk;
                }
                frontier = next;
            }
            assert_eq!(
                predicted, built,
                "inputs={inputs}: predicted {predicted} merge nodes, builder makes {built}"
            );
        }
    }

    #[test]
    fn rigid_islands_bind_direct_insertion_and_expose_belt_terminals() {
        let machine = |x, recipe: &str| crate::models::PlacedEntity {
            name: "assembling-machine-3".into(),
            x,
            y: 0,
            recipe: Some(recipe.into()),
            ..Default::default()
        };
        let layout = LayoutResult {
            entities: vec![
                machine(0, "producer"),
                machine(4, "consumer"),
                crate::models::PlacedEntity {
                    name: "inserter".into(),
                    x: 3,
                    y: 1,
                    direction: EntityDirection::East,
                    ..Default::default()
                },
                crate::models::PlacedEntity {
                    name: "inserter".into(),
                    x: 5,
                    y: 3,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
                crate::models::PlacedEntity {
                    name: "transport-belt".into(),
                    x: 5,
                    y: 4,
                    carries: Some("gear".into()),
                    segment_id: Some("gear".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let islands = extract_rigid_islands(&layout);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].recipes, vec!["consumer", "producer"]);
        assert_eq!(islands[0].entity_indices, vec![0, 1, 2, 3]);
        assert_eq!(
            islands[0].terminals,
            vec![IslandTerminal {
                item: "gear".into(),
                kind: RouteTerminalKind::ProducerDrop,
                dx: 5,
                dy: 4,
                inserter_entity_index: 3,
            }]
        );

        let mut placed = islands.clone();
        placed[0].block.x = 20;
        placed[0].block.y = 10;
        let moved = apply_island_placement(&layout, &islands, &placed).unwrap();
        assert_eq!((moved.entities[0].x, moved.entities[0].y), (20, 10));
        assert_eq!((moved.entities[3].x, moved.entities[3].y), (25, 13));
        // Replaceable belt routing is deliberately not translated.
        assert_eq!((moved.entities[4].x, moved.entities[4].y), (5, 4));
        assert_eq!(
            PlacedMachineSignature::from_layout(&layout),
            PlacedMachineSignature::from_layout(&moved),
        );

        let ir = CompactIr::from_layout(&layout);
        let manifolds = build_manifold_nets(&ir, &placed).unwrap();
        assert_eq!(manifolds.len(), 1);
        assert_eq!(manifolds[0].item, "gear");
        assert_eq!(manifolds[0].producers().count(), 1);
        assert_eq!(manifolds[0].consumers().count(), 0);
        assert_eq!(
            (manifolds[0].terminals[0].x, manifolds[0].terminals[0].y),
            (25, 14),
        );
    }
}

// ---------------------------------------------------------------------------
// RFC-057 Phase 2: snake-fold transform
// ---------------------------------------------------------------------------

/// Rotate an `EntityDirection` by 180° (East ↔ West, North ↔ South).
///
/// Alternate fold segments are rotated, not reflected. A reflection in X
/// alone is chirality-flipping: it swaps a splitter's left/right input and
/// output priorities and invalidates the Space Age fluid-box `mirror` flag,
/// neither of which the transform compensates for. Composing the X mirror
/// with a Y mirror gives a 180° rotation, which is a rigid motion — every
/// entity keeps its handedness. It is also the transform the routing
/// measurement prefers by a wide margin (RFC-057 decision log, 2026-07-29):
/// reflecting in X alone leaves each segment's trunk `height + gap` from the
/// next one's, where rotating brings them together.
fn rotate_180_direction(dir: EntityDirection) -> EntityDirection {
    match dir {
        EntityDirection::East => EntityDirection::West,
        EntityDirection::West => EntityDirection::East,
        EntityDirection::North => EntityDirection::South,
        EntityDirection::South => EntityDirection::North,
    }
}

/// Replace UG-belt pairs that straddle any fold boundary with surface belts.
///
/// A pair straddles fold `f` when one half is at `x < f` and the other at
/// `x >= f`.  Only East/West-facing pairs can straddle (South-facing pairs
/// are perpendicular to the fold axis).
fn replace_straddling_ug_pairs(entities: &[PlacedEntity], folds: &[i32]) -> Vec<PlacedEntity> {
    let ug_at: BTreeMap<(i32, i32), usize> = entities
        .iter()
        .enumerate()
        .filter(|(_, e)| is_ug_belt(&e.name))
        .map(|(i, e)| ((e.x, e.y), i))
        .collect();

    let mut to_surface: FxHashSet<usize> = FxHashSet::default();
    for (idx, entity) in entities.iter().enumerate() {
        if !is_ug_belt(&entity.name) || entity.io_type.as_deref() != Some("input") {
            continue;
        }
        let (dx, _dy) = dir_to_vec(entity.direction);
        if dx == 0 {
            continue;
        }
        let max_reach = ug_max_reach(ug_to_surface_tier(&entity.name)) as i32;
        // ug_max_reach returns tiles BETWEEN entrance and exit (exclusive),
        // so the max step from entrance to exit is max_reach + 1.
        for step in 1..=(max_reach + 1) {
            let look_x = entity.x + dx * step;
            if let Some(&out_idx) = ug_at.get(&(look_x, entity.y)) {
                let out = &entities[out_idx];
                if out.name == entity.name
                    && out.direction == entity.direction
                    && out.io_type.as_deref() == Some("output")
                {
                    for &f in folds {
                        if (entity.x < f && out.x >= f) || (out.x < f && entity.x >= f) {
                            to_surface.insert(idx);
                            to_surface.insert(out_idx);
                        }
                    }
                    break;
                }
            }
        }
    }

    let mut result: Vec<PlacedEntity> = entities
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            if to_surface.contains(&idx) {
                let mut s = e.clone();
                s.name = ug_to_surface_tier(&e.name).to_string();
                s.io_type = None;
                s
            } else {
                e.clone()
            }
        })
        .collect();

    // Fill in surface belts between replaced UG entrance/exit pairs.
    // The UG pair spanned from entrance to exit with items going
    // underground.  After replacement, we need surface belts at every
    // tile between them to carry items across.
    let existing: FxHashSet<(i32, i32)> = result.iter().map(|e| (e.x, e.y)).collect();
    for (idx, entity) in entities.iter().enumerate() {
        if !to_surface.contains(&idx) || entity.io_type.as_deref() != Some("input") {
            continue;
        }
        let (dx, _dy) = dir_to_vec(entity.direction);
        if dx == 0 {
            continue;
        }
        let max_reach = ug_max_reach(ug_to_surface_tier(&entity.name)) as i32;
        // Find the matching output (already confirmed to exist above).
        for step in 1..=(max_reach + 1) {
            let look_x = entity.x + dx * step;
            if let Some(&out_idx) = ug_at.get(&(look_x, entity.y)) {
                let out = &entities[out_idx];
                if to_surface.contains(&out_idx) && out.io_type.as_deref() == Some("output") {
                    // Fill the WHOLE span, both sides of the fold. Filling
                    // only the entrance side left the exit tile an orphan
                    // with a gap before it — an unfed belt on one side and a
                    // dead end on the other. The fold's junction pass is what
                    // severs the run; it needs a continuous surface belt on
                    // each side to attach to, which means every tile between
                    // entrance and exit must exist first.
                    let surface = ug_to_surface_tier(&entity.name);
                    for fill_x in 1..step {
                        let fx = entity.x + dx * fill_x;
                        if !existing.contains(&(fx, entity.y)) {
                            result.push(PlacedEntity {
                                name: surface.to_string(),
                                x: fx,
                                y: entity.y,
                                direction: entity.direction,
                                carries: entity.carries.clone(),
                                ..Default::default()
                            });
                        }
                    }
                    break;
                }
            }
        }
    }

    result
}

/// One severed belt run that a fold junction has to reconnect.
struct UturnRequest {
    /// Tile the upstream belt deposits into — the connector's first tile.
    start: (i32, i32),
    from_dir: EntityDirection,
    /// Tile that feeds the downstream belt — the connector's last tile.
    end: (i32, i32),
    to_dir: EntityDirection,
    belt: String,
    carries: Option<String>,
}

/// Place a vertical U-turn chain at column `jx` joining two severed belt ends.
///
/// `start` is the first tile the connector must occupy — the tile the
/// upstream belt deposits into — and `end` is the last, the tile that feeds
/// the downstream belt. The two are independent: under the 180° segment
/// rotation the downstream end lands at a different column *and* a different
/// row than the upstream end, so a single shared `edge_x`/row (as an earlier
/// version assumed) is only correct when consecutive segments happen to be
/// the same width. When they are not, the connector either stops short in
/// empty space — a dead-end belt — or runs into the next segment's body and
/// sideloads into a belt carrying a different item.
///
/// `occupied` tracks anchor tiles so the connector can skip tiles that are
/// already taken without rescanning the entity list; a linear scan per tile
/// made an exhaustive fold search quadratic and unusably slow.
fn place_uturn(
    entities: &mut Vec<PlacedEntity>,
    occupied: &mut FxHashSet<(i32, i32)>,
    corners: &mut Vec<(i32, i32)>,
    jx: i32,
    request: &UturnRequest,
) -> Result<(), (i32, i32)> {
    let UturnRequest {
        start,
        from_dir,
        end,
        to_dir,
        belt: belt_name,
        carries,
    } = request;
    let (from_dir, to_dir) = (*from_dir, *to_dir);
    let belt_name = belt_name.as_str();
    let (start_x, y_top) = *start;
    let (end_x, y_bot) = *end;

    // Build the whole connector first, commit only if every tile it needs is
    // free. A partially stamped U-turn that discovers a conflict halfway is
    // worse than no U-turn: it leaves a belt run ending in mid air.
    let mut planned: Vec<PlacedEntity> = Vec::new();
    // belt_name is a surface belt name (e.g. "express-transport-belt").
    let surface_name = belt_name;
    let ug_name = match belt_name {
        "express-transport-belt" => "express-underground-belt",
        "fast-transport-belt" => "fast-underground-belt",
        "transport-belt" => "underground-belt",
        _ => "underground-belt",
    };
    let max_reach = ug_max_reach(surface_name) as i32;

    // Top leg: from the tile the upstream belt deposits into, out to the
    // junction column, travelling in the upstream belt's direction.
    let top_step = if jx >= start_x { 1 } else { -1 };
    let mut x = start_x;
    while x != jx {
        {
            planned.push(PlacedEntity {
                name: surface_name.to_string(),
                x,
                y: y_top,
                direction: from_dir,
                carries: carries.clone(),
                ..Default::default()
            });
        }
        x += top_step;
    }

    // The vertical run may go either way: a belt crossing the fold westward
    // travels from the LOWER segment back up to the upper one.
    let (vdir, vstep) = if y_bot >= y_top {
        (EntityDirection::South, 1)
    } else {
        (EntityDirection::North, -1)
    };

    // Corner at the top of the run — receives from the horizontal leg.
    planned.push(PlacedEntity {
        name: surface_name.to_string(),
        x: jx,
        y: y_top,
        direction: vdir,
        carries: carries.clone(),
        ..Default::default()
    });

    // Vertical chain between the two corners, undergrounding where it can.
    let mut cy = y_top + vstep;
    while cy != y_bot {
        let remaining = (y_bot - cy).abs();
        // `remaining == 2` would put the entrance and exit on adjacent tiles:
        // nothing travels underground, and the validator reads it as two
        // unpaired halves plus a one-tile belt loop. Needs a tile between.
        if remaining >= 3 && remaining - 1 <= max_reach {
            planned.push(PlacedEntity {
                name: ug_name.to_string(),
                x: jx,
                y: cy,
                direction: vdir,
                io_type: Some("input".into()),
                carries: carries.clone(),
                ..Default::default()
            });
            planned.push(PlacedEntity {
                name: ug_name.to_string(),
                x: jx,
                y: cy + vstep * (remaining - 1),
                direction: vdir,
                io_type: Some("output".into()),
                carries: carries.clone(),
                ..Default::default()
            });
            cy += vstep * remaining;
        } else if remaining > max_reach + 1 {
            planned.push(PlacedEntity {
                name: ug_name.to_string(),
                x: jx,
                y: cy,
                direction: vdir,
                io_type: Some("input".into()),
                carries: carries.clone(),
                ..Default::default()
            });
            planned.push(PlacedEntity {
                name: ug_name.to_string(),
                x: jx,
                y: cy + vstep * max_reach,
                direction: vdir,
                io_type: Some("output".into()),
                carries: carries.clone(),
                ..Default::default()
            });
            cy += vstep * (max_reach + 1);
            planned.push(PlacedEntity {
                name: surface_name.to_string(),
                x: jx,
                y: cy,
                direction: vdir,
                carries: carries.clone(),
                ..Default::default()
            });
            cy += vstep;
        } else {
            planned.push(PlacedEntity {
                name: surface_name.to_string(),
                x: jx,
                y: cy,
                direction: vdir,
                carries: carries.clone(),
                ..Default::default()
            });
            cy += vstep;
        }
    }

    // Belt at (jx, y_bot) going to_dir — receives from South chain.
    planned.push(PlacedEntity {
        name: surface_name.to_string(),
        x: jx,
        y: y_bot,
        direction: to_dir,
        carries: carries.clone(),
        ..Default::default()
    });

    // Bottom leg: from the junction column back to the tile that feeds the
    // downstream belt, inclusive, travelling in the downstream direction.
    //
    // Expressed as a bounded range on purpose. A step-and-test loop here had
    // no terminating case when `end_x == jx` — the corner already feeds the
    // downstream belt, so there is no leg — and ran away allocating until the
    // OOM killer took the process (and, being a global OOM, whatever else was
    // running alongside it).
    if end_x != jx {
        let (lo, hi) = if end_x > jx {
            (jx + 1, end_x)
        } else {
            (end_x, jx - 1)
        };
        for x in lo..=hi {
            {
                planned.push(PlacedEntity {
                    name: surface_name.to_string(),
                    x,
                    y: y_bot,
                    direction: to_dir,
                    carries: carries.clone(),
                    ..Default::default()
                });
            }
        }
    }

    // Commit only if the connector is entirely clear. Reserving every tile —
    // corners and vertical run included, not just the horizontal legs —
    // is what makes a later chain's leg unable to stamp over this run.
    if let Some(clash) = planned
        .iter()
        .find(|e| occupied.contains(&(e.x, e.y)))
        .map(|e| (e.x, e.y))
    {
        return Err(clash);
    }
    for e in &planned {
        occupied.insert((e.x, e.y));
    }
    corners.push((jx, y_top));
    corners.push((jx, y_bot));
    entities.extend(planned);
    Ok(())
}

/// A fold that validated no worse than the layout it came from.
pub struct FoldOutcome {
    pub folds: Vec<i32>,
    pub layout: LayoutResult,
}

/// Outcome of a fold search, including why candidates were turned away.
///
/// The refusal mix is the useful half when nothing is found: "no fold" is
/// not actionable, whereas "every candidate stranded an input" or "there were
/// no legal columns to begin with" says exactly which constraint is binding.
pub struct FoldSearch {
    pub best: Option<FoldOutcome>,
    pub legal_columns: usize,
    /// Candidates the folder refused, by cause.
    pub refusals: Vec<(Vec<i32>, FoldRefusal)>,
    /// Candidates that folded but validated worse than the source.
    pub rejected_by_validation: usize,
    /// Which categories those candidates regressed on, and how many candidates
    /// regressed on each.
    ///
    /// The bare count above says how many candidates were turned away and
    /// nothing about why, so a geometry fix that converts refusals into
    /// validation rejections looks like progress in one column and is
    /// invisible in the other. That is the reporting shape documented in
    /// `docs/validator-reporting.md`, and it cost real time here: the
    /// side-partition fix took `GapLaneConflict` 32 -> 0 while pushing
    /// `rejected_by_validation` 22 -> 54, and the counter could not say which
    /// check the 32 newly-buildable layouts were failing.
    pub validation_regressions: BTreeMap<String, usize>,
}

/// Search for the squarest snake fold that does not break the factory.
///
/// Geometric legality is necessary but nowhere near sufficient: a fold column
/// can be perfectly cuttable and still sever a belt run whose reconnection
/// the junction pass does not see, and the symptom then shows up as a starved
/// machine somewhere else entirely. Rather than try to make every column
/// work, this admits a candidate only when it validates no worse than the
/// source — same issue categories, same counts. Anything that introduces a
/// new warning is rejected, whatever its geometry.
///
/// The comparison is against the source's own issue profile, not against
/// zero, because the source is itself allowed to carry known warnings.
pub fn search_snake_fold(
    layout: &LayoutResult,
    solver: &SolverResult,
    max_folds: usize,
) -> FoldSearch {
    use crate::validate::{self, LayoutStyle, ValidationIssue};
    use crate::verdict::{self, GatePolicy, MatchTier, Policy};

    let validate_issues =
        |l: &LayoutResult| -> Option<Vec<ValidationIssue>> { validate::validate(l, Some(solver), LayoutStyle::Bus).ok() };

    let mut out = FoldSearch {
        best: None,
        legal_columns: 0,
        refusals: Vec::new(),
        rejected_by_validation: 0,
        validation_regressions: BTreeMap::new(),
    };
    let Some(native_issues) = validate_issues(layout) else {
        return out;
    };

    let legal = legal_fold_columns(layout);
    out.legal_columns = legal.len();
    if legal.is_empty() {
        return out;
    }
    let snap = |target: i32| -> Option<i32> {
        legal.iter().copied().min_by_key(|&f| (f - target).abs())
    };

    // Gated at instance level, matched through a per-candidate
    // `fold_point_correspondence` map. The fold's own geometry (a plain
    // translation per even segment, a 180-degree point reflection per odd
    // one — see `fold_point_correspondence`'s docs) is exact and
    // closed-form, so the map is not an approximation the way it would be
    // for a transform this module can't fully characterize.
    //
    // This is STRICTER than the count-diff comparison this function used
    // before it (P2a, RFC-064 decision log — pre-approved by the project
    // owner): intra-category churn, where issues resolved on one row net
    // against new ones introduced on another, now rejects instead of
    // passing. See P2a's PR report for what that flips on the existing
    // corpus.
    let policy = Policy::new(GatePolicy::GateInstances);

    let mut best: Option<(i64, Vec<i32>, LayoutResult)> = None;

    for k in 1..=max_folds {
        // Slide the whole comb of fold lines, snapping each tooth to the
        // nearest legal column. Cheap, and it explores the seams that matter
        // without a combinatorial blow-up over independent columns.
        for delta in -24..=24 {
            let mut folds: Vec<i32> = Vec::with_capacity(k);
            for i in 1..=k {
                let target = layout.width * i as i32 / (k + 1) as i32 + delta;
                let Some(f) = snap(target) else { continue };
                folds.push(f);
            }
            folds.dedup();
            if folds.len() != k {
                continue;
            }
            let mut bounds = vec![0];
            bounds.extend_from_slice(&folds);
            bounds.push(layout.width);
            if bounds.windows(2).any(|b| b[1] - b[0] < 24) {
                continue;
            }

            let folded = match fold_snake(layout, &folds) {
                Ok(f) => f,
                Err(reason) => {
                    out.refusals.push((folds.clone(), reason));
                    continue;
                }
            };
            let Some(candidate_issues) = validate_issues(&folded) else {
                continue;
            };
            let correspondence = fold_point_correspondence(layout, &folds);
            let verdict = verdict::never_worse(
                &native_issues,
                &candidate_issues,
                &policy,
                MatchTier::Provenance,
                Some(&correspondence),
            );
            if !verdict.pass {
                out.rejected_by_validation += 1;
                for cat in verdict.regressed_categories() {
                    *out.validation_regressions.entry(cat.to_string()).or_default() += 1;
                }
                continue;
            }

            // Prefer square, then small. Aspect in tenths keeps the ordering
            // integral and therefore deterministic.
            let (w, h) = (folded.width.max(1) as i64, folded.height.max(1) as i64);
            let aspect10 = (w.max(h) * 10) / w.min(h);
            let score = aspect10 * 1_000_000 + w * h;
            if best.as_ref().is_none_or(|(bs, _, _)| score < *bs) {
                best = Some((score, folds, folded));
            }
        }
    }

    out.best = best.map(|(_, folds, layout)| FoldOutcome { folds, layout });
    out
}

/// Columns a fold may legally cut, i.e. those that pass between entities
/// rather than through one.
///
/// Multi-fold candidates picked at arithmetic fractions of the width are
/// almost always rejected: on a dense layout most columns land inside some
/// machine's footprint. Choosing fold lines from the entity-free seams
/// instead turns "k evenly spaced folds" from a near-certain refusal into a
/// short search.
/// A fold may only cut things the junction pass can put back — that is,
/// surface belt runs. Everything else it severs stays severed, and the
/// symptom is not a geometry error but a starved machine several tiles away
/// ("items can't reach input", "delivers 0.0/s").
///
/// So a column is blocked when it would split:
/// - a multi-tile entity's footprint;
/// - an inserter's pickup→drop span, which carries items across the column
///   with no belt to reconnect;
/// - a splitter's input or output adjacency — a splitter is not a surface
///   belt, so a run entering or leaving one is invisible to the crossing
///   detector;
/// - a pipe-to-pipe adjacency, since fluid networks are connected by contact
///   and a cut silently isolates a segment.
pub fn legal_fold_columns(layout: &LayoutResult) -> Vec<i32> {
    let mut blocked = vec![false; (layout.width.max(0) + 2) as usize];
    let mut block_span = |lo: i32, hi: i32| {
        // A column f splits [lo, hi] when lo < f <= hi.
        for f in (lo + 1)..=hi {
            if f > 0 && f < layout.width {
                blocked[f as usize] = true;
            }
        }
    };

    let is_pipe = |name: &str| name.starts_with("pipe");
    let pipe_at: FxHashSet<(i32, i32)> = layout
        .entities
        .iter()
        .filter(|e| is_pipe(&e.name))
        .map(|e| (e.x, e.y))
        .collect();

    for entity in &layout.entities {
        let (w, _) = entity_dims(&entity.name, entity.direction);
        block_span(entity.x, entity.x + w - 1);

        if is_inserter(&entity.name) {
            let (dx, _) = dir_to_vec(entity.direction);
            if dx != 0 {
                let reach = inserter_reach(&entity.name);
                let a = entity.x - dx * reach;
                let b = entity.x + dx * reach;
                block_span(a.min(b), a.max(b));
            }
        }

        if crate::common::is_splitter(&entity.name) {
            // Whichever way it faces, its feed and its output sit on the
            // columns either side of its own footprint.
            block_span(entity.x - 1, entity.x + w);
        }

        if is_pipe(&entity.name) && pipe_at.contains(&(entity.x + 1, entity.y)) {
            block_span(entity.x, entity.x + 1);
        }
    }

    (1..layout.width)
        .filter(|&f| !blocked[f as usize])
        .collect()
}

/// Pick `k` legal fold columns as near as possible to evenly spaced targets,
/// keeping them strictly increasing and at least `min_seg` apart.
pub fn even_legal_folds(layout: &LayoutResult, k: usize, min_seg: i32) -> Option<Vec<i32>> {
    let legal = legal_fold_columns(layout);
    if legal.is_empty() {
        return None;
    }
    let mut chosen: Vec<i32> = Vec::with_capacity(k);
    for i in 1..=k {
        let target = layout.width * i as i32 / (k + 1) as i32;
        let lower = chosen.last().map(|f| f + min_seg).unwrap_or(min_seg);
        let pick = legal
            .iter()
            .copied()
            .filter(|&f| f >= lower && layout.width - f >= min_seg)
            .min_by_key(|&f| (f - target).abs())?;
        chosen.push(pick);
    }
    Some(chosen)
}

/// Why a fold could not be produced. Every refusal is a distinct physical
/// cause, and lumping them into a bare `None` made the common ones invisible:
/// a fold rejected for cutting a machine and one rejected for an unroutable
/// junction need completely different responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldRefusal {
    /// Fold columns were empty, out of order, or outside the layout.
    BadFoldColumns,
    /// A fold column passes through a multi-tile entity's interior.
    CutsEntity,
    /// A severed belt run's reconnection collided with geometry already
    /// placed, so no U-turn could be routed for it. Carries the first
    /// conflicting tile — without it the cause is indistinguishable from any
    /// other refusal and the fix is guesswork.
    JunctionBlocked { at: (i32, i32) },
    /// Two gap lanes would have to share a tile. Carries the tile.
    ///
    /// Covers exits and input feeds alike — both are lanes in the same gap.
    /// It was `GapLaneConflict`, which the input feed pass reused, so an
    /// input-side collision reported a name that sent the reader to the exit
    /// pass.
    GapLaneConflict { at: (i32, i32) },
    /// A U-turn corner gained a second feeder, demoting a both-lane turn to a
    /// single-lane sideload (`docs/factorio-mechanics.md` B8 vs B11).
    CornerNotATurn,
    /// Reconnection produced implausibly many entities — a runaway backstop.
    EntityExplosion,
    /// A belt run that continued in the source now dead-ends. Carries the
    /// source tile whose hand-off was lost.
    RunSevered { at: (i32, i32) },
    /// A boundary input landed on an interior row, where nothing outside the
    /// factory can feed it.
    InputStranded { at: (i32, i32) },
}

/// Reconnect a folded layout's power network.
///
/// This used to strip every pole and re-place from scratch, on the reasoning
/// that poles depend on final geometry. That was wrong: a fold is a RIGID
/// MOTION per segment, so poles inside a segment keep their relative
/// positions, and a connected source network stays connected within each
/// segment — only the seams between segments break. Re-placing discarded a
/// good network and rebuilt a worse one with `place_poles`, which is shaped
/// for row layouts rather than folded geometry. On `chain-mil5ore` that turned
/// a fully connected source into 89 unreachable poles.
///
/// So: keep what the transform produced, and bridge only what the seams
/// severed.
fn replace_poles(layout: &LayoutResult) -> LayoutResult {
    let mut out = layout.clone();
    // Coverage first, then connectivity — in that order, deliberately.
    //
    // A fold breaks both, differently: rotation is rigid so coverage survives
    // INSIDE a segment, but an entity next to a fold column loses the pole that
    // covered it from across the seam, and the two segments' pole networks are
    // no longer wired to each other. Topping up coverage before repairing
    // connectivity means the new poles are wire nodes the repair can use,
    // rather than fresh islands it has to bridge on a second pass.
    crate::bus::layout::cover_unpowered(&mut out);
    crate::bus::layout::repair_pole_network(&mut out);
    out
}

/// Segment boundaries and gap height for a fold at `folds` — the exact
/// geometry `fold_snake` itself places every entity against, extracted so
/// [`fold_point_correspondence`] can reproduce `fold_snake`'s per-tile
/// mapping without re-running the whole transform. See `fold_snake`'s body
/// (step 3, the `transform` closure) for how `bounds`/`gap` are consumed;
/// this function only computes them.
///
/// Assumes `folds` already passed `fold_snake`'s own validity checks
/// (non-empty, strictly increasing, in-bounds, no cut entities) — it is
/// only ever called from `fold_snake` itself (after those checks) or from
/// `search_snake_fold` (after a successful `fold_snake` call on the exact
/// same `folds`), never independently.
fn fold_bounds_and_gap(layout: &LayoutResult, folds: &[i32]) -> (Vec<i32>, i32) {
    let h = layout.height;
    let mut bounds = vec![0];
    bounds.extend_from_slice(folds);
    bounds.push(layout.width);
    let n_segs = bounds.len() - 1;

    // Gap height is sized from lane demand, not fixed at 2.
    //
    // A gap carries the belts that cross between the segments either side of
    // it, and two DIFFERENT items cannot share a lane — merging them onto one
    // belt is the corruption this pass exists to avoid. So the gap needs one
    // row per distinct item, and a fixed two-row gap silently capped every
    // layout at one item per side.
    //
    // Segment parity decides what each gap carries. An exit is a bottom-edge
    // belt in the source, which an unrotated segment keeps at its bottom (into
    // the gap below) and a rotated one carries to its top (into the gap
    // above); an input is a top-edge belt, mirrored. Consecutive segments
    // alternate, so a gap carries either two exit sets or two input sets,
    // never a mix.
    //
    // One height for every gap rather than a per-gap vector: it wastes a few
    // rows on the lighter gaps and keeps the segment offset a plain multiple
    // of `h + gap`, which the transform, junction and boundary passes all
    // depend on.
    let edge_items = |seg: usize, row: i32, dir: EntityDirection| -> std::collections::BTreeSet<String> {
        let (lo, hi) = (bounds[seg], bounds[seg + 1]);
        layout
            .entities
            .iter()
            .filter(|e| {
                e.y == row
                    && e.x >= lo
                    && e.x < hi
                    && e.direction == dir
                    && is_surface_belt(&e.name)
            })
            .filter_map(|e| e.carries.clone())
            .collect()
    };
    let gap = {
        let mut widest = 2;
        for k in 0..n_segs.saturating_sub(1) {
            // Even gaps take both neighbours' exits (source bottom row,
            // facing South); odd gaps take both neighbours' inputs (source top
            // row, facing South into the factory).
            let (row, dir) = if k % 2 == 0 {
                (h - 1, EntityDirection::South)
            } else {
                (0, EntityDirection::South)
            };
            // Exits and inputs need different counts, and the difference is
            // load-bearing for the sim-verified single fold.
            //
            // The exit pass keys lanes by ITEM, so both neighbours' copies of
            // one item share a lane: merged distinct count.
            // The input pass keys by (item, SIDE) — an item both neighbours
            // consume needs one lane per side, because they sit in different
            // row blocks — so it needs the SUM, not the union.
            //
            // Sizing per parity rather than taking the sum everywhere keeps a
            // single fold (which has only the even/exit gap) byte-identical,
            // and with it the registry pin measured at 5.00/s in Factorio.
            let need = if k % 2 == 0 {
                let mut items = edge_items(k, row, dir);
                items.extend(edge_items(k + 1, row, dir));
                items.len() as i32 + 1
            } else {
                edge_items(k, row, dir).len() as i32 + edge_items(k + 1, row, dir).len() as i32 + 1
            };
            widest = widest.max(need);
        }
        widest
    };
    (bounds, gap)
}

/// Per-tile correspondence for a fold at `folds`: maps every position in
/// `layout`'s (pre-fold) frame to its position in `fold_snake(layout,
/// folds)`'s output, for [`crate::verdict::never_worse`]'s Provenance tier.
///
/// This is a closed-form point transform, not a per-entity lookup — even
/// though `fold_snake`'s own `transform` closure computes a per-ENTITY
/// mapping (anchor position + `(w, h)` -> new anchor). The two coincide
/// exactly: for an entity at `e.x` with width `w`, a point at offset `dx`
/// from that anchor (`0 <= dx < w`) has original x-coordinate `e.x + dx`,
/// and a 180-degree flip within the entity's own footprint (odd segments)
/// puts it at new offset `w - 1 - dx` from the entity's new anchor.
/// Substituting `fold_snake`'s anchor formula (`new_x = bounds[seg+1] - w -
/// e.x`) gives `new_x + (w - 1 - dx) = bounds[seg+1] - 1 - (e.x + dx)` — the
/// `w` cancels. So the point-level map for ANY tile in an odd segment is
/// `bounds[seg+1] - 1 - x` on the column axis (symmetrically `h - 1 - y` on
/// the row axis), independent of what entity — if any — occupies it; even
/// segments are a plain translation. That means this function needs none of
/// `fold_snake`'s per-entity bookkeeping, only the same `bounds`/`gap`
/// `fold_snake` itself computes (via the shared [`fold_bounds_and_gap`]),
/// and it can answer for a bare coordinate no entity's anchor sits on — an
/// inserter's reach tile, or the far corner of a multi-tile machine.
///
/// Covers exactly `[0, layout.width) x [0, layout.height)`; a lookup miss
/// (e.g. an inserter reach tile one step outside that box, which does
/// happen at layout edges) degrades `never_worse`'s Provenance tier to a
/// count comparison for that issue's category — see that function's docs.
///
/// Does not itself validate `folds` — call only after a successful
/// `fold_snake(layout, folds)`, whose own checks (`BadFoldColumns`,
/// `CutsEntity`) this deliberately does not repeat.
pub fn fold_point_correspondence(layout: &LayoutResult, folds: &[i32]) -> CorrespondenceMap {
    let (bounds, gap) = fold_bounds_and_gap(layout, folds);
    let h = layout.height;
    let n_segs = bounds.len() - 1;

    let mut pairs: Vec<((i32, i32), (i32, i32))> = Vec::new();
    for x in 0..layout.width {
        let Some(seg) = (0..n_segs).find(|&k| x >= bounds[k] && x < bounds[k + 1]) else {
            continue;
        };
        for y in 0..h {
            let mapped = if seg % 2 == 0 {
                (x - bounds[seg], y + seg as i32 * (h + gap))
            } else {
                (bounds[seg + 1] - 1 - x, (h - 1 - y) + seg as i32 * (h + gap))
            };
            pairs.push(((x, y), mapped));
        }
    }
    CorrespondenceMap::from_pairs(pairs)
}

/// Snake-fold a layout at the given fold columns.
///
/// The layout is divided into `folds.len() + 1` vertical segments.  Even
/// segments keep their X orientation; odd segments are mirrored.  At each
/// fold junction, belts that were cut are reconnected with vertical
/// U-turns in the empty junction column.
///
/// Returns `None` if any fold column cuts through a multi-tile entity
/// interior or the folds are out of order.
pub fn fold_snake(
    layout: &LayoutResult,
    folds: &[i32],
) -> Result<LayoutResult, FoldRefusal> {
    if folds.is_empty() {
        return Ok(layout.clone());
    }
    for w in folds.windows(2) {
        if w[0] >= w[1] {
            return Err(FoldRefusal::BadFoldColumns);
        }
    }
    for &f in folds {
        if f <= 0 || f >= layout.width {
            return Err(FoldRefusal::BadFoldColumns);
        }
    }
    for &f in folds {
        for entity in &layout.entities {
            let (w, _) = entity_dims(&entity.name, entity.direction);
            if entity.x < f && entity.x + w > f {
                return Err(FoldRefusal::CutsEntity);
            }
        }
    }

    let h = layout.height;
    let (bounds, gap) = fold_bounds_and_gap(layout, folds);
    let n_segs = bounds.len() - 1;

    // 1. Replace straddling UG pairs with surface belts.
    let entities = replace_straddling_ug_pairs(&layout.entities, folds);

    // 2. Left-align all segments to x=0.  This prevents X-overlap between
    // segments at different Y levels (the main multi-fold failure mode).
    // The junction columns go OUTSIDE the segment range.
    let max_seg_w = (0..n_segs)
        .map(|k| bounds[k + 1] - bounds[k])
        .max()
        .unwrap_or(0);

    // 3. Transform every entity. Odd segments are rotated 180°, not mirrored
    // in X — see `rotate_180_direction`. Because a 180° rotation does not
    // change an entity's bounding box, the same `(w, h)` serves both axes.
    let seg_of = |x: i32| (0..n_segs).find(|&k| x >= bounds[k] && x < bounds[k + 1]);
    let transform = |e: &PlacedEntity| -> Option<(i32, i32, EntityDirection)> {
        let seg = seg_of(e.x)?;
        let (w, hh) = entity_dims(&e.name, e.direction);
        if seg % 2 == 0 {
            Some((
                e.x - bounds[seg],
                e.y + (seg as i32) * (h + gap),
                e.direction,
            ))
        } else {
            Some((
                bounds[seg + 1] - w - e.x,
                (h - hh - e.y) + (seg as i32) * (h + gap),
                rotate_180_direction(e.direction),
            ))
        }
    };

    let mut folded: Vec<PlacedEntity> = Vec::with_capacity(entities.len());
    for entity in &entities {
        let Some((new_x, new_y, new_dir)) = transform(entity) else {
            continue;
        };
        let mut e = entity.clone();
        e.x = new_x;
        e.y = new_y;
        e.direction = new_dir;
        folded.push(e);
    }

    // Anchor-tile occupancy, kept in step with `folded` so the reconnection
    // and exit passes can test a tile in O(1).
    let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &folded {
        let (w, hh) = entity_dims(&e.name, e.direction);
        for dx in 0..w.max(1) {
            for dy in 0..hh.max(1) {
                occupied.insert((e.x + dx, e.y + dy));
            }
        }
    }
    // Every U-turn corner tile, checked for lane integrity once all junction
    // and exit geometry is down.
    let mut corners: Vec<(i32, i32)> = Vec::new();

    // 4. Reconnect every belt run the folds severed.
    //
    // Crossings are identified in the ORIGINAL coordinate frame — a surface
    // belt at `f-1` flowing East into `f`, or at `f` flowing West into `f-1`
    // — and only then mapped through the transform. Scanning the *folded*
    // frame for belts at a segment edge (as an earlier version did) cannot
    // tell which severed end pairs with which, and silently assumed the two
    // ends kept the same row and column.
    //
    // Junction columns sit outside the segment range, staggered so chains do
    // not collide: right of the widest segment for even folds, left of zero
    // for odd ones.
    // Every belt entity, not just surface ones. A run crossing a fold may
    // have an underground half on either side — an exit feeding the fold
    // column, or an entrance receiving from it — and those are exactly as
    // severed as a surface belt. Matching only surface belts left them
    // unreconnected, which shows up not as a geometry error but as a starved
    // machine downstream ("items can't reach input").
    //
    // Straddling pairs are already surface belts by this point, so a
    // remaining underground half never spans the column itself.
    let belt_at: BTreeMap<(i32, i32), &PlacedEntity> = entities
        .iter()
        .filter(|e| is_belt_entity(&e.name))
        .map(|e| ((e.x, e.y), e))
        .collect();
    // An underground entrance swallows items; it cannot be a run's upstream
    // side. It is a perfectly good downstream side, since a straight feed
    // into an entrance carries both lanes.
    let feeds_forward =
        |e: &PlacedEntity| !(is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input"));

    for k in 0..n_segs - 1 {
        let f = bounds[k + 1];
        let mut crossings: Vec<UturnRequest> = Vec::new();
        for y in 0..h {
            // East crossing: (f-1) feeds (f). West crossing: (f) feeds (f-1).
            let (up, down) = match (belt_at.get(&(f - 1, y)), belt_at.get(&(f, y))) {
                (Some(left), Some(right))
                    if left.direction == EntityDirection::East && feeds_forward(left) =>
                {
                    (*left, *right)
                }
                (Some(left), Some(right))
                    if right.direction == EntityDirection::West && feeds_forward(right) =>
                {
                    (*right, *left)
                }
                _ => continue,
            };

            // The connector's legs are horizontal, and `place_uturn` stamps
            // every tile of the bottom leg with the DOWNSTREAM belt's
            // direction. If that belt runs vertically — a tap turning at the
            // fold column, which nothing above excludes — the leg becomes a
            // row of north/south belts that cannot carry items along it and
            // dump into whatever sits beside them. Refuse: reconnecting into
            // a turn needs a sideload this pass does not synthesize.
            if !matches!(
                down.direction,
                EntityDirection::East | EntityDirection::West
            ) {
                return Err(FoldRefusal::JunctionBlocked {
                    at: (down.x, down.y),
                });
            }
            let (Some((ux, uy, udir)), Some((dx_, dy_, ddir))) = (transform(up), transform(down))
            else {
                continue;
            };
            let (uvx, uvy) = dir_to_vec(udir);
            let (dvx, dvy) = dir_to_vec(ddir);
            // First tile of the connector is what the upstream belt feeds;
            // last tile is what feeds the downstream belt.
            let belt = if is_ug_belt(&up.name) {
                ug_to_surface_tier(&up.name).to_string()
            } else {
                up.name.clone()
            };
            crossings.push(UturnRequest {
                start: (ux + uvx, uy + uvy),
                from_dir: udir,
                end: (dx_ - dvx, dy_ - dvy),
                to_dir: ddir,
                belt,
                carries: up.carries.clone(),
            });
        }

        // Junction-column assignment order is what keeps chains from crossing
        // each other, and it is not free choice.
        //
        // Chain `j`'s horizontal legs sweep every column between the segment
        // edge and `jx_j`, so a leg runs over chain `i`'s vertical column
        // exactly when `jx_j` is further out than `jx_i` and one of `j`'s two
        // rows falls inside the row span `i`'s vertical run occupies. Ordering
        // the chains so their spans NEST — narrowest nearest the segment,
        // each subsequent span enclosing all the ones before it — makes that
        // unsatisfiable, because an enclosing span's endpoints are by
        // definition outside every span it contains.
        //
        // Sorting by landing row would do the same job if every connector ran
        // downward, but west-flowing crossings run *upward* (their upstream
        // end is in the lower segment), so row order mixes two orientations
        // and the nesting silently breaks. Span width is orientation-free.
        //
        // Getting this wrong is not a validation error at the junction; it is
        // a belt stamped on top of an underground run several tiles away.
        crossings.sort_by_key(|c| {
            let (lo, hi) = if c.start.1 <= c.end.1 {
                (c.start.1, c.end.1)
            } else {
                (c.end.1, c.start.1)
            };
            (hi - lo, lo)
        });

        // Slot 0 sits one column clear of the segment: a connector's own
        // start/end tiles land on the column immediately outside the segment
        // edge, so a junction column there would collide with them.
        let mut slot = 0i32;
        for request in &crossings {
            let mut clash = (0, 0);
            let mut placed = false;
            // Nesting should make the first slot fit. The retry covers spans
            // that genuinely cannot nest (mixed-orientation crossings at the
            // same junction) rather than refusing the whole fold for one.
            for attempt in 0..32 {
                let s = slot + attempt;
                let jx = if k % 2 == 0 {
                    max_seg_w + 1 + s * 2
                } else {
                    -2 - s * 2
                };
                match place_uturn(&mut folded, &mut occupied, &mut corners, jx, request) {
                    Ok(()) => {
                        slot = s + 1;
                        placed = true;
                        break;
                    }
                    Err(at) => clash = at,
                }
            }
            if !placed {
                return Err(FoldRefusal::JunctionBlocked { at: clash });
            }
        }
    }

    // Extent after all junction geometry is down — the exits below have to
    // reach these edges to actually leave the finished bounding box.
    let edge_min_x = folded.iter().map(|e| e.x).min().unwrap_or(0);
    let edge_max_x = folded.iter().map(|e| e.x).max().unwrap_or(0);

    // An exit that gets rerouted along a gap lane finishes somewhere new.
    // Its boundary record has to follow it: the record is where a consumer
    // (or the sim harness) expects to collect the output, and leaving it at
    // the old tile means the factory produces into a spot nothing drains —
    // every producer upstream then backs up and the whole line stalls.
    let mut exit_moved: BTreeMap<(i32, i32), ((i32, i32), EntityDirection)> = BTreeMap::new();
    // Same, for inputs the fold moved off the boundary.
    let mut input_moved: BTreeMap<(i32, i32), ((i32, i32), EntityDirection)> = BTreeMap::new();

    // 5. Carry the layout's edge belts across the gaps they now land in.
    //
    // A belt on the source layout's bottom edge facing South deposited
    // outside the bounding box — an exit. After folding it deposits into an
    // inter-segment gap instead, and a gap row is interior, so the exit
    // becomes a dead end. Rotation means each gap collects from both sides:
    // the segment above contributes South-facing belts on its last row, the
    // rotated segment below North-facing belts on its first.
    //
    // Each distinct ITEM gets its own lane. Sharing a lane between two items
    // merges them onto one belt, which is the corruption this pass exists to
    // prevent; previously that was refused, capping every gap at one item per
    // side.
    //
    // Lane assignment order is load-bearing. A lane runs from its exit column
    // to the bounding-box edge, so lane j occupies `[edge, x_j]`. An exit
    // whose lane is not the adjacent row must descend to it through the rows
    // above, at its own column. Assigning lanes in ASCENDING order of exit
    // column makes every such descent free: j < i implies x_j < x_i, so lane
    // j — which stops at x_j — never covers the column x_i descends through.
    // Any other order blocks descents (measured: descending order blocks 10
    // of 5 exits' descents, arbitrary 5).
    for k in 0..n_segs - 1 {
        let gap_top = (k as i32) * (h + gap) + h;

        // Which side a lane leaves by is forced, not a preference. Only fold
        // `k`'s own U-turns span gap `k`, and fold `k` puts its junction
        // columns on the right when `k` is even, so the opposite side is the
        // one guaranteed clear of vertical runs in these rows.
        let to_left = k % 2 == 0;
        let run_dir = if to_left {
            EntityDirection::West
        } else {
            EntityDirection::East
        };

        // Collect this gap's crossings from both sides, in the folded frame.
        let mut crossings: Vec<(i32, i32, EntityDirection, String, Option<String>)> = Vec::new();
        for (src_y, want_dir) in [
            (gap_top - 1, EntityDirection::South),
            (gap_top + gap, EntityDirection::North),
        ] {
            for e in folded.iter().filter(|e| {
                e.y == src_y && e.direction == want_dir && is_surface_belt(&e.name)
            }) {
                // A U-turn's vertical run passes through these rows at a
                // junction column outside the segment range; it is not an exit.
                if e.x < 0 || e.x >= max_seg_w {
                    continue;
                }
                crossings.push((e.x, src_y, want_dir, e.name.clone(), e.carries.clone()));
            }
        }
        if crossings.is_empty() {
            continue;
        }

        // One lane per item; lanes ordered by the leftmost column that item
        // must descend at, so the descent-freedom argument above holds.
        let mut by_item: BTreeMap<Option<String>, Vec<(i32, i32, EntityDirection, String)>> =
            BTreeMap::new();
        for (x, src_y, dir, name, carries) in crossings {
            by_item.entry(carries).or_default().push((x, src_y, dir, name));
        }
        let mut lanes: Vec<(Option<String>, Vec<(i32, i32, EntityDirection, String)>)> =
            by_item.into_iter().collect();
        lanes.sort_by_key(|(_, members)| members.iter().map(|m| m.0).min().unwrap_or(0));
        if lanes.len() as i32 > gap {
            return Err(FoldRefusal::GapLaneConflict { at: (-1, gap_top) });
        }

        for (lane_idx, (carries, members)) in lanes.into_iter().enumerate() {
            let row = gap_top + lane_idx as i32;
            let belt = members[0].3.clone();

            // Descend each member from its source row to (but NOT onto) this
            // lane: the lane row is owned by the horizontal run below, which
            // must face the run direction. Stamping the descent direction
            // there left a South-facing belt sitting on a West-flowing lane,
            // dead-ending immediately. For an adjacent lane the range is
            // empty and the source belt feeds the lane directly.
            for &(x, src_y, dir, ref name) in &members {
                let (from, to) = if src_y < row {
                    (src_y + 1, row - 1)
                } else {
                    (row + 1, src_y - 1)
                };
                for y in from..=to {
                    if !occupied.insert((x, y)) {
                        return Err(FoldRefusal::GapLaneConflict { at: (x, y) });
                    }
                    folded.push(PlacedEntity {
                        name: name.clone(),
                        x,
                        y,
                        direction: dir,
                        carries: carries.clone(),
                        ..Default::default()
                    });
                }
            }

            // Run the lane to the bounding-box edge. Members of the same lane
            // carry the same item, so overlapping spans merge legitimately.
            let far = members.iter().map(|m| m.0).max().unwrap_or(0);
            let near = members.iter().map(|m| m.0).min().unwrap_or(0);
            let (lo, hi) = if to_left {
                (edge_min_x, far)
            } else {
                (near, edge_max_x)
            };
            for cx in lo..=hi {
                if occupied.contains(&(cx, row)) {
                    // REFUSE, never skip. The previous version of this line
                    // was `continue` with the comment "already this lane's own
                    // belt" — which cannot happen: the descent loop above ends
                    // at `row - 1` (or starts at `row + 1`) precisely so a lane
                    // never occupies its own run row. Anything found here
                    // belongs to a DIFFERENT lane's descent.
                    //
                    // Skipping it leaves a hole in this run and sideloads this
                    // lane's items onto the crossing lane's belt — a cross-item
                    // merge, which is the exact failure per-item lanes exist to
                    // prevent — while `exit_moved` records a terminus that
                    // never receives anything. `main` refused here for that
                    // stated reason; the WIP lane work replaced it with a skip
                    // and an incorrect justification, and the input pass added
                    // later refuses in the mirror-image case, so the two halves
                    // disagreed. Found in review of #500.
                    return Err(FoldRefusal::GapLaneConflict { at: (cx, row) });
                }
                occupied.insert((cx, row));
                folded.push(PlacedEntity {
                    name: belt.clone(),
                    x: cx,
                    y: row,
                    direction: run_dir,
                    carries: carries.clone(),
                    ..Default::default()
                });
            }
            let term_x = if to_left { lo } else { hi };
            for &(x, src_y, _, _) in &members {
                exit_moved.insert((x, src_y), ((term_x, row), run_dir));
            }
        }
    }


    // 5b. Feed the inputs that folding moved off the bounding box.
    //
    // Mirror of the exit pass. A top-edge belt facing into the factory is fed
    // from OUTSIDE the box, so it only works on an edge. Segment parity puts
    // it there for a single fold — segment 0 keeps its inputs at the top,
    // segment 1 rotates them to the bottom — but from two folds up they land
    // on interior gap rows with nothing able to supply them, which is what
    // `InputStranded` refuses.
    //
    // Same structure as the exits, reversed: a lane runs from the bounding-box
    // edge to the input's column, then climbs to the tile that feeds it. Lane
    // order is by column again, for the same descent-freedom reason.
    for k in 0..n_segs - 1 {
        let gap_top = (k as i32) * (h + gap) + h;
        let to_left = k % 2 == 0;
        let run_dir = if to_left {
            EntityDirection::East
        } else {
            EntityDirection::West
        };

        // An input belt adjacent to this gap, facing away from it, is one the
        // gap must supply.
        let mut needs: Vec<(i32, i32, EntityDirection, String, Option<String>)> = Vec::new();
        for (src_y, want_dir) in [
            (gap_top - 1, EntityDirection::North),
            (gap_top + gap, EntityDirection::South),
        ] {
            for e in folded.iter().filter(|e| {
                e.y == src_y && e.direction == want_dir && is_surface_belt(&e.name)
            }) {
                if e.x < 0 || e.x >= max_seg_w {
                    continue;
                }
                needs.push((e.x, src_y, want_dir, e.name.clone(), e.carries.clone()));
            }
        }
        if needs.is_empty() {
            continue;
        }

        // Keyed by (item, side), NOT by item alone.
        //
        // Keying by item alone made the commonest possible arrangement refuse:
        // any item that BOTH neighbouring segments consume appears twice in the
        // gap — once from above, once from below — and two members tripped the
        // splitter refusal below. In the source every input sits on the top
        // edge, so after folding, consecutive segments' copies of the same item
        // land on opposite sides of the same gap. That is the normal case, not
        // an edge case, and it was the dominant refusal across the whole corpus
        // once the gap-lane conflicts were fixed (InputStranded 100 / 155 / 72
        // on chem5raw / pu4raw / usp2raw).
        //
        // Two lanes for one item is physically fine: they are at different rows
        // in different row blocks, each drawing from the bounding-box edge, and
        // each input belt carries its own `boundary_inputs` record which the
        // relocation pass moves independently. It is the same item supplied at
        // two edge points, which is exactly what the unfolded layout did.
        //
        // What still needs a splitter — and still refuses — is one item wanted
        // at two different COLUMNS on the SAME side, since one lane cannot
        // serve both.
        let mut by_item: BTreeMap<(Option<String>, bool), Vec<(i32, i32, EntityDirection, String)>> =
            BTreeMap::new();
        for (x, src_y, dir, name, carries) in needs {
            let is_above = src_y < gap_top;
            by_item
                .entry((carries, is_above))
                .or_default()
                .push((x, src_y, dir, name));
        }
        // One column per item per side: a lane serving two columns would have
        // to split, which needs a splitter this pass does not synthesize.
        if by_item.values().any(|m| m.len() > 1) {
            return Err(FoldRefusal::InputStranded {
                at: (bounds[k + 1], 0),
            });
        }
        type GapLane = ((Option<String>, bool), Vec<(i32, i32, EntityDirection, String)>);
        let lanes: Vec<GapLane> = by_item.into_iter().collect();

        // Rows are partitioned by SOURCE SIDE, and that is the whole fix for
        // the dominant multi-fold refusal (#492 measured 28 of 32 gap-lane
        // refusals on mil5 as cross-side).
        //
        // A gap takes inputs from both neighbours: the segment above feeds
        // from `gap_top - 1`, the one below from `gap_top + gap`. A lane's
        // climb runs from its row to its source, so it crosses every row
        // between them — and which rows those are depends on which side the
        // source is on. The two sides therefore impose OPPOSITE ordering
        // requirements on a single row assignment:
        //
        //   above-sourced, filling from the far side: lane 0 has the longest
        //     climb and crosses every other lane, so it needs the column no
        //     other lane's span covers — the largest (running left).
        //   below-sourced: the ordering inverts, because its climb runs the
        //     other way and crosses the rows on the other side of it.
        //
        // No single sort satisfies both, which is why the previous version
        // refused rather than mis-stacked: one global order is the bug, not a
        // bad choice of order.
        //
        // Partitioning removes the interaction instead of trying to order
        // around it. Above-sourced lanes take rows from `gap_top` downward,
        // below-sourced from `gap_top + gap - 1` upward. An above lane's climb
        // then spans only `[gap_top, row)` — entirely inside the above block —
        // and symmetrically for below, so the two groups provably never cross
        // each other as long as they fit: `n_above + n_below <= gap`.
        //
        // Within a group the span-nesting argument survives unchanged, but the
        // order FLIPS: index 0 is now adjacent to its source and crosses
        // nothing, while the deepest index crosses all the shallower ones. A
        // lane running left spans `[edge, x]` and so covers every column at or
        // below its own, so each successive lane needs a LARGER column —
        // ascending, where the old scheme wanted descending. Running right the
        // span is `[x, edge]` and the test inverts.
        //
        // Getting this backwards is not a validation error at the gap; it is
        // two items' belts on one tile several rows away.
        // Side comes off the key now, not re-derived from the member's row.
        let (mut above, mut below): (Vec<_>, Vec<_>) =
            lanes.into_iter().partition(|((_, is_above), _)| *is_above);
        for group in [&mut above, &mut below] {
            if to_left {
                group.sort_by_key(|(_, m)| m[0].0);
            } else {
                group.sort_by_key(|(_, m)| std::cmp::Reverse(m[0].0));
            }
        }
        if (above.len() + below.len()) as i32 > gap {
            return Err(FoldRefusal::GapLaneConflict { at: (-1, gap_top) });
        }
        // (row, item, member) — rows now come from the side-partitioned
        // assignment rather than a single running index.
        let placements: Vec<(i32, Option<String>, (i32, i32, EntityDirection, String))> = above
            .into_iter()
            .enumerate()
            .map(|(i, ((carries, _), m))| (gap_top + i as i32, carries, m[0].clone()))
            .chain(below.into_iter().enumerate().map(|(j, ((carries, _), m))| {
                (gap_top + gap - 1 - j as i32, carries, m[0].clone())
            }))
            .collect();

        // Instrumentation for #492. The refusal alone cannot distinguish two
        // very different causes, and they want different fixes:
        //   cross-side — a climb from above sweeps rows owned by lanes whose
        //     source is below (or vice versa). Row assignment ignores which
        //     side an input arrives from, so the two groups interleave. Fixable
        //     by partitioning rows by side; no underground needed.
        //   same-side tie — two same-side items share a column. No assignment
        //     order avoids this one; it needs a B12 dive.
        // Which dominates is a measurement, not a guess.
        let side_of = |src_y: i32| if src_y < gap_top { "above" } else { "below" };
        let mut owner: BTreeMap<(i32, i32), (String, &'static str, &'static str)> = BTreeMap::new();
        let debug_fold = std::env::var("SPAGHETTIO_FOLD_DEBUG").is_ok();

        for (lane_idx, (row, carries, member)) in placements.into_iter().enumerate() {
            let (x, src_y, dir, name) = member;

            // Lane from the box edge to the input's column.
            let (lo, hi) = if to_left { (edge_min_x, x) } else { (x, edge_max_x) };
            for cx in lo..=hi {
                if !occupied.insert((cx, row)) {
                    if debug_fold {
                        let mine = side_of(src_y);
                        let (other_item, other_side, other_kind) = owner
                            .get(&(cx, row))
                            .cloned()
                            .unwrap_or_else(|| ("<pre-existing>".into(), "?", "not-a-lane-tile"));
                        let verdict = if other_kind == "not-a-lane-tile" {
                            "PRE-EXISTING"
                        } else if other_side != mine {
                            "CROSS-SIDE"
                        } else if other_item != carries.clone().unwrap_or_default() {
                            "SAME-SIDE-TIE"
                        } else {
                            "SELF"
                        };
                        eprintln!(
                            "CLASH {verdict} kind=lane-run gap_top={gap_top} gap={gap} \
                             lane={lane_idx} row={row} at ({cx},{row}) item={carries:?} \
                             side={mine} span=[{lo},{hi}] vs item={other_item} \
                             side={other_side} kind={other_kind}"
                        );
                    }
                    return Err(FoldRefusal::GapLaneConflict { at: (cx, row) });
                }
                if debug_fold {
                    owner.insert(
                        (cx, row),
                        (carries.clone().unwrap_or_default(), side_of(src_y), "lane-run"),
                    );
                }
                folded.push(PlacedEntity {
                    name: name.clone(),
                    x: cx,
                    y: row,
                    // The lane's terminal tile TURNS toward the input; every
                    // other tile runs along the gap.
                    //
                    // This is the whole difference between the exit pass and
                    // this one, and getting it wrong is silent. Exits flow
                    // source -> lane -> edge, so the descent feeds the lane and
                    // every lane tile can face the run direction. Inputs flow
                    // edge -> lane -> source, so the lane has to hand items UP
                    // (or down) at the input's column. Left facing `run_dir`,
                    // the terminal tile outputs to the next column instead:
                    // items traverse the whole lane, pass the input, and dead-
                    // end at the far side. Measured on the mil5 2-fold: four
                    // orphan lane segments, each reported at exactly its own
                    // terminal column, and 45 furnaces starved behind them.
                    //
                    // A belt turning 90 degrees here is a corner, not a
                    // sideload — it preserves both lanes (factorio-mechanics
                    // B11) because nothing feeds the terminal tile's back.
                    direction: if cx == x { dir } else { run_dir },
                    carries: carries.clone(),
                    ..Default::default()
                });
            }
            // Climb from the lane to the tile that feeds the input belt,
            // facing the way the input flows.
            let (from, to) = if src_y < row {
                (src_y + 1, row - 1)
            } else {
                (row + 1, src_y - 1)
            };
            for y in from..=to {
                if !occupied.insert((x, y)) {
                    if debug_fold {
                        let mine = side_of(src_y);
                        let (other_item, other_side, other_kind) = owner
                            .get(&(x, y))
                            .cloned()
                            .unwrap_or_else(|| ("<pre-existing>".into(), "?", "not-a-lane-tile"));
                        let verdict = if other_kind == "not-a-lane-tile" {
                            "PRE-EXISTING"
                        } else if other_side != mine {
                            "CROSS-SIDE"
                        } else {
                            "SAME-SIDE-TIE"
                        };
                        eprintln!(
                            "CLASH {verdict} kind=climb gap_top={gap_top} gap={gap} \
                             lane={lane_idx} row={row} at ({x},{y}) item={carries:?} \
                             side={mine} climb=[{from},{to}] vs item={other_item} \
                             side={other_side} kind={other_kind}"
                        );
                    }
                    return Err(FoldRefusal::GapLaneConflict { at: (x, y) });
                }
                if debug_fold {
                    owner.insert(
                        (x, y),
                        (carries.clone().unwrap_or_default(), side_of(src_y), "climb"),
                    );
                }
                folded.push(PlacedEntity {
                    name: name.clone(),
                    x,
                    y,
                    direction: dir,
                    carries: carries.clone(),
                    ..Default::default()
                });
            }
            input_moved.insert((x, src_y), ((if to_left { lo } else { hi }, row), run_dir));
        }
    }

    // 5a. Runaway backstop.
    //
    // A fold rearranges a fixed entity set and adds reconnection geometry; it
    // has no business more than doubling the entity count. Two unbounded
    // reconnection loops once allocated until the OOM killer fired, and
    // because that OOM was global it took down unrelated processes with it.
    // The loops are ranges now, but a cheap absolute ceiling means the next
    // bug in here refuses a fold instead of taking out the machine.
    if folded.len() > entities.len() * 2 + 1024 {
        return Err(FoldRefusal::EntityExplosion);
    }

    // 5c. Belt continuity: folding may not create a dead end.
    //
    // Every belt that handed off to another belt in the source must still
    // hand off in the fold, or else leave the bounding box the way an output
    // does. A severed run that nothing reconnected is not a geometry error —
    // the tiles are all perfectly legal — so it surfaces far away as a
    // starved machine ("items can't reach input", "delivers 0.0/s"), which is
    // exactly the kind of silent functional break this transform must not
    // produce. Checking it here turns that into an explicit refusal.
    {
        // Footprints, not anchors: a belt handing off to a splitter feeds
        // whichever of its two tiles it abuts, which is often not the tile
        // the splitter is anchored at.
        let belt_tiles = |src: &[PlacedEntity]| -> FxHashSet<(i32, i32)> {
            let mut tiles = FxHashSet::default();
            for e in src.iter().filter(|e| is_belt_entity(&e.name)) {
                let (w, hh) = entity_dims(&e.name, e.direction);
                for dx in 0..w.max(1) {
                    for dy in 0..hh.max(1) {
                        tiles.insert((e.x + dx, e.y + dy));
                    }
                }
            }
            tiles
        };
        let src_belt = belt_tiles(&entities);
        let out_belt = belt_tiles(&folded);
        let (lo_x, hi_x) = (
            folded.iter().map(|e| e.x).min().unwrap_or(0),
            folded.iter().map(|e| e.x).max().unwrap_or(0),
        );
        let (lo_y, hi_y) = (
            folded.iter().map(|e| e.y).min().unwrap_or(0),
            folded.iter().map(|e| e.y).max().unwrap_or(0),
        );

        let debug = std::env::var("SPAGHETTIO_FOLD_DEBUG").is_ok();
        let mut severed: Vec<((i32, i32), &'static str)> = Vec::new();

        for e in entities.iter().filter(|e| is_belt_entity(&e.name)) {
            // An underground entrance hands off to its exit, not to the tile
            // in front of it.
            if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input") {
                continue;
            }
            let (dx, dy) = dir_to_vec(e.direction);
            let (w, hh) = entity_dims(&e.name, e.direction);
            let hands_off = |tiles: &FxHashSet<(i32, i32)>, x: i32, y: i32, w: i32, hh: i32, d: (i32, i32)| {
                (0..w.max(1)).any(|ox| {
                    (0..hh.max(1)).any(|oy| tiles.contains(&(x + ox + d.0, y + oy + d.1)))
                })
            };
            if !hands_off(&src_belt, e.x, e.y, w, hh, (dx, dy)) {
                continue; // already a terminus in the source
            }
            let Some((nx, ny, ndir)) = transform(e) else {
                continue;
            };
            let (ndx, ndy) = dir_to_vec(ndir);
            let (nw, nh) = entity_dims(&e.name, ndir);
            let leaves_box = (0..nw.max(1)).any(|ox| {
                (0..nh.max(1)).any(|oy| {
                    let (fx, fy) = (nx + ox + ndx, ny + oy + ndy);
                    fx < lo_x || fx > hi_x || fy < lo_y || fy > hi_y
                })
            });
            if !hands_off(&out_belt, nx, ny, nw, nh, (ndx, ndy)) && !leaves_box {
                severed.push(((e.x, e.y), "downstream"));
            }
        }

        // And the mirror: a belt that HAD an upstream feeder must still have
        // one. A severed run is usually not a dead end — it is perfectly well
        // connected downstream and simply has nothing arriving, which is why
        // it reads as "items can't reach input" on a machine rather than as a
        // belt error on the run itself.
        // Every footprint tile hands off, not just the anchor. A splitter is
        // two tiles across the flow and outputs from both; keying off the
        // anchor alone made a belt fed by a splitter's far tile look unfed —
        // and the 180° rotation swaps which physical tile the anchor names,
        // so it only ever showed up after folding.
        let fed_tiles = |src: &[PlacedEntity]| -> FxHashSet<(i32, i32)> {
            let mut fed = FxHashSet::default();
            for e in src.iter().filter(|e| is_belt_entity(&e.name)) {
                if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input") {
                    continue;
                }
                let (dx, dy) = dir_to_vec(e.direction);
                let (w, hh) = entity_dims(&e.name, e.direction);
                for ox in 0..w.max(1) {
                    for oy in 0..hh.max(1) {
                        fed.insert((e.x + ox + dx, e.y + oy + dy));
                    }
                }
            }
            fed
        };
        let src_fed = fed_tiles(&entities);
        let out_fed = fed_tiles(&folded);

        for e in entities.iter().filter(|e| is_belt_entity(&e.name)) {
            let (w, hh) = entity_dims(&e.name, e.direction);
            let had_feed = (0..w.max(1)).any(|dx| {
                (0..hh.max(1)).any(|dy| src_fed.contains(&(e.x + dx, e.y + dy)))
            });
            if !had_feed {
                continue;
            }
            let Some((nx, ny, ndir)) = transform(e) else {
                continue;
            };
            let (nw, nh) = entity_dims(&e.name, ndir);
            let still_fed = (0..nw.max(1))
                .any(|dx| (0..nh.max(1)).any(|dy| out_fed.contains(&(nx + dx, ny + dy))));
            if !still_fed {
                severed.push(((e.x, e.y), "upstream"));
            }
        }

        if debug && !severed.is_empty() {
            eprintln!("fold {folds:?}: {} severed belt(s)", severed.len());
            for ((x, y), side) in severed.iter().take(24) {
                let e = entities
                    .iter()
                    .find(|e| e.x == *x && e.y == *y && is_belt_entity(&e.name));
                eprintln!(
                    "  ({x},{y}) {side} name={:?} dir={:?} carries={:?}",
                    e.map(|e| e.name.as_str()),
                    e.map(|e| e.direction),
                    e.and_then(|e| e.carries.clone()),
                );
            }
        }
        if let Some((at, _)) = severed.first() {
            return Err(FoldRefusal::RunSevered { at: *at });
        }
    }

    // 5d. A boundary input must stay on the boundary.
    //
    // Inputs are fed from outside the bounding box, so they only work on an
    // edge. Segment parity decides where they land: an unrotated segment
    // keeps its inputs on its top row, a rotated one carries them to its
    // bottom row, and in both cases that row is interior for every segment
    // except the outermost. A single fold is fine — segment 0's inputs stay
    // at the top, segment 1's rotate to the layout's bottom — but from two
    // folds up, inputs land on gap rows with nothing able to supply them.
    //
    // Supplying them means routing a feed lane per item along a two-row gap
    // that already carries the exits, which is a channel-routing problem this
    // pass does not solve. Refuse rather than emit a factory whose inputs are
    // stranded: the failure is otherwise silent, showing up as starved
    // machines rather than as anything wrong at the boundary.
    {
        let (lo_x, hi_x) = (
            folded.iter().map(|e| e.x).min().unwrap_or(0),
            folded.iter().map(|e| e.x).max().unwrap_or(0),
        );
        let (lo_y, hi_y) = (
            folded.iter().map(|e| e.y).min().unwrap_or(0),
            folded.iter().map(|e| e.y).max().unwrap_or(0),
        );
        for b in &layout.boundary_inputs {
            let seg = seg_of(b.x).unwrap_or(n_segs - 1);
            let (nx, ny) = if seg % 2 == 0 {
                (b.x - bounds[seg], b.y + (seg as i32) * (h + gap))
            } else {
                (
                    bounds[seg + 1] - 1 - b.x,
                    (h - 1 - b.y) + (seg as i32) * (h + gap),
                )
            };
            // On the box edge it is fed from outside, as in the source. Off
            // the edge it is fed by the gap-lane pass above — but only if
            // that pass actually reached it, which it records. Anything
            // neither on an edge nor supplied really is stranded.
            let on_edge = nx <= lo_x || nx >= hi_x || ny <= lo_y || ny >= hi_y;
            let supplied = input_moved.keys().any(|&(mx, my)| (mx, my) == (nx, ny));
            if std::env::var("SPAGHETTIO_FOLD_DEBUG").is_ok() {
                eprintln!(
                    "input check: ({nx},{ny}) on_edge={on_edge} supplied={supplied} \
                     bbox=[{lo_x}..{hi_x}]x[{lo_y}..{hi_y}]"
                );
            }
            if !on_edge && !supplied {
                return Err(FoldRefusal::InputStranded { at: (b.x, b.y) });
            }
        }
    }

    // 5b. Lane integrity at every U-turn corner.
    //
    // A corner whose only input is perpendicular is a 90° TURN and carries
    // both lanes (`docs/factorio-mechanics.md` B11). A corner that also has a
    // straight input — or a second perpendicular one — is a SIDELOAD (B8) and
    // fills one lane only, halving the run's throughput and merging whatever
    // the second feeder carries.
    //
    // Nothing in the geometry guarantees the difference: one U-turn's
    // horizontal leg can cross another's vertical run. Where that run is
    // underground the leg passes over it harmlessly, but at a surface tile or
    // a UG endpoint the leg becomes a second input and silently downgrades
    // the turn. Refuse the fold rather than emit a layout whose belt
    // behaviour differs from the source it claims to preserve.
    {
        let belt_at: BTreeMap<(i32, i32), &PlacedEntity> = folded
            .iter()
            .filter(|e| is_belt_entity(&e.name))
            .map(|e| ((e.x, e.y), e))
            .collect();
        for &(cx, cy) in &corners {
            let corner = belt_at
                .get(&(cx, cy))
                .ok_or(FoldRefusal::CornerNotATurn)?;
            let mut straight = 0usize;
            let mut perpendicular = 0usize;
            for (nx, ny) in [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)] {
                let Some(n) = belt_at.get(&(nx, ny)) else {
                    continue;
                };
                // A UG entrance swallows items; it never feeds a neighbour.
                if is_ug_belt(&n.name) && n.io_type.as_deref() == Some("input") {
                    continue;
                }
                let (dx, dy) = dir_to_vec(n.direction);
                if (nx + dx, ny + dy) != (cx, cy) {
                    continue;
                }
                if n.direction == corner.direction {
                    straight += 1;
                } else {
                    perpendicular += 1;
                }
            }
            // Exactly one perpendicular feeder and nothing else is a turn.
            if straight > 0 || perpendicular != 1 {
                return Err(FoldRefusal::CornerNotATurn);
            }
        }
    }

    // 6. Compute new dimensions and normalise X origin.
    // Footprint-inclusive, not anchor-only: a 3x3 assembler anchored at the
    // bottom row occupies two rows below its anchor, so an anchor-derived
    // height under-reports by up to `footprint - 1` and the declared
    // dimensions describe a box the factory does not fit in. The rest of this
    // function already uses `entity_dims`; this was the one place that did not.
    let min_x = folded.iter().map(|e| e.x).min().unwrap_or(0);
    let max_x = folded
        .iter()
        .map(|e| e.x + entity_dims(&e.name, e.direction).0 - 1)
        .max()
        .unwrap_or(0);
    let max_y = folded
        .iter()
        .map(|e| e.y + entity_dims(&e.name, e.direction).1 - 1)
        .max()
        .unwrap_or(0);
    let x_shift = if min_x < 0 { -min_x } else { 0 };
    if x_shift > 0 {
        for e in &mut folded {
            e.x += x_shift;
        }
    }
    let new_width = (max_x - min_x + 1).max(1);
    let new_height = (max_y + 1).max(1);

    // 6. Build the result.
    let mut result = LayoutResult {
        entities: folded,
        width: new_width,
        height: new_height,
        ..layout.clone()
    };
    result.regions.clear();
    result.trace = None;
    // Poles must be re-placed, not carried: see `replace_poles`.
    result = replace_poles(&result);
    // Boundary records and surplus exits are coordinates into the layout and
    // must go through the fold like everything else. Shifting only their X
    // (as an earlier version did) left every one of them pointing at an
    // unrelated tile the moment any segment rotated, which the boundary and
    // input-rate-delivery checks then reported against the wrong geometry.
    let fold_point = |x: i32, y: i32| -> (i32, i32) {
        // Boundary tiles may sit one past the right edge; clamp into the last
        // segment so an output terminal folds with the machinery feeding it.
        let seg = seg_of(x).unwrap_or(n_segs - 1);
        let (nx, ny) = if seg % 2 == 0 {
            (x - bounds[seg], y + (seg as i32) * (h + gap))
        } else {
            (
                bounds[seg + 1] - 1 - x,
                (h - 1 - y) + (seg as i32) * (h + gap),
            )
        };
        (nx + x_shift, ny)
    };
    let fold_dir = |x: i32, d: EntityDirection| -> EntityDirection {
        match seg_of(x) {
            Some(seg) if seg % 2 == 1 => rotate_180_direction(d),
            _ => d,
        }
    };
    for b in &mut result.boundary_inputs {
        let folded_dir = fold_dir(b.x, b.direction);
        let (fx, fy) = fold_point(b.x, b.y);
        // An input the gap-lane pass rerouted is now fed at the lane's edge
        // terminus, so its record has to move there — the same reasoning that
        // made a stale `boundary_outputs` record produce 0.00/s.
        match input_moved.get(&(fx - x_shift, fy)) {
            Some(((tx, ty), tdir)) => {
                b.x = tx + x_shift;
                b.y = *ty;
                b.direction = *tdir;
            }
            None => {
                b.direction = folded_dir;
                (b.x, b.y) = (fx, fy);
            }
        }
    }
    for b in &mut result.boundary_outputs {
        let folded_dir = fold_dir(b.x, b.direction);
        let (fx, fy) = fold_point(b.x, b.y);
        // `fold_point` already applied the x-shift; the relocation map is in
        // pre-shift coordinates.
        match exit_moved.get(&(fx - x_shift, fy)) {
            Some(((tx, ty), tdir)) => {
                b.x = tx + x_shift;
                b.y = *ty;
                b.direction = *tdir;
            }
            None => {
                b.direction = folded_dir;
                (b.x, b.y) = (fx, fy);
            }
        }
    }
    for (_, x, y) in &mut result.surplus_exits {
        (*x, *y) = fold_point(*x, *y);
    }

    // Collapse boundary records that now describe ONE physical terminus.
    //
    // A gap carries one lane per item, so N source exits of the same item merge
    // into that lane and `exit_moved` maps every one of them to the same
    // terminus tile. The geometry is right — one lane, one exit — but the record
    // list ends up with N copies of it, and a consumer that trusts the list
    // builds N of whatever a record implies.
    //
    // The sim harness does exactly that: two identical output records made it
    // build two drain rigs on one tile, whose chest banks overlapped by 7 tiles
    // (`ext_len = 11 + 2*idx` separates rigs by only 2), and it correctly
    // invalidated the whole run rather than report a rate measured through
    // cross-feeding chests. That is issue #499, and it presented as a harness
    // bug because the overlap is in harness-side geometry — but the harness was
    // faithfully building what the manifest declared.
    //
    // Deduped on the full record, not just position: two records at one tile
    // carrying DIFFERENT items would be a real defect and must stay visible to
    // `check_boundary_record_integrity` rather than be quietly merged away.
    // Order-preserving, because the harness indexes rigs by record order and a
    // reordering would move every rig's geometry.
    // Linear scan on `BoundaryRecord`'s own PartialEq: a handful of records per
    // layout, so O(n^2) is free, and equality-on-the-whole-record is exactly the
    // semantics wanted — no hand-written key to drift from the struct's fields.
    let dedupe = |recs: &mut Vec<crate::models::BoundaryRecord>| {
        let mut seen: Vec<crate::models::BoundaryRecord> = Vec::with_capacity(recs.len());
        recs.retain(|b| {
            if seen.contains(b) {
                false
            } else {
                seen.push(b.clone());
                true
            }
        });
    };
    dedupe(&mut result.boundary_inputs);
    dedupe(&mut result.boundary_outputs);

    Ok(result)
}
