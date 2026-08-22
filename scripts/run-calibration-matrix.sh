#!/usr/bin/env bash
# Run Factorio over every not-yet-measured fixture in a freshly exported bank.
#
# The exporter creates immutable bp.txt/manifest-real.json pairs.  This script
# resumes safely after interruption by leaving existing report.json files
# alone; regenerate into a new directory after changing the engine.
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
  cargo run --release -p spaghettio_sim_harness -- run \
    --bp "$bp" --manifest "$manifest" \
    --warmup 432000 --speed 32 --out "$report"
done
