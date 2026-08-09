#!/usr/bin/env bash
# Stage 4: the numbers CLAUDE.md's change-size norm rests on.
#
# Prints BOTH averaging methods for the size buckets. They agree across the
# first three buckets (which is why the norm's threshold is 400) and disagree
# in direction on the top one — mean rises 8.5 -> 10.5, pooled FALLS 8.3 -> 6.9.
# Do not quote one without the other.
set -euo pipefail
WORK="${WORK:-./audit-work}"

awk -F'\t' '{t[$2]+=$5} END{for(k in t) print k"\t"t[k]}' "$WORK/rework_edges.tsv" \
  > "$WORK/reworked_totals.tsv"

echo "=== rework age distribution (all edges)"
awk -F'\t' '{print $4}' "$WORK/rework_edges.tsv" | sort -n | awk '
  {a[NR]=$1} END{printf "  n=%d  p50=%s  p75=%s  p90=%s\n", NR, a[int(NR*.5)], a[int(NR*.75)], a[int(NR*.9)]}'
awk -F'\t' '{n++; if($4<=3)c++} END{printf "  within 3 days: %d/%d = %d%%\n", c, n, 100*c/n}' \
  "$WORK/rework_edges.tsv"

echo
echo "=== rework per 100 added lines, by PR size"
echo "    buckets cover PRs with >20 adds, merged from BUCKET_SINCE (see 01-fetch.sh)"
echo "    NOTE the numerator is Rust/TS rework only (crates/*.rs, web/src/*.ts),"
echo "    while the denominator is GitHub's TOTAL additions across all files."
echo "    Consistent across buckets, so the comparison holds — but it is not"
echo "    literally 'per 100 added lines'."
awk -F'\t' -v T="$WORK/reworked_totals.tsv" '
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
    nb=0
    for(b in n) rows[++nb]=sprintf("  %-11s %4d %14.1f %10.1f", b, n[b], s[b]/n[b], 100*tr[b]/ta[b])
    for(i=1;i<nb;i++) for(j=i+1;j<=nb;j++) if(rows[j]<rows[i]){t=rows[i];rows[i]=rows[j];rows[j]=t}
    printf "  %-11s %4s %14s %10s\n","bucket","n","mean-of-rates","pooled"
    for(i=1;i<=nb;i++) print rows[i]
  }' "$WORK/review_rounds.tsv"

echo
echo "=== share of all rework, by originating PR size"
awk -F'\t' -v T="$WORK/reworked_totals.tsv" '
  BEGIN{ while((getline l < T)>0){ split(l,p,"\t"); rw[p[1]]=p[2] } }
  FNR>1 { r=(rw[$1]+0); if($6+0<400){sm+=r; sn++} else {lg+=r; ln++} }
  END{ g=sm+lg
       printf "  large (>=400 adds): PRs=%d  lines=%d  %.1f%%\n", ln, lg, 100*lg/g
       printf "  small (<400 adds) : PRs=%d  lines=%d  %.1f%%\n", sn, sm, 100*sm/g }' \
  "$WORK/review_rounds.tsv"
