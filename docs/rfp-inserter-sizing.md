# RFP: Inserter sizing — matching machine-side throughput to planned rates

## Summary

Every row template places exactly one regular inserter (~0.84/s) per
machine side, while the solver plans machines at per-side rates up to
several items/s — so machines are inserter-bound engine-wide, capped
at a fraction of their planned utilization in a real game. This was
invisible until the `inserter-throughput` check landed
(`rfp-lane-demand-flow.md` Phase 1); the 1/s gauntlet now carries
2/8/11/22/42/55 honest warnings across the six packs. This RFP sizes
inserters to rates: a shared per-side ladder (in-place tier upgrade
first, extra inserters where the face's REAL free-column budget
allows), applied across all row templates, best-effort where geometry
caps out — with the shortfall kept honestly visible, never hidden. It
is the layout-side completion of the lane-demand-flow work. It moves
every golden hash in the corpus, so it ships with full re-blessing
discipline and browser-eyeball verification.

## Motivation

Reproducible today (`science_gauntlet`, HEAD ≥ `6849935`): all six
Nauvis packs at 1/s warn on `inserter-throughput`. Canonical case —
the logistic-science chain's iron-gear-wheel machine (AM2, 1.5
crafts/s planned): needs 3.0 plates/s in and 1.5 gears/s out, gets one
regular inserter (~0.84/s) on each side. Input caps the machine at
~28%. In a real game the factory under-delivers its target by roughly
3× at that machine, and the tier ladder's "SOLVED (0 errors)" rows
have never been true at full rate in-game.

Tier-swap alone cannot fix everything (one fast inserter ≈ 2.31/s <
3.0/s), so multi-inserter faces are required for the hungriest sides —
which makes this a geometry problem. The adversarial review round
(2026-07-12, decision log) established that the geometry is much
tighter than a naive reading of the templates suggests; the design
below is built on that census, not on assumption.

## Design

### The real free-column budget (review census, verified in code)

Extra inserters can only occupy columns that are belt-adjacent AND
unoccupied. The actual budget per face, from `templates.rs`:

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

(Bridge-anchor/last-in-row modifiers compose with the per-template
rows; Phase 0's census enumerates the full position product, this
table is the shape reference.)

Two consequences drive the whole design:

1. **The workhorse rung is the in-place tier swap** (regular → fast,
   ≤2.31/s), which needs ZERO extra columns and therefore works at
   every position in every template. Extra-column rungs are the
   exception, available mostly on single-input faces.
2. **Reach-2 (far-belt) sides ladder by long-handed count only**
   (1.2/s per column, no fast long-handed exists) and their column
   budget is 0–1 contested — so hungry far ingredients on
   triple/quad rows are geometrically capped TODAY, by derivation,
   not as a Phase 3 discovery. Those sides stay best-effort + warned
   until a template redesign (explicitly out of scope, spun off if
   the Phase 0 census shows the 1/s gauntlet needs it).

### The ladder

Shared helper (one constants table with the validator's
`check_inserter_throughput` — I8 values, no-capacity-research; a unit
test fails if fix and check ever diverge):

```
size_side(rate, reach, position_budget) -> SidePlan
// SidePlan = { picks: Vec<(dx, entity)>, shortfall: Option<f64> }
```

- Rung 0: keep 1 regular (rate ≤ 0.84).
- Rung 1: in-place swap to fast (rate ≤ 2.31). Zero columns. Reach-2:
  stays long-handed (≤1.2).
- Rung 2+: add fast/long-handed inserters into the position's actual
  free columns, up to its budget.
- Beyond budget: **best-effort placement + `shortfall`**. The
  template places the best achievable config; the existing
  inserter-throughput warning remains (honesty preserved by the
  check, not by this code); a `InserterSideCapped` trace event
  records the geometric cap. Layouts never FAIL from sizing — they
  degrade exactly as visibly as they do today.
- **Contested columns** (dual/triple rows): resolved deterministically
  — the side with the larger relative shortfall wins; tie breaks to
  the far/reach-2 side (its per-inserter rate is lower, so it needs
  the column more). Determinism is a hard project contract: the rule
  is a pure function of the two rates, unit-tested, no iteration
  order anywhere.

### Integration

- Rates: `MachineSpec.inputs`/`outputs` are per-machine (verified:
  netflow constructs them as per-machine at 100%); utilization
  scaling reproduces the validator's exact convention
  (`count/ceil(count)` global scalar, validate/inserters.rs:254).
- **Scope note (review finding 2)**: no template currently receives
  any rate parameter — all ~10 row-template signatures gain a
  per-side-rates argument threaded from `build_one_row`, and every
  placer call site updates. Mechanical but wide; the diff is larger
  than "add a helper".
- Recyclers keep direct ejection (exempt in fix and check); sushi
  sort inserters already size by rate (untouched).
- Power: added inserters land inside existing row footprints already
  blanketed by the pole grid — verified per phase via the existing
  power checks, called out in the verification plan (review noted it
  was previously unstated).

### Composition gap (review finding 5, recorded)

I8's chest-to-chest constants are correct for belt-adjacent inserters
under saturation (swing timing is source-agnostic) — no derating. The
genuine residual: `check_inserter_throughput` assumes the feeding
belt sustains the demanded rate; nothing cross-checks an upsized
inserter's demand against its feeding lane's provision. Mitigation
here: a doc note in `factorio-mechanics.md` I8 stating the saturation
precondition, and the demand-pull walker (already landed) naturally
raises lane delivery toward real demand. A dedicated cross-check is
noted for the validator backlog, not built here.

### Non-goals

- No belt-tier escalation (hard user constraint).
- No `max_inserter_tier` user knob in v1 (UI backlog note).
- No template geometry changes — no pitch, belt, or machine moves.
- No stack inserters in the v1 ladder (research/cost assumptions
  need their own decision; escape hatch if the census demands it).
- Multi-input-row redesign for capped far-ingredients: separate
  follow-up if needed.
- The ~5 residual input-rate-delivery warnings from cyclic demand
  (lane-demand-flow known limitation): separate.

## Kill criteria

1. **Phase 0 coverage gate (rewritten after review)**: the census
   compares every machine side's utilization-scaled rate against its
   POSITION's real ceiling from the free-column table above — NOT
   against the abstract ladder maximum. Method: enumerate machine
   sides per template per row-position across the six 1/s gauntlet
   layouts (the warnings carry rates; the table carries budgets). If
   >5% of sides exceed their position ceiling, OR any of the six
   packs cannot reach zero warnings at 1/s within existing geometry,
   the ladder design is insufficient for its own definition of done
   — stop and rethink (template redesign or stack inserters) before
   any template code.
2. **Geometry containment**: if `single_input_row` or
   `dual_input_row` (the workhorses) turn out to need pitch/belt/
   machine-position changes to reach zero at 1/s, stop — this RFP is
   under-scoped. (Triple/quad far-side caps are already known and
   handled as best-effort residue, per the census — they do NOT trip
   this criterion.)
3. **Honest-zero gate**: per phase, the fixtures that phase's
   templates own must reach their predicted counts with NO edits to
   `check_inserter_throughput` or the I8 constants in any commit
   that also touches templates — the check gates the fix; they may
   never move together.
4. **Budget gate**: entity-count increase ≤ +15% and zero area
   growth on every gauntlet fixture. Worse means the ladder
   over-places — stop and audit.
5. **In-game anchors (two, review-extended)**: (a) the logistic
   gear machine (single-input face) after Phase 1; (b) one
   contested-column dual/triple-input machine after Phase 2. If
   either still starves with its warnings at zero, the CHECK is
   miscalibrated — stop everything; the validator debt is deeper
   than this RFP.

## Verification plan

Per the CLAUDE.md protocol, plus:

- Definition of done: `science_gauntlet` 1/s — every side within its
  geometric ceiling at zero warnings; any residual capped sides
  enumerated in the decision log with their shortfalls (Phase 0
  predicts this set; if it's non-empty for the six packs, kill
  criterion 1 already fired).
- `science_scaling_gauntlet` re-run; delta into the decision log.
- Golden re-bless per phase with browser eyeball of tier1,
  tier2-from-ore, tier4-am2, one self-loop row (user validates).
- Power checks green after each phase (added inserters draw from the
  existing pole grid — verify, don't assume).
- Unit tests: ladder boundary rates, reach-2 count-laddering,
  contested-column resolution (both orders + tie), best-effort
  shortfall path, shared-constants identity with the check,
  determinism of column choice.
- Netflow determinism sweep untouched.

## Phasing

- **Phase 0 — corpus census** (no layout change): per-side rate ×
  position ceiling across the six 1/s gauntlet layouts, built on the
  review's per-template column table; kill criterion 1 evaluated;
  the per-phase expected-warning-count table for every gauntlet
  fixture is FROZEN here (so re-blessing is predicted, not
  discovered).
- **Phase 1 — ladder helper + `single_input_row`**: tier1 fixtures
  reach zero. Mixed fixtures (tier2+ — electronic-circuit is a
  DUAL-input row, review finding 7) re-bless to the reduced counts
  predicted by Phase 0. Anchor (a).
- **Phase 2 — `dual_input_row` + remaining solid templates**
  (triple/quad/horizontal-stack/self-loop/voider/scrap): tier2
  reaches zero; triple/quad best-effort residue lands as predicted.
  Anchor (b).
- **Phase 3 — reach-2 / fluid-adjacent rows**: long-handed count
  ladder within real budgets; enumerate the geometrically-capped
  residue; if that residue blocks any 1/s pack from PASS, spin off
  the template-redesign RFP (it will already have the census data).
- **Phase 4 — close-out**: both gauntlets re-run, delta tables
  recorded, CLAUDE.md ladder refreshed (tier rows finally mean
  "delivers the planned rate"), both in-game anchors confirmed.

Every mixed fixture is re-blessed at most twice (Phase 1 reduced
count, Phase 2/3 final count), and both blessings are against counts
predicted by the Phase 0 table — a blessing that doesn't match its
prediction is itself a stop signal.

## Decision log

- *2026-07-12 — drafted, immediately following lane-demand-flow
  Phase 1 (`6849935`): 1/s gauntlet carries 140 inserter-throughput
  warnings across the six packs. Pending review.*
- *2026-07-12 — adversarial review round. Verdict: REVISE (core
  ladder concept sound, constants verified — chest-to-chest I8
  numbers hold for belt-adjacent inserters under saturation, no
  derating). Three blocking findings, all incorporated: (1) the
  free-column budget is position-dependent and far tighter than the
  draft assumed (census table now in Design — bridge-anchor output
  faces have ZERO free columns; quad-row north faces likewise;
  multi-input rows have one CONTESTED column); the in-place tier
  swap is therefore the workhorse rung, and beyond-budget sides are
  best-effort + honestly-warned rather than hard errors. (2) Kill
  criterion 1 rewritten to measure against per-position ceilings,
  not the abstract ladder max — as drafted it would have
  systematically underreported its own target failure mode. (3) The
  Phase 1 claim "tier1/tier2 go zero" was factually wrong —
  electronic-circuit is a dual-input row (recipes.json: iron-plate +
  copper-cable×3; placer::row_kind routes 2 solid inputs to
  DualInput), so tier2 clears in Phase 2; phasing corrected, and
  every fixture's per-phase expected count is frozen in the Phase 0
  table so re-blessing is predicted, not discovered. Also adopted:
  deterministic contested-column resolution rule (golden-churn
  guard), power verification made explicit, second in-game anchor on
  a contested-column face, composition-gap note (inserter check
  assumes feeding-belt saturation — I8 doc note + validator-backlog
  cross-check). Pending re-review / acceptance.*
- *2026-07-12 — re-review verification: **SHIP** after two census
  cells corrected (dual_input_row and fluid_input_row interior
  OUTPUT budgets are 2, not 1 — their output rows are
  single_input_row-shaped; the draft extrapolated from triple/quad's
  genuinely-1 shape). Both were understated (safe direction). Also
  added: bridge-anchor successor position (1 output col) and the
  east-flow last-in-row note, so Phase 0's census enumerates the
  full position product. Re-review confirmed: KC1 rewrite
  enforceable; phasing double-churn resolved; best-effort+warning
  floor is provably ≥ today's baseline at every position (rung-1
  in-place swap needs zero columns), so no new honesty hole — the
  check remains the independent auditor. **Ready for acceptance.***
- *2026-07-12 — **accepted by user.** Phase 0 census started.*
