import { Graphics, Container } from "pixi.js";
import { TILE_PX } from "./entities";

export interface ValidationIssue {
  severity: "Error" | "Warning";
  category: string;
  message: string;
  x?: number;
  y?: number;
  /** Structured delivered/needed rates for rate-shaped issues (mirrors
   *  Rust `IssueDetail`; absent for all other categories). */
  detail?: { delivered: number; needed: number };
}

const COLORS: Record<string, number> = {
  Error: 0xff4444,
  Warning: 0xffaa00,
};

/** Resting alpha for the stroke around an issue tile. The pulse helper
 *  in `issuesDialog.ts` blinks markers between this and 1.0 to draw the
 *  eye after a row click in the issues dialog. */
export const VALIDATION_BORDER_ALPHA = 0.85;

export interface ValidationOverlayResult {
  layer: Container;
  /** Map from "x,y" to all border Graphics at that tile position. */
  circleMap: Map<string, Graphics[]>;
}

export function renderValidationOverlay(
  issues: ValidationIssue[],
  container: Container,
  onHover: (text: string | null) => void,
): ValidationOverlayResult {
  const layer = new Container();
  const circleMap = new Map<string, Graphics[]>();
  for (const issue of issues) {
    if (issue.x == null || issue.y == null) continue;
    const color = COLORS[issue.severity] ?? 0x44aaff;
    const g = new Graphics();
    // Stroke-only rectangle around the tile. No fill — the entity
    // underneath stays visible (issue #209 design constraint).
    g.rect(issue.x * TILE_PX, issue.y * TILE_PX, TILE_PX, TILE_PX)
      .stroke({ width: 2, color, alpha: VALIDATION_BORDER_ALPHA });
    g.eventMode = "static";
    g.on("pointerenter", () => onHover(`[${issue.severity}] ${issue.category}: ${issue.message}`));
    g.on("pointerleave", () => onHover(null));
    layer.addChild(g);
    const key = `${issue.x},${issue.y}`;
    const existing = circleMap.get(key);
    if (existing) {
      existing.push(g);
    } else {
      circleMap.set(key, [g]);
    }
  }
  container.addChild(layer);
  return { layer, circleMap };
}
