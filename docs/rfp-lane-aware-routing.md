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

Plan to land in chunks. Each phase ships a working subset.

1. **Phase 1: per-lane variables and caps, no sideload semantics.**
   Replace `flow[r][t][d]` with `flow[r][t][d][lane]`. Add per-lane caps.
   Since no sideload rule yet, every route still has a single
   predecessor and lane is unambiguous. Existing 10 shapes should
   produce the same layout (with lane assignments now being explicit
   but trivially satisfiable). Validates the variable refactor.
   ~2 days.

2. **Phase 2: sideload semantics + lane-walker cross-check.** Add the
   `(receiver_dir, feeder_dir) → lane` table, drop the at-most-one-
   route-per-tile constraint. Add the lane-walker assertion on placed
   templates. At this point `(3, m)` should solve correctly (if there's
   a feasible layout) or come back UNSAT (no feasible layout in the
   given grid). For shapes that come back UNSAT, widen the grid and
   retry — the placer should find a valid layout with a lane-balancer
   splitter inserted along the over-saturated path.
   ~3-4 days.

3. **Phase 3: splitter lane re-distribution + coprime shapes.** Once
   sideloads are safe, the natural fix for over-saturated paths is to
   route through a splitter. Phase 3 verifies this works — declares
   `(1, 5), (1, 6), (1, 7), (1, 9), (1, 10)` and all `(2, m)`
   coprimes, then `(3, m)` and `(4, m)` for dyadic m. The router should
   handle these by inserting balancer splitters along feedback channels
   when the per-lane caps would otherwise be exceeded.
   ~3-4 days.

4. **Phase 4: full coverage.** All `(n, m)` in `1..=10 × 1..=10`. Issue
   [#136] closes. ~2 days, mostly testing.

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
- *2026-05-02 — `(1, 5)` deferral resolved by a different track:
  `rfp-lane-safe-synth.md`. Rather than continue wrestling with
  sideloading in the placer, push the merging upstream into synth via
  cascading balancer splitters. The placer never sees multi-arc port
  relaxation under that approach; phase 2 of that RFP rolls back the
  sideload table and per-lane caps as obsolete. Splitter direction
  support, dual-lane input modeling, and rate-aware encoding all
  survive (still useful as foundational pieces).*
