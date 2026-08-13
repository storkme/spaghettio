# RFC-068: Multi-group stamping — celldb fragments as row bands

## Summary

RFC-067's Phase 3 reopening condition was met on 2026-08-12 (two
sim-anchored community donors pass the never-worse floor and win their
composites at 18–22× the tie epsilon — decision log, fresh-gate entry).
The recorded resumption path is multi-group stamping "under a plan of its
own"; this is that plan. The mechanism: **in-place row-band
substitution** — inside `place_rows`'s per-group row loop, a machine
group whose demand matches a store entry stamps the stored fragment at
the group's existing `(x = bus_width, y_cursor)` slot and synthesizes the
same `RowSpan` contract (input belt ys, output belt y/x-extent,
`output_east`) that the native row templates would have produced there.
The lane planner and ghost router consume only `RowSpan`; they never
learn the interior came from a stamp. Targets are the two priced prizes
from the hotspot scoreboard
([`hotspot-scoreboard-2026-08.md`](hotspot-scoreboard-2026-08.md)):
advanced-circuit (8,049 pooled overhead tiles, 16.1/machine — double the
smelters) and electronic-circuit (3,106 tiles, donors exist as fused
cable→circuit blocks, which is what the store's dormant `Motif::Fused`
was reserved for). Stamping stays inert (candidate_runner only) and
selection stays untouched throughout — the standing K67-4 discipline,
inherited verbatim as K68-4.

## Motivation

- **The reopening evidence says composition is where donors pay.** At
  matched single-group demand, engine seeds tie the engine (K67-3 NULL,
  3 of 3 realizable motifs) — but community donors win on *shape*
  (composites +0.366/+0.447: the winners are 74×13 and 19×57, an order
  of magnitude closer to square than the engine's 144×7 strip), and
  shape value compounds inside multi-group layouts where ragged
  row-width variance is 55.2% of whitespace. The single-group
  harness could measure a donor's cell; only multi-group stamping can
  measure what a donor does to a *layout*.
- **The prizes are multi-group by construction.** `advanced-circuit`
  never appears as a single-group solve (3–5 groups on the corpus
  fixtures: ac co-solves with electronic-circuit and copper-cable, plus
  plastic-bar/petroleum-gas from plates); `electronic-circuit` co-solves
  with copper-cable everywhere. The v1 template candidate refuses both
  today (`template_candidate.rs`: single-group refusal, with its own
  regression test).
- **The two winning copper-plate donors are already in the store** with
  demand-matched fixtures; their sim anchors (30.1/s vs 29.99 planned,
  32.1/s vs 32.49) are recorded in RFC-067's decision log, while the
  store rows themselves still read `sim_anchor: "unanchored"` — the
  field flips only with a recorded anchor run, which is P3 bookkeeping
  this RFC inherits, not a fact it may pre-claim. Band substitution is
  what makes those rows (and every future donor) reachable from real,
  multi-group demand rather than only from the synthetic matched-demand
  fixtures.

## Design

### Mechanism: `RowSpan` substitution, not relocation

`place_rows` (`crates/core/src/bus/placer.rs`) walks machine groups in
dependency order and emits one `RowSpan` per band: `y_start`/`y_end`,
`input_belt_y` per input item, `output_belt_y` +
`output_belt_x_min`/`_max`, `output_east` (final east-flowing band vs
intermediate west-flowing producer band), fluid port fields, and
`row_width`. `route_bus_ghost`'s `row_exit_origin` and
`plan_bus_lanes`'s tap-off derivation read only these fields.

The stamp path, per eligible group:

1. **Lookup** — dominance-filtered store query at the group's ceil'd
   count (exact-count only in this RFC; count ladders are a recorded
   follow-up, below). Belt-tier admissibility identical to the v1
   candidate's surface-tier mapping (unknown transport names refuse).
2. **Adapter** — map the entry's declared ports to `RowSpan` fields:
   west-edge `belt-in` ports → `input_belt_y` (one per declared input
   item, multi-port per item allowed per the amended contract);
   `belt-out` port edge must match the slot's role (east edge for a
   final band, west edge for an intermediate band) → `output_belt_*` +
   `output_east`; pipe ports → the fluid fields. Two obligations the
   field list alone doesn't show (both from this RFC's review round):
   - **`input_belt_y` ordering**: the lane planner indexes
     `input_belt_y[input_idx]` by the row spec's *input schedule*
     order, not by port declaration order — the adapter orders by the
     group's schedule and any item it cannot place at its schedule
     index is a refusal, because a misordering misroutes taps silently
     with no Error.
   - **`output_feed_x_min`**: the adapter derives the fragment's drop
     coverage onto its output run and **branches on it**. Coverage
     continuous from the run's start (the ordinary-row shape — which is
     what engine seeds are) → `None`, exactly what `place_rows` sets,
     so P0 self-stamps stay native-identical. Discrete drop columns
     (the DI-cell shape a community donor may carry) →
     `Some(rightmost drop column)`; `None` there claims continuous
     coverage and reproduces the structural-cap bug the field exists
     to prevent (a bridge upstream of the last drop permanently misses
     later drops' share — validator-clean, meter-visible; see the
     field's doc in `placer.rs`). A fragment whose drop structure
     cannot be derived at all is a refusal.

   Beyond these, any port the adapter cannot map, or any `RowSpan`
   field left unfilled, is a **refuse-on-ambiguity** for that entry
   (the K67-1 discipline; a refusal is recorded, never worked around
   inline).
3. **Stamp** — entities translated to the slot origin, occupancy
   including splitter second tiles and direction-aware dims (the v1
   candidate's hardened passes, reused not rewritten); `y_cursor`
   advances by the fragment height; poles remain LAST, placed by the
   normal `place_poles` pass over the whole layout.

No downstream **code** changes — the lane planner and ghost router run
unmodified; they replan geometry from the stamped fragment's footprint
and belt positions exactly as they would for any row whose width or
height differed (a stamp does change `row_width` and band height, so
routes differ; what never happens is fabric being re-derived around
*relocated* bands). That is the load-bearing distinction from the five
adjudicated packing deaths (prior-adjudication map in the hotspot
scoreboard): RFC-057/058 **relocated** bands and paid 6–8× logistics
re-routing them; this mechanism substitutes an interior at the band's
native slot in the row sequence.

### Orientation is resolved at storage time, never at stamp time

Community donors naturally declare west-in/east-out ports — the *final
band* orientation. Both headline prizes are final bands in their own
fixtures (ac in ac solves, ec in ec solves), so this RFC needs **no
geometric transforms at all**: an entry whose output edge does not match
the slot role is inadmissible for that slot, full stop. Stamp-time
mirroring is rejected — a fragment IS its verified entities, and
mirrored belt geometry has its own lane semantics (splitters are
directional; the (n,m)≠(m,n) balancer lesson). If intermediate-band
stamping is ever wanted (smelter donors inside chains), the path is
mirrored *variants stored as their own entries*, re-verified by
`check_entry` and re-sim-anchored — recorded here as the follow-up
shape, out of scope for this RFC.

### Fused stamping: two groups, one fragment

`Motif::Fused` exists in the schema but is dead — no `query_fused`, zero
fused entries, `query_unit` filters the variant out. The ec prize
requires it: green-circuit donors exist almost only as fused
cable→circuit blocks (Phase-0 community mining). Scope:

- `query_fused(recipe_a, recipe_b, count_a, count_b, allowed)` beside
  `query_unit`, same dominance filter.
- The donor translator (`celldb_donor`) accepts fused specs: ports are
  the *pair's external* interface only (plates in, circuits out); the
  internal cable→circuit flow never crosses the contract and is verified
  by `check_entry`'s carry derivation like any other interior belt.
- The stamp path recognizes an adjacent producer+consumer group pair
  (the existing DI coupling derivation identifies exactly these) with a
  matching fused entry and substitutes one fragment for both bands,
  synthesizing a single `RowSpan` whose inputs are the pair's external
  inputs. `RowSpan.spec` is strictly single-recipe, so the fused span
  follows the **DI-cell convention** the placer already has: the
  consumer's spec owns the span (DI cells are keyed by the consumer for
  the same reason — all external inputs must already be available at
  its slot), and producer-side external inputs ride the same
  `RowSpan`-level mechanism DI uses (`di_input` and the cell fields);
  P2's code PR pins the exact field set against the DI implementation
  rather than this doc restating it. This is the "multi-band cell"
  RFC-053 deferred as its own (still-open) Phase 3 — the DI coupling
  map's un-split refusal names it in-code — built here as a stamp of
  verified stored geometry, not as computed straddle geometry.

### What is deliberately out of scope

- **Count ladders** (inexact-count stamping) — RFC-067's other named
  reopening lever. Exact-count refusal stays; the adjudication fixtures
  are demand-matched, and the follow-up is recorded as a checklist item
  on the tracking issue (#629), not smuggled in.
- **Port inference from wild blueprints** — the standing v1 gap; donor
  ports remain hand-declared, machine-verified.
- **Intermediate-band stamping / mirrored variants** — above.
- **Any selection change** — the candidate ships inert in
  `candidate_runner`; promotion is a separate, later, sim-gated
  decision (K68-4).

## Kill criteria

- **K68-1 (self-stamp fidelity — runs first, before any donor work):**
  Phase 0 stamps the store's own *engine-derived* seeds back into the
  multi-group fixtures they were extracted from (`seed_sources()`:
  ec@20-from-ore and ac@4-from-plates), covering **both band roles** —
  an intermediate west-flowing band (e.g. copper-plate or copper-cable
  in the ec fixture) and a final east-flowing band (ec or ac in its own
  fixture) — so both adapter arms are exercised. (A fused self-stamp is
  impossible in P0 — the store holds zero fused entries until P2; the
  fused mechanism gets its own differential control there.) The probe
  selects fragments **by provenance (`engine@…`), not by the normal
  query**: copper-plate@48 is a recorded key collision where the
  dominance sort resolves to the community donor (753 < 817 interior
  tiles, the pre-registered rule in `celldb_template.rs`), and a
  query-driven "self"-stamp would silently stamp the donor and void
  the isolation premise. Error-parity is also stated with its limit:
  it cannot catch the `output_feed_x_min` class (a throughput defect,
  not an Error) — that obligation is verified by P2/P3's meter and sim
  instruments, not by this gate. The fragment is what `place_rows` would have
  emitted, so this isolates the mechanism from donor quality. If a
  self-stamped layout cannot reach **validator Error-parity with the
  native layout** on any probe fixture — excess Errors attributable to
  the stamp seam (router-to-port reachability, seam belts, power) — the
  in-place substitution premise is wrong: stop before building any
  product mechanism, and record what the seam broke.
- **K68-2 (adapter expressiveness):** if mapping ports→`RowSpan` across
  the Phase-0 seeds plus the Phase-2 harvest requires **more than one
  escape hatch in total** (any per-entry special case or hand-resolved
  ambiguity — K67-1's bar, reused verbatim), the port contract lacks
  the vocabulary this RFC needs — stop and amend the contract as its
  own decision before continuing. Honest weighting, from this RFC's
  review round: the P0 half is near-trivially clean by construction
  (engine-seed ports were extracted from the very `RowSpan` semantics
  the adapter targets), so the criterion's real information is in the
  donor half — **the first translated donor adjudicates it before the
  rest of the harvest is funded**, so a contract-narrowness stop costs
  one translation, not a full harvest. A contract-narrowness stop is
  its own verdict and does not count toward K68-3's denominator.
- **K68-3 (value at the prizes):** after a documented Phase-2 harvest
  (target ≥3 translatable donors across advanced-circuit and fused
  cable→ec; fewer only if the harvest cannot produce 3, shortfall
  recorded), if **no multi-group corpus fixture prefers a stamped
  layout** under the never-worse floor with composite > +0.02
  (`COMPOSITE_TIE_EPSILON`, the donor-probe gate's constants) — Phase 3
  parks again, this time with the composition thesis itself adjudicated,
  not just single-group density. Scope of a PASS, stated to prevent
  over-reading: K68-3's fixtures are demand-matched, so a pass
  establishes **donor value under composition** — it does not establish
  that stamping fires on pre-existing corpus layouts, whose group
  counts will essentially never coincide with a store count under
  exact-count lookup. Deployment reach is exactly the count-ladder
  follow-up, tracked durably as a checklist item on #629; a K68-3 pass
  funds it, not a default flip. Calibration note on the bar itself: the
  +0.02 epsilon is the pre-registered donor-gate constant and stays,
  but the reopening evidence cleared it by 18–22× — a pass that merely
  scrapes the epsilon is weak evidence for the composition thesis and
  should be reported as such, not rounded up to vindication.
- **K68-4 (standing constraints, inherited verbatim):** belt tier is a
  user constraint, never a search axis — an entry exceeding the caller's
  tier is inadmissible by construction. Stamping ships inert; no
  selection influence without sim anchors; any meter-below-plan reading
  firewalls selection. Restated so this RFC cannot be read as relaxing
  them.

## Verification plan

Per the CLAUDE.md layout-change protocol:

- **Full e2e suite green at every phase**; WASM build green.
- **Engine-corpus control**: with stamping disabled (its default), all
  stress goldens and fixture verdicts byte-identical to origin/main —
  asserted in each phase's PR, the #626 precedent (pinned SAT zone
  cache for the host-drifting goldens).
- **Phase 0**: self-stamp differential — entity-level diff of stamped
  vs native band interiors (expected: identical modulo entity id/order),
  full-layout validator verdict diff (Error-parity bar), snapshot decode
  of the seam tiles on at least one fixture.
- **Phase 2/3**: the candidate_runner harness verbatim from the donor
  probe — `Policy::fold()` never-worse floor, `score_vs_native`
  composite at 0.5/0.5 weights, one demand-matched multi-group fixture
  per donor. **Meter `check_one` on every fixture where a stamp wins**
  (refutes cheaply); **sim anchor at matched demand for any winner**
  before the win is claimed (the only clearing instrument). Note the
  standing caveat: floor verdicts that lean on the walker's pooled-pair
  pickup credit are model-circular until sim-grounded (#627 tracks the
  pair-debit modeling; the fresh-gate entry records the precedent).
- **Trace**: stamp events land as trace events (fragment id, slot,
  refusals with reasons) so snapshot forensics can see which bands are
  stamps — one positioned event per stamp/refusal, never counts in
  messages (validator-reporting.md rules).

## Phasing

- **P0 — self-stamp probe (no product code):** probe/example harness
  that performs the substitution for engine seeds on their source
  fixtures; adjudicates K68-1 and the unit half of K68-2. Cheap,
  falsifiable, first.
- **P1 — mechanism:** the stamp path in `place_rows` behind an inert
  `DecompositionCandidate` (registered in `candidate_runner` only);
  ports→`RowSpan` adapter with refuse-on-ambiguity; trace events;
  byte-identical controls.
- **P2 — fused + harvest:** `query_fused`, fused donor translation, the
  fused stamp over DI-coupled pairs; harvest advanced-circuit donors
  (10 engine-legal candidates in the Phase-0 community corpus) and
  fused cable→ec donors; demand-matched multi-group fixtures; store
  regeneration (geometry-only, module payloads stripped — the ON0
  lesson).
- **P3 — adjudication:** K68-3 under the harness; sim anchors for
  winners; close-out entry in this log and RFC-067's. Default-on
  promotion is explicitly NOT this RFC's scope regardless of outcome.

Each phase is a separate PR; P0 may share a PR with this doc's
registry row if it stays within the size norm.

## Decision log

- *2026-08-13 — opened, on RFC-067's fresh-gate adjudication (Phase-3
  reopening condition MET, 2026-08-12). Mechanism chosen: in-place
  `RowSpan` substitution at the band's native slot — explicitly NOT
  relocation-and-reroute, the shape all five prior packing attempts
  died on (prior-adjudication map: hotspot scoreboard). Orientation
  resolved at storage time (no stamp-time transforms; splitter
  directionality). Self-stamp fidelity chosen as the Phase-0 gate so
  the mechanism is adjudicated before any donor is harvested.*
- *2026-08-13 — review round (second-opinion bot on #628), six findings
  verified against source and absorbed rather than argued: (1) the
  adapter owes `output_feed_x_min` derivation for every stamp — a
  stored fragment's output belt is drop-fed at discrete columns, the
  DI-cell shape, and `None` reproduces the structural-cap bug the
  field exists to prevent; (2) `input_belt_y` must be ordered by the
  spec's input schedule, misordering refused; (3) P0 selects seeds by
  provenance because the copper-plate@48 key collision resolves the
  normal query to the community donor; (4) K68-2's information is in
  its donor half — first translation adjudicates it before the harvest
  is funded; (5) K68-3's pass scope pinned to donor-value-under-
  composition, not deployment reach; (6) the motivation's sim-anchor
  claim corrected — anchors live in RFC-067's decision log, store rows
  are `unanchored` until a recorded anchor run.*
- *2026-08-13 — second review round (all 1/3-pass minors) absorbed:
  `output_feed_x_min` derivation branches continuous→`None` /
  discrete→`Some(rightmost drop)` — the first round's blanket
  never-`None` rule would have made P0 self-stamps diverge from native
  rows; the fused span pinned to the DI-cell convention
  (consumer-owned spec, `di_input`-style external inputs, exact fields
  pinned in P2's code PR); "nothing downstream changes" rephrased to
  code-unchanged/geometry-replans; donor aspect stated numerically
  (74×13, 19×57) instead of "near-square"; RFC-063 dropped from the
  registry row's death enumeration (its Phase-C spike cleared its bbox
  bar — it died on other grounds); follow-ups given a durable home:
  tracking issue #629, count ladders and mirrored variants as
  checklist items; K68-3 calibration note added (an epsilon-scraping
  pass is weak evidence, the reopening cleared the bar 18–22×).
  Tracking: work proceeds under #629.*
