# PR churn audit pipeline

Reproduces the numbers behind the **change-size norm** in
[`CLAUDE.md`](../../CLAUDE.md#workflow-branches-review-merging) and the findings
in [`docs/pr-churn-audit-2026-08.md`](../../docs/pr-churn-audit-2026-08.md).

Checked in because that doc is labelled load-bearing and CLAUDE.md cites it as
the evidence for a standing rule. A norm whose numbers cannot be re-derived is
folklore.

```bash
export WORK=/tmp/audit-work          # scratch dir (default ./audit-work)
export SINCE=2026-07-12              # corpus window start
export BUCKET_SINCE=2026-07-20       # size-bucket window start (see below)
export UNTIL=2026-08-09              # window end — omit it and n drifts forever
bash scripts/pr-churn-audit/01-fetch.sh        # PR corpus + commit counts   (~5 min, API-bound)
bash scripts/pr-churn-audit/02-commit-map.sh   # commit -> PR map            (~1 min)
bash scripts/pr-churn-audit/03-blame-edges.sh  # rework edges via git blame  (~10 min)
bash scripts/pr-churn-audit/04-analyze.sh      # the headline numbers
```

Needs `gh` (authenticated), `jq`, **GNU coreutils and GNU grep**, and a full
clone — stage 3 blames history, so a shallow checkout silently produces nothing.

The GNU requirement is not cosmetic and stage 3 now refuses to start without
it: BSD `date` makes `epoch()` return 0, so every age becomes 0 and the age
distribution collapses to a plausible-looking lie; BSD `grep` has no `-P`, so
every edge is dropped. Both fail quietly, which is why they are checked up
front.

**Two windows, deliberately.** `SINCE..UNTIL` bounds the corpus (edges, ages);
`BUCKET_SINCE..UNTIL` bounds the per-PR review pull the size buckets divide by.
They differ because the two datasets were collected at different points in the
session — 218 merged PRs in the corpus, 217 in the bucket pull, 194 after the
>20-adds floor. Do not quote a bucket `n` against a corpus `n` without
reconciling them; that is the mixed-denominator trap the audit doc warns about.

## Three mistakes this pipeline exists to not repeat

All three were made while producing these numbers, all three changed the
answer, and every one was caught by a reviewer rather than by the author.

**1. The diff range, too narrow.** A PR's recorded `mergeCommit` is only its
**last** commit when the PR was rebase-merged. Diffing `sha^1..sha` then sees a
fraction of the change — measured on #317, 18 lines of one docs file out of 281
additions across 3 commits. Worst where it matters: PRs above 1k additions
average **13.6 commits** against 1.5 below 100.

**2. The diff range, too wide.** The obvious fix — `sha~N..sha`, with N from
`gh pr view --json commits` — is also wrong, and this is the subtle one. `gh`
reports the PR *branch's* commit count, but the number of commits that reach
main depends on the merge strategy, and **this repo uses all three**. A
squash-merged PR contributes exactly one commit while `gh` still reports N, so
`sha~N` walks N−1 commits back into *earlier* PRs; stage 3 then blames a range
spanning other people's work. Measured: **48 of 218 in-window PRs (22%)**, and
size-correlated (17% of PRs under 400 adds, 34% of those over) — i.e. skewed in
the same direction as the size finding it feeds.

Stage 2 therefore resolves each range by walking back from the merge commit and
**stopping at the first commit that announces a different PR** (`Merge pull
request #N`, or a trailing `(#N)`). That is strategy-agnostic: for a squash it
stops at depth 1, for a rebase it stops at the previous PR's boundary, and true
merge commits take `sha^1..sha^2`. The resolved base is written once to
`pr_base.tsv` and stage 3 consumes it rather than re-deriving.

**3. The commit → PR map.** Parsing `(#N)` out of commit subjects does not work
here, because this project writes *issue* references into subjects and a regex
cannot distinguish them. That error mis-attributed **35% of commits** (322 of
907). Stage 2 derives the map from the resolved ranges instead.

### What each version produced

| Measure | v1 (narrow + regex) | v2 (wide) | v3 (current) |
|---|---:|---:|---:|
| Edges | 1,639 | 3,114 | 1,841 |
| Median lag | 1d | 2d | 1d |
| Within 3 days | 61% | 57% | 65% |
| Rate 400–1k | 6.3 | 8.5 | 6.3 |
| Rate >1k | 5.7 | 10.5 | 6.0 |
| Large-PR share | 89.7% | 93.2% | 90.1% |

v2 is the one that was published, and it is the outlier. v3 lands almost
exactly on v1, whose two errors had partially cancelled. The finding the norm
rests on — a ~4× climb from <100 to 400–1k, and ~90% of rework from PRs ≥400
adds — is present in all three; only its magnitude moved.

## Reading the output

`04-analyze.sh` prints **both** averaging methods per size bucket. They agree
across the first three, which is why the norm's threshold is 400. They disagree
in *direction* on the top bucket — mean rises to 10.5, pooled falls to 6.9 —
so `>1k` is unresolved on this data and neither figure should be quoted alone.

Buckets cover PRs with more than 20 added lines merged from the start of
`review_rounds.tsv`'s coverage; the share-of-rework figure uses the unfiltered
set. These denominators differ, and the audit doc's "Denominators" note explains
why. Reconcile before quoting them against each other.

### Scope of "reproduces exactly"

These scripts regenerate the **blame-edge, age-distribution and size-bucket**
figures — the ones CLAUDE.md's norm rests on. They do **not** regenerate the
rest of the audit: the corrective-PR rate, the merge-latency bands, the
within-PR complexity control, the day-level concurrency correlation, and the
warm-review scoring were all hand-classified or one-off, and are recorded in
the docs rather than automated. Treat "reproduces every figure" as scoped to
what stage 4 prints.

Also note the numerator/denominator scope mismatch stage 4 now prints: rework
is blamed only over `crates/*.rs` and `web/src/*.ts`, while `adds` is GitHub's
total additions across all files. The ratio is consistent across buckets so the
comparison holds, but it is not literally "per 100 added lines".

# What is not here

The blame-edge dataset counts a later PR rewriting an earlier PR's lines. A
sampled classification put that at roughly **57% planned iteration / 23%
genuine correction / 20% collision** — so it is *not* a defect rate, and no
script here separates those. The ratios the norm rests on survive the
distinction; absolute counts do not.
