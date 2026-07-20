# Archived design docs

These documents describe **historical** project state — proposals,
investigations, and refactors that have either landed, been
superseded, or were paused. They are kept as prior art and decision
context but are **not current**.

For the live pipeline reference see:

- Top-level `CLAUDE.md` for the architecture overview.
- [`../ghost-pipeline-contracts.md`](../ghost-pipeline-contracts.md)
  for the phase-by-phase contract the ghost router upholds.
- [`../junction-solver-followups.md`](../junction-solver-followups.md)
  for the live junction-solver work-in-progress notes.

## What's here

| File | Status |
|---|---|
| `rfc-band-regions.md` | Rejected — SAT-on-bands produced cycles / phantom belts. Superseded by the junction solver. |
| `rfc-belt-flow-aware-astar.md` | Aspirational — still-valid design for a future spaghetti-routing revival. |
| `rfc-ghost-cluster-routing.md` | Implemented — ghost A* + union-find + cluster SAT (the current pipeline). |
| `rfc-ghost-occupancy-refactor.md` | Implemented — typed `Occupancy` map; see `crates/core/src/bus/ghost_occupancy.rs`. |
| `rfc-junction-solver.md` | Partially implemented — T1 perpendicular template + SAT fallback wired; T2/T3 pending. |
| `rfc-region-routing.md` | Partially implemented — framework + tier 1 in place; tier 2/3 pending. |
| `rfc-remove-corridor-template.md` | Implemented — corridor-template pre-pass removed (`fcec676`); junction solver is now the only path for crossings. |
| `rfc-streaming-reconciliation.md` | Implemented — streaming renderer reconciles to `layout.entities` at finish (`d845e56`). |
| `rfc-veto-directed-growth.md` | Implemented — walker-informed growth policy in the junction solver (`1ef2a7f`). |
| `rfc-multi-fluid-rows.md` | Implemented — stacked-T multi-fluid input rows (`c3a9cbb`). |
| `phase-f-sat-zone-editor.md` | Implemented — in-canvas SAT-zone editor with tier-1/tier-2 validation (`fd0f170`); lives in `web/src/ui/satEditor.ts`. |
| `handoff-2026-04-15.md` | Session handoff after the topology-aware ghost-router exploration; historical snapshot. |
| `sat-band-investigation.md` | Investigation notes explaining why SAT bands failed. |
| `lane-column-packing-investigation.md` | Rejected — PR #160's bus-width packer; crossing resolution would need to be rewritten before column-sharing is safe. |
| `spaghetti-roadmap.md` | Original spaghetti-router roadmap from before the pivot to bus/ghost routing. |
| `port-plan.md` | Original Python → Rust port plan. |
| `port-status.md` | Per-unit Python → Rust port status at the time the port wrapped up. |

Anything you need from these that isn't in the current docs, you can
read directly — just remember they describe a point in time, not
today's behaviour.
