# Retrospective: the factory-compaction arc ("the original spaghetti goal")

Produced 2026-07-31 by a research pass over the compaction-lineage RFC
decision logs (RFC-053 through RFC-061) and the issue threads they
reference (#135, #456, #507, #519, #520, #526). This is the evidence base
for [`rfc-063-compaction-primitives.md`](rfc-063-compaction-primitives.md)
— read that RFC for the funded next steps; this document is the record of
how the arc got there. Citations were spot-checked against the source
docs before this was committed; one drifted characterization (RFC-060's
sim-verified flips) is corrected below from what an earlier draft claimed.

## Timeline + scoreboard

Prehistory (2026-04-11): #135 measured on stress_electronic_circuit_30s_from_ore that 34 of 37 adjacent-row bands have a 0-tile gap — the waste is 3 balancer bands of 15/10/8 tiles (150–280 entities each), not inter-row slack. Prediction never cashed.
2026-07-25: #456 filed (spaghettifier post-pass; "density and disorder are the same objective"); RFC-054 fast meter declared hard prerequisite.
2026-07-26: RFC-056 (folded chains) rejected at its own admission gate same day (only chem5raw cleared; pu4raw +11.1% distance/+78.7% critical path). RFC-055 (compact linear chains) selected — weighted distance −16.3–39.6% but PHYSICAL belts only −10.1–17.3% on three fixtures and +8.5% on USP; Factorio gates never adjudicated; never shipped; later superseded by RFC-057.
2026-07-26→30: RFC-057 (topology-preserving dense repacking), registry still "Active". Trajectory: machine-bbox bound −50.5–68.6% → rigid islands −41–63% → conservative undergroundify post-pass belts −36.8–63.8%, tiles −20.5–54.0%, mil5 meter throughput IMPROVED 1.73→2.16/s → first materialized candidates +38% to +250% bbox vs source (5/6 fixtures; logistics 6–8× machinery; sci1-ore manifold spends 375 logistics entities where the bus spends 201). Recorded-and-never-built recommendation: single-lane shared trunks; "the 2D placement itself is vindicated". Snake folding refused as density lever (~20% ceiling) but succeeded as shape: chain-mil5ore 553×32 → 153×141 at 5.016/s vs 5.00 planned in Factorio, +26% entities (PR #500). Only compact_layout (opt-in, default false) shipped.
2026-07-30→31: RFC-058 (band packing) KILLED in two days, by design. Phase 0: KC2 36.8% vs 30% bar. Phase 3 spike: −35.9% vs −33.0%. Phase 4 real planner: −44.0% (corrupt) → −34.6% (tree router) → −27.0% (legal, faithful instrument; instrument itself corrected −31.1→−23.3→−27.0 under review). Six points below bar with KC3 parity still distant; kill criterion 1 fired; "stop; do not re-tune."
Shipped density (all decomposition-search candidates, none post-pass): RFC-053 DI (−36% flagship cell; spf@1 2684→1904 entities with 33→0 warnings; 5 flips 0 regressions, all sim PASS). RFC-060 HorizontalStack (headline flips are correctness, not density: native 0.00/s deadlocks on ac5/ac7/pu3 while the candidate delivers non-zero — below plan due to the #519 lane-flux gap, but the winner key is not lying; pure-density arm DROPPED for ≤5% shaves). #520 cautionary tale: validator-clean DI layout, 37 entities denser, 2.52/s vs 5.00 planned ("density beat correctness because correctness was invisible"; flagship cable→EC cell claimed on 68 targets ships on 0 — #526). RFC-061 phase 1: ac@5 75.0%→96.8% of plan, ac@7 85.7%→95.4% (real templates now default-path). RFC-054 meter: ~20× cheaper than headless but KC1 TRIPPED (EC family 0.3–0.6pp agreement; military family wrong by 57.8pp; fluids −100% on 7/12 configs).

## What worked

Mechanisms that change WHAT gets built (DI cells, HS) paid; the only post-pass that improved both axes at once was the topology-preserving undergroundify (belts −37–64% AND meter throughput up). Folding works as shape transform. RFC-058's band census is durable knowledge (bands ~5 tall; ragged-right margin 38.4% of bbox; belts 39.6% of occupied tiles vs machines 38.4%). Process: kill-criteria machinery worked and improved — 056 died day one; 057 burned ~5 days (cheap phases first, premise died in the expensive one); 058 named that failure, reordered phases around it, ran open-to-killed in two days with the instrument attacked before the kill number froze. Compaction hardened the validator as a by-product (boundary-record false pass PR #482; pole-connectivity aggregation; #519/#520 family).

## What made progress hard

Wall 1: logistics overhead eats machine-area savings (known since #135; rediscovered at machine, band, macro granularity; the row is a near-optimal shared-delivery structure). Wall 2: proxy metrics halve per step of realism (057: −50/69% bound → +38–250% built; 058: −66.1% probe → −35.9% spike → −27.0% legal). Wall 3: density-vs-correctness invisibility (4 confirmed payouts; #520 flagship). Wall 4: instrument trust (default warmup certified buffer-fill as deficits — mil5ore −28.7% FAIL → +0.7% PASS at 288k warmup; 058's kill number needed three corrections). Wall 5: the step-change bar held (~−27% is the measured ceiling for legal 2D repacking; bar was −33%; the project honored its own bar).

## Ranked next + don't-refund

Next: (1) #135 template shrinking — only move that attacks the floor; (2) #526 DI flagship cell repair; (3) DI-aware packing (058's kill doesn't cover it); (4) wide-band reshaping (only max_per_row capping was falsified); (5, deliberately low) RFC-057's trunk recommendation — 058 built essentially that one granularity up and landed below bar. Don't re-fund: tree manifolds, band packing, folding-as-density, RFC-055, meter expansion pre-attribution.

## The honest paragraph

The record voted against the post-pass. Every gram of shipped density came from candidates inside the decomposition search under never-worse gates; every post-pass failed its gate, was demoted to shape-only, or sits opt-in and unused. #456's thesis assumed the grid's sparsity was waste; the arc measured that it mostly isn't — the row is near-optimal shared delivery, and the slack is either load-bearing margin (RFC-061) or template footprint (#135, named fifteen weeks ago). The post-pass framing bought the meter, a harder validator, and a clean falsification record — its residual value is as an audit instrument, not a product direction. Fund density where wins came from: new primitives and candidates under sim-anchored never-worse gates; do not reopen whole-factory repacking until a primitive-level change moves the logistics floor that killed it twice.

**Addendum, 2026-08-14 (#632 A2, owner call):** `bus::compaction` and the
`compact_layout`/`fold_layout` flags this retro's "opt-in and unused" line
refers to are deleted — the audit-instrument residual value this document
identified was banked (this retro, the RFC decision logs, the validator
hardening it describes) and the dead code itself removed. Git history at
the deletion PR is the revival path if a primitive-level change ever moves
the logistics floor this arc measured twice.
