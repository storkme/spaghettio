# RFC-052: Oil mega-cells — fluid subgraphs as uncropped composed units

**Status**: Landed (Phases A/B/C delivered; close-out 2026-07-24 — see the status ledger and the decision log's close-out entry)

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

- *2026-07-25 (#438) — **the parked registrations resolved: chem5 IN,
  PU@4 OUT.** The close-out below made registration of chem5/PU4/USP2
  conditional on "#383 lifting the smelter bound". #383 resolved (root
  cause: an input-side long-handed bind at hand 1, cleared by #431's L2
  default), so all three were re-measured at the L2 default on Factorio
  2.0.77. Outcome is split, and the split is informative:
  **chem5 PASSES at plan — 5.00/5.00 EXACT** (delivered 5.07/s, 172/172
  machines working, converged) and is registered as the first
  L2-blessed entry. It turns out chem5 never carried the #383 deficit
  class at all; blocking it on #383 was over-cautious, not wrong.
  **PU@4 FAILS at −27.3%** (2.91/4.00) and is NOT registered — the
  registry takes measured PASS baselines only, so a failing fixture
  stays out rather than entering as a warned entry. Filed as #437. Note
  its first attribution (sulfur chem-plant output inserter COUNT) was
  DISPROVEN on investigation: the chain replicates the mega block K=8,
  so each replica's sulfur plant carries 0.25/s not 2.00/s, and all 64
  inserter warnings are a validator artifact — `effective_rows` is empty
  on composed layouts, so utilization falls back to `count/ceil(count)`
  and every replica is charged the entire chain's demand. The −27.3% is
  real but currently unattributed. **USP@2 not yet re-measured.**
  Registering chem5 also required extending `cell_registry_hashes_current`,
  which correctly refused the entry: that gate re-derived every chain
  entry at a hardcoded L0, so the blessed capacity is now a per-config
  field (pre-#431 entries are L0-geometry baselines; post-flip entries
  are real L2 geometry).*

- *2026-07-24 (#434) — `mega_chain_usp2_resolves_bus_failure` moved to
  `#[ignore]` (opt-in). This is a PERFORMANCE opt-out, not a weakening
  of the gate: the test still passes (re-verified opt-in, 0 errors),
  and the two in-loop mega gates it sits beside — `mega_chain_chem5`
  and `mega_chain_pu4` — keep the mega path covered on every run. USP's
  10-member oil complex generates crossing zones absent from the baked
  `sat-zones.bin`, so it re-solves them live every run (>6 min under
  cold-cache 16-way contention; its sibling chem5 does the same class
  of work in 0.67s purely because ITS zones are cached). It alone held
  the `cell_composition` binary at ~1378s. Trade recorded per the #434
  review: the opt-in also drops it from the default CI lane; the
  cheapest restore-to-loop is to bake its zones into `sat-zones.bin`
  (the chem5 route), NOT to un-ignore it as-is. Supersedes this log's
  earlier "permanently gated" framing for USP@2 specifically (the gate
  stands; only its run cadence changed).*

- *2026-07-24 — CLOSE-OUT (#421 merged). All three phases delivered;
  every kill criterion resolved without firing: (1)
  generic-attachment held — the mega attaches through boundary
  records, the generic corridor machinery, and the (new) fan/west-
  entry idioms, no per-recipe hand-composition; (2) value-existence
  proven three times over (chem5, PU@4, USP@2 — each composes 0
  errors where the bus hard-fails, permanently gated); (3) sim-gate
  escape clause satisfied at every rung — deficits attribute to the
  #383 solid declared-bound class shared with solid chains, and NO
  fluid-specific deficit class ever appeared (kit and fluid errors
  empty in every measurement); (4) isolation clean on every composed
  fixture. Registration of chem5/PU4/USP2 waits on #383 lifting the
  smelter bound (registry records measured PASS baselines).
  Remaining tracked threads: #423 (pitch-1 splitter-passthrough —
  the one engine class Phase C worked AROUND rather than through),
  #409 landed independently and the flagship re-verified against it,
  and the harness drained-counter window semantics oddity noted in
  the #421 review. Phase C's answer to the RFC's own question: the
  uncropped approach scales to the advanced complex, by
  measurement.*

- *2026-07-24 (flagship) — **USP@2-from-raw COMPOSES AT ZERO ERRORS**
  (2232×193, 48k entities; gate
  `mega_chain_usp2_resolves_bus_failure`). The crossing-zone probe
  resolved the residual 18 through three engine fixes: (1) tapoff
  splitters SURVIVE zone release when an interior boundary anchors on
  them — the #295 prose ("SAT models splitters as fixed structure and
  never re-stamps them") was implemented for balancer segments only;
  released-but-uncovered splitters were dropped by the Step-6 retain,
  leaving the trunk hole (prune_dangling was probed FIRST per the
  standing lesson and exonerated — its drops were other candidates'
  tiles; the killer was the release/retain pair). Zones that route
  AROUND a splitter (no interior boundary) keep the release,
  byte-identical. (2) merger hops TIER-ESCALATE within the user's
  belt cap (alternating blocked columns at 1-tile gaps are unhoppable
  at yellow reach; the entrance conversion accepts any surface-belt
  tier and refuses loudly-safely instead of mutating row machinery —
  fulgora's latent case). (3) splitter-tap SPACERS are a
  LayoutOptions opt-in set only by compose_mega_block: mega
  sub-solves pack every trunk span from y=0 so tap splitters land on
  live neighbor trunks and the zones don't bridge them; PU@2-AM2 and
  EC35-from-ore have the same overlap but their zone machinery
  bridges it and the spacer's geometry shift broke them — hence
  opt-in, main-line byte-identical (the pitch-1 splitter-passthrough
  class remains tracked here). Process note: an interim "suite green"
  read was VOID — a test target failed to compile and printed no
  result lines; re-verified from one clean invocation (the
  single-run-counts lesson generalizes to compile failures).
  Env-gated forensic tracing (SPAGHETTIO_MEGA_DEBUG) kept in the
  adapter and SAT prune (failure/prune paths only; the merger's
  per-hop probe removed after use, #421 review). **Flagship sim verdict
  (48,366/48,366 revived, zero kit/fluid errors, 394 machines
  working — the deepest measurement ever, 21 items)**: USP 0.84/2.00
  at the ceiling, PRODUCED-rate trend still rising at cutoff
  (0.70→0.88 over the final windows, from the produced-counter
  deltas — the drained counters are flat and their window semantics
  are unresolved, #421 review), attributed by time-series forensics
  to copper-plate supply as the ROOT constraint: it settles at
  ~70/s vs 99.7 planned (−30%) — the #383 smelter-cell
  declared-bound class (PU4's plates were −21%; USP@2's per-copy
  copper rates are higher). PG/EC deficits (−27%/−9%) read as
  demand-coupled cascades of that root, not independent bounds: the
  oil complex DEMONSTRATED capacity above plan during the buffer-fill
  transient (PG 240/s vs 203 planned; steady state is below plan
  under the throttled demand), and the 2 fluid-starved-census
  machines are the cracking plants during the demand transition.
  Single-root framing is the parsimonious read, not a proof. Kill-3's escape clause SATISFIED: deficit
  attributed to the named engine-wide solid bound solid chains
  share; NO new fluid-specific deficit class. Not registered
  (registry records measured PASS baselines); registers when #383's
  template sizing lifts the bound. Phase C architecture verdict: the
  uncropped approach SCALES to the advanced complex — the RFC's own
  question, answered by measurement.*

- *2026-07-24 (latest) — residual-18 diagnosis SHARPENED; one fix
  attempt made and REVERTED. The trunk emitter's silent
  `hard-tile continue` was bridged with in-segment South UG pairs —
  scoped first to all hard tiles (regressed two e2e baselines:
  downstream machinery like crossing zones owns some of those
  skips), then to splitter-second-tiles only (still: mouths land on
  tap rows/own hard tiles → asymmetric stamping → unpaired inputs;
  PU@2-AM2 baseline stayed red; block errors shifted 6→8 not
  improved). REVERTED cleanly — baselines green again. The sharpened
  truth: at PITCH-1 trunks with trunks on BOTH sides, a tap-off
  SPLITTER's second tile has no hostable position at all — the
  horizontal interactions (feed rows crossing neighbor columns) are
  already solved by the crossing zones; only the 2-tile splitter
  entity is unresolvable by routing. CORRECTION on candidate fixes:
  ALL bus lanes are pitch-1 (`lane_planner` assigns x = i+1) and
  main-line fixtures with splitter taps are CLEAN — so the
  neighbor-trunk gap is normally bridged by the CROSSING-ZONE / SAT
  machinery, and the (a)-(c) lane-geometry ideas are misdirected.
  The real defect: the crossing zone at block-local rows 13 did not
  cover/bridge the row-12 gap in the engine-unit trunk. Next probe
  per the standing lesson: prune_dangling first (diff raw SAT output
  vs sol.entities for that zone before theorising about
  clustering/growth/encoder); then zone scoping (why the zone
  excluded row 12). Merger unpaired-output
  at block-local (73,39) still separate and undiagnosed. Both
  reproduce standalone: compose_mega_block(USP@2 sub-solve, scale
  0.5) → 6 errors.*

- *2026-07-24 (later) — C3 LANDED, C2 LANDED, USP@2 composes
  end-to-end (2232×193, 48k entities — the largest layout ever
  composed); error ladder 71 → 44 → 27 → 18 through five fixes, each
  a named class: (1) fan splitter/branch columns dodge other drain
  descents; (2) pole trio nudge learns its own placements; (3) the
  OUTPUT MERGER had the same blocked-free-blocked pair-destroyer as
  the ghost router's fluid bridging (second instance of the class —
  runs now cluster across 1-tile gaps, mutation guards on plain
  belts); (4) fan pass-through takes its own dodged column; (5) fan
  branches descend via ALLOCATED lanes in slot pi+1 (lane_demand
  sizes them) instead of ad-hoc splitter columns that collided
  head-on with consumer ascents. REMAINING 18 = 6 × 3 copies, ALL
  engine-internal to the mega sub-layout (not Phase-C machinery),
  two bugs: (a) a trunk tap-off SPLITTER's second tile lands on the
  adjacent pitch-1 trunk column; the neighbor lane SKIPS that hard
  tile but emits no UG bridge across the skip (a crossing zone paves
  one row; the dangling surface belts feed the foreign splitter →
  items mix). Suspect: the trunk segmenter treats a FOREIGN
  splitter tile like an own-tap row (flow-through), but foreign
  splitters don't pass a different item — the skip needs a bridge.
  Block-local (4,12)/(5,11..13), trunk:engine-unit vs
  tapoff:iron-plate. (b) merger unpaired UG output at block-local
  (73,39) (east, no matching input) beside the electric-engine-unit
  merger run. Both reproduce standalone via compose_mega_block on
  the USP@2 sub-solve at scale 0.5 (debug_usp_block example,
  gitignored). C3's landed machinery: PTG spans stopped blocking
  surface crossings (span map, same-axis-only interference); fluid
  head slots dodge foreign fluid columns as retry rung 2/4;
  un-hoppable chain-fed clusters vertical-retrofit every column;
  entry mouth may sit at the west-edge entry tile; +1 margin row for
  cf blocks. C2: ascent-terminal retrofit generalizes to corr:/fan
  rows both directions.*

- *2026-07-24 — Phase C OPENED; the identity precondition is
  DISCHARGED BY MEASUREMENT and it falsified the engine. Eight
  single-refinery sim probes (A–H2: starve-vs-craft discrimination +
  a pipe-contents census on dead-end stubs) measured the full port
  identity table for advanced-oil-processing: recipe fluids bind
  x-ASCENDING on the unmirrored refinery (water/crude in W→E;
  heavy/light/PG out W→E — pure prototype box order) and x-DESCENDING
  on the engine-mirrored (dir+8 exported) form (crude WEST + water
  EAST; PG/light/heavy W→E) — the 180° rotation reverses port
  x-order while identities stay glued to their boxes, exactly the
  FFF #394 trap. The engine's ascending-always zip STARVED mirrored
  refineries in-game (probe A). Fix: the placer reverses the fluid
  list for mirrored rows (single-fluid sides reverse to themselves —
  every registered fixture bit-identical, hash gate green);
  `fluid_ports::port_fluid_assignment` is the shared measured rule
  with a pin test; foundry/cryo inherit the geometric rule with the
  refinery as measured anchor. The new tap arrangement immediately
  flushed out a LATENT ROUTER BUG: horizontal fluid-branch bridging
  mutated a previous hop's EXIT mouth into the next hop's entrance on
  blocked-free-blocked patterns (pair destroyed, branch stitched onto
  the foreign network west of it) — runs now cluster across 1-tile
  gaps within reach, and the conversion guards on plain-pipe.
  Harness: the multi-fluid-target report collapse fixed (checkpoint
  scalar is first-target-only; fluid targets verdict on PRODUCED —
  they have no drain rig). Frontier mapped (probe): lubricant targets
  are fluid exports (composition refuses by design); FRF@0.5/1 bus is
  CLEAN (no win there) but compose hits the C2 ascent-terminal
  collision (terminal lands on a corr: row; refusal now names the
  blocker); **USP@2 is the Phase C flagship — bus hard-fails
  (belt-loop + underground-belt), the mega swallows the full oil
  complex (10 members incl. BOTH oil processings + cracking +
  lubricant, 4 solid exports, 5 chain-fed inputs)**, eligibility now
  passes after C1 (multi-consumer export fan on the drain bypass row,
  splitter-chain idiom, single-consumer path byte-identical), and the
  remaining blocker is C3: the joint fluid planner cannot route
  crude+water feeds on the 10-member block at either spacing. Named
  increments: C2 (ascent congestion), C3 (adapter routing for the
  advanced complex).*

- *2026-07-24 — #411 adversarial review folds (APPROVE-WITH-NITS; both
  bus-refusal claims independently re-verified, retrofit guards probed
  and held, full battery re-run clean). Folds: (1) the chain-fed hop
  reach guard now derives from the RECORD's belt tier via
  `ug_max_reach` instead of a hardcoded express cap — the mouths stamp
  the record's tier, so a yellow record with a 6..9-tile span would
  have planned a pair the game never connects (latent,
  validator-backstopped, not triggered by any current fixture);
  (2) origin/main merged into the branch — the review's one MAJOR was
  merge hygiene, not code: the stale base made the two-dot diff show
  #410's review-bot guard as phantom reverts (three-dot diff touches
  only the 5 real files; 0 contested commits). Bot review's
  CLAUDE.md-compliance finding (skipped browser step) resolved by
  DOING the step: composed PU@4 verified in the web pipeline (0
  errors / 81 warnings, candidate selected) and eyeballed at three
  zoom levels; the "session convention" carve-out is memory-side, not
  CLAUDE.md — future layout PRs do the step or cite sim coverage
  explicitly.*

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
