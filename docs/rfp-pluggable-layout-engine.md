# RFP: Pluggable row placement — `RowPlacer` trait + `RowInfo` trait

## Summary

Introduce two traits at the layout-engine boundary so that different row-arrangement
strategies can be swapped in without editing `placer.rs` or `templates.rs`.
The traits sit between `SolverResult` and `LayoutResult`, replacing the current
monolithic `place_rows` / `build_one_row` functions with a registered strategy.

**Scope:** row placement only. The lane planner (`plan_bus_lanes`) and ghost router
(`route_bus_ghost`) remain untouched — they consume `RowInfo` instead of `RowSpan`
directly. This RFP produces a *second `RowSpan`-shaped layout strategy* (e.g.
staggered rows), not a fundamentally different bus topology. A fully pluggable
lane planner is a follow-up RFP.

## Motivation

Today `place_rows` and `build_one_row` in `placer.rs` (together ~1,800 lines)
hardcode two row geometries:

- **VerticalSplit** — input-bottlenecked recipes split into short rows reconciled
  by an N→M balancer.
- **HorizontalStack** — one long row with K stacked input belts at the top.

Adding a third geometry means editing `place_rows`, `build_one_row`, and
`templates.rs` — and the new code can't be tested in isolation from the existing
placers. The existing `LayoutStrategy` enum (Pooled / PartitionedDecomposed) only
controls *how many modules* a multi-consumer item gets split into — the spatial
layout is identical.

We want to experiment with new row arrangements (e.g. staggered rows, compact
grouping heuristics) without touching the lane planner or ghost router.

## Design

### 1. Core traits (`bus/engine.rs`)

Two traits. Everything else is plumbing.

```rust
/// A row-placement strategy. Called by the layout orchestrator to stack
/// assembly rows vertically. Returns entities plus per-row metadata that
/// the lane planner and router consume.
pub trait RowPlacer: Send + Sync {
    /// Placement strategy name (for trace events, URL params).
    fn name(&self) -> &str;

    /// Place all rows for the given machine specs.
    ///
    /// Called twice by `layout_pass` (see §3 below). The first call
    /// uses an estimated bus width; the second uses the actual width
    /// computed from the planned lanes. The placer must produce
    /// consistent geometry given the same inputs.
    ///
    /// Returns (entities, per-row metadata, total_width, total_height).
    fn place_rows(
        &self,
        machines: &[MachineSpec],
        dependency_order: &[String],
        bus_width: i32,
        y_offset: i32,
        max_belt_tier: Option<&str>,
        final_output_items: &FxHashSet<String>,
        extra_gaps: &FxHashMap<usize, i32>,
    ) -> (Vec<PlacedEntity>, Vec<Box<dyn RowInfo>>, i32, i32);

    /// Build a single row (used by `place_rows` internally).
    fn build_one_row(
        &self,
        spec: &MachineSpec,
        count: usize,
        bus_width: i32,
        y_cursor: i32,
        max_belt_tier: Option<&str>,
        output_east: bool,
    ) -> (Vec<PlacedEntity>, Box<dyn RowInfo>, i32);
}

/// Per-row metadata consumed by the lane planner and ghost router.
///
/// `RowSpan` implements this trait. The lane planner reads these fields
/// to determine where tap-offs land, where output belts flow, how to key
/// `(item, module_id)` consumer buckets, and whether a row uses horizontal
/// stacking.
///
/// The ghost router reads `output_belt_y`, `output_belt_x_range`, and
/// `spec.inputs/outputs` (via `inputs()`/`outputs()`) for output merging
/// and producer-row lookups.
pub trait RowInfo: Send + Sync {
    fn y_start(&self) -> i32;
    fn y_end(&self) -> i32;
    fn input_belt_ys(&self) -> &[i32];
    fn output_belt_y(&self) -> i32;
    fn output_east(&self) -> bool;
    fn output_belt_x_range(&self) -> (i32, i32);
    fn horizontal_stack_info(&self) -> Option<&HorizontalStackInfo>;
    fn fluid_port_ys(&self) -> &[i32];
    fn fluid_port_pipes(&self) -> &[(String, i32, i32)];
    fn fluid_output_port_pipes(&self) -> &[(String, i32, i32)];
    fn machine_count(&self) -> usize;
    fn module_id(&self) -> u32;
    fn recipe(&self) -> &str;
    fn entity_type(&self) -> &str;

    /// Inputs as consumed by `plan_bus_lanes` (item, module_id) keying
    /// and rate computation. Returns a lightweight view — not `&[ItemFlow]`
    /// from `MachineSpec` — but the shape is the same.
    fn inputs(&self) -> &[ItemFlow];

    /// Outputs as consumed by `plan_bus_lanes` and `ghost_router`
    /// (output merging). Shape matches `MachineSpec.outputs`.
    fn outputs(&self) -> &[ItemFlow];
}
```

The `inputs()`/`outputs()` methods are the **only** fields that `plan_bus_lanes`
and `ghost_router` read from `RowSpan` that are not geometry. They expose
`ItemFlow` (item name, rate, module_id, is_fluid) — not the full `MachineSpec` —
so the trait boundary is real: a new row type can compute different input/output
profiles from its own internal data, even if `RowSpan` happens to wrap a
`MachineSpec` today.

### 2. Registry of row placers

```rust
pub struct RowPlacerRegistry {
    placers: RwLock<FxHashMap<String, Box<dyn RowPlacer>>>,
}

impl RowPlacerRegistry {
    pub fn register(&self, name: String, placer: Box<dyn RowPlacer>) { /* ... */ }
    pub fn get(&self, name: &str) -> Option<&dyn RowPlacer> { /* ... */ }
    pub fn list(&self) -> Vec<String> { /* ... */ }
}

pub fn row_placer_registry() -> &'static RowPlacerRegistry;
```

Registration at startup:

```rust
pub fn register_default_placers() {
    let reg = row_placer_registry();
    reg.register("vertical-split".to_string(), Box::new(VerticalSplitPlacer));
    reg.register("horizontal-stack".to_string(), Box::new(HorizontalStackPlacer));
}
```

### 3. Pipeline composition

`build_bus_layout` delegates row placement:

```rust
pub fn build_bus_layout(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String> {
    // 1. Strategy dispatch (partitioning) — same as today
    // 2. Estimate bus width — same as today
    // 3. PLACE ROWS — delegate to RowPlacer (called twice by layout_pass)
    let placer = row_placer_registry()
        .get(opts.row_layout.name())
        .ok_or_else(|| format!("unknown row placer: {}", opts.row_layout.name()))?;

    // layout_pass calls place_rows twice:
    //   pass 1: estimated bus width → plan lanes → compute actual width
    //   pass 2: actual bus width + balancer gaps → final placement
    let (row_entities, row_infos, row_width, total_height) = placer.place_rows(
        &solver_result.machines,
        &solver_result.dependency_order,
        actual_bw,
        y_offset,
        max_belt_tier,
        &final_output_items,
        &extra_gaps,
    );

    // 4. Plan lanes — reads RowInfo trait (NOT RowSpan directly)
    let (lanes, families) = plan_bus_lanes(solver_result, &row_infos, max_belt_tier)?;

    // 5. Place poles — same as today
    // 6. Route via ghost_router — same as today
    // 7. Merge results — same as today
}
```

**Two-pass placement:** `layout_pass` calls `place_rows` twice (see
`layout.rs:302` and `:344`), first with an estimated bus width, then with the
actual width computed from planned lanes. The trait must support re-placement
on actual width with merged retry/balancer gaps.

### 4. Module DAG (circular-dep visibility)

```
engine.rs  →  placer.rs, models.rs
placer.rs  →  layout.rs (RowLayout), common.rs, models.rs
layout.rs  →  engine.rs, lane_planner.rs, placer.rs
lane_planner.rs  →  engine.rs (RowInfo), models.rs, placer.rs (RowSpan for tests)
ghost_router.rs  →  placer.rs (RowSpan)
```

`engine.rs` depends on `placer.rs` (for `VerticalSplitPlacer` impl).
`placer.rs` depends on `layout.rs` (for `RowLayout` enum). `layout.rs` depends
on `engine.rs` (for the trait and registry). This is a clean DAG: no cycles.

`lane_planner.rs` will switch from `RowSpan` to `RowInfo` (trait object),
breaking the direct `use crate::bus::placer::RowSpan` import.
`ghost_router.rs` keeps `RowSpan` — it reads geometry fields directly and
doesn't need the trait.

### 5. Backward compatibility

- `LayoutStrategy` enum unchanged.
- `RowLayout` enum unchanged — it maps to registry names.
- `RowSpan` stays in `placer.rs` and implements `RowInfo`. The existing
  `place_rows` function becomes `VerticalSplitPlacer` behind the trait.
- The ghost router (`route_bus_ghost`) is untouched.
- `LayoutOptions` unchanged.
- The WASM API (`#[wasm_bindgen]` functions) is untouched.

### 6. What this does NOT include (out of scope)

- **A pluggable ghost router.** 4,300 lines, tightly coupled to `BeltSpec`,
  `GhostRouteResult`, `Occupancy`, `GrowingRegion`, and the SAT junction
  solver. Separate project.
- **A pluggable lane planner.** `plan_bus_lanes` is 1,100 lines. This RFP
  changes it to consume `RowInfo` instead of `RowSpan` directly, but the
  lane-planning algorithm itself stays monolithic. A follow-up RFP may
  pluggify it.
- **A pluggable solver.** Already decoupled — `SolverResult` has no spatial data.
- **A pluggable balancer library.** Pre-generated data, not a strategy.
- **Grid bus / multi-axis bus.** These require changes to the `(item, module_id)
  → consumer rows` shape that `plan_bus_lanes` builds. Out of scope.

## Kill criteria

**Required.** Unambiguous conditions under which the RFP is abandoned.

1. **Phase 1 byte-equivalence:** every tier1–4 e2e snapshot's entity list is
   unchanged. Any divergence → revert. This is the proof that the adapter layer
   is transparent.
2. **Phase 3 canary isolation:** a new placer must not edit
   `lane_planner.rs`, `ghost_router.rs`, or `templates.rs`. If it does, the
   trait is leaking and we abandon (or widen scope to a follow-up RFP).
3. **Non-canonical geometry:** a new placer must be able to produce a row with
   non-canonical `output_east` / `output_belt_x_min` / `output_belt_x_max`
   without edits outside the `engine.rs` module. This proves the trait covers
   the full output surface.
4. **WASM API surface:** no new functions exposed, no existing functions
   removed, no different serialization. The WASM layer is the public contract
   and must not break.

## Verification plan

1. **Run full e2e suite** — `cargo test --manifest-path crates/core/Cargo.toml`.
   All 9 non-ignored e2e tests must stay green with zero changes to test
   expectations (the `VerticalSplitPlacer` produces byte-identical output).
2. **Canary experiment** — implement a trivial new row layout (e.g.
   `StaggeredRowPlacer` that places rows with an alternating x-offset).
   Verify it produces a valid blueprint in the browser. If it takes more
   than a day, the abstraction is too hard to use.
3. **Check WASM API** — `wasm-pack build crates/wasm-bindings --target web`
   must succeed with no changes to `lib.rs`'s `#[wasm_bindgen]` functions.
4. **Clippy + tsc** — pre-commit hooks must pass.

## Phasing

### Phase 1 + 3 (merged): traits + canary

Phase 1 (adapter) and Phase 3 (canary) are merged because Phase 1 alone has
zero correctness signal — the only evidence the abstraction works is the
canary placer. There's nothing to bisect against until Phase 3.

- Create `bus/engine.rs` with `RowPlacer` and `RowInfo` trait definitions.
- Make `RowSpan` implement `RowInfo` (trait impl, no behavioral changes).
- Extract `place_rows` into `VerticalSplitPlacer` implementing `RowPlacer`
  (same logic, behind the trait).
- Extract `build_one_row` + template dispatch into
  `VerticalSplitPlacer::build_one_row`.
- Create `RowPlacerRegistry` with `register()` and `get()`.
- Wire `build_bus_layout` to look up the placer from `opts.row_layout` and
  delegate. Default: `VerticalSplitPlacer`.
- **Implement a canary placer** (e.g. `StaggeredRowPlacer`) that produces
  rows with non-canonical geometry. Must not touch `lane_planner.rs`,
  `ghost_router.rs`, or `templates.rs`.
- All existing tests pass with byte-identical entity lists.

### Phase 2: HorizontalStack behind a trait

- Create `HorizontalStackPlacer` implementing `RowPlacer`.
- Map `RowLayout::HorizontalStack` to the registry key.
- The `RowLayout` enum becomes a registry key selector — no match arms
  in `build_one_row`.
- Existing `HorizontalStack` layouts produce identical output.

### Phase 3: Pluggable lane planner (follow-up RFP)

- If Phase 1+3 succeeds, spec a `LanePlanner` trait in a separate RFP.
- This is a larger change: `plan_bus_lanes` is 1,100 lines and currently
  reads `RowSpan` fields directly. Decoupling it requires making the
  lane planner consume `RowInfo` instead of `RowSpan`.
- Only attempt after Phase 1+3 validates the approach.

## Trade-offs considered

| Alternative | Pros | Cons |
|-------------|------|------|
| **Keep monolithic, add enum variants** | Zero refactor | Every new layout = edit `layout.rs` + `placer.rs` + `templates.rs`. Hard to test in isolation. |
| **RowPlacer trait only (this RFP)** | Row placement is the main variation point. New layouts = new `RowPlacer` impl. | ~300 LOC of trait definitions + adapters. Minimal indirection. |
| **ECS/data-driven layout** | Maximum flexibility. Layouts are data files. | Massive rewrite. Loss of compile-time geometry guarantees. Overkill for the problem. |
| **Plugin system (dynamic loading)** | Truly open ecosystem. | WASM can't load dynamic libs. Rust's plugin system is unstable. Not worth the complexity. |

## Decision log

- *2026-04-30 — initial spec written.*
- *2026-04-30 — review feedback incorporated: added `inputs()`/`outputs()` to
  `RowInfo` (review point 1), replaced LOC/file-count kill criteria with
  byte-equivalence and canary-isolation gates (point 2), merged Phase 1+3
  (point 3), narrowed motivation to "second RowSpan-shaped strategy"
  (point 4), added two-pass placement dance and module DAG (point 5).*
