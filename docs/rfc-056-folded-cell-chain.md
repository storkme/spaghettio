# RFC-056: Folded cell chains — cut-aware multi-row macro placement

Status: Design  
Tracking: #456  
Competes with: RFC-055

## Summary

Fold a logical cell chain across multiple physical rows, using validated cell
orientations and cut-aware fold points to shorten the factory's dominant
transport paths.

RFC-055 asks how far a better one-dimensional order can go. This RFC keeps the
same weighted placement graph and cell variants but adds row assignment,
serpentine orientation, and explicit inter-row routing corridors.

The initial candidate is deliberately narrower than general 2D placement:

- cells remain in an ordered sequence;
- each cell belongs to one row;
- rows are contiguous intervals of that sequence;
- alternate rows may run in opposite physical directions;
- inter-row edges use reserved vertical trunks.

This is a constrained floorplanner: more expressive than a line, much easier
to reason about than arbitrary rectangle placement.

## Motivation

A single line has an unavoidable limitation: a macro with several high-rate
inputs cannot be close to producer clusters on both sides without making
other edges long. Folding provides a second spatial dimension while
preserving most of the chain's ordering and routing structure.

A naïve width cap is not sufficient. It can fold directly through a dense set
of high-rate edges and turn short horizontal connections into congested
inter-row trunks. Fold points must therefore minimize weighted graph cuts.

## Goals

- Beat RFC-055 on rate-weighted route length and startup time.
- Bound factory width without merely exchanging it for longer belts.
- Reuse the same validated orientation interface and candidate metrics.
- Preserve independent K replicas.
- Keep placement deterministic and bounded.
- Retain a clean fallback to RFC-055 or the current composer.

## Non-goals

- Arbitrary free placement of rectangular macros.
- Interleaving cells from unrelated sequence intervals within one row.
- Machine-level movement inside a cell.
- Post-route rubber-band compaction.
- Shared production across capacity replicas.

## Logical model

Begin with an order proposed by RFC-055's search. Partition it into `R`
contiguous intervals:

```text
row 0: A → B → C → D
                    |
row 1: H ← G ← F ← E
|
row 2: I → J → K → L
```

Alternate rows use 180° variants where available so sequence neighbours meet
near a fold. If a macro has no validated 180° variant, the row may retain its
orientation and pay the resulting port-distance cost; transform availability
is a placement constraint, never an unchecked mutation.

## Fold objective

For a candidate order and row partition:

```text
score =
    RFC-055 edge-distance objective
  + inter_row_weight × Σ(rate crossing a row boundary)
  + trunk_weight × estimated vertical-trunk length
  + congestion_weight × maximum weighted cut
  + aspect_weight × aspect-ratio penalty
```

The partitioner chooses fold points at low weighted cuts, not merely at equal
width.

Candidate row counts are bounded, initially `R ∈ {2,3,4}`. Row height and
width limits are derived from macro dimensions plus corridor reservations.

## Physical structure

Each row owns:

- one macro band;
- local east/west connection corridors;
- a north and south escape strip;
- reserved columns at each end for fold and inter-row trunks.

The floorplan owns:

- external feed boundary on the north or west perimeter;
- target drains on the south or east perimeter;
- vertical trunk channels between row bands;
- power-continuity corridors;
- no-build margins required by the simulation boundary kit.

Inter-row edges do not improvise paths through macro bands. They descend or
ascend through allocated trunk columns, cross in the gap between rows, then
enter the destination row through its escape strip.

## Serpentine versus same-direction rows

Both are candidates:

- **Serpentine:** alternate rows rotate 180°. Consecutive sequence endpoints
  meet at the same side, making fold edges short.
- **Same direction:** every row faces the same way. Cell variants are simpler,
  but sequence folds traverse the full row width.

The search scores both. Serpentine is not assumed superior because shared
producer edges may prefer aligned port sides.

## Validated orientation contract

RFC-056 imports RFC-055's `CellVariant` contract unchanged. In particular:

- rotation is first-class;
- unavailable transforms constrain placement;
- splitter anchors, underground pairs, inserter semantics, fluid ports,
  priorities, boundaries, power, and metadata are transformed and validated;
- mega-cells rotate as complete adapted blocks.

RFC-056 adds one requirement: a row reversal must use a validated 180° variant
for every macro it reverses. There is no “flip the row after stamping” path.

## Search

The bounded search has three nested decisions:

1. Select one of RFC-055's top linear orders.
2. Select row count and contiguous fold points.
3. Select legal variants and serpentine/same-direction row orientation.

Initial algorithms:

- dynamic programming for fixed-order weighted-cut partitioning;
- adjacent fold-point improvement;
- seeded annealing over order swaps plus fold shifts for the final bounded
  candidate set.

The dynamic program provides an exact baseline for “best folds of this order.”
Annealing is allowed to improve the order but must beat that baseline.

## Routing strategy

Phase 1 supports:

- adjacent cells in the same row;
- fold edge between the last cell of one row and first of the next;
- non-adjacent same-row bypass;
- non-adjacent inter-row trunks;
- solid belts only outside an unchanged mega-cell.

Fluid edges crossing rows are initially refused unless their complete
mega-cell adapter owns both endpoints. This prevents the first implementation
from expanding into general multi-row pipe routing.

The refusal is an RFC-056 candidate refusal: RFC-055 remains available.

## Shared benchmark

RFC-056 uses exactly RFC-055's corpus and metrics:

- `mega-chain-usp2raw`;
- `mega-chain-chem5raw`;
- `mega-chain-pu4raw`;
- small solid-only control;
- geometry-hashed cell registry.

Report:

- belt and pipe entities;
- area and aspect ratio;
- maximum and rate-weighted route length;
- critical-path length;
- inter-row weighted cut and trunk occupancy;
- fast-meter throughput;
- Factorio final throughput and time to 90%.

## Decision gate against RFC-055

RFC-056 earns implementation/default consideration only if, under comparable
search time:

- rate-weighted route length improves at least 15% beyond RFC-055 on two deep
  fixtures, or critical-path length improves at least 25%;
- belt entity count does not grow more than 5%;
- fast-meter target throughput does not regress more than 2%;
- Factorio throughput is no worse within measurement noise;
- time to 90% improves materially on at least one fixture where RFC-055 has
  already improved the control;
- routing/validation runtime stays within 2× RFC-055's bounded search.

If it fails, the result is still useful: RFC-055 becomes the chosen compact
chain architecture and full 2D placement remains a separate future RFC.

## Relationship to general 2D placement

Folded placement is an experiment, not a commitment to stop at rows.

Evidence for a later general macro-placement RFC:

- large remaining weighted distance caused by the contiguous-row constraint;
- high-rate edges repeatedly crossing row cuts;
- strong sensitivity to fold count;
- a clear meter benefit from small manual non-contiguous moves.

Evidence against escalation:

- RFC-055 or RFC-056 already removes most transport inventory;
- routing congestion, not macro distance, becomes dominant;
- Factorio startup stops improving with geometric metrics.

## Phases

1. Metrics-only fold simulator over RFC-055 placement graphs.
2. Two-row solid-only composer.
3. Validated serpentine variants and fold routing.
4. Three/four-row search plus fast-meter ranking.
5. Deep-fixture Factorio comparison and accept/reject decision.

## Open questions

- Should row count be selected by score or exposed as a user constraint?
- Where should external raw feeds enter a folded replica?
- Should row gaps be fixed or congestion-sized?
- Can power corridors double as reserved inter-row routing channels?
- When does a high weighted cut justify duplicating a producer rather than
  routing across rows? Duplication is out of scope here but should be measured.
- Should K replicas be tiled in a grid after each replica is folded?

