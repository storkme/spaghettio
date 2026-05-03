#!/usr/bin/env bash
# Top-level orchestrator for the overnight CP-SAT bake job.
#
# Phase 1 — sweep many seeds for each target shape via
#   `bake_cp_sat_runner.py`. First successful seed per shape wins.
# Phase 2 — generate `docs/bake-overnight-results.md` from the journal +
#   sweep TSV via `bake_cp_sat_report.py`.
#
# Resume-safe: re-running with `--resume` skips shapes already solved
# in `scripts/cp_sat_journal.jsonl`.
#
# Usage:
#   scripts/bake_overnight.sh                # default: 1,9 1,10 × seeds 0-127 × 300s
#   scripts/bake_overnight.sh 1,9 1,10 1,11  # custom shape list
#   SEEDS=0-255 TIMEOUT=600 scripts/bake_overnight.sh
#   scripts/bake_overnight.sh --resume       # skip already-solved
#
# Defaults assume an 8-hour budget on 16 cores:
#   - 16 workers × 300s probe budget = ~5 min per seed
#   - 256 seeds × 2 shapes = 512 probes
#   - 512 / 16 cores × 300s = ~2.7 hours wall (sequential per shape)
#   - Plus startup/teardown overhead → fits comfortably overnight

set -uo pipefail

cd "$(dirname "$0")/.."

# Args / env.
SHAPES=()
RESUME=""
for arg in "$@"; do
    case "$arg" in
        --resume) RESUME="--resume" ;;
        *,*) SHAPES+=("$arg") ;;
        *) echo "unknown arg: $arg" >&2; exit 2 ;;
    esac
done
[[ ${#SHAPES[@]} -eq 0 ]] && SHAPES=("1,9" "1,10")

SEEDS="${SEEDS:-0-127}"
TIMEOUT="${TIMEOUT:-300}"
WORKERS="${WORKERS:-16}"

ts=$(date +%Y%m%d_%H%M%S)
log="/tmp/bake_overnight_${ts}.log"

echo "=== overnight bake — $(date -Is) ===" | tee "$log"
echo "shapes:  ${SHAPES[*]}" | tee -a "$log"
echo "seeds:   $SEEDS" | tee -a "$log"
echo "timeout: ${TIMEOUT}s/probe" | tee -a "$log"
echo "workers: $WORKERS" | tee -a "$log"
echo "log:     $log" | tee -a "$log"
echo | tee -a "$log"

# Build the dump_synth helper once (the runner shells out to it per shape).
echo "[bake] building dump_synth..." | tee -a "$log"
cargo build --manifest-path crates/core/Cargo.toml --example dump_synth --quiet 2>&1 | tee -a "$log"

# Phase 1 — sweep.
echo | tee -a "$log"
echo "[bake] phase 1 — sweep" | tee -a "$log"
scripts/bake_cp_sat_runner.py \
    --shapes "${SHAPES[@]}" \
    --seeds "$SEEDS" \
    --timeout "$TIMEOUT" \
    --workers "$WORKERS" \
    $RESUME 2>&1 | tee -a "$log"

phase1_rc=$?

# Phase 2 — report.
echo | tee -a "$log"
echo "[bake] phase 2 — report" | tee -a "$log"
scripts/bake_cp_sat_report.py 2>&1 | tee -a "$log"
report_rc=$?

echo | tee -a "$log"
echo "=== overnight bake done — phase1=$phase1_rc report=$report_rc ===" | tee -a "$log"
echo "log: $log"
echo "report: docs/bake-overnight-results.md"
