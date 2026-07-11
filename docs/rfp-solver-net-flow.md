# RFP: Net-flow solver (LP replacement for the recursive tree walk)

## Summary

Replace the solver's recursive tree walk (`crates/core/src/solver.rs`,
`resolve()`) with a small linear program over the recipe graph: one
variable per recipe (crafts/sec), one flow-conservation constraint per
item, minimize weighted external inputs. This fixes four classes of
silently-wrong output the tree walk structurally cannot fix —
uncredited byproducts, double-counted multi-product machine fleets,
physically-nonsensical cycle handling, and cost-blind recipe selection.
The formulation is the one factoriolab and the Kirk McDonald calculator
use, giving us external cross-validation targets. The LP is tiny (339
recipe columns × ~306 item rows, sparse) and solves in microseconds;
this is not a combinatorial-search project.

The rollout is deliberately conservative: the default swap (Phase 1)
runs the LP in **compatibility mode** — restricted to the recipe set
the tree walk would have chosen — so recipe *selection* behavior only
changes in a later phase, after the layout engine has grown the pieces
that cost-based selection needs (fluid surplus routing, multi-machine
multi-fluid-output rows). Adversarial review of an earlier draft
(2026-07-10, three independent reviewers; see decision log) found that
an immediate unrestricted swap would flip currently-green oil fixtures
onto known-broken layout shapes; the phasing below is designed around
that finding.

## Motivation

The tree walk computes each ingredient's demand inside its own branch
(`solver.rs:258`) and recurses immediately. No global item ledger
exists, so:

1. **Byproducts are never credited, and the flagship gauntlet result
   is physically invalid because of it.** A fresh
   `science_gauntlet` run at HEAD `ce732d9` (2026-07-10; reproduce:
   `cargo test --release --test science_gauntlet -- --ignored
   --nocapture`) shows `utility-science-pack@1/s` passing **0 errors /
   0 warnings** (note: CLAUDE.md's tier-7 row still records the
   pre-`ce732d9` probe numbers and needs refreshing). Yet the USP
   chain pulls `advanced-oil-processing` for lubricant's heavy oil,
   and AOP's light-oil and petroleum-gas outputs get **no pipes at
   all**: the solver records them in `output_flows`
   (`solver.rs:233-242`) but nothing consumes that data, and the lane
   planner skips any produced item with zero consumers
   (`lane_planner.rs:233`). In real Factorio a refinery with a blocked
   output port stalls when its internal buffer fills. The validator
   has no check for this — the layout is validator-clean and
   game-dead, under either the fresh or the stale error counts.
   Independently reproducible today: solve `rocket-fuel@1/s` — it
   builds **two independent refinery chains** (basic-oil ×22.2 *and*
   advanced-oil ×1.1) and discards AOP's own 11/s petroleum-gas
   byproduct instead of crediting it against basic-oil's output.

2. **Multi-product machine fleets can be double-counted.** When the
   same recipe is reached from two different `resolve()` calls (once
   per co-product), `solver.rs:244-246` does `existing.count +=
   count`, where each `count` was derived from a *different* product's
   ratio — summing two sizings of a fleet that already produces both
   outputs simultaneously. No currently-green e2e test happens to
   trigger it (rocket-fuel dodges it because its two oil products get
   different canonical recipes), but it is live code. An LP cannot
   have this bug: `x[r]` is one variable feeding every output
   constraint.

3. **Cycles are punted to nonsense.** The `resolving` guard
   (`solver.rs:182-185`) redirects re-entrant items to external
   inputs. Forcing the kovarex path for `uranium-235@1/s` yields
   external inputs of **0.976 U-235/s** — "supply almost as much of
   the thing you're making, from nowhere." Kovarex and
   coal-liquefaction are masked today only because JSON insertion
   order picks their non-cyclic alternatives
   (`recipe_db.rs:139-166`, first-match); `pentapod-egg` and
   `fish-breeding` hit the guard by default and silently return
   nonsense today.

4. **Recipe selection is JSON-order luck, and users can't steer it.**
   `solve_with_exclusions` exists but its only non-test caller is
   `e2e.rs`; production users have no lever.

5. **Adjacent, fixed in passing: silent wrong-machine assignment.**
   `category_machines()` (`recipe_db.rs:291-302`) has no entry for
   `centrifuging` or `rocket-building`, and the fall-through treats an
   unmapped category as assembler-compatible — `uranium-235@1/s`
   solves onto assembling-machine-3 with zero signal. There is no
   centrifuge in the machine data table at all.

All of the above are reproducible today at HEAD (`ce732d9`).

## Design

### Formulation

Variables, all ≥ 0:

- `x[r]` — crafts/sec of recipe `r` (one column per eligible recipe;
  columns with empty products — the `parameter-N` / `recipe-unknown`
  placeholders — are filtered explicitly, not left for the solver to
  zero)
- `s[i]` — external input rate. Eligible for items in
  `available_inputs` **or items with no producing recipe *after* the
  exclusion set is applied** — eligibility must be recomputed per
  solve, not read off the unfiltered DB, or excluding an item's only
  producer turns "graceful external fallback" (today's
  `solver.rs:174` behavior) into `Infeasible`.
- `o[i]` — surplus rate (byproduct produced beyond internal demand)

One constraint per item `i`:

```
Σ_r net(i,r)·x[r] + s[i] − o[i] = target(i)
```

with `net(i,r) = Σ products (amount × probability) − Σ ingredients
(amount)` and `target(i)` the requested rate for the target item, 0
otherwise.

**Implementation requirement (would otherwise panic at runtime):**
`net(i,r)` must be computed as a *single netted scalar per (item,
recipe) pair before insertion*. Six recipes have the same item on both
sides (kovarex, coal-liquefaction, both bacteria cultivations,
pentapod-egg, fish-breeding), and `microlp::LinearExpr` **panics if
the same variable is added twice to one constraint** (verified in
microlp 0.4.0 source, `lib.rs:109-118`). A naive
"loop ingredients, then loop products" column builder dies on exactly
the kovarex golden test.

Objective (single weighted sum):

```
minimize  Σ w[i]·s[i]  +  ε_o·Σ o[i]  +  ε_m·Σ x[r]·energy[r]/speed[r]
```

**Frozen cost table, revision 1** (locked so kill criterion 3 can't be
tuned into passing; revision 1 is the single permitted revision, made
during Phase 0 *before* any cross-validation ran — see decision log):

- `w[i]` = 1.0 default, `water` = 0.01 (when not available), items in
  `available_inputs` = **1e-4 — cheap but strictly positive**. Free
  (w=0) available inputs turned out to be gameable: Phase 0 observed
  three variants of "surplus laundering" (sink chains outside the
  demand closure, sink chains through demanded items, and overdriving
  legitimate recipes past demand) that are all profitable only when
  raw inputs cost nothing. A positive `w_available` makes raw
  efficiency dominate surplus-shrinking, killing all three at once,
  and still never builds producers for an available item (production
  costs more than 1e-4-priced imports).
- `ε_o` = 1e-8, `ε_m` = 1e-6 (ordering: `w_default ≫ w_available ≫
  ε_m ≫ ε_o` — surplus preference is now the *last* tiebreak, below
  machine count, because any stronger surplus penalty re-opens the
  laundering incentive)

Review evidence: a scratch implementation against the real recipe data
found the rocket-fuel solution stable across `ε` values spanning
1e-2…1e-7 including swapped orderings, with qualitative solution
changes only at the degenerate boundary `ε_m = 0` — so the separation
is empirically robust, but Phase 0 golden tests include an explicit ε
sensitivity sweep (10×/100×) as a regression guard.

Machine counts are computed **after** the LP: `count[r] = x[r] ·
recipe.energy / crafting_speed(machine)`. Note the basis change from
the tree walk: `x[r]` is *crafts/sec*, not items/sec of one designated
product — which is what makes multi-product recipes unambiguous
(Motivation #2). The constraint *matrix* is machine-independent;
the objective's `ε_m` coefficients require palette-resolved crafting
speeds up front, so `MachinePalette` resolution runs before matrix
build (exactly as it runs before the walk today), and the
`IncompatibleMachine` pre-flight carries over verbatim.

### What falls out, and what doesn't

- **Byproducts**: AOP's column carries per-craft coefficients
  `{heavy-oil: +25, light-oil: +45, petroleum-gas: +55, crude-oil:
  −100, water: −50}` (verified against `recipes.json`); the LP credits
  gas against gas demand, cracking columns absorb surplus when cheaper
  than exporting, and true surplus lands in `o[i]`.
- **Kovarex**: net U-235 is +1/craft (41−40), net U-238 is −3/craft.
  `uranium-235@1/s` → `x[kovarex] = 1.0`, U-238 external input 3/s.
  With the Phase-1 centrifuge data (speed 1.0, energy 60) that's 60
  centrifuges; without it the number is meaningless (today's
  fall-through would put it on AM3). Correct steady state; the in-game
  40-unit startup seed is a transient, not a rate, and is out of
  scope.
- **Alternatives**: all producing recipes are columns; cost decides.
  Pins/exclusions = column removal or bounds on `x[r]`, subsuming
  `solve_with_exclusions` and finally giving it production wiring.
- **Probabilistic outputs**: already expected-value in the
  coefficients (`solver.rs:208,238`), unchanged.
- **Honest limits, found in review:**
  - *Recipe blending is real and sometimes optimal.* A scratch solve
    of `rocket-fuel@1/s` with free crude+water returns AOP alone (no
    basic-oil blend) but a **provably-optimal 3-way split across all
    three solid-fuel recipes** (heavy 1.62 / light 4.82 / gas 3.56
    crafts/s) achieving zero surplus. Simplex's vertex property bounds
    the number of active recipes (≤ #items with nonzero flow) but
    does *not* prevent genuinely-optimal co-product blends. This is a
    correct, zero-waste factory — three chem-plant rows — and layout
    already groups machines per recipe, so it lays out fine; it is
    *more* rows than a human might build. If aesthetics demand fewer
    active recipes, an optional consolidation post-pass (force
    low-usage recipes to zero, re-solve, accept if objective degrades
    < X%) is a Phase 4 knob, not a formulation change.
  - *Not all vanilla cycles are self-loops.* An SCC census over the
    canonical recipe graph found, besides the single-recipe self-loops
    (kovarex, coal-liquefaction, pentapod-egg, fish-breeding), a
    2-cycle (`carbon` ↔ `coal` via `coal-synthesis`) and a 4-cycle
    (`sulfuric-acid` → `sulfur` → `water`/`steam` chain via
    `steam-condensation`/`acid-neutralisation`). Multi-recipe cycles
    span multiple machine rows and are a genuine bus-level routing
    problem — **explicitly out of scope for this RFP**. The solver
    refuses them with a typed error (below); a future RFP can take
    them on if a real use case demands it. No current e2e or gauntlet
    case hits them (their items are always pre-supplied as
    `available_inputs` in the corpus).

### Cycle policy (all phases of this RFP)

After solving, compute SCCs of the *active* recipe graph (`x[r] > 0`).

- Trivial SCCs: fine.
- Single-recipe self-loops (recipe consumes its own output): rejected
  with typed `SolverError::UnsupportedSelfLoop` in Phase 1; supported
  in Phase 2 via a row-local loop template (output belt/pipe fed back
  to the row's own input — a bounded template feature, not a bus
  change).
- Multi-recipe cycles: rejected with typed
  `SolverError::UnsupportedCycle`, permanently within this RFP's
  scope.

This converts today's *silent nonsense* (pentapod-egg returns garbage
external inputs at HEAD) into a *loud, typed refusal* — a behavior
change, and an improvement, in its own right. Rationale for refusing
rather than shipping: adversarial review traced what a kovarex-shaped
`MachineSpec` does to the current layout pipeline — `row_kind`
(`placer.rs:371-411`) counts it as `DualInput` and expects an external
uranium-235 bus lane that can never exist, and `lane_planner`
(`lane_planner.rs:180-253`) manufactures a lane whose producer and
consumer are the same row with `source_y` *below* its tap-off — a
silent violation of the lanes-run-south invariant that surfaces only
as unrelated-looking validator errors. Refusing loudly until Phase 2
makes those templates self-loop-aware is strictly better than that.

### Code shape

- New module `crates/core/src/netflow.rs` (~400-600 LOC + tests):
  matrix build from `RecipeDb`, LP solve, `SolverResult` assembly.
- Solver backend: `microlp` 0.4.0 (Apache-2.0, pure Rust, maintained
  fork of `minilp`). Verified in review: compiles for
  `wasm32-unknown-unknown` (scratch build), supports equality
  constraints and per-variable bounds, and pivot selection is
  deterministic (plain vector iteration, Harris-style ratio test — no
  hashing, no randomness). Two caveats from source review: (a) no
  documented anti-cycling guarantee — Phase 0 adds an *iteration*
  ceiling treated as a hard error (never a wall-clock limit and never
  a best-so-far result, both of which would break determinism);
  (b) it pulls `sprs`/`ndarray`/`matrixmultiply` transitively —
  heavier than the varisat precedent suggests, so Phase 0 records the
  WASM bundle-size delta. Fallback if either caveat bites: hand-rolled
  dense simplex (~300 LOC; the matrix is tiny) with Bland's rule.
- Public API: existing `solve_*` signatures unchanged. Phase 0 adds a
  parallel `solve_netflow_with_palette_and_exclusions(...)` used only
  by the parity harness. Phase 1 swaps the internals of the public
  entry points; no caller changes.
- **`SolverResult` grows one additive field**: `surplus_outputs:
  Vec<ItemFlow>`. `external_outputs` keeps its exact current meaning
  (the requested target(s)) — chosen specifically because
  `decomposition_search::compute_overproduction` (live in
  `build_bus_layout` via `layout.rs`) computes overproduction as
  `total_produced − external_outputs rate` and would silently
  miscount if surplus entries were folded into `external_outputs`.
  Downstream consumers of `external_outputs` (step-7 output merger,
  overproduction scoring, web result panel) are untouched until Phase
  2 deliberately teaches them about `surplus_outputs`.
- The `is_fluid: false` hardcode in today's `external_outputs`
  construction (`solver.rs:151` — wrong for fluid targets like
  `sulfuric-acid`, currently harmless only because the step-7 merger
  independently re-checks fluidness) gets fixed in `netflow.rs` via an
  item→fluid map built during matrix construction.
- `dependency_order`: emitted as a **DFS pre-order over the active
  recipe set using the tree walk's exact traversal rule**
  (target-rooted, ingredient order, first-visit wins) — *not* a
  generic Kahn/SCC order. Rationale: `dependency_order` feeds
  `placer.rs::order_specs` row placement directly; several e2e tests
  gate on `assert_golden_hash` over the whole layout, so a
  tie-breaking change would churn golden hashes even when flows are
  identical. With Phase 1's compatibility mode (same recipe set), the
  emitted order is identical to today's by construction.
- **Prerequisite fix in `placer.rs::order_specs`** (Phase 1,
  unconditional): its `producer` map is `item → single recipe`,
  last-write-wins (`placer.rs:240-244`) — safe today only because the
  tree walk never returns two recipes producing the same item. Under
  the LP (even compatibility mode: e.g. rocket-fuel's set contains
  both AOP and basic-oil, both producing petroleum-gas), a consumer
  can lose its ordering edge to all-but-the-last producer and get
  placed *above* one of its producers, breaking the lanes-run-south
  invariant. Fix: `item → set of producing recipes`, dependency edges
  to all of them. Small, low-risk, and needed before any mixed-recipe
  result can be laid out.
- Machine data (Phase 1): add `centrifuge` to the machine table and
  `centrifuging` to `category_machines()`; unmapped categories become
  a new `SolverError::UnmappedCategory` instead of the silent
  assembler fall-through; `rocket-building` gets an explicit
  unsupported error (silo batch semantics are unmodeled and should say
  so rather than pretend).

### Compatibility mode (Phase 1) vs free selection (Phase 3)

Adversarial review established that unrestricted cost-based selection
flips oil-consuming fixtures from basic-oil-processing to
advanced-oil-processing (AOP + full cracking is marginally cheaper per
gas under the frozen cost table once crude+water are free — and
strictly higher-yield per craft). That is the *correct* answer, but it
lands squarely on multi-machine 3-fluid-output refinery rows — the
layout shape the tier-6 probe documents as broken (32 pipe-isolation
errors; the `machine_count == 1` staggered-template gate, issue #277)
— and produces fluid surplus that nothing can physically route until
Phase 2. Swapping selection behavior before the layout engine can
absorb it would trade solver correctness for layout breakage.

So the default swap is split:

- **Compatibility mode (Phase 1)**: per solve, restrict the LP's
  columns to the recipe set the tree walk would select (JSON-first per
  item, honoring exclusions and `available_inputs`). Within that set
  the LP still nets flows — byproduct crediting (rocket-fuel's AOP gas
  offsets basic-oil production), fleet double-count elimination, and
  honest surplus bookkeeping all land — but recipe *selection* deltas
  are zero by construction. Analysis of the gated e2e corpus
  (tier1–5, stress, palette) under this mode: every fixture's
  restricted set has a single producer per item and no byproduct
  overlap, so pinned parity must be *exact* there, and no surplus
  entries appear — the gated suite stays green with unchanged golden
  hashes. Deltas are confined to non-gated probes (rocket-fuel, USP
  gauntlet).
- **Free selection (Phase 3)**: remove the restriction once Phase 2
  (surplus routing) and the multi-machine staggered 3-output template
  (#277 generalization) exist. Gate the flip on the Phase 0 unpinned
  delta report (below) so we know exactly which fixtures change and
  why before flipping.

### Determinism (hard requirement)

URL state and `.fls` snapshots promise byte-identical reproduction.
Mitigations, in order: stable column order (recipes.json order),
stable row order (item first-seen order), netted single-insert
coefficients, no wall-clock limits, a CI test that runs the full
290-item sweep twice and byte-compares serialized `SolverResult`s, and
a one-off native-vs-WASM comparison in Phase 0. Degenerate alternate
optima are the known hazard; the magnitude-separated objective is the
first defense (empirically robust per the ε sweep above),
lexicographic re-solve the second.

### Alternatives considered and rejected

- **Patch the tree walk with a byproduct ledger** (two-pass: walk,
  then net out). Becomes fixpoint iteration over a mutating demand
  vector — a bad hand-rolled LP without optimality or termination
  guarantees, and still can't price recipe alternatives. Rejected.
- **Gaussian elimination on a square recipe matrix** (early Kirk
  McDonald approach). Requires choosing *which* recipes are active
  before solving — but recipe choice is half the problem. No
  inequality/surplus handling. Its successor projects moved to LP for
  these reasons. Rejected.
- **MILP for integer machine counts.** Fractional counts are today's
  contract (layout ceils); integrality adds solve complexity for zero
  contract benefit. Rejected.
- **Interior-point (e.g. Clarabel).** Heavier dependency, interior
  (non-vertex) optima blend *more*, worse determinism story. Rejected.
- **Keep tree walk for easy chains, LP only for hard cases.** Two
  solvers, two behaviors, a classifier deciding which runs — the
  special-case accretion pattern this project's history warns about.
  Compatibility mode gets the same risk-reduction without a second
  solver: it is the *same* LP with a column filter. Rejected.

## Kill criteria

1. **Pinned parity on the gated corpus.** Phase 0 harness runs
   compatibility mode on every currently-green gated e2e config
   (tier1–5, the stress_* gates, palette tests — enumerated in the
   harness). Per recipe: `|count_LP − count_walk| > max(0.001
   machines, 0.1% of the larger)` fails; per external-input item, same
   tolerance on rates; any `surplus_outputs` entry on a gated config
   fails (analysis says none should exist — one appearing means the
   compatibility-mode restriction or the corpus analysis is wrong).
   Any failure → **stop; do not proceed to Phase 1 until root-caused.**
   On non-gated probes with byproduct overlap (rocket-fuel, USP),
   machine-count and surplus deltas are *expected* (they are the bug
   fix) and are reported, not gated — each must be hand-explained in
   the Phase 0 report as either crediting or double-count repair.
2. **Determinism.** If the 290-item double-run byte-comparison fails
   and cannot be fixed by input-ordering stabilization within Phase 0,
   swap microlp for the hand-rolled simplex (Bland's rule). If *that*
   also fails, kill the RFP — the approach conflicts with a
   non-negotiable project invariant.
3. **Cross-validation with the frozen cost table.** The cost table in
   this doc is frozen before any factoriolab comparison runs. If the
   LP disagrees with factoriolab (settings recorded alongside the
   comparison) by >1% on machine ratios for the four motivating cases
   (rocket-fuel, kovarex-forced U-235, coal-liquefaction-forced
   petroleum-gas, utility-science-pack), that is a formulation bug —
   stop before Phase 1. At most one documented cost-table revision is
   permitted before this criterion is evaluated; passing-by-retuning
   is not passing.
4. **Perf.** Native, full 290-item sweep: median solve ≤ 2ms and max
   ≤ 10ms (the RFP's own claim is µs–low-ms; missing the claim by
   >10× is a structural encoding problem even if UX survives). Any
   single solve > 50ms is an immediate abort-and-investigate. Also:
   solver iteration ceiling hit on any input = hard failure (see
   anti-cycling caveat), never a soft result.
5. **Cycle census stability.** If Phase 0's SCC census over *default*
   recipe selection finds any nontrivial multi-recipe SCC reachable by
   a default solve of any of the 290 producible items (the review
   census says none — only `pentapod-egg` self-loops by default),
   re-scope before Phase 1: the refusal policy would then be
   user-visible on default paths, not just forced/exotic ones.

*Scope review trigger (not a kill):* if Phase 0's netflow core
(matrix + solve + result assembly, excluding tests/harness) exceeds
~800 LOC, stop and review — we are overbuilding a microseconds-scale
LP. (LOC criteria have misfired here before — see
`rfp-streaming-reconciliation`'s retracted criterion — hence trigger,
not auto-kill.)

## Verification plan

Per the CLAUDE.md verification protocol, plus solver-specific checks:

- **Parity harness** (`crates/core/tests/solver_netflow_parity.rs`):
  - Compatibility-mode parity vs tree walk on every gated config
    (kill criterion 1, with the tolerances and surplus rule above).
  - Flow-conservation property over all 290 producible items: net
    production + external inputs − surplus = target, exactly. The
    tree walk *fails* this on oil/uranium chains; the harness
    documents each such case as a fixed bug, not a parity break.
  - **Unpinned delta report** (not a gate): free-selection solve of
    all 290 items; for each item whose recipe set differs from the
    tree walk's, record the delta. This is the evidence base for
    Phase 3's flip and for kill criterion 5's census.
  - Golden tests with hand-derived numbers:
    - `rocket-fuel@1/s` (compat mode): AOP sized by light-oil demand
      with its gas byproduct credited against basic-oil's share (so
      basic-oil shrinks vs the tree walk but need not vanish); exact
      machine counts hand-derived during Phase 0 and locked.
    - `rocket-fuel@1/s` (free mode, non-gating until Phase 3): AOP +
      3-way solid-fuel split, **zero surplus** — the reviewer-verified
      optimum, locked as the expected shape.
    - kovarex-forced `uranium-235@1/s`: Phase 0/1 expectation is the
      typed `UnsupportedSelfLoop` error; the 60-centrifuge rate answer
      becomes the expectation when Phase 2's self-loop support lands.
    - ε sensitivity sweep: 10×/100× perturbations of `ε_o`/`ε_m` must
      not change any golden solution.
- **Determinism test**: full-sweep double-run byte comparison (CI);
  one-off native-vs-WASM comparison during Phase 0; WASM bundle-size
  delta recorded.
- **Full e2e suite green** after Phase 1, with **zero golden-hash
  regenerations expected** (compatibility mode + preserved
  dependency-order semantics). A golden-hash change in Phase 1 is a
  bug in the compatibility restriction, not churn to wave through.
- **New validator check** (Phase 1): "machine output port with no
  physical extraction" — checks *entities* (inserter/pipe present at
  the port), not solver bookkeeping, because a `surplus_outputs` entry
  with no routed lane is exactly the stalled-refinery bug wearing a
  ledger. Self-loop-aware from day one (a self-consumed item is not
  "extracted" — though Phase 1's refusal policy means no self-loop
  spec reaches layout until Phase 2). Land it *before* the Phase 1
  default swap and confirm it correctly flags today's USP layout
  (expected-red on the non-gating gauntlet), then stays red until
  Phase 2 routes the surplus, then goes green. Gated corpus is
  unaffected (no surplus, no unextracted ports there).
- **Browser eyeball** (protocol step 2): rocket-fuel@1/s and USP@1/s
  after each phase; blueprint export round-trips through
  `blueprint-analyze`.

## Phasing

- **Phase 0 — spike behind a flag** (~1-2 sessions): `netflow.rs`,
  parity harness, unpinned delta report, golden tests, determinism
  test, SCC census, bundle-size check. No default change, no layout
  change. Kill criteria 1–5 all evaluated here.
- **Phase 1 — compatibility-mode default swap + honesty**: public
  `solve_*` entry points route to netflow in compatibility mode;
  `surplus_outputs` field added; cycle refusal (typed errors);
  `order_specs` multi-producer fix; port-extraction validator check;
  centrifuge data + `UnmappedCategory` + `rocket-building`
  unsupported; `is_fluid` fix. Standalone value: correct rates and
  fleet counts wherever byproduct overlap exists, honest surplus
  bookkeeping, loud errors where output was silently wrong, and a
  validator that stops certifying stalling refineries. Gated e2e
  stays green with unchanged hashes; the USP gauntlet goes honestly
  red on the new check until Phase 2.
- **Phase 2 — layout follow-through**: fluid surplus to perimeter;
  multi-item solid merge (the step-7 per-item loop currently computes
  each item's `merge_x` independently and **will overlap entities**
  for two solid outputs — needs a shared column cursor plus a
  multi-item fixture test; review finding, previously claimed "free");
  self-loop row template + self-loop-aware `row_kind`/`lane_planner`;
  partitioner audit for multi-solid-output specs
  (`apply_partition_plan`'s single `primary_solid_idx` assumption).
  Each layout piece gets its own design pass; this RFP commits only to
  the solver-side contract.
- **Phase 3 — free recipe selection**: remove the compatibility
  restriction, gated on Phase 2 + the #277 multi-machine staggered
  template + the Phase 0 unpinned delta report. Expected visible
  changes (AOP replacing basic-oil in oil fixtures) are enumerated
  from the delta report *before* the flip, and golden hashes for
  affected fixtures are regenerated deliberately, with browser
  eyeballs, in the same PR.
- **Phase 4 — UX**: recipe pins/exclusions in the sidebar; surplus
  outputs surfaced in the UI; error-banner routing for the new typed
  errors (today only `INCOMPATIBLE_MACHINE_PREFIX` gets the dedicated
  banner); optional low-usage-recipe consolidation pass if blends
  prove annoying in practice.

Phase boundaries are landable: Phase 0 is inert; Phase 1 changes
solver semantics only where the tree walk was demonstrably wrong,
plus loud refusals where it was silently wrong; Phases 2–4 are
independent follow-ups with their own gates.

## Decision log

- *2026-07-10 — drafted, following the July 2026 strategy review
  (solver identified as the weakest pipeline layer).*
- *2026-07-10 — adversarial review round: three independent reviewers
  (formulation/data, process/history, downstream contracts). Major
  outcomes folded into this revision: (1) unrestricted Phase-1 swap
  shown to flip green oil fixtures onto known-broken multi-fluid row
  shapes → compatibility-mode phasing introduced; (2) microlp
  duplicate-variable panic on same-item-both-sides recipes → netted
  coefficient requirement; (3) "self-loops are the only vanilla
  cycles" falsified (carbon↔coal 2-cycle, sulfuric-acid 4-cycle) →
  explicit cycle refusal policy, multi-recipe cycles out of scope;
  (4) "vertex ⇒ no blending" falsified by a provably-optimal 3-way
  solid-fuel split → claim corrected, consolidation pass deferred to
  Phase 4; (5) surplus entries moved to a new `surplus_outputs` field
  after `decomposition_search::compute_overproduction` was found to
  consume `external_outputs` with a targets-only assumption; (6)
  `order_specs` single-producer map identified as a Phase-1
  prerequisite fix; (7) step-7 multi-item merge collision identified —
  "solid surplus is free" claim retracted, moved to Phase 2; (8) kill
  criteria tightened (frozen cost table, defined parity tolerances,
  µs-calibrated perf gate, cycle census). Pending acceptance.*
- *2026-07-10 — accepted. Two open decisions resolved by the user:
  the port-extraction check reports **errors** (not warnings) from day
  one — honest red on the USP gauntlet and oil-chain URLs until Phase
  2; and this session attempts **Phase 0 + Phase 1**, with Phase 1
  landing only if all five kill criteria pass on Phase 0's data.*
- *2026-07-10 — Phase 0 landed (`netflow.rs` + parity harness). Kill
  criteria: **KC1 PASS** (exact dependency-order and external-order
  parity on all 23 gated configs, machine counts within tolerance,
  zero gated surplus — the pre-flight concern about
  `tier3_advanced_oil_processing_multi_machine` was unfounded: the
  walk picks basic-oil there, single output). **KC2 PASS**
  (byte-identical double sweep). **KC3 partial**: goldens are
  hand-derived/analytic; a factoriolab spot-check of the four
  motivating cases remains open. **KC4 PASS** with margin (release:
  median 223µs, p90 510µs, max 1.65ms across all 290 items). **KC5
  FIRED** — census below, needs ratification before Phase 1.
  Design amendments made during Phase 0, in response to observed
  failures (all reproducible via the harness):
  (a) **demand-closure column restriction** — columns limited to the
  fixpoint of "recipes net-producing a demanded item"; first defense
  against surplus-laundering sink chains (`ice` solve activating
  solid-fuel-from-ammonia + ammonia-rocket-fuel purely to shrink Σo);
  (b) **deterministic acyclic fallback** — an optimum containing an
  unsupported cycle retries with the first cycle member excluded whose
  demanded outputs all have alternative producers (breaks optional
  cycles; forced ones still refuse);
  (c) **surplus-processor exclusion** — active recipes with no product
  on the active demand tree are excluded and re-solved (second
  laundering variant, through in-closure demanded items);
  (d) **barreling recipes filtered** (`*-barrel` items) — fill/empty
  pairs are graph noise posing as fake alternative producers,
  factoriolab-style exclusion;
  (e) **cost-table revision 1** (the one permitted): `w_available`
  0 → 1e-4 and `ε_o` 1e-3 → 1e-8, after the delta report exposed a
  third laundering variant (overdriving the processing-unit recipe to
  16 PU/s surplus on a tesla-turret solve) that closure and
  reachability guards structurally cannot catch — free raw inputs
  were the root cause of all three variants. Post-revision the delta
  report is clean (56 recipe-set deltas, all explainable; large fluid
  surpluses on deep Aquilo chains verified as genuine — AOP pinned by
  lubricant demand).
  **KC5 census**: 18 of 290 items refuse under both free AND
  compatibility mode: 6 Gleba items on forced self-loops
  (pentapod-egg, fish-breeding) and 12 Aquilo items on the mandatory
  fluoroketone coolant loop (fresh fluoroketone is produced HOT; the
  only cold producer is the cooler; cryogenic-science-pack,
  quantum-processor and dependents). The tree walk "solves" all 18
  today with physically-broken output (e.g. agricultural-science-pack
  demands 0.5 pentapod-eggs/sec as an external input from nowhere;
  cryo/quantum strand hot fluoroketone → machines stall in-game).
  Every one has a user workaround: declare the loop-carried item
  (pentapod-egg, raw-fish, fluoroketone-cold) as an external input
  and the chain solves honestly. Phase-1 go/no-go on shipping these
  as typed errors: pending user decision.*
- *2026-07-10 — KC5 ratified by the user: **ship honest refusals**
  (typed errors for the 18 census items; grandfathering the tree walk
  rejected). KC3 closed on the analytic goldens (factoriolab
  spot-check downgraded to nice-to-have). All five kill criteria
  resolved → Phase 1 GO.*
- *2026-07-10 — **Phase 1 landed.** Public `solve_*` entry points now
  route through netflow in compatibility mode (tree walk retained as
  the recipe-selection oracle and parity reference,
  `solve_tree_walk_with_palette_and_exclusions`). Also landed:
  `order_specs` multi-producer dependency edges (placer.rs);
  `check_stranded_byproducts` validator (error severity, per
  decision); `centrifuge` machine data + `centrifuging` category
  mapping; explicit `GENERAL_CATEGORIES` whitelist — unknown recipe
  categories (`rocket-building`, `captive-spawner-process`) now fail
  machine pre-flight with the typed banner-routed error instead of
  silently landing on assemblers.
  Verification: full suite green **including unchanged golden layout
  hashes on every gated e2e config** (the compat-mode LP reproduces
  the walk's layouts byte-identically); science gauntlet shows the
  designed honest-red — utility-science-pack 0/0 → **FAIL×1**
  (stranded 9/s light-oil at the AOP rows, the game-stalling defect
  the old pipeline certified as clean); uranium-235 now solves onto
  centrifuges (was: silently AM3) with the U-238 co-product honestly
  reported as surplus; rocket-part returns a typed error (was:
  silently wrong AM3 layout). Phase 2 (fluid surplus routing +
  self-loop rows) is the path back to gauntlet-green for USP and the
  18 census items.*
- *2026-07-10 — **Phase 2 landed** (fluid surplus → perimeter, #277
  multi-machine staggered rows, multi-item merge cursor). USP gauntlet
  FAIL → PASS with its light-oil surplus physically routed and
  entity-verified; new forced-AOP fixture exercises 2 refineries + 2
  clean surplus exits. Design docs from three subagents (fluid-surplus
  lanes as per-item extensions with `perimeter_exit_y`; #277 pitch
  msz+1; self-loop rows) — the self-loop design is complete but
  UNIMPLEMENTED (blocked on a priority-splitter extension to the
  lane-rate walker; the walker models all splitters 50/50). Fixed in
  passing: the step-3.7 branch stamper silently skipped blocked tiles
  (now UG-bridges), and fluid TARGETS (not just surplus) now route to
  the perimeter — they were equally stranded, just unflagged
  (sulfuric-acid / heavy-oil-cracking golden hashes regenerated
  deliberately for the added exit trunks; both fixtures stay
  0 errors / 0 warnings).*
- *2026-07-10 — **Phase 3 attempted; flip HELD BACK, groundwork
  landed.** Free selection is fully implemented and verified at the
  solver level (`solve_free_with_palette_and_exclusions`): the
  incompatible-machine guard (dropped columns must not become silent
  imports; empty target row is a hard error), LP float snapping
  (15.000000000000016 must not ceil to 16 machines or flip the 15/s
  belt-tier threshold), and the refusal census extended with the
  unsupported-category family (biolab via captive-spawner-process).
  Under the flip the entire gated e2e corpus passes (the forced-AOP
  fixture needs 24/s to stay multi-machine — free selection's
  cracking chain yields 97.5 gas/craft vs 55). The flip was reverted
  to compatibility mode as default because of ONE remaining layout
  gap: dense oil complexes (AOP + both cracking rows, e.g. USP free
  mode) put adjacent fluid trunks' surface-filled short anchor gaps
  in the same y-band, which merge (F1) — USP would regress PASS →
  FAIL×2. Also known: the trunk walker's tail mis-chains the last UG
  pair when a config has 3+ stacked perimeter exits (budgeted at ≤1
  fluid-network error in the forced-AOP fixture, FIXME in e2e.rs).
  Next session: fix the fluid-lane stagger for dense oil complexes +
  the exit-tail chain, then flip the default (one-line change in
  solver.rs) and re-bless the delta-report fixtures.*
- *2026-07-11 — **Phase 3 FLIPPED: free cost-based selection is the
  default.** Both blockers root-caused via walker instrumentation and
  fixed: (1) the trunk walker's UG-S pick no longer slides past
  foreign-tap rows (a PTG ON a foreign tap row is F5a-safe; the slide
  surface-filled the very rows it avoided — the dense-oil merge
  vector) and its range caps at y1−2 so the UG pair always fits (the
  stacked-exit tail bug); (2) an anchor-spacer pass separates adjacent
  fluid lanes whose surface-anchor sets intersect (cross-ROW port
  sharing that no template stagger covers), with `source_y` counted
  only when the entry can't be a UG. USP free mode: 0 errors /
  0 warnings, zero surplus (everything cracked). Full suite green with
  zero golden-hash churn; gauntlet 4 PASS / 2 WARN / 0 FAIL.
  Compatibility mode retained as `solve_compat_with_palette_and_
  exclusions`. Forced-AOP fixture at 24/s (multi-machine under
  cracking) asserts zero errors — the exit-tail budget was removed.*
- *2026-07-11 — **Phase 2(c) self-loop rows LANDED** (three subagents:
  solver netting, priority-splitter rate walkers, row template).
  Netflow emits pure-solid self-loop recipes (kovarex,
  iron/copper-bacteria-cultivation) with NET flows in inputs/outputs
  plus raw recirculation in `MachineSpec.self_loop`; both lane-rate
  walkers honor `PlacedEntity.loop_priority_rate` (loop branch =
  min(total, cap), replacing 50/50 for `:selfloop:`-tagged splitters
  only); `check_belt_loops` exempts the tagged recirculation;
  `templates::self_loop_row` builds the physical loop (3 north belts,
  2 south belts, priority splitter, corridors that tunnel under the
  output merger). `tier_kovarex_self_loop` fixture: uranium-235@0.1/s
  → 6 centrifuges, **0 errors / 0 warnings**, round-trip clean. Fixed
  in passing: centrifuge missing from two machine-entity tables.
  Still refused (typed): coal-liquefaction (fluid self-loop item) and
  pentapod-egg/fish-breeding (water ingredient) — fluid self-loops
  are the remaining follow-up, along with high-rate self-loop row
  splitting (v1 caps documented in the template) and the fluoroketone
  coolant loop (multi-recipe cycle, needs the forced-surplus edge-cut
  or its own RFP). Browser eyeball of the kovarex row is outstanding
  (user validates UI per project convention).*
