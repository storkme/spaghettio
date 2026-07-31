# RFC-060: HorizontalStack as a scored decomposition candidate

Status: ACTIVE (2026-07-30) — implementation in progress on
`claude/ui-simplification-w8y9p6`. Evidence base: issue #513 (the
`full_knob_sweep` winner map, `051368b`). Companion: #512 (sidebar
simplification), whose Engine drawer loses the Row layout knob when this
RFC completes.

## Summary

Promote `RowLayout::HorizontalStack` (RFC-013) from an opt-in knob to a
default-on **scored candidate** in the decomposition search, following
the lifecycle `cell_composition` (RFC-051) and direct insertion
(RFC-053, #474) already walked: the native pass runs vertical-split as
today, a horizontal variant competes where it could differ, and it
displaces native only on **strict improvement across both issue
channels** (ties → native, so every layout horizontal does not strictly
improve stays bit-identical). `RowLayout::HorizontalStack` remains as
the force mode (debug), mirroring `DirectInsertion::Forced`.

## Motivation

First `full_knob_sweep` run (14-case corpus, debug + CI zone-cache pin,
engine `40e2acc`): every one of the 8 contested cases is won by a
`*/horiz` combo; `pool/vert` (the shipped default) wins none of them and
hard-refuses `ec@15 am3 plates` outright — a config horizontal builds at
0 errors / 252 entities / 51.2% density. Headline flips: `ac@7` E10/W64
→ E0/W0 (−460 entities), `pu@3` E11/W149 → E0/W35 (−292), `ac@5` E4/W11
→ E0/W0 (−456). Where ineligible, horizontal falls back and ties
bit-identically (all 6 simple cases). Full table: #513.

## Design

- `LayoutOptions.horizontal_candidate: bool`, **default `true`**. `false`
  = pure vertical (tests, sweep baselines, debug). No new user-facing
  knob: the wasm surface pins it `true`; force-horizontal stays reachable
  via the existing `row_layout` param (`rl=hs`).
- `HorizontalStackCandidate` in `decomposition_search.rs`, structured
  exactly like `DirectInsertionCandidate`: clone opts, set `row_layout =
  HorizontalStack`, `run_layout_with_retry`, self-validate and refuse a
  layout carrying validation errors (conscious conservatism inherited
  from DI: a horizontal layout with 1 error never displaces a native
  with 10 — the pairwise rule only ever upgrades to error-free).
- Gate (cost control, mirrors `try_di`): candidate mode on AND
  `row_layout == VerticalSplit` AND `placer::has_dual_input_row(
  solver_result)` — `row_kind()` is a pure function of `MachineSpec`,
  and `RowKind::DualInput` is the only kind whose construction consults
  `row_layout`, so "no dual-input row" ⇒ bit-identical by construction
  and the extra pass is skipped. `catch_unwind` like cells/DI/merge-tap.
- Scoped pairwise choice `horizontal_choice`: `accepted &&
  strictly_better_issues` → win; `None` otherwise. **Narrower than
  `di_choice`** — no equal-issues-and-denser arm in v1 (see decision
  log: the density arm's measured value was ≤5% entity shaves on clean
  layouts, its measured cost ten flipped structural artifacts). Never
  returns `Some(NATIVE_IDX)` (the #474 shadowing lesson). When native
  produced nothing, returns `None` so horizontal competes in the
  generic ranking against `cell-composed`/DI — that is the `ec@15`
  rescue path.
- Candidate sits LAST (index 6). `ranking_len` = `DI_IDX` when native
  produced (excludes both DI and horizontal from the generic ranking —
  the single enforcement point the DI default rests on, extended), else
  `H_IDX + 1`.
- DI-vs-horizontal when **both** beat native: the same pairwise metric
  between the two winners, ties → DI (earlier candidate, preference-
  order consistency with the array). Deliberately NOT a 3-way generic
  score — same reasoning as keeping DI out of the soft score.
- No composition of DI *within* the horizontal pass in this RFC: the
  variant runs with the same opts otherwise, and `di_couplings`-driven
  fusion inside `place_rows` applies as it does for native. Cross-
  candidate stacking (horizontal + forced-DI hybrid pass) is out of
  scope; recorded as a possible follow-up.

## Kill criteria

- **K60-1 (runtime)**: suite wall-clock multiplier from the extra pass
  exceeds 1.5× on the e2e suite (same budget K-DS1-3 gave DI, which
  landed at 1.23×) → tighten the gate or abandon default-on.
- **K60-2 (never-worse has teeth)**: the pinning test
  (`horizontal_candidate_never_degrades_a_succeeding_bus_layout`) red at
  any point, or any corpus case where candidate-on regresses either
  issue channel vs candidate-off → block; the contract is structural,
  not aspirational.
- **K60-3 (sim honesty)**: any flipped corpus case whose sim-measured
  delivery drops below native's by >5% of plan → the static winner key
  is lying for that class; revert default pending a fixed comparator.
  **This criterion gates the merge**, not the implementation: the PR
  stays draft until the flipped cases (`ac@5`, `ac@7`, `pu@2`, `pu@3`,
  `ec@15`) sim at/above native.
- **K60-4 (scope)**: net engine LOC (excluding tests/docs) exceeds ~400
  → the integration point is wrong; stop and redesign rather than
  ballooning.

## Verification plan

1. Pinning test mirroring `di_candidate_never_degrades_a_succeeding_bus_layout`.
2. `full_knob_sweep` re-run: default column should now match the
   previous per-case winner wherever horizontal won, and stay
   bit-identical on the 6 all-tie cases.
3. Full suite with the CI zone-cache pin green; goldens/scoreboards that
   move are re-blessed only where the movement is an improvement on both
   issue channels, each re-bless recorded in the decision log.
4. Sim harness on the five flipped cases with a LONG `--warmup`
   (`docs/status.md` deep-chain caveat) — merge gate per K60-3.
5. Browser eyeball of at least `ac@5` and `ec@15` (verification protocol
   step 2).

## Decision log

- *2026-07-30 — RFC opened. Candidate-shape (mirror DI) chosen over
  (a) riding the generic soft score — refused for the same measured
  reason DI was: horizontal is typically denser and would win score
  while regressing warnings; and (b) a `RowLayoutMode` enum replacing
  `RowLayout` — refused because `row_layout` is the pass-level template
  selector consumed inside `place_rows`, and conflating policy with
  selector would touch every construction site for no behavioural gain.*
- *2026-07-30 — `horizontal_candidate` stays engine/test-surface only;
  wasm pins it `true` and no web UI is added. A vertical-only web escape
  hatch can ride a later wasm param if a real debugging need appears;
  adding it now would mint a new knob in the same change that exists to
  retire one.*
- *2026-07-30 — Self-validation refusal (no error-laden horizontal ever
  competes) accepted knowing it forgoes E10→E1-class improvements;
  the sweep shows horizontal's wins land at E0, so the forgone region is
  empty on current evidence. Revisit only with a measured case.*
- *2026-07-30 — K60-1 measured at the final green suite: e2e wall-clock
  129.7s (baseline, 64 tests) → 190.2s (candidate on, 67 tests) =
  **1.47×**, inside the 1.5× budget with no headroom to spare — the
  `any_dual_input_row` short-circuit is load-bearing, and any future
  widening of eligibility must re-measure this. Zone-cache pin refreshed
  per the ci.yml protocol (+185KB of new signatures from horizontal
  variant routes).*
- *2026-07-30 — Cell-geometry derivation pins `horizontal_candidate:
  false` (extract + both mega sub-solve sites), joining the existing
  cells/DI recursion-guard pins. Found by `probe_registry_pin_survey`
  bisection: chain-ec15/ec30 registry hashes drifted deterministically
  at HEAD while every other fixture held — the EC sub-builds carry
  exactly the belt-margin warnings the candidate strictly improves, so
  the extraction path silently swapped to horizontal rows. Rule joins
  the forced-modes rule: geometry-DERIVATION paths must be
  candidate-independent by construction; a registry baseline that moves
  because a sibling candidate got smarter is a false re-bless.*
- *2026-07-30 — Error-free refusal tier now orders by WARNINGS first,
  then score (was: score only). Adding a third refusal-resolving
  candidate exposed that the #392 error-free tier re-admitted the
  density-over-warnings class on the refusal path: horizontal's ec@15
  resolution (0 errors, 6 warnings, denser) would have outranked DI's
  genuinely clean 0/0. `cell_candidate_resolves_ec15_refusal` pins the
  corrected order. Within error-free candidates, quiet beats dense.*
- *2026-07-30 — Density arm DROPPED from v1 after the second full-suite
  run measured its blast radius: with `equal_issues_and_denser` active,
  ten pinned structural artifacts across two suites failed (4 cell
  registry/chain fixtures, 6 e2e including the stacking per-tile audits
  and `tier2_electronic_circuit_20s_from_ore`) — every one a CLEAN
  layout flipped for a ≤5% entity shave (sweep: ec@20 1392→1386, pu@2
  6499→6210). All headline value (`ac@5` E4/W11→0/0, `ac@7` E10/W64→0/0,
  `pu@3` E11/W149→0/W35, `ec@15` refusal rescue) comes from the
  strictly-better arm, which flips nothing that was clean. Rule:
  horizontal displaces native only where native has issues to fix.
  The density arm is a candidate to re-enable alongside RFC-058 packing
  work, where structural churn is already priced in.*
- *2026-07-30 — First full-suite run caught a forced-mode interaction:
  under `DirectInsertion::Forced` (explicit A/B topology request) the
  horizontal candidate competed, won, and returned a DI-free layout —
  `di_bridge_feeds_cable_only_at_high_research` failed with "DI must be
  stamped (got 0)". Fix: `try_horizontal` stands down when DI is Forced,
  the same exclusion `try_cells` already carries. Rule generalized for
  future candidates: forced modes beat candidates, always.*
- *2026-07-30 — Sim verification (K60-3) CANNOT run in this session's
  remote container: factorio.com is unreachable through the environment
  network policy (connection reset on both the site and the pinned
  2.0.77 headless download URL — measured, not assumed). Additionally
  the manifest generator `crates/core/examples/sim_probe_export.rs` is
  gitignored and absent on fresh clones (the RFC-050 "known gap").
  Consequence: the implementation lands as a DRAFT PR; K60-3's five
  flipped-case sims (`ac@5`, `ac@7`, `pu@2`, `pu@3`, `ec@15`, long
  warmup) are the ready-for-review gate and must run from a
  harness-capable machine.*
- *2026-07-31 — `pu@2` drops OUT of the K60-3 flipped set: with the
  density arm gone it stays bit-identical native (sweep: 6499 entities
  both columns), so there is no differential to measure. Four cases
  remain; artifacts regenerated from a harness-capable machine via the
  now-tracked `rfc060_sim_export` e2e exporter (committed this session
  to close the RFC-050 reproducibility gap — E/W signatures reproduce
  exactly; entity counts on SAT-routed cases can drift ±3 with
  zone-cache state).*
- *2026-07-31 — **K60-3 measured: no trip.** All arms simmed at
  `--warmup 216000` (pu3: 288000), converged. Per case, delivered/s
  (on = shipped default with the candidate, off = candidate disabled):
  `ac5` **3.75 (−25%) vs 0.00** — native E4/W11 deadlocks outright;
  `ac7` **6.00 (−14.3%) vs 0.00** — native E10/W64 deadlocks;
  `ec15` **14.45 (−3.6% WARN) vs 14.18 (−5.5% WARN)** — the off arm is
  the cell-composed rescue (this branch's #392 reorder lets cells
  resolve the old native refusal even without horizontal; raw bus still
  refuses, pinned by `cell_candidate_resolves_ec15_refusal`).
  In every measurable case the candidate's delivery is at or far above
  native's; the winner key is not lying. pu3 recorded separately below
  (first run invalidated by a harness defect).*
- *2026-07-31 — Harness defect found and fixed during K60-3: pu3's
  crude-oil@(38,0) and water@(39,0) boundary ports are ADJACENT, and
  the kit's bare infinity-pipes (one tile beyond each port) merged into
  a single network — crude won, the water trunk carried crude, the
  sulfuric-acid chain never ran, and both pu3 arms measured 0.00/s with
  436 machines `full_output`. Fix in `crates/sim-harness/scenario.rs`:
  each fluid feed now sits behind an isolated ug-pipe run with per-slot
  staggered length so the merge-capable surface caps never touch. No
  blessed baseline exercises fluid feeds (all five are solid chains),
  so no re-bless. The invalid first runs are kept in the session
  artifacts, not cited as evidence.*
- *2026-07-31 — **New measured finding (does not trip K60-3, tracked
  separately): the flux blind spot is real and quantified.** `ac5-on`
  is E0/W0 yet delivers 75.0% of plan (drift +0.0%, converged): the EC
  row's copper-cable arrives via three tap drops that fill ONE lane of
  yellow (~7.5/s each, 22.5/s against a 30/s demand) → EC pins at
  exactly 7.5/s → AC at exactly 3.75/s, while cable machines sit
  `full_output` and 10/40 AC machines never receive a single item.
  `ac7-on` same mechanism, milder (−14.3%). The `belt_flow` lane walker
  models sideload-to-one-lane semantics but credited this chain at
  both-lane rate — zero warnings. K60-3's comparator holds (native
  delivers 0.00), so the graduation stands; the validator calibration
  gap is tracked in
  [#519](https://github.com/storkme/spaghettio/issues/519) with the
  tile-level forensics.*
- *2026-07-31 — **pu3 re-run on the fixed kit: K60-3 holds on the last
  case.** `pu3-on` (E0/W35) delivers **2.47/s (−17.6%)** with the chain
  flat at ~−24% (cable/plates/EC all −24%, the #519 single-lane-tap
  signature; the PU stage's machine-count ceiling absorbs it back to
  −17.6%). Formally `converged=false` at drift +2.4%, but the window
  series (2.57→2.47→2.52→2.49→2.55→2.48→2.55→2.49→2.52) is oscillation
  around 2.51, not a ramp — quantization noise on 300-item windows, not
  a buffer-fill transient. `pu3-off` (native, E11/W149) delivers
  **0.00/s** — a genuine deadlock (380 `full_output` + 82 starved), not
  a kit artifact, on the fixed kit. Comparative verdict: 2.47 ≫ 0; no
  flipped case sims below native. K60-3 does not trip; the draft→ready
  gate is cleared.*
- *2026-07-31 — pu3-on warmup sweep per the deep-chain rule: re-run at
  `--warmup 432000` (2 game-hours) CONVERGES at **2.53/s delivered
  (−15.7%, drift +1.5%)** vs 2.47 at 288k — flat against warmup, so the
  deficit is real, not a buffer-fill transient. Chain still uniformly
  ~−24% (EC −23.9%, AC −23.7%): the #519 single-lane-tap mechanism is
  warmup-invariant. Recorded as the case's honest floor; #519's fix
  should move it.*
