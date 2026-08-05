# Meter fluid modelling — follow-ups (#570)

**Status (2026-08-05, follow-up `f5a-ptg-edge` + `fluid-followups`): Phase A + B
LANDED and merged (#571). Calibration within ±10pp on the whole compared corpus
EXCEPT `tier5_processing_unit_from_ore_am3` (−13%, most likely a **productivity
tech-state parity gap** between the sim and the meter rather than a layout or
belt defect — deliberately deferred pending one measurement; see below. Three
earlier causes were proposed and retired in turn: belt-cycle order (08-04),
head-hog distribution (08-05) and upstream EC/plate production (08-05, third
revision)). CI second-opinion
findings triaged: F5a stacked-PTG edge FIXED (#572); three latent code fixes
(census precedence, chem-plant "shared box", orientation-keyed binding) were
PROPOSED then all REVERTED on review — see the sections below; byproduct
backpressure consciously rejected (kept drain philosophy).** #570.

Phase B replaced Phase A's port-adjacency `tick_fluids` (which delivered fluid
one unit a tick and throttled petroleum→plastic→AC→PU to ~20%) with a real pipe
network: connected components of pipe tiles (incl. `pipe-to-ground` pairs)
plus machine fluid ports and boundary feeds, routing each fluid pipe-fast from
boundary sources and producer outputs to consumer buffers, fairly shared across
consumers (a greedy index-order allocator starved the last consumer on tight
supply). Honours F4 (PTG pairing), F5 (blueprint direction = surface-opening
side) and F5a (PTG perpendicular sides closed), and the measured mirrored-port
x-descending fluid binding for oil-refinery/foundry/cryogenic-plant.

Result over the corpus (meter `delivered_per_s`/`produced_per_s` vs sim):
gear exact; EC + stress-EC ±0–2%; AOP/refinery exact; **the dedicated AC
variants now ±0–2% (were −80%; the PU-from-ore exception fixture's own AC is
−3.9%)**; PU from ore −80% → −13%. The lone residual is PU-from-ore, an
**instrument-parity gap, not a fluid or belt one**: the sim calls
`research_all_technologies()` and so runs with productivity researched, while
the meter models no productivity at all. Its effective 21.74 EC/PU against the
recipe's 24 implies ≈10%, which with the meter's −3.9% EC deficit accounts for
−12.7% of the −13.6% observed — *conditional on EC/AC carrying no
productivity in the sim, which this sweep's own EC/AC ±0–2% figures make an
open question*. See the PU entry — and note this is a hypothesis awaiting one
measurement, after three retired predecessors. Open
RFC-064 Phase 2 item 7. Full divergence log:
[`meter-divergence.md`](meter-divergence.md).

## Goal / success criteria

- AC, PU, advanced-oil-processing, plastic-from-crude, uranium layouts produce a
  **non-zero** `produced_per_s` (currently hard 0).
- Meter within **±10pp of the measured sim** on those families (KC1), verified by
  re-running the corpus meter sweep (`crates/meter/examples/sweep_corpus.rs`).
- Solid chains do **not regress** (the ~25/70 that already agree must stay put).

## Where it stands in the code (current, post Phase A + B)

- `machine.rs`: fluid-aware — fluid ingredient buffers (`fluid_input`/`fluid_needs`),
  fluid products→`fluid_output`, `MachineState::FluidIngredientShortage`, and a
  craft gate that consumes solids and fluids together.
- `fluid.rs`: the pipe network — connected components of `pipe`/`pipe-to-ground`/
  `pump` + machine fluid ports + boundary feeds, honoring F4/F5/F5a topology.
- `factory.rs: tick_fluids`: per-component, per-fluid pipe-fast routing from
  boundary + producer outputs to consumer buffers, shared fairly. Element-boundary
  fluid feeds that touch no pipe are reported ("touches no pipe network"), not
  silently skipped.

## Scope (bounded, spike-first per RFC-063/064 discipline)

**Phase A — DONE.** Fluid items + fluid recipes in `Machine`, port-adjacency
delivery, `fluid_ingredient_shortage`. AC/PU/oil chains went non-zero.

**Phase B — DONE.** Pipe/port network (`crates/meter/src/fluid.rs` +
`Factory::tick_fluids`): connected components of `pipe`/`pipe-to-ground`/`pump`
tiles plus machine fluid ports and boundary feeds; per-component, per-fluid
pipe-fast routing from boundary + producer outputs to consumer buffers, shared
fairly (proportional + largest-remainder). Machine port tiles derived from
`entity_data::base_fluid_ports` + a direction rotation, fluids bound to ports
x-ascending except on the engine-mirrored set (oil-refinery/foundry/
cryogenic-plant bind x-descending). Topology: F4 (PTG underground pairs), F5
(blueprint direction = surface-opening side), F5a (PTG perpendicular sides
closed — keeps crossing/stacked fluid lines isolated).

**Phase C — calibration (close to done; one open residual).**
- Re-run the meter corpus sweep (`examples/sweep_corpus.rs`); all compared
  fixtures within ±10pp EXCEPT `tier5_processing_unit_from_ore_am3` at −13%
  (most likely a **productivity tech-state parity gap**: the sim researches
  everything, the meter models no productivity. The layout-side hypotheses are
  bounded small — head-hog distribution ≈5% of the gap at fixed EC supply,
  belt-cycle order ≈14% — so neither is the cause. See
  [`meter-divergence.md`](meter-divergence.md)).
- Log any residual divergence in [`meter-divergence.md`](meter-divergence.md).

## Next steps / open items (2026-08-05)

### F5a stacked-PTG edge — FIXED
A pipe-to-ground's surface mouth now only joins a regular pipe or a **back-facing**
pipe-to-ground (F5b); a same-facing stacked PTG no longer merges the two lines.
Previously the mouth unioned *any* pipe on its tile, breaking stacked-trunk
isolation. New regression test (`stacked_same_facing_ptgs_stay_isolated`). Corpus
sweep unchanged (zero regression).

### Fluid byproduct backpressure — consciously REJECTED (kept drain philosophy)
CI second-opinion flags that `tick_fluids` drains every unconsumed producer fluid
unit as `delivered`, so a machine whose fluid byproduct has no consumer never
backs up (in-game it would stall the producer). **Decision (2026-08-03): keep the
documented max-throughput philosophy** — `factory.rs`'s header states outputs drain
at the layout edge so "backpressure cannot falsify the measurement", matching the
sim harness's remove-mode-chest methodology the meter calibrates against. Adding
backpressure would make the meter *diverge* from its own reference instrumentation,
and no compared fixture exercises a byproduct loop (all 8 fluid-target fixtures
are NaN — no sim baseline), so it is unverifiable. The in-game viewpoint is valid
Factorio physics but a different measurement philosophy; recorded here so the call
is explicit, not accidental. Revisit only if a sim-baselined byproduct-loop fixture
ever enters the corpus.

### Confirm/close the PU-from-ore −13% — CHARACTERISED (root cause revised), deferred
Deep-dive (2026-08-03 + 08-04): the sim itself under-produces almost everything
on this fixture (intermediates ≈ −10%, petroleum −17%; only target PU hits 99%).
The meter matches the sim within ±4% on the entire direct chain. Its PU output
of 1.716/s is **0.271/s below the sim's 1.987/s** (the −13.6% sim-relative
residual) and 0.284/s below the 2.0/s ideal — two different bases, kept
separate here because earlier revisions welded them into one "~0.29/s ≈ −13%".
**2026-08-04: the root cause is revised — it is an EC supply shortfall, not the
belt-cycle update order.** Experiment: permuting the 26-tile cyclic order moves
PU only 1.716→1.754/s (+2.2%); inter-loop reorder has no effect.
**2026-08-05: revised again — distribution is not the dominant driver either.**
Each PU consumes 24 EC (20 direct + 2 AC × 2), so the meter's 41.5 EC/s caps PU
at 41.5/24 = 1.729/s and it measures 1.716/s — **99.2% of its own ceiling**. EC
is scarce only *relative to the 48/s plan rate*: at the operating point
production and consumption balance (41.2/s consumed vs 41.5/s produced). The
head-hog gradient is real — 12/16 PU machines craft at full 0.125/s while the
four deepest (`m301/m302/m309/m310`, x=55/58) sit on EC buffers 1–12/280 (craft
0.023–0.088/s; only `m310` labels `ItemIngredientShortage`, the other three read
`Working` but run below rate) — but perfect redistribution would gain just
**+0.013/s, ≈5% of the gap** — *at fixed EC supply* (the 08-04 permuted run hit
1.754/s, needing 42.1 EC/s against the baseline's 41.5/s, so the ceiling is an
operating point rather than an invariant; see `meter-divergence.md`).
**2026-08-05, third revision: the "upstream EC/plate production" reading is
retracted too.** It rested on imputing 47.7 EC/s from the sim's PU output, and
the sim's own copper-cable measurement refutes that: 3×43.2 + 4×3.59 =
143.96/s against 143.9/s measured, a 0.04% match, while 47.7 would need
157.5/s. The sim's reported EC is corroborated; the imputation is not. At face
value the meter's EC is only −3.9%, inside its band.
**Leading hypothesis instead: a productivity tech-state parity gap.** The sim
calls `research_all_technologies()` (`crates/sim-harness/src/scenario.rs`) and
its parity block corrects only inserter capacity and belt stacking, not
productivity; the meter documents that it takes nothing from `module_policy`
and models no productivity at all. The sim's effective 21.74 EC/PU vs the
recipe's 24 means it *behaves as if* at ≈+10%; attributing that to a
+10%/level research is unverified. **Open joint**: the gap shows on PU alone
(gear exact, EC/AC ±0–2%), so either productivity research is per-recipe with
none for EC/AC, or they are boosted and the −3.9% EC term is not independent —
in which case productivity alone covers the whole −13.6% rather than the
−12.7% the compounded reading gives.
**Deferred deliberately, and not fixed on arithmetic**: three causes have now
been proposed and retired here, so the fourth needs a measurement — dump the
force's realized `processing-unit` productivity bonus in a sim run, the same
self-audit pattern the inserter and belt-stacking parity blocks already use.
Tracked as item 7 in
[`rfc064-phase2-followups.md`](rfc064-phase2-followups.md); full evidence in
[`meter-divergence.md`](meter-divergence.md).

### Orientation-keyed port binding — PROPOSED then REVERTED (this thread)
An attempt to key `mirrored` on orientation (`mirror_entity && direction == South`)
was proposed and then **reverted on review**: community blueprints re-freeze these
machines as `North + mirror:true` (and the engine's own import parser treats both
South and West wire forms as the mirrored collision), so a South-only key mis-binds
them — a regression vs the unconditional `mirrored = mirror_entity`. A complete
fix must key on **both** signals: a parsed `mirror` flag (for community
`North+mirror:true`) AND the engine's `direction+8` South wire form (which the
exporter uses in place of a mirror flag for these machines) — the direction
heuristic alone is insufficient, but so is parsing `mirror` alone. Left as a
documented future change; the unconditional binding (merged baseline) is kept,
with a comment recording the limitation.

### Two proposed fixes — REVERTED after review (record the call)
- **Census precedence** (report `FluidIngredientShortage` whenever a fluid is
  short): proposed then reverted. Reviewer: the sim labels a machine by whichever
  ingredient blocks next, so an unconditional fluid-priority would *diverge* from
  the module's "census lines up with the sim" contract, and neither precedence is
  verifiable here. Kept the original solids-first order.
- **Chem-plant "shared fluid box"** (bind a single fluid to both ports of a face):
  proposed then reverted. Independent reviews **disagreed** on the underlying
  box topology (`recipes.json` lists 4 `pipe_connection` entries on
  chemical-plant/biochamber — read by one review as 4 separate boxes, by another
  as 2 boxes × 2 connections). Rather than rely on an unverified topology claim,
  the change was reverted because the *implementation* was unsafe regardless:
  binding both ports + per-network pooling introduced real over-credit and
  cross-network starvation paths that the existing single-port routing cannot
  reproduce. Reverted to the single-port x-ordered binding; the "other-port
  starves" behaviour stays open and needs a correct (network-partition-safe)
  fix plus a verified fluid-box topology before it is worth re-attempting.

### Remaining latent minors (recorded, not chased)
- A machine short of both a solid and a fluid is classified `ItemIngredientShortage`
  (solids first); a chem-plant single-fluid pipe on the non-x-ordered port tile
  starves (the two above — both intentionally left as-is after review).
- Re-bless any golden/snapshot baselines the corpus ingest tests depend on.

## Constraints / gates

- **KC4 independence:** fluid modelling must read recipe facts from `recipe_db::db()`
  (craft time/speed) and delivery topology from the blueprint — it may NOT import
  the engine's derived fluid rates or module math.
- **No regression on solid chains** (the already-agreeing fixtures are the guard).
- `cargo clippy` clean; no WASM impact (native binary only).
- Anything approximated (buffer depth, port throughput) gets an explicit stated
  default + sweepable param, the `DEFAULT_BUFFER_CRAFTS` pattern.

## Risk

- Multi-output refinery byproduct loops (forced-pipe-isolation AOP) are routed
  and isolated correctly but the cracking chem-plants in that uncalibrated
  fixture run slightly starved (no sim baseline to compare). See divergence doc.
- Pipe throughput is not modelled (assumed pipe-fast); only relevant if a fixture
  chokes on a long/undersized pipe run — none in the corpus does yet.
