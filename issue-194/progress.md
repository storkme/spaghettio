# Progress on issue #194

## What was done
1. Read PR #191 full diff (Phase 0 scaffolding): `LayoutStrategy` enum, `LayoutOptions`,
   `module_id` in `LaneFamily`/`BusLane`/`FamilyInfo`, golden hash regression tests,
   WASM + web UI wiring.
2. Read PR #193 full diff (Phase 1 demand-partitioning): new `partitioner.rs` (543 LOC),
   `ItemFlow.module_id`, `placer.rs` module_id inheritance, partitioned e2e tests,
   trace events (`ModulePartitioned`, `PartitionRejectedByUtilization`).
3. Read the RFP (`docs/rfp-modular-production.md`) in full.
4. Checked current codebase state — agent branch is at same commit as main, PRs not
   yet merged.
5. Ran `cargo test` on current branch — all tests pass.
6. Posted comprehensive review comment on issue #194.
7. Added `agent-done` label.
8. Owner (storkme) responded with verification results.

## Review findings summary

### PR #191: Ready to merge ✅
- Clean abstraction, golden hash regression is smart, LOC budget respected.
- Minor: `unimplemented!()` for non-Pooled is correct Phase 0 stopgap.

### PR #193: Mostly ready ✅

#### Stale findings (owner confirmed)
1. **"K1-4 inertness test is missing"** — STALE. `tests/e2e.rs:492` defines
   `assert_partitioned_inertness`, auto-invoked from `assert_golden_hash:476`.
   Every K=1 e2e test re-runs under `PartitionedPerConsumer` and asserts
   byte-identical output against the Pooled golden hash.
2. **"HierarchicalComposed prototype is absent"** — STALE. Deliberately dropped
   from RFP (decision log 2026-04-25). Hand-authoring a Factorio-correct 12→12
   balancer is fiddly; the trade-off is explicit: without a pre-built fallback,
   reassess rather than swap on K2-2.

#### Valid nit (owner fixed in 1f4066b)
- **`ModuleAssignment.utilization` doc comment** — clarified to spell out
  "per-lane saturation" instead of ambiguous "saturation fraction."

#### Real bug (owner found and fixed in 1f4066b)
- **Capacity table mismatch** — `partitioner.rs` used 2× values
  (15.0/30.0/45.0) vs `lane_planner.rs` (7.5/15.0/22.5 per-side). The
  partitioner's K1-3 utilization gate was 2× too permissive (triggered at
  11.25/s instead of 5.625/s on yellow belt). Also surfaced the
  `split_overflowing_lanes` `module_id: 0` hardcoding bug — both fixed in
  commit `1f4066b`.

### Key strengths
- `(item, module_id)` tuple keying is orthogonal to spatial layout — good for
  future escargio (folded layouts) RFP.
- `LayoutOptions` absorbing `max_belt_tier` is the right design for future options.
- Recipe-level granularity is a sensible conservative choice.
- Golden hash regression (K0-1) is a smart approach to snapshot drift detection.

### Latent concerns (noted, not blocking)
- `SolverResult` cloning in `apply_partition_plan()` is fine for now but could
  be a concern for very deep chains.
- Recursive up-tree partitioning deferred (RFP step 6) — shared-upstream
  distributor relies on existing pool-balancer mechanism for PR2.
