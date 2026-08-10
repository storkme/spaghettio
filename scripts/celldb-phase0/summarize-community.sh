#!/usr/bin/env bash
# Phase-0 probe 4, stage 2: summarize community.jsonl into
#   (a) the community DEMAND distribution (recipe motif mass across all
#       production blueprints — the independent check on the test-suite
#       census's concentration claim),
#   (b) DONOR candidates: single-recipe-dominant arrays (>=80% of machines
#       on one recipe), with density / area-per-machine / aspect baselines,
#       split into engine-legal vs exotic (rails, trains, combinators,
#       roboports, beacons, logistic chests — mechanics the layout engine
#       does not model; flags come straight from the analyzer).
#
# "Production blueprint" floor: >=4 crafting machines. Below that a record
# is a snippet or a mall cell, not an array baseline. Both thresholds are
# printed with the output so they cannot drift silently from prose.
set -euo pipefail
OUT="${OUT:-./celldb-phase0-work}"
J="$OUT/community.jsonl"
[ -s "$J" ] || { echo "ERROR: $J missing/empty — run mine-community.sh first." >&2; exit 2; }

MIN_MACHINES=4
DOM=0.8

echo "=== thresholds: production floor >=$MIN_MACHINES machines; donor dominance >=$DOM ==="
echo
echo "=== community demand: recipe motif mass across production blueprints ==="
jq -r --argjson mm "$MIN_MACHINES" '
  select(.machine_count >= $mm) | .recipe_groups[] |
  [.recipe, .machine_type, .count] | @tsv' "$J" |
awk -F'\t' '
  { mass[$1] += $3; total += $3 }
  END{
    printf "  %-34s %8s %7s\n","recipe","mass","share%"
    n=0
    for (r in mass) { order[++n]=r }
    for (i=1;i<n;i++) for (j=i+1;j<=n;j++)
      if (mass[order[j]]>mass[order[i]]) {t=order[i];order[i]=order[j];order[j]=t}
    cum=0
    for (i=1;i<=n && i<=20;i++){ r=order[i]; cum+=mass[r]
      printf "  %-34s %8d %6.1f%%   cum %.1f%%\n", r, mass[r], 100*mass[r]/total, 100*cum/total }
    printf "  (%d distinct recipes, total machine mass %d)\n", n, total
  }'

echo
echo "=== donor candidates: single-recipe arrays (dominant >= $DOM) ==="
jq -r --argjson mm "$MIN_MACHINES" --argjson dom "$DOM" '
  select(.machine_count >= $mm)
  # Degenerate geometry (zero-extent bbox) cannot yield density/area
  # baselines. (The analyzer serializes width/height/area unconditionally —
  # a null-check here would be dead code, a round-2 review finding.)
  | select(.width > 0 and .height > 0)
  | (.recipe_groups | max_by(.count)) as $d
  | select($d.count != null and (($d.count / .machine_count) >= $dom))
  | [$d.recipe, $d.machine_type, $d.count, .density, .area,
     (if .width > .height then (.width/.height) else (.height/.width) end),
     (if (.features.rails + .features.train_stops + .features.combinators +
          .features.roboports + .features.beacons + .features.logistic_chests) == 0
      then "legal" else "exotic" end),
     .source]
  | @tsv' "$J" |
tee "$OUT/donors.tsv" |
awk -F'\t' '
  { key=$1" ["$2"]"; n[key]++
    if($7=="legal"){ legal[key]++; apml[key]=apml[key]" "($5/$3) }
    dens[key]=dens[key]" "$4; apm[key]=apm[key]" "($5/$3); asp[key]=asp[key]" "$6 }
  # NB awk single-space split strips leading whitespace, so the leading " "
  # in the accumulators does NOT produce a phantom empty element (round-2
  # review claimed it did; refuted empirically — split(" 5 10") is [5,10]).
  function med(s,  a,m){ m=split(s,a," ")
    for(i=1;i<m;i++)for(j=i+1;j<=m;j++)if(a[j]<a[i]){t=a[i];a[i]=a[j];a[j]=t}
    return (m%2) ? a[(m+1)/2] : (a[m/2]+a[m/2+1])/2 }
  END{
    for (k in n) {
      lm = (legal[k] > 0) ? sprintf("%10.1f", med(apml[k])) : sprintf("%10s", "-")
      printf "  %-44s %4d %6d %10.2f %10.1f %s %8.2f\n", k, n[k], legal[k]+0, med(dens[k]), med(apm[k]), lm, med(asp[k])
    }
  }' | sort -k3 -rn > "$OUT/.donor_rows"
# Header OUTSIDE the sort stream — piping it through sort lands it mid-table
# (the 04-analyze.sh lesson, re-learned here in round 2). The sort key is
# field 3: the key "recipe [machine]" spans fields 1-2.
printf "  %-44s %4s %6s %10s %10s %10s %8s\n" "dominant recipe [machine]" "n" "legal" "med dens" "med area/m" "legal a/m" "med asp"
cat "$OUT/.donor_rows"
rm -f "$OUT/.donor_rows"
echo "  (rows: donor blueprints per dominant recipe, ranked by donor count; full list in $OUT/donors.tsv)"
