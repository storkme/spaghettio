#!/usr/bin/env bash
# Stage 4: the numbers CLAUDE.md's change-size norm rests on.
#
# Prints BOTH averaging methods for the size buckets. They agree across the
# first three (which is why the norm's threshold is 400) and DISAGREE in
# direction on the top one — the top bucket has flipped in every version of
# this dataset and is right-censored besides, so neither figure may be quoted
# alone there. Printing one alone is how that went unnoticed the first time.
#
# `reworked_totals` is keyed by the REWORKED pr (column 2) and divided by that
# PR's own additions: the figure is "how much of what this PR wrote did not
# survive", NOT "how much rewriting this PR did". A PR that rewrites lots of old
# code therefore scores 0 in its own bucket, which is correct — see the README's
# "What the rate actually measures".
set -euo pipefail
WORK="${WORK:-./audit-work}"

# An empty edge file would make every figure below 0/0 — gawk prints nan/inf
# and exits 0, the quiet-garbage mode this pipeline bans. Refuse instead.
[ -s "$WORK/rework_edges.tsv" ] || {
  echo "ERROR: $WORK/rework_edges.tsv is missing or empty — run stage 3 first." >&2; exit 3; }

# Enforce the completeness contract AT THE QUOTING POINT. Stages 1 and 3 warn
# about their own failures in their own terminal output, but this is the stage
# whose output gets transcribed into docs — a partial dataset must not produce
# complete-looking numbers from a clean run here.
for f in fetch_failures.txt blame_failures.txt; do
  [ -s "$WORK/$f" ] && {
    echo "ERROR: $WORK/$f is non-empty — the dataset is INCOMPLETE (see the" >&2
    echo "       stage that wrote it). Refusing to print quotable numbers." >&2
    exit 3; }
done
if [ -s "$WORK/range_unverified.txt" ]; then
  echo "NOTE: $(wc -l < "$WORK/range_unverified.txt") flagged range(s) in range_unverified.txt —"
  echo "      not fatal, but spot-check before quoting figures involving those PRs."
fi

awk -F'\t' '{t[$2]+=$5} END{for(k in t) print k"\t"t[k]}' "$WORK/rework_edges.tsv" \
  > "$WORK/reworked_totals.tsv"

# Population reconciliation FIRST, computed ONCE. Every denominator dispute in
# this pipeline's review history came from a number hand-copied into prose and
# then not updated with its siblings. These three variables are the only place
# the populations are derived; the bucket table asserts against $pop below, so
# an edit that changes one filter and not the other fails the run instead of
# shipping a table that disagrees with its own caption.
tot=$(awk -F'\t' 'NR>1{n++}          END{print n+0}' "$WORK/review_rounds.tsv")
pop=$(awk -F'\t' 'NR>1 && $6>20{n++} END{print n+0}' "$WORK/review_rounds.tsv")
big=$(awk -F'\t' 'NR>1 && $6>=400{n++} END{print n+0}' "$WORK/review_rounds.tsv")
[ "$pop" -gt 0 ] || {
  echo "ERROR: bucket population is empty — review_rounds.tsv missing or unfiltered." >&2; exit 3; }
echo "=== population (cite these, do not hand-derive them)"
awk -v tot="$tot" -v pop="$pop" -v big="$big" 'BEGIN{
    printf "  review_rounds rows (unfiltered) : %d\n", tot
    printf "  bucket population (>20 adds)    : %d\n", pop
    printf "  of those, >=400 adds            : %d  (%.0f%% of the bucket population)\n", big, 100*big/pop
    printf "  NB %d/%d = %.0f%% is the UNFILTERED share — quoting it against the\n", big, tot, 100*big/tot
    printf "     bucket population is the mixed-denominator trap. Pick one base.\n"
  }'

echo
echo "=== rework age distribution (all edges)"
awk -F'\t' '{print $4}' "$WORK/rework_edges.tsv" | sort -n | awk '
  # Nearest-rank percentiles: index = ceil(N*p). The tempting int(N*p)
  # undershoots by one rank whenever N*p is fractional (N=7, p50 -> a[3]).
  function nr(p,  i){ i=int(NR*p); if (i < NR*p) i++; return a[i] }
  {a[NR]=$1} END{printf "  n=%d  p50=%s  p75=%s  p90=%s\n", NR, nr(.5), nr(.75), nr(.9)}'
# "under 4 days", because age is FLOORED to whole days: floor(age)<=3 covers
# [0,4) days. The old "within 3 days" label promised [0,3] while measuring
# [0,4) — same number, honest name. The comparator is deliberately unchanged
# so the figure stays comparable across every version of this dataset.
awk -F'\t' '{n++; if($4<=3)c++} END{printf "  under 4 days: %d/%d = %d%%\n", c, n, 100*c/n}' \
  "$WORK/rework_edges.tsv"

echo
echo "=== rework per 100 added lines, by PR size"
echo "    buckets cover PRs with >20 adds, merged from BUCKET_SINCE (see 01-fetch.sh)"
echo "    NOTE the numerator is Rust/TS rework only (crates/*.rs, web/src/*.ts),"
echo "    while the denominator is GitHub's TOTAL additions across all files."
echo "    Consistent across buckets, so the comparison holds — but it is not"
echo "    literally 'per 100 added lines'."
awk -F'\t' -v T="$WORK/reworked_totals.tsv" -v POP="$pop" '
  BEGIN{ while((getline l < T)>0){ split(l,p,"\t"); rw[p[1]]=p[2] } }
  FNR>1 && $6>20 {
    adds=$6+0; r=(rw[$1]+0); rate=100*r/adds
    if(adds<100)      b="1. <100"
    else if(adds<400) b="2. 100-400"
    else if(adds<1000)b="3. 400-1k"
    else              b="4. >1k"
    n[b]++; s[b]+=rate; tr[b]+=r; ta[b]+=adds
  }
  END{
    # Sort the bucket rows INSIDE awk. Piping the whole block through `sort`
    # sorted the header too — it begins with "b", so it landed BELOW the four
    # numbered rows it labels. That shipped once and was caught in review after
    # I had already quoted the output as verification.
    nb=0; sum=0
    for(b in n){ rows[++nb]=sprintf("  %-11s %4d %14.1f %10.1f", b, n[b], s[b]/n[b], 100*tr[b]/ta[b]); sum+=n[b] }
    for(i=1;i<nb;i++) for(j=i+1;j<=nb;j++) if(rows[j]<rows[i]){t=rows[i];rows[i]=rows[j];rows[j]=t}
    printf "  %-11s %4s %14s %10s\n","bucket","n","mean-of-rates","pooled"
    for(i=1;i<=nb;i++) print rows[i]
    # Assert, not narrate: the rows must sum to the population the caption
    # cites. Guards the drift where someone edits one filter and not the other.
    if (sum != POP) {
      printf "ERROR: bucket rows sum to %d but the stated population is %d\n", sum, POP > "/dev/stderr"
      exit 3
    }
    printf "  (rows above sum to %d, the bucket population)\n", sum
  }' "$WORK/review_rounds.tsv"

echo
echo "=== share of rework, by originating PR size"
echo "    Both denominators, because prose keeps mixing them. The numerator is"
echo "    identical (every >=400-add PR clears the >20 floor); only the base"
echo "    changes. NB either way the denominator counts only rework of PRs in"
echo "    review_rounds.tsv (merged from BUCKET_SINCE) — rework of code written"
echo "    before that window is in the edge total but in neither share."
awk -F'\t' -v T="$WORK/reworked_totals.tsv" '
  BEGIN{ while((getline l < T)>0){ split(l,p,"\t"); rw[p[1]]=p[2] } }
  FNR>1 { r=(rw[$1]+0); g+=r
          if($6+0>=400){lg+=r; ln++}
          if($6+0>20) gp+=r }
  END{ if (g==0 || gp==0) {
         print "ERROR: zero total rework in a denominator — refusing to print shares." > "/dev/stderr"
         exit 3
       }
       printf "  large (>=400 adds): PRs=%d  lines=%d\n", ln, lg
       printf "    of the bucket population(>20 adds) rework: %.1f%%\n", 100*lg/gp
       printf "    of all review_rounds rework (unfiltered) : %.1f%%\n", 100*lg/g }' \
  "$WORK/review_rounds.tsv"
