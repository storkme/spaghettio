#!/usr/bin/env python3
"""Phase 3.1 spike — CP-SAT splitter placement.

Reads a topology spec from stdin (JSON), encodes splitter positions and
no-overlap constraints with OR-Tools CP-SAT, writes the placement back
to stdout (JSON).

Belt routing between splitters is *not* solved here yet — that's phase
3.2 work. The spike confirms CP-SAT can place the 2D no-overlap
component within the time budget; once we trust that, we layer on the
edge-routing constraints.

Input schema (stdin):
  {
    "n_splitters": int,
    "bounds": [W, H]       # grid bounding box
  }

Output schema (stdout):
  {
    "status": "OPTIMAL" | "FEASIBLE" | "INFEASIBLE" | "UNKNOWN",
    "elapsed_s": float,
    "splitters": [{"x": int, "y": int}, ...]   # only on success
  }
"""

import json
import sys
import time

from ortools.sat.python import cp_model


def main():
    req = json.load(sys.stdin)
    n_splitters = req["n_splitters"]
    width, height = req["bounds"]

    model = cp_model.CpModel()

    # All splitters south-facing for the spike: they occupy 2 tiles
    # along the x-axis. Each splitter's anchor at (x, y), second tile at
    # (x+1, y). Bounds: x ∈ [0, W-2], y ∈ [0, H-1].
    xs = [model.NewIntVar(0, width - 2, f"x{i}") for i in range(n_splitters)]
    ys = [model.NewIntVar(0, height - 1, f"y{i}") for i in range(n_splitters)]

    # NoOverlap2D over fixed-size 2×1 rectangles.
    x_intervals = [
        model.NewFixedSizeIntervalVar(xs[i], 2, f"xi{i}")
        for i in range(n_splitters)
    ]
    y_intervals = [
        model.NewFixedSizeIntervalVar(ys[i], 1, f"yi{i}")
        for i in range(n_splitters)
    ]
    model.AddNoOverlap2D(x_intervals, y_intervals)

    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 30.0
    t0 = time.monotonic()
    status = solver.Solve(model)
    elapsed = time.monotonic() - t0

    status_name = solver.StatusName(status)
    out = {"status": status_name, "elapsed_s": elapsed}
    if status in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        out["splitters"] = [
            {"x": solver.Value(xs[i]), "y": solver.Value(ys[i])}
            for i in range(n_splitters)
        ]

    json.dump(out, sys.stdout)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
