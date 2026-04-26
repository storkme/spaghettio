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
import type { Application, Ticker, Texture } from "pixi.js";
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
  getFluidPorts,
  type DrawContext,
  type HighlightController,
} from "./entities";
import { buildConnectivityGraph, bfsDistances, type EntityKey } from "./connectivityGraph";
import { beginAnimating, endAnimating, requestRender } from "./app";

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
// Highlight animation constants
// ---------------------------------------------------------------------------

/** Minimum alpha for dimmed-but-reachable entities (configurable). */
const HIGHLIGHT_FLOOR = 0.2;

/** Minimum effective max-distance so tiny graphs don't squash their gradient. */
const HIGHLIGHT_MIN_EFFECTIVE_MAX = 5;

/** Fade-in (raise alpha) duration in ms — sharp ease-out-cubic pop. */
const HIGHLIGHT_FADE_IN_MS = 100;

/** Fade-out (lower alpha) duration in ms — gentle linear fade. */
const HIGHLIGHT_FADE_OUT_MS = 200;

// ---------------------------------------------------------------------------
// Highlight animation types
// ---------------------------------------------------------------------------

const easeOutCubic = (t: number): number => 1 - Math.pow(1 - t, 3);
const easeLinear = (t: number): number => t;

interface HighlightAnimation {
  entityParticle: Particle;
  iconParticle: Particle | undefined;
  startAlpha: number;
  targetAlpha: number;
  startTime: number;
  duration: number;
  ease: (t: number) => number;
}

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
  /** Graphics layer between belts and machines: pipe-into-machine
   *  extension stubs. Drawn under machine sprites so the machine art
   *  occludes the inner end of the stub. */
  pipeStubLayer: Container;
  machineContainer: ParticleContainer;
  ghostContainer: ParticleContainer;
  iconContainer: ParticleContainer;
  /** The layout this scene was built from (set by `renderLayoutAsParticles`). */
  layout: LayoutResult | null;
  /**
   * Add the containers to `parent` in the correct z-order:
   * belt → pipeStub → machine → ghost → icon.
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
  const pipeStubLayer = new Container();
  const machineContainer = makeParticleContainer();
  const ghostContainer = makeParticleContainer();
  const iconContainer = makeParticleContainer();

  const scene: ParticleScene = {
    beltContainer,
    pipeStubLayer,
    machineContainer,
    ghostContainer,
    iconContainer,
    layout: null,
    attachTo(parent: Container): void {
      parent.addChild(beltContainer);
      parent.addChild(pipeStubLayer);
      parent.addChild(machineContainer);
      parent.addChild(ghostContainer);
      parent.addChild(iconContainer);
    },
    clear(): void {
      beltContainer.removeParticles();
      pipeStubLayer.removeChildren();
      machineContainer.removeParticles();
      ghostContainer.removeParticles();
      iconContainer.removeParticles();
      particleMap.clear();
      ghostByTile.clear();
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
 * N=1, E=2, S=4, W=8. Looks at ctx.tileMap and ctx.machineByTile.
 * Pipe-to-machine adjacency only counts as a connection when the pipe
 * sits at a real fluid port for that machine.
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
    if (nb && pipeConnectsToNeighbour(nb, dx, dy)) {
      mask |= bit;
      continue;
    }
    const machine = ctx.machineByTile.get(key);
    if (machine && pipeIsAtFluidPort(ex, ey, machine)) {
      mask |= bit;
    }
  }
  return mask;
}

/** True iff `(px, py)` is one of `machine`'s fluid ports. Inlined here
 *  to keep particleLayout independent of the entities-internal helper. */
function pipeIsAtFluidPort(px: number, py: number, machine: PlacedEntity): boolean {
  const mx = machine.x ?? 0;
  const my = machine.y ?? 0;
  for (const [rx, ry] of getFluidPorts(machine)) {
    if (mx + rx === px && my + ry === py) return true;
  }
  return false;
}

const PIPE_STUB_COLOR = 0x8a8a8a;

/** Walk every pipe in the layout and, for each pipe that sits at a real
 *  fluid port of an adjacent machine, draw a stub Graphics extending one
 *  tile from the pipe's edge into the machine. The stub layer renders
 *  under machine sprites so the machine art occludes the inner end —
 *  visually the pipe "goes into" the machine.
 *
 *  The (pipe, machine) pair is naturally deduped: each port is exactly
 *  one tile, and a pipe sits at at most one of a machine's ports, so
 *  the inner loop emits at most one stub per (pipe, machine) pair. */
function drawPipeStubsForLayout(
  scene: ParticleScene,
  layout: LayoutResult,
  ctx: DrawContext,
): void {
  const layer = scene.pipeStubLayer;
  layer.removeChildren();

  const stubW = Math.max(2, (TILE_PX - 1) * 0.4);

  for (const e of layout.entities) {
    if (e.name !== "pipe") continue;
    const ex = e.x ?? 0;
    const ey = e.y ?? 0;

    for (const [dx, dy] of [[0, -1], [1, 0], [0, 1], [-1, 0]] as [number, number][]) {
      const key = `${ex + dx},${ey + dy}`;
      const machine = ctx.machineByTile.get(key);
      if (!machine) continue;
      if (!pipeIsAtFluidPort(ex, ey, machine)) continue;

      // Pipe centre in canvas coords; stub extends from centre 1.5 tiles
      // into the machine direction (0.5 tiles to reach the pipe's tile
      // edge, then 1 tile beyond into the machine).
      const cx = ex * TILE_PX + TILE_PX / 2;
      const cy = ey * TILE_PX + TILE_PX / 2;
      const reach = TILE_PX * 1.5;
      const tx = cx + dx * reach;
      const ty = cy + dy * reach;

      const g = new Graphics();
      g.moveTo(cx, cy)
        .lineTo(tx, ty)
        .stroke({ width: stubW, color: PIPE_STUB_COLOR, cap: "round" });
      layer.addChild(g);
    }
  }
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
      const spriteScale = 1.8;
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
    // Centre the carries icon on the tile.
    const iconSize = TILE_PX * 0.35;
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
 * Drop every ghost belt particle from `scene` and clear the
 * tile-keyed tracker. Call this once streaming finishes — any
 * ghosts that weren't replaced by committed entities (because the
 * router speculated a path it didn't end up using) would otherwise
 * stay parked at alpha 0.5 forever.
 */
export function clearAllGhostParticles(scene: ParticleScene): void {
  scene.ghostContainer.removeParticles();
  ghostByTile.clear();
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
 * Drop every committed particle at tile `(x, y)`, regardless of name or
 * recipe. Returns the entity-keys that were evicted so the caller can
 * also clear them from any external "already committed" tracker (e.g.
 * `committedKeys` in streamingRenderer.ts). Used by SAT-zone commit
 * handling: the ghost router commits speculative belts before the SAT
 * solver runs, and SAT can replace those tiles with belts that have a
 * different `name` (transport-belt → underground-belt) — different
 * `entityKey` → both particles end up coexisting unless the old ones
 * are explicitly evicted first.
 */
export function evictParticlesAtTile(
  scene: ParticleScene,
  x: number,
  y: number,
): string[] {
  const evicted: string[] = [];
  for (const [key, entry] of particleMap.entries()) {
    if (entry.placedEntity.x !== x || entry.placedEntity.y !== y) continue;
    if (MACHINE_ENTITIES.has(entry.placedEntity.name)) {
      scene.machineContainer.removeParticle(entry.entity);
    } else {
      scene.beltContainer.removeParticle(entry.entity);
    }
    if (entry.icon) scene.iconContainer.removeParticle(entry.icon);
    particleMap.delete(key);
    evicted.push(key);
  }
  return evicted;
}

/**
 * Walk every committed pipe particle and refresh its atlas texture
 * against the (now-complete) draw context. Streaming commits a pipe
 * with whatever neighbours are visible at the moment its phase fires,
 * so a pipe whose neighbour appears in a later phase ends up with a
 * stale, under-connected texture. The non-streaming path doesn't hit
 * this — it stages the whole tileMap before committing — but the
 * streaming finish() does, so it calls this once at finalisation.
 *
 * Implementation note: `beltContainer` is configured with
 * `uvs: false` (see `makeParticleContainer`), so reassigning
 * `particle.texture` is a silent no-op — the UV buffer is uploaded
 * once at `addParticle` time and never refreshed. To actually swap
 * the texture we have to remove the old particle and add a fresh
 * one with the new texture, which forces the container to rebatch.
 *
 * Returns a Map<oldParticle, newParticle> for any pipe whose
 * particle was replaced, so the caller can patch its scrub-mode
 * reveals list (which holds particle references built before this
 * runs).
 */
export function refreshPipeTextures(
  scene: ParticleScene,
  ctx: DrawContext,
): Map<Particle, Particle> {
  const swaps = new Map<Particle, Particle>();
  for (const [key, entry] of particleMap.entries()) {
    if (entry.placedEntity.name !== "pipe") continue;
    const newTex = getEntityAtlasTexture(entry.placedEntity, ctx);
    if (entry.entity.texture === newTex) continue;

    const oldP = entry.entity;
    const newP = new Particle({
      texture: newTex,
      x: oldP.x,
      y: oldP.y,
      alpha: oldP.alpha,
      anchorX: oldP.anchorX,
      anchorY: oldP.anchorY,
      scaleX: oldP.scaleX,
      scaleY: oldP.scaleY,
    });
    scene.beltContainer.removeParticle(oldP);
    scene.beltContainer.addParticle(newP);
    particleMap.set(key, { ...entry, entity: newP });
    swaps.set(oldP, newP);
  }
  return swaps;
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
// Shared highlight animation engine
// ---------------------------------------------------------------------------

/**
 * Creates a shared highlight animation engine that drives per-particle
 * alpha transitions.  One instance is created per highlight controller
 * and registers a single ticker callback while animations are active.
 *
 * @param ticker  The Pixi Application ticker (app.ticker).
 */
function createHighlightAnimator(ticker: Ticker) {
  const animations = new Map<EntityKey, HighlightAnimation>();
  let tickerActive = false;

  function startTicker(): void {
    if (tickerActive) return;
    tickerActive = true;
    ticker.add(tick);
    beginAnimating();
  }

  function stopTicker(): void {
    if (!tickerActive) return;
    tickerActive = false;
    ticker.remove(tick);
    endAnimating();
  }

  function tick(): void {
    const now = performance.now();
    let anyActive = false;

    for (const [key, anim] of animations) {
      const elapsed = now - anim.startTime;
      const t = Math.min(1, elapsed / anim.duration);
      const eased = anim.ease(t);
      const alpha = anim.startAlpha + (anim.targetAlpha - anim.startAlpha) * eased;

      anim.entityParticle.alpha = alpha;
      if (anim.iconParticle) anim.iconParticle.alpha = alpha;

      if (t >= 1) {
        animations.delete(key);
      } else {
        anyActive = true;
      }
    }

    if (!anyActive) {
      stopTicker();
    }
    requestRender();
  }

  /**
   * Schedule a transition for one entity's particles.
   * If a transition is already in flight, captures current interpolated alpha
   * as the new start, then retargets to `targetAlpha`.
   */
  function animateTo(
    key: EntityKey,
    entityParticle: Particle,
    iconParticle: Particle | undefined,
    targetAlpha: number,
  ): void {
    const existing = animations.get(key);
    const now = performance.now();

    // Current actual alpha (interpolated if in-flight, otherwise the particle's value).
    let currentAlpha: number;
    if (existing) {
      const elapsed = now - existing.startTime;
      const t = Math.min(1, elapsed / existing.duration);
      currentAlpha = existing.startAlpha + (existing.targetAlpha - existing.startAlpha) * existing.ease(t);
    } else {
      currentAlpha = entityParticle.alpha;
    }

    if (Math.abs(currentAlpha - targetAlpha) < 0.001) {
      // Already at target — no animation needed.
      animations.delete(key);
      entityParticle.alpha = targetAlpha;
      if (iconParticle) iconParticle.alpha = targetAlpha;
      return;
    }

    const raising = targetAlpha > currentAlpha;
    animations.set(key, {
      entityParticle,
      iconParticle,
      startAlpha: currentAlpha,
      targetAlpha,
      startTime: now,
      duration: raising ? HIGHLIGHT_FADE_IN_MS : HIGHLIGHT_FADE_OUT_MS,
      ease: raising ? easeOutCubic : easeLinear,
    });

    startTicker();
  }

  /** Cancel all in-flight animations and set everything to `alpha`. */
  function cancelAll(alpha: number): void {
    animations.clear();
    for (const entry of particleMap.values()) {
      entry.entity.alpha = alpha;
      if (entry.icon) entry.icon.alpha = alpha;
    }
    stopTicker();
    requestRender();
  }

  /** Stop ticker and drop all state — call on controller destroy. */
  function destroy(): void {
    animations.clear();
    stopTicker();
  }

  return { animateTo, cancelAll, destroy };
}

// ---------------------------------------------------------------------------
// Shared per-entity alpha computation from BFS distances
// ---------------------------------------------------------------------------

function computeTargetAlphas(
  distances: Map<EntityKey, number>,
): Map<EntityKey, number> {
  // Find actual max distance.
  let actualMax = 0;
  for (const d of distances.values()) {
    if (d > actualMax) actualMax = d;
  }
  const effectiveMax = Math.max(actualMax, HIGHLIGHT_MIN_EFFECTIVE_MAX);

  const targets = new Map<EntityKey, number>();
  for (const [key, d] of distances) {
    const alpha = HIGHLIGHT_FLOOR + (1 - HIGHLIGHT_FLOOR) * (1 - d / effectiveMax);
    targets.set(key, alpha);
  }
  return targets;
}

/** Build entity key (tile-only, for connectivity graph lookup). */
function tileEntityKey(e: PlacedEntity): EntityKey {
  return `${e.x ?? 0},${e.y ?? 0}:${e.name}:${e.recipe ?? ""}`;
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
  app?: Application,
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

  // Pipe-to-machine extension stubs (drawn under machine sprites).
  drawPipeStubsForLayout(scene, layout, ctx);

  // Set all particles to alpha=1 immediately.
  applyParticleReveals(scene, FADE_IN_MS + 1);

  // Build connectivity graph for distance-based focus.
  const connGraph = buildConnectivityGraph(layout);

  // If no app was provided (legacy call sites), fall back to non-animated behaviour.
  if (!app) {
    return createLegacyHighlightController();
  }

  const animator = createHighlightAnimator(app.ticker);

  function applyDistances(distances: Map<EntityKey, number>): void {
    const targets = computeTargetAlphas(distances);
    for (const entry of particleMap.values()) {
      const key = tileEntityKey(entry.placedEntity);
      const target = targets.get(key) ?? HIGHLIGHT_FLOOR;
      animator.animateTo(key, entry.entity, entry.icon, target);
    }
  }

  return {
    highlightItem(item: string | null): void {
      animator.cancelAll(1);
      if (!item) return;
      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const chainKey = pe.carries ?? pe.recipe ?? null;
        const target = chainKey === item ? 1 : 0.15;
        const key = tileEntityKey(pe);
        animator.animateTo(key, entry.entity, entry.icon, target);
      }
    },

    highlightBeltNetwork(entity: PlacedEntity | null): void {
      if (!entity) {
        animator.cancelAll(1);
        return;
      }
      const startKey = tileEntityKey(entity);
      const distances = bfsDistances(connGraph, startKey);
      applyDistances(distances);
    },

    clearHighlight(): void {
      // Animate everything back to 1.0 with the fade-out (linear) curve.
      // animateTo picks the correct easing automatically (lowering alpha → linear).
      for (const entry of particleMap.values()) {
        const key = tileEntityKey(entry.placedEntity);
        animator.animateTo(key, entry.entity, entry.icon, 1);
      }
    },

    chainKey(entity: PlacedEntity): string | null {
      return entity.carries ?? entity.recipe ?? null;
    },
  };
}

// ---------------------------------------------------------------------------
// Legacy (non-animated) fallback controller — used when no app is available
// ---------------------------------------------------------------------------

function createLegacyHighlightController(): HighlightController {
  function clearAll(): void {
    for (const entry of particleMap.values()) {
      entry.entity.alpha = 1;
      if (entry.icon) entry.icon.alpha = 1;
    }
  }

  return {
    highlightItem(item: string | null): void {
      clearAll();
      if (!item) return;
      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const chainKey = pe.carries ?? pe.recipe ?? null;
        const alpha = chainKey === item ? 1 : 0.15;
        entry.entity.alpha = alpha;
        if (entry.icon) entry.icon.alpha = alpha;
      }
    },
    highlightBeltNetwork(): void {
      // No-op in legacy mode (no app = no animation, and overlay is removed).
    },
    clearHighlight(): void {
      clearAll();
    },
    chainKey(entity: PlacedEntity): string | null {
      return entity.carries ?? entity.recipe ?? null;
    },
  };
}

// ---------------------------------------------------------------------------
// Particle-aware HighlightController (streaming path)
// ---------------------------------------------------------------------------

/**
 * Build a particle-aware `HighlightController` with distance-based focus
 * dimming and smooth alpha animations.
 *
 * Used by `streamingRenderer.ts:finish()` after all entities are particles.
 * The dashed/solid Graphics overlay is removed — distance-based dimming
 * replaces it entirely.
 *
 * @param layout  The final layout (used to build the connectivity graph).
 * @param app     The Pixi Application (provides the ticker for animation).
 */
export function createParticleHighlightController(
  layout: LayoutResult,
  app: Application,
): HighlightController & { destroy(): void } {
  const connGraph = buildConnectivityGraph(layout);
  const animator = createHighlightAnimator(app.ticker);

  function applyDistanceFocus(entity: PlacedEntity): void {
    const startKey = tileEntityKey(entity);
    const distances = bfsDistances(connGraph, startKey);
    const targets = computeTargetAlphas(distances);

    for (const entry of particleMap.values()) {
      const key = tileEntityKey(entry.placedEntity);
      const target = targets.get(key) ?? HIGHLIGHT_FLOOR;
      animator.animateTo(key, entry.entity, entry.icon, target);
    }
  }

  return {
    highlightItem(item: string | null): void {
      animator.cancelAll(1);
      if (!item) return;
      for (const entry of particleMap.values()) {
        const pe = entry.placedEntity;
        const chainKey = pe.carries ?? pe.recipe ?? null;
        const target = chainKey === item ? 1 : 0.15;
        entry.entity.alpha = target;
        if (entry.icon) entry.icon.alpha = target;
      }
    },

    highlightBeltNetwork(entity: PlacedEntity | null): void {
      if (!entity) {
        // Animate back to full alpha.
        for (const entry of particleMap.values()) {
          const key = tileEntityKey(entry.placedEntity);
          animator.animateTo(key, entry.entity, entry.icon, 1);
        }
        return;
      }
      applyDistanceFocus(entity);
    },

    clearHighlight(): void {
      for (const entry of particleMap.values()) {
        const key = tileEntityKey(entry.placedEntity);
        animator.animateTo(key, entry.entity, entry.icon, 1);
      }
    },

    chainKey(entity: PlacedEntity): string | null {
      return entity.carries ?? entity.recipe ?? null;
    },

    destroy(): void {
      animator.destroy();
    },
  };
}
