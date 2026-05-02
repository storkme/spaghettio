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
# 90° counterclockwise / clockwise rotations. Used by the per-lane
# sideload rule: items entering a belt from the LEFT side land on the
# left lane (lane 0); from the RIGHT side land on the right lane
# (lane 1); from the BACK (opposite of belt direction) preserve the
# upstream lane.
_LEFT_OF = (3, 0, 1, 2)
_RIGHT_OF = (1, 2, 3, 0)
_FACTORIO_DIR = (0, 2, 4, 6)


def _splitter_tiles(
    positions: list[tuple[int, int, int]] | list[tuple[int, int]],
) -> set[tuple[int, int]]:
    """Tiles occupied by the given splitter anchors.

    `positions` is a list of `(x, y)` (defaulting to south-facing) or
    `(x, y, direction)` tuples where direction is the internal code
    (0=N, 1=E, 2=S, 3=W). North/south splitters span 2 tiles
    east-west: `(x, y)` and `(x+1, y)`. East/west splitters span 2
    tiles north-south: `(x, y)` and `(x, y+1)`.
    """
    tiles: set[tuple[int, int]] = set()
    for pos in positions:
        if len(pos) == 2:
            sx, sy = pos
            d = 2  # south
        else:
            sx, sy, d = pos
        tiles.add((sx, sy))
        if d in (0, 2):  # north or south: footprint is east-west
            tiles.add((sx + 1, sy))
        else:  # east or west: footprint is north-south
            tiles.add((sx, sy + 1))
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

    # Per route, per free tile, per direction, per lane: route uses
    # this lane of this belt. `lane ∈ {0, 1}` = {LEFT, RIGHT} relative
    # to the belt's direction of travel.
    n_lanes = 2
    f_lane = {
        (r, t, d, lane): model.new_bool_var(f"f_r{r}_{t[0]}_{t[1]}_d{d}_l{lane}")
        for r in range(len(routes))
        for t in free
        for d in _INTERNAL_DIRS
        for lane in range(n_lanes)
    }

    def f_sum(r: int, t: tuple[int, int], d: int):
        """Lane-summed flow indicator: 1 iff route r uses (t, d) on either lane."""
        return sum(f_lane[(r, t, d, lane)] for lane in range(n_lanes))

    # Per-lane cap: each (tile, direction, lane) carries at most one
    # route. Half a belt per route in the normalized fluid model. With
    # the at-most-one-route-per-tile constraint below this is currently
    # redundant; it becomes load-bearing in phase 2 when we drop the
    # tile-exclusivity rule and let routes share tiles on different
    # lanes (sideloading).
    for t in free:
        for d in _INTERNAL_DIRS:
            for lane in range(n_lanes):
                model.add(
                    sum(f_lane[(r, t, d, lane)] for r in range(len(routes))) <= 1
                )

    # Belt at (t, d) iff some route uses (t, d) on any lane. The
    # tile-exclusivity constraint below means at most one route per
    # tile in phase 1, so this acts like the old `belt = sum_r f`
    # (each lane contributes 0 or 1 with at most one nonzero). In
    # phase 2 sideloading lets two routes share a tile on different
    # lanes; this OR-style equivalence still holds.
    for t in free:
        for d in _INTERNAL_DIRS:
            s = sum(f_lane[(r, t, d, lane)] for r in range(len(routes)) for lane in range(n_lanes))
            model.add(s >= belt[(t, d)])
            model.add(s <= 2 * belt[(t, d)])

    # Each route uses at most one (direction, lane) per tile. Without
    # this a route could occupy multiple lanes/dirs at the same tile,
    # which is incoherent for a single belt-path.
    for r in range(len(routes)):
        for t in free:
            model.add(
                sum(
                    f_lane[(r, t, d, lane)]
                    for d in _INTERNAL_DIRS
                    for lane in range(n_lanes)
                )
                <= 1
            )

    # Per-route flow conservation, per-lane (phase 2).
    # Items at tile t (direction d_recv, lane L_recv) arrive from a
    # neighbor n at side s of t whose belt heads d_pred = OPPOSITE(s).
    # The lane the items land on at t depends on s relative to d_recv:
    #   - s = OPPOSITE(d_recv) (back / natural input): lane preserved
    #     from predecessor.
    #   - s = LEFT_OF(d_recv) (left side feed): items land on lane 0
    #     regardless of predecessor lane.
    #   - s = RIGHT_OF(d_recv) (right side feed): items land on lane 1
    #     regardless of predecessor lane.
    #   - s = d_recv (forward / output side): items can't enter.
    for r, (src, sink, sink_dir) in enumerate(routes):
        # Sink belt's direction is fixed; route uses sink with
        # exactly one lane (solver picks).
        model.add(belt[(sink, sink_dir)] == 1)
        model.add(f_sum(r, sink, sink_dir) == 1)

        if src != sink:
            # Source has exactly one (direction, lane).
            model.add(
                sum(
                    f_lane[(r, src, d, lane)]
                    for d in _INTERNAL_DIRS
                    for lane in range(n_lanes)
                )
                == 1
            )

        for t in free:
            for d_recv in _INTERNAL_DIRS:
                for L_recv in range(n_lanes):
                    in_flows = []
                    for s in _INTERNAL_DIRS:
                        if s == d_recv:
                            continue  # output side
                        sdx, sdy = _DELTAS[s]
                        n = (t[0] + sdx, t[1] + sdy)
                        if n not in free_set:
                            continue
                        d_pred = _OPPOSITE[s]
                        if s == _OPPOSITE[d_recv]:
                            # Natural input: predecessor on same lane.
                            in_flows.append(f_lane[(r, n, d_pred, L_recv)])
                        elif s == _LEFT_OF[d_recv]:
                            # Sideload from left → lane 0 of t.
                            if L_recv == 0:
                                for L in range(n_lanes):
                                    in_flows.append(f_lane[(r, n, d_pred, L)])
                        elif s == _RIGHT_OF[d_recv]:
                            # Sideload from right → lane 1 of t.
                            if L_recv == 1:
                                for L in range(n_lanes):
                                    in_flows.append(f_lane[(r, n, d_pred, L)])

                    # Conservation only meaningful when the tile actually
                    # has a belt heading d_recv; otherwise predecessor
                    # contributions are accounted for at the tile's true
                    # belt direction (or the tile is empty).
                    if t == src:
                        if in_flows:
                            model.add(sum(in_flows) == 0).only_enforce_if(
                                belt[(t, d_recv)]
                            )
                    else:
                        model.add(
                            sum(in_flows) == f_lane[(r, t, d_recv, L_recv)]
                        ).only_enforce_if(belt[(t, d_recv)])

            # Outflow: if heading direction d at t (any lane), next tile
            # must be a free tile that the route uses too — except at
            # the sink, whose next tile is the consumer splitter.
            if t != sink:
                for d in _INTERNAL_DIRS:
                    ndx, ndy = _DELTAS[d]
                    nxt = (t[0] + ndx, t[1] + ndy)
                    if nxt not in free_set:
                        for lane in range(n_lanes):
                            model.add(f_lane[(r, t, d, lane)] == 0)
                    else:
                        on_route_nxt = sum(f_sum(r, nxt, dd) for dd in _INTERNAL_DIRS)
                        for lane in range(n_lanes):
                            model.add(on_route_nxt >= 1).only_enforce_if(
                                f_lane[(r, t, d, lane)]
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

    Each direct input boundary belt is modeled as **two** length-1
    routes at the same tile. Real Factorio input belts arrive with
    items on both lanes (the upstream bus or balancer feeds both),
    so two routes at the same `(t, S)` sink tile force both lanes
    to be claimed via the per-lane cap (≤ 1 route per
    `(tile, dir, lane)`). Without this, single-route input modeling
    leaves the "other" lane of the boundary belt available for
    sideload — under-counting saturation.

    Sideload inputs (n=3 case) emit a single route, since they only
    fill the lane the side-feed forces.

    n=1: 2 lane-routes feeding port 0 (left tile of root).
    n=2: 4 lane-routes (2 per port).
    n=3: 4 + 1 sideload from west.
    """
    DIR_S = 2
    direct_left = (x_root, y_root - 1)
    direct_right = (x_root + 1, y_root - 1)
    if n == 1:
        # Two routes at the same tile force both lanes to be claimed.
        return [direct_left], [
            (direct_left, direct_left, DIR_S),
            (direct_left, direct_left, DIR_S),
        ]
    if n == 2:
        tiles = [direct_left, direct_right]
        routes = []
        for t in tiles:
            routes.append((t, t, DIR_S))
            routes.append((t, t, DIR_S))
        return tiles, routes
    if n == 3:
        sideload = (x_root - 1, y_root - 1)
        tiles = [direct_left, direct_right, sideload]
        routes = [
            (direct_left, direct_left, DIR_S),
            (direct_left, direct_left, DIR_S),
            (direct_right, direct_right, DIR_S),
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
        # (3, 4) deferred: per-lane cap correctly rejects it. Needs a
        # structural mitigation (lane-balancer splitter) — phase 3.
        (1, 8): lambda: place_x_to_eight(1),
        (2, 8): lambda: place_x_to_eight(2),
        (1, 16): lambda: place_x_to_sixteen(1),
        (2, 16): lambda: place_x_to_sixteen(2),
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
