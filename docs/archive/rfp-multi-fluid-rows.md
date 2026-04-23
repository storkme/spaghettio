# RFP: Multi-fluid rows (stacked-T pipe isolation pattern)

## Summary

Add a row-template pattern for machines that consume or produce **two or more distinct fluids on the same face** (e.g. advanced-oil-processing with water + crude-oil inputs on its north face, multi-output refineries with petroleum-gas + light-oil + heavy-oil on their south face). Each fluid gets its own trunk row above (or below) the machine, with `UG-pipe-UG` isolation flanks so the trunk rows can stack vertically without adjacency-merging. Single-fluid-per-side rows do **not** need this — the simple continuous pipe row shipped in the Phase 1–4 work of the fluid-bus series handles them with no isolation ceremony.

## Motivation

The simple pattern (continuous east-west pipe row at the port-adjacent row) only works when **every port on that side carries the same fluid**. As soon as a side has two distinct fluids, a continuous row would merge them via pipe adjacency (Factorio rule F3), producing a "water and crude-oil mixed in one pipe network" bug that fluid isolation validation would flag.

Cases that trigger this today (or will soon):

| Recipe | Fluid shape | Same-side fluids |
|---|---|---|
| `advanced-oil-processing` | 2 fluid in, 3 fluid out | inputs: water + crude-oil; outputs: heavy-oil + light-oil + petroleum-gas |
| `heavy-oil-cracking` | 2 fluid in, 1 fluid out | inputs: water + heavy-oil |
| `light-oil-cracking` | 2 fluid in, 1 fluid out | inputs: water + light-oil |
| `solid-fuel-from-light-oil` (some setups) | mixed | varies |
| Foundry molten-metal recipes (Space Age) | multi in/out | varies |

None of these recipes currently produce zero-error layouts. `tier3_plastic_bar_from_crude`'s oil-refineries use `basic-oil-processing` (only 1 fluid per side), so the simple pattern already covers them; but any recipe that recurses through `advanced-oil-processing` will be stuck on this.

Reproducer (once implemented): `?item=heavy-oil-cracking&rate=5` — expect fluid-connectivity or fluid-isolation errors in the current main.

## Design

Adopt the stacked-T pattern originally sketched in the deleted `docs/fluid-row-pattern.md`. For a row that consumes `N` distinct fluids on its north face, the row grows by `N + 1` tiles above the solid belt (or above the machine if no solid inputs), with one vertical pipe column per fluid:

```
y = my - 4   ...── UG ── pipe ── UG ──...    ← fluid B trunk row
                         │                      (UG-pipe-UG: only pipe is at the T-drop)
y = my - 5   ...── UG ── pipe ── UG ──...    ← fluid A trunk row
                                  │
                                  │            (fluid A's vertical drop column)
y = my - 3           [UG pipe IN]             ← fluid B drops into belt tunnel
y = my - 2         [solid belt row]           ← ingredient belt (both fluid tunnels cross under)
y = my - 1           [UG pipe OUT]            ← adjacent to fluid B's port
y = my             ▓▓▓ machine top ▓▓▓
```

The load-bearing trick is `UG–pipe–UG` on each trunk row: the only surface-pipe tile per trunk row is at the T-drop for that fluid's column. The flanking tiles use pipe-to-ground, so adjacent trunk rows can stack without their pipes touching and merging. The flank UGs are oriented horizontally (east-west); the drop UGs between trunk rows are oriented vertically (north-south). Per [F5a](factorio-mechanics.md#fluids--pipes), a PTG's two perpendicular sides have no surface connection, so a horizontal flank and a vertical drop meeting edge-to-edge do not cross-contaminate.

### Code shape

- **`templates.rs`**: add `fluid_multi_input_row` and `fluid_multi_output_row`. Dispatched from `placer.rs::_build_one_row` when `fluid_inputs.len() >= 2` (distinct items) on the input side, or `fluid_outputs.len() >= 2` on the output side. For now, the two cases are independent — a row with multi-fluid inputs and single-fluid output uses multi-input pattern on top + simple pipe row on bottom.
- **`ghost_router.rs`**: the fluid trunk emission in step 3.6 needs to handle per-fluid tap-points. Currently `fluid_port_positions` is per-lane; with multiple fluid lanes for one row, each lane's branch targets its own trunk row. The branch tiles cross the other fluid's trunk row — a pipe on fluid A's branch row touching fluid B's trunk row's `UG-pipe-UG` flank is fine (pipe + UG-pipe don't merge since UG flanks don't have surface fluid).
- **`lane_planner.rs`**: each distinct fluid already gets its own lane, so no planner changes expected. Verify that `fluid_port_pipes` + `fluid_output_port_pipes` correctly report the per-fluid tap positions from the new templates.
- **Row pitch**: `placer.rs` currently sizes rows by a single "has fluid" flag. The multi-fluid template reports a taller `row_height` (by `N - 1` extra rows), which flows through to the placer naturally as long as the return type stays the same.

### Trade-offs rejected

- **Per-machine fluid tap-offs from the bus** (skip the per-machine T-drop, each machine gets its own trunk column): uses more horizontal space, and complicates `lane_planner`'s tap-off routing which currently emits one tap per row.
- **Mix solids onto one belt upstream** (feed two solids on two lanes of a shared belt, use filter-inserters per machine): requires a lane balancer + filter-inserter support, bigger scope.
- **Increase per-machine pitch to fit inserters and fluid chain on the same row**: only solves the 2-solid + 1-fluid case already handled by `fluid_dual_input_row`, doesn't help multi-fluid.

## Kill criteria

- If after implementing `fluid_multi_input_row` for advanced-oil-processing, the layout's fluid isolation validation still reports cross-fluid adjacency anywhere in the row region, the T + UG-flank geometry is wrong and we should reconsider the stacked-T approach (e.g. pivot to per-machine tap-offs).
- If making the multi-fluid template adds more than **300 LOC** to `templates.rs` (excluding tests), the abstraction is too case-specific and we should consolidate — most likely by making `fluid_only_row` recipe-shape-aware rather than adding a parallel function per shape.
- If the row-height increase from N-fluid stacking causes `tier3_plastic_bar_from_crude` (single-fluid, shouldn't be affected) to fail for unrelated geometry reasons, the placer's row-pitch logic has hidden assumptions that need untangling before this lands.

## Verification plan

Per [`CLAUDE.md`'s verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Full e2e suite stays green.** All 377 currently-passing tests must still pass.
2. **New active test**: un-ignore or add `tier3_heavy_oil_cracking` or `tier3_advanced_oil_processing` and assert zero validation errors. This is the primary regression signal for this RFP.
3. **Browser eyeball**: load `?item=heavy-oil-cracking&rate=5` and confirm the two input fluid trunks (water, heavy-oil) are visibly disjoint networks, each with its own column and T-drop into the machine.
4. **Snapshot decoder**: `FUCKTORIO_DUMP_SNAPSHOTS=1` on the new test, decode via `docs/layout-snapshot-debugger.md`. Grep the entity list for adjacent pipes with different `carries` values — should be zero hits.
5. **Fluid validation unit tests** in `crates/core/src/validate/fluids.rs` must stay green (they test validator correctness, not geometry).
6. **Clippy + WASM build** both pass.

## Phasing

1. `fluid_multi_input_row` for 2-fluid-input cases (heavy-oil-cracking, light-oil-cracking). Smallest increment that exercises the stacked-T geometry.
2. `fluid_multi_output_row` for 2+ fluid outputs (advanced-oil-processing, cryogenic-plant).
3. Dispatcher in `placer.rs` to route recipes to the right template based on `(fluid_inputs.len(), fluid_outputs.len())`.
4. Integration with `lane_planner.rs` fluid-lane column reservation if the multi-lane tap-off collides with adjacent-row fluid trunks.

## Decision log

- *2026-04-22 — filed after the single-fluid simplification shipped. The original `docs/fluid-row-pattern.md` described the stacked-T pattern as current behaviour; in practice the engine only ever implemented the single-fluid variant. This RFP captures what still needs to happen for multi-fluid recipes.*
