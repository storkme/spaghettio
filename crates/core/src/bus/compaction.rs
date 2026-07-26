//! RFC-057 topology-preserving dense repacking foundation.
//!
//! This module freezes the logical production graph and the placed machine
//! multiset before any geometric search.  The first placement primitive is an
//! exact per-axis constraint-graph compactor: for a fixed relative order it
//! computes the minimum legal coordinates by longest paths in a DAG.

use std::collections::BTreeMap;

use crate::common::{
    dir_to_vec, entity_size, inserter_reach, is_belt_entity, is_inserter, is_surface_belt,
    is_ug_belt, oriented_splitter_dims, ug_max_reach, ug_to_surface_tier, QualityTier,
};
use crate::models::{EntityDirection, LayoutResult, ModuleItem, SolverResult};

const RATE_SCALE: f64 = 1_000_000_000.0;

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
pub fn compact_validated_columns(
    layout: &LayoutResult,
    solver: &SolverResult,
    max_commits: usize,
) -> LayoutResult {
    use crate::validate::{self, LayoutStyle, Severity};

    let mut current = compact_transport_geometry(layout);
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

    let mut current = compact_transport_geometry(layout);
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
    let mut current = compact_transport_geometry(layout);
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
    drop(protect_both);

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

    // Boundary terminals have no island-relative representation.
    for net in &ir.route_nets {
        let terminals = by_item.get_mut(&net.item).unwrap();
        for terminal in &net.terminals {
            if !matches!(
                terminal.kind,
                RouteTerminalKind::BoundaryInput | RouteTerminalKind::BoundaryOutput
            ) {
                continue;
            }
            terminals.push(ManifoldTerminal {
                kind: terminal.kind,
                x: terminal.x,
                y: terminal.y,
                island_id: None,
                inserter_entity_index: None,
            });
        }
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
        let area: i64 = island_ids
            .iter()
            .map(|&id| {
                let block = &ir.islands[id].block;
                i64::from(block.width + clearance) * i64::from(block.height + clearance)
            })
            .sum();
        let target_width = (area as f64).sqrt().ceil() as i32;
        let mut x = 0;
        let mut y = 0;
        let mut row_height = 0;
        let mut max_x = 0;
        for &id in &island_ids {
            let width = islands[id].block.width;
            let height = islands[id].block.height;
            if x > 0 && x + width > target_width {
                x = 0;
                y += row_height + clearance;
                row_height = 0;
            }
            islands[id].block.x = x;
            islands[id].block.y = y;
            x += width + clearance;
            row_height = row_height.max(height);
            max_x = max_x.max(x - clearance);
        }
        let max_y = island_ids
            .iter()
            .map(|&id| islands[id].block.y + islands[id].block.height)
            .max()
            .unwrap_or(0);
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

fn merger_stages(mut inputs: u32) -> Vec<BalancerStage> {
    let mut stages = Vec::new();
    while inputs > 1 {
        let copies = inputs / 2;
        if copies > 0 {
            stages.push(BalancerStage { n: 2, m: 1, copies });
        }
        inputs = copies + inputs % 2;
    }
    stages
}

fn distributor_stages(outputs: u32) -> Vec<BalancerStage> {
    if outputs <= 1 {
        return Vec::new();
    }
    // Grow a possibly ragged binary tree to exactly the requested leaves.
    let mut stages = Vec::new();
    let mut leaves = 1;
    while leaves < outputs {
        let copies = leaves.min(outputs - leaves);
        stages.push(BalancerStage { n: 1, m: 2, copies });
        leaves += copies;
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
            let mut chunks = merge_frontier.into_iter();
            loop {
                let Some(first) = chunks.next() else {
                    break;
                };
                let Some(second) = chunks.next() else {
                    next.push(first);
                    break;
                };
                let node = nodes.len();
                nodes.push(ManifoldBalancerNode {
                    id: node,
                    lane: group.lane,
                    role: BalancerNodeRole::Merge,
                    n: 2,
                    m: 1,
                    level,
                });
                edges.push(ManifoldGraphEdge {
                    from: first,
                    to: ManifoldEndpoint::NodeInput { node, port: 0 },
                });
                edges.push(ManifoldGraphEdge {
                    from: second,
                    to: ManifoldEndpoint::NodeInput { node, port: 1 },
                });
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
        let mut distribute_frontier = vec![ManifoldEndpoint::LaneOutput(group.lane)];
        let mut level = 0;
        while distribute_frontier.len() < consumer_count {
            let source = distribute_frontier.remove(0);
            let node = nodes.len();
            nodes.push(ManifoldBalancerNode {
                id: node,
                lane: group.lane,
                role: BalancerNodeRole::Distribute,
                n: 1,
                m: 2,
                level,
            });
            edges.push(ManifoldGraphEdge {
                from: source,
                to: ManifoldEndpoint::NodeInput { node, port: 0 },
            });
            distribute_frontier.push(ManifoldEndpoint::NodeOutput { node, port: 0 });
            distribute_frontier.push(ManifoldEndpoint::NodeOutput { node, port: 1 });
            level += 1;
        }
        distribute_frontier.sort();
        for (source, terminal) in distribute_frontier
            .into_iter()
            .zip(group.consumers.iter().cloned())
        {
            edges.push(ManifoldGraphEdge {
                from: source,
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

fn entity_dims(name: &str, direction: EntityDirection) -> (i32, i32) {
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
    fn binary_balancer_hierarchies_cover_arbitrary_terminal_counts() {
        for count in 0..=129 {
            let mergers = merger_stages(count);
            let mut remaining = count;
            for stage in &mergers {
                assert_eq!((stage.n, stage.m), (2, 1));
                remaining = remaining - stage.copies * 2 + stage.copies;
            }
            assert_eq!(remaining, count.min(1));

            let distributors = distributor_stages(count);
            let leaves = distributors
                .iter()
                .fold(1u32, |leaves, stage| leaves + stage.copies);
            assert_eq!(leaves, count.max(1));
            assert!(mergers
                .iter()
                .chain(distributors.iter())
                .all(|stage| crate::bus::balancer::shape_is_stampable(stage.n, stage.m)));
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
