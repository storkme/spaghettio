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
- The voider row template is the **net-negative variant of
  `self_loop_row`**: loop corridor recirculating the 25% return, net
  INPUT from the surplus stream, no export splitter. The spike's
  break-even test already pins the LP-side arithmetic; the template is
  new but sits on proven geometry.

Why curated / why this dodges the laundering family: voider rows are
synthesized ONLY for items in `surplus_outputs` — streams the LP
already decided are genuinely excess at honest pricing. No recipe
selection happens at voiding time, so "craft a combinator to recycle
it" can never arise: crafting a combinator was never in the plan.

### D2. Solid surplus export (needed regardless of policy)

`Export` must work for solids: extend the step-7 output merger to
`surplus_outputs` entries (solid), reusing the multi-item merge-cursor
+ UG-hop machinery landed for multi-item outputs. Each solid surplus
stream gets a merge column tiling east of the target's. This is the
smallest piece of the RFP and is independently valuable (uranium
chains: the U-238 surplus stream currently errors as stranded).

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
  solver so inserter counts are computable. Cons: the sushi belt
  segment violates item-isolation as modeled — needs a validator
  concept of an intentional mixed segment (analogous to the
  `:selfloop:` exemption, e.g. a `:sushi:` segment class with its own
  saturation math).
- **(b) Filtered-splitter cascade**: a splitter tree peeling one item
  per stage. Pros: belt-only. Cons: 12 stages of new splitter
  semantics in both rate walkers, larger footprint.

The RFP does not pick (a) vs (b) — Phase 0 does, with the decision
recorded here. Everything downstream of the sorter is ordinary bus:
sorted per-item lanes join the bus exactly like any producer row's
output.

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

1. **Physicals**: if Phase 0 finds the recycler cannot be modeled in
   the current row abstractions at all (e.g. direct ejection with no
   viable collection geometry under our belt model), stop and rescope
   before any template work — the sorting architecture depends on it.
2. **Sorter viability**: if neither sorting architecture can express a
   12-way sort within ~2× the footprint of community reference
   designs (measured via `blueprint-analyze` on the corpus), the bus
   model is the wrong substrate for scrap sorting — stop and consider
   a dedicated "sorter block" pre-solved template (balancer-library
   style) instead of generative geometry.
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

- **Phase 0 — physicals + sorter design spike** (no production code):
  factorio-expert verification of recycler footprint/ejection and
  filter-entity semantics; `blueprint-analyze` sweep of community
  Fulgora designs; sorting architecture decision recorded here.
- **Phase 1 — solid surplus Export**: step-7 merger extension to solid
  `surplus_outputs` + un-strand the uranium fixture class.
  Independently valuable; ships alone.
- **Phase 2 — voider rows**: net-negative `self_loop_row` variant +
  layout-synthesized voider specs from `surplus_outputs` under
  `SurplusPolicy::Void`; voider-purity fixture.
- **Phase 3 — recycler rows + sorter**: per the Phase 0 decision;
  scrap-recycling row template + sorting into per-item bus lanes;
  `holmium-plate` fixture goes green.
- **Phase 4 — UI/API**: scrap flow exposure, surplus-policy toggle,
  `electromagnetic-science-pack` fixture; CLAUDE.md ladder gains a
  Fulgora tier.

Phases 1 and 2 are independent of the Phase 0 decision and can start
immediately; Phase 3 is the long pole.

## Decision log

- *2026-07-11 — drafted, following the Fulgora data + solver spike
  (see `docs/rfp-solver-net-flow.md` decision log for the spike's
  findings: LP economics exact; eps_surplus price-tuning falsified as
  a voiding mechanism — entity-crafting disposal laundering; hence D1's
  layout-level voiding design). Pending review.*
