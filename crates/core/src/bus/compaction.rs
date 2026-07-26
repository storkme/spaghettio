//! RFC-057 topology-preserving dense repacking foundation.
//!
//! This module freezes the logical production graph and the placed machine
//! multiset before any geometric search.  The first placement primitive is an
//! exact per-axis constraint-graph compactor: for a fixed relative order it
//! computes the minimum legal coordinates by longest paths in a DAG.

use std::collections::BTreeMap;

use crate::common::{
    dir_to_vec, entity_size, inserter_reach, is_belt_entity, is_inserter, oriented_splitter_dims,
    QualityTier,
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
    compacted.width -= removed_before[layout.width as usize];
    compacted.regions.clear();
    compacted.trace = None;
    compacted
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
}

impl CompactIr {
    pub fn from_layout(layout: &LayoutResult) -> Self {
        Self {
            islands: extract_rigid_islands(layout),
            route_nets: extract_route_nets(layout),
        }
    }
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
    }
}
