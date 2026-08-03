# Meter fluid modelling — follow-ups (#570)

**Status (2026-08-03, follow-up `f5a-ptg-edge` + `fluid-followups`): Phase A + B
LANDED and merged (#571). Calibration within ±10pp on the whole compared corpus
EXCEPT `tier5_processing_unit_from_ore_am3` (−13%, a SOLID belt-delivery
residual — characterised, deliberately deferred; see below). CI second-opinion
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
gear exact; EC + stress-EC ±0–2%; AOP/refinery exact; **all AC variants now
±0–2% (were −80%)**; PU from ore −80% → −13%. The lone residual is PU-from-ore,
whose census is dominated by `full_output`/`working` with ~9 machines left short
on a solid and a 26-tile belt
cycle — a solid-side belt-model gap (open RFC-064 Phase 2 item), not a fluid
gap. Full divergence log:
[`meter-divergence.md`](meter-divergence.md).

## Goal / success criteria

- AC, PU, advanced-oil-processing, plastic-from-crude, uranium layouts produce a
  **non-zero** `produced_per_s` (currently hard 0).
- Meter within **±10pp of the measured sim** on those families (KC1), verified by
  re-running the corpus meter sweep (`crates/meter/examples/sweep_corpus.rs`).
- Solid chains do **not regress** (the ~25/70 that already agree must stay put).

## Where it stands in the code

- `machine.rs`: takes **solids only** — *"fluids are PR-3 out of scope."* Machine has
  recipe data from `recipe_db::db()` (full ingredients incl. fluids), but ignores
  fluid ingredients in the craft check and never emits fluid products.
- `network.rs`: "Deliberately not modelled yet" — no fluid pipe/port flow.
- `factory.rs:237`: fluid boundary inputs get the note *"fluid boundary input X not
  modelled"* and are skipped — so any chain fed crude-oil/water produces nothing.

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
  (a solid belt-delivery gap: ~9 machines short on a solid, 26-tile
  belt cycle — see [`meter-divergence.md`](meter-divergence.md)).
- Log any residual divergence in [`meter-divergence.md`](meter-divergence.md).

## Next steps / open items (2026-08-03)

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

### Confirm/close the PU-from-ore −13% — CHARACTERISED, deferred
Deep-dive (2026-08-03): the sim itself under-produces almost everything on this
fixture (intermediates ≈ −10%, petroleum −17%; only target PU hits 99%). The
meter matches the sim within ±4% on the entire direct chain (~the same, slightly
larger, absolute shortfall from plan as the sim), so the extra ~10% the meter
loses on PU is solely downstream belt delivery of electronic-circuit to the PU
machine — which sits short on EC despite adequate EC production. The fixture
carries the corpus's only topology note ("26 tiles in a belt cycle; update order
arbitrary"), the likely culprit. **Deferred deliberately**: a fix needs a
speculative belt-cycle-update-order / merge-priority model change, unverifiable
on this noisy fixture and risky for the ~40 agreeing fixtures. Tracked as item 7
in [`rfc064-phase2-followups.md`](rfc064-phase2-followups.md); full evidence in
[`meter-divergence.md`](meter-divergence.md).

### Orientation-keyed port binding — PROPOSED then REVERTED (this thread)
An attempt to key `mirrored` on orientation (`mirror_entity && direction == South`)
was proposed and then **reverted on review**: community blueprints re-freeze these
machines as `North + mirror:true` (and the engine's own import parser treats both
South and West wire forms as the mirrored collision), so a South-only key mis-binds
them — a regression vs the unconditional `mirrored = mirror_entity`. The correct
fix is to **parse the real `mirror` flag** in the blueprint decoder (the meter
reads no such field today), not to guess from direction. Left as a documented
future change; the unconditional binding (merged baseline) is kept, with a comment
recording the limitation.

### Two proposed fixes — REVERTED after review (record the call)
- **Census precedence** (report `FluidIngredientShortage` whenever a fluid is
  short): proposed then reverted. Reviewer: the sim labels a machine by whichever
  ingredient blocks next, so an unconditional fluid-priority would *diverge* from
  the module's "census lines up with the sim" contract, and neither precedence is
  verifiable here. Kept the original solids-first order.
- **Chem-plant "shared fluid box"** (bind a single fluid to both ports of a face):
  proposed then reverted. Reviewer: `recipes.json` shows chem-plant/biochamber have
  **four separate** fluid boxes (2 in + 2 out), so the shared-box premise is wrong;
  and binding both ports + per-network pooling introduced real over-credit and
  cross-network starvation paths. Reverted to single-port x-ordered binding.

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
