#!/usr/bin/env bash
# Run a sim fixture AND capture its per-machine forensic dump.
#
# WHY THIS EXISTS: the scenario writes `sim-state.json` (every machine's
# position, name, status, fluid + solid inventory contents; plus belts,
# inserters, UG pairings, splitters, pipes, chests) into the run's scratch
# write dir — but `orchestrate` DELETES that dir on success. So the richest
# forensic artifact in the harness is invisible for exactly the runs you most
# want it for: the ones that completed and came back WARN/FAIL.
#
# The aggregate `machine_census` in the report tells you "1 full_output,
# 1 item_ingredient_shortage". This tells you WHICH machine and WHAT it is
# missing — which is what turned #435 from a two-week mystery into a named
# defect (copper-cable depleting 42->34->20->6->2 along the EC row, tail
# machine starved while a producer sat output-blocked).
#
# Usage:
#   scripts/sim-capture-state.sh <fixture-stem> [extra spaghettio-sim args...]
#
#   fixture-stem is the basename under crates/core/target/tmp, e.g.
#     scripts/sim-capture-state.sh chain-ec15-d7
#     scripts/sim-capture-state.sh mega-chain-pu4raw --ticks 20000
#
# Writes <stem>.sim-state.json and <stem>.report.json next to the fixture.
set -uo pipefail

STEM="${1:?usage: sim-capture-state.sh <fixture-stem> [extra args...]}"
shift || true

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$ROOT/crates/core/target/tmp"
BP="$TMP/$STEM.bp"
MANIFEST="$TMP/$STEM.manifest.json"
OUT_STATE="$TMP/$STEM.sim-state.json"
OUT_REPORT="$TMP/$STEM.report.json"
RUNS="${TMPDIR:-/tmp}/spaghettio-sim-runs"

[ -f "$BP" ] || { echo "no such fixture: $BP" >&2; echo "generate it with the --ignored export_* artifact producers in tests/cell_composition.rs" >&2; exit 1; }
[ -f "$MANIFEST" ] || { echo "no manifest: $MANIFEST" >&2; exit 1; }

DONE="$(mktemp)"; rm -f "$DONE" "$OUT_STATE"

(
  cargo run -q -p spaghettio_sim_harness --bin spaghettio-sim -- \
    run --bp "$BP" --manifest "$MANIFEST" --out "$OUT_REPORT" "$@"
  echo done > "$DONE"
) &
SIM_PID=$!

# Race the poller against the scratch-dir cleanup. The dump is written just
# before the server is killed, so there is a window of ~seconds; poll fast.
# Match on the fixture stem so concurrent runs don't steal each other's dumps
# (per-run scratch dirs are named <scenario>-<epoch>-<pid>).
for _ in $(seq 1 6000); do
  f=$(find "$RUNS" -name sim-state.json -path "*$STEM*" 2>/dev/null | head -1)
  if [ -n "$f" ] && cp "$f" "$OUT_STATE" 2>/dev/null; then
    echo "captured: $OUT_STATE"
    break
  fi
  [ -f "$DONE" ] && break
  sleep 0.2
done

wait "$SIM_PID"
RC=$?

if [ -f "$OUT_STATE" ]; then
  echo "--- machine census from the dump ---"
  python3 - "$OUT_STATE" <<'PY'
import json, sys, collections
d = json.load(open(sys.argv[1]))
by = collections.Counter(m[3] for m in d.get("machines", []))
for status, n in by.most_common():
    print(f"  {status}: {n}")
print(f"  (machines: {len(d.get('machines', []))})")
PY
else
  echo "MISSED the dump window (run finished/cleaned before the poller saw it)" >&2
fi

exit "$RC"
