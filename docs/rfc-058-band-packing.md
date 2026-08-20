# RFC-058: Band packing — 2D placement at row granularity

Registry: [`rfcs.md`](rfcs.md). Status: **Concluded 2026-07-31 — kill
criterion 1 FIRED in phase 4: the real planner under physically-legal
routing, measured faithfully (criterion-scope non-pole extents, honest
footprints, scoring bypassed), holds −27.0% against the −33.0% bar, with
the trajectory adverse as correctness increased. Phases 0–3's evidence and the inert
scaffolding stand; the packed builder remains flag-gated and default-off
as the falsification record. See the final decision-log entry.**
Tracking: [#507](https://github.com/storkme/spaghettio/issues/507).

## Summary

Place machine **row bands** in two dimensions instead of stacking them in a
single left-aligned column, and re-route the trunk taps that serve them.

A band is a maximal run of rows containing machines or inserters — one recipe's
machine row plus the inserter rows serving it. Belt rows are deliberately not
part of it; they are the transport this RFC re-plans. Today the placer stacks bands
vertically and left-aligns them, so the widest band fixes the layout width and
every narrower band leaks its entire right margin. Measured across ten
fixtures, that ragged right margin is **38.4% of all bounding-box area**.

This RFC keeps each band rigid and internally untouched, and changes only where
bands sit. That is the deliberate difference from RFC-057, which shattered rows
into per-machine islands, packed those, and lost more on transport than it
saved on space.

## Motivation

Two measurements, both from 2026-07-30.

**The waste is structural, not scattered.** Across ten bus-path fixtures the
bounding box is 61–65% empty, and that void splits into ragged-right margin
(38.4% of all bbox) and interior corridor space (26.6%). The ragged-right half
is one contiguous rectangle beside the narrow bands — on `belt5-ore` it is a
single 28×20 region, half the layout. No local move reaches it; using it means
relocating whole bands.

**Machine granularity was already tried and lost.** RFC-057 packed per-machine
islands and, once its candidates finally materialised, they came out **38–250%
larger** than the bus they replaced — logistics outweighing machinery 6–8×. The
diagnosis in that RFC's decision log is that a row band is already a very
efficient delivery structure: one belt serving N machines of a recipe, each
extra machine costing about one belt tile. Shattering it destroys that and
nothing cheaper replaces it.

**Band granularity keeps the efficient structure and fixes the void.** A
headroom probe (`crates/core/examples/band_packing.rs`, obstacle-free, shelf
packing swept over target widths under a 3:1 aspect cap):

| fixture | bands | control | packed | band-bbox | transport |
|---|---:|---|---|---:|---:|
| `belt5-ore` | 3 | 36×26 | 36×14 | −46.2% | **−26.9%** |
| `sci1-ore` | 4 | 30×33 | 30×12 | −63.6% | **−36.6%** |
| `sci2-ore` | 8 | 54×76 | 61×21 | −68.8% | **−34.5%** |
| `pu1-plate` | 11 | 49×115 | 50×40 | −64.5% | **−39.6%** |

Aggregate over those four: band-bbox **−64.5%**, transport **−38.1%**.

That aggregate is over the fixtures that *pack*, which is selection bias if
read as a corpus result. Across all seven multi-band fixtures — counting the
three that cannot pack at their unchanged control area — band-bbox is
**−39.6%** (18,976 → 11,456 tiles). **−39.6% is the number to carry away**;
−64.5% is what the technique achieves where it applies.

Transport *falls*. That is the opposite of RFC-057's result and the reason this
is worth building: the control stacks bands in dependency order, so a consumer
band can sit far in y from its producer, and squaring the arrangement shortens
exactly those links. The area saving and the transport saving come together
rather than trading off.

**The transport figure is the weakest number here and should be treated as
provisional.** The probe prices band-centre to band-centre Manhattan distance.
Real bus transport runs down a trunk column and then across to the row, which
is close to Manhattan for the stacked control but not necessarily for a packed
layout, where trunks must reach bands that are no longer in one column. The
proxy may therefore distort the two cases *unequally* and flatter packing.
Claiming the comparison is fair because the same proxy applies to both is only
valid if the distortion is symmetric, and there is no evidence yet that it is.
The area result does not depend on this; the "transport falls too" claim does.
Phase 3's spike (below) is what settles it.

### Where it does not apply

The probe also characterised the failures, and they are structural rather than
tuning problems. Bands are **almost always 5 tiles tall** — the machine row plus
its two inserter rows — with occasional 7-tall exceptions (`belt5-ore` and
`sci2-ore` have one each). Widths vary far more, measured range 3–144. So:

- **Fewer than ~3 bands** — nothing to pack. `ec10-ore`, `ec15-plate`,
  `gear5-plate` have one band each.
- **A wide band combined with too few bands.** The widest band floors the
  layout *width*; it does not by itself floor the aspect ratio, because
  stacking further shelves at that width adds height and squares the result
  up. What blocks squaring is having too few bands to build that height.
  `gear15-ore` (2 bands, widest 144) can reach only 144×12 — two shelves —
  giving 12:1, and `lds2-plate` (2 bands, widest 121) reaches 121×12 at
  10.08:1. Both are correctly refused. A layout with a 144-wide band and
  twenty bands to stack would pack fine.

This RFC does **not** address the dominant-wide-band case, which is 2 of the 10
fixtures measured. The obvious fix was tested on 2026-07-30 and failed: capping
machines per row gave −0.9% bbox at zero errors (−5.3% only by introducing 10
errors), because each sub-row re-pays its own belt overhead.

That result is narrower than "wide bands are unsolvable", and should not be
read as such. It falsifies *one* reshaping — `max_per_row` capping, which
forces additional sub-rows each carrying their own belt row. Other reshapings
were not tested; two machine rows sharing a single belt row, for instance,
would not obviously carry the same cost. The honest status is "one obvious fix
doesn't work", and a wide-band RFC remains open if those 2-in-10 fixtures
justify it.

## Design

### Band extraction

A band is a maximal run of rows `y` containing at least one machine or
inserter. Trunk belts span all rows and are deliberately **not** band-forming —
they are the transport being priced, not the structure being packed. The band
rectangle is the x-extent of its machines and inserters.

`place_rows` already returns `Vec<RowSpan>` carrying `row_width`, so the placer
has this natively; the probe reconstructs it by y-projection to stay decoupled.
Phase 1 makes the placer's own spans the source of truth.

### Packing

Shelf packing over band rectangles, with:

- target shelf width swept from the widest band upward;
- both source order and height-descending order;
- a **fixed inter-band gap** of 2 tiles for trunk/tap space;
- an **aspect cap**, so the optimum is not a degenerate one-shelf ribbon. The
  probe used 3:1, chosen to be comfortably squarer than anything the engine
  ships today and no more principled than that. It is not free: `insert3-ore`
  packs at 3.16:1 and is refused, so the cap decides one fixture in ten. Phase
  2 should sweep it and report the applicable-fixture count as a function of
  the cap rather than inheriting 3:1 by default.

The gap is genuinely fixed, and the packed heights in the results table
reconcile against it once varying band heights are accounted for: a shelf is as
tall as its tallest band, so `sci1-ore` (all bands 5 tall) packs to
`5 + 2 + 5 = 12`, and `belt5-ore` (bands 5, 5 and 7 tall) to `5 + 2 + 7 = 14`.
Assuming a uniform 5 makes those two look like they need different gaps.

That last point is load-bearing and was learned the hard way: minimising
bounding-box *area* alone drives every packing to a single shelf (all bands
side by side, height = tallest band), producing 176×5 and 219×5 "wins". A
30:1 ribbon is exactly the shape this whole workstream exists to remove —
`pu4-raw` ships one at 2752×90 today. Area is only a valid objective under an
aspect constraint.

Better packers (sequence-pair, B\*-tree, corner block list with annealing) are
the obvious upgrade if shelf packing leaves material headroom, but shelf
packing is what the probe measured and it is sufficient to test the premise.

### Integration

The bus pipeline is `place_rows` → `plan_bus_lanes` → `route_bus_ghost` →
`place_poles`. Bands are produced by the first stage and consumed by the
second, so band packing belongs **between** them:

1. `place_rows` builds bands as today (unchanged — band interiors are rigid).
2. A new packing step assigns each band a 2D position.
3. `plan_bus_lanes` plans trunks for the packed arrangement.
4. `route_bus_ghost` and `place_poles` run unchanged.

Step 3 is the real work. The lane planner currently assumes a vertical trunk
column with horizontal rows tapping off it; a 2D arrangement needs trunks that
reach bands in both axes. The routing machinery below it (negotiated A\*,
junction solver) is position-agnostic and should not need changes.

**Alternative considered and rejected: a post-pass over a finished layout**, in
the style of `LayoutOptions::compact_layout`. Attractive because it is
strictly additive and cannot regress the default path. Rejected because moving
a band after routing invalidates every trunk that served it, so the post-pass
would have to re-run lane planning anyway — with less information than the
planner has natively, and with the boundary-record hazard that RFC-057's fold
work already paid for once (a relocated boundary whose record stayed put
produced 0.00/s in Factorio while validating clean).

Packing is gated behind a `LayoutOptions` flag, default off, until it clears
its gates — same discipline as `compact_layout`.

### Phase 4 design (2026-07-31): a packed geometry planner beside the bus planner, with an explicit refusal surface

Read of `plan_bus_lanes` (1,667 lines) and the ghost-pipeline contracts,
looking for kill criterion 5. The planner splits cleanly in two:

- **Aggregation (~60%, geometry-free, reused verbatim):** consumer/producer
  maps keyed `(item, module_id)`, DI-coupled consumer filtering, rate
  totals and belt-tier selection, lane splitting, family construction.
  Nothing here knows where anything is.
- **Geometry (1D-specific, NOT modified — paralleled):** contiguous west
  column strip (`x = i+1` + spacers), east-turning `tap_off_ys`, fluid
  anchor staggering, balancer blocks demanding contiguous columns plus
  `compute_extra_gaps` vertical slack, and the two-pass width negotiation
  with `place_rows`. These assumptions are load-bearing and correct for
  the linear bus; they are left untouched.

The packed path is a **separate geometry planner** (`bus::bands`), fed by
the same aggregation outputs, producing per-item **nets** instead of
column lanes: a source anchor (external items at the arrangement's west
edge; produced items at their band's output row) plus one tap anchor per
consumer band's feed rows — the spike's model, made real. Bands are
translated rigidly to their packed positions (the fold's per-segment
transform precedent), and routing reuses the position-agnostic machinery
the contracts promise: `Occupancy`, negotiated A\*, junction handling.
Splitter tap stamping and balancer blocks are 1D-trunk concepts and do
not carry over.

**The refusal surface is what keeps KC5 from firing.** Packing refuses —
and the decomposition candidate abstains, shipping the native bus — when
the arrangement needs anything the packed planner does not model: any
item over one lane's capacity (balancer families), `HorizontalStack`
rows, partitioned modules (`module_id > 0`), or a band census below the
packer's own floor. Every KC2-winning fixture is single-lane throughout,
so the refusals cost none of the measured reach; they cost only claims
the RFC never made. This is the same strictly-additive entry pattern
cell composition and DI used: compete where you can build, abstain
loudly where you cannot.

**Two implementation specifics, fixed before code (2026-07-31).** First,
phase 4 packs the **content rect**, not the structural rect: a band must
carry its fluid pipe rows (and any other non-belt span content) rigidly,
so the packing input is the span's y-range minus belt-only rows, with the
x-extent of all non-belt entities. The phase 0–3 instruments measured
structural rects (machines+inserters only) and their published numbers
stand; the content rect adds a small area cost on fluid-carrying bands
(`pu1-plate`'s chem rows), which the KC1 re-measure absorbs or reports.
Second, the integration seam is INSIDE `layout_pass`, immediately after
`place_rows` — the RFC's rejected-alternatives entry still holds: a
post-pass over the finished layout would re-run lane planning with less
information. When the flag is on and nothing refuses, the packed builder
takes over (translate bands, plan nets, route, place poles, emit
boundary records — the fold's 0.00/s lesson makes the records
first-class, not an afterthought); on any refusal it emits the typed
event and falls through to the untouched native pipeline.

KC5 verdict: **extension, not rewrite — the criterion does not fire.**
`plan_bus_lanes` is not modified; the packed planner sits beside it
behind the same flag, and the linear path stays byte-identical.

## Kill criteria

1. **Transport eats the gain.** If, after real trunk routing, the **bounding-box
   area** saving on `sci1-ore`, `sci2-ore` and `pu1-plate` is below **half** the
   probe's estimate for those same three fixtures — worse than **−33.0%**
   against an estimated **−66.1%** (10,729 → 3,641 tiles) — then
   2D trunk cost is consuming the reclaimed area and band granularity fails the
   same way machine granularity did. Stop; do not re-tune the packer.

   The baseline is the three-fixture aggregate, not the −64.5% four-fixture
   figure quoted in Motivation. Those differ because the four-fixture number
   includes `belt5-ore`, the weakest packer at −46.2%, which this criterion
   does not name — so reusing −64.5% here would set the bar ~1pp more lenient
   than the rule's own wording implies.

   The metric is bounding-box area, matching what the probe measured. It is
   deliberately **not** occupied tiles: bands are rigid, so machines and
   inserters cannot move relative to each other and only transport tiles can
   shrink. Against belts at ~39.6% of occupied tiles and a best-case ~38%
   transport saving, the achievable occupied-tile reduction is only ~15% — so
   an occupied-tile bar set at −32% would fire on every run regardless of
   whether the approach worked, killing the RFC spuriously.
2. **Reach is too narrow.** If fewer than **30%** of the e2e corpus has ≥3
   bands and no width-dominant band, the technique cannot pay for its
   complexity regardless of how well it works where it applies. Measure this
   in Phase 0, before writing the packer. *(Evaluated 2026-07-31: cleared at
   36.8%, cap-insensitive across 3:1–4:1 — see decision log.)*
3. **Correctness regression that isn't mechanical.** If packed candidates
   validate worse than their controls (new categories, or higher counts in any
   category) and the cause is not a mechanical record-relocation fix, stop.
4. **Throughput regression.** If any packed candidate loses more than **2%**
   fast-meter delivered target rate against its control, stop. Density may not
   buy a slower factory.
5. **Lane planner doesn't fit.** If 2D trunk planning requires substantially
   rewriting `plan_bus_lanes` rather than extending it, that is a signal the
   bus architecture assumes single-column stacking more deeply than this RFC
   credits — stop and re-scope rather than pushing through.

Criterion 1 is the one this RFC exists to test. The probe deliberately could
not measure it: it prices band-to-band distance, not trunk corridors, so the
−66.1% excludes the space trunks will consume. *(Evaluated 2026-07-31 by the
phase-3 spike: cleared at −35.9%, a 2.9-point margin — see decision log. The
criterion stays armed through phase 4; the spike's model is throwaway and
the real lane planner must re-clear it.)*

## Verification plan

Per the layout-engine protocol in [`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Full e2e suite green** — `cargo test --manifest-path crates/core/Cargo.toml`,
  all non-ignored tests.
- **Inertness gate** — with the flag off, output is byte-identical to today.
  A focused test asserts this. (The style example this line originally cited,
  `compact_layout_option_is_explicit_and_validated`, was deleted 2026-08-14
  along with `compact_layout` itself — #632 A2; `rfc058_band_packing_premise_holds`
  in `cell_composition.rs`, this RFC's own surviving CI guard, is the
  in-tree pattern to follow now.)
- **Per-fixture validation parity**, reported **per issue with a position**,
  never as a count in prose — `docs/validator-reporting.md` records nine
  instances of that failure mode, one of which concealed a total failure in
  the fold work.
- **Trace events** — packing emits its own event carrying band count, chosen
  target width, and achieved aspect, so a disappointing result is diagnosable
  without a debugger.
- **Fast meter** on `sci1-ore`, `sci2-ore`, `pu1-plate` against controls, for
  kill criterion 4.
- **Browser eyeball** on at least `sci1-ore` and `pu1-plate` — a zero-warning
  layout with visibly disconnected belts is a validator bug, not a success.
- **Factorio adjudication** for the finalist, once the meter agrees.
- The probe itself becomes a **regression test**: the estimate for the four
  winning fixtures must not silently drift.

## Phasing

0. **Reach measurement** — band census across the e2e corpus. Answers kill
   criterion 2 before any packer exists. Carries the aspect-cap sweep
   originally assigned to phase 2: the cap changes KC2's applicable-fixture
   count, so it belongs with KC2's measurement (reordered 2026-07-31, see
   decision log).
1. **Band extraction from `RowSpan`** — placer-native, replacing the probe's
   y-projection. No behaviour change. **Landed 2026-07-31** (`bus::bands`,
   after the phase-3 gate cleared per the reorder): spans are the grouping
   authority, geometry is measured from their own entities, and a parity
   test pins the result against the probe's y-projection oracle.
2. **Packer** — shelf packing with aspect cap and swept target width, behind a
   default-off flag. Emits positions only; nothing consumes them yet.
   **Landed 2026-07-31**: `LayoutOptions.band_packing` records the plan as
   a `BandPackingPlanned` trace event (or a typed refusal); an inertness
   test asserts entity-identical output with the flag on or off.
3. **Trunk spike — throwaway code, all three gate fixtures.** Route trunks for
   the packed layouts with whatever is quickest and measure real bounding-box
   area and real transport against each control. Start with `sci1-ore` (4
   bands, cheapest) as a smoke test, then `sci2-ore` (8) and `pu1-plate` (11).

   **All three are required before the gate is considered cleared.** Kill
   criterion 1 names all three, and the two larger ones are precisely the
   fixtures most likely to expose 2D-trunk cost — spiking only `sci1-ore` would
   defer that exposure to phase 4, i.e. past the lane-planner work this gate
   exists to protect against. An explicit area-vs-control comparison is part of
   this phase, not deferred to phase 5's meter run, which prices throughput
   (criterion 4) rather than area.

   This phase exists because phases 0–2 are cheap *and prove nothing*: a
   census, an extractor and a packer can all land green while saying nothing
   about whether trunks fit. RFC-057 front-loaded its cheap work the same way
   and its premise died in the expensive phase. The spike also settles whether
   the Manhattan transport proxy flattered packing — the one number in this
   RFC with a known directional risk.
4. **2D lane planning** — `plan_bus_lanes` for packed bands, properly. Only
   after the spike clears on all three fixtures. Where kill criterion 5 is
   evaluated.
5. **Validation, meter, and Factorio adjudication.**
6. **Default-on decision**, as a scored decomposition candidate rather than a
   forced path.

Phases 0–2 are cheap, land independently, and are not evidence. Phase 3 is the
gate: it is deliberately throwaway so that a negative result costs a day rather
than the lane-planner rewrite.

**Execution order (2026-07-31): 0 → 3 → 1 → 2 → 4 → 5 → 6.** The spike runs
on the probe's packed positions, so phases 1 and 2 are not prerequisites for
it — they are production scaffolding for phase 4, built only if the gate
clears. Rationale in the decision log.

## Relationship to earlier RFCs

- **RFC-057** supplies the negative result this RFC is built on: machine
  granularity is too fine, because it destroys the row's shared-belt
  efficiency. Its 2D placement machinery (`place_recipe_clusters`,
  terminal-inclusive packing) is directly reusable at band granularity.
- **RFC-055/056** established that macro reordering and contiguous folding are
  shape transforms, not density levers (~20% routing ceiling).
- **RFC-053** (direct insertion) is complementary and attacks a different
  category. From the 2026-07-30 density audit over the same ten fixtures,
  belts are 39.6% of *occupied tiles* and machines 38.4%, so removing belt
  segments and packing bands are additive, not competing. (Two coincidences
  worth naming so they are not read as copy errors: 38.4% is also the
  ragged-right share of *bounding-box area* quoted in Motivation — a different
  metric with a different denominator — and 39.6% happens to equal
  `pu1-plate`'s transport delta in the results table. Both pairs are genuine
  collisions of unrelated quantities.)
- Folding composes on top: bands give a squarer layout, and multi-fold (#500)
  can shape whatever remains.

## Decision log

- **2026-07-30 — premise measured before writing this RFC.** Decision rule was
  fixed in advance: proceed only if some packing variant saves ≥20% band-bbox
  while adding less rate-weighted transport than it saves. Result on the four
  packable fixtures was −46% to −69% area with transport *falling* 27–40%, so
  the rule passed far wider than required. Aggregate −64.5% area, −38.1%
  transport.

  Two corrections made during the probe, both recorded because they change how
  the number should be read. First, the initial run optimised bounding-box
  **area** alone and produced 176×5 / 219×5 "wins" — degenerate one-shelf
  ribbons, the exact shape `pu4-raw` already ships at 2752×90. Area is only a
  valid objective under an aspect cap; the headline numbers above are all
  under 3:1. Second, three fixtures reporting "no packing" were checked rather
  than assumed to be probe bugs: `gear15-ore` (144-wide band) and `lds2-plate`
  (121-wide band) are genuinely unpackable because the widest band floors the
  layout width, and `insert3-ore` is a threshold miss at 3.16:1 against a 3.0
  cap.

  Both figures are obstacle-free and exclude trunk corridor space, so −64.5%
  is an upper bound. That gap is precisely what kill criterion 1 tests.

- **2026-07-30 — scope fixed to multi-band layouts.** Wide-row splitting was
  measured the same day and falsified (−0.9% bbox at zero errors), so the
  width-dominant case is explicitly out of scope rather than deferred.

- **2026-07-30 — kill criterion 2 provisionally clears, on the probe corpus
  only.** Of the ten fixtures measured: four pack cleanly, one
  (`insert3-ore`) is a threshold miss at 3.16:1 that a 3.5 cap would admit,
  two (`gear15-ore`, `lds2-plate`) are width-dominant, and three are
  single-band. That is 40–50% applicable against a 30% bar.

  This is **not** Phase 0. The probe corpus is ten hand-picked fixtures, not
  the e2e corpus, and it was chosen to span layout shapes rather than to be
  representative of what users request. Phase 0 still runs the census properly
  before the packer is written; this entry records that the early signal is
  favourable, not that the criterion is settled.

- **2026-07-30 — kill criterion 1's measured quantity changed from occupied
  tiles to bounding-box area.** Recorded separately because it is a
  consequential decision, not a wording fix, and CLAUDE.md requires those to
  live in the owning RFC's decision log.

  As first written the criterion set a −32% bar on *occupied-tile* saving
  against a −64.5% baseline that is bounding-box *area*. Bands are rigid, so
  machines and inserters cannot move relative to each other and only transport
  tiles can shrink; against belts at ~39.6% of occupied tiles and a best-case
  ~38% transport saving, achievable occupied-tile reduction is only ~15%. The
  bar would therefore have fired on every run regardless of whether the
  approach worked — a false-negative gate on the criterion this RFC exists to
  test. Changed to bounding-box area, matching the probe.

  Four other defects landed in the same pass, all internal inconsistencies
  rather than decisions: the Summary defined a band as including belt rows
  while the Design section excluded them; the width-floor argument claimed a
  wide band floors *aspect* when it floors *width*; "every band is exactly 5
  tiles tall" was generalised from the three fixtures whose dimensions had
  been printed, all of which were the failing ones (`belt5-ore` and `sci2-ore`
  each carry a 7-tall band); and 38.4%/39.6% appeared for unrelated quantities
  without attribution.

- **2026-07-30 — self-review after the bot pass; four presentation and
  method defects fixed.** None change the measured result; all change how it
  should be read.

  The headline aggregate was over the four fixtures that pack — selection bias
  if read as a corpus number. Corpus-wide across all seven multi-band fixtures
  is **−39.6%**, now the figure the RFC leads with.

  The transport saving rests on a band-centre Manhattan proxy. That is close to
  real routing for the stacked control (down a trunk, across to a row) but not
  necessarily for a packed layout, so it may distort the two cases unequally
  and flatter packing. Recorded as the weakest number in the RFC; the area
  result does not depend on it.

  The wide-band scope-out was overstated. `max_per_row` capping was falsified,
  but that is one reshaping, not the whole space — two machine rows sharing a
  belt row was never tested. Softened to "one obvious fix doesn't work".

  Phasing was restructured: the original phases 0–2 were all cheap and all
  evidence-free, exactly the shape that let RFC-057 run until its premise died
  in the expensive phase. A throwaway **trunk spike** on all three gate
  fixtures is now phase 3 and carries kill criterion 1, so a negative result
  costs a day rather than the lane-planner rewrite.

  The 3:1 aspect cap is acknowledged as arbitrary and consequential — it alone
  refuses `insert3-ore` at 3.16:1 — with a sweep assigned to phase 2.

- **2026-07-31 — band structure is host-geometry-relative; corpus aggregates
  must be read like stress goldens.** Re-running the committed phase-0
  instrument on a second machine, at the same commit with no core changes in
  between, reproduced the three gate fixtures and the winners aggregate
  exactly (−66.1% KC1 baseline, −64.5% winners, −38.1% transport) but
  extracted **3 bands from `ec10-ore` where the #510 run saw 1**, moving the
  corpus headline from −39.6% (7 multi-band) to −35.4% (8 multi-band).
  Layout geometry is already known to vary with host SAT-cache state — the
  reason stress goldens are never enforced in CI without pinning the cache —
  and band extraction inherits that. Consequences: the premise-drift guard's
  loose ≥50% bounds on the gate fixtures are the right shape (exact corpus
  pins would flap across machines); phase 0's census reports KC2 as measured
  on one named machine, not as a portable constant; and any KC2 result within
  a few points of the 30% bar must be re-run on a second machine before the
  criterion is called.

- **2026-07-31 — execution order changed to 0 → 3 → 1 → 2 (phases 1–2
  deferred behind the spike).** As circulated, the plan built the placer-native
  band extractor and the flagged packer before the trunk spike. Both are
  production scaffolding for phase 4, and the spike does not need them — it
  consumes packed positions the phase-0 probe already computes. Building them
  first would repeat, in miniature, the shape this RFC's own phasing section
  criticises: cheap green work stacked ahead of the phase that can kill the
  premise (RFC-057 died in its expensive phase after exactly that pattern).
  The aspect-cap sweep moves from phase 2 into phase 0 for the same reason:
  the cap changes KC2's applicable-fixture count, so it is part of KC2's
  measurement, not a packer tuning knob. Nothing about the gates changes —
  KC2 is still evaluated at phase 0, KC1 still at the spike, and all three
  gate fixtures are still required before phase 4 starts.

- **2026-07-31 — Phase 0 complete: KC2 clears at 36.8%, and the aspect cap
  turns out not to matter on the real corpus.** The census
  (`probe_band_census_e2e_corpus` in `crates/core/tests/cell_composition.rs`)
  transcribes every distinct production request exercised by a non-ignored
  test in `e2e.rs` — 38 rows; the inclusion rule lives in the test's doc
  comment. Two consequences worth naming. `pu1-plate`, a KC1 gate fixture,
  is **not** in KC2's denominator, because its owning e2e test
  (`pipe_belt_processing_unit_1s_routes`) is `#[ignore]`d. And the rule cuts
  the numerator too: `pu20-plates` — measured during the census as an
  85-band packable winner at −81% (145×735 → 166×124) — is excluded the
  same way, so it stands as evidence that reach extends beyond the counted
  corpus, not as part of the 36.8%. (The first transcription violated its
  own rule on 4 rows — three ignored stress tests and one ignored fixture
  source — and claimed completeness while missing expressible requests; two
  rounds of #516 review caught all of it, the second round by auditing
  every remaining non-ignored test. The numbers here are from the corrected
  38-row corpus.)

  All 38 rows built. 19/38 (50%) have ≥3 bands; **14/38 (36.8%) also pack,
  against the 30% bar**. The sweep (3.0 / 3.5 / 4.0) changes nothing — 14/38
  at every cap. The probe-corpus concern (`insert3-ore` refused at 3.16:1)
  does not generalise: on the e2e corpus every ≥3-band fixture either packs
  inside 3:1 or is width-dominant beyond 4:1. The 3:1 default stands and the
  sweep obligation from the reorder entry is discharged.

  Two honesty notes on the margin. First, one packable row (`ec10-plate`, 3
  bands, 30×15 control) packs at exactly ±0%; KC2's wording counts it, but
  excluding zero-gain rows gives 13/38 (34.2%) — the verdict does not hinge
  on it. Second, this is one machine's geometry (see the sensitivity entry
  above); the margin is ~2.6 fixtures, and the packable set is dominated by
  deep multi-band fixtures (8–23 bands) whose candidacy small band-count
  jitter cannot flip, so the second-machine re-run is not being demanded.

  Findings that shape later phases: width-dominance is the *entire* failure
  mode among ≥3-band candidates (5/19 — `ec20-ore`, `ac4-nauvis`,
  `ac5-nauvis`, `pu2-ore-hs`, `pu2.5-plates-hs`; widest bands 96–692 tiles,
  all smelter or assembler mega-rows), which sharpens the case for a
  separate wide-band RFC. Belt tier shapes candidacy more than rate does: `ac7-nauvis-yellow`
  (the yellow-capped AC@7) splits into 16 narrow bands and packs at −78%,
  while the uncapped ignored variant of the same request is a 169-wide
  monolith. In-corpus winners save −63% to −78% (`ac7-nauvis-yellow` 16
  bands, `pu2-ore-red` 23 bands at −77%), consistent with the probe corpus.
  And band extraction has a benign artifact — `ec10-plate` yields a 1-tall
  inserter-only band [(30,5), (20,1), (21,5)] — to keep in mind when phase 1
  makes `RowSpan` the source of truth.

- **2026-07-31 — Phase 3 trunk spike: KC1 clears at −35.9% against the
  −33.0% bar. Narrowly — and the margin's history is the real content of
  this entry.** The spike (`probe_trunk_spike_gate_fixtures`) packs the gate
  fixtures with the phase-0 packer, then routes every band-to-band item flow
  as a 1-tile A\* corridor: band rects opaque, perpendicular crossings
  allowed (the UG dive), same-axis overlap forbidden, corners opaque, global
  gap widening (2→8) on any failure. Score = real bounding box of bands +
  belt rows + corridors, against the control band-bbox — the same quantity
  as KC1's 10,729-tile baseline, which this machine reproduces exactly.

  The first model let a flow terminate on any tile touching its destination
  band, and it reported **−54.3%** with every fixture routing at gap 2. That
  number was discarded as too generous, not recorded as a pass: a band's
  machines are fed by full-width belt rows — the shared-belt structure the
  whole RFC is premised on — and a single-tile termination silently omits
  them. With `ceil(distinct inputs / 2)` feed rows above each band and one
  output row below reserved *before* any through-routing, sci2 needs gap 6,
  pu1 gap 4, and the result is:

  | fixture | control | packed+trunks | saving | gap | flows | belt-row tiles | corridor tiles |
  |---|---:|---:|---:|---:|---:|---:|---:|
  | `sci1-ore` | 990 | 672 | −32.1% | 2 | 6 | 102 | 55 |
  | `sci2-ore` | 4,104 | 2,618 | −36.2% | 6 | 14 | 301 | 428 |
  | `pu1-plate` | 5,635 | 3,584 | −36.4% | 4 | 20 | 603 | 368 |
  | **aggregate** | **10,729** | **6,874** | **−35.9%** | | | | |

  KC1 is defined on the aggregate; `sci1-ore` individually sits at −32.1%.
  Trunk cost consumed roughly half the obstacle-free −66.1% — the exact
  tolerance the criterion was written around.

  Model honesty, both directions. Conservative choices: same-axis corridor
  overlap is forbidden even for the same item (reality merges and splits);
  gap widening is global (one congested seam widens every shelf boundary);
  feed rows are sized per band with no sharing between vertically adjacent
  bands (reality could direct-sideload a neighbour's output row). Generous
  choices: poles and balancers are absent; lane-to-item assignment within a
  feed row is not checked (capacity is sized at two items per row but
  which-item-which-lane is unmodelled); fluids are priced as belt lanes;
  external inputs are assumed available anywhere on the west edge. Net
  lean is conservative, but the 2.9-point margin is thin enough that phase
  4's real lane planner must treat −33.0% as a live bar, not a cleared
  formality — kill criterion 1 stays armed through phase 4.

  Two router defects found by #517's review were fixed and re-measured
  before this entry's numbers were frozen: a corridor could TURN on a tile
  it was only entitled to cross (the corner check ignored pre-existing
  perpendicular occupancy — generous), and the A\* heuristic aimed at a
  hint point instead of the target set, making it inadmissible and the
  corridors non-minimal (conservative). The fixes move the aggregate
  −35.2% → −35.9% (`sci2-ore` 2,695 → 2,618; corridor tiles 454 → 428 and
  379 → 368): the shorter true-minimal corridors outweigh the stricter
  turn rule, so the verdict stands with a slightly wider margin.

- **2026-07-31 — phase 4 built end-to-end; KC1 on the real planner:
  buildable-fixture aggregate −44.0% against the −33.0% bar.** The packed
  pipeline (refusals → content-rect packing with the spike's 2..=8 gap
  widening → rigid translation → inserter-aligned belt rows → corridor
  routing with UG pairs at crossings → pole grid → boundary records) now
  ships real layouts behind the flag: `sci1-ore` 990 → 512 (−48.3%),
  `sci2-ore` 4,104 → 2,340 (−43.0%) — both beating the spike's estimates,
  which reserved worst-case feed rows the real templates do not need.
  Three router defects were caught by the tests and fixed: rect-blocking
  made interior feed rows unreachable, a single-tile pickup was killed by
  an earlier corridor's turn, and the fixed gap denied sci2.

  **Scope gap, recorded not hidden: `pu1-plate` refuses on the real
  planner** — the native pass is DI-free under the Candidate pattern, so
  copper-cable rides the bus at 81/s and trips the multi-lane refusal
  that DI normally removes. The three-fixture KC1 aggregate is therefore
  NOT evaluated on the real planner; the buildable-fixture aggregate
  (−44.0%) clears the bar with room, and closing the pu1 gap means
  packing the DI candidate's rows — future work this RFC's phase-6
  candidate wiring should inherit, not a silent re-basing of the gate.
  Probe: `probe_packed_kc1_real_planner`.

- **2026-07-31 — phase 5 opened: KC3 baseline measured, not yet parity.**
  Controls validate at 0 issues; packed `sci1-ore` carries 48 (21
  entity-overlap Errors, belt-dead-end, flow-reachability, throughput
  warnings) and `sci2-ore` 156 (item-isolation, belt-loops, UG pairing,
  power coverage among them) — `probe_packed_validation_parity` prints
  the per-category tables. These are mechanical geometry defects of the
  young builder (the class KC3 explicitly tolerates fixing, not the
  criterion firing), and each category now gets the verification
  protocol's treatment: snapshot-decode, tile-level inspection, fix,
  re-measure — never trusting a count drop alone. KC4 (fast meter),
  browser eyeball, and sim adjudication queue behind validation parity;
  phase 6's candidate wiring stays blocked until KC3 closes.

- **2026-07-31 — hardening loop, honest checkpoint: correctness rules are
  eating the density, and one "0 issues" was the trap, caught.** The
  foreign-feed legality rule (a tile a foreign-carrying belt points into
  is unroutable) fixed the diagnosed item-isolation class — and made a
  sci2 net unroutable at every gap, so the builder REFUSES sci2 and the
  parity probe's "sci2 packed: 0 issues" was the native fallback
  validating clean, not a fixed packed layout (the check-went-quiet
  failure mode CLAUDE.md documents; caught by re-running the KC1 probe,
  which detects refusal by entity identity). sci1 still builds at 30
  issues but its saving fell to −28.9% — BELOW the −33.0% bar. Current
  truth: no gate fixture both builds AND clears KC1 under the hardened
  router. The tension is structural: each legality rule the validator
  demands lengthens corridors or forces refusal, which is exactly the
  trade KC1 exists to police. Next iterations owe either smarter routing
  under the same rules (same-item merging instead of blanket overlap
  bans, per-item feed-row assignment making sideloads legal, per-seam gap
  growth) or the honest conclusion that KC1 fires on the real planner —
  neither is decided yet, and no number in this entry is a pass.

- **2026-07-31 — next-iteration direction fixed: per-net corridor TREES
  with splitter branches.** Of the checkpoint's two options, smarter
  routing is chosen over concluding KC1 fires — because the current
  router duplicates a full corridor per (src, dst) pair, which is where
  both the wasted area and the unroutability come from, and the bus's own
  answer to one-producer-many-consumers is a trunk with splitter
  tap-offs. Design: route a net's first consumer normally; every later
  consumer may terminate on any tile of the net's OWN earlier corridor,
  and the junction point becomes a SPLITTER (1→2, facing the trunk's flow
  direction) whose second output starts the branch — same-item merging
  stays forbidden across nets, so the foreign-feed rule and item
  isolation hold. Expected effects: sci2's unroutable net regains a path
  (it can join its own trunk), corridor tile counts drop materially, and
  KC1 gets its honest re-measure on the tree router. If THAT still lands
  below −33.0%, the criterion fires and the RFC stops — the option is
  spent after this iteration.

- **2026-07-31 — the tree-router iteration lands: sci2 builds again,
  KC1's buildable aggregate clears at −34.6%, and sci1 validates
  ERROR-FREE.** Five routing advances, each from a tile-level diagnosis:
  splitter-carved branch junctions; feed-row west continuations;
  negotiated net ordering (failing net promoted, one promotion per net
  per gap) which is what actually unlocked sci2; multi-producer
  collectors (the producer map kept ONE band per item — every other
  producer's output row stranded); and immediate-continuation pickups
  (seeding at ox−2 left a one-tile hole rows dead-ended into). State:
  sci1 builds at −28.9% with 16 warnings and ZERO errors; sci2 at −36.0%
  with 153 issues (4 dead-ends, 1 isolation, rest warnings); buildable
  aggregate 5,094 → 3,331 (−34.6% vs the −33.0% bar). pu1 still refuses
  on multi-lane scope. The KC1 tree-router condition from the previous
  entry is MET on the buildable set; KC3 remains open (sci2 errors,
  warning classes on both) and the loop's next diagnoses are queued in
  the probes.

- **2026-07-31 — open diagnosis, recorded for pickup: sci2's four
  dead-ends are orphaned BRANCH stubs.** Neighborhood dumps
  (`probe_packed_overlap_diagnosis`, now fixture-switchable via
  `SPAGHETTIO_DIAG_FIXTURE=sci2`) show the pattern at (8,11): a gear
  trunk runs north at x=7 with its carved splitter at (7,12); the branch
  entry (8,11) is stamped East pointing into an EMPTY (9,11) while the
  branch's continuation exists at (10,11) — a one-tile gap exactly where
  the branch crossed the x=9 corridor, i.e. the crossing-to-UG
  conversion did not fire for the branch's first crossing. Same class at
  (26,27) and two south-pointing west-margin stubs. TWO hypotheses now
  measured and falsified: the trailing-run sideload assumption (west-
  continuation follow-through fill landed, no delta) and foreign-goal
  acceptance (free-or-own-item goal rule landed, no delta — kept as
  guards, both physically required). The class survives blind iteration;
  next session opens the SNAPSHOT DEBUGGER on the sci2 packed layout
  (SPAGHETTIO_DUMP_SNAPSHOTS=1 + the decoder in
  docs/layout-snapshot-debugger.md) and walks the four stub tiles with
  full entity context — then sci2's last isolation error, the warning
  classes (reachability 63, power 49), the pu1 scope decision,
  KC4/eyeball/sim, and phase 6's candidate wiring.

- **2026-07-31 — KILL CRITERION 1 FIRES on the real planner; RFC-058
  concludes.** The final materialization-correctness fix (the UG
  entrance conversion must target the actual predecessor belt, not
  `out.last_mut()` — the corruption the instrumented route dumps
  exposed) forces legal-but-longer routes, and two rounds of #523 review
  then corrected the MEASUREMENT itself: the packed bbox was anchor-only
  with no minimum (understating), the pole grid drifted up to ~6 tiles
  past the last real entity (inflating, and outside the criterion's
  scope — the spike placed no poles), and a scoring flip could ship a
  native-shaped K1 variant mislabelled as packed, so the flag now
  bypasses candidate scoring outright (an instrument, not a candidate).
  On the faithful instrument — packed artifact, criterion-scope
  non-pole extents, honest footprints — the buildable-fixture aggregate
  lands at **−27.0% (sci1 −27.3%, sci2 −26.9%), six points below the
  −33.0% bar**. The trajectory across the hardening loop remains
  adverse — −44.0% (naive, corrupt routing) → −34.6% (tree router,
  still-corrupt materialization) → −27.0% (legal materialization,
  faithful measurement) — with
  KC3 parity still distant (sci2: 5 dead-ends, 1 isolation, 63
  reachability warnings), so every remaining correctness repair can only
  push density further below the bar. The tree router was pre-committed
  as the LAST routing option ("the option is spent after this
  iteration"), and the criterion's own text says stop; do not re-tune.

  What stands: phases 0–3's measured evidence (KC2 36.8%; the spike's
  −35.9% on a model now known to be generous about materialization
  legality); the inert extraction/packer scaffolding on main; and this
  branch's packed pipeline as the falsification instrument. What is
  falsified: that 2D band packing can hold ≥33% real-bbox saving under
  physically-legal single-lane trunk routing on the gate fixtures. The
  phase-6 default-on question resolves NO by this evidence; phases 5–6
  are closed by the kill, per the phasing section's own gate structure.
  The flag and packed builder remain in-tree, default-off and inert, as
  the reproducible record. Known latent defects in that record — found
  by #523's two review rounds and deliberately left, since each fix
  makes routing stricter or restores dropped transport and so moves
  density further below the bar: the splitter carve can silently no-op
  (re-selected junction, or a junction whose belt a crossing bridger
  already renamed to UG); the collector loop lacks crossing/UG handling
  and the foreign-feed filter; secondary/sorted output-belt rows are not
  re-stamped (no gate fixture carries them); `src_bands` self-exclusion
  is evaluated against the first consumer only; and the sketch pole grid
  can exceed wire reach after free-tile drift (clamped to the pre-pole
  extent, but not reach-verified). Each is annotated at its site in
  `bus::bands`.

- **2026-07-31 — phase 4 designed; kill criterion 5 evaluated and does not
  fire.** `plan_bus_lanes` divides into geometry-free aggregation (reused
  verbatim) and 1D-specific geometry (left untouched; paralleled by a
  packed-geometry planner in `bus::bands` producing per-item source/tap
  nets in packed coordinates, routed by the position-agnostic
  Occupancy/A* machinery). The load-bearing scope decision: the packed
  candidate REFUSES — abstains, native bus ships — on anything it does
  not model: multi-lane items (balancer families), HorizontalStack rows,
  partitioned modules. Every KC2 winner is single-lane, so the refusals
  cost none of the measured reach. Full design in the Phase 4 design
  section; implementation follows it behind the existing flag, with KC1
  re-measured on the real planner before any phase-5 gate runs.

- **2026-07-31 — phases 1–2 landed: placer-native bands and the flag-gated
  packer, both inert by construction.** New module `bus::bands`. Extraction
  takes the placer's `RowSpan`s as the grouping authority — each band
  records the row indices that contribute to it, which is the linkage
  phase 4's lane planner needs — while geometry is measured
  footprint-aware from those spans' own machine and inserter entities.
  Structural runs are collected within each span's y-range and merged
  across spans when they touch (direct-insertion fusions), which
  reproduces the probe's maximal-run semantics exactly. The probe keeps
  its deliberately decoupled y-projection as the oracle;
  `rfc058_placer_bands_match_y_projection` pins band rects, packed
  dimensions, AND planned positions against it on the three gate fixtures
  plus `gear15-ore` (a refusal case) — exact agreement. The packer is the
  probe's shelf packer ported verbatim; the strict-`<` first-minimum
  tie-break is documented as part of the published-numbers contract.

  `LayoutOptions.band_packing` (default off) gates one call at the end of
  `layout_pass`: extract, pack, and emit `BandPackingPlanned` (band rects,
  control/packed dims, aspect, positions) or a typed `BandPackingRefused`.
  Positions live in the trace and nowhere else — phase 2's "emits
  positions only" is structural (the seam borrows everything immutably),
  and `band_packing_option_is_inert_and_traced` asserts entity-identical
  output flag-on vs flag-off. Candidate variants' inner runs do not
  re-emit (same discipline as `compact_layout`'s inner-opts handling), and
  the wasm surface pins the flag off until phase 4 gives it a visible
  effect. Three intentional copies of the band/packing logic now exist —
  engine (`bus::bands`), probe (frozen instrument), CI premise guard
  (self-contained by design) — with the parity test and the guard holding
  them together; consolidation would couple the oracle to the thing it
  checks.

- *2026-08-20 — DELETED FROM TREE (owner call, offpath-code-followups
  Tier 2, PR pending): the flag-gated `bus::bands` builder, its
  `band_packing`/`band_pack_selection` `LayoutOptions` seam, the
  `BandPackingPlanned`/`BandPackingRefused` trace events, the
  `rfc064_packed_router.rs` gates, and the RFC-058/RFC-064-P3 probe
  section in `cell_composition.rs` (~4.4k lines total). Rationale: the
  2026-07-31 close-out kept the builder as "the reproducible record",
  and all three consumers subsequently arranged for it — this RFC's own
  phase 4, RFC-063 Phase C, RFC-064 Phase 3 — have since run and
  concluded/failed, so the 2026-08-20 golden-path audit put it to the
  owner, who extended the #632 A2 precedent (which deleted the sibling
  retracted spike `bus::compaction` outright). THIS decision log is now
  the falsification record; the KC1 numbers, phase verdicts, and the
  code itself remain recoverable from git history at this entry's
  date.*
