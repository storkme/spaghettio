# PR churn audit — 2026-07-12 → 2026-08-09

**Status:** closed 2026-08-09. Backs the change-size norm in
[`CLAUDE.md`](../CLAUDE.md#workflow-branches-review-merging). Numbers here are
the durable record; re-measure before citing them outside this window.

Motivating question from the owner: PR churn feels like it's rising — is the
model getting worse, or is the code getting harder to work in?

Answer: **neither, primarily.** Change size is the dominant controllable
variable. Two of the audit's own measurement defects had to be found and fixed
before the numbers below were trustworthy (see [Method](#method)).

## Headline

| Measure | Value |
|---|---|
| PRs opened / merged | 232 / 218 |
| Blame-paired rework edges | 3,114 |
| Median rework lag | 2 days (57% within 3) |
| Share of rework from PRs ≥400 added lines | **93.2%** (from 32% of PRs) |
| Corrective-PR rate | flat 20–30% since April; 12% in Aug wk0 |

### Rework per 100 added lines, by PR size

| Bucket | n | Mean of per-PR rates | Pooled (Σrework/Σadds) |
|---|---:|---:|---:|
| <100 adds | 42 | 1.4 | 1.1 |
| 100–400 | 82 | 3.2 | 3.0 |
| 400–1k | 41 | 8.5 | 8.3 |
| **>1k** | 29 | **10.5** | 6.9 |

**What is robust:** the rate rises steeply and monotonically across the first
three buckets under *both* statistics — roughly 6–8× from <100 to 400–1k. That
alone carries the norm, whose threshold is 400.

**What is not:** the top bucket disagrees between the two columns (mean 10.5,
pooled 6.9). Pooled weights by PR size, so a handful of very large PRs with
proportionally little rework dominate that cell. Do not claim the relationship
is monotonic across all four buckets on this data — it is only monotonic on the
per-PR mean. The safe reading is that >1k is *at least as bad* as 400–1k, not
demonstrably worse.

> **Denominators.** These buckets cover the 194 PRs that merged 2026-07-20 →
> 08-09 with more than 20 added lines. That is narrower than the 218 merged
> since 07-12 in the headline table: `review_rounds.tsv` (the per-PR
> commits/comments/latency pull) starts at 07-20 and covers 217 PRs, and 23 of
> those fall below the 20-add floor used to keep tiny PRs from producing
> meaningless per-line ratios. The "32% of PRs ≥400 adds" figure is 70/217 from
> the unfiltered set. Reconcile before quoting any of these against each other.

## What was refuted

- **Model capability declining.** The corrective-PR share (fix/revert/restore
  titles) is flat 20–30% April→August and lowest in August, while throughput
  quadrupled (98 merged in the week of Jul 22 vs 27/week in April). What
  changed is volume, not defect rate.
- **Code complexity.** Tested with a within-PR control: for 36 PRs touching
  both an old entangled file (`ghost_router.rs`, `belt_flow.rs`, `placer.rs`)
  and other files in the *same commit* — holding author, session, review and
  size constant — old files churned at **3.30** per 100 adds vs **6.87** for
  the rest, and churned higher in only 3 of 36 pairs. Pooled: 2.48 vs 4.62.
  Independently confirmed by size-bucketing all 217 PRs by complexity-touch:
  the complex-touch group is at or below the other in every powered cell.
  Real code-quality problems exist (`route_bus_ghost` is a 3,875-line function
  whose rework arrives at 12–97 days, the classic long-lag signature) but they
  are ~2% of rework, not the driver.
- **Merging too fast.** 69% of PRs merge within an hour, but normalized rework
  is flat across latency bands (3.7 / 3.7 / 2.9). Soak time buys nothing.
- **Short-instruction / long-autonomy runs.** Sessions with 38–54 tool calls
  per human turn were traced; their rework is planned scaffolding reuse by
  later RFCs, not correction. Zero cases of a later PR fixing their mistakes.
- **RFC kill criteria never firing.** 7 of 30 in-window RFCs (23%) hit a named
  kill criterion and visibly changed course, with zero silent overrides.
- **Handoffs, subagent delegation, compaction.** All clean. 0/6 cold starts
  needed context repair; 47/47 sampled subagents self-verified before
  reporting; 5 compaction boundaries read in full showed no dropped
  constraints.

## The mechanism: claim surface area

Large PRs don't churn because they're reviewed less carefully — review on them
is heavy (#569 took 6 bot passes, #574 absorbed eleven). They churn because a
big PR asserts **more independently-falsifiable things** than its verification
covers: several sim numbers, several fixtures, several corpus percentages.

Five fix-chains were traced end to end. In none of them was the first fix's
verification fabricated or absent — every one ran real tests or real sims and
reported honestly. The recurring failure is **narrow-population
generalization**:

| Chain | First fix verified against | Next fix found |
|---|---|---|
| #525 → #603 | Bidirectional sim cross-validation at the starving machines | A second arithmetic path that only fires on sub-one-machine plans |
| #474 → #520 → #521 | Validator clean on *both* arms of a never-worse comparison | Both 0 err/0 warn; one sims at plan, the other at **0.00/s** |
| #460 → #467 → #469 | Each hypothesis falsification-tested in turn | Every test ran against a mis-generated fixture at too short a warmup |
| #354 → #362 | That the harness ran and produced a report | Kit chests cross-feeding ores — "poisoned every multi-input fixture ever measured" |
| #605 → #606 | Two sim fixtures | A six-fixture population broke the meter's floor property on 2 of 6 |

Not all of it was preventable. #365's "runs at plan in headless Factorio" was
invalidated by a tech-state parity fix (#378) that landed **14 minutes after
#365 merged**. But #569's was: RFC-064 §(b)'s own spec text said "mean over
consumer terminals" and the implementation used producers. And #510 named its
own risk — "the RFC's evidence existed on one machine only" — before a
different host produced a different number the next day.

## Method

Blame-pairing: for each merged PR, the lines its commits deleted or modified
were blamed against the base of its own commit range, and the introducing
commit mapped back to its PR.

**Two defects in the first version of this pipeline, both material:**

1. **Truncated diff range.** v1 diffed `sha^1..sha`. Correct for squash and
   true merge commits; wrong here, because this repo *rebase-merges* — a
   multi-commit PR lands as N commits and the recorded merge SHA is only the
   last. Verified on #317: 281 additions across 3 commits, of which
   `sha^1..sha` showed 18 lines in one docs file. Worst on large PRs, which
   average **13.6 commits** above 1k adds vs 1.5 below 100.
2. **Issue numbers read as PR numbers.** v1 built the commit→PR map by parsing
   `(#N)` from commit subjects. This project writes *issue* refs there.
   Rebuilt from authoritative per-PR commit ranges: **35% of commits (322 of
   907) had been mis-attributed**.

Effect of correcting both:

| Measure | v1 | v2 |
|---|---:|---:|
| Edges | 1,639 | 3,114 |
| Median lag | 1d | 2d |
| Within 3 days | 61% | 57% |
| Chain-interior share | 61% | 38% |
| Rate >1k adds | 5.7 | **10.5** |
| Large-PR share | 89.7% | 93.2% |

The size finding strengthened and became monotonic — v1's dip in the top
bucket was the artifact, not a floor effect. The fix-chain finding shrank by a
third.

### Caveats

- **Rework ≠ defect.** A sampled classification of 85 PR-pairs splits edges
  ~57% planned iteration / 23% genuine correction / 20% collision. Absolute
  edge counts overstate churn; the size finding is ratio-based and survives.
- **Size and difficulty are confounded.** Large PRs may be large because the
  work is harder. Per-line normalization controls for volume, not difficulty.
- **One signal is unresolved.** The rate at which the owner pushes back *in
  conversation* rose steeply (two independent hand-classifications of the same
  939 turns disagree 2.3× on level — 4.4% vs 10.2% — but both find it roughly
  doubling week over week). It moves opposite to the code-level corrective
  rate, and the late sessions were meta-work auditing the instruments
  themselves, which both selects for correction-heavy dialogue and means the
  observer had already changed. Thin base (150–173 turns). Not settled.

## Open follow-ups

Ranked by expected value, not yet actioned:

1. **Require claims to name their population.** "At plan" / "safe floor" /
   "gate PASSED" should be unspeakable without stating fixture set, warmup,
   host and tech-state — and what is *not* covered. Overclaim concentrates in
   PR **titles**; bodies are usually honestly hedged.
2. **Wire the validator into export and the sim harness.** Export never
   consults validation and the sim manifest carries no validator state, so a
   7-warning layout can be exported, simmed and reported on with nothing
   objecting. RFC-050 promised `Manifest.validator_errors/warnings` and never
   delivered. Of ~40 checks only 4 carry real refusal power, all documented
   "never sim-anchored".
3. **Review-bot reliability.** The single most systemic complaint in the
   conversation record — 8 distinct sessions, more than any other theme —
   plus 9 of 13 closed-unmerged PRs are skip-guard duplicates or stacked-PR
   orphans, meaning ~4% of the "232 PRs" figure is the same work re-counted.
4. **Turnaround guardrail on large PRs**, not a concurrency cap. Concurrency
   is a threshold effect on one day (Jul 22 carries 31% of all reworked lines;
   dropping it collapses the day-level correlation from 0.318 to 0.145), and
   Jul 31 at 34 open PRs was calm.
5. **Stale doctrine.** `CLAUDE.md`'s "Primary workflow" bullet still says to
   "hit the web app to eyeball the layout", contradicting step 2 of
   [Verification protocol for layout engine changes](../CLAUDE.md#verification-protocol-for-layout-engine-changes),
   which says the eyeball "was never a good substitute". The PR template's
   Verification block carries the same stale checkbox — **this PR edits that
   template to add the size bullet and deliberately leaves the checkbox alone**,
   to keep the change-size commit reviewable in isolation; it is not an
   oversight. The `validate/` submodule list names a non-existent
   `underground.rs` and omits `belt_detour.rs` / `sushi.rs`; the crate list
   omits `meter` and `sim-harness`.

   (Anchors here are section links, not line numbers, because this PR's own
   insertion shifted the protocol text by ~16 lines — a line-number citation
   would have been stale in the commit that added it.)
