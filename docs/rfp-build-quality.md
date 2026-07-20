# RFP: Build quality — quality-tier entities in the layout engine

Status: ACCEPTED (2026-07-20) — v2 after a three-lens adversarial
review (game mechanics / codebase integration / design rigor). v1 → v2
changes and the acceptance are in the decision log. Follow-up tracked:
[#310 pole-band thinning](https://github.com/storkme/spaghettio/issues/310).

## Summary

Add a user-facing **build quality** parameter (normal / uncommon / rare /
epic / legendary, default normal) that makes the generator plan and
place quality entities. Quality machines craft +30% per level faster
(fewer machines per row), quality inserters swing +30% per level faster
(higher per-side ceilings in the sizing ladder and the validator), and
quality poles cover more ground (+1 supply radius, +2 wire reach per
level). The exported blueprint stamps `quality` on **functional**
entities — machines, inserters, poles — while logistics (belts, UGs,
splitters, pipes) stay normal-quality, since vanilla quality gives them
no function and quality logistics are only obtainable via recycler
upcycling (see "Stamping policy"). Belt tier remains the independent
user constraint it is today. This RFP is **build quality only**: making
quality *items* (quality modules' quality% effect, recycler upcycling
loops, probabilistic outputs) is explicitly out of scope and stays a
separate future RFP (already flagged as a Phase-3 concern in
`rfp-fulgora-scrap.md`).

## Motivation

Not a failing case today — a capability gap, and per the template
that's a yellow flag, so stating it plainly: no validator error
reproduces this, and the motivating case is **n=1** (one very-lategame
save with legendary everything, user request 2026-07-20). The feature
nonetheless ships as a public dropdown on the GitHub Pages deployment,
so the design must be safe for users who never touch it: the default is
Normal and kill criterion 2 makes default-quality behavior preservation
a hard gate. Honest context: the 2026-07 strategy review ranked solver
byproducts and validator calibration above this; this RFP is
opportunistic user-driven work, phased so it can stop after any phase
with standalone value banked.

Target scenario: **60 electronic-circuit/s from ore with legendary AM3
+ legendary electric furnaces + legendary inserters**. Today the
generator would emit ~305 normal machines with normal inserter
ceilings; the legendary build is ~122 machines (legendary AM3 = 3.125
speed → 6.25 EC/s per machine → 10 EC assemblers) with single-side
feeds.

One piece *is* reproducible today: `blueprint_parser.rs` silently
drops the `quality` field from community blueprint strings
(`BpEntity`, `blueprint_parser.rs:162`, has no `quality` field), so
the analyzer misreports any quality-built community blueprint. That
fix is standalone and lands in Phase 0 regardless of the rest.

## Game rules (the data this encodes)

Quality levels: uncommon = 1, rare = 2, epic = 3, **legendary = 5**
(level 4 is reserved/skipped in vanilla — FFF-375). All figures below
were independently verified against wiki.factorio.com and
lua-api.factorio.com during the 2026-07-20 adversarial review; the
in-game tooltip check (kill criterion 1) remains the final gate.

| What | Scaling | Verified examples |
|---|---|---|
| Crafting machine speed | ×(1 + 0.3·level) | AM3 1.25 → 3.125; electric furnace 2 → 5 |
| Inserter rotation speed | ×(1 + 0.3·level), tick-quantized | standard inserter 302/393/484/575/756 °/s across the five tiers (note 484 ≠ 302×1.6=483.2 — real quantization: `ticks_per_cycle = floor(0.5/rot)*2+2`) |
| Electric pole supply radius | +1 tile per level | medium 3.5 → 8.5 (7×7 → 17×17); substation 9 → 14 (18×18 → 28×28) |
| Electric pole wire reach | +2 tiles per level | medium 9 → 19; substation 18 → 28 |
| Belt speed, UG belt reach, UG pipe reach, splitters | **unchanged** (health only) | quality UG reach is a mod/forum wish, not vanilla |
| Entity footprints, inserter reach, module slots | **unchanged** | placer geometry untouched |

**Pole margin asymmetry (review finding, load-bearing):** medium poles
keep a 2-tile margin between supply diameter and wire reach at every
tier (7<9 … 17<19), so coverage-spaced medium poles can never strand.
**Substations have zero margin at every tier** — supply diameter equals
wire reach exactly (18=18, 20=20, 22=22, 24=24, 28=28). This is
quality-invariant (it's already true at Normal), but it means "sparser
placement cannot strand a pole" is *provable for medium poles only*;
substation spacing must never be derived as "up to supply diameter"
with connectivity assumed. The Phase 3a-ii substation-band code is the
live code path here — see "Machine-count collapse side-effects".

Blueprint format: entity-level `quality :: string?` field, sibling to
`name`/`position`/`direction`, per the authoritative
[lua-api.factorio.com BlueprintEntity concept](https://lua-api.factorio.com/latest/concepts/BlueprintEntity.html).
(Do **not** consult wiki.factorio.com/Blueprint_string_format for this
— that page predates 2.0 and has no mention of quality.) The repo's
exported version constant `562949955518464` decodes to 2.0.32 —
quality-capable. In-game, ghosts and logistics requests demand
**exact** quality matches (no substitution in either direction), which
is what makes the stamping policy below a real decision rather than a
cosmetic one.

Non-effects we deliberately don't model: quality raises inserter energy
draw and entity health; we model neither power load nor health.
Modeling note: we apply the rotation multiplier linearly to the
existing throughput constants; tick quantization makes linear a few
percent optimistic at some tiers — the same fidelity the existing I8
constants already accept.

## Design

### Core type

`QualityTier` in `common.rs`: `{ Normal (default), Uncommon, Rare,
Epic, Legendary }`, with `level() -> u8` (0/1/2/3/5), `multiplier()
-> f64` (`1.0 + 0.3 * level`), and `name() -> &str`
(`"normal"`…`"legendary"`, the blueprint-JSON strings). The multiplier
is keyed off `level()`, not tier ordinal — the skipped level 4 is
exactly the kind of off-by-one the differential tests target.

### Where it multiplies — all existing single sources of truth

1. **Solver (machine counts).** Both count formulas use raw
   `crafting_speed` (`solver.rs:342`, `netflow.rs:793`; the speed
   comes from the free function `recipe_db::get_crafting_speed`,
   `recipe_db.rs:169` — a module-level function over the global db,
   not a `RecipeDb` method). Add one shared helper
   `effective_crafting_speed(machine, quality)` used by both solvers,
   so the legacy tree walk (the recipe-selection oracle) and netflow
   cannot diverge. `get_crafting_speed` itself stays unchanged.
   **Rounding hazard (review finding):** machine counts get ceil'd
   downstream; `×1.0` is bit-exact in IEEE 754, but the helper must be
   the *same code path* at Normal (no `if quality == Normal` fork), and
   a unit test sweeps rates near whole-machine boundaries asserting
   `effective_crafting_speed(m, Normal) == get_crafting_speed(m)`
   bit-for-bit, plus per-tier count checks at boundary-adjacent rates.
   Machine palette selection (`machine_for_recipe_with_palette`) is
   untouched — quality is orthogonal to which machine tier the user
   picked.
2. **Inserter ladder + validator.** `inserter_throughput(name)`
   (`common.rs:346`) gains a quality parameter. The sizing ladder
   (`bus/inserter_ladder.rs`) and `check_inserter_throughput` both
   read this one table, so they cannot diverge structurally; the
   existing `constants_identity` test (`inserter_ladder.rs:248`) pins
   the table's values against drift (it is a drift-pin on the shared
   table, not a fix-vs-check cross-check — there is only one table)
   and gets extended across all five tiers. Ladder semantics
   (cheapest-sufficient, count-ladder, budgets, shortfall) are
   unchanged; only the per-tier ceilings scale.
3. **Power geometry.** `supply_area_distance(name)` (`common.rs:107`)
   and the wire-reach constants in `validate/power.rs:20-25` become
   quality-aware functions. `place_poles` genuinely derives its
   spacing from `supply_area_distance` (`layout.rs:1029`), so medium
   pole cadence scales automatically. The Phase 3a-ii substation bands
   (`compute_substation_bands`, `layout.rs:321`) do **not** read the
   radius — they use `SUBSTATION_BAND_TILES = 2`, a row-count constant
   whose sizing rationale was tuned against today's reach 9. Higher
   quality reach (10/11/12/14) only adds slack, but this constant's
   adequacy must be re-verified per tier, not assumed, and the
   zero-margin property above means substation spacing logic may never
   assume wire reach exceeds supply diameter. No new radius literals
   anywhere — everything flows through the two shared functions.
4. **Export + parse.** `PlacedEntity` gains `quality:
   Option<QualityTier>` (`models.rs:136`; `None` = normal, not
   serialized). `BlueprintEntity` (`blueprint.rs:32`) gains a
   `quality` string field, skipped when normal. `BpEntity` learns to
   parse it (the Phase 0 fidelity fix).
5. **Plumbing.** `LayoutOptions.quality` plus a solver-side param —
   quality affects machine counts, so unlike `max_inserter_tier` it
   must reach the wasm solve entry points: **both** `solve` and
   `solve_with_palette` (`wasm-bindings/src/lib.rs:75,91` — flat
   positional params today, so two signatures change, not one).
   Otherwise the plumbing mirrors `max_inserter_tier`'s six hops:
   `LayoutOptions` (`layout.rs:86`) → wasm string→enum map (unknown →
   Normal, matching `lib.rs:51`'s pattern) → `web/src/engine.ts` →
   `FormState` + URL codec (`web/src/state.ts`, short code `q=` with
   `u/r/e/l`, absent = normal — verified free, and absent-key decode
   keeps old URLs working) → sidebar dropdown. Renderer: small quality
   badge/tint on entities (`web/src/renderer/entities.ts`) —
   cosmetic, user validates visually per standing feedback.

**Phase 2 design note (2026-07-20, pre-build):** the validators read
quality from **each entity** (`entity.quality.unwrap_or_default()`),
not from a global param. Placement (ladder, placer, `place_poles`)
stamps `PlacedEntity.quality` on the functional entities it creates
from `LayoutOptions.quality`; export then just serializes what's
stamped, and the quality-aware checks (inserter throughput, pole
supply/wire) rate each entity by its own tier. This keeps a single
ground truth per entity, makes mixed-quality layouts (Phase 3
per-class overrides) free at the validator layer, and — because the
parser now populates the same field — means validation of *imported*
community quality blueprints works with zero extra plumbing.

### Pole placement: what scales for free vs. what a redesign could exploit

Scope boundary, made explicit (user question, 2026-07-20): this RFP
scales pole *parameters*, not pole-placement *design*.

Free with the parameter change: in-band pole spacing stretches
automatically (cadence derives from `supply_area_distance`), and the
reactive substation fallback triggers less often because each pole
covers more inserters.

Not free, and **deliberately not in this RFP**: exploiting the larger
radii structurally. A legendary medium pole reaches ±8 tiles
vertically — enough to cover several machine rows from a single band —
so `pole_candidate_ys`'s two-bands-per-row scheme (`common.rs:168`)
could thin to one band per N rows, removing whole pole rows from the
layout. That changes band *selection*, interacts with the Phase 3a-ii
fallback's assumptions, and deserves its own small design pass with
before/after entity counts as evidence. Tracked in
[#310](https://github.com/storkme/spaghettio/issues/310) — the user
intends to pick it up **immediately after this RFP lands**. (If Phase
2's browser eyeball shows legendary pole placement looking absurd —
double bands of nearly-idle poles — that's the trigger to pull it
forward.)

### Stamping policy (trade-off, decided — reversed from v1)

The export stamps the chosen quality on **functional entities only**:
machines, inserters, poles. Belts, undergrounds, splitters, and pipes
stay normal.

v1 chose uniform stamping ("the knob means my palette is this
quality"). Two review findings reversed it: (a) quality logistics are
functionally inert (health only) but **economically severe** — ghosts
demand exact quality, and quality belts are not craftable, only
recycler-upcycled, so a uniform legendary bus would demand thousands
of the hardest-to-farm items in the game for zero function; (b) the
motivating request itself lists "assemblers, furnaces, inserters" —
functional entities. The entity-class predicate at export is trivial
(same shape as `needs_electricity`).

Revisit trigger (named, per review): if a user asks for
uniform stamping (e.g. for consistency or spare-part logistics), add
it as an export toggle — it is a strict superset of the predicate and
needs no engine changes. Until someone asks, functional-only is the
default and only mode.

### Machine-count collapse side-effects (new in v2)

A 2.5× speed multiplier crushes machine counts (~305 → ~122 in the
target scenario), which moves layouts into regimes the engine
special-cases:

- **Balancer family shapes** are keyed off producer machine counts
  (`balancer.rs` / `balancer_library.rs`). Collapsed counts land on
  different `(n, m)` shapes — including the already-open library gaps
  ([#135 oversized](https://github.com/storkme/spaghettio/issues/135),
  [#136 missing coprime shapes](https://github.com/storkme/spaghettio/issues/136)).
  Quality doesn't create these gaps but will hit them from new angles;
  the differential fixture pair (Verification) exists partly to
  surface which shapes the legendary variant demands.
- **1–2-machine rows**: `can_lane_split` requires `count >= 2`
  (`placer.rs`) and `inline_bridge_anchor` requires `machine_count >=
  2` (`templates.rs`, "only 1 machine — no room to bridge"). Legendary
  pushes more recipes into this less-capable small-N regime. The
  fast e2e legendary fixture (Verification) is chosen to contain at
  least one single-machine row on purpose.
- **Pole/substation triggers**: fewer machines → shorter rows → fewer
  poles → the reactive Phase 3a-ii substation fallback triggers under
  different conditions. Combined with the zero-margin property and the
  band-constant note above, substation behavior at quality is
  *verified per tier*, not extrapolated from the legendary-medium-pole
  inequality (which is now a unit test, not prose).

### Pre-existing risk the headline fixture is aimed at (new in v2)

The integration review found that the **final-product output path**
does not share the bus machinery's overflow handling. Intermediate
lanes split when `rate > lane_cap`
(`lane_planner.rs::split_overflowing_lanes`, proven at 60/s-on-red in
`stress_electronic_circuit_60s_red_from_ore`). But
`output_merger.rs::merge_output_rows` cascades all producing rows down
to **exactly one** surviving belt, sized by `belt_entity_for_rate`,
which falls back to the best available tier without ever signalling
insufficiency. The committed golden for that very stress twin reports
**0 errors / 0 warnings** on a layout whose single final belt is
logically carrying 60/s against a 30/s physical cap — i.e. either the
lane-rate walker doesn't cover merger-cascade tiles (validator blind
spot) or there is an unaccounted mechanism making it fine. Unresolved
either way.

This predates and is independent of quality — but the proposed 60 EC/s
legendary fixture (60/s > blue belt's 45/s) is *exactly* this shape,
so its "0 errors" would otherwise inherit the blind spot — the precise
trap CLAUDE.md's verification protocol warns about. Hence the Phase 2
entry gate and kill criterion 5 below.

### Pre-existing wrinkle, treated as out of scope

The ladder's fast mover is the 1.x name `stack-inserter` (12/s in
`inserter_throughput`), while `blueprint.rs` also knows 2.0's
`bulk-inserter` (2.4/s). In 2.0 the name `stack-inserter` denotes the
*Space Age stacking inserter* (the 1.x entity was renamed
`bulk-inserter` in 2.0.7 — verified). Quality treats entity names as
opaque and multiplies whatever base number the table returns, so this
RFP neither fixes nor worsens the naming split — but the 2.0-import
consequences compound with quality stamping, so the naming
reconciliation is tracked as
[#313](https://github.com/storkme/spaghettio/issues/313) and must be
resolved before the in-game import anchor runs.

### What this deliberately does not do

- No quality **item** production: `module_effect` keeps ignoring the
  quality% stat of quality modules; no recycler loops; no
  probabilistic outputs.
- No per-class quality (legendary machines + normal poles) — Phase 3,
  deferred.
- No mixed-quality optimization ("minimum quality per entity to hit
  rate") — uniform tier across functional entities only.
- No belt-speed or UG-reach changes — vanilla quality doesn't grant
  them, so the router and belt validators are untouched.
- No fixing of the output-merger capacity gap itself — it is audited
  and fenced (kill 5), and fixed under its own issue if confirmed.

## Kill criteria

1. **Data gate (Phase 0).** Verify the scaling table in-game against
   the user's save: machine tooltip speeds, inserter tooltip speeds,
   pole supply/wire tooltips — at Normal and Legendary minimum, plus
   at least one mid-tier for poles (the +1/+2-per-level rules, and the
   substation supply==wire equality, must hold at every tier, not just
   the endpoints). Any tooltip number ≠ table number → halt engine
   work and re-derive the data table first.
2. **Identity at Normal (Phase 1+2 gate, mechanism-specific).** The
   quality-aware helpers must be **on the default path** (called with
   `Normal`, no branch bypassing them — a vacuous identity where old
   call sites never invoke the new code proves nothing). Gate, in
   order: (a) unit: `effective_crafting_speed(m, Normal)` and
   quality-aware `inserter_throughput`/`supply_area_distance` at
   Normal are bit-identical (`==` on f64) to their pre-RFP values,
   swept across all machines/inserters/poles and across rates adjacent
   to whole-machine ceil boundaries; (b) full e2e suite green with
   unchanged assertion/scoreboard counts; (c)
   `SPAGHETTIO_STRESS_GOLDEN=check` clean, run on the same host/cache
   that blessed the current goldens (this is an opt-in, host-relative
   check per `goldens/stress/README.md` — it is *named here as a
   required step* precisely because nothing else forces it). If (a–c)
   cannot all pass without forking helpers into quality and
   non-quality variants, the parameterization is wrong — stop and
   redesign.
3. **Ladder-model tripwire (Phase 2, with an explicit read-off
   protocol).** Predicted: in the 60 EC/s legendary fixture every side
   fits in 1 column (tightest: EC cable side 18.75/s vs legendary
   stack ~30/s). The existing `InserterSideCapped` taxonomy
   (`capped_limit()`: tier-cap / column-contest / geometry) cannot
   distinguish "quality ceiling insufficient" — so the protocol is
   manual and specified now: if the fixture emits *any*
   inserter-throughput warning, decode the snapshot and, for each
   warned side, compare its demanded rate against (quality-scaled
   1-column ceiling) and the census budget table for its position. A
   side whose demand ≤ its position's quality-scaled ceiling yet still
   warns, or any side needing >2 columns, means the linear ×2.5
   throughput model is materially wrong — stop and measure real
   quality inserter rates in-game before proceeding.
4. **Scope fence.** If implementation ever needs per-recipe or
   per-entity quality decisions to be *correct* (not merely nicer),
   that's the quality-production boundary being crossed — stop and
   split into the future RFP rather than growing this one.
5. **Output-path honesty gate (Phase 2 entry).** Before the legendary
   fixture can count as evidence, decode the existing
   `stress_electronic_circuit_60s_red_from_ore` snapshot and determine
   whether its final output belt carries rate > capacity while
   `lane-throughput` stays silent. If the blind spot is confirmed and
   fixing it is out of this RFP's scope: file it as its own issue, and
   the legendary fixture's rate drops to ≤45/s (one blue belt) so its
   "0 errors" is meaningful — the 60/s headline then re-lands only
   after the merger issue closes. This RFP does not ship a fixture
   whose green status is known to be unmeasurable.

## Verification plan

Per the CLAUDE.md layout-change protocol:

- **Unit**: `QualityTier` level/multiplier table across all five tiers
  (targets the skipped level 4); the `constants_identity` drift-pin
  extended per tier; identity-at-Normal bit-equality sweeps (kill 2a);
  solver counts at boundary-adjacent rates per tier; pole-margin
  invariants for every pole type × tier — asserting medium's strict
  2-tile margin and *documenting* substation's exact equality
  (`supply_diameter == wire_reach`, equality allowed, strictness
  asserted false so a data-table typo can't hide).
- **Differential e2e pair** (replaces v1's single fresh fixture): the
  same recipe/shape at Normal vs Legendary asserting the
  machine-count ratio and validation deltas, so quality is isolated as
  the only variable. Small fast pair in plain e2e — EC at a rate sized
  so the legendary variant contains a **single-machine row** (small-N
  regime on purpose); big pair in the **stress corpus** (not plain
  e2e — the normal-quality twin runs ~9190 entities with a 10-min
  timeout; the suite's time budget is already a tracked backlog):
  legendary sibling of `stress_electronic_circuit_60s_red_from_ore`
  on blue belts, rate per kill criterion 5's outcome (45/s until the
  merger issue closes, then 60/s).
- **Browser eyeball**: dev server with `&q=l` — visibly shorter rows,
  visibly sparser poles, quality badges; validation markers clean;
  compare side-by-side with the same URL minus `q`.
- **Round-trip**: export → `blueprint_parser` → quality preserved
  (unit test); parser exercised against a **synthetic** quality
  blueprint JSON constructed in the test (not blocked on finding a
  community artifact), plus any real community string if available.
- **In-game import anchor** (user-run, same shape as
  `rfp-inserter-sizing` kill criterion 5): import the legendary
  fixture string into the lategame save; entities arrive at legendary
  (functional entities only — belts arrive normal, by design);
  spot-check a machine hits its planned rate. Held open in the
  decision log until run. Blocked behind the stack/bulk naming
  follow-up (see wrinkle above).

## Phasing

- **Phase 0 — data, fidelity, audits.** `QualityTier` type + scaling
  table; in-game verification (kill 1); parser reads `quality`
  (standalone fix, landable alone); output-merger audit (kill 5's
  snapshot decode — read-only investigation, files its own issue if
  the blind spot is real).
- **Phase 1 — solver.** `effective_crafting_speed` in both solvers;
  quality plumbed through wasm `solve` **and** `solve_with_palette`.
  **Guard rail (explicit, not inferred): Phase 1 adds no URL codec
  entry and no UI surface.** The deployed app cannot set
  quality ≠ Normal in this state — that is *why* the intermediate
  state (shrunken machine counts, normal-tier inserter ceilings,
  honest warnings) is unshippable-yet-safe, and a patch that exposes
  the param early violates the phase.
- **Phase 2 — layout, validation, export, UI.** Entry gate: kill 5
  resolved. Quality-aware inserter ceilings, pole geometry (per-tier
  substation verification), functional-only export stamping,
  `LayoutOptions` plumbing, URL codec + sidebar dropdown, renderer
  badge. Differential fixture pairs + identity gate (kill 2) land
  here.
- **Phase 3 — deferred.** Per-class quality overrides; uniform-
  stamping toggle if its trigger fires; quality-aware pole-band
  thinning ([#310](https://github.com/storkme/spaghettio/issues/310),
  see "Pole placement" above — user picks this up immediately after
  this RFP; pulled forward if Phase 2's eyeball shows redundant pole
  bands).
- **Future, separate RFP.** Quality item production (modules,
  recyclers, probabilistic solver).

## Decision log

- *2026-07-20 — v1 drafted from the brainstorm session (build-quality
  scope confirmed by user; quality-production explicitly out).*
- *2026-07-20 — three-lens adversarial review (mechanics /
  integration / design). Mechanics: all 10 game-rules claims
  confirmed to the digit; citation corrected to lua-api
  BlueprintEntity (wiki blueprint page has no quality field); found
  the substation zero-margin property. Integration: all 12 file:line
  citations confirmed; found the output-merger single-belt capacity
  gap and its suspiciously-clean 60/s golden (now kill 5); flagged
  `solve_with_palette` as second wasm surface and ceil-boundary
  rounding as the concrete identity-at-Normal hazard. Design: killed
  v1's byte-identical wording (no mechanism ran it), killed kill 3's
  unreadable "attributable to" clause (taxonomy can't express it —
  replaced with a manual protocol), surfaced balancer-shape (#135/
  #136) and small-N template regimes, re-paired the fixture plan as
  Normal-vs-Legendary differentials anchored on the existing 60/s
  stress twin, and named Phase 1's no-UI guard rail. Stamping policy
  reversed to functional-only on combined evidence (exact-quality
  ghosts + upcycle-only logistics economics + the request itself
  listing functional entities). Review's null-hypothesis steelman
  (five-subsystem change on an n=1 anecdote, ranked below solver
  byproducts/validator calibration by the 2026-07 strategy review)
  recorded in Motivation; proceed/park is the user's call.*
- *2026-07-20 — **accepted** by user. Functional-only stamping
  approved. Pole-band-thinning deferral approved on condition of
  tracking — filed as
  [#310](https://github.com/storkme/spaghettio/issues/310), user
  returning to it immediately after this RFP. Status → ACCEPTED;
  Phase 0 work starting in a worktree (main carries unrelated WIP).*
- *2026-07-20 — kill criterion 5 resolved: **blind spot confirmed**.
  Snapshot decode of `stress_electronic_circuit_60s_red_from_ore`
  (`e3fcfc8`): the merger cascade collapses to one fast belt stamped
  `rate=60.0` (287 tiles, 2× the 30/s cap) with `validation.issues:
  []` — the lane-rate walker never visits merger-segment tiles. Filed
  as [#311](https://github.com/storkme/spaghettio/issues/311) (fix
  validator first, then merger). Per kill 5: the differential
  legendary fixture caps at ≤45/s (one blue belt) until #311 closes;
  the 60 EC/s headline re-lands afterwards.*
- *2026-07-20 — kill criterion 1 verified in-game by the user against
  the legendary save ("checks out": AM3 3.125, furnace 5, medium pole
  17×17/19, substation 28×28/28 + mid-tier). Phase 1 landed:
  `effective_crafting_speed` choke point in `recipe_db`, quality rides
  `NetflowOptions` (both net-flow modes), new
  `solve_with_palette_exclusions_and_quality` entry, wasm `solve` +
  `solve_with_palette` accept optional quality (unknown → Normal).
  Kill 2a evidence: bit-identity boundary sweep + per-machine
  bit-equality tests green — **machine-speed third only**; the
  inserter-throughput and pole-geometry thirds of the 2a sweep land
  with their helpers in Phase 2. Per-tier counts match the RFP's hand
  math (legendary EC@60/s → 9.6). **One deviation from the design text**:
  the legacy tree walk stays quality-blind rather than sharing the
  helper — recipe selection is quality-invariant (JSON-first / cost
  table, never speed) and the walk's counts are documented
  oracle-only, so threading quality through it would churn signatures
  with zero effect on any output; documented at the walk's doc
  comment. Phase 1 guard rail held: no URL codec entry, no UI
  surface.*
- *2026-07-20 — Phase 2 core landed. Quality-aware `inserter_throughput`
  + `supply_area_distance` (+1 radius/level) + `wire_reach` (+2/level,
  per-entity); ladder/`capped_limit`/`contest_favors_far` take the
  planning tier; `place_poles` cadence and the substation set-cover
  bounds derive from the shared functions (Normal ⇒ the original 8/9
  constants, bit-identical); functional-only **stamp pass** at the end
  of `layout_pass` (one audit point, per-class-ready); validators rate
  each entity by its own `entity.quality`; export emits `quality`;
  six-hop UI plumbing (`q=` codec, sidebar dropdown, renderer badge).
  Kill 2a now covers all three thirds (bit-identity sweeps for speed,
  inserter, pole); kill 2b full suite 783/783 green. **Two findings
  while landing the differential fixture**: (1) at EC@6/s legendary on
  yellow, every decomposition candidate fails on the pre-existing
  lane-planner "consumer-clamped fan-in / multi-stage balancer not
  wired" refusal — machine-count collapse concentrates consumer trunks,
  so quality hits this wall far earlier than normal builds do (fixture
  retuned to 4/s on red; the wall predates quality — filed as
  [#312](https://github.com/storkme/spaghettio/issues/312)); (2) the
  all-candidates-failed error discarded every
  candidate's reason — `CandidateRun` now carries the error and the
  terminal message names each candidate's failure (observability fix,
  found because of (1)). Differential fixture
  `quality_differential_ec_normal_vs_legendary` green: exact per-tier
  counts, per-entity stamping asserted entity-by-entity, export→parse
  quality round-trip.*
- *2026-07-20 — Phase 2 adversarial review (code + contract lenses)
  and fixes. **Code review found one real bug**:
  `repair_pole_connectivity` still hardcoded the Normal-tier medium
  wire reach (9), silently inserting needless bridge poles at higher
  tiers (and risking an unrepairable real gap >9 within true reach) —
  fixed by unifying the wire-reach table into
  `common::pole_wire_reach` (same single-source shape 3a-i used for
  supply radii; validator delegates) and threading quality through the
  repair; regression test
  `repair_pole_connectivity_uses_quality_wire_reach` forces the repair
  path at both tiers (Normal must bridge a 12-tile band gap, Legendary
  must not, and the sparser legendary net must pass the validator's
  own connectivity walk). Everything else: mechanical-edit integrity
  verified across all ~40 threaded call sites; export lifetime/guard
  logic confirmed (the Normal-filter is live for parsed-blueprint
  re-export, not dead code); `q=`/`it=` codec letter overlap confirmed
  collision-free; misindented regex insertions folded. **Contract
  review: all functional claims verified under re-execution**; its
  five process fixes landed here: kill 2c
  (`SPAGHETTIO_STRESS_GOLDEN=check`) was run and is now RECORDED —
  clean twice (the 52-test full-suite form and the reviewer's
  independent 9/9 stress-golden form, against the shared host cache
  that predates this RFP, so it is a legitimate same-host identity
  check, not an empty-cache pass); browser eyeball: smoke-level pass
  performed via scripted browser (the `q=l` URL loads, Build-quality
  dropdown reads legendary, canvas renders, solver reports 92
  machines = the hand-computed legendary count) — the FULL visual
  pass stays with the user per standing feedback, held open below;
  stack/bulk naming filed as
  [#313](https://github.com/storkme/spaghettio/issues/313) (blocks
  the in-game anchor);
  [#312](https://github.com/storkme/spaghettio/issues/312) now linked
  from the Phase 2 entry; the stale Phase-1 guard-rail comment in
  `wasm-bindings/src/lib.rs` updated. Post-fix full suite 785/785.
  **Held open for the user**: full browser eyeball of a legendary
  layout, and the in-game import anchor (blocked on #313).*
- *2026-07-20 — browser eyeball confirmed by the user ("badges look
  good") after the in-game badge icons landed in the particle path
  (`66589dd`; the first implementation only covered the Graphics
  path — lesson recorded in `web/CLAUDE.md`). The only remaining open
  verification item is the in-game import anchor, blocked on
  [#313](https://github.com/storkme/spaghettio/issues/313).*
