# RFP: Modular production — demand-partitioned modules + subtree decomposition

> **Status**: Draft, not yet accepted. Living document — update the
> Decision log and Task tracker as work progresses. Kill-criteria are
> explicit and meant to be acted on.

## Current status

- **Phase**: Draft / brainstorming complete.
- **Flag state**: both features gated behind off-by-default flags;
  current behavior is the no-flags path.
- **Last update**: 2026-04-24 — initial draft from brainstorm.

## Summary

The current layout engine pools each intermediate item into one shared
bus lane family and stamps a single balancer at the producer row
output. This caps the widest producible intermediate at 8 lanes
(the largest template in `balancer_library.rs`). We propose two
*optional, composable* strategies, layered behind feature flags:

1. **Demand-partitioning (outer)** — split a high-demand intermediate
   into one producer module per consumer, sized to that consumer's
   exact demand, wired directly to it. No shared pool, no
   pool-balancer, and the widest balancer ever needed is bounded by
   the *largest single consumer's* demand, not the sum.
2. **Subtree decomposition (inner)** — if a single demand-partitioned
   module is still wider than 8 lanes (big consumer, or recipe ratios
   expand going up-tree), shard *that* module into ⌈widest / 8⌉
   sibling subtrees, each producing a proportional share.

Both flags default to off. The existing pooled/balancer-stamped path
stays the baseline until one or both strategies earn the right to
become default via the kill-criteria checks below.

## Motivation

Concrete failing case: `advanced_circuit` at moderate-to-high rates
needs more copper cable than any 8-lane balancer can feed.
[`#136`](https://github.com/storkme/fucktorio/issues/136) tracks the
missing-shape problem. Related: [`#135`](https://github.com/storkme/fucktorio/issues/135)
(balancer templates oversized) and [`#68`](https://github.com/storkme/fucktorio/issues/68)
(fluid row 3-tile pitch).

Today the layout engine has no recourse when it hits a >8-lane
family: it either can't find a template and falls back to a
degenerate passthrough, or stamps an undersized balancer that leaves
downstream inserters starved. Both produce bad layouts; neither is
detectable purely via validator error count.

### Architectural reframe

This change reframes the "bus" from a *shared resource pool*
(homogenized supply, lane-agnostic tap-off) to a *routing fabric for
locally-scoped producer→consumer pairs*. The lane still exists, but
it belongs to a specific consumer rather than to "anyone who wants
cable." This is a real semantic shift in `lane_planner.rs` /
`lane_order.rs`, not a balancer swap.

### Load-bearing assumption

Partitioning works without a pool-balancer because belts are wildly
over-provisioned relative to machine consumption (yellow belt =
15/s; a single assembler consumes 1–2/s of most ingredients).
Per-machine timing jitter produces mild lane-to-lane variance in the
producer's output, but the over-provisioning absorbs it. This
assumption breaks for speed-module-3'd rows on marginal lanes —
document it, and watch for it in verification.

## Design

### New option surface

```rust
// crates/core/src/bus/layout.rs
pub struct LayoutOptions {
    pub demand_partition: bool,     // Feature flag — outer pass
    pub subtree_decompose: bool,    // Feature flag — inner pass
    // ... existing options merged in
}

pub fn build_bus_layout(
    solver: &SolverResult,
    belt_tier: BeltTier,
    opts: LayoutOptions,
) -> LayoutResult;
```

Both flags default to `false`. With both off, the pipeline is
byte-identical to today (enforced by a flag-off regression snapshot
on every e2e test).

WASM bindings (`crates/wasm-bindings/src/lib.rs`) expose the flags as
optional params on `layout()`. The web app (`web/src/ui/sidebar.ts`)
adds two debug toggles; URL state (`?partition=1&decompose=1`) makes
links reproduce the experimental mode.

### Phase 0 — shared scaffolding (no behavior change)

The scaffolding both features need is: **the lane planner must be
able to represent multiple same-item families**. Today
`LaneFamily` is implicitly one-per-item; downstream code (lane-order
optimiser, balancer stamper, tap-off router) assumes this.

Touch list:

- `crates/core/src/bus/lane_planner.rs` — make `LaneFamily`
  identity include a `module_id: u32` alongside the item name. One
  `module_id=0` per item in the baseline = today's behavior.
- `crates/core/src/bus/lane_order.rs` — key the exact-search on the
  `(item, module_id)` tuple. The ≤7-family cutoff applies to the
  tuple count, not the item count.
- `crates/core/src/bus/balancer.rs` — `stamp_family_balancer` takes
  the tuple; behavior unchanged when `module_id=0` is the only one.
- `crates/core/src/bus/templates.rs` — tap-off routing looks up by
  tuple; consumer→module mapping defaults to "consumer of item X
  taps the single `(X, 0)` family."

Flag-off behavior must be identical. All 9 non-ignored e2e tests
stay green with zero snapshot drift.

### Phase 1 — demand-partitioning (outer pass)

When `demand_partition = true`:

1. For each intermediate item with multiple consumers, allocate one
   `LaneFamily` per consumer. Each family's lane count is sized to
   the consumer's exact demand (rounded up to the nearest belt).
2. Consumer→module mapping is 1:1 (no tap decision — each consumer
   owns its module).
3. The producer row for item X is split into K sibling rows, one per
   consumer, each sized to that consumer's demand. Shared upstream
   ingredients feed all K rows via a simple tee/split (not a
   balancer — ⌈K / 8⌉ wide, typically 2–3).
4. No pool-balancer is stamped. If K > 1, no single-family balancer
   needs to be wider than the widest single consumer's lane count.

The producer-side belt math:

- Shared upstream ingredient feeds K rows. The split is `K`
  outputs, which we emit as a splitter tree. For K ≤ 8 (the common
  case) this is a single library template.
- If K > 8, the ingredient's own demand-partitioning recurses
  naturally: treat the K rows as K consumers.

### Phase 2 — subtree decomposition (inner pass)

When `subtree_decompose = true` (requires `demand_partition = true`
for the outer framing — they compose, not alternate):

For each partitioned module, walk the upstream subtree. If any
recipe's required belt count exceeds 8, shard the whole *module*
into ⌈widest / 8⌉ pieces via proportional demand split. Each shard
is an independent `LaneFamily` with its own upstream chain (with
shared external inputs).

Shard allocation rule: equal shares rounded so the widest recipe in
each shard's subtree is ≤ 8 lanes. One-pass greedy; no search.

### Consumer lumpiness — known failure mode

If a consumer needs 7 lanes and another needs 5, and the item's
widest recipe forces a 2-shard split (6+6), proportional allocation
produces a 7-from-(6+6) tap that's ugly. This is the case where
hierarchical balancer composition would win over decomposition.

**Decision**: don't solve it in Phase 2. Log the event
(`TraceEvent::LumpyShardTap`) so we can count how often it occurs
in the real corpus, then decide later whether to tackle it.

### Rejected alternatives

- **Hierarchical balancer composition** (12→12 = two 8→8s + mixer)
  — preserves single-pool semantics but requires a synthesizer over
  a composition library we don't have. Preserve as a future option
  if demand-partitioning's lumpy-tap case becomes common.
- **Status quo + hand-authored 9+/10+/12+ templates** — doesn't
  scale past the next ceiling, and `balancer_library.rs` is already
  the largest file in the repo.
- **Unbalanced splitter trees for >8** — gives up the "every
  consumer gets its share" property that makes the bus work.

## Kill criteria

Explicit, observable, falsifiable. Act on these, don't rationalize
around them.

**Phase 0 (scaffolding):**

- **K0-1**: If any of the 9 non-ignored e2e tests snapshot-drifts
  with both flags off after Phase 0 lands, the multi-family
  abstraction is not clean — abandon and rethink the types.
- **K0-2**: If the scaffolding needs > ~400 LOC of new code (not
  counting tests) just to preserve current behavior, the
  `module_id` keying is in the wrong place — likely needs to be
  deeper in `placer.rs` instead of the planner.

**Phase 1 (partitioning):**

- **K1-1**: If enabling `demand_partition` on a tier2 case that
  *doesn't need it* (e.g. `tier2_electronic_circuit_from_ore`)
  produces a layout with > 1.5× the entity count of the flag-off
  baseline, the partitioning waste is too steep — reconsider the
  "one module per consumer" rule.
- **K1-2**: If a consumer row still emits belt-flow validator
  warnings (starved inserters) with `demand_partition=true` on a
  correctly-sized partition and balanced external inputs, the
  "belts are over-provisioned so variance doesn't matter"
  assumption is false — we need balancers back, possibly scoped per
  module.
- **K1-3**: If the flag combination doesn't produce a valid layout
  for the target failing case (`advanced_circuit` at a rate that
  currently trips the 8-lane ceiling) within 2 weeks of Phase 1
  work, the approach isn't unblocking the motivating problem —
  reassess rather than push further.

**Phase 2 (decomposition):**

- **K2-1**: If Phase 2 needs > ~300 LOC outside the Phase-0
  scaffolding, the split between outer/inner passes is wrong.
- **K2-2**: If `TraceEvent::LumpyShardTap` fires on >30% of the
  stress corpus runs with decomposition enabled, lumpiness is the
  common case, not the edge — pivot to hierarchical composition
  instead of shipping decomposition.

## Verification plan

Follow the
[verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).
Specifics:

1. **Flag-off regression**: every e2e test asserts snapshot equality
   with the pre-RFP baseline. Non-negotiable for Phase 0 landing.
2. **Flag-on golden cases**: add `tier3_advanced_circuit_partitioned`
   and `tier4_advanced_circuit_decomposed` e2e tests using a rate
   that requires >8 cable lanes. Assert zero validator warnings.
3. **Browser verification**: load
   `?item=advanced-circuit&rate=<high>&partition=1` and
   eyeball the layout per CLAUDE.md verification protocol. A zero-
   warning layout that visibly has disconnected belts is a
   validator blind spot, not a success.
4. **Trace signals**: confirm
   `ModulePartitioned{item, modules}`, `ShardSplit{item, shards}`,
   and `LumpyShardTap{item, consumer}` events fire as expected.
   Absent trace events = the code path isn't exercising.
5. **Entity-count diff**: for each tier2/tier3 regression case,
   compare flag-off vs flag-on entity counts. K1-1 requires this
   stays under 1.5× on cases that don't need partitioning.

## Phasing

Three independently landable PRs:

- **PR 1 — Phase 0 scaffolding**: multi-family-per-item in the lane
  planner, with `module_id=0` baseline preserving current behavior.
  Both flags exist in `LayoutOptions` but are unused code paths.
- **PR 2 — Phase 1 partitioning**: `demand_partition` flag wired
  through. Tests for flag-on path. Browser toggle + URL state.
- **PR 3 — Phase 2 decomposition**: `subtree_decompose` flag wired
  through. Lumpy-tap trace event. Tests for flag-on path.

Each PR must pass all kill criteria for its phase before the next
PR starts.

## Task tracker

Living checklist — update as work progresses.

### Phase 0 — shared scaffolding

- [ ] Add `LayoutOptions` struct to `crates/core/src/bus/layout.rs`
      with both flags defaulting to `false`
- [ ] Thread `LayoutOptions` through `build_bus_layout` call sites
      (core, wasm-bindings, examples, tests)
- [ ] Extend `LaneFamily` identity to include `module_id: u32`
- [ ] Update `lane_order.rs` exact-search to key on `(item, module_id)`
- [ ] Update `stamp_family_balancer` to accept the tuple
- [ ] Update tap-off routing in `templates.rs` for tuple lookup
- [ ] Add consumer→module mapping defaulting to `(item, 0)`
- [ ] Verify all 9 e2e tests pass with zero snapshot drift (K0-1)
- [ ] Verify LOC budget (K0-2)
- [ ] Expose flags on WASM bindings (no-op when false)

### Phase 1 — demand-partitioning

- [ ] Add `ModulePartitioned` trace event
- [ ] Implement outer-pass partitioning in `lane_planner.rs`
- [ ] Implement producer-row splitting in `placer.rs`
- [ ] Implement shared-ingredient tee splitter (≤ 8-wide)
- [ ] Add `tier3_advanced_circuit_partitioned` e2e test
- [ ] Browser UI toggle in `web/src/ui/sidebar.ts`
- [ ] URL state: `?partition=1`
- [ ] Verify K1-1 on tier2 cases
- [ ] Verify K1-2 on tier3 partitioned case
- [ ] Verify K1-3 on motivating failure case

### Phase 2 — subtree decomposition

- [ ] Add `ShardSplit` and `LumpyShardTap` trace events
- [ ] Implement widest-recipe walker
- [ ] Implement proportional shard allocator
- [ ] Wire into `demand_partition` outer pass
- [ ] Add `tier4_advanced_circuit_decomposed` e2e test
- [ ] Browser URL state: `?decompose=1`
- [ ] Verify K2-1 LOC budget
- [ ] Verify K2-2 lumpy-tap rate on stress corpus

### Rollout (post-both-phases)

- [ ] Run full stress corpus with both flags on; compare
      scoreboards to baseline
- [ ] Decide per kill criteria: flip defaults, keep as opt-in, or
      archive this RFP

## Decision log

Append entries. Date them. Don't skip this — it's the part that
prevents "why did we drop this?" amnesia.

- *2026-04-24 — RFP drafted. Brainstorm preceding this doc
  established: (a) balancer width is the true constraint, not bus
  width; (b) demand-partitioning and subtree-decomposition are
  complementary outer/inner passes sharing a scaffolding layer;
  (c) "belts over-provisioned" is the load-bearing assumption
  behind dropping the pool-balancer. Both features to land as
  off-by-default flags to keep the current path as baseline.*
