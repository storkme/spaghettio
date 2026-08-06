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
- **Cause (measured 2026-08-06)**: a **productivity tech-state parity gap**
  between the sim and the fast meter — not a layout, belt, distribution or
  supply defect. The sim calls `force.research_all_technologies()`, and its
  tech-state parity block corrects only inserter capacity (#370) and belt
  stacking (#385); nothing corrects productivity. The meter models no
  productivity at all, deliberately (`crates/meter/src/machine.rs` takes
  nothing from `module_policy` and not `effective_crafting_speed`). So on a
  boosted recipe the instrument and its reference measure different worlds.

  Realized force/research productivity, dumped by the harness probe (PR #580):

  | recipe | bonus |
  |---|---|
  | **processing-unit** | **+10.0%** |
  | **plastic-bar** | **+10.0%** |
  | advanced-circuit, electronic-circuit, iron-plate, copper-plate, copper-cable, sulfur, sulfuric-acid, basic-/advanced-oil-processing | 0.0% |

  `productivity_modules: {}` — empty, so the source is research, not modules.

  **Corroboration, out-of-sample**: the +10% came from a force-state read, not
  from any rate figure, and it correctly predicts the relationship between two
  independently measured rates — 43.2 EC/s ÷ 24 × 1.1 = **1.980 PU/s** against
  the **1.987** the sim measured, 0.35% apart. At zero productivity that same
  EC caps output at 1.800 PU/s, 9% below what it delivered. This is the check
  the four retired causes never had.

- **Decomposition**: the meter's −3.9% EC deficit compounded with the −9.1%
  productivity it does not model = **−12.7%** against **−13.6%** observed,
  ~1pp inside this fixture's noise (every intermediate here is ≈10% off plan).
  Only PU's boost moves the target: meter and sim both deliver 7.2 plastic/s,
  so plastic's boost changes the sim's petroleum input rather than its output.

- **⚠ Provenance of the probe run**: it used `--warmup 600` to reach finalize
  quickly, so its **throughput numbers are buffer-fill artifacts** and must not
  be compared against the recorded baselines above. Only the productivity
  fields are load-bearing — force state set at init, warmup-independent.
  (Factorio 2.0.77, matching `PINNED_VERSION`.)

- **Four causes were proposed and retired before this one.** Kept as a list
  because the bounds are the reusable part; the full derivations were removed
  on 2026-08-06 after they had cost more review rounds than they were worth.
  - *Belt-cycle update order* — permuting the 26-tile cyclic order moves PU
    1.716→1.754/s, ≈14% of the 0.271/s gap. Real, not the cause.
  - *Head-hog distribution* — 12/16 PU machines run at full rate while the four
    deepest starve, but perfectly redistributing the available EC yields only
    1.729/s: **+0.013/s, ≈5% of the gap**, at fixed EC supply. (That ceiling is
    an operating point, not an invariant — the permuted run reached 1.754/s,
    which needs more EC than the baseline produced, so EC production itself
    responds to belt-model changes.)
  - *Upstream EC/plate production* — rested on imputing 47.7 EC/s from the sim's
    PU output. Falsified by the sim's own copper-cable balance: 3×43.2 + 4×3.59
    = 143.96/s against 143.9/s measured, while the imputed figure needs 157.5/s.
  - *Sim-side reporting artifact* — falsified by the probe: there is a real
    bonus.

- **Verdict**: an instrument/reference parity gap, same class as the
  inserter-capacity (#370) and belt-stacking (#385) fixes already in the
  scenario.
- **DECIDED (owner, 2026-08-06): teach the meter productivity.** Rationale
  recorded verbatim-in-spirit — *"it's important for the meter to be able to be
  flexible, and for it to match what we actually produce as measured in the
  sim."* The sim stays the reference; the instrument learns to model what the
  reference actually does.

  One scoping fact found while writing this up, which widens the fix beyond the
  meter: **the solver does not model research productivity either.**
  `netflow.rs` applies productivity from *modules* and a machine's
  `base_effect`, gated on the recipe's `allow_productivity`, and
  `ModulePolicyKind` defaults to `None`. Research-sourced productivity — the
  +10% the sim actually runs with — is modelled nowhere on the engine side. So
  the *plan* is over-provisioned on PU by the same 10% as the meter's
  prediction is short; this is not only an instrument gap.

  Shape the fix should take, following the pattern #370 and #385 already
  established for exactly this problem: make research productivity a
  **declared axis** carried on the manifest alongside `stacking` and
  `inserter_capacity`, so that (a) the meter applies it, and (b) the sim
  *pins* it in its tech-state parity block instead of inheriting whatever
  `research_all_technologies()` grants. Declaring it on both sides makes them
  match by construction rather than by coincidence — a meter that models a
  declared level while the sim runs an incidental one agrees only by luck.

- **Next / falsifiable prediction**: teach the meter +10% on PU and its output
  should land at **≈1.902 PU/s** — 41.5 EC/s ÷ 24 = 1.729 crafts/s, each
  yielding 1.1 PU — i.e. a residual of **≈−4.3%**, essentially the −3.9% EC
  deficit alone. Note the trap: the 1.729 figure is a ceiling *at zero
  productivity*. Productivity does not change EC consumed per craft (still 24),
  it changes PU produced per craft (1 → 1.1), so it raises that ceiling rather
  than capping output beneath it.

- **Open, not closed**: (a) the ~1pp decomposition residual is attributed to
  fixture noise, not explained; (b) plastic-bar's input-side consequence is
  asserted, not checked — at +10% the sim makes its 7.2 plastic/s from ~9% less
  petroleum than the meter models (≈65.5 vs ≈72/s), and the sim's petroleum-gas
  was already −17%, so if that input ever binds it is a second gap of the same
  class.

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
