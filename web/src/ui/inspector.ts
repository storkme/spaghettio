import type { PlacedEntity } from "../engine";
import { niceName, getRecipeFlows, isBeltEntity, type HighlightController } from "../renderer/entities";
import type { TileContext, TileInfo } from "./tileContext";

export interface InspectorControls {
  onHover(entity: PlacedEntity | null, tileX?: number, tileY?: number): void;
  setHighlightController(ctrl: HighlightController | null): void;
  setTooltipOverride(text: string | null): void;
  /** Track the cursor's current tile so the coord line stays visible
   * regardless of what else is in the tooltip (entity info, lane
   * overlay override, etc.). Pass nulls when the cursor leaves the
   * canvas. */
  setCursorTile(x: number | null, y?: number): void;
  /** Replace the per-tile aggregator. Call whenever a new layout lands
   * so the inspector has fresh ghost/axis/junction data to display. */
  setTileContext(ctx: TileContext | null): void;
  /** Pin the inspector to a specific tile. The pinned panel sits in a
   * fixed corner of the viewport and keeps showing full detail until
   * cleared (pass null) or re-pinned to a different tile. */
  pinTile(entity: PlacedEntity | null, x: number, y: number): void;
  /** Remove any pinned tile. */
  clearPin(): void;
  /** Current pinned tile (if any) — callers can check so they can draw
   * a highlight ring on canvas. */
  getPinnedTile(): { x: number; y: number } | null;
  /** Register a listener for pin changes so overlay layers can redraw
   * the pinned-tile highlight. */
  onPinChange(cb: (tile: { x: number; y: number } | null) => void): () => void;
}

const DIR_ARROW: Record<string, string> = {
  North: "\u2191", East: "\u2192", South: "\u2193", West: "\u2190",
};
const COMPACT_DIR: Record<string, string> = {
  N: "\u2191", E: "\u2192", S: "\u2193", W: "\u2190",
};

export function createInspector(container: HTMLElement): InspectorControls {
  const tooltip = document.createElement("div");
  tooltip.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5";
  document.body.appendChild(tooltip);

  // Pinned detail panel — fixed in the top-right of the canvas container
  // so it never obscures the hovered tile. Interactive (pointer events
  // enabled) so the user can click the close button.
  const pinned = document.createElement("div");
  pinned.style.cssText = "position:absolute;top:8px;right:8px;background:#141414;color:#e0e0e0;border:1px solid #888;padding:8px 10px;font:12px monospace;border-radius:4px;display:none;z-index:20;min-width:220px;max-width:340px;line-height:1.55;box-shadow:0 4px 14px rgba(0,0,0,0.5)";
  container.appendChild(pinned);

  document.addEventListener("mousemove", (e) => {
    tooltip.style.left = e.clientX + 14 + "px";
    tooltip.style.top = e.clientY - 10 + "px";
  });

  let highlightCtrl: HighlightController | null = null;
  let tooltipOverride: string | null = null;
  let hoveredEntity: PlacedEntity | null = null;
  let cursorTile: { x: number; y: number } | null = null;
  let tileContext: TileContext | null = null;
  let pinnedState: { entity: PlacedEntity | null; x: number; y: number } | null = null;
  const pinListeners = new Set<(tile: { x: number; y: number } | null) => void>();

  function notifyPin(): void {
    const tile = pinnedState ? { x: pinnedState.x, y: pinnedState.y } : null;
    for (const cb of pinListeners) cb(tile);
  }

  function iconTag(slug: string, size = 16): string {
    return `<img src="${import.meta.env.BASE_URL}icons/${slug}.png" width="${size}" height="${size}" style="vertical-align:middle;margin-right:3px;image-rendering:pixelated" onerror="this.style.display='none'">`;
  }

  function coordLine(x: number | null, y: number | null): string {
    if (x == null || y == null) return "";
    return `<span style="color:#888">(${x}, ${y})</span>`;
  }

  function entityHtml(entity: PlacedEntity): string {
    let html = `${iconTag(entity.name)}<b>${niceName(entity.name)}</b>`;
    if (entity.direction && entity.name !== "pipe") html += `<br>${DIR_ARROW[entity.direction] ?? ""} ${entity.direction}`;
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
    return html;
  }

  /** One-line compact ghost summary for the hover tooltip. */
  function ghostCompact(info: TileInfo): string {
    if (info.ghosts.length === 0) return "";
    if (info.ghosts.length === 1) {
      const g = info.ghosts[0];
      const arrow = g.direction ? COMPACT_DIR[g.direction] : "";
      return `<span style="color:#8af">ghost</span> ${iconTag(g.item, 12)}${g.item} ${arrow}`;
    }
    return `<span style="color:#ffa060">\u26A0 ${info.ghosts.length} ghosts crossing</span>`;
  }

  function axisCompact(info: TileInfo): string {
    if (!info.axis) return "";
    const { vert, horiz } = info.axis;
    if (vert === 0 && horiz === 0) return "";
    const conflict = vert >= 2 || horiz >= 2;
    const perp = vert >= 1 && horiz >= 1;
    const color = conflict ? "#ff6060" : perp ? "#60b0ff" : "#888";
    return `<span style="color:${color}">axis V${vert} H${horiz}</span>`;
  }

  function junctionCompact(info: TileInfo): string {
    if (!info.junction) return "";
    const j = info.junction;
    const color = j.outcome === "Solved" ? "#80d080" : j.outcome === "Capped" ? "#e0b060" : "#c06060";
    return `<span style="color:${color}">junction seed (${j.seedX},${j.seedY}) \u00b7 ${j.outcome}</span>`;
  }

  /** Expanded ghost list for the pinned panel. */
  function ghostExpanded(info: TileInfo): string {
    if (info.ghosts.length === 0) return "";
    const rows: string[] = [];
    for (const g of info.ghosts) {
      const arrow = g.direction ? COMPACT_DIR[g.direction] : "\u00b7";
      const endpointTag = g.isStart ? " <span style=\"color:#80d080\">start</span>" :
                         g.isEnd ? " <span style=\"color:#d08080\">end</span>" : "";
      rows.push(`${arrow} ${iconTag(g.item, 14)}${g.item}${endpointTag}`);
    }
    const header = info.ghosts.length >= 2
      ? `<div style="color:#ffa060;margin-top:4px">\u26A0 ${info.ghosts.length} ghost specs at this tile</div>`
      : `<div style="color:#8af;margin-top:4px">ghost</div>`;
    return header + rows.map(r => `<div style="margin-left:4px">${r}</div>`).join("");
  }

  function renderHover(): void {
    // Override beats everything except — we still append the cursor
    // coord so lane/row/ghost overlays stop hiding the coordinate.
    if (tooltipOverride !== null) {
      const coord = cursorTile ? coordLine(cursorTile.x, cursorTile.y) : "";
      tooltip.innerHTML = coord ? `${tooltipOverride}<br>${coord}` : tooltipOverride;
      tooltip.style.display = "block";
      return;
    }

    const parts: string[] = [];
    if (hoveredEntity) parts.push(entityHtml(hoveredEntity));
    if (cursorTile) {
      const info = tileContext?.lookup(cursorTile.x, cursorTile.y);
      if (info) {
        const g = ghostCompact(info);
        const a = axisCompact(info);
        const j = junctionCompact(info);
        for (const s of [g, a, j]) if (s) parts.push(s);
      }
      parts.push(coordLine(cursorTile.x, cursorTile.y));
    }

    if (parts.length === 0) {
      tooltip.style.display = "none";
      if (highlightCtrl) highlightCtrl.clearHighlight();
      return;
    }

    tooltip.innerHTML = parts.join("<br>");
    tooltip.style.display = "block";

    if (hoveredEntity && highlightCtrl) {
      if (isBeltEntity(hoveredEntity.name)) {
        highlightCtrl.highlightBeltNetwork(hoveredEntity);
      } else {
        highlightCtrl.highlightItem(highlightCtrl.chainKey(hoveredEntity));
      }
    } else if (highlightCtrl) {
      highlightCtrl.clearHighlight();
    }
  }

  function renderPinned(): void {
    if (!pinnedState) {
      pinned.style.display = "none";
      return;
    }
    const { entity, x, y } = pinnedState;
    const info = tileContext?.lookup(x, y);

    const parts: string[] = [];
    parts.push(`<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px"><span style="color:#8af;font-weight:bold">pinned</span> <span style="color:#888">(${x}, ${y})</span></div>`);

    if (entity) parts.push(entityHtml(entity));
    else parts.push(`<span style="color:#888">no entity at tile</span>`);

    if (info) {
      if (info.junction) {
        const color = info.junction.outcome === "Solved" ? "#80d080" : info.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
        parts.push(`<div style="margin-top:6px;padding-top:4px;border-top:1px solid #333">` +
          `<span style="color:${color}">junction seed (${info.junction.seedX},${info.junction.seedY})</span> <span style="color:#888">\u00b7 ${info.junction.outcome}</span>` +
          `</div>`);
      }
      if (info.axis) {
        const { vert, horiz } = info.axis;
        if (vert > 0 || horiz > 0) {
          const conflict = vert >= 2 || horiz >= 2;
          const perp = vert >= 1 && horiz >= 1;
          const label = conflict ? " same-axis conflict" : perp ? " perpendicular crossing" : "";
          const color = conflict ? "#ff6060" : perp ? "#60b0ff" : "#bbb";
          parts.push(`<div style="color:${color};margin-top:4px">axis: V=${vert} H=${horiz}${label}</div>`);
        }
      }
      const expanded = ghostExpanded(info);
      if (expanded) parts.push(expanded);
    }

    parts.push(`<div style="color:#555;margin-top:6px;font-size:10px">click elsewhere or press Esc to unpin</div>`);

    pinned.innerHTML = parts.join("");
    pinned.style.display = "block";
  }

  function render(): void {
    renderHover();
    renderPinned();
  }

  function onHover(entity: PlacedEntity | null, tileX?: number, tileY?: number): void {
    hoveredEntity = entity;
    if (tileX !== undefined && tileY !== undefined) {
      cursorTile = { x: tileX, y: tileY };
    } else if (entity) {
      cursorTile = { x: entity.x ?? 0, y: entity.y ?? 0 };
    }
    render();
  }

  // Allow Escape to unpin without the user having to aim.
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && pinnedState) {
      pinnedState = null;
      notifyPin();
      render();
    }
  });

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
    setTileContext(ctx: TileContext | null): void {
      tileContext = ctx;
      render();
    },
    pinTile(entity: PlacedEntity | null, x: number, y: number): void {
      pinnedState = { entity, x, y };
      notifyPin();
      render();
    },
    clearPin(): void {
      pinnedState = null;
      notifyPin();
      render();
    },
    getPinnedTile(): { x: number; y: number } | null {
      return pinnedState ? { x: pinnedState.x, y: pinnedState.y } : null;
    },
    onPinChange(cb: (tile: { x: number; y: number } | null) => void): () => void {
      pinListeners.add(cb);
      return () => pinListeners.delete(cb);
    },
  };
}
