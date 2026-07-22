# RFC-049: Inserter capacity research (hand-size axis)

Registry: [`rfcs.md`](rfcs.md). Status: **Draft** (2026-07-22).

## Summary

Add a user-facing **inserter capacity research** parameter (level 0–7,
default 0) modeling the hand-size research chain, composing with build
quality (RFC-041: swings) and belt stacking (RFC-046: belt-drop
rounding). At max research a legendary stack inserter belt-drops
**16/swing even at S=4** — ~96 items/s vs the 24/s modeled today — so
the engine currently over-places output inserters ~4× for a
max-research player (conservative-safe, but wasteful hardware and
column pressure). Additive-only, level-0 bit-identical, same
single-source-of-truth shape as its two sibling axes.

## Motivation

The engine's I8 constants model zero research. The user's profile is
max research. RFC-046 deliberately deferred this axis; its S=4
belt-drop dip (hand 6 → 4/swing) is REAL only below capacity level 5 —
at L7 (hand 16) it vanishes entirely (16 is a multiple of 4). Output
sides at S=4/L7 need ¼ the stack inserters the engine places today;
tight position budgets (the contest machinery) feel that directly.

## Ground truth (2026-07-22, three-source cross-check)

1. **Endpoint hand sizes are pinned and self-consistent** across (a)
   entity prototypes (draftsman/data-file extraction:
   `stack-inserter` is **bulk-class** — `bulk: true` — with built-in
   `stack_size_bonus: 4` and `max_belt_stack_size: 4`;
   `rotation_speed: 0.04` = 2.4 swings/s, confirming RFC-046's
   constant from source), (b) the wiki throughput table (fast @ max
   bonus = 10/s = 2.5 × hand 4), (c) the research page's totals:
   **non-bulk 1 → 4, bulk 2 → 12, stack 6 → 16** (= bulk 12 + 4
   built-in — the classes are structurally linked, not independent).
2. **The per-level schedule is PINNED (spec review, 2026-07-22)** via
   raw-wikitext extraction (`api.php?action=parse&prop=wikitext`),
   reproduced identically across two independent fetches — the method
   the spike itself missed. Schedule: **non-bulk** base 1, +1 at L2,
   +1 at L7 → 3 (levels 1/3/4/5/6 contribute nothing to non-bulk);
   **bulk** base 2, +1 at L1–4, +2 at L5–7 → 12; **stack** = bulk
   schedule + 4 built-in → 16. TBC2's "non-bulk inserter capacity +1"
   is LITERAL wikitext (kill 5 CONFIRMED — corroborated by a wiki
   moderator, a dev forum post, and an in-game report of non-bulk 4).
   Historical note kept for the method lesson: two *summarized* fetches
   of the same table had contradicted each other (non-bulk 10 vs 4);
   summarized wiki prose remains BANNED as a constant source — the
   acceptance bar is raw-wikitext with ≥2-fetch reproducibility, or
   in-game measurement.
3. **Belt-fed throughput is sub-linear in hand size** ("timing
   factors": the hand waits to fill from the belt), while
   chest-to-chest is linear (fast: 2.5/5/10 at hand 1/2/4; stack at
   the engine's shipped constant `2.4 swings/s × hand`:
   14.4/19.2/38.4 at hand 6/8/16 — the wiki's rendered 15/20/40 runs
   a uniform 25/24 higher, tick-quantization fidelity already blessed
   by I8's "few percent" caveat; the CONSTANT-derived values are the
   engine's numbers, never recalibrate toward the wiki's). Engine
   side-class mapping (all three classes, spec-review finding 6):
   - **machine-pickup → belt-drop** (output sides): swing-limited like
     chest-pickup — linear `swings × hand_on_belt`, RFC-046's model
     generalized. (One-line nuance: destination belt-tier caps are a
     separate subsystem — belt tier selection — and stay there.)
   - **belt-pickup → machine-drop** (input sides): sub-linear —
     constants ONLY from reproducible measurement (kill 2). Caution,
     demonstrated live in review: the wiki's large belt-fed tables do
     NOT reproduce reliably even via raw wikitext (same cell fetched
     twice: 6.43/s vs 13.33/s) — the in-game anchor may end up the
     only admissible source; the kill-2 fallback (ship L0 for that
     side class) is the honest default.
   - **belt-pickup → belt-drop** (exactly one engine site: the sushi
     sort inserters, `templates.rs` ~5460, hardcoded fast-inserter):
     NO wiki scenario measures this class at all — it stays flat at
     L0 by the kill-2 fallback, explicitly, mirroring RFC-046's
     disposition of the same site.
4. **TBC2 interaction**: the Space Age Transport-belt-capacity-2
   research appears to grant the final non-bulk +1 (reaching 4).
   Assumed bundled at L7 (a max-research player has both); Phase 0
   confirms; falsification means non-bulk maxes at 3 — a one-cell
   table fix (kill 5).

## Design

- **Param**: `inserter_capacity: u8` (0..=7), default 0. User-
  specified like its siblings. URL code: **`ir=`** ("inserter
  research") — chosen deliberately away from the natural `ic`, which
  is one transposition from the already-claimed `ci` (custom inputs);
  a near-miss collision class the spec review flagged as worse than a
  literal one because nothing forces you to notice it. Sidebar select
  next to Build quality / Belt stacking; whether all 8 levels surface
  or just Off/Max is a Phase-2 UX decision (the Motivation's own "mid
  levels are completeness, not need").
- **Constants**: `common::inserter_hand(name, level) -> f64` (per-
  class schedule table, Phase-0-pinned endpoints 4/12/16) and
  `inserter_belt_pickup_throughput(name, level, quality)` for input
  sides (measured table). The existing flat `inserter_throughput`
  stays untouched — the L0 baseline (bit-identity, RFC-046's "No
  recalibration" pattern). Research-aware paths activate only at
  level > 0.
- **Belt-drop generalization**: `stack_inserter_belt_hand(S)` grows a
  level: `belt_hand(name, level, S) = floor(hand(name, level) / S) × S`.
  **The dip heals exactly when `hand ≡ 0 (mod S)`, not monotonically**
  (spec-review correction of this RFC's earlier "heals at hand ≥ 8"
  claim, which was false): with the pinned stack schedule
  (6,7,8,9,10,12,14,16), S=4 is dip-free at L2/L5/L7 but dips
  REAPPEAR at L3 (−1), L4 (−2), L6 (−2); S=3 still dips at max
  research (16 mod 3 = 1). Unit tests must cover an intermediate
  dip level (L3 or L4), not just endpoints. Ladder
  (`size_belt_drop_side`) and validator (`belt_drop_throughput`) both
  consume the shared table (constants-identity test extended).
- **Ladder**: near-rung ceilings scale for input sides via the
  measured table; belt-drop counts via swings × belt_hand. Contest
  (`contest_favors_far`) and `capped_limit` get level threaded like
  quality was. **Site census required first** (spec-review finding
  13): the 8 `size_belt_drop_side` sites are cleanly extendable, but
  ~29 `size_side` sites MIX genuine input sides with
  stacking-exempt OUTPUT sides that deliberately stayed on
  `size_side` (D2b secondary ~templates.rs:766, self-loop majors
  ~4447/4480) yet need the linear output treatment here — call sites
  do not self-identify. Phase 2 runs an explicit census with the
  RFC-046 DropTarget discipline, budgeting for a first-pass miss
  (that census tripped its own kill criterion mid-phase; assume this
  one can too).
- **Quality composition**: `swings(quality) × hand(level)` — quality
  scales rotation, research scales hand, orthogonal by prototype
  structure (rotation_speed vs stack_size_bonus). Documented
  approximation: tick quantization, same fidelity note as RFC-041.
- **Degrade-don't-guess**: if Phase 0 cannot pin the mid-schedule,
  ship the axis as `{0, 7}` (Off / Max research) rather than
  interpolating (kill 3) — Max is the user's actual profile; mid
  levels are completeness, not need.

Out of scope: belt stacking research bundling (RFC-046's `st=` param
stays separate — different tech line), bulk-inserter placement (the
engine still never places it; its column updates for parsed-blueprint
validation only), per-side sub-linear refinements beyond the measured
table (Phase 3).

## Kill criteria

1. **L0 bit-identity, swept across stacking**: with
   `inserter_capacity = 0`, layouts are bit-identical to pre-RFC **at
   every stacking value 1–4** (explicit sweep — RFC-046's kill 3
   tripped on exactly an incomplete-sweep gap; `belt_hand(name, 0, S)`
   must structurally reduce to the shipped
   `stack_inserter_belt_hand(S)`), plus full suite, goldens 8/8 zero
   diffs, zero re-blesses. Any L0 drift is a threading bug; stop.
2. **No linear extrapolation on belt-pickup sides**: every input-side
   constant at level > 0 cites a source **reproduced across ≥2
   independent raw extractions, or a human-verified in-game
   measurement** — a single fetch is demonstrably insufficient (the
   review pulled the same wiki cell twice and got 6.43 vs 13.33). If
   that bar can't be met for a level, that level's input-side scaling
   ships as L0 (conservative floor) with the gap documented — never
   derived. The belt→belt sushi-sort class ships flat at L0
   unconditionally (no measured scenario exists).
3. **Schedule ground truth or degrade**: per-level schedule from raw
   extraction with the 2-fetch reproducibility bar only. *(Largely
   discharged at spike time: the review's raw-wikitext method pinned
   the full schedule reproducibly — see Ground truth 2. The degrade
   path {Off, Max} remains the fallback if Phase 0's re-verification
   fails, but is now unlikely to be needed.)*
4. **In-game anchor** (user-run, max-research save; held open per
   precedent): L7 output-side counts match in-game sustained rates;
   the anchor doubles as the TBC2 confirmation (kill 5).
5. **TBC2 bundling — CONFIRMED at spike time** (raw wikitext literal:
   "Stack size 4, non-bulk inserter capacity +1"; corroborated by a
   wiki moderator, a dev forum post, and an in-game report of
   non-bulk 4). Retained as a criterion only for the in-game anchor's
   final word; falsification now would be surprising and means
   non-bulk max 3 — one-cell fix, note in decision log.

## Verification plan

- Suite + goldens 8/8 at every phase gate (single-run counts).
- Differential fixtures: L0 vs L7 output-inserter count collapse at
  S=4 (L7/S=4 is genuinely dip-free: 16 ≡ 0 mod 4; counts DROP ~4×
  at equal rate); **plus an intermediate-level dip unit test (L3 or
  L4 at S=4, where the dip REAPPEARS — endpoints alone would miss
  it)**; an L7
  legendary express fixture extending `stacking_ec_60s_express_...`'s
  audit discipline; ladder unit tests per level (constants-identity
  extended).
- Browser eyeball (user): L7 layout visibly thins output inserters.
- In-game anchor: kill 4.

## Phasing

- **Phase 0 — ground truth, zero behavior change.** Re-verify the
  pinned schedule via the raw-wikitext 2-fetch bar (method proven in
  review); attempt the input-side measured table under kill 2's bar
  (expected outcome: in-game anchor or L0-fallback per side);
  `inserter_hand`/`belt_hand` helpers + tests, unconsumed; mechanics
  doc I8 extension (per-level rows, cited) — **including fixing I8's
  pre-existing copy-paste bug: the stack row's "up to 27.7/s
  researched" duplicates the bulk row's number (2.4×12≈28.8); stack
  at hand 16 is ~38.4/s** (review finding 5).
- **Phase 1 — plumbing + validators.** `LayoutOptions.inserter_capacity`
  → `LayoutResult` recorded; validator rating research-aware; L0
  identity gate.
- **Phase 2 — ladder + UX.** Side-class census FIRST (the ~29
  mixed `size_side` sites, DropTarget discipline, first-pass-miss
  budget); then belt-drop + input-side sizing; wasm params; URL `ir=`
  + sidebar (levels-vs-Off/Max decided here); differential fixtures.
- **Phase 3 — deferred.** Sub-linear per-side refinements; mid-level
  completeness if Phase 0 degraded to {Off, Max}.

## Decision log

- **2026-07-22 — Adversarial spec review: APPROVE-WITH-CHANGES; v2
  folds all 16 findings.** The review out-sourced the spike: its
  raw-wikitext API method (2-fetch reproducibility) pinned the full
  per-level schedule the spike had left open, and confirmed kill 5
  (TBC2 bundling) outright — both now in Ground truth. It also
  demonstrated the measured-table hazard live (same cell, 6.43 vs
  13.33) → kill 2's bar tightened to reproduced-or-in-game. Corrected
  by the review: the "dip self-heals at hand ≥ 8" claim was FALSE
  (healing is exactly hand ≡ 0 mod S; dips reappear at L3/L4/L6 for
  S=4 and persist at max research for S=3) — fixed, with an
  intermediate-level unit test added to the plan; the wiki-rendered
  15/20/40 replaced by constant-consistent 14.4/19.2/38.4 (25/24
  tick-fidelity gap noted, no recalibration). Added: the third side
  class (belt→belt sushi sorters — ships flat at L0, no measured
  scenario exists), the Phase-2 site census for ~29 mixed size_side
  sites (with first-pass-miss budget per RFC-046 precedent), the
  `ir=` URL code with the ci-transposition rejection recorded, kill
  1's explicit stacking sweep, and Phase 0's I8 27.7-copy-paste fix.
  Also of note: this RFC was drafted as RFC-048 and renumbered to 049
  same-day when PR #341 (cell composition) claimed 048 first — third
  registry collision; the check-before-claiming step ran but both
  sessions read the same pre-PR registry state.

- **2026-07-22 — Spike (this document).** Three-source ground-truth
  cross-check ran before design: entity prototypes extracted via
  draftsman (`scripts/extract_inserter_research.py`; NB: the installed
  draftsman lacks `technologies` data — the schedule needs another
  source), wiki throughput table, wiki research page. Two summarized
  wiki fetches contradicted each other on non-bulk totals (10 vs 4);
  resolved to 4 by prototype+throughput evidence, and the RFC bans
  summarized fetches as constant sources (kill 3). The
  stack-is-bulk-class structural link (16 = 12 + 4 built-in) makes
  the endpoint table internally consistent. Design reuses RFC-046's
  additive-only/no-recalibration pattern and its belt-drop
  decomposition, generalized by one dimension.
