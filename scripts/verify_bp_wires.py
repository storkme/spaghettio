"""Load a generated blueprint string with draftsman and verify the pole
copper `wires` are present, well-formed, and connect every pole into one
electric network — the strongest pre-in-game-paste check."""

import sys
from draftsman.blueprintable import get_blueprintable_from_string

path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/claude-1000/ec60.bp"
bp_str = open(path).read().strip()

bp = get_blueprintable_from_string(bp_str)  # draftsman decodes + validates
d = bp.to_dict()["blueprint"]
ents = d["entities"]
wires = d.get("wires", [])

POLE = {"medium-electric-pole", "small-electric-pole", "substation", "big-electric-pole"}
# entity_number -> (name, is_pole). draftsman numbers sequentially from 1.
num_name = {i + 1: e["name"] for i, e in enumerate(ents)}
pole_nums = {n for n, name in num_name.items() if name in POLE}
print(f"draftsman parsed OK: {len(ents)} entities, {len(pole_nums)} poles, {len(wires)} wires")

assert wires, "no wires array in blueprint!"

# Every wire is pole-copper [a,5,b,5] between two poles.
adj = {n: set() for n in pole_nums}
for w in wires:
    assert len(w) == 4, f"malformed wire {w}"
    a, ca, b, cb = w
    assert ca == 5 and cb == 5, f"non-copper connector in {w}"
    assert a in pole_nums and b in pole_nums, f"wire endpoint not a pole: {w}"
    adj[a].add(b)
    adj[b].add(a)

# Connectivity: BFS from the first pole must reach every pole.
start = min(pole_nums)
seen = {start}
stack = [start]
while stack:
    x = stack.pop()
    for y in adj[x]:
        if y not in seen:
            seen.add(y)
            stack.append(y)

missing = pole_nums - seen
assert not missing, f"{len(missing)} poles NOT connected: {sorted(missing)[:10]}..."
print(f"OK: all {len(pole_nums)} poles form ONE connected copper network (BFS from #{start}).")
