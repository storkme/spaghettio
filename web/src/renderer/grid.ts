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

export function updateGrid(g: Graphics, widthTiles: number, heightTiles: number): void {
  g.clear();
  if (widthTiles <= 0 || heightTiles <= 0) return;
  const pw = widthTiles * TILE_SIZE;
  const ph = heightTiles * TILE_SIZE;
  for (let i = 0; i <= widthTiles; i++) {
    const x = i * TILE_SIZE;
    const major = i % 10 === 0;
    g.moveTo(x, 0).lineTo(x, ph).stroke({ width: major ? 1.5 : 1, color: major ? MAJOR_COLOR : MINOR_COLOR });
  }
  for (let j = 0; j <= heightTiles; j++) {
    const y = j * TILE_SIZE;
    const major = j % 10 === 0;
    g.moveTo(0, y).lineTo(pw, y).stroke({ width: major ? 1.5 : 1, color: major ? MAJOR_COLOR : MINOR_COLOR });
  }
}
