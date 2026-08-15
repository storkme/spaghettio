# What `PlacedEntity::rate` means

**Status (2026-08-07):** settled. Supersedes the open question in
[`handoff-e-rate-semantics.md`](handoff-e-rate-semantics.md).

## The answer, in one line

**`PlacedEntity::rate` is always a planned *aggregate* — a row, lane-family
or merger-cascade total — broadcast onto every tile that participates. It is
never the flow through that tile.**

The handoff asked whether the number is per-tile flow or a family total, and
expected "probably both, depending on which code stamped it." It is not both.
Across every stamp site in the engine, it is the aggregate. There is no
per-tile stamp site.

Therefore **`entity.rate > belt_capacity` is a category error** and cannot
evidence a physical defect. Any check, audit, or issue built on that
comparison is measuring the planner's tier-selection input, not the belt.

## Why the number exists

`rate` is the *tier-selection* figure. The planner needs to know how much a
family of items must move in order to pick yellow/red/blue and to decide how
many parallel belts to run. It is stamped onto tiles as provenance — so the
web inspector and the debugger can attribute a tile to its family — and it is
carried, not consumed. Its source of truth is `BusLane::rate`, whose own
docstring has always said what it is:

> Total throughput (items/s or fluid/s) for belt/pipe tier selection.

## Stamp-site census

The handoff estimated 76 sites in three files. The real census is **89 sites
across eight files** — 87 sites in the six files that stamp a value, plus 2 that only ever write `None` —
and the two highest-volume ones are *mutation* sites the handoff's method
would have missed entirely, because it enumerated struct literals only.

(`templates.rs` is 64, not 65: a naive `rate: Some(` grep also matches the
tail of `loop_priority_rate: Some(` at `templates.rs:4478`, which is a
different field. Count on a word boundary.)

| site | count | stamps | denotes |
|---|---|---|---|
| `bus/templates.rs` | 64 | `minor_total`, `major_loop_rate`, `near_total`, `far_total`, `major_total`, `sushi_total`, `major_export_rate`, `input_total_rate` | family/row totals — every expression is a `*_total` or an explicit loop/export rate |
| `bus/ghost_router.rs` | 10 | `lane.rate`, `tail.rate` | the `BusLane` total, verbatim |
| `bus/output_merger.rs` | 9 | `total_rate` | the merger cascade's total, stamped on **every** tile it emits — input limbs, south columns, splitters, and pass-through belts for *uninvolved* columns |
| `bus/row_rotation.rs` | 2 | `edge.rate` | rotation-graph edge total |
| `blueprint_parser.rs`, `sat.rs` | 2 | `None` | never stamped |
| **`bus/placer.rs:1543`** | **1 (mutation)** | `f.rate * count` | **the row's aggregate for that item**, written onto every row entity carrying it |
| **`bus/trunk_renderer.rs:121`** | **1 (mutation)** | the caller's `rate` arg | broadcasts one value across a whole rendered path |

`lane_planner.rs` has 69 `rate:` initializers and stamps **zero** tiles — its
`split_rate = lane.rate / effective_n_splits` lands on a `BusLane`, not on a
`PlacedEntity`. The handoff cited that divided figure as evidence the stamp
might be per-lane; it never reaches a tile. The post-split lane's total is
what `ghost_router` later stamps.

## The evidence

Method: a temporary `rate_site: Option<&'static str>` field on
`PlacedEntity`, mechanically attached at all 89 sites, so every flagged tile
could be attributed to the code that wrote its number. Config is the
handoff's: EC@60/s, `assembling-machine-2`, from ore, `fast-transport-belt`
ceiling. Arm B lifts the `input-rate-delivery` selection exemption — the
parked PU fix — which is what produces the 376-tile layout.

Every tile the `stacking_ec_60s` audit flags, attributed, with what the two
independent graph-walking lane models say actually flows through it:

| arm | S | flagged | stamp site | STAMPED | actual (`belt_flow`) | actual (`belt_structural`, dispatched) |
|---|---|---|---|---|---|---|
| main | 1 | 291 | `output_merger.rs` ×5 sites | 60.0 | 7.5–9.0 | 7.5–9.0 |
| main | 2 | 0 | — | — | — | — |
| fix | 1 | 17 | `output_merger.rs` ×2 sites | 60.0 | 30.0 | 30.0 |
| fix | 1 | **376** | **`placer.rs:1543`** | **90.0** | 1.5–30.0 | 0.0 |
| fix | 2 | **376** | **`placer.rs:1543`** | **90.0** | 1.5–30.0 | 0.0–7.5 |

Three independent lines agree, and all three contradict the stamp:

1. **Code.** `placer.rs:1543` stamps `f.rate * count` — a row aggregate —
   onto every entity in the row carrying that item, including per-machine
   feeder stubs. `output_merger` stamps the cascade total onto pass-through
   belts for columns that are not even part of the merge.
2. **Arithmetic.** The copper-cable stamp is 90/s. The solver plans
   **180/s** of copper-cable (60 machines × 3/s). 90/s is one *split row's*
   share — so the stamp is not the tile's flow, and not even the item's
   total. It is one row block's aggregate.
3. **Both lane models.** Neither model puts more than 30/s through any
   flagged tile, against a 60/s stacked cap at S=2. The dispatched model
   (`belt_structural`) reports **0** tiles over capacity in every arm at
   every stack size, and its global maximum flow is exactly at cap (30.00/s
   at S=1, 60.00/s at S=2) — never above.

The sim measurements already in the handoff are the fourth line: the same
S=2 layout the audit calls "3.00× over capacity" measured **96.0% of plan**.

**The audit has zero true positives.** Across the two S=1 arms that is 684
tiles (291 + 393); counting the S=2 arm as well, 1060. Not one of them
carries more than its belt's capacity by either model.

## Consequences

- **The `stacking_ec_60s` / `stacking_fanin_wall_lift` audit is invalid.**
  All three fixtures carry it (the third,
  `stacking_fanin_wall_lift_ec6_yellow_legendary`, is the one #597 missed).
- **The PU fix's "physically impossible layout" objection is void** — it was
  this comparison and nothing else. That is *not* the same as the lift being
  ready: attempting it on 2026-08-07 surfaced a second, unrelated blocker
  (`input-rate-delivery` false positives inverting candidate ranking on
  `big-electric-pole@1`). Lift status is tracked in ONE place —
  [`validator-trust.md`](validator-trust.md) hole 2. Do not restate it here
  or in `status.md`; this document went out with three records saying
  "unblocked" and two saying "blocked" and a reviewer had to catch it.
- **#311 needs re-evidencing — which is NOT the same as "#311 is fiction".**
  Its *stamp-based* evidence is void: the tiles stamped 60/s carry 7.5–9.0/s
  by both lane models. But an independent measurement points the other way at
  S=1 — `stress_electronic_circuit_60s_red_from_ore` sims at **~50% of plan**
  (30.5/s measured against 60/s planned, `status.md`), and 30.5/s is
  suspiciously close to exactly one red belt's 30/s. That is consistent with a
  real high-rate bottleneck of the shape #311 describes, on tiles this audit
  never flags. **Do not close #311 on this document.** Re-argue it from a
  walked model or a sim; the open defect is already tracked in
  `rfc064-phase2-followups.md` §1.
- **A stamp-based over-capacity check cannot be written.** This is the
  strongest claim here and it rests on the *stamping code*, not on either
  lane model: the number is an aggregate, so no threshold on it can mean
  "this tile is over-committed". The weaker, model-dependent claim — that no
  flagged tile is *actually* over capacity — rests on the two walkers, and
  the S=1 arm of that is not settled (see below). The falsified 2026-08-07
  check was not a near miss; the data does not support the comparison at all. The per-tile question already has an owner —
  `validate::check_lane_throughput`, which walks the belt graph seeded from
  machine specs and never reads the stamp.

## Two things found on the way that are not the answer

- **There were two parallel lane-rate models**, `belt_flow::compute_lane_rates`
  and `belt_structural::compute_lane_rates`. *(2026-08-15, #632 B5: the
  dispatch now runs the `belt_flow` model — the question below is SETTLED
  in its favor with sim/meter receipts, see `validator-trust.md`'s
  lane-throughput row — and the `belt_structural` twin is scheduled for
  deletion. The paragraphs below are kept as the record of the
  then-unresolved state.)* They disagreed: on the S=1 arms
  `belt_flow` puts 36/s through ore belts with a 30/s cap (96–109 tiles),
  where `belt_structural` reports nothing over capacity. Note that
  quoting a non-dispatched model as "actual flow" is the same error class
  this document exists to close.

  **The disagreement was unresolved and it mattered.** `belt_structural`'s
  global maximum lands exactly at capacity in every arm (30.00/s at S=1,
  60.00/s at S=2) and never above, which is what either a correctly-sized
  planner *or* a saturating model would produce — this document does not
  establish which. The S=1 ~50%-of-plan sim measurement above is more
  consistent with `belt_flow`'s reading than with `belt_structural`'s. So:
  the S=2 conclusions here are corroborated by sim (96.0% of plan); **the
  S=1 arm is not**, and the fixtures' `errors1.is_empty()` guarantee rests
  on the unarbitrated model choice. Arbitrating the two models is the
  highest-value follow-up this investigation produced.
- **`crates/core/tests/e2e.rs:7783`'s claim that "the lane walker never
  visits merger tiles" is stale.** Both models returned entries for all 291
  merger tiles.

## Rules

1. **Never compare `PlacedEntity::rate` to a belt's capacity as a claim
   about a tile.** It is an aggregate over a family that may be realized as
   several parallel belts. The one sanctioned use of that arithmetic is as a
   statement about **tier selection** — "this family does/doesn't fit on one
   belt of the chosen tier" — which is what the reframed
   `family_over_one_belt` probes in the `stacking_*` fixtures assert, and
   they say so at the call site. Even then it is not a physical invariant: a
   family exceeding one belt is perfectly legal when the planner realizes it
   as parallel belts.
2. **For per-tile flow, use `validate::check_lane_throughput`** (or
   `belt_structural::compute_lane_rates` directly). It walks the graph from
   machine specs; it does not trust the stamp.
3. **Differing stamped values across tiles prove nothing** about the number
   being per-tile — parallel *paths* differ from one another while every
   tile within a path shares one number. This misreading is what made the
   falsified check look justified.
4. **A stamp census must include mutation sites.** Two `ent.rate = …`
   assignments outside any struct literal stamp more tiles than most of the
   87 literal sites combined, and enumerating literals alone finds neither.

Related: [`validator-reporting.md`](validator-reporting.md) (checks that go
quiet), [`validator-trust.md`](validator-trust.md) (whether a check is
believed). This one covers the third failure: **a number whose name implies a
meaning it never had.**
