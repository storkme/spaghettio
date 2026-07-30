# RFC-058: Band packing — 2D placement at row granularity

Registry: [`rfcs.md`](rfcs.md). Status: **Design (circulated for review)**.

## Summary

Place machine **row bands** in two dimensions instead of stacking them in a
single left-aligned column, and re-route the trunk taps that serve them.

A band is a maximal run of rows containing machines or inserters — one recipe's
machine row plus its inserter and belt rows. Today the placer stacks bands
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

Transport *falls*. That is the opposite of RFC-057's result and the reason this
is worth building: the control stacks bands in dependency order, so a consumer
band can sit far in y from its producer, and squaring the arrangement shortens
exactly those links. The area saving and the transport saving come together
rather than trading off.

### Where it does not apply

The probe also characterised the failures, and they are structural rather than
tuning problems. **Every band is exactly 5 tiles tall**; only widths vary
(measured range 6–144). So:

- **Fewer than ~3 bands** — nothing to pack. `ec10-ore`, `ec15-plate`,
  `gear5-plate` have one band each.
- **One band dominates the width** — the widest band is a hard floor on layout
  width, so no packing can be squarer than that band's own aspect.
  `gear15-ore` has a 144-wide band, `lds2-plate` a 121-wide one; the closest
  packing for either is ~10:1 and correctly refused.

This RFC does **not** address the dominant-wide-band case. Splitting wide rows
was measured separately on 2026-07-30 and falsified: capping machines per row
gave −0.9% bbox at zero errors (−5.3% only by introducing 10 errors), because
each sub-row re-pays its own belt overhead. That is a different problem and
wants its own RFC if it is ever worth solving.

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
- a **fixed inter-band gap** for trunk/tap space;
- an **aspect cap**, so the optimum is not a degenerate one-shelf ribbon.

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

1. **Transport eats the gain.** If, after real trunk routing, the occupied-tile
   saving on `sci1-ore`, `sci2-ore` and `pu1-plate` is below **half** the
   probe's estimate (i.e. worse than −32% against an estimated −64.5%), then
   2D trunk cost is consuming the reclaimed area and band granularity fails the
   same way machine granularity did. Stop; do not re-tune the packer.
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
−64.5% excludes the space trunks will consume.

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
3. **2D lane planning** — `plan_bus_lanes` for packed bands. The load-bearing
   phase, and where kill criteria 1 and 5 are evaluated.
4. **Validation, meter, and Factorio adjudication.**
5. **Default-on decision**, as a scored decomposition candidate rather than a
   forced path.

Phases 0–2 are cheap and land independently. Phase 3 is the risk.

## Relationship to earlier RFCs

- **RFC-057** supplies the negative result this RFC is built on: machine
  granularity is too fine, because it destroys the row's shared-belt
  efficiency. Its 2D placement machinery (`place_recipe_clusters`,
  terminal-inclusive packing) is directly reusable at band granularity.
- **RFC-055/056** established that macro reordering and contiguous folding are
  shape transforms, not density levers (~20% routing ceiling).
- **RFC-053** (direct insertion) is complementary and attacks a different
  category — belts are 39.6% of occupied tiles against machines' 38.4%, so
  removing belt segments and packing bands are additive, not competing.
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
