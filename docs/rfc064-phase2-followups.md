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

### 7. Meter belt-delivery residual: PU-from-ore −13% (from meter-fluid, #570)
Pick-up entry so the deferral is actually tracked here (the meter
`meter-divergence.md` records the full evidence). The fast meter is −13% on
`tier5_processing_unit_from_ore_am3` vs the sim — a **downstream solid belt
delivery** divergence, not a fluid one: the meter matches the sim within ±4% on
the whole direct chain but loses ~10% delivering electronic-circuit to the PU
machine (the served PU machine, `m#310`, sits short on EC despite adequate EC
production). The fixture's only topology note is a 26-tile belt cycle; the
meter steps cyclic belts in an arbitrary-but-deterministic order, the likely
cause. Deferred deliberately: a fix needs a speculative belt-cycle-update-order
/ merge-priority model change, unverifiable on this noisy fixture (the sim is
~−10% on every intermediate). Note the blast radius is narrow — this fixture has
the corpus's only cycle note, so a `CycleInUpdateOrder`-gated change would touch
~nothing else; the binding reasons to defer are the noisy sim baseline and the
unverifiability, not a broad corpus risk. Investigate
as a belt-model divergence, re-measuring after any belt-network change.

## Decided / closed (from Stage B)
- **Never-worse holds** on the measurable subset → evidence supports
  `compact_layout` default-on (representative-scope).
- Sim **speed 32** (validated speed-invariant) + **deep warmup 108000** (30
  game-min) adopted for the bank.
- `bacteria_self_loop_regression` added to the sim skip-list (dead class).
