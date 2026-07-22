# RFC-049: Inserter capacity research (hand-size axis)

Registry: [`rfcs.md`](rfcs.md). Status: **Complete** (2026-07-22; in-game anchor open — kill 4; input-side measured-data gap open — #343).

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
  next to Build quality / Belt stacking; RESOLVED in 049-2-ux: the full Off/1–7 select shipped (decision log, 2026-07-22).
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

- **2026-07-22 — Code-lens review: APPROVE, zero correctness findings;
  both hygiene items folded.** The reviewer re-ran every gate, hand-
  traced `belt_drop_rate` against the schedule (confirming the
  L2/L3/L4 plateau, L6 dip, and S=3-dips-at-L7), independently
  re-derived the census with zero missed sites, verified the
  `size_side_rated` refactor byte-preserving, confirmed the
  contest/capped_limit non-threading (and that the residual is real,
  conservative, honestly documented, and identical to RFC-046's), and
  traced the wasm/web param ordering end-to-end. Folded: the stale
  status text (already fixed by the honesty fold) and the ungated
  wasm-bindings clippy debt — `#[allow(too_many_arguments)]` added to
  the 4 param-per-axis wasm surface fns (debt was pre-existing since
  RFC-046 and gated nowhere; now clean rather than growing).

- **2026-07-22 — Honesty-review fold: UX phase logged, wasm-break
  disclosed, staleness swept.** (1) The `049-2-ux` phase (commit
  36fc266) landed wasm-bindings + `ir=` codec + the sidebar select and
  had no log entry — recorded now, including the consequential call it
  quietly made: **the levels-vs-Off/Max UX question resolved as the
  full Off/1–7 select** (all levels surface; the mid-levels are real
  research states and the pinned schedule makes them cheap). Design/
  Phasing prose updated to match. (2) Disclosure the Phase-0+1 entry
  omitted: Phase 1's commit (40fd48d) transiently broke the
  wasm-bindings build (core `LayoutOptions` gained the field; wasm's
  struct literal wasn't updated — CLAUDE.md treats WASM builds as a
  hard check). Fixed ~7 minutes later by 36fc266 in the same session;
  no other phase landed on the broken base. (3) Status Draft→Complete
  (header + registry); RFC-046 ground truth 3 and mechanics BS3 gain
  "modeled since RFC-049 — see I8b" pointers; the input-side
  measured-data gap is now durably tracked as #343. All six findings
  from the honesty lens folded; every numeric claim survived its
  independent re-derivation (schedule re-fetched raw, census
  recounted, fixtures re-run, per-commit suite counts checked out at
  three historical SHAs).

- **2026-07-22 — Phase 2c landed (differential fixtures).** (i) e2e
  `research_l7_thins_output_inserters_s4`: hazard-concrete @ 60/s on
  assembling-machine-1 (per-machine output 20/s — above the 9.6/s L0
  belt-drop rate, below the 38.4/s L7 rate), S=4, red belts. Output
  stack-inserter count (filtered by `carries == output item`, isolating
  the belt-drop side from level-invariant inputs) drops **L0=9 → L7=3, a
  3× thinning** — the per-inserter belt-drop rate rises 9.6→38.4/s (4×),
  realized as 3× because 20/s discretizes to 3 vs 1 inserters (3
  machines × 3→1). Asserted `l7 < l0` and `l0 >= 3·l7` (the sharpness).
  (ii) unit `belt_drop_intermediate_dip_non_monotonic_s4`: at S=4/20-s,
  L4 places 2 stack inserters where L5 places 1 (L4 > L5) and L4 == L2 —
  the mod-4 plateau (hands 8/9/10 all floor to 8 = 19.2/s, then L5 jumps
  to 12 = 28.8/s); endpoints alone would miss it. (iii) L0 identity is
  the kill-1 gate (goldens 8/8, `size_side_output_identity_at_l0`,
  `belt_drop_identity_at_s1_l0`). No existing fixture weakened — the 5
  `stacking_` fixtures and all 8 goldens are byte-unchanged. Gates: full
  suite (lib 779 / e2e 60 / netflow parity 10, one clean run), STRESSGOLD
  check 8/8 zero diffs, clippy `--lib -D warnings` clean (the `--tests`
  target carries pre-existing clippy debt in junction_fixtures /
  mode_d_baseline / e2e helpers, untouched by this work — the project
  gates on `--lib`).

- **2026-07-22 — Phase 2b landed (exempt-output scaling).** New
  `size_side_output(required, reach, budget, max_tier, quality, level)`
  for the class-(c) stacking-exempt outputs: it routes through the same
  `size_side_rated` core at `belt_drop_rate(name, quality, 1, level)` —
  the **unstacked** belt-drop rate scaled by research — so the far
  long-handed ceiling rises 1.2→4.8/s and near tiers scale linearly,
  WITHOUT stack-forcing (exemption intact; a low-rate near output still
  gets the cheapest tier, never a forced stack). Chosen over a post-plan
  scale because it reuses the shared ladder core and stays a drop-in for
  the existing `size_side` call sites; at level 0 it is bit-identical to
  `size_side`. Wired at the three class-(c) template sites: the D2b
  secondary output (`single_input_row` ~772) and both self-loop major +
  the minor output (`self_loop_row` 4464/4480/4497). `self_loop_row`
  gained a `level` param (it already took `&StackingCtx` for per-item
  exemption; `level` is the global research dimension, threaded from
  `build_one_row` like the other 8 templates — kept as a plain param, not
  folded into the ctx). Unit tests: L0 identity sweep, far-ceiling rise
  (4.0/s shortfalls at L0, one LHI covers at L7), and the exemption
  guard (near output never forces stack at max research). Gates: full
  suite (e2e 59 / lib 778, one clean run), STRESSGOLD check 8/8 zero
  diffs, clippy --lib clean. No first-pass census miss surfaced.

- **2026-07-22 — Phase 2a landed (site census + belt-drop ladder).**
  Explicit census of every `size_side` (29) + `size_belt_drop_side` (8)
  site in `templates.rs`, plus the hardcoded inserter pushes and the
  sushi sorter (RFC-046 DropTarget re-census discipline: call sites do
  not self-identify their drop target). No first-pass miss surfaced —
  the RFC-046 census had already mapped the exempt output sites, and
  this census reconciled cleanly against them; still budgeting for a
  later miss on the class-(c)/contest boundary.

  | Class | Site(s) `templates.rs` | Role | RFC-049 treatment |
  |---|---|---|---|
  | (a) input | 645; 998/1018; 1458/1473; 1773/1793; 1855; 2172; 2249; 2486; 2780/2792; 4317/4378/4400; 5129/5144; 5431; hardcoded LHI 2143/2155/4304/4336/4358 | belt-pickup → machine-drop | **UNCHANGED** — flat `size_side`, no `level` (kill 2 L0 floor; input scaling needs measured data or the in-game anchor) |
  | (b) output belt-drop | 715; 1080; 1510; 1870; 2233; 2534; 2853; 3859 (all `Reach::Near`) | machine-pickup → belt-drop, already on `size_belt_drop_side` | **level threaded** — S>1 forces stack at `belt_drop_rate(stack,q,S,level)`; S≤1&L>0 cheapest-tier ladder at research-scaled rates |
  | (c) exempt output | 766 (D2b secondary, Far); 4447/4480 (self_loop major, Near); 4463 (self_loop minor, Far) — all `size_side` deliberately | stacking-exempt output (family plans unstacked) | **`size_side_output` (Phase 2b)** — linear `swings/throughput × hand(level)`, no stack-forcing; exemption intact |
  | (d) probe/contest | 943/945/947, 1716/1718/1720 (far-capped/near shortfall probes on INPUT rates); `contest_favors_far` ×7 (992,1454,1768,1850,2228,4443); `capped_limit` (464) | shortfall probes + shared-column tie-break + trace-string | **NOT threaded** (verified, see below) |
  | belt→belt | 5499 sushi sorter (hardcoded `fast-inserter`) | belt-pickup → belt-drop | **flat L0, no code change** — no measured scenario exists (ground truth 3, kill 2) |

  **Ladder.** `common::belt_drop_rate(name, quality, stacking, level)` is
  now the single source of truth for belt-drop rates, consumed by BOTH
  the ladder and the validator's `belt_drop_throughput` (which became a
  one-line wrapper) — constants-identity by construction, pinned by a new
  `belt_drop_constants_identity` unit test. `size_belt_drop_side` gained
  `level`; the S>1 forced-stack path count-ladders at
  `belt_drop_rate(stack,…,level)`, the S≤1&L>0 near path and the Far path
  route through a rate-parametric `size_side_rated` core (extracted from
  `size_side`, which is now a thin flat-rate wrapper — golden path
  byte-preserved: goldens 8/8, kill 1 held). Threading mechanism:
  `level` is a `place_rows`/`build_one_row` parameter parallel to
  `max_inserter_tier`/`quality` (an inserter-ladder input), fed from
  `LayoutOptions.inserter_capacity` — NOT folded into `StackingCtx`,
  which stays purely belt-stacking (validators derive it for belt
  capacity and never want a level).

  **`contest_favors_far` / `capped_limit` — verified NOT threaded (task's
  "only if the math consumes a rate that changes").** Both consume only
  `inserter_throughput` (the flat, level-INDEPENDENT I8 constants) and
  `size_side` (likewise level-independent) — no rate in their math moves
  at level>0, so threading `level` would be a literal no-op. This is
  exactly RFC-046's disposition: it did NOT thread `stacking` into either,
  even though the same contest sites (1850/2228 output-vs-input3/4,
  4443 self_loop major-vs-minor) already pit belt-drop OUTPUT sides
  against input sides. The residual approximation — the tie-break and the
  shortfall-trace string use the flat machine-drop ceiling for output
  sides, so at level>0 an output-near side with `required` in the narrow
  band `(flat 12/s, level-scaled ceiling]` is treated as slightly more
  constrained than it is — is pre-existing, conservative (never
  under-provisions the actual sizing, which `size_belt_drop_side` does
  separately), and cosmetic (column tie-break + trace explainability
  only). Deferred with RFC-046's identical residual; noted here rather
  than hidden.

- **2026-07-22 — Phases 0+1 landed.** Phase 0: schedule re-verified
  in-session (two raw-wikitext extractions, identical — kill 3's bar
  met); `inserter_hand` + `stack_inserter_belt_hand_at` landed
  additive/unconsumed with the dip-matrix unit test (L2/L5/L7 heal at
  S=4, L3/L4/L6 dip, S=3 dips at max research) and the L0-reduces-to-
  RFC-046 identity sweep; I8b added to the mechanics doc and the
  pre-existing 27.7 copy-paste on the stack row fixed (→ ~38.4).
  Phase 1: `LayoutOptions.inserter_capacity` → recorded on
  `LayoutResult` (serde skip-0/default-0 — pre-RFC snapshots
  deserialize unresearched); `belt_drop_throughput` grew the level
  dimension (stack: swings × belt_hand_at whenever either axis is
  active; non-bulk belt-drops: flat × hand, L0-reduces-to-flat; bulk:
  flat always — never placed, conservative floor for parsed
  blueprints). Kill-1 gate: suite 871/0/36 (one clean run), stress
  suite green, goldens 8/8 zero diffs, zero re-blesses. Input sides
  untouched (kill-2 L0 floor stands until measured data or the
  anchor). Note: the branch is named `rfc-048-inserter-research` from
  before the renumber — cosmetic; the PR will note it.

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

- **2026-07-22 — Phase 2: kill 2 closed WITH measured data; input side
  now level-aware.** Kill 2 ("no linear extrapolation on belt-pickup
  sides without measured data") was written waiting for an instrument
  that now exists. Prerequisite: sim tech-state parity (#370) — the
  harness set `research_all`, granting bonus 7 regardless of the
  fixture's `inserter_capacity`; the scenario now assigns the force
  bonuses directly (probe-verified model: non-bulk hand = 1 +
  `inserter_stack_size_bonus`, {bulk, stack} hands = {1, 5} +
  `bulk_inserter_capacity_bonus` — the two fields reproduce all three
  I8b tables exactly; tech un-research left non-bulk +1 via an
  unidentified tech, so direct assignment). Calibration matrix (5
  inserter types × L{0,2,7}, flooded express feed → quad-speed-3 AM3
  stone-wall sink, 37.5/s ceiling, kit self-audit clean on all 15
  cells), measured belt→machine intake vs `flat × hand(L)/hand(0)`:
  regular 0.88/1.75/3.62 (model 0.84/1.68/3.36 — +4–8% margin),
  long-handed 1.25/2.50/5.00 (1.20/2.40/4.80, +4%), fast
  2.75/5.38/10.88 (2.31/4.62/9.24, +16–19%), stack 12.62/20.75/34.25
  (12.0/16.0/32.0, +5–30%), bulk 5.38/10.88/27.75 (flat floor
  retained — engine never places bulk). **Hand-ratio scaling is
  conservative in every measured cell**; the belt-DROP swings×hand
  decomposition is ~12% optimistic for machine feeds and is NOT used
  on the input side. Landed: `common::machine_feed_rate` (bit-identical
  flat at L0), both input-side call sites in the throughput checks
  switched, unit test pins the L0 identity and the measured-floor
  ceilings. Full suite green (787 unit + 60 e2e). NOT changed: the
  sizing ladder stays L0-flat — letting placement exploit L>0 hands
  means denser layouts that require research, a user-facing trade
  explicitly deferred to its own decision. #352's four warnings are
  hereby adjudicated: correct at the L0 semantics they were computed
  under; the clean-kit sim PASS was measured at research-inflated
  hands (pre-parity) and does not contradict them.

- **2026-07-22 — Phase 2 amendment: the #376 adversarial review's
  blocker was CONFIRMED BY MEASUREMENT; stack switched to a measured
  floor table.** The review flagged that L1/L3–L6 were credited by
  formula without measurement, against this RFC's own "never derived"
  rule. Completing the matrix (all 8 levels for stack and bulk — 10
  further cells, kit-audit clean) proved the objection empirically:
  stack's measured machine-feed curve is **non-monotone in hand size**
  (12.62, 12.38, 20.75, 20.75, 23.12, 24.00, 23.88, 34.25 across
  L0–L7 — dips at hands 7 and 14, swing-cycle quantization), and
  hand-ratio over-credited L1 (14.0 vs 12.38) and L6 (28.0 vs 23.88)
  with L5 at exactly zero margin. A monotonicity argument would have
  failed too — vindicating kill 2's "measured, never derived" in full.
  Landed: `machine_feed_rate`'s stack branch uses the measured floor
  table [12.0, 12.0, 19.2, 19.2, 21.6, 22.8, 22.8, 32.0] — monotone,
  3–8% under every measured cell, flat-12.0 bit-identity at L0. Bulk
  measured safe at all 8 levels under hand-ratio (margins 1.93–2.27×);
  non-bulk hand values {1,2,4} were fully covered by the original
  cells. Test pins credit ≤ measured at every level for both types
  plus monotonicity. Review findings 2–4 (Lua-table drift guard,
  parity self-audit into kit_errors + report-surfaced bonuses,
  bless/check level-mismatch test) all landed in the same pass.*

- **2026-07-22 — Phase 3: the sizing ladder honors the declared level.**
  The Phase-2 close-out deferred L>0-aware placement as a user-facing
  trade; decided jointly (user + session, 2026-07-22): the axis is
  user-DECLARED, exactly like belt tier — honoring it is not
  auto-escalation, and refusing to use it ignores the user's input.
  Landed: `size_side` (input sizing) and `contest_favors_far` (near/far
  slot contests) take the research level and rate through the measured
  `machine_feed_rate`; `capped_limit` diagnoses counterfactuals in the
  same level world; threaded through all template call sites,
  `voider_row`/`scrap_recycling_row`, and the placer. At L0 bit-identical
  (792 unit + 60 e2e green untouched). Differentials pinned: 3.0/s input
  thins stack→single-regular at L7; far ceiling 1.2→4.8; a 13.0-near vs
  1.0-far contest flips with the level. Sim verification of an
  L7-thinned fixture through the parity harness queued behind the
  running sweep. Corroborating datum from the same day's parity re-runs:
  ec10 FAILS at L0-parity hands — the four #352 warnings were correct
  at the semantics they were computed under, confirming both the L0
  floor's honesty and the value of sizing to the declared level.
