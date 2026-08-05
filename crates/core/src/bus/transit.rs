//! RFC-064 post-route transit measurement.
//!
//! The metric is intentionally artifact-facing: it walks the belts, underground
//! spans, pipes, and pipe-to-ground spans that actually exist in a final
//! [`LayoutResult`]. It does not read the packer's placement estimate. Terminal
//! discovery follows the same physical machine/inserter and fluid-port geometry
//! the validators use, while adjacency and underground pairing reuse validator
//! helpers so the two views cannot silently drift.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::bus::compaction::{ProductionEdge, ProductionSignature};
use crate::common::{
    dir_to_vec, entity_size, inserter_reach, is_belt_entity, is_inserter, is_machine_entity,
    is_splitter, is_ug_belt, splitter_second_tile,
};
use crate::fluid_ports::fluid_ports;
use crate::models::{EntityDirection, LayoutResult, SolverResult};

type Tile = (i32, i32);
type Graph = FxHashMap<Tile, Vec<(Tile, i64)>>;

const RATE_SCALE: f64 = 1_000_000_000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTransit {
    pub producer_recipes: Vec<String>,
    pub item: String,
    pub consumer_recipe: String,
    pub planned_rate: f64,
    pub is_fluid: bool,
    pub producer_terminals: usize,
    pub consumer_terminals: usize,
    pub path_length: f64,
    pub weighted_cost: f64,
    pub direct_insertion: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransitMeasurement {
    pub total: f64,
    pub solid_total: f64,
    pub fluid_total: f64,
    pub edges: Vec<EdgeTransit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitError {
    InvalidFluidWeight,
    Signature(String),
    MissingProducerTerminal {
        item: String,
        consumer_recipe: String,
    },
    MissingConsumerTerminal {
        item: String,
        consumer_recipe: String,
    },
    UnreachableConsumerTerminal {
        item: String,
        consumer_recipe: String,
        terminal: Tile,
    },
}

impl std::fmt::Display for TransitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFluidWeight => write!(f, "fluid_weight must be finite and non-negative"),
            Self::Signature(message) => write!(f, "production signature: {message}"),
            Self::MissingProducerTerminal {
                item,
                consumer_recipe,
            } => write!(
                f,
                "edge {item} -> {consumer_recipe}: no matching producer transport terminal",
            ),
            Self::MissingConsumerTerminal {
                item,
                consumer_recipe,
            } => write!(
                f,
                "edge {item} -> {consumer_recipe}: no matching consumer transport terminal",
            ),
            Self::UnreachableConsumerTerminal {
                item,
                consumer_recipe,
                terminal,
            } => write!(
                f,
                "edge {item} -> {consumer_recipe}: consumer terminal {terminal:?} is unreachable",
            ),
        }
    }
}

impl std::error::Error for TransitError {}

#[derive(Debug, Clone)]
struct TransportTile {
    direction: EntityDirection,
    carries: Option<String>,
    is_splitter: bool,
    is_ug_input: bool,
}

#[derive(Debug, Clone)]
struct PipeTile {
    direction: EntityDirection,
    carries: Option<String>,
    is_ptg: bool,
}

#[derive(Default)]
struct TerminalBuckets {
    exact: FxHashSet<Tile>,
    unlabelled: FxHashSet<Tile>,
}

impl TerminalBuckets {
    fn insert(&mut self, tile: Tile, carries: Option<&str>, item: &str) {
        match carries {
            Some(carried) if carried == item => {
                self.exact.insert(tile);
            }
            None => {
                self.unlabelled.insert(tile);
            }
            Some(_) => {}
        }
    }

    fn finish(self) -> FxHashSet<Tile> {
        if self.exact.is_empty() {
            self.unlabelled
        } else {
            self.exact
        }
    }
}

/// Measure RFC-064's realized, rate-weighted production-edge transit.
///
/// Each edge uses a directed multi-source shortest-path walk from every
/// matching producer terminal to every distinct consumer terminal. Surface
/// steps cost one tile; underground jumps cost their physical Manhattan span.
/// A solver-declared direct-insertion edge with no transport terminals uses
/// its actual machine-to-machine inserter span. Any other missing or
/// unreachable terminal is an error, making the candidate inadmissible.
pub fn measure_realized_transit(
    layout: &LayoutResult,
    solver_result: &SolverResult,
    fluid_weight: f64,
) -> Result<TransitMeasurement, TransitError> {
    if !fluid_weight.is_finite() || fluid_weight < 0.0 {
        return Err(TransitError::InvalidFluidWeight);
    }
    let signature =
        ProductionSignature::from_solver(solver_result).map_err(TransitError::Signature)?;
    let machine_by_tile = machine_tiles(layout);
    let belt_tiles = belt_tiles(layout);
    let pipe_tiles = pipe_tiles(layout);

    let mut result = TransitMeasurement {
        total: 0.0,
        solid_total: 0.0,
        fluid_total: 0.0,
        edges: Vec::with_capacity(signature.edges.len()),
    };

    for edge in &signature.edges {
        let direct_lengths = direct_insertion_lengths(layout, &machine_by_tile, edge);
        let (sources, consumers, graph) = if edge.is_fluid {
            let (sources, consumers) = fluid_terminals(layout, &pipe_tiles, edge);
            (
                sources,
                consumers,
                fluid_graph(layout, &pipe_tiles, &edge.item),
            )
        } else {
            let (sources, consumers) = solid_terminals(layout, &machine_by_tile, &belt_tiles, edge);
            (
                sources,
                consumers,
                solid_graph(layout, &belt_tiles, &edge.item),
            )
        };

        let declared_di = solver_result.di_couplings.iter().any(|coupling| {
            coupling.item == edge.item
                && coupling.consumer_recipe == edge.consumer_recipe
                && edge.producer_recipes.contains(&coupling.producer_recipe)
        });
        let (path_length, producer_count, consumer_count, direct_insertion) = if sources.is_empty()
            && consumers.is_empty()
            && declared_di
            && !direct_lengths.is_empty()
        {
            (
                direct_lengths.iter().sum::<i64>() as f64 / direct_lengths.len() as f64,
                direct_lengths.len(),
                direct_lengths.len(),
                true,
            )
        } else {
            if sources.is_empty() {
                return Err(TransitError::MissingProducerTerminal {
                    item: edge.item.clone(),
                    consumer_recipe: edge.consumer_recipe.clone(),
                });
            }
            if consumers.is_empty() {
                return Err(TransitError::MissingConsumerTerminal {
                    item: edge.item.clone(),
                    consumer_recipe: edge.consumer_recipe.clone(),
                });
            }
            let distance = shortest_distances(&graph, &sources);
            let mut total_length = 0i64;
            for &terminal in &consumers {
                let Some(&length) = distance.get(&terminal) else {
                    return Err(TransitError::UnreachableConsumerTerminal {
                        item: edge.item.clone(),
                        consumer_recipe: edge.consumer_recipe.clone(),
                        terminal,
                    });
                };
                total_length += length;
            }
            (
                total_length as f64 / consumers.len() as f64,
                sources.len(),
                consumers.len(),
                false,
            )
        };

        let planned_rate = edge.rate as f64 / RATE_SCALE;
        let weighted_cost =
            planned_rate * path_length * if edge.is_fluid { fluid_weight } else { 1.0 };
        if edge.is_fluid {
            result.fluid_total += weighted_cost;
        } else {
            result.solid_total += weighted_cost;
        }
        result.edges.push(EdgeTransit {
            producer_recipes: edge.producer_recipes.clone(),
            item: edge.item.clone(),
            consumer_recipe: edge.consumer_recipe.clone(),
            planned_rate,
            is_fluid: edge.is_fluid,
            producer_terminals: producer_count,
            consumer_terminals: consumer_count,
            path_length,
            weighted_cost,
            direct_insertion,
        });
    }
    result.total = result.solid_total + result.fluid_total;
    Ok(result)
}

fn compatible(carries: Option<&str>, item: &str) -> bool {
    carries.is_some_and(|carried| carried == item)
}

fn machine_tiles(layout: &LayoutResult) -> FxHashMap<Tile, usize> {
    let mut result = FxHashMap::default();
    for (index, entity) in layout.entities.iter().enumerate() {
        if !is_machine_entity(&entity.name) {
            continue;
        }
        let (width, height) = entity_size(&entity.name);
        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                result.insert((entity.x + dx, entity.y + dy), index);
            }
        }
    }
    result
}

fn belt_tiles(layout: &LayoutResult) -> FxHashMap<Tile, TransportTile> {
    let mut result = FxHashMap::default();
    for entity in &layout.entities {
        if !is_belt_entity(&entity.name) {
            continue;
        }
        let tile = TransportTile {
            direction: entity.direction,
            carries: entity.carries.clone(),
            is_splitter: is_splitter(&entity.name),
            is_ug_input: is_ug_belt(&entity.name) && entity.io_type.as_deref() == Some("input"),
        };
        result.insert((entity.x, entity.y), tile.clone());
        if tile.is_splitter {
            result.insert(splitter_second_tile(entity), tile);
        }
    }
    result
}

fn pipe_tiles(layout: &LayoutResult) -> FxHashMap<Tile, PipeTile> {
    layout
        .entities
        .iter()
        .filter(|entity| matches!(entity.name.as_str(), "pipe" | "pipe-to-ground"))
        .map(|entity| {
            (
                (entity.x, entity.y),
                PipeTile {
                    direction: entity.direction,
                    carries: entity.carries.clone(),
                    is_ptg: entity.name == "pipe-to-ground",
                },
            )
        })
        .collect()
}

fn solid_terminals(
    layout: &LayoutResult,
    machine_by_tile: &FxHashMap<Tile, usize>,
    belts: &FxHashMap<Tile, TransportTile>,
    edge: &ProductionEdge,
) -> (FxHashSet<Tile>, FxHashSet<Tile>) {
    let mut sources = TerminalBuckets::default();
    let mut consumers = TerminalBuckets::default();
    for inserter in &layout.entities {
        if !is_inserter(&inserter.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(inserter.direction);
        let reach = inserter_reach(&inserter.name);
        let pickup = (inserter.x - dx * reach, inserter.y - dy * reach);
        let drop = (inserter.x + dx * reach, inserter.y + dy * reach);
        if let (Some(&machine_index), Some(tile)) = (machine_by_tile.get(&pickup), belts.get(&drop))
        {
            let recipe = layout.entities[machine_index].recipe.as_deref();
            if recipe.is_some_and(|recipe| edge.producer_recipes.iter().any(|r| r == recipe)) {
                sources.insert(drop, tile.carries.as_deref(), &edge.item);
            }
        }
        if let (Some(tile), Some(&machine_index)) = (belts.get(&pickup), machine_by_tile.get(&drop))
        {
            if layout.entities[machine_index].recipe.as_deref()
                == Some(edge.consumer_recipe.as_str())
            {
                consumers.insert(pickup, tile.carries.as_deref(), &edge.item);
            }
        }
    }
    (sources.finish(), consumers.finish())
}

fn fluid_terminals(
    layout: &LayoutResult,
    pipes: &FxHashMap<Tile, PipeTile>,
    edge: &ProductionEdge,
) -> (FxHashSet<Tile>, FxHashSet<Tile>) {
    let mut sources = TerminalBuckets::default();
    let mut consumers = TerminalBuckets::default();
    for machine in &layout.entities {
        if !is_machine_entity(&machine.name) {
            continue;
        }
        let Some(recipe) = machine.recipe.as_deref() else {
            continue;
        };
        let is_producer = edge.producer_recipes.iter().any(|r| r == recipe);
        let is_consumer = recipe == edge.consumer_recipe;
        if !is_producer && !is_consumer {
            continue;
        }
        for &(dx, dy, kind) in fluid_ports(&machine.name, machine.mirror, machine.direction) {
            let position = (machine.x + dx, machine.y + dy);
            let Some(pipe) = pipes.get(&position) else {
                continue;
            };
            if is_producer && kind == "output" {
                sources.insert(position, pipe.carries.as_deref(), &edge.item);
            }
            if is_consumer && kind == "input" {
                consumers.insert(position, pipe.carries.as_deref(), &edge.item);
            }
        }
    }
    (sources.finish(), consumers.finish())
}

fn direct_insertion_lengths(
    layout: &LayoutResult,
    machine_by_tile: &FxHashMap<Tile, usize>,
    edge: &ProductionEdge,
) -> Vec<i64> {
    let mut lengths = Vec::new();
    for inserter in &layout.entities {
        if !is_inserter(&inserter.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(inserter.direction);
        let reach = inserter_reach(&inserter.name);
        let pickup = (inserter.x - dx * reach, inserter.y - dy * reach);
        let drop = (inserter.x + dx * reach, inserter.y + dy * reach);
        let (Some(&producer), Some(&consumer)) =
            (machine_by_tile.get(&pickup), machine_by_tile.get(&drop))
        else {
            continue;
        };
        let producer_recipe = layout.entities[producer].recipe.as_deref();
        let consumer_recipe = layout.entities[consumer].recipe.as_deref();
        if producer_recipe.is_some_and(|recipe| edge.producer_recipes.iter().any(|r| r == recipe))
            && consumer_recipe == Some(edge.consumer_recipe.as_str())
        {
            lengths.push((drop.0 - pickup.0).abs() as i64 + (drop.1 - pickup.1).abs() as i64);
        }
    }
    lengths
}

fn solid_graph(layout: &LayoutResult, belts: &FxHashMap<Tile, TransportTile>, item: &str) -> Graph {
    let ug_pairs = crate::validate::belt_flow::build_ug_pairs(layout);
    let splitter_siblings = crate::validate::belt_flow::build_splitter_siblings(layout);
    let mut graph = Graph::default();
    for (&position, tile) in belts {
        if !compatible(tile.carries.as_deref(), item) {
            continue;
        }
        if tile.is_ug_input {
            if let Some(&peer) = ug_pairs.get(&position) {
                if belts
                    .get(&peer)
                    .is_some_and(|other| compatible(other.carries.as_deref(), item))
                {
                    let span =
                        (peer.0 - position.0).abs() as i64 + (peer.1 - position.1).abs() as i64;
                    graph.entry(position).or_default().push((peer, span));
                }
            }
            continue;
        }
        let (dx, dy) = dir_to_vec(tile.direction);
        if tile.is_splitter {
            let sibling = splitter_siblings
                .get(&position)
                .copied()
                .unwrap_or(position);
            for output_base in [position, sibling] {
                let next = (output_base.0 + dx, output_base.1 + dy);
                if belts
                    .get(&next)
                    .is_some_and(|other| compatible(other.carries.as_deref(), item))
                {
                    let cost =
                        (next.0 - position.0).abs() as i64 + (next.1 - position.1).abs() as i64;
                    graph.entry(position).or_default().push((next, cost));
                }
            }
        } else {
            let next = (position.0 + dx, position.1 + dy);
            if belts
                .get(&next)
                .is_some_and(|other| compatible(other.carries.as_deref(), item))
            {
                graph.entry(position).or_default().push((next, 1));
            }
        }
    }
    graph
}

fn fluid_graph(layout: &LayoutResult, pipes: &FxHashMap<Tile, PipeTile>, item: &str) -> Graph {
    let ptg_pairs = crate::validate::fluids::find_ptg_pairs(layout);
    let mut graph = Graph::default();
    for (&position, pipe) in pipes {
        if !compatible(pipe.carries.as_deref(), item) {
            continue;
        }
        let surface_neighbours: Vec<Tile> = if pipe.is_ptg {
            vec![crate::validate::fluids::ptg_surface_neighbour(
                position.0,
                position.1,
                pipe.direction,
            )]
        } else {
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .into_iter()
                .map(|(dx, dy)| (position.0 + dx, position.1 + dy))
                .collect()
        };
        for neighbour in surface_neighbours {
            let Some(other) = pipes.get(&neighbour) else {
                continue;
            };
            if !compatible(other.carries.as_deref(), item) {
                continue;
            }
            if other.is_ptg
                && crate::validate::fluids::ptg_surface_neighbour(
                    neighbour.0,
                    neighbour.1,
                    other.direction,
                ) != position
            {
                continue;
            }
            graph.entry(position).or_default().push((neighbour, 1));
        }
        if let Some(&peer) = ptg_pairs.get(&position) {
            if pipes
                .get(&peer)
                .is_some_and(|other| compatible(other.carries.as_deref(), item))
            {
                let span = (peer.0 - position.0).abs() as i64 + (peer.1 - position.1).abs() as i64;
                graph.entry(position).or_default().push((peer, span));
            }
        }
    }
    graph
}

fn shortest_distances(graph: &Graph, sources: &FxHashSet<Tile>) -> FxHashMap<Tile, i64> {
    let mut distances = FxHashMap::default();
    let mut queue = BinaryHeap::new();
    for &source in sources {
        distances.insert(source, 0);
        queue.push(Reverse((0i64, source)));
    }
    while let Some(Reverse((distance, tile))) = queue.pop() {
        if distances.get(&tile).copied().unwrap_or(i64::MAX) < distance {
            continue;
        }
        for &(next, cost) in graph.get(&tile).map(Vec::as_slice).unwrap_or(&[]) {
            let next_distance = distance + cost;
            if distances.get(&next).copied().unwrap_or(i64::MAX) <= next_distance {
                continue;
            }
            distances.insert(next, next_distance);
            queue.push(Reverse((next_distance, next)));
        }
    }
    distances
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DICoupling, ItemFlow, MachineSpec, PlacedEntity};

    fn flow(item: &str, rate: f64, is_fluid: bool) -> ItemFlow {
        ItemFlow {
            item: item.to_string(),
            rate,
            is_fluid,
            ..Default::default()
        }
    }

    fn solver(item: &str, rate: f64, is_fluid: bool) -> SolverResult {
        SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "producer".to_string(),
                    count: 1.0,
                    outputs: vec![flow(item, rate, is_fluid)],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "consumer".to_string(),
                    count: 1.0,
                    inputs: vec![flow(item, rate, is_fluid)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn machine(recipe: &str, x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "assembling-machine-2".to_string(),
            recipe: Some(recipe.to_string()),
            x,
            y,
            ..Default::default()
        }
    }

    fn inserter(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity {
            name: "inserter".to_string(),
            x,
            y,
            direction: EntityDirection::East,
            ..Default::default()
        }
    }

    fn belt(name: &str, x: i32, y: i32, io_type: Option<&str>) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            direction: EntityDirection::East,
            io_type: io_type.map(str::to_string),
            carries: Some("part".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn measures_directed_surface_and_underground_span_costs() {
        let mut surface = LayoutResult {
            entities: vec![machine("producer", 0, 0), inserter(3, 1)],
            ..Default::default()
        };
        surface
            .entities
            .extend((4..=8).map(|x| belt("transport-belt", x, 1, None)));
        surface
            .entities
            .extend([inserter(9, 1), machine("consumer", 10, 0)]);
        let measured = measure_realized_transit(&surface, &solver("part", 2.0, false), 0.5)
            .expect("surface path must measure");
        assert_eq!(measured.edges[0].path_length, 4.0);
        assert_eq!(measured.total, 8.0);

        let underground = LayoutResult {
            entities: vec![
                machine("producer", 0, 0),
                inserter(3, 1),
                belt("transport-belt", 4, 1, None),
                belt("underground-belt", 5, 1, Some("input")),
                belt("underground-belt", 9, 1, Some("output")),
                belt("transport-belt", 10, 1, None),
                inserter(11, 1),
                machine("consumer", 12, 0),
            ],
            ..Default::default()
        };
        let measured = measure_realized_transit(&underground, &solver("part", 2.0, false), 0.5)
            .expect("underground path must measure");
        assert_eq!(measured.edges[0].path_length, 6.0);
        assert_eq!(measured.total, 12.0);
    }

    #[test]
    fn measures_fluid_paths_with_weight() {
        let mut layout = LayoutResult {
            entities: vec![machine("producer", 0, 0), machine("consumer", 0, 7)],
            ..Default::default()
        };
        layout.entities.extend((3..=6).map(|y| PlacedEntity {
            name: "pipe".to_string(),
            x: 1,
            y,
            carries: Some("water".to_string()),
            ..Default::default()
        }));
        let measured = measure_realized_transit(&layout, &solver("water", 10.0, true), 0.5)
            .expect("fluid path must measure");
        assert_eq!(measured.edges[0].path_length, 3.0);
        assert_eq!(measured.fluid_total, 15.0);
        assert_eq!(measured.total, 15.0);
    }

    #[test]
    fn direct_insertion_uses_actual_machine_port_span() {
        let mut sr = solver("part", 2.0, false);
        sr.di_couplings.push(DICoupling {
            producer_recipe: "producer".to_string(),
            consumer_recipe: "consumer".to_string(),
            item: "part".to_string(),
            producer_count: 1.0,
            consumer_count: 1.0,
        });
        let layout = LayoutResult {
            entities: vec![
                machine("producer", 0, 0),
                inserter(3, 1),
                machine("consumer", 4, 0),
            ],
            ..Default::default()
        };
        let measured = measure_realized_transit(&layout, &sr, 0.5)
            .expect("declared direct insertion must measure");
        assert_eq!(measured.edges[0].path_length, 2.0);
        assert_eq!(measured.total, 4.0);
        assert!(measured.edges[0].direct_insertion);
    }

    /// An unlabelled transport tile must NOT be traversable by every net.
    ///
    /// `compatible` returned `carries.is_none_or(...)`, so a tile with no
    /// `carries` was a universal shortcut: usable by any item, in both the
    /// solid and fluid graphs. That is not hypothetical on this path —
    /// `bands.rs` stamps band belt rows with
    /// `carries: tag_items.then(...)` where `tag_items =
    /// explicit_selection.is_some()`, so the **legacy RFC-058 `None` path
    /// leaves every band belt row unlabelled**, and the metric would route
    /// every net through every row.
    ///
    /// A decision-log entry previously downgraded this to "inert, 0 of 1889
    /// belt-ish entities unlabelled". That census covered the six *recorded*
    /// layouts, which are all explicit-selection and therefore tagged — i.e.
    /// exactly the population where the bug cannot fire. Wrong scope, not
    /// wrong count (PR #575 bot review, 3/3 passes, twice).
    ///
    /// Strict is also the behaviour this module's philosophy already implies:
    /// an unmeasurable edge should surface as an `Unreachable*` refusal, not
    /// as a shorter path.
    #[test]
    fn unlabelled_tiles_are_not_universal_shortcuts() {
        assert!(compatible(Some("iron-plate"), "iron-plate"));
        assert!(!compatible(Some("copper-plate"), "iron-plate"));
        assert!(
            !compatible(None, "iron-plate"),
            "an unlabelled tile must not be usable by an arbitrary net"
        );
    }
}
