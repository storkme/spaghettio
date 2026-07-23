# RFC-052: Oil mega-cells — fluid subgraphs as uncropped composed units

**Status**: Design (circulated for review)

## Summary

Extend chain composition (RFC-051) past its solid-only boundary by
collapsing a chain's fluid-touching specs into ONE **mega-cell**: the
engine's own layout for the fluid subgraph, used **uncropped** as a
single composable unit. Fluids never cross cell boundaries — they live
inside the mega-cell and at the layout's outer boundary records (crude,
water), both of which the sim can already calibrate (#364; the plastic
cell PASSED at 2.20/s). Between cells, everything stays solid belts and
the existing corridor machinery. Flagship target: **advanced-circuit
from fully raw inputs** (iron ore, copper ore, crude oil, water, coal)
— smelting cells + an oil mega-cell + cable/EC/AC cells, composable at
rates where the bus refuses.

## Motivation

- `chain_eligible` refuses any fluid ("solid-only Phase B"), so every
  oil-touching chain falls back to the bus alone. The bus handles the
  oil ladder cleanly at LOW rates (tier 3–7 all SOLVED) — the refusal
  frontier is HIGH-rate configs, exactly where solid composition
  already earns its keep (ec15/ec30/ec60, mil5).
- The generator half of the work is already done and measured: the
  engine lays out plastic-from-crude at 0 errors / 0 warnings (20×19,
  probe 2026-07-23), and the hand-composed single plastic cell measured
  at plan in the sim (RFC-048 gate (a), post-#364).
- RFC-048 explicitly deferred fluid interiors to a Phase-3-shaped arc;
  this is that arc, scoped to what the evidence supports.

## Design

### The mega-cell is UNCROPPED — decided by measurement, not taste

`extract_cell`'s segment crop (keep `row:*` + poles + machines) is
load-bearing for solid cells and **fatal for fluid complexes**: on
plastic-from-crude it sheds the petroleum trunk between the refinery
row and the chem row (22 trunk entities, 10 of them pipes), leaving a
"cell" with orphaned petroleum-IN ports, a severed petroleum-OUT, and
spurious W-edge "crude ports" derived from per-refinery pipe stubs
(probe 2026-07-23, decision log). The internal plumbing IS the cell.
So a mega-cell is the **whole engine layout** for the fluid subgraph:

- Generated via `generate_cell_layout`-style bootstrap (bus pipeline,
  composition forced Off) on the subgraph's terminal solid target(s).
- Attached via the layout's own `boundary_inputs`/`boundary_outputs`
  (probe: exactly `coal` + `crude-oil` for plastic; `water` +
  `crude-oil` for sulfur) — fluid feeds as pipe columns (the
  `compose_plastic_calibrated` idiom: column derived from the terminal
  pipe, adjacency-asserted), solid feeds as belt feed columns exactly
  as today.
- Its solid outputs are ordinary out-ports for the chain corridor
  machinery; no new corridor kinds.

### Chain integration: the fluid-subgraph partition

Eligibility extends instead of refusing outright: a chain is
mega-eligible when its fluid-touching specs form **one connected
subgraph** whose edges to the rest of the chain are all SOLID items.
The subgraph collapses into a single super-spec (inputs: the subgraph's
external inputs; outputs: its terminal solids) and the chain placer
treats it as one slot — sized by a sub-solve of the subgraph at the
chain's rate share. Multiple disconnected fluid subgraphs refuse in v1
(named reason; N-mega-cells is future work). Quantization applies to
the super-spec's SOLID outputs exactly as to any spec (fluids ride
pipes, which in Factorio 2.0's segment model have no per-pipe
throughput falloff — connectivity and isolation are the constraints,
not rate).

### Fluid isolation

Pipes merge with ANY adjacent pipe, so cross-network adjacency is the
hazard class. The mega-cell is internally self-consistent (the engine's
own pipe-isolation validation passes), neighboring cells are belt/
machine-only, and the only new pipes are the boundary feed columns.
Design rule: fluid feed columns keep the same ≥4-tile pitch as belt
feeds and the composed layout must pass the pipe-isolation validators
— gated, not assumed.

### Two rungs, deliberately

1. **Basic complex** (this RFC's deliverable): refinery →
   chem-plant chains via basic-oil-processing (the solver's own pick
   for plastic/sulfur — single fluid intermediate, no cracking).
2. **Advanced complex** (Phase C, deferred): advanced-oil-processing
   with heavy/light/petroleum + cracking + lubricant (FRF/USP
   territory). Byproduct balance is solver-side and already done
   (net-flow); the layout question is whether the uncropped approach
   scales to the 3-fluid-output row geometry. Not started until the
   basic rung's gates pass.

## Kill criteria

1. **Generic-attachment or bust**: the mega-cell must attach through
   the generic boundary machinery (feed columns + drains + the
   existing Router). If plastic-from-crude needs per-recipe
   hand-composition beyond what `compose_plastic_calibrated` already
   established as reusable idiom, kill — the point is the generic
   path.
2. **Value-existence**: there must be at least one reachable config
   where the composed chain resolves a BUS refusal at 0 validation
   errors (the ec15-of-oil). If the bus already covers the whole
   reachable oil frontier at every rate the sub-solve supports, record
   the scoreboard and kill — a capability no-op is not worth the
   machinery.
3. **Sim gate**: the flagship fixture measures at plan in the honest
   world (declared 0), or its deficit is attributed to a named
   engine-wide bound (#385/#381-class) that solid chains share — a
   NEW fluid-specific deficit class kills until diagnosed.
4. **Isolation**: pipe-isolation and fluid-port validators clean on
   every composed fixture. One cross-network merge = stop.

## Verification plan

- **Gate (a)** — standalone mega-cell: plastic-from-crude composed via
  the generic path at 0 errors / 0 warnings, sim-measured at plan
  (declared 0, honest world), registered in cell-sim-registry with
  world fields.
- **Gate (b)** — chain integration: AC-from-raw (iron ore, copper ore,
  crude, water, coal) composes at 0 errors where the bus refuses
  (rate chosen by scoreboard sweep); sim-measured; the mega-cell's
  registry entry covers the composed-in-chain geometry.
- Differential scoreboard rows for plastic/sulfur/AC-from-raw at a
  rate ladder (bus vs composed), single-run quoted.
- Full suite + canonical goldens + clippy + WASM as always; K=1
  bit-identity of existing solid chains enforced by the registry gate
  (mega-cells are additive machinery — solid-only chains must compose
  byte-identically).

## Phasing

- **Phase A** — mega-cell generation + generic boundary attachment;
  gate (a).
- **Phase B** — fluid-subgraph partition in eligibility + placer;
  flagship AC-from-raw; gate (b); scoreboard sweep for kill 2.
- **Phase C** (deferred) — advanced-oil complex (cracking, lubricant).
  Own go/no-go on Phase A/B evidence.

## Decision log

- *2026-07-23 — Phase A: validator half of gate (a) MET; sim half
  BLOCKED by a discovery bigger than the phase. Delivered:
  `cells/mega.rs` (`compose_mega_calibrated` — uncropped engine
  layout + generic boundary re-pitching adapter with per-record band
  lanes and ADJACENCY-AWARE fluid paths: the sulfur fixture
  immediately proved naive tails merge foreign fluid networks, so
  fluid feed routes plan against a fluid-occupancy map, shift their
  tails sideways, and join their own trunk by adjacency — refusing
  loudly when no isolation-safe path exists). plastic@2, plastic@5,
  sulfur@2 all compose 0 errors / 0 warnings
  (`mega_cell_plastic_from_crude_zero_issues` gates it). En route the
  harness could not measure fluid INTERMEDIATES at all (scenario Lua
  crashed on `get_input_count("petroleum-gas")`) — fixed by routing
  fluid-only prototype names to fluid production statistics. Then the
  sim delivered the real finding: the FIRST refinery measurement ever
  (no blessed baseline contains one) shows crude never enters the
  refineries — the trunk's UG hops sit exactly on the input-port
  tiles while its connector pipes sit on non-port columns; the
  `fluid_ports.rs` table is RIGHT and the trunk stamper contradicts
  it; the engine's own validator accepts what the game rejects
  (#348/#364 validator-blind class; direction-flip experiments ruled
  out export orientation). Filed as #400 — oil-ladder-wide (chemical
  pack, FRF, USP), blocks this RFC's sim gates, and re-litigates
  OIL_MIRROR's "in-game-validated" comment. No registry entries
  (measured-at-plan only; both mega fixtures FAIL honestly until
  #400).*

- *2026-07-23 — RFC authored (number claimed after fresh origin/main
  registry + open-PR collision check). Scoping probe
  (`debug_oil_cell_probe`, local example): (1) the solver picks
  basic-oil-processing for plastic/sulfur-from-crude — the basic rung
  has NO cracking complexity, so it is genuinely reachable now;
  (2) engine layouts for it are small and clean (plastic 20×19 and
  35×19 at 2/s and 5/s, sulfur 25×20 — all 0 errors / 0 warnings);
  (3) `extract_cell`'s crop sheds the petroleum trunk (22 entities, 10
  pipes) and derives nonsense ports from refinery pipe stubs — hence
  the UNCROPPED mega-cell decision, made on measurement; (4) raw
  boundary records are clean and small (coal+crude / water+crude),
  matching the calibrated fluid-feed idiom the sim already PASSED
  (plastic 2.20/s, post-#364). Deferred by decision: advanced-oil
  complex (Phase C), multiple disconnected fluid subgraphs, fluid
  corridors between cells (explicitly rejected — isolation risk with
  no rate payoff under 2.0's segment model).*
