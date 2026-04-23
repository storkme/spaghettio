#!/usr/bin/env python3
"""Analyze SAT junction-solver calls across a corpus of .fls snapshots.

Usage
-----
    FUCKTORIO_DUMP_SNAPSHOTS=1 cargo test \\
        --manifest-path crates/core/Cargo.toml --test e2e
    python scripts/analyze_sat_calls.py
    python scripts/analyze_sat_calls.py --dir crates/core/target/tmp

The analyzer pulls `SatInvocation` and `SatCostDescent` trace events out of
every `.fls` file in the target directory and reports:

- per-call stats (count, feasibility, solve-time percentiles, zone size,
  boundary count),
- cost-descent stats (iterations used, absolute + relative improvement,
  time breakdown),
- raw repeat rate keyed by a content hash of the zone problem. This is
  the pre-canonicalization floor: if it's already high, canonicalization
  is probably unnecessary; if it's low, canonicalization is the next
  lever to try.

Stdlib-only. Runs without a venv.
"""

from __future__ import annotations

import argparse
import base64
import gzip
import hashlib
import json
import statistics
from collections import Counter, defaultdict
from pathlib import Path

DEFAULT_DIR = Path("crates/core/target/tmp")
MAGIC = b"fls1"


def decode_fls(path: Path) -> dict:
    raw = path.read_bytes()
    if raw[:4] != MAGIC:
        raise ValueError(f"{path}: bad magic {raw[:4]!r}")
    return json.loads(gzip.decompress(base64.b64decode(raw[4:])))


def percentiles(values: list[float], ps: tuple[int, ...] = (50, 90, 99)) -> dict[int, float]:
    if not values:
        return {p: 0.0 for p in ps}
    s = sorted(values)
    out = {}
    for p in ps:
        if len(s) == 1:
            out[p] = s[0]
            continue
        k = (len(s) - 1) * p / 100.0
        lo = int(k)
        hi = min(lo + 1, len(s) - 1)
        out[p] = s[lo] + (s[hi] - s[lo]) * (k - lo)
    return out


def fmt_pct_row(label: str, values: list[float], unit: str = "") -> str:
    if not values:
        return f"  {label:<28} (no samples)"
    pct = percentiles(values, (50, 90, 99))
    return (
        f"  {label:<28} "
        f"n={len(values):<6} "
        f"p50={pct[50]:>8.1f}{unit}  "
        f"p90={pct[90]:>8.1f}{unit}  "
        f"p99={pct[99]:>8.1f}{unit}  "
        f"max={max(values):>8.1f}{unit}"
    )


def canonical_signature(data: dict) -> str:
    """Hash the zone problem (pre-canonicalization) to measure raw repeat
    rate. Sorts the boundary and forced_empty lists so field-order doesn't
    affect the hash, but does NOT rotate/reflect/rename — that's the
    next step if repeat rate is too low."""
    boundaries = sorted(
        (
            b.get("x"),
            b.get("y"),
            b.get("direction"),
            b.get("item"),
            b.get("is_input"),
            b.get("interior"),
        )
        for b in data.get("boundaries", [])
    )
    forced = sorted(tuple(xy) for xy in data.get("forced_empty", []))
    sig = {
        "w": data.get("zone_w"),
        "h": data.get("zone_h"),
        "belt_tier": data.get("belt_tier"),
        "max_reach": data.get("max_reach"),
        "boundaries": boundaries,
        "forced_empty": forced,
    }
    blob = json.dumps(sig, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(blob).hexdigest()[:16]


def collect(root: Path) -> tuple[list[dict], list[dict]]:
    """Return (sat_invocations, cost_descents). Each event has its
    originating `_file` attached for provenance."""
    invocs: list[dict] = []
    descents: list[dict] = []
    for path in sorted(root.glob("*.fls")):
        try:
            snap = decode_fls(path)
        except (OSError, ValueError, json.JSONDecodeError) as e:
            print(f"skip {path.name}: {e}")
            continue
        for ev in snap.get("trace", {}).get("events", []):
            phase = ev.get("phase")
            data = ev.get("data", {})
            if phase == "SatInvocation":
                data["_file"] = path.name
                invocs.append(data)
            elif phase == "SatCostDescent":
                data["_file"] = path.name
                descents.append(data)
    return invocs, descents


def per_call_stats(invocs: list[dict]) -> None:
    print("=" * 72)
    print("PER-CALL STATS")
    print("=" * 72)
    print(f"Total SAT calls: {len(invocs)}")
    files = {i["_file"] for i in invocs}
    print(f"Unique snapshots contributing: {len(files)}")

    if not invocs:
        return

    sat_ok = [i for i in invocs if i["satisfied"]]
    print(f"Feasibility: {len(sat_ok)}/{len(invocs)} = {100 * len(sat_ok) / len(invocs):.1f}%")
    print()

    print(fmt_pct_row("solve_time (µs)", [i["solve_time_us"] for i in invocs]))
    print(fmt_pct_row("  (satisfied only, µs)", [i["solve_time_us"] for i in sat_ok]))
    print(
        fmt_pct_row(
            "  (UNSAT only, µs)",
            [i["solve_time_us"] for i in invocs if not i["satisfied"]],
        )
    )
    print(fmt_pct_row("variables", [i["variables"] for i in invocs]))
    print(fmt_pct_row("clauses", [i["clauses"] for i in invocs]))
    print(fmt_pct_row("zone area (tiles)", [i["zone_w"] * i["zone_h"] for i in invocs]))
    print(fmt_pct_row("boundary count", [len(i.get("boundaries", [])) for i in invocs]))
    print(fmt_pct_row("forced_empty count", [len(i.get("forced_empty", [])) for i in invocs]))
    if sat_ok:
        print(fmt_pct_row("initial_cost (SAT ok)", [i["initial_cost"] for i in sat_ok]))
    print()

    # Zone-shape histogram (top shapes)
    shape_counts = Counter((i["zone_w"], i["zone_h"]) for i in invocs)
    print("Top zone shapes (w × h):")
    for (w, h), n in shape_counts.most_common(10):
        print(f"    {w:>3} × {h:<3}  {n:>5}  {100 * n / len(invocs):>5.1f}%")
    print()


def descent_stats(invocs: list[dict], descents: list[dict]) -> None:
    print("=" * 72)
    print("COST-DESCENT STATS")
    print("=" * 72)

    def key(e: dict) -> tuple:
        return (e["_file"], e["seed_x"], e["seed_y"], e["iter"], e["variant"])

    grouped: dict[tuple, list[dict]] = defaultdict(list)
    for d in descents:
        grouped[key(d)].append(d)
    for steps in grouped.values():
        steps.sort(key=lambda d: d["descent_iter"])

    sat_ok = [i for i in invocs if i["satisfied"]]
    with_descent = [i for i in sat_ok if key(i) in grouped]
    print(f"Total cost-descent events: {len(descents)}")
    print(
        f"SAT-feasible calls that entered descent: "
        f"{len(with_descent)}/{len(sat_ok)} "
        f"({100 * len(with_descent) / max(1, len(sat_ok)):.1f}%)"
    )
    print()

    if not with_descent:
        return

    iters_used = [len(grouped[key(i)]) for i in with_descent]
    print(fmt_pct_row("descent iterations used", iters_used))

    improvements_abs: list[int] = []
    improvements_rel: list[float] = []
    for i in with_descent:
        steps = grouped[key(i)]
        improving = [s for s in steps if s["cost_after"] is not None]
        if not improving:
            continue
        final_cost = improving[-1]["cost_after"]
        delta = i["initial_cost"] - final_cost
        improvements_abs.append(delta)
        if i["initial_cost"]:
            improvements_rel.append(100.0 * delta / i["initial_cost"])

    print(fmt_pct_row("absolute cost improvement", improvements_abs))
    print(fmt_pct_row("relative cost improvement", improvements_rel, unit="%"))
    print()

    # Time breakdown
    initial_us_sum = sum(i["solve_time_us"] for i in with_descent)
    descent_us_sum = sum(d["solve_time_us"] for steps in grouped.values() for d in steps)
    total = initial_us_sum + descent_us_sum
    if total:
        print(
            f"Time breakdown over descent-touching calls: "
            f"initial solve {initial_us_sum / 1000:.1f} ms "
            f"({100 * initial_us_sum / total:.1f}%), "
            f"descent solves {descent_us_sum / 1000:.1f} ms "
            f"({100 * descent_us_sum / total:.1f}%)."
        )
        unsat_descent_us = sum(
            d["solve_time_us"] for d in descents if d["cost_after"] is None
        )
        print(
            f"  of descent time, {unsat_descent_us / 1000:.1f} ms "
            f"({100 * unsat_descent_us / max(1, descent_us_sum):.1f}%) "
            f"was spent on non-improving (UNSAT / stall) attempts."
        )
    print()


def repeat_stats(invocs: list[dict]) -> None:
    print("=" * 72)
    print("REPEAT RATE (raw, pre-canonicalization)")
    print("=" * 72)
    if not invocs:
        return

    sigs: Counter[str] = Counter()
    example: dict[str, dict] = {}
    for i in invocs:
        s = canonical_signature(i)
        sigs[s] += 1
        example.setdefault(s, i)

    unique = len(sigs)
    total = sum(sigs.values())
    repeated = sum(n for n in sigs.values() if n > 1)
    print(f"Unique signatures: {unique} / {total} calls")
    print(f"Calls whose signature recurs (seen >=2x): "
          f"{repeated} ({100 * repeated / total:.1f}%)")
    print(f"Cache hit ceiling (calls saved if perfect cache): "
          f"{total - unique} ({100 * (total - unique) / total:.1f}%)")
    print()

    # Distribution of repeat counts
    repeat_hist = Counter(sigs.values())
    print("Signature occurrence histogram:")
    for count, n_sigs in sorted(repeat_hist.items()):
        print(f"    seen {count:>4}×  {n_sigs:>5} signatures  "
              f"({n_sigs * count:>5} calls)")
    print()

    print("Top 10 most frequent signatures:")
    for s, n in sigs.most_common(10):
        ex = example[s]
        print(
            f"    {s}  {n:>4}×  "
            f"{ex['zone_w']}×{ex['zone_h']}  "
            f"tier={ex['belt_tier']}  "
            f"boundaries={len(ex.get('boundaries', []))}  "
            f"first_seen={ex['_file']}"
        )
    print()


def filter_invocs(
    invocs: list[dict],
    min_solve_us: int,
    min_area: int,
    min_boundaries: int,
) -> list[dict]:
    """Keep only calls that look interesting. Filters are AND-ed."""
    out = []
    for i in invocs:
        if i["solve_time_us"] < min_solve_us:
            continue
        if i["zone_w"] * i["zone_h"] < min_area:
            continue
        if len(i.get("boundaries", [])) < min_boundaries:
            continue
        out.append(i)
    return out


def filter_descents(descents: list[dict], kept: set[tuple]) -> list[dict]:
    """Keep descent events whose parent invocation survived filtering."""

    def key(e: dict) -> tuple:
        return (e["_file"], e["seed_x"], e["seed_y"], e["iter"], e["variant"])

    return [d for d in descents if key(d) in kept]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dir",
        type=Path,
        default=DEFAULT_DIR,
        help=f"Directory of .fls files (default: {DEFAULT_DIR})",
    )
    parser.add_argument(
        "--min-solve-us",
        type=int,
        default=0,
        help="Only include SAT calls with solve_time_us >= this. Use to "
             "focus on the painful tail and ignore sub-millisecond trivia.",
    )
    parser.add_argument(
        "--min-area",
        type=int,
        default=0,
        help="Only include SAT calls with zone_w * zone_h >= this.",
    )
    parser.add_argument(
        "--min-boundaries",
        type=int,
        default=0,
        help="Only include SAT calls with >= this many boundaries.",
    )
    args = parser.parse_args()

    if not args.dir.is_dir():
        print(f"no such directory: {args.dir}")
        print("run: FUCKTORIO_DUMP_SNAPSHOTS=1 cargo test "
              "--manifest-path crates/core/Cargo.toml --test e2e")
        return 1

    invocs, descents = collect(args.dir)
    if not invocs and not descents:
        print(f"no SAT events in {args.dir}")
        return 1

    raw_count = len(invocs)
    if args.min_solve_us or args.min_area or args.min_boundaries:

        def invoc_key(e: dict) -> tuple:
            return (e["_file"], e["seed_x"], e["seed_y"], e["iter"], e["variant"])

        invocs = filter_invocs(
            invocs, args.min_solve_us, args.min_area, args.min_boundaries
        )
        descents = filter_descents(descents, {invoc_key(i) for i in invocs})
        print(
            f"Filters applied: min_solve_us={args.min_solve_us} "
            f"min_area={args.min_area} min_boundaries={args.min_boundaries}  ->  "
            f"{len(invocs)}/{raw_count} SAT calls survive"
        )
        print()
        if not invocs:
            print("nothing to analyze after filtering.")
            return 0

    per_call_stats(invocs)
    descent_stats(invocs, descents)
    repeat_stats(invocs)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
