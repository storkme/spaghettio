# RFP: Logical-graph + placement-only balancer solver

## Summary

Generate balancer layouts for coprime `(m, n)` shapes by **decomposing
graph discovery from grid placement**:

1. **Rust topology generator** (in-tree, runtime-callable) — given
   `(m, n)`, emit a logical splitter graph (universal-balancer with
   back-loops sized to introduce the right non-binary denominator
   for `m/n`).
2. **Rust + CP-SAT placement solver** (build-time, offline binary) —
   given the graph + a bounding box, encode placement as a CP problem
   using OR-Tools' CP-SAT (via the `cp_sat` Rust crate). Solve, decode
   to entity positions.
3. **Bake** the resulting `BalancerTemplate` into `balancer_library.rs`
   via a regeneration step, same shape as the existing
   `scripts/generate_balancer_library.py` pipeline — but written in
   Rust and using a fixed topology, so search is dramatically narrower.

The premise: today's Factorio-SAT pipeline searches topology *and*
placement jointly, which is why it falls over on tier-9/10 and is the
reason `rfp-balancer-runner.md` exists. If we hand the solver a *known*
graph and ask only for a placement, the search shrinks dramatically —
and we can do it all in Rust, replacing the Python+Factorio-SAT pipeline
end-to-end.

The runtime classifier ([`balancer_classify::classify_ref`](../crates/core/src/bus/balancer_classify.rs))
verifies every generated template before it lands in the library, so
correctness is checked by tooling we already trust.

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
  faster per shape (placement-only search is much smaller than the
  joint search Factorio-SAT does), and the entire pipeline is Rust —
  no Python interpreter, no Factorio-SAT venv, no tier-by-tier overnight
  runs.

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

### D2 — Placement encoding (CP-SAT)

CP-SAT's high-level constraints map naturally onto the placement
problem — none of the boolean-encoding tedium that pure SAT needs:

Variables (integer / interval, native to CP-SAT):

- For each graph node `v`: `(x_v, y_v)` integer position.
- For each graph edge `(u → v)`: an ordered sequence of `(x, y, dir)`
  tile triples representing the belt path.
- Per-tile occupancy: a single integer "what's here" per tile (0 = empty,
  ≥1 = entity id).

Constraints:

- **`AddNoOverlap2D`** for splitter footprints (2×1 perpendicular to
  flow direction). One global constraint instead of pairwise overlap
  bools.
- **`AddAllDifferent`** on tile occupancy — at most one entity per tile.
- For each edge, **`AddCircuit` / `AddPathConstraint`** — encodes a
  legal belt path from `u` to `v` with direction continuity.
- Splitter second-tile offset modelled as a fixed displacement between
  paired tile variables (per
  [`splitter_second_tile`](../crates/core/src/common.rs)).
- UG-pair constraints — input/output same axis, same direction, distance
  ≤ tier limit (U3, U5).
- Bounding box: integer variables `(W, H)` with `AddMaxEquality` over
  all entity x/y coordinates. Minimised in a single CP-SAT objective
  pass — no outer search loop needed (CP-SAT handles bounding-box
  optimisation natively).

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

### D4 — Build pipeline integration

The pipeline mirrors the existing `scripts/generate_balancer_library.py`
flow but in Rust:

```
crates/balancer-gen/                  # NEW Rust binary crate
  src/main.rs                         # CLI: --shapes, --max-tier, --check
  src/topology.rs                     # D3: emits LogicalGraph
  src/placement.rs                    # D2: CP-SAT encoding via cp_sat crate
  src/sync.rs                         # writes BalancerTemplate constants
                                      # into balancer_library.rs

crates/core/src/bus/balancer_topology.rs    # NEW (in-tree, runtime)
                                      # topology generator + verifier hooks
                                      # for unit tests; no CP-SAT dep here

crates/core/src/bus/balancer_library.rs     # extended with new shapes
```

Workflow:

1. Run `cargo run -p balancer-gen -- --shapes 4x9,5x9,7x9` (or
   `--max-tier 10` to fill all coprime gaps).
2. Each shape: topology generator → CP-SAT placement → verify via
   `classify_ref` → emit Rust constant.
3. The binary writes directly into `balancer_library.rs` (or a sibling
   `balancer_library_extra.rs` to keep the diff isolated). Atomic
   replace, same pattern as the existing Python sync.
4. `cargo test` then exercises the new templates; CI catches any
   regressions.

The runtime `crates/core/` code does **not** depend on `cp_sat` or
OR-Tools. It only consumes `BalancerTemplate` constants. WASM build is
unaffected.

`stamp_family_balancer` doesn't change — it just sees more shapes in
`balancer_templates()`.

### Trade-offs considered

- **CP-SAT vs pure SAT (varisat).** SAT requires manual boolean
  encoding of integer positions and overlap constraints — high LOC and
  fragile. CP-SAT's `AddNoOverlap2D`, `AddAllDifferent`, `AddCircuit`
  map directly onto the problem. The win is encoding clarity, not
  necessarily solve speed.
- **CP-SAT in-tree (runtime + WASM) vs offline.** Build-time is the
  right call — CP-SAT is a C++ library, WASM-incompatible without
  serious work. Offline keeps the runtime crate clean and lets us pick
  the best solver for the job.
- **Custom Rust placer instead of CP-SAT.** Tempting but the
  combinatorics (UG crossings, splitter-overlap shortcuts) are subtle;
  reinventing CP-SAT's propagation badly is a tarpit.
- **Pure-Rust CP solver (`pumpkin-solver`, `copper`).** Maturity
  uncertain; defer until/unless OR-Tools turns out to be a build-time
  pain. If a credible WASM-friendly Rust CP solver lands during this
  project, we revisit and *might* be able to move the work in-tree at
  runtime.
- **Python+OR-Tools subprocess instead of cp_sat Rust crate.** Same
  result for the user, but introduces a Python+venv dep we're trying
  to phase out. Use only if `cp_sat` Rust binding is too immature.
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
- **CP-SAT solve time per shape > 5 minutes.** Per-shape time matters
  even build-time — full library regeneration shouldn't take more than
  a coffee break. If we hit this, either the topology is too loose
  (try D1b) or the encoding has redundancy.
- **Round-trip placement fails.** If we hand-construct a topology we
  know is placeable (extract via `recover_graph` from an existing
  library template, feed back through the placement solver) and the
  solver doesn't return a result, the encoding has a bug.
- **`cp_sat` Rust binding doesn't build cleanly cross-platform.** The
  binary needs to run in CI (Linux) and on dev machines (macOS, Linux).
  If `cp_sat` requires manual OR-Tools install with multiple platform
  variants, fall back to a Python+OR-Tools subprocess invocation.
  Less elegant but ships.

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

- **Phase 3.0 — topology generator only.** Land
  `crates/core/src/bus/balancer_topology.rs`. Verify via `classify_ref`
  over all coprime shapes for `(m, n) ≤ 6`. *No placement yet.* Output
  is logical graph data only — independently useful for the audit and
  for round-trip tests later. ~200-400 LOC, zero new deps.
- **Phase 3.1 — `cp_sat` spike.** Add `crates/balancer-gen/` binary
  crate, depend on the `cp_sat` Rust crate. Encode placement of a
  *known-good* template (e.g. extract `(2, 3)` from the library, feed
  back through). Confirm CP-SAT can place it within 30s. Cross-platform
  build verified on macOS + Linux CI. Bail to "Python subprocess"
  fallback if blocked. ~200 LOC + Cargo wiring.
- **Phase 3.2 — full placement encoding.** Generalise the spike to
  arbitrary topology graphs. Round-trip test: every existing library
  template extracted via `recover_graph`, re-placed, classified — must
  match original class. ~500-800 LOC.
- **Phase 3.3 — beat-Factorio-SAT measurement.** Generate `(4, 9)`,
  `(5, 9)`, `(7, 9)`, `(5, 7)`. Compare entity counts + solve times
  against current Factorio-SAT pipeline (run that pipeline once on
  these as baseline). Decision-log the numbers; trigger kill criteria
  if missed.
- **Phase 3.4 — bake into library.** Run the binary, atomically replace
  the relevant section of `balancer_library.rs`, run the full test
  suite. New shapes are now usable by `stamp_family_balancer` with no
  runtime change.
- **Phase 3.5 (optional) — replace existing library entries** where
  our generator beats Factorio-SAT on entity count. Phase 2.1-style
  work; gated on 3.4 stability.
- **Phase 3.6 (longer-term, optional) — retire Factorio-SAT.** Once
  the new pipeline covers the full `(1..10) × (1..10)` envelope and
  has run in CI for a release cycle, archive
  `scripts/generate_balancer_library.py` and `external/factorio-sat/`.
  Closes the dependency loop.
- **Out of scope:** lane-aware MX5 verification, mixed-content buses,
  D1b topology variant (deferred unless D1a misses kill criteria),
  runtime in-WASM placement (would require a pure-Rust CP solver).

## Open questions

- **`cp_sat` crate maturity / cross-platform build.** Phase 3.1 spike
  is gated on this. If the crate works cleanly on Linux + macOS we
  proceed; otherwise fall back to Python+OR-Tools subprocess (still
  a build-time-only dep) or revisit a pure-Rust CP solver.
- **D1a or D1b topology?** Defer until phase 3.0 has D1a entity
  counts in hand. If D1a's `(4, 9)` is already <100 entities, ship
  it. If 150+, try D1b.
- ~~**Where does the generator's output go in `balancer_library.rs`?**~~
  *Resolved 2026-05-01:* sibling `balancer_library_extra.rs`. Keeps the
  Factorio-SAT and graph+place pipelines writing to independent files
  until phase 3.6 retires Factorio-SAT and the contents are merged.
  The library lookup will consult both maps (extra first, then library
  fallback).
- **What about the existing rfp-balancer-runner work?** That work
  parallelises Factorio-SAT for tier-9/10. It's complementary: even
  with phase-3 shipped, the SAT-runner remains useful as a fallback
  for shapes our topology generator can't handle (if any). They
  converge at phase 3.6 when Factorio-SAT can be retired.
- **Cross-shape solver caching.** OR-Tools' CP-SAT supports incremental
  solving. If we generate (4, 9), (5, 9), (7, 9) in one run, can we
  share search state? Probably not worth optimising in phase 3, but
  worth noting if the per-shape solve time turns out to dominate.

## Decision log

- *2026-05-01 — drafted (initial: in-tree varisat + runtime
  placement).*
- *2026-05-01 — revised after feedback: keep CP-SAT placement
  offline / build-time, written in Rust. Topology generator stays
  in-tree and runtime-callable for verification. WASM bundle /
  runtime perf concerns drop.*
- *2026-05-01 — phase 3.0 (topology generator) landed. SplitterGraph
  and NodeId promoted to public API; classify_graph and
  topology_of_template entrypoints added. Atoms (passthrough,
  one_to_two, two_to_one), composers (parallel, series, series_permuted,
  clos_interleave), and library_atom bootstrap. (4, 9) Clos composition
  via series_permuted(parallel(library(1, 3), 4),
  parallel(library(4, 3), 3), clos_interleave(4, 3)) classifies as
  MX3 — the headline coprime case is verified end-to-end at the
  topology layer. (3, 5) verified similarly. Topology splitter counts:
  (4, 9) Clos = 33; (3, 5) Clos = 36 (small shapes the library has
  hand-tighter unified topologies).*

- *2026-05-01 — phase 3.1 spike result: `cp_sat` Rust crate failed to
  build in our sandbox (needs OR-Tools C++ at `/opt/ortools/include/`).
  Cross-platform-build kill criterion hit; pivoted to Python+OR-Tools
  subprocess as documented in the RFP fallback. `pip install ortools`
  worked first try. Spike (2, 3) library round-trip: 10ms; (4, 9) Clos
  composition with 33 splitters: 12ms. Both well under the 30s kill
  budget. Splitter no-overlap is what's encoded so far — belt routing
  is phase 3.2 work and will materially increase solve time. The
  framework is viable; new Rust crate `crates/balancer-gen/` and
  `crates/balancer-gen/scripts/place.py` ship the spike.*
