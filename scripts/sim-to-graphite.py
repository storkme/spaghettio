#!/usr/bin/env python3
"""Push a sim-harness report's per-item time series to Grafana Cloud Graphite.

The harness scenario already samples every planned item's cumulative
production every 1200 ticks (`storage.samples`, scenario.rs:1456) and emits
it into `raw_result.samples`. That is the same data graftorio would give us —
Factorio's own `get_item_production_statistics` — so no mod is involved.

Counters are pushed CUMULATIVE; derive rates in Grafana with `perSecond()`.
Pushing pre-computed rates would turn a dropped sample into a fake spike.

Usage:
    scripts/sim-to-graphite.py <report.json> [--arm lift] [--dry-run]

Auth: reads the Grafana Cloud API token (needs `metrics:write`) from
$GRAFANA_GRAPHITE_TOKEN, else ~/.config/spaghettio/grafana-token.
"""
import argparse
import json
import os
import pathlib
import sys
import urllib.request

GRAPHITE_URL = "https://graphite-prod-55-prod-gb-south-1.grafana.net/graphite/metrics"
GRAPHITE_USER = "3416501"
TOKEN_FILE = pathlib.Path.home() / ".config" / "spaghettio" / "grafana-token"
TICKS_PER_SECOND = 60
# scenario.rs samples on `ev.tick % 1200 == 0`
SAMPLE_INTERVAL_TICKS = 1200
SERIES = ("produced", "drained", "fed")


def load_token() -> str:
    tok = os.environ.get("GRAFANA_GRAPHITE_TOKEN")
    if tok:
        return tok.strip()
    if TOKEN_FILE.is_file():
        return TOKEN_FILE.read_text().strip()
    sys.exit(
        f"no token: set $GRAFANA_GRAPHITE_TOKEN or write it to {TOKEN_FILE}"
    )


def build_points(report: dict, arm: str, label: str, run_id: str, end_wallclock: int):
    """One Graphite datapoint per (series, item, sample)."""
    raw = report.get("raw_result", {})
    samples = raw.get("samples") or []
    if not samples:
        sys.exit("report has no raw_result.samples — nothing to push")

    rep = report.get("report", {})
    item_reports = rep.get("items") or []
    targets = [i.get("item") for i in item_reports if i.get("is_target")]
    target_tag = targets[0] if targets else label

    samples = sorted(samples, key=lambda s: s.get("tick", 0))
    max_tick = max(s.get("tick", 0) for s in samples)
    interval_s = SAMPLE_INTERVAL_TICKS // TICKS_PER_SECOND

    # Anchor the LAST sample at the run's end wall-clock, walk backwards in
    # GAME seconds, and snap to `interval_s` boundaries. Metrictank buckets
    # by interval; unaligned timestamps read back as nulls under any function
    # that needs consecutive points (perSecond returned all-null before this).
    def ts_for(tick: int) -> int:
        raw = end_wallclock - (max_tick - tick) // TICKS_PER_SECOND
        return raw - (raw % interval_s)

    def tag_list(item, series):
        return [
            f"item={item}",
            f"series={series}",
            f"fixture={label}",
            f"arm={arm}",
            f"target={target_tag}",
            f"run={run_id}",
        ]

    points = []
    prev = None
    for s in samples:
        tick = s.get("tick", 0)
        ts = ts_for(tick)
        for series in SERIES:
            cur_map = s.get(series) or {}
            for item, value in cur_map.items():
                if not isinstance(value, (int, float)):
                    continue
                points.append(
                    {
                        "name": f"spaghettio.sim.{series}",
                        "value": float(value),
                        "time": ts,
                        "interval": interval_s,
                        "tags": tag_list(item, series),
                    }
                )
                # Rate computed HERE, from the authoritative game-tick delta,
                # rather than left to Graphite's perSecond() — which has to
                # infer the step from wall-clock spacing and gets it wrong for
                # a batch backfill. This is items per GAME second, directly
                # comparable to `planned_rate`.
                if prev is not None:
                    dtick = tick - prev["tick"]
                    pv = (prev.get(series) or {}).get(item)
                    if dtick > 0 and isinstance(pv, (int, float)):
                        rate = (value - pv) / (dtick / TICKS_PER_SECOND)
                        if rate >= 0:
                            points.append(
                                {
                                    "name": f"spaghettio.sim.rate_{series}",
                                    "value": float(rate),
                                    "time": ts,
                                    "interval": interval_s,
                                    "tags": tag_list(item, f"rate_{series}"),
                                }
                            )
        prev = s
    # `planned_rate` is constant for a run but pushing it as a series (at the
    # first and last sample) lets a dashboard divide measured by planned per
    # stage without the plan being baked into the panel.
    for ir in item_reports:
        item, planned = ir.get("item"), ir.get("planned_rate")
        if item is None or not isinstance(planned, (int, float)):
            continue
        for ts in [ts_for(sm.get("tick", 0)) for sm in samples]:
            points.append(
                {
                    "name": "spaghettio.sim.planned_rate",
                    "value": float(planned),
                    "time": ts,
                    "interval": interval_s,
                    "tags": [
                        *tag_list(item, "planned_rate")[:1],
                        "series=planned_rate",
                        f"fixture={label}",
                        f"arm={arm}",
                        f"target={target_tag}",
                        f"run={run_id}",
                    ],
                }
            )
    return points


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("report", type=pathlib.Path)
    ap.add_argument("--arm", default="unknown", help="e.g. lift / main")
    ap.add_argument("--label", default=None, help="fixture label (default: report stem)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    report = json.loads(args.report.read_text())
    label = args.label or args.report.stem.replace("-report", "")
    run_id = f"{label}-{int(args.report.stat().st_mtime)}"
    end_wallclock = int(args.report.stat().st_mtime)

    points = build_points(report, args.arm, label, run_id, end_wallclock)
    items = sorted({t.split("=", 1)[1] for p in points for t in p["tags"] if t.startswith("item=")})
    print(f"{len(points)} datapoints | {len(items)} items | run={run_id}")
    print(f"  items: {', '.join(items)}")

    if args.dry_run:
        print(json.dumps(points[:3], indent=2))
        return

    token = load_token()
    body = json.dumps(points).encode()
    req = urllib.request.Request(GRAPHITE_URL, data=body, method="POST")
    req.add_header("Content-Type", "application/json")
    import base64

    auth = base64.b64encode(f"{GRAPHITE_USER}:{token}".encode()).decode()
    req.add_header("Authorization", f"Basic {auth}")
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            print(f"pushed: HTTP {resp.status} {resp.read(200).decode(errors='replace')}")
    except urllib.error.HTTPError as e:
        sys.exit(f"push failed: HTTP {e.code} {e.read(500).decode(errors='replace')}")


if __name__ == "__main__":
    main()
