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
   disambiguation pair (2026-08-25): with every boundary at ~67% belt
   load, the assembly stage delivers **100.0%** when its cable arrives as
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
edge_side, offset)`, plus the constraint vector celldb already derives
(RFC-067's port contracts are the existing shape to promote — exact type
reuse to be pinned from the machinery recon before implementation). Two
rules carry the correctness weight:

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
way an external boundary input's entry path is provisioned (margin
included). The exact divergence points between the internal and boundary
paths are being pinned by recon (lane planner trunk sizing vs entry-belt
sizing; tap-off vs straight feed); the mechanism lands wherever the two
paths measurably differ. This phase pays standalone — today's cell-chain
and DI candidates carry these same embedded seams.

### Phase 2 — homogeneous replication

One recipe target beyond the wall: `ec@240 = 4 × ec@60` cells tiled on a
grid, inputs fanned out and outputs merged through stamp-oracle-vetted
balancer shapes. The output-side refusal (the deferred RFC-069 follow-up)
ships here as a prerequisite: a composer that merges outputs must refuse
a cell whose output cannot leave its boundary, or it inherits the
cable-90 half-plan lie at every seam.

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
  on the meter (the boundary leg's own measured level), the
  embedded-stage hypothesis is wrong — stop Phase 1 and re-diagnose;
  no composer work starts on an unproven mechanism.
- **K72-2 (Phase 1 never-worse).** If the Phase-1 change regresses any
  sim-anchored calibration-bank row by more than 1% delivered (sim, not
  meter — "never worse means never worse by the sim", the #520 lesson),
  it reverts regardless of what it wins elsewhere.
- **K72-3 (Phase 2 pays).** If the composed `ec@240` does not exceed 94%
  delivered on the meter — beating the single-bus structural ceiling
  measured at 120/s (88.9%) by ≥5 points — replication overhead ate the
  win; stop before Phase 3.
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
