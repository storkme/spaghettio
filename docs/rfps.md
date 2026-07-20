# RFP registry

Assigned numbers for every RFP, ordered by first commit (same-day ties
alphabetical). **Existing files keep their names** — the number lives here
(renames would break decision-log cross-references and commit-message
links). **New RFPs**: take the next number, name the file
`rfp-NNN-short-name.md`, and add a row here in the same commit.

Statuses: Active / Complete / Design / Superseded / Archived (rejected or
obsolete; file lives in `docs/archive/`). Unmarked = predates the registry;
backfill the status next time the doc is touched.

| # | First commit | Doc | Status |
|---|---|---|---|
| RFP-001 | 2026-04-11 | [`rfp-belt-flow-aware-astar.md`](archive/rfp-belt-flow-aware-astar.md) | Archived |
| RFP-002 | 2026-04-12 | [`rfp-ghost-cluster-routing.md`](archive/rfp-ghost-cluster-routing.md) | Archived |
| RFP-003 | 2026-04-12 | [`rfp-junction-solver.md`](archive/rfp-junction-solver.md) | Archived |
| RFP-004 | 2026-04-13 | [`rfp-band-regions.md`](archive/rfp-band-regions.md) | Archived |
| RFP-005 | 2026-04-13 | [`rfp-ghost-occupancy-refactor.md`](archive/rfp-ghost-occupancy-refactor.md) | Archived |
| RFP-006 | 2026-04-14 | [`rfp-region-routing.md`](archive/rfp-region-routing.md) | Archived |
| RFP-007 | 2026-04-21 | [`rfp-remove-corridor-template.md`](archive/rfp-remove-corridor-template.md) | Archived |
| RFP-008 | 2026-04-21 | [`rfp-veto-directed-growth.md`](archive/rfp-veto-directed-growth.md) | Archived |
| RFP-009 | 2026-04-21 | [`rfp-unified-belt-specs.md`](rfp-unified-belt-specs.md) |  |
| RFP-010 | 2026-04-22 | [`rfp-multi-fluid-rows.md`](archive/rfp-multi-fluid-rows.md) | Archived |
| RFP-011 | 2026-04-22 | [`rfp-streaming-reconciliation.md`](archive/rfp-streaming-reconciliation.md) | Archived |
| RFP-012 | 2026-04-24 | [`rfp-modular-production.md`](rfp-modular-production.md) |  |
| RFP-013 | 2026-04-25 | [`rfp-horizontal-trunks.md`](rfp-horizontal-trunks.md) |  |
| RFP-014 | 2026-04-25 | [`rfp-pipe-belt-junctions.md`](rfp-pipe-belt-junctions.md) |  |
| RFP-015 | 2026-04-25 | [`rfp-renderer-particle-container.md`](rfp-renderer-particle-container.md) |  |
| RFP-016 | 2026-04-27 | [`rfp-junction-solver-capability.md`](rfp-junction-solver-capability.md) |  |
| RFP-017 | 2026-04-27 | [`rfp-validator-emission-units.md`](rfp-validator-emission-units.md) |  |
| RFP-018 | 2026-04-29 | [`rfp-decomposition-search.md`](rfp-decomposition-search.md) |  |
| RFP-019 | 2026-04-29 | [`rfp-fluid-dual-input-row.md`](rfp-fluid-dual-input-row.md) | Design (open, #68) |
| RFP-020 | 2026-04-30 | [`rfp-balancer-runner.md`](rfp-balancer-runner.md) |  |
| RFP-021 | 2026-05-01 | [`rfp-balancer-graph-place.md`](rfp-balancer-graph-place.md) |  |
| RFP-022 | 2026-05-01 | [`rfp-balancer-place-routing.md`](rfp-balancer-place-routing.md) |  |
| RFP-023 | 2026-05-01 | [`rfp-cp-sat-placement.md`](rfp-cp-sat-placement.md) |  |
| RFP-024 | 2026-05-01 | [`rfp-inline-balancer-placement.md`](rfp-inline-balancer-placement.md) |  |
| RFP-025 | 2026-05-01 | [`rfp-mx2-merge-generator.md`](rfp-mx2-merge-generator.md) |  |
| RFP-026 | 2026-05-01 | [`rfp-throughput-priority-merges.md`](rfp-throughput-priority-merges.md) |  |
| RFP-027 | 2026-05-02 | [`rfp-balancer-bake-lane-validation.md`](rfp-balancer-bake-lane-validation.md) |  |
| RFP-028 | 2026-05-02 | [`rfp-balancer-jh-search.md`](rfp-balancer-jh-search.md) |  |
| RFP-029 | 2026-05-02 | [`rfp-balancer-spatial-pruning.md`](rfp-balancer-spatial-pruning.md) |  |
| RFP-030 | 2026-05-02 | [`rfp-cache-first-junction-probe.md`](rfp-cache-first-junction-probe.md) |  |
| RFP-031 | 2026-05-02 | [`rfp-lane-aware-routing.md`](rfp-lane-aware-routing.md) |  |
| RFP-032 | 2026-05-02 | [`rfp-lane-safe-synth.md`](rfp-lane-safe-synth.md) |  |
| RFP-033 | 2026-05-03 | [`rfp-ug-sideload-prevention.md`](rfp-ug-sideload-prevention.md) |  |
| RFP-034 | 2026-07-10 | [`rfp-solver-net-flow.md`](rfp-solver-net-flow.md) | Complete |
| RFP-035 | 2026-07-11 | [`rfp-fulgora-scrap.md`](rfp-fulgora-scrap.md) | Active |
| RFP-036 | 2026-07-11 | [`rfp-lane-demand-flow.md`](rfp-lane-demand-flow.md) |  |
| RFP-037 | 2026-07-12 | [`rfp-inserter-sizing.md`](rfp-inserter-sizing.md) | Complete (in-game anchor open) |
| RFP-038 | 2026-07-13 | [`rfp-merge-tap-trunks.md`](rfp-merge-tap-trunks.md) | Active |
| RFP-039 | 2026-07-14 | [`rfp-validation-explainability.md`](rfp-validation-explainability.md) | Complete |
| RFP-040 | 2026-07-19 | [`rfp-power-supply.md`](rfp-power-supply.md) | Complete |
| RFP-041 | 2026-07-20 | [`rfp-build-quality.md`](rfp-build-quality.md) | Complete (in-game anchor open) |
| RFP-042 | 2026-07-20 | [`rfp-power-reservation.md`](rfp-power-reservation.md) | Complete |

Next number: **RFP-043**.
