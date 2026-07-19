# Committed stress-scoreboard goldens

One file per stress fixture, written and checked by `check_stress_scoreboard`
in `tests/e2e.rs`. Each golden is the fixture's canonical scoreboard —
entity count, warning/error counts by category, crossing-zone and band
metrics — plus the structural layout hash (same hash as the `STRESSGOLD`
protocol lines).

These replace the re-derive-the-before-leg step of the STRESSGOLD gate
protocol: the "before" baseline is whatever is committed here, shared by
construction across implementer/reviewer agents, and every deliberate
layout movement is a visible re-bless diff in git history. See item 2 of
`docs/test-suite-followups.md`.

## Usage

```bash
# Gate check — diff current scoreboards against the committed goldens:
SPAGHETTIO_STRESS_GOLDEN=check cargo test --manifest-path crates/core/Cargo.toml \
    --test e2e -- --test-threads=1 --nocapture stress_

# Re-bless after an intentional layout-moving change, then commit the diff:
SPAGHETTIO_STRESS_GOLDEN=bless cargo test --manifest-path crates/core/Cargo.toml \
    --test e2e -- --test-threads=1 --nocapture stress_
```

`SPAGHETTIO_STRESS_GOLDEN=1` still prints the legacy `STRESSGOLD <test>
<hash>` lines without touching the goldens (capture-and-diff by hand).

## Why opt-in, not enforced by default or in CI

The scoreboards are only deterministic **relative to the SAT zone-cache
state of the host** (`~/.cache/spaghettio/sat-zones.bin`):

- A cache hit replays a stored solution; a miss runs a fresh SAT solve.
  A stored solution can be *valid but different* from what a fresh solve
  of current code produces (any satisfying assignment is acceptable), so
  warm and cold machines can disagree on layout bytes — measured on
  2026-07-19: 2 of 8 fixtures had different hashes warm-cache vs
  cache-disabled.
- Even the counts differ by cache path: cached Unsat/Timeout hits skip
  the solver without emitting the trace events a fresh unsat solve emits
  (`zones skipped` read 0 warm vs 164 cold on the same fixture), and
  entity/warning counts follow the solution bytes.
- `record_zone_with_solution` records unconditionally, so any
  `SPAGHETTIO_USE_ZONE_CACHE=0` diagnostic run refreshes the host cache
  in place — after which zone-dependent goldens legitimately drift and
  need a re-bless (the diff shows exactly which fixtures moved and how).
- Fresh solves are additionally **wall-clock-budget shaped** (25 ms
  `cost_descent_budget_ms` in the junction strategy): a faster machine or
  a faster build reaches a different point in the search before the
  budget cuts off, and lands on a different valid solution. Measured
  2026-07-19: switching `varisat` from opt-level 0 to 3 changed the
  fresh-solve layouts of 5 of 8 fixtures. Cache-state relativity and
  speed relativity are independent reasons the goldens are host-local.

So a golden blessed on one host is not portable to a cold machine, and CI
enforcement would flake. The always-on `StressBaseline` ceilings in
`e2e.rs` remain the portable, cache-robust regression gate; the goldens
are the exact-match gate for same-host before/after verification.
Promoting them to CI enforcement requires pinning the cache (e.g. running
the golden subset against a committed cache snapshot with ambient
read/write disabled) — a deliberate follow-up, not this mechanism.

## Coverage

Goldens exist for the non-ignored fixtures that call
`check_stress_scoreboard`. For the partitioned stress tests
(`check_partitioned_stress_scoreboard`) the golden covers the Pooled leg
only — the partitioned leg is gated by its ceilings, as before.
