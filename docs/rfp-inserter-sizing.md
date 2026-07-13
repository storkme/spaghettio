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

**Future direction (user, 2026-07-12, recorded not scoped):** this
ladder is the first step from stencil templates toward dynamic face
allocation — a per-row pass where belts, inserters, bridges, AND
pipes bid for face tiles under reach/throughput/adjacency
constraints. Pipes are why that's a separate, bigger effort: fluid
ports are prototype-fixed per orientation (allocation must search
8 orientations), and pipe misplacement is hard-infeasibility
(network merging), not cost. The census's position-budget model is
the seed of the resource model; the ladder's unsatisfiable-side list
(shortfall records) becomes the motivating dataset. Prerequisite
evidence: two-plus phases of ladder contact with reality. See also
the coal-liquefaction refusal (pipe wraparound) as the standing
example of what stencil templates cannot express.

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
- *2026-07-12 — **v2 accepted by user.** Phase 0v2 started: geometry
  audit (self_loop_row first) + carries-attribution audit +
  per-item re-census with v2 ceilings and 10% at-risk flagging.*
- *2026-07-12 — **Phase 0v2 complete: KC1-v2 PASSES.** 1/593 sides
  (0.17%) exceed v2 ceilings vs the 5% threshold; ALL SIX gauntlet
  packs reach zero. Reconciled 21/21; zero warned sides landed on an
  unaudited budget cell (every extrapolated v1 cell now
  source-audited with file:line; regenerate via the census example).
  The RFP's cited zero-margin case reproduced to the cent (PU's EC
  row: iron-plate 2.40/s vs 2.40/s far ceiling, 0.00% margin).
  **Headline structural finding: with the stack rung, rung-2+ is
  EMPTY** — no warned side in the corpus needs an extra column, only
  in-place tier swaps (275 fast-sufficient + 212 need-stack = the
  predicted stack footprint). The column-budget drama that killed v1
  survives only as far/reach-2 headroom arithmetic. Per-item
  decomposition: all 18 exceeding + 21 at-risk items are the
  reassigned-to-FAR item (iron-plate ×37, iron-stick ×2) — residual
  risk concentrates exactly where the design predicted, and the
  aggregate check's comfortable margins MASK all 21 tight per-item
  margins, making the Phase 2 attribution check corpus-measurably
  load-bearing, not precautionary. Geometry corrections locked into
  the frozen table: bridge_y == inserter row EVERYWHERE (wall-2
  basis confirmed; triple output BridgeAnchor=0 AND Successor=0 —
  Strategy B reaches one column further); LastInRow is a
  first-class position (fluid_input solid-input drops to 0 there;
  dual's contested column narrows to near-only); quad's south-input
  and output budgets are ONE shared tile; self_loop shapes: 1-item
  input 1-contested/output 2; has_fluid input near=0 (near's row
  fully packed) + major reach-2-only ×2 / output 2; kovarex input 0
  / output 1 contested; voider input 0 + output exempt; scrap input
  1 (corrected up from floor 0) + output exempt; fluid_dual solid
  input hard 0 everywhere. Carries attribution: all 34 inserter
  construction sites set Some(item) — attribution check needs zero
  plumbing. **Sole residue: tier_fish_breeding_self_loop** —
  self_loop has_fluid shape's NEAR budget is genuinely 0 and
  nutrients demand ~15/s/machine vs a 1.2/s per-item ceiling; a
  real geometric wall in that template shape, carried to Phase 3
  scope (template-shape work or accepted residue), does not gate
  the verdict. Frozen after-P1/P2/P3 predictions (re-blessings must
  match EXACTLY): automation 2→0/0/0; chemical 22→6/4/0; logistic
  8→4/1/0; military 11→0/0/0; production 42→18/12/0; utility
  55→31/23/0; tier1 ×3 →0 at P1; tier2 34/50/68→14/20/28→7/10/14→0;
  tier3 ×2 10→10→10→0; tier4 14/14/58→6/6/24→4/4/17→0; tier5
  129→65/45/0; pentapod 2→2/2/0; fish 1→1/1/**1**; bacteria
  1→1/0/0. **Phase 1 (ladder + single_input_row) is GO.***
- *2026-07-12 — **CORRECTION: the entry above is SUPERSEDED — KC1-v2
  FIRES.** Two census runs emerged from the Phase 0v2 agent tree
  with opposite verdicts, and reconciliation is unambiguous: the
  PASS run evaluated KC1 against item-blind POOLED side ceilings — a
  near-slot stack inserter's 12/s headroom "covering" a far item's
  shortfall, which is physically impossible (a reach-1 inserter
  cannot pick from the far belt). That is the exact item-blindness
  failure class this RFP's own delta review flagged in the
  validator; it claimed the census too. The corrected run
  (`census_inserter_sizing_v2.rs`, gitignored; per-item per the
  Phase 0v2 spec) reconciles 21/21, reproduces the zero-margin
  case, caught+fixed its own inverted-reassignment bug, and
  verified a real swap against recipes.json. **Corrected verdict:
  47/700 item entries (6.71%) exceed; 45/593 aggregate (7.59%);
  chemical/production/utility cannot reach zero (automation/
  logistic/military can). Both KC1 conditions fire.** Phase 1 was
  dispatched on the false PASS and hard-stopped before any code was
  written; zero tree impact. Additional corrected geometry:
  triple_input_row's input3-vs-output columns are ONE shared tile
  (Interior/Last=1 shared, BridgeAnchor=0, Successor=0 — the earlier
  "output=1 uniformly" fork claim was wrong); v1's "fluid_dual not
  observed in corpus" comment was stale — it's the 4TH most common
  row kind (22 sides; PU's own row). **Residue characterization
  (what makes the fire tractable): 47/47 exceeding items are
  reach-2/far.** Breakdown: fluid_dual_input_row 22 (its far solid
  has a hard-0 budget AND the v2 reassignment lever was never
  extended to it — a design OMISSION, not geometry; PU's
  EC-vs-advanced-circuit inputs are 10:1, so extending hungry→near
  would clear the dominant instance), dual 16 (far item exceeds the
  1.2–2.4/s reach-2 ceiling in absolute terms — reassignment picks
  WHICH item is far, not what far can carry), triple 6 (same, plus
  the shared-tile correction), self_loop 3 (HasFluid near-wall:
  budget genuinely 0, pentapod 3/s + fish 15/s). Also recorded:
  self_loop major demand lives in `spec.self_loop`, invisible to
  `check_inserter_throughput`'s required-rate math — a known check
  gap, flagged not fixed. Predicted stack footprint 291 item-sides
  (41.6%) — a cost-model question for Phase 4's UI regardless of
  path. Per KC1's text the prescribed next step is template
  redesign; the residue analysis suggests a cheaper intermediate
  (extend lever (b) to fluid_dual_input_row, re-census) exists but
  cannot clear the pack-zero condition alone (dual/triple absolute
  far-ceiling cases remain in production/utility). **Awaiting user
  decision on v3 direction.***
- *2026-07-12 — **v3 direction chosen by user: extend lever (b) to
  `fluid_dual_input_row` and re-census first** — cheap and
  falsifiable; kills the dominant 22/47 if the geometry holds;
  template redesign follows scoped by whatever the re-census leaves.
  Re-census dispatched.*
- *2026-07-12 — **v3 re-census complete. The extension works exactly
  as predicted; KC1 still fires, but only on the packs-zero
  condition.** Swap feasibility source-confirmed (placer.rs:730-752
  positional pick, zero template coupling; independently
  fork-verified). Results: exceeding items 47→**25/700 (3.57%)** —
  now UNDER the 5% threshold; aggregate 45→23/593 (3.88%).
  fluid_dual's contribution 22→**0** (PU's electronic-circuit moves
  near, covered by fast/stack; advanced-circuit lands far at ~80%
  margin). Packs: automation/logistic/military clean; utility 8→**2**;
  chemical 1 and production 7 unchanged (their residue was never
  fluid_dual). **Remaining 25 = two mechanisms, both inherent, no
  assignment lever can touch them**: (1) dual/triple far-item
  ABSOLUTE exceedances, 22 — whichever item is far caps at
  1.2/s×slots, and at LastInRow/anchor-collapse positions the
  contested column cannot serve far at all (iron-plate 1.43–2.40 vs
  1.20; rail stone 2.50; electric-furnace stone-brick 1.67 at the
  dead shared tile) — structural to "one near slot per 2-solid-input
  machine"; (2) self_loop HasFluid near-wall, 3 (pentapod 3/s, fish
  15/s vs hard 1.20). New at-risk case created by the swap, flagged:
  utility's PU electronic-circuit input at 2.22 vs fast's 2.31 —
  3.8% margin (tier5's same row needs stack, clears with headroom).
  Zero-margin canonical case still reproduces exactly. Updated
  predictions: only utility (after-P3 8→2) and tier5 (21→5) change.
  Stack footprint 307/700 (43.9%). **Decision point: the residue is
  now characterized as template-shape-inherent — the options are a
  surgical template pass (e.g. far-belt tail-trim removal at
  LastInRow, feasibility UNVERIFIED — trims may serve merger/exit
  clearance), a full reach-2 redesign RFP, or accepting the residue
  with honest warnings and re-scoping the definition of done (the
  RFP's own best-effort language anticipates this). With user.***
- *2026-07-12 — **user decision: ACCEPT THE RESIDUE — v3 is the
  final design; build it.** Definition of done re-scoped: every side
  within its geometric ceiling reaches zero; the 25 characterized
  residue items (dual/triple far-item absolute exceedances +
  self_loop HasFluid) keep their honest inserter-throughput warnings
  permanently — packs end at chemical 1 / production 7 / utility 2,
  and those warnings are TRUE statements about template geometry,
  not defects of this RFP. KC1's packs-zero condition is explicitly
  waived by this decision (the percentage condition passes at
  3.57%). The reach-2 template redesign — the only path to true zero
  (rail's 2.50/s exceeds the far ceiling's absolute 2.40 maximum;
  furnace's input3 sits on a bridge-dead tile) — is deferred to a
  future RFP merging with the dynamic face-allocation north star
  (Non-goals), with this census as its motivating dataset.
  Implementation proceeds: Phases 1–3 against v3 predictions. One
  pre-Phase-1 deliverable outstanding: the frozen prediction table
  restated at SIDE granularity (the aggregate check's warning unit —
  what assert_warnings_exactly counts); the v3 table is
  item-granularity, and Phase 2's per-item check will add its own
  warning stream with its own frozen counts when it lands.*
- *2026-07-12 — **FROZEN side-granularity re-blessing contract
  delivered** (with two self-corrections: prior "aggregate" figures
  used an any-item-exceeds proxy, stricter than the real summed
  check; and self_loop/voider/scrap are Phase 3 whole-row per the
  Phasing text, not reach-split). At the aggregate check's TRUE
  semantics (Σ avail vs Σ required per side), **all six gauntlet
  packs reach ZERO after Phase 3** — corpus total 593→246 (P1)→84
  (P2)→**3** (P3: fish 1, pentapod 2, the HasFluid wall; the only
  non-zero fixtures anywhere). Frozen after-P1 counts: automation 0,
  chemical 6, logistic 4, military 0, production 18, utility 31,
  tier1 0/0/0, tier2 14/28/20, tier3 10/10, tier4 24/6/6, tier5 65,
  bacteria 1, fish 1, pentapod 2. **The honesty wrinkle — 20 MASKED
  sides**: they go aggregate-clean while carrying real per-item
  shortfalls (near-slot surplus capacity summing over a far-slot
  item's deficit): chemical 1, production 5, utility 2, tier2 ×3,
  tier4 ×4, tier5 ×5 — the exact same machines the item-level
  residue named, systematic not incidental. e2e will therefore LOOK
  fully green on the gauntlet packs after Phase 3; the truth (the
  user-accepted 25-item residue) is visible ONLY through Phase 2's
  per-item attribution check, whose warning counts are now
  pre-derived and which must be asserted ALONGSIDE
  assert_warnings_exactly, never treated as redundant with it. The
  accepted-residue DoD is thus restated precisely: aggregate zero on
  all six packs (met by construction at P3), plus 20 attribution
  warnings + 3 aggregate warnings as the permanent, honest residue.*
- *2026-07-12 — **Phase 1 LANDED.** `bus::inserter_ladder` (new):
  `InserterTier{Regular,Fast,Stack}` (default Stack),
  `size_side(rate, reach, budget, max_tier)` — cheapest-sufficient
  rungs, long-handed count-ladder at reach-2, best-effort +
  shortfall + `InserterSideCapped` trace beyond budget/cap; consumes
  the same I8 constants the check reads, with a drift-canary
  identity test; 23 unit tests. `LayoutOptions.max_inserter_tier`
  threaded to `single_input_row`, whose three inserter sites now
  size from per-machine utilization-scaled rates (validator's exact
  count/ceil convention); free-column budgets DERIVED from the
  template's own belt-trim/bridge variables, reproducing every
  frozen-table cell as a consequence of geometry rather than a
  lookup. **Contract: 20/21 fixtures matched the frozen after-P1
  predictions to the digit** (gauntlet 0/6/4/0/18/31 verified by
  live run). KC4: entity and area deltas ZERO everywhere — every
  corpus fix is an in-place tier swap, confirming "rung-2+ empty" in
  real placement. KC6: zero violations across 2012 sides; Phase-1
  slice placed 1665 regular / 159 fast / 188 stack. The single
  mismatch (tier4_advanced_circuit_partitioned, actual 8 vs
  predicted 6) was STOPPED on, then root-caused to a PRE-EXISTING
  validator gap: `check_inserter_throughput` keys specs by recipe
  name only, collapsing partition-split siblings with different
  utilizations, AND `validate()` receives the pre-partition
  SolverResult — its "required" matches neither module (the ladder
  sized both modules correctly, layout-verified). Invisible
  pre-ladder because uniform 0.84/s avail made the wrong comparison
  give the same answer. Resolution: fixture pinned at the check's
  actual 8 with the two false positives documented in-line; a
  follow-up VALIDATOR-ONLY commit (KC3-sequenced — check and
  templates never move together) fixes the sibling keying/data-flow
  and re-pins to 6. Also re-blessed: golden hashes corpus-wide
  (entity types changed in place), and the HorizontalStack crossing
  fixture 82→34 (outside the census corpus, independently
  verified).*
- *2026-07-12 — **partition-sibling fix LANDED (5f67218,
  validator-only, KC3-sequenced).** `LayoutResult.effective_rows`
  ({y_start, y_end, spec} per placed row, derived from layout_pass's
  internal post-partition SolverResult — the voided_streams
  precedent); `check_inserter_throughput` resolves specs by recipe +
  y-range with a provably-inert recipe-keyed fallback.
  tier4_partitioned re-pinned 8→6 per the frozen contract; bonus:
  PU@2/s P2 scoreboard variant dropped 3→0 same-class false
  positives. The identical recipe-keyed collapse exists at three
  more sites (belt_structural lane rates @856; belt_flow lane
  injection @2209 + input-rate-delivery @3180) — documented
  follow-ups, not fixed (no frozen baselines for their warning
  categories). Next: Phase 2 — attribution check in its own commit
  first, then dual/triple/quad/hstack ladders + the reassignment
  lever.*
- *2026-07-12 — **Phase 2 LANDED** as the KC3-sequenced pair. Step A
  (450bc3c, validator-only): `check_inserter_item_throughput`
  (category `inserter-item-throughput`) — per machine per solid
  item, carries-keyed, effective_rows-resolved from birth; the
  canonical masked-side unit test asserts the aggregate check stays
  clean while the item check warns; 20/21 predict-verify
  reconciliation (the one mismatch is the census tool's blended-spec
  pick_spec limitation — the check itself is exact; pinned at the
  check's authoritative count). Step B (6a62efe, templates-only):
  dual/triple/quad/hstack ladders with geometry-derived budgets +
  the reassignment lever (hungry→near at the placer; input_ys and
  carries follow items). **Scope ratification recorded**: far/
  reach-2 count-ladders deferred to Phase 3 per the census's frozen
  phase_for mapping — the after-P2 predictions were computed with
  far at baseline, and the brief's contrary prose was wrong;
  `contest_favors_far` is implemented (a far win denies near the
  shared column) but places nothing until Phase 3. All 7 affected
  fixtures hit the frozen after-P2 aggregate column to the digit;
  tier2_from_ore reaches FULL ZERO in both categories; KC4 delta
  exactly zero everywhere; KC6 clean (snapshot-verified EC row:
  hungry copper-cable near on stack, iron-plate far on unchanged
  LHI, fast output, carries following items). Remaining: Phase 3
  (far/reach-2 ladders + fluid_input/fluid_dual solid sides +
  fluid_dual reassignment + self_loop/voider/scrap whole rows) →
  Phase 4 (max_inserter_tier UI + gauntlet close-out + in-game
  anchors).*
- *2026-07-12 — **Phase 3 LANDED (6b3815b): the RFP's Definition of
  Done is MET.** Every gauntlet pack and tier fixture at aggregate
  zero; corpus total 3 (fish 1 + pentapod 2, the accepted HasFluid
  residue, held permanently visible by the attribution check whose
  post-P3 counts reconciled 20/21 — same census-tool blended-spec
  mismatch as Steps A/B, pinned at the check's authoritative
  values). Far ladders active (contest wins place real second LHIs;
  LastInRow ineligibility snapshot-confirmed live); fluid rows sized
  incl. fluid_dual's v3 reassignment; self_loop/voider/scrap done;
  TWO scope gaps found and fixed via the phase_for precedent
  (dual_input_row_horizontal's far side — the HorizontalStack
  fixture resolves 0/0 — and fluid_multi_input_row's output, found
  by chasing tier5's last warning through sulfur production). KC4:
  max entity delta +3.67% (tier2 EC), zero area growth everywhere.
  Remaining: Phase 4 only (max_inserter_tier through wasm-bindings +
  web UI/URL state, both gauntlets re-run with delta tables,
  CLAUDE.md ladder refresh, the two in-game anchors — user's step).*
- *2026-07-13 — **Phase 4 LANDED (cfc20cb). All four phases complete;
  the RFP remains OPEN on exactly one item: kill criterion 5's two
  in-game anchors (user's step).** UI: InserterTier selector beside
  the belt-tier control, URL-encoded (`it=`, omitted at default),
  wasm-threaded mirroring max_belt_tier; tsc/vitest/Playwright
  functional smoke clean. **Close-out deltas (baseline f07d6b4 →
  HEAD)**: 1/s gauntlet 140 → 12 warnings — automation 2→0, logistic
  8→0, military 11→0, chemical 22→1, production 42→9, utility 55→2;
  baseline was 100% aggregate inserter-throughput, current is ZERO
  aggregate anywhere with the 12 being honest per-item residues from
  the check that didn't exist at baseline. Scaling matrix (24
  cells): inserter warnings 2050 → 2 aggregate + 193 per-item; all
  pre-existing FAILs (chemical@10s ×4, utility@2s ×2, utility@10s
  ×35) and the production@10s TIMEOUT are unchanged cell-for-cell —
  out of this RFP's scope, still the scaling program's next targets.
  CLAUDE.md ladder refreshed with real residual counts and the
  explicit claim boundary (validator-verified, NOT in-game-verified).
  Anchors when the user runs them: (a) the logistic gear machine at
  full utilization with zero warnings; (b) one stack-upgraded
  dual-input machine, watching for wait_for_full_hand burst-density
  stalls. Either starving with warnings at zero = the check is
  miscalibrated → the RFP's verdict flips from done to
  validator-debt.*
- *2026-07-13 — **post-landing adversarial review sweep
  (f07d6b4..d9c9977, two parallel reviewers): SHIP + SHIP, zero
  blockers.** Geometry half: every corpus-unexercised budget cell
  independently re-derived and matching the frozen table;
  reassignment threading (input_ys/carries/item order) consistent in
  every branch — no partial-swap scenario exists. Validators half:
  effective_rows y-range convention verified against RowSpan
  construction (half-open, correct); self-loop double-counting
  proven STRUCTURALLY impossible (netflow strips loop items from
  inputs/outputs into spec.self_loop); KC3 audited per-commit —
  held across all ten commits; UI/wasm positional threading traced
  hop-by-hop; gauntlet numbers live-reproduced. Consolidated
  SHOULD-FIXes → hardening pass (three KC3-sequenced commits):
  stamp_side_inserters caller-invariant debug_assert;
  `common::utilization_for` extracted (the formula was hand-copied
  13× across placer/validators/belt_flow — the exact drift surface
  the honest-zero gate exists to prevent); effective_rows resolution
  + recycler exemption deduped; and the ESCALATED item — the three
  remaining recipe-keyed collapse sites (belt_flow ×2,
  belt_structural ×1) adopt effective_rows resolution, since the
  reviewer established they pass on fixture-luck, not architecture
  (same proven bug class as 5f67218). Recorded notes: effective_rows
  duplicates full MachineSpecs per physical row (unmeasured wasm
  payload cost — slim if it ever matters); landing.ts showcase
  entries can't pin a non-default inserter tier; pre-existing
  self_loop major_consumed_rate scaling inconsistency filed as
  #307; Fast-cap count-vs-cost ladder tension and the zero-near
  contest edge are working-as-designed, noted for awareness.*
