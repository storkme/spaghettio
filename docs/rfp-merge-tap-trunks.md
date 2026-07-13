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
  unmerged)**: CP-SAT flow-conservation placement/routing SYNTHESIS
  of arbitrary balancer shapes — 3/8 round-trips, stalled on greedy
  slot assignment for coprime back-loop shapes; the later (4,9)
  Clos attempt died on memory exhaustion. Separately, **the
  Factorio-SAT offline bake wall** (docs/bake-overnight-results.md,
  behind #136): 0/32 overnight seeds on (1,9)/(1,10). Two different
  tools, two different failure modes, one conclusion: the "make
  balancers harder" direction is researched and parked. This RFP is
  the opposite move: need balancers less.
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
becomes **K = ceil(family_rate / full_belt_cap)** — throughput-
driven, not consumer-driven. The capacity constant is PINNED (review
round 1): `full_belt_cap = 2 × lane_capacity()` — D3's splitter-only
plumbing gives merge-tap trunks the same both-lanes-loaded guarantee
balancer-fed trunks have today, matching the existing precedent at
lane_planner.rs:692-696 ("the balancer feeds full belts"). Using the
conservative half-belt constant would double K corpus-wide and
silently defeat kill criterion 4. Consumer rows are assigned to
trunk lanes by deterministic demand bin-packing (largest-demand-first
into least-loaded lane; ties by existing row order — determinism is
a hard project contract).

**Multi-tap status, stated honestly (review round 1)**: shared
trunks are proven today only for EXTERNAL-input lanes
(`LaneConsolidated`, lane_planner.rs:660-674); intermediate lanes
are forced to one-trunk-per-consumer (`n_splits.max(consumer_rows
.len())`, line 673, with a live "multiple consumers in one trunk is
a non-starter" comment referencing the OLD single-tap renderer). The
ghost router's general multi-tap rendering (rfp-unified-belt-specs
Phases 1+2, ghost_router.rs:2182-2264) has since superseded that
renderer but has NEVER been exercised on a family-tagged lane —
Phase 1 treats the family+multi-tap combination as new integration
territory with its own tests, not a drop-in. The round-robin
assignment code at lane_planner.rs:828 (currently dead — the floor
makes `i % effective_n_splits == i`) becomes live and must be
replaced by the bin-packing above.

### D2. Merge-trees replace family balancers

Producer rows are partitioned across the K trunks (same bin-packing
discipline, producer rates against trunk capacity). Each trunk's
producer group merges via a splitter merge-tree. Merging is
associative, so ANY n decomposes into small steps and the coprime
problem cannot exist on this path — but the BUILDER IS NEW CODE
(review round 1): the library holds fixed pre-baked (n,1) lookups
only to n≈10, and the sole runtime composition primitive is a (2,1)
splitter replicated in parallel (`balancer_generate::two_to_one`) —
a recursive n-ary merge-tree generator does not exist and is a
named Phase 1 deliverable, sized against the scope trigger (simple
— no combinatorial search, unlike balanced synthesis — but not
free). The feeder-spec generator's balancer-mirroring branches are
replaced by (or fall back to) the merge-tree path. Geometry note:
merge-trees live where the family balancer stands today (the trunk
head zone); Phase 0 confirms K parallel merge-trees fit the head
area a single (N,M) balancer occupies (expectation: comfortably —
they're narrower — but measured, not assumed).

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
consumers fed first, surplus flows on). Plumbing status, verified
(review round 1): `PlacedEntity` carries the priority fields
(models.rs:178,182) and blueprint export round-trips them — that
half is real. **The validation half is NOT built and is a named
Phase 1 prerequisite**: no validator reads the real priority fields
(zero references in validate/); the only priority-aware walker
mechanism is `loop_priority_rate`, gated on a `:selfloop:`
segment-tag string match, so a bus tap falls back to the generic
splitter model — and a BACKWARDS-STAMPED TAP (continuation and feed
swapped) would pass every existing check silently.
**Phase 1 deliverable: generalize priority-branch detection past
`:selfloop:`** — a tap-segment convention (e.g. `:tap:` segment
kind) honored by both rate walkers (belt_flow.rs:2345-2354,
belt_structural.rs:1041-1050), plus a structural check that a tap's
priority points at the feed branch. Kill criterion 5's in-game
anchor is the calibration for this new modeling, not a substitute
for it.

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
- **Phase 1 — fallback lands**: merge-tree BUILDER (new, D2) +
  K-trunk splitting + priority taps + the priority-aware walker
  generalization (D4 prerequisite), activated ONLY where the shape
  is unstampable — decided at PLANNING time via the EXISTING
  `shape_is_stampable(n, m)` (balancer.rs:143, live in production
  via shape_fix.rs and decomposition_search.rs:166 despite a stale
  `#[allow(dead_code)]`; it mirrors `stamp_family_balancer`'s full
  decision tree), called at `split_overflowing_lanes`' family
  decision point (lane_planner.rs:892). No chicken-and-egg:
  `FeederSpecsSkipped` fires at routing time and remains the
  it-still-happened alarm, not the decision input. Strictly
  additive; broken cells heal; kill criteria 2–3 evaluated.
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
- *2026-07-13 — adversarial review round 1: REVISE, two blockers +
  three should-fixes, all incorporated. (1) BLOCKER — D4's "walkers
  already model priority splitters" was half-false: the priority
  FIELDS exist and export, but no validator reads them; the only
  priority-aware walker path is gated on `:selfloop:` tags, so a
  backwards-stamped tap would validate silently. Now a named Phase 1
  prerequisite (generalized tap-segment priority detection in both
  walkers + a structural direction check). (2) BLOCKER — D1's
  "multi-tap workhorse" holds only for external-input lanes;
  intermediate lanes force one-trunk-per-consumer today, and the
  ghost router's general multi-tap rendering has never been combined
  with family-tagged lanes — that combination is Phase 1 integration
  territory with its own tests, and lane_planner's dead round-robin
  code becomes live. (3) K's capacity constant PINNED to
  full_belt_cap (2× lane_capacity) matching the balancer-present
  precedent — the conservative constant would double K and defeat
  KC4. (4) The (n,1) merge-tree builder acknowledged as NEW code
  (library atoms are fixed lookups to n≈10; the only runtime
  primitive is a parallel (2,1)). (5) Review gift: the Phase-1
  chicken-and-egg dissolves — `shape_is_stampable` already exists,
  mirrors the stamper's full decision tree, and is live in
  production; named at its exact call site. (6) Prior-art citation
  split: PR #272's CP-SAT placement/routing (3/8, Clos OOM) vs
  #136's Factorio-SAT bake wall (0/32 overnight) — different tools,
  different failure modes. Pending re-review verification, then
  Phase 0 (census results from the spike).*
