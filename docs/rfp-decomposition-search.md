# RFP: Decomposition-search — strategy-as-candidate selection layer

> **Status**: Phase 0 in progress (this RFP + scaffolding land together).
> Living document — update the Decision log and Task tracker as work progresses.
> Kill criteria are explicit and meant to be acted on.

## Summary

Replace the fixed-strategy menu in [`docs/rfp-modular-production.md`](rfp-modular-production.md)
(`Pooled` / `PartitionedDecomposed`) with a **search-and-score** layer that
sits *above* the existing `LayoutStrategy` enum. Existing strategies become
members of a candidate set; the layout engine generates a layout for each
candidate, scores each by predicted layout cost, and returns the winner.
Future candidates (module-size splits, producer round-up) land as additional
`DecompositionCandidate` impls without refactoring dispatch.

This RFP **does not** propose a new way to lay out machines — it proposes a
decision layer that picks among candidate decompositions. The ugly `(n, m)`
balancer shapes (e.g. `(4, 9)` on PU@3/s ore red copper-plate) are *created*
by the current fixed strategy choice rather than being intrinsic; a richer
candidate set + per-candidate scoring should let the engine sidestep the
worst shapes by picking a different decomposition rather than fighting the
shape with downstream balancer fixes.

Phase 0 (this chunk) lands the abstraction with a single `NativeCandidate`
that wraps today's dispatch — **no behaviour change**. Future phases add
real candidates, gated by the kill criteria below.

## Motivation

Concrete failing case:
[`?item=processing-unit&rate=3&machine=assembling-machine-3&in=coal,water,crude-oil,iron-ore,copper-ore&belt=fast-transport-belt&strategy=partitioned-decomposed`](https://storkme.github.io/spaghettio/?item=processing-unit&rate=3&machine=assembling-machine-3&in=coal%2Cwater%2Ccrude-oil%2Ciron-ore%2Ccopper-ore&belt=fast-transport-belt&strategy=partitioned-decomposed)
— copper-plate produces a `(4, 9)` balancer module that no library template
covers (gcd=1, coprime). After commits `8191cd2` / `18c1493` / `cd62344`
landed pad-lanes / shard fix strategies in `bus/shape_fix.rs`, the case
improved from 21 errors to 7 — but those 7 remain. More importantly, the
fix lives at the *balancer* layer where it can only patch shapes the
decomposition already produced; it cannot ask "could we have chosen a
different decomposition that doesn't produce this shape at all?"

Every existing layout strategy is a fixed shape:

- `Pooled`: one balancer, capped at 8 lanes.
- `PartitionedDecomposed`: per-consumer-row sub-modules, sharded if any
  exceeds 8 lanes, with shape-fixes applied if the resulting `(n, m)` is
  unstampable.

Both produce `(4, 9)` for this case. Neither searches a wider space. The
candidates worth considering — splitting one rate-R module into two rate-R/2
modules; rounding producer count up to a friendlier `n`; using a different
shard split than `⌈original / 8⌉` — exist *outside* the current strategy
vocabulary and have no place to live.

## Design

### New abstraction surface

```rust
// crates/core/src/bus/decomposition_search.rs

pub trait DecompositionCandidate {
    fn name(&self) -> &str;
    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String>;
}

pub struct CandidateScore {
    pub score: f64,
    pub density: f64,
    pub entity_count: usize,
    pub overproduction: f64,
    pub accepted: bool,
}

pub fn score_layout(
    layout: &LayoutResult,
    solver_result: &SolverResult,
) -> CandidateScore;

pub fn select_best_decomposition(
    solver_result: &SolverResult,
    opts: LayoutOptions,
) -> Result<LayoutResult, String>;
```

Today's `build_bus_layout` body (the retry orchestrator at
`crates/core/src/bus/layout.rs:78-153`) extracts to a private
`run_layout_with_retry(solver, opts)`. The new `build_bus_layout` is a thin
wrapper that delegates to `select_best_decomposition`, which iterates
candidates and calls each one's `produce`. The `NativeCandidate` impl calls
`run_layout_with_retry` directly. No recursion through `build_bus_layout`.

### Scoring function

```
score(candidate) = α · density − β · overproduction − γ · entity_count
```

Three terms only:

- **density** — `density::score_density(layout, (1, 1)).density`, in `[0, 1]`.
  Higher is better. Tight square packings score well; sparse linear layouts
  score badly.
- **overproduction** — sum over external outputs of
  `max(0, production_rate − demand_rate)`. Captures the cost of strategies
  like `ProducerCountRoundUp` that build extra producers to reach a friendlier
  `n`. Native layouts have a small positive overproduction from solver
  ceiling-rounding (`ceil(rate / machine_speed)`); other candidates may have
  more.
- **entity_count** — `layout.entities.len()`. A tiebreaker that hedges
  against pathological cases where density looks fine because the layout
  is small but the per-unit-output entity cost is high.

Constants `α`, `β`, `γ` are frozen at the top of `decomposition_search.rs`
with a comment tying them to the calibration corpus once Phase 1 lands.
For Phase 0 (single candidate), only ordering of candidates matters and
ordering with one element is trivial — the constants are placeholders
until Phase 1 introduces a second candidate.

### Hard constraints (reject candidate if violated)

- **Demand met** — every consumer-row's required input rate is satisfied.
  *Phase 0 stub*: always-accepted (we don't have a multi-candidate scenario
  exercising rejection, and Native produces whatever today's pipeline
  produces — which may have validator warnings, but those are the user's
  problem at the same level they always were). Phase 1 implements the
  actual check.
- **All balancer shapes resolvable** — every `(n, m)` family in the
  candidate's layout is either natively stampable or fixable via
  `shape_fix.rs` (pad-lanes / shard). Today this is enforced via the
  `missing-balancer-template` validator warning; rejecting candidates on
  that warning is a Phase 1 lift.
- **Belt tier respected** — `opts.max_belt_tier` is a hard constraint.
  No candidate may unilaterally escalate from yellow to red etc. Belt tier
  reflects user intent about the throughput point they're targeting; auto-
  changing it silently changes the meaning of the request.

### Soft scoring is not a substitute for hard constraints

If `score_layout` returns `accepted: false`, the candidate is dropped from
the ranking entirely — its score is irrelevant. The scoring function is
only for ranking *valid* candidates against each other.

### Per-module shape-fix stays where it is

`apply_shape_fixes` in `crates/core/src/bus/partitioner.rs:530` runs
*inside* candidate evaluation (it's part of the existing pipeline that
`run_layout_with_retry` invokes). The search layer doesn't replace it —
it sits above it. Padded / sharded layouts simply have lower density /
higher entity counts naturally, which the scoring function picks up.

### Candidate ideas worth keeping in scope (not implemented in Phase 0)

The candidate catalogue is the load-bearing part of this design. Worth
naming the ideas now so future-phase work has a target to land against:

- **`ModuleSizeSplit`** — split one rate-R module into N rate-R/N modules
  with independent buses. Each module's `(n, m)` is friendlier; downstream
  consumers tap from the appropriate sub-module rather than a merged
  pool. Cost: extra vertical space (multiple production sites).
- **`ProducerCountRoundUp`** — round `n` to a friendlier value (e.g.
  4 → 6 producers running each at 67% utilization). Reaches a stampable
  shape without padding lanes. Cost: bubbles upstream — extra producers
  consume more inputs, propagating through the recipe tree to
  ore/external inputs. Need to surface the cascade explicitly so the
  user can see what they paid.
- **`DirectInsertion`** — for tight producer/consumer pairs with clean
  ratios (most commonly 2:1 — copper-cable into green-circuits, plate
  smelters into gears, etc.), skip the bus entirely: co-locate the two
  recipe rows so the producer's output inserter feeds directly into the
  consumer's input slot. **Eliminates that intermediate's bus presence
  altogether** — no balancer, no trunk lane, no tap-off. Density wins
  should be substantial when it applies.
  - **Applicability constraints**: ratios must multiply to integer machine
    counts (2:1, 1:1, 3:1 all work; 1:3 awkward; non-integer ratios
    need padding/rounding); the intermediate item cannot be externally
    consumed (because there's no bus presence to tap from); all
    consumers of the intermediate must be co-located on the directly-
    inserted row.
  - **Layout impact**: requires the placer to understand "these two
    rows are inserter-coupled, place adjacent and skip the bus". That's
    new machinery beyond the existing row-stacking; the candidate impl
    is more invasive than `ModuleSizeSplit` or `ProducerCountRoundUp`.
    Likely the largest non-scaffolding chunk in this RFP.
  - **Why it's worth it**: when `DirectInsertion` is viable for an
    intermediate, the bus loses an entire trunk family (and the
    associated balancer + crossings + tap-offs). On deep chains
    (advanced-circuit, processing-unit) where most intermediates
    have a single dominant consumer, the cumulative density gain
    could be larger than every other candidate combined.

### What this RFP does not cover

- New ways to lay out machines or route belts *outside* of what's needed
  for the candidates above. The existing pipeline stays; new candidates
  add what they need (e.g. `DirectInsertion` requires placer changes,
  but the rest of the pipeline is unchanged).
- Generative balancer synthesis (Clos / Beneš networks) — an orthogonal
  way to fix the corpus-coverage problem at the balancer layer. Not in
  scope.
- CP/SAT modeling of the whole layout problem. Discussed as a possible
  v3 if scoring-with-finite-candidates plateaus, but not proposed.
- Web app UI changes for Phase 0. Default behaviour is unchanged so no
  URL state or dropdown is needed.

## Kill criteria

Explicit, observable, falsifiable. Act on these.

**Phase 0 (scaffolding):**

- **K-DS0-1** (inertness): With only `NativeCandidate` in the catalogue,
  every existing e2e test produces a byte-equal layout (golden-hash
  match per `e2e.rs:478` `assert_golden_hash`, table at `e2e.rs:521`).
  Any drift means the wrapping is doing work it shouldn't — debug
  before adding non-Native candidates.
- **K-DS0-2** (LOC budget): Phase 0 scaffolding ≤ ~400 LOC of new
  code (excluding tests). Over budget = abstraction is too granular;
  simplify.

**Phase 1 (first non-Native candidate):**

- **K-DS1-1** (search picks Pool when Pool wins): On cases where Pool
  layouts are already clean and dense (tier 1–3 currently green tests),
  the search must pick `NativeCandidate` (or score-equivalent). If it
  picks a non-Native candidate and density regresses, scoring is wrong
  — fix scoring before extending coverage.
- **K-DS1-2** (motivating-case improvement): Adding the first non-
  Native candidate must reduce PU@3/s ore-red error count below 7
  (current post-shape-fix-Phase-3 baseline). If not, that candidate
  isn't doing useful work — drop it before adding more.
- **K-DS1-3** (runtime budget): End-to-end stress corpus runtime
  regresses ≤ 1.5× with search enabled. Over budget = drop the search
  loop, keep one-shot candidate selection at module-creation time.

**Phase 2+ (more candidates):**

- **K-DS2-1** (scoring-term count): If the scoring function needs > 4
  weighted terms to behave sensibly, the model is wrong — switch to a
  learned cost or actual layout-and-measure rather than tuning more
  knobs.

## Verification plan

Per the [verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).

1. **Golden-hash inertness**: `cargo test --manifest-path crates/core/Cargo.toml`
   — all 9 non-ignored e2e tests pass with zero golden-hash drift. K-DS0-1.
2. **Trace event smoke test**: a new e2e test asserts that
   `DecompositionCandidateScored { name: "native", accepted: true, ... }`
   and `DecompositionChosen { name: "native", ... }` fire on a tier-1
   case. Confirms the search layer is exercising — not just compiling
   but actually running.
3. **Scoreboard format**: `cargo test --test e2e -- scoreboard_strategy_sweep
   --ignored --nocapture` — output still parseable, new candidate-name
   column present.
4. **Browser eyeball**: load `?item=electronic-circuit&rate=2` (any
   currently-clean URL) and confirm the layout is visually unchanged
   from main.
5. **LOC budget**: count new non-test LOC; assert ≤ ~400.
6. **Clippy clean**: `cargo clippy --manifest-path crates/core/Cargo.toml
   --all-targets -- -D warnings`.

## Phasing

- **Phase 0** (this chunk): scaffolding + `NativeCandidate` + scoring +
  trace events + tests. No behaviour change. Lands together with this
  RFP.
- **Phase 1**: first real candidate — likely `ModuleSizeSplit` (split
  one rate-R module into two rate-R/2 modules with independent buses).
  Addresses the motivating case directly: PU@3/s ore-red copper-plate
  becomes `2 × (2, 5)` instead of `(4, 9)`. Activates K-DS1-* kill
  criteria.
- **Phase 2**: `ProducerCountRoundUp` (round `n` to a friendlier value;
  upstream rate cascade is the cost). Activates K-DS2-* once multiple
  non-Native candidates compete.
- **Phase 3**: `DirectInsertion` candidate. Substantial new placer
  machinery (inserter-coupled adjacent rows, bypass bus for that
  intermediate). Highest potential density win but most invasive —
  scoped as its own phase deliberately. Likely needs its own RFP
  expansion at that point.
- **Phase 4**: enumerate-all-fixes-per-module refinement (today's
  `select_shape_fix` returns first viable; v2 returns all viable so
  the outer search can pick the combination with best total density).
- **Belt-tier escalation is explicitly off the table** — see Design.

## Decision log

- *2026-04-29 — RFP drafted alongside Phase 0 scaffolding. Scope
  established in conversation: search-and-score above `LayoutStrategy`
  enum, existing strategies as candidate-set members, three-term
  scoring (density − overproduction − entity_count). Kill criteria
  framed up-front per project's "exploration rework" pattern. Phase 0
  is intentionally a no-op pass-through to validate the abstraction
  shape before adding real candidates.*

- *2026-04-29 — Belt-tier escalation removed from candidate catalogue.
  Belt tier is user-specified (URL `belt=` param) and reflects
  throughput-point intent; auto-escalating yellow → red silently
  changes the meaning of the user's request. `opts.max_belt_tier` is
  treated as a hard constraint, not a soft preference. Same pattern
  as the modular-production RFP's "no silent strategy downgrade".*

- *2026-04-29 — `DirectInsertion` added to the candidate catalogue as
  a Phase 3 deliverable. Premise: producer/consumer pairs with clean
  ratios (commonly 2:1 — copper-cable into green-circuits, plate
  smelters into gears) skip the bus entirely via co-located inserter-
  coupled rows. Eliminates the intermediate's bus presence altogether,
  meaning whole trunk families and their balancers disappear. Most
  invasive candidate (requires new placer machinery for adjacency-
  coupled rows) but potentially the largest density win. Scoped to
  its own phase with its own RFP expansion at land-time.*

- *2026-04-30 — Phase 1 landed (`ModuleSizeSplit` candidate, k=2). Key
  deviations from the original plan:*
  - *Search is **sequential, not parallel**. Native runs first; if its
    layout has zero `missing-balancer-template` warnings, search
    exits and Native wins. Only when Native is rejected does
    `ModuleSizeSplit` run. Reason: parallel evaluation busts the
    600s test timeout on big partitioned cases (advanced-circuit /
    processing-unit at 5s+). Trade-off: gives up the "score every
    candidate, pick best by density" framing the RFP committed to.
    Acceptable: in practice no candidate would beat Native on
    density when Native is already clean, so the early-exit doesn't
    miss anything valuable. Phase 1a/1b split (proposed earlier) is
    therefore collapsed into one chunk.*
  - *Acceptance check is `count_missing_balancer_template_warnings(layout)
    == 0`. Narrower than the Plan agent proposed (which also gated
    on `belt-throughput` / `belt-flow-path` errors). Reason: those
    wider categories fire on harmless edge cases and would break
    inertness on currently-clean tier 1-3 layouts. The narrow check
    targets exactly the (n, m) coprime trap this RFP exists to
    address.*
  - *`ModuleSizeSplit::produce` is wrapped in `std::panic::catch_unwind`
    in the search dispatcher. Reason: PU@3/s ore-red — the
    motivating case — trips a `todo!()` in `lane_planner.rs:571`
    (consumer-clamped fan-in for `electronic-circuit` when the
    split creates a configuration the multi-stage balancer hasn't
    been wired for). Without the catch, ModuleSizeSplit would
    propagate the panic to the caller (web app or test suite),
    a strict regression vs today's "7 errors but produces a
    layout." With the catch, ModuleSizeSplit gracefully degrades
    to Native's layout. The `lane_planner.rs:571` panic remains
    a blocker for ModuleSizeSplit actually delivering the
    promised improvement on PU@3/s ore-red — **fixing it is the
    next concrete task before Phase 1 can be considered done in
    spirit (motivating case still at baseline 7 errors).** Also
    RAII-fied `trace::with_muted` so a panic doesn't leak
    `MUTED=true` to subsequent calls on the thread.*
  - *Pre-layout shape-stampability guard inside
    `ModuleSizeSplit::produce`: skip the candidate when no module's
    `(estimated_n, lane_count)` is unstampable. Avoids wasting
    layout work on cases where the split can't help (preserves
    the runtime budget on partition_strategy_scoreboard).*
  - *Verification: 483 tests green (was 478 at Phase 0). Added 4 unit
    tests for `apply_size_split` and one e2e test
    (`decomposition_search_picks_native_on_clean_partitioned_case`)
    pinning K-DS1-1 — search picks Native on a clean partitioned
    case (tier-3 plastic-bar). K-DS1-2 (PU@3/s ore-red error count
    drops below 7) is **not yet satisfied** because of the
    `lane_planner.rs:571` blocker.*

- *2026-04-30 — `apply_cap_driven_split` helper added (partitioner.rs)
  and composed inside `ModuleSizeSplit::produce` after the size split.
  Splits any module whose post-size-split rate exceeds full belt
  capacity into K = ceil(rate/full_belt_cap) sub-modules. Trace event
  `ModuleCapSplitApplied`. Lane planner's consumer-clamped fan-in path
  (`lane_planner.rs:570`) won't be asked to fit more flow than a
  single belt can carry — when the helper fires.*

- *2026-04-30 — Investigation: K-DS1-2 still not satisfied even with
  cap-driven split. Probed PU@3/s ore-red and found the (4, 9)
  copper-plate trap is on a **K=1 item** (only one consumer recipe —
  copper-cable). K=1 items don't enter `plan.modules` unless their
  lane_count exceeds `SHARD_THRESHOLD_LANES = 10`. Copper-plate's
  lane_count is below that threshold, so `apply_shape_fixes` never
  sees it. `ModuleSizeSplit` operates on `plan.modules` too, so it
  also misses copper-plate.*

  *The actual fix for PU@3/s ore-red requires either (a) extending
  `apply_shape_fixes` to include K=1 items (probably by adding a pass
  that enrolls them in the plan with `module_id=0` when their lane-
  planner shape would be unstampable), or (b) a lane-planner-level
  shape fix that pads `(4, 9) → (4, 12)` directly when constructing
  the family. Both are tractable but are their own pieces of work.
  Tracking K-DS1-2 as **blocked on shape-fix-for-K1** for the next
  chunk.*
