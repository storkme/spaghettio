# RFP: Horizontal trunks for input-bottlenecked rows

## Summary

Today, when a recipe's per-machine input demand exceeds what a single
input belt can carry (e.g. electronic-circuit needs 7.5/s copper-cable
per machine, an express belt at 45/s feeds at most 6 machines), the
placer **vertically splits** the recipe into multiple short rows. Each
row gets its own input belts, its own output belt, and downstream the
fragmented outputs are reassembled with N→M balancers and bus trunks.

This RFP proposes **horizontal partitioning** instead: keep the recipe
on **one long row** but stack multiple input belts at the top of the
row, each routed via underground belts into its dedicated machine
block. Output is one continuous east-flowing belt that already runs at
full belt capacity (lane-split bridge in the middle, as today). No
return-belt routing back to the bus, no N→M balancer family, and no
trunk-vs-consumer-count mismatch.

The change is local to the row template + the lane planner's
"row-splitting" decision. Bus routing, return-belt logic, and the
balancer library remain unchanged for the cases where vertical
splitting is still required (output-bottlenecked rows, recipes whose
horizontal extent would exceed the layout budget).

## Motivation

The concrete failing case today is `processing-unit @ 2/s`,
URL `?item=processing-unit&rate=2&machine=assembling-machine-3&in=iron-plate,copper-plate,coal,water,crude-oil`:

- electronic-circuit total rate ≈ 50/s.
- `max_machines_per_row` for EC under express-belt cap is **6** (driven
  by 7.5/s/machine copper-cable input × 6 = 45/s = full express belt).
- Solver allocates 20 machines → placer splits into **4 rows of 5
  machines** each.
- Lane planner sees 4 producers, 2 consumers, total rate 50/s. With
  the per-lane cap (22.5/s for express) it computes `n_splits = 3`,
  exceeding consumer count, and `split_overflowing_lanes` skips the
  third (consumer-empty) split — silently dropping producer row 10.
- Symptom: the row at y=113 emits its output belt (5 tiles east-west)
  but no return spec is generated, leaving the belt severed at (24,
  113). The new `fluid-network`-style validator now flags this as a
  "belt-dead-end" error.

A separate tactical fix (extend the balancer-family path to handle
fan-in N>M cases) is in flight as a workaround. This RFP addresses
the underlying architectural mismatch: **rows fragment when they
shouldn't**, then we spend complexity stitching them back together.

It also addresses a known under-utilisation: trunks fed by sideload
return belts run at half-belt capacity (one lane only, per F5/B8). The
horizontal-trunk pattern bypasses the return belts entirely — output
goes directly onto a full belt that the bus consumes.

## Design

### Row template (the new shape)

Today, an N-input row template lays a fixed number of input belts at
the top of the row. For `electronic-circuit` (DualInput row), that's
roughly:

```
y+0 : input belt 1 (iron-plate, east-flow)
y+1 : inserter row (drops onto machine input from belt at y+0)
y+2 : input belt 2 (copper-cable)
y+3 : inserter row
y+4..y+6 : machine 3×3
y+7 : output inserter row
y+8 : output belt
```

The horizontal-trunk variant adds **K extra input belts** at the top
of the row (where K is the number of machine blocks). For example, a
24-machine EC row with 4 blocks of 6 machines each:

```
y+0 : copper-cable belt for block 4 (running east, dives to block 4)
y+1 : copper-cable belt for block 3 (dives to block 3)
y+2 : copper-cable belt for block 2 (dives to block 2)
y+3 : copper-cable belt for block 1 (dives to block 1)
y+4 : iron-plate belt (one belt suffices, ≤ 45/s)
y+5 : inserter row for iron-plate
y+6 : inserter row for copper-cable per-block
y+7..y+9 : machines (block 1 starts at x_offset)
y+10: output inserter row
y+11: output belt (single, full lane-split capacity)
```

The block layout repeats horizontally: each block has its own
copper-cable input feeding only those 6 machines, but iron-plate (low
demand) is shared across the whole row via one belt.

**The "dive" mechanism:** at the eastern end of belt N, a UG-belt
input dives down past intervening belts and emerges at the inserter
row for block N. Belts for further-east blocks pass over the UG
tunnel (per U4). Underground reach (yellow 4 / red 6 / blue 8) limits
how many belts can stack — at express tier, 8 stacked belts is the
hard cap for a single dive depth.

**Full-belt utilisation:** the dived belt feeds the inserter row
straight (B7), filling both lanes. No sideload, no half-capacity.

### Lane planner

`max_machines_per_row` becomes a soft hint rather than a hard cap.
For each input that would otherwise force a split:

1. Compute the per-belt demand: `machines × per_machine_rate`.
2. Compute how many belts of that input the row needs:
   `ceil(per_belt_demand / full_belt_capacity)`.
3. Pick the input with the highest belt count (the "horizontally
   constrained" input).
4. Use that count as K, the number of stacked input belts at the top
   of the row.

If K=1, fall through to today's vertical-split behaviour.

### Bus integration

A horizontally-trunked row is a single producer with a single output
belt. From the lane planner's perspective, it's exactly what we
already model for an "intermediate with one producer". The K stacked
input belts each tap off the bus separately — same as today's K=1
case but with K parallel taps at sequential ys. No balancer family
needed at the producer side.

Output flow goes through:

- Row template's lane-split bridge → both lanes of output belt
  filled.
- Existing tap-off splitters along the bus deliver to consumers.

(Return-belt routing in `ghost_router.rs` for this row type may be
**unnecessary** — the output belt can live on the bus directly via
the same `output_merger` machinery used for final products. To
investigate during phase 1.)

### Code touched

- `crates/core/src/bus/templates.rs` — new template variants for
  horizontally-trunked single-input, dual-input, fluid-input rows.
  Estimate: 1-2 new functions per row kind, ~200-400 LOC each.
- `crates/core/src/bus/placer.rs` — change `max_per_row` decision to
  return both a row-machine-count and a stack-depth K; drive
  template selection from K.
- `crates/core/src/bus/lane_planner.rs` — collapse K=1 horizontal
  rows into the single-producer code path; update tap-off generation
  for the K parallel input taps.
- `crates/core/src/bus/ghost_router.rs` — verify K parallel taps
  route correctly; possibly drop the return-belt path for K>1 rows.
- `crates/core/tests/e2e.rs` — regenerate golden hashes for affected
  tier3+ recipes (electronic-circuit-heavy ones first).

### Trade-offs against alternatives

**Alt 1: balancer-family fan-in** (the in-flight tactical fix). Keeps
the vertical-split architecture; uses N→M balancers from
`balancer_library.rs` to merge fragmented producer outputs onto
M=consumer-count trunks. Pros: small change, uses existing
machinery. Cons: still wastes the second lane of intermediate
trunks; doesn't reduce row count; the balancer block adds rows of
height to the layout.

**Alt 2: dual-sided returns**. Feed return belts onto the trunk from
both sides (east AND west) so both lanes fill via two sideloads.
Pros: doubles per-trunk capacity; localised change. Cons: only
helps if there's space on both sides of the trunk; doesn't reduce
total row count; adds routing complexity for the west-side feed.

**Alt 3 (this RFP): horizontal trunks**. Rejects the premise that
rows must split when input-bottlenecked. Pros: fewer rows, simpler
bus, no return belts, no balancer block, output runs at full
capacity by construction. Cons: more template work, wider rows,
doesn't help output-bottlenecked recipes (rare for AM3).

The tactical fix and this RFP are not mutually exclusive: the
balancer-family fix is the **interim** while horizontal trunks are
the **long-term direction**. Output-bottlenecked rows continue to
need the balancer machinery.

## Kill criteria

This RFP should be abandoned or rethought if any of the following
trips:

- *After implementing K-belt stacking for `electronic-circuit`, the
  total tile area for `processing-unit @ 2/s` (entities × bounding
  box) regresses by more than 15% relative to today's vertical-split
  layout.* — the win must outweigh the wider rows.

- *After implementing dive UG routing, more than 10% of the K-belt
  cases in the test corpus require dive depth > 8 (express UG max
  reach).* — would mean the design has hidden constraints that
  collapse the strategy to something else.

- *The K-belt template adds more than ~600 LOC to `templates.rs` per
  row kind (single-input, dual-input, fluid-input).* — too much
  surface for a single architectural improvement; reconsider whether
  a more general row-builder abstraction is warranted first.

- *After phase 1, the validator still reports `belt-dead-end` errors
  on the `processing-unit @ 2/s` URL specifically attributable to
  intermediate-row return routing.* — would mean the K=1 fall-through
  is broken or the K>1 path didn't actually replace return belts.

- *End-to-end runtime for `cargo test --manifest-path
  crates/core/Cargo.toml` regresses by more than 1.5x on the existing
  test corpus.* — the new code path is more complex than expected.

## Verification plan

Per
[CLAUDE.md verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Full e2e suite green.** `cargo test --manifest-path
   crates/core/Cargo.toml`. Golden hashes for tier2+ tests will
   regenerate during phase 1; expected.
2. **The processing-unit URL renders cleanly.** Open
   `?item=processing-unit&rate=2&machine=assembling-machine-3&in=iron-plate,copper-plate,coal,water,crude-oil`
   in the dev server, tick Debug → Validation, confirm zero
   belt-dead-end errors and zero balancer-misroute errors.
3. **Snapshot inspection on three sample layouts:**
   - Tier3 `electronic-circuit` from ores at 5/s (existing test).
   - Tier4 `advanced-circuit` (which produces ec internally).
   - Tier5 `processing-unit` at 1/s and 2/s.
4. **Trace events.** New `RowSplit` events should report `kind=
   horizontal, k=N` for cases that would previously report
   `split_into=N+1` vertical. No `RouteFailure`, no `BridgeDropped`
   for horizontally-trunked items.
5. **Visual sanity in browser.** The K input belts should be visibly
   distinct (one per machine block), and items should be visibly
   present on both lanes of the output belt.

## Phasing

1. **Phase 1 — single dual-input recipe (electronic-circuit).** Add
   the horizontal-trunk dual-input template variant. Update placer
   to pick K for this recipe class. Verify on tier3 EC and tier5
   processing-unit. Land + regenerate goldens. Kill criteria above
   apply at end of phase 1.
2. **Phase 2 — single-input + triple-input recipes.** Same machinery,
   different templates.
3. **Phase 3 — fluid-input + multi-fluid rows.** More complex
   because the fluid-trunk dive shares the row's top space with
   solid-belt dives; needs careful y-allocation.
4. **Phase 4 — drop the return-belt path for K>1 rows** (optional).
   Investigate whether `ghost_router.rs`'s ret-spec generation is
   still needed for these rows; if not, simplify.

## Decision log

- *2026-04-25 — proposed.*
