"""Ground-truth + orientation-transform verification for machine fluid ports.

Provenance for the `fluid_ports` shared tables (RFC `docs/rfc-power-supply.md`
Phase 0e-i). Extends `verify_fluid_ports_emag_cryo.py` (which only dumps the
NORTH/unmirrored base) with the two orientation transforms the layout engine
uses to bring a machine's fluid inputs onto the template's north face:

  * mirror (y-flip): the oil-refinery precedent — used for foundry + cryo.
  * East rotation (direction=4, +90 deg): used for electromagnetic-plant.

Both transforms are the *authoritative Factorio* transforms as implemented by
draftsman:
  * rotation is `draftsman.utils.rotate_point(p, radians(dir*22.5))`, which for
    direction=4 (East) is (x,y) -> (-y, x) and maps North(0,-1) -> East(1,0);
  * the mirror table is validated by reproducing the in-game-validated
    oil-refinery OIL_MIRROR (inputs move to the north face) exactly.

For foundry + cryo the mirror (y-flip) and a 180-degree rotation produce
IDENTICAL port tiles (the ports are x-symmetric), so the mirror table is
unambiguous regardless of the physical mirror axis. This script prints that
equivalence as a cross-check.

Run: python3 scripts/verify_fluid_ports_transforms.py
"""
import math
from draftsman.data import entities

DIR_NAME = {0: "N", 4: "E", 8: "S", 12: "W"}
DIR_OFF = {0: (0, -1), 4: (1, 0), 8: (0, 1), 12: (-1, 0)}


def base_ports(name):
    """(production_type, pos_x, pos_y, dir, cx, cy) per fluid box, center-relative."""
    raw = entities.raw[name]
    (minx, miny), (maxx, maxy) = raw["selection_box"]
    w, h = round(maxx - minx), round(maxy - miny)
    cx, cy = w / 2.0, h / 2.0
    out = []
    for fb in raw.get("fluid_boxes", []):
        pt = fb.get("production_type", "?")
        for pc in fb.get("pipe_connections", []):
            pos = pc["position"]
            out.append((pt, pos[0], pos[1], pc.get("direction"), cx, cy))
    return out


def pipe_tile(cx, cy, px, py, d):
    ptx, pty = math.floor(cx + px), math.floor(cy + py)
    ox, oy = DIR_OFF[d]
    return (ptx + ox, pty + oy)


def transform(px, py, d, kind):
    if kind == "base":
        return px, py, d
    if kind == "east":      # direction=4, +90 CW: (x,y)->(-y,x)
        return -py, px, (d + 4) % 16
    if kind == "south":     # direction=8, 180 deg
        return -px, -py, (d + 8) % 16
    if kind == "mirror":    # oil-refinery precedent: y-flip, N<->S faces swap
        return px, -py, ((d + 8) % 16 if d in (0, 8) else d)
    raise ValueError(kind)


def table(name, kind):
    ins, outs = [], []
    for pt, px, py, d, cx, cy in base_ports(name):
        tx, ty, td = transform(px, py, d, kind)
        pipe = pipe_tile(cx, cy, tx, ty, td)
        (ins if pt == "input" else outs).append(pipe)
    return sorted(ins), sorted(outs)


def show(name, kind):
    ins, outs = table(name, kind)
    print(f"  {name:22} {kind:7} input={ins}  output={outs}")


print("== base (must match Rust fluid_ports base tables) ==")
for n in ["assembling-machine-2", "chemical-plant", "oil-refinery",
          "foundry", "cryogenic-plant", "electromagnetic-plant"]:
    show(n, "base")

print("\n== oil-refinery mirror reproduces in-game OIL_MIRROR ==")
show("oil-refinery", "mirror")   # expect input=[(1,-1),(3,-1)] output=[(0,5),(2,5),(4,5)]

print("\n== foundry / cryo mirror (== 180 rotation, x-symmetric) ==")
for n in ["foundry", "cryogenic-plant"]:
    show(n, "mirror")
    show(n, "south")
    assert table(n, "mirror") == table(n, "south"), f"{n}: mirror != south (not x-symmetric!)"

print("\n== electromagnetic-plant East rotation (input reaches north face) ==")
show("electromagnetic-plant", "east")  # expect a north input at (1,-1)
