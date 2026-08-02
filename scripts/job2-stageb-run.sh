#!/usr/bin/env bash
#
# RFC-064 Phase 2 Stage B sim-campaign runner (spare-compute playbook, Job 2).
#
# Consumes the index.json produced by the export driver
# (crates/core/examples/rfc064_phase2_stageb_export.rs) and runs the real
# headless-Factorio measurement for every (fixture, variant) pair, at most
# $MAX_CONCURRENT in flight, with the playbook's governances baked in:
#
#   - NO blessing (no `bless`), NO golden, NO commits — this script only
#     measures and records.
#   - One row per instance in $DATE/results.tsv (never aggregate counts).
#   - Fixed retry rule: each run gets exactly one retry; a second failure is
#     recorded as `failed` (stderr tail left in the run's run.log) and the
#     loop continues. Never debug, never tune.
#   - Idempotent/resumable: a (fixture, variant) whose report.json already
#     exists is skipped and recorded as `already-done`, so an interrupted
#     campaign picks up where it left off.
#   - Provenance stamped into $DATE/provenance.txt (git HEAD, rustc, Factorio
#     version, the exact command). Sim results are tech-state-sensitive; an
#     unstamped number is undiagnosable.
#
# Dependencies/order:
#   1. Build the harness once up front (avoids concurrent `cargo run` lock
#      contention and lets n runs share one binary):
#        cargo build --release -p spaghettio_sim_harness
#   2. Sanity-check the pinned Factorio install exists (like the harness
#      `fetch` step). Runs only READ the install; this script never fetches
#      while runs are live.
#
# Usage:
#   scripts/job2-stageb-run.sh [--root DIR] [--date DATE] [--concurrency N]
#
#   --root DIR       parent corpora dir (default
#                    $HOME/spaghettio-corpora/job2-sim-baselines)
#   --date DATE      dated subdir holding index.json (default today YYYY-MM-DD)
#   --concurrency N  max simultaneous Factorio servers (default 3)

set -uo pipefail

# ---- resolve repo + env -------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
DATE="$(date +%F)"
ROOT="$HOME/spaghettio-corpora/job2-sim-baselines"
MAX_CONCURRENT=3

while [[ $# -gt 0 ]]; do
    case "$1" in
        --root) ROOT="$2"; shift 2;;
        --date) DATE="$2"; shift 2;;
        --concurrency) MAX_CONCURRENT="$2"; shift 2;;
        *) echo "unknown arg: $1" >&2; exit 2;;
    esac
done

INDEX="$ROOT/$DATE/index.json"
RUNS_DIR="$ROOT/$DATE/sim"
RESULTS="$ROOT/$DATE/results.tsv"

if [[ ! -f "$INDEX" ]]; then
    echo "error: index not found: $INDEX" >&2
    echo "run the export driver first:" >&2
    echo "  cargo run --release --manifest-path $REPO/crates/core/Cargo.toml \\" >&2
    echo "    --example rfc064_phase2_stageb_export -- --alpha --out \"$ROOT\" --date $DATE" >&2
    exit 1
fi

# Pre-requisites: harness binary + install presence.
HARNESS="$REPO/target/release/spaghettio-sim"
if [[ ! -x "$HARNESS" ]]; then
    echo "[setup] building harness (one-time)..." >&2
    (cd "$REPO" && cargo build --release -p spaghettio_sim_harness) >&2 || exit 1
fi

JSON_AVAIL=1
command -v python3 >/dev/null 2>&1 || JSON_AVAIL=0

# ---- provenance ---------------------------------------------------------
GIT_HEAD="unknown"
RUSTC_VER="unknown"
FACTORIO_VER="unknown"
{ cd "$REPO" && GIT_HEAD="$(git rev-parse HEAD 2>/dev/null || true)"; } &>/dev/null || true
RUSTC_VER="$(rustc --version 2>/dev/null || echo unknown)"
INSTALL_DIR="${SPAGHETTIO_FACTORIO_DIR:-}"
if [[ -z "$INSTALL_DIR" ]]; then
    INSTALL_DIR="$(ls -d "$HOME/.cache/spaghettio-sim"/factorio-* 2>/dev/null | tail -1 || true)"
fi
if [[ -n "$INSTALL_DIR" ]]; then
    FACTORIO_VER="$("$INSTALL_DIR/bin/x64/factorio" --version 2>/dev/null | head -1 || echo unknown)"
fi
if [[ "$JSON_AVAIL" -eq 1 ]]; then
    WARMUP_DEEP="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1])).get("warmup_deep_ticks",""))' "$INDEX" 2>/dev/null)"
fi

mkdir -p "$RUNS_DIR"
cat > "$ROOT/$DATE/provenance.txt" <<EOF
job        : job2-sim-baselines
phase      : RFC-064 Phase 2 Stage B
date       : $DATE
root       : $ROOT
git HEAD   : $GIT_HEAD
rustc      : $RUSTC_VER
factorio   : $FACTORIO_VER
install    : $INSTALL_DIR
concurrency: $MAX_CONCURRENT
warmup deep: ${WARMUP_DEEP:-288000} game ticks
command    : $0 --root "$ROOT" --date "$DATE" --concurrency "$MAX_CONCURRENT"
EOF

# ---- worker -------------------------------------------------------------
# One sheet of work: TAB fields fixture, variant, bp, manifest, warmup.
#   warmup == ""  -> run at the harness default (dim-scaled)
#   warmup == "0" -> same as ""  (driver uses 0 to mean shallow/default)
#   otherwise     -> explicit --warmup value
worker() {
    local sheet="$1"
    IFS=$'\t' read -r fixture variant bp manifest warmup <<< "$sheet"
    if [[ "$warmup" == "0" ]]; then warmup=""; fi

    local outdir="$RUNS_DIR/${fixture}__${variant}"
    mkdir -p "$outdir"
    local report="$outdir/report.json"
    local log="$outdir/run.log"

    if [[ -f "$report" ]]; then
        printf '%s\t%s\t%s\t%s\t%s\n' "$fixture" "$variant" "${warmup:-default}" "already-done" "$report" >> "$RESULTS"
        return 0
    fi

    local cmd=( "$HARNESS" run --bp "$bp" --manifest "$manifest" --speed 32 --timeseries --out "$report" )
    if [[ -n "$warmup" ]]; then
        cmd+=( --warmup "$warmup" )
    fi

    # exactly one retry, per the playbook fixed-retry rule
    local status="ok"; local stage="attempt-1"
    if ! "${cmd[@]}" >"$log" 2>&1; then
        stage="attempt-2-retry"
        if ! "${cmd[@]}" >>"$log" 2>&1; then
            status="failed"
        else
            status="ok-retry"
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\n' "$fixture" "$variant" "${warmup:-default}" "$status" "$report" >> "$RESULTS"
    echo "[$stage] $fixture/$variant -> $status"
}

export -f worker
export RUNS_DIR RESULTS
export HARNESS

# ---- build the work list -------------------------------------------------
# native always; compact only when the dry stage says the geometry changed
# (a geometry-identical fixture contributes ZERO runs — it drops out of the
# sim bill for both variants; the 34-fixture × 2 = 68 budget assumes this).
# An optional per-date skip list (`$ROOT/$DATE/skip.tsv`, one fixture name per
# line, blank/#-comments allowed) removes fixtures entirely — used to drop
# sub-corpus fixtures the alpha already showed to be unmeasurable (e.g.
# fluid/fuel-starved ones that grind to their ceiling producing 0, wasting
# hours of CPU per run).
if [[ "$JSON_AVAIL" -eq 1 ]]; then
    SKIP_FILE="$ROOT/$DATE/skip.tsv"
    SKIP_ARG=""
    if [[ -f "$SKIP_FILE" ]]; then SKIP_ARG="$SKIP_FILE"; fi
    python3 - "$INDEX" "$RUNS_DIR" "$SKIP_ARG" <<'PY' > "$ROOT/$DATE/.jobs.tsv"
import json, sys
idx = json.load(open(sys.argv[1]))
skip = set()
if len(sys.argv) > 3 and sys.argv[3]:
    for line in open(sys.argv[3]):
        s = line.strip()
        if s and not s.startswith("#"):
            skip.add(s)
    sys.stderr.write(f"[skip-list] excluding {len(skip)} fixture(s): {sorted(skip)}\n")
for r in idx["rows"]:
    if r["fixture"] in skip:
        continue  # explicitly skipped for this campaign
    if not r.get("geometry_changed"):
        continue  # identical geometric output -> no native, no compact run
    n = r["native"]; c = r["compact"]
    w = str(r.get("warmup", ""))
    print("\t".join([r["fixture"], "native", n["bp"], n["manifest"], w]))
    print("\t".join([r["fixture"], "compact", c["bp"], c["manifest"], w]))
PY
else
    echo "error: python3 not available; cannot build work list" >&2
    exit 1
fi

echo "[start] $(wc -l < "$ROOT/$DATE/.jobs.tsv") runs queued (concurrency $MAX_CONCURRENT)"
printf 'fixture\tvariant\twarmup\tstatus\treport\n' > "$RESULTS"

# ---- dispatch with a hard concurrency cap --------------------------------
# Slot-based dispatcher: fill up to MAX_CONCURRENT background slots; when any
# one finishes (`wait -n`), drop the dead PIDs and refill. A slow deep run
# never holds an entire batch idle.
declare -a sheets=()         # full queue
while IFS= read -r line; do
    [[ -n "$line" ]] && sheets+=("$line")
done < "$ROOT/$DATE/.jobs.tsv"

idx=0
declare -a pids=()
while [[ $idx -lt ${#sheets[@]} || ${#pids[@]} -gt 0 ]]; do
    while [[ ${#pids[@]} -lt $MAX_CONCURRENT && $idx -lt ${#sheets[@]} ]]; do
        worker "${sheets[$idx]}" &
        pids+=("$!")
        idx=$((idx+1))
    done
    if [[ ${#pids[@]} -gt 0 ]]; then
        wait -n          # return when any background job ends (nonzero on a failing one)
        newpids=()
        for p in "${pids[@]}"; do
            if kill -0 "$p" 2>/dev/null; then
                newpids+=("$p")
            fi
        done
        pids=("${newpids[@]}")
    fi
done

# make sure all background workers have flushed their CSV row before we exit
wait
echo "[done] $ROOT/$DATE"
echo "  results: $RESULTS"
echo "  reports: $RUNS_DIR"
