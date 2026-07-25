//! Direct-insertion cell geometry (RFC-053 Phase 1).
//!
//! Plans the **straddle**: where to put a consumer row's machines relative
//! to a producer row's so that every consumer can be fed entirely by
//! machine→machine inserters, with no belt between the rows.
//!
//! The problem, and why the consumer row cannot simply share the
//! producer's pitch: direct insertion is **source-limited, not
//! inserter-limited**. A stack inserter moves 12/s machine-to-machine at
//! zero research, but a `copper-cable` machine only *makes* 5/s while an
//! `electronic-circuit` machine wants 7.5/s — so each consumer must draw
//! from more than one producer, which forces it to sit across a producer
//! boundary. The community corpus shows exactly this signature: lateral
//! offsets of 0, ±1, ±2 dominate real cable→EC builds (see the
//! `di-patterns` miner).
//!
//! Two halves: [`plan_straddle`] works out the geometry and the
//! producer→consumer edges (pure flow + columns, no entities), and
//! [`stamp_di_cell`] turns a plan into placed machines and inserters.
//! Wiring the cell into `place_rows` — belt suppression for the coupled
//! item and the lane-planner skip — is the remaining Phase 1 step.

/// One producer→consumer coupling in a planned cell.
#[derive(Debug, Clone, PartialEq)]
pub struct DiEdge {
    /// Index into the producer row's machines.
    pub producer: usize,
    /// Index into the consumer row's machines.
    pub consumer: usize,
    /// Items/s flowing across this edge.
    pub flow: f64,
    /// Columns where the two machines are vertically adjacent — the tiles
    /// an inserter for this edge may occupy. Edges CANNOT pool slots: an
    /// inserter draws from exactly one producer.
    pub columns: Vec<i32>,
}

impl DiEdge {
    /// Minimum per-inserter rate that makes this edge feasible: the flow
    /// spread across every slot the edge actually owns.
    pub fn required_rate(&self) -> f64 {
        if self.columns.is_empty() {
            f64::INFINITY
        } else {
            self.flow / self.columns.len() as f64
        }
    }
}

/// A planned DI cell: machine positions for both rows plus the coupling
/// edges between them.
#[derive(Debug, Clone, PartialEq)]
pub struct StraddlePlan {
    /// Producer machine origins (x), laid at the row's natural pitch.
    pub producer_xs: Vec<i32>,
    /// Consumer machine origins (x), offset to straddle producer bounds.
    pub consumer_xs: Vec<i32>,
    pub edges: Vec<DiEdge>,
}

impl StraddlePlan {
    /// The binding per-inserter rate for the whole cell — the worst edge.
    /// A cell is feasible iff `machine_feed_rate >= this`.
    pub fn required_rate(&self) -> f64 {
        self.edges.iter().map(|e| e.required_rate()).fold(0.0, f64::max)
    }

    /// Total width in tiles.
    pub fn width(&self, machine_w: i32) -> i32 {
        let max_p = self.producer_xs.iter().copied().max().unwrap_or(0) + machine_w;
        let max_c = self.consumer_xs.iter().copied().max().unwrap_or(0) + machine_w;
        max_p.max(max_c)
    }
}

/// Overlapping columns between two machine spans of width `w`.
fn overlap_columns(a: i32, b: i32, w: i32) -> Vec<i32> {
    let lo = a.max(b);
    let hi = (a + w - 1).min(b + w - 1);
    (lo..=hi).collect()
}

/// Plan a DI cell.
///
/// `producer_rate` is one producer machine's output of the coupled item;
/// `consumer_rate` is one consumer machine's demand for it. Both are
/// per-machine items/s at the utilization the caller already applied.
///
/// Returns `None` when the shape is outside Phase 1's scope rather than
/// guessing:
/// - either row empty, or a non-positive rate;
/// - total production and total demand disagree by more than a rounding
///   epsilon (an unbalanced coupling is not a DI cell — the surplus or
///   deficit has to go somewhere, which is a bus problem);
/// - a consumer would need to straddle more than two producers (Phase 3
///   territory — the multi-band cell);
/// - no integer placement satisfies every consumer's edges without the
///   consumer machines overlapping each other.
pub fn plan_straddle(
    producer_count: usize,
    consumer_count: usize,
    producer_rate: f64,
    consumer_rate: f64,
    machine_w: i32,
) -> Option<StraddlePlan> {
    if producer_count == 0 || consumer_count == 0 || machine_w <= 0 {
        return None;
    }
    // Positive AND finite — written affirmatively so NaN is rejected
    // (every comparison against NaN is false) without the `!(x > 0.0)`
    // idiom clippy rightly dislikes on partially-ordered types.
    let rates_usable = producer_rate.is_finite()
        && consumer_rate.is_finite()
        && producer_rate > 0.0
        && consumer_rate > 0.0;
    if !rates_usable {
        return None;
    }
    let total_out = producer_rate * producer_count as f64;
    let total_in = consumer_rate * consumer_count as f64;
    // Balanced within a rounding epsilon; scale-relative so big cells
    // aren't held to an absolute tolerance.
    if (total_out - total_in).abs() > 1e-6 * total_out.max(total_in).max(1.0) {
        return None;
    }

    let producer_xs: Vec<i32> = (0..producer_count as i32).map(|i| i * machine_w).collect();

    // Edges come from overlapping FLOW intervals: producer i owns
    // [i·prod, (i+1)·prod) of the item stream, consumer j owns
    // [j·demand, (j+1)·demand). Their intersection is the flow that must
    // cross that edge. This is what makes the row balance exactly.
    let mut flows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); consumer_count];
    for (j, f) in flows.iter_mut().enumerate() {
        let (c_lo, c_hi) = (j as f64 * consumer_rate, (j + 1) as f64 * consumer_rate);
        for i in 0..producer_count {
            let (p_lo, p_hi) = (i as f64 * producer_rate, (i + 1) as f64 * producer_rate);
            let shared = c_hi.min(p_hi) - c_lo.max(p_lo);
            if shared > 1e-9 {
                f.push((i, shared));
            }
        }
        // Phase 1 handles a consumer straddling at most two producers.
        if f.len() > 2 {
            return None;
        }
    }

    // Place each consumer. The ideal continuous position maps flow-space
    // to column-space (flow f ↔ column f·w/prod); we then search nearby
    // integers for one that gives every edge at least one column, keeps
    // the machine inside the producer row, and does not collide with the
    // previous consumer. Among the candidates we take the one whose
    // worst edge needs the LOWEST inserter rate — i.e. the placement that
    // splits columns most nearly in proportion to flow, which is what
    // makes the canonical cable→EC cell land on 2 slots for the 5.0/s
    // edge and 1 for the 2.5/s edge.
    let row_end = producer_count as i32 * machine_w;
    let mut consumer_xs: Vec<i32> = Vec::with_capacity(consumer_count);
    for (j, edges) in flows.iter().enumerate() {
        let ideal = (j as f64 * consumer_rate) * machine_w as f64 / producer_rate;
        let lo_bound = consumer_xs.last().map(|p| p + machine_w).unwrap_or(0);
        let mut best: Option<(f64, i32)> = None;
        let start = (ideal.floor() as i32 - machine_w).max(lo_bound);
        let end = (ideal.ceil() as i32 + machine_w).min(row_end - machine_w);
        for x in start..=end {
            if x < lo_bound || x + machine_w > row_end {
                continue;
            }
            // Every edge must own at least one column at this placement.
            let mut worst = 0.0f64;
            let mut ok = true;
            for &(i, flow) in edges {
                let cols = overlap_columns(producer_xs[i], x, machine_w);
                if cols.is_empty() {
                    ok = false;
                    break;
                }
                worst = worst.max(flow / cols.len() as f64);
            }
            if !ok {
                continue;
            }
            if best.is_none_or(|(w, _)| worst < w) {
                best = Some((worst, x));
            }
        }
        consumer_xs.push(best?.1);
    }

    let mut out = Vec::new();
    for (j, edges) in flows.iter().enumerate() {
        for &(i, flow) in edges {
            out.push(DiEdge {
                producer: i,
                consumer: j,
                flow,
                columns: overlap_columns(producer_xs[i], consumer_xs[j], machine_w),
            });
        }
    }

    Some(StraddlePlan { producer_xs, consumer_xs, edges: out })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The RFC's canonical worked example: copper-cable (5.0/s each) into
    /// electronic-circuit (7.5/s each), 6 producers to 4 consumers, AM3
    /// 3 wide. The planner must reproduce the RFC's published geometry
    /// exactly — machine positions, edge set, per-edge slot counts, and
    /// the `>= 2.5/s` feasibility rule.
    #[test]
    fn canonical_cable_to_ec_matches_the_rfc() {
        let p = plan_straddle(6, 4, 5.0, 7.5, 3).expect("canonical cell must plan");

        assert_eq!(p.producer_xs, vec![0, 3, 6, 9, 12, 15]);
        assert_eq!(p.consumer_xs, vec![1, 5, 10, 14], "RFC-053 Design section positions");

        // Eight directed edges, alternating 5.0/2.5 with 2/1 slots.
        assert_eq!(p.edges.len(), 8);
        let summary: Vec<(usize, usize, f64, usize)> = p
            .edges
            .iter()
            .map(|e| (e.producer, e.consumer, e.flow, e.columns.len()))
            .collect();
        assert_eq!(
            summary,
            vec![
                (0, 0, 5.0, 2),
                (1, 0, 2.5, 1),
                (1, 1, 2.5, 1),
                (2, 1, 5.0, 2),
                (3, 2, 5.0, 2),
                (4, 2, 2.5, 1),
                (4, 3, 2.5, 1),
                (5, 3, 5.0, 2),
            ],
            "edge set / slot split must match the RFC's overlap table"
        );

        // The RFC's headline feasibility rule.
        assert!(
            (p.required_rate() - 2.5).abs() < 1e-9,
            "canonical cell must require exactly 2.5/s per inserter, got {}",
            p.required_rate()
        );
    }

    /// Balance is the property that makes the flow-interval construction
    /// correct: every producer ships its full output, every consumer
    /// receives its full demand.
    #[test]
    fn canonical_cell_balances_both_sides() {
        let p = plan_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        for i in 0..6 {
            let shipped: f64 = p.edges.iter().filter(|e| e.producer == i).map(|e| e.flow).sum();
            assert!((shipped - 5.0).abs() < 1e-9, "producer {i} ships {shipped}, want 5.0");
        }
        for j in 0..4 {
            let got: f64 = p.edges.iter().filter(|e| e.consumer == j).map(|e| e.flow).sum();
            assert!((got - 7.5).abs() < 1e-9, "consumer {j} gets {got}, want 7.5");
        }
    }

    /// Feasibility composes with the real rate table: the canonical cell
    /// works at the engine defaults and for a Fast-capped user at the
    /// default research level, but not at true L0 without stack inserters
    /// (RFC-053's tier × level matrix).
    #[test]
    fn canonical_feasibility_matches_the_tier_matrix() {
        use crate::common::{machine_feed_rate, QualityTier};
        let need = plan_straddle(6, 4, 5.0, 7.5, 3).unwrap().required_rate();
        let q = QualityTier::Normal;
        // Stack at any level clears it.
        assert!(machine_feed_rate("stack-inserter", q, 0) >= need);
        // Fast clears it at the L2 engine default but not at raw L0.
        assert!(machine_feed_rate("fast-inserter", q, 2) >= need);
        assert!(machine_feed_rate("fast-inserter", q, 0) < need);
        // Plain inserters need max research.
        assert!(machine_feed_rate("inserter", q, 2) < need);
        assert!(machine_feed_rate("inserter", q, 7) >= need);
    }

    /// A 1:1 coupling needs no straddle at all — each consumer sits
    /// squarely on its producer and owns the full face.
    #[test]
    fn one_to_one_coupling_aligns_without_offset() {
        let p = plan_straddle(3, 3, 4.0, 4.0, 3).expect("1:1 must plan");
        assert_eq!(p.consumer_xs, p.producer_xs);
        assert_eq!(p.edges.len(), 3);
        assert!(p.edges.iter().all(|e| e.columns.len() == 3));
        assert!((p.required_rate() - 4.0 / 3.0).abs() < 1e-9);
    }

    /// Unbalanced couplings are refused, not approximated: the surplus or
    /// deficit has to reach the bus, which is not a DI cell.
    #[test]
    fn unbalanced_coupling_is_refused() {
        assert!(plan_straddle(6, 4, 5.0, 7.0, 3).is_none(), "under-supplied");
        assert!(plan_straddle(6, 4, 5.0, 8.0, 3).is_none(), "over-supplied");
    }

    /// A consumer that would span three producers is Phase 3 territory
    /// (multi-band cell) — refuse rather than emit a cell that cannot be
    /// fed.
    #[test]
    fn triple_straddle_is_out_of_phase1_scope() {
        // 6 producers at 1.0/s feeding 2 consumers at 3.0/s: each consumer
        // spans three producers.
        assert!(plan_straddle(6, 2, 1.0, 3.0, 3).is_none());
    }

    #[test]
    fn degenerate_inputs_are_refused() {
        assert!(plan_straddle(0, 4, 5.0, 7.5, 3).is_none());
        assert!(plan_straddle(6, 0, 5.0, 7.5, 3).is_none());
        assert!(plan_straddle(6, 4, 0.0, 7.5, 3).is_none());
        assert!(plan_straddle(6, 4, 5.0, 7.5, 0).is_none());
    }

    /// Consumer machines must never overlap each other.
    #[test]
    fn consumers_never_collide() {
        for (pc, cc, pr, cr) in [(6, 4, 5.0, 7.5), (3, 3, 4.0, 4.0), (4, 2, 3.0, 6.0)] {
            let Some(p) = plan_straddle(pc, cc, pr, cr, 3) else { continue };
            for w in p.consumer_xs.windows(2) {
                assert!(w[1] - w[0] >= 3, "consumers overlap: {:?}", p.consumer_xs);
            }
        }
    }
}

// ── cell stamping ───────────────────────────────────────────────────────────

use crate::models::{EntityDirection, PlacedEntity};

/// Everything the stamper needs beyond the geometry: which entities to
/// place and how they're labelled.
#[derive(Debug, Clone)]
pub struct DiCellSpec<'a> {
    pub producer_entity: &'a str,
    pub consumer_entity: &'a str,
    pub producer_recipe: &'a str,
    pub consumer_recipe: &'a str,
    /// The directly-inserted item.
    pub item: &'a str,
    /// Inserter entity for the coupling — chosen by the caller's ladder
    /// against [`StraddlePlan::required_rate`]. Must be reach-1: the cell
    /// puts the rows one tile apart, so a long-handed inserter would
    /// reach straight past both machines.
    pub inserter: &'a str,
    /// What one of those inserters moves, items/s, machine→machine.
    pub inserter_rate: f64,
}

/// Stamp a planned DI cell as entities, with its top-left at `(x0, y0)`.
///
/// Layout, for machines `h` tiles tall:
/// ```text
///   y0 .. y0+h-1     producer machines
///   y0+h             inserter band (reach 1: picks north, drops south)
///   y0+h+1 .. +h     consumer machines
/// ```
///
/// The inserter band is exactly ONE tile — that is the whole point of the
/// cell. It is what lets a reach-1 inserter (and therefore a stack
/// inserter, the only high-rate reach-1 class) couple the machines
/// directly, with **no belt for the coupled item on either side**.
///
/// Returns `None` if `machine_h` is non-positive, or if any edge cannot be
/// served within the inserter slots it owns — the caller must then fall
/// back (bridge or bus) rather than emit an under-fed cell.
pub fn stamp_di_cell(
    plan: &StraddlePlan,
    spec: &DiCellSpec<'_>,
    x0: i32,
    y0: i32,
    machine_h: i32,
) -> Option<Vec<PlacedEntity>> {
    let rate_usable = spec.inserter_rate.is_finite() && spec.inserter_rate > 0.0;
    if machine_h <= 0 || !rate_usable {
        return None;
    }
    let band_y = y0 + machine_h;
    let consumer_y = band_y + 1;
    let seg = format!("di-cell:{}:{}", spec.item, spec.consumer_recipe);

    let mut out = Vec::new();
    for &px in &plan.producer_xs {
        out.push(PlacedEntity {
            name: spec.producer_entity.to_string(),
            x: x0 + px,
            y: y0,
            direction: EntityDirection::North,
            recipe: Some(spec.producer_recipe.to_string()),
            segment_id: Some(format!("{seg}:producer")),
            ..Default::default()
        });
    }
    for &cx in &plan.consumer_xs {
        out.push(PlacedEntity {
            name: spec.consumer_entity.to_string(),
            x: x0 + cx,
            y: consumer_y,
            direction: EntityDirection::North,
            recipe: Some(spec.consumer_recipe.to_string()),
            segment_id: Some(format!("{seg}:consumer")),
            ..Default::default()
        });
    }

    // One inserter per slot needed, drawn from the columns that edge owns.
    // Edges cannot borrow each other's columns: an inserter picks from
    // exactly one producer.
    for edge in &plan.edges {
        let needed = (edge.flow / spec.inserter_rate).ceil() as usize;
        if needed > edge.columns.len() {
            return None;
        }
        for &col in edge.columns.iter().take(needed) {
            out.push(PlacedEntity {
                name: spec.inserter.to_string(),
                x: x0 + col,
                y: band_y,
                // South = picks from the tile north (producer), drops to
                // the tile south (consumer), at reach 1.
                direction: EntityDirection::South,
                carries: Some(spec.item.to_string()),
                segment_id: Some(seg.clone()),
                ..Default::default()
            });
        }
    }
    Some(out)
}

#[cfg(test)]
mod stamp_tests {
    use super::*;
    use crate::common::{dir_to_vec, entity_size, inserter_reach, machine_feed_rate, QualityTier};
    use rustc_hash::FxHashMap;

    fn canonical() -> (StraddlePlan, f64) {
        let p = plan_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        let rate = machine_feed_rate("stack-inserter", QualityTier::Normal, 0);
        (p, rate)
    }

    fn spec(rate: f64) -> DiCellSpec<'static> {
        DiCellSpec {
            producer_entity: "assembling-machine-3",
            consumer_entity: "assembling-machine-3",
            producer_recipe: "copper-cable",
            consumer_recipe: "electronic-circuit",
            item: "copper-cable",
            inserter: "stack-inserter",
            inserter_rate: rate,
        }
    }

    /// THE defining property of a DI cell: every inserter picks from a
    /// PRODUCER machine tile and drops into a CONSUMER machine tile — no
    /// belt on either side. This is the same test `classify.rs` applies
    /// when counting direct insertion in community blueprints, so passing
    /// it means the engine now emits what the corpus is full of.
    #[test]
    fn every_inserter_couples_machine_to_machine() {
        let (plan, rate) = canonical();
        let ents = stamp_di_cell(&plan, &spec(rate), 0, 0, 3).expect("canonical cell must stamp");

        let mut machine_at: FxHashMap<(i32, i32), &str> = FxHashMap::default();
        for e in ents.iter().filter(|e| e.name.starts_with("assembling-machine")) {
            let (w, h) = entity_size(&e.name);
            let role = if e.recipe.as_deref() == Some("copper-cable") { "producer" } else { "consumer" };
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    machine_at.insert((e.x + dx, e.y + dy), role);
                }
            }
        }

        let inserters: Vec<_> = ents.iter().filter(|e| e.name.contains("inserter")).collect();
        assert!(!inserters.is_empty());
        for ins in &inserters {
            let (dx, dy) = dir_to_vec(ins.direction);
            let r = inserter_reach(&ins.name);
            assert_eq!(r, 1, "the 1-tile band requires a reach-1 inserter");
            let pick = (ins.x - dx * r, ins.y - dy * r);
            let drop = (ins.x + dx * r, ins.y + dy * r);
            assert_eq!(
                machine_at.get(&pick).copied(),
                Some("producer"),
                "inserter at {:?} must pick from a producer machine",
                (ins.x, ins.y)
            );
            assert_eq!(
                machine_at.get(&drop).copied(),
                Some("consumer"),
                "inserter at {:?} must drop into a consumer machine",
                (ins.x, ins.y)
            );
        }
    }

    /// No belt entity is emitted for the coupled item — the cell removes
    /// the interface rather than bridging it (the distinction from #432).
    #[test]
    fn cell_emits_no_belts() {
        let (plan, rate) = canonical();
        let ents = stamp_di_cell(&plan, &spec(rate), 0, 0, 3).unwrap();
        assert!(
            !ents.iter().any(|e| e.name.contains("belt") || e.name.contains("splitter")),
            "a DI cell must contain no belt for the coupled item"
        );
    }

    /// Geometry: 3-tall machines, a ONE-tile inserter band, consumers
    /// directly below — 7 tiles for the whole coupling.
    #[test]
    fn cell_is_seven_tiles_tall_with_a_one_tile_band() {
        let (plan, rate) = canonical();
        let ents = stamp_di_cell(&plan, &spec(rate), 0, 0, 3).unwrap();
        let prod_y = ents.iter().find(|e| e.recipe.as_deref() == Some("copper-cable")).unwrap().y;
        let band_y = ents.iter().find(|e| e.name.contains("inserter")).unwrap().y;
        let cons_y = ents.iter().find(|e| e.recipe.as_deref() == Some("electronic-circuit")).unwrap().y;
        assert_eq!((prod_y, band_y, cons_y), (0, 3, 4));
        let bottom = cons_y + 3;
        assert_eq!(bottom - prod_y, 7, "coupling height");
    }

    /// At stack tier one inserter per edge suffices (12/s vs a 5.0/s
    /// worst edge), so the canonical cell is exactly 8 inserters — the
    /// count RFC-053's Design section predicts.
    #[test]
    fn canonical_cell_uses_eight_inserters_at_stack_tier() {
        let (plan, rate) = canonical();
        let ents = stamp_di_cell(&plan, &spec(rate), 0, 0, 3).unwrap();
        assert_eq!(ents.iter().filter(|e| e.name.contains("inserter")).count(), 8);
        assert_eq!(ents.iter().filter(|e| e.name.starts_with("assembling")).count(), 10);
    }

    /// A weaker inserter needs more slots per edge — and the cell still
    /// stamps while the edges' own columns can hold them. Fast at the L2
    /// engine default: the 5.0/s edge needs 2 of its 2 slots.
    #[test]
    fn weaker_inserters_fill_more_slots_per_edge() {
        let plan = plan_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        let rate = machine_feed_rate("fast-inserter", QualityTier::Normal, 2);
        let ents = stamp_di_cell(&plan, &spec(rate), 0, 0, 3).unwrap();
        // 4 big edges × 2 + 4 small edges × 1
        assert_eq!(ents.iter().filter(|e| e.name.contains("inserter")).count(), 12);
    }

    /// Below the cell's required rate the stamper REFUSES rather than
    /// emitting an under-fed cell — the caller falls back to bridge/bus.
    #[test]
    fn under_rate_inserter_refuses_to_stamp() {
        let plan = plan_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        let weak = machine_feed_rate("inserter", QualityTier::Normal, 0); // 0.84/s
        assert!(weak < plan.required_rate());
        assert!(stamp_di_cell(&plan, &spec(weak), 0, 0, 3).is_none());
    }

    /// Origin offset shifts the whole cell rigidly.
    #[test]
    fn cell_translates_with_origin() {
        let (plan, rate) = canonical();
        let a = stamp_di_cell(&plan, &spec(rate), 0, 0, 3).unwrap();
        let b = stamp_di_cell(&plan, &spec(rate), 10, 20, 3).unwrap();
        assert_eq!(a.len(), b.len());
        for (p, q) in a.iter().zip(b.iter()) {
            assert_eq!((q.x - p.x, q.y - p.y), (10, 20));
        }
    }
}
