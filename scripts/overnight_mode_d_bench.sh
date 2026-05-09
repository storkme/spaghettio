#!/usr/bin/env bash
# Conservative overnight Mode D bench.
#
# For each (n, m) library shape, attempts to re-place its topology
# through Mode D and records the result. Designed to run unattended
# overnight on a memory-constrained box (~20 GB RAM in WSL) without
# risking OOM that would take WSL down.
#
# Three layers of protection:
#
#   1. **Per-process address-space cap** via `prlimit --as=<bytes>`. If a
#      single solve exceeds the cap, the kernel kills *only that process*
#      with a clean error — host stays up.
#   2. **Pre-flight memory check** before each shape. If `MemAvailable`
#      drops below the floor, abort the whole run cleanly. Better to
#      stop early than crash mid-shape.
#   3. **Sequential** — never run two CP-SAT processes in parallel.
#      Even at low memory each can blow up; no concurrency = predictable
#      footprint.
#
# Plus `nice -n 19` and `ionice -c idle` so the host stays interactive
# (keyboard/mouse responsiveness, browser tabs, etc.) even if a solve
# is going hard. These don't help with memory but they help with the
# system feeling alive.
#
# Resumable: if killed mid-run, re-running skips shapes already in the
# log file. Worst case the in-flight shape gets retried.
#
# Usage:
#   scripts/overnight_mode_d_bench.sh                        # all library shapes
#   scripts/overnight_mode_d_bench.sh '(5,1);(7,1);(11,1)'   # specific shapes
#
# Output: TSV log at /tmp/mode_d_overnight_<timestamp>.tsv with columns
#   shape  status  splitters  belts  ugs  solve_s  wall_s  notes
#
# Tunables (override via env):
#   PER_PROCESS_MEM_GB   per-process address-space cap (default 4)
#   MIN_FREE_GB          abort floor — if MemAvailable drops below, stop (default 4)
#   PER_SHAPE_TIMEOUT_S  per-shape wall-time limit (default 300)
#   LOG                  output TSV path (default /tmp/mode_d_overnight_<ts>.tsv)

set -uo pipefail

# Tunables (overridable via env)
MIN_FREE_GB=${MIN_FREE_GB:-6}              # abort floor — 6GB host headroom by default
PER_PROC_CAP_GB=${PER_PROC_CAP_GB:-14}     # prlimit --as for the one running process
PER_SHAPE_TIMEOUT_S=${PER_SHAPE_TIMEOUT_S:-300}
RETRY_SLACK=${RETRY_SLACK:-2}              # bbox slack to retry with on INFEASIBLE (0 disables)
LOG=${LOG:-/tmp/mode_d_overnight_$(date +%Y%m%d_%H%M%S).tsv}

MIN_FREE_KB=$((MIN_FREE_GB * 1024 * 1024))

# We run sequentially, one process at a time. With ~20GB available on the
# host, a flat ~14GB cap on virtual address space gives Python+ortools+CP-SAT
# room to start (the prior 2GB tier killed Python at startup) while still
# leaving enough host headroom that a runaway process can't take WSL down.
# Cap is on virtual size (--as), not RSS — actual memory used is typically
# a fraction of this.

BIN="./target/release/balancer-gen"
if [[ ! -x "$BIN" ]]; then
    echo "ERROR: $BIN not found. Build first:" >&2
    echo "  cargo build --release -p balancer-gen" >&2
    exit 1
fi

# Determine shape list. Either from arg ((n,m);(n,m);...) or "all" via
# the binary's curated query.
shapes_arg=${1:-"all"}

if [[ "$shapes_arg" == "all" ]]; then
    # Pull library shapes via a one-shot Mode D query that returns
    # immediately; we just need the side-effect listing. Easier: do
    # the loop in shell over the whole grid and let the binary skip
    # missing entries.
    shape_list=()
    for n in {1..10}; do
        for m in {1..10}; do
            shape_list+=("($n,$m)")
        done
    done
else
    IFS=';' read -ra shape_list <<< "$shapes_arg"
fi

# Resumability: if LOG exists, parse it for shapes already done.
declare -A done_shapes=()
if [[ -f "$LOG" ]]; then
    while IFS=$'\t' read -r shape _; do
        [[ "$shape" == "shape" ]] && continue
        done_shapes["$shape"]=1
    done < "$LOG"
    echo "Resuming: ${#done_shapes[@]} shape(s) already in $LOG"
else
    printf 'shape\tstatus\tsplitters\tbelts\tugs\tsolve_s\twall_s\tnotes\n' > "$LOG"
fi

# Run Mode D for one shape at one slack level, with cap and timeout.
# Echoes the raw spike output to stdout. Caller parses.
run_mode_d_attempt() {
    local shape=$1 slack=$2 cap_gb=$3 timeout_s=$4
    local cap_bytes=$((cap_gb * 1024 * 1024 * 1024))
    SPAGHETTIO_MODE_D_ONLY="$shape" \
    SPAGHETTIO_MODE_D_BBOX_SLACK="$slack" \
    nice -n 19 ionice -c idle \
    prlimit --as="$cap_bytes" \
    timeout "$timeout_s" \
    "$BIN" 2>&1
}

echo "Mode D overnight bench"
echo "  log:                $LOG"
echo "  shape count:        ${#shape_list[@]}"
echo "  per-process cap:    ${PER_PROC_CAP_GB}GB virtual (sequential, one at a time)"
echo "  min-free floor:     ${MIN_FREE_GB}GB"
echo "  per-shape timeout:  ${PER_SHAPE_TIMEOUT_S}s (×$((1 + RETRY_SLACK > 0 ? 1 : 0)) attempts on INFEASIBLE)"
echo "  retry slack:        ${RETRY_SLACK} cells (0 disables retry)"
echo

start_time=$(date +%s)

for shape in "${shape_list[@]}"; do
    # Skip already-done.
    if [[ -n "${done_shapes[$shape]:-}" ]]; then
        continue
    fi

    # Pre-flight: abort if we're below the memory floor.
    avail_kb=$(awk '/MemAvailable/ { print $2 }' /proc/meminfo)
    if [[ $avail_kb -lt $MIN_FREE_KB ]]; then
        avail_gb=$((avail_kb / 1024 / 1024))
        echo "ABORT: only ${avail_gb}GB available, need ${MIN_FREE_GB}GB. Stopping cleanly." >&2
        printf '%s\tABORT\t\t\t\t\t\tlow_memory(%dGB)\n' \
            "$shape" "$avail_gb" >> "$LOG"
        break
    fi

    cap_gb=$PER_PROC_CAP_GB
    echo "=== $shape (cap=${cap_gb}GB, avail: $((avail_kb/1024/1024))GB) ==="
    t0=$(date +%s)

    # First attempt at slack=0 (library bbox).
    out=$(run_mode_d_attempt "$shape" 0 "$cap_gb" "$PER_SHAPE_TIMEOUT_S") || true
    used_slack=0

    # Retry with bbox slack if first attempt was INFEASIBLE.
    if (( RETRY_SLACK > 0 )) && echo "$out" | grep -q 'INFEASIBLE'; then
        echo "  → INFEASIBLE at library bbox, retrying with +${RETRY_SLACK} slack..."
        # Re-check memory before retry (slack solves use more).
        avail_kb_retry=$(awk '/MemAvailable/ { print $2 }' /proc/meminfo)
        if [[ $avail_kb_retry -lt $MIN_FREE_KB ]]; then
            echo "  skip retry: low memory ($((avail_kb_retry/1024/1024))GB)" >&2
        else
            out=$(run_mode_d_attempt "$shape" "$RETRY_SLACK" "$cap_gb" "$PER_SHAPE_TIMEOUT_S") || true
            used_slack=$RETRY_SLACK
        fi
    fi

    wall=$(($(date +%s) - t0))

    # Parse result. The spike prints lines like:
    #   --- (2, 2) ---  bbox: 2×3  splitters: 1  edges: 4
    #     synth-place: status=OPTIMAL elapsed=0.007s splitters=1 belts=4 ugs=0
    #     ✓ (2, 2): classified Balanced ...
    # On failure (timeout, OOM, infeasible), various error patterns.
    status=$(echo "$out" | grep -oP 'status=\K\w+' | head -1)
    splitters=$(echo "$out" | grep -oP 'synth-place: status=\w+ elapsed=[0-9.]+s splitters=\K\d+' | head -1)
    belts=$(echo "$out" | grep -oP 'belts=\K\d+' | head -1)
    ugs=$(echo "$out" | grep -oP 'ugs=\K\d+' | head -1)
    solve_s=$(echo "$out" | grep -oP 'elapsed=\K[0-9.]+(?=s)' | head -1)

    notes=""
    if [[ -z "$status" ]]; then
        # No "status=..." line — fall back to error-pattern matching.
        if echo "$out" | grep -q 'INFEASIBLE'; then
            status="INFEASIBLE"
            # Capture extra context — usually the bbox where the solver
            # gave up. Keep short for TSV cleanliness.
            ctx=$(echo "$out" | grep -oP 'bbox: \K[0-9]+×[0-9]+' | head -1)
            notes="bbox=${ctx:-?}_at_library_dims"
        elif echo "$out" | grep -qE 'process killed by signal|Cannot allocate|MemoryError|OOM'; then
            status="KILLED"
            notes="oom_or_signal"
        elif [[ $wall -ge $PER_SHAPE_TIMEOUT_S ]]; then
            status="TIMEOUT"
            notes="${PER_SHAPE_TIMEOUT_S}s_wall"
        else
            status="ERROR"
            # Capture last error line for diagnosis. Strip newlines, keep
            # short. The leading `✗` marks spike-side failure summaries.
            err=$(echo "$out" | grep -oP '✗ \([^)]+\): \K[^\n]{1,120}' | head -1 \
                | tr '\t\n' '  ' | sed 's/[[:space:]]\+/ /g')
            notes="${err:-unknown}"
        fi
    fi

    # Tag notes with slack level when retry was used.
    if (( used_slack > 0 )); then
        if [[ -n "$notes" ]]; then
            notes="$notes; slack=+${used_slack}"
        else
            notes="slack=+${used_slack}"
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$shape" "$status" "${splitters:-}" "${belts:-}" "${ugs:-}" \
        "${solve_s:-}" "$wall" "$notes" >> "$LOG"

    slack_msg=""
    if (( used_slack > 0 )); then
        slack_msg=" (slack=+$used_slack)"
    fi
    echo "  → $status  splitters=${splitters:-?}  belts=${belts:-?}  ugs=${ugs:-?}  ${wall}s${slack_msg}"
done

elapsed=$(($(date +%s) - start_time))
echo
echo "=== done in ${elapsed}s ==="
echo "Summary by status:"
tail -n +2 "$LOG" | awk -F'\t' '{ print $2 }' | sort | uniq -c | sort -rn
echo
echo "Full log: $LOG"
