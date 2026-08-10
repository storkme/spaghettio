# Handoff: wire the meter into a gate (and what else 2026-08-07 left open)

**Status: session note, committed 2026-08-10 to stop it being lost.** Written at
the end of a long session on 2026-08-07; treat every number here as sourced
(each says where from) but the framing as one session's read. It carries no
durability contract — archive or delete it once absorbed.

**Take me to the work:** `main` at `cedc01ae` *as of writing*; well behind now.
Nothing was blocked at the time. The next thread was §1 — since superseded in
part by [RFC-066](rfc-066-lane-rate-arbitration.md), which cites this note's
§2d/§2e framing but depends on none of it.

---

## 0. What landed today

| PR | what | state |
|---|---|---|
| #601 | `PlacedEntity::rate` semantics settled — it is an aggregate at all 89 stamp sites, never per-tile flow | merged |
| #603 | `physical_utilization` lost fractional-duty scaling on the banded path | merged |
| #605 | lifted the `input-rate-delivery` selection exemption | merged |
| #604 | Graphite export, dashboard, live streaming incl. warmup, divergence log | **open, on CI** |

Product outcome: three target configs went from broken to working —
`processing-unit@1/s` **68.2 → 102.0%** of plan, `advanced-circuit@5/s`
**83.3 → 99.7%**, `tier2_electronic_circuit` **58 → 91%**, with
`big-electric-pole@1` holding at plan as the regression canary. All four
sim-measured, kit-clean, converged.

`docs/status.md` recorded this deficit class as "a throughput/distribution
ceiling, NOT a productivity-modelling error". It was neither — it was
candidate selection being blind to a starvation warning the validator was
already emitting.

---

## 1. THE NEXT THREAD — meter as a gate

**Why it's worth doing:** `crates/meter/` predicts below-plan in **~19
seconds** where the headless sim takes 10–20 minutes. It would have caught
`tier2_electronic_circuit` before any of today happened.

### The distinction I got wrong first, stated properly

"Safe as a floor" is meaningless without saying what the gate *does*:

- **Report-only** cares about **missed defects** — meter says at-plan when
  reality is below. That quadrant is **empty across all 41 compared corpus
  rows at every threshold ≥90%** (only appears at a bit-exact 100% cutoff,
  where the hits are sim readings of 99.0–99.7% against a meter 100.0%,
  i.e. inside the noise band). **Safe today.**
- **Blocking** cares about **false accusations** — meter says below-plan
  when reality is fine. There the corpus has a bad one: `PU-from-ore` reads
  **85.8% where the sim reads 99.4%** — a **13.6pp** false accusation. At a
  95% threshold that rejects a good layout. **Not safe today.**

### Critical path

1. **Land the research-productivity axis.** PU's −13.6pp is exactly this —
   both PU rows match `docs/meter-divergence.md`'s recorded ≈−13% for the
   unmerged axis. It is a missing feature on a branch, not a modelling
   mystery, and it is the single largest false positive. Removing it is the
   unlock.
2. **Re-characterise** with `crates/meter/examples/sweep_corpus.rs` against
   `~/spaghettio-corpora/job2-sim-baselines/2026-08-01/` (54 reports, ~minutes).
3. **Wire report-only first**, per `validator-trust.md`'s own trust ladder —
   rate models report before they steer. At 19s/fixture it cannot run in the
   default suite; it wants STRESSGOLD-style opt-in.
4. **Only then consider teeth**, with the false-positive rate measured.

### Calibration facts you can rely on (41 compared rows, corpus-wide)

- 25 pessimistic, 6 optimistic, 10 at ~0pp. Mean −1.27pp, median −0.3pp.
- **Every optimistic error is ≤ +1.3pp anywhere in the corpus.** Pessimistic
  errors run to −13.6pp.
- **The meter's own convergence floor is ~20–40k ticks**, measured down to
  warmup 0 on the deepest fixture (6499 entities: 78.9% at zero, plateau by
  ~40k). The 108k that `check_one.rs` and `sweep_corpus.rs` both use has a
  3–5× margin even there.
- **"The default warmup is too short" does NOT transfer to the meter.** That
  caveat is a property of headless Factorio's convergence needs (hence the
  corpus's escalating per-fixture warmups). Assuming it carries over is a
  mistake I made and it cost a round trip.

---

## 2. Open, with evidence

### 2a. The meter's post-lift optimism — unexplained

On the **post-lift** `tier2_electronic_circuit` layout the meter reads
**96%** where the sim reads **90–91%** — optimistic by ~5–6pp, which is **4×
the largest optimistic error in the whole corpus**. The corpus's own tier2
row is the **pre-lift** layout and reads *pessimistic* (56.0% vs 57.7–58.1%).
Different layouts; do not average them.

Warmup mismatch was the obvious suspect and is **falsified**: the meter reads
96.0% at every warmup from 108k to 864k (8× range, including the sim's own
432k), zero movement, converged throughout. The corpus was checked too — 11
of ~20 fixture families genuinely did run sim baselines at 288k against the
meter's 108k, and re-running moved nothing by more than 0.5pp.

So it is **genuine model-level disagreement** and needs snapshot/trace-level
work. **This matters for §1**: the floor property is established for the
corpus and NOT for post-lift layouts, which is the population a gate runs on.

### 2b. Zero-headroom integral machine counts — prevalent, not sufficient

When a stage's planned count lands exactly on an integer there is no rounding
headroom, so the plan implicitly demands 100% duty forever. Measured:
`copper-cable` plans at exactly 10.0 machines (30.0/s capacity vs 30.0/s
required) and runs at 90% duty; EC inherits it stoichiometrically (3 cable
per EC → 27/3 = 9.0 vs 9.09 measured).

- **69/146 stages (47%) are exactly zero-headroom**, across 28/40 fixtures.
- The near-miss class is as large: **92/146 (63%) under 2% headroom**.
- Cost: a flat "+1 machine when headroom < X%" is ~3× cheaper than
  multiplicative (+107 machines at <5% vs +357 at ×1.05). Entity cost is not
  linear — ~10 entities per bumped machine shallow, 50–75 on deep chains.
- **NECESSARY BUT NOT SUFFICIENT.** Every fixture below plan has a
  zero-headroom stage, but several whose *target* stage has zero headroom
  measure 97–107%. Decisive:
  `stress_advanced_circuit_partitioned_5s_from_plates` has **identical**
  solver output and headroom in its pooled vs partitioned variants and
  measures **80% vs 98–100%**. Layout strategy is a confound.

**Measured cost: ~8–10% per fully-zero-headroom fixture** (see §2e, which
separated it from the selection bug). So zero-headroom is the **removal of the
margin** that would otherwise absorb routing/inserter loss, not the defect
itself — which means *reducing the
loss* is as valid a direction as *adding machines*. **Adding headroom costs
footprint, and that trade is the owner's call.**

### 2c. `input-rate-delivery` is blind to zero-headroom

The check now steering selection derives required rate at **nominal duty**, so
it structurally cannot warn on an exactly-integral-count stage — ~47% of
stages. The re-ranked tier2 winner has **zero** warnings from it and still
sims at 91%. Recorded at the call site in `validate/mod.rs`. The lift is a
partial fix to a class with at least two independent causes.

### 2d. Two lane-rate models disagree

`belt_flow::compute_lane_rates` and `belt_structural::compute_lane_rates`
both exist; `validate/mod.rs` dispatches **`belt_structural`**, while
`bus/template_validate.rs` uses **`belt_flow`**. They disagree on the S=1 ore
belts (36/s vs nothing over cap). Both are independent Python→Rust ports —
the duplication is a port artifact. `belt_flow` has the #519 consumption
decrement and an iterative convergence pass; `belt_structural` has neither.

**Parked during the session — but §2e weakens the reason.** I parked it on the
arithmetic that `belt_flow`'s 36-on-a-30 predicts a ~17% throttle where the
fixture measured ~50%, i.e. the wrong order of magnitude to be the cause. Post
lift those fixtures measure **~90%**, a ~10% shortfall — now the *same* order
as that prediction. So `belt_flow` flagging the S=1 ore belts may be pointing
at the same residual §2b attributes to zero-headroom, and the two explanations
are not obviously distinct.

That does not make `belt_flow` right — it is the *non*-dispatched model and
neither has ever been arbitrated — but "it can't be the cause, wrong
magnitude" no longer holds. Worth re-examining alongside §2b rather than
staying parked. Full reasoning in `docs/rate-stamp-semantics.md`.

### 2e. RESOLVED — the zero-headroom tax is ~8–10%, not ~50%

Both stress-EC fixtures re-measured post-lift. Valid runs (`kit_errors` empty,
`fluid_errors` empty, converged, 4 checkpoints), pushed to Grafana as
`arm=postlift`.

| fixture | pre-lift | post-lift |
|---|---|---|
| `stress_electronic_circuit_30s_from_ore` | ~50% | **~92%** (27.27/s vs 30 planned) |
| `stress_electronic_circuit_60s_red_from_ore` | ~50–51% | **~90%** (53.90/s vs 60 planned) |

**The historical ~50% was overwhelmingly the selection bug**, the same jump PU
(68.2→102%) and AC (83.3→99.7%) made. My scoping report suspected the two
defects were confounded; this separates them by measurement rather than
inference.

**But a real residual remains, and it is the zero-headroom tax.** Both land at
~90–92%, not ~100%, and the shortfall is *uniform across every stage with no
dispersion* — a 360s wide-window cross-check on all four stages of the 30s
fixture reads 91.8% on every one (EC 27.55, cable 82.61, copper-plate 41.29,
iron-plate 27.54). Both fixtures have **all four stages exactly
zero-headroom**, and the shape and magnitude match tier2's already-root-caused
residual. Drift 0.6–2.0%, so this is not noise.

**So: read §2b's 47%-of-stages prevalence against an ~8–10% typical cost per
fully-zero-headroom fixture — not against the dramatic pre-lift deltas.** That
changes the economics of a fix substantially, and it is the number to weigh
against the footprint cost.

**Not settled**: two data points, same recipe family (EC-from-ore), adjacent
rates. Whether ~8–10% is a general zero-headroom tax or specific to this
chain's inserter/belt geometry needs a structurally different fixture —
`tier5_processing_unit_from_ore_am3` also has 4 exactly-zero stages and would
triangulate it. Not run (5540+ entities, ~20+ min).

---

## 3. Traps that cost time today — each one bit

1. **Raising `--warmup` requires raising `--timeout-secs`.** The derived
   timeout is 4× the tick budget at the *requested* speed; a loaded box runs
   slower, so a long warmup gets killed mid-warmup. The failure is quiet:
   no verdict, no `kit_errors`, empty `timeseries.csv`, and no Factorio
   process — which reads exactly like a completed run. I reported a result
   that did not exist. Grep for `timed out after Ns waiting for
   harness-result.json`. Documented in `sim-harness.md` (#604).
2. **Research-productivity parity refuses runs ~16 minutes in.** The install
   has productivity on `plastic-bar`, `processing-unit`, `steel-plate`. If a
   chain touches one and the manifest doesn't declare it, `kit_errors` is
   non-empty and the verdict is forced `NO DATA`. Read the realized set from
   any existing report's `raw_result.productivity_force` before exporting.
   **Never quote a run with non-empty `kit_errors`.**
3. **`base_hand_size()` IS `hand_size(0)`**, not an additive constant. I
   added a force bonus on top, double-counted L0, and "found" a bulk
   hand-size bug that does not exist. `entity_data.rs` says *transcribe,
   don't derive* because this table already attracted one wrong fix (#458).
   Mine would have been the second.
4. **Grafana wiring produced four separate false successes** — `perSecond()`
   on backfilled data, interval-unaligned timestamps, stale-point rejection
   (Grafana Cloud silently drops >~1 day old while returning `200
   {published: N}`), and unset dashboard variables. Every one reported
   success with an empty panel. **Verify a panel returns data; never trust
   the write.**
5. **Do not copy files between branches.** I copied `validate/mod.rs` from a
   branch off `main` onto the lift branch and silently **reverted the lift
   itself** — the one line #605 exists for. Only the test suite caught it.
6. **A check that stops discriminating is worse than one that fails.** I
   scoped two fixture probes to a headline item and made both **vacuous**;
   review caught them. Any scoping change needs a non-vacuity guard asserting
   the probe still has something to inspect.

---

## 4. Instruments now available (use them)

- **Dashboard** `/d/spaghettio-sim` — per-stage % of plan, production,
  delivered, raw input, and cumulative production. Filter by `fixture`/`arm`.
- **`scripts/sim-to-graphite.py`** — push any `report.json`; `--anchor now`
  for anything older than a day. Works retroactively on the whole corpus.
- **`scripts/sim-live.sh`** — run a fixture with live streaming and print a
  pre-filtered dashboard link. Includes the **warmup ramp**.
- **The y=mx+c reading** (`sim-harness-forensics.md`) — a healthy stage is
  piecewise {flat, ramp, straight line}. **Straightness is the starvation
  test**: a starved stage is a staircase that averages to a plausible rate.
  Uniform slope-scaling across stages ⇒ shared constraint; one shallow slope
  ⇒ that stage is the bottleneck and downstream inherits it.

Both scripts are on `tooling/sim-graphite-export` (#604), **not yet on
`main`** — check that branch out or run them from it.
