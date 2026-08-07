# Handoff — meter fluid support (RFC-054 / #570), Phase A landed

**Take me to the work:** branch `feature/meter-fluid-phase-a`, PR
https://github.com/storkme/spaghettio/pull/571 (open, ready). Base is `main`
at `5ca1b192` (which carries the merged #567 output-merge fix).
Older session threads already merged: the `--timeseries` telemetry, RFC-064
Phase 2 sim results, and the #567 N-to-M output fix.

Most of this session was **RFC-064 Phase 2 sim verification + the #567 output
fix (both merged)**. This handoff is about the NEW thread: teaching the fast
meter (RFC-054) to model fluids so recipe chains that need one stop returning 0
(#570).

## What the meter thread is

The fast meter (`crates/meter/`, a workspace member, native item-level sim —
**no `[[bin]]`, library + examples only**) returned a hard **0** on any chain
needing a fluid (AC, PU, oil, plastic, uranium), because `machine.rs` took
*solids only* ("fluids are PR-3 out of scope"), `network.rs` had no pipe flow,
and `factory.rs:237` skipped fluid boundary inputs with a note.

Phase A (landed, in #571) made these chains produce:
- `Machine` understands fluids: `MachineState::FluidIngredientShortage`,
  `fluid_input`/`fluid_output` buffers, `insert_fluid`/`fluid_room_for`,
  fluids included in `has_ingredients`, fluid ingredients consumed per craft,
  fluid products → `fluid_output`, the PR-3 "refuse to craft" gate removed.
- `Factory` delivers via a new `tick_fluids` step (port-adjacency): a machine's
  missing fluid is pulled from the nearest boundary `fluid_feeds` standing
  source, else an adjacent producer's `fluid_output`; unconsumed `fluid_output`
  is drained as delivered.
- `examples/sweep_corpus.rs` = the meter-vs-sim calibration driver.

## Corrected calibration (read this — earlier numbers were buggy)

After the sweep metric was fixed (see Gotchas), the meter is within ±10pp on
**almost everything**:

| family | meter vs sim |
|---|---|
| gear | exact (10, 21) |
| EC + all stress-EC (22/23/30/60, decomposed) | **±0–2%** (models the bottleneck) |
| AOP / oil-refinery (petroleum-gas) | **18 = 18, exact** |
| sulfur, heavy-oil | covered |

The **lone real residual is AC/PU/AC-partitioned ~−80%** (AC_from_plates 0.2 vs
1.0, PU 0.4 vs 2.0, AC_from_ore 1.0 vs 5.0, all AC variants).

**Root cause of the residual (this is Phase B):** `tick_fluids` delivers fluid
ONE UNIT PER TICK, throttling the petroleum→plastic→AC→PU chain to ~20%. Fluid
should flow pipe-fast (and multi-output refinery byproducts need balancing).

Corpus: `~/spaghettio-corpora/job2-sim-baselines/2026-08-01/` (fixtures have
`<fixture>/<variant>/{bp.txt,manifest-real.json}`, sim results in
`sim/<fixture>__<variant>/report.json`).

## Next steps (Phase B → C), in order

1. **Phase B — make fluid delivery pipe-fast/balanced.** `tick_fluids` should
   deliver the full throughput a consumer can accept per tick-cycle (not one
   unit), so petroleum→plastic→AC→PU reaches plan; handle multi-output
   refinery byproducts (heavy/light/petroleum) so nothing starves.
2. **Phase B — multi-output byproduct check.** The single-source-per-fluid
   nearest-match ignores real pipe routing; verify byproduct loops don't
   over/under-credit.
3. **Phase C — ±10pp across the whole corpus.** Re-run `sweep_corpus`; target
   AC/PU within ±10pp; log residual divergence.

## Verification commands

```bash
# meter unit tests (includes the fluid craft test)
cargo test --manifest-path crates/meter/Cargo.toml --lib

# calibrate against the sim corpus (drives sweep_corpus)
cargo build --release --manifest-path crates/meter/Cargo.toml --example sweep_corpus
./target/release/examples/sweep_corpus \
  "$HOME/spaghettio-corpora/job2-sim-baselines/2026-08-01" /tmp/meter-vs-sim.csv
# then analyze the CSV (compare meter_vs_sim_pp by metric)

# per-fixture debug (machine state/fluid buffers)
cargo build --release --manifest-path crates/meter/Cargo.toml --example debug_fluid
./target/release/examples/debug_fluid "$HOME/.../<fixture>/native"
```

Clippy clean; 51 unit tests pass. Full corpus sweep ~3–4 min.

## Gotchas / traps (read carefully — each one bit this session)

1. **`sweep_corpus.rs` had THREE metric bugs** (all fixed):
   (a) it picked the alphabetically-first planned item → must use
   `manifest.targets[0]`; (b) for the meter rate it read `produced_per_s` only →
   fluids live in `delivered_per_s`; (c) the metric choice must be based on
   `manifest.targets[0].is_fluid`, NOT on whether the sim report has a produced
   value (the sim reports both for fluid targets). If you re-derive or extend
   this, re-check all three.
2. **`ingest_real_fixtures::topology_builds_cleanly_on_every_fixture` FAILS** on
   `rfc057-topology-free-mil5`: "only 719/1026 tiles link downstream". This is a
   belt-topology check, **NOT** the fluid change — attributed to the **#567
   output-merge** (new multi-belt geometry) now on `main`. Needs its own look;
   don't chase it inside the meter-fluid work.
3. **The earlier `rfc064_db_maps` (atlas/fold) jobs PANICKED and stuck 12h** at
   `chain.rs:1500` ("below-approach (10 vs 9)") producing zero output while
   holding 2 cores — a real cell-composition edge in the driver. If re-running
   those maps, fix the panic + the rayon-collect-after-panic deadlock first.
4. **The `--timeseries`/sim campaign OOM'd the box once** (22 GB fuzz) — never
   run unbounded-parallel heavy solves alongside live sims. Everything is
   currently killed; box is idle (~33 GB free).
5. **Sim tuning adopted:** speed 32 (validated speed-invariant) + deep warmup
   108000 (30 game-min) — a 2.7× cut. Numbers comparable to the 80-min/16 bank.
6. Only the meter `--lib` tests are the fast unit gate; the corpus/ingest tests
   build+replay layouts and are slow — budget for them.

## Other PRs / open threads on the radar

- **PR #569** (branch `eval-primitives`): RFC-064 P1/P2 evaluation primitives —
  objective metrics + never-worse verdict + candidate runner. Unrelated to this
  meter work; just noting it's open.
- `docs/meter-fluid-followups.md` — the scoped plan + corrected status.
- `docs/rfc064-phase2-followups.md` — Phase 2 residuals (stress-EC ceiling now
  fixed by #567; input-delivery and bio/fluid-class items open).

## How to start the next session

1. `git checkout feature/meter-fluid-phase-a` (you'll be on the right branch).
2. Confirm PR #571 state / merge status.
3. Pick up Phase B: make `tick_fluids` deliver pipe-fast, then re-calibrate with
   `sweep_corpus` and confirm AC/PU move off −80%.
