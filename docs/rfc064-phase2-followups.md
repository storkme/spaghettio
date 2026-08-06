# RFC-064 Phase 2 — follow-ups

**Status 2026-08-01:** Phase 2 Stage B subset sim campaign adjudicated —
never-worse HOLDS on the measurable subset (gate GREEN, representative scope);
tuning adopted (speed 32, deep warmup 30 game-min). Open below: the surfaced
findings that need engineering or a decision. The two layout/rig defects are
tracked together in **#567**; the dropped duplicate stress fixtures are being
re-run at the tuned settings (this file's item 6, in progress).

## Open items (pick-up notes)

### 1. stress-EC high-rate throughput ceiling — ROOT CAUSE CONFIRMED (in-game) + fix spec (#567)
The full stress-family sweep (native ≈ compact throughout — never a compaction
regression) caps high-rate EC-from-ore at ~one belt's throughput: `22s`
15.00/15.00, `23s` 15.10/15.10, `30s` 15.15/15.15, `60s-red` 30.5/30.0, and
`35s`/`40s` do not converge (census 140–200 `full_output` ↔ `item_ingredient_shortage`
surge/jam). **Root cause confirmed in-game:** `bus/output_merger.rs::merge_output_rows`
reduces every row's output column to a **single belt** via the "sequential
splitter cascade merging N south columns into 1" (lines 309–385), capped at
`max_belt_tier` (line 58) — so a target over one belt (at the chosen tier) is
throttled to one belt's throughput (yellow 15 / red 30) → the visible back-up +
`full_output` cascade. (The earlier "single-exit boundary dedup" and "input
delivery" hypotheses were superseded by this — the merger is the choke.)
**Fix speced on #567 (scoped, not yet implemented):** reduce the cascade to
`n_output = ceil(total_rate / belt_throughput(belt_name))` columns and route/
drain N parallel output belts, emitting N `boundary_outputs`; byte-identical for
all under-cap fixtures. Requires full layout-engine verification (e2e + sim
re-measure + snapshot + eyeball + adversarial review; USP mega-chain / RFC-052 /
#309 forensics must not regress). NOTE: `tier2_electronic_circuit`'s 5.77/s is a
DIFFERENT cause (the #519 input-rate-delivery/inserter class), not this fix.

### 2. Bio / fluid self-loop fixtures are unmeasurable in the current harness rig
`bacteria_self_loop_regression` measured **0/s, `no_fuel`** (validating the
same class as `pentapod_fish`/`fish_breeding`); `sulfuric-acid`/`heavy-oil-cracking`
starve on `fluid_ingredient_shortage` (water never reaches the plant, per
sim-state). The validator reports these clean, yet the sim shows them dead —
a gap in BOTH the harness rig (fuel/nutrient/fluid delivery to these machines)
and validator coverage (the check that would catch it). **Action:** track the
fluid/fuel-delivery fix (the earlier `#363 non-south-feed` thread is adjacent);
until fixed, keep these on the sim skip-list so they don't burn hours of
0-output grind. Do NOT read "both sides 0" as a compaction pass on them.

### 3. Robustness fuzz — rewrite memory-bounded and rerun
The whole-recipe-DB fuzz (Job 3) that OOMed the box (22 GB RSS, killed the
session + the concurrent sims) is worth doing, but **only** after a
memory-bounded rewrite: no per-cell spawned threads, single/low concurrency
(max ~2 concurrent solves at once), a per-cell size/time guard. value: maps the
refusal/panic surface (RFC-064 tetris direction's declared risk). The
`crates/core/examples/rfc064_fuzz_robustness.rs` driver is the base; it needs
concurrency + memory bounding before any rerun.

### 4. Adaptive warmup (rate-of-change driven) — cut sim time further
Stage B's tuning showed the blanket deep warmup was ~2.7× over-conservative and
that a warmup **sweep** (finding where the per-window rate flattens) right-sizes
it. Natural follow-up: surface the produced-rate the scenario already collects
during warmup so a watchable `--timeseries` run can stop warming the moment the
transient decays (true adaptive early-exit), instead of a fixed bound. Harness
change; see `docs/sim-harness.md` "Live progress telemetry".

### 5. Baseline bank blessing
The measured native baselines in the Job-2 corpora should be `bless`ed
(playbook: a "complete baseline bank makes every future layout change
regression-checkable"). Do NOT bless during a live campaign; do it from the
settled artifacts, with provenance stamped.

### 6. Remaining full-bill fixtures (the dropped duplicates)
The representative-subset scope deliberately dropped the duplicate stress
fixtures (EC 22/23/35/40_s, decomposed pooled/partitioned, AC 4s/5s pooled,
uranium voider). With the tuning now making sim ~2.7× cheaper, closing the
"corpus-wide" gap by re-running these at speed 32 + 30 game-min is affordable
if the coordinator wants the stronger gate. Otherwise the subset verdict stands
as recorded.

### 7. Meter productivity-parity residual: PU-from-ore −13% — **diagnosis CLOSED 2026-08-06, fix OPEN**
**Measured, not inferred.** The sim harness now dumps realized productivity
(PR #580 — the verification channel this axis lacked; the tech-state parity
block covered inserter capacity (#370) and belt stacking (#385) and nothing
else). Against `tier5_processing_unit_from_ore_am3`:

| recipe | realized force/research productivity |
|---|---|
| **processing-unit** | **+10.0%** |
| **plastic-bar** | **+10.0%** |
| advanced-circuit | 0.0% |
| electronic-circuit | 0.0% |
| iron-plate / copper-plate / copper-cable | 0.0% |

No productivity modules anywhere, so the source is
`force.research_all_technologies()`. The fast meter models no productivity at
all by design (`crates/meter/src/machine.rs` deliberately takes nothing from
`module_policy` and not `effective_crafting_speed`), so on this recipe the
instrument and its reference measure different worlds.

**Decomposition**: the meter's −3.9% EC deficit compounded with the −9.1%
productivity it cannot model = **−12.7%** against **−13.6%** observed, ~1pp
inside the fixture's noise. The measurement also explains the selectivity —
EC/AC unboosted, hence their single-digit deviation (both −3.9%, not the
±0–2% an earlier revision claimed) while only PU diverged — and kills the competing
reading that the signature was a sim-side reporting artifact.

**This is an instrument-parity gap, not a layout, belt, distribution or supply
defect.** Same class as #370 and #385. **DECIDED (owner, 2026-08-06): teach the meter productivity** — the sim stays
the reference and the instrument learns to model what it actually does.
Widened scope found while recording it: the **solver** does not model
research-sourced productivity either (`netflow — **implemented 2026-08-06 (#587)**; only caller wiring remains.rs` covers modules and
`base_effect` only), so the plan is over-provisioned on PU by the same 10%.
The fix should follow #370/#385: carry research productivity as a declared
manifest axis beside `stacking`/`inserter_capacity`, applied by the meter and
*pinned* by the sim's parity block, so the two match by construction rather
than by coincidence. Falsifiable prediction if the
latter: output was predicted at ≈1.902 PU/s. **Measured 2026-08-06** on
`feat/research-productivity-axis`: 1.850/s, a −6.9% residual, not the ≈−4.3%
predicted. The prediction's *ceiling* was right (1.9018 measured vs 1.902
predicted); it wrongly assumed the meter would sit on it. The shortfall is AC
over-production (2.39 AC per PU craft against the recipe's 2) consuming EC —
i.e. the solver's own blind spot to research productivity, not a meter defect.
Full numbers in [`meter-divergence.md`](meter-divergence.md).

**Process note worth keeping.** Four causes were proposed and retired before
this one — belt-cycle update order (≈14% of the gap), head-hog distribution
(≈5%), upstream EC/plate production (falsified by the sim's own copper-cable
balance), and the sim-reporting-artifact reading (falsified by the probe). What
finally worked was balancing the sim's reported figures against each other —
the AC:PU ratio predicted +10.7% against a measured +10.0% — and then running
the measurement. Four narrative root causes cost more than one probe did.


### 8. Declared research productivity: two open seams (from #591 review)
Both found by adversarial review of the wiring PR; neither reachable today,
both the same class as bugs this campaign already paid for.

- **The axis stops at the cell-composition pipeline.** `CellComposedCandidate`
  is default-eligible and re-derives cells through `extract.rs`'s plain solve,
  which cannot see the axis; `chain.rs` declares empty on the composed result.
  So if a declared export ever ships a cell-composed rescue — and the e2e pins
  that this happens under *default* options for chain shapes
  (`chain_am2_default_options_ships_cell_composed_rescue`) — the manifest
  silently loses the caller's declaration while `planned_rates` stay boosted.
  The sim's parity check then fail-closes the run, correctly but as an
  unexplained mystery. Same class #415 solved for inserter capacity.
- **`SolverResult` does not record the axis it solved at.** The manifest
  records what the LAYOUT was told, which equals the solve only because
  `sim_export` passes one parsed value to both knobs. That is caller
  discipline, not a guarantee, and nothing detects plan-vs-manifest
  disagreement. Durable fix: have `SolverResult` carry the axis and have the
  layout and manifest copy it from there.

### 9. Deep-chain throughput deficits, measured 2026-08-06 — **DO NOT TOUCH UNTIL EYEBALLED IN THE SIM**

> **Owner instruction, 2026-08-06, and it is the first thing to do here:**
> **nobody starts fixing these until the user has loaded the fixture in a real
> Factorio client and looked at it.** Use
> `cargo run --release -p spaghettio_sim_harness -- serve --bp <bp.txt> --manifest <manifest-real.json>`
> (`docs/sim-harness.md`; note the version-match and WSL/UDP gotchas) and join
> it. The rationale is time, not ceremony: this class has already burned
> multiple sessions on hypotheses that a few minutes of watching the factory
> would have killed outright — item 1 above went through "single-exit boundary
> dedup" and "input delivery" before an in-game look found the real choke in
> `merge_output_rows`. Numbers say *how much*; eyes say *where*. Do the cheap
> thing first.

Six sim runs on the RFC-064 productivity stack tip (`fdf1cd35`), Factorio
2.0.77, `--speed 32`, deep warmups, all `converged: true` with flat window-rate
series (not buffer-fill ramps):

| fixture | axis declared | plan/s | measured/s | ratio | verdict |
|---|---|---|---|---|---|
| `processing-unit@1` | no | 1.000 | 0.644 | 64.4% | **NO DATA** (kit_errors) |
| `processing-unit@1` | yes | 1.000 | 0.682 | **68.2%** | FAIL |
| `electronic-circuit@10` | no | 10.000 | 10.000 | 100.0% | PASS |
| `electronic-circuit@10` | yes | 10.000 | 10.000 | 100.0% | PASS |
| `advanced-circuit@5` | no | 5.000 | 4.167 | 83.3% | **NO DATA** (kit_errors) |
| `advanced-circuit@5` | yes | 5.000 | 4.167 | **83.3%** | FAIL |

**The two `NO DATA` rows are the new parity check working, not a defect.** An
undeclared manifest against an install carrying +10% on `processing-unit` /
`plastic-bar` now raises `kit_errors` and forces `NO DATA` — so a run that
would previously have returned a quietly-biased 0.644 now returns no
measurement and says why. Do not difference a rate against a `NO DATA` run.

**What is actually open, and it is not productivity.** In the *declared* PU
run every stage of the chain sits at the same ratio — copper-cable 0.6874,
copper-plate 0.6878, electronic-circuit 0.6875, iron-plate 0.6874, plastic-bar
0.6875, PU 0.6818. A modelling error would be *differential* (only bonused
recipes drifting); a **uniform** ~68.5% is the whole factory running at 68.5%
of plan. Machine census agrees: ~1/3 of machines in `full_output`
(blocked-downstream) in both PU runs — 49/145 declared, 61/157 undeclared.
`advanced-circuit` shows the same shape at 83.3%, **bit-identical whether or
not the axis is declared**, even though declaring it measurably shrinks the
plan (crude-oil 222.2→202.0/s, coal 5.0→4.55/s) — so its ceiling is set by
something the machine-count change does not touch. `electronic-circuit@10` at
exactly 100% says the deficit is depth-specific, not general.

Probably the same family as item 1 and the `#519` input-rate-delivery class,
but **not confirmed as such** — item 1's confirmed choke is `merge_output_rows`
capping output at one belt, and a 1/s PU target is nowhere near a belt's
throughput, so that specific mechanism does not obviously apply here.

**Two caveats that must survive to whoever picks this up:**
1. Every run converged at the **minimum** 4 checkpoints (3 closed windows).
   `sim-harness-forensics.md` class 5c warns that a converged-at-minimum
   reading needs a second, longer-warmup confirmation before it is trusted as
   the asymptote. Not re-run. Treat these ratios as provisional.
2. **Unreconciled discrepancy with `#591`'s own PR note** (`c2c4ecae`), which
   reported "VERIFIED END TO END — OVERALL WARN (90–98% band)" for the
   declared-flag PU scenario. This sweep measures 68.2%, a clean FAIL well
   outside that band. The two are not reconcilable without knowing the rate and
   warmup behind the PR's run. Resolve this **before** trusting either number —
   one of them is measuring something other than what it claims.

Repro (from a worktree at the stack tip):
```
cargo run --release --example sim_export -- processing-unit 1 \
  --research-productivity processing-unit=0.10,plastic-bar=0.10 \
  --label pu1-flag --out <dir>
cargo run --release -p spaghettio_sim_harness -- run \
  --bp <dir>/pu1-flag/bp.txt --manifest <dir>/pu1-flag/manifest-real.json \
  --warmup 432000 --speed 32
```

## Decided / closed (from Stage B)
- **Never-worse holds** on the measurable subset → evidence supports
  `compact_layout` default-on (representative-scope).
- Sim **speed 32** (validated speed-invariant) + **deep warmup 108000** (30
  game-min) adopted for the bank.
- `bacteria_self_loop_regression` added to the sim skip-list (dead class).
