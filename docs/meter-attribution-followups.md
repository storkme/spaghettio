# Meter attribution — open work

**Status (2026-07-26): the KC1 gap is EXPLAINED and the attribution hunt is
CLOSED.** Neither cause was in the meter; both were in how it was measured.
This file previously sent readers at a coal-starvation mechanism that does
not exist — that guidance is withdrawn, and the reasons are below so nobody
re-runs it.

Owning RFC: [`rfc-054-fast-meter.md`](rfc-054-fast-meter.md) — the
2026-07-26 decision-log entry is the full account.

## What the gap actually was

Two independent defects, each sufficient alone.

1. **The fixtures were built at the wrong geometry.**
   `export_chain_fixtures_for_sim` composed at the ambient engine default,
   which #431 moved to L2 on 2026-07-24 — one day before these fixtures
   were regenerated. It exported **L2 geometry under an L0 label**. So the
   meter and Factorio were measuring **different factories**. Fixed in
   #466, with a gate on the exporter path.

2. **The measurements had not converged.** The corpus replay warmed up for
   **two game-minutes**. `chain-mil5plates-d0` reads −38.4% at 2 minutes
   and **+0.7%** once actually settled. The entire military deficit was
   buffer fill being read as a rate. Fixed by raising `WARMUP` to 80
   game-minutes and asserting `converged` on every solid config.

Corrected picture, both instruments converged — every solid config within
~4pp against a ±10pp bar:

| config | real | meter | gap |
|---|---|---|---|
| chain-ec15-d1 | −8.0% | −12.0% | 4.0pp |
| chain-ec15-d2 | −6.0% | −5.6% | 0.4pp |
| chain-ec15-d7 | −5.3% | −5.6% | 0.3pp |
| chain-ec30-d2 | −5.3% | −5.6% | 0.3pp |
| chain-mil5plates-d0 | −3.3% | +0.7% | 4.0pp |
| chain-mil5ore-d2 | +0.7% (was −28.7%) | −1.3% | 2.0pp |
| chain-ac1-d0 | −0.3% | +0.6% | 0.9pp |

## Withdrawn — do not re-run these

The previous version of this file named a "pickup draws from BOTH lanes"
investigation as the next step. **That hunt was chasing a fixture
artifact.** On correct geometry the grenade row does not starve at all:
16/16 machines working, −0.2%. Specifically retracted:

- The coal-belt starvation gradient (9,9,8,8,7,7,6,6,5,5) — an artifact of
  L2-geometry inserters running in an L0 world.
- `take_from_tile_filtered` / **I6** both-lane pickup, and
  `drop_onto_tile`'s far-lane placement, as suspects. Neither is implicated.
- The four "eliminated hypotheses" (supply/topology, swing rate, machine
  buffering, belt→machine rate model). They were all tested against the
  wrong factory, so their elimination proves nothing either way.
- **`chain-ac1-d0`'s −42.8% is not the fluid limitation.** The RFC's PR-3
  entry attributes it to fluids; at convergence it reads +0.6%, essentially
  at plan. That explanation was wrong.

Real defects found en route **do** stand — the splitter second-cell sign
error, I11, and the six review findings. None of them moved the corpus.

## Open work

### 1. Re-measure the corpus at adequate warmup (needs Factorio)

**This is the blocker for re-evaluating KC1**, and it is measurement work,
not modelling work.

The corpus has entries in the wrong band. `chain-mil5ore-d2` is recorded
FAIL at −28.7% and measures **+0.7% PASS, 146/146 working** at
`--warmup 288000`. KC1's rank half grades against these bands, so it cannot
be evaluated until they are right.

```bash
cargo run --release -p spaghettio_sim_harness -- run \
    --bp crates/core/target/tmp/<label>.bp \
    --manifest crates/core/target/tmp/<label>.manifest.json \
    --warmup 288000 --out <label>-long.json
```

Priority order — both are recorded at default warmup and suspect for
exactly the same reason:

- [#453](https://github.com/storkme/spaghettio/issues/453) — USP@2, −57.0%.
  #453 calls this "the single highest-value unknown left in the composition
  path". It may simply be an unconverged measurement.
- [#437](https://github.com/storkme/spaghettio/issues/437) — PU@4, −27.3%.

**Discipline that matters here:** re-banding must be justified by the
oracle alone. Never adjust a band because it disagrees with the meter —
that inverts the whole integrity argument (information flows meter →
constant, never the reverse) and turns KC1 into something tuned to its
answer.

### 2. Convergence detection is a floor, not a ceiling

`Factory::detect_converged` reports `converged: true` for
`chain-mil5ore-d2` at a 40-minute warmup, where it reads −13.8%; the true
value is −1.3% at 80. Three consecutive 1-minute windows within 2% cannot
distinguish steady state from a large factory filling slowly and smoothly.

The real harness has the identical limitation, documented on
`RunParams::with_warmup`. Both currently paper over it with a generous
fixed warmup. A detector that scales its window count or tolerance with
factory size would let both instruments stop guessing.

### 3. Still genuinely unexplained

`chain-ec15-d1`: meter −12.0% against real −8.0%, stable under warmup and
therefore a real 4pp disagreement rather than a transient. The smallest
open gap in the corpus and the cleanest one to chase — it is a 15-machine
fixture.

## Standing habit, amended

The earlier version of this file ended with **"fix on merit, test the story
separately."** That still holds, but it was not enough — it was being
applied correctly through three attribution rounds that were all worthless,
because every test ran against a mis-generated fixture at a warmup too
short to mean anything.

The amendment: **validate the instrument's inputs before testing stories
told about its outputs.** The unexamined assumption was never a mechanism.
It was "these two numbers describe the same factory, and both have finished
settling" — and nobody checked either.
