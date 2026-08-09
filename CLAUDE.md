# Spaghettio

Automated Factorio factory blueprint generator. Takes a target item + production rate, solves recipe dependencies, generates a spatial bus layout, and exports a Factorio-importable blueprint string.

## Quick start

```bash
# Rust workspace check + unit + e2e tests
cargo test --manifest-path crates/core/Cargo.toml

# A single test with output
cargo test --manifest-path crates/core/Cargo.toml --test e2e -- \
    tier2_electronic_circuit_from_ore --exact --nocapture

# Web app (dev server at http://localhost:5173)
cd web && npm install && npm run dev
```

For full build commands (WASM rebuild, release builds), see [`docs/build-systems.md`](docs/build-systems.md).

## Development conventions

- **Primary workflow**: edit Rust, run `cargo test`, then rebuild WASM and hit the web app to eyeball the layout.
- **WASM rebuild**: `wasm-pack build crates/wasm-bindings --target web --out-dir "$(pwd)/web/src/wasm-pkg"`. Always pass an absolute `--out-dir`.
- **Pre-commit hooks**: in `.githooks/pre-commit`, activate with `git config core.hooksPath .githooks`. Runs `cargo clippy` on staged Rust and `tsc` on staged TS. Bypass with `--no-verify` only for genuine emergencies.
- **Scripts**: put exploratory snippets in `scripts/` rather than inline one-liners. Rust debug scripts go in `crates/core/examples/` or as `#[test] #[ignore]` benchmarks.
- **Snapshots**: `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ...` writes `.fls` files under `crates/core/target/tmp/`. Decode with `tail -c +5 <file> | base64 -d | gunzip`. See [`docs/layout-snapshot-debugger.md`](docs/layout-snapshot-debugger.md).
- **Docs taxonomy** (everything in `docs/` is one of these):
  - **RFCs** (`rfc-*.md`) — design docs circulated for review: design,
    **kill criteria** (required — the dominant rework shape here is
    exploration that overruns its evidence), verification plan, and a
    **decision log**, the canonical record of every call made while the work
    ran. Template: [`docs/rfc-template.md`](docs/rfc-template.md).
    **Numbered**: the registry at [`docs/rfcs.md`](docs/rfcs.md) assigns
    `RFC-NNN` chronologically; existing files keep their names (numbers live
    in the registry), new RFCs are named `rfc-NNN-short-name.md` and get a
    registry row in the same commit. Rejected/obsolete RFCs move to
    `docs/archive/`.
  - **Followups** (`*-followups.md`) — deferred-work backlogs with pick-up
    notes. Named for what they track, not the session that produced them;
    status line at the top so a cold pick-up knows what's open.
  - **Status ledger** ([`docs/status.md`](docs/status.md)) — the
    cross-cutting capability record: recipe complexity ladder, residual
    warnings, RFC close-outs, open tracking issues. Update it when status
    changes; don't record status here in `CLAUDE.md`.
  - **Reference** — evergreen how-things-work docs (`factorio-mechanics.md`,
    `build-systems.md`, `file-reference.md`, `ghost-pipeline-contracts.md`,
    the debugger guides). Kept current when the subject changes; no
    decision-log duty.
  - **Notes** (handoffs, investigations, scratch) — session artifacts with no
    durability contract; archive or delete freely once absorbed.
- **PRs** follow [`.github/pull_request_template.md`](.github/pull_request_template.md), which captures intent, scope, verification actually run, and any deviations from agreed approach. Trivial changes can omit sections explicitly rather than leaving them blank.

### Workflow (branches, review, merging)

- **Branches + PRs by default.** Don't work directly on `main` unless
  explicitly agreed for the task at hand. Work on a feature branch and open a
  PR to get code onto `main`. Besides review, this serializes `main` across
  the multiple concurrent Claude sessions that share this repo.
- **Keep a PR under ~400 added lines.** This is the strongest predictor of
  *rework* we have measured. Across the 197 PRs merged **2026-07-20 → 08-09**
  with >20 added lines, rework per 100 added lines climbed with size under
  both averagings — per-PR mean 1.6 (<100 adds) · 2.5 (100–400) · **4.7
  (400–1k)**, pooled 2.1 · 2.4 · **4.6** (≈3× and ≈2×) — and PRs ≥400 adds,
  36% of that population, accounted for **89% of its rework**. Above 1k the
  two averagings disagree in direction (mean 5.7, pooled 3.6) and the step is
  confounded by right-censoring, so read 400 as "where the climb is measured
  to happen", not as a ceiling. Denominators and caveats:
  [audit](docs/pr-churn-audit-2026-08.md).
  - Read "rework" literally — a later PR rewriting an earlier one's lines. A
    sampled split puts it at ~57% planned iteration / 23% genuine correction /
    20% collision, so this is **not** a defect rate and a big PR is not
    presumed wrong. The norm is about how much work you commit to one shape
    before anything downstream can disagree with it.
  - The mechanism is claim surface area, not carelessness — review on big PRs
    is heavy. A large PR asserts more independently-falsifiable things
    (several sim numbers, several fixtures, several corpus percentages) than
    its verification actually covers, so one of them is wrong and a different
    instrument finds it days later.
  - Above ~400 added lines, **say in the PR body why it could not be split**.
    "It's one RFC phase" is not a reason; phases are splittable into
    scaffolding / behaviour / fixtures / docs.
  - Generated, vendored, and pure-fixture files don't count toward the ceiling.
  - This is a norm, not a gate — nothing blocks on it. It exists so that
    "should I split this?" is a question you answer before opening the PR
    rather than one nobody asks.
- **Adversarial review before anything is commit-ready** — code *and*
  documentation. Preferred: the CI review bot —
  [`.github/workflows/second-opinion.yml`](.github/workflows/second-opinion.yml)
  (the [storkme/second-opinion](https://github.com/storkme/second-opinion)
  action: DeepSeek V4 Flash via OpenRouter, K=3 unioned agentic passes —
  swapped in 2026-08-01 because the previous Claude-backed bot consumed the
  subscription's usage windows) reviews every non-draft PR and posts one
  advisory comment per head SHA (known gap, accepted: a base-branch
  retarget keeps the old SHA's green check — see the workflow's trigger
  comment; re-review those session-side). Its silent-failure mode is loud: a
  degraded pass that posts no review FAILS the `second-opinion` check
  (`fail-on-degraded`), and that check is **required on main**
  (`enforce_admins=true`; escape hatch `scripts/review-gate.sh unrequire`).
  Treat its silence as weak evidence, not clearance — the model tier is
  chosen for cost, not maximum recall. The predecessor,
  claude-code-review.yml, is PARKED (dispatch-only stub, hardened job kept
  intact; ci.yml's workflow-guard still asserts its pieces because the
  `/install-github-app` stock-template overwrite trap applies to parked
  files too). Failure-class history and forensics playbook:
  [`docs/review-bot.md`](docs/review-bot.md).
  **Nothing blocks a merge while the review is still running** except the
  required check itself — waiting is still the merger's job. Use
  `scripts/review-gate.sh wait <pr>`, never a hand-rolled poll: a running
  check reports `IN_PROGRESS` (and the PR `UNSTABLE`), so the obvious
  `!= "PENDING"` test passes instantly and merges mid-review.
  Local adversarial review (an
  independent agent that re-runs gates and probes the claims) is the
  fallback when a PR isn't in play — and it remains **required in addition
  to the bot** for layout-engine or validator-semantics changes: the bot
  reviews the diff, but it cannot run STRESSGOLD, decode snapshots, or do
  tile-level verification. Fork PRs and workflow-file PRs also need
  session-side review — the bot step-gates off forks (no secrets), and a
  PR editing second-opinion.yml runs its own modified workflow **with
  access to `OPENROUTER_API_KEY`** — the concrete stake is secret
  exfiltration (same class as the claude workflows' long-standing
  `CLAUDE_CODE_OAUTH_TOKEN` exposure, not a new hole) — so eyeball such
  diffs before they run, not after.
- **Keep `origin/main` current** — push promptly after merging. Worktree
  agents branch from `origin/main`; a stale origin hands every spawned agent
  a stale base.
- **Review freeze**: once a branch/PR is under review, no branch surgery
  (rebase, delete, cherry-pick) until the verdict lands — route
  restructuring through whoever is coordinating, who retargets the reviewer
  with an equivalence check.
- **Agent autonomy**: the agent decides and proceeds autonomously — making
  reasonable assumptions is fine and expected. Surface to the user only the
  genuinely big or unexpected: kill-criterion trips, scope changes,
  falsified premises, destructive/irreversible actions, and trade-offs the
  process explicitly reserves for the user (e.g. footprint-vs-power,
  belt-tier choices). Everything else: pick the recommended path, execute,
  and report in the running narrative — the user reviews asynchronously and
  will object if something looks wrong. **The trade is documentation**:
  every consequential autonomous decision (and every assumption it rests
  on) is recorded where its subject lives — the owning RFC's decision log,
  or the commit message / followups doc when no RFC owns it. An
  undocumented decision is the only kind that's not allowed.

## Architecture

**Rust workspace** (`crates/`) is where all work happens:

- **`crates/core/`** — pure shared logic: solver, recipe DB, bus layout engine, blueprint export, A\*, validation. All new features and bug fixes land here. The `wasm` feature gates WASM-only derives.
- **`crates/wasm-bindings/`** — thin wasm-bindgen wrapper exposing `solve`, `layout`, `export_blueprint`, and recipe lookups to the browser. Consumed by `web/src/engine.ts`.
- **`crates/mining-cli/`** — `blueprint-analyze` native binary for dissecting community blueprint strings (stdin / file / `--batch` / `--json`). Uses `spaghettio_core::analysis` to expand books and report entity counts, recipes, and shape summaries.

**Web app** (`web/`) is the primary interactive interface. Vite + vanilla TS + PixiJS v8 + pixi-viewport. Runs the full solver → layout → blueprint pipeline client-side via WASM. URL state encodes the recipe, rate, machine tier, external inputs, and belt tier, so links reproduce layouts exactly. Renderer-specific constraints live in `web/CLAUDE.md`.

## Tooling

- **Blueprint analyzer** — `cargo run -p spaghettio_mining --bin blueprint-analyze -- [file|--batch|--json]`. Useful for auditing community blueprints or spot-checking our own export round-trips.
- **Sim harness** (RFC-050) — `cargo run -p spaghettio_sim_harness -- run --bp <file> --manifest <file>` runs a layout in a real headless Factorio server and reports planned-vs-measured rates (`fetch` once first; `bless`/`check` freeze and enforce measured baselines). `serve` hosts the same fixture as a joinable realtime server so you can **look** at a layout in a client — see the doc for the version-match and WSL/UDP gotchas. Full how-to: [`docs/sim-harness.md`](docs/sim-harness.md). Concurrent runs against one install just work (per-run scratch write dirs); only `fetch` and `check-data` still write into the install itself.
  - **Deep chains need a long `--warmup`.** The dim-scaled default is too short for multi-stage chains and reads buffer fill as throughput; treat any default-warmup deficit as unproven until re-run with a long one. Current measurements and what they invalidate: [`docs/status.md`](docs/status.md) §"Default warmup is too short for deep chains".
- **Containerised Claude-Code runner** — `Dockerfile` + `docker-compose.yml` + `docker-entrypoint.sh` at the repo root. Ships a `node:24` image with Claude Code, `gh`, Rust, and the pi-coding-agent preinstalled. `docker compose run --rm claude-agent` drops into an interactive container with the workspace mounted and host creds (`~/.claude`, `~/.config/gh`) bind-mounted read-only. Used for one-shot / llama-backed watcher agent runs.

### Pipeline stages (all Rust)

1. **Solver** (`crates/core/src/solver.rs`) — Resolves recipe dependencies via the net-flow LP in `netflow.rs` (free cost-based recipe selection, default since 2026-07; legacy recursive tree walk retained as the compat/parity oracle), computes machine counts and flow rates. Loads `crates/core/data/recipes.json` via `include_str!`. Returns a `SolverResult`.
2. **Bus layout** (`crates/core/src/bus/`) — Deterministic row-based layout. Machines group by recipe into rows, trunks run on parallel columns, tap-offs are routed via the ghost router (negotiated congestion A* + region-growth junction solver). See [`docs/ghost-pipeline-contracts.md`](docs/ghost-pipeline-contracts.md) for the phase-by-phase contracts the router promises.
3. **Blueprint export** (`crates/core/src/blueprint.rs`) — Emits the JSON + zlib + base64 envelope directly (no draftsman dependency).
4. **Validation** (`crates/core/src/validate/`) — 40 functional checks: pipe isolation, fluid port connectivity, inserter chains + direction, power coverage + pole connectivity, belt flow/structural, underground belt pairs + sideloading, lane throughput, input-rate delivery, module slots + eligibility, record integrity (effective_rows bands + power-wire indices vs geometry, RFC-065).

## Key models (`crates/core/src/models.rs`)

- `ItemFlow` — item name, rate, fluid flag
- `MachineSpec` — machine type, recipe, count, inputs/outputs
- `SolverResult` — machines, external inputs/outputs, dependency order
- `PlacedEntity` — entity name, position, direction, recipe, carries, segment id
- `LayoutResult` — entities, connections, dimensions

## Key source files

Most-visited files. Full reference in [`docs/file-reference.md`](docs/file-reference.md).

| File | Purpose |
|------|---------|
| `crates/core/src/bus/layout.rs` | Top-level `build_bus_layout`: `place_rows` → `plan_bus_lanes` → `route_bus_ghost` → `place_poles` (poles are LAST — placed after routing, never router obstacles) |
| `crates/core/src/bus/placer.rs` | Row placement: group machines by recipe, split for throughput, `place_rows` geometry |
| `crates/core/src/bus/templates.rs` | Belt/inserter row templates (single-input, dual-input, lane-splitting sideload bridges) |
| `crates/core/src/bus/lane_planner.rs` | `BusLane` / `LaneFamily` types, `plan_bus_lanes`, lane splitting + tap-off coordinate finding |
| `crates/core/src/bus/ghost_router.rs` | Ghost A* + negotiated congestion routing; junction solver integration; output merger call-site |
| `crates/core/src/netflow.rs` | Net-flow LP solver (default since 2026-07, compatibility mode; byproduct crediting, typed cycle refusals). Legacy tree walk retained in `solver.rs` as the recipe-selection oracle. See `docs/rfc-solver-net-flow.md`. |
| `crates/core/src/validate/` | The 40 functional checks, dispatched from `mod.rs` (`belt_flow` lane-rate walker, `belt_structural`, `fluids`, `inserters`, `modules`, `power`, `underground`) |
| `crates/core/src/trace.rs` | Thread-local trace event collector; `TraceEvent` variants drive the snapshot debugger and stress scoreboards |
| `crates/core/src/snapshot.rs` | `.fls` snapshot reader/writer for the layout debugger |
| `crates/core/tests/e2e.rs` | End-to-end test harness: tier regression tests and stress corpus with scoreboards |
| `crates/wasm-bindings/src/lib.rs` | wasm-bindgen wrapper exposing `solve`, `layout`, `export_blueprint` to the browser |

## Factorio game rules (constraints for the layout engine)

Physical rules the layout engine must satisfy:

- **Machines** craft recipes, need ingredients delivered and products extracted.
- **Inserters** pick from one side, drop to the other. Regular reach 1 tile; long-handed reach 2.
- **Belts** move items in a direction, connect when adjacent. Tiers: yellow 15/s, red 30/s, blue 45/s.
- **Pipes** carry fluids and merge with any adjacent pipe — separate fluid networks must be physically isolated.
- **Fluid ports** on machines are at specific tile positions that depend on direction and `mirror`.
- **Fluid-box mirroring (Space Age)** — entities with fluid boxes support `mirror: true` in the blueprint. Combined with `direction`, this gives 8 orientations (4 rotations × 2 mirrors). Only honored by Factorio 2.0+.
- **Entities** cannot overlap.
- **Power** — machines need electricity; medium-electric-pole covers a 7×7 area.
- **Belt lane mechanics** — sideloading, UG lane rules, splitter behavior — detailed in [`docs/factorio-mechanics.md`](docs/factorio-mechanics.md).

## Verification protocol for layout engine changes

Layout bugs are easy to get wrong — zero validation errors can mean the check was wrong, not that the layout is. Follow this protocol:

1. **Run the full e2e suite** — `cargo test --manifest-path crates/core/Cargo.toml`. All non-ignored tests across `crates/core/tests/` must stay green.
2. **Measure it, don't look at it** (revised 2026-08-08 — the browser eyeball used to be this step). The validator is not ground truth, but the eyeball was never a good substitute: it catches *disconnected*, not *underdelivering*, and the defects that survive validation are overwhelmingly the second kind. Two instruments now cover it properly:
   - **The fast meter** (`crates/meter/`, seconds) — `check_one <dir>` on an exported fixture. Its asymmetry is the calibrated one: **"meter says below plan ⇒ believe it"**; *"meter says at plan"* is evidence of nothing (the floor property does **not** hold post-lift — see [`meter-divergence.md`](docs/meter-divergence.md) §2026-08-08). So it refutes cheaply and clears nothing.
   - **The sim** ([`sim-harness.md`](docs/sim-harness.md), minutes) — ground truth, and the only thing that can *clear* a layout. Anchor anything that changes shipped geometry.

   **Eyes are still the best tool for *where*, and they are not a gate.** "Numbers say how much; eyes say where" — a served world is how #607 was found (the owner asked why a bank of inserters was bridging two belts, which no rate figure would have surfaced). Reach for it when an instrument says something is wrong and you cannot see why, not to sign off a change.
3. **Check the snapshot for the exact bug you intended to fix** — `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ... --nocapture <test>` then decode with the snippet in [`docs/layout-snapshot-debugger.md`](docs/layout-snapshot-debugger.md). Inspect entities at the suspect coordinates, not just the warning count.
4. **Trace events are reliable signals** — `JunctionGrowthCapped`, `JunctionStrategyAttempt`, `GhostSpecFailed`, `CrossingZoneSkipped`, `BalancerStamped`, `TapBridgeUnbridgeable`, `LayoutRetried` are emitted by the pipeline and land in the snapshot's `trace.events`. Use them to confirm the specific failure mode before theorizing. (`RouteFailure` and `BridgeDropped` are declared in `trace.rs` but never emitted — don't wait for them.)
5. **Don't trust an error-count drop alone** — if warnings go 5 → 0, ask *why*. Does the topology still make sense? Were belts actually re-routed, or did a check get silently skipped? Check the specific change caused the fix you wanted.
6. **Clippy + WASM builds are checks, not nits** — a layout change that clippy-fails or breaks the WASM build is not done.
7. **A check going quiet is not evidence the problem is fixed** — it is equally consistent with the check having stopped discriminating. Verify the specific invariant: instrument it and *count*, don't sample. When writing a check, emit one positioned issue per instance rather than a count inside a message — anything comparing issue counts by category cannot tell 2 from 218. This failure mode has recurred ten times; the rules and the evidence are in [`docs/validator-reporting.md`](docs/validator-reporting.md).

## Where to find X

| Looking for | Location |
|-------------|----------|
| Project status (complexity ladder, residual warnings, open issues) | [`docs/status.md`](docs/status.md) |
| Recipe data | `crates/core/data/recipes.json` (embedded via `include_str!`) |
| Balancer templates | `crates/core/src/bus/balancer_library.rs`. Regenerate: `python scripts/generate_balancer_library.py` (needs Factorio-SAT on `PATH`). |
| Belt tier thresholds | `crates/core/src/common.rs` (`belt_entity_for_rate`, `ug_max_reach`) |
| Entity sizes | `crates/core/src/common.rs` (`entity_size`) |
| Validation checks | `crates/core/src/validate/` (40 checks, dispatched from `mod.rs`) |
| How a check should report (and the ten times it didn't) | [`docs/validator-reporting.md`](docs/validator-reporting.md) |
| What `PlacedEntity::rate` denotes (an aggregate — **never** per-tile flow) | [`docs/rate-stamp-semantics.md`](docs/rate-stamp-semantics.md) |
| What a check's report is worth (severity, selection participation, calibration receipts, known holes) | [`docs/validator-trust.md`](docs/validator-trust.md) — update it in the same PR as any severity/category/selection change |
| Snapshot format | `crates/core/src/snapshot.rs` + [`docs/layout-snapshot-debugger.md`](docs/layout-snapshot-debugger.md) |
| Sim measurement semantics + forensics | [`docs/sim-harness-forensics.md`](docs/sim-harness-forensics.md) (what each spaghettio-sim number means, artifact classes, debug playbook) |
| Belt lane physics | [`docs/factorio-mechanics.md`](docs/factorio-mechanics.md) |
| Sim harness (headless Factorio measurement runs) | [`docs/sim-harness.md`](docs/sim-harness.md) |
| Ghost pipeline contracts | [`docs/ghost-pipeline-contracts.md`](docs/ghost-pipeline-contracts.md) |
| Walker-veto debugging | [`docs/walker-veto-debugging.md`](docs/walker-veto-debugging.md) |
| Build commands | [`docs/build-systems.md`](docs/build-systems.md) |
| Full source file list | [`docs/file-reference.md`](docs/file-reference.md) |

## Visualizations

The web app at `http://localhost:5173` is the primary visualization — any URL (`?item=...&rate=...&in=...&belt=...`) renders a live layout with entity overlays, segment highlighting, and validation markers. The same app is deployed to GitHub Pages on every push to main: https://storkme.github.io/spaghettio/
