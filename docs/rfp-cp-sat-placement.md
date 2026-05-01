# RFP: CP-SAT placement engine for (n, m) balancer graphs

## Summary

Design a CP-SAT-based placement model that takes an abstract
[`BalancerGraph`](../crates/core/src/balancer/graph.rs) (from
[`balancer::synth`](../crates/core/src/balancer/synth.rs)) and produces a
Factorio-importable [`PlacedTemplate`](../crates/core/src/balancer/placement/mod.rs)
— grid-positioned splitters, transport belts, and underground belts that
satisfy the splitter network's connectivity and rate constraints.

The subprocess plumbing is already live (commit `5687ddf`,
`scripts/cp_sat_placer.py` ↔ `balancer/placement/cp_sat.rs`); v1 only
handles `(1, 1)`. This RFP covers the placement model itself: the
encoding of splitter footprints, belt routing, rotation, and underground
pairing as CP-SAT constraints, plus the objective function and grid
sizing strategy.

The deliverable is a placement engine that solves *every* shape `(n, m)`
in `1..=10 × 1..=10` — including the 20 coprime shapes from [#136] that
Factorio-SAT currently fails on — given a synth graph as input. Tighter
footprints than Factorio-SAT on covered shapes is a stretch goal, not a
gate.

## Motivation

### Concrete gaps driving this

- **[#136] missing coprime shapes** — `(1,9), (1,10), (4,9), (5,9), …`
  through `(10,9), (10,10)`. Factorio-SAT's net-free mode times out on
  these. Our [synth](../crates/core/src/balancer/synth.rs) produces
  abstract graphs for all of them (verified balanced for every `(n, m)`
  in `1..=10` via the `synth_every_shape_up_to_10x10` test), but they
  can't be *placed* without a working engine.
- **[#135] oversize balancers** — Factorio-SAT optimizes for what it
  can express in pure SAT; CP-SAT's richer global constraints
  (`no_overlap_2d`, `circuit`, table) may find tighter encodings on the
  shapes the library does cover.
- **Pipeline cleanliness** — increment 5 of the
  [graph-synthesis decoupling plan](../crates/core/src/balancer/) wants
  to delete `src/bus/balancer_library.py`,
  `scripts/generate_balancer_library.py`, and the `factorio_balancers`
  external dep. That depends on having an engine that can regenerate
  the library deterministically.

### Why CP-SAT over Factorio-SAT

Factorio-SAT discovers *both* the splitter network *and* the placement
in one SAT solve, which is why it stalls on hard coprime cases.
Decoupling synth from placement (the pre-condition for this RFP)
collapses the placement search space to "lay out a *given* graph"
rather than "find any working graph + lay it out." CP-SAT's global
constraints encode this restricted problem natively:

- `add_no_overlap_2d` for splitter rectangles (vs. tile-by-tile SAT
  variables).
- `add_circuit` for belt routing (vs. emergent connectivity from
  per-tile direction variables).
- `add_allowed_assignments` for rotation→port-position lookups.

These run on the same OR-tools backend the `cp_sat` Rust crate would
use; we're going through the Python wheel because it ships the C++
libs bundled (decision and tradeoffs documented in commit `5687ddf`).

## Design

### Inputs

```
Request {
    graph: BalancerGraph,    // from synth(n, m) — splitters, edges
    n: u32, m: u32,
    timeout_ms: u64,
    seed: Option<u64>,
    // Future: max grid bounds, tier (belt speed), etc.
}
```

`graph.n_splitters` is the count to place. `graph.arcs` enumerate the
required connections; each arc has a `Source` (input port or splitter
out-port) and a `Sink` (output port or splitter in-port).

### Output

```
PlacedTemplate {
    width, height,
    entities: Vec<{name, x, y, direction, io_type}>,
    input_tiles: Vec<(x, y)>,    // where graph inputs enter the grid
    output_tiles: Vec<(x, y)>,   // where graph outputs leave the grid
}
```

`name ∈ {"transport-belt", "splitter", "underground-belt"}`. Direction is
Factorio 1.0 8-way (0=N, 2=E, 4=S, 6=W). Underground belts have
`io_type ∈ {"input", "output"}` distinguishing entry from exit ends.

### CP-SAT encoding

**Splitters** — each splitter `s ∈ 0..n_splitters` gets:

- `(x[s], y[s])`: integer position of the splitter's anchor tile (0..W,
  0..H bounds).
- `dir[s] ∈ {0, 1, 2, 3}`: rotation index (N/E/S/W).
- The splitter occupies a 2-tile rectangle: anchor + (offset by `dir`
  using a table constraint mapping `dir → (dx, dy)` of the second tile).
- `add_no_overlap_2d` over all splitter rectangles ensures none collide.

**Belt tiles** — model the grid as a `(W × H)` array of tile-state
variables. Each tile `t` gets:

- `belt_kind[t] ∈ {empty, belt, ug_in, ug_out, splitter}`.
- `belt_dir[t] ∈ {0, 1, 2, 3}` (only meaningful when `belt_kind ≠ empty`).
- `flow_carried[t] ∈ 0..max_arcs`: which logical arc passes through (0
  reserved for "none").

Connectivity:

- For each `belt` tile with direction `d`, the downstream tile (in
  direction `d`) must have a `belt_kind ∈ {belt, splitter, output}` that
  accepts a connection from `d`. Encoded via per-tile table constraints
  on neighbor pairs.
- Splitter ports: the in-tile and out-tile of each splitter port (two
  per side, derived from `(x[s], y[s], dir[s])` via table) must be the
  endpoint of an incoming or outgoing belt.

**Underground belts** — each underground pair (entry, exit) is modeled
as:

- An `ug_in[t]` tile with direction `d`, an `ug_out[t']` tile with
  direction `d`, where `t' = t + k·d` for some `k ∈ 1..ug_max_reach`
  (5 for yellow, 7 for red, 9 for blue).
- Tiles between `t` and `t'` may carry other belts or be empty (UG
  belts pass under).
- `add_allowed_assignments` enforces the (entry, exit) pairing rules.

**Routing** — for each arc in the graph, a path from source-tile to
sink-tile through the belt-grid:

- Encoded as a `add_circuit` over a routing graph where nodes are
  (tile, direction) states and edges are valid transitions.
- Or, simpler v1: per-arc variable for "which tiles does this arc
  pass through" with adjacency + non-overlap constraints (each tile
  carries at most one arc unless it's a splitter or a UG bridge).

**Sideloading** — multiple arcs can land on the same splitter input
port (the synth's multi-arc relaxation). Belt-level: two upstream
belts merge onto a single belt feeding the splitter. Encoded as
"in-tile of splitter port may have ≥1 incoming neighbor belt with
direction pointing inward."

### Objective

Primary: minimize bounding box area `W * H`.

Secondary (lexicographic): minimize total entity count (fewer belts
preferred over longer routings).

Approach: solve hierarchically — first find any feasible placement,
then minimize `W * H`, then minimize entities. CP-SAT handles
lexicographic minimization via repeated solves with constraints
tightened on each.

### Grid sizing

Two strategies:

1. **Bound-and-grow**: start with `W = ceil(2·sqrt(n_splitters))`,
   `H = ceil(2·sqrt(n_splitters))`. If UNSAT, double both and retry,
   up to 50×50 ceiling.
2. **Fixed bound**: pick a generous fixed bound (e.g., 30×30 for
   tier-≤8) and let the optimizer minimize area within it.

(2) is simpler and likely fast enough for tier-≤8. (1) is needed if
solve time blows up at the larger bounds.

### Phasing

Plan to land in chunks. Each phase ships a working subset and moves
the bench's "shapes covered by CpSat" number up.

1. **Phase 1: dyadic `(1, m)` for `m ∈ {1, 2, 4, 8}`.**
   Hardcoded geometry (complete-binary-tree column layout, no real
   CP-SAT search). Validates the entity-emission path end-to-end:
   Rust spawns Python, Python emits valid PlacedTemplate, Rust passes
   it back, the existing `bus::balancer_classify` recovers a graph
   from it that matches the input synth graph. ~1 day.

2. **Phase 2: minimal real CP-SAT — `(2, 2)` and `(4, 4)` Beneš.**
   Splitter rectangles only (no belt routing yet); belts emitted by a
   fixed post-process from synth's port assignments to splitter
   positions. Tests `no_overlap_2d` and rotation table constraints.
   ~2 days.

3. **Phase 3: belt routing via `add_circuit`.**
   Replace the post-process with real CP-SAT belt routing for arcs
   between splitters. UG belts deferred. Should solve all dyadic
   `(1, m)` and `(n, n)` shapes ≤ tier-8. ~2-3 days.

4. **Phase 4: underground belts + sideloading.**
   Adds UG-pair encoding and multi-arc port handling. Closes coverage
   over all `(n, m)` in `1..=10`, including the 20 missing shapes
   from [#136]. ~2 days.

Phase 1 lands within a week; phase 4 ships when it ships.

## Kill criteria

**Required.** Stop or rethink if any of these trip:

1. **Solve time > 10 min on `(4, 4)` after phase 3.** Factorio-SAT
   solves `(4, 4)` in seconds with kissat; if our CP-SAT encoding takes
   100x longer on a shape this small, the encoding is fundamentally
   wrong (most likely belt routing is over-constrained or under-
   propagated). Don't try to optimize further — go back to the model.

2. **Any tier-≤8 shape needs a grid > 50×50 to find ANY feasible
   solution.** Factorio-SAT keeps everything ≤30×30 in this range. If
   our model can't find solutions in similar bounds, we're missing a
   global constraint (probably circuit/connectivity) and the placer
   produces sprawling layouts that won't fit any reasonable bus.

3. **>25% of placed templates fail the existing bus-engine validators**
   (lane connectivity, belt flow, inserter direction —
   `crates/core/src/validate/`). The model is missing a Factorio
   semantics rule. Track which validator fails most often as a guide
   to which constraint is missing.

4. **Footprint > 2x the existing library on the 63 covered shapes.**
   A constructive engine that's twice as big as the SAT-baked library
   is worse for the bus generator than just keeping the library; the
   project goal is at parity or tighter.

5. **CP-SAT solve nondeterministic across same-seed runs.** If we can't
   reproduce a placement given the same `(graph, seed)`, the engine
   isn't usable as a build-time library generator (needed for
   increment 5 cleanup). Should be free if we set `random_seed` and
   `num_workers=1` in solver parameters; if it isn't, OR-tools
   limitation may force us back to Factorio-SAT.

## Verification plan

Per the [layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).

After each phase:

1. **Cross-engine bench** — run
   `cargo test --test balancer_engine_bench -- --nocapture` to compare
   CpSat output against LibraryLookup on the 63 covered shapes. Track
   splitter count, entity count, bbox area per shape.

2. **Recovery round-trip** — for every CpSat-produced template, run
   `bus::balancer_classify::recover_graph` on it and confirm the
   recovered graph is flow-equivalent (via `verify_balancer`) to the
   input synth graph. This catches placement bugs that produce valid-
   looking but disconnected layouts.

3. **Bus-engine integration** — for the issue [#136] missing shapes,
   patch the placed CpSat template into a fresh `balancer_library.rs`
   and run the existing tier-4 e2e tests
   (`tier4_advanced_circuit_from_ore_*`). The shapes must stamp into
   a complete bus layout that passes all 23 validators (or
   regress-test against the documented baseline if some validators
   were already failing).

4. **Browser eyeball** per CLAUDE.md — open
   `http://localhost:5173/?item=processing-unit&rate=3&in=ore` and
   visually confirm the balancer regions match what Factorio would
   build. The `(4, 9)` snapshot is the canary.

5. **Determinism check** — run the same `(graph, seed=42)` 10 times
   and assert the resulting `PlacedTemplate` is byte-identical
   each run. Tied to kill criterion #5.

6. **Full e2e** — `cargo test --manifest-path crates/core/Cargo.toml`
   stays green. The CpSat engine is build-time only; runtime path
   is unaffected unless we explicitly wire it in (separate RFP).

## Decision log

- *2026-05-01 — RFP drafted. Subprocess plumbing landed in `5687ddf`;
  this RFP scopes the placement model itself. Phase 1 starts when
  approved.*
- *2026-05-01 — Phase 1 shipped on branch `claude/cp-sat-place-dyadic`.
  6 shapes covered: `(1, 1), (1, 2), (2, 1), (2, 2), (1, 4), (1, 8)`,
  all round-tripping synth → CpSat → topology_of_template →
  verify_balancer at the expected `n/m` rate. Layout details:
  - `(1, 4)`: library-style tight-stack with 1-col stagger between
    root and level-1; 3 splitters, 5 belts, 4×4 grid.
  - `(1, 8)`: adds one routing row between root and level-1; bottom
    two splitter levels still tight-stack; 7 splitters, 13 belts,
    8×6 grid.
  Underground belts deferred — not needed at depth ≤ 3. UGs become
  useful for depth ≥ 4 (multi-row routing) and for crossing flows
  in the asymmetric coprime shapes (#136); both phase 3+ territory.*
