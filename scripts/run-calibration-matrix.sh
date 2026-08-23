#!/usr/bin/env bash
# Run Factorio over every not-yet-measured fixture in a freshly exported bank.
#
# The exporter creates immutable bp.txt/manifest-real.json pairs.  This script
# resumes safely after interruption by leaving existing report.json files
# alone; regenerate into a new directory after changing the engine.
#
# One fixture's failure (harness timeout, Factorio crash, pre-flight error) is
# logged and the campaign moves on.  Letting `set -e` abort on the harness
# call would end the whole run at the first such fixture and, on resume, end
# it at the same fixture again — a campaign that can never complete.  The
# script exits non-zero at the end if anything failed; re-run it to retry
# (a failed fixture has no report.json, so it is not skipped).  The harness
# writes --out only after a completed run, so an absent report is the whole
# failure signature.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: scripts/run-calibration-matrix.sh <bank-dir>" >&2
  exit 2
fi

bank_dir=$1
if [ ! -f "$bank_dir/matrix.json" ]; then
  echo "missing $bank_dir/matrix.json; run calibration_matrix_export first" >&2
  exit 2
fi

failed=0
for fixture_dir in "$bank_dir"/*; do
  [ -d "$fixture_dir" ] || continue
  bp="$fixture_dir/bp.txt"
  manifest="$fixture_dir/manifest-real.json"
  report="$fixture_dir/report.json"
  if [ ! -f "$bp" ] || [ ! -f "$manifest" ]; then
    echo "skip $fixture_dir: missing bp.txt or manifest-real.json" >&2
    continue
  fi
  if [ -f "$report" ]; then
    echo "skip $(basename "$fixture_dir"): report.json already exists"
    continue
  fi
  echo "measure $(basename "$fixture_dir")"
  if ! cargo run --release -p spaghettio_sim_harness -- run \
      --bp "$bp" --manifest "$manifest" \
      --warmup 432000 --speed 32 --out "$report"; then
    echo "FAILED $(basename "$fixture_dir"): no report written; re-run this script to retry it" >&2
    failed=$((failed + 1))
  fi
done

if [ "$failed" -gt 0 ]; then
  echo "$failed fixture(s) failed; re-run to retry them" >&2
  exit 1
fi
