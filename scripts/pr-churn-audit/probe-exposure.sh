#!/usr/bin/env bash
# Not a pipeline stage — answers one confound question (round-9 review): does
# merge timing correlate with PR size? Edges only accrue from later in-window
# reworkers, so a bucket whose PRs cluster late is deflated. If exposure time
# (days between merge and window end) is flat across buckets, that censoring
# cannot manufacture the size climb. The audit doc quotes this probe's output.
#
# Run AFTER stage 1:  probe-exposure.sh "$WORK"
set -euo pipefail
WORK="${1:-${WORK:-./audit-work}}"
. "$(dirname "$0")/window.env"
# Same GNU guard as stage 3: this probe can run right after stage 1, which
# performs no such check, and a BSD `date -d` would either abort or silently
# compute a wrong END — feeding a wrong exposure table into the doc.
date -u -d "2026-01-01" +%s >/dev/null 2>&1 || {
  echo "ERROR: GNU date required (BSD date breaks -d parsing)." >&2; exit 2; }
END=$(date -u -d "$UNTIL_TS" +%s)
dropped=0
jq -r '.[]|"\(.number)\t\(.mergedAt)"' "$WORK/prs_merged.json" | sort > "$WORK/.pr_dates"
awk -F'\t' 'NR>1 && $6>20 {print $1"\t"$6}' "$WORK/review_rounds.tsv" | sort > "$WORK/.pr_adds"
join -t $'\t' "$WORK/.pr_adds" "$WORK/.pr_dates" | while IFS=$'\t' read -r pr adds md; do
  if ! e=$(date -u -d "$md" +%s 2>/dev/null); then
    echo "DROPPED: PR $pr has unparseable mergedAt '$md'" >&2; continue
  fi
  printf '%s\t%s\n' "$adds" "$(( (END - e) / 86400 ))"
done | awk -F'\t' '
  {
    if($1<100)      b="1. <100"
    else if($1<400) b="2. 100-400"
    else if($1<1000)b="3. 400-1k"
    else            b="4. >1k"
    n[b]++; s[b]+=$2; d[b,n[b]]=$2
  }
  END{
    # sort rows inside awk — see 04-analyze.sh for why the header must not be
    # piped through sort with the data
    nb=0
    for(b in n){
      cnt=n[b]
      for(i=1;i<=cnt;i++) for(j=i+1;j<=cnt;j++)
        if(d[b,j]<d[b,i]){t=d[b,i];d[b,i]=d[b,j];d[b,j]=t}
      # Textbook median: mean of the two middles for even n. The lower-middle
      # shortcut was off by up to a day on three of these four buckets, and
      # the doc quotes these values.
      if (cnt % 2) med = d[b,(cnt+1)/2]
      else         med = (d[b,cnt/2] + d[b,cnt/2+1]) / 2
      rows[++nb]=sprintf("  %-11s %4d %18.1f %8.1f", b, cnt, s[b]/cnt, med)
    }
    for(i=1;i<nb;i++) for(j=i+1;j<=nb;j++) if(rows[j]<rows[i]){t=rows[i];rows[i]=rows[j];rows[j]=t}
    printf "  %-11s %4s %18s %8s\n","bucket","n","mean-exposure-days","median"
    for(i=1;i<=nb;i++) print rows[i]
  }'
rm -f "$WORK/.pr_dates" "$WORK/.pr_adds"
