import { Container, Graphics } from "pixi.js";
import type { Viewport } from "pixi-viewport";
import type { PlacedEntity, LayoutResult } from "../engine";
import { TILE_PX, MACHINE_SIZES, SPLITTER_ENTITIES } from "./entities";

/** Tile footprint [w, h] for an entity. Mirrors the rendered bounding
 *  box: machines from MACHINE_SIZES, splitters 2×1 / 1×2 by direction,
 *  everything else (belts, pipes, inserters, small poles, ugs) 1×1. */
function entityFootprint(e: PlacedEntity): [number, number] {
  if (SPLITTER_ENTITIES.has(e.name)) {
    return e.direction === "East" || e.direction === "West" ? [1, 2] : [2, 1];
  }
  return MACHINE_SIZES[e.name] ?? [1, 1];
}

const SEL_COLOR = 0x00e0a0;

export interface SelectionController {
  /** Release event listeners and destroy graphics. Call when layout changes. */
  destroy(): void;
  /** Clear the current selection. */
  clear(): void;
  /** Currently selected entities. */
  getSelected(): PlacedEntity[];
  /** Serialize selection + params + note to a JSON string. */
  buildJson(params: { item: string; rate: number } | null, note: string): string;
}

export function createSelectionController(
  canvas: HTMLCanvasElement,
  viewport: Viewport,
  entityLayer: Container,
  layout: LayoutResult,
  onSelectionChange: (entities: PlacedEntity[]) => void,
): SelectionController {
  // Tile lookup
  const tileMap = new Map<string, PlacedEntity>();
  for (const e of layout.entities) {
    tileMap.set(`${e.x ?? 0},${e.y ?? 0}`, e);
  }

  let dragStart: { sx: number; sy: number } | null = null;
  let isDragging = false;
  let selected: PlacedEntity[] = [];

  // Drag rect — shown while the mouse is held
  const dragRectG = new Graphics();
  entityLayer.addChild(dragRectG);

  // Persistent selection borders
  const borderG = new Graphics();
  entityLayer.addChild(borderG);

  function toWorld(clientX: number, clientY: number): { x: number; y: number } {
    const r = canvas.getBoundingClientRect();
    return viewport.toWorld(clientX - r.left, clientY - r.top);
  }

  function redrawDragRect(currClientX: number, currClientY: number): void {
    if (!dragStart) return;
    const sw = toWorld(dragStart.sx, dragStart.sy);
    const cw = toWorld(currClientX, currClientY);
    const minX = Math.min(sw.x, cw.x);
    const minY = Math.min(sw.y, cw.y);
    const w = Math.abs(cw.x - sw.x);
    const h = Math.abs(cw.y - sw.y);

    dragRectG.clear();
    dragRectG.rect(minX, minY, w, h).fill({ color: SEL_COLOR, alpha: 0.18 });
    dragRectG.setStrokeStyle({ width: 1, color: SEL_COLOR, alpha: 0.8 });
    dragRectG.rect(minX, minY, w, h).stroke();
  }

  function redrawBorders(entities: PlacedEntity[]): void {
    borderG.clear();
    if (entities.length === 0) return;
    borderG.setStrokeStyle({ width: 1.5, color: SEL_COLOR, alpha: 0.9 });
    for (const e of entities) {
      const [tw, th] = entityFootprint(e);
      const px = (e.x ?? 0) * TILE_PX + 1;
      const py = (e.y ?? 0) * TILE_PX + 1;
      borderG.rect(px, py, tw * TILE_PX - 2, th * TILE_PX - 2).stroke();
    }
  }

  function collectEntities(currClientX: number, currClientY: number): PlacedEntity[] {
    if (!dragStart) return [];
    const sw = toWorld(dragStart.sx, dragStart.sy);
    const cw = toWorld(currClientX, currClientY);
    const minTx = Math.min(Math.floor(sw.x / TILE_PX), Math.floor(cw.x / TILE_PX));
    const maxTx = Math.max(Math.floor(sw.x / TILE_PX), Math.floor(cw.x / TILE_PX));
    const minTy = Math.min(Math.floor(sw.y / TILE_PX), Math.floor(cw.y / TILE_PX));
    const maxTy = Math.max(Math.floor(sw.y / TILE_PX), Math.floor(cw.y / TILE_PX));
    const out: PlacedEntity[] = [];
    for (let tx = minTx; tx <= maxTx; tx++) {
      for (let ty = minTy; ty <= maxTy; ty++) {
        const e = tileMap.get(`${tx},${ty}`);
        if (e) out.push(e);
      }
    }
    return out;
  }

  const onDown = (e: PointerEvent) => {
    if (e.button !== 0 || !e.shiftKey) return;
    dragStart = { sx: e.clientX, sy: e.clientY };
    isDragging = false;
  };

  const onMove = (e: PointerEvent) => {
    if (!dragStart) return;
    const dx = e.clientX - dragStart.sx;
    const dy = e.clientY - dragStart.sy;
    if (!isDragging && dx * dx + dy * dy > 36) {
      isDragging = true;
    }
    if (isDragging) {
      redrawDragRect(e.clientX, e.clientY);
    }
  };

  const onUp = (e: PointerEvent) => {
    if (e.button !== 0) return;
    if (isDragging) {
      e.stopImmediatePropagation();
      dragRectG.clear();
      selected = collectEntities(e.clientX, e.clientY);
      redrawBorders(selected);
      onSelectionChange(selected);
    } else if (dragStart !== null) {
      // Shift was held but drag threshold not reached — Shift+click.
      // Only clear selection when clicking on an entity; clicking empty
      // space is pure navigation and leaves selection untouched.
      const w = toWorld(e.clientX, e.clientY);
      const tx = Math.floor(w.x / TILE_PX);
      const ty = Math.floor(w.y / TILE_PX);
      if (tileMap.has(`${tx},${ty}`)) {
        selected = [];
        borderG.clear();
        onSelectionChange([]);
      }
    }
    // Plain click/drag with no Shift: pure navigation — leave selection alone.
    dragStart = null;
    isDragging = false;
  };

  function clearSelection(): void {
    selected = [];
    dragRectG.clear();
    borderG.clear();
    onSelectionChange([]);
  }

  // Right-click on canvas → clear selection. Always suppress the browser
  // context menu inside the canvas regardless of selection state, so the
  // canvas behaves consistently.
  const onContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    if (selected.length > 0) clearSelection();
  };

  // Escape clears selection. Only fire when there's actually something to
  // clear so we don't shadow other Escape handlers (dialogs, pin, etc.).
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && selected.length > 0) {
      clearSelection();
    }
  };

  canvas.addEventListener("pointerdown", onDown, { capture: true });
  canvas.addEventListener("pointermove", onMove, { capture: true });
  canvas.addEventListener("pointerup", onUp, { capture: true });
  canvas.addEventListener("contextmenu", onContextMenu);
  window.addEventListener("keydown", onKeyDown);

  return {
    destroy() {
      canvas.removeEventListener("pointerdown", onDown, { capture: true });
      canvas.removeEventListener("pointermove", onMove, { capture: true });
      canvas.removeEventListener("pointerup", onUp, { capture: true });
      canvas.removeEventListener("contextmenu", onContextMenu);
      window.removeEventListener("keydown", onKeyDown);
      dragRectG.destroy();
      borderG.destroy();
    },
    clear: clearSelection,
    getSelected() {
      return [...selected];
    },
    buildJson(params, note) {
      return JSON.stringify(
        {
          params,
          selected: selected.map((e) => ({
            x: e.x ?? 0,
            y: e.y ?? 0,
            name: e.name,
            direction: e.direction,
            carries: e.carries,
            recipe: e.recipe,
            rate: e.rate,
            io_type: e.io_type,
          })),
          note,
        },
        null,
        2,
      );
    },
  };
}
