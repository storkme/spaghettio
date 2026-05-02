# RFP: lane-aware CP-SAT belt routing

## Summary

Extend the CP-SAT belt router (`scripts/cp_sat_placer.py::_route_belts`) to
track per-lane flow rates instead of treating each belt as a single fluid
stream. This is the prerequisite for placing balancer shapes that require
merging (any `(n, m)` with `n > 2`) or feedback channels (every coprime
shape — `(1, 5), (1, 6), (1, 7), (1, 9), (4, 9), …`). Without lane awareness
those layouts pass our all-fluid verifier but back up at runtime in
Factorio because a single belt lane carries flow above its `0.5` normalized
cap.

The deliverable is a router that:

1. Models each belt tile as two per-lane flow variables instead of one
   per-direction belt indicator.
2. Encodes the Factorio sideload rule (perpendicular feed lands on the
   lane facing the source).
3. Caps per-lane rate at `0.5` in normalized units.
4. Models splitter lane semantics so a splitter can be used as a
   lane-balancer when needed.

After this lands, the existing dyadic shapes still place identically; the
new shapes that previously needed sideloading (e.g. `(3, 4)`) can place
correctly using a lane-balancer splitter or wider channels; coprime
shapes (`(1, 5)` and beyond) can place with feedback channels that don't
saturate.

## Motivation

### Concrete failures

- **Reverted in `fe09494`**: `(3, 4)`, `(3, 8)`, `(3, 16)`, and `(1, 5)`.
  Each placed successfully under the relaxed routing model but had
  belt/lane overloads that the fluid verifier (Couëtoux all-fluid) didn't
  catch.

  - `(3, 4)`: input belt at `(1, 0)` carried `2.0` on a `1.0`-cap belt
    (direct input + perpendicular sideload). Total throughput, not just
    lane.
  - `(1, 5)`: 3 feedback arcs (rate `0.2` each) all sideloaded onto the
    row-7 east channel from the north → all `0.6` ended up on the left
    lane (cap `0.5`). The other lane sat empty.

- **Issue [#136] still blocked.** The 20 missing coprime shapes in
  `1..=10 × 1..=10` all need feedback from the leaf level back to a
  merger splitter. The synth produces these graphs; the placer can't
  realize them lane-safely.

### Why the all-fluid verifier doesn't catch this

`balancer/verify.rs` solves a steady-state linear system over arc rates
under conservation. It treats each arc as a single rate `r ∈ ℝ` with no
upper bound — the Couëtoux model assumes "infinite belt" semantics
matching the saturation-rich proof framework. That's correct for the
mathematical balance property but blind to physical belt and lane caps.
We won't change the verifier; we'll teach the *router* to respect caps
the verifier ignores.

The lane-aware belt walker in `validate/belt_flow.rs` does track lanes,
but it runs over an already-placed template — too late to influence
routing decisions. Its rules are the source of truth for the Factorio
semantics we need to encode in CP-SAT.

## Design

### Variable changes

Replace the current per-tile, per-direction belt indicator with per-lane
flow variables:

**Old (current):**
```
belt[t][d] ∈ {0, 1}     # tile t carries a belt heading direction d
f[r][t][d] ∈ {0, 1}      # route r passes through tile t with belt direction d
```

**New (lane-aware):**
```
belt[t][d] ∈ {0, 1}                      # unchanged: tile structure
flow[r][t][d][lane] ∈ {0, 1}             # route r uses lane L of tile t (d = belt dir)
lane_used[t][d][lane] ∈ {0, 1}           # any route uses this (t, d, lane)
```

`lane ∈ {LEFT, RIGHT}` relative to the belt's direction of travel.

The variable count grows by ~2× for the flow vars, which is the dominant
term. For a 16×8 grid with ~20 routes, this is `(128 free tiles) × 4 dirs
× 2 lanes × 20 routes ≈ 20K bools` — still well within CP-SAT's
comfortable range.

### Constraints

**Lane caps.**
```
sum over routes r: flow[r][t][d][lane] ≤ 1     # at most one route per (tile, dir, lane)
```
We're modeling rates as 0/1 indicators per route here — each route
carries at most one "unit" of flow, and a single lane fits one route. For
multi-arc cases (multiple arcs converging) this still works because each
arc gets its own route. To model fractional lane occupancy (e.g. a
0.25-rate route only taking a quarter of a lane) we'd extend to integer
flow with a denominator — deferred unless it turns out we need it.

**Sideload semantics.** When a perpendicular belt feeds into a receiver,
items land on the side of the receiver facing the feeder. Encoded as a
table over `(receiver_dir, feeder_dir) → lane`:

| Receiver dir | Feeder dir (relative side) | Items land on |
|:------------:|:--------------------------:|:-------------:|
| East         | from N (south-flow above)  | LEFT lane     |
| East         | from S (north-flow below)  | RIGHT lane    |
| East         | from W (head-on east)      | preserves lane|
| Other directions: rotate by 90° per direction |

So when route `r` enters tile `t` (heading direction `d_recv`) from
neighbor `n` (heading direction `d_feed`), the constraint is:

```
flow[r][t][d_recv][lane(d_recv, d_feed)] = flow[r][n][d_feed][?]
```

(The `?` lane is whatever lane the route was on at `n`, possibly subject
to similar transition rules at the previous step.)

For head-on flow the lane is preserved; for sideload it's pinned.

**Splitter lane semantics.** A south-facing splitter at `(x, y)` with
inputs at `(x, y-1)` and `(x+1, y-1)`:

- Each input port reads from both lanes of the upstream belt.
- The splitter internally re-distributes: 50% of *each input lane* to
  each output port. This is the "splitter as lane balancer" property.
- Each output port emits items split across both output lanes equally.

For the router's purposes, the rule that matters is: **a splitter
output's lane composition is independent of its input's lane
composition** — items leaving a splitter are 50/50 left/right regardless
of how they came in. This means dropping a splitter into a saturated
single-lane stream re-balances the lanes downstream.

Encoded as: at the output port tile of every splitter, the flow vars on
both lanes are unconstrained from upstream (in particular, a single-lane
input becomes a half-rate two-lane output).

**Source-lane forcing for splitter-output drops** *(the missing piece
for `(1, 5)`)*. The above sets the splitter-internal rule. The router
also needs to force the lane *at the drop tile* based on whether the
drop heads in the same direction as the splitter (head-on) or
perpendicular (sideload from the splitter's face).

For a south-facing splitter `S` at `(x, y)` with output port `p ∈
{0, 1}`, the drop tile is `(x + p, y + 1)`. Let `d_drop` be the direction
the drop tile's belt heads (chosen by the solver):

| `d_drop` | Source-lane rule |
|:---:|---|
| S (continues splitter face) | Head-on: items fill **both lanes**. Model as 2 lane-routes per arc, like input boundaries. |
| E (perpendicular, splitter face on left) | Items sideload from N onto E-belt → LEFT lane (lane 0). Force. |
| W (perpendicular, splitter face on right) | Items sideload from N onto W-belt → RIGHT lane (lane 1). Force. |
| N (against splitter face) | Forbidden — would route items back into the splitter. |

For non-south splitter directions, the table rotates by 90°/180°/270°
according to `_LEFT_OF` / `_RIGHT_OF`.

The router currently treats source tiles as having **free** lane
choice. That's correct for input-boundary belts (where `Task 1` already
emits 2 lane-routes for both lanes) but **wrong for splitter-output
drops** — without source-lane forcing, the model accepts layouts where
3 feedback drops all "freely" pick lane 1 to avoid saturating lane 0,
even though physics says they all hit lane 0.

Encoding: at every source tile that is a splitter-output drop, add a
constraint that pairs `(d_drop, lane)` per the table above. The route
tuple grows an optional `splitter_dir` field to signal "this source is
a splitter-output drop with the upstream splitter facing direction X."
Input-boundary sources omit the field and keep the free-lane behavior.

**Belt structure unchanged.** The `belt[t][d]` indicator and the
"which-direction-per-tile" rules are unchanged. The lane vars are added
*on top of* the existing belt structure, not in place of it.

### Sideload is now permitted (with caps)

The at-most-one-route-per-tile constraint reintroduced in `fe09494`
goes away. Multiple routes may share a tile, subject to:

- Same belt direction.
- Per-lane cap of 1 route per `(t, d, lane)`.
- Each route's lane assignment respects sideload semantics from its
  predecessor.

This means the previously-broken `(3, 4)` and `(1, 5)` layouts will
either solve correctly with the constraint "you can't shove 3 sources
onto one lane" or come back UNSAT, prompting the placer to widen the
channel or insert a balancer splitter. Both are correct outcomes.

### Belt-throughput cap as a side benefit

Once we track per-lane occupancy, the overall belt throughput cap
(`≤ 1.0` normalized) drops out automatically: two lanes × `0.5` per
lane = `1.0`. The `(3, 4)` failure case (`2.0` on a `1.0`-cap belt) was
diagnosed by hand in the post-mortem; the new model would have rejected
it as UNSAT directly.

### Things explicitly out of scope for this RFP

- **Belt tiers / different belt rates.** All shapes in `1..=10 × 1..=10`
  fit in a single tier; mixed-tier balancers are an optimization for
  later.
- **Underground belt support.** UGs preserve lanes through the
  underground span (no flip), so once lane-awareness lands, UGs slot in
  with the same per-lane variables. But we don't need them for the
  shapes this RFP unblocks (the merger sub-network can reach all
  feedback paths via surface belts on grids ~12×10). UG support is
  phase 4 of the placement RFP.
- **Sideload trick variants** (split-on-UG, "balancer compaction"). We
  encode the simple sideload rule and accept that some Factorio
  community tricks won't be expressible. They produce tighter layouts;
  we'll revisit if the bench shows unacceptable footprint regressions.

## Kill criteria

**Required.** Stop or rethink if any of these trip:

1. **Smallest coprime shape `(1, 5)` doesn't solve in under 30 minutes
   on commodity hardware.** Set this as the sentinel for the *easy*
   coprimes — if `(1, 5)` is intractable, the harder ones (`(4, 9)`,
   `(7, 9)`, etc.) almost certainly are too, and the whole approach
   is wrong. Note: 30 minutes is generous because the deliverable is
   **offline library generation**, not interactive solving;
   Factorio-SAT times out at 6+ hours on `(4, 9)` today, so we're
   fighting against an `∞`-time baseline. We'd rather discover this
   sooner than later.

2. **Solve time on a hard coprime shape (`(4, 9)`, `(7, 9)`, or
   `(8, 9)`) exceeds 4 hours.** The current Factorio-SAT timeout on
   these is 6+ hours and they don't finish; bringing them under 4h
   is a real win. Beyond 4h is "still on the wrong side of the
   Factorio-SAT failure mode" — the model isn't unlocking what it
   needs to.

3. **Aggregate library regeneration (all 99 shapes in `1..=10 ×
   1..=10`) takes more than 8 hours of wall-clock time.** This is
   the practical ceiling for an overnight build-time generation
   step. Above this we either need to parallelize aggressively or
   reconsider the encoding.

4. **Sideload-rule encoding requires more than one direct table per
   `(receiver_dir, feeder_dir, in_lane) → out_lane`.** If we end up
   needing per-shape special cases or branching logic ("but in this
   geometry the lane swaps differently"), the rule isn't actually
   encodable as a constraint and we're modelling Factorio-the-game
   instead of Factorio-the-belt-mechanic. Write the table out
   explicitly first; if any cell is "depends on context", stop.

5. **Lane-aware router can't place `(1, 5)` in a 12×10 grid.** That's
   roughly 1.2× the previous (broken) `(1, 5)` footprint — generous
   margin for adding a lane-balancer splitter or widening the feedback
   channel. If 12×10 is UNSAT for `(1, 5)`, our lane semantics are
   too restrictive and probably double-counting some sideload that's
   actually safe.

6. **`(3, m)` shapes still need an n-input merger sub-network we don't
   have.** The whole point of lane awareness is that sideloading
   becomes safe-when-respected. If after this lands `(3, m)` still
   demands a merger, we've solved the wrong problem and should pivot
   to building the merger sub-network instead.

7. **Lane-walker disagreement on placed templates.** Run
   `validate/belt_flow.rs` against every CpSat-produced template; if
   the lane walker reports lane-saturation warnings on > 10% of newly
   covered shapes, the router and walker disagree on Factorio
   semantics. The walker is ground truth (it's been validated against
   real layouts); fix the encoding to match it.

Note on dyadic regressions: phase 1 already shows the lane-vars
refactor costs 2-3× on existing dyadic shapes (`(1, 8)`: 0.85s → 1.7s;
`(1, 16)`: 4s → 12s). This is expected — we're trading speed for
expressiveness. Dyadic regression is **not** a kill criterion; the
goal is coprime coverage, and dyadic shapes are already shipped via
the existing library so a 10-20× regression there doesn't break
anything user-visible.

## Verification plan

Per the [layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).

After each phase:

1. **Regression on the existing 10 round-trip tests.** They should still
   pass byte-for-byte equivalent layouts where the routing model has no
   sideloading to consider. If `(1, 4)` or `(2, 8)` start producing
   different layouts, something in the lane vars is leaking into single-
   lane shapes.

2. **New round-trip tests for `(3, m)` and `(1, 5)`.** Same fluid-model
   verification we have today, plus a new lane-walker assertion on the
   placed template.

3. **Lane-walker cross-check.** New test: for every CpSat-produced
   template (existing 10 + new ones), instantiate it as a
   `BalancerTemplateRef` and walk it through `validate/belt_flow.rs`.
   Assert no lane-saturation warnings. This catches drift between the
   router and the walker.

4. **Browser eyeball** on a tier-4+ recipe that requires `(3, m)` or a
   coprime shape. `processing-unit @ 3/s from ore` is the canary —
   currently uses `(4, 9)` from the library. If we can place `(4, 9)`
   directly and it produces a working bus layout, the model is
   end-to-end correct.

5. **CP-SAT solve time** logged for every shape in
   `tests/cp_sat_round_trip.rs`. Plot wall-time before/after to confirm
   kill criterion #1 hasn't tripped.

## Phasing

1. **Phase 1: per-lane variables and caps.** ✅ Shipped in `c6d1264`.
2. **Phase 2: sideload semantics + lifted tile-exclusivity.** ✅
   Shipped in `e5250a1`.
3. **Phase 3 (foundation): rate-aware caps, dual-lane inputs, splitter
   direction support.** ✅ Shipped in `29f353b` / `1a71e31` / `0c21003`.

The remaining work, in order:

4. **Phase 3 (next): source-lane forcing for splitter-output drops.**
   ✅ Infrastructure shipped. The route tuple now carries optional
   `splitter_dir`; `_route_belts` forces source-lane per the sideload
   table when set; dyadic placers thread `splitter_dir = S` for every
   drop and emit 2 lane-routes for head-on south continuations. All
   10 dyadic round-trip tests stay green; `(1, 16)` solve time bumped
   from ~12s to ~16s (consistent with the head-on-route doubling and
   within the 60s round-trip budget).

   `(1, 5)` validation **moved to phase 4** — see the decision log for
   the analysis. Source-lane forcing is necessary but not sufficient
   for `(1, 5)`: the feedback channel needs to wrap E→N→W back to
   `M.in1`, and every perpendicular turn collapses both lanes onto the
   receiver's LEFT lane regardless of source-lane forcing. With 0.6
   total feedback rate, that violates the 0.5 lane cap. Unblocking
   needs a lane-balancer splitter at the wrap (extra splitter beyond
   what the synth provides) or a topology change.

5. **Phase 4: lane-balancer splitter + cover the rest of
   `1..=10 × 1..=10`.** Phase 4 now bundles two pieces:

   1. **Lane-balancer splitter** in the placer. When a route's
      perpendicular turn would saturate the receiver's lane, the
      placer inserts an extra south-facing splitter at the turn
      (taking a 1×2 bite of the grid) so the items rebalance to 50/50
      on its outputs before continuing. This splitter is *placement-
      only* — the synth graph is unchanged; the recovery in
      `topology_of_template` collapses it back into a single arc.

   2. **Coprime coverage.** With the lane-balancer available, the
      same machinery extends to other coprimes by computing per-route
      rates from the synth flow analysis and choosing grid sizes per
      shape. Shape-specific tuning may be needed for the larger
      coprimes (`(4, 9)`, `(7, 9)`, `(8, 9)`); the lane-walker
      cross-check (verification step #3) catches drift between the
      router and Factorio semantics. Issue [#136] closes here.
   ~1-2 weeks.

6. **Phase 5: bench + library regeneration.** Run the cross-engine
   bench across all 99 shapes; compare against the community library.
   Fold the regenerated templates into `balancer_library.rs`. Update
   `bus::balancer::stamp_family_balancer` to consume the new shapes.
   ~1-2 days.

## Decision log

- *2026-05-02 — RFP drafted after `fe09494` reverted the lane-unsafe
  shapes.*
- *2026-05-02 — Phase 1 shipped in `c6d1264`. Per-lane flow vars added,
  no behavioral change. Cost: 2× on `(1, 8)` (0.85s → 1.7s), 3× on
  `(1, 16)` (4s → 12s). Within the 5x kill criterion at the time but
  trajectory was concerning.*
- *2026-05-02 — Kill criteria revised to anchor on coprime-shape
  solve time (the actual deliverable) rather than dyadic regression.
  The 5× dyadic kill criterion was flagged as too tight given the goal
  is offline library generation, where 20-min single-shape solves are
  acceptable and Factorio-SAT's baseline is ∞ on the target shapes.
  New criteria gate on `(1, 5)` <30min, hard-coprime <4h, full library
  <8h.*
- *2026-05-02 — Phase 2 shipped in `e5250a1`. Sideload table + lifted
  tile-exclusivity. Multiple routes can share a tile in the same
  direction; perpendicular feed → forced lane via the table.*
- *2026-05-02 — Phase 3 task 1 (dual-lane input modeling) shipped in
  `29f353b`. Each direct input emits 2 lane-routes; sideload routes
  remain 1 route. `(3, 4)` now correctly UNSAT.*
- *2026-05-02 — Phase 3 task 2 design captured in
  `docs/scratch-lane-balancer-design.md`. Balancer insertion to absorb
  ≥3 same-side sideloads via the head-on+sideload lane trick.*
- *2026-05-02 — Phase 3 task 3 shipped in `1a71e31`. Rate-aware
  per-lane cap: per-lane caps now use weighted sums
  `sum (rates[r] * f_lane[r, t, d, lane]) ≤ lane_cap` instead of
  unit caps. Defaults preserve discrete behavior; foundation for
  fractional-rate convergence (coprime feedback channels). Existing
  10/10 tests unaffected.*
- *2026-05-02 — Open: `(1, 5)` integration. Rate-aware foundation is
  in place but applying it requires *also* modeling that (a) head-on
  splitter-output drops fill both lanes (so should emit 2 lane-routes
  per arc), and (b) perpendicular drops force a specific lane via the
  sideload table at the source tile. Without these, the routing model
  treats splitter-output sources as freely-laned which under-counts
  saturation. Deferred pending design — three approaches in play:
  source-lane forcing + grid 10×9, structural balancer insertion (per
  the design doc), or both. See `scratch-lane-balancer-design.md` for
  the balancer track.*
- *2026-05-02 — `(1, 5)` deferral re-routed through
  `rfp-lane-safe-synth.md` — but that RFP got tested empirically
  before commit and doesn't hold up: the self-loop construction
  works for single splitters but multi-splitter cascades for K→1
  mergers re-introduce the multi-arc relaxation we were trying to
  eliminate. See the decision log in `rfp-lane-safe-synth.md` for
  the full breakdown. Net: `(1, 5)` integration remains blocked on
  source-lane forcing in the placer (the original lane-aware path),
  not on synth enrichment. The lane-aware infrastructure shipped in
  this PR is the right foundation; what's missing is source-lane
  forcing for splitter-output drops. That's the next concrete piece
  of work.*
- *2026-05-02 — Phase 3 (next) source-lane forcing infrastructure
  shipped. Route tuple extended to `(src, sink, sink_dir,
  splitter_dir | None)`; `_route_belts` forces lane per the sideload
  table at every splitter-output drop; dyadic placers thread
  `splitter_dir = S` everywhere and emit 2 lane-routes for head-on
  south continuations (`L1 → outputs`, `L2 → outputs`, `L3 →
  outputs`). All 10 dyadic round-trip tests pass; `(1, 16)` cost
  rose from ~12s to ~16s (within the 60s round-trip budget — the test
  timeout was raised from 10s to match).*
- *2026-05-02 — `(1, 5)` not unlockable by source-lane forcing
  alone. Walked the layout: `M, S1` tight-stacked vertical (rate-0.8
  arcs preserved as head-on); `S1 → L1` also tight-stacked (rate 0.8
  perpendicular doesn't fit the 0.5 lane cap); `L1 → L2` routing-row
  offset (rate 0.4 perpendicular fits); `L2 → outputs` head-on south
  drops; 3 feedback drops at the bottom row at rate 0.2 each = 0.6
  total. The feedback channel needs to wrap E→N→W back to `M.in1` —
  and **every perpendicular turn collapses both lanes of the feeder
  onto the receiver's LEFT lane** (sideload table: feeder W onto
  N-belt → lane 0; feeder N onto W-belt → lane 0; etc). 0.6 lumped
  onto a single lane violates the 0.5 cap regardless of how the
  drops were distributed at the source. The "1-drop-south-detour"
  trick from the design splits 0.4 + 0.2 across the two lanes of the
  east-bound channel correctly, but the very next turn (E→N at the
  east edge) collapses 0.6 back onto N-lane-0. Unblocking needs a
  lane-balancer splitter at the turn — an extra splitter beyond
  what the synth provides — or a different topology. Bumped to
  phase 4 (now bundled with the lane-balancer-splitter piece).*
- *2026-05-02 — Phase 4 increments 1+2 shipped (per-arc rate
  plumbing + `_add_lane_balancer_south` helper). Wire format gains
  optional `arc_throughputs: Vec<f64>` from `verify_balancer`;
  Python placer exposes `_find_arc_rate(src, dst)` returning scaled
  integer rates (`RATE_SCALE = 10`, `LANE_CAP_SCALED = 5`). Helper
  appends a south-facing placement-only splitter and returns its port
  tiles. No behaviour change for dyadic shapes — they still default
  to discrete unit rates. All 10 round-trip tests stay green.*
- *2026-05-02 — Phase 4 increment 3 (`(1, 5)` placer) parked. The
  layout problem is harder than the design anticipated: rate-0.8
  arcs (`M → S1`, `S1 → L1`) require head-on flow (perpendicular
  doesn't fit the lane cap), forcing a tight-stack vertical chain.
  This pins S2/S3 to `(S1.x ± 1, S1.y + 1)` — adjacent. Their L2
  children then have to be at `(S1.x - 3, S1.x - 1, S1.x + 1, S1.x + 3)`
  if routing-row offset, and the `S2.out1 → S5` (eastward) and
  `S3.out0 → S6` (westward) routes both pass through the inner two
  cols of the routing row, requiring a route-crossing. Without UG
  belts (out of scope per the original RFP) there's no way to
  cross. Alternative — collapse S2/S3 outputs onto a shared L2
  splitter via tight-stack — fails verification because the merged
  splitter outputs at rate 0.4 instead of the expected 0.2.
  Three viable directions for unblocking, each material work:
  (a) add UG-belt support to the placer (matches the existing
  spike at `crates/balancer-gen/scripts/place.py`); (b) add a
  pass-before-route step that re-distributes route paths (e.g.
  swap port assignments to make the cross unnecessary); (c)
  modify the synth to emit a lane-safe topology with extra
  splitters in the tree (revisits the rejected
  `rfp-lane-safe-synth.md` track but with different motivation).
  Phase 4 increments 1+2 shipped as standalone infrastructure
  value; (1, 5) and the rest of coprime coverage need a
  layout-design follow-up before they become tractable.*
