# RFP: Render static layout via ParticleContainer + texture atlas

## Summary

Replace entity rendering with a Pixi v8 `ParticleContainer` filled
with `Particle` instances that draw from a shared texture atlas. Each
entity becomes a single quad; the whole bus renders in a handful of
draw calls instead of hundreds. **One render path**: streaming and
settled use the same `ParticleContainer` from event 1 onward. Pan
during long routings (e.g. 16 s of `processing-unit` ghost routing)
is therefore as fast as pan after settle. `streamingHandle.finish()`
no longer swaps anything — it just stops the live-phase ticker and
freezes the particle alpha state.

Expected outcome: per-frame render cost drops from ~30 ms to <10 ms on
a 3000-entity layout, making pan/zoom feel native, both during
streaming and after.

## Motivation

Trace `Trace-20260425T135149.json.gz`, captured panning a settled
`processing-unit @ rate=2` layout (2892 entities, 87×144 tiles), shows
the steady-state render is dominated by GL state changes between many
small batches:

| GL operation | Self time over 9.7 s | What it represents |
|---|---|---|
| `uniformMatrix3fv` | 1207 ms | Per-batch matrix uploads |
| `drawElements` | 826 ms | Actual draw calls |
| `bindVertexArray` | 670 ms | VAO switches |
| `useProgram` | 338 ms | Shader switches |
| `bindTexture` | 131 ms | Texture binds |
| `_buildInstructions` / `_updateRenderGroups` | ≈0 ms | **Cache is hot** |

State changes (program/VAO/texture/uniforms) cost more than the actual
draws. The render-group's instruction cache works correctly — the
problem is the *playback* of those instructions: many small batches,
broken by z-order interleaving of Graphics (belts, pipes, inserters),
Sprites (machine bodies, item icons), and Text (recipe panel labels).

After the dedup fix in PR #211 the per-frame cost roughly halves,
landing in the ~25–30 ms range. To push pan to feel native (≤5 ms /
frame at 60 Hz) we need to fundamentally cut the number of batches.

## Design

### High-level shape

A single render path. Each streaming event adds particles to a shared
`ParticleContainer`, and the same particles persist into the settled
state. `finish()` is now a lifecycle marker, not a scene rebuild:

```
[engine streams events] ─► add particles to ParticleContainer ─┐
                                                               │ finish() stops
                                                               │ live ticker
                                                               ▼
                                       same ParticleContainer; pan/zoom/hover
                                       cost identical to streaming-pan cost
```

- Each streaming event handler (`TrunkBeltCommitted`,
  `GhostSpecCommitted`, `JunctionCommitted`, etc.) constructs
  `Particle`s and appends to the container.
- `revealAt` timestamps go into `revealByEntityKey` exactly as today.
  The live-phase ticker mutates `particle.alpha` based on
  `(now − revealAt) / FADE_IN_MS`, identical animation behaviour.
- Ghost-belt previews (`addGhostBelt`) add to a sibling
  `ParticleContainer` with item-color tints and lower alpha; fading
  out a ghost belt is `particle.alpha *= (1 − outT)` then
  `removeParticle` when fully transparent.
- At `finish()`, the live-phase ticker is removed; the
  `ParticleContainer` and all its particles persist as the settled
  scene. No `removeChildren` / rebuild step.

The non-streaming path (corpus / parsed blueprints) calls the same
particle-creation helpers without going through the streaming event
machinery.

### Atlas

Generated at runtime, lazily, on first need per `(entity, variant)`:

- `entity-frames/${name}.png` is already loaded for machines via
  `Assets.load` (`renderer/entities.ts:initEntityIcons`). Reuse
  directly as atlas frames.
- For belts, pipes, inserters, splitters, UG belts: render the
  existing `drawBelt` / `drawPipe` / `drawInserter` / `drawSplitter`
  / `drawEntityGraphic` outputs into a `RenderTexture` once per
  unique `(name, direction, turn variant)`, cache by key. The
  variant set is bounded:

  | Type | Variants | Count |
  |---|---|---|
  | Belt body | tier × direction × turn-variant | 3 × 4 × 5 = 60 |
  | UG belt | tier × direction × in/out | 3 × 4 × 2 = 24 |
  | Splitter | tier × direction | 3 × 4 = 12 |
  | Pipe | connection-mask | 16 |
  | PTG | direction | 4 |
  | Inserter | type × direction | 4 × 4 = 16 |
  | Machine | per-name | 21 (from `MACHINE_SLUGS`) |
  | Item icon | per-item (see below) | ~80–120 |

  Total upper bound: ~250 unique textures. Comfortably fits on one
  GPU texture (4096×4096 with 64 px tiles holds 4096 entries).

- Item-color tinting on belts (used today for belt body coloring per
  `carries`) is **dropped** — item identity is conveyed by the icon
  particle layer (see "Item icons on every belt / pipe / inserter"
  below). Belt textures stay neutral; the tint slot is reserved for
  hover-dim multipliers.

- Item icons: one atlas entry per `(name, kind)` pair where `name` is
  the item slug. Loaded from existing `icons/${slug}.png` PNGs
  (cached via `Assets.load` in `preloadCarriesIcons`). Items without
  icons (fluids, currently) use a generated placeholder texture
  (colored circle from `itemColor(item)`). See "Item icons on every
  belt / pipe / inserter" below.

- Atlas packing: Pixi v8 has no first-party runtime atlas packer. We
  pre-allocate a single oversized `RenderTexture` and `blit` each
  variant into a deterministic grid slot (`textureKey →
  (atlasX, atlasY)`). Each `Particle` references the atlas
  `Texture` with adjusted UVs. ~250 entries × ~150 LOC total atlas
  code.

### Atlas readiness

The atlas must be ready before the first streaming event arrives,
because streaming events now produce particles directly. Two
acceptable timings:

- **Eager at engine init.** Pre-render every variant in the bounded
  set (~150 textures) up-front, before `initEngine()` returns. Cost
  measured ≈1 ms per `RenderTexture` blit, so ~150 ms total. Fits
  inside the existing init wait without user-perceivable delay.
- **Lazy on first miss.** Same `getEntityTexture` API, but the cache
  populates on demand. First entity of a new variant triggers one
  blit (~1 ms) before its particle is created. Distributed across
  the streaming phase; per-event latency is negligible.

Pick lazy initially — it has no startup cost and all variants
typically appear within the first ~1 s of streaming anyway.
Kill criterion #2 enforces the upper bound.

### Per-particle properties used

```ts
new Particle({
  texture: atlasTexture(entityName, direction, variant),
  x: entityX * TILE_PX,
  y: entityY * TILE_PX,
  scaleX: 1,
  scaleY: 1,
  tint: itemColor(entity.carries),  // belt/pipe/inserter; white for others
  alpha: 1,                          // mutated for hover dim
});
```

`ParticleContainer` configured with
`dynamicProperties: { color: true, position: false, rotation: false,
vertex: false, uvs: false }` so tint/alpha mutations cost nothing
beyond the GPU upload.

### Hover dim and highlights

Today: `HighlightController.highlightItem` walks `allGraphics`, sets
`g.alpha`. With particles: walks `allParticles`, sets `p.alpha`. The
`color: true` dynamic property uploads alpha changes to GPU at the
next render. Identical UX, identical code shape.

### Multi-tile entities

3×3 machines: pre-rendered texture is 3×TILE_PX wide; one particle
per machine at the anchor tile. Same for splitters (1×2).

### Item icons on every belt / pipe / inserter

Today the carries-icon overlay only appears on every fifth belt tile
(`(ex + ey) % 5 === 0` in `entities.ts:1201`) — a heuristic to avoid
visual clutter when icons are drawn as separate Graphics children of
each belt. With particles we can drop the heuristic entirely:

> **Every belt, pipe, and inserter tile gets an item-icon particle
> based on `entity.carries`.**

The icons live in their own `ParticleContainer` (so they batch
independently of the belts/pipes underneath). Atlas adds one entry
per item slug:

```ts
ParticleContainer "entities"  ← belt/pipe/inserter/machine textures
ParticleContainer "icons"     ← item-icon textures
```

For each conveyor entity the renderer adds two particles: the entity
particle (plain texture, no item color baked in) and an icon particle
at the same tile center. The existing `Particle.tint` + `alpha`
properties cover hover dim and item highlight on both.

This collapses two heuristics in `entities.ts`:

- The `% 5` icon-spacing rule (no longer needed — icons are cheap
  particles, every tile gets one).
- The "tint belt body by item color" trick (the icon texture conveys
  item identity now; belt body stays neutral).

#### Icons on inserters: the machine-input/output channel

Per discussion, **machines themselves do not get icons or recipe
panels**. The recipe label panel currently drawn inside `drawMachine`
(`entities.ts:740-810`) is **removed entirely**. Recipe info is
conveyed by:

1. The icon on the machine's inserter, showing what's being delivered
   in or extracted out.
2. (Future, out of scope for this RFP) a richer hover tooltip on
   machines showing rates, recipe, etc.

Inserters are 1-tile entities adjacent to machines, with `direction`
indicating which way they're pulling/pushing and `carries` set to
the item they transport. Icon-on-inserter tells the user "this item
flows through here," with the inserter's position relative to the
machine implicitly conveying input vs output.

Fluid machines (chemical plant, oil refinery, etc.) take fluid via
adjacent pipes. Pipes already get per-tile icons under the rule
above, so the same channel works.

#### Fluid icon placeholders

We don't have `icons/water.png` / `icons/crude-oil.png` / etc. yet.
Two-stage:

- **In this RFP**: `getItemIconTexture(item)` returns the cached
  PNG via `Cache.has` if present, otherwise a generated placeholder
  texture — a 14 px colored circle, where the colour comes from
  `itemColor(item)` (which already has hardcoded fluid colors). Belts
  carrying fluids show a recognisable colored dot.
- **Out of scope, ticketed separately**: real fluid icons. Once they
  exist, `preloadCarriesIcons` picks them up automatically and the
  placeholder fallback stops firing. No code change needed at that
  point.

### Scrubber compatibility

The timeline scrubber's `seekTo(virtualMs)` mechanism in
`web/src/renderer/streamingRenderer.ts:879-916` walks a `reveals`
array of `{ graphic: Graphics, revealAt: number }`, computes alpha as
a linear ramp over `FADE_IN_MS = 150ms` from `revealAt`, and writes
`r.graphic.alpha`. That's the entire scrub-mode mutation surface.

Under particles, the array becomes
`{ particle: Particle, revealAt: number }` and the mutation is
`r.particle.alpha`. With `dynamicProperties.color: true`, the
per-particle alpha change uploads to GPU on the next render. **No
conceptual change to scrub mode** — same loop, same source map
(`revealByEntityKey`), different mutation target.

The line `r.graphic.eventMode = alpha > 0.01 ? "static" : "none"` in
`applyReveals` is already dead code post-PR #212 (entity-level
`eventMode` is unconditionally `"none"` since hover went tile-based).
Drop during the migration.

### Debug-mode logging

Animation timings are inherently hard to debug — fade-in onset,
stagger spread, junction-zone reveal — so the migration adds an
optional logging channel. Gated on the existing `debugCb` toggle in
`main.ts`, surfaced to `streamingRenderer.ts` via a `globalThis`
flag (mirroring the `__TRACE_LOGS` pattern already in
`streamingRenderer.ts:734-739` and `engine.ts:113-123`):

```ts
// main.ts — sync the flag with debugCb on every change
const syncAnimLogs = (): void => {
  (globalThis as { __ANIM_LOGS?: boolean }).__ANIM_LOGS = debugCb.checked;
};
debugCb.addEventListener("change", syncAnimLogs);
syncAnimLogs();

// streamingRenderer.ts — single helper, called at every animation start
const animLog = (phase: string, detail: Record<string, unknown>): void => {
  if (!(globalThis as { __ANIM_LOGS?: boolean }).__ANIM_LOGS) return;
  console.log(`[anim t=${clock().toFixed(0)}ms] ${phase}`, detail);
};
```

Per-batch (not per-entity), at every animation-start call site:

| Site | Phase tag | Detail to include |
|---|---|---|
| `handleRowsPlaced` | `rows_placed` | `{ count, stagger_ms, span_ms }` |
| `handleGhostSpecRouted` | `ghost_routed` | `{ spec_key, item, tile_count, span_ms }` |
| `handleGhostSpecCommitted` / silent-stamp variants | `committed` | `{ source: "spec" \| "trunk" \| "balancer" \| ..., count, span_ms }` |
| `handleJunctionCommitted` | `junction` | `{ cluster_id, zone, count, span_ms }` |
| `handleGhostClusterSolved` | `cluster_outline` | `{ cluster_id, zone, lifetime_ms, fade_ms }` |
| `runSeekAnimation` (`main.ts`) | `seek_step` | `{ added_count, stagger_ms }` |

Plus three scope-boundary logs: `streaming start`, `streaming finish`
(with `{ entity_count, latest_fade_end_ms }`), and `scrub` (one log
per drag, deduped by last virtualMs to avoid per-pixel spam).

Phase-tag strings are literal so logs are grep-able. `span_ms`
(= `count × stagger_ms`) tells the developer when the batch
completes, which is the bit that's invisible without instrumentation.

### Code shape

New: `web/src/renderer/atlas.ts` (~200 LOC) — texture cache,
`getEntityTexture(entity, ctx)` returning `Texture` from atlas,
on-miss renders + packs.

New: `web/src/renderer/particleLayout.ts` (~250 LOC) — particle
scene builder. Exposes:

- `commitEntityAsParticle(entity, revealAt)` — used by
  `streamingRenderer` event handlers and by the non-streaming path.
  Looks up the texture in the atlas, constructs a `Particle`, adds
  to the appropriate `ParticleContainer`, registers in
  `revealByEntityKey`.
- `addGhostParticle(x, y, direction, item, revealAt, specKey)` — for
  ghost-belt previews; same shape but lower target alpha and
  item-color tint.
- `removeParticleAt(key)` — for fade-out completion.
- `ParticleHighlightController` — `highlightItem` /
  `highlightBeltNetwork` / `clearHighlight` / `chainKey`, walks
  particles instead of Graphics.

Modified: `web/src/renderer/streamingRenderer.ts` — every event
handler routes through `commitEntityAsParticle`. The live-phase
ticker mutates `particle.alpha` instead of `g.alpha`. `finish()` is
reduced to "stop ticker, freeze state, return controller." `seekTo`
walks particles in the same `reveals` shape as today.

Modified: `web/src/main.ts` — `__ANIM_LOGS` sync wired to
`debugCb`. Non-streaming path calls the same particle helpers
without going through streaming events.

Unmodified: `entities.ts` `drawBelt` / `drawPipe` / etc. remain the
source-of-truth pixel logic; the atlas calls them at texture-render
time.

## Alternatives considered

### Pixi tilemap (`@pixi/tilemap` v5.0.2)

**Rejected.** Conceptually a perfect fit (Factorio-style grid), but
the v5 `tile()` API does **not** expose a per-tile `tint` parameter
(verified against the source). Belt item-coloring is currently a tint
multiplication; without tint we'd have to bake every item × belt
combination into the atlas — ~5 items × 60 belt variants = 300 extra
textures. Also blocks future item-color additions without atlas
re-bakes. ParticleContainer gives us tint for free.

If item-color tinting moves to an overlay (separate sprite layer for
the carry-item indicator, leaving the belt texture untinted), tilemap
becomes viable. This is a defensible refactor but bigger surgery
than the ParticleContainer path.

### Single static mesh

**Rejected for now.** Encoding all entity geometry into one big
vertex buffer with per-vertex color/UV/alpha gives the absolute
lowest per-frame cost (one draw call), but loses Pixi's batching
infrastructure entirely and requires custom shader code. Per-entity
mutation (hover dim, item highlight) needs a tile→vertex-range index
and partial buffer uploads. Considerable surgery for a marginal win
over ParticleContainer (which already drops to a handful of draws).
Revisit only if ParticleContainer underperforms.

### OffscreenCanvas

**Orthogonal — defer.** Moves Pixi off the main thread. Doesn't
change the drawing shape; complementary to ParticleContainer. Worth
considering after ParticleContainer ships, *if* main-thread input
responsiveness is still an issue. Big refactor (event bridge,
inspector cross-thread reads). Not on the critical path.

### Status quo + further micro-optimization

**Insufficient.** Layer separation by entity type (the proposed PR B
from the discussion) gets us maybe 2× — still ~15 ms/frame, still
not native-feel at 60 Hz. The state-change cost is fundamental to
having heterogeneous Graphics/Sprite/Text content per entity. Solving
it requires homogenising to one type — particles.

## Kill criteria

Each kill criterion is something I commit to acting on if it trips,
not just a soft target.

- **Prototype underperforms during streaming-pan.** If the Phase 1
  prototype (trunks-as-particles) doesn't show ≥2× per-frame render
  speedup **measured during the `ghost_routing` phase** on
  `processing-unit @ rate=2`, the approach doesn't deliver and we
  abandon. Streaming-pan is the more demanding test than settled-pan
  (live ticker, more transient state); if Phase 1 doesn't move the
  needle there, the architecture isn't right. Measurement: DevTools
  Performance recording during pan mid-routing, compare
  `executeInstructions` self-time before/after.
- **Atlas-build cost regresses startup.** If lazy texture generation
  collectively takes >300 ms wall-clock for a typical layout
  (sum of all on-miss `RenderTexture` blits during the first
  streaming second), interactive responsiveness regresses. Pivot to
  eager atlas generation at engine init.
- **Hover/highlight degrades.** If the per-particle alpha mutation
  for `highlightItem` on 3000 entities takes >5 ms (measured by
  DevTools Performance during a `highlightItem` call), the
  particle-iteration path is too slow vs Graphics. Pivot to a
  shader-uniform highlight scheme.
- **Scrub-mode regresses.** If a scrub-mode rewind / forward step is
  visibly less smooth than today's Graphics-based scrub on a
  3000-entity layout (measured: drag the timeline scrubber from end
  to start; framerate during drag must stay within 10% of pre-RFP
  baseline), pivot — the per-particle alpha update path during scrub
  is too slow. Most likely fix: throttle `applyReveals` to once per
  rAF instead of once per `seekTo` call.
- **Recipe label legibility drops noticeably.** If the baked-into-
  atlas approach (option a) makes recipe labels unreadable at the
  default zoom, we move panels to a separate non-particle layer
  (option b) — not a kill, but worth acknowledging up-front.

## Verification plan

1. **Unit-level**: a smoke test that renders a fixture layout via
   `renderLayoutAsParticles`, asserts the right number of particles
   exist with the expected positions and tints. Lives alongside
   existing rendering tests if any (currently the `web/` codebase has
   none — would establish pattern).
2. **Trace comparison**: re-record a 9 s pan trace at the
   `processing-unit @ rate=2` URL after each phase lands. Compare
   `executeInstructions` self-time and the GL-state-change tally
   table from the Motivation section. Land each phase only if it
   moves the numbers in the expected direction.
3. **Browser eyeball checklist**:
   - Streaming reveal animations look identical to today (fade-in
     timings, NW→SE stagger, junction zone reveals, ghost belt
     previews, cluster outline pulses).
   - Layout transitions cleanly from "live ticker mutating alphas"
     to "static settled scene" at `finish()` — no visual blip.
   - Hover dim looks identical to today.
   - Item-color highlight (`highlightItem`) preserves all current
     visual cues.
   - Belt-network highlight (`highlightBeltNetwork`) preserves
     dashed/solid distinction (this currently uses a Graphics
     overlay — that overlay can stay; only entity dim/light changes).
   - Multi-tile machines render at correct size. (Recipe label
     panels are gone — recipe info now flows from the icon-on-
     inserter / icon-on-pipe channel, with rate detail moving to a
     future hover tooltip.)
   - Every belt, pipe, and inserter shows its `carries` item icon at
     tile center. Fluid pipes show the placeholder colored-circle
     until real fluid icons land.
   - Selection box and pin highlight unaffected.
   - **Scrubber**: drag the timeline scrubber back and forth on a
     settled layout. Entities fade in/out smoothly, milestones snap,
     framerate stays smooth.
   - **Debug logging**: tick the debug-mode checkbox, reload, watch
     console — `[anim ...]` lines appear at every animation start.
4. **Anti-test**: leave the page idle after layout settles. CPU
   should drop to near zero (already verified post-PR-#205, just
   confirm we haven't regressed).

## Phasing

The work splits naturally into landable chunks. Each one is a PR
that's good on its own; subsequent ones build on it.

1. **Phase 1 — atlas plumbing + trunk particles.** Add
   `renderer/atlas.ts` with `getEntityTexture` and lazy texture
   generation backed by a single `RenderTexture`. Add
   `renderer/particleLayout.ts` with `commitEntityAsParticle` for
   trunk entities only (chosen because trunks already have streaming
   events from PR #206 and are the highest-volume single entity type
   on a typical bus). Trunk events route through the new path during
   streaming; other entity types still hit the Graphics path. Hybrid
   container layout: trunk `ParticleContainer` underneath the
   Graphics-based `committedLayer`. Wire the `__ANIM_LOGS` flag in
   `main.ts` so debug logging is ready for Phase 4. **Kill criterion
   #1 lives here** — measure streaming-pan trace numbers before
   merging Phase 2+.

2. **Phase 2 — extend particles to all entity types + add icons
   layer.** Replace each remaining entity type's draw path with a
   particle path, in priority order (highest count first): belts,
   pipes, machines, inserters, splitters, UG belts, mergers, poles.
   `addGhostBelt` moves to a sibling `ParticleContainer`. **Add the
   icons `ParticleContainer`**: every belt / pipe / inserter that
   has `carries` set gets an item-icon particle at tile center; the
   `% 5` icon-spacing heuristic in `entities.ts` is removed.
   Machines render without their recipe label panel — that code path
   in `drawMachine` (`entities.ts:740-810`) is deleted; rate-detail
   future work goes through hover tooltips. At end of phase,
   `finish()` no longer `removeChildren`+rebuilds — it just stops
   the live ticker. **Kill criteria #2, #3, #4** evaluated here.

3. **Phase 3 — wire highlight controller.** Replace
   `HighlightController` with a particle-aware version. Today's
   alpha-walk pattern translates directly. Drop the dead `eventMode`
   line in `applyReveals`.

4. **Phase 4 — debug-mode animation logging.** Add the `animLog`
   helper and call it at all 6 sites in the table above plus the 3
   scope-boundary logs. Small follow-up; could merge into Phase 1
   if it stays trivial.

5. **Phase 5 (optional follow-up)** — recipe labels as separate
   layer (option b) if Phase 2 labels are unreadable.

## Out of scope

- OffscreenCanvas. Separate RFP if main-thread cost is still an
  issue after this lands.
- Build-time atlas generation. Lazy runtime is sufficient unless
  kill criterion #2 trips.
- Cluster outline rectangles. They stay as Graphics — only ~9 of
  them, drawn each frame from a shared `clusterOverlay` Graphics,
  cheap and a poor fit for particles.
- Real fluid icons (`icons/water.png`, `crude-oil.png`, etc.).
  Tracked separately; placeholder colored circles via
  `itemColor(item)` in the interim.
- Hover tooltips with rate / recipe detail for machines. The recipe
  label panel that currently overlays each machine is removed in
  Phase 2; the richer hover-tooltip replacement is a follow-up.

## Decision log

- *2026-04-25 — Drafted. Awaiting human review before scheduling
  Phase 1.*
- *2026-04-25 — Revised: streaming and settled paths unified under a
  single `ParticleContainer`; added scrubber-compatibility section,
  debug-mode logging section, atlas-readiness section, and a fifth
  kill criterion for scrub-mode framerate. Cluster outlines moved to
  out-of-scope explicitly.*
- *2026-04-25 — Revised: icon-on-every-conveyor-tile design folded
  in. Item icons render via a separate `ParticleContainer`, one
  particle per belt / pipe / inserter tile. The `% 5` icon-spacing
  heuristic in `entities.ts:1201` is removed. Machine recipe label
  panels are removed entirely; recipe info flows through icon-on-
  inserter (and pipe for fluid inputs/outputs), with rate detail
  deferred to a future hover-tooltip enhancement. Fluid icons get a
  placeholder colored circle until real PNGs ship; ticketed
  separately. Tint-belt-by-item-color trick dropped — neutral belt
  textures, item identity is conveyed by the icon layer.*
