# Snake-fold followups

**Status (2026-07-30):** **multi-fold works and is Factorio-verified.**
`chain-mil5ore` folds three times — 521×31 (16.8:1) → **147×138 (1.07:1)**,
PASS at 4.92/s against 5.00/s planned, 146/146 machines, one pole network,
4620/4620 entities revived. Single fold remains verified for throughput and
power. Read "The second trap" below anyway: it is why the earlier power
fragmentation went unnoticed, and the reasoning generalises.

Scope, because it is easy to over-read: **one fixture of four.**
`mega-chain-chem5raw`, `mega-chain-pu4raw` and `mega-chain-usp2raw` still find
no fold. And this is a SHAPE result, not a density one — it costs +21%
entities, and RFC-057's refusal of folding as a density lever stands.

**#492's stated blocker was not the blocker.** It recorded a shared-column B12
underground dive as the remaining work. No dive was written. Three other causes
were found by instrumenting the refusal: rows ignoring which side an input
arrives from, a lane terminus that never turned toward the input it feeds, and
lanes keyed by item rather than (item, side). The sections below are kept with
their original measurements, annotated where a claim has since been falsified —
the pattern of *what looked like the blocker and wasn't* is the reusable part.
Owning design doc:
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

### 1. Multi-fold: one crossing left (an INPUT lane at a shared column)

Branch: `feat/multifold-gap-lanes`. `InputStranded` is **resolved**; what is
left is a single, well-specified crossing.

**Built.** A gap now carries one lane per distinct ITEM rather than one lane
per side, which is what capped every layout at one item each way:

- gap height is sized from lane demand instead of the constant 2;
- both exits and input feeds get per-item lanes, with boundary records
  following their relocated terminus — the same discipline that a stale
  `boundary_outputs` record taught by producing 0.00/s;
- lane assignment is ordered by column, which makes the descent from a source
  row to a non-adjacent lane provably free. A lane spans `[edge, x]`, so
  ascending order means an earlier lane never covers the column a later one
  descends through. Verified offline before implementing: ascending gives 0
  blocked descents over a 5-exit sample, descending 10, arbitrary 5. The input
  side inverts — it fills from the far side and climbs the other way, so the
  deepest row needs the largest column.

`InputStranded` no longer fires on **candidates that clear the gap-lane pass**.
That scoping matters: `fold_snake` returns on a `GapLaneConflict` some 200 lines
before the stranding check runs, so a candidate that dies on a lane clash is
never measured. The stranding-prone candidates may be exactly the ones dying
earlier — this does not establish anything about them.

What it does establish, instrumenting every evaluation across the fold ladder:
504 input checks, of which **82 are interior** (`on_edge=false`) and **all 82
are `supplied=true`** — the same 82. So among candidates that get that far, the
interior branch still fires and passes because the feed pass genuinely supplies
them, not because the test stopped discriminating. That was worth checking,
because "the check went quiet" is not evidence on its own, and the retained note
in item 3 predicted adaptive gap sizing would invalidate this very edge test.
`SPAGHETTIO_FOLD_DEBUG=1` reproduces it.

Re-measure once the shared-column crossing is fixed and lane clashes stop
refusing candidates — that is when the excluded population finally reaches the
check.

**The remaining blocker**, measured rather than assumed:

```
input lane clash: lane=2 row=71 at (34,71) item=iron-ore span=[34,210]
occupied by express-transport-belt dir=North carries=coal
```

The occupied tile is another lane's **climb**, and both are at column 34 — two
different items whose input belts share a column, one feeding the segment above
the gap and one below. The ordering argument holds only for lanes at *distinct*
columns; at a shared column one item's climb must cross the other's lane, and
no assignment order avoids it.

**Fix shape (SUPERSEDED 2026-07-30):** this called for an underground dive
where two lanes share a column (`factorio-mechanics.md` B12). No dive was
needed. The clash above was **cross-side** — the two items' sources are on
opposite sides of the gap (note the excerpt's own `dir=North` occupant against
a lane fed from above), and row assignment ignored the side entirely, so climbs
from opposite sides swept through each other's rows. Measured on mil5: 28 of 32
gap-lane refusals were cross-side, 4 same-side. Partitioning rows by source
side took `GapLaneConflict` to 0 on mil5, pu4raw and usp2raw.

The lesson worth keeping: the excerpt contains the evidence that would have
falsified the shared-column reading — `carries=coal` facing North against an
iron-ore lane — and it was read as confirming it instead.

Note the clash surfaces under the `GapLaneConflict` refusal even though it is
an *input*-side collision: the input feed pass reuses that variant. A misnomer
that sends the reader to the wrong pass — renamed to `GapLaneConflict` on the
work branch. Reward: 3-fold reaches roughly 152×132 (1.15:1) on
mil5.

**Do not** retry a lane allocation that ignores the crossing. One was tried
before the ordering rule existed, regressed the verified single fold, and was
reverted.

### 2. Three of four corpus fixtures find no fold

Superseded in part by (1), which addresses both refusal classes with per-item
lanes. Re-run `probe_fold_corpus` on the branch before trusting the table
below, which predates that work.

`search_snake_fold` reports legal-column count, refusals by cause, and
validation rejections for the not-found path — read those before theorising.

Measured across the corpus (`probe_fold_corpus`, before per-item lanes):

| fixture | legal cols | GapLane | InputStranded | JunctionBlocked | rejected-by-validation |
|---|---:|---:|---:|---:|---:|
| `chain-mil5ore` | 251/551 | 21 | 121 | 29 | 9 → folds |
| `mega-chain-chem5raw` | 275/699 | 22 | 132 | 15 | 0 |
| `mega-chain-pu4raw` | 1052/2380 | **173** | **0** | 23 | 0 |
| `mega-chain-usp2raw` | 888/1938 | 46 | 107 | 43 | 0 |

Two things fall out and still hold. Legal columns are 40–50% everywhere, so
column legality is **never** the binding constraint — the pipe-adjacency
hypothesis for chem was wrong. And `pu4raw` records **zero** `InputStranded`,
failing on `GapLaneConflict` (173) instead — the only fixture where fixing the
input side could not help at all, which is why the exit side was tackled first.
(It still records 23 `JunctionBlocked`, so lane conflict is not its sole
refusal — just its dominant one, and the only one input work could not touch.)

By raw count `InputStranded` is the larger refusal on three of the four
fixtures (121 vs 21, 132 vs 22, 107 vs 46); only `pu4raw` inverts that. "Fix
the exits first" was a judgement about `pu4raw` being unblockable any other
way, not a claim that exits dominated the corpus.

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
  by coincidence of the current junction-column scheme. This predicted that
  adaptive gap sizing (item 1) would invalidate that coincidence. **Tested when
  that landed: it did not** — 82 interior inputs are still detected as interior,
  every one supplied. Gap height is vertical; the edge test's fragility is
  horizontal (junction columns), so adaptive sizing does not touch it. The
  looseness itself remains a latent risk: re-run that measurement if
  junction-column *placement* ever changes.

### 4. Lower-confidence items

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
`GapLaneConflict`, `CornerNotATurn`, `RunSevered`, `InputStranded`,
`EntityExplosion`. `SPAGHETTIO_FOLD_DEBUG=1` lists every severed belt instead
of just the first.

Two unbounded reconnection loops once allocated ~20 GB each until the OOM
killer fired; being a global OOM it killed unrelated processes including the
editor session, four times over. Both are bounded ranges now plus an
entity-count backstop. Long probes are worth running under
`systemd-run --user --scope -p MemoryMax=8G` regardless.
