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
  warmup + 8 worst-case measurement windows, rounded up to the 60-tick
  cadence. (Worst case: at plan a window closes at its nominal length,
  a quarter of the cap, so the budget is rarely spent — chem5 and usp2
  converge on ~30–37% of it.) An explicit value is **floored at
  viability**: a ceiling that cannot fit four checkpoints makes
  convergence structurally impossible, so `converged: false` would
  describe the budget rather than the factory (#454).
- `--timeout-secs N` (derived, ≥900) — wall-clock bound on the whole
  launch. This is a net for a hung or crashed server, **not** a second
  tick budget: the scenario force-finalizes itself at the ceiling, and a
  timeout firing first kills the server before anything is written,
  turning a useful non-converged report into no report at all. The
  default therefore scales with the tick budget (4× the time the budget
  would take at the requested `--speed`, plus setup) — Factorio's tick
  loop is effectively single-threaded, so a large factory or a busy
  machine simply runs slower than asked.
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

  > **The default warmup is too short for deep chains, and this has
  > produced wrong numbers that were recorded as layout defects.**
  > Measured 2026-07-26: `chain-mil5ore-d2` is recorded in RFC-054's
  > calibration corpus as a **FAIL at −28.7%**. Re-run unchanged at
  > `--warmup 288000` (80 game-minutes) it measures **+0.7%, 146/146
  > machines working, PASS**. Nothing about the layout changed; the
  > original measurement simply started before the factory finished
  > filling. `chain-mil5plates-d0` shows the same shape — the native
  > meter reads −38.4% at a 2-minute warmup and +0.7% once converged.
  >
  > Practical rule: **for any chain with more than a couple of stages,
  > treat a deficit measured at the default warmup as unproven until
  > re-run with a long one.** Sweep warmup and watch the number move; a
  > real deficit is flat against warmup, a transient is not. Deep chains
  > have needed 40–80 game-minutes, far above the dim-scaled default.
  >
  > This is not a hypothetical concern about precision: it is the reason
  > RFC-054's KC1 appeared to fail, and it puts a question mark over
  > every recorded deficit taken at the default — see
  > [#453](https://github.com/storkme/spaghettio/issues/453) and
  > [#437](https://github.com/storkme/spaghettio/issues/437).

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

- **fewer than 4 checkpoints** (`STABILITY_WINDOWS + 1` — three closed
  windows plus the one that opens them) — the convergence test never ran;
  `converged` describes the tick budget, not the factory.
- **`short_sampled`** — the window hit the tick cap without filling,
  so the rate is quantization-noisy.
- **NOT CONVERGED** — the rate is a point on a transient. The printed
  window-rate series shows whether it was ramping or decaying; a
  monotone series is not noise, and a single number off it should not
  be compared against another run's.
## Serving a fixture live (`serve`) — looking at it with your eyes

`run` races at `game.speed = 16` and tears the world down the moment it
has its number, so it cannot answer "what does this actually look like".
`serve` hosts the same scenario as a **joinable multiplayer server** at
real time, with no tick ceiling, and does not exit until you stop it.

```bash
cargo run --release -p spaghettio_sim_harness -- serve \
    --bp  crates/core/target/tmp/chain-mil5plates-d0.bp \
    --manifest crates/core/target/tmp/chain-mil5plates-d0.manifest.json
```

Then in a Factorio client: **Multiplayer → Connect to address**.

Knobs: `--port N` (**34197**, Factorio's default — fixed, not ephemeral,
because a human has to type it), `--speed N` (**1**), `--warmup N`.

### Connecting: the three things that will bite you

1. **The client version must match the server install exactly.** The
   install is pinned (`paths::PINNED_VERSION`); check with
   `~/.cache/spaghettio-sim/factorio-<ver>/bin/x64/factorio --version`.
   Point at a different install with `SPAGHETTIO_FACTORIO_DIR` if your
   client is on an older build. A mismatch simply refuses to connect.
2. **On WSL2, connect to the VM's IP, not `localhost`.** WSL2 forwards
   *TCP* to localhost but not *UDP*, and Factorio multiplayer is UDP.
   Get it with `ip -4 addr show eth0` — e.g. `172.31.66.164:34197`. The
   address changes when WSL restarts, so re-check rather than
   remembering it.
3. **Nothing is discoverable.** The server is not advertised on LAN or
   public (the harness settings set both false), so it will never show
   up in the server browser — always use *Connect to address*.

### What you get on join

`serve` sets these on player join; measurement runs never do, because
they change force bonuses and a measurement must run in the world its
fixture declares:

- The whole paste **charted** with a 64-tile margin, so the map opens
  usable instead of black, plus a printed line giving the layout's world
  bounds.
- **6× running speed** and **+24 reach** — a bus layout is a couple of
  hundred tiles wide and walking it is the bottleneck.
- **Commands enabled for everyone** (`allow_commands: true`), written to
  a per-run settings copy rather than mutating the shared measurement
  settings. So no admin list is needed for:
  - `/editor` — free camera, no character. The best way to inspect a
    large layout; sweep it without walking.
  - `/c game.speed = 4` — fast-forward to steady state, then back to 1.
    Useful because deep chains take **tens of game-minutes** to converge
    (see the warmup warning above) — at 1× you are watching the
    buffer-fill transient, not the steady state.

The scratch run dir is kept rather than cleaned up, so the scenario and
its `script-output/` survive for inspection afterwards.

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
