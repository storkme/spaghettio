# Meter fluid modelling — follow-ups (#570)

**Status (2026-08-03): Phase A landed (machine fluid + port-adjacency);
Phase B LANDED — pipe networks + pipe-fast/balanced delivery. Calibration now
within ±10pp on the whole compared corpus EXCEPT `tier5_processing_unit_from_ore_am3`
(−13%, a solid belt-delivery residual, not fluid).** #570.

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
whose census shows 30 `item_ingredient_shortage` machines and a 26-tile belt
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
  (a solid belt-delivery gap: 30 `item_ingredient_shortage` machines, 26-tile
  belt cycle — see [`meter-divergence.md`](meter-divergence.md)).
- Log any residual divergence in [`meter-divergence.md`](meter-divergence.md).

## Next steps (uncommitted for this thread)

1. **Confirm/close the PU-from-ore −13%.** Its census shows 30 `item_ingredient_shortage`
   and only 1 fluid-short machine — petroleum/fluid are not the limit. Investigate
   as a belt-model divergence (the fixture notes "26 tiles in a belt cycle").
2. **Fluid byproduct backpressure (open gap, flagged by CI second-opinion).**
   `tick_fluids` drains every unconsumed producer fluid unit as `delivered`, so a
   machine whose fluid byproduct has no consumer (or more byproduct than consumer
   capacity) never backs up: the excess drains and the producer keeps crafting at
   full speed. In-game, excess heavy/light oil would fill the pipe and stall the
   refinery, capping the target petroleum output too. This is entangled with the
   sweep's `delivered_per_s` metric for fluid targets (AOP relies on the drain),
   so it needs a deliberate design (e.g. only count drained surplus as delivered
   when the item is a declared boundary-output/target, plus a fluid-output
   full-output stall on the producer) — a follow-up, not a rushed change. The
   current corpus has no over-capacity byproduct, so ±10pp stands.
3. Re-bless any golden/snapshot baselines the corpus ingest tests depend on.

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
