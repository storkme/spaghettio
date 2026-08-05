//! RFC-064 "spaghetti objective" primitives (P1): the aspect-ratio score
//! and rate-weighted belt-transit metric, computed on a validated, fully
//! routed [`LayoutResult`] — never on an unrouted IR estimate, per
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
//! ## Realized per-edge path length: approach and known gaps
//!
//! [`measure`] computes `Transit(L)` per [`crate::bus::compaction::ProductionEdge`]
//! (the RFC's own edge model — reused verbatim from `bus::compaction`, never
//! re-derived) as a **realized physical path length** over the routed
//! geometry:
//!
//! - **Solid (belt/underground) edges**: for every producer machine of the
//!   edge's `producer_recipes` and every consumer machine of its
//!   `consumer_recipe`, the inserter that drops the item onto a belt tile
//!   (a "producer port") and the inserter that picks it up off a belt tile
//!   (a "consumer port") are located from `PlacedEntity` geometry. A
//!   Dijkstra search over the belt/underground tile graph (built from the
//!   same `pub(crate)` adjacency helpers `belt_detour.rs` uses —
//!   `belt_dir_map_from`/`build_ug_pairs`/`build_splitter_siblings` in
//!   `validate::belt_flow` — so this module never re-derives belt/UG/
//!   splitter adjacency) finds the shortest tile-count path from each
//!   producer port to the nearest consumer port, filtered to tiles carrying
//!   the edge's item. Splitters are crossed (both outputs reachable from
//!   either input, weight 2 per crossing — a documented over-connection,
//!   not a claim any specific item can freely choose lanes) rather than
//!   treated as hard walls the way `belt_detour`'s run decomposition does,
//!   since a producer-to-consumer path routinely crosses balancers.
//!   Direct-insertion edges (producer and consumer bridged by one inserter
//!   with no belt at all) fall back to the Manhattan distance between the
//!   inserter's pickup and drop tile, exactly as RFC-064's Phase 0 decision
//!   log (2026-08-01) describes ("Manhattan fallback only for
//!   direct-insertion edges").
//! - **Fluid edges**: producer/consumer fluid ports come from
//!   [`crate::fluid_ports::fluid_ports`] (the same geometry table the bus
//!   templates and fluid validator use); the path is a BFS over adjacent
//!   `pipe`/`pipe-to-ground` tiles carrying the edge's item. **Known gap,
//!   not silently papered over**: pipe-to-ground pairing (the underground
//!   jump) is NOT modeled here — plain 4-adjacency BFS cannot see across a
//!   genuinely buried pipe run with no surface tiles in between. A fluid
//!   edge routed underground for its entire span reports `path_length:
//!   None` (excluded from `Transit`, counted in `unattributed_edge_count`)
//!   rather than a silently wrong number. Reimplementing the validator's
//!   pipe-to-ground pairing (`validate::fluids::find_ptg_pairs`, private to
//!   that module) was judged out of scope for this additive pass; see the
//!   PR report for the sizing of that follow-up.
//! - When multiple producer or consumer instances exist for one edge, each
//!   producer port's shortest reachable distance is one sample; the edge's
//!   `path_length` is the arithmetic mean of all samples found. An edge
//!   with zero samples (no port pairing found by any means above) is
//!   `path_length: None` — excluded from `Transit(L)`, never substituted
//!   with a proxy (e.g. total belt length), per this project's own
//!   instruction that doing so would defeat the point of this module.
//! - When a physical layout offers multiple routes between a producer and
//!   consumer port (e.g. across a balancer), the **shortest** is used as
//!   the representative length — a deterministic choice, not a claim that
//!   every unit of the commodity takes that exact path.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::bus::compaction::{self, ProductionSignature};
use crate::common::{dir_to_vec, inserter_reach, is_belt_entity, is_inserter, is_ug_belt};
use crate::fluid_ports;
use crate::models::{LayoutResult, SolverResult};
use crate::validate::belt_flow;

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

/// One production edge's realized measurement. `path_length: None` means
/// this edge could not be attributed by any means this module implements
/// (see module docs "Known gaps") — it is excluded from
/// [`LayoutMeasure::transit`], not zero-filled.
#[derive(Debug, Clone)]
pub struct EdgeMeasurement {
    pub producer_recipes: Vec<String>,
    pub item: String,
    pub consumer_recipe: String,
    /// Raw (unweighted) solved rate, items/s or fluid-units/s.
    pub rate: f64,
    pub is_fluid: bool,
    /// Realized physical tile length of the routed path, or `None` if
    /// unattributed. See module docs for the attribution method and gaps.
    pub path_length: Option<f64>,
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
    /// `Σ rate(e) * fluid_weight?(e) * path_length(e)` over attributed
    /// edges only.
    pub transit: f64,
    /// Per-edge breakdown, for debugging and reporting (RFC-064 §(b)'s
    /// insistence that a per-category count never hide inside one number).
    pub edges: Vec<EdgeMeasurement>,
    /// Count of edges with `path_length: None` — see module docs.
    pub unattributed_edge_count: usize,
}

/// Compute [`LayoutMeasure`] for `layout`, the routed output of the solve
/// `solver` describes. Errors if `layout` has no non-pole entities (nothing
/// to measure) or if `solver`'s production graph cannot be derived.
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

    let sig = ProductionSignature::from_solver(solver)?;
    let graph = SolidGraph::build(layout);

    let mut edges = Vec::with_capacity(sig.edges.len());
    let mut transit = 0.0;
    let mut unattributed_edge_count = 0usize;
    for pe in &sig.edges {
        let rate = pe.rate as f64 / compaction::RATE_SCALE;
        let path_length = if pe.is_fluid {
            measure_fluid_edge(layout, &pe.producer_recipes, &pe.consumer_recipe, &pe.item)
        } else {
            graph.measure_edge(layout, &pe.producer_recipes, &pe.consumer_recipe, &pe.item)
        };
        match path_length {
            Some(pl) => {
                let weight = if pe.is_fluid { FLUID_WEIGHT } else { 1.0 };
                transit += rate * weight * pl;
            }
            None => unattributed_edge_count += 1,
        }
        edges.push(EdgeMeasurement {
            producer_recipes: pe.producer_recipes.clone(),
            item: pe.item.clone(),
            consumer_recipe: pe.consumer_recipe.clone(),
            rate,
            is_fluid: pe.is_fluid,
            path_length,
        });
    }

    Ok(LayoutMeasure {
        bbox_width,
        bbox_height,
        aspect_ratio,
        entity_count: layout.entities.len(),
        transit,
        edges,
        unattributed_edge_count,
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
    /// `None` = **no transit evidence**: one side of the comparison has
    /// production edges but attributed NONE of them. A `Transit` of `0.0`
    /// from total unattribution would otherwise masquerade as
    /// `transit_score = +1.0` ("100% shorter") — indistinguishable from a
    /// genuinely perfect candidate (PR #569 adversarial review, finding 2;
    /// the RFC-064 Phase 3 gate driver hit exactly this artifact). The
    /// composite treats `None` as `0.0` — neutral: no claimed win, no
    /// claimed loss — and the four `*_edges` counts below let callers
    /// report the evidence gap instead of silently ranking through it.
    pub transit_score: Option<f64>,
    /// Attributed / total production edges on the candidate side.
    pub candidate_attributed_edges: usize,
    pub candidate_total_edges: usize,
    /// Attributed / total production edges on the native side.
    pub native_attributed_edges: usize,
    pub native_total_edges: usize,
    /// Edges attributed on BOTH sides — the only subset `transit_score` is
    /// computed over (each side's own sum would bias the comparison, see
    /// `score_vs_native_weighted`). Callers reporting a transit claim
    /// should report this alongside it: `Some(score)` over 1 common edge
    /// of 10 is much weaker evidence than over 10 of 10.
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
    let cand_attr = cand_total - candidate.unattributed_edge_count;
    let native_total = native.edges.len();
    let native_attr = native_total - native.unattributed_edge_count;
    // Transit is only comparable over edges BOTH sides attributed. Summing
    // each side over its own attributed subset biases the comparison in
    // both directions: a candidate that attributes MORE edges (e.g. a
    // fluid run the native routed underground) is penalized for the extra
    // terms, and one that attributes fewer gets an artificially short sum
    // (round-3 bot review, finding A). Both measures derive their edge
    // list from the same `ProductionSignature` order, so edges pair by
    // index; mismatched lengths mean the two measures are not from the
    // same solve and no transit comparison is valid at all — which also
    // closes the zero-edge-candidate hole (finding B: `cand_total == 0`
    // vs a nonzero native previously scored a "perfect" +1.0).
    let mut common_attributed = 0usize;
    let ts = if cand_total != native_total {
        None
    } else if candidate.unattributed_edge_count == 0 && native.unattributed_edge_count == 0 {
        // Fully attributed on both sides: the pre-summed totals ARE the
        // common-subset sums.
        common_attributed = cand_total;
        Some(transit_score(candidate.transit, native.transit))
    } else {
        let (mut cand_common, mut native_common) = (0.0f64, 0.0f64);
        for (ce, ne) in candidate.edges.iter().zip(&native.edges) {
            debug_assert!(
                ce.item == ne.item && ce.consumer_recipe == ne.consumer_recipe,
                "paired edges must describe the same production edge",
            );
            if let (Some(cl), Some(nl)) = (ce.path_length, ne.path_length) {
                let cw = if ce.is_fluid { FLUID_WEIGHT } else { 1.0 };
                let nw = if ne.is_fluid { FLUID_WEIGHT } else { 1.0 };
                cand_common += ce.rate * cw * cl;
                native_common += ne.rate * nw * nl;
                common_attributed += 1;
            }
        }
        if common_attributed == 0 {
            None
        } else {
            Some(transit_score(cand_common, native_common))
        }
    };
    let delta_entities_pct = if native.entity_count == 0 {
        0.0
    } else {
        (candidate.entity_count as f64 - native.entity_count as f64) / native.entity_count as f64
    };
    ObjectiveScores {
        ar_score: ars,
        transit_score: ts,
        candidate_attributed_edges: cand_attr,
        candidate_total_edges: cand_total,
        native_attributed_edges: native_attr,
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
// Solid (belt/underground) realized-path graph
// ---------------------------------------------------------------------------

/// Tile-level directed graph over a layout's belt/underground network,
/// built once per [`measure`] call and reused across every solid edge.
/// Adjacency comes from `validate::belt_flow`'s `pub(crate)` helpers
/// (`belt_dir_map_from`/`build_ug_pairs`/`build_splitter_siblings`) — the
/// same ones `validate::belt_detour` uses — so this module never re-derives
/// belt/UG/splitter adjacency rules.
struct SolidGraph {
    dir_map: FxHashMap<(i32, i32), crate::models::EntityDirection>,
    splitter_tiles: FxHashSet<(i32, i32)>,
    splitter_siblings: FxHashMap<(i32, i32), (i32, i32)>,
    ug_pairs: FxHashMap<(i32, i32), (i32, i32)>,
    ug_input: FxHashSet<(i32, i32)>,
    carries: FxHashMap<(i32, i32), String>,
}

impl SolidGraph {
    fn build(layout: &LayoutResult) -> Self {
        let dir_map = belt_flow::belt_dir_map_from(&layout.entities);
        let splitter_siblings = belt_flow::build_splitter_siblings(layout);
        let splitter_tiles: FxHashSet<(i32, i32)> = splitter_siblings.keys().copied().collect();
        let ug_pairs = belt_flow::build_ug_pairs(layout);

        let mut ug_input = FxHashSet::default();
        let mut carries = FxHashMap::default();
        for e in &layout.entities {
            if is_ug_belt(&e.name) && e.io_type.as_deref() == Some("input") {
                ug_input.insert((e.x, e.y));
            }
            if is_belt_entity(&e.name) {
                if let Some(c) = &e.carries {
                    carries.insert((e.x, e.y), c.clone());
                }
            }
        }

        Self { dir_map, splitter_tiles, splitter_siblings, ug_pairs, ug_input, carries }
    }

    /// One forward step from tile `t`: `(next_tile, weight)` pairs.
    /// Multiple results only at a splitter crossing (both outputs reachable
    /// from either input, weight 2 — see module docs). Never steps onto or
    /// from a splitter footprint tile as a plain belt tile, mirroring
    /// `belt_detour`'s "splitters are anchors, not run tiles" rule.
    fn step(&self, t: (i32, i32)) -> Vec<((i32, i32), i64)> {
        if self.ug_input.contains(&t) {
            return match self.ug_pairs.get(&t) {
                Some(&out) if self.dir_map.contains_key(&out) && !self.splitter_tiles.contains(&out) => {
                    let dist = (out.0 - t.0).unsigned_abs() as i64 + (out.1 - t.1).unsigned_abs() as i64;
                    vec![(out, dist.max(1))]
                }
                _ => vec![],
            };
        }
        let Some(&d) = self.dir_map.get(&t) else { return vec![] };
        let (dx, dy) = dir_to_vec(d);
        let next = (t.0 + dx, t.1 + dy);
        if let Some(&sib) = self.splitter_siblings.get(&next) {
            // Crossing a splitter: `next` is one of its two footprint
            // tiles, both facing the same direction (belt_dir_map_from
            // stamps both tiles with the splitter's own direction).
            let sdir = self.dir_map[&next];
            let (sdx, sdy) = dir_to_vec(sdir);
            let mut outs: Vec<((i32, i32), i64)> = Vec::new();
            for cand in [(next.0 + sdx, next.1 + sdy), (sib.0 + sdx, sib.1 + sdy)] {
                if self.dir_map.contains_key(&cand)
                    && !self.splitter_tiles.contains(&cand)
                    && !outs.iter().any(|&(c, _)| c == cand)
                {
                    outs.push((cand, 2));
                }
            }
            return outs;
        }
        if self.dir_map.contains_key(&next) {
            vec![(next, 1)]
        } else {
            vec![]
        }
    }

    /// Dijkstra shortest distance from `start` to the nearest tile in
    /// `targets`, restricted to tiles carrying `item`. `start` itself must
    /// already carry `item` (callers only ever pass producer ports, which
    /// are selected for exactly that).
    fn shortest_to_any(&self, start: (i32, i32), targets: &FxHashSet<(i32, i32)>, item: &str) -> Option<i64> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        if targets.contains(&start) {
            return Some(0);
        }
        let mut dist: FxHashMap<(i32, i32), i64> = FxHashMap::default();
        let mut heap: BinaryHeap<Reverse<(i64, (i32, i32))>> = BinaryHeap::new();
        dist.insert(start, 0);
        heap.push(Reverse((0, start)));
        while let Some(Reverse((d, t))) = heap.pop() {
            if d > *dist.get(&t).unwrap_or(&i64::MAX) {
                continue;
            }
            if targets.contains(&t) {
                return Some(d);
            }
            for (next, w) in self.step(t) {
                if self.carries.get(&next).map(String::as_str) != Some(item) {
                    continue;
                }
                let nd = d + w;
                if nd < *dist.get(&next).unwrap_or(&i64::MAX) {
                    dist.insert(next, nd);
                    heap.push(Reverse((nd, next)));
                }
            }
        }
        None
    }

    /// Producer/consumer "ports" for one solid edge: `producer_starts` are
    /// belt tiles where a producer machine's output inserter drops `item`;
    /// `consumer_targets` are belt tiles where a consumer machine's input
    /// inserter picks `item` up; `di_distances` are Manhattan distances for
    /// any inserter found bridging a producer machine directly to a
    /// consumer machine (no belt in between at all).
    fn edge_ports(
        &self,
        layout: &LayoutResult,
        producer_recipes: &[String],
        consumer_recipe: &str,
        item: &str,
    ) -> (Vec<(i32, i32)>, FxHashSet<(i32, i32)>, Vec<f64>) {
        let producer_set: FxHashSet<&str> = producer_recipes.iter().map(String::as_str).collect();

        let mut producer_boxes: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut consumer_boxes: Vec<(i32, i32, i32, i32)> = Vec::new();
        for e in &layout.entities {
            let Some(recipe) = e.recipe.as_deref() else { continue };
            let (w, h) = crate::common::oriented_entity_dims(&e.name, e.direction);
            let bbox = (e.x, e.y, e.x + w, e.y + h);
            if producer_set.contains(recipe) {
                producer_boxes.push(bbox);
            }
            if recipe == consumer_recipe {
                consumer_boxes.push(bbox);
            }
        }
        let in_any_box = |t: (i32, i32), boxes: &[(i32, i32, i32, i32)]| {
            boxes.iter().any(|&(x0, y0, x1, y1)| t.0 >= x0 && t.0 < x1 && t.1 >= y0 && t.1 < y1)
        };

        let mut producer_starts = Vec::new();
        let mut consumer_targets: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut di_distances = Vec::new();

        for e in &layout.entities {
            if !is_inserter(&e.name) {
                continue;
            }
            // A multi-output producer / multi-input consumer pair can have
            // inserters moving a DIFFERENT item between the same two boxes;
            // counting those would inject stray samples into THIS edge's
            // measurement (round-3 bot review, minor 3). When the engine
            // stamped what the inserter carries, require it to match;
            // `carries: None` stays permissive (documented assumption:
            // un-stamped inserters between the pair are assumed to serve
            // the edge under measurement).
            if let Some(c) = &e.carries {
                if c != item {
                    continue;
                }
            }
            let (dx, dy) = dir_to_vec(e.direction);
            let reach = inserter_reach(&e.name);
            let pickup = (e.x - dx * reach, e.y - dy * reach);
            let drop = (e.x + dx * reach, e.y + dy * reach);

            let pickup_on_producer = in_any_box(pickup, &producer_boxes);
            let drop_on_consumer = in_any_box(drop, &consumer_boxes);
            let drop_is_item_belt = self.carries.get(&drop).map(String::as_str) == Some(item)
                && self.dir_map.contains_key(&drop)
                && !self.splitter_tiles.contains(&drop);
            let pickup_is_item_belt = self.carries.get(&pickup).map(String::as_str) == Some(item)
                && self.dir_map.contains_key(&pickup)
                && !self.splitter_tiles.contains(&pickup);

            if pickup_on_producer && drop_is_item_belt {
                producer_starts.push(drop);
            }
            if drop_on_consumer && pickup_is_item_belt {
                consumer_targets.insert(pickup);
            }
            if pickup_on_producer && drop_on_consumer {
                let dist = (drop.0 - pickup.0).unsigned_abs() as i64 + (drop.1 - pickup.1).unsigned_abs() as i64;
                di_distances.push(dist as f64);
            }
        }

        (producer_starts, consumer_targets, di_distances)
    }

    fn measure_edge(
        &self,
        layout: &LayoutResult,
        producer_recipes: &[String],
        consumer_recipe: &str,
        item: &str,
    ) -> Option<f64> {
        let (starts, targets, di_distances) = self.edge_ports(layout, producer_recipes, consumer_recipe, item);
        let mut samples = di_distances;
        for start in starts {
            if let Some(d) = self.shortest_to_any(start, &targets, item) {
                samples.push(d as f64);
            }
        }
        if samples.is_empty() {
            None
        } else {
            Some(samples.iter().sum::<f64>() / samples.len() as f64)
        }
    }
}

// ---------------------------------------------------------------------------
// Fluid (pipe) realized-path measurement
// ---------------------------------------------------------------------------

/// Fluid counterpart of [`SolidGraph::measure_edge`]. See module docs for
/// the pipe-to-ground gap this does NOT model.
fn measure_fluid_edge(
    layout: &LayoutResult,
    producer_recipes: &[String],
    consumer_recipe: &str,
    item: &str,
) -> Option<f64> {
    let producer_set: FxHashSet<&str> = producer_recipes.iter().map(String::as_str).collect();

    let mut pipe_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if (e.name == "pipe" || e.name == "pipe-to-ground") && e.carries.as_deref() == Some(item) {
            pipe_tiles.insert((e.x, e.y));
        }
    }

    let mut producer_ports: Vec<(i32, i32)> = Vec::new();
    let mut consumer_targets: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        let Some(recipe) = e.recipe.as_deref() else { continue };
        let is_producer = producer_set.contains(recipe);
        let is_consumer = recipe == consumer_recipe;
        if !is_producer && !is_consumer {
            continue;
        }
        for &(dx, dy, io) in fluid_ports::fluid_ports(&e.name, e.mirror, e.direction) {
            let port = (e.x + dx, e.y + dy);
            if !pipe_tiles.contains(&port) {
                continue;
            }
            if is_producer && io == "output" {
                producer_ports.push(port);
            }
            if is_consumer && io == "input" {
                consumer_targets.insert(port);
            }
        }
    }

    if producer_ports.is_empty() || consumer_targets.is_empty() {
        return None;
    }
    let mut samples = Vec::new();
    for p in producer_ports {
        if let Some(d) = fluid_bfs(&pipe_tiles, p, &consumer_targets) {
            samples.push(d as f64);
        }
    }
    if samples.is_empty() {
        None
    } else {
        Some(samples.iter().sum::<f64>() / samples.len() as f64)
    }
}

/// Unweighted BFS over 4-connected `pipe`/`pipe-to-ground` tiles carrying
/// one item. Does not model pipe-to-ground's underground jump — see module
/// docs "Known gaps".
fn fluid_bfs(pipe_tiles: &FxHashSet<(i32, i32)>, start: (i32, i32), targets: &FxHashSet<(i32, i32)>) -> Option<i64> {
    use std::collections::VecDeque;

    if targets.contains(&start) {
        return Some(0);
    }
    if !pipe_tiles.contains(&start) {
        return None;
    }
    let mut dist: FxHashMap<(i32, i32), i64> = FxHashMap::default();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    dist.insert(start, 0);
    queue.push_back(start);
    while let Some(t) = queue.pop_front() {
        let d = dist[&t];
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (t.0 + dx, t.1 + dy);
            if dist.contains_key(&n) || !pipe_tiles.contains(&n) {
                continue;
            }
            dist.insert(n, d + 1);
            if targets.contains(&n) {
                return Some(d + 1);
            }
            queue.push_back(n);
        }
    }
    None
}

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
            unattributed_edge_count: 0,
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
            candidate_attributed_edges: 1,
            candidate_total_edges: 1,
            native_attributed_edges: 1,
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
            path_length: Some(100.0),
        };
        let mut native = measure_with(1.5, 100.0, 500);
        native.edges = vec![edge.clone()];
        let mut ghost = measure_with(1.5, 0.0, 500);
        // Same edge, but nothing attributed on the candidate side.
        ghost.edges = vec![EdgeMeasurement { path_length: None, ..edge }];
        ghost.unattributed_edge_count = 1;
        let scores = score_vs_native(&ghost, &native);
        assert_eq!(scores.transit_score, None, "zero attribution is no evidence, not a win");
        assert_eq!(scores.candidate_attributed_edges, 0);
        assert_eq!(scores.common_attributed_edges, 0);
        assert_eq!(scores.composite, 0.0, "composite must treat missing transit evidence as neutral");
    }

    /// Round-3 bot review, finding A: transit must compare over the
    /// COMMONLY-attributed edge subset, not each side's own attributed sum.
    /// A candidate that attributes MORE edges than native (here: it routed
    /// on the surface an edge native left unmeasurable) must not be
    /// penalized for the extra measured term.
    #[test]
    fn partial_attribution_compares_common_subset_only() {
        let edge = |len: Option<f64>| EdgeMeasurement {
            producer_recipes: vec!["iron-gear-wheel".to_string()],
            item: "iron-gear-wheel".to_string(),
            consumer_recipe: "automation-science-pack".to_string(),
            rate: 1.0,
            is_fluid: false,
            path_length: len,
        };
        let mut native = measure_with(1.5, 100.0, 500);
        native.edges = vec![edge(Some(100.0)), edge(None)];
        native.unattributed_edge_count = 1;
        let mut cand = measure_with(1.5, 550.0, 500);
        cand.edges = vec![edge(Some(50.0)), edge(Some(500.0))];
        cand.unattributed_edge_count = 0;

        let scores = score_vs_native(&cand, &native);
        // Common subset = edge 0 only: 50 vs 100 → halved → score +0.5.
        // The old own-subset comparison computed 550 vs 100 → −4.5, ranking
        // the MORE-measurable candidate as a transit disaster.
        assert_eq!(scores.common_attributed_edges, 1);
        assert_eq!(scores.transit_score, Some(0.5));
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
            path_length: Some(100.0),
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
        assert_eq!(m.unattributed_edge_count, 0);
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

    /// Hand-built two-machine chain: a 1x1 stand-in producer at (0,0), a
    /// 5-tile belt run east, and a 1x1 stand-in consumer at (7,0) —
    /// producer -> output inserter -> belt(0,0..4,0) -> input inserter ->
    /// consumer. `entity_size` gives unknown names 1x1, so plain
    /// single-tile stand-ins keep the footprint math trivial and exact:
    /// the realized path is exactly 5 tiles (entry to exit inclusive, one
    /// tile per belt entity).
    #[test]
    fn measure_solid_edge_realized_path_length() {
        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "producer-stub".into(),
                    recipe: "produce-x".into(),
                    count: 1.0,
                    outputs: vec![ItemFlow { item: "x".into(), rate: 2.0, is_fluid: false, module_id: 0 }],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "consumer-stub".into(),
                    recipe: "consume-x".into(),
                    count: 1.0,
                    inputs: vec![ItemFlow { item: "x".into(), rate: 2.0, is_fluid: false, module_id: 0 }],
                    ..Default::default()
                },
            ],
            external_inputs: vec![],
            external_outputs: vec![ItemFlow { item: "y".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
            ..Default::default()
        };

        let mut entities = vec![
            machine(0, 0, "produce-x", "producer-stub"),
            inserter_at(1, 0, EntityDirection::East), // picks up (0,0), drops (2,0)
        ];
        for x in 2..7 {
            entities.push(belt(x, 0, EntityDirection::East, "x"));
        }
        entities.push(inserter_at(7, 0, EntityDirection::East)); // picks up (6,0), drops (8,0)
        entities.push(machine(8, 0, "consume-x", "consumer-stub"));

        let layout = LayoutResult { entities, width: 12, height: 3, ..Default::default() };
        let m = measure(&layout, &sr).expect("measure should succeed");

        assert_eq!(m.edges.len(), 1);
        let edge = &m.edges[0];
        assert_eq!(edge.item, "x");
        assert_eq!(edge.consumer_recipe, "consume-x");
        assert_eq!(edge.rate, 2.0);
        assert!(!edge.is_fluid);
        // Belt tiles (2,0)..=(6,0): drop lands on (2,0), pickup is (6,0) —
        // 5 belt tiles, Dijkstra distance 4 (one hop per tile).
        assert_eq!(edge.path_length, Some(4.0));
        assert_eq!(m.unattributed_edge_count, 0);
        assert_eq!(m.transit, 2.0 * 4.0);
    }

    /// Direct-insertion edge: producer and consumer bridged by one inserter
    /// with no belt at all. Path length must fall back to the Manhattan
    /// pickup-to-drop distance (module docs "Known gaps" / RFC-064 Phase 0
    /// decision log), not `None`.
    #[test]
    fn measure_direct_insertion_edge_uses_manhattan_fallback() {
        let sr = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "producer-stub".into(),
                    recipe: "produce-x".into(),
                    count: 1.0,
                    outputs: vec![ItemFlow { item: "x".into(), rate: 3.0, is_fluid: false, module_id: 0 }],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "consumer-stub".into(),
                    recipe: "consume-x".into(),
                    count: 1.0,
                    inputs: vec![ItemFlow { item: "x".into(), rate: 3.0, is_fluid: false, module_id: 0 }],
                    ..Default::default()
                },
            ],
            di_couplings: vec![DICoupling {
                producer_recipe: "produce-x".into(),
                consumer_recipe: "consume-x".into(),
                item: "x".into(),
                producer_count: 1.0,
                consumer_count: 1.0,
            }],
            external_outputs: vec![ItemFlow { item: "y".into(), rate: 1.0, is_fluid: false, module_id: 0 }],
            ..Default::default()
        };

        let entities = vec![
            machine(0, 0, "produce-x", "producer-stub"),
            inserter_at(1, 0, EntityDirection::East), // pickup (0,0) on producer, drop (2,0) on consumer
            machine(2, 0, "consume-x", "consumer-stub"),
        ];
        let layout = LayoutResult { entities, width: 5, height: 3, ..Default::default() };
        let m = measure(&layout, &sr).expect("measure should succeed");

        assert_eq!(m.edges.len(), 1);
        assert_eq!(m.edges[0].path_length, Some(2.0), "DI edge distance is Manhattan(pickup, drop) = |2-0| = 2");
        assert_eq!(m.unattributed_edge_count, 0);
    }

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
        assert_eq!(edge.path_length, Some(6.0));
        assert_eq!(m.unattributed_edge_count, 0);
        assert_eq!(m.transit, 4.0 * FLUID_WEIGHT * 6.0, "fluid edges must carry FLUID_WEIGHT, not the solid weight");
    }

    /// A fluid edge whose only route requires an underground pipe jump
    /// (module docs "Known gaps": pipe-to-ground pairing is not modeled)
    /// must report `path_length: None` and count toward
    /// `unattributed_edge_count` — never silently fall back to a proxy.
    #[test]
    fn measure_fluid_edge_unattributed_when_disconnected() {
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

        let m = measure(&layout, &sr).expect("measure should succeed");
        assert_eq!(m.edges.len(), 1);
        assert_eq!(m.edges[0].path_length, None);
        assert_eq!(m.unattributed_edge_count, 1);
        assert_eq!(m.transit, 0.0, "an unattributed edge must not silently contribute via a proxy");
    }
}
