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
END=$(date -u -d "$UNTIL_TS" +%s)
jq -r '.[]|"\(.number)\t\(.mergedAt)"' "$WORK/prs_merged.json" | sort > "$WORK/.pr_dates"
awk -F'\t' 'NR>1 && $6>20 {print $1"\t"$6}' "$WORK/review_rounds.tsv" | sort > "$WORK/.pr_adds"
join -t $'\t' "$WORK/.pr_adds" "$WORK/.pr_dates" | while IFS=$'\t' read -r pr adds md; do
  e=$(date -u -d "$md" +%s) || continue
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
      rows[++nb]=sprintf("  %-11s %4d %18.1f %8d", b, cnt, s[b]/cnt, d[b,int((cnt+1)/2)])
    }
    for(i=1;i<nb;i++) for(j=i+1;j<=nb;j++) if(rows[j]<rows[i]){t=rows[i];rows[i]=rows[j];rows[j]=t}
    printf "  %-11s %4s %18s %8s\n","bucket","n","mean-exposure-days","median"
    for(i=1;i<=nb;i++) print rows[i]
  }'
rm -f "$WORK/.pr_dates" "$WORK/.pr_adds"
