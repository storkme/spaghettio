//! RFC-064 "spaghetti objective" primitives (P1): the aspect-ratio score,
//! the composite, and the admissible-ranking rule, computed on a validated,
//! fully routed [`LayoutResult`] — never on an unrouted IR estimate, per
//! `docs/rfc-064-spaghetti-objective.md`'s own realism-step discipline.
//!
//! This module is pure and additive: nothing in the pipeline calls it.
//! It exists so the metrics used to gate RFC-064's Phases 1-5 are
//! implemented once, tracked, and tested, instead of re-derived by hand in
//! every phase (as they were in Phase 0's throwaway example script).
//!
//! Read `docs/rfc-064-spaghetti-objective.md`'s "Metrics" section before
//! changing anything here — the definitions below are transcriptions of
//! that section, not independent designs.
//!
//! ## Where transit comes from
//!
//! **This module does not measure transit.** [`measure`] delegates §(b)
//! wholly to [`crate::bus::transit::measure_realized_transit`] and adds only
//! §(a)'s bbox/aspect terms around it.
//!
//! It used to carry its own implementation, and that implementation was
//! non-conforming — it meaned over *producer* ports where §(b) specifies "the
//! arithmetic mean of those **consumer-terminal** distances", averaged over
//! whatever it could reach instead of refusing, modelled no pipe-to-ground
//! jump, and mixed direct-insertion samples into belt means. The two
//! disagreed by −29% to +98% on real fixtures. Do not reintroduce any of it;
//! the spec settled every one of those points before either version existed.
//!
//! Two consequences worth knowing before editing:
//!
//! - **A measurement is total or it does not exist.** §(b): "any other
//!   unreachable terminal makes the metric unmeasurable and the candidate
//!   inadmissible — never silently fall back to Manhattan for a broken routed
//!   edge." So [`measure`] returns `Err` where the old one returned a partial
//!   result, and every [`EdgeMeasurement`] in a successful measure carries a
//!   real `path_length`. There is no "attributed subset" to reason about.
//! - **Pipe-to-ground is modelled**, by `bus::transit` via
//!   `validate::fluids::find_ptg_pairs`. The known gap this module used to
//!   document (fully-underground fluid edges reporting no length) is closed.
use crate::models::{LayoutResult, SolverResult};

/// Weight applied to fluid-edge rate in [`LayoutMeasure::transit`], per
/// RFC-064 §(b): "fluid edges weighted by `fluid_weight < 1`, exactly as
/// RFC-055 did." RFC-055's own `fluid_weight` (`bus/cells/placement.rs`) is
/// a caller-supplied parameter with no canonical named constant in-tree —
/// its test call sites use `0.25` for an unrelated cell-placement scorer,
/// not a value RFC-055 itself adjudicated. RFC-064's Phase 0 decision log
/// (2026-08-01, finding 5) records choosing and documenting `0.5` for this
/// exact purpose after finding no in-tree canonical value, and validated it
/// by reproducing PR #500's anchor numbers — that is the value used here.
pub const FLUID_WEIGHT: f64 = 0.5;

/// Non-gating WARN threshold for `ΔEntities%`, per RFC-064 §(c): "roughly
/// 2x the folding calibration anchor" (`chain-mil5ore`'s Factorio-verified
/// +26%, PR #500).
pub const ENTITY_GROWTH_WARN_PCT: f64 = 0.52;

/// Tie-break epsilon for [`rank_admissible`], per RFC-064 §(d) step 3.
pub const COMPOSITE_TIE_EPSILON: f64 = 0.02;

const DEGENERATE_EPS: f64 = 1e-9;

/// One production edge's realized measurement, as `bus::transit` reported it.
/// Every field is populated: §(b) makes an unmeasurable edge fatal to the
/// whole measurement, so there is no "this edge could not be attributed"
/// state for an edge that reaches this struct.
#[derive(Debug, Clone)]
pub struct EdgeMeasurement {
    pub producer_recipes: Vec<String>,
    pub item: String,
    pub consumer_recipe: String,
    /// Raw (unweighted) solved rate, items/s or fluid-units/s.
    pub rate: f64,
    pub is_fluid: bool,
    /// Realized physical tile length of the routed path.
    ///
    /// Not optional: RFC-064 §(b) makes an unreachable terminal fatal to the
    /// whole measurement rather than a per-edge hole, so a `LayoutMeasure`
    /// that exists has a length for every edge. The old `Option` encoded a
    /// partial state this metric is not allowed to be in.
    pub path_length: f64,
    /// Producer / consumer transport terminals the measurement paired.
    /// Carried for reporting: §(b)'s mean is over consumer terminals, so the
    /// count is what that mean was taken over.
    pub producer_terminals: usize,
    pub consumer_terminals: usize,
    /// True when this edge was measured as a solver-declared direct-insertion
    /// edge with no transport network (§(b)'s port-to-port Manhattan case)
    /// rather than over the belt/pipe graph.
    pub direct_insertion: bool,
}

/// Raw per-layout numbers RFC-064's Metrics section defines, computed on a
/// validated, fully routed [`LayoutResult`].
#[derive(Debug, Clone)]
pub struct LayoutMeasure {
    /// Non-pole entity bounding box width (RFC-064 §(a)'s footprint
    /// convention, reusing `common::oriented_entity_dims`).
    pub bbox_width: i32,
    pub bbox_height: i32,
    /// `max(width, height) / min(width, height)`.
    pub aspect_ratio: f64,
    /// Total placed-entity count (all entities, matching the folding
    /// anchor's own `2831 -> 3567` counting convention).
    pub entity_count: usize,
    /// `Σ rate(e) × fluid_weight?(e) × path_length(e)` over ALL edges.
    ///
    /// There is no "attributed subset" qualifier any more: §(b) makes an
    /// unmeasurable edge fatal to the measurement, so either every edge is in
    /// this sum or there is no `LayoutMeasure` at all. The counters this
    /// struct used to carry (`unattributed_edge_count`,
    /// `partially_attributed_edge_count`) described states the conforming
    /// metric cannot be in.
    pub transit: f64,
    /// The same total split by transport medium, as `bus::transit` reports it.
    pub solid_transit: f64,
    pub fluid_transit: f64,
    /// Per-edge breakdown, for debugging and reporting (RFC-064 §(b)'s
    /// insistence that a per-category count never hide inside one number).
    pub edges: Vec<EdgeMeasurement>,
}

/// Compute [`LayoutMeasure`] for `layout`, the routed output of the solve
/// `solver` describes.
///
/// Errors if `layout` has no non-pole entities (nothing to measure), if
/// `solver`'s production graph cannot be derived, **or if any production edge
/// is unmeasurable** — an unreachable producer or consumer terminal fails the
/// whole call, per RFC-064 §(b), rather than yielding a partial result. That
/// third case arrives via [`crate::bus::transit`] and is the common one in
/// practice; callers that treat `measure` as near-infallible are wrong.
pub fn measure(layout: &LayoutResult, solver: &SolverResult) -> Result<LayoutMeasure, String> {
    let (min_x, min_y, max_x, max_y) = non_pole_bbox(layout)
        .ok_or_else(|| "layout has no non-pole entities to measure".to_string())?;
    let bbox_width = max_x - min_x;
    let bbox_height = max_y - min_y;
    let long = bbox_width.max(bbox_height);
    let short = bbox_width.min(bbox_height);
    // `short <= 0` is unreachable for any layout this function accepts:
    // `measure()` errors on an empty non-pole entity set, and any real
    // entity's oriented footprint is >= 1x1, so both bbox dims are >= 1.
    // The guard exists only so a hypothetical future caller can't divide
    // by zero — and it must NOT silently score such a degenerate as a
    // perfect square (round-3 bot review, minor 4).
    debug_assert!(short > 0, "non-pole bbox must have positive extent");
    let aspect_ratio = if short <= 0 { f64::INFINITY } else { long as f64 / short as f64 };

    // RFC-064 §(b) is implemented ONCE, in `bus::transit`. This module used
    // to carry a second implementation that disagreed with it by -29% to +98%
    // on real fixtures — because it aggregated in the opposite direction
    // (mean over PRODUCER ports of the shortest path to any consumer, rather
    // than §(b)'s "arithmetic mean of those consumer-terminal distances").
    // It also silently averaged over whatever it could reach instead of
    // refusing, and modelled no pipe-to-ground jump. §(b) settles all three,
    // and `bus::transit` is the conforming reading, so the duplicate is gone
    // rather than reconciled.
    let realized = crate::bus::transit::measure_realized_transit(layout, solver, FLUID_WEIGHT)
        .map_err(|e| format!("transit is not measurable: {e}"))?;

    // §(b): "Any other unreachable terminal makes the metric unmeasurable and
    // the candidate inadmissible — never silently fall back to Manhattan for
    // a broken routed edge." So a returned measurement is TOTAL: every edge
    // carries a length, and there is no partial state left to represent.
    let edges = realized
        .edges
        .iter()
        .map(|e| EdgeMeasurement {
            producer_recipes: e.producer_recipes.clone(),
            item: e.item.clone(),
            consumer_recipe: e.consumer_recipe.clone(),
            rate: e.planned_rate,
            is_fluid: e.is_fluid,
            path_length: e.path_length,
            producer_terminals: e.producer_terminals,
            consumer_terminals: e.consumer_terminals,
            direct_insertion: e.direct_insertion,
        })
        .collect();

    Ok(LayoutMeasure {
        bbox_width,
        bbox_height,
        aspect_ratio,
        entity_count: layout.entities.len(),
        transit: realized.total,
        solid_transit: realized.solid_total,
        fluid_transit: realized.fluid_total,
        edges,
    })
}

/// `AR_score(L) = 1 - (AR(L) - 1) / (AR(native) - 1)`, RFC-064 §(a).
///
/// Degenerate case (native already square, `AR(native) = 1`): **`0.0`** if the
/// candidate is also square, else **`-1.0`**.
///
/// This deliberately does *not* match the RFC's original parenthetical, which
/// said `1.0` if `AR(L) = 1` else `0`. That rule contradicted the load-bearing
/// invariant stated two sentences above it in the same paragraph —
/// `AR_score(native) = 0` "by construction (no change scores neutral, not
/// positive)". Under the old rule a perfectly square native self-scored `1.0`,
/// so [`score_vs_native`] put the incumbent into its own ranking at composite
/// `0.5` rather than `0.0`, and **no candidate could outrank it on this axis**
/// (every non-square candidate scored `0.0`, every square one tied at `1.0`).
/// Amended to the shape [`transit_score`] already uses — no-change is neutral,
/// strictly-worse is a negative sentinel — which is also what that function's
/// doc comment already claimed the two shared. RFC text and decision log
/// updated together (2026-08-05).
pub fn ar_score(ar_candidate: f64, ar_native: f64) -> f64 {
    if (ar_native - 1.0).abs() < DEGENERATE_EPS {
        if (ar_candidate - 1.0).abs() < DEGENERATE_EPS { 0.0 } else { -1.0 }
    } else {
        1.0 - (ar_candidate - 1.0) / (ar_native - 1.0)
    }
}

/// `Transit_score(L) = 1 - Transit(L) / Transit(native)`, RFC-064 §(b).
///
/// Degenerate case (`Transit(native) == 0`, i.e. native has zero production
/// edges) is not specified by the RFC's text. Treated analogously to
/// `AR_score`'s native-square rule, since both share the same shape (no
/// meaningful baseline to normalize a change against): `0.0` if the
/// candidate's transit is also zero (no change), else the candidate is
/// worse than a zero-edge native by definition, so this returns a large
/// negative sentinel rather than dividing by zero. This is a decision made
/// here, not in the RFC — flag for the RFC's decision log.
///
/// The claimed analogy was false when written: [`ar_score`] returned `1.0`,
/// not `0.0`, for its no-change degenerate case. That was an RFC defect,
/// amended 2026-08-05 — the two now genuinely share this shape.
pub fn transit_score(transit_candidate: f64, transit_native: f64) -> f64 {
    if transit_native.abs() < DEGENERATE_EPS {
        if transit_candidate.abs() < DEGENERATE_EPS { 0.0 } else { -1.0 }
    } else {
        1.0 - transit_candidate / transit_native
    }
}

/// `w_AR`/`w_T` in RFC-064 §(d)'s composite. Default `0.5`/`0.5`, per the
/// RFC's own text — "provisional pending Phase 0" — and confirmed unchanged
/// by Phase 0's decision log (2026-08-01: gate cleared at default weights,
/// reweighting allowance unused).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompositeWeights {
    pub w_ar: f64,
    pub w_transit: f64,
}

impl Default for CompositeWeights {
    fn default() -> Self {
        Self { w_ar: 0.5, w_transit: 0.5 }
    }
}

/// Scores for one candidate layout relative to its native incumbent.
/// `native` always scores `ar_score: 0.0, transit_score: 0.0` by
/// construction when compared to itself — never asserted here since callers
/// build this for arbitrary candidate/native pairs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectiveScores {
    pub ar_score: f64,
    /// `None` = **not comparable**: the two measures have different edge
    /// counts, so they are not from the same solve and no transit comparison
    /// is meaningful. That also covers a zero-edge candidate against a
    /// nonzero native, which would otherwise report `Transit 0.0` as
    /// `transit_score = +1.0` ("100% shorter") and rank as a perfect layout
    /// (PR #569 adversarial review, finding 2 — the Phase 3 gate driver hit
    /// exactly that artifact).
    ///
    /// It no longer signals *unattribution*. That was the other way this
    /// could be `None` while this module carried its own partial-capable
    /// measurement; §(b) makes an unmeasurable edge fail the whole measure
    /// instead, so a `LayoutMeasure` in hand is always fully attributed.
    ///
    /// The composite treats `None` as `0.0` — neutral: no claimed win, no
    /// claimed loss.
    pub transit_score: Option<f64>,
    pub candidate_total_edges: usize,
    pub native_total_edges: usize,
    /// Edges the transit comparison covered: `0` when `transit_score` is
    /// `None`, otherwise the full edge count of both sides.
    ///
    /// **Effectively vestigial.** It existed to express partial coverage —
    /// "`Some(score)` over 1 common edge of 10 is weaker evidence than over
    /// 10 of 10" — and under §(b) there is no partial coverage to express.
    /// Retained because callers report it alongside a transit claim, and
    /// because a future §(b) amendment that permits partial measurement would
    /// need it back.
    pub common_attributed_edges: usize,
    /// `(entities(L) - entities(native)) / entities(native)`, RFC-064 §(c).
    pub delta_entities_pct: f64,
    /// Non-gating report-only flag, RFC-064 §(c): fires when
    /// `delta_entities_pct` exceeds [`ENTITY_GROWTH_WARN_PCT`]. Never used
    /// for ranking — see [`rank_admissible`].
    pub entity_growth_warn: bool,
    /// `w_AR * ar_score + w_T * transit_score`, RFC-064 §(d).
    pub composite: f64,
}

/// [`score_vs_native_weighted`] at the RFC's default weights (0.5/0.5).
pub fn score_vs_native(candidate: &LayoutMeasure, native: &LayoutMeasure) -> ObjectiveScores {
    score_vs_native_weighted(candidate, native, CompositeWeights::default())
}

/// Implements RFC-064 §(a)-(d): `AR_score`, `Transit_score`,
/// `ΔEntities%`, and the weighted composite. Does not implement §(d)'s
/// admissibility gate (step 1) — that is a sim-anchored, external fact
/// about `candidate`, not something derivable from two [`LayoutMeasure`]s.
/// See [`rank_admissible`] for steps 2-4 of the ranking rule.
pub fn score_vs_native_weighted(
    candidate: &LayoutMeasure,
    native: &LayoutMeasure,
    weights: CompositeWeights,
) -> ObjectiveScores {
    let ars = ar_score(candidate.aspect_ratio, native.aspect_ratio);
    let cand_total = candidate.edges.len();
    let native_total = native.edges.len();
    // Both measures are TOTAL by construction now — §(b) makes an unmeasurable
    // edge fatal, so a `LayoutMeasure` that exists covers every edge. The
    // common-subset machinery this function used to run (summing only edges
    // both sides attributed, to avoid penalising a candidate that measured
    // MORE than the native) described a partial state the conforming metric
    // cannot produce, and is gone with it.
    //
    // The length guard stays, and is the whole guard now: both edge lists come
    // from the same `ProductionSignature` order, so equal length means they
    // pair by index and unequal length means the two measures are not from one
    // solve — in which case no transit comparison is valid at all. That also
    // still closes the zero-edge-candidate hole (a 0-edge candidate against a
    // nonzero native must not score a "perfect" +1.0).
    // NOTE the guard is length-mismatch ONLY. An earlier cut of this also
    // refused `cand_total == 0`, which broke the same "native scores 0.0
    // against itself by construction" invariant that §(a)'s degenerate rule
    // was amended for: an edgeless layout measured against ITSELF is no
    // change, not absence of evidence. The zero-edge-candidate hole stays
    // closed regardless, because a 0-edge candidate against a nonzero native
    // is a length mismatch and lands in the `None` branch anyway.
    let (ts, common_attributed) = if cand_total != native_total {
        (None, 0)
    } else {
        debug_assert!(
            candidate
                .edges
                .iter()
                .zip(&native.edges)
                .all(|(c, n)| c.item == n.item && c.consumer_recipe == n.consumer_recipe),
            "paired edges must describe the same production edge",
        );
        (
            Some(transit_score(candidate.transit, native.transit)),
            cand_total,
        )
    };
    let delta_entities_pct = if native.entity_count == 0 {
        0.0
    } else {
        (candidate.entity_count as f64 - native.entity_count as f64) / native.entity_count as f64
    };
    ObjectiveScores {
        ar_score: ars,
        transit_score: ts,
        candidate_total_edges: cand_total,
        native_total_edges: native_total,
        common_attributed_edges: common_attributed,
        delta_entities_pct,
        entity_growth_warn: delta_entities_pct > ENTITY_GROWTH_WARN_PCT,
        composite: weights.w_ar * ars + weights.w_transit * ts.unwrap_or(0.0),
    }
}

/// Ranks already-ADMISSIBLE candidates per RFC-064 §(d) steps 2-4:
/// composite descending; ties within [`COMPOSITE_TIE_EPSILON`] broken by
/// lower `ΔEntities%`; remaining ties by lower absolute entity count, then
/// by `K`'s own `Ord` (the RFC's "deterministic candidate-id order").
/// Callers must pre-filter to admissible candidates themselves — step 1 of
/// the RFC's rule is a sim-anchored fact this module cannot compute.
///
/// **ε-banding is anchored at each band's leader, not pairwise.** Pairwise
/// "within ε of my neighbor" is non-transitive (A~B, B~C can hold while A~C
/// does not), and a sort comparator built on it produced input-order-
/// dependent winners (PR #569 adversarial review, finding 1) — directly
/// violating §(d) step 4's own reproducibility clause. Here a band is
/// closed over "within ε of the band's BEST composite": deterministic under
/// any input permutation. This is a formalization decision the RFC's text
/// leaves open; recorded in the RFC-064 decision log.
pub fn rank_admissible<K: Clone + Ord>(candidates: &[(K, ObjectiveScores, usize)]) -> Vec<K> {
    let tie_break = |a: usize, b: usize| {
        let (ka, sa, ea) = &candidates[a];
        let (kb, sb, eb) = &candidates[b];
        if (sa.delta_entities_pct - sb.delta_entities_pct).abs() > DEGENERATE_EPS {
            return sa
                .delta_entities_pct
                .partial_cmp(&sb.delta_entities_pct)
                .unwrap_or(std::cmp::Ordering::Equal);
        }
        if ea != eb {
            return ea.cmp(eb);
        }
        ka.cmp(kb)
    };

    // Pass 1: strict total order — composite descending, full tie-break
    // chain as secondary so the pre-sort itself is already deterministic.
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| {
        candidates[b]
            .1
            .composite
            .partial_cmp(&candidates[a].1.composite)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| tie_break(a, b))
    });

    // Pass 2: walk the descending list, close each band over "within ε of
    // the band leader", and re-rank each band by the tie-break chain alone
    // (§(d) step 3: within a tie band, composite differences are noise).
    let mut ranked: Vec<usize> = Vec::with_capacity(order.len());
    let mut i = 0;
    while i < order.len() {
        let leader = candidates[order[i]].1.composite;
        let mut j = i + 1;
        while j < order.len() && leader - candidates[order[j]].1.composite <= COMPOSITE_TIE_EPSILON
        {
            j += 1;
        }
        let band = &mut order[i..j];
        band.sort_by(|&a, &b| tie_break(a, b));
        ranked.extend_from_slice(band);
        i = j;
    }
    ranked.into_iter().map(|i| candidates[i].0.clone()).collect()
}

// ---------------------------------------------------------------------------
// Non-pole bounding box
// ---------------------------------------------------------------------------

/// Same predicate used throughout `bus::layout.rs`/`power_wires.rs` (e.g.
/// `layout.rs:2132`: `e.name.ends_with("electric-pole") || e.name ==
/// "substation"`) — no dedicated `common::is_pole` helper exists to reuse,
/// so this mirrors that inline convention rather than inventing a new one.
fn is_pole_entity(name: &str) -> bool {
    name.ends_with("electric-pole") || name == "substation"
}

/// `(min_x, min_y, max_x, max_y)` over every non-pole entity's oriented
/// footprint (`common::oriented_entity_dims`), RFC-064 §(a)'s "non-pole entity
/// bounding box" convention. `None` if `layout` has no non-pole entities.
fn non_pole_bbox(layout: &LayoutResult) -> Option<(i32, i32, i32, i32)> {
    let mut acc: Option<(i32, i32, i32, i32)> = None;
    for e in &layout.entities {
        if is_pole_entity(&e.name) {
            continue;
        }
        let (w, h) = crate::common::oriented_entity_dims(&e.name, e.direction);
        let (x0, y0, x1, y1) = (e.x, e.y, e.x + w, e.y + h);
        acc = Some(match acc {
            None => (x0, y0, x1, y1),
            Some((ax0, ay0, ax1, ay1)) => (ax0.min(x0), ay0.min(y0), ax1.max(x1), ay1.max(y1)),
        });
    }
    acc
}

// ---------------------------------------------------------------------------
// Realized transit measurement lives in `bus::transit`, not here
// ---------------------------------------------------------------------------
//
// This module used to carry ~340 lines implementing RFC-064 §(b) a second
// time: a solid belt/underground Dijkstra, a fluid BFS, port discovery, and
// their aggregation. It disagreed with `bus::transit` by -29% to +98% on real
// fixtures, because it aggregated over the wrong side of the edge (mean over
// producer ports rather than §(b)'s "arithmetic mean of those
// consumer-terminal distances"), averaged over whatever it could reach
// instead of refusing, and modelled no pipe-to-ground jump.
//
// It was deleted rather than reconciled: §(b) already specifies the
// aggregation, the refusal semantics and the direct-insertion rule, and
// `bus::transit` is the conforming reading of all three. Two implementations
// of one spec clause is the defect; picking the better one and keeping both
// would not have fixed it.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DICoupling, EntityDirection, ItemFlow, MachineSpec, PlacedEntity};

    // -----------------------------------------------------------------
    // Formula-level tests (no layout construction needed)
    // -----------------------------------------------------------------

    /// RFC-064's own worked calibration anchor (§(a)): chain-mil5ore's
    /// Factorio-verified 3-fold, AR(native)=17.3, AR(folded)=1.09 ->
    /// AR_score ≈ 0.9945. Pins the formula exactly against the RFC's own
    /// numbers, independent of reproducing the (expensive) fixture.
    #[test]
    fn ar_score_matches_rfc_calibration_anchor() {
        let score = ar_score(1.09, 17.3);
        assert!((score - 0.9945).abs() < 0.001, "got {score}, expected ~0.9945");
    }

    /// The invariant the RFC calls "by construction". This test previously
    /// asserted only the non-degenerate case (3.0 vs 3.0), which is exactly
    /// the case that could not fail — the square-native branch was the one
    /// that violated it, and it went untested. Enumerate both.
    #[test]
    fn ar_score_native_is_zero_by_construction() {
        assert_eq!(ar_score(3.0, 3.0), 0.0);
        assert_eq!(ar_score(17.3, 17.3), 0.0);
        // The degenerate branch: a square native scored against itself.
        assert_eq!(ar_score(1.0, 1.0), 0.0);
    }

    /// A square native cannot be outranked on the AR axis, but it must not
    /// outrank everything either — it sits at neutral, like any other native.
    #[test]
    fn ar_score_square_native_does_not_self_inflate() {
        // Regression: this returned 1.0, putting the incumbent into its own
        // ranking at composite 0.5 and making it unbeatable on this axis.
        assert_eq!(ar_score(1.0, 1.0), 0.0);
        // Anything less square than an already-square native is strictly worse.
        assert!(ar_score(2.0, 1.0) < 0.0);
    }

    #[test]
    fn ar_score_perfect_square_is_one() {
        assert_eq!(ar_score(1.0, 5.0), 1.0);
    }

    #[test]
    fn ar_score_more_elongated_than_native_is_negative() {
        assert!(ar_score(10.0, 5.0) < 0.0);
    }

    /// Degenerate case: `AR(native) = 1`. Amended 2026-08-05 — was
    /// `1.0`/`0.0`, which contradicted `AR_score(native) = 0`. Now mirrors
    /// [`transit_score`]'s zero-native rule: neutral for no change, negative
    /// sentinel for strictly worse.
    #[test]
    fn ar_score_degenerate_native_square() {
        assert_eq!(ar_score(1.0, 1.0), 0.0);
        assert_eq!(ar_score(2.0, 1.0), -1.0);
    }

    /// The two degenerate rules are documented as sharing one shape. Pin that,
    /// so they cannot drift apart again silently.
    #[test]
    fn degenerate_rules_agree_between_ar_and_transit() {
        assert_eq!(ar_score(1.0, 1.0), transit_score(0.0, 0.0));
        assert_eq!(ar_score(2.0, 1.0), transit_score(5.0, 0.0));
    }

    #[test]
    fn transit_score_no_change_is_zero() {
        assert_eq!(transit_score(100.0, 100.0), 0.0);
    }

    #[test]
    fn transit_score_shorter_is_positive() {
        let s = transit_score(60.0, 100.0);
        assert!((s - 0.4).abs() < DEGENERATE_EPS);
    }

    #[test]
    fn transit_score_longer_is_negative() {
        assert!(transit_score(150.0, 100.0) < 0.0);
    }

    /// Degenerate case: zero-edge native (no production edges at all).
    #[test]
    fn transit_score_degenerate_zero_native() {
        assert_eq!(transit_score(0.0, 0.0), 0.0);
        assert_eq!(transit_score(50.0, 0.0), -1.0);
    }

    #[test]
    fn composite_default_weights_are_half_half() {
        let w = CompositeWeights::default();
        assert_eq!(w.w_ar, 0.5);
        assert_eq!(w.w_transit, 0.5);
    }

    fn measure_with(ar: f64, transit: f64, entities: usize) -> LayoutMeasure {
        LayoutMeasure {
            bbox_width: 1,
            bbox_height: 1,
            aspect_ratio: ar,
            entity_count: entities,
            transit,
            edges: vec![],
            solid_transit: transit,
            fluid_transit: 0.0,
        }
    }

    #[test]
    fn score_vs_native_native_scores_zero_composite() {
        let native = measure_with(5.0, 100.0, 500);
        let scores = score_vs_native(&native, &native);
        assert_eq!(scores.ar_score, 0.0);
        assert_eq!(scores.transit_score, Some(0.0));
        assert_eq!(scores.delta_entities_pct, 0.0);
        assert_eq!(scores.composite, 0.0);
        assert!(!scores.entity_growth_warn);
    }

    #[test]
    fn score_vs_native_entity_growth_warn_threshold() {
        let native = measure_with(5.0, 100.0, 1000);
        let just_under = measure_with(5.0, 100.0, 1519); // +51.9%
        let just_over = measure_with(5.0, 100.0, 1521); // +52.1%
        assert!(!score_vs_native(&just_under, &native).entity_growth_warn);
        assert!(score_vs_native(&just_over, &native).entity_growth_warn);
    }

    /// Synthetic calibration case (RFC-064 Phase 1 found a real fixture,
    /// stress-ec-60s-red's 2-fold, at AR_score ≈ -90.24; reproducing that
    /// exact fold search is out of scope here). A candidate far MORE
    /// elongated than a near-square native must score deeply negative on
    /// AR and lose the composite outright, demonstrating the "fold found
    /// is not fold good" rule the composite exists to enforce.
    #[test]
    fn negative_ar_score_candidate_loses_composite_to_native() {
        let native = measure_with(1.02, 500.0, 1000); // near-square, cheap transit
        let bad_candidate = measure_with(95.0, 500.0, 1197); // same transit, badly elongated, +19.7% entities
        let scores = score_vs_native(&bad_candidate, &native);
        assert!(scores.ar_score < -50.0, "expected deeply negative AR_score, got {}", scores.ar_score);
        assert!(scores.composite < 0.0, "a badly-elongated candidate must lose the composite to native (0.0)");

        let ranked = rank_admissible(&[
            ("native".to_string(), score_vs_native(&native, &native), native.entity_count),
            ("bad_candidate".to_string(), scores, bad_candidate.entity_count),
        ]);
        assert_eq!(ranked[0], "native", "native (composite 0.0) must outrank the negative-AR_score candidate");
    }

    /// Test-literal helper: an [`ObjectiveScores`] with the given composite
    /// and ΔEntities%, full transit evidence, no warn.
    fn scores_of(composite: f64, delta_entities_pct: f64) -> ObjectiveScores {
        ObjectiveScores {
            ar_score: composite,
            transit_score: Some(composite),
            candidate_total_edges: 1,
            native_total_edges: 1,
            common_attributed_edges: 1,
            delta_entities_pct,
            entity_growth_warn: false,
            composite,
        }
    }

    #[test]
    fn rank_admissible_orders_by_composite_then_tie_break() {
        let a = scores_of(0.9, 0.5);
        let b = scores_of(0.4, 0.1);
        let ranked = rank_admissible(&[("b".to_string(), b, 10), ("a".to_string(), a, 10)]);
        assert_eq!(ranked, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn rank_admissible_tie_break_prefers_lower_delta_entities() {
        // Composites within COMPOSITE_TIE_EPSILON of each other.
        let a = scores_of(0.500, 0.30);
        let b = scores_of(0.505, 0.05);
        let ranked = rank_admissible(&[("a".to_string(), a, 100), ("b".to_string(), b, 100)]);
        assert_eq!(ranked, vec!["b".to_string(), "a".to_string()], "lower ΔEntities% must win a composite tie");
    }

    /// PR #569 adversarial review, finding 1 — the reviewer's own
    /// counterexample shape: three candidates where a~b and b~c are each
    /// inside the ε=0.02 window but a~c is not (chained, non-transitive
    /// ties). Under the old pairwise comparator the winner depended on
    /// input order (three different winners across orderings). Leader-
    /// anchored banding must produce ONE ranking under every permutation.
    #[test]
    fn rank_admissible_is_input_order_independent_under_chained_ties() {
        let a = ("a".to_string(), scores_of(0.100, 0.30), 100);
        let b = ("b".to_string(), scores_of(0.085, 0.10), 100);
        let c = ("c".to_string(), scores_of(0.070, 0.01), 100);
        let perms: [[&(String, ObjectiveScores, usize); 3]; 6] = [
            [&a, &b, &c], [&a, &c, &b], [&b, &a, &c],
            [&b, &c, &a], [&c, &a, &b], [&c, &b, &a],
        ];
        let mut rankings = Vec::new();
        for p in &perms {
            let field: Vec<_> = p.iter().map(|t| (*t).clone()).collect();
            rankings.push(rank_admissible(&field));
        }
        for r in &rankings[1..] {
            assert_eq!(r, &rankings[0], "ranking must not depend on input order");
        }
        // Band semantics: leader a's band is {a, b} (c is 0.030 > ε from a),
        // so b wins the band on lower ΔEntities%; c ranks after the band.
        assert_eq!(rankings[0], vec!["b".to_string(), "a".to_string(), "c".to_string()]);
    }

    /// PR #569 adversarial review, finding 2: a candidate whose every
    /// production edge is unattributed measures transit 0.0 — which the old
    /// scorer turned into transit_score = +1.0, indistinguishable from a
    /// genuinely perfect layout. It must now score `None` (no evidence),
    /// contribute 0.0 to the composite, and expose the counts.
    #[test]
    fn fully_unattributed_transit_scores_none_not_perfect() {
        let edge = EdgeMeasurement {
            producer_recipes: vec!["iron-gear-wheel".to_string()],
            item: "iron-gear-wheel".to_string(),
            consumer_recipe: "automation-science-pack".to_string(),
            rate: 1.0,
            is_fluid: false,
            path_length: 100.0,
            producer_terminals: 1,
            consumer_terminals: 1,
            direct_insertion: false,
        };
        let mut native = measure_with(1.5, 100.0, 500);
        native.edges = vec![edge.clone()];
        // A candidate from a DIFFERENT solve (no edges) is not evidence of a
        // perfect layout. This used to be reachable by a candidate whose edges
        // all failed attribution; §(b) now makes that a refused measurement
        // instead, so the zero-edge case survives only as a cross-solve guard.
        let ghost = measure_with(1.5, 0.0, 500);
        let scores = score_vs_native(&ghost, &native);
        assert_eq!(scores.transit_score, None, "zero edges is no evidence, not a win");
        assert_eq!(scores.common_attributed_edges, 0);
        assert_eq!(scores.composite, 0.0, "composite must treat missing transit evidence as neutral");
    }

    /// A successful measure covers every edge, so transit compares whole
    /// against whole.
    ///
    /// This replaces `partial_attribution_compares_common_subset_only`, which
    /// pinned the common-subset machinery that existed to stop a candidate
    /// being punished for measuring MORE edges than the native. Under RFC-064
    /// §(b) neither side can measure a subset — an unreachable terminal makes
    /// the whole measurement fail — so the asymmetry that test defended
    /// against cannot arise, and the machinery is gone with it.
    #[test]
    fn full_attribution_compares_totals_directly() {
        let edge = |len: f64| EdgeMeasurement {
            producer_recipes: vec!["iron-gear-wheel".to_string()],
            item: "iron-gear-wheel".to_string(),
            consumer_recipe: "automation-science-pack".to_string(),
            rate: 1.0,
            is_fluid: false,
            path_length: len,
            producer_terminals: 1,
            consumer_terminals: 1,
            direct_insertion: false,
        };
        let mut native = measure_with(1.5, 200.0, 500);
        native.edges = vec![edge(100.0), edge(100.0)];
        let mut cand = measure_with(1.5, 100.0, 500);
        cand.edges = vec![edge(50.0), edge(50.0)];

        let scores = score_vs_native(&cand, &native);
        assert_eq!(scores.common_attributed_edges, 2);
        assert_eq!(scores.transit_score, Some(0.5), "half the transit → +0.5");
    }

    /// Round-3 bot review, finding B: mismatched edge-list lengths mean the
    /// two measures are not from the same solve — no transit comparison is
    /// valid (previously a zero-edge candidate against a nonzero native
    /// scored a "perfect" +1.0 through the old guard's gap).
    #[test]
    fn mismatched_edge_sets_are_evidence_free() {
        let mut native = measure_with(1.5, 100.0, 500);
        native.edges = vec![EdgeMeasurement {
            producer_recipes: vec!["iron-gear-wheel".to_string()],
            item: "iron-gear-wheel".to_string(),
            consumer_recipe: "automation-science-pack".to_string(),
            rate: 1.0,
            is_fluid: false,
            path_length: 100.0,
            producer_terminals: 1,
            consumer_terminals: 1,
            direct_insertion: false,
        }];
        let zero_edge_cand = measure_with(1.5, 0.0, 500);
        let scores = score_vs_native(&zero_edge_cand, &native);
        assert_eq!(scores.transit_score, None, "cross-solve/zero-edge comparison is no evidence");
        assert_eq!(scores.common_attributed_edges, 0);
    }

    // -----------------------------------------------------------------
    // measure(): non-pole bbox
    // -----------------------------------------------------------------

    fn machine(x: i32, y: i32, recipe: &str, name: &str) -> PlacedEntity {
        PlacedEntity { name: name.into(), x, y, recipe: Some(recipe.into()), ..Default::default() }
    }

    fn pole(x: i32, y: i32) -> PlacedEntity {
        PlacedEntity { name: "medium-electric-pole".into(), x, y, ..Default::default() }
    }

    #[test]
    fn non_pole_bbox_excludes_poles() {
        // A pole sitting far outside the machine footprint must not widen
        // the bbox RFC-064 §(a) scores aspect ratio on.
        let layout = LayoutResult {
            entities: vec![
                machine(0, 0, "iron-gear-wheel", "assembling-machine-1"),
                pole(50, 50),
            ],
            width: 60,
            height: 60,
            ..Default::default()
        };
        let (w, h) = crate::common::oriented_entity_dims("assembling-machine-1", EntityDirection::North);
        let bbox = non_pole_bbox(&layout).expect("non-pole entities present");
        assert_eq!(bbox, (0, 0, w, h));
    }

    #[test]
    fn non_pole_bbox_none_when_only_poles() {
        let layout = LayoutResult { entities: vec![pole(0, 0)], width: 10, height: 10, ..Default::default() };
        assert!(non_pole_bbox(&layout).is_none());
    }

    // -----------------------------------------------------------------
    // measure(): zero-edge layout (degenerate case #2)
    // -----------------------------------------------------------------

    /// A single-recipe layout with only external inputs/outputs produces
    /// zero `ProductionEdge`s (`ProductionSignature::from_solver` only
    /// creates an edge when another machine in the graph produces the
    /// input) — `Transit` must be exactly 0.0 with an empty edge list, not
    /// an error.
    #[test]
    fn measure_zero_edge_layout() {
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-1".into(),
                recipe: "iron-gear-wheel".into(),
                count: 1.0,
                inputs: vec![ItemFlow { item: "iron-plate".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
                outputs: vec![ItemFlow { item: "iron-gear-wheel".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
                ..Default::default()
            }],
            external_inputs: vec![ItemFlow { item: "iron-plate".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
            external_outputs: vec![ItemFlow {
                item: "iron-gear-wheel".into(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            ..Default::default()
        };
        let layout = LayoutResult {
            entities: vec![machine(0, 0, "iron-gear-wheel", "assembling-machine-1")],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let m = measure(&layout, &sr).expect("measure should succeed");
        assert!(m.edges.is_empty());
        assert_eq!(m.transit, 0.0);
    }

    #[test]
    fn measure_errors_on_no_non_pole_entities() {
        let sr = SolverResult::default();
        let layout = LayoutResult { entities: vec![pole(0, 0)], width: 10, height: 10, ..Default::default() };
        assert!(measure(&layout, &sr).is_err());
    }

    // -----------------------------------------------------------------
    // measure(): realized solid-edge path length, hand-built geometry
    // -----------------------------------------------------------------

    fn belt(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity { name: "transport-belt".into(), x, y, direction: dir, carries: Some(item.into()), ..Default::default() }
    }

    fn inserter_at(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity { name: "inserter".into(), x, y, direction: dir, ..Default::default() }
    }
    // `measure_solid_edge_realized_path_length` and
    // `measure_direct_insertion_edge_uses_manhattan_fallback` lived here.
    // Both pinned THIS module's own measurement, which is gone — RFC-064 §(b)
    // is implemented once, in `bus::transit`, and that module carries the
    // equivalent pins (`measures_directed_surface_and_underground_span_costs`,
    // `direct_insertion_uses_actual_machine_port_span`,
    // `measures_fluid_paths_with_weight`).
    //
    // They are deleted rather than ported because their fixtures could not be
    // ported honestly: they identified producers and consumers by RECIPE and
    // used stub entity names ("producer-stub"), which `bus::transit`'s
    // machine-geometry discovery cannot see. Rewriting them against real
    // machine entities would have produced a second copy of tests
    // `bus::transit` already owns, against the same code.


    // -----------------------------------------------------------------
    // measure(): fluid edge weighting
    // -----------------------------------------------------------------

    fn pipe(x: i32, y: i32, item: &str) -> PlacedEntity {
        PlacedEntity { name: "pipe".into(), x, y, carries: Some(item.into()), ..Default::default() }
    }

    /// AM2's fluid ports: input north `(1,-1)`, output south `(1,3)`
    /// (`fluid_ports::AM2`). Two AM2-family stand-ins 6 tiles apart
    /// connected by a straight pipe run must measure exactly the pipe tile
    /// count, and the edge's contribution to `Transit` must carry
    /// [`FLUID_WEIGHT`], not the solid weight of 1.0.
    #[test]
    fn measure_fluid_edge_applies_fluid_weight() {
        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-2".into(),
                    recipe: "produce-f".into(),
                    count: 1.0,
                    outputs: vec![ItemFlow { item: "f".into(), rate: 4.0, is_fluid: true, module_id: 0 }],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "assembling-machine-2".into(),
                    recipe: "consume-f".into(),
                    count: 1.0,
                    inputs: vec![ItemFlow { item: "f".into(), rate: 4.0, is_fluid: true, module_id: 0 }],
                    ..Default::default()
                },
            ],
            external_outputs: vec![ItemFlow { item: "y".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
            ..Default::default()
        };

        // Producer AM2 at (0,0): output port at (0+1, 0+3) = (1,3).
        // Consumer AM2 at (0,10): input port at (0+1, 10-1) = (1,9).
        // Pipe run (1,3)..=(1,9): 7 tiles, BFS distance from (1,3) to
        // (1,9) is 6 hops.
        let mut entities = vec![machine(0, 0, "produce-f", "assembling-machine-2")];
        for y in 3..=9 {
            entities.push(pipe(1, y, "f"));
        }
        entities.push(machine(0, 10, "consume-f", "assembling-machine-2"));
        let layout = LayoutResult { entities, width: 10, height: 15, ..Default::default() };

        let m = measure(&layout, &sr).expect("measure should succeed");
        assert_eq!(m.edges.len(), 1);
        let edge = &m.edges[0];
        assert!(edge.is_fluid);
        assert_eq!(edge.path_length, 6.0);
        assert_eq!(m.transit, 4.0 * FLUID_WEIGHT * 6.0, "fluid edges must carry FLUID_WEIGHT, not the solid weight");
    }

    /// A fluid edge with no route at all must make the whole measurement
    /// refuse — never silently fall back to a proxy, and never report a
    /// partial result for the edges that did measure.
    #[test]
    fn measure_refuses_when_a_fluid_terminal_is_unreachable() {
        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-2".into(),
                    recipe: "produce-f".into(),
                    count: 1.0,
                    outputs: vec![ItemFlow { item: "f".into(), rate: 4.0, is_fluid: true, module_id: 0 }],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "assembling-machine-2".into(),
                    recipe: "consume-f".into(),
                    count: 1.0,
                    inputs: vec![ItemFlow { item: "f".into(), rate: 4.0, is_fluid: true, module_id: 0 }],
                    ..Default::default()
                },
            ],
            external_outputs: vec![ItemFlow { item: "y".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
            ..Default::default()
        };

        // No pipe entities at all between the two machines: producer's
        // output port and consumer's input port are both isolated tiles.
        let entities = vec![
            machine(0, 0, "produce-f", "assembling-machine-2"),
            machine(0, 20, "consume-f", "assembling-machine-2"),
        ];
        let layout = LayoutResult { entities, width: 10, height: 25, ..Default::default() };

        // RFC-064 §(b): "Any other unreachable terminal makes the metric
        // unmeasurable and the candidate INADMISSIBLE — never silently fall
        // back to Manhattan for a broken routed edge."
        //
        // This test previously asserted the opposite, because this module's
        // own measurement reported such an edge as `path_length: None` with
        // `unattributed_edge_count: 1` and carried on. That was documented
        // in-module as an honest known gap, and it is honest — but it is not
        // what §(b) specifies, and a partially-measured candidate could still
        // be scored and ranked. Delegating to `bus::transit` makes the
        // measurement refuse instead, which is both conforming and the
        // stronger guarantee: a candidate nobody can measure cannot be
        // admitted on the strength of the edges that did measure.
        let err = measure(&layout, &sr)
            .expect_err("an unreachable fluid terminal must make the whole measure refuse");
        assert!(
            err.contains("transit is not measurable"),
            "refusal should name the cause, got: {err}"
        );
    }
}
