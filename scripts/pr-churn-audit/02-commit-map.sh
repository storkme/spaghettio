#!/usr/bin/env bash
# Stage 2: build the commit -> PR map from AUTHORITATIVE per-PR commit ranges.
#
# Do NOT rebuild this by parsing "(#N)" out of commit subjects. This project
# writes ISSUE references into subjects, and a regex cannot tell an issue from
# a PR. The first version of this audit did exactly that and mis-attributed
# **35% of commits** (322 of 907) — verified on PR #317, whose three commits
# were all credited to issue 315.
#
# Rule per merged PR:
#   true merge commit (>=2 parents) -> every commit in sha^1..sha^2
#   otherwise (squash / rebase)     -> every commit in sha~N..sha  (N = commits)
# Later PRs win on duplicate commits (relands).
set -euo pipefail
WORK="${WORK:-./audit-work}"
REPO_DIR="${REPO_DIR:-$(git rev-parse --show-toplevel)}"
cd "$REPO_DIR"

declare -A NCOM
while IFS=$'\t' read -r pr c; do NCOM[$pr]=$c; done < "$WORK/pr_commits.tsv"

tmp="$WORK/.c2p.raw"; : > "$tmp"
jq -r '[.[]|select(.mergeCommit!=null)]|sort_by(.number)|.[]|"\(.number)\t\(.mergeCommit.oid)"' \
  "$WORK/prs_merged.json" |
while IFS=$'\t' read -r pr sha; do
  git cat-file -e "${sha}^{commit}" 2>/dev/null || continue
  np=$(git rev-list --parents -n1 "$sha" 2>/dev/null | wc -w)
  if [ "$np" -ge 3 ]; then
    rng="${sha}^1..${sha}^2"
  else
    n="${NCOM[$pr]:-1}"; [ "$n" -lt 1 ] && n=1
    if git rev-parse -q --verify "${sha}~${n}" >/dev/null 2>&1; then
      rng="${sha}~${n}..${sha}"
    else
      rng="${sha}^1..${sha}"
    fi
  fi
  git rev-list "$rng" 2>/dev/null | sed "s/\$/\t$pr/" >> "$tmp"
done

sort -k1,1 -k2,2n "$tmp" | awk -F'\t' '{m[$1]=$2} END{for(k in m) print k"\t"m[k]}' \
  > "$WORK/commit2pr.tsv"
rm -f "$tmp"
echo "commit2pr entries: $(wc -l < "$WORK/commit2pr.tsv")"
