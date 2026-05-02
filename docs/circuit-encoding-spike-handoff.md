# circuit-encoding spike — handoff

## RESULT (2026-05-02): SPIKE LANDED

**Root cause:** UG arcs were modelled `A → B` in the circuit graph, but
Factorio's UG-output cell holds an *entity* while the *flow* lands one cell
*beyond* it. `A → B` forces `B` to also have an out-arc (a belt) which
collides with the UG-output entity already at `B` in at-most-one. Same
shape applies to the dst's south-belt.

**Fix:** UG arcs now hop `A → C` where `C = A + (L+1)·step`; `B` is a
phantom entity counted in at-most-one but not in the circuit graph. The
dst's south-belt is a real arc to a virtual `EXIT_NODE`, with closing arc
`EXIT_NODE → src`. See commit and `solve_pure_routing_circuit` in
`crates/balancer-gen/scripts/place.py`.

**Numbers** (FUCKTORIO_DEBUG_4_9=1 + FUCKTORIO_PURE_ROUTING_ENCODING=circuit):

| jh | status | solver | wall |
|----|--------|--------|------|
| 5  | INFEASIBLE | 0.5s | 1.3s |
| 6  | INFEASIBLE | 0.7s | 1.6s |
| 7  | INFEASIBLE | 2.9s | 4.1s |
| 8  | INFEASIBLE | 80.1s | 81.6s |
| 9  | **OPTIMAL** | **118.8s** | **120.6s** |

Total compose+route: **209.1s** (vs original baseline 917s) — **4.4× wall
speedup**, classified `Balanced`. The minimum target was <300s; this hits
that comfortably.

Next: bake the remaining 7 shapes ((3, 9), (5, 9), (6, 9), (7, 9), (8, 9),
(9, 9)) and close issue #136. The `FUCKTORIO_PURE_ROUTING_ENCODING=circuit`
env var still gates the new path; consider promoting it to default once
the bake corpus passes.

---

## Original investigation (kept for reference)

Phase 4.4's `compose_series` calls `solve_pure_routing` (in
`crates/balancer-gen/scripts/place.py`) once per candidate `junction_height`.
The current generic encoding (per-(cell, edge, direction) bool vars +
explicit conservation + at-most-one + forced south-belt at dst) takes
~900s on the (4, 9) Clos compose, dominated by the actual feasible-solve
at jh=9 (~640s).

**Goal:** swap the per-edge routing model to CP-SAT's dedicated
`AddCircuit` constraint and see if the specialised propagation cuts solve
time meaningfully (folklore: 2–5× on TSP-shaped problems). The new
function `solve_pure_routing_circuit` lives next to `solve_pure_routing`
in the same file and is gated by `req["encoding"] == "circuit"` (set via
the env var `FUCKTORIO_PURE_ROUTING_ENCODING=circuit` in
`compose_series`).

**Status:** the encoding is wired end-to-end and runs, but produces
**INFEASIBLE during probing** on any multi-edge layout where paths must
cross. Single-edge cases solve correctly. The bug is in the *combination*
of constraints; each constraint works individually.

## What works

- **Per-edge `AddCircuit` alone** (no cross-edge constraints):
  multi-edge cases all return OPTIMAL fast. Output has multiple entities
  per cell because nothing forbids it; pure-routing is broken
  semantically but the circuit itself is fine.
- **Single-edge with full constraints** (at-most-one + forced_dst_south
  + UG-arrival + UG-pairing): solves in 10–20ms with a valid path.
- **Multi-edge identity perm** (each edge `i → i`, no crossings): solves
  with full constraints. Output is correct.
- **Multi-edge with at-most-one but no `forced_dst_south`**: solves but
  layouts are physically invalid (UG outputs land on cells that should
  hold the junction-exit south-belt — the model has no way to know).
- **Multi-edge with at-most-one + forced_dst_south + at-most-one terms
  but excluding UG-arrival**: solves. *But* this is also unsafe — without
  UG-arrival in at-most-one, the model allows two edges' UG endpoints to
  collide.

## What's broken

The 2-edge swap `(0, 0)→(3, 5)` and `(3, 0)→(0, 5)` on a 4×6 grid (and
larger) returns INFEASIBLE during CP-SAT probing in 10–60ms with the
full constraint set. Same applies to the (2, 2) Clos compose at every
`jh ∈ [3, 8]` we tested.

Crucially, the original (non-circuit) `solve_pure_routing` *does* find a
feasible layout for the (2, 2) Clos at `jh=6` and (4, 9) at `jh=9`, so
the problems are genuinely solvable.

CP-SAT log shows: `INFEASIBLE: 'during probing'`. Probing fixes ~242 of
358 booleans by implication chains and then derives a contradiction.

## Repro

Three layered tests, all run from the repo root.

**Repro 1 — minimal failing case (2-edge swap):**

```bash
echo '{
  "kind": "pure_routing", "encoding": "circuit",
  "bounds": [4, 6],
  "input_port_tiles": [[0, 0], [3, 0]],
  "output_port_tiles": [[0, 5], [3, 5]],
  "edges": [
    {"src_kind": "InputPort", "src_idx": 0, "dst_kind": "OutputPort", "dst_idx": 1},
    {"src_kind": "InputPort", "src_idx": 1, "dst_kind": "OutputPort", "dst_idx": 0}
  ],
  "max_time_s": 10
}' | uv run --no-project --script crates/balancer-gen/scripts/place.py
```

Expected: `INFEASIBLE` in <100ms. The same request with
`"debug_no_forced_in_atmost": true` returns `OPTIMAL`.

**Repro 2 — full (2, 2) Clos compose:**

```bash
FUCKTORIO_DEBUG_2_2=1 FUCKTORIO_PURE_ROUTING_ENCODING=circuit \
  cargo run --release -p balancer-gen
```

Expected: every `jh ∈ [3, 8]` returns INFEASIBLE in 0.0s solver time
(CP-SAT proves it during probing). Without the env var, same test
returns `Balanced` at jh=6.

**Repro 3 — full (4, 9) Clos compose** (slow; only run if you need it):

```bash
FUCKTORIO_DEBUG_4_9=1 FUCKTORIO_PURE_ROUTING_ENCODING=circuit \
  cargo run --release -p balancer-gen
```

## Constraint isolation matrix

The minimal repro JSON has these debug toggles. All default off. Each
toggles **off** the named piece of the model.

| Flag | Effect |
|------|--------|
| `debug_no_atmost_one` | Skip the per-cell at-most-one entity constraint. |
| `debug_no_ug_arrival` | Don't include UG-output landings in at-most-one. |
| `debug_no_ug_pairing` | Skip the Factorio UG re-pairing rule. |
| `debug_no_forced_in_atmost` | Keep `forced_dst_south = 1` but don't add it to at-most-one terms. |
| `debug_no_forced_dst` | Pin `forced_dst_south = 0` (no junction-exit belt). |
| `debug_no_ugs` | Don't generate UG arcs at all (belts only). |
| `debug_log_progress` | Enable `solver.parameters.log_search_progress = True` and pipe to stderr. |

What the matrix tells us:

| Constraints | Result |
|---|---|
| circuit only | OPTIMAL (garbage output) |
| + at-most-one | OPTIMAL |
| + forced_dst_south (not in at-most-one) | OPTIMAL |
| + forced_dst_south *in* at-most-one | **INFEASIBLE** ← the failure |
| + UG-arrival not in at-most-one | OPTIMAL (allows UG collisions, unsafe) |
| + drop UGs entirely | INFEASIBLE (correct: belt-only swap is impossible) |

**The smallest configuration that fails:** `at-most-one` + `forced_dst_south
in at-most-one` on any multi-edge swap. UG-arrival being in at-most-one
*is* required for correctness (without it, UG endpoints collide), but
it's part of what makes the model presolve-infeasible.

## Hypotheses for the bug (untested)

1. **The "forced south-belt at dst" should be a real arc, not a phantom
   entity.** The original `solve_pure_routing` models the south-belt as
   `arcs[(dx, dy, south=2, e)] == 1` with the off-grid south arc allowed
   at `y = jh-1`. The circuit version models it as a separate
   `forced_dst_south` bool that's pinned to 1 and counted in
   at-most-one, but has no corresponding circuit arc. The dst's outgoing
   arc in the circuit is the closing arc (`dst → src`, virtual).
   Maybe CP-SAT's circuit propagator *plus* the pin-to-1 bool *plus*
   at-most-one creates a chain it can't satisfy. The fix would be: add
   a virtual "exit" node south of each dst, model the south-belt as
   `dst → exit` (real arc, real bool), and make the closing arc go from
   exit back to src. The exit is then dst's outgoing in the circuit,
   and the south-belt is a regular arc that participates in at-most-one
   naturally.

2. **Closing arc as forced=1 may be over-constraining.** Try
   `closing_lit` left as a free bool (no `model.Add(closing_lit == 1)`).
   The circuit constraint should still force it via the
   "every visited node has one outgoing" rule. Worth testing.

3. **UG transit cells need explicit at-most-one participation.** Right
   now UG transit cells aren't in the at-most-one count (correct
   per-Factorio: another belt CAN sit on top of a UG transit). But CP-SAT
   may need a different way to express this so probing doesn't derive
   bad implications. Long shot.

4. **Disable presolve probing as a workaround test.**
   `solver.parameters.cp_model_probing_level = 0` would skip the probing
   step. If the model becomes feasible without probing but produces a
   correct layout when allowed to search, the bug is specifically in the
   probing inference chain — and the fix is to make the constraints
   strong enough to *survive* probing without false contradictions.

## What I'd try first

In order of effort:

1. **Add `cp_model_probing_level = 0` to the spike to confirm probing is
   the inference layer hitting the contradiction.** 1-line change. If
   the model then solves the swap, the bug is in probing's reasoning
   over our constraint set, not in correctness.
2. **Reformulate "forced south-belt at dst" as a real arc with a virtual
   exit node** (hypothesis #1). This is the most likely correct shape
   and aligns with how the original `solve_pure_routing` models the
   junction exit. Estimate: 1 hour to rewrite + test.
3. **Drop the closing-arc forcing and let the circuit constraint imply
   it.** Quick test (hypothesis #2).

## Files touched

- `crates/balancer-gen/scripts/place.py` — `solve_pure_routing_circuit`
  is the new function (lines 1601–1880ish, near the bottom). The
  dispatcher in `main()` checks `req.get("encoding") == "circuit"`.
- `crates/balancer-gen/src/main.rs` — `compose_series` reads
  `FUCKTORIO_PURE_ROUTING_ENCODING` and sets `req.encoding` accordingly.
  `PlaceRequest::PureRouting` got an optional `encoding` field.

The original `solve_pure_routing` is **untouched** — the spike is purely
additive.

## Baseline numbers to beat

- (4, 9) Clos compose, original `solve_pure_routing`: **917s wall**, jh=9
  (with heuristic initial jh + stratified timeout). Solve time at jh=9
  is 642s of that.
- (4, 9) Clos compose, original baseline (jh=1 search, single timeout):
  1173s.

If the circuit encoding gets (4, 9) under ~300s wall, the spike
delivers. Anything around 900s is no win and we should accept the
current model and move on.

## Next-step menu when this gets parked

If the circuit spike lands and is fast: bake the remaining 7 shapes
((3, 9), (5, 9), (6, 9), (7, 9), (8, 9), (9, 9)) and close issue #136.

If it doesn't land or solve time matches the baseline:

- **Spatial pruning per edge** in the existing `solve_pure_routing`:
  for each edge, only allocate arc vars within a Manhattan-distance
  bound of src and dst. Cuts var count 30–50% on wide junctions.
  Predictable ~30 minutes of work.
- **Minimal parallelism**: add `FUCKTORIO_BAKE_ONLY=m,n` to balancer-gen
  so a shell wrapper can run multiple shape bakes concurrently. Wall
  time per overnight ≈ max(individual) instead of sum.
- **Accept current performance** and bake the remaining shapes
  sequentially (~3-4 hours wall time). Ship phase 3.4 as-is.
