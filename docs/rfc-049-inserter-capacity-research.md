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
2. **The per-level mid-schedule is NOT reliably known.** Two wiki
   fetches of the same table produced contradictory per-level columns
   (one claimed non-bulk → 10; falsified by sources (a)/(b)). The
   installed draftsman has no `technologies` data to extract from.
   Phase 0 must extract the schedule from an authoritative source
   (Factorio install's tech prototypes, wiki raw HTML table, or the
   user's save); summarized wiki prose is BANNED as a source for
   these constants (kill 3).
3. **Belt-to-belt throughput is sub-linear in hand size** ("timing
   factors": the hand waits to fill from the belt), while
   chest-to-chest is linear (fast: 2.5/5/10 at hand 1/2/4; stack:
   15/20/40 at hand 6/8/16). Engine mapping: **machine-pickup →
   belt-drop sides** (output inserters) are swing-limited like
   chest-pickup — linear `swings × hand_on_belt` is defensible and is
   exactly RFC-046's existing belt-drop model, generalized. **Belt-
   pickup → machine-drop sides** (input inserters) are the sub-linear
   case — their constants must come from measured tables or the
   in-game anchor, never linear derivation (kill 2).
4. **TBC2 interaction**: the Space Age Transport-belt-capacity-2
   research appears to grant the final non-bulk +1 (reaching 4).
   Assumed bundled at L7 (a max-research player has both); Phase 0
   confirms; falsification means non-bulk maxes at 3 — a one-cell
   table fix (kill 5).

## Design

- **Param**: `inserter_capacity: u8` (0..=7), default 0. User-
  specified like its siblings. URL code TBD in implementation —
  **check codec collisions first** (the `s=`→`st=` lesson); sidebar
  select next to Build quality / Belt stacking.
- **Constants**: `common::inserter_hand(name, level) -> f64` (per-
  class schedule table, Phase-0-pinned endpoints 4/12/16) and
  `inserter_belt_pickup_throughput(name, level, quality)` for input
  sides (measured table). The existing flat `inserter_throughput`
  stays untouched — the L0 baseline (bit-identity, RFC-046's "No
  recalibration" pattern). Research-aware paths activate only at
  level > 0.
- **Belt-drop generalization**: `stack_inserter_belt_hand(S)` grows a
  level: `belt_hand(name, level, S) = floor(hand(name, level) / S) × S`
  — the S=4 dip self-heals at hand ≥ 8. Ladder (`size_belt_drop_side`)
  and validator (`belt_drop_throughput`) both consume it (shared
  table, constants-identity test extended).
- **Ladder**: near-rung ceilings scale for input sides via the
  measured table; belt-drop counts via swings × belt_hand. Contest
  (`contest_favors_far`) and `capped_limit` get level threaded like
  quality was.
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

1. **L0 bit-identity**: with `inserter_capacity = 0`, layouts are
   bit-identical to pre-RFC — full suite, goldens 8/8 zero diffs, zero
   re-blesses across the RFC. Any L0 drift is a threading bug; stop.
2. **No linear extrapolation on belt-pickup sides**: every input-side
   constant at level > 0 cites a measured source (wiki throughput
   table cells or the in-game anchor). If measured data can't be
   obtained for a level, that level's input-side scaling ships as L0
   (conservative floor) with the gap documented — never derived.
3. **Schedule ground truth or degrade**: per-level schedule from tech
   prototypes / raw table extraction only (two summarized fetches
   already disagreed — that failure mode is documented and banned).
   Can't pin mid-levels ⇒ ship {Off, Max} and defer the rest.
4. **In-game anchor** (user-run, max-research save; held open per
   precedent): L7 output-side counts match in-game sustained rates;
   the anchor doubles as the TBC2 confirmation (kill 5).
5. **TBC2 bundling**: Phase 0 confirms the +1-at-L7 assumption from an
   authoritative source or the anchor; falsified ⇒ non-bulk max 3,
   one-cell fix, note in decision log.

## Verification plan

- Suite + goldens 8/8 at every phase gate (single-run counts).
- Differential fixtures: L0 vs L7 output-inserter count collapse at
  S=4 (the dip-heals case: counts DROP ~4× at equal rate); an L7
  legendary express fixture extending `stacking_ec_60s_express_...`'s
  audit discipline; ladder unit tests per level (constants-identity
  extended).
- Browser eyeball (user): L7 layout visibly thins output inserters.
- In-game anchor: kill 4.

## Phasing

- **Phase 0 — ground truth, zero behavior change.** Authoritative
  schedule extraction (tech prototypes or raw wiki table — NOT
  summarized fetches); measured input-side table; TBC2 confirmation;
  `inserter_hand`/`belt_hand` helpers + tests, unconsumed; mechanics
  doc I8 extension (per-level rows, cited).
- **Phase 1 — plumbing + validators.** `LayoutOptions.inserter_capacity`
  → `LayoutResult` recorded; validator rating research-aware; L0
  identity gate.
- **Phase 2 — ladder + UX.** Belt-drop + input-side sizing; wasm
  params; URL codec (collision-checked) + sidebar; differential
  fixtures.
- **Phase 3 — deferred.** Sub-linear per-side refinements; mid-level
  completeness if Phase 0 degraded to {Off, Max}.

## Decision log

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
