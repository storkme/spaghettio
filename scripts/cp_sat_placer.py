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
) -> dict[tuple[int, int], tuple[int, str]] | None:
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

    Returns a dict mapping each used tile to `(direction_code, kind)`
    where `kind ∈ {"belt", "ug_in", "ug_out"}` — surface transport-belt,
    underground-belt input, or underground-belt output. Direction is
    one of 0/1/2/3 = N/E/S/W. Returns `None` if no routing is feasible.

    Splitter tiles are off-limits to belt entities. Multiple routes may
    share a free tile if they agree on belt direction and the per-lane
    cap is not exceeded — phase 2's lifted tile-exclusivity rule.

    Underground belts (yellow tier, `UG_MAX_REACH = 4`) are modeled
    globally: any route may use UG-pairs as alternative to surface belts
    (a route enters underground at an input tile and emerges at the
    paired output tile up to 4 tiles downstream). The tiles between
    entry and exit are unconstrained — surface belts of any direction
    can sit there. Two UG-pairs in the same direction whose tunnels
    overlap are forbidden (otherwise Factorio's auto-pairing rule would
    re-pair them). The objective penalises surface and UG entities
    equally; UG is picked only when it shortens the path or unlocks a
    crossing. v1 doesn't model lane-shared UGs (each UG pair carries
    one route on one lane).
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

    def in_bounds(t: tuple[int, int]) -> bool:
        return 0 <= t[0] < width and 0 <= t[1] < height

    model = cp_model.CpModel()

    # Per-tile per-direction entity occupancy. Each tile holds at most
    # one entity; the entity is one of three kinds heading some direction.
    # `belt[(t, d)]` is the union indicator (any kind heading d).
    surf = {
        (t, d): model.new_bool_var(f"surf_{t[0]}_{t[1]}_d{d}")
        for t in free
        for d in _INTERNAL_DIRS
    }
    ug_in = {
        (t, d): model.new_bool_var(f"ugin_{t[0]}_{t[1]}_d{d}")
        for t in free
        for d in _INTERNAL_DIRS
    }
    ug_out = {
        (t, d): model.new_bool_var(f"ugout_{t[0]}_{t[1]}_d{d}")
        for t in free
        for d in _INTERNAL_DIRS
    }
    belt = {
        (t, d): model.new_bool_var(f"belt_{t[0]}_{t[1]}_d{d}")
        for t in free
        for d in _INTERNAL_DIRS
    }
    for t in free:
        for d in _INTERNAL_DIRS:
            model.add(belt[(t, d)] == surf[(t, d)] + ug_in[(t, d)] + ug_out[(t, d)])
        # Tile uniqueness: at most one entity per tile (any direction,
        # any kind). Sum over all (d, kind) ≤ 1.
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

    # Per-route UG entry/exit indicators. Subset of f_lane: a route that
    # enters/exits UG at (t, d, lane) is also "present" there.
    ug_in_r = {
        (r, t, d, lane): model.new_bool_var(
            f"ugin_r{r}_{t[0]}_{t[1]}_d{d}_l{lane}"
        )
        for r in range(len(routes))
        for t in free
        for d in _INTERNAL_DIRS
        for lane in range(n_lanes)
    }
    ug_out_r = {
        (r, t, d, lane): model.new_bool_var(
            f"ugout_r{r}_{t[0]}_{t[1]}_d{d}_l{lane}"
        )
        for r in range(len(routes))
        for t in free
        for d in _INTERNAL_DIRS
        for lane in range(n_lanes)
    }
    for r in range(len(routes)):
        for t in free:
            for d in _INTERNAL_DIRS:
                for lane in range(n_lanes):
                    # UG role implies route presence.
                    model.add(
                        ug_in_r[(r, t, d, lane)] <= f_lane[(r, t, d, lane)]
                    )
                    model.add(
                        ug_out_r[(r, t, d, lane)] <= f_lane[(r, t, d, lane)]
                    )
                    # A given (route, tile, direction, lane) can be at
                    # most one of: UG entry, UG exit (or just surface).
                    model.add(
                        ug_in_r[(r, t, d, lane)] + ug_out_r[(r, t, d, lane)]
                        <= 1
                    )

    # UG-pair vars. `pair[(r, t1, k, d, lane)]` = 1 iff route r enters
    # UG at t1 with reach k in direction d on lane (exit at t1 + k·δ).
    # Constructed sparsely: only valid (entry, exit) pairs where exit is
    # in `free` and all interior tiles are in bounds.
    pair: dict[
        tuple[int, tuple[int, int], int, int, int], Any
    ] = {}
    for r in range(len(routes)):
        for t1 in free:
            for d in _INTERNAL_DIRS:
                dx, dy = _DELTAS[d]
                for k in range(2, UG_MAX_REACH + 1):
                    t2 = (t1[0] + k * dx, t1[1] + k * dy)
                    if t2 not in free_set:
                        continue
                    interior_ok = True
                    for i in range(1, k):
                        ti = (t1[0] + i * dx, t1[1] + i * dy)
                        if not in_bounds(ti):
                            interior_ok = False
                            break
                    if not interior_ok:
                        continue
                    for lane in range(n_lanes):
                        pair[(r, t1, k, d, lane)] = model.new_bool_var(
                            f"pair_r{r}_{t1[0]}_{t1[1]}_k{k}_d{d}_l{lane}"
                        )

    # Couple pair vars to per-route ug_in_r / ug_out_r.
    for r in range(len(routes)):
        for t in free:
            for d in _INTERNAL_DIRS:
                dx, dy = _DELTAS[d]
                for lane in range(n_lanes):
                    pairs_from_t = [
                        pair[(r, t, k, d, lane)]
                        for k in range(2, UG_MAX_REACH + 1)
                        if (r, t, k, d, lane) in pair
                    ]
                    if pairs_from_t:
                        model.add(
                            ug_in_r[(r, t, d, lane)] == sum(pairs_from_t)
                        )
                    else:
                        model.add(ug_in_r[(r, t, d, lane)] == 0)
                    pairs_to_t = []
                    for k in range(2, UG_MAX_REACH + 1):
                        from_t = (t[0] - k * dx, t[1] - k * dy)
                        if (r, from_t, k, d, lane) in pair:
                            pairs_to_t.append(pair[(r, from_t, k, d, lane)])
                    if pairs_to_t:
                        model.add(
                            ug_out_r[(r, t, d, lane)] == sum(pairs_to_t)
                        )
                    else:
                        model.add(ug_out_r[(r, t, d, lane)] == 0)

    # Couple per-tile ug_in / ug_out to per-route ug_in_r / ug_out_r
    # (OR-style: per-tile = 1 iff some route uses it).
    for t in free:
        for d in _INTERNAL_DIRS:
            in_route_uses = [
                ug_in_r[(r, t, d, lane)]
                for r in range(len(routes))
                for lane in range(n_lanes)
            ]
            out_route_uses = [
                ug_out_r[(r, t, d, lane)]
                for r in range(len(routes))
                for lane in range(n_lanes)
            ]
            big_m = max(1, len(routes) * n_lanes)
            s_in = sum(in_route_uses)
            model.add(s_in >= ug_in[(t, d)])
            model.add(s_in <= big_m * ug_in[(t, d)])
            s_out = sum(out_route_uses)
            model.add(s_out >= ug_out[(t, d)])
            model.add(s_out <= big_m * ug_out[(t, d)])

    # Same-direction tunnel non-overlap: at most one tunnel covers any
    # given tile in any given direction. A tunnel covers tiles
    # `t1, t1+δ, ..., t1+k·δ` (k+1 tiles inclusive). Two same-direction
    # tunnels whose coverage intervals overlap would confuse Factorio's
    # auto-pairing rule (the entry pairs with the nearest exit), so we
    # forbid the overlap up front.
    for d in _INTERNAL_DIRS:
        dx, dy = _DELTAS[d]
        for t in free:
            covers: list[Any] = []
            for r in range(len(routes)):
                for lane in range(n_lanes):
                    for k in range(2, UG_MAX_REACH + 1):
                        for offset in range(k + 1):
                            t1 = (t[0] - offset * dx, t[1] - offset * dy)
                            if (r, t1, k, d, lane) in pair:
                                covers.append(pair[(r, t1, k, d, lane)])
            if covers:
                model.add(sum(covers) <= 1)

    # Per-lane cap: weighted sum of route rates ≤ lane_cap. Applies at
    # all tiles (surface, UG entry, UG exit) — items still have lane
    # discipline through the underground.
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

    # belt[(t, d)] = OR over (r, lane) of f_lane[(r, t, d, lane)]:
    # the entity heading d at t exists iff some route is at t heading d.
    for t in free:
        for d in _INTERNAL_DIRS:
            s = sum(
                f_lane[(r, t, d, lane)]
                for r in range(len(routes))
                for lane in range(n_lanes)
            )
            model.add(s >= belt[(t, d)])
            model.add(s <= len(routes) * belt[(t, d)])

    # Each route uses at most one (direction, lane) per tile.
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

    # Forbid UG roles at each route's source and sink. The source is a
    # surface drop from a splitter (or boundary input belt); the sink
    # is a surface drop into a downstream splitter. Going underground
    # at either endpoint would break source-lane forcing or sink-direction
    # constraints.
    for r, (src, sink, _sink_dir, _splitter_dir) in enumerate(routes):
        for d in _INTERNAL_DIRS:
            for lane in range(n_lanes):
                model.add(ug_in_r[(r, src, d, lane)] == 0)
                model.add(ug_out_r[(r, src, d, lane)] == 0)
                model.add(ug_in_r[(r, sink, d, lane)] == 0)
                model.add(ug_out_r[(r, sink, d, lane)] == 0)

    # Per-route flow conservation, per-lane.
    # Items at tile t (direction d_recv, lane L_recv) arrive from a
    # neighbor n at side s of t whose belt heads d_pred = OPPOSITE(s).
    # The lane the items land on at t depends on s relative to d_recv:
    #   - s = OPPOSITE(d_recv) (back / natural input): lane preserved
    #     from predecessor.
    #   - s = LEFT_OF(d_recv) (left side feed): items land on lane 0.
    #   - s = RIGHT_OF(d_recv) (right side feed): items land on lane 1.
    #   - s = d_recv (forward / output side): items can't enter.
    # UG handling layered on top:
    #   - At a UG entry tile (ug_in[(t, d_recv)] = 1): inflow comes via
    #     surface predecessors (route arrives on surface, descends).
    #   - At a UG exit tile (ug_out[(t, d_recv)] = 1): inflow is the
    #     teleported flow from the paired entry; surface predecessors
    #     don't feed an exit (Factorio physics).
    #   - At an entry, the route's outflow doesn't continue on surface
    #     in d (it teleports), so the surface successor check is
    #     skipped when ug_in_r is set.
    for r, (src, sink, sink_dir, splitter_dir) in enumerate(routes):
        # Sink belt direction is fixed; sink is surface (we forbade UG
        # roles there above, so `surf[(sink, sink_dir)] = 1` here).
        model.add(surf[(sink, sink_dir)] == 1)
        model.add(f_sum(r, sink, sink_dir) == 1)

        if src != sink:
            model.add(
                sum(
                    f_lane[(r, src, d, lane)]
                    for d in _INTERNAL_DIRS
                    for lane in range(n_lanes)
                )
                == 1
            )

        if splitter_dir is not None:
            for d in _INTERNAL_DIRS:
                if d == splitter_dir:
                    continue
                if d == _LEFT_OF[splitter_dir]:
                    model.add(f_lane[(r, src, d, 1)] == 0)
                elif d == _RIGHT_OF[splitter_dir]:
                    model.add(f_lane[(r, src, d, 0)] == 0)
                elif d == _OPPOSITE[splitter_dir]:
                    # Belt heading back into the splitter face would push
                    # items into the splitter front — invalid in Factorio
                    # and breaks topology recovery (creates self-loops).
                    for lane in range(n_lanes):
                        model.add(f_lane[(r, src, d, lane)] == 0)

        for t in free:
            for d_recv in _INTERNAL_DIRS:
                for L_recv in range(n_lanes):
                    in_flows = []
                    for s in _INTERNAL_DIRS:
                        if s == d_recv:
                            continue
                        sdx, sdy = _DELTAS[s]
                        n = (t[0] + sdx, t[1] + sdy)
                        if n not in free_set:
                            continue
                        d_pred = _OPPOSITE[s]
                        if s == _OPPOSITE[d_recv]:
                            in_flows.append(f_lane[(r, n, d_pred, L_recv)])
                        elif s == _LEFT_OF[d_recv]:
                            if L_recv == 0:
                                for L in range(n_lanes):
                                    in_flows.append(f_lane[(r, n, d_pred, L)])
                        elif s == _RIGHT_OF[d_recv]:
                            if L_recv == 1:
                                for L in range(n_lanes):
                                    in_flows.append(f_lane[(r, n, d_pred, L)])

                    if t == src:
                        if in_flows:
                            model.add(sum(in_flows) == 0).only_enforce_if(
                                belt[(t, d_recv)]
                            )
                    else:
                        # Three mutually-exclusive cases by entity type:
                        #   surf  : f_lane == sum(in_flows)
                        #   ug_in : f_lane == sum(in_flows) (route arrives
                        #           on surface, then descends)
                        #   ug_out: f_lane == ug_out_r (teleport from
                        #           paired entry; no surface predecessors)
                        # When no entity at (t, d_recv): f_lane is forced
                        # to 0 by the `belt = OR f_lane` aggregator.
                        model.add(
                            sum(in_flows) == f_lane[(r, t, d_recv, L_recv)]
                        ).only_enforce_if(surf[(t, d_recv)])
                        model.add(
                            sum(in_flows) == f_lane[(r, t, d_recv, L_recv)]
                        ).only_enforce_if(ug_in[(t, d_recv)])
                        model.add(
                            f_lane[(r, t, d_recv, L_recv)]
                            == ug_out_r[(r, t, d_recv, L_recv)]
                        ).only_enforce_if(ug_out[(t, d_recv)])

            # Outflow: surface continuation. Skipped when the route
            # enters UG here (ug_in_r=1), since the route teleports
            # rather than continuing to the next surface tile.
            if t != sink:
                for d in _INTERNAL_DIRS:
                    ndx, ndy = _DELTAS[d]
                    nxt = (t[0] + ndx, t[1] + ndy)
                    if nxt not in free_set:
                        for lane in range(n_lanes):
                            model.add(f_lane[(r, t, d, lane)] == 0)
                    else:
                        on_route_nxt = sum(
                            f_sum(r, nxt, dd) for dd in _INTERNAL_DIRS
                        )
                        for lane in range(n_lanes):
                            model.add(on_route_nxt >= 1).only_enforce_if(
                                [
                                    f_lane[(r, t, d, lane)],
                                    ug_in_r[(r, t, d, lane)].Not(),
                                ]
                            )

    # Minimise total entity count (surface + UG entries + UG exits each
    # contribute 1). UG is picked only when it shortens the path or
    # unlocks a crossing — for shapes that don't need it, the optimiser
    # leaves UG vars at 0 and the surface-only model is recovered.
    model.minimize(
        sum(belt[(t, d)] for t in free for d in _INTERNAL_DIRS)
    )

    solver = _make_solver()
    status = solver.solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        print(
            f"_route_belts: status={solver.status_name(status)} "
            f"wall={solver.wall_time:.1f}s",
            file=sys.stderr,
        )
        return None

    out: dict[tuple[int, int], tuple[int, str]] = {}
    for t in free:
        for d in _INTERNAL_DIRS:
            if solver.value(surf[(t, d)]):
                out[t] = (d, "belt")
                break
            if solver.value(ug_in[(t, d)]):
                out[t] = (d, "ug_in")
                break
            if solver.value(ug_out[(t, d)]):
                out[t] = (d, "ug_out")
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

# Yellow underground-belt max reach: entry and exit can be up to 4 tiles
# apart (i.e., exit = entry + k·δ for k ∈ {2, 3, 4}; k=1 is illegal as
# the exit would touch the entry). Higher tiers (red=6, blue=8) are
# deferred — the placer emits yellow undergrounds and the bus engine
# upgrades them at stamping time if a faster belt tier is in use.
UG_MAX_REACH = 4


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
    for (x, y), (d, kind) in belt_dirs.items():
        if kind == "belt":
            entities.append(
                {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
            )
        elif kind == "ug_in":
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "input",
                }
            )
        else:  # ug_out
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "output",
                }
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
    for (x, y), (d, kind) in belt_dirs.items():
        if kind == "belt":
            entities.append(
                {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
            )
        elif kind == "ug_in":
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "input",
                }
            )
        else:  # ug_out
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "output",
                }
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
    for (x, y), (d, kind) in belt_dirs.items():
        if kind == "belt":
            entities.append(
                {"name": "transport-belt", "x": x, "y": y, "direction": _FACTORIO_DIR[d]}
            )
        elif kind == "ug_in":
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "input",
                }
            )
        else:  # ug_out
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "output",
                }
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
    """`(1, 5)` coprime balancer — first non-dyadic shape.

    Synth structure (see ``balancer::synth``):
      - S0 (M, merger): in0 = 1.0 external, in1 = 0.6 feedback.
        Outputs 0.8 each to S1.
      - S1 (root): two head-on inputs (0.8 each from M).
        Outputs 0.8 each to S2 / S3.
      - S2, S3 (L1): each takes 0.8 head-on. Outputs 0.4 each to L2.
      - S4-S7 (L2): each takes 0.4. S4/S5 emit two outputs; S6 emits one
        output + one feedback; S7 emits two feedbacks.

    The naive "M as merger at row 0" layout UNSATs because M.in1 at the
    top edge has only east-side feeds available (M.in0 occupies the west
    neighbour), so the 0.6 feedback lumps onto a single lane → exceeds
    the 0.5 lane cap. We sidestep this by inserting an extra
    south-facing splitter B' (placement-only) above M that takes the
    feedback head-on (split across two input ports, fed from west AND
    east sides) and rebalances onto its outputs. B's outputs feed M.in1
    via the back-feed + east-sideload trick documented in
    ``rfp-lane-aware-routing.md``.

    Recovery sees 9 splitters (8 from synth + 1 lane-balancer). The
    extra splitter shows up as a regular 1→2 node in the recovered graph
    and is absorbed by ``from_splitter_graph`` / ``verify_balancer``
    without special handling — both deal in flow-balance, not splitter
    counts.

    Layout (south-facing throughout, width 11 × height 12):

    ::

        Row 0–1: feedback wrap channels (from L2 row 9 back up).
        Row 2: B' inputs at (5, 2), (6, 2). Feedback merges here.
        Row 3: B' body at (5, 3)-(6, 3) (placement-only splitter).
        Row 4: B' outputs land here. (5, 4) S → M.in1.
               (6, 4) W → sideload (5, 4) east, balancing M.in1 lanes.
               External input boundary at (4, 4) S → drops into M.in0.
        Row 5: M body at (4, 5)-(5, 5).
        Row 6: S1 body at (4, 6)-(5, 6) (tight-stack).
        Row 7: S2=(3, 7), S3=(5, 7) staggered tight-stack.
        Row 8: L1→L2 routing row.
        Row 9: L2 — A=(1, 9), B=(3, 9), C=(5, 9), D=(7, 9).
        Row 10: 5 outputs (cols 1-5) + 3 feedback drops (cols 6, 7, 8).
        Row 11: feedback channel south-leg.

    L1→L2 arc assignment (avoids the routing-row crossing that blocked
    the original "M at row 0" attempt — see RFP decision log entries):

      - S2.out0 (3, 8) heads west, drops south into A at (1, 8).
      - S2.out1 (4, 8) head-on south into B (tight-stacked).
      - S3.out0 (5, 8) head-on south into C (tight-stacked).
      - S3.out1 (6, 8) heads east, drops south into D at (7, 8).

    Rate-aware encoding: this placer uses a finer scale than the default
    ``RATE_SCALE = 10`` so the lane balancer's 0.3 outputs (0.15 per
    lane in head-on encoding) round to integers. Local
    ``LOCAL_RATE_SCALE = 20`` and ``lane_cap = 10``.
    """
    width, height = 11, 12
    DIR_S = 2

    LOCAL_RATE_SCALE = 20
    LOCAL_LANE_CAP = LOCAL_RATE_SCALE // 2  # = 10

    def r(absolute: float) -> int:
        """Scaled rate at the local scale."""
        return int(round(absolute * LOCAL_RATE_SCALE))

    M = (4, 5)
    Bprime = (5, 3)
    S1 = (4, 6)
    S2 = (3, 7)
    S3 = (5, 7)
    A = (1, 9)
    B_pos = (3, 9)
    C = (5, 9)
    D = (7, 9)
    pos: list[tuple[int, int] | tuple[int, int, int]] = [
        M, Bprime, S1, S2, S3, A, B_pos, C, D,
    ]

    routes: list[
        tuple[tuple[int, int], tuple[int, int], int, int | None]
    ] = []
    rates: list[int] = []

    # External input boundary at (4, 4) S — 2 lane-routes pin both lanes.
    routes.append(((4, 4), (4, 4), DIR_S, None))
    routes.append(((4, 4), (4, 4), DIR_S, None))
    rates.extend([r(0.5), r(0.5)])

    # B' → M.in1 connection. B' total input = 0.6, so each B' output
    # carries 0.3.
    # B'.out0 lands at (5, 4) heading S (head-on, 2 lane-routes each at 0.15).
    # B'.out1 lands at (6, 4) heading W (perpendicular, 1 lane-route at 0.3).
    # (6, 4) W feeds (5, 4) east-side → sideload onto lane 0 of (5, 4).
    routes.append(((5, 4), (5, 4), DIR_S, DIR_S))
    routes.append(((5, 4), (5, 4), DIR_S, DIR_S))
    rates.extend([r(0.15), r(0.15)])
    routes.append(((6, 4), (5, 4), DIR_S, DIR_S))
    rates.append(r(0.3))

    # L1 → L2 arcs (drops at row 8, L2 inputs at row 8).
    # S2.out0 → A: perpendicular west drop, route to (1, 8) S sink.
    routes.append(((3, 8), (1, 8), DIR_S, DIR_S))
    rates.append(r(0.4))
    # S2.out1 → B: head-on south at (4, 8). 2 lane-routes (0.2 per lane).
    routes.append(((4, 8), (4, 8), DIR_S, DIR_S))
    routes.append(((4, 8), (4, 8), DIR_S, DIR_S))
    rates.extend([r(0.2), r(0.2)])
    # S3.out0 → C: head-on south at (5, 8). 2 lane-routes.
    routes.append(((5, 8), (5, 8), DIR_S, DIR_S))
    routes.append(((5, 8), (5, 8), DIR_S, DIR_S))
    rates.extend([r(0.2), r(0.2)])
    # S3.out1 → D: perpendicular east drop, route to (7, 8) S sink.
    routes.append(((6, 8), (7, 8), DIR_S, DIR_S))
    rates.append(r(0.4))

    # L2 → outputs (5 head-on south drops at row 10).
    output_drops: list[tuple[int, int]] = [
        (1, 10), (2, 10), (3, 10), (4, 10), (5, 10),
    ]
    for drop in output_drops:
        routes.append((drop, drop, DIR_S, DIR_S))
        routes.append((drop, drop, DIR_S, DIR_S))
        rates.extend([r(0.1), r(0.1)])

    # 3 feedback arcs from L2 outputs back to B' inputs at row 2.
    # Two drops feed B'.in0 at (5, 2) (total 0.4); one drop feeds
    # B'.in1 at (6, 2) (total 0.2).
    feedback_drops_to_in0: list[tuple[int, int]] = [(6, 10), (7, 10)]
    feedback_drops_to_in1: list[tuple[int, int]] = [(8, 10)]
    for src in feedback_drops_to_in0:
        routes.append((src, (5, 2), DIR_S, DIR_S))
        rates.append(r(0.2))
    for src in feedback_drops_to_in1:
        routes.append((src, (6, 2), DIR_S, DIR_S))
        rates.append(r(0.2))

    belt_dirs = _route_belts(
        pos,
        routes,
        width,
        height,
        rates=rates,
        lane_cap=LOCAL_LANE_CAP,
    )
    if belt_dirs is None:
        raise RuntimeError("(1, 5) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for p in pos:
        entities.append(
            {
                "name": "splitter",
                "x": p[0],
                "y": p[1],
                "direction": _FACTORIO_DIR[_splitter_dir(p)],
            }
        )
    for (x, y), (d, kind) in belt_dirs.items():
        if kind == "belt":
            entities.append(
                {
                    "name": "transport-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                }
            )
        elif kind == "ug_in":
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "input",
                }
            )
        else:  # ug_out
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "output",
                }
            )

    return {
        "n_inputs": 1,
        "n_outputs": 5,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [[4, 4]],
        "output_tiles": [[c, 10] for c in (1, 2, 3, 4, 5)],
    }


def place_one_to_six() -> dict[str, Any]:
    """`(1, 6)` coprime balancer.

    Synth structure: 8 splitters, identical topology to `(1, 5)` —
    M (merger), S1 (root), S2/S3 (L1), S4-S7 (L2). Only the leaf
    arc assignment differs: S4/S5/S6 each emit two outputs (= 6 outputs
    total), and S7 emits two feedback arcs.

    Rates: M outs at 0.667, S1 outs at 0.667, L1 outs at 0.333, L2 outs
    at 0.167. Feedback total = 2 × 0.167 = 0.333 — fits a single
    east-side sideload onto lane 0 of M.in1, so no lane balancer is
    needed (unlike `(1, 5)`).

    Local rate scale `12` so 0.667 (= 8/12), 0.333 (= 4/12), 0.167 (= 2/12),
    and 0.0833 (per-lane head-on for output drops, = 1/12) all encode
    as integers.

    Layout reuses the `(1, 5)` upper structure verbatim:

    ::

        Row 0: external input (4, 0).
        Row 1: M at (4, 1).
        Row 2: S1 at (4, 2) tight-stack.
        Row 3: S2 (3, 3), S3 (5, 3) staggered tight-stack.
        Row 4: L1→L2 routing row.
        Row 5: L2 splitters A=(1, 5), B=(3, 5), C=(5, 5), D=(7, 5).
        Row 6: 6 outputs (cols 1-6) + 2 feedback drops (cols 7, 8).
        Row 7+: feedback channel wraps east+north back to (5, 0).
    """
    # Tighter grid than `(1, 5)` because no lane balancer is needed —
    # feedback total 0.333 fits a single east-side sideload into M.in1.
    width, height = 10, 9
    DIR_S = 2

    LOCAL_RATE_SCALE = 12
    LOCAL_LANE_CAP = LOCAL_RATE_SCALE // 2  # = 6 (= 0.5 absolute)

    def r(numer: int, denom: int) -> int:
        # Exact fraction encoding to avoid float rounding issues for 1/3, 1/6, 1/12.
        scaled = numer * LOCAL_RATE_SCALE
        if scaled % denom != 0:
            raise ValueError(
                f"rate {numer}/{denom} doesn't fit scale {LOCAL_RATE_SCALE}"
            )
        return scaled // denom

    M = (4, 1)
    S1 = (4, 2)
    S2 = (3, 3)
    S3 = (5, 3)
    A = (1, 5)
    B_pos = (3, 5)
    C = (5, 5)
    D = (7, 5)
    pos: list[tuple[int, int] | tuple[int, int, int]] = [
        M, S1, S2, S3, A, B_pos, C, D,
    ]

    routes: list[
        tuple[tuple[int, int], tuple[int, int], int, int | None]
    ] = []
    rates: list[int] = []

    # External input boundary at (4, 0) S — 2 lane-routes pin both lanes.
    routes.append(((4, 0), (4, 0), DIR_S, None))
    routes.append(((4, 0), (4, 0), DIR_S, None))
    rates.extend([r(1, 2), r(1, 2)])  # 0.5 per lane

    # L1 → L2 arcs (rate 1/3 each).
    # S2.out0 → A: perpendicular west drop, route to (1, 4) S sink.
    routes.append(((3, 4), (1, 4), DIR_S, DIR_S))
    rates.append(r(1, 3))
    # S2.out1 → B: head-on south at (4, 4). 2 lane-routes (1/6 per lane).
    routes.append(((4, 4), (4, 4), DIR_S, DIR_S))
    routes.append(((4, 4), (4, 4), DIR_S, DIR_S))
    rates.extend([r(1, 6), r(1, 6)])
    # S3.out0 → C: head-on south at (5, 4). 2 lane-routes.
    routes.append(((5, 4), (5, 4), DIR_S, DIR_S))
    routes.append(((5, 4), (5, 4), DIR_S, DIR_S))
    rates.extend([r(1, 6), r(1, 6)])
    # S3.out1 → D: perpendicular east drop, route to (7, 4) S sink.
    routes.append(((6, 4), (7, 4), DIR_S, DIR_S))
    rates.append(r(1, 3))

    # L2 → outputs (6 head-on south drops at row 6 across A, B, C).
    output_drops: list[tuple[int, int]] = [
        (1, 6), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6),
    ]
    for drop in output_drops:
        routes.append((drop, drop, DIR_S, DIR_S))
        routes.append((drop, drop, DIR_S, DIR_S))
        rates.extend([r(1, 12), r(1, 12)])  # 1/12 per lane head-on

    # 2 feedback arcs from D back to M.in1 at (5, 0).
    feedback_drops: list[tuple[int, int]] = [(7, 6), (8, 6)]
    for src in feedback_drops:
        routes.append((src, (5, 0), DIR_S, DIR_S))
        rates.append(r(1, 6))

    belt_dirs = _route_belts(
        pos,
        routes,
        width,
        height,
        rates=rates,
        lane_cap=LOCAL_LANE_CAP,
    )
    if belt_dirs is None:
        raise RuntimeError("(1, 6) belt routing UNSAT")

    entities: list[dict[str, Any]] = []
    for p in pos:
        entities.append(
            {
                "name": "splitter",
                "x": p[0],
                "y": p[1],
                "direction": _FACTORIO_DIR[_splitter_dir(p)],
            }
        )
    for (x, y), (d, kind) in belt_dirs.items():
        if kind == "belt":
            entities.append(
                {
                    "name": "transport-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                }
            )
        elif kind == "ug_in":
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "input",
                }
            )
        else:  # ug_out
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "output",
                }
            )

    return {
        "n_inputs": 1,
        "n_outputs": 6,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [[4, 0]],
        "output_tiles": [[c, 6] for c in (1, 2, 3, 4, 5, 6)],
    }


def _src_splitter(arc: dict[str, Any]) -> tuple[int, int] | None:
    """Return (idx, port) if arc src is a splitter, else None."""
    s = arc.get("src", {})
    if isinstance(s, dict) and "Splitter" in s:
        sp = s["Splitter"]
        return (sp["idx"], sp["port"])
    return None


def _dst_splitter(arc: dict[str, Any]) -> tuple[int, int] | None:
    """Return (idx, port) if arc dst is a splitter, else None."""
    d = arc.get("dst", {})
    if isinstance(d, dict) and "Splitter" in d:
        sp = d["Splitter"]
        return (sp["idx"], sp["port"])
    return None


def _dst_output(arc: dict[str, Any]) -> int | None:
    """Return output id if arc dst is an output, else None."""
    d = arc.get("dst", {})
    if isinstance(d, dict) and "Output" in d:
        return d["Output"]
    return None


def _pick_rate_scale(rates: list[float]) -> int:
    """Smallest integer scale that makes every absolute rate in `rates` an
    integer when multiplied. Uses Python's `Fraction` to extract exact
    denominators (CP-SAT gives us floats with rounding error, but the
    rates are rational with small denominators in practice).
    """
    from fractions import Fraction
    from math import gcd

    def lcm(a: int, b: int) -> int:
        return a * b // gcd(a, b)

    scale = 1
    for r in rates:
        if r <= 0:
            continue
        # `limit_denominator(1000)` snaps the float back to its rational form
        # — synth rates have denominators dividing m × something small.
        denom = Fraction(r).limit_denominator(1000).denominator
        scale = lcm(scale, denom)
    return scale


def place_one_to_m_from_synth() -> dict[str, Any]:
    """Generalised `(1, m)` placer driven by the synth graph.

    Handles 3-level trees (8 splitters: M, S1, S2/S3, four L2 leaves) for
    `m ∈ {5, 6, 7}` — the existing `(1, 5)` and `(1, 6)` hand-tuned
    placers should be functionally equivalent under this routine. Deeper
    trees (`(1, 9)`, `(1, 10)` with their L3 layer) are NOT yet
    supported here; they raise `NotImplementedError` for the dispatcher
    to surface as `unimplemented`.

    Algorithm: walk the synth in topological order (mutable Kahn's),
    placing splitters at offsets dictated by the parent arc's rate:
      - rate > 0.5 → must head-on. M→S1 is vertical tight-stack (both M
        outputs feed S1.in0/S1.in1). S1→S2/S3 is staggered tight-stack.
      - rate ≤ 0.5 → either works. L1→L2 uses the inner-tight-stack +
        outer-perpendicular pattern from `place_one_to_five`.

    Feedback: if the sum of arcs targeting M.in1 exceeds 0.5 (the lane
    cap), insert a placement-only south-facing splitter B' above M and
    partition the feedback drops across B's two input ports. B's outputs
    feed M.in1 via north back-feed + east-side sideload (the `(1, 5)`
    trick).

    Rate scale: derived from the LCM of all per-route rate denominators
    (including head-on per-lane halves and B' output halves when
    relevant). For `(1, 5)` this gives 20; for `(1, 6)`, 12; for
    `(1, 7)`, 14.
    """
    graph = _SYNTH_CTX.get("graph")
    throughputs = _SYNTH_CTX.get("arc_throughputs")
    if graph is None or throughputs is None:
        raise RuntimeError(
            "place_one_to_m_from_synth: missing synth context (graph / arc_throughputs)"
        )

    n_splitters = graph["n_splitters"]
    if n_splitters != 8:
        raise NotImplementedError(
            f"place_one_to_m_from_synth handles only 3-level (8-splitter) trees; "
            f"got {n_splitters} splitters"
        )

    arcs: list[dict[str, Any]] = graph["arcs"]
    n_outputs = graph["n_outputs"]

    # ----- Index outgoing arcs per splitter port -----
    # out_arc[(s_idx, port)] = arc index of the arc with that source.
    out_arc: dict[tuple[int, int], int] = {}
    for i, arc in enumerate(arcs):
        sp = _src_splitter(arc)
        if sp is not None:
            out_arc[sp] = i

    # ----- Identify feedback arcs (target = M.in1 = (0, 1)) -----
    feedback_arc_indices = [
        i
        for i, arc in enumerate(arcs)
        if _dst_splitter(arc) == (0, 1)
    ]
    feedback_total = sum(throughputs[i] for i in feedback_arc_indices)
    needs_balancer = feedback_total > 0.5

    # ----- Pick rate scale -----
    # All absolute rates we'll need to encode integrally.
    needed_rates: list[float] = list(throughputs)
    # Include per-lane halves (head-on encoding) for every arc rate.
    needed_rates.extend(r / 2.0 for r in throughputs)
    if needs_balancer:
        # B' takes feedback_total in, splits to feedback_total/2 per output,
        # which encodes head-on as feedback_total/4 per lane.
        b_out = feedback_total / 2.0
        needed_rates.extend([b_out, b_out / 2.0])
    needed_rates.append(0.5)  # input-boundary per-lane rate
    scale = _pick_rate_scale(needed_rates)
    lane_cap = scale // 2

    def r(absolute: float) -> int:
        return int(round(absolute * scale))

    # ----- Splitter placement -----
    # Uniform layout: M at (4, M_y) where M_y depends on whether B' is needed.
    # L2 splitters at (1, 5, 7) with anchor cols (1, 3, 5, 7), all at a
    # row 4 below M (so output row is 5 below M).
    M_X = 4
    if needs_balancer:
        M_Y = 5  # rows 0-4 reserved for B' + B's input feeds + feedback wrap top.
    else:
        M_Y = 1  # M.in tiles at row 0 (top edge → external input boundary).

    DIR_S = 2

    # Splitter positions, with `pos[idx]` matching synth splitter index.
    pos: list[tuple[int, int] | tuple[int, int, int]] = [(0, 0)] * n_splitters
    pos[0] = (M_X, M_Y)               # M
    pos[1] = (M_X, M_Y + 1)            # S1 (vertical tight-stack under M)
    pos[2] = (M_X - 1, M_Y + 2)        # S2 (staggered tight-stack left)
    pos[3] = (M_X + 1, M_Y + 2)        # S3 (staggered tight-stack right)
    # L2 row at M_Y + 4 (with row M_Y + 3 as routing row between L1 and L2).
    L2_Y = M_Y + 4
    # L2 col mapping: synth assigns S4=L2_outer_left (S2.out0), S5=L2_inner_left
    # (S2.out1), S6=L2_inner_right (S3.out0), S7=L2_outer_right (S3.out1).
    pos[4] = (M_X - 3, L2_Y)           # outer-left  (receives S2.out0)
    pos[5] = (M_X - 1, L2_Y)           # inner-left  (receives S2.out1)
    pos[6] = (M_X + 1, L2_Y)           # inner-right (receives S3.out0)
    pos[7] = (M_X + 3, L2_Y)           # outer-right (receives S3.out1)

    # Optional B' lane balancer above M.
    if needs_balancer:
        # B' at (M_X + 1, M_Y - 2) span (M_X+1, M_X+2). Its outputs land at
        # (M_X+1, M_Y - 1) and (M_X+2, M_Y - 1). M.in1 at (M_X+1, M_Y - 1)
        # = B'.out0 (back-feed). B'.out1 at (M_X+2, M_Y - 1) heads W to
        # sideload M.in1 east-side onto lane 0.
        bprime_pos = (M_X + 1, M_Y - 2, DIR_S)
        pos.append(bprime_pos)
    bprime_idx = n_splitters if needs_balancer else None

    # ----- Routes -----
    routes: list[
        tuple[tuple[int, int], tuple[int, int], int, int | None]
    ] = []
    rates: list[int] = []

    # 1. External input boundary at M.in0 = (M_X, M_Y - 1).
    input_tile = (M_X, M_Y - 1)
    routes.append((input_tile, input_tile, DIR_S, None))
    routes.append((input_tile, input_tile, DIR_S, None))
    rates.extend([r(0.5), r(0.5)])

    # 2. M → S1: tight-stack vertical (no route — splitter geometry adjacency).
    # 3. S1 → S2/S3: staggered tight-stack (no route — adjacency).

    # 4. L1 → L2 arcs (4 arcs, parent = S2 or S3).
    # Mapping per synth indexing (verified against (1, 5) / (1, 6) dumps):
    #   arc src (S2.out0) → L2 idx 4 (outer-left A): perpendicular west drop.
    #   arc src (S2.out1) → L2 idx 5 (inner-left B): head-on tight-stack south.
    #   arc src (S3.out0) → L2 idx 6 (inner-right C): head-on tight-stack south.
    #   arc src (S3.out1) → L2 idx 7 (outer-right D): perpendicular east drop.
    L1_drop_y = pos[2][1] + 1  # row below L1
    # S2.out0 → L2[4] (outer west).
    s2_out0_drop = (pos[2][0], L1_drop_y)  # (M_X-1, ...)
    s2_out0_sink = (pos[4][0], L1_drop_y)  # (M_X-3, ...)
    arc_s2_0 = out_arc[(2, 0)]
    routes.append((s2_out0_drop, s2_out0_sink, DIR_S, DIR_S))
    rates.append(r(throughputs[arc_s2_0]))
    # S2.out1 → L2[5] (inner head-on).
    s2_out1_tile = (pos[2][0] + 1, L1_drop_y)  # (M_X, ...)
    arc_s2_1 = out_arc[(2, 1)]
    half = throughputs[arc_s2_1] / 2.0
    routes.append((s2_out1_tile, s2_out1_tile, DIR_S, DIR_S))
    routes.append((s2_out1_tile, s2_out1_tile, DIR_S, DIR_S))
    rates.extend([r(half), r(half)])
    # S3.out0 → L2[6] (inner head-on).
    s3_out0_tile = (pos[3][0], L1_drop_y)  # (M_X+1, ...)
    arc_s3_0 = out_arc[(3, 0)]
    half = throughputs[arc_s3_0] / 2.0
    routes.append((s3_out0_tile, s3_out0_tile, DIR_S, DIR_S))
    routes.append((s3_out0_tile, s3_out0_tile, DIR_S, DIR_S))
    rates.extend([r(half), r(half)])
    # S3.out1 → L2[7] (outer east).
    s3_out1_drop = (pos[3][0] + 1, L1_drop_y)  # (M_X+2, ...)
    s3_out1_sink = (pos[7][0], L1_drop_y)      # (M_X+3, ...)
    arc_s3_1 = out_arc[(3, 1)]
    routes.append((s3_out1_drop, s3_out1_sink, DIR_S, DIR_S))
    rates.append(r(throughputs[arc_s3_1]))

    # 5. L2 → output / feedback: emit one route per L2 output.
    # For each L2 splitter idx 4..7, look up its 2 outgoing arcs. If dst is
    # Output, emit head-on south drop (2 lane-routes). If dst is M.in1
    # (feedback), emit perpendicular route to the appropriate sink.
    output_drop_y = L2_Y + 1
    output_drops: list[tuple[int, int]] = []
    feedback_drops: list[tuple[tuple[int, int], int]] = []  # (drop_tile, arc_idx)

    for l2_idx in (4, 5, 6, 7):
        l2_anchor_x = pos[l2_idx][0]
        for port in (0, 1):
            arc_idx = out_arc[(l2_idx, port)]
            arc = arcs[arc_idx]
            drop_x = l2_anchor_x + port
            drop_tile = (drop_x, output_drop_y)
            arc_rate = throughputs[arc_idx]
            if _dst_output(arc) is not None:
                # Output drop: head-on south, 2 lane-routes at half-rate.
                half = arc_rate / 2.0
                routes.append((drop_tile, drop_tile, DIR_S, DIR_S))
                routes.append((drop_tile, drop_tile, DIR_S, DIR_S))
                rates.extend([r(half), r(half)])
                output_drops.append(drop_tile)
            elif _dst_splitter(arc) == (0, 1):
                # Feedback drop: defer route emission to step 6.
                feedback_drops.append((drop_tile, arc_idx))
            else:
                raise RuntimeError(
                    f"place_one_to_m_from_synth: L2[{l2_idx}].out{port} has "
                    f"unexpected dst {arc['dst']}"
                )

    # 6. Feedback routes — depends on whether B' is in play.
    if not needs_balancer:
        # Direct routing: each feedback arc → M.in1 = (M_X + 1, M_Y - 1).
        m_in1 = (M_X + 1, M_Y - 1)
        for drop_tile, arc_idx in feedback_drops:
            arc_rate = throughputs[arc_idx]
            routes.append((drop_tile, m_in1, DIR_S, DIR_S))
            rates.append(r(arc_rate))
    else:
        # Through B': partition feedback arcs across B's 2 input ports such
        # that each port's running total ≤ 0.5 (the lane cap). Assign drops
        # left-to-right into the left port (B'.in0) until adding the next
        # would overflow, then switch to the right port. This contiguous
        # split produces simpler routing geometry than a greedy
        # smallest-running-total approach: feedback drops are physically
        # contiguous on row 10 east-of-centre, so giving the leftmost
        # drops to in0 (the leftmost B' port) keeps each port's wrap
        # channel locally bundled.
        bprime_in0 = (M_X + 1, M_Y - 3)
        bprime_in1 = (M_X + 2, M_Y - 3)
        in0_total = 0.0
        for drop_tile, arc_idx in feedback_drops:
            arc_rate = throughputs[arc_idx]
            if in0_total + arc_rate <= 0.5 + 1e-9:
                target = bprime_in0
                in0_total += arc_rate
            else:
                target = bprime_in1
            routes.append((drop_tile, target, DIR_S, DIR_S))
            rates.append(r(arc_rate))
        # B' → M.in1 wiring.
        # B'.out0 lands at (M_X+1, M_Y-1) = M.in1: head-on south, 2 lane-routes.
        b_out_total = feedback_total / 2.0  # rate per B' output port.
        b_out0_tile = (M_X + 1, M_Y - 1)
        b_out_half = b_out_total / 2.0
        routes.append((b_out0_tile, b_out0_tile, DIR_S, DIR_S))
        routes.append((b_out0_tile, b_out0_tile, DIR_S, DIR_S))
        rates.extend([r(b_out_half), r(b_out_half)])
        # B'.out1 lands at (M_X+2, M_Y-1), heads W to sideload M.in1 east-side.
        b_out1_src = (M_X + 2, M_Y - 1)
        routes.append((b_out1_src, b_out0_tile, DIR_S, DIR_S))
        rates.append(r(b_out_total))

    # ----- Grid bounds -----
    # Width: outer L2 cols span M_X-3 to M_X+4 (anchor + second tile), plus
    # one east-edge column for the feedback wrap. M_X = 4 gives width 10
    # for the simple (no-balancer) case. With a balancer, the wrap also
    # needs a second east column to bring B' outputs around to M.in1 east
    # sideload — width 11. Tight bounds matter: extra space lets CP-SAT
    # find structurally-invalid alternates.
    width = M_X + (7 if needs_balancer else 6)
    height = output_drop_y + (2 if needs_balancer else 3)

    # ----- Solve -----
    belt_dirs = _route_belts(
        pos,
        routes,
        width,
        height,
        rates=rates,
        lane_cap=lane_cap,
    )
    if belt_dirs is None:
        raise RuntimeError(
            f"(1, {n_outputs}) belt routing UNSAT (scale={scale}, "
            f"needs_balancer={needs_balancer})"
        )

    # ----- Build entities -----
    entities: list[dict[str, Any]] = []
    for p in pos:
        entities.append(
            {
                "name": "splitter",
                "x": p[0],
                "y": p[1],
                "direction": _FACTORIO_DIR[_splitter_dir(p)],
            }
        )
    for (x, y), (d, kind) in belt_dirs.items():
        if kind == "belt":
            entities.append(
                {
                    "name": "transport-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                }
            )
        elif kind == "ug_in":
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "input",
                }
            )
        else:  # ug_out
            entities.append(
                {
                    "name": "underground-belt",
                    "x": x,
                    "y": y,
                    "direction": _FACTORIO_DIR[d],
                    "io_type": "output",
                }
            )

    # Sort output drops left-to-right so output_tiles is deterministic.
    output_drops.sort(key=lambda t: t[0])

    return {
        "n_inputs": 1,
        "n_outputs": n_outputs,
        "width": width,
        "height": height,
        "entities": entities,
        "input_tiles": [list(input_tile)],
        "output_tiles": [list(t) for t in output_drops],
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
        # `(1, 5)` stays on the hand-tuned placer for now — it works under
        # the generalised placer too, but flakes in full-suite runs (CP-SAT
        # finds OutputDegree-violating alternates with longer timeouts when
        # other tests share the system). The hand-tuned `(1, 5)` is more
        # reliable under load. Tracking this as a known limitation; a
        # follow-up should add structural constraints (e.g., explicit
        # forbid-direction at specific tiles) so the generalised placer can
        # subsume it.
        (1, 5): place_one_to_five,
        # Generalised placer for `(1, m)` 3-level trees that fit the
        # one-feedback-or-no-balancer regime.
        (1, 6): place_one_to_m_from_synth,
        (1, 7): place_one_to_m_from_synth,
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
