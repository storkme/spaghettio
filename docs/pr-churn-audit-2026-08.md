# PR churn audit — 2026-07-12 → 2026-08-09

**Status:** closed 2026-08-09. **Reference, not a note** — `CLAUDE.md`'s
change-size norm cites this file as its evidence, so it is load-bearing and
outside the "archive or delete freely" contract that the docs taxonomy gives
session notes. If the norm goes, this can go with it; not before. Numbers here
are scoped to the window in the title — re-measure before citing them outside
it, using the checked-in pipeline at
[`scripts/pr-churn-audit/`](../scripts/pr-churn-audit/README.md). That pipeline
reproduces the **blame-edge, age and size-bucket** figures exactly — the ones
the norm rests on. It does not regenerate the hand-classified parts (corrective
rate, latency bands, the within-PR complexity control, concurrency); those live
only in this doc. Its README also records the measurement defects — a
truncated diff range, an over-wide one, issue numbers read as PR numbers —
that made earlier versions of these numbers wrong.

Motivating question from the owner: PR churn feels like it's rising — is the
model getting worse, or is the code getting harder to work in?

Answer: **neither, primarily.** Change size is the dominant controllable
variable. Two of the audit's own measurement defects had to be found and fixed
before the numbers below were trustworthy (see [Method](#method)).

## Headline

| Measure | Value |
|---|---|
| PRs opened / merged | 237 / 221 |
| Blame-paired rework edges | 2,022 |
| Median rework lag | 1 day (61% within 3) |
| Share of the bucket population's rework from PRs ≥400 added lines | **89.3%** (from 36% of that population) |
| Corrective-PR rate | flat 20–30% since April; 12% in Aug wk0 |

### Rework per 100 added lines, by PR size

| Bucket | n | Mean of per-PR rates | Pooled (Σrework/Σadds) |
|---|---:|---:|---:|
| <100 adds | 42 | 1.6 | 2.1 |
| 100–400 | 84 | 2.5 | 2.4 |
| 400–1k | 42 | 4.7 | 4.6 |
| **>1k** | 29 | 5.7 | 3.6 |

**What is robust:** the rate rises across the first three buckets under *both*
statistics — ≈3× from <100 to 400–1k on the per-PR mean (1.6→4.7), ≈2×
pooled (2.1→4.6). That carries the norm, whose threshold is 400.

**What is not:** the >1k step. The two statistics disagree in *direction*
there (mean 4.7 → 5.7, pooled 4.6 → 3.6), and the bucket has flipped under
every measurement correction so far (fell in v1, climbed in v2, fell in v3,
splits in v4 — see [Method](#method)).

Do not read any of that as a ceiling. The step is **confounded by
right-censoring**:
an edge exists only when the *reworker* is inside the window, so a PR merged
near 08-09 accrues almost no rework regardless of its size, and stage 4 divides
that near-zero by its full additions. Any bucket whose PRs cluster late is
deflated. The floor-effect reading (largest PRs are often standalone new
subsystems with less existing code to collide with) is plausible but this
pipeline cannot distinguish it from truncation — only the left-censoring was
previously acknowledged. Treat >1k as unmeasured, in either direction.

> **Denominators.** `04-analyze.sh` prints these; do not hand-derive them.
> The per-PR pull (`review_rounds.tsv`) covers **220** PRs merged 2026-07-20 →
> 08-09; **197** of those clear the >20-add floor and form the bucket
> population; **71** of the 197 are ≥400 adds — **36%** of the bucket
> population, or 32% of the unfiltered 220.
>
> Quoting 32% against 197 is the mixed-denominator trap, and an earlier revision
> of this very doc did exactly that. Stage 4 now prints all four figures
> together and names the trap, because every denominator dispute in this
> pipeline's review history came from one number being hand-copied into prose
> and not updated with its siblings.

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
  Independently confirmed by size-bucketing by complexity-touch (run on the
  earlier 217-row per-PR pull, not the current 220 — that control has not been
  re-derived against the corrected pipeline, and its conclusion is directional
  rather than exact):
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

Effect of correcting both, and of the two later review-caught fixes (v4):

| Measure | v1 | v2 | v3 | **v4 (current)** |
|---|---:|---:|---:|---:|
| Edges | 1,639 | 3,114 | 1,999 | **2,022** |
| Median lag | 1d | 2d | 2d | **1d** |
| Within 3 days | 61% | 57% | 60% | **61%** |
| Rate <100 adds | 1.6 | 1.4 | 1.6 | **1.6** |
| Rate 400–1k | 6.3 | 8.5 | 6.1 | **4.7** |
| Rate >1k adds | 5.7 | 10.5 | 6.0 | **5.7** |
| Large-PR share | 89.7% | 93.2% | 90.1% | **89.3%** |

These v4 figures are the **checked-in pipeline's own output**, regenerated
end-to-end from scratch rather than from any surviving intermediate. (An
earlier draft of v3 quoted 1,841 edges and 1.6/2.7/6.3; those came from
reusing a `prs_merged.json` capped at 300 PRs, which silently dropped blame
hits whose target PR fell outside the cap. Raising the cap is why the edge
count rose and the age tail lengthened (p90 9 → 81 days): rework of older PRs
was previously invisible, not absent.)

**v4 is v3 plus two fixes caught in this PR's own review.** First, the
boundary walk now validates a trailing `(#N)` against the merged-PR list —
defect 2's issue-ref regex had re-entered through the walk and truncated the
range of #317, the very PR defect 1 was measured on. Isolated, that fix moved
almost nothing (edges 1,999 → 2,005). Second, the diff now passes `-w` to
match the blame's `-w`, so whitespace-only churn no longer counts as rework —
and that is what moved the numbers: the 400–1k bucket drops 6.1 → 4.7,
meaning roughly a quarter of that bucket's previously-counted rework was
whitespace-only lines. (The edge *row* count still rises 2,005 → 2,022 under
`-w`: rows are hunk-grained, and a block whose interior lines changed only in
whitespace splits into several smaller hunks even as the counted lines fall —
large-bucket rework lines drop 4,885 → 4,332.)

**v2 was the outlier, and v2 is what was originally published here.** Replacing
`sha^1..sha` with a bare `sha~N..sha` fixed a too-narrow range by introducing a
too-wide one: `gh` reports the PR *branch's* commit count, but a squash-merged
PR contributes only one commit to main, so `sha~N` walks N−1 commits back into
earlier PRs. Measured (`probe-v2-defect.sh`): **50 of 221 in-window PRs
(22%)**, and size-correlated (17% under 400 adds, 33% over) — biased in the
same direction as the finding.
v3 resolves each range by walking back only until it meets a commit announcing
a different PR, which handles squash, rebase and merge commits alike. It lands
almost exactly on v1, whose two errors had partially cancelled.

The size finding survives all four versions: the climb from <100 to 400–1k,
and large PRs producing ~90% of rework, are present in every one and
exaggerated in v2. What survives *no* pair of versions is the top bucket: it
falls in v1, climbs in v2, falls in v3, and splits in v4 (mean up, pooled
down). A bucket that changes direction under every measurement correction is
telling you it is unmeasured, not what its value is.

### Caveats

- **Rework ≠ defect.** A sampled classification of 85 PR-pairs splits edges
  ~57% planned iteration / 23% genuine correction / 20% collision. Absolute
  edge counts overstate churn; the size finding is ratio-based and survives.
- **Lag figures and the edge count are edge-weighted.** The median/percentile
  lag, "within 3 days" and the headline edge count are computed over
  (hunk × origin-commit) rows, each counting once regardless of how many lines
  it carries; only the bucket rates and the large-PR share are line-weighted.
  The two weightings answer slightly different questions and have not been
  cross-checked against each other.
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
   plus 9 of the 13 closed-unmerged PRs are skip-guard duplicates or
   stacked-PR orphans, meaning ~4% of the "237 PRs" figure is the same work
   re-counted. (237 = 221 merged + 13 closed-unmerged + 3 open at close-out;
   the merged count is stage 1's, the other two are `gh pr list` snapshots
   taken 2026-08-09.)
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
