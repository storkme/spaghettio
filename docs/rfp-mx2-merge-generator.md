# RFP: MX2 merge generator (phase 2 of throughput-priority merges)

## Summary

Phase 1 ([`rfp-throughput-priority-merges.md`](rfp-throughput-priority-merges.md))
confirmed the spec premise: 60/63 library templates are MX3 (composition-
balanced), which is stronger than the MX2 (throughput-unlimited)
property fucktorio's single-recipe-per-trunk bus actually needs.

This RFP scopes phase 2: build a Rust-side MX2 merge generator that, for
any `(m, n)`, produces a `BalancerTemplate`-compatible logical+physical
layout, verified MX2 by the existing classifier. Goals:

1. Provide significantly smaller templates for cases where MX3's
   composition guarantee is wasted (the dominant case).
2. Generate templates Factorio-SAT struggles with — coprime
   `(m, n)` like `(5, 9)`, `(7, 9)`, asymmetric `(8, 9)`, `(9, 10)` —
   without the SAT-runtime cost of [`rfp-balancer-runner.md`](rfp-balancer-runner.md).
3. Stop the audit-surfaced (5, 8) / (8, 6) MX1 latent-bug class:
   fresh templates classified MX2 by the verifier are correct by
   construction.

The dominant lever is **#135** (template oversize). Spec brief estimate:
3-5× fewer splitters than MX3 for asymmetric cases.

## Motivation

### Where the existing library hurts

- **#135 — oversize.** `cargo test --lib audit_report -- --nocapture`
  shows footprint per template:
  - `(5, 6)`: 148 entities, 175 tiles
  - `(6, 5)`: 172 entities, 208 tiles
  - `(8, 7)`: 174 entities, 207 tiles
  - These are the bands that drive recipe-row gaps in
    `compute_extra_gaps` (`bus/layout.rs:463`). A 200-tile band per
    item-type transition is a major layout-area cost.
- **Coprime gaps and tier 9-10.** Library covers `(1..8) × (1..8)`; the
  SAT runner work targets `(1..10) × (1..10)`. Tier 5/6 layouts will
  soon demand shapes like `(9, 10)` and bigger.
- **Latent bugs**: (5, 8) and (8, 6) are MX1 in the current library
  ([#266](https://github.com/storkme/fucktorio/issues/266)). A
  classifier-verified generator can replace them with correct MX2.

### Why MX2 not MX3

Per [`docs/factorio-mechanics.md` — belt merger taxonomy](factorio-mechanics.md#belt-merger-taxonomy-mn-splitter-networks):
fucktorio's bus has one item per trunk (single-recipe-per-row by
construction). Composition guarantees (MX3) are wasted; the only
load-bearing property is throughput-unlimited (MX2): every k-subset of
inputs can simultaneously route to k outputs.

## Design

This is the central section. Several interlocking choices need to be
made before any code lands; each has a default I'd take if pushed, and
a short rationale.

### D1 — How strong an MX2 do we need?

The spec brief lists three levels of "throughput-priority":
1. **Saturation under all-saturated input.** Outputs are saturated
   when inputs are saturated. *Property of any feedforward tree.*
2. **Equal output rate under uniform input.** Each output runs at
   `total_in / n`. Required for fucktorio's bus (homogeneous consumer
   rows must not see uneven feed rates, or some assemblers idle).
3. **Max-flow under partial output blockage.** Inputs reroute around
   blocked outputs. Requires cross-mixing or back-loops.

For fucktorio's bus design, output blockage is *uniform*: all consumer
rows for a recipe consume at the same rate. So (3) isn't load-bearing.
**(1) and (2) together** are the target — and (2) is what makes
non-divisible `(m, n)` non-trivial.

For divisible `(m, n)` (`n` divisible by `m` or vice versa), pure
feedforward `1→k` trees give us both (1) and (2) with `n - m` splitters
(linear in problem size). For non-divisible `(m, n)` we need
*cross-mixers* — splitters that combine streams from multiple inputs
to even out the per-output rate.

**Default choice:** target properties (1) + (2). Skip (3). That's still
true MX2 by the classifier's max-flow check (which, recall, tests both
input subsets and output subsets — so if we pass the input-subset half
without (3), the classifier will land us at MX2 not MX3 only when the
construction has property-(2)-but-not-(3) cross-mixers).

Wait — that's actually a subtle point. The classifier tests both
directions of Menger. A construction with property (1)+(2) but not (3)
will *fail* the output-subset half of the MX2 check and be classified
MX1. So the classifier as currently built can't distinguish "MX2
properties (1)+(2)" from "MX3" — it'd label our intended target as MX1.

**Resolution options:**
- **D1a.** Relax the classifier: split MX2 into MX2a (saturation +
  balanced-rate, weaker) and MX2b (full max-flow, stronger). Update the
  taxonomy doc.
- **D1b.** Bite the bullet and build property-(3) too. The construction
  cost is higher but the verifier's already there.

I'd default to **D1a** (relax the classifier). MX2b is over-specified for
fucktorio's bus, and the cleaner taxonomy makes the audit results more
useful.

### D2 — Logical-graph generator vs direct physical layout

The spec brief favors generating a logical splitter graph and handing
it to SAT for placement. After phase 1, two practical considerations
push the other way:

- **Existing code path.** `bus/balancer.rs::stamp_family_balancer`
  reads `BalancerTemplate { entities, input_tiles, output_tiles, ... }`
  — physical entity positions. To plug in, we need a physical layout,
  not just a graph.
- **SAT placement is what we're trying to skip.** If we generate a
  graph and then SAT-place it, we lose half the speed advantage the
  brief promised.

**Default:** generate physical layouts directly from a small library of
hand-coded patterns ("atoms"). For a given `(m, n)`:
1. Decompose into a sequence of atoms (e.g., `(1, 2)`, `(1, 3)`,
   `(2, 2)` Benes-mixer, `(2, 3)`).
2. Stamp each atom at fixed grid coordinates with belt routing
   between them.
3. The classifier verifies the result.

The atoms are tiny (1-3 splitters each), so the library is small and
hand-auditable.

### D3 — Decomposition strategy

Given target `(m, n)`, build the network how?

**Default approach: prime-factor decomposition with mixers.**

For each input, allocate `q = n / m` or `q + 1` outputs (where
`r = n mod m` of the inputs get `q + 1`).

If `r == 0` (divisible): pure feedforward, `m` parallel `1→q` trees.
Smallest possible. ~`(q-1) * m = n - m` splitters.

If `r != 0` (non-divisible): we need cross-mixers to even the rates.
The simplest pattern:
1. First layer: `m` parallel `1→2` splitters → `2m` belts at rate
   `1/2`.
2. Mixer layer: combine pairs of `1→2` outputs from *different inputs*
   via `2→2` Benes-mixers. Each mixer takes 2 belts (each from a
   different input) and produces 2 belts each at the average rate.
3. Repeat until reaching exactly `n` outputs at rate `m/n`.

Number of splitters: O(m + n + non-divisibility-cost). For `(4, 9)`,
hand estimate: ~6-8 splitters vs the existing library (would be
~70+ if it existed; current decomposition fallback uses 2× `(2, 4)` +
1× `(0, 1)` which doesn't cover the shape).

I'm not going to claim this is optimal — that's what the kill criteria
below check.

### D4 — Verification

Reuse the existing classifier. For each generated `(m, n)`:
1. Build the template entities + input/output tiles.
2. Call `classify(&template)`.
3. Assert class is `MX2 (a or b per D1)` or `MX3`. If `MX1`, the
   generator is buggy.

### D5 — Integration

`balancer_templates()` is currently a `OnceLock<FxHashMap>` populated
once from baked-in `BalancerTemplate` constants. To add generated
shapes:

- **Default**: change the lookup path in `balancer.rs::stamp_family_balancer`
  to call a new `bus::balancer_generate::generate(m, n)` *after* the
  baked-library miss + decomposition fallback. Generated templates are
  `OnceLock`-cached per `(m, n)` so we don't regenerate per stamp.
- The generator stays Rust-side (no Python / no SAT). Build-time cost:
  zero. Runtime cost: ~ms per shape, amortised behind the cache.

This means we *don't* replace existing templates wholesale. We add
generated shapes for cases the library lacks, and have a separate
follow-up to swap library entries for smaller generated equivalents
once we trust the generator.

### Trade-offs considered

- **Replace the library wholesale right now.** Rejected. The existing
  library works for everything `(1..8) × (1..8)` minus the (5, 8) and
  (8, 6) bugs; risking regressions for footprint wins isn't worth it
  on day one. Land the generator beside the library, swap selectively
  later.
- **Generate during balancer_library.py instead of Rust.** Rejected.
  We're trying to escape the Python+SAT pipeline. Rust generation is
  cheap, deterministic, and deployable to the WASM build with zero
  extra dependencies.
- **Skip property (2) and ship pure feedforward.** Tempting (smallest
  possible templates), but breaks bus uniformity for non-divisible
  shapes. Tier-5/6 will see plenty of those. Worth the cross-mixer
  cost.

## Kill criteria

- **Generator can't produce a verified MX2 template for `(4, 4)`.**
  This is the smallest non-trivial case — divisible, fully symmetric.
  If we can't even hit this, the construction is wrong.
- **Generated `(4, 9)` is bigger than 50 entities or 80 tiles.** Spec
  brief estimate is 3-5× fewer splitters than MX3; library MX3
  templates of comparable shape are 70-150 entities. If we land in the
  same ballpark, the project's value prop evaporates and we should
  reconsider.
- **A generated template trips the classifier as MX1.** The whole
  point is correctness-by-construction. Any MX1 generation is a
  generator bug; fix or kill.
- **Generated WASM binary size grows by >100 KB.** The existing
  library is bulky (4400+ lines of static data). If our generator
  adds significant weight without removing the library, the WASM
  bundle suffers.
- **Stamp call latency >5ms in WASM.** Per `web/src/engine.ts`
  expectations the layout computation is interactive. A heavy
  generator on the hot path is a regression.
- **D1a relaxation is ambiguous.** If splitting MX2 into MX2a/MX2b
  produces a taxonomy that's harder to reason about than the current
  three-tier split, we keep the original taxonomy and require D1b
  (full property-3 networks) — at the cost of larger generated
  templates.

## Verification plan

1. **Unit test per atom.** Each atom (e.g., `(1, 2)`, `(1, 3)`,
   `(2, 3)`) classified MX2 or MX3 by the existing classifier.
2. **Smoke over all `(m, n)` for `m, n ∈ 1..10`.** Generate, classify,
   assert MX2-or-MX3. Compare entity-count / footprint against the
   library where present.
3. **In-game spot-check.** Stamp a generated `(4, 9)` in a real
   blueprint. Saturate inputs, observe output throughput. Repeat with
   one input dropped to half rate — outputs should each drop to
   `(input_total) / n`.
4. **e2e regression.** Run the full layout suite. Any bus that
   previously worked with library templates must continue working
   when generated templates take over for missing shapes (the
   generator is additive in this RFP — we don't replace existing
   library entries yet).
5. **Trace events.** Add a `BalancerGenerated { shape, entities,
   class }` trace event. Useful for the snapshot debugger and for
   measuring the generator's reach in real layouts.

## Phasing

- **Phase 2.0** (this RFP): atoms + decomposition + classifier-gated
  generation, behind the library miss in `stamp_family_balancer`. No
  library replacement.
- **Phase 2.1** (separate RFP): swap library entries with smaller
  generated equivalents. Gated on phase 2.0 stability (plus
  in-game verification of representative shapes).
- **Phase 2.2** (separate RFP): replace `scripts/generate_balancer_library.py`
  + Factorio-SAT entirely once 2.1 has shipped. Closes
  [#135](https://github.com/storkme/fucktorio/issues/135) and the
  rfp-balancer-runner work.
- **Out of scope:** lane-aware MX5 verification, mixed-content buses.

## Decision log

- *2026-05-01 — drafted. Awaiting user feedback on D1 (taxonomy
  relax-or-bite-the-bullet), D2 (physical-direct-vs-graph), D3
  (decomposition strategy), and the kill-criterion bounds.*
