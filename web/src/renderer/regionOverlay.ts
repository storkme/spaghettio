import { Container, Graphics } from "pixi.js";
import { TILE_PX } from "./entities";
import type { LayoutResult, LayoutRegion, RegionKind } from "../engine";
import { classifyRegion, kindColor, classColor, type RegionClassification } from "./regionClassify";

interface LayoutRegionWithPorts {
  kind: RegionKind;
  x: number;
  y: number;
  width: number;
  height: number;
  ports?: unknown[];
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface RegionOverlayItem {
  region: LayoutRegion;
  classification: RegionClassification;
  bboxPixels: { x: number; y: number; w: number; h: number };
}

export interface RegionOverlayResult {
  layer: Container;
  items: RegionOverlayItem[];
  /** Returns the region whose bbox contains the given world-pixel point, or null. */
  hitTest: (wx: number, wy: number) => RegionOverlayItem | null;
}

export function renderRegionOverlayDetailed(layout: LayoutResult): RegionOverlayResult {
  const layer = new Container();
  const regions = (layout.regions ?? []) as LayoutRegionWithPorts[];
  const items: RegionOverlayItem[] = [];

  if (regions.length === 0) {
    return { layer, items, hitTest: () => null };
  }

  for (const region of regions) {
    const classification = classifyRegion(region as LayoutRegion);
    const fillColor = kindColor(region.kind);
    const strokeColor = classColor(classification.cls);

    const rx = region.x * TILE_PX;
    const ry = region.y * TILE_PX;
    const rw = region.width * TILE_PX;
    const rh = region.height * TILE_PX;

    items.push({
      region: region as LayoutRegion,
      classification,
      bboxPixels: { x: rx, y: ry, w: rw, h: rh },
    });

    const rect = new Graphics();
    const fillAlpha = region.kind === "crossing_zone" ? 0.06 : 0.14;
    rect.rect(rx, ry, rw, rh).fill({ color: fillColor, alpha: fillAlpha });
    // Thin dark outer edge for contrast against light belts
    rect.setStrokeStyle({ width: 1, color: 0x000000, alpha: 0.55 });
    rect.rect(rx - 1, ry - 1, rw + 2, rh + 2).stroke();
    // Class-colored inner border (this is the "classification visible" channel)
    rect.setStrokeStyle({ width: 2, color: strokeColor, alpha: 0.85 });
    rect.rect(rx, ry, rw, rh).stroke();
    layer.addChild(rect);
    // The "4×1 no-ports" corner label used to sit here. Region dimensions
    // are self-evident from the bbox; the class is available on the
    // `classification.cls` field if a future panel wants to show it.

    // Port markers (red/green circles + arrows + dashed connectors) were
    // removed — they cluttered the overlay without adding actionable info.
    // Port identity (item + IO) is available via hover in the inspector.
  }

  // Hit test: smallest-area containing region wins, so nested rectangles
  // prefer the inner one.
  const hitTest = (wx: number, wy: number): RegionOverlayItem | null => {
    let best: RegionOverlayItem | null = null;
    let bestArea = Infinity;
    for (const it of items) {
      const b = it.bboxPixels;
      if (wx >= b.x && wx < b.x + b.w && wy >= b.y && wy < b.y + b.h) {
        const area = b.w * b.h;
        if (area < bestArea) {
          bestArea = area;
          best = it;
        }
      }
    }
    return best;
  };

  return { layer, items, hitTest };
}
