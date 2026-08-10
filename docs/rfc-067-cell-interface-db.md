# RFC-067: The cell-interface database

## Summary

An in-repo, searchable store of production-subtree implementations, keyed by
**demand motif** — `(recipe, machine, count)`, plus fused two-recipe motifs —
with every implementation carrying a **port contract** (where flows enter and
leave) and a **derived constraint vector** (what the implementation needs to
be legal). Three consumers were designed; the kill criteria adjudicated two of
them out (see the decision log — this Summary records outcomes, not the
original promise): the **interface-first preview** module exists but its
consumer is **KILLED per K67-2** (six calibration runs, 31–33% vs a 30%
bar); **template candidates** exist inert in the candidate harness and are
**PARKED per K67-3** (NULL on all three realizable motifs — engine-derived
seeds tie the engine); the **standing regression corpus** is the surviving
consumer, delivered as the store's every-entry drift test. Evidence that this pays lives in the
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

1. **Preview (Phase 2) — CONSUMER KILLED per K67-2, design retained for
   the record:** `SolverResult -> Vec<PlacedBox>` — one box per machine
   group sized from the store's entry, ports on the contract edges. The
   core function and calibration instrument exist (#621); the wasm/web
   consumer described by the original design was never built and MUST NOT
   be without a decision-log amendment — six calibration runs landed
   31–33% against the 30% bar (decision log).
2. **Template candidates (Phase 3) — PARKED per K67-3, harness landed
   inert:** for motifs with store entries, a candidate that stamps the
   stored fragment instead of running row placement. Competes under the existing candidate scoring;
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
  printed by `celldb_preview_calibration` (lands in #621 with the module) —
  K67-2 adjudicated on its output; until #621 merges the numbers quoted in
  this decision log cite that PR's checked-in instrument, not loose files.
- Candidates: candidate-scoring parity harness on the survey corpus; meter
  `check_one` on every fixture where a template wins; sim anchor required
  before default-on (the flip is a separate, later PR by design).
- Full e2e suite green at every phase; WASM build green (checks, not nits).

## Phasing

- **P1 (this RFC + store):** schema, loader, seed tool, top-5 engine
  seeds — implemented in #620. K67-1 adjudicated CLEAN (decision log).
- **P2 (preview):** core function + calibration instrument — implemented
  in #621. **K67-2 adjudicated FAIL six times; the wasm/web consumer was
  never built and must not be** until a recalibration passes (decision
  log has the sequence and the live kill clause). A reader building the
  consumer from this bullet would be building what the criterion killed.
- **P3 (candidates):** template candidate source in the candidate_runner
  harness, inert by construction — implemented in #621. **K67-3
  adjudicated NULL, Phase 3 PARKED** (decision log); the default-on flip
  was never in scope and is now doubly gated.

**Certification status of the adjudications below:** the measurements come
from instruments checked into #620/#621, which are sequenced immediately
after this RFC (620's review required the design authority on main first).
Until those PRs merge, the numbers are verifiable from the PR branches, not
from main; when they merge, every figure re-derives from main. Docs-lead
ordering is a deliberate trade recorded here, not an oversight.

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
  top-5 engine seeds. Accounting, both readings recorded (a review round
  correctly objected that counting only under the final contract is a
  retroactive re-score): **under K67-1 as originally written, the first
  run TRIPS it** — five PORT-WARNs against a >1 threshold. The response
  was a CONTRACT AMENDMENT (multi-port per (kind, item)), logged above as
  its own decision, and a re-adjudication under the amended contract:
  zero hatches. The amendment is the recorded event; the CLEAN verdict
  applies to the amended contract only, and the original trip is not
  erased by it.*
- *2026-08-10 — **K67-2 adjudicated FAIL**: median |total-area error|
  32.3% vs the 30% bar (n=29, `celldb_preview_calibration`; SIXTH adjudication RUN. Count accounting, explicit because the kill-clause math depends on it: six runs = one initial + three estimator levers (fabric allowance; uniform Phase-0 factor after a banded variant measured worse; lane-count physics) + two instrument corrections from review (belt-tier default artifact; lane math onto the planner's half-throughput constant). The sequence ran 31.9/33.1/32.6/31.5/32.7/32.3 — every run in the 31-33% band, so the criterion's kill clause — "if recalibration cannot reach 30%, kill the preview consumer" — FIRED, and this log records the kill: **the preview CONSUMER is KILLED per K67-2's letter** — no discretion clause exists and none is invented here. The module and calibration instrument remain as the measured baseline; any future consumer requires an explicit decision-log amendment reopening the criterion, which the owner can always make — by amendment, not by this log hedging). Three
  principled levers tried — fabric allowance, uniform Phase-0 non-interior
  factor (banded variant measured WORSE than uniform), lane-count physics
  (`ceil(rate/belt capacity)` trunk lanes). Residual error is structural:
  bbox dead space and balancer explosions are not predictable from solver
  output at this granularity. Preview ships DISABLED per the criterion —
  module and instrument land, no wasm/web consumer. Remaining levers for a
  reopening attempt: per-machine-class fallback factors; balancer cost
  modeled from lane count; predicting the placer's row-split widths (at
  which point the preview has become the placer — the likely kill line).*
- *2026-08-10 — **K67-3 adjudicated NULL**, twice. First pass stamped
  unscaled entries against smaller demands and the losses were foregone
  (review caught it — three rounds flagged the shape). RE-ADJUDICATED
  DEMAND-MATCHED (the harness derives the rate demanding exactly the
  seeded count): copper-cable's template scores a POSITIVE composite
  (+0.18 aspect, fewer entities) but fails a never-worse validation
  category; iron-plate passes never-worse and ties within noise. No
  template wins — because engine-derived seeds TIE the engine at matched
  demand, which is this RFC's own density thesis. **Phase 3 PARKED per
  K67-3**. Discharge scope, corrected after review: a round claimed
  ac/ec-rooted survey fixtures made those motifs testable — false, those
  are multi-group solves (ec co-solves with cable, ac with both) and v1
  refuses them by design — but the objection exposed a real gap:
  copper-plate IS single-group-realizable and was untested. Added and
  adjudicated: passes never-worse, ties within noise (composite −0.059),
  incumbent wins. **The NULL now covers 3 of 3 realizable motifs** —
  complete over everything v1 stamping can express; ec/ac require
  multi-group stamping, which is exactly the parked work. The reopening path is community/hand donors with inferred ports,
  plus count ladders so matched demand is the common case.*
