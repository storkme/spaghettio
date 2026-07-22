# RFC-047: Lane-aware tap delivery (stacked rate ceilings)

Registry: [`rfcs.md`](rfcs.md). Status: **Complete** (2026-07-22; browser eyeball of the flipped configs open — user-run; follow-ups #334/#335/#336/#337).

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
  when this RFC lands. *(Done, Leg C — flipped to
  `stacking_fanin_wall_lift_ec6_yellow_legendary`: S=1 refuses, S=2
  lays out clean. See the 2026-07-22 step-3 decision-log entry.)*
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
  is a smaller residual on the same config. *(RESOLVED 2026-07-22 —
  both closed: the junction failure died with 047-1b's row
  consolidation, the overshoot with worst-lane output-belt sizing;
  the express config now validates with zero errors, pinned by
  `stacking_ec_60s_express_legendary_s2`. See the decision log's
  Phase 3 entry; residual ore-routing warnings are #335.)*
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
   **Correction (2026-07-22 implementation, supersedes the "every
   producer sideloads" reading):** only the **non-topmost** producers
   B8-sideload. The trunk is stamped from `source_y = min producer
   `output_belt_y`` down (ghost_router step 3.5), so the **topmost**
   producer's `ret:` lands at the trunk *head* (`trunk start_y ==
   source_y == that producer's out_y`, nothing north of it) and is a
   lane-preserving **B11 corner**, not a sideload. Single-producer
   intermediate lanes therefore already carry both lanes end-to-end
   with no dead trunk-head tiles — which is why the live single-trunk
   corpus lanes stay walker-green, and why the 2026-07-21 experiment
   entry's prescribed "single-producer corner-feed" is a no-op (see
   the 2026-07-22 decision-log entry). The genuine single-lane
   overload is the **2nd+ producer of a fragmented multi-producer
   trunk** sideloading mid-trunk.
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

- **2026-07-22 — Phase 3 residuals resolved; the ORIGINAL express
  headline recovered.** Kill 3 (junction): the express@60-legendary-S2
  junction failure is GONE — 047-1b's row consolidation removed the
  50-tile crossing entirely; no junction-solver work was needed (the
  bounded investigation closed at its first probe). Kill 4
  (overshoot): root-caused, not margined — the 15.4–15.5/s tiles were
  furnace-row OUTPUT belts sized at `output_rate` (assumes perfect
  50/50 lanes) while the midpoint bridge splits ⌈n/2⌉/⌊n/2⌋, so odd
  machine counts overload one lane at zero-headroom tier choices
  (29.69/s planned on a 30/s stacked-yellow). Fix: lane-split rows
  size their output belt by worst-lane rate (`2 × ⌈n/2⌉ ×
  per-machine`); S=1-inert empirically (suite 869/0/36, STRESSGOLD
  9/9, zero re-blesses — no live S=1 row sits at an odd-split
  zero-headroom boundary). Result: **EC@60/s legendary EXPRESS S=2
  now validates with zero errors** — the config RFC-046 demoted as
  blocked is delivered, pinned by `stacking_ec_60s_express_legendary_s2`
  (per-tile audit + teeth >45/s). Honest residual: 39 warnings — one
  furnace bank's ore routing never materializes (19 reachability + 20
  input-rate-delivery), a distinct defect the junction error had been
  masking — filed as #335, out of scope here (routing, not delivery
  accounting).

- **2026-07-22 — Leg C step-3 (wall ×S re-scale) landed; parity fixture
  flipped to the lift differential (kill 5 met).** Changed the fan-in
  wall's `full_belt_cap` from `max_lane_cap × 2` to `lane_cap × 2`
  (`lane_cap = max_lane_cap × for_item(item)`), so a full stacked belt's
  `2 × lane_cap` credit is available where the geometry grounds it. Every
  crediting path is now geometry-grounded at ×S (steps 1b/2 established
  this); the post-Phase-0 honest walker adjudicates (kill 2). S=1
  bit-identical (`for_item()==1` ⇒ `lane_cap == max_lane_cap`).
  - **Parity fixture flipped**:
    `stacking_fanin_wall_conservative_parity_ec6_yellow_legendary` →
    `stacking_fanin_wall_lift_ec6_yellow_legendary`. S=1 still REFUSES
    (25/s cable > 15/s full yellow — the wall holds honestly). S=2 now
    LAYS OUT with **0 validation errors**, passes the per-tile stacked-
    capacity audit (every rate-stamped belt/UG/splitter ≤
    `belt_throughput_stacked(tier, 2) + 0.01`, mirroring
    `stacking_ec_60s_red_one_belt_headline`), and the teeth assertion
    (a 25/s belt tile > 15/s unstacked full yellow — the lift is real).
  - **Walker-verified corner-feed (kill 2).** Probe (`probe_047`) on the
    S=2 layout: the copper-cable is ONE lane-split producer row (2
    legendary AM3 machines) whose both-lane output corner-feeds the
    single trunk at its head; `compute_lane_rates` on the trunk column
    (x=3, y=7..11) reports **left=9.00, right=9.00 on every tile** — both
    lanes evenly loaded, well under the 15/s stacked-yellow per-lane cap.
    Contrast the pre-047 experiment (2 fragmented producers): 18/s on ONE
    lane. The credit is re-derivable by the walker, exactly as kill 2
    requires — no #311-style unwitnessed credit.
  - Gates: full suite green (e2e 58 / unit 773, one clean run),
    STRESSGOLD `check` 9/9, clippy `-D warnings` clean. Landed as
    `feat(047-1c)`. Ceiling fixture (a rate that refuses at S=1 and lays
    out at S=2) is satisfied by this same fixture: it refuses at S=1 and
    lifts at S=2. Junction/overshoot residuals (kills 3–4, Phase 3)
    remain out of this leg.

- **2026-07-22 — Leg B step-2 (late sideload check) landed; the Phase-1
  census's "no such shape live at S=1" claim ALSO falsified — the check
  exposes a real pre-existing S=1 overload.** Added the ghost_router
  ret-spec-time refusal (RFC-047 Leg B (ii)): a single-consumer trunk
  (`consumer_rows.len() == 1`) fed by `>= 2` producers with no balancer,
  whose `lane.rate` exceeds one stacked lane's capacity
  (`lane_capacity_stacked(horiz_belt, for_item(item))`), returns a named
  `"lane-aware delivery: item X rate R exceeds per-lane capacity C on a
  sideload-fed single trunk"` Err. Rationale: the topmost producer
  corner-feeds the trunk head (both lanes, ground-truth 2), but every
  later producer B8-sideloads mid-trunk onto ONE physical lane, so
  `rate > per-lane×S` cannot be delivered.
  - **Census falsification (credit: step-2 probe).** The Phase-1 census
    inferred "the post-Phase-0 honest walker does NOT flag these live
    single-trunk lanes" from suite-green — but the one fixture with this
    shape (`tier2_electronic_circuit_splitter_stamp_regression` =
    EC@10/s, AM1, fast/red belts, from plates) only ever asserted on
    "sideloads into underground input" warnings, never lane-throughput.
    Probing with the check disabled shows the config builds with **38
    silent `lane-throughput` errors** — copper-cable 30/s from 2
    fragmented producer rows into one red trunk, near lane 22/s vs a
    15/s red per-lane cap. So the walker WAS flagging it; nothing
    asserted on the flag. The shape is live at S=1 and was silently
    broken; step 2 refuses it honestly. This is S=1-identical to base
    5f838be (047-1b is S=1-inert), so the overload is pre-existing, not
    introduced.
  - **Merge-tap cannot rescue it (verified, answering the "transparent
    fix?" question).** For EC@10/s AM1 red: (a) the in-plan merge-tap
    gate needs `n_lanes_with_consumers >= 2`, but this shape has ONE
    consumer → merge-tap never restructures it (probe: forcing
    `merge_tap=true` still yields the 38-error plain sideload); (b) the
    `MergeTapCandidate` decomposition fallback only runs when Native
    produced an *unaccepted layout*, but step 2 makes Native hard-**Err**
    (`outcome == None`) → the candidate is skipped ("merge-tap: did not
    run"). A (2,1) single-consumer merge is genuinely unwired, so a
    named refusal is the honest outcome, not a transparent fix.
  - **Fixture handling (plan (b), coordinator-approved).**
    `tier2_electronic_circuit_splitter_stamp_regression` is converted
    from a (now-vacuous) sideload-into-UG retry guard into the RFC-047
    named-refusal guard for this shape. The old retry-loop coverage was
    already defunct on the current pipeline: the retry is driven by
    junction `cap_coords` (`LayoutRetried`), not `DroppedBridge`, and
    neither fires for EC AM1 fast at any rate 6..=10 (probe-verified). No
    live coverage lost; its golden-hash entry removed (config no longer
    builds). Fresh UG-retry regression coverage needs a config that
    actually emits `LayoutRetried`/`BridgeDropped` — filed as a
    follow-up, out of RFC-047 scope.
  - Gates: full suite green (e2e 58 / unit 773, one clean run),
    STRESSGOLD `check` 9/9, clippy `-D warnings` clean. Landed as
    `feat(047-2)`.

- **2026-07-22 — Leg B step-1 premise falsified during implementation;
  real root cause is a stacking-blind row-split cap, not a `ret:`
  sideload. Fix relocated from ghost_router to `place_rows`.** The
  2026-07-21 experiment entry (below) diagnosed the EC@6-legendary-yellow
  S=2 overload as a *single-producer* row whose both-lane row-bridge fill
  gets B8-collapsed by its `ret:` sideload, and prescribed a
  ghost_router "single-producer corner-feed" (trim the dead trunk-head
  tiles, corner the ret in). Implementation probing (`probe_047`
  example; AM3-legendary makes copper-cable at 12.5/s **per machine**)
  falsified both halves:
  - The parity fixture's cable trunk is **two-producer, not one**.
    `place_rows` fragments the 2 cable machines into 2 single-machine
    rows (machines at y=3 and y=10), so producer-1 corner-feeds the
    trunk head cleanly while **producer-2 sideloads mid-trunk** — that
    B8 is the 18/s-on-one-lane overload (9 lane-throughput errors at
    S=2). Not a single-producer bridge-collapse.
  - The prescribed ghost_router corner-feed is a **no-op**. Single-
    producer intermediate lanes already have `trunk start_y == source_y
    == producer out_y`, so the topmost `ret:` already lands as a B11
    corner (nothing above the head to trim). Verified: no dead trunk-
    head tiles exist for the single-producer shape.
  - **Root cause**: `placer::max_machines_for_belt_both_lanes` is
    stacking-blind — it caps machines-per-row by `lane_capacity` (7.5
    yellow) while the belt-tier choice at the *same* call site already
    uses `belt_entity_for_rate_stacked`. Cable's 12.5/s per-machine
    output > 7.5 forces `max_per_row = 1`, fragmenting a stacked
    producer that would otherwise be one lane-split row. This is an
    RFC-046 oversight (no doc/comment guards the unstacked cap), and it
    re-introduces exactly the mid-trunk sideload this RFC set out to
    remove. Leg C's own premise ("inputs are stack-loaded row outputs
    under RFC-046's forcing") requires the row output to actually be one
    stacked belt, which this fixes.
  - **Fix**: thread `out_stack` (`StackingCtx::for_item`) into
    `max_machines_for_belt_both_lanes` and cap by
    `lane_capacity_stacked`. Collapses the cable to one lane-split row
    whose both-lane output corner-feeds the trunk → both trunk lanes
    carry ~9.4/s each; with the wall re-scale (below), the parity
    fixture lays out at S=2 with **0 validation errors** (probe-
    confirmed). **S=1 is bit-identical** (`for_item()==1` ⇒
    `lane_capacity_stacked==lane_capacity`): full suite green
    (e2e 58 / unit 773), STRESSGOLD `check` clean, zero golden shift.
    Two invariants confirmed (spec-review asks): (a) `out_stack =
    ctx.for_item(output_item)` respects the RFC-046 **family
    exemption** — exempt items return `for_item()==1`, so an exempt
    row's per-machine cap stays unstacked and never widens on stacked
    credit (`stacking_kovarex_family_exempt_s2` covers it); (b) the
    **single-lane** variant `max_machines_for_belt` is left stacking-
    blind **deliberately** — its output is sideloaded onto ONE lane
    (B8/I5), so crediting it ×S would just relocate the single-lane
    overload; only the both-lanes (bridge+corner-feed) output
    legitimately fills two lanes and so scales ×S. The asymmetry is
    documented at both functions.
  - Leg B's part (ii) **late sideload check** is kept as a genuine
    multi-producer safety refusal (a still-fragmented multi-producer
    single-consumer-trunk lane over per-lane×S cap is refused by name),
    and part (iii) the wall ×S re-scale lands last. Part (i)
    "single-producer corner-feed" is **withdrawn as a no-op** and
    replaced by the row-split cap fix above. Landed as commit
    `feat(047-1b)`.

- **2026-07-21 — Leg B/C wall-lift experiment: the honest walker
  vetoed the optimistic credit, and the veto tells us exactly what to
  build.** Experiment (run then reverted, branch green): re-scaled the
  wall/K by effective stacking and flipped the parity fixture. Result:
  EC@6-legendary-yellow S=2 lays out past the wall but validation
  fails honestly — 18/s on the RIGHT lane of the cable trunk and row
  input belt (identical tiles to RFC-046's original probe), because
  this shape's `ret:` feed genuinely sideloads (start_x ≥ goal_x) and
  B8 collapses the row bridge's both-lane fill into ONE trunk lane.
  Kill 2 performed exactly as designed: the wrong credit failed
  loudly, pre-merge. **Leg B implementation, now concrete:**
  (i) **single-producer corner-feed** — when a lane has exactly one
  producer and no flow enters the trunk above the ret junction, the
  trunk-head tiles above the junction are dead: trim the trunk to
  start AT the junction and turn the ret in as a B11 corner, which
  preserves both lanes end-to-end (row bridge fill survives). This
  unlocks the parity flip: 25/s cable ≤ 30/s stacked full-belt on a
  corner-fed trunk. (ii) **late sideload check** for multi-producer
  no-balancer single-trunk shapes at ret:-spec time (adjacency known):
  sideload-fed rate above per-lane×S ⇒ named refusal. (iii) The wall's
  early ×S re-scale then lands LAST, gated on (i)+(ii) being in place.
  Merge-tap (option i-as-specced) is inapplicable to the single-
  producer case (gate needs n_producers ≥ 2) — corner-feed replaces it
  there; multi-producer single-trunk shapes still get merge-tap under
  Pooled per the earlier scoping.

- **2026-07-21 — Feed-geometry mechanism confirmed; Leg B design fork
  identified.** ghost_router ~1565: a `ret:` sideload spec is SKIPPED
  when the producer row's exit already covers the trunk column
  (`start_x < goal_x` — "the row's own exit belt already covers it"),
  i.e. **adjacency, not producer count, decides sideload-vs-direct
  feed** — direct feeds carry the row bridge's both-lane fill, which
  is why the live single-trunk-clamp corpus lanes stay walker-green.
  Consequence: trunk-machinery classification at wall time cannot know
  adjacency (column assignment happens later), so the two-pass "pure
  function ahead of the stamp" plan hits its predicted ordering fork.
  Candidate resolutions for Phase 1 (decide with fresh context, per
  the Leg C stop-and-redesign rule): (a) conservative wall-time
  classification (only balancer/merge-tap = full-belt) — REJECTED
  as-is: it would refuse live, working corpus shapes (cable@30
  single-trunk direct-fed); (b) grandfather S=1 wall semantics
  unchanged and apply per-shape distinction only to the Phase-2 ×S
  term — avoids regressions but splits the credit formula's honesty
  story; (c) move the clamped-single-trunk wall evaluation
  post-column-assignment where adjacency is known — an ordering move,
  not a fixed point, but touches the planner's phase structure.
  Leaning (c) with (b) as fallback; whichever is chosen, the walker
  (post-Phase-0) independently checks the result, so a wrong choice
  fails loudly, not silently.

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
