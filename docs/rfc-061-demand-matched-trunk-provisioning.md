# RFC-061: Demand-matched trunk provisioning

**Status**: Active
**Tracking**: #519 (layout-side fix)
**Owner branch**: `fix/519-row-input-margin` superseded → new branch per phase

## Summary

Consumer rows that need K > 1 trunk feeds of one item are today fed by
**topologically disjoint producer subsets** — whichever producer outputs
the ghost router happens to tap into each trunk. Allocation is
demand-blind and frozen at plan time: no in-game backpressure can move
flux between disjoint subsets. This RFC pools each item's supply and
balances it across the K trunk feeds, so subsets share and the network
self-corrects.

## Evidence (probe, 2026-07-31 — `probe_519_allocation`)

On `ac@5 am2 plates yellow` (the RFC-060 flipped case, sim-measured at
75% of plan while validator-clean pre-#525):

- The 17 copper-cable producers partition into **5 disjoint
  reachability subsets** over the cable belt graph.
- The two subsets feeding the EC row's 3-machine blocks are
  **structurally under-provisioned**: 3 producers × 2.94/s = 8.82/s
  against 3 × 4.29 = 12.86/s demand — twice.
- One EC subset and both AC subsets have surplus (8.82 vs 4.29;
  11.76 vs 10.00 ×2) that **cannot reach** the starved blocks.
- Deliverable sum ≈ 21.9/s vs the sim-measured 22.5/s consumption and
  the exact-75% production cascade — the partitioning explains the
  measured deficit to within utilization rounding.

The #525 walker sees the same thing statically (that is what the 11
pinned `input-rate-delivery` warnings on `tier4_am2` are), and the
`ROW_INPUT_PLANNING_MARGIN` probe (parked branch) showed row-split
margins make this WORSE: more taps split the same frozen subsets
thinner (tier2 4→5 warnings at 0.95 margin).

## Design

At the K-trunk feed seam (the caller of `dual_input_row_horizontal`,
and the general tap-segment planning for vertical rows' current-feed
segments):

1. **Pool**: the item's producer-row output belts merge into a single
   logical source pool (they already converge structurally in most
   layouts — the pooling makes it a contract).
2. **Balance**: insert an (n → K) balancer from the library (or a
   splitter tree where the library lacks the shape — the
   `missing-balancer-template` warning path already names this) between
   the pool and the K trunk heads.
3. **Audit**: a per-layout allocation audit (the Phase-0 probe as a
   library function) asserts no consumer group's reachable supply is
   below its demand. This is the plan-time counterpart of the #525
   walker's flow-time check.

Explicitly NOT in scope: belt-tier escalation (user-capped, hard
constraint); row-geometry changes (measured a net loss, see the parked
margin probe); the merge-aware demand-attribution rework (recorded
separately on #519 — this RFC fixes the plan, that one fixes the
model's absolute attribution).

## Kill criteria

- **K61-1 (seam)**: after Phase 1, the allocation audit still reports an
  UNDER group on any of the four #519 corpus cases (`ac@5`, `ac@7`,
  `pu@3`, `ec@15`) → the balancer insertion point is wrong; stop and
  re-locate the seam rather than special-casing.
- **K61-2 (runtime)**: e2e suite wall-clock exceeds 1.15× (balancer
  SAT/library lookups are cached; anything above this means live
  solving leaked into the default path).
- **K61-3 (sim, the decisive gate)**: a Phase-1-complete `ac@5` layout
  sims **below 95% of plan** at long warmup → balancing the trunks does
  not recover the measured deficit for this class; the approach is
  falsified, revert default-on.
- **K61-4 (scope)**: net engine LOC beyond ~500 → wrong integration
  point.

## Verification plan

1. Phase-0 probe tracked as an `#[ignore]` e2e instrument + allocation
   audit as a reusable test helper.
2. Full suite: the 12 #525-re-blessed warning pins must move DOWN
   (each retightened in the same commit that moves it); registry/golden
   churn adjudicated per fixture.
3. Sim close-loop: re-export `ac5`/`pu3` via `rfc060_sim_export`,
   re-run the harness at long warmup; K61-3 gates on `ac@5 ≥ 95%`,
   `pu@3` recorded (its chain shares the mechanism but adds fluid
   stages; not gated in Phase 1).
4. Browser eyeball of `ac@5` (balancer visible between cable row and
   EC trunks, no orphaned belts).

## Phasing

- **Phase 0**: probe + audit instrument (this session's evidence,
  committed).
- **Phase 1**: (n → K) balancer insertion for `dual_input_row_horizontal`
  K-trunk feeds (the measured ac5/ac7 mechanism).
- **Phase 2**: general tap-segment provisioning for vertical rows'
  current-feed segments (pu@3's remaining deficit share).
- **Phase 3**: fold the allocation audit into the validator as a
  plan-time check (one positioned issue per UNDER group), retire the
  #525 selection-count exemption once K61-3's sim anchor exists (the
  #520 teeth — coordinate with rfc-lane-demand-flow's decision log).

## Decision log

- *2026-07-31 — RFC opened on the day's measurements: the #525 walker
  recalibration (reporting), the parked `ROW_INPUT_PLANNING_MARGIN`
  probe (row margins measured-rejected: tier5 −1 warning, tier2 +1
  net, 8 fixtures churned), and `probe_519_allocation` (disjoint
  producer subsets explain ac@5's measured 75% to within rounding).
  Balancer-pooling chosen over demand-matched partitioning: per-machine
  rates (2.94 supply vs 4.29 demand) never divide evenly, so any fixed
  partition strands remainder flux; pooling lets backpressure do the
  fractional allocation the plan cannot.*
- *2026-07-31 — **Phase 1 lands; K61-1 and K61-3 both clear.** Root cause
  narrowed one level below the RFC's framing: the (m, m) family
  passthrough (issue #268: "balancing is unnecessary for a fungible
  item") freezes a producer→column pairing; under demand skew some
  column's consumers outdraw its producer and the real library
  templates — "kept as a safety net but never consulted" — were exactly
  the unused fix. `LaneFamily::demand_skewed` (permutation-invariant:
  max per-lane demand > min per-producer supply) routes skewed families
  to the real template. K61-1: the ac@5 allocation probe collapses from
  5 disjoint groups (2 UNDER) to one unified pool. K61-3, measured on
  the real harness at long warmup, both HS cases: **ac@5 3.75/s → 4.84/s
  (75.0% → 96.8% of plan)**, **ac@7 6.00/s → 6.68/s (85.7% → 95.4%)**,
  both converged, machine censuses from mass-starvation to 78/81 and
  108/113 working.*
- *2026-07-31 — Phase 1 SCOPED to HS K-trunk families after the corpus
  measured mixed movement on VerticalSplit families (tier5's walker
  count +14 with no sim evidence either way): VS keeps the passthrough
  until Phase 2 runs its own K61-3-style gate. One sim-adjudicated
  tolerance rides along: ac@7's (6,6) stamp takes a feeder arrival on
  its flank as a UG sideload — a real single-lane hazard the validator
  correctly names, measured off the critical path (95.4% with the tile
  flagged); fixing the feeder approach is Phase 1.5 and removes the
  tolerance. Zone-cache pin refreshed deliberately for the new balancer
  routes (ci.yml protocol).*
- *2026-07-31 — **Phase 1.5 complete.** The ac@7 flank sideload was
  misread in Phase 1 as template-internal wiring: the offending tile is
  a `trunk:copper-cable` belt. Root cause: THREE independent mirrors of
  the passthrough decision (stamper — gated in Phase 1; ghost-router
  feeder targeting and lane-planner height resolution — ungated), so a
  skewed family got a real 19-tall template while feeders aimed at
  phantom passthrough columns and trunks started 18 rows early, driving
  through the template body. `family_uses_passthrough` is now THE
  single predicate at all three sites. tier4_7s passes with no
  tolerated categories; the UG-entry-sideload check gains a narrow
  intra-balancer exemption (same `balancer:` segment both tiles) for
  SAT-baked internals — routed belts entering sideways still warn,
  which is exactly how this bug stayed visible. Re-sims: ac@5 4.82/s
  delivered / 4.92 produced (96.4%/98.4% of plan), ac@7 6.68/s (95.4%,
  converged) — K61-3 numbers hold; the fix is structural, not
  throughput-regressive.*
