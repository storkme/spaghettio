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
- **Direction**: underproduction in the **downstream solid belt delivery**, not a
  fluid gap. Deep-dive (2026-08-03) established this precisely:
  - The **sim itself** under-produces almost everything on this fixture
    (copper-plate 71.98/80, copper-cable 143.9/160, EC 43.2/48, plastic 7.2/8 —
    all ≈ −10%; petroleum-gas −17%); only the PU target reaches 99%. So plan is
    not the reference; the sim is.
  - The meter matches sim within ±4% on the whole direct chain: copper-plate
    69.5 vs sim 71.98 (−3.4% — the meter is slightly *further* below plan),
    copper-cable 139 vs 143.9, iron-plate 41.9 vs 43.4, EC 41.5 vs 43.2,
    plastic 7.2 vs 7.2 (=), AC 3.45 vs 3.59. So the direct-production chain
    tracks the sim closely; the order-of-magnitude-larger shortfall is PU alone.
  - The extra loss is in delivering EC to the PU machine: producer `m#310`
    sits short on EC (12 of a 20/craft need) despite EC production (41.5/s)
    meeting total demand (AC ~6.9 + PU ~34 = ~41/s). The sim feeds PU fully.
  - The machine census is dominated by `full_output` + `working`; only ~9 are
    left short on a solid and one on a fluid, all downstream of the EC belt
    (a precedence-sensitive number, so not a stable discriminator on its own).
  - The fixture's wiring produces the **only** topology note in the corpus:
    *"26 tiles in a belt cycle; update order arbitrary"* — a cyclic belt (the
    many-EC-producers merge trunk into the few PU/AC consumers), which the meter
    steps in an arbitrary-but-deterministic order and which is the strong
    candidate for the delivery shortfall.
- **Verdict**: a genuine belt-model divergence, on a single fixture whose sim
  baseline is itself noisy (≈ −10% on everything). **Deferred, deliberately**:
  fixing it needs a speculative belt-cycle-update-order / merge-priority model
  change that cannot be validated against a clean reference on this fixture and
  would risk the ~40 fixtures that do agree. Tracked as item 7 in
  `rfc064-phase2-followups.md`; do not chase it inside this meter-fluid thread.
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

## Deliberately-not-modelled (decision log)

- **Byproduct backpressure.** The meter drains every unconsumed producer fluid
  unit as `delivered` and never stalls a producer on a full fluid output, so a
  multi-output recipe (AOP heavy/light/petroleum) with an unhandled byproduct
  reads its *maximum* throughput rather than stalling as real Factorio would.
  This is a **conscious choice, not a bug**: the meter's stated philosophy
  (see `factory.rs` header) is to drain outputs so measurement is not falsified
  by backpressure, matching the sim harness's remove-mode-chest methodology that
  the meter calibrates against. Consistent with that, all 8 fluid-target corpus
  fixtures are uncompared (no sim baseline), so this divergence is unverifiable
  and would only matter if a sim-baselined byproduct-loop fixture joined the
  corpus. Revisit then; recorded so the call is explicit. (2026-08-03)
