#!/usr/bin/env -S uv run --no-project
# /// script
# requires-python = ">=3.11"
# ///
"""Generate a markdown report from `bake_cp_sat_runner.py` output.

Reads:
  scripts/cp_sat_journal.jsonl   — successful solves (one line per shape)
  /tmp/cp_sat_sweep_*.tsv        — per-probe results from the latest run

Writes:
  docs/bake-overnight-results.md  (or path given via --out)

Usage:
  scripts/bake_cp_sat_report.py
  scripts/bake_cp_sat_report.py --tsv /tmp/cp_sat_sweep_20260503_021500.tsv
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from collections import defaultdict
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
JOURNAL = REPO_ROOT / "scripts" / "cp_sat_journal.jsonl"
DEFAULT_OUT = REPO_ROOT / "docs" / "bake-overnight-results.md"


def latest_tsv() -> Path | None:
    candidates = sorted(Path("/tmp").glob("cp_sat_sweep_*.tsv"))
    return candidates[-1] if candidates else None


def load_journal() -> dict[tuple[int, int], dict]:
    out: dict[tuple[int, int], dict] = {}
    if not JOURNAL.exists():
        return out
    with open(JOURNAL) as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            shape = entry.get("shape")
            if not (isinstance(shape, list) and len(shape) == 2):
                continue
            key = (shape[0], shape[1])
            out[key] = entry  # last write wins (re-bakes overwrite)
    return out


def load_tsv(path: Path) -> list[dict]:
    rows: list[dict] = []
    with open(path) as f:
        reader = csv.DictReader(f, delimiter="\t")
        for row in reader:
            rows.append(row)
    return rows


def summarise_per_shape(tsv_rows: list[dict]) -> dict[tuple[int, int], dict]:
    """Aggregate per-shape: total attempts, OK count, fastest OK seed/wall,
    median wall (only of probes that finished, not timeouts)."""
    by_shape: dict[tuple[int, int], list[dict]] = defaultdict(list)
    for row in tsv_rows:
        try:
            n_str, m_str = row["shape"].split(",")
            shape = (int(n_str), int(m_str))
        except (ValueError, KeyError):
            continue
        by_shape[shape].append(row)
    out: dict[tuple[int, int], dict] = {}
    for shape, rows in by_shape.items():
        ok_rows = [r for r in rows if r["status"] == "OK"]
        ok_walls = sorted(float(r["wall_s"]) for r in ok_rows)
        all_walls = sorted(float(r["wall_s"]) for r in rows)
        median = all_walls[len(all_walls) // 2] if all_walls else None
        fastest = ok_rows[0] if ok_rows else None
        if ok_rows:
            fastest = min(ok_rows, key=lambda r: float(r["wall_s"]))
        out[shape] = {
            "attempts": len(rows),
            "ok_count": len(ok_rows),
            "fastest_seed": int(fastest["seed"]) if fastest else None,
            "fastest_wall_s": float(fastest["wall_s"]) if fastest else None,
            "fastest_entities": int(fastest["entities"]) if fastest and fastest["entities"] else None,
            "median_wall_s": median,
            "max_wall_s": all_walls[-1] if all_walls else None,
        }
    return out


def render_report(journal: dict, tsv_summary: dict, tsv_path: Path | None) -> str:
    lines: list[str] = []
    lines.append("# Overnight CP-SAT bake — results")
    lines.append("")
    lines.append(f"Generated from journal `{JOURNAL.relative_to(REPO_ROOT)}`")
    if tsv_path is not None:
        lines.append(f"and sweep log `{tsv_path}`.")
    lines.append("")
    lines.append("## Solved shapes")
    lines.append("")
    if not journal:
        lines.append("_(none yet)_")
        lines.append("")
    else:
        lines.append("| Shape | Seed | Wall (s) | Entities | Solver wall (ms) |")
        lines.append("|-------|-----:|---------:|---------:|-----------------:|")
        for shape, entry in sorted(journal.items()):
            n_ents = len(entry["template"].get("entities", [])) if entry.get("template") else "?"
            wall = entry.get("wall_s")
            wall_str = f"{wall:.1f}" if isinstance(wall, (int, float)) else "?"
            solve_ms = entry.get("solve_wall_ms", "?")
            lines.append(
                f"| `({shape[0]}, {shape[1]})` | {entry.get('seed', '?')} | "
                f"{wall_str} | {n_ents} | {solve_ms} |"
            )
        lines.append("")
    lines.append("## Sweep summary")
    lines.append("")
    if not tsv_summary:
        lines.append("_(no TSV log found)_")
        lines.append("")
    else:
        lines.append("| Shape | Attempts | OK | Fastest seed | Fastest wall (s) | Median wall (s) | Max wall (s) |")
        lines.append("|-------|---------:|---:|-------------:|------------------:|----------------:|-------------:|")
        for shape, s in sorted(tsv_summary.items()):
            fast = f"{s['fastest_wall_s']:.1f}" if s['fastest_wall_s'] is not None else "—"
            med = f"{s['median_wall_s']:.1f}" if s['median_wall_s'] is not None else "—"
            mx = f"{s['max_wall_s']:.1f}" if s['max_wall_s'] is not None else "—"
            seed = s['fastest_seed'] if s['fastest_seed'] is not None else "—"
            lines.append(
                f"| `({shape[0]}, {shape[1]})` | {s['attempts']} | {s['ok_count']} | "
                f"{seed} | {fast} | {med} | {mx} |"
            )
        lines.append("")
    # Recommendations.
    lines.append("## Recommendations")
    lines.append("")
    if not tsv_summary:
        lines.append("- No sweep data to recommend from.")
    else:
        for shape, s in sorted(tsv_summary.items()):
            n, m = shape
            if s["ok_count"] == 0:
                lines.append(
                    f"- `({n}, {m})`: NO seed solved out of {s['attempts']} attempts — "
                    f"likely needs layout work, not more compute."
                )
            elif s["ok_count"] < s["attempts"] // 4:
                lines.append(
                    f"- `({n}, {m})`: solved at seed {s['fastest_seed']} in "
                    f"{s['fastest_wall_s']:.1f}s. Pin this seed in the round-trip test. "
                    f"Only {s['ok_count']} / {s['attempts']} seeds solved — "
                    f"seed-sensitive, treat the pin as load-bearing."
                )
            else:
                lines.append(
                    f"- `({n}, {m})`: solved {s['ok_count']} / {s['attempts']} seeds; "
                    f"fastest seed={s['fastest_seed']} at {s['fastest_wall_s']:.1f}s. "
                    f"Pick this seed for the test."
                )
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument("--tsv", type=Path, default=None,
                        help="Path to sweep TSV (default: latest /tmp/cp_sat_sweep_*.tsv)")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT,
                        help=f"Output markdown path (default: {DEFAULT_OUT.relative_to(REPO_ROOT)})")
    args = parser.parse_args()
    tsv_path = args.tsv or latest_tsv()
    tsv_summary = summarise_per_shape(load_tsv(tsv_path)) if tsv_path else {}
    journal = load_journal()
    report = render_report(journal, tsv_summary, tsv_path)
    args.out.write_text(report)
    print(f"wrote {args.out}", file=sys.stderr)
    print(report)
    return 0


if __name__ == "__main__":
    sys.exit(main())
