# RFC-057: Topology-preserving dense factory repacking

Registry: [`rfcs.md`](rfcs.md). Status: **Active prototype**.

## Summary

Increase factory density by moving the existing machines into compact
two-dimensional production islands and synthesizing all transport geometry
again.

The production topology is already solved and is immutable:

- the recipe and machine multiset;
- machine counts, qualities and modules;
- producer→item→consumer relationships;
- planned rates and external inputs;
- surplus and target outputs;
- direct-insertion eligibility.

The physical realization is deliberately **not** immutable. The repacker may:

- translate, rotate and mirror machines where their port contract validates;
- replace every belt, underground belt, splitter, merger and balancer;
- choose new belt lanes and underground spans;
- share trunks between consumers of the same item;
- use `(n,m)` balancers to distribute a producer bank evenly;
- replace a belt edge with direct insertion when the existing production edge
  is eligible;
- regenerate pipes and power infrastructure around the new placement.

This is the **spaghettifier** proposed in
[#456](https://github.com/storkme/spaghettio/issues/456): a post-pass over a
known-correct factory, not another from-scratch generator or cell-ordering
RFC. Cells are optional seeds and reusable local patterns, not indivisible
placement units.

## Motivation

RFC-055 demonstrated the limit of macro reordering. It reduced estimated
rate-weighted distance by 16–40% and reduced belts on three fixtures, but the
physical belt reduction was only 10–17% on the deep winners and USP grew by
8.5%. RFC-056's contiguous folds did not materially improve two of the three
deep fixtures.

The retained architecture dominates the result:

- each edge receives a mostly private corridor;
- K capacity replicas duplicate feeds, corridors and poles;
- producers and consumers retain full cell boundary adapters even when they
  could form one local island;
- long belts are routed around macro rectangles whose internal slack the
  placer cannot use;
- arbitrary 2D adjacency and shared item trunks are unavailable.

Shuffling rectangles cannot remove interfaces. RFC-057 is allowed to remove
and rebuild those interfaces while preserving what the factory produces.

## Topology invariant

Define a canonical `ProductionSignature` before placement:

```text
ProductionSignature {
    machines: multiset(recipe, entity, quality, modules, count),
    edges: multiset(producer_recipe, item, consumer_recipe, planned_rate),
    external_inputs: multiset(item, rate, fluid),
    target_outputs: multiset(item, rate),
    surplus_outputs: multiset(item, rate, fluid),
}
```

Every emitted candidate reconstructs the same signature from its placed
machines and transport reachability. A mismatch is a hard refusal.

This distinguishes two meanings of topology:

- **Production topology is fixed.** Copper cable still comes from the same
  cable machines and supplies the same circuit consumers.
- **Logistics topology is free.** That edge may be a direct inserter, a short
  manifold, a balanced shared trunk, or a belt/underground route around other
  machinery.

The invariant is checked mechanically; it is not an assumption carried from
the source layout.

The source layout's belt graph is also not part of the signature. Preserving
it would preserve the dominant source of sprawl. A candidate may change
splitter count, branching order, underground spans, route homotopy and shared
trunks as long as every production edge remains physically reachable at the
required capacity.

## Starting artifact and topological IR

The pass consumes a validated `LayoutResult` plus its `SolverResult`. It does
not modify the source in place.

Extract a new `CompactIR` containing:

- movable machine/pattern blocks and validated orientation variants;
- fixed production edges from `ProductionSignature`;
- external feed and drain terminals;
- inserter reach/face constraints;
- fluid-port constraints;
- power requirements;
- existing routes as an initial topological embedding only.

The existing geometry supplies a guaranteed incumbent and useful routing
intent (`carries`, segment ids and boundary records). The compactor may use
that engine IR. The meter remains independent and sees only the exported
blueprint and manifest.

`CompactIR` separates three things the current layout conflates:

1. production edges that must survive;
2. relative-placement constraints required by direct insertion and fluid
   ports;
3. replaceable logistics routes.

## Constraint-graph compaction baseline

The first competitor is the exact polynomial baseline from #456 and VLSI
layout compaction.

For one axis, construct a directed separation graph:

```text
A ── minimum_clearance(A,B) ──> B
```

An edge records the minimum coordinate difference needed to prevent entity
footprint overlap or to retain a deliberately reserved face/channel. Longest
paths in the resulting DAG give the minimum legal coordinates for that
ordering.

Run:

```text
compact X → reroute → compact Y → reroute
```

until occupied area no longer improves. Alternate once more after any
orientation or ordering move.

This competitor is important even if the later search wins:

- it establishes how much empty coordinate slack exists without changing
  relative order;
- every move is explainable by a named separation constraint;
- it supplies a deterministic compact incumbent;
- it distinguishes "the rows were padded" from "the architecture needs a
  different embedding."

The constraint graph must not encode every existing belt tile as required
clearance. It reserves endpoints, machine faces and channels needed by the
current topological embedding; the route is then re-embedded and may use
underground transit beneath otherwise occupied surface tiles.

## Rubber-band routing and move set

Constraint compaction alone cannot move a producer through its consumer or
choose a better side of an obstacle. The second competitor uses a topological
routing IR and bounded search.

Each route initially records:

- source and destination terminals;
- ordered obstacle-side decisions;
- required branch/merge semantics;
- belt tier and capacity;
- shared-trunk membership, if any.

After blocks move, the router re-embeds that intent onto tiles. It may also
propose a different embedding—changing obstacle sides, introducing an
underground crossing, or joining a shared trunk. Existing route homotopy is an
incumbent, not a correctness constraint.

Initial flow-preserving moves:

- translate one block or island;
- swap two blocks;
- rotate/mirror through a validated variant;
- move a machine between compatible rows;
- bind/unbind a direct-insertion pattern;
- merge/split production islands;
- replace private edges with one shared item trunk;
- replace a serial fan with an `(n,m)` balancer;
- rip up and reroute one net inside a bounded box;
- change a surface span into a legal underground span.

Every move is transactional: place, reroute affected nets, validate the
production signature and physical layout, meter if it survives, otherwise
restore the incumbent.

## Placement unit

The primitive placement unit is a machine with typed face demand:

```text
MachineNode {
    recipe,
    entity,
    footprint,
    orientation_variants,
    item_inputs,
    fluid_inputs,
    outputs,
    inserter_slots,
    pipe_ports,
    power_demand,
}
```

The search may bind several nodes into a temporary rigid pattern when doing so
is useful:

- direct-insertion producer/consumer sandwiches from RFC-053;
- smelting columns;
- one producer bank plus an `(n,m)` output balancer;
- fluid machinery whose pipe-port geometry is already validated;
- a known-good cell used as an initial incumbent.

Patterns are conveniences, not permanent boundaries. Solid rows may be broken
apart when a denser placement meters better.

## Production islands

Build islands around high-rate dependencies rather than dependency order.

1. Weight every production edge by planned rate and belt inventory cost.
2. Seed clusters with direct-insertion pairs and the highest-weight edges.
3. Grow clusters while local faces and route capacity remain feasible.
4. Place consumers around their dominant producer banks.
5. Give each island a small number of shared raw/intermediate interfaces.
6. Place islands in 2D and route the remaining inter-island edges.

For utility science, likely islands include:

- copper plate + plastic + steel → low-density structures;
- engines + electric engines + batteries → flying robot frames;
- circuits + processing units;
- final utility-science assembly.

The clustering is proposed by the algorithm and judged by realized geometry
and meter output; these examples are not hard-coded recipe knowledge.

## Logistics synthesis

Routing starts from an empty logistics layer.

### Local delivery

In priority order:

1. validated machine→machine direct insertion;
2. short balanced manifold beside the consumer bank;
3. shared item trunk with balanced taps;
4. private point-to-point route as fallback.

The router may use ordinary and underground belts interchangeably. Underground
belts are routing edges with endpoint occupancy and a maximum span, not copies
of the source layout's underground choices.

### Shared trunks

One item with several consumers should normally have one owned distribution
network. The producer bank feeds an `(n,m)` balancer selected from the existing
library; its outputs serve consumer manifolds with planned-rate capacity.

The fast meter, not a priority heuristic, decides whether a smaller serial tap
is acceptable. A topology that only becomes fair after upstream consumers
back up is expected to lose on startup and trailing delivery.

### Crossings

The router may:

- cross surface belts with underground pairs;
- tunnel beneath machines where the underground endpoints remain outside the
  footprint and the span is legal;
- reserve multi-item routing channels only where congestion requires them;
- change route direction and entry face when the machine variant permits it.

Machine footprints remain hard obstacles. Underground transit tiles are not
surface occupancy, but their endpoints and same-axis pairing constraints are.

### Fluids

The first physical slice keeps each validated fluid mega-block rigid while
moving it as one island member. Its solid adapters may be rerouted freely.

A later slice may repack fluid machines individually only after pipe-port
identity, pipe-to-ground orientation and recipe fluid-box validation are part
of the candidate gate. There is no fluid-void escape hatch.

## Search

Use a staged, deterministic search:

1. **Topology extraction.** Freeze and hash `ProductionSignature`.
2. **Exact compaction incumbent.** Alternate constraint-graph X/Y compaction
   with route re-embedding.
3. **Incumbents.** Exact compacted source, RFC-055 order, and existing DI
   patterns.
4. **Island proposals.** Weighted graph clustering with several deterministic
   thresholds.
5. **Coarse placement.** Seeded annealing or large-neighbourhood search over
   node position, orientation, cluster membership and bounded route rip-up.
6. **Route synthesis.** Route high-rate edges first, then shared trunks,
   local manifolds and remaining edges.
7. **Static gate.** Production signature, entity overlap, belts, undergrounds,
   inserters, fluids, item isolation, power and blueprint round trip.
8. **Fast-meter gate.** Export the actual blueprint and meter it; no IR-only
   approximation enters the score.
9. **Factorio finalist.** Run only the control and final candidate when the
   meter predicts a material win or exposes uncertainty.

The cheap search objective is:

```text
score =
    occupied_tiles
  + belt_weight × belt_entities
  + inventory_weight × rate_weighted_route_length
  + congestion_weight × reserved_channel_area
  + interface_weight × logistics_boundary_count
```

After routing, candidates are ranked lexicographically:

1. meet the throughput floor;
2. maximize delivered target rate per occupied tile;
3. minimize occupied tiles;
4. minimize belt entities and startup inventory;
5. minimize search/runtime cost.

Density may never buy a slower factory and call itself a win.

## Fast-meter loop

`spaghettio_meter::Factory` consumes the exported blueprint and manifest, so
candidate evaluation remains independent of the engine's derived rate model.

For each statically valid candidate:

1. export blueprint + manifest;
2. build a meter `Factory`;
3. run a short screening window;
4. discard candidates below 95% of the incumbent's delivered target rate;
5. run survivors to meter convergence;
6. record target delivery, time series, machine census and boundary refusals.

The search may cache by blueprint geometry hash. Meter notes are candidate
refusals when they concern unsupported physics on the target path.

Factorio remains the oracle. Meter results rank candidates; they do not bless
new mechanics.

## Metrics

Report for every finalist:

- bounding-box area;
- occupied footprint tiles;
- target throughput per occupied tile;
- belt, underground-belt, splitter and inserter counts;
- pipe and pipe-to-ground counts;
- rate-weighted route length;
- estimated in-flight item inventory;
- production-island count;
- shared-trunk and private-edge counts;
- logistics interfaces crossed per produced target item;
- fast-meter final target rate, convergence and startup curve;
- Factorio final target rate and time to 90%, when run;
- placement, routing and meter runtime.

Occupied tiles, not bounding box alone, is the primary spatial denominator:
deliberate routing courtyards should not look dense merely because a rectangle
around them is narrow.

## Acceptance gates

Corpus:

- `mega-chain-usp2raw`;
- `mega-chain-chem5raw`;
- `mega-chain-pu4raw`;
- `chain-mil5ore`;
- one direct-insertion-rich solid control.

Correctness:

- production signature exactly preserved;
- zero validator errors and no new item/fluid isolation warnings;
- deterministic geometry hash for fixed seed and budget;
- blueprint export→parse preserves machines, recipes, modules, qualities and
  boundary records;
- no simulator-only fluid void.

Density:

- at least 35% fewer belt/splitter entities on two deep fixtures;
- at least 40% fewer occupied tiles on two deep fixtures;
- at least 2× target throughput per occupied tile on one deep fixture and
  at least 1.5× on a second;
- no deep fixture may grow occupied area by more than 5%.

Performance:

- no candidate loses more than 2% fast-meter target throughput;
- at least two deep fixtures improve meter time-to-90% by 20%;
- Factorio target throughput is no worse within measurement noise for the
  final candidate;
- any meter/Factorio ranking disagreement is recorded before promotion.

These thresholds intentionally demand a step change. Results in RFC-055's
10–17% belt-reduction range reject the approach as another marginal
rearrangement.

## Phases

1. Production-signature and `CompactIR` extractor.
2. Alternating X/Y constraint-graph compactor with route re-embedding.
3. Transactional boxed moves and underground-aware rip-up/reroute.
4. Metrics-only island clustering and machine-level 2D placement.
5. Solid local manifold/shared-trunk router with `(n,m)` balancers.
6. Blueprint export, full validation and fast-meter ranking.
7. Rigid fluid-block integration and deep-fixture comparison.
8. Factorio control/finalist adjudication and accept/reject decision.

## Kill criteria

- If preserving machine-level topology prevents a 35% belt reduction on both
  chemistry and processing units, stop and investigate topology-changing
  production plans rather than relaxing the density gate.
- If shared trunks repeatedly lose meter throughput to private corridors,
  keep island placement but reject shared logistics as the default.
- If routing consumes more than half of total candidate-search time before
  producing four meterable candidates, introduce incremental routing or
  stronger placement feasibility constraints.
- If compact candidates depend on unsupported meter mechanics, do not tune
  around the meter; adjudicate the mechanic in Factorio or refuse it.
- If fluid-block rigidity dominates remaining area after solid compaction,
  open a dedicated fluid-repacking RFC rather than silently weakening port
  validation.

## Relationship to earlier RFCs

- RFC-053 supplies validated direct-insertion patterns.
- RFC-054 supplies the candidate-ranking instrument.
- RFC-055 supplies the linear incumbent and demonstrated that ordering alone
  is insufficient.
- RFC-056 supplies negative evidence against contiguous folding.
- Existing `(n,m)` balancer templates are logistics primitives, not placement
  constraints.

RFC-057 supersedes RFC-055/056 as the density research direction. It does not
make either experimental composer a production default.

## Decision log

- **2026-07-26 — Phase 1 signature and constraint baseline implemented.**
  `ProductionSignature` canonically freezes machines, fixed-point rates,
  item producer sets, consumers, external inputs, targets and surplus.
  `PlacedMachineSignature` independently freezes the exact emitted machine
  multiset while ignoring coordinates and entity ordering. Multiple oil
  producers are represented explicitly rather than forced through the
  single-producer assumption used by solid cell chains.

  The first exact per-axis longest-path compactor preserves the relative
  order of rectangles whose cross-axis footprints overlap. Eight alternating
  X/Y passes over the current physical machine rectangles produced:

  | Fixture | machines | source machine bbox | compacted bbox | area delta |
  |---|---:|---:|---:|---:|
  | `mega-chain-usp2raw` | 495 | 2202×118 | 1667×49 | −68.6% |
  | `mega-chain-chem5raw` | 184 | 817×32 | 607×17 | −60.5% |
  | `mega-chain-pu4raw` | 640 | 2684×63 | 2239×25 | −66.9% |
  | `chain-mil5ore` | 146 | 706×5 | 583×3 | −50.5% |

  Every resulting machine rectangle is pairwise non-overlapping. These are
  coarse potential bounds, not factory candidates: inserters, route endpoints,
  pipes, power and logistics have not yet been re-embedded. Their purpose is
  to answer whether the source contains enough positional slack to justify
  the router work. A 50–69% machine-bbox reduction says yes and clears that
  continuation gate by a wide margin.

- **2026-07-26 — First runnable post-pass and route-intent extraction.**
  Global empty-column stripping preserves every entity, remaps functional
  boundaries and only shortens horizontal underground spans. On
  `chain-mil5ore` it validates cleanly and reduces 720×34 to 712×34 (1.1%),
  confirming that whitespace removal alone is safe but immaterial.

  The fast meter measured the source and stripped artifacts identically:
  1.73/s military science produced, 1.28/s delivered, the same per-item
  rates, 136 working / 10 ingredient-short machines and 68,748 boundary
  refusals. Each 18,000-tick measurement took 0.40s.

  Route-intent extraction now groups physical belt provenance into commodity
  nets by item rather than by debug segment. Segment identity was falsified
  immediately: one real copper-plate delivery spans row, fan and corridor
  IDs. Capacity-copy suffixes were also rejected as net identity because
  corridors carry them while the row segments containing machine terminals
  do not. The corrected extractor permits the new router to repartition
  logistics capacity without changing production. It recovers 13 commodity
  nets on `chain-mil5ore` and proves every solved solid production edge has
  producer-drop and consumer-pickup terminals.

- **2026-07-26 — Rigid production islands extracted.** `CompactIR` now
  separates movable production islands from replaceable commodity routes.
  Each recipe-bearing machine retains every inserter that touches it. An
  inserter whose two ends touch machines unions them into one island, so
  direct insertion remains physical geometry rather than being accidentally
  converted into a belt edge. Belt-facing inserters expose item-labelled,
  island-relative pickup/drop terminals for the new router.

  `chain-mil5ore` yields 146 islands containing 508 machines/inserters and
  362 route terminals; the largest island contains five entities. Its rigid
  machine-plus-inserter bbox compacts from 706×7 to 583×5 under the existing
  alternating constraint pass (41.0% less bbox area). This is still a routing
  bound, not a runnable candidate, but unlike the earlier machine-only bound
  it reserves the complete inserter geometry the router must serve.
  Applying a proposed island placement is transactional and preserves the
  exact placed-machine signature; incumbent belts are explicitly left for
  rip-up/reroute rather than silently translated.

- **2026-07-26 — Existing-router reuse boundary identified.** The current
  `route_bus_ghost` entry point is coupled to `BusLane`, `RowSpan`, pre-stamped
  south-flowing trunks and output-row mergers, so treating dense islands as
  synthetic bus rows would reintroduce the architecture this RFC is trying to
  escape. RFC-057 will instead reuse its lower-level proven pieces:
  `ghost_astar` for negotiated paths, `render_path` for underground
  materialisation, and the junction solver for remaining crossings. A small
  manifold adapter will own multi-terminal commodity nets and shared-trunk /
  `(n,m)` selection.

- **2026-07-26 — Cross-layout `CompactIR` gate passed.** Solid production-edge
  terminal coverage and manifold construction now pass on the four
  representative layouts. With complete machine-plus-inserter islands
  reserved, the alternating placement bound is:

  | Fixture | islands | terminals | commodity manifolds | source bbox | compacted bbox | area delta |
  |---|---:|---:|---:|---:|---:|---:|
  | `mega-chain-usp2raw` | 495 | 1,149 | 20 | 2202×120 | 1715×57 | −63.0% |
  | `mega-chain-chem5raw` | 184 | 582 | 13 | 817×34 | 607×23 | −49.7% |
  | `mega-chain-pu4raw` | 640 | 1,416 | 11 | 2684×65 | 2239×33 | −57.6% |
  | `chain-mil5ore` | 146 | 362 | 13 | 706×7 | 583×5 | −41.0% |

  `build_manifold_nets` now materialises every island-relative terminal at a
  proposed placement and joins it with external input/output terminals by
  commodity. Every resulting manifold on these fixtures has at least one
  producer/input and consumer/output. This is the typed input to the new
  shared-trunk router; coordinates no longer depend on the source belt paths.

- **2026-07-26 — Flow-sized manifolds and first runnable belt compactor.**
  `CompactIR` now carries exact fixed-point commodity flow requirements from
  the solver. The representative express-belt requirements are only 28 total
  lanes (maximum three for one item) on USP, 14 (maximum two) on chemistry,
  30 (maximum eight) on processing units and 14 (maximum two) on military
  science. This disproves “one route per inserter” as the manifold shape:
  hundreds of terminals must aggregate into a small number of capacity lanes
  through local `(n,m)` structures.

  A conservative runnable post-pass now replaces uninterrupted horizontal
  surface-belt spans with maximal legal underground pairs, protects
  inserter/boundary/junction tiles, normalises pairs collapsed to adjacency
  during coordinate stripping, and then removes globally empty columns.
  Every representative candidate preserves the exact placed-machine
  signature and passes full structural validation:

  | Fixture | source → compacted | belt entities | belt delta |
  |---|---:|---:|---:|
  | `mega-chain-usp2raw` | 2214×193 → 2166×193 | 44,031 → 28,998 | −34.1% |
  | `mega-chain-chem5raw` | 834×56 → 800×56 | 4,202 → 3,182 | −24.3% |
  | `mega-chain-pu4raw` | 2704×90 → 2624×90 | 23,128 → 15,504 | −33.0% |
  | `chain-mil5ore` | 720×34 → 670×34 | 3,750 → 2,308 | −38.5% |

  The fast meter measured the military-science control at 1.73/s produced
  and 1.28/s delivered. The underground candidate improved that to 2.18/s
  produced and 1.79/s delivered while reducing meter wall time from 0.35s to
  0.24s. Thus the first real candidate clears the 35% belt gate and improves,
  rather than trades away, throughput. Chemistry and processing units remain
  below the RFC's 35% gate, so this primitive is retained as a strong
  incumbent but does not complete RFC-057.

  A competing global-band sketch was rejected before implementation: placing
  one commodity trunk band below all compacted islands produced coarse route
  estimates of 20,269 tiles for military science and 146,773 for USP because
  every inserter was dragged to a distant global band. The active router uses
  local, flow-sized manifolds interleaved with their production clusters.

- **2026-07-26 — Axis-symmetric transport fixed point.** The runnable
  compactor now undergrounds both horizontal and vertical straight runs,
  strips empty rows as well as columns, and repeats to a fixed point. The
  expanded candidates retain exact machine signatures, have no underground
  pairing warnings and validate cleanly:

  | Fixture | belt delta | occupied-tile delta |
  |---|---:|---:|
  | `mega-chain-usp2raw` | −63.8% | −54.0% |
  | `mega-chain-chem5raw` | −36.8% | −20.5% |
  | `mega-chain-pu4raw` | −55.7% | −38.8% |
  | `chain-mil5ore` | −41.2% | −27.0% |

  This clears the 35% belt gate on chemistry and processing units. USP also
  clears the 40% occupied-tile gate; PU is close. Chemistry and military
  science demonstrate why route-entity reduction alone is not completion:
  immutable machines/inserters dominate their remaining filled tiles, and
  local island placement must remove more interfaces.

  The updated military candidate meters at 2.13/s produced and 1.91/s
  delivered versus the control's 1.73/s and 1.28/s. Processing units improve
  electronic circuits from 64.27/s to 91.20/s at the standard window.
  Chemistry's standard-window cable rate falls from 38.29/s to 27.64/s, but
  neither artifact has a meaningful long-window chemistry result: without
  fluid-boundary simulation both eventually fill downstream buffers and
  report zero cable/EC production. This is a meter capability boundary, not
  evidence of a steady-state loss; chemistry requires Factorio adjudication
  once the candidate is otherwise final.

- **2026-07-26 — Transactional X/Y coordinate compaction implemented.**
  After transport resynthesis, the compactor now attempts occupied coordinate
  cuts on both axes. A cut shifts the entire downstream half-plane, coalesces
  only physically equivalent seam belts, refuses multi-tile footprint
  collisions, recomputes the authoritative copper-wire graph and commits only
  if full layout validation remains error-free. X and Y alternate to a fixed
  point.

  Final validated geometry on the representative artifacts is:

  | Fixture | source | compacted | entity count |
  |---|---:|---:|---:|
  | `mega-chain-usp2raw` | 2214×193 | 1966×192 | 47,875 → 19,443 |
  | `mega-chain-chem5raw` | 834×56 | 710×55 | 5,707 → 4,078 |
  | `mega-chain-pu4raw` | 2704×90 | 2397×87 | 27,634 → 14,530 |
  | `chain-mil5ore` | 720×34 | 568×32 | 4,550 → 2,846 |

  Military science remains meter-stable after the extra coordinate cuts:
  2.16/s produced and 1.90/s delivered versus 2.13/s and 1.91/s for the
  transport-only incumbent, with 2,078 meter belt tiles versus 3,754 in the
  control. The cut search therefore adds real footprint reduction without
  undoing the transport gain.

  This lands the deterministic, explainable #456 baseline as a working
  compaction system. It does not yet implement topology-free local manifolds,
  rotations or annealed island reordering; those remain the higher-risk
  competitors for the occupied-tile gate.

- **2026-07-26 — Normal pipeline integration.** `LayoutOptions` now exposes
  `compact_layout`, default `false`. When explicitly enabled,
  `build_bus_layout` runs the validated RFC-057 fixed point on the winning
  decomposition before returning it; candidate decompositions themselves
  remain uncompacted so selection scoring and retry logic are unchanged.
  A focused end-to-end test proves the default path is inert, the option
  preserves the exact placed-machine signature, never increases either
  dimension and returns a fully valid layout.

- **2026-07-26 — Debug-segment identity removed from seam constraints.**
  Coordinate cuts may now coalesce surface belts when item, tier and
  direction agree even if the incumbent `segment_id` differs. Segment IDs
  describe row/corridor provenance, not independent production topology; the
  earlier equality guard unnecessarily preserved those interfaces. On
  military science this topology-free seam move reduces the final candidate
  from 568×32 / 2,078 meter belt tiles to 552×32 / 2,024, while improving
  delivered science from 1.90/s to 1.93/s. Full validation remains the
  transactional admission gate.

- **2026-07-26 — Topology-free recipe-cluster placement implemented.**
  Rigid machine/inserter islands are now shelf-packed into recipe banks; the
  banks are placed in 2D by solver-rate-weighted production adjacency.
  Incumbent row order, corridor positions and segment provenance do not enter
  the placement. Every resulting island rectangle is pairwise disjoint and
  all manifold terminals move with its island.

  | Fixture | recipe clusters | clustered rigid bbox | rate-weighted manifold span delta |
  |---|---:|---:|---:|
  | `mega-chain-usp2raw` | 22 | 195×110 | −54.1% |
  | `mega-chain-chem5raw` | 10 | 109×83 | −40.2% |
  | `mega-chain-pu4raw` | 10 | 163×173 | −63.9% |
  | `chain-mil5ore` | 9 | 67×77 | −18.0% |

  These are route-ready placements, not yet emitted factories. The result is
  especially strong on the deep mixed fixtures: it replaces kilometre-wide
  rows with compact 2D production geometry while also shortening the
  rate-weighted commodity spans. The next phase must materialise those spans
  as local flow-sized manifolds and refuse any candidate that cannot route.

- **2026-07-26 — Local manifold capacity and hierarchy planning.** Each
  commodity now receives the minimum number of express capacity lanes from
  its exact solver rate (28 total on USP, 14 chemistry, 30 processing units,
  14 military science). Producer and consumer terminals are distributed
  evenly across those lanes. Each lane is reduced through a ragged binary
  `(2,1)` merger tree and expanded through an exact-leaf `(1,2)` distributor
  tree, avoiding the unavailable giant `(n,1)` / `(1,m)` templates.

  All producer and consumer hierarchies for all four representative fixtures
  decompose entirely into stampable library primitives. Hubs are placed near
  their terminal medians and reserve collision-free rectangles against both
  production islands and previously placed hubs. Rendering must now stamp
  these nodes and route their explicit inter-stage/terminal edges.
