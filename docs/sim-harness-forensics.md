# Sim-harness forensics — reading and debugging measured results

Reference doc for `spaghettio-sim` (RFC-050). What each reported number
actually is, the measurement-artifact classes we have hit (each was found
by challenging a number that looked decisive), and the forensic playbook
that localizes a bad result. Keep current as the harness changes.
Setup, CLI usage, and the concurrency/lock rules live in
[`sim-harness.md`](sim-harness.md).

## The shape of a healthy stage: y = mx + c

Owner observation, 2026-08-07. Cumulative production for any one machine
row, plotted over time, should be close to **piecewise {flat, ramp,
straight line}**:

- **c / start offset** — the stage produces nothing until its inputs
  arrive, so deeper stages start later. Not purely recipe depth: it is
  belt transit plus buffer fill, so a stage far along a long bus starts
  later than its depth alone implies.
- **ramp** — belts filling and machines spinning up. Not instant.
- **m / slope** — the steady-state rate. Set by machine count, quality,
  modules, and effective duty. **This is the number the plan predicts.**

Why this is worth more than the rate panel alone — three things fall out
of it that a scalar cannot show:

1. **Straightness is the starvation test.** A healthy stage's cumulative
   curve is *straight* through the measurement window. A starved one is a
   **staircase**: flat while it waits, steep when fed. It averages to a
   perfectly plausible rate. The mean is the same; the shape is not. This
   is the signature an eye catches and a number hides, and it is the class
   that has repeatedly shipped here as "validator-clean but game-dead".

2. **It gives a visual test for warmup adequacy.** We warm up specifically
   to get past the ramp, and the default warmup is known to be too short
   for deep chains — it "reads buffer fill as throughput"
   (`status.md`). Today that is a judgement call. On this graph it is a
   look: **if a stage is still on the curve during the measurement
   window, the reported number is buffer fill, not throughput.**

3. **Uniform slope-scaling vs. one shallow slope localizes the fault.**
   If every stage's slope is scaled by the *same* factor, the constraint
   is shared (input supply, power, a global cap) and no single stage is
   guilty. That is exactly the PU signature recorded in `status.md`
   (copper-cable .6874, copper-plate .6878, EC .6875, iron-plate .6874,
   plastic .6875 — one factor across the whole chain). If instead **one**
   stage is shallower than its neighbours, that stage is the bottleneck
   and the ones downstream of it inherit its slope.

The raw cumulative counters are pushed to Graphite alongside the derived
rates precisely so this shape is inspectable — see the "Cumulative
production per stage" panel on `/d/spaghettio-sim`.

## What each number is

- **Target item rates** (`measured_produced_rate` / `delivered` for the
  manifest target): Δ(cumulative production counter) over the **last
  checkpoint window**, which closes on 300 accumulated items rather than
  on a fixed duration (#454), so its length varies with how the factory
  is actually running. `converged=true` means the trailing **three**
  window rates agreed as a group, widest vs narrowest, within 2%.
- **Intermediate item rates**: measured over the **same trailing
  checkpoint window** as the target (since #362). Before that they were
  the last two 20-second samples — badly aliased for bursty producers (a
  gear machine crafting in bursts read 0.40/s on a snapshot vs 0.80/s
  honest).
- **`samples`** (in `raw_result`): cumulative production counters for
  every planned item every 1200 ticks (20 game-seconds), covering the
  whole run *including warmup*. This is the trajectory record — bin it
  to see transients, plateaus, and oscillations. Whole-run averages and
  first-divergence ordering both come from here.
- **`sim_state`** (frame at finalize): per-belt per-line item contents
  (belts now also carry entity name, direction, and underground pairing
  type, and empty belts are included rather than skipped — see
  `docs/sim-harness.md`), machine statuses + input/output inventories,
  inserter statuses, UG pairing as the game resolved it, splitter
  priority/filter state, kit chest census. A *single frame* — statuses
  are instantaneous (a demand-limited machine flickers
  `working`/`full_output`; do not read one frame's status as a
  time-average).
- **`kit_errors`**: the boundary kit's self-audit. Non-empty ⇒ the run
  is invalid and the verdict is forced NO DATA. Never interpret rates
  from a run with kit errors.

## Measurement-artifact classes (all real, all found in one day)

1. **Buffer-fill transient read as convergence.** The 2% stability
   window cannot distinguish steady state from a slow drift: a
   deep-chain fixture "converges" while trunk buffers are still
   filling. Signature: intermediate stages at or *above* plan while the
   target fails — above-plan draws are never steady state. Cure:
   `--warmup 216000`-class steady-state probes; measure after the
   transient.
2. **Snapshot aliasing.** Any rate computed over a window much shorter
   than the producer's burst cycle is noise. Signature: an intermediate
   rate wildly inconsistent with its neighbors' consumption arithmetic.
   Cure: recompute from `samples` over a long trailing window (the
   default reports now do).
3. **Kit contamination.** Boundary-kit rigs that collide cross-feed
   items: overlapping bank chests (Factorio's `create_entity` in script
   mode **stacks entities silently**) let an inserter latch the wrong
   item's chest. Wrong-tier items then poison the factory — see the
   poison-plug mechanic below. Signature: `item_ingredient_shortage`
   beside a full input belt; wrong item in a belt-in's per-line
   contents; nonuniform "starvation" that no capacity arithmetic
   explains. Cure: the kit self-audit (chest census → `kit_errors`),
   plus depth-staggered rigs. **When a sim result shows wrong-item or
   inexplicable starvation signatures, suspect the kit before the
   layout.**
4. **Underperformance-proportional undersampling** (#454, fixed
   2026-07-25). Windows were sized from the *planned* rate and closed on
   a fixed tick count, so a factory at 40% of plan got 40% of the
   intended 300-item sample and the 2% agreement test became
   unreachable — **the worse a factory performed, the less measurable it
   became**, failing closed to NO DATA. Signature: `converged: false` or
   NO DATA on exactly the fixtures that underperform, with the deficit
   tracking layout *size* rather than any intervention you made. Cure:
   windows now close on accumulated items; check the `measurement:` line
   for `short_sampled` and the checkpoint count.
5. **A transient reported as a steady state.** The reported rate is the
   trailing window whether or not the run converged, so a
   non-converged run publishes a point on a slope as a two-decimal
   number. Signature: a monotone window-rate series — usp2-sup120
   climbed 0.70 → 0.74 → 0.88 while usp2-shortrows decayed 0.86 → 0.80
   → 0.72, and the two were compared against each other as if both were
   settled. Cure: read the `NOT CONVERGED` line and its window-rate
   series before believing any number; **never compare rates across
   runs that did not converge.**
5b. **A ramp certified as convergence** — the same artifact reaching
   *converged* runs, and the nastier half. The stability test compared
   only the last two windows, which any decelerating ramp passes once
   its slope flattens under 2%, at a point short of its asymptote.
   chem5 (registered PASS) was certified on 4.62 → 4.92 → 5.00/s and
   published the trailing window as "5.00/s EXACT at plan" while the
   measured span averaged 4.84/s. Signature: a monotone window-rate
   series in a run marked `converged: true`. Cure: convergence now
   compares the trailing three windows as a group; a `converged` run
   whose `drift_pct` is near the tolerance still deserves a longer
   `--warmup` before its number is blessed.
5c. **A plateau certified as the asymptote** — a residual the group
   rule reduces but cannot eliminate, since convergence only means three
   consecutive windows agreed and on a long chain that *could* be a step
   on a staircase. Kept as a standing caution rather than an observed
   class: **the one candidate sighting did not reproduce.** A 3-window
   480k-warmup probe of usp2 read 0.83 → 0.85 → 0.97 and looked like a
   staircase, but a 9-window run at the blessed geometry stayed flat
   across 47 game-minutes (mean 0.850/s, spread 2.9%, net trend −0.35%).
   Three windows reporting `NOT CONVERGED` with +13.9% drift was the
   instrument correctly refusing to answer — reading a trend into it was
   over-reading. Cure regardless: on a deep chain, confirm a converged
   number with a longer `--warmup` before blessing it, and prefer a run
   with many windows over one with the bare minimum; the
   intermediates-at-or-above-plan tell from class 1 applies here too.
6. **A budget that cannot fit the test.** `--warmup` used to re-floor the
   tick ceiling at warmup + ONE window while convergence needs four
   checkpoints (three closed windows), so any warmup past the default
   ceiling reported
   `converged: false` by construction. Signature: fewer than 4
   checkpoints, `final_tick` ≈ warmup + one window. Cure: fixed in
   `viable_end_tick`; the report now warns when checkpoints < 4.

## The poison-plug mechanic (game truth, mechanics rule I11)

Inserters refuse to pick items their destination cannot accept. On a
dead-end feed belt the inserter is the only exit, so a single wrong
item reaching the front tile **plugs that lane permanently** — the
machine starves with a "full" belt beside it. One contaminant item is
enough; the plug never clears. This is also why contamination is so
destructive out of proportion to its rate: ~17 stray copper plates
capped an entire factory.

## Reading time-series decay shapes

`report.timeseries` / `raw_result.timeseries` (per-window, since #537 —
see [`sim-harness.md`](sim-harness.md#reading-the-time-series) for the
schema) adds two things `samples` and `sim_state` don't have on their
own: a **per-machine** series (not just per-item), and a series aligned
to the **same checkpoint windows** the reported rates are computed over,
rather than a fixed 20-second cadence. Read the shape of the target
item's `items[target]` deltas across windows, cross-referenced against
`machines[].status`, before reaching for any deeper forensics:

- **Flat zero from tick 0** — every window's `crafts_delta`/`produced_delta`
  is zero from the first checkpoint on, with machine `status` sitting on
  a shortage from the start (`item_ingredient_shortage`,
  `fluid_ingredient_shortage`). This implicates the **feed path**, not
  the layout's internal throughput: the boundary kit, an exporter bug
  severing a connection (#373's inverted pipe-to-ground direction was
  exactly this shape), or a genuinely disconnected belt/pipe. The factory
  never ran, so nothing downstream is informative — start at the
  boundary, not the machine that's short.
- **Ramp, then decay** — early windows show positive, often climbing,
  deltas, followed by a decline back toward (or to) zero. This is the
  buffer-fill mirage's OTHER half: not "converges while still filling"
  (class 1, `docs/sim-harness-forensics.md` above) but the case where the
  fill **empties into a jam** — a downstream dead end, a capacity
  mismatch that only bites once buffers between stages exhaust, or a
  kit-side chest that stops draining. The `machines[].status` series
  across the same windows usually shows the transition directly: a
  machine flips from `working` to `full_output` or a shortage status at
  the same tick the deltas turn over.
- **Stable, but below plan** — deltas settle into a flat, non-zero band
  under the planned rate, with `status` steady (not flickering between
  shortage states). This is the shape of a genuine capacity deficit —
  too few machines, an inserter-throughput ceiling, an undersized belt —
  the kind of thing the verdict machinery is designed to catch, and the
  one shape that does NOT implicate the harness or a transient.

**#537, the motivating case:** `land-mine@1` measured 0/s at
`--warmup 288000` under both DI claim orders and all three machine
tiers, with a census reading `fluid_ingredient_shortage: 2,
item_ingredient_shortage: 2, full_output: 4` — a snapshot consistent with
either "never started" or "ran fine, then jammed". The harness's own
UNCALIBRATED-fluid-boundary note (see `report.rs`/`scenario.rs`,
`fluid_fed`) was stale — #373 had already fixed the pipe-to-ground
direction bug it was warning about — and got misread as the explanation,
sending the investigation in the wrong direction (RFC-059's decision log,
`docs/status.md`, and PR #535 all had to be corrected). A `timeseries`
would have settled it in one look: flat zero from the first checkpoint
onward is a feed-path defect a control (`plastic-bar@1` from crude-oil,
which measures fine) doesn't share — no need to trust a note about the
instrument that was true when written and false when read.

## Forensic playbook (in escalation order)

**Step 0, before any of the below: `scripts/sim-localize.py <report.json>`.**
It renders the "where" in one command instead of an improvised read (which
has gotten the belt-count semantics wrong before — see the `n` warning
above): a kit-error banner if the run is invalid, the item table with its
below-plan intermediates listed (a listing, not a causal order), a starved/backpressured machine ranking (from
`timeseries` when present, falling back to the final `sim_state` frame with
an explicit "can't distinguish transient from persistent" caveat), an ASCII
map of machines/inserters/belts by status and direction, and per-lane belt
contents around the worst machines. It renders and ranks; it does not
diagnose — steps 1-4 below are still where the reasoning happens.

1. **Trajectory first** (`samples`, and now `timeseries` for a
   per-machine breakdown on the SAME checkpoint-window cadence the
   reported rates use — see "Reading time-series decay shapes" above):
   bin per-item rates over game-time.
   Distinguishes transient vs plateau vs oscillation, and the *order*
   in which stages diverge from plan points at the causal root.
2. **Frame reading** (`sim_state`): machine statuses + inventories
   joined with belt per-line contents against `entities.json` segments.
   Full-upstream/starved-downstream = backpressure from below; wrong
   item anywhere = stop, check the kit.
3. **Micro-fixture isolation**: rebuild the suspect geometry as a
   ~10-entity blueprint + hand manifest, flood it, measure. If the
   micro passes at capacity, the local mechanic is innocent — the
   defect is systemic or infrastructural. **Derive `bbox_min`/dims/
   boundaries from the entity list programmatically** — two of our
   three false reproductions were hand-typed manifest errors (anchor
   off by one; drain boundary one tile past the belt end).
4. **Infrastructure census**: kit chests (overlaps, contents), UG
   pairing vs engine intent, splitter priority state as revived. The
   instrument is a suspect too.

## Operational pitfalls

- **Factorio instance lock**: back-to-back runs race the previous
  server's shutdown (`Couldn't acquire exclusive lock`). Guard with
  `until flock -n <install>/.lock -c true; do sleep 2; done` between
  runs. Never `pgrep`-wait with a pattern that matches your own shell.
- **Belt counts are per transport line** of the entity: a 2-tile
  single-lane line reads as "8" on *each* entity; splitters (8 lines)
  and UGs read higher. Use the per-line contents, not the total, when
  lanes matter.
- **`create_entity` in script mode ignores collisions.** Anything the
  scenario builds must be followed by an occupancy audit; a silent
  overlap is invisible on every belt and poisons everything downstream.
