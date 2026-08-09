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
. "$(dirname "$0")/window.env"
cd "$(git rev-parse --show-toplevel)"
declare -A NCOM BASE
while IFS=$'\t' read -r pr c; do NCOM[$pr]=$c; done < "$WORK/pr_commits.tsv"
while IFS=$'\t' read -r pr b; do BASE[$pr]=$b; done < "$WORK/pr_base.tsv"
tot=0; wrong=0; small=0; wrong_small=0; big=0; wrong_big=0; skipped=0
while IFS=$'\t' read -r pr sha adds; do
  resolved="${BASE[$pr]:-}"
  [ -z "$resolved" ] && { skipped=$((skipped+1)); continue; }
  np=$(git rev-list --parents -n1 "$sha" 2>/dev/null | wc -w) || np=0
  [ "$np" -eq 0 ] && { skipped=$((skipped+1)); continue; }
  tot=$((tot+1))
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
# awk, not $((...)): integer division floors 22.6% to 22%, and an empty size
# class would make it a fatal divide-by-zero under set -e.
awk -v w="$wrong" -v t="$tot" -v ws="$wrong_small" -v s="$small" \
    -v wb="$wrong_big" -v b="$big" 'BEGIN{
  printf "v2 range wrong: %d / %d in-window PRs (%.1f%%)\n", w, t, (t?100*w/t:0)
  printf "  <400 adds : %d/%d (%.1f%%)\n", ws, s, (s?100*ws/s:0)
  printf "  >=400 adds: %d/%d (%.1f%%)\n", wb, b, (b?100*wb/b:0)
}'
echo "  (a disagreement rate between v2's heuristic and stage 2's resolution —"
echo "   the walk is the better-validated heuristic, not ground truth)"
if [ "$skipped" -gt 0 ]; then
  echo "WARNING: $skipped in-window PR(s) skipped (no stage-2 base, or object"
  echo "         unreadable) — they are missing from BOTH numerator and"
  echo "         denominator above. Fix stage 2 coverage before quoting this."
fi
