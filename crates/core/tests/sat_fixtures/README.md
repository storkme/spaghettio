# SAT Fixture Library

Reproducible regression tests for the junction SAT solver.

Each `.json` file captures a specific `CrossingZone` problem extracted from the
running web app. `cargo test sat_fixtures` replays every fixture and asserts
the solver behaves as expected.

---

## How fixtures are consumed

`crates/core/tests/sat_fixtures.rs` reads every `*.json` in this directory,
deserialises each into a `Fixture` struct, constructs a `CrossingZone`, calls
`solve_crossing_zone_with_stats`, and compares the result against
`expected.mode`. Failures are accumulated and reported together at the end.

Run the suite:

```bash
cargo test --manifest-path crates/core/Cargo.toml --test sat_fixtures
```

---

## Fixture schema

```jsonc
{
  "version": 1,                        // schema version — bump on breaking changes
  "name": "descriptive_snake_case",    // used in failure messages
  "notes": "Free-form context …",      // human-readable; links to issues, bug desc
  "source_url": "http://localhost:5173/?item=…",
  "seed": [6, 75],                     // [x, y] of the junction seed tile
  "bbox": { "x": 4, "y": 73, "w": 12, "h": 5 },
  "forbidden": [[x, y], …],           // tiles that must be empty (tap-off passages)
  "belt_tier": "transport-belt",       // "transport-belt" | "fast-transport-belt" | "express-transport-belt"
  "max_reach": 4,                      // UG max reach (yellow=4, red=6, blue=8)
  "boundaries": [
    {
      "x": 7, "y": 73,
      "dir": "South",                  // "North" | "East" | "South" | "West"
      "item": "copper-cable",
      "in": true,                      // true = IN (flow enters zone), false = OUT
      "interior": false                // optional, defaults to false
    }
  ],
  "expected": {
    "mode": "solve",                   // see below
    "max_cost": 97,                    // optional hard anti-regression ceiling
    "optimal_cost": 85                 // optional aspiration — reported as gap
  },
  "context": {                         // optional — informational only in v1
    "ghost_paths": [
      { "item": "iron-ore", "spec_key": "tap:iron-ore:1:75", "tiles": [[x, y], …] }
    ]
  }
}
```

### `expected.mode` values

| Mode | Meaning |
|------|---------|
| `"solve"` | The solver must return `Some(entities)`. Use this for zones that should be solvable. |
| `"no_solve"` | The solver must return `None` (UNSAT). Use this to lock in a known-unsolvable configuration. |
| `"snapshot"` | Reserved for Phase F — exact entity list comparison. Not yet implemented. |

When in doubt, use `"solve"` for zones you expect to work and `"no_solve"` for
minimal configurations you know cannot be satisfied (e.g. a zone with
contradictory boundaries).

### `max_cost` vs `optimal_cost`

Each fixture carries two independent quality signals, both optional:

| Field | Semantics | Failure mode |
|-------|-----------|--------------|
| `max_cost` | **Hard anti-regression ceiling.** The solver's current best — don't let it get worse. | Test **fails** if `solution_cost > max_cost`. |
| `optimal_cost` | **Aspirational target.** The known-achievable cost (hand-painted reference or proved bound). | Never fails. Reported as `gap: N` in the harness output. |

The reporter prints both when present, e.g.:

```
PASS  ec_seed_18_96_iter2 — solved with 37 entities, cost 97 / optimal 85 / gap 12
```

So every run tells you how much headroom remains. Three hard rules:

1. **Correctness is binary**: `mode: solve` must return a solution; `mode: no_solve` must return UNSAT. No soft gate here.
2. **`max_cost` is a ratchet**: when you improve the solver, bump `max_cost` down to the new number in the same PR. Never move it *up* except with a clearly-justified reason (e.g. trading one bug fix for a benign cost increase elsewhere).
3. **`optimal_cost` only moves if the reference itself is wrong**: it represents a human-proved target. Don't edit it to match what the solver happens to produce; edit it only when you discover a strictly better hand-painted (or mathematically-derived) solution.

For a fixture where the solver already achieves the optimum, set
`max_cost == optimal_cost`. The gap line reads `gap 0` and a regression
in either direction is caught by `max_cost`.

### `context` field

The `context` object is **informational only** in v1. The harness loads and
ignores it. It is included so a human reading the fixture can reconstruct the
pre-SAT routing. Future Phase F will use `context.ghost_paths` to drive a
paint-based fixture builder view.

---

## Adding a new fixture

1. **In the running web app**, open a recipe URL, click a SAT-zone cell to open
   the junction debugger, then click the **⧉** (Copy as fixture JSON) button in
   the stepper row.

2. **Paste** the clipboard contents into a new file in this directory:
   ```bash
   # name the file after the zone and recipe, e.g.:
   crates/core/tests/sat_fixtures/ec_seed_6_75_iter3.json
   ```

3. **Edit** the file if needed:
   - Rename `"name"` to something descriptive.
   - Add useful `"notes"` (link to the GitHub issue, describe the failure).
   - Flip `"expected"."mode"` to `"no_solve"` if the zone is currently UNSAT
     and you want a regression guard against it accidentally becoming solvable
     without a real fix.

4. **Run the suite** to confirm it picks up and passes (or fails in the expected
   way):
   ```bash
   cargo test --manifest-path crates/core/Cargo.toml --test sat_fixtures -- --nocapture
   ```

5. Commit the fixture file alongside any solver fix.

---

## Belt tier → max_reach mapping

| Belt tier | `belt_tier` value | `max_reach` |
|-----------|-------------------|-------------|
| Yellow | `transport-belt` | 4 |
| Red | `fast-transport-belt` | 6 |
| Blue | `express-transport-belt` | 8 |

The ⧉ button derives these automatically from the SAT invocation data when
available.
