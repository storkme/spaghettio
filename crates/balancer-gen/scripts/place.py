#!/usr/bin/env -S uv run --no-project --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["ortools>=9.10"]
# ///
"""Phase 3.2A — flow-conservation belt routing for fixed-direction
splitters.

Two modes, selected by the input JSON:

**Mode A — splitter placement only** (phase 3.1 spike).
Input has `n_splitters` and `bounds` only. CP-SAT places splitters with
no-overlap and reports positions. Belt routing not solved.

**Mode B — belt routing with given splitter layout** (phase 3.2A.1).
Input has `splitter_positions`, `input_port_tiles`, `output_port_tiles`,
`edges` (with source/dest slot assignments), and `bounds`. CP-SAT
encodes belt routing as flow conservation per topology edge:

  - For each cell `c`, each topology edge `e`, each direction `d`:
    bool `arc[c, d, e]` — is there flow of edge e leaving cell c
    heading direction d?
  - Each non-splitter cell can host at most one edge's arc (no shared
    belts).
  - Splitter cells can host arcs for multiple edges (the splitter is
    the merge/split point).
  - Conservation per (cell, edge):
      outflow - inflow = 1 if cell is edge's source,
      inflow - outflow = 1 if cell is edge's dest,
      0 otherwise.
  - Source/dest cells are determined from caller-provided splitter
    positions + per-edge slot assignments. South-facing splitters only:
    anchor at (sx, sy), second at (sx+1, sy); inputs come from
    (sx, sy-1)/(sx+1, sy-1), outputs go to (sx, sy+1)/(sx+1, sy+1).
    For source=Splitter the source cell IS the splitter tile (anchor or
    second per slot); for dest=Splitter the dest cell IS the splitter
    tile (anchor or second per slot).

Output: belt list — cells with edge flow that are NOT splitter tiles.
Each belt has the direction of its outgoing flow.

Future phases:
- 3.2B: underground belts (let crossings happen).
- 3.2C: per-splitter direction variable (currently fixed south).
- 3.2D: bounding-box minimisation (currently caller-provided).
"""

import json
import sys
import time

from ortools.sat.python import cp_model

# Internal direction encoding: 0=N, 1=E, 2=S, 3=W. Translated to
# Factorio's 0/2/4/6 on output.
DIR_STEPS = [(0, -1), (1, 0), (0, 1), (-1, 0)]
INTERNAL_TO_FACTORIO_DIR = [0, 2, 4, 6]


def solve_overlap_only(req: dict) -> dict:
    """Original phase 3.1 spike — splitter no-overlap only."""
    n_splitters = req["n_splitters"]
    width, height = req["bounds"]
    model = cp_model.CpModel()
    xs = [model.NewIntVar(0, width - 2, f"x{i}") for i in range(n_splitters)]
    y_lo = 1
    y_hi = max(y_lo, height - 2)
    ys = [model.NewIntVar(y_lo, y_hi, f"y{i}") for i in range(n_splitters)]
    x_intervals = [
        model.NewFixedSizeIntervalVar(xs[i], 2, f"xi{i}") for i in range(n_splitters)
    ]
    y_intervals = [
        model.NewFixedSizeIntervalVar(ys[i], 1, f"yi{i}") for i in range(n_splitters)
    ]
    model.AddNoOverlap2D(x_intervals, y_intervals)

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 30.0
    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    out = {"status": solver.StatusName(status), "elapsed_s": elapsed}
    if status in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        out["splitters"] = [
            {"x": solver.Value(xs[i]), "y": solver.Value(ys[i])}
            for i in range(n_splitters)
        ]
    return out


def solve_routing(req: dict) -> dict:
    """Phase 3.2A.1 — belt routing with given splitter positions."""
    width, height = req["bounds"]
    splitter_positions = req["splitter_positions"]
    input_port_tiles = [tuple(t) for t in req["input_port_tiles"]]
    output_port_tiles = [tuple(t) for t in req["output_port_tiles"]]
    edges = req["edges"]

    # Splitter cells (south-facing: anchor + second to the east).
    splitter_cells: set[tuple[int, int]] = set()
    for sp in splitter_positions:
        splitter_cells.add((sp["x"], sp["y"]))
        splitter_cells.add((sp["x"] + 1, sp["y"]))

    def src_cell(edge: dict) -> tuple[int, int]:
        if edge["src_kind"] == "InputPort":
            return input_port_tiles[edge["src_idx"]]
        sp = splitter_positions[edge["src_idx"]]
        slot = edge.get("src_slot", 0)
        return (sp["x"] + slot, sp["y"])

    def dst_cell(edge: dict) -> tuple[int, int]:
        if edge["dst_kind"] == "OutputPort":
            return output_port_tiles[edge["dst_idx"]]
        sp = splitter_positions[edge["dst_idx"]]
        slot = edge.get("dst_slot", 0)
        return (sp["x"] + slot, sp["y"])

    edge_src = [src_cell(e) for e in edges]
    edge_dst = [dst_cell(e) for e in edges]

    model = cp_model.CpModel()
    arcs: dict[tuple[int, int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                for e_idx in range(len(edges)):
                    arcs[(cx, cy, d, e_idx)] = model.NewBoolVar(
                        f"a_{cx}_{cy}_{d}_{e_idx}"
                    )

    # No arcs leaving the grid.
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                ncx, ncy = cx + DIR_STEPS[d][0], cy + DIR_STEPS[d][1]
                if not (0 <= ncx < width and 0 <= ncy < height):
                    for e_idx in range(len(edges)):
                        model.Add(arcs[(cx, cy, d, e_idx)] == 0)

    # At most one outgoing direction per (cell, edge).
    for cx in range(width):
        for cy in range(height):
            for e_idx in range(len(edges)):
                model.AddAtMostOne(
                    [arcs[(cx, cy, d, e_idx)] for d in range(4)]
                )

    # At most one edge owns a non-splitter cell.
    for cx in range(width):
        for cy in range(height):
            if (cx, cy) in splitter_cells:
                continue
            cell_uses = []
            for e_idx in range(len(edges)):
                cell_uses.append(
                    sum(arcs[(cx, cy, d, e_idx)] for d in range(4))
                )
            # Each entry is 0 or 1 (from at-most-one constraint above).
            model.Add(sum(cell_uses) <= 1)

    # Flow conservation per (cell, edge).
    for cx in range(width):
        for cy in range(height):
            for e_idx, edge in enumerate(edges):
                outflow = sum(arcs[(cx, cy, d, e_idx)] for d in range(4))
                inflow_terms = []
                for d in range(4):
                    ncx = cx - DIR_STEPS[d][0]
                    ncy = cy - DIR_STEPS[d][1]
                    if 0 <= ncx < width and 0 <= ncy < height:
                        inflow_terms.append(arcs[(ncx, ncy, d, e_idx)])
                inflow = sum(inflow_terms) if inflow_terms else 0

                is_src = (cx, cy) == edge_src[e_idx]
                is_dst = (cx, cy) == edge_dst[e_idx]
                if is_src and is_dst:
                    # 0-length edge — should never happen with our
                    # source/dest conventions, but fall back to no-flow.
                    model.Add(outflow == 0)
                    model.Add(inflow == 0)
                elif is_src:
                    model.Add(outflow - inflow == 1)
                elif is_dst:
                    model.Add(inflow - outflow == 1)
                else:
                    model.Add(outflow - inflow == 0)

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 30.0
    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    out: dict = {"status": solver.StatusName(status), "elapsed_s": elapsed}
    if status in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        belts = []
        # Emit a belt for every non-splitter cell that carries edge
        # flow. Direction = outgoing arc direction if there is one
        # (transit or source); else incoming arc direction (terminal
        # belt at an output port — flow arrives and continues off-grid
        # into the downstream consumer).
        for cx in range(width):
            for cy in range(height):
                if (cx, cy) in splitter_cells:
                    continue
                # Outgoing arc?
                emitted = False
                for d in range(4):
                    for e_idx in range(len(edges)):
                        if solver.Value(arcs[(cx, cy, d, e_idx)]) == 1:
                            belts.append(
                                {
                                    "x": cx,
                                    "y": cy,
                                    "dir": INTERNAL_TO_FACTORIO_DIR[d],
                                    "edge_idx": e_idx,
                                }
                            )
                            emitted = True
                            break
                    if emitted:
                        break
                if emitted:
                    continue
                # No outgoing — check incoming. Terminal belt direction
                # equals the incoming arc's direction.
                for d in range(4):
                    ncx, ncy = cx - DIR_STEPS[d][0], cy - DIR_STEPS[d][1]
                    if not (0 <= ncx < width and 0 <= ncy < height):
                        continue
                    for e_idx in range(len(edges)):
                        if solver.Value(arcs[(ncx, ncy, d, e_idx)]) == 1:
                            belts.append(
                                {
                                    "x": cx,
                                    "y": cy,
                                    "dir": INTERNAL_TO_FACTORIO_DIR[d],
                                    "edge_idx": e_idx,
                                }
                            )
                            emitted = True
                            break
                    if emitted:
                        break
        out["belts"] = belts
    return out


def main() -> None:
    req = json.load(sys.stdin)
    if "edges" in req and "splitter_positions" in req:
        out = solve_routing(req)
    else:
        out = solve_overlap_only(req)
    json.dump(out, sys.stdout)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
