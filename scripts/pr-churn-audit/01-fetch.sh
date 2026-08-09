#!/usr/bin/env bash
# Stage 1: pull the PR corpus and per-PR commit counts from GitHub.
#
# The commit count is not a nicety — stage 3 needs it to compute each PR's
# diff range. This repo rebase-merges multi-commit PRs, so a PR's recorded
# mergeCommit is only its LAST commit; `sha^1..sha` sees a fraction of it.
set -euo pipefail
WORK="${WORK:-./audit-work}"
REPO="${REPO:-storkme/spaghettio}"
SINCE="${SINCE:-2026-07-12}"
mkdir -p "$WORK"

echo "fetching merged PRs from $REPO..."
gh pr list --repo "$REPO" --state merged --limit 300 \
  --json number,title,mergedAt,mergeCommit,headRefName,additions,deletions,changedFiles \
  > "$WORK/prs_merged.json"
echo "  $(jq --arg s "$SINCE" '[.[]|select(.mergedAt>$s)]|length' "$WORK/prs_merged.json") merged since $SINCE"

echo "fetching per-PR commit counts (slow — one API call per PR)..."
: > "$WORK/pr_commits.tsv"
jq -r --arg s "$SINCE" '[.[]|select(.mergedAt>$s)]|.[].number' "$WORK/prs_merged.json" |
while read -r n; do
  c=$(gh pr view "$n" --repo "$REPO" --json commits --jq '.commits|length' 2>/dev/null || echo 1)
  printf '%s\t%s\n' "$n" "${c:-1}" >> "$WORK/pr_commits.tsv"
done
echo "  $(wc -l < "$WORK/pr_commits.tsv") rows"

echo "fetching per-PR review/latency stats..."
printf 'pr\tcommits\tcomments\treviews\thours\tadds\ttitle\n' > "$WORK/review_rounds.tsv"
jq -r --arg s "$SINCE" '[.[]|select(.mergedAt>$s)]|sort_by(.mergedAt)|.[].number' "$WORK/prs_merged.json" |
while read -r n; do
  gh pr view "$n" --repo "$REPO" \
    --json number,title,createdAt,mergedAt,commits,comments,reviews,additions 2>/dev/null |
  jq -r '[ .number, (.commits|length), (.comments|length), (.reviews|length),
           (((.mergedAt|fromdateiso8601)-(.createdAt|fromdateiso8601))/3600|floor),
           .additions, (.title|.[0:70]) ] | @tsv' >> "$WORK/review_rounds.tsv" || true
done
echo "  $(( $(wc -l < "$WORK/review_rounds.tsv") - 1 )) rows"
