# RFC registry

Assigned numbers for every RFC, ordered by first commit (same-day ties
alphabetical). **Existing files keep their names** — the number lives here
(renames would break decision-log cross-references and commit-message
links). **New RFCs**: take the next number, name the file
`rfc-NNN-short-name.md`, and add a row here in the same commit.

Statuses: Active / Complete / Design / Superseded / Archived (rejected or
obsolete; file lives in `docs/archive/`). Unmarked = predates the registry;
backfill the status next time the doc is touched.

| # | First commit | Doc | Status |
|---|---|---|---|
| RFC-001 | 2026-04-11 | [`rfc-belt-flow-aware-astar.md`](archive/rfc-belt-flow-aware-astar.md) | Archived |
| RFC-002 | 2026-04-12 | [`rfc-ghost-cluster-routing.md`](archive/rfc-ghost-cluster-routing.md) | Archived |
| RFC-003 | 2026-04-12 | [`rfc-junction-solver.md`](archive/rfc-junction-solver.md) | Archived |
| RFC-004 | 2026-04-13 | [`rfc-band-regions.md`](archive/rfc-band-regions.md) | Archived |
| RFC-005 | 2026-04-13 | [`rfc-ghost-occupancy-refactor.md`](archive/rfc-ghost-occupancy-refactor.md) | Archived |
| RFC-006 | 2026-04-14 | [`rfc-region-routing.md`](archive/rfc-region-routing.md) | Archived |
| RFC-007 | 2026-04-21 | [`rfc-remove-corridor-template.md`](archive/rfc-remove-corridor-template.md) | Archived |
| RFC-008 | 2026-04-21 | [`rfc-veto-directed-growth.md`](archive/rfc-veto-directed-growth.md) | Archived |
| RFC-009 | 2026-04-21 | [`rfc-unified-belt-specs.md`](rfc-unified-belt-specs.md) |  |
| RFC-010 | 2026-04-22 | [`rfc-multi-fluid-rows.md`](archive/rfc-multi-fluid-rows.md) | Archived |
| RFC-011 | 2026-04-22 | [`rfc-streaming-reconciliation.md`](archive/rfc-streaming-reconciliation.md) | Archived |
| RFC-012 | 2026-04-24 | [`rfc-modular-production.md`](rfc-modular-production.md) |  |
| RFC-013 | 2026-04-25 | [`rfc-horizontal-trunks.md`](rfc-horizontal-trunks.md) |  |
| RFC-014 | 2026-04-25 | [`rfc-pipe-belt-junctions.md`](rfc-pipe-belt-junctions.md) |  |
| RFC-015 | 2026-04-25 | [`rfc-renderer-particle-container.md`](rfc-renderer-particle-container.md) |  |
| RFC-016 | 2026-04-27 | [`rfc-junction-solver-capability.md`](rfc-junction-solver-capability.md) |  |
| RFC-017 | 2026-04-27 | [`rfc-validator-emission-units.md`](rfc-validator-emission-units.md) |  |
| RFC-018 | 2026-04-29 | [`rfc-decomposition-search.md`](rfc-decomposition-search.md) |  |
| RFC-019 | 2026-04-29 | [`rfc-fluid-dual-input-row.md`](rfc-fluid-dual-input-row.md) | Design (open, #68) |
| RFC-020 | 2026-04-30 | [`rfc-balancer-runner.md`](rfc-balancer-runner.md) |  |
| RFC-021 | 2026-05-01 | [`rfc-balancer-graph-place.md`](rfc-balancer-graph-place.md) |  |
| RFC-022 | 2026-05-01 | [`rfc-balancer-place-routing.md`](rfc-balancer-place-routing.md) |  |
| RFC-023 | 2026-05-01 | [`rfc-cp-sat-placement.md`](rfc-cp-sat-placement.md) |  |
| RFC-024 | 2026-05-01 | [`rfc-inline-balancer-placement.md`](rfc-inline-balancer-placement.md) |  |
| RFC-025 | 2026-05-01 | [`rfc-mx2-merge-generator.md`](rfc-mx2-merge-generator.md) |  |
| RFC-026 | 2026-05-01 | [`rfc-throughput-priority-merges.md`](rfc-throughput-priority-merges.md) |  |
| RFC-027 | 2026-05-02 | [`rfc-balancer-bake-lane-validation.md`](rfc-balancer-bake-lane-validation.md) |  |
| RFC-028 | 2026-05-02 | [`rfc-balancer-jh-search.md`](rfc-balancer-jh-search.md) |  |
| RFC-029 | 2026-05-02 | [`rfc-balancer-spatial-pruning.md`](rfc-balancer-spatial-pruning.md) |  |
| RFC-030 | 2026-05-02 | [`rfc-cache-first-junction-probe.md`](rfc-cache-first-junction-probe.md) |  |
| RFC-031 | 2026-05-02 | [`rfc-lane-aware-routing.md`](rfc-lane-aware-routing.md) |  |
| RFC-032 | 2026-05-02 | [`rfc-lane-safe-synth.md`](rfc-lane-safe-synth.md) |  |
| RFC-033 | 2026-05-03 | [`rfc-ug-sideload-prevention.md`](rfc-ug-sideload-prevention.md) |  |
| RFC-034 | 2026-07-10 | [`rfc-solver-net-flow.md`](rfc-solver-net-flow.md) | Complete |
| RFC-035 | 2026-07-11 | [`rfc-fulgora-scrap.md`](rfc-fulgora-scrap.md) | Active |
| RFC-036 | 2026-07-11 | [`rfc-lane-demand-flow.md`](rfc-lane-demand-flow.md) |  |
| RFC-037 | 2026-07-12 | [`rfc-inserter-sizing.md`](rfc-inserter-sizing.md) | Complete (in-game anchor open) |
| RFC-038 | 2026-07-13 | [`rfc-merge-tap-trunks.md`](rfc-merge-tap-trunks.md) | Active |
| RFC-039 | 2026-07-14 | [`rfc-validation-explainability.md`](rfc-validation-explainability.md) | Complete |
| RFC-040 | 2026-07-19 | [`rfc-power-supply.md`](rfc-power-supply.md) | Complete |
| RFC-041 | 2026-07-20 | [`rfc-build-quality.md`](rfc-build-quality.md) | Complete (in-game anchor open) |
| RFC-042 | 2026-07-20 | [`rfc-power-reservation.md`](rfc-power-reservation.md) | Complete |
| RFC-043 | 2026-07-20 | [`rfc-043-pole-band-thinning.md`](rfc-043-pole-band-thinning.md) | Complete (Phase 1; Phase 2 cross-row sharing deferred) |
| RFC-044 | 2026-07-21 | [`rfc-044-machine-modules.md`](rfc-044-machine-modules.md) | Complete (all 4 phases + KC2 in-game anchor; #321/#322/#323/#325) |
| RFC-045 | 2026-07-21 | [`rfc-045-pole-wire-modes.md`](rfc-045-pole-wire-modes.md) | Complete (browser eyeball open) |
| RFC-046 | 2026-07-21 | [`rfc-046-belt-stacking.md`](rfc-046-belt-stacking.md) | Complete (in-game anchor open; Phase 3 deferred) |
| RFC-047 | 2026-07-21 | [`rfc-047-lane-aware-tap-delivery.md`](rfc-047-lane-aware-tap-delivery.md) | Complete (browser eyeball open) |
| RFC-048 | 2026-07-22 | [`rfc-048-cell-composition.md`](rfc-048-cell-composition.md) | Phase 1 complete (PR #365) — GO for Phase-2 integration RFC |
| RFC-049 | 2026-07-22 | [`rfc-049-inserter-capacity-research.md`](rfc-049-inserter-capacity-research.md) | Complete (in-game anchor open; input-side data gap #343) |
| RFC-050 | 2026-07-22 | [`rfc-050-headless-sim-harness.md`](rfc-050-headless-sim-harness.md) | Complete (fluid feed CALIBRATED via #364 — first fluid factory PASS; fluid-pack sweep + #345 re-measure open) |
| RFC-051 | 2026-07-22 | [`rfc-051-cell-composition-integration.md`](rfc-051-cell-composition-integration.md) | Complete — default Candidate; sim registry; K-quantization (corridor-cap); EC-row re-measure waits on #381 |
| RFC-052 | 2026-07-23 | [`rfc-052-oil-mega-cell.md`](rfc-052-oil-mega-cell.md) | Design (circulated for review) |

Next number: **RFC-053**.
