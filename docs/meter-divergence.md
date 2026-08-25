# Meter divergence log (#570, RFC-054 Phase B/C)

Running record of where the fast meter's `produced_per_s`/`delivered_per_s`
diverges from the measured headless-Factorio sim by more than ±10pp, and why
that divergence is believed to live (model gap vs. known-open-item). Updated
when either sweep — `crates/meter/examples/sweep_corpus.rs` (Job-2 bank) or
`crates/meter/examples/sweep_postlift.rs` (post-lift layouts) — moves a number
or reveals a new one.

**⚠ 2026-08-25: a third, broader hole — the turn-path under-read class
(see the section at the head of this log). "Below plan ⇒ believe it" is
falsified for turn-heavy fixtures until the meter's turn model is fixed;
sim-anchor before deciding on any below-plan meter reading there.**

**Two open residuals, and they are different animals:**

1. `tier5_processing_unit_from_ore_am3`, −13.6% on the corpus (sim-relative,
   **on produced** — see the units note in the 2026-08-08 section; this log has
   historically written `sweep_corpus`'s percentages as "pp"). Diagnosis
   closed 2026-08-06: a research-productivity parity gap. Its fix is **merged**
   (#584 meter / #585 sim / #587 solver / #591 `sim_export` wiring) — what
   remains is that **the 2026-08-01 corpus predates the axis and declares
   none**, so this row still reads at the old value until the bank is
   re-exported or re-declared. Not a modelling mystery; a stale reference.
2. `pu1-lift`, **−22.7% sim-relative on produced** (the metric-matched figure
   against item 1; −23.7% on delivered, −24.2pp of plan), found 2026-08-08 on the post-lift population and
   **not** the same cause — that fixture declares the axis and its sim run is
   kit-clean. A petroleum-gas distribution shortfall inside the meter's own
   fluid network. See the 2026-08-08 section, which is the current head of this
   log.



## 2026-08-25 — turn-path under-read: BELOW PLAN IS NOT BELIEVABLE on turn-heavy fixtures

Found during RFC-072 Phase-0/1 (the composition probes): the meter
manufactured a "seam cost" on composed two-stage fixtures that headless
Factorio does not exhibit. Four fixtures adjudicated against the sim
(exports under the RFC-072 evidence set; sims converged, drift ≤ +1.8%):

| fixture | topology | meter delivered | sim verdict | divergence |
|---|---|---:|---|---|
| `dis-ec20-comp` | 2-row, trunk turns | 94.6% | produced +0.0%, PASS | **−5.4pp phantom** |
| `seam-ec30-comp` | 2-row, trunk turns | 81.6% | produced +0.0%, PASS | **−18.4pp phantom** |
| `dis-ec15-comp` | single straight row, no turns | 99.7% | (not simmed) | consistent |
| `seam-cable90` | capacity-bound (90/s target, one output belt) | 50.0% | delivered −50.2%, FAIL | **accurate** |

The under-read is **geometry-correlated**: the one straight-line fixture
meters clean, every turn-path fixture under-reads 5–18pp, and the
capacity-bound failure is measured accurately — so the suspect is the
turn-path flow model (per-lane geometry through turns interacting with
mid-run inserter drops), not belt-capacity math and not a global
inserter-capacity mismatch (a global cause would have hit the straight
fixture too). The meter's own counters blame drops
(`output_inserter_blocked` ≈ 27% of machine-ticks on the phantom
fixtures) that the real game does not exhibit (sim census: all machines
working).

**Consequence for the calibrated asymmetry: "meter says below plan ⇒
believe it" is FALSIFIED for turn-heavy fixtures.** Until the turn model
is fixed and re-validated, a below-plan meter reading on a layout whose
item paths turn (which is most bus layouts) refutes nothing by itself —
it must be sim-anchored before any decision rides on it. The four
fixtures above are the ready-made calibration set for the fix; the
bank's own rows are unaffected (they were sim-anchored directly).
Instrument used for the forensics:
`crates/meter/examples/lane_heatmap.rs` (whole-map per-lane occupancy +
splitter counters + RECT path dump). Follow-up: the meter-side turn
model fix, tracked from RFC-072's decision log.

## 2026-08-08 — post-lift calibration: the FLOOR PROPERTY DOES NOT HOLD on the gate population

**What this answers.** The 2026-08-07 section below established the meter as a
safe floor across 41 corpus rows and then flagged, in its own "two things this
does NOT establish", that the result was a property of *pre-lift* layouts and
that a gate would run on *post-lift* ones. This is that measurement. **The
caveat was right and the floor property does not survive it.**

**On novelty, plainly:** the tier2 optimism was already known and already
written down — the 2026-08-07 section records "meter 96% vs sim 90–91%" and
calls it unexplained. What this section adds is not the observation but its
*status*: measured from the banked blueprint, classified against gate
tolerances on the metric the harness actually verdicts on, and set against the
corpus bounds in the corpus's own units. So this is a **known caveat promoted
to a falsification**, not a new discovery — and the retraction is warranted on
exactly that basis, because the corpus claim was universal ("0 rows at 99%,
98%, 95% and 90%") and one classified counterexample is what a universal claim
costs.

### Provenance

Six post-lift fixtures, i.e. layouts selected by the ranking that #605 produced
by lifting the `input-rate-delivery` exemption. Sim baselines are the runs made
2026-08-07 on those exact blueprints — Factorio 2.0.77, `--speed 32`, **warmup
432 000** on every one, all `converged: true` and empty
`kit_errors`/`fluid_errors`. Meter side: `sweep_postlift`, the same 108k/216k
window `check_one` and `sweep_corpus` use.

These claims are now **checked by the tool rather than asserted here**, split
by what the failure means:

- **Gated** (a row that fails is excluded, with its reason printed) —
  `kit_errors`, `fluid_errors`, `converged`, the report schema, a
  `label`↔directory binding so a misfiled report cannot pair meter(layout A)
  with sim(layout B), and the **432 000-tick warmup**. Warmup gates because a
  short one makes the reading *invalid*: `CLAUDE.md` says it reads buffer fill
  as throughput and must not be quoted.
- **Reported, not gated** — checkpoint depth. That is a *strength* gradient
  rather than a validity bar: a class-5c row is weak but real, and gating it
  would either discriminate nothing (at ≥4) or delete five of six rows (at ≥5).

The sweep prints the per-fixture provenance table it checked.

**Checkpoint depth — a limitation, not a credential.** An earlier revision of
this section cited "≥4 checkpoints" as provenance. That is meaningless: the
harness's `MIN_CHECKPOINTS` *is* 4 (`scenario.rs`,
`STABILITY_WINDOWS + 1`), so every `converged: true` run has **at least** 4 by
construction — runs continue to a tick ceiling, which is why `tier2-ec10-lift`
has 8. Stated properly, **5 of the 6 rows sit at exactly the minimum** —
`ac5-lift`, `bigpole1-lift-v2`, `pu1-lift`, and both stress-EC rows — which is
precisely the "converged at the minimum" case `sim-harness-forensics.md`
class 5c says needs a longer-warmup confirmation before it is trusted as the
asymptote. Those five are provisional in that specific sense.

The exception matters: **`tier2-ec10-lift` has 8**, and it is the row the whole
floor retraction rests on. So the load-bearing measurement is the
best-provenanced one in the bank, while the rows carrying the class-5c caution
are the supporting cast. `sweep_postlift` now prints the per-fixture count and
flags the minimums instead of gating on a threshold that discriminates nothing.

**All seven fixture dirs carry a banked `report.json`** (7/7), and every one is
re-vetted, so no row here bypasses the tool's checks; the excluded
`bigpole1-lift` row is that vetting firing. The reports come from a mix of
`sim-live.sh` and `run --out` invocations, which write the same schema and are
vetted identically. The bank is a local, out-of-repo directory at
`~/spaghettio-corpora/postlift-2026-08-07/`, recovered from session scratch
before it aged out — like the Job-2 corpus it sits beside, it is **not
version-controlled**, so reproducing this on another machine means rebuilding
the bank from `sim_export` plus fresh sim runs.

**One fixture is excluded and it is worth naming.** `bigpole1-lift` reports
`kit_errors: ["research-productivity parity: 'steel-plate' realized 0.1 but the
manifest declares 0"]` — the parity check working as designed. Its rates are
inflated ~10% across the board (target 109.9% of plan) and are **not
comparable**. `bigpole1-lift-v2` is the re-export that declares
`steel-plate=0.1`, and it is the row used below. The sweep prints excluded
fixtures with their reason rather than dropping them silently.

### The six rows

**Measured on both rates, because they disagree.** `sweep_corpus` compares
**produced** for solid targets, and matching it keeps the two sweeps
commensurable. But the sim harness verdicts a solid target on **delivered**
(`crates/sim-harness/src/report.rs`, `verdict` for `!is_fluid_target`), so a
gate mirroring that verdict thresholds on delivered — grading the meter on
produced would grade it against a number no gate consults. `sweep_postlift`
emits both, and the conclusions below are stated on delivered with produced
kept as the calibration view.

**The difference comes from the sim side, not the meter side.** What changes
between the two metrics is the **reference**: the sim delivers 89.70% on
`tier2-ec10-lift` where it produces 90.91%, and 102.01% on `pu1-lift` where it
produces 100.67%. The meter's own two readings move by at most 0.01pp anywhere
in the bank. So "delivered is worse" is a statement about which sim column the
meter is held against, not about the meter having a distinct delivered model.

*(The meter's produced≈delivered agreement is weak evidence on its own — for a
target item the delivered figure largely mirrors crafts through the sink, so
near-equality is close to definitional. It is quoted only to locate where the
metric difference originates, not as a finding.)*

All figures below are **% of plan**; the meter and sim columns are given
separately per rate, because conflating them is how the first draft of this
section went wrong.

| fixture | target | meter prod | sim prod | Δpp prod | meter deliv | sim deliv | **Δpp deliv** |
|---|---|---:|---:|---:|---:|---:|---:|
| `bigpole1-lift-v2` | big-electric-pole | 100.53 | 100.67 | −0.14 | 100.53 | 102.01 | **−1.49** |
| `ac5-lift` | advanced-circuit | 99.23 | 99.67 | −0.44 | 99.22 | 99.67 | **−0.45** |
| `stress_ec_30s_postlift` | electronic-circuit | 91.92 | 90.91 | +1.01 | 91.92 | 92.12 | **−0.20** |
| `stress_ec_60s_red_postlift` | electronic-circuit | 87.90 | 89.83 | −1.93 | 87.90 | 90.67 | **−2.77** |
| `tier2-ec10-lift` | electronic-circuit | 96.00 | 90.91 | +5.09 | 96.00 | 89.70 | **+6.30** |
| `pu1-lift` | processing-unit | 77.81 | 100.67 | −22.87 | 77.81 | 102.01 | **−24.21** |

On delivered: mean −3.80pp, worst optimistic **+6.30pp**, worst pessimistic
**−24.21pp**. (On produced: −3.21 / +5.09 / −22.87pp.)

*Summary statistics are computed from the unrounded rates, so recomputing them
from the 2dp table can differ in the last digit — `pu1-lift` reads −24.20pp if
you subtract the table's rounded percentages, against −24.21pp from the raw
values. **The CSV's `delta_pp_*` columns carry the correct −24.21**, because
they are computed from the unrounded rates and only then rounded; it is
re-deriving the delta from the CSV's own 4dp rate columns that reproduces the
−24.20 artifact. Take deltas from the delta columns, not by subtracting rate
columns.*

**Why `bigpole1-lift-v2` and `pu1-lift` show identical sim baselines.** Raised
twice in review, and the answer is structural rather than coincidental, so it
belongs here.

The harness closes a measurement window on whichever comes first: the
**`WINDOW_ITEM_FLOOR = 300`** item count or a tick cap (`scenario.rs`; a
tick-cap closure sets `short_sampled`). Both of these runs closed on the item
floor with `short_sampled: false` and `window_items: 300`, so for **these**
windows the numerator is 300 by construction — never evidence of anything — and
the reported rate is `300 / window_seconds` for the final window. (It is not a
universal: a tick-capped window carries a different numerator, and an item-floor
closure only guarantees `>= 300` at the sampling cadence.) Both runs'
final window closed at `window_ticks = 17880` (298 s), hence

```
produced  300 / 298 s = 1.0067114093959733   <- both, exactly
delivered 304 / 298 s = 1.0201342281879195   <- both, exactly
```

That is the whole of it: a forced numerator plus a shared final-window
duration, which for two ~1/s factories both running at plan is unremarkable
given tick granularity (the observed windows across these runs are 17820,
17880, 17940 and 18060 ticks).

**And why delivered (102.01%) exceeds produced (100.67%)** on the same target
in the same window — which looks impossible and is worth pre-empting, since the
blocking-quadrant claim leans on that 102.01%. Over a *window* you can deliver
more than you produce if items already in flight reach the sink inside it. The
cumulative counters show exactly that, and show it is an oscillating buffer
rather than a drain: `pu1-lift`'s produced-minus-delivered backlog reads
**17 → 13 → 17 → 13** across the four checkpoints. The final window happened to
close on a 4-item fall, so it delivered 304 against 300 produced. Nothing is
accumulating or draining away; the in-flight buffer is small and flat.

**The runs are provably independent**, and the decisive evidence is the
per-window series, which differ:

| fixture | `window_rates` |
|---|---|
| `bigpole1-lift-v2` | 1.0067, 1.0033, **1.0067** |
| `pu1-lift` | 0.9967, 1.0101, **1.0067** |

Only the last entry — the reported one — coincides. Everything else about the
two reports differs as well: scenarios `…-bigpole1-lift-v2-1786128688` vs
`…-pu1-lift-1786126106`, 1058 vs 2516 entities, 6 vs 10 items, different
`final_tick`, different file hashes, and different raw counters (7114→8014 vs
6873→7773).

### ⚠ Units: this sweep and `sweep_corpus` do not report the same quantity

`sweep_postlift` reports **planned-relative percentage points**,
`(meter − sim) / planned × 100`. `sweep_corpus` reports **sim-relative percent
error**, `(meter − sim) / sim × 100` (`sweep_corpus.rs`, the `delta` binding) —
so the corpus's headline bounds are percentages of the sim reading, despite
this log having always called them "pp". The two agree only where sim ≈ plan,
which is precisely not the case on a below-plan fixture.

Planned-relative is the right unit for the gate classification below, because a
gate thresholds *% of plan*. But **cross-sweep comparisons must be made in the
corpus's units**, so `sweep_postlift` now prints both. The same six rows,
sim-relative:

| | mean | worst optimistic | worst pessimistic |
|---|---:|---:|---:|
| **on produced** | −3.12% | **+5.60%** | **−22.71%** |
| on delivered | −3.65% | +7.03% | −23.73% |

**And the metric must match too.** `sweep_corpus` computes its bound on
**produced** for solid targets, so the only like-for-like comparison against
the corpus's +1.3% / −13.6% is the **produced** row above. Quoting the
delivered +7.03% against a produced bound would commit the exact conflation
this section warns about — an earlier revision of this doc did precisely that
and claimed 5.4×.

### Both corpus-wide bounds are broken by this population

The corpus's two load-bearing numbers were *"every optimistic error is ≤
+1.3pp anywhere in the corpus"* and *"pessimistic errors run to −13.6pp"* —
both of which are, per the units note above, **sim-relative percentages**.
Post-lift, quoted in those same units so the multipliers mean something:

- **Optimism reaches +5.60% — 4.3× the corpus maximum** (produced, matching the
  corpus's own metric; +7.03% on delivered, +6.30pp planned-relative). This is
  the tier2 divergence the 2026-08-07 section already flagged as unexplained
  (it recorded meter 96% vs sim 90–91%); it **reproduces exactly** — 96.00 vs
  90.91 produced — from the banked blueprint. Warmup was falsified as its cause
  on 2026-08-07 (96.0% flat from 108k to 864k) and nothing here changes that.
  Still unexplained.
- **Pessimism reaches −22.71% — 1.7× the corpus maximum** (produced; −23.73% on
  delivered, −24.21pp planned-relative), on `pu1-lift`, a layout the sim
  measures at **102.01% of plan**. This one is new (below).

### The gate verdict, stated as the two quadrants

`sweep_postlift` classifies every target at each tolerance, on the delivered
rate the harness itself verdicts on. Against the corpus's **0 missed defects in
41 rows at every threshold ≥90%** — noting that the corpus figure was computed
on **produced** (`sweep_corpus` compares produced for solid targets), so
"1-in-6 vs 0-in-41" is not literally apples-to-apples. **The retraction does
not depend on that seam**: on produced, tier2 reads meter 96.00 vs sim 90.91,
which is still a missed defect at 95%. See the metric-sensitivity note below
the table for where it *does* matter.

| threshold | missed defects | false accusations |
|---|---|---|
| **90%** | **1/6 (`tier2-ec10-lift`)** | 2/6 (`pu1-lift`, `stress_ec_60s_red`) |
| **95%** | **1/6 (`tier2-ec10-lift`)** | 1/6 (`pu1-lift`) |
| 98% | 0/6 | 1/6 (`pu1-lift`) |
| 99% | 0/6 | 1/6 (`pu1-lift`) |

**Which of these rows is robust, and which is knife-edge.** The **95%** miss
holds on *both* metrics — meter 96.00 against sim 90.91 produced and 89.70
delivered — and is the load-bearing one.

**The entire 90% row is knife-edge, in both quadrants**, because two fixtures
straddle that cutoff between the reference instrument's own two columns:

- the miss (`tier2-ec10-lift`) exists only on delivered, by **0.30pp** — sim
  delivers 89.70% but produces 90.91%;
- one of the two false accusations (`stress_ec_60s_red`) likewise flips — sim
  delivers 90.67% (so the meter's 87.90% reads as a false accusation) but
  produces 89.83% (so it is a correct below-plan call).

So do not lean on the 90% row at all. Only the 95% classification is
metric-stable.

**But "metric-stable" is not "threshold-robust", and the 95% row is not that.**
These are two different axes and it is worth separating them, because an
earlier revision of this section labelled only the 90% row knife-edge and so
invited reading 95% as solid in a way it is not:

- *Metric-stability* — does the classification survive swapping produced for
  delivered? 95% yes, 90% no.
- *Threshold-sensitivity* — does it survive moving the cutoff? **No.** The miss
  exists because the meter's 96.0% sits above 95%. A gate at 97%, 98% or 99%
  sees the meter read below plan and there is no miss at all in this bank.

What the measurement actually establishes is the **underlying optimism** —
+5.09pp produced / +6.30pp delivered on `tier2-ec10-lift`, 4.3× the corpus
maximum — which is a property of the meter on that layout and does not move
with the cutoff. Whether that optimism *converts into a missed defect* depends
on where the threshold falls between the two readings, and at 95% it does.

That is still enough to retract the corpus claim, because the corpus claim was
universal over thresholds ("0 rows at 99%, 98%, 95% and 90%") and a universal
claim is refuted by one threshold. It is **not** enough to say a gate at 98%
would miss anything in this bank — it would not. If the gate lands at 98% or
99%, the honest statement is that this bank shows no misses there **and** that
the meter is 4.3× more optimistic than the corpus bound on one of six layouts,
which is a reason to distrust the margin rather than a demonstrated failure.

One thing this is *not* uncertainty about: the meter's 96.0% itself. The meter
is deterministic and warmup-invariant here — 96.0% flat across 108k→864k, an 8×
range — so the reading is exact and reproducible. The uncertainty is entirely
in whether it tracks reality, which is the thing being measured.

- **Report-only.** At 90% *and* 95% the meter reads 96.0% on `tier2-ec10-lift`
  — at plan — where the sim delivers 89.7%. That is the dangerous quadrant,
  occupied on the *first* post-lift fixture, at both tolerances a gate is most
  likely to pick. It is empty at 98% and 99% only because the meter's 96.0% is
  itself below those cutoffs, which is luck about where one number fell, not a
  property. **"Meter says below plan ⇒ believe it" is not established here**,
  and neither is its converse.

  **This is the row that retracts the floor claim, and it is independent of
  `pu1-lift`.** The floor property is a statement about *missed defects* —
  about the meter never being materially optimistic — so it is broken by tier2
  and only by tier2. `pu1-lift`'s −24pp is pessimistic; it bears on blocking,
  which the corpus had already ruled out. Fixing the petroleum-distribution
  defect below would therefore remove the false accusation and **leave the
  floor retraction standing**, because tier2's optimism has a different, still
  unknown cause. The two findings must not be treated as one.
- **Blocking.** `pu1-lift` is rejected at *every* tolerance from 90% up while
  actually delivering 102.01%. (That 102.01% is itself a class-5c reading — its
  run converged at the harness minimum, see the provenance note — so treat the
  *reference* as provisional; the −24pp gap is far too large for any plausible
  correction to close, but "rejected at every tolerance" is stated against an
  unconfirmed asymptote.) Blocking was already ruled out by the corpus's
  −13.6pp; this is worse and, importantly, **is not removed by the fix that
  removes the corpus outlier**.

**Consequence for the trust ladder.** Report-only remains shippable — a
report-only gate that misses is merely worth less, it blocks nobody — but it
must ship **without** the floor claim attached, because the floor claim is
**not established** on the population it would serve. That is a documentation
constraint on how the gate's output is worded, not a reason to hold it.

**How strong is this?** One fixture, `tier2-ec10-lift`, produces the missed
defect. That is enough to remove the *warrant* — a property asserted as
universal over a population fails on one counterexample, and the corpus claim
was explicitly "0 rows at 99%, 98%, 95% and 90%". It is **not** enough to
estimate a miss *rate*: "1 in 6" is a sample of six, and quoting it as a
frequency would overclaim in the opposite direction. The honest statement is
that the meter is **not known to be a floor** post-lift, not that it fails at
any particular rate. Widening the bank is what would turn one into the other.

### `pu1-lift` −24.21pp: petroleum-gas distribution, not the productivity axis

Ruled out first, by inspection rather than inference: `pu1-lift`'s manifest
declares `{"plastic-bar": 0.1, "processing-unit": 0.1}`, matching the install's
realized force bonuses, and its sim run has **empty `kit_errors`** — the parity
check would have failed the run otherwise. So the meter and the sim are
modelling the same world, and the residual is not item 1's cause.

The deficit is **uniform across every solid stage** — PU 77.81%, AC 77.79%,
EC 77.79%, copper-cable 77.79%, copper-plate 77.75%, plastic-bar 77.81%,
iron-plate 78.64% — against a sim measuring ~101% on all of them. A uniform
ratio means one shared constraint, and `attribute` locates it in two machines:

```
plastic-bar          2 machines   0 working   2 starved
sulfur               1 machine    0 working   1 starved
  [fluid] plastic-bar  at (21, 28)  coal=14/1 petroleum-gas=10/20 (fluid)
  [fluid] plastic-bar  at (24, 28)  coal=14/1 petroleum-gas=10/20 (fluid)
  [fluid] sulfur       at (21,  6)  water=420/30 petroleum-gas=15/30 (fluid)
```

Coal and water are abundant; **petroleum-gas is the shortage**, and
plastic-bar's starvation propagates exactly as observed (AC machines hold
`plastic-bar=0/2`, PU machines hold `advanced-circuit=0/2`).

**Distribution, not production.** `debug_fluid` shows all five oil refineries
in state `Working` — not `FullOutput` — with `fout=[("petroleum-gas", 0)]`:
producing steadily and never backing up, while the same three consumers hold
**1–3 units** against a per-craft need of 20–30. Producers unblocked and
consumers starved is a throughput limit **in the network between them**, not a
shortfall at the source.

*(The two probes report different buffer levels for the same machines — 10/20
and 15/30 above, 1–3 here — because they sample at very different points.
`attribute` runs `run_for(60*60*2)` then `run_for(60*60*3)`: **7 200 warmup +
10 800 measured ticks, i.e. 2 and 3 game-MINUTES**, 18 000 ticks all told.
`debug_fluid` uses the 108k/216k window. Both are instantaneous end-of-run
snapshots of a buffer that never fills, so the levels differ while the finding
— chronically short, never backed up — is the same.*

***And note what that arithmetic means: `attribute`'s whole run is 18 000 ticks,
which is BELOW the meter's own ~20–40k convergence floor recorded in the
2026-08-07 section.*** *So `attribute` is a **localisation** probe here — which
machines are in which state, and what they are short of — and its rates are not
to be quoted. The rate claim rests on `sweep_postlift`/`debug_fluid` at
108k/216k, which is 6× the floor and which independently shows the same three
machines short of the same fluid. An earlier revision of this note described
`attribute`'s window as "2 game-hours then 3", overstating it 60×; caught in
review of this PR.)*

**Related but not identified as the same defect.** `meter-fluid-followups.md`
records that Phase A's `tick_fluids` "delivered fluid one unit a tick and
throttled petroleum→plastic→AC→PU to ~20%", which Phase B's real pipe-network
model replaced. The chain throttled here is the same one, at ~78% rather than
~20%. Whether this is a Phase B residual on this topology, or something
specific to this layout's pipe run, is **not established** — the probes above
localise the shortage and separate distribution from production, and stop
there.

### Limits of this result

Six rows, four fixture families (EC ×3 counting both stress variants,
AC, PU, big-electric-pole), one machine tier per fixture, solid targets only.
It is a sixth the size of the corpus sweep and inherits the corpus's
fluid-target blind spot. What it is sufficient for is falsification: two
corpus-wide bounds and the empty-quadrant result do not survive contact with
the post-lift population, and that is enough to change what a gate may claim.
It is **not** sufficient to characterise the post-lift error distribution —
that needs the bank widened, which is the natural next increment.



## 2026-08-07 — corpus-wide calibration: the meter is safe as a FLOOR

Swept all 70 corpus layouts (35 fixtures × native/compact) with
`sweep_corpus`; **41** have a sim baseline to compare against.

**Distribution**: 25 pessimistic (meter < sim), 6 optimistic, 10 at ~0pp.
Mean −1.27pp, median −0.3pp. **Every optimistic error is small — max
+1.3pp anywhere in the corpus** — while pessimistic errors run to −13.6pp.
When the meter is wrong by a meaningful margin it is *always* wrong in the
safe direction.

> **Superseded in scope by the 2026-08-08 section above, not retracted.**
> Everything below is a correct statement about the **pre-lift Job-2 corpus**.
> Its two headline bounds — the empty dangerous quadrant, and "every optimistic
> error is ≤ +1.3pp" — are both **broken on post-lift layouts**, which is the
> population a gate serves. Read the corpus numbers as calibration history, not
> as a gate warrant.

**The gate question.** Classifying each row at-plan vs below-plan for both
instruments, the dangerous quadrant (meter says AT plan, sim says BELOW) is
**empty at every realistic tolerance** — 0 rows at 99%, 98%, 95% and 90%.
It appears only at a literal bit-exact 100% cutoff, where its 4 hits are sim
readings of 99.0–99.7% against a meter reading of exactly 100.0%, i.e. inside
the same ~1pp band that agreeing fixtures show as noise.

So: **"meter says below plan" ⇒ believe it** holds throughout, and is the
property a gate needs. **"meter says at plan" ⇒ evidence of nothing** remains
the correct caution: 3 of 23 meter-at-plan fixtures were softer sim passes
(96–99.4% meter vs 99–102% sim). Never a real miss, but clearance semantics
would overclaim precision the corpus doesn't support.

**The −13.6pp outlier is the known item, not a new one.** Both
`tier5_processing_unit_from_ore_am3` rows (−13.6 / −12.8pp) match this
document's existing ≈−13% entry for the unmerged research-productivity axis.
It is not a fluid gap: fluid-*ingredient* fixtures sit at −2.2 to +1.0pp.

### Two things this does NOT establish

1. **Fluid targets are untested.** All 4 fluid-target fixtures
   (heavy-oil-cracking, sulfuric-acid, 2× AOP) have **no sim baseline** in
   this corpus. The floor verdict is a **solid-target-only** result.
2. **The tier2 entry above is a different layout, and flips direction.**
   The corpus's `tier2_electronic_circuit` is the pre-lift zero-headroom
   layout: meter 56.0% vs sim 57.7–58.1% — **pessimistic**, in line with
   everything else. The 96%/90–91% figures recorded above are the *re-ranked
   post-lift* layout, which is not in this corpus — and there the meter is
   **optimistic by ~5–6pp**, which would be **four times the largest
   optimistic error in the entire 41-row corpus**.

   Those are not the same data point and must not be averaged. **Until this
   is resolved, the floor property is established for the corpus and NOT for
   post-lift layouts** — precisely the population a gate would run on.

   **RESOLVED 2026-08-08, against this section:** measured on six post-lift
   layouts (section above). The tier2 reading reproduces exactly — meter 96.00%
   vs sim 90.91%, **+5.09pp** — and at a 95% tolerance it *is* a missed defect,
   so the dangerous quadrant is occupied on the post-lift population. The
   caveat this bullet raised was correct; the floor property does not extend.

   **Warmup mismatch investigated and FALSIFIED (same day).** The obvious
   suspect was that the meter reading came from `check_one.rs`'s hardcoded
   108k-tick warmup against the sim's 432k, and that a short warmup reads
   buffer fill as throughput — which inflates, i.e. the right direction. It
   does not hold: the meter reads **96.0% at every warmup from 108k to 864k**,
   an 8× range including the sim's own 432k, with zero movement and
   `converged = true` throughout. The corpus was checked too — 11 of ~20
   fixture families genuinely did run their sim baselines at 288k against the
   meter's 108k, and re-running those across 108k–432k moved nothing by more
   than **0.5pp** (against PU's 13.6pp gap). So the mismatch was real in the
   setup and immaterial in the results; the 41-row calibration stands.

   Two things worth keeping from that check:

   - **The meter's own convergence floor is ~20–40k ticks**, characterised
     here for the first time, measured down to warmup 0 on the deepest
     fixture in the corpus (PU-from-ore, 6499 entities: 78.9% at 0 →
     plateau by ~40k). The 108k both drivers already use carries a 3–5×
     margin even there.
   - **"The default warmup is too short" does NOT transfer to the meter.**
     That caveat in `CLAUDE.md` / `status.md` is a property of headless
     Factorio's own convergence needs — which is why the corpus carries
     escalating per-fixture warmups — and assuming it applies to the meter
     is a mistake worth not repeating.

   So the residual is **genuine model-level disagreement**, and closing it
   needs snapshot/trace-level work rather than more black-box comparison.

## 2026-08-07 — tier2_electronic_circuit: meter 96%, sim 90–91% (open)

First divergence recorded from the *plan* side rather than the sim side, and
it is **under** the ±10pp bar, so it is a precision note rather than a defect
report.

| | copper-cable | electronic-circuit |
|---|---|---|
| planned | 30.0/s | 10.0/s |
| meter | 28.8/s (96%) | 9.6/s (96%) |
| sim | 27.0/s (90%) | 9.09/s (91%) |

**The meter got the important part right**: it said *below plan* on both
stages, which is the verdict that matters, and it said it in **19 seconds**
against ~10 minutes for the headless run. The underlying cause is
zero-headroom integral machine counts (`status.md`) — copper-cable plans at
exactly 10.0 machines, so any duty loss becomes a permanent shortfall. The
meter sees it because it models inserter swing and lost swings.

It is **~5pp optimistic**, and that direction is the one that matters for
any future use as a gate: a predictor that understates a deficit is safe as
a **floor** ("meter says below plan" ⇒ believe it) and unsafe as clearance
("meter says at plan" ⇒ not yet evidence).

**A candidate hand-size discrepancy was raised here and is now RETRACTED
(same day, investigated against primary prototype data).** It claimed the
meter's `BULK_HAND_BY_LEVEL` was one research level behind, on the reasoning
that at `inserter_capacity = 2` the game realizes `bulk_inserter_capacity_bonus
= 3`, so the hand should be `base_hand_size() + bonus = 2 + 3 = 5` where the
meter returns 4.

**That reasoning was wrong, and the meter is correct at every level 0–7.**
`base_hand_size()` *is* `hand_size(0)` — it already contains L0's bonus
(1 raw prototype floor + 1 from the `bulk-inserter` tech = 2). Adding the
force bonus on top double-counts that L0 increment. The true decomposition
is `hand = 1 (raw prototype floor) + bonus`, giving `1 + 3 = 4` — exactly
what `BULK_HAND_BY_LEVEL[2]` returns.

Primary sources (Factorio 2.0.77 install, not the wiki and not our docs):

- `data/base/prototypes/entity/entities.lua` — `bulk-inserter` sets
  `bulk = true` and has **no** `stack_size_bonus`, so its raw floor is 1.
- `data/base/prototypes/technology.lua` — the `inserter-capacity-bonus-N`
  chain carries Wube's own `-- result of N` comments, and every one equals
  `1 + cumulative_bonus`. Those reproduce `BULK_HAND_BY_LEVEL` and
  `NON_BULK_HAND_BY_LEVEL` exactly at L0–L6.
- `data/space-age/.../entities.lua` — `stack-inserter` is `bulk = true` with
  a literal `stack_size_bonus = 4`, i.e. `5 + bulk_bonus`, which is exactly
  the meter's `BULK_HAND_BY_LEVEL[level] + 4`.

The L7 non-bulk value is a **deliberate** divergence, already documented in
`entity_data.rs`: `transport-belt-capacity-2` is a separate Space-Age tech on
a different branch that grants a further `+1` non-bulk, so a
`research_all_technologies()` force reads 3 where the declared axis says 2.
`scenario.rs` overrides the force bonuses by direct assignment specifically to
keep that contamination out — the two agree by design.

**Lesson worth keeping**: `base_hand_size()` reads like an additive constant
and is actually `hand_size(0)`. Its doc comment says so; I misread it anyway.
This is the second time this table has attracted a wrong "fix" (PR #458 was
the first), which is why `entity_data.rs` says *transcribe, don't derive*.

So the ~5pp optimism remains **entirely unexplained** — this ladder has no
bearing on it in either direction.

## Corpus status (2026-08-06; Phase B landed 2026-08-03)

`meter sweep: 70 layouts measured, 41 compared`. Every compared fixture is
within ±10pp of sim except the one below. AC/PU families that were −80% in
Phase A are now within ±2% (the dedicated AC fixtures) / −13% (PU-from-ore; its
in-fixture AC reads −3.9%).

## Residual: diagnosis closed 2026-08-06, fix open

### `tier5_processing_unit_from_ore_am3` — meter ≈ −13% (`produced`) — diagnosis CLOSED, fix OPEN

- **meter**: processing-unit 1.716/s vs sim produced 1.987/s / delivered 1.961/s.
- **Cause (measured 2026-08-06)**: a **productivity tech-state parity gap**
  between the sim and the fast meter — not a layout, belt, distribution or
  supply defect. The sim calls `force.research_all_technologies()`, and its
  tech-state parity block corrects only inserter capacity (#370) and belt
  stacking (#385); nothing corrects productivity. The meter models no
  productivity at all, deliberately (`crates/meter/src/machine.rs` takes
  nothing from `module_policy` and not `effective_crafting_speed`). So on a
  boosted recipe the instrument and its reference measure different worlds.

  Realized force/research productivity, dumped by the harness probe (PR #580):

  | recipe | bonus |
  |---|---|
  | **processing-unit** | **+10.0%** |
  | **plastic-bar** | **+10.0%** |
  | advanced-circuit, electronic-circuit, iron-plate, copper-plate, copper-cable, sulfur, sulfuric-acid, basic-/advanced-oil-processing | 0.0% |

  `productivity_modules: {}` — empty, so the source is research, not modules.

  **Corroboration, out-of-sample**: the +10% came from a force-state read, not
  from any rate figure, and it correctly predicts the relationship between two
  independently measured rates — 43.2 EC/s ÷ 24 × 1.1 = **1.980 PU/s** against
  the **1.987** the sim measured, 0.35% apart. At zero productivity that same
  EC caps output at 1.800 PU/s, 9% below what it delivered. This is the check
  the four retired causes never had.

- **Decomposition**: the meter's −3.9% EC deficit compounded with the −9.1%
  productivity it does not model = **−12.7%** against **−13.6%** observed,
  ~1pp inside this fixture's noise (every intermediate here is ≈10% off plan).
  Only PU's boost moves the target: meter and sim both deliver 7.2 plastic/s,
  so plastic's boost changes the sim's petroleum input rather than its output.

- **⚠ Provenance of the probe run**: it used `--warmup 600` to reach finalize
  quickly, so its **throughput numbers are buffer-fill artifacts** and must not
  be compared against the recorded baselines above. Only the productivity
  fields are load-bearing — force state set at init, warmup-independent.
  (Factorio 2.0.77, matching `PINNED_VERSION`.)

- **Four causes were proposed and retired before this one.** Kept as a list
  because the bounds are the reusable part; the full derivations were removed
  on 2026-08-06 after they had cost more review rounds than they were worth.
  - *Belt-cycle update order* — permuting the 26-tile cyclic order moves PU
    1.716→1.754/s, ≈14% of the 0.271/s gap. Real, not the cause.
  - *Head-hog distribution* — 12/16 PU machines run at full rate while the four
    deepest starve, but perfectly redistributing the available EC yields only
    1.729/s: **+0.013/s, ≈5% of the gap**, at fixed EC supply. (That ceiling is
    an operating point, not an invariant — the permuted run reached 1.754/s,
    which needs more EC than the baseline produced, so EC production itself
    responds to belt-model changes.)
  - *Upstream EC/plate production* — rested on imputing 47.7 EC/s from the sim's
    PU output. Falsified by the sim's own copper-cable balance: 3×43.2 + 4×3.59
    = 143.96/s against 143.9/s measured, while the imputed figure needs 157.5/s.
  - *Sim-side reporting artifact* — falsified by the probe: there is a real
    bonus.

- **Verdict**: an instrument/reference parity gap, same class as the
  inserter-capacity (#370) and belt-stacking (#385) fixes already in the
  scenario.
- **DECIDED (owner, 2026-08-06): teach the meter productivity.** Rationale
  recorded verbatim-in-spirit — *"it's important for the meter to be able to be
  flexible, and for it to match what we actually produce as measured in the
  sim."* The sim stays the reference; the instrument learns to model what the
  reference actually does.

  One scoping fact found while writing this up, which widens the fix beyond the
  meter: **the solver did not model research productivity either.**
  *(Implemented 2026-08-06 — `netflow.rs` now folds a declared per-recipe
  research bonus into `effects.prod_bonus` alongside module and base-effect
  productivity. The three parts land as #584 meter / #585 sim / #587 solver.
  Note the plan does NOT scale uniformly by 1/(1+bonus): a stage that carries
  its own bonus AND serves reduced downstream demand compounds — plastic-bar
  scales 1/1.21 on this fixture, basic-oil-processing 1/1.19. Correct
  arithmetic; an earlier write-up of mine claimed "every stage scales by
  exactly 1/1.1", which is only true of the boosted target stage.)*
  `netflow.rs` applies productivity from *modules* and a machine's
  `base_effect`, gated on the recipe's `allow_productivity`, and
  `ModulePolicyKind` defaults to `None`. Research-sourced productivity — the
  +10% the sim actually runs with — is modelled nowhere on the engine side. So
  the *plan* is over-provisioned on PU by the same 10% as the meter's
  prediction is short; this is not only an instrument gap.

  Shape the fix should take, following the pattern #370 and #385 already
  established for exactly this problem: make research productivity a
  **declared axis** carried on the manifest alongside `stacking` and
  `inserter_capacity`, so that (a) the meter applies it, and (b) the sim
  *pins* it in its tech-state parity block instead of inheriting whatever
  `research_all_technologies()` grants. Declaring it on both sides makes them
  match by construction rather than by coincidence — a meter that models a
  declared level while the sim runs an incidental one agrees only by luck.

- **Prediction, and its result (2026-08-06).** Teaching the meter +10% on PU
  was predicted to land its output at **≈1.902 PU/s**. Implemented on
  `feat/research-productivity-axis` and measured on this fixture at the Stage B
  deep-chain warmup:

  | config | PU | EC | ceiling@EC | % of ceiling |
  |---|---|---|---|---|
  | declared none | 1.7056/s | 41.49 | 1.7287 | 98.7% |
  | declared +10% | **1.8500/s** | 41.49 | **1.9018** | 97.3% |

  **The ceiling under productivity measures 1.9018 against the predicted
  1.902** — the productivity model is exactly right. What the prediction got
  wrong is assuming the meter would *reach* its ceiling.

  **What the shortfall actually is — AC over-production, not distribution.**
  The `E/24` ceiling assumes AC is produced exactly in proportion (2 per PU
  craft). It is not. Computing against the *measured* AC instead —
  `(E − 2·AC)/20 × 1.1` — fits far better:

  | config | observed | `E/24` | net of measured AC |
  |---|---|---|---|
  | none | 1.7056 | 1.7288 (98.7%) | 1.7123 (**99.6%**) |
  | +10% | 1.8500 | 1.9016 (97.3%) | 1.8407 (**100.5%**) |

  Because AC runs over: **2.12 AC per PU craft baseline (+6%), 2.39 with
  productivity (+19%)**, against the recipe's 2. That surplus consumes EC which
  would otherwise reach PU, and it is not a distribution loss — it is the
  **solver's** blind spot to research productivity showing up downstream. The
  plan sizes the AC stage for a PU stage that needs no productivity; give PU
  +10% and it needs fewer crafts, so the unchanged AC sizing over-serves it.
  An earlier revision of this entry attributed the gap to distribution; that
  was wrong, and it was caught by a reviewer pointing out that `24 EC per
  craft` is a full-chain figure rather than a per-craft one (a PU craft
  consumes 20 EC directly plus 2 AC).

  So the residual after the meter-side fix is **the other half of the same
  bug**, and closing the solver side should close most of what is left.
  Note also that EC production is **identical (41.49/s) in both configs**, so
  declaring productivity does not move it — EC is genuinely unboosted and
  supply-limited, corroborating the probe's `electronic-circuit: 0.0%`
  independently.
- **Still open after the meter fix**: the −2.7% ceiling shortfall (the
  distribution term, which grows slightly as the same EC supply feeds more
  output) and the −3.9% EC deficit itself. The latter is where the **solver's**
  own blind spot to research productivity lands: `netflow.rs` models modules and
  `base_effect` only, so the plan is over-provisioned by the same factor. That
  is the other half of the fix — **now implemented** (#587); what remains is
  wiring a caller to declare a real value, which changes plans and is
  deliberately separate.
- **Superseded**: this bullet previously predicted the residual would fall to
  ≈−4.3%, "essentially the −3.9% EC deficit alone". The measurement above puts
  it at −6.9%: the prediction's *ceiling* was right to four figures, but it
  assumed the meter would sit on that ceiling, and it does not. The trap it
  warned about still stands and is worth keeping — the 1.729 figure quoted
  elsewhere is a ceiling *at zero productivity*; productivity does not change
  EC consumed per PU including its AC leg (still 24; a craft itself takes 20 EC plus 2 AC), it changes PU produced per craft (1 → 1.1),
  so it raises the ceiling rather than capping output beneath it.

- **Open, not closed**: (a) the ~1pp decomposition residual is attributed to
  fixture noise, not explained; (b) plastic-bar's input-side consequence is
  asserted, not checked — at +10% the sim makes its 7.2 plastic/s from ~9% less
  petroleum than the meter models (≈65.5 vs ≈72/s), and the sim's petroleum-gas
  was already −17%, so if that input ever binds it is a second gap of the same
  class.

## Closed / moved entries

- **AC / PU / AC-partitioned (~−80%)**: closed by Phase B pipe-fast, fair,
  buffer-absorbing fluid routing. AC variants are now ±0–2% (the PU-from-ore
  exception fixture's own AC is −3.9% by the same meter).
- **`tier3_plastic_bar_from_crude` (~−80%)**: closed (now 10 = plan).
- **AOP / refinery / sulfur / heavy-oil cracking (delivered)**: no divergence;
  the fluid networks (incl. the forced-pipe-isolation AOP) route correctly.

## Non-gated observations (no sim baseline, so NaN — not counted)

- `tier3_advanced_oil_processing_forced_multi_machine_pipe_isolation` has no
  `report.json` (sim NA) and is not compared. The meter's pipe-isolation
  topology keeps crude/water/heavy/light/petroleum in separate networks, and
  the refinery refines correctly, but the two light-oil + one heavy-oil
  cracking chemical-plants run slightly starved (hold ~75% of a per-craft
  need). Worth a calibration fixture one day; not a regression vs Phase A
  (which could not route fluids at all).

## Deliberately-not-modelled (decision log)

- **Byproduct backpressure.** The meter drains every unconsumed producer fluid
  unit as `delivered` and never stalls a producer on a full fluid output, so a
  multi-output recipe (AOP heavy/light/petroleum) with an unhandled byproduct
  reads its *maximum* throughput rather than stalling as real Factorio would.
  This is a **conscious choice, not a bug**: the meter's stated philosophy
  (see `factory.rs` header) is to drain outputs so measurement is not falsified
  by backpressure, matching the sim harness's remove-mode-chest methodology that
  the meter calibrates against. Consistent with that, all 8 fluid-target corpus
  fixtures are uncompared (no sim baseline), so this divergence is unverifiable
  and would only matter if a sim-baselined byproduct-loop fixture joined the
  corpus. Revisit then; recorded so the call is explicit. (2026-08-03)

## 2026-08-21 — curve chirality ADJUDICATED (meter cleared); mirror flag parsed

- **Curve chirality**: the domain-physics audit accused this meter's
  `LaneMap` (no swap variant; every curve lane-for-lane) of being unable
  to represent the chirality-dependent lane swap `validate/belt_flow.rs`
  implemented. Adjudicated against game rule B11 (expert-confirmed: lane
  contents never jump lanes through a turn, either chirality) plus both
  models' handed seeding conventions: **the meter was CORRECT and is
  cleared; the swap was core's bug — fix on PR #683** (whose corpus
  consequence, one fabricated `input-rate-delivery` reading on the
  ac-am2-ore fixture, is adjudicated with position forensics in that
  PR's re-bless commit). The discriminating both-chirality lock lives in
  belt_flow's `lane_transfer` test on #683; the meter-side test is a
  documentation-level pin (its comment states why it cannot discriminate
  today). Model-comparison lesson recorded in
  `domain-physics-audit-2026-08.md`: a divergence identifies a
  disagreement, not the guilty side.
- **Fluid-port mirroring**: the decoder now PARSES the blueprint
  `mirror` flag; the factory deliberately does not yet CONSUME it — for
  the three engine-mirrored machines the name heuristic already applies
  the binding flip (their mirror is tile-identical), and for any other
  machine an explicit mirror also moves port TILES, so honoring it as a
  binding flip alone would mis-place ports on asymmetric machines
  (#685 review). The complete fix is a `reflect_port` (w−1−dx) plus the
  flag; the engine-form undecidable (unmirrored community instance of
  the three, multi-fluid face) remains documented rather than guessed.
- **UG turbo tier**: core's `ug_max_reach` gained an explicit
  `turbo-transport-belt => 10` arm (was falling to the yellow-4
  fallback), closing the meter/core tier-table asymmetry for imported
  blueprints.
