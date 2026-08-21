# RFC-070: One Selection Loop

## Summary

Replace `select_best_decomposition` — the production candidate-selection
loop, which today braids three incompatible verdict mechanisms through
~880 lines and lets three of its seven candidate arms carry bespoke
refusal logic — with a single policy-driven selection loop derived from
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
  (`DirectInsertionCandidate` :142, `CellComposedCandidate` :323, among
  seven implementors of the `DecompositionCandidate` trait :63).
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
  `selection_warning_count` (`validate/mod.rs:422`) or
  `SELECTION_EXCLUDED_WARNING_CATEGORIES` (:416), has no
  `catch_unwind`, and no sim-anchor firewall.

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
- **Producers** — the seven existing `DecompositionCandidate`
  implementors register unchanged at first; the trait survives. The
  incumbent (native) is a ranked competitor, as in `candidate_runner`
  today.
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

- **K70-1 (expressibility — the premise test).** After Phase 1b, the
  shadow loop must reproduce the Phase-0 baseline winner on every
  corpus fixture using policy data alone. If matching the baseline
  requires *candidate-identity-conditioned logic in the loop body*
  (any branch keyed on a producer's name rather than on policy), the
  "four answers to one question" premise is false: stop, keep
  generation 1, and instead delete generation 4's unused surface
  (`objective.rs`/`verdict.rs` beyond what RFC-068 needs). Record and
  archive.
- **K70-2 (parity budget).** At the Phase-2b flip gate: if more than
  **3** corpus fixtures diverge from baseline *after* policy fixes,
  or any single divergence cannot be adjudicated
  equal-or-sim-verified-better, the flip halts and the RFC pauses for
  re-design. Divergences are adjudicated individually in this log.
- **K70-3 (cost).** If the e2e corpus wall-time regresses >10% under
  the shadow-mode run, or the stress corpus trips its 600s timeout
  where it did not before, the shadow design is wrong — fix the cost
  gating before any further phase, or stop.

## Verification plan

Per the CLAUDE.md layout-engine protocol, plus campaign-specific
instruments:

1. **Phase-0 baseline is the oracle.** The selection scoreboard
   (extension of #686's census) records, per corpus fixture: candidate
   set, per-candidate verdicts under each mechanism, winner, and
   *which precedence-chain stage decided it*. Committed as data;
   every later phase diffs against it.
2. **Shadow parity in CI** (Phase 2a onward): the v2 loop runs beside
   production on the corpus; winner mismatch fails the check.
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
  baseline winners reproduced (K70-1's test). **Owner design review
  gate before Phase 2.**
- **Phase 2 — migration.** 2a: v2 loop in shadow + parity CI gate.
  2b: flip `build_bus_layout` to v2 (**owner evidence review gate**;
  K70-2 adjudicated here). 2c: hoist per-arm bespoke logic into loop
  stages. 2d: delete the gen-1 loop — flip condition for the deletion:
  corpus parity held through 2b + 2c with zero unadjudicated
  divergences.

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
