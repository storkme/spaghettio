# RFC-046: Belt stacking (Space Age stacked belts)

Registry: [`rfcs.md`](rfcs.md). Status: **Draft** (2026-07-21).

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
Headline outcome: the original 60 EC/s legendary scenario fits **one**
stacked express belt (45 × 2 = 90/s ≥ 60/s), clearing the cap that
[#311](https://github.com/storkme/spaghettio/issues/311) imposed,
without waiting for the output-merger rework.

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
(`feedback_belt_tier_user_specified`). URL codec `s=2|3|4` (absent =
1), sidebar dropdown "Belt stacking: Off / 2 / 3 / 4". Threaded:
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

**Known hole — reach-2 belt-drops (ground rule 5)**: the census is
done (spec review, 2026-07-21) — the single long-handed belt-drop in
active templates is `self_loop_row`'s minor output. **Decision:
descope, guarded.** Kovarex-class minor-output rates are fractions of
an item/s, so unstacked capacity is never the binding constraint
today; rather than re-slotting the template or introducing per-lane
stackedness for one lane, lane planning gains a **minor-lane guard**:
if a self-loop minor-output lane ever plans above *unstacked* lane
capacity, refuse with a named `CandidateRun.error` (same pattern as
the #312 fan-in refusal) instead of silently over-crediting. The
guard turns an in-principle-unsound uniform credit into a checked
assumption. See kill criterion 3.

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
2. **No blind-spot laundering.** The 60 EC/s legendary headline counts
   as delivered **only** with a decoded snapshot showing every
   final-output tile's stamped rate ≤ its belt's physical cap × S. If
   it "passes" only because #311's unvisited merger tiles hide the
   overload, the headline is NOT delivered and the RFC must say so.
3. **Minor-lane guard bound.** The one reach-2 belt-drop
   (`self_loop_row` minor output) is descoped-with-guard, not
   re-slotted. If implementing the guard reveals more unstackable
   belt-load sites than that one (i.e. the review census was
   incomplete), stop and re-run the census before Phase 2 continues —
   do not add per-site exemptions ad hoc, and do not redesign the
   template system inside this RFC.
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
   researched size; the 60 EC/s legendary layout sustains ≥ 59 EC/s.

## Verification plan

- Full suite from one clean invocation (single-run counts) +
  `SPAGHETTIO_STRESS_GOLDEN=check`.
- Differential fixtures: (a) EC 60/s legendary express S=2 — the
  headline, snapshot-decoded per kill 2; (b) an S=4 case pinning the
  hand-4 rounding (output inserter counts *rise* vs S=3 at equal
  rate); (c) S=1 identity pair.
- Browser eyeball (user): stacked layout renders sanely; trunk count
  visibly shrinks vs S=1 at equal rate.
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
- **Phase 3 — deferred.** Per-lane stackedness (mixed economies);
  inserter capacity-bonus research axis; #312 composed fix; renderer
  stack visuals.

## Decision log

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
