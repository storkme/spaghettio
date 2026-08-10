# RFC-067: The cell-interface database

## Summary

An in-repo, searchable store of production-subtree implementations, keyed by
**demand motif** — `(recipe, machine, count)`, plus fused two-recipe motifs —
with every implementation carrying a **port contract** (where flows enter and
leave) and a **derived constraint vector** (what the implementation needs to
be legal). Three consumers, in order of shipping: an **interface-first
preview** (place boxes and trunks before interiors exist), **template
candidates** competing in the existing candidate machinery under the standing
never-worse and sim-anchor gates, and a **standing regression corpus** (every
entry is sim-anchorable, so "did this engine change regress the top motifs"
becomes a check instead of a hope). Evidence that this pays lives in the
Phase-0 scoreboard ([`celldb-phase0-scoreboard.md`](celldb-phase0-scoreboard.md)):
demand is power-law concentrated (top 5 motifs = 87.7% of machine mass),
interiors dominate layout area at low/mid rates (fabric median 18.3%), and
the community corpus corroborates both the demand distribution and the
supply of donor implementations.

## Motivation

Every layout run re-derives implementations of the same handful of motifs.
Measured on the survey corpus (Phase-0, all numbers regenerable from
checked-in probes):

- Top 5 unit motifs carry **87.7%** of machine mass; top 12 carry 97.5%.
- The same top 5 are ~89% of attributed interior area (17.0–25.1
  tiles/machine).
- Fabric (trunks, taps, ghost routes, balancer stamps) is a **median 18.3%**
  of interior+fabric area — cached-interior improvements have headroom at
  low/mid rates. Above ~20/s on low belt tiers fabric approaches parity
  (max 55.1%), so that regime's lever is fabric motifs, not interiors.
- Community per-machine density is already engine-ballpark (19.2 vs our 17.0
  on smelting, definitions differ and are both stated in the scoreboard) —
  so the win is **composition, aspect control and tail cases**, not raw
  density. This RFC promises accordingly.

The fused-pair finding triangulates independently three ways: the census's
edge motifs (cable→circuit in 15/29 solves), the community's building habits
(green-circuit donors exist almost only as fused blocks), and the engine's
own DI cells. The DB's unit of caching must therefore include two-recipe
motifs from day one.

## Design

### Identity, and what is deliberately not identity

- **Key** = demand motif: `(recipe, machine entity, count)` for unit motifs;
  `(recipe_a, recipe_b, count_a, count_b)` for fused pairs. Counts are
  integers; **rates are always derived by the current solver at lookup
  time** — an entry never stores a rate, so the store cannot go stale when
  the rate model changes (productivity work is in flight in five worktrees
  as this is written).
- **Constraints are not key axes.** Each entry's requirements — belt tiers
  used, inserter kinds, fluid-box mirroring, UG reach classes — are
  **derived by the loader from the entry's own entities**, never declared.
  Tech level is the entity vocabulary. A declared field can lie; a derived
  one cannot outlive its state. Lookup is a dominance filter: entries whose
  derived requirements ⊆ the caller's allowed set, ranked by metrics.

### Port contract (v1)

An entry is a blueprint fragment (entities with coordinates relative to the
fragment origin) plus a declared port list:

```
port := { tile: (dx, dy), kind: belt-in | belt-out | pipe-in | pipe-out,
          item: <name> }
```

A `(kind, item)` pair may declare **multiple ports** — split rows have one
entry per half, and composition must feed all of them (amended during
seeding; see the decision log).

v1 constrains geometry to the bus row conventions the engine already emits
(inputs arrive on one long edge, outputs leave on the other; fluid ports at
their direction/mirror-determined tiles) so that engine-generated seeds are
expressible without translation. Community-derived entries that cannot meet
the contract are *not* admitted in v1 — port inference from wild blueprints
is explicitly out of scope (recorded gap).

### Store

`crates/core/data/celldb.json`, embedded via `include_str!` exactly like
`recipes.json` — versioned, diffable, no infrastructure. Loader in
`crates/core/src/celldb.rs`: parse, validate fragments against the
40-check validator at load time in tests (an entry that validates with
errors is a build failure, not a runtime surprise), derive constraint
vectors, expose `query(motif, allowed) -> ranked entries`.

Entry metrics: bbox, interior tiles, entity count, provenance
(`engine@<sha>` | `community:<source>` | `hand`), and a sim-anchor status
(`unanchored` | `anchored@<sha>` with measured rate). **Metrics are recorded
by the seeding tool, not typed by hand.**

### Consumers

1. **Preview (Phase 2):** `SolverResult -> Vec<PlacedBox>` — one box per
   machine group sized from the store's entry (or, for uncached motifs, from
   the Phase-0 tiles/machine table), with ports on the contract edges and
   trunk polylines between them. Pure function in core, exposed through
   wasm; the web app renders boxes as a toggle. No correctness stakes: the
   preview is labelled an estimate, and its calibration against realized
   layouts is measured, not assumed (see verification).
2. **Template candidates (Phase 3):** for motifs with store entries, a
   candidate that stamps the stored fragment instead of running row
   placement. Competes under the existing candidate scoring;
   **never-worse by the validator is the admission floor, the meter refutes
   cheaply, and selection stays firewalled until sim-anchored** — the
   standing #519/#520 discipline, inherited verbatim. Ships inert
   (candidate mode off by default), the way DI (RFC-053) did —
   HorizontalStack is the *scoring* precedent, not the rollout one: it
   shipped default-on.
3. **Regression corpus:** a check that re-derives metrics for every entry
   and diffs against the recorded ones; a drift is a loud failure naming
   the entry.

### Rejected alternatives

- **Keying on rates** — continuous, and stales under any rate-model change.
- **Declared constraint fields** — the records-outlive-state failure class;
  derive instead.
- **External storage/service** — violates the falsifiability norm; the
  balancer library already proves the in-repo pattern at smaller scale.
- **Arbitrary-depth subtree entries** — combinatorial; the evidence says
  two-recipe fusion is where community practice stops, and the census's
  deep chains decompose into the same dozen shallow motifs.

## Kill criteria

- **K67-1 (contract expressiveness):** if expressing the top-5 engine-
  generated seeds under port-contract v1 requires **more than one escape
  hatch in total across all five entries** (an escape hatch = any
  per-entry special case, ambiguity override, or PORT-WARN the seed tool
  emits that has to be resolved by hand), the contract is wrong — stop and
  redesign before any consumer ships.
- **K67-2 (preview honesty):** if the preview's total-area estimate is off
  by more than 30% median against realized layouts on the survey corpus,
  the preview ships disabled until recalibrated; if recalibration cannot
  reach 30%, kill the preview consumer.
- **K67-3 (candidate value):** if, after seeding the top-5 motifs, no
  survey-corpus fixture's candidate scoring prefers a template under the
  never-worse floor, the DB adds no value at current engine density — park
  Phase 3 and keep preview/regression consumers only. (Phase-0's density
  reality check makes this a live possibility, not a formality.)
- **K67-4 (standing constraints):** belt tier is a user constraint, never a
  strategy knob — a template that would auto-escalate tier is inadmissible
  by construction. Any selection change that the meter reads below plan is
  firewalled until sim-anchored. These are inherited rules restated as kill
  criteria so this RFC cannot be read as relaxing them.

## Verification plan

- Loader: round-trip and derived-constraint unit tests; every entry
  validator-clean at test time (0 errors, warnings recorded).
- Seeds: generated by tool from engine output at a named SHA; metrics
  recorded by the tool; spot-decoded via the snapshot debugger.
- Preview: a calibration table (estimate vs realized, per survey fixture)
  printed by a probe and quoted in the shipping PR — K67-2 adjudicated on
  its output.
- Candidates: candidate-scoring parity harness on the survey corpus; meter
  `check_one` on every fixture where a template wins; sim anchor required
  before default-on (the flip is a separate, later PR by design).
- Full e2e suite green at every phase; WASM build green (checks, not nits).

## Phasing

- **P1 (this RFC + store):** schema, loader, seed tool, top-5 engine seeds.
- **P2 (preview):** core function + wasm + web toggle; calibration probe.
- **P3 (candidates):** template candidate source, inert by default; gates
  as above. The default-on flip is *not* part of this RFC's scope.

## Decision log

- *2026-08-10 — opened, on the Phase-0 scoreboard's four GO verdicts. Schema
  decisions banked from the probe work: counts-not-rates, derived-not-declared
  constraints, fused pairs as first-class motifs.*
- *2026-08-10 — community port inference declared out of scope for v1 after
  the mining pass showed donor value concentrates in density baselines and
  demand corroboration; admitting wild fragments without inferred ports would
  bypass the contract that makes entries composable.*
- *2026-08-10 — the "win is composition, not density" reframing (community
  density already engine-ballpark) is recorded here so Phase-3 expectations
  stay honest: K67-3 exists because a null result is genuinely possible.*
- *2026-08-10 — port contract v1 AMENDED during seeding: a (kind, item)
  pair maps to MULTIPLE ports. Split rows have one entry point per half;
  the seed tool's first run surfaced this as 5 PORT-WARNs, and min-picking
  one port would starve the other half under composition. With the
  amendment, **K67-1 adjudicated CLEAN: zero escape hatches** across the
  top-5 engine seeds.*
- *2026-08-10 — **K67-2 adjudicated FAIL**: median |total-area error|
  32.6% vs the 30% bar (n=29, `celldb_preview_calibration`). Three
  principled levers tried — fabric allowance, uniform Phase-0 non-interior
  factor (banded variant measured WORSE than uniform), lane-count physics
  (`ceil(rate/belt capacity)` trunk lanes). Residual error is structural:
  bbox dead space and balancer explosions are not predictable from solver
  output at this granularity. Preview ships DISABLED per the criterion —
  module and instrument land, no wasm/web consumer. Remaining levers for a
  reopening attempt: per-machine-class fallback factors; balancer cost
  modeled from lane count; predicting the placer's row-split widths (at
  which point the preview has become the placer — the likely kill line).*
- *2026-08-10 — **K67-3 adjudicated NULL** on the harness fixtures
  (`tests/celldb_template.rs` prints the data): iron-plate template passes
  never-worse but loses the ranking (32-furnace entry stamped for a
  16-furnace need — ar_score −1.31); copper-cable inadmissible (20-machine
  entry for a 4-machine need, +3.3% entities). No template wins. Root
  cause structural and predicted by the density reality check:
  engine-derived seeds cannot beat the engine that produced them, and
  single-count entries oversize for smaller demands. **Phase 3 PARKED per
  K67-3**; the reopening path is count-matched seed ladders and
  community/hand donors with inferred ports — both explicitly future
  work.*
