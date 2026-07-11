# RFP: Fulgora scrap-based layouts

## Summary

Support production chains that START from scrap and work downward —
Fulgora's inverted economics. The solver side is already proven: the
2026-07-11 spike (see `docs/rfp-solver-net-flow.md` decision log)
landed 310 real recycling recipes (excluded by default), opt-in
`NetflowOptions { allow_recycling, allow_voiding }`, and demonstrated
the LP's scrap arithmetic is exact (holmium golden to 4 decimal
places) with value-recovery recycling activating for free. This RFP
covers what remains: the **voiding policy** (the spike falsified the
obvious mechanism), **solid surplus routing**, the **recycler row +
item sorting** problem (the layout centerpiece — recycler output is
inherently mixed), and end-to-end Fulgora fixtures.

The scope cut that shapes everything below: **voiding is a layout
policy, not a solver objective.** The spike proved that pricing
surplus high enough to make the LP void also makes it profitable to
craft entities purely to recycle them (a laundering family the cycle
guard refuses loudly, killing the solve). Rather than fight the
objective function, the solver keeps reporting surplus honestly at the
frozen cost table, and the LAYOUT decides what to do with surplus
streams: export them to the perimeter, or synthesize voider rows that
destroy them — the same way it already synthesizes balancers and
output mergers without solver involvement.

## Motivation

Fulgora is the one place the game inverts production: a single input
(scrap) fans out into ~12 probabilistic outputs, and the challenge is
consuming/sorting/destroying the excess while extracting the binding
resource (holmium). Today:

- The solver handles it exactly (spike-verified: `holmium-plate@1/s`
  → 40.0 scrap/s, hand-derived and matched; recycling columns recover
  value from the byproduct soup at default pricing).
- The layout cannot build ANY of it: no recycler row template, no
  item-sorting story for mixed recycler output, no solid surplus
  routing (only fluids reach the perimeter today), no voiders.
- The UI can't express "scrap as raw input" chains.

Concrete target the RFP must make real: `electromagnetic-science-pack`
at a modest rate from `{scrap, water}`, 0 validation errors, with
every one of scrap's ~12 output streams either consumed, exported, or
voided — nothing stranded.

## Design

### D1. Voiding policy: curated, layout-level

The solver's `allow_voiding` machinery (validated in isolation by the
spike) stays available but is NOT the shipping mechanism. Instead:

- The solver runs Fulgora chains with `allow_recycling: true` and
  default pricing. Surplus lands in `surplus_outputs`, honestly.
- `LayoutOptions` gains `surplus_policy: SurplusPolicy` with variants
  `Export` (today's behavior: perimeter routing) and `Void`.
- Under `Void`, the layout synthesizes **voider rows** for solid
  surplus: recycler banks that consume a surplus stream and recycle it
  down to nothing. The voider cascade is deterministic arithmetic, not
  an LP decision: an item chain-recycles through its recycling recipes
  until it reaches self-voiders (X → 0.25X), and a self-feeding
  recycler bank destroys any input completely. Machine counts derive
  from gross throughput (each pass returns 25%, so a stream of `r`/s
  needs recyclers sized for `r / 0.75` gross plus the cascade's
  intermediate volumes — all computable from the recipe data).
- The voider row template is a **new template inspired by
  `self_loop_row`** — the vocabulary transfers (loop corridor,
  recirculation), but the plumbing inverts: the corridor is fed by a
  bus TAP consuming the surplus stream (not by a priority splitter off
  the machine's own output), it recirculates only the ~25% return, and
  there is no export splitter or declared bus output at all. It gets
  its own design pass in Phase 2 — the reuse claim is vocabulary, not
  code. The spike's break-even test already pins the LP-side
  arithmetic. Phase 2 additionally depends on the recycler physicals
  from Phase 0 (footprint, belt-vs-inserter ejection) — voider rows
  ARE recycler banks.

Why curated / why this dodges the laundering family: voider rows are
synthesized ONLY for items in `surplus_outputs` — streams the LP
already decided are genuinely excess at honest pricing. No recipe
selection happens at voiding time, so "craft a combinator to recycle
it" can never arise: crafting a combinator was never in the plan.

### D2. Solid surplus export (needed regardless of policy)

`Export` must work for solids, and it splits into two pieces of very
different size:

- **D2a — merger extension** (small): extend the step-7 output merger
  to solid `surplus_outputs` entries whose stream already has its own
  east-flowing belt — i.e. surplus from single-solid-output rows, and
  (once Phase 3 lands) the sorter's per-item lanes. Reuses the
  multi-item merge-cursor + UG-hop machinery; each stream gets a merge
  column tiling east of the target's.
- **D2b — multi-solid-output row belts** (not small, inherited): a row
  whose machine produces TWO solid items (uranium-processing: U-235 +
  U-238) only emits an output belt for `solid_outputs.first()` today
  (`placer.rs` single `output_belt_y` per `RowSpan`) — the merger has
  nothing to read for the second stream. This is the netflow RFP's
  still-open "multi-solid-output specs" item, folded in here because
  the uranium surplus stream cannot be un-stranded without it. It
  needs template work (a second output belt + inserters per row), not
  merger work.

D2a alone is independently valuable and ships first; the uranium
fixture goes green only after D2b.

### D3. Recycler rows and the sorting problem (the centerpiece)

Scrap-recycling's output is inherently MIXED — one machine emits ~12
item types. The entire bus model assumes one item per lane, and
`check_belt_item_isolation` errors on mixing. This needs a genuinely
new sub-system, and its design is gated on **Phase 0 physicals
verification** (via the factorio-expert pass + draftsman data):

- Recycler footprint (believed non-square) and whether it EJECTS
  directly onto belts (mining-drill-style) or is inserter-extractable.
  Both break current template assumptions (square `machine_size`,
  inserter extraction) in different ways.
- Filter entities available to sort: filter inserters (blueprint
  `filter` field) and/or filtered splitters (`filter` on splitter).
  Neither is modeled today (`PlacedEntity` has no filter field;
  validators have no filter semantics).

Candidate sorting architectures to evaluate in Phase 0 (with
`blueprint-analyze` runs over the community corpus for Fulgora sorter
designs as reference):

- **(a) Filter-inserter sorting**: recyclers drop to a short "sushi"
  collection belt per row; a bank of filter inserters lifts each item
  type onto its own bus lane. Pros: inserter mechanics already
  modeled except the filter field; per-item rates are known from the
  solver so inserter counts are computable. Cons: the sushi segment
  violates item isolation as modeled. The exemption mechanism, named
  now so KC5 is evaluable: `check_belt_item_isolation`
  (`validate/belt_structural.rs`) looks up segment KIND by a
  `:sushi:` segment-id prefix and skips only sushi↔sushi adjacencies;
  a NEW companion check owns the sushi↔ordinary boundary, requiring
  every transition off a sushi segment to pass through a filter
  inserter (or a declared sink), and validating per-item saturation
  on the sushi belt as Σ(per-item rates from the solver) vs belt
  capacity — `carries` on sushi tiles is a segment-level item SET,
  not a single tag. (Note the analogous precedent, the `:selfloop:`
  exemption, lives in `check_belt_loops`, not item isolation — this
  is a new exemption in a different check, not an extension of that
  one.)
- **(b) Filtered-splitter cascade**: a splitter tree peeling one item
  per stage. Pros: belt-only. Cons: 12 stages of new splitter
  semantics in both rate walkers, larger footprint.
- **(c) Pre-solved sorter block**: a balancer-library-style stamped
  template (possibly imported/adapted from a community design) for
  the whole 12-way sort, treated as an opaque block with typed ports.
  Pros: sidesteps generative geometry entirely; the corpus proves
  such designs exist. Cons: fixed shapes per (item-set, rate) tier;
  a new library to curate. This is a first-class alternative, not a
  fallback — Phase 0 evaluates all three.

The RFP does not pick (a)/(b)/(c) — Phase 0 does, with the decision
recorded here. Everything downstream of the sorter is ordinary bus:
sorted per-item lanes join the bus exactly like any producer row's
output.

**Footprint benchmark protocol (frozen now, before Phase 0 — the
frozen-cost-table precedent)**: reference = Fulgora sorter designs in
`scripts/blueprints/` (or added to it from Factorio Prints, committed
before measurement), measured with `blueprint-analyze` as (bounding
box area, entity count) of the sorter subregion handling ≥10 item
types at ≥1 full yellow belt of scrap. Candidate architectures are
measured the same way on their Phase 0 paper designs. The 2× bound in
KC2 refers to bbox area under this protocol.

### D4. Solver/UI polish

- Plumb `allow_recycling` through the public solve API and the WASM
  boundary behind the Fulgora flow (a `scrap` external input can
  toggle it implicitly, or an explicit UI switch — decide in Phase 4
  with the UI work, not before).
- `scrap` joins the raw-input set in the sidebar; the item picker
  already lists Fulgora targets.
- The `recycling-or-hand-crafting` category quirk (scrap-recycling's
  actual category) is already handled by the spike's mapping.

## Kill criteria

1. **Physicals**: if fitting the recycler forces a new `RowKind`
   variant AND either (a) on-belt ejection forces a machine shape no
   current inserter-extractable template can wrap, or (b) collection
   geometry requires the bus abstraction itself to carry multi-item
   lanes at the template level (beyond the tagged sushi segment),
   stop and rescope before any template work. Falsifiable in Phase 0
   from the physicals artifact alone.
2. **Sorter viability (pivot within scope)**: if neither generative
   architecture (a)/(b) fits within 2× the frozen benchmark (protocol
   in D3), pivot to (c) the pre-solved block — recorded as a decision,
   not a kill.
   **Sorter kill (actual)**: if (c) ALSO fails — no pre-solved block
   can be expressed with typed ports the bus can feed and drain under
   existing lane semantics — then scrap sorting has no home in this
   engine: stop Fulgora layout support entirely, keep the solver-side
   support (spike state) and document the wall. This is the evidence
   that would make us abandon the RFP, stated up front.
3. **Voider purity**: synthesized voider rows must not change any
   solver-reported rate for non-surplus items (they are pure sinks).
   The spike's bit-identical isolation test extends to the layout
   level: if adding voider rows perturbs anything outside the voided
   streams, the synthesis is wrong — stop.
4. **No default regression**: every phase lands with the full suite
   green, zero golden-hash movement outside explicitly-blessed Fulgora
   fixtures, and the netflow determinism sweep byte-identical.
   Non-negotiable, evaluated per phase.
5. **Mixed-belt containment**: if architecture (a) is chosen and the
   sushi segment's validator exemption cannot be expressed WITHOUT
   weakening item-isolation checking for ordinary belts (i.e. the
   exemption leaks beyond tagged segments), the exemption approach is
   wrong — stop and switch to (b) or the pre-solved block.

*Scope review trigger (not a kill)*: if the sorter sub-system exceeds
~800 LOC of new template/validator code, pause and review against the
pre-solved-block alternative.

## Phase 0 findings (2026-07-11 spike)

Both artifacts below are backed by draftsman 3.3.0 (`python3`,
`draftsman.data.entities` / `draftsman.data.recipes`, plus
`draftsman.entity.new_entity` for blueprint-field round-trips) and a
`blueprint-analyze` sweep of `scripts/blueprints/`.

### (i) Recycler physicals

- **Footprint: non-square, as suspected.** `Furnace('recycler')`
  reports `tile_width=2, tile_height=4` (from
  `collision_box [[-0.7,-1.7],[0.7,1.7]]`; `selection_box` is
  marginally larger, `[[-0.9,-1.85],[0.9,1.85]]`). 4 valid directions
  (N/E/S/W), no fluid box, so `mirror` is a no-op for this entity.
- **Output: ejects directly, mining-drill-style — confirmed, not
  inserter-only.** The prototype sets
  `vector_to_place_result: [-0.5, -2.3]`. Cross-checked against every
  other machine on `MACHINES` in `scripts/extract_factorio_data.py`
  (`stone/steel/electric-furnace`, `assembling-machine-1`, `foundry`,
  `electromagnetic-plant`, `crusher`): all report `None` for this
  field. Recycler is the only crafting machine in our set with it set.
  At the default (north) orientation the vector lands 0.6 tiles beyond
  the entity's own north edge (`y=-1.7`) and 0.5 tiles off-center in x
  — i.e. one specific column of the 2-wide footprint, one tile past
  the machine, matching a mining drill's belt-drop exactly. It also
  keeps a conventional inserter-extractable output buffer
  (`result_inventory_size: 12`, sized for all 12 possible
  scrap-recycling results at once), so both extraction paths are
  physically legal — the belt-drop is the one that needs no inserter,
  which is what the reference design (below) uses.
- **Input: inserter-fed**, `source_inventory_size: 1` — one input slot,
  no belt-side direct input analogous to the output.
- **Filter entities — blueprint-format fields, draftsman-verified
  round-trip:**
  - Inserters: Factorio 2.0 removed the standalone
    `filter-inserter`/`stack-filter-inserter` entities. Every inserter
    type (`inserter`, `long-handed-inserter`, `bulk-inserter`,
    `fast-inserter`, `stack-inserter`) natively carries
    `filter_count: 5`. Blueprint fields: `use_filters: true`,
    `filters: [{"index": 1, "name": "<item>"}, ...]` (1-indexed),
    optional `filter_mode: "blacklist"` (omitted at the default
    `"whitelist"`).
  - Splitters: `filter: {"type": "item", "name": "<item>"}` (one item
    per splitter) routes that item to whichever side is
    `output_priority` (`"left"`/`"right"`); the other side gets
    everything else. Optional `input_priority`, same two values.
- **Rate math.** `crafting_speed: 0.5`. `scrap-recycling`:
  `energy_required: 0.2`, 1 scrap in, 12 possible results (amount 1
  each) at probabilities 0.20/0.07/0.06/0.05/0.04/0.04/0.04/0.03/0.03/
  0.02/0.01/0.01 (iron-gear-wheel, solid-fuel, concrete, ice,
  steel-plate, battery, stone, advanced-circuit, copper-cable,
  processing-unit, low-density-structure, holmium-ore respectively) —
  Σ = 0.60 expected items/craft. Craft time = 0.2/0.5 = 0.4s → 2.5
  scrap/s per recycler at 100% uptime → 1.5 items/s average mixed
  output per recycler. Saturating 1 yellow belt (15/s) of scrap needs
  **6 recyclers** (bare arithmetic, no modules/margin); red belt 12;
  blue belt 18. (Aside, not load-bearing for this RFP:
  `steel-plate-recycling` is a genuine `X → 0.25X` self-voider —
  `energy_required: 1`, ingredient 1 steel-plate, result 0.25 expected
  steel-plate — confirming D1's cascade arithmetic assumption for at
  least one real item.)

### (ii) Sorter architecture decision

**Corpus sweep result: the corpus is not empty — it has a genuine,
production, scrap-to-product Fulgora build.**
`scripts/blueprints/-OPqXI7f5b36Cd1Tpb2Z_electromagnetic_science_pack_from_begin.json`
("Electromagnetic science pack from begin": *"you need to submit only
basic resources to the input: Scrap metal ~3600 units per minute"*).
Decoded with draftsman and cross-checked against
`blueprint-analyze --json`: 1000 entities total (`blueprint-analyze`
sees 9 recipes / 85 non-recycler machines — it doesn't classify
`recycler` as a recipe-bearing machine today, a small gap worth a
follow-up issue but not blocking here). Full entity breakdown:
68 `recycler`, 96 `bulk-inserter` (32 filtered, covering 10 distinct
items: concrete, copper-plate, electronic-circuit, holmium-ore,
iron-plate, low-density-structure, plastic-bar, solid-fuel,
steel-plate, stone), 7 `long-handed-inserter` (unfiltered, positioned
directly against the recycler cluster — likely direct extraction of
the rarest items from the 12-slot output buffer rather than sushi
sorting), 24 `turbo-splitter` (21 filtered — holmium-ore, ice, stone,
advanced-circuit, iron-plate, copper-cable, electronic-circuit,
plastic-bar, battery, processing-unit, solid-fuel, iron-gear-wheel),
523 `turbo-transport-belt`, 101 `turbo-underground-belt`. **This is a
hybrid of (a) and (b)**: filter inserters do fine per-item sushi
extraction, filtered splitters do coarser redistribution elsewhere in
the base. It runs ~60/s scrap (turbo belt, matches the "3600/min"
description) — 4× the RFP's 1-yellow-belt benchmark rate.

**Frozen-protocol measurement (60/s scale, the corpus's native rate):**
the recycler+belt+splitter+inserter subregion (`recycler`,
`turbo-transport-belt`, `turbo-underground-belt`, `turbo-splitter`,
`bulk-inserter`, `long-handed-inserter` — excluding the
`electromagnetic-plant`/`chemical-plant`/`assembling-machine-3`
production machines and their pipes) is 819 entities in a
59.5×47.0 = **2796-tile bbox** — essentially the *entire* blueprint
footprint (total bbox 60.5×48.0 = 2904 tiles). The sorter dominates a
real Fulgora build; this benchmark reference is provisional-by-scale
(it's 4× our target rate) but not provisional-by-existence — it's a
real, working design.

Sanity check on recycler count: bare rate math predicts 60/2.5 = 24
recyclers for 60/s scrap; the design uses 68 (2.83×). Most likely
cause: productivity/quality modules inflating `energy_required` (the
corpus's own "Clover of Legendary Quality Advanced Circuit" design
confirms quality-module use is standard on Fulgora). **Phase 3 should
size recycler banks from the bare crafting-speed arithmetic above
against the LP's solved rate, not from community blueprint headcount.**

**Paper-design sketch at 1 yellow belt (15/s scrap), architecture (a):**
6 recyclers (2×4 each) side by side = 12 tiles wide × 4 tall row;
sushi collection belt 1 tile north (matches the confirmed drop-vector
finding above). Total mixed output ≈ 6 × 1.5/s = 9 items/s — under a
single yellow belt's 15/s cap, so one sushi belt suffices at this
scale. A bank of filter inserters (≤12, one per item type; the
rarest — holmium-ore at 6×0.025=0.15/s, low-density-structure,
processing-unit — could go to direct long-handed extraction instead,
mirroring the reference design's 7 long-handed inserters) lifts each
item onto its own lane. Rough footprint: ~12 (belt run) × ~7-8 (input
inserter row + machine + sushi belt + sort-inserter row + short
per-lane merge run) ≈ 84–96 tiles, ~30–35 entities. Comfortably under
the frozen-protocol 2× bound under any reasonable scaling of the
2796-tile/819-entity reference.

Architecture (b) (filtered-splitter cascade) is confirmed feasible
(`splitter.filter` verified above, and the reference design uses 21 of
them) but the RFP's stated con holds on paper: a 12-item cascade needs
12 sequential stages, and splitter-level item routing isn't modeled in
either rate walker (`belt_flow.rs`) at all today — a materially bigger
validator lift than (a)'s single new companion check.

Architecture (c) (pre-solved block) is proven to exist in the wild —
the reference design effectively *is* one — but adopting it as our
mechanism means building a new blueprint-import/port-typing subsystem,
which is a bigger new subsystem than extending validators for (a). Not
needed to clear KC2 since (a) already fits the benchmark; kept as the
fallback KC2 specifies.

**Decision: architecture (a), filter-inserter sushi**, for the sorting
sub-problem specifically. Smallest validator surface (one `filters`
field + one new companion check next to, not modifying,
`check_belt_item_isolation`; `BusLane.item: String` in
`lane_planner.rs` is untouched — a sorted item becomes an ordinary
single-item lane the instant it clears the filter-inserter boundary),
proven at scale by the corpus, comfortable headroom at the RFP's
reference rate. (b) remains a viable *later* addition — the corpus
shows real designs blend both — but is not the Phase 3 starting point.

### Kill criteria evaluation

- **KC1 (physicals) — TRIPS.** The recycler's non-square footprint
  (2×4) cannot be represented by the engine's machine-footprint
  primitive: `common::machine_size(entity: &str) -> u32` is a single
  scalar, and `machine_tiles(x, y, size)` generates a square `size ×
  size` region unconditionally (`crates/core/src/common.rs:32-48`).
  This primitive is called at 45 sites across `placer.rs`,
  `templates.rs` (via `placer.rs`'s `msz`), `validate/inserters.rs`,
  `validate/power.rs`, `validate/belt_structural.rs`,
  `validate/belt_flow.rs`, `bus/junction_solver.rs`,
  `bus/ghost_router.rs`, `bus/lane_planner.rs`, `bus/layout.rs`,
  `blueprint.rs`, `analysis.rs`, `density.rs` — collision, occupancy,
  power coverage, and belt-structural checks all assume it. Worse than
  a mechanical scalar→tuple rename: several call sites thread the
  *same* `msz` through both the row's vertical stacking math and its
  horizontal per-machine pitch (e.g. `placer.rs:696`,
  `out_y = y_cursor + 2 + msz as i32 + 1` — one number used as both
  "how tall is the machine" and implicitly assumed equal to "how wide
  is the machine" elsewhere in the same function), so a 2×4 machine
  needs each call site individually triaged for which dimension
  applies, not a blind type swap. Independently, no existing `RowKind`
  template models direct-to-belt ejection at all — a repo-wide grep
  for `mining-drill`/`vector_to_place_result`/`drop_position` in
  `crates/core/src/` finds nothing outside `blueprint_parser.rs`
  (generic field parsing, unrelated to row generation). Both facts
  satisfy KC1 condition (a) independently. **Per KC1, this means stop
  and rescope before Phase 2/3 template work — not abandon Fulgora.**
  Recommended rescoping: extend `machine_size` to return `(width,
  height)` (existing square machines become `(s, s)`, zero behavior
  change) as its own mechanical prerequisite sub-phase touching the 45
  call sites above, done *before* any recycler-specific row/template
  work, rather than hand-rolling a bespoke collision path for
  recyclers alone (which would risk the router/validators
  silently disagreeing about recycler tile occupancy — worse than the
  refactor). KC1 condition (b) does **not** trip — see the sushi/
  `BusLane` analysis above; the core bus abstraction needs no changes.
- **KC2 (sorter viability) — does not trip.** Architecture (a) fits
  the frozen benchmark with large headroom at the RFP's reference
  rate; no need to invoke the 2× fallback margin or pivot to (c).
- **KC5 (mixed-belt containment) — does not trip.** The sushi
  exemption is additive and narrowly scoped as designed: a new
  segment-KIND lookup plus a standalone companion check, not a
  modification of `check_belt_item_isolation`'s existing logic for
  non-sushi segments; `BusLane` stays single-item throughout.

## Verification plan

Per the CLAUDE.md protocol, plus:

- **Phase 0 outputs are artifacts**: recycler physicals documented in
  this RFP; sorting architecture decision + community-corpus footprint
  benchmarks recorded in the decision log before Phase 3+ code.
- **Fixtures** (end state): `holmium-plate@1/s` from `{scrap, water}`
  (0 errors, both surplus policies exercised — Export and Void
  variants); `electromagnetic-science-pack` at a rate TBD by Phase 0
  belt math (0 errors, Void policy); a voider-purity fixture asserting
  bit-identical non-surplus behavior with/without voider rows.
- **Sushi/sorter rate math**: if (a), the mixed segment gets explicit
  saturation validation (sum of per-item rates vs belt capacity), unit
  tested against hand-derived scrap output rates.
- **Browser eyeball** of the first full scrap layout (user validates
  UI per project convention).
- Blueprint round-trip on every fixture; spot-check one exported
  blueprint in-game if feasible (first real ground-truth anchor for
  filter entities).

## Phasing

- **Phase 0 — physicals + sorter design spike** (no production code).
  Two distinct artifacts:
  (i) **recycler physicals**: factorio-expert verification of
  footprint, belt-vs-inserter ejection, and filter-entity semantics,
  recorded in this doc;
  (ii) **sorter architecture decision**: `blueprint-analyze` sweep of
  community Fulgora designs under the frozen benchmark protocol, then
  the (a)/(b)/(c) decision recorded here.
- **Phase 1 — solid surplus Export, D2a**: merger extension for
  surplus streams that already have their own belt. Independently
  valuable; ships alone; no Phase 0 dependency.
- **Phase 2 — voider rows**: new voider template (D1) +
  layout-synthesized voider specs under `SurplusPolicy::Void`;
  voider-purity fixture. **Depends on Phase 0 artifact (i)** — voider
  rows are recycler banks; footprint and ejection change the template.
- **Phase 2b — D2b multi-solid-output row belts**: second output belt
  on rows with two solid products; uranium fixture goes green. No
  Phase 0 dependency; sequenced by value whenever convenient.
- **Phase 3 — recycler rows + sorter**: per Phase 0 artifact (ii);
  scrap-recycling row template + sorting into per-item bus lanes;
  `holmium-plate` fixture goes green. **Depends on both Phase 0
  artifacts.**
- **Phase 4 — UI/API**: scrap flow exposure, surplus-policy toggle,
  `electromagnetic-science-pack` fixture; CLAUDE.md ladder gains a
  Fulgora tier.

Only Phase 1 (and 2b) can start before Phase 0 completes; Phases 2
and 3 wait on its artifacts as marked. Phase 3 is the long pole.

## Decision log

- *2026-07-11 — drafted, following the Fulgora data + solver spike
  (see `docs/rfp-solver-net-flow.md` decision log for the spike's
  findings: LP economics exact; eps_surplus price-tuning falsified as
  a voiding mechanism — entity-crafting disposal laundering; hence D1's
  layout-level voiding design). Pending review.*
- *2026-07-11 — PR #306 review round: (1) D2 split into D2a (merger
  extension) / D2b (multi-solid-output row belts — the uranium
  un-stranding actually lives there, inheriting the netflow RFP's
  open item); (2) voider template reuse claim hedged to
  "inspired by", own design pass; (3) Phase 2's dependency on Phase
  0's physicals artifact made explicit (voider rows are recycler
  banks) — phase artifacts split into (i)/(ii); (4) KC1 rephrased to
  a falsifiable RowKind/geometry test; (5) pre-solved sorter block
  promoted to first-class architecture (c) with the ACTUAL kill
  stated (all three fail → stop Fulgora layout support); (6) sushi
  item-isolation exemption mechanism named (segment-kind lookup +
  boundary check requiring filter inserters), correcting the
  `:selfloop:`-precedent reference (that lives in check_belt_loops).*
- *2026-07-11 — re-review verified all six resolutions against the
  code; merged as #306. **Accepted.** Phase 0 (physicals + sorter
  decision) and Phase 1+2b (solid surplus export, D2a+D2b) kicked off
  in parallel; Phases 2/3 gated on Phase 0's artifacts as specified.*
- *2026-07-11 — Phase 0 complete, both artifacts recorded above
  (draftsman 3.3.0 + `blueprint-analyze` sweep of
  `scripts/blueprints/`). Findings: recycler is 2×4 (non-square,
  confirmed via `tile_width`/`tile_height`) and ejects output directly
  onto a belt via `vector_to_place_result` (mining-drill-style,
  confirmed as unique among the machines in
  `scripts/extract_factorio_data.py`'s `MACHINES` list), while also
  keeping an inserter-extractable 12-slot output buffer. **KC1 trips**:
  the non-square footprint breaks `common::machine_size() -> u32`
  (single scalar) at 45 call sites spanning placement, occupancy,
  power, and all three belt/inserter validators — bigger than "one new
  `RowKind` variant," a `(width, height)` refactor is now a prerequisite
  sub-phase before Phase 2/3 template work, not a kill of the RFP.
  Sorter architecture: **(a) filter-inserter sushi selected** — the
  corpus has a real scrap-to-electromagnetic-science-pack build
  (`-OPqXI7f5b36Cd1Tpb2Z`, 68 recyclers + 96 bulk-inserters, hybrid of
  (a)/(b), running 4× our reference rate) that proves the mechanism at
  scale; a paper design at the RFP's 1-yellow-belt benchmark rate
  (6 recyclers, ~30-35 entities, ~85 tiles) clears the frozen 2× bound
  with room to spare. KC2 and KC5 do not trip. Phase 2/3 should budget
  for the `machine_size` refactor before recycler-row template work
  begins.*
- *2026-07-11 — `machine_size → machine_dims (width, height)` refactor
  LANDED (prerequisite sub-phase from the KC1 rescope above). All ~45
  call sites migrated with explicit per-axis choices; zero behavior
  change (full suite green, zero golden-hash movement — necessarily so,
  since every placed machine today is square, which also means tests
  cannot catch a wrong axis choice; an independent adversarial audit
  re-derived the geometry at every single-axis site and found zero
  defects). `"recycler" => (2, 4)` is in the dims table and in
  `MACHINE_ENTITY_NAMES` (it was missing — without it, every
  `is_machine_entity` gate would have made the footprint dead code).
  All 11 row templates keep scalar `machine_size` signatures but now
  open with `debug_assert_eq!(machine_dims(entity), (msz, msz))`
  tripwires. Notes for the Phase 2/3 implementer: (1) the `row_kind`
  square-assert fires for ANY non-square machine at classification
  time — a recycler panicking there is the tripwire working, forcing
  an explicit `RowKind`, not a placement bug; (2) templates use one
  scalar for both pitch and row-height internally, so giving a
  non-square machine an existing row kind needs a genuine per-axis
  template rework, not just passing width; (3) blueprint export
  centers entities as `(x + w/2, y + h/2)` but does not yet swap
  w/h for rotated non-square machines — recycler rows placed with
  east/west `direction` must handle this.*
- *2026-07-11 — Phase 2 (voider rows) LANDED. v1 scope: self-voiders
  only (`<item>-recycling`: X → fraction·X, e.g. uranium-238); cascade
  hops and multi-output hops fall back to Export with a
  `VoiderFallbackExport` trace event — never silently dropped. Fluid
  surplus is never voided. Synthesis is `bus::voider::
  synthesize_voiders` (layout-level clone of the SolverResult, one
  shared `size_self_voider` sizing fn used by synthesis, placer, and
  the stranded-byproducts check so they can't drift). Template:
  north-facing recycler bank at pitch 2, direct ejection onto a
  collector belt (`common::recycler_eject_tile`), 100% recirculation
  corridor (`:voider:` segment tag, loop-exemption shares the
  `:selfloop:` mechanism), inserter-fed from a near/far belt pair.
  **One design deviation from D1's prose, recorded**: the voider row
  is NOT fed by an ordinary west-trunk bus tap — the surplus producer
  row is east-flowing (its primary output is the solve's target), and
  forcing a west-directed ret spec walks backward across the row's own
  output belt (reproducible junction/isolation/dead-end failures).
  Instead ghost_router gained Step 7c: reuse the Step 7b
  producer-gathering + `merge_output_rows` machinery, then route the
  merged tail south-around to a dedicated supply column on the voider
  row, UG-hopping foreign columns. Chasing this exposed a dormant
  shared-code bug, fixed: `row_exit_origin` and lane `source_y`
  ignored `RowSpan::secondary_output_belt`, so a D2b secondary item
  with a real consumer would have exited from the primary belt's y
  (nothing exercised that shape before; suite green + zero golden
  movement proves the fix inert for existing layouts).
  `check_stranded_byproducts` accepts voided streams only with
  physical backing: `LayoutResult::voided_streams` ledger AND enough
  recycler entities running the recipe. Fixtures green:
  `tier_uranium_processing_voider` (0/0, 1 recycler eating 7.09/s
  U-238) and `voider_purity` (KC3: uranium-processing machines
  byte-identical between Export and Void legs — scoped to machine
  entities because bus width legitimately shifts when an item stops
  needing export lane geometry). KC4 held: zero golden-hash movement,
  Export remains the default.*
- *2026-07-11 — architecture **(d) chest-buffered sorting** recorded
  post-hoc (user-flagged; it was absent from Phase 0's (a)/(b)/(c)
  candidate set — a gap in that analysis, noted honestly). The
  dominant community meta for compact scrap setups: the recycler's
  drop vector inserts directly into a container on the drop tile
  (drill-style), and filtered bulk inserters pull from the chest on
  full-hand availability. What it buys over (a): the chest converts
  the probabilistic output trickle into random-access buffered stock
  (belt-pickup throughput for rare items — holmium-ore at p=0.01 —
  degrades badly at scale, which is sushi's weak point), twelve
  inserters physically fit around one chest, and a mixed-item chest
  violates no belt invariant — the entire `:sushi:` isolation
  exemption exists only because (a) puts the mixed stream on a belt.
  Why it isn't v1: chests are unmodeled (no entity, and — the real
  cost — no buffer-node vertex in either rate walker, which are
  belt-segment graphs with inserters as machine↔belt edges), whereas
  (a) needed one model field + one contained exemption. **Standing
  note: (d) is the likely v2 evolution when scrap rates rise past
  what per-item belt pickup sustains; prerequisites are a container
  entity class + buffer-node flow semantics + eject-into-container
  support in the ejection model.***
- *2026-07-11 — Phase 3 PARTIAL LANDING; **RFP closes here**. What
  landed (all green, inert for existing layouts, zero golden
  movement): `templates::scrap_recycling_row` + `RowKind::
  ScrapRecycling` (south-facing recycler bank, direct ejection onto a
  `:sushi:` belt, per-item filter-inserter bank, crossing-free
  fan-out to one east-flowing belt per item);
  `RowSpan.secondary_output_belt` generalized to
  `sorted_output_belts: Vec<(item, y)>` behind one
  `output_belt_y_for(item)` helper; KC5 validators in
  `validate/sushi.rs` (isolation exemption sushi↔sushi only, new
  boundary check — every off-sushi transition through a matching
  filter inserter — and Σ-rate saturation check); e2e
  `fulgora_scrap_sorter_mechanism_present` as the green reproduction
  point. **KC5 does not fire** — containment held with zero
  ordinary-belt weakening. What did NOT land: the holmium-plate 0/0
  fixture. Snapshot-diagnosed wall, two independent structural
  failures: (1) a row has ONE `output_east` direction but the 12
  sorted outputs need per-item fates (3 consumer items west to
  trunks, ~9 surplus items east to the merger) — same
  east-row/west-consumer hazard the Phase 2 voider entry recorded,
  here structural; (2) `merge_output_rows` self-collides at ~11
  simultaneous surplus streams (UG entrances need clear tiles the
  dense fan-out occupies; built for 1–2 streams). Dual-fate items
  (stone, ice: partly consumed, partly surplus) need a solid split
  neither the lane planner nor merger models. The remaining work is
  a hybrid-direction multi-exit row + merger-at-scale design pass —
  **parked as an explicitly separate future sub-phase with those two
  failures as its entry criteria, alongside Phase 4 (UI, also
  parked). Priority shifted to Nauvis science scaling by user
  direction, 2026-07-11.***
