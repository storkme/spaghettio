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
import os
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

# CP-SAT random seed for pure-routing solvers. CP-SAT runs a 16-worker
# portfolio with random strategy selection; the default wall-clock seed
# produces 35× run-to-run variance on hard problems (observed on (4, 9)
# Clos compose jh=9: 10s vs 353s, same code, same encoding).
# Pinning the seed makes solves deterministic and lets us choose a seed
# whose portfolio assignment happens to be fast for the common shapes.
# Override via env var FUCKTORIO_CP_SAT_SEED=<int> (sweep / testing).
# See docs/rfp-balancer-jh-search.md decision log for selection rationale.
DEFAULT_SEED: int = 42  # placeholder; updated after empirical sweep


def _get_cp_sat_seed(req: dict) -> int:
    """Return the CP-SAT random_seed to use for this request.

    Priority: env var FUCKTORIO_CP_SAT_SEED > req["random_seed"] > DEFAULT_SEED.
    """
    env_val = os.environ.get("FUCKTORIO_CP_SAT_SEED")
    if env_val is not None:
        return int(env_val)
    return int(req.get("random_seed", DEFAULT_SEED))


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
                for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                # UG outputs at this cell (i.e., inputs upstream).
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, ug_max_reach_param + 1):
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
                for L in range(1, ug_max_reach_param + 1):
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
            for L2 in range(1, ug_max_reach_param + 1):
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
                        for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
                        ucx = cx - L * DIR_STEPS[d][0]
                        ucy = cy - L * DIR_STEPS[d][1]
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0)
                # Bound total outflow ≤ is_src_term, total inflow ≤ is_dst_term.
                outflow_terms = [arcs[(cx, cy, facing, e_idx)]]
                for L in range(1, ug_max_reach_param + 1):
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
                for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
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
            for L in range(1, ug_max_reach_param + 1):
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
                        for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
                        ucx = cx - L * DIR_STEPS[d][0]
                        ucy = cy - L * DIR_STEPS[d][1]
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] == 0).OnlyEnforceIf(isp)
                # UG entities can't be at splitter cells (input or output).
                for d in range(4):
                    for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, ug_max_reach_param + 1):
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
            for L2 in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
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
    solver.parameters.max_time_in_seconds = float(req.get("max_time_s", 60.0))
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


def solve_synth_place_dirs(req: dict) -> dict:
    """Phase 3.2D.3 — synth-place with splitter direction freedom.

    Same input as `solve_synth_place` plus `allow_dirs`: a list of
    Factorio directions (0/2/4/6 = N/E/S/W) the solver can pick from.
    Splitter rectangle dimensions become direction-dependent (2×1 for
    N/S, 1×2 for E/W), the second tile's offset depends on direction,
    and routing direction constraints are reified per cell × direction.

    Materially slower than the all-south path: more variables, more
    reification overhead, and CP-SAT has more layout possibilities to
    rule out. Use only for shapes whose library/topology actually
    requires non-south splitters (e.g., (1, 3) with its back-loop).
    """
    n_inputs = req["n_inputs"]
    n_outputs = req["n_outputs"]
    n_splitters = req["n_splitters"]
    edges = req["edges"]
    width, height = req["bounds"]
    factorio_to_internal = {0: 0, 2: 1, 4: 2, 6: 3}
    raw_dirs = req.get("allow_dirs", [4])
    allowed_dirs_internal = sorted({factorio_to_internal[d] for d in raw_dirs})

    if height < 3 or width < max(n_inputs, n_outputs, 2):
        return {"status": "INFEASIBLE", "elapsed_s": 0.0,
                "error": f"bbox {width}×{height} too small"}

    model = cp_model.CpModel()
    sp_y_lo, sp_y_hi = 1, height - 2

    def second_offset(d: int) -> tuple[int, int]:
        return (1, 0) if d in (0, 2) else (0, 1)

    # Splitter position + direction vars.
    anchor_at: dict[tuple[int, int, int], any] = {}
    for s in range(n_splitters):
        cells = []
        for cx in range(width):
            for cy in range(sp_y_lo, sp_y_hi + 1):
                v = model.NewBoolVar(f"a_{s}_{cx}_{cy}")
                anchor_at[(s, cx, cy)] = v
                cells.append(v)
        model.AddExactlyOne(cells)

    dir_at: dict[tuple[int, int], any] = {}
    for s in range(n_splitters):
        dirs = []
        for d in range(4):
            v = model.NewBoolVar(f"d_{s}_{d}")
            dir_at[(s, d)] = v
            if d not in allowed_dirs_internal:
                model.Add(v == 0)
            dirs.append(v)
        model.AddExactlyOne(dirs)

    # Forbid (anchor, dir) where second tile is OOG or in IO row.
    for s in range(n_splitters):
        for cx in range(width):
            for cy in range(sp_y_lo, sp_y_hi + 1):
                for d in allowed_dirs_internal:
                    ofx, ofy = second_offset(d)
                    bx, by = cx + ofx, cy + ofy
                    if not (0 <= bx < width and sp_y_lo <= by <= sp_y_hi):
                        model.AddBoolOr([
                            anchor_at[(s, cx, cy)].Not(),
                            dir_at[(s, d)].Not(),
                        ])

    # Cell-level reified facing predicates.
    def reify_and(name: str, vs: list) -> any:
        b = model.NewBoolVar(name)
        model.AddBoolAnd(vs).OnlyEnforceIf(b)
        model.AddBoolOr([v.Not() for v in vs]).OnlyEnforceIf(b.Not())
        return b

    cell_facing_d: dict[tuple[int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                if d not in allowed_dirs_internal:
                    cell_facing_d[(cx, cy, d)] = 0
                    continue
                terms = []
                for s in range(n_splitters):
                    if (s, cx, cy) in anchor_at:
                        terms.append(reify_and(
                            f"af_{s}_{cx}_{cy}_{d}",
                            [anchor_at[(s, cx, cy)], dir_at[(s, d)]],
                        ))
                    ofx, ofy = second_offset(d)
                    if (s, cx - ofx, cy - ofy) in anchor_at:
                        terms.append(reify_and(
                            f"sf_{s}_{cx}_{cy}_{d}",
                            [anchor_at[(s, cx - ofx, cy - ofy)], dir_at[(s, d)]],
                        ))
                cfd = model.NewBoolVar(f"cf_{cx}_{cy}_{d}")
                if terms:
                    model.Add(sum(terms) == cfd)
                else:
                    model.Add(cfd == 0)
                cell_facing_d[(cx, cy, d)] = cfd

    is_splitter_cell: dict[tuple[int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            isp = model.NewBoolVar(f"isp_{cx}_{cy}")
            terms = [cell_facing_d[(cx, cy, d)] for d in allowed_dirs_internal]
            model.Add(sum(terms) == isp)
            model.Add(isp <= 1)
            is_splitter_cell[(cx, cy)] = isp

    # IO port columns.
    ix = [model.NewIntVar(0, width - 1, f"ix{i}") for i in range(n_inputs)]
    ox = [model.NewIntVar(0, width - 1, f"ox{j}") for j in range(n_outputs)]
    for i in range(n_inputs - 1):
        model.Add(ix[i] < ix[i + 1])
    for j in range(n_outputs - 1):
        model.Add(ox[j] < ox[j + 1])

    input_at, output_at = {}, {}
    for i in range(n_inputs):
        for cx in range(width):
            v = model.NewBoolVar(f"in_{i}_at_{cx}")
            model.Add(ix[i] == cx).OnlyEnforceIf(v)
            model.Add(ix[i] != cx).OnlyEnforceIf(v.Not())
            input_at[(i, cx)] = v
    for j in range(n_outputs):
        for cx in range(width):
            v = model.NewBoolVar(f"out_{j}_at_{cx}")
            model.Add(ox[j] == cx).OnlyEnforceIf(v)
            model.Add(ox[j] != cx).OnlyEnforceIf(v.Not())
            output_at[(j, cx)] = v

    # Slot vars (3.2A.2).
    src_slot_anchor, src_slot_second = {}, {}
    dst_slot_anchor, dst_slot_second = {}, {}
    for e_idx, edge in enumerate(edges):
        if edge["src_kind"] == "Splitter":
            a = model.NewBoolVar(f"sa_e{e_idx}")
            b = model.NewBoolVar(f"ss_e{e_idx}")
            model.AddExactlyOne([a, b])
            src_slot_anchor[e_idx] = a
            src_slot_second[e_idx] = b
        if edge["dst_kind"] == "Splitter":
            a = model.NewBoolVar(f"da_e{e_idx}")
            b = model.NewBoolVar(f"ds_e{e_idx}")
            model.AddExactlyOne([a, b])
            dst_slot_anchor[e_idx] = a
            dst_slot_second[e_idx] = b

    for s in range(n_splitters):
        for users in (
            [src_slot_anchor[e] for e, edge in enumerate(edges)
             if edge["src_kind"] == "Splitter" and edge["src_idx"] == s],
            [src_slot_second[e] for e, edge in enumerate(edges)
             if edge["src_kind"] == "Splitter" and edge["src_idx"] == s],
            [dst_slot_anchor[e] for e, edge in enumerate(edges)
             if edge["dst_kind"] == "Splitter" and edge["dst_idx"] == s],
            [dst_slot_second[e] for e, edge in enumerate(edges)
             if edge["dst_kind"] == "Splitter" and edge["dst_idx"] == s],
        ):
            if len(users) >= 2:
                model.AddAtMostOne(users)

    def is_src_term_at(cx: int, cy: int, e_idx: int):
        edge = edges[e_idx]
        if edge["src_kind"] == "InputPort":
            return input_at[(edge["src_idx"], cx)] if cy == 0 else 0
        s = edge["src_idx"]
        terms = []
        if (s, cx, cy) in anchor_at:
            terms.append(reify_and(
                f"src_a_{s}_{cx}_{cy}_e{e_idx}",
                [anchor_at[(s, cx, cy)], src_slot_anchor[e_idx]],
            ))
        for d in allowed_dirs_internal:
            ofx, ofy = second_offset(d)
            if (s, cx - ofx, cy - ofy) in anchor_at:
                terms.append(reify_and(
                    f"src_s_{s}_{cx}_{cy}_d{d}_e{e_idx}",
                    [anchor_at[(s, cx - ofx, cy - ofy)], dir_at[(s, d)],
                     src_slot_second[e_idx]],
                ))
        return sum(terms) if terms else 0

    def is_dst_term_at(cx: int, cy: int, e_idx: int):
        edge = edges[e_idx]
        if edge["dst_kind"] == "OutputPort":
            return output_at[(edge["dst_idx"], cx)] if cy == height - 1 else 0
        s = edge["dst_idx"]
        terms = []
        if (s, cx, cy) in anchor_at:
            terms.append(reify_and(
                f"dst_a_{s}_{cx}_{cy}_e{e_idx}",
                [anchor_at[(s, cx, cy)], dst_slot_anchor[e_idx]],
            ))
        for d in allowed_dirs_internal:
            ofx, ofy = second_offset(d)
            if (s, cx - ofx, cy - ofy) in anchor_at:
                terms.append(reify_and(
                    f"dst_s_{s}_{cx}_{cy}_d{d}_e{e_idx}",
                    [anchor_at[(s, cx - ofx, cy - ofy)], dir_at[(s, d)],
                     dst_slot_second[e_idx]],
                ))
        return sum(terms) if terms else 0

    # Routing arcs + UG arcs (UGs in all 4 directions to match direction freedom).
    arcs = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                for e_idx in range(len(edges)):
                    arcs[(cx, cy, d, e_idx)] = model.NewBoolVar(f"a_{cx}_{cy}_{d}_{e_idx}")
    ug_arcs = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                dx, dy = DIR_STEPS[d]
                for L in range(1, ug_max_reach_param + 1):
                    ncx, ncy = cx + L * dx, cy + L * dy
                    if 0 <= ncx < width and 0 <= ncy < height:
                        for e_idx in range(len(edges)):
                            ug_arcs[(cx, cy, d, L, e_idx)] = model.NewBoolVar(
                                f"u_{cx}_{cy}_{d}_{L}_{e_idx}")

    # No arcs leaving grid + at-most-one direction per (cell, edge).
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                ncx, ncy = cx + DIR_STEPS[d][0], cy + DIR_STEPS[d][1]
                if not (0 <= ncx < width and 0 <= ncy < height):
                    for e_idx in range(len(edges)):
                        model.Add(arcs[(cx, cy, d, e_idx)] == 0)
            for e_idx in range(len(edges)):
                model.AddAtMostOne([arcs[(cx, cy, d, e_idx)] for d in range(4)])

    # Splitter cell direction + transit constraints, reified per facing.
    for cx in range(width):
        for cy in range(height):
            isp = is_splitter_cell[(cx, cy)]
            for d_facing in allowed_dirs_internal:
                cfd = cell_facing_d[(cx, cy, d_facing)]
                for e_idx in range(len(edges)):
                    for d_other in range(4):
                        if d_other == d_facing:
                            continue
                        model.Add(arcs[(cx, cy, d_other, e_idx)] + cfd <= 1)
                        for L in range(1, ug_max_reach_param + 1):
                            if (cx, cy, d_other, L, e_idx) in ug_arcs:
                                model.Add(ug_arcs[(cx, cy, d_other, L, e_idx)] + cfd <= 1)
                        ncx = cx - DIR_STEPS[d_other][0]
                        ncy = cy - DIR_STEPS[d_other][1]
                        if 0 <= ncx < width and 0 <= ncy < height:
                            model.Add(arcs[(ncx, ncy, d_other, e_idx)] + cfd <= 1)
                        for L in range(1, ug_max_reach_param + 1):
                            ucx = cx - L * DIR_STEPS[d_other][0]
                            ucy = cy - L * DIR_STEPS[d_other][1]
                            if (ucx, ucy, d_other, L, e_idx) in ug_arcs:
                                model.Add(ug_arcs[(ucx, ucy, d_other, L, e_idx)] + cfd <= 1)
            # No UG entities at splitter cells.
            for e_idx in range(len(edges)):
                for d in range(4):
                    for L in range(1, ug_max_reach_param + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(cx, cy, d, L, e_idx)] + isp <= 1)
                        dx, dy = DIR_STEPS[d]
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            model.Add(ug_arcs[(ucx, ucy, d, L, e_idx)] + isp <= 1)

    # At most one entity per non-splitter cell.
    for cx in range(width):
        for cy in range(height):
            isp = is_splitter_cell[(cx, cy)]
            terms = []
            for e_idx in range(len(edges)):
                terms.append(sum(arcs[(cx, cy, d, e_idx)] for d in range(4)))
                for d in range(4):
                    for L in range(1, ug_max_reach_param + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(1, ug_max_reach_param + 1):
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
            model.Add(sum(terms) <= 1).OnlyEnforceIf(isp.Not())

    # UG pairing rule.
    for (c1x, c1y, d1, L1, e1), arc1 in ug_arcs.items():
        dx, dy = DIR_STEPS[d1]
        for k in range(1, L1):
            mcx, mcy = c1x + k * dx, c1y + k * dy
            for L2 in range(1, ug_max_reach_param + 1):
                ucx, ucy = mcx - L2 * dx, mcy - L2 * dy
                if not (0 <= ucx < width and 0 <= ucy < height):
                    continue
                for e2 in range(len(edges)):
                    if (ucx, ucy, d1, L2, e2) in ug_arcs and (ucx, ucy, d1, L2, e2) != (c1x, c1y, d1, L1, e1):
                        model.AddBoolOr([arc1.Not(), ug_arcs[(ucx, ucy, d1, L2, e2)].Not()])

    # Conservation.
    for cx in range(width):
        for cy in range(height):
            for e_idx in range(len(edges)):
                belt_outflow = sum(arcs[(cx, cy, d, e_idx)] for d in range(4))
                ug_outflow_terms = []
                for d in range(4):
                    for L in range(1, ug_max_reach_param + 1):
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
                    for L in range(1, ug_max_reach_param + 1):
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

    # Objective: minimize entity count.
    entity_terms = []
    for var in arcs.values():
        entity_terms.append(var)
    for var in ug_arcs.values():
        entity_terms.append(var)
        entity_terms.append(var)
    model.Minimize(sum(entity_terms))

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = float(req.get("max_time_s", 120.0))
    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    out: dict = {"status": solver.StatusName(status), "elapsed_s": elapsed}
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return out

    splitters_out = []
    for s in range(n_splitters):
        anchor_cell = next(((cx, cy) for (ss, cx, cy), v in anchor_at.items()
                            if ss == s and solver.Value(v) == 1), None)
        chosen_d = next((d for d in range(4) if solver.Value(dir_at[(s, d)]) == 1), None)
        ax, ay = anchor_cell
        splitters_out.append({"x": ax, "y": ay, "dir": INTERNAL_TO_FACTORIO_DIR[chosen_d]})

    splitter_cells: set[tuple[int, int]] = set()
    for sp in splitters_out:
        ax, ay = sp["x"], sp["y"]
        chosen_d_internal = factorio_to_internal[sp["dir"]]
        ofx, ofy = second_offset(chosen_d_internal)
        splitter_cells.add((ax, ay))
        splitter_cells.add((ax + ofx, ay + ofy))

    out["splitters"] = splitters_out
    out["input_port_tiles"] = [[solver.Value(ix[i]), 0] for i in range(n_inputs)]
    out["output_port_tiles"] = [[solver.Value(ox[j]), height - 1] for j in range(n_outputs)]

    belts, ugs = [], []
    for (cx, cy, d, L, e_idx), arc_var in ug_arcs.items():
        if solver.Value(arc_var) == 1:
            fdir = INTERNAL_TO_FACTORIO_DIR[d]
            ugs.append({"x": cx, "y": cy, "dir": fdir, "io": "input", "edge_idx": e_idx})
            dx, dy = DIR_STEPS[d]
            ugs.append({"x": cx + L * dx, "y": cy + L * dy, "dir": fdir, "io": "output", "edge_idx": e_idx})

    ug_cells = {(u["x"], u["y"]) for u in ugs}
    for cx in range(width):
        for cy in range(height):
            if (cx, cy) in splitter_cells or (cx, cy) in ug_cells:
                continue
            emitted = False
            for d in range(4):
                for e_idx in range(len(edges)):
                    if solver.Value(arcs[(cx, cy, d, e_idx)]) == 1:
                        belts.append({"x": cx, "y": cy,
                                      "dir": INTERNAL_TO_FACTORIO_DIR[d], "edge_idx": e_idx})
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
                        belts.append({"x": cx, "y": cy,
                                      "dir": INTERNAL_TO_FACTORIO_DIR[d], "edge_idx": e_idx})
                        emitted = True
                        break
                if emitted:
                    break

    out["belts"] = belts
    out["ugs"] = ugs
    return out


def solve_pure_routing(req: dict) -> dict:
    """Phase 4.2 — belt/UG routing between fixed IO tiles, no splitters.

    Used by the composition combinator (`compose_series`) to route the
    inter-stage permutation. Strips out everything from `solve_routing`
    that's about splitters (slot vars, splitter-cell direction
    constraints, splitter-cell exclusions): every edge has
    `src_kind=InputPort` and `dst_kind=OutputPort` with constant cells,
    so conservation is a fixed integer comparison.

    Adds the same `Minimize(arcs + 2*ugs)` objective as Mode D so paths
    pull tight and UGs are only chosen when they save cells.

    Request fields: `bounds`, `input_port_tiles`, `output_port_tiles`,
    `edges` (each `{src_kind: "InputPort", src_idx, dst_kind: "OutputPort",
    dst_idx}`), optional `max_time_s`.
    """
    width, height = req["bounds"]
    input_port_tiles = [tuple(t) for t in req["input_port_tiles"]]
    output_port_tiles = [tuple(t) for t in req["output_port_tiles"]]
    edges = req["edges"]
    # ug_max_reach_param: transit-tile reach for this request.
    # Defaults to 4 (yellow belt) which matches the legacy UG_MAX_REACH = 5
    # constant (L_max_in_loop = ug_max_reach_param + 1 = 5, same as before).
    # Pass 6 for red or 8 for blue to allow longer underground belt spans.
    ug_max_reach_param: int = req.get("ug_max_reach", 4)

    edge_src: list[tuple[int, int]] = []
    edge_dst: list[tuple[int, int]] = []
    for edge in edges:
        if edge["src_kind"] != "InputPort":
            return {"status": "INVALID", "elapsed_s": 0.0,
                    "error": f"pure routing: src must be InputPort, got {edge['src_kind']}"}
        if edge["dst_kind"] != "OutputPort":
            return {"status": "INVALID", "elapsed_s": 0.0,
                    "error": f"pure routing: dst must be OutputPort, got {edge['dst_kind']}"}
        edge_src.append(input_port_tiles[edge["src_idx"]])
        edge_dst.append(output_port_tiles[edge["dst_idx"]])

    model = cp_model.CpModel()

    arcs: dict[tuple[int, int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                for e_idx in range(len(edges)):
                    arcs[(cx, cy, d, e_idx)] = model.NewBoolVar(
                        f"a_{cx}_{cy}_{d}_{e_idx}"
                    )

    ug_arcs: dict[tuple[int, int, int, int, int], any] = {}
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                dx, dy = DIR_STEPS[d]
                for L in range(2, ug_max_reach_param + 1):
                    ncx, ncy = cx + L * dx, cy + L * dy
                    if 0 <= ncx < width and 0 <= ncy < height:
                        for e_idx in range(len(edges)):
                            ug_arcs[(cx, cy, d, L, e_idx)] = model.NewBoolVar(
                                f"u_{cx}_{cy}_{d}_{L}_{e_idx}"
                            )

    # No arcs leaving the grid — except south at the bottom row, which
    # represents the junction's exit into stage2's input belts. The dst
    # cell is forced to a south-facing belt below; that belt's south arc
    # leaves the grid and counts as the per-edge outflow that balances the
    # inflow from above.
    for cx in range(width):
        for cy in range(height):
            for d in range(4):
                ncx, ncy = cx + DIR_STEPS[d][0], cy + DIR_STEPS[d][1]
                if not (0 <= ncx < width and 0 <= ncy < height):
                    if d == 2 and cy == height - 1:
                        continue  # south exit allowed at the bottom row.
                    for e_idx in range(len(edges)):
                        model.Add(arcs[(cx, cy, d, e_idx)] == 0)
    # At most one outgoing direction per (cell, edge).
    for cx in range(width):
        for cy in range(height):
            for e_idx in range(len(edges)):
                model.AddAtMostOne([arcs[(cx, cy, d, e_idx)] for d in range(4)])

    # Force a south-facing belt at every dst cell — this is the belt that
    # physically passes flow out of the junction into stage2's input belt
    # one row below. Without this, the dst cell ends up empty (or wrong-
    # facing) and the recovered topology drops the splitter→splitter edge.
    for e_idx, dst in enumerate(edge_dst):
        dx, dy = dst
        model.Add(arcs[(dx, dy, 2, e_idx)] == 1)

    # At most one entity per cell (pure routing — no splitters anywhere).
    for cx in range(width):
        for cy in range(height):
            terms = []
            for e_idx in range(len(edges)):
                terms.append(sum(arcs[(cx, cy, d, e_idx)] for d in range(4)))
                for d in range(4):
                    for L in range(2, ug_max_reach_param + 1):
                        if (cx, cy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(cx, cy, d, L, e_idx)])
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(2, ug_max_reach_param + 1):
                        ucx, ucy = cx - L * dx, cy - L * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
            model.Add(sum(terms) <= 1)

    # UG pairing rule (matches Mode B / Mode D): forbid configurations
    # where one UG's intermediate transit cell coincides with another's
    # output, since Factorio would re-pair them in-game.
    for (c1x, c1y, d1, L1, e1), arc1 in ug_arcs.items():
        dx, dy = DIR_STEPS[d1]
        for k in range(1, L1):
            mcx, mcy = c1x + k * dx, c1y + k * dy
            for L2 in range(2, ug_max_reach_param + 1):
                ucx, ucy = mcx - L2 * dx, mcy - L2 * dy
                if not (0 <= ucx < width and 0 <= ucy < height):
                    continue
                for e2 in range(len(edges)):
                    key = (ucx, ucy, d1, L2, e2)
                    if key in ug_arcs and key != (c1x, c1y, d1, L1, e1):
                        model.AddBoolOr([arc1.Not(), ug_arcs[key].Not()])

    # Conservation per (cell, edge), fixed IO positions.
    for cx in range(width):
        for cy in range(height):
            for e_idx in range(len(edges)):
                belt_outflow = sum(arcs[(cx, cy, d, e_idx)] for d in range(4))
                ug_outflow_terms = []
                for d in range(4):
                    for L in range(2, ug_max_reach_param + 1):
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

                # UG inflow lands one cell ahead of the UG output (matches
                # Mode B / Mode D semantics; see 3.2A.2 fix).
                ug_inflow_terms = []
                for d in range(4):
                    dx, dy = DIR_STEPS[d]
                    for L in range(2, ug_max_reach_param + 1):
                        ucx = cx - (L + 1) * dx
                        ucy = cy - (L + 1) * dy
                        if (ucx, ucy, d, L, e_idx) in ug_arcs:
                            ug_inflow_terms.append(ug_arcs[(ucx, ucy, d, L, e_idx)])
                ug_inflow = sum(ug_inflow_terms) if ug_inflow_terms else 0

                outflow = belt_outflow + ug_outflow
                inflow = belt_inflow + ug_inflow

                # Conservation: outflow == inflow + (1 if src else 0).
                # The dst cell's south-facing belt (forced above) emits its
                # south arc off the bottom of the grid (allowed at y=jh-1),
                # which counts as outflow and balances the inflow from
                # above — so dst cells need no special-case.
                is_src = (cx, cy) == edge_src[e_idx]
                if is_src:
                    model.Add(outflow - inflow == 1)
                else:
                    model.Add(outflow - inflow == 0)

    # Objective: when "minimize_entities" is set, minimise total entity
    # count (belt arcs + 2× UG arcs since UGs are 2 entities). Default for
    # the composition baker is feasibility-only — the first valid layout is
    # accepted. Skipping the optimisation loop is the difference between
    # a 600-second jh=N solve and a ~50-second one. Fatness in the junction
    # is acceptable; the consumer (`compose_series`) just stamps whatever
    # the solver returns.
    if req.get("minimize_entities", False):
        entity_terms = []
        for var in arcs.values():
            entity_terms.append(var)
        for var in ug_arcs.values():
            entity_terms.append(var)
            entity_terms.append(var)
        model.Minimize(sum(entity_terms))

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = float(req.get("max_time_s", 30.0))
    # Stop on first feasible when no objective is set. With Minimize() this
    # parameter is ignored — the solver always pursues optimality.
    solver.parameters.stop_after_first_solution = not req.get("minimize_entities", False)
    solver.parameters.random_seed = _get_cp_sat_seed(req)
    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    out: dict = {"status": solver.StatusName(status), "elapsed_s": elapsed}
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return out

    belts = []
    ugs = []
    for (cx, cy, d, L, e_idx), arc_var in ug_arcs.items():
        if solver.Value(arc_var) == 1:
            fdir = INTERNAL_TO_FACTORIO_DIR[d]
            ugs.append({"x": cx, "y": cy, "dir": fdir, "io": "input", "edge_idx": e_idx})
            dx, dy = DIR_STEPS[d]
            ugs.append({"x": cx + L * dx, "y": cy + L * dy, "dir": fdir,
                        "io": "output", "edge_idx": e_idx})

    ug_cells = {(u["x"], u["y"]) for u in ugs}
    for cx in range(width):
        for cy in range(height):
            if (cx, cy) in ug_cells:
                continue
            emitted = False
            for d in range(4):
                for e_idx in range(len(edges)):
                    if solver.Value(arcs[(cx, cy, d, e_idx)]) == 1:
                        belts.append({"x": cx, "y": cy,
                                      "dir": INTERNAL_TO_FACTORIO_DIR[d],
                                      "edge_idx": e_idx})
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
                        belts.append({"x": cx, "y": cy,
                                      "dir": INTERNAL_TO_FACTORIO_DIR[d],
                                      "edge_idx": e_idx})
                        emitted = True
                        break
                if emitted:
                    break

    out["belts"] = belts
    out["ugs"] = ugs
    return out


def solve_pure_routing_circuit(req: dict) -> dict:
    """Phase 4.2 alternative — same problem as `solve_pure_routing` but
    uses CP-SAT's `AddCircuit` constraint instead of per-(cell, edge, dir)
    bool vars + explicit conservation. CP-SAT has dedicated propagation
    for circuit constraints (cycle detection, reachability bounds) which
    can be much faster than generic AC-3 over flow conservation.

    Per-edge model: build a directed graph where nodes are cells (plus a
    virtual `exit` node), arcs are belt-moves (cell → adjacent cell) and
    UG-jumps (cell → cell + L*step). Each cell that may be off-path gets
    a self-loop arc with literal `not_on_path`. The src and dst nodes
    have no self-loops — they MUST be on the path. A virtual closing arc
    `dst → src` (literal pinned to 1) closes the circuit.

    The forced south-belt at dst is recorded as an entity in the result,
    not as an arc — the arc into dst is whichever real arc the circuit
    selected; the south-belt is the additional exit-the-junction entity.

    Cross-edge constraints (at-most-one entity per cell, UG pairing) are
    posted on top of the per-edge circuits.

    Spatial pruning (RFP `docs/rfp-balancer-spatial-pruning.md`): per-edge
    variables are restricted to cells inside a Manhattan-ellipse around
    (src, dst). slack defaults to `bounds height + 2` (the bake passes
    height = junction_height; this matches "jh + 2"). Override with
    `req["routing_slack"]` (an int) or pass JSON null to disable pruning
    entirely.

    Fallback: the RFP originally specified a post-INFEASIBLE retry with
    full encoding to guard against false-INFEASIBLE from the heuristic.
    Verification (Decision log entry 2026-05-02) showed the retry never
    rescues anything in the bake context — `compose_series` in main.rs
    treats INFEASIBLE and UNKNOWN identically (both bump `jh`), so a
    pruned-INFEASIBLE just advances the bake's outer loop, which is the
    real safety net. The fallback meanwhile doubled wall time at every
    infeasible jh and tripped kill criterion #3. Default is therefore
    fallback OFF; opt back in by setting FUCKTORIO_ROUTING_FALLBACK=1
    (paranoia mode for one-off requests where +1 row of jh is more
    expensive than the re-solve).
    """
    width, height = req["bounds"]
    # Default slack = jh + 2 (RFP §design "Initial implementation").
    default_slack = height + 2
    slack_arg = req.get("routing_slack", default_slack)

    out = _solve_pure_routing_circuit_inner(req, slack=slack_arg)
    # Fallback is OFF by default; opt in via FUCKTORIO_ROUTING_FALLBACK=1.
    # See docstring above + RFP decision log for why.
    fallback_enabled = os.environ.get("FUCKTORIO_ROUTING_FALLBACK") == "1"
    if (out.get("status") == "INFEASIBLE"
            and slack_arg is not None
            and fallback_enabled):
        print(
            f"  circuit: pruned solve INFEASIBLE at slack={slack_arg} — "
            "retrying with full encoding (FUCKTORIO_ROUTING_FALLBACK=1)",
            file=sys.stderr,
        )
        out = _solve_pure_routing_circuit_inner(req, slack=None)
    return out


def _solve_pure_routing_circuit_inner(req: dict, slack) -> dict:
    """Inner helper: build and solve the CP-SAT circuit model with
    optional spatial pruning.

    `slack=None` disables pruning (full encoding, every cell in-corridor
    for every edge). Otherwise `slack` is a non-negative int and the
    per-edge corridor mask restricts variable creation to cells with
    `manhattan((cx, cy), src) + manhattan((cx, cy), dst)
        <= manhattan(src, dst) + slack`.
    """
    width, height = req["bounds"]
    input_port_tiles = [tuple(t) for t in req["input_port_tiles"]]
    output_port_tiles = [tuple(t) for t in req["output_port_tiles"]]
    edges = req["edges"]
    n_edges = len(edges)
    # ug_max_reach_param: transit-tile reach for this request (same semantics
    # as solve_pure_routing). Defaults to 4 (yellow) = legacy UG_MAX_REACH - 1.
    ug_max_reach_param: int = req.get("ug_max_reach", 4)

    edge_src: list[tuple[int, int]] = []
    edge_dst: list[tuple[int, int]] = []
    for edge in edges:
        if edge["src_kind"] != "InputPort" or edge["dst_kind"] != "OutputPort":
            return {"status": "INVALID", "elapsed_s": 0.0,
                    "error": "circuit pure routing: edges must be InputPort→OutputPort"}
        edge_src.append(input_port_tiles[edge["src_idx"]])
        edge_dst.append(output_port_tiles[edge["dst_idx"]])

    # Per-edge corridor masks. Out-of-corridor cells get only a
    # self-loop arc (no belt/UG arcs), so the solver can skip them in
    # the circuit and doesn't propagate conservation over dead vars.
    edge_corridors: list[set[tuple[int, int]]] = []
    for e_idx in range(n_edges):
        sx, sy = edge_src[e_idx]
        dx_, dy_ = edge_dst[e_idx]
        if slack is None:
            mask = {(cx, cy) for cy in range(height) for cx in range(width)}
        else:
            base = abs(sx - dx_) + abs(sy - dy_)
            threshold = base + slack
            mask = set()
            for cy in range(height):
                for cx in range(width):
                    if (abs(cx - sx) + abs(cy - sy)
                            + abs(cx - dx_) + abs(cy - dy_)) <= threshold:
                        mask.add((cx, cy))
            # Defensive: src and dst are foci of the ellipse and are
            # already in-corridor for slack >= 0, but make this explicit
            # so a degenerate slack value can't accidentally exclude
            # them. The forced south-belt at dst depends on it.
            mask.add((sx, sy))
            mask.add((dx_, dy_))
        edge_corridors.append(mask)

    model = cp_model.CpModel()

    def cell_id(x: int, y: int) -> int:
        return y * width + x

    # Virtual exit node — one per edge would also work, but circuits are
    # per-edge constraints so we can reuse the same id across edges.
    EXIT_NODE = width * height

    # Per-edge variable maps. We need them by (cell, dir, e) etc. for
    # post-processing into belts/ugs. Storing the bool var alongside the
    # circuit arc list keeps the solver's encoding aligned with what we
    # extract afterward.
    belt_vars: dict[tuple[int, int, int, int], any] = {}
    ug_vars: dict[tuple[int, int, int, int, int], any] = {}
    self_loop_vars: dict[tuple[int, int, int], any] = {}

    gen_ugs = not req.get("debug_no_ugs", False)

    for e_idx in range(n_edges):
        sx, sy = edge_src[e_idx]
        dx_, dy_ = edge_dst[e_idx]
        src_node = cell_id(sx, sy)
        dst_node = cell_id(dx_, dy_)
        corridor = edge_corridors[e_idx]

        arc_list = []  # (tail_node, head_node, literal)

        # Count what would have been generated unpruned (so the debug
        # log can show pruning ratio). Cheap relative to var creation.
        belt_potential = 0
        ug_potential = 0
        for cy in range(height):
            for cx in range(width):
                for d in range(4):
                    sdx, sdy = DIR_STEPS[d]
                    nx, ny = cx + sdx, cy + sdy
                    if 0 <= nx < width and 0 <= ny < height:
                        belt_potential += 1
                    elif d == 2 and (cx, cy) == (dx_, dy_):
                        belt_potential += 1
                    if not gen_ugs:
                        continue
                    for L in range(2, ug_max_reach_param + 1):
                        bx, by = cx + L * sdx, cy + L * sdy
                        cx_c, cy_c = cx + (L + 1) * sdx, cy + (L + 1) * sdy
                        if not (0 <= bx < width and 0 <= by < height):
                            continue
                        in_bounds_c = (0 <= cx_c < width and 0 <= cy_c < height)
                        is_dst_exit_pot = (
                            d == 2 and bx == dx_ and by == dy_
                            and not in_bounds_c
                        )
                        if not in_bounds_c and not is_dst_exit_pot:
                            continue
                        ug_potential += 1

        # Belt arcs: cell → in-bounds neighbor in each direction. Both
        # endpoints must be in-corridor (cells outside the ellipse can
        # never lie on a path that respects the corridor; arcs into
        # them are dead). The dst's south arc is special — it's a real
        # belt facing south (the junction-exit), and the arc head is
        # the virtual exit node so the circuit can close via
        # exit → src. Dst is always in-corridor (it's a focus), so the
        # forced south-belt is always emitted.
        for cy in range(height):
            for cx in range(width):
                if (cx, cy) not in corridor:
                    continue
                for d in range(4):
                    sdx, sdy = DIR_STEPS[d]
                    nx, ny = cx + sdx, cy + sdy
                    if 0 <= nx < width and 0 <= ny < height:
                        if (nx, ny) not in corridor:
                            continue
                        var = model.NewBoolVar(f"b{cx}_{cy}_{d}_e{e_idx}")
                        belt_vars[(cx, cy, d, e_idx)] = var
                        arc_list.append((cell_id(cx, cy), cell_id(nx, ny), var))
                    elif d == 2 and (cx, cy) == (dx_, dy_):
                        # Forced south belt at dst; goes off-grid into
                        # the virtual exit node. Pinned to 1.
                        var = model.NewBoolVar(f"b{cx}_{cy}_{d}_e{e_idx}")
                        belt_vars[(cx, cy, d, e_idx)] = var
                        model.Add(var == 1)
                        arc_list.append((cell_id(cx, cy), EXIT_NODE, var))

        # UG arcs: per-Factorio semantics, the UG-input is at A, the
        # UG-output entity sits at B = A + L*step, and the flow lands
        # one cell *beyond* B at C = A + (L+1)*step. In the circuit
        # graph the arc therefore hops A → C — the B cell is consumed
        # as an entity (counted in at-most-one as a UG-output landing)
        # but isn't itself part of the path. Modelling A → B directly
        # would force B to also have an out-arc (a belt or another UG),
        # which collides with the UG-output entity at B in at-most-one.
        # Skip L=1 — equivalent to a belt, never picked when minimizing
        # entities.
        #
        # Pruning: BOTH A and C must be in-corridor for the arc to
        # exist. The dst-exit case (d==2, B==dst, C off-grid) is always
        # allowed unconditionally — dst is a focus of the corridor and
        # the EXIT_NODE is logically downstream.
        if gen_ugs:
            for cy in range(height):
                for cx in range(width):
                    if (cx, cy) not in corridor:
                        continue
                    for d in range(4):
                        sdx, sdy = DIR_STEPS[d]
                        for L in range(2, ug_max_reach_param + 1):
                            bx, by = cx + L * sdx, cy + L * sdy
                            cx_c, cy_c = cx + (L + 1) * sdx, cy + (L + 1) * sdy
                            if not (0 <= bx < width and 0 <= by < height):
                                continue
                            in_bounds_c = (0 <= cx_c < width and 0 <= cy_c < height)
                            is_dst_landing = (cx_c, cy_c) == (dx_, dy_) and in_bounds_c
                            # Allow C off-grid only when the UG would
                            # land flow at the south exit beneath dst.
                            is_dst_exit = (
                                d == 2 and bx == dx_ and by == dy_
                                and not in_bounds_c
                            )
                            if not in_bounds_c and not is_dst_exit:
                                continue
                            # Corridor check on C. Dst-exit is always
                            # in-corridor (head is EXIT_NODE).
                            if not is_dst_exit and (cx_c, cy_c) not in corridor:
                                continue
                            var = model.NewBoolVar(f"u{cx}_{cy}_{d}_{L}_e{e_idx}")
                            ug_vars[(cx, cy, d, L, e_idx)] = var
                            head = EXIT_NODE if is_dst_exit else cell_id(cx_c, cy_c)
                            arc_list.append((cell_id(cx, cy), head, var))
                            _ = is_dst_landing  # kept for readability

        # Self-loops for cells that may be off-path. src and dst are
        # forced on-path by omitting their self-loops. Out-of-corridor
        # cells DO get a self-loop — AddCircuit needs every node to
        # either be visited or self-looped, and out-of-corridor cells
        # have no other arcs so the self-loop is their only option.
        for cy in range(height):
            for cx in range(width):
                if (cx, cy) == (sx, sy) or (cx, cy) == (dx_, dy_):
                    continue
                var = model.NewBoolVar(f"sl{cx}_{cy}_e{e_idx}")
                self_loop_vars[(cx, cy, e_idx)] = var
                node = cell_id(cx, cy)
                arc_list.append((node, node, var))

        # Closing arc: exit_node → src, literal pinned to true. Closes
        # the cycle (dst → exit via south-belt, exit → src here).
        closing_lit = model.NewBoolVar(f"close_e{e_idx}")
        model.Add(closing_lit == 1)
        arc_list.append((EXIT_NODE, src_node, closing_lit))

        # Debug: log arc counts before adding the constraint. With
        # pruning, also report the unpruned-potential numbers so the
        # reduction ratio is visible per edge.
        sl_count = sum(1 for (a, b, _) in arc_list if a == b)
        belt_arc_count = sum(
            1 for (cx, cy, d, ee) in belt_vars.keys() if ee == e_idx
        )
        ug_arc_count = sum(
            1 for k in ug_vars.keys() if k[4] == e_idx
        )
        if slack is None:
            print(
                f"  circuit edge_idx={e_idx} src={(sx, sy)} dst={(dx_, dy_)} "
                f"arcs={len(arc_list)} belts={belt_arc_count} ugs={ug_arc_count} "
                f"self_loops={sl_count} closing=1 (no pruning)",
                file=sys.stderr,
            )
        else:
            print(
                f"  circuit edge_idx={e_idx} pruned slack={slack}: "
                f"belts={belt_arc_count}/{belt_potential} "
                f"ugs={ug_arc_count}/{ug_potential} "
                f"src={(sx, sy)} dst={(dx_, dy_)} "
                f"arcs={len(arc_list)} self_loops={sl_count}",
                file=sys.stderr,
            )
        model.AddCircuit(arc_list)

    debug_no_atmost_one = req.get("debug_no_atmost_one", False)
    debug_no_ug_arrival = req.get("debug_no_ug_arrival", False)
    debug_no_ug_pairing = req.get("debug_no_ug_pairing", False)
    if debug_no_atmost_one:
        print("  circuit DEBUG: at-most-one disabled", file=sys.stderr)
    if debug_no_ug_arrival:
        print("  circuit DEBUG: UG arrival in at-most-one disabled", file=sys.stderr)
    if debug_no_ug_pairing:
        print("  circuit DEBUG: UG pairing rule disabled", file=sys.stderr)
    # At-most-one entity per cell across edges. An entity is:
    #   - belt outgoing from this cell (any of 4 directions, any edge);
    #   - UG-input at this cell;
    #   - UG-output landing at this cell;
    #   - forced south-belt at this cell (dst of some edge).
    for cy in range(height):
        for cx in range(width):
            terms = []
            for e_idx in range(n_edges):
                for d in range(4):
                    if (cx, cy, d, e_idx) in belt_vars:
                        terms.append(belt_vars[(cx, cy, d, e_idx)])
                for d in range(4):
                    sdx, sdy = DIR_STEPS[d]
                    for L in range(2, ug_max_reach_param + 1):
                        if (cx, cy, d, L, e_idx) in ug_vars:
                            terms.append(ug_vars[(cx, cy, d, L, e_idx)])
                        # UG outputs land at (cx, cy) when input was at
                        # (cx - L*step). Each such input arc-var counts.
                        ux, uy = cx - L * sdx, cy - L * sdy
                        if (ux, uy, d, L, e_idx) in ug_vars and not debug_no_ug_arrival:
                            terms.append(ug_vars[(ux, uy, d, L, e_idx)])
            if terms and not debug_no_atmost_one:
                model.Add(sum(terms) <= 1)

    # UG pairing: forbid configurations where one UG's transit cell
    # coincides with another UG's exit (Factorio re-pairs them).
    if debug_no_ug_pairing:
        pass
    else:
      for (c1x, c1y, d1, L1, e1), arc1 in ug_vars.items():
        sdx, sdy = DIR_STEPS[d1]
        for k in range(1, L1):
            mcx, mcy = c1x + k * sdx, c1y + k * sdy
            for L2 in range(2, ug_max_reach_param + 1):
                ucx, ucy = mcx - L2 * sdx, mcy - L2 * sdy
                if not (0 <= ucx < width and 0 <= ucy < height):
                    continue
                for e2 in range(n_edges):
                    key = (ucx, ucy, d1, L2, e2)
                    if key in ug_vars and key != (c1x, c1y, d1, L1, e1):
                        model.AddBoolOr([arc1.Not(), ug_vars[key].Not()])

    # Optional objective. Default feasibility-only; minimize entity count
    # if requested.
    if req.get("minimize_entities", False):
        ent_terms = []
        for var in belt_vars.values():
            ent_terms.append(var)
        for var in ug_vars.values():
            ent_terms.append(var)
            ent_terms.append(var)
        model.Minimize(sum(ent_terms))

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = float(req.get("max_time_s", 30.0))
    solver.parameters.stop_after_first_solution = not req.get("minimize_entities", False)
    solver.parameters.random_seed = _get_cp_sat_seed(req)
    if req.get("debug_no_probing", False):
        solver.parameters.cp_model_probing_level = 0
        print("  circuit DEBUG: probing disabled", file=sys.stderr)
    if req.get("debug_log_progress", False):
        solver.parameters.log_search_progress = True
        solver.parameters.log_to_stdout = False
        solver.log_callback = lambda msg: print(f"  [cp-sat] {msg}", file=sys.stderr)

    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    out: dict = {"status": solver.StatusName(status), "elapsed_s": elapsed}
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return out

    # Extract belts and UGs from selected vars.
    belts = []
    ugs = []
    for (cx, cy, d, L, e_idx), arc_var in ug_vars.items():
        if solver.Value(arc_var) == 1:
            fdir = INTERNAL_TO_FACTORIO_DIR[d]
            ugs.append({"x": cx, "y": cy, "dir": fdir, "io": "input", "edge_idx": e_idx})
            sdx, sdy = DIR_STEPS[d]
            ugs.append({"x": cx + L * sdx, "y": cy + L * sdy, "dir": fdir,
                        "io": "output", "edge_idx": e_idx})

    ug_cells = {(u["x"], u["y"]) for u in ugs}
    for (cx, cy, d, e_idx), arc_var in belt_vars.items():
        if (cx, cy) in ug_cells:
            continue
        if solver.Value(arc_var) == 1:
            belts.append({"x": cx, "y": cy,
                          "dir": INTERNAL_TO_FACTORIO_DIR[d],
                          "edge_idx": e_idx})

    out["belts"] = belts
    out["ugs"] = ugs
    return out


def main() -> None:
    req = json.load(sys.stdin)
    if req.get("kind") == "pure_routing":
        if req.get("encoding") == "circuit":
            out = solve_pure_routing_circuit(req)
        else:
            out = solve_pure_routing(req)
    elif "edges" in req and "splitter_positions" in req:
        out = solve_routing(req)
    elif "edges" in req and "n_splitters" in req:
        if "allow_dirs" in req and req["allow_dirs"] != [4]:
            out = solve_synth_place_dirs(req)
        else:
            out = solve_synth_place(req)
    else:
        out = solve_overlap_only(req)
    json.dump(out, sys.stdout)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
