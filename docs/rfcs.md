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
| RFC-053 | 2026-07-24 | [`rfc-053-direct-insertion-cells.md`](rfc-053-direct-insertion-cells.md) | Active — **Phases 0 + 1 + 2 landed**; engine emits fused DI cells (stacked and horizontal-row). Both TOP corpus pairs build, validate at 0 issues and sim at/above plan: `copper-cable → electronic-circuit` 101.3%, `electric-furnace → electric-furnace` 109.5%. KC1–KC4 evaluated and passing (KC6 fired → pipes re-scoped into Phase 2, still outstanding). Inert by default. Remaining: pipes/fluids, Phase 3 multi-band, Phase 4 wasm+UI. |
| RFC-054 | 2026-07-25 | [`rfc-054-fast-meter.md`](rfc-054-fast-meter.md) | Design (circulated for review) |
| RFC-055 | 2026-07-26 | [`rfc-055-compact-cell-chain.md`](rfc-055-compact-cell-chain.md) | Superseded — experiment complete, selected over RFC-056 same day (weighted distance −16.3–39.6%; physical belts −10.1–17.3% on 3/4 fixtures, +8.5% on USP); Factorio gates never adjudicated, never shipped. Superseded in practice by RFC-057's broader compaction work; see `compaction-retro-2026-07.md` |
| RFC-056 | 2026-07-26 | [`rfc-056-folded-cell-chain.md`](rfc-056-folded-cell-chain.md) | Superseded — rejected at its own admission gate 2026-07-26 (only chem5raw cleared; pu4raw +11.1% distance/+78.7% critical path); RFC-055 selected instead |
| RFC-057 | 2026-07-26 | [`rfc-057-topology-preserving-dense-repacking.md`](rfc-057-topology-preserving-dense-repacking.md) | Parked (2D placement vindicated; transport layer falsified) — first materialized candidates cost +38–250% bbox vs the bus they replaced (logistics 6–8× machinery). The RFC's own 2026-07-30 decision log: "do not pursue tree-based local manifolds further ... build the single-lane shared-trunk tier first ... the 2D placement itself is vindicated — it is the transport layer that is uneconomic." Never rebuilt; `compact_layout` (undergroundify-only, no manifold trees) ships opt-in, default false. Follow-up funded in RFC-063 |
| RFC-058 | 2026-07-30 | [`rfc-058-band-packing.md`](rfc-058-band-packing.md) | **Concluded 2026-07-31 — KC1 fired in phase 4.** Phases 0–3 cleared their gates (KC2 36.8%; spike −35.9%), but the real planner under physically-legal routing holds only −27.0% vs the −33.0% bar (faithful instrument per two #523 review rounds: criterion-scope non-pole extents, honest footprints, candidate scoring bypassed), trajectory adverse as correctness increased. Falsified: 2D band packing holding ≥33% under legal single-lane trunk routing. Inert scaffolding + flag-gated builder remain in-tree as the record. Tracking #507 |
| RFC-059 | 2026-07-30 | [`rfc-059-di-coupling-assignment.md`](rfc-059-di-coupling-assignment.md) | **Complete — `Downstream` shipped.** Split out of #473. Corpus sweep over 3 machine tiers: 179 targets contend; after #520 fixed the validator's starved-pickup blind spot, `Downstream` is never worse and better on 6. Flipped on IN-GAME evidence, not validator parity: on `big-electric-pole@1` am2 the old default sims at **0.51/s against a planned 1.00/s** and `Downstream` at 1.10/s. P2/P3 dropped — no per-target assignment beats a static order. The motivating `rail` case never contends |
| RFC-060 | 2026-07-30 | [`rfc-060-horizontal-stack-candidate.md`](rfc-060-horizontal-stack-candidate.md) | Active — HorizontalStack as default-on scored candidate (never-worse contract, mirrors DI). Evidence #513; **K60-3 sim gate cleared 2026-07-31** — no flipped case sims below native (native deadlocks at 0/s on ac5/ac7/pu3); absolute deficits on statically-clean winners are the #519 lane-flux validator gap (sibling of #520) |
| RFC-061 | 2026-07-31 | [`rfc-061-demand-matched-trunk-provisioning.md`](rfc-061-demand-matched-trunk-provisioning.md) | Active — #519 layout-side fix: pool producer outputs and balance across K trunk feeds (disjoint-subset partitioning measured to explain ac@5's 75%-of-plan exactly; probe evidence in the RFC) |
| RFC-062 | 2026-07-31 | [`rfc-062-multi-target-outputs.md`](rfc-062-multi-target-outputs.md) | Design — N simultaneous user-specified targets (e.g. `electronic-circuit@10/s` + `advanced-circuit@3/s` from ore in one factory); probe shows the cheap hand-sum shortcut silently undercounts a shared item, motivating a real multi-seed LP solve |
| RFC-063 | 2026-07-31 | [`rfc-063-compaction-primitives.md`](rfc-063-compaction-primitives.md) | **Concluded 2026-08-01 — all three phases adjudicated.** Phase A killed at A0 (≥25% bar unreachable, ≈8.1%/5.9% measured ceiling vs verified community-best balancer references); Phase B killed on paper (5.00–7.14% structural ceiling vs the escalated ≥25% bar); Phase C's DI-composed packing spike CLEARS its escalated −40.0% bbox bar on 2/3 gate fixtures (aggregate −49.9%, `sci1-ore`/`sci2-ore`; `pu1-plate` still refuses — a solver-coupling gap unrelated to #526's scope) but funds no production follow-on — un-sim-anchored, inherited RFC-058 correctness debt, and (per RFC-064's dual-recorded `AR_score`) an aspect-ratio REGRESSION on the same candidates. Bbox-area verdicts here do not transfer to RFC-064's aspect-ratio/transit framing (see RFC-064) |
| RFC-064 | 2026-07-31 | [`rfc-064-spaghetti-objective.md`](rfc-064-spaghetti-objective.md) | Design — successor-in-reframing to RFC-063; replaces bbox-area minimization with aspect-ratio + rate-weighted belt-transit scoring (entity count reported, non-gating) under sim-anchored never-worse gates. Owner-contested objective + "tetris model" (rows rigid, connection fabric flexible). Five phases: promote folding (Phase 1), undergroundify default-on (Phase 2), row-granularity rigid packing — RFC-058 rescored (Phase 3), row-flipping spike (Phase 4), bidirectional feeds spike (Phase 5); Phase 0 calibrates the scoring rule against the owner's judgment before any auto-selection is built |

Next number: **RFC-065**.
