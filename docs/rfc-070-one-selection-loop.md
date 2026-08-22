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
- *2026-08-21 — **Phase 1b built** (#689 track W2b), additively:
  `bus/selection_policy.rs` plus the severity channels on `verdict.rs`.
  Nothing is wired; `select_best_decomposition` is unchanged apart from
  the stale-prose sweep the spec asked for and one `pub(crate)`
  widening. **`policy_replay` reproduces all 140 decided #694 cells by
  winner AND deciding stage** (`best-error-free` 87, `scoped-pairwise`
  26, `merge-tap` 15, `best-accepted` 12 — identical to the committed
  baseline), so K70-1's offline precursor PASSES: **no
  candidate-identity-conditioned logic was needed anywhere**, and the
  boundary is now a mechanical test (`k70_1_fence_holds` reads the
  module's own source between fence markers and fails on any candidate
  name or `.name` read inside the stage logic). Transcription calls,
  each then checked by the replay: the `ranking_len` slice becomes the
  AdmissionRule as a FILTER — equivalent because the two `scoped`
  producers are exactly the tail the slice excluded, so the rule is
  field-keyed rather than position-keyed; the #474 non-shadowing rule
  becomes `ChainBehavior::DeferToRemainingPairwiseStages`, where a held
  incumbent answer waits on the remaining pairwise stages and then
  TERMINATES rather than falling through to the ranked ones; v1's lazy
  `clean_flags` needs no laziness flag because the gap rule already
  reproduces it (a single-layout solve records no counts, so the
  error-free tier is empty and `best-accepted` decides); the gate
  becomes an ordered clause list reporting the FIRST failing conjunct,
  which closes Phase-0b oracle gap (c); the `verdict.rs` severity
  channels are RECORDED-ONLY, never consulted by `regressed`/`pass`, so
  the celldb-harness-green obligation holds with those tests
  unmodified.*
- *2026-08-21 — **the parity corpus does NOT discriminate two of the
  three comparators**, found by EXECUTING the discrimination checks
  rather than assuming them, and this is a direct warning to Phase 2a.
  Three deliberate breaks, one whole corpus run each:*
  - *making the component-wise floor LEXICOGRAPHIC — the #474 bug the
    `IssueCounts` doc is written against — **passes 140/140**. No corpus
    cell has a scoped candidate that is better on one severity channel
    and worse on another.*
  - *making stage 1's `on_incumbent_win` `Terminate` — the #474
    SHADOWING bug, where merge-tap's "native won" short-circuits DI's
    already-computed result — **passes 140/140**. On every merge-tap
    cell in the corpus the scoped candidates refuse or lose, so the
    deferral is never load-bearing there (consistent with the
    `ec@30` observation in the oracle-gaps entry).*
  - *`AdmissionRule::AdmitAll` — dropping `ranking_len` — **FAILS**, and
    the failures are exactly the regression class the rule exists to
    block: horizontal-stack displacing native in the error-free tier on
    the `electronic-circuit` / `advanced-circuit` / `processing-unit`
    rows. So the harness is not vacuous; it is BLIND in two specific
    places.*
  - ***The generalisation for Phase 2a: corpus parity is not evidence
    about the floor's component-wise-ness or about non-shadowing.***
    That is the same shape as the recorded `first-produced` hole — a
    shadow loop can hold parity across all 160 cells with either of
    those two semantics implemented wrongly. The unit tier is what
    covers them, and each cover was executed rather than assumed: the
    lexicographic-floor break failed
    `the_floor_is_component_wise_not_lexicographic` ("a layout-warning
    regression must not hide behind a validator-warning improvement");
    the `Terminate` break failed
    `an_incumbent_win_at_stage_one_does_not_shadow_the_pairwise_floor`
    (`{winner: native, MergeTap}` where `{winner: DI, ScopedPairwise}`
    was required) and — correctly — left
    `a_held_incumbent_stands_when_no_pairwise_stage_displaces_it`
    passing, since that test guards the OPPOSITE direction; deleting the
    held-answer early return failed that second test plus three
    stage-1 pins. All four breaks were restored and re-verified green.
    A future widening of the corpus toward a cell where merge-tap runs
    AND a scoped candidate wins would close the non-shadowing hole;
    nothing in the current fixture list reaches it.*
- *2026-08-21 — **#698 review round 1 adjudicated** (2 major, 5 minor, 1
  nit; all absorbed, nothing refuted). The headline is a **correction to
  this RFC's own §"Validation-once and laziness"**, and it is a Phase-2a
  constraint rather than a W2b bug: the spec says eager vs lazy
  measurement "cannot change outcomes, only cost" because `validate()`
  is deterministic. **That is true of the WINNER and false of the
  deciding STAGE**, and the divergence-equivalence rule compares
  `(status, winner, stage)`. `IssueProfile::measure` always fills
  `counts`, so the gap rule cannot fire on a live profile; v1 skips
  `clean_flags` entirely on a single-layout solve (`n_layouts > 1`),
  leaving the error-free tier empty so `best-accepted` decides, while an
  eagerly-measured v2 populates the tier and decides at
  `best-error-free`. **That is exactly the shape of the 12
  `best-accepted` cells** this log already attributes to the
  cells-off/only-native mechanism. So Phase 2a must either preserve the
  laziness AS POLICY (skip measuring below two produced candidates) or
  accept those 12 as minor divergences and adjudicate them — it is not
  the free implementer's choice the spec offers. Demonstrated, not
  argued: `eager_measurement_moves_the_deciding_stage` pins both halves
  (same winner, different stage) from hand-built profiles.*
  - *Also absorbed: `policy_replay` could **manufacture a false K70-1
    finding** — where v1 itself had drifted off the committed baseline,
    the harness reported the drift as "an ENGINE change, not a policy
    finding" and then ALSO pushed the same cell into the campaign-level
    assertion, whose message says the opposite. The v2-vs-baseline
    comparison is now skipped on a drifted cell. Same round: an absent
    baseline silently no-op'd every v2-vs-baseline comparison while the
    run still read green (the #693 "compared nothing reads as clean"
    shape, one path over) — the baseline is now required; the
    `decided == 140` literal now derives from the committed record's own
    decided count, so a deliberate corpus widening travels with its
    re-bless; `GateContext::any_prior_accepted` scanned the WHOLE
    `prior` array while its doc claimed "registrations before this one",
    which would have let a later producer's acceptance stand
    `size-split-2` down — bounded, with a test for the later-slot case;
    and `contamination_weight` is now sourced from
    `KIND_CONTAMINATION_WEIGHT` rather than re-typed beside it.*
  - ***Two further blind spots in the acceptance harness, recorded
    alongside the two comparator holes above***: `decide()` consumes
    already-produced profiles and never evaluates a `ProducerGate`, so
    **the 140/140 result covers zero gate transcription** — a
    mis-transcribed eligibility clause is invisible until the Phase-2a
    shadow, where it moves the candidate SET rather than the ranking;
    and `policy_replay` is `#[ignore]`d, so **CI never runs the
    acceptance bar** — "the parity harness passes" always means a
    hand-run sweep with the zone-cache pin. Both are now stated at the
    test's own doc so the claim cannot be quoted without them.*
- *2026-08-21 — **#698 review round 2 adjudicated** (2 major, 5 minor, 1
  nit; 6 absorbed, 1 half-refuted, 1 refuted). **Both majors were about
  the same thing and it is the useful pattern of the round: a
  Phase-1b helper that Phase 2a will call is a place where a v1
  discipline can be silently dropped.** (a) `refuse_on_error` was policy
  data no code path applied — a naive `measure → decide` wiring would
  have handed DI / horizontal / cell-composed an error-laden `Produced`
  profile able to displace a healthy incumbent, inverting the asymmetry
  the flag exists to state, and `policy_replay` cannot see it because it
  replays rows where v1 already refused. `IssueProfile::measure` now
  takes the registration and applies the gate — and, unlike v1, KEEPS
  the measurement: the refusal reason carries the error categories and
  the counts/kinds stay on the profile, which is Phase-0b oracle gap (d)
  closed rather than merely deferred. (b) `measure` ran `validate()`
  with no emission discipline, where v1 wraps every one of its
  `validate()` calls in peek/truncate so a loser's `ValidationCompleted`
  cannot leak into the winner's replayed stream — that is #396, hit
  twice before; `measure` now runs muted.*
  - *Also absorbed: the category→kind table was a SECOND hand-typed copy
    of `classify_errors`'s match, guarded only by a seven-category unit
    test, so a category added there would have fallen silently to
    Starvation here — both now read
    `CONTAMINATION_CATEGORIES` / `STRUCTURAL_CATEGORIES`, hoisted out of
    the match (behaviour-identical; the merge-tap corpus cells exercise
    it); `any_prior_accepted`'s bound was a `min()` that silently
    degraded an out-of-range index back into the whole-array scan it had
    just removed — now `debug_assert`ed; and
    `Verdict::candidate_selection_warnings`'s doc promised
    `selection_warning_count` semantics that only hold when the policy
    carries the exclusions, which `fold()`/`decomposition()`
    deliberately do not.*
  - *Half-refuted: `quality_key_stage`'s "incumbent produced nothing →
    the rival wins" arm IS unreachable under today's gates, as the
    reviewer says — but it is a faithful transcription of v1's
    `merge_tap_choice` arm, which carries the same unreachability note
    at its own site, and deleting it would leave the stage undefined in
    a state v1 answers. The claimed inconsistency with the floor stage's
    opposite convention is also v1's (`di_choice`'s early return):
    two mechanisms, deliberately different. Comment strengthened, branch
    kept. Refuted outright: the gate-coverage / `#[ignore]` finding is a
    re-raise of round 1's, absorbed there and already disclosed in the
    code the reviewer is reading. The nit (a `debug_assert` on the
    incumbent-kinds gap branch) is declined on principle: this module's
    stated rule is that a gap SKIPS, and a panic path inside a pure
    decision function contradicts it — the caller-contract violation in
    `any_prior_accepted` is a different class, which is why that one got
    the assert.*
- *2026-08-21 — **#698 review round 3 adjudicated** (3 major, 6 minor, 4
  nits; 5 absorbed, 4 refuted as re-raises or as-designed). **The
  round's real contribution is that it refused to let round 2's
  eager-measurement finding stay a documented note**, and it was right
  to: v1's `clean_flags` laziness is now
  `MeasurementRule { min_produced_for_error_free_tier: 2 }` — policy
  data that `decide()` ENFORCES, rather than a property that held only
  because the recorder happened not to compute counts on single-layout
  solves. Eager and lazy measurement now reach the same stage by
  construction, so **the 12 `best-accepted` cells are no longer a
  Phase-2a trap**; the RFC's "cannot change outcomes, only cost" is true
  as written once the rule is part of the program. `policy_replay` is
  unaffected (the recorded profiles carry the gap either way) and was
  re-run to confirm.*
  - *Also absorbed: `Verdict::candidate_selection_warnings` now returns
    `Option<usize>` and answers `None` unless the policy DECLARED a
    selection scope — under `fold()`/`decomposition()` it previously
    returned a plausible number that counted `belt-detour`, i.e. the
    opposite of the selection-scoped figure, to a caller who asked for
    selection semantics. A gap, not a wrong number, per this campaign's
    own rule; `Policy::selection()` is the named preset that answers.
    A `refuse_on_error` refusal no longer reports `accepted: Some(true)`
    (v1 never emits `Refused`-and-accepted; the gate's observation now
    rides in the refusal reason). `decide()`'s length check became a
    `debug_assert` plus a release refusal, so a caller bug degrades to
    "no decision" instead of a plausible wrong winner. The
    `any_prior_accepted` bound was `<=` where valid indices are
    `0..len`, which let `== len` slice the whole array through the
    `min` — the exact scan the bound removes.*
  - *Refuted: the "acceptance bar is not executable / should be
    CI-gated" finding, for the third time across three rounds — the
    answer is unchanged and is #694's, adjudicated four times there: a
    corpus gate today asserts only "production has not changed", which
    every engine PR legitimately falsifies. The gate that means
    something is Phase 2a's shadow-vs-production check, which
    Verification plan item 2 already commits to; the always-on gate here
    is the comparator unit tier, which is what the finding recommends
    and what already exists. Also refuted: a `debug_assert` on
    `quality_key_stage`'s unreachable incumbent-refused arm (same
    reasoning as round 2 — a gap SKIPS, and a pure decision function
    does not gain panic paths for impossible states), and the
    NaN-vs-None score asymmetry in `ranks_ahead` (both are v1's: NaN
    ties via `partial_cmp().unwrap_or(Equal)`, and a produced candidate
    never lacks a score).*
- *2026-08-21 — **#698 review round 4 adjudicated** (no majors — 8 minors
  and 3 nits, the round the findings converge). Absorbed, and the two
  worth keeping: (a) **four of the seven producer gates had no test at
  all** — `policy_replay` evaluates ZERO gates by construction, and only
  merge-tap's and size-split's were unit-pinned, so a mis-transcribed
  eligibility clause in k1 / cell-composed / DI / horizontal would have
  survived 140/140 and first appeared in Phase 2a as a changed candidate
  SET. All seven are now pinned clause-by-clause against v1's
  conjunctions. Writing them found one of my own assumptions wrong: a
  gear solve IS chain-eligible, so the `chain-eligible` clause has no
  negative case among the cheap fixtures — recorded at the test rather
  than papered over. (b) `firewalls` was a vector nothing read, which
  makes "firewall" a word for a comment; a `Firewall` now names the
  categories its receipt argues for and a test pins that set against the
  live exclusions, so changing them without touching the argument fails.*
  - *Also absorbed: `incumbent_accepted()` was not order-bounded the way
    `any_prior_accepted` is — both now go through one `prior_slot`
    helper enforcing "a gate may only read producers registered before
    it"; `incumbent_index()` asserts at most one incumbent (two would
    silently rank the second as an ordinary challenger); the
    `min_produced_for_error_free_tier` doc now states the dependence
    that makes it equal v1's `n_layouts` — v1's scoped arms self-refuse
    INSIDE `produce()`, so a producer given `refuse_on_error` without an
    equivalent produce-side refusal would shift the tier's availability
    invisibly to the replay; and the #632-B6-deletion provenance now
    names BOTH numbers at every site — **PR #684** did the removal under
    **issue #675**'s Tier 2 item 9 — since the RFC cited one and the
    canonical constant the other, which reads as a contradiction to
    anyone reconciling them (verified against both: #684 is
    "del(t2e): remove the never-sim-anchored inserter-throughput check
    pair (item 9)", #675 is the off-path tracking issue).*
  - *Refuted: CI-gating the corpus, for the FOURTH time in four rounds —
    the answer is #694's and has now been given eight times across two
    PRs. `measure`'s refusal string not matching v1's is deliberate and
    nothing compares them: the equivalence rule is (status, winner,
    stage), and v1's string is the lossy artifact being replaced.*
- *2026-08-21 — **#698 review round 5 adjudicated** (1 "major", 9 minors;
  5 absorbed, 4 refuted — closing round). **The one thing worth the
  round**: the fifth raise of "the corpus isn't CI-gated" carried a NEW
  sub-argument that was correct — `policy_replay`'s provenance guard
  PRINTED a note and passed, so a hand-run under a mis-pinned zone cache
  green-lit a comparison against a record it could not reproduce. That
  is the "compared nothing reads as clean" shape #693 closed and #694
  round 3 closed again in `parity_corpus`'s own `check` mode,
  reappearing one path over in the sibling test. It now hard-fails, with
  `SPAGHETTIO_POLICY_REPLAY=any-cache` as the named escape — the same
  posture, and the same escape shape, as `check-any-cache`. **The
  generalisation: when a file gains a second consumer of the same
  cache-relative data, the hardening does not come with it.** The
  CI-gating half is refuted for the fifth time; the answer is unchanged.*
  - *Also absorbed: the K70-1 fence now strips COMMENT lines before
    scanning, so stage-logic prose may name a candidate while stage code
    may not — a fence that policed prose would push a future author to
    write worse comments or widen the fence, both worse than what it
    prevents (discrimination re-executed after the change: a
    `p.name == "native"` branch inside the fence fails it with the right
    message, restored green). The gate tests now say what they are NOT —
    self-consistency against the v1 condition as READ, not equivalence
    against production's dispatch, which only the 2a shadow can give.
    `ranks_ahead`'s `usize::MAX` fallback is labelled DEAD rather than
    described as reproducing v1's unclean key. `Verdict::candidate_errors`
    says it is a whole-side total, not a regression count.
    `Policy::selection()` says its `pass` is always true because it gates
    nothing.*
  - *Refuted with receipts: (a) "`prior_slot`'s `min` re-introduces the
    later-producer bug in release when `registration_index >
    prior.len()`" — it does not: if the index exceeds the array, EVERY
    slot is registered before it, so scanning all of them is exactly the
    intended set. The assert catches the authoring mistake; the release
    fallback is not wrong. (b) "the two kind lookups could drift" —
    stale by one round: both now read the same
    `CONTAMINATION_CATEGORIES` / `STRUCTURAL_CATEGORIES` constants, so
    there is one definition, not two. (c) a `debug_assert` on
    `quality_key_stage`'s unreachable arm, for the third round running —
    same answer. (d) the scoped pair-winner's gap handling matches v1
    (a candidate whose counts are absent has no `di_choice`/
    `horizontal_choice` either), and the reviewer concedes it is
    unreachable against the live recorder.*
- *2026-08-21 — **#698 review round 6 adjudicated** (no majors, 9 minors,
  and the pass itself was thinner — union ×2 rather than ×3, with most
  findings re-raises). **Review close-out: six rounds, ~40 findings; the
  load-bearing ones were all about the same thing — a Phase-1b helper
  that Phase 2a will call, silently dropping a v1 discipline
  (`refuse_on_error` unapplied, `validate()` unmuted, laziness
  unenforced, the provenance guard that printed instead of failing).
  The 140 cells never moved once across five reproductions.***
  - *Absorbed: `severity_split` counted warnings as `len() - errors`,
    which hardwires `Severity` to two variants — a third would land
    silently in the warning channel; `ranks_ahead`'s warnings-first arm
    now ABSTAINS on a missing key instead of sorting it last through a
    sentinel, so the module's "a gap skips" rule holds locally rather
    than depending on a non-local invariant to keep the sentinel dead;
    `decide`'s terminal `return held` is labelled unreachable under
    today's program (the prose implied it did work); and the fence is
    now described as mechanical **for the literal form** rather than as
    a proof — a runtime-assembled name passes it, an inline comment
    naming a candidate fails it, and the second direction is the safe
    one.*
  - ***The gap-assert family is settled***, having been raised in some
    form in rounds 3, 4, 5 and 6: `decide`'s doc now states the
    precondition explicitly — projections are present for every
    candidate a mechanism examined and absent for every one it did not —
    and names where it is CHECKED, which is the boundary
    (`profile_from_row` rejects a partial count or kind triple), not the
    stages. Two reasons, recorded so the next round does not re-open it:
    a rule that says "a gap skips" cannot also panic on selected gaps
    without becoming unstatable, and a pure decision function is the
    wrong place to validate data it did not build. Refuted with
    receipts: the ignored-test governance finding (sixth raise), and the
    claim that `policy_replay` still conflates its two comparison
    signals — the drifted-cell guard from round 1 is exactly that fix,
    and on a NON-drifted cell v1 equals the baseline, so the two
    comparisons cannot disagree.*
- *2026-08-21 — **#698 review round 7: NO BLOCKERS, NO MAJORS**, seven
  minors and four nits, every one a re-raise or refutable. The campaign
  note worth keeping is about the INSTRUMENT: across seven rounds the
  bot raised "the corpus is not CI-gated" seven times and the
  unreachable-arm assert four times, while the four findings that
  actually changed the design all appeared exactly once each. **A
  re-raise carries no additional evidence, and the count of raises is
  not a measure of a finding's weight** — the same lesson #694's
  close-out recorded about its own six rounds.*
  - *Refuted with a worked receipt, and this is the round's one
    genuinely new claim: "the NaN tie-break direction diverges from v1
    — v1 keeps the later candidate, v2 the earlier". It does not. v1's
    comparator is `a.partial_cmp(b).unwrap_or(Equal).then(ib.cmp(ia))`
    under `max_by`; the index term is REVERSED, so a smaller index
    compares as Greater and the max is the EARLIEST index — which is
    what v2's strictly-better-only fold also yields, since `partial_cmp`
    against a NaN is never `Some(Greater)`. The reading dropped the
    reversal. Now pinned both directions by
    `a_nan_score_keeps_the_earliest_registration_as_v1_does`, so the
    claim is answerable from the test rather than from a re-derivation.*
  - *Absorbed: a `debug_assert` that a program declares at most ONE
    deferring stage — the chain holds a single held answer and a second
    would overwrite it silently, which is the same policy-authoring
    class as two incumbents and now gets the same treatment. And the
    `score: Some` / `accepted: None` asymmetry on a produce-time refusal
    is now explained where it lives: a score is a MEASUREMENT the
    refusal does not invalidate, `accepted` is a VERDICT about admitting
    a layout that no longer exists. Refuted: the corpus's two comparator
    blind spots (this PR's own finding, disclosed in three places), the
    speculative third-scoped-producer fold (v2's fold reduces exactly to
    v1's two-valued join on the two that exist), and the unreachable-arm
    assert for the fourth time.*
- *2026-08-21 — **#698 review round 8: no majors for the third round
  running**, four minors, union ×2. **Review cycle CLOSED here**, on the
  instrument's own evidence rather than on patience: three consecutive
  rounds without a major, the passes thinning from ×3 to ×2, and the
  "not CI-gated" finding raised for the EIGHTH time. Absorbed, all
  one-liners: `incumbent_index` now asserts EXACTLY one incumbent rather
  than at most one — zero is equally malformed and less obvious, since
  the two pairwise stages disagree about what it means (the quality-key
  one hands its rival an unconditional win, the floor abstains forever);
  the `firewall_receipts` test checks each receipt claims a non-empty
  set, not only that the union matches, so an empty one cannot ride
  along inside another's set-equality; and `measure`'s comment now says
  outright that a profile built there and one built from a recorded row
  are NOT field-identical for a refused candidate — compare decisions
  across construction sites, never profiles.*
  - ***Standing hand-off note for Phase 2a***, since it is the one thing
    every round agreed on and it is this PR's own finding: **the 140/140
    result covers the comparators' RANKING, not the gates, and not two
    of the three comparators' defining semantics.** What a shadow loop
    must therefore not conclude from corpus parity: that its floor is
    component-wise, that it honours the #474 non-shadowing rule, that
    `first-produced` behaves, or that any eligibility gate was
    transcribed correctly. Each of those has its own cover — the unit
    tier for the first three, the clause-by-clause gate tests for the
    fourth — and the 2a shadow, which runs both dispatches on the same
    solve, is the first instrument that can check them together.*
