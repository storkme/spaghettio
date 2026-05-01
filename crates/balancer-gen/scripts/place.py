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

# Underground belt max reach (yellow-belt tier; phase 3.2B-scoped to
# single tier). One UG pair spans 1..UG_MAX_REACH cells in the chosen
# direction (length 1 = input and output adjacent — admittedly silly
# but lets the encoder treat the trivial case uniformly).
UG_MAX_REACH = 4


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
    """Phase 3.2A.1 / 3.2B / 3.2C — belt routing with given splitter
    positions AND directions (caller-provided)."""
    width, height = req["bounds"]
    splitter_positions = req["splitter_positions"]
    input_port_tiles = [tuple(t) for t in req["input_port_tiles"]]
    output_port_tiles = [tuple(t) for t in req["output_port_tiles"]]
    edges = req["edges"]

    # Per-splitter facing direction (Factorio encoding 0/2/4/6 → internal 0/1/2/3).
    # Default to S (4 → internal 2) for backward compat with 3.2A.1.
    factorio_to_internal = {0: 0, 2: 1, 4: 2, 6: 3}
    splitter_dirs_internal = [
        factorio_to_internal[sp.get("dir", 4)] for sp in splitter_positions
    ]

    def splitter_second_offset(dir_internal: int) -> tuple[int, int]:
        # Internal 0=N, 1=E, 2=S, 3=W. Second tile is perpendicular to
        # facing (e.g. south-facing has second to the east).
        if dir_internal in (0, 2):  # N/S
            return (1, 0)
        return (0, 1)  # E/W

    # Splitter cells (anchor + perpendicular second).
    splitter_cells: set[tuple[int, int]] = set()
    splitter_anchor_to_idx: dict[tuple[int, int], int] = {}
    for s_idx, sp in enumerate(splitter_positions):
        ax, ay = sp["x"], sp["y"]
        bx, by = ax + splitter_second_offset(splitter_dirs_internal[s_idx])[0], ay + splitter_second_offset(splitter_dirs_internal[s_idx])[1]
        splitter_cells.add((ax, ay))
        splitter_cells.add((bx, by))
        splitter_anchor_to_idx[(ax, ay)] = s_idx

    def splitter_tile(s_idx: int, slot: int) -> tuple[int, int]:
        sp = splitter_positions[s_idx]
        ax, ay = sp["x"], sp["y"]
        if slot == 0:
            return (ax, ay)
        bx, by = (
            ax + splitter_second_offset(splitter_dirs_internal[s_idx])[0],
            ay + splitter_second_offset(splitter_dirs_internal[s_idx])[1],
        )
        return (bx, by)

    def src_cell(edge: dict) -> tuple[int, int]:
        if edge["src_kind"] == "InputPort":
            return input_port_tiles[edge["src_idx"]]
        return splitter_tile(edge["src_idx"], edge.get("src_slot", 0))

    def dst_cell(edge: dict) -> tuple[int, int]:
        if edge["dst_kind"] == "OutputPort":
            return output_port_tiles[edge["dst_idx"]]
        return splitter_tile(edge["dst_idx"], edge.get("dst_slot", 0))

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

    # UG pair variables. ug_arcs[(cx, cy, d, L, e)] = 1 means edge e has
    # an underground-belt input at (cx, cy) facing d, output at
    # (cx + L*dx, cy + L*dy). Only created where the output cell is
    # in-grid.
    ug_arcs: dict[tuple[int, int, int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                dx, dy = DIR_STEPS[d]
                for L in range(1, UG_MAX_REACH + 1):
                    ncx, ncy = cx + L * dx, cy + L * dy
                    if 0 <= ncx < width and 0 <= ncy < height:
                        for e_idx in range(len(edges)):
                            ug_arcs[(cx, cy, d, L, e_idx)] = model.NewBoolVar(
                                f"u_{cx}_{cy}_{d}_{L}_{e_idx}"
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

    # At most one entity per non-splitter cell. A cell can be:
    #   - a regular belt for some edge (any of arcs[(cx, cy, d, e)])
    #   - a UG input for some edge (any of ug_arcs[(cx, cy, d, L, e)])
    #   - a UG output for some edge (= UG input upstream by L)
    # Sum across all of these is ≤ 1.
    for cx in range(width):
        for cy in range(height):
            if (cx, cy) in splitter_cells:
                continue
            terms = []
            for e_idx in range(len(edges)):
                terms.append(sum(arcs[(cx, cy, d, e_idx)] for d in range(4)))
                # UG inputs at this cell.
                for d in range(4):
                    for L in range(1, UG_MAX_REACH + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                # UG outputs at this cell (i.e., inputs upstream).
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
            model.Add(sum(terms) <= 1)

    # UG cells cannot coincide with splitter cells either (a splitter
    # tile is 1x1 just like a UG entity, and they can't share).
    for sp_cell in splitter_cells:
        cx, cy = sp_cell
        for e_idx in range(len(edges)):
            for d in range(4):
                for L in range(1, UG_MAX_REACH + 1):
                    if (cx, cy, d, L, e_idx) in ug_arcs:
                        model.Add(ug_arcs[(cx, cy, d, L, e_idx)] == 0)
                    dx, dy = DIR_STEPS[d]
                    ucx, ucy = cx - L * dx, cy - L * dy
                    if (ucx, ucy, d, L, e_idx) in ug_arcs:
                        model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0)
            # Splitter cells also can't carry regular edge belts.
            for d in range(4):
                # (Already handled by conservation — splitter cells with
                # outgoing edge-belt flow would correspond to source
                # arcs, which are valid; this comment is a no-op.)
                pass

    # UG pairing rule (simplified for single tier): if two UG arcs
    # share a direction-axis and the output of one falls strictly
    # between another's input and output, Factorio would re-pair them.
    # Forbid that configuration so the encoded arcs match in-game
    # pairing 1:1.
    for (c1x, c1y, d1, L1, e1), arc1 in ug_arcs.items():
        dx, dy = DIR_STEPS[d1]
        for k in range(1, L1):
            mcx, mcy = c1x + k * dx, c1y + k * dy
            # Any OTHER ug_arc whose output is at (mcx, mcy) heading d1.
            for L2 in range(1, UG_MAX_REACH + 1):
                ucx, ucy = mcx - L2 * dx, mcy - L2 * dy
                if not (0 <= ucx < width and 0 <= ucy < height):
                    continue
                for e2 in range(len(edges)):
                    if (ucx, ucy, d1, L2, e2) in ug_arcs and (ucx, ucy, d1, L2, e2) != (c1x, c1y, d1, L1, e1):
                        model.AddBoolOr([arc1.Not(), ug_arcs[(ucx, ucy, d1, L2, e2)].Not()])

    # Direction constraints at splitter sources/dests. For an edge that
    # sources at a splitter, the outflow at the source cell must head in
    # the splitter's facing direction (the splitter's "front"). Similarly
    # for dests, the incoming arc must come from the back side (i.e.,
    # the cell behind the splitter contributes flow heading in the
    # facing direction).
    for e_idx, edge in enumerate(edges):
        if edge["src_kind"] == "Splitter":
            s_idx = edge["src_idx"]
            facing = splitter_dirs_internal[s_idx]
            scx, scy = edge_src[e_idx]
            for d in range(4):
                if d != facing:
                    model.Add(arcs[(scx, scy, d, e_idx)] == 0)
                    # Also forbid UG arcs at splitter source in non-facing direction.
                    for L in range(1, UG_MAX_REACH + 1):
                        if (scx, scy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(scx, scy, d, L, e_idx)] == 0)
        if edge["dst_kind"] == "Splitter":
            s_idx = edge["dst_idx"]
            facing = splitter_dirs_internal[s_idx]
            dcx, dcy = edge_dst[e_idx]
            # Incoming flow at dest must be from direction = facing
            # (i.e., the back-side neighbor's outflow heads in facing
            # direction toward the splitter).
            for d in range(4):
                if d == facing:
                    continue
                # Forbid belt-arcs from non-facing directions.
                ncx, ncy = dcx - DIR_STEPS[d][0], dcy - DIR_STEPS[d][1]
                if 0 <= ncx < width and 0 <= ncy < height:
                    model.Add(arcs[(ncx, ncy, d, e_idx)] == 0)
                # Forbid UG arcs ending at dest from wrong direction.
                for L in range(1, UG_MAX_REACH + 1):
                    ucx, ucy = dcx - L * DIR_STEPS[d][0], dcy - L * DIR_STEPS[d][1]
                    if (ucx, ucy, d, L, e_idx) in ug_arcs:
                        model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0)

    # Flow conservation per (cell, edge). Outflow includes UG arcs
    # SOURCED at this cell (the input half of a UG); inflow includes UG
    # arcs ENDING at this cell (the output half of a UG that started L
    # cells upstream in direction d).
    for cx in range(width):
        for cy in range(height):
            for e_idx, edge in enumerate(edges):
                belt_outflow = sum(arcs[(cx, cy, d, e_idx)] for d in range(4))
                ug_outflow_terms = []
                for d in range(4):
                    for L in range(1, UG_MAX_REACH + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            ug_outflow_terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                ug_outflow = sum(ug_outflow_terms) if ug_outflow_terms else 0

                belt_inflow_terms = []
                for d in range(4):
                    ncx = cx - DIR_STEPS[d][0]
                    ncy = cy - DIR_STEPS[d][1]
                    if 0 <= ncx < width and 0 <= ncy < height:
                        belt_inflow_terms.append(arcs[(ncx, ncy, d, e_idx)])
                belt_inflow = sum(belt_inflow_terms) if belt_inflow_terms else 0

                ug_inflow_terms = []
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            ug_inflow_terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
                ug_inflow = sum(ug_inflow_terms) if ug_inflow_terms else 0

                outflow = belt_outflow + ug_outflow
                inflow = belt_inflow + ug_inflow

                is_src = (cx, cy) == edge_src[e_idx]
                is_dst = (cx, cy) == edge_dst[e_idx]
                if is_src and is_dst:
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
        ugs = []  # underground-belt entities: kind "input" or "output"
        # First pass: emit UG inputs and outputs.
        for (cx, cy, d, L, e_idx), arc_var in ug_arcs.items():
            if solver.Value(arc_var) == 1:
                fdir = INTERNAL_TO_FACTORIO_DIR[d]
                ugs.append({"x": cx, "y": cy, "dir": fdir, "io": "input", "edge_idx": e_idx})
                dx, dy = DIR_STEPS[d]
                ugs.append(
                    {"x": cx + L * dx, "y": cy + L * dy, "dir": fdir, "io": "output", "edge_idx": e_idx}
                )

        # Second pass: emit belts for cells with regular arcs that
        # aren't UG cells. Direction = outgoing arc direction if any,
        # else incoming (terminal belt at output port).
        ug_cells = {(u["x"], u["y"]) for u in ugs}
        for cx in range(width):
            for cy in range(height):
                if (cx, cy) in splitter_cells or (cx, cy) in ug_cells:
                    continue
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
                # No outgoing — terminal belt with incoming direction.
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
        out["ugs"] = ugs
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
