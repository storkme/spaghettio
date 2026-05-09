# RFP: Pipe-vs-belt junctions

## Summary

Teach the junction-solver pipeline to treat fluid-trunk tiles as
participating crossings. Today, belts routed across a fluid-trunk column
are dropped at the pipe tile (correct — no overlap allowed) but no
underground bypass is ever stamped, leaving the belt severed. The fix
has two parts: (1) emit synthetic spec paths for fluid trunks so
`classify_crossing` sees them as the second spec at crossing tiles, and
(2) teach `solve_perpendicular_template` about pipe-kind specs so it
forces the belt underground and never stamps a belt on top of a pipe.

## Motivation

Live repro: `processing-unit @ 1/s`, view centred around column x=20,
rows y=103-105. Three belts flow east/west across the sulfuric-acid
fluid trunk at x=20:

| tile      | entity                                  | carries            |
|-----------|-----------------------------------------|--------------------|
| (18,103)  | fast-transport-belt West                | electronic-circuit |
| (19,103)  | fast-transport-belt West                | electronic-circuit |
| (20,103)  | **pipe**                                | sulfuric-acid      |
| (21,103)  | fast-transport-belt West                | electronic-circuit |
| (22,103)  | transport-belt West                     | electronic-circuit |
| (18,104)  | fast-transport-belt East                | iron-plate         |
| (19,104)  | fast-transport-belt East                | iron-plate         |
| (20,104)  | **pipe-to-ground South, io=input**      | sulfuric-acid      |
| (21,104)  | fast-transport-belt East                | iron-plate         |
| (22,104)  | fast-transport-belt East                | iron-plate         |

The electronic-circuit belt at y=103 and the iron-plate belt at y=104
are severed at x=20. The belt-flow validator sees disconnected segments
but no UG bypass was ever stamped. This is the shape we intended to
prevent when we designed the "belts cross pipes via junctions" story.

What is already working:

- Fluid trunk pipes get `segment_id = "trunk:{item}"` at
  `ghost_router.rs:474-481`.
- The catch-up pass at `ghost_router.rs:669-680` registers those pipe
  tiles in `trunk_tile_items` → `ghost_item_at`.
- The ghost-route survivor filter (`ghost_router.rs:1308-1311`) correctly
  drops the belt entity at the pipe tile.
- The crossing tile is added to `all_ghost_crossings` via the
  `ghost_item_at != spec.item` check at `ghost_router.rs:1344-1352`.

What is not working:

- `trunk_synth_paths` is only populated for **solid** lanes
  (`ghost_router.rs:277-278`: `if lane.is_fluid { continue; }`). Fluid
  lanes never get a path entry injected into `routed_paths`.
- Consequently `classify_crossing` at a pipe-vs-belt tile finds only
  one spec (the belt). `crossing_specs.len() != 2` returns `None`
  (`ghost_router.rs:2467-2469`).
- `emit_unresolved_junctions` falls into `unwrap_or_default()` at
  `ghost_router.rs:2528` → the `Junction` carries **empty** specs.
- With no specs, `PerpendicularTemplateStrategy` is a no-op, and every
  SAT strategy has nothing to solve. The crossing stays unresolved
  with zero bridging.
- The belt stays severed and nothing emits a trace event explaining
  why.

A secondary issue lurks behind this: even if we synthesize a fluid
trunk path, `solve_perpendicular_template` / `try_bridge` would
mishandle it. `try_bridge` stamps `surface_belt` (a transport-belt
family name, via `belt_name_for_tier`) at the crossing tile on top of
the existing pipe, and its "bridge vertical first" preference would
try to UG the trunk (nonsensical for a pipe). The first viable surface
stamp would panic in `Occupancy::place` against the pipe's `Permanent`
claim.

## Design

### Part 1 — synthetic fluid trunk paths

Extend the trunk-synth loop (`ghost_router.rs:277-354`) to emit fluid
lanes. Key shape: `"trunk:fluid:{item}:{x}"` (distinct from solid
trunks so downstream predicates can branch on it without parsing the
item name). Tiles come from the same anchors the fluid stamper already
computes in step 3.6: `start_y`, every tap_y, `end_y`, and every
UG-in/out emitted by the back-to-back chain between anchors. The
cleanest implementation pulls the anchor list out of step 3.6 into a
helper shared with the synth emitter, so both loops agree on which
tiles belong to the fluid trunk column.

Extend the `spec_items` / `spec_belt_tiers` maps at
`ghost_router.rs:1506-1517` to include fluid keys. Tier is irrelevant
for pipes, but we need an entry so `classify_crossing` doesn't
`continue` at line 2443. Add a parallel `spec_kinds: FxHashMap<String,
SpecKind>` with `enum SpecKind { Belt, Pipe }` seeded from `lane.is_fluid`.

`CrossingInfo` gains a per-spec kind field:

```rust
struct CrossingInfo {
    tile: (i32, i32),
    spec_a: (String, EntityDirection, SpecKind),
    spec_b: (String, EntityDirection, SpecKind),
    belt_a: &'static str,
    belt_b: &'static str,
}
```

`classify_crossing` populates `SpecKind` from the new map. `belt_*`
stays `&'static str` but is unused for `SpecKind::Pipe` specs.

### Part 2 — pipe-aware bridging

`solve_perpendicular_template` gets a short circuit at the top:

```rust
match (info.spec_a.2, info.spec_b.2) {
    (SpecKind::Pipe, SpecKind::Pipe) => return None,      // pipe × pipe — out of scope
    (SpecKind::Pipe, SpecKind::Belt) => return bridge_belt_over_pipe(info, ..., pipe=a, belt=b),
    (SpecKind::Belt, SpecKind::Pipe) => return bridge_belt_over_pipe(info, ..., pipe=b, belt=a),
    (SpecKind::Belt, SpecKind::Belt) => { /* existing logic */ }
}
```

`bridge_belt_over_pipe` is a specialisation of `try_bridge` that:

1. **Never** stamps anything at `(cx, cy)` — the pipe entity is already
   there and stays. No `surface_belt` entity.
2. Always UG-bridges the belt spec (direction + tier from the belt
   spec's entry in `spec_belt_tiers`).
3. Reuses `try_bridge`'s existing obstacle / sideload / turn checks for
   the UG-in/out tiles (belt-side checks only — the pipe tile doesn't
   need them).
4. Emits a single `junction:{belt_item}:{cx},{cy}` segment id, with the
   UG pair as its entities. `ClusterZone` covers the three-tile footprint
   just like the belt-vs-belt path.

Same fall-back structure: try bridging the belt at the crossing tile;
if the UG-in or UG-out is blocked, return `None` and let the SAT
strategies try. SAT strategies inherit the `SpecKind` through the same
maps, so they can emit their own pipe-aware constraint (the pipe tile
is a fixed surface spec; the belt must exit via a corridor).

### Non-goals

- **Pipe × pipe crossings.** Two fluid trunks crossing each other is a
  different problem (requires a UG pipe pair) and doesn't reproduce on
  any current fixture. We return `None` for that shape and let it fall
  through to an Unresolved region that logs in the scoreboard.
- **Same-direction pipe × belt.** A belt running parallel to a pipe
  column isn't a crossing — they live on different tiles. Not in scope.
- **Rewriting the fluid stamper.** Step 3.6's anchor logic stays; we
  only extract the tile list for spec synthesis.

## Kill criteria

Explicit and observable. After Part 1 + Part 2 land:

1. **If the `processing-unit @ 1/s` layout still shows severed belts at
   pipe columns** (manual browser check: navigate to the URL, spot-check
   three fluid-trunk × belt intersections, verify each has a visible UG
   pair on the belt side), the approach is wrong — we're missing a
   failure mode in classify_crossing or the growth loop and should stop
   before investing more.

2. **If `stress_processing_unit_20s_from_plates` (currently `#[ignore]`)
   doesn't drop its belt-flow-disconnect warning count to zero after
   both phases**, OR if any of the nine always-on e2e tests regress on
   warning count, the kill criterion trips. A drop from N to 0 that's
   not matched by a corresponding rise in `zones_solved` / stamped UG
   pairs means we silenced a validator rather than fixing the layout —
   see CLAUDE.md verification protocol #5.

3. **If the SAT strategy list grows a `SpecKind`-aware branch longer
   than ~150 LOC**, the `SpecKind` abstraction is too expensive for
   what's essentially "don't stamp on this tile." Reconsider — the
   right fix might be to make the pipe tile a hard obstacle in SAT's
   obstacle set and drop the spec-kind plumbing entirely.

## Verification plan

Follows CLAUDE.md's
[layout-engine verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes).

1. **Unit**: `cargo test --manifest-path crates/core/Cargo.toml` must
   stay green across all nine non-ignored e2e tests.
2. **Stress**: run `stress_processing_unit_20s_from_plates` with
   `--nocapture`. Before/after scoreboard: `zones_solved` up,
   `zones_skipped` down, belt-flow disconnects at 0.
3. **Trace events**: new `JunctionTemplateAccepted` entries with the
   `junction:{belt_item}:...` segment id should appear at every
   pipe-trunk × belt intersection. Zero `JunctionTemplateRejected` with
   reason `surface_conflict` or similar at those tiles.
4. **Snapshot**: `SPAGHETTIO_DUMP_SNAPSHOTS=1 cargo test ...
   stress_processing_unit_20s_from_plates --nocapture`, decode, and
   confirm each pipe column has its adjacent belt rows using UG-in /
   UG-out pairs directly next to the pipe tile (not five tiles away,
   not replaced by a detour).
5. **Browser**: `/?item=processing-unit&rate=1` in the dev server.
   Spot-check three pipe columns. Expected: the pipe stays as-is; the
   intersecting belts use UGs to bypass. No belts rendered on top of
   pipes. No missing belt tiles in the middle of a row.
6. **WASM build**: `wasm-pack build` must succeed after the change.

## Phasing

- **Phase 1**: synthetic fluid trunk paths + `spec_kinds` plumbing.
  End-state: `classify_crossing` returns a two-spec `CrossingInfo` at
  pipe × belt tiles. Nothing bridges yet, but every affected crossing
  emits a `JunctionTemplateRejected` event (kind mismatch) instead of
  silently vanishing. Zero belt layout change. Landable on its own as
  diagnostic progress.

- **Phase 2**: `bridge_belt_over_pipe` + `SpecKind` short-circuit in
  `solve_perpendicular_template`. End-state: pipe × belt crossings
  stamp UG pairs; the processing-unit layout survives browser
  inspection.

- **Phase 3 (deferred)**: SAT pipe-awareness. Only land if Phase 2
  leaves a fixture in the corpus where the perpendicular template is
  genuinely insufficient (e.g. two belts crossing at the same pipe
  tile, or a pipe adjacent to a balancer so the UG-in tile is blocked).
  Not speculating — write this phase once a failing case exists.

## Decision log

- *2026-04-24 — drafted. Awaiting acceptance before implementation.*
- *2026-04-24 — Phase 1 implementation pivoted twice during planning,
  landing on the smallest viable shape: relax the
  `classify_crossing` pre-cluster gate at `ghost_router.rs:1691-1699`
  so single-spec clusters whose seed sits on a forbidden tile reach
  `solve_crossing`. The bet was that the existing growth loop + SAT
  would handle pipe×belt natively (pipes are auto-classified as
  forbidden via `tile_is_forbidden_kind`).
- *2026-04-24 — Phase 1 partial result: change is correct and
  contained (0 of 9 always-passing e2e tests affected, no regressions),
  but the bet that SAT would solve pipe×belt didn't pan out. The SAT
  encoder doesn't constrain single-spec **continuity** — for a
  cluster with one spec whose entry/exit boundaries are already
  satisfied by pre-existing belts, SAT proposes a degenerate
  "place a single belt at the existing path tile" model that
  satisfies all constraints without bridging the forbidden tile.
  The walker correctly vetoes; growth doesn't help because SAT
  proposes the same trivial model at every iteration. Confirmed
  via `SPAGHETTIO_DUMP_WALKER_VETO=seed:12,44` on the new
  `pipe_belt_processing_unit_1s_routes` test.
- *2026-04-24 — kept the gate fix and the
  `debug_assert!`-to-`continue` change at the cluster-handled-tile
  guard (the original assert held only because the gate filtered
  out the cases where overlapping cluster footprints could occur —
  a real bug it was masking). Both are correct on their own and
  serve as scaffolding for Phase 2.
- *2026-04-24 — Phase 2 owners: decision pending. Three viable
  shapes: (1) extend SAT encoder with single-spec continuity
  constraints (likely benefits the in-flight SAT solver work), (2)
  add a dedicated single-spec bridge strategy that runs before SAT
  and stamps UG-in/UG-out around forbidden tiles for the
  straight-through case, (3) layout-time fluid-trunk-placement
  avoidance so pipes don't land on belt feeder corner tiles.
  Tracked via the `#[ignore]`'d
  `pipe_belt_processing_unit_1s_routes` regression test.*
