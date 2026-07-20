# RFP: Reserved power space (Phase 3 of the power arc)

*Draft revision 2 — rewritten after two-reviewer adversarial review (REVISE
×2), delta-verdict ACCEPT with residuals applied; see decision log. Companion
to `docs/rfp-power-supply.md`, whose Phase 3 this specifies. 0f is merged
(`debb398`, committed scoreboard goldens regenerated post-0f). Pending: user
acceptance + sequencing slot.*

## Summary

Phase 0f proved a hard geometric wall: in packed row stacks (7–8-row cycles,
zero inter-row gap), inserter rows sit at distance exactly 4 from every
physically possible pole position — outside the medium pole's ±3 supply — and
every tile in reach is occupied, so no placement heuristic, and not even a
post-hoc substation, can power them. Five gating fixtures pin **153**
honest-red uncovered-inserter warnings (plus two known-hard non-gating
cases). This RFP proposes **reactive, selective space reservation**: when a
completed layout's `place_poles` reports uncovered inserters, the pipeline
re-runs in full with **+1 row inserted at each flagged band** at the placer
level; the widened band is ordinary free space all the way through routing,
and **substations** (2×2, 18×18 supply — the only power entity whose reach
spans a packed cycle) are placed in it **after routing**, by the power placer,
exactly as poles are today. Poles and substations are never router obstacles.
Clean layouts pay zero footprint by construction.

## Motivation

- **The acceptance harness — five gating pins, 153 warnings** (all
  adjudicated hard: zero free tiles in each uncovered inserter's 7×7, exact +
  conservative footprints, verifier-confirmed):
  `tier2_electronic_circuit_20s_from_ore` ×14,
  `stress_electronic_circuit_60s_red_from_ore` ×60 (scoreboard, ceiling
  tightened to expose), `tier5_processing_unit_from_ore_am3` ×20, the PU
  **am2 baseline** ×43 (e2e.rs:3862 — labeled "the fifth Phase 3 substation
  fixture"), `tier_kovarex_self_loop` ×16. Known-hard **non-gating**,
  recorded separately: utility-science-pack@1 ×16 and chemical@10 ×23
  (verified on manual runs; not suite-gating).
- **The geometry** (verified tile maps, 0f record): cycle = band / 3 belt
  rows / inserter row / 3-row machines. Any insertable free row is ≥4 from
  the inserter row (inserting between belts and inserters breaks pick
  adjacency), so medium poles can never reach; only the substation's 18×18
  supply spans a packed cycle — and a substation's 2×2 footprint needs a
  2-row band that packed stacks don't have. Hence reservation, not placement.
- **Interconnect is a sizing constraint, not a discovery**: pole-to-pole wire
  reach is `min(both ends)` (validate/power.rs min-of-both; draftsman
  collection.py:1062) — substation↔substation 18, but
  **substation↔medium-pole 9**. Every substation band must either sit within
  9 tiles of the existing pole network or budget explicit connector poles as
  a stated cost, decided up front at design time.
- **Cost baseline** (landed committed-config table, post-0f): corpus poles
  1770→3464 (**+96%**); gear@5 5→11, EC@20 62→117, EC@35 119→232, EC@60-red
  179→357, AC-am2 82→162, PU-am3 181→343. Kovarex and PU-am2-baseline
  baselines come from the pending post-0f re-census; trigger (b) of the
  parent RFP re-anchors to that census. The two-band+mop-up covers
  everything shallow — reservations serve a small minority, which is why
  selectivity is a hard requirement, not an optimization.

## Design

**Position 1 — selectivity is reactive, not predictive.** A predictive
placer-time predicate would need template tile occupancy that doesn't exist
until templates stamp (cross-phase inference — the bug class this arc keeps
killing). Instead: run the pipeline; if `place_poles` ends with a non-empty
uncovered set (it computes exactly this), re-run with reservations derived
from that set. **Precedent: `run_layout_with_retry_inner` (layout.rs:159-231)**
— a full `layout_pass` re-run including `route_bus_ghost`, reactively
triggered off pass-1 data, with wholesale trace truncation
(`truncate_events`) for pass-1 events. NOT the cheap inner width-correction
pass (which never re-runs routing): the cost shape is a **full second
pipeline pass** for starved layouts, and the runtime kill criterion budgets
that reality. Selectivity is perfect by construction — the trigger is a
measured fact.

**Position 2 — shape: +1 row at each flagged band; substations in the
widened band.** Pass 2's placer inserts one row at each starved cycle
boundary (a `y_cursor` adjustment in `place_rows`, placer.rs:606+; rows
below shift down; templates untouched). The now-2-row band hosts substations
at intervals sized by the 18×18 supply (~18-tile horizontal coverage ≈ 6
machines per substation; the exact tile arithmetic is deliberately NOT
derived in prose — see the half-tile bugs below — it gets pinned by test
during 3a). Vertical supply reaches both adjacent cycles' inserter and
machine rows (distance ≤4–6); one starved boundary = one widened band (the
next boundary's inserter rows are out of reach). Rejected: vertical power
columns (break belt continuity / shift machine x mid-row), uniform power
rows (tax the clean majority), medium poles in a widened band (still ≥4 —
physically insufficient; precisely what makes this a substation design).

**Position 3 — power hardware stays post-routing; the reservation is just
space.** The pipeline is `place_rows → plan_bus_lanes → route_bus_ghost →
place_poles`; poles are **never** router obstacles (`route_bus_ghost`'s pole
param is fed `&[]` today and is listed for deletion in the parent RFP's 0f
follow-ups — 3a inherits that deletion rather than treating the vestigial
param as a designed seam; `Occupancy` is `pub(super)` with no
injection point — pre-routing pole occupancy was removed during 0f's own
review after it broke `census_logistic_science_pack` routing and created
pentapod's phantom bridge). This design conforms: the widened band enters
pass 2 as ordinary free rows — the router may route through it like any
free space — and the power placer claims what it needs **after** routing,
mop-up style, substations first (4 tiles each) then poles. No pre-routing
claims anywhere. If routing consumes so much of the band that a substation
no longer fits, that is a measurable pass-2 failure (see kill criteria),
not a reason to reintroduce pre-routing reservations.

**Position 4 — Phase 1 interplay: shared machinery is an option, not a
mandate.** Phase 1 (fluid-row pole gaps) may need nothing more than
stamp-time width guarantees in the fluid templates; whether any mechanism
from this phase is worth sharing is decided when Phase 1 is designed, on its
own evidence. (Rev 1 mandated a shared `PowerReservation` record —
retracted; it prejudged Phase 1 with a mechanism this revision has itself
retired.)

**Position 5 — acceptance = the five gating pins → 0, clean layouts
unmoved.** Done when 14/60/20/43/16 → 0 with reservation active (strict
pins flip to `[]`, EC@60-red's scoreboard returns to 0 power), the two
non-gating cases verified on manual runs, and non-starved layouts show
unchanged goldens under the real mechanisms (see Verification).

## Export/validator work surface (itemized — the generator has NEVER emitted a substation)

- `blueprint.rs:100-111` — export center math places a 2×2 at x+0.5; a
  substation needs x+1.0. **Latent half-tile bug #1** if the pole path is
  copy-pasted.
- `pole_center()` — its 2×2 comment is wrong even today. **Latent half-tile
  bug #2**; fix comment and math together, pinned by an export round-trip
  test.
- `validate/power.rs:113` — pole filter hardcodes `medium-electric-pole`;
  needs the substation arm.
- `wire_reach()` (power.rs:31-36) — no substation arm; must encode
  min-of-both interconnect (18/18 vs 18/9 pairs).
- `POLE_RANGE = 3` — flat radius in TWO places (`validate/power.rs:102` and
  the placer's own copy at `bus/layout.rs:863`); becomes per-pole-type
  supply in ONE shared home, per the arc's one-fact-one-place rule — fixing
  the validator and missing the placer (or vice versa) is the trap.
- `place_poles` — claims one tile per pole; a substation claims 4.
- `entity_size` — needs the substation 2×2 entry.

## Kill criteria

- **Convergence**: if pass 2's uncovered set is non-empty after ONE repair
  iteration on any fixture (the reservation created new starvation, or
  routing consumed the band past substation fit) — stop; the reactive
  design oscillates, and the predictive/uniform trade-off goes to the user.
- **Selectivity leak**: any currently-clean corpus layout's committed golden
  hash or pinned warning set moves — evaluated under the goldens' documented
  same-host/same-cache discipline (2 of 8 stress goldens are SAT-cache-
  sensitive per `crates/core/tests/goldens/stress/README.md`; a cache-state
  hash flap is not a trip) — stop; the trigger fired where it shouldn't.
- **Shape failure**: any of the five gating fixtures' pinned count fails to
  reach 0 under widen-plus-substation — stop and re-derive from that
  fixture's tile map before writing more code.
- **Movement budget** (enforced by mechanisms that exist): for starved
  layouts, every re-blessed golden must be explained in review as
  "y-translation below inserted rows + power entities only" and verified at
  tile level per the CLAUDE.md layout protocol; `assert_warnings_exactly`
  pins on non-power categories must be unchanged. Any unexplained
  non-power, non-translation delta — stop. (Analog of the belt-tier rule:
  reservations may cost space, never semantics — machine counts, belt
  routing, and recipes are invariant.)
- **Interconnect honesty**: connector poles bridging a substation band to
  the pole network are part of the stated cost table; if any fixture's
  total power-entity count exceeds 1.5× its post-0f pole count, pause and
  show the user before landing. Kovarex and PU-am2-baseline have no landed
  post-0f baselines yet — theirs come from the post-0f re-census, which is
  therefore an explicit 3a entry precondition for this criterion to be
  evaluable on all five fixtures.
- **New starved case**: if the reactive pass fires on any case outside the
  known seven (five gating + two non-gating) — expected to be possible; the
  known set grew mid-0f (kovarex was a new find) — pin it per the 0f
  protocol (hardness-scan, then assert) and report. More than one new case,
  or any new case NOT hard under the 0/49 scan → pause and re-present scope
  to the user.
- **Runtime**: the repair is a full second pipeline pass — ≤2× layout time
  for starved cases only; corpus wall-clock +≤10% (measured; the starved
  set is 5–7 of ~45).

## Verification plan

Per CLAUDE.md layout protocol, using mechanisms that exist: (1) the five
gating pins flip — strict `assert_warnings_exactly([(power, N)])` → `[]`,
EC@60-red scoreboard to 0 power; (2) golden coverage — the 8 committed
stress-fixture hashes (unchanged where non-starved; re-blessed with
per-case explanations where starved) plus `assert_golden_hash` cases, plus
the committed scoreboard goldens (PR #308, landed and regenerated post-0f
in the `debb398` integration);
(3) both-direction position-keyed warning diffs (the standing standard);
(4) gauntlet run once for the non-gating pair; (5) connectivity: one
powered network per case including substations, min-of-both wire reach
verified against game data via factorio-expert BEFORE implementation;
(6) user browser eyeball on PU-am3 and kovarex; (7) the ultimate anchor:
in-game import of one starved layout, carried as a user step alongside the
inserter-sizing RFP's open KC5 anchors. If review-level tile audits of
re-blessed goldens prove insufficient in practice, a translation-aware diff
helper is scoped as explicit 3a work — not assumed to exist.

## Phasing

- **3a** — reactive repair pass (placer-level band insertion + full re-run
  per the retry precedent) + post-routing substation placement + the full
  export/validator work surface above: clears EC@20, EC@60-red, PU-am3,
  PU-am2-baseline. Largest unit; team flow + user eyeball.
- **3b** — kovarex. The evidence leans SAME shape (identical 0/49-free
  hardness signature; nothing about recirculation changes the
  widen-and-substation move). What justifies a separate phase is only the
  **boundary computation**: the starved recirc rows sit at `top_y−1/−2`,
  above the row origin rather than between stacked cycles, so the band
  flagging/insertion site is the row's top edge with no "next cycle"
  neighbor. 3b verifies that boundary variant; it does not design a new
  shape unless the variant fails its fixture.
- **3c** — re-census, trigger-(b) re-anchor, close-out numbers into the
  parent RFP.

## Decision log

- *2026-07-19 — draft rev 1 written in parallel with the 0f merge review
  (lead-directed parallelization). Positions: reactive two-pass
  selectivity; widen-band + substations; pre-routing `Permanent`
  reservations; mandated shared Phase 1 mechanism; five-fixture acceptance.*
- *2026-07-19 — **adversarial review ×2: REVISE; rewritten as revision 2.**
  Corrections folded: (a) pre-routing reservation INVERTED — poles/
  substations are never router obstacles (that mechanism was removed during
  0f itself); reservation is placer-level free space, power placed
  post-routing; (b) precedent corrected to `run_layout_with_retry_inner`
  (full pipeline re-run incl. routing + trace truncation), not the inner
  width-correction pass — runtime budget now reflects a full second pass;
  (c) substation numbers corrected (18×18 supply; interconnect to medium
  poles capped at 9 by min-of-both — promoted to an up-front sizing
  constraint); (d) fixture set corrected to the five GATING pins totalling
  153 incl. PU-am2-baseline ×43 (e2e.rs:3862), with USP ×16 and chemical@10
  ×23 recorded as known-hard non-gating; (e) new itemized export/validator
  work surface — the generator has never emitted a substation; two latent
  half-tile bugs named; (f) cost baseline restricted to the landed
  committed-config table (+96% corpus); (g) shared Phase 1 mechanism
  demoted to an option decided at Phase 1 design time; (h) verification
  rewritten onto mechanisms that exist (golden re-bless + PR #308
  scoreboard goldens + warning pins), translation-diff helper scoped as
  optional 3a work; (i) kill criteria: movement budget tied to real
  enforcement, new-starved-case criterion added; (j) 3b engaged with the
  evidence — same shape, different boundary computation. Pending 0f merge
  outcome + user acceptance.*
- *2026-07-19 — **delta-verdict: ACCEPT (design layer)**, all ten directives
  verified landed with source spot-checks; feasibility findings confirmed
  absorbed. Four minor residuals applied in place by the lead: PR #308
  goldens cited as landed (regenerated post-0f in `debb398`); selectivity-
  leak KC scoped to the goldens' same-host/same-cache discipline; the
  interconnect-honesty KC's missing kovarex/PU-am2 baselines made an
  explicit 3a entry precondition (post-0f re-census, in flight); both
  `POLE_RANGE` sites named (validator + placer) under one-fact-one-place;
  3a inherits the vestigial router pole-param deletion. Document is
  acceptance-ready; committed to docs/ when Phase 3's sequencing slot
  arrives.*

## Decision log (Phase 3a-i)

- *2026-07-20 — **Phase 3a-i LANDED** (`534f6ad`; adversarial review APPROVE,
  one mandatory constraint carried to 3a-ii). Substation is now a first-class
  generator entity: `common::entity_size` (2×2) and `common::pole_supply_range`
  (medium 3, substation 9) are single sources both export and validators
  consume; the two latent half-tile bugs are fixed and pinned by a hand-placed
  round-trip test (bug #1 blueprint center x+0.5→x+1.0 via entity_size; bug #2
  pole_center size-aware + corrected comment). Geometry draftsman-grounded: 2×2,
  supply ±9 (18×18), wire 18, substation↔medium min(18,9)=9 — RFP numbers
  confirmed. Non-layout-moving, corpus byte-identical (STRESSGOLD 9/9).
  **CARRIED CONSTRAINT (review finding, corrects the implementer's
  "conservative" label): the integer ±9-from-center coverage test is
  DANGEROUS-direction for the even 2×2 footprint — it marks +x/+y edge index
  ex+10/ey+10 as covered when the game powers only through ex+9/ey+9, a +1-tile
  FALSE-ACCEPT (the 0f blind-spot class). Harmless in 3a-i (dead path), but
  3a-ii MUST (a) replace the integer-Chebyshev coverage with a
  continuous-coordinate check |subject_center − pole_center| ≤
  supply_area_distance (subject_center = index + size/2 — keeps medium exact,
  makes substation exact) BEFORE placing any substation, and (b) guarantee real
  coverage at placement independently, never leaning on the validator bound.**
  Non-blocking residual: export/parse footprint-table asymmetry for
  engine-never-emitted 2×2s (big-electric-pole, steel-furnace) — unreachable,
  one-line note if the vocabulary grows.*
