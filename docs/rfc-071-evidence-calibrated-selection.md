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
  could not participate. The deficit's root cause was believed at opening to
  be a template geometry defect (the bridge-filler B8 story on the issue);
  execution FALSIFIED that — the real cause was the cell chain's final drain
  hardcoding yellow belt, and the decision log's #715 entry carries the
  correction with receipts (one red lane and a full yellow belt are both
  15/s, which is why the tile-level story looked confirmed).
- **ec30 shipped 0.00/s against a 30/s plan** (#701; sim-verified) because
  `ErrorKinds` classed `belt-dead-end` (a total stop with chain-wide
  back-pressure) and `lane-throughput` (a local throttle) at the same weight,
  so 3 total-stops beat 65 throttles. #706 removed the trigger (the (3,2)
  balancer hole); the taxonomy hole remains.
- **The first calibration campaign vetted only 13/35 rows** — the sweep's
  buckets: 13 measured, 9 kit-errored on research-productivity parity
  (world ≠ declared axes; harness-side), 11 non-converged (including the
  kit-chest-overlap row), 2 awaiting measurement — 13+9+11+2 = 35. The
  instrument exists but is not yet at strength.

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
- **B4 — the gear@20 fix** (independent of policy). As opened, this named
  a bridge-geometry redesign; execution falsified that diagnosis (decision
  log, #715): the deficit was the cell chain's final drain hardcoding
  yellow belt, fixed by tiering the drain for its planned rate. The
  rejected guard's predicate as a **report-only** validator check (the
  phantom-feeder class stays real as a *pattern*) remains a follow-up.
  Golden and parity drift adjudicated, never blessed blind.

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
- *2026-08-23 — A1 implemented, hardened (only pure-productivity techs are
  un-researched; a tech carrying any other effect is refused loudly) and
  sim-anchored twice on live-bank blueprints: tier3_plastic_bar shows the
  pin working (kit_errors [], realized plastic productivity 0 vs the
  campaign run's 0.1 + parity error — its own 0/s non-convergence is
  byte-identical pre/post-pin: a pre-existing validator-clean defect, held
  for post-A2 triage) and tier4_advanced_circuit_from_plates shows a
  healthy fixture unperturbed (converged, kit-clean, PASS, 100.3%/101.7%).
  PR #714.*
- *2026-08-23 — B1 landed as PR #713 (probe + joiner + first table, 33/35
  reports at join time; 35/35 rebuilt-blueprint hashes matched the bank).
  K71-1 direction positive on the draft table: route-severing categories
  (belt-dead-end, belt-flow-path, belt-flow-reachability,
  orphan-belt-segment, unresolved-junction) appear ONLY on broken rows;
  input-rate-delivery and row-input-belt-margin straddle shortfall and
  at-plan rows (the weighting targets). Headline: tier1_iron_gear_wheel_20s
  sims at 76.0% delivered with a fully clean validator sheet — the Factorio
  confirmation of #700's meter 75%. Formal K71-1 adjudication waits for the
  post-A2 regeneration.*
- *2026-08-23 — B4 attempt 1 (Codex, stopped at its stop condition, recorded
  on #700): removing the filler yields a true B11 curve but single-lane
  starves the output run (~11.4/s vs 7.5/s lane cap); the bridge is the
  template's only lane-rebalance for I5 far-lane drops. Meter unchanged
  15.0/20.0 on both attempts; nothing committed. Reclassified from bounded
  fix to lead design work on the output-run lane loading.*
- *2026-08-23 — A2 launched: bank forked to
  /tmp/calibration-matrix-2026-08-23-pinned (blueprints hash-identical to
  the engine at 0c7bc637, receipt on #710), the 9 parity-contaminated
  reports replaced — 2 by the A1 anchors, 7 re-measuring under the frozen
  pinned harness binary (7da4d90d). Uranium rows arrive from the owner's
  still-running campaign.*
- *2026-08-23 — process note: #711/#712/#713/#714 all have their required
  second-opinion review failing on an OpenRouter 402 (key credit
  exhaustion); merges queue on the owner's key/balance fix, then the failed
  workflow runs get re-run. No unrequire without the owner's word.*
- *2026-08-23 — B4 landed as PR #715 with the root cause CORRECTED: not the
  bridge — the chain's final drain hardcoded yellow (15/s) under a 20/s
  product. Discrimination (patch six tiles → meter 15→20), fix (rate-based
  drain tier; meter 20.0/20.0), sim anchor (converged, kit-clean, PASS
  20.0/20.0), parity 160/160 no drift, ec30 registry re-blessed off a real
  re-measurement (27.73/30, identical — its drains were never binding),
  tripwire −25→0 with a plain-main attribution run proving the other rows'
  sub-tolerance wiggles pre-exist the change. #700 updated; the bridge's
  filler/B8 subtlety survives only as the report-only detector follow-up.*
- *2026-08-23 — A2 complete: 7/7 rows re-measured under the pinned harness,
  0 failures; surplus_export arrived from the owner campaign (non-converged
  at 95.7% — U235's probabilistic recipe at 0.05/s does not stabilize);
  voider queued. Coverage: 20 measured / 1 awaiting / 1 kit-error / 13
  non-converged.*
- *2026-08-23 — B2 landed as PR #716: `IssueKind::RouteSevered`, ranked
  lexicographically between Structural and the weighted functional total;
  membership = the five evidence-exclusive categories (three latent while
  Warning-severity). The #701 mechanism is pinned as a unit test. K71-2
  adjudicated on the only two flips (ec35, ec40 — both previously
  measured-dead): meters 0→8.0/35 and 0→6.75/40, sims 0→8.0 converged and
  0→7.5 converged kit-clean. Parity corpus 160/160 zero drift. ird/margin
  weights deliberately untouched (evidence straddles). DISCOVERY along the
  way, fixed in the same PR: the shipping path classified through the
  category consts and the policy's `error_kind_classes` table was
  decorative — a policy-table edit did not steer shipped selection.
  `classify_errors` now reads the table; K71-4's premise (policy edits are
  data edits) is true only as of this PR.*
- *2026-08-23 — B3 landed as PR #717 (stacked on #715), with a measured
  design pivot recorded: the first draft was a produce-time acceptance
  gate on the "geometry NOT sim-verified" note, and the suite refuted it —
  refusing unverified cells re-shipped broken natives in the rescue class
  (four cells tests red). Shipped shape: an ORDERING —
  `RankSpec::verified_geometry_first` on BestErrorFree, over the measured
  `IssueProfile.unverified_geometry` bit — verified outranks score,
  unverified-but-only still wins. gear@20's cell geometry graduated into
  cell-sim-registry off the #715 sim anchor (20.0/20.0, PASS), with
  re-derivable fixture rows in both registry gates; discrimination
  executed (corrupt the entry → the fixture flips to native, restored).
  Parity 160/160 zero drift; the rule bites future contests only. Along
  the way #715 absorbed two record corrections found by B3's checks: the
  gear20 K0-1 golden re-bless my earlier failure-grep missed (the PR's
  verification claim was corrected in a comment), and the fixture's
  stale "KNOWN UNDER-DELIVERING" pin prose.*
- *2026-08-23 — **K71-3 MET** (20/35 ≥ 20) and **K71-1 PASSES**: on the
  full-strength table the five route-severing categories (belt-dead-end,
  belt-flow-path, belt-flow-reachability, orphan-belt-segment,
  unresolved-junction) appear ONLY on broken rows — zero occurrences on
  all 20 working factories — while ird/row-input-belt-margin straddle both
  sides (5-vs-6, 4-vs-1: weights territory, too weak to retune) and
  belt-detour leans false-alarm. **B2 proceeds** with exactly that shape:
  elevate the route-severing kinds into a class above the local-throttle
  classes; change no weights. B3 remains next after B2.*
- *2026-08-23 — CLOSING STATUS (supersedes the stale present-tense wording
  in earlier entries): every phase is landed as a PR — A1 #714, A2 done
  in-bank, A3 decided, A4 #712, B1 #713, B2 #716, B3 #717, B4 #715 — with
  K71-1 PASS, K71-2 receipts on every flip, K71-3 MET (20/35), K71-4 held.
  Remaining is process only: bot-round absorption, the merge queue
  (#711/#712/#714/#713/#716 → #715 → retarget #717), and the registry row
  flip to Complete at close-out. Open follow-ups (out of scope, recorded):
  ird/row-input-belt-margin weights (evidence straddles both sides), the
  phantom-feeder report-only check, the >45/s single-drain ceiling and K>1
  drain coverage (#715 review), RouteSevered trace observability (#716
  review, absorbed there), RFC-068 P1 (owner-gated).*
- *2026-08-24 — COMPLETE. The merge queue landed in the recorded order
  (#712, #711, #713, #714, #716, #715, #717-retargeted); 19 bot review
  rounds adjudicated across the seven PRs, every absorb and refutation
  receipted in the PR threads. Registry row flipped to Complete;
  status ledger updated. #700 and #701 closed with their primary
  subjects fixed and sim-anchored; residual follow-ups live in this
  log's closing-status entry and the issues' final comments.*
- *2026-08-24 (post-close) — the bank-regeneration and CI-wiring follow-ups
  executed. A fresh export from merged main diffs against the pinned bank at
  exactly the campaign's three geometry changes (gear@20 drain, ec35/ec40
  merge-tap flips) — nothing else drifted; the 32 unchanged rows' bp+manifest
  pairs byte-verified and their reports carried (voider's overnight report
  included, closing the last awaiting row). Re-measured: gear@20 100.0/100.0
  PASS; ec40 clean-measured 18.5% (corroborates the B2 receipt); ec35 22.9%
  but kit-errored (overlapping drain chests) — labelled evidence, not a clean
  row. First full 35/35 coverage: measured 21, kit-error 2, non-converged 12.
  The bank's `matrix.json` is committed at `crates/core/data/calibration-bank/`
  and CI's rust job runs the issue-breakdown probe against it (verbatim-command
  pass + corrupt-one-hash fail both executed locally; the committed zone cache
  is byte-identical after a full export, so the gate is deterministic in CI).
  New next-round signal recorded, no policy change made: belt-flow-reachability
  (Warning, latent RouteSevered) and lane-throughput's 631 E now co-occur with
  ec40's measured deficit — evidence for the ird/margin-weights follow-up and
  the reachability severity-promotion question in validator-trust.md.*
