//! RFC-057 topology-preserving dense repacking foundation.
//!
//! This module freezes the logical production graph and the placed machine
//! multiset before any geometric search.  The first placement primitive is an
//! exact per-axis constraint-graph compactor: for a fixed relative order it
//! computes the minimum legal coordinates by longest paths in a DAG.

use std::collections::BTreeMap;

use crate::common::{entity_size, oriented_splitter_dims, QualityTier};
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
        (&a.0, a.1, a.2.map(|q| q.level()))
            .cmp(&(&b.0, b.1, b.2.map(|q| q.level())))
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
            CompactAxis::Y => (
                self.x,
                self.x + self.width,
                other.x,
                other.x + other.width,
            ),
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
            if matches!(entity.direction, EntityDirection::East | EntityDirection::West)
                && width != height
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
                    coordinate[previous]
                        + blocks[previous].axis_size(axis)
                        + clearance.max(0),
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
    a.x < b.x + b.width
        && b.x < a.x + a.width
        && a.y < b.y + b.height
        && b.y < a.y + a.height
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
            CompactBlock { id: 0, x: 10, y: 0, width: 3, height: 3 },
            CompactBlock { id: 1, x: 30, y: 1, width: 4, height: 2 },
            CompactBlock { id: 2, x: 50, y: 8, width: 5, height: 2 },
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
        let a = LayoutResult { entities: vec![machine.clone(), belt.clone()], ..Default::default() };
        let mut moved = machine;
        moved.x = -7;
        moved.y = 4;
        let b = LayoutResult { entities: vec![belt, moved], ..Default::default() };
        assert_eq!(
            PlacedMachineSignature::from_layout(&a),
            PlacedMachineSignature::from_layout(&b)
        );
    }
}
