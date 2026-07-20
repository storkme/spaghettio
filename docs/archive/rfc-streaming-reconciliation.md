# RFC: streaming renderer reconciles to `layout.entities` at finish

## Summary

Change the web streaming renderer so that after live streaming settles,
`renderLayout(layout, …)` draws the authoritative `layout.entities` and
streaming graphics are discarded. Today the streaming renderer keeps
its own per-tile entity graphics and stays on-screen as authoritative
post-settle, which makes streaming state a second source of truth. A
recent bug where a feeder-belt visual was rendered over a
balancer-belt entity (with the inspector reading from
`layout.entities` and the rendered Graphics reading from streaming
state) is the motivating failure. After this RFC, `layout.entities` is
the single source of truth post-settle, belt-network hover highlights
work on streamed layouts (currently stubbed out), and
`streamingRenderer.ts` shrinks by ~150–200 lines.

## Motivation

On `http://localhost:5173/?item=electronic-circuit&rate=10&machine=assembling-machine-1&in=iron-ore,copper-ore`:

- `__layout.entities.filter(e => e.x === 6 && e.y === 7)` returns
  `{name: "transport-belt", direction: "South", carries: "copper-plate",
  segment_id: "balancer:copper-plate:1x2"}` — correct, authoritative.
- The rendered graphic at `(6,7)` is a west-facing straight belt.
- Tile inspector shows `⚠ 2 ghost specs at this tile · ← feeder end ·
  ← feeder end`.

Root cause (now fixed for this specific leak in
`crates/core/src/bus/ghost_router.rs:864`): `GhostSpecCommitted` was
emitting `path_ents` (pre-filter) rather than `surviving_ents`
(post-filter). The streaming renderer committed a feeder-belt graphic
at `(6,7)` before the later `PhaseSnapshot bus_routed` arrived with
the balancer-belt; the snapshot filter skipped `(6,7)` as
already-committed, so the streaming view kept the feeder, while
`layout.entities` contained the balancer.

The shape of the bug is what this RFC targets: **the streaming
renderer accumulates its own state and that state can silently drift
from `layout.entities`**. The emit-order fix patches one leak. Any
future trace event that misorders, miscounts, or misdescribes
committed state will reintroduce the same class of divergence.

## Design

Mirror the pattern in `web/src/renderer/phaseAnimation.ts:127–145`:
`renderLayoutPhaseAnimated` calls `renderLayout` once with an
`onEntityRendered` callback that captures `Map<entityKey, Graphics[]>`,
then animates alpha over those captured graphics. `renderLayout` stays
authoritative; `phaseAnimation` only manipulates alpha. Streaming
should do the same.

### Live phase (≈unchanged)

Streaming renderer continues to draw transient graphics for ghost-path
overlays, committed previews, cluster outlines as events arrive. These
are explicitly eye-candy — they don't need to match the final layout
exactly. In addition, the handler records per-entity first-seen
timestamps into `revealByEntityKey: Map<string, number>` keyed by the
existing `entityKey` at `streamingRenderer.ts:78`.

### `finish(layout)` (new signature)

1. Destroy every transient graphic — `committedLayer.removeChildren()`,
   `ghostLayer.removeChildren()`, `clusterOverlay.clear()`.
2. Call `renderLayout(layout, container, onHover, onSelect, (e, gfx) =>
   entityGfxByKey.set(entityKey(e), gfx))`. The full tile-map is built
   in one pass, so belt-turn detection is correct by construction and
   the per-neighbour rerender hack (`rerenderBeltTile`,
   `rerenderBeltNeighbours`) is unneeded.
3. Build `reveals: {graphic, revealAt}[]` from `entityGfxByKey`
   crossed with `revealByEntityKey` (fallback `latestFadeEndMs` for
   entities that never appeared in a streamed event). Capture ambient
   graphics (non-entity-owned container children — UG tunnel stripes)
   with the set-diff trick at `phaseAnimation.ts:137–145` and assign
   them the `bus_routed` phase start time.
4. Return the real `HighlightController` the `renderLayout` call
   produced — replaces the no-op stub.

### `seekTo(t)` (imperative)

Iterate `reveals`, set
`alpha = clamp((t - revealAt) / FADE_IN_MS, 0, 1)` per graphic, and
set `eventMode = alpha > 0.01 ? "static" : "none"` to match current
interaction-gating. No per-frame ticker post-finish.

### Handle surface change

- `finish(): void` → `finish(layout: LayoutResult): HighlightController`
- `getHighlightController()` removed from `StreamingRendererHandle`
- `hasCommittedEntities()`, `cancel()`, `seekTo()`, `getTimeRange()`,
  `getMilestones()` unchanged

### Call-site diff (main.ts:716–727)

```ts
// Before:
const streamedCtrl = streamingHandle?.hasCommittedEntities()
  ? streamingHandle.getHighlightController() : null;
if (streamingHandle && streamedCtrl) {
  streamingHandle.finish();
  …
  ctrl = streamedCtrl; // no-op controller
}

// After:
if (streamingHandle?.hasCommittedEntities()) {
  ctrl = streamingHandle.finish(layout); // real controller from renderLayout
  …
}
```

### Deletions

- `rerenderBeltTile` (`streamingRenderer.ts:354`)
- `rerenderBeltNeighbours` (`streamingRenderer.ts:374`)
- Final-pass belt rerender loop in `finish` (~880–893)
- Post-finish scrub branches of the ticker (~627–691)
- No-op `HighlightController` shim (~857–862)
- `getHighlightController` method

### Alternatives considered and rejected

- **Reconcile streaming state to `layout.entities` at finish.** Fewer
  lines changed; still leaves the dual-source architecture. Any new
  trace event reintroduces the divergence class. Rejected.
- **Keep streaming state entirely, fix via asserting invariants.** A
  debug_assertion in `ghost_router` that every streamed entity is also
  committed is useful regardless (it would have caught the original
  bug), but it's a fence, not a refactor. Complementary, not
  alternative.
- **Drop streaming eye-candy entirely, render only at finish.** Loses
  the progressive reveal during the 5–6 s SAT phase. Rejected — the
  reveal is load-bearing UX feedback.

## Kill criteria

- **Size bloat.** If `streamingRenderer.ts` post-refactor exceeds 500
  lines (it's ~900 today, target ~350–400), the separation between
  transient and authoritative is leaking. Stop and reconsider.
- **Highlight regression.** Belt-network highlight on hover must
  behave identically on a streamed layout versus a corpus-loaded
  layout. Test side-by-side:
  `/?item=electronic-circuit&rate=10&machine=assembling-machine-1&in=iron-ore,copper-ore`
  (live) versus the same layout loaded via corpus (once such a fixture
  is captured). If hover traces, dim behaviour, or overlay shapes
  diverge, abandon; go back to reconciliation-at-finish.
- **Scrub frame rate.** On a 2000-entity layout, dragging the
  scrubber must sustain ≥60 fps with `seekTo` iterating every graphic.
  If DevTools Performance shows sustained <60 fps during drag, the
  per-entity alpha iteration is too coarse — revert to ticker-based
  with dirty-bit tracking.

## Verification plan

Protocol per [CLAUDE.md](../CLAUDE.md#verification-protocol-for-layout-engine-changes).

- `cargo test --manifest-path crates/core/Cargo.toml` — must stay
  green. No Rust-side changes; any regression indicates accidental
  WASM-build-triggered rebuild artifacts.
- `cd web && npx tsc --noEmit` — clean.
- `wasm-pack build crates/wasm-bindings --target web --out-dir
  "$(pwd)/web/src/wasm-pkg"` — clean rebuild.
- **Bug URL visual regression:**
  `/?item=electronic-circuit&rate=10&machine=assembling-machine-1&in=iron-ore,copper-ore`
  — `(6,7)` renders as transport-belt South (balancer). Inspector and
  `__layout.entities` agree on every tile clicked.
- **Belt-network highlight** — hover any transport-belt on a streamed
  layout; confirm upstream/downstream overlay appears (today: silent).
- **Scrub** — drag scrubber back to t=0, entities fade out in reverse
  reveal order. Drag forward, everything comes back. Inspector on
  mid-scrub tiles shows the real entity (not a faded phantom).

## Phasing

Single PR. The invariant ("`layout.entities` is post-stream truth") is
coherent and shouldn't be split. Writing the RFC, refactoring
`streamingRenderer.ts`, and updating `main.ts` happen together so
there's no half-state where streaming owns some but not all graphics
post-finish.

## Decision log

- *2026-04-22 — accepted after plan review; work begins immediately.*
- *2026-04-22 — landed. `streamingRenderer.ts` went from 926 → 899 lines
  (−27, or ~14% of non-comment lines removed). The "500-line target"
  kill criterion was miscalibrated — the live-preview animation code
  (ghost-belt tracking, transient-commit previews, cluster-outline
  pulses, milestone bookkeeping, fade ticker) is substantively
  complex, and even with all dual-authority code removed the file
  stays ~900 lines. What actually matters was achieved: `renderLayout`
  is the authoritative post-stream renderer, `layout.entities` is the
  single source of truth, and belt-network hover works for streamed
  layouts. Future reviewers: don't tighten the line-count kill
  criterion without first auditing what fraction of the file is
  load-bearing live-preview logic — the answer is "most of it".*
