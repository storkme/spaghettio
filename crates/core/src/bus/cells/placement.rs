//! Shared macro-placement model for RFC-055 and RFC-056.
//!
//! This module deliberately knows nothing about tile routing. It converts a
//! set of cell-like machine specs into a weighted producer→consumer graph and
//! scores proposed linear orders or contiguous row folds cheaply. Expensive
//! composition, validation, and metering are reserved for the best candidates.

use rustc_hash::FxHashMap;

use crate::models::MachineSpec;

/// A rectangular cell or collapsed mega-cell considered by macro placement.
#[derive(Clone, Debug, PartialEq)]
pub struct MacroNode {
    pub recipe: String,
    pub width: i32,
    pub height: i32,
}

/// One transported item from a producer macro to a consumer macro.
#[derive(Clone, Debug, PartialEq)]
pub struct FlowEdge {
    pub producer: usize,
    pub consumer: usize,
    pub item: String,
    /// Planned rate consumed by this consumer, in items or fluid units / s.
    pub rate: f64,
    pub is_fluid: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlacementGraph {
    pub nodes: Vec<MacroNode>,
    pub edges: Vec<FlowEdge>,
}

/// Cheap placement metrics shared by both RFCs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlacementMetrics {
    pub rate_weighted_distance: f64,
    pub max_edge_distance: i32,
    pub backward_rate: f64,
    pub weighted_cut_sum: f64,
    pub max_weighted_cut: f64,
    pub estimated_width: i32,
    pub estimated_height: i32,
    pub estimated_area: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearCandidate {
    pub order: Vec<usize>,
    pub metrics: PlacementMetrics,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FoldCandidate {
    pub order: Vec<usize>,
    pub row_ends: Vec<usize>,
    pub serpentine: bool,
    pub metrics: PlacementMetrics,
}

impl PlacementGraph {
    /// Build the placement graph from cell-like specs and their estimated
    /// rectangular footprints.
    ///
    /// Each item must have at most one producer macro, matching chain
    /// eligibility. Consumer edge rates come from the consumer spec's total
    /// demand (`per_machine_rate × count`), not producer output, so fan-outs
    /// retain their actual unequal weights.
    pub fn from_specs(
        specs: &[MachineSpec],
        dimensions: &FxHashMap<String, (i32, i32)>,
    ) -> Result<Self, String> {
        let nodes: Vec<MacroNode> = specs
            .iter()
            .map(|m| {
                let (width, height) = dimensions.get(&m.recipe).copied().unwrap_or((1, 1));
                MacroNode {
                    recipe: m.recipe.clone(),
                    width: width.max(1),
                    height: height.max(1),
                }
            })
            .collect();

        let mut producer_of: FxHashMap<&str, usize> = FxHashMap::default();
        for (idx, spec) in specs.iter().enumerate() {
            for output in &spec.outputs {
                if let Some(previous) = producer_of.insert(output.item.as_str(), idx) {
                    if previous != idx {
                        return Err(format!(
                            "placement: {} produced by both {} and {}",
                            output.item, specs[previous].recipe, spec.recipe
                        ));
                    }
                }
            }
        }

        let mut edges = Vec::new();
        for (consumer, spec) in specs.iter().enumerate() {
            for input in &spec.inputs {
                let Some(&producer) = producer_of.get(input.item.as_str()) else {
                    continue; // external input
                };
                if producer == consumer {
                    continue; // self-loop stays inside the macro
                }
                edges.push(FlowEdge {
                    producer,
                    consumer,
                    item: input.item.clone(),
                    rate: input.rate * spec.count,
                    is_fluid: input.is_fluid,
                });
            }
        }

        Ok(Self { nodes, edges })
    }

    /// Score a left-to-right order using macro centres and a fixed inter-slot
    /// gap. `fluid_weight` is deliberately explicit: fluid distance consumes
    /// geometry but not belt-style in-flight item inventory.
    pub fn score_linear(
        &self,
        order: &[usize],
        gap: i32,
        fluid_weight: f64,
    ) -> Result<PlacementMetrics, String> {
        self.validate_order(order)?;
        let mut centre_x = vec![0i32; self.nodes.len()];
        let mut cursor = 0i32;
        let mut max_height = 0i32;
        for &idx in order {
            let node = &self.nodes[idx];
            centre_x[idx] = cursor + node.width / 2;
            cursor += node.width + gap.max(0);
            max_height = max_height.max(node.height);
        }
        let width = (cursor - gap.max(0)).max(0);

        let mut metrics = PlacementMetrics {
            estimated_width: width,
            estimated_height: max_height,
            estimated_area: i64::from(width) * i64::from(max_height),
            ..Default::default()
        };
        for edge in &self.edges {
            let signed = centre_x[edge.consumer] - centre_x[edge.producer];
            let distance = signed.abs();
            let weight = edge.rate * if edge.is_fluid { fluid_weight } else { 1.0 };
            metrics.rate_weighted_distance += weight * f64::from(distance);
            metrics.max_edge_distance = metrics.max_edge_distance.max(distance);
            if signed < 0 {
                metrics.backward_rate += edge.rate;
            }
        }

        // Weighted cut: total rate crossing every boundary between adjacent
        // order positions. This is both a congestion estimate for RFC-055 and
        // the fold-point input for RFC-056.
        let mut position = vec![0usize; self.nodes.len()];
        for (pos, &idx) in order.iter().enumerate() {
            position[idx] = pos;
        }
        for cut in 0..order.len().saturating_sub(1) {
            let mut weight = 0.0;
            for edge in &self.edges {
                let a = position[edge.producer];
                let b = position[edge.consumer];
                if a.min(b) <= cut && a.max(b) > cut {
                    weight += edge.rate * if edge.is_fluid { fluid_weight } else { 1.0 };
                }
            }
            metrics.weighted_cut_sum += weight;
            metrics.max_weighted_cut = metrics.max_weighted_cut.max(weight);
        }
        Ok(metrics)
    }

    /// Score contiguous rows of one logical order. Rows alternate direction
    /// when `serpentine` is true. This is a geometric estimator for RFC-056,
    /// not a claim that the current router can yet realize the candidate.
    pub fn score_folded(
        &self,
        order: &[usize],
        row_ends: &[usize],
        gap_x: i32,
        gap_y: i32,
        fluid_weight: f64,
        serpentine: bool,
    ) -> Result<PlacementMetrics, String> {
        self.validate_order(order)?;
        if row_ends.is_empty() || *row_ends.last().unwrap() != order.len() {
            return Err("placement: row_ends must terminate at order length".into());
        }
        if row_ends.windows(2).any(|w| w[0] >= w[1]) || row_ends[0] == 0 {
            return Err("placement: row_ends must be strictly increasing and nonzero".into());
        }

        let mut centres = vec![(0i32, 0i32); self.nodes.len()];
        let mut start = 0usize;
        let mut y = 0i32;
        let mut total_width = 0i32;
        let mut total_height = 0i32;
        for (row, &end) in row_ends.iter().enumerate() {
            let slice = &order[start..end];
            let row_height = slice
                .iter()
                .map(|&i| self.nodes[i].height)
                .max()
                .unwrap_or(1);
            let row_width = slice.iter().map(|&i| self.nodes[i].width).sum::<i32>()
                + gap_x.max(0) * slice.len().saturating_sub(1) as i32;
            let reverse = serpentine && row % 2 == 1;
            let mut x = if reverse { row_width } else { 0 };
            for &idx in slice {
                let width = self.nodes[idx].width;
                if reverse {
                    x -= width;
                    centres[idx] = (x + width / 2, y + self.nodes[idx].height / 2);
                    x -= gap_x.max(0);
                } else {
                    centres[idx] = (x + width / 2, y + self.nodes[idx].height / 2);
                    x += width + gap_x.max(0);
                }
            }
            total_width = total_width.max(row_width);
            y += row_height;
            total_height = y;
            if end != order.len() {
                y += gap_y.max(0);
                total_height = y;
            }
            start = end;
        }

        let mut metrics = PlacementMetrics {
            estimated_width: total_width,
            estimated_height: total_height,
            estimated_area: i64::from(total_width) * i64::from(total_height),
            ..Default::default()
        };
        for edge in &self.edges {
            let (px, py) = centres[edge.producer];
            let (cx, cy) = centres[edge.consumer];
            let distance = (cx - px).abs() + (cy - py).abs();
            let weight = edge.rate * if edge.is_fluid { fluid_weight } else { 1.0 };
            metrics.rate_weighted_distance += weight * f64::from(distance);
            metrics.max_edge_distance = metrics.max_edge_distance.max(distance);
            if cx < px {
                metrics.backward_rate += edge.rate;
            }
        }

        // Row boundaries are the cuts RFC-056 must carry through inter-row
        // trunks. Preserve both total and worst weighted cut explicitly.
        let mut position = vec![0usize; self.nodes.len()];
        for (pos, &idx) in order.iter().enumerate() {
            position[idx] = pos;
        }
        for &end in row_ends.iter().take(row_ends.len().saturating_sub(1)) {
            let cut = end - 1;
            let mut weight = 0.0;
            for edge in &self.edges {
                let a = position[edge.producer];
                let b = position[edge.consumer];
                if a.min(b) <= cut && a.max(b) > cut {
                    weight += edge.rate * if edge.is_fluid { fluid_weight } else { 1.0 };
                }
            }
            metrics.weighted_cut_sum += weight;
            metrics.max_weighted_cut = metrics.max_weighted_cut.max(weight);
        }
        Ok(metrics)
    }

    /// Deterministic RFC-055 baseline: repeatedly take the best improving
    /// adjacent swap until reaching a local optimum.
    pub fn improve_adjacent(
        &self,
        initial: &[usize],
        gap: i32,
        fluid_weight: f64,
    ) -> Result<LinearCandidate, String> {
        let mut order = initial.to_vec();
        let mut metrics = self.score_linear(&order, gap, fluid_weight)?;
        loop {
            let mut best: Option<(usize, PlacementMetrics)> = None;
            for i in 0..order.len().saturating_sub(1) {
                order.swap(i, i + 1);
                let candidate = self.score_linear(&order, gap, fluid_weight)?;
                order.swap(i, i + 1);
                if candidate.rate_weighted_distance + 1e-9 < metrics.rate_weighted_distance
                    && best.as_ref().is_none_or(|(_, current)| {
                        candidate.rate_weighted_distance + 1e-9 < current.rate_weighted_distance
                    })
                {
                    best = Some((i, candidate));
                }
            }
            let Some((i, improved)) = best else {
                break;
            };
            order.swap(i, i + 1);
            metrics = improved;
        }
        Ok(LinearCandidate { order, metrics })
    }

    /// Deterministic best-improving single-node relocation. This explores a
    /// materially wider neighbourhood than adjacent swaps while remaining
    /// cheap for the small macro graphs used by cell chains.
    pub fn improve_relocate(
        &self,
        initial: &[usize],
        gap: i32,
        fluid_weight: f64,
    ) -> Result<LinearCandidate, String> {
        self.validate_order(initial)?;
        let mut order = initial.to_vec();
        let mut metrics = self.score_linear(&order, gap, fluid_weight)?;
        loop {
            let mut best: Option<(Vec<usize>, PlacementMetrics)> = None;
            for from in 0..order.len() {
                for to in 0..order.len() {
                    if from == to {
                        continue;
                    }
                    let mut candidate_order = order.clone();
                    let node = candidate_order.remove(from);
                    candidate_order.insert(to, node);
                    let candidate = self.score_linear(&candidate_order, gap, fluid_weight)?;
                    if candidate.rate_weighted_distance + 1e-9
                        < best.as_ref().map_or(metrics.rate_weighted_distance, |(_, m)| {
                            m.rate_weighted_distance
                        })
                    {
                        best = Some((candidate_order, candidate));
                    }
                }
            }
            let Some((improved_order, improved_metrics)) = best else {
                break;
            };
            order = improved_order;
            metrics = improved_metrics;
        }
        Ok(LinearCandidate { order, metrics })
    }

    /// RFC-055 bounded deterministic competitor set: improve the supplied
    /// control and its reverse with both relocation and adjacent-swap
    /// neighbourhoods, then return the lowest weighted-distance result.
    pub fn best_linear(
        &self,
        control: &[usize],
        gap: i32,
        fluid_weight: f64,
    ) -> Result<LinearCandidate, String> {
        self.validate_order(control)?;
        let mut starts = vec![control.to_vec()];
        let mut reverse = control.to_vec();
        reverse.reverse();
        starts.push(reverse);
        let mut best: Option<LinearCandidate> = None;
        for start in starts {
            let relocated = self.improve_relocate(&start, gap, fluid_weight)?;
            let candidate = self.improve_adjacent(&relocated.order, gap, fluid_weight)?;
            if best.as_ref().is_none_or(|current| {
                candidate.metrics.rate_weighted_distance + 1e-9
                    < current.metrics.rate_weighted_distance
            }) {
                best = Some(candidate);
            }
        }
        Ok(best.expect("two deterministic starts"))
    }

    /// Exact RFC-056 two-row baseline for a fixed logical order. Every
    /// non-empty contiguous split is evaluated in serpentine mode and,
    /// optionally, with both rows facing the same direction.
    #[allow(clippy::too_many_arguments)]
    pub fn best_two_row_fold(
        &self,
        order: &[usize],
        gap_x: i32,
        gap_y: i32,
        fluid_weight: f64,
        inter_row_weight: f64,
        include_same_direction: bool,
    ) -> Result<FoldCandidate, String> {
        self.validate_order(order)?;
        if order.len() < 2 {
            return Err("placement: two-row fold needs at least two nodes".into());
        }
        let orientations: &[bool] = if include_same_direction {
            &[false, true]
        } else {
            &[true]
        };
        let mut best: Option<(f64, FoldCandidate)> = None;
        for end in 1..order.len() {
            for &serpentine in orientations {
                let row_ends = vec![end, order.len()];
                let metrics =
                    self.score_folded(order, &row_ends, gap_x, gap_y, fluid_weight, serpentine)?;
                let objective =
                    metrics.rate_weighted_distance + inter_row_weight * metrics.weighted_cut_sum;
                let candidate = FoldCandidate {
                    order: order.to_vec(),
                    row_ends,
                    serpentine,
                    metrics,
                };
                if best
                    .as_ref()
                    .is_none_or(|(score, _)| objective + 1e-9 < *score)
                {
                    best = Some((objective, candidate));
                }
            }
        }
        Ok(best.expect("non-empty split search").1)
    }

    fn validate_order(&self, order: &[usize]) -> Result<(), String> {
        if order.len() != self.nodes.len() {
            return Err(format!(
                "placement: order has {} nodes, expected {}",
                order.len(),
                self.nodes.len()
            ));
        }
        let mut seen = vec![false; self.nodes.len()];
        for &idx in order {
            let Some(slot) = seen.get_mut(idx) else {
                return Err(format!("placement: node index {idx} out of range"));
            };
            if std::mem::replace(slot, true) {
                return Err(format!("placement: node index {idx} repeated"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ItemFlow;

    fn spec(
        recipe: &str,
        count: f64,
        inputs: &[(&str, f64)],
        outputs: &[(&str, f64)],
    ) -> MachineSpec {
        MachineSpec {
            recipe: recipe.into(),
            count,
            inputs: inputs
                .iter()
                .map(|(item, rate)| ItemFlow {
                    item: (*item).into(),
                    rate: *rate,
                    ..Default::default()
                })
                .collect(),
            outputs: outputs
                .iter()
                .map(|(item, rate)| ItemFlow {
                    item: (*item).into(),
                    rate: *rate,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn fixture() -> PlacementGraph {
        let specs = vec![
            spec("plate", 1.0, &[], &[("plate", 10.0)]),
            spec("plastic", 1.0, &[], &[("plastic", 4.0)]),
            spec(
                "lds",
                1.0,
                &[("plate", 10.0), ("plastic", 4.0)],
                &[("lds", 1.0)],
            ),
            spec("science", 1.0, &[("lds", 1.0)], &[("science", 1.0)]),
        ];
        let dims = [
            ("plate".into(), (10, 4)),
            ("plastic".into(), (6, 4)),
            ("lds".into(), (12, 6)),
            ("science".into(), (8, 4)),
        ]
        .into_iter()
        .collect();
        PlacementGraph::from_specs(&specs, &dims).unwrap()
    }

    #[test]
    fn graph_weights_consumers_by_total_demand() {
        let g = fixture();
        assert_eq!(g.edges.len(), 3);
        assert_eq!(
            g.edges.iter().find(|e| e.item == "plate").unwrap().rate,
            10.0
        );
    }

    #[test]
    fn closer_high_rate_consumer_scores_better() {
        let g = fixture();
        let compact = g.score_linear(&[0, 2, 1, 3], 2, 0.25).unwrap();
        let sprawling = g.score_linear(&[0, 1, 3, 2], 2, 0.25).unwrap();
        assert!(
            compact.rate_weighted_distance < sprawling.rate_weighted_distance,
            "{compact:?} vs {sprawling:?}"
        );
    }

    #[test]
    fn folded_score_reports_inter_row_cut() {
        let g = fixture();
        let m = g
            .score_folded(&[0, 1, 2, 3], &[2, 4], 2, 5, 0.25, true)
            .unwrap();
        assert!(m.weighted_cut_sum > 0.0);
        assert!(m.max_weighted_cut > 0.0);
        assert!(m.estimated_height > 6);
    }

    #[test]
    fn malformed_orders_and_folds_refuse() {
        let g = fixture();
        assert!(g.score_linear(&[0, 1, 1, 3], 2, 0.25).is_err());
        assert!(g
            .score_folded(&[0, 1, 2, 3], &[2, 3], 2, 5, 0.25, true)
            .is_err());
    }

    #[test]
    fn adjacent_improvement_is_deterministic_and_non_regressing() {
        let g = fixture();
        let initial = [0, 3, 1, 2];
        let before = g.score_linear(&initial, 2, 0.25).unwrap();
        let a = g.improve_adjacent(&initial, 2, 0.25).unwrap();
        let b = g.improve_adjacent(&initial, 2, 0.25).unwrap();
        assert_eq!(a, b);
        assert!(a.metrics.rate_weighted_distance <= before.rate_weighted_distance);
    }

    #[test]
    fn relocation_search_is_deterministic_and_beats_control() {
        let g = fixture();
        let initial = [0, 3, 1, 2];
        let before = g.score_linear(&initial, 2, 0.25).unwrap();
        let a = g.best_linear(&initial, 2, 0.25).unwrap();
        let b = g.best_linear(&initial, 2, 0.25).unwrap();
        assert_eq!(a, b);
        assert!(a.metrics.rate_weighted_distance < before.rate_weighted_distance);
    }

    #[test]
    fn best_two_row_fold_checks_every_cut_and_orientation() {
        let g = fixture();
        let candidate = g
            .best_two_row_fold(&[0, 1, 2, 3], 2, 5, 0.25, 2.0, true)
            .unwrap();
        assert_eq!(*candidate.row_ends.last().unwrap(), 4);
        assert_eq!(candidate.row_ends.len(), 2);
        assert!(candidate.row_ends[0] > 0 && candidate.row_ends[0] < 4);
    }
}
