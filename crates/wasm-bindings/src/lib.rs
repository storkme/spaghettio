//! wasm-bindgen bindings for the Spaghettio pipeline.
//!
//! Thin wrapper around `spaghettio_core` that exposes the full pipeline to the
//! browser via WASM. Loaded by `web/src/engine.ts`.
//!
//! Build: `wasm-pack build crates/wasm-bindings --target web --out-dir ../../web/src/wasm-pkg`
//!
//! Exposed functions: `init`, `solve`, `layout`, `export_blueprint`, `validate`,
//! `get_all_items`, `get_recipes_for_item`, `parse_blueprint`.

use spaghettio_core::bus::inserter_ladder::InserterTier;
use spaghettio_core::models::{LayoutResult, PlacedEntity, SolverResult};
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::validate::{self, LayoutStyle, ValidationIssue};
use spaghettio_core::{
    blueprint, blueprint_parser, bus::junction_cost::solution_cost,
    bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy, RowLayout, SurplusPolicy},
    fixture as fixture_mod, recipe_db, sat, solver,
};
use rustc_hash::FxHashSet;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Build `LayoutOptions` from the optional belt-tier, strategy,
/// row-layout, and inserter-tier strings passed in across the WASM
/// boundary. The TS engine layer validates URL params, so unknown values
/// fall back to defaults silently.
fn layout_options(
    max_belt_tier: Option<String>,
    strategy: Option<String>,
    row_layout: Option<String>,
    max_inserter_tier: Option<String>,
    quality: Option<String>,
) -> LayoutOptions {
    let strategy = match strategy.as_deref() {
        // `partitioned-per-consumer` is the deprecated P1 string; the
        // P1 enum variant was hard-deleted (it was strictly dominated
        // by P2 across the diag corpus). Bookmarked URLs continue to
        // load by transparently mapping to `PartitionedDecomposed`.
        Some("partitioned-per-consumer") | Some("partitioned-decomposed") => {
            LayoutStrategy::PartitionedDecomposed
        }
        _ => LayoutStrategy::Pooled,
    };
    let row_layout = match row_layout.as_deref() {
        Some("horizontal-stack") => RowLayout::HorizontalStack,
        _ => RowLayout::VerticalSplit,
    };
    // Mirrors `max_belt_tier`'s hard-cap semantics (`docs/rfp-inserter-
    // sizing.md`): unrecognized or absent values fall back to the
    // default (`Stack`), not an error.
    let max_inserter_tier = match max_inserter_tier.as_deref() {
        Some("regular") => InserterTier::Regular,
        Some("fast") => InserterTier::Fast,
        _ => InserterTier::default(),
    };
    LayoutOptions {
        strategy,
        max_belt_tier,
        row_layout,
        surplus_policy: SurplusPolicy::default(),
        max_inserter_tier,
        // rfp-build-quality Phase 2: unknown/absent → Normal, same
        // hard-cap fallback semantics as the two tiers above.
        quality: quality_tier(quality),
        // The merge-tap fallback is chosen internally by the
        // decomposition search (`MergeTapCandidate`), never requested by the
        // web UI — always default-off at the public boundary.
        merge_tap: false,
    }
}

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Map the optional quality string from JS to a tier; unknown/absent →
/// Normal, matching the `max_inserter_tier` unknown→default pattern
/// (rfp-build-quality Phase 1). NOTE (Phase 1 guard rail): the web UI and
/// URL codec must NOT set this yet — Phase 2 lands that surface.
fn quality_tier(quality: Option<String>) -> spaghettio_core::common::QualityTier {
    quality
        .as_deref()
        .and_then(spaghettio_core::common::QualityTier::from_name)
        .unwrap_or_default()
}

#[wasm_bindgen]
pub fn solve(
    target_item: &str,
    target_rate: f64,
    available_inputs: Vec<String>,
    machine_entity: &str,
    quality: Option<String>,
) -> Result<SolverResult, JsError> {
    let inputs: FxHashSet<String> = available_inputs.into_iter().collect();
    solver::solve_with_palette_exclusions_and_quality(
        target_item,
        target_rate,
        &inputs,
        &MachinePalette::default(),
        machine_entity,
        &FxHashSet::default(),
        quality_tier(quality),
    )
    .map_err(|e| JsError::new(&e.to_string()))
}

/// Solve with a per-category machine palette. Categories absent from the
/// palette fall through to the hardcoded mapping (see
/// `recipe_db::machine_for_recipe`); fully unmapped categories use
/// `default_machine`.
#[wasm_bindgen]
pub fn solve_with_palette(
    target_item: &str,
    target_rate: f64,
    available_inputs: Vec<String>,
    palette: MachinePalette,
    default_machine: &str,
    quality: Option<String>,
) -> Result<SolverResult, JsError> {
    let inputs: FxHashSet<String> = available_inputs.into_iter().collect();
    solver::solve_with_palette_exclusions_and_quality(
        target_item,
        target_rate,
        &inputs,
        &palette,
        default_machine,
        &FxHashSet::default(),
        quality_tier(quality),
    )
    .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn all_producible_items() -> Vec<String> {
    recipe_db::all_producible_items()
}

#[wasm_bindgen]
pub fn all_producer_machines() -> Vec<String> {
    recipe_db::all_producer_machines()
}

#[wasm_bindgen]
pub fn default_machine_for_item(item: &str, fallback: &str) -> String {
    recipe_db::default_machine_for_item(item, fallback)
}

#[wasm_bindgen]
pub fn layout(
    solver_result: SolverResult,
    max_belt_tier: Option<String>,
    strategy: Option<String>,
    row_layout: Option<String>,
    max_inserter_tier: Option<String>,
    quality: Option<String>,
) -> Result<LayoutResult, JsError> {
    build_bus_layout(
        &solver_result,
        layout_options(max_belt_tier, strategy, row_layout, max_inserter_tier, quality),
    )
    .map_err(|e| JsError::new(&e))
}

/// Traced variant of `layout()`. Returns the same `LayoutResult` plus
/// the structured `trace` events that drive the debug overlays. Ghost
/// routing is the only routing path — the legacy direct router was
/// deleted; both `layout()` and `layout_traced()` go through it.
#[wasm_bindgen]
pub fn layout_traced(
    solver_result: SolverResult,
    max_belt_tier: Option<String>,
    strategy: Option<String>,
    row_layout: Option<String>,
    max_inserter_tier: Option<String>,
    quality: Option<String>,
) -> Result<LayoutResult, JsError> {
    spaghettio_core::bus::layout::build_bus_layout_traced(
        &solver_result,
        layout_options(max_belt_tier, strategy, row_layout, max_inserter_tier, quality),
    )
    .map_err(|e| JsError::new(&e))
}

/// Filter predicate — determines which TraceEvent variants are forwarded to
/// the streaming JS callback. The full event set remains collected in
/// `LayoutResult.trace` for post-hoc consumption.
///
/// Streamed today:
/// - `PhaseSnapshot`: drives phase-to-phase entity fade-ins.
/// - `GhostSpecRouted`: per-spec routed path — feeds the live ghost-path
///   overlay during negotiation. Negotiation is fast (<20 ms in practice)
///   so most of these fire in a burst near the start.
/// - `GhostClusterSolved` / `JunctionSolved` / `SatInvocation`: give the
///   junction-solver phase something visible. SAT zone solving dominates
///   the 5-6 s wait on hard layouts, and without streaming these, the
///   overlay goes silent for that whole window.
///
/// The overlay renderer uses a single shared `Graphics` redrawn per frame,
/// so event count doesn't blow up Pixi's tree.
fn streamable(evt: &spaghettio_core::trace::TraceEvent) -> bool {
    use spaghettio_core::trace::TraceEvent as T;
    matches!(
        evt,
        T::PhaseSnapshot { .. }
            | T::GhostSpecRouted { .. }
            | T::GhostSpecCommitted { .. }
            | T::GhostClusterSolved { .. }
            | T::JunctionCommitted { .. }
            | T::JunctionSolved { .. }
            | T::SatInvocation { .. }
            | T::SatImprovement { .. }
            // Stream siblings of metadata-only stamp events. Without these,
            // bus trunks, balancers, output mergers, and poles surface only
            // via the bus_routed / poles_placed PhaseSnapshot safety nets
            // and slam onto the canvas all at once at the end of layout.
            | T::TrunkBeltCommitted { .. }
            | T::BalancerCommitted { .. }
            | T::OutputMergerCommitted { .. }
            | T::PolesCommitted { .. }
            // Retry signal — UI clears its rendered state so pass-2 events
            // don't stack on top of pass-1 entities that were abandoned.
            | T::LayoutRetried { .. }
    )
}

/// Streaming variant — invokes `emit` synchronously for every filtered trace
/// event during the layout run. The JS callback fires on the worker thread;
/// use it to `postMessage` events to the main thread as the engine emits
/// them. Returns the completed `LayoutResult` with the *full* (unfiltered)
/// `trace` populated, so callers that ignore streaming still get a usable
/// result identical to `layout_traced`.
#[wasm_bindgen]
pub fn layout_streaming(
    solver_result: SolverResult,
    max_belt_tier: Option<String>,
    strategy: Option<String>,
    row_layout: Option<String>,
    max_inserter_tier: Option<String>,
    quality: Option<String>,
    emit: &js_sys::Function,
) -> Result<LayoutResult, JsError> {
    let emit = emit.clone();
    let on_event: Box<dyn FnMut(&spaghettio_core::trace::TraceEvent)> = Box::new(move |evt| {
        if !streamable(evt) {
            return;
        }
        if let Ok(js_evt) = serde_wasm_bindgen::to_value(evt) {
            let _ = emit.call1(&JsValue::NULL, &js_evt);
        }
    });
    spaghettio_core::bus::layout::build_bus_layout_streaming(
        &solver_result,
        layout_options(max_belt_tier, strategy, row_layout, max_inserter_tier, quality),
        on_event,
    )
    .map_err(|e| JsError::new(&e))
}

/// Interactive cost-descent pass for a single SAT crossing zone.
///
/// Finds the region in `layout_result.regions` by `region_id`, rebuilds
/// the `CrossingZone` from its stored ports + metadata, and runs a
/// long-running cost-descent loop. Every time a strictly-cheaper
/// layout is found, `emit` fires with a `SatImprovement` trace event
/// carrying the new entity list and cost — the frontend animates these.
///
/// Returns the `LayoutResult` with the zone's tiles replaced by the
/// final best entities, so blueprint export and subsequent renders
/// reflect the improvement.
///
/// `budget_ms` — wall-clock budget for the descent loop. Clamped to
/// 100..=60_000 ms server-side; typical UI call passes ~10_000.
///
/// `max_iters` — cap on descent steps. 0 is treated as "unbounded"
/// (1024). The round-robin "Optimize all" driver passes 1 to get
/// "emit initial snapshot, attempt one cap-probe, return" semantics.
#[wasm_bindgen]
pub fn improve_region_streaming(
    mut layout_result: LayoutResult,
    region_id: u32,
    budget_ms: u32,
    max_iters: u32,
    emit: &js_sys::Function,
) -> Result<LayoutResult, JsError> {
    use spaghettio_core::bus::region_reimprove::{
        deadline_in, descend, prune_dangling, rebuild_zone_from_region,
    };
    use spaghettio_core::models::RegionKind;

    let region = layout_result
        .regions
        .iter()
        .find(|r| r.id == region_id && r.kind == RegionKind::CrossingZone)
        .ok_or_else(|| JsError::new(&format!("no CrossingZone region with id {region_id}")))?
        .clone();

    let (zone, belt_tier, max_ug_reach) = rebuild_zone_from_region(&region).ok_or_else(|| {
        JsError::new("region missing SAT zone metadata (belt_tier / max_ug_reach)")
    })?;

    // Extract the zone's current entities (the starting point for
    // descent) from the layout. Every belt/UG inside the bbox is fair
    // game; anything else stays where it is.
    let x0 = zone.x;
    let y0 = zone.y;
    let x1 = x0 + zone.width as i32;
    let y1 = y0 + zone.height as i32;
    let in_bbox = |e: &PlacedEntity| e.x >= x0 && e.x < x1 && e.y >= y0 && e.y < y1;
    let initial: Vec<PlacedEntity> = layout_result
        .entities
        .iter()
        .filter(|e| in_bbox(e) && is_belt_or_ug(&e.name))
        .cloned()
        .collect();

    if initial.is_empty() {
        return Err(JsError::new("no belt/UG entities found inside the zone"));
    }

    let budget_ms = budget_ms.clamp(100, 60_000) as u64;
    let deadline = deadline_in(budget_ms);

    let emit = emit.clone();
    let boundaries = zone.boundaries.clone();
    let zx = zone.x;
    let zy = zone.y;
    let zw = zone.width;
    let zh = zone.height;
    let mut iter: u32 = 0;
    let iter_cap = if max_iters == 0 { 1024 } else { max_iters };
    let (final_raw, stop_reason) = descend(
        &zone,
        &belt_tier,
        max_ug_reach,
        None,
        initial,
        deadline,
        iter_cap,
        |imp| {
            let pruned =
                prune_dangling(imp.entities.to_vec(), &boundaries, max_ug_reach, zx, zy);
            let evt = spaghettio_core::trace::TraceEvent::SatImprovement {
                region_id,
                zone_x: zx,
                zone_y: zy,
                zone_w: zw,
                zone_h: zh,
                cost: imp.cost,
                iter,
                solve_time_us: imp.solve_time_us,
                entities: pruned,
            };
            if let Ok(js_evt) = serde_wasm_bindgen::to_value(&evt) {
                let _ = emit.call1(&JsValue::NULL, &js_evt);
            }
            iter += 1;
        },
    );

    // Splice the pruned final result back into the layout. Drop every
    // belt/UG inside the zone bbox, then push the new set. SAT stamps
    // per-channel tiers at solve time so no post-pass retype needed.
    let pruned_final = prune_dangling(final_raw, &boundaries, max_ug_reach, zx, zy);

    // Descent terminated with UNSAT on cap-1: this is the proven optimum
    // for the zone. Record it (updates the in-memory cache for same-session
    // reuse) and emit a terminal trace event carrying the binary record so
    // the frontend can persist to localStorage.
    if matches!(stop_reason, spaghettio_core::bus::region_reimprove::StopReason::Optimal) {
        // Per-channel reaches: descent uses uniform `max_ug_reach`, so the
        // signature should match. Size the array to cover every channel_id
        // referenced by any boundary. `max_ug_ins = None` matches the
        // descent's call into `solve_crossing_zone_with_cost_cap`.
        let n_channels = boundaries
            .iter()
            .map(|b| b.channel_id as usize + 1)
            .max()
            .unwrap_or(0);
        let channel_reaches = vec![max_ug_reach; n_channels];
        let stats = spaghettio_core::zone_cache::ZoneStats {
            variables: 0,
            clauses: 0,
            solve_time_us: 0,
        };
        let (signature, record_bytes) = spaghettio_core::zone_cache::record_zone_with_solution(
            &zone,
            &channel_reaches,
            None,
            stats,
            &pruned_final,
            Some("wasm-optimize"),
        );
        let evt = spaghettio_core::trace::TraceEvent::SatOptimumProven {
            region_id,
            signature,
            record_bytes,
        };
        if let Ok(js_evt) = serde_wasm_bindgen::to_value(&evt) {
            let _ = emit.call1(&JsValue::NULL, &js_evt);
        }
    }

    layout_result
        .entities
        .retain(|e| !(in_bbox(e) && is_belt_or_ug(&e.name)));
    layout_result.entities.extend(pruned_final);

    Ok(layout_result)
}

fn is_belt_or_ug(name: &str) -> bool {
    matches!(
        name,
        "transport-belt"
            | "fast-transport-belt"
            | "express-transport-belt"
            | "underground-belt"
            | "fast-underground-belt"
            | "express-underground-belt"
    )
}

#[wasm_bindgen]
pub fn export_blueprint(layout_result: LayoutResult, label: String) -> String {
    blueprint::export(&layout_result, &label)
}

#[wasm_bindgen]
pub fn parse_blueprint(bp_string: &str) -> Result<LayoutResult, JsError> {
    blueprint_parser::parse_blueprint_string(bp_string).map_err(|e| JsError::new(&e))
}

/// Seed the in-memory SAT crossing-zone cache with `bytes` — a sequence of
/// length-prefixed binary records in the same format as
/// `crates/core/data/sat-zones.bin`. The frontend calls this on boot with
/// records previously persisted to localStorage by `SatOptimumProven`.
///
/// Returns the number of records successfully ingested. Malformed records
/// are skipped silently, so a partially-corrupt blob still installs
/// whatever decodes cleanly.
#[wasm_bindgen]
pub fn seed_zone_cache(bytes: &[u8]) -> u32 {
    let count = spaghettio_core::zone_cache::parse_records(bytes).len() as u32;
    spaghettio_core::zone_cache::install_prebaked(bytes);
    count
}

/// Response shape for `solve_fixture`. Serialised via
/// `serde_wasm_bindgen` — the TS side mirrors this in `engine.ts`.
#[derive(Serialize)]
struct SolveFixtureResponse {
    entities: Vec<PlacedEntity>,
    cost: u32,
    stats: SolveFixtureStats,
}

#[derive(Serialize)]
struct SolveFixtureStats {
    variables: u32,
    clauses: u32,
    solve_time_us: u64,
    zone_width: u32,
    zone_height: u32,
}

/// Solve a SAT-zone fixture, optionally with a set of painted entities
/// pinned as assumptions. Returns `null` (JS) on UNSAT or invalid pins.
///
/// `fixture_json` — JSON matching the v1 fixture schema (see
/// `crates/core/src/fixture.rs`).
/// `pins_json` — JSON array of `PlacedEntity` to assume; `"[]"` for an
/// unconstrained solve.
///
/// Used by the F2 SAT-zone editor to (a) validate the user's painted
/// state and (b) render a ghost-completion overlay showing how SAT
/// would extend the paint.
#[wasm_bindgen]
pub fn solve_fixture(fixture_json: &str, pins_json: &str) -> Result<JsValue, JsError> {
    let fixture: fixture_mod::Fixture = serde_json::from_str(fixture_json)
        .map_err(|e| JsError::new(&format!("fixture parse: {e}")))?;
    let pins: Vec<PlacedEntity> = serde_json::from_str(pins_json)
        .map_err(|e| JsError::new(&format!("pins parse: {e}")))?;

    let zone = fixture_mod::build_zone(&fixture);
    let (result, stats) = sat::solve_crossing_zone_with_pins(
        &zone,
        &pins,
        fixture.max_reach,
        &fixture.belt_tier,
        None,
    );

    let Some(entities) = result else {
        return Ok(JsValue::NULL);
    };

    let response = SolveFixtureResponse {
        cost: solution_cost(&entities),
        entities,
        stats: SolveFixtureStats {
            variables: stats.variables,
            clauses: stats.clauses,
            solve_time_us: stats.solve_time_us,
            zone_width: stats.zone_width,
            zone_height: stats.zone_height,
        },
    };

    serde_wasm_bindgen::to_value(&response).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn validate_layout(
    layout_result: LayoutResult,
    solver_result: Option<SolverResult>,
    layout_style: Option<LayoutStyle>,
) -> Vec<ValidationIssue> {
    let style = layout_style.unwrap_or_default();
    let solver_ref: Option<&SolverResult> = solver_result.as_ref();
    // Always return the full issue list to the web UI. The native
    // `validate::validate` returns `Err(ValidationError)` when any
    // error-severity issues exist (Python parity), but the error path
    // discards the structured issue list. The UI needs the issues
    // themselves to render borders + the badge (#209), so we unwrap
    // the error case back into the same `Vec<ValidationIssue>`.
    match validate::validate(&layout_result, solver_ref, style) {
        Ok(issues) => issues,
        Err(err) => err.issues,
    }
}

// ---------------------------------------------------------------------------
// Balancer showcase (issue #274) — feeds the `/balancers` web QA tool.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ShowcaseTemplate {
    /// High-level source — e.g. `"Raynquist (TU)"`, `"compose"`,
    /// `"Factorio-SAT"`. Set from `template_provenance(shape)`.
    source: String,
    /// Strategy descriptor for compose-baked entries, e.g.
    /// `"Lib(7, 1) → Lib(1, 2)"`. Empty for atomic entries.
    strategy: String,
    /// Optional reference URL — Factoriobin link for Raynquist's
    /// imports, etc. Empty if no link is available.
    reference: String,
    width: u32,
    height: u32,
    n_inputs: u32,
    n_outputs: u32,
    /// Stamped at origin (0, 0). Belt-tier strings are the yellow-tier
    /// defaults (`transport-belt` / `splitter` / `underground-belt`); the
    /// showcase doesn't care about higher tiers since it's a topology QA
    /// tool, not a throughput tool.
    entities: Vec<PlacedEntity>,
}

#[derive(Serialize)]
struct ShowcaseCell {
    n_inputs: u32,
    n_outputs: u32,
    library: Option<ShowcaseTemplate>,
}

/// Enumerate balancer templates for `(1..=max_inputs) × (1..=max_outputs)`
/// with provenance metadata. The showcase displays one cell per shape
/// with source / strategy / reference labels.
///
/// Single round-trip — calling per-shape would be 100+ wasm-bindgen calls
/// for the default 10×10 grid; one call instead.
#[wasm_bindgen]
pub fn balancer_showcase(max_inputs: u32, max_outputs: u32) -> Result<JsValue, JsError> {
    use spaghettio_core::bus::balancer_library::{balancer_templates, template_provenance};

    let templates = balancer_templates();
    let mut cells: Vec<ShowcaseCell> = Vec::with_capacity(
        (max_inputs as usize).saturating_mul(max_outputs as usize),
    );
    for n_in in 1..=max_inputs {
        for n_out in 1..=max_outputs {
            let library = templates.get(&(n_in, n_out)).map(|t| {
                let p = template_provenance((n_in, n_out));
                ShowcaseTemplate {
                    source: p.source.to_string(),
                    strategy: p.strategy.to_string(),
                    reference: p.reference.to_string(),
                    width: t.width,
                    height: t.height,
                    n_inputs: n_in,
                    n_outputs: n_out,
                    entities: t.stamp(0, 0, "transport-belt", "splitter", "underground-belt", None),
                }
            });
            cells.push(ShowcaseCell { n_inputs: n_in, n_outputs: n_out, library });
        }
    }
    serde_wasm_bindgen::to_value(&cells).map_err(|e| JsError::new(&e.to_string()))
}
