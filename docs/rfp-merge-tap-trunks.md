# RFP: Merge-and-tap trunks — retiring large balanced lane families

## Summary

When an item's lane family splits, the engine today gives every
consumer row its own dedicated lane and stitches producers to lanes
through an (N,M) balancer template. That architecture manufactures
balancer shapes no human has ever built — utility-science at 10/s
demands (15,14), (12,7), and (8,19), none of which exist, and the
resulting silent failure orphans 35 producer rows (95% of that cell's
1,636 validation issues). This RFP replaces large balanced families
with the human pattern: **merge N producer outputs onto K =
ceil(rate/belt_cap) trunk belts via splitter merge-trees, and let
consumer rows tap those shared trunks with priority splitters,
letting belt backpressure do distribution**. Merging is associative —
(n,1) mergers compose from small library atoms with no coprime
arithmetic, ever — and the demand-pull walker (landed 2026-07-12)
models exactly the backpressure physics that makes tap distribution
work, which the old validator could not. Rollout is staged like the
netflow compat flip: merge-and-tap lands first as the FALLBACK for
families with no balancer template (strictly additive — fixes the
broken cells, zero movement for working layouts), then flips to the
default for all split families, then the balancer-family machinery
retires to the shapes that genuinely need fairness. Expected side
effect, measured as first-class evidence: bus width and area drop
substantially — oversized balancers were the July review's dominant
lever on the 2–3× area gap vs human builds.

## Motivation

Reproducible today (`science_scaling_gauntlet`, utility@10/s):
35 belt-dead-end errors — 15 copper-cable + 12 electronic-circuit +
8 iron-plate producer rows, 100% of those items' rows — because their
family shapes have no template, no gcd decomposition (coprime), and
no runtime-generator coverage, and the feeder-spec generator then
silently skips every feeder (`FeederSpecsSkipped` trace event, added
`e050465`, makes this visible). 1,557 of the cell's 1,636 issues
(95%) cascade from those three orphaned lanes.

The deeper problem is the architecture that demands those shapes:
`split_overflowing_lanes` forces one lane per consumer row
(`n_splits ≥ consumer_rows.len()`), so family fan-out scales with
CONSUMER COUNT rather than throughput. At 10/s, 14 consumer rows of
copper-cable ⇒ a (15,14) balancer — while the actual throughput
needs K = ceil(rate/45) ≈ 2–3 blue belts. Humans merge onto
throughput-sized belts and tap them; balancers appear only where
fairness under scarcity matters (train unloading). "Relatively
efficient but pretty boring layouts" is the project goal; merge-and-
tap IS the boring human answer.

Why now and not before: the balancer-centric design was partly a
VALIDATOR artifact. The old lane-rate walker modeled every splitter
as a static 50/50 and every consumer as a fixed share — balanced
per-consumer lanes were the only topology it could verify. The
demand-pull walker (rfp-lane-demand-flow Phase 1, `6849935`) models
splitter backpressure redistribution and per-tap demand natively.
The engine can now verify what humans build.

## Prior art (read before re-litigating)

- **PR #272 / docs/rfp-balancer-place-routing.md (May 2026, closed
  unmerged)**: CP-SAT flow-conservation SYNTHESIS of arbitrary
  balancer shapes — 3/8 round-trips, stalled on greedy slot
  assignment for coprime back-loop shapes; the overnight CP-SAT runs
  put 0/32 seeds on (1,9)/(1,10). The "make balancers harder"
  direction is researched and parked. This RFP is the opposite move:
  need balancers less.
- **#136 (closed)**: the previous coprime-shape wall, closed via
  `balancer-gen bake_missing_shapes` composition recipes — the
  precedented interim fix, and still the fallback interim here if
  this RFP stalls (the 2026-07 spike's shape census doubles as the
  bake worklist).
- **Uniform output balancing (design fossil)**: producer rows emit
  uniformly at the midpoint BY DESIGN to match vertical-split
  balancer semantics. Its premise — the validator can't reason about
  anything but even splits — was removed by the demand-pull walker.
  This RFP retires the premise's consequences deliberately, not
  accidentally.
- **Lane physics (hard-won)**: sideloads fill ONE lane of the target;
  the engine's belts are kept lane-balanced today by the vertical-
  split row trick (split the machine row in half, jiggle half the
  content onto each lane) — crude but working. Splitters are
  lane-preserving; sideloads are not. Hence D3 below.

## Design

### D1. Throughput-sized trunks

`split_overflowing_lanes` (lane_planner.rs): a family's lane count
becomes `K = ceil(family_rate / lane_capacity)` — throughput-driven,
not consumer-driven. Consumer rows are assigned to trunk lanes by
deterministic demand bin-packing (largest-demand-first into
least-loaded lane; ties by existing row order — determinism is a hard
project contract). Multiple consumer rows share a lane via the
EXISTING multi-tap machinery — single-lane multi-consumer items are
already the engine's workhorse path; this design extends the
workhorse to K-lane groups instead of escalating to balanced
families.

### D2. Merge-trees replace family balancers

Producer rows are partitioned across the K trunks (same bin-packing
discipline, producer rates against trunk capacity). Each trunk's
producer group merges via a splitter merge-tree stamped from (n,1)
merger atoms — the library already holds these, and merging is
associative, so ANY n decomposes into small atoms: the coprime
problem cannot exist on this path. The feeder-spec generator's
balancer-mirroring branches are replaced by (or fall back to) the
merge-tree path.

### D3. Splitter-only plumbing (lane-balance invariant)

All new plumbing — merge-trees and taps — is SPLITTER-BUILT.
Sideloads are forbidden in v1 merge-tap paths: splitters preserve
lane balance end-to-end, so the vertical-split row trick's output
guarantee (both lanes loaded) survives the merge-tree and reaches
every tap intact. (Future work may relax this where lane discipline
is provable; not v1.)

### D4. Priority tap-offs

A consumer tap is an inline splitter on the trunk: one output
continues the bus, the other feeds the row, with Factorio 2.0 native
`output_priority` set toward the TAP (human bus discipline:
consumers fed first, surplus flows on). The blueprint export already
carries priority fields (sushi/sorter work) and the rate walkers
already model priority splitters (`loop_priority_rate` vocabulary,
self-loop work). Priority-toward-tap directly mitigates the
transient-starvation caution (D6).

### D5. Where balancers remain

- External-input distribution and any point where fairness UNDER
  SCARCITY is a requirement rather than throughput delivery — the
  Phase 0 census decides whether any such points exist in the
  current corpus (suspicion: few or none).
- Small shapes already working may be left on the balancer path
  during the fallback phase (Phase 1) and retired at the flip
  (Phase 2) — the flip is evaluated, not assumed.

### D6. The scarcity/transient question (answered, not shrugged)

Balancers guarantee fair split under scarcity; merge-and-tap
guarantees steady-state delivery when supply ≥ demand (solver plans
supply == demand exactly). Transients (ramp-up, hiccups) favor
upstream taps; the tail tap recovers last. Position: this is the
accepted behavior of every human main bus ever built, priority taps
soften it further, and the demand-pull walker verifies steady-state
delivery per tap. Phase 0 records this as an explicit accepted
semantic, and kill criterion 5's in-game anchor checks a tail tap
specifically.

### Validation

- The demand-pull walker already models shared-lane taps and
  priority splitters — the family uniform-rate bookkeeping
  (`LaneSplit` rate = total/n) retires with the balancers it
  described.
- Per-trunk saturation: Σ(assigned producer rates) ≤ lane capacity
  and Σ(assigned consumer demands) ≤ lane capacity, enforced by the
  bin-packing and validated by the existing lane-throughput checks.
- Merge-tree structural validation rides the existing belt
  structural/flow checks (splitter pairing machinery is mature).

## Kill criteria

1. **Phase 0 blast-radius gate**: the shape census (2026-07 spike)
   quantifies how many families across the 24-cell corpus would move
   to merge-and-tap and their K values. If the affected set is tiny
   AND all demanded shapes are cheaply bakeable compositions, this
   RFP is over-engineering — close it and bake shapes instead.
   (Falsifiable both directions: the utility@10/s evidence suggests
   it is not tiny.)
2. **Fallback-phase inertness**: Phase 1 (merge-tap as fallback for
   template-missing families ONLY) must produce ZERO golden movement
   and zero warning-count movement on every fixture that has
   templates today. If it can't be made strictly additive, the
   integration design is wrong — stop.
3. **The broken cells must actually heal**: after Phase 1,
   utility@10/s belt-dead-ends 35 → 0 and its cascade (≈1,522
   issues) collapses. If dead-ends go to zero but the cascade
   doesn't, the diagnosis was incomplete — stop and re-probe.
4. **Width/area must improve at the flip**: Phase 2 (default flip)
   is accepted only if gauntlet-corpus area (W×H) and entity counts
   are ≤ baseline on net, with per-fixture deltas reported —
   trading balancer footprint for K trunks + merge-trees must be a
   measured win, not an assumed one. Any fixture regressing >10% in
   area without a documented cause blocks the flip.
5. **In-game anchor (tail tap)**: one flipped layout gets an in-game
   check on its LAST tap along a shared trunk at planned rates —
   sustained full utilization in steady state. Starvation there with
   green validators = the walker's tap model is miscalibrated — stop
   everything (validator-debt, same standard as the inserter RFP).
6. **Router-pressure guard**: junction/SAT failure counts
   (unresolved-junction, CrossingZoneSkipped, RouteFailure) must not
   increase on any corpus cell at the flip — tap-density per lane
   rises by design, and if the ghost router can't absorb it, that's
   a real cost to surface, not bury.
7. **Scope trigger**: >~800 LOC of new lane-planner/stamper code →
   pause and review against the bake-shapes interim.

## Verification plan

Per the CLAUDE.md protocol, plus:

- Both gauntlets before/after at each phase; utility@10/s is the
  motivating fixture (35 dead-ends → 0); full delta tables to this
  decision log.
- Area/entity deltas as FIRST-CLASS evidence at the flip (kill
  criterion 4) — this RFP claims an efficiency win; measure it.
- Predict-then-verify re-blessing (the inserter-RFP discipline):
  Phase 0 freezes expected per-fixture movement for the flip; a
  blessing that misses its prediction is a stop signal.
- Browser eyeball of a flipped high-fanout layout (user validates —
  the merge-trees and taps should LOOK like a boring human bus).
- Netflow determinism sweep byte-identical throughout.

## Phasing

- **Phase 0 — census + frozen decisions**: shape census (running
  spike) → blast radius + K table; freeze bin-packing rules, the
  balancer-retention boundary (D5), the D6 semantic; predict
  per-fixture flip movement.
- **Phase 1 — fallback lands**: merge-tree stamper + K-trunk
  splitting + priority taps, activated ONLY where
  `stamp_family_balancer` finds no shape (the FeederSpecsSkipped
  population). Strictly additive; broken cells heal; kill criteria
  2–3 evaluated.
- **Phase 2 — the flip**: merge-and-tap becomes the default for all
  split families; balancer families retire to the D5 boundary.
  Kill criteria 4–6 evaluated; full re-bless against Phase 0
  predictions.
- **Phase 3 — cleanup + harvest**: retire dead balancer-family code
  paths (feeder mirroring, family uniform bookkeeping), area-win
  accounting in CLAUDE.md, #135/#136 closed or re-scoped to
  whatever balancer usage remains.

## Decision log

- *2026-07-13 — drafted, following the utility@10/s topology
  diagnosis (three coprime family shapes → silent feeder skip → 95%
  of the cell's issues), the visibility fix (`e050465`), and the
  design conversation: merge-and-tap is the human pattern, the
  demand-pull walker removed the validator limitation that made
  balanced families the only verifiable topology, and prior art
  shows the synthesis direction (PR #272, CP-SAT walls) is parked
  while composition baking (#136) remains the interim fallback.
  Shape census spike in flight; its results land in Phase 0.
  Pending adversarial review.*
