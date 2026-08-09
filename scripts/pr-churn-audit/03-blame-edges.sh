#!/usr/bin/env bash
# Stage 3: for each merged PR, blame the lines it deleted/modified back to the
# PR that introduced them. Emits rework edges:
#
#   reworker_pr  reworked_pr  file  age_days  lines
#
# Diff range matters. This repo REBASE-MERGES multi-commit PRs, so the PR's
# mergeCommit is only its last commit and `sha^1..sha` sees a fraction of the
# change. Measured on #317: 281 additions across 3 commits, of which sha^1..sha
# showed 18 lines in one docs file. Worst on large PRs, which average 13.6
# commits above 1k adds vs 1.5 below 100 — i.e. the truncation is heaviest
# exactly where the headline finding lives.
#
# Slow: one `git blame` per changed hunk. Budget ~10 min for ~220 PRs.
set -euo pipefail
WORK="${WORK:-./audit-work}"
REPO_DIR="${REPO_DIR:-$(git rev-parse --show-toplevel)}"
SINCE="${SINCE:-2026-07-12}"
cd "$REPO_DIR"

declare -A C2P
while IFS=$'\t' read -r sha pr; do C2P[$sha]=$pr; done < "$WORK/commit2pr.tsv"
declare -A PRDATE
while IFS=$'\t' read -r pr d; do PRDATE[$pr]=$d; done < <(
  jq -r '.[]|"\(.number)\t\(.mergedAt)"' "$WORK/prs_merged.json")
declare -A NCOM
while IFS=$'\t' read -r pr c; do NCOM[$pr]=$c; done < "$WORK/pr_commits.tsv"

epoch() { date -u -d "$1" +%s 2>/dev/null || echo 0; }
out="$WORK/rework_edges.tsv"; : > "$out"

jq -r --arg s "$SINCE" '[.[]|select(.mergedAt>$s and .mergeCommit!=null)]
  | sort_by(.mergedAt) | .[] | "\(.number)\t\(.mergeCommit.oid)\t\(.mergedAt)"' \
  "$WORK/prs_merged.json" |
while IFS=$'\t' read -r pr sha mdate; do
  np=$(git rev-list --parents -n1 "$sha" 2>/dev/null | wc -w)
  if [ "$np" -ge 3 ]; then
    base="${sha}^1"
  else
    n="${NCOM[$pr]:-1}"; [ "$n" -lt 1 ] && n=1
    base="${sha}~${n}"
  fi
  git rev-parse -q --verify "$base" >/dev/null 2>&1 || base="${sha}^1"
  git rev-parse -q --verify "$base" >/dev/null 2>&1 || continue

  git diff --unified=0 --no-color "$base" "$sha" -- 'crates/*.rs' 'web/src/*.ts' 2>/dev/null |
  awk '
    /^--- a\// { f=substr($0,7); next }
    /^\+\+\+ /  { next }
    /^@@/ {
      match($0, /-[0-9]+(,[0-9]+)?/); spec=substr($0,RSTART+1,RLENGTH-1)
      split(spec,a,","); st=a[1]; cn=(length(a)>1?a[2]:1)
      if (cn>0 && f!="") print f "\t" st "\t" cn
    }' |
  while IFS=$'\t' read -r f st cn; do
    en=$((st+cn-1))
    git blame -w --line-porcelain -L "${st},${en}" "$base" -- "$f" 2>/dev/null |
      grep -oP '^[0-9a-f]{40}(?= )' | sort | uniq -c |
      while read -r nl bsha; do
        opr="${C2P[$bsha]:-}"; [ -z "$opr" ] && continue
        [ "$opr" = "$pr" ] && continue
        od="${PRDATE[$opr]:-}"; [ -z "$od" ] && continue
        age=$(( ( $(epoch "$mdate") - $(epoch "$od") ) / 86400 ))
        [ "$age" -lt 0 ] && continue
        printf '%s\t%s\t%s\t%s\t%s\n' "$pr" "$opr" "$f" "$age" "$nl" >> "$out"
      done
  done
done
echo "rework edges: $(wc -l < "$out")"
