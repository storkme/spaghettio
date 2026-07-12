# RFP: Demand-aware lane provisioning and splitter flow modeling

## Summary

Every `input-rate-delivery` warning in the science gauntlets traces to
one shared modeling assumption: that balancer/splitter outputs divide
flow **evenly**. The lane planner books per-consumer-row lanes at
`total_rate / n_splits` (`LaneSplit` trace event), and the belt-flow
walker propagates splitter output as a static 50/50 division
(`belt_flow.rs:1845`). Real Factorio splitters redistribute under
backpressure — when one output backs up, the other receives everything
— so an even 1×N balancer feeding rows with unequal draws is
demand-adaptive in game as long as aggregate supply and belt capacity
suffice. This RFP runs one decisive ground-truth experiment to
determine which side is lying (the layout or the validator), then fixes
that side: either a demand-pull flow model in the walker (expected
branch) or physically demand-weighted provisioning in the planner.
This is lens-cleaning for the entire scaling program: the 10/s cells
of the scaling gauntlet emit `input-rate-delivery` warnings in the
hundreds under the same even-split model, so we currently cannot rank
scaling walls with confidence.

## Motivation

Reproducible today, stable since at least `ce732d9` (bisect verified
byte-identical warnings across 25 commits — not a regression):

- `logistic-science-pack@1/s` (gauntlet): WARN×2 —
  *"Input belt at (16,29) delivers 1.4/s but machine needs 3.0/s of
  iron-plate (across 1 inserter)"* and a 0.8 vs 1.0/s iron-gear case.
- `military-science-pack@1/s`: WARN×1 — 1.9 vs 2.0/s iron-plate.
- Trace mechanism: `LaneSplit { item: iron-plate, rate: 5.625,
  n_splits: 4 }` → four family lanes at 1.40625/s each, one per
  consumer row, while one row locally draws 3.0/s. The 1×4 balancer
  is stamped valid and connected; the *bookkept* per-lane rate is
  what falls short. `lane_planner.rs::split_overflowing_lanes` forces
  `n_splits ≥ consumer_rows.len()` for non-external lanes, and
  divides rate uniformly.

Evidence that this is a modeling artifact rather than a physical
starvation (to be confirmed in Phase 0, not assumed):

- **Splitter mechanics**: a splitter with a blocked output routes its
  full input to the unblocked output. Steady-state, an even balancer
  feeding unequal consumers self-corrects via backpressure whenever
  Σ(supply) ≥ Σ(demand) — which holds here (5.625 = the solve's own
  total).
- **Non-monotonicity** (scaling gauntlet at `825e940`): logistic
  clears to PASS at 5/s and military at 2/s. A genuine provisioning
  shortfall would worsen with scale; a small-N bookkeeping artifact
  (few machines → coarse slices) fades as demand evens out.
- **Precedent in our own walker**: self-loop priority splitters
  already get demand-like semantics (`loop_priority_rate`: the loop
  branch receives `min(total, cap)`, not half) — the 50/50 rule is
  already known to be wrong for at least one splitter class.

Why it matters beyond two warnings: `utility-science-pack@10/s` emits
655 `input-rate-delivery` + 730 `belt-flow-reachability` warnings
around 35 real dead-end errors. If the even-split model inflates
warning counts at scale, the scaling program's fix-target ranking is
being read through a dirty lens. Validator fidelity is upstream of
every scaling decision (this is also the July review's "validator
never calibrated against real Factorio" item, made concrete).

## Design

### Phase 0 — the decisive experiment (no production code)

1. Export the `logistic-science-pack@1/s` blueprint and ground-truth
   it in game (first real calibration anchor): does the iron-plate
   row at the warned coordinates starve, or does balancer
   backpressure deliver its 3.0/s? User assist welcome; a
   desk-verification against `docs/factorio-mechanics.md` splitter
   rules plus a snapshot audit of the actual delivery path
   (sideloads and UG lane rules can genuinely block backpressure
   redistribution — the path must be checked tile-by-tile, not
   assumed) is the fallback if in-game isn't convenient.
2. Record the verdict in this RFP's decision log. It picks the
   branch; the branches are mutually exclusive.

### Branch A — walker demand-pull model (expected)

Replace the static 50/50 splitter division with a bounded fixed-point
demand-pull pass in `validate/belt_flow.rs`:

- Downstream demands propagate upstream (Kahn order already exists);
  each splitter routes flow toward unmet demand up to per-output belt
  capacity, falling back to even division only for genuinely
  symmetric residuals.
- Iterate to a fixed point with a hard iteration budget; reuse the
  existing splitter-pairing and balancer-feedback machinery rather
  than growing a parallel graph.
- True positives must survive: a genuinely under-supplied input
  (Σ supply < Σ demand, or a capacity-capped path) must still warn.
  New unit fixtures pin both directions (redistributes-under-
  backpressure AND still-warns-when-truly-starved).
- The planner's `LaneSplit` bookkeeping becomes advisory; no layout
  geometry changes at all in this branch (hard gate: zero golden
  movement).

### Branch B — physical demand-weighted provisioning (if Phase 0
shows real starvation)

Options in preference order, all respecting the hard constraint that
belt tier is user-specified and never auto-escalated:

1. **No-split-below-cap**: when `total_rate ≤ lane_cap`, keep ONE
   shared lane with per-row taps (upstream taps drain first;
   backpressure feeds the rest) instead of forcing
   `n_splits = consumer_rows.len()`.
2. **Priority-splitter chain**: first row taps a priority output
   sized to its draw; overflow continues. Walker already has
   `loop_priority_rate` vocabulary to model it.
3. **Slice-and-merge weighted balancing**: split into K > N even
   slices and merge subsets per row to approximate demand ratios.
   Rejected-by-default: inflates balancer footprint (#135) and risks
   requiring balancer shapes the library lacks (#136 — CP-SAT
   synthesis of coprime shapes is a confirmed hard wall, 0/32
   overnight seeds).

## Kill criteria

1. **Phase 0 contradiction**: if in-game ground truth contradicts
   `docs/factorio-mechanics.md`'s splitter rules, STOP this RFP
   entirely — the mechanics doc is the substrate for every walker
   rule, and recalibrating it is a bigger, prior problem. Do not
   build on a substrate known to be wrong.
2. **Branch A convergence**: if the demand-pull fixed point fails to
   converge within `3 × segment_count` iterations on any existing
   corpus layout, or needs special-case handling beyond what the
   existing splitter-pairing/feedback machinery already isolates,
   the iterative walker model is wrong — fall back to Branch B
   option 1 or stop.
3. **Branch A runtime**: if full-suite validation wall-time regresses
   more than 2× on the 1/s gauntlet corpus, drop the iterative model
   regardless of correctness.
4. **Wrong root cause**: after the chosen fix, if any golden hash
   moves under Branch A (a validator-only change must not alter
   layouts), or the belt-model warnings persist at the warned
   coordinates, the diagnosis was wrong — stop and re-run Phase 0
   rather than iterating on the fix. (Original "logistic/military
   must PASS" gate superseded 2026-07-12: honest inserter-throughput
   warnings are EXPECTED to replace the belt-model warnings — see
   decision log; the all-PASS gate belongs to the follow-up
   inserter-sizing RFP.)
5. **Scope fence (evidence gate)**: if utility@10/s's
   `input-rate-delivery` count does not drop by at least half under
   the fixed model, the mass warnings have a different root cause —
   record it as its own finding and do NOT expand this RFP to chase
   it. (The 35 dead-end errors and the production@10/s runtime cliff
   are explicitly out of scope regardless.)

## Verification plan

Per the CLAUDE.md layout-change protocol, plus:

- `science_gauntlet` at 1/s: target 6/6 PASS, zero new warnings.
- `science_scaling_gauntlet`: before/after delta table committed to
  this decision log (headline: utility@10/s warning counts).
- Snapshot inspection of the exact warned row in logistic@1/s
  (delivery path tile-by-tile, not just the warning count).
- Unit fixtures: backpressure redistribution, symmetric residual,
  true-starvation still warns, priority-splitter interop, feedback
  loop interop.
- Netflow determinism sweep byte-identical; zero golden movement
  (Branch A) or explicitly re-blessed with layout diffs reviewed
  (Branch B).
- Browser eyeball of logistic@1/s (user validates per convention).

## Phasing

- **Phase 0** — ground-truth experiment + branch decision. No code.
- **Phase 1** — the chosen branch's implementation + unit fixtures.
- **Phase 2** — re-run both gauntlets, commit the delta table,
  re-rank the scaling program's remaining walls with the clean lens,
  refresh CLAUDE.md ladder notes if tier rows change.

## Decision log

- *2026-07-11 — drafted, following the gauntlet bisect (warnings
  byte-identical since ce732d9; uniform LaneSplit mechanism traced)
  and the first science_scaling_gauntlet run (825e940): non-monotonic
  1/s-only warnings for logistic/military, mass same-category
  warnings at 10/s cells. The scaling scoreboard demoted this from
  "presumed dominant scaling fix" to "correctness fix + measurement
  lens" — the genuine scale walls (utility@10/s topology break,
  production@10/s runtime cliff, chemical@10/s errors) are separate
  work items that should be ranked AFTER this lens is clean.
  Pending review.*
- *2026-07-12 — **Phase 0 resolved without the in-game run, and the
  RFP gains a mandatory companion.** (a) Splitter backpressure
  redistribution confirmed by the user as a domain call, consistent
  with `docs/factorio-mechanics.md` — kill criterion 1 does not
  fire; Branch A's belt-model premise stands. (b) Reviewing the
  warned machine exposed a THIRD mechanism neither branch covered:
  the gear machine needs 3.0 plates/s in and 1.5 gears/s out at
  planned 100% utilization, but the template feeds and drains it
  with ONE regular inserter per side (~0.84/s each, mechanics table
  I8; blueprint-verified at the warned coordinates). The machine is
  INSERTER-bound at ~28% regardless of belt delivery — the existing
  warning reaches the right verdict via the wrong mechanism, and
  shipping the walker fix alone would flip it to a false PASS.
  Systemic: every template places one regular inserter per machine
  side, so any machine whose per-side rate exceeds ~0.84/s is
  silently capped engine-wide (the July review's "inserter-throughput
  saturation unchecked" gap, now concrete). **Phase 1 therefore
  ships as an inseparable pair**: the demand-pull walker model AND a
  new inserter-throughput check (per machine side: required rate vs
  Σ inserter rates by type from the I8 table, utilization-scaled).
  Expected honest outcome: belt-model warnings are REPLACED by
  inserter-throughput warnings and total counts may rise — that is
  the lens getting cleaner. Kill criterion 4's "logistic/military
  must PASS" gate transfers to the follow-up layout-side RFP
  (inserter sizing: faster tiers or multiple inserters per hungry
  machine side — touches every template and every golden, explicitly
  out of scope here). The optional in-game run's observable, if
  performed: inserter swinging nonstop over a full belt = inserter-
  bound (expected); idle inserter over an empty belt = belt-bound
  (would reopen Branch B).*
