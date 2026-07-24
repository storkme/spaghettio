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
inserters and machines bidding for face tiles under
reach/throughput/adjacency constraints. **Solids only; pipes are
explicitly out of scope** (that RFC's own reason: fluid ports are
prototype-fixed per orientation and misplacement is hard-infeasibility,
not cost).

## Motivation

**DI is the dominant human strategy the engine cannot produce.** #429's
corpus sweep: 17,009 DI inserters across 793 of 6,038 blueprint members
(13.1%), with cable→EC the single most common pair in the game.

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

**The corpus says humans don't build that** (mined 2026-07-24,
`crates/core/examples/di_mine.rs`, 172 corpus files, 98 containing DI):

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

`gap=1` (reach-1, machines one tile apart) dominates; the `gap=2`
long-handed shape #432 implements is a ~5% minority. Lateral offsets of
0, ±1, ±2 are all common — the empirical signature of consumers
straddling two producers (see Design).

Other DI pairs the same sweep found, i.e. the catalogue this generalizes
to: `electric-furnace → electric-furnace` (1,585 — smelting columns),
`solid-fuel → rocket-fuel` (652), `engine-unit → electric-engine-unit`
(547), `casting-copper-cable → electronic-circuit` (544, Space Age
foundry), `iron-stick → rail` (351).

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

## Design

### The DI cell

A producer row, a one-tile inserter band, a consumer row:

```
   x: 0     3     6     9    12    15
      ┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐
 y0-2 │cab1││cab2││cab3││cab4││cab5││cab6│   6 × copper-cable (3×3)
      └────┘└────┘└────┘└────┘└────┘└────┘
 y3      S  S    S  S       S  S    S  S     8 × stack inserter, reach 1
          ┌────┐┌────┐      ┌────┐┌────┐
 y4-6     │ EC1││ EC2│      │ EC3││ EC4│     4 × electronic-circuit
          └────┘└────┘      └────┘└────┘
         x=1    x=5        x=10   x=14
```

**Source-limited, not inserter-limited.** One stack inserter could move
12/s, but the cable machine behind it only *makes* 5/s. An EC machine
needing 7.5/s must therefore draw from **two** producers — which forces
the consumer row off the producer row's pitch so every consumer straddles
a producer boundary. Flow for the drawing above: `cab1→EC1 5.0`,
`cab2→EC1 2.5` + `cab2→EC2 2.5`, `cab3→EC2 5.0`, mirrored on the right.
Every consumer receives exactly 7.5/s; every producer ships exactly
5.0/s. **The corpus's non-zero lateral offsets are this same straddle.**

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

- **Pipes / fluids** — the reason full face allocation is a bigger
  effort. A fluid-touching coupling is refused, not approximated.
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
   **≤ L2** research (i.e. the cell is only feasible at max research),
   the topology is under-scoped — stop.
3. **Honest throughput.** If a DI cell validates clean but the sim
   harness measures **< 98% of plan** on the canonical fixture, the
   model is wrong and the checks are lying — stop everything. (This is
   the #383 lesson: validator-clean concealed a real starve for weeks.)
4. **Density premise.** If a DI cell is not **strictly smaller** than
   the equivalent bus rows on the canonical fixture, the entire premise
   is falsified — stop. (Expected: ~7 tiles of coupling height against
   ~13 for #432's bridge.)
5. **Solver escalation bound.** If Tier 1 leaves > 20% of the corpus's
   top-10 DI pairs infeasible, escalate to Tier 2 — but if CP-SAT
   cannot place a single pair within **500 ms**, stop: too slow for the
   layout loop, and the constructive path is the answer we ship.
6. **Scope integrity.** If any canonical case requires pipes/fluids to
   work, stop — that is the full face-allocation effort, not this RFC.

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

- **Phase 0 — mine + feasibility.** Canonicalize corpus patterns into
  the generated library (version-gated); evaluate kill criterion 1 on
  the canonical case and the corpus top-10 pairs. **No placer code.**
  A written feasibility table is the deliverable.
- **Phase 1 — the DI cell.** `bus::di_cell` for the simplest shape: one
  producer recipe, one consumer recipe, consumer's only solid input is
  the DI'd item. Straddle offsets + min-cost-flow assignment.
- **Phase 2 — face allocation.** The consumer's remaining flows on the
  opposite face, mixed reach (reach-2 stepping over a near belt).
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
