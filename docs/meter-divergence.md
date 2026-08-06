# Meter divergence log (#570, RFC-054 Phase B/C)

Running record of where the fast meter's `produced_per_s`/`delivered_per_s`
diverges from the measured headless-Factorio sim by more than ±10pp, and why
that divergence is believed to live (model gap vs. known-open-item). Updated
when the corpus sweep (`crates/meter/examples/sweep_corpus.rs`) moves a number
or reveals a new one. The last open residual closed 2026-08-06 —
measured as an instrument-parity gap, not a model defect.

## Corpus status (2026-08-06; Phase B landed 2026-08-03)

`meter sweep: 70 layouts measured, 41 compared`. Every compared fixture is
within ±10pp of sim except the one below. AC/PU families that were −80% in
Phase A are now within ±2% (the dedicated AC fixtures) / −13% (PU-from-ore; its
in-fixture AC reads −3.9%).

## Closed residual (2026-08-06)

### `tier5_processing_unit_from_ore_am3` — meter ≈ −13% (`produced`) — CLOSED

- **meter**: processing-unit 1.716/s vs sim produced 1.987/s / delivered 1.961/s.
- **Direction**: **not a layout/belt divergence at all** — measured
  2026-08-06 as a productivity tech-state gap between the sim and the meter
  (see the ⇒ bullet). Four earlier readings were proposed and retired in turn:
  belt-cycle update order (08-04), head-hog distribution (08-05), upstream
  EC/plate production (08-05), and a sim-reporting-artifact reading (08-06,
  killed by the probe). Deep-dive (2026-08-03) established the surrounding
  figures precisely, and those still stand:
  - The **sim itself** under-produces almost everything on this fixture
    (copper-plate 71.98/80, copper-cable 143.9/160, EC 43.2/48, plastic 7.2/8 —
    all ≈ −10%; petroleum-gas −17%); only the PU target reaches 99%. So plan is
    not the reference; the sim is. (Intermediates here are *measured production*
    and do not always reconcile arithmetically with consumers. EC demand from
    PU+AC reads ~46.9/s at the measured operating point against ~43/s reported
    produced (48/s is the *plan* rate), but **that 46.9 is computed at zero
    productivity** — 20 EC per PU *unit*. At the +10% this entry settles on,
    1.987 PU/s takes 1.987/1.1 = 1.806 crafts/s, so demand is
    1.806×20 (direct) + 3.61 AC/s ×2 = **≈43.3/s** — *smaller* than the
    zero-productivity figure, and consistent with the 43.2/s the sim reports
    producing. (An earlier revision of this line published ≈50.5/s by taking
    1.987×24/1.1 **and** adding AC's 3.59×2 again: the 24 already bundles AC's
    share, so that double-counted it — and 50.5 flatly contradicted this
    entry's own 0.35% reconciliation two bullets down, which would have made
    the sim EC-starved to ~86% and unable to deliver the PU it measured. The
    same welding-two-bases error this whole entry exists to correct.)
    The copper-plate→cable pair does reconcile (1 plate → 2 cable). Treat the
    intermediate figures as indicative, not load-bearing for the deferral's
    rationale.)
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
  - **⇒ MEASURED AND SETTLED (2026-08-06): a productivity tech-state parity
    gap.** No longer a hypothesis. The sim harness now dumps the realized
    productivity bonus (PR #580, **merged 2026-08-06** — the verification channel
    this axis lacked;
    the parity block corrected inserter capacity (#370) and belt stacking
    (#385) and nothing else). Run against this fixture:

    | recipe | realized force/research productivity |
    |---|---|
    | **processing-unit** | **+10.0%** |
    | **plastic-bar** | **+10.0%** |
    | advanced-circuit | 0.0% |
    | electronic-circuit | 0.0% |
    | iron-plate | 0.0% |
    | copper-plate | 0.0% |
    | copper-cable | 0.0% |
    | sulfur / sulfuric-acid / basic- + advanced-oil-processing | 0.0% |

    **`plastic-bar` was missed by the first probe** and found only when the
    review pushed the coverage wider: plastic is crafted in a *chemical plant*,
    and both the probed recipe list and the entity-type filter covered only
    assembler/furnace legs. The same wrong-population mistake as the "0 of 1889
    unlabelled tiles" triage earlier in this campaign — an accurate count over
    exactly the cases where the thing cannot appear. It does **not** move the
    decomposition below (meter and sim both deliver 7.2 plastic/s, so the boost
    changes the sim's *petroleum input per plastic*, not its plastic output),
    but "processing-unit alone" was a claim the first probe could not support.
    **One consequence is asserted rather than checked**: at +10% the sim makes
    its 7.2 plastic/s from ~9% less petroleum than the meter models (≈65.5 vs
    ≈72/s — plastic-bar is 20 petroleum → **2** plastic, i.e. 10 per unit; an
    earlier revision doubled both figures by treating it as 20 per unit). The sim's petroleum-gas was already reported −17%, so if that
    input ever binds this is a second parity gap of the same class. It almost
    certainly does not bind here (PU reaches 99%), but nobody has verified the
    sulfur/oil leg against it.

    `productivity_modules: {}` — empty, so the source is **research**, not
    modules.
  - **The sim's own figures reconcile under it, to 0.35%.** A check worth
    running because it uses none of the disputed quantities: at +10%, 43.2 EC/s
    ÷ 24 EC per craft = 1.800 crafts/s × 1.1 = **1.980 PU/s**, against the
    **1.987** the sim measured. The sim's EC production and its PU output are
    consistent with each other *only* under the measured bonus — at zero
    productivity the same EC would cap it at 1.800 PU/s, 9% below what it
    delivered. This is independent of the AC:PU route below and of the probe
    itself. `force.research_all_technologies()` does grant it, which also
    settles a doubt recorded here on 08-05 about repeatable technologies.
  - **What that closes.** The meter models no productivity at all (deliberately
    — `crates/meter/src/machine.rs` takes nothing from `module_policy` and not
    `effective_crafting_speed`), so on PU the instrument and its reference are
    measuring different worlds. **Decomposition**: the meter's −3.9% EC deficit
    compounded with the −9.1% productivity it does not model = **−12.7%**
    against **−13.6%** observed, ~1pp inside this fixture's noise.
    The measurement also confirms the **selectivity** the corpus had been
    showing all along — EC and AC unboosted, hence their **single-digit** deviation (EC −3.9%, AC −3.9% — *not* the ±0–2% an earlier
    revision of this line claimed, which the same file's own figures contradict), while the two
    boosted recipes are PU (which diverges) and plastic-bar (whose output does
    not, for the reason above) — and **kills the competing reading** that the signature was a
    sim-side reporting artifact. There is a real bonus.
  - **Scoreboard for the prediction.** The AC:PU ratio (1.807 against the
    recipe's 2) predicted **+10.7%** before the run; measured **+10.0%**. The
    0.7pp is the sim's own noise on the AC figure. Recorded because the
    arithmetic route was right and the three preceding *narrative* root causes
    were not — the discipline that paid was balancing the sim's numbers against
    each other, not reasoning about mechanism.
  - **⚠ Provenance of the probe run.** It used `--warmup 600` to reach finalize
    quickly. Its **throughput numbers are buffer-fill artifacts** and must not
    be compared against the recorded baselines above. Only the productivity
    fields are load-bearing: they are force state set at init and
    warmup-independent. (Factorio 2.0.77, matching `PINNED_VERSION`.)
- **Verdict**: **an instrument/reference parity gap, measured** — not a meter
  modelling defect in the layout sense, and not a layout, belt, distribution or
  supply defect. The sim runs processing-unit at +10% research productivity;
  the meter models none. Same failure class as the inserter-capacity (#370) and
  belt-stacking (#385) parity fixes already in the scenario, and the fix is the
  same shape: align the sim's productivity to the fixture's declared level, or
  teach the meter the fixture's productivity. **Which of those two is a
  separate call**, now made on evidence rather than inference.
  Four causes were proposed and retired before this one — belt-cycle update
  order (≈14% of the gap), head-hog distribution (≈5%), upstream EC/plate
  production (falsified by the cable balance), and a fourth reading in which
  the signature was a sim reporting artifact (falsified by the probe). The
  thing that finally settled it was a measurement, not a fifth derivation.
- **Next**: decide the fix direction (sim-side parity assignment vs meter-side
  productivity model). Nothing further to diagnose here.
  **Falsifiable prediction.** Teach the meter +10% on PU and its output should
  land at **≈1.902 PU/s** — 41.5 EC/s ÷ 24 = 1.729 crafts/s, each yielding 1.1
  PU — i.e. a residual of **≈−4.3%** against the sim's 1.987, essentially the
  −3.9% EC deficit alone.
  Note the trap in that arithmetic, since a reviewer fell into it: the 1.729
  figure quoted elsewhere in this entry as the meter's "ceiling" is a ceiling
  *at zero productivity*. Productivity does not change EC consumed per craft
  (still 24) — it changes PU produced per craft (1 → 1.1), so it raises the
  ceiling rather than leaving output capped beneath it. Reading 1.729 as a cap
  that survives the fix predicts a residual of ≈−13% and concludes the
  productivity story cannot work; that is arithmetic on a stale ceiling, not a
  contradiction in the evidence.

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
