# RFC-075: The far hand's credit is a flooded-belt number — pickup-side derating for single long-handed hands

**Status: Active — Phase 0 (forensics) complete 2026-08-27, Phase 1
(experiments) running.** Opened on RFC-073's replacement pointer for
RFC-072 residual (a): "the K=18/K=20 grid failures and PU-from-ore's
twenty 2.40/2.40 iron sides are serial single long-handed hands on one
belt at the credit — a pickup-side (belt density / lane fill) effect to
test in the sim, not a ladder constant." This RFC tests it.

## Summary

The inserter ladder credits a long-handed input hand
`machine_feed_rate("long-handed-inserter", L)` = 1.2 × hand(L)/hand(0)
items/s (2.40/s at the default L2). That number was **calibrated on a
flooded express feed** — a stationary, fully compressed belt
(`common.rs::machine_feed_rate`'s doc: "flooded express feed into a
37.5/s sink", RFC-049). A row's far belt is only in that state at its
dead-end tail. Every hand upstream of the tail picks from a *moving*
stream whose density is the row's remaining downstream demand over the
belt's capacity — 12/45 = 27% on the ec@12 cell's express iron belt —
and the Phase-0 forensics below show the same hand type at the same
credit delivering **2.11/s at the head and 2.48/s at the tail** of one
five-machine row, with every producer upstream *blocked* (`full_output`).
Supply is not the constraint; the hand's realized pickup rate is, and it
is a function of where on the belt the hand sits.

Proposed: (1) confirm the mechanism with two sims that hold the
geometry fixed and vary only the belt state (E1: the same cell
harness-flooded from plates; E3: the same cell with two far hands per
machine); (2) if confirmed, derate the **single reach-2 hand** in the
count ladder by a measured factor so a far side whose plan lands within
that factor of one hand's credit gets a second hand — the row's
`HAND_MARGIN` that RFC-072 P2 unit 2 already applies in grid territory,
moved into the ladder where RFC-049 P3 always said it belonged, but
**scoped to the hand class the census implicates** (far, single,
long-handed) rather than the uniform margin RFC-073 measured to be
non-discriminating; (3) price it against the registry and the
calibration bank before it ships, and re-bless what it re-shapes.

What you would be agreeing to: a sizing change on one hand class that
adds one long-handed inserter to affected machines (no row re-layout),
gated by sim receipts on both the mechanism and the fix, and by a
registry/bank pricing pass with a hard cap.

## Motivation

Reproducible today. `cell_export electronic-circuit 12 <label> <dir>`
(from ore, the ec@240 grid's constituent) composes 849 entities, 0
errors / 0 warnings, and sims **11.15/12.00 produced (−7.1%), WARN,
converged** (`scratchpad/cell-ec12.json`, 2026-08-26, Factorio 2.0.77,
432k warmup). The K=20 grid of that cell measured −6.7% (FAIL) with a
per-strip census machine-for-machine identical to the constituent, so
the grid is exonerated and this is the cell's own gap (RFC-072 decision
log, 2026-08-26).

### Phase 0 forensics — the machines say where

Per-machine crafts over the last checkpoint window (27 s), from the
report's `timeseries` (`scratchpad/rfc075_forensics.py`):

| stage | n | crafts/s per machine | status |
|---|---|---|---|
| iron furnaces (y=6, x=146–203) | 20 | 17 × 0.630 (max), 3 × 0.556–0.593, **2 × 0.000 `full_output`** | sum **11.19/s** vs 12.0 planned |
| cable machines (y=6, x=107–128) | 8 | 6 × 2.37–2.41, 1 × 1.89, **1 × 0.67 `full_output`** | 33.6 cable/s vs 36 |
| **EC machines (y=8, x=217→229)** | 5 | **2.11, 2.11, 2.19, 2.26, 2.48** | all `working` |

Read together: the producers on both input belts are *blocked* — two
iron furnaces and a cable machine sit at `full_output` with a full
belt in front of them — while the EC consumers are `working` yet craft
below their 2.40/s plan. A producer blocked behind a full belt whose
consumers are under-pulling is the signature of **the consumers' input
hands** being the constraint, not supply. And the consumers are not
uniformly slow: the profile is **monotone along the belt**.

### The geometry — one belt, one hand type, five positions

Decoded from the fixture (`bp_window.py`): the EC row's **far belt is
express** (`express-transport-belt`, 45/s) at y=4.5, carrying iron
**eastward** from x=208.5, fed at its west end by the furnace row's
output rising from y=9.5; it **dead-ends at x=228.5**. Each EC machine
(centres x=217.5 … 229.5, y=8.5) has exactly one `long-handed-inserter`
at y=6.5 (x=216.5, 219.5, 222.5, 225.5, 228.5) picking iron from that
belt, and one `stack-inserter` picking cable from the near express belt
at y=5.5. The row plans 12/s ÷ 5 = 2.40/s of iron per machine — exactly
one long-handed hand's credit at L2 (`SidePlan{count:1, capacity:2.40}`,
RFC-073's census: "EC iron 1×LHI 2.40/2.40").

So along one belt, in order of flow:

| machine x | hand position on the iron belt | iron flow passing the hand | crafts/s |
|---|---|---|---|
| 217 | head — 4 machines downstream | ≈ 9.4/s of 45 (21% dense, moving) | **2.11** |
| 220 | 3 downstream | ≈ 7.0/s | 2.11 |
| 223 | 2 downstream | ≈ 4.7/s | 2.19 |
| 226 | 1 downstream | ≈ 2.5/s | 2.26 |
| 229 | **tail — dead end, stationary** | 0 (belt backs up under it) | **2.48** |

The tail hand *exceeds* its credit (2.48 > 2.40, the sim's 1.04×
"conservative margin" the calibration recorded); the head hand delivers
88% of it. Nothing differs between those hands except the belt under
them. The K=20 grid's forty copies show the identical profile
(`2.10 2.20 2.20 2.20 2.50` in every copy, y=8 and y=57), and the K=18
grid's `2.50 2.50 1.70 0.00 2.30 2.30 2.40 2.50` cable rows show the
blocked-producer half of the same picture.

### Why the credit is a flooded-belt number

`machine_feed_rate`'s calibration (RFC-049, 2026-07-22) measured every
hand "flooded express feed into a 37.5/s sink" — the belt under the hand
stationary and fully compressed, which is the *best* case for a
two-item hand: both items are already under the pickup point. A hand
whose belt moves at express speed with 21% occupancy must wait for
items to arrive, and for a hand of 2 must catch a second one before it
swings. That regime was never in the matrix. RFC-073's census found the
credit non-discriminating *as a scalar* — at-plan rows ship near-side
fast hands at 0.93–0.974 and the ec15 cell's over-credit 1.042 hand
produces at plan — and this RFC says why: the 1.042 hand is a
**last-in-row** hand (the trimmed tail machine) sitting on a backed-up
belt, and the fast hands are near-side stack/fast hands on the hungrier
belt, not reach-2 singles. The class that fails is specific: **a single
long-handed hand at or near its credit with consumers downstream of it
on the same belt.**

### What this is not

- Not the zero-headroom provisioning class (#448, RFC-069): supply here
  is blocked, not short — the furnaces have spare. PU-from-ore's rows at
  y=160–192 (`2.26 2.33 2.43 1.32*`) show the *opposite* profile (tail
  short) on tap-fed belts and belong to that class; this RFC does not
  claim them, though its twenty 2.40/2.40 iron sides may carry both.
- Not the ladder's uniform margin: RFC-073 killed that by census.
- Not a harness artifact: kit clean, converged, drift ~0, and the
  profile is reproduced 40× in the grid.

## Design

### Phase 1 — two sims that vary only the belt state

Same solver, same composer, same hands:

- **E0** (have it): ec@12 from ore — furnace-fed express far belt, 27%
  dense at the head. **−7.1%**, profile 2.11 → 2.48.
- **E1**: ec@12 **from plates** (`cell_export … iron-plate,copper-plate`,
  213 entities, 0/0). The harness feed rig tops its chests every second
  (`scenario.rs` `on_nth_tick(60)`: refill to 400), so a boundary belt
  is **flooded** — compressed and creeping at the consumption rate,
  which is the calibration regime at every hand. Prediction if the
  mechanism is belt state: **at plan, flat profile ≈ 2.40 on all five**.
  If E1 also reads −7% with the same profile, the belt state is not the
  variable and the credit is simply wrong (K75-1 does not trip — a
  margin still follows — but the Design's mechanism claim is retracted
  and the fix is priced as a plain recalibration).
- **E3**: ec@12 from ore with **two far hands per machine** — the
  candidate fix, applied to this cell only (the ladder derating behind
  an env gate on this branch). Prediction: at plan (the receipted ec150
  cell at 12.5/s runs its EC iron on two hands at 52% and sims at plan).
  This is the fix's own receipt, and K75-2's instrument.
- **E2** (optional, only if E1 and E3 disagree): the same cell with the
  far belt one tier lower (a denser, slower stream) — separates
  "density" from "speed" if the first two do not settle it.

### Phase 2 — the fix, scoped by the census

`inserter_ladder::size_side` for `Reach::Far` runs a **count ladder at
the single long-handed rate** (`count_ladder`; no fast/stack
long-handed exists). Derate that per-hand rate on **input** (belt-pickup)
far sides — every hand upstream of the tail sees the moving stream, so
the derating is per hand, not per side:

```text
far_pickup_credit(L) = machine_feed_rate(LONG_HANDED, q, L) × FAR_PICKUP_FACTOR
```

Belt-drop (output) far sides are untouched: a hand dropping onto a belt
is not picking from one, and `size_belt_drop_side` has its own
lane-capped table (#385).

with `FAR_PICKUP_FACTOR` the measured head-of-row realization
(E0: 2.11/2.40 = 0.88; take the floor of E0/E1/E3's head readings,
rounded down to 0.85 unless the sims say otherwise — that is the
constant RFC-072 P2 unit 2 already ships as `HAND_MARGIN` in grid
territory, and this RFC replaces that scoped term with the ladder's
own). A plan that fits within one derated hand keeps one hand; a plan
between the derated and the flooded credit gets a second hand, sized
by the existing ladder with no other change. Near sides are untouched
(RFC-073: their 0.93–0.974 hands produce at plan). The trimmed
last-in-row far side is untouched too — it is the tail hand, the
flooded case, and the ec15 receipt says so — which the templates
already distinguish (`LastInRow`).

Where it lands: `inserter_ladder.rs` (the far branch of `size_side` /
`count_ladder`), `SidePlan::capacity` reads the derated credit so
RFC-073's census reports fullness against the number that is actually
credited, and `cells::chain::required_copies_at` drops its own
`HAND_MARGIN` term for the far side (the ladder now carries it; the
near-side and belt terms stay). Expected diff: well under 100 lines of
engine code plus tests.

### Pricing before shipping

RFC-073's census instrument (`bus/sizing_census.rs`,
`InserterSideSized`) is what enumerates the affected sides — this RFC
is the consumer that RFC-073's retention contract named, and its
"flip condition" is discharged by that use. A probe on this branch
lists every `Reach::Far`, `count == 1` side in the calibration bank and
the sim registry with `required > FAR_PICKUP_FACTOR × capacity`, joined
to the fixture's sim verdict, and states for each whether the fix adds a
hand (a geometry change → registry hash / bank fingerprint moves →
re-bless with a re-sim). From RFC-073's tables the expected set is
small and already-deficient: PU-from-ore (20 sides, 87.7%), uranium
(1 side, non-converged), the ec@12-class cells; the at-plan rows'
near-side hands do not qualify.

### Alternatives considered

- **Position-aware credit** (derate only hands with downstream
  consumers): more precise, saves one hand on tail machines. Rejected
  for Phase 2 — the tail machine is already handled by `LastInRow`
  trimming in the templates, and interior positions differ by at most
  0.15/s on the profile; not worth a second code path.
- **Flood the belt by over-provisioning supply**: not possible — the
  plan is exact by construction and the furnaces are already blocked.
- **Lower the far belt's tier** (denser stream): changes the belt margin
  term RFC-072 P2 added and every cell's belt choice; E2 exists to
  measure whether density or speed is the lever, but the hand is the
  cheaper fix even if density helps.
- **Keep the grid-only `HAND_MARGIN`**: it already protects K > K_MAX;
  it does nothing for the K=1 cells (the ec@12 constituent is
  sub-K_MAX and ships −7%), native rows, or PU-from-ore.

## Kill criteria

- **K75-1 (mechanism)**: if **E3** (two far hands, same cell, same
  supply) does not produce ≥ 99% of plan, the hand is not the
  constraint and the profile has another cause — stop, record, and
  hand the cell back to RFC-072's residual list. (E1 refines the
  mechanism claim but does not by itself kill: a wrong credit is fixed
  by the same margin.)
- **K75-2 (cost)**: if the scoped derating re-shapes more than **6**
  calibration-bank rows or **3** registered strips that currently
  measure at plan (produced ≥ 99%), the scope is wrong and the fix is a
  re-sizing, not a correction — narrow the class or stop (RFC-073's
  K73-4 pricing rule).
- **K75-3 (no regression)**: any re-shaped row that measured at plan
  before must re-sim at ≥ its previous produced rate − 1 pt. One
  regression → the derating does not ship; investigate the row first.
- **K75-4 (evidence)**: the constant ships only with the three receipts
  (E0/E1/E3) in this RFC's decision log and the re-blessed rows in the
  registry/bank. A derating with a guessed constant is RFC-073's
  killed shape.

## Verification plan

1. E1 and E3 sims: `--warmup 432000 --speed 32 --timeseries --out`,
   converged, kit clean; per-machine profiles via
   `scratchpad/rfc075_forensics.py`, recorded in the decision log.
2. Ladder unit tests: the derated far credit at L0/L2/L7, the
   one-vs-two hand boundary at the factor, the last-in-row exemption,
   near sides bit-identical; the discrimination check (restore the
   flooded credit → the boundary test fails).
3. `SidePlan::capacity` reflects the derated credit; RFC-073's
   `census_sees_the_ec15_cells_far_hand_at_the_credit` re-reads its
   fullness accordingly (or is re-pinned with the reason).
4. The pricing probe's table in the decision log (K75-2), then: full
   core suite, the registry gate (`cell_composition.rs`), the bank
   fingerprint probe; re-bless of every moved row **by sim**, never by
   hash alone (K75-3).
5. Verification protocol for layout-engine changes (`CLAUDE.md`):
   snapshot the ec@12 row and count the far hands; clippy; WASM build.

## Phasing

- **Phase 0 — forensics.** COMPLETE (this document's Motivation).
- **Phase 1 — E1/E3 (E2 if needed).** In flight.
- **Phase 2 — ship the scoped derating + pricing + re-bless.** Gated on
  K75-1 clearing.
- **Close-out** updates RFC-072 residual (a) (the pointer this RFC
  discharges), RFC-073's retention contract (consumed), `status.md`,
  `rfcs.md`.

## Decision log

- *2026-08-27 — opened.* Phase 0 done from artifacts already on disk:
  `cell-ec12.json` (the −7.1% constituent), the K=18/K=20 grid reports,
  the PU-from-ore bank report. The head→tail craft profile on one belt
  with blocked producers is the finding; the calibration regime
  ("flooded express feed") is the explanation offered. Decisions: the
  belt-state experiments (E1/E3) come before any engine change; the
  fix is scoped to the single reach-2 hand, not the uniform margin
  RFC-073 killed; the grid-only `HAND_MARGIN` is the thing this
  generalizes, not a second mechanism. E1 launched (from-plates cell,
  213 entities, 0/0).
- *2026-08-27 — Phase 1 scaffolding.* The far pickup derating is
  wired into `inserter_ladder::size_side` behind
  `SPAGHETTIO_FAR_PICKUP_FACTOR` (default 1.0 = bit-identical; input
  far sides only, `LONG_HANDED` only). Under the gate at 0.85 the ec@12
  cell composes at **856 entities** (from 849): every EC machine's far
  side goes from one long-handed hand to **two** (x=217.5+218.5 …
  229.5+230.5 at y=6.5), the stack hand shifts one column east, the
  row's pole moves to the head (x=216.5), and the row grows one tile
  wider (the belts at y=3.5–5.5 extend by one) — 0 errors / 0
  warnings, 238×17 unchanged. That is E3's fixture; its sim and E1's
  are running. `contest_favors_far` still reads the flooded far
  ceiling, and the far side won the contested column on the tie rule
  (0 ≥ 0) — Phase 2 must route the derated credit through the contest
  too so the outcome is by margin, not by tie. The K75-2 pricing runs
  (bank fingerprint probe + registry gate under the gate) were
  launched alongside.
- *2026-08-27 — K75-2 pricing at 0.85 (before any sim verdict, so the
  cap is read cold).* Bank fingerprint probe (`SPAGHETTIO_CALIBRATION_BANK=
  /tmp/calibration-matrix-2026-08-27-732`, zone cache copied): **34/35
  rows byte-identical**; the one that moves is
  `tier5_processing_unit_from_ore_am3` (0 E / 14 W vs the bank's 0 E /
  11 W) — the 87.7% row whose twenty 2.40/2.40 iron sides RFC-073
  censused. Registry pin survey (`probe_registry_pin_survey`): **every
  sub-K_MAX strip unchanged** (ac@1, mil5 ×2, plastic@2, sulfur@2,
  ac@2, ec@15 ×4, ec@30, ec@75, ec@150, gear@20, chem5); **three grid
  rows move** — ec@240 from ore (4f95009b → eac22938), ec@240 from
  plates (5c83b419 → 7ffba350), ac@56 (d2ed8119 → b955d9aa). The grid
  moves are the quantizer re-planning: with the derated credit K=18's
  2.22/s iron hands get two hands (54%) instead of one at 92.6%, so
  `required_copies_at` stops bumping at K=18 rather than K=24 — a
  different, smaller grid, which K75-3 requires re-simmed before the
  factor ships. Cost against the cap: 1 bank row (already deficient)
  + 3 registry rows (all at plan today) ≤ K75-2's 6 / 3. Proceeds.
- *2026-08-27 — E1 receipt: the mechanism is the belt state.*
  `cell-ec12-plates` (the same 12/s cell composed from plates — 5 EC
  machines on ONE long-handed iron hand each at the 2.40 credit, 8
  cable machines, 213 entities, 0 errors; the harness feed rig floods
  both plate belts), Factorio 2.0.77, 432k warmup, speed 32,
  converged, drift +0.0%, kit clean: **PASS — produced 12.50/12.00
  (+4.2%), delivered +5.6%; all 13 machines `working`; every one of
  the five EC machines crafts 2.500/s** (its crafting maximum; the row
  planned 2.40) — a flat profile at the head as at the tail
  (`x=47..59`: 2.500 ×5). The cable machines read 1.96–2.42/s, under
  their 2.5 ceiling only because the EC row is the one pulling on
  them. Contrast E0 (furnace-fed, 27%-dense moving express stream):
  2.11 → 2.48, −7.1%. Same hand, same credit, same geometry class;
  the belt under the hand is the only variable, and it moves the hand
  from 88% of credit to 104%. The 2.40 credit is a flooded-belt
  number. (E1's "1L" is the composition receipt riding `warnings`,
  RFC-074 K74-1 — not a validator finding.)
