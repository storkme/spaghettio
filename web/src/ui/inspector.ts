import type { PlacedEntity } from "../engine";
import { niceName, getRecipeFlows, isBeltEntity, type HighlightController } from "../renderer/entities";

export interface InspectorControls {
  onHover(entity: PlacedEntity | null, tileX?: number, tileY?: number): void;
  setHighlightController(ctrl: HighlightController | null): void;
  setTooltipOverride(text: string | null): void;
  /** Track the cursor's current tile so the coord line stays visible
   * regardless of what else is in the tooltip (entity info, lane
   * overlay override, etc.). Pass nulls when the cursor leaves the
   * canvas. */
  setCursorTile(x: number | null, y?: number): void;
}

export function createInspector(container: HTMLElement): InspectorControls {
  const tooltip = document.createElement("div");
  tooltip.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:200px;line-height:1.5";
  document.body.appendChild(tooltip);

  document.addEventListener("mousemove", (e) => {
    tooltip.style.left = e.clientX + 14 + "px";
    tooltip.style.top = e.clientY - 10 + "px";
  });

  let highlightCtrl: HighlightController | null = null;
  let tooltipOverride: string | null = null;
  let hoveredEntity: PlacedEntity | null = null;
  let cursorTile: { x: number; y: number } | null = null;

  function iconTag(slug: string, size = 16): string {
    return `<img src="${import.meta.env.BASE_URL}icons/${slug}.png" width="${size}" height="${size}" style="vertical-align:middle;margin-right:3px;image-rendering:pixelated" onerror="this.style.display='none'">`;
  }

  function coordLine(): string {
    if (!cursorTile) return "";
    return `<span style="color:#888">(${cursorTile.x}, ${cursorTile.y})</span>`;
  }

  function entityHtml(entity: PlacedEntity): string {
    const dirArrow: Record<string, string> = { North: "\u2191", East: "\u2192", South: "\u2193", West: "\u2190" };
    let html = `${iconTag(entity.name)}<b>${niceName(entity.name)}</b>`;
    if (entity.direction && entity.name !== "pipe") html += `<br>${dirArrow[entity.direction] ?? ""} ${entity.direction}`;
    if (entity.carries) html += `<br>${iconTag(entity.carries)} ${niceName(entity.carries)}`;
    if (entity.rate != null) html += `<br><span style="color:#b5cea8">${entity.rate.toFixed(1)}/s</span>`;
    if (entity.io_type) html += `<br>io: ${entity.io_type}`;
    if (entity.recipe) {
      html += `<br>${iconTag(entity.recipe)} ${niceName(entity.recipe)}`;
      const flows = getRecipeFlows(entity.recipe);
      if (flows) {
        for (const inp of flows.inputs) html += `<br><span style="color:#aaa">\u25b6 ${iconTag(inp.item, 14)}${niceName(inp.item)} ${inp.rate.toFixed(1)}/s</span>`;
        for (const out of flows.outputs) html += `<br><span style="color:#aaa">\u25c0 ${iconTag(out.item, 14)}${niceName(out.item)} ${out.rate.toFixed(1)}/s</span>`;
      }
    }
    if (entity.segment_id) html += `<br><span style="color:#9cdcfe">${entity.segment_id}</span>`;
    html += `<br><span style="color:#888">(${entity.x ?? 0}, ${entity.y ?? 0})</span>`;
    return html;
  }

  function render(): void {
    // Override beats everything except — we still append the cursor
    // coord so lane/row/ghost overlays stop hiding the coordinate.
    if (tooltipOverride !== null) {
      const coord = coordLine();
      tooltip.innerHTML = coord ? `${tooltipOverride}<br>${coord}` : tooltipOverride;
      tooltip.style.display = "block";
      return;
    }

    if (hoveredEntity) {
      tooltip.innerHTML = entityHtml(hoveredEntity);
      tooltip.style.display = "block";
      if (highlightCtrl) {
        if (isBeltEntity(hoveredEntity.name)) {
          highlightCtrl.highlightBeltNetwork(hoveredEntity);
        } else {
          highlightCtrl.highlightItem(highlightCtrl.chainKey(hoveredEntity));
        }
      }
      return;
    }

    if (highlightCtrl) highlightCtrl.clearHighlight();

    if (cursorTile) {
      tooltip.innerHTML = coordLine();
      tooltip.style.display = "block";
      return;
    }

    tooltip.style.display = "none";
  }

  function onHover(entity: PlacedEntity | null, tileX?: number, tileY?: number): void {
    hoveredEntity = entity;
    if (!entity && tileX !== undefined && tileY !== undefined) {
      cursorTile = { x: tileX, y: tileY };
    }
    render();
  }

  void container;

  return {
    onHover,
    setHighlightController(ctrl: HighlightController | null): void {
      highlightCtrl = ctrl;
    },
    setTooltipOverride(text: string | null): void {
      tooltipOverride = text;
      render();
    },
    setCursorTile(x: number | null, y?: number): void {
      if (x === null || y === undefined) {
        cursorTile = null;
      } else {
        cursorTile = { x, y };
      }
      render();
    },
  };
}
