# RFC-053: Direct-insertion cells — machine→machine coupling

Registry: [`rfcs.md`](rfcs.md). Status: **Draft (circulated for review)**.

## Summary

Produce **machine → inserter → machine** direct insertion (DI): a
producer row and a consumer row separated by a single tile, coupled by
inserters, with **no belt between them**. This is the topology
[#429](https://github.com/storkme/spaghettio/issues/429) actually asked
for and the one the community corpus overwhelmingly builds — 4,116
mined `copper-cable → electronic-circuit` instances, of which the top-20
patterns (3,866) contain exactly **one** that is not a 1-tile gap.

The win is throughput *and* density, and it comes from removing an
interface rather than adding one: a machine-to-machine hand is
exact-hand, so a **stack inserter moves 12.0/s at zero research**,
against the 1.2/s (L0) – 4.8/s (L7) ceiling of the reach-2 long-handed
inserter that [#432](https://github.com/storkme/spaghettio/pull/432)'s
belt→belt bridge is structurally stuck with.

Scoped as the **first cut of the "dynamic face allocation" north star**
recorded in [`rfc-inserter-sizing.md`](rfc-inserter-sizing.md) — belts,
inserters, pipes and machines bidding for face tiles under
reach/throughput/adjacency constraints. **Phase 1 is solids-only
(69.4% of top-10 corpus demand); pipes are required Phase 2 scope** —
the original "no pipes at all" cut tripped kill criterion 6 in Phase 0
and was widened (see Phase 0 results).

## Prerequisite: #432 — ✅ MERGED 2026-07-25 (`4df8b0a7`)

This RFC builds on types that landed with PR
[#432](https://github.com/storkme/spaghettio/pull/432):
`SolverResult.di_couplings`, the `DICoupling` struct,
`LayoutOptions.direct_insertion` (default `false`) and
`validate::is_di_bridge_inserter` are all on `main` now, verified after
the merge. **Phase 1 is unblocked.**

Notes carried forward from when this was a live blocker:

- The Motivation's "reproducible today" case was measured on #432's
  branch at `a50c45cf`; it reproduces on `main` from `4df8b0a7`.
- The rebase-drift risk is **closed** — the merged `DICoupling` shape is
  the one this RFC's integration section describes (re-verified against
  `main`, not the branch).
- #432 also merged `main` in before landing, so it carries the L2
  inserter-capacity default (#431) and the recalibrated bridged floor
  (#434). The `input-rate-delivery` figures quoted below (3 warnings at
  L0 / 1 at L2 / 0 at L7) predate those and should be re-measured in
  Phase 1 rather than trusted verbatim.

## Motivation

**DI is the dominant human strategy the engine cannot produce.** #429's
corpus sweep: 17,009 DI inserters across 793 of 6,038 blueprint members
(13.1%), with cable→EC the single most common pair in the game. (That
sweep's Node extractor put cable→EC at 3,887; this RFC's Rust miner puts
it at 4,116 — a ~6% divergence of the same class #389 already recorded
between the two implementations. Neither number is load-bearing here;
the *geometry distribution* is, and it is reproduced below from the Rust
miner alone.)

**#432 built a different, lesser thing.** Its "DI bridge" is
belt→belt: the producer's output belt and the consumer's input belt both
remain, and a long-handed inserter spans the gap between them. Item
path:

```
producer ─i→ belt ─BRIDGE→ belt ─i→ consumer     (3 inserter hops, 2 belts)
```

versus the bus's 2 hops + trunk routing, versus true DI's:

```
producer ─────i─────→ consumer                    (1 hop, 0 belts)
```

So #432 deletes the *trunk lane* (a real footprint win, worth keeping)
but pays the belt-drop tax on one side, the belt-pickup tax on the
other, and adds a reach-2 hop in the middle. It sidesteps none of the
belt-interface ceilings #429 was about.

**Reproducible today** (`direct_insertion: true`, EC@10/s from plates,
post-#432 `a50c45cf`): the coupling needs two long-handed bridges per
consumer machine and clears `input-rate-delivery` only at **L7** —
3 warnings at L0, 1 at L2, 0 at L7. The bind is structural: the bridge
spans a 2-tile gap, so it *must* be long-handed (**I8a**: the only
reach-2 inserter in vanilla), and long-handed tops out at 4.8/s against
a 7.5/s demand.

**The corpus says humans don't build that.** Mined 2026-07-24 with the
tracked `di-patterns` tool (16,507 DI observations across 172 parsed
corpus files, 98 containing DI) — reproduce with:

```bash
cargo run --release -p spaghettio_mining --bin di-patterns -- \
    geometry scripts/blueprints copper-cable electronic-circuit
```

| count | inserter | machine gap | lateral offset | axis |
|---|---|---|---|---|
| 475 | bulk-inserter | **1** | 0 | horizontal |
| 398 | fast-inserter | **1** | +2 | horizontal |
| 396 | fast-inserter | **1** | −1 | horizontal |
| 369 | fast-inserter | **1** | 0 | horizontal |
| 285 | fast-inserter | **1** | +1 | vertical |
| 255 | fast-inserter | **1** | −2 | vertical |
| … | … | | | |
| 200 | long-handed | **2** | 0 | horizontal |

`gap=1` (reach-1, machines one tile apart) dominates. Lateral offsets of
0, ±1, ±2 are all common — the empirical signature of consumers
straddling two producers (see Design).

**Read the `gap=2` rows correctly** (corrected 2026-07-24 — the first
draft got this wrong). The miner only records a pair when **both** the
pickup and the drop tile resolve to a machine, so every one of the 4,116
instances is machine→machine. The 200 `gap=2` long-handed rows are
therefore **not** #432's bridge — they are a legitimate *third* DI
variant: long-handed machine→machine, leaving one spare tile between the
machines that a belt or pole can thread through (routing bought with
throughput). #432's belt→belt shape is structurally *uncountable* by this
miner — its pickup is a belt tile — so the corpus contains **zero**
instances of it, rather than the "~5% minority" the first draft claimed.
Both the `gap=1` and `gap=2` machine→machine variants belong in the
pattern library.

Other DI pairs the same sweep found, i.e. the catalogue this generalizes
to: `electric-furnace → electric-furnace` (1,585 — smelting columns),
`solid-fuel-from-light-oil → rocket-fuel` (652 — note the specific
recipe variant; vanilla has three solid-fuel recipes),
`engine-unit → electric-engine-unit` (547),
`casting-copper-cable → electronic-circuit` (544, Space Age foundry —
fluid-adjacent, the known borderline case for kill criterion 6),
`iron-stick → rail` (351).

## Ground truth

**Machine→machine feed rates** (exact hand, no belt tax —
`common::machine_feed_rate`, printed 2026-07-24):

| inserter | L0 | L2 | L7 |
|---|---|---|---|
| inserter | 0.84 | 1.68 | 3.36 |
| long-handed | 1.20 | 2.40 | 4.80 |
| fast | 2.31 | 4.62 | 9.24 |
| bulk | 2.40 | 4.80 | 14.40 |
| **stack** | **12.00** | 19.20 | 32.00 |

Stack is **reach-1 only** (I8a) — it is available to a 1-tile-gap
sandwich and structurally unavailable to a 2-tile bridge. That single
fact is the RFC.

**Canonical case** (AM3, speed 1.25, both recipes 0.5s → 2.5 crafts/s):

- `copper-cable`: 1 copper plate → 2 cable ⇒ **5.0 cable/s produced** per machine
- `electronic-circuit`: 1 iron + 3 cable → 1 EC ⇒ **7.5 cable/s**, 2.5 iron/s in; 2.5 EC/s out
- Ratio **3:2** — EC@10/s solves to 6 cable machines and 4 EC machines
  (matches the `DICoupling` the solver already emits: `producer_count
  6.0, consumer_count 4.0`).

**Corpus vintage caveat (must be handled before trusting corpus
rates).** Inserter *names* shifted between game versions: 1.1's
`stack-inserter` is 2.0's `bulk-inserter`, and 2.0 introduced a new
`stack-inserter` (belt-stacking). Mined **geometry** is trustworthy
immediately; mined **throughput attribution** is not, until patterns are
version-gated. Phase 0 records the version split or drops rate claims
from the library.

## Phase 0 results (run 2026-07-24) — KC1 passes, **KC6 fires**

Census of the corpus's top-10 DI pairs, with both kill criteria evaluated
(pair counts reproducible via `di-patterns census`; the ratio arithmetic
was run in a scratch example):

| pair | instances | gap* | fluid-touching | need/s | verdict @ (Stack, L2) |
|---|---|---|---|---|---|
| copper-cable → electronic-circuit | 4,116 | 1 | — | 2.50 | OK |
| solid-fuel-from-light-oil → rocket-fuel | 652 | 3 | YES | 0.28 | OK |
| engine-unit → electric-engine-unit | 547 | 3 | YES | 0.04 | OK |
| casting-copper-cable → electronic-circuit | 544 | 1 | YES | 2.50 | OK |
| copper-cable → advanced-circuit | 360 | 3 | — | 0.28 | OK |
| copper-cable → space-platform-foundation | 353 | 1 | — | 0.83 | OK |
| iron-stick → rail | 351 | 1 | — | 0.83 | OK |
| casting-iron → electronic-circuit | 339 | 1 | YES | 1.00 | OK |
| electric-engine-unit → flying-robot-frame | 318 | 1 | YES | 0.02 | OK |
| yumako-processing → bioflux | 268 | 1 | — | n/a¹ | n/a |

**KC1 (ratio feasibility): PASSES, with margin.** Every pair needs
≤ 2.50/s per inserter slot — the canonical cable→EC is the *worst* case
in the whole top-10, and a stack inserter at the L2 default supplies
19.2/s. The straddle construction is not near its limit anywhere.

**KC6 (fluid coverage): FIRES — 5 of 10 > the threshold of 2.**
Its prescribed action is *"stop and re-scope toward full face allocation
rather than shipping a DI that covers a rump of the corpus."* Taking that
seriously, with the criterion's own rationale as the test:

- **The criterion conflates two different things.** An inserter cannot
  move fluid, so the *coupled item* is solid in 100% of DI pairs by
  construction. In all five flagged pairs the fluid is a **separate
  ingredient on the other face** (lubricant into `electric-engine-unit`,
  molten metal into the `casting-*` recipes). The DI coupling itself is
  solid and placeable; what those machines need is a **pipe on a
  non-DI face**. So the criterion does not measure "unreachable without
  pipe placement" — it measures "fluid-adjacent somewhere".
- **The threshold uses the wrong metric.** Its rationale is *demand*
  excluded, but it counts pairs unweighted. Instance-weighted,
  solids-only covers **69.4%** of top-10 DI instances (5,448 / 7,848),
  and the single dominant pair — cable→EC at **52.4%** on its own — is
  fully solid. "A rump of the corpus" is falsified by the data.

**Disclosure on process**: this criterion was written earlier the same
day in response to review feedback that its predecessor was toothless,
and the defects above were diagnosed *after* seeing it fire. Rewriting a
tripped kill criterion is precisely the failure mode kill criteria exist
to prevent, so the resolution below is deliberately the *criterion's own
prescribed action* (re-scope), not a reprieve: **pipes move out of
Non-goals and into required Phase 2 scope.** Phase 1 remains solids-only
as a landable slice, now justified by weight (69.4%) rather than by
assertion.

¹ `yumako-processing → bioflux`'s coupled-item lookup returned no
amount (multi-result recipe) — a census-tool gap, not a design finding.

**Recorded data-quality note**: `electric-furnace → electric-furnace`
(1,585 instances — the *second* most common DI pair in the raw sweep) is
absent from this table because furnace entities carry no explicit
`recipe`, and the census only counts pairs where both recipes resolve.
Furnace→furnace DI (smelting columns) is real, common, and will need its
own handling; it is not covered by the ratio analysis above.

## Design

### The DI cell

A producer row, a one-tile inserter band, a consumer row:

```
 y0-2   [ copper-cable machines ]      6 × 3×3, pitch 3
 y3       S   S    S   S   ...         stack inserters, reach 1
 y4-6     [ EC machines ]              4 × 3×3, offset to straddle
```

*Schematic only — not to tile scale.* The authoritative geometry is the
column table below; an earlier draft carried a tile-aligned ASCII drawing
whose boxes disagreed with their own `x=` labels, and the derivation
depends on exact columns, so the numbers live in tables from here on.

**Column spans** (every machine 3 wide, so a machine at `x` occupies
`x..x+2`):

| row | machines (column span) |
|---|---|
| producers | `cab1` 0–2 · `cab2` 3–5 · `cab3` 6–8 · `cab4` 9–11 · `cab5` 12–14 · `cab6` 15–17 |
| consumers | `EC1` 1–3 · `EC2` 5–7 · `EC3` 10–12 · `EC4` 14–16 |

**Source-limited, not inserter-limited.** One stack inserter could move
12/s, but the cable machine behind it only *makes* 5/s. An EC machine
needing 7.5/s must therefore draw from **two** producers — which forces
the consumer row off the producer row's pitch so every consumer straddles
a producer boundary. **The corpus's non-zero lateral offsets are this
same straddle.**

Derivation (why 8 inserters and not 12), from the x-positions above —
all machines 3 wide, so `EC1` spans columns 1–3, `cab1` spans 0–2,
`cab2` spans 3–5:

| edge | overlapping columns | = inserter slots | flow |
|---|---|---|---|
| cab1 → EC1 | 1, 2 | **2** | 5.0/s |
| cab2 → EC1 | 3 | **1** | 2.5/s |
| cab2 → EC2 | 5 | 1 | 2.5/s |
| cab3 → EC2 | 6, 7 | 2 | 5.0/s |
| cab4 → EC3 | 10, 11 | 2 | 5.0/s |
| cab5 → EC3 | 12 | 1 | 2.5/s |
| cab5 → EC4 | 14 | 1 | 2.5/s |
| cab6 → EC4 | 15, 16 | 2 | 5.0/s |

Eight directed producer→consumer edges, hence eight inserters at Stack
tier (one per edge suffices there). Balance check: each producer ships
exactly 5.0/s (`cab2` = 2.5 + 2.5; `cab1` = 5.0), each consumer receives
exactly 7.5/s (`EC1` = 5.0 + 2.5), totalling **30.0/s on both sides**.
`cab1` and `cab6` are the row's end machines and feed a single consumer
each, which is why the spans are not uniformly pitched.

**The per-edge slot count is the real budget** — not the machine's
3-column width. An inserter draws from exactly one producer, so edges
cannot pool slots. Feasibility for the canonical case is therefore:
`2 slots × rate ≥ 5.0` **and** `1 slot × rate ≥ 2.5`, which both reduce
to the same clean condition:

> **A DI straddle is feasible iff `machine_feed_rate ≥ 2.5/s`.**

### `max_inserter_tier` — the axis the first draft missed

`LayoutOptions.max_inserter_tier` is an existing **user-facing hard cap**
(`Regular | Fast | Stack`, default `Stack`) that the ladder never
exceeds — the same never-auto-escalate contract as `max_belt_tier`
(`rfc-inserter-sizing.md`). It is **orthogonal** to
`inserter_capacity` (the 0–7 research level); the first draft conflated
them. Applying the `≥ 2.5/s` rule across both axes (the ladder places
only `inserter`/`fast-inserter`/`stack-inserter` — bulk is deliberately
not in its catalogue):

| `max_inserter_tier` | L0 | L2 (engine default) | L7 |
|---|---|---|---|
| `Regular` (0.84 / 1.68 / 3.36) | ✗ | ✗ | ✓ |
| `Fast` (2.31 / 4.62 / 9.24) | ✗ (2.31 < 2.5) | ✓ | ✓ |
| **`Stack`** (12.0 / 19.2 / 32.0) — default | ✓ | ✓ | ✓ |

So **at engine defaults (Stack, L2) the canonical coupling is feasible
with one inserter per edge**, and it stays feasible for a `Fast`-capped
user at the default research level. It is infeasible only for a
`Regular`-capped user below L7, and for anyone at true L0 without stack
inserters — where the shortfalls are brutal but narrow (`fast` misses by
0.19/s, `bulk` would miss by 0.10/s were it in the catalogue).

**When infeasible, DI is refused for that coupling** and the item falls
back — to #432's bridge where geometry permits, otherwise to the bus —
with an honest warning. DI is never silently under-fed (the #432 lesson:
a silently-disabled DI looked like a clean layout).

### Face allocation

A row-layout machine has two usable faces (north/south). DI consumes
one, so the consumer's *remaining* flows share the other — for EC, iron
in (2.5/s) and EC out (2.5/s). Mixed reach is the lever (the user's
observation, and what the corpus's inserter mix implies):

```
 y4-6   │  EC machine  │
 y7        i        L        i = reach-1 → near belt (iron in)
 y8       ═══════════        iron-plate belt          L = reach-2, steps
 y9       ═══════════        EC output belt               OVER the iron belt
```

This is the seed of dynamic face allocation: each face-tile is a
resource that a belt, an inserter, or nothing bids for, under reach and
throughput constraints.

### Candidate generation: constructive first, SAT only if needed

The full problem is genuinely combinatorial — machine x-offsets per row,
which (producer, consumer) pairs get an inserter and at which column,
inserter type per slot, and what occupies each face-tile — subject to
reach, per-inserter throughput, producer output ≤ production, consumer
input ≥ demand, and non-overlap. That is CP-SAT-shaped, and the repo has
SAT machinery ([`rfc-cp-sat-placement.md`](rfc-cp-sat-placement.md), the
crossing-zone solver).

**We deliberately do not start there.** Two tiers:

- **Tier 1 (this RFC's default): mined patterns + constructive
  straddle.** Frequency-ranked corpus patterns supply the geometry;
  offsets are chosen by the straddle rule above; producer→consumer
  assignment is a min-cost flow on the physical-adjacency bipartite
  graph. Deterministic, fast, no solver in the layout loop.
- **Tier 2 (escalation, gated by kill criterion 5): CP-SAT** over the
  variables above, for pairs Tier 1 cannot serve.

Rationale: this project's dominant rework shape is exploration that
overruns its evidence, and reaching for a solver before measuring the
cheap constructive path is exactly that shape. Tier 1 is also the
honest baseline against which Tier 2 must justify itself.

### Pattern library

`di_pattern_library.rs`, **generated and committed**, mined from the
corpus by a script — the same architecture as
`bus::balancer_library` (regenerated by
`scripts/generate_balancer_library.py`). Entries are canonicalized
`(producer recipe, consumer recipe, machine gap, lateral offset, axis,
inserter class)` with corpus frequency, version-gated per the vintage
caveat.

### Integration

- **Solver**: unchanged. `SolverResult.di_couplings` (`DICoupling`)
  already lands from #432 and is the input to this work.
- **Placer**: a new cell/template class (`bus::di_cell`) that emits a
  producer+inserter+consumer band as one unit, rather than two
  independent rows plus a bridge.
- **`LayoutOptions.direct_insertion`** keeps its meaning and its
  `false` default; the strategy behind it changes from bridge to cell.
- **#432's belt bridge is retained as a fallback**, not deleted: it is
  the legal move when the sandwich is infeasible (rows that cannot be
  adjacent, couplings whose ratio no offset satisfies). It keeps its
  own honest warnings. Phase 3 decides whether any corpus-common case
  still needs it; if none does, removing it becomes a follow-up.
- **Validators**: `is_di_bridge_inserter` (from #432) generalizes to a
  DI-inserter predicate; machine→machine inserters already satisfy
  `check_inserter_direction` (both sides touch machines), so the
  exemption burden *decreases* relative to the bridge.

### Non-goals

- **Fluid *couplings*** — a DI'd item must be a solid. This is not a
  restriction in practice: inserters cannot move fluid, so every DI
  coupling is solid by construction.
- ~~**Pipes / fluids**~~ — **RE-SCOPED IN (Phase 2) by the KC6 trip,
  2026-07-24.** The original Non-goal excluded *fluid-touching machines*
  altogether; Phase 0 measured that this excludes 5 of the top-10 pairs
  (30.6% of instances), so the exclusion is too broad to stand. What
  remains true, and is why this is Phase 2 rather than Phase 1: a
  fluid-touching machine needs a **pipe on a non-DI face**, fluid ports
  are prototype-fixed per orientation (so face allocation must search
  orientations), and pipe misplacement is hard-infeasibility (network
  merging) rather than cost.
- **No belt-tier escalation** (standing user constraint).
- **Not a bus replacement**: DI is a strategy for qualifying couplings;
  everything else keeps the bus.
- **No new inserter-rate model** — `machine_feed_rate` is the existing
  single source of truth and is not re-derived here.

## Kill criteria

1. **Ratio feasibility (evaluated in Phase 0, before any placer
   code).** If for the canonical cable→EC case no integer straddle
   offset gives every consumer ≥ its full demand from physically
   reachable producers, the sandwich shape is wrong for the game's own
   ratios — stop and reconsider the topology, do not "mostly" feed
   consumers.
2. **Face contention. ✅ EVALUATED 2026-07-25 — PASSES, with zero
   margin.** If the consumer's remaining face cannot carry its non-DI
   flows (iron in + EC out) within its tile budget at **≤ L2**
   `inserter_capacity` (i.e. the cell is only feasible at max research),
   the topology is under-scoped — stop. Measured against the real rate
   tables (`kc2_face_contention` in `tests/e2e.rs`), AM3 canonical,
   2.5/s each way, 3-wide face:

   | side | inserter | rate @L2 | needed | columns |
   |---|---|---|---|---|
   | near (reach-1) | fast | 4.62/s | 2.5/s | **1** |
   | far (reach-2) | long-handed | **2.40/s** | 2.5/s | **2** |
   | | | | | **3 of 3 → PASSES** |

   The criterion does not fire, but every column is consumed. The bind
   is structural rather than a research shortfall: long-handed is the
   only reach-2 inserter (I8a) and its belt-drop rate is 2.40/s at L2
   rising only to 3.20/s at L7, so max research buys back exactly one
   column and no more. Swapping which item takes the near belt is
   symmetric (both flows are 2.5/s), so 1 + 2 is the budget either way.

   **Consequences Phase 2 must design around, not discover later:** a
   Phase 2 cell has **no spare face column**, so any consumer with a
   third flow (second output, third input) does not fit and must refuse;
   and pipe placement — which KC6 re-scoped INTO this phase — needs face
   tiles this budget does not have, so fluid-touching consumers will
   need a different face plan rather than this one extended.

   *Incidental finding, raised and then CLEARED:* `bulk-inserter`'s
   belt-drop rate is flat 2.40/s across every research level and belt
   tier, while its machine-feed rate scales 2.40 → 4.80 → 14.40. That
   looked like a modelling gap Phase 2's allocator might trip over.
   It is not — `belt_drop_rate`'s own doc states it: *"bulk inserters:
   always flat — the engine never places one"*, i.e. a deliberate
   simplification for an entity that only ever arrives by parsing
   community blueprints. It also cannot affect this phase twice over:
   bulk is reach-1, so it is structurally excluded from the binding
   far/reach-2 column, and on the near side stack dominates it (6.38/s
   vs 2.40/s at L2). Recorded because "verify the load-bearing number"
   was the right instinct even though the answer was benign.

   **2b. Tier-cap degradation.** `max_inserter_tier` is a hard user cap,
   orthogonal to research level. If, at the engine defaults
   (`Stack`, L2), the canonical coupling needs **more than one inserter
   per producer→consumer edge**, the per-edge slot budget derived above
   is wrong and the straddle geometry must be re-derived — stop. And if
   a `Fast`-capped user cannot get a feasible cell at the **default**
   research level, DI is too fragile to ship default-on — it stays an
   opt-in strategy with an honest refusal, not a silent degradation.
3. **Honest throughput. ✅ EVALUATED 2026-07-25 — PASSES.** If a DI cell
   validates clean but the sim harness measures **< 98% of plan** on the
   canonical fixture, the model is wrong and the checks are lying — stop
   everything. (This is the #383 lesson: validator-clean concealed a
   real starve for weeks.) **Measured on a real cell** (`steel-plate@2/s`
   from `iron-ore`, 16:16 furnace→furnace — the corpus's dominant DI
   shape; the fixture is `di_cell_kc3_export` in `tests/e2e.rs`):

   | | planned/s | produced/s | delivered/s | entities |
   |---|---|---|---|---|
   | **DI cell** | 2.00 | 2.21 (+10.7%) | 2.24 (+12.0%) | **213** |
   | control (DI off) | 2.00 | 2.20 (+10.0%) | 2.24 (+12.0%) | 335 |

   `converged=true`, 32/32 machines working, both runs. DI delivers
   **112% of plan**, nowhere near the 98% floor, and matches its own
   bus control to within 0.7pp produced / 0.0pp delivered. The
   criterion does not fire.

   **The "validates clean" half was NOT true on the first pass, and
   finding that out took a second look.** KC3 is a conjunction — a cell
   that *validates clean* AND under-delivers — and the first measurement
   only checked the throughput half. Running the validator afterwards
   returned **16 `Error`s and 48 warnings**: every producer furnace
   flagged `output-belt: no output inserter has a belt at its drop
   position`, plus `inserter-throughput` crediting cell machines 0.00/s.
   All false positives — the sim had already proved the shape moves
   112% of plan — but a DI feature that buries the validator in false
   errors is not shippable, and worse, real errors would have been
   indistinguishable. Two distinct causes, both now fixed and both
   inherent to the cell design rather than incidental:

   - a cell producer has **no output belt at all** (its output leaves
     through the band), so every "machine must have an output inserter
     dropping onto a belt" test fails by construction;
   - the fused `RowSpan` carries the producer's inputs, so
     `resolve_row_spec` attributes the producer's input item to the
     *consumer* machines — the check asked furnaces eating iron-plate to
     account for iron-ore.

   `validate::is_di_cell_entity` now exempts cell entities the way
   `is_di_bridge_inserter` handles #432's bridges. **Post-fix: 0
   validation issues, same 112% delivery.** So KC3's conjunction is
   satisfied on both halves — but only after the gap was closed.

   **The control is the load-bearing part.** A single DI run showing
   +10.7% is uninterpretable: a solver rate-model artifact and a DI
   artifact are indistinguishable without it. Running the same target
   with `direct_insertion: false` attributes the overshoot to the
   *model*, not the topology — the engine under-predicts electric-furnace
   steel output by ~10% in both. That discrepancy is real, pre-existing
   and orthogonal to this RFC; it deserves its own issue rather than
   being quietly absorbed here.
4. **Density premise. ✅ EVALUATED 2026-07-25 — PASSES; re-confirmed
   end-to-end.** The KC3 run above measures it on a whole solved layout
   rather than a hand-derived cell: **213 entities against the bus
   control's 335, a 36% reduction** at identical delivered throughput.
   Original hand-derived evaluation follows. Measured on
   the canonical fixture (EC@10/s from plates, DI off) against the real
   engine: the bus places `copper-cable` at y1–7 and `electronic-circuit`
   at y10–17 with a `copper-cable` trunk lane at y7–9 — a **17-tile**
   combined vertical extent. The Phase-1a/1b cell is machines + a
   one-tile band + machines = **7 tiles**. Cell is strictly smaller by
   59%, so the density premise holds and this criterion does not fire.
5. **Solver escalation bound.** If Tier 1 leaves > 20% of the corpus's
   top-10 DI pairs infeasible, escalate to Tier 2 — but if CP-SAT
   cannot place a single pair within **500 ms**, stop: too slow for the
   layout loop, and the constructive path is the answer we ship.
6. **Scope integrity.** Sharpened so it can actually fire: if **more
   than 2 of the corpus's top-10 DI pairs** turn out to have a
   fluid-touching producer or consumer (making them unreachable without
   pipe placement), then "solids only" excludes too much of the real
   demand to be a useful first cut — stop and re-scope toward full face
   allocation rather than shipping a DI that covers a rump of the
   corpus. (Phase 0 measures this; `casting-copper-cable → EC` at 544
   instances is the known borderline case — foundry casting is
   fluid-adjacent.)

## Verification plan

Per the layout-engine protocol in [`CLAUDE.md`](../CLAUDE.md#verification-protocol-for-layout-engine-changes), plus:

- **Full e2e** green; goldens **byte-identical** while
  `direct_insertion == false` (the whole feature stays opt-in until
  Phase 4).
- **New e2e fixture** — EC@10/s from plates, DI on: zero errors, and
  specifically **zero `input-rate-delivery` warnings at L0** (the
  bridge's L7-only bind must be gone, not merely reduced).
- **Footprint assertion** in the same fixture: DI cell height < bus
  baseline height (kill criterion 4 as a test, not a vibe).
- **Sim harness** (RFC-050) on the canonical fixture: produced ≥ 98% of
  plan, `converged: true`, zero output-blocked machines. This is the
  real gate — kill criterion 3 makes validator-clean insufficient.
- **Corpus cross-check**: the generated geometry must match a mined
  corpus pattern, or the divergence is explained in the decision log.
- **Trace signals**: a new `DiCellPlaced` / `DiCellInfeasible` pair so
  fallback-to-bridge and fallback-to-bus are observable rather than
  silent (the #432 lesson — a silently disabled DI looked like a clean
  layout).
- **Browser eyeball** (user's step) once Phase 4 plumbs the option.

## Phasing

- **Phase 0 — mine + feasibility. ✅ COMPLETE (2026-07-24).** Census
  delivered above: KC1 passes with margin (worst case 2.50/s vs 19.2/s
  available at defaults); **KC6 fired and forced the pipes re-scope**.
  **#432 merged 2026-07-25 (`4df8b0a7`), so Phase 1 is ready to start.**
  The miner is now the **tracked** `di-patterns` binary
  (`crates/mining-cli/src/bin/di_patterns.rs`), so every corpus figure in
  this RFC is reproducible from a clean checkout — it was written as a
  gitignored example, which would have left the RFC's central evidence
  unverifiable. Remaining Phase-0 debt carried into Phase 1: emit the
  canonicalized, version-gated `di_pattern_library.rs` (deliberately
  deferred until Phase 1 has a consumer for it — committing a generated
  file nothing references is dead code), and close the furnace→furnace
  recipe-resolution gap noted above.
- **Phase 1 — the DI cell.** `bus::di_cell` for the simplest shape: one
  producer recipe, one consumer recipe, consumer's only solid input is
  the DI'd item.

  **⚠️ The worked example in this RFC is NOT a Phase 1 shape (found
  2026-07-25).** `electronic-circuit` takes `[iron-plate,
  copper-cable]` — *two* solid inputs — so `copper-cable → EC`, the pair
  every geometry figure above is derived from, fails Phase 1's own scope
  line. The consumer's north face is spent on the DI band and its south
  face on the output inserter, so the second input has nowhere to land
  until Phase 2's face allocation. The geometry stays valid (the
  straddle math is rate-driven and item-agnostic — that is why 1a could
  reproduce `[1,5,10,14]`), but the **wiring target** must be a
  single-solid-input consumer. From `recipes.json`, discounting
  `*-recycling`, the in-scope pairs are:

  | producer → consumer | note |
  |---|---|
  | `iron-plate → steel-plate` | furnace→furnace; the corpus's 1,585-instance shape |
  | `iron-plate → iron-gear-wheel` | canonical assembler cell |
  | `iron-plate → pipe`, `→ iron-stick` | same shape |
  | `copper-plate → copper-cable` | producer is the smelter |
  | `stone → stone-brick` | furnace→furnace |

  **This gate must live in the placer, not the solver.**
  `netflow::detect_di_couplings` gates on one-producer / one-consumer /
  no-external-supply / no-surplus / not-fluid — it does **not** look at
  the consumer's input count. So it *will* emit a `copper-cable →
  electronic-circuit` coupling, and Phase 1c wiring that trusts the
  coupling list would build a cell whose EC machines never receive
  iron-plate: a layout that validates clean and starves, which is
  exactly the #383/#432 failure mode this RFC's own verification note
  warns about.
  - **1a ✅ LANDED — straddle geometry + edge assignment.**
    `bus::di_cell::plan_straddle` is the algorithmic core: producer and
    consumer machine positions, the producer→consumer edge set, per-edge
    inserter slots, and the cell's binding `required_rate()`. Pure
    geometry + flow, no entity placement. Independently reproduces this
    RFC's worked example (positions `[1,5,10,14]`, the eight-edge set
    with 2/1 slot splits, `required_rate == 2.5/s`) — the construction
    was derived from flow-interval overlap and landed on the
    hand-derived geometry without being fitted to it.
  - **1b ✅ LANDED — cell stamping.** `bus::di_cell::stamp_di_cell`
    turns a plan into placed machines and inserters: producers, a
    **one-tile** inserter band, consumers. Pinned by the defining DI
    property — every inserter picks from a producer machine tile and
    drops into a consumer machine tile, at reach 1, with **no belt
    emitted for the coupled item** (the same test `classify.rs` applies
    when counting DI in community blueprints). Refuses rather than
    under-feeding when the chosen inserter can't cover an edge within
    the slots that edge owns.
  - **1c ✅ LANDED — placer wiring. Phase 1 is COMPLETE.**
    `place_rows` now fuses an eligible pair into one cell row via
    `cell_eligible` → `try_build_cell` → `fused_cell_spec`. The engine
    emits machine→inserter→machine DI for the first time. Inert by
    default (`direct_insertion: false`), so no existing layout moved —
    the cell path is covered by unit tests only until Phase 4 exposes
    the flag. The notes below are kept as the record of how the step was
    scoped; where they disagree with what shipped, what shipped wins
    (notably: no new `RowKind` was needed — a fused `RowSpan` built
    directly from `DiCellLayout` was enough).

  - *Original 1c pick-up notes (superseded).* Pick-up
    notes, so this doesn't need re-deriving:
    - **Seam**: `bus::placer::place_rows`, the DI branch that currently
      calls `stamp_di_bridge` (search `is_di_consumer` / `di_lookup`).
      Today that branch fires *after* both rows already exist.
    - **Why it is not a small change**: a cell REPLACES both row
      emissions rather than decorating them. `build_one_row` gives a row
      its own belts and pitch; a cell needs the producer and consumer
      machines at `StraddlePlan` positions, one tile apart, with **no
      belt for the coupled item** — so the two specs must be intercepted
      *before* `build_one_row`, while still getting the producer's other
      input belts and the consumer's output belt from the template
      system. Expect a new `RowKind` (cell-shaped) rather than a
      post-hoc stamp.
    - **Foreclosed shortcut (verified 2026-07-25, don't retry it)**: you
      cannot get a cell by passing the consumer row a different
      `x_offset`. Row templates place machines at a UNIFORM pitch, and no
      uniform offset serves the canonical straddle — 4 consumers at pitch
      3 span 12 tiles against the producers' 18, so producers 5 and 6 are
      unreachable at *every* offset (checked exhaustively). The straddle
      needs PER-MACHINE x positions (`StraddlePlan::consumer_xs` is
      `[1,5,10,14]` — non-uniform gaps of 4/5/4), which no existing
      template can express. That is the concrete reason 1c needs a new
      `RowKind` rather than a parameter.
    - **Order of work**: (1) ✅ **cell emission is DONE** —
      `stamp_di_cell_io` emits the complete sub-layout (input belt, feed
      inserters, producers, DI band, consumers, output inserters, output
      belt) and returns a `DiCellLayout` carrying `input_belt_y` /
      `output_belt_y` / x-extent, i.e. exactly the fields a `RowSpan`
      needs. What remains of this step is constructing the `RowSpan`
      itself and hanging it off a `RowKind`. **Contract question —
      RESOLVED 2026-07-25, by design change rather than by either
      prescribed remedy.** The question was whether a cell's producer
      row can safely carry a plain-`i32` `output_belt_y` when it owns no
      output belt. Two things were established:

      - *The premise it rested on was wrong.* #447 recorded that
        "`lane_planner` only reads `output_belt_y_for(item)`". There are
        in fact **11 bare `output_belt_y` reads** across
        `lane_planner`, `lane_order` and `ghost_router`. Auditing every
        one: eight index through a `BusLane`
        (`all_producers()` / `producer_row` / `extra_producer_rows`, or
        `all_producer_rows` during lane construction), so the
        `di_input` argument does cover them; but the three
        output-merger sites (`ghost_router.rs:3435`, `:3519`, `:3603`)
        index rows by **`rs.spec.outputs` containing the item**, not
        through any lane. A row is reachable there regardless of what
        the lane planner did, so the original inference was not merely
        unverified — it was insufficient.
      - *The fix is to remove the hazard, not detect it.* A cell emits
        **one fused `RowSpan`**, not two. Its `spec` carries the
        producer's inputs and the consumer's outputs (the cell really is
        a composite machine: it eats iron-ore off a belt and emits gears
        onto one), and its `output_belt_y` is the cell's real output
        belt. **The producer never gets a row of its own**, so no row
        with a phantom output belt is ever constructed and all 11 read
        sites are correct by construction. This also spares every
        downstream consumer — lane planner, output merger, validators —
        from special-casing cells.

      **What the audit does and does not establish.** It shows #447's
      *argument* was insufficient — three sites are reachable by a path
      `di_input` does not guard. It does **not** show the property
      itself is false. In fact the property probably does hold today,
      via a guard #447 never cited: the merger loops iterate
      `output_items`, built from `solver_result.external_outputs`, and
      `detect_di_couplings` only couples an item with **no surplus**, so
      a coupled item is never an external output and the producer row is
      never selected there. Both #447 remedies were therefore
      *reasonable*; an empirical check would have been a valid way to
      test the claim. The fused row is preferred not because they were
      wrong-headed but because it **stops depending on the question**:
      its correctness rests on the row owning a real belt, not on a
      solver-side invariant ("coupled items never carry surplus") that
      Phase 3 could plausibly relax; (2) intercept the producer/consumer
      spec pair
      in `place_rows` — note this restructures the main placement loop
      (skipping a spec, custom `y_cursor` accounting, `module_id` and
      stacking-context threading), which is the highest-risk part of
      this step; (3) lane-planner skip for the coupled item —
      `di_input` already exists and is item-keyed, so reuse it rather
      than inventing a second mechanism; (4) fall back to the existing
      bridge, then the bus, whenever `plan_straddle` returns `None` or
      the ladder cannot supply `required_rate()`.
    - **Verification this step needs** (the earlier phases did not, being
      pure functions): the full layout-engine protocol — snapshot
      inspection at the cell's coordinates and a browser eyeball, not
      just a green suite. A DI cell that validates clean but starves is
      exactly the #383/#432 failure mode.
- **Phase 2 — face allocation, now including fluids (re-scoped by the
  KC6 trip).** The consumer's remaining flows on the opposite face,
  mixed reach (reach-2 stepping over a near belt), **plus pipe placement
  for fluid-touching machines** — the 30.6% of top-10 demand Phase 1
  cannot serve. Orientation search is required here because fluid ports
  are prototype-fixed per direction.
- **Phase 3 — ratios + fallback policy.** Multi-producer straddle for
  the corpus's awkward ratios; decide whether #432's bridge is still
  needed anywhere.
- **Phase 4 — close-out.** Sim verification, `direct_insertion` through
  wasm + web UI (URL state), CLAUDE.md/status refresh.

## Decision log

- *2026-07-24 — drafted. Claimed RFC-053: RFC-052 is
  `rfc-052-oil-mega-cell.md` (composition lane) — the main checkout's
  registry copy was stale, so the number was verified against
  `origin/main` before claiming (the known parallel-session collision
  hazard). Evidence base: corpus mining run of the same date (4,116
  cable→EC DI instances, gap=1 dominant, lateral straddle visible) and
  the `machine_feed_rate` table showing stack at 12.0/s machine-to-
  machine versus long-handed's 4.8/s reach-2 ceiling. Relationship to
  #432 recorded explicitly: that PR's belt→belt bridge is a different,
  lesser mechanism (3 hops, 2 belts) that is correct and honestly
  validated as a trunk-lane-removal optimization, and is retained here
  as a fallback rather than deleted. Pending review.*

- *2026-07-24 — **adversarial review: REVISE; both MAJOR findings
  incorporated, plus one self-caught error.** The reviewer independently
  re-derived every numeric claim (the `machine_feed_rate` table traced
  through `inserter_throughput`/`inserter_hand`, the recipe ratios from
  `recipes.json`, the 3:2 machine split), re-derived the straddle
  column-overlap from the stated x-positions, and **re-ran the corpus
  miner in release mode — reproducing the geometry table byte-for-byte**
  (172 files parsed, 98 with DI, 4,116 cable→EC, top-20 summing to
  3,866). No error found in any of those. Fixes applied:*
  - ***MAJOR 1 — the `#432` dependency was stated as present-tense
    fact.*** `DICoupling` / `di_couplings` / `direct_insertion` /
    `is_di_bridge_inserter` do not exist on `main`; they live on the open
    PR #432. New "Prerequisite" section states the dependency, scopes
    Phase 0 as the only phase that can start now, and records
    rebase-drift risk.
  - ***MAJOR 2 — `max_inserter_tier` was never addressed.*** The design
    is stack-load-bearing, but that tier cap is an existing hard user
    constraint *orthogonal* to `inserter_capacity`; the draft conflated
    the two axes. Added the per-edge slot derivation (the real budget is
    per-edge, not the machine's 3 columns), the resulting clean
    feasibility rule **`machine_feed_rate ≥ 2.5/s`**, the tier×level
    matrix, and kill criterion 2b. Outcome: feasible at engine defaults
    (Stack, L2) *and* for a `Fast`-capped user at default research —
    infeasible only for `Regular` below L7 or true L0 without stack
    inserters, where DI is refused with an honest warning rather than
    silently under-fed.
  - ***Self-caught (surfaced by the user asking about 2× long-handed):
    the `gap=2` corpus rows were mischaracterized.*** The miner requires
    BOTH pickup and drop to resolve to machine tiles, so all 4,116
    instances are machine→machine — the 200 `gap=2` rows are a
    legitimate long-handed machine→machine variant (one spare tile for
    routing), and #432's belt→belt shape is uncountable by the miner,
    appearing **zero** times rather than as a "~5% minority". Corrected,
    and the `gap=2` variant is now a first-class library pattern.
  - *Minor: reconciled the 4,116-vs-3,887 corpus divergence (Rust vs
    Node extractors, the #389 tolerance class); corrected
    `solid-fuel` → `solid-fuel-from-light-oil`; sharpened kill
    criterion 6 from a Non-goals restatement into a measurable
    fluid-coverage threshold. Pending user approval.*

- *2026-07-24 — **PHASE 0 COMPLETE. KC1 passes; KC6 FIRED and forced a
  re-scope.** Census of the corpus's top-10 DI pairs
  (`crates/core/examples/di_phase0.rs`) — full table in the Phase 0
  section.*
  - ***KC1 (ratio feasibility): PASSES with margin.*** Worst case across
    all ten pairs is the canonical cable→EC at 2.50/s per slot; a stack
    inserter at the L2 default supplies 19.2/s. The straddle
    construction is nowhere near its limit. This was the criterion most
    likely to kill the RFC outright, and it didn't.*
  - ***KC6 (fluid coverage): FIRED — 5/10 against a threshold of 2.***
    Diagnosed as a specification defect in the criterion itself, on two
    independent grounds: (a) it conflates a *fluid coupling* (impossible
    — inserters cannot move fluid, so every DI'd item is solid) with a
    *fluid-adjacent machine* (common — the fluid is a separate
    ingredient needing a pipe on a non-DI face); (b) its rationale is
    about excluded *demand* but it counts pairs unweighted — by
    instance weight, solids-only covers **69.4%** (5,448/7,848) and the
    dominant pair, cable→EC at 52.4% alone, is fully solid.
    **Resolution is the criterion's own prescribed action — re-scope,
    not reprieve**: pipes move out of Non-goals into **required Phase 2
    scope**; Phase 1 stays solids-only, now justified by measured weight
    rather than assertion. Process disclosure recorded in-section: the
    defects were diagnosed after the criterion fired, which is exactly
    the pattern kill criteria exist to catch, hence taking the
    prescribed action rather than deleting the criterion.*
  - *Data-quality finding: `electric-furnace → electric-furnace` (1,585
    instances, the 2nd-most-common DI pair) is invisible to the census
    because furnaces carry no explicit `recipe` — smelting-column DI
    will need its own handling and is not covered by this analysis.*
  - *Phase-0 debt carried to Phase 1: emit the canonicalized,
    version-gated `di_pattern_library.rs` (the census currently lives in
    a gitignored example) and close the furnace recipe-resolution gap.*

- *2026-07-25 — **Phase 1a landed: `bus::di_cell::plan_straddle`.** The
  straddle is derived from FLOW-INTERVAL OVERLAP — producer `i` owns
  `[i·prod, (i+1)·prod)` of the item stream, consumer `j` owns
  `[j·demand, (j+1)·demand)`, and their intersection is the flow that
  must cross that edge — then each consumer is positioned so its column
  overlaps split in proportion to those flows. That construction
  reproduces the RFC's published geometry exactly (consumer origins
  `[1,5,10,14]`, eight edges alternating 5.0/2.5 across 2/1 slots,
  `required_rate` 2.5/s) WITHOUT being fitted to it, which is meaningful
  corroboration that the hand-derived Design-section example was right.
  Also pinned: both-sides balance (every producer ships exactly 5.0/s,
  every consumer receives exactly 7.5/s), the tier×level feasibility
  matrix against the real `machine_feed_rate` table, 1:1 couplings
  needing no offset, and non-overlap of consumer machines. Deliberate
  refusals rather than approximations: unbalanced couplings (the surplus
  has to reach the bus — not a DI cell), and consumers straddling more
  than two producers (Phase 3's multi-band cell). 8 unit tests; full
  suite 863 lib + 61 e2e green, clippy clean. Remaining for Phase 1b is
  the placer wiring — belt suppression, 1-tile row gap, reach-1 inserter
  stamping from the planned edges, lane-planner skip.*

- *2026-07-25 — **KC4 (density premise) evaluated: PASSES.** Measured
  rather than asserted: on the canonical fixture the bus needs a
  17-tile vertical extent for the cable→EC coupling (cable row y1–7, EC
  row y10–17, plus the copper-cable trunk lane at y7–9), against the
  cell's 7 tiles. 59% smaller, so the premise the RFC rests on is
  confirmed against the real engine rather than the drawing. **With
  this, every kill criterion evaluable without the placer wiring has
  been evaluated and the RFC survives all of them**: KC1 passed with
  margin, KC4 passes here, KC6 fired and was resolved by its own
  prescribed re-scope. KC2/KC3/KC5 are inherently post-wiring (face
  contention, sim-measured throughput, solver escalation) and remain
  open against Phase 1c+.*

- *2026-07-25 — **Phase 1b landed: `bus::di_cell::stamp_di_cell`.** Turns
  a `StraddlePlan` into entities — producer row, ONE-tile inserter band,
  consumer row (7 tiles for a 3-tall coupling, against ~13 for #432's
  belt bridge). The test that matters asserts the defining DI property
  directly: every stamped inserter is reach-1 and its pickup tile lands
  on a PRODUCER machine while its drop tile lands on a CONSUMER machine,
  and the cell emits **no belt at all** for the coupled item — the same
  predicate `classify.rs` uses to count DI in community blueprints, so
  passing it means the engine now emits the shape the corpus is full of.
  Inserter counts follow the per-edge budget: 8 at stack tier (one per
  edge, matching the Design section), 12 with fast at the L2 default (the
  5.0/s edges take both their slots), and a REFUSAL below the cell's
  required rate rather than an under-fed cell. 7 more unit tests (15 in
  the module); 870 lib + 61 e2e green, clippy clean. Remaining: Phase 1c,
  the placer wiring — nothing calls these functions yet, so engine
  behaviour is still unchanged.*

- *2026-07-25 — **The Phase 1c contract question is resolved, and the
  premise it rested on was false.** #447 recorded the open question as
  "does anything read a cell producer row's `output_belt_y`?", with the
  supporting inference that `lane_planner` only ever reads
  `output_belt_y_for(item)` during lane construction. Auditing all
  **11** bare `output_belt_y` reads killed that inference: three of them
  (`ghost_router.rs:3435/:3519/:3603`, the output merger) select rows by
  `rs.spec.outputs`, never touching a `BusLane`, so `di_input` cannot
  protect them. **Corrected after review (#449):** the first draft of
  this entry escalated that into "the property is false, so both #447
  remedies were aimed at the wrong target" — an overclaim the audit does
  not support. Reachability by an unguarded path is not the same as a
  phantom value actually being read. The property probably *does* hold
  today, via a guard #447 never cited: the merger iterates
  `output_items`, built from `solver_result.external_outputs`, and
  `detect_di_couplings` only couples items with **no surplus**, so a
  coupled item never appears there. An empirical check would have been a
  perfectly valid way to test it. What the audit actually establishes is
  narrower and still worth having: #447's stated *argument* did not
  cover all the read sites, and the real guard is a different, narrower
  one than the entry claimed. **Resolved instead by
  design**: a cell emits ONE fused `RowSpan` (producer's inputs +
  consumer's outputs, cell's real output belt) rather than two, so no
  row with a phantom output belt exists to be read. Recorded because the
  general lesson is cheap and repeatable: an inference about a field's
  read-paths is only as good as an exhaustive grep for that field, and
  #447's was a partial one. The **second** lesson came from the review
  bot catching this entry's first draft overclaiming — "incomplete
  argument" was evidence-backed, "false property" was not, and the two
  are easy to conflate when you have just found a gap in someone's
  reasoning. The fused row's real virtue is that it needs neither
  question settled: it depends on the row owning a real belt, not on a
  solver-side invariant that Phase 3 could relax.*

- *2026-07-25 — **Phase 1's wiring target is retargeted off the RFC's own
  worked example.** `electronic-circuit` has two solid inputs
  (`iron-plate`, `copper-cable`), so `copper-cable → EC` — the pair every
  geometry figure in this RFC is derived from — violates Phase 1's stated
  scope ("consumer's only solid input is the DI'd item"): the consumer's
  two usable faces are already spent on the DI band and the output
  inserter, leaving the iron-plate feed nowhere to land until Phase 2.
  The geometry work is unaffected (`plan_straddle` is rate-driven and
  item-agnostic), but 1c must be built and tested against a
  single-solid-input pair — `iron-plate → iron-gear-wheel` for the
  assembler case, `iron-plate → steel-plate` for the furnace→furnace
  shape that dominates the corpus at 1,585 instances. **The eligibility
  gate belongs in the placer**: `detect_di_couplings` gates on
  one-producer/one-consumer/no-surplus/not-fluid and never inspects the
  consumer's input count, so it emits the cable→EC coupling regardless;
  1c wiring that trusted the coupling list would emit a clean-validating,
  starving cell. Caught by checking the recipe DB before writing the
  wiring rather than after — the RFC had carried the contradiction
  since the first draft because prose scope and worked example were
  never cross-checked.*

- *2026-07-25 — **Phase 1c landed; Phase 1 is complete.** `place_rows`
  fuses an eligible producer/consumer pair into a single cell row, so the
  engine emits true machine→inserter→machine DI for the first time.
  Three things worth carrying forward. **(a) The fused-row design made
  the "invasive" step small.** The pick-up notes predicted a new
  `RowKind` and a restructured placement loop; what it actually took was
  a precomputed `cell_pairs` map, a skip-set for absorbed consumer specs,
  and a `RowSpan` built straight from `DiCellLayout`. The reason is the
  design choice, not luck: describing the cell as a composite machine
  (producer's inputs, consumer's outputs) means nothing downstream has to
  learn what a cell is. **(b) `cell_eligible` is load-bearing.** It is
  the only thing standing between `detect_di_couplings`' cable→EC
  coupling and a starving cell — the solver will keep emitting couplings
  Phase 1 cannot serve, and that is fine as long as the placer refuses
  them. **(c) A test passed vacuously and had to be caught by hand.**
  `di_cell_row_has_a_real_output_belt` went green while no cell was being
  built at all — the shared `iron_plate_spec`/`iron_gear_spec` helpers
  are unbalanced (1.0/s against 2.0/s), `plan_straddle` rightly refused
  them, and the assertion silently landed on an ordinary row. It now
  guards on `spans.len() == 1` first. Textbook "zero warnings can mean
  the check was wrong" from the verification protocol, and worth the
  reminder that a green new test is evidence of nothing until you have
  seen it fail for the right reason.*

- *2026-07-25 — **KC3 evaluated and PASSES; the RFC now survives every
  kill criterion that is evaluable pre-Phase-2.** Sim-measured a real
  cell built by the full solver→layout pipeline (`steel-plate@2/s` from
  `iron-ore`, 16:16 furnace→furnace): 2.24/s delivered against 2.00/s
  planned, `converged=true`, 32/32 machines working. The floor is 98%;
  this is 112%. **Ran a DI-off control on the same target and that is the
  finding worth keeping** — the control delivers *the same* 2.24/s
  (+12.0%), so the overshoot is a solver rate-model artifact
  (electric-furnace steel under-predicted by ~10%), not a property of
  DI. Without the control the +10.7% would have been unattributable, and
  the temptation to read it as "DI beats plan" would have been strong
  and wrong. The same pair of runs re-confirms KC4 end-to-end at a scale
  the hand-derived 7-vs-17 figure couldn't: **213 entities vs 335, −36%**,
  at identical delivered throughput. Chose to run KC3 before starting
  Phase 2 specifically because Phase 2 is the expensive phase and KC3 is
  the criterion most able to invalidate the topology — evaluating it the
  moment it became evaluable (which Phase 1c's landing made possible) is
  the RFC's own kill-criterion discipline. Two gates remain open and both
  are inherently Phase 2+: KC2 (face contention — a Phase 1 cell has no
  second face flow by construction) and KC5 (solver escalation bound).
  Orthogonal follow-up, deliberately NOT folded into this RFC: the ~10%
  electric-furnace steel rate discrepancy the control exposed.*

- *2026-07-25 — **#450 review found three real bugs; a fourth was found
  by closing my own verification gap.** The bot's findings, all
  confirmed against the code before fixing rather than taken on trust:
  **(1)** `fused_cell_spec` dropped the producer's module loadout — the
  module post-pass keys `(entity, recipe)` off `row_spans`, and a cell
  contributes only the consumer's recipe, so producer machines would get
  no modules while the solver had already folded the bonus into their
  count. Now refused outright (matching loadouts wouldn't help: the
  *key* is missing). **(2)** `SidePlan.count`/`.shortfall` were computed
  and discarded — `DiCellIo` carries an entity name and no count, so a
  face needing two inserters got one and silently under-fed. Now sized
  against a single slot and refused when that isn't enough; the comment
  claiming "a single-inserter face isn't an implicit throughput cap"
  asserted the exact opposite of the behaviour. **(3)** the cell's
  output belt was sized against the raw combined rate although it is
  physically single-lane (all output inserters share a y and a facing,
  so every drop lands in the far lane) — a 2× over-estimate, and cells
  never split by throughput the way ordinary rows do. Now sized at
  `rate * 2.0` and refused when one lane can't carry it. **(4)** Mine:
  the KC3 run measured throughput but never ran the validator, so
  "validates clean" went unverified — and was false (16 errors). Fixed
  via `is_di_cell_entity`. The through-line in all four is the same
  shape of mistake — trusting a number that was never checked
  (`.count`, lane capacity, the module key, the warning list) — which is
  the failure mode this RFC's own verification protocol exists to catch,
  and which three of four green test runs did not surface.*

- *2026-07-25 — **Two more #450 findings, one of them missed on the first
  fold-in, plus a CI gap that made every green on that PR meaningless.**
  (a) The cell's input belt was sized from the cell's LOCAL demand, but
  it is a bus tap-off target like any other row's input — `lane_planner`
  taps it identically — so it must take the trunk tier via
  `row_input_belt`, whose doc comment exists precisely to prevent the
  seam mismatch I reintroduced (fast trunk feeding a yellow row belt,
  lane-throughput warnings, items backing up at the join). I fixed the
  output belt in the first pass and did not notice the input belt had the
  same class of bug. (b) Neither belt refused on capacity:
  `belt_entity_for_rate_stacked` **saturates** rather than failing, and
  `plan_straddle` is scale-invariant in machine count, so any pair
  eligible at rate R stays eligible at 100R while the belts silently
  under-carry. Both sides now refuse. (c) **`ci.yml` triggers on
  `pull_request: branches: [main]`, which filters the PR's BASE branch —
  so #450, stacked on #449, got `ci.yml runs: 0` for its entire life.**
  The signal was subtle and worth recording: on a docs-only PR the rust
  jobs show `SKIPPED` (the `changes` filter ran), whereas on #450 they
  were **absent** — never scheduled — while the two workflows that
  aren't base-filtered passed and made the checks list look plausible.
  This affects any stacked PR in the repo; widening the trigger is a
  workflow-file change and deliberately left for a reviewed PR of its
  own, given #369's history of an installer silently reverting these
  files. Also worth noting for the next session: the review bot
  **skipped** its later passes ("Claude has already left review comments
  on this PR"), so a green `claude-review` on the fixed SHA is not
  evidence the fixes were reviewed — the CLAUDE.md warning applies to
  re-reviews too, not just first passes.*

- *2026-07-25 — **Phase 1 coverage measured, so "Phase 1 complete" isn't
  taken on faith.** Five refusal gates landed during the #450 review
  fold-in; if they collectively refuse almost everything, the phase is
  hollow. `di_cell_coverage_sweep` (ignored, in `tests/e2e.rs`) reports
  cell-vs-fallback across 11 real targets. At yellow: **4 build cells**
  (`steel-plate` at 1 and 2/s, `iron-stick`, `pipe`, `copper-cable`),
  and every refusal has an explicable cause rather than a silent one —
  `electronic-circuit` is Phase 2 (two solid inputs), `iron-gear-wheel`
  is out on ratio (~4.8 furnaces per gear machine, straddle > 2),
  `stone-brick` yields no coupling at all, and the high-rate
  `steel-plate` cases hit belt capacity. **The capacity refusals track
  real belt limits, verified by re-sweeping at each tier**:
  `steel-plate@5` needs 25/s of iron-plate, so it refuses on yellow
  (15/s) and builds on red (30/s); `steel-plate@10` needs 50/s and
  refuses even on express (45/s), which is correct — one belt cannot
  feed it. That last case is the real Phase 1 ceiling and it is a
  **fan-in** limit, not a DI limit: the ordinary path would split the
  recipe across rows, and a cell cannot (its machines sit at
  `StraddlePlan` positions), so high-rate couplings need the Phase 3
  multi-band cell. Recorded as a measurement rather than a guess because
  the gates were added under review pressure and their aggregate effect
  was not obvious from any individual one. NB the sweep only varies
  `max_belt_tier` to characterise the ceiling — the engine must never
  auto-escalate tier, which stays a hard user-specified constraint.*

- *2026-07-25 — **Phase 3 (multi-band) measured before building, and the
  measurement argues against doing it next.** Extended `di-patterns` with
  a `fan` subcommand — `Obs` previously carried only recipe names and
  relative geometry, so fan-in/fan-out/chain were structurally
  uncomputable; it now records the blueprint-member id and the two
  machine indices. Corpus results:*

  | shape | fan-in (producers per consumer) | chain (machine is both) |
  |---|---|---|
  | `electric-furnace → electric-furnace` (1,585) | **1 for all 1,585** | **0** |
  | `copper-cable → electronic-circuit` (4,116) | 1 for 634, **2 for 1,405; max 2** | **0** |
  | all pairs (16,507) | >2 for 237 of 11,536 consumers = **2.1%** | 1,674 / 21,622 = 7.7% |

  *Three conclusions. **(1) `plan_straddle`'s existing ≤2 limit already
  covers ~98% of real fan-in.** The RFC's stated Phase 3 motivation —
  "multi-producer straddle for the corpus's awkward ratios" — is chasing
  a 2.1% tail. **(2) Neither dominant shape uses stacked bands at all**
  (chain = 0 for both); the 7.7% global figure lives in pairs nobody has
  asked for. **(3) cable→EC needs exactly the 2-producer straddle Phase 1
  already implements** — what blocks it is the second solid input, i.e.
  Phase 2 face allocation, not the straddle. Separately, the Phase 1
  ceiling measured earlier (`steel-plate@10` refusing on input-belt
  capacity) is **not** a straddle problem either: it needs more than one
  input belt feeding a cell, which multi-band does not address. So the
  honest ordering is **Phase 2 before Phase 3**, and Phase 3's scope
  should be rewritten around the fan-in belt limit rather than around
  multi-producer straddle, which is largely already solved.*

- *2026-07-25 — **Phase 2's input-belt contract, checked before building:
  one question discharged, one sharpened, one found.** The Phase 2 cell
  puts a second input belt (iron-plate) BELOW its consumers, a shape no
  existing row template produces, so the worry was that something derives
  tap-off position from row geometry rather than reading it. **It does
  not.** Both consumers — `lane_planner.rs:1292` and
  `ghost_router.rs:163` — use the identical form: filter `spec.inputs` to
  non-fluid, enumerate, match on item, then read `input_belt_y[idx]`
  literally. No `y_start` arithmetic, no "first belt row", no comparison
  against machine y. An input belt below its machines is fine.*

  *So the whole contract reduces to: **`input_belt_y[i]` is the belt for
  the i-th non-fluid entry of `spec.inputs`, in spec order.** Violating it
  makes BOTH consumers wrong identically, so they agree with each other
  and yield a self-consistent wrong layout — there is no disagreement for
  a check to catch.*

  ***The trap the grep actually found.*** *Both consumers `break` on the
  first item match, and `ghost_router` documents the assumption inline:
  "assumes one input slot per item per recipe." A Phase 2 fused spec is
  producer's inputs + consumer's non-coupled inputs, and nothing prevents
  those being the SAME item (producer eats iron-plate, consumer also eats
  iron-plate). Then `spec.inputs` holds two iron-plate entries at
  different y, both consumers match the first and break, and **the second
  belt is never tapped** — built, never fed, machines starve, and no lane
  is ever routed to it to warn about. Merging the two is not available:
  they sit at different y by construction (one above the producers, one
  below the consumers). Phase 2 must therefore either refuse such a pair
  outright — the Phase 1-style honest answer, and the default unless
  measurement says the shape matters — or teach tap-off resolution to sum
  across matching slots, which means editing that documented assumption in
  two files. Invisible in the canonical cable→EC case (copper-plate vs
  iron-plate are distinct), so it would have shipped and bitten later.*

- *2026-07-25 — **Face allocation mined instead of drawn, and it overturns
  two things this RFC asserted.** New `di-patterns faces` subcommand: for
  every machine that RECEIVES direct insertion, report which sides the DI
  arrives on and where all its other interfaces live (side, reach,
  in/out, belt-or-machine at the far end). 2,039 cable→EC consumers.
  Top whole-machine plans:*

  | n | plan |
  |---|---|
  | **177** | `DI@E+W \| S:in1→belt S:out1→belt` |
  | 110 | `DI@S \| N:in1 N:out1 N:out2 N:out2` |
  | 80 | `DI@N \| S:in1 S:out1 S:out2 S:out2` |
  | 68 | `DI@W \| N:in1 N:in1 S:out1 S:out1` |

  ***(1) The dominant shape is a horizontal straddle in ONE row, not a
  stacked cell.*** *The most common plan has the consumer between two
  producers on its east and west faces, with its remaining input and its
  output both on the south face and **both reach-1** — north entirely
  free. That is `P C P C P` interleaved in a single row with ordinary
  belts above and below, which is much closer to what `place_rows`
  already does than the producer-row-above-consumer-row cell Phase 1
  builds. The RFC's hand-drawn sketch (rows 2 and 3 above, 190 combined)
  is real but is not what most people build.*

  ***(2) KC2's "zero margin" was computed on a false premise.*** *That
  evaluation assumed the consumer's spare face is ONE inserter row of 3
  columns, and concluded 1 near + 2 far exactly fills it. But plans 2 and
  3 carry **four** interfaces on a single face (`in1 out1 out2 out2`),
  which three columns cannot hold. The resolution is that a reach-2
  inserter can sit in the SECOND row out — at `y+h+1`, picking from the
  machine at `y+h-1` and dropping at `y+h+3` — so the face is two rows
  deep. KC2 still passes; its margin is simply wider than recorded, and
  the "no spare column, so a third flow must refuse" consequence I drew
  from it does not follow. Both the criterion's arithmetic and the Phase
  2 geometry derived from it need redoing against the two-row face.*

  *Method note, since this is the second time it has bitten: the sketch's
  reach arithmetic was verified and that was mistaken for validating the
  design. Checking that a drawing is physically possible says nothing
  about whether it is good or common. The corpus could have answered this
  at any point in the last three phases and was not asked.*

- *2026-07-25 — **Phase 2 pivots to the horizontal row straddle, on
  corpus evidence, and it BENDS LESS than the stacked cell rather than
  more.** `plan_row_straddle` lands in `bus::di_cell`: producers and
  consumers interleaved in ONE horizontal row, coupled by inserters in the
  1-tile gaps between neighbours. Same flow-interval argument as
  `plan_straddle`, applied in 1-D, with the extra constraint that a
  consumer has only two horizontal neighbours — so a consumer needing
  three producers is refused rather than approximated. It reproduces the
  hand-derived canonical sequence `P C P C P P C P C P` for the 6:4
  cable→EC ratio without being fitted to it.*

  ***Why this is the better shape, and why it is cheaper:***
  - *`required_rate() == 5.0/s` for cable→EC, because each edge owns
    exactly one gap. A stack inserter moves **12.0/s at zero research**,
    so the pair is feasible with no research at all — against the stacked
    cell's face plan, which needed L2 and two long-handed inserters.*
  - ***No reach-2 inserter anywhere.*** *The stacked cell's whole
    difficulty was the consumer's spare face carrying two flows, forcing
    a long-handed hop over the near belt at 2.40/s.*
  - ***It reuses `place_rows` rather than replacing it.*** *A line of
    machines with belts above and below is what the row templates already
    emit; the delta is a mixed-recipe machine sequence plus inserters in
    the gaps. The stacked cell needed a bespoke stamper. Per the standing
    guidance — reuse where we can, extend where we must — this is the
    cheaper extension AND the one the corpus endorses.*

  *Consequence for the phase: the mixed-reach face row, the two-row face
  budget, and KC2's margin all become moot for this shape, since the
  consumer's remaining input and output sit on one face at reach-1 with
  the opposite face free. They remain relevant only to the stacked
  variant, which the corpus puts second (190 combined vs 177 for the top
  single plan).*

- *2026-07-25 — **Phase 2 row cell builds cable→EC end to end; 9
  validation errors remain, and they are OURS.** `stamp_row_cell` +
  `try_build_row_cell` produce a horizontal row cell for
  `copper-cable → electronic-circuit` at 10/s from ore (153 `di-row`
  entities, express belts) — the corpus's #1 DI pair, and the first time
  the engine has used DI for it. **Not merged**: on
  `wip/rfc053-phase2-row-cell`, off #452's branch, which stays green.*

  ***The control run is the load-bearing datum.*** *The identical target
  with `direct_insertion: false` validates at **0 issues**, so all 9
  errors are caused by the row cell rather than being pre-existing. No
  need to re-establish that next session.*

  *Blocking error, diagnosed: a vertical `fast-transport-belt` at x=46 is
  routed through occupied tiles at y=20–25, colliding with an
  `electric-furnace` (from the copper-plate producer row) and with cell
  entities. The row cell is ~39 tiles wide at pitch `machine_w + 1`,
  wider than any ordinary row, and something in lane/return routing is
  not respecting occupancy across that span. Ruled out already: the
  `input_belt_y` ordering contract holds (fused inputs are
  `[copper-plate, iron-plate]`, belts are `[y0, face_y+1]`, matching).*

  *Four bugs fixed reaching this point, each a silent mis-build:
  `cell_pairs` required exactly one coupling per consumer, excluding
  `electronic-circuit` (coupled on two items) entirely; claiming a pair
  on eligibility alone let the unbuildable 16:4 `iron-plate → EC`
  coupling block the workable `copper-cable → EC` one; the build path
  called `try_build_cell` unconditionally, bypassing the gate that stops
  a two-solid-input consumer being fused into a STACKED cell; and the
  producer's feed was reach-2, reintroducing long-handed's 2.40/s ceiling
  that the row shape exists to avoid.*

  *Also outstanding: 2 of the new `row_stamp` tests still assert the
  pre-rework geometry, and `di_bridge_feeds_cable_only_at_high_research`
  is a genuine regression — cable→EC now forms a row cell where that test
  pins bridge behaviour. Decide whether it should assert the row cell or
  pin DI off.*

- *2026-07-25 — **A latent output-merger bug, found by the row cell; and
  a KC3-CANDIDATE TRIP that needs sim forensics before it can be
  called.** Two things, one good and one that must not be waved through.*

  ***(1) `merge_x_cursor` never honoured its own stated invariant for
  single-output layouts.*** *The comment says "Start east of EVERY row
  (not just the participating ones) so south columns never clip a wider
  foreign row" — but only the `output_items.len() > 1` branch did that;
  the single-item branch started at `0` and let `merge_output_rows`
  derive its start from the participating rows alone. Safe only while
  the output-producing row is also the widest, which is true of ordinary
  layouts and false for a row cell: at EC@10/s the cell is `bus+39` wide
  against the iron-plate row's `bus+48`, so the merger drove a column
  straight through it — 7 `entity-overlap` errors, plus knock-on
  reachability and isolation failures. Fixed by applying the max
  unconditionally. **Verified pre-existing, not introduced**: the same
  target with `direct_insertion: false` validated at 0 issues, and the
  fix adds no new test failures (895 pass, same 5 known ones). This bug
  was reachable before RFC-053 by any layout whose final row is narrower
  than an intermediate one; the row cell just made it easy to hit.*

  ***(2) cable→EC now VALIDATES CLEAN but SIMS AT ZERO.*** *After the
  merger fix the layout has **no validation errors** (8 warnings), so
  goal clause (b) holds. The sim says: `electronic-circuit` 0.00/s
  against 10.00/s planned, `converged=false`, and at a 60,000-tick
  warmup it degrades to "NO DATA" with every item at 0.00 — including
  `copper-plate` and `iron-plate`, which come from ORDINARY rows, not
  cells. **This is the exact shape KC3 exists to catch** ("validates
  clean but the sim measures < 98% — the model is wrong and the checks
  are lying"). It is NOT yet a confirmed trip: universal zeros plus
  "NO DATA" plus starved non-cell rows is also the documented signature
  of a sim-KIT problem (`docs/sim-harness-forensics.md`; audit
  `kit_errors` and the chest census before blaming geometry). Those two
  diagnoses have opposite consequences — one kills the Phase 2 shape,
  the other is a harness artifact — so the forensics must be run before
  either is recorded as fact.*

  ***Forensics run; it is NEITHER.*** *The machine census settles it:
  **`full_output: 46`, `item_ingredient_shortage: 4`**. Forty-six
  machines are backed up with a FULL OUTPUT — producing normally and
  unable to offload — against only four short of ingredients. Import was
  clean (654/654 ghosts revived), power fine (1 pole network), no kit
  errors. So:*
  - *Not a sim-kit artifact: the kit fed the layout and the machines ran.*
  - ***Not a KC3 trip.** KC3 fires when the topology or the model is
    wrong. `full_output` means production works and the OFFLOAD PATH is
    blocked, which is an incomplete integration in unfinished Phase 2
    code, not evidence against the DI shape. Recording this distinction
    because the raw numbers (0.00/s, -100%) look identical to a KC3 trip
    and would have justified killing the row shape on a misreading.*

  *Where the blockage is, from the same run: `copper-plate` produces
  1.27/s then stalls and `copper-cable` produces 0.00/s, so the cable
  machines never receive copper-plate. The cell's belts carry `di-row:`
  segment ids rather than the `row:...:belt-in:<item>` ids ordinary rows
  use, so the tap-off almost certainly never joins them to the bus —
  consistent with the belts being stamped by the cell rather than by the
  row templates.*

  ***That hypothesis is DISPROVEN — probe it before acting on it.*** *Both
  cell input belts do have tap-offs joined to them:
  `copper-plate` at y=10 (x=6..44) with `ghost:tap:copper-plate:3:10`
  immediately west, and `iron-plate` at y=16 with
  `ghost:tap:iron-plate:4:16`. The stamped geometry matches the design
  exactly (`p_belt=10, feed=11, machines=12-14, face=15, c_belt=16,
  out=17`). Feeding is not the problem.*

  ***Current best candidate: the merger fix traded an overlap for a
  gap.*** *The cell's output belt ends at **x=44**, but
  `merge_x_cursor` now starts east of EVERY row, and the copper-plate row
  is ~48 wide — so the merge column begins beyond the cell's output belt.
  If that belt is not extended east to meet it, `electronic-circuit` has
  nowhere to go, backs up, and cascades upstream into precisely the
  observed `full_output: 46`. Next session: check whether
  `merge_output_rows` extends a participating row's output belt east from
  `output_belt_x_max` (ordinary rows must rely on this), and if it does,
  why the cell row is excluded.*

  ***That is disproven too.*** *The EC output belt (y=17, x=6..44) IS
  joined east to the merger at x=45..48 (`merger:electronic-circuit`).
  Both inputs are tapped and the output is merged; the cell is fully
  connected on all three belts.*

  ***The census localises it exactly, and it is the Phase 2 input.***
  *50 machines: 40 furnaces + 6 cable + 4 EC. `full_output: 46` = the 40
  furnaces plus the 6 cable machines; `item_ingredient_shortage: 4` =
  exactly the 4 EC machines. So the EC machines cannot craft for want of
  an ingredient; their coupled cable input then fills, the couplers
  stall, the cable machines back up, and the backpressure reaches the
  furnaces. Copper-cable arrives by DI and its couplers are stalled
  BECAUSE EC cannot consume — so the missing ingredient is **iron-plate,
  the consumer's belt-fed second input**. That is precisely the flow
  Phase 2 exists to add and that Phase 1 never had to serve, so the
  defect is in the new south-face plan (consumer feed at `face_y`,
  reach-1, picking the belt at `face_y+1`), not in the coupling, the
  taps, or the merge. **Start there next session: verify a consumer feed
  inserter exists per consumer and actually picks iron-plate off y=16.**
  Three hypotheses have now been proposed and two killed by probing —
  probe this one before changing anything.*

  ***Probed; the face is correct too — fourth hypothesis eliminated.***
  *y=15 holds 12 inserters, exactly 4 consumers x (1 `fast-inserter`
  carrying iron-plate + 2 `long-handed-inserter` carrying EC), at the
  consumer x-positions 10/18/30/38, with the iron-plate belt at y=16 and
  the EC belt at y=17 beneath them. Reach arithmetic verified for all
  three roles.*

  ***Conclusion after four eliminations: the cell is STRUCTURALLY SOUND
  and the defect is DYNAMIC.*** *Taps connected, output merged, face
  inserters present and correctly placed — so this is not a placement
  bug. It is a flow problem: iron-plate is most likely never arriving on
  y=16 even though the tap SEGMENT exists at x=5. The next probe is
  whether that tap connects to its trunk at the FAR (west) end, and
  whether the iron-plate trunk is routed at all — note the iron-plate
  furnaces are themselves at `full_output`, which is consistent with
  their output having nowhere to go rather than with a full belt
  downstream. Do not change placement code: four structural hypotheses
  have now been proposed and all four killed by probing, which is
  itself the finding — the remaining fault is in routing/flow, not
  geometry.*

- *2026-07-25 — **PHASE 2 WORKS: both top corpus DI pairs now build,
  validate clean and sim at or above plan.** The root cause of the EC
  starve was ORDERING, not geometry — the fifth hypothesis after four
  structural ones were probed to destruction. A fused cell consumes the
  union of both halves' belt-fed inputs, so it must be placed where all
  of them are available: at the CONSUMER's slot in the topological order,
  not the producer's. Emitting at the producer's slot put the cell north
  of its own iron-plate supply (iron's row landed at y=22 against the
  cell at y=10–17), breaking the lanes-run-south invariant; the router
  could only answer with a 1-entity "return path", iron never arrived,
  the EC machines were ingredient-short and the whole chain backed up.*

  | pair | uses DI | validates | sim delivered |
  |---|---|---|---|
  | `copper-cable → electronic-circuit` (#1, 4,116) | 153 `di-row` | **0 issues** | **101.3%**, 50/50 working |
  | `electric-furnace → electric-furnace` (#2, 1,585) | 176 `di-cell` | **0 issues** | **109.5%**, 32/32 working |

  *Two further bugs fell out of that fix and are worth keeping: skipping
  the producer lazily was too late (it sorts earlier, so it had already
  been placed — its own output belt was stamped over the cell's
  iron-plate belt), so `fused_specs` is pre-populated from `cell_pairs`;
  and pre-populating made an unbuildable claim FATAL, because the
  producer would be skipped while the cell then refused, dropping its
  production silently — so selection now does a trial build at `y=0`
  (every refusal is y-independent) and only claims pairs that will
  actually build.*

  ***Followup, deliberately narrowed not dropped:*** *the `merge_x_cursor`
  fix is scoped to layouts containing a fused cell row. The unconditional
  form is the more principled reading of the invariant its own comment
  states, but it regressed `mega_chain_ac_from_raw_zero_issues`, and
  diagnosing why mega chains depend on the old cursor was out of scope.
  Scoping keeps every pre-existing layout bit-identical. **The
  unconditional form remains the right long-term fix** once that
  interaction is understood.*

  *Method note: five hypotheses, four disproven by probing before any
  code changed. The four eliminations (taps connected, output merged,
  face inserters correct, geometry sound) are what made the fifth
  findable — each had been committed to this log as a diagnosis and then
  retracted. Probing before fixing was the whole difference.*

- *2026-07-25 — **Pipe scope (the KC6 re-scope) measured, and it is much
  smaller than feared.** New `di-patterns fluid` subcommand reports, per
  DI machine, which sides carry an adjacent pipe. Two representative
  pairs:*

  | pair | producer | consumer |
  |---|---|---|
  | `casting-copper-cable → EC` (592 machines) | piped: S 85, E 58, W 53, N 25 | **NO PIPE ×298** |
  | `engine-unit → electric-engine-unit` (1,094) | **NO PIPE ×440** | piped: S 128, W 125, N 84 |

  ***Three findings.*** *(1) **Only ONE machine per pair is
  fluid-touching**, and cleanly so — the other never has a pipe.
  Inserters cannot move fluid, so the coupled item is always solid by
  construction; the fluid is a SIDE input. (2) **There is no canonical
  pipe face** — all four sides occur with comparable frequency, so the
  engine is free to choose rather than having to match a convention.
  (3) Decisively, from `recipes.json`: `casting-copper-cable`
  (molten-copper → copper-cable), `casting-iron` (molten-iron →
  iron-plate) and `solid-fuel-from-light-oil` (light-oil → solid-fuel)
  all take **only a fluid** and emit a solid.*

  ***Consequence: the fluid producer has NO solid input, so its north
  face — where a solid producer's feed belt and inserters sit — is
  entirely free, and the pipe goes exactly where the belt would have
  been.*** *That explains finding (2): there is no contention to force a
  canonical face. The change is therefore small and local: relax
  `row_cell_eligible`/`cell_eligible` to admit a producer whose inputs
  are all fluid, and have the stampers emit a pipe run adjacent to the
  machine row instead of a belt + feed inserters.*

  *Coverage: **1,535 corpus instances** across three top-12 pairs, and
  they split across BOTH cell variants — `casting-* → EC` is a ROW cell
  (EC has two solid inputs) while `solid-fuel-from-light-oil →
  rocket-fuel` is a STACKED cell (rocket-fuel's only solid input is
  solid-fuel). Both stampers need the treatment.*

  ***Out of scope for that first cut, deliberately:*** *the mirror shape,
  a fluid-touching CONSUMER (`electric-engine-unit` = engine-unit +
  electronic-circuit + lubricant, 547 instances). Its south face already
  carries a solid input and the output, so a pipe needs a face the row
  cell does not have spare — genuinely harder, and worth doing only after
  the easy 1,535 land. Verify the exact fluid-port tile against the
  existing `fluid_port_pipes` machinery before stamping: ports are
  prototype-fixed per direction, so a pipe run that merely LOOKS adjacent
  may not connect.*

  ***Implementation notes for the pipe cut, so they need not be
  rediscovered.*** *Do NOT derive port tiles by hand — a pipe run that
  merely looks adjacent may not connect, because ports are
  prototype-fixed per direction. Reuse the shared table-driven module the
  existing row templates already use (`bus/templates.rs:35`,
  `fluid_input_port_dx`):*
  - *`fluid_ports::north_input_orientation(entity)` → `(mirror, dir)`.
    **Place the producer machines at that orientation**, exactly as
    `single_input_row` does, so the delivered pipe lands on a real port.*
  - *`fluid_ports::north_input_dxs(entity, mirror, dir)` → the port
    columns.*
  - *Geometry: the pipe run must be **adjacent** to the machine row, i.e.
    at `machine_y - 1` (the row that holds feed inserters for a
    solid-input producer). The belt row above it is simply unused for an
    all-fluid producer. Consumers sharing the row are unaffected — EC has
    no fluid box, so a continuous pipe run passing over its columns forms
    no connection.*
  - *Lane-planner integration goes through `RowSpan.fluid_port_ys` and
    `fluid_port_pipes`, NOT `input_belt_y`: `lane_planner` has a separate
    fluid branch (`if !rs.fluid_port_ys.is_empty() { tap_ys.push(...) }`).
    The fused spec must therefore carry the producer's fluid input as a
    fluid `ItemFlow` and the row must populate those fields, or the
    molten-copper lane will never be tapped — the same class of silent
    starve that the iron-plate ordering bug produced.*

- *2026-07-25 — **CORRECTION: the pipe cut is built and unit-tested, and
  it unlocks NOTHING yet. The scoping above was wrong.*** *The stamper and
  eligibility changes landed (`fluid_producer_gets_a_pipe_run_on_a_free_north_face`
  pins the geometry), but every pair the pipe analysis claimed is blocked
  by a DIFFERENT prerequisite — found only by attempting an end-to-end
  build:*

  | pair | real blocker |
  |---|---|
  | `casting-copper-cable → EC` (544) | **foundry is 5×5, assembler 3×3** — heterogeneous footprints. `row_cell_eligible` requires equal dims because `plan_row_straddle` takes a single `machine_w`. |
  | `casting-iron → EC` (339) | same |
  | `solid-fuel-from-light-oil → rocket-fuel` (652) | **fluid on BOTH sides** — `rocket-fuel` takes light-oil as well as solid-fuel, so it is the fluid-CONSUMER shape that was explicitly scoped out. |

  *What went wrong, recorded because it is a repeatable mistake: the
  mining measured pipe ADJACENCY and recipe INPUTS, and both answers were
  correct. It never checked machine FOOTPRINTS, and it checked
  `electric-engine-unit`'s fluid needs (correctly excluding it) while not
  checking `rocket-fuel`'s. **Measuring the thing you thought of is not
  the same as measuring the thing that blocks you** — a build attempt
  found in minutes what three rounds of corpus mining had missed.*

  ***The real prerequisite for fluid DI is heterogeneous machine
  footprints, not pipes.*** *Two of the three pairs need only that; the
  third additionally needs the fluid-consumer face plan. The pipe code is
  kept rather than reverted because it is small, correct and pinned by a
  test — but it is honestly unreachable today, and **anyone picking this
  up should do footprints first**.*

- *2026-07-25 — **The `merge_x_cursor` "fix" was treating a symptom and is
  REVERTED.** #459's review found that `cell_rows_present` inspected
  `route_bus_ghost`'s own local entity accumulator — router-authored
  segments only — so it could never see placer-authored cell entities and
  the branch never fired. Tested rather than argued: forcing it `false`
  leaves BOTH pairs at 0 validation issues. The overlap it was written to
  cure was a symptom of the ORDERING bug (cell emitted at the producer's
  slot instead of the consumer's); once ordering was fixed the merger
  change became inert. Reverted to the original, which also retires the
  "narrowed followup" recorded earlier — there is nothing to narrow.*

  *The underlying observation still stands and is now the ONLY claim
  made: `merge_x_cursor`'s comment says it starts east of every row, but
  only the multi-output branch implements that. No layout is known to hit
  it. Left alone deliberately — the unconditional form regressed
  `mega_chain_ac_from_raw_zero_issues`, and a fix with no reproducing
  case is not worth that risk.*

  *Also from the same review: the row cell's input-belt capacity check
  derived its stacking factor from the PRODUCER's item and then gated the
  CONSUMER's belt with it, although `row_cell_eligible` guarantees the
  two items differ and `StackingCtx::for_item` is item-keyed. Now checked
  per item.*

  *Lesson worth keeping: two of this session's "fixes" — the merger
  cursor and the first pipe scoping — were confidently reasoned, landed,
  and later shown to do nothing. Both were caught by an experiment that
  took minutes (force the flag false; attempt an end-to-end build). The
  cheap experiment beat the careful argument every time it was run.*

- *2026-07-25 — **Heterogeneous machine footprints landed; `casting-* → EC`
  is STILL blocked, by a third prerequisite.** The row cell now paces x by
  each machine's own width and **bottom-aligns** the two roles so they
  share one south face row (top-aligning would leave a shorter machine's
  south face two tiles above the face row, unreachable by its own feed and
  output inserters). Couplers sit on the bottom row — the only row both
  roles are guaranteed to occupy. `row_cell_eligible` no longer requires
  equal dims; the STACKED cell still does, its straddle being derived
  from a single machine width. Unit-tested with a real 5×5 foundry beside
  a 3×3 assembler.*

  ***But the pair still refuses, on RATIO.*** *A foundry emits 8.0
  cable/s and an EC machine wants 7.5/s — a **16:15** ratio whose
  smallest integer solution is 15 producers : 16 consumers, i.e. 31
  machines emitting 40/s, which no single output lane carries (express
  caps at 22.5/s). Machine counts snap to integers, so at any smaller
  rate supply and demand disagree and `plan_row_straddle` correctly
  refuses. **`casting-* → EC` therefore needs ratio tolerance too** —
  either a straddle that admits partial-utilisation machines, or
  multi-lane output.*

  ***Running tally for this pair: three prerequisites, found one at a
  time, each only by building.*** *Pipes (done) → footprints (done) →
  ratio tolerance (open). Each was invisible until the one before it was
  cleared. Worth stating plainly in case a fourth is hiding behind the
  third: the corpus tells us what shape to build, but it does not tell us
  what our own engine will refuse, and only an end-to-end attempt does.*

  > **RETRACTED 2026-07-25 (same day).** *The ratio claim above is wrong,
  > and the "run the experiment" lesson it was written to illustrate is
  > exactly what it failed to do. See the next entry.*

- *2026-07-25 — **There was no ratio prerequisite. `casting-* → EC` was
  blocked on a validator false positive, and both pairs now build and
  validate clean.** The entry above computed the straddle from the raw
  per-machine rates — a foundry's 8.0 cable/s against an assembler's
  7.5/s, hence the 16:15 arithmetic. That is not what the caller passes.
  `try_build_row_cell` scales both rates by `utilization_for`, and
  utilization is precisely the fraction that makes a fractional machine
  count integral: 8.0 × 0.9375 = 7.5. The pair lands exactly 1:1 at every
  rate, and `plan_row_straddle` has always accepted it.*

  *The real blocker was `check_belt_connectivity`. A fluid-fed producer
  in a row cell takes its ingredients through a pipe and hands its
  product straight to the neighbouring machine, so no inserter of its
  ever touches a belt — which the check reported as
  `"no inserter connects to a belt"`, one error per foundry, at every
  rate. `fluid_only_recipes` did not cover it: `casting-copper-cable` has
  a fluid input but a SOLID output. Added `fluid_input_only_recipes` and
  a deliberately narrow exemption — the machine must be in a DI cell,
  have an adjacent COUPLER (proof its product has a route), and have no
  solid ingredient (nothing a belt would have had to deliver) — so a cell
  machine that fails to get a real belt is still caught. Pinned by
  `di_row_cell_fluid_fed_producer_validates_clean`, which was canaried:
  it fails with the exemption forced off.*

  | pair | corpus | result | sim @10/s |
  |---|---|---|---|
  | `casting-copper-cable → EC` | 544 | cell at 2.5–20/s, **0 errors 0 warnings** throughout | produced 100.0%, delivered **101.3%** — PASS |
  | `casting-iron → EC` | 339 | cell at 2.5–15/s, **0 errors 0 warnings** throughout | produced 100.0%, delivered **101.3%** — PASS |

  *Both sim runs converged. They are the FIRST fixtures ever to exercise
  the harness's infinity-pipe fluid feed, which RFC-050 declares
  uncalibrated — worth stating, though the risk runs one way: an
  uncalibrated feed could under-supply and show a false FAILURE, it
  cannot inflate EC output past what the cell's own belts and inserters
  carry, and produced matched planned exactly in both runs.*

  *Also checked, because a cell alone in a layout proves little: with a
  smelting row placed alongside (`iron-ore` instead of `iron-plate` as
  the external input) both rates still validate 0/0. The cell's
  `RowSpan.y_start` is taken from `input_belt_ys[0]`, which for a
  fluid-fed producer is the CONSUMER's belt — below the machines and
  below the pipe row, so the span understates the cell's extent. No
  observable effect in a 3-row layout; recorded as a smell, not a
  finding.*

  *Above those rates the cell refuses honestly — the consumer's OTHER
  input (60/s of cable into 8 EC machines) exceeds an express belt's
  45/s. A DI-off control run confirms the refusal is not a regression:
  without the cell neither pair lays out at all today (4–32 errors, the
  foundry left with no adjacent inserter and no pipe), so the cell is
  currently the only path by which a fluid-fed producer works at all.*

  ***The lesson from the retracted entry stands, sharpened: I wrote
  "cheap experiments beat careful arguments" and then, in the very next
  paragraph, blocked a 544-instance pair on an argument I never ran.***
  *The probe that overturned it took one file and one `cargo run`. When
  the engine refuses, print what the caller actually passes before
  reasoning about why.*

- *2026-07-25 — **The fluid-CONSUMER shape is BUILT and currently
  UNREACHABLE, and the sim is the only reason we know.**
  `solid-fuel-from-light-oil → rocket-fuel` (652 instances, the corpus's
  #2 DI pair) builds a row cell that validates **0 errors 0 warnings** at
  0.25–2/s — and in a headless Factorio produces **literally nothing**:
  `rocket-fuel planned 1.00, produced 0.00, −100.0%`, census `no_fuel: 8`
  with all ten upstream chemical plants backed up behind the stall.*

  ***The solver resolves `rocket-fuel` (category `organic-or-assembling`)
  to a `biochamber`, which is burner-fuelled — fuel category
  `nutrients` — and nothing anywhere in the engine delivers burner
  fuel.*** *`validate::power` deliberately exempts biochambers from
  coverage (correctly: they draw no grid power) and no check takes over
  the obligation, so a burner row is invisible to every gate we have.
  `recipe_db::category_machines` offers only `["biochamber"]` for
  `organic-or-assembling`, ignoring the `-or-assembling` half of the name
  — the same shape as `metallurgy-or-assembling` and
  `cryogenics-or-assembling`. So there is no way to steer this pair onto
  an assembler today.*

  *Two fixes follow, and only the narrow one is taken here:
  `cell_machines_are_powerable` now refuses a cell whose either role is
  non-electric. **A cell that cannot run is worse than no cell, because
  it validates clean and lies.** The engine-wide half — delivering burner
  fuel, or honouring `-or-assembling` in machine selection — is NOT
  attempted; it is a recipe-db/solver decision with blast radius well
  beyond this RFC, and it wants its own issue.*

  *This is the third time in this RFC that a mechanism has landed
  correct-but-unreachable (pipe cut → footprints → this). The difference
  is how it was caught: validation, unit tests and an entity census all
  said yes. **Only the sim said no.** Zero validation errors means the
  checks we wrote passed, not that the factory runs — and for a burner
  machine we have not written any check at all.*

  *What the shape itself achieves, kept and unit-tested against the day
  it becomes reachable:*

  *The scoping note said this shape was "genuinely harder" because the
  consumer's south face already carries a solid input and the output, so
  a pipe needs a face the cell does not have. Probing the pair first —
  the discipline the previous entry was written to enforce — showed that
  premise did not hold either:*

  - *the consumer draws **light-oil**, the same fluid the producer is
    already piped, so no second run is needed;*
  - *chemical plant and biochamber are **both 3×3** with geometrically
    identical fluid boxes (`fluid_ports` already models biochamber as
    `CHEM`), so bottom-alignment puts both north faces on the pipe row
    the producer's run already occupies;*
  - *the coupled item is the consumer's **only** solid ingredient, so its
    south face was never contended — it carries the output alone.*

  *Eligibility is gated tightly on those three properties — one fluid,
  the same fluid (different fluids on one run would cross-contaminate),
  equal heights, and a real north port read from `fluid_ports` rather
  than assumed.*

  ***A fourth "fix" that did nothing, caught the same way as the merger
  cursor.*** *The consumer originally registered its own fluid tap points
  alongside the producer's. Forcing that off produced a **byte-identical
  layout**: the pipe run is stamped across the full cell width from the
  producer's side, so it is one connected network and the consumer's
  north ports are adjacent to it either way — and `fluid_port_pipes` only
  tells the lane planner where to tap the bus INTO the cell, which the
  producer's ports already do. Deleted rather than shipped. What IS
  load-bearing is applying `north_input_orientation` to the consumer:
  eligibility admits it on the strength of having a north input port, so
  the stamp has to actually put it there. The fused spec's fluid-rate SUM
  is likewise unobserved today (pipes have no tier; the sim manifest
  reads feed rates from the `SolverResult`) but was KEPT — a spec that
  understates what its row draws is a lie waiting for its first reader,
  which is a different thing from redundant code.*

  ***A second shape fell out of it.*** *With no belt-fed consumer input
  the inner belt row is empty, so the output belt moves up into it. That
  drops a row AND puts the output drop at reach-1, off long-handed's
  2.40/s ceiling — the constraint that forces two output columns in the
  ordinary shape. It also removed a `belt-connectivity` error without an
  exemption: the check looks only at an inserter's 4-neighbours, so a
  long-handed drop across an empty row reads as "touches no belt". Fixing
  the geometry was the honest fix; a second validator carve-out would
  have hidden a real waste.*

  ***The ratio limit is real, but it is alignment, not magnitude.*** *The
  pair refuses above ~2/s: at P30:C23 the flow intervals put THREE
  producers against one consumer, and a consumer has two horizontal
  neighbours. P20:C15 — exactly 4:3 — is fine at any scale. So the row
  cell's reach is bounded by whether the P:C ratio is a small-denominator
  fraction, not by how large the counts are. This is the real form of the
  "ratio tolerance" the retracted entry mis-stated, it does not touch the
  casting pairs (1:1), and it is Phase 3 / multi-band territory.*

  *`engine-unit → electric-engine-unit` (547) is NOT unlocked by this and
  was not attempted. Its producer takes **three** solid inputs against
  the row's one north belt, and its lubricant is a fluid the producer
  does not draw — two independent blockers, either sufficient. Recorded
  so the next reader does not re-derive it.*
