import { Graphics, Container } from "pixi.js";
import { TILE_PX, MACHINE_SIZES } from "./entities";
import type { ValidationIssue } from "./validationOverlay";
import type { PlacedEntity } from "../engine";

/** Starvation heatmap (RFP validation-explainability, D3 Phase 1).
 *
 *  Tints machines by how starved they are, using the structured
 *  `detail.delivered / detail.needed` pair carried on rate-shaped
 *  validation issues (input-rate-delivery, inserter-item-throughput).
 *  Machines with no such issue are at/above threshold and get no tint —
 *  the overlay only marks what the validator flagged; it never guesses.
 *
 *  Issue anchoring differs per check: inserter-item-throughput sits at
 *  the machine origin (footprint-tinted via MACHINE_SIZES); input-rate-
 *  delivery sits at the pickup belt tile (single-tile tint). Where
 *  several issues touch one machine, the worst ratio wins.
 */

/** ratio 0 (fully starved) → saturated red; ratio → 1 fades out. */
function heatColor(ratio: number): { color: number; alpha: number } {
  const r = Math.max(0, Math.min(1, ratio));
  // red → orange as supply improves; alpha fades toward fed.
  const g = Math.round(0x40 + 0x60 * r);
  return { color: (0xff << 16) | (g << 8) | 0x20, alpha: 0.45 * (1 - r) + 0.12 };
}

export function renderStarvationHeatmap(
  issues: ValidationIssue[],
  entities: PlacedEntity[],
  container: Container,
): Container | null {
  // Worst delivered/needed ratio per anchor tile.
  const worstByTile = new Map<string, number>();
  for (const issue of issues) {
    if (issue.x == null || issue.y == null || !issue.detail) continue;
    const { delivered, needed } = issue.detail;
    if (!(needed > 0)) continue;
    const ratio = Math.max(0, Math.min(1, delivered / needed));
    const key = `${issue.x},${issue.y}`;
    const prev = worstByTile.get(key);
    if (prev === undefined || ratio < prev) worstByTile.set(key, ratio);
  }
  if (worstByTile.size === 0) return null;

  // Machine origins so machine-anchored issues tint the full footprint.
  const machineAt = new Map<string, [number, number]>();
  for (const e of entities) {
    const size = MACHINE_SIZES[e.name];
    if (size) machineAt.set(`${e.x},${e.y}`, size);
  }

  const layer = new Container();
  // Purely decorative: never intercept pointer events — validation
  // markers' hover/click stays in charge.
  layer.eventMode = "none";
  for (const [key, ratio] of worstByTile) {
    const [x, y] = key.split(",").map(Number);
    const size = machineAt.get(key);
    const [w, h] = size ?? [1, 1];
    const { color, alpha } = heatColor(ratio);
    const g = new Graphics();
    g.rect(x * TILE_PX, y * TILE_PX, w * TILE_PX, h * TILE_PX).fill({ color, alpha });
    layer.addChild(g);
  }
  container.addChild(layer);
  return layer;
}
