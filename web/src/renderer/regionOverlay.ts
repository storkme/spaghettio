import { Container, Graphics } from "pixi.js";
import { TILE_PX, itemColor } from "./entities";
import type { LayoutResult, LayoutRegion, EntityDirection, RegionKind, RegionPort } from "../engine";
import { classifyRegion, kindColor, classColor, type RegionClassification } from "./regionClassify";

interface LayoutRegionWithPorts {
  kind: RegionKind;
  x: number;
  y: number;
  width: number;
  height: number;
  ports?: RegionPort[];
}

// ---------------------------------------------------------------------------
// Zone colours — sourced from regionClassify (kind / class-based palettes).
// Each region is drawn with its kind as fill and class as outline, so you
// can see both channels at once.
// ---------------------------------------------------------------------------

const INPUT_COLOR = 0x50c050;  // green
const OUTPUT_COLOR = 0xd04040; // red

// ---------------------------------------------------------------------------
// Arrow drawing helper
// ---------------------------------------------------------------------------

/** Draw a directional arrow at (cx, cy) in pixel coords. */
function drawArrow(
  g: Graphics,
  cx: number,
  cy: number,
  direction: EntityDirection | undefined,
  color: number,
): void {
  const size = TILE_PX * 0.45;
  g.setStrokeStyle({ width: 3, color, alpha: 0.95 });

  // Direction vectors
  let dx = 0, dy = -1; // default North
  switch (direction) {
    case "East":  dx = 1;  dy = 0; break;
    case "South": dx = 0;  dy = 1; break;
    case "West":  dx = -1; dy = 0; break;
  }

  const tipX = cx + dx * size;
  const tipY = cy + dy * size;
  const tailX = cx - dx * size;
  const tailY = cy - dy * size;

  // Shaft
  g.moveTo(tailX, tailY).lineTo(tipX, tipY).stroke();

  // Arrowhead wings (perpendicular)
  const wingLen = size * 0.55;
  const wx = -dy * wingLen;
  const wy = dx * wingLen;
  g.moveTo(tipX - dx * wingLen + wx, tipY - dy * wingLen + wy)
    .lineTo(tipX, tipY)
    .lineTo(tipX - dx * wingLen - wx, tipY - dy * wingLen - wy)
    .stroke();
}

// ---------------------------------------------------------------------------
// Port world position
// ---------------------------------------------------------------------------

function portWorldPos(port: RegionPort): [number, number] {
  return [port.point.x, port.point.y];
}

/**
 * Draw a dashed straight line from (x0, y0) to (x1, y1). PixiJS v8 has no
 * native dashed stroke, so we segment the line and draw each dash individually.
 */
function drawDashedLine(
  g: Graphics,
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  color: number,
  width = 2,
  dashLen = 6,
  gapLen = 4,
  alpha = 0.9,
): void {
  const dx = x1 - x0;
  const dy = y1 - y0;
  const dist = Math.hypot(dx, dy);
  if (dist < 0.5) return;
  const ux = dx / dist;
  const uy = dy / dist;
  g.setStrokeStyle({ width, color, alpha });
  let traveled = 0;
  while (traveled < dist) {
    const segStart = traveled;
    const segEnd = Math.min(traveled + dashLen, dist);
    g.moveTo(x0 + ux * segStart, y0 + uy * segStart)
      .lineTo(x0 + ux * segEnd, y0 + uy * segEnd)
      .stroke();
    traveled = segEnd + gapLen;
  }
}

/**
 * Pair up input and output ports within a single region by item. For each
 * item, greedily match inputs to outputs in order — if the counts differ,
 * any leftover ports on one side are returned unpaired (they still get
 * rendered individually, just without a connecting dashed line).
 */
function pairRegionPorts(
  ports: RegionPort[],
): { item: string; inPort: RegionPort; outPort: RegionPort }[] {
  const byItem = new Map<string, { inputs: RegionPort[]; outputs: RegionPort[] }>();
  for (const p of ports) {
    const key = p.item ?? "?";
    let bucket = byItem.get(key);
    if (!bucket) {
      bucket = { inputs: [], outputs: [] };
      byItem.set(key, bucket);
    }
    if (p.io === "Input") bucket.inputs.push(p);
    else bucket.outputs.push(p);
  }
  const pairs: { item: string; inPort: RegionPort; outPort: RegionPort }[] = [];
  for (const [item, { inputs, outputs }] of byItem) {
    const n = Math.min(inputs.length, outputs.length);
    for (let i = 0; i < n; i++) {
      pairs.push({ item, inPort: inputs[i], outPort: outputs[i] });
    }
  }
  return pairs;
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

    // Boundary ports — draw input→output dashed connectors first so the
    // port markers and arrows sit on top.
    const ports = region.ports ?? [];
    const pairs = pairRegionPorts(ports);
    for (const { item, inPort, outPort } of pairs) {
      const [ix, iy] = portWorldPos(inPort);
      const [ox, oy] = portWorldPos(outPort);
      const ipx = ix * TILE_PX + TILE_PX / 2;
      const ipy = iy * TILE_PX + TILE_PX / 2;
      const opx = ox * TILE_PX + TILE_PX / 2;
      const opy = oy * TILE_PX + TILE_PX / 2;
      const lineColor = itemColor(item);
      const dashG = new Graphics();
      drawDashedLine(dashG, ipx, ipy, opx, opy, lineColor);
      layer.addChild(dashG);
    }

    for (const port of ports) {
      const [wx, wy] = portWorldPos(port);
      const px = wx * TILE_PX + TILE_PX / 2;
      const py = wy * TILE_PX + TILE_PX / 2;

      const portColor = port.io === "Input" ? INPUT_COLOR : OUTPUT_COLOR;
      const pg = new Graphics();
      pg.circle(px, py, TILE_PX * 0.3).fill({ color: portColor, alpha: 0.8 });
      layer.addChild(pg);

      const ag = new Graphics();
      const arrowColor = port.item ? itemColor(port.item) : portColor;
      drawArrow(ag, px, py, port.point.direction, arrowColor);
      layer.addChild(ag);

      // The "ele IN" / "ele OUT" per-port text labels used to sit around
      // each marker. They overlapped whenever a region had >1 port on the
      // same edge. Port identity (item + IO) is now available via hover
      // in the inspector.
    }
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
