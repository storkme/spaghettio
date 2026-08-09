# PR churn audit pipeline

Reproduces the numbers behind the **change-size norm** in
[`CLAUDE.md`](../../CLAUDE.md#workflow-branches-review-merging) and the findings
in [`docs/pr-churn-audit-2026-08.md`](../../docs/pr-churn-audit-2026-08.md).

Checked in because that doc is labelled load-bearing and CLAUDE.md cites it as
the evidence for a standing rule. A norm whose numbers cannot be re-derived is
folklore.

```bash
export WORK=/tmp/audit-work          # scratch dir (default ./audit-work)
export SINCE=2026-07-12              # window start
bash scripts/pr-churn-audit/01-fetch.sh        # PR corpus + commit counts   (~5 min, API-bound)
bash scripts/pr-churn-audit/02-commit-map.sh   # commit -> PR map            (~1 min)
bash scripts/pr-churn-audit/03-blame-edges.sh  # rework edges via git blame  (~10 min)
bash scripts/pr-churn-audit/04-analyze.sh      # the headline numbers
```

Needs `gh` (authenticated), `jq`, and a full clone — stage 3 blames history, so
a shallow checkout silently produces nothing.

## Two mistakes this pipeline exists to not repeat

Both were made in the first version of the audit, both changed the answer, and
both were caught by someone else rather than by the author.

**The diff range.** This repo *rebase-merges* multi-commit PRs, so a PR's
recorded `mergeCommit` is only its **last** commit. Diffing `sha^1..sha` sees a
fraction of the change — measured on #317, 18 lines of one docs file out of 281
additions across 3 commits. It is worst where it matters most: PRs above 1k
additions average **13.6 commits** against 1.5 below 100. Stage 3 therefore
uses `sha~N..sha`, with `sha^1..sha^2` for true merge commits.

**The commit → PR map.** Parsing `(#N)` out of commit subjects does not work
here, because this project writes *issue* references into subjects and a regex
cannot distinguish them. That error mis-attributed **35% of commits** (322 of
907). Stage 2 derives the map from authoritative per-PR commit ranges instead.

Correcting both roughly doubled the edge count (1,639 → 3,114), moved the median
rework lag from 1 day to 2, and turned the size relationship from
1.6/2.7/6.3/**5.7** into 1.4/3.2/8.5/**10.5** — the dip in the top bucket had
been an artifact of the truncation, not a floor effect.

## Reading the output

`04-analyze.sh` prints **both** averaging methods per size bucket. They agree
across the first three, which is why the norm's threshold is 400. They disagree
in *direction* on the top bucket — mean rises to 10.5, pooled falls to 6.9 —
so `>1k` is unresolved on this data and neither figure should be quoted alone.

Buckets cover PRs with more than 20 added lines merged from the start of
`review_rounds.tsv`'s coverage; the share-of-rework figure uses the unfiltered
set. These denominators differ, and the audit doc's "Denominators" note explains
why. Reconcile before quoting them against each other.

## What is not here

The blame-edge dataset counts a later PR rewriting an earlier PR's lines. A
sampled classification put that at roughly **57% planned iteration / 23%
genuine correction / 20% collision** — so it is *not* a defect rate, and no
script here separates those. The ratios the norm rests on survive the
distinction; absolute counts do not.
