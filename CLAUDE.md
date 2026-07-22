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
- **WASM rebuild**: `wasm-pack build crates/wasm-bindings --target web --out-dir "$(pwd)/web/src/wasm-pkg"`. Always pass an absolute `--out-dir` (see memory: `feedback_wasmpack_outdir`).
- **Pre-commit hooks**: in `.githooks/pre-commit`, activate with `git config core.hooksPath .githooks`. Runs `cargo clippy` on staged Rust and `tsc` on staged TS. Bypass with `--no-verify` only for genuine emergencies.
- **Scripts**: put exploratory snippets in `scripts/` rather than inline one-liners. Rust debug scripts go in `crates/core/examples/` or as `#[test] #[ignore]` benchmarks.
- **Snapshots**: `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ...` writes `.fls` files under `crates/core/target/tmp/`. Decode with `tail -c +5 <file> | base64 -d | gunzip`. See [`docs/layout-snapshot-debugger.md`](docs/layout-snapshot-debugger.md).
- **Docs taxonomy** (official as of 2026-07-20; everything in `docs/` is one of these):
  - **RFCs** (`rfc-*.md`) — spec docs plus context: design, **kill criteria**
    (required — the dominant rework shape here is exploration that overruns
    its evidence), verification plan, and a **decision log**, which is the
    canonical record of every call made while the work ran. Template:
    [`docs/rfc-template.md`](docs/rfc-template.md). (Standard industry sense —
    a design doc circulated for review; these were called "RFPs" here before
    2026-07-20, renamed because that term means something else entirely. The
    decision log is the part that matters.) **Numbered**: the
    registry at [`docs/rfcs.md`](docs/rfcs.md) assigns `RFC-NNN` to every RFC
    chronologically; existing files keep their names (numbers live in the
    registry), new RFCs are named `rfc-NNN-short-name.md` and get a registry
    row in the same commit. Rejected/obsolete RFCs move to `docs/archive/`.
  - **Followups** (`*-followups.md`) — deferred-work backlogs with pick-up
    notes. Named for what they track, not the session that produced them;
    status line at the top so a cold pick-up knows what's open.
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
  the multiple concurrent Claude sessions that share this repo — interleaved
  direct commits from two sessions in one checkout caused real branch
  tangles (2026-07-20).
- **Adversarial review before anything is commit-ready** — code *and*
  documentation. Preferred: the CI review bot —
  [`.github/workflows/claude-code-review.yml`](.github/workflows/claude-code-review.yml)
  runs a Claude code review on every PR (opened/synchronized/reopened), and
  `clear-agent-reviewed.yml` drops the `agent-reviewed` label when new
  commits land so the new SHA gets re-reviewed. **A green `claude-review`
  check is NOT evidence a review happened — confirm the bot actually
  posted.** From #305 through #330 the bot posted *nothing*: three
  stacked, each-sufficient causes — template read-only permissions
  (fixed #327), the plugin's `--comment` flag never passed (its own
  contract is "do not post" without it; fixed #329), and no
  harness-level tool allowlist (`claude_args --allowedTools`), which
  denied every posting/diff call (fixed #331). Validated 2026-07-21 via
  a planted-bug canary (#330): the bot's first-ever comment correctly
  flagged the bug inline with a committable fix. All substantive PR
  review feedback before then was session-side. Expected behavior now:
  inline comments on findings, or a "no issues" summary comment on
  clean substantive PRs — a green check with *neither* on a
  non-trivial PR means it's broken again. Known benign no-comment
  cases: PRs that modify the workflow file itself (the action's
  anti-hijack self-skip) and changes its triviality gate deems
  obviously correct. Local adversarial review (an
  independent agent that re-runs gates and probes the claims) is the
  fallback when a PR isn't in play — and it remains **required in addition
  to the bot** for layout-engine or validator-semantics changes: the bot
  reviews the diff, but it cannot run STRESSGOLD, decode snapshots, or do
  tile-level verification, and every materially wrong claim caught in the
  2026-07 power arc was caught by that deeper class of review.
- **Keep `origin/main` current** — push promptly after merging. Worktree
  agents branch from `origin/main`; a stale origin hands every spawned agent
  a stale base (35 unpushed commits once sent an agent rebuilding a fix
  against a pre-arc tree).
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

**Web app** (`web/`) is the primary interactive interface. Vite + vanilla TS + PixiJS v8 + pixi-viewport. Runs the full solver → layout → blueprint pipeline client-side via WASM. URL state encodes the recipe, rate, machine tier, external inputs, and belt tier, so links reproduce layouts exactly.

## Tooling

- **Blueprint analyzer** — `cargo run -p spaghettio_mining --bin blueprint-analyze -- [file|--batch|--json]`. Useful for auditing community blueprints or spot-checking our own export round-trips.
- **Containerised Claude-Code runner** — `Dockerfile` + `docker-compose.yml` + `docker-entrypoint.sh` at the repo root. Ships a `node:24` image with Claude Code, `gh`, Rust, and the pi-coding-agent preinstalled. `docker compose run --rm claude-agent` drops into an interactive container with the workspace mounted and host creds (`~/.claude`, `~/.config/gh`) bind-mounted read-only. Used for one-shot / llama-backed watcher agent runs — see commit `56e2eeb`.

### Pipeline stages (all Rust)

1. **Solver** (`crates/core/src/solver.rs`) — Resolves recipe dependencies via the net-flow LP in `netflow.rs` (free cost-based recipe selection, default since 2026-07; legacy recursive tree walk retained as the compat/parity oracle), computes machine counts and flow rates. Loads `crates/core/data/recipes.json` via `include_str!`. Returns a `SolverResult`.
2. **Bus layout** (`crates/core/src/bus/`) — Deterministic row-based layout. Machines group by recipe into rows, trunks run on parallel columns, tap-offs are routed via the ghost router (negotiated congestion A* + region-growth junction solver). See [`docs/ghost-pipeline-contracts.md`](docs/ghost-pipeline-contracts.md) for the phase-by-phase contracts the router promises.
3. **Blueprint export** (`crates/core/src/blueprint.rs`) — Emits the JSON + zlib + base64 envelope directly (no draftsman dependency).
4. **Validation** (`crates/core/src/validate/`) — 34 functional checks: pipe isolation, fluid port connectivity, inserter chains + direction, power coverage + pole connectivity, belt flow/structural, underground belt pairs + sideloading, lane throughput, input-rate delivery, module slots + eligibility.

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
| `crates/core/src/bus/layout.rs` | Top-level `build_bus_layout`: `place_rows` → `plan_bus_lanes` → `route_bus_ghost` → `place_poles` (poles are LAST — placed after routing, never router obstacles; invariant restored 2026-07-19, see `docs/rfc-power-supply.md` Phase 0f) |
| `crates/core/src/bus/ghost_router.rs` | Ghost A* + negotiated congestion routing; junction solver integration; output merger call-site |
| `crates/core/src/bus/lane_planner.rs` | `BusLane` / `LaneFamily` types, `plan_bus_lanes`, lane splitting + tap-off coordinate finding |
| `crates/core/src/bus/lane_order.rs` | Left-to-right lane column order optimiser (exact search ≤7 lanes, hill-climb above) |
| `crates/core/src/bus/balancer.rs` | `stamp_family_balancer` + splitter/UG name helpers |
| `crates/core/src/bus/trunk_renderer.rs` | `render_path` (A* path → belts), `trunk_segments`, `is_intermediate` |
| `crates/core/src/bus/output_merger.rs` | Final-product east-flowing output merger |
| `crates/core/src/bus/placer.rs` | Row placement: group machines by recipe, split for throughput, `place_rows` geometry |
| `crates/core/src/bus/templates.rs` | Belt/inserter row templates (single-input, dual-input, lane-splitting sideload bridges) |
| `crates/core/src/bus/junction_solver.rs` | Region-growth junction solver framework (trait, growth loop) |
| `crates/core/src/bus/junction_sat_strategy.rs` | SAT-backed `JunctionStrategy` fallback |
| `crates/core/src/bus/ghost_occupancy.rs` | Typed `Occupancy` map (HardObstacle / RowEntity / Permanent / GhostSurface / Template / SatSolved) |
| `crates/core/src/bus/balancer_library.rs` | Pre-generated N→M balancer templates (do not edit manually) |
| `crates/core/src/netflow.rs` | Net-flow LP solver (default since 2026-07, compatibility mode; byproduct crediting, typed cycle refusals). Legacy tree walk retained in `solver.rs` as the recipe-selection oracle. See `docs/rfc-solver-net-flow.md`. |
| `crates/core/src/astar.rs` | `ghost_astar` pathfinder primitive (the negotiated-congestion loop lives in `bus/ghost_router.rs`) |
| `crates/core/src/sat.rs` | Varisat-backed crossing-zone SAT solver (see memory: `project_sat_crossing_solver`) |
| `crates/core/src/validate/belt_flow.rs` | Lane-rate walker (Kahn topo sort with splitter pairing and balancer feedback-loop handling) |
| `crates/core/src/validate/` | Rest of the 34 checks: `belt_structural`, `fluids`, `inserters`, `modules`, `power`, `underground` |
| `crates/core/src/trace.rs` | Thread-local trace event collector; `TraceEvent` variants drive the snapshot debugger and stress scoreboards |
| `crates/core/src/snapshot.rs` | `.fls` snapshot reader/writer for the layout debugger |
| `crates/core/tests/e2e.rs` | End-to-end test harness: tier1–4 regression tests and stress corpus with scoreboards |
| `crates/wasm-bindings/src/lib.rs` | wasm-bindgen wrapper exposing `solve`, `layout`, `export_blueprint` to the browser |
| `web/src/engine.ts` | WASM loader and typed wrappers |
| `web/src/renderer/entities.ts` | PixiJS entity renderer (bus layout view) |
| `web/src/ui/sidebar.ts` | Searchable item picker, rate input, live solve, URL state |

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

## Recipe complexity ladder

Tracks which recipes produce zero-error bus blueprints. Moving up = real progress. Tests for each tier live in `crates/core/tests/e2e.rs`.

| Tier | Recipe | Complexity | Bus status |
|------|--------|-----------|-----|
| 1 | `iron-gear-wheel` | 1 recipe, 1 solid input | SOLVED |
| 2 | `electronic-circuit` | 2 recipes, 2 solid inputs | SOLVED (incl. from ores) |
| 3 | `plastic-bar` | 1 recipe, 1 fluid + 1 solid input | SOLVED |
| 4 | `advanced-circuit` | 5+ recipes, mixed solid/fluid | SOLVED (`tier4_advanced_circuit_from_ore_am2` green: AC@5/s ores AM2 yellow, 0 errors). Carries 1 input-rate-delivery (unrelated, pre-existing demand-pull modeling residual); inserter-item-throughput 0 since the last-in-row belt extension (`0d7132c`, 2026-07-19; was 4, and 58 masked sides pre-`rfc-inserter-sizing.md`). From plates still has lane-throughput warnings, [#65](https://github.com/storkme/spaghettio/issues/65). |
| 5 | `processing-unit` | Deep chain, multiple fluids | SOLVED (`tier5_processing_unit_from_ore_am3` green: PU@2/s ores AM3 red, 0 errors, Pooled — fully clean, 0 warnings, since the last-in-row belt extension `0d7132c` 2026-07-19; was 5 inserter-item-throughput, and 129 masked sides pre-`rfc-inserter-sizing.md`). Higher rates / partitioned strategies still have junction + starvation issues — see `partition_strategy_scoreboard_extended`. |
| 6 | `flying-robot-frame` | Adds lubricant: advanced-oil-processing refinery rows with 3 fluid outputs | SOLVED via the USP chain (0 errors). The 2026-07-11 "0 warnings" reading predates the per-item inserter-attribution check landing — see tier 7 and the corpus-wide note below. No dedicated FRF fixture yet. |
| 7 | `utility-science-pack` | Very deep chain (LDS + PU + FRF) | SOLVED (`science_gauntlet` USP@1/s AM3: 0 errors, 6615 entities, 208×281). Utility itself fully clean since the last-in-row belt extension (`0d7132c`, 2026-07-19; was 2 inserter-item-throughput). Across the six packs the only residual is production-science: 8 inserter-item-throughput, likely the same last-in-row trim still present in the triple/quad/hstack templates (follow-up). Logistic/military science packs clean at 1/s (previously carried input-rate-delivery residue, since fixed). |

**`rfc-inserter-sizing.md` close-out (2026-07-13)**: bus inserters are now sized to planned per-machine throughput via a shared regular→fast→stack ladder (long-handed count-ladder for reach-2 sides), with an ingredient-to-belt reassignment lever and a user-facing `max_inserter_tier` engine param (wasm-bindings + web UI, URL-encoded). `science_gauntlet` 1/s inserter-throughput/item-throughput warnings across the six packs: **140 → 12** at close-out (automation/logistic/military fully clean; chemical 1, production 9, utility 2 residual, all under the newer, stricter per-item check — the old aggregate check is at 0 everywhere), then **12 → 8** after the 2026-07-19 last-in-row belt extension (`0d7132c`: chemical and utility now clean; all 8 remaining are production-science). The "untouched triple/quad/hstack trims" hypothesis for those 8 was **falsified 2026-07-20** (`acd147e` extended the pattern to triple_input_row — quad/hstack are structurally immune — and the 8 turned out to be 6 input3 contest-losses + 2 genuine far-side rate walls; see [`docs/inserter-throughput-followups.md`](docs/inserter-throughput-followups.md)). This is **validator-verified only** — the RFC's two in-game blueprint-import anchors (kill criterion 5) remain open until the user runs them; see the decision log in `docs/rfc-inserter-sizing.md` for the full phase-by-phase evidence trail.

**`rfc-build-quality.md` close-out (2026-07-20)**: user-facing **build quality** param (normal→legendary, `quality`/`q=` URL-encoded through wasm `solve`+`layout` and the sidebar). Solver machine counts scale ×(1+0.3·level) via `effective_crafting_speed`; the inserter ladder, pole supply radii (+1/level), and wire reach (+2/level, shared table `common::pole_wire_reach` consumed by placement, the emitted `wires` artifact, and the validator) are quality-aware; functional entities (machines/inserters/poles — never logistics) get `PlacedEntity.quality` stamped in one `layout_pass` post-pass, validators rate each entity by its own tier, and export emits the lua-api `quality` field (parser reads it too, so imported quality blueprints validate). Normal is bit-identical to pre-RFC (kill-criterion-2 gates: unit bit-equality sweeps + full suite + STRESSGOLD check). The 60 EC/s legendary headline is capped at 45/s (one blue belt) until [#311 output-merger capacity](https://github.com/storkme/spaghettio/issues/311) closes; [#312](https://github.com/storkme/spaghettio/issues/312) tracks the quality-magnified consumer-clamped fan-in wall; [#310 pole-band thinning](https://github.com/storkme/spaghettio/issues/310) is the designated next pick-up. **In-game import anchor still open** (user-run; unblocked — #313 resolved as premise-falsified: the engine's `stack-inserter` IS the current Space Age stacking inserter). Full trail: `docs/rfc-build-quality.md` decision log; renderer constraints learned en route: `web/CLAUDE.md`.

**`rfc-046-belt-stacking.md` close-out (2026-07-21)**: user-facing **belt stacking** param (off/×2/×3/×4 = Space Age belt stack size research, `stacking`/`st=` URL-encoded through wasm `layout*` and the sidebar; solver untouched). Belt tier selection, lane caps, merger capacity, and the validators scale ×S via `common::*_stacked` helpers; belt-dropping output sides are **forced to stack inserters** at S>1 (`size_belt_drop_side`, modeling the belt-drop hand rounding incl. the S=4 dip: 9.6/s vs 14.4/s per inserter); a **static family-level exemption** (`bus/stacking_ctx.rs`) keeps uniform ×S sound for unstackable producers (self-loop/kovarex, D2b secondary outputs, recycler ejection — validators re-derive it independently, per-tile). Full-belt delivery thresholds initially did NOT scale (tap/sideload flow fills one lane) — *superseded by RFC-047 (2026-07-22), which grounded and scaled them.* Headline: the #311 stress config (EC@60/s red from ore, whose golden stamps a physically-impossible 60/s merger belt) is **physically valid end-to-end at S=2** (red stacked = 60/s), proven by an in-fixture per-tile capacity audit. S=1 is bit-identical to pre-RFC (zero golden re-blesses). Mechanics: `factorio-mechanics.md` BS1–BS7. In-game import anchor open (user-run — red/normal headline config, or legendary-express since RFC-047 resolved its residuals; #335's one-bank ore-routing warnings persist there). Full trail: `docs/rfc-046-belt-stacking.md` decision log.

**`rfc-047-lane-aware-tap-delivery.md` close-out (2026-07-22)**: made delivery **lane-aware** so belt stacking raises rate CEILINGS, not just belt tiers. Leg A: the lane-rate walker's convergence-phase splitter model was physically false (pooled lanes — real splitters preserve them, S4) — replaced by `splitter_output_rates_convergence`, exposing #334 (two lane-imbalanced balancer-library shapes, carved out with a fix tripwire); the mechanics doc's I5 was backwards (inserters drop the FAR lane — code was always right). Leg B: RFC-046's stacking-blind row-split cap was fragmenting rows at S>1 and manufacturing sideload overloads — fixed at the root; a late sideload check now refuses multi-producer single-trunk over-cap shapes by name (exposed 38 pre-existing SILENT S=1 overload errors in a fixture that never asserted on them; (n,1) merge-tap is unwired, #336). Leg C: the fan-in wall scales ×S on geometry-grounded credits — EC@6/s legendary yellow refuses at S=1 and builds clean at S=2 (`stacking_fanin_wall_lift_...`), and the **original legendary-express@60 headline landed** (`stacking_ec_60s_express_legendary_s2`: junction failure died with the row consolidation, ±3% overshoot died with worst-lane output-belt sizing; #335 tracks one unreached furnace bank). Three falsified premises decision-logged. Zero golden re-blesses across the arc. Full trail: `docs/rfc-047-lane-aware-tap-delivery.md`.

Open tracking issues for layout quality: [#135 balancer templates are oversized](https://github.com/storkme/spaghettio/issues/135), [#136 missing coprime balancer shapes](https://github.com/storkme/spaghettio/issues/136), [#68 fluid row 3-tile pitch](https://github.com/storkme/spaghettio/issues/68) (design: [`docs/rfc-fluid-dual-input-row.md`](docs/rfc-fluid-dual-input-row.md)).

Deferred tooling tasks — test-suite time recovery (audited 2026-07-19, pick-up notes per item in [`docs/test-suite-followups.md`](docs/test-suite-followups.md)): committed STRESSGOLD baseline goldens landed 2026-07-19 (`SPAGHETTIO_STRESS_GOLDEN=check|bless`, see `crates/core/tests/goldens/stress/README.md` — host-cache-relative, opt-in, not CI-enforced); CI nextest parallelism re-enable via timeout-ceiling bumps (~5 min/push, experiment already documented in `.config/nextest.toml`); `[profile.test]` opt experiment for SAT/A*-heavy tests (measure before adopting).

## Verification protocol for layout engine changes

Layout bugs are easy to get wrong — zero validation errors can mean the check was wrong, not that the layout is. Follow this protocol:

1. **Run the full e2e suite** — `cargo test --manifest-path crates/core/Cargo.toml`. All non-ignored tests across `crates/core/tests/` must stay green.
2. **Load the case in the browser** — start the dev server, open the URL for the recipe you changed, and look at the layout with your eyes. A zero-warning layout that visibly has disconnected belts is a validator bug, not a success.
3. **Check the snapshot for the exact bug you intended to fix** — `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ... --nocapture <test>` then decode with the snippet in [`docs/layout-snapshot-debugger.md`](docs/layout-snapshot-debugger.md). Inspect entities at the suspect coordinates, not just the warning count.
4. **Trace events are reliable signals** — `JunctionGrowthCapped`, `JunctionStrategyAttempt`, `GhostSpecFailed`, `CrossingZoneSkipped`, `BalancerStamped`, `TapBridgeUnbridgeable`, `LayoutRetried` are emitted by the pipeline and land in the snapshot's `trace.events`. Use them to confirm the specific failure mode before theorizing. (`RouteFailure` and `BridgeDropped` are declared in `trace.rs` but never emitted — don't wait for them.)
5. **Don't trust an error-count drop alone** — if warnings go 5 → 0, ask *why*. Does the topology still make sense? Were belts actually re-routed, or did a check get silently skipped? Check the specific change caused the fix you wanted.
6. **Clippy + WASM builds are checks, not nits** — a layout change that clippy-fails or breaks the WASM build is not done.

## Where to find X

| Looking for | Location |
|-------------|----------|
| Recipe data | `crates/core/data/recipes.json` (embedded via `include_str!`) |
| Balancer templates | `crates/core/src/bus/balancer_library.rs`. Regenerate: `python scripts/generate_balancer_library.py` (needs Factorio-SAT on `PATH`). |
| Belt tier thresholds | `crates/core/src/common.rs` (`belt_entity_for_rate`, `ug_max_reach`) |
| Entity sizes | `crates/core/src/common.rs` (`entity_size`) |
| Validation checks | `crates/core/src/validate/` (34 checks, dispatched from `mod.rs`) |
| Snapshot format | `crates/core/src/snapshot.rs` + [`docs/layout-snapshot-debugger.md`](docs/layout-snapshot-debugger.md) |
| Belt lane physics | [`docs/factorio-mechanics.md`](docs/factorio-mechanics.md) |
| Ghost pipeline contracts | [`docs/ghost-pipeline-contracts.md`](docs/ghost-pipeline-contracts.md) |
| Walker-veto debugging | [`docs/walker-veto-debugging.md`](docs/walker-veto-debugging.md) |
| Build commands | [`docs/build-systems.md`](docs/build-systems.md) |
| Full source file list | [`docs/file-reference.md`](docs/file-reference.md) |

## Visualizations

The web app at `http://localhost:5173` is the primary visualization — any URL (`?item=...&rate=...&in=...&belt=...`) renders a live layout with entity overlays, segment highlighting, and validation markers. The same app is deployed to GitHub Pages on every push to main: https://storkme.github.io/spaghettio/
