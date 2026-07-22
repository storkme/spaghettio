# RFC-051: Cell composition integration (production path)

Registry: [`rfcs.md`](rfcs.md). Status: **Delivered — all phases complete (PRs #377 + Phase B/C); gate passed, kill criteria evaluated, flag default remains Off pending the flip decision.**

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

## Phase B/C close-out: kill-criteria evaluation and the flag decision (2026-07-22)

**The gate (kill 4): PASS, same-session.** AC-from-plates (fan-out 2:
cable → EC and AC; plastic external) composes through the production
auto-placer at **0 errors / 0 warnings** and runs in headless Factorio
at plan: **produced 1.00/s (−0.33%), 8/8 machines working, converged.**
EC@15-from-plates (the #336 refusal config) through the same path:
0 errors / 6 adjudicated warnings, sim **produced 15.0/s exactly
(+0.0%), 15/15 working.**

**Kill 1 (parallel stack): PASS.** Cells engine-generated; the
auto-placer is placement arithmetic + the crossing Router (deterministic
local span-3 UG hops — rows hop under registered columns, columns under
registered rows); validation and scoring fully shared.

**Kill 2 (corridor machinery): PASS.** The gate config used straight
runs, corners, UG hops, one 2→1 merge splitter, one 1→2 fan-out
splitter, and the south bypass — no negotiation, no growth, no junction
solving. The Router is a fixed local rule, not a search.

**Kill 3 (no-value): PASS via clause (a).** The differential scoreboard
(all numbers from one probe run, in the decision log): EC@15-from-plates
— bus REFUSES, composed delivers (now a permanent gate,
`cell_candidate_resolves_ec15_refusal`, asserting BOTH directions:
flag Off preserves the refusal, flag On resolves it). Clause (b)
honestly untested as worded: EC@5 shows fewer composed warnings (2 vs
4) but the strict-category-subset comparison wasn't run. Where both
paths succeed, the bus wins on area (composed 1.5–3.1×) — recorded
without spin; the unbiased scorer will pick the bus there, which is
exactly the designed behavior.

**Kill 5 (tripwire): coverage GREW.** Phase-1 baseline 2 chains → the
auto-placer now composes gear@15, EC@5, EC@15, AC@1, AC@2 (three
distinct chains) plus the hand-composed plastic geometry; EC@30
refuses honestly (3 out-runs exceeds the merge cap — a known, named
limitation, not a silent gap).

**The flag decision: `Off` stays the default, deliberately.** Under
`Candidate` the unbiased scorer would only ever pick composition where
the bus refuses or fails acceptance (composed density is strictly worse
where both succeed), so a flip is strictly additive — but the flip
waits for: (1) the browser eyeball step of the house verification
protocol on composed layouts, (2) wasm/web flag plumbing, (3) the
Tier-1 sim-verified registry. Flipping is its own logged decision, with
the user.

**Deferred, honestly:** the geometry-hash sim-verified registry (Tier 1)
is designed but not implemented — today's catalog is small enough that
this RFC's decision log IS the registry (four sim-verified configs, all
listed with their measurements). The mechanism becomes load-bearing at
flag-flip or when the catalog outgrows the log; implement it then.
EC@30's 3-run merge and ratio quantization (K>1 cells) are named
follow-ups.

## Decision log

- *2026-07-22 — K-quantization + #383 ROOT CAUSE (the premise
  falsified mid-implementation). Ratio quantization landed in
  `compose_chain`: K identical side-by-side copies at 1/K rate
  (`required_copies`), copies generated once and stamped K times,
  producer→consumer matching restricted within a copy, per-copy segment
  suffixes, bypass rows reused across copies (disjoint x), K_MAX=12.
  K=1 is bit-identical to the pre-quantization placer — proven by the
  registered chain-ac1 hash surviving unchanged. **The plan's premise
  died on first sim contact**: quantum=15/s ("the measured-exact
  Phase-1 rate") produced ec15 K=3 at −23.7% — WORSE than K=1's −8%.
  Belt-dump forensics + tile diff against the hand-composed pair showed
  identical cell geometry, iron arriving on both lanes, and the east
  machine's single long-handed iron inserter delivering ~1.2/s against
  a 2.5/s need — exactly the validator's inserter-item-throughput
  warning. The hand-composed K=3's "15.0/s EXACT" and the K=1 fixture's
  earlier PASS both predate #378 (tech-state parity: the sim now forces
  bonuses to the layout's declared inserter_capacity=0); the PASS-era
  report lacks the realized-bonus fields entirely. So: the warnings
  were RIGHT under declared tech, the "sim-adjudicated conservatism"
  verdict was a researched-bonus artifact, the package-2 log's
  splitter-saturation-loss speculation is retired (fast and express
  measured identical deltas), and small rows CONCENTRATE the
  per-machine inserter deficit rather than fixing it. Quantum reshaped
  to 45/s = express capacity — a physical cap, not a quality knob:
  quantization now activates only where the placer previously refused,
  so every K=1 geometry (ec15, ec15-ore, ac1, ac2, mil5-plates) is
  bit-identical to the flip package. Verification: ec30 K=2 (each copy
  ≡ the ec15 K=1 shape) predicted ≈−8% and measured **−8.0% exactly**
  (WARN, 28/30 working) — the deficit is per-row, K-invariant, and
  belongs to RFC-049 Phase 3 (#381, in flight), after which cells
  regenerate with honest inserter counts, every registry hash trips,
  and ec15/ec30 get re-measured for registration. Scoreboard: ec30
  REFUSED→140×21 0 err/12 warn; ec60 REFUSED→280×21 0/24 (bus
  validation-fails there); mil5-ore goes K=2 (stone feed 50/s exceeds
  the per-copy feed cap) and its Router-class overlaps persist (8, was
  5 at K=1) — still the named next target. No registry additions: the
  registry carries measured-at-plan only. **Belt-tier guard**: the
  eligibility lift exposed a latent flip-era hole — composed corridors
  are express-only, and an eligible chain whose bus path fails under a
  sub-express `max_belt_tier` would have won with express corridors,
  violating the tier-is-a-user-constraint rule. The candidate now
  refuses under any sub-express cap (gate:
  `cell_candidate_respects_belt_tier_cap` proves flag-inertness there);
  tier-parameterized corridors (quantum = allowed tier's capacity,
  tier-matched belt entities) are the followup if tier-capped
  composition is ever wanted. Stress goldens under the lifted
  eligibility: 9 ran, 0 drift (bus wins all blessed fixtures on
  density; the canonical `stress_` check-mode run).*

- *2026-07-22 — Coverage expansion (package 2, follows the flip). Caps
  lifted: n-run MERGE CASCADES (2→1 splitter chains, below-approach
  corner per stage) and FAN-OUT TREES (1→2 splitter chains) replace the
  fixed 2-run/2-consumer limits; Router hops now cluster crossings
  closer than 3 tiles (independent per-column hops shared tiles);
  vertical lanes allocate per bypass edge with strips sized by edge
  in/out-degree (sizing by a slot's own fan-out under-counted ascents).
  Corridors upgraded to EXPRESS with an honest eligibility cap: any
  produced item over 45/s refuses ("run matching unimplemented") — this
  replaced a silent physical overload (ec30's 90/s cable on a 30/s
  belt) with a named refusal, and it also retro-tightened ec15's
  corridor (45/s on red was over per-lane capacity; express is correct
  and the geometry was re-sim-verified after the change). **New
  scoreboard rows: EC@15-FROM-ORE — furnace cells work — composes at
  0 errors / 14 warnings where the bus refuses** (270×22; the first
  composed chain with smelting). mil5-ore (9 specs, the scaling-wall
  fixture) is 5 overlap errors from clean — the residuals are the same
  local crossing class, named as the next target. mil5-plates carries
  bus-side validation errors too (both paths fail it today). ec30/ec60
  refuse honestly at the corridor cap; K>1 ratio quantization and full
  run matching are the follow-ups that would lift it. **SIM-CAUGHT
  SATURATION DEFECT (validator-blind class):** the express upgrade made
  ec15's merged corridor run at exactly 45/45 = 100% capacity — 0
  validator errors, and the sim measured produced −8% (real belts lose
  a few percent through splitters at saturation; the earlier
  fast-corridor PASS at 15.0 exact is retro-suspect for the same
  reason — its window likely rode buffer drain). A 2×2 balancer
  run-matching fix was designed and implemented, then REMOVED as
  falsified-for-this-fixture: the K=1 EC cell is a single 6-machine
  dual-input row with ONE cable port, so the 2×2 case never triggers
  and the −8% persists through the mandatory merge (identical delta on
  both corridor geometries). The deficit sits between a 2-run 45/s
  producer and a single saturated port — root cause open (#383 carries
  both sim reports and the candidate hypotheses: saturation loss vs
  #343-class input-inserter limits). Registry consequence: chain-ac1
  re-registered (PASS at −0.33%); chain-ec15's entry REMOVED — the
  refusal-resolution gate (validator-level) stands, but measured-at-
  plan is only claimed for geometries the sim passed; the K=3 pair
  topology (Phase 1) remains the measured-exact form and K>1
  quantization is the likely fix.*

- *2026-07-22 — Phase B/C delivered (same session as Phase A). The
  chain auto-placer (`cells/chain.rs`): eligibility (solid
  tree-with-fan-out, fan-out cap 2, one producer per item),
  K=1 cells at spec rate × count, dependency-ordered west→east
  placement, the two-registry crossing Router, early-jog corridors
  (jog to the target port row in the producer's gap — where all rows
  provably end before the splitter — so same-y row collisions are
  structurally impossible), one feed column PER PORT (multi-row cells
  taught this: a single-port find left second-row machines unfed and
  belt-flow-reachability caught it), fan-out splitter + south bypass.
  `CellComposedCandidate` competes unbiased in the decomposition
  search, catch_unwind fail-soft, flag-gated. Differential scoreboard
  (single probe run): gear15 bus 184 tiles/0/0 vs composed 462/0/0;
  ec5 bus 325/0/4 vs composed 1008/0/2; ec15 bus REFUSED vs composed
  1428/0/6; ec30 bus 1624/0/18 vs composed REFUSED (merge cap); ac1
  bus 754/0/0 vs composed 1349/0/0; ac2 bus 1189/0/0 vs composed
  1767/0/0. Sims: chain-ec15 15.0/s exactly, 15/15 working; chain-ac1
  1.00/s (−0.33%), 8/8 working — both PASS, converged. Suite
  889/0/48, goldens 8/8 (flag-Off inertness), clippy 0.*

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
