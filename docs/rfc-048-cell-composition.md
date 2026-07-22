# RFC-048: Cell composition (city-block layout) — feasibility spike

Registry: [`rfcs.md`](rfcs.md). Status: **Spike — Phase 0 complete (GO); Phase 1 not started.**

## Summary

Explore whether the bus layout engine's tile-level routing can be
replaced — for the bulk of a factory — by **composition of pre-verified
cells**: fixed-footprint recipe blocks with contractual edge ports
("west edge, y=2: one lane of iron-plate in"; "east edge, y=4: one lane
of electronic-circuit out"). Layout becomes block-level place-and-route
(place cells on a coarse grid, route corridors between named ports)
instead of per-belt negotiated A\* with a junction-solver repair phase.
This spike is **feasibility only**: one recipe chain
(electronic-circuit from plates), hand-designed cells, hand-composed,
measured against the current engine on the same solve. No production
engine changes.

## Motivation

The 2026-07 architecture audit
([`architecture-audit-2026-07.md`](architecture-audit-2026-07.md) §3.2,
§6-A) found that the engine's failures have become **interaction
failures** — geometry vs belts vs power vs fluids — and that the repair
machinery keeps applying belt-level fixes to geometry-level problems:

- The junction solver can only grow a bbox and re-stamp belts; it can
  never ask the lane planner to swap columns or widen one gap. The one
  geometry retry is global, +1 tile, once.
- The 1-trunk-per-consumer-row split rule manufactures balancer shapes
  no template library can contain ((15,14), (12,7), (8,19) at
  utility@10/s); the merge-tap fix is built but parked (Phase 2 gated
  on priced work), and the rule was **confirmed retained** post-RFC-047
  (`lane_planner.rs:720-727`, "retained deliberately").
- Both attempts to *synthesize* the missing balancer templates failed
  (CP-SAT placement; Factorio-SAT overnight bake, 0/32 seeds — see
  `rfc-merge-tap-trunks.md`), while the **pre-baked** template library
  has quietly been the most reliable routing component in the system.

The asymmetry is the hypothesis: **pre-verified composable templates
work here; open-ended per-instance synthesis does not.** The balancer
library is cell composition at splitter scale. This RFC tests the idea
at recipe scale.

Concrete failing case to beat: `stress_electronic_circuit_35s_from_ore`
carries 4 belt-dead-end errors from a missing (4,9) balancer shape on
current main; utility@10/s sits at 46 errors. A cell-composed EC chain
has no (N,M) balancer *inside* cells and no cross-row junction solver
*between* cells — if the approach is sound, both error classes are
absent by construction, not repaired after the fact.

## Design

### The cell

A cell is `(recipe-set, machine-tier, module/quality config, rate
quantum)` → a fixed-footprint stamp:

- **Rate-quantized, not rate-continuous.** This is the dominant design
  constraint: a cell sized for 15 EC/s has different machine counts
  than one for 5 EC/s, so arbitrary rates can't share a footprint.
  Answer: tileable ratio cells repeated K times. For the spike chain
  (AM3, no modules, from recipes.json):
  - copper-cable: 0.5 s craft, 1 plate → 2 cable ⇒ 5 cable/s/machine
  - electronic-circuit: 0.5 s craft, 1 iron + 3 cable → 1 EC ⇒
    2.5 EC/s/machine
  - ⇒ the natural quantum is **1 cable machine + 2 EC machines =
    5 EC/s** per cell. EC@15/s = 3 cells; EC@2/s = one cell at 40%
    utilization (rate headroom inside a cell is fine — machines simply
    run below capacity).
- **Port contract.** Each cell declares named edge ports:
  `(edge, y, kind, item, direction, rate-ceiling)`, e.g.
  `("W", 1, belt, copper-plate, in, 15/s)`. The contract is the only
  thing the composer knows about the cell interior. Ports are also the
  validation seam: a cell is pre-verified once (existing 34 checks on
  the cell in isolation +, later, the headless in-game harness from the
  audit §8.3), and composition validates only port-to-port wiring.
- **Power.** Cells carry their own poles internally; inter-cell power
  is either per-cell EEIs (spike) or a corridor-level pole line
  (design question, Phase 1).

### The composer (spike scope: manual)

Solver output (machine counts) → round up to cell counts → place cells
on a coarse grid (spike: one column of cells, ports aligned) → route
corridors between connected ports. Corridors are several tiles wide,
carry whole belts/pipes, and cross only other corridors — never cell
interiors. The crossing problem at this granularity is small enough to
be template-driven (straight corridor, UG hop, or corridor-level
splitter tree — no negotiated congestion, no region growth).

### Cell generation: reuse, don't rebuild

Two sources, in order of preference:

1. **The existing engine as the cell generator.** Run today's pipeline
   on a single-recipe solve, crop the result, freeze it as a cell
   asset. Keeps exactly one layout stack; cells inherit every future
   engine improvement. (This is the bootstrap path — it also means the
   spike needs no new layout code for cell interiors.)
2. Hand-designed cells (for the ratio-group cells like 1-cable+2-EC
   that the engine can't emit today because it groups rows by single
   recipe).

Existing row templates (`templates.rs`, `placer.rs` RowKind) are
proto-cells at single-recipe scale — the spike should catalogue how
much of the port contract they already satisfy (input/output belt y's
are already computed; the audit's "templates return their geometry"
item is a prerequisite-shaped refactor).

### What this is NOT

- Not a replacement for the solver (unchanged; it emits counts as
  today).
- Not beaconed builds, not trains, not fluid-heavy chains in the spike
  (fluids are Phase 2+ questions; the port contract has a `pipe` kind
  reserved).
- Not a commitment to replace the bus engine. The spike's output is
  evidence for a go/no-go; the bus engine remains the fallback —
  *scoped by the 2026-07-22 end-state entry: fallback during catalog
  growth, with no chain class structurally excluded (see the decision
  log; the phase map for solids → oil mega-cell → catalog accretion
  lives there too, superseding this section's Phase-2 fluids
  assignment).*

## Kill criteria

1. **Catalog blow-up.** If covering the EC-chain rate bands used by the
   tier 1–4 fixtures (gear@15/s, EC@5/s, EC@15/s, EC@35/s-class)
   requires more than ~6 distinct cell variants, rate variability
   dominates and the approach collapses into today's per-instance
   templates — stop.
2. **Contract failure.** If the hand-composed EC@15/s (3× 5/s cells +
   external feed) cannot reach **0 validator errors** within the
   spike's manual-routing budget (~2 days of iteration), the port
   contract is wrong or insufficient — stop.
3. **Area blow-out without compensation.** If the composed EC@15/s is
   >2× the area of the current engine's EC@15/s layout AND shows no
   compensating win (warnings, failure modes, predictability), the
   robustness claim isn't real — stop.
4. **Duplication kill.** If cells cannot be produced by the existing
   engine + validators and instead require a parallel layout stack
   (separate inserter/power/belt logic) — stop; that doubles the
   maintenance surface the audit already flagged as drift-prone.
5. **Premise falsification.** If corridor routing for even the 3-cell
   EC chain needs machinery beyond orthogonal corridors + UG hops +
   simple splitter trees (i.e. anything resembling negotiated
   congestion or the junction solver), the coarse-granularity premise
   is false at the smallest possible scale — stop.

## Verification plan

- **Oracle:** the existing 34 validation checks, run over assembled
  cell compositions (cell interiors pre-verified once; composition
  re-validated end-to-end). Follow the CLAUDE.md verification protocol:
  snapshot inspection of composed layouts at suspect port coordinates,
  not just warning counts.
- **Fixture:** a new e2e-style test harness that stamps cells from a
  catalog and assembles `PlacedEntity` lists, so composed layouts flow
  through `validate()` unchanged. (Requires a Rust toolchain host — the
  RFC author's environment has none; Phase 0 is paper + asset design,
  Phase 1 lands where compilation is available.)
- **Comparison table:** current engine vs cell composition on the same
  solves — area, entity count, validation errors/warnings by category,
  trace-event failure signals (`JunctionGrowthCapped`, `GhostSpecFailed`
  should be *absent* from the composed path by construction).
- **Strong anchor (when available):** the headless in-game harness
  (audit §8.3) — cells are the natural unit for in-game pre-verification
  (verify 6 cells once, vs every layout instance).

## Phasing

- **Phase 0 (this spike, paper recon):** port contract spec; EC-chain
  cell catalog with footprints drawn; hand-composed EC@15/s layout on
  paper; comparison against current engine output. Deliverable:
  evidence appended to the decision log + go/no-go for Phase 1.
- **Phase 1 (only on Phase-0 go):** minimal catalog + stamper + manual
  composition harness behind a test-only flag; corridor routing by the
  coarse template set. Kill criteria 2/3/5 evaluated with real
  validator output.
- **Phase 2 (only on Phase-1 go):** integration RFC for solver → cell
  rounding, automatic cell placement, fluids, and the relationship to
  merge-tap (corridors still need taps — merge-tap is the candidate
  mechanism) and to beacons (a beaconed cell is just another catalog
  entry).

## Phase 0 findings (2026-07-22, picked up by the RFC-046/047/049 session)

**F1 — the ratio-cell math in the Design section is WRONG (falsified by
the solver).** "1 cable machine + 2 EC machines = 5 EC/s" under-supplies
cable 3×: 5 EC/s consumes 15 cable/s and one AM3 cable machine makes
5/s. The correct quantum is **3 cable + 2 EC machines = 5 EC/s**
(solver-confirmed: EC@5 solves to cable ×3.00, EC ×2.00). Cell
footprints below use the corrected ratio; kill criterion 1's variant
budget is unaffected (the quantum is still one cell shape, just wider).

**F2 — engine comparators frozen (kill 3).** EC@5/s from plates, AM3,
normal, defaults: **13×25 = 325 tiles, 112 entities, 0 errors / 4
warnings**. EC@15/s from plates: **the engine now REFUSES** — RFC-047's
late sideload check fires ("copper-cable 45.00/s exceeds per-lane
capacity 22.50/s on a sideload-fed single trunk (2 producers, no
balancer)"), i.e. the #336 (n,1)-merge-tap gap on a mainstream config
(no fixture covered it; pre-RFC-047 it built with silent physical
overloads). **The spike's primary comparison case is therefore not
"beat the engine's layout" but "exist where the engine honestly
cannot"** — stronger motivation than the RFC's original framing, and
kill 3's area comparison falls back to the EC@5 point plus per-cell
arithmetic (3 cells ≈ 3× the cell footprint + corridors vs 325×3-ish).

**F3 — the verification story upgrades: RFC-050's `spaghettio-sim` is
live** (blessed baselines landed same-day). Cells are the natural unit
for its boundary kit — feed W ports with tier-matched loaders (S=1) or
stack-inserter banks (S>1, measured 179–186/s on S=4 express), drain E
ports with count-and-clear chests, measure planned-vs-actual per cell
ONCE at catalog time. "Pre-verified cell" can mean sim-verified, not
just validator-verified, from the first catalog entry. (The harness's
inserter-direction discovery — every historical export ran backwards
in-game, fixed #348 — is also the strongest possible argument for the
spike's pre-verification premise.)

**F4 — port contract v2 (lane-aware; folds the PR #341 review's
load-bearing finding).** A port is
`(edge, y, kind, item, direction, lanes, per-lane rate ceiling)` with
`lanes ∈ {1, 2}` and one hard composition rule: **corridors connect to
ports only via lane-preserving forms** — straight feeds (B7), corners
(B11), or splitter outputs (S4); sideloading into a port is forbidden
(B8 halves the contract invisibly; the post-RFC-047 walker vetoes it).
A `lanes: 2` in-port promises both lanes arrive loaded; a `lanes: 2`
out-port promises the cell fills both (row bridges do this today —
templates.rs midpoint bridge). Rate ceilings are per-lane so stacking
composes multiplicatively later (per-lane × S), matching the engine's
capacity layer.

**F5 — corrected cell sketch (paper), EC ratio cell @ 5 EC/s:**
- Cable row: 3 machines (single-input template, height 7, width 9) —
  in: copper-plate 7.5/s, out: cable 15/s (both-lane via bridge).
- EC row: 2 machines (dual-input template, height 8, width 6) — in:
  iron-plate 5/s + cable 15/s, out: EC 5/s.
- In-cell connection: cable out-belt corners into the EC row's far
  input belt (lane-preserving, no trunk, no balancer, no sideload —
  the geometry class RFC-047 proved sound).
- Cell ≈ **11 wide × 17 tall (187 tiles)** incl. 1-tile port margins;
  ports: W iron-plate in (1 lane, 5/s), W copper-plate in (1 lane,
  7.5/s), E EC out (2 lanes, 2.5/s/lane). Power: internal medium pole
  pair (or per-cell EEI under sim).
- EC@15/s = 3 cells stacked vertically + 2 shared W feed corridors +
  1 E collection corridor ≈ **13×55 ≈ 715 tiles** vs the engine's
  refusal (and vs 3× the EC@5 engine footprint ≈ 975 tiles if it could
  scale linearly). Within kill 3's 2× bound against the linear
  extrapolation; catalog count stands at 2 variants (ratio cell +
  collection corridor template) — far under kill 1's ~6.

**Review folds (2026-07-22, PR #359):**
- *F1 addendum*: the corrected quantum makes the cell ~50% wider than
  the original sketch assumed — every area number derived from the
  1+2 design is discarded entirely, not adjusted.
- *F2 addendum*: with EC@15-from-plates unable to serve as a
  denominator, the approximate cross-check is EC@10/s **from ore**
  (236 entities, 34×18 = 612 bbox tiles; approximate — different
  input chain). And the sell is reframed: **cells win on warnings,
  not area** — engine EC configs carry 4+ residual warnings
  (inserter/demand-pull classes); a sim-verified cell carries zero by
  construction. That is the Phase-1 reviewer's headline metric.
- *F4 hardened*: the sideload prohibition is a **contract invariant**,
  not a walker-compatibility note — sideloading silently turns a
  `lanes: 2` promise into a `lanes: 1` reality with no detectable
  contract violation; a corridor router adding a sideload shortcut
  "because it fits" would be invisible until sim measurement. Ports
  are lane-counted; sideloading violates the count. (That it also
  trips the post-RFC-047 walker is defense-in-depth, not the reason.)
- *F5 clarified*: the 11-wide estimate assumes one 3-machine
  single-input row, which the existing engine templates DO emit
  natively (row templates take a machine count and stamp per-machine
  positions at 3-tile pitch; the placer's `max_per_row` governs the
  split) — so the reuse path (F3, engine-as-cell-generator) covers it
  without a new composite template. What Phase 1 must still verify:
  the CROPPED row keeps its shared input-belt geometry intact at cell
  boundaries (the belt stubs at the crop line become the port tiles).
- *Phase-1 gate addition*: the second sim-verified catalog entry must
  be a **fluid consumer** (plastic-bar class) — the harness's fluid
  boundaries are uncalibrated, and catalog time is the right forcing
  function; success converts Phase 2's fluids question from
  blocked-on-calibration to integration.

**Go/no-go: GO for Phase 1** (catalog + stamper + manual composition
harness behind a test-only flag), with the corrected ratio, the
lane-aware port contract, and sim-verification of the first two catalog
entries as the Phase 1 gate.

## Decision log

- *2026-07-22 — FIRST SIM RUN FAILED HONESTLY, root cause harness-side:
  the composed layout fed EAST-facing boundary inputs, and
  scenario.rs's non-south rig geometry is UNCALIBRATED by its own
  module doc — empirically the east feed rigs misassemble (1 of 9
  lanes fed, 50 feeder inserters waiting_for_space; #363 filed with
  the datum). The sim-state dump proved the COMPOSITION itself flowed
  correctly wherever fed: cable ran cell→corridor→splitter→EC-cell
  end-to-end (belts full), row-1 machines crafted to full_output.
  RESPONSE: composer reworked to the CALIBRATED orientation —
  horizontal pair stacking, north-edge feed columns cornering east
  per-pair (inner column to topmost port, no crossings; iron still
  UG-hops under the cable cell), south-edge drains. Calibrated EC@15:
  89×22 = 1958 tiles, 422 entities, 0 errors / 6 inherited warnings
  (pole-network stitching needed a nudge-not-skip spanning line —
  skipped poles broke the chain). Plastic cell reworked likewise
  (9×16, 0 errors / 0 warnings). Both sims queued behind the shared
  install lock.*
- *2026-07-22 — EC@15 composes at 0 errors (3 pairs, 29×47 = 1363
  tiles, 345 entities — 1.4× the engine's EC@5 linear extrapolation,
  inside kill 3's 2× bound, vs an engine that REFUSES the config; the
  only warnings are the 2 inherited row residuals ×3, zero
  composition-new). Boundary records wired into the composer →
  export_with_manifest produces sim artifacts (9 feeds / 3 drains).
  AXIS MEASUREMENT (gate c): the EC cell at AM2 regenerates as a valid
  cell (14×9, 4 machines, ports auto-derived at shifted positions) from
  the same call with one changed string — **a variant is a parameter,
  not a project**; marginal authoring cost ≈ 0, the real axis cost is
  sim-verifying variants actually used. Verdict: the end-state is a
  PLAN. FLUID CELL (gate a, second half): plastic-bar extracts with
  pipe segments intact; pipe-kind ports added to derivation; the
  composed fluid cell (coal belt + petroleum boundary pipe + out belt)
  validates at **0 errors / 0 warnings**. Sim runs queued behind the
  shared Factorio install lock (another session mid-run — retry loop
  standing).*
- *2026-07-22 — FIRST COMPOSED PAIR VALIDATES AT 0 ERRORS (milestone).
  One ratio pair (cable cell + corridor + EC cell, 29×14, 114
  entities): external feeds attach at port terminals, the EC iron feed
  UG-hops under the cable cell (express reach 8 = the cell width,
  exactly), the two cable outs merge via a single 2→1 fast-splitter,
  and the EC out extends to the boundary. Composer bugs found and
  fixed en route: ports must carry terminal X (multi-row interiors
  have per-row widths), splitters are ONE entity, feeds may not cross
  cell footprints (UG weaving is the corridor form for that).
  Residual: 2 warnings INHERITED from the engine-generated EC row
  (iron input inserter-attribution, the pre-existing #65-class) — an
  honest wrinkle for the "zero warnings" gate: generated cells inherit
  generation-time residuals. Disposition deferred to the sim step —
  measured delivery adjudicates whether the attribution warning is
  real (RFC-050's purpose); the gate reading of "0 warnings" will be
  resolved honestly against that evidence, not by quietly re-wording.*
- *2026-07-22 — Extraction probe: EC cell extracts CLEAN (8×8, 32
  entities, 3 ports — iron/cable W-in at y=0/1, EC E-out at y=7,
  midpoint bridge intact). Cable@15 extracts as TWO internal rows (the
  placer splits 3 machines 2+1), which is contractually fine — cell
  interiors are opaque — but broke naive per-SEGMENT port derivation
  (both rows share `row:copper-cable:*` segment names, so terminal
  picking collapsed across rows). Design decision: ports derive from
  CONTIGUOUS BELT RUNS within a segment, one port per run terminating
  at a cell edge; the cable cell therefore exposes two W plate-in and
  two E cable-out ports, and the corridor merges the two outs via a
  2→1 splitter (lane-preserving, allowed by contract F4). Multi-row
  interiors are inevitable at scale, so run-contiguity is the general
  mechanism, not a workaround.*
- *2026-07-22 — Phase 1 started (branch rfc-048-phase1; goal-driven).
  Environment verified: Factorio 2.0.76 cached from the RFC-050
  dogfood, spaghettio-sim invocable, export_with_manifest present.
  Harness home: crates/core/tests/cell_composition.rs (test-only,
  kill-4-compliant: cells generated by the live engine). First
  geometry probe finding: SEGMENT IDS ARE THE CROP MECHANISM —
  `row:*` segments are the cell interior; `trunk:/tapoff:/ghost:/
  merger:*` are precisely the bus machinery to shed (cable@15
  grows an output-merger cascade, EC@5 is nearly clean). Ports
  derive from the kept belts' terminal stubs. Open detail: the EC
  dual-input row's cable in-belt didn't surface in the edge survey —
  full-inventory dump next.*

- *2026-07-22 — End-state definition set (user steer, post-#359): the
  method is not "done" with structural exemptions. Revised target: NO
  chain class structurally excluded — the existing tier ladder
  (through the refinery tiers) reachable by composition, phased:
  solids with AC-from-plates as the Phase-2 gate; the oil complex as
  an opaque pipe-ported MEGA-CELL (fluid loops are cell INTERIORS —
  the boundary contract is simple even when the inside is stateful;
  gated on the harness's fluid calibration, which is why the Phase-1
  fluid entry is an enabler, not a side quest); high rates via
  pre-verified CORRIDOR/DISTRIBUTION cells (the (N,M) problem at
  corridor granularity is shape-chosen, so it stays template-driven —
  and RFC-046 stacking makes belt-count growth slow: one S=4 express
  corridor = 180/s). The two PERMANENT disciplines (contract rules,
  not exemptions): distribution stays fixed-shape templates (kill 5
  applied to corridors), and fluid loops never cross cell boundaries.
  The bus engine's role: fallback during catalog growth, not
  permanent owner of the hard chains.*
- *2026-07-22 — PR #359 review folded (5 observations, verdict "Ship
  it"): old-design area numbers discarded; EC@10-from-ore added as the
  approximate comparator with the warnings-not-area reframing; the
  sideload prohibition promoted to a contract invariant; the
  multi-machine-row width accounting clarified (native template
  capability; crop-boundary belt-stub question assigned to Phase 1);
  fluid-consumer second catalog entry added to the Phase-1 gate.*
- *2026-07-22 — Phase 0 executed by the picking-up session (see
  findings above): ratio-cell math corrected (F1, solver-falsified),
  comparators frozen incl. the EC@15 honest-refusal discovery (F2 —
  reported to #336), sim-harness verification woven in (F3), port
  contract v2 lane-aware (F4, folding the PR #341 review), corrected
  cell sketch + composed estimate (F5). Go decision recorded.*

- *2026-07-22 — opened as a spike. Motivated by
  `architecture-audit-2026-07.md` §6-A (the audit's highest-value
  untaken direction alongside the headless harness). Context checked
  post-RFC-047: the 1-per-consumer intermediate split rule is retained
  (`lane_planner.rs:720-727`), so the (N,M)-explosion pressure this
  RFC routes around is still live. Spike scope deliberately frozen to
  the EC-from-plates chain (2 recipes, solids only, AM3, no modules) —
  smallest case that exercises rate quantization, ratio cells, external
  feeds, and output collection.*
