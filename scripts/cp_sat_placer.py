#!/usr/bin/env -S uv run --no-project
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "ortools>=9.10",
# ]
# ///
"""CP-SAT placement engine — reads a BalancerGraph from stdin (JSON),
solves placement via Google OR-tools CP-SAT, writes a PlacedTemplate to
stdout (JSON).

Wire format mirrors `crates/core/src/balancer/placement/cp_sat.rs`:

Request (stdin)::

    {
      "graph": { ...BalancerGraph serde... },
      "n": 1, "m": 1,
      "timeout_ms": 1000,
      "seed": null
    }

Response (stdout) — one of::

    { "kind": "ok", "template": {...PlacedTemplate...},
      "solve_wall_ms": 5 }
    { "kind": "unsat" }
    { "kind": "timeout" }
    { "kind": "engine", "message": "..." }
    { "kind": "unimplemented", "message": "..." }

v1 scope: handles `(1, 1)` only — a single pass-through belt. Other
shapes return ``unimplemented``. The integration plumbing is the
deliverable; expanding shape coverage is incremental work in
follow-up commits, all of which slot into this same wire format.
"""

from __future__ import annotations

import json
import sys
import time
from typing import Any


# Direction codes (internal): 0=N, 1=E, 2=S, 3=W. Mapped to Factorio's
# 0/2/4/6 at output time.
_INTERNAL_DIRS = (0, 1, 2, 3)
_DELTAS = ((0, -1), (1, 0), (0, 1), (-1, 0))
_OPPOSITE = (2, 3, 0, 1)
_FACTORIO_DIR = (0, 2, 4, 6)


def _splitter_tiles(positions: list[tuple[int, int]]) -> set[tuple[int, int]]:
    """Tiles occupied by the given south-facing 2x1 splitter anchors."""
    tiles: set[tuple[int, int]] = set()
    for sx, sy in positions:
        tiles.add((sx, sy))
        tiles.add((sx + 1, sy))
    return tiles


def _route_belts(
    splitter_positions: list[tuple[int, int]],
    routes: list[tuple[tuple[int, int], tuple[int, int], int]],
    width: int,
    height: int,
) -> dict[tuple[int, int], int] | None:
    """Solve belt routing on the grid given fixed splitter positions.

    Each route in `routes` is `(src_tile, sink_tile, sink_dir)`:
      - `src_tile` is where the path begins (typically the tile directly
        south of a parent splitter's output port). It is assumed to be a
        free tile; the solver picks the belt direction at this tile.
      - `sink_tile` is where the path ends. It must be a free tile and
        will carry a belt in `sink_dir` (so its outflow drops into the
        downstream consumer — typically south into a child splitter).

    Returns a dict mapping each used belt tile to a direction code (one
    of 0/1/2/3 = N/E/S/W), or `None` if no routing is feasible.

    Each free tile is either empty or carries one belt direction; tiles
    are not shared between routes (no sideloading in this MVP). Splitter
    tiles are off-limits.
    """
    from ortools.sat.python import cp_model

    occupied = _splitter_tiles(splitter_positions)
    free: list[tuple[int, int]] = [
        (x, y)
        for x in range(width)
        for y in range(height)
        if (x, y) not in occupied
    ]
    free_set = set(free)
    for src, sink, _ in routes:
        if src not in free_set or sink not in free_set:
            return None

    model = cp_model.CpModel()

    # Per free tile, per direction: belt[(t, d)] = "tile t carries a
    # belt heading d".  Each tile has at most one direction.
    belt = {
        (t, d): model.new_bool_var(f"belt_{t[0]}_{t[1]}_d{d}")
        for t in free
        for d in _INTERNAL_DIRS
    }
    for t in free:
        model.add(sum(belt[(t, d)] for d in _INTERNAL_DIRS) <= 1)

    # Per route, per free tile, per direction: f[(r, t, d)] = "route r
    # uses tile t with belt direction d".  Tying f to belt:
    # f[(r, t, d)] = 1 implies belt[(t, d)] = 1.
    f = {
        (r, t, d): model.new_bool_var(f"f_r{r}_{t[0]}_{t[1]}_d{d}")
        for r in range(len(routes))
        for t in free
        for d in _INTERNAL_DIRS
    }
    # Belt at (t, d) iff some route uses (t, d). Combined with
    # at-most-one-direction-per-tile and at-most-one-route-per-tile
    # (below), this means belts are exactly the tiles needed by routes.
    for t in free:
        for d in _INTERNAL_DIRS:
            model.add(
                belt[(t, d)]
                == sum(f[(r, t, d)] for r in range(len(routes)))
            )

    # No tile shared between routes (MVP: no sideloading).
    for t in free:
        model.add(
            sum(f[(r, t, d)] for r in range(len(routes)) for d in _INTERNAL_DIRS)
            <= 1
        )

    # Per-route flow constraints. Conservation model:
    #   - At each free tile, "on_route" ∈ {0, 1} = sum of direction
    #     bools for this route.
    #   - "Inflow" at tile t = number of neighbors feeding t (a neighbor
    #     n feeds t if n is on the route AND n's belt points at t).
    #   - At source: on_route = 1, inflow = 0.
    #   - At sink: on_route = 1, inflow = 1 (via sink_dir's predecessor).
    #   - Otherwise: inflow == on_route. If on the route, exactly one
    #     neighbor feeds; outgoing direction must land on a free tile
    #     also on the route.
    for r, (src, sink, sink_dir) in enumerate(routes):
        # Sink belt's direction is fixed.
        model.add(belt[(sink, sink_dir)] == 1)
        model.add(f[(r, sink, sink_dir)] == 1)

        if src != sink:
            # Source has exactly one outgoing direction.
            model.add(sum(f[(r, src, d)] for d in _INTERNAL_DIRS) == 1)

        for t in free:
            on_route = sum(f[(r, t, d)] for d in _INTERNAL_DIRS)

            # Inflow: a neighbor n in direction d_n_to_t (relative to t)
            # feeds t iff n's belt heads d_n_to_t (toward t).
            in_flows = []
            for d_n_to_t in _INTERNAL_DIRS:
                opp_dx, opp_dy = _DELTAS[_OPPOSITE[d_n_to_t]]
                n = (t[0] + opp_dx, t[1] + opp_dy)
                if n in free_set:
                    in_flows.append(f[(r, n, d_n_to_t)])

            if t == src:
                # No inflow at source.
                if in_flows:
                    model.add(sum(in_flows) == 0)
            else:
                # Inflow matches on_route.
                model.add(sum(in_flows) == on_route)

            # Outflow: if heading direction d at t, the next tile must
            # be a free tile on the route — except at the sink, whose
            # next tile is the consumer splitter (off-route).
            if t != sink:
                for d in _INTERNAL_DIRS:
                    ndx, ndy = _DELTAS[d]
                    nxt = (t[0] + ndx, t[1] + ndy)
                    if nxt not in free_set:
                        model.add(f[(r, t, d)] == 0)
                    else:
                        on_route_nxt = sum(
                            f[(r, nxt, dd)] for dd in _INTERNAL_DIRS
                        )
                        model.add(on_route_nxt >= 1).only_enforce_if(
                            f[(r, t, d)]
                        )

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        return None

    out: dict[tuple[int, int], int] = {}
    for t in free:
        for d in _INTERNAL_DIRS:
            if solver.value(belt[(t, d)]):
                out[t] = d
                break
    return out


def emit(payload: dict[str, Any]) -> None:
    json.dump(payload, sys.stdout)
    sys.stdout.write("\n")
    sys.stdout.flush()


def emit_unimplemented(reason: str) -> None:
    emit({"kind": "unimplemented", "message": reason})


def place_one_to_one() -> dict[str, Any]:
    """Trivial passthrough — a single south-facing belt at (0, 0).

    Width 1, height 1. Input and output are the same tile. The bus
    engine reads `input_tiles` to find where flow enters and
    `output_tiles` to find where it exits.
    """
    return {
        "n_inputs": 1,
        "n_outputs": 1,
        "width": 1,
        "height": 1,
        "entities": [
            {"name": "transport-belt", "x": 0, "y": 0, "direction": 4},
        ],
        "input_tiles": [[0, 0]],
        "output_tiles": [[0, 0]],
    }


def place_single_splitter(n: int, m: int) -> dict[str, Any]:
    """Single-splitter shapes for n, m ∈ {1, 2}: (1, 2), (2, 1), (2, 2).

    South-facing splitter at anchor (0, 1) occupying (0, 1) and (1, 1).
    Input ports above at (0, 0) and (1, 0); output ports below at
    (0, 2) and (1, 2). The first `n` of the 2 input slots get input
    belts; the first `m` of the 2 output slots get output belts.

    Width 2, height 3. Mirrors the layout in
    `crates/balancer-gen/src/main.rs::emit_single_splitter_template`
    so the two entrypoints converge on a single geometry.
    """
    entities: list[dict[str, Any]] = [
        {"name": "splitter", "x": 0, "y": 1, "direction": 4},
    ]
    input_tiles: list[list[int]] = []
    output_tiles: list[list[int]] = []
    for slot in range(n):
        entities.append({"name": "transport-belt", "x": slot, "y": 0, "direction": 4})
        input_tiles.append([slot, 0])
    for slot in range(m):
        entities.append({"name": "transport-belt", "x": slot, "y": 2, "direction": 4})
        output_tiles.append([slot, 2])
    return {
        "n_inputs": n,
        "n_outputs": m,
        "width": 2,
        "height": 3,
        "entities": entities,
        "input_tiles": input_tiles,
        "output_tiles": output_tiles,
    }


def place_one_to_four() -> dict[str, Any]:
    """`(1, 4)` complete binary tree, placed via real CP-SAT.

    Phase 2 of the placement RFP — this replaces the prior hardcoded
    geometry with an actual CP-SAT spatial model. The constraint set
    in a 4×4 grid is tight enough that only one assignment satisfies
    it (root at (1, 1), level-1 children at (0, 2) and (2, 2)), so
    CP-SAT functions here as a proof of model correctness rather
    than as a search. The same modeling pattern extends to harder
    shapes where the search space is non-trivial.

    Constraints:
      - 3 splitter rectangles (2×1, south-facing), `no_overlap_2d`.
      - Tight-stack equalities between root and each child:
        left child anchor = (xR - 1, yR + 1); right = (xR + 1, yR + 1).
      - Grid bounds: input row above requires yR ≥ 1; output row
        below requires yR + 2 ≤ H - 1; columns must fit so leftmost
        and rightmost output belts stay in [0, W - 1].

    Returns the solved template dict, or raises if the model proves
    UNSAT (which would indicate a bug in the constraint set).
    """
    from ortools.sat.python import cp_model

    width, height = 4, 4
    model = cp_model.CpModel()

    # 3 splitters: 0 = root, 1 = left child, 2 = right child.
    xs = [model.new_int_var(0, width - 2, f"x{i}") for i in range(3)]
    ys = [model.new_int_var(0, height - 1, f"y{i}") for i in range(3)]

    # No-overlap on the splitter rectangles. Each splitter is 2 tiles
    # wide × 1 tile tall (south-facing). Redundant given the
    # tight-stack equalities below place children at y = yR+1
    # (different row from root), but exercises `add_no_overlap_2d` so
    # the same model structure carries to harder shapes where the
    # constraint isn't redundant.
    x_intervals = [
        model.new_interval_var(xs[i], 2, xs[i] + 2, f"x_iv_{i}") for i in range(3)
    ]
    y_intervals = [
        model.new_interval_var(ys[i], 1, ys[i] + 1, f"y_iv_{i}") for i in range(3)
    ]
    model.add_no_overlap_2d(x_intervals, y_intervals)

    # Tight-stack: each child sits exactly one row below the root,
    # offset by ±1 column so the root's output port tiles coincide
    # with each child's tile.
    model.add(xs[1] == xs[0] - 1)
    model.add(xs[2] == xs[0] + 1)
    model.add(ys[1] == ys[0] + 1)
    model.add(ys[2] == ys[0] + 1)

    # Boundary rows.
    model.add(ys[0] >= 1)
    model.add(ys[0] + 2 <= height - 1)
    model.add(xs[0] - 1 >= 0)
    model.add(xs[0] + 2 <= width - 1)

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"(1, 4) CP-SAT model UNSAT: {solver.status_name(status)}")

    x_root = solver.value(xs[0])
    y_root = solver.value(ys[0])
    x_left = solver.value(xs[1])
    y_left = solver.value(ys[1])
    x_right = solver.value(xs[2])
    y_right = solver.value(ys[2])

    entities: list[dict[str, Any]] = [
        {"name": "transport-belt", "x": x_root, "y": y_root - 1, "direction": 4},
        {"name": "splitter", "x": x_root, "y": y_root, "direction": 4},
        {"name": "splitter", "x": x_left, "y": y_left, "direction": 4},
        {"name": "splitter", "x": x_right, "y": y_right, "direction": 4},
    ]
    output_y = y_root + 2
    output_cols = [x_left, x_left + 1, x_right, x_right + 1]
    for col in output_cols:
        entities.append(
            {"name": "transport-belt", "x": col, "y": output_y, "direction": 4}
        )
    return {
        "n_inputs": 1,
        "n_outputs": 4,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [[x_root, y_root - 1]],
        "output_tiles": [[c, output_y] for c in output_cols],
    }


def place_one_to_eight() -> dict[str, Any]:
    """`(1, 8)` 7-splitter tree, placed via real CP-SAT for both
    splitter positions and belt routing.

    Two-phase model: the structural CP-SAT model fixes splitter
    positions (root + routing-row offset to level-1, tight-stack to
    level-2), then [`_route_belts`] solves the per-tile belt
    directions via flow-conservation constraints over a tile graph.
    Replaces the prior hand-coded W-S-E-S routing strip with a
    generic router that scales to any graph topology.

    Width 8, height 6. Splitter ordering: 0 = root, 1-2 = level-1,
    3-6 = level-2.
    """
    from ortools.sat.python import cp_model

    width, height = 8, 6
    n_splitters = 7
    model = cp_model.CpModel()
    xs = [model.new_int_var(0, width - 2, f"x{i}") for i in range(n_splitters)]
    ys = [model.new_int_var(0, height - 1, f"y{i}") for i in range(n_splitters)]

    x_intervals = [
        model.new_interval_var(xs[i], 2, xs[i] + 2, f"x_iv_{i}")
        for i in range(n_splitters)
    ]
    y_intervals = [
        model.new_interval_var(ys[i], 1, ys[i] + 1, f"y_iv_{i}")
        for i in range(n_splitters)
    ]
    model.add_no_overlap_2d(x_intervals, y_intervals)

    # Routing-row offsets between root and level-1.
    model.add(xs[1] == xs[0] - 2)
    model.add(xs[2] == xs[0] + 2)
    model.add(ys[1] == ys[0] + 2)
    model.add(ys[2] == ys[0] + 2)

    # Tight-stack between level-1 and level-2.
    model.add(xs[3] == xs[1] - 1)
    model.add(xs[4] == xs[1] + 1)
    model.add(ys[3] == ys[1] + 1)
    model.add(ys[4] == ys[1] + 1)
    model.add(xs[5] == xs[2] - 1)
    model.add(xs[6] == xs[2] + 1)
    model.add(ys[5] == ys[2] + 1)
    model.add(ys[6] == ys[2] + 1)

    # Boundary rows.
    model.add(ys[0] >= 1)
    model.add(ys[0] + 4 <= height - 1)
    model.add(xs[0] - 3 >= 0)
    model.add(xs[0] + 4 <= width - 1)

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"(1, 8) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_root, y_root = pos[0]

    # Derive routes from the tree topology + splitter positions. Each
    # route is (src_tile, sink_tile, sink_dir) where sink_dir is south
    # (= 2 internal, = 4 Factorio) so items drop into the next splitter
    # or out of the grid.
    DIR_S = 2
    routes: list[tuple[tuple[int, int], tuple[int, int], int]] = []
    # Input belt feeds root's right tile (port 1) via south drop.
    input_tile = (x_root + 1, y_root - 1)
    routes.append((input_tile, input_tile, DIR_S))
    # Root → level-1: each side picks the closer L1 input port.
    for parent_idx, port, child_idx in ((0, 0, 1), (0, 1, 2)):
        px, py = pos[parent_idx]
        cx, cy = pos[child_idx]
        drop = (px + port, py + 1)
        approach_port = 0 if drop[0] < cx else 1
        approach = (cx + approach_port, cy - 1)
        routes.append((drop, approach, DIR_S))
    # L1 → L2: tight-stack, no belt needed.
    # L2 → outputs: 8 length-1 south belts on the bottom row.
    for parent_idx in (3, 4, 5, 6):
        px, py = pos[parent_idx]
        for port in (0, 1):
            drop = (px + port, py + 1)
            routes.append((drop, drop, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError("(1, 8) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for x, y in pos:
        entities.append({"name": "splitter", "x": x, "y": y, "direction": 4})
    for (x, y), d in belt_dirs.items():
        entities.append(
            {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
        )

    output_y = pos[3][1] + 1
    leftmost = pos[3][0]
    rightmost = pos[6][0] + 1
    output_cols = list(range(leftmost, rightmost + 1))
    return {
        "n_inputs": 1,
        "n_outputs": 8,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [list(input_tile)],
        "output_tiles": [[c, output_y] for c in output_cols],
    }


def place_one_to_sixteen() -> dict[str, Any]:
    """`(1, 16)` 15-splitter tree, placed via real CP-SAT.

    Phase 3 of the placement RFP — extends the routing-row pattern to a
    second level. The CP-SAT model has 15 splitter rectangles with
    `add_no_overlap_2d` plus structural equalities:
      - Routing-row offset between root and level-1: `(root.x ± 4,
        root.y + 2)`. The 1-row gap holds a 9-belt
        W-W-W-W-S-E-E-E-S routing strip that spreads root's two
        outputs to columns 4 apart.
      - Routing-row offset between level-1 and level-2: `(L1.x ± 2,
        L1.y + 2)`. The 1-row gap holds two mirrored W-W-S / E-S
        routing groups (10 belts, columns -6..-2 and +2..+6 of the
        root) that spread each L1 splitter's two outputs by ±2.
      - Tight-stack between level-2 and level-3: each level-3
        splitter is at `(L2.x ± 1, L2.y + 1)`.

    Width 16, height 8. Like (1, 4) and (1, 8), the constraint set has
    only one valid assignment (root at (7, 1)); the model proves this
    rather than asserting it. Splitter ordering is BFS:
    0 = root, 1-2 = level-1, 3-6 = level-2, 7-14 = level-3.
    """
    from ortools.sat.python import cp_model

    width, height = 16, 8
    n_splitters = 15
    model = cp_model.CpModel()
    xs = [model.new_int_var(0, width - 2, f"x{i}") for i in range(n_splitters)]
    ys = [model.new_int_var(0, height - 1, f"y{i}") for i in range(n_splitters)]

    x_intervals = [
        model.new_interval_var(xs[i], 2, xs[i] + 2, f"x_iv_{i}")
        for i in range(n_splitters)
    ]
    y_intervals = [
        model.new_interval_var(ys[i], 1, ys[i] + 1, f"y_iv_{i}")
        for i in range(n_splitters)
    ]
    model.add_no_overlap_2d(x_intervals, y_intervals)

    # Routing-row offsets between root and level-1 (4-col spread).
    model.add(xs[1] == xs[0] - 4)
    model.add(xs[2] == xs[0] + 4)
    model.add(ys[1] == ys[0] + 2)
    model.add(ys[2] == ys[0] + 2)

    # Routing-row offsets between level-1 and level-2 (2-col spread per L1).
    for parent, (lc, rc) in [(1, (3, 4)), (2, (5, 6))]:
        model.add(xs[lc] == xs[parent] - 2)
        model.add(xs[rc] == xs[parent] + 2)
        model.add(ys[lc] == ys[parent] + 2)
        model.add(ys[rc] == ys[parent] + 2)

    # Tight-stack between level-2 and level-3.
    for parent, (lc, rc) in [(3, (7, 8)), (4, (9, 10)), (5, (11, 12)), (6, (13, 14))]:
        model.add(xs[lc] == xs[parent] - 1)
        model.add(xs[rc] == xs[parent] + 1)
        model.add(ys[lc] == ys[parent] + 1)
        model.add(ys[rc] == ys[parent] + 1)

    # Boundary rows: input row above root, output row below level-3.
    model.add(ys[0] >= 1)
    model.add(ys[0] + 6 <= height - 1)
    model.add(xs[0] - 7 >= 0)
    model.add(xs[0] + 8 <= width - 1)

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"(1, 16) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_root, y_root = pos[0]

    entities: list[dict[str, Any]] = []
    # Input belt above root's right tile.
    entities.append(
        {"name": "transport-belt", "x": x_root + 1, "y": y_root - 1, "direction": 4}
    )
    # Splitters.
    for x, y in pos:
        entities.append({"name": "splitter", "x": x, "y": y, "direction": 4})

    # Routing strip between root (y_root) and level-1 (y_root + 2).
    # 9 belts: W on cols [x_root-3 .. x_root], S on x_root-4, E on
    # cols [x_root+1 .. x_root+3], S on x_root+4. Direction codes:
    # 0=N, 2=E, 4=S, 6=W.
    routing_y_1 = y_root + 1
    entities.append(
        {"name": "transport-belt", "x": x_root - 4, "y": routing_y_1, "direction": 4}
    )
    for col in range(x_root - 3, x_root + 1):
        entities.append(
            {"name": "transport-belt", "x": col, "y": routing_y_1, "direction": 6}
        )
    for col in range(x_root + 1, x_root + 4):
        entities.append(
            {"name": "transport-belt", "x": col, "y": routing_y_1, "direction": 2}
        )
    entities.append(
        {"name": "transport-belt", "x": x_root + 4, "y": routing_y_1, "direction": 4}
    )

    # Routing strip between level-1 (y_root + 2) and level-2 (y_root + 4).
    # Two groups, each spreading an L1 splitter's outputs by ±2.
    # Left group (under L1-left): W on its two output cols and the col to
    #   their west, then S two cols further west; E on the col east of
    #   port-1's output, then S one further east.
    # Right group: mirror layout under L1-right.
    routing_y_2 = pos[1][1] + 1
    # Left group: under L1-left at x_l1l = x_root - 4.
    x_l1l = pos[1][0]
    entities.append(
        {"name": "transport-belt", "x": x_l1l - 2, "y": routing_y_2, "direction": 4}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1l - 1, "y": routing_y_2, "direction": 6}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1l, "y": routing_y_2, "direction": 6}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1l + 1, "y": routing_y_2, "direction": 2}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1l + 2, "y": routing_y_2, "direction": 4}
    )
    # Right group: under L1-right at x_l1r = x_root + 4.
    x_l1r = pos[2][0]
    entities.append(
        {"name": "transport-belt", "x": x_l1r - 2, "y": routing_y_2, "direction": 4}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1r - 1, "y": routing_y_2, "direction": 6}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1r, "y": routing_y_2, "direction": 6}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1r + 1, "y": routing_y_2, "direction": 2}
    )
    entities.append(
        {"name": "transport-belt", "x": x_l1r + 2, "y": routing_y_2, "direction": 4}
    )

    # Output row at y = level-3.y + 1, cols spanning all 8 level-3 splitters.
    output_y = pos[7][1] + 1
    leftmost = pos[7][0]
    rightmost = pos[14][0] + 1
    output_cols = list(range(leftmost, rightmost + 1))
    for col in output_cols:
        entities.append(
            {"name": "transport-belt", "x": col, "y": output_y, "direction": 4}
        )
    return {
        "n_inputs": 1,
        "n_outputs": 16,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [[x_root + 1, y_root - 1]],
        "output_tiles": [[c, output_y] for c in output_cols],
    }


def main() -> int:
    raw = sys.stdin.read()
    try:
        request = json.loads(raw)
    except json.JSONDecodeError as e:
        emit({"kind": "engine", "message": f"parse stdin: {e}"})
        return 0

    n = int(request.get("n", 0))
    m = int(request.get("m", 0))
    timeout_ms = int(request.get("timeout_ms", 1000))

    started = time.monotonic()

    # Map shape → geometry-emitting function. v1 uses hardcoded
    # geometry; phase 2 of the placement RFP replaces this with a
    # real CP-SAT spatial model.
    geometry: dict[tuple[int, int], Any] = {
        (1, 1): place_one_to_one,
        (1, 2): lambda: place_single_splitter(1, 2),
        (2, 1): lambda: place_single_splitter(2, 1),
        (2, 2): lambda: place_single_splitter(2, 2),
        (1, 4): place_one_to_four,
        (1, 8): place_one_to_eight,
        (1, 16): place_one_to_sixteen,
    }

    if (n, m) in geometry:
        try:
            template = geometry[(n, m)]()
        except RuntimeError as e:
            emit({"kind": "engine", "message": str(e)})
            return 0
        elapsed_ms = int((time.monotonic() - started) * 1000)
        emit({"kind": "ok", "template": template, "solve_wall_ms": elapsed_ms})
        return 0

    emit_unimplemented(f"shape ({n}, {m}) not yet supported by cp_sat_placer")
    return 0


if __name__ == "__main__":
    sys.exit(main())
