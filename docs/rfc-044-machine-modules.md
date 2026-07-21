# RFC-044: Machine modules (no beacons)

## Summary

End-to-end support for modules *inside crafting machines*: export them in
the Factorio 2.0 blueprint format (today the exporter silently drops them),
teach the validator slot counts and productivity eligibility, display module
slots on machines in the web UI, and — in the final phase — let the solver
plan with a user-chosen module policy (type, tier, module quality), with
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
   2-slot machine, or prod modules on a non-eligible recipe, validates
   clean.
4. **Generated factories can't use modules at all** — no solver input, no
   stamping, no UI.

What already exists (audit 2026-07-21): parser support for all three
`items` formats; `ModuleItem` + `PlacedEntity.items` in the model;
`common::module_effect` / `module_slots` / beacon constants; and
`analysis.rs` (blueprint-analyze CLI) as the only consumer, computing
module+beacon bonuses for community blueprint dissection.

**Naming hazard**: `ItemFlow.module_id` refers to *partition* modules
(decomposition machinery), not game modules. Game modules are
`ModuleItem` / `PlacedEntity.items` throughout. No rename (churn); code
touching both must keep the vocabularies apart.

## Design

### Phase 0 — artifact boundary + data groundwork

The wires lesson (RFC-040, arc retro rule 4): verify the artifact encoding
*first*, not after five phases of model-side work.

- Extend `ModuleItem` with `quality: Option<QualityTier>`; parser populates
  it from the 2.0 `id.quality` field.
- Export `items` in the 2.0 insert-plan format:
  `[{"id": {"name": …, "quality": …?}, "items": {"in_inventory":
  [{"inventory": <module-inventory-id>, "stack": k, "count": 1}, …]}}]`.
  The exact `inventory` ID for crafting-machine module slots is verified
  empirically, not assumed: run blueprint-analyze over community corpus
  strings that carry modules (or have the game generate one) and copy the
  game's own encoding byte-shape.
- Round-trip test: import → export → import preserves `items` including
  quality.
- **In-game paste anchor at Phase 0** (user-run): one exported machine
  with modules must show module insert requests when pasted. Kill
  criterion 2 gates Phase 3 on this.
- Data extraction: extend `scripts/extract_recipes.py` (draftsman) to emit
  per-recipe `allow_productivity` and per-result `ignored_by_productivity`
  (catalyst) amounts — kovarex is in our corpus, and its returned U-235
  catalyst must not be prod-multiplied. Reconcile the extracted module
  effect data against the hand-written `common::module_effect` table;
  extracted data wins.

### Phase 1 — validator groundwork

Works on imported blueprints immediately; no solver dependency.

- `module-slots` check: Σ `items` counts ≤ `module_slots(entity)` (rated
  per entity); unknown module names warn.
- `module-eligibility` check: productivity modules only in machines whose
  recipe has `allow_productivity`.
- Both dispatched from `validate/mod.rs` (checks 24–25), with trace events
  per the explainability conventions (RFC-039 call-site pattern).

### Phase 2 — web UI display

- Renderer overlay: an in-game-style module slot row on each machine —
  `module_slots(entity)` slots, filled from `PlacedEntity.items`, empty
  slots shown empty. Read-only. Applies to imported blueprints today and
  generated layouts after Phase 3.
- Respect the renderer constraints in `web/CLAUDE.md`; user validates
  visuals (no screenshot iteration loops).

### Phase 3 — solver + generation

- **Engine param: a single global module policy** — `{type: none | speed |
  productivity, tier: 1–3, quality: QualityTier}` applied to every machine
  whose recipe is eligible (prod policy falls back to no modules on
  non-eligible recipes; speed applies everywhere with slots). Per-recipe
  overrides and mixed loadouts are a deliberate non-goal (followup if the
  global policy proves insufficient). Threads through wasm
  `solve`/`layout`, URL state, and the sidebar exactly like the quality
  arc's `q=` param (RFC-041 pattern).
- **Netflow**: per-column module factors at the existing choke point
  (`effective_crafting_speed` gains a module-aware sibling). Per craft:
  ingredients **unchanged**, results **×(1+Σprod)** (catalyst amounts
  exempt via `ignored_by_productivity`), craft rate **×(1+Σspeed)** where
  prod modules contribute their speed penalty. Modeling note: productivity
  multiplies *results*, it does not divide *ingredients* — these differ
  for multi-output recipes and catalysts, and getting this backwards is
  the classic mistake.
- **Module quality scaling**: beneficial effects ×(1+0.3·level) — the same
  QualityTier factor table the quality arc uses — negative effects flat.
  Verified against extracted data in Phase 0.
- **Stamping**: a layout post-pass fills `PlacedEntity.items` on eligible
  machines per the policy, mirroring the quality-stamping post-pass
  (machines only; never logistics).
- **Legacy tree-walk solver stays module-blind** — it is the
  recipe-selection oracle; machine counts come from netflow. If the parity
  harness objects, thread the same factor rather than forking the math.
- **Downstream effects are inherited, not rebuilt**: inserter sizing, belt
  tier selection, and lane checks consume planned per-machine rates, so
  speed-boosted (fewer, hotter) machines stress them automatically. New
  honest warnings under module-on configs are findings to record, not
  regressions to mask.

### Non-goals

- Beacons (own RFC; the `BEACON_*` constants stay analysis-CLI-only, and
  the known staleness of `BEACON_DISTRIBUTION_EFFECTIVITY = 0.5` vs 2.0's
  count-based falloff is a separate small fix in analysis land).
- Efficiency modules (no speed/prod effect; we don't model power capacity).
- Power-draw modeling (speed/prod modules raise consumption; we validate
  coverage, not capacity).
- Per-recipe module policies / mixed loadouts.

## Kill criteria

1. **Modules-off bit-identity.** With the policy at `none`, layouts must
   be bit-identical to pre-RFC across the corpus (unit bit-equality sweep +
   `SPAGHETTIO_STRESS_GOLDEN=check`) at *every* phase landing. Any diff:
   halt the phase and fix before proceeding. (Quality-arc precedent,
   RFC-041 KC2.)
2. **Phase 0 artifact anchor.** If the exported encoding doesn't match a
   game-generated moduled blueprint's byte-shape, or the in-game paste
   doesn't show module requests, Phase 3 does not start until it does. No
   solver work on top of an unverified artifact encoding.
3. **Eligibility data must be extractable.** If draftsman cannot yield
   `allow_productivity` / `ignored_by_productivity` for our 2.0 dataset,
   the prod half of Phases 1 and 3 stops for re-scoping — we do not
   hand-maintain a recipe whitelist.
4. **Prod must fit the LP.** If productivity crediting can't be expressed
   as per-column result scaling in the existing netflow shape (i.e. it
   breaks byproduct crediting or cycle-refusal semantics), Phase 3 splits:
   speed-only modules land, productivity becomes its own RFC.
5. **Machine counts must be explainable.** For spot-checked fixtures,
   solver machine count must equal
   `ceil(rate / (base_crafts_per_s × (1+Σspeed) × (1+Σprod)))` computed by
   hand. An unexplained delta means the choke point is leaking — halt.

## Verification plan

- Full e2e suite green at each phase; STRESSGOLD check for KC1.
- Round-trip unit test (parse → export → parse, items + quality preserved).
- blueprint-analyze re-analysis of our own exported moduled blueprints must
  report the module effects the plan assumed (closes the loop through the
  only pre-existing module-math consumer).
- Hand math spot-check: AM3 + 4× prod-3 → speed 1.25×(1−0.60)=0.5
  crafts-basis, products ×1.4; verify a fixture's machine count.
- Browser eyeball: slot overlay on an imported moduled blueprint (Phase 2);
  a moduled PU@2/s generated layout (Phase 3).
- In-game paste anchor at Phase 0 (user-run), per kill criterion 2.

## Phasing

| Phase | Deliverable | Depends on |
|-------|-------------|------------|
| 0 | Export `items` (2.0 format) + module quality in model/parser + round-trip test + data extraction + in-game anchor | — |
| 1 | Validator: slot-count + prod-eligibility checks | 0 (eligibility data) |
| 2 | Web UI module-slot overlay | 0 (quality in model) |
| 3 | Solver policy param + netflow factors + stamping + UI/URL plumbing | 0 (anchor, data); 1–2 landable in parallel |

## Decision log

- *2026-07-21 — drafted from the module/beacon audit (this session's
  validator-blindness question). Beacons deliberately split out as a
  non-goal; export-drops-items and parser-drops-quality findings recorded
  in Motivation. Awaiting review.*
