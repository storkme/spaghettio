# Meter divergence log (#570, RFC-054 Phase B/C)

Running record of where the fast meter's `produced_per_s`/`delivered_per_s`
diverges from the measured headless-Factorio sim by more than ±10pp, and why
that divergence is believed to live (model gap vs. known-open-item). Updated
when the corpus sweep (`crates/meter/examples/sweep_corpus.rs`) moves a number
or reveals a new one. One residual remains: its **diagnosis** closed
2026-08-06 (an instrument-parity gap, not a model defect), its **fix** is
decided but unmerged.



## 2026-08-07 — corpus-wide calibration: the meter is safe as a FLOOR

Swept all 70 corpus layouts (35 fixtures × native/compact) with
`sweep_corpus`; **41** have a sim baseline to compare against.

**Distribution**: 25 pessimistic (meter < sim), 6 optimistic, 10 at ~0pp.
Mean −1.27pp, median −0.3pp. **Every optimistic error is small — max
+1.3pp anywhere in the corpus** — while pessimistic errors run to −13.6pp.
When the meter is wrong by a meaningful margin it is *always* wrong in the
safe direction.

**The gate question.** Classifying each row at-plan vs below-plan for both
instruments, the dangerous quadrant (meter says AT plan, sim says BELOW) is
**empty at every realistic tolerance** — 0 rows at 99%, 98%, 95% and 90%.
It appears only at a literal bit-exact 100% cutoff, where its 4 hits are sim
readings of 99.0–99.7% against a meter reading of exactly 100.0%, i.e. inside
the same ~1pp band that agreeing fixtures show as noise.

So: **"meter says below plan" ⇒ believe it** holds throughout, and is the
property a gate needs. **"meter says at plan" ⇒ evidence of nothing** remains
the correct caution: 3 of 23 meter-at-plan fixtures were softer sim passes
(96–99.4% meter vs 99–102% sim). Never a real miss, but clearance semantics
would overclaim precision the corpus doesn't support.

**The −13.6pp outlier is the known item, not a new one.** Both
`tier5_processing_unit_from_ore_am3` rows (−13.6 / −12.8pp) match this
document's existing ≈−13% entry for the unmerged research-productivity axis.
It is not a fluid gap: fluid-*ingredient* fixtures sit at −2.2 to +1.0pp.

### Two things this does NOT establish

1. **Fluid targets are untested.** All 4 fluid-target fixtures
   (heavy-oil-cracking, sulfuric-acid, 2× AOP) have **no sim baseline** in
   this corpus. The floor verdict is a **solid-target-only** result.
2. **The tier2 entry above is a different layout, and flips direction.**
   The corpus's `tier2_electronic_circuit` is the pre-lift zero-headroom
   layout: meter 56.0% vs sim 57.7–58.1% — **pessimistic**, in line with
   everything else. The 96%/90–91% figures recorded above are the *re-ranked
   post-lift* layout, which is not in this corpus — and there the meter is
   **optimistic by ~5–6pp**, which would be **four times the largest
   optimistic error in the entire 41-row corpus**.

   Those are not the same data point and must not be averaged. The
   post-lift layout is either exercising something the meter over-credits,
   or that single comparison is unsound. **Until that is resolved, the
   floor property is established for the corpus and NOT for post-lift
   layouts** — which is precisely the population a gate would run on.
   This is the open question; resolving it needs a sim-baselined post-lift
   corpus, not more analysis of the old one.

## 2026-08-07 — tier2_electronic_circuit: meter 96%, sim 90–91% (open)

First divergence recorded from the *plan* side rather than the sim side, and
it is **under** the ±10pp bar, so it is a precision note rather than a defect
report.

| | copper-cable | electronic-circuit |
|---|---|---|
| planned | 30.0/s | 10.0/s |
| meter | 28.8/s (96%) | 9.6/s (96%) |
| sim | 27.0/s (90%) | 9.09/s (91%) |

**The meter got the important part right**: it said *below plan* on both
stages, which is the verdict that matters, and it said it in **19 seconds**
against ~10 minutes for the headless run. The underlying cause is
zero-headroom integral machine counts (`status.md`) — copper-cable plans at
exactly 10.0 machines, so any duty loss becomes a permanent shortfall. The
meter sees it because it models inserter swing and lost swings.

It is **~5pp optimistic**, and that direction is the one that matters for
any future use as a gate: a predictor that understates a deficit is safe as
a **floor** ("meter says below plan" ⇒ believe it) and unsafe as clearance
("meter says at plan" ⇒ not yet evidence).

**A candidate hand-size discrepancy was raised here and is now RETRACTED
(same day, investigated against primary prototype data).** It claimed the
meter's `BULK_HAND_BY_LEVEL` was one research level behind, on the reasoning
that at `inserter_capacity = 2` the game realizes `bulk_inserter_capacity_bonus
= 3`, so the hand should be `base_hand_size() + bonus = 2 + 3 = 5` where the
meter returns 4.

**That reasoning was wrong, and the meter is correct at every level 0–7.**
`base_hand_size()` *is* `hand_size(0)` — it already contains L0's bonus
(1 raw prototype floor + 1 from the `bulk-inserter` tech = 2). Adding the
force bonus on top double-counts that L0 increment. The true decomposition
is `hand = 1 (raw prototype floor) + bonus`, giving `1 + 3 = 4` — exactly
what `BULK_HAND_BY_LEVEL[2]` returns.

Primary sources (Factorio 2.0.77 install, not the wiki and not our docs):

- `data/base/prototypes/entity/entities.lua` — `bulk-inserter` sets
  `bulk = true` and has **no** `stack_size_bonus`, so its raw floor is 1.
- `data/base/prototypes/technology.lua` — the `inserter-capacity-bonus-N`
  chain carries Wube's own `-- result of N` comments, and every one equals
  `1 + cumulative_bonus`. Those reproduce `BULK_HAND_BY_LEVEL` and
  `NON_BULK_HAND_BY_LEVEL` exactly at L0–L6.
- `data/space-age/.../entities.lua` — `stack-inserter` is `bulk = true` with
  a literal `stack_size_bonus = 4`, i.e. `5 + bulk_bonus`, which is exactly
  the meter's `BULK_HAND_BY_LEVEL[level] + 4`.

The L7 non-bulk value is a **deliberate** divergence, already documented in
`entity_data.rs`: `transport-belt-capacity-2` is a separate Space-Age tech on
a different branch that grants a further `+1` non-bulk, so a
`research_all_technologies()` force reads 3 where the declared axis says 2.
`scenario.rs` overrides the force bonuses by direct assignment specifically to
keep that contamination out — the two agree by design.

**Lesson worth keeping**: `base_hand_size()` reads like an additive constant
and is actually `hand_size(0)`. Its doc comment says so; I misread it anyway.
This is the second time this table has attracted a wrong "fix" (PR #458 was
the first), which is why `entity_data.rs` says *transcribe, don't derive*.

So the ~5pp optimism remains **entirely unexplained** — this ladder has no
bearing on it in either direction.

## Corpus status (2026-08-06; Phase B landed 2026-08-03)

`meter sweep: 70 layouts measured, 41 compared`. Every compared fixture is
within ±10pp of sim except the one below. AC/PU families that were −80% in
Phase A are now within ±2% (the dedicated AC fixtures) / −13% (PU-from-ore; its
in-fixture AC reads −3.9%).

## Residual: diagnosis closed 2026-08-06, fix open

### `tier5_processing_unit_from_ore_am3` — meter ≈ −13% (`produced`) — diagnosis CLOSED, fix OPEN

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
  meter: **the solver did not model research productivity either.**
  *(Implemented 2026-08-06 — `netflow.rs` now folds a declared per-recipe
  research bonus into `effects.prod_bonus` alongside module and base-effect
  productivity. The three parts land as #584 meter / #585 sim / #587 solver.
  Note the plan does NOT scale uniformly by 1/(1+bonus): a stage that carries
  its own bonus AND serves reduced downstream demand compounds — plastic-bar
  scales 1/1.21 on this fixture, basic-oil-processing 1/1.19. Correct
  arithmetic; an earlier write-up of mine claimed "every stage scales by
  exactly 1/1.1", which is only true of the boosted target stage.)*
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

- **Prediction, and its result (2026-08-06).** Teaching the meter +10% on PU
  was predicted to land its output at **≈1.902 PU/s**. Implemented on
  `feat/research-productivity-axis` and measured on this fixture at the Stage B
  deep-chain warmup:

  | config | PU | EC | ceiling@EC | % of ceiling |
  |---|---|---|---|---|
  | declared none | 1.7056/s | 41.49 | 1.7287 | 98.7% |
  | declared +10% | **1.8500/s** | 41.49 | **1.9018** | 97.3% |

  **The ceiling under productivity measures 1.9018 against the predicted
  1.902** — the productivity model is exactly right. What the prediction got
  wrong is assuming the meter would *reach* its ceiling.

  **What the shortfall actually is — AC over-production, not distribution.**
  The `E/24` ceiling assumes AC is produced exactly in proportion (2 per PU
  craft). It is not. Computing against the *measured* AC instead —
  `(E − 2·AC)/20 × 1.1` — fits far better:

  | config | observed | `E/24` | net of measured AC |
  |---|---|---|---|
  | none | 1.7056 | 1.7288 (98.7%) | 1.7123 (**99.6%**) |
  | +10% | 1.8500 | 1.9016 (97.3%) | 1.8407 (**100.5%**) |

  Because AC runs over: **2.12 AC per PU craft baseline (+6%), 2.39 with
  productivity (+19%)**, against the recipe's 2. That surplus consumes EC which
  would otherwise reach PU, and it is not a distribution loss — it is the
  **solver's** blind spot to research productivity showing up downstream. The
  plan sizes the AC stage for a PU stage that needs no productivity; give PU
  +10% and it needs fewer crafts, so the unchanged AC sizing over-serves it.
  An earlier revision of this entry attributed the gap to distribution; that
  was wrong, and it was caught by a reviewer pointing out that `24 EC per
  craft` is a full-chain figure rather than a per-craft one (a PU craft
  consumes 20 EC directly plus 2 AC).

  So the residual after the meter-side fix is **the other half of the same
  bug**, and closing the solver side should close most of what is left.
  Note also that EC production is **identical (41.49/s) in both configs**, so
  declaring productivity does not move it — EC is genuinely unboosted and
  supply-limited, corroborating the probe's `electronic-circuit: 0.0%`
  independently.
- **Still open after the meter fix**: the −2.7% ceiling shortfall (the
  distribution term, which grows slightly as the same EC supply feeds more
  output) and the −3.9% EC deficit itself. The latter is where the **solver's**
  own blind spot to research productivity lands: `netflow.rs` models modules and
  `base_effect` only, so the plan is over-provisioned by the same factor. That
  is the other half of the fix — **now implemented** (#587); what remains is
  wiring a caller to declare a real value, which changes plans and is
  deliberately separate.
- **Superseded**: this bullet previously predicted the residual would fall to
  ≈−4.3%, "essentially the −3.9% EC deficit alone". The measurement above puts
  it at −6.9%: the prediction's *ceiling* was right to four figures, but it
  assumed the meter would sit on that ceiling, and it does not. The trap it
  warned about still stands and is worth keeping — the 1.729 figure quoted
  elsewhere is a ceiling *at zero productivity*; productivity does not change
  EC consumed per PU including its AC leg (still 24; a craft itself takes 20 EC plus 2 AC), it changes PU produced per craft (1 → 1.1),
  so it raises the ceiling rather than capping output beneath it.

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
