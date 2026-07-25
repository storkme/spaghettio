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
2. **Face contention.** If the consumer's remaining face cannot carry
   its non-DI flows (iron in + EC out) within its tile budget at
   **≤ L2** `inserter_capacity` (i.e. the cell is only feasible at max
   research), the topology is under-scoped — stop.

   **2b. Tier-cap degradation.** `max_inserter_tier` is a hard user cap,
   orthogonal to research level. If, at the engine defaults
   (`Stack`, L2), the canonical coupling needs **more than one inserter
   per producer→consumer edge**, the per-edge slot budget derived above
   is wrong and the straddle geometry must be re-derived — stop. And if
   a `Fast`-capped user cannot get a feasible cell at the **default**
   research level, DI is too fragile to ship default-on — it stays an
   opt-in strategy with an honest refusal, not a silent degradation.
3. **Honest throughput.** If a DI cell validates clean but the sim
   harness measures **< 98% of plan** on the canonical fixture, the
   model is wrong and the checks are lying — stop everything. (This is
   the #383 lesson: validator-clean concealed a real starve for weeks.)
4. **Density premise. ✅ EVALUATED 2026-07-25 — PASSES.** Measured on
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
  - **1c — placer wiring** (remaining, and the invasive step). Pick-up
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
      itself and hanging it off a `RowKind`; (2) intercept the
      producer/consumer spec pair in `place_rows`; (3) lane-planner skip
      for the coupled item —
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
