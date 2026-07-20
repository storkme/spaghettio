# RFC: UG-input sideload prevention in compose_series

## Problem

The compose pipeline (`balancer-gen` `compose_series` + CP-SAT
`solve_pure_routing`) produces inter-stage junctions that
topologically balance but contain belt-into-UG-input sideloads. In
real Factorio these halve UG throughput (only one lane loaded), so
the resulting balancer can't deliver full N/M throughput.

Library audit at the time of this RFC: **0 errors, 38 warnings across
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

### Phase 2C: Port all four constraints to `solve_synth_place` (Mode D)

Mode D synthesises splitter+routing in one go and produces the
single-stage shapes (no compose junction). It has its own UG
generation that doesn't share `solve_pure_routing`'s constraint
plumbing. All four physical principles still apply; the translation
requires adapting for Mode D's structural differences.

#### Key structural differences between Mode D and `solve_pure_routing`

| Aspect | `solve_pure_routing` | `solve_synth_place` (Mode D) |
|--------|---------------------|------------------------------|
| UG directions | All 4 directions | South only (`d = facing = 2`) |
| UG L range | `range(2, ug_max_reach_param+1)` | `range(1, UG_MAX_REACH)` (i.e. `range(1, 5)`) |
| Splitter cells | Static set `splitter_cells` | Reified bool `is_splitter_cell[(cx, cy)]` |
| Src cells | `edge_src[e_idx]` — fixed tuples | InputPort edges: `cy==0`, column tracked by `input_at[(i, cx)]` |
| Dst cells | `edge_dst[e_idx]` — fixed tuples | OutputPort edges: `cy==height-1`, column tracked by `output_at[(j, cx)]` |
| `arcs` keys | `(cx, cy, d, e_idx)` | `(cx, cy, d, e_idx)` — same structure |
| `ug_arcs` keys | `(cx, cy, d, L, e_idx)` | `(cx, cy, d, L, e_idx)` — same structure |

Because all UGs in Mode D face south, the constraint loops that
iterate over `d_ug` in `solve_pure_routing` collapse to a single
iteration (`d_ug = 2`). The guard `if d_ug == d_in: continue`
becomes `if d_in == 2: continue` (skip only south, the straight
direction). This simplification also means constraint 3 (chained
UG) vanishes entirely — two UG-inputs of the same edge that are
chained must both face south, which is already guaranteed by the
all-south restriction.

The `is_splitter_cell` reification is the other key difference.
Splitter cells are not free routing cells, so the UG-sideload
constraints don't need to fire at them (a splitter tile can't host
a regular belt arc anyway). The existing splitter-cell direction
constraints already prevent non-south arcs at those positions. For
safety the sketches below follow the same pattern as the existing
at-most-one entity constraint: apply constraints regardless, since
the splitter constraints already force those cells' arc vars to
zero. The sketches note where explicit splitter cell guards are
worth adding for clarity.

#### Constraint 1 (Mode D port): same-edge belt into UG-input

**Principle**: When a UG-input is active at `(cx, cy)` facing some
direction `d_ug`, only a belt from the straight-behind cell (flowing
in `d_ug`) may feed it. Since Mode D restricts UGs to south, the
"straight-behind" cell is always `(cx, cy-1)` (north of the
UG-input), and the feeding arc must face south.

**Code sketch** (insert after the UG-pairing rule, before flow
conservation):

```python
# Constraint 1 (Mode D): same-edge belt into UG-input sideload.
# All UGs face south (d=facing=2), so only a south-facing arc from
# the cell directly above is a straight feed. Belts entering from
# any other direction would sideload the UG (only one lane loaded).
# Splitter cells are safe — the splitter direction constraints
# already zero out non-south arc vars there.
for (cx, cy, d_ug, L, e_idx), ug_var in ug_arcs.items():
    # d_ug is always facing (2=south); iterate non-straight neighbours.
    for d_in in range(4):
        if d_in == d_ug:
            continue  # straight feed is allowed
        n_dx, n_dy = DIR_STEPS[d_in]
        ncx, ncy = cx - n_dx, cy - n_dy
        if not (0 <= ncx < width and 0 <= ncy < height):
            continue
        arc_var = arcs.get((ncx, ncy, d_in, e_idx))
        if arc_var is not None:
            model.Add(arc_var + ug_var <= 1)
```

**LOC**: ~12 (identical structure to `solve_pure_routing`; d_ug
loop collapses to one iteration but code shape is the same).

**Open questions**:
- Mode D's UG arcs start at L=1 (vs L=2 in `solve_pure_routing`).
  The sideload risk is identical at L=1 — any active UG-input,
  regardless of span length, should not receive a perpendicular belt.
  The sketch handles this correctly.
- Splitter tiles: the existing splitter constraint zeros all
  non-south arcs anyway, so the `model.Add(arc_var + ug_var <= 1)`
  at a splitter neighbour cell is redundant but not harmful.

---

#### Constraint 2 (Mode D port): InputPort src-cell direction

**Principle**: In `solve_pure_routing`, junction src cells (y=0)
receive south-flowing belts from stage1 above; the constraint forces
only south-direction entities there. In Mode D there is no compose
junction, but InputPort edges are sourced at y=0 — the input row.
Factorio's balancer template classifier reads belts off the input
row as south-facing (items arrive from above, flowing south into the
shape). Any non-south entity at an InputPort source cell would
sideload items arriving from the north.

The complication: in Mode D the x-coordinate of each InputPort
source is a CP-SAT variable (`ix[i]`), not a fixed tuple. The
constraint must be gated on whichever column the port lands in,
using `input_at[(i, cx)]` as the enforcing literal.

**Code sketch**:

```python
# Constraint 2 (Mode D): InputPort src-cell direction.
# InputPort sources are at y=0, column determined by ix[i].
# Items arrive from the north facing south; only south-facing arcs
# and south-facing UG-inputs are valid at the chosen column.
# (south UGs are the only kind in Mode D — this effectively allows
# any UG arc originating at y=0, and forbids all non-south belts.)
for e_idx, edge in enumerate(edges):
    if edge["src_kind"] != "InputPort":
        continue
    i = edge["src_idx"]
    for cx in range(width):
        in_v = input_at[(i, cx)]  # 1 iff this input port is at column cx
        # Forbid non-south belt arcs at (cx, 0) for this edge when port is here.
        for d in range(4):
            if d == facing:
                continue
            arc_var = arcs.get((cx, 0, d, e_idx))
            if arc_var is not None:
                model.Add(arc_var + in_v <= 1)
        # UG-inputs at (cx, 0): all face south (d=facing) — already fine.
        # No non-south UG vars exist in Mode D, so no UG loop needed.
```

**LOC**: ~15. Belt-arc loop only; UG loop is a no-op because Mode D
only creates south UG arcs.

**Open questions**:
- `in_v` is a reified bool from `model.Add(ix[i] == cx)
  .OnlyEnforceIf(in_v)`. Using `model.Add(arc_var + in_v <= 1)` is
  the correct half-reification pattern (matches the existing
  splitter-cell direction constraints at lines 776-793 of the
  function).
- Mode D does not have a concept of "south arrives from stage1
  above"; instead, InputPort tiles are defined as the entry points
  by topology. If a shape ever legitimately has a non-south input
  (e.g., a shape with horizontal inputs), this constraint would
  over-restrict it. For the all-south Mode D (`solve_synth_place`)
  this is not a concern; for `solve_synth_place_dirs` (direction
  freedom), the analog must be reified per `dir_at` as well — see
  §`solve_synth_place_dirs` note below.

---

#### Constraint 3 (Mode D port): chained UG sideload

**Principle**: when one UG-input's output lands at a cell that hosts
another UG-input facing a different direction, the second UG is
sideloaded by the first UG's exiting flow. Both must face the same
direction.

**Mode D status**: this constraint **has no meaningful content in
Mode D**. All UG arcs are restricted to `d = facing = 2` (south).
Two chained UGs in Mode D are necessarily both south-facing, so the
same-direction requirement is automatically satisfied. No code is
needed.

Document this explicitly to show the reasoning, not as a gap:

> Constraint 3 is vacuously satisfied in `solve_synth_place`: UG
> arcs are restricted to `d = facing = 2` by construction, so a
> chained pair can never face different directions. No additional
> constraint is needed.

In `solve_synth_place_dirs` (direction freedom), `d` is no longer
fixed. Constraint 3 applies there and must be ported using the same
loop structure as in `solve_pure_routing`, with the addition that
`d_ug` and `d_up` must be looked up via the generated `ug_arcs`
key iteration (which covers all four directions in that mode).

---

#### Constraint 4 (Mode D port): UG-output head-on collision

**Principle**: when a UG-output exits at `(ox, oy)` and the cell
one step ahead `(nx, ny)` has a belt for the same edge flowing in
the opposite direction, the two collide head-on (validator error:
"Underground belt exit at X facing W collides head-on with belt at
Z facing E"). Forbid the opposite-direction arc at `(nx, ny)`.

Since Mode D's UGs are all south, the UG-output is always at
`(cx + L*0, cy + L*1) = (cx, cy+L)` and the cell one ahead is
`(cx, cy+L+1)`. The opposite direction is north (`d=0`). The
constraint reduces to: if UG at `(cx, cy, south, L, e)` is active,
forbid a north-facing arc at `(cx, cy+L+1, e)`.

**Code sketch**:

```python
# Constraint 4 (Mode D): UG-output head-on collision prevention.
# All UGs face south; UG-output is at (cx, cy+L); cell ahead is
# (cx, cy+L+1). A north-facing arc there would flow back into the
# UG-output, producing a head-on collision.
OPPOSITE = {0: 2, 1: 3, 2: 0, 3: 1}
for (cx, cy, d_ug, L, e_idx), ug_var in ug_arcs.items():
    dx, dy = DIR_STEPS[d_ug]  # (0, 1) for south
    ox, oy = cx + L * dx, cy + L * dy     # UG-output cell
    nx, ny = ox + dx, oy + dy             # cell ahead of output
    if not (0 <= nx < width and 0 <= ny < height):
        continue
    d_opposite = OPPOSITE[d_ug]           # north = 0
    opp_arc = arcs.get((nx, ny, d_opposite, e_idx))
    if opp_arc is not None:
        model.Add(opp_arc + ug_var <= 1)
```

**LOC**: ~14 (identical structure to `solve_pure_routing`; no
structural simplification because the d_ug loop was already a
no-op for a single direction).

**Open questions**:
- In Mode D the cell `(cx, cy+L+1)` may be the output row
  (`cy+L+1 == height-1`). OutputPort destination cells have arcs
  forced south by the flow conservation. A north arc at an output
  cell is already excluded by the forced-south arc at that cell —
  so this constraint is vacuously satisfied at the output row. It
  can be added unconditionally (harmless) or guarded with
  `if ny < height - 1: ...` for clarity.
- `arcs.get(...)` returns `None` for keys that don't exist (e.g.,
  if the cell is out of grid or the arc was zeroed). The `if
  opp_arc is not None` guard handles this correctly.

---

#### Constraint placement in `solve_synth_place`

Insert all four blocks immediately after the UG-pairing rule (the
`for (c1x, c1y, d1, L1, e1), arc1 in ug_arcs.items()` block) and
before the flow-conservation loop. This matches the layout in
`solve_pure_routing` and keeps the UG-correctness constraints
co-located.

For `solve_synth_place_dirs`: constraints 1, 2, and 4 all need
additional reification because `d_ug` is no longer fixed. Constraint
2 must be reified per `dir_at[(s, d)]` for the InputPort source cell
(the analog of the constraint being dependent on which port column is
chosen). Constraint 3 becomes live again (two chained UGs may face
different directions). A separate Phase 2C.2 RFC note should cover
the `_dirs` variant once the all-south path is validated.

#### Verification

Verification depends on the comparison harness work happening
separately (a mode that bakes a sample of Mode D shapes and runs the
library validator on the results, reporting before/after UG warning
counts). Once that harness exists:

1. Run the harness on the existing Mode D shapes without the new
   constraints — record the UG warning baseline.
2. Add the four constraints, re-run the harness.
3. Expected outcome: zero new UG-sideload or head-on warnings from
   Mode D shapes.
4. Confirm no feasibility regressions — if any previously-solvable
   shape becomes INFEASIBLE, inspect whether the constraint is
   over-tight or the original solution was genuinely invalid.
5. Rebuild and run `cargo test --manifest-path crates/core/Cargo.toml
   --lib` to confirm the Rust library still builds clean (the
   Python-side change has no Rust surface, but the baked output
   feeds `balancer_library.rs`).

#### Effort estimate

| Item | LOC | Effort |
|------|-----|--------|
| Constraint 1 (same-edge belt → UG-input) | ~12 | 15 min |
| Constraint 2 (InputPort src-cell direction) | ~15 | 30 min — reification pattern is non-trivial |
| Constraint 3 (chained UG) | 0 | — (vacuous in all-south mode) |
| Constraint 4 (UG-output head-on) | ~14 | 15 min |
| `solve_synth_place_dirs` port | ~50 | not in Phase 2C scope |
| Verification harness integration | — | depends on harness state |

Total in-scope change: **~40 LOC**, estimated **1–2 hours** including
local bake verification on a handful of Mode D shapes. The
`solve_synth_place_dirs` extension is deferred; flag it as a follow-on
if the all-south path ships cleanly.

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
and similar compose-generated shapes. Run `SPAGHETTIO_REBAKE_SHAPES`
on each compose-generated shape, replace library entries that bake
clean.

Phase 2B (UG-output collision) and 2C (Mode D) are independent
follow-ups.

## Decision log

- *2026-05-03 — drafted. PR #288 (phase 1) shipped. Investigation
  confirmed the residual sideloads are stage1→junction-src-cell
  sideloads, not within-junction sideloads. Phase 2A is the natural
  next step but blocked on a fresh session for the implementation.*

- *2026-05-03 — Phase 2C scoped. Read `solve_synth_place` (Mode D)
  thoroughly. Key finding: all four constraints translate directly,
  but constraint 3 (chained UG sideload) is vacuous in the all-south
  path because Mode D only generates south-facing UG arcs. Constraint
  2 (src-cell direction) requires half-reification via `input_at`
  variables (IO port columns are CP-SAT vars, not fixed tuples).
  `solve_synth_place_dirs` deferred — needs constraint 3 and
  direction-reified versions of 1, 2, 4; flagged as Phase 2C.2.*
