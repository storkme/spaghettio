# Meter fluid modelling — follow-ups (#570)

**Status (2026-08-02): Phase A Machine + Factory fluid delivery LANDED; non-zero
achieved; Phase B/C open.** Enhancement #570: the fast meter (RFC-054)
previously returned a hard 0 on any chain needing a fluid. Implemented: fluids
as machine ingredients/products (`MachineState::FluidIngredientShortage`),
fluid ingredient buffers + `insert_fluid`, `has_ingredients` includes fluids,
fluid products accumulate to `fluid_output`, and a crude `tick_fluids`
port-adjacency delivery step (boundary fluid feed + producer->consumer;
unconsumed fluid output drained as delivered). Result over the corpus:
**AC/PU/AC-from-ore/plastic go non-zero** (AC_from_plates 0->0.2, PU 0->0.4,
AC_from_ore 0->1.0, plastic 0->10); solid chains stay byte-identical (gear
exact). Open: **magnitudes under for AC/PU (~20%) and the multi-output
refinery still 0** — the port-adjacency delivery + missing multi-output
byproduct balance are Phase B; ±10pp calibration is Phase C. Owning RFC:
[`rfc-054-fast-meter.md`](rfc-054-fast-meter.md).

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

**Phase A — fluid items + fluid recipes in `Machine` (the unblock).**
- Intern fluids as `ItemId`s; add fluid ingredient buffers + fluid product slots.
- Satisfy a machine's fluid ingredient from a **port-adjacent** fluid source
  (boundary crude-oil/water feed, or an adjacent machine's fluid output port) —
  port adjacency only, NOT full pipe routing yet.
- Honor fluid products (refinery/chem) into the fluid output.
- Add `fluid_ingredient_shortage` to `MachineState` (census parity with sim).
- Deliverable: AC/PU/oil chains go non-zero; solid chains byte-identical.

**Phase B — fluid delivery + pipe/port network.**
- Model fluid flow through pipes/ports (the `network.rs` gap): crude-oil/water
  boundary injection, pipe segments, port geometry.
- Multi-output fluid recipes (refinery heavy/light/petroleum) with **byproduct
  crediting** balanced so they don't over/under-produce.

**Phase C — calibration.**
- Re-run the meter corpus sweep; confirm fluid families non-zero + within ±10pp.
- Log any residual divergence in [`meter-divergence.md`](meter-divergence.md).

## Constraints / gates

- **KC4 independence:** fluid modelling must read recipe facts from `recipe_db::db()`
  (craft time/speed) and delivery topology from the blueprint — it may NOT import
  the engine's derived fluid rates or module math.
- **No regression on solid chains** (the already-agreeing fixtures are the guard).
- `cargo clippy` clean; no WASM impact (native binary only).
- Anything approximated (buffer depth, port throughput) gets an explicit stated
  default + sweepable param, the `DEFAULT_BUFFER_CRAFTS` pattern.

## Risk

- Biggest: Phase B pipe/port network routing (genuinely unimplemented today;
  refined to a **crude port-adjacency** pass in A to de-risk before full routing).
- Multi-output refinery recipes need byproduct/loop handling to stay balanced.
