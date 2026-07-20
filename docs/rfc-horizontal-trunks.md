# RFC: Horizontal trunks for input-bottlenecked rows

## Summary

Add a second row-layout strategy alongside the existing vertical-split
approach. When a recipe's per-machine input demand exceeds a single
input belt's capacity, the existing layout engine **vertically
partitions** the recipe into many short rows, then reassembles them
downstream with N→M balancers and bus trunks. This RFC proposes a
**horizontal layout** — one long row with K stacked input belts at
the top, each ending its own east-flowing trunk that dives via
underground belt to feed a dedicated machine block. Output is a
single continuous east-flowing belt that runs at full belt capacity.

The two strategies coexist as a per-call layout option (`RowLayout`
enum: `VerticalSplit`, `HorizontalStack`). Vertical-split remains the
default for backward compatibility. The user picks via a UI control.
Long-term, this becomes one of several layout-strategy axes the
engine can search across to optimise for compactness or aesthetics.

This RFC does **not** propose deprecating vertical-split — it has
genuine strengths (works for output-bottlenecked rows, narrower row
footprint at low rates, balancer-stamped output is well-tested) and
will continue to be the default for many recipes. The two strategies
solve the same problem with different geometric trade-offs.

## Motivation

The concrete failing case that prompted this exploration was
`processing-unit @ 2/s`,
URL `?item=processing-unit&rate=2&machine=assembling-machine-3&in=iron-plate,copper-plate,coal,water,crude-oil`,
where the lane planner was silently dropping a producer row whose
output ended at (24, 113) — see PR #217 for the tactical fix. That
fix improved the vertical-split path; this RFC introduces a different
geometric approach that sidesteps the structural cause altogether.

Beyond that one bug, the broader motivation is that vertical-split
fragments rows whenever the input belt is the bottleneck (very common
for high-demand intermediates like `electronic-circuit`,
`copper-cable`, `iron-gear-wheel`). Each fragment needs its own input
belts, output belt, return belt back to the bus, and a balancer block
to reconcile producer count vs trunk count. That's a lot of
machinery for a problem that disappears entirely if the row is laid
out horizontally instead.

The aesthetic motivation is more speculative: with two valid layouts
for the same subchain, the engine eventually has the option to
search across them and pick the result that's tighter, prettier, or
fits better in a given footprint. This RFC doesn't build that
search; it just unlocks it by establishing an alternative.

## Design

### Row geometry

For a recipe with N inputs, ranked by per-machine throughput
demand from highest (input₀) to lowest (input_{N-1}):

```
y+0..y+(K-1) : K stacked east-flowing trunks of input₀, each ending
                in a dive at its assigned sub-row boundary
y+K..y+K+N-2 : input₁ … input_{N-1} continuous east-flowing belts
                (low-demand inputs stay one-belt-each, no stacking)
y+K+N-1      : "current-feed" belt for input₀ (immediately above
                inserter row, fed by the trunk dives)
y+K+N        : inserter row (mixed types — handwaved per the
                "stack by throughput, near-slot only" rule)
y+K+N+1..y+K+N+3 : machine 3×3
y+K+N+4      : output inserter row
y+K+N+5      : output belt (single, full lane-split capacity)
```

The "near-slot is highest-throughput input₀" rule pins input₀ at
y+K+N-1 (one tile from the machine), reachable by a stack/bulk
inserter at reach 1. The other inputs sit further north and are
reached by long-handed inserters.

Per the inserter-throughput handwave (see "Out of scope" below), the
exact inserter type per slot is the user's responsibility for now.
The template emits one inserter per belt per machine; if throughput
is insufficient at the user's research level, that's a layout-time
research-level mismatch, not an engine bug.

### Sub-row boundary

Between sub-row N and sub-row N+1 (each block is `block_size`
machines wide, where `block_size = floor(full_belt_capacity /
per_machine_demand_input₀)`):

- One of the K trunks (typically trunk N+1) dives via a **south-axis
  UG pair** from y+? down to y+K+N-1, becoming the current-feed for
  the next sub-row.
- The low-demand belts at y+K..y+K+N-2 each go via **east-axis UG
  pair** across the boundary. Surface tiles in the gap row are empty
  on those rows, leaving room for the trunk dive without crossing.
- Iron-plate UG (east-axis) and trunk-dive UG (south-axis) are on
  perpendicular axes — they don't interfere underground (per F4/F7).

### Code touched

- New: `crates/core/src/bus/layout.rs` — add `pub enum RowLayout {
  VerticalSplit, HorizontalStack }` and a field on `LayoutOptions`.
  Default is `VerticalSplit`.
- `crates/core/src/bus/templates.rs` — new template variants for
  `HorizontalStack` for at least dual-input solid recipes (the most
  common case). Triple-input and fluid-input variants in later
  phases.
- `crates/core/src/bus/placer.rs` — branch on `RowLayout`; for
  `HorizontalStack`, skip vertical splitting and emit a single long
  row with sub-row blocks.
- `crates/core/src/bus/lane_planner.rs` — for `HorizontalStack`
  rows, the K trunks each need a bus tap-off at the row's west end;
  no balancer family needed (the row consumes its own outputs into
  one consolidated belt).
- `crates/core/src/bus/ghost_router.rs` — bus tap-offs deliver to
  the K trunk entry points at y+0..y+(K-1). Within-row routing is
  template-stamped, not router-routed (the structure is regular
  enough).
- `web/src/ui/sidebar.ts` — add a `RowLayout` dropdown (or
  checkbox), persisted in URL state alongside `strategy` and `belt`.
- `crates/core/tests/e2e.rs` — new tier3 / tier5 test cases with
  `RowLayout::HorizontalStack` — separate goldens.

### Trade-offs vs. vertical-split

| Property | Vertical-split (today) | Horizontal-stack (this RFC) |
|---|---|---|
| Row count | one per ~6 machines | one (regardless of size) |
| Footprint shape | tall and narrow | short and wide |
| Return belts | yes, routed via ghost-router | no (output belt fed directly) |
| Balancer block | yes (N→M template) | no |
| Tap-offs from bus | per output trunk | K per row (one per stacked trunk) |
| Output capacity | bound by per-row output belt | always full belt by construction |
| Output-bottlenecked recipes | works | doesn't help (output belt is the limit either way) |
| UG tunnel reach pressure | low | moderate (sub-row gap dive paths) |

**This RFC is not a replacement.** Output-bottlenecked rows
(uncommon at vanilla rates but real) still need vertical-split or
some other approach. Vertical-split is also a more compact choice
when the recipe is small enough to fit a single row.

### Trade-offs vs. PR #217 (balancer-family fan-in)

PR #217 fixes a correctness bug in vertical-split where producer
rows could be silently dropped when `n_splits > consumer_count`.
That fix is *necessary* regardless of this RFC — vertical-split
needs to work correctly. Horizontal-stack just offers a different
layout shape; it doesn't replace the balancer machinery.

## Kill criteria

This RFC should be abandoned or rethought if any of the following
trips:

- *After landing phase 1 (dual-input solid recipes), the
  `processing-unit @ 2/s` URL with `RowLayout::HorizontalStack`
  produces a layout that is wider than 1.5× the equivalent
  vertical-split layout's bounding box AND fewer than 50% of the
  validator errors are resolved.* — would mean the new approach is
  actively worse for this benchmark.

- *After implementing the sub-row boundary template (the trunk dive
  + low-demand UG cross), more than 10% of the test corpus requires
  UG dive depth > 8 (express UG max reach).* — would mean the
  geometry has hidden constraints we didn't account for.

- *The `templates.rs` additions exceed ~600 LOC per row kind.* —
  too much surface for one strategy variant; reconsider whether a
  shared template-builder abstraction is warranted first.

- *End-to-end runtime for `cargo test --manifest-path
  crates/core/Cargo.toml` regresses by more than 1.5× on the
  existing test corpus.* — the new code path is more complex than
  expected.

- *After phase 1, the validator catches structural correctness bugs
  in `RowLayout::HorizontalStack` layouts that don't appear in
  `RowLayout::VerticalSplit` for the same recipe — and resolving
  them requires changes outside templates / placer.* — would mean
  the new strategy is leaking complexity into bus / ghost-router
  / validator that we'd be paying for forever.

## Verification plan

Per
[CLAUDE.md verification protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Full e2e suite green for both `RowLayout` variants.** New
   tier3 / tier5 tests with `RowLayout::HorizontalStack` get their
   own golden hashes. Tests with `RowLayout::VerticalSplit`
   (default) keep today's hashes.
2. **The processing-unit URL renders cleanly with both
   strategies.** Open
   `?item=processing-unit&rate=2&machine=assembling-machine-3&in=iron-plate,copper-plate,coal,water,crude-oil`
   in the dev server, switch the new `RowLayout` dropdown to
   `HorizontalStack`, confirm zero belt-dead-end errors and zero
   balancer-related errors. Switch back to `VerticalSplit`,
   confirm previous behaviour preserved.
3. **Snapshot inspection on three sample layouts under each
   strategy:**
   - Tier3 `electronic-circuit` from ores at 5/s.
   - Tier4 `advanced-circuit` (which produces ec internally).
   - Tier5 `processing-unit` at 1/s and 2/s.
4. **Trace events.** New `RowLayoutSelected { recipe, kind }` event
   per row, so the snapshot debugger can render strategy choice
   per-row. Verify the event fires for every row and the kind is
   consistent with the option flag.
5. **Visual sanity in browser.** Switch strategies; the layout
   should be visibly different (wider/shorter or
   taller/narrower) but produce equivalent products at the
   same rate.

## Out of scope

- **Inserter throughput math.** The "stack by throughput,
  highest-demand nearest" rule is the only inserter-related
  constraint encoded. Whether the user has the right inserter
  research level to actually achieve a given throughput is the
  user's problem; the engine just emits "the right number of
  inserters of the natural type for that slot."
- **Search / auto-selection across strategies.** The UI exposes a
  manual choice. Future work could try multiple strategies per
  recipe and pick the most compact / most aesthetic, but that's a
  separate RFC.
- **Triple-input and fluid-input recipes.** Phase 1 covers
  dual-input solid only (e.g., `electronic-circuit`,
  `iron-gear-wheel`, `copper-cable`). Phases 2+ extend to other
  shapes.

## Test strategy

Both `RowLayout` variants must remain green throughout the
multi-strategy phases. Tests parametrise as follows:

- **Input-bottlenecked recipes.** Add explicit `_horizontal`
  variants alongside the existing tests, each with its own
  golden hash. Examples: `tier3_electronic_circuit_horizontal`,
  `tier4_advanced_circuit_horizontal`,
  `tier5_processing_unit_horizontal`.
- **Non-input-bottlenecked recipes.** Where the recipe collapses
  to a single row regardless of strategy (e.g.
  `tier1_iron_gear_wheel @ 10/s`), a single golden suffices. Add
  an explicit equivalence assertion: same product output,
  semantically equivalent layout. The validator should agree.
- **Coverage during default-switch (phase 4).** When the default
  flips to `HorizontalStack`, the existing tests' golden hashes
  regenerate en masse against the new default. Keep the
  `_vertical` variants permanently — they guard against
  accidental loss of vertical-split, which is still needed for
  output-bottlenecked recipes and remains a valid alternative.

The transition risk is real: doubling the corpus during phases 1-3
and the big-bang regen at phase 4 are the visible costs the user
is signing up for in exchange for layout-strategy flexibility.

## Phasing

1. **Phase 1 — `RowLayout` plumbing + dual-input solid.** Add the
   enum and option field. **Default stays `VerticalSplit`** so
   existing tests are untouched. Implement `HorizontalStack` for
   `RowKind::DualInput` recipes only. UI dropdown wired. Add
   new `_horizontal` test variants with their own goldens. Kill
   criteria above apply at end of phase 1.
2. **Phase 2 — single-input + triple-input solid recipes.** Same
   structure: extend the strategy to more row kinds, add
   `_horizontal` variants of the relevant tier tests.
3. **Phase 3 — fluid-input + multi-fluid rows.** Significantly
   more complex because the fluid-trunk dive shares the row's
   top space with solid-belt dives. By the end of phase 3,
   horizontal-stack covers every row kind currently supported
   by vertical-split.
4. **Phase 4 — switch default to `HorizontalStack`.** Flip the
   default in `LayoutOptions::default()`. Regenerate the
   non-`_vertical` golden hashes en masse. The existing
   tier1–tier3 tests now exercise the horizontal path; the
   permanent `_vertical` variants keep guarding the legacy
   strategy. Kill criterion at this phase: if regenerated layouts
   show a measurable regression in entity-density or
   bounding-box compared to the previous defaults across more
   than 25% of the corpus, hold the switch.
5. **Phase 5 (later, separate RFC) — multi-strategy search.**
   Engine tries multiple strategies per recipe, picks the
   better-scoring layout per some metric (compactness, belt
   utilisation, aesthetic).

## Decision log

- *2026-04-25 — proposed.*
- *2026-04-25 — revised after design discussion: corrected
  geometry (high-demand belts are the current-feed adjacent to
  machine, low-demand stacked above; trunks at top dive via
  S-axis UG to current-feed, low-demand crosses sub-row boundaries
  via E-axis UG so the two don't interfere); reframed as a
  coexisting alternative to vertical-split rather than a
  replacement; inserter-throughput math punted out of scope.*
- *2026-04-25 — refined: horizontal-stack will become the default
  in phase 4 once tested across all row kinds. Permanent
  `_vertical` test variants guard the legacy strategy. Test
  strategy section added.*
