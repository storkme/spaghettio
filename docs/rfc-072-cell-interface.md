# RFC-072: The cell interface — boundary contracts and rate-scaling composition

Status: Proposed
Evidence base: [`composition-frontier-probes.md`](composition-frontier-probes.md)
Successor framing to: the 2026-07-24 strategy call ("bus stays the
low-rate/intra-cell winner; high rates via composition")

## Summary

Give cells a typed boundary contract — ports as `(item, rate, belt tier,
edge position)` with an attached measurement receipt — and make the engine
honor it in three steps: provision *embedded* producer stages the way
boundaries are provisioned (closing the measured 5.4-point embedding cost),
compose k replicated cells to reach rates the single bus structurally
cannot (measured wall between 90 and 120/s uncapped), and extend the
deliver-plan-or-refuse-by-name contract (RFC-069) to the composed level.
The strategic payoff is the verification inversion RFC-067 promised: a
sim-anchored cell library turns per-layout verification cost into a
reusable asset, and the celldb power law (top 5 motifs = 87.7% of machine
mass) says a small library covers most demand.

## Motivation

All numbers are measured and reproducible today
([`composition-frontier-probes.md`](composition-frontier-probes.md),
instruments `sim_export` + meter `check_one`, main @ `7cec5ca9`):

1. **The single bus has a structural ceiling.** Uncapped ec-from-ore
   delivers 88.7–91.4% through 120/s with no cliff, but at 120 the plan
   itself exceeds express lane physics (18 `lane-throughput` errors,
   lanes planned at 23.2–25.1/s against the 22.5/s cap; cable clamps at
   exactly 320/s). No layout improvement reaches plan past the wall —
   only running k units each below saturation does.
2. **Embedding costs delivery, and the mechanism is isolated.** The ec20
   disambiguation pair (2026-08-25): with every boundary at or under
   ~67% belt load (copper-plate 30/45, iron-plate 20/45), the assembly stage delivers **100.0%** when its cable arrives as
   a boundary input and **94.6%** when the same cable is produced
   internally — the internal cable stage under-delivers its own plan
   (56.78/60), and the circuit output tracks it exactly. At zero boundary
   margin (ec30) the gap widens to 16.5 points. The hand-off itself
   converts loss-free in both experiments (#724 round 1) — the cost is in
   how an embedded producer's output rate is planned, not in the
   transfer.
3. **The composed-level refusal gap is live.** `copper-cable 90` ships a
   0-error layout that delivers exactly one full express belt (45.0/s,
   half plan) — the output-side sibling of the RFC-069 Phase C refusal,
   deferred there (#723 round-1 adjudication) and required here before
   any composer merges cell outputs.

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

### Phase 1 — boundary-style provisioning for embedded stages

Close the 5.4-point embedding cost inside the existing bus, before any
composer exists: provision an embedded producer stage's output path the
way an external boundary input's entry path is provisioned. The recon
pinned the divergence precisely — consumer-row input belts are provisioned
identically regardless of source (`row_input_belt`, placer.rs:32–50), and
trunk *counts* match too (internal 60/s at express gets the same three
20/s trunks external entry does, lane_planner.rs:886–933 vs 917–933).
The internal path's exclusive loss points, none of which an external
boundary has:

1. **Producer output lane filling** — some producer row kinds sideload
   their output belt, filling one physical lane (placer.rs, `row_output_belt` sideload path; the
   `sideload_bridge` both-lane path exists but is gated by
   `can_lane_split()`, placer.rs).
2. **Producer-to-trunk `ret:` sideloads** — non-topmost producer returns
   merge into one physical lane of the trunk
   (ghost_router.rs, the `ret:` walk).
3. **Balancer partial-load** — a non-throughput-unlimited family
   balancer can deliver less than its input supply
   (template_validate.rs); the RFC-069 pad guarantees a stamp
   *exists*, not that it is throughput-unlimited at the operating point.

Phase 1's mechanism is to make the internal producer→consumer path
boundary-shaped at these three points (both-lane fill, capacity-split
returns, throughput-unlimited-or-refuse merge shapes), instrumented one
point at a time against the K72-1 fixture so the 5.4 points get
attributed, not just removed. This phase pays standalone — the native
bus wins these configs today (selection recon: cell-chain loses on
density, DI displaces only when strictly better), so the fix lands on
the shipping path.

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

- **K72-1 (Phase 1 mechanism).** If boundary-style provisioning of the
  embedded cable stage does not lift `dis-ec20-comp` from 94.6% to ≥98%
  on the meter (two points of margin below the boundary leg's measured
  100.0%), the embedded-stage hypothesis is wrong — stop Phase 1 and
  re-diagnose; no composer work starts on an unproven mechanism.
- **K72-2 (Phase 1 never-worse).** If the Phase-1 change regresses any
  sim-anchored calibration-bank row by more than 1% delivered (sim, not
  meter — "never worse means never worse by the sim", the #520 lesson),
  it reverts regardless of what it wins elsewhere.
- **K72-3 (Phase 2 pays), two-part so a trip is attributable** (#725
  round 1: an absolute bar would mis-read an inherited per-cell gap as
  replication overhead). **(a)** The composed `ec@240` must beat the
  single-bus structural ceiling measured at 120/s (88.9% meter) — else
  composition lost to the wall it exists to cross. **(b)** Composed
  delivery must sit within 2 points of the constituent cell's own
  standalone meter receipt at the chosen quantum — else the seams cost
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

Per the CLAUDE.md layout-engine protocol. The meter iterates (its
below-plan direction is calibrated); **sims anchor every phase gate**:
the ec20 pair after Phase 1 (K72-1's meter verdict confirmed by sim
before the phase closes), the composed ec@240 fixture at Phase 2, and
any calibration-bank row the changes touch re-blessed only through the
bank refresh protocol. Seam checks (boundary rates match, ports align)
land as validator checks with per-instance positioned issues
(`validator-reporting.md` rules). Trace events for composer decisions so
the snapshot debugger sees them.

## Phasing

- **Phase 0 — evidence (COMPLETE 2026-08-25).** The frontier and seam
  probes plus the ec20 disambiguation; all numbers in
  `composition-frontier-probes.md`.
- **Phase 1 — boundary-style provisioning.** Standalone value; gates on
  K72-1/K72-2.
- **Phase 2 — replication composer + output-side refusal.** Gates on
  K72-3/K72-4.
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
