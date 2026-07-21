# RFC-047: Lane-aware tap delivery (stacked rate ceilings)

Registry: [`rfcs.md`](rfcs.md). Status: **Draft** (2026-07-21).

## Summary

Make trunk→row and row→trunk delivery **lane-aware**, so the lane
planner's full-belt thresholds stop assuming both lanes fill when the
geometry only fills one. This is the unlock RFC-046 deliberately
descoped: with delivery lane-accounted, the consumer-clamped fan-in
wall ([#312](https://github.com/storkme/spaghettio/issues/312)) can
scale honestly — at S=1 by using real two-lane tap forms where they
exist, and at S>1 by the belt-stacking factor — turning belt stacking
from "cheaper belts at the same rate" into "higher rate ceilings."
Riders: the walker-overshoot residual at zero-headroom tier boundaries
(fixtures land here), a kill-criterion-bounded look at the
legendary-express junction failure, and the per-lane stackedness
infrastructure RFC-046 deferred.

## Motivation

RFC-046's Phase 2 differentials falsified full-belt ×S on tap-delivered
flow: sideloads fill one lane (B8), inserter drops fill one lane (the
**far** lane — the recon corrected the mechanics doc's I5 here; the
code was always right), so a trunk or row-input belt fed that way
carries everything on one physical lane. The S=1 fan-in wall had been *accidentally
shielding* the gap by refusing such configs before they laid out;
scaling it ×S exposed walker-caught single-lane overloads (18/s on a
15/s stacked yellow lane — probe-verified, see RFC-046's decision
log). RFC-046 therefore froze trunk-count geometry at S=1 and demoted
the ceiling lift here.

Concrete failing cases today:

- `stacking_fanin_wall_conservative_parity_ec6_yellow_legendary`
  (e2e): EC@6/s legendary yellow refuses at S=1 **and** S=2 — the
  fixture's own doc comment says to flip it to a differential success
  when this RFC lands.
- EC@60/s legendary express S=2: junction-solver failure near a dense
  crossing plus ~3% walker overshoot on zero-headroom lanes at exact
  tier boundaries (characterized in RFC-046's decision log; no issue
  number — this RFC files one if the junction half survives its
  bounded investigation as a real, separable defect). **Probe evidence
  (2026-07-21, `debug_overshoot_probe`): the junction failure is the
  dominant defect, not a peer** — the unresolved 50-tile crossing at
  (2,38) orphans ghost belts, every furnace bank in two full rows
  reports `belt-flow-reachability` unreached, and ore input belts
  deliver 0.0/s. The 15.4–15.5/s lane overloads (vs a 15/s stacked
  yellow per-lane cap, tiles (24–25, 14/21/118)) are secondary. Kill 3
  (junction) therefore gates the express variant; kill 4 (overshoot)
  is a smaller residual on the same config.
- More broadly: at high build quality, collapsed machine counts shrink
  consumer trunk counts while flows stay constant, so the wall bites
  configs that plain-quality builds handle (#312) — and stacking,
  which physically multiplies belt capacity ×4, currently cannot lift
  it at all.

## Ground truth (delivery geometry today — recon 2026-07-21, cited)

The recon **inverted the draft's premise**: consumption-side taps were
never the bottleneck; the production-side feed is.

1. **Trunk→row tap-offs are real splitters, always.**
   `find_tap_off_ys` (lane_planner.rs ~1174) plans exactly one tap y
   per consumer row; ghost_router Step 2 (~390–453) stamps a genuine
   2-tile splitter per non-last tap (50/50 for ordinary lanes;
   PRIORITY splitter with `loop_priority_rate` = per-tap consumer
   demand for merge-tap lanes), and the last tap is a lane-preserving
   corner. No raw-sideload or dual-sideload (B10) tap form exists
   anywhere — matching mechanics rule S9. **Taps are full-belt
   capable today.**
2. **Producer→trunk feeds are raw sideloads** (ghost_router
   ~1549–1592: one `ret:` West-exiting belt per producer row,
   perpendicular B8 merge into the South trunk = ONE lane) — *unless*
   a family balancer exists. The balancer gate (lane_planner
   ~1058–1079) fires only for `n_producers ≥ 1 && n_lanes_with_
   consumers ≥ 2` and stamps a real SAT-generated splitter cascade
   whose contract genuinely is full-belt-cap per trunk; the merge-tap
   fallback (K shared trunks, n→1 merge trees + priority taps) is
   equally full-belt-grounded.
3. **Row-internal output balancing fills both lanes but it doesn't
   survive the feed.** Every `lane_split` row stamps the midpoint
   `sideload_bridge` (templates.rs ~273–320) filling both lanes of
   the row's own output belt — then geometry class 2 collapses it
   back to one lane at the trunk sideload (when no balancer).
4. **The fan-in wall's `full_belt_cap` is grounded, not aspirational,
   for ≥2-consumer-trunk shapes** (the balancer gate covers exactly
   those). The unsound residue is the **single-consumer-trunk,
   no-balancer shape**: clamping to `consumer_trunk_count == 1` fails
   the balancer gate (`n_lanes_with_consumers >= 2`), the trunk falls
   through to the sideload feed, yet the wall still credits
   `1 × full_belt_cap`. This is precisely the shape of RFC-046's
   observed 18/s-on-one-lane overload (EC@6 legendary yellow: one
   cable trunk, sideload-fed). Candidate gap at S=1 too — not yet
   confirmed live in the corpus (partition/pad logic may shield it).
5. **The engine's own sizing convention already assumes single-lane
   delivery as baseline**: trunk belts sized at `lane.rate * 2.0`
   (ghost_router ~1502), final merge at `total_rate * 2.0`
   (output_merger ~56) — full-belt-cap is opt-in where balancer/
   merge-tap machinery explicitly backs it. Independent corroboration
   of 1–4.
6. **Two model bugs (verified against the game, three independent
   sources), MUST fix before trusting verification here:**
   - **I5 is backwards in the mechanics doc**: inserters drop on the
     **far** lane, not the near lane. The code
     (`common::inserter_target_lane`) is correct and says so; the doc
     (and comments citing I5 for near-lane reasoning, e.g.
     templates.rs ~276, and RFC-046's prose) have the wrong mental
     model. Single-lane *conclusions* survive (it's still one lane —
     the other one); the doc and citations need the sweep.
   - **The walker's convergence-phase splitter model invents lane
     mixing**: `splitter_output_rates_mixed` (belt_flow.rs ~1880,
     used in the demand-pull convergence that drives final lane
     rates) pools both input lanes and re-splits evenly — real
     splitters never mix lanes (S4), and the codebase's *other*
     splitter function models that correctly. Net effect: every
     splitter acts as a free lane-rebalancer in validation, masking
     genuine lane starvation downstream — exactly the defect class
     this RFC verifies against.
7. **Multi-stage balancer**: genuinely absent (balancer_library is
   single-stage; `ModuleSizeSplit` routes *around* the wall by
   re-partitioning, and its own RFC records K-DS1-2 unsatisfied). The
   fan-in refusal is a real capacity wall, correctly placed.

## Design

Three legs, each grounded above:

**Leg A — honest walker first (fix 6b, then trust it).** Replace the
convergence-phase splitter pooling with the lane-preserving model the
walker already has (`splitter_output_rates`), reconciling the two
functions. This is a validator-semantics change: existing fixtures may
gain honest lane-starvation warnings that pooling was masking — each
flip is investigated (real starvation → layout issue filed/fixed;
walker artifact → model corrected), never blanket-re-blessed. The I5
doc fix + citation sweep (6a) rides along. Only after Leg A is the
validator a trustworthy oracle for Legs B/C.

**Leg B — ground the wall where it's false (fix 4).** The
single-consumer-trunk no-balancer shape either (i) gets routed through
the existing merge-tap machinery (n→1 merge tree + priority tap — the
full-belt-grounded form that already exists, degenerate K=1 case), or
(ii) keeps the sideload feed and the wall's credit drops to per-lane
cap for that shape. Choose (i) when the wall would otherwise refuse or
under-deliver, (ii) when per-lane suffices — decided by the planner
from rates, never silently. This makes `full_belt_cap` universally
grounded at S=1.

*Strategy scoping (spec-review finding)*: merge-tap is **Pooled-only**
today (`MergeTapCandidate` refuses other strategies; the flag "fights
the partitioner's module IDs"). Phase 1 therefore applies option (i)
under Pooled only; under `PartitionedDecomposed` the shape gets option
(ii) unless a Phase-1 census shows the single-trunk shape cannot arise
there. Called out explicitly because the RFC's own proof fixtures are
Pooled — the asymmetry would otherwise stay invisible until a
Partitioned fixture regressed.

*External-input lanes* (`n_producers == 0`) are **outside Leg B**: no
producer rows exist to merge-tree, and their flow enters at the trunk
head as a boundary condition rather than via a B8 feed (the
`LaneConsolidated` mechanism, not this RFC's machinery). Phase 1
verifies that moot-ness with a one-line census; if falsified, they get
option (ii) per-lane credit — never option (i).

**Leg C — the stacked lift.** With every wall credit
geometry-grounded, scaling becomes sound: balancer/merge-tap-backed
trunks carry `full_belt_cap × S` (stacks flow through splitters, BS4;
inputs are stack-loaded row outputs under RFC-046's forcing), and
sideload-fed shapes carry `lane_capacity × S`. The wall and K-trunk
retirement sum per-shape deliverable rate instead of assuming one
formula. RFC-046's parity fixture flips to the differential success it
was written for; per-lane stackedness (the deferred rider) falls out
of the same per-shape accounting using `StackingCtx::for_item` (the
per-ITEM exemption axis composes multiplicatively with the per-SHAPE
lane axis: deliverable = lanes(shape) × lane_capacity(tier) ×
for_item(item) — an exempt item stays ×1 regardless of shape).

*Plan-time ordering (spec-review finding)*: the wall check currently
runs **before** the balancer/merge-tap stamp decision resolves
(`n_lanes_with_consumers` is computed downstream of the wall's own
trunk-count clamp). Phase 2 restructures this as two passes: first
classify the candidate trunk's machinery (balancer / merge-tap /
sideload) as a **pure function** of the same inputs the stamp decision
reads (producer count, consumer slots, rates); then evaluate the wall
against that class's deliverable rate. If classification cannot be
made a pure pre-stamp function — if a genuine fixed point emerges —
stop and redesign the ordering; do not iterate to convergence inside
the planner (the merge-tap RFC's shape/family-ordering history is the
cautionary precedent).

Explicitly out of scope: a true multi-stage balancer generator (7) —
Leg B reuses merge-tap trees instead; if a shape needs genuine
multi-stage balancing beyond that, the refusal stays honest and the
generator remains future work.

## Kill criteria

1. **Current-behavior identity.** With no configuration change (S=1,
   existing corpus), layouts are bit-identical: full suite green,
   STRESSGOLD `check` 9/9, zero golden re-blesses — *unless* a
   documented fixture flip is the point (the parity fixture, and any
   fixture this RFC's honest accounting proves was passing on a
   fiction). Any other S=1 diff is a threading bug — stop and fix.
2. **No credit without geometry.** Every lane-capacity credit above
   one lane must be justified by a delivery edge whose geometry class
   provably fills both lanes (cited to the ground-truth section) and
   must be visible to the walker's lane attribution. If the only way a
   fixture passes is a credit the walker cannot re-derive, the design
   is the #311 trap again — rework, do not ship.
3. **Junction investigation bound.** The legendary-express junction
   failure gets a time-boxed, snapshot-driven characterization (repro,
   trace events, zone identification). If the fix is not localized —
   if it requires junction-solver framework changes beyond a bounded
   strategy/cost fix — file the issue with the characterization and
   descope it from this RFC; do not redesign the junction solver here
   (three RFCs burned that way pre-`prune_dangling`; check
   `sol.entities` vs raw SAT output FIRST).
4. **Overshoot honesty.** The ~3% walker overshoot at exact tier
   boundaries must be root-caused (walker modeling artifact vs real
   physics) before any fixture is "fixed" by adding headroom margins.
   A margin without a root cause is evidence-overrun — the dominant
   failure shape this repo's process exists to prevent.
5. **The lift is real.** Close-out requires the parity fixture flipped
   to a differential success (EC@6/s legendary yellow: refuses at S=1
   for honest geometric reasons or lays out clean; lays out clean at
   S=2) with the same per-tile physical audit discipline as RFC-046's
   headline — and at least one fixture where the *rate ceiling*
   (not just belt tier) demonstrably rises with S.
6. **Phase-0 blast-radius bound** (spec-review addition; precedent:
   the merge-tap RFC's census-first Phase 0). Before committing the
   walker splitter fix, land it uncommitted and census the flips: one
   full suite + STRESSGOLD check, counting flipped fixtures and golden
   category-count changes (the stress goldens record warning counts,
   so re-blesses are expected — the question is how many and why). If
   **more than ~10** fixtures/goldens flip, or any flip requires
   touching layout-generation code (rather than accepting new honest
   warnings / filing layout issues) in **more than 2** places, this is
   not a Phase-0-sized fix — stop: narrow the fix to the trunks Legs
   B/C actually credit, or split the walker fix into its own RFC.
   Note the mitigating structural fact (ground truth 5): sideload-fed
   trunks were always tier-sized for single-lane delivery, so new
   over-cap findings should concentrate on balancer-backed trunks
   receiving contamination — if they appear elsewhere en masse, the
   model of the fix is wrong, which is exactly what the bound catches.

## Verification plan

- Full suite single-run counts + STRESSGOLD at every phase gate.
- Fixture flips: the fan-in parity fixture → differential success;
  a ceiling fixture (rate that refuses at S=1, lays out at S=2, with
  per-tile stacked-capacity audit).
- Walker cross-check: lane attribution re-derives every two-lane
  credit (kill 2).
- Browser eyeball (user) on the flipped configs.

## Phasing

- **Phase 0 — model honesty (Leg A).** I5 doc correction (near→far) +
  sweep of **every comment/doc asserting the old near-lane or
  lane-mixing model**, not just literal I5 citations (the spec review
  found belt_flow.rs ~2916 asserting lane-mixing as fact with no I5
  cite); walker convergence-phase splitter fix (lane-preserving model
  replaces pooling), gated by the kill-6 blast-radius census. Every fixture
  the walker fix flips is investigated individually — real starvation
  becomes a filed/fixed layout issue, walker artifacts become model
  corrections; blanket re-blessing is forbidden. Only after Phase 0 is
  validator output a trustworthy oracle for the rest.
- **Phase 1 — ground the wall at S=1 (Leg B).** Single-consumer-trunk
  no-balancer shapes routed through merge-tap (degenerate K=1) or
  credited per-lane, planner-decided by rate; `full_belt_cap`
  universally geometry-grounded; identity gates modulo Phase-0
  documented flips (kill 1).
- **Phase 2 — the stacked lift (Leg C).** Per-shape deliverable-rate
  credits ×S; the RFC-046 parity fixture flips to differential
  success; a ceiling fixture proves rate ceilings rise with S
  (kill 5); per-lane stackedness rider on the same accounting.
- **Phase 3 — bounded residuals.** Junction characterization
  (kill 3); overshoot root-cause (kill 4 — first hypothesis to test:
  whether the Phase-0 splitter fix already changes those lane rates).

## Decision log

- **2026-07-21 — Phase 1 censuses run (probe committed, env-gated
  `SPAGHETTIO_047_CENSUS`).** (1) **External-input lanes confirmed
  moot**: they split by per-lane cap at construction
  (lane_planner ~724, `n_splits = ceil(rate / max_lane_cap)`), so no
  external lane ever plans above one lane's capacity — Leg B correctly
  excludes them. (2) **The single-trunk-clamp shape is LIVE in the
  corpus** — `copper-cable rate=30 n_splits=2 plan=false` and
  `copper-cable rate=20 n_splits=2 plan=true` (the latter proving it
  arises under PartitionedDecomposed, where merge-tap option (i) is
  unavailable — the spec review's asymmetry is real, not
  hypothetical). (3) **Open geometric question for Phase 1**: the
  post-Phase-0 honest walker does NOT flag these live single-trunk
  lanes, so their delivery must exceed one lane — hypothesis: a
  single-producer trunk is fed end-on (straight/corner, lane-
  preserving, carrying the row bridge's both-lane fill) rather than
  via the `ret:` B8 sideload, meaning the ground-truth-4 gap may be
  narrower than "single-consumer-trunk": it may require BOTH multiple
  producers (forcing sideload merges) AND a single consumer trunk.
  Phase 1's classification function must distinguish producer-count
  geometry, and the gap census needs per-fixture attribution (test
  interleaving made the quick attribution fuzzy; use
  `--test-threads=1` or per-test runs).

- **2026-07-21 — Phase 0 (Leg A) landed; kill 6 census PASSED by a
  wide margin.** Census (uncommitted scratch change + full gates): 1
  flipped test / 2 balancer templates / 5 issues vs the >10 threshold;
  STRESSGOLD 9/9 unchanged; zero layout-code touch points. The walker
  fix landed as `splitter_output_rates_convergence` (per-lane pooled
  totals, demand-allocated independently; priority-loop ratio computed
  branch-level, applied per-lane), replacing the physically-false
  lane-mixing model and its false code comment. The one honest
  finding — balancer library shapes (7,3)/(7,4) genuinely
  lane-imbalanced at saturation (worst 8.112/s on 7.5/s per-lane;
  invisible to the old model structurally) — filed as #334 and carried
  in `balancer_lane_audit` as a known-imbalanced carve-out with a
  tripwire that fails when the shapes get re-baked (so the carve-out
  cannot silently outlive the fix). I5 doc correction + sweep landed
  (0a). Recorded simplifications (census ambiguities, for Phase 1-2
  awareness): no per-lane demand signal exists (both lanes share the
  scalar demand ratio); the asymmetric-branch per-lane cap (cap/2) was
  moot in the census (all flips hit the uncapped symmetric branch) and
  wants a merge-tap priority fixture before Leg B leans on it; the
  priority-loop branch is measurement-unverified (no priority
  splitters in the library corpus) — same fixture covers it. Gates:
  suite 869/0/36 (one clean run), STRESSGOLD 9/9.

- **2026-07-21 — Adversarial spec review: APPROVE-WITH-CHANGES; v2
  folds all four required changes.** Every ground-truth citation
  survived independent code-level verification, and both model-bug
  claims were re-confirmed (I5 direction independently game-verified
  a second time). The four folded changes: (1) kill 6 — a Phase-0
  blast-radius bound with explicit flip-count/LOC circuit breakers
  (the review's top finding: the walker fix is the highest-blast-
  radius change here and had no census-first discipline, unlike the
  merge-tap RFC precedent; STRESSGOLD goldens record warning counts,
  so re-blesses are structurally expected); (2) Leg B strategy
  scoping — merge-tap is Pooled-only, PartitionedDecomposed gets
  per-lane credit pending a census, and external-input lanes are
  explicitly out of Leg B (no producers to merge-tree; boundary-
  condition delivery via LaneConsolidated); (3) Leg C — the plan-time
  circularity (wall runs before the stamp decision) resolved as a
  two-pass pure-function classification with a stop-and-redesign
  trigger if a real fixed point emerges, plus kill-2 sub-item: tiles
  Legs B/C introduce must carry `carries` attribution so the walker
  never falls back to layout-wide stacking on them (contamination via
  routing bugs is a proven failure class here — the foreign-trunk
  hijack saga); (4) the Phase-0 sweep broadened beyond literal I5
  citations to any lane-mixing assertion (belt_flow ~2916 found by
  the review with no I5 cite). Also noted: exemption×shape axes
  compose multiplicatively (one added sentence in Leg C).
- **2026-07-21 — Recon folded; design set (three legs).** The recon
  inverted the draft premise: taps are already splitter-based and
  full-belt-capable; the raw-sideload producer→trunk feed is the
  single-lane ceiling, and the fan-in wall's full-belt credit is
  already grounded for ≥2-consumer-trunk shapes (balancer gate) —
  the unsound residue is the single-consumer-trunk no-balancer shape,
  which matches RFC-046's observed overload exactly. Two verified
  model bugs became Phase 0: the mechanics doc's I5 lane direction is
  backwards (code correct), and the walker's convergence-phase
  splitter model pools lanes (physically false, masks the exact
  defect class this RFC verifies). Design: honest walker first, then
  ground the wall at S=1 via existing merge-tap machinery (no new
  balancer generator), then the ×S lift on per-shape credits.
  Multi-stage balancer generation explicitly out of scope.
- **2026-07-21 — RFC drafted (skeleton).** Number claimed as RFC-047
  per registry. Ground-truth and Design sections deliberately held
  open for the tap-geometry recon — writing geometry claims from
  memory is how RFC-046's spec review found a falsified census; this
  RFC starts from cited geometry instead.
