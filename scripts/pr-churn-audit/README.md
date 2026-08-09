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
bash scripts/pr-churn-audit/probe-v2-defect.sh "$WORK"   # re-derives the "50 of 221" range-defect figure
bash scripts/pr-churn-audit/probe-exposure.sh "$WORK"    # exposure-per-bucket (censoring confound check)
```

Needs `gh` (authenticated), `jq`, **GNU coreutils and GNU grep**, and a full
clone — stage 3 blames history, so on a shallow checkout the bases don't
resolve; every affected PR lands in `blame_failures.txt` and the INCOMPLETE
warning fires rather than quoting a truncated dataset.

The GNU requirement is not cosmetic and stage 3 now refuses to start without
it: BSD `date` makes `epoch()` return 0, so every age becomes 0 and the age
distribution collapses to a plausible-looking lie; BSD `grep` has no `-P`, so
every edge is dropped. Both fail quietly, which is why they are checked up
front.

**Two windows, deliberately.** `SINCE..UNTIL` bounds the corpus (edges, ages);
`BUCKET_SINCE..UNTIL` bounds the per-PR review pull the size buckets divide by.
They differ because the bucket data starts later.

**Do not hand-derive the denominators — stage 4 prints them**, together and
with the trap named:

```
review_rounds rows (unfiltered) : 220
bucket population (>20 adds)    : 197
of those, >=400 adds            : 71  (36% of the bucket population)
NB 71/220 = 32% is the UNFILTERED share
```

Quoting 32% against 197 mixes the two. Every denominator dispute in this
pipeline's review history — and there were several — came from one number being
copied into prose and not updated alongside its siblings. Transcribe the block
above; do not recompute it.

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
spanning other people's work. Measured, and re-derivable with
`probe-v2-defect.sh`: **50 of 221 in-window PRs (22.6%)**, size-correlated
(17.4% of PRs under 400 adds, 33.3% of those over) — i.e. skewed in the same
direction as the size finding it feeds.

Stage 2 therefore resolves each range by walking back from the merge commit and
**stopping at the first commit that announces a different PR** — `Merge pull
request #N`, or a trailing `(#N)` **that resolves to a known merged-PR number**.
The qualifier is mistake 3 applied to this walk: the trailing form is the same
regex that cannot distinguish PR refs from issue refs, so stopping at an
intermediate commit whose subject merely ends in an issue ref would silently
re-introduce the too-narrow defect. Candidates that fail the lookup are walked
through, not stopped at, and logged to `issue_ref_skips.txt`.

Subjects alone are still not enough: a squash-merged PR sitting on an
*unlabelled* rebase-merged PR would absorb that PR's commits without ever
meeting a labelled boundary, and no endpoint check can see what was absorbed
on the way. So PRs are processed in **merge order** and the walk also stops at
any commit **already claimed** by a previously-processed PR's range — the
subject-blind backstop for exactly that case. With both stops, the walk is
strategy-agnostic: for a squash it stops at depth 1, for a rebase it stops at
the previous PR's boundary, and true merge commits take `sha^1..sha^2`. The
resolved base is written once to `pr_base.tsv` and stage 3 consumes it rather
than re-deriving.

**3. The commit → PR map.** Parsing `(#N)` out of commit subjects does not work
here, because this project writes *issue* references into subjects and a regex
cannot distinguish them. That error mis-attributed **35% of commits** (322 of
907). Stage 2 derives the map from the resolved ranges instead.

### What each version produced

| Measure | v1 (narrow + regex) | v2 (wide) | v3 (walk) | v4 (current) |
|---|---:|---:|---:|---:|
| Edges | 1,639 | 3,114 | 1,999 | 2,022 |
| Median lag | 1d | 2d | 2d | 1d |
| Under 4 days (floored ages ≤3) | 61% | 57% | 60% | 61% |
| Rate 400–1k | 6.3 | 8.5 | 6.1 | 4.7 |
| Rate >1k | 5.7 | 10.5 | 6.0 | 5.7 |
| Large-PR share | 89.7% | 93.2% | 90.1% | 89.3% |

v2 is the one that was originally published, and it is the outlier. v3 lands
almost exactly on v1, whose two errors had partially cancelled. v4 is v3 plus
two review-caught fixes: the issue-ref boundary validation (negligible —
edges 1,999 → 2,005) and the paired `-w` (the real mover — 400–1k drops
6.1 → 4.7, i.e. roughly a quarter of that bucket's counted rework was
whitespace-only lines). The finding the norm rests on — the climb from <100
to 400–1k and ~90% of rework from PRs ≥400 adds — is present in all four;
its magnitude has moved with every correction, and the top bucket's
*direction* has never once been stable across versions.

## Reading the output

`04-analyze.sh` prints **both** averaging methods per size bucket. They agree
across the first three, which is why the norm's threshold is 400 — and they
**disagree in direction** on the top one (mean 4.7 → 5.7, pooled 4.6 → 3.6).
Do not quote either alone there: across v1→v4 the top bucket has fallen,
climbed, fallen and split, and it is right-censored besides (the audit doc's
"Denominators" and censoring notes). Treat >1k as unmeasured.

### What the rate actually measures

`reworked_totals` is keyed by the **reworked** PR — column 2 of
`rework_edges.tsv` — and divided by *that* PR's own additions. So the figure is
**"how much of what this PR wrote did not survive"**, not "how much rewriting
this PR did". That is the reading the norm needs: it claims a large PR's own
code gets rewritten more per line.

Two consequences that are correct but look like bugs at a glance:

- An in-window PR that rewrites lots of *old* code scores 0 in its own bucket.
  It is a reworker, not reworked, and the metric is not about reworkers.
- Edges whose reworked PR predates `BUCKET_SINCE` appear in the edge total but
  in no bucket, because that PR is not in the bucket population.

If you want the other reading — rework *caused* per PR — key on column 1
instead. It is a different question and the norm does not rest on it.

Buckets cover PRs with more than 20 added lines merged from the start of
`review_rounds.tsv`'s coverage. The share-of-rework figure is printed under
**both** denominators — the bucket population and the unfiltered set — because
the "36% of the population produced ~90% of rework" sentence needs its
numerator and denominator drawn from the same base, and an earlier revision
quoted the 36% against one and the 90% against the other. Note also that
either share counts only rework of code written by PRs inside
`review_rounds.tsv`'s window: rework of older code is in the edge total but in
neither share.

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

**Deletions count as rework; additions do not.** Stage 3 blames the *old* side
of each hunk, so removing 40 of an earlier PR's lines scores 40 edges against
it, while adding 40 new lines scores none. That asymmetry is deliberate — the
question is "how much of what this PR wrote did not survive" — but it means a
PR that deletes a stale subsystem inflates the rework attributed to whoever
wrote it. Keep it in mind when a single old PR shows a surprisingly large
reworked total.

**Whitespace-only churn is not counted.** Both the diff and the blame pass
`-w`, deliberately paired: with `-w` on blame alone, a reformat-only sweep
diffs whole blocks as deletions while the blame attributes those lines to
their *original* authors — a large fake rework spike against old PRs for a
semantic no-op.

**The age distribution and edge count are edge-weighted, not line-weighted.**
Each (hunk × origin-commit) row counts once in the lag percentiles and once in
the headline edge count, regardless of its `lines` column; only the bucket
rates and the large-PR share are line-weighted (they sum that column). A
40-line deletion and a 1-line edit pull equally on "median lag / under 4
days"; nothing here says lag would look the same weighted per-line.

**The boundary walk is a heuristic, not a proof.** Three flag classes land in
`range_unverified.txt`, all meaning "spot-check before quoting affected PRs":

- **ambiguous-base** — the walk ran to its cap and the resolved base announces
  no other PR: either a rebase onto an unlabelled commit (harmless) or a
  squash whose range is over-wide (the v2 defect). 11 on the current corpus.
- **boundary-stop-after-absorbing** — the walk absorbed anonymous commits and
  *then* hit a boundary: either a rebase whose `gh` count overestimated
  (harmless) or a squash sitting on a direct push to main, whose range is
  over-wide and whose absorbed commits' rework is misattributed. 4 on the
  current corpus. The round-8 review caught that the previous revision ran its
  ambiguity check only on walks that reached the cap, so this class passed as
  clean with no marker at all.
- **unreadable-merge-commit** — the merge commit's parents could not be read
  (partial clone); the PR is skipped and recorded rather than guessed at.

All 15 flagged PRs are numbered 144–306 — none inside the audit window, so
the published figures are unaffected. The flags are deliberately narrow: an
earlier marker fired for every clean multi-commit rebase, the dominant case,
and a warning that fires on everything is a warning about nothing.

One residual no flag can catch: a commit subject ending in `(#N)` where N is a
*merged PR's number* being used as an issue reference. At depth 1 that reads
as a clean squash boundary and silently truncates the range; deeper in, it at
least lands in boundary-stop-after-absorbing. Subjects cannot disambiguate
that collision — the ISPR lookup narrows the issue-ref hole to exactly it.
