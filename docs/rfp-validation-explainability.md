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
