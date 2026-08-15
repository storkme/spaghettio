# RFC-069: Achievable-duty provisioning for the zero-headroom family

**Status**: Draft
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

## Decision log

- *2026-08-15 — ec60-red's non-response ROOT-CAUSED by the scoring
  trace, and the reach fix pre-registered.* `DecompositionChosen`:
  at duty 1.0 the HS candidate wins (the 4,967-entity baseline); at
  duty 0.6 the capped HS variant balloons to 6,572 entities, its
  density score loses, and NATIVE wins — whose vertical dual-input EC
  rows (7×6) never consult the HS-branch cap. The lever's reach was
  hostage to a delivery-blind density objective. Fix (post-#650, next
  PR): the input₀ block cap now applies to DUAL-input rows on every
  candidate path (≥2 solid inputs only — the measured 1a harm was
  shrinking single-input producer rows; the measured 1c win was
  capping dual-input consumers). First artifact: ec60-red at duty 0.6
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
