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

## Outcome — decided, 2026-07-31

**The tie-break stays `Upstream` (P0). The measured-better policy is built,
reachable, and deliberately not the default.** P2 and P3 are dropped.

This is the second outcome the Summary contemplated — "keep upstream-first, pin
it with a test, and write down why" — and it is reached on evidence rather than
by declining to act.

### What the measurement said

Over every producible item at 1, 5 and 20 per second across **three machine
tiers**, under production `DirectInsertion::Candidate`:

| | |
|---|---:|
| targets where a spec is contended | 179 |
| of those, both fixed arms ship the same layout | 169 |
| a two-arm SEARCH beats fixed **upstream** | **6** |
| the search beats fixed **downstream** | **2** |
| the search worse than either fixed arm | **0** |
| a per-target assignment beating the search | **0** |
| no layout under some arm (native refuses too — not a claim-order effect) | 2 |

The eight differing targets:

| target | tier | upstream | downstream |
|---|---|---:|---:|
| `display-panel@1` | am1 | 221 | **202** |
| `land-mine@1` | am1 | 326 | **317** |
| `small-electric-pole@5` | am1 | **126** | 163 |
| `big-electric-pole@1` | am2 | 1146 | **1127** |
| `land-mine@1` | am2 | 312 | **296** |
| `medium-electric-pole@5` | am2 | 2351 (2 warnings) | **2340** (0) |
| `small-electric-pole@5` | am2 | **109** | 136 |
| `land-mine@1` | am3 | 294 | **282** |

The direction FLIPS: `small-electric-pole` wants upstream-first, everything else
wants downstream. So no fixed order dominates, and the natural conclusion is to
search both arms and keep the better — never worse than either by construction,
better on 8 targets, at a constant two builds rather than the unbounded
per-coupling cost Design rejects.

That conclusion was implemented, defaulted, pinned, and then falsified.

### Why it is not the default

**A headless-Factorio run on `display-panel@1` / am1, controlled against the
status quo:**

| arm | ships | validator | sim |
|---|---|---|---|
| `Upstream` (status quo) | native, 221 entities | 0 errors, 0 warnings | **PASS** — 1.00/s produced, 1.01/s delivered, converged |
| `Search` (picks downstream) | DI, 202 entities | 0 errors, 0 warnings | **FAIL** — 0.00/s, jammed (`full_output: 10`), never converged |

Same harness, same warmup (288000 ticks), same export path, same recipe. The
only difference is which coupling fused: `Upstream`'s arm claims
`copper-plate → copper-cable` as a stacked cell and validates with 3 errors, so
the DI candidate is refused and native ships. `Search` prefers the arm that
claims `copper-cable → electronic-circuit` as a **row** cell, which validates
perfectly clean and does not work.

**The premise that made this RFC safe is the one that broke.** Design's
load-bearing constraint reads: "a claim policy cannot make anything *worse* — the
worst a bad policy does is leave a better cell unbuilt", because DI may only
displace native on a strict improvement. That is true of the *validator's*
judgement and the validator cannot see this failure. So the risk was never
bounded to "missed upside"; it was bounded to "missed upside, as far as 36
functional checks can tell".

Shipping the search would therefore trade 8 measured-denser layouts for at least
one working factory, and the corpus offers no way to tell which of the other 7 are
in the same state — sim is per-target and off the critical path.

`DiClaimOrder::Search` is kept live rather than deleted: it is correct modulo one
cell, and re-deriving the measurement means the three-tier sweep again. The pin
`di_claim_order_status_quo_ships_and_search_stays_reachable` asserts both halves
— that the default ships the sim-verified layout, and that `Search` still picks
the better arm where nothing blocks it.

### What this RFC delivered

- The tie-break is now a **decision with evidence** rather than a loop direction,
  which was the stated goal.
- **P2 and P3 are dropped** on a stronger finding than KC4 asks for: pinning each
  contended coupling to claim first and rebuilding, no assignment beats the
  two-arm search anywhere. The per-target optimum is always one of the two static
  orders, so there is nothing for a gain estimator to estimate and no matching to
  solve.
- A **validator blind spot with a named instance**: `di-row:copper-cable:
  electronic-circuit` on am1 is clean and physically jammed. Tracked as
  [#520](https://github.com/storkme/spaghettio/issues/520); an RFC-053/#474 defect
  that RFC-059 exposed rather than caused.
- Reusable instruments: `DiClaimOrder::{Upstream, Downstream, Search, Pinned}`,
  three trace events, and `probe_di_claim_order_shipped_corpus_verdict`, which
  re-derives every number above in ~10 minutes.

### Re-measured after #520 (2026-07-31) — the answer moved

The blocking defect was in the **validator**, not the cell geometry, and fixing
it changed this RFC's measurement.

`check_belt_flow_reachability` asked its question per MACHINE over the union of a
machine's input belts, so one fed input masked a starved one, and it did not
model belt-to-belt lift inserters at all. With both fixed, DI's never-worse gate
sees the jammed cell and declines it on its own — `display-panel@1` am1 ships the
sim-verified native layout under **both** the default and `Search`.

The corpus re-measurement is the consequential part:

| | before #520's fix | after |
|---|---:|---:|
| search beats fixed **upstream** | 6 | **6** |
| search beats fixed **downstream** | 2 | **0** |
| search worse than either arm | 0 | 0 |

**Downstream-first now dominates.** The two targets where it looked strictly
worse were `small-electric-pole@5` on am1 and am2 — and those are exactly the
layouts where UPSTREAM shipped a validator-clean factory measured at **2.52/s
against a planned 5.00/s**. Downstream was never worse there; it was better, and
the instrument could not tell. The Outcome table above records the pre-fix
numbers, and they were honest measurements of what the engine could then see.

Two consequences:

- The two-arm `Search` is now **equivalent to a fixed `Downstream`** on every
  corpus target, so it buys nothing over flipping the default — KC4's own logic
  ("do not ship matching machinery for a tie") applied one level up.
- **The flip is not made here.** It needs sim verification of the targets it
  improves first. The whole content of #520 is that validator-clean is not
  evidence a layout works, and shipping a policy change on the strength of a
  re-run of the same validator would repeat the mistake this RFC just made.

## Motivation

> **Superseded by measurement (2026-07-31).** The `rail` case below does **not
> contend**: at 1, 5 and 10 per second from `iron-ore` the census reports zero
> contention under both orders and identical layouts (269 entities at `rail@1` —
> neither the 261 nor the 264 quoted here). `rail`'s couplings die at
> **buildability**, before the contention check they were said to lose. The
> section is kept as written because the reasoning it prompted was sound and the
> RFC's question survived its motivating example being wrong — which is the case
> for gating on a corpus sweep rather than on the example that started the work.

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

| Policy | Rule | Cost | Outcome |
|---|---|---|---|
| **P0 — upstream-first** | status quo; topological order claims | zero | **kept as default** |
| **P1 — downstream-first** | reverse the walk | zero | measured; better on 6 targets, worse on 2 |
| **P2 — greedy by gain** | score each candidate coupling in isolation, claim in descending order of predicted gain | one extra pass per coupling | dropped, unbuilt |
| **P3 — optimal matching** | max-weight matching on the SPEC graph, couplings as EDGES — general, not bipartite | small; contention sets are tiny | dropped, unbuilt |

A fifth policy emerged from the measurement and is not in this table because
nobody proposed it: **search both static arms and keep the better**
(`DiClaimOrder::Search`). It is implemented and measured strictly better than
either fixed arm, and it is not the default — see Outcome for why.

The rest of this section is the design as circulated, kept because P3's
formulation was corrected in review and that correction is worth preserving for
whoever reopens this. **The matching was never implemented** — see Outcome.

**Predicted shape of the diff.** The dispatcher's claim loop
(`bus/di_cell.rs`, the walk that produces fused specs) gains a policy parameter
rather than an inverted loop. P0/P1 are orderings of the same loop. P2 needs a
per-coupling gain estimate that does **not** require building the layout —
otherwise the cost is a full layout per candidate. P3 needs the contention graph
materialised, which is only worth it if P2's greedy proves to be measurably
sub-optimal.

**The matching is over the SPEC graph with couplings as EDGES**, not a
coupling/spec bipartite graph — an earlier draft said the latter and it does not
enforce the constraint this policy exists for. A coupling claims **two** specs
(`iron-plate → iron-stick` claims both; `placer.rs` inserts two indices into
`claimed`), so a bipartite matching only guarantees each coupling gets ≤1 spec.
On this RFC's own motivating case it selects **both** `C1={iron-plate,
iron-stick}` and `C2={iron-stick,rail}` — cardinality 2 beats either singleton —
leaving `iron-stick` claimed by both. Modelling specs as vertices and couplings
as edges makes a matching vertex-disjoint, which *is* spec-disjointness, by
construction. That graph admits odd cycles (a recipe fan such as iron-plate /
iron-gear-wheel / transport-belt forms a triangle), so it needs general
matching — Blossom, not Hungarian. Caught in review; an implementer following the
earlier wording literally would have built a solver able to emit infeasible
double-claimed assignments.

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

**Verdicts (2026-07-31).** None tripped; the work ended by answering the
question, not by a criterion firing.

| # | verdict | measured |
|---|---|---|
| 1 | does not trip | 179 targets contend, not 0 |
| 2 | **never evaluated** | no estimator was written — P2 was dropped on the finding below, so there was nothing to rank |
| 3 | **not reached** | the search would have cost one extra DI-candidate build; it did not ship, so the corpus pays nothing |
| 4 | subsumed | P3 buys nothing over P2, but neither buys anything over the two-arm search, which is a stronger negative than KC4 asks for |
| 5 | **not reached** | it bounds the improvement of a SHIPPED policy, and nothing shipped. Had `Search` shipped it would not have tripped: 19 entities best single, 86 summed, 2 validator warnings resolved |

KC2's "never evaluated" is the honest entry and it is not a dodge: the criterion
bounds how long P2 may be attempted before being dropped, and P2 was dropped
before the first attempt on evidence the criterion does not cover. A criterion
that never fires because its subject was abandoned earlier is a criterion that
was correctly scoped to a phase that did not run.

KC5 is shown rather than asserted because it was twice nearly decisive and is
now moot. A narrow, am3-only version of the measurement put it on a knife edge
(12 entities on one target, one above the per-target bound, resolving nothing);
widening the sweep moved every component well clear; and then the sim removed the
shipped policy the criterion measures. Recorded for whoever unblocks it:

| KC5 bound | trips if | measured |
|---|---|---:|
| per-target improvement | ≤ 5 entities | **19** (`display-panel@1`, `big-electric-pole@1`) |
| corpus-summed improvement | ≤ 20 entities | **86** |
| validator issues resolved | none, anywhere | **2 warnings** (`medium-electric-pole@5` am2) |

All three conjuncts must hold for KC5 to revert; none does. These are the numbers
the policy would deliver once the blocking cell is fixed.

1. **The question is empirically empty.** If phase 1 finds **every target's
   contention set empty** (output 2), close this RFC as *rejected — not a real
   contention in practice*: pin P0 with a test, and record the reasoning in
   RFC-053's decision log.

   **An empty contention set is sufficient on its own, and that is a stronger
   statement than it looks.** If no spec was ever eligible in two couplings,
   claim ORDER cannot change which couplings claim — every eligible coupling
   claims regardless of the walk direction. So the layout diff is *entailed* to
   be empty; it is not an independent condition. An earlier draft ANDed the two
   and offered "pin P1 if the `rail` case reproduces and P1 is better there",
   which is unreachable: inside a trip condition requiring zero contention, no
   target can differ. Caught in review, and the dead branch is the evidence that
   the conjunction was redundant rather than cautious.

   **Use the diff as a CONSISTENCY CHECK, not a conjunct.** Contention-empty
   entails diff-empty, so observing empty contention *together with* a
   non-empty P0-vs-P1 diff means the instrument is wrong — a coupling decision
   is being made somewhere the census does not see. Phase 1 should assert that
   implication and fail loudly if it breaks, rather than quietly reporting both
   numbers.

   **Why the contention set and not the diff.** A diff alone cannot distinguish
   "no spec was ever contended" from "specs were contended and both static
   orders happened to resolve them the same way" — and KC2 says of that same
   instrument that it is unevaluable exactly where a target's optimum differs
   from BOTH P0 and P1, which is where P2/P3 would earn their cost. A diff-only
   KC1 could therefore close this RFC precisely in the scenario it exists to
   investigate.

   The empty outcome is a live possibility, not a formality: Motivation records
   the `rail` measurement as unreproducible, and the first phase-1 census (10
   targets, `DirectInsertion::Forced`) found **0 contention anywhere**, with
   `rail`'s three couplings failing at buildability rather than contention.

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

### What was run (2026-07-31)

| plan item | done | note |
|---|---|---|
| change-surface sweep, policy named | yes | `di_change_surface_sweep` now prints the live claim order in its header |
| never-degrades pin stays green | yes | `di_candidate_never_degrades_a_succeeding_bus_layout`, unchanged |
| teeth test | yes | `di_claim_order_status_quo_ships_and_search_stays_reachable`; asserts the sim-verified layout ships AND that the blocked policy still picks the better arm |
| rate sweep instead of a named rate | yes | 1/5/20 per second across every producible item and three machine tiers, plus `rail` at 1/5/10 from `iron-ore` |
| tile-level assertion at a differing rate | **no** | see below |
| sim the newly-built cell | yes — **and it changed the outcome** | `land-mine@1` is unmeasurable (fluid boundary, uncalibrated harness path); `display-panel@1` on am1 showed the searched policy shipping a validator-clean factory that produces 0/s |
| suite, clippy, wasm | yes | 1047 pass / 0 fail (single invocation, post-rebase onto RFC-058 #516/#517/#518); `cargo clippy --workspace -D warnings` clean; wasm-pack build clean |

**The tile-level assertion was not written, deliberately.** The plan asked for it
"once a differing rate exists", expecting the difference to be a row cell whose
`input_belt_ys` ↔ fused-spec contract needed pinning. The difference that
materialised is not that: on every differing target the claim order changes
**which candidate wins the decomposition search** — commonly native under one arm
and DI under the other — so what needs pinning is the shipped layout's identity,
not a cell's internal geometry.
`di_claim_order_status_quo_ships_and_search_stays_reachable` asserts that: on
`display-panel@1` the default must ship the sim-verified 221-entity layout, and
the rejected variant must still look better on every signal the engine has, so a
future scoring change cannot silently re-select it. It then checks `Search` on two
targets that disagree about which arm wins (`small-electric-pole@5` am1 wants
upstream, `land-mine@1` am3 wants downstream), which is what stops the blocked
policy rotting into unreachable code. A tile-level assertion on the DI cell would
pin geometry both arms agree on, which is the wrong invariant.

## Phasing

1. **Measure P0 vs P1 across the corpus, sweeping rates on `rail`, and record
   each target's CONTENTION SET.** Cheap, and it may trip kill criterion 1
   immediately — in which case the RFC closes having cost a day and answered the
   question. This phase also has to *establish* the motivating measurement, since
   the recorded rate is contradictory and the scratch flag that produced it was
   never committed.

   Three outputs, not one, and the second is what makes kill criterion 1
   evaluable while the third does the same for kill criterion 2:

   - the P0-vs-P1 layout diff per target — a **consistency check** for kill
     criterion 1, not its trigger. Contention-empty entails diff-empty, so an
     empty contention set beside a non-empty diff means the census is missing a
     coupling decision, and phase 1 must fail loudly rather than report both;
   - **the contention set per target** — which specs were eligible in more than
     one coupling at all. This is **kill criterion 1's sole trigger**. A target
     with no contention cannot be evidence about claim policy, and the binary
     diff does not distinguish "no contention" from "contention that both orders
     happened to resolve the same way";
   - **the per-coupling outcome for each contended spec**, which is the ground
     truth kill criterion 2 tests an estimator against. Without it that criterion
     is unevaluable exactly where P2/P3 would matter.
2. **P2 gain estimate**, only if (1) shows contention beyond `rail`.
3. **P3 matching**, only if (2) shows greedy is measurably sub-optimal.

Landing (1) alone is a legitimate outcome: the deliverable is a *decided*
tie-break with evidence, not necessarily a new algorithm.

### What actually happened

(1) ran and produced all three outputs. (2) and (3) were **not built**, and the
reason is a finding rather than a budget call — see Outcome: no target's best
reachable assignment differs from both static orders, so a per-target policy has
nothing to find. The gate on (2) was "contention beyond `rail`", which *is*
satisfied — 57 targets contend, none of them `rail` — but satisfying a gate is
permission to look, not an obligation to build, and looking is what phase 1's
third output was for.

Phase 1's own gate on (2) turned out to be the wrong test. It asked whether
contention exists; the question that decides P2 is whether the OPTIMUM is
reachable by a fixed walk. Those come apart exactly here: contention is
widespread and the optimum is static everywhere.

## Decision log

- *2026-07-30 — opened.* Split out of #473, which built the three-input row-cell
  geometry and discovered the face count was never `rail`'s blocker: the
  dispatcher's claim order was. #473 deliberately did not flip that order, on the
  grounds that a corpus-wide tie-break should not change on 3 entities of evidence
  at one rate. That judgement is endorsed here and is why this file exists. The
  question only became measurable once #474 defaulted DI to `Candidate`, so phase
  1 could not be run meaningfully before #474 landed.

- *2026-07-30 — the motivating measurement is not reproducible, and that raised
  phase 1's value.* Asked which rate produces 261-vs-264, the record contradicts
  itself: RFC-053 says `rail@1` in one place and, in its own coupling table, that
  the straddle balances only at 5/s and 10/s with 1/s explicitly unbalanced. The
  measurement came from an uncommitted scratch flag. So phase 1's first
  deliverable is a reproducible rate sweep rather than a policy, and kill
  criterion 1 cannot be evaluated until it exists. Corrected at source in #473.

- *2026-07-30 — renumbered RFC-058 → RFC-059.* #506 landed
  [`rfc-058-band-packing.md`](rfc-058-band-packing.md) on `main` mid-review. 058
  was verified unclaimed on `origin/main` before writing — necessary but not
  sufficient, since the check and the merge are not atomic, and first-to-main
  wins.

- *2026-07-30 — P3's matching formulation was wrong; a correctness bug, not
  wording.* P3 read "max-weight matching over the coupling/spec bipartite graph".
  That does not enforce the pairwise spec-disjointness the Design section
  requires, because a coupling claims TWO specs (`placer.rs` inserts two indices
  into its `claimed` set), so a bipartite matching only guarantees each coupling
  gets at most one. On this RFC's own motivating case — `C1 = {iron-plate,
  iron-stick}`, `C2 = {iron-stick, rail}` — it selects BOTH (cardinality 2 beats
  either singleton), leaving `iron-stick` claimed twice. Corrected to
  specs-as-vertices with couplings as EDGES, where a matching is vertex-disjoint
  and therefore spec-disjoint by construction; that graph admits odd cycles (a
  recipe fan such as iron-plate / iron-gear-wheel / transport-belt is a triangle),
  so it needs general matching — Blossom, not Hungarian. Consequential because P3
  is the baseline KC4 measures P2 against, and an implementer following the old
  wording would have built a solver able to emit infeasible double-claimed
  assignments. Verified by evaluating the counterexample, not by reading the
  definition.

- *2026-07-30 — KC1 could have killed the RFC in the case KC2 calls valuable.*
  KC1 tripped on a binary P0-vs-P1 layout diff alone, which cannot distinguish
  "nothing was contended" from "specs were contended and two arbitrary orders
  coincidentally agreed" — while KC2 says of that same instrument that it is
  unevaluable exactly where the optimum differs from both static orders, which is
  where P2/P3 would earn their cost. KC1 now additionally requires every target's
  contention set to be empty (phase 1 output 2).

- *2026-07-30 — review tightened every kill criterion except KC4.* Ten revisions
  across four criteria, each one a threshold or condition that read as protection
  while permitting what it forbade:

  | criterion | revisions | what was wrong |
  |---|---:|---|
  | KC1 | 4 | missed the empty-sweep result; "pin P0" forced the worse of two free options; a diff-only trip condition that contradicted KC2; then the conjunct fixing that made its own P1 branch unreachable |
  | KC2 | 2 | unfalsifiable "cannot be made to"; no ground truth to check against |
  | KC3 | 1 | a local cost multiplier that did not compose with #474's spend |
  | KC4 | 0 | — |
  | KC5 | 3 | a percentage that could not fire on its own case; per-target vs aggregate ambiguity; a both-bounds rationale with the conjunction backwards |

  The generalisable half: most were written to catch the central case and
  silently excluded a boundary; two were different — a criterion that could never
  fire at all, and a rationale whose prose was wrong while its trip condition was
  right. Kill criteria are worth stating as tables or predicates rather than
  sentences, and worth testing against their own named scenario before shipping.

  (Earlier drafts of this log narrated each editing slip in the tally itself,
  which added surface faster than it removed error. Trimmed on 2026-07-30: a
  decision log records calls made, not typos corrected.)

- *2026-07-30 — KC1 does not trip, and the sample that said it would was
  falsified.* The first phase-1 census (15 hand-picked targets) reported **zero**
  contention anywhere and would have closed this RFC as *not a real contention in
  practice*. The corpus sweep — every producible item at 1/5/20 per second, 714
  target/rate pairs with couplings, 3296 couplings — found **57 contended
  targets**. The sample was not merely small, it was **biased in the exact
  direction that hides the effect**: it was picked around recipes where DI
  visibly claims, and contention lives on HUB specs consumed by several recipes
  (`electronic-circuit` 7 targets, `steel-plate` 5, `engine-unit` 4, `rocket` 3),
  which is a different set. KC1's insistence on sweeping rather than sampling is
  what caught it.

- *2026-07-30 — the corpus sweep runs `place_rows`, not `build_bus_layout`.* The
  claim loop is the entire order-dependent decision, so routing, poles and
  validation are pure cost for a contention census. The full-layout version ran
  39 minutes without finishing and was abandoned as a **bad instrument, not a
  slow one**. Full layouts are then built only for the contended targets, which
  is sound on KC1's own entailment: a target with no contention cannot differ.

- *2026-07-31 — the motivating case does not contend at all.* `rail` at 1, 5 and
  10 per second from `iron-ore` reports **zero** contention under both orders and
  builds byte-identically — 269 entities at `rail@1`, matching neither the 261
  nor the 264 the record disputes. So the earlier finding is not just
  unreproducible (Motivation), it describes a state this corpus does not reach:
  `rail`'s three couplings die at **buildability**, before the contention check
  they were said to lose. The RFC's question survives its own motivating example
  being wrong, which is the argument for gating on a corpus sweep rather than on
  the case that prompted the work.

- *2026-07-31 — `Forced` overstates the win by two orders of magnitude; the
  shipped number is measured under `Candidate`.* Under `DirectInsertion::Forced`
  P1 clears **every validation error on five targets** (P0: 3 errors each; P1: 0
  errors, 0 warnings) — a spectacular-looking result. It is not the win.
  `DirectInsertionCandidate` refuses its own layout on any error, so those
  P0 layouts were never shipped to anyone; production runs `Candidate`, where the
  same comparison is **12 entities on one target**. Reporting the `Forced`
  figure would have been accurate about a layout nobody receives — the same
  category error as a validator count that no longer discriminates
  ([`validator-reporting.md`](validator-reporting.md)), arrived at from the
  opposite direction. Every headline number in Outcome is a `Candidate` number
  for this reason.

- *2026-07-31 — a one-machine-tier sweep said "flip to P1"; widening it to three
  said "neither order wins". The narrow instrument was wrong twice in this RFC,
  the same way.* On `assembling-machine-3` alone, downstream-first was strictly
  better on 1 of 57 contended targets and worse on **0** — a free flip, and it
  was implemented and pinned as the decision. Sweeping am1 and am2 as well turned
  that into **6 better, 2 worse**: `small-electric-pole@5` regresses 126 → 163
  entities on am1 and 109 → 136 on am2, because am1/am2 give the same recipe a
  different ROW STRUCTURE and the claim order acts on exactly that. Shipping the
  am3-only answer would have regressed two working targets to gain five others.

  This is the second time in one RFC that a narrower instrument returned a
  confident answer in the direction of "there is a clean winner" — the first was
  the 15-target sample reporting zero contention. Both were cheap to widen and
  neither error was detectable from inside the narrow run. **The generalisable
  rule: when a sweep reports a clean sweep, widen an axis before believing it.**

- *2026-07-31 — a two-arm SEARCH was built, measured better than either fixed
  order, and held back on sim evidence; P2 and P3 dropped.* Since neither static
  order dominates, `DirectInsertionCandidate` gained the ability to build both
  and keep the better, ties to upstream so unaffected targets stay
  bit-identical. It resolves all 8 differing targets optimally and is worse than
  a fixed arm on none — asserted by measurement, not by the fact that the picker
  picks the better arm, because the picker orders on (validator warnings, layout
  warnings, entities) while `di_choice` gates component-wise against native, and
  two orderings that look aligned can disagree. It is **not the default**; the
  sim entry below is why.

  P2/P3 are dropped on a stronger finding than KC4 asks for: pinning each
  contended coupling to claim first and building the result, **no assignment
  beats the search on any target**. A per-target policy can only pay where the
  optimum is unreachable by a fixed walk, and here it never is — which also makes
  two arms exhaustive rather than a heuristic.

  Reversing Design's "rejected alternative" needed an argument, since that
  section refuses score-driven claiming that builds each variant. The refusal
  stands as written: it costs a build **per candidate coupling**, unbounded in
  the coupling count. Two arms is a constant factor on solves that have couplings
  at all, which is the same shape of cost #474 already accepted for DI itself.

- *2026-07-31 — the default flip to `Downstream` is DEFERRED, not declined.*
  Fixing #520's validator blind spot changed this RFC's measurement: the search
  now beats fixed upstream on 6 targets and fixed downstream on **0**, so
  downstream-first dominates and the two-arm `Search` is equivalent to simply
  flipping the default — KC4's "do not ship machinery for a tie", one level up.
  The two targets that made downstream look worse (`small-electric-pole@5` on am1
  and am2) were the ones where UPSTREAM shipped a factory measured at 2.52/s
  against a planned 5.00/s; downstream was never worse there, and the instrument
  could not tell.

  Not flipped in the same change, and the reason is this RFC's own lesson rather
  than caution: the evidence for the flip is a re-run of the validator, and #520
  is the demonstration that a clean validator is not evidence a layout works.
  The flip needs sim verification of the targets it improves —
  `display-panel@1`, `land-mine@1` at three tiers, `big-electric-pole@1`,
  `medium-electric-pole@5` — of which only `display-panel@1` has been simmed.
  Recorded here rather than only in Outcome because RFC-059 owns
  `DiClaimOrder::default()`, so deferring a change to it is a call made on this
  RFC's subject.

- *2026-07-31 — `DiClaimOrder::Pinned` kept although P2/P3 were dropped.* It is
  measurement machinery for a policy that will not ship, which normally argues
  for deleting it. Kept because it is the instrument that produced the negative,
  and this RFC exists **because** the measurement that motivated it was taken
  with an uncommitted scratch flag and could not be re-run. A negative result
  with no executable form would repeat exactly that mistake; `Pinned` is what
  lets a future reader re-derive "no per-target assignment wins" instead of
  trusting this paragraph. It is also the mechanism P3 would apply through if the
  corpus ever moves.

  The same argument keeps `Upstream` and `Downstream` as public arms alongside
  `Search`: without them the corpus sweep could not measure the search against
  the pre-RFC status quo, and "the search picks the better arm, so it cannot be
  worse" would be an argument rather than a measurement. (Written when `Search`
  was briefly the default; it is not — see below — and the arms matter more now,
  since `Upstream` is what ships.)

- *2026-07-31 — the sim falsified the RFC's safety premise, and the policy was
  held back rather than shipped.* The verification plan requires simming any
  newly-built cell the corpus starts producing, and that requirement is the only
  reason this RFC did not ship a regression.

  `land-mine@1` was the first candidate and is **unmeasurable**: it needs water
  and crude-oil, and the harness prints its own caveat that infinity-pipe
  feed/void paths are UNCALIBRATED (RFC-050 Phase 1). Both arms returned a flat
  0/s, which measures the harness rather than the layout — recorded rather than
  quietly re-rolled, because a FAIL that is really a harness limitation is exactly
  the artifact class [`sim-harness-forensics.md`](sim-harness-forensics.md) exists
  to stop being read as a layout defect.

  `display-panel@1` on am1 has no fluid boundary and gave a clean controlled
  answer. The status quo ships native and measures **1.00/s produced, 1.01/s
  delivered, converged**; the searched policy ships a 202-entity DI layout and
  measures **0.00/s, jammed at `full_output: 10`, never converged**. Both validate
  with zero errors and zero warnings. The broken cell is
  `di-row:copper-cable:electronic-circuit` — which RFC-053 records simming at
  101.3%, but at a different machine tier, so the pair is not broken everywhere.

  **What this refutes is Design's load-bearing constraint**, not merely a number:
  "a claim policy cannot make anything worse — the worst a bad policy does is
  leave a better cell unbuilt" is a statement about the VALIDATOR's judgement, and
  the validator is blind here. So `Search` stays built and non-default until the
  cell is fixed and re-simmed. Eight entity-denser layouts are not worth one
  factory that stops, and the corpus cannot say which of the other seven are in
  the same state — sim is per-target and off the critical path.

  The generalisable half, and it is uncomfortable: **every "never worse" claim in
  this project is implicitly "never worse as far as 36 functional checks can
  tell."** #474's DI gate, RFC-057's fold and #511's compaction transaction all
  rest on that substitution. This is the first time it has been caught paying
  out.
