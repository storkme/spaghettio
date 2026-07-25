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

// ── full cell: I/O belts around the coupling ────────────────────────────────

/// The belts and inserters that surround a cell's coupling: the producer's
/// solid input arriving on a belt, and the consumer's product leaving on
/// one. Phase 1's scope is deliberately narrow — the consumer's ONLY solid
/// input is the directly-inserted item, so it needs no input belt at all,
/// which is exactly what makes the cell tight.
#[derive(Debug, Clone)]
pub struct DiCellIo<'a> {
    pub input_item: &'a str,
    pub input_belt: &'a str,
    /// Inserter carrying `input_item` from the input belt into a producer.
    pub feed_inserter: &'a str,
    pub output_item: &'a str,
    pub output_belt: &'a str,
    /// Inserter carrying `output_item` from a consumer onto the output belt.
    pub out_inserter: &'a str,
}

/// A stamped cell plus the row geometry a caller needs to register it with
/// the lane planner (`input_belt_y` for tap-off, `output_belt_y` for the
/// bus, and the x-extent).
#[derive(Debug, Clone)]
pub struct DiCellLayout {
    pub entities: Vec<PlacedEntity>,
    pub input_belt_y: i32,
    pub producer_y: i32,
    pub band_y: i32,
    pub consumer_y: i32,
    pub output_belt_y: i32,
    pub x_min: i32,
    pub x_max: i32,
}

impl DiCellLayout {
    pub fn height(&self) -> i32 {
        self.output_belt_y - self.input_belt_y + 1
    }
}

/// Stamp a complete DI cell: input belt, feed inserters, producers, the
/// one-tile DI band, consumers, output inserters, output belt.
///
/// Vertical layout for `h`-tall machines (every inserter is reach-1 and
/// faces south, so each picks from the band above it and drops into the
/// band below):
/// ```text
///   y0            input belt        (flows east)
///   y0+1          feed inserters    belt   → producer
///   y0+2 ..       producer machines
///   y0+2+h        DI band           producer → consumer
///   y0+3+h ..     consumer machines
///   y0+3+2h       output inserters  consumer → belt
///   y0+4+2h       output belt       (flows east)
/// ```
///
/// Returns `None` for the same reasons [`stamp_di_cell`] does.
pub fn stamp_di_cell_io(
    plan: &StraddlePlan,
    spec: &DiCellSpec<'_>,
    io: &DiCellIo<'_>,
    x0: i32,
    y0: i32,
    machine_w: i32,
    machine_h: i32,
) -> Option<DiCellLayout> {
    if machine_w <= 0 || machine_h <= 0 {
        return None;
    }
    let input_belt_y = y0;
    let feed_y = y0 + 1;
    let producer_y = y0 + 2;
    let coupling = stamp_di_cell(plan, spec, x0, producer_y, machine_h)?;
    let band_y = producer_y + machine_h;
    let consumer_y = band_y + 1;
    let out_ins_y = consumer_y + machine_h;
    let output_belt_y = out_ins_y + 1;

    let x_min = x0;
    let x_max = x0 + plan.width(machine_w) - 1;
    let seg = format!("di-cell:{}:{}", spec.item, spec.consumer_recipe);

    let mut ents = coupling;
    // Belts span the whole cell so every machine on either row can be
    // served, and so the lane planner sees one contiguous run per side.
    for x in x_min..=x_max {
        ents.push(PlacedEntity {
            name: io.input_belt.to_string(),
            x,
            y: input_belt_y,
            direction: EntityDirection::East,
            carries: Some(io.input_item.to_string()),
            segment_id: Some(format!("{seg}:belt-in:{}", io.input_item)),
            ..Default::default()
        });
        ents.push(PlacedEntity {
            name: io.output_belt.to_string(),
            x,
            y: output_belt_y,
            direction: EntityDirection::East,
            carries: Some(io.output_item.to_string()),
            segment_id: Some(format!("{seg}:belt-out")),
            ..Default::default()
        });
    }
    // One feed inserter per producer and one output inserter per consumer,
    // at the machine's middle column so a 1-wide machine still works.
    let mid = machine_w / 2;
    for &px in &plan.producer_xs {
        ents.push(PlacedEntity {
            name: io.feed_inserter.to_string(),
            x: x0 + px + mid,
            y: feed_y,
            direction: EntityDirection::South,
            carries: Some(io.input_item.to_string()),
            segment_id: Some(format!("{seg}:inserter-in")),
            ..Default::default()
        });
    }
    for &cx in &plan.consumer_xs {
        ents.push(PlacedEntity {
            name: io.out_inserter.to_string(),
            x: x0 + cx + mid,
            y: out_ins_y,
            direction: EntityDirection::South,
            carries: Some(io.output_item.to_string()),
            segment_id: Some(format!("{seg}:inserter-out")),
            ..Default::default()
        });
    }

    Some(DiCellLayout {
        entities: ents,
        input_belt_y,
        producer_y,
        band_y,
        consumer_y,
        output_belt_y,
        x_min,
        x_max,
    })
}

#[cfg(test)]
mod io_tests {
    use super::*;
    use crate::common::{dir_to_vec, entity_size, inserter_reach, machine_feed_rate, QualityTier};
    use rustc_hash::{FxHashMap, FxHashSet};

    fn build() -> DiCellLayout {
        let plan = plan_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        let rate = machine_feed_rate("stack-inserter", QualityTier::Normal, 0);
        let spec = DiCellSpec {
            producer_entity: "assembling-machine-3",
            consumer_entity: "assembling-machine-3",
            producer_recipe: "copper-cable",
            consumer_recipe: "electronic-circuit",
            item: "copper-cable",
            inserter: "stack-inserter",
            inserter_rate: rate,
        };
        let io = DiCellIo {
            input_item: "copper-plate",
            input_belt: "transport-belt",
            feed_inserter: "stack-inserter",
            output_item: "electronic-circuit",
            output_belt: "transport-belt",
            out_inserter: "stack-inserter",
        };
        stamp_di_cell_io(&plan, &spec, &io, 0, 0, 3, 3).expect("full cell must stamp")
    }

    /// Every inserter in the cell must be reach-1 and connect the two
    /// things it is meant to: belt→producer, producer→consumer,
    /// consumer→belt. This is the whole cell's correctness in one test.
    #[test]
    fn every_inserter_connects_its_intended_pair() {
        let cell = build();
        let mut machine_role: FxHashMap<(i32, i32), &str> = FxHashMap::default();
        for e in cell.entities.iter().filter(|e| e.name.starts_with("assembling")) {
            let (w, h) = entity_size(&e.name);
            let role = if e.recipe.as_deref() == Some("copper-cable") { "producer" } else { "consumer" };
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    machine_role.insert((e.x + dx, e.y + dy), role);
                }
            }
        }
        let belt_at: FxHashMap<(i32, i32), &str> = cell
            .entities
            .iter()
            .filter(|e| e.name.contains("transport-belt"))
            .map(|e| ((e.x, e.y), e.carries.as_deref().unwrap_or("")))
            .collect();

        for ins in cell.entities.iter().filter(|e| e.name.contains("inserter")) {
            let (dx, dy) = dir_to_vec(ins.direction);
            let r = inserter_reach(&ins.name);
            assert_eq!(r, 1, "cell inserters must be reach-1");
            let pick = (ins.x - dx * r, ins.y - dy * r);
            let drop = (ins.x + dx * r, ins.y + dy * r);
            let seg = ins.segment_id.clone().unwrap_or_default();
            if seg.ends_with(":inserter-in") {
                assert_eq!(belt_at.get(&pick).copied(), Some("copper-plate"), "feed picks from input belt");
                assert_eq!(machine_role.get(&drop).copied(), Some("producer"), "feed drops into producer");
            } else if seg.ends_with(":inserter-out") {
                assert_eq!(machine_role.get(&pick).copied(), Some("consumer"), "output picks from consumer");
                assert_eq!(belt_at.get(&drop).copied(), Some("electronic-circuit"), "output drops on out belt");
            } else {
                assert_eq!(machine_role.get(&pick).copied(), Some("producer"), "DI picks from producer");
                assert_eq!(machine_role.get(&drop).copied(), Some("consumer"), "DI drops into consumer");
            }
        }
    }

    /// No two entities may occupy the same tile (machines expanded).
    #[test]
    fn cell_has_no_overlaps() {
        let cell = build();
        let mut seen: FxHashSet<(i32, i32)> = FxHashSet::default();
        for e in &cell.entities {
            let (w, h) = if e.name.starts_with("assembling") { entity_size(&e.name) } else { (1, 1) };
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    assert!(seen.insert((e.x + dx, e.y + dy)), "overlap at {:?} ({})", (e.x + dx, e.y + dy), e.name);
                }
            }
        }
    }

    /// Both belts are contiguous runs flowing the same way — the lane
    /// planner and the belt-connectivity check both rely on that.
    #[test]
    fn belts_are_contiguous_single_direction_runs() {
        let cell = build();
        for (y, item) in [(cell.input_belt_y, "copper-plate"), (cell.output_belt_y, "electronic-circuit")] {
            let mut xs: Vec<i32> = cell
                .entities
                .iter()
                .filter(|e| e.y == y && e.name.contains("transport-belt"))
                .map(|e| {
                    assert_eq!(e.direction, EntityDirection::East);
                    assert_eq!(e.carries.as_deref(), Some(item));
                    e.x
                })
                .collect();
            xs.sort_unstable();
            assert_eq!(xs.first().copied(), Some(cell.x_min));
            assert_eq!(xs.last().copied(), Some(cell.x_max));
            for w in xs.windows(2) {
                assert_eq!(w[1] - w[0], 1, "belt run at y={y} has a hole");
            }
        }
    }

    /// The full cell — belts included — is 11 tiles for 3-tall machines,
    /// against the bus's measured 17 for the same coupling (KC4).
    #[test]
    fn full_cell_is_shorter_than_the_bus_baseline() {
        let cell = build();
        assert_eq!(cell.height(), 11);
        assert_eq!(
            (cell.input_belt_y, cell.producer_y, cell.band_y, cell.consumer_y, cell.output_belt_y),
            (0, 2, 5, 6, 10)
        );
        assert!(cell.height() < 17, "KC4: cell must stay under the 17-tile bus baseline");
    }
}

// ---------------------------------------------------------------------------
// RFC-053 Phase 2: horizontal row straddle (the corpus's dominant shape)
// ---------------------------------------------------------------------------

/// A producer/consumer sequence laid out in ONE horizontal row, coupled by
/// inserters in the gaps between horizontally-adjacent machines.
///
/// This is the shape the corpus actually builds: `di-patterns faces` puts
/// `DI@E+W | S:in1→belt S:out1→belt` at the top of the cable→EC face-plan
/// distribution (177 of 2,039 consumers) — a consumer straddling two
/// producers east and west, with its remaining input and its output both
/// on ONE face, both reach-1, and the opposite face entirely free.
///
/// Why it is worth preferring over Phase 1's stacked cell: it needs no
/// reach-2 inserter anywhere, it leaves a whole face free, and a row of
/// machines with belts above and below is what `place_rows` already
/// emits — so it reuses the existing row mechanism rather than replacing
/// it.
#[derive(Debug, Clone, PartialEq)]
pub struct RowCellPlan {
    /// Machines left to right. `true` = producer, `false` = consumer.
    pub sequence: Vec<bool>,
    /// x of each machine in `sequence`, at `machine_w + 1` pitch (the
    /// 1-tile gap is where the coupling inserter goes).
    pub xs: Vec<i32>,
    /// `(sequence_idx_producer, sequence_idx_consumer, flow)`. Every edge
    /// is between physically ADJACENT machines — that is the invariant the
    /// arrangement exists to guarantee.
    pub edges: Vec<(usize, usize, f64)>,
}

impl RowCellPlan {
    /// Rate one coupling inserter must sustain, i.e. the busiest edge.
    /// Each edge owns exactly one gap, so there is no slot division here —
    /// unlike the stacked cell, where an edge can own several columns.
    pub fn required_rate(&self) -> f64 {
        self.edges.iter().map(|&(_, _, f)| f).fold(0.0, f64::max)
    }

    pub fn width(&self, machine_w: i32) -> i32 {
        match self.xs.last() {
            Some(&last) => last + machine_w,
            None => 0,
        }
    }
}

/// Arrange `producer_count` producers and `consumer_count` consumers in a
/// single row so that every consumer is physically adjacent to exactly the
/// producers whose flow it takes.
///
/// Construction: producer `i` owns flow interval `[i·prod, (i+1)·prod)` and
/// consumer `j` owns `[j·demand, (j+1)·demand)`; they share an edge where
/// the intervals overlap (the same flow-interval argument `plan_straddle`
/// uses). Ordering the machines by interval start and inserting each
/// consumer between its producers makes every edge adjacent.
///
/// Returns `None` when the shape is out of scope rather than approximating
/// it: unbalanced flow, or a consumer needing more than two producers (it
/// has only two horizontal neighbours, so a third could not reach it).
pub fn plan_row_straddle(
    producer_count: usize,
    consumer_count: usize,
    producer_rate: f64,
    consumer_rate: f64,
    machine_w: i32,
) -> Option<RowCellPlan> {
    if producer_count == 0 || consumer_count == 0 || machine_w <= 0 {
        return None;
    }
    let usable = producer_rate.is_finite()
        && consumer_rate.is_finite()
        && producer_rate > 0.0
        && consumer_rate > 0.0;
    if !usable {
        return None;
    }
    let total_out = producer_rate * producer_count as f64;
    let total_in = consumer_rate * consumer_count as f64;
    let tol = 1e-6 * total_out.max(total_in).max(1.0);
    if (total_out - total_in).abs() > tol {
        return None;
    }

    // Flow-interval overlap → which producers feed each consumer.
    let mut per_consumer: Vec<Vec<(usize, f64)>> = vec![Vec::new(); consumer_count];
    for (j, slot) in per_consumer.iter_mut().enumerate() {
        let (c_lo, c_hi) = (j as f64 * consumer_rate, (j + 1) as f64 * consumer_rate);
        for i in 0..producer_count {
            let (p_lo, p_hi) = (i as f64 * producer_rate, (i + 1) as f64 * producer_rate);
            let flow = p_hi.min(c_hi) - p_lo.max(c_lo);
            if flow > tol {
                slot.push((i, flow));
            }
        }
        // Only two horizontal neighbours exist, so a third producer could
        // never physically reach this consumer. Refuse instead of
        // silently under-feeding (Phase 3 / multi-band territory).
        if slot.len() > 2 || slot.is_empty() {
            return None;
        }
    }

    // Emit left to right: walk producers in order, dropping each consumer
    // in immediately after the FIRST producer that feeds it. A consumer
    // fed by {i, i+1} therefore lands between them; one fed by {i} alone
    // lands directly after it.
    let mut sequence: Vec<bool> = Vec::new();
    let mut prod_slot: Vec<usize> = vec![usize::MAX; producer_count];
    let mut cons_slot: Vec<usize> = vec![usize::MAX; consumer_count];
    let mut next_consumer = 0usize;
    for (i, slot) in prod_slot.iter_mut().enumerate() {
        *slot = sequence.len();
        sequence.push(true);
        while next_consumer < consumer_count {
            let firsts = per_consumer[next_consumer][0].0;
            let lasts = per_consumer[next_consumer].last().unwrap().0;
            if firsts != i {
                break;
            }
            // A consumer straddling {i, i+1} must wait until i+1 exists to
            // its right; emitting it here places it between them, which is
            // exactly what we want, so only the single-producer case needs
            // the last-producer check.
            if lasts > i + 1 {
                return None;
            }
            cons_slot[next_consumer] = sequence.len();
            sequence.push(false);
            next_consumer += 1;
        }
    }
    if next_consumer != consumer_count {
        return None;
    }

    let pitch = machine_w + 1;
    let xs: Vec<i32> = (0..sequence.len()).map(|k| k as i32 * pitch).collect();

    let mut edges = Vec::new();
    for (j, feeders) in per_consumer.iter().enumerate() {
        let cs = cons_slot[j];
        for &(i, flow) in feeders {
            let ps = prod_slot[i];
            // The invariant this whole construction exists to hold.
            if cs.abs_diff(ps) != 1 {
                return None;
            }
            edges.push((ps, cs, flow));
        }
    }
    Some(RowCellPlan { sequence, xs, edges })
}

#[cfg(test)]
mod row_straddle_tests {
    use super::*;

    /// The canonical cable→EC ratio, hand-derived independently in the RFC
    /// as `P0 C0 P1 C1 P2 P3 C2 P4 C3 P5`. If the construction reproduces
    /// that without being fitted to it, the flow-interval argument holds
    /// in 1-D the same way it does for the stacked cell.
    #[test]
    fn canonical_cable_to_ec_row_matches_the_hand_derivation() {
        let p = plan_row_straddle(6, 4, 5.0, 7.5, 3).expect("6:4 must arrange");
        let seq: String = p
            .sequence
            .iter()
            .map(|&is_p| if is_p { 'P' } else { 'C' })
            .collect();
        assert_eq!(seq, "PCPCPPCPCP", "sequence must match the hand derivation");
        assert_eq!(p.sequence.len(), 10);
        // Pitch 4 = 3-wide machine + the 1-tile coupling gap.
        assert_eq!(p.xs, vec![0, 4, 8, 12, 16, 20, 24, 28, 32, 36]);
        assert_eq!(p.width(3), 39);
    }

    /// The defining property: every coupling is between physically
    /// ADJACENT machines. Without it the plan is unbuildable, since an
    /// inserter only spans one gap.
    #[test]
    fn every_edge_couples_adjacent_machines() {
        for (pc, cc, pr, cr) in [(6, 4, 5.0, 7.5), (4, 4, 2.5, 2.5), (2, 1, 3.0, 6.0)] {
            let p = plan_row_straddle(pc, cc, pr, cr, 3).expect("must arrange");
            for &(ps, cs, _) in &p.edges {
                assert_eq!(ps.abs_diff(cs), 1, "edge {ps}->{cs} is not adjacent in {p:?}");
                assert!(p.sequence[ps], "edge source must be a producer");
                assert!(!p.sequence[cs], "edge target must be a consumer");
            }
        }
    }

    /// Conservation: every consumer receives exactly its demand, and every
    /// producer's output is fully consumed.
    #[test]
    fn flow_is_conserved_per_machine() {
        let (pr, cr) = (5.0, 7.5);
        let p = plan_row_straddle(6, 4, pr, cr, 3).unwrap();
        for (slot, &is_p) in p.sequence.iter().enumerate() {
            let moved: f64 = p
                .edges
                .iter()
                .filter(|&&(ps, cs, _)| if is_p { ps == slot } else { cs == slot })
                .map(|&(_, _, f)| f)
                .sum();
            let want = if is_p { pr } else { cr };
            assert!(
                (moved - want).abs() < 1e-9,
                "slot {slot} moves {moved}, wants {want}"
            );
        }
    }

    /// No reach-2 anywhere: each edge owns exactly one gap, so the busiest
    /// edge IS the per-inserter rate. For cable→EC that is 5.0/s, which a
    /// stack inserter covers at zero research (12.0/s).
    #[test]
    fn required_rate_is_the_busiest_single_edge() {
        let p = plan_row_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        assert_eq!(p.required_rate(), 5.0);
    }

    /// Out-of-scope shapes refuse rather than approximate.
    #[test]
    fn out_of_scope_shapes_are_refused() {
        // Unbalanced flow.
        assert!(plan_row_straddle(6, 4, 5.0, 9.0, 3).is_none());
        // A consumer needing three producers has only two neighbours.
        assert!(plan_row_straddle(9, 3, 1.0, 3.0, 3).is_none());
        // Degenerate inputs.
        assert!(plan_row_straddle(0, 4, 5.0, 7.5, 3).is_none());
        assert!(plan_row_straddle(6, 4, f64::NAN, 7.5, 3).is_none());
        assert!(plan_row_straddle(6, 4, 5.0, 7.5, 0).is_none());
    }

    /// 1:1 pairing (the furnace→furnace shape) interleaves strictly.
    #[test]
    fn one_to_one_interleaves() {
        let p = plan_row_straddle(4, 4, 2.5, 2.5, 3).unwrap();
        let seq: String = p.sequence.iter().map(|&x| if x { 'P' } else { 'C' }).collect();
        assert_eq!(seq, "PCPCPCPC");
        assert_eq!(p.edges.len(), 4, "1:1 needs exactly one edge per pair");
    }
}

/// Entities and row geometry for a stamped [`RowCellPlan`].
#[derive(Debug, Clone)]
pub struct RowCellLayout {
    pub entities: Vec<PlacedEntity>,
    /// Input belt rows, parallel to the caller's solid-input order.
    pub input_belt_ys: Vec<i32>,
    pub machine_y: i32,
    pub output_belt_y: i32,
    pub x_min: i32,
    pub x_max: i32,
}

/// Everything the row stamper needs beyond the plan.
pub struct RowCellSpec<'a> {
    pub producer_entity: &'a str,
    pub consumer_entity: &'a str,
    pub producer_recipe: &'a str,
    pub consumer_recipe: &'a str,
    /// The directly-inserted item.
    pub item: &'a str,
    /// Coupling inserter — reach-1, sits in the 1-tile gap.
    pub coupler: &'a str,
    pub coupler_rate: f64,
    /// `(item, belt_entity, feed_inserter)` for the producer's belt-fed
    /// input, then the consumer's. The producer's belt is the OUTER row
    /// (reached by a reach-2 inserter stepping over the inner belt), the
    /// consumer's is the inner row at reach 1.
    pub producer_input: (&'a str, &'a str, &'a str),
    pub consumer_input: (&'a str, &'a str, &'a str),
    /// Inserters per machine face. The output face needs reach-2 (it steps
    /// over the consumer's input belt), and long-handed is the only
    /// reach-2 inserter (I8a) at 2.40/s at L2 — under the 2.5/s an EC
    /// machine emits — so more than one column is the NORMAL case here,
    /// not an edge case. Ignoring these counts silently under-feeds.
    pub producer_feed_count: usize,
    pub consumer_feed_count: usize,
    pub out_count: usize,
    pub output_item: &'a str,
    pub output_belt: &'a str,
    pub out_inserter: &'a str,
}

/// Stamp a horizontal row cell with its belts.
///
/// ```text
///   y0                  producer input belt   (outer, reach-2 feed)
///   y1                  consumer input belt   (inner, reach-1 feed)
///   y2                  feed inserters
///   y3 .. y3+h-1        machines, interleaved P/C at plan.xs
///   y3+h                output inserters
///   y3+h+1              output belt
/// ```
///
/// Couplers sit in the gap columns BETWEEN machines, at reach 1 — the
/// property that makes this DI at all. Returns `None` if the coupler
/// cannot carry the busiest edge, rather than emitting an under-fed row.
pub fn stamp_row_cell(
    plan: &RowCellPlan,
    spec: &RowCellSpec<'_>,
    x0: i32,
    y0: i32,
    machine_w: i32,
    machine_h: i32,
) -> Option<RowCellLayout> {
    if machine_w <= 0 || machine_h <= 0 {
        return None;
    }
    if plan.required_rate() > spec.coupler_rate + 1e-9 {
        return None;
    }
    // The producer's belt is NORTH at reach-1; both consumer flows go
    // SOUTH. Putting the producer's feed on a reach-2 hop instead would
    // reintroduce the long-handed 2.40/s ceiling the row shape exists to
    // avoid, and it is the arrangement the corpus shows
    // (`DI@E+W | S:in1 S:out1`, the top cable->EC face plan).
    //
    //   y0                  producer input belt   (north)
    //   y1                  producer feed         reach-1
    //   y2 .. y2+h-1        machines, interleaved
    //   y2+h                face row: consumer feed (reach-1, picks the
    //                       belt below) + output (reach-2, steps over it)
    //   y2+h+1              consumer input belt
    //   y2+h+2              output belt
    let p_belt_y = y0;
    let p_feed_y = y0 + 1;
    let machine_y = y0 + 2;
    let face_y = machine_y + machine_h;
    let c_belt_y = face_y + 1;
    let output_belt_y = face_y + 2;
    let x_min = x0;
    let x_max = x0 + plan.width(machine_w) - 1;
    let seg = format!("di-row:{}:{}", spec.item, spec.consumer_recipe);
    let mut ents = Vec::new();

    let belt_run = |ents: &mut Vec<PlacedEntity>, y: i32, name: &str, carries: &str| {
        for x in x_min..=x_max {
            ents.push(PlacedEntity {
                name: name.to_string(),
                x,
                y,
                direction: EntityDirection::East,
                carries: Some(carries.to_string()),
                segment_id: Some(seg.clone()),
                ..Default::default()
            });
        }
    };
    belt_run(&mut ents, p_belt_y, spec.producer_input.1, spec.producer_input.0);
    belt_run(&mut ents, c_belt_y, spec.consumer_input.1, spec.consumer_input.0);
    belt_run(&mut ents, output_belt_y, spec.output_belt, spec.output_item);

    // Columns available on a face, innermost first so single-inserter
    // faces keep the natural centre-ish position.
    let cols = |n: usize| -> Vec<i32> { (0..machine_w).take(n).collect() };

    for (k, &is_producer) in plan.sequence.iter().enumerate() {
        let mx = x0 + plan.xs[k];
        ents.push(PlacedEntity {
            name: if is_producer { spec.producer_entity } else { spec.consumer_entity }.to_string(),
            x: mx,
            y: machine_y,
            direction: EntityDirection::North,
            recipe: Some(
                if is_producer { spec.producer_recipe } else { spec.consumer_recipe }.to_string(),
            ),
            segment_id: Some(seg.clone()),
            ..Default::default()
        });
        if is_producer {
            // North face, reach-1: picks the belt above, drops into the machine.
            for dx in cols(spec.producer_feed_count.max(1)) {
                ents.push(PlacedEntity {
                    name: spec.producer_input.2.to_string(),
                    x: mx + dx,
                    y: p_feed_y,
                    direction: EntityDirection::South,
                    carries: Some(spec.producer_input.0.to_string()),
                    segment_id: Some(seg.clone()),
                    ..Default::default()
                });
            }
        } else {
            // South face shares one row: reach-1 feeds from the inner belt,
            // reach-2 outputs over it. Feed takes the low columns, output
            // the high ones, so they never contend for a tile.
            let nf = spec.consumer_feed_count.max(1);
            let no = spec.out_count.max(1);
            if nf + no > machine_w as usize {
                return None;
            }
            for dx in 0..nf as i32 {
                ents.push(PlacedEntity {
                    name: spec.consumer_input.2.to_string(),
                    x: mx + dx,
                    y: face_y,
                    direction: EntityDirection::North,
                    carries: Some(spec.consumer_input.0.to_string()),
                    segment_id: Some(seg.clone()),
                    ..Default::default()
                });
            }
            for j in 0..no as i32 {
                ents.push(PlacedEntity {
                    name: spec.out_inserter.to_string(),
                    x: mx + nf as i32 + j,
                    y: face_y,
                    direction: EntityDirection::South,
                    carries: Some(spec.output_item.to_string()),
                    segment_id: Some(seg.clone()),
                    ..Default::default()
                });
            }
        }
    }

    for &(ps, cs, _) in &plan.edges {
        let (gap_x, dir) = if cs == ps + 1 {
            (x0 + plan.xs[ps] + machine_w, EntityDirection::East)
        } else {
            (x0 + plan.xs[cs] + machine_w, EntityDirection::West)
        };
        ents.push(PlacedEntity {
            name: spec.coupler.to_string(),
            x: gap_x,
            y: machine_y + machine_h / 2,
            direction: dir,
            carries: Some(spec.item.to_string()),
            segment_id: Some(seg.clone()),
            ..Default::default()
        });
    }

    Some(RowCellLayout {
        entities: ents,
        input_belt_ys: vec![p_belt_y, c_belt_y],
        machine_y,
        output_belt_y,
        x_min,
        x_max,
    })
}

#[cfg(test)]
mod row_stamp_tests {
    use super::*;
    use crate::common::{dir_to_vec, inserter_reach};

    fn spec() -> RowCellSpec<'static> {
        RowCellSpec {
            producer_entity: "assembling-machine-3",
            consumer_entity: "assembling-machine-3",
            producer_recipe: "copper-cable",
            consumer_recipe: "electronic-circuit",
            item: "copper-cable",
            coupler: "stack-inserter",
            coupler_rate: 12.0,
            // Reach-1: the producer's belt is the NORTH face, adjacent.
            producer_input: ("copper-plate", "transport-belt", "fast-inserter"),
            consumer_input: ("iron-plate", "transport-belt", "fast-inserter"),
            output_item: "electronic-circuit",
            output_belt: "transport-belt",
            out_inserter: "long-handed-inserter",
            producer_feed_count: 1,
            consumer_feed_count: 1,
            out_count: 2,
        }
    }

    fn stamped() -> (RowCellPlan, RowCellLayout) {
        let plan = plan_row_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        let l = stamp_row_cell(&plan, &spec(), 0, 0, 3, 3).unwrap();
        (plan, l)
    }

    /// THE defining DI property, the same predicate `classify.rs` applies
    /// to community blueprints: every coupler picks from a machine tile
    /// and drops into a machine tile, at reach 1, and no belt anywhere
    /// carries the coupled item.
    #[test]
    fn couplers_are_machine_to_machine_and_the_item_never_hits_a_belt() {
        let (_, l) = stamped();
        let mtiles: std::collections::HashSet<(i32, i32)> = l
            .entities
            .iter()
            .filter(|e| e.name.starts_with("assembling-machine"))
            .flat_map(|e| {
                (0..3).flat_map(move |dx| (0..3).map(move |dy| (e.x + dx, e.y + dy)))
            })
            .collect();
        let couplers: Vec<_> = l
            .entities
            .iter()
            .filter(|e| e.carries.as_deref() == Some("copper-cable"))
            .collect();
        assert_eq!(couplers.len(), 8, "6:4 straddle has 8 edges");
        for c in &couplers {
            assert!(c.name.contains("inserter"), "coupled item must move by inserter");
            let r = inserter_reach(&c.name);
            assert_eq!(r, 1, "coupler must be reach-1 (it sits in a 1-tile gap)");
            let (dx, dy) = dir_to_vec(c.direction);
            let pick = (c.x - dx * r, c.y - dy * r);
            let drop = (c.x + dx * r, c.y + dy * r);
            assert!(mtiles.contains(&pick), "coupler {:?} picks off a machine", (c.x, c.y));
            assert!(mtiles.contains(&drop), "coupler {:?} drops into a machine", (c.x, c.y));
        }
        assert!(
            !l.entities.iter().any(|e| e.name.contains("transport-belt")
                && e.carries.as_deref() == Some("copper-cable")),
            "no belt may carry the DI'd item"
        );
    }

    /// Producers put nothing on the output belt; only consumers do.
    #[test]
    fn only_consumers_reach_the_output_belt() {
        let (plan, l) = stamped();
        // Two long-handed output inserters per consumer: EC emits 2.5/s
        // and long-handed belt-drop is 2.40/s at L2, so one column cannot
        // cover a consumer's output.
        let n_consumers = plan.sequence.iter().filter(|&&p| !p).count() * 2;
        let out_ins = l
            .entities
            .iter()
            .filter(|e| e.carries.as_deref() == Some("electronic-circuit") && e.name.contains("inserter"))
            .count();
        assert_eq!(out_ins, n_consumers);
    }

    /// Feed inserters must actually reach their belts: the consumer's is
    /// reach-1 off the inner belt, the producer's reach-2 over it.
    #[test]
    fn feed_inserters_reach_their_belts() {
        let (_, l) = stamped();
        let belt_at: std::collections::HashMap<(i32, i32), String> = l
            .entities
            .iter()
            .filter(|e| e.name.contains("transport-belt"))
            .map(|e| ((e.x, e.y), e.carries.clone().unwrap_or_default()))
            .collect();
        for e in l.entities.iter().filter(|e| {
            e.name.contains("inserter")
                && matches!(e.carries.as_deref(), Some("copper-plate") | Some("iron-plate"))
        }) {
            let r = inserter_reach(&e.name);
            let (dx, dy) = dir_to_vec(e.direction);
            let pick = (e.x - dx * r, e.y - dy * r);
            let carried = e.carries.as_deref().unwrap();
            assert_eq!(
                belt_at.get(&pick).map(String::as_str),
                Some(carried),
                "feed inserter at {:?} must pick {carried} off its belt",
                (e.x, e.y)
            );
        }
    }

    /// No two entities may share a tile.
    #[test]
    fn row_cell_has_no_overlaps() {
        let (_, l) = stamped();
        let mut seen = std::collections::HashSet::new();
        for e in &l.entities {
            let (w, h) = if e.name.starts_with("assembling-machine") { (3, 3) } else { (1, 1) };
            for dx in 0..w {
                for dy in 0..h {
                    assert!(
                        seen.insert((e.x + dx, e.y + dy)),
                        "overlap at {:?} from {}",
                        (e.x + dx, e.y + dy),
                        e.name
                    );
                }
            }
        }
    }

    /// Under-rate couplers refuse rather than under-feed.
    #[test]
    fn under_rate_coupler_refuses() {
        let plan = plan_row_straddle(6, 4, 5.0, 7.5, 3).unwrap();
        let mut s = spec();
        s.coupler = "inserter";
        s.coupler_rate = 0.84;
        assert!(stamp_row_cell(&plan, &s, 0, 0, 3, 3).is_none());
    }
}
