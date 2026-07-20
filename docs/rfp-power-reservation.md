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
- *2026-07-20 — **Phase 3a-ii LANDED** (`e2fb777`/`5e37826`/`867deae`/`3481841`/`50408a3`;
  full adversarial review APPROVE; geometry independently pre-verified by a
  separate fork). The reactive repair mechanism works exactly as the RFP
  designed — but the RFP's CENTRAL PREMISE is FALSIFIED and recorded: it
  assumed a 3-belt-row cycle and concluded "only a substation's 18×18 can
  reach these." The actual electronic-circuit dual-input rows have **2** input
  belt rows, so widening the starved boundary by +2 lands the freed band
  exactly 3 tiles above the deep inserters — inside a medium pole's ±3.
  Independently verified from real tile maps: **2,167 inserters across the
  four fixtures, zero beyond medium reach, ZERO substations placed**. All four
  gating pins flip to 0 (EC@20 14→0, EC@60-red 60→0, PU-am3 20→0,
  PU-am2-baseline 43→0), asserted EXACT — coverage is distance exactly 3,
  zero margin, so the pins are the drift lock (a future 3→4 breaks a pin
  loudly). Composition: PU-am2-baseline is the both-at-once case (junction
  retry + substation-band widen), folded via `.max()` into one pass-2 —
  neither preempts the other, junction baselines intact. Movement budget:
  machines/inserters invariant, only y-widening + power/belt entities (EC@60
  340→340 machines, +12 rows). Footprint deltas: EC@20 +2, PU-am3 +8,
  PU-am2 +10, EC@60-red +12 rows. Interconnect: all-medium, 0 connector
  poles, every fixture UNDER the 1.5× census-baseline ceiling. Selectivity:
  only the four fixtures moved; kovarex byte-identical (its recirc inserters
  have no stacked-cycle predecessor → `compute_substation_bands` skips them →
  correctly left to 3b). **Substation machinery KEPT** (reviewer
  recommendation) — hardness-gated fallback (fires only for inserters still
  0/49-free after widening), unit-tested with a control case, zero
  runtime cost, correct for genuinely deeper future geometry. Reviewer
  corrections carried: (1) the convergence guard is the four exact pins, NOT
  an explicit code check (the report mis-stated this; a genuinely-new future
  starved case outside the corpus would ship uncovered without an alarm —
  FOLLOWUP: make the reactive pass assert its own convergence); (2)
  zero-margin distance-3 warrants a comment near `compute_substation_bands`
  for template authors. Merged ff-only onto 1eddaee.*

- *2026-07-20 — **Phase 3b LANDED** (kovarex top-edge substation boundary
  variant). The RFP's 3b hypothesis is CONFIRMED shape, and — unlike all four
  3a-ii fixtures — this is the case where the dormant **substation path finally
  fires**. Real tile map (pass 1): the self-loop packs its 16 recirc input
  inserters at `top_y-1/-2` with **5 belt/corridor rows stacked ABOVE them**
  (far-corridor return, descent, far belt, near belt, near2 belt) — so the
  top-edge freed band lands 5+ rows up, beyond a medium pole's ±3; only the
  substation's ±9 reaches down. `compute_substation_bands` now flags the
  STARVED row's own top edge when it has no predecessor cycle (`target == 0`,
  `top_edge: true`) instead of skipping it; the widen is applied as a
  y-offset bump (`layout_pass`'s new `top_widen`, a distinct channel from the
  interior `extra_gap_after_row` map) and the target-resolution powers the
  row's own input-inserter band (deeper than the interior `y_start..+4`
  window). Result: pin `[("power", 16)] → []`, **+2 rows** (row 0
  `y_start 1→3`, layout `29x15 → 29x17`), **exactly 1 substation** at (11,1)
  covering the recirc bank under the exact continuous ±9 check, medium poles
  5→6 (one connectivity bridge), network one connected component, 0 footprint
  overlaps. **Self-loop composition intact**: `assert_no_errors` +
  `assert_round_trip` green, 6 centrifuges invariant, no belt-dead-end /
  unresolved-junction introduced, and the substation landed IN the freed band
  (routing did not consume it — the KC risk did not trip). Selectivity:
  STRESSGOLD `=check` 8/8 byte-identical, four 3a-ii pins still 0, full suite
  green (lib 695, e2e 50), clippy lib clean, wasm target check clean. New pin
  asserts `substation_count == 1` so a future geometry change that re-routes
  coverage through a different (or absent) power entity fails loudly.*

- *2026-07-20 — **Phase 3c LANDED — arc close-out** (re-census + trigger-(b)
  re-anchor + the two review followups). **Re-census** (post-3b, `ca8730e`, 49
  snapshots — the 45-case post-0f corpus + the four `phase0e1_*` fixtures that
  landed with 0e-i after that census; raw per-case census AND the parts A–E
  summary are now BOTH reproducible from committed code —
  `scripts/pole_census.py` + `scripts/pole_census_analysis.py`, data at
  `scripts/pole-census-2026-07-20-post3b.json`, closing the post-0f census's
  uncommitted-analysis gap):
  - **(A) ZERO power warnings/errors corpus-wide.** All five gating pins
    (EC@20 14, EC@60-red 60, PU-am3 20, PU-am2-baseline 43, kovarex 16) AND the
    known-hard non-gating `census_utility_science_pack` (16) now read 0. The arc
    opened with 153 gating + 16 non-gating honest-red uncovered-inserter
    warnings; it closes at **0 pinned honest-red power warnings**.
  - **(B) trigger (a) UNCHANGED — no state change.** Solid-row zero-local-slack
    medium poles: **18** (post-0f: 18), same advanced-circuit / deep-AM
    south-band character (15 advanced-circuit + 3 deep science-pack rows; the
    exact pole set shifted 4 members from unrelated layout movement between the
    two censuses, count identical). Trigger (a) was already TRUE post-0f with
    Phase 3 already activated — still TRUE, still activated, reinforcing
    evidence not a new trip. Fluid zero-slack 108 and solid/fluid medians 4/2 all
    unchanged from post-0f.
  - **(C) trigger (b) baseline re-anchored to 4251** (the new 49-case corpus
    total; supersedes the post-0f 45-case anchor of 4226). Matched-45 growth is
    **+9 poles (+0.21%)**, and every one is a Phase-3 case: EC@20 +1,
    EC@60-red +4, PU-am2-baseline +1, kovarex +2 (+1 medium +1 substation),
    USP +1 (substation); PU-am3 net 0 (its widening repositioned poles without
    net-adding). **All other 40 matched cases: ZERO delta** — perfect
    selectivity, exactly as designed, and nowhere near the >20% trip threshold.
    The four new `phase0e1_*` fixtures add +16 (2/8/3/3), giving the 49-case
    4251.
  - **Substations counted for the first time.** `pole_census.py` counted only
    `medium-electric-pole`; extended to count `substation` as a distinct pole
    TYPE — part-C totals include it (`real_pole_count = medium + substation`),
    the part-B ±3 slack analysis excludes it (a ±9-supply / 2×2 substation has no
    meaningful single-tile slack window), and it is modelled as a 2×2 slack
    obstacle for neighbouring mediums. **Two corpus substations:**
    `tier_kovarex_self_loop` (top-edge band; 5→7 = +1 medium +1 substation) and
    `census_utility_science_pack` (the deep-geometry FALLBACK — 278 medium
    unchanged, +1 substation, which cleared its old 16 warnings). The substation
    machinery is therefore exercised by **two** real corpus cases, not just
    kovarex.
  - **Substation-necessity premise — final tally.** The RFP's central premise
    ("only a substation's 18×18 can reach a packed cycle") is FALSIFIED for the
    four interior gating fixtures (medium poles suffice after the +2 widen —
    the dual-input row has 2 belt rows, not the assumed 3) but VINDICATED for the
    two genuinely-deep cases: kovarex's 5-row-deep top-edge recirc and USP's
    deep-geometry fallback both need the ±9 substation. Not dead code; not
    over-built either — it fires exactly where the geometry demands it.
  **Convergence assertion** (review followup, real code): 3a-ii's reactive pass
  discarded pass-2's uncovered set (`let (result_2, _, _, _)`), leaving the
  per-fixture pins as the ONLY convergence guard — a genuinely-new starved case
  outside the corpus would ship uncovered with no alarm. Now the pass captures
  `uncovered_2` and, when non-empty, emits a `ReactivePassNotConverged` trace
  event (lands in snapshots / can drive a scoreboard) plus an env-gated
  (`SPAGHETTIO_WARN_ON_STDERR`) eprintln. Deliberately NOT a `debug_assert` —
  release builds skip those and would ship the break silently, exactly the hole
  the review flagged. The block is skipped on the converging path (every corpus
  case reaches zero uncovered), so it adds zero entities and the corpus stays
  byte-identical. **Zero-margin comment** (review followup): a warning at
  `SUBSTATION_BAND_TILES` records that the +2 interior widen clears the four
  gating fixtures via medium poles at distance EXACTLY 3 (zero margin) — a
  template author adding a belt row or shifting an inserter one tile deeper tips
  3→4 and re-uncovers (4 is outside medium ±3), caught only by the four exact
  pins plus the substation fallback; the constant must be re-derived, not raised
  blindly, if a dual-input template's belt-row count changes. **Gates:**
  STRESSGOLD `=check` byte-identical (8/8 — no fixture moved), full suite green
  (lib + e2e 50), clippy lib clean, wasm target check clean.*

- *2026-07-20 — **chemical@10 accounting closed (arc review).** The reservation
  RFP promised "the gauntlet is run once for the non-gating pair," but the 3c
  close-out accounted only USP — chemical-science-pack @ 10/s, the *other*
  non-gating wall case, never made it onto the record. The arc reviewer ran it;
  this unit re-ran it independently to confirm: **0 power-category issues** —
  15777 entities, **998 poles (0 substations)**, the reactive band-widening
  pass fired **once** (a single `LayoutRetried`) and **converged** (no
  `ReactivePassNotConverged`; zero uncovered inserters), the widening alone
  landing every freed inserter row within a medium pole's ±3 (no substation
  needed). The layout carries **6 pre-existing non-power errors** (belt-loop,
  lane-throughput, underground-belt, unresolved-junction) that are **out of
  this arc's scope** — routing / junction quality on a 10/s chemical wall, not
  power. So the power arc is clean on **both** non-gating wall cases, not just
  the one the 3c close-out named.*

## Phase 3 complete
All of Phase 3 (3a-i, 3a-ii, 3b, 3c) is landed. Reactive band-widening clears
all five hard cases — EC@20, EC@60-red, PU-am3, PU-am2-baseline via medium poles
(the substation premise falsified there), kovarex via the substation fallback
(premise vindicated) — plus the non-gating USP via a second substation. The
substation machinery is exercised, not speculative; and both 3c review followups
(convergence assertion + zero-margin comment) are in. Trigger (a) is unchanged
(18 solid zero-slack, already-activated) and trigger (b) is re-anchored to the
4251-pole post-3b baseline.

**Scope of "zero power warnings" (clarified 2026-07-20, arc review).** The
"zero" is **power-category only** and covers two named scopes: the **census
corpus** (the 45/49-case snapshot set) and the **pinned e2e fixtures** (the
five gating pins + the non-gating `census_utility_science_pack`) all read 0
power warnings/errors; the **gauntlet non-gating pair** — USP @ 1/s and
chemical @ 10/s — likewise reads **0 power-category issues**, but those gauntlet
layouts still carry *non-power* errors (chemical@10: 6 — belt-loop /
lane-throughput / underground-belt / unresolved-junction) that are out of this
arc's scope. It is NOT a claim that the whole corpus validates clean.

**Remaining after the 2026-07-20 arc review.** The earlier "no remaining Phase
3 followups" was wrong; the independent arc review found live gaps. The three
code defects it raised are **fixed** (F1 overlap footprint — `d5efe0b`
"footprint-check multi-tile non-machines in overlap check"; F2 pole-repair
metric ↔ emitted wires + F3 dead router pole param — `0398659` "align
pole-repair metric with emitted wires; drop dead router pole param"). What **still remains**, none of
it landed here:
- **Kovarex's medium-pole count is unpinned.** The fixture pins
  `substation_count == 1` and warnings-exactly-empty, but not the 6 medium
  poles — the repair topology the F2 rewrite touched. A cheap
  `assert_eq!(medium_count, 6)` beside the substation pin would lock it
  (closure-check note, 2026-07-20); smaller cousin of the USP gap below.
- **F4 — the footprint-table quartet is not unified.** `check_power_coverage`
  (power.rs:62) hardcodes the supply-source filter to `medium-electric-pole ||
  substation` (2 types), disagreeing with `power_wires::is_pole` (4 types:
  medium / small / substation / big); and `common::supply_area_distance` falls
  back to the medium value (3.5) for any non-substation, so a small pole (real
  2.5) or big pole (real 2.0) would be given the wrong coverage radius AND be
  skipped as a source entirely. `entity_size`, `supply_area_distance`,
  `wire_reach`/`is_pole` should be unified behind one pole-attribute table with
  a drift test, the way Phase 3a-i unified the two `POLE_RANGE = 3` constants.
  Latent today (the generator only ever places medium + substation) but a trap
  for any future pole tier.
- **F5 — reactive-pass / census edge cases (not fixed here):** (a) the
  `give_up` set ignores substation coverage of *non-target* inserters — a
  substation placed for one band may already cover a neighbouring band's
  inserter, but the give-up accounting doesn't credit it; (b) **machine**
  starvation does not trigger the reactive pass (only uncovered *inserters*
  do), so a genuinely uncovered machine would not widen; (c) the `PoleSlack`
  trace's substation window disagrees with `pole_census.py`'s substation
  handling (the census excludes the 2×2 substation from the ±3 medium-slack
  analysis; the instrument's window does not match by construction); (d)
  consider promoting `ReactivePassNotConverged` from a trace event to a
  first-class `ValidationIssue` so non-convergence surfaces in the normal
  validator output, not only in snapshots.
- **USP's substation path is ignored-test-only.** The deep-geometry USP
  fallback substation is exercised only by the `#[ignore]`d
  `census_utility_science_pack` gauntlet case, not a promoted e2e regression
  test — a future geometry change that silently re-routes its coverage would
  not fail the default suite (only kovarex's substation is pinned by a
  non-ignored test).

**Open (out of Phase 3 scope):** the arc's ultimate anchor — in-game import of
one starved layout — remains a user step, carried alongside the inserter-sizing
RFP's KC5 anchors; validator-verified only until then. **THE WIRES ARC** (the
export encoded no pole copper wires until `a7d9a48`; every prior export pasted
power-dead) is the strongest evidence yet that this anchor is not optional —
recorded in full in `docs/rfp-power-supply.md`'s decision log (2026-07-20).
