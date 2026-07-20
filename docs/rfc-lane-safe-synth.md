# RFC: lane-safe synth — push convergence into the graph

## Summary

Move structural lane balancing out of the placer and into synth. The
current synth produces graphs with multi-arc port relaxation —
multiple arcs converging on a single splitter input port — which
forces the placer to physically realize that convergence via
sideloading. Sideloading is hard to model correctly (per-lane caps,
side-determined lane assignments, splitter-output lane semantics) and
the place where the lane-aware-routing RFC got stuck.

The proposal: a `synth_lane_safe(n, m)` pass that takes the existing
synth output and inserts cascading balancer splitters wherever a port
has `>1` incoming arc, producing an enriched graph with no multi-arc
ports. Every belt in the resulting layout has exactly one source. The
placer can drop the sideload table entirely; lane safety becomes a
graph-level invariant rather than a layout-level one.

The deliverable is a placer that can lay out the 20 missing coprime
shapes from issue #136 by consuming the enriched synth graph as a
fully-explicit splitter network. Splitter counts grow modestly
(typically `+(K-1)` per K-arc convergence point); the trade is
strictly buying simplicity and correctness at the cost of a few extra
splitters per shape.

## Motivation

### Concrete failures the lane-aware-routing RFC couldn't resolve

- **`(1, 5)` with 3 same-side feedback sideloads.** All 3 feedback
  arcs sideload from north onto the same east-channel, all forced to
  lane 0. Per-lane cap (`0.6 > 0.5`) rejects. The lane-aware RFC's
  proposed fix — a structural balancer that absorbs all 3 — never
  fully resolved the geometry; the design doc itself ended up
  recommending 2 cascading balancers for 3 sources, at which point we
  may as well do the cascade in synth and skip the sideload modeling.
- **`(3, m)` and beyond.** Same problem at the input side: 3+ inputs
  feeding a 2-port root requires merging. Sideloading 3 inputs onto a
  single belt overloads the belt at 2× capacity. Correct fix: merge
  inputs via a (3, 2) sub-network at the front of the graph.
- **All 20 issue #136 coprime shapes.** Each has a feedback path with
  some `K ≥ 3` convergence; same lane-saturation problem in every
  case.

### Why sideloading was the wrong primitive

Sideloading in Factorio is deterministic but underspecified at the
*model* level — its rules depend on:

- The receiver belt's direction.
- The feeder's relative side.
- Whether feeder is a belt or splitter (different lane semantics).
- Whether the feeder enters head-on (back) vs perpendicular.

We tried to encode all of this in CP-SAT (the sideload table, per-lane
flow vars, source-lane forcing). Each piece worked individually but
the interactions kept producing layouts that satisfied the model but
failed in real Factorio (or vice versa). The post-mortem in
`rfc-lane-aware-routing.md` (decision log, 2026-05-02) captures the
specific gaps:

- Source-lane forcing for splitter-output drops still pending.
- Head-on splitter-output drop modeling (both lanes) pending.
- Multi-tile turn-driven lane swaps interact subtly with the above.

By construction, the lane-safe synth proposal sidesteps all four —
because it eliminates sideloading from the placer's vocabulary.

### The community-library evidence

The published Raynquist coprime balancers (the templates we're trying
to reach parity with) all use explicit splitter-cascade merging for
feedback paths. They don't sideload 3 streams onto one belt. The
"sideload everything onto a single port" pattern in our synth was a
mathematical convenience for proving rate balance, not a practical
construction. Aligning synth output with how Factorio practitioners
actually build these networks should make the placer's job
straightforward.

## Design

### `synth_lane_safe` pass

```rust
pub fn synth_lane_safe(n: u32, m: u32) -> Result<BalancerGraph, SynthError> {
    let g = synth(n, m)?;            // existing tree-with-feedback synth
    Ok(enrich_lane_safe(g))           // new pass
}

fn enrich_lane_safe(g: BalancerGraph) -> BalancerGraph {
    // For each (splitter, port) with k > 1 incoming arcs:
    //   - Allocate k - 1 fresh splitters arranged as a binary cascade.
    //   - Reroute the k incoming arcs through the cascade.
    //   - Single arc out of the cascade feeds the original port.
    // Result: every (splitter, port) and (output, _) has exactly 1
    // incoming arc.
    ...
}
```

Cascade structure for K incoming arcs: a left-leaning binary tree of
K-1 splitters, each with 2 inputs, both heading the merger direction.
The two outputs of the bottommost splitter are wasted (one feeds the
target port; the other terminates as a feedback to itself, or feeds
something downstream that needs the rate). For balance correctness,
both outputs at the cascade root must end up consumed, so the simplest
implementation tees the second output back into the cascade as a
fictitious second-port feed to the topmost splitter.

Two patterns to support:

- **Input-side merging** (e.g. `(3, m)`): n inputs → merge to fit a
  2-port root. Cascade size `n - 1`.
- **Feedback-side merging** (e.g. `(1, 5)`): K feedback arcs → 1
  combined feedback. Cascade size `K - 1`.

For the (1, 5) case the synth output grows from 8 splitters to 10:
the original 7-splitter tree + 1 merger + 2 cascade splitters in the
feedback path. Splitter counts for the 20 missing shapes will land in
the 10–18 range — modest growth but not pathological.

### What the placer changes look like

After this lands, the placer can drop:

- **The sideload table** (`_LEFT_OF`, `_RIGHT_OF` direction maps).
- **Per-route per-lane flow vars.** Reverts to single direction-only
  flow per route, since each belt has at most one route.
- **Per-lane caps.** Only per-belt caps remain (≤ 1 route per belt).
- **Dual-lane input modeling** for input boundaries — though this
  stays for correctness, since input belts at the grid boundary still
  fill both lanes by convention.

What the placer keeps:

- **Splitter direction support** (commit `0c21003`). Cascade splitters
  may be non-south.
- **Splitter no-overlap and structural constraint encoding.** This is
  the placer's actual job.
- **Per-route conservation and outflow constraints.** Reverts to the
  pre-phase-2 form.
- **Belt-count minimization objective.**

The placer's API stays the same — `_route_belts(splitter_positions,
routes, width, height)`. The shape functions need to:

- Consume the enriched graph (more splitters than before).
- Compute splitter positions for the cascade splitters in addition to
  the tree.
- Declare routes between every adjacent pair in the enriched graph.

### Worked example: `(1, 5)`

Before (synth):
- 8 splitters (1 merger + 7 tree).
- Arcs: 1 input → merger.in_0; 3 feedbacks → merger.in_1 (multi-arc);
  the rest are tree internals + 5 outputs.

After (synth_lane_safe):
- 10 splitters (1 merger + 7 tree + 2 cascade).
- Arcs: 1 input → merger.in_0; cascade.S1.in_0 = feedback_0;
  cascade.S1.in_1 = feedback_1; cascade.S1.out_0 → cascade.S2.in_0;
  cascade.S1.out_1 → ??? (terminator); cascade.S2.in_1 = feedback_2;
  cascade.S2.out_0 → merger.in_1; cascade.S2.out_1 → ??? (terminator).

The terminators are the awkward part — 2 unused cascade outputs that
would back up. Three options:

1. **Loop back into the cascade**: cascade.S1.out_1 feeds
   cascade.S1.in_0 (its own input), creating a stable loop. Rate
   balance verified by the all-fluid model — both arcs at the same
   port carry equivalent rates.
2. **Feed back into the original tree**: cascade.S1.out_1 →
   merger.in_0 alongside the input belt. Then merger.in_0 has 2 arcs
   (input + cascade leftover), reintroducing multi-arc. NOT clean.
3. **Saturate the cascade**: make the cascade a `(K, K)` Beneš
   sub-network where all K outputs feed downstream consumers. Doesn't
   fit our use case (we want K → 1, not K → K).

Option 1 is the right answer. Rate balance: the loop arc carries the
same rate as the cascade input, so the cascade is in steady state.
The verifier sees each arc with consistent rates.

Layout-wise, cascade splitters occupy a region near the merger and
fit in `~3×3` of grid. For `(1, 5)`, the existing `10×8` grid
probably accommodates with a slight widening to `12×8` or similar.

### Worked example: `(3, 4)`

Before:
- Synth produces `(3, 4)`: 3 splitters (root + 2 L1). 3 inputs all →
  root.in_0 (multi-arc).

After:
- Add 2 cascade splitters at the front: 3 inputs → 2-cascade → 1
  merged input → root.
- 5 splitters total.
- Layout: roughly mirror the (1, 5) approach but at the input side.

### Pattern for general (n, m)

```
inputs (n)            outputs (m)
   │                       ▲
   ▼                       │
input cascade          tree (existing)
   │                       ▲
   ▼                       │
root ────────────►─────────┘
   ▲
   │
feedback cascade ◀── feedback arcs (m mod 2 != 0 cases)
```

Both cascades use the K-1 splitter pattern with self-loop terminators.

### Things explicitly out of scope

- **Optimal cascade ordering.** A K-1 left-leaning cascade is one
  valid construction; there are others (balanced binary tree, etc.)
  that may produce tighter layouts. Pick one valid construction; tune
  later if benchmarks show it matters.
- **Sharing cascades across multiple convergence points.** If two
  ports each have K=3 incoming arcs, we'd build two separate
  cascades. Could share if the arcs are routable to a shared cascade,
  but doesn't generalize cleanly.
- **Cascade splitter rotation.** Treat cascades as south-facing by
  default. Non-south orientations may be needed for tight layouts but
  add complexity; defer until needed.

## Kill criteria

**Required.** Stop or rethink if any of these trip:

1. **Cascade self-loop verifier disagreement.** If `verify_balancer`
   on the enriched graph disagrees with the same on the original
   non-enriched graph (different output rates), the self-loop
   semantics aren't matching. The cascade should be transparent — its
   inputs and outputs balance to the same rates as the multi-arc
   relaxation it replaces. If they don't, the cascade construction is
   wrong.

2. **Splitter count grows by more than 1.5× over the original synth
   for any shape in `1..=10 × 1..=10`.** Cascades should add at most
   `K-1` per convergence point. Across the 99 shapes, total splitter
   growth should be `≤ 50%`. If we're seeing 2× or more growth, the
   cascade is over-engineering and we should reconsider whether to
   apply it everywhere or only where strictly needed.

3. **`(1, 5)` placer solve time > 30 minutes.** Same kill criterion
   as the lane-aware RFC — `(1, 5)` is the sentinel for the whole
   coprime track. If we can't crack it under this approach, we're
   stuck on a different problem (probably layout density, not lane
   semantics).

4. **Hard coprime (`(4, 9)`, `(7, 9)`, `(8, 9)`) placer solve time
   > 4 hours.** Same as lane-aware RFC. The coprime shapes are the
   actual deliverable.

5. **Layout footprint exceeds the existing community library by more
   than 50% for any shape that the library covers.** Adding 2-4
   cascade splitters per shape grows footprints; we accept some
   growth, but if `(4, 4)` ends up at `8×8` instead of `4×8`
   (community minimum), we're losing too much.

## Verification plan

Per the [layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).

After each phase:

1. **Synth fluid-verifier consistency.** For every shape in
   `1..=10 × 1..=10`, run `verify_balancer` on both `synth(n, m)` and
   `synth_lane_safe(n, m)`. Both should report the same per-output
   rate (`n/m` for balanced, error-out for the same shapes that the
   original errors on — `(5, 8)`, `(7, 6)`, `(8, 6)`).

2. **Topology recovery round-trip.** For every CpSat-produced template
   from `synth_lane_safe`, recover the graph via `topology_of_template`
   and confirm `verify_balancer(recovered)` matches the original
   `verify_balancer(synth_lane_safe(n, m))`. The cascade splitters
   appear in the recovered graph as regular splitters; rates should
   work out.

3. **Cross-engine bench.** Run the existing balancer engine bench
   against `synth_lane_safe` output. Compare splitter count, entity
   count, footprint vs. the existing library shapes.

4. **Browser eyeball** on a tier-4+ recipe that uses a coprime shape
   — `processing-unit @ 3/s from ore` or similar. Confirm the
   resulting bus layout looks right and stamps without errors.

5. **Solve-time tracking** per shape. Log wall-time on every shape
   in the round-trip suite; track regressions against the dyadic
   baseline shipped in the lane-aware-routing PR.

## Phasing

1. **Phase 1: cascade construction in synth.** Add
   `synth_lane_safe(n, m)` and `enrich_lane_safe`. Verify on the
   existing fluid verifier — every shape in `1..=10` produces a
   graph with no multi-arc ports and the same per-output rate as
   the unenriched synth. ~2 days. No placer changes.

2. **Phase 2: simplify the placer.** Drop the sideload table,
   per-lane vars, and per-lane caps from `_route_belts`. Revert to
   the simpler conservation model (single flow indicator per
   route/tile/dir, ≤ 1 route per tile). Verify all 10 dyadic
   round-trip tests still pass with the simpler placer + the
   enriched synth. ~1 day.

3. **Phase 3: place coprime shapes.** Update each shape function
   (or add new ones) to consume the enriched graph and emit a
   layout. Start with `(1, 5)`, then `(1, 6)`, `(1, 7)`, `(1, 9)`,
   `(1, 10)`, then non-trivial multi-input coprimes (`(3, 4)`,
   `(4, 9)`, etc.). Each shape adds 2-4 splitters in the cascade
   region. ~1 week.

4. **Phase 4: bench + library regeneration.** Run the cross-engine
   bench across all 99 shapes; compare against the community
   library. Fix any shapes where footprint regression exceeds kill
   criterion 5. Fold the regenerated templates into
   `balancer_library.rs`. Close issue #136. ~1-2 days.

## Decision log

- *2026-05-02 — RFC drafted in the same PR (#273) that ships the
  lane-aware-routing infrastructure. The lane-aware approach got
  stuck on `(1, 5)`'s structural lane balancing; this RFC captures
  the cleaner alternative — push the merging work upstream into
  synth so the placer never sees multi-arc port relaxation. The
  lane-aware infrastructure (per-lane vars, sideload table) becomes
  partially obsolete after this lands; phase 2 of this RFC rolls it
  back, keeping only the bits that are still useful (splitter
  direction support, dual-lane input modeling, rate-aware foundation).*

- *2026-05-02 — Self-loop construction tested. The single-splitter
  case (one input port empty, output→empty-port self-loop)
  verifies fluid-stable: `verify_balancer` returns full-rank with
  output rate = input rate. **However**, the multi-splitter cascade
  the RFC describes for K=3 mergers does NOT extend cleanly: any
  attempt to feed a leftover output back into a port that already
  has an input creates exactly the multi-arc relaxation
  lane-safe-synth was supposed to eliminate. Working through the
  conservation by hand:

  - Single-splitter self-loop: works because port 1 starts empty,
    the loop arc is the only arc on that port, no multi-arc.
  - 2-splitter cascade for 3-input merger (S1 absorbs fb1+fb2,
    S2 absorbs S1.out_0 + fb3, S2.out_0 → final, S1.out_1 + S2.out_1
    leftover): no port left empty for self-loops to land on without
    re-introducing multi-arc.
  - Padded Beneš (4 inputs, 1 dummy at cap 0.0): verifies fine
    (each output = 0.75) but produces 4 outputs, not 1 — doesn't
    solve the K→1 single-feedback merger that the original `(1, 5)`
    needs. We'd need an additional `(4, 1)` merger downstream,
    which is itself a multi-arc convergence.

  **Conclusion: the lane-safe-synth approach as drafted does not
  actually eliminate multi-arc convergence for K→1 mergers.** Binary
  splitters can only produce K outputs from K inputs at conserved
  rate; reducing to a single belt at the convergence point requires
  sideloading no matter how the upstream cascade is arranged.

  The path forward is to fix lane-aware routing properly — the
  problem the lane-aware-routing RFC was trying to solve. Specifically:
  source-lane forcing for splitter-output drops (head-on fills both
  lanes; perpendicular forces one lane) needs to land. With that,
  the placer can find lane-safe layouts for `(1, 5)` directly,
  using sideloading from opposite sides where geometry permits.
  This RFC is parked, not abandoned — we may revisit some elements
  (e.g., padded Beneš for `(n, n)`) in narrower contexts.*

## What this means for the in-flight PR

The current PR (`claude/cp-sat-place-dyadic`) ships:

- Phase 1 of lane-aware-routing (per-lane flow vars).
- Phase 2 (sideload table + lifted tile-exclusivity).
- Phase 3 partial (dual-lane inputs, rate-aware foundation, splitter
  directions).

These all land as built. They're still net-correct for the dyadic
shapes (no regression) and the rate-aware encoding stays useful
even after lane-safe synth lands (some convergent paths in
`(2, m)`/`(4, m)` etc. could still benefit). Phase 2 of *this*
RFC would later strip back the sideload table and per-lane caps if
benchmarks show the simpler placer is fast enough.

The lane-aware RFC's outstanding `(1, 5)` integration is closed by
this RFC's existence — the right path is `synth_lane_safe`, not
source-lane forcing or in-placer balancer insertion. Mark the
lane-aware RFC's `(1, 5)` deferral resolved in the decision log
when this lands.
