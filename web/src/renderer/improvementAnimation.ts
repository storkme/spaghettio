// Visual feedback for the auto-optimize animation queue. Spawns a
// cyan rect over a SAT zone that fades to transparent, then
// self-destructs. Fire-and-forget — no handle needed.

import { Graphics, type Application } from "pixi.js";
import type { Viewport } from "pixi-viewport";
import { TILE_PX } from "./entities";
import { beginAnimating, endAnimating } from "./app";

const FLASH_COLOR = 0x40c0e0;

export function spawnRegionFlash(
  app: Application,
  viewport: Viewport,
  zone: { x: number; y: number; w: number; h: number },
  durationMs: number = 240,
): void {
  const flashG = new Graphics();
  viewport.addChild(flashG);

  const px = zone.x * TILE_PX;
  const py = zone.y * TILE_PX;
  const pw = zone.w * TILE_PX;
  const ph = zone.h * TILE_PX;

  let elapsed = 0;
  const tick = (): void => {
    elapsed += app.ticker.deltaMS;
    const remaining = Math.max(0, (durationMs - elapsed) / durationMs);
    if (remaining <= 0) {
      app.ticker.remove(tick);
      flashG.destroy();
      endAnimating();
      return;
    }
    flashG.clear();
    flashG
      .rect(px, py, pw, ph)
      .fill({ color: FLASH_COLOR, alpha: 0.55 * remaining });
  };
  app.ticker.add(tick);
  beginAnimating();
}
