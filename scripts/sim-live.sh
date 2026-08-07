#!/usr/bin/env bash
# Run a sim fixture with live stats streaming to Grafana, and print a
# dashboard link pre-filtered to that fixture with a sensible live window.
#
#   scripts/sim-live.sh <label> <bp> <manifest> [-- extra run args...]
#
# Example:
#   scripts/sim-live.sh ac5-baseline /tmp/x/bp.txt /tmp/x/manifest-real.json \
#       -- --warmup 432000 --speed 32
#
# Why a wrapper: the harness creates its scratch dir with a random suffix, so
# the live CSV path is only knowable after launch. This finds it, starts the
# follower against it, and hands you the link.
set -euo pipefail

GRAFANA_BASE="${GRAFANA_BASE:-https://cyanteal2982.grafana.net}"
ARM="${ARM:-live}"

label="${1:?usage: sim-live.sh <label> <bp> <manifest> [-- run args]}"
bp="${2:?missing bp.txt}"
manifest="${3:?missing manifest-real.json}"
shift 3
[ "${1:-}" = "--" ] && shift

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${SIM_OUT:-/tmp/spaghettio-sim-live}"
mkdir -p "$out"

echo "starting run (label=$label) ..."
cargo run --release -p spaghettio_sim_harness -- run \
    --bp "$bp" --manifest "$manifest" --timeseries \
    --out "$out/$label-report.json" "$@" &
run_pid=$!

# The scratch dir appears within a few seconds of launch.
csv=""
for _ in $(seq 1 60); do
    d=$(ls -dt /tmp/spaghettio-sim-runs/*"$label"* 2>/dev/null | head -1 || true)
    if [ -n "$d" ] && [ -d "$d/script-output" ]; then
        csv="$d/script-output/timeseries.csv"
        break
    fi
    sleep 2
done
[ -n "$csv" ] || { echo "could not locate the run's script-output dir" >&2; wait $run_pid; exit 1; }

python3 "$repo/scripts/sim-to-graphite.py" "$csv" --follow --label "$label" --arm "$ARM" &
follow_pid=$!
trap 'kill $follow_pid 2>/dev/null || true' EXIT

cat <<EOF

────────────────────────────────────────────────────────────────────────
  LIVE: $GRAFANA_BASE/d/spaghettio-sim/?var-fixture=$label&var-arm=$ARM&from=now-15m&to=now&refresh=10s
────────────────────────────────────────────────────────────────────────
  Windows land every ~10s of wall clock at speed 32. The measured window
  starts only after warmup, so expect a flat lead-in — that is the ramp,
  and a stage still curving once measurement starts is reporting buffer
  fill rather than throughput (sim-harness-forensics.md).
────────────────────────────────────────────────────────────────────────

EOF

wait $run_pid
run_rc=$?
sleep 3           # let the follower drain the last window
kill $follow_pid 2>/dev/null || true

# The report carries the full per-item sample series; push it so the run's
# history survives at full fidelity, not just the live windows.
if [ -f "$out/$label-report.json" ]; then
    python3 "$repo/scripts/sim-to-graphite.py" "$out/$label-report.json" \
        --label "$label" --arm "$ARM" --anchor now || true
fi
exit $run_rc
