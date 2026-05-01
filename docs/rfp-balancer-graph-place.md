# RFP: Logical-graph + placement-only balancer solver

## Summary

Generate balancer layouts for coprime `(m, n)` shapes by **decomposing
graph discovery from grid placement**:

1. **Rust topology generator** — given `(m, n)`, emit a logical splitter
   graph (universal-balancer with back-loops sized to introduce the right
   non-binary denominator for `m/n`).
2. **Rust placement solver** — given the graph + a bounding box, encode
   placement as a SAT problem and solve with `varisat` (already in tree).
3. **Cache + integrate** — produced templates are verified by
   [`balancer_classify::classify`](../crates/core/src/bus/balancer_classify.rs)
   and inserted into the same `OnceLock` lookup the phase-2.0 generator
   uses.

The premise: today's Factorio-SAT pipeline searches topology *and*
placement jointly, which is why it falls over on tier-9/10 and is the
reason `rfp-balancer-runner.md` exists. If we hand SAT a *known* graph
and ask only for a placement, the search shrinks dramatically — and we
can do it inside the Rust process with no Python dependency.

## Motivation

### What this unlocks

Phase 1 audit ([`rfp-throughput-priority-merges.md`](rfp-throughput-priority-merges.md)
decision log) and the impossibility argument in the throughput-priority
discussion established that:

- For divisible `(m, n)`, the phase-2.0 generator already wins (60-94%
  footprint reductions, dramatic).
- For **coprime** `(m, n)` (gcd = 1, denominator not a power of 2), no
  pure-feedforward 50/50 splitter network can achieve exact balanced
  rate — proven by the binary-fraction summation argument. Back-loops are
  *required*.

The library currently ships back-loop templates for coprime `(m, n)` ≤ 8,
generated offline by Factorio-SAT. For shapes outside that envelope (and
for the audit-flagged latent bugs in `(5, 8)` / `(8, 6)` from
[#266](https://github.com/storkme/fucktorio/issues/266)), the choice is:

- Wait for the SAT-runner work ([`rfp-balancer-runner.md`](rfp-balancer-runner.md))
  to finish a tier-9/10 corpus. Costs overnight wall-clock per tier; still
  produces 100-200 entity templates per coprime shape.
- Or build the decomposed approach proposed here. Order-of-magnitude
  faster for the placement step (the topology is already known), so per-
  shape generation drops from minutes-to-hours to milliseconds-to-seconds.
  Runs in the same Rust process — no Python dependency.

### Why it should be smaller

Factorio-SAT today minimises width/height of an unknown topology over an
unknown placement. The combined search space is enormous, so the solver
often settles for any solution that meets minimum bounds — not
necessarily an entity-count minimum.

With topology fixed (we choose the *smallest* universal-balancer
topology), placement is a tighter problem with a known lower bound on
entities (the graph's edge + node count). The solver can search smaller
bounding boxes more aggressively. Plausible target: same shapes the
library has today at 80-100% of current entity count (i.e. comparable),
but generated at runtime in <1s with no offline pipeline.

If the savings turn out to be marginal, the *generation cost* alone is
still a meaningful win — closing the rfp-balancer-runner work without
the SAT subprocess infrastructure.

## Design

### D1 — Topology recipe

For coprime `(m, n)`, the topology must produce rates with denominator
`n` (or a multiple of it). Two candidate constructions:

**D1a — Clos-with-residue back-loop.**
- Stage 1: `m` parallel `1→k` trees, where `k = ceil(n/m)` (or `floor`).
- Stage 2: cross-mixing layer reducing `m·k` belts to `n`.
- Back-loop: only on the residue belt(s) where the rate doesn't divide
  cleanly. Routes overflow back to stage 1.
- Estimated splitter count: `O(m + n)` for the trees + a small constant
  for the back-loop.

**D1b — Stabiliser-and-distributor (universal-balancer textbook form).**
- A single recirculating loop with back-pressure-balanced ports.
- The loop's denominator depends on the loop length; chose to give `n`.
- Estimated splitter count: `O(log_2 n)` per fan-out path with a
  `n`-cycle stabiliser at the core.

Both are well-known constructions. D1a is easier to verify
(decomposable into known atoms); D1b is potentially smaller but harder
to construct mechanically.

**Default: D1a** — start with the easier-to-verify form. Once the
classifier confirms it's MX3 (or MX2a, depending on whether back-loop
position respects composition), measure entity count vs library. If
significantly larger, switch to D1b.

### D2 — Placement encoding

Variables (boolean, fed to varisat):

- For each graph node `v` (a splitter or port) and each grid tile `(x, y)`:
  `place_v_at_xy` — is node `v` placed at tile `(x, y)`?
- For each graph edge `(u → v)` and each tile `(x, y)`:
  `belt_at_xy_for_edge` — is there a belt for this edge at this tile?
- Direction of each belt at each tile.

Constraints:

- Each node placed exactly once (one-hot over grid tiles).
- No two entities at the same tile.
- For each edge, the belt sequence forms a path from `u` to `v`.
- For each splitter, the second tile is at the appropriate offset (per
  `splitter_second_tile`).
- Belts respect direction continuity (S2-S5 from
  [`docs/factorio-mechanics.md`](factorio-mechanics.md)).
- Underground belts pair correctly when used (U1-U5).

Bounding box: a search loop tries `(W, H)` from `(min_w, min_h)` upward,
solving each. First SAT result wins. The min_w/h are derived from the
graph (entity count, max fan-out, etc.).

### D3 — Topology generator

Input: `(m, n)`.
Output: `LogicalGraph { nodes, edges }` where:
- `nodes`: list of `Splitter { in: [edge_id; 2], out: [edge_id; 2] }`,
  plus `InputPort(i)` and `OutputPort(j)` boundary nodes.
- `edges`: directed connections.

This is a pure data-structure construction in Rust; the math (placing
back-loops to give the right denominator) is the entire algorithmic
content. No Factorio-specific concepts — just abstract graph.

The topology generator can be unit-tested in isolation by piping its
output through the existing `classify_ref` (which already accepts
graph-structured input via `BalancerTemplateRef`). If the topology
classifies MX3, the topology is correct.

### D4 — Integration

The placement solver returns `Option<OwnedTemplate>` (or None on
infeasibility / timeout). The phase-2.0 generator's `generate(m, n)`
function gets a new code path: if the divisible case doesn't apply,
try the topology+placement path. Cache results in a `OnceLock` keyed
by `(m, n)`.

`balancer_templates()` is left untouched — the generator wraps
everything. Existing call sites in
[`stamp_family_balancer`](../crates/core/src/bus/balancer.rs) get
broader coverage with no API change.

### Trade-offs considered

- **Custom Rust placer instead of SAT.** Tempting (no SAT bottleneck,
  no constraint encoding), but the placement problem is genuinely
  combinatorial — handling underground-belt crossings and
  splitter-tile-overlap shortcuts in a hand-rolled placer is complex.
  varisat is already in tree, the encoding is well-understood (we did
  it for junction zones), and the placement search is small enough that
  SAT performance won't be a bottleneck.
- **CP-SAT instead of pure SAT.** No Rust binding for OR-Tools that's
  in tree. Adding one is its own dependency-management work. Defer
  unless varisat hits perf walls.
- **Reuse Factorio-SAT's placement primitives via Python subprocess.**
  Brings back the dependency we're trying to escape. Possible as a
  shortcut for D2 prototyping but not for the shipped path.
- **D1b instead of D1a as default topology.** Smaller, but less
  verifiable. If D1a runs into entity-count problems, switch.

## Kill criteria

- **Topology generator can't produce a verified MX3 (or MX2a) for any
  coprime shape.** If `classify_ref` reports MX1 or fails for the first
  topology we generate, the math is wrong. Stop and re-examine.
- **`(4, 9)` placement entity count > 200.** That's worse than what
  Factorio-SAT typically produces for coprime tier-≤8 shapes
  (100-150 range). If we can't beat that, this approach has no value
  prop and we revert to the SAT-runner pipeline.
- **`varisat` solve time per shape > 30s.** Above this, runtime
  generation isn't viable; we'd have to fall back to offline / cached
  generation. Not necessarily a kill, but pushes us toward an
  offline-only flow which complicates the build pipeline.
- **WASM bundle size grows by >200 KB.** varisat is already shipped, so
  the new code is mostly the topology generator. If we balloon the
  bundle materially, the web app's interactivity suffers.
- **Placement solver loops on a known-good graph.** If we hand-construct
  a topology we know is placeable (e.g. round-trip the library's
  existing `(2, 3)` template through the placement solver) and the
  solver doesn't return a result, the encoding has a bug.

## Verification plan

1. **Topology unit tests.** For each coprime `(m, n)` for `m, n ≤ 6`,
   run the topology generator → `classify_ref`. Class must be MX3 or
   MX2a. Composition matrix must satisfy mass balance.
2. **Placement round-trip.** For each library template `T`, extract its
   logical graph (we have `recover_graph` from
   `balancer_classify.rs`), feed to the placement solver. Solver must
   produce a placement (not necessarily identical to `T`) that
   `classify_ref` confirms is the same class as `T`.
3. **Beat-Factorio-SAT spot-check.** Generate `(4, 9)`, `(5, 9)`,
   `(7, 9)`, `(5, 7)`. Compare entity count + footprint against
   Factorio-SAT (run that pipeline once for these shapes as a baseline).
   Report savings or losses in the decision log.
4. **End-to-end layout.** Pick a tier-4 layout that needs `(4, 9)`
   somewhere. Stamp the generated template via
   `stamp_family_balancer`. Run the full e2e suite. Confirm the
   `partition_strategy_scoreboard` doesn't regress beyond the existing
   pre-existing failure.
5. **Trace events.** `BalancerGenerated` (already exists) emits with
   `class` and `entity_count`. The snapshot debugger can show generator
   reach in real layouts.
6. **Clippy + WASM.** Per the layout-engine verification protocol —
   no new clippy warnings, WASM build green.

## Phasing

- **Phase 3.0 — topology generator only.** Land the Rust-side topology
  emitter as a new module `bus/balancer_topology.rs`. Verify via
  `classify_ref` over all coprime shapes for `(m, n) ≤ 6`. *No
  placement yet.* Output is logical graph data only. ~200-400 LOC.
- **Phase 3.1 — placement solver scaffold.** New module
  `bus/balancer_place_sat.rs`. Encode placement constraints, run
  varisat, decode result into `OwnedTemplate`. Validate via the
  round-trip test (phase 1 case). ~500-800 LOC.
- **Phase 3.2 — beat-Factorio-SAT comparison.** Run on the headline
  shapes (`(4, 9)`, etc.). Decide whether kill criteria are tripped or
  this approach is shipping.
- **Phase 3.3 — integrate.** Wire into `generate(m, n)` after the
  divisible path. Add the `OnceLock` cache. Run full e2e + WASM
  smoke.
- **Phase 3.4 (optional) — replace library coprime entries with
  generated equivalents.** Phase 2.1-style work; gated on phase 3.3
  stability.
- **Out of scope:** lane-aware MX5 verification, mixed-content buses,
  D1b topology variant (deferred unless D1a misses kill criteria).

## Open questions

- **Does `varisat` handle the placement encoding fast enough?** The
  junction-zone solver uses varisat on much smaller problem sizes.
  Placement of a 30-node graph in a 12×12 grid is in the same ballpark
  but with more variables. Worth a spike before committing to phase
  3.1.
- **Do we cache the placement results offline?** Runtime generation is
  attractive (no build-time pipeline), but if solve time is variable
  (some shapes 100ms, others 30s) the user-facing latency is bad.
  Compromise: cache results in a checked-in JSON/RON, generate on
  first miss, write back to source. Decide based on phase 3.1
  benchmarks.
- **D1a or D1b?** Defer until phase 3.0 has the D1a entity counts in
  hand. If D1a's `(4, 9)` is already <100 entities, ship it. If
  150+, try D1b.
- **What about the existing rfp-balancer-runner work?** That work
  parallelises Factorio-SAT for tier-9/10. It's complementary: even
  with phase-3 shipped, the SAT-runner is still useful as a fallback
  for shapes our topology generator can't handle (if any). No conflict.

## Decision log

- *2026-05-01 — drafted. Awaiting user feedback on D1 (topology
  recipe), D2 (SAT vs custom placer), and the kill criterion bounds
  (esp. the entity-count target for `(4, 9)`). Spike on `varisat`
  placement perf is the gating item before committing to phase 3.1.*
