# RFC-058: Band packing — 2D placement at row granularity

Registry: [`rfcs.md`](rfcs.md). Status: **Design (circulated for review)**.
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
   in Phase 0, before writing the packer.
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
−66.1% excludes the space trunks will consume.

## Verification plan

Per the layout-engine protocol in [`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

- **Full e2e suite green** — `cargo test --manifest-path crates/core/Cargo.toml`,
  all non-ignored tests.
- **Inertness gate** — with the flag off, output is byte-identical to today.
  A focused test asserts this, in the style of
  `compact_layout_option_is_explicit_and_validated`.
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
   criterion 2 before any packer exists.
1. **Band extraction from `RowSpan`** — placer-native, replacing the probe's
   y-projection. No behaviour change.
2. **Packer** — shelf packing with aspect cap and swept target width, behind a
   default-off flag. Emits positions only; nothing consumes them yet.
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
