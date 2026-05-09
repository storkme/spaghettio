#!/usr/bin/env bash
# Sweep CP-SAT random seeds for (4, 9) Clos compose to find the fast regime.
#
# Background: seed=42 (current DEFAULT_SEED placeholder in place.py) takes
# ~600s end-to-end on (4, 9). The pruner once observed jh=9 OPTIMAL in ~10s,
# suggesting some seeds are dramatically faster. With the seed pin in place
# (commit f6a9aa8), runs are now reproducible per seed, so we can sweep and
# pick a winner.
#
# Usage:
#   scripts/sweep_balancer_seeds.sh                  # sweeps 0..9
#   scripts/sweep_balancer_seeds.sh 1 7 13 42 99     # explicit list
#
# Each run is capped at 20 min by `timeout`. Plan ~2-3 hours wall for the
# default 10-seed sweep — best run overnight.
#
# Output: TSV log at /tmp/seed_sweep_<timestamp>.log with columns
#   seed  total_s  jh9_solver_s  status
# Plus a summary sorted by total wall on stdout when done. Update
# DEFAULT_SEED in crates/balancer-gen/scripts/place.py to the winner.

set -uo pipefail

SEEDS=("$@")
if [[ ${#SEEDS[@]} -eq 0 ]]; then
    SEEDS=(0 1 2 3 4 5 6 7 8 9)
fi

LOG="/tmp/seed_sweep_$(date +%Y%m%d_%H%M%S).log"
echo "Sweeping seeds: ${SEEDS[*]}"
echo "Log: $LOG"
echo

cargo build --release -p balancer-gen 2>&1 | tail -3

printf "seed\ttotal_s\tjh9_solver_s\tstatus\n" | tee "$LOG"

for seed in "${SEEDS[@]}"; do
    out=$(SPAGHETTIO_CP_SAT_SEED="$seed" SPAGHETTIO_DEBUG_4_9=1 \
        timeout 1200 ./target/release/balancer-gen 2>&1) || true

    total=$(echo "$out" | grep -oP 'compose\+route in \K[0-9.]+' | head -1)
    jh9=$(echo "$out" | grep -oP '\[(SHORT|LONG)\] jh=9 status=OPTIMAL solver_elapsed=\K[0-9.]+' | head -1)

    if echo "$out" | grep -q "verified MX3"; then
        status="OK"
    elif echo "$out" | grep -q "no feasible junction_height"; then
        status="INFEASIBLE"
    else
        status="TIMEOUT_OR_ERROR"
    fi

    printf "%s\t%s\t%s\t%s\n" "$seed" "${total:-NA}" "${jh9:-NA}" "$status" | tee -a "$LOG"
done

echo
echo "=== summary (sorted by total wall, ascending) ==="
{
    head -1 "$LOG"
    tail -n +2 "$LOG" | sort -t$'\t' -k2 -g
}
