# Test-suite time audit (2026-07-19)

Status: **in progress** — findings from a session-time audit. Each item below
has pick-up notes so a future session can start cold. Priority: **item 2 →
item 1 → item 3** (2 cheapens every gate run of the active power RFP; 1 is the
best value-per-line; 3 needs measurement before commitment). Item 2 landed
2026-07-19 (see its section); items 1, 3, 4 still open.

## Measured baseline

| What | Time | Source |
|---|---|---|
| CI, Rust-touching push (whole pipeline) | ~9–10 min | `gh run list` durations, 2026-07 |
| CI, docs-only push | 8 s | paths-filter skips all jobs |
| STRESSGOLD scoreboard subset (9 tests, forced serial) | 80–91 s | saved gate runs, 2026-07-19 session |
| `science_gauntlet` (all 6 packs) | 22 s warm | adversarial-review run, 2026-07-19 |
| Heaviest single tests, debug mode | 50–190 s each | comments in `.config/nextest.toml` |

The test *content* is nearly all useful — the stress corpus exercises the
product end-to-end and the golden/warning-population diffs are the
discriminating gates. The recoverable waste is in *how* it runs.

## Recoverable time

### 1. CI serialization tax (~5 min per Rust push) — fix already documented

`.config/nextest.toml` pins `test-threads = 1` in the ci profile. Its own
comment records the 2026-05-02 experiment (commit `8eb6ace`): parallel
execution dropped the Rust job **11m → 6m02s**, but
`tier4_advanced_circuit_7s_horizontal_stack_belt_pipe_crossing` blew its 30s
`#[ntest::timeout]` under 2-core CI contention (15× slowdown vs local), so it
was reverted — with the note *"to re-enable parallelism, bump the affected
timeout ceilings first."* The bump never happened.

**Pick-up**: raise the `#[ntest::timeout]` ceilings on the SAT-heavy tests
(they were sized for serial execution), flip `test-threads` back to default in
the ci profile only (keep local serial for STRESSGOLD runs, which need
ordered output). Gate: 3 consecutive green CI runs on Rust-touching pushes
with no timeout flakes.

### 2. Committed STRESSGOLD baseline (~90 s + a full-suite leg per work unit)

The current gate protocol re-derives the "before" scoreboard from HEAD for
every unit, even though that baseline is fully determined by the main commit —
and in the implementer/adversarial-reviewer team flow, both agents capture
their own copy.

**Pick-up**: make the scoreboard a committed golden artifact (e.g.
`crates/core/tests/goldens/stress_scoreboard.txt`); the test regenerates and
diffs against it, and a deliberate re-bless is "regenerate + commit", visible
in the diff like any golden. Kills the before-leg entirely, makes baselines
shared-by-construction across agents, and moves drift detection into git
history. This is the best ergonomics payoff for the RFP team flow — do it
first.

**LANDED 2026-07-19**: one golden per fixture under
`crates/core/tests/goldens/stress/` (canonical scoreboard + layout hash),
driven by `SPAGHETTIO_STRESS_GOLDEN=check|bless` in `check_stress_scoreboard`
(`=1` keeps the legacy hash-print protocol). Opt-in only — **not** enforced
by default or in CI, because measurement showed the scoreboards are only
deterministic relative to the host's SAT zone-cache state: 2 of 8 fixtures
had different layout hashes warm-cache vs cache-disabled (stale-but-valid
cached solutions), cached Unsat/Timeout hits skip the trace events fresh
solves emit (`zones skipped` 0 vs 164 on the same fixture), and cache-off
runs are 2.6× slower (174 s vs 67 s) *and* refresh the host cache in place
(recording is unconditional). The always-on `StressBaseline` ceilings remain
the portable gate. Follow-up if CI enforcement is ever wanted: pin the golden
subset to a committed cache snapshot with ambient read/write disabled. Full
rationale in `crates/core/tests/goldens/stress/README.md`.

### 3. Debug-mode tax on SAT/A*-heavy tests (~3×) — measure, then decide

Documented in `.config/nextest.toml`'s own comments: the tier5 PU
horizontal-stack test runs ~29 s in release vs >90 s in debug;
`partition_strategy_scoreboard` ~50 s local debug with 200–480 s observed in
CI.

**Pick-up**: measure `[profile.test]` `opt-level = 1` (or per-package
`opt-level` for the SAT/pathfinding-heavy dependencies) against the added
compile time on both cold and warm builds. Adopt only if the wall-clock win
survives the compile cost on a typical edit-test cycle. Decide on data, not
assumption.

### 4. (Low priority) Fresh-worktree compile cost for agents

Worktree-isolated agents pay a from-scratch build (minutes; dominated one
agent's 32-minute census run and is the true source of the folkloric
"science_gauntlet takes 25 minutes" — the SAT zone cache was never the
problem). sccache or a shared read-only cargo cache would recover most of it.
Occasional cost, so lowest priority.

## Not waste — leave alone

- **The SAT zone cache already works**: `zone_cache.rs` persists crossing-zone
  solutions to `~/.cache/spaghettio/sat-zones.bin` (WASM embeds a pre-baked
  blob). Warm-cache science_gauntlet is 22 s. "Cold" pain is compile, not SAT.
- **Implementer + reviewer each running the full suite** is deliberate
  independent verification — the redundancy is the point of the adversarial
  flow. Don't dedupe it.
- **paths-filter short-circuit** already reduces docs-only pushes to 8 s.
