# Meter divergence log (#570, RFC-054 Phase B/C)

Running record of where the fast meter's `produced_per_s`/`delivered_per_s`
diverges from the measured headless-Factorio sim by more than ±10pp, and why
that divergence is believed to live (model gap vs. known-open-item). Updated
when the corpus sweep (`crates/meter/examples/sweep_corpus.rs`) moves a number
or reveals a new one. Currently **one** residual.

## Corpus status (2026-08-03, Phase B landed)

`meter sweep: 70 layouts measured, 41 compared`. Every compared fixture is
within ±10pp of sim except the one below. AC/PU families that were −80% in
Phase A are now within ±2% (the dedicated AC fixtures) / −13% (PU-from-ore; its
in-fixture AC reads −3.9%).

## Open residual

### `tier5_processing_unit_from_ore_am3` — meter ≈ −13% (`produced`)

- **meter**: processing-unit 1.716/s vs sim produced 1.987/s / delivered 1.961/s.
- **Direction**: underproduction in the **downstream solid belt delivery**, not a
  fluid gap. Deep-dive (2026-08-03) established this precisely:
  - The **sim itself** under-produces almost everything on this fixture
    (copper-plate 71.98/80, copper-cable 143.9/160, EC 43.2/48, plastic 7.2/8 —
    all ≈ −10%; petroleum-gas −17%); only the PU target reaches 99%. So plan is
    not the reference; the sim is. (Intermediates here are *measured production*
    and do not always reconcile arithmetically with consumers — e.g. EC demand
    from PU+AC ≈ 47/s vs ~43/s reported produced; the copper-plate→cable pair does
    reconcile (1 plate → 2 cable). Treat the intermediate figures as indicative,
    not load-bearing for the deferral's rationale.)
  - The meter matches sim within ±4% on the whole direct chain: copper-plate
    69.5 vs sim 71.98 (−3.4% — the meter is slightly *further* below plan),
    copper-cable 139 vs 143.9, iron-plate 41.9 vs 43.4, EC 41.5 vs 43.2,
    plastic 7.2 vs 7.2 (=), AC 3.45 vs 3.59. So the direct-production chain
    tracks the sim closely; the order-of-magnitude-larger shortfall is PU alone.
  - **2026-08-04 revision: the residual is supply-marginal tail starvation on the
    EC trunk, NOT a belt-cycle update-order bug.** A direct experiment
    (permuting the cyclic update order of the 26-tile belt loop) moves PU only
    1.716→1.754/s (+2.2%); reordering between the two loops does nothing. The
    dominant driver is distribution of a genuinely scarce item: PU demand alone
    needs ~40 EC/s and after AC it is ~48/s, but both meter (41.5/s) and sim
    (43.2/s) underproduce EC. At steady state the meter runs **15/16 PU machines
    at full rate (0.125/s)** while the four deepest (x=55/58: `m301/m302/m309/
    m310`) sit starved on EC buffers of 1–12/280 and craft at 0.023–0.088/s,
    `m310` fully blocked — a head-buffers-starve-tail gradient losing ~0.29/s of
    the 2.0/s ideal. The sim distributes the scarce EC more evenly (feeds PU to
    99%), which is exactly the unverifiable difference described below.
- **Verdict**: a genuine belt-model **delivery/distribution** divergence on a
  single fixture whose sim baseline is itself noisy (≈ −10% on everything).
  **Deferred, deliberately**: closing it means changing how the meter distributes
  a supply-limited item between machines (a merge-priority / head-hog fairness
  model), which **cannot be validated here** — the sim's per-machine EC
  distribution is not in `report.json`, its aggregate is −10% below plan on this
  very fixture, and the meter's EC production already matches the sim within ±4%.
  Whether the meter (starving the tail) or the sim (feeding the target) is
  "right" is not decidable against a clean reference; the meter may be *correctly*
  exposing that the factory genuinely cannot deliver 2/s PU. Tracked as item 7 in
  `rfc064-phase2-followups.md`; do not chase it inside this meter-fluid thread.
- **Next**: investigate as a belt **distribution/throughput** divergence, not a
  fluid one and not a cycle-order one (the cycle order is a ≤~2% contributor).
  Any change needs a non-noisy, sim-baselined solid fixture to be verifiable.
  Re-measure after any belt-network (notably merger/priority) changes.

## Closed / moved entries

- **AC / PU / AC-partitioned (~−80%)**: closed by Phase B pipe-fast, fair,
  buffer-absorbing fluid routing. AC variants are now ±0–2% (the PU-from-ore
  exception fixture's own AC is −3.9% by the same meter).
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
