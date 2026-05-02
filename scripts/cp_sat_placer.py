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
    # Belt at (t, d) iff at least one route uses (t, d). Multiple
    # routes can share a tile in the same direction (sideloading: a
    # perpendicular belt feeds into the receiving belt's stream).
    n_routes = len(routes)
    for t in free:
        for d in _INTERNAL_DIRS:
            s = sum(f[(r, t, d)] for r in range(n_routes))
            # belt = 1 → s >= 1 (some route uses this tile).
            model.add(s >= belt[(t, d)])
            # belt = 0 → s = 0 (no route on a tile without a belt).
            model.add(s <= n_routes * belt[(t, d)])

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

    # Minimize total belts so the solver picks shortest paths instead of
    # wandering. Without this, large grids admit many long-path
    # alternatives that all satisfy the structural constraints.
    model.minimize(
        sum(belt[(t, d)] for t in free for d in _INTERNAL_DIRS)
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


def _input_routes_for_root(
    x_root: int, y_root: int, n: int
) -> tuple[
    list[tuple[int, int]],
    list[tuple[tuple[int, int], tuple[int, int], int]],
]:
    """Build input belt tiles and routes for `n ∈ {1, 2, 3}` inputs.

    Returns `(input_tiles, routes)`. Each route is
    `(src, sink, sink_dir)` for [`_route_belts`].

    n=1: feed port 0 (left tile of root).
    n=2: feed both root ports directly.
    n=3: 2 direct + 1 sideload from west: (x_root - 1, y_root - 1)
         heads east into the south-belt above port 0. Requires the
         routing model to allow multiple routes per tile.
    """
    DIR_S = 2
    direct_left = (x_root, y_root - 1)
    direct_right = (x_root + 1, y_root - 1)
    if n == 1:
        return [direct_left], [(direct_left, direct_left, DIR_S)]
    if n == 2:
        tiles = [direct_left, direct_right]
        return tiles, [(t, t, DIR_S) for t in tiles]
    if n == 3:
        sideload = (x_root - 1, y_root - 1)
        tiles = [direct_left, direct_right, sideload]
        routes = [
            (direct_left, direct_left, DIR_S),
            (direct_right, direct_right, DIR_S),
            (sideload, direct_left, DIR_S),
        ]
        return tiles, routes
    raise ValueError(f"unsupported input count {n}; expected 1, 2, or 3")


def place_x_to_four(n: int) -> dict[str, Any]:
    """`(n, 4)` complete binary tree for `n ∈ {1, 2}`.

    All splitter-to-splitter arcs are tight-stack (no belts needed);
    the routing solver handles only the input belt(s) and the 4
    output belts.
    """
    from ortools.sat.python import cp_model

    width, height = 4, 4
    model = cp_model.CpModel()

    xs = [model.new_int_var(0, width - 2, f"x{i}") for i in range(3)]
    ys = [model.new_int_var(0, height - 1, f"y{i}") for i in range(3)]

    x_intervals = [
        model.new_interval_var(xs[i], 2, xs[i] + 2, f"x_iv_{i}") for i in range(3)
    ]
    y_intervals = [
        model.new_interval_var(ys[i], 1, ys[i] + 1, f"y_iv_{i}") for i in range(3)
    ]
    model.add_no_overlap_2d(x_intervals, y_intervals)

    model.add(xs[1] == xs[0] - 1)
    model.add(xs[2] == xs[0] + 1)
    model.add(ys[1] == ys[0] + 1)
    model.add(ys[2] == ys[0] + 1)

    model.add(ys[0] >= 1)
    model.add(ys[0] + 2 <= height - 1)
    model.add(xs[0] - 1 >= 0)
    model.add(xs[0] + 2 <= width - 1)

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"({n}, 4) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(3)]
    x_root, y_root = pos[0]

    DIR_S = 2
    input_tiles, routes = _input_routes_for_root(x_root, y_root, n)
    for parent_idx in (1, 2):
        px, py = pos[parent_idx]
        for port in (0, 1):
            drop = (px + port, py + 1)
            routes.append((drop, drop, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError(f"({n}, 4) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for x, y in pos:
        entities.append({"name": "splitter", "x": x, "y": y, "direction": 4})
    for (x, y), d in belt_dirs.items():
        entities.append(
            {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
        )

    output_y = pos[1][1] + 1
    output_cols = [pos[1][0], pos[1][0] + 1, pos[2][0], pos[2][0] + 1]
    return {
        "n_inputs": n,
        "n_outputs": 4,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [list(it) for it in input_tiles],
        "output_tiles": [[c, output_y] for c in output_cols],
    }


def place_x_to_eight(n: int) -> dict[str, Any]:
    """`(n, 8)` 7-splitter tree for `n ∈ {1, 2}`.

    Structural CP-SAT model fixes splitter positions (root +
    routing-row offset to level-1, tight-stack to level-2), then
    [`_route_belts`] solves per-tile belt directions via flow
    conservation. Splitter ordering: 0 = root, 1-2 = level-1,
    3-6 = level-2.

    Width 8, height 6.
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
        raise RuntimeError(f"({n}, 8) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_root, y_root = pos[0]

    DIR_S = 2
    input_tiles, routes = _input_routes_for_root(x_root, y_root, n)
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
        raise RuntimeError(f"({n}, 8) belt routing UNSAT")

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
        "n_inputs": n,
        "n_outputs": 8,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [list(it) for it in input_tiles],
        "output_tiles": [[c, output_y] for c in output_cols],
    }


def place_x_to_sixteen(n: int) -> dict[str, Any]:
    """`(n, 16)` 15-splitter tree for `n ∈ {1, 2}`.

    Structural offsets:
      - Root → level-1: routing-row offset, ±4 cols, +2 rows.
      - Level-1 → level-2: routing-row offset, ±2 cols, +2 rows.
      - Level-2 → level-3: tight-stack, ±1 col, +1 row.

    [`_route_belts`] solves per-tile belt directions. Splitter
    ordering is BFS: 0 = root, 1-2 = level-1, 3-6 = level-2,
    7-14 = level-3.

    Width 16, height 8.
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

    model.add(xs[1] == xs[0] - 4)
    model.add(xs[2] == xs[0] + 4)
    model.add(ys[1] == ys[0] + 2)
    model.add(ys[2] == ys[0] + 2)

    for parent, (lc, rc) in [(1, (3, 4)), (2, (5, 6))]:
        model.add(xs[lc] == xs[parent] - 2)
        model.add(xs[rc] == xs[parent] + 2)
        model.add(ys[lc] == ys[parent] + 2)
        model.add(ys[rc] == ys[parent] + 2)

    for parent, (lc, rc) in [(3, (7, 8)), (4, (9, 10)), (5, (11, 12)), (6, (13, 14))]:
        model.add(xs[lc] == xs[parent] - 1)
        model.add(xs[rc] == xs[parent] + 1)
        model.add(ys[lc] == ys[parent] + 1)
        model.add(ys[rc] == ys[parent] + 1)

    model.add(ys[0] >= 1)
    model.add(ys[0] + 6 <= height - 1)
    model.add(xs[0] - 7 >= 0)
    model.add(xs[0] + 8 <= width - 1)

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"({n}, 16) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_root, y_root = pos[0]

    DIR_S = 2
    input_tiles, routes = _input_routes_for_root(x_root, y_root, n)
    # Root → L1.
    for parent_idx, port, child_idx in ((0, 0, 1), (0, 1, 2)):
        px, py = pos[parent_idx]
        cx, cy = pos[child_idx]
        drop = (px + port, py + 1)
        approach_port = 0 if drop[0] < cx else 1
        approach = (cx + approach_port, cy - 1)
        routes.append((drop, approach, DIR_S))
    # L1 → L2: routing-row offset, not tight-stack.
    for parent_idx, child_left, child_right in ((1, 3, 4), (2, 5, 6)):
        px, py = pos[parent_idx]
        for port, child_idx in ((0, child_left), (1, child_right)):
            cx, cy = pos[child_idx]
            drop = (px + port, py + 1)
            approach_port = 0 if drop[0] < cx else 1
            approach = (cx + approach_port, cy - 1)
            routes.append((drop, approach, DIR_S))
    # L2 → L3: tight-stack, no belts.
    # L3 → outputs: 16 length-1 south belts.
    for parent_idx in range(7, 15):
        px, py = pos[parent_idx]
        for port in (0, 1):
            drop = (px + port, py + 1)
            routes.append((drop, drop, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError(f"({n}, 16) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for x, y in pos:
        entities.append({"name": "splitter", "x": x, "y": y, "direction": 4})
    for (x, y), d in belt_dirs.items():
        entities.append(
            {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
        )

    output_y = pos[7][1] + 1
    leftmost = pos[7][0]
    rightmost = pos[14][0] + 1
    output_cols = list(range(leftmost, rightmost + 1))
    return {
        "n_inputs": n,
        "n_outputs": 16,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [list(it) for it in input_tiles],
        "output_tiles": [[c, output_y] for c in output_cols],
    }


def place_one_to_five() -> dict[str, Any]:
    """`(1, 5)` — first coprime shape, with a merger and feedback arcs.

    Topology (synth(1, 5)): input → merger → 7-splitter (1, 8) tree
    → 5 outputs + 3 feedback arcs back to merger.in_1 (sideloaded).

    Layout (10 × 8):
      - Row 1: merger at (3, 1).
      - Row 2: two length-1 belts feed root from above.
      - Row 3: root at (3, 3).
      - Row 4: routing row root → L1.
      - Row 5: L1 at (1, 5), (5, 5).
      - Row 6: L2 at (0, 6), (2, 6), (4, 6), (6, 6) — cols 8-9 free
        for the feedback channel.
      - Row 7: 5 output belts + 3 feedback starts.

    Feedback paths from row 7 wrap up the right side (cols 8-9)
    back to row 0 then west into merger.in_1. All 3 feedback routes
    converge via sideloading.

    Splitter ordering: 0 = merger, 1 = root, 2-3 = L1, 4-7 = L2.
    """
    from ortools.sat.python import cp_model

    width, height = 10, 8
    n_splitters = 8
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

    # Merger at row 1 (inputs come from row 0).
    model.add(ys[0] == 1)
    # Root directly below merger with a 1-row belt gap.
    model.add(xs[1] == xs[0])
    model.add(ys[1] == ys[0] + 2)
    # Root → L1: routing-row offset.
    model.add(xs[2] == xs[1] - 2)
    model.add(xs[3] == xs[1] + 2)
    model.add(ys[2] == ys[1] + 2)
    model.add(ys[3] == ys[1] + 2)
    # L1 → L2: tight-stack.
    model.add(xs[4] == xs[2] - 1)
    model.add(xs[5] == xs[2] + 1)
    model.add(ys[4] == ys[2] + 1)
    model.add(ys[5] == ys[2] + 1)
    model.add(xs[6] == xs[3] - 1)
    model.add(xs[7] == xs[3] + 1)
    model.add(ys[6] == ys[3] + 1)
    model.add(ys[7] == ys[3] + 1)

    # Boundary: leave columns 8-9 free for the feedback channel —
    # L2[3] (rightmost) must end at col 7, so root.x ≤ 3.
    model.add(xs[1] >= 3)
    model.add(xs[1] <= 3)
    # Output row at L2.y + 1 must fit.
    model.add(ys[4] + 1 <= height - 1)

    solver = cp_model.CpSolver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"(1, 5) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_merger, y_merger = pos[0]
    x_root, y_root = pos[1]

    DIR_S = 2
    routes: list[tuple[tuple[int, int], tuple[int, int], int]] = []

    # Input belt at (x_merger, 0), feeds merger.in_0.
    input_tile = (x_merger, 0)
    routes.append((input_tile, input_tile, DIR_S))
    # Feedback approach belt at (x_merger + 1, 0), receives 3 feedback
    # routes via sideloading. Declared as the sink_dir's belt by each
    # feedback route below.

    # Merger → root: 2 vertical length-1 belts at row y_merger + 1 = 2.
    for port in (0, 1):
        belt = (x_merger + port, y_merger + 1)
        routes.append((belt, belt, DIR_S))

    # Root → L1: routing-row offset; pick the closer L1 input port.
    for port, child_idx in ((0, 2), (1, 3)):
        cx, cy = pos[child_idx]
        drop = (x_root + port, y_root + 1)
        approach_port = 0 if drop[0] < cx else 1
        approach = (cx + approach_port, cy - 1)
        routes.append((drop, approach, DIR_S))

    # L1 → L2: tight-stack, no belts.

    # L2 → outputs/feedback. Leaf assignment from synth(1, 5):
    #   leaf 0..4 → Output(0..4); leaf 5..7 → feedback.
    # In synth's BFS-heap layout, leaves of inner-tree node `old_i` are
    # at child_old = 2*old_i + {1, 2}. The L2 splitters are inner
    # indices 4-7 (after +1 inner_offset for the merger), corresponding
    # to old_i ∈ 3..6. Leaf indices: old_i=3 → leaves 0,1; old_i=4 → 2,3;
    # old_i=5 → 4,fb; old_i=6 → fb,fb.
    output_tiles: list[tuple[int, int]] = []
    feedback_srcs: list[tuple[int, int]] = []
    leaf_targets = [
        (4, [0, 1]),       # L2[0]
        (5, [2, 3]),       # L2[1]
        (6, [4, "fb"]),   # L2[2]
        (7, ["fb", "fb"]),# L2[3]
    ]
    for splitter_idx, targets in leaf_targets:
        px, py = pos[splitter_idx]
        for port, target in zip((0, 1), targets):
            drop = (px + port, py + 1)
            if target == "fb":
                feedback_srcs.append(drop)
            else:
                routes.append((drop, drop, DIR_S))
                output_tiles.append(drop)

    # Feedback: 3 arcs from L2 leaves back to merger.in_1's approach
    # tile (x_merger + 1, 0). All sideload onto the same sink belt.
    feedback_sink = (x_merger + 1, 0)
    for src in feedback_srcs:
        routes.append((src, feedback_sink, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError("(1, 5) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for x, y in pos:
        entities.append({"name": "splitter", "x": x, "y": y, "direction": 4})
    for (x, y), d in belt_dirs.items():
        entities.append(
            {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
        )

    return {
        "n_inputs": 1,
        "n_outputs": 5,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [list(input_tile)],
        "output_tiles": [list(t) for t in output_tiles],
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
        (1, 4): lambda: place_x_to_four(1),
        (2, 4): lambda: place_x_to_four(2),
        (3, 4): lambda: place_x_to_four(3),
        (1, 5): place_one_to_five,
        (1, 8): lambda: place_x_to_eight(1),
        (2, 8): lambda: place_x_to_eight(2),
        (3, 8): lambda: place_x_to_eight(3),
        (1, 16): lambda: place_x_to_sixteen(1),
        (2, 16): lambda: place_x_to_sixteen(2),
        (3, 16): lambda: place_x_to_sixteen(3),
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
