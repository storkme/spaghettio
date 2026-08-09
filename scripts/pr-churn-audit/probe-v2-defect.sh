#!/usr/bin/env bash
# Not a pipeline stage — a probe that re-derives one quoted figure: how many
# in-window PRs get a WRONG diff range under the v2 strategy (bare sha~N for
# squash/rebase merges, sha^1 for true merge commits), compared against stage
# 2's resolved bases. The "50 of 221 in-window PRs (22%)" sentence in the
# README, the audit doc and 02-commit-map.sh's header comes from here.
#
# Run AFTER stages 1-2:  probe-v2-defect.sh "$WORK"
set -euo pipefail
WORK="${1:-${WORK:-./audit-work}}"
SINCE="${SINCE:-2026-07-12}"
UNTIL="${UNTIL:-2026-08-09}"; UNTIL_TS="${UNTIL}T23:59:59Z"
cd "$(git rev-parse --show-toplevel)"
declare -A NCOM BASE
while IFS=$'\t' read -r pr c; do NCOM[$pr]=$c; done < "$WORK/pr_commits.tsv"
while IFS=$'\t' read -r pr b; do BASE[$pr]=$b; done < "$WORK/pr_base.tsv"
tot=0; wrong=0; small=0; wrong_small=0; big=0; wrong_big=0
while IFS=$'\t' read -r pr sha adds; do
  resolved="${BASE[$pr]:-}"; [ -z "$resolved" ] && continue
  tot=$((tot+1))
  np=$(git rev-list --parents -n1 "$sha" 2>/dev/null | wc -w)
  if [ "$np" -ge 3 ]; then v2base="${sha}^1"
  else cap="${NCOM[$pr]:-1}"; [ "$cap" -lt 1 ] && cap=1; v2base="${sha}~${cap}"; fi
  v2=$(git rev-parse -q --verify "$v2base" 2>/dev/null || echo MISSING)
  res=$(git rev-parse -q --verify "$resolved" 2>/dev/null || echo NONE)
  w=0; [ "$v2" != "$res" ] && { wrong=$((wrong+1)); w=1; }
  if [ "$adds" -ge 400 ]; then big=$((big+1)); wrong_big=$((wrong_big+w))
  else small=$((small+1)); wrong_small=$((wrong_small+w)); fi
done < <(jq -r --arg s "$SINCE" --arg u "$UNTIL_TS" \
  '[.[]|select(.mergedAt>$s and .mergedAt<$u and .mergeCommit!=null)]
   |.[]|"\(.number)\t\(.mergeCommit.oid)\t\(.additions)"' "$WORK/prs_merged.json")
echo "v2 range wrong: $wrong / $tot in-window PRs ($(( 100*wrong/tot ))%)"
echo "  <400 adds : $wrong_small/$small ($(( 100*wrong_small/small ))%)"
echo "  >=400 adds: $wrong_big/$big ($(( 100*wrong_big/big ))%)"
