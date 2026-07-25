# Sim harness (`spaghettio-sim`)

Runs an exported layout in a **real headless Factorio server** and reports
planned vs measured per-item rates. This is the ground-truth check the
validator can't give you: the prototype run found an inserter-direction
export bug that deadlocked every factory the project had ever exported,
invisible to all 36 validation checks. Design rationale, kill criteria,
and the decision log live in
[`rfc-050-headless-sim-harness.md`](rfc-050-headless-sim-harness.md); this
doc is the how-to.

It is an **offline engineer tool** — not a validator stage, not CI. Runs
are opt-in, like STRESSGOLD.

## One-time setup

```bash
cargo run -p spaghettio_sim_harness -- fetch
```

Downloads the **pinned Factorio 2.0.77** headless build (via system
`curl`/`tar` — deliberately no HTTP/archive crates) into
`~/.cache/spaghettio-sim/factorio-2.0.77`, and writes the harness server
settings (`auto_pause: false`, autosaves off — a paused or saving server
breaks the measurement loop) plus the Space Age mod-list. Override the
install location with `SPAGHETTIO_FACTORIO_DIR`. The pin is load-bearing
(`latest` has already drifted to 2.1.12, which adds a `recycler`
prototype module our `recipes.json` baseline doesn't have); never point
fetch at `latest`.

`cargo run -p spaghettio_sim_harness -- check-data` spot-checks the
pinned install's dumped prototype data against `recipes.json` (RFC-050
KC1) — run it after a pin bump, not routinely.

## Getting a blueprint + manifest pair

`run` consumes two artifacts produced together by
`blueprint::export_with_manifest(layout, solver_result, label)`
(`crates/core/src/blueprint.rs`): the blueprint string and a JSON
manifest recording the feed/drain boundary positions, bbox, dims, and
planned per-item rates. The harness deliberately does **not** depend on
`spaghettio_core` — it consumes the manifest JSON schema only.

**Known gap:** the only existing generator is
`crates/core/examples/sim_probe_export.rs`, and `crates/core/examples/`
is gitignored (local-only debug scripts) — on a fresh clone there is no
tracked way to produce the pair. Usage of the local example, where
present:

```bash
# writes $SIM_PROBE_OUT/bp.txt + manifest-real.json (default /tmp)
cargo run --example sim_probe_export <item> <rate> <stacking> <inserter_cap> [quality] [belt]
```

Pass **`manifest-real.json`** to `run` — it's the `export_with_manifest`
output the harness parses. The example also writes a sibling
`manifest.json` in a stale pre-Phase-0 ad hoc shape (no `label` field);
the harness will reject it with a missing-field error.

In-process generation from a fixture name is deferred Phase 1 wiring
(see the dependency note in `crates/sim-harness/Cargo.toml`); until then,
either use the local example or write the two files from any code path
that calls `export_with_manifest`.

## Running a measurement

```bash
cargo run -p spaghettio_sim_harness -- run \
    --bp bp.txt --manifest manifest.json --out report.json
```

What happens: the harness generates a scenario (`control.lua` that pastes
the blueprint, superforce-builds it, revives ghosts, attaches feed/drain
boundary infrastructure at the manifest's coordinates), launches the
server on an ephemeral port, and polls `script-output/` for the result;
the server is killed as soon as the result lands (or on timeout). The
Factorio log is kept in the OS temp dir (`$TMPDIR`, usually `/tmp`) as
`spaghettio-sim-<scenario>.log`, and `run` prints the exact path at the
end — first stop for any failure.

Knobs (defaults in parentheses):

- `--speed N` (16) — `game.speed`; wall-clock scales inversely until the
  machine can't keep up.
- `--ticks N` (derived) — hard ceiling tick, the **one** thing that
  force-finalizes a run that never stabilizes. Default is derived from
  warmup + 4 worst-case measurement windows, rounded up to the 60-tick
  cadence. An explicit value is **floored at viability**: a ceiling that
  cannot fit three checkpoints makes convergence structurally impossible,
  so `converged: false` would describe the budget rather than the factory
  (#454).
- `--timeout-secs N` (900) — wall-clock bound on the whole launch.
- `--out FILE` — write the full JSON artifact: `{report, raw_result,
  sim_state, run_params, game_version}`. This file is what `bless`,
  `check`, and the web overlay consume — always pass it for anything you
  might want to keep.
- `--warmup N` (derived from manifest dims) — override the warmup before
  measurement starts. Use for **steady-state probes** on deep chains:
  the 2% stability windows cannot distinguish a slow buffer-fill drift
  from real convergence, so a run can "converge" while trunk buffers are
  still filling (intermediates at or above plan are the tell). One game
  hour (`--warmup 216000`) settled the #357 fixtures.

Reading and debugging the resulting numbers — what each rate actually
measures, the known measurement-artifact classes, and the forensic
playbook (per-lane belt dumps, machine inventories, kit chest census) —
is covered in [`sim-harness-forensics.md`](sim-harness-forensics.md).

### How a measurement window is chosen

Rates are the **trailing window** between the last two checkpoints, so
what that window contains decides what the number is worth. Windows
close on **accumulated items** (300, the sample size the 2% stability
tolerance is built around), bounded by a tick cap at 4× the nominal
at-plan length. A factory below plan therefore gets a *longer* window
rather than a thinner sample.

Windows used to be sized from the *planned* rate and closed on a fixed
tick count, which is the same thing only when the factory runs at plan;
below it, sample size fell in proportion and the run failed closed to
NO DATA. The rule of thumb that fell out: **the worse a factory
performed, the less measurable it became** (#454).

### What "converged" means

Convergence requires the **trailing three window rates to agree as a
group** (widest vs narrowest within 2%), not just the last two.

Comparing the last two only asks "was the last step small", and *any
decelerating ramp eventually passes that* — at a point systematically
short of where it is heading. chem5, a registered PASS, was certified on
4.62 → 4.92 → 5.00/s: monotone, still climbing, final step +1.6%. The
trailing window got published as "5.00/s EXACT at plan" while the whole
measured span averaged 4.84/s. Across a group a ramp keeps accumulating
(+8.3% there) while genuine noise cancels.

This is also the answer to #454's second question — `converged: true` at
160k ticks and `false` at 480k on identical geometry was one long ramp
sampled at two points, not an unstable factory.

Every report prints a `measurement:` line — window length, achieved
items against the 300 floor, checkpoint count, and the drift across the
stability group — plus an explicit warning for each way the number
can mislead:

- **fewer than 3 checkpoints** — the convergence test never ran;
  `converged` describes the tick budget, not the factory.
- **`short_sampled`** — the window hit the tick cap without filling,
  so the rate is quantization-noisy.
- **NOT CONVERGED** — the rate is a point on a transient. The printed
  window-rate series shows whether it was ramping or decaying; a
  monotone series is not noise, and a single number off it should not
  be compared against another run's.

## Reading the report

Per item: planned rate, measured produced rate, measured delivered rate
(drain count), deltas. The target item gets a verdict per RFC-050 KC2,
**one-sided** because overshoot is expected (machine counts are ceilings):

- **PASS** — measured ≥ 98% of planned (overshoot is still PASS)
- **WARN** — ≥ 90%
- **FAIL** — below 90%
- **NO DATA** — no measurement reached a checkpoint; ranks between WARN
  and FAIL for the overall (worst-of-targets) verdict

Overall verdict is the worst target verdict. Non-target intermediates are
informational — a two-sided tolerance would spuriously fail honest
layouts. The run also dumps `sim-state.json` (belt contents, machine
status), included in `--out`.

**Web overlay (RFC-050 Phase 4):** load the `--out` file via the sim
report panel in the web app to get the verdict banner plus a `sim-state`
entity overlay tinting machines/belts/inserters by their simulated state
— the fastest way to see *where* a FAIL is starving.

## Baselines (`bless` / `check`)

```bash
cargo run -p spaghettio_sim_harness -- bless --report report.json \
    --baselines crates/sim-harness/baselines [--label gear10]
cargo run -p spaghettio_sim_harness -- check --report fresh.json \
    --baselines crates/sim-harness/baselines [--tolerance 0.02]
```

`bless` freezes a measured baseline keyed on label; `check` fails on
drift beyond tolerance ("re-bless deliberately if intended"). FAIL
reports are deliberately blessable — freezing today's honest floor means
fixes must move the number and regressions can't hide. See
[`crates/sim-harness/baselines/README.md`](../crates/sim-harness/baselines/README.md)
for the blessed set and its (game pin, mod set) key.

## Concurrency: runs are independent

Concurrent `run` invocations against the same install **just work**:
every run gets its own scratch write directory under the OS temp dir
(`spaghettio-sim-runs/<scenario>-<pid>/`), wired via a generated
per-run `config.ini` — `read-data` points at the shared install's
`data/` (never written), `write-data` at the scratch dir. Factorio's
exclusive write-dir lock, the scenario dir, and `script-output/` result
files are all per-run; the scratch dir is deleted on success and kept
(path printed) on failure for forensics. Validated with two
simultaneous same-second runs against one install: both passed, with
byte-identical (deterministic) reports and nothing written into the
install.

Two residual exclusivities, both rare: `fetch` populates the install
itself (don't fetch while runs are live against it), and `check-data`
still dumps into the install's write dir (one at a time; it's a
post-pin-bump check, not a routine step). The old workaround — `cp -r`
the install and point `SPAGHETTIO_FACTORIO_DIR` at the clone — is no
longer needed for concurrency, but remains the way to test a different
install (e.g. a candidate pin bump) side by side.

## Troubleshooting

- **`factorio exited early`** — a real crash (bad blueprint string, Lua
  error at startup): read the log at the path `run` printed; the kept
  run dir has the generated `config.ini` and scenario for repro.
- **`KIT ERRORS` in the report** — the boundary kit's self-audit failed
  (e.g. overlapping bank chests); the run is invalid and the verdict is
  forced NO DATA. Never interpret rates from such a run — see the kit
  -contamination artifact class in
  [`sim-harness-forensics.md`](sim-harness-forensics.md).
- **Timeout waiting for `harness-result.json`** — the scenario never
  finished; check the log for Lua errors, or raise `--timeout-secs` on
  slow machines.
- **Determinism** — Factorio's lockstep sim is deterministic: two
  identical runs produce identical reports. A report that changes across
  runs of the same artifacts means the artifacts (or pin) changed.
- Scenario dirs accumulate under the install's `scenarios/`
  (timestamp-suffixed); they are small and safe to delete.
