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
- `tier5_processing_unit_from_ore_am3`: meter **85.6% produced**
  (1.712/2.0), uniform choke signature across the chain — one shared
  constraint propagating (status.md's y=mx+c reading). Sim anchor:
  Phase 0 below.
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
   90%, and the family's deficits cluster at 85–92%). Documented as
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

Expected shape of the diff: ec30 copper goes 3 trunks / 3×24-furnace
rows → 4 trunks / 4×18-furnace rows (each at 11.25/s = 75% of a belt);
footprint grows by roughly one row + one trunk column per over-tight
item. Footprint cost is reported per fixture in the decision log —
if it lands anywhere near the measured +10–75 entities/machine of the
solver-side rule's scoping, the trade goes to the owner before merge.

## Kill criteria

- **K69-1 (mechanism)**: Phase 1's duty-provisioned `ec30` layout does
  not improve sim delivered% by ≥ +4pp over the 92.1% baseline
  (i.e. fails to reach ≥ 96%) at any duty in {0.85, 0.9} → the
  dominant loss is not headroom on these chains; stop, return to the
  loss-reduction lever (the pooled-vs-partitioned confound), and take
  the probe evidence to #644.
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

- *2026-08-15 — RFC opened.* Grounded in #648's re-attribution: the
  family's Error-level validator signal was retracted as a walker
  artifact; the sim/meter deficits stand. The #519 margin-probe
  rejection is explicitly discounted as instrument-contaminated (its
  warning-count penalty partly measured the phantom-source bug), while
  its geometric end-to-end insight is adopted as a design constraint.
- *2026-08-15 — Phase-0 follow-up candidate: the phantom-era sprawls.*
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
  at 95.6% delivered* (1.91/2.00, warmup 432k, converged, drift +1.3%;
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
