# RFP: Multi-splitter belt routing for the placement solver

## Summary

Extend the phase 3.1 / 3.2-MVP spike (
[`crates/balancer-gen/`](../crates/balancer-gen/)) from
"place splitters with no-overlap" to "place splitters AND route belts
between them, including underground crossings and back-loops, on
arbitrary topology graphs." This is the genuinely combinatorial part
of the phase-3 project that
[`rfp-balancer-graph-place.md`](rfp-balancer-graph-place.md) framed as
"phase 3.2 — full placement encoding."

Concretely: take a `SplitterGraph` and a bounding box, return an
`OwnedTemplate` (entity list with positions + directions) such that the
recovered topology classifies as the same `BalancerClass` the input
graph did. Multi-splitter shapes with back-loops (`(2, 3)`,
`(1, 3)`, `(4, 9)` Clos) round-trip cleanly.

The MVP confirmed the *plumbing* works. This RFP nails down the
*encoding* — and the encoding is where the scary numbers live.

## Motivation

### What the MVP proved (and didn't)

[`crates/balancer-gen/src/main.rs`](../crates/balancer-gen/src/main.rs)
+ [`scripts/place.py`](../crates/balancer-gen/scripts/place.py) closes
end-to-end for `(1, 2)` and `(2, 2)`:

- 1 splitter placed by CP-SAT, no-overlap only.
- Boundary belts emitted in Rust by simple slot assignment.
- `OwnedTemplate` round-trips through `classify_ref` as MX3.

What it dodged:

- **Multiple splitters with edges between them.** No inter-splitter
  belt path needed for single-splitter shapes; everything's at the
  bounding-box boundary.
- **Back-loops.** Universal-balancer shapes like `(1, 3)` have an
  output that loops back to a splitter input. The MVP can't represent
  this.
- **Underground belts.** Library `(2, 3)` uses UGs to fit 4 splitters +
  10 edges in 3×14. Without UG support, our router needs significantly
  larger bounds — at which point we're not competitive with
  Factorio-SAT.
- **Splitter direction freedom.** Library `(1, 3)` has a west-facing
  splitter. South-only splitters cap our reachable shapes.

The headline use cases (`(2, 3)` round-trip, `(4, 9)` Clos placement)
exercise *all* of these. The MVP's "framework is solid" is true, but
the actual encoding work is in front of us.

### Why this is the lever

Phase 3.1 measured CP-SAT solve time for splitter no-overlap on `(4, 9)`
Clos at 12ms — ~3000× under the 30s kill criterion. Belt routing is
genuinely harder, but the spike data suggests CP-SAT has plenty of
headroom for the larger encoding. If we land this, every coprime shape
in the `(1..10) × (1..10)` envelope becomes generable in Rust without
Factorio-SAT — closes [#266](https://github.com/storkme/fucktorio/issues/266)
candidates and unlocks tier-9/10 layouts.

## Design

This RFP picks defaults for each decision. They're all overridable.

### D1 — Routing model (CP-SAT vs A* vs hybrid)

**Three candidates:**

- **D1a — Full CP-SAT.** Encode each edge's belt path as flow-conservation
  constraints over a directed grid graph. Per-cell, per-edge bool vars
  for "edge `e` enters cell `c` from direction `d`." Standard
  formulation; CP-SAT handles it well. Costs ~thousands of bool
  variables but solver search is still tractable for our sizes.
- **D1b — Sequential A*-per-edge.** Place splitters via CP-SAT, then for
  each edge run a Rust grid-A* between the source/dest tiles, treating
  previously routed belts as obstacles. Fails when an early edge blocks
  a later edge — backtrack via "edge ordering" search.
- **D1c — Hybrid.** Try A* first (cheap, almost-always-works for sparse
  graphs); fall back to CP-SAT on backtracking-exhaustion.

**Default: D1a.** CP-SAT-native because (i) we're already paying the
Python-subprocess cost, (ii) it scales cleanly to UGs and direction
freedom (just more variables), (iii) gives global-optimal placement vs
sequential greedy.

D1b is tempting for simplicity but the existing fucktorio bus pipeline
proves grid-routing-with-priorities is non-trivial — see
[`bus/junction_solver.rs`](../crates/core/src/bus/junction_solver.rs).
We'd be reinventing what the SAT solver already does well.

### D2 — Underground belt support

UGs are required for any tight bounding box. Library `(2, 3)` would not
fit in 3×14 without them.

**D2a — UG pairs as part of the encoding.** For each "edge needs to
cross" point, a UG-input + UG-output pair is materialised. CP-SAT
variables: per-cell `is_ug_input[c, e, d]`, `is_ug_output[c, e, d]`,
plus pairing constraints (same axis, distance ≤ tier limit, no other
UG of the same tier between them).

**D2b — Skip UGs, accept bigger bounding box.** Forbid crossings;
require enough space for parallel routing.

**Default: D2a.** Without UGs we're stuck at suboptimal sizes
(estimated 2-3× larger templates), which trips the entity-count kill
criterion from the parent RFP. UG encoding adds maybe 30% more
constraints but stays in CP-SAT's wheelhouse.

### D3 — Splitter direction freedom

**D3a — Fixed south-facing.** Simplest. Caps reachable layouts —
universal balancers often need a perpendicular splitter to back-loop.

**D3b — Per-splitter direction variable.** Each splitter gets a
direction enum (4 options). Slot positions become direction-dependent.

**Default: D3b.** Necessary for back-loops. Cost: ~4× more cases per
splitter, but still small per shape.

### D4 — Edge → port assignment

The topology graph (`SplitterGraph`) records edges as `(NodeId, NodeId)`
with no port (slot) info. A splitter has 2 input slots and 2 output
slots; an edge from splitter A to splitter B implicitly uses one slot
of each.

**D4a — Solver assigns slots.** For each edge, additional bool variables
choose source slot ∈ {0, 1} and dest slot ∈ {0, 1}. Constraints: each
slot used at most once.

**D4b — Caller-provided port assignment.** Augment `SplitterGraph` with
slot info; topology generators decide upfront.

**Default: D4a.** Decouples the topology layer from placement
constraints; topology generators stay simple. Cost: 2 extra bool
variables per edge.

### D5 — Bounding box strategy

**D5a — Caller-provided fixed bounds.** Like the MVP. Caller picks W×H,
solver finds *some* placement or returns infeasible.

**D5b — Iterative bounding-box minimisation.** Start at a feasible
upper bound (or library's bounds for known shapes), run CP-SAT to
minimise W·H subject to feasibility, return tightest layout.

**Default: D5b** for offline generation (we have time), **D5a** for
spike runs and per-shape debugging.

### D6 — Belt direction continuity

Each belt tile has a direction. The path of belts between splitters
must have compatible directions: belt at cell `c` with direction `d`
flows to cell `c + step(d)`, which must either be (a) another belt
that "accepts" flow from direction `d` (any direction works for
default belts), (b) a splitter input slot facing the right way, or
(c) the destination tile.

**Encoded via per-cell direction variables + flow-conservation
constraints.** Same machinery as D1a; just a layer on top.

### D7 — Library-template round-trip vs from-scratch generation

Two distinct uses:

- **Round-trip:** load library template `(m, n)`, recover topology,
  re-place. Output should classify as same class as input. Used for
  testing the encoding doesn't lose information.
- **From-scratch:** topology composed in Rust (e.g., `(4, 9)` Clos),
  no physical reference. Just place and classify.

**Both supported by the same encoder.** Round-trip is the verification
mechanism for from-scratch.

### D8 — Output bake target

Per the parent RFP D4 (sibling `balancer_library_extra.rs`):
- `crates/balancer-gen/` binary CLI: `--shapes 4x9,5x9` or
  `--max-tier 10`.
- Output: regenerates `crates/core/src/bus/balancer_library_extra.rs`,
  same data layout as `balancer_library.rs`.
- The runtime crate's lookup consults both maps (extra first, then
  fallback to library).

This RFP doesn't change that decision, just implements it.

### Trade-offs considered

- **Reuse fucktorio's existing routing infra.** The bus's ghost-router
  + junction solver is sophisticated, but it's tightly coupled to the
  layout pipeline (consumer rows, lane families, etc.). Extracting it
  for standalone balancer placement would be a refactor; CP-SAT from
  scratch is shorter.
- **Custom Rust CP solver instead of OR-Tools.** Pure-Rust CP solvers
  (`pumpkin-solver`, etc.) are immature for grid routing. Re-evaluate
  if/when one matures; not worth blocking on.
- **Place all entities in CP-SAT including belts (no Rust router).**
  This is the proposed D1a. Yes — that's the whole point.

## Kill criteria

- **`(2, 3)` round-trip exceeds 100 entities or fails to solve.**
  Library is 35 entities; we need to come in under ~50 to be
  competitive. If we're at 100+, the encoding is too loose.
- **`(4, 9)` Clos placement exceeds 250 entities.** Parent RFP's
  kill criterion. Phase 3.0 estimated ~80-120 entities for placement;
  if we land at 250+, the value prop is gone.
- **CP-SAT solve time per shape > 5 minutes.** Parent RFP. If single
  shapes take this long, full library regeneration is overnight again
  and we haven't escaped Factorio-SAT's pain.
- **Encoding can't represent UGs correctly.** Round-trip a library
  template that uses UGs (e.g., `(1, 3)`); if the recovered topology
  classification differs from the original (because we silently
  dropped/misplaced a UG), encoding is unsound — stop until fixed.
- **Direction freedom causes search to blow up.** If solve time is
  >>10× higher with D3b than D3a-fixed-south, the search space is
  poorly structured and we should reconsider. Possible mitigation:
  symmetry breaking via "splitter 0 always faces south."

## Verification plan

1. **Encoding unit tests (Python).** Place a fixed splitter graph,
   confirm CP-SAT returns a valid placement (no overlaps, edges
   continuous, UG pairs aligned). Asserts on the placement structure
   directly without going through Rust.
2. **Library round-trip suite.** For every shape in `balancer_templates()`:
   - Recover topology.
   - Place via CP-SAT.
   - Assemble `OwnedTemplate`.
   - Classify; must match original class.
   This is the load-bearing correctness check.
3. **From-scratch generation.** `(4, 9)` Clos topology → place → classify
   → MX3. Compare entity count to Factorio-SAT baseline (run that
   pipeline once on `(4, 9)` for the comparison).
4. **Bounding-box minimisation.** For `(2, 3)`, minimise W·H. Report
   the minimum size found and compare to library's 3×14.
5. **Per-shape solve-time benchmark.** Run the encoder on every
   `(m, n)` ≤ 10. Tabulate solve times. Flag outliers > 30s for
   investigation.
6. **In-game spot-check.** Stamp the generated `(4, 9)` template via
   `stamp_family_balancer` (after the library_extra path is wired) in
   a real layout. Saturate inputs, observe outputs run at 4/9 each.
7. **Trace events.** `BalancerGenerated` (already exists) emits
   per-stamp; verify reach in real layouts via the snapshot debugger.

## Phasing

- **Phase 3.2A — flow-conservation belt routing for fixed-direction
  splitters.** No UGs, no direction freedom. Round-trip `(1, 3)` (no
  UGs needed if we use a generous bounding box). Demonstrates the
  encoding works for back-loops. ~300-500 LOC of Python on top of the
  existing spike.
- **Phase 3.2B — underground belts.** Add UG variables and pairing
  constraints. Round-trip `(2, 3)` and `(2, 5)` — library shapes that
  use UGs. Tighten bounding box.
- **Phase 3.2C — direction freedom.** Per-splitter direction. Round-trip
  shapes that need perpendicular splitters (the back-loop universal
  patterns). Likely tier-≤8 coprime shapes.
- **Phase 3.2D — bounding-box minimisation.** Add CP-SAT objective for
  W·H, run on the headline shapes. Compare entity counts to library
  and to Factorio-SAT.
- **Phase 3.3 — measurement against Factorio-SAT.** Headline `(4, 9)`,
  `(5, 9)`, `(7, 9)`, `(5, 7)`. Decide on kill criteria.
- **Phase 3.4+** — bake into `balancer_library_extra.rs` per parent RFP.

Each sub-phase is landable independently with its own round-trip test.

## Coordination with PR #270

[PR #270](https://github.com/storkme/fucktorio/pull/270) is a parallel
audit-phase fork that diverged in two interesting ways. Both deserve
explicit notes here so phase-3.2+ implementation lands without
re-litigating decisions:

- **Verifier divergence on saturated-merge constructions.** PR #270
  introduces an all-fluid Couëtoux verifier that rejects any layout
  whose steady state depends on internal back-pressure saturation —
  including the `two_to_one()` atom in `balancer_generate.rs` (the
  dangling-output trick caps a `(2, 1)` merger at 1 unit of throughput
  by saturating the splitter via back-pressure). Both signals are
  honest: this RFP's classifier checks the saturated-model invariants
  the layout layer relies on; the all-fluid verifier checks an
  unsaturated-flow invariant. Cross-validation disagreements on
  `two_to_one`-using templates are *expected*, not bugs. Whichever
  verifier becomes authoritative for the placement output, this RFP's
  generator will be cross-checked against the other; the disagreement
  count is the right signal, not the absolute pass/fail.

- **CP-SAT subprocess pattern convergence.** PR #270's
  `scripts/cp_sat_placer.py` lives behind a `PlacementEngine` trait
  inside `crates/core/` and uses PEP 723 + `uv run --no-project` for
  self-installing dependencies. The phase-3.1 spike binary in
  [`crates/balancer-gen/`](../crates/balancer-gen/) drove a separate
  `python3` subprocess at first; this PR's review brought us to the
  same `uv run --no-project` PEP 723 pattern, so the dep convention now
  matches. The deferred work is folding `crates/balancer-gen/` into
  PR #270's `PlacementEngine` trait once the placement encoding
  graduates from spike to real engine — the spike binary's only
  remaining job is exercising CP-SAT against non-trivial topologies
  (the Clos composition test, the single-splitter MVP round-trip).

  Concretely: phase 3.2C is a natural seam for consolidation. By that
  point the encoder handles direction-free splitters and UGs, which
  matches the trait's expected surface. We adopt the trait, retire the
  spike binary's standalone main, and the `crates/balancer-gen/` crate
  becomes a thin wrapper around the trait's `place(graph, bounds)`
  call (or disappears entirely if the trait subsumes the CLI).

## Open questions

- **How big does the encoding get for `(4, 9)` Clos?** 33 splitters,
  67 edges. With per-cell-per-edge-per-direction bools on a 24×24 grid:
  24·24·67·4 = 154k bool vars + flow-conservation constraints. Worth a
  spike in phase 3.2A before fully committing.
- **Can we share placement state across shapes?** Probably not worth
  the complexity in phase 3 — defer.
- **Should we cache placements offline (checked-in JSON) so repeat
  invocations are fast?** Maybe. Decide once we have phase 3.2D
  solve-time numbers.
- **Does the cp_sat Rust crate's build situation change?** If it
  becomes installable cleanly cross-platform, we could move the encoding
  fully into Rust later. Track but don't block.
- **What if a topology has a node with > 2 outputs (not a Factorio
  splitter)?** Topology should never produce this — the abstraction
  matches Factorio's splitter primitive. Add a debug assertion in the
  encoder.

## Decision log

- *2026-05-01 — drafted. Awaiting feedback on D1 (CP-SAT vs A*),
  D5 (bounding-box minimisation default), and the kill-criterion
  bounds — particularly `(2, 3)` ≤100 entities and `(4, 9)` ≤250
  entities. Phase 3.2A is the gating item; if `(1, 3)` round-trip
  works in <30s, the encoding is viable and we proceed through 3.2B-D.*

- *2026-05-01 — phase 3.2A.1, 3.2B, 3.2C all encoded. Status:*

  | Sub-phase | Encoding | Round-trip on simple shapes | Round-trip on coprime |
  |-----------|----------|------------------------------|------------------------|
  | 3.2A.1 (flow-conservation) | ✅ shipped | ✅ (2, 2), (2, 4), (1, 4) | n/a |
  | 3.2B (UG belts)             | ✅ shipped | ✅ same                   | partial — see below   |
  | 3.2C (direction freedom)    | ✅ shipped | ✅ same                   | partial — see below   |

  *Working coprime / library-with-UG / library-with-non-south-splitter
  shapes round-trip 3/8 (the simple shapes). The remaining 5 fail with
  one of two symptoms — INFEASIBLE (CP-SAT can't route inside library
  bbox with our slot assignment) or Singular (CP-SAT routes but the
  recovered topology has degenerate cycles).*

  *Single root cause: the Rust-side **greedy min-distance slot
  assigner** picks the same slot the library used only for shapes
  without back-loops. For shapes where the library uses specific
  slot orderings to enable the back-loop pattern, the greedy diverges
  and either (a) routes into a topology equivalent to the library
  but in a way the classifier's linear-system solver finds singular,
  or (b) can't route at all in the same bbox.*

  *Fix is **phase 3.2A.2** — let CP-SAT pick slots as variables, not
  Rust greedy. Estimated 200-300 LOC of Python (slot bool vars per
  edge × splitter, slot-usage constraints per splitter, reified
  source/dest-cell expressions in conservation). Deferred to next
  session.*

  *Phases 3.2D (bbox minimisation), 3.3 (measure vs Factorio-SAT),
  3.4 (bake into library_extra) all deferred — they're gated on full
  shape coverage, which 3.2A.2 unlocks.*

- *2026-05-01 (later) — phase 3.2A.2 shipped. Slot assignment is now a
  CP-SAT variable: per-edge `src_slot_anchor[e] / src_slot_second[e]`
  bool vars (and dst counterparts) with `ExactlyOne` per edge and
  `AtMostOne` per splitter slot. Conservation uses reified
  `is_src_term_at` / `is_dst_term_at` linear expressions so source/dest
  cells are picked by the solver. Two follow-on bugs found and fixed
  along the way:*

  1. *UG output emission — the original 3.2B code accounted UG inflow
     at the output cell, but the at-most-one-entity rule already used
     that cell for the UG output entity, leaving no place for a
     forwarding belt. Fix: shift UG inflow accounting one cell ahead
     in the UG's direction (the UG output entity itself emits its
     flow forward). The output cell becomes a passthrough — its UG
     inflow cancels with its own emission, conservation is balanced.*
  2. *`UG_MAX_REACH` was set to 4 but yellow belts allow up to 4
     transit tiles between input and output, i.e., a max length of 5
     (`output = input + 5*direction`). Bumped to 5.*

  *Round-trip results: 7/8 pass — (2, 2), (2, 4), (1, 4), (1, 3),
  (2, 3), (4, 8), (3, 5). Solve times 0.01s..16s. Only (5, 3) still
  fails INFEASIBLE: its library template uses **belt-into-UG-output
  sideloading** (a regular belt's flow merges into a UG output's
  lane), which the flow-conservation encoding doesn't model — every
  edge carries one indivisible flow along one path. This is a
  fundamental limitation of the encoding rather than a slot issue.*

  *Remaining work: phases 3.2D (bbox minimisation), 3.3 (measure vs
  Factorio-SAT), 3.4 (bake into library_extra) now unblocked. The
  sideload limitation means we can't reproduce every Factorio-SAT
  output exactly, but for coprime shape generation we don't need to —
  we just need a working topology in a tight bbox.*
