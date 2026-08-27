# RFC-073: Margin in the inserter ladder

Registry: [`rfcs.md`](rfcs.md). Status: **Concluded at Phase 0,
2026-08-27 — K73-1 tripped on the census; the margin was never written.**
The instrument (Phase 0) is retained under the contract in the decision
log. Owner of RFC-072's Phase-2 residual (a) and of the "RFC-049 Phase 3"
pointer that RFC-072's decision log uses for the general fix (RFC-049
itself is Complete — its Phase 3 was the level-aware ladder; the margin
was never in its scope). What the census found instead — that the
ec@240 grid failures are a serial-single-hand pickup effect the ladder's
credit does not describe — is recorded below as the follow-up.

## Summary

The row templates size every machine side with the cheapest plan whose
credited capacity covers the planned rate — `required <= n·rate`, no
margin (`bus/inserter_ladder.rs`). A side can therefore ship with one
hand at exactly 100% of what the ladder credits it, and the ec@240
receipts (RFC-072 P2 unit 2) measured what that costs in the sim: one
machine short per copy with the far hand at 92.6% of its credit, two
short at 100%, and the same cell at plan with two hands at 52%. RFC-072
bought the exemplar back with a margin in the GRID quantizer only (its
`HAND_MARGIN = 0.85`, scoped to K > K_MAX because the general form
re-shaped a receipted strip). This RFC puts the margin where the sizing
happens — the ladder — so every row, native or composed, plans its input
hands with headroom, and retires the quantizer's stand-in. It is a
measured campaign, not a constant: Phase 0 instruments the ladder and
takes a census of how full the shipped hands are across both receipted
corpora (the sim registry and the calibration bank), joined to their
measured verdicts; Phase 1 sets the margin from that evidence, lists
exactly which fixtures re-shape, and re-measures each before its receipt
is re-blessed.

## Motivation

Reproducible today, three ways:

1. **The grid receipts** (RFC-072 decision log, 2026-08-26). ec@240 at
   K=20 (12/s per copy): the EC row's far belt is iron, a 12/s copy's
   five machines at 96% draw iron at exactly 2.40/s, the ladder credits
   one long-handed hand 2.4/s at L2 → one hand at 100% → produced
   224.00/240 (−6.7%), two machines short per copy. K=18 (13.33/s):
   92.6% of one hand → one short per copy, −1.7%. K=24 (10/s): two hands
   at 52% → 240.00/240.00, all 1,200 machines working. The constituent
   ec@12 cell alone measures −7.1% — the deficit is the cell's, not the
   composition's.
2. **The registry's FAIL rows.** `electronic-circuit@15` in the L1 world
   produces 12.58/15 (83.9%), `@30` L1 26.5/30 (88.3%); the same ec15
   geometry at L2 produces 15.0. Phase 0's raw census of that cell
   (sized at geo_cap 0): the far iron side asks 2.5/s of two long-handed
   hands credited 1.2 each (a shortfall on the interior machines) and of
   ONE hand on the trimmed last-in-row machine (208% of credit); the
   cable input rides one fast hand at 97.4%.
3. **The native corpus.** Phase 0's bank census (table below): the
   fullest shipped input hands sit at 0.95–0.985 of credit on exactly
   the rows with standing residuals — `stress_electronic_circuit_40s`
   (0.962, produced 93.0%), `_35s` (0.947, 94.3%) — while rows at plan
   sit under 0.90.

The ladder's own comment says why the margin belongs there: its rate
table is the SAME one the validator reads, "so the fix and the check can
never disagree on what an inserter moves". A margin anywhere else (the
grid quantizer today) is a second opinion on the same number.

## Design

### Phase 0 — the instrument (landed with this RFC)

- `SidePlan` carries `capacity` — `count ×` the per-entity rate of the
  table the plan was sized against. The ladder is the single source of
  truth for what a hand moves; consumers read utilization as
  `required / capacity` instead of re-deriving the table (the grid's
  hand term already does).
- `TraceEvent::InserterSideSized { recipe, side_is_output, item,
  required, entity, count, capacity, machine_x, machine_y }` — emitted
  by the templates' per-side helper for EVERY sized side (the existing
  `InserterSideCapped` is its shortfall subset). Free when no collector
  or sink is active.
- `bus/sizing_census.rs` — `capture` (collector + sink, so abandoned
  pass-1 events are dropped the way a streaming consumer never sees
  them), `side_loads` (joins a native build's events onto the shipped
  layout's machines; the selection loop builds several candidates under
  one collector, and a key that saw two plans is reported as
  ambiguous, never silently resolved), `side_loads_unjoined` (composed
  layouts — cells are generated once in their own frame and cloned),
  `repriced(level)` (the registry runs one geometry in several declared
  worlds; the hand that starves is the hand at the world's rate),
  `summarize` (bands `<0.85 | 0.85–0.90 | 0.90–0.95 | 0.95–1.00 |
  shortfall` over input sides).
- Two ignored probes: `inserter_sizing_census_calibration_bank`
  (`tests/inserter_sizing_census.rs`) and
  `inserter_sizing_census_registry` (`tests/cell_composition.rs`).
- Coverage, exactly: every side the row templates size — including
  `quad_input_row`'s mirrored input3 pair, which sizes per slot and
  records itself as one side of two hands (#735 review found it
  missing from the first cut). The nine direct `size_side` calls in
  `placer.rs` (DI bridge, fused/straddle cells) emit nothing — the
  census's recorded gap, not fixed here. Hands that are not
  ladder-sized at all (the quad row's input1/input2 long-handed pair
  is hardcoded, one per belt) are not census subjects by definition. The event is built only under
  the census's own scope (`trace::with_sizing_census`, entered by
  `capture`) — NOT whenever a collector or sink is present, because
  the web's streaming solve installs both on every interactive layout
  and would otherwise build and serialize one event per machine side
  for nothing (#735 round 2). Ordinary traced builds and snapshots
  never contain it.

### Phase 1 — the margin (gated on K73-1)

`size_side` (machine-drop INPUT sides, the `machine_feed_rate` table)
gains `INPUT_HAND_MARGIN`: a plan covers when `required <=
n·rate·INPUT_HAND_MARGIN`. The shortfall a capped plan reports is
measured against the margined capacity (what the margined ladder could
not cover), so `InserterSideCapped`, `capped_limit`'s counterfactuals,
and the grid's `shortfall.is_some()` violation all stay consistent.
`capacity` stays the unmargined credit. Belt-drop sides
(`size_belt_drop_side`, `size_side_output`) are unchanged: their rate is
a sim-measured min-form with the lane cap already in it, and no receipt
implicates them.

The constant is set FROM the census, not by fiat: the largest input-side
utilization the census finds on a fixture the sim measures at plan is the
floor the margin must sit above (a margin below it would re-shape a
receipted-good side for nothing), and the smallest utilization on a
fixture the sim measures short — with no other standing explanation — is
the ceiling. 0.85 (the grid's value) is the prior.

Blast radius is a list, not a guess: every fixture with an input side in
a band at or above the margin re-shapes; each registered one is
re-measured before its registry hash is re-blessed (never re-bless on
the validator); each bank row that re-shapes is re-exported into a new
bank and re-measured, with the identical-blueprint rows carrying their
existing reports (the RFC-072 re-bless protocol).

### Phase 2 — retire the stand-in

With the margin in the ladder, `cells::chain::required_copies_at`'s
hand term reduces to its shortfall clause (a copy whose margined plan
the geometry cannot hold is still a K violation); the `HAND_MARGIN`
constant and the utilization comparison go. RFC-072 residual (a)
closes.

### Rejected alternatives

- *A per-entity margin* (long-handed only). The receipts implicate the
  long-handed far hand, but the census shows fast and regular near hands
  at 0.93–0.985 on the residual rows too; a single input-side margin is
  the hypothesis the census tests, and per-entity refinement is a
  Phase-1 decision the census can make.
- *Margin in the rate table* (`machine_feed_rate`). The table is
  measured; a margin is a planning policy, and the validator reads the
  table as ground truth.
- *Keep the grid-only margin.* It fixes the exemplar and leaves every
  sub-K_MAX strip and every native row at the brim — the d1 FAIL rows,
  ec60 at K=5, and the ec35/ec40 residuals.

## Kill criteria

- **K73-1 (the premise, adjudicated on Phase 0's census before any
  margin is written).** If a fixture the sim measures AT PLAN (registry
  PASS, or bank produced ≥ 98%) ships an input side at ≥ 0.95 of its
  credit, AND a fixture the sim measures SHORT ships no input side above
  0.90, then fullness does not predict deficit and the uniform margin
  is refuted: stop, record, leave the grid term as the scoped exception.
- **K73-2 (never worse by the sim).** Every re-shaped registry row and
  bank row is re-measured; any row whose produced rate drops more than
  2 points, or whose verdict worsens, at the chosen margin reverts the
  margin to the largest value at which no row regressed — or kills the
  RFC if that value is 1.0.
- **K73-3 (it pays).** At least one of the short class — ec15 L1
  (83.9%), ec30 L1 (88.3%), the ec@12 constituent (−7.1%),
  `stress_electronic_circuit_30s` (90.9%), `_60s_red` (90.7%) — must
  gain ≥ 3 points of produced under the margin. None moving means the
  ladder is not their mechanism; kill and record.
- **K73-4 (a margin, not a re-sizing).** Across the bank the inserter
  count rises ≤ 10% and no row's dimensions change: a margin adds hands
  into the free columns the templates already budget; if it widens rows,
  it is re-sizing under another name — stop and re-derive from the
  measured table instead.
- **K73-5 (one margin).** With the ladder margin in, `required_copies_at`
  for ec@240 at L2 must still yield K=24 with the quantizer's
  utilization clause deleted; a disagreement means the two margins are
  not the same policy — reconcile before deleting, never keep both.

## Verification plan

Per the CLAUDE.md layout-engine protocol. Phase 0: `cargo test` green
(1276+), registry gate `cell_registry_hashes_current` and the
calibration fingerprint probe unchanged (zero geometry change), clippy
clean; the census tables below reproduced from the two probes. Phase 1:
the census re-run under the margin gives the blast-radius list; each
listed registry row re-simmed (`export_chain_fixtures_for_sim` →
spaghettio-sim → registry re-bless with the receipt); each listed bank
row re-exported and re-measured (`calibration_matrix_export` →
`run-calibration-matrix.sh` on the changed rows → `calibration_evidence.py`);
K73-2..K73-4 read off those receipts; the copy-count pins and
`grid_composes_ec240_*` hold. Phase 2: K73-5 by the pins.

## Phasing

- **Phase 0 — instrument + census.** This RFC's PR. Zero behaviour
  change.
- **Phase 1 — the margin.** Opens only on K73-1 clean; ships with its
  re-measures.
- **Phase 2 — retire the grid term.** Same PR as Phase 1 if K73-5 holds
  on the first pass, else its own.

## Decision log

- *2026-08-27 — RFC opened; Phase 0 instrument landed.* Two instrument
  defects found and fixed while taking the census, recorded because
  they are the kind a later reader would re-discover: (1) the event
  first lacked the item, so a machine's near and far inputs collapsed
  into one key and every bank row read `sides_in == sides_out`; (2) the
  first probes read the trace COLLECTOR, which sees every candidate
  build and both layout passes, and joined by coordinates — which can
  never work for composed layouts (cells are generated once in their
  own frame and cloned/translated; the registry census read zero sides
  everywhere). The sink-based `capture` and the unjoined composed-form
  fix both. Census tables: below.
- *2026-08-27 — K73-1 TRIPPED on the Phase 0 census; concluded without
  writing the margin.* Both clauses hold — the first four times, the
  second once. **Fixtures the sim measures at plan ship input hands at
  ≥ 0.95 of credit** — `stress_electronic_circuit_30s_decomposed_partitioned`
  (20 sides at 0.974, the EC row's cable fast hand at 4.50/4.62;
  produced 99.4%), `stress_electronic_circuit_22s` (0.952; delivered
  99.4%), `_23s` (0.933; 100.0%), and the composed `chain-ec15` at L2,
  whose trimmed last-in-row long-handed iron hand sits at **1.042 of
  its credit (2.50 on 2.40) and produces 15.0/15** — while **a fixture
  the sim measures short ships no hand above 0.90**:
  `tier2_electronic_circuit_from_ore` (produced 93.3%, fullest input
  0.649). The other short rows (`stress_electronic_circuit_30s`,
  `_60s_red`, `_35s`, `_40s` at 90.7–94.3%) DO ship full hands
  (0.947–0.974) — but so does the at-plan partitioned twin of the
  first, with the same twenty hands, which is what makes fullness
  non-discriminating rather than merely non-necessary. Fullness on the
  ladder's credit is neither sufficient nor
  necessary for a deficit; a uniform margin at any value between 0.85
  and 0.97 would re-shape a dozen at-plan bank rows (every one with
  sides in the 0.85–0.97 bands) for no measured gain, and K73-4 would
  price it as a re-sizing. Stopped before Phase 1, as the criterion
  says. **What the census DOES say about the receipts:** the two grid
  failures (K=20: five EC machines per copy each on ONE far hand at
  2.40/2.40, two short; K=18: 92.6%, one short) and PU-from-ore (20
  iron sides at exactly 2.40/2.40, produced 87.7%, with other standing
  causes) are all *serial single long-handed hands on one belt at the
  credit*, whereas the at-plan 1.042 case is a single last machine
  behind five double-handed ones on a 15/s belt. The credited rate was
  measured on a full belt; a row of single hands each at the credit
  sees the belt the previous hands left. The mechanism to test is
  pickup-side (belt density / lane fill at the far hand), not the
  hand's swing rate — a sim experiment, not a ladder constant. Recorded
  in RFC-072's residual (a) as the replacement pointer.
  **Retention contract:** the Phase 0 instrument (`SidePlan::capacity`,
  `InserterSideSized`, `bus/sizing_census.rs`, the two probes) is
  retained as the standing sizing instrument — it is what the pickup
  follow-up reads, and `cells::chain` now reads `capacity` instead of
  re-deriving it. Flip condition: if that follow-up concludes without
  consuming the census, the event and the module go in its close-out
  (the `capacity` field stays; it is a simplification, not scaffolding).

### Phase 0 census — sim registry (composed cells, input sides re-priced at the declared level)

`inserter_sizing_census_registry`, 2026-08-27. Bands over INPUT sides:
`≤0.85 | 0.85–0.90 | 0.90–0.95 | 0.95–1.00 | shortfall` — every band
closed at its top edge, so a hand at exactly 0.85 is in the first and a
hand at exactly its credit (1.000) in `0.95–1.00`; `shortfall` is a
plan the ladder itself could not cover. Side counts are **per generated cell**
(one copy per spec — ec75 and ec150 read the same because they seed the
same per-copy cell), not per fixture. Verdicts from
`cell-sim-registry.json` (produced % of plan). The tables are the
survey the verdict was read from, dated; the instrument's end-to-end
behaviour on the decisive row is pinned by the non-ignored
`census_sees_the_ec15_cells_far_hand_at_the_credit`
(`tests/cell_composition.rs`) — a pinned-geometry gate on purpose: the
ec15 cell is frozen by the registry hash, so its 2.5/s far side, its
two-hand interior and its one-hand last machine cannot move without a
re-bless, and a recalibration of `machine_feed_rate` should fail it
loudly. The rest of the tables are not gated.

| fixture | level | sides in/out | bands | fullest input | sim |
|---|---|---|---|---|---|
| chain-ac1 | 0 | 36/14 | 36·0·0·0·0 | 0.833 EC iron 2×LHI 2.00/2.40 | PASS 100% |
| chain-ec15 | 1 | 17/11 | 6·0·0·5·6 | **2.083** EC iron 1×LHI 2.50/1.20 (last-in-row) | **FAIL 83.9%** |
| chain-ec15 | 2 | 17/11 | 15·0·0·0·2 | **1.042** EC iron 1×LHI 2.50/2.40 (last-in-row) | WARN, produced **100%** |
| chain-ec15 | 7 | 17/11 | 17·0·0·0·0 | 0.521 | WARN, produced 100% |
| chain-ec30 | 1 | 22/14 | 14·0·0·0·8 | 2.083 (as ec15) | **FAIL 88.3%** |
| chain-ec15g2 | 2 | 17/11 | 17·0·0·0·0 | 0.521 EC iron 2×LHI 2.50/4.80 | WARN, produced 100% |
| chain-ec75 | 2 | 78/68 | 78·0·0·0·0 | 0.521 | PASS 100% |
| chain-ec150 | 2 | 78/68 | 78·0·0·0·0 | 0.521 | PASS 100% |
| chain-ec240 | 2 | 62/54 | 62·0·0·0·0 | 0.541 cable copper 1×fast 2.50/4.62 | PASS 100% |
| chain-gear20 | 2 | 8/8 | 8·0·0·0·0 | 0.260 | PASS 100% |
| chain-mil5ore | 0 / 2 / 7 | 162/94 | all <0.85 | 0.744 / 0.372 / 0.195 | PASS 100% |
| chain-mil5plates | 0 / 2 | 224/88 | all <0.85 | 0.676 / 0.338 | PASS 96.6% |

The ec15 L1 FAIL and the ec15 L2 at-plan rows are the SAME geometry
(hash `8f2473ec…`): what changes between them is the world's hand rate,
and the L2 world produces at plan with one hand 4% over its credit.
(The ec15 cell is six EC machines per copy — five interior with two far
hands each, one trimmed last-in-row with one — which is why its rows
read 17 input sides: 6 iron + 6 cable + 5 cable-row inputs.)

### Phase 0 census — calibration bank (native builds)

`inserter_sizing_census_calibration_bank`, 2026-08-27, joined to
`docs/selection-policy-calibration-evidence.md` (produced % of plan;
`ambiguous` was 0 on every row; same band semantics as above — PU's
twenty `2.40/2.40` sides are AT the credit, band `0.95–1.00`, not
shortfalls). Rows with no sized side (fluid-only chains, the
cell-composed `iron_gear_wheel_20s`, `ac_45s`) omitted. Taken before
the quad row's input3 was instrumented; no bank fixture uses that
template's third input at a rate near its credit, so the rows stand.

| fixture | sides in | bands | fullest input | produced |
|---|---|---|---|---|
| tier1_iron_gear_wheel | 10 | 10·0·0·0·0 | 0.433 | 100.0 |
| tier1_iron_gear_wheel_from_ore | 39 | 39·0·0·0·0 | 0.618 | 100.3 |
| tier2_electronic_circuit | 24 | 7·10·7·0·0 | 0.928 EC cable 1×fast 4.29/4.62 | 100.0 |
| tier2_electronic_circuit_from_ore | 75 | 75·0·0·0·0 | **0.649** | **93.3** |
| tier2_electronic_circuit_20s_from_ore | 128 | 94·20·14·0·0 | 0.928 | 100.0 |
| tier3_plastic_bar_from_crude | 5 | 5·0·0·0·0 | 0.595 | 100.0 |
| tier4_advanced_circuit_from_plates | 33 | 33·0·0·0·0 | 0.744 | 100.3 |
| tier4_advanced_circuit_partitioned | 33 | 31·2·0·0·0 | 0.893 | 100.7 |
| tier4_advanced_circuit_from_ore_am2 | 212 | 188·17·7·0·0 | 0.928 | 98.1 |
| tier5_processing_unit_from_ore_am3 | 376 | 356·0·0·20·0 | **1.000** EC iron 1×LHI 2.40/2.40 (×20) | **87.7** |
| tier_uranium_processing_voider | 88 | 87·0·0·1·0 | 0.985 U-238 1×LHI 2.36/2.40 | non-converged |
| stress_electronic_circuit_30s_from_ore | 190 | 140·30·0·20·0 | **0.974** EC cable 1×fast 4.50/4.62 | **90.9** |
| stress_advanced_circuit_partitioned_5s (pooled / partitioned) | 156 | 132·17·7·0·0 | 0.928 | 101.7 / 100.0 |
| stress_advanced_circuit_partitioned_4s (pooled / partitioned) | 126 | 106·20·0·0·0 / 112·14·0·0·0 | 0.866 / 0.893 | 100.3 / 100.3 |
| stress_electronic_circuit_30s_decomposed_pooled | 190 | 140·30·0·20·0 | 0.974 | 90.9 |
| stress_electronic_circuit_30s_decomposed_partitioned | 190 | 140·30·0·20·0 | **0.974** (same sides) | **99.4** |
| stress_electronic_circuit_60s_red_from_ore | 380 | 280·60·0·40·0 | 0.974 | 90.7 |
| stress_electronic_circuit_22s_from_ore | 141 | 104·22·0·15·0 | 0.952 | 95.5 (delivered 99.4) |
| stress_electronic_circuit_23s_from_ore | 148 | 109·23·16·0·0 | 0.933 | 100.0 |
| stress_electronic_circuit_35s_from_ore | 223 | 164·35·24·0·0 | 0.947 | 94.3 |
| stress_electronic_circuit_40s_from_ore | 254 | 187·40·0·27·0 | 0.962 | 93.0 |

The pooled and partitioned ec30 rows ship the SAME twenty 0.974 cable
hands and measure 90.9% vs 99.4% — the deficit is in what differs
between them (the trunk/tap provisioning RFC-069 owns), not in the
hands they share.
