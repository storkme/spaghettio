# RFC-051: Cell composition integration (production path)

Registry: [`rfcs.md`](rfcs.md). Status: **Active — Phase A (lift) delivered; Phase B (candidate) next.**

## Summary

Convert RFC-048's cell-composition method from a test-only harness into
a **production layout path**: a flag-gated `CellComposedCandidate` that
competes in the decomposition search for solid linear/fan-out chains,
built from engine-generated cells, contract-derived ports, and
template-level corridors. The phase gate (set by the user's end-state
entry in RFC-048): **advanced-circuit-from-plates composed through the
production path at 0 validator errors, sim-verified at plan.**

Phase 1 delivered the existence proof — a composed EC@15/s factory runs
at exactly plan rate in headless Factorio, on a config the bus engine
refuses outright ([#336](https://github.com/storkme/spaghettio/issues/336)).
This RFC is the "integration RFC" that Phase 1's GO verdict commissioned.

## Motivation

- **The refusal class.** The bus engine's honest refusals (military@5/s
  stone-brick sideload wall, EC@15-from-plates, the (n,1) merge-tap gap)
  are configs where composition already measurably works. Integration
  turns "existence proof in a test" into layouts users get.
- **Warnings-aware competition.** Phase 1 showed composed layouts carry
  *fewer and better-understood* warnings than bus layouts on shared
  configs (6 sim-disproven attribution warnings vs the engine's
  unadjudicated set). The decomposition search already scores candidates
  by issue kinds — composition slots in as one more candidate, no new
  decision machinery.
- **The tripwire clock is running.** RFC-048's never-finishing tripwire
  (#361 fold) requires fixture-ladder coverage growth at every phase
  close-out, from Phase 1's baseline of 2 chains. Integration is how
  coverage grows without hand-composing every config.

## Design

### What moves where (kill-4 discipline: one layout stack)

The Phase-1 harness code (`crates/core/tests/cell_composition.rs`)
splits into production modules and thin test consumers:

- `src/bus/cells/extract.rs` — segment-crop extraction (`row:*` + poles
  + machines), contiguous-run port derivation with terminal coordinates,
  the lane-aware port contract (lanes ∈ {1,2}; sideload-into-port
  forbidden). Cells remain **engine-generated at layout time** — run the
  existing pipeline on a single-recipe solve, crop, done. No static
  cell-asset catalog: Phase 1's axis measurement ("a variant is a
  parameter, not a project") showed authoring is free; verification is
  the scarce resource (below).
- `src/bus/cells/compose.rs` — placement + corridor stamping for solid
  chains, generalized from the Phase-1 composer: cells placed
  left-to-right in dependency order, corridors from the template set
  ONLY (straight, corner, UG-hop, 2→1 splitter merge, splitter fan-out,
  pipe column). The sim-kit feed scaffolding (4-tile rig pitch, the 9
  columns of width per pair) does NOT move — production boundary
  attachment emits ordinary `boundary_inputs`/`boundary_outputs` records
  at the layout edge, same contract as bus layouts. A
  `calibrated_for_sim` mode keeps the rig-compatible geometry for
  verification runs, under a stated and **asserted invariant** (review
  fold, 2026-07-22): a production layout and its calibrated twin are
  **bit-identical in every non-boundary entity** — cell interiors,
  corridor topology, port attachment — differing only in the boundary
  scaffolding outside the ports. A parity test diffs the two entity
  sets minus boundary segments; verification provenance holds only as
  long as that diff is empty, so it fails the build, not a hope.
- Solver→cell rounding: per-chain ratio quantum from solver machine
  counts (e.g. 3 cable : 2 EC = 5 EC/s), K copies rounded up, headroom
  explicit in the candidate's metadata.

### Chain eligibility (Phase A/B scope)

A solve is composition-eligible when every recipe in the chain is
solid-only and the dependency graph is a tree with fan-out (an
intermediate may feed multiple consumers — AC-from-plates has cable
feeding both EC and AC, which is exactly why it's the gate; fan-IN
beyond 2→1 merges of same-item runs stays out of scope). Fluids enter
only as boundary pipe columns to a single consuming cell (plastic
class) — the oil mega-cell is RFC-048 Phase 3, not this RFC.

### Search integration

`LayoutOptions.cell_composition: Off | Candidate` (default **Off**;
flips to Candidate only after the gate passes, as its own decision).
Under `Candidate`, eligible solves generate a `CellComposedCandidate`
scored by the existing decomposition-search machinery (error/warning
kinds, area) against the bus candidates. No score biasing: if
composition only wins where the bus engine refuses, that is the honest
value statement (see kill 3).

### Verification tiers (what "pre-verified cell" means in production)

- Tier 0 (every layout, free): the 34 validator checks on the composed
  result.
- Tier 1 (per cell config, cached): a sim-verified registry — key →
  sim verdict — checked in as data, grown deliberately (CI cannot run
  the sim; goldens stay host-cache-relative per the standing rule).
  **The key includes a hash of the generated cell's entity list**
  (review fold, 2026-07-22), not just the config tuple (recipe, tier,
  modules, quality, rate quantum): cells regenerate from the live
  engine, so a config-only key would let row-template/placer/inserter
  changes silently decay "pre-verified" into "unverified with a stale
  verdict" — the exact drift class the method exists to kill. The
  geometry hash is self-maintaining: any engine change that alters the
  cell changes the key, and the verdict simply doesn't match until
  re-verified. Scoring may prefer sim-verified cells; absence is a
  warning-level note, not a refusal.

## Kill criteria

1. **Parallel-stack kill** (RFC-048 kill 4, inherited): if production
   integration requires cell-specific inserter/power/belt logic not
   shared with the engine and validators — stop.
2. **Corridor-machinery kill** (kill 5, inherited): if AC-from-plates
   needs corridor machinery beyond the template set (anything shaped
   like negotiated congestion or junction solving between cells) — stop.
3. **No-value kill**: if, on the fixture ladder, the composed candidate
   neither (a) produces a 0-error layout for at least one config the bus
   engine refuses, nor (b) reaches a strictly better **acceptance
   class** (fewer validation errors, or the same errors with a strict
   subset of warning categories) than the bus candidate on at least one
   config both paths handle — integration adds maintenance surface
   without value — stop and keep composition as a harness. *(Worded in
   acceptance-class terms because `CandidateScore` has no warnings term
   today — review fold, 2026-07-22. If Phase B adds one, it is shared
   scoring machinery for ALL candidates, logged as its own decision,
   and 3(b) may then be restated in score terms.)*
4. **Gate kill**: AC-from-plates composed at 0 errors + sim PASS within
   two working sessions of Phase C starting — else stop and escalate
   with the partial evidence.
5. **Tripwire** (standing, #361): fixture-ladder coverage at this RFC's
   close-out must exceed Phase 1's 2 chains, or the keep/kill escalation
   fires.

## Verification plan

- Full suite green at every phase; **zero golden re-blesses** (flag
  default Off guarantees bus-path bit-stability).
- Phase-A parity: the Phase-1 harness tests rewritten as consumers of
  the production modules must reproduce the Phase-1 results bit-for-bit
  (EC pair cells, EC@15 composition, plastic cell) — the lift is a
  refactor, proven by its tests.
- Differential scoreboard: composed vs bus on every eligible ladder
  fixture (errors, warnings by kind, area, refusals) — the kill-3
  evidence, committed to the RFC decision log.
- Sim verification: the AC-from-plates gate runs at plan in headless
  Factorio (post-#373 the fluid-consuming plastic cell re-verifies too,
  closing RFC-048 gate (a) fully).
- Layout-engine change class: local adversarial review required in
  addition to the CI bot, per house rules.

## Phasing

- **Phase A — lift.** Harness → `src/bus/cells/` behind the flag; tests
  become consumers; parity proven. No search integration yet. Parity
  means parity with the **post-review-fold** Phase-1 results: the two
  fragilities the #365 review caught were already fixed there (the
  splitter merge's approach geometry generalized with an
  `o2.y > o1.y + 1` assert and sim re-verified at plan; the petroleum
  column derived from the cell terminal with an adjacency assert) — the
  lift carries those asserts into production code, and enshrines
  neither original defect.
- **Phase B — candidate.** Eligibility check, `CellComposedCandidate`,
  scorer integration, differential scoreboard over the ladder.
- **Phase C — gate.** AC-from-plates (fan-out 2) composed + sim-verified
  at plan; kill 3/4 evaluated with the scoreboard; close-out with
  coverage datum and the flag-default decision.

## Decision log

- *2026-07-22 — Phase A delivered (same session as the RFC's merge).
  The harness lifted verbatim into `src/bus/cells/{extract,compose}.rs`
  (engine-as-generator bootstrap, segment-crop extraction,
  contiguity-port derivation, corridor stamping, both calibrated
  composers); `LayoutOptions.cell_composition` added (`Off` default,
  nothing reads it yet — plumbed through the 21 construction sites
  incl. wasm-bindings). Tests became consumers, and the two permanent
  gates now double as the parity proof with PINNED geometry: EC@15
  asserts 110×22 / 461 entities (the sim-verified artifact), plastic
  asserts 0 issues. The superseded east-feed composer + its two probes
  were dropped in the lift (pre-#363 orientation; findings preserved
  in the RFC-048 log). Panic policy documented in `extract.rs`:
  generation panics like the harness did; Phase B converts to `Result`
  at the candidate boundary where fail-soft is required. Suite
  888/0/45, goldens 8/8 (flag Off = bus path bit-identical, as
  designed), clippy 0, WASM builds.*

- *2026-07-22 — Session-side design review folded (verdict: ship after
  two one-paragraph fixes; both made, plus three smaller items). (1)
  **Registry key gains a cell-geometry hash** — config-only keys decay
  under engine evolution; the entity-list hash is self-maintaining. (2)
  **Calibrated-twin invariant stated and made assertable**: production
  and sim geometries share all non-boundary entities bit-identically,
  enforced by a diff test, or verification provenance is void. (3)
  Kill 3(b) reworded to acceptance-class terms — `CandidateScore` has
  no warnings term; adding one is shared machinery and its own logged
  decision. (4) Phase-A parity clarified as parity with post-fold
  Phase-1 results — both review-caught fragilities were already fixed
  and asserted on #365; the lift carries the asserts. (5) RFC-048
  registry row backfilled per the registry's backfill-on-touch rule,
  using byte-identical text to #365's own edit so the branches merge
  cleanly in either order.*

- *2026-07-22 — RFC authored (number claimed after fresh
  origin/registry + open-PR collision check). Scope decisions from the
  Phase-1 evidence: (1) **no static cell catalog** — cells generate at
  layout time (axis verdict: variant = parameter); the checked-in
  artifact is the sim-verified *registry*, not cell geometry. (2)
  **AC-from-plates is the gate deliberately one notch beyond Phase 1**:
  fan-out 2 (cable → EC and AC) forces the splitter fan-out corridor,
  the smallest structural step past the linear pair. (3) **Sim-kit
  scaffolding stays out of production geometry** — Phase 1's 2.48× area
  carried ~24% sim-rig overhead; production boundary attachment uses
  ordinary boundary records, and `calibrated_for_sim` preserves the
  verification path. (4) Flag default Off until the gate passes; the
  flip is its own logged decision. (5) Fan-in beyond same-item 2→1
  merges and all fluid interiors (oil complex) explicitly out of scope —
  RFC-048 Phase 3 territory.*
