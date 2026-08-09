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
mkdir -p "$WORK"

echo "fetching merged PRs from $REPO..."
# --limit well above the window's size: at 300 a later re-run could silently
# drop the OLDEST in-window PRs, moving numerator and denominator together with
# no warning.
gh pr list --repo "$REPO" --state merged --limit 1000 \
  --json number,title,mergedAt,mergeCommit,headRefName,additions,deletions,changedFiles \
  > "$WORK/prs_merged.json"
echo "  $(jq --arg s "$SINCE" --arg u "$UNTIL" '[.[]|select(.mergedAt>$s and .mergedAt<$u)]|length' "$WORK/prs_merged.json") merged in $SINCE..$UNTIL"

echo "fetching per-PR commit counts (slow — one API call per PR)..."
: > "$WORK/pr_commits.tsv"
# Every PR in the file, not just in-window: stage 2 builds its map over all of
# them, and a missing count silently degrades that PR to a 1-commit range,
# dropping its earlier commits from the map and losing edges that blame to them.
jq -r '.[].number' "$WORK/prs_merged.json" |
while read -r n; do
  c=$(gh pr view "$n" --repo "$REPO" --json commits --jq '.commits|length' 2>/dev/null || echo 1)
  printf '%s\t%s\n' "$n" "${c:-1}" >> "$WORK/pr_commits.tsv"
done
echo "  $(wc -l < "$WORK/pr_commits.tsv") rows"

echo "fetching per-PR review/latency stats..."
printf 'pr\tcommits\tcomments\treviews\thours\tadds\ttitle\n' > "$WORK/review_rounds.tsv"
jq -r --arg s "$BUCKET_SINCE" --arg u "$UNTIL" \
  '[.[]|select(.mergedAt>$s and .mergedAt<$u)]|sort_by(.mergedAt)|.[].number' "$WORK/prs_merged.json" |
while read -r n; do
  gh pr view "$n" --repo "$REPO" \
    --json number,title,createdAt,mergedAt,commits,comments,reviews,additions 2>/dev/null |
  jq -r '[ .number, (.commits|length), (.comments|length), (.reviews|length),
           (((.mergedAt|fromdateiso8601)-(.createdAt|fromdateiso8601))/3600|floor),
           .additions, (.title|.[0:70]) ] | @tsv' >> "$WORK/review_rounds.tsv" || true
done
echo "  $(( $(wc -l < "$WORK/review_rounds.tsv") - 1 )) rows"
