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
2. **Only stack inserters create stacks**, and only when dropping onto
   a belt. Drops into machines/chests are exact-hand (no rounding).
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
   belt-drop side cannot contribute stacked flow, ever.
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
- `stack_inserter_belt_hand(stacking) = floor(6 / S.max(1)) * S` …
  effectively 6, 6, 6, 4 — used by the ladder for belt-drop sides.

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
> `swings(quality) × stack_inserter_belt_hand(S)`).

With uniform forcing, *every* engine-loaded belt is stack-loaded, so
×S applies uniformly in planning and validation with no per-lane
stackedness bookkeeping. External input trunks are assumed stacked at
the boundary (documented UI assumption — the user's mall feeds them;
if it doesn't, real < plan on externals only).

Rejected alternatives:
- **Per-lane stackedness tracking** (only force where a lane plans
  above unstacked cap): strictly better hardware cost, but mixed
  stacked/unstacked flows sharing trunk segments turn capacity
  accounting per-segment; deferred to Phase 3, not needed for the
  endgame profile this serves.
- **Capacity-only change without forcing**: dishonest; killed on
  sight.

**Known hole — reach-2 belt-drops (ground rule 5)**: if any active row
template has a *long-handed* inserter dropping onto a belt, that side
cannot stack and uniform-×S is wrong for its lane. Phase 0 audits the
templates; if such sides exist, the options are re-slotting (near-side
swap, as in I8a's layout consequence) or capping those rows'
contribution at unstacked rate. See kill criterion 3.

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

### Stack-inserter base hand: 5 → 6 recalibration

Table I8 deliberately kept hand 5 (~12/s) vs the wiki's 6 (~14.4/s),
"revisit only with evidence." The belt-drop rounding **is** that
evidence: `floor(5/3)*3 = 3` per swing at S=3 vs the real 6 — a 2×
under-credit, far past the ≤20% headroom that justified keeping 5.
This RFC bumps the stack-inserter row to hand 6 / 14.4/s base
(`common::inserter_throughput`), leaving regular / long-handed / fast
untouched. Expected fallout: ladder threshold shifts where stack
inserters are chosen → a handful of golden re-blesses. Kill criterion
4 bounds this.

### Export

No new blueprint fields expected: stack inserters stack to the force's
researched max by default. Phase 0 verifies against a community
blueprint (blueprint-analyze) that no `stack_size_override`-style
field is required; the in-game anchor (kill criterion 5) is the final
word.

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
   green, STRESSGOLD `check` 9/9. Any S=1 diff is a threading bug —
   stop and fix before the phase proceeds. (Mirror of RFC-041 kill 2.)
2. **No blind-spot laundering.** The 60 EC/s legendary headline counts
   as delivered **only** with a decoded snapshot showing every
   final-output tile's stamped rate ≤ its belt's physical cap × S. If
   it "passes" only because #311's unvisited merger tiles hide the
   overload, the headline is NOT delivered and the RFC must say so.
3. **Reach-2 belt-drop audit bound.** If Phase 0 finds long-handed
   belt-drop sides in active templates and making them stackable
   (re-slotting) exceeds roughly a phase of work, descope: those rows
   cap at unstacked contribution, documented — do not redesign the
   template system inside this RFC.
4. **Recalibration containment.** If the hand 5→6 bump cascades beyond
   stress-golden re-blesses plus a handful of fixture updates (i.e. it
   changes behavior on paths that never place stack inserters), the
   recalibration is entangled somewhere it shouldn't be — revert to
   hand 5 and model belt-drop rounding on 5 (6,4,3,4 per-swing) instead.
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
  doc rules (cited); reach-2 belt-drop template audit (kill 3);
  export-field check via blueprint-analyze; `stacking`-aware capacity
  helpers landed but unconsumed; hand 5→6 decision executed under
  kill 4.
- **Phase 1 — plumbing + validator honesty.** `LayoutOptions.stacking`
  → `LayoutResult.stacking`; walker + inserter checks ×S; S=1
  identity gate (kill 1) before Phase 2 entry.
- **Phase 2 — layout + UX.** Belt tier selection, lane planner caps,
  output merger ×S; uniform belt-drop forcing through the ladder;
  wasm `layout*` params, URL `s=`, sidebar dropdown; differential
  fixtures + headline snapshot (kill 2).
- **Phase 3 — deferred.** Per-lane stackedness (mixed economies);
  inserter capacity-bonus research axis; #312 composed fix; renderer
  stack visuals.

## Decision log

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
