#!/usr/bin/env bash
# Run Factorio over every not-yet-measured fixture in a freshly exported bank.
#
# The exporter creates immutable bp.txt/manifest-real.json pairs.  This script
# resumes safely after interruption by leaving existing report.json files
# alone; regenerate into a new directory after changing the engine.
#
# What "resume" means here: one completed Factorio run per fixture.  A run
# that completed but did not converge, or that reported kit errors, still
# wrote a report.json — that IS its result (deterministic; re-running it
# reproduces it), and the sweep reports the row as excluded with the reason.
# To re-measure such a row deliberately, delete its report.json first.
#
# Two things are NOT results and are retried on the next invocation:
#   - a harness failure (timeout, crash, pre-flight error): no report is
#     written, the failure is logged, the loop continues, and the script
#     exits non-zero at the end.  Letting `set -e` abort on the harness call
#     would end the campaign at the first such fixture and, on resume, end
#     it at the same fixture again.
#   - a report.json that does not parse (a kill or a full disk mid-write):
#     treated as absent and re-measured, since the harness writes it with
#     one non-atomic write.
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
if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required (used to recognise a partially written report.json)" >&2
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
    if jq -e . "$report" >/dev/null 2>&1; then
      echo "skip $(basename "$fixture_dir"): report.json already exists"
      continue
    fi
    echo "re-measure $(basename "$fixture_dir"): report.json exists but does not parse (partial write?)" >&2
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
