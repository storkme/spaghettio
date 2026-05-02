#!/usr/bin/env -S uv run --no-project --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["ortools>=9.10"]
# ///
"""Phase 3.2A — flow-conservation belt routing for splitter layouts.

Two modes, selected by the input JSON:

**Mode A — splitter placement only** (phase 3.1 spike).
Input has `n_splitters` and `bounds` only. CP-SAT places splitters with
no-overlap and reports positions. Belt routing not solved.

**Mode B — belt routing with given splitter layout** (phase 3.2A).
Input has `splitter_positions` (each with `x`, `y`, `dir` in Factorio
encoding 0/2/4/6), `input_port_tiles`, `output_port_tiles`, `edges`
(topology only — no slot info), and `bounds`. CP-SAT encodes:

  - For each cell `c`, each topology edge `e`, each direction `d`:
    bool `arc[c, d, e]` — is there flow of edge e leaving cell c
    heading direction d?
  - UG pairs as virtual arcs spanning 1..UG_MAX_REACH cells.
  - Each non-splitter cell hosts at most one entity (belt or UG).
  - For each edge with src_kind=Splitter: bool `src_slot_anchor[e]` and
    `src_slot_second[e]` with ExactlyOne — picks which of the splitter's
    two output tiles is the source. Symmetric for dests. Per splitter,
    AtMostOne enforces no two edges share an output (or input) slot.
  - Conservation per (cell, edge): outflow - inflow = is_src - is_dst,
    where is_src/is_dst at splitter tiles are reified slot bool vars.
  - Splitter cells: outflow only in facing direction, inflow only from
    facing direction, total outflow ≤ is_src, total inflow ≤ is_dst
    (no transit through splitters).

Output: belt and UG entity lists with their chosen positions and
directions. Slot assignment is implicit in the placed entities.

Future phases:
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

# Underground belt max length (yellow-belt tier; phase 3.2B-scoped to
# single tier). L in `ug_arcs[(c, d, L, e)]` is the offset from input
# to output (output cell = input + L*d_step). Yellow belts allow up to
# 4 transit tiles between input and output, i.e., L ∈ 1..5.
UG_MAX_REACH = 5


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
    """Phase 3.2A — belt routing with given splitter positions and
    directions; CP-SAT picks slot assignments as variables."""
    width, height = req["bounds"]
    splitter_positions = req["splitter_positions"]
    input_port_tiles = [tuple(t) for t in req["input_port_tiles"]]
    output_port_tiles = [tuple(t) for t in req["output_port_tiles"]]
    edges = req["edges"]

    # Per-splitter facing direction (Factorio encoding 0/2/4/6 → internal 0/1/2/3).
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
    for s_idx, sp in enumerate(splitter_positions):
        ax, ay = sp["x"], sp["y"]
        ofx, ofy = splitter_second_offset(splitter_dirs_internal[s_idx])
        bx, by = ax + ofx, ay + ofy
        splitter_cells.add((ax, ay))
        splitter_cells.add((bx, by))

    def splitter_tile(s_idx: int, slot: int) -> tuple[int, int]:
        sp = splitter_positions[s_idx]
        ax, ay = sp["x"], sp["y"]
        if slot == 0:
            return (ax, ay)
        ofx, ofy = splitter_second_offset(splitter_dirs_internal[s_idx])
        return (ax + ofx, ay + ofy)

    model = cp_model.CpModel()

    # Per-edge slot vars: when src_kind=Splitter, ExactlyOne picks anchor
    # vs second as the source tile. Symmetric for dest. Constants for
    # InputPort/OutputPort sides — the cell is fixed.
    src_slot_anchor: dict[int, any] = {}
    src_slot_second: dict[int, any] = {}
    dst_slot_anchor: dict[int, any] = {}
    dst_slot_second: dict[int, any] = {}
    for e_idx, edge in enumerate(edges):
        if edge["src_kind"] == "Splitter":
            a = model.NewBoolVar(f"src_anchor_e{e_idx}")
            s = model.NewBoolVar(f"src_second_e{e_idx}")
            model.AddExactlyOne([a, s])
            src_slot_anchor[e_idx] = a
            src_slot_second[e_idx] = s
        if edge["dst_kind"] == "Splitter":
            a = model.NewBoolVar(f"dst_anchor_e{e_idx}")
            s = model.NewBoolVar(f"dst_second_e{e_idx}")
            model.AddExactlyOne([a, s])
            dst_slot_anchor[e_idx] = a
            dst_slot_second[e_idx] = s

    # Per-splitter slot uniqueness: across all edges sharing a splitter
    # endpoint, at most one picks anchor (and at most one picks second)
    # as its output slot. Same for input slots.
    for s_idx in range(len(splitter_positions)):
        out_anchor_users = [
            src_slot_anchor[e_idx]
            for e_idx, edge in enumerate(edges)
            if edge["src_kind"] == "Splitter" and edge["src_idx"] == s_idx
        ]
        out_second_users = [
            src_slot_second[e_idx]
            for e_idx, edge in enumerate(edges)
            if edge["src_kind"] == "Splitter" and edge["src_idx"] == s_idx
        ]
        in_anchor_users = [
            dst_slot_anchor[e_idx]
            for e_idx, edge in enumerate(edges)
            if edge["dst_kind"] == "Splitter" and edge["dst_idx"] == s_idx
        ]
        in_second_users = [
            dst_slot_second[e_idx]
            for e_idx, edge in enumerate(edges)
            if edge["dst_kind"] == "Splitter" and edge["dst_idx"] == s_idx
        ]
        if len(out_anchor_users) >= 2:
            model.AddAtMostOne(out_anchor_users)
        if len(out_second_users) >= 2:
            model.AddAtMostOne(out_second_users)
        if len(in_anchor_users) >= 2:
            model.AddAtMostOne(in_anchor_users)
        if len(in_second_users) >= 2:
            model.AddAtMostOne(in_second_users)

    def is_src_term_at(cx: int, cy: int, e_idx: int):
        """Linear expr / constant: 1 iff (cx, cy) is the chosen source
        cell for edge e_idx."""
        edge = edges[e_idx]
        if edge["src_kind"] == "InputPort":
            return 1 if (cx, cy) == input_port_tiles[edge["src_idx"]] else 0
        s_idx = edge["src_idx"]
        anchor = splitter_tile(s_idx, 0)
        second = splitter_tile(s_idx, 1)
        if (cx, cy) == anchor:
            return src_slot_anchor[e_idx]
        if (cx, cy) == second:
            return src_slot_second[e_idx]
        return 0

    def is_dst_term_at(cx: int, cy: int, e_idx: int):
        edge = edges[e_idx]
        if edge["dst_kind"] == "OutputPort":
            return 1 if (cx, cy) == output_port_tiles[edge["dst_idx"]] else 0
        s_idx = edge["dst_idx"]
        anchor = splitter_tile(s_idx, 0)
        second = splitter_tile(s_idx, 1)
        if (cx, cy) == anchor:
            return dst_slot_anchor[e_idx]
        if (cx, cy) == second:
            return dst_slot_second[e_idx]
        return 0

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

    # Splitter-cell direction + transit constraints (apply to every edge
    # at every splitter tile, regardless of whether the slot is chosen
    # for that edge). Outflow allowed only in facing direction; inflow
    # allowed only from facing direction. Total outflow/inflow at the
    # cell are bounded by the reified is_src/is_dst slot vars (no
    # transit through splitters).
    for s_idx in range(len(splitter_positions)):
        facing = splitter_dirs_internal[s_idx]
        for slot in (0, 1):
            cx, cy = splitter_tile(s_idx, slot)
            for e_idx in range(len(edges)):
                # Forbid outflow in non-facing directions.
                for d in range(4):
                    if d != facing:
                        model.Add(arcs[(cx, cy, d, e_idx)] == 0)
                        for L in range(1, UG_MAX_REACH + 1):
                            if (cx, cy, d, L, e_idx) in ug_arcs:
                                model.Add(ug_arcs[(cx, cy, d, L, e_idx)] == 0)
                # Forbid inflow from non-facing directions.
                for d in range(4):
                    if d == facing:
                        continue
                    ncx = cx - DIR_STEPS[d][0]
                    ncy = cy - DIR_STEPS[d][1]
                    if 0 <= ncx < width and 0 <= ncy < height:
                        model.Add(arcs[(ncx, ncy, d, e_idx)] == 0)
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx = cx - L * DIR_STEPS[d][0]
                        ucy = cy - L * DIR_STEPS[d][1]
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0)
                # Bound total outflow ≤ is_src_term, total inflow ≤ is_dst_term.
                outflow_terms = [arcs[(cx, cy, facing, e_idx)]]
                for L in range(1, UG_MAX_REACH + 1):
                    if (cx, cy, facing, L, e_idx) in ug_arcs:
                        outflow_terms.append(ug_arcs[(cx, cy, facing, L, e_idx)])
                model.Add(sum(outflow_terms) <= is_src_term_at(cx, cy, e_idx))

                inflow_terms = []
                ncx = cx - DIR_STEPS[facing][0]
                ncy = cy - DIR_STEPS[facing][1]
                if 0 <= ncx < width and 0 <= ncy < height:
                    inflow_terms.append(arcs[(ncx, ncy, facing, e_idx)])
                # UG inflow at splitter cells: a UG output one step
                # behind the splitter (in facing direction) emits forward
                # into the splitter. Its input is at (cx - (L+1)*d, ...).
                for L in range(1, UG_MAX_REACH + 1):
                    ucx = cx - (L + 1) * DIR_STEPS[facing][0]
                    ucy = cy - (L + 1) * DIR_STEPS[facing][1]
                    if (ucx, ucy, facing, L, e_idx) in ug_arcs:
                        inflow_terms.append(ug_arcs[(ucx, ucy, facing, L, e_idx)])
                if inflow_terms:
                    model.Add(sum(inflow_terms) <= is_dst_term_at(cx, cy, e_idx))

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

                # UG inflow at this cell: a UG arc with input at
                # (cx - (L+1)*dx, ...) and output at (cx - dx, ...) emits
                # forward in direction d, landing here as inflow. The
                # output cell itself is a passthrough — its UG inflow
                # cancels with its UG output emission.
                ug_inflow_terms = []
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx = cx - (L + 1) * dx
                        ucy = cy - (L + 1) * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            ug_inflow_terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
                ug_inflow = sum(ug_inflow_terms) if ug_inflow_terms else 0

                outflow = belt_outflow + ug_outflow
                inflow = belt_inflow + ug_inflow

                src_term = is_src_term_at(cx, cy, e_idx)
                dst_term = is_dst_term_at(cx, cy, e_idx)
                model.Add(outflow - inflow == src_term - dst_term)

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


def solve_synth_place(req: dict) -> dict:
    """Phase 3.2D.1 — synth-graph placement at a fixed bbox.

    Takes a topology (n_inputs, n_outputs, n_splitters, edges) and a
    bbox; CP-SAT picks splitter anchor cells, IO port columns, slot
    assignments, and belt/UG routing in one shot. All splitters are
    forced to face south (direction freedom deferred to 3.2D.2).

    IO layout: input ports on row y=0, output ports on row y=H-1.
    x-positions of each port are CP-SAT variables (all-different per
    side). Splitters anchor in [0, W-2] × [1, H-2] (so they can't
    collide with IO rows; second tile at anchor + east).

    For bbox minimization, the caller iterates: try (W, H), shrink on
    success, expand on infeasibility.
    """
    n_inputs = req["n_inputs"]
    n_outputs = req["n_outputs"]
    n_splitters = req["n_splitters"]
    edges = req["edges"]
    width, height = req["bounds"]

    if height < 3 or width < max(n_inputs, n_outputs, 2):
        return {
            "status": "INFEASIBLE",
            "elapsed_s": 0.0,
            "error": f"bbox {width}×{height} too small for {n_inputs}→{n_outputs}",
        }

    model = cp_model.CpModel()
    facing = 2  # internal south
    sp_y_lo, sp_y_hi = 1, height - 2

    # anchor_at[(s, cx, cy)]: splitter s has anchor at (cx, cy).
    anchor_at: dict[tuple[int, int, int], any] = {}
    for s in range(n_splitters):
        anchor_cells = []
        for cx in range(width - 1):  # need cx+1 in-grid for second tile
            for cy in range(sp_y_lo, sp_y_hi + 1):
                v = model.NewBoolVar(f"anchor_{s}_{cx}_{cy}")
                anchor_at[(s, cx, cy)] = v
                anchor_cells.append(v)
        model.AddExactlyOne(anchor_cells)

    # Cell-level predicates: is the cell some splitter's anchor / second tile?
    is_anchor_cell: dict[tuple[int, int], any] = {}
    is_second_cell: dict[tuple[int, int], any] = {}
    is_splitter_cell: dict[tuple[int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            anchors_here = [
                anchor_at[(s, cx, cy)]
                for s in range(n_splitters)
                if (s, cx, cy) in anchor_at
            ]
            seconds_here = [
                anchor_at[(s, cx - 1, cy)]
                for s in range(n_splitters)
                if (s, cx - 1, cy) in anchor_at
            ]
            ia = model.NewBoolVar(f"is_anchor_{cx}_{cy}")
            is2 = model.NewBoolVar(f"is_second_{cx}_{cy}")
            isp = model.NewBoolVar(f"is_splitter_{cx}_{cy}")
            if anchors_here:
                model.Add(sum(anchors_here) == ia)
            else:
                model.Add(ia == 0)
            if seconds_here:
                model.Add(sum(seconds_here) == is2)
            else:
                model.Add(is2 == 0)
            # At most one splitter tile per cell (no-overlap).
            model.Add(ia + is2 <= 1)
            model.Add(isp == ia + is2)
            is_anchor_cell[(cx, cy)] = ia
            is_second_cell[(cx, cy)] = is2
            is_splitter_cell[(cx, cy)] = isp

    # IO port column vars and reified per-cell bools. y is fixed (input=0,
    # output=H-1).
    ix = [model.NewIntVar(0, width - 1, f"ix{i}") for i in range(n_inputs)]
    ox = [model.NewIntVar(0, width - 1, f"ox{j}") for j in range(n_outputs)]
    # Sorted by x so port indices match physical left-to-right order
    # (the classifier identifies output ports by their tile position
    # in the template's output_tiles array, so ordering matters for
    # the "right" output to receive the right lane's flow).
    for i in range(n_inputs - 1):
        model.Add(ix[i] < ix[i + 1])
    for j in range(n_outputs - 1):
        model.Add(ox[j] < ox[j + 1])

    input_at: dict[tuple[int, int], any] = {}
    for i in range(n_inputs):
        for cx in range(width):
            v = model.NewBoolVar(f"in_{i}_at_{cx}")
            model.Add(ix[i] == cx).OnlyEnforceIf(v)
            model.Add(ix[i] != cx).OnlyEnforceIf(v.Not())
            input_at[(i, cx)] = v

    output_at: dict[tuple[int, int], any] = {}
    for j in range(n_outputs):
        for cx in range(width):
            v = model.NewBoolVar(f"out_{j}_at_{cx}")
            model.Add(ox[j] == cx).OnlyEnforceIf(v)
            model.Add(ox[j] != cx).OnlyEnforceIf(v.Not())
            output_at[(j, cx)] = v

    # Per-edge slot vars (3.2A.2). For Splitter sources/dests, ExactlyOne
    # picks anchor vs second; AtMostOne per splitter slot enforces no two
    # edges share an output (or input) slot.
    src_slot_anchor: dict[int, any] = {}
    src_slot_second: dict[int, any] = {}
    dst_slot_anchor: dict[int, any] = {}
    dst_slot_second: dict[int, any] = {}
    for e_idx, edge in enumerate(edges):
        if edge["src_kind"] == "Splitter":
            a = model.NewBoolVar(f"src_anchor_e{e_idx}")
            b = model.NewBoolVar(f"src_second_e{e_idx}")
            model.AddExactlyOne([a, b])
            src_slot_anchor[e_idx] = a
            src_slot_second[e_idx] = b
        if edge["dst_kind"] == "Splitter":
            a = model.NewBoolVar(f"dst_anchor_e{e_idx}")
            b = model.NewBoolVar(f"dst_second_e{e_idx}")
            model.AddExactlyOne([a, b])
            dst_slot_anchor[e_idx] = a
            dst_slot_second[e_idx] = b

    for s in range(n_splitters):
        for users in (
            [src_slot_anchor[e_idx] for e_idx, edge in enumerate(edges)
             if edge["src_kind"] == "Splitter" and edge["src_idx"] == s],
            [src_slot_second[e_idx] for e_idx, edge in enumerate(edges)
             if edge["src_kind"] == "Splitter" and edge["src_idx"] == s],
            [dst_slot_anchor[e_idx] for e_idx, edge in enumerate(edges)
             if edge["dst_kind"] == "Splitter" and edge["dst_idx"] == s],
            [dst_slot_second[e_idx] for e_idx, edge in enumerate(edges)
             if edge["dst_kind"] == "Splitter" and edge["dst_idx"] == s],
        ):
            if len(users) >= 2:
                model.AddAtMostOne(users)

    # is_src_term_at(cx, cy, e_idx): linear expr — 1 iff (cx, cy) is the
    # chosen source cell for edge e_idx, considering reified splitter +
    # IO positions + slot vars.
    def is_src_term_at(cx: int, cy: int, e_idx: int):
        edge = edges[e_idx]
        if edge["src_kind"] == "InputPort":
            if cy != 0:
                return 0
            return input_at[(edge["src_idx"], cx)]
        s = edge["src_idx"]
        # At anchor cell: contributes anchor_at[s, cx, cy] AND src_slot_anchor[e]
        # At second cell (anchor at cx-1, cy): contributes anchor_at[s, cx-1, cy] AND src_slot_second[e]
        terms = []
        if (s, cx, cy) in anchor_at:
            ab = model.NewBoolVar(f"src_a_{s}_{cx}_{cy}_e{e_idx}")
            model.AddBoolAnd([anchor_at[(s, cx, cy)], src_slot_anchor[e_idx]]).OnlyEnforceIf(ab)
            model.AddBoolOr([
                anchor_at[(s, cx, cy)].Not(), src_slot_anchor[e_idx].Not(),
            ]).OnlyEnforceIf(ab.Not())
            terms.append(ab)
        if (s, cx - 1, cy) in anchor_at:
            sb = model.NewBoolVar(f"src_s_{s}_{cx}_{cy}_e{e_idx}")
            model.AddBoolAnd([anchor_at[(s, cx - 1, cy)], src_slot_second[e_idx]]).OnlyEnforceIf(sb)
            model.AddBoolOr([
                anchor_at[(s, cx - 1, cy)].Not(), src_slot_second[e_idx].Not(),
            ]).OnlyEnforceIf(sb.Not())
            terms.append(sb)
        if not terms:
            return 0
        return sum(terms)

    def is_dst_term_at(cx: int, cy: int, e_idx: int):
        edge = edges[e_idx]
        if edge["dst_kind"] == "OutputPort":
            if cy != height - 1:
                return 0
            return output_at[(edge["dst_idx"], cx)]
        s = edge["dst_idx"]
        terms = []
        if (s, cx, cy) in anchor_at:
            ab = model.NewBoolVar(f"dst_a_{s}_{cx}_{cy}_e{e_idx}")
            model.AddBoolAnd([anchor_at[(s, cx, cy)], dst_slot_anchor[e_idx]]).OnlyEnforceIf(ab)
            model.AddBoolOr([
                anchor_at[(s, cx, cy)].Not(), dst_slot_anchor[e_idx].Not(),
            ]).OnlyEnforceIf(ab.Not())
            terms.append(ab)
        if (s, cx - 1, cy) in anchor_at:
            sb = model.NewBoolVar(f"dst_s_{s}_{cx}_{cy}_e{e_idx}")
            model.AddBoolAnd([anchor_at[(s, cx - 1, cy)], dst_slot_second[e_idx]]).OnlyEnforceIf(sb)
            model.AddBoolOr([
                anchor_at[(s, cx - 1, cy)].Not(), dst_slot_second[e_idx].Not(),
            ]).OnlyEnforceIf(sb.Not())
            terms.append(sb)
        if not terms:
            return 0
        return sum(terms)

    # Routing arcs over the full grid.
    arcs: dict[tuple[int, int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                for e_idx in range(len(edges)):
                    arcs[(cx, cy, d, e_idx)] = model.NewBoolVar(
                        f"a_{cx}_{cy}_{d}_{e_idx}"
                    )

    # UG arcs. Direction restricted to facing (south) so the solver
    # doesn't take U-turns via UGs heading away from the main flow
    # direction — the conservation alone allows valid-but-nonsensical
    # loops via reverse-direction UGs (a (1, 4) spike with all 4
    # directions allowed produced a north-bound UG from the output
    # row, classified MX2a). All-south matches the all-south splitter
    # constraint in this phase.
    ug_arcs: dict[tuple[int, int, int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            d = facing
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
                model.AddAtMostOne([arcs[(cx, cy, d, e_idx)] for d in range(4)])

    # Splitter-cell direction + transit constraints — gated on
    # is_splitter_cell[cx, cy]. Since splitters are all-south, the only
    # allowed in/out direction at a splitter cell is south (d=2).
    for cx in range(width):
        for cy in range(height):
            isp = is_splitter_cell[(cx, cy)]
            for e_idx in range(len(edges)):
                # Forbid outflow in non-facing directions if cell is splitter.
                for d in range(4):
                    if d != facing:
                        # arcs[(cx, cy, d, e)] = 0 if isp = 1.
                        model.Add(arcs[(cx, cy, d, e_idx)] == 0).OnlyEnforceIf(isp)
                        for L in range(1, UG_MAX_REACH + 1):
                            if (cx, cy, d, L, e_idx) in ug_arcs:
                                model.Add(ug_arcs[(cx, cy, d, L, e_idx)] == 0).OnlyEnforceIf(isp)
                # Forbid inflow from non-facing directions if cell is splitter.
                for d in range(4):
                    if d == facing:
                        continue
                    ncx = cx - DIR_STEPS[d][0]
                    ncy = cy - DIR_STEPS[d][1]
                    if 0 <= ncx < width and 0 <= ncy < height:
                        model.Add(arcs[(ncx, ncy, d, e_idx)] == 0).OnlyEnforceIf(isp)
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx = cx - L * DIR_STEPS[d][0]
                        ucy = cy - L * DIR_STEPS[d][1]
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0).OnlyEnforceIf(isp)
                # UG entities can't be at splitter cells (input or output).
                for d in range(4):
                    for L in range(1, UG_MAX_REACH + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(cx, cy, d, L, e_idx)] == 0).OnlyEnforceIf(isp)
                        dx, dy = DIR_STEPS[d]
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0).OnlyEnforceIf(isp)

    # At most one entity per non-splitter cell (regular belt or UG endpoint).
    for cx in range(width):
        for cy in range(height):
            isp = is_splitter_cell[(cx, cy)]
            terms = []
            for e_idx in range(len(edges)):
                terms.append(sum(arcs[(cx, cy, d, e_idx)] for d in range(4)))
                for d in range(4):
                    for L in range(1, UG_MAX_REACH + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
            # If this cell is a splitter, the at-most-one rule is relaxed
            # (the splitter itself "is" the entity; multiple edges' arcs
            # may legitimately route through it). Otherwise sum ≤ 1.
            model.Add(sum(terms) <= 1).OnlyEnforceIf(isp.Not())

    # UG pairing rule (matches Mode C).
    for (c1x, c1y, d1, L1, e1), arc1 in ug_arcs.items():
        dx, dy = DIR_STEPS[d1]
        for k in range(1, L1):
            mcx, mcy = c1x + k * dx, c1y + k * dy
            for L2 in range(1, UG_MAX_REACH + 1):
                ucx, ucy = mcx - L2 * dx, mcy - L2 * dy
                if not (0 <= ucx < width and 0 <= ucy < height):
                    continue
                for e2 in range(len(edges)):
                    if (ucx, ucy, d1, L2, e2) in ug_arcs and (ucx, ucy, d1, L2, e2) != (c1x, c1y, d1, L1, e1):
                        model.AddBoolOr([arc1.Not(), ug_arcs[(ucx, ucy, d1, L2, e2)].Not()])

    # Flow conservation per (cell, edge), reified is_src/is_dst.
    for cx in range(width):
        for cy in range(height):
            for e_idx in range(len(edges)):
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

                # UG inflow: UG arc with input at (cx - (L+1)*dx, cy - (L+1)*dy)
                # emits forward in direction d, landing here.
                ug_inflow_terms = []
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, UG_MAX_REACH + 1):
                        ucx = cx - (L + 1) * dx
                        ucy = cy - (L + 1) * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            ug_inflow_terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
                ug_inflow = sum(ug_inflow_terms) if ug_inflow_terms else 0

                outflow = belt_outflow + ug_outflow
                inflow = belt_inflow + ug_inflow

                src_term = is_src_term_at(cx, cy, e_idx)
                dst_term = is_dst_term_at(cx, cy, e_idx)
                model.Add(outflow - inflow == src_term - dst_term)

    # Objective: minimize total entity count (regular arcs + UG endpoints).
    # Without this CP-SAT can find valid-but-meandering paths that take
    # detours around the grid; with it, paths are pulled tight, U-turns
    # become uneconomical, and UGs are used only when they save cells.
    entity_terms = []
    for var in arcs.values():
        entity_terms.append(var)
    for var in ug_arcs.values():
        # Each UG counts as 2 entities (input + output).
        entity_terms.append(var)
        entity_terms.append(var)
    model.Minimize(sum(entity_terms))

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 60.0
    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    out: dict = {"status": solver.StatusName(status), "elapsed_s": elapsed}
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return out

    # Extract splitter positions.
    splitters_out = []
    for s in range(n_splitters):
        for (ss, cx, cy), v in anchor_at.items():
            if ss == s and solver.Value(v) == 1:
                splitters_out.append({"x": cx, "y": cy, "dir": INTERNAL_TO_FACTORIO_DIR[facing]})
                break

    # Build splitter cell set for entity emission.
    splitter_cells: set[tuple[int, int]] = set()
    for sp in splitters_out:
        ax, ay = sp["x"], sp["y"]
        splitter_cells.add((ax, ay))
        splitter_cells.add((ax + 1, ay))

    out["splitters"] = splitters_out
    out["input_port_tiles"] = [[solver.Value(ix[i]), 0] for i in range(n_inputs)]
    out["output_port_tiles"] = [[solver.Value(ox[j]), height - 1] for j in range(n_outputs)]

    # UG and belt entity emission (same shape as Mode C).
    belts = []
    ugs = []
    for (cx, cy, d, L, e_idx), arc_var in ug_arcs.items():
        if solver.Value(arc_var) == 1:
            fdir = INTERNAL_TO_FACTORIO_DIR[d]
            ugs.append({"x": cx, "y": cy, "dir": fdir, "io": "input", "edge_idx": e_idx})
            dx, dy = DIR_STEPS[d]
            ugs.append(
                {"x": cx + L * dx, "y": cy + L * dy, "dir": fdir, "io": "output", "edge_idx": e_idx}
            )

    ug_cells = {(u["x"], u["y"]) for u in ugs}
    for cx in range(width):
        for cy in range(height):
            if (cx, cy) in splitter_cells or (cx, cy) in ug_cells:
                continue
            emitted = False
            for d in range(4):
                for e_idx in range(len(edges)):
                    if solver.Value(arcs[(cx, cy, d, e_idx)]) == 1:
                        belts.append({
                            "x": cx, "y": cy,
                            "dir": INTERNAL_TO_FACTORIO_DIR[d],
                            "edge_idx": e_idx,
                        })
                        emitted = True
                        break
                if emitted:
                    break
            if emitted:
                continue
            for d in range(4):
                ncx, ncy = cx - DIR_STEPS[d][0], cy - DIR_STEPS[d][1]
                if not (0 <= ncx < width and 0 <= ncy < height):
                    continue
                for e_idx in range(len(edges)):
                    if solver.Value(arcs[(ncx, ncy, d, e_idx)]) == 1:
                        belts.append({
                            "x": cx, "y": cy,
                            "dir": INTERNAL_TO_FACTORIO_DIR[d],
                            "edge_idx": e_idx,
                        })
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
    elif "edges" in req and "n_splitters" in req:
        out = solve_synth_place(req)
    else:
        out = solve_overlap_only(req)
    json.dump(out, sys.stdout)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
