# Lane column packing: can we share bus columns between y-disjoint lanes?

**Status**: Investigation closed 2026-04-23. Answer: **Not without rebuilding ghost-router crossing resolution.**
**Related**: PR [#160](https://github.com/storkme/spaghettio/pull/160) (closed unmerged).

## What we were trying to do

Every `BusLane` currently gets its own column: `lane.x = (i + 1)` for the i-th lane after `optimize_lane_order`. A typical tier2 layout is ~11 columns wide even though most lanes only occupy a short y-slice of the bus. PR #160 proposed greedy bin-packing: if lane A's y-range is `[0, 30]` and lane B's is `[40, 80]`, share a column — the trunks never overlap physically.

Potential win on the tier2-from-ore snapshot: bus width 12 → 8 columns (~33%).

## The original PR

PR #160 carried three commits, but only one was new work:

| SHA | Commit | Fate |
|---|---|---|
| `3ad0026` | `fix(web): make streaming render visible during layout` | Already on main via squash-merged #159 |
| `752f4f7` | `feat(web): grid follows layout bounding box` | Already on main via #159 |
| `e1b6da9` | `feat(bus): pack lane columns by sharing non-overlapping y-ranges` | The genuinely new work |

Merging as-is was not an option: the two web commits conflict with further rewrites of `web/src/main.ts` (streaming reconciliation in `d845e56`, timeline scrubber in #166). The audit focused on cherry-picking just `e1b6da9`.

## What killed it

Cherry-picking the packer onto current `main` fails `tier2_electronic_circuit_from_ore` with three validation errors:

```
[belt-loop] Belt loop detected: 6 tiles form a cycle near (1,20)
[belt-item-isolation] Belt at (1,47) carries iron-ore but feeds into (2,47) which carries copper-cable
[belt-item-isolation] Belt at (1,38) carries iron-ore but feeds into (1,39) which carries copper-cable
```

Other 7 non-ignored e2e tests pass. Baseline is green on clean `main`.

### Two bugs in the packer itself

Initial audit found two concrete bugs in `e1b6da9`'s packing logic:

1. **`active_range` under-reports `end_y`.** `ghost_router.rs:317-325` computes the real trunk `end_y` as `max(tap_off_ys ∪ producer_ys).max(balancer_y + 1)`. The helper in `lane_order.rs:active_range` uses only `max(tap_off_ys)`. A lane with a `balancer_y` past its last tap has its footprint truncated, and an unrelated lane gets packed on top of the balancer row.
2. **`family_balancer_range` is set AFTER the packer runs.** `lane_planner.rs` assigns `lane.family_balancer_range` at line ~344, but `pack_lane_columns` is called at line ~304. The packer's `eff()` helper that folds the family range in is dead code — always reads `None`. N→M balancer blocks can be 5-10 rows tall, so the packer systematically under-reports family footprints.

Both fixes were landed locally:

- `active_range` rewritten in `lane_order.rs` to mirror `ghost_router.rs:317-325` exactly, folding in `balancer_y + 1` and `family_balancer_range`.
- The block that assigns `fam.balancer_y_end` + `lane.family_balancer_range` was split out of the post-pack loop and moved to before `pack_lane_columns`. (The `fam.lane_xs` assignment stayed after, since it reads packed `lane.x` values.)

### The failure mode after the fixes

Same three errors, same coordinates. Snapshot inspection (`SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ... tier2_electronic_circuit_from_ore`) explains why.

Under the packer, the lane columns come out as:

| Item | Cols (packed) | Cols (sequential baseline) |
|---|---|---|
| iron-ore | 1 | 1 |
| copper-ore | 2 | 2 |
| copper-plate (1→3 family) | 2, 3, 4 | 6, 7, 8 |
| copper-cable (1→3 family) | 2, 3, 4 | 3, 4, 5 |
| iron-plate (1→3 family) | 5, 6, 7 | 9, 10, 11 |

iron-ore is **alone** at col 1 — no on-column conflict, so neither of the two fixes applies. The violation is at col 1 y=38-44 where copper-cable entities appear on iron-ore's column. They are **not** trunk tiles — they are the upstream feeder that delivers copper-cable into its 1→3 balancer's input at (4, 38).

Under the sequential baseline, the copper-cable family sits at cols 3-5. Its feeder either doesn't need to cross col 1 at all, or crosses at y-coordinates where the junction solver's pre-existing crossing-zone logic sets up UG bridges (y=19-32 on iron-ore both in baseline and after packing — those do resolve cleanly).

Under packing, the copper-cable family sits at cols 2-4. The feeder now crosses col 1 at y=39-44. The junction solver **does not detect this as a crossing**: iron-ore gets stamped as surface-facing belts at y=33-38 and y=45-47, with no UG bridge spanning the copper-cable feeder. (1,38) ends up pointing south into (1,39) copper-cable — item-isolation violation.

### Why the router can't handle it today

`lane_order::score_lane_ordering` already has template-aware scoring — `lane_order.rs:83` computes `landing_x = (ox as i32) + dx + 1` for each balancer template input tile and penalises other lanes that would land on it. But `ox = pos + 1` assumes **sequential** column assignment. Once the packer can share columns, `ox` and `pos` decouple: the scorer still predicts landings against sequential positions while the packer assigns real columns elsewhere.

Worse, even if the scorer were updated to know packed columns, the junction solver's crossing-zone detection would still need to see the feeder/trunk cross. The current detection fires on a fixed set of geometric patterns aligned with the sequential lane spacing; a feeder routed through a previously-empty column that now holds a trunk is not one of them.

## Conclusion

Lane column packing is structurally tempting (~30% bus width savings is real) but it is **not a localized change**. It reaches into at least three places that today all assume sequential column layout:

1. `lane_order::score_lane_ordering` — scoring for balancer template input landings.
2. `ghost_router` step 2 / step 3.5 — trunk stamping assumes no one else claims the column.
3. `junction_solver` + crossing-zone detection — fires on fixed geometric patterns relative to the sequential layout.

A correct packer implementation would need to either (a) restrict packing to cases where no feeder/tap routing crosses a shared column (likely eliminates most of the savings), or (b) teach the router to synthesise crossing zones for feeder paths through any occupied column, not just sequentially-assigned ones. (b) is a larger piece of work than the packer itself.

**Verdict**: close the PR. The bus-width-compaction opportunity is real but the next step is not this packer — it's rethinking how the router discovers crossings. That's a separate investigation.

## What lands from this investigation

Nothing lands in `main`. The two packer bugs (under-reported `active_range`, mis-ordered `family_balancer_range` assignment) would only matter in a world where the packer exists, so they don't merit a standalone fix.

## What doesn't land

- `pack_lane_columns` and the `bus_width_for_lanes = max(lane.x) + 2` derivation.
- Promotion + rewrite of `active_range` in `lane_order.rs`.
- Reordering of `family_balancer_range` assignment in `lane_planner.rs`.
- A `layout.width` / `layout.height` scoreboard line in `report_stress_scoreboard` (drafted in the plan, never written — contingent on having a packer to measure).

## If you pick this up again

Starting points worth knowing:

- `ghost_router.rs:317-325` is the source-of-truth y-footprint formula any packer must mirror (mirrored in the attempted `active_range` rewrite).
- `lane_order.rs:83` is the template-aware scoring that needs generalising for packed columns. The `ox = pos + 1` assumption is the first thing to break.
- The crossing-zone detection is in `ghost_router` steps 4-6 plus `junction_solver`. A packed-column feeder vs. trunk crossing needs to land in that detector before anything else will work.
- Snapshot inspection recipe: `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test --manifest-path crates/core/Cargo.toml --test e2e --release tier2_electronic_circuit_from_ore`, then decode `target/tmp/snapshot-*.fls` per `docs/layout-snapshot-debugger.md`. Filter `layout.entities` by `x == 1 and 15 <= y <= 55` to see the iron-ore vs copper-cable interleaving directly.

The original PR branch `feat/streaming-visibility-and-grid` and the exploration branch `pack-lane-columns-fix` were both deleted. The packer code is reachable via PR #160's final commit `e1b6da9`.
