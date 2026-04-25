/**
 * Particle-scene builder — Phase 2+3+4.
 *
 * Provides `ParticleScene` — three `ParticleContainer`s (entities, icons,
 * ghosts) — and helpers for committing entities as particles with fade-in
 * reveals.
 *
 * Every entity type is routed through `commitEntityAsParticle`. The
 * Graphics path in `streamingRenderer.ts` is no longer used.
 */

import {
  Container,
  Graphics,
  Particle,
  ParticleContainer,
  Sprite,
  Assets,
  Cache,
} from "pixi.js";
import type { Texture } from "pixi.js";
import type { LayoutResult, PlacedEntity, EntityDirection } from "../engine";
import {
  beltAtlasKey,
  pipeAtlasKey,
  ugBeltAtlasKey,
  splitterAtlasKey,
  inserterAtlasKey,
  machineAtlasKey,
  poleAtlasKey,
  ptgAtlasKey,
  CELL_PX,
  getEntityTexture,
  getMultiCellTexture,
  getItemIconTexture,
} from "./atlas";
import {
  drawBelt,
  drawEntityGraphic,
  addEntityToDrawContext,
  createDrawContext,
  TILE_PX,
  BELT_ENTITIES,
  UG_BELT_ENTITIES,
  SPLITTER_ENTITIES,
  INSERTER_ENTITIES,
  POLE_ENTITIES,
  MACHINE_ENTITIES,
  MACHINE_SIZES,
  itemColor,
  type DrawContext,
  type HighlightController,
} from "./entities";
import {
  buildBeltGraph,
  traceBeltNetwork,
  findAdjacentInserters,
  findAdjacentMachines,
  drawBeltNetworkOverlay,
} from "./beltGraph";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Fade-in duration in virtual milliseconds — mirrors streamingRenderer.ts. */
export const FADE_IN_MS = 150;

/**
 * Pixel-per-tile used when entity-frame PNGs were generated.
 * Must match `ENTITY_FRAME_TILE_PX` in entities.ts.
 */
const ENTITY_FRAME_TILE_PX = 64;

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/** Per-entity particle pair stored for alpha animation and diagnostics. */
interface ParticleEntry {
  entity: Particle;
  icon?: Particle;
  revealAt: number;
  placedEntity: PlacedEntity;
}

// ---------------------------------------------------------------------------
// ParticleScene
// ---------------------------------------------------------------------------

/**
 * Top-level container for the particle-based render path.
 * Four `ParticleContainer`s in z-order (bottom to top):
 *   - beltContainer:    belts, pipes, inserters, splitters, UG belts, poles
 *   - machineContainer: machines (rendered above belts so top-edge belt tiles
 *                       don't bleed through machine bodies)
 *   - ghostContainer:   ghost belt previews (z-order: above committed entities)
 *   - iconContainer:    item-icon textures (z-order: topmost)
 * All use `dynamicProperties: { color: true }` so per-particle
 * alpha/tint mutations are cheap.
 */
export interface ParticleScene {
  beltContainer: ParticleContainer;
  machineContainer: ParticleContainer;
  ghostContainer: ParticleContainer;
  iconContainer: ParticleContainer;
  /** The layout this scene was built from (set by `renderLayoutAsParticles`). */
  layout: LayoutResult | null;
  /**
   * Add the particle containers to `parent` in the correct z-order:
   * belt → machine → ghost → icon.
   */
  attachTo(parent: Container): void;
  /**
   * Reset for a fresh layout — removes all particles and clears the
   * internal entity-key map.
   */
  clear(): void;
  /** Total particle count across all containers (for diagnostics). */
  count(): number;
}

function makeParticleContainer(): ParticleContainer {
  return new ParticleContainer({
    dynamicProperties: {
      color: true,
      position: false,
      rotation: false,
      vertex: false,
      uvs: false,
    },
  });
}

export function createParticleScene(): ParticleScene {
  const beltContainer = makeParticleContainer();
  const machineContainer = makeParticleContainer();
  const ghostContainer = makeParticleContainer();
  const iconContainer = makeParticleContainer();

  const scene: ParticleScene = {
    beltContainer,
    machineContainer,
    ghostContainer,
    iconContainer,
    layout: null,
    attachTo(parent: Container): void {
      parent.addChild(beltContainer);
      parent.addChild(machineContainer);
      parent.addChild(ghostContainer);
      parent.addChild(iconContainer);
    },
    clear(): void {
      beltContainer.removeParticles();
      machineContainer.removeParticles();
      ghostContainer.removeParticles();
      iconContainer.removeParticles();
      particleMap.clear();
    },
    count(): number {
      return (
        beltContainer.particleChildren.length +
        machineContainer.particleChildren.length +
        ghostContainer.particleChildren.length +
        iconContainer.particleChildren.length
      );
    },
  };
  return scene;
}

// ---------------------------------------------------------------------------
// Internal particle registry
// ---------------------------------------------------------------------------

/**
 * Global particle registry keyed by entity key.
 * One scene exists at a time; cleared on `scene.clear()`.
 */
const particleMap = new Map<string, ParticleEntry>();

export function entityKey(entity: PlacedEntity): string {
  return `${entity.x ?? 0},${entity.y ?? 0}:${entity.name}:${entity.recipe ?? ""}`;
}

// ---------------------------------------------------------------------------
// Texture resolution helpers
// ---------------------------------------------------------------------------

/**
 * Detect belt turn variant from the tileMap context.
 * Returns "corner-cw", "corner-ccw", or "straight".
 */
function detectBeltTurnVariant(
  entity: PlacedEntity,
  tileMap: Map<string, PlacedEntity>,
): "straight" | "corner-cw" | "corner-ccw" {
  // Re-implement detectBeltTurn logic inline to get the string form.
  const d = entity.direction ?? "North";
  const dirVecMap: Record<string, [number, number]> = {
    North: [0, -1], East: [1, 0], South: [0, 1], West: [-1, 0],
  };
  const [mdx, mdy] = dirVecMap[d] ?? [0, -1];
  let hasStraightFeeder = false;
  let perpTurn: "cw" | "ccw" | null = null;

  for (const [dx, dy] of [[0, -1], [1, 0], [0, 1], [-1, 0]] as [number, number][]) {
    const nx = (entity.x ?? 0) + dx;
    const ny = (entity.y ?? 0) + dy;
    const nb = tileMap.get(`${nx},${ny}`);
    if (!nb) continue;
    const feeds =
      BELT_ENTITIES.has(nb.name) ||
      (UG_BELT_ENTITIES.has(nb.name) && nb.io_type === "output") ||
      SPLITTER_ENTITIES.has(nb.name);
    if (!feeds) continue;
    const [ndx, ndy] = dirVecMap[nb.direction ?? "North"] ?? [0, -1];
    const nbFlowX = SPLITTER_ENTITIES.has(nb.name) ? nx : (nb.x ?? 0);
    const nbFlowY = SPLITTER_ENTITIES.has(nb.name) ? ny : (nb.y ?? 0);
    if (nbFlowX + ndx !== (entity.x ?? 0) || nbFlowY + ndy !== (entity.y ?? 0)) continue;
    if (nb.direction === d) {
      hasStraightFeeder = true;
    } else {
      const cross = ndx * mdy - ndy * mdx;
      if (cross !== 0) perpTurn = cross > 0 ? "cw" : "ccw";
    }
  }
  if (perpTurn && !hasStraightFeeder) {
    return perpTurn === "cw" ? "corner-cw" : "corner-ccw";
  }
  return "straight";
}

/**
 * Compute the pipe connection mask for a regular pipe entity.
 * N=1, E=2, S=4, W=8. Looks at ctx.tileMap and ctx.machineTileSet.
 */
function computePipeConnectionMask(entity: PlacedEntity, ctx: DrawContext): number {
  if (entity.name !== "pipe") return 0;
  const ex = entity.x ?? 0;
  const ey = entity.y ?? 0;

  // Helper — mirrors entities.ts:pipeConnectsToNeighbour
  function pipeConnectsToNeighbour(nb: PlacedEntity, dx: number, dy: number): boolean {
    if (nb.name === "pipe") return true;
    if (nb.name === "pipe-to-ground") {
      // PTG surface delta: opposite of its tunnel direction.
      const dirVecMap: Record<string, [number, number]> = {
        North: [0, -1], East: [1, 0], South: [0, 1], West: [-1, 0],
      };
      const [tdx, tdy] = dirVecMap[nb.direction ?? "North"] ?? [0, -1];
      const [sx, sy] = [-tdx, -tdy];
      return -dx === sx && -dy === sy;
    }
    return false;
  }

  let mask = 0;
  for (const [dx, dy, bit] of [[0, -1, 1], [1, 0, 2], [0, 1, 4], [-1, 0, 8]] as [number, number, number][]) {
    // Top-edge pipe connects "upward" to represent external fluid source.
    if (dy === -1 && ey + dy < 0) { mask |= bit; continue; }
    const key = `${ex + dx},${ey + dy}`;
    const nb = ctx.tileMap.get(key);
    if ((nb && pipeConnectsToNeighbour(nb, dx, dy)) || ctx.machineTileSet.has(key)) {
      mask |= bit;
    }
  }
  return mask;
}

/**
 * Resolve the entity texture for any entity type. Uses the atlas
 * helpers in atlas.ts, calling the draw functions from entities.ts
 * to render on cache-miss.
 */
function getEntityAtlasTexture(entity: PlacedEntity, ctx: DrawContext): Texture {
  if (BELT_ENTITIES.has(entity.name)) {
    const turn = detectBeltTurnVariant(entity, ctx.tileMap);
    const key = beltAtlasKey(entity.name, entity.direction ?? "North", turn);
    return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
      const scale = CELL_PX / TILE_PX;
      // Build turn info for drawBelt.
      let turnArg: { turn: "cw" | "ccw" } | null = null;
      if (turn === "corner-cw") turnArg = { turn: "cw" };
      else if (turn === "corner-ccw") turnArg = { turn: "ccw" };
      const belt = drawBelt(entity, turnArg);
      belt.scale.set(scale);
      g.addChild(belt);
    });
  }

  if (UG_BELT_ENTITIES.has(entity.name)) {
    const ioType = (entity.io_type as "input" | "output") ?? "input";
    const key = ugBeltAtlasKey(entity.name, entity.direction ?? "North", ioType);
    return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
      const scale = CELL_PX / TILE_PX;
      const gfx = drawEntityGraphic(entity, ctx);
      gfx.scale.set(scale);
      gfx.x = 0;
      gfx.y = 0;
      g.addChild(gfx);
    });
  }

  if (SPLITTER_ENTITIES.has(entity.name)) {
    const key = splitterAtlasKey(entity.name, entity.direction ?? "North");
    const isNS = entity.direction === "North" || entity.direction === "South";
    const wCells = isNS ? 2 : 1;
    const hCells = isNS ? 1 : 2;
    return getMultiCellTexture(key, wCells, hCells, (g, wPx, hPx) => {
      // drawSplitter draws at TILE_PX scale; we need to scale it up to CELL_PX.
      const scale = CELL_PX / TILE_PX;
      const gfx = drawEntityGraphic(entity, ctx);
      gfx.scale.set(scale);
      gfx.x = 0;
      gfx.y = 0;
      // Clip/mask to the allocated dimensions.
      void wPx; void hPx;
      g.addChild(gfx);
    });
  }

  if (entity.name === "pipe") {
    const mask = computePipeConnectionMask(entity, ctx);
    const key = pipeAtlasKey(mask);
    return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
      const scale = CELL_PX / TILE_PX;
      const gfx = drawEntityGraphic(entity, ctx);
      gfx.scale.set(scale);
      gfx.x = 0;
      gfx.y = 0;
      g.addChild(gfx);
    });
  }

  if (entity.name === "pipe-to-ground") {
    const key = ptgAtlasKey(entity.direction ?? "North");
    return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
      const scale = CELL_PX / TILE_PX;
      const gfx = drawEntityGraphic(entity, ctx);
      gfx.scale.set(scale);
      gfx.x = 0;
      gfx.y = 0;
      g.addChild(gfx);
    });
  }

  if (INSERTER_ENTITIES.has(entity.name)) {
    const key = inserterAtlasKey(entity.name, entity.direction ?? "North");
    return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
      const scale = CELL_PX / TILE_PX;
      const gfx = drawEntityGraphic(entity, ctx);
      gfx.scale.set(scale);
      gfx.x = 0;
      gfx.y = 0;
      g.addChild(gfx);
    });
  }

  if (POLE_ENTITIES.has(entity.name)) {
    const key = poleAtlasKey(entity.name);
    return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
      const scale = CELL_PX / TILE_PX;
      const gfx = drawEntityGraphic(entity, ctx);
      gfx.scale.set(scale);
      gfx.x = 0;
      gfx.y = 0;
      g.addChild(gfx);
    });
  }

  if (MACHINE_ENTITIES.has(entity.name)) {
    const [tw, th] = MACHINE_SIZES[entity.name] ?? [1, 1];
    const key = machineAtlasKey(entity.name);
    return getMultiCellTexture(key, tw, th, (g, wPx, hPx) => {
      // Try entity-frame PNG first (same logic as entities.ts:drawMachine).
      const framePath = `${import.meta.env.BASE_URL}entity-frames/${entity.name}.png`;
      const frameTex: Texture | null = Cache.has(framePath) ? (Assets.get<Texture>(framePath) ?? null) : null;
      const spriteScale = 1.5;
      const scale = CELL_PX / ENTITY_FRAME_TILE_PX;

      if (frameTex) {
        const sprite = new Sprite(frameTex);
        sprite.scale.set(scale * spriteScale);
        const pw = wPx;
        const ph = hPx;
        sprite.x = -pw * (spriteScale - 1) / 2;
        sprite.y = -ph * (spriteScale - 1) / 2;
        g.addChild(sprite);
      } else {
        // Fallback: colored rect matching drawMachine's placeholder.
        const MACHINE_COLORS: Record<string, number> = {
          "assembling-machine-1": 0x5a6e82, "assembling-machine-2": 0x4a6278,
          "assembling-machine-3": 0x3a526a, "chemical-plant": 0x3a7a50,
          "oil-refinery": 0x5a3a8a, "electric-furnace": 0x6a5a80,
          "steel-furnace": 0x7a5030, "stone-furnace": 0x8a6040,
          centrifuge: 0x3a7a80, lab: 0x4a6a50, "rocket-silo": 0x4a4a6a,
          foundry: 0x8a6a30, biochamber: 0x4a7a3a, biolab: 0x3a6a5a,
          "electromagnetic-plant": 0x2a5a9a, "cryogenic-plant": 0x4a7a8a,
          recycler: 0x6a5a4a, crusher: 0x5a4a3a, beacon: 0x4a6080,
          "storage-tank": 0x4a6a5a, "electric-mining-drill": 0x7a6a30,
        };
        const color = MACHINE_COLORS[entity.name] ?? 0x4a5a6a;
        g.roundRect(2, 2, wPx - 4, hPx - 4, 3).fill({ color, alpha: 0.5 });
      }
    });
  }

  // Generic fallback.
  const key = `generic:${entity.name}`;
  return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
    const scale = CELL_PX / TILE_PX;
    const gfx = drawEntityGraphic(entity, ctx);
    gfx.scale.set(scale);
    gfx.x = 0;
    gfx.y = 0;
    g.addChild(gfx);
  });
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Commit one entity as a particle pair (entity + optional icon).
 *
 * Sets `particle.alpha = 0` initially; call `applyParticleReveals` from the
 * live-phase ticker to animate alpha to 1 over `FADE_IN_MS`.
 * Particles persist at full alpha after the fade — no removal on settle.
 *
 * Handles all entity types: belts, UG belts, splitters, pipes, inserters,
 * poles, machines.
 *
 * For machines, no icon particle is added (machines don't carry items).
 */
export function commitEntityAsParticle(
  scene: ParticleScene,
  entity: PlacedEntity,
  revealAt: number,
  ctx: DrawContext = createDrawContext(),
): void {
  const key = entityKey(entity);
  if (particleMap.has(key)) return; // idempotent

  const ex = entity.x ?? 0;
  const ey = entity.y ?? 0;

  // --- Entity particle ---
  const entityTex = getEntityAtlasTexture(entity, ctx);

  let scaleX = TILE_PX / CELL_PX;
  let scaleY = TILE_PX / CELL_PX;

  if (SPLITTER_ENTITIES.has(entity.name)) {
    const isNS = entity.direction === "North" || entity.direction === "South";
    const wCells = isNS ? 2 : 1;
    const hCells = isNS ? 1 : 2;
    // Texture is wCells*CELL_PX × hCells*CELL_PX; display at wCells*TILE_PX × hCells*TILE_PX.
    scaleX = (wCells * TILE_PX) / (wCells * CELL_PX);
    scaleY = (hCells * TILE_PX) / (hCells * CELL_PX);
  } else if (MACHINE_ENTITIES.has(entity.name)) {
    const [tw, th] = MACHINE_SIZES[entity.name] ?? [1, 1];
    // Texture is tw*CELL_PX × th*CELL_PX; display at tw*TILE_PX × th*TILE_PX.
    scaleX = (tw * TILE_PX) / (tw * CELL_PX);
    scaleY = (th * TILE_PX) / (th * CELL_PX);
  }

  const px = ex * TILE_PX;
  const py = ey * TILE_PX;

  const ep = new Particle({
    texture: entityTex,
    x: px,
    y: py,
    alpha: 0,
    anchorX: 0,
    anchorY: 0,
    scaleX,
    scaleY,
  });
  // Machines render above belts; everything else goes into beltContainer.
  if (MACHINE_ENTITIES.has(entity.name)) {
    scene.machineContainer.addParticle(ep);
  } else {
    scene.beltContainer.addParticle(ep);
  }

  // --- Icon particle (if the entity carries an item, and isn't a machine) ---
  let iconParticle: Particle | undefined;
  if (entity.carries && !MACHINE_ENTITIES.has(entity.name)) {
    const iconTex = getItemIconTexture(entity.carries);
    // Center a 14 px icon on the tile.
    const iconSize = TILE_PX * 0.5;
    const iconOffset = (TILE_PX - iconSize) / 2;
    iconParticle = new Particle({
      texture: iconTex,
      x: px + iconOffset,
      y: py + iconOffset,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: iconSize / CELL_PX,
      scaleY: iconSize / CELL_PX,
    });
    scene.iconContainer.addParticle(iconParticle);
  }

  particleMap.set(key, { entity: ep, icon: iconParticle, revealAt, placedEntity: entity });
}

/**
 * Add a ghost belt particle to the ghost container with item-color tint.
 * Ghost belts fade out fully before being removed from the container.
 *
 * `specKey` is used to de-duplicate crossings: a second ghost at the same
 * tile from the same spec is a no-op; from a different spec it adds an
 * overlay particle.
 *
 * Returns the created Particle (or null if skipped).
 */
const ghostByTile = new Map<string, { particle: Particle; specKey: string }>();

export function addGhostParticle(
  scene: ParticleScene,
  x: number,
  y: number,
  direction: EntityDirection,
  item: string,
  revealAt: number,
  specKey: string,
): Particle | null {
  const tk = `${x},${y}`;
  const existing = ghostByTile.get(tk);
  if (existing && existing.specKey === specKey) return null; // same spec, skip

  // Build a synthetic belt entity for texture lookup.
  const synthEntity: PlacedEntity = {
    name: "transport-belt",
    x,
    y,
    direction,
    recipe: null,
    carries: item,
    segment_id: null,
    io_type: null,
  } as unknown as PlacedEntity;

  const ctx = createDrawContext();
  addEntityToDrawContext(synthEntity, ctx);
  const tex = getEntityAtlasTexture(synthEntity, ctx);
  const tint = itemColor(item);
  const p = new Particle({
    texture: tex,
    x: x * TILE_PX,
    y: y * TILE_PX,
    alpha: 0,
    anchorX: 0,
    anchorY: 0,
    scaleX: TILE_PX / CELL_PX,
    scaleY: TILE_PX / CELL_PX,
    tint,
  });
  scene.ghostContainer.addParticle(p);
  if (!existing) {
    ghostByTile.set(tk, { particle: p, specKey });
  }
  // Schedule fade-in via revealAt in the animation loop.
  // Ghost particle alpha is mutated by commitGhostParticleReveals.
  void revealAt; // fade-in handled by caller's animation loop
  return p;
}

/**
 * Remove ghost particle at a tile — called when a committed entity
 * replaces a ghost belt.
 */
export function removeGhostParticle(
  scene: ParticleScene,
  x: number,
  y: number,
): void {
  const tk = `${x},${y}`;
  const entry = ghostByTile.get(tk);
  if (!entry) return;
  scene.ghostContainer.removeParticle(entry.particle);
  ghostByTile.delete(tk);
}

/**
 * Remove a particle by entity key — for fade-out completion cleanup.
 */
export function removeParticleAt(
  scene: ParticleScene,
  key: string,
): void {
  const entry = particleMap.get(key);
  if (!entry) return;
  // Entity particle lives in machineContainer or beltContainer depending on type.
  if (MACHINE_ENTITIES.has(entry.placedEntity.name)) {
    scene.machineContainer.removeParticle(entry.entity);
  } else {
    scene.beltContainer.removeParticle(entry.entity);
  }
  if (entry.icon) scene.iconContainer.removeParticle(entry.icon);
  particleMap.delete(key);
}

/**
 * Walk all registered particles and update alpha based on
 * `(now - revealAt) / FADE_IN_MS`. Mirrors `applyReveals` in
 * `streamingRenderer.ts`. Called from the live-phase ticker.
 *
 * `now` is the virtual-ms clock value (same clock as `revealAt`).
 */
export function applyParticleReveals(
  _scene: ParticleScene,
  now: number,
): void {
  for (const entry of particleMap.values()) {
    const alpha = Math.min(1, Math.max(0, (now - entry.revealAt) / FADE_IN_MS));
    entry.entity.alpha = alpha;
    if (entry.icon) entry.icon.alpha = alpha;
  }
}

/**
 * Iterate (particle, revealAt) pairs for scrub-mode integration.
 *
 * Yields one entry per entity (the entity particle + optional icon particle).
 * Called by `streamingRenderer.ts:finish()` to build the `reveals` list.
 */
export function* getParticleReveals(
  _scene: ParticleScene,
): Iterable<{ particle: Particle; iconParticle: Particle | undefined; revealAt: number }> {
  for (const entry of particleMap.values()) {
    yield { particle: entry.entity, iconParticle: entry.icon, revealAt: entry.revealAt };
  }
}

// ---------------------------------------------------------------------------
// Non-streaming path: renderLayoutAsParticles
// ---------------------------------------------------------------------------

/**
 * Render a fully-built layout into a `ParticleScene` without going through
 * the streaming event machinery.
 *
 * This replaces `renderLayout(layout, container)` for corpus / parsed-blueprint
 * paths. Builds the draw context in one pass (for belt turn detection, pipe
 * connection masks), then commits each entity as a particle with `revealAt = 0`
 * so all particles start fully visible.
 *
 * Returns a particle-aware `HighlightController`.
 */
export function renderLayoutAsParticles(
  layout: LayoutResult,
  scene: ParticleScene,
): HighlightController {
  // Clear any previous particles.
  scene.clear();
  scene.layout = layout;

  // Build draw context in one pass so belt turn detection and pipe connections
  // are correct by construction when we commit each entity.
  const ctx = createDrawContext();
  for (const e of layout.entities) {
    addEntityToDrawContext(e, ctx);
  }

  // Commit every entity with revealAt = 0 (fully visible immediately).
  const revealAt = 0;
  for (const e of layout.entities) {
    commitEntityAsParticle(scene, e, revealAt, ctx);
  }

  // Set all particles to alpha=1 immediately.
  applyParticleReveals(scene, FADE_IN_MS + 1);

  // Build belt graph for highlight controller.
  const beltGraph = buildBeltGraph(layout);

  // Belt network overlay Graphics — drawn on top when a network is highlighted.
  // Persists between highlight calls; replaced on each new highlight.
  let overlayGraphics: Graphics | null = null;

  // The container that the scene is attached to — we need it for the overlay.
  // Since attachTo is called separately, we hold a reference.
  let overlayParent: Container | null = null;

  function getOverlayParent(): Container | null {
    // Reach the parent container via the beltContainer's parent.
    return scene.beltContainer.parent as Container | null;
  }

  function clearHighlightInternal(): void {
    if (overlayGraphics) {
      const parent = overlayParent ?? getOverlayParent();
      if (parent) parent.removeChild(overlayGraphics);
      overlayGraphics.destroy();
      overlayGraphics = null;
    }
    for (const entry of particleMap.values()) {
      entry.entity.alpha = 1;
      if (entry.icon) entry.icon.alpha = 1;
    }
  }

  return {
    highlightItem(item: string | null): void {
      clearHighlightInternal();
      if (!item) return;
      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const chainKey = pe.carries ?? pe.recipe ?? null;
        const dim = chainKey !== item;
        const alpha = dim ? 0.15 : 1;
        entry.entity.alpha = alpha;
        if (entry.icon) entry.icon.alpha = alpha;
      }
    },

    highlightBeltNetwork(entity: PlacedEntity | null): void {
      clearHighlightInternal();
      if (!entity) return;

      const startKey = `${entity.x ?? 0},${entity.y ?? 0}`;
      const anchor = beltGraph.tileToAnchor.get(startKey) ?? startKey;
      if (!beltGraph.nodes.has(anchor)) return;

      const { downstream, upstream } = traceBeltNetwork(anchor, beltGraph);
      const allBelt = new Set([...downstream, ...upstream]);
      const inserters = findAdjacentInserters(allBelt, beltGraph.entityMap);
      const machines = findAdjacentMachines(inserters, beltGraph.entityMap);

      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const k = `${pe.x ?? 0},${pe.y ?? 0}`;
        let alpha: number;
        if (allBelt.has(k)) {
          alpha = 0.5;
        } else if (inserters.has(k)) {
          alpha = 0.9;
        } else if (machines.has(k)) {
          alpha = 0.75;
        } else {
          alpha = 0.15;
        }
        entry.entity.alpha = alpha;
        if (entry.icon) entry.icon.alpha = alpha;
      }

      const parent = getOverlayParent();
      if (parent) {
        overlayParent = parent;
        overlayGraphics = new Graphics();
        drawBeltNetworkOverlay(overlayGraphics, downstream, upstream, anchor, beltGraph);
        parent.addChild(overlayGraphics);
      }
    },

    clearHighlight(): void {
      clearHighlightInternal();
    },

    chainKey(entity: PlacedEntity): string | null {
      return entity.carries ?? entity.recipe ?? null;
    },
  };
}

// ---------------------------------------------------------------------------
// Particle-aware HighlightController (Phase 3)
// ---------------------------------------------------------------------------

/**
 * Build a particle-aware `HighlightController` that walks the `particleMap`
 * instead of a `allGraphics` array.
 *
 * Used by `streamingRenderer.ts:finish()` after all entities are particles.
 * The belt network overlay (dashed/solid lines) remains a Graphics overlay
 * added to `overlayContainer` — separate from the entities.
 */
export function createParticleHighlightController(
  layout: LayoutResult,
  overlayContainer: Container,
): HighlightController {
  const beltGraph = buildBeltGraph(layout);
  let overlayGraphics: Graphics | null = null;

  function clearHighlightInternal(): void {
    if (overlayGraphics) {
      overlayContainer.removeChild(overlayGraphics);
      overlayGraphics.destroy();
      overlayGraphics = null;
    }
    for (const entry of particleMap.values()) {
      entry.entity.alpha = 1;
      if (entry.icon) entry.icon.alpha = 1;
    }
  }

  return {
    highlightItem(item: string | null): void {
      clearHighlightInternal();
      if (!item) return;
      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const chainKey = pe.carries ?? pe.recipe ?? null;
        const alpha = chainKey === item ? 1 : 0.15;
        entry.entity.alpha = alpha;
        if (entry.icon) entry.icon.alpha = alpha;
      }
    },

    highlightBeltNetwork(entity: PlacedEntity | null): void {
      clearHighlightInternal();
      if (!entity) return;

      const startKey = `${entity.x ?? 0},${entity.y ?? 0}`;
      const anchor = beltGraph.tileToAnchor.get(startKey) ?? startKey;
      if (!beltGraph.nodes.has(anchor)) return;

      const { downstream, upstream } = traceBeltNetwork(anchor, beltGraph);
      const allBelt = new Set([...downstream, ...upstream]);
      const inserters = findAdjacentInserters(allBelt, beltGraph.entityMap);
      const machines = findAdjacentMachines(inserters, beltGraph.entityMap);

      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const k = `${pe.x ?? 0},${pe.y ?? 0}`;
        let alpha: number;
        if (allBelt.has(k)) alpha = 0.5;
        else if (inserters.has(k)) alpha = 0.9;
        else if (machines.has(k)) alpha = 0.75;
        else alpha = 0.15;
        entry.entity.alpha = alpha;
        if (entry.icon) entry.icon.alpha = alpha;
      }

      overlayGraphics = new Graphics();
      drawBeltNetworkOverlay(overlayGraphics, downstream, upstream, anchor, beltGraph);
      overlayContainer.addChild(overlayGraphics);
    },

    clearHighlight(): void {
      clearHighlightInternal();
    },

    chainKey(entity: PlacedEntity): string | null {
      return entity.carries ?? entity.recipe ?? null;
    },
  };
}
