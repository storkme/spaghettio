import { Container, Graphics, Text } from "pixi.js";
import { TILE_PX } from "./entities";
import type { CompositionReceipt } from "../engine";

/** Cell-composition overlay (RFC-074 Unit 2).
 *
 *  A cell-composed layout (`layout.composition`, RFC-074 Unit 1) is a
 *  chain of K identical cells, or a grid of R such strips stacked with a
 *  clearance between them. Nothing in the entity picture says so — the
 *  strips read as one wide factory. This overlay outlines each strip in
 *  layout coordinates and labels it with its copy count, so the shape
 *  the registry receipt describes ("2×12") is the shape on screen.
 *
 *  Always on when a receipt exists; nothing to toggle, nothing drawn for
 *  native layouts. Decorative: never intercepts pointer events.
 */

const STROKE = 0xf0c060;
const STROKE_ALPHA = 0.75;
const LABEL_ALPHA = 0.85;

export function renderCompositionOverlay(
  receipt: CompositionReceipt,
  container: Container,
): Container | null {
  if (!receipt.strips || receipt.strips.length === 0) return null;

  const layer = new Container();
  layer.eventMode = "none";

  const g = new Graphics();
  g.setStrokeStyle({ width: 3, color: STROKE, alpha: STROKE_ALPHA });
  for (const s of receipt.strips) {
    // Outline sits half a tile outside the strip so it never covers the
    // strip's own edge entities.
    const pad = TILE_PX / 2;
    g.rect(
      s.x * TILE_PX - pad,
      s.y * TILE_PX - pad,
      s.width * TILE_PX + 2 * pad,
      s.height * TILE_PX + 2 * pad,
    ).stroke();
  }
  layer.addChild(g);

  receipt.strips.forEach((s, i) => {
    const label = new Text({
      text: receipt.strips.length > 1
        ? `strip ${i + 1}/${receipt.strips.length} · ${s.copies} copies`
        : `${receipt.kind} · ${s.copies} copies`,
      style: { fontFamily: "monospace", fontSize: 14, fill: STROKE },
    });
    label.alpha = LABEL_ALPHA;
    // Above the strip's top-left corner, clear of the outline.
    label.x = s.x * TILE_PX;
    label.y = s.y * TILE_PX - TILE_PX / 2 - label.height - 2;
    layer.addChild(label);
  });

  container.addChild(layer);
  return layer;
}
