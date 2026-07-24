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

- *2026-07-24 — Increment 2 DELIVERED (chain-fed mega inputs) and
  KILL-2 RESOLVED: NOT INVOKED. `mega_subgraph` now collects inputs
  produced by non-member chain specs as `plan.chain_fed` instead of
  refusing; the super-spec DECLARES them, so the generic
  consumer/fan-out/corridor machinery routes producers into the mega
  like any consumer cell. Geometry: chain-fed records take the
  adapter's DEEPEST lanes with west-edge entries at (0, lane_row) —
  one east-facing port per record, distinct approach rows, straight
  merges; residual crossings consolidate onto the (always-solid)
  chain-fed lateral, resolved by UG hops under any occupied column,
  with the orig-adjacent case retrofitting the crossing solid tail
  into a vertical UG pair. Consumer-less mega drains extend to the
  chain drain row (latent dead-end + rig-depth gap, exposed by PU
  whose export has no chain consumer). Honesty note: the bus no
  longer hard-fails PU@2 (junction geometry shifted under #408) — the
  class's bus-refusal win lives at PU@4 (unresolved junctions), which
  composes 0 errors (gate `mega_chain_pu4_resolves_bus_failure`).
  **Sim (after a harness fix — the scenario's fixed radius-12 chunk
  pregeneration truncated any fixture wider than ~768 tiles;
  build_blueprint creates no ghosts on ungenerated chunks, so the
  2704-wide PU@4 chain lost 2/3 of its entities and reported dead
  feed rigs/NO DATA; radius now derives from manifest dims):
  27498/27498 revived, CONVERGED, zero kit/fluid errors — the deepest
  chain ever measured, raw ore → plates → cable/EC → AC + oil →
  sulfur → acid → PU across 8 quantized copies. PU 3.17/s of 4.00
  (−20%)**, and the chain-fed machinery is NOT the bottleneck: AC (a
  chain-fed input) overproduces +4.1%, plastic +10%, PG −4.7%
  demand-limited; the deficit originates at the plate/sulfur cells'
  DECLARED inserter/lane bounds (#383 class — plates −21% at the
  source, sulfur output inserter declared 0.84/1.00). Not registered
  (registry records measured PASS baselines); registers when #383's
  template sizing lifts the declared bounds. With both named
  increments landed — chem (validator + fluid-proven sim) and PU
  (validator + converged sim) — the deferred kill-2 criterion
  resolves: the chain-integration machinery has real bus-refusal
  wins; the architecture stands.*

- *2026-07-24 — #408 adversarial review folds (APPROVE-WITH-NITS; the
  reviewer independently re-derived the reach semantics from game data
  and re-ran the full battery). The one MAJOR is SIGNED OFF as a
  deliberate trade: the supply-aware split-network check no longer
  flags a severed network whose fragments each retain a supply tile —
  the alternative flagged every legitimate multi-copy composed layout;
  the documented router failure modes it was built for still fire on
  their (supply-less) fixtures. Tracked for tightening in #409
  (component-count-vs-K or per-component supply-vs-demand). Folds: the
  adapter's PTG-tail length check now cites `UG_PIPE_REACH`
  (behavior-identical — deliberately one tile conservative to keep
  registered geometry stable), the candidate-space comment corrected
  (10^F not 5^F), the mouth-adjacency construction guarantee
  documented in the 'check loop, probe scripts got purpose headers.
  Review also strengthened the #406 attribution: the drifting stress
  golden's LAYOUT HASH is identical — drift is warning-category only,
  in belt validators this PR doesn't touch.*

- *2026-07-24 — Increment 1 (chem class) unblocked by a GAME-RULE
  falsification, not adapter work (#407): the chem5 sim TOTAL STALL
  traced to an 11-apart vertical PTG pair in the RAW engine block —
  the game's `max_underground_distance: 10` caps the UG-in/UG-out
  ENTITY distance at 10 (gap 9), belt `max_distance` semantics; the
  engine read it as gap 10 in BOTH the trunk stamper
  (`FLUID_UG_MAX_DISTANCE` +1 formula) and the validator
  (`MAX_PIPE_PTG_DISTANCE = 11`), so the validator blessed pairs the
  game never connects (same validator-blind class as #348/#364/#400).
  Fixed everywhere + F4 corrected; full suite green, registry hashes
  unchanged (no registered geometry had an over-reach pair), stress
  drift identical to the known #406 set. Post-fix sim: chem5 goes
  0.00 → 4.50/5.00 packs/s with ZERO fluid shortages (PG +5.6%) — the
  mega fluid architecture is sim-proven for the chem class; residual
  −10% is solid-side and forensically attributed: `kit_errors` and
  `fluid_errors` both EMPTY; the single bottleneck is the layout's OWN
  declared inserter-item-throughput warnings on the EC iron-plate
  feeds (declared cap 6.0/7.5 per copy predicts EC ≤ 12.0/s, measured
  12.60; plastic's consumption-limited signature is exact — AC 6.35/s
  × 2/craft = 12.7 ≈ 12.62 measured; cable/plastic throttle via
  full_output backpressure, sulfur/PG buffer). That bound is the #383
  template-sizing class owned by Lane B. INCREMENT-1 VERDICT: the
  chem class's bus-refusal win is REAL — compose 0 errors where the
  bus hard-fails, fluids sim-proven end-to-end, solids at the
  declared bound. NOT registered: registry entries record measured
  PASS baselines, and chem5 measures −10% vs plan; it registers when
  #383's template sizing lifts the declared EC bound. chem10 sim
  deferred (same solid bound would dominate; the reach fix is already
  exercised by chem5's tall block).*

- *2026-07-24 — Phase B DELIVERED: gate (b) MET — the flagship runs.
  `mega_subgraph` partitions fluid-touching specs (one weakly-connected
  subgraph, solid-only edges out, external-only inputs, v1 single
  consumer per exported item); the chain placer collapses it into a
  SUPER-SPEC whose placed form is the boundary-adapted mega block
  (generated from the parent chain's OWN member specs — counts match
  by construction, and multi-output subgraphs get every export laid
  out); mega corridors ride their own bypass rows from each drain head.
  Solid-only chains are bit-identical (registry gate). **Sim:
  mega-chain-ac2raw PASS — advanced circuits delivered at 2.00/s EXACT
  from fully raw inputs (iron ore, copper ore, crude, water, coal),
  45/45 machines working**; registered. The composed ladder is honest:
  AC@1/2/4 compose 0/0; above 4 the plastic sub-solve outgrows the
  engine's own oil layout and the candidate self-refuses. KILL-2
  STATUS: the bus covers AC-from-raw clean through 8/s (denser, wins
  the search — composition never displaces it in production), and the
  configs where the bus GENUINELY FAILS (chem-pack@5/10: junction
  crossings; PU@2–4: split sulfuric networks) are blocked by two NAMED
  adapter/eligibility increments, not by the architecture: (1) the
  chem block needs vertical PTG hops through foreign fluid bands (its
  water trunk is sandwiched — crude column west, petroleum row below;
  the joint dx-search + adaptive lane spacing added this phase widen
  the adapter but cannot clear column-adjacency without hops); (2) the
  PU class needs chain-fed mega inputs (its fluid subgraph swallows
  the PU spec itself, which consumes chain-produced EC/AC). Kill-2 is
  therefore NOT invoked — the frontier is reachable with named work —
  but the verdict is explicitly deferred to those increments: if BOTH
  fail, the chain-integration machinery has no bus-refusal win and the
  criterion applies with this sweep as its record. En route: the joint
  fluid planner exposed and fixed a residual terminus bug in the
  no-tail path (the registered plastic geometry pinned it; re-measured
  PASS and re-registered), and `required_copies` now exempts fluids
  from the belt quantum (no-op for solid chains).*

- *2026-07-23 — #403 review folds (no blockers; both findings are
  verification-rigor gaps, not live bugs): (1) the mirror-as-rotation
  wire encoding COLLIDES with genuinely South/West-unmirrored
  placements — the parser maps both to the engine's mirrored-North
  form, exact for our own round-trips but input-face-180°-wrong for
  such community imports (12/24 enumeration cases; overclaiming
  comments corrected, trade-off pinned by a parser unit test).
  (2) KNOWN GAP, Phase-C precondition: per-fluidbox IDENTITY (crude vs
  water) swaps sides under rotation-vs-mirror — the game itself warns
  about this exact confusion (FFF #394). Inert for basic processing
  (single fluid; both sim PASSes confirmed unexercised by decoding the
  fixtures' entity lists), but advanced-oil-processing on a mirrored
  refinery is UNVERIFIED for fluid identity until Phase C measures it;
  `verify_fluid_ports_transforms.py` checks positional SET equality
  only and cannot catch identity swaps.*

- *2026-07-23 — #400 FIXED, gate (a) fully MET: the first working
  refineries in the project's history. Three stacked defects, each
  found by the sim and fixed at its proper layer: (1) TEMPLATE — the
  fluid-only row's pole reservation bridged the strip with a UG pair
  whose mouths sat exactly ON the two input-port tiles; the strip is
  now continuous surface pipe (ports connected) and the two template
  tests re-pinned to the real port columns. (2) POWER — with a full
  strip no medium pole can reach a 5×5 center, so `place_poles` now
  reports uncoverable machine centers of FLUID-ONLY rows into the
  Phase-3a-ii reactive channel, and the substation-band targets accept
  machine centers for rows with no inserters; scoped twice by
  regression evidence (EC@20-from-ore golden caught eager substations
  on mixed rows; two AC-partitioned stress goldens caught center-driven
  bands on inserter-covered rows — mixed rows keep pre-#400 behavior
  bit-identically). The two AC-partitioned stress goldens re-blessed
  deliberately: they are the only golden-pinned refinery-bearing
  fixtures, and the new geometry improves pole slack (zero-slack 9→1).
  (3) ARTIFACT BOUNDARY (the #348/#364 class, third instance) — the
  engine's "mirror" models a front-back port flip, but the game's
  mirror flag flips LEFT-RIGHT: an exported (North, mirror) refinery
  still has inputs on the south in-game, so crude sat ON the intended
  port tiles and never entered. For the x-symmetric port layouts
  (refinery/foundry/cryo) the y-flip is tile-identical to a 180°
  rotation, so export encodes (direction+8, mirror:false) and the
  parser reverses it — engine geometry and all registry hashes
  untouched. The earlier direction-8 patch experiment failed because
  it KEPT mirror:true, which misdirected the first diagnosis toward
  the strip alone (recorded so the next reader distrusts single-factor
  experiments on compound defects). Sim: mega-plastic2 PASS (delivered
  2.20/s vs 2.00 planned, 4/4 working — the +10% matches RFC-048's
  known chem-plant planning conservatism), mega-sulfur2 PASS (produced
  2.00/s EXACT, 5/5 working, two-fluid adjacency-planned feeds). Both
  registered with world fields. The adapter also gained
  descending-tail joins en route (post-#400 raw trunk heads are PTG
  mouths whose sides don't connect — the adapter now descends past the
  band boundary until an honest plain-pipe join materializes, and its
  join predicate applies #400's own lesson recursively: plain pipes
  join on any side, PTGs only at their axis opening). Suite 906/0/52;
  goldens 9 ran / 0 drift post-re-bless; harness 44/44; WASM clean.
  Phase B (fluid-subgraph partition + AC-from-raw flagship) is GO.*

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
