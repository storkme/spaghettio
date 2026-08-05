//! Functional blueprint validation.
//!
//! Port of `src/validate.py` — foundation types and top-level `validate()` dispatcher.

pub mod belt_detour;
pub mod belt_flow;
pub mod inserters;
mod fluids;
pub mod modules;
pub mod power;
pub mod sushi;

pub use fluids::{
    check_fluid_network_connectivity, check_fluid_port_connectivity, check_pipe_isolation,
};

// Fluid-port classification accessor for the `common` drift regression
// (RFC `docs/rfc-power-supply.md` Phase 0b/0e-i). Test-only re-export; the
// tables now live in the shared `crate::fluid_ports` module.
#[cfg(test)]
pub(crate) use crate::fluid_ports::machine_has_fluid_ports;

pub mod belt_structural;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::{LayoutResult, MachineSpec, RegionKind, SolverResult};
use power::{check_pole_network_connectivity, check_power_coverage};
use rustc_hash::FxHashSet;

use belt_flow::{
    check_belt_connectivity, check_belt_flow_path,
    check_belt_flow_reachability, check_belt_junctions, check_belt_network_topology,
    check_input_rate_delivery, check_underground_belt_entry_sideload,
    check_underground_belt_pairs, check_underground_belt_sideloading,
};

/// Layout style: affects which validation checks run and how.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutStyle {
    /// Constraint-based spaghetti layout (default).
    #[default]
    Spaghetti,
    /// Deterministic row-based main-bus layout.
    Bus,
}

/// Severity level of a single validation finding.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// Machine-readable payload for rate-shaped issues. The prose `message`
/// already states these numbers; carrying them structurally lets the web
/// UI compute severity ratios (starvation heatmap) without parsing text.
/// See `docs/rfc-validation-explainability.md` (D1). Cause attribution
/// deliberately does NOT live here — causes are stamp-time facts and ride
/// the trace-event side (D2).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueDetail {
    /// Rate actually delivered/moved, in items per second.
    pub delivered: f64,
    /// Rate the machine needs at the compared boundary (per-inserter for
    /// input-rate-delivery, per-item totals for inserter-item-throughput —
    /// always the pair the check itself compared).
    pub needed: f64,
}

/// A single validation finding, mirroring Python's `ValidationIssue` dataclass.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    /// Category tag, e.g. `"pipe-isolation"`, `"fluid-connectivity"`, `"inserter"`, `"power"`.
    pub category: String,
    pub message: String,
    /// Optional grid position associated with the issue.
    pub x: Option<i32>,
    pub y: Option<i32>,
    /// Structured numbers for rate-shaped issues; `None` for all others.
    /// `serde(default)` keeps old `.fls` snapshots readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<IssueDetail>,
}

impl ValidationIssue {
    /// Construct a new issue without an associated position.
    pub fn new(severity: Severity, category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            category: category.into(),
            message: message.into(),
            x: None,
            y: None,
            detail: None,
        }
    }

    /// Construct a new issue with an associated grid position.
    pub fn with_pos(
        severity: Severity,
        category: impl Into<String>,
        message: impl Into<String>,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            severity,
            category: category.into(),
            message: message.into(),
            x: Some(x),
            y: Some(y),
            detail: None,
        }
    }

    /// Attach structured delivered/needed rates (builder style). The pair
    /// must be exactly what the emitting check compared — not re-derived.
    pub fn with_detail(mut self, delivered: f64, needed: f64) -> Self {
        self.detail = Some(IssueDetail { delivered, needed });
        self
    }
}

/// Raised when critical validation issues block blueprint generation.
///
/// `issues` contains the full list — both errors and warnings — so callers
/// that want a complete picture (e.g. scoreboards) don't lose warning
/// counts when an error is also present. The `Display` impl only renders
/// the error subset to keep the "Validation failed" message focused on
/// what actually blocked generation.
#[derive(Debug, Error)]
#[error("Validation failed:\n{}", format_errors(.issues))]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationError {
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }
}

fn format_errors(issues: &[ValidationIssue]) -> String {
    issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("  [{}] {}", i.severity.as_str(), i.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Tile set covered by `RegionKind::Unresolved` regions in the layout.
/// These come from clusters where the ghost-router junction solver
/// gave up (`JunctionGrowthCapped`); the speculatively-routed ghost
/// belts inside are orphans, not real layout features. Validators that
/// flag belt-to-belt adjacency consult this set so they don't pile
/// follow-on errors onto a single underlying junction failure.
pub fn unresolved_region_tiles(layout: &LayoutResult) -> FxHashSet<(i32, i32)> {
    let mut tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    for r in &layout.regions {
        if r.kind != RegionKind::Unresolved {
            continue;
        }
        for dx in 0..r.width {
            for dy in 0..r.height {
                tiles.insert((r.x + dx, r.y + dy));
            }
        }
    }
    tiles
}

/// Whether a splitter output branch is a *priority branch* — one whose
/// downstream belt is tagged either as a self-loop recirculation
/// (`:selfloop:`, the kovarex / voider precedent) or as a merge-and-tap
/// priority tap ([`crate::common::MERGE_TAP_SEGMENT_TAG`], RFC
/// `docs/rfc-merge-tap-trunks.md` D4). Both walkers route a priority branch
/// with the `loop_priority_rate` min(total, cap) law instead of the generic
/// even split; the two tags share that one rate law and differ only in the
/// demand source (the loop's recirculation rate vs the tap's consumer
/// demand). `seg` is the segment id of the belt tile immediately downstream
/// of the splitter output tile.
///
/// Voider rows recirculate without a splitter, so they never reach this
/// predicate; only `:selfloop:` and the tap tag identify a splitter branch.
/// Warning count for CANDIDATE SELECTION (decomposition ranking, the
/// never-worse channel contracts, the refusal tier). Excludes categories
/// whose model is not yet calibrated enough to steer selection:
/// `input-rate-delivery` gained real teeth from the #519 consumption
/// decrement — honest REPORTING — but letting the new counts re-rank
/// candidates flipped winners on configs where the audit then caught the
/// new winner over-stamping physical capacity (stacking_ec_60s: a
/// reporting recalibration must not silently change which layout ships).
/// Lift the exemption deliberately once the flux model is sim-anchored
/// (#519 follow-up), with the fixture drift adjudicated case by case.
///
/// `belt-detour` (2026-08-01) is excluded for the identical reason on
/// first principles, not just precedent: it is a brand-new category with
/// no prior data point at zero, so including it by default would flip
/// EVERY close candidate-selection decision across the corpus the moment
/// it started firing — confirmed empirically when wiring it in: several
/// e2e fixtures (`tier2_electronic_circuit_20s_from_ore`'s golden hash,
/// `tier4_advanced_circuit_from_ore_am2`'s `input-rate-delivery` count)
/// changed their SELECTED layout, not just their warning list, purely
/// because `belt-detour`'s mere presence nudged `decomposition_search`'s
/// ranking — with no sim evidence either candidate is actually worse. Its
/// thresholds are corpus-survey-calibrated (see the check's own doc
/// comment), not physically-grounded the way an error is, so it stays
/// report-only until a similar sim-anchoring case is made for letting it
/// steer selection.
pub(crate) fn selection_warning_count(issues: &[ValidationIssue]) -> usize {
    issues
        .iter()
        .filter(|i| {
            i.severity == Severity::Warning
                && i.category != "input-rate-delivery"
                && i.category != "belt-detour"
        })
        .count()
}

pub(crate) fn segment_is_priority_branch(seg: Option<&str>) -> bool {
    seg.is_some_and(|s| {
        s.contains(":selfloop:") || s.contains(crate::common::MERGE_TAP_SEGMENT_TAG)
    })
}

/// A direct-insertion bridge inserter (RFC decomposition-search Phase 3):
/// belt-to-belt by design, it lifts the DI'd item off the producer's output
/// belt and drops it onto the consumer's input belt, touching no machine.
/// The placer tags these `di-bridge:<item>:<recipe>` (see
/// `bus::placer::stamp_di_bridge`); the direction / delivery / reachability
/// checks recognize them so belt-to-belt DI is not mistaken for a
/// misdirected inserter, an unfed input, or an unreachable dead-end.
pub(crate) fn is_di_bridge_inserter(seg: Option<&str>) -> bool {
    seg.is_some_and(|s| s.starts_with("di-bridge:"))
}

/// A direct-insertion **cell** inserter (RFC-053 Phase 1): machine-to-machine
/// by design. The placer tags every entity of a fused cell
/// `di-cell:<item>:<consumer_recipe>` (see `bus::di_cell::stamp_di_cell_io`).
///
/// The distinction that matters to the checks below: a cell's PRODUCER has
/// **no output belt at all** — its output leaves through the one-tile band
/// straight into the consumer machine. Every "machine must have an output
/// inserter dropping onto a belt" test therefore flags a cell producer as a
/// hard error unless it recognises this tag, and the item-throughput tests
/// see 0.00/s moved because they only credit belt-bound hands. Those are
/// false alarms, not defects — `bus::di_cell`'s own invariants (every band
/// inserter picks from a producer tile and drops into a consumer tile, at
/// reach 1, with no belt for the coupled item) own the cell's correctness,
/// and RFC-053 KC3 sim-measured the shape at 112% of plan.
pub(crate) fn is_di_cell_entity(seg: Option<&str>) -> bool {
    // `di-cell:` is the Phase 1 stacked cell, `di-row:` the Phase 2
    // horizontal row cell. Both couple machine-to-machine and both leave
    // their producers without a belt-bound output hand, so both need the
    // same exemptions.
    seg.is_some_and(|s| s.starts_with("di-cell:") || s.starts_with("di-row:"))
}

/// Resolve the exact `MachineSpec` sibling the layout pipeline placed at `y`
/// for `recipe`, preferring `layout.effective_rows`'s position attribution
/// over a recipe-name lookup — partition siblings share a recipe name but
/// carry different utilizations, and a recipe-keyed lookup collapses them
/// to whichever sibling iterated last (`docs/rfc-inserter-sizing.md` Phase 1
/// finding). Falls back to `fallback_spec` when no row attribution is
/// available (hand-built `LayoutResult`s in tests, or spaghetti-style
/// layouts that never populate `effective_rows`) — a byte-for-byte no-op
/// wherever partitioning never occurred. Shared across every `validate::`
/// check that resolves a machine's spec by recipe name — see
/// `belt_flow::compute_lane_rates_impl`, `belt_flow::check_input_rate_delivery`,
/// `belt_structural::compute_lane_rates`, `inserters::check_inserter_throughput`,
/// and `inserters::check_inserter_item_throughput`.
pub(crate) fn resolve_row_spec<'a>(
    layout: &'a LayoutResult,
    recipe: &str,
    y: i32,
    fallback_spec: &'a MachineSpec,
) -> &'a MachineSpec {
    resolve_row_spec_banded(layout, recipe, y, fallback_spec).0
}

/// Like [`resolve_row_spec`] but also returns the matched row's `[y_start,
/// y_end)` band, `None` when the fallback (recipe-global) spec applied.
/// The band is the spec's SCOPE: a per-machine utilization derived from
/// physically-placed machines must count within it (per-row for partition
/// siblings, layout-wide for the global fallback) — see
/// `belt_flow::physical_utilization` (#519 fallout: chain/mega replication
/// places ceil-per-copy machines, so `ceil(spec.count)` understates the
/// physical count and `utilization_for` over-states per-machine rates).
pub(crate) fn resolve_row_spec_banded<'a>(
    layout: &'a LayoutResult,
    recipe: &str,
    y: i32,
    fallback_spec: &'a MachineSpec,
) -> (&'a MachineSpec, Option<(i32, i32)>) {
    layout
        .effective_rows
        .iter()
        .find(|row| row.spec.recipe == recipe && y >= row.y_start && y < row.y_end)
        .map(|row| (&row.spec, Some((row.y_start, row.y_end))))
        .unwrap_or((fallback_spec, None))
}

/// Emits one error per connected component of unresolved tiles. The
/// ghost router emits an `Unresolved` region per individual tile, so a
/// single failed junction often appears as a cluster of 1×1 regions —
/// emitting one error per region inflated counts (a 10-tile failed
/// crossing counted as 10 errors). This BFS-coalesces adjacent
/// unresolved tiles so each underlying junction failure surfaces as
/// one error. Region-tiles inside the cluster are still excluded from
/// `belt-item-isolation` so orphan ghosts don't pile follow-on noise on
/// top.
/// Byproduct flows the solver could not credit against any demand and the
/// layout cannot yet route anywhere (`SolverResult::surplus_outputs`). Until
/// Phase 2 of docs/rfc-solver-net-flow.md lands surplus-to-perimeter
/// routing, every such flow is a machine output port that physically backs
/// up in-game: the producing machine stalls once its internal buffer fills.
/// Reported as an **error** (not a warning) by explicit decision — this is
/// exactly the "validator-clean but game-dead" class the net-flow RFC
/// exists to eliminate, and it was previously invisible (the tree walk
/// dropped these flows on the floor; e.g. utility-science-pack's AOP
/// light-oil, stranded silently for as long as the chain has existed).
///
/// Solid surplus routing (RFC Fulgora D2a/D2b, docs/rfc-fulgora-scrap.md)
/// extends the same entity-cross-checked acceptance fluids already had —
/// the step-7 solid-surplus merger records an exit tile in
/// `LayoutResult::surplus_exits` the same way the fluid trunk router does.
/// A boundary record is a ledger entry; the entity at its tile is the fact.
///
/// `boundary_inputs` / `boundary_outputs` say where the factory expects to
/// receive and hand over goods. Everything downstream treats them as
/// authoritative — blueprint export, and the sim harness, which places its
/// source and collection chests at exactly these coordinates. Nothing
/// previously checked they still described reality.
///
/// That gap is not hypothetical. A layout transform relocated an output belt
/// to a new bounding-box edge and left the record on the old tile. The
/// geometry was flawless and validation passed with an issue profile
/// identical to the untransformed control; in Factorio the factory produced
/// **0.00/s**, because output arrived where nothing collected it and every
/// producer upstream backed up behind a full buffer.
///
/// `check_stranded_byproducts` already applies exactly this reasoning to
/// `surplus_exits` and `voided_streams` — "a ledger without the physical
/// entity is exactly the stalled-machine bug this check exists to catch". It
/// was never extended to the primary product, which is the one boundary the
/// factory exists to serve.
///
/// The assertion is deliberately narrow: the tile must hold a transport
/// entity of the right family carrying the right item. Facing is not checked,
/// because a boundary belt may legitimately run along an edge rather than
/// through it, and a false error here would be worse than the gap.
pub fn check_boundary_record_integrity(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let mut check = |record: &crate::models::BoundaryRecord, role: &str| {
        let matched = layout.entities.iter().any(|e| {
            e.x == record.x
                && e.y == record.y
                && if record.is_fluid {
                    e.name == "pipe" || e.name == "pipe-to-ground"
                } else {
                    crate::common::is_belt_entity(&e.name)
                }
        });
        if !matched {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                category: "boundary-record-integrity".to_string(),
                message: format!(
                    "{role} boundary for {} claims ({},{}), but no {} entity is there —                      goods would be handed over at a tile nothing reaches",
                    record.item,
                    record.x,
                    record.y,
                    if record.is_fluid { "pipe" } else { "belt" },
                ),
                x: Some(record.x),
                y: Some(record.y),
                detail: None,
            });
            return;
        }
        // Present, but carrying something else: the record and the geometry
        // disagree about what crosses here.
        let carries_ok = layout.entities.iter().any(|e| {
            e.x == record.x
                && e.y == record.y
                && e.carries.as_deref() == Some(record.item.as_str())
        });
        if !carries_ok {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                category: "boundary-record-integrity".to_string(),
                message: format!(
                    "{role} boundary for {} at ({},{}) sits on transport that does not                      carry {}",
                    record.item, record.x, record.y, record.item,
                ),
                x: Some(record.x),
                y: Some(record.y),
                detail: None,
            });
        }
    };

    for record in &layout.boundary_inputs {
        check(record, "Input");
    }
    for record in &layout.boundary_outputs {
        check(record, "Output");
    }
    issues
}

pub fn check_stranded_byproducts(
    layout: &LayoutResult,
    solver: &SolverResult,
) -> Vec<ValidationIssue> {
    // A surplus flow counts as routed only when BOTH hold: the router
    // recorded a perimeter/merge exit for the item (`LayoutResult::
    // surplus_exits` — first-class layout data, populated with or without
    // tracing) AND a matching physical entity carrying that item exists at
    // the recorded tile — a pipe/pipe-to-ground for fluids (perimeter
    // routing), a belt/underground-belt/splitter for solids (the step-7
    // merger cascade). The entity cross-check is deliberate — an exit
    // record alone is a ledger entry, and a ledger without the physical
    // entity is exactly the stalled-machine bug this check exists to catch.
    let is_routed = |f: &crate::models::ItemFlow| {
        layout.surplus_exits.iter().any(|(ei, ex, ey)| {
            ei == &f.item
                && layout.entities.iter().any(|e| {
                    e.x == *ex
                        && e.y == *ey
                        && e.carries.as_deref() == Some(f.item.as_str())
                        && if f.is_fluid {
                            e.name == "pipe" || e.name == "pipe-to-ground"
                        } else {
                            crate::common::is_belt_entity(&e.name)
                        }
                })
        })
    };

    // RFC Fulgora Phase 2 (docs/rfc-fulgora-scrap.md D1): under
    // `SurplusPolicy::Void`, a solid surplus item may instead be
    // consumed by a layout-synthesized voider recycler bank —
    // `LayoutResult::voided_streams`, recorded first-class and
    // trace-independent like `surplus_exits`. Verified PHYSICALLY, not
    // trusted alone: real `recycler` entities running the right
    // `<item>-recycling` recipe must actually be present, in at least
    // the recorded machine count — a ledger entry with no matching
    // hardware is exactly the stalled-machine bug this check exists to
    // catch, the same standard `is_routed` holds Export to.
    let is_voided = |f: &crate::models::ItemFlow| {
        layout.voided_streams.iter().any(|v| {
            v.item == f.item
                && layout
                    .entities
                    .iter()
                    .filter(|e| e.name == "recycler" && e.recipe.as_deref() == Some(v.recipe.as_str()))
                    .count()
                    >= v.machines
        })
    };

    solver
        .surplus_outputs
        .iter()
        .filter(|f| !is_routed(f) && !is_voided(f))
        .map(|f| {
            ValidationIssue::new(
                Severity::Error,
                "stranded-byproduct",
                format!(
                    "byproduct {} ({:.3}/s) has no consumer and no route out of the \
                     layout — the producing machine will stall in-game once its \
                     output buffer fills (workaround: consume it downstream, \
                     supply the loop item externally, or route it to the \
                     perimeter/merger)",
                    f.item, f.rate
                ),
            )
        })
        .collect()
}

/// RFC-062 Phase 2: a "shared row" produces an item with more than one
/// live physical claim on it at once — some combination of an external
/// target export, an internal consumer's tap-off, and a surplus export —
/// where two independent mechanisms (`ghost_router` Step 7's row-level
/// export merge and `lane_planner`'s dual-purpose-lane tap-offs) each
/// place a claim on the same producer row/lane. Nothing before this check
/// cross-validated those claims against what the row actually builds.
///
/// Rates are all solver-side, using the CEILED per-`MachineSpec` machine
/// count — the same rounding `bus::placer::place_rows` applies turning
/// the LP's continuous count into a physical row, so `production` here
/// matches what actually gets built, not the LP's (possibly fractional)
/// demand.
///
/// Two failure directions, both positioned at a real producer-machine
/// entity for the item (rule 4, `docs/validator-reporting.md`: a solver
/// row is a claim, the entity at its tile is the fact):
///
/// - **Over-claim** (`shared-row-outflow-overclaim`, the RFC's primary
///   ask): target rate + tap demand + surplus rate exceeds what the row
///   actually produces — the row cannot honor every claim and something
///   downstream starves. The main invariant `docs/rfc-062-multi-target-
///   outputs.md`'s Layout section calls for.
///
///   **Severity split, added after review (RFC-062 Phase 2 fix round):**
///   naively comparing BUILT numbers alone (every machine count
///   independently ceiled — the same rounding `bus::placer::place_rows`
///   applies) hard-errors on legitimate ceil-slack contention: measured
///   on EC@10.4/s + AC@3.05/s, claimed (10.4 target + 6.25 built taps) =
///   16.65 > produced (built) 16.5, the sole issue on an otherwise-clean
///   layout. The row's PRODUCER count and the tap's CONSUMER count are
///   two INDEPENDENT `MachineSpec`s, each ceiled on its own — nothing
///   guarantees their slack cancels, so this is not a bug, it's built-
///   capacity contention between two roundings that both individually
///   round up. Fix: also compute the same totals from the LP's RAW
///   (un-ceiled) `MachineSpec::count` — the row-conservation identity
///   the net-flow LP solves guarantees these balance (`target_plan +
///   taps_plan + surplus_plan ≈ production_plan`) UNLESS something is
///   genuinely wrong at the plan level, not just the build level. Only
///   a PLAN-level overclaim is `Severity::Error`; a BUILT-only overclaim
///   (plan balances, built doesn't) is `Severity::Warning` with a
///   distinct message — still surfaced (this row has zero headroom for
///   its exact built machine-count pairing, worth knowing), just not a
///   build-blocking claim of a starved consumer.
/// - **Under-claim** (`shared-row-outflow-underclaim`): the item has a
///   target rate AND at least one other live claim (tap or surplus), but
///   no PHYSICAL export record (`boundary_outputs`/`surplus_exits`,
///   entity-cross-checked, matching `check_stranded_byproducts`'s and
///   `check_boundary_record_integrity`'s standard) exists for it at all.
///   Added after RFC-062 Phase 0's own probe observed exactly this
///   silent failure under the cell-composed candidate — EC's target
///   export dropped from `boundary_outputs` entirely, zero validator
///   errors raised. See the Phase 2 decision log for why this direction
///   was added rather than left as a documented gap. NOTE (review
///   finding, recorded here so it isn't mistaken for a stronger claim
///   later): "physically exported" here means a ledger record backed by
///   a real entity carrying the right item AT the recorded tile — it is
///   NOT a flow/rate measurement. A belt can satisfy this check while
///   under-delivering (or, per `docs/validator-reporting.md`'s own
///   history, being disconnected one tile further along). The sim
///   harness (Phase 3/4) remains the real bar for "the export actually
///   moves the claimed rate in-game" — this check only closes the
///   "nothing there at all" gap.
pub fn check_shared_row_outflow_conservation(
    layout: &LayoutResult,
    solver: &SolverResult,
) -> Vec<ValidationIssue> {
    use rustc_hash::FxHashMap;

    let ceiled = |count: f64| if count > 0.0 { count.ceil().max(1.0) } else { 0.0 };
    let planned = |count: f64| count.max(0.0);

    let mut target_rate: FxHashMap<&str, f64> = FxHashMap::default();
    for ext in &solver.external_outputs {
        if !ext.is_fluid {
            *target_rate.entry(ext.item.as_str()).or_insert(0.0) += ext.rate;
        }
    }
    let mut surplus_rate: FxHashMap<&str, f64> = FxHashMap::default();
    for sur in &solver.surplus_outputs {
        if !sur.is_fluid {
            *surplus_rate.entry(sur.item.as_str()).or_insert(0.0) += sur.rate;
        }
    }
    // BUILT (ceiled per-`MachineSpec` count — what actually gets placed)
    // and PLAN (the LP's raw, un-ceiled count) production/tap totals,
    // side by side — the severity split above needs both.
    let mut production_built: FxHashMap<&str, f64> = FxHashMap::default();
    let mut production_plan: FxHashMap<&str, f64> = FxHashMap::default();
    let mut producer_recipes: FxHashMap<&str, Vec<&str>> = FxHashMap::default();
    for m in &solver.machines {
        let built = ceiled(m.count);
        let plan = planned(m.count);
        for out in &m.outputs {
            if out.is_fluid {
                continue;
            }
            *production_built.entry(out.item.as_str()).or_insert(0.0) += out.rate * built;
            *production_plan.entry(out.item.as_str()).or_insert(0.0) += out.rate * plan;
            producer_recipes.entry(out.item.as_str()).or_default().push(m.recipe.as_str());
        }
    }
    let mut tap_demand_built: FxHashMap<&str, f64> = FxHashMap::default();
    let mut tap_demand_plan: FxHashMap<&str, f64> = FxHashMap::default();
    for m in &solver.machines {
        // A voider's draw is bus-invisible by design (see the matching
        // exclusion in `bus::placer::place_rows`'s `internally_consumed_
        // items` and `lane_planner::plan_bus_lanes`'s `item_to_consumers`)
        // — it never competes with a target export for the same lane.
        if m.voider {
            continue;
        }
        let built = ceiled(m.count);
        let plan = planned(m.count);
        for inp in &m.inputs {
            if inp.is_fluid {
                continue;
            }
            *tap_demand_built.entry(inp.item.as_str()).or_insert(0.0) += inp.rate * built;
            *tap_demand_plan.entry(inp.item.as_str()).or_insert(0.0) += inp.rate * plan;
        }
    }

    const EPS: f64 = 1e-6;
    let mut issues = Vec::new();
    for (&item, &prod_built) in &production_built {
        let target = target_rate.get(item).copied().unwrap_or(0.0);
        let taps_built = tap_demand_built.get(item).copied().unwrap_or(0.0);
        let surplus = surplus_rate.get(item).copied().unwrap_or(0.0);

        // "Shared" means at least two of {target, taps, surplus} are
        // live — a pure export, a pure intermediate, or a pure byproduct
        // alone never collides; the solver's own conservation already
        // guarantees those are correct.
        let live_claims = (target > EPS) as u8 + (taps_built > EPS) as u8 + (surplus > EPS) as u8;
        if live_claims < 2 {
            continue;
        }

        let producer_pos = producer_recipes.get(item).and_then(|recipes| {
            layout
                .entities
                .iter()
                .find(|e| e.recipe.as_deref().map(|r| recipes.contains(&r)).unwrap_or(false))
        });
        let (px, py) = producer_pos.map(|e| (e.x, e.y)).unwrap_or((0, 0));

        let claimed_built = target + taps_built + surplus;
        if claimed_built > prod_built + EPS {
            let prod_plan = production_plan.get(item).copied().unwrap_or(0.0);
            let taps_plan = tap_demand_plan.get(item).copied().unwrap_or(0.0);
            // `target`/`surplus` are already plan-precision (the solver
            // never ceils a target rate or a surplus remainder) — only
            // the tap side needs a separate plan-level total.
            let claimed_plan = target + taps_plan + surplus;
            if claimed_plan > prod_plan + EPS {
                issues.push(
                    ValidationIssue::with_pos(
                        Severity::Error,
                        "shared-row-outflow-overclaim",
                        format!(
                            "{item}: shared row claims {claimed_built:.3}/s ({target:.3}/s \
                             export + {taps_built:.3}/s internal taps + {surplus:.3}/s \
                             surplus) but only produces {prod_built:.3}/s — the plan-level \
                             totals also overclaim ({claimed_plan:.3}/s claimed vs \
                             {prod_plan:.3}/s planned production), so this is a genuine \
                             demand overrun, not ceiling slack — the export or an internal \
                             consumer will be starved in-game",
                        ),
                        px,
                        py,
                    )
                    .with_detail(prod_built, claimed_built),
                );
            } else {
                issues.push(
                    ValidationIssue::with_pos(
                        Severity::Warning,
                        "shared-row-outflow-overclaim",
                        format!(
                            "{item}: shared row's BUILT claims ({claimed_built:.3}/s = \
                             {target:.3}/s export + {taps_built:.3}/s internal taps + \
                             {surplus:.3}/s surplus) exceed its BUILT production \
                             ({prod_built:.3}/s), but the underlying PLAN rates balance \
                             ({claimed_plan:.3}/s claimed vs {prod_plan:.3}/s produced) — \
                             this row has zero headroom for its exact machine-count \
                             pairing (two independently-ceiled `MachineSpec`s whose slack \
                             doesn't cancel), not a genuine demand overrun",
                        ),
                        px,
                        py,
                    )
                    .with_detail(prod_built, claimed_built),
                );
            }
        }

        if target > EPS {
            let physically_exported = layout.boundary_outputs.iter().any(|r| {
                r.item == item
                    && layout.entities.iter().any(|e| {
                        e.x == r.x
                            && e.y == r.y
                            && e.carries.as_deref() == Some(item)
                            && crate::common::is_belt_entity(&e.name)
                    })
            }) || layout.surplus_exits.iter().any(|(ei, ex, ey)| {
                ei == item
                    && layout.entities.iter().any(|e| {
                        e.x == *ex
                            && e.y == *ey
                            && e.carries.as_deref() == Some(item)
                            && crate::common::is_belt_entity(&e.name)
                    })
            });
            if !physically_exported {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "shared-row-outflow-underclaim",
                    format!(
                        "{item}: target export ({target:.3}/s) has no physical export \
                         record (boundary_outputs/surplus_exits) backed by a real entity \
                         — the row also has {taps_built:.3}/s of internal taps and \
                         {surplus:.3}/s of surplus, so a dual-purpose-lane mechanism was \
                         expected but the export path was silently dropped",
                    ),
                    px,
                    py,
                ));
            }
        }
    }
    issues
}

pub fn check_unresolved_junctions(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let tiles = unresolved_region_tiles(layout);
    if tiles.is_empty() {
        return Vec::new();
    }
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut components: Vec<((i32, i32), usize)> = Vec::new();
    for &start in &tiles {
        if visited.contains(&start) {
            continue;
        }
        let mut queue = vec![start];
        let mut size = 0usize;
        let mut anchor = start;
        while let Some(t) = queue.pop() {
            if !visited.insert(t) {
                continue;
            }
            size += 1;
            if t < anchor {
                anchor = t;
            }
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let n = (t.0 + dx, t.1 + dy);
                if tiles.contains(&n) && !visited.contains(&n) {
                    queue.push(n);
                }
            }
        }
        components.push((anchor, size));
    }
    components.sort();
    components
        .into_iter()
        .map(|((x, y), size)| {
            ValidationIssue::with_pos(
                Severity::Error,
                "unresolved-junction",
                format!(
                    "Junction solver could not resolve a crossing near ({},{}) \
                     covering {} tile{}. Orphan ghost belts in this cluster are \
                     excluded from belt-adjacency checks.",
                    x,
                    y,
                    size,
                    if size == 1 { "" } else { "s" },
                ),
                x,
                y,
            )
        })
        .collect()
}

/// Surface "balancer template missing" as a warning per affected family.
///
/// Background: when `stamp_family_balancer` finds neither a direct
/// `(n, m)` template nor a gcd-decomposable `(n/g, m/g)` template, it
/// returns an empty entity vec and the producer→trunk handoff is silently
/// dropped. The downstream symptom is dead-end belts at the row's exit
/// column (see PU@3/s ore red copper-plate (4, 9) — issue #136 / PR #257).
///
/// `BalancerStamped { template_found: false }` trace events flag exactly
/// this case. Read them off `layout.trace` and emit a warning per shape so
/// users see "missing balancer template (4, 9) for copper-plate" instead
/// of having to chase the dead-end belts back to their cause.
///
/// Warning, not Error — the layout is still rendered (with broken
/// connectivity), and Pool fallback can sometimes produce a valid
/// alternative. The downstream belt-dead-end errors fire too if connectivity
/// is genuinely broken.
pub fn check_balancer_template_coverage(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let Some(trace) = layout.trace.as_ref() else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    for ev in trace {
        if let crate::trace::TraceEvent::BalancerStamped {
            item, shape, template_found, ..
        } = ev
        {
            if !*template_found {
                issues.push(ValidationIssue::new(
                    Severity::Warning,
                    "missing-balancer-template",
                    format!(
                        "no balancer template for shape ({}, {}) for item {item}; \
                         producer→trunk handoff dropped (downstream belts will dead-end)",
                        shape.0, shape.1,
                    ),
                ));
            }
        }
    }
    issues
}

/// Count "No N→M balancer template for X" warnings on a layout.
///
/// These warnings are emitted inline by `bus::layout::layout_pass` when a
/// `LaneFamily`'s `(n, m)` shape has no direct template AND no gcd-
/// decomposition path. Cheap proxy used by the decomposition-search
/// hard-constraint check (`docs/rfc-decomposition-search.md`) — avoids
/// running the full validator just to spot unstampable shapes.
///
/// Reads `LayoutResult.warnings` directly (no trace dependency, unlike
/// `check_balancer_template_coverage`).
pub fn count_missing_balancer_template_warnings(layout: &LayoutResult) -> usize {
    layout
        .warnings
        .iter()
        .filter(|w| w.contains("balancer template"))
        .count()
}

/// Run all functional validation checks on a layout.
///
/// Returns a list of issues found.  Returns `Err(ValidationError)` if any
/// error-severity issues are present.
pub fn validate(
    layout_result: &LayoutResult,
    solver_result: Option<&SolverResult>,
    layout_style: LayoutStyle,
) -> Result<Vec<ValidationIssue>, ValidationError> {
    use rayon::prelude::*;

    let layout = layout_result;
    let solver = solver_result;

    // Individual validation checks must NOT call `trace::emit` — the
    // trace collector is thread-local, so events raised from a rayon
    // worker thread would either panic (if the thread-local isn't
    // initialised there) or silently vanish. The only trace emit from
    // this function is the terminal `ValidationCompleted` below, which
    // runs on the caller's thread after `par_iter` collects. If you
    // ever need per-check tracing, gather the data into the returned
    // `ValidationIssue` list and emit once from here.
    let checks: Vec<Box<dyn Fn() -> Vec<ValidationIssue> + Send + Sync>> = vec![
        Box::new(|| check_power_coverage(layout)),
        Box::new(|| check_pole_network_connectivity(layout)),
        Box::new(|| inserters::check_inserter_chains(layout, solver)),
        Box::new(|| inserters::check_inserter_direction(layout)),
        Box::new(|| inserters::check_inserter_throughput(layout, solver)),
        Box::new(|| inserters::check_inserter_item_throughput(layout, solver)),
        Box::new(|| inserters::check_row_output_lane_budget(layout, solver)),
        Box::new(|| inserters::check_row_input_belt_margin(layout, solver)),
        Box::new(|| check_pipe_isolation(layout)),
        Box::new(|| check_fluid_port_connectivity(layout, layout_style)),
        Box::new(|| check_fluid_network_connectivity(layout, solver)),
        Box::new(|| check_belt_connectivity(layout, solver)),
        Box::new(|| check_belt_flow_path(layout, solver, layout_style)),
        Box::new(|| belt_structural::check_entity_overlaps(layout)),
        Box::new(|| belt_structural::check_belt_throughput(layout)),
        Box::new(|| belt_structural::check_output_belt_coverage(layout, solver)),
        Box::new(|| if layout_style == LayoutStyle::Spaghetti {
            check_belt_network_topology(layout, solver)
        } else {
            vec![]
        }),
        Box::new(|| check_belt_junctions(layout)),
        Box::new(|| check_underground_belt_pairs(layout)),
        Box::new(|| check_underground_belt_sideloading(layout)),
        Box::new(|| check_underground_belt_entry_sideload(layout)),
        Box::new(|| belt_structural::check_belt_dead_ends(layout)),
        Box::new(|| belt_structural::check_belt_loops(layout)),
        Box::new(|| belt_structural::check_tap_splitter_priority(layout)),
        Box::new(|| belt_structural::check_belt_item_isolation(layout)),
        Box::new(|| sushi::check_sushi_boundary(layout)),
        Box::new(|| {
            solver
                .map(|s| sushi::check_sushi_saturation(layout, s))
                .unwrap_or_default()
        }),
        Box::new(|| check_unresolved_junctions(layout)),
        Box::new(|| check_boundary_record_integrity(layout)),
        // RFC-065 Phase 1: record-vs-geometry integrity (effective_rows
        // bands, power_wires indices) joins the dispatch so every
        // validate() caller — including the compaction/fold admission
        // loops — guards the records automatically. Zero findings on the
        // green corpus by the Phase 0 parity gate.
        Box::new(|| crate::connectivity::check_record_integrity(layout)),
        Box::new(|| {
            solver
                .map(|s| check_stranded_byproducts(layout, s))
                .unwrap_or_default()
        }),
        Box::new(|| {
            solver
                .map(|s| check_shared_row_outflow_conservation(layout, s))
                .unwrap_or_default()
        }),
        Box::new(|| belt_structural::check_belt_inserter_conflict(layout)),
        Box::new(|| check_belt_flow_reachability(layout, solver, layout_style)),
        Box::new(|| belt_structural::check_lane_throughput(layout, solver)),
        Box::new(|| check_input_rate_delivery(layout, solver)),
        Box::new(|| check_balancer_template_coverage(layout)),
        Box::new(|| modules::check_module_slots(layout)),
        Box::new(|| modules::check_module_eligibility(layout)),
        Box::new(|| belt_detour::check_belt_detour(layout)),
    ];

    let issues: Vec<ValidationIssue> = checks.par_iter().flat_map(|f| f()).collect();

    let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warning_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();
    crate::trace::emit(crate::trace::TraceEvent::ValidationCompleted {
        error_count,
        warning_count,
        issues: issues.iter().map(|i| crate::trace::ValidationIssueTrace {
            severity: i.severity.as_str().to_string(),
            category: i.category.clone(),
            message: i.message.clone(),
            x: i.x,
            y: i.y,
        }).collect(),
    });

    let any_errors = issues.iter().any(|i| i.severity == Severity::Error);
    if any_errors {
        // Pass the full issues list (errors + warnings) so callers that
        // do `Err(e) => e.issues` keep an accurate picture. Without this,
        // a single masking error silently dropped every warning produced
        // in the same run (issue #298).
        return Err(ValidationError::new(issues));
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, ItemFlow, LayoutResult, PlacedEntity};

    fn empty_layout() -> LayoutResult {
        LayoutResult {
            entities: vec![],
            width: 0,
            height: 0,
            ..Default::default()
        }
    }

    /// Reconstructs the failure this check exists for: a transform moved the
    /// output belt and left the boundary record behind. The geometry is
    /// entirely legal — a belt, carrying the right item, correctly placed —
    /// which is why every geometry check passed while the factory produced
    /// nothing in Factorio.
    #[test]
    fn stale_boundary_output_record_is_an_error() {
        use crate::models::BoundaryRecord;
        let belt_at = |x: i32, y: i32| PlacedEntity {
            name: "transport-belt".to_string(),
            x,
            y,
            direction: EntityDirection::South,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        };

        // The belt actually lives at (5,9).
        let mut layout = LayoutResult {
            entities: vec![belt_at(5, 9)],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let record = |x: i32, y: i32| BoundaryRecord {
            item: "iron-plate".to_string(),
            x,
            y,
            direction: EntityDirection::South,
            is_fluid: false,
            entity: "transport-belt".to_string(),
        };

        // Record agrees with the belt: nothing to report.
        layout.boundary_outputs = vec![record(5, 9)];
        assert!(
            check_boundary_record_integrity(&layout).is_empty(),
            "a record that matches its belt must not be flagged"
        );

        // Record left behind at the pre-transform tile.
        layout.boundary_outputs = vec![record(5, 3)];
        let issues = check_boundary_record_integrity(&layout);
        assert_eq!(issues.len(), 1, "stale record must be reported: {issues:?}");
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].category, "boundary-record-integrity");

        // Inputs get the same treatment; they were previously checked for
        // fluids only, so a solid input record was unguarded entirely.
        layout.boundary_outputs = vec![];
        layout.boundary_inputs = vec![record(5, 3)];
        assert_eq!(check_boundary_record_integrity(&layout).len(), 1);
    }

    fn layout_with_machine() -> LayoutResult {
        LayoutResult {
            entities: vec![PlacedEntity {
                name: "assembling-machine-1".to_string(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                io_type: None,
                carries: None,
                mirror: false,
                segment_id: None,
                ..Default::default()
            }],
            width: 10,
            height: 10,
            ..Default::default()
        }
    }

    fn solid_surplus_solver(item: &str, rate: f64) -> SolverResult {
        SolverResult {
            machines: vec![],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![ItemFlow {
                item: item.to_string(),
                rate,
                is_fluid: false,
                module_id: 0,
            }],
            dependency_order: vec![],
            ..Default::default()
        }
    }

    // ── check_stranded_byproducts (solid surplus, RFC Fulgora D2a/D2b) ──────

    #[test]
    fn stranded_byproducts_solid_surplus_with_exit_and_belt_is_clean() {
        let solver = solid_surplus_solver("uranium-238", 7.09);
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "transport-belt".to_string(),
                x: 10,
                y: 20,
                carries: Some("uranium-238".to_string()),
                ..Default::default()
            }],
            width: 30,
            height: 30,
            surplus_exits: vec![("uranium-238".to_string(), 10, 20)],
            ..Default::default()
        };
        let issues = check_stranded_byproducts(&layout, &solver);
        assert!(
            issues.is_empty(),
            "expected no stranded-byproduct issues, got {issues:?}"
        );
    }

    #[test]
    fn stranded_byproducts_solid_surplus_exit_without_belt_still_errors() {
        let solver = solid_surplus_solver("uranium-238", 7.09);
        // Exit tile recorded but no matching entity actually sits there —
        // a ledger entry without the physical belt is exactly the
        // stalled-machine bug this check exists to catch.
        let layout = LayoutResult {
            entities: vec![],
            width: 30,
            height: 30,
            surplus_exits: vec![("uranium-238".to_string(), 10, 20)],
            ..Default::default()
        };
        let issues = check_stranded_byproducts(&layout, &solver);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "stranded-byproduct");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn stranded_byproducts_solid_surplus_wrong_entity_kind_still_errors() {
        // A pipe carrying the item at the exit tile doesn't count for a
        // SOLID surplus — solids need a belt/underground-belt/splitter,
        // mirroring fluids needing a pipe (not a belt).
        let solver = solid_surplus_solver("uranium-238", 7.09);
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "pipe".to_string(),
                x: 10,
                y: 20,
                carries: Some("uranium-238".to_string()),
                ..Default::default()
            }],
            width: 30,
            height: 30,
            surplus_exits: vec![("uranium-238".to_string(), 10, 20)],
            ..Default::default()
        };
        let issues = check_stranded_byproducts(&layout, &solver);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn stranded_byproducts_solid_surplus_no_exit_record_errors() {
        let solver = solid_surplus_solver("uranium-238", 7.09);
        let layout = LayoutResult {
            entities: vec![],
            width: 30,
            height: 30,
            ..Default::default()
        };
        let issues = check_stranded_byproducts(&layout, &solver);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
    }

    #[test]
    fn issue_new_has_no_position() {
        let issue = ValidationIssue::new(Severity::Error, "pipe-isolation", "test message");
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, "pipe-isolation");
        assert_eq!(issue.message, "test message");
        assert_eq!(issue.x, None);
        assert_eq!(issue.y, None);
    }

    #[test]
    fn issue_with_pos_stores_coordinates() {
        let issue = ValidationIssue::with_pos(Severity::Warning, "power", "no pole", 3, 7);
        assert_eq!(issue.severity, Severity::Warning);
        assert_eq!(issue.x, Some(3));
        assert_eq!(issue.y, Some(7));
    }

    #[test]
    fn validation_error_contains_issues() {
        let issues = vec![
            ValidationIssue::new(Severity::Error, "pipe-isolation", "fluids merged"),
            ValidationIssue::new(Severity::Error, "power", "no coverage"),
        ];
        let err = ValidationError::new(issues.clone());
        assert_eq!(err.issues.len(), 2);
        assert_eq!(err.issues[0].category, "pipe-isolation");
    }

    #[test]
    fn validation_error_message_format() {
        let issues = vec![ValidationIssue::new(Severity::Error, "power", "no pole nearby")];
        let err = ValidationError::new(issues);
        let msg = err.to_string();
        assert!(msg.contains("Validation failed:"));
        assert!(msg.contains("[error]"));
        assert!(msg.contains("no pole nearby"));
    }

    #[test]
    fn validate_empty_layout_returns_ok_with_no_poles_warning() {
        let lr = empty_layout();
        let result = validate(&lr, None, LayoutStyle::Spaghetti);
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "power");
    }

    #[test]
    fn validate_with_machine_returns_errors() {
        let lr = layout_with_machine();
        let result = validate(&lr, None, LayoutStyle::Bus);
        assert!(result.is_err(), "expected errors for a machine with no belts");
    }

    #[test]
    fn validation_error_carries_warnings_alongside_errors() {
        // Regression test for #298: when both errors and warnings exist,
        // the Err path used to drop the warnings, hiding pre-existing
        // issues from any caller that checked `e.issues.len()`.
        let issues = vec![
            ValidationIssue::new(Severity::Error, "pipe-isolation", "fluids merged"),
            ValidationIssue::new(Severity::Warning, "input-rate-delivery", "slow input"),
            ValidationIssue::new(Severity::Warning, "belt-flow-reachability", "stranded furnace"),
        ];
        let err = ValidationError::new(issues);
        assert_eq!(err.issues.len(), 3, "all issues must survive on Err path");
        assert_eq!(
            err.issues.iter().filter(|i| i.severity == Severity::Error).count(),
            1
        );
        assert_eq!(
            err.issues.iter().filter(|i| i.severity == Severity::Warning).count(),
            2
        );
        // Display should still focus on errors only.
        let msg = err.to_string();
        assert!(msg.contains("fluids merged"), "error message must surface");
        assert!(!msg.contains("slow input"), "warnings shouldn't pollute error message");
    }

    #[test]
    fn validate_default_layout_style_is_spaghetti() {
        assert_eq!(LayoutStyle::default(), LayoutStyle::Spaghetti);
    }

    #[test]
    fn layout_style_equality() {
        assert_eq!(LayoutStyle::Bus, LayoutStyle::Bus);
        assert_ne!(LayoutStyle::Bus, LayoutStyle::Spaghetti);
    }

    // ── check_shared_row_outflow_conservation (RFC-062 Phase 2) ────────────

    /// EC+AC shape, hand-numbers matching the RFC-062 Phase 0 decision
    /// log: an `electronic-circuit` row built to 16.5/s (11 AM2 machines
    /// at 1.5/s each) serving both AC's 6.0/s ingredient draw and a
    /// 10.0/s export target. `machine_count` is a synthetic
    /// (entity, recipe) pair — one entity per producer/consumer, matching
    /// what `producer_recipes` needs to resolve a position.
    fn ec_ac_shared_row_solver(ec_target_rate: f64) -> SolverResult {
        SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "electronic-circuit".to_string(),
                    count: 11.0,
                    outputs: vec![ItemFlow {
                        item: "electronic-circuit".to_string(),
                        rate: 1.5,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "advanced-circuit".to_string(),
                    count: 24.0,
                    inputs: vec![ItemFlow {
                        item: "electronic-circuit".to_string(),
                        rate: 0.25,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    ..Default::default()
                },
            ],
            external_outputs: vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: ec_target_rate,
                is_fluid: false,
                module_id: 0,
            }],
            ..Default::default()
        }
    }

    fn ec_producer_entity() -> PlacedEntity {
        PlacedEntity {
            name: "assembling-machine-2".to_string(),
            x: 5,
            y: 1,
            recipe: Some("electronic-circuit".to_string()),
            ..Default::default()
        }
    }

    /// The row's real production (16.5/s) covers export (10) + taps (6) =
    /// 16 with slack — the KC2 canonical case, and the check must stay
    /// quiet on it (validator-reporting rule 5: quiet is not evidence of
    /// correctness on its own, but a check that fires on the FIXED case
    /// is simply wrong).
    #[test]
    fn shared_row_outflow_conservation_quiet_when_within_capacity() {
        let solver = ec_ac_shared_row_solver(10.0);
        let layout = LayoutResult {
            entities: vec![ec_producer_entity()],
            width: 30,
            height: 30,
            boundary_outputs: vec![crate::models::BoundaryRecord {
                item: "electronic-circuit".to_string(),
                x: 5,
                y: 30,
                direction: EntityDirection::South,
                is_fluid: false,
                entity: "transport-belt".to_string(),
            }],
            surplus_exits: vec![("electronic-circuit".to_string(), 5, 30)],
            ..Default::default()
        };
        // Physical entity at the recorded exit, matching the record.
        let mut layout = layout;
        layout.entities.push(PlacedEntity {
            name: "transport-belt".to_string(),
            x: 5,
            y: 30,
            carries: Some("electronic-circuit".to_string()),
            ..Default::default()
        });
        let issues = check_shared_row_outflow_conservation(&layout, &solver);
        assert!(issues.is_empty(), "expected no issues on the within-capacity case: {issues:#?}");
    }

    /// Bump the target so export (11) + taps (6) = 17 exceeds the row's
    /// real 16.5/s production — one positioned
    /// `shared-row-outflow-overclaim` ERROR, not a count folded into a
    /// message (docs/validator-reporting.md rule 1). `ec_ac_shared_row_
    /// solver`'s machine counts (11.0, 24.0) are both exact integers, so
    /// PLAN and BUILT totals are identical here — this is a genuine
    /// plan-level overclaim, not the ceil-slack case the next test
    /// covers, and must stay `Error`.
    #[test]
    fn shared_row_outflow_conservation_flags_overclaim() {
        let solver = ec_ac_shared_row_solver(11.0);
        let layout = LayoutResult {
            entities: vec![ec_producer_entity()],
            width: 30,
            height: 30,
            ..Default::default()
        };
        let issues = check_shared_row_outflow_conservation(&layout, &solver);
        let overclaims: Vec<_> =
            issues.iter().filter(|i| i.category == "shared-row-outflow-overclaim").collect();
        assert_eq!(overclaims.len(), 1, "expected exactly one over-claim issue: {issues:#?}");
        assert_eq!(overclaims[0].severity, Severity::Error);
        assert_eq!(overclaims[0].x, Some(5));
        assert_eq!(overclaims[0].y, Some(1));
        let detail = overclaims[0].detail.as_ref().expect("over-claim issue must carry IssueDetail");
        assert!((detail.delivered - 16.5).abs() < 1e-9, "delivered should be the row's real production: {detail:?}");
        assert!((detail.needed - 17.0).abs() < 1e-9, "needed should be the claimed total: {detail:?}");
    }

    /// F2 (RFC-062 Phase 2 review fix round): reproduces the reviewer's
    /// measured shape — two INDEPENDENTLY ceiled `MachineSpec`s (producer
    /// count 10.0 exact, consumer count 6.001) whose BUILT totals collide
    /// (claimed 10.5 > produced 10.0) purely because the consumer's own
    /// ceiling rounds 6.001 up to a whole 7 machines, while the PLAN
    /// (raw, un-ceiled) totals comfortably balance (claimed 9.501 ≤
    /// planned 10.0). Must be `Warning`, not `Error` — this is built-
    /// capacity contention between two independent roundings, not a
    /// genuine demand overrun (the reviewer's own EC@10.4+AC@3.05 example
    /// is the same shape, just with the real recipe's numbers).
    #[test]
    fn shared_row_outflow_conservation_overclaim_is_warning_when_only_ceil_slack() {
        let solver = SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "electronic-circuit".to_string(),
                    count: 10.0,
                    outputs: vec![ItemFlow {
                        item: "electronic-circuit".to_string(),
                        rate: 1.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    ..Default::default()
                },
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "advanced-circuit".to_string(),
                    count: 6.001,
                    inputs: vec![ItemFlow {
                        item: "electronic-circuit".to_string(),
                        rate: 1.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    ..Default::default()
                },
            ],
            external_outputs: vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: 3.5,
                is_fluid: false,
                module_id: 0,
            }],
            ..Default::default()
        };
        // A real physical export claim, so this test isolates the
        // severity split — without it the under-claim direction would
        // also fire (target > 0, no boundary/surplus_exits record),
        // which is a different finding this test doesn't probe.
        let layout = LayoutResult {
            entities: vec![
                ec_producer_entity(),
                PlacedEntity {
                    name: "transport-belt".to_string(),
                    x: 5,
                    y: 30,
                    carries: Some("electronic-circuit".to_string()),
                    ..Default::default()
                },
            ],
            width: 30,
            height: 30,
            surplus_exits: vec![("electronic-circuit".to_string(), 5, 30)],
            ..Default::default()
        };
        let issues = check_shared_row_outflow_conservation(&layout, &solver);
        let overclaims: Vec<_> =
            issues.iter().filter(|i| i.category == "shared-row-outflow-overclaim").collect();
        assert_eq!(overclaims.len(), 1, "expected exactly one over-claim issue: {issues:#?}");
        assert_eq!(
            overclaims[0].severity,
            Severity::Warning,
            "ceil-slack-only overclaim must be a Warning, not an Error: {:#?}",
            overclaims[0]
        );
        let detail = overclaims[0].detail.as_ref().expect("over-claim issue must carry IssueDetail");
        assert!((detail.delivered - 10.0).abs() < 1e-9, "delivered should be BUILT production: {detail:?}");
        assert!((detail.needed - 10.5).abs() < 1e-9, "needed should be BUILT claimed total: {detail:?}");
        assert!(
            !issues.iter().any(|i| i.category == "shared-row-outflow-underclaim"),
            "expected no under-claim issue — a real physical export record is present: {issues:#?}"
        );
    }

    /// Target + taps are both live (a "shared row" by the check's own
    /// gate) but NO physical export record exists at all — the RFC-062
    /// Phase 0 "dropped export under the cell-composed candidate" finding,
    /// reproduced as a minimal synthetic case. The under-claim direction
    /// exists specifically to catch this: `check_belt_network_topology`'s
    /// own output-network check stays silent when `belt_starts` is
    /// entirely empty (see the Phase 2 decision log).
    #[test]
    fn shared_row_outflow_conservation_flags_missing_export_record() {
        let solver = ec_ac_shared_row_solver(10.0);
        let layout = LayoutResult {
            entities: vec![ec_producer_entity()],
            width: 30,
            height: 30,
            // Deliberately no `boundary_outputs`/`surplus_exits` entry for
            // electronic-circuit at all.
            ..Default::default()
        };
        let issues = check_shared_row_outflow_conservation(&layout, &solver);
        let underclaims: Vec<_> =
            issues.iter().filter(|i| i.category == "shared-row-outflow-underclaim").collect();
        assert_eq!(underclaims.len(), 1, "expected exactly one under-claim issue: {issues:#?}");
        assert_eq!(underclaims[0].severity, Severity::Error);
        assert_eq!(underclaims[0].x, Some(5));
        assert_eq!(underclaims[0].y, Some(1));
    }

    /// A `surplus_exits` ledger entry alone, with no matching physical
    /// entity, must not satisfy the under-claim check — a ledger without
    /// the entity is exactly the stalled-machine bug this direction
    /// exists to catch (mirrors `check_stranded_byproducts`'s own
    /// standard).
    #[test]
    fn shared_row_outflow_conservation_ledger_without_entity_still_flags() {
        let solver = ec_ac_shared_row_solver(10.0);
        let layout = LayoutResult {
            entities: vec![ec_producer_entity()],
            width: 30,
            height: 30,
            // Record claims an exit at (5,30), but no entity carrying
            // electronic-circuit actually sits there.
            surplus_exits: vec![("electronic-circuit".to_string(), 5, 30)],
            ..Default::default()
        };
        let issues = check_shared_row_outflow_conservation(&layout, &solver);
        assert!(
            issues.iter().any(|i| i.category == "shared-row-outflow-underclaim"),
            "a surplus_exits record with no backing entity must still be flagged: {issues:#?}"
        );
    }

    /// A row with only ONE live claim (a pure internal intermediate, or a
    /// pure export with no internal consumer) is not "shared" — the
    /// solver's own flow conservation already guarantees it's correct, so
    /// the check must stay quiet regardless of production/tap numbers.
    #[test]
    fn shared_row_outflow_conservation_ignores_non_shared_rows() {
        let solver = SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-2".to_string(),
                recipe: "electronic-circuit".to_string(),
                count: 1.0,
                outputs: vec![ItemFlow {
                    item: "electronic-circuit".to_string(),
                    rate: 0.1,
                    is_fluid: false,
                    module_id: 0,
                }],
                ..Default::default()
            }],
            // No external_outputs, no other machine consumes it: a purely
            // internal (or in this case fully unclaimed) item.
            ..Default::default()
        };
        let layout = LayoutResult { entities: vec![ec_producer_entity()], width: 30, height: 30, ..Default::default() };
        assert!(
            check_shared_row_outflow_conservation(&layout, &solver).is_empty(),
            "a row with fewer than 2 live claims must never be flagged"
        );
    }
}
