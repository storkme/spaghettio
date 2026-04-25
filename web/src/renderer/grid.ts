import { Graphics } from "pixi.js";
import type { Viewport } from "pixi-viewport";

const TILE_SIZE = 32;
const MINOR_COLOR = 0x2a2a2a;
const MAJOR_COLOR = 0x3a3a3a;

export function drawGrid(viewport: Viewport): Graphics {
  const g = new Graphics();
  viewport.addChildAt(g, 0);
  return g;
}

/** Draw the grid lines covering `widthTiles × heightTiles`.
 *
 *  `revealFraction` (default 1) clips the drawing to the top portion of the
 *  grid, drawing only down to `heightTiles × TILE_SIZE × revealFraction`.
 *  Used by the streaming-layout fade-in to wipe the grid in top-to-bottom
 *  on first reveal. Pass 1 for a fully-drawn grid. */
export function updateGrid(
  g: Graphics,
  widthTiles: number,
  heightTiles: number,
  revealFraction: number = 1,
): void {
  g.clear();
  if (widthTiles <= 0 || heightTiles <= 0) return;
  const t = Math.max(0, Math.min(1, revealFraction));
  if (t === 0) return;
  const pw = widthTiles * TILE_SIZE;
  const ph = heightTiles * TILE_SIZE;
  const visibleH = ph * t;
  // Vertical lines — clip each to the revealed height.
  for (let i = 0; i <= widthTiles; i++) {
    const x = i * TILE_SIZE;
    const major = i % 10 === 0;
    g.moveTo(x, 0).lineTo(x, visibleH).stroke({
      width: major ? 1.5 : 1,
      color: major ? MAJOR_COLOR : MINOR_COLOR,
    });
  }
  // Horizontal lines — only draw those whose y has been revealed.
  for (let j = 0; j <= heightTiles; j++) {
    const y = j * TILE_SIZE;
    if (y > visibleH) break;
    const major = j % 10 === 0;
    g.moveTo(0, y).lineTo(pw, y).stroke({
      width: major ? 1.5 : 1,
      color: major ? MAJOR_COLOR : MINOR_COLOR,
    });
  }
}
