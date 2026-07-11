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
