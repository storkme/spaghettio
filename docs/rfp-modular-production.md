# RFP: Modular production — demand-partitioned modules + subtree decomposition

> **Status**: Draft, not yet accepted. Living document — update the
> Decision log and Task tracker as work progresses. Kill-criteria are
> explicit and meant to be acted on.

## Current status

- **Phase**: Draft / brainstorming complete, design revised once.
- **Strategy state**: selectable via `LayoutStrategy` enum;
  `Pooled` (today's path) is default. `PartitionedPerConsumer` and
  `PartitionedDecomposed` are first-class peer strategies, not
  opt-in degradations of the default.
- **Last update**: 2026-04-24 — revisions from second-pass review
  (see Decision log).

## Summary

The current layout engine pools each intermediate item into one shared
bus lane family and stamps a single balancer at the producer row
output. This caps the widest producible intermediate at 8 lanes
(the largest template in `balancer_library.rs`). We propose two
*composable* strategies, exposed as selectable layout modes:

1. **Demand-partitioning (outer)** — split a high-demand intermediate
   into one producer module per consumer, sized to that consumer's
   exact demand, wired directly to it. No shared pool, no
   pool-balancer, and the widest balancer ever needed is bounded by
   the *largest single consumer's* demand, not the sum.
2. **Subtree decomposition (inner)** — if a single demand-partitioned
   module is still wider than 8 lanes (big consumer, or recipe ratios
   expand going up-tree), shard *that* module into ⌈widest / 8⌉
   sibling subtrees, each producing a proportional share.

Both are exposed as peer variants of a `LayoutStrategy` enum (see
Design). `Pooled` is the default because it's the current baseline,
not because it's inherently preferred — the point of this RFP is
that **different recipes and different user preferences deserve
different layout shapes**, and modular production gives us a second
and third shape to offer. The strategy the user selects is the
strategy we produce; we don't silently upgrade or downgrade.

## Motivation

Concrete failing case: `advanced_circuit` at moderate-to-high rates
needs more copper cable than any 8-lane balancer can feed.
[`#136`](https://github.com/storkme/spaghettio/issues/136) tracks the
missing-shape problem. Related: [`#135`](https://github.com/storkme/spaghettio/issues/135)
(balancer templates oversized) and [`#68`](https://github.com/storkme/spaghettio/issues/68)
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

Partitioning works without a pool-balancer because belts are
over-provisioned relative to machine consumption (yellow belt =
15/s; a single assembler consumes 1–2/s of most ingredients).
Per-machine timing jitter produces mild lane-to-lane variance in the
producer's output, but the over-provisioning absorbs it *as long as
per-lane utilization stays well below saturation*.

**Concrete numeric threshold**: if the proposed partition would put
any lane above **75% of belt tier capacity** (e.g. >11.25/s on a
yellow belt, >22.5/s on red, >33.75/s on blue), we assume variance
will cause starvation. This shows up with speed-module-3'd
producers, tight partitions where integer belt rounding leaves
little headroom, or deep chains where intermediate demand is tight
against belt capacity. The partitioner treats 75% as a hard ceiling:
if a proposed partition can't stay under it, the partitioner emits
`TraceEvent::PartitionRejectedByUtilization` and the strategy falls
back — but because strategy is user-selected rather than
auto-chosen, "fallback" here means **producing an invalid layout
with a loud warning**, not silently switching to `Pooled`. The user
picked `PartitionedPerConsumer`; if the case can't satisfy that
shape under the utilization rule, that's diagnostic output they need
to see, not a quiet downgrade.

## Design

### New option surface

```rust
// crates/core/src/bus/layout.rs
pub enum LayoutStrategy {
    /// Current behavior: one shared lane family per item, single
    /// balancer at the producer row. Capped at 8 lanes.
    Pooled,
    /// Phase 1: one lane family per consuming recipe-row, sized to
    /// that consumer's exact demand, no pool-balancer.
    PartitionedPerConsumer,
    /// Phase 2: PartitionedPerConsumer plus subtree sharding when a
    /// single module's widest upstream recipe still exceeds 8 lanes.
    PartitionedDecomposed,
}

// New struct. Today's signature is
// `build_bus_layout(solver_result, max_belt_tier)`; the existing
// `max_belt_tier` arg folds in here, and future per-call options
// (e.g. escargio's fold parameters) attach as additional fields.
pub struct LayoutOptions {
    pub strategy: LayoutStrategy,   // default: LayoutStrategy::Pooled
    pub max_belt_tier: Option<String>,
}

pub fn build_bus_layout(
    solver: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String>;
```

`LayoutStrategy::default() == Pooled`. With the default selected the
pipeline is byte-identical to today (enforced by a regression
snapshot on every e2e test).

WASM bindings (`crates/wasm-bindings/src/lib.rs`) expose `strategy`
as an optional string param on `layout()`. The web app
(`web/src/ui/sidebar.ts`) gets a strategy dropdown; URL state
(`?strategy=partitioned-per-consumer`) makes links reproduce the
selected mode.

### Why an enum, not two booleans

Two key reasons:

1. **Invalid combinations don't exist.** `subtree_decompose=true`
   with `demand_partition=false` was meaningless; the enum doesn't
   let you express it.
2. **Future strategies slot in cleanly.** The `escargio` RFP
   (folded / spiral layouts) will add `PartitionedFolded` and
   similar as new variants rather than orthogonal booleans that
   cartesian-product into sixteen combinations, most of which don't
   make sense.

The cost is a modest refactor in Phase 0 scaffolding.

### Phase 0 — shared scaffolding (no behavior change)

The scaffolding the partitioning strategies need is: **the lane
planner must be able to represent multiple same-item families**.
Today `LaneFamily` is implicitly one-per-item; downstream code
(lane-order optimiser, balancer stamper, tap-off router) assumes
this.

Phase 0 is split into two independently reviewable sub-phases to
keep each PR small and reviewable:

**Phase 0a — core scaffolding (~400 LOC):**

- Add `LayoutStrategy` enum + `LayoutOptions.strategy` field.
- `crates/core/src/bus/lane_planner.rs` — make `LaneFamily`
  identity include a `module_id: u32` alongside the item name. One
  `module_id=0` per item in the `Pooled` baseline = today's
  behavior.
- `crates/core/src/bus/lane_order.rs` — key the exact-search on the
  `(item, module_id)` tuple. The ≤7-family cutoff applies to the
  tuple count, not the item count.
- `crates/core/src/bus/balancer.rs` — `stamp_family_balancer` takes
  the tuple; behavior unchanged when `module_id=0` is the only one.
- `crates/core/src/bus/templates.rs` — tap-off routing looks up by
  tuple; consumer→module mapping defaults to "consumer of item X
  taps the single `(X, 0)` family."

**Phase 0b — surface wiring (~200 LOC):**

- Expose `strategy` on WASM bindings (no-op when `Pooled`).
- Web app strategy dropdown, URL state.
- Threaded through call sites (examples, tests).

`Pooled`-strategy behavior must be byte-identical to today's.
All 9 non-ignored e2e tests stay green with zero snapshot drift.

### Phase 1 — demand-partitioning (outer pass)

When `strategy == PartitionedPerConsumer` (or
`PartitionedDecomposed`):

#### Consumer definition

A **consumer** is *one consuming recipe-row*, not one consuming
machine and not one consuming item. If `advanced-circuit` has
assemblers split across two rows because of throughput (standard
`placer.rs` row-splitting for rate), each row is a separate
consumer. Eight assemblers on a single row count as one consumer.
This keeps the partition count bounded by recipe-row count (small)
rather than machine count (potentially large), which is the right
granularity to make `LaneFamily` splitting meaningful.

#### Algorithm

1. For each intermediate item with K ≥ 1 consuming recipe-rows,
   allocate K `LaneFamily` instances, one per consumer. Each
   family's lane count is sized to that consumer's exact demand,
   rounded up to the nearest belt.
2. Before allocating, the partitioner checks the utilization
   threshold (75%, see *Load-bearing assumption*). If any proposed
   family would violate it, emit
   `TraceEvent::PartitionRejectedByUtilization` and produce an
   invalid layout with a loud warning — **do not silently fall back
   to `Pooled`**, because strategy is user-selected.
3. Consumer→module mapping is 1:1 (no tap decision — each consumer
   owns its module).
4. The producer row for item X is split into K sibling rows, one per
   consumer, each sized to that consumer's demand.
5. No pool-balancer is stamped. If K > 1, no single-family balancer
   needs to be wider than the widest single consumer's lane count.
6. The K producer-rows allocated in step 4 are themselves K
   consumers of every shared upstream ingredient. Step 1 applies
   recursively up-tree: an ingredient with K' = K consuming
   producer-rows for item X gets K' lane families of its own. This
   is what bounds the input distributor's width; see *The
   shared-upstream distribution* below for the K' > 8 case.

#### The shared-upstream distribution — not "just a tee"

Shared upstream ingredients feed all K producer-rows. A symmetric
K-way splitter tree is *approximately* equal-share only if the tree
is balanced and the belt compression stays sub-saturation; a
naive tee is measurably asymmetric on early branches under load.
Call this what it is: a **small-scale balancer** (K-to-K where K
≤ 8 in the common case), reusing `balancer_library.rs` templates
rather than inventing a new splitter-tree primitive. The "no
pool-balancer" property of Phase 1 is accurate at the *output* of
the producer module — there's no giant stamped balancer consolidating
item X from all producers. But the *input* side of a multi-row
producer still needs a balanced distributor of its shared
ingredients. This is a smaller balancer (K ≤ 8) and reuses existing
templates, so it's not an open problem — but the RFP shouldn't
pretend it isn't there.

If K > 8 on the input distributor, the ingredient's own
demand-partitioning recurses naturally: treat the K producer-rows
as K consumers of the ingredient, and the distributor disappears
into per-consumer lane families one level up. Recursion depth is
bounded by the recipe dependency depth (≤ ~10 for anything we'll
see in practice, including processing-unit chains).

#### Fluids

Fluids (petroleum gas, sulfuric acid, lubricant, etc.) stay pooled
under `PartitionedPerConsumer` and `PartitionedDecomposed`. Pipe
networks merge freely, there's no per-lane identity to partition
against, and the whole premise ("one belt lane per consumer")
doesn't apply. Partitioning only touches solid-item lane families.
This is an explicit non-goal, not a deferred TODO.

### Phase 2 — subtree decomposition (inner pass)

When `strategy == PartitionedDecomposed`, the outer pass runs
exactly as `PartitionedPerConsumer` (Phase 1), then the inner pass
runs over each resulting module:

For each partitioned module, walk the upstream subtree. If any
recipe's required belt count exceeds 8, shard the whole *module*
into ⌈widest / 8⌉ pieces via proportional demand split. Each shard
is an independent `LaneFamily` with its own upstream chain (with
shared external inputs).

Shard allocation rule: equal shares rounded so the widest recipe in
each shard's subtree is ≤ 8 lanes. One-pass greedy; no search.

### Consumer lumpiness — known failure mode without a fallback

If a consumer needs 7 lanes and another needs 5, and the item's
widest recipe forces a 2-shard split (6+6), proportional allocation
produces a 7-from-(6+6) tap that's ugly. This is the case where
hierarchical balancer composition would win over decomposition.

**Decision** (revised): we accept this as an unmitigated risk for
Phase 2. An earlier draft proposed prototyping a hand-authored
12→12 = 8→8 + 8→8 + mixer template in Phase 1 as a pivot target.
Implementing a Factorio-correct hierarchical balancer is real
belt-mechanic engineering — the existing `balancer_library`
templates were machine-generated by Factorio-SAT precisely because
hand-authoring is fiddly with sharp edges (lane-pair distribution,
underground reach, splitter priority). We do not have the
engineering bandwidth to author and verify one as a side quest.

If K2-2 fires under Phase 2's stress runs, the response is to
**reassess the strategy** — possibly going back to a flag that
preserves the current Pooled balancer-warning behaviour, or
extending Factorio-SAT to widen the template library — not to
invoke a hand-authored fallback. `TraceEvent::LumpyShardTap` still
fires from Phase 2 code so we can measure how often the case
actually occurs; if K2-2 turns out to be rare we may never need a
mitigation at all.

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
around them. Kill criteria are correctness- and bounds-focused; they
do **not** include "strategy X produces more entities than strategy
Y on case Z" because that's a tradeoff the user opted into, not a
bug. See *Observables* below for the tradeoff data we report but
don't gate on.

**Phase 0 (scaffolding):**

- **K0-1**: If any of the 9 non-ignored e2e tests snapshot-drifts
  with `strategy: Pooled` after Phase 0 lands, the multi-family
  abstraction is not clean — abandon and rethink the types.
- **K0-2**: If Phase 0a scaffolding needs > ~400 LOC of new code
  (not counting tests) just to preserve current behavior, the
  `module_id` keying is in the wrong place — likely needs to be
  deeper in `placer.rs` instead of the planner.
- **K0-3**: If Phase 0b (UI + WASM wiring) needs > ~200 LOC of new
  code, the strategy plumbing is leaking through too many call
  sites — `LayoutOptions` is the wrong shape or the WASM binding
  is exposing internals it shouldn't.

**Phase 1 (partitioning):**

- **K1-1** (correctness on motivating case): Once the
  `PartitionedPerConsumer` code path is wired and the partitioner's
  75%-utilization gate is satisfied for the smallest valid
  partition, the target failing case (`advanced_circuit` at a rate
  that currently trips the 8-lane ceiling) must produce a
  **validator-clean** layout. If validator warnings remain on the
  smallest gate-passing partition, the approach isn't unblocking
  the motivating problem — reassess rather than push further.
- **K1-2** (load-bearing assumption holds): If a consumer row
  emits belt-flow validator warnings (starved inserters) under
  `PartitionedPerConsumer` on a correctly-sized partition with
  balanced external inputs *and* the partitioner's 75%-utilization
  gate is satisfied, the "belts over-provisioned" assumption is
  wrong — we need balancers back, scoped per module.
- **K1-3** (utilization gate is rare-firing): If
  `TraceEvent::PartitionRejectedByUtilization` fires on > 20% of
  the stress corpus at default rates, the 75% threshold is too
  aggressive and is blocking reasonable cases; retune or treat
  partition-utilization as a per-case concern.
- **K1-4** (inert on uninvolved cases): If `PartitionedPerConsumer`
  doesn't produce an **entity-equivalent** layout to `Pooled` on
  cases with no multi-consumer intermediates (iron-gear-wheel,
  anything with K=1 for every intermediate), the partitioning code
  is doing something it shouldn't when it has nothing to do.
  "Entity-equivalent" = same validator result, same density score,
  same entity-type counts (not necessarily byte-equal since
  module_id≠0 paths will traverse different code). K1-4 replaces
  the previous "<1.5× entity count" kill criterion with a sharper
  inertness test.

**Phase 2 (decomposition):**

- **K2-1** (LOC budget): If Phase 2 needs > ~300 LOC outside the
  Phase-0 scaffolding, the split between outer/inner passes is
  wrong.
- **K2-2** (lumpiness common vs rare): If
  `TraceEvent::LumpyShardTap` fires on > 30% of the stress corpus
  runs under `PartitionedDecomposed`, lumpiness is the common case
  not the edge — reassess the strategy. There is no pre-built
  fallback (see "Consumer lumpiness — known failure mode without a
  fallback" above for the rationale). Possible responses include
  preserving today's Pooled balancer-warning behaviour behind a
  flag, or extending Factorio-SAT to widen the template library.

## Observables (reported, not gated)

These are numbers we want to watch because they describe the
tradeoff between strategies, not because we want to prevent the
tradeoff. They're reported per-run in the e2e scoreboards.

- **Entity count per strategy, per tier test.** Expected:
  `Pooled` ≤ `PartitionedPerConsumer` ≤ `PartitionedDecomposed` on
  shallow cases (more strategies = more infrastructure); inverted
  on deep cases that would otherwise fail under `Pooled`.
- **Density per strategy, per tier test** (1:1 and tight-bbox).
  The density metric added in the preceding work on this branch
  picks up the layout-shape tradeoff directly.
- **`PartitionRejectedByUtilization` events per case.** Feeds K1-3.
- **`LumpyShardTap` events per case.** Feeds K2-2.
- **Max family width per case.** Confirms the "no balancer wider
  than the largest consumer" property holds.

## Verification plan

Follow the
[verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).
Specifics:

1. **Pooled-strategy regression**: every e2e test asserts snapshot
   equality under `strategy: Pooled` with the pre-RFP baseline.
   Non-negotiable for Phase 0 landing (gated by K0-1).
2. **Strategy inertness on uninvolved cases**: for every tier test
   where every intermediate has K=1 consumer (iron-gear-wheel, any
   single-consumer chain), assert that `Pooled` and
   `PartitionedPerConsumer` produce layouts with the **same
   validator result, same density score, and same entity-type
   count breakdown**. This is the K1-4 gate — the sharpest
   available test that the partitioning code path doesn't do
   anything when it has nothing to do.
3. **Strategy-on golden cases**: add
   `tier3_advanced_circuit_partitioned` and
   `tier4_advanced_circuit_decomposed` e2e tests at rates that
   require >8 cable lanes. Assert zero validator warnings.
4. **Per-strategy tier sweep**: for every existing tier test, run
   under all three strategies and dump entity count + density
   scores into the scoreboard. These feed the *Observables*
   section, not kill criteria — we report the tradeoff, not gate
   on it.
5. **Browser verification**: load
   `?item=advanced-circuit&rate=<high>&strategy=partitioned-per-consumer`
   and eyeball the layout per CLAUDE.md verification protocol. A
   zero-warning layout that visibly has disconnected belts is a
   validator blind spot, not a success.
6. **Trace signals**: confirm `ModulePartitioned{item, modules}`,
   `ShardSplit{item, shards}`, `LumpyShardTap{item, consumer}`,
   and `PartitionRejectedByUtilization{item, lane_util}` events
   fire as expected. Absent trace events = the code path isn't
   exercising.

## Phasing

Four independently landable PRs:

- **PR 1 — Phase 0a core scaffolding**: multi-family-per-item in
  the lane planner, with `module_id=0` per item under
  `strategy: Pooled` preserving current behavior. `LayoutStrategy`
  enum + `LayoutOptions.strategy` field added but only `Pooled`
  arm is reachable.
- **PR 2 — Phase 0b surface wiring**: WASM binding for
  `strategy`, web app dropdown, URL state, threaded through call
  sites. Still no behavior change (non-`Pooled` arms still
  unreachable or panic!).
- **PR 3 — Phase 1 partitioning**: `PartitionedPerConsumer` arm
  wired through. Shared-upstream balancer for multi-row producers.
  Utilization gate. Fluids-stay-pooled carve-out. Tests for
  strategy-on path. (Earlier drafts also included a hand-authored
  `HierarchicalComposed` prototype as a K2-2 fallback; that work
  was dropped — see "Consumer lumpiness" above.)
- **PR 4 — Phase 2 decomposition**: `PartitionedDecomposed` arm
  wired through. Lumpy-tap trace event. Tests for strategy-on
  path.

Each PR must pass all kill criteria for its phase before the next
PR starts.

## Task tracker

Living checklist — update as work progresses.

### Phase 0a — core scaffolding

- [ ] Add `LayoutStrategy` enum + `LayoutOptions` struct to
      `crates/core/src/bus/layout.rs` (default `Pooled`)
- [ ] Thread `LayoutOptions` through `build_bus_layout` call sites
      (core, examples, tests)
- [ ] Extend `LaneFamily` identity to include `module_id: u32`
- [ ] Update `lane_order.rs` exact-search to key on
      `(item, module_id)`
- [ ] Update `stamp_family_balancer` to accept the tuple
- [ ] Update tap-off routing in `templates.rs` for tuple lookup
- [ ] Add consumer→module mapping defaulting to `(item, 0)`
- [ ] Non-`Pooled` strategy arms panic with "not yet implemented"
- [ ] Verify all 9 e2e tests pass with zero snapshot drift under
      `Pooled` (K0-1)
- [ ] Verify Phase 0a LOC budget ≤ 400 (K0-2)

### Phase 0b — surface wiring

- [ ] Expose `strategy` on WASM bindings
- [ ] Strategy dropdown in `web/src/ui/sidebar.ts`
- [ ] URL state: `?strategy=...`
- [ ] Verify Phase 0b LOC budget ≤ 200

### Phase 1 — demand-partitioning

- [ ] Add `ModulePartitioned`,
      `PartitionRejectedByUtilization` trace events
- [ ] Implement outer-pass partitioning in `lane_planner.rs`
      under `strategy == PartitionedPerConsumer`
- [ ] Wire in 75%-utilization gate; emit rejection event rather
      than silently fall back
- [ ] Implement producer-row splitting in `placer.rs`
- [ ] Implement shared-upstream balancer (reuses
      `balancer_library.rs`, not a new splitter-tree primitive)
- [ ] Fluids-stay-pooled carve-out in `lane_planner.rs`
- [ ] Add `tier3_advanced_circuit_partitioned` e2e test
- [ ] Per-tier strategy sweep in scoreboards (all 3 strategies ×
      all tier tests)
- [ ] Verify K1-1 on motivating failure case
- [ ] Verify K1-2 on stress corpus
- [ ] Verify K1-3 utilization gate rate ≤ 20%
- [ ] Verify K1-4 inertness on single-consumer cases

### Phase 2 — subtree decomposition

- [ ] Add `ShardSplit` and `LumpyShardTap` trace events
- [ ] Implement widest-recipe walker
- [ ] Implement proportional shard allocator
- [ ] Wire into `PartitionedDecomposed` outer pass
- [ ] Add `tier4_advanced_circuit_decomposed` e2e test
- [ ] Verify K2-1 LOC budget
- [ ] Verify K2-2 lumpy-tap rate on stress corpus
- [ ] If K2-2 fires: reassess. No pre-built fallback exists (see
      "Consumer lumpiness" — the `HierarchicalComposed` prototype
      was dropped in the review pass). Possible responses include
      preserving today's Pooled balancer-warning behaviour behind
      a flag, or extending Factorio-SAT to widen the template
      library.

### Rollout (post-all-phases)

- [ ] Run full stress corpus under all strategies; publish
      per-strategy scoreboards so users can see the tradeoffs
- [ ] Decide per kill criteria: promote strategies to tier test
      coverage, keep as opt-in, or archive specific strategy
      variants

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

- *2026-04-24 (revision pass) — Second-pass review against the draft
  above produced the following changes:*
  - *Replaced the two-boolean option surface with a
    `LayoutStrategy` enum. Motivated by (a) avoiding invalid
    combinations (`decompose && !partition`), and (b) future
    strategies (escargio `Folded`) composing cleanly without a
    cartesian product of flags.*
  - *Reframed the strategy model: `Pooled`,
    `PartitionedPerConsumer`, and `PartitionedDecomposed` are
    **peer citizens**, not degradations or upgrades of a default.
    The layout engine respects what the user selected; no silent
    auto-downgrade when a strategy can't satisfy a case — instead
    produce an invalid layout with a loud diagnostic so the user
    knows the shape they picked doesn't fit. Rationale: different
    recipes and different users prefer different shapes, and
    silently picking the "safe" one hides that tradeoff.*
  - *Defined "consumer" explicitly as one consuming recipe-row,
    not one consuming machine or one consuming item. Under-defined
    in the original draft.*
  - *Made the "belts over-provisioned" assumption numeric: 75% of
    belt tier capacity is the utilization ceiling. A dedicated
    trace event reports violations, which seeds K1-3.*
  - *Reshaped K1-1 from "entity count < 1.5× on irrelevant cases"
    to K1-4 "entity-equivalent layout on single-consumer cases".
    Entity count per strategy becomes an **observable** (reported
    in scoreboards) not a kill criterion, because the entire point
    of offering multiple strategies is that users see the
    tradeoff.*
  - *Honest about the shared-upstream distributor: it's a
    small-scale balancer reusing `balancer_library.rs`, not a
    "simple tee". The "no pool-balancer" claim applies at the
    output side, not the input side.*
  - *Fluids explicitly out of scope for partitioning (pipe
    networks merge freely, no per-lane identity to partition).*
  - *Added the `HierarchicalComposed` prototype to Phase 1 to
    de-risk the K2-2 fallback: if decomposition produces too many
    lumpy taps, we pivot to a strategy that already exists in
    working code rather than a hypothetical one.*
  - *Split Phase 0 into 0a (core, ≤400 LOC) and 0b (surface
    wiring, ≤200 LOC) for reviewability. Previous single budget
    of 400 LOC was optimistic given the WASM-and-UI-threading
    costs.*
  - *Added K1-4 inertness test + snapshot-equality-on-irrelevant-
    cases verification step. Sharpest available "new code path
    does nothing when it should do nothing" check.*
  - *Noted that escargio (folded layouts, separate upcoming RFP)
    will slot in as additional `LayoutStrategy` variants on top of
    this scaffolding. The `module_id` keying in Phase 0a should
    not bake in any row-orientation assumption; a row's
    `(item, module_id)` identity is orthogonal to its spatial
    orientation.*

- *2026-04-24 (review pass) — Third pass against the revised draft.
  Fixes:*
  - *Phase 2 section still referenced the pre-revision booleans
    (`subtree_decompose = true` / `demand_partition = true`).
    Rewritten in `LayoutStrategy` terms.*
  - *Code-block `LayoutOptions` was implied to extend an existing
    struct; clarified that it is new and absorbs today's
    `max_belt_tier` arg. Function signature corrected to
    `Result<LayoutResult, String>`.*
  - *Added K0-3 to gate Phase 0b's ≤200 LOC budget (the task
    tracker checkbox previously had no kill criterion behind it).*
  - *Reframed K1-1 from a wall-clock criterion ("within 2 weeks")
    to a behavioural one (validator warnings on the smallest
    gate-passing partition). Time-based kill criteria are hard
    to act on consistently.*
  - *Made the up-tree recursion of the partitioning algorithm
    explicit as step 6, instead of leaving it to be inferred from
    the shared-upstream prose.*
  - *Added verification step 7 + a task-tracker checkbox for a
    smoke test of the `HierarchicalComposed` prototype. The
    prototype's job is to be a real, exercised fallback for
    K2-2 — an unexercised one doesn't de-risk anything.*

- *2026-04-25 — `HierarchicalComposed` prototype dropped during
  Phase 1 PR3. Authoring a Factorio-correct 12→12 = 8→8 + 8→8 +
  mixer balancer is real belt-mechanic engineering — the existing
  `balancer_library` templates are machine-generated by
  Factorio-SAT precisely because hand-authoring is fiddly with
  sharp edges. We do not have the engineering bandwidth to do it
  as a side quest, and Phase 1's per-consumer partitioning
  already addresses the immediate motivating case
  (`advanced_circuit` copper-cable: each module is bounded by a
  single consumer's demand and fits the 8→8 template). The
  trade-off: if K2-2 fires under Phase 2's stress runs, we
  reassess rather than swap in a pre-built fallback. Removed: the
  `LayoutStrategy::HierarchicalComposed` enum variant and dispatch
  arm in `bus/layout.rs`; verification step 7; the prototype +
  smoke-test task-tracker entries; the K2-2 promotion task in the
  Phase 2 tracker. Rewrote "Consumer lumpiness" and K2-2's kill-
  criterion text to make the no-fallback stance explicit.*
