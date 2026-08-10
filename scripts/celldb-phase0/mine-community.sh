#!/usr/bin/env bash
# Phase-0 probe 4 (cell-interface DB): mine the downloaded community
# blueprint corpus for (a) an independent demand distribution over recipe
# motifs and (b) per-motif density/aspect baselines — the numbers our
# generated layouts have to beat — plus donor candidates (single-recipe
# arrays legal under the engine's modeled mechanics).
#
# Stage 1 of 2: expand every corpus file through blueprint-analyze --json
# into one JSONL of per-blueprint records (books expanded by the analyzer).
# Summarize with summarize-community.sh.
#
# CORPUS default points at the untracked download dir in the primary
# checkout; override for other hosts. Failures are counted and listed, not
# silently skipped.
set -euo pipefail
CORPUS="${CORPUS:-/home/stork/code/fucktorio/scripts/blueprints}"
OUT="${OUT:-./celldb-phase0-work}"
mkdir -p "$OUT"

BIN="$(git rev-parse --show-toplevel)/target/release/blueprint-analyze"
[ -x "$BIN" ] || {
  echo "building blueprint-analyze..." >&2
  cargo build --manifest-path "$(git rev-parse --show-toplevel)/crates/core/Cargo.toml" \
    -p spaghettio_mining --bin blueprint-analyze --release
}

: > "$OUT/community.jsonl"
: > "$OUT/mine_failures.txt"
n=0
for f in "$CORPUS"/*.json; do
  base="$(basename "$f")"
  if ! jq -r '.blueprintString // empty' "$f" > "$OUT/.bp.txt" 2>/dev/null || [ ! -s "$OUT/.bp.txt" ]; then
    printf 'no-blueprint-string\t%s\n' "$base" >> "$OUT/mine_failures.txt"; continue
  fi
  if ! "$BIN" --json "$OUT/.bp.txt" > "$OUT/.res.json" 2>/dev/null; then
    printf 'analyze-failed\t%s\n' "$base" >> "$OUT/mine_failures.txt"; continue
  fi
  # Books nest: an output element can itself be an array. Recursively take
  # every object that looks like a blueprint record, at any depth. Guarded —
  # a malformed record is a counted failure, not a dead loop under set -e.
  if ! jq -c --arg src "$base" \
      '[.. | objects | select(has("recipe_groups") and has("machine_count"))]
       | .[] | . + {source: $src}' "$OUT/.res.json" >> "$OUT/community.jsonl" 2>/dev/null; then
    printf 'jq-extract-failed\t%s\n' "$base" >> "$OUT/mine_failures.txt"; continue
  fi
  n=$((n+1))
done
rm -f "$OUT/.bp.txt" "$OUT/.res.json"

echo "corpus files analyzed : $n"
echo "blueprint records     : $(wc -l < "$OUT/community.jsonl")  (books expanded)"
nf=$(wc -l < "$OUT/mine_failures.txt")
if [ "$nf" -gt 0 ]; then
  echo "WARNING: $nf file(s) failed — listed in $OUT/mine_failures.txt; the"
  echo "         demand distribution below is missing them."
fi
