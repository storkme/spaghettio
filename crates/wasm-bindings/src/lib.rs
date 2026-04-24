//! wasm-bindgen bindings for the Fucktorio pipeline.
//!
//! Thin wrapper around `fucktorio_core` that exposes the full pipeline to the
//! browser via WASM. Loaded by `web/src/engine.ts`.
//!
//! Build: `wasm-pack build crates/wasm-bindings --target web --out-dir ../../web/src/wasm-pkg`
//!
//! Exposed functions: `init`, `solve`, `layout`, `export_blueprint`, `validate`,
//! `get_all_items`, `get_recipes_for_item`, `parse_blueprint`.

use fucktorio_core::models::{LayoutResult, PlacedEntity, SolverResult};
use fucktorio_core::validate::{self, LayoutStyle, ValidationIssue};
use fucktorio_core::{
    blueprint, blueprint_parser, bus::junction_cost::solution_cost,
    bus::layout::{build_bus_layout, LayoutOptions}, fixture as fixture_mod, recipe_db, sat, solver,
};
use rustc_hash::FxHashSet;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn solve(
    target_item: &str,
    target_rate: f64,
    available_inputs: Vec<String>,
    machine_entity: &str,
) -> Result<SolverResult, JsError> {
    let inputs: FxHashSet<String> = available_inputs.into_iter().collect();
    solver::solve(target_item, target_rate, &inputs, machine_entity)
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
pub fn layout(solver_result: SolverResult, max_belt_tier: Option<String>) -> Result<LayoutResult, JsError> {
    build_bus_layout(&solver_result, LayoutOptions::from_belt_tier(max_belt_tier.as_deref()))
        .map_err(|e| JsError::new(&e))
}

/// Traced variant of `layout()`. Returns the same `LayoutResult` plus
/// the structured `trace` events that drive the debug overlays. Ghost
/// routing is the only routing path — the legacy direct router was
/// deleted; both `layout()` and `layout_traced()` go through it.
#[wasm_bindgen]
pub fn layout_traced(solver_result: SolverResult, max_belt_tier: Option<String>) -> Result<LayoutResult, JsError> {
    fucktorio_core::bus::layout::build_bus_layout_traced(
        &solver_result,
        LayoutOptions::from_belt_tier(max_belt_tier.as_deref()),
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
fn streamable(evt: &fucktorio_core::trace::TraceEvent) -> bool {
    use fucktorio_core::trace::TraceEvent as T;
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
    emit: &js_sys::Function,
) -> Result<LayoutResult, JsError> {
    let emit = emit.clone();
    let on_event: Box<dyn FnMut(&fucktorio_core::trace::TraceEvent)> = Box::new(move |evt| {
        if !streamable(evt) {
            return;
        }
        if let Ok(js_evt) = serde_wasm_bindgen::to_value(evt) {
            let _ = emit.call1(&JsValue::NULL, &js_evt);
        }
    });
    fucktorio_core::bus::layout::build_bus_layout_streaming(
        &solver_result,
        LayoutOptions::from_belt_tier(max_belt_tier.as_deref()),
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
    use fucktorio_core::bus::region_reimprove::{
        deadline_in, descend, prune_dangling, rebuild_zone_from_region,
    };
    use fucktorio_core::models::RegionKind;

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
    let (final_raw, _stop) = descend(
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
            let evt = fucktorio_core::trace::TraceEvent::SatImprovement {
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
) -> Result<Vec<ValidationIssue>, JsError> {
    let style = layout_style.unwrap_or_default();
    let solver_ref: Option<&SolverResult> = solver_result.as_ref();
    validate::validate(&layout_result, solver_ref, style)
        .map_err(|e| JsError::new(&e.to_string()))
}
