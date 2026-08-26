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
2. **The −50.2% specimen (sim-confirmed) — SINCE RE-DIAGNOSED AND
   FIXED.** `copper-cable 90` shipped a 0-error layout the sim FAILed
   at −50.2%. The first reading ("a 90/s target cannot leave on one
   express belt — an output-side refusal gap") did not survive
   Phase 1's forensics: the real defects were an input-tap
   disconnection (six machines wired to a dead stub) plus the output
   merger's rate-blind partition — both fixed (#727, #728), the
   specimen now at plan (90.00/90.00 produced, PASS). Kept as the
   record of the motivating measurement; the decision log carries the
   re-diagnosis chain.
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

### Phase 1 — the specimen's two real defects (COMPLETE 2026-08-26; two earlier framings superseded)

Two framings died on the way here, both recorded in the decision log:
the first draft's "boundary-style provisioning for embedded stages"
(closed by measurement — the 5.4-point embedding cost was the meter's
turn-path artifact, `meter-divergence.md` §2026-08-25), and then "the
output-side refusal" (closed by forensics — the −50.2% specimen was
never an output-capacity refusal case; the engine's merger already
provisions `ceil(rate/cap)` tails).

What Phase 1 actually shipped, each sim-verified:

1. **The tap-assignment repair with a repair-or-refuse contract**
   (#727): a non-last tap whose splitter tile is occupied by an
   adjacent trunk column committed a SOURCELESS belt run (six dead
   machines, warnings only). `repair_tap_splitter_collisions`
   reassigns sibling consumers (detection-gated, differential global
   guard, restore-verified) or refuses by name — never silent.
2. **The capacity-aware merger partition** (#728):
   `merge_output_rows`' count-based fold put 60/s on a 45/s tail;
   `partition_columns` (greedy first-fit, column-order-correct,
   utilization-scaled true flows) sizes and assigns from one walk,
   with the voider path pinned single-tail.

Specimen arc: 44.8 → 74.4 → **90.00/90.00 produced, sim PASS**. The
never-over-fire asymmetry discipline was enforced across the review
rounds (the refusal upgrade shipped only on receipts). Residuals are
listed once, in the Phasing section.

### Phase 2 — homogeneous replication

One recipe target beyond the wall, composed as the quantizer actually
splits it (**corrected twice**: first from the draft's `6 × ec@40`
guess when Phase-2 recon showed the quantum applies to the chain's
TOTAL flow, intermediates included; then re-derived when unit 1
shipped `QUANTUM_RATE = 40.0` — at 40, ec@240's cable runs 720/s so
`required_copies` gives K = 18 > K_MAX, the unit-2 territory; the
shipped above-the-wall exemplars are ec150 = 12 × ec@12.5 and
ec75 = 6 × ec@12.5, both sim-verified at plan). Cells tile side by
side with per-copy feeds and drains; unit 2's grid/merge composer
owns rates past K_MAX. The recon found the quantizer already exists:
the cell chain's `required_copies` splits against the quantum
and plans each stage at `outputs[0].rate × count / K`
(chain.rs, `required_copies` + the per-stage rate at the compose loop) —
so Phase 2 promotes an in-tree mechanism from a density-losing candidate
to the above-the-wall composer, rather than inventing replication. The
quantum itself is a Phase-2 tunable, not a constant to defend: the
status ledger shows small cells deliver near plan (ec22 sims 99.4%)
while 40–60/s buses carry the family's ~10% gap, so the
delivery-optimal quantum may sit well below 45 and the fixture's copy
count follows the measurement, not the constant. The
output-capacity side is already sound: Phase 1's partition provisions
`ceil`-packed tails (the cable-90 half-plan lie is fixed), and the
composer's merge primitive is the stamp-oracle-vetted successor of
that same fold.

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
- **K72-6 — RETIRED 2026-08-26 with Phase 1's close-out** (recorded,
  not deleted, like K72-1/K72-2): its subject — an output-side
  refusal gate — never shipped, because forensics re-diagnosed the
  specimen (decision log) and the output side needed provisioning,
  not refusal. The criterion's PRINCIPLE (a refusal gate must never
  over-fire; regressions on sim-anchored bank rows revert) was
  applied verbatim to the tap unit's repair-or-refuse contract and
  enforced through #727's six review rounds.
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
- **Phase 1 — COMPLETE 2026-08-26** (#727 the tap-assignment repair
  with the repair-or-refuse contract; #728 the capacity-aware merger
  partition). The Phase-0 specimen's arc is the verification record:
  44.8 (silent wreck) → 74.4 → **90.00/90.00 produced, sim PASS**.
  **The canonical Phase-1 residual list** (log entries point here;
  none gating): (a) the boundary-fed-reachability SEVERITY PROMOTION
  (foreign-column collisions and any residual sourceless class made
  Error-severity under the validator-trust protocol — distinct from
  #727's "loudness" refusal contract, which SHIPPED); (b) the
  merger's zero-rate-column guard (`[0.0, 60.0]` mis-groups) and the
  D2b-secondary/scrap-row rate reads (pre-existing input-quality —
  the old `total_rate` read the same source); (c) the zero-fold
  continuation row's occupancy check (bridge like the east-extension
  path when Phase 2 touches the merger); (d) the at-cap fold and
  fractional multi-row meter readings; (e) the single over-cap
  column (the placer's per-row output ceiling domain).
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
  meter only on capacity-bound questions.

Known limitation (#730 round 2, recorded): a sim-FAILED geometry and a
never-measured one both rank as unverified — a three-tier ordering
(verified > never-measured > measured-failing) is future policy work,
not a silent choice; the FAIL rows' notes carry their receipts either
way. The meter's turn-model fix
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
- *2026-08-26 — #728 rounds 1–3 adjudicated into the unit.* Round 1
  (bot + codex jointly): the column-order reversal (the bot's 1/3
  critical — east extensions place row 0 RIGHTMOST, so un-reversed
  col_rates read the wrong row per column; the symmetric specimen
  masked it) and the codex HIGH (the ceil floor over the greedy count
  empty-grouped the assignment — the floor is gone, greedy first-fit
  is the sole authority, sizing and assignment in ONE walk via the
  extracted `partition_columns`). Round 2: merger-level end-to-end
  pins for the zero-fold branch and the reversal, the latter's
  discrimination EXECUTED (bug restored → pin fails [11,12] vs
  [11,13] → fix restored). Round 3: the **utilization multiplier is
  deliberate unit scope, now recorded here**: both `total_rate` and
  per-column rates apply `utilization_for` — the shared
  placement/validation single-source formula — because a fractional
  row's true steady flow is what the tails must carry (nominal-rate
  packing splits belts a fractional row cannot fill); pinned by the
  discriminating `merger_fractional_rows_pack_by_true_flow` (two
  22.5/s true flows fit one tail where nominal 50 would split). The
  placer's own unscaled row-output sizing is conservative oversizing,
  not a contradiction. Zero-fold tails share their tile with the
  continuation belt by the standing marker convention (flush starts
  at tail.y+1), now commented at the site.*
- *2026-08-26 — #728 round 4: the voider merge forces single-tail
  packing* (`force_single_tail` — the voider row has one input belt,
  so multi-tail packing at that merge strands columns; the flag
  preserves that caller's pre-partition behavior, and the hole
  predates the unit: the old ceil count could also multi-tail there).
  **Unit follow-ups recorded**: (a) an at-cap fold measurement (the
  exactly-full splitter merge from the asymmetric pin's {5+40}
  group); (b) a fractional multi-row end-to-end reading — both
  meter-instrument work (capacity-bound class, where the meter
  measured accurately), neither gating the unit (the partition
  arithmetic is directly unit-pinned; the geometry-change class's
  exemplar is sim-anchored at plan).*
- *2026-08-26 — #728 merged on a MAJORLESS round 5; **Phase 1
  COMPLETE**.* The round's five minors are recorded as unit
  follow-ups rather than bought as a sixth ~12-minute cycle (the
  stop-point rule's exact case): the `acc > 0.0` guard mis-groups a
  zero-rate column preceding an over-cap one (`[0.0, 60.0]` packs as
  one group — reachable only through the secondary-row rate-read
  class below); D2b-secondary and scrap-recycling columns can read
  `column_rate = 0.0` while genuinely flowing (pre-existing — the
  old `total_rate` read the same source on every path); the
  zero-fold continuation row stamps without an occupancy check
  (the east-extension path bridges; this branch should too when
  Phase 2 touches the merger); the single over-cap column's
  under-delivery is the placer's recorded domain. Phase 1 closes
  with both sim-anchored defects fixed and the specimen at plan:
  the campaign's deliver-or-refuse contract now holds on the
  input side (tap repair-or-refuse) and the output capacity side
  (partition), with loudness residuals recorded. Phase 2 — the
  composer — is the RFC's remaining committed substance.*
- *2026-08-26 — Phase 2 reconnaissance: the composer's build surface
  mapped by three probes.* (1) ec@240 today: the composed candidate
  is gated off by `chain_eligible` — "needs 16 quantized copies (max
  12 at quantum 45/s)". Sixteen, not the fixture's assumed six: the
  quantum applies to the chain's TOTAL flow (intermediates included),
  so copies are ~15/s-of-ec each and the `6 × ec@40` fixture in the
  Phase-2 section needs correcting to the real quantization
  arithmetic. `K_MAX = 12`'s own doc names this RFC's job: beyond it
  "the chain should be decomposed differently; refuse loudly" — the
  grid-tiling composer IS that different decomposition. (2) ec@150
  (K≈10, gate passes): the candidate RUNS and refuses in production —
  "copper-cable has 2 in-ports for copper-plate across 2 rows" (the
  multi-row internal corridor class its own test pins): at quantum 45
  a copy's cable stage is 9 machines > the 8-per-row cap. The
  native bus ships with 37 errors at that rate; above the wall the
  engine currently has NO error-free path. (3) A local
  quantum-40 experiment (reverted) makes the cable stage single-row
  but the candidate goes NotRun despite `chain_eligible` passing — a
  selection-laziness interaction to root-cause first. Phase 2's
  build order therefore: (i) root-cause the NotRun, (ii) the quantum
  as a real tunable with the single-row-per-stage constraint driving
  its value, (iii) the K_MAX successor (grid tiling + stamp-oracle
  merges) for rates the strip cannot honestly serve, measured
  against K72-3 as restated (zero lane-throughput errors; sim vs the
  standing 97.9% bar; seams within 2 points of the constituent
  cell's sim receipt).*
- *2026-08-26 — recon item (3) RETRACTED as my own instrument error,
  and the quantum-40 experiment measures AT PLAN.* The "selection-
  laziness NotRun" was a probe-reading artifact: each cell sub-solve
  emits its own seven-row board, and my head-limited grep read
  sub-boards as the outer one. The outer board decides
  **cell-composed at BestErrorFree** at ec@150/quantum-40 — the
  composer already wins above the wall once every stage fits one row
  per copy. Measurements on that config: 10,392 entities (2892×17 —
  the strip's footprint-honesty concern made visible), validator
  0E/0W, **meter 150.0/150.0 delivered** (at-plan clears nothing;
  the sim is running as the anchor). Phase 2 unit 1 is therefore the
  quantum change (45 → the single-row-per-stage value) shipped WITH
  its sim receipts and the calibration-bank re-bless its blast
  radius requires (every cell-chain bank row re-shapes); the K_MAX
  successor remains unit 2 for rates beyond ~150.*
- *2026-08-26 — the sim FAILS the quantum-40 composed strip at
  exactly −50.0%: unit 1 re-scoped from "the quantum const" to the
  chain's DRAIN capacity at high K.* Verdict (432k warmup): produced
  75.00/150 (−50.0%), delivered −48.3%, FAIL; census 378 machines
  full_output vs 374 working — HALF the factory output-blocked while
  the meter read 150.0 at plan (its at-plan direction proven
  worthless again, exactly as calibrated) and the validator was
  clean (0E/0W — a THIRD sim-anchored silent-deficiency specimen,
  this one in the chain's drain/corridor provisioning). The
  exactly-half signature on the output side at K≈12 points at the
  chain's final-drain/corridor capacity — the same subsystem as
  gear@20's yellow-hardcoded drain (#715), now at scale where the
  drain must carry 150/s. Unit 1 is therefore drain forensics
  (lane_heatmap on this fixture) + fix + the quantum, shipped
  together with sim receipts; K72-3(a) as it stands would fail this
  layout and correctly so.*
- *2026-08-26 — THE −50% VERDICT IS INVALIDATED AS A LAYOUT
  MEASUREMENT: the harness kit mis-placed every sink on this fixture
  class.* Kit-forensics (per the sim-kit-first-suspect rule) on the
  saved run: all 12 drain-rig chest banks sit displaced ≈ +dims_x/2
  (world x 237+241i vs exits at world 236+241i−1446), every bank
  EMPTY, kit_errors empty (the belts placed fine — on empty land);
  the 24 feed rigs are similarly off-frame; offx/offy themselves are
  correct (−1445/−8). "Copies 6–11 dead" was the kit's geometry, not
  the strip's — the chain's drain is UN-blamed and the previous
  entry's re-scope is superseded. The displacement does NOT affect
  narrow fixtures (fp-ec90 at 241 wide simmed 97.9% with working
  sinks), so the bug class is wide/multi-exit-specific — the
  composed-strip class is exactly what it breaks. Codex is on the
  frame forensic (scenario.rs; feed/drain calls vs the centered-paste
  offset); the ec75 6-copy strip sim now in flight will be read with
  the same lens (its rigs displace by its own dims/2 = 723).
  Phase 2's measurement path is BLOCKED on the harness fix — which
  therefore becomes part of unit 1, with the ec150 sim re-run as its
  verification.*
- *2026-08-26 — THE INVALIDATION IS ITSELF RETRACTED: the kit is
  exonerated by the codex frame forensic; the displacement was MY
  double conversion.* `sim_state.chests` coordinates are already
  manifest-framed (the dumper converts `floor(world − offx) + LX0`,
  scenario.rs:2020) — subtracting offx again manufactured the
  +dims/2 shift; and the drain banks read empty because the delivery
  counter EMPTIES registered chests each cycle by design
  (scenario.rs:2325). Rigs are correctly placed at every exit; the
  −50% sim verdict on the ec150 strip STANDS as a layout
  measurement. Two instrument-error lessons in one campaign day (the
  turn-path meter class, now this frame misread) — both caught by
  the instrument-before-finding rule applied to my own probes. The
  physical cause of "copies 6–11 output-blocked with working sinks"
  is REOPENED; the in-flight ec75 6-copy sim discriminates next
  (same class, half the width: at-plan kills scale-independent
  chain-mechanism theories; a repeat east-half block points at
  position-dependent layout structure). The harness gains one
  follow-up all the same: a `world` field beside the layout coords
  in sim_state dumps, so this misread class cannot recur.*
- *2026-08-26 — ROOT CAUSE, THIRD INSTRUMENT'S THE CHARM: the harness
  drain rigs' POWER runs out with rig index; the layout is innocent
  after all.* The discriminating pair: ec75 (6 copies) sims AT PLAN
  (75.00/75.00, +3.5% delivered, all 374 machines working) while
  ec150 delivered EXACTLY the same 77.6/s — the 12-copy run behaved
  as a 6-copy one. With frames corrected, the saved report shows
  exit 6's extension belts PACKED (8s) against exit 0's flowing
  (2-3): items reach the far drains; the bank inserters never pick.
  The rig places its substation+EEI at the extension HEAD
  (scenario.rs, `exit + lateral·4/7 + flow·1`) while the bank sits at
  `t = ext_len−8..ext_len` and `ext_len = 11 + 2·idx` — from rig ~6
  the bank leaves the substation's supply area: unpowered legendary
  stack inserters, full extension, blocked copy, kit_errors empty
  (placement all succeeded). FIXED: the substation/EEI now anchor at
  the bank's center (`t = ext_len−4`), covering every bank at any
  ext_len; the ec150 re-run with the fixed harness is in flight as
  the verification. If it lands at plan, the quantum-40 receipts
  complete and unit 1 (the quantum change + this harness fix + the
  bank re-bless) ships.*
- *2026-08-26 — the ec150 re-run lands AT PLAN (150.00/150.00, +1.9%
  delivered, all 748 working): unit 1's receipts complete; the
  re-bless adjudicates a MEASURED TRADE.* K72-3 clears both parts on
  this exemplar (plan-valid 0E; 100% ≥ the 97.9% bar; seams within
  bounds trivially). The quantum's blast radius, re-blessed with
  honest verdicts: chain-ec15-d2 — the **L0 d-sweep calibration
  geometry** (hash `8f2473ec`) measured in the shipping-default
  WORLD — now PASSES at plan, and d7 improved (−5.3% → −4.0%),
  while the low-bonus d1 calibration worlds REGRESS (ec15: −8.0% →
  −16.1%; ec30: −7.7% → −11.7%) — more copy hand-offs × the #383
  inserter plateau. **What production actually ships** is the
  different L2 geometry (`e442f54f`, the default compose at
  capacity 2), whose own receipt (chain-ec15g2, #730 round 3) is
  **WARN: produced 15.00/15.00 (+0.0%), delivered −4.0%** — the d7
  drain-tail class; "at plan" for the shipped geometry is a
  produced-side claim, never a delivered-side one. Adjudication: the regression is calibration-world-only
  and selection-shielded at shipping rates (the chain loses to HS/DI
  below the wall — the succession record in the ec15 test — and
  above the wall no old receipts existed), while the trade buys the
  engine's ONLY error-free path above ~120/s plus a produced-at-plan
  default world. Registry rows carry the FAIL verdicts openly with
  the trade note; the copy-count and refusal pins updated on
  semantics (the adjudicated zero-margin cable input is dissolved);
  the L2 self-consistency golden re-blessed per its own contract
  with the d2 PASS as corroboration. Suite 1272/0.*
- *2026-08-26 — #730 round 4 absorbed: the narrative catches up with
  the receipts, and the mismatch note goes deterministic.* The
  round's 3/3 major was prose, and correctly so: the entry above
  said "the shipping-default world PASSES at plan" while that
  receipt belongs to the L0 d-sweep calibration geometry — the
  SHIPPED L2 geometry's own receipt (chain-ec15g2, added in round
  3) is WARN, produced +0.0% / delivered −4.0%. Reworded in place
  (produced-side at plan; delivery carries the d7-class drain
  tail). Taken with it: (1) `verification_note`'s world-mismatch
  sibling is now picked deterministically with FAIL-dominance — one
  hash carries FAIL/PASS/WARN across worlds today, and file order
  must not decide the tier; an other-world PASS transfers nothing
  under #383, so mixed evidence ranks unverified until the actual
  world is measured (real-producer test pins both directions).
  (2) The chain's drain under-delivery warning is provably dead for
  ALL K, not just K≥2 as round 3's comment claimed — kept as
  defensive code, never an assert (a warning-class condition must
  not become a panic path); the #715 loud-exit contract now rides
  the quantization bound itself. (3) The selection-policy comment
  now names the three-tier reality. Recorded, not taken: the
  derived-quantum form min(belt, max single-row stage rate) was
  already the standing follow-up (the reviewer independently
  re-derived it); the harness drain-rig supply-area self-check
  joins the residual list as harness hardening — the fix shipped
  with its own verification (77.6 → 152.8 on the same fixture) and
  the assert is belt-and-braces, not a gap.*
- *2026-08-26 — the calibration-bank half of unit 1's promised
  re-bless, found red by CI four rounds in.* The `rust` check had
  been failing since round 2 on the calibration-bank fingerprint
  probe and nobody — not me (the review gate watches only
  `second-opinion`; the probe is an ignored test local `cargo test`
  never runs), not five bot rounds — looked. Quantum 40 re-shaped 7
  of the 35 bank rows: six ec fixtures (the exact blast radius this
  log promised to re-bless; the sim-registry half shipped in round
  1, this half was missed) and ac45. Re-blessed per ci.yml's
  golden-style protocol: fresh `calibration_matrix_export` (diff
  confirmed exactly the 7 rows), the 28 unchanged rows' reports
  adopted byte-verified from the standing measured bank, the 7
  re-measured in Factorio (432k warmup). THE TRADE, measured: no
  row regressed — ec22/ec30/ec30dp/ec60red reproduce their old
  delivered numbers to the decimal, ec23 holds at plan (99.4) —
  and two rows materially improved: **ac45 non-converged 63.7% →
  converged 99.6% PASS** (the phantom-warning fix + re-quantized
  internals) and **ec35's "overlapping kit chests" kit-error
  cleared** (93.7 → 96.0 measured clean). matrix.json and the
  evidence doc regenerate from the new bank
  (`/tmp/calibration-matrix-2026-08-26-q40`). Process lesson
  recorded in memory: arming the review gate is not watching CI —
  check the full `gh pr checks` list after every push on a
  geometry-moving PR.*
- *2026-08-26 — #730 MERGED (six rounds absorbed, all checks green;
  squash e58a95fc). Unit 1 complete. Carried review items land in
  unit 2's opening commit: the full-match-FAIL arm's real-producer
  test + the L0==L1 coincidence guard; the corridor pin's 40.0
  razor's-edge override moved to 39.5; the quantum comment scoped to
  the ec corpus (the single-row cap is recipe-specific — the derived
  form stays the recorded refinement); the full-match-precedence
  clarification in the selection-policy comment; the coupled
  tier/warning rate-argument comment at the drain; the registry
  JSON trailing newline. New standing residual (#730 round 6): the
  d1 FAIL rows (`8f2473ec` ec15, `f4b22018` ec30) are #383
  inserter-plateau verdicts, not geometry defects — when RFC-049
  Phase 3 inserter sizing lands, RE-MEASURE them rather than letting
  a sizing-era FAIL fossilize the hash's unmeasured worlds at the
  unverified rank tier.*
- *2026-08-26 — Phase 2 unit 2 design adjudicated: STACKED
  INDEPENDENT STRIPS.* The K_MAX successor was recon'd three ways
  (two codex read-only passes with file:line receipts). (1)
  Interior-corridor grids with central merges: real machinery for a
  merge nothing requires. (2) Through-columns (all records on the
  outer perimeter, trunks crossing inner strips): REFUTED — a shared
  feed column needs a 1→2 branch primitive at the port turn
  (stamp_path corners are flow turns, not splits); the inbound port
  band spans ~10 rows, more than one express UG hop (reach 8); and
  `CORRIDOR_GAP = 6` is a dynamic allowance consumed by
  merges/fan-out/bypass/poles with no reservation API — a
  through-drain cannot be certified from source. (3) The winner
  falls out of the shipped interface: strips already carry fully
  INDEPENDENT per-copy feeds and drains, so the composed artifact
  needs NO inter-strip flow. K > K_MAX splits into R = ceil(K/K_MAX)
  balanced strips; each composes exactly as today and translates by
  (0, r × (strip_h + CLEARANCE)) — entities AND boundary records
  (also fixing the recon-found mega-input y-translation gap at the
  boundary-record translation); one stamped vertical pole column per
  gap bridges the power networks (the power validator refuses
  islands). K72-4 is satisfied by construction — the only
  inter-strip geometry is a straight pole line. The Phase-2 section's
  "stamp-oracle merges" language was speculative design; per-copy
  exits ARE the shipped contract K72-3 measures, and merge machinery
  would be spend on a measured non-problem (the same adjudication
  class as Phase 0's hand-off finding). Implementation must settle:
  CLEARANCE sized for the harness kit (strip r's south drain rigs
  and strip r+1's north feed rigs share the gap; x-disjoint by slot
  construction; the rig ext_len growth `11 + 2·idx` and the 12-tile
  feed jog are the collision candidates — tune in the harness if the
  kit census shows contact); the refusal moves from K_MAX to a
  grid-level honesty bound (R_MAX = 4 → K ≤ 48, same loud-refusal
  wording contract). Exemplar: ec@240 = 2×9. FRESH WALL RECEIPTS
  (wall_probe.rs, local example; CELLCOMP off, AM3, from-ore): at
  150/s native ships 35 lane-throughput + 2 dead-end errors (the
  recorded "37"); at 240/s the failure MODE shifts — 0
  lane-throughput but a caught ghost-router panic ("template place
  failed: AlreadyClaimed") plus 13 dead-end + 9 unresolved-junction
  errors in 251 issues. K72-3(a)'s "zero lane-throughput errors" is
  therefore necessary but under-describes the 240 wall: the unit-2
  bar is validator-0E, recorded against the native panic. (The
  criterion's "18 such errors at 120/s" figure does not reproduce
  from plates on current main — the wall receipts are the from-ore
  numbers above; cite these, don't relitigate the old figure.)*
- *2026-08-26 — unit 2 ENGINE SHIPPED (grid composer + record-aware
  validators): the composed ec@240 grid validates at ZERO ERRORS —
  K72-3(a)'s plan bar — where native carries 22 structural errors
  around a caught router panic.* Shape: `compose_chain_with_capacity`
  dispatches K ≤ K_MAX to the unchanged strip composer and K > K_MAX
  to `compose_grid_with_capacity` — R = ceil(K/K_MAX) balanced strips
  (18 → 9+9), each a proportionally scaled `SolverResult` whose own
  `required_copies` is verified to land on the planned count, composed
  as today and translated by strip_h + STRIP_CLEARANCE (32) —
  entities AND boundary records — then `repair_pole_network` bridges
  the power islands and recomputes the wire graph; `CellGridComposed`
  trace event; the refusal moves to R_MAX × K_MAX = 48 (ec700, K=53)
  with the same wording contract (ec600's K=45 now composes as 4
  strips). Measured: ec@240 from ore = 2304×66, 16,600 entities, 36
  feeds / 18 exits, selection ships it at BestErrorFree; the only
  warnings are 18 zero-margin cable inputs (6 EC machines per copy can
  demand exactly 45/s against express — the honest class the sim
  adjudicates). **The finding on the way**: every validator's notion
  of "boundary" was the bounding-box EDGE — the single strip's records
  sat there by coincidence, so a grid's interior strip edges read as 9
  dead-end Errors, 48 flow-path and 502 reachability warnings.
  Corrected to record-aware semantics (`boundary_record_tiles`): a
  declared record tile is boundary wherever it sits, and
  `check_boundary_integrity` already holds every record to a real
  matching belt, so the exemption cannot be bought by an unbacked
  record (validator-trust.md rows updated; core suite unchanged
  elsewhere, 1274/0). Mega-input boundary records now translate y as
  well as x (the recon-found latent gap). Harness side (#731): the
  rig ladders banded + clustered, plus two structural refusals (feed-
  vs-drain, rig-vs-layout) — and the rig-vs-layout sweep over the
  real bank found a LATENT KIT DEFECT unrelated to grids: fluid feed
  heads that are bare pipes carry direction 0, so the harness builds
  their rigs INTO the layout; the bank's three "non-converged 0.000"
  rows (tier3 heavy-oil cracking / sulfuric acid / plastic bar) were
  never layout measurements. Fix is engine-side (`bus/layout.rs`
  records the head entity's direction — a pipe has none) with a
  3-row manifest re-bless + re-measure: a followup outside this RFC's
  scope, tracked on the status ledger. Next: the chain-ec240 fixture,
  its sim (the interior-edge kit, now structurally guarded), registry
  row + gates, K72-3(b)/K72-5, and Phase 2's close-out.*
- *2026-08-26 — the ec@240 grid sims: the GRID is exonerated by
  census, the deficit is the inherited cell gap, and K72-3's
  "re-quantize before killing" provision is taken twice.* Receipts
  (432k warmup, converged, kit clean, Factorio 2.0.77): **K=18**
  (2×9, 13.33/s per copy) produced 235.80/240 (−1.7%), delivered
  −4.0%, WARN — 18 machines ingredient-short, ONE per copy, 54
  full_output; **K=20** (2×10, 12/s per copy — the belt-margin
  re-quantization) produced 224.00 (−6.7%), delivered −13.3%, FAIL —
  40 short, TWO per copy. In BOTH runs the per-strip census is
  machine-for-machine identical (crafts 2770/2770 furnace,
  2740/2740 assembler at K=20): the two strips behave as one strip
  copied — interior edges, kit, clearance, pole bridge all clean.
  **The constituent alone** (ec@12, one copy, 849 entities) produces
  11.15/12 (−7.1%), delivered −8.6%, WARN — so K72-3(b) HOLDS (the
  composed grid is within 0.4 points of its constituent on the
  produced side) and (a)'s trip is the per-cell gap the criterion
  names. Mechanism (codex recon, inserter_ladder.rs `count_ladder`,
  common.rs `machine_feed_rate`): the row's ladder sizes each side
  by `required <= n·rate` with NO margin, crediting a long-handed
  hand 2.4/s at the default level; the EC row's far belt is IRON
  (the hungrier cable rides the near belt on stack hands), and a
  12/s copy's 5 EC machines at 96% draw iron at exactly 2.40/s → one
  far hand at 100% of its credit (two short per copy); K=18's copies
  sit at 92.6% of one hand (one short); the receipted ec150 cell at
  12.5/s draws 2.5 → TWO hands at 52% → plan. The general fix is
  RFC-049 Phase 3 (margin in the ladder). Unit 2's bounded response:
  `required_copies` gains an input-HAND margin (0.85 of the ladder's
  own credited capacity, asking `size_side`/`machine_feed_rate`, gated
  on plans the ladder believes cover so the receipted low-level
  worlds do not move) — SCOPED TO K > K_MAX after the registry gate
  caught the general form re-shaping the registered
  military-science-pack@5 strip: receipted strips keep their measured
  geometry, a grid's copies carry no receipt and are planned with
  margin. ec@240 → K=24 (2×12, the 4-machine 10/s cell, two hands at
  52%); strips now compose at the CALLER's planned count (the grid's
  scaled sub-result would re-derive a lower rate-only K — the
  verify-then-refuse tripped exactly there). **K=24 PASSES AT PLAN:
  produced 240.00/240.00 (+0.0%), delivered 249.60 (+4.0%),
  converged, kit clean, all 1,200 machines working — 600 per strip,
  zero short, zero output-blocked** (17,148 entities; registry hash
  `4f95009b0204275e`, row + gate config + probe entry). K72-3
  CLEARS on the exemplar: (a) the composed plan carries zero
  validator errors (native at 240/s: 22 structural errors around a
  caught router panic) and sim delivery sits above the 97.9% bar;
  (b) at-plan composition is within 2 points of the constituent
  family's at-plan receipts (ec75/ec150 cells) — and the K=18/K=20
  runs showed composition reproducing its constituent to 0.4 points
  even when the constituent was deficient. K72-4 holds by
  construction (the only inter-strip geometry is the pole bridge);
  K72-5: the existing corpus never enters the grid path, the
  exemplar solves+composes+validates in 0.08 s. Recorded, not fixed here:
  the delivered-vs-produced gap widens with layout size (ec12 alone
  −1.5 pts, K=18 −2.3, K=20 −6.6 — a transit/window effect at
  2304-wide grids, to be checked with `--fixed-window` before any
  delivered-side bar is read on grids); ec60 at K=5 has the same
  zero-margin iron hand and no registry row (pin annotated).*
