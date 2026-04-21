# RFP: Unified belt specs — one `BeltSpec` per physical belt

## Summary

Collapse the `trunk:`/`tap:`/`ret:` spec decomposition into a single
unified `BeltSpec` per physical belt. The lane planner emits one spec
per continuous belt (a trunk and its one tap are the same belt with a
corner; they should be one spec), with routed paths that can bend.
Every downstream consumer — ghost router, junction solver, trunk
renderer, validator — sees each item flow as a single coherent
object instead of the current multi-spec decomposition.

## Motivation

The capped fixture
[`advanced_circuit_iron_plate_trio_capped`](../crates/core/tests/region_fixtures/advanced_circuit_iron_plate_trio_capped.json)
fails deterministically because of a **pinned spec handoff**.

At the cluster seed `(21..23, 161)`:

- `trunk:iron-plate:23` has its out-boundary at `(23, 161) South` — "iron-plate exits the region going south here."
- `tap:iron-plate:23:162` has its in-boundary at `(23, 162) East` with feeder `(23, 161) South` — "iron-plate enters the region already turned east, fed by a south-going belt at (23, 161)."
- `ret:electronic-circuit:2:161` has its in-boundary at `(23, 161) West` feeder `(24, 161) West` — "electronic-circuit enters the region at (23, 161) going west."

Two specs independently declare `(23, 161)` must be iron-plate-south;
a third declares it must be ec-west. The junction solver's
item-conflict pre-check fires (`(23,161) carries [electronic-circuit, iron-plate]`)
and the region grows in every iteration trying to get the conflict
into the interior, but the pin at `(23, 161)` never moves — because
both pinning specs anchor there by design.

The capped fixture stays capped regardless of region size, regardless
of growth policy, regardless of SAT strategy. The pin is upstream of
all of it.

Physically, the iron-plate flow at x=23 is **one continuous belt**:
south from the balancer → corner at `(23, 162)` → east to the last
consumer. Modelling it as two specs with a fixed handoff in the
middle gives the solver a contradiction that doesn't exist in the
actual layout.

## Design

### The new `BeltSpec`

```rust
pub struct BeltSpec {
    pub key: String,            // "iron-plate:23" — item + originating lane
    pub item: String,
    pub belt_name: &'static str,
    pub path: Vec<(i32, i32)>,  // tile sequence from source to sink; may bend
    pub source: FlowBoundary,   // where items enter (outside bus)
    pub sink: FlowBoundary,     // where items leave (last consumer, or reaches bus edge)
}

pub struct FlowBoundary {
    pub tile: (i32, i32),
    pub direction: EntityDirection,  // direction items flow
    pub kind: BoundaryKind,          // BalancerOutput, ConsumerInput, ReturnProducer, BusEdge, ...
}
```

One spec per physical belt. No `trunk:` / `tap:` / `ret:` split.

### Lane planner changes

`plan_bus_lanes` currently emits specs by role. Under A1 it emits one
spec per continuous belt:

- **Single-tap lanes** (the common case): one spec, path goes down the
  trunk column and bends east at the tap row. `source` = balancer
  output tile; `sink` = last consumer input tile.
- **Multi-tap lanes**: one spec per splitter output branch. The spec
  ending at a non-last tap has `sink.kind = BoundaryKind::SplitterInput`;
  the continuing-trunk spec has `source.kind = BoundaryKind::SplitterOutput`.
  No tree types — splitters are explicit spec boundaries, and the belt
  graph between splitters stays linear.
- **Return flows**: one spec per continuous return path (producer row
  exit → trunk merge tile). Same unification principle.

### Ghost router changes

A* currently routes `start → goal` in one shot. Under A1 the router
handles bent paths by accepting a `waypoint_hint: Option<(i32, i32)>`
on the spec, which tells A* where the known corner sits. Corner tiles
still emerge naturally if A*'s cost model prefers straight runs, but
the hint keeps the path shape predictable when the planner has a
preferred geometry. A* is otherwise unchanged.

### Trunk renderer changes

`render_path` at [`trunk_renderer.rs`] already walks a path and emits
belts per tile-pair, so bent paths work for free — it just needs to
not assume the whole path flows in one cardinal direction. The
existing `trunk_segments` helper that splits a path into straight
runs for segment-id tracking is where the change lives.

### Validator changes

Belt-flow walker's segment ids currently include `trunk:` / `tap:`
prefixes. Under A1 the prefix is just the item. Segment-id string
changes ripple through placed-entity metadata in snapshots, which is
an observable interface change — snapshots need to be regenerated.

### Junction solver changes

None, by construction. The solver's API is "given participating specs
and their routed paths, solve the junction." With specs unified, the
inputs are naturally a complete picture. The item-conflict check in
particular stops firing on pinned-handoff cases, because there is no
handoff.

### Alternatives considered and rejected

**A2 — region-scope stitching.** Keep the current `trunk:`/`tap:`/`ret:`
decomposition and insert a stitching step at cluster-formation time
that merges participating specs into unified views for the solver.
Smaller change (~100 LOC, localised to `solve_crossing`'s input prep),
delivers the same solver-side API, and naturally handles multi-tap
flows because in-region splitters are rare. **Rejected** because it
keeps the decomposition globally and defers the cleanup — the
pipeline stays two-models, and we'd eventually want A1 anyway for any
new feature that wants a coherent view of flows.

**A3 — release the handoff tiles.** Mark pre-stamped entities at
trunk-tap handoff points as releasable. Rejected because the specs'
*boundary declarations* still demand iron-plate at the handoff tile
regardless of what's physically placed there; release alone doesn't
help.

## Kill criteria

1. **Phase 1 fails to unblock the fixture.** If, after single-tap
   lanes are unified end-to-end, the committed capped fixture still
   returns `None` from `solve_crossing` (even with the Phase 2
   veto-directed growth already in place), then the spec
   decomposition isn't the root cause of the pin. Abandon A1;
   reopen the hypothesis search.

2. **Implementation balloons past ~800 LOC.** Scope estimate below
   expects 300-500 LOC across lane planner, ghost router hint,
   trunk renderer, validator. If the net diff at end of Phase 1
   exceeds 800 LOC, the scope has been misjudged and the refactor
   should be paused for re-scoping before continuing.

3. **E2E suite regresses by more than 1 test.** Baseline: 375 pass,
   23 ignored on main. More than one new failure means Phase 1 is
   breaking invariants the current tests care about, and a focused
   investigation is needed before proceeding.

4. **Paired passing fixture regresses.**
   `advanced_circuit_ret_plus_three_trunks` solves today at cost 56.
   If it solves at > 60 or stops solving under unified specs, the
   unification is producing worse layouts for the common case.

5. **Multi-tap flows (Phase 2) hit structural issues.** If emitting
   one-spec-per-splitter-output-branch turns out to need tree types
   after all — e.g. if the ghost router's sibling-aware routing
   needs simultaneous visibility of all branches — escalate before
   building tree infrastructure. Falling back to A2 for multi-tap
   specifically is a valid compromise.

## Verification plan

Following the [layout engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Region-fixture suite**: both committed fixtures pass, and the
  `advanced_circuit_iron_plate_trio_capped` fixture flips from
  `expected.mode = "capped"` to `"solve"` with a new `max_cost`
  ratchet.
- **Full e2e**: 375 pass (no regressions), ignored count unchanged
  or smaller.
- **Browser check**: the motivating URL
  http://localhost:5173/?item=advanced-circuit&rate=5&machine=assembling-machine-3&in=coal%2Cwater%2Ccrude-oil%2Ciron-ore%2Ccopper-ore&belt=transport-belt
  loads cleanly with no validation errors around `(19..28, 159..164)`,
  and the iron-plate tap at x=23 looks like a single bent belt rather
  than a two-structure handoff.
- **Multi-tap smoke check**: a recipe that produces a multi-tap
  lane (candidate:
  http://localhost:5173/?item=electronic-circuit&rate=20) still
  lays out correctly and passes validation.
- **Clippy + WASM build**: both clean.

## Phasing

### Phase 0 — scope audit (≤ 1 hour, no code changes)

Identify every call site that reads `spec.key` or checks a
`trunk:`/`tap:`/`ret:` prefix. Grep + manual audit. Output: a list
of touch points in the Decision log, a measured diff estimate, and
a go/no-go on Kill criterion (2) before we start.

### Phase 1 — unify single-tap lanes

Covers the capped fixture and the common case.

- Lane planner emits single spec for single-tap lanes.
- Ghost router accepts a waypoint hint for bent paths.
- Trunk renderer handles bent paths (likely minimal work — it
  already walks tile-pairs).
- Validator segment ids updated.
- All existing tests re-snapshot as needed.

At end of Phase 1: capped fixture should solve. If it doesn't,
Kill criterion (1) fires.

### Phase 2 — multi-tap lanes

Emit one spec per splitter output branch. Explicit splitter
boundaries at `SplitterOutput` / `SplitterInput`. No tree types.
If mid-Phase-2 this approach reveals structural issues, Kill
criterion (5) fires and we may fall back to A2 for multi-tap.

### Phase 3 — return flows

Unify `ret:` specs where the unification makes sense (continuous
flow from producer exit to trunk merge). Returns are structurally
similar to forward flows, so this should reuse Phase 1 infrastructure.

### Phase 4 — cleanup

Remove obsolete helpers, unused spec-type predicates, dead
`trunk:`/`tap:` string-handling.

## Decision log

- *2026-04-21 — RFP drafted after
  [`rfp-veto-directed-growth.md`](rfp-veto-directed-growth.md)
  (Phase 2 landed; theory falsified by kill criterion 1).
  Analysis of the capped fixture identified the spec-handoff pin
  as the root cause. User selected A1 over A2 for long-term
  pipeline cleanliness.
  Status: proposed, awaiting Phase 0 audit.*

- *2026-04-21 — Phase 0 scope audit complete. **Go.***

  **Touch points by file** (grep + manual review):

  | File | Current LOC | Est. diff | What changes |
  |---|---|---|---|
  | `bus/ghost_router.rs` | 2864 | ~150-250 | `BeltSpec` struct; spec emission at `lane_planner.rs:394-614`-adjacent block; 7× `starts_with("trunk:")` / 3× `starts_with("tap:")` prefix checks; routed_paths key format; unit tests at line 2700+ using `"trunk:X"`/`"tap:X"` mock keys |
  | `bus/ghost_occupancy.rs` | 714 | ~10-30 | 2× prefix checks at lines 429-430; docstring at line 401 re: coarse segment_ids |
  | `bus/trunk_renderer.rs` | 224 | **~0** | `render_path` already handles bent paths naturally — walks tile-pairs, emits belts with direction derived per-step from (dx, dy). A south→east corner falls out for free. |
  | `bus/region_walker.rs` | 744 | ~15 | Unit-test mock segment_ids use `trunk:`/`tap:` prefixes; production code reads `segment_id` without checking prefix |
  | `validate/belt_flow.rs` | 3789 | ~0-5 | Reads `segment_id` in 2 places; doesn't match on prefix |
  | `tests/e2e.rs` | — | ~10 | Diagnostic tests reference `ret:`/`feeder:`/`tap:` in comments + one format!; no assertions on spec key format |
  | `astar.rs` | — | ~0 | `ghost_astar` routes start→goal; bent paths emerge naturally from the cost model. No waypoint support needed. |

  **Total Phase 1 diff estimate: ~200-350 LOC across 3-4 files.** Well
  under the 800-LOC kill criterion (2); comfortable margin for surprises.

  **Key enabler — A* needs no changes.** `ghost_astar(start, goal, ...)`
  naturally produces a south-then-east bent path when that's cheapest,
  because the cost function prefers straight runs and obstacles shape the
  solution. Calling it with `start = (23, lane_top)`, `goal = (last_tap_x, tap_y)`
  gives us the bent belt without any waypoint-hint plumbing. If A* picks
  a different path shape than we want, we can add hints later — but the
  audit suggests this won't be necessary for the single-tap case.

  **Key enabler — `render_path` needs no changes.** It already renders
  belts with per-step direction; a bent path renders correctly by
  construction. This removes the biggest risk from the trunk-renderer
  touchpoint.

  **Scope-limiter for Phase 1.** Apply unification *only* when
  `lane.tap_off_ys.len() == 1` (single-tap, the common case and the one
  that matters for the capped fixture). Multi-tap lanes keep the
  current decomposed emission untouched. This isolates the Phase 1
  blast radius and defers the multi-tap splitter-branch modelling
  (Phase 2) entirely.

  **Risks identified:**
  1. **Segment-id semantics.** `ghost_occupancy.rs` treats tiles with
     `seg.starts_with("trunk:")` specially ("coarse segment id, shared
     across whole trunk"). Under unified specs the prefix becomes
     `"flow:iron-plate:23"` or similar. The audit confirms the
     check is a *class membership* test, not a literal-string
     comparison, so updating the match list is sufficient. ~5 LOC.
  2. **Snapshot drift.** Placed-entity `segment_id` strings appear in
     `.fls` snapshots. No test asserts on literal segment_id strings,
     but dumped snapshots may differ across the change. Not a
     correctness issue; regenerate snapshots post-land if needed.
  3. **A*'s bent-path behaviour.** Assumed but not proven. First
     implementation step (spike, pre-commit) is to invoke A* from
     trunk_top to tap_end and inspect the returned path for our
     fixture. If it's not bent, we'll add a waypoint hint.

  Ready to start Phase 1.*
