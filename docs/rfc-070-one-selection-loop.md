# RFC-070: One Selection Loop

## Summary

Replace `select_best_decomposition` — the production candidate-selection
loop, which today braids three incompatible verdict mechanisms through
~880 lines and lets three of its seven candidate arms carry bespoke
refusal logic inside their own `produce()` — with a single policy-driven selection loop derived from
`candidate_runner`'s architecture, migrated under a netflow-style
shadow/parity/flip sequence. After this RFC, "how does a candidate
layout win?" has exactly one answer, expressed as policy data in one
place: one severity-aware verdict model, one ranking, one calibration
point for every floor and firewall. Every future capability (celldb
template promotion included) integrates by registering a producer, not
by growing an eighth bespoke arm.

## Motivation

The 2026-08 off-path audit (`docs/offpath-code-followups.md`, #675) and
the churn audit (`docs/pr-churn-audit-2026-08.md`) converge on the same
diagnosis: progress stalls because every engine change carries more
independently-falsifiable claims than its verification covers — and
candidate selection is the worst offender, because every capability
must integrate there and each one has grown its own verdict logic.

**What exists today** (all confirmed at source, 2026-08-21):

- The only production selection loop is
  `decomposition_search::select_best_decomposition`
  (`crates/core/src/bus/decomposition_search.rs:1056`, reached
  unconditionally from `bus/layout.rs:300`). The supposed "DI candidate
  mode" and "cell-composition candidate mode" are not sibling loops —
  they are flag-gated candidate arms *inside* it
  (`DirectInsertionCandidate` :144, `CellComposedCandidate` :325, among
  **seven candidate arms**: six implementors of the
  `DecompositionCandidate` trait (:65) plus the `k1-shape-fix` arm,
  which is an inline closure passed to `run_candidate` (≈:1150,
  `build_k1_enrollment_plan`) carrying a bespoke plan argument the
  trait signature cannot pass —
  a distinction Phase 1b must handle explicitly, see Design).
- **Three verdict mechanisms coexist in the one function**:
  1. a generic soft score (`score_layout` :896 — density minus
     overproduction minus entity count), hard-gated only on missing
     balancer templates;
  2. the component-wise `IssueCounts` floor (:827) used for the DI and
     horizontal pairwise "never-worse" comparisons — deliberately
     **non-lexicographic** (the comment at :817-825 explains that a
     derived ordering would let a layout-warning regression hide behind
     a validator-warning improvement), ties to native;
  3. an `ErrorKinds` lexicographic quality key (:754, `quality_key`
     :770 — structural errors dominate weighted functional errors),
     used only for the scoped merge-tap choice.
- **Per-arm bespoke behavior**: DI, horizontal-stack, and cell-composed
  self-refuse on structural error inside their own `produce()`
  (:191-211, :295-311, :358-374); five of seven arms run under
  `catch_unwind` (:996) and two do not; the #519/#520
  warning-recalibration firewall (:862-883) and the corpus-timeout cost
  gating (:1040-1054) are further loop-adjacent special cases. The
  final winner falls out of a four-stage precedence chain (:1589-1600)
  whose stages use different verdict mechanisms.
- `candidate_runner.rs` (RFC-064 Phase 2b) is a structurally clean
  produce→verdict→rank loop — incumbent ranked as a real competitor,
  uniform validation, scoreboard result type — but has **zero
  production callers** (offpath audit TSV) and is semantically
  incomplete: its verdict path (`verdict::never_worse`,
  `verdict.rs:352`) is **severity-blind** (errors and warnings in one
  category are fungible counts), knows nothing of
  `selection_warning_count` (`validate/mod.rs:424`) or
  `SELECTION_EXCLUDED_WARNING_CATEGORIES` (:416), has no
  `catch_unwind`, and no sim-anchor firewall. (Line anchors here and
  throughout are as-of 2026-08-21 origin/main — re-locate before
  editing.)

**Why this is worth an RFC-scale arc** — the incident record shows
selection miscalibration damages shipped output, not just velocity:
#519 shipped 68%-of-plan factories through a selection exclusion; the
#644 phantom warnings steered winners for three days; and the #686
firing census proved two checks (`input-rate-delivery`, `belt-detour`)
do *invisible* selection work — they fire only on losers, so corpus
quietness proves nothing about them. Three mechanisms means three
places to miscalibrate and three places to firewall; nobody can state
today's complete selection semantics without reading ~900 lines.

The reproducible failing case, per the template's preference: run the
#686 census — the current machinery's verdicts cannot even be *audited*
per-mechanism without the instrument this RFC builds in Phase 0.

## Design

**Central call: generation 4's shape absorbs generation 1's semantics
as explicit policy.** The unified loop keeps `candidate_runner`'s
architecture; the verdict layer is rebuilt so that everything currently
scattered across arms becomes data the loop is configured with:

- **`SelectionPolicy`** — one type expressing, per validator category:
  severity channels (errors vs selection-participating warnings vs
  layout warnings — the three channels `IssueCounts` tracks today),
  gate mode (hard floor / pairwise never-worse / report-only), and
  participation (the `SELECTION_EXCLUDED_WARNING_CATEGORIES` set
  becomes policy data). The non-lexicographic floor comparison and the
  structural-dominance key are policy *combinators*, not open-coded
  mechanisms. `verdict.rs` gains severity awareness; `never_worse`'s
  category-count model is extended, not discarded.
- **Uniform loop stages** replace per-arm bespoke behavior: structural
  self-refusal, `catch_unwind`, and cost gating (the corpus 600s
  timeout is a real constraint — the loop stays sequential) move into
  the runner and apply to every producer identically. The #519
  firewall becomes a named policy guard with its calibration receipt.
- **Producers** — the six existing `DecompositionCandidate`
  implementors register unchanged at first; the trait survives. The
  `k1-shape-fix` arm is **not** a trait implementor — it is an inline
  closure carrying a `build_k1_enrollment_plan` plan argument the
  trait cannot pass — so Phase 1b specifies a plan-accepting producer
  variant for it (a producer constructed *with* its plan, or a
  two-stage producer contract) rather than pretending it registers as
  a plain arm. The incumbent (native) is a ranked competitor, as in
  `candidate_runner` today.
- **`LayoutOptions` split** — the doc-only legend at
  `bus/layout.rs:79-111` is promoted to types: user-pinned constraints
  (public boundary; belt tier is never a search axis, per standing
  rule) vs base-production search axes (internal). The two unclassified
  fields (`planning_duty` :136, `research_productivity` :173) get
  classified as part of this phase. This is the structural fix for
  dead strategies hiding behind "user-reachable via a flag"
  (`row_rotation`, `band_packing` precedents).

**Migration shape — the netflow compat pattern**
(`docs/rfc-solver-net-flow.md`): the v2 loop first runs in shadow
behind `select_best_decomposition` under a parity gate (same winner, or
an adjudicated divergence), flips only after corpus-wide parity plus
sim anchors on divergences, then the old loop is deleted with the flip
condition named in this decision log.

**Rejected alternatives**:

- *Adopt `candidate_runner`/`never_worse` as-is.* Severity-blind
  verdicts would silently change selection semantics in exactly the
  #519/#644 failure class. The clean shape is worth keeping; the
  semantics are not optional.
- *Keep gen 1 and delete gen 4.* Viable (it is K70-1's contingency),
  but it leaves three mechanisms braided and the integration tax
  standing; it is the fallback, not the plan.
- *Parallelize the loop.* The corpus timeout is managed by cost
  gating today; parallelism trades a measured constraint for
  nondeterministic scheduling in WASM-hostile territory. Out of scope.
- *Rewrite the big modules while we're in there.* `templates.rs` /
  `ghost_router.rs` are near-perfectly exercised and domain-essential;
  rewriting trades verified code for unverified code. Explicit
  non-goal.

**Cross-RFC obligation**: `candidate_runner` is RFC-068's entry path
(`template_candidate.rs` registers only there; "inert by
construction"). RFC-068 P1 remains owner-gated and is **not** part of
this campaign; in exchange, every Phase-2 change keeps the celldb
harness (`tests/celldb_template.rs`, `tests/candidate_runner.rs`)
green, and the post-migration loop must offer RFC-068 an equivalent
inert registration point. After unification, RFC-068's promotion path
becomes "register `TemplateCandidate` as a producer behind its sim
gate" — strictly simpler than today's.

## Kill criteria

- **K70-1 (expressibility — the premise test).** Adjudicated at the
  **Phase-2a shadow gate** (Phase 1b's acceptance is the offline
  precursor: policy replay against the Phase-0 baseline records). The
  v2 shadow loop — built with the per-arm bespoke logic *already
  hoisted* into uniform stages, so the shadow exercises the target
  architecture, not a delegating shim — must reproduce the Phase-0
  baseline winner on every fixture of the named parity corpus using
  policy data alone. If matching the baseline requires
  *candidate-identity-conditioned verdict logic anywhere* — in the
  loop body **or surviving inside a producer's `produce()`** (any
  branch keyed on a producer's identity rather than on policy), the
  "four answers to one question" premise is false: stop **before any
  flip**, keep generation 1, and instead delete generation 4's unused
  surface (`objective.rs`/`verdict.rs` beyond what RFC-068 needs).
  Record and archive. (Absorbed from #688 review round 1: the
  original phasing hoisted the arms' self-refusal *after* the flip,
  which would have let K70-1 pass trivially on functionally-unchanged
  arms and pushed the falsification past the point where it is
  cheapest to act on.)
- **K70-2 (parity budget).** At the Phase-2b flip gate, over the
  **named parity corpus** — the explicit fixture × machine-tier list
  committed with the Phase-0c baseline PR, together with the
  divergence-equivalence rule used to adjudicate — if more than **3**
  fixtures diverge from baseline *after* policy fixes, or any single
  divergence cannot be adjudicated equal-or-sim-verified-better, the
  flip halts and the RFC pauses for re-design. Divergences are
  adjudicated individually in this log. K70-1/K70-2 are not
  re-runnable until that corpus is named; naming it is a Phase-0c
  deliverable, not an option.
- **K70-3 (cost).** Comparator: **isolated runs** — the v2-only loop
  vs the gen-1-only loop on the same corpus and machine. If v2 in
  isolation regresses corpus wall-time >10% vs gen 1 in isolation, or
  the *production configuration* (never the doubled shadow
  configuration, whose 2× cost is accepted and CI-scoped by
  construction) trips the stress corpus's 600s timeout where it did
  not before, the design is wrong — fix the cost gating before any
  further phase, or stop.

## Verification plan

Per the CLAUDE.md layout-engine protocol, plus campaign-specific
instruments:

1. **Phase-0 baseline is the oracle.** The selection scoreboard
   (extension of #686's census) records, per corpus fixture: candidate
   set, per-candidate verdicts under each mechanism, winner, and
   *which precedence-chain stage decided it*. Committed as data;
   every later phase diffs against it.
2. **Shadow parity in CI** (Phase 2a onward): the v2 loop runs beside
   production on the **named parity corpus** (fixture × machine-tier
   list committed with the Phase-0c baseline); winner mismatch fails
   the check. The census slice alone is NOT the parity corpus — it is
   a hardcoded approximation that cannot re-enact the search-internal
   arms.
3. **Sim anchors** on every adjudicated divergence and on a
   pre-registered contested sample (fixtures where `di_choice` /
   `horizontal_choice` actually fired) before the flip.
4. **Meter tripwire** (separate track, dependency): report-only
   below-plan meter check over e2e exports — "meter says below plan ⇒
   believe it" applied to every campaign PR.
5. **Standing suites**: full e2e green, clippy, WASM build, celldb
   harness green, region/netflow fixtures untouched-or-explained.
6. **Validator-trust duty**: any severity/participation change that
   falls out of policy unification updates
   `docs/validator-trust.md` in the same PR.

### The named parity corpus, and the divergence-equivalence rule

Landed Phase 0c (#689 W1c). The corpus is
`crates/core/tests/parity_corpus.rs` — an explicit **fixture × machine
tier × option set** grid — and its committed result is
`crates/core/tests/parity_corpus_baseline.json`
(`SPAGHETTIO_PARITY_CORPUS=bless|check`, `#[ignore]`d, never CI-gated:
like the stress goldens it is host-cache-relative and must be run with
the zone-cache pin).

**160 cells.** 12 fixtures (the #691 corpus verbatim: G2's six
tier-ladder solves plus the six e2e "from-ore" ones) × the machine tiers
each recipe permits (three assembler tiers, or the one chemical plant)
× five option sets — `default`, `cells-off`, `e2e-harness`, `di-off`,
`hs-off`. 140 decide; 20 are recorded `no-solve`
(`assembling-machine-1`'s two ingredient slots cannot run
advanced-circuit or processing-unit). The option-set axis is not
optional decoration: it is the axis W1b's finding made load-bearing, and
it is where the corpus's claim surface lives (below).

Two cells are **equal** iff their `(status, winner, deciding stage)`
triples are equal. Nothing else is compared. The per-candidate outcome
vector is recorded alongside, for adjudication only; the verdict
NUMBERS are deliberately absent, because they are structurally holed
(see the Phase-0b oracle-gaps entry) and a baseline pinning them would
pin gaps as facts.

- **Minor divergence** — same `status` and `winner`, different deciding
  stage. The shadow loop reached the same shipped layout by answering a
  different question, which is expected wherever v2's policy merges two
  of today's five stages. Adjudicated individually in this log; **no sim
  required**, because no shipped geometry changed.
- **Major divergence** — `winner` or `status` differs. A different
  layout ships. Adjudicated individually in this log **and sim-anchored
  before the flip**.
- A **new or missing cell** is a major divergence by definition: the
  candidate field moved, and the corpus must be re-taken and re-named
  here before parity means anything.
- K70-2's budget of 3 counts diverging **FIXTURES**, per its own wording
  — not cells. One fixture spans up to 15 cells, so a single policy
  difference would otherwise spend the whole budget in one place.

## Phasing

- **Phase 0 — instruments.** 0b: scoreboard instrumentation of
  `select_best_decomposition` (trace events for verdicts and the
  deciding precedence stage) + census extension. 0c: committed corpus
  baseline, plus the machine-tier axis widening; sim-anchor the
  contested sample. (0a, the meter tripwire, is a parallel campaign
  track, not this RFC's deliverable — listed here because Phase 2
  leans on it.)
- **Phase 1 — contracts.** 1a: `LayoutOptions` constraint/axis split.
  1b: `SelectionPolicy` + severity-aware `verdict.rs`; acceptance =
  offline policy replay reproduces the baseline verdicts recorded in
  Phase 0 (the precursor to K70-1, which is adjudicated live at 2a).
  1b also specifies the plan-accepting producer variant for the
  `k1-shape-fix` arm. **Owner design review gate before Phase 2.**
- **Phase 2 — migration.** 2a: v2 loop **built with the per-arm
  bespoke logic already hoisted into uniform stages** (structural
  refusal, `catch_unwind`, cost gating), run in shadow beside
  production + parity CI gate — **K70-1 adjudicated here**, before
  anything flips. 2b: flip `build_bus_layout` to v2 (**owner evidence
  review gate**; K70-2 adjudicated here). 2c: delete the gen-1 loop —
  flip condition for the deletion: parity held through 2b on the
  named corpus with zero unadjudicated divergences. (There is no
  post-flip hoist phase: the hoist happens in 2a's construction so
  the shadow exercises the final architecture — absorbed from #688
  review round 1.)

Each phase lands as sub-400-line PRs (scaffolding / behavior /
fixtures / docs split per the churn norm).

## Decision log

- *2026-08-21 — RFC opened; registry number 070 claimed. Campaign
  context: off-path audit complete (#675), which mapped the four
  selection generations and their nesting; source map in Motivation
  verified at origin/main the same day.*
- *2026-08-21 — RFC-068 P1 explicitly excluded from this campaign
  (remains owner-gated on #675). Phase 2 carries the celldb-harness
  obligation instead; see Cross-RFC obligation.*
- *2026-08-21 — owner review gates fixed at two points (post-Phase-1
  design, pre-2b flip); all other decisions proceed autonomously and
  land here.*
- *2026-08-21 — #688 review round 1 absorbed in full (no refutations):
  (a) the post-flip hoist phase is deleted — v2 is built with per-arm
  bespoke logic hoisted from the start, and K70-1 adjudicates at the
  2a shadow gate, not after the flip (the original sequencing would
  have let K70-1 pass trivially on unchanged arms); (b) candidate-arm
  count corrected — six trait implementors + the `k1-shape-fix`
  closure arm, whose plan argument needs a plan-accepting producer
  variant (Phase 1b); (c) K70-3's comparator named (isolated v2 vs
  gen-1 runs; shadow's 2× cost excluded by construction); (d) the
  parity corpus is a named Phase-0c deliverable, not an assumption;
  (e) line-anchor nits fixed with an as-of note.*
- *2026-08-21 — **Phase 0b landed** (#689 track W1b): two trace events —
  `SelectionCandidateEvaluated`, one per candidate SLOT (all seven, every
  call, including the ones a gate excluded before they cost a layout
  pass) and `SelectionDecided` (winner + deciding stage). Design call,
  and the one the later phases depend on: the events **record what each
  verdict mechanism already computed at its own site, and never
  recompute**. A scoreboard that re-ran `validate()` on the side could
  disagree with the number the decision actually used, and Phase 2a
  diffs the shadow loop against these records — an oracle that disagrees
  with the thing it is oracling is worse than none. The price is
  structural holes (below); Phase 1b's offline policy replay must treat
  a missing count as "nothing computed this", never as zero.*
- *2026-08-21 — **the precedence chain is named as FIVE stages**, where
  Motivation above calls it four. The first `.or()` link answers with two
  different mechanisms: merge-tap's `ErrorKinds::quality_key` verdict
  (which can name merge-tap OR native) and the DI/horizontal pairwise
  `IssueCounts` resolution that may displace it. Collapsing them into one
  tag would lose which question was asked, which is the column K70-1
  turns on. Measured over the #686 census slice (six fixtures, default
  options): `best-error-free` ×4, `merge-tap` ×1, `scoped-pairwise` ×1;
  `best-accepted` and `first-produced` never fired **on that slice** —
  see the next entry, which found `best-accepted` deciding as soon as the
  option set changes. The stage distribution is a function of the OPTIONS,
  not just the fixture.*
- *2026-08-21 — **the e2e harness does not run production's candidate
  set**, found by decoding a `tier1_iron_gear_wheel` snapshot and getting
  a different deciding stage than the census reported for the same
  fixture (`best-accepted` vs `best-error-free`). Cause:
  `LayoutOptions::default()` sets `cell_composition: Candidate`
  (`bus/layout.rs:241`), but `run_e2e` spells the field explicitly as
  `cell_composition: Default::default()` (`tests/e2e.rs:355`) — the
  ENUM's default, which is `Off` (`bus/cells/mod.rs:24`) — next to a
  `..Default::default()` that would have given `Candidate`. The line
  dates to RFC-051 Phase A when Off WAS the struct default (`5090da99`);
  Phase B flipped the struct and the harness kept resolving to the enum.
  Consequence for the mechanism: with cells off, only native produces,
  `clean_flags` is skipped by its own `n_layouts > 1` guard, the
  error-free tier is empty, and `best-accepted` decides — which is why
  the whole e2e suite exercises a stage the census slice never reaches.
  **Not fixed here**: flipping it changes the candidate set under every
  regression test, i.e. a selection change, which is a campaign call and
  not an additive Phase-0b one. Two obligations fall out: (1) the
  Phase-0c parity corpus must be defined over fixture × MACHINE TIER ×
  OPTION SET, since "same fixture" does not pin the candidate field; (2)
  suite greenness is not evidence about the cell-composed arm, whose
  production exposure the e2e corpus does not cover at all.*
- *2026-08-21 — **Phase-0b oracle gaps**, recorded so no later phase
  assumes the baseline says more than it does. (a) Issue counts exist
  only where a comparison needed them: a merge-tap-decided selection
  short-circuits the `clean_flags` tier entirely, so the only counts such
  a fixture can carry are whatever a scoped pairwise already computed —
  and on `ec@30`, where DI and horizontal both refused, that is NOTHING
  for either native or merge-tap, leaving the kinds key as the whole of
  what that decision looked at. (Corrected after #692 review, 2/3: the
  first wording said a merge-tap decision means no counts, full stop.
  It does not — native's counts DO get recorded whenever DI or
  horizontal produced, since `di_choice`/`horizontal_choice` run on the
  produced-but-unaccepted native that merge-tap's own gate requires. The
  `ec@30` observation was true; the generalisation from it was not.)
  Likewise a scoped-pairwise-decided selection leaves every
  non-participant countless. (b) `ErrorKinds` is
  computed only by the merge-tap decision, so no candidate on a
  non-Pooled or native-clean solve has one. (c) A `not-run` row has no
  reason: the gates (`try_cells` / `try_di` / `try_horizontal` /
  `try_k1_shape_fix` / `try_size_split` / `try_merge_tap`) are
  conjunctions of booleans at the call site, so the scoreboard can say a
  candidate was not tried but not WHICH conjunct excluded it — Phase 1b's
  uniform loop should make the gate a first-class reportable predicate.
  (d) **The refusal-attribution gap #686 named is NOT closed**: DI,
  horizontal-stack and cell-composed stringify their own validation
  failure as `e.to_string().lines().next()`, which yields
  "…failed validation: Validation failed:" and discards the issue list
  before anything can record it. Which CATEGORIES fire inside a
  self-refused candidate therefore remains invisible; recovering it means
  changing what `produce()` keeps, which is not additive and was kept out
  of 0b. (e) DI's internal `DiClaimOrder::Search` two-arm race lives
  inside `produce()`; the scoreboard sees one DI row, not two arms —
  moot under the default, which is `Downstream` (checking this turned up
  a stale in-code comment claiming the default was `Upstream`, corrected
  in the same PR; RFC-059's sim close-out had flipped it). (f) The scoreboard is per selection
  CALL — a candidate whose `produce` runs its own search emits its own
  block; none occurred on the census slice. (g) …and only the WINNER's
  nested blocks survive at all: `run_candidate` truncates each
  candidate's events out of the collector and replays only the winner's,
  so a LOSING candidate's inner selection is dropped before any reader
  sees it. Absence of a nested block is not evidence that none ran — a
  shadow-loop diff assuming otherwise would be comparing against a stream
  the production loop deliberately edits.*
- *2026-08-21 — **#692 review round 2 adjudicated** (7 findings). Absorbed:
  (a) the Phase-0b oracle had NO CI assertion coverage — everything was
  `#[ignore]`d, so a broken stage tag or a dropped row shipped green. Added
  `selection_scoreboard_contract`, a non-ignored test pinning the contract
  (every slot emits a row, canonical order, rows before the terminal event,
  winner ∈ its own block's rows, `not-run` ≠ `refused`, and the fixture's
  deciding stage). **Discrimination check executed, not assumed**: mis-tagging
  the error-free stage, dropping the `native` row, and swapping two runs in
  the index list each made it fail with a message naming the right cause;
  restored and re-verified green. (b) The four hand-maintained same-order
  candidate lists are now one: `CandidateRun` carries its own name, a
  `CANDIDATE_ORDER` const is the single source, and `Scoreboard::from_runs`
  plus a `candidates` post-check assert each run's own name against its slot,
  so a reorder panics instead of recording one candidate's verdicts against
  another's row. (c) The census's nested-block banner no longer labels the
  OUTER block, and the block walker now checks that a winner is among its own
  block's rows, printing a loud marker if not. Refuted with receipts:
  (d) "the failure path emits with the sink detached" — the sink is
  re-attached at `decomposition_search.rs:1659` and the failure-path
  `board.emit()` is at `:1881`, 222 lines later with no intervening
  `swap_sink` (anchors re-located against merged main, W1c: the round-2
  entry quoted `:1604`/`:1814`, which do not point at those statements in
  the file as merged — the argument is unchanged, the coordinates were
  not); failure-path rows reach a streaming consumer exactly like
  success-path ones, so no doc note was added for a behaviour that does not
  exist. (e) "the e2e cells-off gap is left unfixed" — correct and
  deliberate, recorded in the entry above and on #689.*
- *2026-08-21 — **`PlacedEntity::rate` is NOT validator-visible; the no-op
  label is redefined accordingly.** Two reviews claimed opposite things, so
  it was settled by count: all 21 `.rate` reads across `validate/` and
  `connectivity.rs` are on solver `ItemFlow`s, none on a `PlacedEntity`, and
  the three `belt_flow.rs` lines #686 round 7 cited as proof read `e.carries`,
  `e.carries` and `build_ug_pairs`. #686's adjudication was therefore wrong
  about `rate` (right about `carries`). Decision: KEEP `rate` in
  `EntitySignature` — a differing stamp means a different lane-family decision
  reached the same tiles, which is worth not calling identical — and redefine
  the label at every site that states it: **a no-op is "tiles AND stamps
  identical", not "validator-identical"**. That is a strictly stricter test,
  so it can only under-report no-ops, never over-report them; measured, it
  changes nothing on the current slice (ratios identical to the pre-`rate`
  run). Dropping `rate` was the alternative and was rejected: it would make
  the label mean less while matching neither the recorded #675 follow-up nor
  the provenance question the counter exists to answer.*
- *2026-08-21 — **#692 review round 3 adjudicated** (6 minors, no majors;
  closing round). Absorbed: the all-refused message's refusal reasons now come
  from the checked `run_refs`/`CANDIDATE_ORDER` pair instead of a hand-typed
  7-slot tuple zipped positionally against candidate names — the same
  misattribution class round 2 retired elsewhere, and the last positional
  literal in the function; the two alignment checks became `debug_assert_eq!`,
  so the tripwire fires in every debug build (which is what `cargo test` and
  CI run — coverage unchanged) while release and WASM cannot panic a browser
  solve over a code-level ordering mistake, restoring the degradation
  philosophy the `catch_unwind` arms twenty lines away already follow; the
  contract test's `decided.len()`, row-order and event-order assertions now
  each name BOTH readings of a failure (engine legitimately changed vs.
  instrumentation broke) and say which sibling assertion discriminates,
  extending what the stage assertion already did; the no-op denominator
  header now states that "+N with no default" counts SUCCESSFUL builds
  lacking a baseline and that a variant's own refusals never reach that tally
  (they are in the refusals summary), so it cannot be read as "all builds with
  a failing default"; and a doc said `count_issues` runs at five sites when it
  runs at six. Adjudicated as designed, no code change: a single scoreboard row
  can carry counts sourced to one mechanism and kinds to another — that is what
  a per-candidate summary across three mechanisms IS, and the decision
  authority is `SelectionDecided::stage`, which the row does not duplicate; one
  clarifying sentence added at the field doc. Incidental finding, NOT fixed and
  out of scope: `ghost_occupancy::is_claimed` is dead code in RELEASE builds
  because its only non-test uses are inside `debug_assert!`s in
  `ghost_router.rs` — pre-existing, and invisible to CI because the clippy job
  is debug-only. It surfaced here because the same hazard applied to
  `CANDIDATE_ORDER`, which is why the refusal message now uses it for real
  rather than only inside assertions.*
- *2026-08-21 — **Phase 0c landed** (#689 track W1c): the parity corpus is
  NAMED, at **160 cells** (12 fixtures × permitted machine tiers × 5 option
  sets; 140 decide, 20 `no-solve`), committed as
  `crates/core/tests/parity_corpus_baseline.json`, together with the
  divergence-equivalence rule above. K70-1 and K70-2 are re-runnable from
  here. Equality is `(status, winner, deciding stage)` and nothing else —
  the verdict numbers stay out of the baseline on the Phase-0b principle
  that a hole must not be committed as a fact.*
- *2026-08-21 — **the option-set axis carries the claim surface, and it is
  large**: 15 of the 32 fixture×machine rows change verdict somewhere
  across the five option sets — **10 major** (winner changes) and **12
  minor** (stage-only), 22 changed cells of 140 decided. The stage
  distribution over the whole corpus is `best-error-free` 87,
  `scoped-pairwise` 26, `merge-tap` 15, `best-accepted` 12,
  `first-produced` 0 — so four of the five stages are live and the census
  slice's "best-accepted and first-produced never fire" was a property of
  that slice, not of the engine. Every minor divergence has the same
  mechanism (W1b's): with cells off only native produces, `clean_flags` is
  skipped by its `n_layouts > 1` guard, and the decision falls from
  `best-error-free` to `best-accepted`. **`first-produced` remains
  unmeasured on any corpus** — a shadow loop can reproduce this baseline
  without ever exercising that arm, and Phase 2a must not read corpus
  parity as evidence about it.*
- *2026-08-21 — **the e2e harness carries a SECOND fossil of the same
  shape**, found while building the corpus: `run_e2e_inner` also pins
  `inserter_capacity: 0` (`tests/e2e.rs:354`), which was the struct default
  when the line was written (`40fd48dc`, RFC-049 Phase 1, 2026-07-22) and
  went stale two days later when #383 flipped the default to
  `DEFAULT_INSERTER_CAPACITY` = 2. Identical cause to the
  `cell_composition` fossil, in the same struct literal. It is not
  cosmetic: `e2e-harness` (cells off AND capacity 0) diverges from BOTH
  `default` and `cells-off` on two cells —
  `tier2_ec_am1_10_ore` and `e2e_tier2_electronic_circuit_20s_from_ore`,
  both at `assembling-machine-3`, where `direct-insertion` wins under
  default and cells-off but `native` wins once the capacity drops. So the
  inserter ladder independently steers selection, and a corpus that had
  modelled the harness as "cells-off" — the W1c brief's stated minimum —
  would have recorded the wrong winner for the configuration the whole
  regression suite runs under. Both fossils stay UNFIXED here for the
  reason W1b gave: flipping either changes the candidate set under every
  regression test, which is a Wave-2 campaign call, not a Phase-0 one.
  What changes is that the corpus now measures both.*
- *2026-08-21 — **contested sample SIM-ANCHORED** (Verification plan item 3;
  both runs `--warmup 432000 --speed 32`, entity-count-exact against the
  meter tripwire's blessed rows, so the sim and the meter measured the same
  artifacts).*
  - ***`ac5-am2` (scoped-pairwise, winner `horizontal-stack`): PASS.***
    5.03/s delivered vs 5.00 planned (**+0.6%**; produced 4.92/s, −1.6%),
    every stage at plan (cable +0.2%, copper-plate 0.0%, EC −0.2%,
    iron-plate 0.0%, plastic 0.0%, petroleum −0.6%), 136/136 machines
    working, kit-clean, 12 checkpoints. `converged: false`, but the series
    OSCILLATES across plan (4.92 ↔ 5.12, period 2) rather than decaying —
    the harness's decay test reads a sawtooth as a trend. **So the one
    pairwise displacement of native in the corpus is measured good.**
    Caveat that must travel with the number: the run needs
    `--research-productivity plastic-bar=0.10` (the force realizes +10% on
    plastic and the harness kit-fails a manifest declaring 0), and the
    declared-axis export is **2125 entities against the default cell's
    2134** — same dims, 9 entities apart, so this anchors a
    near-neighbour of the corpus cell, not the cell itself. That also
    retires `status.md`'s "AC is bit-identical declared-or-not". The
    undeclared run measured identically on every solid stage and differed
    only on petroleum (−8.9% vs −0.6%), which is the axis mismatch itself.
  - ***`ec30-am2` (merge-tap, winner `native`): FAIL, and it is REAL.***
    **0.00/s at every stage, −100.0%**, 4 checkpoints at a 2-game-hour
    warmup; kit-clean, fluid-clean, 1991/1991 ghosts revived; census 150
    `full_output` + 20 `item_ingredient_shortage` — a jam, not a starve.
    The layout ships **3 `belt-dead-end` Errors**. Not an instrument
    artifact and not a warmup artifact. Full entry and the
    fixture-confusion it clears up (the "#644-era ec30 ≈99.4%" anchor is a
    different fixture AND a different artifact) in `docs/status.md`. **This
    is a Phase-0 finding, not a Phase-0 fix**: nothing here changes the
    layout.
  - *Structural consequence for Phase 2a, recorded now: the corpus's
    `default` cells are not all directly sim-anchorable. Any fixture whose
    chain contains a force-boosted recipe (plastic-bar, processing-unit)
    plans at 0 productivity while the game realizes +10%, so the harness
    kit-fails the run; the anchor has to be taken on a declared-axis
    re-export, which is a slightly different artifact. A major divergence
    on such a fixture must say which artifact its sim anchored.*
- *2026-08-21 — **#694 review round 1 adjudicated** (5 findings, 3 nits).
  Absorbed: (a) `bless` could rotate the baseline onto a DIFFERENT zone
  cache and rewrite the recorded hash in the same move, leaving 160
  unreproducible rows with nothing to notice it by — it now refuses unless
  the pin matches (escape hatch `bless-repin`), proven by an executed
  discrimination check that exits 101 and leaves the file untouched. Only
  half of that finding was taken: refusing to bless on differing VERDICTS
  would make bless unusable, since re-taking after an intentional engine
  change is what it is for. (b) `check`'s provenance warning now PREFIXES
  the failure instead of trailing it as a note the reader met before the
  diff list. (c) the `Cell::status` doc listed a `refused` no branch emits
  — the records-outlive-state class again; it now enumerates the five real
  statuses and marks `decided-then-refused` as unreached (zero cells).
  Refuted with receipts: (d) "the corpus needs a CI gate" — a gate on this
  baseline today asserts only "production has not changed", which every
  engine PR legitimately falsifies; the gate that means something is
  Phase 2a's shadow-vs-production check, which Verification plan item 2
  already commits to. The instrument is gated by the three non-ignored
  contract tests; the data is guarded by the next phase re-taking it. The
  reasoning is now in the module doc so it reads as a decision.
  (e) "the new contract helpers should use the corpus's nesting-robust
  extractor" — the brittleness IS the pin: `assert_scoreboard_contract`
  must fail when a selection nests, `outer_selection` must survive it
  across 160 cells. Making the pin robust deletes its detection.*
- *2026-08-21 — **#694 review round 2 adjudicated** (7 findings, 4 nits;
  none contradicted round 1). Absorbed: (a) the committed provenance hash
  was `DefaultHasher`, whose algorithm is documented as unstable across
  Rust releases — a value that is committed and compared against forever
  cannot be that, so it is SHA-256 now. The re-bless is its own receipt:
  **only the hash line changed, all 160 cells byte-identical**, a fourth
  independent reproduction, taken through `bless-repin` because the plain
  path correctly refused. (b) a `null` prior hash satisfied the new bless
  guard through an `is_none()` disjunct, so one repin with no cache would
  have disarmed it permanently. (c) `outer_selection` now asserts the
  winner is among the seven rows it extracted — the timing guard catches
  an out-of-order terminal but not an in-order one naming another block's
  candidate, and that cell would look self-consistent and be re-verified
  green forever because `check` reads through the same extractor.
  (d) the contract tests' unpinned cache posture is reconciled by
  measurement rather than assertion: CI pins at the job level, and all
  three pass under both caches (1.10s pinned, 2.66s unpinned).
  Adjudicated as designed: `no-winner`/`no-selection` do not consult the
  build result because both IMPLY it errored; `e2e-harness` cannot assert
  its delta automatically (`run_e2e_inner` is private to another test
  binary) so the hand-verified receipt is written into the doc instead.
  Refuted: the 180s contract-test timeouts are not a flake source (the
  three run in 1.06s, ~170× headroom) and the "builds the jammed ec30
  layout" concern conflates a build with a simulated factory — the layout
  builds in milliseconds, it is only the SIM that deadlocks.*
- *2026-08-21 — **#694 review round 3 adjudicated** (8 findings, 3 nits;
  closing round). Absorbed: (a) `check` passed GREEN on a provenance
  mismatch whenever the cells happened to match — the same
  "compared-nothing reads as clean" shape #693 closed, and now a hard
  failure with a `check-any-cache` escape. Its discrimination check came
  free from (b). (b) the provenance hash covered only the disk pin, not
  the compiled-in `EMBEDDED_CACHE` that `zone_cache.rs` merges alongside
  it, so a different embedded payload would have reported an identical
  hash while consulting different zones; widening it re-blessed to **160
  byte-identical cells, hash line only — a fifth reproduction**.
  (c) a corrupt committed baseline bypassed the bless guard entirely,
  because `.ok().and_then(parse.ok())` collapsed "missing" into
  "truncated" — the same disjunct-disarms-the-guard class as round 2, one
  path over. (d) the two new stage pins now state that a red is EXPECTED
  when the ec30 jam is fixed, and that ac5's pairwise win is a knife-edge
  to re-take and sim-check rather than revert toward. (e) the
  `e2e-harness` label names an OPTION SET, not a cell the suite runs.
  Refuted: `outcome_of`'s `&String == &str` compiles via the blanket
  `PartialEq<&B> for &A` impl over `String: PartialEq<str>`, and an
  always-false comparison is structurally unhidable there because the
  lookup ends in a `panic!`, not a default.*
