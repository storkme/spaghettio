# RFC-054: The fast meter — a native item-level factory simulator

Registry: [`rfcs.md`](rfcs.md). Status: **Draft (circulated for review)**.

## Summary

Build `spaghettio-meter`: a native, item-level discrete simulator that
takes **the exported blueprint string plus the sim manifest** — the same
two artifacts [`spaghettio-sim`](sim-harness.md) takes — moves items
around at tick granularity, and reports **measured** per-item rates and a
per-machine census. Roughly **20× cheaper** than headless Factorio and
with no install, no server and no process startup — enough to run in CI
and, if KC3 holds, inside the layout candidate search. (Concretely:
`spaghettio-sim` is ~25–45 s plus ~10 s startup; KC3 sets the meter's bar
at ≤2 s and its kill threshold at 5 s.)

It is **not** a validator and must not become one. A validator returns a
verdict; a meter returns a number. The difference is the whole point: you
cannot optimize against "0 errors", but you can optimize against
"14.20/s against a plan of 15.00/s". Its output is `produced_per_s` and a
machine census, never an `Issue`.

RFC-050 gave the project an oracle. This RFC builds the cheap instrument
that the oracle makes it possible to trust.

## Motivation

### The defect class the validator structurally cannot see

[#448](https://github.com/storkme/spaghettio/issues/448) found one
mechanism behind three separately-diagnosed failures: an intermediate
item is **simultaneously backed up at its producers and absent at its
consumers**, with starvation concentrated at the *tail* of the consuming
row. On `chain-ec15` the per-machine dump reads:

```
EC row:  (46,8) cable 42 | (49,8) 34 | (52,8) 20 | (55,8) 6 | (58,8) 22 | (61,8) cable 2  <- SHORTAGE
cable cell: (19,6) FULL_OUTPUT, copper-cable 32 stuck
```

The tail machine at `(61,8)` holds 2 cable and is in
`item_ingredient_shortage`, while a producer cannot push into the belt at
all. #448's own narrative adds the detail that makes it diagnostic (not
visible in the excerpt above, which carries only cable counts): that tail
machine has **20 iron plates idle beside it**. So it is not
output-blocked and not generally starved — it is short of exactly one
ingredient, the one whose belt runs the length of the row.

The mechanism is a *flux* deficit, not a storage one, which is why time
does not resolve it:

1. A belt's nominal rate (15/30/45) is the flux past a point on a fully
   compressed, freely-moving belt with nothing taken off it.
2. Inserter extraction punches gaps. **Gaps on a free-flowing belt do not
   heal** — items travel at uniform speed and only compress against a
   blockage.
3. Inserter throughput degrades on a gappy belt: the hand arrives ready
   and finds nothing, and multi-item hands need *consecutive* items.
4. At exactly-100% provisioning the flux arriving at the last machine
   equals its entire demand, so it must extract **100% of what passes**.
   From a gappy belt, it cannot.
5. The dead-end that would normally back up and re-compress never does,
   because re-compression needs *surplus* reaching the end and the
   starving tail consumes everything that arrives. The rescue mechanism
   is disabled by the starvation it would rescue.

Every step there is time-domain: gap propagation, buffer depth, inserter
burst size, consumer order along a belt. The validator's model is flow
conservation on a static graph, and in that language `supply == demand`
is **correct**. This is not a missing check. It is a dimension the model
does not have.

Corroboration that it is flux and not storage: the deficit is stable
across a one-game-hour steady run, and across research levels d1 → d7 it
**barely moves** — −8.0% → −6.0% → −5.3%, with an *identical machine
census* at every level (1 output-blocked, 1 ingredient-starved, 13
working) ([#435](https://github.com/storkme/spaghettio/issues/435)). The
d1 → d2 step is a separate input-side bind that L2 clears; from d2 to d7,
five research levels buy 0.1/s. No inserter capacity helps a machine with
nothing under its pickup tile.

### The cost of only having a slow instrument

`spaghettio-sim` is ~25–45 s per blueprint at USP scale plus ~10 s
startup, needs a Factorio install, and cannot run in CI. So it is a
close-out ritual. Consequences visible in the backlog today:

- Real deficits are invisible until someone runs a job by hand:
  `chain-ec15` −5.3%, `mega-chain-pu4raw` −27.3%
  ([#437](https://github.com/storkme/spaghettio/issues/437)),
  `mega-chain-usp2raw` −57.0%
  ([#453](https://github.com/storkme/spaghettio/issues/453)),
  `chain-mil5ore` −28.7%.
- The response to each is a hand-written check for that shape
  (`row-output-lane-budget`, then `row-input-belt-margin`). #453 states
  plainly that three of the four failing fixtures remain **unattributed**
  — margin is available and nobody knows what binds them. Hand-written
  checks do not scale to defects nobody has root-caused.
- #448's threshold sits at *exactly* 100% and says so, because 100% is
  the only condition anyone has measured failing. The check is an
  explicit **lower bound**; the true required margin is unknown and
  almost certainly varies with row length, inserter tier and research,
  belt tier, stacking, and consumer order. Finding it means sweeping
  hundreds of variants of one row — a weekend of compute for a native
  sim, a week of hand-run Factorio jobs otherwise.

### Why now, and not before RFC-050

A homemade simulator is normally untrustworthy because nothing checks it.
That objection is now answered: there are **14 committed measured
configurations** (5 blessed baselines + 9 registry entries) plus **3
more quantified in issues**, spanning clean passes, marginal ~5% shortfalls,
and outright failures. The calibration set exists, is committed, and
predates this RFC — so the kill criteria cannot be tuned to.

**This is a different proposal from the audit's §6-D**
([`architecture-audit-2026-07.md`](architecture-audit-2026-07.md)), which
suggested black-box search "with validators as fitness". Validators are
precisely what is blind to this class. §6-D is superseded by this RFC.

## Design

### Placement: a separate crate, deliberately

`crates/meter/` (`spaghettio_meter`), a new workspace member with a
`spaghettio-meter` binary.

It depends on `spaghettio_core` for **exactly two things**:
`blueprint_parser::parse_blueprint_string` and `recipe_db` (ingredients,
craft times, machine crafting speeds). Both are *data*. Everything
physical is the meter's own.

**The load-bearing rule — the meter may not import the engine's derived
rate model.** Specifically banned: `machine_feed_rate`, `belt_drop_rate`,
`lane_capacity*`, `utilization_for`, `LANE_UTILIZATION`,
`ROW_LANE_FACTOR_*`, and anything else in `common.rs` that is a
hand-calibrated *estimate* rather than a game constant.

This distinction is the entire integrity argument:

| | Sharing is | Why |
|---|---|---|
| Belt speed, inserter swing time, recipe ingredients, machine crafting speed | **fine** | facts about Factorio |
| `machine_feed_rate`, `belt_drop_rate`, lane capacity, utilization factors | **fatal** | the engine's *beliefs*, hand-calibrated |

If the meter imported `machine_feed_rate` it would reproduce the engine's
estimate instead of measuring one, and its agreement would be circular —
the exact failure that made `carries` labels worthless as ground truth
(audit §3.3 #17) and that let the backwards-inserter bug survive for the
project's entire history behind three artifacts that all shared the
engine's direction convention. A separate crate makes this a *compiler*
boundary rather than a matter of discipline, and KC4 tests it.

### Input: the artifact, not the IR

```
spaghettio-meter run --bp <file.bp> --manifest <file.manifest.json>
```

Byte-identical invocation shape to `spaghettio-sim`. The meter reads the
same exported blueprint string Factorio would read, and the same manifest
for boundary sources/sinks and planned rates. It never sees
`LayoutResult`, `carries`, segment ids, or any rate annotation.

Two payoffs beyond integrity: A/B against the real harness is a one-line
change, and an export-level bug (the #348 class) is *reachable* by the
meter rather than invisible to it.

### Physics: items, not rates

**This is the one decision that determines whether the RFC is worth
executing.** A rate-based belt model is another flow model and will
reproduce the validator's blindness exactly. The model must be
item-level.

- **Belts** — each lane is a sequence of discrete slots at Factorio's
  item spacing. Each tick an item advances iff the slot ahead is free.
  This cellular-automaton form gives the three properties the defect
  needs *for free*: compression only occurs against a blockage, gaps do
  not heal on a moving belt, and order is FIFO. Nothing about tail
  starvation is written into the model; it falls out.
- **Underground belts** — pair endpoints, lane preserved.
- **Splitters** — alternating output, lane behaviour and priority per
  [`factorio-mechanics.md`](factorio-mechanics.md) §Splitters.
- **Inserters** — an explicit state machine: `idle → swing-to-pickup →
  grab → swing-to-drop → release`, with swing time from entity data and
  hand size from tier + declared research level. Pickup draws from
  **both lanes** under the pickup tile (mechanics **I6**) and **stalls
  when the slots are empty**; drops go to the **far lane** (**I5**).
  Density-dependent throughput is therefore *derived*, not modelled —
  which is precisely why the meter can see what `machine_feed_rate`
  cannot. Note I6's own wording already concedes the point this RFC
  turns on: effective pickup rate is limited by what is physically on
  the belt, and only a fully loaded belt delivers full throughput.
- **Machines** — ingredient buffers with Factorio's insertion limit, a
  craft timer of `recipe_time / effective_crafting_speed`, and a capped
  output slot. States map onto the harness's existing census vocabulary
  (`working`, `full_output`, `item_ingredient_shortage`) so reports are
  directly comparable and `scripts/sim-capture-state.sh` forensics
  transfer unchanged.
- **Boundary** — manifest `boundary_inputs` are infinite sources,
  `boundary_outputs` infinite sinks. No feed rigs, no drain banks, hence
  none of the kit-contamination artifact classes
  ([`sim-harness-forensics.md`](sim-harness-forensics.md)).

**Stated approximations** (each a candidate divergence, all logged):
Factorio's real transport lines are continuous-position, not slotted;
inserter swings are modelled as fixed-duration rather than
distance-dependent; power is assumed present everywhere; **fluids and
pipes are out of scope for Phases 0–2** (see Phasing).

### Output: a measurement, not a verdict

```rust
pub struct MeterReport {
    pub produced_per_s: BTreeMap<String, f64>,
    pub delivered_per_s: BTreeMap<String, f64>,
    pub machine_census: BTreeMap<MachineState, usize>,
    pub machines: Vec<MachineDetail>,   // position, state, inventory
    pub belts: Vec<BeltDetail>,         // per-segment density + item counts
    pub converged: bool,
    pub ticks: u64,
}
```

No `Issue`, no `Severity`, no verdict field. Callers (a test, a gate, the
candidate search) decide what a number means. Warmup and steady-state
detection mirror `spaghettio-sim --warmup`, since buffer-fill transients
reading as convergence is an artifact class the harness already learned
the hard way.

### Integration (Phase 2, deliberately not Phase 1)

`decomposition_search.rs` already scores candidates on issue kinds and
area. Phase 2 adds a measured-throughput term. Nothing about Phase 1
presumes this lands — the meter is worth having as a CI gate even if
in-loop scoring never happens.

**Note this is the first thing in the project that can gate on measured
rate in CI.** The real harness cannot; the baselines are committed and
the meter is native and fast, so `meter_agrees_with_blessed_corpus` is an
ordinary `cargo test`.

### Rejected alternatives

- **More validator checks.** Two exist for this class already; #453 says
  three of four fixtures are still unattributed. Checks require a
  root-caused mechanism per shape; the failures are outrunning the
  analysis.
- **A rate/flow model with a "density" correction factor.** This is
  `machine_feed_rate` again — another hand-calibrated constant with the
  same drift and Goodhart exposure, and it cannot express order-dependence
  along a row at all.
- **Only speeding up `spaghettio-sim`.** The floor is Factorio's own UPS
  plus process startup. Parallelism helps throughput, not the ~35 s
  latency that keeps it out of any loop.
- **Full-fidelity Factorio reimplementation.** Unbounded, and unnecessary:
  search needs *rank correlation*, not absolute accuracy (KC1).

## Kill criteria

The calibration corpus is **frozen and committed before this RFC**: 5
baselines in `crates/sim-harness/baselines/` (tech state
`research_all_technologies;inserter_capacity<=L0;belt_stacking<=S1`) and 9
geometry-hashed entries in `crates/core/data/cell-sim-registry.json`, plus
3 measured failures recorded in issues. These numbers cannot be tuned to
because they already exist. In three bands:

| band | n | configs |
|---|---|---|
| **PASS** (at plan) | 10 | gear10, automation, logistic, military, chem5@5, AC@1, AC@2, mil5-from-plates, sulfur@2, plastic@2 |
| **MARGINAL** (−5% to −8%) | 3 | chain-ec15 @d1 (13.8/15), chain-ec15 @d7 (14.2/15), chain-ec30 (27.7/30) |
| **FAIL** | 4 | ec10 @L0 (−50%), PU@4 (−27.3%, [#437](https://github.com/storkme/spaghettio/issues/437)), USP@2 (−57.0%, [#453](https://github.com/storkme/spaghettio/issues/453)), mil5-from-ore (−28.7%) |

**KC1 — discrimination first, magnitude second.** Replay the meter over
the corpus.
- *Rank*: **zero inversions between bands.** Every FAIL must score below
  every MARGINAL, and every MARGINAL below every PASS. The MARGINAL band
  is the real test — separating −5.3% from at-plan is the resolution an
  in-loop score actually needs, and it is where a rate-shaped model will
  fail.
- *Magnitude*: meter `produced_per_s` within **±10 percentage points** of
  the real measurement on **≥80%** of the corpus.

  **If any between-band inversion survives the Phase-1 calibration
  budget, stop.** An instrument that cannot order known-good above
  known-bad cannot steer a search, and magnitude accuracy is worthless
  without it.

**KC2 — the phenomenon must emerge, not be programmed.** Two conjuncts,
both on `chain-ec15`, both with **no rule, check, or fudge written to
produce the outcome**:

- *(a) The gradient.* Reproduce the head→tail depletion and place the
  tail machine in `item_ingredient_shortage`. Deliberately **not**
  "monotone": the measured dump is `42 | 34 | 20 | 6 | 22 | 2`, which
  rises at the fifth machine. The assertion is on the head-to-tail trend
  and the terminal shortage, not on a strictly decreasing sequence — a
  meter faithfully reproducing the real measurement must pass.
- *(b) The level-invariance.* The two committed ec15 registry entries are
  the **same geometry hash** (`cde5f2fcb0f5ef21`) measured at declared
  levels d1 and d7, moving only 13.8 → 14.2. The meter must reproduce
  that near-flatness. This is the sharper of the two: every rate-shaped
  model predicts research helps, and the measurement says it does not.

If either requires special-casing, the meter is a validator in a sim
costume and delivers nothing the existing checks don't — stop.

**KC3 — speed, and it is the premise.** A 5,000-entity layout must reach
detected steady state in **≤2 s wall, single-threaded**, on the dev box.
If it cannot beat **5 s**, in-loop scoring (Phase 2) is dead and the RFC
degrades to a CI gate — a materially weaker proposition. At >5 s:
re-scope to CI-gate-only *as an explicit decision-log entry*, or stop.

**KC4 — independence, mechanically enforced.** A test greps the meter
crate's sources for the banned symbol list (`machine_feed_rate`,
`belt_drop_rate`, `lane_capacity*`, `utilization_for`,
`LANE_UTILIZATION`, `ROW_LANE_FACTOR_*`) and fails on any hit. **If
passing KC1 turns out to require importing one of them, the meter's
agreement is circular — stop and record it loudly.** This is the
Goodhart guard made mechanical rather than aspirational.

**KC5 — complexity ceiling.** If the belt + inserter + machine core
exceeds **~3,000 LOC** before KC1 passes, the fidelity target is too
ambitious for an approximation. Re-scope to a coarser model or stop.

## Verification plan

Per [`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes),
noting the protocol's own warning that a number moving is not evidence
the right thing moved.

1. **Phase 0 microbenchmark, real-sim anchored.** Before any full-layout
   work: a purpose-built single-row fixture (one belt, N inserters, one
   producer) measured in *real* Factorio across a margin sweep, then
   replayed in the meter. This tests the core hypothesis — the slotted
   belt model — in isolation, where a divergence is attributable.
2. **Corpus replay** (KC1), as `meter_agrees_with_blessed_corpus`. CI-run,
   unlike every other measured gate this project has.
3. **KC2 as a named test** — `meter_reproduces_row_tail_starvation`,
   asserting on the gradient shape, not a rate.
4. **Divergence log**, `docs/meter-divergence.md`: every fast-vs-real
   disagreement with its config and magnitude. This maps the meter's
   trusted envelope and is the artifact that keeps it honest over time. A
   log that fills with "meter missed X" is the RFC failing in slow motion
   and should be read as such.
5. **Clippy clean, no WASM impact** (the meter is a native binary and must
   not enter the WASM build).

## Phasing

- **Phase 0 — belt physics, anchored.** Entity-constant extraction; the
  slotted belt + inserter core; the margin microbenchmark above.
  **Ships value even if everything after it is killed**: it produces the
  real margin number #448 asked for and could not derive, replacing that
  check's admitted lower bound with a measured one.
- **Phase 1 — full solid meter.** Machines, splitters, undergrounds,
  boundary, convergence. KC1–KC5 evaluated here. Gate: the corpus replay.
- **Phase 2 — in-loop.** Measured-throughput term in the decomposition
  search. Gated on KC3.
- **Phase 3 — fluids.** Pipes and fluid boxes. Deliberately last: the
  solid sweep is where the sim's evidence is strongest, and the fluid
  feed path was harness-blocked until recently
  ([#364](https://github.com/storkme/spaghettio/issues/364)).

## Non-goals

- **Replacing `spaghettio-sim`.** Real Factorio remains the oracle. The
  meter is a cheap proxy whose licence to be believed is continuously
  renewed by agreement with it. If the two disagree, the meter is wrong
  by definition.
- **Emitting validation issues.** Ever. See Summary.
- **Full Factorio fidelity.** Rank correlation is the requirement.
- **Modelling power, pollution, bots, trains, or spoilage.**

## Decision log

- *2026-07-25 — drafted. Commissioned by a project-direction review that
  scored the four post-audit initiatives (sim harness, direct insertion,
  cell composition, corpus mining) and found the sim harness had created
  an unexploited asymmetry: a trustworthy slow oracle makes a cheap fast
  approximation buildable, and nobody had cashed it. Recorded there and
  restated here: the project now has **two complexity ladders** —
  validator-solved (`status.md`: tiers 1–7 SOLVED) and sim-solved
  (composed/mega chains 27–57% short) — and the ledger leads with the
  first. Sim-keying the ladder is proposed as separate, cheaper work; it
  is not in this RFC's scope.*
- *2026-07-25 — two design decisions pinned before review, as the RFC's
  whole risk surface: (a) **items, not rates** for the belt model — a
  rate model reproduces the validator's blindness by construction; (b)
  **the exported blueprint string as input**, with the engine's derived
  rate model banned by crate boundary and KC4 — because the failure this
  instrument exists to prevent (three artifacts agreeing because they
  share one assumption) is one this project has already suffered twice,
  in `carries` labels and in the backwards-inserter export bug.*
- *2026-07-25 — audit §6-D ("simulator-in-the-loop search", scored with
  **validators** as fitness) is superseded: validators are blind to the
  motivating class.*
- *2026-07-25 — review round 1 (bot, PR #455): five findings, all valid,
  all applied. Four were the draft claiming more than its own cited data
  supported, which is worth recording given this RFC is about not doing
  that. (1) "Three orders of magnitude cheaper" was contradicted by KC3's
  own ≤2 s / 5 s thresholds against a 35–55 s harness — the real figure is
  ~20×, now stated with both numbers. (2) The "20 iron plates idle beside
  it" detail is real but comes from #448's narrative, not from the dump
  excerpt quoted directly above it, which carries only cable counts; now
  attributed explicitly. (3) "Identical at every research level" was
  wrong for the **rate** (−8.0% → −6.0% → −5.3%) and right for the
  **census** — the draft conflated them; corrected, and the d1→d2
  input-side bind is now separated from the flat d2→d7 tail. (4) **KC2(a)
  required a shape the real data does not have** — the measured dump
  `42 | 34 | 20 | 6 | 22 | 2` rises at the fifth machine, so a meter
  faithfully reproducing it would have FAILED the kill criterion as
  originally worded. "Monotone" dropped; this was the most serious of the
  five, since an unsatisfiable kill criterion is worse than none. (5) Two
  stray tool-call closing tags were committed at end-of-file; removed.*
- *2026-07-25 — **ACCEPTED** (user), merged as PR #455. Implementation
  tracked in [#457](https://github.com/storkme/spaghettio/issues/457),
  split into four PRs rather than one so that PR 2 (the anchored margin
  sweep) is a designed kill point for the belt model at ~1k LOC rather
  than after machines, boundary and convergence are built on top of it.*
- *2026-07-25 — **PR 1 landed the physics core, and it does NOT reproduce
  #448.** This is the RFC's first real result and it is negative, so it
  belongs here and not only in a PR body.*

  *What works — every belt-level property the design claimed. Gaps do not
  heal on a moving lane, dead ends back up, compressed lanes move at full
  speed, and both belt throughput (B5: 15/30/45) and inserter rates (I8:
  0.84/1.20/2.40) are **derived** from speed × spacing and `rotation_speed`
  rather than read from a table. Partial hands and lost swings make
  density-dependent inserter throughput emergent, with no
  `machine_feed_rate` anywhere. KC4 is green and was guarded from the
  first commit, not retrofitted.*

  *What does not — with a **smooth** boundary supply at exactly aggregate
  demand and bounded consumer buffers, a 6-consumer express row delivers
  `[7.50 × 6]`. No tail starvation. The conservation intuition simply
  holds in that configuration. Worse, a margin sweep is **non-monotonic**:
  margin 1.02 starves where 1.00 does not, recovering by 1.25
  (`cargo run -p spaghettio_meter --example row_probe`). In a fully
  deterministic simulator with periodic sources and periodic inserter
  swings, that is the signature of **phase aliasing** between the two
  cadences, not a physical effect.*

  *Two modelling bugs were caught by the crate's own derivation tests
  before any of that: the inserter cycle lost a tick to the grab and
  `round()`ed its half-cycles, under-crediting a fast inserter by ~8%
  (2.222/s against I8's 2.40); and an unbounded consumer let a row's head
  pull 16.2/s against a 7.5/s demand, which both overstated head-hogging
  and made added margin actively **worse**. The second is why `Chest`
  gained a buffer cap and a demand rate — a machine's input side, minus
  the crafting.*

  *Deliberately NOT done: adding burstiness to the supply. A real
  row-input belt is fed by a producer cell's output inserters — discrete,
  bursty drops — and real consumers draw in craft batches; adding either
  would plausibly produce the starvation. That is exactly why it must not
  be added before PR 2's anchor exists. Choosing mechanisms until the
  answer matches the expected one is how an instrument acquires the quirks
  it was built to detect, and this RFC's whole integrity argument is about
  not doing that. The negative result is pinned by
  `smooth_supply_at_zero_margin_does_not_starve_the_row` so that a later
  change cannot make the row starve silently.*

  *Consequence for PR 2: its first question is no longer "what is the
  margin number" but **"does real Factorio starve this configuration at
  all"**. If it does, the belt model is missing something and the sweep
  attributes it. If it does not, then #448's zero-margin attribution needs
  revisiting — which would connect directly to
  [#453](https://github.com/storkme/spaghettio/issues/453)'s finding that
  three of the four failing fixtures starve **with margin available**, and
  are still unattributed.*
- *2026-07-25 — PR 1 review (bot, PR #458): two findings, both valid, both
  applied — and the second one's **hypothesis was tested and falsified**,
  which is recorded here because the distinction matters.*

  *(1) **Inserter cycle timer was assigned, not accumulated.** `cycle_timer
  = cycle_ticks()` discarded the previous cycle's negative overshoot,
  quantising the period up to `ceil(cycle_ticks)` — 72 ticks instead of
  71.43 for a regular inserter, i.e. 0.8333/s against I8's 0.84/s, a
  systematic −0.79%. Every other periodic accumulator in the crate
  (`Lane::tick`, `Source::tick`, `Chest::tick`) carries its remainder
  forward; this was the one that didn't, while its own doc comment claimed
  the timer was "never rounded". Fixed to `+=`. The test that exists to
  catch exactly this had a **5% tolerance**, which is why it didn't —
  tightened to 1%.*

  *(2) **`Chest::accept` was all-or-nothing.** It rejected an entire hand
  whenever the hand did not wholly fit, where a real Factorio inserter
  performs a partial insert — transferring what fits and retaining the
  remainder, stalling fully only when nothing fits. Fixed: `accept` now
  drains what fits and returns the count, and `Inserter::tick` holds the
  remainder and retries.*

  ***The reviewer's attached hypothesis — that this all-or-nothing rule was
  "a plausible undisclosed contributor to the non-monotonic
  starvation-vs-margin behavior" — is FALSIFIED.*** *Re-running the margin
  probe after the fix gives rates identical to the pre-fix run at every
  margin (1.02 → 5.50/6.20; 1.05 → 7.12/5.25/7.13; 1.10 → 6.00). Only the
  buffers moved — they now top up to 39 rather than stalling at 32–37,
  which confirms the fix genuinely changed behaviour and that the
  non-monotonicity is not caused by it. The phase-aliasing reading stands,
  and PR 2's anchored sweep remains the way to settle it.*

  *Worth stating plainly since this RFC is about instruments that do not
  launder assumptions: a correct fix arriving with a plausible causal story
  attached is not evidence for the story. The fix was applied on its own
  merits (it is what the game does); the story was checked separately and
  did not survive.*
- *2026-07-25 — PR 1 review round 2 (bot, PR #458): one finding, valid,
  applied — and again its **impact claim was falsified by measurement**.
  Recording the pattern, not just the fix.*

  *The finding: `InserterKind::hand_size` mis-transcribed I8b on both
  branches. Bulk read `2,4,5,6,7,9,11,12` against I8b's
  `2,3,4,5,6,8,10,12` (L1–L6 each over-credited by +1), and the non-bulk
  closed form `1 + level.saturating_sub(2).min(2)` evaluated to
  `1,1,1,2,3,3,3,3` against I8b's `1,1,2,2,2,2,2,3` (L2 under, L4–L6
  over). Verified against `factorio-mechanics.md` directly before
  applying. Both ladders are now **literal tables** — neither is
  expressible as a clean closed form, and deriving them is what produced
  the error. Transcribe, don't derive.*

  *The test was the real failure. It asserted L0/L2/L7 only; the endpoints
  happened to be correct, so it **pinned a mis-transcribed middle rather
  than catching it** — and its one middle assertion
  (`Stack.hand_size(2) == 9`) was itself the wrong value. Replaced with an
  exhaustive level-by-level ladder assertion. Sampling a table is not
  testing it, and this is the second constants-defect in two rounds that a
  loose test let through (the first being the 5% tolerance over the cycle
  timer).*

  ***The load-bearing claim is FALSIFIED.*** *The reviewer argued the wrong
  value changes the headline, since `RowFixture` runs at `capacity_level =
  2` where stack should be 8 rather than 9. Re-running the sweep after the
  correction gives byte-identical rates at every margin. The reason is
  structural, not luck: a single belt tile holds at most **8** items (4
  slots × 2 lanes), so `take_from_tile` caps any grab at 8 — hands of 8
  and 9 saturate the same ceiling. The reviewer's supporting argument
  (that 8 is where the BS3 S=4 belt-drop dip vanishes) also does not
  apply here: the fixture runs unstacked at S=1, and BS3 governs drops
  *onto belts*, whereas these inserters drop into consumers.*

  *Pattern worth naming after two rounds: the bot's defect identification
  has been accurate every time (three real bugs, all applied), and its
  causal attribution to the headline finding has been wrong every time
  (two for two). Both fixes were taken on their own merits; neither
  explanation survived being checked. The non-monotonicity remains
  unattributed and remains PR 2's first question.*
- *2026-07-25 — **PR 3: the meter runs a real factory, and on the EC family
  it agrees with Factorio to within ~0.4 percentage points.** Machines,
  blueprint ingestion, boundary handling and `MeterReport` landed; the
  first end-to-end measurements are below. This supersedes PR 1's negative
  result as the RFC's best evidence, and it arrived without Factorio —
  fixtures are generated locally by the engine and compared against
  measurements already frozen in the repo, which is exactly the property
  the "Why now, and not before RFC-050" section rested on.*

  | config | real Factorio | meter | Δ |
  |---|---|---|---|
  | chain-ec15 @d1 | −8.0% | −7.7% | 0.3pp |
  | chain-ec15 @d2 | −6.0% | −5.6% | 0.4pp |
  | chain-ec15 @d7 | −5.3% | −5.6% | 0.3pp |
  | chain-ec30 @d2 | −5.3% | −5.6% | 0.3pp |

  *(**Superseded for the d1 row** by the 2026-07-25 correction at the end of
  this log: d1 now reads −7.4%, Δ 0.6pp. The other three rows still hold.
  Left as recorded rather than edited in place — this log is a history, and
  the drift is the interesting part.)*

  ***KC2(b) is met on this family***: the meter reproduces the
  **level-invariance** unprogrammed — d2 through d7 sit flat at −5.6%
  while d1 is worse at −7.7%, the same shape #435 measured and the same
  shape every rate-based model predicts wrongly. Nothing in the code knows
  about research levels except the hand-size table.*

  ***KC3 looks comfortable***: `chain-mil5ore` — 3,754 belt tiles, 146
  machines, 360 inserters — simulates 18,000 ticks in **0.34 s wall**.
  KC3's bar is ≤2 s at 5k entities.*

  ***The first end-to-end run immediately found a modelling error, and it
  was the one that mattered most.*** *The network drained ANY tile with no
  downstream, so items fell off interior belt ends instead of backing up.
  Symptom: copper-cable read 45.00/45.00 exactly at plan while
  electronic-circuit sat at **−57.8%**. Only manifest-designated boundary
  outputs may drain; an unlinked interior tile is a **dead end** and must
  hold. That backpressure is precisely the mechanism #448 turns on — the
  bug would have deleted the phenomenon the meter exists to measure. Fixed,
  and EC moved −57.8% → −5.6%.*

  **Known disagreements, both under-reporting:**
  - *`chain-ac1-d0`: meter −42.8% against a real **PASS** at −0.3%. This is
    the disclosed PR-3 fluid limitation doing its job — fluid-fed machines
    are held in shortage rather than allowed to craft from nothing, so an
    on-site plastic-bar step starves the chain. Honest under-report, not a
    silent wrong number. Resolves when fluids land.*
  - *`chain-mil5ore`: meter −66.2% against a real −28.7%. **Unexplained.**
    Its solid intermediates (iron-plate −0.0%, stone-brick 0.0%) are at
    plan while the pack itself is not, so the loss is downstream of
    smelting. First candidate for the divergence log.*

  *Not yet claimed: KC1 proper. That needs the full 17-config replay as a
  test with the frozen baselines as fixtures, which is PR 4. Four configs
  of one family agreeing is encouraging, not a verdict — and two other
  families disagree.*
- *2026-07-25 — **PR 4: KC1 EVALUATED. IT TRIPS.** Reporting the trip
  rather than re-scoping around it, because rewriting a kill criterion
  after seeing it fire is the exact failure kill criteria exist to prevent
  (cf. RFC-053's KC6 disclosure).*

  | config | band | real | meter | gap |
  |---|---|---|---|---|
  | chain-ec15-d1 | Marginal | −8.0% | −7.7% | **0.3pp** |
  | chain-ec15-d7 | Marginal | −5.3% | −5.6% | **0.3pp** |
  | chain-ec30-d2 | Marginal | −5.3% | −5.6% | **0.3pp** |
  | chain-mil5plates-d0 | Pass | −3.3% | −61.1% | **57.8pp** |
  | chain-mil5ore-d2 | Fail | −28.7% | −66.2% | **37.5pp** |
  | *fluid-dependent (7)* | — | — | mostly −100% | 43–110pp |

  ***Verdict: both halves of KC1 fail on solid chains.*** *Rank — three
  inversions, all from `chain-mil5plates-d0`, a real-measured PASS the
  meter puts at −61.1%, ranking it below every Marginal EC config.
  Magnitude — 3/5 within 10pp against a bar of 4/5.*

  **Two distinct causes, and only one is excused:**

  1. ***Fluids (excused, phased).*** *Every `mega-*` chain reads −100%: the
     meter holds fluid-fed machines in shortage, so an early fluid step
     stops everything downstream. This is the RFC's own Phase 3 boundary
     behaving as designed — under-report honestly, never over-report. But
     it exposes a **defect in this RFC's plan**: KC1's corpus is majority
     fluid-dependent (7 of 12 reachable), and the phasing put KC1's
     evaluation before fluids existed. The criterion and the phase order
     were mutually inconsistent from the start. Recorded as a planning
     error, not discovered as a surprise.*
  2. ***The military family (NOT excused).*** `chain-mil5plates` *is a
     **solid** chain measured PASS in game and −61.1% here, and it alone
     causes every rank inversion.* `chain-mil5ore` *is 37.5pp off in the
     same direction. Fluids explain neither. This is a genuine defect in
     the belt/inserter/machine model, unattributed, and it is the thing
     that must be fixed before KC1 can be re-evaluated honestly.*

  ***What survives, and it is not nothing.*** *On the EC family — the
  MARGINAL band, which the RFC named as "the real test... where a
  rate-shaped model will fail" — agreement is **0.3pp across all three
  configs**, and the level-invariance is reproduced unprogrammed. That is
  the hardest discrimination KC1 asks for, and the meter does it. The
  instrument is not wrong everywhere; it is wrong somewhere specific.*

  *The two gate tests stay in the tree, exact, marked `#[ignore]` with the
  trip documented at the assertion. Not deleted (that is rewriting),
  not loosened (same), and not left red in the default suite (that trains
  people to ignore red). Runnable on demand.*

  ***Recommended next step is attribution, not more building***: find why
  the military family under-reports. It is solid, it is small enough to
  dump per-machine, and the answer decides whether the belt model has a
  real hole or the fixture wiring does. Building Phase 3 fluids on top of
  an unexplained 57.8pp error would be exactly the "exploration that
  overruns its evidence" this project names as its dominant rework shape.*
- *2026-07-25 — **KC1 attribution, round 1: one real defect found and
  fixed; the military deficit narrowed but NOT closed.** Recording a
  partial result rather than a clean one, because the remaining gap is the
  thing that matters and it is still open.*

  ***Found and fixed: inserters grabbed blind (mechanics I11).*** *The
  meter's inserters took whatever was under the hand without checking
  whether the destination would accept it. On a mixed belt the first
  foreign item jams the hand permanently — `insert` returns 0, the hand
  never empties, and the inserter stops forever. Real inserters check the
  destination before swinging; **I11** says so explicitly ("inserters
  refuse items the destination can't accept"), and the same mechanic is
  what plugged the sim harness's own feed rigs in the #357 forensics.
  `take_from_tile_filtered` now applies the destination's `room_for` as a
  pickup predicate. **Effect: mil5plates −61.1% → −59.6%.** Real, correct,
  and nowhere near sufficient — which is worth stating plainly, because a
  fix that moves 1.5pp against a 56pp gap is not the explanation.*

  ***Where the remaining defect lives, narrowed by measurement:*** *the
  grenade row of `chain-mil5plates`. Per-machine dump after warmup:*

  ```
  grenade at (35,7)  iron-plate=70/5  coal=9/10     <- iron buffer CAPPED
  grenade at (38,7)  iron-plate=70/5  coal=9/10
  ...                                    8,8,7,7,6,6
  grenade at (62,7)  iron-plate=70/5  coal=5/10     <- declining head->tail
  ```

  *Boundary injection against plan: stone-brick **25.00/25.0 exact**, coal
  **13.45/25.0**, iron 15.69/22.5. So one input arrives at plan and coal
  does not — while the coal belt reads full on **78% of ticks**. Coal is
  on the belt, near the head, and not reaching the tail. Iron backing up to
  its cap is a *consequence* (grenades cannot craft without coal), not a
  second fault.*

  *That is the #448 signature — head-full, tail-starved, monotone gradient
  — but far more severe than the game shows for the same layout. So the
  meter is not inventing the phenomenon; it is **over-applying** something.
  Remaining suspects, in order: the belt-drop model
  (`drop_onto_tile` places on the far lane via `try_insert_anywhere`), the
  tap-off splitter, and the sideload lane restriction (**B8**) firing where
  the game would curve (**B11**).*

  ***Not chased further this session.*** *The honest state is: KC1 remains
  tripped, the cause is localized to one row's coal delivery on one
  fixture, and the next step is a tile-level dump of that row's belt
  occupancy rather than more model-building. Recorded here so a cold
  pick-up starts from the measurement, not from the theory.*
- *2026-07-25 — **KC1 attribution, round 2: the leading hypothesis is
  FALSIFIED.** Belt→machine inserter rate explains ~20pp of the 56pp
  military gap and then plateaus. Something else binds.*

  ***The hypothesis, and why it was the obvious one.*** *The grenade row
  needs 10 coal per craft over 6.4 s = **1.5625 coal/s per machine**, fed
  by exactly **one regular inserter** (the long-handed one on each machine
  reaches past it to the iron belt). The meter rates a regular inserter at
  I8's **0.84/s** — a 1.86× shortfall. And I8 states it is a
  **chest-to-chest** figure and self-flags "actual throughput varies with
  pickup/drop distance and belt speed", while RFC-049 Phase 2 **measured**
  belt→machine intake and found `machine_feed_rate` credits it with
  **1.04–2.27× margins** over the naive number. 1.86× sits squarely inside
  that measured band. Everything lined up.*

  ***The test, and the answer.*** *A throwaway env-gated multiplier on the
  swing cycle (diagnostic only — reverted, never committed as a model):*

  ```
  swing ×1.0  → −59.6%     swing ×1.86 → −40.4%
  swing ×1.5  → −41.8%     swing ×2.2  → −38.7%
  ```

  *It **plateaus near −39%**. At 2.2× — beyond the top of the measured
  margin band — the deficit is still 36pp away from the real −3.3%.
  Inserter swing rate is a real contributor and **not** the binding
  constraint.*

  *Recorded as a falsification rather than a fix because the arithmetic was
  persuasive enough that it would have been easy to apply a
  belt→machine correction, watch mil5plates improve 20pp, and call it
  solved. The improvement is real and the explanation is still wrong. This
  is the third time in this RFC that a plausible causal story attached to a
  correct-looking change did not survive being checked (twice from the PR-1
  review, once here) — the pattern is now worth naming as a standing habit
  rather than a coincidence: **fix on merit, test the story separately.***

  ***What is established about the residual:*** *the coal belt is
  **fully compressed (4/4 both lanes) along its entire length**, with one
  picker per machine, verified by a tile-level downstream walk
  (`--example trace_belt`). Supply and topology are not the problem.
  Machines starve with a monotone head→tail coal gradient (9,9,8,8,7,7,6,
  6,5,5) while iron sits capped at 70/70. Faster inserters do not close it.
  So the bind is in how much of a compressed belt one inserter can actually
  claim — remaining suspects: `drop_onto_tile`'s far-lane placement via
  `try_insert_anywhere`, and whether **I6** (pickup draws from BOTH lanes)
  is being honoured in the take path when the first lane is exhausted.*

  ***Tooling added en route***, both reusable: `--example attribute`
  (per-recipe census, starved machines with have/need per ingredient,
  boundary injection vs plan, inserter wiring census) and
  `--example trace_belt` (walk a path downstream from a boundary feed,
  per-tile lane occupancy, pickers/droppers/sinks annotated). These are the
  meter's equivalent of `scripts/sim-capture-state.sh` and are how both
  rounds of attribution were done.*
- *2026-07-25 — **KC1 attribution, round 3: buffering exonerated, and a real
  ingestion bug found underneath it.** The user's hypothesis — that the
  starvation was an artifact of machine input buffering — was tested
  directly and is **wrong**; chasing it nonetheless led to the defect.*

  ***Buffering, tested and cleared.*** `DEFAULT_BUFFER_CRAFTS = 14` is a
  stated approximation of Factorio's ingredient-slot cap, so it was a fair
  suspect. *An env-gated override (diagnostic, reverted — same discipline as
  the swing probe) swept it across 1, 2, 4, 14 and 40 crafts on
  `logistic-science-pack`. Every value returned **−100.0%**, byte-identical.
  A quantity that changes nothing across a 40× range is not the binding
  constraint. Buffer depth governs how long a machine rides out a supply
  gap; it cannot manufacture supply that never arrives.*

  ***What the probe showed instead.*** *Dumping each starved machine's input
  inserters with their pickup-tile contents split the population cleanly by
  **reach**, not by item or by row:*

  ```
  iron-gear-wheel (15,31):
    Fast       (16,30) <- belt (16,29)  occ 0/8  []            delivered 0
  transport-belt (15,41):
    LongHanded (15,40) <- belt (15,38)  occ 8/8  [iron-plate]  delivered 14
    Regular    (17,40) <- belt (17,39)  occ 0/8  []            delivered 0
  ```

  *Every reach-2 pickup is saturated; every reach-1 pickup is empty for the
  full 18,000 ticks. Reach-2 hands land on the **trunk**; reach-1 hands land
  on the **tap-off branch** beside it. So the branches were never receiving
  anything — an upstream-linkage question, not a rate question. An orphan-head
  check (belt tiles with no upstream tile, no dropping inserter, and no
  boundary feed) confirmed it: **11 of 519** tiles, clustered on the gear
  machine.*

  ***Root cause: splitter second-cell sign error in the network builder.***
  *A splitter occupies two tiles. `NetworkBuilder` derived the second from
  `left_of(direction).delta()`, which **flips sign between north and south**,
  so SOUTH and WEST splitters registered a cell one tile outside their own
  footprint — creating a phantom tile with no upstream while the tile the
  splitter actually occupies never entered the network at all. Belts feeding
  into it linked to nothing and everything downstream became an orphan head.
  Bus trunks run **south**, so this silently unlinked tap-off branches across
  essentially every bus layout, and was invisible on the cell-composition
  fixtures (north/east) that the meter had been developed against. Fixed to
  `(x+1,y)` / `(x,y+1)` off the decoded top-left, which is unconditional
  because `blueprint_in::decode` already resolves the oriented footprint.*

  ***Effect.*** `logistic-science-pack` **−100% → −68.3%**; `iron-gear-wheel`
  0 → 1.08/1.50 crafts/s. *`gear10` (−0.1%) and `ec10` (−3.8%) are unchanged
  to the digit, as expected — neither contains a south- or west-facing
  splitter. So the fix is not a global rate shift dressed up as a repair.*

  ***Pinned by two tests, not one.*** *A geometry test asserts both cells in
  **all four directions** — a single-case test would have passed throughout
  the bug's life, since north and east were always right. A behavioural test
  asserts a belt links through a south-facing splitter and that both outputs
  have upstream feeders, which is the shape the defect actually took.*

  ***Standing habit held.*** *The user proposed buffering; it was tested
  rather than assumed, and reported as falsified. Fourth instance in this RFC
  of a plausible causal story failing its check — but the first where the
  wrong theory still routed to the right defect, because testing it required
  looking at the machines that were starving.*

  ***KC1 remains tripped.*** *The military family is unmoved: mil5plates
  −59.6%, mil5ore −64.0% against real −3.3% / −28.7%. Those layouts starve on
  coal along a compressed trunk, which round 2 established is a different
  mechanism from the unlinked-branch defect fixed here. Logistic at −68.3%
  is now a plausible-shaped failure rather than a total one, but it is still
  a failure.*
- *2026-07-25 — **Session-side adversarial review (the CI bot abandoned
  twice), and it caught a stale number in this very log.** PR #460's
  `claude-review` run failed the silent-no-op guard on both attempts:
  run 1 at 116s / 11 turns / $1.72 / 3 denials, run 2 at 21 turns — the
  class-5/6 mid-review abandonment cluster in `docs/review-bot.md`, not a
  code problem and not a conscious skip. Re-rolling twice was the documented
  escape hatch; a third roll would have been superstition, so the review was
  done session-side instead. Three findings, all in code this RFC added:*

  ***1. A headline number in this log had gone stale, and nobody noticed.***
  `chain-ec15 @d1` *was recorded at −7.7% in the PR-3 entry. It now measures
  **−7.4%** — Δ against real Factorio is **0.6pp, not 0.3pp**, and the
  RFC's "agrees to within ~0.4pp" becomes **0.3–0.6pp**. The drift happened
  mid-branch (the I11 filtered-take fix or the splitter fix; both touched
  the EC path) and was never re-measured, because each fix was verified
  against the fixtures it was expected to move. The other three rows are
  unchanged. Caught only by re-running the full corpus while checking
  something else — which is the argument for the corpus replay being a
  **test**, not an example you run when you remember to.*

  ***2. `drain_sinks` carried two representations of one fact.*** *A `sinks:
  FxHashSet<usize>` on `Factory` duplicated `BeltTile::is_sink`, and the
  drain read `if sinks.is_empty() || sinks.contains(&tile)`. Both arms were
  unreachable: `exited_log` is only ever appended from the two `is_sink`
  arms in `BeltNetwork`, so every entry is already a declared boundary
  output, and with no sinks the log stays empty. Harmless today, but the
  `is_empty()` branch reads like a deliberate "count everything when the
  manifest declares no outputs" fallback that a later reader would trust.
  Removed the set; the drain now counts what left, with the invariant
  stated. **Verified behaviour-preserving by measurement, not by argument**
  — the whole corpus is byte-identical across the change, checked by
  stashing and re-running rather than by reasoning about it.*

  ***3. The known-open topology allowlist was looser than its own comment.***
  `KNOWN_OPEN` *matched with a bare* `starts_with(label)`*, so a future
  fixture named* `military-science-pack-large` *would have been silently
  allowlisted — the opposite of the "this list can only shrink" property the
  comment claims. Tightened to match the full* `label:` *prefix.*

  *Worth stating plainly: an author reviewing their own diff is the weakest
  form of the repo's adversarial-review rule, and finding #1 is precisely
  the kind of thing an independent reviewer is better placed to catch. It
  was found here by re-running the numbers, not by reading the code.*
- *2026-07-25 — **The review bot came back on the third run and found four
  real defects, three of which no fixture would ever have caught.** After
  two abandonments it completed and posted inline. Every finding was valid;
  all four are fixed. This is the strongest argument yet for the bot being
  worth its failure rate — the session-side review that preceded it found
  three issues and **missed all four of these**.*

  ***1. Unguarded sentinel index — a live panic.*** `TileKind::Splitter`
  *carried* `partner: usize` *initialised to* `usize::MAX`*, patched only
  when both halves were placed.* `step_splitter_exit` *indexed it
  unconditionally, so the first tick over an orphan half panicked.
  Reproduced with a three-entity fixture before fixing. Now
  `partner: Option<usize>`, and an unpaired half degrades to plain-belt
  behaviour — closer to the truth than either panicking or dropping items,
  and the* `OrphanSplitterHalf` *note already warns that its rates are
  suspect. The sentinel was the defect; making it an* `Option` *makes the
  case unrepresentable rather than merely handled.*

  ***2. Long-handed inserters dropped on the wrong lane.*** `near_lane_from`
  *decided the near lane by* **exact tile equality** *against the single
  tile one step to the left. That can only ever match a reach-1 hand; a
  long-handed inserter stands two tiles away, so the test failed
  unconditionally, `near` came back 1 every time, and `far = 1 - near` put
  every reach-2 drop on lane 0 whichever side it came from. Now decided by
  the sign of the perpendicular projection, which holds at any distance and
  reproduces the old answers exactly at distance 1 (so the belt-to-belt
  sideload caller is untouched).*

  ***The corpus does not move — and that is the finding, not an excuse.***
  *Every headline number is identical after the fix: ec15 d1/d2/d7
  −7.4/−5.6/−5.6, ec30-d2 −5.6, logistic −68.3, mil5plates −59.6, mil5ore
  −64.0. The bot predicted exactly this ("corrupts lane-sensitive behaviour
  ... even though aggregate throughput often survives") and was right. A
  wrong lane assignment that leaves throughput unchanged is invisible to
  every aggregate check the meter has; it would have surfaced only as an
  inexplicable sideload or splitter result much later. **The regression test
  was verified against the old code** — it fails at* `dist 2` *and passes at*
  `dist 1`*, the bug's exact shape. An unverified regression test is not one.*

  ***3. Probabilistic products over-credited while claiming expectation.***
  `(amount * probability).round().max(1.0)` *credited a whole unit per craft
  no matter how small the probability — 4× for a p=0.25 recycling product,
  ~143× for uranium-235 at p=0.007 — directly above a comment asserting
  "credited at expectation". Products are now* `f64` *expectations with a
  per-product fractional accumulator that emits whole units as the carry
  crosses 1.0, so the long-run rate is the expectation and nothing invents a
  fraction of an item. (Simply dropping* `.max(1.0)` *would have credited 0
  forever for anything under 0.5 — the bot flagged that too.) No corpus
  fixture uses a probabilistic recipe, which is exactly why it survived: the
  meter's own tests cannot catch what its fixtures never exercise.*

  ***4. A vacuous assertion in the KC1 gate.*** `meter = got/planned − 1`
  *with* `got ≥ 0` *and* `planned > 0` *is bounded below by exactly −1.0, so
  the sanity check* `meter > −1.001` **could not fail for any input** *—
  including the "produced literally nothing" case it claimed to catch.
  Tightened to −0.999 and scoped to solids: the fluid chains genuinely sit
  at −100% because fluids are unimplemented, so asserting over them would
  red the suite for a documented gap. Their count is still printed.*

  ***The pattern from earlier in this log now has a counterexample worth
  recording.*** *Twice before, the bot's defect identification was right and
  its causal attribution wrong. This round, three of four came with
  explanations that survived checking, and finding 2's prediction about
  aggregate throughput was confirmed exactly. The habit stands — fix on
  merit, test the story separately — but "the bot's stories are unreliable"
  would be the wrong generalisation to carry forward.*

  ***And the uncomfortable one:*** *findings 1–4 are all in code the
  session-side review had just read and pronounced sound. Author
  self-review found the stale number and two readability traps; it did not
  find a reachable panic, an inverted lane, a 143× over-credit, or a
  tautological assertion. That is the case for the independent reviewer,
  made concrete.*
- *2026-07-25 — **Two more from the bot's next pass, and the first one is a
  bug the previous round's fix introduced.** Both valid, both fixed.*

  ***1. The expectation fix leaked one layer up.*** *Making* `Machine::products`
  *fractional was right, but* `Factory::tick_machines` *credited* `crafted`
  *by re-deriving from that same vector —* `*amount as u64` *— which
  truncates 0.25 to 0 and loses a third of a 1.5. The machine's own carry
  was correct, so* `produced_per_s` *and belt-delivered throughput, the two
  halves of one* `MeterReport`*, would silently disagree. Exactly the defect
  the carry was added to kill, reintroduced one level away from it, in the
  same commit.*

  *Fixed at the root rather than with a second accumulator:* `Machine::tick`
  *now records* `emitted_this_tick` *— the whole units it actually pushed —
  and the factory credits those. One carry, one source of truth.* `products`
  ***is now private***, *so nothing outside* `machine.rs` *can re-derive
  production from expectations again. Same move as the* `Option<usize>`
  *fix: prefer making the mistake unrepresentable over fixing this instance
  of it.*

  ***The lesson is about the shape of the fix, not the arithmetic.*** *A
  correct change to a data representation is not finished until every
  consumer of that representation has been re-read. The compiler was no
  help —* `f64 as u64` *is a legal lossy cast — and the corpus was no help
  either, because every fixture's products are integers with probability
  1.0, where the truncation is exact. Silent under the type system and
  silent under the tests.*

  ***2. A doc comment that described a different function.*** `footprint`'s
  *docs claimed in bold that unknown names are an error, not a 1×1 default;
  the body is* `footprint_checked(name).unwrap_or((1, 1))`*. The claim was
  true of* `footprint_checked` *and false of the function carrying it.
  Unreachable today — both callers gate on* `footprint_checked` *first — but
  a future caller trusting the comment would have got exactly the silent
  wrong-tile placement it warned about. The bold paragraph moved to the
  function it actually describes;* `footprint` *now documents its fallback
  honestly.*

  ***Running tally for this PR: the bot has found six defects across two
  completed runs, the session-side review three.*** *No overlap between the
  two sets. The bot found the panic, the inverted lane, the over-credit, the
  tautology, the truncation and the false doc; the self-review found a stale
  measurement, a dead branch and a loose allowlist. Different failure modes,
  and neither list is a subset of the other — which is a better argument for
  running both than for preferring either.*
- *2026-07-25 — **Merged into a pending review check, and it cost four
  findings.** The* `claude-review` *run on the final commit was still going at
  25 minutes, past any signature in* `docs/review-bot.md` *(a completed review
  is ~8 min). It was judged hung and PR #460 was merged with it pending. It
  was not hung: it completed **success** and posted four inline findings
  minutes later, against code that was by then on* `main`*.*

  ***The reasoning was wrong in a specific way worth naming.*** *The evidence
  said the run was **unusual** — longer than anything previously recorded. It
  was read as saying the run was **dead**. Those are different claims, and the
  supporting argument ("the delta since its last completed review is one
  markdown file") was irrelevant: the bot reviews the **whole PR diff**, not
  the incremental delta, so a fresh run always has all the code to look at.
  The argument justified a conclusion it did not actually support.*

  ***Finding 1, fixed here: `Machine::tick` assigned craft progress instead of
  accumulating it.*** `self.progress = self.craft_ticks` *discards the
  overshoot from the craft that just finished, quantising the effective period
  to* `ceil(craft_ticks)` *— a 4.5-tick recipe runs a 5-tick cycle, −10%
  throughput, silently.*

  ***This is the same defect fixed in `inserter.rs` earlier in this same
  RFC***, *whose comment already states the rule and names* `Lane::tick`,
  `Source::tick` *and* `Chest::tick` *as following it.* `Machine` *was the one
  place that did not. A repeated-mistake sweep after the inserter fix would
  have found it; none was done. The lesson generalises past this instance:
  **when a bug class is identified, grep for the class, not the instance.***

  ***And the story does not hold.*** *Corpus is **byte-identical** after the
  fix — every corpus recipe has integer* `craft_ticks` *(24/48/384/480 at AM3
  speed 1.25; plates smelt at furnace speed 2.0 → 96), so the quantisation
  never bites. It would have been convenient for this to be part of the KC1
  gap. It is not. Fixed on merit; the explanation is tested and negative.*

  *Remaining three from that run — U7 (sideload onto a UG **input** fills the
  FAR lane, a documented "critical quirk", with no* `UgInput` *carve-out in*
  `link_downstream`*), silently-dropped unrecognised inserters (no* `notes`
  *entry, unlike every other skip path), and* `converged` *hardcoded true with
  no detector — are open and tracked in this branch.*
- *2026-07-26 — **KC1's trip is EXPLAINED, and neither cause was in the
  meter. Both were in how it was measured.** Two independent defects, each
  sufficient on its own to produce the observed failure; between them they
  account for the entire military gap and invalidate the earlier
  attribution work. This entry supersedes attribution rounds 1–3.*

  ***Cause 1: the corpus fixtures were built at the wrong geometry.***
  `export_chain_fixtures_for_sim` *composed via bare* `compose_chain(&sr)`
  *— the ambient engine default — then stamped* `inserter_capacity = lvl`
  *afterwards, which sets only the DECLARED harness world. Until #431 the
  chain path hardcoded L0, so the stamp was the whole story and the code
  was correct; its doc comment said so ("the declaration changes zero
  geometry pre-#381"). #431 moved the default to L2 on 2026-07-24, one day
  before these fixtures were regenerated for this RFC, and silently began
  exporting **L2 geometry under an L0 label**: inserters placed for L2
  capacity bonuses, then run by the harness in an L0 world.*

  *Measured on* `chain-mil5plates-d0`*, same manifest, same tech state
  (`nb=0 bulk=1, S=1`), both Factorio 2.0.76 and 2.0.77:*

  | geometry | entities | real Factorio |
  |---|---|---|
  | L2 (what was exported) | 1180 | −40.7% FAIL, 29/46 working |
  | L0 (blessed) | 1182 | **−3.3% PASS, 46/46 working** |

  ***So KC1 compared meter-on-fixture-A against Factorio-on-fixture-B.***
  *The comparison was never apples-to-apples — which invalidates the
  celebrated agreements as much as the failure. The EC family's "0.3–0.6pp"
  was between two different factories and was **coincidence, not
  evidence**. All five chain fixtures were affected; the L0 hashes are the
  registered ones in every case. Fixed in #466, with a gate
  (`chain_fixture_geometry_matches_registry`) watching the exporter path
  rather than the composer path —* `cell_registry_hashes_current` *stayed
  green throughout precisely because it re-derives through a different code
  path than the one writing the bytes. Two paths to one artifact, one of
  them checked.*

  ***The baselines were never stale.*** *An earlier reading of this session
  was that the −3.3% baseline no longer reproduced. It reproduces exactly —
  46/46 working, delivered 5.00/s — once the fixture is built correctly.
  Recorded because the wrong conclusion was stated before it was checked.*

  ***Cause 2: the measurements had not converged.*** *With the fixture
  corrected, the military gap narrowed to ~35pp and moved to a new place:
  four tail machines on the MSP row starved of piercing-rounds-magazine.
  That is the #448 signature and it was tempting to treat as the real
  defect. It is not. Sweeping warmup on* `chain-mil5plates-d0`*:*

  | warmup | meter | converged |
  |---|---|---|
  | 2 game-min (the corpus default) | −38.4% | false |
  | 5 | −10.2% | false |
  | 10 | −1.1% | **true** |
  | 20 / 40 / 80 | +0.7% | true |

  ***The corpus replay warms up for two game-minutes.*** *A 46-machine
  chain with deep ingredient buffers is nowhere near steady state by then.
  The whole military deficit was a buffer-fill transient being read as a
  rate.*

  ***And the same is true of real Factorio.*** `chain-mil5ore-d2` *is
  recorded in this RFC's own frozen corpus as a **FAIL at −28.7%**. Re-run
  unchanged at* `--warmup 288000` *(80 game-minutes) it measures **+0.7%,
  146/146 machines working, PASS**. The layout was never broken. This is
  not a meter problem at all — it is a **corpus problem**, and the corpus
  is the thing KC1 grades against.*

  ***Corrected KC1 table, both instruments converged:***

  | config | real | meter | gap |
  |---|---|---|---|
  | chain-ec15-d1 | −8.0% | −12.0% | 4.0pp |
  | chain-ec15-d2 | −6.0% | −5.6% | 0.4pp |
  | chain-ec15-d7 | −5.3% | −5.6% | 0.3pp |
  | chain-ec30-d2 | −5.3% | −5.6% | 0.3pp |
  | chain-mil5plates-d0 | −3.3% | +0.7% | 4.0pp |
  | chain-mil5ore-d2 | **+0.7%** (was −28.7%) | −1.3% | 2.0pp |
  | chain-ac1-d0 | −0.3% | +0.6% | 0.9pp |

  *Every solid config within ~4pp against a ±10pp bar. The instrument was
  in far better shape than its own kill criterion suggested.*

  ***What this falsifies, explicitly, because these are recorded above as
  findings:***
  - *Attribution rounds 1–3 (coal belt supply, inserter swing rate, machine
    input buffering, belt→machine rate model, `I6`/`drop_onto_tile`) were
    **all chasing a fixture artifact**. On correct geometry the grenade row
    does not starve at all: 16/16 working, −0.2%. The four falsified
    hypotheses were falsified against a factory that was not the one being
    compared to.*
  - *The PR-3 entry attributes `chain-ac1-d0`'s −42.8% to the disclosed
    **fluid limitation** — "honest under-report, resolves when fluids
    land". **Wrong.** At convergence it reads +0.6%, essentially at plan.
    That was a plausible narrative attached to an unexamined number.*
  - *The four review findings fixed in #467 (craft accumulation, U7 far
    lane, silent inserter drops, dead `converged`) move **nothing** on the
    corpus. All four were latent. None was the gap.*

  ***KC1 is NOT hereby declared passed.*** *The magnitude half now looks
  comfortable, but the rank half grades against band assignments that are
  themselves wrong —* `chain-mil5ore` *is recorded FAIL and is a PASS. A
  criterion cannot be evaluated against a corpus with entries in the wrong
  band, and re-banding the corpus after seeing the meter's answers is
  exactly the tuning KC1's "frozen and committed before this RFC" clause
  exists to prevent. **The corpus must be re-measured at adequate warmup,
  by the oracle, before KC1 is re-evaluated** — and that re-measurement has
  to be justified on measurement grounds alone, never by reference to what
  the meter says.* [#453](https://github.com/storkme/spaghettio/issues/453)
  *(USP@2, −57.0%) and* [#437](https://github.com/storkme/spaghettio/issues/437)
  *(PU@4, −27.3%) are the next candidates; #453 calls its residual the
  single highest-value unknown left in the composition path, and it may
  simply be an unconverged measurement.*

  ***The instrument that caught it did not exist that morning.***
  `converged` *was hardcoded* `true` *with no detector — bot finding #4 on
  #460, which read as a tidy-up. Implementing it honestly (measured on
  delivered, not buffers; zero throughput is not converged; too few samples
  is not converged) is what made the transient visible, and it flags false
  at exactly the warmups where the number was lying. **The field that was
  always true was hiding the reason the RFC's headline criterion failed.***

  ***The generalisable lesson, and it is not the one the earlier entries
  were converging on.*** *This log already names a standing habit — "fix on
  merit, test the story separately" — after four hypotheses failed their
  checks. That habit was being applied correctly and still did not help,
  because every one of those tests was run against a mis-generated fixture
  at a warmup too short to mean anything. **Testing a story carefully is
  worthless if the instrument's inputs are unvalidated.** The unexamined
  assumption was never a mechanism; it was "these two numbers describe the
  same factory, and both have finished settling". Neither was true, and
  neither was ever checked, through three rounds of increasingly careful
  work.*
