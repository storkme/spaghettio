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
    """`(1, 8)` 7-splitter tree, placed via real CP-SAT.

    Phase 2 of the placement RFP. The CP-SAT model has 7 splitter
    rectangles with `add_no_overlap_2d` plus structural equalities:
      - Tight-stack between level-1 and level-2: each level-2
        splitter is at `(level1.x ± 1, level1.y + 1)`.
      - Routing-row offsets between root and level-1: each level-1
        splitter is at `(root.x ± 2, root.y + 2)`. The 1-row gap
        accommodates the 4-belt routing strip W-S-E-S.

    With width 8, height 6, the constraint set has only one valid
    assignment (root at (3, 1)); the model proves this rather than
    asserting it. The same constraint pattern extends to (1, 16)
    with a 5-row gap and additional routing belts.
    """
    from ortools.sat.python import cp_model

    width, height = 8, 6
    n_splitters = 7
    # Splitter ordering: 0 = root, 1 = level-1-left, 2 = level-1-right,
    # 3-4 = level-2 under level-1-left, 5-6 = level-2 under level-1-right.
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

    # Boundary rows: input row above root, output row below level-2.
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

    entities: list[dict[str, Any]] = []
    # Input belt above root's right tile (matches phase 1 layout — feeds
    # root's port-1 input via south flow).
    entities.append(
        {"name": "transport-belt", "x": x_root + 1, "y": y_root - 1, "direction": 4}
    )
    # Splitters.
    for x, y in pos:
        entities.append({"name": "splitter", "x": x, "y": y, "direction": 4})
    # Routing row at y = y_root + 1. Direction codes: 0=N, 2=E, 4=S, 6=W.
    routing_y = y_root + 1
    entities.append(
        {"name": "transport-belt", "x": x_root, "y": routing_y, "direction": 6}
    )
    entities.append(
        {"name": "transport-belt", "x": x_root - 1, "y": routing_y, "direction": 4}
    )
    entities.append(
        {"name": "transport-belt", "x": x_root + 1, "y": routing_y, "direction": 2}
    )
    entities.append(
        {"name": "transport-belt", "x": x_root + 2, "y": routing_y, "direction": 4}
    )
    # Output row at y = level-2.y + 1, cols spanning all 4 level-2 splitters.
    output_y = pos[3][1] + 1
    leftmost = pos[3][0]
    rightmost = pos[6][0] + 1
    output_cols = list(range(leftmost, rightmost + 1))
    for col in output_cols:
        entities.append(
            {"name": "transport-belt", "x": col, "y": output_y, "direction": 4}
        )
    return {
        "n_inputs": 1,
        "n_outputs": 8,
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
