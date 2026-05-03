# Design: Structural lane balancing in CP-SAT belt placer

> Drafted by a planning subagent on 2026-05-02 in response to phase-2's
> rejection of `(1, 5)` and other coprime shapes. This is a working
> document, not yet ratified — the user-facing kill criteria for this
> phase live in `rfp-lane-aware-routing.md`.

## Problem recap

`(1, 5)` has 3 feedback arcs (rate 0.2 each) emerging from L2 leaf ports
at `(5, 7)`, `(6, 7)`, `(7, 7)`. They all flow east in row 7, then turn
north up column 8, then west in row 0 to land at the merger's `in_1`
approach tile `(x_merger + 1, 0) = (4, 0)`. Each route enters the row-7
east channel from the **north side** (sideload from a south-flowing leaf
belt onto an east belt). Per `_route_belts` line 218–220 and the
sideload table in `rfp-lane-aware-routing.md`, sideload from north onto
an east belt lands on lane 0 (LEFT). Three routes ×
`f_lane[(r, t, EAST, 0)] ≤ 1` is `3 ≤ 1` — UNSAT.

This is structural, not a search failure: no permutation of the 3
sideloads avoids the LEFT lane while feeding from the north. A splitter
is the only Factorio mechanic that re-distributes a saturated lane to
a half-rate two-lane stream (encoded in the RFP under "Splitter lane
semantics").

## 1. Detection — recommendation: **static analysis of the route declarations**

Three options were considered:

- **(a) Try-then-retry.** Solve once without a balancer; on UNSAT,
  retry with one inserted. Simple to wire in, but conflates *any* UNSAT
  (grid too small, port mismatch, structural lane saturation) into
  "needs balancer". Each retry is a fresh CP-SAT solve; on `(1, 5)`'s
  80-tile grid this is ~1s today, but on `(4, 9)`-class shapes a retry
  doubles wall-time and we don't know which channel needs the balancer
  (the shape-specific code has to re-compute the layout anyway).
- **(b) Static analysis.** Before calling `_route_belts`, scan the
  routes list and count, for each candidate sink belt direction at each
  tile reachable as a sideload target, how many routes will sideload
  from each side. If `count ≥ 2` from a single side onto a tile that
  all routes share as part of their channel path, flag the channel for
  balancer insertion.
- **(c) Always emit balancers in feedback channels.** Wasteful:
  feedback channels with ≤ 2 sideloads from the same side are already
  lane-safe. Over-conservative also adds 1 splitter ≈ 2 tiles to grids
  that were already minimally sized, risking grid widening for shapes
  that don't need it.

**Pick (b).** The shape-specific functions (`place_one_to_five`, future
`place_one_to_seven`, etc.) already enumerate the feedback arcs
explicitly — they construct the source list `feedback_srcs` and a
single `feedback_sink`. The detection rule reduces to "if
`len(feedback_srcs) ≥ 3` and they converge on a shared east/west
channel sideloaded from a common side, insert a balancer." This is
local, deterministic, and does not require any solver round trip.

Static analysis lets us decide grid dimensions *before* calling
`_route_belts`. With (a) the placer would have to widen the grid AND
re-place splitters AND re-call the routing solver — three nested
retries.

Failure mode of (b): false negatives if a shape-specific function emits
routes that *converge implicitly* (different sources, same channel
discovered by the router). Mitigation fallback: if `_route_belts`
returns None and no balancer was inserted, retry with one — i.e., add
option (a) as a *fallback* to option (b)'s primary path. Belt and
suspenders.

## 2. Geometry — placement rule

The balancer is a 2x1 splitter inserted **inline in the saturated
channel**, downstream of the convergence and upstream of the channel
sink.

**Placement rule.** For a saturated channel `C` with flow direction
`d`:

1. Pick the *longest contiguous straight run* of `C` after the last
   sideload point.
2. The balancer occupies two tiles perpendicular to `d`. For
   north-flow this is `(p, q)` and `(p+1, q)` if the balancer faces
   north (anchor convention matches existing south-facing splitters:
   anchor at left tile of the 2-wide footprint).
3. The channel must be widened to 2 columns (for vertical channels)
   or 2 rows (for horizontal) for the length of the splitter footprint
   plus its input and output approach tiles.

For `(1, 5)`: the natural placement is on **column 8–9 of rows 4 and
5** (or equivalent). A north-facing splitter with anchor at `(8, 4)`
occupies `(8, 4)` and `(9, 4)`; inputs at `(8, 5)` and `(9, 5)`;
outputs at `(8, 3)` and `(9, 3)`.

The current `(1, 5)` layout already reserves cols 8–9 ("Cols 8-9 left
free for the feedback channel"). Column 9 was already free, just
unused by the lane-unsafe layout. The fix uses the slack column for
the balancer's second lane.

## 3. Route impact — additive integration with `_route_belts`

The lane-balancer splitter is just a regular splitter from
`_route_belts`'s perspective. The current API takes
`splitter_positions: list[tuple[int, int]]` and
`routes: list[tuple[src, sink, sink_dir]]`. We do **not** need a
"floating splitter"; the balancer's position is decided by the
shape-specific function (`place_one_to_five`) before calling
`_route_belts`, exactly like the merger and tree splitters.

The shape function is already a CP-SAT model for splitter placement.
We add the balancer as splitter index 8 (after the 7 inner-tree
splitters) with structural constraints that anchor it inside the
feedback channel.

The routes list changes:

- Each of the 3 feedback routes splits into two segments: one
  pre-balancer (`feedback_src` → balancer input drop) and one
  post-balancer (balancer output drop → `feedback_sink`).
- Items leaving the balancer are 50/50 across both lanes regardless of
  how they entered. Two post-balancer routes share row 0's west channel
  with each route on its own lane after the natural-input-from-back
  rule preserves whatever lane the solver picked at the balancer
  output.

**API impact on `_route_belts`: zero.** The balancer enters via
`splitter_positions`, the routes are restructured by the shape
function. `_route_belts` doesn't need to know "this splitter is a
balancer" — splitter outputs already drop the lane-coupling constraint
because each output drop tile is the *src* of a new route, and srcs
are unconstrained by upstream flow (per the existing source-handling
in `_route_belts`).

The one structural change required: **support splitter directions
other than south.** Currently `_splitter_tiles` and the rest of the
placer assume south-facing splitters everywhere (`direction: 4`). Adding
a north-facing balancer requires a small extension: a `splitter_dir`
field per splitter, with `_splitter_tiles` agnostic on footprint (still
2 tiles wide perpendicular) and the route construction flipping
input/output rows for non-south splitters.

## 4. Splitter port assignment — the "head-on + sideload" lane trick

3 same-side sideloads onto a single belt is UNSAT. But **2 routes can
share a belt if one enters head-on and one enters sideload-from-north**:
head-on takes lane 1 (free choice), sideload pins to lane 0. Lane cap
satisfied (1 route per lane).

For the balancer's 2 input ports:
- **Port 0** receives 2 routes via "1 head-on + 1 sideload" arrangement
  (2 srcs collapsed onto one belt feeding port 0).
- **Port 1** receives 1 route head-on (1 src directly).
- **Outputs** emit 2 routes each on their own port → 2 routes on 2
  ports = trivially safe.

For 3 srcs total, 2 + 1 = 3, fitting cleanly into a 2-input balancer.

For larger source counts: balancers needed = `ceil((n_srcs - 2) / 2)`
per saturated channel. `(1, 5)`: `ceil(1/2) = 1`. `(1, 7)` (5 srcs):
`ceil(3/2) = 2`.

## 5. Concrete sketch for `(1, 5)` (grid 10×9)

```
      0 1 2 3 4 5 6 7 8 9
   0  . . . . S . . . . F     row 0: input belt + feedback merge into merger.in_1 at (4,0)
   1  . . . X X . . . . F     row 1: merger
   2  . . . . . . . . . F     row 2: merger→root drop
   3  . . . X X . . . . F     row 3: root
   4  . . . . . . . . X X     row 4: BALANCER at (8,4)–(9,4) north-facing
   5  . X X . . X X . F F     row 5: L1; balancer inputs at (8,5)/(9,5)
   6  X X X X X X X X . F     row 6: L2 cols 0-7; col 8-9 free
   7  O O O O O * . . F F     row 7: 5 outputs + 1 sideload src at col 5
   8  . . . . . . * * F .     row 8: 2 srcs routed via row 8 east, sideload-from-north
```

Splitter list (existing + balancer):

```
splitter_positions = [
  (3, 1, S),   # merger
  (3, 3, S),   # root
  (1, 5, S), (5, 5, S),  # L1
  (0, 6, S), (2, 6, S), (4, 6, S), (6, 6, S),  # L2
  (8, 4, N),   # balancer (north-facing — new direction support required)
]
```

Routes (replacing the 3-route feedback block in `place_one_to_five`):

- **3 pre-balancer routes**:
  - `((5, 7), (8, 5), DIR_S)` — east in row 7 to col 7, then south to
    row 8, head-on into the row-8 channel.
  - `((6, 7), (8, 5), DIR_S)` — sideloads from north onto the row-8
    east channel.
  - `((7, 7), (9, 5), DIR_S)` — separate channel via col 9, head-on
    into balancer port 1.
- **2 post-balancer routes**:
  - `((8, 3), (4, 0), DIR_S)` — north up col 8, west in row 0 to the
    merger feedback approach.
  - `((9, 3), (4, 0), DIR_S)` — symmetric on col 9.

Lane analysis at the row-8 east channel feeding port 0:
- Src `(5, 7)` enters head-on (route's first tile in row 8 is the
  westmost tile of the channel). Solver freely picks lane → say lane 1.
- Src `(6, 7)` sideloads from north. Pinned to lane 0.
- Per-lane cap satisfied: 1 route on each lane. ✓

## Failure modes / edge cases

1. **Balancer-direction support required.** Current placer is
   south-only. Adding north (and eventually east/west) requires a
   `direction` field on `splitter_positions` entries; mechanical but
   real change.
2. **Grid growth cascades.** Adding rows for the balancer's footprint
   may push other splitters off-grid in tighter shapes. For `(1, 5)`
   the height grows from 8 to 9 (under the 12 cap from kill criterion
   3 in the RFP).
3. **More than 3 sideloads from the same side.** Cascade balancers per
   `ceil((n_srcs - 2) / 2)`. `(1, 7)` (5 srcs) needs 2 balancers.
4. **Cycle in routing.** Once the balancer enters the splitter list
   with structural constraints, the post-balancer route shares row 0
   with the input belt. Lane analysis confirms no conflict (input belt
   at `(3, 0)` flows south, not east-west) — but check on each new
   shape.
5. **Solve-time blow-up.** Adding 1 balancer = +2 splitter tiles and
   +5 routes (3 pre + 2 post replacing 3 original), variable count
   grows ~50%. Expect 2–5s on `(1, 5)` with balancer (was 0.8s
   pre-revert). Comfortably under the 30-min kill criterion.
6. **Detection false negative.** If a shape function emits routes
   whose convergence isn't apparent at declaration time, static
   detection misses it and `_route_belts` returns UNSAT. Fallback to
   option (a) on UNSAT-without-balancer.

## Summary

Detect lane-saturated channels by **static analysis of the routes
list** at shape-function build time (count same-side sideloads onto a
shared sink channel). **Insert a splitter** as the balancer by adding
it to `splitter_positions` with a direction tag (mechanical extension
to the current south-only convention). **Restructure the affected
routes** so each balancer input port receives ≤ 1 same-side sideload
(combining head-on and sideload entries on the same belt), and replace
each pre-balancer route with one segment ending at a balancer input
plus one segment starting at a balancer output. For `(1, 5)`, grow the
grid from 10×8 to 10×9, place the balancer at `(8, 4)` (north-facing),
and split the 3 feedback arcs into a "2 head-on + 1 sideload"
arrangement. `_route_belts` needs no API change; the splitter
direction tag is the only structural addition.
