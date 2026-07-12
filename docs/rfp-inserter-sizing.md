# RFP: Inserter sizing — matching machine-side throughput to planned rates

## Summary

Every row template places exactly one regular inserter (~0.84/s) per
machine side, while the solver plans machines at per-side rates up to
several items/s — so machines are inserter-bound engine-wide, capped
at a fraction of their planned utilization in a real game. This was
invisible until the `inserter-throughput` check landed
(`rfp-lane-demand-flow.md` Phase 1); the 1/s gauntlet now carries
2/8/11/22/42/55 honest warnings across the six packs. This RFP sizes
inserters to rates: a shared per-side ladder (regular → fast →
multiple per face) chosen from the required rate, applied across all
row templates. It is the layout-side completion of the lane-demand-flow
work — the change that turns the honest warnings into honest PASSes.
It moves every golden hash in the corpus, so it ships with full
re-blessing discipline and browser-eyeball verification.

## Motivation

Reproducible today (`science_gauntlet`, HEAD ≥ `6849935`): all six
Nauvis packs at 1/s warn on `inserter-throughput`. Canonical case —
the logistic-science chain's iron-gear-wheel machine (AM2, 1.5
crafts/s planned): needs 3.0 plates/s in and 1.5 gears/s out, gets one
regular inserter (~0.84/s) on each side. Input caps the machine at
~28%; even the output side alone caps it at ~56%. In a real game the
factory under-delivers its target rate by roughly 3× at that machine —
and the tier ladder's "SOLVED (0 errors)" rows have never been true at
full rate in-game because no check measured this until now.

The fix cannot be tier-swap alone: one fast inserter (~2.31/s) still
does not feed 3.0/s. Multiple inserters per machine face are required
for the hungry sides, which makes this a geometry problem, not a
constants problem — hence an RFP.

## Design

### The ladder

New shared helper in `templates.rs` (or `common.rs`):

```
inserter_ladder(rate: f64, reach: Reach, free_columns: usize)
    -> Result<Vec<InserterPick>, LadderError>
// InserterPick = { dx_offset, entity_name }
```

Selection: cheapest configuration whose Σ throughput (mechanics table
I8, no-capacity-research constants, same source as the validator's
check — ONE constants table shared by both, so the fix and the check
cannot drift) covers the utilization-scaled per-machine side rate:

- `rate ≤ 0.84` → 1 regular (today's output, byte-identical intent)
- `rate ≤ 2.31` → 1 fast
- `rate ≤ 4.62` → 2 fast, adjacent free columns on the same face
- `rate ≤ 6.93` → 3 fast
- reach-2 positions (across-belt feeds): long-handed only, 1.2/s per
  column — there is no fast long-handed inserter, so reach-2 sides
  ladder by COUNT only (1.2 → 2.4 → 3.6…).
- above the face's column budget → `LadderError`, surfaced as a typed
  template failure (never silently under-provision).

Stack inserters (12/s, Space Age) are deliberately NOT in the v1
ladder — hand-size research assumptions and cost realism need their
own decision; noted as the escape hatch if Phase 0 finds sides the
fast-ladder can't cover.

### Integration

Each row template replaces its hardcoded single-inserter placement
with ladder output, per machine, per side:

- Input sides: rate = that ingredient's per-machine flow
  (`MachineSpec.inputs`, already per-machine), utilization-scaled the
  same way the validator scales (count/ceil convention).
- Output side: per-machine output rate, same scaling. Recyclers keep
  direct ejection (no output inserters — exempt in both fix and
  check).
- Sushi sort inserters already size by rate (Phase 3 sorter math) —
  untouched.
- Multi-inserter faces spread across the machine's belt-adjacent
  columns (width from `machine_dims`); the template owns which columns
  are free (fluid ports, second belts, and bridge rows occupy some).
  A face's free-column budget is template-specific and must be
  computed, not assumed.

No row pitch, belt position, or machine position changes — inserters
only fill columns that are already belt-adjacent and empty. That
constraint is load-bearing: if a template cannot satisfy a side within
existing geometry, it surfaces `LadderError` rather than growing the
row (see kill criteria).

### Non-goals

- No belt-tier escalation (hard user constraint, unchanged).
- No `max_inserter_tier` user knob in v1 (note for the UI backlog:
  early-game players may want to forbid fast inserters; the ladder is
  a pure function, so adding the knob later is cheap).
- No fluid-side changes (pipes don't use inserters).
- The ~5 residual input-rate-delivery warnings from cyclic demand
  (lane-demand-flow known limitation) are separate.

## Kill criteria

1. **Phase 0 coverage gate**: inventory every machine side in the 1/s
   gauntlet corpus (rate, reach, free columns — extractable from the
   inserter-throughput warnings plus template knowledge). If >5% of
   sides need more throughput than their face's fast-ladder maximum,
   the ladder design is insufficient — stop and rethink (stack
   inserters, direct insertion, or row redesign) before touching
   templates.
2. **Geometry containment**: if any template needs pitch, belt, or
   machine position changes to satisfy its sides at 1/s gauntlet
   rates, this RFP is under-scoped — stop; that template's redesign is
   its own work item (expected suspect: reach-2-only fluid rows, whose
   long-handed ladder caps low).
3. **Honest-zero gate**: after each template phase, the fixtures that
   template owns must show inserter-throughput = 0 with NO weakening
   of the check itself (any edit to `check_inserter_throughput` or the
   I8 constants in the same PR/commit as a template change is
   forbidden — the check gates the fix, so they must not move
   together).
4. **Budget gate**: entity-count increase ≤ +15% and zero area growth
   on every gauntlet fixture (inserters are the only new entities).
   Worse means the ladder is over-placing — stop and audit.
5. **In-game anchor**: one re-blessed layout (logistic@1/s) gets an
   in-game or user-confirmed spot check that the previously-starved
   gear machine now runs at full utilization. If it still starves with
   the warnings at zero, the CHECK is wrong (constants, scaling, or
   side attribution) — stop everything; the validator debt is deeper
   than this RFP.

## Verification plan

Per the CLAUDE.md protocol, plus:

- `science_gauntlet` 1/s: all six packs PASS (0 warnings) is the
  RFP's definition of done.
- `science_scaling_gauntlet` re-run; delta table into the decision
  log (higher-rate cells will retain their structural failures —
  those are out of scope; the inserter-throughput category should
  go to ~0 on structurally-valid cells).
- Golden re-bless per phase, with browser eyeball of at least tier1,
  tier2-from-ore, tier4-am2, and one self-loop row (user validates
  visuals per convention).
- Unit tests: ladder selection at each boundary rate, reach-2
  count-laddering, LadderError on impossible sides, shared-constants
  identity between check and ladder (a test that fails if they
  diverge).
- Netflow determinism sweep untouched (solver is not involved).

## Phasing

- **Phase 0 — corpus inventory** (no layout change): per-side rate ×
  reach × free-column census across the 1/s gauntlet; kill criterion
  1 evaluated; ladder table frozen.
- **Phase 1 — ladder helper + `single_input_row`** (the workhorse):
  tier1/tier2 fixtures go inserter-warning-zero; goldens re-blessed;
  browser eyeball.
- **Phase 2 — remaining solid templates**: dual/triple/quad input,
  horizontal stack, self-loop, voider, scrap row.
- **Phase 3 — reach-2 / fluid-adjacent rows**: long-handed count
  ladder; kill criterion 2 is most likely to fire here — evaluate
  honestly, spin off template redesign if it does.
- **Phase 4 — close-out**: both gauntlets re-run, delta tables
  recorded, CLAUDE.md ladder refreshed (tier rows finally mean
  "delivers the planned rate", not just "validates clean"), in-game
  anchor (kill criterion 5).

## Decision log

- *2026-07-12 — drafted, immediately following lane-demand-flow
  Phase 1 (`6849935`), which made the defect visible: 1/s gauntlet
  carries 140 inserter-throughput warnings across the six packs.
  This RFP is the layout-side completion of that work. Pending
  review.*
