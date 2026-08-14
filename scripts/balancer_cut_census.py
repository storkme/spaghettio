#!/usr/bin/env python3
"""Per-row-cut south-flow capacity census for balancer library templates.

The structural waist check from #631, in script form (PR #630 review round
8 committed it so the RFC-027 cut claims are reproducible): a balancer
whose min row-cut capacity is below its rated output count is
throughput-capped no matter what classify's max-flow or the lane walker
say — both are blind to the waist class.

Flow across the cut between row y and y+1 is carried by:
  - a surface transport-belt at row y facing south (1 tile)
  - a splitter at row y facing south (2 tiles: x and x+1)
  - an underground-belt pair (input at y1, output at y2, same x, facing
    south): 1 tile across every cut y1..y2 INCLUSIVE — the item travels
    underground across cuts y1..y2-1, then the output tile itself emits
    it southward across cut y2. Counting only y1..y2-1 (or counting
    surface belts alone) reads false waists on clean templates.

North-facing entities are return/feedback paths; they are flagged for
visibility but do not reduce forward cut capacity.

Usage: python3 scripts/balancer_cut_census.py [TEMPLATE ...]
  e.g. python3 scripts/balancer_cut_census.py T_6_3 T_6_4
  With no arguments, censuses every T_<n>_<m>_ENTITIES in the library.

Rated throughput of an (n, m) balancer is min(n, m) belts — a (1, 4)
delivering one belt is at rated, not waisted. Exit code 1 if any censused
template's min cut is below min(n, m).

The instrument is one-sided, like the meter: "min cut < rated" is a
structural throughput cap — believe it. "min cut >= rated" clears
nothing on its own (lateral distribution can still under-use a cut);
clearing takes the sim.
"""
import re
import sys

SRC = "crates/core/src/bus/balancer_library.rs"

ENT = re.compile(
    r'BalancerTemplateEntity \{ name: "([^"]+)", x: (-?\d+), y: (-?\d+), '
    r'direction: (\d+), io_type: (None|Some\("(?:input|output)"\))'
)
ARRAY = re.compile(r"static (T_(\d+)_(\d+)_ENTITIES): ")
REG = re.compile(
    r"m\.insert\(\((\d+), (\d+)\), BalancerTemplate \{\s*"
    r"n_inputs: (\d+), n_outputs: (\d+), width: \d+, height: (\d+)",
    re.S,
)


def parse_arrays(src):
    arrays = {}
    for m in ARRAY.finditer(src):
        start = m.start()
        end = src.index("];", start)
        ents = []
        for e in ENT.finditer(src[start:end]):
            io = "input" if "input" in e.group(5) else (
                "output" if "output" in e.group(5) else None)
            ents.append((e.group(1), int(e.group(2)), int(e.group(3)),
                         int(e.group(4)), io))
        arrays[(int(m.group(2)), int(m.group(3)))] = ents
    return arrays


def parse_registrations(src):
    return {(int(m.group(1)), int(m.group(2))):
            (int(m.group(3)), int(m.group(4)), int(m.group(5)))
            for m in REG.finditer(src)}


def census(ents, height):
    cuts = [0] * (height - 1)
    ug_outputs = [(x, y) for (n, x, y, d, io) in ents
                  if n == "underground-belt" and io == "output" and d == 4]
    notes = []
    north = [(n, x, y) for (n, x, y, d, io) in ents if d == 0]
    if north:
        notes.append(f"north-facing return paths: {len(north)}")
    for (n, x, y, d, io) in ents:
        if d != 4:
            continue
        if n == "transport-belt":
            if y < height - 1:
                cuts[y] += 1
        elif n == "splitter":
            if y < height - 1:
                cuts[y] += 2
        elif n == "underground-belt" and io == "input":
            below = sorted(oy for (ox, oy) in ug_outputs if ox == x and oy > y)
            if not below:
                notes.append(f"unpaired south UG input at ({x},{y})")
                continue
            for c in range(y, min(below[0] + 1, height - 1)):
                cuts[c] += 1
    return cuts, notes


def main():
    src = open(SRC).read()
    arrays = parse_arrays(src)
    regs = parse_registrations(src)
    wanted = None
    if len(sys.argv) > 1:
        wanted = set()
        for a in sys.argv[1:]:
            m = re.match(r"T_(\d+)_(\d+)", a)
            if not m:
                sys.exit(f"unrecognised template name {a!r} (expected T_n_m)")
            wanted.add((int(m.group(1)), int(m.group(2))))
    failed = []
    for shape in sorted(arrays):
        if wanted is not None and shape not in wanted:
            continue
        if shape not in regs:
            continue
        n_inputs, n_outputs, height = regs[shape]
        rated = min(n_inputs, n_outputs)
        cuts, notes = census(arrays[shape], height)
        verdict = "ok" if min(cuts) >= rated else "WAIST"
        print(f"({shape[0]},{shape[1]}): min cut {min(cuts)} vs rated "
              f"{rated} [{verdict}]  cuts={cuts}"
              + (f"  ({'; '.join(notes)})" if notes else ""))
        if min(cuts) < rated:
            failed.append(shape)
    if failed:
        print(f"\nWAIST-CAPPED: {failed}")
        sys.exit(1)


if __name__ == "__main__":
    main()
