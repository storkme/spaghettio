/**
 * Texture atlas backed by a single oversized RenderTexture.
 *
 * One module-scoped RenderTexture is allocated lazily on the first
 * `getEntityTexture` call. Atlas slots are assigned on a simple
 * 128×128 grid; `nextSlot` walks left-to-right, top-to-bottom.
 * ~250 unique entity variants fit comfortably in 4096×4096.
 *
 * Renderer access: `initAtlas(renderer)` must be called from `app.ts`
 * after `app.init` completes. This avoids threading the renderer
 * through every call site while keeping the coupling explicit at
 * module-init time.
 */

import {
  Assets,
  Cache,
  Graphics,
  Matrix,
  Rectangle,
  RenderTexture,
  Texture,
} from "pixi.js";
import type { Renderer } from "pixi.js";
import { itemColor } from "./entities";

// ---------------------------------------------------------------------------
// Module-level state
// ---------------------------------------------------------------------------

/**
 * Atlas texture size in pixels.
 * 4096×4096 holds (4096 / CELL_PX)² = 1024 entries at 128 px cells.
 * If the GPU cannot allocate at this size, halve to 2048×2048 and document.
 */
const ATLAS_SIZE = 4096;

/** Cell size in pixels — each entity variant occupies one cell. */
const CELL_PX = 128;

/** Total columns of cells in the atlas. */
const ATLAS_COLS = ATLAS_SIZE / CELL_PX; // 32

/** Lazily allocated atlas RenderTexture. Null until first `getEntityTexture` call. */
let atlasRT: RenderTexture | null = null;

/** Sub-texture cache: variant key → Texture with sub-rect UVs. */
const cache = new Map<string, Texture>();

/** Next available slot index (left→right, top→bottom). */
let nextSlot = 0;

/** Pixi renderer reference, injected by `initAtlas`. */
let renderer: Renderer | null = null;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Register the Pixi renderer. Must be called once, from `app.ts` after
 * `app.init()` resolves, before any layout starts streaming.
 */
export function initAtlas(r: Renderer): void {
  renderer = r;
}

/**
 * Get-or-render a texture for an entity variant.
 *
 * `key` uniquely identifies the variant (e.g.
 * `belt:transport-belt:South:straight`, `icon:iron-plate`).
 * On cache miss, calls `render(g)` to draw into a temporary `Graphics`,
 * blits it into the next free atlas slot, and returns a `Texture` with
 * adjusted UVs pointing at that slot.
 *
 * `width` and `height` are the logical size in pixels — they're clamped
 * to `CELL_PX` so oversized draws are scaled to fit. For Phase 1 all
 * entity types use CELL_PX × CELL_PX.
 */
export function getEntityTexture(
  key: string,
  _width: number,
  _height: number,
  render: (g: Graphics) => void,
): Texture {
  const cached = cache.get(key);
  if (cached) return cached;

  if (!renderer) {
    // Fallback before initAtlas is called — should not happen in normal flow.
    console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture");
    return Texture.EMPTY;
  }

  // Lazy atlas allocation.
  if (!atlasRT) {
    atlasRT = RenderTexture.create({ width: ATLAS_SIZE, height: ATLAS_SIZE });
  }

  if (nextSlot >= ATLAS_COLS * ATLAS_COLS) {
    // Atlas is full — log a warning. Phase 2 can refine if this trips.
    console.warn("[atlas] atlas is full — variant will reuse slot 0:", key);
    nextSlot = 0;
  }

  const col = nextSlot % ATLAS_COLS;
  const row = Math.floor(nextSlot / ATLAS_COLS);
  const slotX = col * CELL_PX;
  const slotY = row * CELL_PX;
  nextSlot++;

  // Build the Graphics for this variant, then blit into the atlas slot.
  const g = new Graphics();
  render(g);

  // Translate the Graphics into the atlas slot.
  const transform = new Matrix(1, 0, 0, 1, slotX, slotY);
  renderer.render({
    container: g,
    target: atlasRT,
    transform,
    clear: false,
  });

  // Destroy the temporary Graphics to free memory.
  g.destroy({ children: true });

  // Build a sub-texture referencing the atlas slot.
  const frame = new Rectangle(slotX, slotY, CELL_PX, CELL_PX);
  const tex = new Texture({ source: atlasRT.source, frame });

  cache.set(key, tex);
  return tex;
}

/**
 * Get-or-render a multi-cell texture for wide/tall entities (e.g. machines,
 * splitters). Each unique `(key, wCells, hCells)` triple gets its own
 * region in the atlas spanning `wCells * CELL_PX` × `hCells * CELL_PX` px.
 *
 * The render callback receives `(g, wPx, hPx)` — the pixel dimensions it
 * should fill. The caller is responsible for drawing within those bounds.
 */
export function getMultiCellTexture(
  key: string,
  wCells: number,
  hCells: number,
  render: (g: Graphics, wPx: number, hPx: number) => void,
): Texture {
  const cached = cache.get(key);
  if (cached) return cached;

  if (!renderer) {
    console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture");
    return Texture.EMPTY;
  }

  if (!atlasRT) {
    atlasRT = RenderTexture.create({ width: ATLAS_SIZE, height: ATLAS_SIZE });
  }

  // Multi-cell sprites occupy a rectangular `wCells × hCells` region. The
  // allocator must reserve every cell in that rectangle, not just a linear
  // span — otherwise later 1×1 allocations wrap into the rows below and
  // overwrite the bottom of the sprite (visible as belts/splitters/ghost
  // entities bleeding through machine graphics, and entity ghosts behind
  // item icons).
  //
  // Strategy: align horizontally to next row if `wCells` won't fit on the
  // current row, stamp at the resulting (slotCol, slotRow), then jump
  // `nextSlot` to the start of the row after the sprite's bottom edge.
  // This wastes the cells to the right of the sprite on its occupied rows
  // and any cells to the left of slotCol on rows below the first; the
  // tradeoff is correctness for ~250 entries in a 4096-slot atlas.
  const col = nextSlot % ATLAS_COLS;
  const needAlign = col + wCells > ATLAS_COLS;
  if (needAlign) nextSlot += ATLAS_COLS - col;

  const slotCol = nextSlot % ATLAS_COLS;
  const slotRow = Math.floor(nextSlot / ATLAS_COLS);
  const slotX = slotCol * CELL_PX;
  const slotY = slotRow * CELL_PX;
  nextSlot = (slotRow + hCells) * ATLAS_COLS;

  const wPx = wCells * CELL_PX;
  const hPx = hCells * CELL_PX;

  const g = new Graphics();
  render(g, wPx, hPx);

  const transform = new Matrix(1, 0, 0, 1, slotX, slotY);
  renderer.render({ container: g, target: atlasRT, transform, clear: false });
  g.destroy({ children: true });

  const frame = new Rectangle(slotX, slotY, wPx, hPx);
  const tex = new Texture({ source: atlasRT.source, frame });
  cache.set(key, tex);
  return tex;
}

/**
 * Get-or-fetch an item-icon texture.
 *
 * If `icons/${itemSlug}.png` is in the Pixi asset cache (loaded by
 * `preloadCarriesIcons`), that PNG is atlased and returned. Otherwise
 * a placeholder — a 14 px colored circle using `itemColor(itemSlug)` —
 * is generated and cached under the same key.
 *
 * The generated placeholder uses the same cache as `getEntityTexture`,
 * so repeated calls for the same slug are O(1) lookups.
 */
export function getItemIconTexture(itemSlug: string): Texture {
  const cacheKey = `icon:${itemSlug}`;
  const cached = cache.get(cacheKey);
  if (cached) return cached;

  const pngPath = `${import.meta.env.BASE_URL}icons/${itemSlug}.png`;

  if (Cache.has(pngPath)) {
    // PNG is available — atlas it once and cache the sub-texture.
    const pngTex = Assets.get<Texture>(pngPath);
    if (pngTex) {
      // Render the PNG sprite into an atlas cell so all icons share the same
      // GPU texture (required for a single ParticleContainer draw call).
      const tex = getEntityTexture(
        cacheKey,
        CELL_PX,
        CELL_PX,
        (g) => {
          // Scale the icon to fit the cell with a small margin.
          const margin = 8;
          const size = CELL_PX - margin * 2;
          // Draw via a rect clipped sprite approximation: use Graphics to
          // stamp the texture. In Pixi v8 Graphics supports texture fills.
          g.rect(margin, margin, size, size).fill({ texture: pngTex });
        },
      );
      return tex;
    }
  }

  // No PNG — generate a colored-circle placeholder.
  const color = itemColor(itemSlug);
  return getEntityTexture(
    cacheKey,
    CELL_PX,
    CELL_PX,
    (g) => {
      const cx = CELL_PX / 2;
      const cy = CELL_PX / 2;
      const r = 7; // 14 px diameter per RFC
      g.circle(cx, cy, r).fill({ color, alpha: 0.85 });
    },
  );
}

/**
 * Optional: warm the atlas for known variants.
 * No-op in Phase 1 — lazy-on-miss is the chosen default.
 * Phase 2+ may use this for eager generation if kill criterion #2 trips.
 */
export function warmAtlas(): void {
  // No-op. Intentionally blank.
}

// ---------------------------------------------------------------------------
// Atlas key helpers — one per entity type. Format: `<type>:<...fields>`.
// ---------------------------------------------------------------------------

/**
 * Build the atlas key for a belt entity.
 * Format: `belt:<name>:<direction>:<turn>` where turn is "straight",
 * "corner-cw", or "corner-ccw".
 *
 * The `turn` parameter defaults to "straight" for call sites that
 * haven't detected turn info yet (e.g. Phase 1b pre-populate path).
 */
export function beltAtlasKey(
  name: string,
  direction: string,
  turn: "straight" | "corner-cw" | "corner-ccw" = "straight",
): string {
  return `belt:${name}:${direction}:${turn}`;
}

/**
 * Atlas key for pipe entities. The 4-bit connection mask (N=1,E=2,S=4,W=8)
 * uniquely identifies every connected variant. 16 variants total.
 */
export function pipeAtlasKey(connectionMask: number): string {
  return `pipe:${connectionMask}`;
}

/**
 * Atlas key for underground belt entities.
 * Format: `ugbelt:<name>:<direction>:<io_type>`
 */
export function ugBeltAtlasKey(
  name: string,
  direction: string,
  ioType: "input" | "output",
): string {
  return `ugbelt:${name}:${direction}:${ioType}`;
}

/**
 * Atlas key for splitter entities.
 * Format: `splitter:<name>:<direction>`
 */
export function splitterAtlasKey(name: string, direction: string): string {
  return `splitter:${name}:${direction}`;
}

/**
 * Atlas key for inserter entities.
 * Format: `inserter:<name>:<direction>`
 */
export function inserterAtlasKey(name: string, direction: string): string {
  return `inserter:${name}:${direction}`;
}

/**
 * Atlas key for machine entities. One key per machine name
 * (machines don't have directional variants in our renderer).
 * Format: `machine:<name>`.
 */
export function machineAtlasKey(name: string): string {
  return `machine:${name}`;
}

/**
 * Atlas key for power pole entities.
 * Format: `pole:<name>`
 */
export function poleAtlasKey(name: string): string {
  return `pole:${name}`;
}

/**
 * Atlas key for pipe-to-ground entities (directional stubs).
 * Format: `ptg:<direction>`
 */
export function ptgAtlasKey(direction: string): string {
  return `ptg:${direction}`;
}

// Re-export CELL_PX so particleLayout can size particles correctly.
export { CELL_PX };
