# RFP: Cache-first junction probing

## Summary

Reorder the junction solver's growth loop so it consults the zone cache for *every* candidate region shape before invoking varisat on any of them. Today the loop interleaves cache-checks with SAT calls inside `try_solve_on_region`, so the loop happily burns ~190ms proving an early variant UNSAT before discovering a later variant (or a deeper expansion of the same seed) had a cached solution all along. Phase 1 batches the lookups within one growth iteration; phase 2 extends the probe N steps deep into hypothetical expansions, so the loop fast-forwards to known-good geometry instead of walking incrementally past reproducing UNSATs.

## Motivation

Profiling the slowest e2e test, `stress_advanced_circuit_partitioned_7s_from_plates` (113s wall, single-threaded), shows the test is **dominated by SAT calls that prove UNSAT for zone shapes already known to be infeasible-or-cached-elsewhere**:

| Metric | Value |
|---|---|
| Wall time | 113.6s |
| Trace events emitted | 1200 |
| `SatInvocation` events | 353 (290 misses + 63 cache hits) |
| Total SAT-miss time | **55.7s** (~49% of wall) |
| Real successful SAT solves | **0** (every committed junction was solved via cache) |
| Unique UNSAT zone shapes | 45 |
| Top 8 shapes' UNSAT repeats | **14× each** within a single test run |

Per-junction breakdown (one test run, 7 junctions, 6 committed + 1 capped):

```
Junction 1 @  (6, 19):  29 attempts  UNSAT= 25  CACHE-HIT= 4   committed via CACHE-HIT
Junction 2 @ (10, 19): 127 attempts  UNSAT=127  CACHE-HIT= 0      capped (gave up)
Junction 3 @  (6, 28):  24 attempts  UNSAT= 19  CACHE-HIT= 5   committed via CACHE-HIT
Junction 4 @  (6, 37):  29 attempts  UNSAT= 19  CACHE-HIT=10   committed via CACHE-HIT
Junction 5 @  (7, 46):  44 attempts  UNSAT= 36  CACHE-HIT= 8   committed via CACHE-HIT
Junction 6 @  (2, 56):  43 attempts  UNSAT= 32  CACHE-HIT=11   committed via CACHE-HIT
Junction 7 @  (3, 67):  57 attempts  UNSAT= 32  CACHE-HIT=25   committed via CACHE-HIT
```

Every successful junction's solution came from the cache. The SAT calls that succeed at finding a fresh layout are zero. The 290 UNSAT solves are the cost of the framework discovering, one variant at a time, that the cached variant was the answer.

The relevant code comment (`junction_solver.rs:1138`) reads *"Variants run sequentially (SAT on these zones is low-ms) and order is fixed so the trace is deterministic."* The "low-ms" assumption was reasonable when the comment was written but isn't true today — average UNSAT solve is ~190ms in debug. The interleaving pattern was sized for a regime where serial-SAT-per-variant was nearly free.

## Design

### Phase 1: Cache-first within current iteration

Today, per growth iteration, `junction_solver.rs:1130-1209` runs:

```
for variant in [primary, N, S, E, W]:
    try_solve_on_region(variant)        # internally: cache-lookup → on miss, SAT-solve
collect successful candidates
pick cheapest
```

Phase 1 splits this into two passes:

```
# Pass 1 — cache-only
for variant in [primary, N, S, E, W]:
    if region viable AND cache_hit(variant):
        candidates.push((cost, cached_solution, variant))

# Pass 2 — SAT only for variants that didn't hit cache
for variant in [primary, N, S, E, W]:
    if variant not in candidates:
        try_solve_on_region(variant)    # → may invoke SAT
        on success, candidates.push(...)

pick cheapest from candidates
```

This is strictly semantics-preserving: same set of variants tried, same cost-minimization, same trace-event order on the SAT path. The only behavior change is that variants whose *cached* answer would have been found are skipped over the SAT codepath entirely. No schema changes, no new public APIs.

To enable this we factor a helper out of `try_solve_on_region`:

```rust
fn probe_cache_for_region(region: &GrowingRegion, ctx: &SolveCtx)
    -> Option<(canonical_form, Vec<PlacedEntity>, cost)>;
```

It does what `try_solve_on_region` does up to and including `lookup_zone`, but bails on cache miss instead of falling through to SAT. The existing `try_solve_on_region` keeps its full path for the pass-2 fallback.

### Phase 2: Multi-step speculative lookahead

Per growth iteration, after pass 1, if no cache hit was found among the 5 immediate variants, speculatively probe expansions 2 steps deep (5² = 25 hypothetical regions) using only `probe_cache_for_region`. If any hit, fast-forward the growth loop to that region by applying the corresponding chain of `expand_bbox` calls and committing the cached solution.

Lookahead depth is a tunable constant; start with `LOOKAHEAD_DEPTH = 2` and revisit. The cost ceiling is bounded — each probe is ~1ms (junction-build + canonicalise + lookup), and 5^2 + 5^3 = 150 probes per iteration is still ~0.15s vs the ~190ms cost of a single UNSAT solve.

Three concerns to resolve in the design phase:

1. **Determinism.** Multiple cache hits at different depths or sides need a fixed tie-breaking rule. Proposed: prefer shallower depth; within a depth, prefer cheapest cost; within equal cost, prefer fixed variant order (primary → N → S → E → W).
2. **Cost optimality vs. cache hits at deeper depth.** If depth-1's primary is cache-miss (UNSAT or unknown) but depth-2's expand-east is cached at cost 50, do we take it, or invoke SAT on depth-1 first to see if a cheaper depth-1 layout exists? Proposed: take the cache hit. Cached solutions are post-cost-descent (i.e. optimal for their shape); the empirical frequency of fresh SAT producing cheaper layouts than any cached variant is zero in this corpus, and the 190ms-per-fresh-SAT cost is steep insurance.
3. **Veto-tile accumulation.** The growth loop's existing fallback uses `veto_tiles` from failed walker checks to drive uniform growth. Fast-forwarding past intermediate iterations means we don't see those veto tiles. This only matters if no cache hit exists anywhere in the lookahead — we then fall through to the original walker-driven growth path, which still works correctly because pass 2 still runs.

### Phase 3 (deferred): cache UNSAT signatures

For the J2-style "all attempts UNSAT, capped, retry the whole layout" pathology, even a cache-first probe finds nothing. Caching UNSAT signatures (extending the cache schema with a one-byte tag for "known infeasible at these constraints") lets repeated proves-of-no-solution become free lookups. Deferred because phase 1+2 already capture the bulk of the wins on the existing corpus, and a schema change deserves its own RFP after we measure whether phase 1+2 was enough.

## Kill criteria

Apply after phase 1 ships, before deciding whether to proceed to phase 2.

1. **No measurable wall-clock improvement on the slow tail.** If `cargo nextest run -p fucktorio_core --profile ci` total runtime drops by less than 20% (currently 275s local single-threaded, target ≤220s) after phase 1, the SAT-on-UNSAT cost wasn't the dominant bottleneck and we abandon both phases. Re-profile to find the real cost.
2. **Cache-hit rate doesn't change.** If the cache-hit:cache-miss ratio in `partition_strategy_scoreboard`'s snapshot stays at 17.8% post-phase-1, the reorder didn't actually capture more cache hits — something about the variant-shape signatures means our scan isn't seeing what `try_solve_on_region` eventually sees. Investigate before phase 2.
3. **Layout regression on any CLEAN test.** If any of the 13 CLEAN-class e2e tests goes from 0 errors / 0 warnings to non-zero, the cache-first reorder broke determinism or cost optimality. Hard kill — revert.
4. **Phase 2 implementation requires more than ~300 LOC of speculative-expansion plumbing.** That signals the lookahead semantics are fighting the existing growth loop, and the small win isn't worth the architectural debt. Stop after phase 1.

## Verification plan

Per [verification protocol for layout engine changes](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Full e2e suite green.** `cargo test --manifest-path crates/core/Cargo.toml`. All 9 non-ignored e2e tests must remain green; the 11 SCOREBOARD tests must not regress beyond their recorded baselines.
2. **Trace-event-driven validation, not just error counts.** After phase 1, dump snapshots for `stress_advanced_circuit_partitioned_7s_from_plates` and `tier5_processing_unit_2s_horizontal_stack_iron_ore_pipe_bypass`. Decode and confirm `SatInvocation` count drops, and the cache-hit fraction rises. The "looks faster, error count unchanged" case where validation was actually skipped is exactly the trap CLAUDE.md warns about — verify the trace events directly.
3. **Wall-clock measurement.** Record `Summary [Xs]` from `cargo nextest run --profile ci` before and after; target ≤220s local single-threaded total.
4. **WASM build.** `wasm-pack build crates/wasm-bindings --target web --out-dir "$(pwd)/web/src/wasm-pkg"` clean, then load the test cases in the web app and visually confirm layouts are unchanged.
5. **Clippy.** `cargo clippy --lib -D warnings` clean before merge.

## Phasing

- **Phase 1** — cache-first batching within current iteration. Single PR. Self-contained.
- **Phase 2** — multi-step speculative lookahead. Separate PR after phase 1 lands and the kill criteria are checked. Defers the schema change in phase 3.
- **Phase 3** — UNSAT caching. Separate RFP if phases 1+2 don't hit the runtime target.

## Decision log

- *2026-05-02 — drafted; awaiting acceptance.*
- *2026-05-03 — Phase 3 (UNSAT caching) promoted from deferred to active and implemented in PR #289 without a separate RFP. The deferral logic was "phase 1+2 should be enough"; Day 1 of a focused-week measurement effort found that ~70% of cache misses are zones where SAT returns UNSAT or times out — these are inherently uncacheable under the v1 schema, so no amount of cleverer cache lookup (Phases 1+2) can recover that traffic. Phase 3's schema change was the prerequisite. Result post-implementation: gauntlet hit rate 30% → 100%, e2e suite wall time 91 s → 29 s (3× speedup). Phases 1+2 (cache-first probing, speculative lookahead) remain valuable for saving SAT calls on cache **hits** that we still re-discover via the strategy ladder, but lower priority than Phase 3 was — pencilled in as Day 4/5 candidates of the same focused week.*
