# RFC-055: Compact cell chains — flow-weighted ordering and validated orientation

Status: Design  
Tracking: #456  
Competes with: RFC-056

## Summary

Reduce the transport distance and startup inventory of composed cell chains
without replacing their one-dimensional slot architecture.

The current chain composer derives one deterministic recipe order, stamps each
cell west-to-east, then routes producer→consumer edges between those fixed
slots. The result is correct but spatially indifferent: a high-rate input can
travel hundreds of tiles because dependency order, not physical transport
cost, chose the slots.

This RFC makes cell order and orientation placement decisions:

1. Extract a weighted producer→consumer graph from `SolverResult`.
2. Enumerate prevalidated orientation variants for every cell.
3. Search deterministic linear orders and variants.
4. Score candidates primarily by rate-weighted port distance and critical-path
   length, with footprint and congestion as secondary terms.
5. Route and validate only the best bounded set of candidates.

The output remains a single logical chain. RFC-056 asks whether folding that
chain across multiple rows justifies a larger architecture change.

## Motivation

`mega-chain-usp2raw` is the concrete anchor. In one replica of the current
layout, approximate recipe positions are:

| Recipe group | x |
|---|---:|
| oil mega-cell / plastic | 236 |
| copper plate | 288 |
| steel plate | 612 |
| low-density structure | 692 |
| utility science | 732 |

LDS is close to steel but roughly 400–450 tiles from its high-rate copper and
plastic sources. A long-warmup Factorio run after the oil deadlock fix produced
only 0.85 LDS/s against 2.0/s planned. Replacing steel's serial fan-out with a
balanced 1→3 topology raised LDS to 1.13/s, but the remaining long copper and
plastic paths still dominate startup inventory.

Local splitter policy cannot solve a placement problem.

## Goals

- Shorten rate-important producer→consumer connections.
- Shorten the external-input→target critical path.
- Reduce belt entities and in-flight item inventory.
- Preserve existing cell interiors and the chain router.
- Make rotations and mirrors first-class, tested placement choices.
- Produce deterministic output for a fixed seed and budget.
- Establish the baseline RFC-056 and later 2D placement must beat.

## Non-goals

- Arbitrary machine-level compaction inside cells.
- General two-dimensional macro placement.
- Rubber-band post-route compaction.
- Sharing machinery between the K capacity replicas.
- Optimizing area without a measured-throughput constraint.

## Placement graph

Create one macro vertex for every ordinary cell and collapsed mega-cell.
Create an edge for each transported item from its producer macro to every
consumer macro.

Each edge records:

- item;
- planned rate delivered to that consumer;
- fluid/solid kind;
- producer port candidates;
- consumer port candidates;
- whether the edge is direct insertion, belt, or pipe;
- whether it lies on an external-input→target critical path.

The graph is placement intent. It is derived before tile routing and must not
be reconstructed from a finished `LayoutResult`.

## Objective

For order `O` and selected variants `V`, estimate:

```text
score =
    Σ solid_edge_rate × estimated_port_distance
  + fluid_weight × Σ fluid_edge_rate × estimated_port_distance
  + critical_weight × longest_external_to_target_path
  + congestion_weight × estimated_cut_congestion
  + area_weight × estimated_bounding_area
  + backward_weight × rate_of_westward_edges
```

The first term is primary. Bounding area is deliberately not primary: #448
showed that removing apparent belt slack can remove the margin required for
real delivery. Throughput gates decide whether a smaller result is acceptable.

`fluid_weight` is nonzero but lower than the solid weight. Factorio 2.0 fluids
do not carry belt-style item inventory, but pipe length still consumes space
and routing capacity.

### Reported metrics

Every candidate report includes:

- total belt and underground-belt entities;
- total pipe and pipe-to-ground entities;
- bounding width, height, and area;
- maximum producer→consumer route length;
- rate-weighted route length;
- external-input→target critical-path length;
- estimated and realized crossing count;
- fast-meter target rate;
- Factorio target rate and time to 90% of its final rate.

## Search

Phase 1 implements several bounded, deterministic competitors:

1. Current order (control).
2. Reverse order.
3. Weighted barycentric insertion.
4. Adjacent-swap hill climbing.
5. Seeded simulated annealing with a fixed evaluation budget.

The cheap objective evaluates thousands of orders without routing. The best
`N` distinct orders advance to geometry construction and validation. The best
validated `M` advance to the fast meter. Factorio is reserved for the final
winner and control.

Default proposal: `N=16`, `M=4`; tune from measured runtime rather than
assuming these values.

Westward connections are legal. They carry a congestion penalty, not a hard
prohibition, because the existing chain router already supports them.

## Validated cell orientations

Rotations are allowed and expected. What is forbidden is rotating arbitrary
placed entities without proving that every associated contract transformed.

Each extracted cell exposes `CellVariant` values:

```text
CellVariant {
    transform: Identity | Rotate90 | Rotate180 | Rotate270
             | MirrorX | MirrorY,
    entities,
    ports,
    width,
    height,
    boundary_records,
    validation_fingerprint,
}
```

A variant enters the placement search only after passing:

- entity-overlap validation;
- belt and underground-belt connectivity;
- inserter pickup/drop reachability;
- pipe segment and pipe-to-ground pairing;
- recipe fluid-port identity;
- power coverage;
- boundary-record/entity agreement;
- item isolation;
- blueprint export→parse round trip.

### Why validation is required

Several entities are not transformed by changing `(x,y,direction)` alone:

- splitter anchors depend on their oriented two-tile footprint;
- underground-belt input/output endpoints must rotate as a pair;
- inserter blueprint direction uses the game's pickup-side convention;
- pipe-to-ground blueprint direction names its surface opening;
- mirrored fluid machines have recipe-specific port identities;
- splitter priorities are relative left/right annotations;
- boundaries, segment metadata, and harness access coordinates move too.

These facts argue for validated variants, not against rotation.

### Variant generation policy

- Ordinary solid cells: attempt all four rotations and both useful mirrors.
- Cells containing fluid machines: begin with identity and 180°; admit 90° and
  mirrors only after the fluid-port validator passes.
- Mega-cells: transform the whole block including surplus records and boundary
  adapters, never the interior entities independently.
- A rejected transform is a normal unavailable variant, not a composition
  failure.

Variants are geometry-hashed and cached beside existing cell artifacts.

## Placement and routing phases

1. Solve and identify the cell/mega-cell graph.
2. Generate or load validated variants.
3. Run cheap order/variant search.
4. Assign slot widths using the selected variants and required corridor
   reservations.
5. Route the best candidates with the existing crossing-aware router.
6. Validate full `LayoutResult`.
7. Meter and rank validated candidates.
8. Return the winner only if it clears the acceptance gates; otherwise return
   the current-order control.

The fallback makes the feature monotonic: search failure cannot make an
otherwise composable factory unavailable.

## Acceptance gates

Corpus:

- `mega-chain-usp2raw`;
- `mega-chain-chem5raw`;
- `mega-chain-pu4raw`;
- one small solid-only chain;
- all geometry-hashed cell registry fixtures.

Correctness:

- zero new validator errors;
- no new item-isolation or fluid-segment warnings;
- deterministic geometry hash for a fixed seed/budget;
- blueprint round trip preserves selected variants and ports.

Quality:

- at least 25% lower rate-weighted route length on `usp2raw`;
- at least 15% fewer belt/underground entities or 20% shorter critical path;
- no fixture loses more than 2% fast-meter target throughput;
- Factorio target throughput must not regress beyond measurement noise;
- time to 90% of final target rate must improve on at least two deep fixtures.

These are design thresholds. The implementation PR records the control
distribution and may tighten them before the decision.

## Interaction with RFC-056

RFC-056 reuses:

- `CellVariant`;
- the weighted placement graph;
- metrics and candidate reports;
- search determinism;
- validation and measurement gates.

RFC-055 ships independently if it clears its gates. RFC-056 is accepted only
if folding materially outperforms RFC-055 after controlling for orientation
and search budget.

## Phases

1. Metrics-only control: report current order without changing geometry.
2. Variant infrastructure and transform gates.
3. Linear order search behind an opt-in flag.
4. Fast-meter candidate ranking.
5. Factorio anchors and default-candidate decision.

## Open questions

- Should belt occupancy (stacking) scale edge weight, or should planned
  item-rate remain the stable cross-configuration metric?
- Should the target macro be pinned to an external boundary?
- Does cut congestion need a hard cap before routing?
- Should K replicas share one chosen order or search independently? This RFC
  says they share one order for deterministic identical copies.
- Which cell transforms are valuable enough to cache eagerly?

## Decision log

- **2026-07-26 — Phase 1 foundation implemented.** Added the shared
  geometry-independent placement graph and candidate metrics in
  `bus::cells::placement`. Consumer edges are weighted by total planned
  demand, linear candidates report rate-weighted distance, maximum distance,
  backward rate, weighted cuts, and estimated footprint, and the first
  deterministic competitor performs best-improving adjacent swaps to a local
  optimum. Tile routing remains unchanged.
