#!/usr/bin/env bash
# Stage 1: pull the PR corpus and per-PR commit counts from GitHub.
#
# The commit count is not a nicety — stage 3 needs it to compute each PR's
# diff range. This repo rebase-merges multi-commit PRs, so a PR's recorded
# mergeCommit is only its LAST commit; `sha^1..sha` sees a fraction of it.
set -euo pipefail
WORK="${WORK:-./audit-work}"
REPO="${REPO:-storkme/spaghettio}"
# TWO windows, deliberately, because the published figures use two.
#   SINCE..UNTIL       the corpus: blame edges, age distribution, edge count.
#   BUCKET_SINCE..UNTIL the per-PR review/latency pull the size buckets divide by.
# They differ because the bucket data was collected later in the session, from
# 07-20. Quoting a bucket count against the corpus count without reconciling
# them is the mixed-denominator trap the audit doc warns about.
SINCE="${SINCE:-2026-07-12}"
BUCKET_SINCE="${BUCKET_SINCE:-2026-07-20}"
# UNTIL is load-bearing for reproducibility: without it, `gh pr list` keeps
# pulling PRs merged after the study and every n drifts upward over time.
UNTIL="${UNTIL:-2026-08-09}"
# End-of-day, NOT the bare date. mergedAt always carries a time, and
# "2026-08-09T09:49:40Z" < "2026-08-09" is FALSE lexicographically — a bare
# bound silently drops every PR merged on the closing day. Stage 3 uses the
# same expansion; if these two ever disagree the corpus and the bucket
# denominator cover different windows, which is the mixed-denominator trap
# this pipeline exists to document.
UNTIL_TS="${UNTIL}T23:59:59Z"
mkdir -p "$WORK"

echo "fetching merged PRs from $REPO..."
# --limit well above the window's size: at 300 a later re-run could silently
# drop the OLDEST in-window PRs, moving numerator and denominator together with
# no warning.
gh pr list --repo "$REPO" --state merged --limit 1000 \
  --json number,title,mergedAt,mergeCommit,headRefName,additions,deletions,changedFiles \
  > "$WORK/prs_merged.json"
echo "  $(jq --arg s "$SINCE" --arg u "$UNTIL_TS" '[.[]|select(.mergedAt>$s and .mergedAt<$u)]|length' "$WORK/prs_merged.json") merged in $SINCE..$UNTIL_TS"

echo "fetching per-PR commit counts (slow — one API call per PR)..."
: > "$WORK/pr_commits.tsv"
# Every PR in the file, not just in-window: stage 2 builds its map over all of
# them, and a missing count silently degrades that PR to a 1-commit range,
# dropping its earlier commits from the map and losing edges that blame to them.
# A failed fetch here is NOT harmless: stage 2 would build a 1-commit range for
# that PR, which is exactly the truncated-range defect this pipeline documents.
# Record failures and refuse to pretend the dataset is complete.
: > "$WORK/fetch_failures.txt"
jq -r '.[].number' "$WORK/prs_merged.json" |
while read -r n; do
  if c=$(gh pr view "$n" --repo "$REPO" --json commits --jq '.commits|length' 2>/dev/null) && [ -n "$c" ]; then
    printf '%s\t%s\n' "$n" "$c" >> "$WORK/pr_commits.tsv"
  else
    printf 'commit-count\t%s\n' "$n" >> "$WORK/fetch_failures.txt"
  fi
done
echo "  $(wc -l < "$WORK/pr_commits.tsv") rows"

echo "fetching per-PR review/latency stats..."
printf 'pr\tcommits\tcomments\treviews\thours\tadds\ttitle\n' > "$WORK/review_rounds.tsv"
jq -r --arg s "$BUCKET_SINCE" --arg u "$UNTIL_TS" \
  '[.[]|select(.mergedAt>$s and .mergedAt<$u)]|sort_by(.mergedAt)|.[].number' "$WORK/prs_merged.json" |
while read -r n; do
  gh pr view "$n" --repo "$REPO" \
    --json number,title,createdAt,mergedAt,commits,comments,reviews,additions 2>/dev/null |
  jq -r '[ .number, (.commits|length), (.comments|length), (.reviews|length),
           (((.mergedAt|fromdateiso8601)-(.createdAt|fromdateiso8601))/3600|floor),
           .additions, (.title|.[0:70]) ] | @tsv' >> "$WORK/review_rounds.tsv" \
    || printf 'review-row\t%s\n' "$n" >> "$WORK/fetch_failures.txt"
done
echo "  $(( $(wc -l < "$WORK/review_rounds.tsv") - 1 )) rows"

nf=$(wc -l < "$WORK/fetch_failures.txt")
if [ "$nf" -gt 0 ]; then
  echo
  echo "WARNING: $nf per-PR fetch(es) failed — the dataset is INCOMPLETE."
  echo "         A missing commit count becomes a 1-commit range in stage 2 (the"
  echo "         truncated-range defect); a missing review row silently shrinks a"
  echo "         bucket denominator. See $WORK/fetch_failures.txt."
  echo "         Re-run stage 1 until this is empty before quoting any figure."
fi
