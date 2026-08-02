#!/usr/bin/env python3
"""Live per-window telemetry scorer for spaghettio-sim.

Consumes the CSV that `run --timeseries` (or `serve`) streams to
`<scratch>/script-output/timeseries.csv` and renders a progress / convergence
scoreboard from the inside out: for every planned item, the trailing
per-window rate against its asymptotic ideal (planned rate), plus a rollup of
every machine's status — so a starving / 0-output fixture is identifiable in
minutes instead of after a multi-hour grind-to-ceiling.

CSV columns (scenario-defined): tick,kind,unit,name,x,y,crafts_delta,status,
item,produced_delta   -- kind in {machine, item}; machine rows carry crafts/lib
status, item rows carry produced_delta (per-window, closed on accumulated items
or tick cap, same windows the JSON report's checkpoints use).

Usage:
  scripts/sim-watch.py <src> [--plan item=rate,...] [--follow] [--windows N]
    <src>      a path to a timeseries.csv, OR a substring matching a live run's
               scratch dir under $TMPDIR/spaghettio-sim-runs/ (e.g. a fixture
               like "processing_unit_from_ore_am3__native")
    --plan     planned rates to score against (e.g. uranium-235=0.1,light-oil=5).
               Omitted -> prints measured rates only (no pct verdicts).
    --windows  how many trailing windows to average for the "current" rate
               (default 3 — the harness's own stability group size).
    --follow   keep re-reading appended rows and redraw (tail -f style).

Exit status: 2 = no CSV found / no windows yet (still in warmup).
"""

import argparse
import glob
import os
import sys
import time

KIND_MACHINE = "machine"
KIND_ITEM = "item"


def find_csv(src: str) -> str:
    if os.path.isfile(src) and src.endswith(".csv"):
        return src
    if os.path.isdir(src):
        cand = os.path.join(src, "script-output", "timeseries.csv")
        if os.path.isfile(cand):
            return cand
        src = os.path.basename(src)
    # glob live scratch dirs: $TMPDIR/spaghettio-sim-runs/<substring>*/
    bases = [os.environ.get(v) for v in ("TMPDIR", "TMP", "TEMP")]
    bases = [b for b in bases if b]
    bases.append("/tmp")  # Linux default for std::env::temp_dir when TMPDIR unset
    for base in bases:
        hits = sorted(
            glob.glob(
                os.path.join(
                    base,
                    "spaghettio-sim-runs",
                    f"*{src}*",
                    "script-output",
                    "timeseries.csv",
                )
            )
        )
        if hits:
            # newest by mtime, not lexicographic PID order — with multiple
            # live same-named runs we must attach to the most recently updated.
            return max(hits, key=os.path.getmtime)
    return ""


def parse(path: str):
    """Return (machine_rows, item_rows) with columns addressed by name."""
    machines = []  # (tick, unit, name, status)
    items = []  # (tick, item, produced_delta)
    with open(path) as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                continue
            c = line.split(",")
            if len(c) < 10:
                continue
            kind = c[1]
            if kind not in (KIND_MACHINE, KIND_ITEM):
                continue  # header or stray row
            try:
                tick = int(c[0])
            except ValueError:
                continue
            if kind == KIND_MACHINE:
                try:
                    unit = int(c[2])
                except ValueError:
                    unit = -1
                machines.append((tick, unit, c[3], c[7]))
            elif kind == KIND_ITEM:
                try:
                    delta = int(c[9])
                except ValueError:
                    continue
                items.append((tick, c[8], delta))
    return machines, items


def per_window_rates(items):
    """item -> list of (tick, rate) using consecutive window tick deltas."""
    by_item = {}
    order = []
    for tick, item, delta in items:
        by_item.setdefault(item, []).append((tick, delta))
        if item not in order:
            order.append(item)
    rates = {}
    for item in order:
        series = by_item[item]
        out = []
        prev_tick = None
        for tick, delta in series:
            if prev_tick is not None:
                dt = (tick - prev_tick) / 60.0
                rate = delta / dt if dt > 0 else 0.0
                out.append((tick, rate))
            prev_tick = tick
        rates[item] = out
    return rates, order


def machine_status_rollup(machines):
    """Status counts at the latest sampled tick."""
    latest = {}
    for tick, unit, name, status in machines:
        latest[(unit, name)] = (tick, status)
    counts = {}
    for (unit, name), (tick, status) in latest.items():
        counts[status] = counts.get(status, 0) + 1
    return counts


def score(rate, plan):
    if rate is None:
        # a window has closed but we cannot yet derive a run-rate (need two
        # consecutive windows) — this is a healthy warmup/ramp transition, NOT
        # a dead run. Never label it DEAD.
        return "warming"
    if plan is None or plan <= 0.0:
        return ""
    if rate <= 0.0:
        return "DEAD"
    pct = rate / plan * 100.0
    if pct >= 98.0:
        return f"PASS {pct:.0f}%"
    if pct >= 90.0:
        return f"WARN {pct:.0f}%"
    return f"FAIL {pct:.0f}%"


def render(path, plan_map, nwindows):
    if not os.path.isfile(path):
        print(f"no csv yet: {path}", file=sys.stderr)
        return 2
    machines, items = parse(path)
    rates, order = per_window_rates(items)
    counts = machine_status_rollup(machines)

    if not order:
        print(f"[{path}] collected, no item windows closed yet (still warming up)")
        return 2

    print(
        f"--- {os.path.basename(os.path.dirname(os.path.dirname(path)))} @ tick {machines[-1][0] if machines else '?'} ---"
    )
    # header
    head = (
        "item".ljust(28)
        + "windows".rjust(4)
        + ("last-rate").rjust(11)
        + ("idea").rjust(10)
        + ("vs-ideal").rjust(12)
    )
    print(head)
    for item in order:
        r = rates[item]
        if len(r) == 0:
            cur = None  # no run-rate yet (fewer than two closed windows)
        elif len(r) < nwindows:
            cur = r[-1][1]
        else:
            use = r[-nwindows:]
            cur = sum(x[1] for x in use) / len(use)
        plan = plan_map.get(item)
        vs = score(cur, plan)
        curstr = "      n/a" if cur is None else f"{cur:10.3f}"
        planstr = f"{plan:9.2f}" if plan and plan > 0 else f"{'n/a':>9}"
        print(f"{item:28} {len(r):4} {curstr} {planstr} {vs:>14}")
    if counts:
        statuses = ", ".join(
            f"{k}={v}" for k, v in sorted(counts.items(), key=lambda kv: -kv[1])
        )
        print(f"machines: {statuses}")
    print_starvation(machines, rates, nwindows)
    return 0


STARVED = [
    "fluid_ingredient_shortage",
    "no_power",
    "no_fuel",
    "item_ingredient_shortage",
]


def print_starvation(machines, rates, nwindows):
    """Advisory kill signal, gated hard so it does not fire on a healthy ramp.

    `item/fluid_ingredient_shortage` and `no_fuel` are NORMAL transient states
    while a factory fills its belts on the way to plan — flagging them on a
    single window mislabels a healthy startup as doomed. Only recommend a kill
    when BOTH hold:
      (1) the starvation status has persisted across >= 2 of the last 3 closed
          windows (not just the latest snapshot), and
      (2) no item has produced a positive run-rate in the trailing window(s) —
          a plant that is producing despite a shortage is transiently starved,
          not dead.
    """
    by_tick = {}
    for tick, unit, name, status in machines:
        d = by_tick.setdefault(tick, {})
        d[status] = d.get(status, 0) + 1
    ticks = sorted(by_tick)
    if not ticks:
        return
    recent = by_tick[ticks[-1]]
    starved_here = [s for s in STARVED if recent.get(s)]
    if not starved_here:
        return  # no starvation in the latest window at all
    persisted = 0
    for t in ticks[-3:]:
        if any(by_tick[t].get(s) for s in STARVED):
            persisted += 1
    recent_rates = [rr for series in rates.values() for _t, rr in series[-nwindows:]]
    any_progress = any(rr > 0.0 for rr in recent_rates)
    if persisted < 2 or any_progress:
        detail = "persisting" if persisted >= 2 else "transient"
        note = "watch" if not any_progress else "ramping (producing, ignoring)"
        print(
            f".. {', '.join(starved_here)} machine(s) seen ({detail}, {note}) — "
            f"not recommending kill yet"
        )
        return
    print(
        f"!! STARVED: {', '.join(starved_here)} machine(s) persisted across "
        f"{persisted} consecutive windows with zero output -> consider kill"
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("--plan", default="")
    ap.add_argument("--windows", type=int, default=3)
    ap.add_argument("--follow", action="store_true")
    args = ap.parse_args()

    plan_map = {}
    for tok in filter(None, args.plan.split(",")):
        if "=" in tok:
            k, v = tok.rsplit("=", 1)
            plan_map[k] = float(v)
    path = find_csv(args.src)
    if not path:
        print(
            f"no CSV found for {args.src!r} (warmup not closed, or done/scratch removed) "
            f"- after finalize use the run's report.json checkpoints instead",
            file=sys.stderr,
        )
        return 2

    if not args.follow:
        return render(path, plan_map, args.windows)

    missing_cycles = 0
    try:
        while True:
            if not os.path.exists(path):
                missing_cycles += 1
                print(
                    f"[{path}] gone (finalized/cleaned) — reading the run's "
                    f"--out report.json timeseries instead",
                    file=sys.stderr,
                )
                if missing_cycles >= 2:
                    break
                time.sleep(5)
                continue
            missing_cycles = 0
            render(path, plan_map, args.windows)
            print("---")
            time.sleep(5)
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    sys.exit(main())
