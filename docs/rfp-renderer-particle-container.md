# RFP: Render static layout via ParticleContainer + texture atlas

## Summary

Replace the post-layout rendering of entity Graphics with a Pixi v8
`ParticleContainer` filled with `Particle` instances that draw from a
shared texture atlas. Each entity becomes a single quad; the whole bus
renders in a handful of draw calls instead of hundreds. The streaming
phase (where transient previews fade in as the engine routes) keeps
its current Graphics-based rendering — that path is short-lived and
not the bottleneck. At `streamingHandle.finish(layout)`, the scene
swaps from Graphics to Particles.

Expected outcome: per-frame render cost drops from ~30 ms to <10 ms on
a 3000-entity layout, making pan/zoom feel native.

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

Two render paths, swapped at one well-defined moment:

```
[engine streams events] ─► streaming Graphics path  ─┐
                                                     │ swap at
                                                     ▼ finish(layout)
                                          settled Particle path
                                          [pan / zoom / hover]
```

- **Streaming path** (existing, untouched) — entities reveal
  progressively as the engine emits `TrunkBeltCommitted` /
  `GhostSpecCommitted` / etc. Each entity is a Pixi `Graphics` added
  to `committedLayer`. This phase is short (≤1.5 s of visible
  reveal, even on a 17 s layout), and the user is busy watching the
  reveal — not panning. Per-frame cost during streaming is not the
  bottleneck.

- **Settled path** (new) — at `streamingHandle.finish(layout)`, walk
  `layout.entities`, look up each entity's pre-rendered texture in
  the atlas, create a `Particle` with `(x, y, texture, tint, alpha)`,
  and add to a `ParticleContainer`. The Graphics-based
  `committedLayer` is destroyed. Hover dim, item-color highlight, and
  selection-overlay machinery talk to particles via direct property
  mutation.

The non-streaming path (corpus / parsed blueprints) goes straight to
the settled path with no streaming intermediate.

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

  Total upper bound: ~150 unique textures. Comfortably fits on one
  GPU texture (4096×4096 with 64 px tiles holds 4096 entries).

- Item-color tinting (used today for belt body coloring per
  `carries`) does **not** require pre-rendering each item × belt
  combination. Pixi's `Particle.tint` is a per-particle multiplier
  applied in shader; we render the belt texture white-on-black and
  tint to the item color at draw time. Same for inserter colors.

- Atlas packing: Pixi v8 has no first-party runtime atlas packer. We
  pre-allocate a single oversized `RenderTexture` and `blit` each
  variant into a deterministic grid slot (`textureKey →
  (atlasX, atlasY)`). Each `Particle` references the atlas
  `Texture` with adjusted UVs. ~150 entries × ~150 LOC total atlas
  code.

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

### Recipe label panels

Currently drawn as a sub-Graphics + Sprite + Text per machine inside
`drawMachine`. Two options:

- **(a)** Render the panel into the machine's atlas slot at a higher
  resolution, accept that recipe info is baked into the texture and
  doesn't reflow on machine click. Simple.
- **(b)** Keep panels as a separate non-particle layer (Container of
  Sprite/Text), one per visible machine. Smaller batched cost
  because the panel layer renders after particles.

Recommend **(a)** initially, **(b)** if recipe-panel readability
suffers at zoom levels where the bake is too low-res.

### Code shape

New: `web/src/renderer/atlas.ts` (~200 LOC) — texture cache,
`getEntityTexture(entity, ctx)` returning `Texture` from atlas,
on-miss renders + packs.

New: `web/src/renderer/particleLayout.ts` (~250 LOC) — `renderLayoutAsParticles(layout, container) → ParticleHighlightController`,
mirroring the existing `renderLayout` shape. Builds tileMap, creates
particles, returns a controller with `highlightItem` /
`highlightBeltNetwork` / `clearHighlight` / `chainKey`.

Modified: `web/src/main.ts` — non-streaming and `finish()` paths
call `renderLayoutAsParticles` instead of `renderLayout`. Hover dim
goes via the new controller.

Modified: `web/src/renderer/streamingRenderer.ts` — `finish(layout)`
swaps to particles.

Unmodified: `entities.ts` `drawBelt`/`drawPipe`/etc. remain the
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

- **Prototype underperforms.** If a one-entity-type prototype (just
  belts, replacing belt Graphics with belt Particles in the settled
  path) doesn't show ≥2× per-frame render speedup on the
  `processing-unit @ rate=2` URL, the approach doesn't deliver and we
  abandon. Measurement: DevTools Performance recording during pan,
  compare `executeInstructions` self-time before/after.
- **Atlas-build cost regresses startup.** If the runtime atlas
  generation takes >300 ms wall-clock for a typical layout
  (measured from `finish(layout)` to first particle render), the
  layout-→-interactive transition feels worse than today even if
  steady-state pan is faster. We'd then need build-time atlas
  generation, which is bigger scope; pivot.
- **Hover/highlight degrades.** If the per-particle alpha mutation
  for `highlightItem` on 3000 entities takes >5 ms (measured by
  DevTools Performance during a `highlightItem` call), the
  particle-iteration path is too slow vs Graphics. Pivot to a
  shader-uniform highlight scheme.
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
   - Layout commits at end of streaming, no visual flicker on the
     Graphics→Particle swap.
   - Hover dim looks identical to today.
   - Item-color highlight (`highlightItem`) preserves all current
     visual cues.
   - Belt-network highlight (`highlightBeltNetwork`) preserves
     dashed/solid distinction (this currently uses a Graphics
     overlay — that overlay can stay; only entity dim/light changes).
   - Multi-tile machines render at correct size, recipe labels
     readable.
   - Selection box and pin highlight unaffected.
4. **Anti-test**: leave the page idle after layout settles. CPU
   should drop to near zero (already verified post-PR-#205, just
   confirm we haven't regressed).

## Phasing

The work splits naturally into landable chunks. Each one is a PR
that's good on its own; subsequent ones build on it.

1. **Phase 1 — atlas plumbing.** Add `renderer/atlas.ts` with
   `getEntityTexture` and lazy texture generation backed by a single
   `RenderTexture`. No behavior change yet; atlas exists, nothing
   uses it. Includes a smoke test that requesting a known belt
   variant returns a valid texture.

2. **Phase 2 — particle path for one entity type.** Implement
   `renderLayoutAsParticles` for belts only; other entities still
   render as Graphics. Wire `finish(layout)` to a hybrid render
   (particles for belts, Graphics for everything else, in a parent
   container). **Kill criterion #1 lives here** — measure trace
   numbers before merging Phase 3+.

3. **Phase 3 — extend to all entity types.** Pipes, inserters,
   splitters, UG belts, machines. Recipe labels go into the atlas
   (option a).

4. **Phase 4 — wire highlight controller.** Replace
   `HighlightController` with a particle-aware version. Today's
   alpha-walk pattern translates directly.

5. **Phase 5 (optional follow-up)** — recipe labels as separate
   layer (option b) if Phase 3 labels are unreadable.

## Out of scope

- Animations during streaming (transient fade-ins). The streaming
  path keeps its current Graphics implementation.
- OffscreenCanvas. Separate RFP if main-thread cost is still an
  issue after this lands.
- Build-time atlas generation. Runtime is sufficient unless kill
  criterion #2 trips.
- Scrub-mode / timeline scrubber rewinding. The scrub phase already
  imperatively recomputes alphas from `revealByEntityKey` — that
  logic translates from Graphics to Particles by changing where the
  mutation lands.

## Decision log

- *2026-04-25 — Drafted. Awaiting human review before scheduling
  Phase 1.*
