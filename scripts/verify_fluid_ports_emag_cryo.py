"""Verify electromagnetic-plant / cryogenic-plant fluid port geometry from
draftsman (authoritative Factorio 2.0 data), north-facing, no mirror.

Prints, for each entity, the tile where an adjacent pipe must sit to connect
to each port, relative to the entity's TOP-LEFT footprint tile, plus the
physical face (N/E/S/W). This is the ground truth the Rust fluid_ports table
must match.
"""
from draftsman.data import entities

# Factorio 16-direction encoding used in the machine data.
DIR_NAME = {0: "N", 4: "E", 8: "S", 12: "W"}
# Unit offset (dx,dy) for the pipe that sits just outside a port facing `dir`.
DIR_OFF = {0: (0, -1), 4: (1, 0), 8: (0, 1), 12: (0, 1) if False else (-1, 0)}
# (dir 12 = West -> pipe one tile west)


def dump(name):
    raw = entities.raw[name]
    sb = raw["selection_box"]
    # selection_box is [[minx,miny],[maxx,maxy]] in tiles relative to center.
    (minx, miny), (maxx, maxy) = sb
    width = round(maxx - minx)
    height = round(maxy - miny)
    # Entity center in continuous coords, measured from the top-left tile
    # (top-left tile spans [0,1)x[0,1)); center is at (width/2, height/2).
    cx, cy = width / 2.0, height / 2.0
    print(f"\n=== {name}  footprint {width}x{height}  center=({cx},{cy}) ===")
    for fb in raw.get("fluid_boxes", []):
        pt = fb.get("production_type", "?")
        for pc in fb.get("pipe_connections", []):
            pos = pc["position"]
            d = pc.get("direction", None)
            # Port tile (machine side) containing the connection point.
            import math
            port_tx = math.floor(cx + pos[0])
            port_ty = math.floor(cy + pos[1])
            ox, oy = DIR_OFF[d]
            pipe_tx, pipe_ty = port_tx + ox, port_ty + oy
            print(f"  {pt:6} pos={pos} dir={d}({DIR_NAME.get(d,'?')})"
                  f" -> port_tile=({port_tx},{port_ty}) pipe_tile=({pipe_tx},{pipe_ty})")


for n in ["electromagnetic-plant", "cryogenic-plant", "foundry", "oil-refinery",
          "chemical-plant", "assembling-machine-2"]:
    try:
        dump(n)
    except KeyError:
        print(f"\n=== {n}: NOT in draftsman ===")
