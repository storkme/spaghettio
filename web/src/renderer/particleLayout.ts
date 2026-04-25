/**
 * Particle-scene builder skeleton for Phase 1a.
 *
 * Provides `ParticleScene` — two `ParticleContainer`s (entities + icons) —
 * and helpers for committing entities as particles with fade-in reveals.
 *
 * Phase 1 scope: only South-facing trunk belts are routed through here by the
 * streaming renderer (Phase 1b). The helpers below already handle any entity
 * type so Phase 2 can extend without changing the API.
 */

import {
  Container,
  Particle,
  ParticleContainer,
} from "pixi.js";
import type { PlacedEntity } from "../engine";
import {
  beltAtlasKey,
  CELL_PX,
  getEntityTexture,
  getItemIconTexture,
} from "./atlas";
import { drawBelt, TILE_PX } from "./entities";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Fade-in duration in virtual milliseconds — mirrors streamingRenderer.ts. */
export const FADE_IN_MS = 150;

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/** Per-entity particle pair stored for alpha animation and diagnostics. */
interface ParticleEntry {
  entity: Particle;
  icon?: Particle;
  revealAt: number;
}

// ---------------------------------------------------------------------------
// ParticleScene
// ---------------------------------------------------------------------------

/**
 * Top-level container for the particle-based render path.
 * One `ParticleContainer` for entity textures, one for item icons.
 * Both use `dynamicProperties: { color: true }` so per-particle
 * alpha/tint mutations are cheap (no position/rotation buffer invalidation).
 */
export interface ParticleScene {
  entityContainer: ParticleContainer;
  iconContainer: ParticleContainer;
  /**
   * Add the parent `ParticleContainer`s to `parent` in the right z-order
   * (entities first, icons on top).
   */
  attachTo(parent: Container): void;
  /**
   * Reset for a fresh layout — removes all particles and clears the
   * internal entity-key map.
   */
  clear(): void;
  /** Total particle count across both containers (for diagnostics). */
  count(): number;
}

export function createParticleScene(): ParticleScene {
  const entityContainer = new ParticleContainer({
    dynamicProperties: {
      color: true,
      position: false,
      rotation: false,
      vertex: false,
      uvs: false,
    },
  });

  const iconContainer = new ParticleContainer({
    dynamicProperties: {
      color: true,
      position: false,
      rotation: false,
      vertex: false,
      uvs: false,
    },
  });

  return {
    entityContainer,
    iconContainer,
    attachTo(parent: Container): void {
      parent.addChild(entityContainer);
      parent.addChild(iconContainer);
    },
    clear(): void {
      entityContainer.removeParticles();
      iconContainer.removeParticles();
      particleMap.clear();
    },
    count(): number {
      return (
        entityContainer.particleChildren.length +
        iconContainer.particleChildren.length
      );
    },
  };
}

// ---------------------------------------------------------------------------
// Internal particle registry
// ---------------------------------------------------------------------------

/**
 * Global particle registry keyed by `${x},${y}:${entity.name}`.
 *
 * Shared across all scenes — Phase 1b routes only trunk belts here;
 * Phase 2 will extend. If two scenes were active simultaneously, this
 * would need to be per-scene. For now one scene exists at a time.
 */
const particleMap = new Map<string, ParticleEntry>();

function entityKey(entity: PlacedEntity): string {
  return `${entity.x ?? 0},${entity.y ?? 0}:${entity.name}`;
}

// ---------------------------------------------------------------------------
// Entity texture resolution
// ---------------------------------------------------------------------------

/**
 * Resolve the atlas texture for a belt entity.
 * For Phase 1 this covers all three belt tiers in any direction.
 * The `drawBelt` function from `entities.ts` is the source-of-truth
 * pixel logic; we call it at atlas-render time.
 */
function getBeltTexture(entity: PlacedEntity): import("pixi.js").Texture {
  const dir = entity.direction ?? "South";
  const key = beltAtlasKey(entity.name, dir);

  return getEntityTexture(key, CELL_PX, CELL_PX, (g) => {
    // `drawBelt` returns a Graphics centred at (0, 0) occupying TILE_PX px.
    // TILE_PX === 32; CELL_PX === 64. Scale by 2 to fill the cell.
    const scale = CELL_PX / TILE_PX;
    const belt = drawBelt(entity, null);
    belt.scale.set(scale);
    g.addChild(belt);
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
 * Phase 1 scope: only trunk belts flow through here. The function itself
 * handles any belt name so Phase 2 can extend coverage without API changes.
 */
export function commitEntityAsParticle(
  scene: ParticleScene,
  entity: PlacedEntity,
  revealAt: number,
): void {
  const key = entityKey(entity);
  if (particleMap.has(key)) return; // idempotent

  const px = (entity.x ?? 0) * TILE_PX;
  const py = (entity.y ?? 0) * TILE_PX;

  // --- Entity particle ---
  const entityTex = getBeltTexture(entity);
  const ep = new Particle({
    texture: entityTex,
    x: px,
    y: py,
    alpha: 0,
    anchorX: 0,
    anchorY: 0,
    scaleX: TILE_PX / CELL_PX,
    scaleY: TILE_PX / CELL_PX,
  });
  scene.entityContainer.addParticle(ep);

  // --- Icon particle (if the entity carries an item) ---
  let iconParticle: Particle | undefined;
  if (entity.carries) {
    const iconTex = getItemIconTexture(entity.carries);
    // Center the icon on the tile.
    const iconSize = TILE_PX * 0.5; // icon is 50% of tile width
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

  particleMap.set(key, { entity: ep, icon: iconParticle, revealAt });
}

/**
 * Walk all registered particles and update alpha based on
 * `(now - revealAt) / FADE_IN_MS`.  Mirrors `applyReveals` in
 * `streamingRenderer.ts`. Phase 1b will call this from the live-phase ticker.
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
 * Yields one entry per entity (the entity particle). Icon particles share
 * the same revealAt and are driven by the same alpha in `applyParticleReveals`,
 * so scrub-mode only needs to touch the entity particle; the icon will be
 * updated on the next live tick or via a separate `applyParticleReveals` call.
 *
 * Called by `streamingRenderer.ts:finish()` to build the `reveals` list.
 */
export function* getParticleReveals(
  _scene: ParticleScene,
): Iterable<{ particle: Particle; iconParticle: Particle | undefined; revealAt: number }> {
  for (const entry of particleMap.values()) {
    yield { particle: entry.entity, iconParticle: entry.icon, revealAt: entry.revealAt };
  }
}
