# Meter divergence log (#570, RFC-054 Phase B/C)

Running record of where the fast meter's `produced_per_s`/`delivered_per_s`
diverges from the measured headless-Factorio sim by more than ±10pp, and why
that divergence is believed to live (model gap vs. known-open-item). Updated
when the corpus sweep (`crates/meter/examples/sweep_corpus.rs`) moves a number
or reveals a new one. Currently **one** residual.

## Corpus status (2026-08-03, Phase B landed)

`meter sweep: 70 layouts measured, 41 compared`. Every compared fixture is
within ±10pp of sim except the one below. AC/PU families that were −80% in
Phase A are now within ±2% (AC) / −13% (PU-from-ore).

## Open residual

### `tier5_processing_unit_from_ore_am3` — meter ≈ −13% (`produced`)

- **meter**: processing-unit 1.716/s vs sim produced 1.987/s / delivered 1.961/s.
- **Direction**: underproduction, not a fluid gap.
- **Evidence it is not the fluid model**: the fixture's machine census shows
  `product_column item_ingredient_shortage ≈ 30`, `fluid_ingredient_shortage = 1`,
  and the refineries hold `crude=1400` / `water=700` (full buffers, never
  starved) while crafting at the correct 300-tick cadence. All downstream items
  (electronic-circuit 41.5/48, copper-cable 139/160, plastic 7.2/8) sit at
  ~86–90%, i.e. the choke is upstream of fluid, in the solid belt delivery.
- **Probable cause**: a solid-side belt-model gap on this deep/complex layout,
  which the fixture itself flags with *"26 tiles in a belt cycle; update order
  arbitrary"*. The sim (real Factorio) sustains ~98% here; the meter's belt
  transport under-delivers ~13%. This is an RFC-064 Phase 2 open item
  ("input-delivery … class items open"), orthogonal to the Phase B fluid work.
- **Next**: investigate as a belt-throughput/cycle divergence, not a fluid one.
  Re-measure after any belt-network changes.

## Closed / moved entries

- **AC / PU / AC-partitioned (~−80%)**: closed by Phase B pipe-fast, fair,
  buffer-absorbing fluid routing. AC variants are now ±0–2%.
- **`tier3_plastic_bar_from_crude` (~−80%)**: closed (now 10 = plan).
- **AOP / refinery / sulfur / heavy-oil cracking (delivered)**: no divergence;
  the fluid networks (incl. the forced-pipe-isolation AOP) route correctly.

## Non-gated observations (no sim baseline, so NaN — not counted)

- `tier3_advanced_oil_processing_forced_multi_machine_pipe_isolation` has no
  `report.json` (sim NA) and is not compared. The meter's pipe-isolation
  topology keeps crude/water/heavy/light/petroleum in separate networks, and
  the refinery refines correctly, but the two light-oil + one heavy-oil
  cracking chemical-plants run slightly starved (hold ~75% of a per-craft
  need). Worth a calibration fixture one day; not a regression vs Phase A
  (which could not route fluids at all).
