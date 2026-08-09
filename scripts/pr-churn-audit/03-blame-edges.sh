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
# NOTE: deliberately NOT `set -e`. A single `git blame` failure (exit 128 on an
# out-of-range -L, a missing object, a transient error) would otherwise abort
# the whole stage mid-corpus and leave rework_edges.tsv SILENTLY SHORT — every
# PR after the failing hunk dropped with no indication. For a script whose
# entire purpose is re-derivable numbers, a silent truncation is the worst
# available failure. Per-hunk failures are counted and reported instead.
set -uo pipefail
WORK="${WORK:-./audit-work}"
REPO_DIR="${REPO_DIR:-$(git rev-parse --show-toplevel)}"
SINCE="${SINCE:-2026-07-12}"
# End-of-day, not the bare date: "2026-08-09T00:00:00Z" < "2026-08-09" is FALSE
# lexicographically, so a bare bound silently drops the entire closing day.
UNTIL="${UNTIL:-2026-08-09}"; UNTIL_TS="${UNTIL}T23:59:59Z"

# GNU coreutils / GNU grep required — see README. Fail loudly here rather than
# producing a plausible-looking but wrong dataset: BSD `date` makes every age 0,
# and `grep -P` errors drop every edge.
date -u -d "2026-01-01" +%s >/dev/null 2>&1 || {
  echo "ERROR: GNU date required (BSD date makes every age 0, silently)." >&2; exit 2; }
echo x | grep -qoP 'x' 2>/dev/null || {
  echo "ERROR: GNU grep with -P required (otherwise every edge is dropped)." >&2; exit 2; }
cd "$REPO_DIR"

declare -A C2P
while IFS=$'\t' read -r sha pr; do C2P[$sha]=$pr; done < "$WORK/commit2pr.tsv"
declare -A PRDATE
while IFS=$'\t' read -r pr d; do PRDATE[$pr]=$d; done < <(
  jq -r '.[]|"\(.number)\t\(.mergedAt)"' "$WORK/prs_merged.json")
# Ranges come from stage 2, which resolves them properly and is the single
# source of truth. Recomputing sha~N here is what produced an over-wide range
# for 22% of PRs — see 02-commit-map.sh's header for the measurement.
declare -A BASE
while IFS=$'\t' read -r pr b; do BASE[$pr]=$b; done < "$WORK/pr_base.tsv"

epoch() { date -u -d "$1" +%s 2>/dev/null || echo 0; }
out="$WORK/rework_edges.tsv"; : > "$out"
fails="$WORK/blame_failures.txt"; : > "$fails"

jq -r --arg s "$SINCE" --arg u "$UNTIL_TS" '[.[]|select(.mergedAt>$s and .mergedAt<$u and .mergeCommit!=null)]
  | sort_by(.mergedAt) | .[] | "\(.number)\t\(.mergeCommit.oid)\t\(.mergedAt)"' \
  "$WORK/prs_merged.json" |
while IFS=$'\t' read -r pr sha mdate; do
  base="${BASE[$pr]:-}"
  if [ -z "$base" ]; then
    printf '%s\t(no base from stage 2)\t-\n' "$pr" >> "$fails"; continue
  fi
  git rev-parse -q --verify "$base" >/dev/null 2>&1 || {
    printf '%s\t(base unresolvable)\t%s\n' "$pr" "$base" >> "$fails"; continue; }

  # Capture the diff and CHECK it. Piped straight into awk, a failed diff
  # (missing object -> exit 128) reads as empty input: the PR contributes zero
  # edges, lands in no failure file, and the "INCOMPLETE" warning stays quiet —
  # the silent-truncation class this script exists to ban.
  # -w matches blame's -w below: whitespace-only churn is not rework. Without
  # it a rustfmt-style sweep diffs whole blocks as deletions while -w blame
  # attributes those lines to their ORIGINAL authors — a fake rework spike
  # against old PRs for a semantic no-op.
  if ! git diff --unified=0 --no-color -w "$base" "$sha" -- 'crates/*.rs' 'web/src/*.ts' \
       > "$WORK/.diff.tmp" 2>/dev/null; then
    printf '%s\t(git diff failed)\t%s..%s\n' "$pr" "$base" "$sha" >> "$fails"; continue
  fi
  awk '
    /^--- a\// { f=substr($0,7); next }
    /^\+\+\+ /  { next }
    /^@@/ {
      match($0, /-[0-9]+(,[0-9]+)?/); spec=substr($0,RSTART+1,RLENGTH-1)
      split(spec,a,","); st=a[1]; cn=(length(a)>1?a[2]:1)
      if (cn>0 && f!="") print f "\t" st "\t" cn
    }' "$WORK/.diff.tmp" |
  while IFS=$'\t' read -r f st cn; do
    en=$((st+cn-1))
    blame=$(git blame -w --line-porcelain -L "${st},${en}" "$base" -- "$f" 2>/dev/null) || {
      printf '%s\t%s\t%s,%s\n' "$pr" "$f" "$st" "$en" >> "$fails"; continue; }
    printf '%s' "$blame" |
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
rm -f "$WORK/.diff.tmp"
nf=$(wc -l < "$fails")
echo "rework edges: $(wc -l < "$out")"
# Loud, not silent: a truncated dataset that reports itself is recoverable; one
# that doesn't is how the first version of this audit shipped wrong numbers.
if [ "$nf" -gt 0 ]; then
  echo "WARNING: $nf hunk(s) failed to blame — dataset is INCOMPLETE."
  echo "         see $fails ; do not quote these figures until it is empty."
fi
