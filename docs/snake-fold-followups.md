# Snake-fold followups

**Status (2026-07-29):** single fold is verified on `chain-mil5ore` for
throughput **and** power — 5.00/s, 146/146 machines, and one connected pole
network, against a control that measures the same. The power fragmentation
that previously blocked it is fixed. Read "The second trap" below anyway: it
is why the fragmentation went unnoticed, and the reasoning generalises. Multi-fold and three of the four corpus fixtures do not
fold; the causes below are diagnosed, not guessed. Owning design doc:
[`rfc-057-topology-preserving-dense-repacking.md`](rfc-057-topology-preserving-dense-repacking.md),
whose decision log carries the measurements.

Folding is a **shape** transform, not a density one — RFC-057 refused it as a
density lever on a measured ~20% routing ceiling. Its value is turning a
kilometre-wide ribbon into something a human can place and inspect.

## The second trap: a green sim proves nothing about power

The harness creates one electric energy interface **per pole network**, so it
energises every disconnected island it finds and reports every machine working.
The mil5 fold measured 146/146 machines and PASS while splitting the factory
into two power islands a player would paste as two dead halves.

Between this and the boundary-record trap below, the rule for this transform
is: **validator-clean and sim-green are each necessary and neither is
sufficient.** Check the specific invariant you care about.

## What works

Throughput only — see the status line. `chain-mil5ore`, compacted, single fold
at the midpoint:

| | geometry | aspect | produced | delivered | machines | verdict |
|---|---|---|---|---:|---:|---:|
| control | 552×32 | 17.25:1 | 5.00/s | 5.00/s | 146 | PASS |
| folded | 276×66 | 4.18:1 | 5.00/s | 5.10/s | 146 | PASS |

Factorio 2.0.77 headless, `--warmup 216000`, both `converged=true`. Cost is
277 entities (2820 → 3097), all of it reconnection geometry.

Entry point is `search_snake_fold`, which admits a candidate only when it
validates no worse than its source. Prefer it to calling `fold_snake`
directly — geometric legality is not sufficient (see below).

## The trap: validation gives false passes here

An earlier fold validated at **exact control parity** and produced 0.00/s in
Factorio, with 110 machines showing `full_output` and rates decaying
1.80 → 2.05 → 1.22 → 0.10 → 0.00.

`chain-mil5ore` emits science on its bottom edge facing south. Folding puts
that edge against an inter-segment gap, so the exit belt is rerouted along a
gap lane to the bounding box — but the `boundary_outputs` *record* stayed on
the machine's folded tile. Output went to a tile nothing drained.

Validation checks geometry and never asks whether a boundary record still
describes where output arrives. **Any transform that relocates a boundary must
move its record, and a geometry-only gate cannot certify one.** Sim before
believing a fold.

## Open work

### 1. Multi-fold: boundary inputs strand (`InputStranded`)

Inputs are fed from outside the bounding box, so they only work on an edge.
Segment parity decides where they land:

- an unrotated segment keeps inputs on its **top** row;
- a rotated segment carries them to its **bottom** row.

With one fold both sets stay on the layout edge — segment 0's at the top,
segment 1's rotated to the bottom — which is exactly why one fold works. From
two folds up they land on interior gap rows.

Because consecutive segments have opposite parity, each gap carries one
segment's exits and the neighbour's inputs. Supplying the inputs needs one
lane per item along a gap that already carries exits — roughly 8 lanes in 2
rows for mil5's 3-fold.

**Fix shape:** size gaps adaptively (`gap` is currently the constant 2) from
the lane demand per gap, and route each item its own row. Needs per-segment Y
offsets rather than the current `seg * (h + gap)`, threaded through the
transform, junction and exit passes. Costs vertical space, which is
acceptable — shape is the goal, not density.

Reward: 3-fold reaches 152×132 (1.15:1) on mil5.

### 2. Three of four corpus fixtures find no fold

`mega-chain-chem5raw`, `mega-chain-pu4raw` and `mega-chain-usp2raw` all yield
nothing. `search_snake_fold` reports legal-column count, refusals by cause,
and validation rejections for the not-found path — read those before
theorising.

Measured across the corpus (`probe_fold_corpus`):

| fixture | legal cols | ExitLane | InputStranded | JunctionBlocked | rejected-by-validation |
|---|---:|---:|---:|---:|---:|
| `chain-mil5ore` | 251/551 | 21 | 121 | 29 | 9 → folds |
| `mega-chain-chem5raw` | 275/699 | 22 | 132 | 15 | 0 |
| `mega-chain-pu4raw` | 1052/2380 | **173** | **0** | 23 | 0 |
| `mega-chain-usp2raw` | 888/1938 | 46 | 107 | 43 | 0 |

Two things fall out. Legal columns are 40–50% everywhere, so column legality
is **never** the binding constraint and the pipe-adjacency hypothesis for
chem was wrong. And `pu4raw` records **zero** `InputStranded` — it fails on
`ExitLaneConflict` alone. Since a single fold cannot strand an input, every
fixture's one-fold candidates die the same way: several *different* items
leave on the bottom edge, and a gap carries one lane per side.

**`ExitLaneConflict` is therefore the highest-value fix in this backlog.** It
alone blocks `pu4raw` completely, and it is a prerequisite for the other two.

**Fix shape, shared with (1):** one lane per distinct item, gap height sized
to fit. Sizing the gap is easy and was tried; the hard part is that a second
item on the *same* side cannot reach its lane without crossing the first
lane's belts — the exit sits adjacent to lane 0 only. That is channel routing:
each additional item needs to travel out past the lanes' extent, jog to its
row, and come back. An attempt that allocated lanes without solving the
crossing regressed the verified single fold and was reverted; do not repeat it
without the jog.

### 3. ~~Power network fragments across the fold~~ — DONE (2026-07-29)

Fixed and Factorio-verified; kept here for the diagnosis, which generalises.

Three separate causes, none sufficient alone:

1. **The chain composer's spanning pole line** stepped by 8 on an absolute grid
   against wire reach 9, and nudged forward past congestion without
   compensating — orphaning the pole behind it. Now steps from the last pole
   actually placed.
2. **The fold re-placed poles from scratch.** A fold is a rigid motion per
   segment, so poles keep their relative positions inside a segment and only
   the seams break. Keeping the transformed poles and bridging the seams took
   `chain-mil5ore` from 89 unreachable poles to 0.
3. **`repair_pole_connectivity` seeded its bridge scan at the MIDPOINT** of the
   two nearest components. That only works while the gap is at most
   `2 * reach`; beyond it the midpoint is further from either endpoint than a
   pole there could wire to, and the scan fails at any radius. Measured on
   `usp2raw`: closest pair 40.3 tiles, reach 9, midpoint 20 from each end, gave
   up with 11 components. Now falls back to stepping one reach-length from an
   endpoint — as a FALLBACK, tried only when the midpoint scan finds nothing,
   so every layout the old seeding handled keeps byte-identical geometry.
   Seeding from the endpoint unconditionally also works but moves three
   sim-verified registry pins to fix one fixture.

Composed pole networks, all four fixtures: **1**.

<!-- superseded detail retained below for the record -->

`replace_poles` places poles well but the two folded segments end up as
separate networks: 89 of 174 poles unreachable from the first. The pipeline's
own `repair_pole_connectivity` is now called (it was being skipped) and adds
**zero** bridges — a compacted layout has had exactly the free tiles a bridge
pole needs removed. Options, none tried: reserve bridge columns during folding,
allow the repair to displace a belt tile, or run the fold before compaction
rather than after.

Adversarial review also raised two it could not exercise:

- `replace_poles` passes an empty substation-target list, while
  `build_bus_layout` computes real ones for deep interior geometry — precisely
  what densification produces. Not exercised by mil5, which has no substations.
- `InputStranded`'s edge test accepts any bounding-box edge, and is correct only
  by coincidence of the current junction-column scheme. Adaptive gap sizing
  (item 1) would invalidate that coincidence.

### 5. Lower-confidence items

- The continuity invariant (`RunSevered`) false-positived twice on splitter
  footprints — a 180° rotation swaps which physical tile the anchor names. It
  is footprint-aware in both directions now and the sim agrees with it, but it
  has earned suspicion.
- U-turn corners are checked to be single-feeder turns (both lanes, B11)
  rather than sideloads (one lane, B8). The check is pass/fail; if it starts
  refusing often on deeper fixtures, stagger junction columns per crossing
  rather than weakening it.
- `search_snake_fold` slides one comb of fold lines and snaps each to the
  nearest legal column. Independent per-column search would explore more, at
  combinatorial cost.

## Guard rails worth keeping

Every failure mode is a typed `FoldRefusal`, so a refusal names its cause
rather than being a bare `None`: `CutsEntity`, `JunctionBlocked`,
`ExitLaneConflict`, `CornerNotATurn`, `RunSevered`, `InputStranded`,
`EntityExplosion`. `SPAGHETTIO_FOLD_DEBUG=1` lists every severed belt instead
of just the first.

Two unbounded reconnection loops once allocated ~20 GB each until the OOM
killer fired; being a global OOM it killed unrelated processes including the
editor session, four times over. Both are bounded ranges now plus an
entity-count backstop. Long probes are worth running under
`systemd-run --user --scope -p MemoryMax=8G` regardless.
