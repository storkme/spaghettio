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
- **Direction**: most likely **not a layout/belt divergence at all** — the
  leading hypothesis is a productivity tech-state gap between the sim and the
  meter (see the ⇒ bullet below). Three earlier readings have been retired in
  turn: belt-cycle update order (08-04), head-hog distribution (08-05), and
  upstream EC/plate production (08-05, third revision). Deep-dive (2026-08-03)
  established the surrounding figures precisely, and those still stand:
  - The **sim itself** under-produces almost everything on this fixture
    (copper-plate 71.98/80, copper-cable 143.9/160, EC 43.2/48, plastic 7.2/8 —
    all ≈ −10%; petroleum-gas −17%); only the PU target reaches 99%. So plan is
    not the reference; the sim is. (Intermediates here are *measured production*
    and do not always reconcile arithmetically with consumers — e.g. EC demand
    from PU+AC is ~46.9/s at the measured operating point (48/s is the *plan*
    rate) vs ~43/s reported produced; the copper-plate→cable pair does
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
    that ceiling — *at that supply level*; the next bullet shows the ceiling
    itself moves. At the operating point EC production and consumption balance
    — 1.716×20 + 3.45×2 = **41.2/s consumed vs 41.5/s produced**, a ~0.3/s
    surplus — so EC is scarce only *relative to the 48/s plan rate*, not in
    absolute terms at the rate this fixture actually runs. (An earlier revision
    of this entry called EC "genuinely scarce" without that qualifier.)
  - **⚠ The ceiling is an operating-point statement, not a hard invariant —
    and the 08-04 experiment proves it.** The permuted-cycle run reached
    1.754 PU/s, which needs 1.754 × 24 = **42.1 EC/s** — above the 41.5/s the
    baseline run produced, and 0.025/s above the 1.729/s ceiling that number
    implies. So EC production is **not fixed**: it responds to belt-model
    changes, because EC's own inputs (iron plate, copper cable) are belt-
    delivered too. Two consequences, and they pull in the same direction:
    (1) every "≈5% of the gap" distribution bound below holds **at fixed EC
    supply** — it bounds what redistributing a *given* 41.5/s can buy, not
    what the meter could reach if supply moved; (2) the cycle-order experiment
    most likely bought its +0.038/s **through EC supply rather than through PU
    distribution**, which is further evidence the binding constraint lives
    upstream. **Unverified**: the permuted run's own EC figure was not
    recorded, so (2) is inference from the arithmetic, not measurement. Re-run
    the permutation capturing EC production before relying on it. Flagged
    rather than silently reconciled — the two numbers *are* in tension as
    published (bot review round 7, 3/3 passes).
  - **The head-hog gradient is how the shortfall lands, not why it exists.**
    At steady state the meter runs **12/16 PU machines at full rate (0.125/s)**
    while the four deepest (x=55/58: `m301/m302/m309/m310`) are EC-constrained,
    with buffers of 1–12/280 and craft rates of 0.023–0.088/s. Note only `m310`
    labels `ItemIngredientShortage`; the other three read `Working` but run
    below rate (the census label and the full-rate count are different things).
    **Provenance unverified**: `main`'s earlier version of this entry carried
    a caveat that its per-machine split was captured with the later-reverted
    census-precedence change active, hence precedence-sensitive and not
    re-verified on merged solids-first code. That caveat was attached to the
    figure this census replaced; whether the 12/16 numbers share the same
    provenance has not been checked. Treat them as diagnostic, not
    load-bearing, until re-measured.
    But redistributing the available EC *perfectly* across all 16 machines
    yields only 1.729/s — **+0.013/s, ≈5% of the 0.271/s sim-relative gap**
    (at fixed EC supply; see the ceiling caveat above). A distribution /
    merge-priority / head-hog-fairness model change therefore **cannot close
    this residual.**
  - **Both minor terms, quoted on one base.** Against the 0.271/s sim-relative
    gap: **cycle order ≈14%** (+0.038/s) and **distribution ≈5%** (+0.013/s).
    Note the ordering — the cycle order is the *larger* of the two, ~2.9×
    distribution, despite being the hypothesis retired first. Earlier
    revisions quoted cycle order as "+2.2%" (of throughput) against
    distribution's "≈5%" (of the gap), two different denominators that made
    the smaller term look bigger. Neither is the cause: together they account
    for under a fifth of the gap — and that 19% is an **upper bound, not a
    sum of independent terms**: if the cycle-order gain arrived through EC
    supply (see the ⚠ block above), it partly overlaps the dominant stage
    rather than adding to it.
  - **The gap against each base, kept separate.** Meter 1.716/s is **0.271/s**
    below the sim's 1.987/s (the −13.6% sim-relative residual) and **0.284/s**
    below the 2.0/s ideal. Different denominators; earlier revisions of this
    entry quoted a single "~0.29/s ≈ −13%" that welded the ideal-relative
    magnitude to the sim-relative percentage.
  - **RETRACTED (2026-08-05, third revision) — the "imputed 47.7 EC/s"
    argument.** An earlier revision reasoned: the sim delivered 1.987 PU/s, a
    PU takes 24 EC, therefore the sim moved ≈47.7 EC/s and its *reported*
    43.2/s must undercount by ~9%. That is falsified by the sim's own
    copper-cable measurement, which nobody had cross-checked. EC takes 3
    cable and AC takes 4, so the sim's reported numbers imply a cable demand
    of 3×43.2 + 4×3.59 = **143.96/s** against a measured **143.9/s** — a
    0.04% match. The imputed 47.7 would need 157.5/s, 9.4% above what was
    measured. **The sim's EC figure is corroborated by an independent
    measurement in the same run; the imputation is not.** Taking 43.2 at face
    value, the meter's EC is only **−3.9%** — inside its own ±4% band — and
    the "EC/plate production is the divergence" story evaporates with it.
  - **⇒ Leading hypothesis (2026-08-05): a tech-state parity gap, not a
    layout or belt defect at all.** Two facts in-repo make it the natural
    reading: `crates/sim-harness/src/scenario.rs` calls
    **`force.research_all_technologies()`** and its tech-state parity block
    corrects **only** inserter capacity (#370) and belt stacking (#385) —
    nothing corrects productivity; and `crates/meter/src/machine.rs`
    documents that it takes *nothing* from `module_policy` and not
    `effective_crafting_speed`, so the meter models **no productivity at
    all**.
  - **The sim's own numbers pin the size and the scope, three ways.** An
    earlier revision read the bonus off a blended ratio (43.2 EC ÷ 1.987 PU =
    21.74 against a flat 24). That is loose — the flat 24 assumes AC:PU = 2,
    and this fixture runs 1.807. Doing it properly is both tighter and more
    informative, because **the AC:PU ratio is itself the productivity
    signature**:
    1. **From AC production alone.** A PU recipe execution consumes 2 AC, so
       AC:PU below 2 means executions are producing more than one PU each:
       `2 / 1.807 − 1` = **+10.7% productivity on PU**. This uses no EC figure
       at all.
    2. **The EC balance then decides AC's status.** With PU at +10.7% and AC
       *unboosted*: `20 × 1.987/1.107 + 2 × 3.59` = **43.08/s** against 43.2
       measured (−0.3%). With AC *also* boosted it gives 42.39/s (−1.9%) — a
       visibly worse fit. **AC is unboosted.**
    3. **The cable balance says the same for EC.** `3 × 43.2 + 4 × 3.59` =
       **143.96/s** against 143.9 measured (+0.04%), i.e. the full recipe
       ratio. Boost EC/AC by 10% and demand drops to ~130.9/s, 13/s under
       supply. *Caveat on this one*: cable production also equals `2 ×
       copper-plate` exactly (2 × 71.98 = 143.96), so supply and unboosted
       demand coincide here; the argument rests on a 13/s surplus having
       produced visible backpressure, which it did not. Suggestive, not
       airtight — unlike (1) and (2).
    **So the scope question is answered by the data rather than by game
    knowledge: PU is boosted ≈+10.7%, EC and AC are not.** That also removes
    the double-counting worry — the meter's −3.9% EC deficit *is* an
    independent term. No claim about which Space Age technologies exist is
    needed for any of this, and the earlier "there is no EC/AC productivity
    tech" assertion is withdrawn as the same class of unverified outside-repo
    claim it was criticising.
  - **Decomposition, on the corrected number.** The meter's −3.9% EC deficit
    compounded with the −9.7% productivity it does not model
    (`1/1.107 − 1`) gives **−13.2%**, against **−13.6%** observed — a 0.4pp
    residual, comfortably inside this fixture's noise.
  - **⚠ STILL NOT VERIFIED, and this entry has been wrong three times.** What
    the arithmetic establishes is that *the sim behaves as if* PU carries
    ≈+10.7% and its inputs carry none. **Why** is still attribution: a
    productivity research granted by `research_all_technologies()` is the
    obvious candidate, but that call does not research infinite/repeatable
    technologies, and Space Age's productivity researches are repeatable — so
    the mechanism may not be the obvious one, and modules (gated by the
    `module_policy` the meter ignores, and defaulting to `None`) are the
    alternative. The settling check is one run: dump the force's realized
    productivity bonus for `processing-unit`, `electronic-circuit` and
    `advanced-circuit`, the same self-audit pattern the inserter and
    belt-stacking parity blocks already use. Note what the balances above do
    and do not buy: they corroborate **the sim's reported EC figure and the
    unboosted status of EC/AC**, which is what killed revision three — they do
    not by themselves establish the *cause* of PU's bonus.
- **Verdict**: **most likely not a meter modelling defect in the layout sense
  at all — an instrument/reference parity gap.** The sim researches everything
  and the meter models no productivity, so on a productivity-eligible recipe
  the two are measuring different worlds on any recipe the sim actually boosts
  (which recipes those are is the open question above). That is the same
  failure class as
  the inserter-capacity (#370) and belt-stacking (#385) parity fixes already
  in the scenario, and the fix would be the same shape: either set the sim's
  productivity to the fixture's declared level, or teach the meter the
  fixture's productivity — *not* a distribution, merge-priority or belt-model
  change. **Deferred**, and deliberately not "fixed" on the strength of
  arithmetic alone: this entry has now proposed three root causes
  (belt-cycle order, head-hog distribution, upstream EC/plate production) and
  retired all three, so the bar for the fourth is a measurement, not another
  derivation — and the fourth already has a known soft joint (the EC/AC
  question above) rather than being clean. Tracked as item 7 in `rfc064-phase2-followups.md`.
- **Next**: run the fixture with the force's `processing-unit` productivity
  bonus dumped into the report, and compare against the meter's implicit zero.
  If it confirms ≈10%, the item closes as a parity gap and the remaining
  ≈0.9pp goes back into the noise budget. Only if it *disconfirms* does the
  upstream-supply question reopen — and then with `timeseries` captured, so
  per-machine detail exists on the reference side. Do **not** re-derive a
  fourth cause from the same three numbers.

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
