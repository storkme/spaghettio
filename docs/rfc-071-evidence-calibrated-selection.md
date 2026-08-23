# RFC: Evidence-calibrated selection policy (RFC-071)

## Summary

RFC-070 turned candidate selection into a data problem: one measurement
(`IssueProfile`), one policy table (`SelectionPolicy`), one `decide()`. It
also recorded, deliberately unfixed, the two known miscalibrations of that
table (#700, #701) — because fixing them without measurement would have been
another round of hand-tuning, the exact practice the migration existed to end.
#710 then built the measurement instrument: the calibration matrix, a
35-fixture bank measured by headless Factorio. This RFC runs the first
evidence-driven calibration: bring the matrix to full strength, derive an
evidence table joining validator findings to measured shortfall, and land the
#700/#701 policy changes as table edits adjudicated against that evidence and
the parity corpus.

## Motivation

Three concrete, reproducible cases:

- **gear@20/am2 ships at 75 % of plan** (#700; meter tripwire row
  `gear20-am2-plate`, armed since #693). The cell-composed winner's own
  producer warning said `geometry NOT sim-verified` at selection time; the
  `best-error-free` stage filters on errors and ranks on score, so the flag
  could not participate. Root cause of the deficit itself is a template
  geometry defect (bridge filler tile turns a B11 curve into a B8 sideload —
  one red lane = 15/s), diagnosed and guard-rejected on the issue.
- **ec30 shipped 0.00/s against a 30/s plan** (#701; sim-verified) because
  `ErrorKinds` classed `belt-dead-end` (a total stop with chain-wide
  back-pressure) and `lane-throughput` (a local throttle) at the same weight,
  so 3 total-stops beat 65 throttles. #706 removed the trigger (the (3,2)
  balancer hole); the taxonomy hole remains.
- **The first calibration campaign vetted only 13/35 rows** — 9 excluded on
  research-productivity parity (world ≠ declared axes; harness-side), 8
  non-converged, 1 kit-chest overlap. The instrument exists but is not yet at
  strength.

Precedent for the stakes: #520 ("never worse means never worse *by the
validator*" — a validator-clean, denser layout that simmed at 0/s).

## Design

Two phases. Policy changes are **data edits** to `SelectionPolicy` — the
K70-1 boundary holds throughout (stage code reads registration fields, never
candidate names; anything needing new stage semantics is out of scope here).

**Phase A — instrument to full strength.**

- **A1** — pin recipe productivity in the sim harness world to the declared
  axis (un-research the per-recipe productivity techs for undeclared recipes;
  same option-A principle as the #370/#385 assignments). Unblocks the 9
  parity-excluded rows.
- **A2** — re-measure those 9 rows into the existing bank (their blueprints
  are hash-identical to the current engine, verified on #710 post-merge; only
  their reports are invalid).
- **A3** — the 8 non-converged rows: re-run the cheap four with explicit
  long `--fixed-window`; accept the heavy error-laden two (ac45, ec40) as
  measured-broken data points; the two uranium self-loops stay whatever their
  in-flight runs conclude. Non-converged reports are *data* for B1 (labelled
  as such), not noise.
- **A4** — #710's deferred hygiene (runner content-level report check,
  `is_target` drift exclusion, corpus fingerprint).

**Phase B — evidence, then policy.**

- **B1 — the evidence table.** Join, per matrix row: validator issue
  categories and severities (rebuilt deterministically from
  `calibration_matrix::build`, hash-checked against the bank) × measured
  delivered/produced as % of plan × convergence/kit status. The deliverable
  is a regenerable tool plus the first table, answering: which categories
  co-occur with measured shortfall, which appear only on broken rows, and
  which fire on rows that measure at plan (false-alarm candidates).
- **B2 — `ErrorKinds` reclassification** (#701 fix 1): a class for
  route-severing findings (`belt-dead-end` on a producer output run, and kin)
  above the local-throttle class, weights informed by B1. Expected effect:
  winner flips on the affected parity cells — each flip adjudicated with
  meter receipts and sim anchors where shipped geometry changes.
- **B3 — verification-aware ranking** (#700): a producer's own
  `NOT sim-verified` flag participates in stage ordering (e.g. an unverified
  candidate cannot beat a verified rival on score alone within the same
  stage). Policy-expressible via registration fields + stage predicate data;
  if it is not, K71-4 trips.
- **B4 — the gear@20 geometry fix** (independent of policy): a true B11
  merge in `templates::sideload_bridge`/`single_input_row` (no filler
  predecessor), plus the rejected guard's predicate as a **report-only**
  validator check so the phantom-feeder class is visible. Golden and parity
  drift adjudicated, never blessed blind.

Trade-offs considered: fixing #700 with a throughput term in the scorer
(rejected — reintroduces a hand-calibrated model where a measured flag
suffices); fixing #701 by hardcoding `belt-dead-end` supremacy without
evidence (rejected — the point of the matrix is to stop guessing); guard in
`extract_cell` for B4 (measured and rejected on blast radius, #700 comment 3).

## Kill criteria

- **K71-1 (the evidence must discriminate).** If the evidence table over the
  full-strength matrix shows no issue category or severity whose presence
  separates measured-below-plan rows from at-plan rows beyond chance (every
  category firing on both sides at similar rates), then table-driven
  reclassification is unfounded: stop B2/B3, record the null result, close.
- **K71-2 (no unproven flips ship).** Any policy edit that flips a
  parity-corpus winner must show the new winner ≥ the old on the meter for
  every flipped cell, and sim-anchored where the flip changes shipped
  geometry. A change that cannot clear this on any single cell is reverted,
  not argued for.
- **K71-3 (calibration base).** If after A1+A2 fewer than 20/35 rows vet,
  the base is too thin: pause Phase B, fix the instrument, or re-scope B1
  explicitly to the vetted subset with the shortfall stated in the table
  itself.
- **K71-4 (policy-data boundary).** If B2 or B3 cannot be expressed as
  `SelectionPolicy` data plus registration fields — i.e. it needs new stage
  code that reads candidate identity — it is out of scope; amend this RFC or
  open a new one rather than eroding K70-1.

## Verification plan

The CLAUDE.md layout-engine protocol applies in full to B4; for policy edits:

- Parity corpus (`parity_corpus.rs`, pinned zone cache, `check` before and
  after; expected flips enumerated in the PR, every flip carrying a meter
  receipt; `bless` only after adjudication).
- Meter tripwire rows for the affected fixtures (gear20 must move toward
  plan under B4; no solid row may worsen).
- Sim anchors (432 k warmup, speed 32) for every change to shipped geometry
  and for at least one flipped-winner cell per policy edit.
- The evidence table regenerated after each landed change — the table is
  also the regression record for the calibration itself.

## Phasing

A1 → A2 (needs A1) → B1 at full strength (a draft B1 on the 13-row bank is
allowed and labelled); A3, A4, B4 independent and parallel; B2 before B3
(B3's adjudication reads B2's re-based parity baseline). Each phase lands as
its own PR(s) under the ~400-line norm.

## Decision log

- *2026-08-23 — opened; goal set this session ("run the selection calibration
  policy campaign"). A1 implemented and under sim anchor
  (`tier3_plastic_bar`, live-bank blueprint, patched harness). A4
  (`pr710-hygiene`), B1 (`calib-evidence`) and B4 (`gear20-bridge`)
  dispatched to Codex agents from `0c7bc637`. #706 already closed #701's
  root (3,2) hole; B2 therefore targets the taxonomy only.*
- *2026-08-23 — A3 decided as described (re-run the cheap four with fixed
  windows; ac45/ec40 accepted as measured-broken data): the owner delegated
  the call ("follow your judgement"); revisit only if B1 shows the four are
  not actually cheap.*
