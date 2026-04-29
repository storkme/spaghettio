# RFP: Fluid dual-input row geometry (#68)

## Summary

A row template is needed for recipes with **2 solid inputs + 1 fluid
input** on a 3×3 chemical-plant or assembling-machine. The shapes
covered are tier-4 oil chains (`sulfuric-acid`, `coal-synthesis`),
plus several `acid-neutralisation` / `basic-oil-processing` /
`steam-condensation` variants of similar pitch. The current
`fluid_input_row` runs at a 3-tile machine pitch and silently drops
`solid_inputs[1]` — there is no room for two inserters and a
machine-to-machine fluid chain in three columns. This RFP enumerates
the geometry options that fit the Factorio rules, makes the
trade-offs against existing layout invariants explicit, and records
kill criteria for the route we recommend so a future debugger can
tell quickly whether the chosen geometry is the wrong one. The
recommendation is **Option B (vertical fluid bypass via PTG drop)**:
keep machine pitch at 3, push the fluid chain to a row *above* the
solid belts, and tunnel each machine's port via a PTG pair under the
solid-belt rows. This is the only option in the field that preserves
the existing 3-tile pitch for every other row kind while unblocking
2-solid + 1-fluid without introducing new lane-throughput or
balancer-alignment regressions.

## Motivation

Concrete failing case: `tier4_advanced_circuit_from_ore_am1` (in
`crates/core/tests/e2e.rs`) is currently `#[ignore]` and times out at
30s when un-ignored. The dependency tree of `advanced-circuit` from
ores includes `sulfuric-acid` and (via Space Age recipes) chains
through `coal-synthesis` — both of which are 2-solid + 1-fluid
recipes.

`fluid_input_row` (`crates/core/src/bus/templates.rs:1305`) packs the
single-solid case into 3 tiles per machine:

```text
(mx)     pipe-to-ground EAST input  [fluid chain + machine port]
(mx+1)   inserter                   [drops solid into machine]
(mx+2)   pipe-to-ground EAST output [fluid chain continuation]
```

For 2 solid inputs we need **two inserters** at the north-face row.
That leaves only one tile for fluid handling, but the
machine-to-machine fluid chain needs an inbound and outbound PTG (2
tiles). Total demand: 4 entities in 3 tiles → infeasible at the
current pitch.

The placer in the Python predecessor handled this by classifying as
`FluidInput` and using `solid_inputs[0]` only — `solid_inputs[1]`
was dropped without a warning, leaving floating belts and
unconnected inserters in the layout. A naive `fluid_dual_input_row`
that duplicated the entity stamping in the same row went from 46 to
71 duplicate-tile errors on `advanced-circuit @ 2/s` (recorded in
the issue).

### Affected recipes

From the `advanced-circuit` solver result with raw-ore inputs:

| Recipe | Inputs | Output | Shape |
|---|---|---|---|
| sulfuric-acid | sulfur + iron-plate + water | sulfuric-acid (fluid) | 2 solid + 1 fluid → 1 fluid |
| coal-synthesis | carbon + sulfur + water | coal | 2 solid + 1 fluid → 1 solid |
| acid-neutralisation | calcite + sulfuric-acid | steam | 1 solid + 1 fluid → 1 fluid |
| sulfur | water + petroleum-gas | sulfur | 0 solid + 2 fluid → 1 solid |
| basic-oil-processing | crude-oil | petroleum-gas | 0 solid + 1 fluid → 1 fluid |
| steam-condensation | steam | water | 0 solid + 1 fluid → 1 fluid |

Only the **2 solid + 1 fluid** rows are in scope here — sulfur is a
2-fluid + 0-solid case (already covered by `fluid_multi_input_row`,
see `docs/archive/rfp-multi-fluid-rows.md`); the 0-solid + 1-fluid
cases on chemical-plants are degenerate (no inserter row needed) and
covered by `fluid_only_row` once `OilRefinery`-style port handling
is generalised.

The 1-solid + 1-fluid → 1-fluid case (`acid-neutralisation`) is
handled by `fluid_input_row` today — it only drops fluid output
plumbing, not a second solid. That row kind is out of scope for this
RFP except where the recommended geometry happens to subsume it.

## Design

This RFP is **research only**. No code is changed by landing it. The
diff a follow-up implementation PR is expected to produce:

- `RowKind::FluidDualInput` enum variant in
  `crates/core/src/bus/placer.rs`, with classification in `row_kind`
  for `solid_inputs == 2 && fluid_inputs == 1`.
- A new `fluid_dual_input_row` function in
  `crates/core/src/bus/templates.rs`, dispatched from
  `build_one_row`.
- `RowKind::row_height` returns the new row's height; downstream
  `place_rows` flow already accommodates per-kind heights.
- No new modules. No public API change. Bus-router fluid-tap
  emission already calls into `fluid_port_pipes`; the new template
  reports its tap point through the same vector.

### Options

For each option below: estimated diff size in LOC buckets
(<100, 100–500, 500+); which recipes it unblocks; what other
invariants it strains; the Factorio rule that gates feasibility.

#### Option A — Extend machine pitch to 4 tiles for fluid rows

Keep the current single-row geometry; widen `pitch` from `msz` (3)
to `msz + 1` (4). The extra column gives space for two inserters
plus a PTG-pipe-PTG chain.

```text
(mx)   pipe-to-ground EAST in
(mx+1) inserter A
(mx+2) inserter B (long-handed if reach-2 needed)
(mx+3) pipe-to-ground EAST out / spacer
```

- **LOC**: <100 — local change to `fluid_input_row` width math, plus
  threading `pitch_override` through `build_one_row` and the row
  width computation. The new "fluid-only pitch=4" branch in
  `place_rows` is a one-flag conditional.
- **Unblocks**: `sulfuric-acid`, `coal-synthesis` directly.
- **Invariants strained**:
  - **Row width**: every fluid row gets ~33% wider. For
    `sulfuric-acid @ 5/s` (4 chemical-plants) that's +4 tiles per
    row. Across an `advanced-circuit @ 5/s` factory which has
    multiple fluid-row tiers stacked, total layout width grows
    visibly. The lane-planner's left-to-right column packer
    (`lane_order.rs`) re-runs with the new widths, so trunks that
    used to fit between two pitch=3 rows might no longer fit and
    hill-climb past the optimal.
  - **Power pole alignment**: medium-electric-pole has a 7×7 supply
    area (P4). Pitch=3 places one pole per machine pair (covers 6
    columns + small overlap); pitch=4 means a pole per machine. The
    current pole-placement pass in `bus/layout.rs` assumes uniform
    pitch — a mixed pitch=3 / pitch=4 row stack means uneven pole
    runs and possibly extra poles. Not catastrophic, but a hidden
    cost.
  - **Balancer alignment**: balancer templates in
    `bus/balancer_library.rs` are pre-baked at pitch=3 (matching the
    output belt span of solid-only rows). A pitch=4 fluid row's
    output belt run lands on a different x grid than its
    pitch=3 neighbours; the balancer between them either has to
    skip one column or the lane-planner has to inject an alignment
    pad. The decomposed-balancer fix (memory:
    `project_decomposed_balancer_fix`) was specifically about
    family-lane / trunk x-alignment — pitch mismatch is the
    same class of bug.
- **Geometry validity**: trivially valid. Inserter B at `mx+2` is
  reach-1 to the machine at `mx+1..mx+3`. The PTG pair at `mx`/`mx+3`
  is a clean horizontal chain (rule F4). Belt connections are
  unaffected. The only new constraint is the *between-rows* one:
  fluid pitch=4 in a stack of pitch=3 rows.

#### Option B — Vertical fluid bypass (PTG drop, multi-row)

Keep machine pitch at 3. Move the machine-to-machine fluid chain off
the inserter row entirely: run a continuous east-west pipe header
*above* the solid-input belts, and drop each machine's port via a
vertical PTG pair through the belt rows.

```text
y+0 : ─── pipe ─── pipe ─── pipe ───   (continuous fluid header)
y+1 : ────── PTG-S (drop into tunnel) ──────
y+2 : solid belt 1 (E)        ─crosses above tunnel─
y+3 : solid belt 2 (E)        ─crosses above tunnel─
y+4 : long-inserter / inserter / PTG-N (tunnel out, faces port)
y+5..y+7 : machine (3×3)
y+8 : output inserter
y+9 : output belt
```

Total row height **9 tiles** vs. **8 tiles** for `FluidInput` /
`DualInput` and **7 tiles** for `SingleInput`. Per-machine column
budget at the inserter row is now: `port_dx` (PTG-N) + 2 inserter
columns = 3 columns — fits exactly in pitch=3.

The two solid belts use long-handed-inserter for the far belt
(reach-2; rule I4) and a regular inserter for the near belt. Per
rule **I8a**, only `long-handed-inserter` is reach-2 in vanilla 2.0
— there is no reach-2 fast/stack variant — so a far-side input
whose per-machine demand exceeds **~1.2/s** (long-handed
throughput) caps the row's machine count. Layout consequence: the
**higher-rate solid input must be assigned to the near slot**.

Fluid isolation between adjacent fluids is preserved because the
PTG pair's two perpendicular sides have no surface fluid (rule
**F5a**) — vertical PTGs sitting next to the y+2 / y+3 belts don't
leak into anything.

- **LOC**: 100–500 — new `fluid_dual_input_row` template (~150 LOC
  by analogy with `fluid_input_row`), `RowKind` variant, classifier
  branch, height entry. No new modules. No router changes (taps the
  bus at `y+0`, exactly where `fluid_input_row` does).
- **Unblocks**: `sulfuric-acid`, `coal-synthesis`. Does **not**
  generalise to multi-fluid (covered separately by
  `fluid_multi_input_row`).
- **Invariants strained**:
  - **Row height**: +1 tile vs. the current `FluidInput` height
    (8 → 9). The placer already supports per-kind heights, so this
    is a localised cost. Layout area regresses by `+1 × #fluid_dual
    rows`; on AC@from-ores that's 2 rows × 1 tile = 2 tiles vertical.
  - **Far-slot throughput cap (I8a)**: imposes a **per-machine
    rate ceiling of ~1.2/s** on the slot fed by long-handed
    inserter. For `sulfuric-acid` (per-machine sulfur demand
    ~0.4/s, iron-plate ~0.4/s, water plenty) this is fine. For
    higher-rate variants it bites — the placer must rank inputs
    by per-machine demand and put the higher-rate one in the near
    slot. This is the same logic already used for
    `RowLayout::HorizontalStack` (memory:
    `feedback_uniform_output_balancing`-adjacent).
  - **Belt routing under PTG drops**: the y+2 / y+3 solid belts
    must run *under* the y+1 / y+4 PTG pair — i.e. the belts pass
    over the tunnel section. This is fine by rule U4 (items
    travel underground past entities on tiles between input/output
    of a UG belt). The PTGs being *pipes* not belts means the
    belts don't even see them — pipes and belts on the same column
    just don't conflict (rule F6). The only failure mode is if the
    PTG vertical span exceeds the max underground reach for pipes
    (vanilla 10 tiles; rule F4); 3 tiles here is comfortable.
  - **Solid input belt placement**: the input belts at y+2 / y+3
    feed straight east into the row (rule B7) — both lanes load
    normally. No sideload-onto-UG-input concerns (rule U7) because
    the belts are surface only.
- **Geometry validity**: load-bearing rules F4, F5a, F6, U4, B7,
  I3/I4, I8a. All standard. The `fluid_input_row` in the codebase
  today already implements the chemical-plant T-junction variant of
  this pattern (`templates.rs:1357` onwards) — so this option is an
  *extension* of an existing geometry, not new physics.

#### Option C — Per-machine fluid tap-offs from the bus

Skip the machine-to-machine fluid chain entirely. Each machine in
the row gets its own dedicated tap-off from the fluid bus lane,
running south alongside the machine. The inserter row recovers the
tile that was previously used for the PTG out.

```text
y+0 : machine row pipe header (one tile per machine, isolated)
        but each machine's fluid trunk goes back to the bus
        directly, not through a shared header
y+1 : (free) inserter A
y+2 : inserter B
y+3..y+5 : machine
y+6 : output inserter
y+7 : output belt
```

The fluid tap-off path is now a **per-machine column** running south
from the fluid trunk to each machine's port. The bus lane planner
(`lane_planner.rs`) currently emits one fluid lane per row that
taps once at the row's west edge; this option requires K tap points
per row (one per machine).

- **LOC**: 500+ — major change. `lane_planner.rs` needs multi-tap
  support; `ghost_router.rs` needs the routing logic to handle K
  fluid drops per row without collisions; `templates.rs` gets a
  thinner row template but now owns the per-column trunk anchor
  metadata. Likely touches `BusLane` / `LaneFamily` types.
- **Unblocks**: `sulfuric-acid`, `coal-synthesis`. Also potentially
  cleans up the `fluid_input_row` chemical-plant T-junction once
  generalised, but that's a refactor not a fix.
- **Invariants strained**:
  - **Bus column budget**: K extra columns per fluid-using row.
    For 4 chemical-plants in a row → 4 fluid columns → +4 tiles row
    width *plus* the trunk separation gap. Worse than Option A on
    layout area.
  - **Lane-planner contract**: today `fluid_port_pipes` returns
    one tap per fluid item per row. K tap points break this
    contract — `ghost_router.rs::route_bus_ghost` needs new logic
    to multiplex.
  - **Trunk fanout**: one fluid trunk fanning out to K vertical
    branches creates K junction zones in the ghost router. Each
    junction has to land in the SAT solver. The current solver's
    capacity is already a known issue (`docs/rfp-junction-solver-
    capability.md`); this option pessimises that.
- **Geometry validity**: all moves are individually valid. The
  problem is *budget*, not legality.

#### Option D — Mix solids onto one belt upstream (lane-balanced + filter inserters)

Send both solid inputs on a single belt with sulfur on lane A and
iron-plate on lane B (rule B2/B5: lanes are independent). Each
machine uses a single **filter-inserter** that picks specifically
from one lane, with a second filter-inserter for the other lane.
Two filter-inserters on the same single belt = one tile each, not
two; the inserter row keeps pitch=3.

```text
y+0 : pipe-to-ground EAST in
y+1 : filter-inserter (picks lane A)  [near lane = lane A by I5]
y+2 : pipe-to-ground EAST out
... but wait, need both inserters on one belt + the fluid chain
```

The closer you look the more this option *also* needs vertical
bypass to fit two filter-inserters in pitch=3 alongside the fluid
chain. The "savings" come only if you're willing to delegate the
solid-input belt entirely to a second tile away (one shared belt
serving both lanes filter-picked). At that point you're solving a
different problem — upstream lane-balancing infrastructure — and
the fluid row geometry is barely simplified.

- **LOC**: 500+. Requires upstream lane-balancer authoring (no
  existing template), filter-inserter recipe in the recipe DB,
  validation support for filter-by-lane, and the inserter-throughput
  unit test corpus.
- **Unblocks**: same set as A/B/C, but only after the lane-balancer
  prerequisite work lands.
- **Invariants strained**:
  - **Filter-inserter throughput**: filter-inserter is a regular-
    speed inserter (~0.84/s base) with a filter rule. Rate ceiling
    for a row using lane-filtered pickup is approximately
    `0.84 × machine_count` per ingredient, which is below
    long-handed (~1.2/s) — a regression vs Option B.
  - **Upstream complexity**: the bus now has a "mixed" lane that
    encodes lane assignments. No other row type in the engine has
    this contract. Solver / partitioner / balancer all need to
    learn it.
  - **Validator scope**: the lane-rate walker
    (`validate/belt_flow.rs`) already handles per-lane mechanics
    but doesn't model "filter by lane on pickup" — it'd need
    extension.
- **Geometry validity**: valid in the game. Practically infeasible
  in this codebase as a fix for the geometric problem; it's a
  larger architectural change that happens to also resolve #68 as a
  side effect. **Reject for #68; revisit only if a separate RFP for
  upstream lane mixing lands first.**

### Recommended path

**Option B** (vertical fluid bypass via PTG drop). This is the
single recommendation; A is rejected as cheaper-but-strains-more-
invariants, C as overbuilt, D as out-of-scope.

Why B:

1. **Pitch invariance.** Every other row in the layout stays at
   pitch=3. No mixed-pitch alignment debt; balancer templates,
   pole spacing, and lane-column packing are unaffected.
2. **Existing precedent in the engine.** `fluid_input_row` already
   stamps a chemical-plant T-junction with PTG drops (the unified
   T pattern, `templates.rs:1357` onwards). Option B is the same
   geometry generalised to two solid inputs — not a new pattern,
   an extension. Reviewer load is low.
3. **Far-slot throughput cap is acceptable for the target shapes.**
   `sulfuric-acid` per-machine demand for both solids is well below
   1.2/s; `coal-synthesis` is similar. Higher-rate variants would
   need re-ranking, but the *chemistry* (small molecules, slow
   recipes) bounds rates naturally.
4. **Multi-fluid is cleanly separable.** Option B handles 2 solid +
   1 fluid; the existing `fluid_multi_input_row`
   (`docs/archive/rfp-multi-fluid-rows.md`) handles 0 solid + 2
   fluid via stacked-T. The two RFPs are orthogonal and compose:
   if a recipe needs 2 solid + 2 fluid in the future, the design
   stacks B's PTG drops on top of multi-fluid's stacked-T headers
   without conflict.

The acknowledged tradeoff: **+1 row tile and a per-machine far-slot
throughput cap of ~1.2/s**. The first is local, monotonic, and
easy to measure. The second forces the placer to rank solid inputs
by demand — but that ranking exists already for
`RowLayout::HorizontalStack`, so the code shape is known.

## Kill criteria

Required. Each criterion is observable on a specific scoreboard
case or test, and either firing means we abandon Option B and
revisit (most likely Option A as the consolation).

1. **`tier4_advanced_circuit_from_ore_am1` does not reach a
   layoutable state within 30s after the implementation lands.**
   Current behavior: 30s timeout. If after the new template the
   test still times out, the geometric blocker wasn't actually #68
   — it's something else (likely [#136 missing balancer
   templates](https://github.com/storkme/fucktorio/issues/136) or a
   junction-solver capacity wall). Conclusion: this RFP is not the
   bottleneck; close it without claiming progress, do not pivot to
   Option A.

2. **Adding `fluid_dual_input_row` causes the existing tier-3
   tests to regress.** Specifically, if `tier3_sulfuric_acid` or
   `tier3_plastic_bar` go from green to non-zero validation
   warnings/errors, the new classifier branch is mis-shadowing
   `fluid_input_row` (the existing 1-solid + 1-fluid path). The
   `RowKind` dispatch is wrong and Option B's classifier needs
   redesign before any geometry work continues.

3. **Layout area for `tier3_sulfuric_acid` regresses by >15% after
   the change** (today: a known baseline; measure as `width ×
   height` from the e2e harness). Option B should *not* affect
   tier-3 because tier-3 doesn't use the new row kind. If it
   regresses, the placer's height-per-row math has a hidden
   coupling that needs untangling first.

4. **More than 300 LOC of new template code in
   `templates.rs`** (excluding tests). The existing
   `fluid_input_row` is ~210 LOC; a 2-solid extension shouldn't add
   more than that. If the diff blows past 300 LOC, the abstraction
   is wrong and we should consolidate `fluid_input_row` and
   `fluid_dual_input_row` into a parameterised single function
   *before* shipping — not after.

5. **Implementing Option B reveals that the bus router cannot
   place its fluid trunk at the new row's `y+0` without conflicting
   with the row above's output belt at `y_prev_end`.** Fluid trunks
   currently emerge from a tap point that the lane planner already
   reserves; if a 9-tile-tall row pushes the next row down past
   the trunk's reach (or a balancer's vertical span), the
   `RouteFailure` trace events will show it. If `RouteFailure` event
   counts on the scoreboard cases that include `FluidDualInput`
   rows go up by ≥1 per such row, Option B's height assumption is
   wrong — pivot to Option A (pitch=4) where the row stays at 8
   tiles.

## Risks

- **Power pole spacing.** Medium-electric-pole has 7×7 coverage
  (rule P4). A 9-tile row + 1-tile gap = 10 tiles vertical between
  recipe groups, just inside the worst-case. Adding multiple
  consecutive `FluidDualInput` rows could push a machine out of
  any pole's coverage if the pole-placement pass assumes a uniform
  ≤8-tile row pitch. Mitigation: audit `bus/layout.rs::place_poles`
  before implementation; the current code uses a per-row offset
  rather than a global step, so this is likely fine — but verify.

- **Balancer-row alignment.** Balancer templates align to a row's
  output belt y-position. A taller row shifts that y-position; the
  balancer below the row must absorb the shift. The decomposed-
  balancer fix landed once for similar reasons (memory:
  `project_decomposed_balancer_fix`); the same class of bug could
  re-emerge here if the new row's `output_belt_y` isn't reported
  consistently. Mitigation: `RowSpan::output_belt_y` is the
  contract; the new template must populate it correctly. Tests
  should assert it.

- **Ghost-router pathing.** The `y+1` PTG-S column is a "permanent
  obstacle" for solid belts to the west. The ghost router's
  occupancy map (`ghost_occupancy.rs`) needs to mark it as such, or
  ghost A* will try to route a tap-off through that column, hit the
  PTG, and emit `RouteFailure`. Mitigation: stamp PTGs as `Permanent`
  occupancy at template-emit time, same as
  `fluid_input_row` does today.

- **Inserter direction asymmetry.** The recommended geometry uses
  long-handed for the far slot only. If the placer ever swaps the
  near/far assignment after the template stamps (e.g. if a future
  re-rank by demand fires after row construction), the long-handed
  inserter would be on the wrong slot and silently underflow. Make
  the rank decision *before* template invocation, with a single
  source of truth.

- **Mirror / Space Age oddities.** Some chemical-plant fluid ports
  behave differently with `mirror=true` (rule SA1). If any 2-solid
  + 1-fluid recipe in the recipe DB requires the mirrored port
  variant, the PTG drop column needs to swap from `port_dx=0` to
  `port_dx=2`. `fluid_input_row` already parameterises `port_dx`
  via `fluid_input_port_dx` — Option B inherits the same hook.

- **Template duplication risk vs. Option D's "fix everything at
  once" temptation.** Reviewers may push for a unified solid-mixing
  upstream (Option D) "since we're touching this code anyway." That
  is a separate RFP. Resist scope creep: Option B is the minimum
  viable fix.

## Out of scope / deferred

- **Multi-fluid recipes (≥2 distinct fluids)** — covered by
  `fluid_multi_input_row` and `docs/archive/rfp-multi-fluid-rows.md`.
- **Recipes with 2 solid + 2 fluid** — composes B with multi-fluid
  but no current recipe in the DB requires it.
- **Filter inserter / lane-mixing infrastructure** — tracked as a
  separate prospective RFP (Option D above).
- **Pitch=4 retrofit** (Option A) — kept as the named fallback if
  any kill criterion fires, but not a parallel work track.
- **Per-machine fluid taps** (Option C) — would solve the same
  problem at higher cost; revisit only if multi-row layouts develop
  a separate need for per-machine fluid topology (none today).
- **`tier4_advanced_circuit_from_ore_am1` un-ignoring** — the
  fluid-row fix is necessary but **not sufficient** for that test
  to go green; it also needs balancer template work
  ([#136](https://github.com/storkme/fucktorio/issues/136)). The
  un-ignore is gated on both RFPs landing.
- **Updating CLAUDE.md tier-ladder language** beyond a one-line
  cross-reference to this RFP — the ladder reflects the *current*
  blockers; once Option B lands, that's a follow-up doc PR.

## Verification plan

Per the [verification protocol in
`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Full e2e suite stays green.** All currently-passing tier-1/2/3
   tests must remain green; no behaviour change for any row that
   doesn't classify as `FluidDualInput`.
2. **New active test** — un-ignore `tier3_sulfuric_acid` if not
   already on, and assert zero validation errors with two different
   counts (1 and 4 chemical-plants) so the multi-machine case is
   exercised.
3. **Browser eyeball** at
   `?item=sulfuric-acid&rate=5&machine=chemical-plant&in=sulfur,iron-plate,water`
   — confirm the fluid header runs continuously above the solid
   belts, both inserters reach their machine, and the PTG pair
   isolates the fluid network from any neighbouring fluid.
4. **Snapshot inspection** with `FUCKTORIO_DUMP_SNAPSHOTS=1` — at
   the new template's `y+1` and `y+4` rows, confirm the PTG pair
   exists per machine and the items at `y+2 / y+3` are
   transport-belts carrying the expected solid items. Check
   `RouteFailure`, `BridgeDropped`, and `JunctionGrowthCapped`
   trace events — should be unchanged or zero on tier-3 cases.
5. **Clippy + WASM build** both clean.

## Phasing (optional)

Option B is one PR. If the diff exceeds the kill-criterion 4 budget
(>300 LOC), split into:

1. Refactor `fluid_input_row` to extract a shared "T-junction with
   PTG drop" helper — no behaviour change.
2. Add `fluid_dual_input_row` as a thin caller of the helper.
3. Add `RowKind::FluidDualInput` and dispatch.

Each phase is independently green on tier-1/2/3 tests.

## Decision log

- *2026-04-29 — proposed. Pending review.*
