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
