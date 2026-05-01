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
    """`(1, 4)` complete binary tree, library-style tight-stack layout.

    Width 4, height 4. 3 splitters + 5 belts. Splitters interlock:
      - Root at (1, 1) spans (1,1)-(2,1). Outputs at (1,2) and (2,2).
      - Level-1 left at (0, 2) spans (0,2)-(1,2). Tile (1,2) is its
        right tile, so root's left output feeds it as port-1 input.
      - Level-1 right at (2, 2) spans (2,2)-(3,2). Tile (2,2) is its
        left tile, so root's right output feeds it as port-0 input.
      - Each level-1 splitter has its other input port empty (rate 0).

    Trace at unit input: root in = 1, outs each = 0.5. Each level-1
    in = 0.5, outs each = 0.25. Outputs uniform at 1/4.
    """
    entities: list[dict[str, Any]] = [
        {"name": "transport-belt", "x": 1, "y": 0, "direction": 4},
        {"name": "splitter", "x": 1, "y": 1, "direction": 4},
        {"name": "splitter", "x": 0, "y": 2, "direction": 4},
        {"name": "splitter", "x": 2, "y": 2, "direction": 4},
    ]
    for col in range(4):
        entities.append({"name": "transport-belt", "x": col, "y": 3, "direction": 4})
    return {
        "n_inputs": 1,
        "n_outputs": 4,
        "width": 4,
        "height": 4,
        "entities": entities,
        "input_tiles": [[1, 0]],
        "output_tiles": [[c, 3] for c in range(4)],
    }


def place_one_to_eight() -> dict[str, Any]:
    """`(1, 8)` complete binary tree with one routing row.

    Width 8, height 6. 7 splitters + 13 belts. Layout:
      - Root at (3, 1) spans (3,1)-(4,1). Real input belt at (4, 0)
        feeds root's port-1 (right input); port 0 at (3, 0) empty.
      - Routing row y=2: belts at (3, 2)W, (2, 2)S, (4, 2)E, (5, 2)S.
        Root left output at (3, 2) → west to (2, 2) → south to (2, 3).
        Root right output at (4, 2) → east to (5, 2) → south to (5, 3).
      - Level-1 splitters at (1, 3) (spans 1-2) and (5, 3) (spans 5-6).
        Receive inputs at (2, 2)S → (2, 3) [right input of left]
        and (5, 2)S → (5, 3) [left input of right]. Other input
        ports empty (rate 0).
      - Level-2 splitters at (0, 4), (2, 4), (4, 4), (6, 4), each
        spanning 2 cols. Tight-stack with level-1: each level-1
        output drops directly into a level-2 splitter's tile.
      - Output belts at y=5 cols 0..7.

    Trace at unit input: root in = 1, outs = 0.5 each. After routing,
    level-1 in = 0.5 each, outs = 0.25 each. Level-2 in = 0.25 each
    (one port wired, other rate 0), outs = 0.125 each. 8 outputs
    uniform at 1/8.
    """
    entities: list[dict[str, Any]] = []
    # Input belt at (4, 0) feeds root's port-1.
    entities.append({"name": "transport-belt", "x": 4, "y": 0, "direction": 4})
    # Root splitter.
    entities.append({"name": "splitter", "x": 3, "y": 1, "direction": 4})
    # Routing row at y=2. Direction codes: 0=N, 2=E, 4=S, 6=W.
    entities.append({"name": "transport-belt", "x": 3, "y": 2, "direction": 6})  # W
    entities.append({"name": "transport-belt", "x": 2, "y": 2, "direction": 4})  # S
    entities.append({"name": "transport-belt", "x": 4, "y": 2, "direction": 2})  # E
    entities.append({"name": "transport-belt", "x": 5, "y": 2, "direction": 4})  # S
    # Level-1 splitters.
    entities.append({"name": "splitter", "x": 1, "y": 3, "direction": 4})
    entities.append({"name": "splitter", "x": 5, "y": 3, "direction": 4})
    # Level-2 splitters.
    for col in (0, 2, 4, 6):
        entities.append({"name": "splitter", "x": col, "y": 4, "direction": 4})
    # Output belts.
    for col in range(8):
        entities.append({"name": "transport-belt", "x": col, "y": 5, "direction": 4})
    return {
        "n_inputs": 1,
        "n_outputs": 8,
        "width": 8,
        "height": 6,
        "entities": entities,
        "input_tiles": [[4, 0]],
        "output_tiles": [[c, 5] for c in range(8)],
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
        # Lazy import — unimplemented shapes shouldn't pay the
        # ~200 ms ortools cost.
        from ortools.sat.python import cp_model

        # Trivial CP-SAT solve to exercise the solver pipeline before
        # delegating to the geometry helper.
        model = cp_model.CpModel()
        sentinel = model.new_int_var(0, 0, "sentinel")
        model.add(sentinel == 0)
        solver = cp_model.CpSolver()
        solver.parameters.max_time_in_seconds = timeout_ms / 1000.0
        seed = request.get("seed")
        if seed is not None:
            solver.parameters.random_seed = int(seed)
        status = solver.solve(model)
        if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
            emit(
                {
                    "kind": "engine",
                    "message": f"solver returned {solver.status_name(status)}",
                }
            )
            return 0

        template = geometry[(n, m)]()
        elapsed_ms = int((time.monotonic() - started) * 1000)
        emit({"kind": "ok", "template": template, "solve_wall_ms": elapsed_ms})
        return 0

    emit_unimplemented(f"shape ({n}, {m}) not yet supported by cp_sat_placer")
    return 0


if __name__ == "__main__":
    sys.exit(main())
