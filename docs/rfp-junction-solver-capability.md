# RFP: Junction-solver capability

## Summary

The ghost-router junction solver gives up on dense crossing clusters
more often than it should. Cases where 5 specs need to cross fail
when 4 of them solve cleanly — the problem isn't structural
infeasibility, it's solver capacity. This RFP proposes a phased
investigation into making the solver handle higher spec counts via
adaptive growth budgets, underground-belt escape on growth-failure,
and pre-fragmenting dense clusters at the router level. Junction
capacity is the bottleneck that gates Phase 2 of
`rfp-modular-production` and any future bus topology that creates
parallel trunks; improvements here are upstream of decomposition,
horizontal trunks, and the modular-production strategies broadly.

## Motivation

Concrete failing case: `processing-unit @ 3/s` from plate inputs at
yellow belt tier, under `LayoutStrategy::PartitionedDecomposed`. From
the `partition_strategy_scoreboard` test and the
`JunctionBlamedSpec` diagnostic introduced on PR #238:

```
capped clusters:                     8
clusters with single-spec blame:     5    (62.5% one-spec failures)
clusters multi-spec entangled:       3    (37.5% need >1 removal)

blame by item:        4× iron-plate, 1× electronic-circuit
blame by direction:   5× East
spatial pattern:      4× iron-plate East at column x=38,
                      y=350/358/366/374 (every 8 rows)
```

Reading: the same tap-off shape (iron-plate east-flowing belt
crossing N other specs at consumer-row pitch) keeps tripping the
solver in 4 separate clusters. **Removing iron-plate makes each of
those clusters solvable** — so the failure is "solver can't pack 5
specs in this bbox", not "this geometry is infeasible".

Each capped cluster leaves orphan ghost belts in the layout; the
fix in PR #238 (`check_unresolved_junctions` + belt-item-isolation
filter) makes them visible as one error per cluster instead of
40+ misleading belt-adjacency errors, but it doesn't actually solve
the underlying junctions. PU@3/s plates yellow P2 still has 34
unresolved-junction errors after that fix.

Performance is regressed too. PU@4/s plates yellow under P2 takes
**290s vs P1's 121s** in the scoreboard probe — the partitioner
intended to take load off the SAT solver via per-consumer
partitioning + sharding, but Phase 2's sharding multiplies consumer
rows, which creates more clusters, which the solver then fails on
more often. Until the solver scales with cluster density, every bus
topology that creates parallel trunks has a hard ceiling at the
current solver capacity.

## Design

This is an investigation, not a single fix. The proposal is a phased
approach where each phase has a well-defined diagnostic outcome that
informs the next.

### Phase 1: Diagnostics expansion (~1 week)

Two extensions to the existing `JunctionBlamedSpec` infrastructure:

- **Pair-removal blame.** When single-spec blame fails (the 3/8
  multi-spec cases), retry with each pair of specs removed. Cost
  is O(N²) extra solves per cluster, but only for the cluster that
  single-removal couldn't characterise. Emits
  `JunctionBlamedSpecPair { spec_a, spec_b, ... }`.
- **Per-cluster solver-cost telemetry.** Augment
  `JunctionGrowthCapped` with `sat_invocations_attempted`,
  `final_bbox_w`, `final_bbox_h`, `participating_count`. Lets us
  build a histogram: at what cluster size does the solver fall over?

Goal: answer the question "is the failure population dominated by
one shape, or is it a long tail?" Without that, picking between
options 2–4 below is guesswork.

### Phase 2: Adaptive growth budget (~1 week)

Today `MAX_GROWTH_ITERS` in `junction_solver.rs` is uniform across
all clusters. Hypothesis: high-spec clusters need a bigger region
to find a feasible packing, but the iteration cap fires before the
SAT solver even gets a chance.

Concrete change: scale the iteration cap as a function of
`participating_count`. Something like
`base_iters + N_specs * extra_iters`. Tune `extra_iters` against
the scoreboard.

### Phase 3: Underground-belt escape on growth-fail (~2-3 weeks)

The biggest leverage. When a cluster caps, instead of giving up,
pick the most-conflicted spec (the one whose path crosses the most
other specs) and route it underground over the cluster bbox + a
margin. Then retry the cluster with that spec excluded.

Geometric constraints:

- UG max-reach is belt-tier dependent (yellow=4, fast=6, blue=8).
  Cluster must fit within reach + entry/exit tile.
- Pairing logic must validate the UG entry/exit don't conflict with
  other specs' surface paths.
- The "evicted" spec's full path through the bbox is replaced by
  a two-tile UG entry/exit pair plus a tunnel; cost ~2 entities
  per spec.

This is essentially extending the partitioner's options into the
router: when partitioning can't reduce the per-junction spec count,
the router does it locally via UG bypass.

### Phase 4: Pre-junction cluster splitting (~3+ weeks)

If diagnostics reveal that a single tile cluster routinely has 6+
specs participating, splitting the cluster into spatial sub-clusters
(by direction, by item, by entry side) before invoking
`solve_crossing` would attack the density problem at its source.
Same total spec count, but distributed across multiple smaller SAT
problems.

This is the most invasive change and the most uncertain — sub-cluster
solutions can spatially conflict with each other, and stitching them
together may need a second-pass merge phase. Defer until phases 1–3
have data demonstrating that density (not capacity per se) is the
real bottleneck.

### Phase 5 (optional): More balancer templates (orthogonal)

Tracked as #136. If the balancer library had templates for 9–12
wide cases, Phase 2's `SHARD_THRESHOLD_LANES = 10` could go higher
or vanish entirely — less sharding → fewer consumer rows → fewer
clusters → less solver pressure. This is bus-engine work, not
solver work, and lives on its own track. But it's part of the same
problem space and worth flagging here.

### What this RFP rejects

- **Increasing the `MAX_GROWTH_ITERS` ceiling uniformly** — feels
  like the obvious cheap fix but the diagnostic data shows growth
  is hitting cap *before* the SAT problem becomes infeasible, not
  *because* the SAT problem is infeasible at the current size. A
  uniform raise would slow easy clusters without unlocking hard
  ones. Adaptive (Phase 2) is the right shape.
- **Replacing the SAT solver** — varisat works fine on the
  problems it's given. The cost isn't "the SAT solver is slow",
  it's "we hand the SAT solver problems it can't always solve in a
  reasonable time". Algorithm work happens above SAT, not below.
- **Manually authoring crossing templates for common shapes** — a
  template library (similar to balancers) would handle frequent
  patterns but doesn't generalise. The blame data shows the *same*
  iron-plate-East-at-x38 pattern recurring 4× across one factory;
  templating that one shape might unlock 4 clusters. But the
  pattern's specifics depend on the factory layout, and we'd be
  authoring templates indefinitely.

## Kill criteria

These are cumulative — if any trigger, the RFP either pivots or
shuts down.

1. **Phase 2 must reduce capped-cluster count by ≥30% on the
   scoreboard's PU@3/s plates yellow case (currently 8 clusters; 5
   or fewer after).** If adaptive growth doesn't help at all on the
   case it's most obviously suited to, the bottleneck isn't growth
   budget and Phase 3 likely won't help either.

2. **After Phase 3, the scoreboard must show a strictly-monotonic
   improvement in `JunctionGrowthCapped` count from Pool → P1 → P2
   on at least one case — i.e. P2 capped count < P1 capped count.**
   Today P2 has *more* capped clusters than P1 (8 vs 3 for PU@3/s
   plates yellow). The whole point of P2 is more, smaller modules
   that should be *easier* to route; if the solver can't keep pace
   even with UG escape, the modular-production approach is gated
   on Phase 4 or a different topology entirely.

3. **End-to-end runtime regresses by >2× on the existing scoreboard
   corpus across phases 1–3.** Hard ceiling: this work is meant to
   make P2 viable at scale; if the cure is slower than the disease,
   we drop it.

4. **If Phase 1 diagnostics reveal that the failure population is
   dominated by structurally unsolvable clusters (i.e. pair-removal
   blame finds that even removing 2 specs doesn't help in >50% of
   cases), accept "junction unsolvable" as a real layout limit and
   stop trying to solve them.** Instead, surface the structural
   limit upward: the partitioner should refuse to shard items whose
   downstream junctions are guaranteed to fail. That's a different
   project; this RFP closes.

## Verification plan

Per CLAUDE.md verification protocol:

1. **Scoreboard regression gate.** PR #238's
   `partition_strategy_scoreboard` is the ground truth. Each phase's
   PR must hold the existing numbers (no regression on Pool / P1)
   and demonstrate improvement on at least one P2 case. New cases
   added as the solver unlocks higher rates: PU@4/s plates yellow
   (currently times out / errors profusely), PU@5/s ore red
   (currently panics in lane_planner.rs:557).

2. **Capped-cluster count tracking.** Each phase's PR description
   must include a before/after table of `JunctionGrowthCapped` event
   counts on the scoreboard cases. Falling counts are the success
   metric; growing counts are a regression.

3. **Blame-event triangulation.** Re-run the blame probe (set
   `SPAGHETTIO_BLAME_JUNCTIONS=1`) before and after each phase; the
   set of "blamed item × direction" patterns should shrink as the
   solver learns to handle them. Patterns that persist across
   phases identify what the next phase needs to attack.

4. **Browser smoke test.** After each phase, load the scoreboard
   cases at the URLs and visually confirm the layout looks
   reasonable — fewer orphan ghost belts, fewer disconnected
   trunks. Type-checking and zero-error counts can lie; the visual
   tells you whether the solver actually solved or just failed
   silently in a different place.

## Phasing

| Phase | Scope | Effort | Lands as |
|-------|-------|--------|----------|
| 1 | Diagnostics expansion | ~1 week | One PR |
| 2 | Adaptive growth budget | ~1 week | One PR |
| 3 | UG-escape on growth-fail | ~2-3 weeks | 1-2 PRs |
| 4 | Pre-junction cluster splitting | ~3+ weeks | 2-3 PRs |
| 5 | More balancer templates (parallel track) | unknown | separate |

Phases 1–3 land sequentially with kill-criteria gates between. Phase
4 only kicks off if 1–3 land and *still* don't move the scoreboard
on the pathological cases. Phase 5 is independent and can run in
parallel with any other phase.

Each phase is one or two PRs. Total work, if all phases run, is on
the order of 6–10 weeks; if kill criteria fire, considerably less.

## Decision log

- *2026-04-27 — proposed. Motivating data captured in PR #238
  blame diagnostic. Pending review.*
