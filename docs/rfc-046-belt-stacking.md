# RFC-046: Belt stacking (Space Age stacked belts)

Registry: [`rfcs.md`](rfcs.md). Status: **Complete** (2026-07-21;
in-game anchor open, Phase 3 deferred).

## Summary

Add a user-facing **belt stacking** parameter (stack size 1–4, default
1 = off) modeling the Space Age mechanic where stack inserters build
item stacks on belts, multiplying effective belt throughput by up to
4×. Stacking enters the engine at the capacity-query layer in
`common.rs` (belt tier selection and lane capacity scale ×S), forces
stack inserters on belt-dropping sides so the ×S credit is honest, and
threads through layout options, the lane-rate validator, wasm, the URL
codec, and the sidebar — the same single-source-of-truth shape as
build quality (RFC-041). Solver untouched: stacking is pure logistics.
Delivered headline (restated during Phase 2 — see the decision log):
the [#311](https://github.com/storkme/spaghettio/issues/311) stress
config, **EC@60/s on red belts from ore** — whose committed golden
stamps 60/s onto a physically-30/s merger belt with zero warnings —
becomes physically valid end-to-end at S=2 (red stacked = 60/s), one
belt, proven by a direct per-tile capacity audit. The originally
drafted legendary-express@60 variant is NOT proven: it hits
pre-existing high-rate residuals unrelated to stacking (Phase 3
pick-up).

## Motivation

The user plays very-lategame Space Age: legendary everything, endgame
research. For that profile our belt model is wrong by up to 4×:

- **Mechanics** (wiki, fetched 2026-07-21 — [Stack
  inserter](https://wiki.factorio.com/Stack_inserter), [Transport belt
  capacity](https://wiki.factorio.com/Transport_belt_capacity_(research))):
  belt stack size is 1 base → **2** (granted by the stack-inserter tech
  itself, Gleba) → **3** / **4** (Transport belt capacity 1/2, each
  needing all seven sciences). A stacked belt carries `tier_rate × S`
  items/s. Stacks are created **only** by stack inserters dropping onto
  belts; splitters, sideloads, and undergrounds preserve stacks; any
  inserter can pick from a stacked belt.
- **Today the engine models S=1 always** — deliberately, and
  conservative in the safe direction (real ≥ plan), per the
  naming-wrinkle close-out in `rfc-build-quality.md` and #313. But
  conservative-by-4× means endgame layouts are 4× wider than they need
  to be (trunk counts, balancer shapes, fan-in walls), and the
  quality-magnified refusal in
  [#312](https://github.com/storkme/spaghettio/issues/312) bites at
  rates a stacked build trivially carries.
- Concrete failing case today: EC@60/s legendary express is capped to
  45/s by #311's single-output-belt reality; EC@6/s legendary yellow
  refuses outright (#312). Both are physically fine builds with S≥2.

## Ground truth (mechanics being modeled)

To be folded into `docs/factorio-mechanics.md` as numbered rules in
Phase 0 (cited; in-game anchor confirms):

1. **Belt stack size S ∈ {1,2,3,4}** is a per-force research state, not
   a per-belt property. All belt tiers stack alike; capacity is
   `belt_throughput(tier) × S`, per-lane `lane_capacity(tier) × S`.
2. **Stack creators are: stack inserters (belt drops only), big
   mining drills, and recyclers** (wiki, verbatim: "Only stack
   inserters, big mining drills and recyclers can create belt
   stacks"). Inserter drops into machines/chests are exact-hand (no
   rounding). Engine relevance of the non-inserter creators: the
   `voider_row` and `scrap_recycling_row` templates eject recycler
   output **directly onto belts with no inserter** (mining-drill-style,
   `templates.rs`) — those belts are stack-loaded by the machine
   itself. Provisionally modeled as stacked at the researched S
   (Phase 0 verifies against the Recycler wiki page; fallback in kill
   criterion 4).
3. **Belt-drop hand rounding**: when dropping onto a belt, the hand is
   rounded **down** to a multiple of S. Wiki base hand is **6**
   (built-in capacity bonus): per-swing on belts = 6 at S∈{1,2,3},
   **4 at S=4**. So per-inserter belt throughput *drops* at S=4 unless
   capacity research raises the hand — which we don't model (deferred,
   Phase 3). Modeling the rounding is mandatory: crediting 6/swing at
   S=4 would over-plan (real < plan — the unsafe direction).
4. **Stack-preserving elements**: splitters, sideloading, underground
   belts, belt-to-belt merges. No engine element destroys stacks.
5. **Non-stack inserters never stack**: a belt loaded by regular / fast
   / long-handed inserters carries S=1 flow regardless of research.
   There is **no reach-2 stacking inserter** (I8a) — a long-handed
   belt-drop side cannot contribute stacked flow, ever. The adversarial
   spec review (2026-07-21) settled the engine census: the **one**
   reach-2 belt-drop in active templates is `self_loop_row`'s minor
   output (kovarex-class, `size_side(minor_produced_rate, Reach::Far,
   …)`, templates.rs ~4436); every other `Reach::Far` site is an input
   side. Handled by the minor-lane guard in the Design section.
6. **Pickup from stacked belts** works for every inserter type and is
   still bounded by the inserter's own items/s ladder — input-side
   sizing is unchanged.
7. **Quality composes orthogonally**: quality scales inserter rotation
   (stack inserter 864°/s → 2160°/s legendary); S is research-only.
   `throughput = swings(quality) × hand_on_belt(S)`.

## Design

### Parameter

`stacking: u8` (1..=4), default 1. **User-specified, never inferred or
auto-escalated** — same contract as belt tier
(`feedback_belt_tier_user_specified`). URL codec `st=2|3|4` (absent =
1; `s=` was the spec'd key but is already claimed by `strategy` —
implementation deviation, documented in `web/src/state.ts`), sidebar
dropdown "Belt stacking: Off / ×2 / ×3 / ×4". Threaded:
`LayoutOptions.stacking` → recorded on `LayoutResult.stacking`
(skip-if-default serde, like `wire_mode`) so validation and
improve-region recompute read the layout's own value. Imported /
parsed blueprints get S=1 (no research context in a blueprint string;
conservative — community stacked builds may warn, documented).

### Capacity layer (`common.rs`, single source of truth)

- `belt_throughput(belt, stacking)` and `lane_capacity(belt, stacking)`
  — base × S.
- `belt_entity_for_rate(rate, max_tier, stacking)` — lowest tier with
  `tier_rate × S ≥ rate`; max-tier cap semantics unchanged.
- `stack_inserter_belt_hand(stacking) = floor(6 / S) * S` …
  effectively 6, 6, 6, 4 — used by the ladder for belt-drop sides.
- `stack_inserter_swings(quality) -> f64` = `2.4 × quality.multiplier()`
  (864°/s ÷ 360°; consistent with the wiki's 14.4/s = 2.4 × hand 6).
  Today `inserter_throughput("stack-inserter", q)` is one opaque
  `12.0 × multiplier` — the swings/hand decomposition is a **new
  additive helper pair**, not a change to the existing constant (see
  "No recalibration" below).

Call sites (13 files consume these helpers) thread the parameter
mechanically, exactly as quality did. Anything that plans a rate onto
a belt uses the stacked capacity; anything measuring machine-side
inserter work does not.

### Honesty rule: forcing, not just crediting

Crediting ×S capacity on a belt that nothing stacks would be the #311
trap again — a validator and planner agreeing on fiction. Rule:

> **When S > 1, every engine-placed belt-dropping inserter side is
> forced to `stack-inserter`** (the ladder's tier floor for belt-drop
> sides becomes stack-inserter; count sizing then uses
> `stack_inserter_swings(quality) × stack_inserter_belt_hand(S)`).

Plumbing reality (spec review finding 4): `size_side(required, reach,
position_budget, max_tier, quality)` cannot distinguish belt-drop from
machine-drop sides today — nothing in its signature says what the drop
target is, and all ~37 template call sites pass the same shape. The
forcing rule therefore requires a new `DropTarget::{Belt, Machine}`
parameter (or a dedicated `size_belt_drop_side` entry point) threaded
through the template call sites — most are distinguishable by their
section (output-inserter blocks vs input blocks), but this is a
mechanical ~37-site diff, not a one-line ladder switch. Scoped as its
own Phase 2 step.

With uniform forcing, *every* engine-loaded belt is stack-loaded, so
×S applies uniformly in planning and validation with no per-lane
stackedness bookkeeping. The claim "every engine-loaded belt" has
exactly three load paths, each covered:

1. **Inserter belt-drops** — forced to stack-inserter (above).
2. **Direct machine ejection** (`voider_row` / `scrap_recycling_row`
   recycler banks, no inserter) — recyclers are themselves stack
   creators (ground rule 2); credited ×S deliberately as a second,
   non-inserter stacking path, pending the Phase 0 wiki verification
   (kill criterion 4 holds the fallback).
3. **External input trunks** — assumed stacked at the boundary
   (documented UI assumption — the user's mall feeds them; if it
   doesn't, real < plan on externals only).

Rejected alternatives:
- **Per-lane stackedness tracking** (only force where a lane plans
  above unstacked cap): strictly better hardware cost, but mixed
  stacked/unstacked flows sharing trunk segments turn capacity
  accounting per-segment; deferred to Phase 3, not needed for the
  endgame profile this serves.
- **Capacity-only change without forcing**: dishonest; killed on
  sight.

**Known hole — unstackable belt-load sites (ground rule 5): census
v2 and the family-exemption rule.** Kill criterion 3 tripped during
Phase 2 implementation: the spec review's census (self_loop minor as
the only far belt-drop) was incomplete. The re-census found **three**
unstackable belt-load classes in active templates:

1. `self_loop_row` minor output (templates.rs ~4436, `Reach::Far`) —
   as reviewed;
2. **Fulgora D2b secondary output** (templates.rs ~757, `Reach::Far`,
   hard-0 budget) — a second long-handed belt-drop the review missed;
3. **sushi sort inserters** (templates.rs ~5460, hardcoded
   `fast-inserter`, belt-to-belt) — inside the recycler subgraph that
   kill 4 already keeps unstacked, so consistent by construction.

The v1 "guard the minor lane" idea is **unsound**, not just
incomplete: kovarex's minor export is the *same item* as its stacked
major output, so both merge into one lane family — and a trunk
carrying mixed stacked + unstacked flow obeys fractional occupancy
(`r_unstacked/cap + r_stacked/(S·cap) ≤ 1`), which uniform ×S
crediting cannot express. Bounding only the unstacked tributary does
not bound the mix.

**Replacement rule — static family-level stacking exemption**: a lane
family is *exempt* (plans at ×1 end-to-end: belt tier selection,
split thresholds, fan-in caps) iff any of its producer rows is
unstackable-classed — `self_loop` (minor ⇒ whole family, since major
shares the item), secondary-output rows for the secondary item, and
`voider`/`scrap_recycling` rows (kill 4). Exemption is derived
statically from `SolverResult` specs before lane planning and carried
as `effective_stacking` on the family/lane — no dynamic per-lane
tracking (still deferred to Phase 3). Soundness: the planner caps
exempt families at unstacked capacity, so their belts never *need*
stacked headroom — the validator's uniform ×S credit can then never
mask a real overload on them. Mixed trunks cannot arise because a
family is exempt as a whole, and distinct items never share trunks.

**`max_inserter_tier` conflict**: forcing belt-drop sides to
stack-inserter at S>1 contradicts a user cap below `Stack`. Belts
cannot stack without stack inserters, so the config is incoherent —
resolved as a **named refusal at layout entry** (`stacking > 1`
requires `max_inserter_tier = Stack`), consistent with the
hard-constraint philosophy (`feedback_belt_tier_user_specified`):
degrade never, refuse honestly.

### Validator

- `validate/belt_flow.rs` lane-rate walker: capacities ×S from
  `LayoutResult.stacking` (line 2133's `lane_capacity(belt_name)` and
  friends).
- Inserter-throughput checks: belt-drop sides rated at
  `swings(quality) × stack_inserter_belt_hand(S)`; machine-drop sides
  unchanged.
- #311's merger-tile blind spot is **out of scope and unchanged** —
  the walker still doesn't visit merger tiles. The headline KC is
  therefore proven by snapshot decode, never by warning-count (kill
  criterion 2).

### No recalibration — the belt-drop model is additive-only

An earlier draft bumped the I8 stack-inserter base (hand 5, 12/s) to
the wiki's hand 6 / 14.4/s, on the argument that belt-drop rounding
makes the delta consequential (`floor(5/3)*3 = 3` per swing at S=3 vs
the real 6 — a 2× under-credit). The adversarial spec review killed
that plan as **self-contradictory with kill criterion 1**: the flat
`inserter_throughput` constant is not conditioned on S, the ladder
already places stack inserters at S=1 today (`InserterTier::Stack` is
the default top rung), so the bump would shift S=1 layouts — the exact
thing the identity gate forbids — and the phasing even scheduled the
cause before the gate.

Resolution: **the existing flat 12.0 × multiplier is not touched.**
All current call sites (machine-drop sizing, S=1 belt-drop sizing,
validators) keep the deliberately conservative I8 constant, and the I8
note's "revisit only with evidence" stands. The wiki-accurate
decomposition `stack_inserter_swings(quality) ×
stack_inserter_belt_hand(S)` exists only on the **new** code path:
sizing belt-drop sides when S > 1 (where the rounding model is
mandatory for correctness — crediting 6/swing at S=4 would over-plan).
The disclosed inconsistency — belt-drop path models hand 6 while the
flat path stays at 12.0 ≈ hand 5 — is deliberate: each side errs
conservative in its own regime, S=1 behavior is provably bit-identical,
and zero goldens re-bless. The in-game anchor (kill criterion 5) is
the check on the hand-6 figure.

### Export

No new blueprint fields expected: stack inserters stack to the force's
researched max by default. The 2.0 blueprint format does carry an
optional per-inserter `override_stack_size` (uint8; absent = default)
— we deliberately never emit it, so exports inherit the importing
force's research. Phase 0 spot-checks a community blueprint
(blueprint-analyze) to confirm the parser tolerates the field on
import; the in-game anchor (kill criterion 5) is the final word.

### Explicitly out of scope

- Inserter capacity-bonus research (hand > 6, non-stack hand bonuses)
  — a full ladder recalibration axis; Phase 3 candidate.
- #311 merger rework and #312 balancer wiring — both *interact* (S
  lifts the fan-in wall ×S and shrinks requested balancer shapes) but
  neither is fixed here.
- Solver / recipe selection — stacking never changes machine counts.
- Renderer stack visualization (badges/overlays) — nice-to-have,
  Phase 3.

## Kill criteria

1. **S=1 bit-identity.** With `stacking = 1` (default), every layout
   is bit-identical to pre-RFC: golden-hash unit sweeps, full suite
   green, STRESSGOLD `check` 9/9, and **zero golden re-blesses across
   the entire RFC** (achievable because nothing recalibrates — see "No
   recalibration"). Any S=1 diff is a threading bug — stop and fix
   before the phase proceeds. (Mirror of RFC-041 kill 2.)
2. **No blind-spot laundering.** The 60 EC/s one-stacked-belt headline
   counts as delivered **only** with direct evidence that every
   final-output tile's stamped rate ≤ its belt's physical cap × S. If
   it "passes" only because #311's unvisited merger tiles hide the
   overload, the headline is NOT delivered and the RFC must say so.
   *(Delivered: `stacking_ec_60s_red_one_belt_headline` audits every
   rate-stamped belt tile in-test — and proves the audit's teeth by
   running it on the S=1 layout, where it finds the #311 overload.)*
3. **Unstackable-site census bound.** *(Tripped and resolved
   2026-07-21 — the criterion worked: the review census missed the
   Fulgora D2b secondary output; implementation stopped, re-ran the
   census, found three unstackable classes and the kovarex
   mixed-family soundness hole, and replaced the per-lane guard with
   the family-exemption rule — see the Design section.)* Residual
   bound: if the family-exemption rule itself needs per-site special
   cases beyond the three census classes, stop — that is per-lane
   stackedness by another name, and it belongs to Phase 3.
4. **Recycler-ejection verification.** Phase 0 verifies (Recycler wiki
   page + in-game anchor) that recycler direct belt ejection stacks
   automatically at the researched S. If it does **not** — if it's
   conditional or capped — the `voider_row` / `scrap_recycling_row`
   output belts revert to unstacked capacity (documented
   conservatism); do not invent per-lane stackedness to keep the ×S
   credit there.
5. **In-game import anchor** (user-run, held open per RFC-037/041
   precedent — does not block merges): a stacked export imports into
   current Space Age and visibly builds stacks on belts at the
   researched size; the **EC@60/s red S=2** layout (the delivered
   headline fixture's config: normal quality, AM2, from ores) sustains
   ≥ 59 EC/s. Do NOT run the anchor on the legendary-express variant —
   it fails today for pre-existing reasons unrelated to stacking
   (Phase 3), and a failure there would misattribute.

## Verification plan

- Full suite from one clean invocation (single-run counts) +
  `SPAGHETTIO_STRESS_GOLDEN=check`.
- Differential fixtures *(as delivered — see decision log for the
  headline restatement)*: (a) EC@60/s **red, normal, from ore** S=2 —
  the headline, verified by an in-fixture per-tile capacity audit
  (stronger than the originally planned one-off snapshot decode: it
  re-runs on every suite run and proves its own teeth on the S=1
  layout); (b) the S=4 hand-4 rounding pin — delivered as ladder
  **unit tests** (`belt_drop_counts_track_hand_dip`: 20/s = 2
  inserters at S=3 but 3 at S=4) rather than a full-layout e2e
  differential, a documented granularity substitution; (c) S=1
  identity via the STRESSGOLD bit-identity gates run at every phase.
- Browser eyeball (user): stacked layout renders sanely. (Note: trunk
  count does NOT shrink vs S=1 — that expectation died with the
  full-belt descope; what changes visibly is belt tiers and the
  forced stack inserters.)
- In-game anchor: kill criterion 5 (user-run).

## Phasing

- **Phase 0 — ground truth + audits, zero behavior change.** Mechanics
  doc rules (cited); recycler-ejection verification (kill 4);
  `override_stack_size` parser-tolerance spot-check via
  blueprint-analyze; `stacking`-aware capacity helpers +
  `stack_inserter_swings` / `stack_inserter_belt_hand` landed but
  unconsumed. (The reach-2 census is already done — review finding 3.)
- **Phase 1 — plumbing + validator honesty.** `LayoutOptions.stacking`
  → `LayoutResult.stacking`; walker + inserter checks ×S; S=1
  identity gate (kill 1) before Phase 2 entry.
- **Phase 2 — layout + UX.** Belt tier selection, lane planner caps,
  output merger ×S; `DropTarget` threading through the ~37 template
  call sites, then uniform belt-drop forcing through the ladder;
  self-loop minor-lane guard (kill 3); wasm `layout*` params, URL
  `s=`, sidebar dropdown; differential fixtures + headline snapshot
  (kill 2).
- **Phase 3 — deferred.** Lane-aware tap delivery + the #312 fan-in
  wall lift at S>1 (demoted from Phase 2 — full-belt thresholds on
  tap-delivered flow are unsound, see decision log); the
  legendary-express@60 headline variant (blocked by pre-existing
  high-rate residuals independent of stacking: a junction-solver
  failure near dense crossings and ~3% lane-rate walker overshoot on
  zero-headroom lanes at exact tier boundaries — no issue number yet,
  file one at pick-up); per-lane stackedness (mixed economies);
  inserter capacity-bonus research axis; renderer stack visuals.

## Decision log

- **2026-07-21 — Code-lens implementation review:
  APPROVE-WITH-CHANGES; MAJOR finding fixed.** The reviewer re-ran
  every gate independently (suite, STRESSGOLD, all four fixtures,
  wasm build + tsc + web tests), verified the full-belt descope's
  completeness site-by-site (including that placer's HS trunk-count
  geometry correctly stays frozen while only its belt-tier selection
  scales), and confirmed all 8 forcing sites are genuine output
  sides with no item cross-contamination. The MAJOR: both
  `check_lane_throughput`s and the walker's splitter cap applied
  blanket layout-wide ×S — trusting the planner's exemption
  discipline instead of verifying it, precisely the fiction-agreement
  trap scoped to exempt lanes. Fixed: the validators re-derive
  `StackingCtx` from the SolverResult they already receive and key
  each tile's cap by its `carries` item (`for_item`), falling back to
  the layout-wide value only for unattributed tiles; an over-planned
  exempt lane now rates at unstacked capacity no matter what the
  planner believed. Also folded: the missing serde pin for
  `LayoutResult.stacking` (`layout_result_stacking_serde_contract` —
  missing field ⇒ 1, ≤1 fieldless, >1 round-trips). Acknowledged
  without code change: the kovarex e2e fixture is a smoke test (the
  exemption's real coverage is `stacking_ctx`'s unit tests; a
  rate-stressed mixed-family fixture is Phase 3 material with
  lane-aware taps). Post-fold gates: suite 835/0/36 (one clean run),
  STRESSGOLD 9/9, lib clippy clean.
- **2026-07-21 — Honesty-lens implementation review:
  APPROVE-WITH-CHANGES; all findings folded.** Every quantitative
  claim survived independent re-derivation (suite 834/0/36, STRESSGOLD
  9/9, empty golden diff, the 100→380 forcing differential and the
  fan-in refusal both reproduced from scratch). The blocker was
  narrative staleness: Summary/kill-2/kill-5/verification-plan still
  described the superseded legendary-express headline as delivered —
  exactly the quiet-weakening failure mode the lens exists for. Fixed:
  Summary restated to the delivered red/normal headline; kill 5
  rescoped so the user-run anchor targets the *verified* config (a
  legendary-express in-game failure would misattribute); the 10-vs-8
  forcing-site count bridged (2 self-loop major sites are
  exemption-guaranteed passthroughs); status Draft→Complete (registry
  + header); the stale Phase-1 field comment and headline-flavored
  unit-test comment corrected; verification-plan items annotated with
  their as-delivered granularity (in-fixture audit > one-off snapshot
  decode; hand-dip pin as unit test); the legendary-express residual
  promoted from log prose to a Phase 3 bullet. Code-correctness lens
  review ran in parallel — see the following entry.
- **2026-07-21 — Phase 2 landed; second simplification falsified and
  descoped (full-belt ×S on tap-delivered flow).** The first S=2
  differential runs failed honestly: scaling the consumer-clamped
  fan-in wall and the K-trunk retirement (both full-belt-cap-based)
  collapsed trunk counts, and tap/sideload-delivered flow — which
  fills ONE lane (B8/I5) — then exceeded per-lane stacked capacity
  (walker-caught 18/s on a 15/s stacked yellow lane; probe-verified
  the trunk and row-input belts carry everything on the near lane).
  The S=1 fan-in wall had been accidentally *shielding* this
  single-lane geometry gap by refusing such configs first. Resolution:
  **full-belt delivery thresholds stay unscaled** (trunk-count
  geometry at S>1 matches S=1); ×S remains on per-lane thresholds,
  belt tier selection, merger capacity, and forced-stack output
  throughput. The #312 wall lift is demoted to Phase 3 alongside
  lane-aware tap delivery; a parity fixture pins the conservative
  behavior and says exactly when to flip it. **Headline restated
  accordingly**: the delivered fixture is the #311 stress config —
  EC@60/s red from ore, whose committed golden stamps 60/s onto a
  30/s merger belt with zero warnings — made *physically valid* at
  S=2 (red stacked = 60/s): 0 errors, 0 warnings, and a kill-2 direct
  audit of every rate-stamped belt tile against stacked capacity,
  with the same audit proving its own teeth on the S=1 run (finds the
  #311 overload) and a stack-inserter count differential (100 → 380)
  proving the forcing engaged. The legendary-express@60 variant
  remains gated by pre-existing high-rate residuals (a junction-solver
  failure and ~3% walker overshoot on zero-headroom lanes at exact
  tier boundaries) — characterized, out of scope, noted for the
  Phase 3 pick-up. Also landed: `max_inserter_tier < Stack` refusal
  fixture, kovarex family-exemption fixture (stack-inserter census
  S=1 == S=2 on the exempt chain), forcing wired into the 8 stackable
  output sites (exempt sites keep `size_side` with the invariant in
  comments), and the wasm/web UX slice (URL `st=`, sidebar select,
  worker bridge). Gates: suite 834/0/36 (one clean run), STRESSGOLD
  9/9 bit-identical at S=1.
- **2026-07-21 — Kill 3 tripped mid-Phase-2; design amended (v3).**
  The Fulgora D2b secondary output is a second reach-2 belt-drop the
  spec review's census missed; re-census (all `size_side` sites +
  all direct inserter `PlacedEntity` pushes in templates.rs)
  additionally surfaced the sushi sort inserters (already
  kill-4-consistent) and, decisively, the kovarex **mixed-family
  soundness hole**: minor export and stacked major share one item ⇒
  one family ⇒ mixed stacked/unstacked trunks, where uniform ×S
  crediting is arithmetically wrong (fractional occupancy). The v1
  per-lane guard was replaced by the **static family-level stacking
  exemption** (see Design), which restores uniform-credit soundness
  by construction. Also resolved out-of-spec: `stacking > 1` with
  `max_inserter_tier < Stack` is an incoherent config → named refusal
  at layout entry, never silent degradation. Near-output forcing
  census: 10 `Reach::Near` output `size_side` sites across the row
  templates; 8 got the belt-drop entry point — the other 2 are
  `self_loop_row`'s major-output sites, which stay on `size_side`
  because the whole self-loop family (major and minor share the item)
  is exemption-guaranteed a passthrough (stated in a comment at the
  site). The 2 far-output sites and the sushi sorters likewise stay
  unforced (their families are exempt).
- **2026-07-21 — Phase 1 landed (plumbing + validator honesty).**
  `LayoutOptions.stacking` (manual `Default` impl so the neutral value
  is literally 1) → recorded as `LayoutResult.stacking` (serde: skip
  ≤1, missing = 1, so pre-RFC snapshots deserialize unstacked; the
  derived-`Default` 0 on hand-built/parsed results is documented as
  "≤1 = unstacked" and every consumer clamps). Validator sites
  threaded: both `check_lane_throughput`s (belt_structural +
  belt_flow) rate lanes at `lane_capacity_stacked(belt,
  layout.stacking)`, the walker's splitter per-output cap uses
  `belt_throughput_stacked` (BS4), and the two inserter-attribution
  checks rate machine-extraction (belt-drop) sides via a new
  `belt_drop_throughput` helper — flat I8 constant at S≤1 and for
  non-stack inserters, `swings × belt-hand` decomposition for stack
  inserters at S>1 (including the 9.6/s S=4 dip). `check_belt_throughput`'s
  overlapping-route message was left unscaled: it reports tier
  capacity in an overlap warning, no rate comparison. Wasm/UI
  deliberately untouched (quality-Phase-1-style guard rail: no
  deployed layout can set S≠1 yet). Kill-1 gate passed: full suite
  820/0 (36 ignored, one clean invocation) + STRESSGOLD check 9/9
  bit-identical. (A stale-artifact linker failure after two killed
  background cargo runs was cleared with `cargo clean -p
  spaghettio_core` — toolchain-internal undefined-symbol signature,
  not a code fault; `target/tmp` zone cache untouched.)
- **2026-07-21 — Phase 0 landed; kill 4 resolved on its conservative
  branch.** Recycler wiki page verified: recyclers DO stack onto belts
  mining-drill-style once stack-inserter tech is researched — but only
  "if the recycler has more than one of an individual item type in its
  inventory," which for probabilistic multi-product outputs (scrap
  recycling) is not guaranteed per item. Full-S crediting there would
  be fragile in the unsafe direction (plan > real), and no current
  fixture comes near even unstacked capacity on a recycler output
  belt — so **`voider_row` / `scrap_recycling_row` output belts keep
  S=1 capacity** (kill 4's documented-conservatism branch, taken on
  partial rather than failed verification). Revisit trigger: a real
  Fulgora fixture that saturates an unstacked recycler output belt.
  Mechanics rules landed as BS1–BS7 in `factorio-mechanics.md` (BS7
  records the buffering caveat). Capacity helpers +
  `stack_inserter_swings` / `stack_inserter_belt_hand` landed additive
  and unconsumed with unit tests (S=1 identity sweep, headline tier
  selections, the S=4 hand dip, quality composition, clamps).
  `override_stack_size` import tolerance pinned by a parser unit test
  (serde ignores unknown fields — no `deny_unknown_fields` anywhere in
  the parser — but the test keeps it true).
- **2026-07-21 — Adversarial spec review: APPROVE-WITH-CHANGES; v2
  folds all findings.** Two blockers, both accepted. (1) The hand
  5→6 recalibration was self-contradictory with kill 1 — the flat
  `inserter_throughput` constant isn't conditioned on S and the ladder
  already places stack inserters at S=1, so the bump would have moved
  the very baseline the identity gate measures, *and* was scheduled
  (Phase 0) before the gate (Phase 1). Resolution: no recalibration;
  swings×hand decomposition is additive-only, active only for
  belt-drop sizing at S>1. (2) Recyclers (and big mining drills) are
  stack creators per the wiki — and `voider_row` /
  `scrap_recycling_row` eject onto belts with **no inserter**, so the
  forcing rule had nothing to force there; the "every engine-loaded
  belt is stack-loaded" premise now enumerates its three load paths
  explicitly, recycler ejection provisionally credited ×S under a new
  kill 4 (verify-or-revert-to-unstacked). Majors: the reach-2 census
  is *done*, not deferred — `self_loop_row` minor output is the single
  long-handed belt-drop (descope-with-guard chosen over re-slotting:
  kovarex minor rates are ≪ unstacked capacity, guard refuses with a
  named error if that ever changes); `size_side` can't see drop
  targets today → `DropTarget` threading scoped as its own Phase 2
  step (~37 call sites); `stack_inserter_swings(quality)` named as the
  concrete decomposition helper. Minors: fallback arithmetic error
  (floor(5/S)·S is 5,4,3,4 — moot after resolution 1) and the
  blueprint field is `override_stack_size`, not "stack_size_override".
  Review confirmed all other mechanics rules verbatim against the
  wiki, the capacity-helper signatures, the 13-file consumer census,
  and that ghost_router/output_merger place no inserters (the forcing
  rule needs no router coverage).
- **2026-07-21 — RFC drafted.** Number claimed as RFC-046 per registry
  (`docs/rfcs.md`); also repaired the registry's duplicate
  "Next number" line left by the RFC-044/045 collision resolution.
  Wiki mechanics fetched fresh (Stack_inserter,
  Transport_belt_capacity_(research)) rather than trusted from memory;
  the S=4 hand-rounding drop (6→4 per swing) and the "only stack
  inserters create stacks" constraint are the two facts that shaped
  the design (forcing rule + belt-drop hand model). Uniform forcing
  chosen over per-lane stackedness for Phase 1-2; recorded as the main
  simplification bet — revisit trigger is a real mixed-economy request.
