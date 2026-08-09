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
# Measured on this corpus (probe-v2-defect.sh): trusting `sha~N` blindly gives
# a wrong range for **50 of 221 in-window PRs (22.6%)**, and the error is
# size-correlated (17.4% of PRs under 400 adds, 33.3% of those over) — biased
# in the same direction as the size finding the audit publishes. Do not
# reintroduce it.
#
# The fix: walk back from the merge commit but STOP at the first commit that
# belongs to a different PR. A boundary commit is one whose subject announces a
# PR — either `Merge pull request #N` or a trailing `(#N)`. For a squash merge
# the very next commit back is another PR's boundary, so the walk stops at
# depth 1, which is correct.
#
# But a trailing `(#N)` is exactly the regex that mistake 3 (see README) says
# cannot distinguish PR refs from ISSUE refs — this project writes issue
# numbers into subjects, and that error once mis-attributed 35% of commits. An
# intermediate commit of a rebase-merged PR whose subject happens to end in
# `(#<issue>)` would read as another PR's boundary and silently truncate the
# range (the v1 too-narrow defect, returning). So a trailing `(#N)` only counts
# as a boundary if N is an actual merged-PR number from prs_merged.json; the
# `Merge pull request #N` form needs no such check because nothing writes issue
# refs in that shape. Rejected candidates are logged, not silently skipped.
# ---------------------------------------------------------------------------
set -euo pipefail
WORK="${WORK:-./audit-work}"
REPO_DIR="${REPO_DIR:-$(git rev-parse --show-toplevel)}"
cd "$REPO_DIR"

# Cross-stage guard: stage 1's failures degrade THIS stage (a missing commit
# count silently becomes a 1-commit range — the truncated-range defect), and a
# stage-scoped warning is easy to miss on a partial re-run days later.
if [ -s "$WORK/fetch_failures.txt" ]; then
  echo "WARNING: stage 1 recorded $(wc -l < "$WORK/fetch_failures.txt") fetch failure(s)."
  echo "         A missing commit count becomes a 1-commit range HERE. Re-run"
  echo "         stage 1 until fetch_failures.txt is empty before trusting this."
fi

declare -A NCOM
while IFS=$'\t' read -r pr c; do NCOM[$pr]=$c; done < "$WORK/pr_commits.tsv"

# Known merged-PR numbers: the referee for the ambiguous `(#N)` form. Complete
# as long as stage 1's --limit was not hit, which stage 1 now checks.
declare -A ISPR
while read -r n; do ISPR[$n]=1; done < <(jq -r '.[].number' "$WORK/prs_merged.json")

# Which PR does a commit's own subject announce, if any? Empty = not a boundary.
# A trailing `(#N)` where N is not a known merged PR is an issue ref, not a
# boundary — announce the rejection on fd 3 so the caller can count it, unless
# the caller passes `quiet` (the cap-ancestor confirmation probes commits that
# were never walk candidates; logging those would inflate the skips count).
boundary_pr() {
  local subj="$1" mode="${2:-log}"
  if [[ "$subj" =~ ^Merge\ pull\ request\ \#([0-9]+) ]]; then
    echo "${BASH_REMATCH[1]}"
  elif [[ "$subj" =~ \(#([0-9]+)\)[[:space:]]*$ ]]; then
    if [ -n "${ISPR[${BASH_REMATCH[1]}]:-}" ]; then
      echo "${BASH_REMATCH[1]}"
    elif [ "$mode" != quiet ]; then
      echo "issue-ref (#${BASH_REMATCH[1]}) not a merged PR: ${subj:0:60}" >&3
    fi
  fi
}

: > "$WORK/pr_base.tsv"
: > "$WORK/.c2p.raw"
: > "$WORK/range_unverified.txt"
exec 3> "$WORK/issue_ref_skips.txt"
overwalks=0
caphits=0
claimstops=0
earlystops=0

# Commits already assigned to a processed PR. PRs are processed in MERGE order
# (sort_by(.mergedAt), not number), so by the time a PR's walk runs, every PR
# merged before it has claimed its own commits — and a walk that reaches one of
# them has walked out of its own range. Subject parsing alone cannot catch
# this: a squash-merged PR sitting on an *unlabelled* rebase-merged PR absorbs
# those commits without ever meeting a labelled boundary, and the cap-ancestor
# check below only inspects the endpoint, not what was absorbed on the way.
declare -A CLAIMED

while IFS=$'\t' read -r pr sha; do
  git cat-file -e "${sha}^{commit}" 2>/dev/null || continue
  np=$(git rev-list --parents -n1 "$sha" 2>/dev/null | wc -w) || np=0
  if [ "$np" -eq 0 ]; then
    printf '%s\tunreadable-merge-commit\t%s\n' "$pr" "$sha" >> "$WORK/range_unverified.txt"
    continue
  fi

  if [ "$np" -ge 3 ]; then
    # True merge commit: the PR's work is the second-parent branch.
    base="${sha}^1"
    while read -r bsha; do
      # First claim wins here too — a branch that contains another PR's
      # commits (branched off it) must not re-claim them, or CLAIMED's
      # in-walk stops would disagree with the final first-claim dedup.
      if [ -z "${CLAIMED[$bsha]:-}" ]; then
        printf '%s\t%s\n' "$bsha" "$pr" >> "$WORK/.c2p.raw"; CLAIMED[$bsha]=$pr
      fi
    done < <(git rev-list "${sha}^1..${sha}^2"; echo "$sha")
  else
    cap="${NCOM[$pr]:-1}"; [ "$cap" -lt 1 ] && cap=1
    depth=1
    hit_boundary=0
    printf '%s\t%s\n' "$sha" "$pr" >> "$WORK/.c2p.raw"; CLAIMED[$sha]=$pr
    while [ "$depth" -lt "$cap" ]; do
      csha=$(git rev-parse -q --verify "${sha}~${depth}" 2>/dev/null) || break
      # Already another PR's commit? Then the walk has left its own range —
      # stop regardless of what the subject says.
      if [ -n "${CLAIMED[$csha]:-}" ] && [ "${CLAIMED[$csha]}" != "$pr" ]; then
        claimstops=$((claimstops+1)); hit_boundary=1; break
      fi
      csubj=$(git log -1 --format='%s' "$csha")
      owner=$(boundary_pr "$csubj")
      # Stop before absorbing a commit that announces a different PR.
      if [ -n "$owner" ] && [ "$owner" != "$pr" ]; then
        overwalks=$((overwalks+1)); hit_boundary=1; break
      fi
      printf '%s\t%s\n' "$csha" "$pr" >> "$WORK/.c2p.raw"; CLAIMED[$csha]=$pr
      depth=$((depth+1))
    done
    # A boundary stop at depth > 1 means the walk ABSORBED depth-1 anonymous
    # commits (unlabelled, unclaimed) before meeting the boundary. For a
    # rebase-merged PR those are its own commits (gh's count overestimated);
    # for a squash-merged PR they are foreign — e.g. a direct push to main —
    # and the range is over-wide with the push's rework misattributed to this
    # PR. Indistinguishable from subjects alone, so FLAG it; the earlier code
    # ran its ambiguity check only on the hit_boundary=0 path, which let this
    # exact case pass as clean (round-8 review, 3/3 passes).
    if [ "$hit_boundary" -eq 1 ] && [ "$depth" -gt 1 ]; then
      earlystops=$((earlystops+1))
      printf '%s\tboundary-stop-after-absorbing\tabsorbed=%s\tcap=%s\n' \
        "$pr" "$((depth-1))" "$cap" >> "$WORK/range_unverified.txt"
    fi
    # The loop bound stops at depth-1, so the cap-th ancestor was never
    # inspected. Look at it now — that is what distinguishes the two cases:
    #
    #   sha~cap announces ANOTHER PR  -> clean rebase-merge sitting exactly on
    #                                    the previous PR's boundary. Expected.
    #   sha~cap announces nothing     -> genuinely ambiguous: either a rebase
    #                                    onto an unlabelled commit, or a squash
    #                                    whose range is now over-wide (the v2
    #                                    defect). Flag it.
    #
    # Without this check the marker fired for every clean multi-commit rebase —
    # i.e. the dominant case — which made it useless for spotting the harmful
    # one. A warning that fires on everything is a warning about nothing.
    if [ "$hit_boundary" -eq 0 ] && [ "$depth" -gt 1 ]; then
      capsubj=$(git log -1 --format='%s' "${sha}~${depth}" 2>/dev/null || echo "")
      capowner=$(boundary_pr "$capsubj" quiet)
      if [ -z "$capowner" ] || [ "$capowner" = "$pr" ]; then
        caphits=$((caphits+1))
        printf '%s\tambiguous-base\tdepth=%s\tbase_subject=%s\n' \
          "$pr" "$depth" "${capsubj:0:60}" >> "$WORK/range_unverified.txt"
      fi
    fi
    base="${sha}~${depth}"
    git rev-parse -q --verify "$base" >/dev/null 2>&1 || base="${sha}^1"
  fi
  printf '%s\t%s\n' "$pr" "$base" >> "$WORK/pr_base.tsv"
done < <(jq -r '[.[]|select(.mergeCommit!=null)]|sort_by(.mergedAt,.number)|.[]|"\(.number)\t\(.mergeCommit.oid)"' \
           "$WORK/prs_merged.json")

# First claim wins, deterministically: rows append in MERGE order and the
# CLAIMED array already stops later walks at a claimed commit, so keeping the
# first occurrence matches the walk's own semantics. (The old
# sort|last-write dedup kept the highest PR number instead — a second,
# unrelated tie-break that disagreed with CLAIMED on any overlap.)
awk -F'\t' '!($1 in m){m[$1]=$2} END{for(k in m) print k"\t"m[k]}' "$WORK/.c2p.raw" \
  > "$WORK/commit2pr.tsv"
rm -f "$WORK/.c2p.raw"

exec 3>&-
echo "pr_base entries : $(wc -l < "$WORK/pr_base.tsv")"
echo "commit2pr       : $(wc -l < "$WORK/commit2pr.tsv")"
echo "walks truncated at another PR's boundary: $overwalks"
echo "  (each of these would have been an over-wide range under a bare sha~N)"
echo "walks truncated at a commit another PR already claimed: $claimstops"
echo "  (subject parsing alone would have absorbed these — the squash-over-"
echo "   unlabelled-rebase case no boundary regex can see)"
skips=$(wc -l < "$WORK/issue_ref_skips.txt")
if [ "$skips" -gt 0 ]; then
  echo "trailing (#N) refs rejected as issue refs, walk continued: $skips"
  echo "  (each would have silently truncated a range if trusted as a boundary;"
  echo "   listed in $WORK/issue_ref_skips.txt)"
fi
if [ "$earlystops" -gt 0 ]; then
  echo "boundary stops that first absorbed anonymous commits: $earlystops"
  echo "  (rebase-with-overcounted-cap or squash-over-direct-push — cannot tell"
  echo "   which from subjects; flagged in range_unverified.txt, spot-check them)"
fi
if [ "$caphits" -gt 0 ]; then
  echo "NOTE: $caphits PR(s) resolved to a base that announces no other PR, so the"
  echo "      range could not be confirmed as ending at the previous PR's boundary."
  echo "      Either a rebase onto an unlabelled commit (harmless) or a squash whose"
  echo "      range is over-wide (the v2 defect). Listed with their base subject in"
  echo "      $WORK/range_unverified.txt — spot-check before quoting affected PRs."
else
  echo "every multi-commit range ended on another PR's boundary (none ambiguous)"
fi
