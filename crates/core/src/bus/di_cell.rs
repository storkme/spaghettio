//! Direct-insertion cell geometry (RFC-053 Phase 1).
//!
//! [`DirectInsertion`] is the mode flag; see its docs for why DI is a
//! scored candidate rather than an unconditional default.
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

/// Direct-insertion mode (RFC-053).
///
/// DI is **not** a plain on/off flag, and the reason is measured rather
/// than assumed. Defaulting it to a bare `true` (2026-07-26) broke 8
/// tests against a 100% green baseline: 5 hard validation errors on
/// `tier4_advanced_circuit_from_ore_am2`, an `input-rate-delivery`
/// warning on `tier2_electronic_circuit` — the *flagship* DI pair — and
/// `mil5-ore` failing to lay out at all.
///
/// Nothing about the cells was wrong. **Fusing a pair changes the row
/// structure**, and trunk lane assignment, junction routing and per-lane
/// capacity are all computed against it. `mil5-ore` is the clearest:
/// DI removes a row, stone-brick demand that spanned two trunks lands on
/// one, and 25/s does not fit a 22.5/s lane. The five corpus pairs were
/// verified at SPECIFIC RATES; defaulting on applies DI at every rate to
/// every eligible pair.
///
/// So DI competes as a scored candidate instead, and may only displace
/// the native layout when it is *strictly better* — see
/// `decomposition_search::di_choice`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DirectInsertion {
    /// No DI anywhere. The escape hatch, and what `NativeCandidate`
    /// effectively runs under `Candidate`.
    Off,
    /// Default. The native layout is built DI-free; a separate
    /// `DirectInsertionCandidate` builds the DI variant and wins only on
    /// a strict quality improvement.
    #[default]
    Candidate,
    /// DI applied directly in the native path — the old `true`. Kept for
    /// A/B comparison and for the `di=1` URL, and used internally by
    /// `DirectInsertionCandidate` to produce its variant.
    Forced,
}

/// Which order the dispatcher walks consumers when deciding who claims a
/// contended DI spec (RFC-059).
///
/// A spec may be fused into at most one cell. When a spec is eligible in two
/// couplings the winner is decided by ITERATION ORDER, which was never a
/// decision anyone made — the walk is topological, so the upstream coupling
/// claims and the downstream one is never evaluated.
///
/// This is an option rather than an env var on purpose. The measurement that
/// motivated RFC-059 was taken with an uncommitted scratch flag and is
/// consequently unrecoverable from the repository; a real option is what makes
/// phase 1 reproducible.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DiClaimOrder {
    /// P0: consumers in topological order, so upstream claims.
    ///
    /// The pre-RFC-059 default, kept as an explicit arm so `Downstream` can be
    /// measured against the status quo rather than assumed better than it. It
    /// lost on evidence: see `Downstream`.
    Upstream,
    /// P1, and **the default** since RFC-059's sim close-out: consumers in
    /// reverse topological order, so the downstream coupling claims a contended
    /// spec.
    ///
    /// Chosen on measurement, and the measurement took two rounds because the
    /// first instrument was wrong. Across three machine tiers, `Downstream` is
    /// never worse than `Upstream` on any of the 179 contended corpus targets
    /// and strictly better on 6.
    ///
    /// That was **not** true before #520 fixed the validator's starved-pickup
    /// blind spot: two targets then appeared to favour `Upstream`, which is why
    /// this RFC first concluded that neither order dominates. Those two are the
    /// `small-electric-pole@5` layouts where `Upstream` shipped a factory
    /// measured at 2.52/s against a planned 5.00/s — it was never ahead there,
    /// the instrument just could not see the deficit.
    ///
    /// The deciding evidence is in-game, not the validator, because RFC-059's
    /// own lesson is that a clean validator is not evidence a layout works.
    /// Headless runs on the three flip targets that produce a usable verdict —
    /// the other three are `land-mine` at am1/am2/am3, which returns 0/s under
    /// BOTH arms for reasons not yet understood (#537), so it is evidence
    /// neither way:
    ///
    /// | target, am2 | `Upstream` | `Downstream` |
    /// |---|---|---|
    /// | `small-electric-pole@5` | 139 ents, PASS 5.00/s | 136 ents, PASS 5.00/s |
    /// | `big-electric-pole@1` | 1146 ents, **FAIL 0.51/s** | 1127 ents, **PASS 1.10/s** |
    /// | `medium-electric-pole@5` | 2351 ents, PASS 5.00/s | 2340 ents, PASS 4.98/s |
    ///
    /// `big-electric-pole@1` is the one that settles it: the status quo ships a
    /// factory running at **half its planned rate**, with 43 machines working
    /// against 96 under `Downstream`, and the flip repairs it. The entity
    /// savings were the least of it.
    #[default]
    Downstream,
    /// Build both static orders and keep the better one.
    ///
    /// **Not the default, and no longer needed:** `Downstream` matches it on
    /// every corpus target, so the second build buys nothing (KC4's "do not ship
    /// machinery for a tie", one level up). Kept as the instrument that
    /// re-derives the corpus verdict. By every validator channel this is
    /// strictly better than either fixed arm on the corpus and worse on none.
    /// In a real Factorio it ships a jammed factory on at least one target
    /// (`display-panel@1` am1), because the DI row cell it selects there is
    /// physically broken while validating clean. Kept live and reachable so the
    /// policy can be re-verified the moment that cell is fixed (#520) — deleting it
    /// would mean re-deriving a measurement that took a corpus sweep across
    /// three machine tiers.
    ///
    /// RFC-059 set out to choose between `Upstream` and `Downstream` and first
    /// measured that neither dominates — `Downstream` better on 6 corpus targets
    /// and worse on 2 — which is why this variant exists. **That reading was an
    /// artefact of the validator's starved-pickup blind spot (#520):** the two
    /// targets it "lost" were ones where `Upstream` shipped a factory running at
    /// half its planned rate. With the blind spot fixed, `Downstream` is never
    /// worse and this arm has nothing left to buy.
    ///
    /// Searching is exhaustive here, not heuristic. The same sweep pinned each
    /// contended coupling to claim first and rebuilt: on no target did any
    /// other assignment beat both static orders. So the per-target optimum is
    /// always one of these two, which is why the RFC ships no gain estimator
    /// and no matching solver.
    ///
    /// Only `DirectInsertion::Candidate` searches — it is a candidate-level
    /// concept, and the placer, which walks one order per call, reads this as
    /// `Upstream`. `Forced` is the single-variant A/B arm and is what the
    /// search itself runs, so it must stay deterministic.
    Search,
    /// An explicit priority over individual couplings: the ones named here are
    /// offered the claim first, in the order given, and everything else follows
    /// in `Upstream` order behind them.
    ///
    /// Two jobs, both from RFC-059:
    ///
    /// - it is how phase 1 produces **output 3**, the per-coupling ground truth
    ///   kill criterion 2 tests an estimator's ranking against. P0 and P1 sample
    ///   only two points of an assignment space that is larger whenever one
    ///   target has several contended specs sharing a coupling, so "which
    ///   coupling wins this spec" cannot be answered by flipping the walk;
    /// - it is the mechanism P3 would APPLY through. A matching is a set of
    ///   couplings, and handing that set to the dispatcher as a priority is how
    ///   the chosen assignment becomes a layout.
    ///
    /// Priority, not restriction: a pinned coupling that turns out unbuildable
    /// still loses, and unpinned couplings still claim what is left. So this
    /// cannot manufacture an infeasible assignment — the same gates apply.
    Pinned(std::sync::Arc<Vec<DiCouplingKey>>),
}

/// One candidate coupling, by name rather than by index — `producer` and
/// `consumer` are *recipe* names, matching `SolverResult::di_couplings`.
///
/// Names because the caller choosing a priority (a test, a matching solver)
/// works from the solver's coupling list, and `ordered`'s indices are a
/// placer-internal detail that does not survive being handed out.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiCouplingKey {
    pub item: String,
    pub producer: String,
    pub consumer: String,
}

impl DiCouplingKey {
    pub fn new(item: &str, producer: &str, consumer: &str) -> Self {
        Self {
            item: item.to_string(),
            producer: producer.to_string(),
            consumer: consumer.to_string(),
        }
    }
}

impl DiClaimOrder {
    /// Build a `Pinned` order from `(item, producer, consumer)` triples.
    pub fn pinned<'a>(keys: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>) -> Self {
        DiClaimOrder::Pinned(std::sync::Arc::new(
            keys.into_iter()
                .map(|(i, p, c)| DiCouplingKey::new(i, p, c))
                .collect(),
        ))
    }
}

impl DirectInsertion {
    /// Does the PLACER act on `di_couplings` for this call? Only
    /// `Forced` does: under `Candidate` the native pass must stay
    /// DI-free, because that is what makes a tie bit-identical.
    pub fn placer_acts(self) -> bool {
        matches!(self, DirectInsertion::Forced)
    }

    /// A/B helper for the old `bool` surface: `true` means "DI in this
    /// layout" (`Forced`), `false` means "no DI at all" (`Off`).
    /// Deliberately never yields `Candidate` — a caller asking for a
    /// definite answer must not get a scored competition instead.
    pub fn forced(on: bool) -> Self {
        if on {
            DirectInsertion::Forced
        } else {
            DirectInsertion::Off
        }
    }
}

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
    /// Leftmost x at which the output belt carries the FULL cumulative
    /// output of every consumer's out-inserter — the RIGHTMOST (last)
    /// drop's own column, not the leftmost. Belts are one-directional: a
    /// pickup west of a given drop can never see that drop's item (it can
    /// only move further east), so a picker positioned before the LAST drop
    /// systematically misses that drop's entire share, not merely reads an
    /// occasional empty tile. #526 first got this wrong by clamping to the
    /// leftmost drop instead: it fixed reachability (no tile is ever
    /// literally empty) but not throughput (the last drop's flow is
    /// permanently unreachable by every picker upstream of it) — caught by
    /// the sim harness measuring 2.94/s against a 5.00/s plan despite a
    /// validator-clean layout, the same "validates clean, physically wrong"
    /// trap #520 itself is about. A downstream `stamp_di_bridge` that picks
    /// up west of THIS tile can still read nonzero belt (so the old bug is
    /// gone) but cannot draw the LAST drop's contribution.
    pub output_feed_x_min: i32,
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

    // The belt spans the whole cell (`x_min..=x_max` above), but only the
    // consumer machines drop onto it. A picker needs the RIGHTMOST
    // (last) consumer's own column, not the leftmost: belts are
    // one-directional, so only downstream of every drop has the belt seen
    // the FULL cumulative output (#526 — see the field doc). `consumer_xs`
    // is non-empty here: `plan_straddle` refuses a plan with zero
    // consumers.
    let output_feed_x_min = x0
        + plan.consumer_xs.iter().copied().max().unwrap_or(0)
        + mid;

    Some(DiCellLayout {
        entities: ents,
        input_belt_y,
        producer_y,
        band_y,
        consumer_y,
        output_belt_y,
        x_min,
        x_max,
        output_feed_x_min,
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

    /// Total x-extent. Takes BOTH widths because a cell may mix machine
    /// footprints (a foundry is 5x5 against an assembler's 3x3), so the
    /// last machine's width depends on its role.
    pub fn width(&self, producer_w: i32, consumer_w: i32) -> i32 {
        match (self.xs.last(), self.sequence.last()) {
            (Some(&last), Some(&is_p)) => last + if is_p { producer_w } else { consumer_w },
            _ => 0,
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
    producer_w: i32,
    consumer_w: i32,
) -> Option<RowCellPlan> {
    if producer_count == 0 || consumer_count == 0 || producer_w <= 0 || consumer_w <= 0 {
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

    // Place consumers into the SLOTS around each producer, then emit.
    //
    // Every producer has two neighbours, a left and a right, so the gap
    // between `P_i` and `P_{i+1}` holds up to TWO consumers: one hugging
    // each side. The previous construction walked producers and appended
    // each one's consumers immediately AFTER it, which can only ever fill
    // the right side — so it refused two shapes the geometry allows:
    //
    //   * more consumers than producers, needing one before `P0`
    //     (15:16 at 8.0 vs 7.5 — `C0 P0 C1 P1 … P14 C15`);
    //   * 1:2 fan-out, one producer feeding a consumer on EACH side
    //     (`copper-cable → space-platform-foundation`, 4:8 at 5.0 vs 2.5 —
    //     `C0 P0 C1 C2 P1 C3 …`), where the appended pair landed `P0 C0 C1`
    //     and left `C1` touching no producer at all.
    //
    // A consumer fed by BOTH `P_i` and `P_{i+1}` is special: it has to sit
    // between them, so it occupies the whole gap and marks it `shared`.
    let mut left: Vec<Option<usize>> = vec![None; producer_count];
    let mut right: Vec<Option<usize>> = vec![None; producer_count];
    let mut shared: Vec<bool> = vec![false; producer_count];
    // How many consumers touch each producer — the tie-breaker below.
    let mut adjacent: Vec<usize> = vec![0; producer_count];
    for feeders in &per_consumer {
        for &(i, _) in feeders {
            adjacent[i] += 1;
        }
    }

    for (j, feeders) in per_consumer.iter().enumerate() {
        let lo = feeders[0].0;
        let hi = feeders.last().unwrap().0;
        if hi > lo + 1 {
            return None;
        }
        if hi == lo + 1 {
            // Straddles the gap: needs it to itself.
            if right[lo].is_some() || left[lo + 1].is_some() {
                return None;
            }
            right[lo] = Some(j);
            left[lo + 1] = Some(j);
            shared[lo] = true;
        } else {
            // Single feeder. Take the LEFT side only when this producer
            // must hold two consumers and this is its first — otherwise
            // keep the historical right-first placement, which is what
            // makes 1:1 rows still emit `PCPCPC…` rather than `CPCPCP…`
            // and leaves every shipped layout byte-identical.
            let needs_both = adjacent[lo] >= 2 && left[lo].is_none() && right[lo].is_none();
            // The `shared[lo - 1]` half is DEFENSIVE and currently
            // unreachable: the flow intervals hand consumers over in strict
            // left-to-right order, so a straddle that claims `left[lo]`
            // always lands before any single-fed consumer of `lo` could
            // want it — an instrumented fuzz saw the condition true 497,747
            // times in 2M trials and load-bearing in ZERO of them. Kept
            // rather than deleted because it costs nothing and the ordering
            // invariant it leans on is not locally obvious; flagged so the
            // next reader does not mistake it for live logic.
            if needs_both && !(lo > 0 && shared[lo - 1]) {
                left[lo] = Some(j);
            } else if right[lo].is_none() {
                right[lo] = Some(j);
            } else {
                // Both sides spoken for: a third neighbour is impossible.
                return None;
            }
        }
    }

    let mut sequence: Vec<bool> = Vec::new();
    let mut prod_slot: Vec<usize> = vec![usize::MAX; producer_count];
    let mut cons_slot: Vec<usize> = vec![usize::MAX; consumer_count];
    for i in 0..producer_count {
        // A shared consumer is registered on both sides of its gap; the
        // `usize::MAX` guard emits it once, on the left of `P_{i+1}`,
        // which is precisely between its two feeders.
        if let Some(j) = left[i] {
            if cons_slot[j] == usize::MAX {
                cons_slot[j] = sequence.len();
                sequence.push(false);
            }
        }
        prod_slot[i] = sequence.len();
        sequence.push(true);
        if let Some(j) = right[i] {
            if !shared[i] {
                cons_slot[j] = sequence.len();
                sequence.push(false);
            }
        }
    }
    if cons_slot.contains(&usize::MAX) {
        return None;
    }

    // Per-machine pitch: each machine's own width plus the 1-tile gap that
    // holds its coupling inserter. A uniform pitch would overlap machines
    // whenever the two roles have different footprints.
    let mut xs: Vec<i32> = Vec::with_capacity(sequence.len());
    let mut cursor = 0i32;
    for &is_p in &sequence {
        xs.push(cursor);
        cursor += if is_p { producer_w } else { consumer_w } + 1;
    }

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
        let p = plan_row_straddle(6, 4, 5.0, 7.5, 3, 3).expect("6:4 must arrange");
        let seq: String = p
            .sequence
            .iter()
            .map(|&is_p| if is_p { 'P' } else { 'C' })
            .collect();
        assert_eq!(seq, "PCPCPPCPCP", "sequence must match the hand derivation");
        assert_eq!(p.sequence.len(), 10);
        // Pitch 4 = 3-wide machine + the 1-tile coupling gap.
        assert_eq!(p.xs, vec![0, 4, 8, 12, 16, 20, 24, 28, 32, 36]);
        assert_eq!(p.width(3, 3), 39);
    }

    /// The defining property: every coupling is between physically
    /// ADJACENT machines. Without it the plan is unbuildable, since an
    /// inserter only spans one gap.
    #[test]
    fn every_edge_couples_adjacent_machines() {
        for (pc, cc, pr, cr) in [(6, 4, 5.0, 7.5), (4, 4, 2.5, 2.5), (2, 1, 3.0, 6.0)] {
            let p = plan_row_straddle(pc, cc, pr, cr, 3, 3).expect("must arrange");
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
        let p = plan_row_straddle(6, 4, pr, cr, 3, 3).unwrap();
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
        let p = plan_row_straddle(6, 4, 5.0, 7.5, 3, 3).unwrap();
        assert_eq!(p.required_rate(), 5.0);
    }

    /// Out-of-scope shapes refuse rather than approximate.
    #[test]
    fn out_of_scope_shapes_are_refused() {
        // Unbalanced flow.
        assert!(plan_row_straddle(6, 4, 5.0, 9.0, 3, 3).is_none());
        // A consumer needing three producers has only two neighbours.
        assert!(plan_row_straddle(9, 3, 1.0, 3.0, 3, 3).is_none());
        // Degenerate inputs.
        assert!(plan_row_straddle(0, 4, 5.0, 7.5, 3, 3).is_none());
        assert!(plan_row_straddle(6, 4, f64::NAN, 7.5, 3, 3).is_none());
        assert!(plan_row_straddle(6, 4, 5.0, 7.5, 0, 0).is_none());
    }

    /// 1:2 fan-out — one producer feeding a consumer on EACH side. This is
    /// `copper-cable → space-platform-foundation` (353 corpus instances):
    /// 4 producers at 5.0/s against 8 consumers at 2.5/s, balancing
    /// exactly. The old append-only emission produced `P0 C0 C1 …`, which
    /// left `C1` touching no producer, and the adjacency invariant
    /// correctly refused what the loop had built.
    #[test]
    fn one_to_two_fan_out_puts_a_consumer_on_each_side() {
        let p = plan_row_straddle(4, 8, 5.0, 2.5, 3, 3).expect("1:2 fan-out must arrange");
        let seq: String = p.sequence.iter().map(|&b| if b { 'P' } else { 'C' }).collect();
        assert_eq!(seq, "CPCCPCCPCCPC");
        assert_eq!(p.edges.len(), 8, "one edge per consumer");
        // Every consumer must be adjacent to its single producer.
        for &(ps, cs, _) in &p.edges {
            assert_eq!(ps.abs_diff(cs), 1, "edge {ps}->{cs} is not adjacent");
        }
    }

    /// More consumers than producers: 15 producers at 8.0/s feeding 16
    /// consumers at 7.5/s (the `casting-*` ratio at scale). Needs a
    /// consumer BEFORE the first producer, which an append-only walk can
    /// never emit.
    #[test]
    fn more_consumers_than_producers_opens_the_leading_slot() {
        let p = plan_row_straddle(15, 16, 8.0, 7.5, 3, 3).expect("15:16 must arrange");
        let seq: String = p.sequence.iter().map(|&b| if b { 'P' } else { 'C' }).collect();
        assert!(seq.starts_with('C'), "a consumer must lead: {seq}");
        assert_eq!(seq.matches('P').count(), 15);
        assert_eq!(seq.matches('C').count(), 16);
        for &(ps, cs, _) in &p.edges {
            assert_eq!(ps.abs_diff(cs), 1, "edge {ps}->{cs} is not adjacent");
        }
    }

    /// 1:1 pairing (the furnace→furnace shape) interleaves strictly.
    #[test]
    fn one_to_one_interleaves() {
        let p = plan_row_straddle(4, 4, 2.5, 2.5, 3, 3).unwrap();
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
    /// Rows carrying a fluid tap point, for `RowSpan.fluid_port_ys`.
    pub fluid_port_ys: Vec<i32>,
    /// `(item, x, y)` of each real fluid input port, for
    /// `RowSpan.fluid_port_pipes`. The lane planner taps fluids through
    /// these, NOT through `input_belt_y`.
    pub fluid_port_pipes: Vec<(String, i32, i32)>,
    /// Topmost row the cell actually occupies, measured over the stamped
    /// entities. `RowSpan.y_start` must come from HERE and not from
    /// `input_belt_ys[0]`: the cell's top is a belt row only when the
    /// producer is belt-fed, and is the pipe row when it is piped —
    /// deriving it from the belt list puts `y_start` below the machines
    /// and silently shrinks the span used for row attribution and pole
    /// banding.
    pub y_top: i32,
    pub machine_y: i32,
    pub output_belt_y: i32,
    pub x_min: i32,
    pub x_max: i32,
    /// Leftmost x at which the output belt carries the FULL cumulative
    /// output of every consumer-role machine's out-inserter — the
    /// RIGHTMOST (last) consumer's own column, not the leftmost. See
    /// `DiCellLayout::output_feed_x_min`'s doc for why: belts are
    /// one-directional, so a picker upstream of the last drop permanently
    /// misses that drop's whole share rather than merely reading an
    /// occasional empty tile (#526).
    pub output_feed_x_min: i32,
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
    /// `(item, belt, feed_inserter)` for a SOLID-fed producer. Ignored
    /// when `producer_fluid` is set. This belt is the producer's own, on
    /// the NORTH face at reach-1 — see the sketch on `stamp_row_cell` for
    /// the row assignments.
    pub producer_input: (&'a str, &'a str, &'a str),
    /// `Some((fluid_item, pipe_entity))` when the producer's inputs are
    /// ALL fluid — `casting-copper-cable` (molten-copper → copper-cable),
    /// `casting-iron`, `solid-fuel-from-light-oil`. Such a producer has no
    /// solid input at all, so the north face that would carry a feed belt
    /// and inserters is free, and the pipe run goes exactly there. That is
    /// also why the corpus shows no canonical pipe face: no contention.
    pub producer_fluid: Option<(&'a str, &'a str)>,
    /// `(item, belt, feed_inserter)` for the consumer's belt-fed input, or
    /// `None` when the coupled item is its ONLY solid ingredient
    /// (`solid-fuel-from-light-oil → rocket-fuel`). Then its south face
    /// carries the output alone and no inner belt is stamped.
    pub consumer_input: Option<(&'a str, &'a str, &'a str)>,
    /// `Some((fluid_item, pipe_entity))` when the CONSUMER also draws a
    /// fluid. Only ever the same fluid the producer is piped: the cell
    /// declares one set of tap points on one row and the lane planner
    /// routes a single run to them, so both roles' north input ports are
    /// registered against that one run. A second, different fluid would
    /// need a second run on a face the cell does not have.
    pub consumer_fluid: Option<(&'a str, &'a str)>,
    /// `(item, belt, feed_inserter)` for the consumer's SECOND belt-fed
    /// input — the three-solid-input shape (`iron-stick -> rail`, whose
    /// consumer wants stone AND steel-plate alongside the coupled stick).
    ///
    /// It lands on the NORTH face, one row ABOVE the producer's belt, fed
    /// by a reach-2 inserter sharing the producer's feed row over the
    /// CONSUMER's columns (the two roles' columns are disjoint, so they
    /// never contend). North rather than south because the south face is
    /// already full — feed plus output — and because moving the OUTPUT to
    /// make room is not a stamping change but an `output_merger` rework:
    /// that merger assumes a south-facing chain universally.
    pub consumer_input_b: Option<(&'a str, &'a str, &'a str)>,
    /// Inserters per machine face. The output face needs reach-2 (it steps
    /// over the consumer's input belt), and long-handed is the only
    /// reach-2 inserter (I8a) at 2.40/s at L2 — under the 2.5/s an EC
    /// machine emits — so more than one column is the NORMAL case here,
    /// not an edge case. Ignoring these counts silently under-feeds.
    pub producer_feed_count: usize,
    pub consumer_feed_count: usize,
    pub consumer_feed_b_count: usize,
    pub out_count: usize,
    pub output_item: &'a str,
    pub output_belt: &'a str,
    pub out_inserter: &'a str,
}

/// Stamp a horizontal row cell with its belts.
///
/// ```text
///   y0-1                consumer input belt B, only in the three-solid-
///                       input shape (absent otherwise)
///   y0                  producer input belt, or the PIPE run when the
///                       producer is fluid-fed
///   y1                  producer feed inserters, reach-1 (none when piped),
///                       sharing the row with B's reach-2 feed inserters
///                       over the consumer's columns
///   y2 .. y2+h-1        machines, interleaved P/C at plan.xs,
///                       BOTTOM-aligned so both roles share a south face
///   y2+h                face row: consumer feed reach-1 + output reach-2
///   y2+h+1              consumer input belt (absent when the coupling
///                       supplies every solid the consumer needs)
///   y2+h+2              output belt — moves up to y2+h+1, at reach-1,
///                       when there is no consumer input belt to step over
/// ```
///
/// The in-body comment beside `p_belt_y` is the authority on the exact
/// rows; this sketch is the shape.
///
/// Couplers sit in the gap columns BETWEEN machines, at reach 1 — the
/// property that makes this DI at all. Returns `None` if the coupler
/// cannot carry the busiest edge, rather than emitting an under-fed row.
#[allow(clippy::too_many_arguments)]
pub fn stamp_row_cell(
    plan: &RowCellPlan,
    spec: &RowCellSpec<'_>,
    x0: i32,
    y0: i32,
    producer_w: i32,
    producer_h: i32,
    consumer_w: i32,
    consumer_h: i32,
) -> Option<RowCellLayout> {
    if producer_w <= 0 || producer_h <= 0 || consumer_w <= 0 || consumer_h <= 0 {
        return None;
    }
    // Machines are BOTTOM-ALIGNED so both roles share one south face row.
    // Top-aligning them would leave a shorter machine's south face two
    // tiles above the face row, unreachable by its own feed and output
    // inserters. Bottom-alignment also guarantees the roles overlap on
    // the bottom row, which is where the coupling inserters sit.
    let max_h = producer_h.max(consumer_h);
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
    // The consumer's SECOND belt-fed input, when it has one, goes one row
    // ABOVE the producer's belt and is picked by a reach-2 inserter sharing
    // the producer's feed row:
    //
    //   y0-1                consumer input belt B   (north, outer)
    //   y0                  producer input belt     (north, inner)
    //   y1                  producer feed reach-1 over PRODUCER columns,
    //                       consumer feed B reach-2 over CONSUMER columns
    //
    // The reach-2 inserter passes OVER the producer's belt at y0 — an
    // inserter interacts with its pick and drop tiles only, never with what
    // it swings across — so no underground gap is needed, unlike
    // `RowKind::QuadInput` where the inserter sits ON the belt row itself.
    //
    // Two constraints follow from the drop tile, and both are enforced
    // rather than assumed. The inserter sits at `y1 = machine_y - 1` and
    // drops at `y1 + 2 = machine_y + 1`, so:
    //   - the CONSUMER must reach that row. Bottom-alignment puts its top
    //     at `machine_y + (max_h - consumer_h)`, so a producer more than
    //     ONE tile taller lifts the drop above the consumer's body and the
    //     item lands on nothing. Foundry(5) over assembler(3) is exactly
    //     this case, which is why the shipped fluid pairs are excluded.
    //   - the consumer must be at least 2 tall, or `machine_y + 1` is past
    //     its bottom.
    let b_belt = spec.consumer_input_b;
    if b_belt.is_some() {
        // A piped producer puts its pipe run on the feed row (see the pipe
        // stamp below), which is exactly where B's inserters would go.
        if spec.producer_fluid.is_some() {
            return None;
        }
        if producer_h - consumer_h > 1 || consumer_h < 2 {
            return None;
        }
    }
    let p_belt_y = y0;
    let p_feed_y = y0 + 1;
    let machine_y = y0 + 2;
    let face_y = machine_y + max_h;
    // Row-relative top of each role once bottom-aligned.
    let top_of = |is_p: bool| machine_y + (max_h - if is_p { producer_h } else { consumer_h });
    let c_belt_y = face_y + 1;
    // With no belt-fed consumer input that inner row is empty, so the
    // output belt moves up into it. Worth doing rather than leaving a gap:
    // it drops a row from the cell AND puts the output inserter at reach-1,
    // off long-handed's 2.40/s ceiling — the constraint that forces two
    // output columns in the ordinary shape.
    let output_belt_y = if spec.consumer_input.is_some() { face_y + 2 } else { face_y + 1 };
    let x_min = x0;
    let x_max = x0 + plan.width(producer_w, consumer_w) - 1;
    let seg = format!("di-row:{}:{}", spec.item, spec.consumer_recipe);
    let mut ents = Vec::new();
    let mut fluid_ports: Vec<(String, i32, i32)> = Vec::new();
    // Leftmost x at which the output belt carries the FULL cumulative
    // output — the RIGHTMOST consumer out-inserter column, tracked as
    // they're stamped below (#526; see `RowCellLayout::output_feed_x_min`).
    let mut output_feed_x_min: Option<i32> = None;

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
    if spec.producer_fluid.is_none() {
        belt_run(&mut ents, p_belt_y, spec.producer_input.1, spec.producer_input.0);
    }
    if let Some((item, belt, _)) = b_belt {
        belt_run(&mut ents, p_belt_y - 1, belt, item);
    }
    if let Some((item, belt, _)) = spec.consumer_input {
        belt_run(&mut ents, c_belt_y, belt, item);
    }
    belt_run(&mut ents, output_belt_y, spec.output_belt, spec.output_item);

    // Columns available on a face, innermost first so single-inserter
    // faces keep the natural centre-ish position.
    let cols = |w: i32, n: usize| -> Vec<i32> { (0..w).take(n).collect() };

    for (k, &is_producer) in plan.sequence.iter().enumerate() {
        let mx = x0 + plan.xs[k];
        // A fluid-fed producer must be placed at the orientation whose
        // fluid INPUT port faces north, or the pipe row below lands on a
        // tile that merely looks adjacent and carries nothing. Ports are
        // prototype-fixed per direction, so this is read from the shared
        // table rather than assumed.
        let (m_mirror, m_dir) = match is_producer {
            true if spec.producer_fluid.is_some() => {
                crate::fluid_ports::north_input_orientation(spec.producer_entity)
            }
            false if spec.consumer_fluid.is_some() => {
                crate::fluid_ports::north_input_orientation(spec.consumer_entity)
            }
            _ => (false, EntityDirection::North),
        };
        let my = top_of(is_producer);
        ents.push(PlacedEntity {
            name: if is_producer { spec.producer_entity } else { spec.consumer_entity }.to_string(),
            x: mx,
            y: my,
            direction: m_dir,
            mirror: m_mirror,
            recipe: Some(
                if is_producer { spec.producer_recipe } else { spec.consumer_recipe }.to_string(),
            ),
            segment_id: Some(seg.clone()),
            ..Default::default()
        });
        if is_producer {
            if let Some((fluid_item, _pipe)) = spec.producer_fluid {
                // Record the machine's REAL north input ports as tap points.
                for dx in crate::fluid_ports::north_input_dxs(spec.producer_entity, m_mirror, m_dir)
                {
                    fluid_ports.push((fluid_item.to_string(), mx + dx, my - 1));
                }
            } else {
                // North face, reach-1: picks the belt above, drops into the machine.
                for dx in cols(producer_w, spec.producer_feed_count.max(1)) {
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
            }
        } else {
            // NOTE: a fluid-drawing consumer registers NO tap points of its
            // own. Tested, not assumed: adding them changes nothing. The
            // pipe run below is stamped across the whole cell width from
            // the producer's side, so it is one connected network and the
            // consumer's north ports are adjacent to it either way;
            // `fluid_port_pipes` only tells the lane planner where to tap
            // the bus INTO the cell, which the producer's ports already do.
            // What IS load-bearing is the consumer's ORIENTATION above —
            // eligibility admits it on the strength of having a north input
            // port, so the stamp has to actually put it there.
            //
            // South face shares one row: reach-1 feeds from the inner belt,
            // reach-2 outputs over it. Feed takes the low columns, output
            // the high ones, so they never contend for a tile. A consumer
            // whose only solid ingredient is the coupled one has no feed at
            // all and gives the whole face to its output.
            // North face, reach-2: picks belt B two rows up, swinging over
            // the producer's belt, and drops into this machine's second
            // row. Shares `p_feed_y` with the producer's own feed
            // inserters, which sit over the PRODUCER's columns — disjoint
            // by construction, since each role's inserters are placed
            // within its own footprint.
            if let Some((item, _, inserter)) = b_belt {
                let nb = spec.consumer_feed_b_count.max(1);
                if nb > consumer_w as usize {
                    return None;
                }
                for dx in cols(consumer_w, nb) {
                    ents.push(PlacedEntity {
                        name: inserter.to_string(),
                        x: mx + dx,
                        y: p_feed_y,
                        direction: EntityDirection::South,
                        carries: Some(item.to_string()),
                        segment_id: Some(seg.clone()),
                        ..Default::default()
                    });
                }
            }
            let nf = if spec.consumer_input.is_some() { spec.consumer_feed_count.max(1) } else { 0 };
            let no = spec.out_count.max(1);
            if nf + no > consumer_w as usize {
                return None;
            }
            for dx in 0..nf as i32 {
                let (item, _, inserter) = spec.consumer_input.expect("nf > 0 implies Some");
                ents.push(PlacedEntity {
                    name: inserter.to_string(),
                    x: mx + dx,
                    y: face_y,
                    direction: EntityDirection::North,
                    carries: Some(item.to_string()),
                    segment_id: Some(seg.clone()),
                    ..Default::default()
                });
            }
            if no > 0 {
                // The RIGHTMOST out-inserter column for this consumer, not
                // the first: `no` output columns are stamped at
                // `mx + nf .. mx + nf + no - 1` below, and with `no >= 2`
                // (the ordinary case when the output side needs a second
                // reach-2 column — see the face-plan comment above) the
                // first column is NOT this consumer's last drop.
                let this_x = mx + nf as i32 + no as i32 - 1;
                output_feed_x_min =
                    Some(output_feed_x_min.map_or(this_x, |cur| cur.max(this_x)));
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

    // Continuous pipe run on the row ADJACENT to the machines, so it meets
    // the north input ports recorded above. A consumer sharing the row
    // either has no fluid box — the run forms no connection over its
    // columns — or draws the SAME fluid, which is the only case
    // `row_cell_eligible` admits, so joining that one network is exactly
    // the intent. Different fluids on one run would cross-contaminate,
    // which is why the gate is same-fluid rather than any-fluid.
    if let Some((fluid_item, pipe)) = spec.producer_fluid {
        let pipe_y = top_of(true) - 1;
        for x in x_min..=x_max {
            ents.push(PlacedEntity {
                name: pipe.to_string(),
                x,
                y: pipe_y,
                direction: EntityDirection::North,
                carries: Some(fluid_item.to_string()),
                segment_id: Some(seg.clone()),
                ..Default::default()
            });
        }
    }

    for &(ps, cs, _) in &plan.edges {
        let (gap_x, dir) = if cs == ps + 1 {
            (x0 + plan.xs[ps] + producer_w, EntityDirection::East)
        } else {
            (x0 + plan.xs[cs] + consumer_w, EntityDirection::West)
        };
        ents.push(PlacedEntity {
            name: spec.coupler.to_string(),
            // Bottom row: the only row both roles are guaranteed to occupy
            // once bottom-aligned, so the coupler reaches both.
            y: face_y - 1,
            x: gap_x,
            direction: dir,
            carries: Some(spec.item.to_string()),
            segment_id: Some(seg.clone()),
            ..Default::default()
        });
    }

    // Derived from the ports actually recorded, not from which role has a
    // fluid: the two can only agree by accident, and a port declared on a
    // row the lane planner is not told about would never be piped.
    let mut fluid_port_ys: Vec<i32> = fluid_ports.iter().map(|&(_, _, y)| y).collect();
    fluid_port_ys.sort_unstable();
    fluid_port_ys.dedup();
    // Parallel to the fused spec's solid-input order: producer's belt
    // first when it has one, then the consumer's. Either may be absent,
    // and both are when the producer is piped and the coupled item is the
    // consumer's only solid ingredient.
    let mut input_belt_ys = Vec::new();
    if spec.producer_fluid.is_none() {
        input_belt_ys.push(p_belt_y);
    }
    if spec.consumer_input.is_some() {
        input_belt_ys.push(c_belt_y);
    }
    // B goes LAST, not in geometric order — the contract is that this list
    // and the fused spec's solid inputs agree INDEX for INDEX, not that
    // either is sorted by y. Appending keeps every existing index stable.
    if b_belt.is_some() {
        input_belt_ys.push(p_belt_y - 1);
    }
    let y_top = ents.iter().map(|e| e.y).min().unwrap_or(y0);
    Some(RowCellLayout {
        entities: ents,
        input_belt_ys,
        y_top,
        fluid_port_ys,
        fluid_port_pipes: fluid_ports,
        machine_y,
        output_belt_y,
        x_min,
        x_max,
        // Always `Some` by construction: `no = spec.out_count.max(1)` and
        // `plan.sequence` contains at least one consumer (`plan_row_straddle`
        // refuses `consumer_count == 0`), so the loop above visits at least
        // one consumer with `no > 0`. `x_max` is a defensive fallback only,
        // and deliberately the CONSERVATIVE bound (rather than `x_min`): a
        // caller reading this field treats it as a floor for safe pickup, so
        // an unreachable fallback should overstate the floor, not understate
        // it.
        output_feed_x_min: output_feed_x_min.unwrap_or(x_max),
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
            producer_fluid: None,
            consumer_input: Some(("iron-plate", "transport-belt", "fast-inserter")),
            consumer_fluid: None,
            output_item: "electronic-circuit",
            output_belt: "transport-belt",
            out_inserter: "long-handed-inserter",
            producer_feed_count: 1,
            consumer_feed_count: 1,
            consumer_input_b: None,
            consumer_feed_b_count: 0,
            out_count: 2,
        }
    }

    fn stamped() -> (RowCellPlan, RowCellLayout) {
        let plan = plan_row_straddle(6, 4, 5.0, 7.5, 3, 3).unwrap();
        let l = stamp_row_cell(&plan, &spec(), 0, 0, 3, 3, 3, 3).unwrap();
        (plan, l)
    }

    /// The shared-fluid shape: a piped producer whose consumer draws the
    /// SAME fluid and takes no belt-fed solid at all
    /// (`solid-fuel-from-light-oil → rocket-fuel`).
    ///
    /// Pinned as a unit test rather than end-to-end because at the time
    /// this was written the shape was UNREACHABLE: the only corpus pair
    /// with it resolved `rocket-fuel` to a `biochamber`, which
    /// `cell_machines_are_powerable` refuses (burner, fuel category
    /// `nutrients`, and nothing in the engine delivers burner fuel).
    /// `chemical-plant` on both sides here stands in for the geometry,
    /// which is what this test is about.
    ///
    /// #461 part (a) changed `organic-or-assembling` (rocket-fuel's
    /// category) to fall through to the caller's assembler tier, so
    /// `rocket-fuel` no longer resolves to a biochamber and
    /// `cell_machines_are_powerable` no longer refuses it on that ground.
    /// Whether the real pipeline now actually forms this cell (both roles
    /// electric, other cell-eligibility heuristics permitting) was not
    /// re-audited here — this synthetic spec remains the only coverage for
    /// the geometry either way, so it is left as-is rather than widened
    /// into an end-to-end fixture.
    fn shared_fluid_spec() -> RowCellSpec<'static> {
        RowCellSpec {
            producer_entity: "chemical-plant",
            consumer_entity: "chemical-plant",
            producer_recipe: "solid-fuel-from-light-oil",
            consumer_recipe: "rocket-fuel",
            item: "solid-fuel",
            coupler: "inserter",
            coupler_rate: 12.0,
            producer_input: ("", "transport-belt", "inserter"),
            producer_fluid: Some(("light-oil", "pipe")),
            // The coupling supplies every solid the consumer needs.
            consumer_input: None,
            consumer_fluid: Some(("light-oil", "pipe")),
            output_item: "rocket-fuel",
            output_belt: "transport-belt",
            out_inserter: "inserter",
            producer_feed_count: 0,
            consumer_feed_count: 0,
            consumer_input_b: None,
            consumer_feed_b_count: 0,
            out_count: 1,
        }
    }

    /// `y_top` is the cell's real top, which for a piped producer is the
    /// PIPE row — not `input_belt_ys[0]`, which is then the consumer's belt
    /// below the machines. `RowSpan.y_start` reads this; deriving it from
    /// the belt list understated the span by the whole machine band.
    #[test]
    fn y_top_is_the_pipe_row_when_the_producer_is_piped() {
        let plan = plan_row_straddle(4, 4, 1.0, 1.0, 3, 3).unwrap();
        let l = stamp_row_cell(&plan, &shared_fluid_spec(), 0, 0, 3, 3, 3, 3).unwrap();
        let pipe_y = l
            .entities
            .iter()
            .filter(|e| e.name == "pipe")
            .map(|e| e.y)
            .min()
            .expect("piped producer must get a run");
        assert_eq!(l.y_top, pipe_y, "cell top is the pipe row");
        assert!(
            l.y_top < l.machine_y,
            "cell top {} must be above the machines at {}",
            l.y_top,
            l.machine_y
        );
        for &b in &l.input_belt_ys {
            assert!(
                b > l.y_top,
                "every input belt ({b}) sits below the cell top ({}) here, which is \
                 exactly why y_start must not come from input_belt_ys",
                l.y_top
            );
        }
    }

    /// With no belt-fed consumer input the inner row is empty, so the
    /// output belt moves up into it and its inserter drops at reach-1.
    /// Leaving the gap would cost a row AND pin the output to long-handed's
    /// 2.40/s ceiling.
    #[test]
    fn coupling_fed_consumer_drops_the_inner_belt_row() {
        let plan = plan_row_straddle(4, 4, 1.0, 1.0, 3, 3).unwrap();
        let l = stamp_row_cell(&plan, &shared_fluid_spec(), 0, 0, 3, 3, 3, 3).unwrap();

        // Face row is directly above the output belt: nothing between.
        let face_y = l.machine_y + 3;
        assert_eq!(l.output_belt_y, face_y + 1, "output belt sits against the face");
        assert!(
            l.input_belt_ys.is_empty(),
            "a piped producer and a coupling-fed consumer tap no belt at all, got {:?}",
            l.input_belt_ys
        );
        // Exactly one belt run — the output.
        let belt_ys: std::collections::BTreeSet<i32> = l
            .entities
            .iter()
            .filter(|e| e.name.ends_with("transport-belt"))
            .map(|e| e.y)
            .collect();
        assert_eq!(
            belt_ys.iter().copied().collect::<Vec<_>>(),
            vec![l.output_belt_y],
            "the only belt should be the output"
        );
    }

    /// Equal heights are an eligibility precondition precisely so that
    /// bottom-alignment lands BOTH roles' north faces on the pipe row the
    /// producer's run occupies. Checked against real port geometry, not by
    /// assuming the run's extent is enough.
    #[test]
    fn shared_fluid_run_reaches_both_roles_north_ports() {
        let plan = plan_row_straddle(4, 4, 1.0, 1.0, 3, 3).unwrap();
        let l = stamp_row_cell(&plan, &shared_fluid_spec(), 0, 0, 3, 3, 3, 3).unwrap();
        let pipes: std::collections::HashSet<(i32, i32)> = l
            .entities
            .iter()
            .filter(|e| e.name == "pipe")
            .map(|e| (e.x, e.y))
            .collect();
        assert!(!pipes.is_empty(), "a piped producer must get a run");

        let machines: Vec<_> =
            l.entities.iter().filter(|e| e.name == "chemical-plant").collect();
        assert_eq!(machines.len(), 8, "4 producers + 4 consumers");
        let (mirror, dir) = crate::fluid_ports::north_input_orientation("chemical-plant");
        let dxs = crate::fluid_ports::north_input_dxs("chemical-plant", mirror, dir);
        assert!(!dxs.is_empty(), "chemical-plant must expose a north input port");
        for m in &machines {
            assert!(
                dxs.iter().any(|dx| pipes.contains(&(m.x + dx, m.y - 1))),
                "machine at ({},{}) has no pipe on any north port (dxs {dxs:?})",
                m.x,
                m.y
            );
        }
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

    /// #526 F1: `output_feed_x_min` must be the RIGHTMOST out-inserter
    /// column of the LAST consumer, not its first. The canonical fixture's
    /// `out_count == 2` (two output columns per consumer — EC's 2.5/s needs
    /// two long-handed columns at L2), so this is exactly the shape an
    /// `mx + nf` computation gets wrong: it would report the first of the
    /// last consumer's two columns, one tile short of where the belt
    /// actually finishes receiving that consumer's output.
    #[test]
    fn output_feed_x_min_is_the_last_consumers_rightmost_column() {
        let (plan, l) = stamped();
        // Last (rightmost) consumer in the sequence.
        let last_consumer_idx = plan
            .sequence
            .iter()
            .enumerate()
            .filter(|&(_, &is_p)| !is_p)
            .next_back()
            .map(|(i, _)| i)
            .expect("canonical plan has consumers");
        let mx = plan.xs[last_consumer_idx];
        let nf = 1; // spec().consumer_feed_count
        let no = 2; // spec().out_count
        assert_eq!(
            l.output_feed_x_min,
            mx + nf + no - 1,
            "must be the last consumer's RIGHTMOST out-inserter column \
             (mx + nf + no - 1), not its first (mx + nf)"
        );
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

    /// The fluid-producer path: an all-fluid producer's north face carries
    /// a PIPE RUN instead of a belt + feed inserters, and the machines are
    /// placed at the orientation whose fluid input port faces north so the
    /// run lands on real ports.
    ///
    /// NOTE: no corpus pair reaches this path yet — see the RFC decision
    /// log. It is unit-tested here so the geometry is pinned for whoever
    /// lands the prerequisites (heterogeneous machine footprints, and
    /// fluid-on-both-sides).
    #[test]
    fn fluid_producer_gets_a_pipe_run_on_a_free_north_face() {
        let plan = plan_row_straddle(4, 4, 2.5, 2.5, 5, 3).unwrap();
        let mut sp = spec();
        sp.producer_recipe = "casting-copper-cable";
        sp.producer_entity = "foundry";
        sp.producer_fluid = Some(("molten-copper", "pipe"));
        let l = stamp_row_cell(&plan, &sp, 0, 0, 5, 5, 3, 3).expect("fluid cell must stamp");

        // No belt for the producer's input, and no feed inserters for it.
        assert!(
            !l.entities.iter().any(|e| e.carries.as_deref() == Some("copper-plate")),
            "an all-fluid producer must have no solid feed at all"
        );
        // A contiguous pipe run sits on the row ADJACENT to the machines.
        let pipe_y = l.machine_y - 1;
        let pipes: Vec<_> = l
            .entities
            .iter()
            .filter(|e| e.name == "pipe" && e.y == pipe_y)
            .collect();
        assert_eq!(
            pipes.len() as i32,
            l.x_max - l.x_min + 1,
            "pipe run must span the cell on the row adjacent to the machines"
        );
        // Every recorded port is on that run and carries the fluid.
        assert!(!l.fluid_port_pipes.is_empty(), "ports must be registered for the lane planner");
        for (item, _px, py) in &l.fluid_port_pipes {
            assert_eq!(item, "molten-copper");
            assert_eq!(*py, pipe_y, "a port off the pipe row would never connect");
        }
        assert_eq!(l.fluid_port_ys, vec![pipe_y]);
        // The fluid never displaces the consumer's belt-fed input.
        assert_eq!(l.input_belt_ys.len(), 1, "only the consumer's belt remains");
    }

    /// Under-rate couplers refuse rather than under-feed.
    #[test]
    fn under_rate_coupler_refuses() {
        let plan = plan_row_straddle(6, 4, 5.0, 7.5, 3, 3).unwrap();
        let mut s = spec();
        s.coupler = "inserter";
        s.coupler_rate = 0.84;
        assert!(stamp_row_cell(&plan, &s, 0, 0, 3, 3, 3, 3).is_none());
    }
}
