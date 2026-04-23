# Key Source Files

Full reference table. The most-visited files are summarised in `CLAUDE.md`.

## Rust pipeline (`crates/core/src/`)

| File | Purpose |
|------|---------|
| `models.rs` | Shared data models: `ItemFlow`, `MachineSpec`, `SolverResult`, `PlacedEntity`, `LayoutResult`, `EntityDirection` |
| `common.rs` | Shared constants and helpers (belt tiers, entity sizes, direction utils, UG reach) |
| `astar.rs` | `ghost_astar` — turn-penalty + per-axis cost A* used by the ghost router |
| `solver.rs` | Recursive recipe resolution producing `SolverResult` |
| `recipe_db.rs` | Recipe DB — loads `crates/core/data/recipes.json` via `include_str!` |
| `blueprint.rs` | Blueprint exporter (JSON + zlib + base64 envelope) |
| `blueprint_parser.rs` | Blueprint string → `LayoutResult` (reverse of `blueprint.rs`) |
| `snapshot.rs` | `.fls` layout snapshot format for debugging (see `docs/layout-snapshot-debugger.md`) |
| `trace.rs` | Structured trace event collection (thread-local, zero overhead when inactive) |
| `sat.rs` | Varisat-backed SAT solver for bus crossing zones |
| `zone_cache.rs` | On-disk memo for expensive SAT solves (native only) |
| `analysis.rs` | Post-layout analytics used by tests + snapshot tooling |

### Bus layout subsystem (`crates/core/src/bus/`)

| File | Purpose |
|------|---------|
| `layout.rs` | Top-level orchestrator: `place_rows → plan_bus_lanes → place_poles → route_bus_ghost` |
| `placer.rs` | Row placement — groups machines by recipe, splits rows for throughput |
| `templates.rs` | Belt/inserter row templates (single-input, dual-input, lane-splitting) |
| `lane_planner.rs` | `BusLane` / `LaneFamily` types, `plan_bus_lanes`, lane splitting + tap-off coordinate finding |
| `lane_order.rs` | Left-to-right lane column order optimiser |
| `balancer.rs` | `stamp_family_balancer` + splitter/UG belt-tier name helpers |
| `balancer_library.rs` | Pre-generated N→M balancer templates (do not edit manually) |
| `trunk_renderer.rs` | Path → entity rendering (`render_path`, `trunk_segments`, `is_intermediate`) |
| `output_merger.rs` | Final-product east-flowing output merger |
| `ghost_router.rs` | Ghost A* + negotiation loop, crossing set construction, junction solver integration |
| `ghost_occupancy.rs` | Typed `Occupancy` map: `HardObstacle` / `RowEntity` / `Permanent` / `GhostSurface` / `Template` / `SatSolved` |
| `junction.rs` | `Junction` / `SpecCrossing` / `Rect` / `BeltTier` — the snapshot strategies consume |
| `junction_solver.rs` | Region-growth outer loop + `JunctionStrategy` trait |
| `junction_sat_strategy.rs` | SAT-backed `JunctionStrategy` fallback |
| `tapoff_search.rs` | Brute-force search for optimal tap-off tile patterns (test/generation only) |

### Validation (`crates/core/src/validate/`)

| File | Purpose |
|------|---------|
| `belt_flow.rs` | Belt connectivity, flow paths, direction continuity, reachability, topology, junctions |
| `belt_structural.rs` | Belt structural checks (loops, dead-ends, throughput, overlaps) |
| `inserters.rs` | Inserter chain and direction checks |
| `fluids.rs` | Pipe isolation and fluid port connectivity |
| `power.rs` | Power coverage and pole network connectivity |
| `underground.rs` | Underground belt pair and sideloading checks |

## Bindings and CLIs

| File | Purpose |
|------|---------|
| `crates/wasm-bindings/src/lib.rs` | wasm-bindgen wrapper: `solve`, `layout`, `layout_traced`, `export_blueprint`, `validate_layout`, `solve_fixture`, recipe lookups |
| `crates/mining-cli/src/main.rs` | `blueprint-analyze` native CLI — reads blueprint strings from stdin/file, `--batch`/`--json` modes, expands books, prints shape summaries via `fucktorio_core::analysis` |

## Web app (`web/src/`)

Entry + shared state:

| File | Purpose |
|------|---------|
| `main.ts` | Web app entry: wires Pixi canvas, sidebar, engine, overlays, URL state |
| `engine.ts` | WASM loader + typed wrappers around `fucktorio_wasm`; emits engine-activity events; re-exported types |
| `state.ts` | `FormState` — the URL-addressable input (item, rate, machine, inputs, belt, custom inputs) |
| `state/debugState.ts` | Debug toggle state (master, step-through, validation, SAT zones, solo regions, ghost tiles, item colours) |
| `workers/engine.worker.ts` | Web Worker wrapping the WASM API so solves + layouts don't block the main thread |

Renderers (`web/src/renderer/`):

| File | Purpose |
|------|---------|
| `app.ts` | PixiJS `Application` + pixi-viewport setup (world size, zoom, pan) |
| `grid.ts` | Background tile grid |
| `graph.ts` | DAG view for the solver's recipe graph |
| `entities.ts` | Entity renderer — belts, inserters, machines, pipes, poles, sprites, hover/highlight |
| `colors.ts` | Palette tables (entity family colours, item colours) |
| `animated.ts` | Showcase modal: staggered fade-in of entities for the landing cards |
| `phaseAnimation.ts` | Phase-by-phase reveal during layout (rows → lanes → bus → poles → final) |
| `streamingRenderer.ts` | Progressive live render driven by `layout_traced` events; hands off to `entities.ts` on finish |
| `beltGraph.ts` | Belt connectivity graph for hover highlighting / network trace |
| `selection.ts` | Selection controller — click-to-select entities, draggable selection box |
| `validationOverlay.ts` | Validation markers (error/warning circles at issue coords) |
| `ghostTilesOverlay.ts` | Tiles touched by `GhostSpecRouted` — ambient debug context for SAT work |
| `satZoneOverlay.ts` | Detailed annotation layer for the selected SAT zone (bbox, crossings, pins) |
| `junctionZoneOverlay.ts` | Clickable rectangles for every junction cluster in the trace, colour-coded by outcome |
| `regionOverlay.ts` / `regionClassify.ts` | Legacy `LayoutRegion` overlay + classifier (educational: engine label vs template classifier) |
| `traceOverlay.ts` | Trace-event layer (rows placed, lanes planned, phase snapshots) |

UI panels (`web/src/ui/`):

| File | Purpose |
|------|---------|
| `sidebar.ts` | Searchable item picker, rate input, machine picker, live solve, URL state, totals |
| `landing.ts` | Landing/showcase page — recipe-ladder cards that open a modal with an animated layout |
| `inspector.ts` | Hover/tooltip overlay; surfaces entity info + tile coords + highlight controller |
| `tileContext.ts` | Per-tile aggregator: ghost specs, axis occupancy, junction cluster membership |
| `overlayPanel.ts` | Debug toggles (colours, validation, regions, SAT zones, ghost tiles, …) |
| `legendPanel.ts` | Bottom-left legend; reactive to active overlays |
| `busyOverlay.ts` | Spinner while the WASM worker is churning |
| `issuesDialog.ts` | Validation issues panel with jump-to-tile |
| `layoutTimingLog.ts` | Formatted console log of layout trace phases with timings |
| `debugPanel.ts` | Collapsible debug panel inside the sidebar |
| `timelineScrubber.ts` | Floating timeline: live milestone chips during stream, scrub-mode after completion |
| `snapshotLoader.ts` | `.fls` snapshot decoder (base64 + gzip + JSON) |
| `snapshotMode.ts` | UI wiring for loading and viewing `.fls` snapshots |
| `junctionDebugger.ts` | Inline junction-cluster panel + modal with SAT stats + veto info |
| `junctionTrace.ts` | Walks `TraceEvent[]` into per-cluster step-through records |
| `satEditor.ts` | In-canvas SAT-zone editor (Phase F): drag-to-paint belts, tier-1/tier-2 validation, accept-ghost |
| `corpus.ts` | Blueprint paste + corpus.json browser (bus detection, row pitch, shape stats) |

## Tests and examples

| File | Purpose |
|------|---------|
| `crates/core/tests/e2e.rs` | End-to-end test harness: tier 1–4 regression tests + stress corpus with scoreboards |
| `crates/core/examples/diagnose_junctions.rs` | Offline diagnostic: runs tier 2/3/4 layouts and dumps junction-solver breakdown + balancer stamps |
| `crates/core/examples/trace_junction.rs` | Replays a single junction and prints its trace timeline |
| `crates/core/examples/replay_region_trace.rs` | Replays a region-routing trace for offline inspection |
| `crates/core/examples/dump_layout.rs` | Quick layout dump helper |
| `crates/core/examples/capture_*.rs`, `inspect_fixture.rs`, `validate_ac_am3_ore.rs` | Targeted regression / capture scripts for hard cases |
