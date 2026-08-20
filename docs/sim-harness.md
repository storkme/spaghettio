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

`crates/core/examples/sim_export.rs` is the tracked generator. The
`examples/` directory is otherwise gitignored (local-only debug scripts);
`.gitignore` carries an explicit negation for this one file so a fresh
clone can produce the pair.

```bash
cargo run --release --example sim_export -- <item> <rate> [flags]

  --tier <entity>       crafting machine (default assembling-machine-3)
  --di off|candidate|forced       direct insertion (default candidate)
  --claim up|down|search          DI claim order (default: engine default)
  --belt <entity>       max belt tier      --quality <name>   normal..legendary
  --stacking <1..4>     belt stacking      --inserter-cap <n> capacity level
  --inputs a,b,c        raw inputs (default: the six-ore set)
  --row-layout <kind>   native (default) | horizontal-stack
  --strategy <kind>     pooled (default) | partitioned-decomposed
  --duty <0..1>         planning duty (default 1.0; <1 needs --belt)
  --research-productivity <recipe=bonus,...>   declared research
  --label <name>        output subdir + manifest label
  --out <dir>           parent dir (default $SIM_PROBE_OUT, else /tmp)
```

Passing any flag that changes the exported layout — `--strategy`,
`--row-layout`, `--duty`, `--belt`, `--tier`, `--quality`, `--stacking`,
`--di`, `--claim`, `--inserter-cap`, `--inputs`,
`--research-productivity` — **requires an explicit `--label`**, and the tool
refuses without one.

The label is also the output directory, so two runs differing by any of
those must not share it. They used to: the label was `{item}-{rate}` and
encoded no configuration, so a second run silently overwrote the first's
`bp.txt` and `manifest-real.json` — a wrong-A/B generator, in the tool
whose whole purpose is A/B. Encoding each axis into the name was tried and
abandoned: it needs every axis enumerated and every engine default stated
correctly, and five review rounds each found another one wrong. Requiring
the label makes collisions between runs that differ only by an axis
impossible by construction, and consults no defaults, so a future default
flip cannot invert it. Reusing the same `--label` twice still collides —
that is a deliberate escape hatch, not an oversight.

A run with no axis flags keeps the old `{item}-{rate}` path unchanged.

The check is **syntactic**: it fires on the PRESENCE of an axis flag, not on
whether the value differs from the engine default. So `--tier
assembling-machine-3` or `--strategy pooled` requires a `--label` even
though neither changes the artifact. This is deliberate — deciding
"is this value the default?" is exactly the defaults-tracking the encoding
scheme was abandoned for, and it is the half that kept being wrong. The cost
is that a previously-valid invocation passing an axis at its default now
needs a label; no in-tree caller does, and the failure is a loud refusal
with the required flag named, not a silent overwrite.

It writes `<out>/<label>/bp.txt` and `<out>/<label>/manifest-real.json`,
and prints the ready-to-paste `run` command. Unknown flags are an error
rather than ignored — a silently-dropped `--belt` would export a layout
the caller did not ask for and then be simmed as though it had.

There is also an older, **untracked** `sim_probe_export.rs` in the same
directory. Prefer `sim_export`: it covers the same axes plus the DI ones,
and it writes only `manifest-real.json`. `sim_probe_export` additionally
writes a sibling `manifest.json` in a stale pre-Phase-0 ad hoc shape (no
`label` field) that the harness rejects with a missing-field error — so
with that one you have to know which of the two files to pass.

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
  measurement starts. **Raise `--timeout-secs` whenever you raise this.**
  The derived timeout is 4× the tick budget at the *requested* speed, but a
  loaded box runs slower than asked, so a long warmup can be killed
  mid-warmup — and the failure is quiet in the worst way: no verdict, no
  `kit_errors`, an empty `timeseries.csv`, and (if you were watching from
  outside) no Factorio process, which reads exactly like a completed run.
  Measured 2026-08-07: `stress_electronic_circuit_30s_from_ore` at
  `--warmup 432000 --speed 32` died at the derived 1095s having written
  zero data rows. `--timeout-secs 3600` cleared it. Check for the
  `timed out after Ns waiting for harness-result.json` line before
  concluding anything about a run that produced no numbers. Use for **steady-state probes** on deep chains:
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

### The world stays alive after it converges (fixed 2026-08-07)

`serve` keeps the boundary kit — feed top-up, drain emptying, and
electric-interface recharge — running after the scenario finalizes
(`RunParams::keep_alive`). **Before this fix it did not**, and the
consequence was severe enough to be worth knowing when reading older
notes: the kit's upkeep is one `on_nth_tick(60)` handler gated on
`storage.finalized`, and `finalize` fires on **convergence** as well as at
the tick ceiling. `serve` pushed the ceiling out to ~a week of game time
to stop the world dying mid-inspection, which covered only one of the two
callers — so a served world self-finalized as soon as its rates
stabilised, typically minutes in, and then had no input, no drain and no
power. Anyone who joined after that was looking at a **stopped factory**
while believing they were looking at the layout. It was found exactly that
way: "the input chests are empty and I can't see much happening".

Two consequences for reading evidence:

- **Flow observations made in a served world before this fix are
  unreliable** if they were made more than a few minutes in — belt
  saturation, which machines are running, where items are backing up.
  **Structural** observations (machine counts, topology, what feeds what)
  are unaffected.
- The time-series CSV still stops at finalize; only the factory keeps
  running. See ["Reading the time-series"](#reading-the-time-series).

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

### The `validator:` line — read this before the rates

Every report header now carries the validator state of the exact layout
that was measured (`validator-trust.md` hole 3, closed 2026-08-09):

```
validator: 3W — input-rate-delivery×3
```

and, when the layout was flagged, a banner immediately above the rate table:

```
!! MEASURED ON A LAYOUT CARRYING VALIDATOR WARNINGS — THIS MEASURES THE
   LAYOUT, NOT THE PIPELINE
```

That distinction is the whole point. **A number measured on a layout the
validator has already condemned is a fact about that layout, not evidence
about the pipeline that produced it.** On 2026-08-07 a PU@1/s fixture was
reported at 68.2% of plan while carrying three `input-rate-delivery`
warnings naming the exact starving machines — the validator had localised
the defect before the sim ran, and nothing printed it.

Four states, recorded in the JSON as `validator_standing`:

| State | Meaning |
|---|---|
| `unflagged` | Validator reported nothing |
| `warned` | Warnings present — the rate describes this layout |
| `condemned` | Errors present — arguably should not have been simmed |
| `unknown` | Manifest predates the field — **not** the same as clean |

Two things it does **not** mean:

- **`unflagged` is not clearance.** Of ~40 checks only a handful carry
  refusal power, and each is documented "never sim-anchored" in
  `validator-trust.md`. A clean line beside an at-plan rate means the two
  instruments agree, not that either is right.
- **It is not a gate.** Nothing refuses to run a condemned layout. If you
  want a parity number from a flagged fixture you can still have one — you
  just can't get it without being told what you are measuring.

Counts are per-category rather than totals because a total cannot tell 2
from 218 (`validator-reporting.md`). `3W` with `input-rate-delivery×3`
tells you which check fired and how often; `warnings: 3` would not.

**Web overlay (RFC-050 Phase 4):** load the `--out` file via the sim
report panel in the web app to get the verdict banner plus a `sim-state`
entity overlay tinting machines/belts/inserters by their simulated state
— the fastest way to see *where* a FAIL is starving.

## Reading the time-series

`sim-state.json` and the machine census are a single frame — a snapshot
at finalize. That answers "where is it stuck right now" but not "was it
ever moving, and when did it stop" — the question #537 needed answered:
a `land-mine@1` fixture measured 0/s, and the only evidence available was
a final census (`fluid_ingredient_shortage: 2, item_ingredient_shortage:
2, full_output: 4`) that reads equally well as "never started" or "ran
fine for an hour then jammed". A rate-vs-time series distinguishes those
at a glance; the final aggregate cannot.

Every report now carries `timeseries`, one entry per checkpoint window
(the same item-driven windows the target/intermediate rates are computed
over — see "How a measurement window is chosen" above):

```jsonc
{
  "tick": 10800,
  "machines": [
    {"unit": 142, "name": "assembling-machine-2", "x": 3, "y": -1,
     "crafts_delta": 15.0, "status": "working"},
    {"unit": 143, "name": "electric-furnace", "x": 5, "y": 2,
     "crafts_delta": 0.0, "status": "item_ingredient_shortage"}
  ],
  "items": {"iron-gear-wheel": 30.0, "iron-plate": 0.0}
}
```

- **`machines`** — every crafting machine (assembler/furnace; chemical
  plants and refineries are `assembling-machine` prototypes, so they're
  covered by the same filter used elsewhere in the harness), identified
  by `unit` (Factorio's `unit_number` — stable across samples even where
  name+position would collide, e.g. after a rebuild). `crafts_delta` is
  the change in `products_finished` since the *previous* checkpoint (a
  per-window count, not the running total); `status` is
  `defines.entity_status` mapped to its short symbolic name (`working`,
  `no_power`, `item_ingredient_shortage`, `full_output`, …) — the same
  vocabulary the machine census already uses.
- **`items`** — per planned item (from the manifest, not just the
  target), the force production-statistics counter's delta over that
  same window — the per-window value, not the cumulative aggregate
  `raw_result.samples` already carries.

`run --out report.json` puts this under `report.timeseries` (parsed,
typed) and `raw_result.timeseries` (the raw Lua-emitted array); it's
purely additive — a report captured before this field existed, or a
`bless`/`check` baseline, parses it as an empty series rather than
erroring, and `bless`/`check` never diff it (they only read specific
named fields).

`serve` writes the same per-window sample as CSV rows appended to
`script-output/timeseries.csv` inside the run's scratch dir (path printed
at startup) — long-format, one row per machine-or-item per window:

```
tick,kind,unit,name,x,y,crafts_delta,status,item,produced_delta
10800,machine,142,assembling-machine-2,3,-1,15,working,,
10800,item,,,,,,,iron-gear-wheel,30
```

`kind` distinguishes the two row shapes; filter on it before further
processing (e.g. `awk -F, '$2=="machine"'`). This is the machine-readable
record a maintainer eyeballing a live `serve` session at speed 10
otherwise has none of.

**The CSV stops at finalize; the world does not** (corrected 2026-08-07 —
this section previously claimed the CSV "keeps growing as long as the
server runs, independent of whether/when the scenario ever finalizes",
which was never true of the convergence path). `finalize` fires on
CONVERGENCE as well as at the tick ceiling, and it ends measurement — so
the series freezes once rates stabilise, typically minutes in. What
`serve` now guarantees is that the **factory keeps running** past that
point (`RunParams::keep_alive`), so the world stays inspectable even
though its time-series has stopped growing.

For a longer series the lever is **`--warmup`** (measurement opens later,
so windows keep closing for longer before the stability test can trip) —
in either mode. Switching to `run --timeseries` does NOT buy length by
itself: serve already runs under a ~36M-tick ceiling while `run`'s
default ceiling is far smaller, so a bare `run` gives you *less* unless
you also raise `--ticks`.

Post-finalize staleness applies to every artifact, not just the CSV:
`sim-state.json` and `harness-result.json` are both written *by*
`finalize` and then frozen, while under `keep_alive` the served world
carries on diverging from them. Treat all three as a snapshot of the
moment the run converged, never as live state.

For the diagnostic reading of a flat-zero vs ramp-then-decay vs
stable-below-plan series, see
["Reading time-series decay shapes"](sim-harness-forensics.md#reading-time-series-decay-shapes)
in the forensics doc.

## Shipping a run to Grafana (`scripts/sim-to-graphite.py`)

`raw_result.samples` carries **every planned item's** cumulative production,
sampled every 1200 ticks by the scenario
(`scenario.rs`, `storage.samples`). That is Factorio's own
`get_item_production_statistics` — the same source graftorio reads — so no
mod, no version bump, and nothing added to the measured environment.

```bash
scripts/sim-to-graphite.py <report.json> --arm lift [--dry-run]
```

Pushes to Grafana Cloud Graphite; the token (needs `metrics:write`) comes
from `$GRAFANA_GRAPHITE_TOKEN` or `~/.config/spaghettio/grafana-token`.
Dashboard: `/d/spaghettio-sim`. It works on any report, so the existing
`job2-sim-baselines` corpus can be backfilled without re-running anything.

### Watching a run live (`scripts/sim-live.sh`)

```bash
scripts/sim-live.sh <label> <bp.txt> <manifest-real.json> -- --warmup 432000 --speed 32
```

Runs the fixture with `--timeseries`, locates the scratch dir (its suffix is
random, so the CSV path is only knowable after launch), streams each
checkpoint window to Grafana as it lands, and prints a dashboard link
**pre-filtered to that fixture** with a live window and auto-refresh.

The live stream carries more than the batch export: the scenario's CSV has a
per-machine line with `crafts_delta` and `status`, so `spaghettio.sim.machines`
gives a live count of machines by status — idle and starved machines visible
as they happen, not inferred afterwards.

**The warmup is included.** The scenario mirrors its 1200-tick `samples`
into the CSV as `sample` rows from tick 0, so the live view covers the ramp
— which is the half that matters most for shape: a stage's start offset
(belt transit + buffer fill) is only visible there, and so is the answer to
"was the warmup long enough". The checkpoint rows cannot cover it, because
checkpoints exist to test convergence and by design do not open until warmup
ends. Consumers should prefer `sample` rows and ignore the coarser
checkpoint `item` rows when both are present, or the two write conflicting
values for the same metric at nearly the same timestamp.

Live rates still come from the **tick span** of each window; only the
timestamp is wall-clock, so the run reads left-to-right at the speed you are
watching it. When the run finishes the wrapper also pushes the full
`report.json`, so the run's history survives at sample fidelity rather than
just the live windows.

**Known-broken, exported only as `produced` (review, #604):** each sample
stores a *reference* to `storage.drained_total` / `fed_total`, which the kit
upkeep mutates every tick, and the result is serialized once at finalize — so
`drained` and `fed` report the same FINAL value at every sample (verified: a
flat 67008 from tick 0). They are excluded from the export until the scenario
snapshots them at sample time. Only `produced` is a real curve.

**Live resolution is coarse.** Points are snapped to the 20s interval
boundary, but at `--speed 32` a 1200-tick window is ~0.6s of wall clock, so
~30 windows collapse into one bucket and last-write-wins keeps one. The live
view is therefore fine for watching the *level* settle and useless for
resolving per-stage **start offsets** — the full ramp only arrives with the
post-run backfill. Sub-second-cadence live streaming needs distinct
timestamps per row, not this snapping.

**`sim-live.sh` requires `<label>` to equal the manifest's own label**, since
it locates the run's scratch dir by globbing on it. Mismatch means it cannot
find the CSV and exits.

Two things that are load-bearing and not obvious:

- **Rates are computed in the exporter, from the game-tick delta**, and
  pushed as `spaghettio.sim.rate_*` alongside the raw counters. Graphite's
  `perSecond()` returned **all-null** on this data: it infers the step from
  wall-clock spacing, which is meaningless for a batch backfill whose
  x-axis is game time. The exporter knows the true tick delta, so its rate
  is both correct and directly comparable to `planned_rate`.
- **Timestamps are snapped to the 20s interval boundary.** Metrictank
  buckets by the declared interval; unaligned points store fine and read
  back as nulls under any function needing consecutive samples.
- **Grafana Cloud's Graphite ingest silently DROPS points more than about a
  day old** — and still answers `200 {"published": N}`. Backfilling the
  2026-08-01 corpus "succeeded" 54/54 and not one point was queryable. Use
  `--anchor now` for anything historical: it lands the run's last sample at
  the current time, so chronology across runs is lost but the shape and the
  %-of-plan comparison — the things actually being read — are intact. Runs
  stay distinguishable by their `fixture` / `run` tags.

**All three of these failed the same way: HTTP 200 and empty panels.** When
wiring a new series, verify a panel *returns data* before believing it.

The panel that earns its keep is **% of plan per stage** — measured rate
divided by the solver's planned rate, per item. A deep chain that
under-delivers shows *which stage* falls behind and *when*, rather than
just that the target came up short.

## Live progress telemetry (`run --timeseries` + `scripts/sim-watch.py`)

`run` normally writes its per-window time-series only at finalize, into the
report JSON. For a long or grinding run you don't want to wait until minute
150 to learn it was flat-zero from minute 10. `run --timeseries` streams the
*same* per-window machine/item rows to the scratch dir's
`script-output/timeseries.csv` **live**, the way `serve` already does — the
only difference from a normal `run` is the extra file I/O. It is
measurement-safe: unlike `serve`'s operator QoL it changes no force bonuses
and reveals no map, so a measurement run can stream it without altering the
world the fixture declares. `run` prints the CSV path and the watch command on
launch.

```bash
cargo run --release -p spaghettio_sim_harness -- run \
    --bp bp.txt --manifest manifest.json --timeseries --out report.json

# in another terminal, score it live against the planned rates:
python3 scripts/sim-watch.py <scenario-substring> --plan "item=rate,item=rate" [--follow]
```

`sim-watch.py` renders, from the streaming CSV: each planned item's trailing
per-window rate against its asymptotic ideal (PASS/WARN/FAIL/DEAD), a rollup
of every machine's status at the latest window, and a starvation signal that
only recommends killing when a `fluid_ingredient_shortage` / `no_fuel` /
`no_power` / `item_ingredient_shortage` state *persists* across ≥2 of the
last 3 windows **and** output is flat-zero (a transient shortage while a
factory is still filling its belts — a normal startup state — is reported as
"watching", not "kill"). So a genuinely 0-output fixture is identifiable in
minutes, not hours, without ever false-firing on a healthy ramp. The first
window reads "warming" until two windows close and a run-rate exists. The
scratch dir is removed on a successful finalize; after the run completes, read
the `timeseries` key of the `--out` JSON instead.

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
