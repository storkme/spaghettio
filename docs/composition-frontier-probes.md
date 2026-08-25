# Composition-direction probes: the single-bus frontier and the seam cost

**Notes** — the measurement record behind
[`rfc-072-cell-interface.md`](rfc-072-cell-interface.md) (its Phase-0
evidence base; retained as the raw-numbers record, the RFC carries the
adjudications). Run 2026-08-25 on
`main` at `7cec5ca9` (post-RFC-069), instruments: `sim_export` (tracked
generator) + the meter's `check_one` (108k/216k calibrated window). Meter
asymmetry applies throughout: **below plan ⇒ believe it**; at-plan clears
nothing. Every number below is below plan and therefore believable; the
decisive points should still be sim-anchored before an RFC commits to them.

Motivation: the 2026-07-24 strategy call ("bus stays the low-rate winner;
high rates via composition") left two empirical questions open — *where does
the single bus actually stop delivering*, and *do per-unit receipts survive
composition*. Both are answerable for the price of seven meter runs, and the
answers should shape the cell-interface RFC before any design is committed.

## Probe 1 — the uncapped single-bus frontier (ec family, from ore)

`sim_export electronic-circuit <rate>` with engine-picked belt (no
`max_belt_tier`), default AM3, six-ore inputs. This is deliberately NOT the
banked stress family: every banked high-rate ec row is belt-capped
(ec30–40 yellow, ec60 red); nobody had measured the *unconstrained* bus.

| rate | entities | dims | validator | meter delivered | of plan |
|-----:|---------:|------|-----------|----------------:|--------:|
| 45   | 2,789    | 229×79  | 0E / 7W  | 39.89 | **88.7%** |
| 60   | 3,930    | 236×107 | 0E / 11W | 53.87 | **89.8%** |
| 90   | 6,077    | 241×132 | 0E / 16W | 82.26 | **91.4%** |
| 120  | 9,427    | 251×230 | **19E** / 50W | 106.67 | **88.9%** |

Issue breakdown at 120 (via a local `frontier_issues` probe): **18 of 19
errors are `lane-throughput`** — express lanes planned at 23.2–25.1/s against
the 22.5/s physical per-lane cap — plus one belt-dead-end; the warnings are
input-rate-delivery (30) and 72-machine rows at zero belt margin.

**Read.** There is no cliff. The uncapped bus degrades gracefully
(88.7–91.4% delivered; the 45/s figure is 88.65% at full meter precision,
rounded up in the table) all the way through 120/s. What changes between 90 and
120 is the *nature* of the deficit: through 90 it is the family's usual
~10% delivery gap on a validator-clean plan; at 120 the plan itself exceeds
express lane physics (the meter's own `produced_per_s` for copper-cable
reads exactly 320.0 — its independent per-item counter, not a value derived
from the ec figure), so the deficit becomes structural — no amount of layout improvement reaches
plan. The composition entry point for this family is therefore ≈ above
90/s uncapped, and earlier wherever a user belt cap binds (the banked
belt-capped rows hit their walls at 30–60/s). Composition's pitch at these
rates is *reaching plan* — dissolving the shared-trunk lane saturation by
running k units each below it — not rescuing a collapse.

## Probe 2 — the seam cost (copper-cable → electronic-circuit)

Three legs isolate the producer→consumer seam inside one solve:

| leg | config | validator | meter delivered | of plan |
|-----|--------|-----------|----------------:|--------:|
| stage alone | `copper-cable 90` (plates external) | 0E / 6W | 45.0 / 90 | **50.0%** |
| interface-as-promised | `ec 30`, cable **external** | 0E / 0W | 29.44 / 30 | **98.1%** |
| composed | `ec 30`, cable made **internally** | 0E / 6W | 24.47 / 30 | **81.6%** |

**Read.** Composition costs ~16.5 points at this config — but the loss does
NOT sit in the producer→consumer hand-off (#724 round 1 corrected the first
draft's attribution). The meter's own stage accounting locates it: in the
composed leg the assembly converted essentially every cable it received
(24.47 ec × 3 = 73.4 cable consumed of 73.4 produced — loss-free), slightly
*better* than the boundary leg's conversion (88.3 of 90 available = 98.1%).
The entire deficit is the **embedded producer stage under-delivering its own
plan**: cable production achieved 73.4/90 = 81.6% of its planned rate when
built as an internal stage, where the boundary case receives the same flow
complete. Per-unit receipts still do not survive composition — but the
mechanism to fix is how a producer's output rate survives being embedded,
not a lossy transfer at the consumer.

Confound to carry (now the live alternative hypothesis): the composed leg's
copper-plate boundary (45/s) is exactly one full express belt with zero
margin, so the embedded cable stage's 81.6% may be input-boundary tightness
rather than embedding per se. The follow-up that separates them: re-run the
pair at a rate where boundaries have margin (e.g. ec 20, cable 60). If the
embedded stage recovers with margin, the interface fix targets boundary
provisioning; if it does not, it targets embedded-stage planning.

### The disambiguation run (2026-08-25, same session)

Re-run at ec 20/s, every boundary at or under ~67% belt load
(copper-plate 30/45 = 67%, iron-plate 20/45 = 44%); `dis-cable40` keeps
the standalone producer's output under one belt (40 ≤ 45) so its receipt
is clean of the output-belt cap that polluted `seam-cable90`:

| leg | config | validator | meter delivered | of plan |
|-----|--------|-----------|----------------:|--------:|
| standalone producer | `copper-cable 40` (plates external) | 0E / 0W | 40.0 / 40 | **100.0%** |
| interface-as-promised | `ec 20`, cable **external** | 0E / 0W | 20.0 / 20 | **100.0%** |
| composed | `ec 20`, cable made **internally** | 0E / 0W | 18.93 / 20 | **94.6%** |

**Adjudication: both mechanisms are real, and the split is measured.**
Margin alone recovers ~11 of ec30's 16.5 points (the boundary leg goes to
a perfect 100.0%), but embedding still costs **5.4 points at full margin**
— the internal cable stage delivers 94.6% of its plan (56.78/60) on a
0-error, 0-warning layout, and the circuit output tracks it exactly. The
interface fix therefore targets **embedded-stage provisioning** as the
primary mechanism, with boundary margin as the secondary lever. Consumed
by [`rfc-072-cell-interface.md`](rfc-072-cell-interface.md) (Phase 0);
its K72-1 pins this exact fixture: boundary-style provisioning must lift
`dis-ec20-comp` from 94.6% to ≥98%.

## Incidental finding — a live specimen of the deferred output-side hole

The `copper-cable 90` stage-alone leg delivered **exactly 45.0/s — precisely
one full express belt** — on a **0-error** validated layout. This is
consistent with the single-output-belt ceiling: a 90/s single-item target
cannot leave the bus on one express belt, the validator does not flag it,
and the engine ships it at half plan. RFC-069 Phase C explicitly scoped the
refusal to the INPUT side and recorded the output side as a follow-up
(#723 round-1 adjudication); this leg is the follow-up's concrete
reproduction. Anyone picking that up starts here: `sim_export copper-cable
90 --inputs copper-plate` → meter → 50.0%.

## What this buys the cell-interface RFC

1. **The requirement is quantified**: beat 91.4% at 90/s / 88.9%-structural
   at 120/s (uncapped), and beat the belt-capped walls at 30–60/s.
2. **The seam is the enemy, measured**: 16.5 points at ec30. An interface
   that makes composed seams behave like boundaries recovers most of it —
   the 98.1% leg is the existence proof.
3. **Kill-criterion material**: if a prototype composed pair does not beat
   81.6% at the same config, the interface design is not paying; if it does
   not approach ~98%, the seam contract is leaking somewhere measurable.
