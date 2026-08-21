#!/usr/bin/env python3
"""Localize WHERE a sim-harness run is failing: starved machines + a map.

Read-only forensics over a `spaghettio-sim run --out report.json` (see
docs/sim-harness.md, docs/sim-harness-forensics.md). Answers "where" —
which machines are starved of what, since when, and the surrounding
belts. Renders and ranks; no root-cause, no belt-walking to first-empty-
tile, no validator cross-referencing.

Usage: python3 scripts/sim-localize.py <report.json> [--top N]
       [--radius N] [--around X,Y[,R]]
--top N     table and lane-detail the worst N machines (default: table
            20, detail 3). Given explicitly, also windows the map on
            their bbox; by default the map is the full extent (the first
            look usually wants the whole factory). --around overrides.
--radius N  lane-detail radius (default 3, or --around's R when given).
--around X,Y[,R]  window the map on this tile (default R=15) AND anchor
            the lane detail there instead of on the worst machines.
            Negative coordinates: --around=-5,-3 (or bare; both work).

Belt `n` — LOAD-BEARING: a transport LINE spans several tiles, and
`get_item_count()` returns the WHOLE LINE's count, so `n` repeats across
every tile of one straight run. Never per-tile: treated here as
empty-vs-nonempty only, never summed along a run or printed as
items-per-tile (yesterday's improvised read made exactly that mistake).

Formats: OLD dumps (web/src/ui/testdata/sim-report-*.json) lack
timeseries/validator_standing/kit_errors, and belts are [x, y, n] or
[x, y, n, det] (nonempty belts only).
NEW dumps add those fields and extend belts to [x, y, n, det, name,
direction, ug_type] (det: per-line [[item, count], ...], index 0=left/
1=right lane; direction: raw Factorio 2.0 defines.direction, 16-way,
0/4/8/12=N/E/S/W; ug_type: "input"/"output"/null). Every tuple here
tolerates extra trailing elements; every access degrades to a one-line
note, never a crash.
"""
import argparse
import json
import sys

def load_report(path):
    with open(path) as f:
        return json.load(f)

# Factorio `defines.entity_status` names, grouped the way the forensics doc
# reads them. Anything not `working` and in neither set still counts as
# impaired (`frac_other`) so an unfamiliar status is surfaced, not hidden.
SHORTAGE = {
    "item_ingredient_shortage", "fluid_ingredient_shortage", "no_ingredients",
    "no_input_fluid", "low_input_fluid", "no_power", "low_power", "no_fuel",
}
BACKPRESSURE = {"full_output", "full_burnt_result_output", "fluid_production_overload"}

def rank_key(r):
    """One sort key for both ranking paths: worst first, then stable by tile."""
    x = r["x"] if r["x"] is not None else 0  # missing ≠ coordinate 0, but both sort after nothing
    y = r["y"] if r["y"] is not None else 0
    return (-r["frac_shortage"], -r["frac_backpressure"], -r.get("frac_other", 0.0), x, y)
INSERTER_GLYPH = {
    "working": "i",
    "waiting_for_source_items": "s",
    "waiting_for_space_in_destination": "d",
    "waiting_for_more_items": "m",
}
ARROWS = ["^", ">", "v", "<"]  # indexed by direction // 4 % 4 (real belts are only ever 0/4/8/12)

POWER = {"no_power", "low_power", "no_fuel"}

def machine_glyph(status):
    """Derived from the same status sets the ranking uses, so the map never
    draws a top-ranked shortage as the catch-all `?`."""
    if status == "working":
        return "W"
    if status in POWER:
        return "P"
    if status in SHORTAGE:
        return "S"
    if status in BACKPRESSURE:
        return "F"
    return "?"

def arrow_for(direction):
    return None if direction is None else ARROWS[int(direction // 4) % 4]

def g(d, *path, default=None):
    """Nested .get() that never raises on a missing key or wrong shape."""
    cur = d
    for key in path:
        if not isinstance(cur, dict) or key not in cur:
            return default
        cur = cur[key]
    return cur if cur is not None else default

def fmt(v):
    return "-" if v is None else f"{v:.2f}"

def fmt_pct(v):
    return "-" if v is None else f"{v:+.1f}%"

# --- header: kit check, verdict, validator line, item table -----------------

def validator_line(top):
    """Mirror crates/sim-harness/src/report.rs's validator_line()."""
    report = top.get("report", {})
    if "validator_standing" not in report:
        return "unknown (pre-field report)"
    v = report.get("validator")
    if v is None:
        return "? (manifest predates the validator field — state unknown, NOT clean)"
    errors, warnings, layout_w = v.get("errors", 0), v.get("warnings", 0), v.get("layout_warnings", 0)
    if errors == 0 and warnings == 0 and layout_w == 0:
        return "clean (no issues reported — note validator silence is not proof of correctness)"
    badge = "/".join(f"{n}{c}" for n, c in ((errors, "E"), (warnings, "W"), (layout_w, "L")) if n)
    by_cat = v.get("by_category", {}) or {}
    ranked = sorted(by_cat.items(), key=lambda kv: (kv[1].get("errors", 0), kv[1].get("warnings", 0)), reverse=True)
    cats = [f"{k}×{c.get('errors', 0) + c.get('warnings', 0)}" for k, c in ranked[:4]]
    return f"{badge} — {', '.join(cats)}" if cats else badge

def kit_errors_of(top):
    return g(top, "report", "kit_errors", default=None) or g(top, "raw_result", "kit_errors", default=[])

def print_header(top):
    report = top.get("report", {})
    kit_errors = kit_errors_of(top)
    if kit_errors:
        print("!! KIT ERRORS — run is invalid, rates below are not interpretable")
        for e in kit_errors:
            print(f"   {e}")
        print()

    label = report.get("label") or g(top, "run_params", "scenario_name", default="?")
    print(f"=== sim-localize: {label} ===")
    print(f"verdict: {report.get('overall_verdict', '?')}   converged: {report.get('converged')}")
    print(f"validator: {validator_line(top)}")
    print()

    items = report.get("items") or []
    if not items:
        print("(no per-item table — report.items absent)\n")
        return
    print(f"{'item':<28} {'planned/s':>10} {'produced/s':>11} {'delivered/s':>12} {'d%':>8}")
    target, target_ratio = None, None
    for it in items:
        mark = "*" if it.get("is_target") else " "
        planned, produced, delivered = it.get("planned_rate"), it.get("measured_produced_rate"), it.get("measured_delivered_rate")
        dpct = it.get("delta_pct_delivered")
        if dpct is None:
            dpct = it.get("delta_pct_produced")
        print(f"{mark + it.get('item', '?'):<28} {fmt(planned):>10} {fmt(produced):>11} {fmt(delivered):>12} {fmt_pct(dpct):>8}")
        if it.get("is_target"):
            target = it
            ratio = None
            if delivered is not None and planned:
                ratio = delivered / planned
            elif produced is not None and planned:
                ratio = produced / planned
            if ratio is not None and (target_ratio is None or ratio < target_ratio):
                target_ratio = ratio  # with several targets the worst one governs
    print()

    if target is not None and target_ratio is not None and target_ratio < 1.0:
        # Only when the target actually falls short. The report carries no
        # recipe graph, so "upstream" is not verifiable from it — this is a
        # plain sorted listing, and the caveat printed with it is load-bearing.
        short = sorted(
            (it["measured_produced_rate"] / it["planned_rate"], it["item"])
            for it in items
            if not it.get("is_target") and it.get("planned_rate")
            and it.get("measured_produced_rate") is not None
        )
        short = [(r, name) for r, name in short if r < 0.98]
        if short:
            print("below-plan intermediates, most short first: "
                  + ", ".join(f"{name} {r * 100:.0f}%" for r, name in short))
            print("  (a listing, not a causal order — a BACKED-UP stage reads below plan just like a "
                  "starved one; cross-check the ranking's backpressure% column)")
        else:
            print("no intermediate below 98% of plan — the shortfall sits at the target stage itself "
                  "(machine count, inserter/belt capacity, or its own feed).")
    print()

# --- machine ranking ---------------------------------------------------------

def classify_shape(deltas, statuses):
    """Coarse decay-shape classifier over one machine's crafts_delta series.
    See docs/sim-harness-forensics.md "Reading time-series decay shapes"
    for the real vocabulary this approximates. Rules, in order:
      flat-zero:       every window's crafts_delta is ~0.
      ramp-then-decay: a rise to a peak strictly before the final window,
                       then the final window under half that peak.
      decaying:        peak in the FIRST window, final window under half it
                       (winding down, never ramped — not a jam signature).
      stable-below:    low relative variation (stdev/mean < 20%) with at
                       least one window in a non-`working` status.
      unsteady:        any other series with a non-`working` status somewhere.
      healthy:         never left `working`, and none of the above.
    `healthy` is never returned for a machine that sat in an impaired
    status: an alternating starving machine is `unsteady`, not healthy.
    """
    if not deltas:
        return "unknown"
    if all(abs(d) < 1e-9 for d in deltas):
        return "flat-zero"
    impaired = any(s != "working" for s in statuses)
    peak = max(deltas)
    peak_idx = deltas.index(peak)
    if peak > 0 and peak_idx < len(deltas) - 1 and deltas[-1] < 0.5 * peak:
        return "ramp-then-decay" if peak_idx > 0 else "decaying"
    mean = sum(deltas) / len(deltas)
    if mean <= 0:
        return "unknown"
    cv = (sum((d - mean) ** 2 for d in deltas) / len(deltas)) ** 0.5 / mean
    if cv < 0.2:
        return "stable-below" if impaired else "healthy"
    return "unsteady" if impaired else "healthy"

def rank_from_timeseries(timeseries):
    by_unit = {}
    for point in timeseries:
        for m in point.get("machines", []):
            unit = m.get("unit")
            if unit is None:
                continue
            rec = by_unit.setdefault(unit, {"name": m.get("name"), "x": None, "y": None, "deltas": [], "statuses": []})
            rec["x"], rec["y"] = m.get("x", rec["x"]), m.get("y", rec["y"])
            rec["deltas"].append(m.get("crafts_delta", 0.0))
            rec["statuses"].append(m.get("status", "?"))
    rows = []
    for unit, rec in by_unit.items():
        n = len(rec["statuses"]) or 1
        frac_s = sum(1 for s in rec["statuses"] if s in SHORTAGE) / n
        frac_b = sum(1 for s in rec["statuses"] if s in BACKPRESSURE) / n
        frac_o = sum(1 for s in rec["statuses"] if s != "working" and s not in SHORTAGE and s not in BACKPRESSURE) / n
        mean_delta = sum(rec["deltas"]) / len(rec["deltas"]) if rec["deltas"] else 0.0
        rows.append({
            "unit": unit, "name": rec["name"], "x": rec["x"], "y": rec["y"],
            "frac_shortage": frac_s, "frac_backpressure": frac_b, "frac_other": frac_o,
            "mean_crafts_delta": mean_delta, "shape": classify_shape(rec["deltas"], rec["statuses"]),
        })
    rows.sort(key=rank_key)
    return rows

def rank_from_final_frame(sim_state):
    """Fallback when no timeseries: a single frame can't distinguish
    transient from persistent — the caller prints that caveat."""
    rows = []
    for m in sim_state.get("machines", []):
        x, y, name, status = m[0], m[1], m[2], m[3]
        if status == "working":
            continue
        rows.append({
            "unit": None, "name": name, "x": x, "y": y,
            "frac_shortage": 1.0 if status in SHORTAGE else 0.0,
            "frac_backpressure": 1.0 if status in BACKPRESSURE else 0.0,
            "frac_other": 1.0 if (status not in SHORTAGE and status not in BACKPRESSURE) else 0.0,
            "mean_crafts_delta": None, "shape": "unknown (single frame)", "status": status,
        })
    rows.sort(key=rank_key)
    return rows

def print_ranking(top, top_n, table_n=20):
    report = top.get("report", {})
    timeseries = report.get("timeseries") or []
    print("--- machine ranking ---")
    if timeseries:
        rows = rank_from_timeseries(timeseries)
        total = len(rows)
        rows = [r for r in rows if r["frac_shortage"] > 0 or r["frac_backpressure"] > 0
                or r["frac_other"] > 0 or r["shape"] != "healthy"]
        print(f"({len(timeseries)} checkpoint window(s); {total - len(rows)} healthy machine(s) omitted)")
    else:
        print("timeseries: absent — falling back to final frame (cannot distinguish transient from persistent)")
        rows = rank_from_final_frame(top.get("sim_state") or {})

    if not rows:
        print("no starved/backpressured machines found.\n")
        return [], rows
    print(f"{'unit':>6} {'name':<26} {'pos':>10} {'shortage%':>10} {'backpressure%':>14} {'mean crafts':>12} {'shape':<16}")
    for r in rows[:table_n]:
        pos = f"({r['x']:.0f},{r['y']:.0f})" if r["x"] is not None and r["y"] is not None else "(?,?)"
        mean = "-" if r["mean_crafts_delta"] is None else f"{r['mean_crafts_delta']:.2f}"
        print(f"{str(r['unit'] or '-'):>6} {str(r['name'] or '?'):<26} {pos:>10} "
              f"{r['frac_shortage']*100:>9.0f}% {r['frac_backpressure']*100:>13.0f}% {mean:>12} {r['shape']:<16}")
    if len(rows) > table_n:
        print(f"... and {len(rows) - table_n} more (raise --top to see them)")
    print()
    return rows[:top_n], rows

# --- map ----------------------------------------------------------------

def parse_belt(b):
    det = b[3] if len(b) > 3 and isinstance(b[3], list) else None
    return {
        "x": b[0], "y": b[1], "n": b[2] if len(b) > 2 else None, "det": det,
        "name": b[4] if len(b) > 4 else None, "direction": b[5] if len(b) > 5 else None,
        "ug_type": b[6] if len(b) > 6 else None,
    }

def belt_glyph(belt):
    nonempty = (belt["n"] or 0) > 0
    if belt["direction"] is None:
        return "#" if nonempty else "."  # old-format dump: no name/direction at all
    if belt["ug_type"] == "input":
        return "U" if nonempty else "u"
    if belt["ug_type"] == "output":
        return "O" if nonempty else "o"
    if "splitter" in (belt["name"] or ""):
        return "Y" if nonempty else "y"
    return (arrow_for(belt["direction"]) or "#") if nonempty else "."

def build_grid(sim_state, window):
    def in_window(x, y):
        if window is None:
            return True
        xmin, xmax, ymin, ymax = window
        return xmin <= x <= xmax and ymin <= y <= ymax

    grid, all_xy = {}, []
    for p in sim_state.get("pipes") or []:
        x, y = p[0], p[1]
        if in_window(x, y):
            grid[(x, y)] = "~" if (len(p) > 4 and p[4]) else "-"
            all_xy.append((x, y))
    for b in sim_state.get("belts") or []:
        belt = parse_belt(b)
        if in_window(belt["x"], belt["y"]):
            grid[(belt["x"], belt["y"])] = belt_glyph(belt)
            all_xy.append((belt["x"], belt["y"]))
    for i in sim_state.get("inserters") or []:
        x, y, status = i[0], i[1], i[2]
        if in_window(x, y):
            grid[(x, y)] = INSERTER_GLYPH.get(status, "?")
            all_xy.append((x, y))
    for m in sim_state.get("machines") or []:  # drawn last: highest priority
        x, y, status = m[0], m[1], m[3]
        if in_window(x, y):
            grid[(x, y)] = machine_glyph(status)
            all_xy.append((x, y))
    return grid, all_xy

def render_map(grid, all_xy, window):
    if window is not None:
        xmin, xmax, ymin, ymax = window
    elif all_xy:
        xs, ys = [p[0] for p in all_xy], [p[1] for p in all_xy]
        xmin, xmax, ymin, ymax = min(xs), max(xs), min(ys), max(ys)
    else:
        print("(nothing to render in this window)")
        return
    xmin, xmax, ymin, ymax = int(xmin), int(xmax), int(ymin), int(ymax)
    margin, width = 6, xmax - xmin + 1

    header = [" "] * (margin + width + 6)  # slack so a label at the right edge isn't truncated
    for x in range(xmin, xmax + 1):
        if x % 10 == 0:
            start = margin + (x - xmin)
            for k, ch in enumerate(str(x)):
                if start + k < len(header):
                    header[start + k] = ch
    print("".join(header).rstrip())
    for y in range(ymin, ymax + 1):
        row_label = f"{y:>5} " if y % 10 == 0 else " " * 6
        print(row_label + "".join(grid.get((x, y), " ") for x in range(xmin, xmax + 1)))

def parse_around(s):
    """`X,Y[,R]` → (x, y, r) or None (with a stderr note) on malformed input."""
    try:
        parts = [int(p) for p in s.split(",")]
        if len(parts) not in (2, 3):
            raise ValueError(len(parts))
        return parts[0], parts[1], (parts[2] if len(parts) > 2 else 15)
    except (ValueError, IndexError):
        print(f"note: --around expects X,Y[,R] (got {s!r}) — rendering the full extent instead", file=sys.stderr)
        return None

def resolve_window(args, worst_rows, around=None):
    if around is not None:
        x, y, r = around
        return (x - r, x + r, y - r, y + r)
    if args.top_explicit and worst_rows:
        xs = [r["x"] for r in worst_rows if r["x"] is not None]
        ys = [r["y"] for r in worst_rows if r["y"] is not None]
        if xs and ys:
            pad = 10
            return (min(xs) - pad, max(xs) + pad, min(ys) - pad, max(ys) + pad)
    return None

LEGEND = """legend:
  machines (anchor tile only, entity is 3x3): W working  S ingredient shortage  F full_output  P no power  ? other
  inserters: i working  s waiting_for_source  d waiting_for_space  m waiting_for_more  ? other
  belts (direction known): ^ > v < = nonempty, moving that way; . = empty
  belts (old format, no direction): # nonempty, . empty
  underground: U/u input mouth (nonempty/empty)  O/o output mouth (nonempty/empty) — direction not shown here (lane detail prints it)
  splitters: Y nonempty, y empty     pipes: ~ has fluid, - empty
  NOTE: belt `n` is a whole TRANSPORT LINE's count (repeats across a straight run) — treated as
  empty-vs-nonempty only, never per-tile.
"""

# --- lane detail --------------------------------------------------------

def lane_str(det, idx):
    if not det or len(det) <= idx or not det[idx]:
        return "—"
    parts = []
    for entry in det[idx]:
        if isinstance(entry, (list, tuple)) and len(entry) == 2:
            parts.append(f"{entry[0]}×{entry[1]}")
        else:
            parts.append(f"?{entry!r}")  # unexpected lane entry shape: show it, don't crash
    return "  ".join(parts)

def print_lane_detail(sim_state, anchors, radius, label=None):
    """`anchors` are ranking rows (machines) or a `{"kind": "tile", ...}` from --around."""
    print(f"--- lane detail ({label or f'worst {len(anchors)}'}, radius {radius}) ---")
    if not anchors:
        print("(nothing to detail)")
        return
    belts = [parse_belt(b) for b in sim_state.get("belts") or []]
    for r in anchors:
        x0, y0 = r["x"], r["y"]
        if x0 is None or y0 is None:
            continue
        if r.get("kind") == "tile":
            print(f"tile ({x0:.0f},{y0:.0f}):")
        else:
            print(f"machine {r.get('unit') or '-'} {r.get('name') or '?'} at ({x0:.0f},{y0:.0f}):")
        nearby = sorted(
            (b for b in belts if abs(b["x"] - x0) <= radius and abs(b["y"] - y0) <= radius),
            key=lambda b: (abs(b["x"] - x0) + abs(b["y"] - y0), b["x"], b["y"]),
        )
        if not nearby:
            print("  (no belts within radius)")
            continue
        for b in nearby:
            arrow = arrow_for(b["direction"]) if b["direction"] is not None else "?"
            name = b["name"] or "belt"
            if b["det"] is not None:  # `[]`/`[{}, {}]` is a real, empty new-format belt — the headline case
                print(f"  ({b['x']:.0f},{b['y']:.0f}) {name} {arrow} L1: {lane_str(b['det'], 0)}  L2: {lane_str(b['det'], 1)}")
            else:
                fill = "nonempty" if (b["n"] or 0) > 0 else "empty"
                print(f"  ({b['x']:.0f},{b['y']:.0f}) {name} {arrow} n={b['n']} ({fill}, no per-lane detail)")
        print()

# --- main -----------------------------------------------------------------

def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    # argparse reads `--around -5,-3` as an unknown option `-5,-3`. Layout
    # coordinates are routinely negative, so glue a leading-minus value onto
    # the flag (`--around=-5,-3`), which argparse accepts.
    for i in range(len(argv) - 1):
        if argv[i] == "--around" and argv[i + 1].startswith("-"):
            argv[i:i + 2] = [f"--around={argv[i + 1]}"]
            break
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("report", help="path to a sim-harness `run --out` JSON file")
    ap.add_argument("--top", type=int, default=None, help="worst-N machines to table/detail (default: table 20, detail 3)")
    ap.add_argument("--radius", type=int, default=None, help="lane-detail radius (default 3, or --around's R)")
    ap.add_argument("--around", default=None, help="X,Y[,R] — window map + lane detail on this tile (negative: --around=-5,-3)")
    args = ap.parse_args(argv)
    args.top_explicit = args.top is not None
    top_n = args.top if args.top is not None else 3
    table_n = args.top if args.top is not None else 20
    around = parse_around(args.around) if args.around is not None else None
    radius = args.radius if args.radius is not None else (around[2] if around is not None else 3)
    if radius < 0:
        print(f"note: --radius {radius} is negative — using 0", file=sys.stderr)
        radius = 0

    try:
        top = load_report(args.report)
    except (OSError, json.JSONDecodeError) as e:
        print(f"error: could not load {args.report}: {e}", file=sys.stderr)
        return 1

    print_header(top)
    worst_rows, _all_rows = print_ranking(top, top_n, table_n)

    sim_state = top.get("sim_state") or {}
    window = resolve_window(args, worst_rows, around)
    print("--- map ---")
    print(LEGEND)
    grid, all_xy = build_grid(sim_state, window)
    render_map(grid, all_xy, window)
    print()

    if around is not None:
        x, y, _r = around
        print_lane_detail(sim_state, [{"kind": "tile", "x": x, "y": y}], radius, label=f"around ({x},{y})")
    else:
        print_lane_detail(sim_state, worst_rows, radius)
    return 0

if __name__ == "__main__":
    sys.exit(main())
