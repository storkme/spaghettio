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

## Phase 1b specification: `SelectionPolicy`

*(Authored 2026-08-21 by the campaign lead against the merged Phase-0
instruments; the implementation track W2b builds exactly this, and its
acceptance bar is §"The Phase-1b acceptance harness" below. Line
anchors are as-of `1e97cc67`.)*

### The reframe: one measurement, three comparators

Reading the mechanisms at source
(`decomposition_search.rs:757-929, 1425-1873`) dissolves the "three
verdict mechanisms" into a cleaner factorization the RFC's Motivation
could not yet see: there is **one underlying measurement** and **three
comparators consuming different projections of it**:

- `classify_errors` (:786), `count_issues` (:852), and the
  `clean_flags` closure (:1585) each independently run
  `validate::validate` on the same layout and project the same issue
  list three different ways (kind classes / severity-channel counts /
  clean-bit + warning key). Today a contested candidate is validated
  up to three times per selection.
- The comparators are: the **quality-key lexicograph** (structural
  dominates weighted-functional; merge-tap scope), the
  **component-wise floor** (non-lexicographic across three channels,
  ties to incumbent; pairwise scope), and the **tiered rank**
  (clean tier ordered warnings-asc-then-score on the refusal path /
  score-desc on the success path; then accepted-by-score; then
  first-produced).

v2 therefore measures **once** and derives all projections.

### `IssueProfile` — the unified per-candidate measurement

One struct, computed from a single `validate()` call plus the layout
and score:

- `errors: usize` — `Severity::Error` count.
- `selection_warnings: usize` — `selection_warning_count` semantics:
  warnings minus the policy's excluded categories.
- `layout_warnings: usize` — `LayoutResult.warnings.len()`, the second
  channel `validate()` never sees (the #462 lesson).
- `kinds: {contamination, starvation, structural}` — the Error-only
  kind classification, with the category→kind map as policy data (the
  match at :799-806 becomes a table).
- `accepted: bool` + `accepted_reason` — the acceptance gates (today
  exactly one: `missing-balancer-template > 0` disqualifies).
- `score: f64` + components — `score_layout` unchanged.

A `None` field is a **gap, never a zero** (the scoreboard's rule):
stages whose inputs are absent skip, exactly as today's lazy sites do.

### `SelectionPolicy` — the data

```
SelectionPolicy {
  excluded_warning_categories: BTreeSet<String>,   // today: belt-detour ONLY — the two #632 B6
                                                   // demotions left the set by DELETION (#684);
                                                   // decomposition_search.rs:870-871 still carries
                                                   // the stale two-demotions prose (pre-existing;
                                                   // W2b sweeps it)
  error_kind_classes: BTreeMap<String, Kind>,      // the :799 match, as a table
  acceptance_gates: Vec<AcceptanceGate>,           // today: MissingBalancerTemplate
  contamination_weight: usize,                     // KIND_CONTAMINATION_WEIGHT
  firewalls: Vec<Firewall>,                        // named, with receipt strings — the #519
                                                   // exemption lives here as the record of WHY
                                                   // excluded_warning_categories contains what it does
  program: SelectionProgram,                       // §below
  producers: Vec<ProducerRegistration>,            // §below
}
```

### `SelectionProgram` — the five stages as data

An ordered stage list; each stage names its `SelectionStage` tag, its
scope, its comparator, and its chain behavior. Today's program:

1. **MergeTap** — scope: merge-tap vs incumbent, gated on merge-tap
   having produced (which its own gate restricts to Pooled +
   incumbent-unaccepted). Comparator: quality-key lexicograph, ties →
   incumbent. **Non-shadowing rule** (the #474 lesson, :1816-1845): an
   incumbent win here does NOT short-circuit past stage 2 — only a
   merge-tap win terminates the chain at this stage.
2. **ScopedPairwise** — scope: each scoped candidate (DI, horizontal)
   vs incumbent, each gated on incumbent-produced (incumbent refusal
   routes them to stage 3 instead — see AdmissionRule). Comparator:
   component-wise floor; per-producer `equal_and_denser` flag (DI yes,
   horizontal no — RFC-060's measured call); unaccepted never
   displaces accepted. Two winners resolve pairwise by the same floor,
   ties → earlier registration.
3. **BestErrorFree** — scope: the admitted slice; requires
   `accepted && errors == 0`. Order: refusal path (selection+layout
   warnings asc, score desc, index asc) / success path (score desc,
   index asc) — the RFC-060 scoping at :1795-1807.
4. **BestAccepted** — `accepted`, score desc, earliest index.
5. **FirstProduced** — the degraded fallback. **This stage is the ec30
   trap measured by #694** (an error-laden best ships rather than a
   refusal); v2 reproduces it bit-for-bit under parity, and any change
   to it is Phase-3 calibration work, not Phase-1/2 migration work.

**AdmissionRule** — `ranking_len` (:1705) becomes named data: *scoped
producers enter the generic stages iff the incumbent produced
nothing*. It is the single enforcement point for DI/horizontal
never-worse defaulting (the `tier2_electronic_circuit` regression
class) and the spec preserves it as such.

### `ProducerRegistration` — per-producer policy

```
ProducerRegistration {
  name, producer,                  // DecompositionCandidate, or PlanProducer for k1
  gate: GatePredicate,             // today's try_* predicates (cost gating), as data
  refuse_on_error: bool,           // DI/horizontal/cells true; native/k1/split/merge-tap FALSE.
                                   // PRECISE SEMANTICS (this is a PRODUCE-TIME gate, never a
                                   // stage/win gate): when true, a produced layout carrying any
                                   // Severity::Error is DISCARDED before any stage sees it —
                                   // the producer records a refusal (v2 retains the full issue
                                   // list, closing Phase-0b oracle gap (d); v1 stringifies it).
                                   // That is exactly v1's self-validation in produce()
                                   // (decomposition_search.rs:191-211 for DI: "Errors refuse;
                                   // warnings pass"). When false, an error-laden layout stays
                                   // in play and can win via stage 4/5 — the ec30 witness.
                                   // Preserving WHICH producers carry the gate is REQUIRED
                                   // for parity.
  equal_and_denser: bool,          // DI true, horizontal false
  scoped: bool,                    // DI/horizontal true → AdmissionRule applies
}
```

The `k1-shape-fix` closure becomes a `PlanProducer` — a registration
constructed *with* its `build_k1_enrollment_plan` output — so it
registers like every other producer without widening the trait.

**K70-1 boundary, stated precisely**: producer-*keyed configuration*
(the fields above) is policy data and does not trip K70-1.
Producer-*name-conditioned branches inside stage logic* do. The test
is mechanical: stage code may read registration fields, never
registration names.

### Uniform loop stages

`catch_unwind` becomes uniform (7/7; today 5/7 — native and k1 are
unprotected, so a native panic aborts the solve today but would become
an all-refused error under v2). This is a deliberate, strictly
degradation-softening divergence on a path the corpus cannot witness
(no corpus fixture panics); it is documented here rather than hidden,
and the parity gate cannot and need not cover it. Producer gates and
the sink detach/replay discipline (winner-only event replay, losers
truncated) move into the loop unchanged.

### Validation-once and laziness

v2 may validate each produced candidate exactly once (eager profile)
or preserve today's lazy per-stage validation — implementer's choice,
adjudicated by K70-3's isolated-run comparator. `validate()` is
deterministic, so eager vs lazy cannot change outcomes, only cost.
The lazy skips that exist today (merge-tap- or scoped-decided solves
skip `clean_flags` entirely; single-layout solves skip) are the
cost-relevant cases to measure.

### `verdict.rs` extension and RFC-068 compat

`Verdict`'s category-count model gains a severity dimension
(per-category `{errors, warnings}` instead of one fungible count);
`Policy` gains the excluded-categories set. `Policy::fold()` keeps its
current meaning so `celldb_template.rs` / `candidate_runner.rs` tests
stay green unmodified (the celldb-harness-green obligation). The
severity-blind path remains available to RFC-068 until its own
campaign migrates.

### The Phase-1b acceptance harness

`policy_replay` (a test, non-ignored where cheap): **one live corpus
run, two consumers.** The committed #694 baseline deliberately stores
only `(status, winner, stage, outcomes)` per cell — the per-candidate
profiles exist only in the live `SelectionCandidateEvaluated` events
and are never persisted (pinning them would pin gaps as facts, per the
Phase-0b principle). So the harness runs the #694 corpus once with the
scoreboard enabled; v1 decides each cell as normal, the harness
captures that cell's emitted per-candidate profiles in-process, feeds
them through the v2 comparator/program, and requires v2's winner
**and** stage to match both the live v1 decision and the committed
baseline on all 140 decided cells. No second layout pass per cell —
the "replay" is over captured profiles, not re-produced layouts.
Profile gaps (fields the recorded mechanisms never computed) must be
handled by stage-skip, exactly as the oracle records them. This is the
offline precursor to K70-1; the live shadow (Phase 2a) then runs the
same program against freshly produced layouts.

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
- *2026-08-21 — **#694 review round 4 adjudicated** (5 findings; the pass
  was DEGRADED — it leaked its own prompt template and cited four symbols
  that grep to zero — but two findings were real and one was the best of
  the review). Absorbed: (a) the provenance hash missed a THIRD zone
  source, the legacy `sat-zones.jsonl` `zone_cache.rs` still reads next to
  the pin; a `.jsonl` appearing would have changed which zones replay
  while the committed hash stayed identical. Re-bless: **zero diff, not
  even the hash line** (none exists here) — a sixth identical
  reproduction. (b) `bless` conflated a NotFound baseline with an
  unreadable one. **The generalisation is the durable part, and it is now
  in the doc: a provenance hash is only worth what its source list is,
  and a source list is a thing that goes stale** — two rounds found two
  missing sources, so all three are enumerated against `zone_cache.rs`
  rather than summarised. Refuted: the "no terminal/row after the chosen
  terminal" concern (the index comes from `rposition`, and `last_row < t`
  has covered the row half since the first hardening pass); the CI-gate
  finding for the third time (answer unchanged); and the objection that
  the scoped pin's message admits it may fail on a correct change — that
  wording exists because round 3 asked for it. **Campaign note on the
  instrument**: four rounds, and the two genuinely load-bearing findings
  (SHA-256 stability, the missing zone sources) were both about the
  PROVENANCE machinery rather than the corpus — the cells themselves
  never moved across six reproductions.*
- *2026-08-21 — **#694 review round 5 adjudicated** (4 findings, 3 nits),
  and the headline is that **round 3 was absorbed too eagerly**:
  `EMBEDDED_CACHE` is merged under `#[cfg(target_arch = "wasm32")]` only
  (`zone_cache.rs:1404-1412`), so the fix that added it to the provenance
  hash was pinning a file the native corpus never reads — a wasm-only edit
  would have hard-failed every native `check` and refused every plain
  `bless` against a byte-identical native zone set. Dropped; the hash now
  covers exactly what `load_existing_jsonl` reads. Re-bless returned the
  hash to its pin-only value and held all 160 cells for the **seventh**
  time. Also absorbed: `None == None` passed `check` green (the
  None-vs-None pair survived round 3's Some-vs-None fix, so a null-hash
  baseline could green-check 160 unreproducible rows forever); and the
  junction-seed census's `bucket_sum` assert still ran before the dumps,
  excused as a tautology — which is the "cannot happen" reasoning that
  file distrusts everywhere else. Both its asserts are now last.
  **The generalisation, which is the part worth keeping**: two rounds got
  the hash's source list wrong in OPPOSITE directions — first too narrow,
  then too wide — so the doc no longer states a list as fact, it states
  how to re-derive one. `Which sources does this consult?` is a `#[cfg]`
  question, and it cannot be answered by reading a function name. Every
  Phase-1/2 instrument that hashes inputs inherits that.*
- *2026-08-21 — **#694 review round 6 adjudicated** (8 findings, 3 nits;
  closing round — seven of the eight were re-raises already answered).
  One absorbed, and it is a **direct warning to Phase 2a**:
  `zone_cache::lookup_table()` is a process-wide `OnceLock` mutated in
  memory as solves append zones, and all 160 cells run in one process, so
  cell N is solved against a map cells 1..N-1 have grown. The committed
  hash describes the run's STARTING disk state — which is what makes the
  full sweep reproducible (seven byte-identical runs) — but says nothing
  about what an individual cell saw. **Re-running one fixture in isolation
  is therefore not guaranteed to reproduce its committed cell**, which is
  exactly the first thing anyone will do on a divergence. Adjudicate by
  re-running the whole corpus and reading the cell out of it. New argument
  answered on merits: relaxing the two stage pins' winner assertions would
  not reduce brittleness, because on both fixtures winner and stage move
  together — if merge-tap stops gating its stage goes too, and if
  horizontal stops winning the stage becomes best-error-free. There is no
  looser pin that still detects a mis-tagged stage. **Review close-out:
  six rounds, ~35 findings; the load-bearing ones were all about the
  provenance machinery and the instrument's failure messages, and the 160
  cells never moved once across seven reproductions.**
- *2026-08-21 — **Phase 1b specification authored** (§"Phase 1b
  specification" above), by the session lead per the W2b design-duty
  decision on #689. The load-bearing reframe: the "three verdict
  mechanisms" factor into ONE measurement (`IssueProfile`) consumed by
  three comparators — today a contested candidate is validated up to
  three times per selection, and v2 measures once. Calls made in the
  spec: per-producer `refuse_on_error` stays asymmetric (native/k1/
  split/merge-tap false — REQUIRED for parity; ec30 is the live
  witness); `catch_unwind` unifies to 7/7 as a documented
  corpus-invisible divergence; the K70-1 boundary is "stage code may
  read registration fields, never registration names"; the FirstProduced
  stage's ships-error-laden-best behavior is explicitly preserved under
  parity (its fix is Phase-3 calibration, not migration). Acceptance =
  `policy_replay` reproducing winner+stage on all 140 decided #694
  cells from recorded profiles.*
- *2026-08-21 — #695 review round 1 absorbed (2 major, 1 minor — all
  three improved the spec; one counter-model refuted with a receipt):
  (a) the excluded-categories "today" comment was stale — the #632 B6
  demotions left the set by DELETION (#684); belt-detour is the whole
  set, and decomposition_search.rs still carries the stale prose (W2b
  sweeps it); (b) `refuse_on_error` now has precise produce-time
  semantics — the reviewer's counter-model ("error-laden DI maxes out
  at stage 4/5 like native") is FALSE at source (DI's produce()
  refuses on any Error at :191-211, so it has no outcome and reaches
  no stage), but the under-definition was real and is fixed; (c) the
  acceptance harness input was mis-specified — the baseline stores no
  profiles, so policy_replay is now "one live corpus run, two
  consumers" (v1 decides, v2 replays the captured in-process
  profiles; no second layout pass).*
- *2026-08-21 — **Phase 1a landed** (#689 track W2a): the doc-only legend at
  `bus/layout.rs:79-111` is promoted to three real types — `UserConstraints`,
  `SearchAxes`, `EngineTuning` — closing the "`LayoutOptions` split" Design
  bullet. **Composition shape: facade views over the flat struct, not
  nesting.** The alternative (`LayoutOptions { constraints: UserConstraints,
  axes: SearchAxes, .. }`) was rejected on a measured count, not a style
  preference: ~238 flat-field reads and ~80 flat struct-literal
  constructions of `LayoutOptions` exist across the workspace today (mostly
  `tests/e2e.rs`), and `LayoutOptions` carries no serde/tsify derive to
  begin with — the WASM boundary never serializes it,
  `wasm-bindings::layout_options` builds one from primitive
  `Option<String>`/`Option<u8>` params field-by-field. Nesting would rename
  hundreds of sites for zero wire-format benefit and blow the PR's line
  budget for no payoff the RFC asked for. Chose instead: fields stay flat
  on `LayoutOptions` (unchanged, zero renames anywhere); `UserConstraints`/
  `SearchAxes`/`EngineTuning` are owned-copy VIEWS obtained via
  `LayoutOptions::constraints()`/`axes()`/`engine_tuning()`, plus a
  `LayoutOptions::from_groups(constraints, axes, engine_tuning)`
  constructor. Net +267/-10 lines, one file.
  **Classification calls**: `planning_duty` (previously unclassified) is
  neither a user constraint (the caller doesn't supply it, the engine picks
  it) nor a search axis (the search loop never varies it) — it gets its own
  tiny `EngineTuning` group rather than being folded into either existing
  one or left doc-only. `research_productivity` (previously unclassified)
  is NOT a preference — it describes the player's save (researched
  productivity bonuses), i.e. world state — but it groups with
  `UserConstraints` because it is exogenous caller input exactly like
  `max_belt_tier`, never a value the engine searches or tunes; a fourth
  group for one world-state field was judged not worth the ceremony.
  **Structural guard**: each group's `Default` impl is manual, not derived
  — in particular `SearchAxes::default().cell_composition` is `Candidate`,
  matching the engine default, where `CellComposition`'s own `#[default]`
  is `Off`. ~~`from_groups` therefore cannot reproduce the `cell_composition`
  fossil~~ **RETRACTED, #696 round 2 (this entry originally overclaimed the
  same thing round 1 below already fixed in the code legend — the log
  itself was left contradicting its own later entry, a records-outlive-
  their-state miss round 2 caught):** the guard only covers the ATOMIC path
  (`SearchAxes::default()` called wholesale). A partial literal of the
  group struct itself (`SearchAxes { cell_composition: Default::default(),
  ..SearchAxes::default() }`) still resolves to `Off` — the identical trap,
  one level down, because `SearchAxes`'s fields are `pub`. See the round-2
  entry below for the corrected claim; the code legend
  (`bus/layout.rs`) has always been the ground truth here since round 1's
  fix, only this entry's earlier wording was stale.
  **What it does not prevent**: the ~80 existing flat struct-literal call
  sites — including both known `run_e2e` fossils — are completely
  unchanged and exactly as fossil-prone as before (fixing those is #689
  track W2c, sequenced after this one), and nothing stops new code from
  writing a flat (or group-level partial) literal instead of calling
  `from_groups`. A `layout_options_group_defaults_match_facade` test pins
  the three group defaults against `LayoutOptions::default()` (executed
  discrimination check: reverting `SearchAxes::default()`'s
  `cell_composition` to `CellComposition::default()` made it fail, naming
  exactly `cell_composition: Candidate` vs `Off`; restored and reverified
  green). ~~A `layout_options_from_groups_round_trips` test pins
  `from_groups` against a non-default value on every group.~~ **STALE,
  #696 round 2: that test was replaced in round 1 by
  `layout_options_constraints_axes_and_from_groups_match_explicit_expectations`
  (see the round-1 entry below for why) — this entry named the retired
  test and was never updated.**
  **Verification**: `cargo test --manifest-path crates/core/Cargo.toml`
  full suite green (no `--no-fail-fast` failures); `cargo clippy -p
  spaghettio_core -- -D warnings` (the exact pre-commit invocation) clean,
  and `--all-targets` clippy warning count unchanged at 42 before/after (no
  new warnings anywhere in the crate, not just the changed file). WASM
  rebuilt via `wasm-pack build crates/wasm-bindings --target web --out-dir`
  twice (stashed/unstashed to get a clean before/after pair): the generated
  `spaghettio_wasm.d.ts`, `spaghettio_wasm.js`, and `package.json` are
  **byte-identical** (`diff` exit 0 on all three); only the `.wasm` binary
  itself differs (expected — new dead code compiled in changes layout/debug
  info, not the JS/TS surface). `web`'s `npm run build` (`tsc --noEmit &&
  vite build`) and `npm run test` (vitest, 41 tests) both green with zero
  web-side edits. The #694 parity-baseline check
  (`SPAGHETTIO_ZONE_CACHE_PATH` pinned to `crates/core/data/sat-zones-ci.bin`,
  `SPAGHETTIO_PARITY_CORPUS=check`) passed: **160/160 cells matched the
  committed baseline exactly** (`test parity_corpus ... ok`, 268s) — the
  printed option-set divergences (10 major, 12 minor) are the
  ALREADY-COMMITTED Phase-0c facts about the option-set axis, not a new
  divergence this PR introduced; the pin file was untouched by the
  read-only `check` mode (confirmed via `git status`), so no restore step
  was needed.
- *2026-08-21 — **#696 (W2a) review round 1 adjudicated** (7 findings, all
  minor, absorbed in full — none refuted). Two were substantive, not just
  wording: (a) the field legend's "the guard means naming a field twice can
  no longer select the wrong value" OVERCLAIMED — it is true only for the
  ATOMIC path (`SearchAxes::default()` called wholesale); a partial literal
  of the GROUP struct itself (`SearchAxes { cell_composition:
  Default::default(), ..SearchAxes::default() }`) reproduces the exact same
  trap one level down, because `UserConstraints`/`SearchAxes`'s fields are
  `pub`. Fixed by rewriting the claim: the guard relocates the trap's
  easiest entry point and gives new call sites a correct atomic default to
  reach for instead, it does not remove the trap's shape from the language.
  (b) the round-trip test (`layout_options_from_groups_round_trips`) had a
  structural blind spot the review named precisely: comparing
  `rebuilt.axes() == original.axes()` calls the SAME (possibly buggy)
  accessor on both sides, so a "wrong-source-field" bug that is consistently
  wrong (e.g. `axes()` reading `merge_tap` from `self.horizontal_candidate`)
  produces identical wrong values on both sides and passes. Replaced with
  `layout_options_constraints_axes_and_from_groups_match_explicit_expectations`,
  which checks every accessor against a HAND-WRITTEN expected struct instead
  of re-deriving one, and checks `from_groups`'s rebuild field-by-field
  against the original (not via the accessors again). **Executed
  discrimination check**: injected exactly the bug above
  (`merge_tap: self.horizontal_candidate` in `axes()`); the new test failed
  immediately, naming `merge_tap: false` (actual) vs `true` (expected) in
  the `SearchAxes` comparison — caught before the round-trip stage was even
  reached. Restored, reverified green. Minor fixes absorbed alongside: the
  legend's "~320... most of them `tests/e2e.rs` struct literals" wording
  contradicted the log's own "~238 reads + ~80 constructions" breakdown
  (reads dominate, not literals) — reworded to state both counts and that
  reads live outside `tests/e2e.rs` too; `from_groups`'s doc now states it
  takes its groups **by value** (moves `max_belt_tier`'s `String` and
  `research_productivity`'s `BTreeMap`), distinct from the `&self`
  accessors' owned-copy contract; the legend now states explicitly that
  `from_groups`/`constraints`/`axes`/`engine_tuning` have **zero production
  callers as of this PR** — pure scaffolding for Phase 1b / #689 W2c: the
  ~80 existing flat-literal sites, including both known `run_e2e` fossils,
  are completely untouched; the defaults-match test's doc now states its
  own stated limitation (catches divergence between the two `Default`
  impls, not wrongness of a value both share). One finding, on the
  round-trip test's original `stacking: 2` colliding with
  `DEFAULT_INSERTER_CAPACITY`'s own value (also 2), is subsumed by the
  test's replacement — the new test asserts `stacking` (3) is distinct from
  `DEFAULT_INSERTER_CAPACITY` and gives the two `u8` fields
  (`stacking`/`inserter_capacity`) different values (3 vs 5) by
  construction. The bool-triple pigeonhole (three `bool` fields, two
  possible values, so some pair must still collide) is inherent to the type
  and documented as an acknowledged residual gap rather than chased further.
  Re-verified after fixes: full `cargo test` green, `cargo clippy -p
  spaghettio_core -- -D warnings` clean, all PR CI checks (`rust`,
  `rust-clippy`, `web`, `second-opinion`, `deploy-preview`,
  `workflow-guard`) green on the prior commit.
- *2026-08-21 — **W2c: both `run_e2e` fossils killed; the `run_e2e*`
  HARNESS now runs production defaults.** Scoped deliberately (#699 review
  round 6 flagged the first wording, "the suite now runs production
  defaults", as overstating what the body itself measures): 15 other tests
  in `tests/e2e.rs` still build their own `LayoutOptions` with
  `cell_composition: Off` / `inserter_capacity: 0` and are pinned, not
  migrated. Every fixture routed through `run_e2e*` — which is every tier
  and stress fixture — does run production defaults.
  `run_e2e_inner` builds its `LayoutOptions` through
  `LayoutOptions::from_groups` (the #696 scaffolding's first production
  caller) via a new `harness_options(HarnessOptions { .. })` helper, so
  every field the harness does not deliberately override is the engine's
  own default. `cell_composition` `Off` → `Candidate`; `inserter_capacity`
  `0` → `DEFAULT_INSERTER_CAPACITY` (2). `rfc060_sim_export`'s hand-copied
  "mirror run_e2e_inner exactly" literal — which had ALREADY drifted, keeping
  `inserter_capacity: 0` but not the cells fossil, so it matched neither the
  harness nor production — now calls the same helper.

  **Blast radius, measured by A/B rather than assumed.** Three full
  `--test e2e` runs under the pinned CI zone cache: cells-fossil-killed
  only, capacity-fossil-killed only, and both. The two effects are exactly
  additive (the cap-only and both-fossils fingerprints differ in one
  fixture, the one cells moves).
  * **Capacity `0` → `2`** moves 6 of 8 golden hashes and 8 of 8 stress
    hashes — every fixture with an inserter-fed row — while changing no
    winner and no deciding stage **on any (fixture, machine) pair the
    suite runs**. That scoping is load-bearing and the first draft of
    this entry dropped it (#699 review round 4): the corpus DOES record
    two capacity-attributable winner changes, both at am3, on fixtures
    the suite invokes at am1/am2. "Additive" likewise means the two
    fossils' effects on the SUITE compose without interacting, not that
    either is inert. The two fluid-target goldens
    (`tier3_sulfuric_acid`, `tier3_heavy_oil_cracking`) are the negative
    control: byte-identical under both arms.
  * **Cells `Off` → `Candidate`** moves exactly ONE layout in the whole
    suite: `tier1_iron_gear_wheel_20s`, where `cell-composed` now wins the
    outer selection (`SelectionDecided { winner: cell-composed, stage:
    best-error-free }`, score 0.1129 vs native's 0.1081) — 148 entities /
    47×8 → 105 / 38×14, same 12.3 % density, zero validation issues on
    both. Everywhere else the cell-composed candidate runs, is scored, and
    loses, exactly as RFC-051's "strictly additive" flip claimed.

  **Adjudication against #694.** The corpus's `e2e-harness` → `default`
  delta is 7 verdict-differing rows of 32: two winner changes
  (`e2e_tier2_electronic_circuit_20s_from_ore`/am3 and
  `tier2_ec_am1_10_ore`/am3, both native → direct-insertion, both
  capacity-attributable — they also differ `cells-off` vs `e2e-harness`)
  and five stage-only changes (`best-accepted` → `best-error-free`, winner
  `native` throughout: `tier1_gear_am1` ×3 tiers, `tier3_plastic_cp_5`,
  `e2e_tier3_plastic_bar_from_crude`), which are the cell-composed
  candidate giving the `best-error-free` stage something to decide on
  instead of falling through. **Both winner changes are at am3, a tier the
  suite never invokes for those fixtures**, so the corpus predicts zero
  winner changes among the (fixture, machine) pairs `e2e.rs` actually runs
  — and zero materialized. 6 golden re-blesses, 1 warning-pin re-bless, 2
  test re-pins; every one traced to the prediction or to the corpus hole
  below. Prediction-match rate on corpus-covered fixtures: 5/5 (the four
  golden-pinned fixtures with an exact corpus row, plus tier5's pin).

  **FINDING — corpus hole.** The one winner change the flip produces,
  `tier1_iron_gear_wheel_20s` (gear @ 20/s, am2, from iron-plate), has NO
  row in the #694 corpus: the corpus carries gear @ 10/s (`tier1_gear_am1`,
  `e2e_tier1_iron_gear_wheel_from_ore`) and nothing at 20/s. Its nearest
  covered neighbours all predict `native`, so the corpus is not falsified
  — it is silent, and the one cell where the campaign's headline flip
  changes a shipped artifact is the cell it does not cover. Recommendation
  for W3a: add gear@20/am2 to the corpus before the shadow loop gates on
  it.

  **FINDING — and the corpus hole was hiding a real defect (#700).** The
  first version of this entry adjudicated that re-bless on validator
  evidence alone: both arms error- and warning-free, the winner smaller
  and denser. #699's review round 1 refused that reasoning — correctly,
  per the verification protocol: the validator cannot CLEAR a layout. So
  the layout was metered. Three arms, `measure(108_000, 216_000)`, no
  notes:

  | arm | cells | capacity | entities | validator | meter |
  |---|---|---|---|---|---|
  | production today | `Candidate` | 2 | 105, 38×14 | 0 issues | **15.0 / 20.0 — 75 %** |
  | cells disabled | `Off` | 2 | 148, 47×8 | 0 issues | 21.0 / 20.0 |
  | pre-W2c golden | `Off` | 0 | 148, 47×8 | 0 issues | 21.0 / 20.0 |

  The cell-composed winner **under-delivers by 25 %**, validator-clean,
  and production has shipped it since RFC-051 Phase B flipped
  `cell_composition` on 2026-07-22.

  **CORRECTION, and it makes the finding better rather than worse
  (#699 review round 4 prompted the check).** The first draft of this
  entry presented the deficit as newly discovered. It was not: **W1a
  found it on day one.** #693's own table reports `gear20-am2-plate` at
  20.000 planned / 15.000 produced / **−25.0 % BELOW PLAN**, and the
  committed tripwire baseline
  (`crates/meter/tests/e2e_tripwire_baseline.json`) carries the row ARMED
  at `entities: 105, deficit_pct: -25.0, converged: true`. The reading
  above reproduces that baseline exactly, which is corroboration, not
  novelty. There is therefore no "prose-only guard" gap either: `check`
  fails on that row worsening, today, without this PR.

  **What W2c actually adds is the attribution, and it is the sharper
  fact.** The tripwire's row says 105 entities. The e2e suite's
  `tier1_iron_gear_wheel_20s` golden, before this PR, pinned 148. *Same
  item, same rate, same machine, same inputs — two different layouts,
  under two instruments, neither able to see the other.* The tripwire
  built `LayoutOptions::default()` (production: cell-composed, −25 %);
  the e2e fixture built the fossil's options (native, +5 %). Nobody
  joined "the meter says gear@20 is 25 % down" to "the regression suite
  says gear@20 is fine" because they were not talking about the same
  artifact. **That** is what the fossil cost, and it is exactly the class
  of thing a campaign about one selection loop exists to remove. Filed as
  #700; a `#[ignore]`d exporter (`w2c_gear20_meter_export`) is committed
  so the three arms are re-measurable side by side, which is what makes
  the divergence checkable rather than narrated.

  The golden re-bless stands — a golden records what the engine produces,
  and this is what it produces — but the fixture's comment now says so,
  including that its `assert_produces(…, 20.0)` passes on a 15/s layout
  because it reads a static estimate, and names all five coupled
  artifacts a #700 fix has to move (the tripwire row among them).

  **SECOND FINDING — the stage-5 policy this exposes.** On
  `tier1_iron_gear_wheel_20s` the outer board records native with
  `layout_warnings: 0` and cell-composed with `layout_warnings: 1`, and
  `best-error-free` picks cell-composed anyway, on score. The stage
  discriminates on ERRORS and then ranks on score; warnings do not
  participate. That is a real property of today's precedence, surfaced
  (not introduced) by this PR, and it is squarely W2b/`SelectionPolicy`
  territory — recorded here rather than acted on.

  **Two tests re-pinned, not overridden.** No test got a deliberate
  old-behaviour override — the two that failed were asserting the fossil
  itself, and pinning them to `cells-off` would have preserved the fossil
  in precisely the two tests that describe the candidate set.
  `decomposition_search_native_candidate_fires_trace_events` asserted
  "exactly one `DecompositionCandidateScored` under Phase 0"; production
  has run more than one candidate since RFC-051 Phase B / RFC-053, so the
  assertion was true only of the fossilized set. It now pins the set #694
  records for this exact fixture — `native` accepted, `cell-composed`
  present, `native` winning the outer selection — which makes it a
  behavioural tripwire on re-fossilization as well as a smoke test.
  `decomposition_search_picks_native_on_clean_partitioned_case` asserted
  native was the ONLY candidate scored, reasoning "sequential dispatch,
  search exits early once native is accepted"; that reasoning describes a
  candidate set nothing ships. Its K-DS1-1 content — native wins the clean
  case, `size-split-2` is not paid for on it — survives verbatim.

  **New non-ignored guard**: `harness_options_are_engine_defaults` compares
  all three group views against the engine's own defaults and names both
  fossils individually. Discrimination check EXECUTED twice: re-spelling
  `cell_composition: Default::default()` inside the `SearchAxes` literal
  fails it (`cell_composition: Off` vs `Candidate`), and the capacity-arm
  A/B run failed it on `inserter_capacity: 0` vs `2` without being asked
  to. Both reverted and re-verified green.

  **tier5's warning pin (`input-rate-delivery 13 → 10`)** is the suite's
  only pin movement, and it is NOT a check going quiet: both issue lists
  were decoded from snapshots and diffed instance by instance. Ten rows
  reading "across 2 inserters" at capacity 0 read "across 1 inserter" at
  capacity 2 (one L2 hand replaces two L0 hands, so the row places fewer,
  fatter inserters); seven equivalent warnings re-appear at shifted
  coordinates. Every survivor still carries its own position and its own
  delivered-vs-needed pair. The fixture's known deficits are untouched, as
  is the meter's open #644 reading. Stress scoreboards moved the same
  direction (warnings fell on 3 of 8, entity counts fell on all 8, errors
  stayed 0; no category went to zero) — the baselines are `≤` ceilings, so
  none required a re-bless and none were loosened.

  **Corpus baseline NOT re-blessed, verified at source** (the W2c brief
  asked for this check explicitly): `parity_corpus.rs` builds its option
  sets as closures over `LayoutOptions::default()` in `OPTION_SETS`, never
  through `run_e2e`, so no cell can move when the harness changes. The
  `e2e-harness` column is now a HISTORICAL record rather than a live
  configuration; kept, not deleted, because it is the only record of what
  the fossilized suite decided — the file's doc comments were updated to
  say so instead of continuing to claim a present-tense fossil.

  **Pre-existing flake, made marginally worse, now fixed for the three
  fixtures that showed it.** `tier2_electronic_circuit_20s_from_ore`
  carried `#[ntest::timeout(10000)]` and tripped it at 10003 ms on the
  BASELINE (pre-change) parallel run and again at 10001 ms on a later
  post-change run, while passing SOLO in 0.63 s. `ntest::timeout` is
  wall-clock, so on a loaded box the budget, not the work, is binding —
  and restoring the cell-composed candidate adds a layout pass to every
  fixture. Raised to 60 s here, and 10 s → 30 s / 30 s → 60 s on the two
  re-pinned decomposition tests (#699 review round 2 predicted exactly
  this). **Residual, not fixed**: 60-odd other tests in the file carry the
  same 10 s wall-clock budget and are subject to the same hazard; a
  blanket raise is housekeeping for its own change, not a rider here.

  **#699 review round 1 absorbed** (7 findings, 1 union-major + 6 minor,
  all absorbed, none refuted — the major paid out immediately):
  * *"the re-bless rests on validator evidence and the docs say the
    validator cannot clear a layout; no meter/sim reading is recorded"* —
    correct, and the meter run it forced produced #700 above. The fix is
    the measurement plus a committed exporter, not prose.
  * *"the fossil PATTERN survives at ~a dozen other call sites in the same
    file, so 'both fossils killed' overstates coverage"* (3/3 passes) —
    correct and verified: **14 `cell_composition: Default::default()` and
    14 `inserter_capacity: 0` literals remain, across 15 distinct tests**
    (the horizontal-stack tier4/tier5 trio, the four `quality_*`, the six
    `stacking_*`, `research_l7_thins_output_inserters_s4`,
    `rfc061_allocation_probe_ac5`). None documents its value as
    deliberate; every one is a copy of `run_e2e_inner`'s old literal.
    Not migrated here — each carries its own pins, so flipping them is a
    second adjudication of this PR's size, not a rider on it. Instead the
    residual is now PINNED by a non-ignored test
    (`residual_fossil_literals_are_pinned`) that reads its own source and
    fails if either count moves, with the reduction direction spelled out.
    The claim itself is scoped to the `run_e2e` path everywhere it appears.
  * *"`chosen.last()` is an ordering-dependent oracle"* (2/3) — true. The
    ordering IS a stated contract (`trace.rs`'s
    `SelectionCandidateEvaluated` doc: terminals emitted adjacently at the
    very end, nested selections replayed inside the winner's events) and
    the #694 census reads the corpus by the same rule, but "stated" is not
    "checked". Both tests now corroborate across the two INDEPENDENT
    terminal emitters (`DecompositionChosen` and `SelectionDecided`) and
    the first also pins the deciding STAGE against #694's row, so a
    reordering has to break both emitters identically to slip through.
  * *"`harness_options_are_engine_defaults` is self-referential"* — partly:
    it compared against the group `Default` impls, a second hand-written
    copy of the engine defaults. Now compares against
    `LayoutOptions::default()`'s own views, which is the value the engine
    ships. (Group-vs-`LayoutOptions` agreement stays #696's
    `layout_options_group_defaults_match_facade`'s job.)
  * *"`rfc060_sim_export`'s artifacts silently change with no pin"* — true
    and worth a warning rather than a pin: the recorded K60-3 numbers were
    measured on capacity-0 artifacts, so post-2026-08-21 exports are not
    comparable to them. Stated in the exporter's doc.
  * The nit *"tier2's warning pin didn't move while its hash did"* is
    expected, not unexplained: warning pins record a per-category TALLY,
    and a geometry change that leaves the tally alone leaves the pin alone.
    Only tier5's tally moved.

  **#699 review round 2 absorbed** (8 findings: 2 major, 6 minor; 6
  absorbed with code, 2 refuted with receipts):
  * *(major, 3/3)* **"the suite now permanently endorses a known-broken
    artifact with no in-suite tripwire — the only guard is prose and an
    external issue number."** Correct, and the fix is an assertion, not
    more prose: `tier1_iron_gear_wheel_20s` now PINS its outer selection
    (`cell-composed` at `best-error-free`) with a message naming #700 and
    telling whoever moves it — including whoever FIXES #700 — to re-take
    the meter reading rather than re-bless on greenness. Neither existing
    assertion could say that: the golden hash's message asks whether the
    change was intentional, and `assert_produces` reads a static estimate
    the meter contradicts. Discrimination check executed: restoring the
    cells fossil fails it with `Some(("native", BestAccepted))` vs
    `Some(("cell-composed", BestErrorFree))`.
  * *(major, 2/3)* *"this cements a new baseline that should be treated as
    provisional until #700 lands"* — agreed, and now literally true: the
    pin above is what makes it provisional rather than silent.
  * *(minor, 1/3)* *"`harness_options_are_engine_defaults` only exercises
    the default path; the four fields the wrappers pass explicitly could
    go stale unseen."* **The best finding of the round** — the guard did
    have that hole. Two of the four are HARD LITERALS in every wrapper
    (`LayoutStrategy::Pooled`, `true`), which is the same fossil shape one
    level out; the guard now asserts the engine defaults still equal them.
    The other two (`row_layout`, `surplus_policy`) are passed as
    `::default()` and follow a flip on their own — stated, so the absence
    is not read as an oversight.
  * *(minor, 2/3)* *"the residual pin counts deliberate test vectors as
    fossils, and its 'lower the number' policy would push someone to
    convert a deliberate low-capacity behaviour test."* The specific
    example is shaky (`stacking_refuses_low_inserter_cap`'s refusal
    predicate names `max_inserter_tier`, not the capacity) but the hazard
    is real and unaudited. The doc now claims only what was checked — none
    carries a COMMENT explaining its value, no per-site audit was done —
    and the guidance is rewritten: decide per site whether the value is
    load-bearing, document it and lower the count if so, migrate if not,
    and name the site in the commit either way.
  * *(minor, 1/3)* *"both re-pinned tests gained a full extra layout pass
    under wall-clock timeouts on a box this PR itself records flaking at
    10003 ms."* Correct and cheap: 10 s → 30 s and 30 s → 60 s. Both run
    in ~0.02 s, so the raised budgets still catch a ~1000x regression.
  * *(minor, 1/3)* *"`run_e2e_pure_combo` silently breaks its 'pure'
    contract."* Half right. "Pure" is documented as *the horizontal-stack
    candidate off*, which is still exactly what it does; `cell-composed`
    was never part of that contract and was absent by fossil, from the
    baseline columns AND the `default` column alike. Disabling it in the
    pure columns only would break the apples-to-apples comparison the
    sweep exists for, so no override — but the doc now says precisely what
    "pure" means and warns that `full_knob_sweep` tables from before
    2026-08-21 are not comparable to ones after.
  * *(minor, 1/3, REFUTED with an acknowledged residual)* *"a consistent
    reversal of both terminal emitters sails through the corroboration."*
    True, and already stated in the test's own comment. Corroboration
    narrows a failure mode, it does not remove it, and it cannot be
    removed from a test: the fix is a structural nesting marker in the
    trace contract, which is Phase 1b/2a's to own. Strengthened as far as
    a test can go — the STAGE is pinned too, so "read the nested board
    instead" shows up as a stage mismatch (gear@20's nested board decides
    at `best-accepted`, the outer at `best-error-free`).
  * *(nit, REFUTED)* *"nothing re-derives tier5's 10; the answer lives
    only in the RFC."* The committed warning pin
    (`tests/goldens/warnings/tier5_processing_unit_from_ore_am3.txt`) is
    re-derived and asserted on every suite run — that is what
    `assert_warnings_golden` does. The RFC carries the ADJUDICATION, which
    is a different artifact from the value.
  * *(nit, REFUTED)* *"the corpus's `e2e-harness` column keeps generating
    a baseline for a candidate set nothing ships."* Deliberate and
    documented: it is the historical record the W2c re-blesses were
    adjudicated against, and deleting it would re-take 32 cells and
    destroy the only evidence of what the fossilized suite decided.

  **#699 review rounds 3 and 4 absorbed** (4 + 5 findings; the reviewer
  states none blocks merge). Three produced real corrections:
  * *(round 3, minor — a defect this PR introduced)* `parity_corpus.rs`'s
    `OPTION_SETS` docblock said both things at once: round 1 appended the
    "both fossils are dead, this column is historical" correction AFTER
    the pre-W2c hand-verification receipt, leaving two mutually exclusive
    descriptions of the same label with no way to tell which was current.
    Restructured — live statement leads, receipt kept behind an explicit
    SUPERSEDED heading as provenance for the committed cells.
  * *(round 4, minor, 3/3 — correct, and measured)* The two re-pinned
    decomposition tests' "two independent terminal emitters corroborate"
    claim is **vacuous on those fixtures**: both emit exactly ONE
    `DecompositionChosen` and ONE `SelectionDecided`, because `native`
    wins and nests nothing — the cell-composed candidate runs and loses,
    and `run_candidate` truncates a loser's nested block (oracle gap
    (g)). Verified from a decoded snapshot. The check is kept as a guard
    that arms itself if a nesting candidate ever wins those fixtures, and
    the comments now say plainly that today's weight is carried by the
    STAGE pin and the cell-composed-presence assertion; the fixture where
    the ordering IS exercised is `tier1_iron_gear_wheel_20s`.
  * *(round 4, docs-only, 1/3)* The capacity-arm claim "changes no winner
    and no deciding stage anywhere" was over-broad — corrected above to
    scope it to the (fixture, machine) pairs the suite runs, since the
    corpus records two capacity-attributable winner changes at am3.
  Absorbed as scoping rather than mechanism: the residual-literal guard's
  reach is now stated as "a textual copy of these two lines cannot be
  added silently" (not "the fossil cannot come back"), with the
  `include_str!` binary cost noted; `run_e2e_pure_combo`'s note now
  covers the inserter ladder as well as the candidate set; the gear@20
  pin's failure message enumerates all five coupled artifacts. The
  restated ordering residual and the restated in-suite-tripwire major are
  answered by the correction above — the armed guard is W1a's tripwire
  row, which predates this PR.

  **#699 review rounds 5 and 6 absorbed** (6 + 6 findings). Four produced
  real changes, and one produced the best single detail in the whole
  campaign so far:
  * *(round 6, minor, 2/2 — asked as a question, answered with a
    receipt)* "How does the same layout score `layout_warnings: 1` at
    selection and 0 at validation?" No contradiction — different fields
    (`LayoutResult.warnings` is the producer's own list; `validate()`'s
    issues are the 39 functional checks, which is what the warning golden
    counts). **But the one entry is the story**: decoded from the
    snapshot, gear@20's winning cell-composed candidate carries
    `"cell-composed: geometry NOT sim-verified (hash c5c5f88087df894c) —
    run spaghettio-sim and add the entry to cell-sim-registry.json"`.
    RFC-051's own machinery flagged this exact geometry as unverified;
    `best-error-free` filters on errors and ranks on score with warnings
    taking no part; and the thing it was not verified for is precisely
    what the meter measured. That is the sharpest available argument for
    W2b's severity-aware verdict, and it was sitting in the trace the
    whole time.
  * *(round 5, minor, 1/3 — correct, and my own bug class in W2a's file)*
    `bus/layout.rs`'s legend said "**Zero production callers as of this
    PR**" about `from_groups`. This PR makes `e2e.rs` the first caller, so
    the sentence became false in the same diff that falsified it.
    Replaced with an adoption-status list naming the caller, the two
    exporters, and what is still open.
  * *(round 5, major, 3/3, fourth restatement — answered with code)*
    `assert_produces(…, 20.0)` was deleted from
    `tier1_iron_gear_wheel_20s`. It reads `analysis.throughput_estimates`
    — a static estimate from the machine count — and asserted "produces
    20/s" about a layout the meter reads at 15/s: a green assertion
    saying the opposite of the measurement. Replaced by an inline check of
    the same number under a message that names it a PLAN check, states the
    measured 15.0/s and the tripwire row, and says not to read the test
    passing as delivery. Rejected alternative recorded at the call site:
    metering in-suite would close a dev-dependency cycle
    (`spaghettio_meter` → `spaghettio_core`) to duplicate a guard that
    already exists armed.
  * *(round 6, minor, 1/2 — correct, and fixed structurally rather than
    by assertion)* Round 2 had added two guard assertions pinning
    `LayoutStrategy::Pooled` and `horizontal_candidate == true`, because
    every `run_e2e*` wrapper spelled those as hard literals. Round 6
    pointed out that an assertion re-spelling an engine default is itself
    a second copy of it. So the wrappers were fixed instead: `run_e2e_inner`
    now takes one `HarnessOptions`, and every wrapper spells only what it
    deliberately varies (`run_e2e_pure_combo`'s `horizontal_candidate:
    false` is the sole non-default in any of them), leaving the rest to
    `HarnessOptions::default()`, which reads the engine's group defaults
    at runtime. The two assertions were deleted as redundant. **Verified
    behaviour-neutral**: a full golden+STRESSGOLD fingerprint run before
    and after the refactor is byte-identical on all 8 + 8 hashes. The
    stale `#[allow(clippy::too_many_arguments)]` went with the four
    dropped params.
  * *(round 5, major, 1/3)* `full_knob_sweep`'s markdown now stamps an
    "Option-set epoch: post-RFC-070-W2c" banner into the ARTIFACT — these
    tables get pasted into issues, where a doc comment on
    `run_e2e_pure_combo` is invisible.
  * *(round 6, major, 2/2, fifth restatement — REFUTED with a receipt)*
    "the pin monitors who wins, not whether it delivers, and the meter
    tripwire never runs under `cargo test -p spaghettio_core`." Both true.
    The tripwire being opt-in is a **standing project decision with its
    own recorded reasoning**, not an oversight of this track:
    `e2e_tripwire.rs`'s "Why report-only stays the default, and this is
    NOT wired into CI as a gate" — gating a host-cache-relative instrument
    with no track record repeats #632 B7's mistake, and promoting `check`
    is named there as future work. Overturning it is not W2c's to do. The
    residual is real and recorded: a further regression from 15.0 to, say,
    13.5 would pass every core-suite check. That is an argument for
    promoting the tripwire, which belongs with whoever owns the gate.
  * *(round 6, minor, 1/2, REFUTED)* "no fixture-level ancestry is
    recorded for the non-golden-pinned fixtures." The per-fixture stress
    deltas ARE recorded above (ec22 5→4, ec23 7→6, ec40 283→235, entity
    counts on all 8), measured under the new config, with the direction
    stated and the alarm condition (a category reaching zero) checked and
    absent.
