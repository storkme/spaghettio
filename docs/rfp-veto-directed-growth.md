# RFP: Veto-directed region growth for tight junction clusters

## Summary

Replace `solve_crossing`'s uniform bbox-growth heuristic with a
**walker-informed growth policy** that absorbs the tiles the walker
reports as break sites, rather than expanding +1 on every side
indifferently. The theory is that tight clusters with many nearby
crossings are unsolvable today not because they need bigger regions,
but because the current growth loop spends its tile budget in
directions the walker never cared about — and runs out of iterations
before the bbox reaches the tiles that were actually blocking the
solve.

## Motivation

The committed regression fixture
[`advanced_circuit_iron_plate_trio_capped`](../crates/core/tests/region_fixtures/advanced_circuit_iron_plate_trio_capped.json)
reproduces the failure deterministically: cluster seeds
`(21,161), (22,161), (23,161)` — three south-flowing iron-plate trunks
crossed by a westbound `ret:electronic-circuit:2:161` return — with
four more pending crossings in neighbouring clusters constraining the
context (7 crossings total in a ~10×6 tile window).

Observed runtime signature (from
`FUCKTORIO_DUMP_WALKER_VETO=1 cargo test --manifest-path crates/core/Cargo.toml --test region_fixtures -- --nocapture`):

- **28 walker vetoes** across 5 growth iterations (4 / 17 / 3 / 3 / 1).
- Every SAT strategy (`sat-surface`, `sat-1ug`, `sat-2ug`, `sat-full`)
  produces a satisfiable model in most variants; the walker
  consistently rejects them.
- Break tiles cluster on just 5 coordinates across all 28 vetoes:
  `(23,162)`, `(24,161)`, `(19,163)`, `(18,163)`, `(22,158)`.
- Terminal outcome: `JunctionGrowthCapped` with reason `iter_cap` or
  `tile_cap` (solver returns `None`).

The per-iteration growth is `expand_bbox(1, 1, 1, 1)` at
`junction_solver.rs:1092` — symmetric +1 on all sides, plus four
single-side variants tried per iter. The walker break tiles are
emitted as `TraceEvent::RegionWalkerVeto` events and then discarded
from the solver's perspective; the growth policy never reads them.

The working hypothesis, confirmed by inspecting the layout in the
browser: all the information needed to pick the right tiles is
already being produced, and we're throwing it away.

This RFP does **not** propose bigger regions, more clustering, raising
`MAX_REGION_TILES`, or reordering the ghost-router pipeline. Those are
separate (heavier) hypotheses. This RFP is the cheap test of "smarter,
not bigger" — if it works, it's the right fix; if it doesn't, we've
falsified the hypothesis and can move on to the architectural changes.

## Design

### Shape of the change

Three files touched, roughly:

- `crates/core/src/bus/region_walker.rs` — extend `WalkBreak` with an
  optional BFS dead-end tile (only populated for `Unreachable`
  reason). Added as a field so existing call-sites keep compiling.
- `crates/core/src/bus/junction_solver.rs` — change
  `try_solve_on_region` to return walker break tiles upward on
  `Vetoed`; replace the uniform `expand_bbox(1,1,1,1)` fallback with
  a `absorb_break_tiles(...)` path that adds specific tiles; keep
  uniform growth as fallback when the walker gave us no tiles (SAT
  returned `Unsatisfiable`).
- `crates/core/src/trace.rs` — new
  `TraceEvent::JunctionTilesAbsorbed { iter, tiles, reason }` so the
  debugger can render the growth decision.

### Growth policy (new outer-loop decision)

After a failed iteration:

1. Collect break tiles from every `Vetoed` strategy attempt this iter.
   Pick the **best** attempt's breaks (lowest-cost partial solution) as
   the absorb set. Break set size is naturally bounded by the number of
   participating specs (typically ≤ 6 here).
2. For each `WalkBreak`:
   - `MissingEntity` / `ItemMismatch` → absorb `tile` directly.
   - `Unreachable` with BFS-dead-end populated → absorb the dead-end
     tile plus one tile along the path toward the endpoint.
   - `Unreachable` without a dead-end → fall back to uniform +1.
3. Track absorbed-but-still-breaking tiles. If the same tile breaks
   twice after being absorbed, the absorb signal isn't helping —
   switch to uniform growth for the next iter to escape the trap.
4. Walker gave us nothing (all strategies `Unsatisfiable` rather than
   `Vetoed`) → uniform +1, same as today.

### Alternatives considered and rejected

- **Raise `MAX_REGION_TILES`** — doesn't fix the underlying waste;
  hides the problem by buying slack until the next harder fixture
  trips the new cap. Runtime hit is real (SAT cost scales with
  variable count). Tested first in a throwaway branch and not
  promoted.
- **Shared-spec union-find clustering** (merge clusters upstream) —
  doesn't help this specific fixture, because the plastic-bar
  crossings at `(25,161), (26,161)` are already
  `corridor_handled` by the time clustering runs; they're not in
  the crossing set to be merged. Addressed in a separate future
  RFP that would reorder the corridor template vs clustering.
- **SAT unsat-core extraction for growth** — valid for the
  `Unsatisfiable` branch but most of our failures are `Vetoed` (SAT
  produces a model, walker rejects it), so this would help maybe 10%
  of cases. Deferred until the cheap walker-signal path is
  exhausted.

## Kill criteria

1. **Fixture still caps after Phase 2 lands.** If
   `advanced_circuit_iron_plate_trio_capped` still returns `None`
   with `JunctionGrowthCapped` fired, the theory "the right tiles
   are reachable within current tile budget via smarter growth" is
   falsified. Abandon this RFP; escalate to the clustering /
   pipeline-ordering discussion.

2. **Paired passing fixture regresses.**
   `advanced_circuit_ret_plus_three_trunks` currently solves at
   cost 57. If its cost ratchets above 60 or it stops solving, the
   new growth policy is worse on normal junctions and must be
   abandoned.

3. **E2E suite regresses by more than one test.** Baseline on main
   today: 375 pass, 24 ignored. More than one new failure means
   veto-directed growth has unknown second-order effects on other
   fixtures and we drop it.

4. **Spike signal quality fails the first test.** If the break tiles
   dumped in Phase 0 turn out to be almost entirely path-starts
   (i.e. `Unreachable.tile` is never the actual BFS dead-end, and
   extracting dead-ends requires restructuring the walker), the
   cost of Phase 1 is too high to justify a speculative Phase 2.
   Kill and move on.

5. **Oscillation is unbreakable.** If the absorbed-but-still-breaking
   fallback still doesn't terminate within `MAX_GROWTH_ITERS` on
   fixtures that previously solved, the interaction is pathological
   and absorbed-tile growth is intrinsically unstable. Abandon.

## Verification plan

Following the [layout engine verification
protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Region-fixture suite**:
  `cargo test --manifest-path crates/core/Cargo.toml --test region_fixtures -- --nocapture`.
  Both fixtures must pass:
  `advanced_circuit_iron_plate_trio_capped` flips to `"solve"` with
  an agreed `max_cost` ratchet; `advanced_circuit_ret_plus_three_trunks`
  stays at `cost ≤ 60`.
- **Full e2e**:
  `cargo test --manifest-path crates/core/Cargo.toml`. Must not drop
  more than one test vs main.
- **Browser check**:
  http://localhost:5173/?item=advanced-circuit&rate=5&machine=assembling-machine-3&in=coal%2Cwater%2Ccrude-oil%2Ciron-ore%2Ccopper-ore&belt=transport-belt
  loads cleanly with no validation errors in the `(19..28, 159..164)`
  box. Eyeball the junction region for sensible belt layout (no
  disconnected belts, no wrong-item merges).
- **Trace inspection**: open the browser's junction debugger on the
  failing cluster; confirm the new `JunctionTilesAbsorbed` events
  render in sequence and the final region shape matches what SAT
  actually received.
- **Clippy + WASM build**: `cargo clippy --manifest-path crates/core/Cargo.toml -- -D warnings`
  and the `wasm-pack` build step must both succeed.

## Phasing

### Phase 0 — spike (≤ 30 min, no commit)

Dump `{segment_id, tile, reason, path_index_of_tile}` for every veto
on the committed fixture. For every `Unreachable`, print the affected
path's tile sequence and eyeball whether the break tile sits at the
start, middle, or end of the path. Output: a one-paragraph finding in
the Decision log below ("most breaks are ... ; BFS-dead-end extraction
is / isn't needed").

This spike decides whether Phase 1 is actually required. Kill
criterion (4) above applies here.

### Phase 1 — walker plumbing (conditional on spike)

Add `dead_end: Option<(i32, i32)>` to `WalkBreak`, populate it from
`walk_single` in the `Unreachable` branch. No behavioural change
outside the walker; existing callers ignore the new field.

### Phase 2 — veto-directed growth

Implement the growth policy above in `solve_crossing`. The capped
fixture should flip to `solve`; capture the cost and set
`max_cost` in the fixture JSON.

### Phase 3 — trace/debugger hooks

Emit `JunctionTilesAbsorbed`; extend
[`web/src/ui/junctionTrace.ts`](../web/src/ui/junctionTrace.ts)
to render absorbed tiles with a distinct colour per iteration. This
phase is a quality-of-life addition for debugging future growth
behaviour; can land separately from the correctness change.

## Decision log

- *2026-04-21 — RFP drafted. Spike not yet run. Status: proposed.*
