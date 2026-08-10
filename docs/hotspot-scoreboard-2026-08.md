# Layout hotspot scoreboard — 2026-08

**Status:** measurement record, complete. One probe, one clean run, all
numbers below from that run. This doc is the citable output of
[`probe_hotspots.rs`](../crates/core/examples/probe_hotspots.rs); if the
engine or corpus moves, re-run the probe and update this file in the same
PR.

```bash
cargo run --manifest-path crates/core/Cargo.toml --release --example probe_hotspots
```

Corpus: the celldb Phase-0 demand corpus (`crates/core/examples/celldb/corpus.rs`,
include!d — cannot drift from the census/cost probes). Engine at
`16cb2926` (main, 2026-08-10). 29 layouts built, 0 failed, 0 tiles
unclassified; every class total reconciles against the occupied-tile
union by assertion in the probe.

## The question

RFC-067 established that the engine ties itself at matched demand
(K67-3), so the cell-interface DB only pays if it holds cells the engine
cannot produce. Which cells are worth acquiring first — and is cell
quality even the dominant area sink? This probe splits every bounding-box
tile of every corpus layout into exactly one class and ranks the sinks.

## Pooled area budget

| class | tiles | share of bbox |
|---|---:|---:|
| machines (crafting footprints) | 25,812 | 9.7% |
| interior overhead (row belts/inserters/pipes) | 28,235 | 10.6% |
| fabric (trunks, taps, ghost routes, balancers, …) | 23,320 | 8.8% |
| infra (poles) | 3,093 | 1.2% |
| **whitespace** | **185,993** | **69.8%** |

266,453 bbox tiles over 29 layouts; whitespace share per layout: median
64.3%. **Empty space dwarfs every occupied class combined.**

## Where the whitespace is

Scanline decomposition (three kinds because they imply three different
fixes): fully-empty stripes **0.0%**, ragged edge (outside a scanline's
occupied span) **79.5%**, in-span holes **20.5%** — as shares of
whitespace. In bbox terms: ragged ≈ 55.5% of ALL bounding-box area, holes
≈ 14.3%.

Zero stripes is structural: trunks run vertically, so nearly every
scanline hits at least one belt. The ragged edge is the bounding box being
set by the widest row (typically a smelter row) while the rest of the
layout runs at roughly half that width — visible directly with
`HOTSPOT_PROFILE=electronic-circuit@40` (two full-width smelter bands;
~150 subsequent scanlines spanning ~60 of 118 tiles).

This replicates RFC-058's motivation measurements on a different corpus
with a different instrument: RFC-058 measured 61–65% void with
ragged-right at 38.4% of bbox across ten hand-picked bus-path fixtures;
this probe measures 69.8% void with ragged at ~55.5% of bbox across the
29-layout everyday-demand corpus. (Decomposition definitions differ —
RFC-058 split void into ragged-right rectangles + interior corridors;
this probe splits by scanline — so the sub-numbers are not directly
comparable. The structural conclusion is identical.)

## Prior adjudications — read before proposing a fix

The void is real, replicated, and **every funded mechanism against it has
died at its own pre-registered gate**. The record, so nobody walks this
loop again:

| mechanism | RFC | outcome |
|---|---|---|
| per-machine repacking | RFC-057 | +38–250% bbox — transport uneconomic; parked |
| band packing, post-hoc re-route, area objective | RFC-058 | KILLED: −27.0% real vs −33% bar |
| same mechanism, aspect/transit objective | RFC-064 P3 | gate FAILS: admissible on 2/4 fixtures, transit sharply worse |
| wide-row splitting | RFC-058 decision log 2026-07-30 | falsified: −0.9% bbox at zero errors; out of scope, not deferred |
| folding | RFC-057 | area-negative (17.7k→21.6k bbox on the multi-fold); value is shape only |

The consistent failure mode is transport: bands are cheap to relocate on
paper and expensive to re-route legally. Until someone brings a mechanism
that survives routing admissibility, the ragged void is the measured
price of the bus architecture's routability, not reclaimable area.

## Motif prize table — the RFC-067 donor target list

Overhead = attributed interior tiles − machine footprint tiles. **Upper
bound**: a real cell still needs belts and inserters; how much is
actually reclaimable is the donor's job to prove under never-worse + sim
gates (`candidate_runner`). Top of the pooled table:

| motif | tiles | overhead | ovh/machine | layouts |
|---|---:|---:|---:|---:|
| advanced-circuit | 12,549 | 8,049 | 16.1 | 6 |
| copper-plate | 12,692 | 5,987 | 8.0 | 10 |
| iron-plate | 8,707 | 4,108 | 8.0 | 11 |
| copper-cable | 8,390 | 3,989 | 8.2 | 15 |
| electronic-circuit | 5,581 | 3,106 | 11.3 | 15 |

`advanced-circuit` is the first donor worth acquiring: the largest
absolute prize and double the per-machine overhead of the smelters. Full
table (19 motifs) in the probe output. Total pooled cell overhead is
28,235 tiles — about a fifth of the ragged void, which calibrates how
much a perfect donor program can move the headline number.

## Fabric by kind

Trunk 44.0%, ghost routes 25.8%, crossings 11.6%, mergers 9.1%,
balancers 5.7%, row-trunk 3.7%, taps/feeds/stamps <0.1% each — of a
fabric total that is only 8.8% of bbox. No fabric class is a first-order
area target on this corpus.

## What this changes

1. **Donor ingestion (RFC-067 reopening path) now has a ranked target
   list** — advanced-circuit, then the smelter pair. That path was
   already the recorded reopening condition; this probe prices it.
2. **Nothing here reopens the packing arc.** The probe's void numbers
   replicate what RFC-058 already measured; they do not invalidate any
   kill. A sixth attempt needs a mechanism that addresses the routing
   admissibility failure specifically, plus owner sign-off on the
   objective (RFC-064 provenance: bbox-area minimization is contested).
3. **Whitespace is now measured and decomposed** on the standing corpus,
   which no other instrument reports. If a future change claims density
   wins, this probe is the cheap first check.
