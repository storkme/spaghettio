# RFC-067: The cell-interface database

## Summary

An in-repo, searchable store of production-subtree implementations, keyed by
**demand motif** — `(recipe, machine, count)`, plus fused two-recipe motifs —
with every implementation carrying a **port contract** (where flows enter and
leave) and a **derived constraint vector** (what the implementation needs to
be legal). Three consumers were designed; the kill criteria adjudicated two of
them out (see the decision log — this Summary records outcomes, not the
original promise): the **interface-first preview** module exists but its
consumer is **KILLED per K67-2** (run count and sequence: decision log); **template candidates** exist inert in the candidate harness and are
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
   be without a decision-log amendment — the calibration sequence never reached the 30% bar (full accounting: decision log).
2. **Template candidates (Phase 3) — PARKED per K67-3, harness landed
   inert:** for motifs with store entries, a candidate that stamps the
   stored fragment instead of running row placement. Competes under the existing candidate scoring;
   **never-worse by the validator is the admission floor, the meter refutes
   cheaply, and selection stays firewalled until sim-anchored** — the
   standing #519/#520 discipline, inherited verbatim. Ships inert (candidate mode off by default). Precedent claimed here is
   only that candidates can land without steering selection — the DI and
   HorizontalStack rollouts each moved to default-on on their own
   evidence, which is exactly the gated path this RFC requires.
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
  in #621. **K67-2 adjudicated FAIL — consumer KILLED** (the decision log is the
  single authority for the run count and sequence; every other site,
  this one included, defers to it after two stale-copy rounds). A reader building the
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
  final run 39.0% vs the 30% bar (n=29 fixtures per run, `celldb_preview_calibration`; EIGHTH adjudication RUN, and the only one on a fully-corrected instrument — the per-run value is that run's median fixture error, never an across-run aggregate. Count accounting, explicit because the kill-clause math depends on it: eight runs = one initial + three estimator levers (fabric allowance; uniform Phase-0 factor after a banded variant measured worse; lane-count physics) + four instrument corrections from review (belt-tier default artifact; lane math onto the planner's half-throughput constant; fluid lanes, removing a PASS-direction bias; per-machine output rates + real bbox scaling, whose earlier absence had been CANCELING against other errors). The sequence ran 31.9/33.1/32.6/31.5/32.7/32.3/33.1/39.0 — the corrected instrument reads WORSE than the earlier 31.5–33.1% band, which was partly compensating bugs, so the criterion's kill clause — "if recalibration cannot reach 30%, kill the preview consumer" — FIRED, and this log records the kill: **the preview CONSUMER is KILLED per K67-2's letter** — no discretion clause exists and none is invented here. The module and calibration instrument remain as the measured baseline; any future consumer requires an explicit decision-log amendment reopening the criterion, which the owner can always make — by amendment, not by this log hedging). Three
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
- *2026-08-12 — **donor-probe reopening attempt OPENED, gate pre-registered
  before any measurement.** The hotspot scoreboard
  ([`hotspot-scoreboard-2026-08.md`](hotspot-scoreboard-2026-08.md), #623)
  priced the donor targets; copper-plate is the largest single-group-
  realizable prize (5,987 pooled overhead tiles). K67-3's NULL covers
  engine-derived seeds only; whether a denser community cell exists is the
  untested half, and this entry registers its adjudication terms in
  advance. **Scope:** hand-translated community copper-plate smelter
  cell(s) under port contract v1 — the recorded port-inference gap stands,
  translation is by hand, provenance `community:<source>`,
  `sim_anchor: unanchored`, metrics machine-verified by the loader
  invariant test (`embedded_db_parses_and_entries_hold_invariants`).
  **Procedure, fixed now:** each donor keeps its natural machine count N; a
  new demand-matched fixture in `crates/core/tests/celldb_template.rs`
  derives the rate demanding exactly N (same `rate_for_count`), incumbent =
  `FullSelectionCandidate`, candidate = `TemplateCandidate`, same
  `Policy::fold()` and 0.5/0.5 composite weights as K67-3. If N collides
  with an engine entry's count, the donor must win `query_unit`'s
  (count, interior_tiles) sort to be stamped at all — recorded if it
  happens, not worked around. **Donor WINS** = passes the never-worse
  floor AND composite > +0.02 (`COMPOSITE_TIE_EPSILON`) vs the incumbent;
  that reopens Phase 3 toward multi-group stamping (the ac/ec prizes),
  with sim-anchor still required before any selection influence (K67-4,
  inherited verbatim). **KILL** = after a documented harvest (target ≥3
  translatable donors; fewer only if the harvest cannot produce 3, with
  the shortfall recorded), no donor wins: the community-donor reopening
  path is adjudicated dead for single-group smelters and Phase 3 stays
  parked. **Distinct verdicts, counted separately from losses:** a donor
  whose belt tiers exceed the fixture's allowed set is inadmissible by
  construction (K67-4), and if no harvested design is expressible under
  port contract v1 at all, the binding constraint is the contract, not
  donor value — a contract-narrowness finding, which neither reopens
  Phase 3 nor counts toward the kill's denominator.*
- *2026-08-12 — **donor probe ADJUDICATED: no win under the registered
  gate, kill does not fire — the floor instrument was caught
  false-positive on 2 of 3 donors, and the gate re-arms after #624.**
  Harvest: the manifest-verified Phase-0 corpus held exactly 7
  engine-legal electric-furnace arrays; 3 distinct designs shortlisted
  and ALL translated cleanly under port contract v1 (`celldb_donor`:
  ports hand-declared, carries derived from inserter drop-side
  semantics with refuse-on-ambiguity, zero refusals — so no
  contract-narrowness verdict). One translation lesson banked: the ON0
  donor's furnaces carried speed-module payloads (its "45/s" label was
  the tell) which inflated its first sim to 44.9/s against a 32.5/s
  plan; donors donate GEOMETRY only, the translator now strips module
  payloads, and the store was regenerated (harness numbers unchanged).
  Results at matched demand, per the registration: composites +0.366
  (F3R#8, count 48) / +0.289 (Lxr#6, 50) / +0.447 (ON0#2, 52) — all
  far above the +0.02 win bar — but all three FAIL the never-worse
  floor, so **no donor wins as registered**. Floor forensics
  (instrument-before-finding): Lxr#6's failure is genuine physics —
  31.24/s demanded through a 30/s fast-belt feed; its own label sells
  it as a chained 30/s cell — **one real loss**. F3R#8 and ON0#2 fail
  on `input-rate-delivery` alone, and that is a MEASURED false
  positive: the walker seeds their entry splitters correctly
  (seed-stats demand-consistent) then strands the rate — propagation
  through unsegmented splitters never runs, and ON0's half-fed inline
  balancer tiles read as 28 phantom sources (Σdemand 48.7 vs 32.5).
  Engine layouts never reach this path (balancer:-segmented splitters
  are skipped), so the hole was real but unreachable until community
  geometry arrived — receipts in #624 and validator-trust.md hole 7.
  **Sim anchors, the only clearing instrument: both blocked donors PASS
  at matched demand** — 30.1/s vs 29.99 planned (48/48 working) and,
  module-less, 32.1/s vs 32.49 (52/52 working, stable windows;
  east/north feed directions carry the harness's uncalibrated-direction
  flag). Verdict accounting, both readings recorded (the K67-1
  discipline; a review round correctly objected that this entry's first
  draft claimed the registration had reserved an instrument-defect
  verdict class — it had not, it reserved exactly belt-tier
  inadmissibility and contract narrowness, and that claim is
  RETRACTED): **under the registration as written, the kill FIRES** —
  zero donors won, and the letter offers the two blocked donors no
  shelter. The response is an AMENDMENT, logged here as its own
  decision exactly as K67-1's contract amendment was: a verdict whose
  floor input is a measured instrument false positive (receipts: #624
  seed-stats, plus both sim clears above) adjudicates the
  *instrument*, not donor value, and is recorded as
  instrument-invalidated rather than as a loss. Under the amended
  registration the kill's denominator holds 1 loss of 3 and the kill
  does not fire; the letter-firing above is the recorded event and is
  not erased by the amendment. **Phase 3 stays parked either way. The
  post-#624 re-adjudication is a FRESH gate under the amended
  registration** — not a continuation of this one — and needs no new
  harvest: the store rows, fixtures, and the tracked `donor_sim_export`
  are all in place. Ripple, recorded: the donor rows change
  `query_unit`'s first pick for copper-plate (753 < 817 interior
  tiles), which also moves the killed preview module's reference
  geometry — `celldb_preview_calibration` re-runs on a donor-bearing
  store are NOT comparable to the K67-2 sequence quoted above. What the probe changed regardless of verdict: the
  count-48 fixture now stamps the F3R donor (753 < 817 interior tiles
  wins the sort — the pre-registered collision rule, recorded in the
  test), and the composite margins say community geometry beats the
  engine's strip-shaped cells on aspect by an order of magnitude more
  than the tie epsilon, which is exactly the shape of value RFC-067
  predicted donors would carry if any did.*
- *2026-08-12 — **FRESH GATE ADJUDICATED after the #624 walker fix: TWO
  DONORS WIN — the Phase-3 reopening condition is MET.** The fix
  (external seeds no longer erased on splitter tiles; unfed splitter
  second-tiles no longer counted as phantom sources; pickups on
  splitter tiles read the pair's pooled stream) removed the
  `input-rate-delivery` false-positive walls, and the demand-matched
  fixtures now record: **count 48 (F3R#8): floor PASS, composite
  +0.366, winner celldb-template; count 52 (ON0#2): floor PASS,
  composite +0.447, winner celldb-template** — 18–22× the +0.02 win
  bar, both sim-anchored at matched demand by the probe (30.1/s vs
  29.99 planned; 32.1/s vs 32.49, module-less). **Win dependency
  accounting (adversarial-review ablation, recorded because the two
  wins do not rest on the same legs)**: count 48 flips on the
  seed-erasure fix alone; count 52 additionally requires the
  pooled-pair pickup read, which is a documented upper bound (a
  splitter pair's own pickups are not debited) — its floor pass
  therefore leans on the credit, and its sim anchor (32.1/s measured,
  all 52 working) is the grounding that makes the win claim honest
  rather than model-circular. Also banked from that review: post-fix
  the ON0 seeding becomes EXACTLY demand-consistent (Σ attributed
  32.494 == solver 32.494, `consistent=true`, demand-weighted seeding
  replacing the even-split fallback) — the strongest single seeding
  receipt in the campaign. The count-50 donor's genuine physics loss
  stands unchanged (lane-throughput/dead-end, not IRD, not instrument
  artifacts). Engine-side controls, verified in the fix PR:
  iron-plate and copper-cable fixtures print byte-identical verdicts;
  all 8 host-drifting stress goldens have byte-identical layout
  hashes vs an origin/main run on the same host with a pinned private
  SAT zone cache (the drift is the documented cache artifact, not the
  fix). The fix's engine-corpus footprint, complete per the review's
  census: `tier4_advanced_circuit_partitioned` 3→1
  `input-rate-delivery` (the two cleared copper-plate warnings were
  the defect pair firing on the engine's own tapoff splitter —
  receipts in the fixture comment);
  `stress_advanced_circuit_partitioned_4s_from_plates`'s
  PartitionedDecomposed leg 8→3 warnings (within its ceiling, Pooled
  golden legs byte-identical); and the balancer-audit un-blinding —
  four shapes' audit drive changed ((6,3), (6,4), (7,4), (8,3); see
  the KNOWN_IMBALANCED doc for the census), (6,3)/(6,4) provisionally
  known-imbalanced, owner ratification pending. **What reopening means and does not mean**:
  per the amended gate, Phase 3 may resume toward multi-group stamping
  (the ac/ec prizes) under a plan of its own; stamping stays inert and
  selection stays untouched until the standing K67-4 discipline is
  separately satisfied — this entry records the gate outcome, it does
  not flip any default.*
- *2026-08-13 — **the Phase-3 resumption plan is
  [RFC-068](rfc-068-multi-group-stamping.md)** (in-place `RowSpan`
  substitution; fused stamping for the ec prize; self-stamp fidelity as
  its Phase-0 gate). Phase-3 status now tracks that RFC's decision log;
  this log takes only the close-out echo.*
