# RFP: Power placement under face densification

## Summary

Poles are placed last (`place_poles`, after row templates) and live off leftover
tiles. Face-densification work — starting with the `beltspan-lastinrow` belt
extension, continuing with the face-allocation direction — consumes exactly
those tiles. A corpus census (2026-07-19; 45 snapshots, 2212 poles, 401 rows)
shows the risk is **not uniform**: solid rows have real headroom, fluid rows are
already at the edge, and power validation has a blind spot for 6 machine types.
This RFP proposes: **(0)** close the validator blind spot, **(1)** replace the
oil-refinery-only pole-gap special case with a principled fluid-row reservation,
**(2)** add a pole-slack guardrail metric to the stress scoreboard so future
densification changes surface their power cost in the golden diff, and **(3)**
defer substation/dedicated-power-row design behind an explicit trigger instead
of building it speculatively.

## Motivation (census evidence, 2026-07-19)

Census: full e2e/stress corpus + the 6 science-gauntlet packs, dumped snapshots,
ground-truth pole analysis (raw data `scripts/pole-census-2026-07-19.json`; scripts in
session scratchpad). Key numbers:

- **0 power-category warnings** across all 45 snapshots today — but see the
  blind spot below before trusting that.
- **82% of poles (1810/2212) sit inside their row's active inserter span** — in
  gaps that face-densification will close. Only 154 sit in genuinely spare
  margin. This is the core motivation: today's power placement is a squatter on
  the exact real estate we're about to develop.
- **Slack** (free alternative tiles within `place_poles`' own probe window, per
  real pole): solid rows median 8, **zero** poles at 0 alternatives (n=1881).
  Fluid rows median 4, **83/219 (38%) at zero alternatives**. Every zero-slack
  pole in the corpus is on a fluid-involving row. Worst cases:
  `tier4_advanced_circuit_partitioned`, `census_chemical_science_pack`,
  `census_production_science_pack`, `stress_advanced_circuit_partitioned_{4,5}s_from_plates`.
- **The fluid pole-gap reservation is a special case, not a rule**:
  `templates.rs` (~3125) gates the gap on `msz==5 && machine=="oil-refinery"`.
  Verified by tile dump: true 5×5 refinery rows get pipe/UG/**gap**/UG/pipe with
  poles landing in the gap (11/12 refineries in the basic-oil row); smaller
  single-fluid rows classified the same `RowKind::OilRefinery` (e.g. 3-wide
  chemical-plant lubricant rows) get **no** reservation and currently survive on
  incidentally-free trunk space. The corpus has no multi-machine example of that
  shape, so its safety under pressure is unproven. The reservation costs nothing
  in inserter terms (0 of 222 `InserterSideCapped` events correlate with fluid
  rows — fluid ports use pipes, not the ladder).
- **Validator blind spot**: `validate::power::MACHINE_ENTITIES` (power.rs:104)
  lists only the original 6 machine types; the layout side covers 12. `foundry`,
  `biochamber`, `centrifuge`, `recycler`, `cryogenic-plant`,
  `electromagnetic-plant` already appear in 8 corpus snapshots and their power
  coverage is **never checked**. "0 warnings" is unverified precisely on the
  fluid-heavy space-age rows where slack is worst.
- **Recorded caveat**: a byte-faithful replay of `place_poles`' greedy probe
  failed to reproduce real pole positions in 34/45 snapshots (multi-block rows;
  cause unknown after line-by-line source verification). The census therefore
  measures slack from ground-truth positions, not replayed decisions. Nothing
  should build on the replayed decision order until that divergence is
  explained.

## Design

- **Phase 0 — close the validator blind spot.** Make `validate/power.rs` consume
  the same canonical machine-entity list as the layout side (export one source
  of truth; regression test asserting the lists cannot drift). Re-run the
  corpus. Any newly revealed coverage failure becomes the RFP's first real fix
  and re-scopes Phase 1. Do this first because it changes what every later
  "0 power warnings" claim means.
- **Phase 1 — principled fluid-row reservation.** Replace the
  `msz==5 && oil-refinery` gate with a rule derived from the row's port
  geometry: when a fluid row's own free-tile budget within pole coverage
  intervals falls below what coverage needs, reserve gap tiles — for all fluid
  RowKinds, sized by machine footprint. Scope: fluid rows only; solid-row
  behavior must not change.
- **Phase 2 — slack guardrail metric.** Surface the census's slack measure
  (zero-slack pole count + median per case) as a stress-scoreboard line, so any
  future densification change shows its power cost inside the golden-diff gate
  we already run. Built on ground-truth pole positions (see caveat above).
- **Phase 3 — substation rows / dedicated power columns (deferred,
  trigger-gated).** Trigger: any solid-row zero-slack pole appearing in the
  Phase 2 scoreboard, or Phase 1 forcing row-pitch growth. Until a trigger
  fires, no design work. (Census geometry facts for whenever it does: row
  heights mostly 7, inter-row gaps mostly 0–2; an 18×18 substation supply spans
  several row cycles.)

## Kill criteria

- If Phase 0 reveals ≥1 real coverage failure, everything else pauses; if the
  post-fix re-census shifts the motivation numbers materially (>10% on the
  slack or in-span fractions), this RFP's Phases 1–3 must be re-derived before
  proceeding — not patched incrementally.
- If Phase 1's rule cannot hold current row pitch (any corpus fluid row needs
  extra row height to fit the reservation), **stop** — the footprint-vs-power
  trade-off goes to the user, not into code.
- If Phase 1 moves goldens on any non-fluid row, the rule is leaking beyond its
  scope — stop and re-scope before re-blessing anything.
- If Phase 2 turns out to require replaying `place_poles`' exact decision order
  (the unexplained divergence), the divergence investigation gets one timeboxed
  session; unexplained after that → build on ground-truth positions only, or
  drop Phase 2.
- Runtime: the guardrail metric adds ≤5% to corpus runtime (same budget the
  explainability RFP held to), else it moves behind an env flag.

## Verification plan

Per CLAUDE.md layout-change protocol. Phase 0: deliberately drop a machine type
from the shared list in a test and watch the drift-regression test fail; corpus
re-run with warning-population diff. Phase 1: STRESSGOLD before/after with
every moved line explained (fluid rows only); tile-level verification on the
worst-slack cases named above; browser eyeball of one refinery-heavy layout
(user). Phase 2: scoreboard line reproduces the census numbers on unchanged
main as a fixed point.

## Phasing

As numbered above; each phase lands separately. Phase 0 is independent and
could land ahead of acceptance if treated as a plain validator bug fix.

## Decision log

- *2026-07-19 — drafted from the power-placement census (this session's
  `power-census` agent; raw data `scripts/pole-census-2026-07-19.json`). Pending
  adversarial review + user acceptance. Sequencing note: lands after
  `beltspan-lastinrow` merges, since Phase 2's baselines assume the
  post-extension corpus.*
