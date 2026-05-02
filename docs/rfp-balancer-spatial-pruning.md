# RFP: Spatial pruning in `solve_pure_routing_circuit`

## Summary

`solve_pure_routing_circuit` (the Phase 4.4 CP-SAT junction solver in
`crates/balancer-gen/scripts/place.py`) creates per-edge belt and UG
arc variables for **every** cell in the junction grid. For tonight's
(5, 9) Clos compose at jh=9 that's 15 edges × ~2200 vars/edge ≈ 33K
booleans, most of which represent paths the edge cannot plausibly
take (cells nowhere near either endpoint). The solver burns time
propagating constraints over those dead variables before concluding
they're zero.

Restrict per-edge variable creation to cells within Manhattan distance
*D* of either the source or destination, where *D = manhattan(src,
dst) + slack*. Cells outside the corridor are implicitly self-looped.
This is a standard trick for routing-style CP-SAT problems and is
called out in the original `circuit-encoding-spike-handoff.md` as the
30-50% var-count reduction lever.

## Motivation

### Concrete failure

(5, 9) Clos compose, jh=9, with the (now-correct) circuit encoding:
returned **UNKNOWN** after 645s wall (timeout 600s — overran the
budget). Solver couldn't find a layout *or* prove infeasibility within
budget. compose_series treated UNKNOWN as "try jh=10", which found
OPTIMAL after another 23 minutes. Total wall for the shape: ~24
minutes.

We don't know whether jh=9 is genuinely infeasible or just hard for
the solver to decide. Either way, the symptom is the same: the
encoding is too big to reason about quickly.

Variable counts at the largest grids tonight:

| shape | grid | edges | belt vars | UG vars | total per edge |
|-------|------|-------|-----------|---------|----------------|
| (4, 9) jh=9 | 12×9 | 12 | 391 | 976 | 1,474 |
| (5, 9) jh=10 | 15×10 | 15 | 493 | 1,300 | 1,927 |
| (9, 9) jh=10? | 27×10 | 27 | 1,000+ | 3,000+ | 4,500+ |

For (9, 9) we'd hit ~120K booleans. CP-SAT will not be happy.

### Why this matters for "going larger"

The compose-from-atoms approach is the path to tier-5/6 shapes. Each
new tier needs bigger compositions: tier-4 needs (n, 9), tier-5 will
likely need (16, 8) / (24, 4) / etc. Variable count grows as
`width × height × edges`; spatial pruning bounds the corridor per
edge regardless of how big the global grid gets, so the per-shape
search cost stays roughly constant in junction width.

## Design

### Where the change lives

Single Python function: `solve_pure_routing_circuit` in
`crates/balancer-gen/scripts/place.py:1597`. No Rust changes.

### Algorithm

For each edge e with `src=(sx, sy)`, `dst=(dx, dy)`:

1. Compute the per-edge corridor mask. A cell `(cx, cy)` is **in
   corridor** iff:

   ```
   manhattan((cx, cy), (sx, sy)) + manhattan((cx, cy), (dx, dy))
       <= manhattan((sx, sy), (dx, dy)) + slack
   ```

   This is the classic "ellipse in Manhattan distance" — the set of
   cells that lie within a detour of `slack` of the shortest path.
   Slack starts as a request parameter (`req["routing_slack"]`),
   default `slack = max(width, height) // 2` so a single edge can
   reasonably weave around obstacles.

2. Only create `belt_vars[(cx, cy, d, e)]` and
   `ug_vars[(cx, cy, d, L, e)]` if `(cx, cy)` is in-corridor.

3. **Always include src, dst, and the dst's south-belt → exit-node
   arc.** The corridor formula already guarantees src and dst are
   in-corridor (they're the foci of the ellipse), but make this
   explicit as a safety check.

4. Out-of-corridor cells get only the self-loop arc, as today. Their
   `belt_vars` and `ug_vars` are absent — equivalent to forcing them
   to 0, but with no variable in the model.

5. The at-most-one constraint at out-of-corridor cells reduces to "no
   terms", so it's skipped (today's code already guards `if terms`).

6. UG-pairing constraints: only iterate over UG vars that exist (the
   existing loop iterates `ug_vars.items()`, so naturally restricted
   to in-corridor UGs).

### Slack tuning

Three knobs:

- `slack=0`: only cells on a shortest Manhattan path. Likely too
  tight — paths must detour around other edges' entities.
- `slack=jh`: roughly "one full jump out and back" per edge. Probably
  a reasonable default.
- `slack=∞`: the current encoding. Fall back here if pruning makes
  the model infeasible.

Initial implementation: `slack = jh + 2`. Tune via experiment.

### Fallback on infeasibility

If the pruned model returns INFEASIBLE, retry with `slack = ∞` (full
encoding) before reporting infeasibility upstream. This guards
against the heuristic killing real solutions.

### Trade-offs considered

- **Tiered slack (start narrow, widen on infeasibility).** Possible
  refinement; deferred. The single-fallback "narrow then full" is
  simpler and probably captures most of the win.
- **Per-edge dynamic slack based on edge complexity.** E.g. edges
  that cross many other edges get more slack. Out of scope for the
  initial RFP; would be a follow-up if a fixed slack proves
  insufficient.
- **Ellipse vs L-shaped corridor.** The Manhattan ellipse is the
  natural choice for L1-distance routing. An L-shaped corridor
  (rectangle bounding-box + slack) is simpler but admits more
  obviously-dead cells in the corners. Stick with the ellipse.

## Kill criteria

- **Pruned encoding is INFEASIBLE on a shape that was feasible
  with the unpruned encoding** (i.e. (3, 9), (4, 9), or any
  successfully-baked shape now becomes INFEASIBLE under pruning).
  That means the heuristic is broken — either slack is too tight
  for the corridor formula, or the formula itself misses valid
  paths. **Bump slack first; if `slack=∞` fixes it, the formula is
  fine and the slack default is too tight. If `slack=∞` is still
  INFEASIBLE, the corridor formula is wrong and we abandon the
  approach entirely.**

- **Pruned encoding doesn't shrink (5, 9) jh=9 wall time by ≥2× at
  any slack tuning.** Then the dead variables aren't where the
  solver is spending time — pruning is decorative, not load-bearing.
  Drop the work.

- **Adding the corridor check makes (4, 9) jh=9 *slower* by >25%.**
  Possible if the per-cell mask cost dominates for tight grids.
  Implementation needs to be low-overhead (precompute mask once per
  edge, not per-loop-iteration).

- **Pruning unlocks (5, 9) jh=9 = OPTIMAL but the layout is
  measurably worse** (more entities, more UGs, longer path) than the
  jh=10 layout we get without pruning. Then we're trading solve time
  for layout quality. Re-evaluate by stamping both in-game and
  comparing throughput-under-load. If layouts are equivalent in
  practice, ship the pruned/jh=9 version (smaller bus footprint).

## Verification plan

Per [the layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Smoke test the existing repros.** The minimal 4×6 swap and the
   4×3 swap should both still solve OPTIMAL after pruning. If either
   regresses to INFEASIBLE the kill criterion is tripped.
2. **Re-run the (2, 2) and (4, 9) Clos compose tests** (`FUCKTORIO_DEBUG_2_2=1`,
   `FUCKTORIO_DEBUG_4_9=1`). Both should still classify Balanced.
   Compare wall times: (4, 9) is the headline benchmark; aim for
   <60s wall (vs tonight's 209s).
3. **Re-run the bake batch** (`FUCKTORIO_BAKE_BATCH=1
   FUCKTORIO_PURE_ROUTING_ENCODING=circuit`). All 7 (n, 9) shapes
   that we couldn't finish tonight should now complete in <2 hours
   total (vs the projected 3 hours unpruned).
4. **Spot-check (5, 9) at jh=9 specifically.** This is the case that
   returned UNKNOWN. With pruning, expect either OPTIMAL (great) or
   INFEASIBLE-then-jh=10-OPTIMAL (acceptable, tells us jh=9 was
   genuinely infeasible).
5. **Variable-count sanity log.** Add a debug print of the per-edge
   var count under pruning so we can see the actual reduction in
   each shape (expect 30-50% per the original handoff's estimate).
6. **Trace events.** No new ones.
7. **Clippy clean** (Rust unaffected).

## Phasing

Single phase. ~150-200 LOC change in `place.py`. No Rust touches.

## Decision log

- *2026-05-02 — drafted. Awaiting approval. Can run in parallel with
  the bake-lane-validation RFP (this one touches Python, that one
  touches Rust orchestration; zero merge-conflict surface).*
