# RFC-072: The cell interface — boundary contracts and rate-scaling composition

Status: Proposed
Evidence base: [`composition-frontier-probes.md`](composition-frontier-probes.md)
Successor framing to: the 2026-07-24 strategy call ("bus stays the
low-rate/intra-cell winner; high rates via composition")

## Summary

Give cells a typed boundary contract — ports as `(item, rate, belt tier,
edge position)` with an attached measurement receipt — and make the engine
honor it in two steps: extend the deliver-plan-or-refuse-by-name contract
(RFC-069) to the OUTPUT side (the sim-confirmed half-plan hole), then
compose k replicated cells to reach rates the single bus structurally
cannot (the 120/s plan-arithmetic wall). A third step — fixing an
"embedding cost" in composed stages — was in the first draft and was
retired when ground truth showed the cost was meter artifact
(Motivation 3).
The strategic payoff is the verification inversion RFC-067 promised: a
sim-anchored cell library turns per-layout verification cost into a
reusable asset, and the celldb power law (top 5 motifs = 87.7% of machine
mass) says a small library covers most demand.

## Motivation

All numbers are measured and reproducible today
([`composition-frontier-probes.md`](composition-frontier-probes.md),
instruments `sim_export` + meter `check_one`, main @ `7cec5ca9`):

1. **The single bus has a structural ceiling, and it is the wall — not
   a gradual sag.** Ground truth (sim, 432k warmup, converged): the
   uncapped ec-from-ore bus produces **97.9% of plan at 90/s** (WARN
   −2.1%; 437/450 machines working, a small real residual). At 120/s
   the *plan itself* exceeds express lane physics — 18
   `lane-throughput` validator errors, lanes planned at 23.2–25.1/s
   against the 22.5/s physical cap. That wall is plan arithmetic, not
   a measurement: no layout improvement reaches plan past it. Only
   running k units each below saturation does.
2. **The output-side refusal gap is live, sim-confirmed.**
   `copper-cable 90` ships a 0-error layout that the sim FAILs at
   **−50.2%** (44.80/90 delivered, converged; 2 machines
   output-blocked, 6 starved): a 90/s single-item target cannot leave
   the bus on one express belt, and the validator does not say so.
   This is the output-side sibling of the RFC-069 Phase C refusal,
   deferred there (#723 round-1 adjudication) — a prerequisite for any
   composer that merges cell outputs, and this RFC's Phase 1.
3. **The seam-cost motivation was tested and DID NOT SURVIVE** — kept
   here as the record of why this RFC is smaller than its first draft.
   The meter measured composed two-stage fixtures 5–18 points below
   plan; the sim anchors both (zero-margin and full-margin) at
   **exactly plan** (+0.0% produced, PASS). Per-unit receipts DO
   survive composition inside today's bus; there is no embedded-stage
   defect and no Phase-1 provisioning work. The false signal became a
   documented meter divergence class
   ([`meter-divergence.md`](meter-divergence.md) §2026-08-25:
   turn-path under-read) — the decision-log entries of 2026-08-25
   carry the full forensic chain.

## Design

### The boundary contract (Phase 1 prerequisite, Phase 2 load-bearing)

A `CellBoundary`: input and output ports, each `(item, rate, belt_tier,
edge_side, offset)`, plus the constraint vector celldb already derives.
The recon (2026-08-25, decision log) found the engine holds **three
positional port shapes and none carries a rate**: `cells::extract::Port
{edge, x, y, item, inbound}` (extract.rs), `BoundaryRecord {item, x, y,
direction, is_fluid, entity}` on `LayoutResult` (models.rs), and
celldb's `Port {dx, dy, kind, item}` with `CellEntry` carrying
`provenance` + `sim_anchor` (celldb.rs) — celldb's `check_entry`
actively rejects stored rate stamps, and among *planner-level port/
boundary shapes* only `MegaPlan` pairs items with rates
(`outputs: Vec<(String, f64)>`, mega.rs; entity- and solver-level
rates exist — `PlacedEntity::rate`, `ItemFlow::rate` — but neither is
a boundary contract). Identifier-level references are deliberate: the
fine line spans drift (#725 round 1); the types are the anchor. So `CellBoundary` is the promotion of celldb's `Port` with
a typed rate and the entry's existing `sim_anchor` receipt — an
extension of the store the regression corpus already guards, not a new
parallel type. Two rules carry the correctness weight:

- **One compatibility oracle.** Whether two boundaries mate — items match,
  rates satisfy, a merge shape is stampable — is answered by ONE function,
  which for merge shapes delegates to `stamp_plan_for_shape` (the RFC-069
  oracle). The coprime-trap class was born from two parallel predictions
  drifting; the composer never re-creates that shape.
- **Standardized port geometry.** Cells present full-belt ports on a fixed
  grid so inter-cell routing degenerates to belt-butting and short
  straight trunks. The composer never invokes global routing — that
  boundary is a kill criterion (K72-4), not an aspiration, because
  relocation-and-reroute is the shape RFC-057/058/064-P3 died on.

### Phase 1 — the output-side refusal (was: embedded-stage provisioning, CLOSED BY MEASUREMENT)

The first draft's Phase 1 (boundary-style provisioning for embedded
stages, chasing a 5.4-point "embedding cost") is closed: both composed
fixtures sim at exactly plan and the cost was the meter's turn-path
artifact — the decision log of 2026-08-25 carries the full forensic
chain, and `meter-divergence.md` §2026-08-25 carries the divergence
class. No embedded-stage work happens under this RFC.

Phase 1 is now the sim-confirmed real defect: **the output-side
refusal**, the RFC-069 Phase C follow-up (#723 round-1 adjudication).
A single-item target whose planned output rate exceeds what its
boundary belts can carry ships a 0-error layout that delivers half
plan (`copper-cable 90`: sim FAIL −50.2%). The fix mirrors Phase C's
input-side shape: at plan time, refuse by name when the target's
output rate exceeds the output boundary's carrying capacity at the
effective tier (per-item stacking-aware, duty-scaled — the Phase C
ceiling machinery reused on the output flows), or provision more
output belts where the boundary contract allows. Same asymmetry
discipline: the gate must never over-fire (full suite + calibration
bank green). This is also the composer's prerequisite — merging cell
outputs inherits the half-plan lie at every seam unless cells refuse
outputs they cannot ship.

### Phase 2 — homogeneous replication

One recipe target beyond the wall: `ec@240 = 6 × ec@40` — the quantizer's
own split — cells tiled on a grid, inputs fanned out and outputs merged
through stamp-oracle-vetted balancer shapes. The recon found the
quantizer already exists: the cell chain's `required_copies` splits a
target into K copies against `QUANTUM_RATE = 45.0` (so 240/s → 6 copies
at 40/s each) and plans each stage at `outputs[0].rate × count / K`
(chain.rs, `required_copies` + the per-stage rate at the compose loop) —
so Phase 2 promotes an in-tree mechanism from a density-losing candidate
to the above-the-wall composer, rather than inventing replication. The
quantum itself is a Phase-2 tunable, not a constant to defend: the
status ledger shows small cells deliver near plan (ec22 sims 99.4%)
while 40–60/s buses carry the family's ~10% gap, so the
delivery-optimal quantum may sit well below 45 and the fixture's copy
count follows the measurement, not the constant. The
output-side refusal (the deferred RFC-069 follow-up) ships here as a
prerequisite: a composer that merges outputs must refuse a cell whose
output cannot leave its boundary, or it inherits the cable-90 half-plan
lie at every seam.

### Phase 3 — heterogeneous composition and the library

Chains of unlike cell groups; celldb-backed reuse with receipts as
library rows in the calibration bank; the web surface (rates past the
wall, cell outlines in the renderer). Scoped deliberately loose until
Phases 1–2 adjudicate — this phase is not committed by this RFC's
acceptance.

### Rejected alternatives

- *Scale the single bus.* Measured structurally impossible past the lane
  wall (Motivation 1); the RFC-069 campaign already spent the layout-side
  levers.
- *Relocation-and-reroute composition.* The RFC-057/058/064-P3 graveyard;
  K72-4 exists to keep this RFC out of it.
- *Hand-off machinery (transfer contracts, seam validators as the fix).*
  Measured loss-free twice; designing against it would be spending on the
  measured non-problem.

## Kill criteria

- **K72-1 — RETIRED 2026-08-25** (recorded, not deleted: kill criteria
  are falsifiable claims and this one's premise was falsified before it
  could gate anything). It demanded lifting `dis-ec20-comp` from 94.6%
  to ≥98% on the meter; the sim anchors that fixture at 100.0%
  produced already — the 94.6% was instrument artifact
  (`meter-divergence.md` §2026-08-25).
- **K72-2 — RETIRED with K72-1** (its subject phase closed by
  measurement). Its principle — never worse by the SIM on any
  calibration-bank row — is inherited verbatim by K72-6.
- **K72-6 (Phase 1 refusal asymmetry).** If the output-side refusal
  fires on any fixture the sim shows delivering ≥95% of plan (the gate
  over-fires on a buildable config), or regresses any sim-anchored
  calibration-bank row, it reverts — the RFC-069 Phase C rule: a
  refusal gate must never over-fire.
- **K72-3 (Phase 2 pays), two-part so a trip is attributable** (#725
  round 1: an absolute bar would mis-read an inherited per-cell gap as
  replication overhead; #726 round 1: both parts on the same trust
  basis — the original (a) anchored on the meter's 88.9% at 120/s,
  a number this RFC's own evidence disqualified). **(a) Validity, by
  plan arithmetic (no measurement):** the composed `ec@240` plan must
  carry ZERO `lane-throughput` errors — the wall the single bus
  cannot cross at that rate (18 such errors at 120/s) — and its sim
  delivery must not fall below the family's sim-anchored single-bus
  receipt (97.9% at 90/s, the standing bar until a 120/s sim
  anchor exists): if k quantized cells plus merges deliver worse
  than one bus at its best, composition pays overhead without gain.
  **(b)** Composed delivery must sit within 2 points of the
  constituent cell's own standalone SIM receipt at the chosen
  quantum (sim, not meter — the meter's below-plan direction is not
  trusted on turn-heavy fixtures, `meter-divergence.md`
  §2026-08-25) — else the seams cost
  more than the interface contract allows, whatever the per-cell level
  is. A trip on (a) alone with (b) clean means the quantum is wrong
  (per-cell gap inherited), not that replication failed — re-quantize
  before killing.
- **K72-4 (no global routing).** If Phase 2's composer needs the ghost
  router across cell boundaries — anything beyond butt-joints, straight
  trunks, and stamped merge shapes — the standardized-port premise
  failed; stop. This is the RFC-057-shape tripwire.
- **K72-5 (runtime).** End-to-end runtime >2× on the existing corpus
  drops the offending phase even if correctness improves.

## Verification plan

Per the CLAUDE.md layout-engine protocol, with one amendment this RFC
itself forced: **the meter's below-plan direction is NOT trusted on
turn-heavy fixtures** (`meter-divergence.md` §2026-08-25 — a divergence
class this RFC's own Phase 0 discovered), so **sims anchor every
claim and every phase gate**: the output-refusal fixtures at Phase 1
(the cable-90 specimen plus the never-over-fire sweep), the composed
ec@240 fixture at Phase 2, and any calibration-bank row the changes
touch re-blessed only through the bank refresh protocol. The meter
remains the fast iterator for capacity-bound questions, where it
measured accurately. Seam checks (boundary rates match, ports align)
land as validator checks with per-instance positioned issues
(`validator-reporting.md` rules). Trace events for composer decisions so
the snapshot debugger sees them.

## Phasing

- **Phase 0 — evidence (COMPLETE 2026-08-25, twice).** The frontier and
  seam probes plus the ec20 disambiguation; then the sim-anchor pass
  that overturned the seam-cost premise and established the turn-path
  meter divergence. All numbers in `composition-frontier-probes.md`;
  the adjudications in this decision log.
- **Phase 1 — the output-side refusal.** Standalone value (closes the
  sim-confirmed half-plan hole); gates on K72-6.
- **Phase 2 — replication composer.** Gates on K72-3/K72-4; Phase 1 is
  its prerequisite.
- **Phase 3 — heterogeneous + library.** Not committed by acceptance;
  opens only on a clean Phase 2.

## Decision log

- *2026-08-25 — RFC opened on the completed Phase-0 evidence.* The ec20
  disambiguation adjudicated the mechanism split: ~11 of ec30's 16.5
  composition points were boundary tightness (recovered by margin
  alone), but embedding still costs 5.4 points at full margin — so
  Phase 1 targets embedded-stage provisioning, with boundary margin as
  the secondary lever. The hand-off is measured loss-free and gets no
  design spend (rejected-alternatives entry). Output-side refusal
  sequenced into Phase 2 as a prerequisite, on the cable-90 live
  specimen. Two read-only recon tasks dispatched (cell machinery
  inventory; embedded-vs-boundary provisioning divergence) to pin
  Phase 1's exact touch points before implementation.
- *2026-08-25 — recon adjudicated into the design (both reports,
  in-repo-verified identifiers).* Three findings reshaped the draft:
  (1) no existing port type carries a rate, and celldb's `check_entry`
  actively rejects rate stamps — `CellBoundary` is therefore a
  promotion of `celldb::Port` + `sim_anchor`, not a new parallel type;
  (2) the embedded-vs-boundary divergence is NOT in consumer-side or
  trunk-count provisioning (identical on both paths) but in three
  internal-only loss points — producer sideload single-lane fill,
  `ret:`-sideload single-lane trunk merges, balancer partial-load —
  which become Phase 1's enumerated mechanism list, instrumented
  point-by-point against K72-1 so the 5.4 points get attributed;
  (3) the cell chain already quantizes at `QUANTUM_RATE = 45.0` via
  `required_copies`, so Phase 2 is a promotion of an in-tree
  density-losing mechanism, not greenfield replication. Also recorded:
  RFC-055's `CellVariant`/`boundary_records` are design prose only
  (its compact-ordering implementation was deleted per the owner's
  #632-A2 extension, offpath Tier 2, #675) — the "de-facto interface
  spec" the 2026-07-24 strategy memory assumed is thinner than
  recorded, which raises this RFC's value rather than lowering it.*
- *2026-08-25 — Phase 1's first attribution, from the meter's own
  per-recipe accounting on the K72-1 fixture (`dis-ec20-comp`):* cable
  machines show `item_shortage_ticks = 0` (the plate boundary with
  margin is fully clean) but are output-inserter-blocked for ~27% of
  machine-ticks (696,800 of 2,592,000, plus 139,200 output-full),
  while the circuit machines starve for cable at exactly 5.4% of
  machine-ticks (92,808 of 1,728,000) — matching the delivered deficit
  to the decimal. The 5.4 points are **producer-output-side blocking**:
  loss points 1–2 (sideload single-lane fill, `ret:` trunk merges)
  implicated, loss point 3 (balancer) and input margin exonerated.
  Phase 1 starts at the producer output path.*
- *2026-08-25 — #725 round 1 adjudicated: the quantizer contradiction
  fixed, K72-3 restructured for attributability, precision items
  taken.* The 2/3 major was right: `4 × ec@60` contradicted the
  promoted quantizer (`required_copies` at 45 yields 6 × 40/s, and 60
  exceeds the quantum) — fixture corrected, and the quantum recorded
  as a Phase-2 tunable (ec22's 99.4% sim receipt says the
  delivery-optimal quantum may sit well below 45). The 1/3 K72-3
  critique was taken in restructured form: an absolute 94% bar would
  mis-read an inherited per-cell gap as replication overhead, so
  K72-3 is now two-part — beat the wall (a), seams within 2 points of
  the constituent cell's own receipt (b) — with a trip on (a)-alone
  routed to re-quantization, not a kill. Precision fixes: K72-1's bar
  phrasing (98% is two points below the boundary leg's measured
  100.0%), line references loosened to identifier level (fine spans
  drift), the rate-pairing claim restricted to planner-level boundary
  shapes, the ~67% boundary-load claim itemized (copper 67%, iron
  44% — the review's own 6.7/s recomputation was arithmetic error:
  ec20 draws 30/s of copper plate, the export manifest's boundary
  line). Probes-doc header reclassified from "absorb when written" to
  the RFC's retained measurement record.*
- *2026-08-25 — Phase 1 forensics complete: the mechanism is drop-point
  contention (head-loading vs mid-loading), one suspect withdrawn.* New
  instrument `crates/meter/examples/lane_heatmap.rs` (whole-map
  per-lane occupancy + splitter routing counters + RECT path dump —
  the questions trace_belt's boundary-feed walker cannot answer for
  internal items). Chain on the K72-1 fixture: (1) the family
  balancer is EXONERATED — the cable path has no splitter at all (two
  producer rows cross-feed two consumer rows via turn-taps; trunks run
  at 50–75% occupancy, flowing); (2) the one blocked splitter (iron
  tap, input lane 1 at 1350/1350 both_blocked) is saturation
  backpressure on a 45/s boundary trunk against 20/s demand — zero
  iron starvation, a saturated input behaving saturated, suspect
  withdrawn per the instrument-before-finding rule; (3) the loss is at
  the producer DROP POINTS: the row's output belt is mid-loaded —
  each output inserter needs a far-lane gap under which upstream
  machines' items already stream, so the last machines in line face
  60–80% local lane occupancy and stall in bursts (the attribution's
  27% output_inserter_blocked), netting exactly the 5.4%. The
  sideload bridge itself works as designed (both lanes fill west of
  the merge). An external boundary belt is HEAD-loaded — compression
  arrives pre-formed, no mid-run gap-hunting — which is why the
  boundary leg measures 100.0% at identical average rates. Phase 1's
  fix direction is therefore drop-point headroom on embedded rows'
  output collectors (split collectors / more parallel output belts
  per row — the output-side cousin of RFC-069's trunk provisioning),
  NOT merge-shape work.*
- *2026-08-25 — **GROUND TRUTH OVERTURNS THE PHASE-1 PREMISE**: the
  sim anchors the K72-1 fixture AT PLAN; the meter's 94.6% — and the
  entire drop-contention mechanism above — is instrument artifact.*
  The scaling discriminator first refuted load-proportional
  contention (2-row geometries block ~27% at 67% load while a
  single-row 9-machine collector at 100% load blocks ZERO), pointing
  at the meter's turn-path handling; per the
  instrument-before-finding rule the fixture went to the sim before
  any engine change. Verdict (288k warmup, converged, drift +0.0%,
  validator clean, all 20 machines working): copper-cable produced
  **60.00/60.00 (+0.0%)**, electronic-circuit produced **20.00/20.00
  (+0.0%)**, delivered 19.73 (−1.3% edge effects) — PASS. There is
  no embedding cost at full margin; the composed pair delivers plan
  in the real game. Consequences: (1) K72-1 as written is MOOT — its
  fixture is already at its target in ground truth; (2) the meter's
  calibrated "below plan ⇒ believe it" asymmetry is FALSIFIED for
  this fixture class (suspects: the turn-path lane model, or the
  meter ignoring the manifest's realized inserter-capacity bonuses
  the sim honors — nb=1/bulk=3) — a new divergence class owed to
  `meter-divergence.md`, and every meter-only number in the Phase-0
  evidence (the uncapped frontier percentages included) inherits the
  suspicion; the validator-level findings (the 120/s lane-saturation
  errors) are plan-math and stand; (3) whether ANY seam cost is real
  now rests on the zero-margin ec30-comp fixture (meter 81.6%) —
  sim dispatched; if it too anchors at plan, Phase 1 closes by
  measurement with no defect to fix, and the RFC's substance is
  Phase 2 (above-the-wall composition, whose motivating wall is
  validator-real) plus the boundary contract.*
- *2026-08-25 — **Phase 1 CLOSED BY MEASUREMENT**: the zero-margin
  ec30-comp fixture also sims at plan.* Verdict (288k warmup,
  converged, drift +0.0%, PASS): electronic-circuit produced
  30.00/30.00 (+0.0%), delivered +1.3% — which requires the full
  90/s of internally produced cable to flow. The meter's 81.6% was
  artifact, so the ENTIRE seam-cost story (16.5 points at zero
  margin, 5.4 at full margin) is instrument error and there is no
  embedded-stage defect to fix. The artifact is GEOMETRY-CORRELATED:
  the one straight-line fixture (ec15-comp, single 65×11 row, no
  trunk turns) metered 99.7% while every turn-path fixture under-read
  by 5–18 points — the root-cause hint for the meter follow-up (the
  turn-path lane model, not a global capacity mismatch, since a
  global cause would have hit ec15 too). Remaining meter-only claims
  under sim adjudication before the RFC's motivation is rewritten:
  the uncapped frontier percentages (fp-ec90 dispatched, 432k
  warmup — deep chain) and the output-side half-plan specimen
  (seam-cable90 dispatched). The 120/s wall needs no sim: 18
  lane-throughput errors are plan arithmetic (planned lane rates
  above the 22.5/s physical cap).*
- *2026-08-25 — the last two anchors land; the RFC restructured around
  what survived.* fp-ec90 (432k warmup, converged, WARN): produced
  **97.9%** of plan (−2.1%; 437/450 working, 11 full-output, 2
  starved) — the uncapped bus is far better than the meter's 91.4%,
  and the small residual is real. seam-cable90 (converged, FAIL):
  delivered **−50.2%** — the output-side hole is REAL and
  sim-confirmed (and the meter was accurate here: capacity-bound
  failures are outside the turn-path artifact class). Restructure:
  Motivation rewritten on sim anchors with the retired seam-cost
  claim kept as Motivation 3 (the record of why the RFC shrank);
  Phase 1 is now the output-side refusal (Phase C's ceiling machinery
  mirrored onto output flows, never-over-fire discipline, K72-6);
  K72-1/K72-2 RETIRED with their falsification recorded in place;
  K72-3(b) re-based on SIM receipts; the verification plan trusts the
  meter only on capacity-bound questions. The meter's turn-model fix
  is deliberately NOT this RFC's scope — it is a meter-crate
  follow-up with the four anchored fixtures as its calibration set.*
- *2026-08-25 — Phase 1's mechanism corrected by its own recon: the
  output side is INNOCENT; the real defect is an input-tap
  disconnection that only warns.* The output-boundary recon found
  `merge_output_rows` already computes `n_output = ceil(total_rate /
  single_cap)` (output_merger.rs) — and the engine probe confirms
  cable-90 gets TWO express tails; ec60-red's two red tails ride the
  same mechanism. The "exactly one full belt" reading in Motivation 2
  was numerology: the sim's 44.8/s ≈ 10–11 working machines × 5/s,
  because SIX machines (the whole first row) are dead — their pickup
  belt is a 2-tile stub with no upstream path from the plate
  boundary, exactly what the fixture's six `belt-flow-reachability`
  warnings say, positioned per machine. "The validator does not say
  so" was also wrong: it says so at Warning severity, which nothing
  gates on, so the layout shipped 0E/6W. Phase 1 therefore targets:
  (1) the row-input tap bug — the topmost row's tap stamps a dead
  stub in this config class (single-recipe solve, external input,
  3 rows); (2) the severity adjudication for
  boundary-fed-reachability failures (a machine that can NEVER
  receive input is a delivery-zero defect; promotion follows the
  validator-trust protocol with its doc updated in the same PR).
  The refusal framing is retired — the engine provisions output
  capacity correctly and refuses nothing it can build.*
- *2026-08-25 — Phase 1 root cause: a NON-LAST tap whose splitter tile
  is occupied by the adjacent trunk column commits a SOURCELESS belt
  run.* Full chain from the trace + entity dump on cable-90: the
  boundary's 45/s enters one belt; the trunk-head splitter at (1,0)
  feeds two adjacent trunk columns (x=1, x=2); row 0's tap on lane 1
  at y=1 is a non-last tap, whose splitter must span (1,1)-(2,1) —
  but (2,1) is trunk 2's head. No splitter is stamped, the tap spec's
  entry tile is dropped at commit (the route even RECORDED the
  crossing: `GhostSpecRouted{tap:copper-plate:1:1,
  crossing_tiles:[(2,1)]}` — then nothing resolved it), and the
  surviving east run (3,1)→(8,1) has no upstream. Six machines dead;
  six `belt-flow-reachability` warnings say so per machine; nothing
  gates. The class boundary is proven in the same fixture: both LAST
  taps (trunk turns at y=8 and y=15) work — only non-last tap
  splitters with an occupied right-neighbor column break (ec20-comp's
  iron tap splitter at (1,17)-(2,17) worked because the copper trunk
  had turned east seven rows up). Fix shape, two mandatory parts:
  (1) LOUDNESS — the router fails the tap spec when its source cannot
  be stamped (sourceless commits are the silent-deficiency class this
  campaign exists to close), surfacing as GhostSpecFailed → retry or
  a named error; (2) AVOIDANCE — the lane planner assigns the
  cramped lane the last-tap/turn role and places non-last taps only
  where the splitter's second tile is free. Severity promotion for
  boundary-fed reachability failures remains on the list, adjudicated
  separately under the validator-trust protocol.*
- *2026-08-25 — Phase 1 unit 1 SHIPPED AND SIM-VERIFIED: the
  tap-assignment repair; unit 2 measured and characterized: the output
  merger's rate-blind partition.* The repair
  (`repair_tap_splitter_collisions`, lane_planner.rs — detection-gated,
  fires only on the collision class, `TapAssignmentRepaired` trace
  event, two pins) heals the specimen: validator 6W→0W/0E, meter
  44.6→73.9, **sim 44.8→74.40** (converged, zero starved machines).
  Full suite green at 1263 — detection-gating keeps every corpus
  fixture byte-identical. The residual −17.3% is DEFECT #2, isolated
  by the sim census (3 machines full_output × 5/s = the 15.6/s
  shortfall exactly): `merge_output_rows` computes `n_output =
  ceil(90/45) = 2` correctly but partitions whole producer rows
  rate-blind — 3 rows × 30/s into 2 groups puts 60/s onto a 45/s
  tail (45+30 = 75 ≈ 74.4 measured). The validator is CLEAN on the
  over-subscribed layout — a second sim-anchored silent-deficiency
  specimen, this one output-side. Fix directions for unit 2, in
  preference order: (a) consult the RFC-069 stamp oracle for a proper
  (rows→tails) balancer shape — which is verbatim what Phase 2's
  composer needs for merging cell outputs, so unit 2 IS the composer's
  merge primitive built early; (b) fallback, first-fit-decreasing
  packing with n_output raised until no tail over-subscribes (3 tails
  here — correct but belt-hungry). Plus the loudness follow-up: a
  merger-tail rate check so over-subscription is at least visible.*
- *2026-08-25 — #727 round 5: the tap unit's contract upgraded to
  REPAIR-OR-REFUSE, with the suite as the discriminating instrument.*
  The round's 3/3 major (the post-repair guard scoped owners to group
  members, missing non-member taps broken by a lengthened member
  column) is fixed by a global DIFFERENTIAL guard: colliding
  (owner, tap_y) pairs are collected across every solid 2+-tap lane
  before and after; the repair stands only if the group's own pairs
  cleared AND no new pair appeared — background model-noise pairs
  (cell-chain sub-layouts carry them) are compared against, not
  demanded away, which the suite forced (the absolute form refused
  healthy chain fixtures). The thrice-recycled loudness major is now
  CLOSED, not deferred: an unrepairable group is a named refusal
  ("tap assignment unrepairable"), upgraded from annotation on two
  receipts — no corpus fixture refuses except one, and that one was
  probed under a temporary restore flag and is GENUINELY broken (the
  am2@1 chain fixture's native build: an Error belt-dead-end plus
  seven reachability findings, four machines unfed and four unable to
  ship output). Its test asserted "native must build" over that
  wreckage since #541 — premise updated with the probe receipts in
  the test comment; the test's real object (default does not silently
  fall back to native) still pinned. Also scoped: merge-tap configs
  skip the repair entirely — mt taps use PRIORITY-splitter machinery
  the splitter-tile model does not describe (the mt yellow-cap
  fixture read as phantom collisions). Suite green at 1265.*
- *2026-08-26 — Phase 1 unit 2 SHIPPED AND SIM-VERIFIED AT PLAN: the
  capacity-aware merger partition.* `merge_output_rows` now sizes
  `n_output` by greedy first-fit over contiguous per-column rates
  (optimal for minimum contiguous groups) and assigns columns by the
  same packing — the count-based `base = n/m` split was the
  rate-blind partition that put 60/s on a 45/s tail. Two bugs found
  and fixed during implementation by the unit's own instruments:
  (1) my first assignment guard compared the wrong remaining-columns
  quantity and collapsed every column into group 0 — caught by unit
  1's three-tails pin failing with ONE tail, root-caused through the
  merger's committed geometry (two folds where zero belonged);
  (2) the zero-fold case (every group a single column) placed tails
  inside the row region — a pre-existing hole unreachable under
  count partitioning, fixed and scoped to `n_output > 1` so the
  single-tail corpus norm stays byte-identical (the unscoped fix
  tripped three cell-registry hash pins, whose own message demands
  sim re-verification — reverted to the scoped form instead).
  Verification: full suite 1265/0; the Phase-0 specimen delivers
  **90.00/90.00 produced (+0.0%), −0.4% delivered, sim PASS, all 18
  machines working** — the complete arc 44.8 (silent wreck) → 74.4
  (unit 1) → 90.0 (units 1+2). The choice of first-fit over the
  stamp-oracle balancer is a recorded sizing deviation from the log's
  preference order: the oracle-backed (rows→tails) merge IS Phase 2's
  composer primitive and supersedes this fold there; the fold is the
  correctness fix at unit scale.*
