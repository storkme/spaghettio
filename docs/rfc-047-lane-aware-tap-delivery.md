# RFC-047: Lane-aware tap delivery (stacked rate ceilings)

Registry: [`rfcs.md`](rfcs.md). Status: **Draft** (2026-07-21).

## Summary

Make trunk→row and row→trunk delivery **lane-aware**, so the lane
planner's full-belt thresholds stop assuming both lanes fill when the
geometry only fills one. This is the unlock RFC-046 deliberately
descoped: with delivery lane-accounted, the consumer-clamped fan-in
wall ([#312](https://github.com/storkme/spaghettio/issues/312)) can
scale honestly — at S=1 by using real two-lane tap forms where they
exist, and at S>1 by the belt-stacking factor — turning belt stacking
from "cheaper belts at the same rate" into "higher rate ceilings."
Riders: the walker-overshoot residual at zero-headroom tier boundaries
(fixtures land here), a kill-criterion-bounded look at the
legendary-express junction failure, and the per-lane stackedness
infrastructure RFC-046 deferred.

## Motivation

RFC-046's Phase 2 differentials falsified full-belt ×S on tap-delivered
flow: sideloads fill one lane (B8), inserter drops fill the near lane
(I5), so a trunk or row-input belt fed that way carries everything on
one physical lane. The S=1 fan-in wall had been *accidentally
shielding* the gap by refusing such configs before they laid out;
scaling it ×S exposed walker-caught single-lane overloads (18/s on a
15/s stacked yellow lane — probe-verified, see RFC-046's decision
log). RFC-046 therefore froze trunk-count geometry at S=1 and demoted
the ceiling lift here.

Concrete failing cases today:

- `stacking_fanin_wall_conservative_parity_ec6_yellow_legendary`
  (e2e): EC@6/s legendary yellow refuses at S=1 **and** S=2 — the
  fixture's own doc comment says to flip it to a differential success
  when this RFC lands.
- EC@60/s legendary express S=2: junction-solver failure near a dense
  crossing plus ~3% walker overshoot on zero-headroom lanes at exact
  tier boundaries (characterized in RFC-046's decision log; no issue
  number — this RFC files one if the junction half survives its
  bounded investigation as a real, separable defect). **Probe evidence
  (2026-07-21, `debug_overshoot_probe`): the junction failure is the
  dominant defect, not a peer** — the unresolved 50-tile crossing at
  (2,38) orphans ghost belts, every furnace bank in two full rows
  reports `belt-flow-reachability` unreached, and ore input belts
  deliver 0.0/s. The 15.4–15.5/s lane overloads (vs a 15/s stacked
  yellow per-lane cap, tiles (24–25, 14/21/118)) are secondary. Kill 3
  (junction) therefore gates the express variant; kill 4 (overshoot)
  is a smaller residual on the same config.
- More broadly: at high build quality, collapsed machine counts shrink
  consumer trunk counts while flows stay constant, so the wall bites
  configs that plain-quality builds handle (#312) — and stacking,
  which physically multiplies belt capacity ×4, currently cannot lift
  it at all.

## Ground truth (delivery geometry today)

*To be filled from the tap-geometry recon before the spec review:
which tap forms exist (single-sideload, dual-sideload, splitter-tap,
priority-splitter merge-tap), which fill both lanes, what the
`full_belt_cap` assumption in the fan-in wall actually corresponds to
geometrically, how row outputs join trunks, and what the walker's
lane-attribution model already knows. Every Design decision below must
cite this section.*

## Design

*Pending ground truth. Expected shape (to be confirmed/refuted by the
recon): a per-delivery-edge lane-capacity model — each tap/feed edge
declares how many lanes it can fill (1 or 2) from its geometry class;
the fan-in wall and K-trunk retirement sum per-edge deliverable rate
(`lanes × lane_capacity_stacked(tier, effective_S)`) instead of
assuming `full_belt_cap` per trunk; planning prefers (or synthesizes)
two-lane tap forms where a one-lane form would bind. Per-lane
stackedness (RFC-046 Phase 3 rider) reuses the same edge model with
per-item effective S.*

## Kill criteria

1. **Current-behavior identity.** With no configuration change (S=1,
   existing corpus), layouts are bit-identical: full suite green,
   STRESSGOLD `check` 9/9, zero golden re-blesses — *unless* a
   documented fixture flip is the point (the parity fixture, and any
   fixture this RFC's honest accounting proves was passing on a
   fiction). Any other S=1 diff is a threading bug — stop and fix.
2. **No credit without geometry.** Every lane-capacity credit above
   one lane must be justified by a delivery edge whose geometry class
   provably fills both lanes (cited to the ground-truth section) and
   must be visible to the walker's lane attribution. If the only way a
   fixture passes is a credit the walker cannot re-derive, the design
   is the #311 trap again — rework, do not ship.
3. **Junction investigation bound.** The legendary-express junction
   failure gets a time-boxed, snapshot-driven characterization (repro,
   trace events, zone identification). If the fix is not localized —
   if it requires junction-solver framework changes beyond a bounded
   strategy/cost fix — file the issue with the characterization and
   descope it from this RFC; do not redesign the junction solver here
   (three RFCs burned that way pre-`prune_dangling`; check
   `sol.entities` vs raw SAT output FIRST).
4. **Overshoot honesty.** The ~3% walker overshoot at exact tier
   boundaries must be root-caused (walker modeling artifact vs real
   physics) before any fixture is "fixed" by adding headroom margins.
   A margin without a root cause is evidence-overrun — the dominant
   failure shape this repo's process exists to prevent.
5. **The lift is real.** Close-out requires the parity fixture flipped
   to a differential success (EC@6/s legendary yellow: refuses at S=1
   for honest geometric reasons or lays out clean; lays out clean at
   S=2) with the same per-tile physical audit discipline as RFC-046's
   headline — and at least one fixture where the *rate ceiling*
   (not just belt tier) demonstrably rises with S.

## Verification plan

- Full suite single-run counts + STRESSGOLD at every phase gate.
- Fixture flips: the fan-in parity fixture → differential success;
  a ceiling fixture (rate that refuses at S=1, lays out at S=2, with
  per-tile stacked-capacity audit).
- Walker cross-check: lane attribution re-derives every two-lane
  credit (kill 2).
- Browser eyeball (user) on the flipped configs.

## Phasing

- **Phase 0 — ground truth + edge model spec.** Fold the recon into
  the Ground-truth section; classify every existing delivery form by
  lanes-filled; adversarial spec review of the completed Design.
- **Phase 1 — edge model, planner-side.** Per-edge deliverable-rate
  accounting behind the existing thresholds; identity gate (kill 1).
- **Phase 2 — two-lane tap forms + the lift.** Prefer/synthesize
  two-lane forms where one-lane binds; scale the wall by summed edge
  capacity; fixture flips (kill 5).
- **Phase 3 — riders.** Overshoot root-cause + fixtures (kill 4);
  junction characterization (kill 3); per-lane stackedness on the edge
  model.

## Decision log

- **2026-07-21 — RFC drafted (skeleton).** Number claimed as RFC-047
  per registry. Ground-truth and Design sections deliberately held
  open for the tap-geometry recon — writing geometry claims from
  memory is how RFC-046's spec review found a falsified census; this
  RFC starts from cited geometry instead.
