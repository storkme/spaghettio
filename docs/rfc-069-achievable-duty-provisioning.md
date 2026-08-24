# RFC-069: Achievable-duty provisioning for the zero-headroom family

**Status**: Active (resumed 2026-08-24 as the trunk/tap-provisioning
campaign — Phases R/A/B/C below; the 2026-08-15 phase results stand)
**Tracking**: #644 (the deficit family this closes)
**Registry**: [`rfcs.md`](rfcs.md)

## Summary

The from-ore high-rate corpus ships layouts whose external-input chains
— trunk belts, tap routes, and row input belts — are planned at
**exactly 100% of nominal belt capacity** (e.g. stress-ec30: 45/s of
copper ore over 3 yellow belts of 15.0/s, into 3 rows of 24 furnaces
drawing exactly 15.0/s each). Real belt physics (gap propagation,
inserter swing, the #448 tail-starvation mechanism) delivers ~85–92% of
nominal on a chain with zero headroom, so these layouts under-deliver
by measurement while being structurally sound. This RFC threads one
**planning-duty factor** through the external-input provisioning
arithmetic — trunk counts, tap obligations, and row sizing, end-to-end
— so chains are planned at ≤ duty × nominal, and adds a typed plan-time
refusal for rates genuinely unreachable at the user's belt tier. Gates
are sim-anchored: the exit criterion for #644 is measured delivery, not
validator silence (the validator carries no Error-level signal for this
family by design — flow conservation is satisfied at 100% load).

## Motivation

Sim/meter receipts, all on current engine output:

- `stress_electronic_circuit_30s_from_ore`: sim **92.1% delivered**
  (post-lift bank 2026-08-07/08, warmup 432k, kit-clean, converged);
  meter 91.9%. Every copper/iron trunk, tap, and row-in lane reads
  exactly [7.5, 7.5] on yellow (verified tile-level, #648).
- `stress_electronic_circuit_60s_red_from_ore`: sim **90.7% delivered**
  (same bank). Same construction at 2× rate on fast belts.
- `tier5_processing_unit_from_ore_am3`: sim **95.6% delivered**
  (Phase 0, axis-declared, kit-clean — see the decision log). The
  meter's earlier 85.6% receipt was measured on an UNDECLARED-axis
  export and is retracted as this fixture's headline number; what
  survives of it is the uniform choke signature (one shared constraint
  propagating, status.md's y=mx+c reading), which the kit-clean sim
  reproduces at ~−5.3% per mid-chain stage.
- The zero-headroom scoping (status.md, 2026-08-07, solver-derived):
  **69/146 stages (47%) across 28/40 fixtures are exactly
  zero-headroom**; 63% sit under 2% headroom. Cost model already
  measured: a flat "+1 machine when headroom < X%" rule costs ~107
  machines corpus-wide vs ~357 for a multiplicative margin.

History this RFC must not ignore:

- **The 2026-07-31 #519 margin-probe rejection is contaminated.**
  `ROW_INPUT_PLANNING_MARGIN = 0.95` was parked as "measured NOT a
  win", but the measurement was walker-modeled warning counts under the
  pre-#648 walker — whose phantom-UG-source bug *fabricated* warnings
  in proportion to tap/crossing count, i.e. the probe's penalty signal
  was partly the instrument punishing the extra taps the margin
  created. The rejection does not bind this RFC; the sim does.
- **The geometric half of that finding survives**: per-row margin
  WITHOUT trunk-count increase moves the 100%-load point upstream
  (same trunk flux, more taps). Duty must apply end-to-end — trunk
  count and row sizing from the same factor — or it does nothing.
- **Zero-headroom is necessary, not sufficient** (the
  pooled-vs-partitioned 80%-vs-98% confound on identical plans):
  loss-reduction is a real alternative lever. This RFC picks headroom
  because the family's chains are at-cap *everywhere* (there is no
  slack to re-route into), and pins the confound in K69-3.

## Design

One constant, threaded through the three provisioning sites that
currently assume nominal capacity is fully achievable:

1. **`PLANNING_DUTY: f64`** in `common.rs` (initial value 0.9;
   Phase 1 calibrates against the sim — tier2's measured row duty was
   90%, and the family's sim-anchored deficits span 90.7–95.6% —
   ec60-red, ec30, tier5-PU — against ec22's clean 99.4%). Documented as
   "the fraction of nominal belt throughput the planner treats as
   deliverable on a zero-buffer chain".
2. **Row sizing** (`placer.rs`: the input-limit terms of
   `max_machines_for_belt`, `max_machines_for_belt_both_lanes`, and the
   third row-cap variant): `in_lane_cap × 2.0` becomes
   `in_lane_cap × 2.0 × PLANNING_DUTY`. This is the root site: the
   current `floor(in_lane_cap / inp.rate) × 2.0` admits exact
   equality, so whenever the division lands integral (copper-plate:
   floor(7.5/0.625)×2 = 24 machines × 0.625 = 15.00/s = exactly one
   full yellow belt) the row is zero-headroom **by construction**.
3. **External trunk provisioning** (`lane_planner.rs`
   `split_overflowing_lanes`, external-input path) — likely a NO-OP
   after site 2, kept for verification: the capacity-derived
   `n_splits` already over-provisions (ceil(45/7.5) = 6 trunks for
   ec30 copper), but the round-robin consumer distribution gives each
   row one split and the empty splits are skipped, collapsing trunks
   to one-per-row at the row's full demand. Since trunks follow rows
   1:1 on this path, shrinking rows (site 2) shrinks per-trunk load
   with no lane-planner change. Phase 1 verifies per-trunk load
   ≤ duty × nominal on the built layout rather than assuming it; if
   the collapse ever binds elsewhere (rows ≥ splits), scale the cap
   here too.
4. **Typed refusal** (small, separate phase): when the user's
   `max_belt_tier` makes the target rate unreachable even at duty
   provisioning (single-row max feeds exhausted), fail the layout with
   a typed error naming the rate, the tier, and the achievable
   ceiling — instead of silently shipping a deficient layout. Belt
   tier is a hard user constraint (never auto-escalate).

Machine-count headroom (the flat "+1 machine when headroom < X%" solver
rule) is deliberately **out of scope** here: it changes solver output
corpus-wide (47% of stages) and is severable — belt-side duty is where
the #644 family's receipts point (their machine counts are integral
multiples that fit; their *belts* are the shared at-cap constraint).
If Phase 1's sim gate shows belt-duty alone insufficient, the solver
rule is the follow-up RFC, not a scope creep here.

Expected shape of the diff, stated precisely (review round 1 caught
the first draft conflating the two numbers): at duty 0.9 the CAP is
`floor(7.5×0.9/0.625)×2 = 20` machines/row = 12.5/s = 83% of a belt;
the REALIZED per-row load is then an even-split artifact below the cap
— 72 furnaces over `ceil(72/20) = 4` rows = 18 each = 11.25/s = 75%.
The cap guarantees ≤83%; the split delivers 75% here. Measured on the
first Phase-1 artifact (ec30 at duty 0.9): 81×168 / 3,797 entities vs
the baseline's 96×140 / 3,369 — **+12.7% entities, +1.3% bbox area** —
validating 0 errors / 10 warnings. Footprint cost is reported per
fixture in the decision log; if it approaches K69-4's bar, the trade
goes to the owner before merge.

## Kill criteria

- **K69-1 (mechanism)**: Phase 1's duty-provisioned `ec30` layout
  sims below **96.0% delivered** (the single number; the baseline is
  92.1%, so the bar demands ≥ +3.9pp) at every duty in {0.85, 0.9} →
  the dominant loss is not headroom on these chains; stop, return to
  the loss-reduction lever (the pooled-vs-partitioned confound), and
  take the probe evidence to #644.
- **K69-2 (no regression)**: any previously at-plan fixture
  (`big-electric-pole@1` canary, `ac@5`, `ec@10`) sims below plan on
  the duty-provisioned engine → stop and bisect before touching
  anything else.
- **K69-3 (confound honesty)**: if the Pool-vs-P2 arms of the same
  fixture diverge by > 5pp delivered under identical duty settings,
  the strategy confound dominates the headroom mechanism → pause the
  rollout and measure the confound first (it invalidates duty
  calibration).
- **K69-4 (cost)**: footprint cost on the gate fixtures exceeds +25%
  bbox area for < +5pp delivered → surface the trade-off to the owner
  before merging anything; do not adjudicate it autonomously.

## Verification plan

- **Phase 0 (baselines; no engine change)**: sim `ec22` (the boundary
  sibling whose clean validator state was never measurement-backed) and
  `tier5-pu2-am3` (meter-only receipt) at warmup 432k. Outcomes decide
  Phase 1's gate set: if ec22 itself under-delivers, the family is
  bigger than the flagged fixtures and the gate corpus widens.
- **Phase 1 (duty knob, sim-gated)**: implement sites 1–3; sim
  `ec30` at duty 0.9 (and 0.85 if 0.9 misses); gates = K69-1/2;
  validator suite green throughout (warning goldens re-blessed with
  adjudication where row counts change layouts).
- **Phase 2 (family rollout)**: `ec60-red`, `tier5-pu`, decomposed +
  partition arms; K69-3 watched on the arms; per-fixture delivered%
  and footprint recorded in the decision log.
- **Phase 3 (typed refusal)**: unit tests for the refusal path
  (unreachable rate at tier cap); no sim needed.
- Instruments: sim = clearance; meter = cheap cross-check
  (below-plan ⇒ believe); walker = regression guard only
  (lane-throughput must stay 0 — pins from #648).

## Resumption plan (2026-08-24): the trunk/tap-provisioning campaign

The RFC-071 evidence table left ec35/ec40 as the corpus's worst measured
rows (22.9% / 18.5% delivered, sim-anchored) — shipped as merge-tap
fallbacks carrying 313/631 lane-throughput errors. The resumption
diagnosis (decision log, 2026-08-24) found their root cause is NOT
headroom arithmetic but **rescue reachability**: the Pooled native hits
the copper-plate **(4,9) coprime balancer trap** (no template, gcd
indivisible — the producer rows genuinely dead-end), and the
`k1-shape-fix` candidate built for exactly that trap is unreachable on
the shipped path behind three stacked, individually era-correct
blockers (its `PartitionedDecomposed` strategy gate; the early-decide
measurement skip; the MergeTap stage terminating the program before
`BestErrorFree` can rank a measured rescue).

Phases (each lands as its own PR set; sim anchors gate anything that
changes shipped geometry):

- **Phase R (re-verification + diagnosis — measurement only, complete)**:
  duty-0.6 receipts reproduce under v2 selection (ec30 4,934 entities /
  meter 96.4%; ec60-red 6,361 / 99.4%); ec35/ec40 root-caused as above;
  the candidate arms measured (table in the decision log).
- **Phase A (make the rescue reachable / fix the family)**: two tracks,
  both carried until measurement picks the shipping shape. **A1
  (selection reachability, the class fix)**: un-gate `k1-shape-fix` on
  Pooled, measure its counts, and amend the stage program so a measured
  error-free candidate outranks the merge-tap pairwise short-circuit —
  policy-table edits with receipts, K70-1-style winner-flip adjudication
  on the full bank (the RFC-071 fingerprint gate makes every flip loud).
  **A2 (family depth)**: close the (4,9) library hole (balancer-gen /
  Clos composition per `balancer-theory.md`) so the native itself
  stamps — the native composes with duty (measured on ec30) where the
  k1 artifact is currently duty-blind (measured, decision log), so A2
  may reach the at-plan end state more directly; A1 remains the class
  fix for coprime shapes the library will never enumerate.
- **Phase B (the default flip — the 2026-08-15 owner call's end state)**:
  duty ships ON by default in the threshold form Phase 2's measurements
  pick (fraction vs block bound); density-term re-weigh so selection
  stops punishing the correct shape; corpus pins/goldens/scoreboards +
  calibration bank + evidence table re-recorded against sim receipts.
  Precondition: Phase A landed — the tier5 re-diagnosis (decision log,
  2026-08-24) dissolved the other one: its "router block" is this same
  coprime-trap class, so Phase A covers it and no junction-solver
  campaign gates the flip.
- **Phase C (typed refusal — the original Phase 3)**: unchanged.

## Decision log

- *2026-08-24 — Phase A2 (the tier5 twin blockers) IMPLEMENTED; the
  campaign's remaining engineering unifies into ONE router item.* Two
  mechanisms, both pinned: (1) **multi-consumer enrollment** — the k1
  refusal was `k1_consumer_for_item`'s single-consumer requirement (the
  Pooled base plan has ZERO modules — the "K≥2 items have modules"
  reading was wrong, measured via plan dump); the new arm enrolls a
  warned multi-consumer item as one module per consumer recipe (the
  same per-item construction `plan_partitioning` performs under PD) and
  runs the partitioner's own `apply_shape_fixes` over just the new
  modules (single source for pad/shard; the K=1 arm untouched, keeping
  ec35's shipped artifact byte-stable). (2) **the held-incumbent
  migration** (#720 round-4 critical): at the ranked boundary an
  ACCEPTED held incumbent stands (#474 unchanged); an UNACCEPTED one
  migrates into the held-answer slot displaceable ONLY by the
  error-free tier — NOT by `BestAccepted`'s score, which would let the
  27-route-severed merge-tap unseat the 18-severed native that just
  beat it on kinds (the acceptance gate must not override the
  better-calibrated quality-key verdict). Measured on tier5@0.6: k1
  goes Refused("no k1 enrollment") → **Produced, accepted, 12 errors**
  — and the decision correctly still ships native@MergeTap (12E is not
  error-free; nothing weaker may displace). **The fingerprint probe
  then falsified this PR's first "inert on every shipped decision"
  claim, in the right direction: ac45 flips** — its broken native (14
  route-severed, unaccepted, bank sim 0.0/s non-converged) had been
  shadowing a produced **error-free cell-composed layout** (0E/30W,
  7,674 entities, 1470×22 — the RFC-067/068 machinery, also trapped
  behind the held-incumbent short-circuit); with the migration,
  cell-composed wins at BestErrorFree, and the refresh protocol ran
  for ac45's row: meter models 97.8% (44.0/45), but the sim says the
  meter is wrong on this shape (the post-lift divergence class) — 432k:
  non-converged 63.7% kit-clean; 864k A/B arm: non-converged 66.7%, so
  doubling warmup bought +3pp, not the +34pp a ramp would show. The
  cell-chain layout has a REAL throughput ceiling ≈ 2/3 of plan. The
  flip still ships on unambiguous grounds (0-error, 63.7–66.7% vs the
  old winner's sim-measured 0.0/s dead), the bank row records the
  protocol run as non-converged evidence, and the residual joins the
  cell-drain ledger (RFC-071's >45/s single-drain + K>1 follow-up —
  a 45/s AC chain at 1470 tiles is squarely that territory). The
  remaining wall for ec40 (k1 10E), tier5 (k1 12E), and therefore the
  Phase-B flip is one shared class: **the tap-bridge/crossing-zone
  router failures on enrolled multi-lane plans** (ec40 receipts: `TapBridgeUnbridgeable`
  copper-cable spans 6/5/4/3 vs yellow UG reach 4, then
  `CrossingZoneSkipped` ×4 "flow_imbalance: ch1 1in/0out" at (15,113);
  10 cable producer rows y=45..108 dead-end at x=48 with their
  balancer at y≈110). Next unit: that router class.*
- *2026-08-24 — #720 round 4 adjudicated at the stop point; one forward
  blocker recorded, three small absorbs, the rest recycled or refuted
  with receipts.* The round's [critical] is REAL but NOT LIVE: on a
  field where the broken native WINS the quality-key pairwise (tier5's
  shape — native rs=18 vs merge-tap rs=27), the held-INCUMBENT
  short-circuit returns native before `BestErrorFree` runs, so a
  produced 0-error rescue would be unreachable there. No shipped field
  has that combination (ec35/ec40: merge-tap wins the pairwise; tier5:
  k1 refuses at enrollment), so nothing behaves wrongly today — but it
  means **tier5 has TWO blockers, not one**: the K≥2 enrollment
  extension AND opening the held-incumbent path to ranked displacement.
  Both belong to the tier5 PR, where the semantics change has a live
  field to be measured against (changing the v1-faithful held-incumbent
  behavior blind, in round 5 of this PR, would be exploration past the
  evidence). Absorbed now: the validator-trust lane-throughput row's
  ec35 half marked superseded (its own same-PR rule); the firing-census
  message that still named the removed PD gate; and the Forced-DI
  stand-down on k1 (registration + call site + gate test, the
  three-lists rule — moot while PD-only, newly reachable on Pooled).
  Refuted with receipts: "ec40's pin breaks if k1 is accepted" — k1 IS
  accepted on ec40 today (probe: acc=true, err=10) and the pin holds by
  ScoreDesc through four green batteries; the "bit-identical by
  accident" major recycles rounds 1–2's adjudicated BestAccepted
  semantics; the fingerprint-flake minor mistakes the recorded
  CROSS-PATH byte variance for within-path nondeterminism — the probe
  rebuilds via one path and has reproduced ec35's hash on every run,
  local and CI; the evidence-columns minor recycles round 3.*
- *2026-08-24 — #720 round 3 absorbed: the measurement trigger scoped to
  third parties.* The recycled cost minor (3/3 this round) earned its
  absorption at the third telling: the trigger now fires only for a
  produced-but-unmeasured candidate OUTSIDE the early decision's own
  pair — an unaccepted incumbent cannot enter the accepted tiers and the
  rival's win is absorbed back to its pairwise tag with or without
  counts, so measuring just those two buys nothing, and the pre-A1
  laziness stands on rescue-less broken fields (tier5/ac45-class, the
  corpus's largest layouts). status.md brought current in the same
  round (a fair process hit — the ledger duty). The round's major was
  refuted on the enum: `LayoutStrategy` has exactly two variants, so
  "all strategies" is {Pooled, PD}, both pinned (the gate test asserts
  both eligibilities; ec35's e2e is the live Pooled behavior pin). The
  combined-invariant minor was refuted by pointing at the fourth policy
  test, whose field exercises the suspension and the scoped-admission
  exclusion jointly; the shrinking-columns minor is the joiner's
  occurring-categories design — the probe JSON underneath remains
  category-complete by construction.*
- *2026-08-24 — #720 round 2 absorbed: the deferral's displacement rule
  completed.* The round's 3/3 major was right — `imposes_quality` still
  let the ScopedPairwise floor displace a held merge-tap with a DI
  winner weighed only against the NATIVE, never against the merge it
  unseats. The rule is now symmetric with the incumbent-deferral: a
  held challenger suspends the remaining PAIRWISE stages and waits on
  the RANKED ones, whose tiers include it (this also reproduces the old
  Terminate world exactly — those stages never ran on these fields).
  Pinned by a fourth policy test whose discrimination was executed
  (suspension disabled → FAIL, restored → 47/47). The BestAccepted
  major was half-refuted: that tier INCLUDES the merge-tap whenever it
  is accepted, so its displacement IS a comparison; the unaccepted-mt
  edge (accepted candidate beats a hard-gated hold) is the acceptance
  gate ranking them — kept deliberately, now documented in the
  ChallengerBehavior contract. The three data minors (empty kit class,
  a typo, stale findings prose) were refuted by direct inspection —
  the row reads "overlapping kit chests", the word is spelled once and
  correctly, and the findings sections regenerate in the same joiner
  run by construction.*
- *2026-08-24 — #720 round 1 absorbed (three real catches, two of them
  majors).* (1) The policy TABLE's k1 registration still declared the
  `partitioned` gate the shipping predicate dropped — the #692
  three-lists drift class, caught 3/3; registration + gate test
  re-pinned to the loosened conjunction. (2) A reachable hole in the
  deferral: with merge-tap produced but itself UNACCEPTED, the held
  challenger fell through to `FirstProduced`, which names the
  first-registered produced candidate — the dead native — strictly
  worse than the old Terminate. Fixed as a principled rule
  (`StageSpec::imposes_quality`): the unconditional fallback's
  registration-order pick is not a quality verdict and may not displace
  a held quality-key win; pinned by a third policy test. (3) The
  measurement widening stated honestly: merge-tap's pre-decide site
  records kinds, not counts, so EVERY Pooled unaccepted-native field
  measures now (not only rescue-bearing ones) and any error-free
  candidate may displace the held merge-tap — which is `BestErrorFree`'s
  job, not a k1 special case; the loop now skips already-counted rows
  (first-write-wins verified in code), so the cost is one validate()
  per uncounted candidate on already-slow broken fields. Also absorbed:
  the challenger-deferral count debug_assert (mirroring #698 round 7's
  incumbent guard) and the evidence header's corpus-fingerprint wording
  (definition-hash, layout-independent — a reviewer read it as a file
  hash).*
- *2026-08-24 — **Phase A1 LANDED** (the reachability fix, this PR): the
  three blockers removed — `try_k1_shape_fix` un-gated from PD (still
  native-unaccepted-only, so clean fixtures never build it), the
  clean-flags laziness measures any produced-but-unmeasured candidate,
  and the MergeTap stage's challenger win defers to the ranked stages
  (`ChallengerBehavior::DeferToRankedStages` — held, re-tagged to itself
  on mere confirmation, so no-rescue fields are bit-identical). Blast
  radius measured three ways: policy suite 45/45 with both defer
  semantics pinned; full core suite 1,249/1,249; the calibration
  fingerprint probe drifts on EXACTLY one of 35 rows (ec35). The new
  ec35 winner (k1, 6,530 entities, 0E/10W): meter 33.49/35 = 95.7%; sim
  **32.8/35 = 93.7% delivered, converged** — kit-flagged (the fixture's
  recurring overlapping-drain rig class, 14 errors) and measured
  IDENTICALLY on two byte-variant builds of the same structure (the
  recorded router-micro-swap nondeterminism class), from the shipped
  22.9%. Bank re-recorded (fingerprint + evidence table; 34 rows
  carried byte-verified). Open follow-ons, Phase A's tail: ec40's k1
  layout carries 10 errors of its own (merge-tap correctly still ships
  there — diagnose next); tier5's three trapped items are K≥2 so
  `build_k1_enrollment_plan` refuses ("no k1 enrollment") — needs the
  K≥2 enrollment extension; the k1 artifact is duty-blind (the ~6%
  residual is its zero-headroom, Phase B's business).*
- *2026-08-24 — tier5's "router block" RE-DIAGNOSED: same class, not a
  junction-solver problem.* The stamp-site probe on current main (duty
  0.6, 175×255 / 7,057 entities — the direct-path shape the 2026-08-17
  entry left open) shows three `BalancerStamped { template_found: false }`
  families — copper-cable **(6,11)**, electronic-circuit **(10,3)**,
  iron-plate **(2,11)** — ALL coprime, all genuinely unstampable, and
  the 18 belt-dead-end errors are those families' disconnected producer
  rows. The 2026-08-15 "known engineering class (junction solver)"
  attribution is RETRACTED; the flip's tier5 precondition folds into
  Phase A (no router campaign needed). Selection footnote: tier5 ships
  its BROKEN native because native's 18 route-severed beat merge-tap's
  27 in the pairwise — k1-shape-fix NotRun (the same Pooled gate), so
  the best broken layout wins a field the purpose-built rescue never
  enters. The coprime-trap ledger is now four shapes across three
  fixtures — (4,9), (6,11), (10,3), (2,11) — which settles A1
  (reachability) as the primary track: the class enumerates faster than
  a library can chase it. Whether `build_k1_enrollment_plan` covers
  multi-family enrollment (tier5 needs three at once) is Phase A's
  first execution question.*
- *2026-08-24 — resumption diagnosis: ec35/ec40's deficit is rescue
  REACHABILITY, not headroom.* Probes on merged main (v2 selection,
  post-RFC-071): the duty knob alone does not move either fixture — both
  build byte-identical artifacts at duty 1.0 and 0.6 (the shipped
  merge-tap winners, 6,311/6,400 entities, 313E/631E). The selection
  scoreboard (trace) shows why: the Pooled native Produces but is
  hard-gated ("1 missing-balancer-template warning"), and the site probe
  names the family — **copper-plate (4,9), merge_tap=false** — the exact
  "(4,9) coprime trap" `try_k1_shape_fix`'s own comment cites, absent
  from the library ((4,1)..(4,8) exist; gcd(4,9)=1), with no
  `BalancerStamped` event fired for it, so the native's 4 route-severed
  errors are the dead-ended producer rows the warning describes. The
  warning is genuine, not the phantom (1,1) class. Three stacked
  blockers then keep the purpose-built rescue off the field: (1)
  `try_k1_shape_fix` requires `PartitionedDecomposed` — ec35/ec40 ship
  Pooled; (2) with the gate removed, k1 Produces and is accepted but the
  early MergeTap preliminary decision skips the clean-flags measurement,
  so its counts stay `None` (the v1-laziness bijection predates a
  Pooled rescue); (3) with counts measured (k1: **0 errors** / 10
  selection warnings vs merge-tap's 313), the MergeTap pairwise stage
  still terminates the program before `BestErrorFree` ranks it. Each
  blocker was correct in the world it shipped in; jointly they ship a
  313-error layout over an available 0-error one. Experiment stack
  banked (scratch diff, 66 lines, all three levers): with stage order
  BestErrorFree-first, `k1-shape-fix` wins and exports end-to-end.*
- *2026-08-24 — PD+duty sim anchors LANDED: the class attains plan.*
  `ec40-pd-duty06` (432k warmup, speed 32): **PASS, converged,
  kit-clean, 39.2/40 = 98.0% delivered** — from the shipped 18.5%.
  `ec35-pd-duty06`: converged at **36.0/35 delivered (+2.9%)** — at
  plan, but 14 overlapping-drain kit errors (the same rig-geometry
  class as this fixture's bank row) make it labelled evidence, not a
  clean clear; the meter's 99.2% corroborates. These anchor the
  EXISTENCE proof (structure + headroom compose to plan on this
  family); the shipping artifact after Phase A is the k1 shape, which
  gets its own anchors before any bank row is re-recorded.*
- *2026-08-24 — the candidate-arm evidence table (meter unless noted;
  sim anchors per the entry above):*

  | ec35 arm | validator | delivered % |
  |---|---|---|
  | shipped Pooled merge-tap | 313 E | **22.9 (sim)** |
  | PD native, duty 1.0 | 0 E | 90.7 |
  | **PD native, duty 0.6** | 0 E | **99.2** |
  | k1 on Pooled, duty 1.0 ≡ 0.6 | 0 E | 95.7 |

  *ec40: shipped 631 E / 18.5 (sim); PD native duty 0.6 = 0 E / 97.3
  meter. Two reach findings ride along: (a) under PD strategy the
  NATIVE builds clean (partitioning never produces the coprime shape),
  k1 not needed there; (b) the k1 artifact is **duty-blind** (byte-
  identical at 1.0/0.6) — its enrollment plan never consults the block
  cap, the third instance of the reach class (ec60-red's candidate
  path, and now this). Phase A's track choice weighs A2's composability
  (native + template responds to duty like ec30) against A1's class
  coverage; the measured PD+duty arms are the at-plan existence proof
  either way.*
- *2026-08-24 — Phase R re-verification: the 2026-08-15 receipts stand
  under v2 selection.* ec30 duty 0.6 → 4,934 entities (the receipted
  gate-clearing artifact's dims/entities; meter 96.4 vs the receipt's
  96.6 whose sim delivered 99.4); ec60-red duty 0.6 → 6,361 entities,
  meter **99.4** (receipt: 96.5 metered, sim 100.0 at plan). The duty-1.0
  baselines reproduce the banked winners exactly (ec30: 3,369 / 96×140).*

- *2026-08-17 — the tier5 router-block receipt is NOT reproducible on
  the direct path, and one of its two blockers was a balancer defect.*
  #652's lever 1 (PR #659) reserves a balancer stamp's WIDTH, not just
  its height: `family_stamp_plan` guarded template width against output
  count on the decomposed and generated paths but never on `Direct`,
  and 31 library templates are wider than their output count, so a
  family's stamp spilled onto its neighbour's trunk columns. Two
  consequences for this RFC:
  - **ac7-HS at duty 0.6 — the flip shape — goes from 14 errors to
    ZERO**, and its 21-tile iter-capped mega-cluster resolves with the
    spill. **SIM-ANCHORED A/B** (both arms 432k warmup, axis declared,
    kit-clean): pre-fix produces **0.00/s on every stage, −100%, not
    converged, zero machines working** (49 producers backed up, 63
    consumers starved — the severed plastic trunk, and plastic is an AC
    ingredient); post-fix **4.51/7.00 = 64.4% of plan**, converged,
    75 machines working. Both arms still FAIL. This removes a
    structural break; it does not make the fixture attain plan, and the
    residual −35.6% is uniform across every stage — one shared
    constraint, i.e. this RFC's own zero-headroom/duty territory rather
    than a router question.
  - Calibration row worth banking: the fast meter read **64.3%** where
    the sim reads **64.4%** on the post-fix artifact, on a fluid chain
    where the meter has a known gap (#570). The meter's below-plan half
    held to 0.1pp; its `produced {}` on the pre-fix arm was likewise
    confirmed by the sim's −100%.
  - **The 2026-08-15 tier5 entry below ("9 belt-dead-end + 1
    unresolved-junction") does not reproduce through
    `build_bus_layout`**, which reports 18 belt-dead-end and **0**
    unresolved-junction on that fixture — and reports them
    IDENTICALLY with and without the balancer fix (175×255, 7057
    entities, both arms). The two numbers came from different export
    paths. The entry stands as the record of what the sim-export
    artifact carried; it should NOT be cited as the direct-path
    baseline, and tier5's gate-on-an-error-free-artifact is still open
    against the 18-dead-end shape. Whoever resumes it re-diagnoses
    against that, not against the "68-tile / 1547-var" mega-zone
    figures.

- *2026-08-15 — **OWNER CALL: correctness over footprint.*** The
  owner's direction (in-session, verbatim: "we should agree that
  correctness is more important than footprint") resolves K69-4's
  reserved trade-off and sets Phase 3's default direction: the
  duty-shaped provisioning ships ON by default once the family
  semantics settle (threshold form, tier generalization, tier5's
  router unblock), accepting the measured footprint cost (+25–45%
  entities on the affected fixtures) in exchange for plan attainment.
  Compactness remains the SECONDARY objective — a "compact" escape
  hatch (duty 1.0) stays available, and the selection objective's
  density term will need re-weighing during the flip so it stops
  punishing the correct shape. Phase 3's corpus adjudication
  (pins/goldens/scoreboards re-blessed against sim receipts) is the
  execution vehicle.

- *2026-08-15 — reach-PR review round 2 tightened the gate, and it was
  right to.* Two correct majors: the `solid_inputs >= 2` guard also
  reached TripleInput and FluidDualInput rows (the kind has TWO solid
  inputs — the "fluid+solid duals never enter" claim was structurally
  wrong), and the "low-draw ⇒ no-op" arithmetic was wrong (for an
  input-bound dual row, block ≈ duty × cap — it always bites at
  duty < 1). The cap is now factored into ONE helper
  (`duty_input0_block`) gated on exact `RowKind::DualInput` AND
  input₀ draw ≥ 10% of the full-belt budget — a fitted threshold
  sitting between the measured-win fractions (EC: 30% yellow, 15%
  fast) and the measured-harm-when-shrunk class (2–7%); Phase 3 owns
  its final form. The MEASURED artifacts are unconfounded either way
  (ec30/ec60-red's only DualInput recipe is EC): re-export under the
  tightened gate reproduces identical row structures and validator
  state; the bytes differ by a 4-tile router micro-swap (3 surface
  belts ↔ 1 UG pair) — the repo's recorded layout-nondeterminism
  class, throughput-neutral, so the sim receipts attach to the
  row-structure class.

- *2026-08-15 — ec60-red's non-response ROOT-CAUSED by the scoring
  trace, and the reach fix pre-registered.* `DecompositionChosen`:
  at duty 1.0 the HS candidate wins (the 4,967-entity baseline); at
  duty 0.6 the capped HS variant balloons to 6,572 entities, its
  density score loses, and NATIVE wins — whose vertical dual-input EC
  rows (7×6) never consult the HS-branch cap. The lever's reach was
  hostage to a delivery-blind density objective. Fix (post-#650, next
  PR): the input₀ block cap now applies on every candidate path, gated
  (after the reach PR's round-2/3 review tightening) on exact
  `RowKind::DualInput` AND input₀ draw ≥ 10% of the full-belt budget —
  NOT this entry's original "≥2 solid inputs" wording, which round 2
  caught reaching TripleInput/FluidDualInput (wording corrected round
  5; the code and the helper's doc are authoritative). The measured 1a
  harm was shrinking single-input rows; the measured wins are the
  high-draw DualInput class. First artifact: ec60-red at duty 0.6
  = 6,355 entities / 176×207 (+28% entities, +64% bbox vs baseline —
  K69-4 watch), 0 errors, meter **96.5% produced** (up from 90.8;
  ec30's gate-clearing artifact metered 96.6 and simmed 99.4).
  **Pre-registered gate: ec60-red 432k sim ≥ 96.0% delivered.
  GATE CLEARED — AT PLAN: 60.00/60.00 produced (+0.0%), 61.60
  delivered (+2.7%, in-flight buffer), PASS, converged, 338/340
  working / 2 ingredient-short. From 90.7% to 100.0%.***
- *2026-08-15 — tier5 under the extended cap: promising but
  ROUTER-BLOCKED.* At duty 0.6 the deep chain reshapes (6,190 → 6,970
  entities) and the meter reads PU at **100% of plan** — but the
  export carries **9 belt-dead-end + 1 unresolved-junction ERRORS**
  (+56 reachability warnings): the ghost router failed to wire the
  fragmented rows cleanly, and a meter reading on structurally broken
  geometry clears nothing. tier5's lever is real but gated on router
  capacity for the fragmented deep-chain shape — a known engineering
  class (junction solver), not a provisioning question. Parked with
  this receipt; the tier5 gate runs only on an error-free artifact.*

- *2026-08-15 — Phase 2 first family probe: ec60-red does NOT respond
  to the block cap.* At duty 0.6 its EC stage reshapes (2×20 → 7×≤6)
  yet the meter reads **90.8% produced — identical to its baseline**;
  duty 0.3 builds byte-identically to 0.6 (unexplained — the cap
  arithmetic should differ; suspect the candidate-selection path or
  the effective in-lane cap resolution on fast belts, to be probed).
  Two open questions for the next session: (a) ec60-red's actual
  binding site (its own attribute/cable-probe pass — its cable rows of
  10 and 48-furnace plate rows differ structurally from ec30's), and
  (b) why 0.3 ≡ 0.6 on this fixture. K69-3's family-divergence
  expectation is now a measurement: the ec30 lever is NOT the whole
  family's lever. **(b) partially answered same day**: builds at duty
  1.0 / 0.6 / 0.3 give 4,967 / 5,395 / 5,395 entities — the duty<1
  winner is IDENTICAL across cap values, i.e. a candidate whose
  construction never consults the cap; its EC 7×6 shape matches the
  shard/partition module size, so the working hypothesis is a
  partition-class candidate winning selection on ec60-red once the
  HS-capped native variant changes the field. The meter's attribute
  pass on its baseline shows the same allocation-skew class as ec30
  (4 cable machines full-output while one EC block starves of cable),
  so the *mechanism* transfers even though the lever's delivery path
  does not. Next: trace which candidate wins and why, then either get
  the capped HS candidate selected or cap the partition module size
  by the same per-pickup rule.

- *2026-08-15 — RFC opened.* Grounded in #648's re-attribution: the
  family's Error-level validator signal was retracted as a walker
  artifact; the sim/meter deficits stand. The #519 margin-probe
  rejection is explicitly discounted as instrument-contaminated (its
  warning-count penalty partly measured the phantom-source bug), while
  its geometric end-to-end insight is adopted as a design constraint.
- *2026-08-15 — **K69-1 FIRED on the Phase-1a design** (duty in the
  general row caps): ec30 at duty 0.9 sims **84.4% delivered**
  (25.33/30, converged, drift +1.0%) — 7.7pp WORSE than the 92.1%
  baseline. Census: 30 machines full_output, the stalled ones being
  row HEADS at zero crafts — the extra producer rows fed a collection
  fabric still provisioned at nominal, so backpressure parked at the
  producers. The 0.85 arm (near-identical layout, different hash) is
  in flight to close the criterion's letter; the design verdict does
  not depend on it. Per the criterion: stop the rows-only lever.
  **0.85 arm landed: identical — 84.4% delivered, same census (30
  full-output / 3 short / 137 working). K69-1's letter is closed:
  FIRED at both pre-registered duties.***
- *2026-08-15 — mechanism REFRAMED by the three-artifact row census.*
  dense (92.1%): plate 3×24 + iron 2×24, EC **2×10**; duty-1a (84.4%):
  plate 4×18 + iron 3×16, EC 3×7/7/6; sprawl (99.4%): plate/iron
  IDENTICAL to dense, EC **10×2**. At-cap producer rows are measured
  FINE; the deficit lives in the **HS consumer fan-in** (dense EC rows
  are output-bound at 10 machines = 45/s of input₀ cable per row via
  K=4 nominal trunks + the collection fabric above them). The
  validator's residual IRD warnings point at exactly these feeds
  (cable 2.9–3.9/s modeled vs 4.5 needed at EC tails). The banked
  dense run's census shows the same head-stall backpressure symptoms
  plus scattered ingredient-shorts — the RFC-061 allocation-skew
  class. The ore-row zero-margin warnings are measured benign for
  delivery (sprawl receipt) — a calibration note for the margin
  check's trust row.*
- *2026-08-15 — **Phase 1b pre-registered** (entry written before its
  sim returned): one input₀ trunk per HS consumer row, gated behind
  the same `planning_duty` knob (duty < 1 ⇒ HS rows capped at
  floor(belt_cap × duty / input₀_rate); the Phase-1a general-cap
  scaling is REVERTED as measured-harmful). Artifact: EC 7×≤3 rows,
  producer rows untouched, 4,161 entities / 100×206 (between dense's
  3,369 and the sprawl's 4,934). Gate: ec30 sims **≥ 96.0% delivered**
  at duty 0.9, else this lever stops too and the campaign escalates to
  RFC-061's pool-and-balance machinery.*
- *2026-08-15 — Phase 1b at block=3 measured DEAD; the physics is the
  per-pickup extraction fraction.* The 1b sim was invalidated by a
  harness kit artifact (overlapping drain banks on adjacent same-item
  exits — a rig-geometry limitation worth its own follow-up: the
  colliding tiles don't even correspond to the manifest's two south
  exits), but the meter refutes independently: **85.0% produced,
  uniform down the chain**. The cable-probe occupancy gradient
  explains why rows-of-3 fail where rows-of-2 succeed: an EC machine
  draws 4.5/s ≈ 30% of a full yellow belt per pickup (vs a furnace's
  4%), so a 3-machine block's tail extracts from a belt already
  two-thirds depleted — the RFC-054 gappy-belt mechanism, 7× harsher
  on EC than on furnace rows. Blocks of 2 kill the within-row
  gradient; DI would kill it entirely.*
- *2026-08-15 — **Phase 1c pre-registered** (entry written before its
  sim returned): same code, `--duty 0.6` ⇒ HS block =
  floor(15×0.6/4.5) = **2** ⇒ EC 10×2 with producer rows untouched —
  the engine now deterministically reproduces the sprawl's shape
  (dims/entities EXACT: 106×222 / 4,934; bp not byte-identical —
  segment/routing deltas). Meter: **96.6% produced — IDENTICAL to the
  sprawl's meter reading**, whose sim delivered 99.4%. Gate: this
  artifact's own 432k sim ≥ 96.0% delivered.
  **GATE CLEARED: 99.4% delivered (29.82/30, −0.6%), PASS — converged,
  kit-clean, 168/170 working / 2 full-output.** The engine produces a
  plan-attaining ec30 behind `planning_duty: 0.6`, +7.3pp over the
  92.1% default. Phase 2 (family rollout) decides the shipping
  semantics: 0.6 was fitted to give block=2 for EC-on-yellow
  (4.5/s draw); whether the fraction generalizes (fast-belt EC →
  block 4) or the rule should pin block ≤ 2 for high-fraction pickups
  is a measurement question, not a design preference.*
- *2026-08-15 — Phase-0 follow-up MEASURED: the phantom-era sprawl
  delivers 99.4%.* Reproduced from `decd63b5` and simmed (warmup 432k,
  converged, drift +0.6%, kit-clean): `ec30-sprawl` (106×222, 4,934
  entities, 170 machines) delivers **29.82/30.0 = 99.4%** vs the
  banked dense winner's 92.1% — the phantom-error steering was buying
  **+7.3pp of delivery** by accident, through exactly this RFC's
  mechanism (more, shorter rows → per-row ore demand below one belt →
  headroom). Cost: +46% entities, +1.5× bbox vs dense. This is the
  headroom hypothesis measured before any engine knob: K69-1's
  question is now whether duty-0.9 buys comparable delivery at a
  fraction of the sprawl's footprint (the duty-0.9 artifact is 3,797
  entities / 81×168 — between the two). Report banked at
  `~/spaghettio-corpora/i644-phase0/ec30-sprawl/`.
- *(superseded by the entry above)* Phase-0 follow-up candidate note:
  For the three days the #646 walker phantoms steered selection, ec30
  shipped a 4964-entity 106×222 sprawl (vs the banked 3369-entity
  winner) that METERS at 96.6% produced vs the dense winner's 91.9%
  (adversarial-review measurement on PR #648). If the sprawl SIMS
  better too, the accidental steering was buying delivery through
  extra rows/trunks — the headroom hypothesis measured for free,
  before any engine code. Reproduce from any pre-#648 main SHA
  (e.g. `decd63b5`, `sim_export` ec30 with the same flags) in a
  throwaway worktree; worth one sim run early in Phase 1.
- *2026-08-15 — Phase 0, tier5-PU sim (axis-declared, kit-clean): WARN
  at 95.6% delivered* (harness delivered-d% = −4.4%, the authoritative
  column — the table's 1.91/2.00 is the rounded display, per
  meter-divergence.md's take-deltas-from-delta-columns rule; warmup
  432k, converged, drift +1.3%;
  uniform ~−5.3% mid-chain; 262 working / 18 full-output / 4
  ingredient-short; report banked at
  `~/spaghettio-corpora/i644-phase0/tier5-pu2-am3-rp/`). The meter's
  85.6% receipt was measured on the UNDECLARED-axis export (different
  plan, 6394 vs 6190 entities) — the like-for-like deficit is ~4–5%,
  not 14%, which also answers the fix-PR review's "zero-headroom
  doesn't explain PU's 14%" objection: the 14% was largely axis
  mismatch. The family's sim-anchored ledger: ec22 −0.6%, tier5-PU
  −4.4%, ec30 −7.9%, ec60-red −9.3%.
- *2026-08-15 — Phase 0, ec22 boundary sim: PASS at 99.4% delivered*
  (21.87/22.00, warmup 432k, converged, drift +0.6%, 120 working /
  5 full-output / 1 ingredient-short). The 22-vs-30 boundary is real
  under measurement, not only under the validator: ec22's ore chains
  run at ~73% belt load (33/s copper over 3 belts) vs ec30's exactly
  100%. Gate corpus stays scoped to the 30/s+ fixtures; ec22 becomes
  the family's clean lower anchor.
