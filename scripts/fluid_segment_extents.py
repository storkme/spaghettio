#!/usr/bin/env python3
"""Compute real fluid SEGMENTS (connected components) from a sim-state dump
and check each against the F10 320x320 limit.

Grouping pipes by fluid NAME is not good enough: K-replicated layouts have
several independent networks carrying the same fluid, and merging them
overstates extent. This builds components from actual adjacency.

Rules honoured:
  F1  a `pipe` connects to all four neighbours
  F5  a `pipe-to-ground` has ONE surface side + one underground side
  F5a a PTG's perpendicular sides do NOT connect
  F4  PTGs pair underground, opposite facing, same axis, gap <= 9
"""
import json, sys, collections

# Blueprint-JSON direction (what the dump carries): 0=N,4=E,8=S,12=W.
DELTA = {0: (0, -1), 4: (1, 0), 8: (0, 1), 12: (-1, 0)}


def load(path):
    r = json.load(open(path))
    return (r.get("sim_state") or {}).get("pipes", [])


def build(pipes):
    at = {}
    for p in pipes:
        x, y, name, direction, fluids = p[0], p[1], p[2], p[3], p[4] or []
        at[(x, y)] = {"name": name, "dir": direction, "fluids": fluids}

    adj = collections.defaultdict(set)

    def link(a, b):
        adj[a].add(b)
        adj[b].add(a)

    for pos, e in at.items():
        x, y = pos
        if e["name"] == "pipe-to-ground":
            # F5: surface opening is the GAME direction (dump is game JSON).
            d = DELTA.get(e["dir"])
            if d:
                s = (x + d[0], y + d[1])
                if s in at:
                    link(pos, s)  # F5a: only this side; perpendicular excluded
        else:
            for dx, dy in ((0, -1), (1, 0), (0, 1), (-1, 0)):
                n = (x + dx, y + dy)
                if n in at:
                    link(pos, n)

    # F4: underground pairing — opposite facing, same axis, gap <= 9.
    ptg = [(p, e) for p, e in at.items() if e["name"] == "pipe-to-ground"]
    for pos, e in ptg:
        d = DELTA.get(e["dir"])
        if not d:
            continue
        # underground side is opposite the surface opening
        ux, uy = -d[0], -d[1]
        for gap in range(1, 11):  # entity-to-entity <= 10 => gap <= 9
            cand = (pos[0] + ux * gap, pos[1] + uy * gap)
            ce = at.get(cand)
            if not ce:
                continue
            if ce["name"] != "pipe-to-ground":
                break  # something else occupies the axis
            cd = DELTA.get(ce["dir"])
            if cd and (cd[0], cd[1]) == (d[0], d[1]):
                continue  # same facing: not a pair, keep looking
            link(pos, cand)
            break
    return at, adj


def components(at, adj):
    seen, comps = set(), []
    for start in at:
        if start in seen:
            continue
        stack, comp = [start], []
        seen.add(start)
        while stack:
            cur = stack.pop()
            comp.append(cur)
            for n in adj[cur]:
                if n not in seen:
                    seen.add(n)
                    stack.append(n)
        comps.append(comp)
    return comps


def main(path):
    pipes = load(path)
    at, adj = build(pipes)
    comps = components(at, adj)
    print(f"{path}\n  pipe-class entities: {len(at)}   segments: {len(comps)}\n")
    over = []
    rows = []
    for c in comps:
        xs = [p[0] for p in c]
        ys = [p[1] for p in c]
        dx, dy = max(xs) - min(xs), max(ys) - min(ys)
        fl = sorted({f[0] for p in c for f in at[p]["fluids"]})
        rows.append((len(c), dx, dy, fl, min(xs), max(xs)))
        if dx > 320 or dy > 320:
            over.append((len(c), dx, dy, fl))
    rows.sort(reverse=True)
    print(f"  {'tiles':>6} {'dx':>6} {'dy':>5}  fluids")
    for n, dx, dy, fl, x0, x1 in rows[:14]:
        flag = "   <-- EXCEEDS 320 (F10: does not flow)" if dx > 320 or dy > 320 else ""
        print(f"  {n:>6} {dx:>6} {dy:>5}  {','.join(fl) or '(empty)'}{flag}")
    print(f"\n  segments over the F10 limit: {len(over)} of {len(comps)}")
    multi = [c for c in comps if len({f[0] for p in c for f in at[p]['fluids']}) > 1]
    print(f"  segments carrying >1 fluid (F3 violation): {len(multi)}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "/tmp/mega-chain-usp2raw-long.json")
