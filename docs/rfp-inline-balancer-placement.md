# RFP: Inline balancer placement

## Summary

Stop reserving a full-width Y band per `LaneFamily` for the balancer. Instead,
stamp the balancer template into the trunk corridor at its minimum Y extent
and let other trunks route around it via the existing ghost router + junction
SAT. The output merger gets the same treatment at the south end of the
layout. Expected win on `electronic-circuit @ 30/s, am2, ftb, s=pd`: roughly
30–50% Y compression, eliminating the visibly-empty bands to the east of each
balancer cluster.

## Motivation

[`/#/l/ecl/30/am2/_/ftb?s=pd`](https://storkme.github.io/spaghettio/#/l/ecl/30/am2/_/ftb?s=pd)
shows three obvious horizontal bands where a tall, narrow balancer hugs the
left trunk and the rest of the band's lane width is empty whitespace. The
producer-to-trunk balancer is ~3 splitters wide in X, but the band reservation
spans the full bus width; everything east of the balancer is dead Y for that
row. Three such bands in this one layout. The output merger at the south end
has the same shape.

This is the dominant source of vertical waste on every non-trivial recipe we
ship today. It compounds with `(m, k·m)` balancer shapes that are tall by
construction and with deep recipe chains where each `LaneFamily` adds a band.

## Design

Four changes, in pipeline order:

1. **Shrink the Y reservation** in
   [`bus/lane_planner.rs`](../crates/core/src/bus/lane_planner.rs). Today
   `plan_bus_lanes` reserves `template.height` rows spanning the full lane
   width per family. Replace with a reservation that spans only the
   balancer's footprint (its `width` columns at its chosen X). Other trunks
   in the same Y range remain free to route through the corridor as long as
   they don't intersect the footprint.

2. **Mark the footprint as `Occupancy::Template`** in
   [`bus/ghost_occupancy.rs`](../crates/core/src/bus/ghost_occupancy.rs).
   The typed-occupancy framework already distinguishes `Template` from
   `HardObstacle`; this RFP just feeds balancer rects into it.
   [`bus/ghost_router.rs`](../crates/core/src/bus/ghost_router.rs) already
   treats `Template` as impassable for negotiated-congestion A*, so trunk
   detours fall out for free at the A* level.

3. **Extend the junction SAT with a no-UG-endpoint exclusion rect** in
   [`bus/junction_sat_strategy.rs`](../crates/core/src/bus/junction_sat_strategy.rs)
   / [`sat.rs`](../crates/core/src/sat.rs). Today UG pair tiles are picked
   freely from the crossing zone; we add a list of forbidden rects per
   zone and forbid any UG endpoint variable from landing inside them. Pad
   each rect by 1 tile around the balancer footprint so UG endpoints don't
   kiss the splitter input/output tiles (which produces lane-direction
   confusion in the validator).

4. **Output merger applies the same machinery** at the south end of the
   layout —
   [`bus/output_merger.rs`](../crates/core/src/bus/output_merger.rs)
   already runs after row placement; it just packs tighter against the last
   row instead of getting its own band.

### Fallback path

For shapes where the balancer-inside-corridor genuinely blocks all routing
options, fall back to the current full-width band with a
`BalancerInlineFallback { item, shape, reason }` trace event so we can
measure how often the inline mode fails. The full-width band stays in the
codebase as a deliberate second tier, not as legacy.

### What's NOT in scope

- Re-ordering balancers within the corridor for better packing (current
  Y assignment from `plan_bus_lanes` is reused as-is).
- Changing balancer template selection (which library entry / generator
  candidate to stamp). Same selection logic, just placed differently.
- Lane-rate validation changes. The walker in
  [`validate/belt_flow.rs`](../crates/core/src/validate/belt_flow.rs)
  shouldn't care that a UG bridge dodges a balancer rather than running
  in clear space.

## Kill criteria

This RFP needs more validation than I've done — the kill criteria are
deliberately measurable so whoever picks it up can confirm or fold quickly.

- **K1.** If `BalancerInlineFallback` fires on more than 30% of the e2e
  test corpus, inline mode can't be the default — keep the full-width band
  as the default and only inline when it's clearly free. (Acceptable
  outcome: opportunistic-only inline; not what this RFP scopes for.)
- **K2.** If the SAT exclusion-rect constraint requires more than ~200 LOC
  of new clause-generation code in `junction_sat_strategy.rs`, the
  per-zone list shape is wrong and we should consider extending
  `Occupancy::Template` to be SAT-visible directly instead.
- **K3.** If end-to-end runtime regresses by >2× on the existing e2e
  corpus, drop this even if Y compression looks good. The router runs once
  per layout per browser interaction; that budget is real.
- **K4.** If `tier2_electronic_circuit_from_ore` (the test that matches
  the motivating screenshot's recipe) doesn't visibly compress in the
  browser after Phase 1, the design didn't actually attack the problem
  it claims to attack — even if e2e validators stay green.

## Verification plan

Per the layout-engine verification protocol in
[`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. Full e2e suite green —
   `cargo test --manifest-path crates/core/Cargo.toml`. All 9 non-ignored
   tests stay green.
2. Browser sanity check on the motivating URL above and at least
   `processing-unit @ 1/s` (deeper recipe chain, more `LaneFamily`s
   stacked) and any layout where the existing `(9, 9)` passthrough lands.
3. Snapshot diff with `SPAGHETTIO_DUMP_SNAPSHOTS=1`. Compare entity counts
   and Y-extent before/after on the e2e corpus. Compression target: >20%
   median Y reduction across the corpus, ideally >40% on layouts with
   multiple balancers.
4. New trace events: `BalancerInlineStamped { item, shape, footprint }`
   on success, `BalancerInlineFallback { item, shape, reason }` on
   fallback. Both feed the snapshot debugger. K1 measurement reads off
   their counts.
5. Validator regression watch: lane-rate, UG-pair, splitter, and inserter
   checks should all continue passing. A clean e2e run that produces a
   layout with disconnected belts is a *validator* bug, not a success —
   pull the snapshot up in the browser before declaring done (per the
   protocol).

## Phasing

Two landable chunks. Phase 1 is mostly mechanical; Phase 2 is where the
SAT work lives.

- **Phase 1 — output merger inline.** Terminal placement, no
  through-routing concerns. `Occupancy::Template` for the merger
  footprint; no junction SAT changes needed because the merger sits south
  of all junctions. Cheap win, ~100 LOC. Validates the typed-occupancy
  pipeline end-to-end before we touch SAT.
- **Phase 2 — producer-to-trunk inline.** Real change. Includes the
  `plan_bus_lanes` reservation shrink, the SAT exclusion rect, and the
  fallback-path trace events. Most of the kill criteria target this
  phase.

## Decision log

- *2026-05-01 — drafted, pending validation pass before acceptance.
  Author flagged "more validation to be done" in chat; specifically the
  Phase 2 router-failure-rate question (K1) and the SAT clause-cost
  question (K2) want a small spike before committing.*
