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


def openings(name, direction):
    """Which orthogonal deltas this entity can connect on, at the surface.

    F1: a plain pipe opens on all four sides.
    F5/F5a: a pipe-to-ground opens on ONE surface side only — the side its
    (game-convention) direction points at. Its back and both perpendicular
    sides are CLOSED, which is what keeps stacked multi-fluid trunk rows
    isolated even though their tiles touch.
    """
    if name == "pipe-to-ground":
        d = DELTA.get(direction)
        return {d} if d else set()
    return {(0, -1), (1, 0), (0, 1), (-1, 0)}


def build(pipes):
    at = {}
    for p in pipes:
        x, y, name, direction, fluids = p[0], p[1], p[2], p[3], p[4] or []
        at[(x, y)] = {"name": name, "dir": direction, "fluids": fluids}

    adj = collections.defaultdict(set)

    def link(a, b):
        adj[a].add(b)
        adj[b].add(a)

    # Surface links, MUTUALLY consented. Both entities must open toward each
    # other: `link` is symmetric, so testing only one end would let a plain
    # pipe re-add a connection through a PTG's closed side and silently merge
    # two independent segments.
    for pos, e in at.items():
        opens = openings(e["name"], e["dir"])
        for d in opens:
            n = (pos[0] + d[0], pos[1] + d[1])
            ne = at.get(n)
            if not ne:
                continue
            back = (-d[0], -d[1])
            if back in openings(ne["name"], ne["dir"]):
                link(pos, n)

    # F4: underground pairing — opposite facing, same axis, entity-to-entity
    # distance <= 10 (gap <= 9).
    for pos, e in list(at.items()):
        if e["name"] != "pipe-to-ground":
            continue
        d = DELTA.get(e["dir"])
        if not d:
            continue
        ux, uy = -d[0], -d[1]  # underground side is opposite the surface mouth
        for gap in range(1, 11):
            cand = (pos[0] + ux * gap, pos[1] + uy * gap)
            ce = at.get(cand)
            if not ce:
                continue
            if ce["name"] != "pipe-to-ground":
                # Underground runs BENEATH surface entities — a plain pipe on
                # the axis does not sever the pair, so keep searching.
                continue
            cd = DELTA.get(ce["dir"])
            if cd != (-d[0], -d[1]):
                # Not opposite-facing: not our partner. Keep looking rather
                # than breaking, or a perpendicular PTG hides the real one.
                continue
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
        # Tile EXTENT, not coordinate span: x=0..320 is 321 tiles.
        dx, dy = max(xs) - min(xs) + 1, max(ys) - min(ys) + 1
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
