# RFC-064 Phase 2 — follow-ups

**Status 2026-08-01:** Phase 2 Stage B subset sim campaign adjudicated —
never-worse HOLDS on the measurable subset (gate GREEN, representative scope);
tuning adopted (speed 32, deep warmup 30 game-min). Open below: the surfaced
findings that need engineering or a decision. The two layout/rig defects are
tracked together in **#567**; the dropped duplicate stress fixtures are being
re-run at the tuned settings (this file's item 6, in progress).

## Open items (pick-up notes)

### 1. stress-EC high-rate throughput ceiling — ROOT CAUSE: single external-output belt (#567)
The full stress-family sweep (native ≈ compact throughout — never a compaction
regression) caps high-rate EC-from-ore at **exactly one belt's throughput**:
`22s` 15.00/15.00, `23s` 15.10/15.10, `30s` 15.15/15.15, `60s-red` 30.5/30.0, and
`35s`/`40s` do not converge (census 140–200 `full_output` ↔ 27–59
`item_ingredient_shortage` surge/jam). **Root cause:** the target's external
output is routed through a **single boundary belt** (`boundary_outputs` has
exactly one entry per fixture, from `bus/layout.rs:1444` collecting terminal
belts of `output:`/`merger:` segments), tiered to the fixture's forced belt
(yellow fixtures exit on `transport-belt` → 15/s; red 60s on
`fast-transport-belt` → 30/s). The internal bus is multi-lane (17 south lanes
observed) but all terminates into one exit belt → machines back up `full_output`,
capping the target at one belt. Both native and compact share the single-exit
design, so they measure identically. **Fix:** a target over one belt (at the
chosen tier) must exit through N boundary belts (ceil(rate/cap)) — split the
`output:` segment across multiple terminal belts. NOTE: `tier2_electronic_circuit`'s
5.77/s is a DIFFERENT cause (below the 15/s exit cap; the #519
input-rate-delivery/inserter class), not this fix.

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

## Decided / closed (from Stage B)
- **Never-worse holds** on the measurable subset → evidence supports
  `compact_layout` default-on (representative-scope).
- Sim **speed 32** (validated speed-invariant) + **deep warmup 108000** (30
  game-min) adopted for the bank.
- `bacteria_self_loop_regression` added to the sim skip-list (dead class).
