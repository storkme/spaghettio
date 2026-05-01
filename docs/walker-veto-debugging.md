# Debugging walker vetoes in the junction solver

Short guide for the failure shape where a zone's SAT strategy is rejected repeatedly by the walker veto, the region grows to the tile cap, and the zone bails without a solution. You'll see it in the web debugger as iterations tagged `Vetoed` with `broken_segment=...`.

## The anatomy of a zone attempt

Each `(iter, variant)` runs the strategies in order and stops on the first success. You'll see one or more of these outcomes per attempt:

| Outcome | Who said it | What it means |
|---|---|---|
| `Skipped` | `find_item_conflict` pre-check | Two boundaries at the same tile carry different items — the tile is provably one-belt-one-item. Grow. |
| `Unsatisfiable` | SAT solver (varisat) | CNF had no model. The bbox is genuinely too small / wrong shape for the boundaries. Grow. |
| `Vetoed` | Walker CEGAR check | SAT found a model but the shadow merge broke an existing routed path. Usually a real catch; sometimes a false positive — see below. |
| `DeferredExit` | Post-SAT sanity | A participating spec's frontier exits on a pending crossing in a *different* cluster. Defer so growth can reach past it. If you see this firing on the current cluster's own seed, that's a regression — see "Red flag" below. |
| `Solved` | Walker passed + variant accepted | Candidate emitted. Variant winner is chosen by cost. |

Know the source of the rejection before theorising — a `Vetoed` means SAT's CNF was *satisfiable*, so the fix isn't in the encoder.

## Fast first pass

1. **Find the trace JSON** for the failing zone: open the layout in the web debugger, click through to the iteration with the rejection, "Copy as JSON" the trace. The key fields you want are `seed`, `participating`, and the per-iter `attempts` list.
2. **Check the fixture directory** for an existing match: `crates/core/tests/sat_fixtures/*.json`. If a fixture has the same boundaries, run `cargo test --manifest-path crates/core/Cargo.toml --test sat_fixtures -- --nocapture` — if it passes, **SAT is not the culprit**. That was the case in both real bugs we've hit so far.
3. **If no fixture exists** and the failure is shape-y (two items in a small bbox, distinct corner turns, etc.), add one. Fixture schema is in `tests/sat_fixtures/README.md`. Running it immediately tells you whether SAT can or can't solve the zone in isolation.

## `DeferredExit` in detail

`DeferredExit` fires when SAT returned a valid model but one of the **participating specs' exit tile** (the tile where the spec leaves the region) sits on another unresolved crossing belonging to a *different* cluster. See `junction_solver.rs::should_defer_on_exit`.

Why defer: if we committed this zone's model now, its final surface belt would land on a tile that a different zone is also going to SAT-solve later. The downstream zone writes that tile from its own model and could overwrite whatever this zone stamped — torn edges, wrong item, or a belt facing the wrong way at the seam. So the solver bails (`break`s out of the whole strategy loop, since every strategy would inherit the same exit geometry) and lets the growth loop push the bbox outward.

The check must exclude two kinds of tiles:

1. **Already-solved crossings** — tiles belonging to clusters we've already processed have committed entities. They're not pending anymore. The solver passes `pending_crossings = crossing_set - corridor_handled` into each cluster's `solve_crossing` call.
2. **The current cluster's own seeds** — a multi-seed cluster (e.g. adjacent crossings that union-find merged) has several tiles in the `pending_crossings` set that *this very call* is in the middle of solving. A spec's frontier exit landing on one of our own seeds is not an external conflict.

Both exclusions have bitten us:

- **Case 1** caused spurious defers when zones exited onto previously-solved tiles. Fix was swapping `all_crossings` for the runtime `pending_crossings` set.
- **Case 2** was the worse bug: the defer check originally only excluded `seeds[0]`, so in any cluster with ≥2 seeds, a spec's frontier could exit on one of our other seeds and trigger defer against ourselves. At advanced-circuit @5/s from ores seed (25,196), the cluster had seeds `[(25,196), (25,197)]` and plastic-bar trunk's exit at (25,197) kept deferring against our own cluster indefinitely. Pre-fix: 23 DeferredExit events across the layout, 2 zones capped. Post-fix: 0 events, 1 zone capped (a genuinely harder case elsewhere).

### When a `DeferredExit` is actually legit

The remaining legitimate case: spec exits on a crossing tile that's truly in a *different, not-yet-solved* cluster. Growth is supposed to either absorb that downstream crossing as an interior tile, or advance the frontier past it.

It's theoretically possible for this to degenerate into a chain — a long run of consecutive crossings downstream, each uncovered one-by-one by growth, each triggering another defer until the region hits `MAX_REGION_TILES`. We haven't observed one in the wild since the fixes above; if you hit one, the options are:

- **Loosen `cluster_adjacent_crossings` further** in `ghost_router.rs`. Currently unions crossings within Manhattan distance 2 that share a spec. Expand to Manhattan 3 or implement path-aware clustering (BFS along each spec's path absorbing crossings within N tiles) if the chain spans more than 2 tile hops.
- **Topological cluster ordering**: order clusters by spec-flow dependency (downstream first) so by the time an upstream zone runs, the downstream tiles are already in `corridor_handled` and case 1's exclusion kicks in.
- **Skip-past in the defer check**: when an exit lands on a pending crossing, advance the frontier past the whole contiguous run of pending tiles along the spec's axis before deciding.

None of these are implemented beyond the current Manhattan-2 rule. If you're staring at a layout where `DeferredExit` is legitimately firing on an external cluster, look at this doc, decide which mitigation fits, then implement it — don't try to weaken the defer check further first.

## Cluster fragmentation

Distinct from the self-defer bug: even when `DeferredExit` is behaving, crossings that *should* be one junction can land in multiple clusters if `cluster_adjacent_crossings`' proximity rule is too tight. The region solver then runs once per tiny cluster, growth has to stitch them back together, duplicate boundaries pile up, and the walker vetoes everything. Eventually the tile cap bails.

Symptoms in the debugger:
- Many small (2- to 3-tile) clusters packed into a small area
- Each one starts a separate region solve, several cap at `MAX_REGION_TILES`
- `JunctionGrowthIteration` events show the bbox growing east / south to absorb neighbours that should've been in the cluster from the start

Seen at advanced-circuit @5/s from ores seed (21,161): five clusters within a 5×3 area all sharing specs (iron-plate trunks, electronic-circuit ret, copper-cable/plastic-bar trunks), none orthogonally adjacent so the pre-fix rule (|dx|+|dy|=1) left them as five separate problems. Relaxed to Manhattan ≤ 2 (`cluster_adjacent_crossings_tests::manhattan_2_straight_sharing_spec_merges`). If you hit a case where even that's not enough, see the escalation list above.

## Region-solver fixtures

For testing the *whole* region solver in isolation — grow-and-retry loop, strategy dispatch, walker veto, DeferredExit — use `crates/core/tests/region_fixtures/`. These capture the complete `solve_crossing` argument set (paths, obstacles, placed entities, pending crossings) and replay them through `fixture::replay_region_fixture`, mirroring the production code path.

This is the right level when:
- A SAT fixture passes but the full layout still misbehaves at that zone
- The bug involves walker-veto inputs (needs placed_entities context)
- The bug involves cluster-seed handling or DeferredExit semantics

SAT fixtures (`sat_fixtures/`) are still the right first move when the encoding itself is in question — they isolate the encoder from everything around it.

Capture workflow (full detail in `crates/core/tests/region_fixtures/README.md`):

```bash
# Capture one junction by seed. Runs any harness that builds the
# target layout — here the existing tier4 ignored test.
mkdir -p /tmp/rfx
FUCKTORIO_DUMP_REGION_FIXTURE=/tmp/rfx \
FUCKTORIO_DUMP_REGION_FIXTURE_SEED="10,161" \
    cargo test --manifest-path crates/core/Cargo.toml \
    --test e2e tier4_advanced_circuit_from_ore_am2 -- --ignored --nocapture

# Promote to a committed fixture
mv /tmp/rfx/seed_10_161.json \
   crates/core/tests/region_fixtures/my_junction.json
# ... edit name, notes, expected.mode, expected.max_cost ...

# Run the harness
cargo test --manifest-path crates/core/Cargo.toml --test region_fixtures -- --nocapture
```

The capture filters to a 20-tile radius around the cluster so fixtures stay at ~100–200 KB instead of dumping the entire layout.

### Red flag: defer firing on own seed

If you're seeing `DeferredExit` and the `break_tile` coordinates match one of the cluster's seeds, you've got a regression of case 2. Quick check: which seeds did the cluster start with? (Visible in the trace's `participating` block — the `start` field.) If the break tile equals any `start`, the defer-predicate exclusion has broken. See `bus::junction_solver::tests::should_not_defer_on_own_cluster_seed` for the regression test.

## Triaging a `Vetoed`: SAT bug or walker false positive?

Same-same from the trace alone — both show as `Vetoed`. The decision tree:

- **SAT fixture solves identical boundaries ⇒ walker false positive.** SAT's output is fine in isolation; the walker is reacting to shadow context that the fixture doesn't replicate (other routed paths, foreign belts, stale placements).
- **SAT fixture is UNSAT ⇒ SAT bug** (or the fixture is missing something the real zone provides). Look at the encoder: `encode_perimeter_boundary` / `encode_corner_boundary` / `encode_interior_boundary` in `crates/core/src/sat.rs`.
- **No fixture exists ⇒ add one first.** It's a 2-minute investment that collapses the search space in half.

## Using the walker-veto dump

`junction_solver.rs` has an env-var-gated dump at the veto site. When the walker rejects, it prints every `AffectedPath`, the shadow window around each break tile, and a tile-by-tile walk of each broken path with `item_ok`/`step_ok` flags. That last piece is usually decisive.

Enable it:

```bash
# All vetoes
FUCKTORIO_DUMP_WALKER_VETO=1 cargo run ...

# Filter to one seed
FUCKTORIO_DUMP_WALKER_VETO="seed:22,143" cargo run ...

# Filter to a specific break tile (useful when the same seed vetoes many times)
FUCKTORIO_DUMP_WALKER_VETO="seed:22,143;tile:11,196" cargo run ...
```

You need a harness that actually exercises the failing layout. If there isn't an existing test, a one-off `crates/core/examples/X.rs` is fine — delete it when you're done. The built-in e2e `tier4_advanced_circuit_from_ore_am2` and `veto_repro`-shaped scripts are the typical shapes.

Read the dump top-down:

1. **Proposed entities** — what SAT actually emitted. Compare to the boundaries. Does it satisfy them? Is the corner belt pointing the right way? Are UG-in/UG-out directions paired?
2. **Affected paths** — every routed path whose tiles touch `near_bbox(bbox)`. If the trimmed length looks way longer than the bbox, you may be pulling in foreign context (see "known false positive shapes").
3. **Breaks** — where the walker says each path failed, with the reason (`MissingEntity`, `ItemMismatch`, `Unreachable`).
4. **Shadow window around break** — what's actually on the tiles around the failure. The key check: does the path expect a surface belt here, is there one, does it carry the right item?
5. **Path tile walk** — `item_ok=N` / `step_ok=N` flags the specific tile where the walker's simulation diverges from the routed plan. This is almost always where the real fault lies.

## Known false positive shapes

Both fixed now, but these are the patterns to recognise next time:

### Foreign belt further along the path

A path routed end-to-end crosses *another* unresolved zone. When the walker BFS's to `path.last()`, it stalls at that zone's stale belt and blames the current SAT. Fix: `trim_path_near_bbox` restricts the walker to tiles within the current bbox's perimeter, not the whole path.

Symptoms:
- `break_tile` is well outside the current bbox
- The break always happens at the same tile regardless of what SAT proposed
- The break tile in the shadow window carries a different item with a different segment id (e.g., `trunk:plastic-bar`)
- Growing the bbox doesn't help; the zone caps out

### Tail runway lands on a foreign belt

Earlier versions of `trim_path_near_bbox` extended the trimmed slice by N tiles of runway on each side to give BFS context. If the tile just past the bbox was already claimed by a *different* unresolved crossing, the runway pulled that foreign belt into the walker's *target*, making BFS's end tile unreachable no matter what.

Symptoms:
- SAT's proposed entities inside the bbox look *correct* (proper UG tunnel, corner, or crossing)
- The break tile is 1–2 tiles past the bbox perimeter
- The shadow entity at the break tile carries a different item from the path

Fix: no runway. Trim to exactly the `[first_near ..= last_near]` span — `near_bbox` already includes a 1-tile perimeter, which is enough to anchor BFS.

### Heuristic UG pairing

`region_walker::build_belt_graph_for_item` pairs UG-in with the nearest colinear unused UG-out *of the same item* in the shadow. If SAT's placement puts the paired end outside the shadow window, under a foreign entity, or with a slightly different item, the walker treats the pair as unpaired and BFS can't tunnel. Haven't hit this one in anger yet, but keep an eye out — the telltale is a UG-in in the shadow window with no matching UG-out visible anywhere nearby.

## Adding a regression

When you fix a walker-veto false positive, add a test in `crates/core/src/bus/junction_solver.rs::tests`. The shape:

1. Build `existing` entities that reproduce the shadow state the walker sees.
2. Build `proposed` entities for SAT's (correct) output.
3. Call `walk_affected(...)` with both the full and trimmed paths.
4. Assert the full path walks broken (control — proves the regression is meaningful) and the trimmed path passes.

See `trim_avoids_false_positive_from_foreign_belt_down_the_path` and `trim_targets_last_bbox_tile_not_foreign_neighbour` for working templates.

## Related files

| File | Role |
|---|---|
| `crates/core/src/bus/junction_solver.rs` | Growth loop + walker veto site + `trim_path_near_bbox` + `dump_walker_veto` |
| `crates/core/src/bus/region_walker.rs` | `ShadowView`, `walk_affected`, BFS through the item-filtered belt graph |
| `crates/core/src/sat.rs` | SAT encoder and solver (unrelated if `Vetoed`, critical if `Unsatisfiable`) |
| `crates/core/tests/sat_fixtures/` | Standalone SAT-level regression fixtures + harness |
