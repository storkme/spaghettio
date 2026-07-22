# RFC-050: Headless simulation harness (`spaghettio-sim`)

Rev 2 — revised 2026-07-22 after dual adversarial review (architecture
lens + probe-driven sim-mechanics lens, both ACCEPT-WITH-CHANGES) **and a
live dogfood run against issue #345 that found and fixed a
factory-killing export bug before this RFC was even accepted** (see
Motivation; PR #348). Both reviewers and the dogfood independently
falsified rev 1's boundary-location mechanism; rev 2's design is the
empirically-validated one.

## Summary

A **standalone, offline engineer tool** that takes a generated blueprint
string plus a verification manifest, builds the factory inside headless
Factorio on an idealized lab world, injects power and inputs, drains
outputs, simulates past warmup, and emits a JSON report of **planned vs
measured** per-item rates. The import path was de-risked by the
discovery spike (`headless-verification-discovery.md`); the measurement
half (boundary kit, pacing, offsets, statistics) has now been
**executed live** by the #345 dogfood and both review probes. The
harness converts the audit's core criticism — "the confidence chain
bottoms out at self-consistency, not the game" — into a command an
engineer runs at close-outs, and retires the standing pattern of
user-run in-game rate/import anchors (browser-eyeball anchors remain).

## Motivation

Every rate-shaped claim the project makes is validator-verified only,
and the validator checks the model against the model. The cost of that
is no longer hypothetical: **on its first real measurement attempt
(dogfooding issue #345), the harness prototype discovered that every
blueprint this project has ever exported had all inserters running
backwards in-game** — Factorio reads an inserter's `direction` as its
*pickup* side ("inserters point backwards"), the engine's convention is
drop-side, and the exporter never translated. Input inserters pulled
from empty machines, outputs from empty belts: total operational
deadlock, invisible to the validator, renderer, and mechanics doc
because all three share the engine convention, and invisible to the
user's in-game pastes because those only ever checked structure and
module slots. Fixed at the artifact boundary in PR #348; the post-fix
factory demonstrably runs (ore→plates→cable flowing, furnaces crafting).
The wires bug (RFC-040) was the same failure class at smaller stakes.
Every open in-game anchor (rfc-inserter-sizing KC5, rfc-build-quality,
rfc-046) queues on manual user pastes that this tool automates.

## Empirical base (all executed, not researched)

From the discovery spike, the two review probes, and the #345 dogfood:

- Anonymous headless download works, **2.0.76 pinned URL verified**
  (`factorio.com/get-download/2.0.76/headless/linux64`); Space Age
  prototypes load without entitlement; runs on WSL2.
- `import_stack` → `build_blueprint{superforced}` → `revive` works on
  our exports (rc 0; 5108/5108 on the #345 fixture). **The paste is
  CENTERED on the build position** — the harness derives the
  layout→world offset from the revived bbox min.
- **Dedicated servers auto-pause with no players**: `on_init` runs but
  the sim never ticks. Ship server settings with `auto_pause: false`
  (+ `autosave_interval: 0`). Live-server pacing is 60 UPS unless
  `game.speed` is raised; at `game.speed = 16` the 5108-entity dogfood
  ran CPU-bound (~60–100 effective UPS on the dev box — scale
  expectations with entity count, not the 149-entity ~640 UPS floor).
- **Boundary kit, measured**: 6 legendary stack inserters saturate one
  S=4 express belt at **179–186/s** (script-refilled chests);
  tier-matched hidden 1×2 loaders (`loader` / `fast-loader` /
  `express-loader` / `turbo-loader`) deliver exact full compression at
  S=1 (express-loader measured 45.0/s) — `loader-1x1` is yellow-speed,
  unusable above 15/s. Loaders are unverified for S>1; stack-inserter
  banks are the stacked-feed mechanism. Drain tier ≥ belt tier or
  backpressure falsifies the run. Every power island needs its own
  energy source: script-maintained `eei.energy` each interval (the
  EEI's own production was unreliable on 2.1;
  `hidden-electric-energy-interface` — 0×0 collision box, placeable AT
  a pole's position — avoids the 2×2 siting problem in dense layouts).
- **Measurement**: `LuaFlowStatistics.get_input_count/get_output_count`
  are exact cumulative integers — difference them at window edges for
  any window size. `get_flow_count` is pre-bucketed and fractional
  (~1% error measured): diagnostics only. Items the boundary kit
  spawns/moves are invisible to production stats (verified) — crafting
  counters stay honest. The target item's DELIVERED rate = script
  count-and-clear on the drain chests (exact, deterministic, no
  overflow). Open question from the dogfood: `get_input_count` on a
  legendary-machine factory read 0 despite visible crafting — possibly
  quality-scoped statistics; Phase 1 must resolve this (the review
  probe's normal-quality counters worked). Drain-counting as primary
  sidesteps it for the target item.
- **Module proxies**: full fulfillment path verified (insert into
  `get_module_inventory()`, destroy proxy, effect live; quality-tagged
  requests carry quality through `item_requests`).
- **Sim-state debug tooling is load-bearing**: the standard dump (belt
  line contents, machine/inserter statuses, in LAYOUT coordinates via
  the derived offset) joined against the layout's segment map found the
  inserter-direction bug within minutes of existing. It is a named
  deliverable, and its JSON is designed to feed a future web-renderer
  overlay (headless has no screenshots; our renderer is the viewer).

## Design

### Shape: offline tool, not a validator check, not (initially) CI

- New workspace crate **`crates/sim-harness/`** (binary `spaghettio-sim`),
  mirroring `mining-cli`. Input: blueprint string + verification
  manifest (or a fixture name that generates both via the engine).
- The headless install resolves at runtime (`SPAGHETTIO_FACTORIO_DIR`;
  `--fetch` downloads the **pinned version — 2.0.76, never `latest`** —
  to a cache dir). Not vendored, not baked into published images (EULA:
  runtime user-fetch only). The pin matches our draftsman data baseline,
  making KC1's drift check zero-by-construction; migrating the pin to
  2.1.x is a separate arc (draftsman tops out at 2.0.77 — there is no
  2.1 extraction path today).
- Runs are minutes-scale: a different verification tier, used like
  STRESSGOLD at close-outs. CI enforcement deferred (revisit at
  close-out with runtime data).

### Engine-side addition: the verification manifest

**`blueprint::export_with_manifest(layout, solver_result, label)`** — it
needs BOTH (rev 1's "SolverResult only" framing was wrong: boundary
geometry is layout-side). The manifest carries:

- target item, rate, config axes (quality/stacking/inserter-capacity);
- **explicit boundary records** — per external input: item, tile,
  direction, belt-or-pipe, belt tier; output exit tile(s); fluid
  surplus exits (`LayoutResult.surplus_exits`). Rev 1 claimed `io_type`
  tags marked these in the artifact — **falsified by both reviewers and
  the dogfood**: `io_type` is the UG entrance/exit marker only, and
  `carries` is not exported, so the artifact alone cannot locate
  boundaries. The engine emits them from lane-planner / output-merger
  knowledge (the dogfood's unfed-belt heuristic worked but misfired on
  UG-shadowed interior belts — the engine's own records are the fix);
- per-item planned rates (from `SolverResult.machines` flows), so the
  report's per-item deltas have a reference;
- validator error/warning counts at export time (report context).

### The scenario (Lua, shipped as a template in the crate)

1. Lab surface (`generate_with_lab_tiles`), all other surfaces deleted;
   `research_all_technologies()` — NOTE this maxes inserter-capacity
   bonuses too, which can mask engine inserter-undersizing; the
   calibration tech state is part of the baseline key.
2. Import, superforced build, revive; **derive the world offset from
   the revived bbox min**; fulfill module proxies.
3. Power: one script-maintained energy source per electric network
   (enumerate pole `electric_network_id`s) + per boundary island.
4. Feed per manifest boundary records: stack-inserter banks (S>1) or
   tier-matched loaders (S=1) from script-refilled chests;
   `infinity-pipe` at fixed fill for fluid inputs.
5. Drain: banks at output exits into script-counted-and-cleared chests;
   `infinity-pipe` voids at every fluid surplus exit (undrained surplus
   dead-ends fill and stall AOP-class fixtures).
6. Warmup **scaled to layout dims** (base + 2×(W+H)×32 ticks), then
   cumulative-counter snapshots at window edges; windows sized so
   expected items ≥ ~300 (rate × window); two consecutive windows must
   agree within tolerance — loop-until-stable with a ceiling, not a
   single retry.
7. Emit report JSON + the standard **sim-state dump** (belt contents,
   machine/inserter statuses, layout coordinates); orchestrator polls
   `script-output/`, kills the server, renders the report.

### The report

Per item: planned rate, measured produced rate (counter delta), measured
delivered rate for the target (drain count), delta %; plus import rc,
ghost/revived/failed counts, network count, unfulfilled proxies, machine
status census, warmup/window parameters, game version + mod list, and a
PASS/WARN/FAIL verdict. The sim-state dump ships alongside for
debugging; a web-renderer overlay consuming it is the designated
follow-on deliverable.

### Determinism and baselines (Phase 3)

Same-machine determinism verified (byte-identical double runs).
Factorio's lockstep architecture is cross-platform deterministic by
design, so blessed measured baselines are **shareable** — keyed on
(pinned game version, mod list, tech state), not per-host.

## Non-goals

- Automated feedback into generation (report-first; humans close the
  loop).
- CI enforcement; UPS tuning beyond save hygiene; parallel fleets.
- Simulating imported community blueprints (resource entities need
  patches — discovery finding; our factories don't place them).
- Gleba spoilage; per-quality flow statistics beyond what Phase 1's
  quality-stats investigation requires.
- 2.1.x pin migration (separate arc, blocked on a 2.1 data-extraction
  path).

## Kill criteria

1. **Pin integrity (Phase 0).** The harness pins 2.0.76 to match
   recipes.json's baseline; Phase 0 runs a `--dump-data` parity
   spot-check (machine speeds, recipe energies/yields for the ladder
   recipes). ANY consumed-fact mismatch = stop and investigate — there
   is no "reconcile in a day" option (re-baselining recipes.json
   invalidates the corpus goldens wholesale).
2. **Calibration (Phase 1), one-sided.** Measured target rate ≥ 0.98 ×
   planned on gear@10/s and EC@10/s at steady state (overshoot is
   expected — placed machines are ceil'd above fractional plans — and
   reported informationally; a two-sided ±2% would spuriously fail 3 of
   6 science packs on ceil headroom alone). If the shortfall is
   harness-boundary artifacts that a redesign doesn't close in a day,
   stop and rethink the kit. If it's a real engine defect — that's the
   tool working (precedent: the inserter-direction bug); file it, fix
   it, measure again. Phase 1 also must resolve the quality-scoped-
   statistics question (legendary-machine counters read 0 in the
   dogfood).
3. **Determinism.** Two identical runs produce identical reports.
   Nondeterminism that survives investigation kills Phase 3 (blessed
   baselines) and demotes the tool to exploratory-only.
4. **Wall-clock.** With dim-scaled warmup + item-floored windows, a
   USP-scale cycle must stay under ~6 min on a dev machine (arithmetic
   at 200–400 UPS: ~2–6 min). Above that, rescope to nightly batch.
5. **Pin-bump fragility.** If TWO consecutive deliberate pin-bump
   attempts require game-version-specific scenario rework, freeze
   support to the current pin and record the constraint.

## Verification plan

- Phase 1 exit: gear + EC calibrated per KC2, determinism double-run,
  report artifacts in the decision log (numbers, not screenshots), and
  the quality-stats question resolved with a probe.
- The #345 dogfood completed with the released tool: measured rates for
  the 120/s (3 dead-end errors) vs 150/s (clean) configs posted to the
  issue — the validator's structural verdicts get their first
  game-side damage quantification.
- Six-pack sweep (Phase 2) cross-checked against `docs/status.md`'s
  gauntlet table; each retired in-game anchor decision-logged in its
  owning RFC.

## Phasing

| Phase | Deliverable | Gate |
|-------|-------------|------|
| 0 | Pinned 2.0.76 fetch helper + `--dump-data` parity spot-check + manifest export (`export_with_manifest`) | KC1 |
| 1 | Harness MVP: scenario template + boundary kit + report + sim-state dump; gear & EC calibrated; quality-stats question resolved | KC2, KC3 |
| 2 | Fixture sweep (six-pack + the #345 pair), anchor retirements, USP-scale timing | KC4 |
| 3 | `--bless`/`--check` shareable measured baselines (version+mods+tech key) | KC3 |
| 4 (follow-on) | Web-renderer sim-state overlay | — |

## Decision log

- *2026-07-22 — drafted after the §8.3 discovery spike
  (`headless-verification-discovery.md`); shape decision: standalone
  offline engineer tool (not validator, not CI) with report-first
  feedback — automated generator tuning explicitly out of scope. PR
  #347.*
- *2026-07-22 — rev 2. Dual adversarial review (architecture +
  probe-driven sim-mechanics, both ACCEPT-WITH-CHANGES) converged with
  the #345 dogfood on the same BLOCKER: rev 1's `io_type`
  boundary-location mechanism is false (the field is the UG-half
  marker; `carries` is unexported) — replaced with explicit manifest
  boundary records plumbed from the LayoutResult
  (`export_with_manifest` takes the layout, not just the SolverResult).
  Other findings folded: KC2 one-sided at ≥0.98× (ceil headroom fails
  two-sided ±2% on 3 of 6 packs spuriously); cumulative-counter
  differencing replaces `get_flow_count` (fractional, pre-bucketed —
  ~1% measured error); item-floored windows (quantization at 1/s
  rates); **pin = 2.0.76** (draftsman has no 2.1 path; anonymous pinned
  download verified) with KC1 reduced to a dump-data parity check;
  fluid input/surplus boundaries specified (infinity-pipe feeds + voids
  at `surplus_exits`); `auto_pause=false` shipped settings; dim-scaled
  warmup with loop-until-stable windows; tier-matched loaders for S=1 +
  stack-inserter banks for S>1 (measured 45.0/s and 179–186/s);
  script-maintained energy + hidden-EEI siting; shareable determinism
  baselines keyed on version+mods+tech; KC5 reworded to pin-bump
  attempts; de-risk claims scoped honestly (import path executed;
  boundary kit now executed too, via the dogfood). **Dogfood outcome
  recorded**: the prototype found the inserter-direction export bug —
  every previously exported factory deadlocked in-game — fixed in PR
  #348 with operational proof, and produced the sim-state debug tooling
  now specced as a deliverable. The measured #345 numbers land when the
  dogfood reruns on the fixed exporter.*
- *2026-07-22 — Phase 0/1 crate landed (`crates/sim-harness`, branch
  `rfc050-sim-harness-crate`): `fetch`/`run`/`check-data` subcommands,
  the scenario templating generalized from the calibrated south-only
  prototype to all four cardinal directions via outward/lateral vector
  rotation (every formula checked tile-by-tile against the literal
  prototype numbers), and dim-scaled warmup + loop-until-stable
  measurement windows. **KC1 exercised live**: `check-data` against a
  freshly fetched real 2.0.76 install passed clean, but only after
  fixing two dump-parsing traps the tool itself caught by running the
  real dump instead of reasoning about the schema — (1) a bare
  first-name-match prototype scan non-deterministically resolved to a
  same-named `item` prototype instead of the intended `recipe`/machine
  prototype for every one of the 4 probe recipes (fixed: require a
  disambiguating field per lookup — `crafting_speed` for machines,
  `ingredients` for recipes); (2) the dump OMITS `energy_required`
  entirely when it equals Factorio's 0.5s default (iron-gear-wheel,
  electronic-circuit, and copper-cable all hit this — only iron-plate's
  non-default 3.2s energy dumps explicitly), which a naive "field
  missing = mismatch" read would have reported as 3 false-positive KC1
  trips. **KC2 gear@10/s PASS on a live server**: freshly generated
  gear10 bp+manifest (`export_with_manifest` on
  `rfc050-phase0-manifest`), measured 10.13/s delivered vs 10.0/s
  planned (+1.3%, well inside the 0.98× floor), 428/428 ghosts revived,
  36/36 machines `working`, loop-until-stable converged — matching the
  RFC's own gear10 PASS precedent. **EC@10/s (AM1, yellow, from ore)
  found and fixed a real power-siting bug**: the factory-network EEI
  placement (ported verbatim from `gen_harness_scenario.py`'s
  west-of-min-x-pole `can_place_entity` scan) placed successfully by
  its own collision check but left the entity out of wire reach on the
  bigger, denser 84×90/1110-entity fixture — 60/65 machines came back
  `no_power`, 0 measured. Root-caused with an isolated probe (single
  pole + machine: hidden-EEI-at-pole-position worked correctly) and a
  targeted probe against the real EC10 factory (confirmed one true
  74-pole electric network, `no_power` only when placement used the
  west-scan). Fixed per the RFC's own empirical-base note — swapped the
  factory-network scan for a `hidden-electric-energy-interface` placed
  directly AT (0×0 collision, always placeable) a representative pole's
  own position, guaranteeing wire-reach adjacency regardless of layout
  density. Confirmed no regression: gear10 re-measured identically
  (10.13/s, PASS) on the fixed code. **EC10 still measured -20% short
  after the power fix** (8.0/s vs 10.0/s, `converged=true`, 4 machines
  `full_output`, a jammed 144-item belt tile visible in the sim-state
  dump) — open per KC2's own fork ("harness-boundary artifact" vs "real
  engine defect"); not resolved in this session, left for whoever closes
  out Phase 1's calibration gate. Quality-scoped-statistics question
  (KC2's other Phase 1 exit item) not investigated this session — no
  quality>normal fixture was run.*
- *2026-07-22 — **Phase 1 calibration gate (KC2) PASSED; Phase-1 exit
  items resolved.** Lead reconciliation runs with the released tool on
  the merged Phase-0 manifest: gear@10/s **PASS** (10.00/s produced
  +0.0%, 10.13/s delivered, 36/36 working, converged) — now verified on
  two independent implementations (python prototype + Rust crate).
  EC@10/s **FAIL −50%** (5.00/s), matching the prototype's dogfood
  number exactly; the crate agent's interim −20% came from a
  differently-configured fixture (1110 vs 805 entities). Per-item
  attribution seals the KC2 fork as REAL ENGINE DEFECT: copper-plate
  produces at exactly plan (feeds proven good) while iron-plate −43.5%
  and copper-cable −48% are the starved stages — #352 stands, and the
  constraint web is wider than the four warned EC machines. The
  quality-scoped-statistics question is RESOLVED as a non-issue: the
  r150 legendary-machine run reported production stats correctly
  (18.8/s); the earlier zeros were genuine zero production under the
  pre-#348/#350 broken exports. Remaining phases: 2 (sweeps/anchor
  retirements), 3 (bless/check), 4 (web sim-state overlay — pulled
  forward at user request, 2026-07-22).*
- *2026-07-22 — **IMPLEMENTATION COMPLETE; RFC closed out.** Phases 0
  (#353 manifest), 1 (#354 crate — KC2 calibration gate passed), 3
  (#355 bless/check), 4 (#356 web sim overlay, pulled forward at user
  request) all merged same-day. Phase 2 sweep, solid fixtures: gear10
  PASS +0.0%; ec10 FAIL −50% (#352); automation-science 1.00/s produced
  (delivered −4%, quantization attribution open); logistic −40% and
  military −48% — both VALIDATOR-CLEAN (#357), the confidence-chain
  critique proven at gauntlet scale. Five baselines blessed into
  `crates/sim-harness/baselines/` (FAILs deliberately included as
  honest floors). Open items carried forward: fluid-path calibration
  (blocks the chemical/production/utility sweep), non-south boundary
  calibration, automation delivered-quantization attribution, #345
  re-measurement through the released tool, USP-scale timing (KC4
  formally unmeasured at USP size). Arc summary: the harness found two
  artifact bugs that made every prior export inert (#348, #350), one
  half-rate warning class (#352), and one clean-but-failing class
  (#357) — four finding classes in its first two days.*

