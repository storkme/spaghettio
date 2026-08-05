# Meter divergence log (#570, RFC-054 Phase B/C)

Running record of where the fast meter's `produced_per_s`/`delivered_per_s`
diverges from the measured headless-Factorio sim by more than ±10pp, and why
that divergence is believed to live (model gap vs. known-open-item). Updated
when the corpus sweep (`crates/meter/examples/sweep_corpus.rs`) moves a number
or reveals a new one. Currently **one** residual.

## Corpus status (2026-08-05; Phase B landed 2026-08-03)

`meter sweep: 70 layouts measured, 41 compared`. Every compared fixture is
within ±10pp of sim except the one below. AC/PU families that were −80% in
Phase A are now within ±2% (the dedicated AC fixtures) / −13% (PU-from-ore; its
in-fixture AC reads −3.9%).

## Open residual

### `tier5_processing_unit_from_ore_am3` — meter ≈ −13% (`produced`)

- **meter**: processing-unit 1.716/s vs sim produced 1.987/s / delivered 1.961/s.
- **Direction**: underproduction in the **upstream EC supply**, not a fluid gap
  — and, per the 2026-08-05 revision below, not downstream belt *delivery*
  either. That was the 2026-08-03 reading; the ceiling arithmetic retired it.
  Deep-dive (2026-08-03) established the surrounding figures precisely:
  - The **sim itself** under-produces almost everything on this fixture
    (copper-plate 71.98/80, copper-cable 143.9/160, EC 43.2/48, plastic 7.2/8 —
    all ≈ −10%; petroleum-gas −17%); only the PU target reaches 99%. So plan is
    not the reference; the sim is. (Intermediates here are *measured production*
    and do not always reconcile arithmetically with consumers — e.g. EC demand
    from PU+AC ≈ 48/s vs ~43/s reported produced; the copper-plate→cable pair does
    reconcile (1 plate → 2 cable). Treat the intermediate figures as indicative,
    not load-bearing for the deferral's rationale.)
  - The meter matches sim within ±4% on the whole direct chain: copper-plate
    69.5 vs sim 71.98 (−3.4% — the meter is slightly *further* below plan),
    copper-cable 139 vs 143.9, iron-plate 41.9 vs 43.4, EC 41.5 vs 43.2,
    plastic 7.2 vs 7.2 (=), AC 3.45 vs 3.59. So the direct-production chain
    tracks the sim closely; the order-of-magnitude-larger shortfall is PU alone.
  - **2026-08-04 revision: the residual is an EC *supply* shortfall, NOT a
    belt-cycle update-order bug.** A direct experiment (permuting the cyclic
    update order of the 26-tile belt loop) moves PU only 1.716→1.754/s (+2.2%);
    reordering between the two loops does nothing.
  - **2026-08-05 correction — the meter sits within 1% of its own EC-supply
    ceiling, so distribution cannot be the dominant driver.** Each PU consumes
    **24 EC** (20 direct + 2 AC × 2 EC each), so the meter's 41.5 EC/s supports
    at most 41.5/24 = **1.729 PU/s**; it measures 1.716/s, i.e. **99.2%** of
    that ceiling. At the operating point EC production and consumption balance
    — 1.716×20 + 3.45×2 = **41.2/s consumed vs 41.5/s produced**, a ~0.3/s
    surplus — so EC is scarce only *relative to the 48/s plan rate*, not in
    absolute terms at the rate this fixture actually runs. (An earlier revision
    of this entry called EC "genuinely scarce" without that qualifier.)
  - **The head-hog gradient is how the shortfall lands, not why it exists.**
    At steady state the meter runs **12/16 PU machines at full rate (0.125/s)**
    while the four deepest (x=55/58: `m301/m302/m309/m310`) are EC-constrained,
    with buffers of 1–12/280 and craft rates of 0.023–0.088/s. Note only `m310`
    labels `ItemIngredientShortage`; the other three read `Working` but run
    below rate (the census label and the full-rate count are different things).
    But redistributing the available EC *perfectly* across all 16 machines
    yields only 1.729/s — **+0.013/s, ≈5% of the 0.271/s sim-relative gap.** A
    distribution / merge-priority / head-hog-fairness model change therefore
    **cannot close this residual.** The dominant term is EC underproduction
    itself — 41.5/s against the **≈47.7/s the real factory moved** (see the
    next-but-one bullet), i.e. −13% sim-relative; −13.5% against the 48/s
    plan, which is the same number by coincidence and not the reference. That
    tracks the plate shortfall upstream (iron-plate 41.9/s caps EC at 41.9/s,
    at 1 plate per EC).
  - **The gap against each base, kept separate.** Meter 1.716/s is **0.271/s**
    below the sim's 1.987/s (the −13.6% sim-relative residual) and **0.284/s**
    below the 2.0/s ideal. Different denominators; earlier revisions of this
    entry quoted a single "~0.29/s ≈ −13%" that welded the ideal-relative
    magnitude to the sim-relative percentage.
  - **What the sim's numbers actually license — and a trap to avoid.** It is
    tempting to run the ceiling arithmetic on the sim too (43.2 EC/s caps PU
    at 1.80/s, yet it reports 1.987/s) and conclude the sim contradicts
    itself. **That inference is wrong, and a previous revision of this entry
    made it.** The sim's intermediates are flagged *indicative* two bullets
    above precisely because they are not a closed mass balance; its **target**
    figure is the reference this whole calibration is defined against. Using
    the unreliable number to impeach the reliable one inverts the hierarchy.
    Read the other way round, the sim is informative: real Factorio delivered
    1.987 PU/s, so it physically moved **≈47.7 EC/s** (1.987 × 24). Its
    reported 43.2/s undercounts that by ~9% — an artifact of intermediate
    reporting, consistent with the caveat. Measured against the real 47.7/s,
    the meter's 41.5 EC/s is **≈13% short**.
- **Verdict**: a real, meter-side divergence, **relocated upstream** — from
  distribution at the PU machines to **EC/plate production**. The chain reads
  cleanly: the meter's EC output is ~13% below what the real factory achieved,
  and its PU output is 99.2% of what that reduced EC supply allows. So the PU
  stage is behaving; the stage that is not is the one feeding it.
  **Deferred, deliberately** — but note the defect is *relocated, not
  retired.* An earlier revision of this entry concluded "no identified meter
  defect left to fix" and that the meter was correctly exposing a factory that
  cannot deliver 2/s PU. Both are overreach: Factorio delivered ~1.99/s on
  this very fixture, so the factory demonstrably can. What the ceiling
  arithmetic retires is the *distribution* hypothesis (≈5%), not meter fault
  as such. Closing the item needs a reference with per-machine detail, which
  this run cannot supply — the sim's per-machine EC distribution is not in its
  stored `report.json` (its `timeseries` field is absent, so no per-machine
  craft/status checkpoints were captured). Tracked as item 7 in
  `rfc064-phase2-followups.md`; do not chase it inside this meter-fluid
  thread.
- **Next**: if this is ever reopened, the question to ask is why the meter's
  **plate/EC stages** produce ~13% less than the real factory did (41.5 vs the
  ≈47.7 EC/s implied by the sim's 1.987 PU/s) — not how EC is distributed
  among the PU machines (bounded at ≈5% of the gap) and not the cycle order
  (~+2.2%). It needs a sim run whose `timeseries` is captured, so per-machine
  detail exists on the reference side. Any change needs a non-noisy,
  sim-baselined solid fixture to be verifiable.
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
