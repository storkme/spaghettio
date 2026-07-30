# RFC-058: Direct-insertion coupling assignment

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
- The measured win is **3 entities at one rate**. At `rail@5` the straddle
  doesn't balance and both orders build the same stacked cell.

Three entities is not a mandate to change a corpus-wide tie-break. The reason to
do this work is that **nobody knows what the right rule is**, not that the
current one is demonstrably costly.

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
   with `DI=Candidate` shows the final chosen layout differing on **only** the
   known `rail` case, then no policy machinery is justified: pin P0 with a test
   naming `rail` as the known sacrifice, add the reasoning to RFC-053's decision
   log, and close this RFC as *rejected — not a real contention in practice*.
2. **P2 cannot be estimated statically.** If a per-coupling gain estimate that
   does not build a layout cannot be made to correlate with measured outcome on
   the `rail` case plus at least two others, drop P2 and P3 — the matching
   objective is unknowable at claim time, and the honest answer is a pinned
   static order.
3. **Cost.** If any policy adds more than **1.25×** to end-to-end layout time on
   the existing corpus, it is refused regardless of quality gain. #474 already
   spent 1.23× of K-DS1-3's 1.5× budget; there is not another 1.5× available.
4. **P3 buys nothing over P2.** If optimal matching produces an identical
   assignment to greedy-by-gain on every corpus target, drop P3 and keep P2.
   Do not ship matching machinery for a tie.
5. **The win stays at three entities.** If, after implementing the best policy
   the above allows, the total measured improvement across the corpus is under
   **1% of entities** on every target and resolves no validator issue anywhere,
   revert it. A tie-break with no measurable consequence should stay an
   arbitrary tie-break with a comment, not become a subsystem.

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
- **`rail@5` and the rate where the straddle balances**: assert the row cell is
  built under the chosen policy, at tile level (belt positions and the
  `input_belt_ys` ↔ fused-spec positional contract), not "it returned `Some`".
- **Sim** any newly-built cell that the corpus starts producing. #473 ran no sim
  because nothing reached the path; if a policy makes it reachable, that
  exemption expires.
- Full suite, clippy `-D warnings` on core and wasm, `tsc`, web tests.

## Phasing

1. **Measure P0 vs P1 across the corpus.** Cheap, and it may trip kill
   criterion 1 immediately — in which case the RFC closes having cost a day and
   answered the question.
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
