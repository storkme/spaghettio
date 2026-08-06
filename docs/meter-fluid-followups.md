# Meter fluid modelling — follow-ups (#570)

**Status (2026-08-06, follow-up `f5a-ptg-edge` + `fluid-followups`): Phase A + B
LANDED and merged (#571). Calibration within ±10pp on the whole compared corpus
EXCEPT `tier5_processing_unit_from_ore_am3` (−13%) — **CLOSED 2026-08-06**:
measured as a **productivity tech-state parity gap**, not a model defect. The
sim runs processing-unit at +10% research productivity (probe: PR #580); the
meter models none. Four earlier causes were proposed and retired first. CI second-opinion
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
−3.9%)**; PU from ore −80% → −13%. The one residual, PU-from-ore, is **closed**: measured 2026-08-06 as a
**productivity parity gap**. The sim carries +10.0% research productivity on
processing-unit and 0% on electronic-circuit, advanced-circuit and every plate/
cable stage (no modules involved); the meter models no productivity at all. That
is also why the gap landed on PU alone while everything else sat at ±0–2%. The
meter's −3.9% EC deficit compounded with the −9.1% it cannot model gives −12.7%
against −13.6% observed. Full divergence log:
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

**Phase C — investigation COMPLETE; calibration acceptance PENDING the parity
fix (2026-08-06).** Stated precisely because the two are different things: the
acceptance criterion is "all compared fixtures within ±10pp", and
`tier5_processing_unit_from_ore_am3` is still at −13%. What is finished is the
*diagnosis* — the cause is measured and is an instrument-parity gap, not a
meter model defect. The fix is undecided and unimplemented, and even once made
is predicted to leave ≈−4.3%.
- Re-run the meter corpus sweep (`examples/sweep_corpus.rs`); all compared
  fixtures within ±10pp EXCEPT `tier5_processing_unit_from_ore_am3` at −13% —
  **closed 2026-08-06** as a productivity tech-state parity gap (sim +10.0% on
  processing-unit, meter models none; see
  [`meter-divergence.md`](meter-divergence.md)). Not a fluid, belt or
  distribution defect.
- Log any residual divergence in [`meter-divergence.md`](meter-divergence.md).

## Next steps / open items (2026-08-06)

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

### Confirm/close the PU-from-ore −13% — **CLOSED 2026-08-06 (parity gap)**
Measured, not inferred. The sim harness now dumps realized productivity (PR
#580): **processing-unit +10.0%** and **plastic-bar +10.0%** (the latter found only
when the probe was widened to chemical-plant legs), with
electronic-circuit / advanced-circuit / iron-plate / copper-plate /
copper-cable / sulfur / the oil steps all **0.0%**, and no productivity
modules anywhere — so the source is `research_all_technologies()`. The meter
models no productivity at all by design (`crates/meter/src/machine.rs`), so on
this recipe instrument and reference measure different worlds. Decomposition:
−3.9% EC deficit compounded with −9.1% unmodelled productivity = −12.7% against
−13.6% observed, ~1pp inside the fixture's noise.

Four causes were proposed and retired before this one: belt-cycle update order
(≈14% of the gap), head-hog distribution (≈5%), upstream EC/plate production
(falsified by the sim's own copper-cable balance), and a sim-reporting-artifact
reading (falsified by the probe). The lesson worth keeping is that what settled
it was balancing the sim's reported numbers against each other and then
*measuring*, not reasoning about mechanism — the AC:PU ratio predicted +10.7%
against a measured +10.0%. Remaining decision (not a diagnosis): align the sim's
productivity to the fixture's declared level, or teach the meter productivity.
Full evidence in [`meter-divergence.md`](meter-divergence.md).

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
