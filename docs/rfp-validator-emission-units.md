# RFP: Validator emission units

## Summary

Validators today emit one `ValidationIssue` per affected tile. A single
root-cause bug — e.g. a missing N→M balancer — produces one issue per
overloaded belt tile in the downstream run, often dozens or hundreds.
This makes error counts a function of layout size, not bug count;
distorts the stress baselines added in #197; and gives misleading
deltas on changes that fix one bug while introducing another. This RFP
proposes extending `ValidationIssue` to carry a list of `affected`
tiles and refactoring each validator to emit **one issue per
root-cause cluster**, with the per-tile detail surfaced through that
list rather than through duplicate issues. Migrate validator-by-
validator behind golden snapshots; regenerate stress baselines once
the migration completes.

## Motivation

Real failing case (live, today):

```
http://localhost:5173/?item=processing-unit&rate=2&machine=assembling-machine-2
   &in=iron-plate,copper-plate,steel-plate,stone,coal,water,crude-oil,iron-ore,copper-ore
   &belt=fast-transport-belt&row_layout=horizontal-stack
```

The green-circuit output row is missing a fan-in balancer. The result
is one long output belt run carrying more than the per-lane capacity
of fast-transport-belt. `check_lane_throughput`
(`crates/core/src/validate/belt_flow.rs:1817-1834`) walks
`lane_rates`, a tile-keyed map of `[left, right]` rates, and emits one
`Severity::Error` for every tile-and-lane where `rate > cap + 0.01`:

```rust
for (&pos, &[left, right]) in &lane_rates {
    let belt_name = ...;
    let cap = lane_capacity(belt_name);
    for (lane_name, rate) in [("left", left), ("right", right)] {
        if rate > cap + 0.01 {
            issues.push(ValidationIssue::with_pos(
                Severity::Error, "lane-throughput",
                format!("Belt at ({},{}): {} lane {:.1}/s exceeds ..."),
                pos.0, pos.1,
            ));
        }
    }
}
```

For the case above this is **~87 errors from one missing balancer
template stamp**.

Three failure modes follow:

1. **Stress baselines encode noise.**
   `stress_electronic_circuit_40s_from_ore` carries `max_errors: 47`
   (#208). That number is not "47 distinct problems" — it is
   ~3-5 root causes radiating across belt tiles. Any structural change
   that fixes one cause and introduces a different small one can
   net-increase the count and trip the baseline. Conversely an
   apparent 47→4 win can be a single root-cause fix masquerading as a
   sweep.

2. **Asymmetric noise on changes.** A refactor that fixes belt routing
   (–80 warnings) but introduces one new fan-in dead-end (+87
   warnings) shows as a regression, contradicting the verification
   protocol's instruction to *"ask why"* when error counts move
   sharply (CLAUDE.md → Verification protocol §5).

3. **Per-validator unit drift.** Some validators already cluster
   correctly — `check_pole_network_connectivity`
   (`crates/core/src/validate/power.rs:47-98`) reports the 27-pole
   T2-from-ore disconnection as **one** warning with `count=27` in
   the message. Others (`check_lane_throughput`, `check_belt_loops`,
   `check_belt_item_isolation`, `check_pipe_isolation`,
   `check_fluid_network_connectivity`) emit per-tile. The codebase has
   no shared convention for "what is the natural unit of an issue",
   and validator authors guess.

Issue [#208](https://github.com/storkme/fucktorio/issues/208) tracks
the baseline-opacity symptom from the category angle (which categories
contribute to a baseline of N?). This RFP attacks the upstream cause:
**the count itself is the wrong quantity** because we conflate
issue count with affected-tile count.

## Design

### Extend `ValidationIssue`

In `crates/core/src/validate/mod.rs:60-102`, replace the single
`Option<x,y>` with a primary tile + a list of affected tiles:

```rust
pub struct ValidationIssue {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub primary: Option<(i32, i32)>,    // anchor tile for UI panning
    pub affected: Vec<(i32, i32)>,       // every tile this issue covers
}

impl ValidationIssue {
    // existing constructors keep their signatures; new ones add `affected`.
    pub fn with_cluster(
        severity: Severity, category: impl Into<String>,
        message: impl Into<String>, primary: (i32, i32),
        affected: Vec<(i32, i32)>,
    ) -> Self { ... }
}
```

`with_pos` continues to work — it produces an issue with `affected =
vec![(x,y)]`. No breaking change for the validators that already emit
per-cause.

### Refactor each validator to emit per cluster

Per-validator audit and target unit:

| Validator | File:line | Current unit | Target unit | Notes |
|---|---|---|---|---|
| `check_pole_network_connectivity` | `power.rs:47` | per island (1 issue, count in message) | per island, but populate `affected` | Already correct — just attach the disconnected-pole tile list. |
| `check_power_coverage` | `power.rs:119` | per uncovered machine | per uncovered machine | Already 1-per-cause; trivial. |
| `check_inserter_chains` | `inserters.rs` | per machine | per machine | Already correct. |
| `check_inserter_direction` | `inserters.rs` | per inserter | per inserter | Already correct. |
| **`check_lane_throughput`** | `belt_flow.rs:1797` | **per (tile, lane)** | **per overloaded flow segment** | Highest-impact target. Group consecutive overloaded tiles on the same belt path into one issue. |
| `check_belt_throughput` | `belt_flow.rs:1142` | per overlapping-route tile | per overlap site | Usually sparse but should cluster spatially. |
| `check_belt_dead_ends` | `belt_flow.rs:1506` | per dead-end | per terminus | Already 1-per-cause. |
| `check_belt_loops` | `belt_flow.rs:1627` | per loop tile | per cycle | One issue per cycle, all member tiles in `affected`. |
| `check_belt_item_isolation` | `belt_flow.rs:1683` | per island tile | per orphan island | Major source of noise on T5 baseline. |
| `check_belt_inserter_conflict` | `belt_flow.rs:1743` | per conflict tile | per conflict site | Investigate clustering value case-by-case. |
| `check_belt_junctions` | `belt_flow.rs:909` | per junction tile | per junction | Likely already 1-per-cause. |
| `check_underground_belt_pairs` | `underground.rs:47` | per orphan UG | per orphan UG | Already correct. |
| `check_underground_belt_sideloading` | `underground.rs:208` | per site | per site | Already correct. |
| `check_underground_belt_entry_sideload` | `underground.rs:252` | per site | per site | Already correct. |
| `check_pipe_isolation` | `fluids.rs:132` | per isolated pipe | per pipe network island | F2/F3 mixing: every pipe in a wrong network fires today. |
| `check_fluid_network_connectivity` | `fluids.rs:413` | per orphan tile | per orphan island | New code (2026-04-25). Fix at source before it hardens. |
| `check_output_belt_coverage` | `belt_flow.rs:1185` | per uncovered output | per uncovered output | Already correct. |

The work isn't uniform — about half the validators need no behaviour
change, just the new `affected` field populated. The other half
(starred) is where the real signal-quality win comes from.

### Update consumers

- **`scoreboard` and stress baselines**: counts are over `issues.len()`
  today. After migration the same expression yields cluster counts
  directly.
- **`crates/core/src/trace.rs` `ValidationIssueTrace`**: mirror the
  new field so snapshots carry the cluster info.
- **Web inspector** (`web/src/renderer/`): on hover/click of a tile,
  if it appears in any issue's `affected`, highlight the full cluster.
  This is a UI win the current per-tile model can't deliver.
- **Snapshot debugger**: cluster info travels into `.fls` files for
  free.

### What this is *not*

- Not severity weighting. A "fix one bug, introduce one bug" still
  shows as net-zero (which it is). Severity weighting would push
  `lane-throughput` (Error) above `power` (Warning); we already do
  this through severity buckets.
- Not category bucketing. That's #208 — orthogonal and complementary.
  After this RFP lands, #208's "47 errors = 8 belt-flow + 6
  lane-throughput + ..." breakdown becomes accurate counts of
  *distinct issues* per category, which is what makes that
  workflow useful.
- Not a global cluster-after-the-fact post-pass. We rejected that in
  the original discussion: it papers over the per-validator unit
  question and produces the same cluster signature for "10 belts dead
  for 10 different reasons" as for "1 missing balancer broke 10
  belts". The fix has to live inside each validator.

## Kill criteria

- **If after migrating `check_lane_throughput`, `check_belt_loops`,
  `check_belt_item_isolation`, `check_pipe_isolation`, and
  `check_fluid_network_connectivity` the
  `stress_electronic_circuit_40s_from_ore` baseline does not drop from
  47 to ≤10**, the per-tile inflation hypothesis is wrong — there are
  genuinely many independent root causes, and the cluster fix doesn't
  buy us interpretability. Stop and revisit.

- **If the per-validator clustering logic exceeds ~80 LOC for any
  single validator** (excluding tests), the natural unit is wrong and
  we're forcing a shape — pick a different cluster definition or back
  the validator out of scope.

- **If `cargo test --release` runtime regresses by >25% on the
  non-stress e2e suite**, the cluster-building cost is too high; spec
  out a lazy-evaluation path or revert.

- **If the new `affected` field exceeds 1MB total across all issues
  for any single test layout**, we're carrying too much detail into
  the snapshot; switch to cluster-id + on-demand resolution.

## Verification plan

Per CLAUDE.md "Verification protocol for layout engine changes":

1. **Full e2e suite green** — `cargo test --manifest-path
   crates/core/Cargo.toml`. All 19 currently-passing e2e tests stay
   green; ignored tests stay ignored.

2. **Stress baselines regenerated and tightened.** After each
   validator's migration phase:
   - Re-run all 6 active stress tests with `--nocapture` and compare
     issue counts before/after.
   - Update `StressBaseline` values in `crates/core/tests/e2e.rs`
     **downward** to match the new (smaller) cluster counts.
   - If a baseline can't be tightened (cluster count = old per-tile
     count), the validator wasn't actually emitting per-tile —
     remove it from the migration plan.

3. **Snapshot debugger sanity check.** For
   `processing_unit_2s_am2_fast_belts_validation_baseline`, dump the
   snapshot, decode the `validation.issues` list, confirm:
   - The current `belt-item-isolation × 9` baseline collapses to 1-3
     cluster issues.
   - Each cluster issue's `affected` matches the tiles previously
     flagged.

4. **Browser eyeball.** For the URL in the Motivation section, the
   sidebar validation panel should show **1** lane-throughput issue
   (not 87), with hover-cluster-highlight populated from `affected`.
   Pre-RFP screenshot saved as kill-criterion ground truth.

5. **Trace events unchanged.** This RFP doesn't add new
   `TraceEvent` variants — `RouteFailure`, `BridgeDropped`,
   `CrossingZoneSkipped`, `BalancerStamped` still fire identically.
   Verify in snapshot diff that no trace events shifted.

## Phasing

1. **Phase 1: type plumbing.** Extend `ValidationIssue` with
   `affected: Vec<(i32,i32)>`. Update all `with_pos` call sites to
   default to `affected = vec![(x,y)]`. Update `ValidationIssueTrace`
   in `trace.rs` and the wasm bindings. Zero behaviour change. PR
   should be all-mechanical; review purely for coverage.

2. **Phase 2: lane-throughput POC.** Migrate
   `check_lane_throughput` to emit per-segment. This is the highest-
   impact validator (your processing-unit case, advanced-circuit
   partitioned case, the bulk of stress baselines). Land with updated
   stress baselines that reflect cluster counts. Verify against kill
   criteria.

3. **Phase 3: remaining inflators.** Migrate `check_belt_loops`,
   `check_belt_item_isolation`, `check_pipe_isolation`,
   `check_fluid_network_connectivity` in a single PR. Tighten
   baselines.

4. **Phase 4: web inspector cluster highlight.** Use the new
   `affected` field to drive hover-cluster overlay in the sidebar
   validation panel. This is the UX payoff.

5. **Phase 5: scoreboard re-grounding.** Once cluster counts are
   stable, revisit the stress-corpus targets — many of the
   `max_errors > 0` baselines should now be tractable to drive to
   zero. Coordinate with #208 to attribute remaining clusters to
   tracking issues.

Phases 1-3 are landable independently and incrementally. Phase 4 is
pure web work. Phase 5 is ongoing.

## Decision log

- *2026-04-26 — drafted following the validation-groundedness
  discussion in the 2026-04-25 review session. Pending acceptance.*
