# RFP: Validation explainability — from warning soup to visible causes

**Revision 2** — rewritten after a two-reviewer adversarial pass
(2026-07-14) falsified the original draft's anchor diagnosis, its
central kill criterion, and its cost estimate. The review findings are
summarized in the decision log; the design below is the corrected one.

## Summary

The validator finds real problems, but the web UI surfaces them as
atomized one-line symptoms, and the *causal* knowledge the engine had
at stamp time is computed and then thrown away. This RFP adds four
composable pieces: **(1) a starvation heatmap** overlay driven by
structured delivered/needed numbers, **(2) causal attribution** carried
on the existing `InserterSideCapped` trace event and joined per-tile in
the UI, **(3) click-to-explain** entity highlighting, and **(4) a cause
rollup** in the sidebar. Metadata and UI only — no new `LayoutResult`
surface, no golden/snapshot risk by construction.

## Motivation

Reproducible today: `#/l/ecl/35/am2/ior,coo/tbr`
(`stress_electronic_circuit_35s_from_ore`). The user eyeballed starved
assemblers and read the scene as "power poles are in the way, so we
can't fit 2× long-handed inserters." The session lead endorsed that
reading and blamed the templates' pole-gap reservation.

**Adversarial review proved both humans wrong — which is this RFP's
best evidence.** The actual mechanism (traced from the `.fls` snapshot
and the code):

- The starved iron sides are capped by `dual_input_row`'s near/far
  **shared-column contest** (`contest_favors_far`,
  `templates.rs:794-796`) — the ladder's position budget loses a column
  to the contest, fits 1 LHI where the plan needs ~1.46/s, and the
  engine records exactly that: 24 `InserterSideCapped` trace events
  (shortfall 0.258) matching 12 `inserter-item-throughput` warnings
  ("moves 1.20/s, needs 1.46/s") to the decimal.
- The pole is a *bystander*: `place_poles` runs after templates and
  only takes tiles the ladder left free. Poles dodge inserters, never
  the reverse. The only pole-gap reservations in the tree are in fluid
  rows, never wired to inserter sizing.
- The 35 `input-rate-delivery` warnings the original draft anchored on
  are a *third* thing entirely: zero-delivery disconnections
  (co-located `belt-flow-reachability` errors, tracked in #297) that no
  inserter ledger could ever attribute.

Three failure modes, three causes; two humans and the original RFP
draft each picked the wrong one from the rendered layout. The engine
knew the truth the whole time — in trace events and warning payloads
nothing surfaces. That is the problem statement.

Explicitly **out of scope**: fixing the contest/face-allocation
geometry itself (parked by the user 2026-07-14, "maybe we'll come back
to this" — belongs to the face-allocation north-star). This RFP makes
that future work's evidence gathering trivial.

## Design

### D1. Structured numbers on `ValidationIssue`

`ValidationIssue` (validate/mod.rs:66) gains
`pub detail: Option<IssueDetail>` with machine-readable
`delivered`/`needed` — both rate checks (`check_input_rate_delivery`,
`check_inserter_item_throughput`) already compute these locally before
formatting prose (verified in review). No cause fields here — cause
lives on the trace side (D2), keeping `ValidationIssue` a symptom
record.

Required TS edits the derive does NOT cover (review finding): the
hand-declared mirrors in `web/src/renderer/validationOverlay.ts:4-10`
and `web/src/ui/snapshotLoader.ts:32-38` must gain `detail` explicitly.
`.fls` additions use `#[serde(default)]` per the
`effective_rows`/`voided_streams` pattern.

### D2. Attribution via the existing `InserterSideCapped` event

The original draft proposed a new `side_provisions` ledger on
`LayoutResult`. Review found the engine already emits
`InserterSideCapped` (trace.rs:218-225) at ~30 of the 31 `size_side`
call sites, carrying recipe/side/required/placed/shortfall — nearly the
whole proposed payload — and the web UI already has a per-tile
trace-event join with click-to-pin (`web/src/ui/tileContext.ts` +
`inspector.ts`). So D2 becomes an extension, not a new type family:

- Add to `InserterSideCapped`: `machine_x`, `machine_y` (in scope at
  every emit site — used two lines later to place the entity) and
  `limit: &'static str`, a small vocabulary of budget-cap reasons.
- **Honest vocabulary, v1**: `"column-contest"` (dual-input near/far
  contest — the anchor mechanism), `"tier-cap"` (max_inserter_tier),
  `"geometry"` (catch-all: the row shape offers no further slots).
  Review confirmed the *why* is currently discarded to a scalar
  `position_budget` before `size_side` — so tagging is per-call-site
  authoring work, and v1 only tags what the call site cheaply knows,
  defaulting to `"geometry"`. `SlotReservedForPole` from the original
  draft is DELETED — that mechanism does not exist for solid rows.
- The join is tile-keyed in TS (`tileContext.ts` pattern): a warning at
  a machine looks up co-located `InserterSideCapped` events. No `Face`
  enum, no new join key on the validator side (review: the item
  -throughput check sums across sides and no Face type exists — the
  original join-key design was fiction).
- Unattributed warnings stay unattributed. The UI never guesses.

### D3. Web UI

- **Heatmap toggle** (Phase 1): color machines by `delivered/needed`
  from D1. Rides the existing `overlayPanel.ts` toggle pattern.
- **Click-to-explain** (Phase 3a): clicking an issue marker pins the
  tile (existing `pinTile` machinery) and renders co-located capped
  -side events — planned vs fitted, limit reason — in the inspector.
- **Cause rollup** (Phase 3b, split per review): sidebar grouping of
  warnings by `(category, limit)` for attributed ones + "unattributed"
  bucket. The original draft's "lane under-provision" example is
  deleted — that cause source doesn't exist in this design (a lane-side
  provision source is future work, listed as a non-goal).

### Trade-offs / rejected

- **New `side_provisions` ledger on `LayoutResult`** (original draft):
  rejected by review — duplicates an existing event stream, adds
  golden/snapshot surface for no attribution power, and its
  "join, not inference" framing hid per-call-site authoring anyway.
- **Post-hoc inference in the validator**: still rejected — carrying
  stamp-time facts forward is the point.
- **Message-string parsing in TS**: still rejected — D1 is the same
  size and doesn't rot.

## Kill criteria

1. **Anchor precondition (checked before Phase 2 starts)**: re-run the
   anchor fixture and confirm ≥10 `inserter-item-throughput` warnings
   still co-locate with `InserterSideCapped` events (baseline measured
   2026-07-14: 12 warnings / 24 events). If the anchor no longer
   exhibits the mechanism (e.g. #297-adjacent fixes moved it), pick a
   new anchor by measurement before building — do not build against a
   stale target.
2. **Attribution coverage, measured against the RIGHT population**: on
   the anchor, every `inserter-item-throughput` warning whose machine
   has a matching capped-side event must render its cause in the UI;
   warnings without events must show "unattributed". If <90% of the
   *event-matched* warnings surface a cause, the join is broken. (The
   original 80%-of-input-rate-delivery criterion is dead: measured
   attribution against that population is 0% — they're #297
   disconnections.)
3. **No inference creep**: if attribution ever requires the validator
   or UI to *compute* a cause (anything beyond the tile-keyed event
   join), the design is wrong — stop.
4. **Scoreboard stability**: full suite + `SPAGHETTIO_STRESS_GOLDEN`
   before/after must be identical (golden_hash covers entities only —
   review confirmed this is a safety net, not a discriminating gate;
   the discriminating gate is warning-count stability in
   `check_stress_scoreboard` across all fixtures).
5. **Runtime**: >5% validate+layout regression on the corpus → trim or
   kill.

## Verification plan

- Criterion 1 measurement first, before any Phase 2 code.
- Unit tests: emit sites carry machine coords + limit; the anchor's
  contest-capped side tags `"column-contest"`; a tier-capped fixture
  tags `"tier-cap"`.
- Anchor case: count attributed vs unattributed in the rollup; spot
  -check 3 attributions against the entity map and the shortfall
  arithmetic (0.258 pattern).
- UI per `feedback_user_validates_ui`: land, let the user eyeball on
  the anchor URL; no screenshot iteration.

## Phasing

1. **Phase 1 — numbers + heatmap** (independent of the disputed-and-
   corrected narrative; safe to build first): `IssueDetail`
   delivered/needed + the two TS mirror edits + heatmap toggle.
2. **Phase 2 — event extension + join**: `InserterSideCapped` coords +
   limit vocabulary; tileContext join. Gated by criteria 1–3.
3. **Phase 3a — click-to-explain**; **Phase 3b — cause rollup**. Two
   separately landable UI diffs (split per review, gold-plating
   control).

## Decision log

- *2026-07-14 — drafted after the user's EC@35s eyeball session; user:
  tooling "sounds worthwhile", geometry fix itself parked.*
- *2026-07-14 — revision 2 accepted by the user ("go for it");
  **Phase 1 LANDED (667c880)**: IssueDetail{delivered,needed} on the
  three rate-shaped emission sites + the two hand-maintained TS
  mirrors + the Starvation heatmap toggle (top-level, persisted,
  pointer-inert layer, footprint-tinted via MACHINE_SIZES). Gates:
  666 lib + 45 e2e green, STRESSGOLD byte-identical before/after
  (KC4), clippy/wasm/tsc clean. One deviation found landing it: a
  ValidationIssue struct literal in tests/e2e.rs (outside the src/
  grep) — switched to the constructor. Awaiting the user's browser
  eyeball per feedback_user_validates_ui; Phase 2 (event extension +
  tileContext join) gated on kill criterion 1's anchor re-check.
  User eyeballed the heatmap on the anchor: "pretty good" — Phase 1
  validated; user greenlit Phase 2.*
- *2026-07-14 — **Phase 2 LANDED (981f40e)**. KC1 anchor re-check
  PASSED first (24 events / 12 warnings, unchanged from baseline).
  Design refinement over the brief: a capped plan is best-effort
  (every slot filled at the richest allowed tier), so budget, reach
  and tier ceiling are recoverable FROM THE PLAN — `capped_limit`
  derives the limit centrally with counterfactual ladder re-runs
  (tier-cap: stack at same budget covers; column-contest: caller
  flags the lost column AND budget+1 covers; else geometry), keeping
  all ~30 emit sites mechanical (coords + a contest bool). Scrap-row
  emit moved inside the machine loop for per-machine joins. KC2
  measured on the anchor: **12/12 warnings join by machine origin
  (100% ≥ the 90% bar)**; KC3 held (plain key lookup, no inference).
  **Finding: limits read 24× "geometry", correcting the review's
  row-level "column contest" narrative** — every capped far side
  sits at a position where the contested column never existed (a
  contest winner places 2 LHIs and doesn't cap at all; the capped
  positions are the trimmed ones with zero budget). The attribution
  tooling sharpened its own RFP's diagnosis on first use — third
  correction of this bug's story (pole → contest → trimmed-position
  geometry), each one cheaper than the last. tier-cap and
  column-contest are unit-tested reachable but absent from this
  corpus cell (anchor runs default Stack tier). Gates: 669 lib + 45
  e2e, STRESSGOLD byte-identical, clippy/wasm/tsc clean. Remaining:
  Phase 3a click-to-explain, 3b cause rollup (both need the
  heatmap-style user eyeball of the new hover line first).*
- *2026-07-14 — **the parked power question ANSWERED with entity
  evidence (poles exonerated), and a trace-hygiene bug found.**
  Item-aware pole probe at all 12 real capped machines: the pole
  sits at the middle inserter column (dx=1, dy=-1) at every one,
  and the far row above that column carries NO iron belt — the tile
  is useless to the starved side; the binding constraint is the
  iron belt's trimmed span (the ladder's "geometry" verdict,
  confirmed). The engine already gives inserters precedence
  (place_poles runs after and dodges); the parked priority-inversion
  fix is MOOT — the real lever is belt-span/row geometry
  (face-allocation territory). NOTE the first, item-BLIND probe
  read "12 implicated" — the same error class as the census's
  item-blind pooled ceilings; item-awareness flipped it. **Bug
  found while verifying: 12 of the anchor's 24 InserterSideCapped
  events are DISCARDED-CANDIDATE debris** — machine coords (x=14,
  pitch 8) match no final machine (a copper balancer sits there);
  LayoutRetried:0 rules out retry residue; the events sit early in
  the stream (idx 7-18). run_candidate captures+truncates per
  candidate, but the merge_tap_choice produce path leaks its
  events into the winner's trace. Attribution risk: a warning
  coincidentally anchored at a phantom coord would mis-explain
  (didn't occur — the 12/12 join was measured against real
  machines). Fix scoped next: scrub candidate-produce events in
  merge_tap_choice the way run_candidate does.*
- *2026-07-15 — **phantom-event root cause CORRECTED (three theories,
  third one measured true) + fix landed (05b205c); Phases 3a+3b
  landed (571cbe0) — ALL RFP PHASES COMPLETE.** The candidate-leak
  theory was falsified by reading run_candidate (capture+truncate is
  correct, merge-tap produce included); the retry-residue theory by
  the stream itself (LayoutRetried absent). Truth: layout_pass runs
  rows+lanes TWICE (estimated bus width → actual), pass-1 entities
  discarded but its events kept — the phantom coords are pass-1
  geometry (Δx=−28 the width correction, Δy=−13 the balancer band),
  and the codebase already had the pattern for this
  (with_merge_tap_fallback_suppressed, same two-pass dedup, opposite
  pass kept since capped coords are NOT pass-invariant).
  remove_capped_events_since scrubs pass-1 capped events when pass 2
  runs; anchor now: 12 events, all real machines, 12/12 joined.
  Streaming caveat noted: the live sink still sees pass-1 events
  (same accepted semantics as the retry); the web joins from the
  final layout.trace, which is scrubbed. 3a: pinned inspector block
  with per-side explanation (plain-language limit text). 3b: sidebar
  cause rollup per category — detail-bearing issues joined via the
  same tileContext, counts per cause, honest "unattributed", unions
  for multi-limit tiles, no guessing anywhere. Gates: 670 lib + 45
  e2e, STRESSGOLD byte-identical, clippy/wasm/tsc clean. REMAINING:
  user eyeball of the 3a pinned block + 3b rollup (per
  feedback_user_validates_ui); then this RFP closes.*
- *2026-07-14 — **adversarial review (2 reviewers, parallel): REVISE
  ×2; draft rewritten as revision 2.** Confirmed findings against the
  original draft: (a) the anchor diagnosis was WRONG — the starved
  sides are capped by dual_input_row's near/far column contest, not a
  pole-gap reservation (poles are placed after inserters and only take
  leftover tiles; pole-gap reservations exist only in fluid rows,
  unwired to sizing) — both the user's eyeball read and the lead's
  endorsement misattributed it, which is itself the strongest
  motivation for this tooling; (b) KC2 gated on the wrong warning
  population — the 35 input-rate-delivery warnings are #297
  zero-delivery disconnections, 0% inserter-attributable (measured
  from the .fls snapshot); the real mechanism lives in 12
  inserter-item-throughput warnings ↔ 24 InserterSideCapped events
  (shortfall arithmetic matches to the decimal); (c) the proposed
  side_provisions ledger duplicated the existing InserterSideCapped
  event + tileContext/inspector UI — design switched to extending
  those; (d) D3's "lane under-provision" rollup example was
  undeliverable by D2's vocabulary — deleted; (e) no Face enum exists
  and the item-throughput check sums across sides — the (machine,
  face, item) join key was fiction; join is now tile-keyed in TS; (f)
  hand-declared TS mirrors (validationOverlay.ts, snapshotLoader.ts)
  named as required edits; (g) golden_hash covers entities only — KC1
  demoted to safety net, warning-count stability promoted to the
  discriminating gate; (h) "light" relabeled honestly: Phase 2 is
  per-call-site authoring (~30 sites), v1 vocabulary kept minimal to
  match. Phase 1 declared safe to build independent of all findings.*
- *2026-07-19 — **first material payoff + one residual**. The
  `geometry` verdict this RFP's attribution surfaced (far-side cap at
  last-in-row machines, pole exonerated) led straight to the
  last-in-row belt extension (`0d7132c`, adversarially reviewed,
  APPROVE): anchor inserter-item-throughput 12 → 0, corpus 12 → 8
  (all remaining in production-science / the untouched
  triple-quad-hstack trims). Residual found during that review: the
  self_loop cap shape has NO InserterSideCapped emit site — fish and
  pentapod self-loop cap warnings carry no events and stay honestly
  unattributed. Known emit-site gap, not a join bug; candidate for a
  Phase-2-style call-site addition if self-loop rows ever matter.*
