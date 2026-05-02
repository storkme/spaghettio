# RFP: Smarter junction-height search in `compose_series`

## Summary

`compose_series` (in `crates/balancer-gen/src/main.rs`, line ~1533)
searches for a feasible junction height by climbing `jh = lo, lo+1,
lo+2, ...` linearly, each step with a stratified-but-still-generous
timeout (90s → 240s → 600s). After tonight's spatial-pruning landing,
**this climb is the dominant wall-time cost** — at jh=8 on (4, 9), the
pruned encoding correctly proves INFEASIBLE in ~quick time but
compose_series sometimes still spends most of the timeout budget
because the per-step timeout is sized for "find a layout", not "prove
infeasibility".

Replace the linear climb with a **two-budget search**: a short budget
(`SHORT_TIMEOUT`) per jh step on the climb to fast-track INFEASIBLE
proofs; promote to the full timeout only on the candidate jh (the
first that doesn't return INFEASIBLE under SHORT). This keeps the
correctness story intact (we never accept INFEASIBLE-as-promotion;
SHORT-INFEASIBLE is a real proof) while cutting the climb cost
substantially.

## Motivation

### Concrete failure / cost data

Pruner's measurements on (4, 9) Clos compose, post-spatial-pruning,
with fallback OFF (current default):

| jh | status | solver wall | step wall (jh-attempt) |
|----|--------|-------------|------------------------|
| 5  | INFEASIBLE | ~quick | 1.3s  |
| 6  | INFEASIBLE | ~quick | 1.5s  |
| 7  | INFEASIBLE | 3-6s   | 4-8s  |
| 8  | INFEASIBLE | 60-180s | 60-180s |
| 9  | OPTIMAL    | ~10s   | ~10s  |

The jh=8 step is the cliff — CP-SAT's INFEASIBLE proof for the
larger model takes 1-3 minutes even with pruning. With a 240s
budget configured at jh=8, that's ~80% of the budget burned just
confirming "no, this jh is too tight". The OPTIMAL hit at jh=9 takes
~10s. The total is dominated by **proving the floor**, not finding
the answer.

### Why now

- Spatial pruning just landed and made jh=9 fast (10× speedup at
  jh=9 alone vs unpruned). The infeasible-jh climb didn't get the
  same speedup, so it's now disproportionately expensive.
- Tier-5 shapes will be wider grids with more edges. The climb is
  harder for those, and the SHORT/LONG split scales naturally with
  problem size.
- The "smarter-jh-search follow-on" was flagged in the spatial-pruning
  RFP decision log (entry by pruner) as the next dominant cost.

## Design

### Where the change lives

Single Rust function: `compose_series` in
`crates/balancer-gen/src/main.rs` (line ~1533). No Python changes,
no schema changes.

### Algorithm

Replace the current `for jh in initial_jh..=max_jh { try once with
big_timeout }` with:

```
for jh in initial_jh..=max_jh {
    timeout = SHORT_TIMEOUT;
    loop {
        resp = solve(jh, timeout);
        match resp.status {
            OPTIMAL | FEASIBLE => return assemble(resp);     // done
            INFEASIBLE => break;                              // climb
            UNKNOWN => {
                if timeout >= LONG_TIMEOUT { break; /* climb */ }
                else { timeout = LONG_TIMEOUT; continue; /* retry same jh */ }
            }
        }
    }
}
return Err("no feasible jh in [lo, hi]");
```

**Constants (initial values, tune via experiment):**
- `SHORT_TIMEOUT = 30s`. Roughly 3× the typical jh=9 OPTIMAL solve
  time; long enough to find feasible if jh is the right one, short
  enough to fast-track infeasible jhs.
- `LONG_TIMEOUT = 600s`. Same as today's max stratified budget.
- The current 90s/240s stratification is replaced by 30s/600s
  uniform across jh levels; per-jh stratification was a workaround
  for not knowing in advance which jh would be the candidate.

### Three observable outcomes per jh attempt

1. **Short-budget INFEASIBLE** → real proof; advance to next jh
   immediately. This is the cheap-win case and the whole point.
2. **Short-budget OPTIMAL/FEASIBLE** → found it; return.
3. **Short-budget UNKNOWN** → ambiguous. Could be "infeasible but
   hard to prove" or "feasible but hard to find". Retry SAME jh
   with `LONG_TIMEOUT`. Outcomes from the retry feed back into the
   same three buckets:
   - LONG-INFEASIBLE → confirmed infeasible; advance.
   - LONG-OPTIMAL/FEASIBLE → confirmed feasible; return.
   - LONG-UNKNOWN → both budgets exhausted on this jh; treat as
     INFEASIBLE-equivalent and advance (matches today's behavior
     where compose_series already advances on UNKNOWN).

### Logging

Keep the existing `compose_series: jh={jh} status={status}
solver_elapsed=…s wall=…s timeout=…s` line. Add a `[SHORT]` /
`[LONG]` tag so the log distinguishes the two budgets. Preserves
debuggability.

### Trade-offs considered

- **Binary search.** Rejected: requires a known-feasible upper
  bound. We don't have one a priori (max_jh is "give up", not
  "guaranteed feasible"). Could synthesize one by always running
  jh=max_jh first as a "feasibility probe", but that doubles the
  worst-case wall time when the true floor is low.
- **Per-jh adaptive budget based on grid size.** Rejected as
  premature; the SHORT/LONG split is a simpler heuristic that
  captures most of the gain. Revisit if data shows jh-by-jh
  variance is the issue.
- **Eliminate the climb entirely via offline lower-bound
  estimation** (e.g. closed-form lower bound for Clos compose
  jh). Out of scope; would be a follow-on RFP if SHORT/LONG isn't
  enough.
- **Run multiple jhs in parallel** (start jh=lo, lo+1, lo+2 in
  separate processes; first feasible wins). Rejected: requires
  worker coordination + concurrent CP-SAT instances + much more
  complex teardown. Single-thread two-budget is the 80/20 win.

## Kill criteria

- **Total (4, 9) wall time regresses** vs the post-spatial-pruning
  baseline (~120s). The SHORT_TIMEOUT is too small or the LONG
  retry path is hit too often. Tune SHORT upward, or accept the RFP
  is wrong.

- **A previously-feasible shape now fails as INFEASIBLE-equivalent**
  (i.e. SHORT-UNKNOWN at the true-floor jh, then LONG-UNKNOWN, then
  advance to a *higher* jh than the original linear climb would have
  found OPTIMAL). The SHORT_TIMEOUT is too aggressive for that
  shape's solver behavior. Either bump SHORT, or fall back to the
  linear climb on UNKNOWN-after-LONG before advancing.

  **Test for this**: re-bake the (3..=9, 9) Clos shapes with the
  new search and check that the resulting `jh` matches what the
  linear climb would have found. Off-by-one on jh = bigger
  template = correctness risk.

- **The new code path is harder to reason about than the linear
  climb, AND the speedup is <30% on (4, 9).** Then the simplicity
  cost outweighs the perf win. Drop and accept the linear climb is
  good enough.

## Verification plan

Per [the layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Smoke baseline.** `(2, 2)` Clos via `FUCKTORIO_DEBUG_2_2=1
   FUCKTORIO_PURE_ROUTING_ENCODING=circuit cargo run --release -p
   balancer-gen` — must classify Balanced. Compare wall time.

2. **Headline.** `(4, 9)` Clos via `FUCKTORIO_DEBUG_4_9=1
   FUCKTORIO_PURE_ROUTING_ENCODING=circuit cargo run --release -p
   balancer-gen` — must classify Balanced. Wall target: ≤ 60s
   (vs current ~120s).

3. **Same-jh check.** The (3..=9, 9) Clos shapes (those baked or
   bakeable) must converge on the same `jh` as the linear climb
   would have produced. Compare against the values in the previous
   bake log entries (`/tmp/bake.log`-style logs, or run the linear
   climb branch in parallel to capture).

4. **Existing tests still green.** `cargo test --manifest-path
   crates/core/Cargo.toml`.

5. **Clippy clean.**

6. **Trace the SHORT/LONG decisions.** Run with
   `RUSTLOG=balancer_gen=debug` (or stderr inspection) and confirm
   each jh attempt logs its budget tag. SHORT-INFEASIBLE on (4, 9)
   jh=5..7 should be ≤ 5s wall each; SHORT-UNKNOWN on jh=8 should
   trigger LONG retry; LONG-INFEASIBLE on jh=8 should log clearly
   then advance to jh=9.

## Phasing

Single phase. ~30-60 LOC change in `compose_series`. No schema or
Python changes.

## Decision log

- *2026-05-02 — drafted. Wave 2 of the balancer-scale push. Will
  dispatch to ug-plumber (carries `main.rs` context from Item 4).*
