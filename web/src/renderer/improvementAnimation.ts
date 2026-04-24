// Visual feedback for the interactive "Improve region" pass. Shows a
// soft cyan pulse around the zone bbox while the solver is running,
// and a burst flash whenever a cheaper solution is found.
//
// Drawn on its own overlay Container so the main render pipeline
// doesn't need to know about it. Tear down with `handle.stop()`.

import { Container, Graphics, type Application } from "pixi.js";
import type { Viewport } from "pixi-viewport";
import { TILE_PX } from "./entities";

const PULSE_COLOR = 0x40c0e0;
const FLASH_COLOR = 0x9fffd8;

export interface ImprovementAnimationHandle {
  /** Notify that a cheaper solution was just committed — triggers a flash. */
  flash(): void;
  /** Teardown. Removes overlay, cancels ticker. */
  stop(): void;
}

/**
 * Start the improve-region animation on a given zone.
 *
 * `zone` — world-tile bbox of the SAT zone being improved.
 * Returns a handle: call `flash()` whenever an improvement arrives,
 * `stop()` when the worker resolves (or the user cancels).
 */
export function startImprovementAnimation(
  app: Application,
  viewport: Viewport,
  zone: { x: number; y: number; w: number; h: number },
): ImprovementAnimationHandle {
  const overlay = new Container();
  viewport.addChild(overlay);

  const pulseG = new Graphics();
  overlay.addChild(pulseG);

  const flashG = new Graphics();
  overlay.addChild(flashG);

  const px = zone.x * TILE_PX;
  const py = zone.y * TILE_PX;
  const pw = zone.w * TILE_PX;
  const ph = zone.h * TILE_PX;

  let t = 0; // seconds since start
  let flashUntil = 0; // `t` value at which the current flash ends; 0 = none

  const tick = (): void => {
    // PIXI 8 Ticker deltaTime is in "60fps frames" — multiply by 1/60
    // to get seconds. Prefer `deltaMS` if available.
    const deltaMs = app.ticker.deltaMS;
    t += deltaMs / 1000;

    // Pulsing outline: sinusoidal alpha 0.35..0.9 at ~1 Hz.
    const alpha = 0.35 + 0.55 * (0.5 + 0.5 * Math.sin(t * 2 * Math.PI));
    pulseG.clear();
    pulseG.setStrokeStyle({ width: 2, color: PULSE_COLOR, alpha });
    pulseG.rect(px - 2, py - 2, pw + 4, ph + 4).stroke();

    // Flash: 300ms bright fill fading out after each improvement.
    flashG.clear();
    if (t < flashUntil) {
      const remaining = (flashUntil - t) / 0.3;
      flashG
        .rect(px, py, pw, ph)
        .fill({ color: FLASH_COLOR, alpha: 0.28 * remaining });
    }
  };
  app.ticker.add(tick);

  return {
    flash() {
      flashUntil = t + 0.3;
    },
    stop() {
      app.ticker.remove(tick);
      overlay.destroy({ children: true });
    },
  };
}
