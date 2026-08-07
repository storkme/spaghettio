# Validator trust table

**Status (2026-08-07):** initial version. Written after the PU@1/s incident
below. This page is the **registry** for two per-check properties the codebase
previously kept implicit and scattered: what happens when a check fires
(consequence), and how much we believe it (trust). **Owner rule: a PR that
changes a check's severity, adds/removes a category, or changes selection
participation must update this table in the same PR.** Graduation
preconditions ("promote X once Y is proven") live *here*, not in doc comments
— the #519 exit condition lived in a code comment plus a status bullet whose
rationale had gone stale, and nothing re-asked whether the precondition had
been met until a sim failure forced the question (see hole 2).

Sibling doc: [`validator-reporting.md`](validator-reporting.md) covers *how* a
check should report (one positioned issue per instance, etc.) and the ten
incidents of checks going quiet. This page covers *what the report is worth*.

## Why this page exists

On 2026-08-07 a PU@1/s parity fixture measured **68.2% of plan** in the sim.
The layout carried three `input-rate-delivery` warnings naming the exact
starving machines (0.0/s electronic-circuit delivered to three consumers) —
the validator had localised the defect before the sim ever ran. Nothing acted
on the warnings: candidate selection was deliberately excluding the category
from ranking, export has no validation gate, the sim manifest carries no
validator state, and the parity report quoted 68.2% with no mention of them.
Every stage behaved as designed; the designs disagreed about what a Warning
means.

The structural lesson: **severity conflates two independent properties.**

- **Consequence** — what the pipeline does when the check fires: refuse the
  candidate, re-rank it, or just print.
- **Trust** — how confident we are that a fire is real, and (separately)
  that silence means absence. Coverage gaps are a trust property too: an
  Error-severity check with a blind spot is *worse* than a Warning, because
  its silence is taken as clearance.

A physically-impossible-layout check and a taste heuristic both emitting
"Warning" is how a factory whose math visibly doesn't add up gets simmed
without comment.

## Consequence channels — what actually has teeth today

| # | Stage | What validation does there |
|---|---|---|
| C1 | e2e fixtures | fixtures that call `validate()` and unwrap fail on any Error; scoreboards count both severities |
| C2 | candidate search (`bus/decomposition_search.rs`) | DI arm **refuses** any Error-carrying layout; `IssueCounts` (errors, selection-scoped warnings, `LayoutResult.warnings`) are component-wise never-worse **floors**; `selection_warning_count` ranks candidates by Warning count **minus exclusions** (`input-rate-delivery`, `belt-detour`); `score_layout` hard-gates `missing-balancer-template` |
| C3 | transactional transforms (`bus/compaction.rs`) | a cut/candidate that has (or adds) Errors is rejected; the pre-transform layout survives. RFC-065 adds a `connectivity::error_certain_regression` reject-fast prefilter ahead of the full validate |
| C4 | web UI | issues render as markers; export button works regardless |
| C5 | blueprint export | **no gate** — export never consults validation |
| C6 | sim / meter harness | **no gate, no visibility** — the manifest (`crates/sim-harness/src/manifest.rs::Manifest`) carries geometry, rates, boundaries, and the declared axes, but no issue state at all |

Second issue channel: `LayoutResult.warnings` — strings stamped by the layout
pipeline itself (e.g. "No N→M balancer template"), never seen by `validate()`.
Floor-protected in `IssueCounts`; reading only the validator has already
produced one false "0 errors 0 warnings" claim (#462).

Note what is *absent*: nothing between "the search picked a winner" and "the
sim measured it" ever re-asks the validator. C5 and C6 are the holes the
2026-08-07 incident went through.

## The trust ladder

- **Physical/structural fact** — deterministic geometry or graph property
  (overlap, UG pairing, pipe adjacency, dead ends). A fire is definitionally
  real; false positives only via modelling bugs. Trust the fires; audit the
  *coverage* (the ten incidents were mostly silence, not noise).
- **Ledger conservation** — cross-checks of solver/layout bookkeeping against
  physical entities (boundary records, stranded byproducts, shared-row
  claims). Each of these was born from a specific validator-clean-but-
  game-dead incident; they encode "a ledger entry is a claim, the entity at
  its tile is the fact".
- **Rate model** — derived flow rates (lane walker, delivery, inserter
  hands). Both false positives (#519 pre-recalibration) and false negatives
  (holes 1, 4) observed. **Needs sim anchoring before it steers selection**;
  after anchoring, needs this table updated with the receipt.
- **Heuristic** — corpus-calibrated taste (`belt-detour`). Report-only by
  design until a sim case is made.

## The table

Severity `E/W` = both emitted, condition-dependent. "Sel" = counts in
`selection_warning_count` (Warnings) / triggers refusal+floors (Errors).

### Physical / structural facts (trust the fire)

| Category | Sev | Sel | Notes |
|---|---|---|---|
| `entity-overlap` | E | yes | |
| `belt-connectivity` | E | yes | |
| `belt-dead-end` | E | yes | |
| `belt-loop` | E | yes | |
| `belt-topology` | E/W | yes | spaghetti-style check |
| `belt-item-isolation` | E | yes | contamination class |
| `belt-junction` | E/W | yes | head-on = Error, else Warning |
| `output-belt` | E | yes | |
| `tap-priority` | E | yes | |
| `underground-belt` | E/W | yes | pair/sideload lane rules per `factorio-mechanics.md`. NB `classify_errors` (decomposition_search.rs) matches `"underground-belt-sideload"`, which nothing emits — see hole 5 |
| `pipe-isolation` | E | yes | |
| `fluid-connectivity` | E | yes | |
| `fluid-network` | E | yes | |
| `inserter` | E | yes | chain completeness |
| `inserter-direction` | E | yes | |
| `power` | E/W | yes | coverage + pole connectivity (incidents 2–4 in validator-reporting.md were here) |
| `unresolved-junction` | E | yes | router self-report; definitionally real |
| `belt-throughput` | W | yes | **misleading name**: overlapping route entities, NOT rate-vs-capacity. The rate-vs-capacity question is owned by `lane-throughput` (walked flow); the *stamped*-rate version is unanswerable — hole 1 |
| `orphan-belt-segment` | W | yes | |
| `sushi-boundary` | E | yes | |
| `module-slots`, `module-eligibility` | W | yes | data-table facts |
| `missing-balancer-template` | W | yes + hard gate | the one Warning with teeth (`score_layout` rejects outright) |

### Ledger conservation (incident-born, trusted)

| Category | Sev | Sel | Born from |
|---|---|---|---|
| `boundary-record-integrity` | E (missing entity) / W (carries mismatch) | yes | the 0.00/s stale-boundary-record incident (`validate/mod.rs` doc comment; validator-reporting.md #1) |
| `stranded-byproduct` | E | yes | net-flow RFC's "validator-clean but game-dead" class (USP's stranded AOP light-oil) |
| `shared-row-outflow-overclaim` | E (plan-level) / W (built-only) | yes | RFC-062; severity split calibrated on a real EC+AC ceil-slack false positive |
| `shared-row-outflow-underclaim` | E | yes | RFC-062 Phase 0 observed a target export silently dropped with zero errors |
| `record-effective-rows` | E | yes | RFC-065: machine footprints vs `effective_rows` bands, harm-calibrated |
| `record-power-wires` | E | yes | RFC-065: stored wire endpoints must be in-bounds pole entities |
| `connectivity-anomaly` | E | **not dispatched** | emitted by `connectivity::scan_graph_anomalies`, deliberately not wired into `validate()` in RFC-065 Phase 0 — a category that exists but reaches no consequence channel; consumed by tests only. Its one candidate consumer (an anomaly-scan reject prefilter on the fold path) was built, measured, and **killed on a pre-registered criterion** in Phase 2b (0.83% reject volume vs a ≥30% bar — see `search_snake_fold_with_stats`'s doc comment). The C3 prefilter is `error_certain_regression`, which reads the derived graph directly, not these issues |

### Rate models (calibration status is the load-bearing column)

| Category | Sev | Sel | Calibration status |
|---|---|---|---|
| `lane-throughput` | E | yes | walked lane rates vs stacking-aware caps; seeding deliberately uncapped so over-commit stays visible. **Vindicated 2026-08-07**: an earlier draft called its silence on the 376-tile stacking winner a blind spot — that layout measures 96.0% of plan, so the silence was correct and the 376 figure was the artifact (hole 1) |
| `input-rate-delivery` | W | **excluded** | **anchored 2026-08-07** (receipts below): positive direction sim-measured sound (warning-free re-ranked layout 102.0% of plan); negative direction confirmed qualitatively in-client (owner observed the flagged EC belt starving, 4 producers vs 8 consumers) — its 68.2% *rate figure* stays provisional (class-5c min-checkpoint run, unreconciled with #591's 90–98% note). Exclusion predates the anchor; its hole-1 blocker was retired by measurement 2026-08-07 (see hole 2) |
| `belt-flow-path` | E spaghetti / **W bus** | yes | graph-flow walk; Warning under `LayoutStyle::Bus`, which every production call site passes (the enum's *derived default* is Spaghetti — don't confuse the two) — hole 4 |
| `belt-flow-reachability` | E spaghetti / **W bus** | yes | the #520 check, rewritten per-tile after incident ten; still cannot block a Bus layout — hole 4 |
| `inserter-throughput` | W | yes | hand-capacity model; never sim-anchored |
| `inserter-item-throughput` | W | yes | never sim-anchored |
| `row-output-lane-budget` | W | yes | never sim-anchored |
| `row-input-belt-margin` | W | yes | deliberately conservative (both-lane ceiling); never sim-anchored |
| `sushi-saturation` | E | yes | Error-severity despite never being sim-anchored (like `lane-throughput` above — the two rate models trusted with refusal power on modelling grounds alone); reporting fixed in incident #5 |

### Heuristics

| Category | Sev | Sel | Status |
|---|---|---|---|
| `belt-detour` | W | **excluded** | corpus-survey-calibrated thresholds; report-only by explicit design until sim-anchored (its own doc comment) |

## Known holes, ranked by measured cost

1. **The layout carries no per-tile flow figure, so "is this belt
   over capacity?" is not answerable from a stamped rate.**
   `PlacedEntity::rate` comes from `BusLane::rate`, whose own doc says
   *"Total throughput (items/s or fluid/s) for **belt/pipe tier
   selection**"* — a lane-FAMILY total. When a family exceeds one belt the
   planner realises it as several parallel belts and stamps **every tile
   with the family total**, so `e.rate > belt_capacity` is routine and
   means nothing about throughput.

   **This entry previously said the opposite twice, and both versions cost
   real work** — recorded here because the misunderstanding is the
   expensive part, not the code:

   - v1 (this page's first draft) claimed *"no check compares planned
     rates to physical capacity"*. False: `check_lane_throughput` does
     exactly that, at `Severity::Error`, and does it **correctly** by
     walking rates through the belt graph instead of trusting a stamp.
   - Acting on v1, a `belt-capacity` check was written comparing `e.rate`
     to `belt_throughput_stacked`. It reported 12 fixtures broken and was
     **falsified by measurement**: a layout it flagged at "3.00× over"
     delivered **94% of plan** in the sim, another at "1.11×" delivered
     **101%** (both `kit_errors` empty, converged). Parked, unmerged.
   - The `stacking_ec_60s` e2e fixtures contain a **test-side audit
     closure making the identical comparison**, and it is wrong for the
     identical reason. Its verdict on the `input-rate-delivery` selection
     fix — "376 tiles at 3.00× over physical capacity" — was settled by
     measuring that exact layout: **96.0% of plan** (57.60/s against 60,
     converged, kit-clean). A 3×-throttled factory cannot do that.

   A false-positive test that ruled out the family-total hypothesis is
   what made this stick: stamped values *differ across tiles* (28.33 and
   45.00 on one item), which reads like genuine per-tile flow. It isn't —
   parallel **paths** differ from each other while every belt within a
   path shares one number.

   **Next action: none of the obvious ones.** Do NOT promote the fixture
   audit into `validate/`; it measures the wrong quantity. A sound
   capacity check needs per-tile flow the layout does not currently carry,
   which is a data-model change, not a check. `check_lane_throughput`
   already covers the answerable version. **And the two `stacking_ec_60s`
   fixtures need rewriting** — their S=2 assertion gates merges on a
   quantity that does not exist.
2. **`input-rate-delivery` is excluded from selection despite being
   anchored.** Positive direction sim-measured (the re-ranked PU layout
   delivers **102.0%** of plan); negative direction confirmed in-client.
   The branch that lifts the exemption exists and improves PU, EC@2/AM2
   and tier2.

   **Its blocker was hole 1 and is now retired.** The objection was that
   lifting the exemption re-ranks `stacking_ec_60s` onto a "physically
   impossible" winner; that layout measures 96.0% of plan. What the
   measurement did surface is a real but much smaller question: on that
   fixture the re-ranked layout delivers **57.60/s vs the control's
   58.80/s** (96.0% vs 98.0%) while being **39% smaller** (163×105 vs
   167×167, 4629 vs 5192 entities, 35 vs 47 warnings). That is a
   footprint-versus-rate trade, which this project reserves for the owner
   — and single runs cannot separate a 2-point gap from run-to-run
   variance, so repeats come before the trade is even put.
3. **The sim/meter side has no validator visibility.** `Manifest` carries no
   issue state, so parity sweeps can quote a condemned layout as a parity
   number — which is precisely how 68.2% was first reported with no mention
   of the three warnings that explained it. This is a **recorded RFC-050
   Phase 0 deferral**, not an unnoticed gap: the RFC's Design section
   promises `validator_errors`/`validator_warnings` in the manifest, and
   `crates/sim-harness/src/manifest.rs`'s module doc documents that Phase 0
   shipped without them (resolved as optional/absent-tolerant). **Next
   action:** emit the fields the RFC already promised — per-category counts,
   not just totals (`validator-reporting.md` rule: totals can't tell 2 from
   218) — and make the sweep/report print them next to every rate, flagging
   any "parity" number measured on a warned layout as measuring the layout,
   not the pipeline.
4. **`belt-flow-path` / `belt-flow-reachability` are Warnings under
   `LayoutStyle::Bus`** — the style every production call site passes
   (`LayoutStyle::default()` is Spaghetti, but nothing in production relies
   on it) — so the check rewritten after the #520 0.50-ratio incident
   cannot block the style of layout that incident shipped.
5. **`classify_errors` string drift**: two of its match strings are dead —
   `"underground-belt-sideload"` (the UG checks emit `"underground-belt"`,
   so sideload errors silently fall through to the starvation `_` arm
   instead of contamination) and `"pipe-to-ground"` (only ever an *entity
   name* in `validate/`, so it contributes nothing to the structural arm —
   which stays live via `entity-overlap`; real pipe errors emit as
   `pipe-isolation`/`fluid-network`/`fluid-connectivity` and land in
   contamination correctly). Consequence is bounded (only the scoped Pooled
   merge-tap quality comparison), but two dead strings in one match
   expression is a live instance of category strings having no registry.
   This table is now that registry.
6. **Severity has no "uncalibrated" tier**, so calibration firewalls are
   implemented as silent exclusions inside `selection_warning_count`. The
   #519 firewall's written exit condition ("lift once sim-anchored") lived
   in a code comment and a status.md bullet; no mechanism ever re-asked
   whether the precondition had been met, and the bullet's rationale
   ("selections are bit-identical") had been falsified by review without
   the bullet changing. Rule going forward: an exclusion or severity choice
   made for *trust* reasons gets a row here with its graduation
   precondition and the receipt that would satisfy it.

## Receipts (sim anchors and falsifications)

- **2026-08-07 — `input-rate-delivery` anchor.** PU@1/s AM3 from-ore,
  research productivity +10% (PU, plastic). **Positive direction (sound):**
  the re-ranked layout with the warnings selected away measured **1.020/s
  delivered vs 1.0 plan (102.0%)**, converged over 4 checkpoints, kit
  clean, census 140 working / 4 full-output / 1 ingredient-short.
  **Negative direction (qualitative, rate provisional):** the flagged
  layout (three warnings, 0.0/s EC to three named machines) measured
  0.682/s — but that run converged at the *minimum* checkpoint count
  (forensics class 5c: provisional until re-run longer) and is
  unreconciled with #591's 90–98% note, so the 68.2% figure is not yet a
  clean measurement. What anchors this direction instead is the owner's
  in-client observation: the EC belt into the PU rows visibly starved,
  4 producers of 2.5/s against 8 consumers of 2.5/s — the structural
  shortfall the warnings named, independent of the rate figure. A long
  re-run + #591 reconciliation would upgrade this to a fully measured
  two-sided anchor.
- **2026-08-07 — the "over-capacity" readings are not throughput.**
  Three measurements, all `kit_errors` empty and converged, retiring the
  stamped-rate-vs-capacity comparison entirely:

  | layout | audit says | measured |
  |---|---|---|
  | AC@7/s horizontal stack, yellow | 104 tiles, **3.00×** over | **94%** of plan |
  | EC@45/s express legendary | 72 tiles, **1.11×** over | **101%** (PASS) |
  | `stacking_ec_60s` S=2, re-ranked | 376 tiles, **3.00×** over | **96.0%** (57.60/60) |

  Its control (main as shipped, audit-clean) measures **98.0%**
  (58.80/60) at **39% more footprint**. An earlier draft of this page
  cited the 376-tile figure as the anchor FOR hole 1; it is now the
  anchor against it. See hole 1 for why the comparison is a category
  error.
- **2026-07-31 — #520 (incident ten).** `small-electric-pole@5` DI layout,
  clean on every channel, measured **2.52/s vs 5.00 plan**; native 5.08/s.
  Falsified "clean means working"; produced the per-tile reachability
  rewrite and parked `di_claim_order` Search.
- **stress-EC throughput ceiling** (rfc064-phase2-followups §1).
  `merge_output_rows` collapsing every row's output to one belt; found only
  by in-client observation — **no check fired**. Standing evidence that the
  rate-model family's coverage is incomplete.
- **RFC-053 KC3.** DI cell shape sim-measured at **112% of plan** — the
  positive case: the cell exemptions in the inserter/output checks are
  justified by measurement, not assumption.
