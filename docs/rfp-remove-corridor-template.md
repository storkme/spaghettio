# RFP: Remove the corridor template

## Summary

Delete the corridor-template pre-pass at `ghost_router.rs:1006-1215`
and all its downstream tracking (`corridor_handled`, `CorridorTemplate`
region kind, related `Unreleasable` paths). Every ghost-crossing
becomes the junction solver's responsibility — the cluster-formation
and SAT strategies already handle the patterns the corridor template
was catching, and the template's greedy stamping actively forecloses
better joint solutions the solver would otherwise find.

Performance of running SAT on patterns the corridor template used to
short-circuit is a recognised cost. The project convention — lock
down correctness first, optimise later — puts that cost outside this
RFP's scope.

## Motivation

The corridor template was introduced in
[`b383bed`](https://github.com/storkme/fucktorio/commit/b383bed)
(2026-04-12) with this rationale:

> Add a corridor template for runs of 2..4 adjacent trunks crossed
> by one horizontal spec — emits a single long UG bridge instead of
> N conflicting per-tile templates.

At the time, the only alternative was the **per-tile template**
system, which produced entity overlaps when stamped on N adjacent
columns. The corridor template fixed that with one long UG pair per
run. Measured validator-error reduction of 28% on
`tier4_advanced_circuit_from_ore_am1_ghost`.

**Two days later**
([`0ea0367`](https://github.com/storkme/fucktorio/commit/0ea0367),
2026-04-14) the SAT-based region-growth junction solver was
introduced. That solver can handle arbitrary crossing shapes, and
the cluster-formation machinery (`cluster_adjacent_crossings`)
automatically groups N adjacent crossings into a single cluster —
eliminating the "N conflicting per-tile templates" problem by
construction, without the corridor template's help.

The corridor template stayed where it was — in front of the junction
solver, greedy-grabbing runs before the solver gets a look. We're
now seeing the downstream cost of that order.

### Concrete failure: `advanced_circuit_iron_plate_trio_capped`

The capped fixture has two adjacent-trunk runs on the
`ret:electronic-circuit:2:161` horizontal spec:

- **plastic-bar run** at `x=25, 26` (2 crossings)
- **iron-plate run** at `x=21, 22, 23` (3 crossings)

separated by a non-trunk gap at `x=24`. The corridor template walks
the path, finds the plastic-bar run first, and stamps a UG pair
`(27,161) → (24,161)`. It then tries to bridge the iron-plate run
with `ug_in=(24,161)` — **but that tile was just claimed by the
plastic-bar bridge**. The iron-plate endpoint check fails, the
bridge is skipped, and the iron-plate run is handed to the junction
solver.

By the time the solver sees (21,161), the corridor's `(24,161)` UG
output is `Unreleasable`. The solver cannot propose a longer single
UG spanning all 5 trunks (which would be the natural joint solution)
because the tile it needs is frozen. No amount of region growth,
veto-directed or otherwise, can recover from that. The fixture caps.

### Why the template is now genuinely redundant

Three independent pieces of machinery have landed since the corridor
template was added, each subsuming part of its role:

1. **`cluster_adjacent_crossings`** — groups N adjacent crossings
   sharing a spec into one cluster. Solves the "N conflicting
   per-tile templates" problem at the clustering level, with no
   template needed.

2. **`PerpendicularTemplateStrategy`** — the per-tile template is
   now exposed as a `JunctionStrategy` tried *first* for each
   cluster. Single-crossing clusters get the same fast path that
   made per-tile templates attractive originally.

3. **SAT strategies (`sat-surface`, `sat-1ug`, `sat-2ug`,
   `sat-full`)** — handle everything per-tile and corridor templates
   would, and more. The SAT search space includes every UG
   placement the corridor template would emit, so SAT can
   always produce at least as good a layout as the corridor template
   — and often better, because it can mix strategies (e.g. one long
   UG for some crossings, surface for others) the corridor can't.

The template was a pragmatic fix when there was nothing smarter
downstream. There's something smarter downstream now. The template
has inverted from asset to liability.

## Design

### What gets deleted

1. **`ghost_router.rs:1006-1215`** — the entire corridor-template
   pre-pass block. This is the main deletion.

2. **`corridor_handled: FxHashSet<(i32, i32)>`** — the tracking set.
   All downstream consumers (the `unhandled_crossings` filter at
   line 1420, the debug_assert at line 1430, the
   `pending_crossings` filter inside the cluster loop) become
   trivial: "all crossings are unhandled," so the filters collapse
   to direct uses of `crossing_set`.

3. **`RegionKind::CorridorTemplate`** region-kind emission. Keep the
   enum variant itself (compatibility with existing snapshots) but
   strip the emission sites.

4. **`segment_id` matches on `corridor:` prefix** — a couple of
   places in `ghost_router.rs` and `ghost_occupancy.rs` check for
   the `corridor:` prefix as a distinct claim class. They simplify
   or merge into the existing `trunk:`/`tapoff:` class.

### What stays untouched

- `PerpendicularTemplateStrategy` — still the fast path for
  single-crossing clusters. This is the true "per-tile template"
  survivor and pulls its weight.
- The SAT strategy ladder (`sat-surface`, `sat-1ug`, `sat-2ug`,
  `sat-full`) — the whole point.
- `cluster_adjacent_crossings` — already handles the adjacency
  pattern the corridor template was catching.
- `Unreleasable` / `Template` occupancy claims — still needed for
  per-tile-template tiles and solver-internal stamping.

### Expected change summary

Net diff: roughly **200-300 LOC deleted** from `ghost_router.rs`,
modest cleanup in `ghost_occupancy.rs`, snapshot compat notes in
`models.rs`. The RFP is predominantly *subtractive*.

### Alternatives considered and rejected

**Demote the corridor template to a post-SAT fallback.** Keep it,
but run it only on crossings the solver didn't handle. Rejected —
the SAT ladder already covers everything the template covers, so the
fallback would be dead code. If the user's long-term intuition is
"SAT can handle every shape," then demotion is a euphemism for
removal.

**Make corridor UG stamps releasable.** Keeps the template but lets
the solver override its stamps. Rejected — the template stamps eagerly
before SAT even sees the cluster, so "releasable" means "the SAT
encoder is handed a region with pre-stamped entities it can choose to
rewrite." That's strictly more complex than just letting SAT start
from scratch.

**Update `routed_paths` when the corridor template stamps UGs.**
Fixes the walker-veto stale-path problem we hit in Phase 1 of the
unified-belt-specs RFP but doesn't address the greedy-ordering
problem. Partial fix; skipped for the full removal.

## Kill criteria

1. **Capped fixture still caps.** If
   `advanced_circuit_iron_plate_trio_capped` still returns `None`
   after the corridor template is removed, the theory "the corridor
   template's boxing is the residual pin" is falsified. Revert and
   reopen the hypothesis search. This is the load-bearing outcome.

2. **E2E suite regresses by more than 1 test.** Baseline: 375 pass,
   23 ignored. More than one new failure means the corridor template
   was preventing a class of bugs the junction solver doesn't
   currently handle — stop, investigate the specific regression, and
   decide case-by-case whether to fix the solver or resurrect a
   narrowed corridor template.

3. **Paired passing fixture regresses.**
   `advanced_circuit_ret_plus_three_trunks` solves at cost 56. If it
   solves above 65, the SAT solver is producing noticeably worse
   layouts on common cases — means we've paid a quality tax for
   removing the fast path. Revisit.

4. **Solver runtime blows up.** Soft criterion. If e2e wall time
   increases by more than 3× (from ~3s to >9s), the SAT cost on
   what was previously trivial is no longer "low-ms per call" and
   the performance concern is real *now*, not later. Doesn't
   automatically trigger revert — it triggers a conversation about
   whether to start optimising the solver sooner than planned.

## Verification plan

Following the [layout engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Region-fixture suite**: both committed fixtures pass. The
  capped fixture should now flip from `expected.mode = "capped"` to
  `"solve"` with a reasonable `max_cost` ratchet (to be measured).
- **Full e2e**: `cargo test --manifest-path crates/core/Cargo.toml`.
  375 pass or better.
- **Browser check**: the motivating URL
  http://localhost:5173/?item=advanced-circuit&rate=5&machine=assembling-machine-3&in=coal%2Cwater%2Ccrude-oil%2Ciron-ore%2Ccopper-ore&belt=transport-belt
  loads cleanly. Junction at (21,161) shows SAT-solved zones, no
  leftover backwards-L of restricted tiles, no validation errors in
  the `(19..28, 159..164)` box.
- **Snapshot diff on a tier2 baseline**. Generate a snapshot of
  `tier2_electronic_circuit_from_ore` before and after the change.
  The snapshot should differ but the validator must agree both are
  correct. Confirms the corridor template was producing equivalent
  (not superior) layouts on common cases.
- **Runtime**: measure `cargo test` wall-clock before and after.
  Note the delta in the decision log regardless of direction — this
  is the baseline we'll optimise against later.
- **Clippy + WASM build**: clean.

## Phasing

This RFP is a single-phase deletion. No Phase 0 audit needed — the
touch points were already mapped during the unified-belt-specs audit.

### Phase 1 — remove and verify

- Delete the corridor-template block and its tracking.
- Simplify downstream filters and claim-kind checks.
- Run the full verification plan.
- Capture the new state of the capped fixture and either flip to
  `"solve"` with a `max_cost` ratchet, or report the new cap reason
  if kill criterion (1) fires.

## Relationship to other RFPs

- **[`rfp-unified-belt-specs.md`](rfp-unified-belt-specs.md)** —
  Phase 1 (single-tap spec unification) has landed and stays landed.
  Phases 2-4 (multi-tap, returns, cleanup) remain valid follow-up
  work but are independent of this RFP. The capped fixture becomes
  the joint success criterion: unified specs removed the handoff pin,
  corridor removal removes the stamping pin, both are needed.
- **[`rfp-veto-directed-growth.md`](rfp-veto-directed-growth.md)** —
  abandoned, Phase 2 code retained. The growth improvements remain
  live and will interact with the solver's increased workload
  post-corridor-removal.

## Decision log

- *2026-04-21 — RFP drafted after the unified-belt-specs Phase 1
  retrospective revealed that the spec-handoff pin was only part
  of the block; the corridor template's greedy stamping is the
  other half. Confirmed via git archaeology that the template
  pre-dates the SAT solver by two days and has been redundant since.
  Status: proposed, awaiting Phase 1 execution.*
