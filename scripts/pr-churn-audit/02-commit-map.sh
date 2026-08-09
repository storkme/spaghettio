#!/usr/bin/env bash
# Stage 2: resolve each PR's OWN commit range, then build the commit -> PR map.
#
# Writes two files:
#   pr_base.tsv     pr -> base sha   (the single source of truth for stage 3)
#   commit2pr.tsv   commit -> pr
#
# ---------------------------------------------------------------------------
# Why the range is not simply `sha~N..sha`
#
# `gh pr view --json commits` reports the PR BRANCH's commit count. How many
# commits that PR actually contributed to main depends on the merge strategy,
# and this repo uses all three:
#
#   squash  -> ONE commit on main, but gh still reports N. `sha~N` then walks
#              N-1 commits up into PREVIOUS PRs.
#   rebase  -> N commits on main. `sha~N` is correct.
#   merge   -> a 2-parent commit; the PR's work is sha^1..sha^2.
#
# Measured on this corpus: trusting `sha~N` blindly gave a wrong range for
# **48 of 218 in-window PRs (22%)**, and the error was size-correlated (17% of
# PRs under 400 adds, 34% of those over) — i.e. biased in the same direction as
# the size finding the audit publishes. Do not reintroduce it.
#
# The fix: walk back from the merge commit but STOP at the first commit that
# belongs to a different PR. A boundary commit is one whose subject announces a
# PR — either `Merge pull request #N` or a trailing `(#N)`. For a squash merge
# the very next commit back is another PR's boundary, so the walk stops at
# depth 1, which is correct.
# ---------------------------------------------------------------------------
set -euo pipefail
WORK="${WORK:-./audit-work}"
REPO_DIR="${REPO_DIR:-$(git rev-parse --show-toplevel)}"
cd "$REPO_DIR"

declare -A NCOM
while IFS=$'\t' read -r pr c; do NCOM[$pr]=$c; done < "$WORK/pr_commits.tsv"

# Which PR does a commit's own subject announce, if any? Empty = not a boundary.
boundary_pr() {
  local subj="$1"
  if [[ "$subj" =~ ^Merge\ pull\ request\ \#([0-9]+) ]]; then
    echo "${BASH_REMATCH[1]}"
  elif [[ "$subj" =~ \(#([0-9]+)\)[[:space:]]*$ ]]; then
    echo "${BASH_REMATCH[1]}"
  fi
}

: > "$WORK/pr_base.tsv"
: > "$WORK/.c2p.raw"
overwalks=0

while IFS=$'\t' read -r pr sha; do
  git cat-file -e "${sha}^{commit}" 2>/dev/null || continue
  np=$(git rev-list --parents -n1 "$sha" | wc -w)

  if [ "$np" -ge 3 ]; then
    # True merge commit: the PR's work is the second-parent branch.
    base="${sha}^1"
    git rev-list "${sha}^1..${sha}^2" | sed "s/\$/\t$pr/" >> "$WORK/.c2p.raw"
    echo "$sha" | sed "s/\$/\t$pr/" >> "$WORK/.c2p.raw"
  else
    cap="${NCOM[$pr]:-1}"; [ "$cap" -lt 1 ] && cap=1
    depth=1
    echo "$sha" | sed "s/\$/\t$pr/" >> "$WORK/.c2p.raw"
    while [ "$depth" -lt "$cap" ]; do
      cand="${sha}~${depth}"
      git rev-parse -q --verify "$cand" >/dev/null 2>&1 || break
      csubj=$(git log -1 --format='%s' "$cand")
      owner=$(boundary_pr "$csubj")
      # Stop before absorbing a commit that announces a different PR.
      if [ -n "$owner" ] && [ "$owner" != "$pr" ]; then
        overwalks=$((overwalks+1)); break
      fi
      git rev-parse "$cand" | sed "s/\$/\t$pr/" >> "$WORK/.c2p.raw"
      depth=$((depth+1))
    done
    base="${sha}~${depth}"
    git rev-parse -q --verify "$base" >/dev/null 2>&1 || base="${sha}^1"
  fi
  printf '%s\t%s\n' "$pr" "$base" >> "$WORK/pr_base.tsv"
done < <(jq -r '[.[]|select(.mergeCommit!=null)]|sort_by(.number)|.[]|"\(.number)\t\(.mergeCommit.oid)"' \
           "$WORK/prs_merged.json")

sort -k1,1 -k2,2n "$WORK/.c2p.raw" | awk -F'\t' '{m[$1]=$2} END{for(k in m) print k"\t"m[k]}' \
  > "$WORK/commit2pr.tsv"
rm -f "$WORK/.c2p.raw"

echo "pr_base entries : $(wc -l < "$WORK/pr_base.tsv")"
echo "commit2pr       : $(wc -l < "$WORK/commit2pr.tsv")"
echo "walks truncated at another PR's boundary: $overwalks"
echo "  (each of these would have been an over-wide range under a bare sha~N)"
