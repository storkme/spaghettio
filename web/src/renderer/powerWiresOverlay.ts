import { Graphics, Container } from "pixi.js";
import { TILE_PX, MACHINE_SIZES } from "./entities";
import type { PlacedEntity } from "../engine";

/** Power-connectivity overlay (pole copper-wire network).
 *
 *  Draws the exact copper-wire graph the blueprint exports: each entry in
 *  `power_wires` is a `[indexA, indexB]` pair into `entities`, meaning those
 *  two pole entities are wired together. We resolve each index to its pole,
 *  take the footprint centre, and stroke a thin copper line between them.
 *
 *  The indices already point at poles (medium-electric-pole / substation), so
 *  there's nothing to filter — a missing index just skips that wire.
 */

/** Footprint centre of an entity in world pixels. Poles are 1×1 unless the
 *  size table says otherwise (substation is 2×2). */
function centerOf(e: PlacedEntity): { cx: number; cy: number } {
  const [w, h] = MACHINE_SIZES[e.name] ?? [1, 1];
  const x = e.x ?? 0;
  const y = e.y ?? 0;
  return { cx: (x + w / 2) * TILE_PX, cy: (y + h / 2) * TILE_PX };
}

export function renderPowerWiresOverlay(
  powerWires: [number, number][],
  entities: PlacedEntity[],
  container: Container,
): Container | null {
  if (!powerWires || powerWires.length === 0) return null;

  const layer = new Container();
  // Purely decorative: never intercept pointer events — entity hover/pin
  // stays in charge.
  layer.eventMode = "none";

  const g = new Graphics();
  for (const [a, b] of powerWires) {
    const ea = entities[a];
    const eb = entities[b];
    if (!ea || !eb) continue;
    const { cx: ax, cy: ay } = centerOf(ea);
    const { cx: bx, cy: by } = centerOf(eb);
    g.moveTo(ax, ay);
    g.lineTo(bx, by);
  }
  // Copper/orange, thin, slightly translucent so belts read underneath.
  g.stroke({ color: 0xff9a3c, width: 1.5, alpha: 0.85 });
  layer.addChild(g);

  container.addChild(layer);
  return layer;
}
