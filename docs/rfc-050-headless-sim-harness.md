# RFC-050: Headless simulation harness (`spaghettio-sim`)

## Summary

A **standalone, offline engineer tool** that takes a generated blueprint
string, builds it inside headless Factorio on an idealized lab world,
injects power and inputs, drains outputs, simulates past warmup, and
emits a JSON report of **planned vs measured** per-item rates. This is
the audit's §8.3 item (`architecture-audit-2026-07.md`), fully de-risked
by live discovery (`headless-verification-discovery.md`): anonymous
headless download works, Space Age loads without entitlement, our
blueprints import and revive through the game's own parser (149/149 on a
real factory, single electric network), and simulation runs ~11× real
time untuned. The harness converts the audit's core criticism — "the
confidence chain bottoms out at self-consistency, not the game" — into a
command an engineer runs at close-outs, and retires the standing pattern
of user-run in-game anchors.

## Motivation

Every rate-shaped claim the project makes today is validator-verified
only: the validator checks the model against the model (audit §3.3 —
`carries` labels are "the engine grading its own homework"; no
end-to-end mass balance exists). Ground truth has meant asking the user
to paste blueprints in-game — open anchors from rfc-inserter-sizing
(KC5), rfc-build-quality, and rfc-046 are all queued on that. The wires
bug (RFC-040) showed the cost concretely: five phases of model-side
power work shipped artifacts that pasted power-dead, caught only by a
manual paste. Discovery has already produced one first: the RFC-040
`wires` array was verified by the game engine, automated, during the
§8.3 spike.

## Design

### Shape: offline tool, not a validator check, not (initially) CI

- New workspace crate **`crates/sim-harness/`** (binary `spaghettio-sim`),
  mirroring `mining-cli`'s pattern. Input: a blueprint string (file /
  stdin) plus a **verification manifest** (target item + rate, external
  inputs with per-item rates, solver-planned output rate); or a fixture
  name that generates both via the engine.
- The headless install is an external dependency resolved at runtime
  (`SPAGHETTIO_FACTORIO_DIR`, with a `--fetch` helper that downloads a
  **pinned version** — never `latest` — to a cache dir). Not vendored,
  not baked into published images (EULA).
- Runs are ~30–60 s each: a different verification tier than the
  millisecond validator, used like STRESSGOLD — opt-in at close-outs and
  investigations. CI enforcement is a non-goal until the tool has a
  track record (revisit in the close-out).

### Engine-side addition: the verification manifest

One small core change: `blueprint::export_with_manifest` (or a sibling
fn) emitting a JSON sidecar from data the pipeline already has — target
item/rate, `SolverResult.external_inputs` (item, rate, fluid flag), and
planned output rate. The blueprint itself already marks injection and
drain points: bus entities carry `type: "input"` / `"output"` tags
(`io_type`), so the harness locates boundary belts from the artifact
alone. No layout changes.

### The scenario (Lua, shipped as a template in the crate)

Discovery-verified skeleton plus the boundary kit:

1. Lab surface (`generate_with_lab_tiles`), all other surfaces deleted
   (the untuned benchmark showed a leftover Nauvis eats sim time);
   `research_all_technologies()` (stack inserters, belt stacking).
2. `import_stack` (rc recorded — a nonzero rc is itself a FAIL),
   `build_blueprint{superforced}`, revive all ghosts; fulfill
   `item-request-proxy` module requests by direct insertion, destroy the
   proxies (discovery finding 7).
3. **Power**: one `electric-energy-interface` per electric network
   (networks enumerated via pole `electric_network_id`).
4. **Feed**: for each blueprint entity tagged `input`, an
   `infinity-chest` (set to the manifest item) + `loader-1x1` pair
   feeding the belt head at full compression; `infinity-pipe` at fixed
   fill for fluid inputs.
5. **Drain**: `loader-1x1` → remove-mode `infinity-chest` at each
   `output` belt end, so backpressure cannot stall the factory and
   falsify the measurement.
6. **Warmup then measure**: run W ticks (default 7200 = 2 game-minutes),
   then read `LuaForce::get_item_production_statistics(surface)
   .get_flow_count` over a measurement window M (default 3600); repeat a
   second window and require the two windows to agree within tolerance
   (steady-state check) before reporting. NOTE: `"input"` in the flow API
   means *produced* (inverted naming — discovery/audit).
7. `helpers.write_file` the report; orchestrator polls, kills the
   server, parses.

### The report

Per item: planned rate, measured rate, delta %; plus import rc, ghost /
revived / failed counts, electric-network count, unfulfilled-proxy
count, warmup/window parameters, game version, and a PASS/WARN/FAIL
verdict per the tolerance. Output shapes: human table + `--json`.

### Determinism and baselines (later phase)

Factorio's sim is deterministic: same save + same script ⇒ identical
counts. Phase 3 adds `--bless`/`--check` measured baselines per fixture
(STRESSGOLD pattern) — a *measured* golden, the first regression gate in
the project backed by the game.

## Non-goals

- **Automated feedback into generation** (tuning the generator from sim
  results) — a different arc; this RFC produces reports and gates,
  humans close the loop.
- CI enforcement (revisit at close-out with runtime data).
- UPS tuning beyond save hygiene; parallel fleet orchestration.
- Simulating **imported** community blueprints (drills/pumpjacks need
  resource patches — discovery finding 8a; our generated factories never
  place them). Follow-up if wanted.
- Gleba spoilage / per-quality flow statistics (needed only when the
  generator plans quality *production*, which it doesn't).

## Kill criteria

1. **Phase 0 — data drift gate.** Re-extract against the pinned game
   version's data (draftsman or the game's own dumps). If any recipe /
   machine fact the solver consumes differs from `recipes.json` (2.0.76
   era) and can't be reconciled by re-baselining the data in ≤1 day,
   STOP the harness arc and run a data-refresh arc first — measured
   rates against drifted prototypes are meaningless. (Headless `latest`
   is already 2.1.12 with a new `recycler` core module.)
2. **Phase 1 — calibration.** The tier-1 gear fixture (and then EC@10/s)
   must measure within **±2% of planned** at steady state. If the delta
   exceeds that and a day of investigation attributes it to
   harness-boundary artifacts (feeding starvation, drain backpressure,
   window placement) that redesign doesn't close, stop and rethink the
   boundary kit before measuring anything else. If instead the delta is
   a REAL engine defect — that's the tool working; file it, measure on.
3. **Determinism.** Two identical runs must produce identical measured
   counts. Nondeterminism that survives investigation kills the blessed-
   baseline phase (3) and demotes the tool to exploratory-only.
4. **Wall-clock budget.** If a single blueprint cycle exceeds ~5 min at
   USP scale (6.8k entities) on a dev machine after save hygiene, the
   at-close-out workflow premise fails — rescope to nightly batch only.
5. **Version fragility.** If the scenario needs game-version-specific
   workarounds that break across TWO consecutive minor updates, the
   maintenance premise fails — freeze support to the pinned version and
   record the constraint.

## Verification plan

- Phase 1 exit: gear + EC fixtures, planned-vs-measured within ±2%,
  determinism double-run, report artifacts checked into the RFC decision
  log (numbers, not screenshots).
- The discovery doc's repro commands re-run against the pinned version.
- Six-pack sweep (Phase 2) cross-checked against `docs/status.md`'s
  gauntlet table — the first game-side confirmation (or refutation) of
  the ledger's PASS column, and the retirement vehicle for the open
  in-game anchors (each retirement decision-logged in the owning RFC).

## Phasing

| Phase | Deliverable | Gate |
|-------|-------------|------|
| 0 | Pinned-download helper + **data-drift check** vs recipes.json | KC1 |
| 1 | Harness MVP: manifest export + scenario + report; gear & EC calibrated | KC2, KC3 |
| 2 | Fixture sweep (six-pack), anchor retirements, USP-scale timing | KC4 |
| 3 | `--bless`/`--check` measured baselines (opt-in, STRESSGOLD pattern) | KC3 |

## Decision log

- *2026-07-22 — drafted after the §8.3 discovery spike
  (`headless-verification-discovery.md`); shape decision: standalone
  offline engineer tool (not validator, not CI) with report-first
  feedback — automated generator tuning explicitly out of scope. Awaiting
  adversarial review.*
