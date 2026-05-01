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
    """The trivial passthrough: a single south-facing belt at (0, 0).

    Width 1, height 1. Input at (0, 0), output also at (0, 0) — the
    same tile is both. The bus engine reads `input_tiles` to find where
    flow enters and `output_tiles` to find where it exits. For a
    single-belt template they're the same tile.
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

    if n == 1 and m == 1:
        # Trivial CP-SAT model: place 1 belt with no constraints.
        # We import lazily so that unimplemented shapes don't pay the
        # ortools import cost (~200 ms).
        from ortools.sat.python import cp_model

        model = cp_model.CpModel()
        x = model.new_int_var(0, 0, "x")
        y = model.new_int_var(0, 0, "y")
        model.add(x == 0)
        model.add(y == 0)
        solver = cp_model.CpSolver()
        solver.parameters.max_time_in_seconds = timeout_ms / 1000.0
        seed = request.get("seed")
        if seed is not None:
            solver.parameters.random_seed = int(seed)
        status = solver.solve(model)
        if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
            emit({"kind": "engine", "message": f"solver returned {solver.status_name(status)}"})
            return 0

        template = place_one_to_one()
        elapsed_ms = int((time.monotonic() - started) * 1000)
        emit({"kind": "ok", "template": template, "solve_wall_ms": elapsed_ms})
        return 0

    emit_unimplemented(f"shape ({n}, {m}) not yet supported by cp_sat_placer")
    return 0


if __name__ == "__main__":
    sys.exit(main())
