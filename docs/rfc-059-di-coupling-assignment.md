# RFC-059: Direct-insertion coupling assignment

## Summary

A DI spec may be fused into at most one cell. When a spec is an eligible
member of two different couplings, the dispatcher currently resolves the
contention by **iteration order** — consumers are walked in topological order,
so the upstream coupling claims the spec and the downstream one is never
evaluated. That is not a decision anyone made; it is a loop direction.

This RFC proposes replacing it with an explicit, measured assignment policy,
and — equally in scope — proposes the possibility that the answer is "keep
upstream-first, pin it with a test, and write down why." The point is to stop
having an unexamined tie-break in a code path that now runs by default.

## Motivation

Concrete and reproducible, from #473 (`iron-plate → iron-stick → rail`):

```text
COUPLING iron-plate -> iron-stick on iron-plate    <- claimed first
COUPLING iron-stick -> rail       on iron-stick    <- never tried
```

`iron-stick` is eligible in both couplings. The greedy upstream-first walk
claims it for a stacked cell, and `iron-stick → rail` is skipped **before its
eligibility is evaluated at all**. Confirmed by instrumenting the dispatch, not
inferred.

Forcing the downstream coupling to claim first (scratch env flag, never
committed) builds `di-row:iron-stick:rail` with **0 validation issues** and
**261 entities against the forward order's 264**.

So the geometry #473 built is reachable, and it is reached only by a policy this
project has never agreed. That is the whole motivation: the blocker for `rail`
turned out not to be geometric.

**The honest counter-evidence, stated up front**, because it is why #473
declined to flip the order in passing:

- `electronic-circuit@10`, `steel-plate@5` and `iron-gear-wheel@10` are
  **byte-identical** under both orders.
- The full suite is green under both.
- The measured win is **3 entities at one rate** — though *which* rate is
  disputed by the record itself, see below. The claim that `rail@5` is the
  no-difference case comes from that same contradictory passage, so it appears
  here as what was recorded, not as established fact.

Three entities is not a mandate to change a corpus-wide tie-break. The reason to
do this work is that **nobody knows what the right rule is**, not that the
current one is demonstrably costly.

### The winning rate is not recoverable from the record

Review of this RFC asked which rate produces 261-vs-264, since `rail@5` is
explicitly the no-difference case. The answer is that **RFC-053 contradicts
itself**, so no rate can be quoted honestly:

- **#473** (branch `feat/di-three-inputs`, *not yet merged*) adds a passage to
  `rfc-053` putting the win at `rail@1`, and saying that at `rail@5` "the
  straddle does not balance and reverse order builds the same stacked cell as
  forward".
- `rfc-053`'s coupling table **on `main` today** says `plan_row_straddle`
  "balances at only 2 of 12 sampled rates (**5/s, 10/s**)", and that at 1/s
  `snap()`'s rounding leaves supply and demand unequal — `P1:C1, 3.0 vs 1.5`.

Cited by PR rather than by file-and-line deliberately: the first passage does not
exist on `main`, so a line reference would not resolve for anyone reading this
RFC after it lands, and did not resolve for the reviewer who checked. #473 — still open at the time of writing — adds a
correction note at that passage (`rfc-053` line ~2260, commit `503ca889`)
recording the conflict rather than silently repairing it, because **which half is
wrong is not determinable from the document alone**. Flagged as pending rather
than stated as fact: if #473 lands in a different shape, this sentence is the one
that goes stale.

Those disagree about which rates balance, and therefore about where the win is.
The measurement came from a scratch env flag that was never committed, so it
cannot be re-run from the repository as it stands.

**This raises the value of phase 1 rather than lowering it.** The first
deliverable is not a policy — it is a reproducible measurement of *where the two
orders differ at all*, sweeping rates rather than trusting either recorded
figure. Until that exists, "the win is 3 entities at one rate" is a claim with no
executable form, and kill criterion 1 cannot be evaluated.

### What changed since #473, and why the question is now answerable

#473 noted that "a green suite is weak evidence here — nearly every test runs
with `direct_insertion: false`." **#474 changes that**: DI defaults to
`Candidate`, so the corpus now exercises DI on every layout, and the never-worse
decision means a *wrong* claim choice shows up as a **missed improvement**
rather than a regression. Before #474 this RFC could not be measured without
building a bespoke harness. After it, the existing change-surface sweep answers
it directly.

## Design

The contention is a **matching problem**, not an ordering problem: given a set
of candidate couplings over a shared pool of specs, choose a subset that is
pairwise spec-disjoint and best by some objective. Framing it as "which
direction do we walk" is what hid it.

Four policies, in increasing cost:

| Policy | Rule | Cost |
|---|---|---|
| **P0 — upstream-first** | status quo; topological order claims | zero |
| **P1 — downstream-first** | reverse the walk | zero |
| **P2 — greedy by gain** | score each candidate coupling in isolation, claim in descending order of predicted gain | one extra pass per coupling |
| **P3 — optimal matching** | max-weight matching over the coupling/spec bipartite graph | small; contention sets are tiny |

**Predicted shape of the diff.** The dispatcher's claim loop
(`bus/di_cell.rs`, the walk that produces fused specs) gains a policy parameter
rather than an inverted loop. P0/P1 are orderings of the same loop. P2 needs a
per-coupling gain estimate that does **not** require building the layout —
otherwise the cost is a full layout per candidate. P3 needs the contention graph
materialised, which is only worth it if P2's greedy proves to be measurably
sub-optimal.

**Load-bearing constraint.** Whatever the policy, it must interact correctly
with #474's `di_choice`: DI competes against a DI-free native layout and may
only displace it on a strict improvement across both issue channels. So a claim
policy cannot make anything *worse* — the worst a bad policy does is leave a
better cell unbuilt. That bounds the risk of this work to "missed upside", and
it is why the RFC is worth doing at all rather than being dangerous.

**Rejected alternative: score-driven claiming that builds each variant.**
Building a full layout per candidate coupling to score it is the obvious
approach and is refused on cost — the decomposition search already builds
several candidates per layout, and DI multiplies that. P2's estimate must be
static.

## Kill criteria

**Required.** Any of these ends the work.

1. **The question is empirically empty.** If sweeping the corpus under P0 and P1
   with `DI=Candidate` shows the final chosen layout differing on **at most the
   known `rail` case — including the case where NOTHING differs at all** — then
   no policy machinery is justified: **pin whichever static order wins on the
   differing case** — P0 if nothing differs at all, P1 if the `rail` case
   reproduces and P1 is better there — with a test, add the reasoning to
   RFC-053's decision log, and close this RFC as *rejected — not a real
   contention in practice*.

   "Pin P0" was unconditional in an earlier draft, which would have pinned the
   status quo even in the branch where this RFC's own figures show P1 zero-cost
   and strictly better (0 issues, 261 vs 264) on the one case known to differ.
   A kill criterion should end the *machinery*, not force the worse of two free
   options.

   The zero-difference outcome is called out explicitly because it is a live
   possibility, not a formality: this RFC's own Motivation records that the
   `rail` measurement is currently unreproducible, so the sweep may find no
   differing target whatsoever. Under the earlier wording ("differing on **only**
   the known `rail` case") that outcome did not satisfy the condition — there
   would be no `rail` difference for it to be "only" — and the remedy presupposed
   a sacrifice that would not exist. Third instance of the same defect in this
   document's kill criteria; see the decision log.
2. **P2 cannot be estimated statically.** Concretely: after **at most three
   distinct estimator formulations**, if none of them RANKS the contended
   couplings in the same order as the measured per-coupling outcome on **every**
   target where phase 1 found a contention, drop P2 and P3.

   This criterion depends on phase 1 being extended to produce the ground truth
   it checks against — see Phasing. As originally written it could not be
   evaluated at all: phase 1's instrument reports a binary P0-vs-P1 layout diff
   per target, and a binary winner is not a ranking. Worse, it was unevaluable
   in precisely the case it guards — a target whose optimal assignment differs
   from BOTH static orders is exactly where P2/P3 would earn their cost, and
   exactly where a P0-vs-P1 diff says nothing. Rank agreement, not a correlation
   coefficient — the estimator's only job is to pick which coupling claims first,
   so ordering is the whole of it and magnitude is irrelevant.

   Bounded at three attempts and stated as exact rank agreement because the
   template rejects "if performance is bad" phrasing, and an unbounded "cannot be
   made to" is that with extra steps: there is always one more estimator to try.
   The other four criteria here are numeric or binary; this one was not until
   review said so.
3. **Cost.** If any policy takes end-to-end layout time on the existing corpus
   past **1.45× of the pre-#474 baseline**, it is refused regardless of quality
   gain — i.e. this RFC's own share is capped at roughly **1.18×** on top of what
   #474 already spends.

   Stated against the shared baseline rather than as a local multiplier, because a
   local one does not compose: #474 spent 1.23× of K-DS1-3's 1.5× budget, so
   "1.25× more" would total ~1.54× and blow the budget the criterion exists to
   protect. Same defect as kill criterion 5's percentage, caught the same way — a
   threshold that reads strict while permitting the thing it forbids.
4. **P3 buys nothing over P2.** If optimal matching produces an identical
   assignment to greedy-by-gain on every corpus target, drop P3 and keep P2.
   Do not ship matching machinery for a tie.
5. **The win stays at three entities.** If, after implementing the best policy
   the above allows, the improvement is **5 entities or fewer on every
   individual target AND 20 entities or fewer summed across the corpus**, and it
   resolves no validator issue anywhere, revert it.

   Both bounds, because the conjunction is what protects genuine wins of either
   SHAPE — and an earlier draft had this backwards. Since the bounds are ANDed,
   adding a conjunct can only make the revert fire *less* often, so neither one
   "catches" what the other "tolerates"; each one rescues a different kind of
   real win from being reverted:

   - **per-target alone** would revert 4 entities on each of forty targets — a
     160-entity aggregate win. The aggregate conjunct (160 > 20) saves it.
   - **aggregate alone** would revert a single target gaining 18 — a real
     concentrated win. The per-target conjunct (18 > 5) saves it.

   The revert fires only when the win is small under *both* readings, which is
   the 3-entity case this criterion is named for. A
   tie-break with no measurable consequence should stay an arbitrary tie-break
   with a comment, not become a subsystem.

   Stated in absolute entities, not a percentage, because a percentage cannot
   fire on the case this criterion is named for: 3 of 264 entities is 1.14%, so
   an "under 1%" threshold would leave the three-entity win *passing* the test
   meant to kill it. Caught in review of this RFC — a kill criterion that cannot
   trip on its own titular scenario is worse than none, because it reads as
   protection.

## Verification plan

Per CLAUDE.md's layout-engine protocol, plus one thing specific to this RFC:
**the same-outcome case is the expected case**, so the verification has to be
able to distinguish "policy had no effect" from "policy was not applied."

- **`di_change_surface_sweep`** (added for #474) run under each policy, reporting
  identical / better / regressed per target. This is the primary instrument, and
  it must be extended to print which policy produced each result — otherwise a
  no-op reads the same as a win.
- **`di_candidate_never_degrades_a_succeeding_bus_layout`** must stay green under
  every policy. It is the structural pin, and no claim policy may weaken it.
- **A teeth test**: force a policy that deliberately claims badly, and assert the
  sweep reports the missed improvement. Without this, "0 regressed" is
  unfalsifiable — the same defect
  [`validator-reporting.md`](validator-reporting.md) catalogues.
- **A rate sweep over `iron-stick → rail`, not a named rate.** The verification
  cannot plug in "the rate where the straddle balances" because the record
  disagrees with itself about which rate that is (see Motivation). So phase 1
  sweeps rates under both orders and reports every rate where the chosen layout
  differs — that output IS the missing measurement. Once a differing rate exists,
  assert the row cell at tile level there (belt positions and the
  `input_belt_ys` ↔ fused-spec positional contract), not "it returned `Some`".
- **Sim** any newly-built cell that the corpus starts producing. #473 ran no sim
  because nothing reached the path; if a policy makes it reachable, that
  exemption expires.
- Full suite, clippy `-D warnings` on core and wasm, `tsc`, web tests.

## Phasing

1. **Measure P0 vs P1 across the corpus, sweeping rates on `rail`, and record
   each target's CONTENTION SET.** Cheap, and it may trip kill criterion 1
   immediately — in which case the RFC closes having cost a day and answered the
   question. This phase also has to *establish* the motivating measurement, since
   the recorded rate is contradictory and the scratch flag that produced it was
   never committed.

   Three outputs, not one, and the second and third are what make the later
   criteria evaluable:

   - the P0-vs-P1 layout diff per target (kill criterion 1);
   - **the contention set per target** — which specs were eligible in more than
     one coupling at all. A target with no contention cannot be evidence about
     claim policy, and the binary diff does not distinguish "no contention" from
     "contention that both orders happened to resolve the same way";
   - **the per-coupling outcome for each contended spec**, which is the ground
     truth kill criterion 2 tests an estimator against. Without it that criterion
     is unevaluable exactly where P2/P3 would matter.
2. **P2 gain estimate**, only if (1) shows contention beyond `rail`.
3. **P3 matching**, only if (2) shows greedy is measurably sub-optimal.

Landing (1) alone is a legitimate outcome: the deliverable is a *decided*
tie-break with evidence, not necessarily a new algorithm.

## Decision log

- *2026-07-30 — opened.* Split out of #473, which built the three-input row-cell
  geometry and discovered that the face count was never `rail`'s blocker: the
  dispatcher's claim order was. #473 deliberately did not flip the order, on the
  grounds that a corpus-wide tie-break should not change on 3 entities of
  evidence at one rate. That judgement is endorsed here and is the reason this
  file exists. Note the question only became measurable once #474 defaulted DI
  to `Candidate`, so the ordering of the two PRs matters: this RFC's phase 1
  cannot be run meaningfully before #474 lands.

- *2026-07-30 — two review findings, both fixed, and the second changed the
  RFC's shape.* Kill criterion 5 was stated as "under 1% of entities", which
  cannot fire on the 3-of-264-entity win it is named for (1.14%); restated in
  absolute entities. And the rate producing 261-vs-264 was asked for and turned
  out to be **unrecoverable**: RFC-053 says `rail@1` in one place and, in its own
  coupling table, that the straddle balances only at 5/s and 10/s — with 1/s
  explicitly unbalanced. The measurement came from an uncommitted scratch flag.
  So phase 1's first deliverable is now a reproducible rate sweep rather than a
  policy, and kill criterion 1 cannot be evaluated until that exists. Worth
  noting the failure mode: the original RFC quoted a measurement it could not
  reproduce, and only a reviewer asking "at what rate?" surfaced that the source
  disagreed with itself.

- *2026-07-30 — renumbered RFC-058 → RFC-059.* #506 landed
  [`rfc-058-band-packing.md`](rfc-058-band-packing.md) on `main` while this was in
  review. Parallel sessions claim numbers optimistically and first-to-main wins,
  so this file cedes 058 and takes 059. Note it was verified unclaimed on
  `origin/main` before writing — necessary but not sufficient, since the check and
  the merge are not atomic.

- *2026-07-30 — two further review findings, both fixed.* Kill criterion 3's
  "1.25×" was a LOCAL multiplier on top of #474's 1.23× of a 1.5× budget, so a
  policy sitting exactly at the threshold would total ~1.54× and exceed the budget
  the criterion exists to protect; restated against the shared pre-#474 baseline.
  And Motivation's counter-evidence bullet asserted the `rail@5` no-difference
  claim flatly while a later section identifies that very claim as half of an
  unresolved contradiction; it is now marked recorded-not-established. Both are
  the same shape as the criterion-5 finding: a statement that reads firmer than
  its evidence.

- *2026-07-30 — three more review findings; one of them was mine and one was the
  reviewer's, in the same finding.* The Motivation cited `rfc-053` line ~2249 for
  the `rail@1` claim. That line resolves to unrelated text **on `main`**, and the
  reviewer reported the string had never existed in any revision. Both halves are
  instructive: the citation WAS invalid — the passage lives on #473's unmerged
  branch, so it would not resolve for anyone reading this RFC after it lands —
  and the reviewer's "never existed" is an artifact of the review workflow's
  `fetch-depth: 1` checkout, which cannot see other branches or history. Fixed by
  citing the PR rather than a file-and-line on a moving target.

  Kill criterion 2 was unfalsifiable ("cannot be made to correlate") where the
  other four are numeric; bounded to three estimator attempts judged on exact
  rank agreement. Kill criterion 1 did not cover a sweep that finds NOTHING
  differing — a live possibility given that the `rail` measurement is
  unreproducible — where the literal condition could not be met and the remedy
  presupposed a sacrifice that would not exist.

  That is the **third** kill criterion in this document to fail on an edge of its
  own scenario (after criterion 5's percentage and criterion 3's non-composing
  budget). The pattern is now explicit enough to name: each was written to catch
  the CENTRAL case and silently excluded a boundary — and all three read as
  protection while providing none.

- *2026-07-30 — three further findings; two fixed, one disputed with evidence.*
  Kill criterion 1's remedy said "pin P0" unconditionally, which would have
  pinned the status quo even where this RFC's own figures show P1 zero-cost and
  strictly better on the one differing case — now "pin whichever static order
  wins". Kill criterion 5's bound was ambiguous between per-target and aggregate
  and is now explicitly both. **Disputed:** the review reported that #473 carries
  no correction note for the rate conflict; it does, at `rfc-053` line ~2260 in
  commit `503ca889`. Almost certainly the same `fetch-depth: 1` blind spot as the
  earlier citation finding — the reviewer can read this PR's diff but cannot
  resolve a claim about another branch's state. The sentence was reworded anyway
  to mark #473 as pending rather than assert it as fact, since a cross-PR claim
  that cannot be checked from either side is fragile regardless of who is right.
 The generalisable lesson is in the shape, not the
  count: each was written to catch the central case and silently excluded a
  boundary. The enumeration and its total live together in the final entry
  below, deliberately — this entry predates several of the revisions, and a list
  here could only ever trail them.

- *2026-07-30 — two more, and the second is the funniest defect in this file.*
  Kill criterion 2 checked an estimator against a ranking phase 1 never produced:
  the sweep reports a binary P0-vs-P1 diff per target, and a binary winner is not
  a ranking — so the criterion was unevaluable in exactly the case it guards, a
  target whose optimal assignment differs from both static orders. Phase 1 now
  has three outputs instead of one (diff, contention set, per-coupling outcome),
  and criterion 2 names its dependency on them. And the decision log said "five
  kill criteria revised" when it is five revisions across FOUR criteria —
  criterion 1 twice, criterion 4 never — i.e. a miscount inside the sentence
  naming a pattern of boundary and counting errors.

- *2026-07-30 — the miscount fix was itself miscounted, twice over.* "Five
  revisions across four criteria" undercounted KC2 and KC5, each revised twice —
  and it undercounted KC2 in the very paragraph that was revising KC2. Correct
  tally is **7 across 4**, now given as a per-criterion table so the total is
  derivable from the enumeration beside it rather than asserted on its own. Same
  shape as the `5.02/s` vs `+0.3%` pairing on #505 the same day: each figure
  individually defensible, the relationship between them unchecked. Three
  attempts at one sentence is the evidence that a bare count is the wrong form
  for this claim.

- *2026-07-30 — the revision tally, as a table, placed last on purpose.* It
  counts every entry above it, so it belongs after them: an earlier draft spliced
  it into an entry that predates two of the revisions it tallies, which made a
  chronological record narrate a later finding early. Review caught that, along
  with a "generalisable lesson" list that claimed all seven revisions and
  enumerated five, and a kill-criterion-5 rationale whose conjunction ran
  backwards — ANDed bounds can only narrow when a revert fires, so neither bound
  "catches" what the other "tolerates"; each rescues a different SHAPE of genuine
  win (diffuse: 40x4=160, saved by the aggregate conjunct; concentrated: 1x18,
  saved by the per-target conjunct). Verified by evaluating the predicate on both
  shapes rather than by reading it.

  Every one has the same shape — written to catch the central case, silently
  excluding a boundary:

  1. a percentage that could not fire on its own scenario (KC5)
  2. a cost budget that did not compose with #474's spend (KC3)
  3. a condition that missed the empty-sweep result (KC1)
  4. an unfalsifiable "cannot be made to" (KC2)
  5. a remedy that forced the worse of two free options (KC1)
  6. a criterion with no ground truth to check against (KC2)
  7. an ambiguity between per-target and aggregate readings (KC5)
  8. a both-bounds rationale with the conjunction backwards (KC5)

  Revision tally, per criterion, so the total is derivable rather than asserted:

  | criterion | revisions | what |
  |---|---:|---|
  | KC1 | 2 | missed the empty-sweep result; then "pin P0" forced the worse of two free options |
  | KC2 | 2 | unfalsifiable "cannot be made to"; then no ground truth to check against |
  | KC3 | 1 | local cost multiplier that did not compose with #474's spend |
  | KC4 | 0 | — |
  | KC5 | 3 | percentage that could not fire on its own case; then per-target vs aggregate ambiguity; then the conjunction rationale backwards |

  **8 revisions across 4 criteria.** Stated as a table because the prose form was
  miscounted three times: first as "five kill criteria" (it was never five
  criteria); then as "five revisions across four criteria", which undercounted
  KC2 in the same paragraph that was revising KC2; then as seven, which
  undercounted the KC5 revision made in the very commit that introduced this
  table. Each miss has the same cause — a total re-derived from memory of a list
  not in front of me. A count is checkable only against an enumeration beside
  it; alone it is a number that looks careful.
