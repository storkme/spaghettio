//! RFC-051 Phase B: the linear/fan-out chain auto-placer.
//!
//! Generalizes the Phase-1 hand composers: cells are engine-generated
//! per chain recipe, placed west→east in dependency order, wired with
//! template corridors only (straight, corner, UG-hop, 2→1 merge
//! splitter, 1→2 fan-out splitter). Chains are RATIO-QUANTIZED
//! (`required_copies`): K identical side-by-side copies each at 1/K of
//! the chain rate, so no corridor or feed column ever exceeds express
//! capacity — the Phase-1 K=3 pair topology generalized as a CAPACITY
//! mechanism (its quality claim was falsified — see `QUANTUM_RATE`).
//! K=1 chains compose bit-identically to the pre-quantization placer
//! (the registry hashes depend on it).
//! Fan-out past an intervening cell routes through a SOUTH
//! BYPASS lane below the cell band — the south side is empty except the
//! final drain, so the only crossings are known corridor rows, each
//! resolved by a local UG hop.
//!
//! Geometry-mode decision (Phase B, 2026-07-22): there is ONE geometry —
//! the calibrated (sim-kit-compatible) form, 4-tile feed pitch included.
//! The RFC's calibrated-twin invariant demands bit-identical
//! non-boundary entities between production and sim forms; a compacted
//! production form would shift cell origins and violate it. Bit-identity
//! outranks the area optimization (that overhead is ~24% and honest);
//! compacting under a translation-aware invariant is future work.

use rustc_hash::{FxHashMap, FxHashSet};
use crate::models::{BoundaryRecord, EntityDirection, LayoutResult, PlacedEntity, SolverResult};

use super::extract::{extract_cell, Cell, Port};
use super::compose::stamp_path;

/// Feed columns need >=4 tiles of lateral separation (#363: sim-kit
/// rigs collide at construction below that).
const FEED_PITCH: i32 = 4;
/// North margin: boundary row at y=0, then clearance for feed corners.
const CELL_Y: i32 = 3;
/// Horizontal clearance east of each cell for merges/fan-out/corridors.
const CORRIDOR_GAP: i32 = 6;
/// Vertical lanes reserved on the east side of each slot's feed block
/// for bypass descents/ascents.
const VLANES: i32 = 2;
/// Ratio-quantization quantum: the max rate any single belt carries —
/// produced items ride one express corridor per copy, external inputs
/// one express feed column per copy, both capped at express capacity.
/// This is a PHYSICAL cap, deliberately not a quality tuning knob: a
/// 15/s "measured-exact" quantum was tried and falsified — the Phase-1
/// K=3 pairs' exact measurement was an artifact of pre-#378 harness
/// tech state (researched inserter bonuses), and under declared
/// capacity small rows measure WORSE (−24% vs −8%) because the row
/// template's long-handed input inserters concentrate their deficit
/// (#383; the fix is RFC-049 Phase 3 inserter sizing, not geometry).
/// RFC-072 Phase 2 unit 1 (was 45.0): the quantum must satisfy TWO
/// caps — the express belt (45/s, universal) AND single-row-per-stage
/// composability, which is RECIPE-SPECIFIC (8 machines per row × that
/// stage's per-machine output): at 45 a copy's copper-cable stage
/// needs 9 machines against the 8-per-row cap, so
/// `CellComposedCandidate` refused every above-the-wall config with a
/// multi-row internal corridor and the engine had NO error-free path
/// past ~120/s. 40 is the value where every EC-FAMILY stage fits one
/// row (#730 round 5: for the ec corpus the two caps coincide at 40 —
/// AM3 cable at 5/s/machine — but that is a corpus fact, not a
/// universal invariant; a chain whose bottleneck stage could carry
/// more than 40/s in one row pays extra copies it does not need).
/// The derived form min(belt, max single-row stage rate) is the
/// recorded refinement that would make it per-recipe. At 40 the
/// composed strip ships and delivers: ec75 (K=6) sim 75.00/75.00 and
/// ec150 (K=12) sim 150.00/150.00, both +0.0% produced, PASS, above
/// the wall where native carries 37 lane-throughput errors (receipts
/// in the RFC's decision log).
const QUANTUM_RATE: f64 = 40.0;
/// Per-STRIP copy-count bound. Beyond this the single strip's aspect
/// ratio stops being honest scaling (ec150 at K=12 is already 2892×17)
/// and the chain composes as a GRID of stacked strips instead
/// (RFC-072 Phase 2 unit 2).
const K_MAX: i32 = 12;
/// Grid strip-count bound — the K_MAX successor's own honesty limit.
/// Beyond R_MAX × K_MAX copies, refuse loudly with the same wording
/// contract the old K_MAX refusal carried.
const R_MAX: i32 = 4;
/// Vertical kit band between stacked strips. Must hold the lower
/// strip's north feed rigs (per-copy clusters: depth ≤ 4+6·(c−1), plus
/// the ±2 bank overhang) and the upper strip's south drain rigs (base
/// ext 11 + bank flow margin). The band holds OPPOSITE-facing rigs
/// (the lower strip's north-extending feed rigs against the upper
/// strip's south-extending drain rigs), so the harness guards that
/// hold it are the cross-type feed-vs-drain guard and the rig-vs-LAYOUT
/// guard (scenario.rs `assert_feed_drain_rigs_disjoint`,
/// `assert_rigs_clear_of_layout`; #733 round 5 corrected the citation —
/// the near-parallel band guards compare same-direction bands only).
/// Both run in CI's default path against a committed real grid fixture,
/// so an undersized clearance fails LOUD there, never silently.
const STRIP_CLEARANCE: i32 = 32;

/// Smallest K such that every produced item and every external input
/// item runs at ≤ `QUANTUM_RATE` per copy. Chains under the cap stay
/// K=1 and compose bit-identically to the pre-quantization placer —
/// quantization only activates where the placer previously refused.
/// Evaluated at the default declared level; the grid path uses
/// [`required_copies_at`] with the level it composes at (#733 round 1).
pub fn required_copies(sr: &SolverResult) -> i32 {
    required_copies_at(sr, crate::common::DEFAULT_INSERTER_CAPACITY)
}

/// [`required_copies`] with the declared inserter-capacity level the
/// chain will compose at — the grid-territory margin terms ask the
/// inserter ladder at THAT level, so the estimator and the composer
/// can never disagree on what a hand moves. Sub-K_MAX results are
/// level-independent by construction (the margin terms are gated to
/// K > K_MAX), so every registered strip's K is unchanged.
pub fn required_copies_at(sr: &SolverResult, level: u8) -> i32 {
    let produced: FxHashSet<&str> = sr
        .machines
        .iter()
        .flat_map(|m| m.outputs.iter().map(|o| o.item.as_str()))
        .collect();
    // Items internal to a mega subgraph never ride chain belts either:
    // a solid intermediate produced AND consumed inside the block
    // (sulfur→sulfuric-acid in a chem-pack chain) is the block's own
    // business — counting it would force a spurious chain-wide K split
    // (#405 review finding 2). No-op for solid-only chains.
    let mega = super::mega::mega_subgraph(sr).ok().flatten();
    let mega_exports: FxHashSet<&str> = mega
        .as_ref()
        .map(|p| p.outputs.iter().map(|(i, _)| i.as_str()).collect())
        .unwrap_or_default();
    let is_member = |recipe: &str| mega.as_ref().is_some_and(|p| p.members.contains(recipe));
    let mut k = 1i32;
    for m in &sr.machines {
        for o in &m.outputs {
            // Fluids ride pipes, not belts — 2.0's segment model has no
            // per-pipe rate falloff, so they never drive the quantum
            // (RFC-052 Phase B; no-op for solid-only chains).
            if o.is_fluid {
                continue;
            }
            if is_member(&m.recipe) && !mega_exports.contains(o.item.as_str()) {
                continue;
            }
            let total = o.rate * m.count;
            k = k.max(((total / QUANTUM_RATE) - 1e-9).ceil() as i32);
        }
    }
    let mut ext: FxHashMap<&str, f64> = FxHashMap::default();
    for m in &sr.machines {
        for i in &m.inputs {
            if !i.is_fluid && !produced.contains(i.item.as_str()) {
                *ext.entry(i.item.as_str()).or_default() += i.rate * m.count;
            }
        }
    }
    for total in ext.values() {
        k = k.max(((total / QUANTUM_RATE) - 1e-9).ceil() as i32);
    }
    // Row-input MARGIN (RFC-072 P2 unit 2, the K72-3 "re-quantize before
    // killing" path, taken on the ec@240 receipt): the rate quantum
    // bounds PLANNED flow, but a copy's row input belt must also clear
    // the MACHINE-CAPACITY demand of the machines it feeds — a copy with
    // ceil(count/K) machines whose full-speed draw equals the belt cap
    // has zero margin, and the sim starves exactly one machine per copy
    // (ec@240 at K=18: 6 EC machines × 7.5/s = 45.00 on express; 18
    // ingredient-shortage machines, one per copy, produced −1.7%). The
    // validator's `row-input-belt-margin` warns on the same condition;
    // this keeps the quantizer from planning it. Bump K until every
    // solid input's per-copy capacity demand is strictly under express.
    // No-op for every registered strip (their K already clears it —
    // the copy-count pins and registry gates hold).
    let express = crate::common::belt_throughput("express-transport-belt");
    // Mega members are the block's business here too (#733 round 2 —
    // the rate loop and the hand term already skipped them; a member's
    // internal solid input rides the block's own belts, not a chain
    // corridor, so a belt margin on it is meaningless and could inflate
    // K past the grid bound for a mega-containing chain).
    let belt_violates = |k: i32| {
        sr.machines.iter().filter(|m| !is_member(&m.recipe)).any(|m| {
            let per_copy = (m.count / k as f64 - 1e-9).ceil();
            m.inputs
                .iter()
                .any(|i| !i.is_fluid && per_copy * i.rate >= express - 1e-9)
        })
    };
    // Input-HAND margin (RFC-072 P2 unit 2, the second re-quantization
    // the ec@240 receipts forced): the row's inserter ladder sizes each
    // machine side with `required <= n·rate` and NO margin, crediting a
    // long-handed hand exactly 2.4/s at the default level — so a copy
    // whose per-machine far-belt draw lands at 2.4/s gets ONE hand at
    // 100% of its credited rate, and the sim starves the row's tail
    // (ec@240 at K=20: 12/s per copy → 5 EC machines at 96% → iron
    // 2.40/s → one hand → two machines short per copy, −6.7%; K=18 →
    // 92.6% of one hand → one short per copy, −1.7%; the receipted
    // ec150 cell at 12.5/s → 2.5/s → TWO hands at 52% → plan). The fix
    // for the ladder itself is RFC-049 Phase 3 (a margin in sizing);
    // until it lands the quantizer refuses to PLAN a copy whose hands
    // the ladder would fill past HAND_MARGIN. Single source of truth:
    // this asks the ladder (`size_side`) and its calibrated rates
    // (`machine_feed_rate`) rather than re-deriving them, for EVERY solid
    // input — the far (long-handed) side and the near (regular/fast/
    // stack) side alike. A plan the ladder cannot cover (an honest
    // shortfall) counts as a violation here (#733 round 1); the
    // receipted low-level strips whose plans carry such shortfalls (the
    // d1 FAIL rows) never reach this term because it is grid-only, see
    // below. Evaluated at
    // the level the chain composes at (`level`), so a low-level grid
    // plans more copies rather than trusting default-level hands (#733
    // round 1); sub-K_MAX strips never enter this term, so their K stays
    // level-independent as before. Mega members are the block's
    // business, as in the rate loop above. SCOPED TO GRID
    // TERRITORY (K > K_MAX): applied to every chain it re-shaped the
    // registered military-science-pack@5 strip (the registry gate
    // caught it) — receipted sub-K_MAX strips keep their measured
    // geometry whatever their hand utilization (that is what a receipt
    // is for; RFC-049 P3 owns the general fix), while a grid's copies
    // carry no receipt and must be planned with margin.
    const HAND_MARGIN: f64 = 0.85;
    let quality = crate::common::QualityTier::Normal;
    let hand_violates = |k: i32| {
        use crate::bus::inserter_ladder::{size_side, InserterTier, Reach};
        sr.machines.iter().filter(|m| !is_member(&m.recipe)).any(|m| {
            let n = (m.count / k as f64 - 1e-9).ceil().max(1.0);
            let u = (m.count / k as f64 / n).min(1.0);
            let mut solids: Vec<f64> =
                m.inputs.iter().filter(|i| !i.is_fluid).map(|i| i.rate * u).collect();
            solids.sort_by(|a, b| a.partial_cmp(b).expect("finite rates"));
            // Reach model mirrors the row templates: with two or more
            // solid inputs the lowest-rate one rides the FAR belt
            // (`reassign_near_far` sends the hungrier item near) with the
            // contested extra column as its budget; the second is NEAR
            // with one extra column; a THIRD (and beyond) sits at reach-2
            // on a single long-handed hand with NO extra column
            // (`triple_input_row`'s input3 — #733 round 4: pricing it as
            // a near stack hand over-credited it ~5×).
            // The far side gets the contested column only when the row's
            // own contest grants it (#733 round 6): `dual_input_row` runs
            // `contest_favors_far(near, far, …)` for the shared dx=1
            // column, so pricing the far hand at budget 1 unconditionally
            // over-credited it by one inserter whenever near wins.
            // …and a row is only as fed as its LAST machine: at `LastInRow`
            // the far belt is trimmed and the contested column exists only
            // when `dual_input_row`'s one-tile extension rule fires (far
            // capped at the baseline hand, covered with one extra, near
            // needing no extra — templates.rs) — so the budget is priced
            // by that rule, not by an always-eligible column (codex review
            // of the close-out PR). For the receipted 12.5/s cell: iron 2.5
            // > 2.4 caps the baseline, two hands cover, cable's stack hand
            // needs nothing → budget 1, as the sim measured.
            let far_budget = if solids.len() >= 2 {
                let (far, near) = (solids[0], solids[1]);
                let contest = crate::bus::inserter_ladder::contest_favors_far(
                    near, far, true, quality, level,
                );
                let far_capped =
                    size_side(far, Reach::Far, 0, InserterTier::Stack, quality, level).shortfall.is_some();
                let far_covered_extended =
                    size_side(far, Reach::Far, 1, InserterTier::Stack, quality, level).shortfall.is_none();
                let near_needs_no_extra =
                    size_side(near, Reach::Near, 0, InserterTier::Stack, quality, level).shortfall.is_none();
                if contest && far_capped && far_covered_extended && near_needs_no_extra { 1 } else { 0 }
            } else {
                0
            };
            solids.iter().enumerate().any(|(idx, &rate)| {
                let (reach, budget) = match (solids.len(), idx) {
                    (n, 0) if n >= 2 => (Reach::Far, far_budget),
                    (_, 1) => (Reach::Near, 1),
                    (_, i) if i >= 2 => (Reach::Far, 0),
                    _ => (Reach::Near, 1),
                };
                let plan = size_side(rate, reach, budget, InserterTier::Stack, quality, level);
                let capacity = plan.capacity;
                // In grid territory a plan the ladder CANNOT cover (an
                // honest shortfall) is the strongest signal a copy would
                // starve, so it counts as a violation too (#733 round 1 —
                // the first cut excluded it, a holdover from when this
                // term also saw the receipted low-level strips).
                //
                // The FAR side's margin now lives in the ladder itself
                // (RFC-075: `size_side` credits a reach-2 pickup hand at
                // `FAR_PICKUP_FACTOR` of its flooded-belt rate, so
                // `capacity` already carries it) — applying HAND_MARGIN on
                // top would double-derate the hand this term was written
                // for. Near sides keep the margin here: their credits are
                // undented, and the grid's copies still carry no receipt.
                let margin = if reach == Reach::Far { 1.0 } else { HAND_MARGIN };
                plan.shortfall.is_some() || rate > margin * capacity + 1e-9
            })
        })
    };
    // BOTH margin terms are grid-territory only (#733 round 1: the belt
    // term was unscoped and re-quantized a K=1 multirow-corridor config
    // — the same re-shaping mechanism the hand term was scoped for).
    // The loop runs THROUGH the grid bound: a chain the margins cannot
    // satisfy within K_MAX × R_MAX copies leaves here at bound + 1 and
    // `chain_eligible` refuses it by name, instead of shipping a 4×12
    // grid with the violation still standing.
    while k > K_MAX && (belt_violates(k) || hand_violates(k)) && k <= K_MAX * R_MAX {
        k += 1;
    }
    k
}

/// Why a solve is not chain-composable. Stable strings — the candidate
/// reports these as its `accepted_reason`.
pub fn chain_eligible(sr: &SolverResult) -> Result<(), String> {
    chain_eligible_at(sr, crate::common::DEFAULT_INSERTER_CAPACITY)
}

/// [`chain_eligible`] at the declared level the chain will compose at —
/// the grid bound is level-dependent through the margin terms (#733
/// round 4: a chain grid-composable at L7 must not be refused by the
/// default-level count).
pub fn chain_eligible_at(sr: &SolverResult, level: u8) -> Result<(), String> {
    if sr.machines.is_empty() {
        return Err("cells: empty chain".into());
    }
    // RFC-052 Phase B: fluid-touching specs collapse into ONE mega-cell
    // (or refuse with the partition's named reason). Members are exempt
    // from the solid per-spec checks — the mega sub-solve owns them.
    let mega = super::mega::mega_subgraph(sr)?;
    let is_member = |recipe: &str| {
        mega.as_ref().is_some_and(|p| p.members.contains(recipe))
    };
    let mut producers: FxHashMap<&str, usize> = FxHashMap::default();
    for m in &sr.machines {
        if m.count <= 0.0 {
            return Err(format!("cells: zero-count spec {}", m.recipe));
        }
        if is_member(&m.recipe) {
            continue;
        }
        for o in &m.outputs {
            if o.is_fluid {
                return Err(format!("cells: fluid output {} outside the mega subgraph", o.item));
            }
            *producers.entry(o.item.as_str()).or_default() += 1;
        }
        for i in &m.inputs {
            if i.is_fluid {
                return Err(format!("cells: fluid input {} outside the mega subgraph", i.item));
            }
        }
        if !m.self_loop.is_empty() {
            return Err(format!("cells: self-loop recipe {}", m.recipe));
        }
    }
    if let Some(plan) = &mega {
        // Each mega output competes for corridor capacity like any
        // produced item. Multi-consumer exports fan out on the drain's
        // bypass row (Phase C) — same splitter-chain idiom as solid
        // cells, so no consumer-count refusal remains.
        for (item, _rate) in &plan.outputs {
            *producers.entry(item.as_str()).or_default() += 1;
        }
    }
    for (item, n) in &producers {
        if *n > 1 {
            return Err(format!("cells: {item} produced by {n} specs (need exactly 1)"));
        }
    }
    // Corridor capacity: ratio quantization bounds every copy's
    // corridors at QUANTUM_RATE, so high rates raise the copy count
    // instead of overloading a belt. Up to K_MAX copies compose as one
    // strip; beyond it the grid composer stacks up to R_MAX strips
    // (RFC-072 P2 unit 2). Refuse only past the grid bound.
    let k = required_copies_at(sr, level);
    if k > K_MAX * R_MAX {
        return Err(format!(
            "cells: chain needs {k} quantized copies (max {} = {R_MAX} strips x {K_MAX} at quantum {QUANTUM_RATE}/s)",
            K_MAX * R_MAX
        ));
    }
    Ok(())
}

struct Placed {
    cell: Cell,
    /// Absolute x of the cell's west edge.
    x: i32,
    /// Vertical placement offset: `CELL_Y` for ordinary cells, 0 for a
    /// mega block (it carries its own margin band with feed heads at
    /// its local y=0 = the chain boundary row).
    y_off: i32,
    /// The mega block's drain heads in ABSOLUTE coords (x, y, item) —
    /// each solid output exits SOUTH at its own head; corridors to the
    /// consumers start below them. Empty for ordinary cells.
    mega_drains: Vec<(i32, i32, String)>,
    /// Absolute x of the slot's west edge (feed block start).
    slot_x: i32,
    /// Base x of this slot's vertical-lane strip (east of feed columns).
    vlane_base: i32,
    recipe: String,
    /// Segment-id name: the recipe, suffixed `#<copy>` when K>1 so the
    /// belt validators never see two disjoint runs sharing a segment.
    /// Equals `recipe` at K=1 (bit-identity).
    seg: String,
    /// Which quantized chain copy this slot belongs to. Corridors only
    /// connect producer→consumer within one copy.
    copy: i32,
    ext_inputs: Vec<String>,
}

fn port_abs(p: &Port, cell_x: i32, cell_y: i32) -> (i32, i32) {
    (cell_x + p.x, cell_y + p.y)
}

/// The consumer-side port an INTERNAL (producer-cell → consumer-cell)
/// corridor wires to for `item`. extract.rs derives one inbound port
/// per 4-adjacency-CONNECTED RUN of the item's `belt-in` segment —
/// usually that's one run (one port) per row, but UG gaps punched into
/// a SINGLE row's belt-in segment (e.g. `quad_input_row`'s belt3:
/// per-machine UG-IN at `mx` / gap / UG-OUT at `mx+2`, still one
/// logical segment) break 4-adjacency and split it into several
/// same-row runs, each getting its own port. Pre-#433-fix, `.find`
/// picked whichever port came first in extraction order — which for
/// these same-row splits is the westmost run, i.e. the belt's true
/// upstream entry — and wired correctly (measured: 7 chain-eligible
/// targets at warnings=0, review round on this PR). A genuinely
/// multi-ROW consumer cell (distinct y per row) exposes one port per
/// row for the same item; v1 corridors route to exactly one port, so
/// wiring "the" port would silently starve every row past the first —
/// THAT's the real #433 defect.
///
/// So: group matches by `y`. All matches share one y → same-row
/// UG-split runs (or the trivial single-port case) — return the min-x
/// (westmost = upstream entry) port, exactly reproducing the pre-fix
/// deterministic behavior. Matches spanning more than one distinct y →
/// genuine multi-row fan-in, which v1 corridors can't wire — refuse
/// loudly instead of silently starving rows past the first (the
/// EXTERNAL world-feed path was already fixed port-aware — commit
/// 620d103c, "every port gets its own feed column"). Wiring every
/// row's port is future work, gated on feed-bound sizing actually
/// reaching multi-row chain cells.
fn single_inbound_port<'a>(
    ports: &'a [Port],
    item: &str,
    consumer_recipe: &str,
) -> Result<&'a Port, String> {
    let matches: Vec<&Port> = ports.iter().filter(|q| q.inbound && q.item == item).collect();
    if matches.is_empty() {
        return Err(format!("cells: {consumer_recipe} lacks in-port for {item}"));
    }
    let mut rows: Vec<i32> = matches.iter().map(|p| p.y).collect();
    rows.sort_unstable();
    rows.dedup();
    if rows.len() == 1 {
        return Ok(matches
            .into_iter()
            .min_by_key(|p| p.x)
            .expect("matches non-empty, checked above"));
    }
    Err(format!(
        "cells: {consumer_recipe} has {} in-ports for {item} across {} rows (multi-row corridor fan not implemented, #433)",
        matches.len(),
        rows.len()
    ))
}

/// Crossing-aware corridor stamper. Horizontal runs hop under
/// registered vertical columns; vertical legs hop under registered
/// horizontal rows; whichever is stamped LATER does the hopping, so
/// stamp order between two crossing corridors doesn't matter. All hops
/// are span-3 UG pairs (within every belt tier's reach) — template
/// machinery only (kill 5).
struct Router {
    h_rows: Vec<(i32, i32, i32)>, // (y, x0, x1) inclusive
    v_cols: Vec<(i32, i32, i32)>, // (x, y0, y1) inclusive
    /// Tiles stamped so far (cells + feeds seeded, then every Router
    /// push). Crossing hops only cover STRICT INTERIORS of registered
    /// runs — corners, terminals, and hop mouths sit on boundary tiles,
    /// which is exactly the mil5-ore overlap class. Collision checks
    /// against this set trigger the local fallbacks; collision-free
    /// layouts take the legacy paths bit-identically.
    occ: FxHashSet<(i32, i32)>,
}

impl Router {
    fn new() -> Self {
        Router { h_rows: Vec::new(), v_cols: Vec::new(), occ: FxHashSet::default() }
    }

    /// Seed occupancy from already-stamped entities (cells, feeds,
    /// merge/fan splitters). Splitters cover two tiles (their second
    /// half extends one tile perpendicular to facing — south half for
    /// east-facing).
    fn seed(&mut self, entities: &[PlacedEntity]) {
        for e in entities {
            self.occ.insert((e.x, e.y));
            if e.name.ends_with("splitter") {
                self.occ.insert((e.x, e.y + 1));
            }
        }
    }

    /// Can an eastward `hrow` legally stamp y over [x0, x1]? Occupied
    /// tiles are fine when they belong to a registered CROSSING column
    /// strictly inside the span — the hrow hops under those. Anything
    /// else occupied (parallel same-row runs, corners, boundary-tile
    /// columns the strict hop filter skips) means no.
    fn is_row_stampable(&self, y: i32, x0: i32, x1: i32) -> bool {
        let (lo, hi) = (x0.min(x1), x0.max(x1));
        (lo..=hi).all(|x| {
            !self.occ.contains(&(x, y))
                || (x > lo
                    && x < hi
                    && self
                        .v_cols
                        .iter()
                        .any(|(cx, cy0, cy1)| *cx == x && y >= *cy0 && y <= *cy1))
        })
    }

    /// Register an externally stamped column (feed columns).
    fn register_col(&mut self, x: i32, y0: i32, y1: i32) {
        self.v_cols.push((x, y0.min(y1), y0.max(y1)));
    }
    fn register_row(&mut self, y: i32, x0: i32, x1: i32) {
        self.h_rows.push((y, x0.min(x1), x0.max(x1)));
    }

    /// Eastward row from x0..=x1 at y, hopping under crossing columns.
    #[allow(clippy::too_many_arguments)]
    fn hrow(
        &mut self,
        out: &mut Vec<PlacedEntity>,
        y: i32,
        x0: i32,
        x1: i32,
        item: &str,
        belt: &str,
        ug: &str,
        seg: &str,
    ) {
        let mut cols: Vec<i32> = self
            .v_cols
            .iter()
            .filter(|(cx, cy0, cy1)| *cx > x0 && *cx < x1 && y >= *cy0 && y <= *cy1)
            .map(|(cx, _, _)| *cx)
            .collect();
        cols.sort_unstable();
        cols.dedup();
        let occ = &mut self.occ;
        let push_east = |out: &mut Vec<PlacedEntity>, occ: &mut FxHashSet<(i32, i32)>, xa: i32, xb: i32| {
            for x in xa..=xb {
                occ.insert((x, y));
                out.push(PlacedEntity {
                    name: belt.into(), x, y,
                    direction: EntityDirection::East,
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
        };
        // Cluster columns closer than 3 tiles: independent per-column
        // hops would share tiles (exit of one = entry of the next).
        let mut clusters: Vec<(i32, i32)> = Vec::new();
        for c in cols {
            match clusters.last_mut() {
                Some((_, hi)) if c - *hi < 3 => *hi = c,
                _ => clusters.push((c, c)),
            }
        }
        let mut x = x0;
        for (lo2, hi2) in clusters {
            assert!(hi2 - lo2 + 2 <= 9, "cells: hop cluster span exceeds express reach");
            if lo2 - 2 >= x {
                push_east(out, occ, x, lo2 - 2);
            }
            for (hx, io) in [(lo2 - 1, "input"), (hi2 + 1, "output")] {
                occ.insert((hx, y));
                out.push(PlacedEntity {
                    name: ug.into(),
                    x: hx,
                    y,
                    direction: EntityDirection::East,
                    io_type: Some(io.into()),
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
            x = hi2 + 2;
        }
        if x <= x1 {
            push_east(out, occ, x, x1);
        }
        self.register_row(y, x0, x1);
    }

    /// WESTWARD row from x0 down to x1 (x0 > x1) at y, hopping under
    /// crossing columns — the mirror of `hrow`, for bypass edges whose
    /// consumer sits WEST of the producer (the reversed-dependency
    /// placement order does not guarantee eastward flow for items
    /// consumed at several depths). Kept separate from `hrow` so the
    /// eastward path stays bit-identical.
    #[allow(clippy::too_many_arguments)]
    fn hrow_west(
        &mut self,
        out: &mut Vec<PlacedEntity>,
        y: i32,
        x0: i32,
        x1: i32,
        item: &str,
        belt: &str,
        ug: &str,
        seg: &str,
    ) {
        let mut cols: Vec<i32> = self
            .v_cols
            .iter()
            .filter(|(cx, cy0, cy1)| *cx < x0 && *cx > x1 && y >= *cy0 && y <= *cy1)
            .map(|(cx, _, _)| *cx)
            .collect();
        cols.sort_unstable_by(|a, b| b.cmp(a));
        cols.dedup();
        let occ = &mut self.occ;
        let push_west = |out: &mut Vec<PlacedEntity>, occ: &mut FxHashSet<(i32, i32)>, xa: i32, xb: i32| {
            for x in (xb..=xa).rev() {
                occ.insert((x, y));
                out.push(PlacedEntity {
                    name: belt.into(), x, y,
                    direction: EntityDirection::West,
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
        };
        let mut clusters: Vec<(i32, i32)> = Vec::new();
        for c in cols {
            match clusters.last_mut() {
                Some((_, lo)) if *lo - c < 3 => *lo = c,
                _ => clusters.push((c, c)),
            }
        }
        let mut x = x0;
        for (hi2, lo2) in clusters {
            assert!(hi2 - lo2 + 2 <= 9, "cells: hop cluster span exceeds express reach");
            if hi2 + 2 <= x {
                push_west(out, occ, x, hi2 + 2);
            }
            for (hx, io) in [(hi2 + 1, "input"), (lo2 - 1, "output")] {
                occ.insert((hx, y));
                out.push(PlacedEntity {
                    name: ug.into(),
                    x: hx,
                    y,
                    direction: EntityDirection::West,
                    io_type: Some(io.into()),
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
            x = lo2 - 2;
        }
        if x >= x1 {
            push_west(out, occ, x, x1);
        }
        self.register_row(y, x1, x0);
    }

    /// West-facing corner belt at (x, y): single perpendicular input
    /// (a southbound descent turning west onto a bypass row).
    fn corner_west(&mut self, out: &mut Vec<PlacedEntity>, x: i32, y: i32, item: &str, belt: &str, seg: &str) {
        self.occ.insert((x, y));
        out.push(PlacedEntity {
            name: belt.into(),
            x,
            y,
            direction: EntityDirection::West,
            carries: Some(item.into()),
            segment_id: Some(seg.into()),
            ..Default::default()
        });
        self.register_row(y, x, x);
    }

    /// Vertical leg from y0 toward y1 at x (either direction), hopping
    /// under crossing rows.
    #[allow(clippy::too_many_arguments)]
    fn vcol(
        &mut self,
        out: &mut Vec<PlacedEntity>,
        x: i32,
        y0: i32,
        y1: i32,
        item: &str,
        belt: &str,
        ug: &str,
        seg: &str,
    ) {
        let (lo, hi) = (y0.min(y1), y0.max(y1));
        let down = y1 > y0;
        let (dir, io_near, io_far) = if down {
            (EntityDirection::South, "input", "output")
        } else {
            (EntityDirection::North, "input", "output")
        };
        let mut rows: Vec<i32> = self
            .h_rows
            .iter()
            .filter(|(ry, rx0, rx1)| *ry > lo && *ry < hi && x >= *rx0 && x <= *rx1)
            .map(|(ry, _, _)| *ry)
            .collect();
        rows.sort_unstable();
        if !down {
            rows.reverse();
        }
        let step = if down { 1 } else { -1 };
        let occ = &mut self.occ;
        let push_v = |out: &mut Vec<PlacedEntity>, occ: &mut FxHashSet<(i32, i32)>, ya: i32, yb: i32| {
            let (lo2, hi2) = (ya.min(yb), ya.max(yb));
            for y in lo2..=hi2 {
                occ.insert((x, y));
                out.push(PlacedEntity {
                    name: belt.into(), x, y,
                    direction: dir,
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
        };
        let mut clusters: Vec<(i32, i32)> = Vec::new();
        for r in rows {
            match clusters.last_mut() {
                Some((_, last)) if (r - *last) * step < 3 => *last = r,
                _ => clusters.push((r, r)),
            }
        }
        let mut y = y0;
        for (first, last) in clusters {
            assert!((last - first).abs() + 2 <= 9, "cells: hop cluster span exceeds express reach");
            if (first - 2 * step - y) * step >= 0 {
                push_v(out, occ, y, first - 2 * step);
            }
            for (hy, io) in [(first - step, io_near), (last + step, io_far)] {
                occ.insert((x, hy));
                out.push(PlacedEntity {
                    name: ug.into(),
                    x,
                    y: hy,
                    direction: dir,
                    io_type: Some(io.into()),
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
            y = last + 2 * step;
        }
        if (y1 - y) * step >= 0 {
            push_v(out, occ, y, y1);
        }
        self.register_col(x, lo, hi);
    }

    /// North-facing corner belt at (x, y): single perpendicular input.
    fn corner_north(&mut self, out: &mut Vec<PlacedEntity>, x: i32, y: i32, item: &str, belt: &str, seg: &str) {
        self.occ.insert((x, y));
        out.push(PlacedEntity {
            name: belt.into(),
            x,
            y,
            direction: EntityDirection::North,
            carries: Some(item.into()),
            segment_id: Some(seg.into()),
            ..Default::default()
        });
        self.register_col(x, y, y);
    }

    /// South-facing corner belt at (x, y): single perpendicular input
    /// (the in-gap descent entry — flow arrives eastbound from the
    /// splitter's south output and turns down).
    fn corner_south(&mut self, out: &mut Vec<PlacedEntity>, x: i32, y: i32, item: &str, belt: &str, seg: &str) {
        self.occ.insert((x, y));
        out.push(PlacedEntity {
            name: belt.into(),
            x,
            y,
            direction: EntityDirection::South,
            carries: Some(item.into()),
            segment_id: Some(seg.into()),
            ..Default::default()
        });
        self.register_col(x, y, y);
    }

    /// East-facing corner belt at (x, y): single perpendicular input =
    /// lane-preserving corner (the post-review splitter-merge idiom).
    fn corner_east(&mut self, out: &mut Vec<PlacedEntity>, x: i32, y: i32, item: &str, belt: &str, seg: &str) {
        self.occ.insert((x, y));
        out.push(PlacedEntity {
            name: belt.into(),
            x,
            y,
            direction: EntityDirection::East,
            carries: Some(item.into()),
            segment_id: Some(seg.into()),
            ..Default::default()
        });
        self.register_row(y, x, x);
    }
}

/// Free the tile at (x, y) by putting the FEED ROW that crosses it
/// underground: replace `feed → feed → feed` with `UG-in → (clear) →
/// UG-out` centered on the collision tile. Only fires when an ascent's
/// terminal (a boundary tile the crossing hops can't cover) lands on a
/// feed row. Refuses loudly on any shape it doesn't recognize — a
/// refusal is honest, a silent overlap is not.
fn retrofit_feed_hop(
    entities: &mut Vec<PlacedEntity>,
    router: &mut Router,
    x: i32,
    y: i32,
) -> Result<(), String> {
    // Any horizontal surface-belt run may host the hop: feed rows
    // (eastbound) and corridor/fan rows (either direction — Phase C:
    // USP's ascent terminal landed on the mega's own fan branch). The
    // three tiles must be same-direction plain belts of one row
    // segment class; anything else refuses loudly.
    let row_dir = entities
        .iter()
        .find(|e| e.x == x && e.y == y)
        .map(|e| e.direction)
        .unwrap_or(EntityDirection::East);
    if row_dir != EntityDirection::East && row_dir != EntityDirection::West {
        return Err(format!(
            "cells: ascent terminal collision at ({x},{y}) is not a horizontal belt run"
        ));
    }
    let mut idxs = [usize::MAX; 3];
    for (k, cx) in [x - 1, x, x + 1].iter().enumerate() {
        let i = entities
            .iter()
            .position(|e| {
                e.x == *cx
                    && e.y == y
                    && e.direction == row_dir
                    && e.name.ends_with("transport-belt")
                    && e.segment_id.as_deref().is_some_and(|s| {
                        s.starts_with("feed:") || s.starts_with("corr:") || s.starts_with("fan")
                    })
            })
            .ok_or_else(|| {
                let blocker = entities
                    .iter()
                    .find(|e| e.x == *cx && e.y == y)
                    .map(|e| {
                        format!(
                            "{} dir={:?} seg={}",
                            e.name,
                            e.direction,
                            e.segment_id.as_deref().unwrap_or("-")
                        )
                    })
                    .unwrap_or_else(|| "empty".into());
                format!(
                    "cells: ascent terminal collision at ({x},{y}) is not a hoppable belt row (tile x={cx}: {blocker})"
                )
            })?;
        idxs[k] = i;
    }
    let ug = match entities[idxs[1]].name.as_str() {
        "express-transport-belt" => "express-underground-belt",
        "fast-transport-belt" => "fast-underground-belt",
        "transport-belt" => "underground-belt",
        other => return Err(format!("cells: unexpected feed belt tier {other}")),
    };
    // Entrance = upstream side of the run's flow direction.
    let (in_k, out_k) = if row_dir == EntityDirection::East {
        (0usize, 2usize)
    } else {
        (2usize, 0usize)
    };
    for (k, io) in [(in_k, "input"), (out_k, "output")] {
        entities[k_idx(&idxs, k)].name = ug.into();
        entities[k_idx(&idxs, k)].io_type = Some(io.into());
    }
    entities.remove(idxs[1]);
    router.occ.remove(&(x, y));
    Ok(())
}

fn k_idx(idxs: &[usize; 3], k: usize) -> usize {
    idxs[k]
}

/// Compose an eligible chain solve into one layout: K quantized copies
/// (`required_copies`) of the chain placed side by side west→east, each
/// copy one cell per recipe at 1/K of the chain rate with its own
/// feeds, corridors, and drain — the Phase-1 pair-topology shape.
/// Returns the composed layout with boundary records populated
/// (calibrated orientation: north feeds, south drains, west→east
/// record order — #363).
pub fn compose_chain(sr: &SolverResult) -> Result<LayoutResult, String> {
    compose_chain_with_capacity(sr, crate::common::DEFAULT_INSERTER_CAPACITY)
}

/// `compose_chain` with the declared inserter-capacity level threaded to
/// every member cell's generation (#415). Capacity 0 is byte-identical
/// to the pre-RFC-049 chain output by construction; the no-argument
/// `compose_chain` now defaults to `common::DEFAULT_INSERTER_CAPACITY`
/// (L2), so pass 0 explicitly for the raw unresearched world.
///
/// RFC-072 Phase 2 unit 2: chains up to K_MAX copies compose as ONE
/// strip, bit-identical to the pre-grid composer; beyond it,
/// `compose_grid_with_capacity` stacks balanced strips.
pub fn compose_chain_with_capacity(
    sr: &SolverResult,
    inserter_capacity: u8,
) -> Result<LayoutResult, String> {
    let k = required_copies_at(sr, inserter_capacity);
    if k > K_MAX {
        return compose_grid_with_capacity(sr, inserter_capacity, k);
    }
    compose_strip_with_capacity(sr, inserter_capacity, k)
}

/// One horizontal strip of `copies` quantized copies — the entire
/// pre-unit-2 composer, unchanged except that the copy count is now the
/// caller's: the single-strip path passes the quantizer's own value,
/// the grid path passes each strip's planned share of a proportionally
/// scaled `SolverResult` (a strip re-deriving its count from the scaled
/// rate would disagree with the grid-only margin terms in
/// `required_copies`, which is what the planned count exists to avoid).
fn compose_strip_with_capacity(
    sr: &SolverResult,
    inserter_capacity: u8,
    copies: i32,
) -> Result<LayoutResult, String> {
    // (RFC-055's ChainOrder::Compact axis and its compose_chain_compact
    // entry were deleted 2026-08-20 with cells/placement.rs — owner call
    // extending #632 A2; record in RFC-055's decision log.)
    chain_eligible_at(sr, inserter_capacity)?; // #733 round 5: at the composing level, like the grid path
    let kq = copies.max(1);
    let scale = 1.0 / kq as f64;
    // RFC-052 Phase B: fluid specs collapse into one SUPER-SPEC whose
    // placed form is the boundary-adapted mega block. Solid-only chains
    // take mega_plan = None and every branch below is bit-identical to
    // the pre-Phase-B placer (the registry gate enforces it).
    let mega_plan = super::mega::mega_subgraph(sr)?;
    // Mega-containing chains honor their DECLARED capacity. #415 is closed
    // (COMPLETED by #422): the non-mega cells thread it via
    // `generate_cell_layout_with_capacity` below, and the mega INTERIOR
    // bootstrap (`compose_mega_block`) is inherently L0 — it takes no
    // capacity argument and pins `inserter_capacity: 0` internally. So a
    // mega chain at L>0 sizes its solid cells at the declared level while
    // the interior stays conservatively L0 (over-provisioned = the safe
    // direction, real ≥ plan), and the layout declares the real level.
    // History: a hard `Err` refusal (pre-#383, #422 stop-gap) → a
    // whole-chain L0 clamp (#383 initial) → dropped here once #422 landed
    // (2026-07-24, PR #431 review coordination). The interior's L0 pin is
    // honest until it too threads capacity; drop the `mega.rs` pins then.
    const MEGA_PREFIX: &str = "mega:";

    let produced: FxHashSet<&str> = sr
        .machines
        .iter()
        .flat_map(|m| m.outputs.iter().map(|o| o.item.as_str()))
        .collect();

    // Place producers-first, west→east. `sr.dependency_order` is
    // TARGET-FIRST (the solver's DFS pushes a recipe before recursing
    // into its ingredients), so reverse it; unlisted recipes go last.
    let mut specs: Vec<crate::models::MachineSpec> = match &mega_plan {
        None => sr.machines.to_vec(),
        Some(plan) => {
            let mut v: Vec<crate::models::MachineSpec> = sr
                .machines
                .iter()
                .filter(|m| !plan.members.contains(&m.recipe))
                .cloned()
                .collect();
            // Synthetic super-spec: one "machine" producing the
            // subgraph's terminal solid at the full chain rate.
            // Chain-fed inputs (increment 2) are DECLARED so the
            // generic consumer/corridor machinery routes their
            // producers here; true externals stay off the spec (they
            // arrive via the block's own boundary heads).
            v.push(crate::models::MachineSpec {
                entity: "mega-cell".into(),
                recipe: format!("{MEGA_PREFIX}{}", plan.target),
                count: 1.0,
                inputs: plan
                    .chain_fed
                    .iter()
                    .map(|(item, rate)| crate::models::ItemFlow {
                        item: item.clone(),
                        rate: *rate,
                        is_fluid: false,
                        module_id: 0,
                    })
                    .collect(),
                outputs: plan
                    .outputs
                    .iter()
                    .map(|(item, rate)| crate::models::ItemFlow {
                        item: item.clone(),
                        rate: *rate,
                        is_fluid: false,
                        module_id: 0,
                    })
                    .collect(),
                ..sr.machines[0].clone()
            });
            v
        }
    };
    let mut pos: FxHashMap<String, usize> = sr
        .dependency_order
        .iter()
        .enumerate()
        .map(|(i, r)| (r.clone(), i))
        .collect();
    if let Some(plan) = &mega_plan {
        // Source-only subgraph (no chain-fed inputs): inherit the
        // DEEPEST member's dependency position, so it places before
        // its consumer like the member rows would have. Chain-fed
        // subgraph (increment 2): inherit the MOST-DOWNSTREAM
        // member's position — the member that consumes the chain-fed
        // items governs, so the mega places AFTER its producers (for
        // the PU class that member is the chain target itself).
        let picked = if plan.chain_fed.is_empty() {
            plan.members.iter().filter_map(|r| pos.get(r).copied()).max()
        } else {
            plan.members.iter().filter_map(|r| pos.get(r).copied()).min()
        };
        if let Some(i) = picked {
            pos.insert(format!("{MEGA_PREFIX}{}", plan.target), i);
        }
    }
    specs.sort_by_key(|m| match pos.get(m.recipe.as_str()) {
        Some(&i) => (0, std::cmp::Reverse(i)),
        None => (1, std::cmp::Reverse(usize::MAX)),
    });

    // Per-slot vertical-lane demand, from the bypass edge list (an edge
    // p→c descends in slot p+1 and ascends in slot c; sizing by the
    // slot's own fan-out under-counted ascents — the mil5-ore overlap
    // class).
    let n = specs.len();
    let mut lane_demand: Vec<i32> = vec![0; n];
    for (pi, m) in specs.iter().enumerate() {
        let pi_mega = m.recipe.starts_with(MEGA_PREFIX);
        for o in &m.outputs {
            for (ci, c) in specs.iter().enumerate() {
                if pi_mega && ci != pi && c.inputs.iter().any(|i| i.item == o.item) {
                    // Mega corridors ride the bypass row (the drain
                    // exits south) — ascent lane always; fanned
                    // outputs (>=2 consumers) ALSO drop each branch
                    // through a lane in the next slot's strip, like
                    // solid bypass descents (ad-hoc splitter-column
                    // drops collided with consumer ascent lanes —
                    // USP@2 head-on/loop cluster).
                    lane_demand[ci] += 1;
                    let consumers_of_o = specs
                        .iter()
                        .enumerate()
                        .filter(|(cj, cc)| {
                            *cj != pi && cc.inputs.iter().any(|i| i.item == o.item)
                        })
                        .count();
                    if consumers_of_o >= 2 && pi + 1 < n {
                        lane_demand[pi + 1] += 1;
                    }
                    continue;
                }
                let consumes = ci != pi && c.inputs.iter().any(|i| i.item == o.item);
                let needs_bypass = ci != pi + 1;
                if consumes && needs_bypass {
                    if pi + 1 < n {
                        lane_demand[pi + 1] += 1;
                    }
                    lane_demand[ci] += 1;
                }
            }
        }
    }

    let mut entities: Vec<PlacedEntity> = Vec::new();
    // Producer warnings the composed result carries (#715 review round 3:
    // a ceiling recorded only in a comment is invisible to selection and
    // to the web app; a warning is countable by the layout-warnings floor
    // and readable by a user).
    let mut chain_warnings: Vec<String> = Vec::new();
    let mut b_in: Vec<BoundaryRecord> = Vec::new();
    let mut b_out: Vec<BoundaryRecord> = Vec::new();
    let mut surplus_exits: Vec<(String, i32, i32)> = Vec::new();
    let mut placed: Vec<Placed> = Vec::new();
    let mut cursor = 0i32;

    // Copies are identical — generate each spec's cell once (the cell
    // generator runs the full engine; K=12 must not run it 12×).
    let mut cell_cache: Vec<Cell> = Vec::with_capacity(n);
    let mut mega_block_cache: Option<LayoutResult> = None;
    for copy in 0..kq {
        for (si, m) in specs.iter().enumerate() {
            let out_item = m
                .outputs
                .first()
                .ok_or_else(|| format!("cells: {} has no output", m.recipe))?
                .item
                .clone();
            let is_mega = m.recipe.starts_with(MEGA_PREFIX);
            // outputs[].rate is PER-MACHINE; the cell serves the whole
            // spec's share of this copy.
            let rate = m.outputs[0].rate * m.count * scale;
            if copy == 0 {
                if is_mega {
                    let plan = mega_plan.as_ref().expect("mega spec implies plan");
                    let (_msr, block) = super::mega::compose_mega_block(sr, plan, scale)?;
                    mega_block_cache = Some(block.clone());
                    // Chain-fed inputs (increment 2): each adapter
                    // west-edge entry becomes an inbound PORT — an
                    // east-facing belt at (0, lane_row), so the
                    // corridor's eastbound approach is a straight
                    // merge (both lanes). v1: one entry per chain-fed
                    // item (multi-entry re-pitches would need a fan-in
                    // the corridor can't provide).
                    let mut ports: Vec<Port> = Vec::new();
                    for (item, _) in &plan.chain_fed {
                        let heads: Vec<_> = block
                            .boundary_inputs
                            .iter()
                            .filter(|r| &r.item == item)
                            .collect();
                        match heads.as_slice() {
                            [h] => ports.push(Port {
                                edge: "W",
                                x: h.x,
                                y: h.y,
                                item: item.clone(),
                                inbound: true,
                            }),
                            [] => {
                                return Err(format!(
                                    "mega: chain-fed {item} has no block feed head"
                                ))
                            }
                            _ => {
                                return Err(format!(
                                    "mega: chain-fed {item} has {} feed heads (v1 routes one corridor)",
                                    heads.len()
                                ))
                            }
                        }
                    }
                    cell_cache.push(Cell {
                        width: block.width,
                        height: block.height,
                        ports,
                        entities: block.entities,
                    });
                } else {
                    let input_names: Vec<&str> = m.inputs.iter().map(|i| i.item.as_str()).collect();
                    let (_csr, cl) = super::extract::generate_cell_layout_with_capacity(&out_item, rate, &input_names, inserter_capacity);
                    cell_cache.push(extract_cell(&cl));
                }
            }
            let cell = cell_cache[si].clone();
            let ext_inputs: Vec<String> = m
                .inputs
                .iter()
                .filter(|i| !produced.contains(i.item.as_str()))
                .map(|i| i.item.clone())
                .collect();
            let n_feed_ports = cell
                .ports
                .iter()
                .filter(|q| q.inbound && ext_inputs.contains(&q.item))
                .count() as i32;

            let n_out_runs = cell.ports.iter().filter(|q| !q.inbound).count() as i32;
            let n_consumers = sr
                .machines
                .iter()
                .filter(|c| c.inputs.iter().any(|i| i.item == out_item))
                .count() as i32;
            // Gap: base + 2 per extra merge stage + 2 per extra fan-out stage.
            let gap = CORRIDOR_GAP + 2 * (n_out_runs - 1).max(0) + 2 * (n_consumers - 1).max(0);
            let slot_x = cursor;
            let feed_w = FEED_PITCH * n_feed_ports + 1;
            let vlane0 = slot_x + feed_w;
            // Multi-edge strips use 2-tile lane pitch: at pitch 1 a
            // lane's corner + hrow-start tile sits ON the neighbor
            // lane's column (a mil5-ore overlap class). Single-edge
            // strips keep pitch 1 — bit-identical to before.
            let strip = VLANES + lane_demand[si] * if lane_demand[si] >= 2 { 2 } else { 1 };
            let x = vlane0 + strip + 1;
            cursor = x + cell.width + gap;

            let y_off = if is_mega { 0 } else { CELL_Y };
            for e in &cell.entities {
                let mut e = e.clone();
                e.x += x;
                e.y += y_off;
                entities.push(e);
            }
            let mega_drains = if is_mega {
                let block = mega_block_cache.as_ref().expect("block cached");
                surplus_exits.extend(
                    block
                        .surplus_exits
                        .iter()
                        .map(|(item, sx, sy)| (item.clone(), sx + x, sy + y_off)),
                );
                // Boundary feeds of the block become CHAIN boundary
                // records at the placed offset; each drain head anchors
                // one outgoing corridor.
                for r in &block.boundary_inputs {
                    // Chain-fed heads are supplied by chain corridors,
                    // not the outside world — they are ports, not
                    // chain boundary records (increment 2).
                    let cf = &mega_plan.as_ref().expect("mega spec implies plan").chain_fed;
                    if cf.iter().any(|(i, _)| i == &r.item) {
                        continue;
                    }
                    // Both axes translate (unit-2 recon finding: x-only
                    // held only because a mega block's y_off is always 0
                    // today — the drain map below always translated both).
                    b_in.push(BoundaryRecord { x: r.x + x, y: r.y + y_off, ..r.clone() });
                }
                if block.boundary_outputs.is_empty() {
                    return Err("mega: block has no drain record".to_string());
                }
                block
                    .boundary_outputs
                    .iter()
                    .map(|d| (d.x + x, d.y + y_off, d.item.clone()))
                    .collect()
            } else {
                Vec::new()
            };
            placed.push(Placed {
                cell,
                x,
                y_off,
                mega_drains,
                slot_x,
                vlane_base: vlane0,
                recipe: m.recipe.clone(),
                seg: if kq > 1 {
                    format!("{}#{}", m.recipe, copy + 1)
                } else {
                    m.recipe.clone()
                },
                copy,
                ext_inputs,
            });
        }
    }

    let band_bottom = placed
        .iter()
        .map(|p| p.y_off + p.cell.height)
        .max()
        .unwrap_or(CELL_Y)
        + 1;

    // --- External feeds: per cell, columns west of it (pitch 4), north
    // boundary at y=0, corner east into the port terminal. Inner column
    // serves the topmost port (no crossings among a slot's own feeds).
    let mut router = Router::new();
    for p in &placed {
        // MULTI-ROW cells expose one in-port PER ROW for the same item —
        // every port gets its own feed column (a single-port find left
        // second-row machines unfed: belt-flow-reachability caught it).
        let mut targets: Vec<(String, i32, i32)> = Vec::new();
        for item in &p.ext_inputs {
            let mut found = false;
            for port in p.cell.ports.iter().filter(|q| q.inbound && q.item == *item) {
                let (tx, ty) = port_abs(port, p.x, p.y_off);
                targets.push((item.clone(), tx, ty));
                found = true;
            }
            if !found {
                return Err(format!("cells: {} lacks in-port for {item}", p.recipe));
            }
        }
        targets.sort_by_key(|t| t.2);
        for (i, (item, tx, ty)) in targets.iter().enumerate() {
            let col_x = p.slot_x + targets.len() as i32 * FEED_PITCH
                - FEED_PITCH * i as i32 - FEED_PITCH + 1;
            stamp_path(
                &mut entities,
                &[(col_x, 0), (col_x, *ty), (tx - 1, *ty)],
                item,
                "express-transport-belt",
                &format!("feed:{item}:{}", p.seg),
            );
            router.register_col(col_x, 0, *ty);
            router.register_row(*ty, col_x, tx - 1);
            b_in.push(BoundaryRecord {
                item: item.clone(),
                x: col_x,
                y: 0,
                direction: EntityDirection::South,
                is_fluid: false,
                entity: "express-transport-belt".into(),
            });
        }
    }

    // Bypass rows sit between the band bottom and the drain row, so the
    // sim's drain rig (which builds south of the drain head) never
    // collides with them. Count bypass edges up front — PER COPY: the
    // copies' x-ranges are disjoint, so every copy reuses the same rows.
    let n_bypass: i32 = specs
        .iter()
        .enumerate()
        .map(|(pi, m)| {
            let pi_mega = m.recipe.starts_with(MEGA_PREFIX);
            // Mega specs count every output's edge (one bypass row per
            // drained corridor); solid specs keep the historical
            // primary-output count exactly (bit-identity).
            let outs: &[crate::models::ItemFlow] =
                if pi_mega { &m.outputs } else { &m.outputs[..1] };
            outs.iter()
                .map(|o| {
                    let edges = specs
                        .iter()
                        .enumerate()
                        .filter(|(ci, c)| {
                            *ci != pi
                                && (pi_mega || *ci != pi + 1)
                                && c.inputs.iter().any(|i| i.item == o.item)
                                && cell_cache[*ci]
                                    .ports
                                    .iter()
                                    .any(|q| q.inbound && q.item == o.item)
                        })
                        .count() as i32;
                    // A fanned mega output (>=2 consumers, Phase C)
                    // spends one extra row hosting the splitter chain
                    // before the per-branch rows.
                    if pi_mega && edges >= 2 { edges + 1 } else { edges }
                })
                .sum::<i32>()
        })
        .sum();
    // Bypass rows sit at PITCH 3 (band_bottom+1, +4, +7, ...): at
    // pitch 1 a leg crossing one row lands its hop mouths and terminal
    // ON the neighboring rows (mouths sit at R±1, a descent terminal at
    // its own row−1) — the residual mil5-ore overlap class. Pitch 2
    // still fails (adjacent rows cluster into one hop whose mouths jump
    // to the next row); pitch 3 keeps every crossing un-clustered with
    // free mouth rows. n_bypass ≤ 1 is bit-identical to the old +n+2.
    let drain_row = band_bottom + 2 + (3 * n_bypass - 2).max(0);

    // --- Chain corridors: producer out → merge (if 2 runs) → fan-out
    // split (if 2 consumers) → per-consumer routing via the Router.
    // Bypass rows allocate per copy (disjoint x-ranges share rows).
    // Occupancy seeds here: cells + feeds are down, and collision
    // fallbacks below check against everything stamped so far.
    router.seed(&entities);
    let mut bypass_idx: FxHashMap<i32, i32> = FxHashMap::default();
    // Per-slot vertical-lane allocation: each bypass descent/ascent
    // claims a fresh lane in its slot's strip (two edges sharing a lane
    // was the mil5-ore overlap class); multi-edge strips step by 2
    // (matching the strip sizing above).
    let mut lane_next: FxHashMap<usize, i32> = FxHashMap::default();
    let alloc_lane = |lane_next: &mut FxHashMap<usize, i32>, slot: usize, base: i32, step: i32| -> i32 {
        let n = lane_next.entry(slot).or_insert(0);
        let x = base + *n * step;
        *n += 1;
        x
    };
    let lane_step = |demand: i32| if demand >= 2 { 2 } else { 1 };
    for (pi, p) in placed.iter().enumerate() {
        let out_item = specs[pi % n].outputs[0].item.clone();
        // Consumers within THIS copy only — the same item flows in every
        // copy, and cross-copy corridors would defeat the quantization.
        let consumers: Vec<usize> = placed
            .iter()
            .enumerate()
            .filter(|(ci, c)| {
                *ci != pi
                    && c.copy == p.copy
                    && specs[*ci % n].inputs.iter().any(|i| i.item == out_item)
                    && c.cell.ports.iter().any(|q| q.inbound && q.item == out_item)
            })
            .map(|(ci, _)| ci)
            .collect();
        // Mega producer (RFC-052 Phase B): each solid output exits
        // SOUTH at its own drain head — no merge/fan machinery, and
        // each corridor rides its own bypass row from directly below
        // its drain to its (single, eligibility-enforced) consumer.
        // Outputs with no consumer stay chain exports at their drain.
        if !p.mega_drains.is_empty() {
            for (dx0, dy0, d_item) in p.mega_drains.clone() {
                let drain_consumers: Vec<usize> = placed
                    .iter()
                    .enumerate()
                    .filter(|(ci, c)| {
                        *ci != pi
                            && c.copy == p.copy
                            && specs[*ci % n].inputs.iter().any(|i| i.item == d_item)
                            && c.cell.ports.iter().any(|q| q.inbound && q.item == d_item)
                    })
                    .map(|(ci, _)| ci)
                    .collect();
                // Fan path (Phase C): >=2 consumers split on the drain's
                // own bypass row, then each branch drops to a fresh row
                // and rides to its consumer via the shared delivery
                // idiom. The single-consumer path below is byte-identical
                // to Phase B (registry gate).
                if drain_consumers.len() >= 2 {
                    let row = bypass_idx.entry(p.copy).or_insert(0);
                    let fan_y = band_bottom + 1 + 3 * *row;
                    *row += 1;
                    let seg0 = format!("fan:{}:{}", d_item, p.seg);
                    router.vcol(&mut entities, dx0, dy0 + 1, fan_y - 1, &d_item,
                        "express-transport-belt", "express-underground-belt", &seg0);
                    router.corner_east(&mut entities, dx0, fan_y, &d_item, "express-transport-belt", &seg0);
                    let mut ordered = drain_consumers.clone();
                    ordered.sort_by_key(|ci| placed[*ci].x);
                    // Splitter chain east of the corner; branch b exits
                    // south at (sx+1, fan_y+1); the last branch is the
                    // pass-through at (sx_last+1, fan_y).
                    let n_br = ordered.len();
                    // Splitter/branch columns must dodge the OTHER
                    // drains' head columns — each of those descends
                    // from the block bottom through every bypass row,
                    // and a branch drop on the same x overlaps it (the
                    // USP@2 overlap cluster at x=272).
                    let reserved: FxHashSet<i32> = p
                        .mega_drains
                        .iter()
                        .filter(|(ox, _, oi)| *ox != dx0 || oi != &d_item)
                        .map(|(ox, _, _)| *ox)
                        .collect();
                    let mut branch_origins: Vec<(i32, i32)> = Vec::new();
                    let mut cursor = dx0 + 1;
                    for b in 1..n_br {
                        let mut sx = cursor;
                        while reserved.contains(&sx) || reserved.contains(&(sx + 1)) {
                            sx += 1;
                        }
                        // Bridge from the previous chain tile to the
                        // splitter input with plain belts.
                        for xx in cursor..sx {
                            router.corner_east(&mut entities, xx, fan_y, &d_item,
                                "express-transport-belt", &format!("fan:{}:{}", d_item, p.seg));
                        }
                        router.occ.insert((sx, fan_y));
                        router.occ.insert((sx, fan_y + 1));
                        entities.push(PlacedEntity {
                            name: "express-splitter".into(), x: sx, y: fan_y,
                            direction: EntityDirection::East,
                            carries: Some(d_item.clone()),
                            segment_id: Some(format!("fan{b}:{}:{}", d_item, p.seg)),
                            ..Default::default()
                        });
                        branch_origins.push((sx + 1, fan_y + 1));
                        if b < n_br - 1 {
                            router.corner_east(&mut entities, sx + 1, fan_y, &d_item,
                                "express-transport-belt", &format!("fan:{}:{}", d_item, p.seg));
                        }
                        cursor = sx + 2;
                    }
                    // Pass-through: step EAST off the last splitter's
                    // output column (branch n-1 drops at sx+1 from the
                    // south output; sharing that column overlapped the
                    // two descents) to its own reserved-dodged column.
                    let mut pt_x = cursor;
                    while reserved.contains(&pt_x) {
                        pt_x += 1;
                    }
                    for xx in (cursor - 1)..pt_x {
                        router.corner_east(&mut entities, xx, fan_y, &d_item,
                            "express-transport-belt", &format!("fan:{}:{}", d_item, p.seg));
                    }
                    branch_origins.push((pt_x, fan_y));
                    for (bi, ci) in ordered.iter().enumerate() {
                        let c = &placed[*ci];
                        let port = single_inbound_port(&c.cell.ports, &d_item, &c.recipe)?;
                        let (tx, ty) = port_abs(port, c.x, c.y_off);
                        let (bx, by) = branch_origins[bi];
                        let seg = format!("corr:{}:{}", p.seg, c.seg);
                        let up_demand = lane_demand[*ci % n];
                        let lane_up = alloc_lane(&mut lane_next, *ci, c.vlane_base, lane_step(up_demand));
                        let row = bypass_idx.entry(p.copy).or_insert(0);
                        let by_y = band_bottom + 1 + 3 * *row;
                        *row += 1;
                        // Drop from the branch origin to this branch's
                        // own row THROUGH an allocated lane in the next
                        // slot's strip (sized via lane_demand above) —
                        // ad-hoc drops at splitter columns collided
                        // with consumer ascent lanes. Falls back to an
                        // in-place corner drop when the run east is not
                        // stampable (the solid path's in-gap idiom).
                        let (drop_x, drop_top) = if pi + 1 < placed.len() {
                            let down_demand = lane_demand[(pi + 1) % n];
                            let cand = placed[pi + 1].vlane_base
                                + *lane_next.get(&(pi + 1)).unwrap_or(&0)
                                    * lane_step(down_demand);
                            if cand > bx && router.is_row_stampable(by, bx, cand - 1) {
                                let lane_down = alloc_lane(
                                    &mut lane_next,
                                    pi + 1,
                                    placed[pi + 1].vlane_base,
                                    lane_step(down_demand),
                                );
                                router.hrow(&mut entities, by, bx, lane_down - 1, &d_item,
                                    "express-transport-belt", "express-underground-belt", &seg);
                                (lane_down, by)
                            } else {
                                router.corner_south(&mut entities, bx, by, &d_item, "express-transport-belt", &seg);
                                (bx, by + 1)
                            }
                        } else {
                            router.corner_south(&mut entities, bx, by, &d_item, "express-transport-belt", &seg);
                            (bx, by + 1)
                        };
                        router.vcol(&mut entities, drop_x, drop_top, by_y - 1, &d_item,
                            "express-transport-belt", "express-underground-belt", &seg);
                        if lane_up < drop_x {
                            router.corner_west(&mut entities, drop_x, by_y, &d_item, "express-transport-belt", &seg);
                            router.hrow_west(&mut entities, by_y, drop_x - 1, lane_up + 1, &d_item,
                                "express-transport-belt", "express-underground-belt", &seg);
                            router.corner_north(&mut entities, lane_up, by_y, &d_item, "express-transport-belt", &seg);
                        } else {
                            router.corner_east(&mut entities, drop_x, by_y, &d_item, "express-transport-belt", &seg);
                            router.hrow(&mut entities, by_y, drop_x + 1, lane_up - 1, &d_item,
                                "express-transport-belt", "express-underground-belt", &seg);
                            router.occ.insert((lane_up, by_y));
                            entities.push(PlacedEntity {
                                name: "express-transport-belt".into(), x: lane_up, y: by_y,
                                direction: EntityDirection::North,
                                carries: Some(d_item.clone()),
                                segment_id: Some(seg.clone()), ..Default::default()
                            });
                        }
                        if router.occ.contains(&(lane_up, ty + 1)) {
                            retrofit_feed_hop(&mut entities, &mut router, lane_up, ty + 1)?;
                        }
                        router.vcol(&mut entities, lane_up, by_y - 1, ty + 1, &d_item,
                            "express-transport-belt", "express-underground-belt", &seg);
                        router.corner_east(&mut entities, lane_up, ty, &d_item, "express-transport-belt", &seg);
                        router.hrow(&mut entities, ty, lane_up + 1, tx - 1, &d_item,
                            "express-transport-belt", "express-underground-belt", &seg);
                    }
                    continue;
                }
                let consumer = drain_consumers.first().map(|&ci| (ci, &placed[ci]));
                let Some((ci, c)) = consumer else {
                    // Consumer-less export: extend the drain to the
                    // chain's drain row like the solid final-product
                    // path — a b_out mid-layout leaves a dead-end belt
                    // (the exemption is bounds-based) and puts the sim
                    // rig at a nonuniform depth (#363).
                    //
                    // Express, matching the corridor convention: this
                    // mega-output's own rate is not in scope here, and a
                    // hardcoded yellow is the same 15/s ceiling that
                    // capped gear@20 at 75% on the final-product path
                    // (#700). Over-tiering an export drain costs
                    // nothing; under-tiering silently caps the chain.
                    let seg = format!("out:{}", p.seg);
                    router.vcol(&mut entities, dx0, dy0 + 1, drain_row, &d_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    b_out.push(BoundaryRecord {
                        item: d_item.clone(),
                        x: dx0,
                        y: drain_row,
                        direction: EntityDirection::South,
                        is_fluid: false,
                        entity: "express-transport-belt".into(),
                    });
                    continue;
                };
                let port = single_inbound_port(&c.cell.ports, &d_item, &c.recipe)?;
                let (tx, ty) = port_abs(port, c.x, c.y_off);
                let seg = format!("corr:{}:{}", p.seg, c.seg);
                let up_demand = lane_demand[ci % n];
                let lane_up = alloc_lane(&mut lane_next, ci, c.vlane_base, lane_step(up_demand));
                let row = bypass_idx.entry(p.copy).or_insert(0);
                let by_y = band_bottom + 1 + 3 * *row;
                *row += 1;
                router.vcol(&mut entities, dx0, dy0 + 1, by_y - 1, &d_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                if lane_up < dx0 {
                    // WESTWARD consumer (#405 review finding 1): the
                    // dependency-position invariant makes this
                    // analytically unreachable today, but an eastward
                    // hrow with x0>x1 stamps NOTHING silently (the
                    // mil5-ore landmine class) — mirror the solid
                    // bypass path's guard instead of trusting the
                    // invariant forever.
                    router.corner_west(&mut entities, dx0, by_y, &d_item, "express-transport-belt", &seg);
                    router.hrow_west(&mut entities, by_y, dx0 - 1, lane_up + 1, &d_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.corner_north(&mut entities, lane_up, by_y, &d_item, "express-transport-belt", &seg);
                } else {
                    router.corner_east(&mut entities, dx0, by_y, &d_item, "express-transport-belt", &seg);
                    router.hrow(&mut entities, by_y, dx0 + 1, lane_up - 1, &d_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.occ.insert((lane_up, by_y));
                    entities.push(PlacedEntity {
                        name: "express-transport-belt".into(), x: lane_up, y: by_y,
                        direction: EntityDirection::North,
                        carries: Some(d_item.clone()),
                        segment_id: Some(seg.clone()), ..Default::default()
                    });
                }
                if router.occ.contains(&(lane_up, ty + 1)) {
                    retrofit_feed_hop(&mut entities, &mut router, lane_up, ty + 1)?;
                }
                router.vcol(&mut entities, lane_up, by_y - 1, ty + 1, &d_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                router.corner_east(&mut entities, lane_up, ty, &d_item, "express-transport-belt", &seg);
                router.hrow(&mut entities, ty, lane_up + 1, tx - 1, &d_item,
                    "express-transport-belt", "express-underground-belt", &seg);
            }
            continue;
        }
        let outs: Vec<&Port> = p.cell.ports.iter().filter(|q| !q.inbound).collect();
        if consumers.is_empty() {
            // Final product: corner south past the band, drain record.
            //
            // The drain tier is chosen FOR THE PLANNED RATE, not hardcoded.
            // A hardcoded "transport-belt" here capped the whole chain at
            // 15/s regardless of plan — gear@20's shipped 75% (#700) was
            // six yellow exit tiles on a 20/s product, meter-verified
            // (patching exactly those tiles to fast measures 20.0/20.0,
            // all 8 machines working).
            //
            // Rate semantics (#715 review round 2 corrected the wording):
            // `outputs[0].rate * count` with NO `scale` factor is the FULL
            // chain rate (the per-copy share is `* scale` — see the cell
            // build above). Drains are PER-COPY and disjoint, so at K>1
            // each copy's drain is over-tiered by up to K× — deliberately:
            // over-tiering costs a belt tier, under-tiering caps the
            // chain. Anyone optimizing this to `* scale` must prove the
            // kq used here matches the placed copies. Known ceiling, out
            // of scope here: `belt_entity_for_rate` tops out at express,
            // so a single-column drain caps at 45/s and a plan above that
            // would under-deliver at the exit. (#730 r3+r4: the
            // under-delivery warning below is provably dead for ALL K,
            // not just K≥2 — `required_copies` bounds per_copy_drain =
            // drain_rate/kq ≤ QUANTUM_RATE (40) < express (45) whenever
            // the ladder tops out, and below the top the tier is chosen
            // for the FULL drain_rate so its cap already covers the
            // per-copy share. Kept as defensive code, NOT an assert (a
            // warning-class condition must never become a panic path):
            // it goes live again if the quantum ever exceeds a belt cap
            // or the drain sizing decouples from `required_copies`. The
            // #715 loud-exit contract is carried by the quantization
            // bound itself now — RFC-072 decision log, #730 round 4.)
            let spec = &specs[pi % n];
            let drain_rate = spec.outputs[0].rate * spec.count as f64;
            // TIER from the FULL rate, warning against the PER-COPY
            // share (#730 round 6): `belt_entity_for_rate(drain_rate)`
            // and the `per_copy_drain` comparison below are a coupled
            // pair — the tier deliberately over-provisions (full rate)
            // while the warning measures what one copy's exit actually
            // carries. Changing either side's rate argument without
            // the other re-opens the phantom-warning class (K>1 exits
            // compared at K× their real flow) or the under-tier class.
            let drain_belt = crate::common::belt_entity_for_rate(drain_rate, None);
            // The tier ladder tops out at express: a single-column drain
            // caps at 45/s, and a plan above that would under-deliver at
            // the exit — the same class this fix kills at 15/s, one tier
            // up, and reachable from the web app with an arbitrary rate.
            // Loud, not silent (#715 review round 3, 3/3): the warning
            // rides LayoutResult.warnings, where selection's
            // layout-warnings floor counts it and a user can read it.
            let drain_cap = crate::common::belt_throughput(drain_belt);
            // The WARNING compares the PER-COPY rate (each copy drains its
            // own disjoint exit at full_rate/kq) — the tier selection above
            // deliberately stays full-rate (recorded over-tiering). The
            // unscaled comparison fired falsely on every K>1 chain
            // (480/s vs 45 on drains carrying 40/s), and those phantom
            // warnings entered selection's layout-warnings floor
            // (codex review of RFC-072 P2 unit 1).
            let per_copy_drain = drain_rate / kq as f64;
            if per_copy_drain > drain_cap {
                chain_warnings.push(format!(
                    "cell chain exit for {} carries {per_copy_drain:.1}/s per copy on a \
                     single {} column capped at {drain_cap:.0}/s — the drain \
                     under-delivers; no single-belt tier can carry this plan",
                    out_item, drain_belt
                ));
            }
            let drain_ug = match drain_belt {
                "express-transport-belt" => "express-underground-belt",
                "fast-transport-belt" => "fast-underground-belt",
                _ => "underground-belt",
            };
            let o1 = outs.first().ok_or_else(|| format!("cells: {} has no out port", p.recipe))?;
            let (ox, oy) = port_abs(o1, p.x, p.y_off);
            let drain_x = ox + 2;
            let seg = format!("out:{}", p.seg);
            router.hrow(&mut entities, oy, ox + 1, drain_x - 1, &out_item,
                drain_belt, drain_ug, &seg);
            router.occ.insert((drain_x, oy));
            entities.push(PlacedEntity {
                name: drain_belt.into(), x: drain_x, y: oy,
                direction: EntityDirection::South,
                carries: Some(out_item.clone()),
                segment_id: Some(seg.clone()), ..Default::default()
            });
            router.vcol(&mut entities, drain_x, oy + 1, drain_row, &out_item,
                drain_belt, drain_ug, &seg);
            b_out.push(BoundaryRecord {
                item: out_item.clone(),
                x: drain_x,
                y: drain_row,
                direction: EntityDirection::South,
                is_fluid: false,
                entity: drain_belt.into(),
            });
            continue;
        }

        // Collect the cell's out-runs into ONE eastbound run via a
        // cascade of 2→1 splitters (below-approach corner idiom per
        // stage; the Router hops any crossings). Runs sorted by y — the
        // topmost is the accumulator row.
        let mut outs_sorted = outs.clone();
        outs_sorted.sort_by_key(|q| q.y);
        let (acc_x0, acc_y) = port_abs(outs_sorted[0], p.x, p.y_off);
        let base_sx = p.x + p.cell.width + 2;
        router.hrow(&mut entities, acc_y, acc_x0 + 1, base_sx - 1, &out_item,
            "express-transport-belt", "express-underground-belt", &format!("cc:a:{}", p.seg));
        let mut run_x = base_sx;
        for (k, o) in outs_sorted.iter().enumerate().skip(1) {
            let (ox, oy) = port_abs(o, p.x, p.y_off);
            assert!(oy > acc_y + 1, "cells: merge assumes below-approach ({oy} vs {acc_y})");
            let seg = format!("cc:b{k}:{}", p.seg);
            router.hrow(&mut entities, oy, ox + 1, run_x - 2, &out_item,
                "express-transport-belt", "express-underground-belt", &seg);
            router.corner_north(&mut entities, run_x - 1, oy, &out_item, "express-transport-belt", &seg);
            router.vcol(&mut entities, run_x - 1, oy - 1, acc_y + 2, &out_item,
                "express-transport-belt", "express-underground-belt", &seg);
            router.corner_east(&mut entities, run_x - 1, acc_y + 1, &out_item, "express-transport-belt", &seg);
            router.occ.insert((run_x, acc_y));
            router.occ.insert((run_x, acc_y + 1));
            entities.push(PlacedEntity {
                name: "express-splitter".into(), x: run_x, y: acc_y,
                direction: EntityDirection::East,
                carries: Some(out_item.clone()),
                segment_id: Some(format!("cc:m{k}:{}", p.seg)), ..Default::default()
            });
            run_x += 2;
            if k < outs_sorted.len() - 1 {
                // Bridge to the next merge stage's input tile.
                router.corner_east(&mut entities, run_x - 1, acc_y, &out_item, "fast-transport-belt", &format!("cc:a:{}", p.seg));
            }
        }
        let run_y = acc_y;
        // After a merge cascade the collected flow's next free tile is
        // run_x - 1 (the last splitter sits at run_x - 2); with a single
        // out-run nothing was consumed east of the hrow, so it's run_x.
        let pass_x = if outs_sorted.len() > 1 { run_x - 1 } else { run_x };

        // Fan-out: a chain of 1→2 splitters, one per extra consumer.
        // Branch b exits south at splitter b's (x+1, y+1); the last
        // consumer takes the pass-through east output.
        let n_branches = consumers.len();
        let mut branch_origins: Vec<(i32, i32)> = Vec::new();
        let mut fx = pass_x;
        for b in 1..n_branches {
            router.occ.insert((fx, run_y));
            router.occ.insert((fx, run_y + 1));
            entities.push(PlacedEntity {
                name: "express-splitter".into(), x: fx, y: run_y,
                direction: EntityDirection::East,
                carries: Some(out_item.clone()),
                segment_id: Some(format!("fan{b}:{}", p.seg)), ..Default::default()
            });
            branch_origins.push((fx + 1, run_y + 1));
            if b < n_branches - 1 {
                router.corner_east(&mut entities, fx + 1, run_y, &out_item, "fast-transport-belt", &format!("fan:{}", p.seg));
            }
            fx += 2;
        }
        // Pass-through (or the only) branch.
        branch_origins.push((
            if n_branches > 1 {
                // The dependency order never sends two fan branches
                // west, so it can begin at the splitters' shared output
                // column. (The compact order that could was deleted
                // 2026-08-20 with ChainOrder::Compact.)
                fx - 1
            } else {
                pass_x
            },
            run_y,
        ));

        // Route each branch. Adjacent-east consumer: port-row corridor
        // (with a vertical jog on the consumer slot's first lane if the
        // rows differ). Farther consumer: south bypass under the band.
        let mut ordered = consumers.clone();
        ordered.sort_by_key(|ci| placed[*ci].x);
        for (bi, ci) in ordered.iter().enumerate() {
            let c = &placed[*ci];
            let port = single_inbound_port(&c.cell.ports, &out_item, &c.recipe)?;
            let (tx, ty) = port_abs(port, c.x, c.y_off);
            let (bx, by) = branch_origins[bi];
            let seg = format!("corr:{}:{}", p.seg, c.seg);
            if *ci == pi + 1 {
                if by == ty {
                    router.hrow(&mut entities, ty, bx, tx - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                } else {
                    // Early jog: one east tile at the branch origin, then
                    // vertical at bx+1 down/up to the TARGET port row, then
                    // east all the way. The stagger keeps a sibling
                    // fan-out branch's row clear of this jog column (it
                    // hops under it via the registry).
                    let vdir = (ty - by).signum();
                    router.corner_east(&mut entities, bx, by, &out_item, "express-transport-belt", &seg);
                    router.vcol(&mut entities, bx + 1, by, ty - vdir, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.corner_east(&mut entities, bx + 1, ty, &out_item, "express-transport-belt", &seg);
                    router.hrow(&mut entities, ty, bx + 2, tx - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                }
            } else {
                // South bypass below the cell band.
                // In-copy by construction: the last slot of a copy is
                // always the sink (dependency order), which has no
                // consumers and never reaches this branch.
                let up_demand = lane_demand[*ci % n];
                let lane_up = alloc_lane(&mut lane_next, *ci, c.vlane_base, lane_step(up_demand));
                let row = bypass_idx.entry(p.copy).or_insert(0);
                let by_y = band_bottom + 1 + 3 * *row;
                *row += 1;
                if lane_up < bx {
                    // WESTWARD consumer: the reversed-dependency
                    // placement can put an item's consumer west of its
                    // producer (shared inputs pulled in at different
                    // depths). Compact fan origins are allocated on
                    // distinct columns, so descend in-gap, run west along the
                    // bypass row, corner north into the consumer's
                    // strip lane; the ascent + port approach below are
                    // position-relative and shared with the eastward
                    // path.
                    router.corner_south(&mut entities, bx, by, &out_item, "express-transport-belt", &seg);
                    router.vcol(&mut entities, bx, by + 1, by_y - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.corner_west(&mut entities, bx, by_y, &out_item, "express-transport-belt", &seg);
                    router.hrow_west(&mut entities, by_y, bx - 1, lane_up + 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.corner_north(&mut entities, lane_up, by_y, &out_item, "express-transport-belt", &seg);
                } else {
                    debug_assert_eq!(placed[pi + 1].copy, p.copy,
                        "bypass descent lane must stay in-copy");
                    // Descent: legacy path runs east on the branch row
                    // to a lane in the NEXT slot's strip. When that row
                    // segment is already occupied (sibling fan-out
                    // branches share the branch row — a mil5-ore
                    // overlap class), descend IN-GAP instead: corner
                    // south at the branch origin and drop straight to
                    // the bypass row inside the producer's own gap,
                    // where nothing else runs.
                    let down_demand = lane_demand[(pi + 1) % n];
                    let legacy_lane_down = placed[pi + 1].vlane_base
                        + *lane_next.get(&(pi + 1)).unwrap_or(&0) * lane_step(down_demand);
                    let (drop_x, drop_top) = if legacy_lane_down < lane_up
                        && router.is_row_stampable(by, bx, legacy_lane_down - 1)
                    {
                        let lane_down = alloc_lane(&mut lane_next, pi + 1, placed[pi + 1].vlane_base, lane_step(down_demand));
                        router.hrow(&mut entities, by, bx, lane_down - 1, &out_item,
                            "express-transport-belt", "express-underground-belt", &seg);
                        (lane_down, by)
                    } else {
                        router.corner_south(&mut entities, bx, by, &out_item, "express-transport-belt", &seg);
                        (bx, by + 1)
                    };
                    router.vcol(&mut entities, drop_x, drop_top, by_y - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.corner_east(&mut entities, drop_x, by_y, &out_item, "express-transport-belt", &seg);
                    router.hrow(&mut entities, by_y, drop_x + 1, lane_up - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.occ.insert((lane_up, by_y));
                    entities.push(PlacedEntity {
                        name: "express-transport-belt".into(), x: lane_up, y: by_y,
                        direction: EntityDirection::North,
                        carries: Some(out_item.clone()),
                        segment_id: Some(seg.clone()), ..Default::default()
                    });
                }
                // Ascent terminal: the vcol ends at ty+1 to corner into
                // the port row — but another item's FEED ROW can sit at
                // exactly ty+1 (crossing hops only cover strict
                // interiors; terminals are boundary tiles). Retrofit the
                // feed with a local UG hop under this lane, freeing the
                // terminal tile.
                if router.occ.contains(&(lane_up, ty + 1)) {
                    retrofit_feed_hop(&mut entities, &mut router, lane_up, ty + 1)?;
                }
                router.vcol(&mut entities, lane_up, by_y - 1, ty + 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                router.corner_east(&mut entities, lane_up, ty, &out_item, "express-transport-belt", &seg);
                router.hrow(&mut entities, ty, lane_up + 1, tx - 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
            }
        }
    }

    // --- Poles: per-cell trio down the corridor gap + a spanning line
    // along the band bottom (nudge-not-skip — Phase-1 pole lesson).
    let mut occupied: FxHashSet<(i32, i32)> = entities.iter().map(|e| (e.x, e.y)).collect();
    for p in &placed {
        let px = p.x + p.cell.width + CORRIDOR_GAP - 1;
        for y in [CELL_Y, CELL_Y + 7, CELL_Y + 14] {
            if y < band_bottom {
                let mut yy = y;
                while occupied.contains(&(px, yy)) {
                    yy += 1;
                }
                // The snapshot must learn each placement — two trio
                // members sliding down one congested column otherwise
                // land on the SAME first-free tile (USP@2 pole-pole
                // overlaps).
                occupied.insert((px, yy));
                entities.push(PlacedEntity {
                    name: "medium-electric-pole".into(), x: px, y: yy,
                    direction: EntityDirection::North,
                    segment_id: Some("pole".into()), ..Default::default()
                });
            }
        }
    }
    let width = entities.iter().map(|e| e.x).max().unwrap_or(0) + 1;
    // Spanning pole line. The step is measured from the LAST POLE ACTUALLY
    // PLACED, not from an absolute grid: pitch 8 against wire reach 9 leaves
    // exactly one tile of slack, so a pole nudged forward past congestion put
    // itself out of reach of its predecessor and silently broke the chain it
    // was nudging to preserve (nudged gap 12, or 16 when no free tile was
    // found at all — both > 9). Advancing from the last placement keeps every
    // consecutive gap within reach regardless of how far a nudge travelled.
    let mut last_x: Option<i32> = None;
    let mut px = 1;
    while px < width {
        let placed_at = (0..5).map(|n| px + n).find(|&x| {
            x < width + 4 && !occupied.contains(&(x, band_bottom))
        });
        if let Some(x) = placed_at {
            occupied.insert((x, band_bottom));
            entities.push(PlacedEntity {
                name: "medium-electric-pole".into(), x, y: band_bottom,
                direction: EntityDirection::North,
                segment_id: Some("pole".into()), ..Default::default()
            });
            last_x = Some(x);
        }
        px = last_x.map_or(px + 8, |x| x + 8);
    }

    let height = (entities.iter().map(|e| e.y).max().unwrap_or(0) + 1).max(band_bottom + 2);
    let mut composed = LayoutResult {
        entities,
        width,
        height,
        warnings: chain_warnings,
        stacking: 1,
        // Declared axes travel with the rebuilt result (a rebuilt
        // LayoutResult must re-declare stacking/productivity/capacity).
        research_productivity: Default::default(),
        // The composed layout DECLARES the capacity its cells were sized
        // at — registry world-matching (`verification_note`) reads this.
        inserter_capacity,
        boundary_inputs: {
            let mut b = b_in;
            b.sort_by_key(|r| r.x); // west→east (#363 rig-depth rule)
            b
        },
        boundary_outputs: b_out,
        // Mega blocks own their internal fluid topology, including any
        // physically routed byproduct relief exits. Preserve those records
        // at chain coordinates so the top-level stranded-byproduct check
        // can cross-check them against the translated pipe entities (#476).
        surplus_exits,
        // The typed receipt (RFC-074 Unit 1): one strip of `kq` copies.
        // Verification is attached by the selection candidate.
        composition: Some(crate::models::CompositionReceipt {
            kind: "cell-chain".to_string(),
            copies_per_strip: vec![kq],
            strips: vec![crate::models::StripRect { x: 0, y: 0, width, height, copies: kq }],
            ..Default::default()
        }),
        ..Default::default()
    };
    // Heuristic pole placement leaves islands, which is why
    // `build_bus_layout` follows its own placement with this repair.
    // Composition never did, and shipped layouts whose power network was in
    // as many as 41 pieces — every machine covered, most of them unreachable
    // from any single power source. Additive: bridge poles only.
    crate::bus::layout::repair_pole_network(&mut composed);
    Ok(composed)
}

/// The inter-strip clearance tiles a grid's bridge poles must not use —
/// the sim harness's rig footprints around every INTERIOR boundary head,
/// taken as the union over every slot the harness's stagger ladder can
/// assign (the composer cannot know which slot a head gets):
///
/// - a feed head on a strip's top edge (flow south): the outward column
///   above it for the whole clearance; for each ladder depth
///   `4 + 6·slot` (4, 10, 16, 22, 28) the 18-tile jog row WEST of the
///   head at `top − depth` — the harness's `rot90` puts the item jog on
///   that side, which is the side the K=20 ec@240 collision was on —
///   the chest bank (±2 rows) at jog 10–12 and the 2×2 substation/EEI at
///   jog 15/18 (`sim-harness/scenario.rs::feed_footprint`). Fluid heads
///   are the column only.
/// - a drain head on a strip's bottom edge (flow south): the extension
///   column below it, the ±2 lateral band and the 2×2 kit at lateral
///   +4/+7 — over the whole clearance, since the extension is staggered
///   per cluster (`drain_footprint`).
///
/// Deliberately the harness's SHAPES rather than a lane: on a narrow copy
/// (the from-plates ec cell is 68 wide with three heads) a ±22 lane per
/// head tiles the entire clearance and no bridge can exist, while the
/// real rigs leave most of it free. The harness's pre-flight guard
/// (`assert_rigs_clear_of_layout`) remains the oracle; this is what lets
/// the composer avoid tripping it.
pub fn interior_rig_lanes(
    strip_rects: &[crate::models::StripRect],
    layout: &LayoutResult,
) -> FxHashSet<(i32, i32)> {
    const FEED_DEPTHS: [i32; 5] = [4, 10, 16, 22, 28];
    let mut ko = FxHashSet::default();
    let last = strip_rects.len().saturating_sub(1);
    for (s, r) in strip_rects.iter().enumerate() {
        let top = r.y;
        let bottom = r.y + r.height - 1;
        if s > 0 {
            for rec in layout.boundary_inputs.iter().filter(|b| b.y == top) {
                let hx = rec.x;
                for d in 1..=STRIP_CLEARANCE {
                    ko.insert((hx, top - d));
                }
                if rec.is_fluid {
                    continue;
                }
                for &depth in &FEED_DEPTHS {
                    let jy = top - depth;
                    for k in 1..=18 {
                        ko.insert((hx - k, jy));
                    }
                    for k in 10..=12 {
                        for dy in -2..=2 {
                            ko.insert((hx - k, jy + dy));
                        }
                    }
                    for k in [15, 18] {
                        for dx in -1..=0 {
                            for dy in -1..=0 {
                                ko.insert((hx - k + dx, jy + dy));
                            }
                        }
                    }
                }
            }
        }
        if s < last {
            for rec in layout.boundary_outputs.iter().filter(|b| b.y == bottom) {
                let hx = rec.x;
                for d in 1..=STRIP_CLEARANCE {
                    let y = bottom + d;
                    for dx in -2..=2 {
                        ko.insert((hx + dx, y));
                    }
                    for off in [4, 7] {
                        for dx in -1..=0 {
                            for dy in -1..=0 {
                                ko.insert((hx + off + dx, y + dy));
                            }
                        }
                    }
                }
            }
        }
    }
    ko
}

/// Stamp one explicit medium-pole bridge column per inter-strip gap at
/// the x with the most rig-free clearance rows (ties → nearest the grid's
/// centre), poles chained down the free rows at pitch ≤ 8 against wire
/// reach 9. Returns the poles added. The column meets each strip's own
/// pole line only if one of its poles is within reach of the column's
/// ends — `repair_pole_network_with_keepout` closes any remaining gap
/// with short hops that stay out of the rig footprints.
fn stamp_bridge_columns(
    layout: &mut LayoutResult,
    strip_rects: &[crate::models::StripRect],
    keepout: &FxHashSet<(i32, i32)>,
) -> usize {
    let mut added = 0usize;
    for pair in strip_rects.windows(2) {
        let (above, below) = (&pair[0], &pair[1]);
        let (top_row, bottom_row) = (above.y + above.height, below.y - 1);
        if bottom_row < top_row {
            continue;
        }
        let width = above.width.max(below.width);
        let centre = width / 2;
        let free_rows = |x: i32| (top_row..=bottom_row).filter(|&y| !keepout.contains(&(x, y))).count();
        let x_b = (0..width)
            .max_by_key(|&x| (free_rows(x), -(x - centre).abs()))
            .unwrap_or(centre);
        // Chain down the free rows: from the first free row, each next pole
        // is the lowest free row within reach of the previous one.
        let mut y = (top_row..=bottom_row).find(|&y| !keepout.contains(&(x_b, y)));
        while let Some(py) = y {
            layout.entities.push(PlacedEntity {
                name: "medium-electric-pole".into(),
                x: x_b,
                y: py,
                direction: EntityDirection::North,
                segment_id: Some("grid:pole-bridge".into()),
                ..Default::default()
            });
            added += 1;
            y = ((py + 1)..=(py + 8).min(bottom_row))
                .rev()
                .find(|&ny| !keepout.contains(&(x_b, ny)));
        }
    }
    added
}

/// RFC-072 Phase 2 unit 2 — the K_MAX successor: K > K_MAX quantized
/// copies compose as a GRID of vertically stacked, fully independent
/// strips. Every strip carries its own per-copy feeds and drains (the
/// interface contract the sim receipts measure), so the composed
/// artifact needs NO inter-strip flow: the only shared geometry is the
/// pole bridge `repair_pole_network` stamps across the clearance —
/// K72-4 (no routing across cell boundaries) holds by construction.
/// Design adjudication, refuted alternatives (through-columns, interior
/// merges), and the wall receipts: the RFC's decision log, 2026-08-26.
fn compose_grid_with_capacity(
    sr: &SolverResult,
    inserter_capacity: u8,
    k: i32,
) -> Result<LayoutResult, String> {
    chain_eligible_at(sr, inserter_capacity)?; // the K_MAX * R_MAX refusal at the composing level
    // Belt-and-braces: `k` is the caller's count at its declared level,
    // the eligibility check above re-derives it at the same level; keep
    // the explicit bound so a caller passing a stale K still refuses
    // with the same wording contract.
    if k > K_MAX * R_MAX {
        return Err(format!(
            "cells: chain needs {k} quantized copies (max {} = {R_MAX} strips x {K_MAX} at quantum {QUANTUM_RATE}/s)",
            K_MAX * R_MAX
        ));
    }
    let strips = (k + K_MAX - 1) / K_MAX;
    // Balanced split: strip copy counts differ by at most one, summing
    // to K (18 -> 9+9, 25 -> 9+8+8).
    let base = k / strips;
    let extra = (k % strips) as usize;
    let mut copies_per_strip = Vec::with_capacity(strips as usize);
    let mut strip_rects: Vec<crate::models::StripRect> = Vec::with_capacity(strips as usize);
    let mut composed: Option<LayoutResult> = None;
    let mut y_off = 0i32;
    for s in 0..strips as usize {
        let k_s = base + if s < extra { 1 } else { 0 };
        copies_per_strip.push(k_s);
        let ratio = k_s as f64 / k as f64;
        // A strip is the SAME chain at k_s/K of the flow: scale every
        // machine count and external total; per-machine rates (and so
        // per-copy cell geometry) are untouched, which is what makes
        // every strip's cells identical to the single-strip case.
        let mut sub = sr.clone();
        for m in sub.machines.iter_mut() {
            m.count *= ratio;
        }
        for f in sub
            .external_inputs
            .iter_mut()
            .chain(sub.external_outputs.iter_mut())
            .chain(sub.surplus_outputs.iter_mut())
        {
            f.rate *= ratio;
        }
        // The strip composes at the PLANNED count k_s (not its own
        // re-derived quantization: the scaled sub-result's rate-only
        // quantum can be lower than the grid's margin-bumped share, and
        // the per-copy flow must be the grid's, not the strip's).
        let strip = compose_strip_with_capacity(&sub, inserter_capacity, k_s)?;
        // The strip's own receipt is one rect at its origin; in the grid
        // frame it sits at `y_off` (strip 0 at 0).
        strip_rects.push(crate::models::StripRect {
            x: 0,
            y: if composed.is_none() { 0 } else { y_off },
            width: strip.width,
            height: strip.height,
            copies: k_s,
        });
        composed = Some(match composed {
            None => {
                y_off = strip.height + STRIP_CLEARANCE;
                strip
            }
            Some(mut acc) => {
                let strip_h = strip.height;
                append_strip_translated(&mut acc, strip, y_off);
                y_off += strip_h + STRIP_CLEARANCE;
                acc
            }
        });
    }
    let mut composed = composed.expect("grid path implies K > K_MAX implies strips >= 2");
    // One power network: the strips' own pole lines are islands until
    // bridged. `repair_pole_network` adds bridge poles across the
    // clearance and recomputes the stored wire graph for the combined
    // entity list (per-strip graphs were dropped in the merge — their
    // indices are strip-local).
    //
    // The clearance is not free space to the sim harness: every interior
    // boundary head (a strip's top-edge feeds, a strip's bottom-edge
    // drains) grows a rig into it, and a bridge pole inside a rig's lane
    // is a silent feed failure in-game — the harness refuses such a
    // fixture at pre-flight (RFC-075: the K=20 ec@240 grid's bridge at
    // x=467 sat ten tiles from a copper-ore head at x=477). Try the plain
    // repair first so every grid whose bridges never touched a lane keeps
    // its receipted geometry byte-identical; only when a bridge lands in a
    // lane, stamp an explicit bridge column at the x farthest from every
    // interior head in each gap and repair again with the lanes kept out.
    let lanes = interior_rig_lanes(&strip_rects, &composed);
    let is_pole = |e: &PlacedEntity| e.name.ends_with("electric-pole");
    let poles_before: FxHashSet<(i32, i32)> =
        composed.entities.iter().filter(|e| is_pole(e)).map(|e| (e.x, e.y)).collect();
    let mut trial = composed.clone();
    let plain_bridges = crate::bus::layout::repair_pole_network(&mut trial);
    let collides = trial
        .entities
        .iter()
        .any(|e| is_pole(e) && !poles_before.contains(&(e.x, e.y)) && lanes.contains(&(e.x, e.y)));
    let pole_bridges = if collides {
        let columns = stamp_bridge_columns(&mut composed, &strip_rects, &lanes);
        crate::trace::emit(crate::trace::TraceEvent::CellGridBridgeRerouted { columns });
        columns + crate::bus::layout::repair_pole_network_with_keepout(&mut composed, &lanes)
    } else {
        composed = trial;
        plain_bridges
    };
    crate::trace::emit(crate::trace::TraceEvent::CellGridComposed {
        copies_per_strip: copies_per_strip.clone(),
        clearance: STRIP_CLEARANCE,
        pole_bridges,
    });
    // The typed receipt (RFC-074 Unit 1): the strips' own single-rect
    // receipts were merged away by `append_strip_translated`; the grid
    // states its shape here. Verification is attached by the selection
    // candidate, which is where the registry is consulted.
    composed.composition = Some(crate::models::CompositionReceipt {
        kind: "cell-grid".to_string(),
        copies_per_strip,
        strips: strip_rects,
        ..Default::default()
    });
    Ok(composed)
}

/// Append `strip` to `acc`, translated down by `dy`. Everything with a
/// coordinate translates — entities, BOTH boundary record sets (the
/// harness attaches rigs at these exact tiles), surplus exits, regions;
/// `power_wires` is dropped (strip-local indices) and rebuilt by the
/// caller's `repair_pole_network`. `effective_rows` and
/// `research_productivity` are NOT merged: the strip composer never
/// populates either today (cells attribute rows by belt adjacency,
/// validate/inserters.rs), so a strip composer that starts to must
/// extend this merge — the debug asserts below are the tripwire for
/// the declared axes.
fn append_strip_translated(acc: &mut LayoutResult, strip: LayoutResult, dy: i32) {
    debug_assert_eq!(acc.inserter_capacity, strip.inserter_capacity);
    debug_assert_eq!(acc.stacking, strip.stacking);
    // The two un-merged fields must actually be empty on both sides —
    // the tripwire the doc comment promises (#733 round 2): a strip
    // composer that starts populating either would otherwise lose the
    // upper strips' contribution silently.
    // Hard asserts, not debug: the "refuse silently" promise must hold
    // in release/WASM builds too (#733 round 6).
    assert!(
        strip.effective_rows.is_empty() && acc.effective_rows.is_empty(),
        "a strip populated effective_rows — extend append_strip_translated to merge (translate y) them"
    );
    assert!(
        strip.research_productivity.is_empty() && acc.research_productivity.is_empty(),
        "a strip declared research_productivity — extend append_strip_translated to merge it"
    );
    for mut e in strip.entities {
        e.y += dy;
        acc.entities.push(e);
    }
    for mut b in strip.boundary_inputs {
        b.y += dy;
        acc.boundary_inputs.push(b);
    }
    for mut b in strip.boundary_outputs {
        b.y += dy;
        acc.boundary_outputs.push(b);
    }
    for (item, x, y) in strip.surplus_exits {
        acc.surplus_exits.push((item, x, y + dy));
    }
    // Regions carry absolute port coordinates inside `RegionPort`, which
    // a bare `r.y += dy` would leave untranslated (#733 round 4). The
    // strip composer never emits regions today; refuse to merge them
    // silently rather than half-translate them.
    assert!(
        strip.regions.is_empty() && acc.regions.is_empty(),
        "a strip emitted regions — extend append_strip_translated to translate their ports too"
    );
    acc.voided_streams.extend(strip.voided_streams);
    acc.warnings.extend(strip.warnings);
    acc.width = acc.width.max(strip.width);
    acc.height = dy + strip.height;
    acc.power_wires = None;
}
