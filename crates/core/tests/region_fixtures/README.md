# Region-Solver Fixture Library

Reproducible regression tests for the **whole** junction region solver — growth loop, strategy dispatch, walker veto, and DeferredExit, all together. The SAT-only fixtures under `sat_fixtures/` exercise the encoder in isolation; this harness exercises everything wrapped around it.

---

## How fixtures are consumed

`crates/core/tests/region_fixtures.rs` reads every `*.json` in this directory, deserialises each into a `RegionFixture` struct, calls `fixture::replay_region_fixture` (which rebuilds the `solve_crossing` argument slice and invokes the solver with the production strategy list), and compares the result against `expected.mode`. Failures are accumulated and reported together at the end.

Run the suite:

```bash
cargo test --manifest-path crates/core/Cargo.toml --test region_fixtures
```

---

## Fixture schema

```jsonc
{
  "version": 1,
  "name": "descriptive_snake_case",    // used in failure messages
  "notes": "What is this junction and why does it matter?",
  "source_url": "http://localhost:5173/?item=…",

  "seeds": [[21, 161], [22, 161], /* … */],         // cluster seed tiles
  "initial_specs": ["trunk:iron-plate:21", /* … */],

  // Per-spec routed path snapshots (A* output at capture time).
  "routed_paths": {
    "trunk:iron-plate:21": [[21, 159], [21, 160], /* … */],
    "ret:electronic-circuit:2:161": [/* … */]
  },

  "hard_obstacles": [[25, 142], /* … */],           // tiles holding
                                                     // machines/poles/row template
  "strict_obstacles": [],                            // usually empty; mirrors the
                                                     // ghost-router's obstacle split
  "unreleasable_obstacles": [[21, 144], /* … */],    // committed ghost belts SAT
                                                     // must preserve

  "spec_belt_tiers": { "trunk:iron-plate:21": "Yellow", /* … */ },
  "spec_items":      { "trunk:iron-plate:21": "iron-plate", /* … */ },
  "spec_exit_dirs":  { "trunk:iron-plate:21": "South", /* … */ },

  "placed_entities": [                               // current layout state —
    { "name": "transport-belt", "x": 21, "y": 160,   // needed for the walker's
      "direction": "South", "carries": "iron-plate", // shadow view
      /* … */ }
  ],

  "pending_crossings": [[25, 145], /* … */],        // crossings in other
                                                     // clusters not yet solved
                                                     // (feeds the DeferredExit check)

  "expected": {
    "mode": "solve",           // "solve" | "capped" | "unsatisfiable"
    "max_cost": 40,            // optional hard anti-regression ceiling
    "optimal_cost": 33         // optional aspiration — reported as gap
  }
}
```

### `expected.mode` values

| Mode | Meaning |
|---|---|
| `"solve"` | The region solver must produce a solution. The harness also checks `solution_cost <= max_cost` when set. |
| `"capped"` | `solve_crossing` returned `None` and the growth loop emitted a `JunctionGrowthCapped` event. Useful for pinning known unsolvable layouts so they don't silently regress into "works but wrong." |
| `"unsatisfiable"` | `solve_crossing` returned `None` without hitting the growth cap. Every strategy said UNSAT. |

### `max_cost` vs `optimal_cost`

Same semantics as `sat_fixtures/README.md`:

- `max_cost` is a **hard ratchet** — the test fails if the solver's entity cost exceeds it. Bump down when the solver improves.
- `optimal_cost` is an **aspirational target** — never fails the test, but reported as `gap: N` so headroom is visible on every run.

---

## Capturing a fixture from a real layout

The region solver's call site in `ghost_router.rs` has a debug-only dump path gated on an environment variable. Off by default.

### Capture one specific junction

```bash
mkdir -p /tmp/rfx
# Dump the cluster whose seed set includes (10, 161).
FUCKTORIO_DUMP_REGION_FIXTURE=/tmp/rfx \
FUCKTORIO_DUMP_REGION_FIXTURE_SEED="10,161" \
    cargo test --manifest-path crates/core/Cargo.toml \
    --test e2e tier4_advanced_circuit_from_ore_am2 -- --ignored --nocapture
```

The dump writes `/tmp/rfx/seed_{x}_{y}.json` where `(x,y)` is the first seed in the cluster (lowest `(y, x)` by sort order).

### Finding the seed you want

The seed filter matches against the cluster's `seeds` vec — the first tile (lowest by `(y, x)`) is what the filename uses, but the filter succeeds if ANY seed in the cluster matches. If your target tile isn't reported, it's probably merged into a larger cluster. Run without `_SEED` and look at stderr for the list of cluster seed sets each `solve_crossing` invocation saw.

### Capture every junction the pipeline encounters

Drop the `_SEED` variable. Produces one file per cluster — useful for bulk capture but noisy.

```bash
FUCKTORIO_DUMP_REGION_FIXTURE=/tmp/rfx \
    cargo test --manifest-path crates/core/Cargo.toml \
    --test e2e tier4_advanced_circuit_from_ore_am2 -- --ignored --nocapture
```

### Filter size

The capture filters `routed_paths`, obstacles, and `placed_entities` to a 20-tile radius around the cluster's tiles. Specs whose paths don't touch that radius are dropped entirely (along with their `spec_belt_tiers` / `spec_items` / `spec_exit_dirs` entries). Typical fixture size: 100–200 KB. If you need a larger context (e.g. a very long UG pair whose mate sits 25+ tiles away), bump `RADIUS` in `dump_region_fixture` temporarily when capturing and note it in the fixture's `notes`.

### Promote a capture to a committed fixture

1. Move the JSON into this directory with a descriptive filename:
   ```bash
   mv /tmp/rfx/seed_21_161.json \
     crates/core/tests/region_fixtures/advanced_circuit_seed_21_161.json
   ```

2. Edit the file:
   - Rename `"name"` to match the filename.
   - Replace the placeholder `"notes"` with a useful description (what the junction represents, which bug it guards against, link to the issue).
   - Set `"source_url"` to the recipe URL.
   - Set `"expected.mode"` to `"solve"`, `"capped"`, or `"unsatisfiable"` based on what the harness should enforce.
   - If `"solve"`, add `"expected.max_cost"` — run the harness once with `--nocapture` and read the reported cost, pad by ~20% as a starting ratchet.

3. Run the suite and confirm your fixture passes:
   ```bash
   cargo test --manifest-path crates/core/Cargo.toml --test region_fixtures -- --nocapture
   ```

4. Commit the file.

---

## Comparison to `sat_fixtures/`

| | `sat_fixtures/` | `region_fixtures/` (this dir) |
|---|---|---|
| Target | `solve_crossing_zone_with_stats` (SAT encoder) | `solve_crossing` (full region solver) |
| Tests | boundary handling, UG budget, encoding correctness | growth loop, walker veto, DeferredExit, strategy fallback |
| Inputs | `CrossingZone` (just boundaries + bbox) | `RegionFixture` (every solve_crossing argument) |
| Size on disk | small (1–3 KB typical) | larger (10–50 KB typical — routed_paths and placed_entities dominate) |
| When to use | isolating a SAT encoding question | isolating a region-solver behaviour question |

A SAT encoding problem is *also* reproducible from a region fixture — the first is a strict subset of the second. Use a SAT fixture when you can; fall back to a region fixture when the bug needs the surrounding context (walker veto inputs, deferral semantics, strategy ordering).
