# Cell-interface DB — Phase-0 scoreboard (2026-08-10)

**Status: measurement note** (docs-taxonomy: Notes — feeds the future
cell-interface RFC; archive once the RFC absorbs it). Every number below is
printed by a checked-in probe; nothing is hand-derived. Probes:
`crates/core/examples/probe_motif_census.rs`, `probe_motif_cost.rs`
(shared corpus in `examples/celldb/corpus.rs`),
`scripts/celldb-phase0/mine-community.sh` + `summarize-community.sh`.

Question under test: is a searchable DB of cached subtree implementations
(keyed by demand motif, constraints as derived entry metadata) worth
building? Four probes, four answers, all GO — with one regime boundary.

## 1. Demand concentration (census, 29 deduped solver runs)

Top 5 unit motifs carry **87.7%** of machine mass; top 12 carry 97.5%:
copper-plate 27.0% · iron-plate 18.5% · advanced-circuit/am2 17.4% ·
copper-cable/am2 16.0% · electronic-circuit/am2 8.9%. Smelting alone is
45.5%. Edge motifs concentrate identically (copper-cable→electronic-circuit
in 15/29 solves). **A DB with a dozen good entries covers the corpus.**

## 2. Cost baseline (layout sweep, 29/29 built)

Interior tiles attributed by segment id (`row:*`, `di-row:*`):

| motif | interior tiles | machines | tiles/machine |
|---|---:|---:|---:|
| copper-plate | 12,692 | 745 | 17.0 |
| advanced-circuit | 12,549 | 500 | 25.1 |
| iron-plate | 8,707 | 511 | 17.0 |
| copper-cable | 8,390 | 489 | 17.2 |
| electronic-circuit | 6,437 | 275 | 23.4 |

Top 5 ≈ 89% of attributed interior area — the scoreboard mirrors the census.

## 3. Fabric share (the pre-registered RFC-057 kill criterion)

Fabric = `trunk/tap/ghost/balancer` segments **plus segmentless stamped
transport** (balancer stamps carry `segment_id: None` by construction —
`balancer_library.rs:84`; classification left zero unexplained tiles).

Share of interior+fabric area: **median 17.8%** (min 1.8%, max 55.1%).
By target rate: <5/s → 18.3% · 5–20/s → 11.4% · **≥20/s → 29.3% median,
55.1% max** (the >50% cases are ec@35/40 on yellow — belt-saturated,
balancer-heavy).

**Verdict: the kill criterion does not fire.** Interiors dominate at low/mid
rates, so cached-interior wins are real there; at high rate on low belt
tiers, fabric approaches/passes parity, so that regime's lever is the
fabric-motif library (balancer-library generalization), not neater
interiors. Consistent with the 2026-07-24 strategy call ("bus stays
low-rate winner; high rates via composition").

## 4. Community mining (169 files → 6,087 records)

Independent demand signal agrees at the head (iron-plate #1,
advanced-circuit #2, copper-cable #3, copper-plate #5, electronic-circuit
#7) and is flatter overall (top-20 = 60.8% of 122,173 machine mass, 317
recipes) — quantifying the test-suite bias (community top-20 adds science
packs, engine units, LDS; our corpus has no science-pack fixtures).

Donor pools track demand: iron-plate **157** single-recipe arrays
(**46 engine-legal**), advanced-circuit 50 (10), processing-unit 46 (13),
plastic-bar 32 (10). Green-circuit/cable donors are scarce *as single-recipe
arrays* because the community fuses cable→circuit into one block —
independent confirmation that the two-recipe motif (what DI cells model) is
the right cacheable unit there.

Density reality check (legal donors, median bbox area/machine — the
`legal a/m` column `summarize-community.sh` prints, so this figure is
script-derivable, not a hand cut): iron-plate/electric-furnace 19.2 (n=7)
vs our 17.0 interior-only; advanced-circuit/am2 25.5 (n=5) vs our 25.1
interior-only. **The engine's per-machine density is already
community-ballpark, at parity on red circuits** — the DB's expected win is
composition, aspect control and the tails, not raw density. (Community
bbox area includes their internal belts; ours excludes fabric — both
definitions stated here on purpose.)

**Corpus drift caveat:** the shared probe corpus is a frozen snapshot of
`survey_fixtures()` at cd78eed7, deliberately decoupled from the live e2e
table. The probes let a reader re-derive every figure above, but they do
NOT detect the survey table growing past the snapshot — re-snapshot when
the RFC absorbs this note.

## Schema decisions banked (argued 2026-08-10, probe headers carry them)

- Identity = demand motif `(recipe, machine, count)`; edge motifs for fused
  pairs. **Counts stored, rates derived** by the current solver — survives
  the in-flight productivity work.
- Constraints are entry metadata, not key axes; tech is **derived from
  entity vocabulary**, never declared. Lookup = dominance filter.

## Next

Write the cell-interface RFC with this scoreboard as its motivation:
port-contract spec, store format (in-repo JSON, balancer-library pattern),
preview mode (interface-first boxes, lazy interiors), template candidates
under the existing never-worse + sim-anchor gates for the top motifs, and a
fabric-motif track for the high-rate regime. Declared gaps for the RFC to
own: no science-pack fixtures in the demand corpus; community donors not
yet port-inferred; cost attribution excludes taps serving two rows
(counted as fabric).
