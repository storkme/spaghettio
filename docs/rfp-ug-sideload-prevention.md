# RFP: UG-input sideload prevention in compose_series

## Problem

The compose pipeline (`balancer-gen` `compose_series` + CP-SAT
`solve_pure_routing`) produces inter-stage junctions that
topologically balance but contain belt-into-UG-input sideloads. In
real Factorio these halve UG throughput (only one lane loaded), so
the resulting balancer can't deliver full N/M throughput.

Library audit at the time of this RFP: **0 errors, 38 warnings across
19 shapes**. 10 of the 19 are compose-generated; the rest are
python-derived (`scripts/generate_balancer_library.py`'s 180-degree
rotation of solved (n, m) → (m, n)).

The validator catches the warnings via
`check_underground_belt_entry_sideload` in
`crates/core/src/validate/belt_flow.rs`. Until #288 the bake gate
ignored them.

## What's been done (PR #288, phase 1)

Two coordinated changes:

1. **Tighten `bake_missing_shapes`'s lane gate** to reject layouts
   with `Severity::Warning` underground-belt issues alongside Errors.
   Previously only Errors gated; warnings shipped silently.

2. **Same-edge UG-input sideload constraint** in `solve_pure_routing`
   (`crates/balancer-gen/scripts/place.py`). For each potential
   UG-input at `(cx, cy)` facing direction `d_ug`, forbid same-edge
   belt arcs at non-straight neighbour cells:

   ```python
   for (cx, cy, d_ug, L_ug, e_idx), ug_var in ug_arcs.items():
       for d_in in range(4):
           if d_in == d_ug:
               continue  # straight feed allowed
           n_dx, n_dy = DIR_STEPS[d_in]
           ncx, ncy = cx - n_dx, cy - n_dy
           arc_var = arcs.get((ncx, ncy, d_in, e_idx))
           if arc_var is not None:
               model.Add(arc_var + ug_var <= 1)
   ```

### Empirical result on (1, 10)

| jh | UG issues without constraint | with constraint |
|---:|------------------------------:|----------------:|
| 4  | 3 | 2 |
| 5  | 3 | 3 |
| 6  | 8 | 2 |

The constraint helps but doesn't eliminate the issues. Notably jh=5
shows no improvement.

## What we learned investigating the residuals

### The compose-junction src-cell sideload mode

The remaining warnings on (1, 10) trace to UG-inputs placed at the
**top row of the junction** (y=0 in junction-local coords) facing a
direction other than south.

Concrete case from the (1, 10) jh=4 attempt:

- Composed layout: stage1 = (1, 5) atom, height 8 (rows 0..7);
  junction starts at row 8 with jh=4 (rows 8..11); stage2 below.
- Validator warning: `Belt at (3, 7) facing South sideloads into
  underground input at (3, 8) facing East`.
- Translating to junction-local: belt at junction `y = -1` (i.e.
  stage1's bottom row), UG-input at junction `(3, 0)` — the **src
  row** of the junction.
- Stage1's bottom row at `(3, 7)` is a south-facing belt (the
  splitter output of (1, 5)'s atom). It physically feeds into the
  junction's src cell `(3, 0)` from above.
- Junction's CP-SAT solver placed an **east-facing UG-input** at the
  src cell. From the model's perspective this is "fine" — `is_src`
  adds 1 to the cell's virtual inflow and the UG-arc supplies 1 of
  outflow, so flow conservation is satisfied. But the model doesn't
  know stage1's south-facing belt is physically above.

The phase-1 constraint can't catch this because the **feeding belt
is in stage1, not in the junction's `arcs` dict**. The constraint
operates on same-edge arcs *within* the junction.

### Why jh=5 didn't improve

Same root cause — at jh=5 the residuals are also at junction src
cells (different shapes, same pattern). The phase-1 constraint
doesn't apply to those layouts at all because there's no within-
junction same-edge belt feeding the UG.

### Cross-edge sideloads are NOT a separate bug

I initially worried that residual sideloads might come from a
different edge's belt feeding a UG-input. Walking through flow
conservation: if edge e2 has a belt at (cx, cy) flowing into (cx',
cy') which hosts edge e1's UG-input, then e2's flow at (cx', cy')
has inflow=1 (from the belt) and outflow=0 (no e2 entity at the
UG-input cell — at-most-one-entity-per-cell prevents it), so
conservation fails. The model rejects this. So cross-edge sideloads
within the junction *are* excluded by the existing model.

## Proposed fix (phase 2)

### Phase 2A: Direction constraint at junction src cells

At each src cell, the only valid entity is one whose forward
direction matches the incoming flow direction (south, given stage1's
output emits south into the junction):

```python
# At src cells, only south-direction belts and south-facing
# UG-inputs allowed. Stage1 above feeds south-flow into these
# cells; any other direction would be a sideload from above.
for e_idx, src in enumerate(edge_src):
    sx, sy = src
    for d in range(4):
        if d == 2:  # south = the stage1 → junction feed direction
            continue
        if (sx, sy, d, e_idx) in arcs:
            model.Add(arcs[(sx, sy, d, e_idx)] == 0)
        for L in range(2, ug_max_reach_param + 1):
            if (sx, sy, d, L, e_idx) in ug_arcs:
                model.Add(ug_arcs[(sx, sy, d, L, e_idx)] == 0)
```

Expected effect: the junction's src row will host only south-belts
or south-facing UG-inputs. Stage1's south flow continues straight
into either, no sideload.

### Phase 2B: UG-output → belt head-on collisions

The bake also surfaces errors of the form:

> Underground belt exit at (X, Y) facing W collides head-on with
> belt at (Z, Y) facing E

These are different from sideloads — a UG-output and a belt facing
each other. The model places a UG-output that would pop into a
cell where another edge's belt flows toward it, producing physical
collision.

This needs its own constraint, distinct from the sideload one.

### Phase 2C: Apply both constraints to `solve_synth_place` (Mode D)

Mode D synthesizes splitter+routing in one go and produces the
single-stage shapes (no compose junction). It has its own UG-input
generation that doesn't share `solve_pure_routing`'s constraint
plumbing. Worth verifying whether Mode D produces sideloads in the
existing library entries; if so, the same constraint logic applies.

## Open questions

1. **Are the python-derived shapes also fixable through bakes?** The
   9 python-derived shapes with warnings (`(3, 2)`, `(5, 1)`,
   `(5, 3)`, `(6, 1)`, `(6, 2)`, `(7, 3)`, `(8, 1)`, `(8, 2)`,
   `(8, 6)`) don't have compose recipes yet. Each needs a recipe
   (composition from clean atoms) similar to (7, 2)'s
   `Lib(7, 1) → Lib(1, 2)`. The fanout/merger pattern works for
   `(n, 1)` and `(1, m)` shapes; mid-shapes need more thought.

2. **Does enforcing south-only at src cells over-constrain
   feasibility?** Suspect not — stage1's south flow always exits
   south, so south-belt or south-UG-input at src is always feasible.
   But worth empirical confirmation.

3. **Does the validator catch ALL UG-throughput issues, or just
   sideloads?** `check_underground_belt_entry_sideload` only flags
   perpendicular sideloads (`dot == 0`). Head-on (`dot == -1`)
   isn't caught here — it's caught by a different validator (UG
   exit collision). Worth understanding the full taxonomy of UG
   throughput pitfalls before claiming "library is UG-correct".

## Where to stop and pick up

PR #288 is **phase 1** — gate tightening + same-edge constraint.
Standalone, it doesn't reduce the audit's 38 warnings (no library
entries change). It's foundation for phase 2.

Phase 2A (src-cell direction constraint) is the next concrete step.
~10-15 LOC. Should fully clear the residual sideloads on (1, 10)
and similar compose-generated shapes. Run `FUCKTORIO_REBAKE_SHAPES`
on each compose-generated shape, replace library entries that bake
clean.

Phase 2B (UG-output collision) and 2C (Mode D) are independent
follow-ups.

## Decision log

- *2026-05-03 — drafted. PR #288 (phase 1) shipped. Investigation
  confirmed the residual sideloads are stage1→junction-src-cell
  sideloads, not within-junction sideloads. Phase 2A is the natural
  next step but blocked on a fresh session for the implementation.*
