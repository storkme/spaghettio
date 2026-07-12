# RFP: Inserter sizing — matching machine-side throughput to planned rates

## Summary

Every row template places exactly one regular inserter (~0.84/s) per
machine side, while the solver plans machines at per-side rates up to
several items/s — so machines are inserter-bound engine-wide, capped
at a fraction of their planned utilization in a real game. This was
invisible until the `inserter-throughput` check landed
(`rfp-lane-demand-flow.md` Phase 1); the 1/s gauntlet now carries
2/8/11/22/42/55 honest warnings across the six packs. This RFP sizes
inserters to rates via a shared per-side ladder — in-place tier
upgrades first (regular → fast → **stack**), extra inserters where
the face's real free-column budget allows — plus **ingredient-to-belt
assignment** (the hungry ingredient goes to the near belt, where the
full ladder applies). The maximum inserter tier is a **user-facing
layout-engine parameter** (`max_inserter_tier`, default Stack),
mirroring `max_belt_tier`: a hard cap the ladder never exceeds, with
below-cap sides degrading best-effort + honestly warned. This is v2:
the v1 design (fast-only ladder) was killed by its own Phase 0 census
— kill criterion 1 fired at 19.6% of sides over ceiling and 4 of 6
packs unable to reach zero (decision log, 2026-07-12).

## Motivation

Reproducible today (`science_gauntlet`, HEAD ≥ `6849935`): all six
Nauvis packs at 1/s warn on `inserter-throughput`. Canonical case —
the logistic-science chain's iron-gear-wheel machine (AM2, 1.5
crafts/s planned): 3.0 plates/s in, 1.5 gears/s out, one regular
inserter (~0.84/s) each side → input caps the machine at ~28%.

The v1 census (593 sides, reconciled 21/21 against actual warning
counts) proved fast-only insufficient and localized the failure to
two walls:

1. **dual_input_row input ceiling ≈ 4.71/s** (fast + long-handed +
   one contested column) vs electronic-circuit-shaped demands of
   5.7–9.6/s per machine — and EC is in nearly every chain.
2. **single_input_row bridge-anchor output ceiling = 2.31/s** (the
   sideload bridge owns all extra columns) vs copper-cable outputs
   of 3.0–5.0/s.

Both walls are reach-1 sides. A stack-inserter rung (~12/s at zero
research) dissolves them in place; the far-belt (reach-2) sides that
can't take stack inserters (no long-handed stack exists) are handled
by assigning the hungry ingredient to the near belt instead.

## Design

### The real free-column budget (v1 census table, retained)

| Template / position | Extra input cols | Extra output cols |
|---|---|---|
| single_input_row, interior machine | 2 (mx, mx+2) | 2 |
| single_input_row, last-in-row | 1 (belt tail-trim removes mx+2) | 1 (west-flow; east-flow keeps 2) |
| single_input_row, bridge-anchor (lane_split) | per input | **0** (bridge owns mx+1..mx+3) |
| single_input_row, bridge-anchor's successor | per input | 1 (own mx is the bridge's 3rd tile) |
| single_input_row + secondary_output (D2b) | — | 1 (mx+2 is the LHI's) |
| dual_input_row | 1, CONTESTED between far/near ingredients | 2 interior (output row is single_input_row-shaped) |
| triple_input_row | 1 contested (far/near pair) + 1 for input3 (bridge-eaten when split) | 1 (second inserter shares the row) |
| quad_input_row | **0** for 3 of 4 inputs (north rows fully packed) | 1 |
| fluid_input_row (solid side) | 1 (fluid UG pipe owns a column) | 2 interior (output row is single_input_row-shaped) |

**v2 census debts (from the v1 census gaps, must close in Phase 0v2):**
`self_loop_row`, `voider_row`, and `scrap_recycling_row` have no
entries — their fixture predictions were untrusted floors; audit
their geometry from source. The dual/triple/quad position
sub-budgets that v1 extrapolated get verified against `templates.rs`
before the v2 census locks its ceilings.

### The ladder (v2)

Shared helper, one constants table with the validator's
`check_inserter_throughput` (I8 values, no-capacity-research; a unit
test fails if fix and check diverge):

```
size_side(rate, reach, position_budget, max_tier) -> SidePlan
// SidePlan = { picks: Vec<(dx, entity)>, shortfall: Option<f64> }
```

- Rung 0: keep 1 regular (rate ≤ 0.84).
- Rung 1: in-place swap, **cheapest sufficient tier**: fast
  (≤2.31/s) then stack (≤~12/s). Zero columns. The
  cheapest-sufficient invariant is unit-tested — stack appears ONLY
  where fast cannot cover, never as a default aesthetic.
- Rung 2+: add inserters into the position's actual free columns,
  again cheapest-sufficient per pick.
- Reach-2 sides: long-handed only (1.2/s per column, count-ladder) —
  no fast/stack long-handed exists. This is now a *minimized* path:
  ingredient assignment (below) keeps hungry ingredients off far
  belts wherever the recipe shape allows.
- Beyond budget or above `max_inserter_tier`: **best-effort
  placement + `shortfall`** — best achievable config placed, the
  inserter-throughput warning remains (honesty lives in the check),
  an `InserterSideCapped` trace event records the cap. Layouts never
  fail from sizing.
- **Contested columns**: larger relative shortfall wins; tie breaks
  to the far/reach-2 side. Pure function of the two rates,
  unit-tested, deterministic.

### Ingredient-to-belt assignment (v2, lever b)

`dual_input_row` (and the far/near pair in `triple_input_row`)
currently assign ingredients to near/far belts by recipe order —
arbitrary (verified: item-agnostic inserter placement; item0/item1
derive from `solid_inputs` order at placer.rs:1027-1047, a clean
local reorder). v2 assigns the ingredient with the higher per-machine
rate to the NEAR (reach-1) belt, where the full ladder applies; ties
preserve the current order (golden-churn minimization). Deterministic,
rate-derived, unit-tested. This is a real template behavior change
that moves goldens on dual/triple-input rows by itself — it lands
inside the same phase as those templates' ladder work so each fixture
re-blesses once per phase, not twice.

**Per-item attribution check (v2 delta-review blocker, resolved
here).** `check_inserter_throughput` is item-blind: it aggregates a
machine's input inserters into one side-wide total and never reads
which item each inserter carries. Pre-v2 that was harmless; post-v2,
assignment is load-bearing — a template bug putting the hungry
ingredient on the far belt would validate CLEAN on aggregate while
starving in-game (total avail unchanged, per-item delivery broken).
Resolution: a NEW companion check (per machine, per solid input
item: Σ throughput of the inserters attributed to that item ≥ that
item's utilization-scaled rate), keyed on inserter `carries`
attribution. It lands in its own commit BEFORE Phase 2's template
commit — thereafter kill criterion 3's no-co-edit rule covers it
like the aggregate check. Phase 0v2 audits that templates reliably
set `carries` on input inserters (prerequisite for attribution); if
they don't, that plumbing is Phase 1 scope.

### `max_inserter_tier` — user-facing engine parameter

`LayoutOptions.max_inserter_tier: InserterTier { Regular, Fast,
Stack }`, default `Stack`. Semantics mirror `max_belt_tier`: a hard
constraint the ladder never exceeds — capping at `Fast` or `Regular`
re-creates the v1 geometry limits and the affected sides degrade to
best-effort + warnings (the same machinery, no special mode).
Plumbed through wasm-bindings and the web UI (URL state) in the
final phase, alongside the existing belt-tier control.

### Integration

- Rates: `MachineSpec.inputs`/`outputs` are per-machine (verified);
  utilization scaling reproduces the validator's exact
  `count/ceil(count)` convention.
- Scope note: all ~10 row-template signatures gain per-side-rate
  parameters threaded from `build_one_row`; every placer call site
  updates. Wide but mechanical.
- Recyclers keep direct ejection (exempt in fix and check); sushi
  sort inserters already size by rate.
- Stack-inserter physicals: 1×1, reach 1, standard power draw —
  no entity-size or pole-coverage novelty; add to any hardcoded
  entity vocabularies that gate on inserter names (blueprint export
  already handles arbitrary inserter entities with filters).
- Power: added/upgraded inserters verified per phase via existing
  power checks.

### Composition gap (recorded, from v1 review; sharpened by v2 delta)

I8 constants are correct for belt-adjacent inserters under
saturation; the residual is that `check_inserter_throughput` assumes
the feeding belt sustains the demanded rate. The stack rung sharpens
this in a QUALITATIVE way, not just rate magnitude: the 2.0
`stack-inserter` prototype sets `wait_for_full_hand=true` (fast/
regular/bulk do not) — it waits to fill all 5 hand slots before
swinging (`grab_less_to_match_belt_stack=true` partially mitigates).
The ~12/s figure therefore assumes the feeding belt delivers items
DENSELY enough to fill a 5-item hand promptly — a burst-density
requirement, not just "12/s average". Kill criterion 5(b)'s
stack-upgraded in-game anchor explicitly watches for hand-fill
stalls. Mitigation otherwise unchanged: I8 doc note, demand-pull
walker, dedicated lane-vs-inserter cross-check on the validator
backlog (priority rises if Phase 0v2 shows stack sides near lane
capacity).

### Non-goals

- No belt-tier escalation (hard user constraint, unchanged).
- No template geometry changes — no pitch, belt, or machine moves.
- Bulk inserters: skipped (2.4/s base ≈ fast; adds nothing between
  fast and stack at zero research).
- Multi-input-row redesign: only if the v2 census still leaves
  gauntlet residue (not expected — both v1 walls are reach-1).
- The ~5 residual input-rate-delivery warnings from cyclic demand:
  separate (lane-demand-flow known limitation).

## Kill criteria

1. **Phase 0v2 coverage gate**: census re-run with v2 ceilings
   (stack rung + ingredient reassignment + audited
   self_loop/voider/scrap geometry + verified sub-budgets), same
   conditions: if >5% of warned sides exceed their v2 position
   ceiling, OR any of the six 1/s packs cannot reach zero, stop —
   v2 is also insufficient and the next step is template redesign,
   not another ladder tier.
2. **Geometry containment**: if `single_input_row` or
   `dual_input_row` need pitch/belt/machine-position changes to
   reach zero at 1/s, stop — under-scoped.
3. **Honest-zero gate**: no edits to `check_inserter_throughput` or
   the I8 constants in any commit touching templates.
4. **Budget gate**: entity-count increase ≤ +15% and zero area
   growth on every gauntlet fixture.
5. **In-game anchors (two)**: (a) the logistic gear machine after
   Phase 1; (b) one contested-column or stack-upgraded dual-input
   machine after Phase 2. Either still starving with warnings at
   zero = the CHECK is miscalibrated — stop everything.
6. **Cheapest-sufficient audit**: if any gauntlet layout places a
   stack inserter where the arithmetic shows fast sufficed, or any
   inserter above the user's `max_inserter_tier`, the ladder
   selection is buggy — stop and fix before proceeding (this guards
   the user-facing aesthetic/cost contract, not just correctness).
   The audit arithmetic is evaluated against the SAME frozen
   Phase 0v2 budget/ceiling table kill criterion 1 uses — a KC6
   audit may never re-derive budgets ad hoc (single source of
   truth, per the v1 review's discipline).

## Verification plan

Per the CLAUDE.md protocol, plus:

- Definition of done: `science_gauntlet` 1/s — all six packs at zero
  inserter-throughput warnings (Phase 0v2 must predict this
  reachable, else KC1 fired).
- `science_scaling_gauntlet` re-run; delta into the decision log.
- Golden re-bless per phase against Phase 0v2's frozen predictions;
  browser eyeball of tier1, tier2-from-ore, tier4-am2, one self-loop
  row (user validates).
- Power checks green per phase.
- Unit tests: ladder boundaries per tier, cheapest-sufficient
  invariant, max_inserter_tier capping (Fast-capped reproduces v1
  ceilings), reach-2 count-laddering, contested-column resolution
  (both orders + tie), ingredient-assignment rule (hungry→near, tie
  →stable), best-effort shortfall path, constants identity with the
  check, determinism of all choices.
- Netflow determinism sweep untouched.

## Phasing

- **Phase 0v2 — geometry audit + re-census**: close the v1 census
  debts (self_loop/voider/scrap rows — audit `self_loop_row` FIRST,
  it's structurally the most complex untabled template; extrapolated
  sub-budgets), audit inserter `carries` attribution reliability,
  re-run the census with v2 ceilings AND per-item decomposition
  (recipe-ratio split per side — the delta review found a
  zero-margin case: PU's electronic-circuit row puts iron-plate at
  exactly 2.40/s against a 2.40/s far ceiling after reassignment),
  flag every side within 10% of its ceiling as at-risk, evaluate
  kill criterion 1, freeze the per-phase per-fixture prediction
  table.
- **Phase 1 — ladder helper + `single_input_row`**: tier1 zero;
  mixed fixtures re-bless to predicted reduced counts; anchor (a).
- **Phase 2 — `dual_input_row` (ladder + ingredient assignment
  together) + remaining solid templates**: tier2 zero; anchor (b).
- **Phase 3 — reach-2 / fluid-adjacent rows + self-loop/voider/
  scrap rows**: long-handed ladders within audited budgets;
  enumerate any residue (expected ~zero per v2 design).
- **Phase 4 — close-out**: `max_inserter_tier` through
  wasm-bindings + web UI (URL state); both gauntlets re-run, deltas
  recorded; CLAUDE.md ladder refreshed; in-game anchors confirmed.

Every mixed fixture re-blesses at most once per phase, against
Phase 0v2's predictions; a blessing that misses its prediction is a
stop signal.

## Decision log

- *2026-07-12 — drafted (v1: fast-only ladder), immediately following
  lane-demand-flow Phase 1 (`6849935`): 1/s gauntlet carries 140
  inserter-throughput warnings across the six packs. Pending review.*
- *2026-07-12 — adversarial review round (v1). Verdict: REVISE.
  Three blocking findings incorporated: real per-position
  free-column census table (bridge-anchor output faces have ZERO
  free columns; quad north faces likewise; multi-input rows have one
  CONTESTED column) — in-place tier swap becomes the workhorse rung
  and beyond-budget sides are best-effort + honestly-warned; kill
  criterion 1 rewritten against per-position ceilings; phasing
  corrected (electronic-circuit is a dual-input row — tier2 clears
  in Phase 2) with per-phase expected counts frozen in Phase 0.
  Also: deterministic contested-column rule, power verification,
  second in-game anchor, composition-gap note. I8 constants verified
  (chest-to-chest holds for belt-adjacent inserters under
  saturation; no derating).*
- *2026-07-12 — re-review verification: SHIP after two census cells
  corrected (dual/fluid_input_row interior output budgets are 2, not
  1); bridge-anchor-successor position and east-flow note added.*
- *2026-07-12 — accepted by user. Phase 0 census started.*
- *2026-07-12 — **Phase 0 census complete: KC1 FIRED on both
  conditions — v1 design killed before any template code.** 593
  warned sides, reconciled 21/21 (regenerate:
  `cargo run --example census_inserter_sizing --release`). 116/593
  (19.6%) over ceiling vs 5% threshold; conservative zero-
  extrapolation recount 66/593 = 11.1%, still double. Only
  automation + logistic could reach zero. Walls: dual_input_row
  input ≈4.71/s vs EC-shaped 5.7–9.6/s; bridge-anchor output 2.31/s
  vs copper-cable 3.0–5.0/s. Histogram: 46% rung-1, 23% rung-2, 4%
  rung-3, 7% LHI-ladder, 20% exceed; 44 machines with genuinely
  contested columns. Census gaps: self_loop/voider/scrap rows
  untabled; dual/triple/quad sub-budgets partially extrapolated;
  contested credit optimistic (real capping higher). The v1
  per-phase prediction table is obsolete.*
- *2026-07-12 — **v2 redesign accepted by user** ("stack inserters
  are the way to go, but ideally the inserter level should be a
  parameter to the layout engine"): levers (a) stack-inserter rung
  (cheapest-sufficient, ~12/s reach-1, dissolves both v1 walls) +
  (b) hungry-ingredient-to-near-belt assignment (no fast/stack
  long-handed exists, so the far belt stays low-ceiling — keep
  hungry ingredients off it). New: `LayoutOptions.max_inserter_tier
  { Regular, Fast, Stack }`, default Stack, hard cap semantics
  mirroring `max_belt_tier`, plumbed to web UI in Phase 4. New kill
  criterion 6 (cheapest-sufficient audit — no stack where fast
  suffices). Phase 0v2 (geometry-debt audit + re-census) precedes
  any template code, same 5%/all-packs-zero gate. Pending delta
  re-review, then Phase 0v2.*
- *2026-07-12 — v2 delta review: REVISE with one blocker, all
  incorporated. (1) BLOCKER — `check_inserter_throughput` is
  ITEM-BLIND (aggregates per side, never reads inserter carries);
  v2 makes item identity load-bearing, so a hungry-ingredient
  misassignment would validate clean while starving in-game.
  Resolved: new per-item attribution companion check (lands in its
  own commit BEFORE Phase 2's templates; carries-attribution audit
  added to Phase 0v2). (2) Stack rung arithmetic CONFIRMED against
  draftsman prototype data: `stack_size_bonus=4` is on the entity,
  not research — 5 items/swing ≈ 12/s at zero research, ~20-25%
  headroom over the 9.6/s worst case. New: `wait_for_full_hand=true`
  on the stack prototype → burst-density requirement recorded in the
  composition-gap note, watched by kill criterion 5(b). (3)
  Reassignment geometry confirmed freely swappable (item-agnostic
  inserters, positional item0/item1). (4) Zero-margin case found in
  real census data: PU's EC row → iron-plate exactly 2.40/s vs
  2.40/s far ceiling post-reassignment — Phase 0v2 now decomposes
  per-item and flags all sides within 10% of ceiling. (5) KC6 pinned
  to the same frozen budget table as KC1 + max-tier violations
  folded in. (6) Phase 0v2 audits self_loop_row first (most complex
  untabled template). Pending final confirm on the blocker
  resolution, then acceptance for Phase 0v2.*
- *2026-07-12 — review loop closed. The reviewer's transcript
  re-verified the stack figures (draftsman prototype data, not
  memory) and the zero-margin finding; the one open sequencing
  question resolves by inspection: Phase 1 is `single_input_row`
  only — one solid input per machine, so aggregate == per-item and
  item-blindness is provably harmless until Phase 2, which the
  attribution check precedes by construction. **v2 ready for
  acceptance → Phase 0v2.***
- *2026-07-12 — reviewer's formal confirm landed: **SHIP**, no
  transcription distortion, blocker sequencing verified
  (single_input_row's `solid_inputs <= 1` row_kind guard makes
  Phase 1 structurally item-blind-safe; the pre-Phase-2 check
  landing means misassignment is caught same-commit). One scope
  pin adopted: the attribution check covers `triple_input_row`'s
  far/near pair as well as dual — Phase 2 bundles both, the check
  is NOT dual-only.*
