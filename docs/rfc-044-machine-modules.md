# RFC-044: Machine modules (no beacons)

Rev 2 — revised 2026-07-21 after dual adversarial review (codebase lens +
mechanics lens, both ACCEPT-WITH-CHANGES; see decision log).

## Summary

End-to-end support for modules *inside crafting machines*: export them in
the Factorio 2.0 blueprint format (today the exporter silently drops them),
teach the validator slot counts and eligibility, display module slots on
machines in the web UI, and — in the final phase — let the solver plan with
a user-chosen module policy (type, tier, module quality), with
speed/productivity effects flowing through machine counts and rates, and the
modules stamped into the placed machines. **Beacons are explicitly out of
scope** — they are a geometry arc (3×3 footprints, supply ranges, power,
row spacing) and get their own RFC if/when we want them.

## Motivation

Modules are the biggest rate/footprint lever in real factories (4× prod-3
in an AM3 is +40% products; legendary modules multiply further), and our
pipeline is almost entirely blind to them. The concrete failures today:

1. **Round-trip strips modules.** The parser reads `items` from imported
   community blueprints (all three formats: 1.x array, 1.x map, 2.0
   insert-plan), but `blueprint.rs` never emits the field — import a
   moduled blueprint, re-export, and every module is silently gone. This
   is the same artifact-boundary asymmetry shape as the power-wires bug
   (RFC-040), found by the same kind of audit.
2. **Module quality is dropped at parse.** The 2.0 `id` object carries
   `{"name", "quality"}`; `extract_id` keeps only the name, and
   `ModuleItem` has no quality field to hold it.
3. **The validator has zero module awareness.** No check reads
   `PlacedEntity.items`; an imported blueprint with 9 modules crammed in a
   2-slot machine, prod modules in a recycler (which forbids productivity),
   or quality modules in a refinery (which forbids quality) validates
   clean.
4. **Generated factories can't use modules at all** — no solver input, no
   stamping, no UI.
5. **Discovered during review — built-in productivity is unmodeled.**
   Foundry, biochamber, and electromagnetic-plant carry
   `effect_receiver.base_effect = {productivity: 0.5}`: the game grants
   them +50% products with *zero* modules, and our solver has never
   credited it. This is a pre-existing divergence independent of modules;
   this RFC extracts the data and credits it under the module-aware path
   (see Phase 3), and defers the `policy=none` fix as a deliberate
   golden-moving followup.

What already exists (audit 2026-07-21): parser support for all three
`items` formats; `ModuleItem` + `PlacedEntity.items` in the model;
`common::module_effect` / `module_slots` / beacon constants — both tables
verified exact against Factorio 2.0.76 data via draftsman, except
`module_slots` is missing mining drills (electric = 3, big = 4), which
matters for validating imported blueprints; and `analysis.rs`
(blueprint-analyze CLI) as the only consumer, computing module+beacon
bonuses for community blueprint dissection.

**Naming hazard**: `ItemFlow.module_id` refers to *partition* modules
(decomposition machinery), not game modules. Phase 3 lands squarely on the
two densest partition-module files — `netflow.rs` (constructs
`module_id: 0` at ~7 sites) and `bus/layout.rs` (owns `(item, module_id)`
flow tagging and will host the stamping post-pass). No rename (churn), but
new code uses distinct identifiers — `ModulePolicy`, `game_module_*` —
never a bare `module`.

## Game-rule model (verified 2026-07-21)

The math the design below implements. Verified against Factorio 2.0.76
data (draftsman 3.3.0 probes), the Lua API docs, and the wiki's per-quality
module tables; probe scripts recorded in the review.

- **Per-module effects** (base): as in `common::module_effect` — verified
  exact, all nine rows.
- **Module quality scaling** is an *engine formula, not prototype data*
  (draftsman only carries base effects): beneficial effects scale
  ×(1 + 0.3·level) with levels normal=0 / uncommon=1 / rare=2 / epic=3 /
  legendary=5, **floored to 1% steps** for speed/prod modules (0.1% steps
  for quality modules, out of scope here). The flooring is real: prod-1
  at uncommon is +5%, not +5.2%. Legendary lands on integers (speed-3
  +125%, prod-3 +25%). Negative effects (prod's speed penalty) do **not**
  scale with quality.
- **Speed**: craft rate = crafting_speed × **max(0.2, 1 + Σspeed)** /
  recipe.energy. The 20% floor is load-bearing — a full prod-3 loadout in
  a cryogenic plant (8 slots) is −120% speed, which goes negative without
  it.
- **Productivity**: results ×(1 + Σprod), ingredients unchanged, applied
  only to the non-catalyst portion — per-result `ignored_by_productivity`
  amounts are exempt (kovarex: 40 of 41 U-235 and 2 of 2 U-238 ignored, so
  prod boosts only the net +1 U-235; kovarex is prod-eligible in 2.0).
  Productivity multiplies *results*, it does not divide *ingredients* —
  these differ for multi-output recipes and catalysts.
- **Built-in productivity**: Σprod additionally includes the machine's
  `effect_receiver.base_effect` productivity where present (foundry,
  biochamber, EMP: +0.5). The per-recipe `maximum_productivity` cap
  (default 3.0) and research productivity never bind at module-only
  magnitudes in this RFC's scope — noted and ignored (non-goal).
- **Eligibility is two-level**: per-recipe `allow_productivity` (116
  recipes in our dataset) AND per-machine `allowed_effects` — gated on
  the module's BENEFICIAL effect only (corrected in the Phase 1 review:
  speed modules carry a quality malus in 2.0, yet speed-in-beacon is
  legal despite beacon forbidding quality — harmful side-effects never
  gate). Recycler forbids productivity; oil-refinery, rocket-silo, and
  pumpjack forbid quality; beacon forbids productivity and quality.
- **Blueprint encoding** (2.0 insert-plan): `"items": [{"id": {"name": …,
  "quality": …?}, "items": {"in_inventory": [{"inventory": <class-id>,
  "stack": k}, …]}}]` — one `in_inventory` entry per module, `stack`
  0-based, no `count` field (defaults to 1 per entry), `quality` omitted
  at normal. The `inventory` define is **per entity class**: crafting
  machines / furnaces / rocket silo = 4, lab = 3, mining drill = 2,
  beacon = 1. A wrong ID fails *silently* on paste (modules request into
  the wrong inventory, or an unfulfillable one) — and our parser discards
  the `inventory` field, so round-trip tests cannot catch it; only the
  paste anchor can (KC2).

## Design

### Phase 0 — artifact boundary + data groundwork

The wires lesson (RFC-040, arc retro rule 4): verify the artifact encoding
*first*, not after five phases of model-side work.

- Extend `ModuleItem` with `quality: Option<QualityTier>`; parser populates
  it from the 2.0 `id.quality` field.
- Export `items` in the 2.0 insert-plan format per the game-rule model
  above, with the per-class inventory table. **Byte-shape source**: a
  draftsman-emitted reference blueprint (the community corpus has *zero*
  2.0 insert-plan examples — every moduled corpus string is 0.15-era map
  format, and our envelope already stamps version 2.0.32, so there is no
  1.x-migration safety net: `items` must be insert-plan shape).
- Round-trip test: import → export → import preserves `items` including
  quality. (Blind to the inventory ID by construction — KC2 covers that.)
- **In-game paste anchor at Phase 0** (user-run): exported machines with
  modules — at minimum one entity per exported inventory class — must
  show the modules *landing in the module slots*, not merely "some insert
  request". Kill criterion 2 gates Phase 3 on this.
- Data extraction: extend `scripts/extract_factorio_data.py` (draftsman;
  this is the actual recipes.json generator) to emit per-recipe
  `allow_productivity`, per-result `ignored_by_productivity`, per-machine
  `allowed_effects` and `effect_receiver.base_effect` productivity, and
  mining-drill module slots. The Rust `Recipe`/machine structs + serde
  gain the matching fields. (Pre-verified extractable: all attributes
  present in draftsman 3.3.0 under exactly these names — KC3 is settled
  in advance and retained only as a regression gate on the extraction
  landing.) Module *quality scaling* is not prototype data — the
  1%-floored ×(1+0.3·level) formula is anchored to the wiki per-quality
  tables instead.

### Phase 1 — validator groundwork

Works on imported blueprints immediately; no solver dependency.

- `module-slots` check: Σ `items` counts ≤ `module_slots(entity)` (rated
  per entity); unknown module names warn. Prerequisite: add mining drills
  to `module_slots` (electric = 3, big = 4) — moduled drills are
  ubiquitous in community blueprints and would otherwise all false-flag.
- `module-eligibility` check, two-level: productivity modules require the
  recipe's `allow_productivity` AND every module's effects must be ⊆ the
  machine's `allowed_effects` (catches prod-in-recycler,
  quality-in-refinery, prod-in-beacon).
- Both dispatched from `validate/mod.rs` (checks 24–25), with trace events
  per the explainability conventions (RFC-039 call-site pattern).

### Phase 2 — web UI display

- Renderer overlay: an in-game-style module slot row on each machine —
  `module_slots(entity)` slots, filled from `PlacedEntity.items`, empty
  slots shown empty. Read-only. Applies to imported blueprints today and
  generated layouts after Phase 3.
- Slot counts reach TS via a new wasm export wrapping
  `common::module_slots` (single source; no duplicated TS table).
- Respect the renderer constraints in `web/CLAUDE.md`; user validates
  visuals (no screenshot iteration loops).

### Phase 3 — solver + generation

- **Engine param: a single global module policy** — `{type: none | speed |
  productivity, tier: 1–3, quality: QualityTier}` applied to every machine
  whose recipe+machine pair is eligible (prod policy falls back to no
  modules where ineligible; speed applies everywhere with slots).
  Per-recipe overrides and mixed loadouts are a deliberate non-goal
  (followup if the global policy proves insufficient). Threads through
  wasm `solve`/`layout`, URL state, and the sidebar exactly like the
  quality arc's `q=` param (RFC-041 pattern).
- **Speed factor has a single choke point**: `effective_crafting_speed`
  (recipe_db.rs) gains a module-aware sibling, consumed at its one netflow
  call site, flowing to both the LP cost and per-machine rates. Apply the
  20% floor there.
- **Productivity has NO single choke point** — result scaling must be
  applied consistently at three separate netflow sites, and missing any
  one silently desyncs LP flows from the `MachineSpec` rates that inserter
  sizing and the validators consume:
  1. the candidate **net build** (netflow.rs ~443–465), applied to gross
     products minus `ignored_by_productivity` amounts **before** netting
     (never to net coefficients — kovarex's sign logic in
     `classify_self_loop`/`raw_net_per_craft` depends on this);
  2. `build_machine_spec`'s per-machine output rates (~825–836);
  3. the self-loop consumed/produced rates (~838–889), where kovarex's
     loop-back belt sizing lives.
- **Built-in productivity**: `base_effect` joins Σprod on the module-aware
  path (any `policy ≠ none`). At `policy = none` the engine stays
  bit-identical to today (KC1), accepting the known pre-existing foundry /
  biochamber / EMP divergence; promoting the fix to the none-path is a
  recorded golden-moving followup, outside this RFC.
- **Recipe selection may legitimately shift.** The default solve path is
  free mode: recipe mix comes from LP column costs, and module factors
  change both costs and coefficients — a module policy can flip the chosen
  mix (e.g. oil routes). This is *accepted* behavior (module economics are
  real economics); observed flips on corpus fixtures get decision-log
  entries, and KC5 spot-checks use the actually-selected recipe.
- **Legacy tree-walk solver stays module-blind — safe by construction**,
  not by harness: on the default path the walk isn't invoked at all (free
  mode), `solve_compat_*` has no non-test callers, and the parity harness
  only builds module-off configs, so module-on parity is out of scope by
  construction.
- **Stamping**: a layout post-pass fills `PlacedEntity.items` on eligible
  machines per the policy, mirroring the quality-stamping post-pass
  (machines only; never logistics).
- **Downstream effects are inherited, not rebuilt**: inserter sizing, belt
  tier selection, and lane checks consume planned per-machine rates, so
  speed-boosted (fewer, hotter) machines stress them automatically. New
  honest warnings under module-on configs are findings to record, not
  regressions to mask.

### Non-goals

- Beacons (own RFC; the `BEACON_*` constants stay analysis-CLI-only, and
  the known staleness of `BEACON_DISTRIBUTION_EFFECTIVITY = 0.5` vs 2.0's
  count-based falloff is a separate small fix in analysis land).
- Quality modules as a *policy* option (they parse and validate, but the
  generator never plans them).
- Efficiency modules (no speed/prod effect; we don't model power capacity).
- Power-draw modeling (speed/prod modules raise consumption; we validate
  coverage, not capacity).
- Per-recipe module policies / mixed loadouts.
- `maximum_productivity` cap and research productivity (never bind at
  module-only magnitudes in scope).
- Crediting `base_effect` at `policy = none` (recorded golden-moving
  followup).
- Teaching `analysis.rs` module-quality scaling (followup; see
  verification plan).

## Kill criteria

1. **Modules-off bit-identity.** With the policy at `none`, layouts must
   be bit-identical to pre-RFC across the corpus (unit bit-equality sweep +
   `SPAGHETTIO_STRESS_GOLDEN=check`) at *every* phase landing. Any diff:
   halt the phase and fix before proceeding. (Quality-arc precedent,
   RFC-041 KC2. Caveat: STRESSGOLD is host-cache-relative and opt-in — this
   is a local gate, not CI-enforced.)
2. **Phase 0 artifact anchor.** If the exported encoding doesn't match the
   draftsman-emitted reference byte-shape, or the in-game paste doesn't
   put modules *into the module slots* for every exported inventory class,
   Phase 3 does not start until it does. Round-trip tests cannot cover
   this (the parser discards the inventory ID) — the anchor is the only
   gate. No solver work on top of an unverified artifact encoding.
3. **Eligibility data must land in recipes.json.** Pre-verified
   extractable (2026-07-21 draftsman 3.3.0 probe: `allow_productivity` on
   122 recipes, per-result `ignored_by_productivity`, `allowed_effects`,
   `base_effect` all present under those names). Retained as a regression
   gate: if the extraction can't be landed faithfully, the prod half of
   Phases 1 and 3 stops for re-scoping — we do not hand-maintain a recipe
   whitelist.
4. **Prod must fit the LP.** If productivity crediting can't be expressed
   as per-column constant coefficient scaling at the three enumerated
   netflow sites (i.e. it breaks byproduct crediting or cycle-refusal
   semantics, which operate on column structure, not values), Phase 3
   splits: speed-only modules land, productivity becomes its own RFC.
5. **Machine counts must be explainable — two-part.**
   (a) *Solver, exact*: the fractional `MachineSpec.count` (netflow snaps
   to 1e-9; it does not ceil — "1.06 refineries" is a real output) must
   equal `rate / (crafts_per_sec × output_per_craft)` within snap
   tolerance, where `crafts_per_sec = crafting_speed × max(0.2, 1+Σspeed)
   / energy` and `output_per_craft = (amount − ignored)·(1+Σprod) +
   ignored`, summed over the target product's results × probability, with
   Σspeed/Σprod the per-module 1%-floored quality-scaled effects
   (+ `base_effect`).
   (b) *Placer, ceil*: placed machine count = ⌈fractional⌉, spot-checked
   on **Pooled single-spec fixtures only** — the ceil lives in the placer
   per row-spec, and partitioned sharding can legitimately exceed
   ⌈total⌉.
   An unexplained delta on either part means the factor plumbing is
   leaking — halt.

## Verification plan

- Full e2e suite green at each phase; STRESSGOLD check for KC1.
- Round-trip unit test (parse → export → parse, items + quality
  preserved), acknowledging its inventory-ID blind spot.
- Export byte-shape diffed against the draftsman-emitted reference
  blueprint (Phase 0).
- blueprint-analyze re-analysis of our own exported moduled blueprints
  must report the module effects the plan assumed — **normal-quality
  modules only** (analysis.rs has no quality scaling; teaching it is a
  followup, and gating on it un-taught would fail spuriously).
- Hand math spot-checks: AM3 + 4× prod-3 → speed 1.25×(1−0.60) = 0.5
  crafts-basis, products ×1.4; cryo + 8× prod-3 → speed floor engages:
  2.0 × max(0.2, 1−1.2) = 2.0×0.2 = 0.4, products ×(1+0.8+0.5 base) =
  ×2.3; verify fixture machine counts per KC5.
- Browser eyeball: slot overlay on an imported moduled blueprint
  (Phase 2); a moduled PU@2/s generated layout (Phase 3).
- In-game paste anchor at Phase 0 (user-run), per kill criterion 2 — one
  entity per exported inventory class.

## Phasing

| Phase | Deliverable | Depends on |
|-------|-------------|------------|
| 0 | Export `items` (2.0 insert-plan, per-class inventory table) + module quality in model/parser + round-trip test + data extraction (`extract_factorio_data.py`) + in-game anchor | — |
| 1 | Validator: slot-count (incl. drills) + two-level eligibility checks | 0 (eligibility data) |
| 2 | Web UI module-slot overlay + `module_slots` wasm export | 0 (quality in model) |
| 3 | Solver policy param + netflow speed/prod factors (floor, three prod sites, base_effect) + stamping + UI/URL plumbing | 0 (anchor, data); 1–2 landable in parallel |

## Decision log

- *2026-07-21 — drafted from the module/beacon audit (this session's
  validator-blindness question). Beacons deliberately split out as a
  non-goal; export-drops-items and parser-drops-quality findings recorded
  in Motivation. PR #319.*
- *2026-07-21 — rev 2 after dual adversarial review (two independent
  Fable cold-readers: codebase-claims lens, game-mechanics lens; both
  ACCEPT-WITH-CHANGES, all findings accepted). Majors folded in: 20%
  speed floor (rev-1 formula went negative on its own default policy —
  8× prod-3 cryo); 1%-step flooring on quality-scaled module effects
  (engine formula, not prototype data — Phase-0 "verify via extraction"
  claim was unfulfillable for it); built-in productivity
  (`effect_receiver.base_effect`: foundry/biochamber/EMP +50%, a
  pre-existing engine divergence — scoped to the module-aware path to
  preserve KC1, none-path fix deferred as golden-moving followup);
  mining-drill slots + per-class inventory IDs (drills = inventory 2,
  slots 3/4); machine-level `allowed_effects` added to eligibility
  (recycler forbids prod; refinery/silo forbid quality); KC5 rewritten
  two-part (netflow count is fractional-snapped, ceil lives in the
  placer; Pooled single-spec fixtures only); productivity choke-point
  claim corrected (three netflow sites enumerated; apply to gross before
  netting); dead parity-harness guard replaced with safe-by-construction
  rationale + explicit acceptance that module policies can flip free-mode
  recipe selection. KC3 settled in advance by draftsman probe
  (`allow_productivity`/`ignored_by_productivity`/`allowed_effects`/
  `base_effect` all extractable), retained as a regression gate. Corpus
  found to contain zero 2.0 insert-plan examples — draftsman-emitted
  reference promoted to primary byte-shape source. Script name corrected
  (`extract_factorio_data.py`).*
- *2026-07-21 — Phase 0 landed (branch `rfc044-phase0-module-export`):
  0a module quality in `ModuleItem` + parser (the 2.0 parse path gained
  its first test coverage — it was live but untested); 0b insert-plan
  export byte-matched against a regenerated draftsman reference, with
  `common::module_inventory_id` as the per-class table and round-trip +
  byte-shape tests; 0c data extraction. **Discovered en route**: a full
  `extract_factorio_data.py` regeneration silently DROPS the ~300
  surgically-appended recycling recipes (their categories are excluded on
  the default path) — added an `--augment` in-place mode as the required
  update path and diff-audited zero drift beyond the new keys. Bonus
  catalysts surfaced by the data: coal-liquefaction heavy-oil (25),
  pentapod-egg, fish-breeding, and the fluoroketone-hot returns on
  quantum-processor / cryogenic-science-pack. KC1 evidence: full suite
  green (34 e2e + lib), `SPAGHETTIO_STRESS_GOLDEN=check` clean (53
  passed, every stress fixture matched its committed golden — check mode
  panics on both missing-golden and drift, so a pass is non-vacuous).
  KC3 now pinned by a bundled-data regression test
  (`module_eligibility_data_is_bundled`). KC2 anchor string generated
  (`crates/core/examples/rfc044_anchor.rs`, local-only) covering all
  four inventory classes — **open, user-run**.*
- *2026-07-21 — Phase 1 landed (branch `rfc044-phase1-module-validators`)
  after deep local adversarial review (Fable, corpus-driven; verdict
  ACCEPT-WITH-CHANGES, all findings folded in). Checks 24–25:
  `module-slots` + `module-eligibility`, both WARNING severity by design
  (an invalid loadout doesn't fail a paste — the requests are silently
  unfulfilled; and imported blueprints must not be error-blocked by
  module quirks). Calls made: non-module item requests (fuel/ammo)
  classified out before counting; pre-2.0 `effectivity-module*` names
  alias to the efficiency family (the game migrates them — 94
  false-unknown warnings across the corpus otherwise); unknown modded
  entities carry no slot claim (`common::module_slots_known` → `None`
  skips the overflow warning — `se-recycling-facility` is not "0
  slots"); pumpjack added (2 slots, forbids quality) alongside the
  drills; rocket-silo + pumpjack join the beacon in the hand-tabled
  `allowed_effects` fallback (labs/drills deliberately absent — nil
  `allowed_effects` in game data means unrestricted, so skipping is
  exact); **eligibility gates on the module's beneficial effect only**
  (the rev-2 "effects ⊆ allowed_effects" rule was falsified by
  draftsman data: speed modules carry a quality malus yet
  speed-in-beacon is legal); slot-overflow issues carry no
  `IssueDetail` (the web starvation heatmap reads detail pairs
  category-blind). RFC-039 trace promise satisfied by the
  `ValidationCompleted` rollup — no per-check emits (checks run under
  rayon; thread-local trace would drop them), recorded as the
  deviation. Evidence: 18 unit tests; full corpus sweep 198/198 files,
  ZERO module warnings, with the census cross-validating Phase 0c's
  `allow_productivity` extraction against every recipe the community
  actually prod-modules; full suite green; KC1 safe by construction
  (generated layouts never populate `items`).*
