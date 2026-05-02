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

This is the canonical placer consumed by
`crates/core/src/balancer/placement/cp_sat.rs` and the
`cp_sat_round_trip` test suite. A separate spike at
`crates/balancer-gen/scripts/place.py` covers UG belts and mixed splitter
directions for the bake pipeline; that spike will eventually migrate
onto this script once the canonical placer covers the same shape
repertoire. New shape coverage should land here, not in the spike.

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

# Route tuple shape:
#   (src, sink, sink_dir, splitter_dir)
# where `splitter_dir ∈ {0, 1, 2, 3, None}`. None means the source is an
# input boundary belt — its lane is unconstrained at the source. A
# direction means the source tile is a splitter-output drop with the
# upstream splitter facing that direction; source-lane forcing applies
# per the table in `docs/rfp-lane-aware-routing.md`:
#   d_drop == splitter_dir          → head-on, items fill both lanes;
#                                      construction emits 2 lane-routes
#                                      per arc and the per-lane cap
#                                      forces them apart.
#   d_drop == LEFT_OF(splitter_dir) → sideload onto LEFT lane (lane 0).
#   d_drop == RIGHT_OF(splitter_dir)→ sideload onto RIGHT lane (lane 1).
#   d_drop == OPPOSITE(splitter_dir)→ would head into the splitter tile;
#                                      blocked by the existing geometry
#                                      constraints (splitter tile not
#                                      free), so we don't emit an
#                                      explicit forbid.


def _splitter_dir(pos: tuple[int, int] | tuple[int, int, int]) -> int:
    """Direction of a splitter from its position tuple. Default south."""
    return pos[2] if len(pos) >= 3 else 2


# Module-level solver parameters. Populated by `main()` from the
# stdin request and consumed by `_make_solver()` at every CP-SAT
# solver instantiation. Centralising the configuration keeps each
# `place_*` function from re-discovering it; threading the params
# through every call site would be ~equivalent code but with more
# surface area for the wiring to drift again.
_SOLVER_PARAMS: dict[str, Any] = {"timeout_ms": 1000, "seed": None}


# Module-level synth context. Populated by `main()` from the stdin
# request. `graph` is the BalancerGraph dict (n_inputs, n_outputs,
# n_splitters, arcs) and `arc_throughputs` is the parallel `Vec<f64>`
# of per-arc steady-state rates from `verify_balancer`. Per-shape
# placers consult these to derive route rates for the rate-aware
# per-lane cap (see `_route_belts` `rates` / `lane_cap` parameters).
# When `arc_throughputs` is absent (the Rust side couldn't verify or
# it's a unit-rate dyadic shape), placers fall back to the discrete
# default — same as the pre-rate-aware behaviour.
_SYNTH_CTX: dict[str, Any] = {"graph": None, "arc_throughputs": None}


def _make_solver():
    """Build a `CpSolver` with timeout and seed from `_SOLVER_PARAMS`.

    Honours the `timeout_ms` and `seed` fields of the request. Pins
    `num_search_workers = 1` so determinism is reproducible across
    same-seed runs (kill criterion #5 in `rfp-cp-sat-placement.md`).
    """
    from ortools.sat.python import cp_model
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = _SOLVER_PARAMS["timeout_ms"] / 1000.0
    solver.parameters.num_search_workers = 1
    seed = _SOLVER_PARAMS.get("seed")
    if seed is not None:
        solver.parameters.random_seed = int(seed)
    return solver


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
    routes: list[
        tuple[tuple[int, int], tuple[int, int], int, int | None]
    ],
    width: int,
    height: int,
    rates: list[int] | None = None,
    lane_cap: int = 1,
) -> dict[tuple[int, int], int] | None:
    """Solve belt routing on the grid given fixed splitter positions.

    Each route in `routes` is `(src_tile, sink_tile, sink_dir, splitter_dir)`:
      - `src_tile` is where the path begins (typically the tile directly
        south of a parent splitter's output port). It is assumed to be a
        free tile; the solver picks the belt direction at this tile.
      - `sink_tile` is where the path ends. It must be a free tile and
        will carry a belt in `sink_dir` (so its outflow drops into the
        downstream consumer — typically south into a child splitter).
      - `splitter_dir` is `None` for input-boundary sources or the
        upstream splitter's facing direction for splitter-output drops.
        See the route-tuple comment at the top of this file for the
        source-lane forcing semantics.

    `rates` and `lane_cap` carry the rate-aware encoding: each route
    consumes `rates[r]` units of lane capacity and the per-lane cap is
    `lane_cap`. Defaults (`rates = [1] * len(routes)`, `lane_cap = 1`)
    reproduce the discrete-cap behavior — one route per lane. For shapes
    whose arcs carry fractional belt rates (e.g. coprime feedback
    channels at rate 0.2), pass per-route rates and a higher lane cap so
    multiple low-rate routes can share a lane when their sum fits.

    Returns a dict mapping each used belt tile to a direction code (one
    of 0/1/2/3 = N/E/S/W), or `None` if no routing is feasible.

    Splitter tiles are off-limits to belts. Multiple routes may share a
    free tile if they agree on belt direction and the per-lane cap is
    not exceeded — phase 2's lifted tile-exclusivity rule.
    """
    from ortools.sat.python import cp_model

    if rates is None:
        rates = [1] * len(routes)
    elif len(rates) != len(routes):
        raise ValueError(
            f"rates length {len(rates)} != routes length {len(routes)}"
        )

    occupied = _splitter_tiles(splitter_positions)
    free: list[tuple[int, int]] = [
        (x, y)
        for x in range(width)
        for y in range(height)
        if (x, y) not in occupied
    ]
    free_set = set(free)
    for src, sink, _, _ in routes:
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

    # Per-lane cap: weighted sum of route rates ≤ lane_cap.
    # In the default discrete encoding (`rates = [1, 1, ...]`,
    # `lane_cap = 1`), each lane carries at most one route. With
    # rate-aware encoding, multiple low-rate routes can share a lane if
    # their rates sum within `lane_cap`. This is the model unblock for
    # coprime-shape feedback channels where 2-3 routes at rate 0.2 each
    # need to share a lane (rate-aware: `0.4 ≤ 0.5`; discrete:
    # `2 ≰ 1`).
    for t in free:
        for d in _INTERNAL_DIRS:
            for lane in range(n_lanes):
                model.add(
                    sum(
                        rates[r] * f_lane[(r, t, d, lane)]
                        for r in range(len(routes))
                    )
                    <= lane_cap
                )

    # Belt at (t, d) iff some route uses (t, d) on any lane. The
    # tile-exclusivity constraint below means at most one route per
    # tile in phase 1, so this acts like the old `belt = sum_r f`
    # (each lane contributes 0 or 1 with at most one nonzero). In
    # phase 2 sideloading lets two routes share a tile on different
    # lanes; this OR-style equivalence still holds.
    for t in free:
        for d in _INTERNAL_DIRS:
            s = sum(
                f_lane[(r, t, d, lane)]
                for r in range(len(routes))
                for lane in range(n_lanes)
            )
            model.add(s >= belt[(t, d)])
            # Big-M = max possible route uses at this (t, d). With
            # rate-aware caps allowing multiple low-rate routes per
            # lane, this is bounded only by the total number of routes.
            model.add(s <= len(routes) * belt[(t, d)])

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
    for r, (src, sink, sink_dir, splitter_dir) in enumerate(routes):
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

        # Source-lane forcing for splitter-output drops. The route is
        # tagged with the upstream splitter's facing direction; for
        # whichever direction the source tile picks, the items entering
        # from the splitter side land on a specific lane via the
        # sideload table. Without this, the model treats the source as
        # free-laned and lets multiple drops "freely" pick whichever
        # lane keeps them feasible — physics doesn't.
        if splitter_dir is not None:
            for d in _INTERNAL_DIRS:
                if d == splitter_dir:
                    continue  # head-on: occupy both lanes via duplicate routes
                if d == _LEFT_OF[splitter_dir]:
                    # Items from the splitter side land on lane 0.
                    model.add(f_lane[(r, src, d, 1)] == 0)
                elif d == _RIGHT_OF[splitter_dir]:
                    # Items from the splitter side land on lane 1.
                    model.add(f_lane[(r, src, d, 0)] == 0)
                # OPPOSITE(splitter_dir): handled by existing geometry
                # (splitter tile is not free, so the outflow constraint
                # already disallows this direction at a drop tile).

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

    solver = _make_solver()
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


# Rate-aware encoding multiplier. The Couëtoux verifier emits arc
# throughputs as floats in `[0, n]`; we multiply by RATE_SCALE and round
# to integers for CP-SAT. Scale 10 covers the rates in 1..=10 × 1..=10
# coprime shapes (denominators 2, 4, 5, 8, 10) without precision loss.
# `LANE_CAP_SCALED = RATE_SCALE // 2 = 5` matches the 0.5 normalised
# lane cap.
RATE_SCALE = 10
LANE_CAP_SCALED = RATE_SCALE // 2


def _arc_rate_scaled(throughput: float) -> int:
    """Round a Couëtoux arc throughput to integer-scaled units.

    Returns `round(throughput * RATE_SCALE)`. Rates in `1..=10 ×
    1..=10` synth graphs are rationals with denominators dividing
    `RATE_SCALE`, so rounding is exact.
    """
    return int(round(throughput * RATE_SCALE))


def _find_arc_rate(src: dict[str, Any], dst: dict[str, Any]) -> int | None:
    """Look up the synth-graph arc throughput for a given (src, dst).

    `src` and `dst` are the JSON-shaped `Source` / `Sink` enum values:
    `{"Input": i}`, `{"Output": j}`, or `{"Splitter": {"idx": i, "port": p}}`.

    Returns the scaled integer rate (`int(throughput * RATE_SCALE)`)
    or `None` if the synth context is missing or the arc isn't in the
    graph. Per-shape placers use this to set per-route rates.
    """
    graph = _SYNTH_CTX.get("graph")
    rates = _SYNTH_CTX.get("arc_throughputs")
    if graph is None or rates is None:
        return None
    for i, arc in enumerate(graph.get("arcs", [])):
        if arc.get("src") == src and arc.get("dst") == dst:
            if i < len(rates):
                return _arc_rate_scaled(rates[i])
            return None
    return None


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


def _add_lane_balancer_south(
    splitter_positions: list[tuple[int, int, int]] | list[tuple[int, int]],
    anchor: tuple[int, int],
) -> dict[str, tuple[int, int]]:
    """Append a south-facing splitter at `anchor` and return its port tiles.

    Returns a dict with keys `in0, in1` (input-port tiles, one row above
    the splitter) and `out0, out1` (output-drop tiles, one row below).
    Callers wire the upstream routes to terminate at the in-tiles and
    seed downstream routes from the out-tiles with `splitter_dir = S`
    so the existing source-lane forcing pins lanes correctly.

    The lane-balancer splitter is **placement-only** — the synth graph
    doesn't know about it. It shows up in the recovered topology as
    an extra splitter node, but `from_splitter_graph` checks degrees
    not counts and `verify_balancer` checks flow not structure, so the
    round-trip absorbs it cleanly.

    Use this at perpendicular turns where the sideload table would
    lump multiple lanes onto the receiver's LEFT lane and exceed the
    0.5 lane cap. The splitter eats the merged flow head-on (both
    lanes filled at the input port) and emits 50/50 on both output
    ports — physically restoring lane balance for the downstream
    channel.
    """
    x, y = anchor
    splitter_positions.append((x, y, 2))  # 2 = south
    return {
        "in0": (x, y - 1),
        "in1": (x + 1, y - 1),
        "out0": (x, y + 1),
        "out1": (x + 1, y + 1),
    }


def _input_routes_for_root(
    x_root: int, y_root: int, n: int
) -> tuple[
    list[tuple[int, int]],
    list[tuple[tuple[int, int], tuple[int, int], int, int | None]],
]:
    """Build input belt tiles and routes for `n ∈ {1, 2}` inputs.

    Returns `(input_tiles, routes)`. Each route is
    `(src, sink, sink_dir, splitter_dir)` for [`_route_belts`]; input
    boundary sources have `splitter_dir = None` (free-lane).

    Each direct input boundary belt is modeled as **two** length-1
    routes at the same tile. Real Factorio input belts arrive with
    items on both lanes (the upstream bus or balancer feeds both),
    so two routes at the same `(t, S)` sink tile force both lanes
    to be claimed via the per-lane cap (≤ 1 route per
    `(tile, dir, lane)`). Without this, single-route input modeling
    leaves the "other" lane of the boundary belt available for
    sideload — under-counting saturation.

    n=1: 2 lane-routes feeding port 0 (left tile of root).
    n=2: 4 lane-routes (2 per port).

    Larger n (≥3) needs an explicit merger sub-network in the synth
    output and is not yet supported. The earlier sideload variant
    for n=3 was removed after the per-lane cap correctly rejected it
    as lane-unsafe (saturation on the receiving belt).
    """
    DIR_S = 2
    direct_left = (x_root, y_root - 1)
    direct_right = (x_root + 1, y_root - 1)
    if n == 1:
        # Two routes at the same tile force both lanes to be claimed.
        return [direct_left], [
            (direct_left, direct_left, DIR_S, None),
            (direct_left, direct_left, DIR_S, None),
        ]
    if n == 2:
        tiles = [direct_left, direct_right]
        routes: list[
            tuple[tuple[int, int], tuple[int, int], int, int | None]
        ] = []
        for t in tiles:
            routes.append((t, t, DIR_S, None))
            routes.append((t, t, DIR_S, None))
        return tiles, routes
    raise ValueError(f"unsupported input count {n}; expected 1 or 2")


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

    solver = _make_solver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"({n}, 4) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(3)]
    x_root, y_root = pos[0]

    DIR_S = 2
    input_tiles, routes = _input_routes_for_root(x_root, y_root, n)
    # L1 → outputs: head-on south drops. Emit 2 lane-routes per arc so
    # the per-lane cap claims both lanes at the drop tile (matches the
    # input-boundary handling).
    for parent_idx in (1, 2):
        px, py = pos[parent_idx]
        for port in (0, 1):
            drop = (px + port, py + 1)
            routes.append((drop, drop, DIR_S, DIR_S))
            routes.append((drop, drop, DIR_S, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError(f"({n}, 4) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for p in pos:
        entities.append({"name": "splitter", "x": p[0], "y": p[1], "direction": _FACTORIO_DIR[_splitter_dir(p)]})
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

    solver = _make_solver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"({n}, 8) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_root, y_root = pos[0]

    DIR_S = 2
    input_tiles, routes = _input_routes_for_root(x_root, y_root, n)
    # Root → level-1: each side picks the closer L1 input port. The
    # drop tile picks E or W (perpendicular to the south splitter face),
    # so source-lane forcing pins the drop's lane via the sideload table.
    for parent_idx, port, child_idx in ((0, 0, 1), (0, 1, 2)):
        px, py = pos[parent_idx]
        cx, cy = pos[child_idx]
        drop = (px + port, py + 1)
        approach_port = 0 if drop[0] < cx else 1
        approach = (cx + approach_port, cy - 1)
        routes.append((drop, approach, DIR_S, DIR_S))
    # L1 → L2: tight-stack, no belt needed.
    # L2 → outputs: 8 head-on south drops, 2 lane-routes per arc.
    for parent_idx in (3, 4, 5, 6):
        px, py = pos[parent_idx]
        for port in (0, 1):
            drop = (px + port, py + 1)
            routes.append((drop, drop, DIR_S, DIR_S))
            routes.append((drop, drop, DIR_S, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError(f"({n}, 8) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for p in pos:
        entities.append({"name": "splitter", "x": p[0], "y": p[1], "direction": _FACTORIO_DIR[_splitter_dir(p)]})
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

    solver = _make_solver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError(f"({n}, 16) CP-SAT model UNSAT: {solver.status_name(status)}")

    pos = [(solver.value(xs[i]), solver.value(ys[i])) for i in range(n_splitters)]
    x_root, y_root = pos[0]

    DIR_S = 2
    input_tiles, routes = _input_routes_for_root(x_root, y_root, n)
    # Root → L1: perpendicular drop, source-lane pinned via sideload table.
    for parent_idx, port, child_idx in ((0, 0, 1), (0, 1, 2)):
        px, py = pos[parent_idx]
        cx, cy = pos[child_idx]
        drop = (px + port, py + 1)
        approach_port = 0 if drop[0] < cx else 1
        approach = (cx + approach_port, cy - 1)
        routes.append((drop, approach, DIR_S, DIR_S))
    # L1 → L2: routing-row offset, perpendicular drops.
    for parent_idx, child_left, child_right in ((1, 3, 4), (2, 5, 6)):
        px, py = pos[parent_idx]
        for port, child_idx in ((0, child_left), (1, child_right)):
            cx, cy = pos[child_idx]
            drop = (px + port, py + 1)
            approach_port = 0 if drop[0] < cx else 1
            approach = (cx + approach_port, cy - 1)
            routes.append((drop, approach, DIR_S, DIR_S))
    # L2 → L3: tight-stack, no belts.
    # L3 → outputs: 16 head-on south drops, 2 lane-routes per arc.
    for parent_idx in range(7, 15):
        px, py = pos[parent_idx]
        for port in (0, 1):
            drop = (px + port, py + 1)
            routes.append((drop, drop, DIR_S, DIR_S))
            routes.append((drop, drop, DIR_S, DIR_S))

    belt_dirs = _route_belts(pos, routes, width, height)
    if belt_dirs is None:
        raise RuntimeError(f"({n}, 16) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for p in pos:
        entities.append({"name": "splitter", "x": p[0], "y": p[1], "direction": _FACTORIO_DIR[_splitter_dir(p)]})
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
    seed = request.get("seed")

    # Configure the module-level solver params so every CP-SAT solve
    # in this run honours the request's timeout and seed. See
    # `_make_solver` for the wiring.
    _SOLVER_PARAMS["timeout_ms"] = timeout_ms
    _SOLVER_PARAMS["seed"] = seed

    # Stash synth context (graph + per-arc throughputs) so per-shape
    # placers can derive route rates for the rate-aware per-lane cap.
    # Both are optional — missing fields fall back to discrete unit
    # rates in `_route_belts`.
    _SYNTH_CTX["graph"] = request.get("graph")
    _SYNTH_CTX["arc_throughputs"] = request.get("arc_throughputs")

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
