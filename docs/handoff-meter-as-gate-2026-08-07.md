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

> **Correction, 2026-08-10 (at commit time).** Partly superseded. The *deficit*
> this section is measured against was root-caused on 2026-08-08 by #607/#608 —
> `docs/status.md` §1015 records it as the `di-bridge` belt→belt transfer bank
> loading one lane only (~21.4/s against 30/s of demand), and marks the
> zero-headroom attribution **falsified for this fixture** with an explicit
> "do not cite". Post-#608 selection ships the bus-lane variant, which measures
> **100.0% of plan** headless. The *meter-vs-sim divergence* below is not
> superseded and has since become its own record:
> [`meter-divergence.md`](meter-divergence.md) §2026-08-08 is the current head,
> and its conclusion is stronger than "unexplained" — **the floor property does
> not hold on the gate population**, which is why the fast meter can refute but
> never clear. Read this section for its falsification of the warmup suspect;
> go there for the calibration.

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
both existed when this handoff was written (the twin was deleted
2026-08-15, #632 B5); `validate/mod.rs` dispatched **`belt_structural`** then *(since 2026-08-15, #632 B5, it dispatches
`belt_flow` — settled, see `validator-trust.md`)*, while
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

> **Correction, 2026-08-10 (at commit time).** Stale: #604 has since merged.
> Both `scripts/sim-to-graphite.py` and `scripts/sim-live.sh` are on `main`.
> Run them from there, not from the branch. Left in place rather than rewritten
> because this is a dated note — but corrected, since it tells the reader to go
> somewhere, and RFC-066 now links here.

## 5. 2026-08-18 pickup — report-only probe landed

The first safe increment from §1 is now available in the sim harness:
`spaghettio-sim run --meter` runs the fast meter against the exact same
blueprint and manifest, using the calibrated 108k warmup / 216k window by
default. It prints target readings and stores the raw `MeterReport` under the
top-level `meter` key in `--out`.

This is intentionally report-only. It does not gate the Factorio verdict,
candidate selection, or validator output. The post-lift calibration in
`meter-divergence.md` falsified the old floor/clearance claim, so the next
decision remains whether a narrower “meter says below plan” finding channel is
worth consuming—not a blocking gate.

## 6. 2026-08-18 divergence follow-up

The requested longer Factorio calibration was attempted at 864,000 warmup
ticks, in parallel at speed 64, with a 3,600-second timeout. It was stopped
before any report was written: after roughly 11 minutes the five instances
were only at ~185,000 ticks. The existing 432,000-tick reports therefore
remain the usable sim reference; this attempt adds no new rate claim.

The meter-side dumps do identify the next modelling seam:

- `tier2-ec10-lift` is a solid-only disagreement: meter **9.60/s**, sim
  delivered **8.97/s**. The meter has no fluid notes and is converged; this is
  the zero-headroom/belt-distribution class, not a fluid or warmup issue.
- `pu1-lift` is the opposite direction: meter **0.778/s**, sim delivered
  **1.020/s**. The meter reports a 16-tile belt cycle and three petroleum-
  starved chemical plants. Its model overproduces sulfur (**1.286/s** versus
  sim **0.453/s**) while underfeeding plastic (**2.829/s** versus sim
  **3.66/s**), even though the sim's petroleum and target rates are at plan.

This points to a coupled meter-model problem: fluid allocation is being made
without enough awareness of solid-output/belt backpressure and cycle ordering.
Do not “fix” the gate threshold or treat this as a pure fluid-network defect.
The next code increment should add per-recipe attribution to the meter probe
(fluid supplied/consumed, crafts, and output-blocked time), then use that trace
to fix the PU1 coupling and separately trace tier2's belt-cycle loss. Keep the
meter report-only until both traces agree with the sim population.

## 7. 2026-08-19 attribution landed

The meter now emits `recipe_attribution` in `MeterReport` (and the
`spaghettio-sim run --meter` human output). Each recipe carries machine count,
crafts, working ticks, output-blocked ticks, item/fluid shortage ticks, and
fluid supplied/consumed during the measurement window. The counters reset with
the existing warmup boundary, so they are not contaminated by startup.

The two divergent fixtures were re-run through the native meter:

- **PU1:** five basic-oil-processing machines completed 3,600 crafts with no
  fluid shortage. Plastic machines recorded **154,200 fluid-shortage ticks**;
  sulfur recorded **77,100**. Neither recipe recorded output blocking. The
  meter therefore allocates the petroleum pool between competing consumers,
  rather than losing it at the refinery stage. It gives sulfur **69,450**
  petroleum units and plastic **92,600**, producing sulfur at 1.286/s while
  the sim is 0.453/s and leaving plastic at 2.829/s versus the sim's 3.66/s.
  The narrow suspect is now the meter's fluid-network allocation/topology,
  not warmup or refinery capacity.
- **Tier2:** cable machines recorded **86,400 output-blocked ticks** and EC
  machines recorded **129,600 item-shortage ticks**. No fluid path is
  involved. The next tier2 trace should follow the cable output lane and its
  downstream consumer handoff.

Tests: `cargo test -p spaghettio_meter --lib` (57 passed) and
`cargo test -p spaghettio_sim_harness` (87 passed). The meter is still
report-only; no gate or validator behavior changed.

## 8. 2026-08-19 fluid backpressure correction and tier2 trace

The recipe/output trace (`crates/meter/examples/trace_recipe.rs`) separated
the two fixtures' causes. PU1's sulfur output line reaches the sulfuric-acid
consumer, but the meter was draining unconsumed sulfuric-acid fluid even when
processing-unit consumers were solid-starved. That kept the chemical plant
working and removed the sim's downstream `full_output` propagation. Tier2's
copper-cable output, by contrast, runs west into a two-tile terminal and the
EC pickup is item-starved; it has no fluid path.

The narrow correction is now in `Factory::tick_fluids`: surplus fluid is
retained only for a producer fluid whose connected component has a same-fluid
consumer, and that fluid is included in the machine output cap. Standalone
fluid outputs and unconsumed byproducts keep the prior drain behavior. A
machine test covers fluid output counting toward the cap.

Native meter remeasurements with 108k warmup / 216k window:

- `pu1-lift`: processing-unit **0.778 → 1.018/s** (sim **1.020/s**), sulfur
  **1.286 → 0.463/s** (sim **0.453/s**); plastic is **3.803/s** (sim
  **3.66/s**). The large PU1 false accusation is closed for this trace.
- `tier2-ec10-lift`: unchanged at copper-cable **28.8/s** and electronic-
  circuit **9.6/s**. Its cable output inserters record both machine output
  blocking and deposit blocking, while EC records item-shortage ticks. The
  tier2 belt/lane correction remains open; the upstream-slot drop experiment
  was rejected because it moved cable to **24.46/s**.

Verification: `cargo test -p spaghettio_meter --lib` (58 passed),
`cargo test -p spaghettio_sim_harness` (87 passed), package checks, and
focused native replays of PU1, tier2, stress-EC30, sulfuric-acid, heavy-oil
cracking, and iron-gear. The meter remains report-only.

## 9. 2026-08-19 tier2 drop-position attribution

The tier2 trace now prints the inserter kind and per-machine belt path. The
simulator and meter agree on the topology and on which output inserters see a
full destination: the sim reports the cable output inserters at `(5,6)`,
`(20,6)`, `(23,6)`, and `(29,6)` waiting for destination space. The meter's
corresponding fast inserters are the only cable taps with meaningful deposit
blocking, while the head tap also reduces its machine to 54 crafts per
3,600-tick window; the other nine remain at 90.

The remaining difference is therefore the *in-tile* drop rule, not a missing
lane or a missing terminal. The meter currently inserts into the first free
slot of the far lane (`try_insert_anywhere`). Diagnostic variants were run
against the native meter:

| drop policy | cable/s | electronic-circuit/s |
|---|---:|---:|
| first free slot (current) | 28.80 | 9.60 |
| upstream entry / fixed early slot | 24.46 | 8.15 |
| fixed middle slot | 23.89 | 7.96 |
| downstream-most free slot | 28.20 | 9.40 |
| simulator reference | 27.00 | 8.97 |

The downstream-most variant is directionally closer, but it is still a
heuristic and the other variants demonstrate that a fixed slot is not the
game rule. All variants were reverted. The open item is now a game-accurate
drop-position model (or a simulator-side dump of the belt insertion point),
not another topology or machine-buffer sweep. No production meter behavior
changed in this round.

## 10. 2026-08-19 simulator insertion-point trace

The simulator now writes an additive `sim_state.inserter_trace` channel with
the inserter kind, held stack, arm position, pickup/drop positions, and
resolved targets. It also writes `sim_state.belt_positions`, using Factorio's
continuous transport-line positions; the legacy compressed `belts` and
`inserters` arrays are unchanged.

A short tier2 replay confirms the geometric seam. The fast cable output
inserters drop at the midpoint of their target red belt tile: for example,
the `(20,6)` inserter's drop position is the `(20,7)` belt at world `x=2.5`,
`y=-1.30078125`, while the belt center is `y=-1.5`. The belt's occupied
positions around that tap are spaced at approximately `.207, .457, .707,
.957`. Blocked inserters hold the cable in their hand, which is now directly
observable rather than inferred from status.

A midpoint-biased meter experiment with fallback slots `[2,1,3,0]` produced
25.8 cable/s, so it is too strict; it was reverted along with the prior fixed
slot experiments. The evidence supports a continuous collision-window model
around the inserter's drop point, not a discrete preferred slot. The first
probe pass was corrected during this follow-up: `get_item_insert_specification`
returns a position on the connected segment, while `LuaTransportLine`'s
`can_insert_at` consumes the local position on that line. Passing the segment
coordinate directly to the local API produced the misleading all-`no`
results in the earlier note.

A controlled straight-belt fixture now exercises the corrected domains. Its
output inserter resolves to line 2, segment position `2.5`, with
`line_length=1` and `total_segment_length=20`; the target map position is the
middle red-belt tile. Local probes at positions `0..0.875` are available
when the lane is clear, while position `1` is the downstream boundary. This
is the calibration needed to translate the game's continuous drop point into
the meter's local four-slot model. No production meter behavior changed in
this round; the next step is to encode that local projection and validate the
tier2 rate.

## 11. 2026-08-20 phase-aware drop projection

The meter now projects a belt drop's local midpoint through the target tier's
fractional belt progress before choosing one of the four discrete slots. A
candidate slot must be within half an item spacing of the requested continuous
position; otherwise the inserter stays blocked. Boundary feeds retain their
entry semantics, and the old arbitrary-slot helper is no longer used for
inserter drops.

The post-lift `tier2-ec10-lift` replay moved in the expected direction:

| drop model | copper-cable/s | electronic-circuit/s |
|---|---:|---:|
| first free slot (previous) | 28.80 | 9.60 |
| phase-aware local projection | 26.00 | 8.67 |
| simulator reference | 27.00 | 8.97 |

This closes the large optimistic bias but does not yet close the residual
under-read. A fresh simulator probe shows that the four problematic output
taps do not fail uniformly: some are more blocked in the meter and others are
less blocked. The remaining gap is therefore transport-segment distribution
through the bends/sideloads, not a single global drop-window width. Widening
the collision window to `0.75` spacing made the rate worse (25.34/s), so it
was rejected. The next refinement should model continuous positions across
the connected segment, rather than add another per-tile slot heuristic. PU1
remains at **1.017/s** processing-unit output against the simulator's
**1.020/s**, so this change did not reopen the fluid correction.

Verification: `cargo test -p spaghettio_meter --lib` (59 passed),
`cargo test -p spaghettio_sim_harness` (87 passed), plus native replays of
tier2, PU1, and stress-EC30. The meter remains report-only.

## 12. 2026-08-20 connected-line groundwork

The next refinement is now represented in the meter without changing the
calibrated rate path. Each lane has a fractional item residual that is set by
the continuous drop projection and preserved through whole-slot shifts and
tile handoffs. The network builder also assigns lane-level connected-line
components and four-slot coordinates, including the two-input sideload at the
tier2 bend. A regression test pins the coordinate step across adjacent tiles,
and a lane test pins residual preservation through a shift.

The first attempted runtime use of that residual—moving a residual item across
tile boundaries before the shared tier phase—was rejected by measurement: the
tier2 replay fell to **25.34 cable/s**, below the calibrated phase-aware result
of **26.00/s** and the fresh simulator result of **27.20/s**. It is not kept.
The retained behavior remains **26.00 cable/s / 8.67 EC/s**; PU1 remains
**1.017/s** against the simulator's **1.020/s**.

This narrows the next implementation seam further: the branch-aware transport
line needs a real continuous admission/movement rule at bends and sideloads.
The weak connected component alone is insufficient because two physically
separate lanes can share a component after they merge. No gate, validator,
commit, or push was made.

## 13. 2026-08-20 branch-aware line identity

The connected-line refinement now distinguishes a weak lane component from a
forward transport line. A sideload's two feeders share a component, but a
drop before the merge walks only its own downstream line; the other feeder is
not considered a collision at that pre-merge position. The distinction is
pinned by `merged_feeders_share_a_component_but_not_a_premerge_line`.

The refined path admission is live only for downstream-of-target collision
checks, where it is behavior-preserving on tier2 (**26.00 cable/s,
8.67 EC/s**). Applying the same collision test to the target tile itself
regressed tier2 to **24.86 cable/s**, so that remains rejected. The target
tile still needs a continuous occupancy representation rather than a
slot-level collision test. Focused meter/harness tests and the full workspace
suite remain green.

## 14. 2026-08-20 controlled belt admission physics

The simulator harness now runs a temporary, isolated express-belt fixture. It
uses `force_insert_at` to place known item positions, samples `can_insert_at`
at the local line coordinates, records detailed item positions, and destroys
the fixture before the factory measurement. This also corrected the earlier
probe-domain mistake: `get_item_insert_specification` gives a connected
segment coordinate, while `LuaTransportLine::can_insert_at` is queried in the
target line's local coordinate.

The engine results are:

| known occupancy around local `.5` | `can_insert_at(.5)` |
|---|---:|
| empty | yes |
| one item at `.4375` | yes |
| one item exactly at `.5` | no |
| items at `.375` and `.625` | no |
| items at `.25` and `.75` | yes |

This is the missing rule: a single nearby item can be displaced to enlarge a
one-sided gap, but the engine does not rearrange two-sided occupancy and does
not move an exact occupant. `Lane::try_insert_at_segment` now models that
narrow case by shifting one contiguous occupied side into the nearest free
slot. It is covered by exact-occupant, one-sided-gap, and two-sided-bracket
tests.

The first tier2 end-to-end replay remains report-only and does not yet close
the whole sim/meter gap: simulator cable was **26.95/s** in the short probe,
while the meter's electronic-circuit delivery was **8.55/s** versus simulator
**8.71/s**. The controlled physics result removes the static target-slot
assumption, but bend/sideload continuous coordinates still need validation.
No commit or push was made.

## 15. 2026-08-20 measured sideload entry geometry

The remaining tier2 loss was isolated to the belt-to-belt sideload at the
bend. The meter had been handing every side-loaded item to the target lane's
upstream slot. Factorio's measured belt geometry instead places a sideload at
one of two positions inside the target tile: 68 or 188 internal positions
from its upstream edge. The side determines which one applies. The meter now
uses those positions, while straight feeds retain their ordinary entry
handoff and the existing inner/outer curve coordinates remain in place.

The first short comparison used a 108,000-tick meter warmup but only the
simulator's default roughly 13,000-tick run. That made the simulator's
delivered rate include a different amount of buffered output and was not an
apples-to-apples steady-state comparison.

With both sides aligned at a 108,000-tick warmup, the replay reports:

| measure | simulator | meter |
|---|---:|---:|
| copper-cable/s | 27.20 | 26.763 |
| electronic-circuit/s produced | 8.94 | 8.921 |
| electronic-circuit/s delivered | 9.18* | 8.921 |

The production gap is effectively closed: the remaining difference is 0.44/s
on cable and 0.019/s on EC. The simulator's delivered value is marked with an
asterisk because its measurement window flushes buffered output; produced is
the comparable steady-state metric here. The new geometry is pinned by
`sideload_enters_at_the_measured_side_dependent_position`, and the meter
package (66 tests), sim harness (87 tests), and full workspace suite pass.
This remains local work only: no commit, PR, or push was made.

## 16. 2026-08-21 same-tick cable-bank localization

The cable bank was replayed at the same nominal tick on both models:
Factorio at tick 114,060, and the meter after 108,000 warmup ticks plus a
6,060-tick window. The four cable taps still resolve to the same inserter
coordinates and midpoint drop tiles, so the remaining seam is downstream
transport state rather than a missing endpoint.

The shared path has a distinct occupancy pattern. Mapping the simulator's
one-based lane numbers to the meter's zero-based lanes, the simulator has
approximately 4/2/2/4/2 items on the active lane through
`(20,7) → (19,7) → (19,6) → (18,6) → (17,6)`, then 4 after the sideload at
`(17,7)`. The meter has 4/4/4/4/4 before that sideload, then 3 after it.
The west terminal itself is full in both models, so terminal capacity alone
does not explain the rate gap.

A focused experiment preserved the source item's fractional residual across
the sideload, matching the existing straight-handoff behavior. It changed
the local occupancy pattern but did not improve production
(`26.7622/s` versus the baseline `26.7633/s`), so it was reverted. The next
probe should isolate the curve-to-sideload admission rule in a small fixture,
rather than add another global residual heuristic.

## 17. 2026-08-21 isolated curve+sideload probe

The harness now creates a temporary four-belt fixture outside the factory:
one express belt turns into the target from the side, while a second express
belt feeds the target from its back. The fixture is sampled at ticks 1–30 and
destroyed before the factory measurement.

Factorio's probe shows the back feeder occupying one target lane first, then
the curved feeder arriving on the other lane; by tick 30 both lanes contain a
full quarter-grid. A minimal meter `NetworkBuilder` replay of the same shape
also reaches two full lanes (by tick 24), so the lane selection and measured
68-position sideload admission are not independently wrong.

The remaining production seam is the curved lane's residence time. Factorio
measures turn lanes as 106 internal positions for the inner lane and 295 for
the outer lane, rather than the 256 positions of a straight tile
([Transport-belt physics](https://wiki.factorio.com/Transport_belts/Physics)).
The meter currently uses those values for connected-line coordinates and
collision queries, but its physical tile shift still advances every lane as a
four-slot straight tile. This explains why the production snapshot can retain
too many items before the bend and too few after the sideload even though the
isolated admission test passes. No production curve change has been made yet;
the next implementation should add turn-lane residence time without another
entry-position heuristic.

## 18. 2026-08-21 exact curve+sideload timing

The isolated fixture was rerun with a sample on every tick from 1 through 30.
Factorio admits the curved items to the target lane at ticks 13, 15, 18, and
21. The meter admits them at ticks 14, 16, 19, and 24. The first three are
within one tick; the final item exposes a separate discrete packing problem:
the meter's target lane has items in slots 1–3 and cannot use the upstream gap
until the block advances, while Factorio finishes the quarter-grid sooner.

Two focused fixes were rejected by the real replay. Changing turn-lane grids
to `ceil(106/64)` and `ceil(295/64)` reduced cable output to **24.16/s**;
allowing the discrete sideload fallback to fill the upstream gap improved the
fixture but reduced production to **25.66/s**. Both experiments were reverted.
The validated baseline remains **26.763 cable/s / 8.921 EC/s produced**. The
next implementation should model continuous occupancy through the turn and
merge as one stateful transport segment, with a regression fixture for both
the tick sequence and the full replay.

## 19. 2026-08-21 turn-boundary timing experiment

The first stateful-boundary approximation was tested and rejected. It allowed
turn edges to hand off an exit item during the bootstrap tick and immediately
after a whole-slot shift. This moved the first three items in the isolated
fixture earlier, but left the fourth item stuck behind the discrete sideload
lane; the full replay fell to **26.53 cable/s**. Restricting the shortcut to
the observed outer lane was worse at **26.01 cable/s**. The implementation was
reverted.

This confirms that the missing state cannot be attached only to the curve
edge. The turn's continuous position, the target lane's moving occupants, and
the merge admission window must advance together. The meter remains at the
validated **26.763 cable/s / 8.921 EC/s produced** baseline while that segment
model is designed.

## 20. 2026-08-21 topology-scoped merge admission

The upstream-gap behavior was retained only when a sideload source is itself
fed by a quarter-turn and the source is on the outer turn lane. In that
topology the target lane may finish a contiguous downstream block using its
remaining upstream gap; ordinary sideloads and inserter drops still use the
strict collision rule. The
behavior is pinned by `turn_merge_drop_can_finish_an_upstream_gap`.

The isolated fixture now reaches a full target lane at tick 22 instead of
tick 24. The full tier2 replay is unchanged at **26.763 cable/s / 8.921 EC/s
produced**, so this is a safe incremental correction while the full continuous
turn model remains future work.

## 21. 2026-08-21 inner/outer lane probe

The isolated fixture was rerun with only Factorio transport line 2 filled,
while the back feeder stayed on line 1. This is not equivalent to the earlier
line-1 probe: the curved item reaches the target lane by tick 5, and three
items are present there by tick 10, but the fourth item is still on the curve
at tick 30. The line-1 fixture instead arrives later and completes its
quarter-grid by tick 30.

The turn constants are therefore not just a single residence delay. The inner
and outer paths also interact differently with the target lane's sideload
admission and packing. The meter now keeps the inner turn lane strict: its
isolated replay also leaves three items on the target and one on the curve at
tick 30. Lane identity must remain explicit through the turn and merge;
applying the outer packing rule to both lanes is incorrect.

## 22. 2026-08-21 inner turn-to-merge residence

The next scoped experiment reduced only the discrete inner-lane capacity when
the turn immediately feeds a sideload merge. Using the measured inner arc
length (106 internal positions, or 1.656 item spacings), the meter retains one
effective quarter-grid slot on that arc. Ordinary turns and outer lanes are
unchanged.

The exact inner probe now admits target items at meter ticks 6, 8, and 11,
versus Factorio ticks 5, 8, and 10, while preserving the measured steady
state of three target items and one item on the curve. The corresponding
outer-lane capacity experiment was rejected: it delayed the outer path and
reduced the full replay to 25.46 cable/s.

On the real tier2 replay, the inner-only change raises the meter from
**26.763 to 26.842 cable/s** and from **8.921 to 8.948 EC/s produced**. Meter
package tests (70), sim-harness tests (87), example compilation, and
`git diff --check` remain green. This is still local work only: no commit,
push, or PR has been made.

## 23. 2026-08-21 divergence-debugging method

The remaining sim/meter difference is being investigated with a repeatable
differential workflow, now documented in `docs/sim-harness.md`:

- align warmup and measurement windows before comparing rates;
- compare produced output before buffered drain delivery;
- localize with meter recipe/inserter/belt attribution and simulator machine
  time-series, item counters, and drop traces;
- reduce the suspected interaction to a disposable isolated fixture;
- seed exact transport-line positions, fill lanes independently, and sample
  every tick with occupancy and continuous positions;
- accept a physics change only when both the fixture and the full replay
  improve, reverting fixture-only wins;
- record rejected hypotheses and measured boundaries in this handoff.

The aligned run shows why the headline rates must not be compared blindly:
the simulator converged after a 2,040-tick window and reported **27.20
cable/s**, but its three recorded checkpoint windows contained 920, 902, and
916 cable items. Averaged across those windows, the simulator is approximately
**26.84 cable/s**, matching the meter's **26.842 cable/s**; simulator EC is
**8.94/s** versus meter **8.948/s**. No simulator verdict is changed by the
meter probe.

## 24. 2026-08-21 residual occupancy at the bend

The remaining observable state difference is still at the shared cable bend,
not at machine endpoints. At the same final snapshot, the active cable lane
through `(20,7) → (19,7) → (19,6) → (18,6) → (17,6) → (17,7)` contains:

| location | Factorio | meter |
|---|---:|---:|
| `(20,7)` | 4 | 4 |
| `(19,7)` | 2 | 3 |
| `(19,6)` | 2 | 3 |
| `(18,6)` | 4 | 4 |
| `(17,6)` | 2 | 1 |
| `(17,7)` after the sideload | 4 | 3 |

So the meter is still distributing two items differently through the quarter
turn and merge: it retains one extra item before the turn at two positions,
then one fewer item on the turn and after the sideload. That state mismatch is
real, but it does not currently represent a throughput mismatch once the
simulator's short-window variance is averaged. The next useful work is either
a longer fixed-window simulator capture or a continuous outer-turn/merge
model; another global capacity heuristic is not justified by the evidence.

## 25. 2026-08-21 fixed-window simulator capture

The harness now has an opt-in `--fixed-window --window N` diagnostic mode. It
disables early convergence and closes one exact post-warmup window; ordinary
runs remain item-driven and convergence-based. The tier2 fixture was rerun
with warmup **108,000** and window **216,000** ticks.

The full-window comparison is:

| measure | Factorio | meter |
|---|---:|---:|
| copper-cable/s | 27.12 | 26.842 |
| electronic-circuit/s | 9.04 | 8.948 |

This confirms a real residual after removing the short-window artifact. It is
not a uniform belt-speed loss: the per-machine craft counts are redistributed
between branches. For cable machines (Factorio coordinates are one tile east
of meter coordinates), Factorio vs meter counts are:

| Factorio x / meter x | 5/4 | 8/7 | 11/10 | 14/13 | 17/16 | 20/19 | 23/22 | 26/25 | 29/28 | 32/31 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| crafts | 4502/2557 | 4804/5400 | 5400/5400 | 5400/5400 | 5400/5400 | 3008/2842 | 4105/5116 | 5400/5400 | 5400/5400 | 5400/5400 |

The EC bank shows the same redistribution: Factorio produces fewer crafts
at meter-equivalent x=4 (3000 vs 4263), then more at x=7–16, while the two
downstream machines at x=19 and x=22 are full in both models. The next probe
should compare the EC input-belt lanes and inserter admission events for those
branches. The evidence now points to branch-level transport distribution,
not warmup, fluid supply, or a global belt-rate constant.

## 26. 2026-08-21 post-lift recipe sweep

The native meter was replayed against the other six post-lift fixtures with
the same 108,000-tick warmup and 216,000-tick window. The target results are:

| fixture | target | meter/s | planned/s | meter % |
|---|---|---:|---:|---:|
| `ac5-lift` | advanced-circuit | 4.845 | 5.0 | 96.9% |
| `bigpole1-lift` | big-electric-pole | 1.000 | 1.0 | 100.0% |
| `bigpole1-lift-v2` | big-electric-pole | 1.005 | 1.0 | 100.5% |
| `pu1-lift` | processing-unit | 1.018 | 1.0 | 101.8% |
| `stress_ec_30s_postlift` | electronic-circuit | 27.500 | 30.0 | 91.7% |
| `stress_ec_60s_red_postlift` | electronic-circuit | 52.468 | 60.0 | 87.4% |

The pole and PU layouts remain close to their existing simulator baselines.
Advanced-circuit and the two stress layouts retain meaningful residuals, so
they are the next candidates for fixed-window Factorio captures. The inner
turn-to-merge change was A/B checked on `ac5-lift`: disabling it produced
4.833/s, so it is not the cause of the advanced-circuit shortfall and remains
retained for the tier2 improvement.

## 27. 2026-08-21 red high-rate follow-up

The red fixture was traced at 108,000 ticks of warmup and a 216,000-tick
measurement window. Its EC machines are not uniformly slow: both rows show
the same spatial pattern, with machines around x=34–40 ingredient-starved and
the final machines around x=82–85 heavily output-blocked, while the sections
between them run at the full 5,400 crafts. This is the signature of repeated
shared-belt admission/backpressure loss, not a global recipe or belt-rate
constant.

The unresolved 10-tile belt cycle was tested with its update order reversed.
Individual machine counts moved, but aggregate EC output stayed effectively
unchanged: **52.468/s** in the normal order versus **about 52.45/s** reversed.
Cycle order is therefore a useful uncertainty note, not the red fixture's
main improvement lever. The temporary diagnostic hook was reverted.

A matching fixed-window Factorio capture was attempted with the red fixture,
but the 166×129 layout remained CPU-bound for the full 1,200-second harness
timeout and produced no report. The existing sim baseline (**90.67% delivered**)
remains the reference; this attempt is recorded as a fixture-scale runtime
limit, not as a measurement.

Next red work should target the shared ingredient/output belt admission model,
using a smaller extracted belt-cycle fixture or a shorter Factorio probe to
avoid spending twenty minutes per iteration.

## 28. 2026-08-21 red reduction probes

The first reduction is now checked in as two meter examples:
`cycle_probe` extracts the actual ten-tile red cycle, and `red_row_probe`
keeps the six EC machines in each row around x=25–40 with their input and
output belts. The cycle is **not connected to a belt feeder or inserter drop**
in the decoded red layout, so it is not the EC deficit's source. Midpoint
admission sweeps at local positions 0.25, 0.50, and 0.75 accepted essentially
the same number of drops (about 902 per feeder per 3,600 ticks).

The row fragment is a useful negative control: with saturated cable and iron
feeds it produces **18.0 EC/s**, all 12 machines working. Therefore the loss
does not arise from the local straight row alone. Widening the crop to include
the downstream machines without their upstream producer branch leaves the
machines beyond x=40 unfed, which identifies the next reduction boundary:
preserve the source branch/tap that supplies the later row segment, rather
than adding more isolated belt tiles.

Commands:

```text
cargo run --release -p spaghettio_meter --example cycle_probe -- \
  /path/to/stress_ec_60s_red_postlift/bp.txt 3600 0.5
cargo run --release -p spaghettio_meter --example red_row_probe -- \
  /path/to/stress_ec_60s_red_postlift/bp.txt 108000 216000
```

## 29. 2026-08-21 branch-preserving red fixture

`red_branch_fixture` now performs the real reduction: it starts from all 40
EC machines, retains their upstream belt closures, keeps producer machines
whose output inserters land on those closures, recursively keeps the cable,
copper-plate, and iron-plate input branches, and walks the EC outputs forward
to the declared sinks. Splitter partners and weakly connected belt branches
are retained as well. The resulting fixture contains **3,581 belt tiles, 340
machines, 720 inserters, and 4,617 entities**—large enough to preserve the
branch geometry, but runnable in a short diagnostic window.

With a 3,600-tick warmup and 3,600-tick window it reproduces repeated spatial
loss across both EC rows. The current phase-aware drop rule delivered **11.22
EC/s**; a temporary legacy first-free-slot A/B delivered **11.45 EC/s**. That
2% short-window movement is not a decisive physics result, so the hook was
reverted and no production drop rule changed. The fixture is therefore ready
for the next A/B probe, while the full 108k/216k run remains an expensive
confirmation rather than the iteration default.

Fast command:

```text
cargo run --release -p spaghettio_meter --example red_branch_fixture -- \
  /path/to/stress_ec_60s_red_postlift/bp.txt 3600 3600
```

## 30. 2026-08-21 red output-bank isolation

The branch fixture's short-run shortage was split from output behavior with
`red_output_bank_probe`. It retains **all 3,581 belt tiles** from the real
blueprint, removes every machine except the 40 EC assemblers, retains their
120 I/O inserters, and adds 80 saturated synthetic ingredient feeds at the EC
input pickup tiles. The ingredient is inferred from the real upstream branch
when the one-way graph exposes it, with the known red-bank lane geometry as a
fallback for side-loaded inputs.

The long run (108,000-tick warmup, 216,000-tick window) produced **57.14
EC/s** and delivered **57.1422 EC/s**. All 40 machines remained working and
there were zero item-ingredient-shortage ticks. The remaining loss was
output-side admission/backpressure (411,708 machine output-blocked ticks and
558,486 output-inserter blocked ticks), plus the existing ten-tile cycle-order
note. Therefore the 52.468 EC/s full-layout result is not explained by the EC
output bank alone: the remaining ~4.67/s is upstream of the EC input pickup
tiles, in the cable/plate branch distribution and admission path.

This is a useful isolation technique: preserve one side's complete geometry,
replace the other side with saturated local feeds, and compare both machine
shortage and output-blocked counters. A saturated boundary's high refusal
count is expected and is not itself a factory shortage.

Command:

```text
cargo run --release -p spaghettio_meter --example red_output_bank_probe -- \
  /path/to/stress_ec_60s_red_postlift/bp.txt 108000 216000
```

## 31. 2026-08-21 red upstream-supply isolation

The complementary `red_input_supply_probe` removes the EC bank while
retaining all raw-material, copper-cable, and plate machines, their inserters,
and the complete belt network. The 80 EC input pickup tiles become declared
sinks, so delivery is measured before an EC input buffer or output belt can
hide the source rate.

At 30,000 ticks of warmup and window it delivered **178.278 copper-cable/s**
and **59.642 iron-plate/s** to those pickup tiles. The full-layout result of
52.468 EC/s needs only about **157.404 cable/s** and **52.468 iron-plate/s**.
The aggregate upstream branches therefore have enough material; the residual
loss is spatial distribution/admission into particular EC input lanes, not
total cable or plate production. This also explains why the direct-feed
output-bank probe reaches 57.14 EC/s while the full layout is 52.468/s.

The next physics A/B should preserve the real producer/tap geometry while
instrumenting or replacing only the admission at the affected EC input lanes.
Changing aggregate source rates would mask the actual failure mode.

Command:

```text
cargo run --release -p spaghettio_meter --example red_input_supply_probe -- \
  /path/to/stress_ec_60s_red_postlift/bp.txt 30000 30000
```

## 32. 2026-08-21 red per-lane admission A/Bs

`red_input_lane_probe` reports each EC input inserter over a matched
30,000-tick warmup/window, subtracting each inserter's lifetime counters at
the warmup boundary. The normal run delivered 52.430 EC/s and showed a
repeatable head-to-tail gradient on both rows: x=25–31 received most of the
material, x=37–40 received almost none. The input-supply probe's branch-end
drains show the corresponding cable supply per row at approximately 14.9,
29.6, 29.5, and 15.0/s across the four physical sections. This is a spatial
distribution problem, not an aggregate source shortage.

Three diagnostic A/Bs were run and reverted or kept example-local:

- Reversing all inserter iteration order produced an identical 52.430 EC/s
  and identical per-lane counts. General entity scheduling is not the cause.
- Relaxing belt-to-belt sideload admission changed the result only to 52.414
  EC/s. The sideload collision projection is not the dominant cause.
- Replacing producer inserter drops with first-free-slot admission raised the
  result to 53.144 EC/s, about 1.4%, but left the same tail starvation. It is
  a small contributor, not the missing 7.5/s.
- Reducing only EC ingredient buffers from 14 crafts to 2 changed throughput
  to 52.632 EC/s and left the gradient intact. Buffer hoarding is not the
  dominant cause.

The next investigation should follow the four cable branch/splitter paths
and compare their lane rates against the game capture. A broad global physics
change is not justified by these A/Bs; the evidence now points to a specific
splitter/tap distribution or lane-routing rule.

Command:

```text
cargo run --release -p spaghettio_meter --example red_input_lane_probe -- \
  /path/to/stress_ec_60s_red_postlift 30000 30000
```

## 33. 2026-08-21 cable topology isolation

`red_cable_distribution_probe` removes every machine and inserter, saturates
all 60 real copper-cable output taps, and drains the 40 EC cable pickup
endpoints while retaining all 3,557 belt/underground/splitter entities. The
network delivers the full **180 cable/s aggregate**, but its four branch ends
carry exactly **15, 30, 30, and 15 cable/s per row**. The first six-EC section
therefore receives one red-belt lane (15/s) while later sections receive two
lanes. This reproduces the spatial deficit without producer timing or EC
consumption, and explains the head-to-tail starvation profile directly.

The decoded blueprint contains 24 ordinary splitters and no input/output
priority or filter fields. A diagnostic splitter lane-swap A/B also produced
the identical 15/30/30/15 result, so simple lane-preservation is not the
cause. The remaining question is whether Factorio's splitter network actually
has this branch capacity distribution under load, or whether its splitter
update/admission semantics redistribute the outer-lane surplus. That is now a
game-physics comparison at named branch endpoints, not a broad meter constant.

Command:

```text
cargo run --release -p spaghettio_meter --example red_cable_distribution_probe -- \
  /path/to/stress_ec_60s_red_postlift/bp.txt 3600 3600
```

## 34. 2026-08-21 fixed-window sim drop telemetry

The first full red telemetry run reached the live timeseries but timed out
before finalization: fixed-window mode still used the fixture's larger derived
ceiling, and `sim_state` was previously written only by `finalize`. The
harness now writes one `sim-state.json` snapshot at the first closed fixed
window. This is diagnostic-only; ordinary runs still write their state at
finalization.

Using a 3,600-tick warmup and 3,600-tick window on the red fixture produced a
complete checkpoint snapshot before stopping the CPU-bound tail. It contained
7,746 detailed belt-position records, 370 drop-event inserters, and 24
splitters. On the 30 copper-cable producer taps resolved on the affected
upstream paths, the game accepted 1,409, 1,356, and 1,144 items on the three
observed rows (23.48/s, 22.60/s, and 19.07/s), with 2,664, 2,606, and 1,818
destination-blocked events respectively. The counters are complete even when
the per-event sample list reaches its 512-record forensic cap.

This confirms that the new telemetry can expose producer-side admission and
the spatial gradient. It does not close the branch comparison: the current
trace has no per-EC-inserter belt-pickup delivery counter or time-series of
branch-end flow. That remains the smallest useful telemetry addition if the
game-vs-meter comparison must be made at the four 15/30/30/15 branch ends.

The next telemetry iteration adds that missing `pickup_event_trace` channel:
for each belt-to-machine inserter it counts held-stack rises as belt pickups
and falls as machine deliveries, recording the resolved machine recipe. This
is still report-only and does not alter the simulation.

The item-aware follow-up capture records 420 copper-cable and 371 iron-plate
items delivered into the EC bank during the 3,600-tick diagnostic window.
Cable delivery falls along each row (for example 27, 22, 11, 3, 1 at the
first branch). This reproduces the meter's spatial starvation signature, but
the short warmup is intentionally not used as a steady-state rate comparison;
the full red fixture remains too CPU-bound to reach a 108,000-tick warmup
before the harness timeout.

## 35. 2026-08-22 splitter lane independence and blocked-side memory

The game comparison narrowed the red gap to splitter scheduling. The isolated
meter topology delivered all 180 cable/s in aggregate, but its four branch
ends were fixed at 15/30/30/15 cable/s per row. Factorio's splitter behavior
has two relevant rules: the two input lanes make independent output decisions,
and a blocked output retains bounded memory for up to five forced-away items.
These are recorded as S10/S11 in `docs/factorio-mechanics.md`, with the
[Factorio Wiki splitter history](https://wiki.factorio.com/Splitter) as the
source.

The meter now models one round-robin state and one five-item blocked-side
counter per input lane. A focused network test covers a blocked output,
fallback routing, and the subsequent remembered return. The isolated probe
changed the branch pattern to 20/25/25/20 per row, which is consistent with
the game capture's less rigid branch distribution.

On the matched 108,000-tick warmup and 216,000-tick window, the red meter
improved from the prior 52.468 EC/s baseline to **53.203 EC/s delivered**
(53.232 EC/s produced). This is a real improvement, but it does not yet erase
the remaining difference from the game's roughly 53.6--54.0 EC/s result; the
remaining work is still branch-level inserter admission/distribution, not a
global belt-rate correction.

The seven-fixture post-lift sweep remained healthy: all six comparable target
rows stayed within the prior calibration envelope, with zero missed defects at
the 90% and 95% thresholds. The current produced target range is 89.47--
101.94% of plan; delivered is 88.67--101.94%. The red fixture's meter target
is now 88.72% produced / 88.67% delivered versus the simulator's 89.83% /
90.67%, so the physics correction closes part of the gap without introducing
a new optimistic failure.

Validation commands/results:

```text
cargo test -q -p spaghettio_meter                 # 73 unit + integration tests passed
cargo test -q -p spaghettio_sim_harness          # 88 tests passed
cargo check -q -p spaghettio_meter --examples    # passed
cargo test -q -p spaghettio_meter --test corpus_replay  # 2 passed, 2 ignored
target/release/examples/sweep_postlift ...       # 6/6 comparable, no 90/95% misses
```

This is a suitable PR seam once the final diff is cleaned: shared telemetry,
parity diagnostics, the validated splitter rule, focused regression coverage,
and the handoff update. The raw probe binaries and temporary captures should
remain local unless a follow-up makes one of them a supported diagnostic.

## 36. 2026-08-22 matched long game profile and remaining branch seam

A matched Factorio run was repeated with the same **108,000-tick warmup** and
**216,000-tick measurement window**, using `--pickup-trace-only` to remove the
unrelated per-tick drop forensics. It produced **54.01 EC/s delivered** versus
the current meter's **53.20 EC/s**, but the hard ceiling yielded only two
stability checkpoints, so the harness correctly marked it non-converged. The
aggregate is therefore directional, not a replacement steady-state baseline.

The per-machine profile is more useful than the aggregate. Coordinates below
are game coordinates; the meter's corresponding coordinates are one tile west.
Both runs still show the same two shortage valleys, but they do not place the
loss at exactly the same branch:

- In the upper row, Factorio's middle valley is at game x=41 (5.56 crafts/s)
  while the meter's corresponding x=40 is lower (1.32/s); Factorio's later
  valley at x=60 is 5.84/s while the meter's corresponding x=59 is 10.48/s.
- In the lower row, the first valley is close (5.58 versus 6.27/s), while the
  later valley again differs (6.25 versus 10.74/s).
- At the final machines, Factorio remains around 13.3 crafts/s and the meter
  is around 11.2--11.7/s.

This rules out a remaining global belt-speed or recipe-rate correction. The
next physics seam is the **timing and state transition of the splitter branch
network**, especially how a remembered blocked side expires while downstream
pickup demand is asymmetric. The current five-item memory correction remains
the validated PR candidate; this profile is follow-up evidence, not grounds
for another unanchored A/B.

Command and artifact:

```text
cargo run --release -p spaghettio_sim_harness --bin spaghettio-sim -- run \
  --bp /path/to/stress_ec_60s_red_postlift/bp.txt \
  --manifest /path/to/stress_ec_60s_red_postlift/manifest-real.json \
  --ticks 324000 --warmup 108000 --window 216000 --fixed-window \
  --speed 32 --pickup-trace-only --out /tmp/red-followup-pickup-long.json
```

## 37. 2026-08-22 pickup telemetry serialization and startup race

The first pickup-only captures appeared to contain no trace records. The
sampling loop was resolving the belt-to-machine inserter population before
Factorio had attached blueprint targets, then caching the empty result for the
rest of the run. In addition, both live trace maps were keyed by sparse numeric
unit numbers, which Factorio's JSON helper serialized as `{}`.

The harness now retries an empty population until targets resolve and converts
both traces to stable arrays sorted by inserter unit number before writing
`sim-state.json`. The short red verification capture found **380** pickup
inserters and emitted 380 records with complete picked/delivered counters and
bounded event lists. This is diagnostic-only and does not change the sim
measurement. The regression is covered by the harness generation test and the
behavior is documented in `docs/sim-harness.md`.

Verification command:

```text
cargo test -q -p spaghettio_sim_harness
```

## 38. 2026-08-22 tickwise pickup validation

The first windowed pickup counters were still low because the 60-tick sample
cadence skipped fast inserter hand cycles. In pickup-only mode the sampler now
runs every tick; ordinary runs retain the cheaper cadence. A fresh 30,000-tick
warmup/window capture produced 380 pickup records and populated the warmup-
reset per-inserter/per-item counters.

As an internal conservation check, the cable pickup rate for every one of the
40 EC machines was divided by the recipe's three-cable input and compared with
the game's per-machine craft delta over the same 500-second window. The
maximum absolute difference was **0.0033 crafts/s**. This validates the
telemetry path independently of the meter and makes it suitable for the next
branch-level sim/meter comparison.

## 39. 2026-08-22 splitter-memory sensitivity check

The tickwise pickup comparison gives a stable baseline for testing the
remaining splitter seam. The blocked-side memory count was varied in the
isolated meter while keeping the real red geometry, producer timing, and
30,000-tick warmup/window unchanged. Delivered electronic-circuit rates were:

| remembered return items | delivered EC/s |
| ---: | ---: |
| 4 | 52.414 |
| 5 | 53.148 |
| 6 | 53.256 |

The five-item value is the documented Factorio rule and remains the selected
implementation. Six items is a small aggregate improvement, but it is not
evidence for changing the rule: it is an unanchored fit to one fixture and
would contradict the current mechanics contract. Four items clearly regresses
the result. The experiment therefore narrows the residual to the timing and
expiry details around remembered splitter routing; it does not justify a new
global rate or memory constant.

Artifacts:

```text
/tmp/red-meter-memory4-repeat-30000.txt
/tmp/red-meter-memory5-repeat-30000.txt
/tmp/red-meter-memory6-repeat-30000.txt
```

## 40. 2026-08-22 splitter admission telemetry and order A/B

The meter now exposes report-only `SplitterStats` for each splitter. The
counters are reset with the measurement window and record input attempts,
first-side rejection, fallback admission, remembered returns, memory expiry,
and both-side rejection. They do not affect movement decisions.

On the full red factory's 30,000-tick warmup/window, the largest repeated
both-side rejection counts were at splitter positions `(12,74)`, `(12,77)`,
`(12,84)`, `(12,86)`, `(10,74)`, `(9,85)`, `(8,86)`, and `(7,83)`. This gives
the next investigation concrete router locations instead of treating the
whole network as one black box.

Two update-order hypotheses were tested and rejected:

- Reversing the update order of every splitter's two occupied halves raised
  the 30,000-tick result to **57.258 EC/s**, overproducing the game's roughly
  54 EC/s profile.
- Deferring both splitter halves until the later half's order position, so the
  pair acted atomically, produced **48.642 EC/s** in a 10,000-tick
  warmup/window versus **48.732 EC/s** for the current order.

The result is not “order does not matter”: the large A/B swing proves that it
matters. It means the remaining rule is more specific than a global half
reversal or a fully atomic pair update. The next useful experiment should
target the identified branch splitters and compare their per-lane admission
timing against the game pickup trace.

The game-facing baseline remains that a splitter is a 2×1 entity with up to
two inputs and two outputs, preserves lanes, and has bounded blocked-output
memory; see the [Factorio transport-network mechanics](https://wiki.factorio.com/Transport_network)
and the [splitter history](https://wiki.factorio.com/Splitter#History).

## 41. 2026-08-22 per-input blocked-memory candidate

The per-half telemetry showed that the major both-side rejection hotspots were
localized to one physical input half of each splitter. This led to a narrow
state-isolation A/B: keep the round-robin toggle shared per splitter lane, but
keep blocked-output memory separately for each physical input half and lane.

On the matched red 30,000-tick warmup/window, compared with the captured game
machine profile, the variants were:

| state model | delivered EC/s | profile MAE | profile RMSE |
| --- | ---: | ---: | ---: |
| shared toggle + shared memory | 53.148 | 0.1097 | 0.1906 |
| per-half toggle + shared memory | 53.228 | 0.0862 | 0.1559 |
| shared toggle + per-half memory | **53.308** | **0.0712** | **0.1330** |
| per-half toggle + per-half memory | 53.432 | 0.0919 | 0.1609 |

The memory-only variant is retained: it improves both aggregate throughput and
the spatial game profile, while the toggle change is not retained. The
post-lift safety sweep remained healthy with **6/6 comparable targets**, zero
missed defects at the 90% and 95% thresholds, and worst optimistic error of
only **+1.44 percentage points** on produced output.

This is a stronger candidate than the earlier global-order experiments, but it
still does not close the red gap. The next comparison should repeat the
per-half pickup profile over a longer game window and verify that the same
physical input half owns the starvation valley; only then should this become a
final PR physics claim.

Artifacts:

```text
/tmp/red-splitter-half-memory-only-30000.txt
/tmp/red-splitter-half-toggle-only-30000.txt
/tmp/red-full-splitter-half-state-30000.txt
/tmp/postlift-splitter-half-memory.csv
```

## 42. 2026-08-22 long-window validation of per-half memory

The retained shared-toggle/per-half-memory candidate was rerun with the same
long budget as the independent game capture: **108,000 ticks of warmup and
216,000 ticks of measurement**. It delivered **53.406 EC/s**, compared with
**54.009 EC/s** in the game's long pickup-only capture. The previous
shared-memory meter run delivered **53.203 EC/s**, so the correction retains a
real **+0.203 EC/s** improvement at the long window.

The per-machine comparison also remains better: MAE is **0.0786 crafts/s**
(RMSE **0.1463**) against the long game profile, versus **0.1102** MAE
(RMSE **0.1867**) for the previous state model. The largest remaining meter
shortages are at x=37 and x=40 in both rows; a later x=59 machine is slightly
overfed. Their upstream splitter sets overlap the previously identified
hotspots, so the residual is still localized to branch admission rather than
recipe speed, belt rate, or global splitter capacity.

Artifact:

```text
/tmp/red-splitter-half-memory-only-long.txt
```

## 43. 2026-08-22 PR-hardening pass

The retained candidate now has a focused regression for the state boundary
that distinguishes it from the rejected shared-memory model:
`splitter_blocked_memory_is_isolated_between_physical_halves`. The test seeds
blocked-output memory on one physical half, changes the shared round-robin
toggle, and verifies that the other half follows the toggle without inheriting
the first half's remembered output.

Validation completed from the meter worktree:

```text
CARGO_TARGET_DIR=/tmp/fucktorio-meter-target cargo test -q -p spaghettio_meter
CARGO_TARGET_DIR=/tmp/fucktorio-meter-target cargo test -q -p spaghettio_sim_harness
CARGO_TARGET_DIR=/tmp/fucktorio-meter-target cargo check -q -p spaghettio_meter --examples
git diff --check
```

All of these pass. The sim-harness test requires localhost ephemeral-port
binding; it passed when run with that permission enabled. Repository-wide
`cargo fmt --check` still reports unrelated pre-existing formatting drift in
other workspace crates, so the PR pass did not reformat files outside this
meter/harness change.

The report-only `meter_probe` remains part of the reviewable harness change.
The branch-specific red probes remain local investigation artifacts and are
not part of the PR scope; their results are captured above rather than used as
runtime behavior.
