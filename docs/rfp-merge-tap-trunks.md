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
- *2026-07-13 — re-review verification: **SHIP.** All five findings
  confirmed faithfully incorporated (D4's validation-gap rewrite
  closes blocker 1 with the backwards-tap mode named; D1's multi-tap
  reality accurate including the round-robin degeneracy needing
  replacement-not-reactivation; K constant and precedent citation
  correct; builder-as-new-code and shape_is_stampable transcription
  accurate; prior-art attribution corrected). **RFP ready for
  acceptance once Phase 0's shape census lands** (spike in flight:
  (12,7) already baked as an interim data point; census's final
  cells computing).*
- *2026-07-13 — **Phase 0 census landed (spike; artifacts committed
  8340f8b). Kill criterion 1 evaluated: the RFP does NOT close — the
  census is its strongest evidence.** Blast radius: utility@10/s is
  the ONLY missing-shape cell in the 24-cell corpus, demanding
  exactly (15,14)/(12,7)/(8,19). But "all shapes cheaply bakeable"
  FAILS: **(8,19) is analytically unreachable by composition** (19
  is prime past the largest fan-out atom — no Clos factorization
  exists), and (15,14) was non-convergent after 9 minutes of
  junction search toward a would-be 30×60-90 monster. Only (12,7)
  was trivial (baked, interim mitigation for 12 of the 35
  dead-ends). **Mechanism confirmed with numbers**: M is a
  consumer-count artifact, not throughput — raw capacity splits
  would be 26/8/15 but get clamped/padded to consumer-row counts
  (iron-plate padded 15→19 to match 19 consumer recipes; its K at
  blue belt is 8, so M = 2.4×K). Prime consumer counts recur
  indefinitely as recipe trees deepen: composition patching is
  categorically unable to close this class, and merge-and-tap's
  Phase 1 fallback is NECESSARY for (8,19)-class shapes, not merely
  preferable. K table (blue belt): electronic-circuit K=4 (vs M=7),
  copper-cable K=13 (vs M=14), iron-plate K=8 (vs M=19). Side
  findings recorded: #136 is more open than its closed state
  suggests (authored (m,9) recipes never baked; an `#[ignore]`d
  coverage test documents the gap — folded into this RFP's Phase 3
  scope decision); production@10/s is independently slow (>20min
  uncontended, the known TIMEOUT cliff — separate wall, not
  balancer-related). Utility@10/s post-(12,7)-bake before/after
  confirmation pending as a follow-up. **RFP ready for user
  acceptance.***
- *2026-07-13 — **accepted by user.** Phase 1 started: validator
  prerequisite (tap-priority generalization) first as its own
  commit, then the merge-tree builder + K-trunk fallback +
  priority taps, activated only where `shape_is_stampable` says no.*
- *2026-07-13 — **Checkpoint B: machinery landed DORMANT (499287b);
  STOP on kill criterion 6.** The merge-tree builder (prime-tested),
  bin-packing, K-trunk fallback, and rendering seams are built and
  inert behind a flag. Two findings from the stop: (1) **KC6 fired
  as a HANG, not slowness** — enabling the fallback deterministically
  hangs the Step-6 junction solver (EC@35/s from ore; copper-plate
  (4,9) unstampable, K=4; steps 3-5 complete, step 6 never returns).
  Mechanism: K trunks × per-consumer taps each fan east to the bus
  edge, crossing every lane to their right — novel crossing density
  the junction machinery has never absorbed, and something in it
  loops unbounded rather than hitting its growth caps. The
  investigation is its own next checkpoint; leads: constrain
  merge-tap tap routing (consumer-column goals / UG bridging over
  the lane block) to slash crossings, and make Step 6 degrade to a
  bounded warning instead of hanging regardless (a hang is a bug
  independent of merge-tap). (2) **KC2's premise corrected**: the
  Phase 0 census was gauntlet-scoped, but the default e2e STRESS
  corpus contains unstampable (4,9) cells (EC@35s, PU@3/s) — the
  flip cannot be zero-movement as written. KC2 is re-scoped: zero
  movement on all STAMPABLE-shape fixtures; unstampable-shape cells
  healing is EXPECTED movement, re-blessed against predictions
  (the accepted-residue restatement precedent). (3) Priority-tap
  stamping (seam 5, lighting up Checkpoint A) deliberately deferred
  to after the hang investigation — no point tagging taps the
  router can't route. Also corrected during B (research fork): the
  RFP's ghost_router.rs:2182-2264 reference is the flow-unify pass,
  not tap stamping (Step 2, ~195-240 stamps taps; they are plain
  50/50 today); the lane_planner "non-starter" comments are fossils
  of the deleted direct-mode router; the round-robin at :828 is
  live for external lanes (sole multi-tap producer) and was
  preserved, with the fallback path added alongside; family lane_xs
  contiguity (hard Err at lane_planner.rs:503-519) is satisfied by
  the K-trunk assignment.*
- *2026-07-14 — **Checkpoint B2 complete (6cb0368 / 0e27a14 /
  bf061e3): the Step-6 hang is dead, and the measurement REFRAMES
  kill criterion 6.** Root cause was an infeasibility trap, not
  hardness: the pathological zone's boundary absorbed one channel's
  outputs into another's trunk column, leaving a 3-source/0-sink
  channel — UNSAT by flow conservation, unboundedly slow to REFUTE
  (varisat 0.2.2 has no interrupt; WASM has no threads, so no
  watchdog). Fixes: flow-balance pre-check (structural, any size,
  never a false refusal; 28s→132ms on the isolated cluster) +
  var-count ceiling at 700 (terminate-guarantee net, calibrated in
  the 630/756 gap) + mergetree segments protected from zone growth
  (the unmasked AlreadyClaimed panic — the #296 claims family).
  Permanent regression guard: a region fixture replaying the exact
  hang zone. Corrected expectation: these crossings DEGRADE (the
  feeder sinks are never captured at any region size), they do not
  heal. **The flag-on measurement (EC@35s: 132 errors = 125
  lane-throughput + 7 unresolved-junction)**: crossing pressure is
  ABSORBABLE — 7 junctions, not catastrophe — and the dominant
  failure is flow MISDISTRIBUTION on shared trunks, exactly the
  signature of the deferred priority taps (plain 50/50 splitters
  metering nothing). Constrained-tap routing is demoted to a
  7-crossing follow-up; **the next step is task #12 (priority taps,
  now unblocked), then re-measure** — the lane-throughput count
  collapsing (or not) is the falsification test for the
  priority-tap hypothesis.*
- *2026-07-14 — **B2.4 diagnosis: RECORD CORRECTION + reframe.** The
  B2.3 entry's "125 lane-throughput / 7 unresolved" and the later
  "215/12" were both measurements of an already-broken FLAG-ON
  state; the TRUE baseline is flag-OFF EC@35s = 4 errors (the
  pinned golden: 4 belt-dead-ends from the missing (4,9) balancer).
  Merge-tap as currently wired is a 4→227 REGRESSION on that cell
  (and EC@40: 13→panic). The machinery itself is CORRECT — K=4
  arithmetic verified, clamp didn't bite, the "4 mergetree entities"
  anomaly is benign (4 single-producer trunks → merge_tree(1) =
  passthrough), priority taps verified firing — the damage is
  **foreign-lane feeder concentration**: the fallback's copper
  feeders route onto/into the iron-ore trunk column at x=20,
  concentrating a non-merge-tap lane to a modeled ~50/s per belt
  (86+ of the 215 errors on iron-ore + adjacent crossings). Same
  geometry that produced the B2.1 UNSAT zone. Priority taps (#12)
  were the wrong lever twice over: right mechanism, error-free
  corner. Ruling: fix the feeder ROUTING (the fallback is worthless
  anywhere if feeders trample foreign trunks); the trigger-scoping
  question (should (4,9)-class cells with 4 cheap dead-ends take
  the fallback at all?) is deferred until routing is clean and
  re-measured. Named follow-ups: EC@40's cluster-COMMIT
  AlreadyClaimed (a GAP in B2.3a's mergetree protection — commit
  loop skips Template/RowEntity but not Permanent; beyond the
  one-liner), and the MergeTapFallback double-emit (plan_bus_lanes
  apparently runs twice; cosmetic).*
- *2026-07-14 — **the decisive measurement: utility@10/s (the
  motivating cell) HEALS under the bridged fallback.** Flag-on vs
  true flag-off baseline: total errors 175→80, belt-dead-ends 23→1
  (KC3's target; baseline drift from the recorded 35 is
  version-skew, same cascade shape), lane-throughput 115→25,
  reachability warnings 755→226. The consumer-side mechanism does
  NOT bite utility (zero row belt-in over-cap) — and EC@35's
  72/belt row reading is LARGELY AN ATTRIBUTION ARTIFACT:
  effective_rows clones the aggregate spec count (35) onto every
  physical row, so per-row demand computed from it overstates ~9×;
  the priority taps carry the correct ~6/s per-row shares. Honest
  regressions on utility: item-isolation 6→19 and junctions 27→31,
  both the residual adjacent-terminating foreign-sideload class the
  pass-through bridge doesn't cover — bounded, named Phase 2 items
  alongside the attribution artifact. Verdict: the fallback wins on
  utility-class cells and loses only on EC-class cells whose
  flag-off residue is 4 cheap dead-ends → **Phase 1 closes via
  (b)-gating**, and the architecturally native gate is the
  decomposition-search candidate pattern (run merge-tap as a
  DecompositionCandidate for solves with unstampable shapes, score
  against the native path, pick the winner — measured selection,
  no heuristic). Feasibility check on scoring semantics dispatched;
  cumulative Phase-1 LOC (~767) crosses the ~800 trigger with the
  gate — reviewed and justified: the bake-shapes interim cannot
  cover (8,19)-class primes, and every increment has been
  checkpoint-verified.*
- *2026-07-14 — **PHASE 1 LANDED (4128440): the fallback ships as
  MergeTapCandidate, measured selection, flag retired.** Constructed
  only when native reports missing-balancer warnings; winner by
  total validation-error count at the comparison site (score_layout
  untouched — deviation ratified: validate() confined to the rare
  Pooled+unstampable case keeps goldens and blessed trace streams
  strictly inert); tie → native. KC2: zero movement, double-proven.
  KC3: utility@10/s native 175/23/1479 → merge-tap 62/1/848
  (errors −65%, dead-ends −96%), fallback firing at K=13/(15,14)
  and K=8/(8,19) exactly as the Phase 0 census predicted; healed
  region snapshot-verified; EC@35 correctly keeps native (4<66).
  Deterministic mechanism fixture non-ignored in e2e (23s); a 0/0
  gate is documented unreachable pending Phase 2 crossing quality.
  Phase 1 cumulative: the fallback machinery + priority-tap
  validation layer + THREE permanent robustness fixes shaken out
  along the way (flow-balance pre-check, SAT var ceiling, mergetree
  zone protection — the junction solver can no longer hang on any
  input). **Phase 2 worklist (named, bounded)**: merge-tap router
  crossing quality (the 62-error/miss-bal-19 residual — K-trunks
  spawn unstampable sub-shapes), consumer-side effective_rows
  aggregate-count attribution artifact, item-isolation blind spots
  on perpendicular sideloads, adjacent-terminating feeder residual,
  EC@40's 69-error state, candidate second-pass cost if
  utility-scale cells ever enter the default suite, MergeTapFallback
  double-emit. Phase 2 (the default flip for ALL split families +
  balancer retirement + the area harvest) remains gated on that
  quality work per the original phasing.*
- *2026-07-14 — **Phase 2 checkpoints 1–2a complete; the
  proportionality gate fired and the flip is now PRICED.** CP1: the
  19 missing-balancer warnings were phantoms from a second emitter
  that never learned merge-tap families exist — 3-line exemption
  (e4e1207), purity-proven (62 errors byte-unchanged), and a flip
  prerequisite (they pinned merge-tap layouts accepted=false). CP2a:
  the 11 copper→iron item-mixing errors were a TRAPPED-GAP bridge
  bug — two UG hops colliding on a kept tile inside the foreign
  block, render_path silently dropping the second, leaving a UG
  output that sideloaded copper onto iron AND severing the feeder.
  Fixed (b8b955a): coalesced single-span bridging + a
  self-validating guard (FeederBridgeUnbridgeable → visible
  dead-end; silent half-bridges structurally impossible). Honest
  accounting: utility@10/s 62→64 errors — the contamination was
  MASKING genuinely-unreachable trunks. **The falsified cheap
  lever**: sorting merge-tap trunks eastmost (producer-adjacent) as
  a post-hoc placement rule measured 62→1024 — the existing lane
  order jointly satisfies fluid anchor spacing, crossing
  minimisation, and throughput distribution, and a naive sort
  violates all three; reverted after one attempt per the
  proportionality rule. **UNIFIED ROOT CAUSE, CONFIRMED**: the
  ordering optimiser is blind to merge-tap feeders — one cause
  drives the 32 unresolved junctions AND the feeder failures. The
  proper fix is merge-tap-aware lane ordering INSIDE
  score_lane_ordering (shared scorer, multi-objective, corpus
  re-bless risk) — priced at days, and it is the gate on BOTH
  further utility@10/s error reduction AND Phase 2's flip with its
  area-harvest promise. **State banked**: Phase 1's measured
  selection stands (utility 175→64 honest, EC native), the
  robustness/honesty floor is in, and the funding decision on
  merge-tap-aware ordering goes to the user. CP2b/2c (EC@40
  characterization, double-emit) and the native EC↔plastic-bar
  mixing finding remain open, unstarted.*
- *2026-07-14 — CP2b/2c closed (884d212). The double-emit was the
  two-pass lane planner, not the retry orchestrator (fixture comment
  corrected); one event per fallback family now. EC@40 is OFF the
  merge-tap worklist: it selects native at 13 errors (all
  belt-dead-end, native output-merger/trunk-tail residual — the
  recorded 69-error state predated the junction/netflow
  improvements). **Phase 2 HOLDS here pending the user's funding
  decision on merge-tap-aware lane ordering** — the single priced
  lever for both the utility@10/s residual (honest floor: 64) and
  the flip/area-harvest. Remaining named items: that ordering work,
  the native EC↔plastic-bar mixing finding, the effective_rows
  attribution artifact, item-isolation's perpendicular-sideload
  inconsistency, the candidate second-pass cost note.*
- *2026-07-14 — **the bounded ladder is exhausted; final stock-take.**
  Arc since the goal push began (baseline: native 175 errors):
  phantom warnings fixed (e4e1207); feeder trapped-gap bridge +
  honesty guard (b8b955a); double-emit dedup + EC@40 off the
  worklist (884d212); tap bridge (cc0fd8a, junctions 42→35
  components); wide-wall honest refusal FALSIFIED (fragments
  clusters, 108→155, reverted); placement-sort FALSIFIED earlier
  (62→1024). Three successive attribution hypotheses resolved by
  measurement: effective_rows aggregate-count — NOT the cause (rate
  injection is per-row-correct); (B)-broke-the-predicate — REFUTED
  (the predicate is never reached); actual mechanism: **all 21
  iron-plate tapoff splitters die at the topological walker's
  sibling-wait GIVEUP branch (MAX_RETRIES=3 → rates (0,0), no
  depletion)** — PRE-EXISTING (18 giveups before the tap bridge; the
  bridge unmasked 3 more by pulling tiles out of junction-exclusion,
  confirming cc0fd8a's honest-unmask framing). **Final decomposition
  of utility@10/s's 108 errors**: (a) ~35 junction components = real
  geometry debt — the 17-21-lane copper-plate wall between
  copper-cable's merge-tree and its producers + general EC/AC
  feeder crossings (native debt) — fixable only by the STEP-3
  merge-tap-aware lane-ordering package (days, shared scorer,
  corpus re-bless); (b) ~61 lane-throughput = UNRELIABLE-WALKER
  NOISE from the sibling-ordering giveup bug — the taps are
  physically per-row-correct (~2.5/s verified against solver
  truth); the layout is likely fine there, the checker is not;
  (c) 8 native EC↔plastic-bar isolation + remainder. Honest
  FUNCTIONAL error mass ≈ 43-47. **Open deep items, all priced,
  none bounded**: the STEP-3 placement package; the walker
  sibling-giveup defect (validator-internals, affects
  lane-throughput trust on trunk+tapoff topologies generally); the
  general feeder bridge (golden-risky, folds into STEP-3). The goal
  push ends here pending direction.*
- *2026-07-14 — **walker package #1 (sibling parking) LANDED
  (6e5be07); a second walker defect uncovered and scoped as #1b.**
  The active /goal funded the walker fixes as the first bounded
  lever. Diagnosis confirmed the giveup mechanism precisely: the 21
  iron-trunk tapoff splitters form a pure dependency CHAIN (each
  tapoff waits on the previous splitter's continuation; DAG, no
  cycles), so depth 21 ≫ MAX_RETRIES=3 — a fixed retry budget can
  never walk a chain. Fix ("parking", Option T): the waiting tile
  parks as input-ready and the sibling fires both through
  `splitter_output_rates` when popped; the only remaining give-up
  is an outer-loop force-resolve (deterministic lowest-coordinate)
  reached only for true cycles. O(E). Gates: all 21 splitters fire
  (0 giveups / 0 force-resolves), full suite green and BYTE-STABLE
  (zero pinned-fixture movement — the STOP zone "movement =
  giveup-masked mis-model elsewhere" did not trigger), clippy
  clean. **But the 61 lane-throughput errors did not move**,
  because firing exposed defect #2: both walkers identify the
  priority branch by sniffing the segment tag one tile downstream
  (`segment_is_priority_branch`), and for most merge-tap taps both
  downstream tiles read `trunk:iron-plate` / `crossing:*` —
  untagged — so the law falls back 50/50 and the trunk never
  depletes (last tap inherits ~29.9/s). The splitter ENTITIES carry
  correct `output_priority` + `loop_priority_rate` (exported
  blueprints are real priority splitters) — validator mis-model,
  not a layout defect. **#1b scoped**: derive the priority branch
  from the entity's own fields via `priority_output_tile` (the
  structural check's existing geometric mapping — single source),
  tag-sniff demoted to instrumented fallback; both walkers
  (belt_flow's `is_priority_branch` verified to share the same
  defect and the same `splitter_entity` map). This is the design
  the delta review originally demanded ("no validator reads the
  real priority fields") — the tag mechanism is now measured
  fragile in the wild. Expected: 61 → near-zero; honest floor
  → ~43-47.*
- *2026-07-14 — **#1b LANDED (618ee0a): 61 → 51, and the residual
  is a THIRD distinct mechanism, not detection.** Both walkers now
  read the entity's `output_priority` via the shared
  `priority_output_tile` (made pub(crate)); tag-sniff demoted to a
  debug-asserted fallback. Four pre-existing priority-share tests
  modelled tag-only splitters and tripped the assert — the fallback
  firing on synthetic fixtures, exactly as predicted — fixed to
  model real splitters; a new no-tag guard test proves the law
  fires from `output_priority` alone. Gates: 743/0, byte-stable,
  clippy clean. **Instrumented finding (one trunk, reverted): the
  x=90 trunk now DEPLETES correctly (34.45/s → 6.0/s), so #1b
  works; and injection is a healthy 333/s at 0.62/inserter, so the
  effective_rows aggregate hypothesis is DEAD.** The remaining 51
  (all ~29.9/s) are a downstream re-convergence: flow re-gains
  6 → 14.93 → 44.86/s at a crossing (seg=crossing:85:472), split
  15.0 left / 29.86 RIGHT — one lane over the 22.5 express cap on
  near-saturated trunks (each iron trunk ~41/45). The planned
  21-chain terminus≈2.5 test was correctly NOT written — its
  prediction was wrong. **#1c funded, diagnosis-only**: classify
  the 51 — (A) walker conflating surface+UG flows at shared
  crossing tiles (UGs cannot mix underground → pure attribution
  artifact) vs (B) genuine sideload merge concentrating one lane
  (real layout lane-balancing gap → folds into STEP-3) vs (C)
  other. Entity-level snapshot evidence at ≥3 sites, count per
  class, priced recommendations; no fixes in-package.*
