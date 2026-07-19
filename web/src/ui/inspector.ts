import type { PlacedEntity } from "../engine";
import { niceName, getRecipeFlows, type HighlightController } from "../renderer/entities";
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
  North: "↑", East: "→", South: "↓", West: "←",
};
const COMPACT_DIR: Record<string, string> = {
  N: "↑", E: "→", S: "↓", W: "←",
};

// ---------------------------------------------------------------------------
// DOM helper: create an <img> icon element. Mutate .src + .width/.height to
// reuse. Using DOM mutation instead of innerHTML avoids repeated HTML parsing.
// ---------------------------------------------------------------------------
function makeIconEl(size = 16): HTMLImageElement {
  const img = document.createElement("img");
  img.width = size;
  img.height = size;
  img.style.cssText = "vertical-align:middle;margin-right:3px;image-rendering:pixelated";
  img.addEventListener("error", () => { img.style.display = "none"; });
  return img;
}

// Set icon src and restore visibility (in case it was hidden by an error event).
function setIconSrc(img: HTMLImageElement, slug: string): void {
  img.style.display = "";
  img.src = `${import.meta.env.BASE_URL}icons/${slug}.png`;
}

// ---------------------------------------------------------------------------
// Row-pool helper: keeps a pool of <div> rows. Callers request N rows;
// extras are hidden. New rows are appended only when the pool is exhausted.
// ---------------------------------------------------------------------------
interface RowPool {
  /** Get or create a row at the given index; all rows beyond `count` will be
   *  hidden when you call `trim(count)`. */
  get(index: number): HTMLElement;
  /** Hide all rows from `from` onward. */
  trim(from: number): void;
  readonly length: number;
}

function makeRowPool(parent: HTMLElement, makeRow: () => HTMLElement): RowPool {
  const rows: HTMLElement[] = [];
  return {
    get(index: number): HTMLElement {
      while (rows.length <= index) {
        const r = makeRow();
        parent.appendChild(r);
        rows.push(r);
      }
      rows[index].style.display = "";
      return rows[index];
    },
    trim(from: number): void {
      for (let i = from; i < rows.length; i++) {
        rows[i].style.display = "none";
      }
    },
    get length() { return rows.length; },
  };
}

export function createInspector(container: HTMLElement): InspectorControls {
  // -------------------------------------------------------------------------
  // Tooltip: floating tooltip that follows the cursor
  // -------------------------------------------------------------------------
  const tooltip = document.createElement("div");
  tooltip.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5";
  document.body.appendChild(tooltip);

  // Override section — shown when setTooltipOverride() is called.
  // We keep innerHTML here deliberately: the override text comes from the
  // debug/trace overlay and may contain <span style="..."> markup that callers
  // build dynamically. The override path is set very rarely (only on debug
  // overlay clicks), so its parsing cost is negligible. Normal hover uses
  // persistent DOM nodes below.
  const tooltipOverrideEl = document.createElement("div");
  tooltip.appendChild(tooltipOverrideEl);

  // Coord line — shown in both override and normal paths.
  const tooltipCoordEl = document.createElement("span");
  tooltipCoordEl.style.color = "#888";
  tooltipCoordEl.style.display = "none";
  tooltip.appendChild(tooltipCoordEl);

  // Normal entity section (hidden when override is active).
  const tooltipEntitySection = document.createElement("div");
  tooltip.appendChild(tooltipEntitySection);

  // Entity header row: icon + bold name
  const ttEntityHeader = document.createElement("div");
  const ttEntityIcon = makeIconEl(16);
  const ttEntityName = document.createElement("b");
  ttEntityHeader.append(ttEntityIcon, ttEntityName);
  ttEntityHeader.style.display = "none";
  tooltipEntitySection.appendChild(ttEntityHeader);

  // Direction row
  const ttDirectionRow = document.createElement("div");
  ttDirectionRow.style.display = "none";
  tooltipEntitySection.appendChild(ttDirectionRow);

  // Carries row: icon + name
  const ttCarriesRow = document.createElement("div");
  const ttCarriesIcon = makeIconEl(16);
  const ttCarriesName = document.createElement("span");
  ttCarriesRow.append(ttCarriesIcon, ttCarriesName);
  ttCarriesRow.style.display = "none";
  tooltipEntitySection.appendChild(ttCarriesRow);

  // Rate row
  const ttRateRow = document.createElement("div");
  ttRateRow.style.color = "#b5cea8";
  ttRateRow.style.display = "none";
  tooltipEntitySection.appendChild(ttRateRow);

  // IO type row
  const ttIoRow = document.createElement("div");
  ttIoRow.style.display = "none";
  tooltipEntitySection.appendChild(ttIoRow);

  // Recipe header row: icon + name
  const ttRecipeRow = document.createElement("div");
  const ttRecipeIcon = makeIconEl(16);
  const ttRecipeName = document.createElement("span");
  ttRecipeRow.append(ttRecipeIcon, ttRecipeName);
  ttRecipeRow.style.display = "none";
  tooltipEntitySection.appendChild(ttRecipeRow);

  // Flow rows pool (recipe inputs + outputs)
  function makeFlowRow(): HTMLElement {
    const row = document.createElement("div");
    row.style.color = "#aaa";
    const arrow = document.createElement("span");
    const icon = makeIconEl(14);
    const label = document.createElement("span");
    row.append(arrow, icon, label);
    return row;
  }
  const ttFlowPool = makeRowPool(tooltipEntitySection, makeFlowRow);

  // Segment row
  const ttSegmentRow = document.createElement("div");
  ttSegmentRow.style.color = "#9cdcfe";
  ttSegmentRow.style.display = "none";
  tooltipEntitySection.appendChild(ttSegmentRow);

  // Tile-context rows (ghost compact, axis compact, junction compact)
  const ttGhostRow = document.createElement("div");
  ttGhostRow.style.display = "none";
  tooltip.appendChild(ttGhostRow);

  const ttAxisRow = document.createElement("div");
  ttAxisRow.style.display = "none";
  tooltip.appendChild(ttAxisRow);

  const ttJunctionRow = document.createElement("div");
  ttJunctionRow.style.display = "none";
  tooltip.appendChild(ttJunctionRow);

  const ttCappedRow = document.createElement("div");
  ttCappedRow.style.display = "none";
  tooltip.appendChild(ttCappedRow);

  // -------------------------------------------------------------------------
  // Pinned detail panel — fixed corner, pointer events enabled
  // -------------------------------------------------------------------------
  const pinned = document.createElement("div");
  pinned.style.cssText = "position:absolute;top:8px;right:8px;background:#141414;color:#e0e0e0;border:1px solid #888;padding:8px 10px;font:12px monospace;border-radius:4px;display:none;z-index:20;min-width:220px;max-width:340px;line-height:1.55;box-shadow:0 4px 14px rgba(0,0,0,0.5)";
  container.appendChild(pinned);

  // Pinned header: "pinned" label + coord
  const pinnedHeader = document.createElement("div");
  pinnedHeader.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
  const pinnedHeaderLabel = document.createElement("span");
  pinnedHeaderLabel.style.cssText = "color:#8af;font-weight:bold";
  pinnedHeaderLabel.textContent = "pinned";
  const pinnedHeaderCoord = document.createElement("span");
  pinnedHeaderCoord.style.color = "#888";
  pinnedHeader.append(pinnedHeaderLabel, pinnedHeaderCoord);
  pinned.appendChild(pinnedHeader);

  // Pinned entity section
  const pinnedEntitySection = document.createElement("div");
  pinned.appendChild(pinnedEntitySection);

  // Pinned entity header row
  const pnEntityHeader = document.createElement("div");
  const pnEntityIcon = makeIconEl(16);
  const pnEntityName = document.createElement("b");
  pnEntityHeader.append(pnEntityIcon, pnEntityName);
  pnEntityHeader.style.display = "none";
  pinnedEntitySection.appendChild(pnEntityHeader);

  // Pinned no-entity fallback
  const pnNoEntity = document.createElement("span");
  pnNoEntity.style.color = "#888";
  pnNoEntity.textContent = "no entity at tile";
  pnNoEntity.style.display = "none";
  pinnedEntitySection.appendChild(pnNoEntity);

  // Pinned direction row
  const pnDirectionRow = document.createElement("div");
  pnDirectionRow.style.display = "none";
  pinnedEntitySection.appendChild(pnDirectionRow);

  // Pinned carries row
  const pnCarriesRow = document.createElement("div");
  const pnCarriesIcon = makeIconEl(16);
  const pnCarriesName = document.createElement("span");
  pnCarriesRow.append(pnCarriesIcon, pnCarriesName);
  pnCarriesRow.style.display = "none";
  pinnedEntitySection.appendChild(pnCarriesRow);

  // Pinned rate row
  const pnRateRow = document.createElement("div");
  pnRateRow.style.color = "#b5cea8";
  pnRateRow.style.display = "none";
  pinnedEntitySection.appendChild(pnRateRow);

  // Pinned IO row
  const pnIoRow = document.createElement("div");
  pnIoRow.style.display = "none";
  pinnedEntitySection.appendChild(pnIoRow);

  // Pinned recipe row
  const pnRecipeRow = document.createElement("div");
  const pnRecipeIcon = makeIconEl(16);
  const pnRecipeName = document.createElement("span");
  pnRecipeRow.append(pnRecipeIcon, pnRecipeName);
  pnRecipeRow.style.display = "none";
  pinnedEntitySection.appendChild(pnRecipeRow);

  // Pinned flow rows pool
  const pnFlowPool = makeRowPool(pinnedEntitySection, makeFlowRow);

  // Pinned segment row
  const pnSegmentRow = document.createElement("div");
  pnSegmentRow.style.color = "#9cdcfe";
  pnSegmentRow.style.display = "none";
  pinnedEntitySection.appendChild(pnSegmentRow);

  // Pinned junction block
  const pnJunctionBlock = document.createElement("div");
  pnJunctionBlock.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333";
  pnJunctionBlock.style.display = "none";
  const pnJunctionLabel = document.createElement("span");
  const pnJunctionOutcome = document.createElement("span");
  pnJunctionOutcome.style.color = "#888";
  pnJunctionBlock.append(pnJunctionLabel, pnJunctionOutcome);
  pinned.appendChild(pnJunctionBlock);

  // Capped inserter sides at the pinned machine (RFP
  // validation-explainability Phase 3a) — the expanded explanation
  // behind rate-shaped warnings: what the ladder placed, the shortfall,
  // and the binding constraint spelled out.
  const pnCappedBlock = document.createElement("div");
  pnCappedBlock.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333";
  pnCappedBlock.style.display = "none";
  pinned.appendChild(pnCappedBlock);

  // Pinned axis block
  const pnAxisBlock = document.createElement("div");
  pnAxisBlock.style.marginTop = "4px";
  pnAxisBlock.style.display = "none";
  pinned.appendChild(pnAxisBlock);

  // Pinned ghost block
  const pnGhostBlock = document.createElement("div");
  pnGhostBlock.style.display = "none";
  pinned.appendChild(pnGhostBlock);

  // Pinned ghost header line (varies count/color)
  const pnGhostHeader = document.createElement("div");
  pnGhostHeader.style.marginTop = "4px";
  pnGhostBlock.appendChild(pnGhostHeader);

  // Pool for individual ghost rows inside the pinned panel
  function makePinGhostRow(): HTMLElement {
    const row = document.createElement("div");
    row.style.marginLeft = "4px";
    return row;
  }
  const pnGhostRowPool = makeRowPool(pnGhostBlock, makePinGhostRow);

  // Pinned hint footer
  const pnHint = document.createElement("div");
  pnHint.style.cssText = "color:#555;margin-top:6px;font-size:10px";
  pnHint.textContent = "click elsewhere or press Esc to unpin";
  pinned.appendChild(pnHint);

  // -------------------------------------------------------------------------
  // Cursor-follow
  // -------------------------------------------------------------------------
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

  // -------------------------------------------------------------------------
  // Shared entity-section mutator — populates entity rows into a given set of
  // persistent DOM elements and returns the number of flow rows used.
  // -------------------------------------------------------------------------
  function populateEntitySection(
    entity: PlacedEntity,
    els: {
      header: HTMLElement; headerIcon: HTMLImageElement; headerName: HTMLElement;
      dirRow: HTMLElement;
      carriesRow: HTMLElement; carriesIcon: HTMLImageElement; carriesName: HTMLElement;
      rateRow: HTMLElement;
      ioRow: HTMLElement;
      recipeRow: HTMLElement; recipeIcon: HTMLImageElement; recipeName: HTMLElement;
      flowPool: RowPool;
      segmentRow: HTMLElement;
    }
  ): number {
    // Header
    setIconSrc(els.headerIcon, entity.name);
    els.headerName.textContent = niceName(entity.name);
    els.header.style.display = "";

    // Direction
    if (entity.direction && entity.name !== "pipe") {
      els.dirRow.textContent = `${DIR_ARROW[entity.direction] ?? ""} ${entity.direction}`;
      els.dirRow.style.display = "";
    } else {
      els.dirRow.style.display = "none";
    }

    // Carries
    if (entity.carries) {
      setIconSrc(els.carriesIcon, entity.carries);
      els.carriesName.textContent = " " + niceName(entity.carries);
      els.carriesRow.style.display = "";
    } else {
      els.carriesRow.style.display = "none";
    }

    // Rate
    if (entity.rate != null) {
      els.rateRow.textContent = `${entity.rate.toFixed(1)}/s`;
      els.rateRow.style.display = "";
    } else {
      els.rateRow.style.display = "none";
    }

    // IO type
    if (entity.io_type) {
      els.ioRow.textContent = `io: ${entity.io_type}`;
      els.ioRow.style.display = "";
    } else {
      els.ioRow.style.display = "none";
    }

    // Recipe + flows
    let flowCount = 0;
    if (entity.recipe) {
      setIconSrc(els.recipeIcon, entity.recipe);
      els.recipeName.textContent = " " + niceName(entity.recipe);
      els.recipeRow.style.display = "";

      const flows = getRecipeFlows(entity.recipe);
      if (flows) {
        const allFlows: { arrow: string; item: string; rate: number }[] = [
          ...flows.inputs.map(f => ({ arrow: "▶", item: f.item, rate: f.rate })),
          ...flows.outputs.map(f => ({ arrow: "◀", item: f.item, rate: f.rate })),
        ];
        for (const f of allFlows) {
          const row = els.flowPool.get(flowCount++);
          const [arrowEl, iconEl, labelEl] = row.children as unknown as [HTMLElement, HTMLImageElement, HTMLElement];
          arrowEl.textContent = `${f.arrow} `;
          setIconSrc(iconEl, f.item);
          labelEl.textContent = `${niceName(f.item)} ${f.rate.toFixed(1)}/s`;
        }
      }
    } else {
      els.recipeRow.style.display = "none";
    }
    els.flowPool.trim(flowCount);

    // Segment
    if (entity.segment_id) {
      els.segmentRow.textContent = entity.segment_id;
      els.segmentRow.style.display = "";
    } else {
      els.segmentRow.style.display = "none";
    }

    return flowCount;
  }

  // -------------------------------------------------------------------------
  // Tooltip tile-context rows (ghost compact, axis compact, junction compact)
  // These return whether the row has content.
  // -------------------------------------------------------------------------

  function renderGhostCompact(info: TileInfo): boolean {
    if (info.ghosts.length === 0) {
      ttGhostRow.style.display = "none";
      return false;
    }
    ttGhostRow.style.display = "";
    if (info.ghosts.length === 1) {
      const g = info.ghosts[0];
      const arrow = g.direction ? COMPACT_DIR[g.direction] : "";
      // Clear and rebuild this compact row (it has mixed children; simpler to
      // reconstruct since it's a one-liner that changes structure based on count)
      ttGhostRow.textContent = "";
      const label = document.createElement("span");
      label.style.color = "#8af";
      label.textContent = "ghost ";
      const icon = makeIconEl(12);
      setIconSrc(icon, g.item);
      const text = document.createTextNode(`${g.item} ${arrow}`);
      ttGhostRow.append(label, icon, text);
    } else {
      ttGhostRow.textContent = "";
      const info_ = document.createElement("span");
      info_.style.color = "#8af";
      info_.textContent = `${info.ghosts.length} ghosts crossing`;
      ttGhostRow.appendChild(info_);
    }
    return true;
  }

  function renderAxisCompact(info: TileInfo): boolean {
    if (!info.axis) { ttAxisRow.style.display = "none"; return false; }
    const { vert, horiz } = info.axis;
    if (vert === 0 && horiz === 0) { ttAxisRow.style.display = "none"; return false; }
    const conflict = vert >= 2 || horiz >= 2;
    const perp = vert >= 1 && horiz >= 1;
    const color = conflict ? "#ff6060" : perp ? "#60b0ff" : "#888";
    ttAxisRow.style.display = "";
    ttAxisRow.style.color = color;
    ttAxisRow.textContent = `axis V${vert} H${horiz}`;
    return true;
  }

  function renderJunctionCompact(info: TileInfo): boolean {
    if (!info.junction) { ttJunctionRow.style.display = "none"; return false; }
    const j = info.junction;
    const color = j.outcome === "Solved" ? "#80d080" : j.outcome === "Capped" ? "#e0b060" : "#c06060";
    ttJunctionRow.style.display = "";
    ttJunctionRow.style.color = color;
    ttJunctionRow.textContent = `junction seed (${j.seedX},${j.seedY}) · ${j.outcome}`;
    return true;
  }

  /** Capped inserter sides at this machine origin (RFP
   *  validation-explainability D2) — the stamp-time cause behind
   *  rate-shaped warnings anchored here. One line per capped side:
   *  what was placed, what it moves vs what's needed, and the binding
   *  constraint the ladder derived. */
  function renderCappedCompact(info: TileInfo): boolean {
    if (info.cappedSides.length === 0) { ttCappedRow.style.display = "none"; return false; }
    const lines = info.cappedSides.map((c) => {
      const moved = c.required - c.shortfall;
      const dir = c.sideIsOutput ? "out" : "in";
      return `⚠ ${dir}: ${c.placedCount}×${c.placedEntity} moves ${moved.toFixed(2)}/s of ${c.required.toFixed(2)}/s · ${c.limit}`;
    });
    ttCappedRow.style.display = "";
    ttCappedRow.style.color = "#ffa060";
    ttCappedRow.textContent = lines.join(" | ");
    return true;
  }

  // -------------------------------------------------------------------------
  // Pinned ghost expanded block
  // -------------------------------------------------------------------------
  function renderPinnedGhostExpanded(info: TileInfo): void {
    if (info.ghosts.length === 0) {
      pnGhostBlock.style.display = "none";
      return;
    }
    pnGhostBlock.style.display = "";
    if (info.ghosts.length >= 2) {
      pnGhostHeader.style.color = "#ffa060";
      pnGhostHeader.textContent = `⚠ ${info.ghosts.length} ghost specs at this tile`;
    } else {
      pnGhostHeader.style.color = "#8af";
      pnGhostHeader.textContent = "ghost";
    }

    let gi = 0;
    for (const g of info.ghosts) {
      const arrow = g.direction ? COMPACT_DIR[g.direction] : "·";
      const row = pnGhostRowPool.get(gi++);
      // Rebuild ghost row children: arrow text + icon + item name + optional endpoint tag
      row.textContent = "";
      const arrowText = document.createTextNode(`${arrow} `);
      const icon = makeIconEl(14);
      setIconSrc(icon, g.item);
      const nameText = document.createTextNode(g.item);
      row.append(arrowText, icon, nameText);
      if (g.isStart) {
        const tag = document.createElement("span");
        tag.style.color = "#80d080";
        tag.textContent = " start";
        row.appendChild(tag);
      } else if (g.isEnd) {
        const tag = document.createElement("span");
        tag.style.color = "#d08080";
        tag.textContent = " end";
        row.appendChild(tag);
      }
    }
    pnGhostRowPool.trim(gi);
  }

  // -------------------------------------------------------------------------
  // Main render functions
  // -------------------------------------------------------------------------

  function renderHover(): void {
    if (tooltipOverride !== null) {
      // Override path: keep innerHTML for this one element only (the override
      // text comes from debug/trace overlays and may contain arbitrary HTML
      // like <span style="...">. This is set rarely so parsing cost is fine.)
      tooltipOverrideEl.innerHTML = tooltipOverride;
      tooltipOverrideEl.style.display = "";
      tooltipEntitySection.style.display = "none";
      ttGhostRow.style.display = "none";
      ttAxisRow.style.display = "none";
      ttJunctionRow.style.display = "none";

      if (cursorTile) {
        tooltipCoordEl.textContent = `(${cursorTile.x}, ${cursorTile.y})`;
        tooltipCoordEl.style.display = "";
        // Need a separator between override and coord — reuse a <br>
        // We insert it as a sibling: override | <br> | coord. Simplest:
        // use display:block on coord so it wraps naturally (monospace, same font).
        tooltipCoordEl.style.display = "block";
      } else {
        tooltipCoordEl.style.display = "none";
      }

      tooltip.style.display = "block";
      return;
    }

    tooltipOverrideEl.style.display = "none";
    tooltipOverrideEl.innerHTML = "";
    tooltipEntitySection.style.display = "";

    let hasContent = false;

    if (hoveredEntity) {
      hasContent = true;
      populateEntitySection(hoveredEntity, {
        header: ttEntityHeader, headerIcon: ttEntityIcon, headerName: ttEntityName,
        dirRow: ttDirectionRow,
        carriesRow: ttCarriesRow, carriesIcon: ttCarriesIcon, carriesName: ttCarriesName,
        rateRow: ttRateRow,
        ioRow: ttIoRow,
        recipeRow: ttRecipeRow, recipeIcon: ttRecipeIcon, recipeName: ttRecipeName,
        flowPool: ttFlowPool,
        segmentRow: ttSegmentRow,
      });
    } else {
      ttEntityHeader.style.display = "none";
      ttDirectionRow.style.display = "none";
      ttCarriesRow.style.display = "none";
      ttRateRow.style.display = "none";
      ttIoRow.style.display = "none";
      ttRecipeRow.style.display = "none";
      ttFlowPool.trim(0);
      ttSegmentRow.style.display = "none";
    }

    if (cursorTile) {
      const info = tileContext?.lookup(cursorTile.x, cursorTile.y);
      if (info) {
        if (renderGhostCompact(info)) hasContent = true;
        if (renderAxisCompact(info)) hasContent = true;
        if (renderJunctionCompact(info)) hasContent = true;
        if (renderCappedCompact(info)) hasContent = true;
      } else {
        ttGhostRow.style.display = "none";
        ttAxisRow.style.display = "none";
        ttJunctionRow.style.display = "none";
        ttCappedRow.style.display = "none";
      }
      tooltipCoordEl.textContent = `(${cursorTile.x}, ${cursorTile.y})`;
      tooltipCoordEl.style.display = "block";
      hasContent = true;
    } else {
      tooltipCoordEl.style.display = "none";
      ttGhostRow.style.display = "none";
      ttAxisRow.style.display = "none";
      ttJunctionRow.style.display = "none";
    }

    if (!hasContent) {
      tooltip.style.display = "none";
      if (highlightCtrl) highlightCtrl.clearHighlight();
      return;
    }

    tooltip.style.display = "block";

    if (hoveredEntity && highlightCtrl) {
      highlightCtrl.highlightBeltNetwork(hoveredEntity);
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

    pinnedHeaderCoord.textContent = `(${x}, ${y})`;

    if (entity) {
      pnNoEntity.style.display = "none";
      populateEntitySection(entity, {
        header: pnEntityHeader, headerIcon: pnEntityIcon, headerName: pnEntityName,
        dirRow: pnDirectionRow,
        carriesRow: pnCarriesRow, carriesIcon: pnCarriesIcon, carriesName: pnCarriesName,
        rateRow: pnRateRow,
        ioRow: pnIoRow,
        recipeRow: pnRecipeRow, recipeIcon: pnRecipeIcon, recipeName: pnRecipeName,
        flowPool: pnFlowPool,
        segmentRow: pnSegmentRow,
      });
    } else {
      pnEntityHeader.style.display = "none";
      pnDirectionRow.style.display = "none";
      pnCarriesRow.style.display = "none";
      pnRateRow.style.display = "none";
      pnIoRow.style.display = "none";
      pnRecipeRow.style.display = "none";
      pnFlowPool.trim(0);
      pnSegmentRow.style.display = "none";
      pnNoEntity.style.display = "";
    }

    if (info) {
      // Junction block
      if (info.junction) {
        const color = info.junction.outcome === "Solved" ? "#80d080" : info.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
        pnJunctionLabel.style.color = color;
        pnJunctionLabel.textContent = `junction seed (${info.junction.seedX},${info.junction.seedY})`;
        pnJunctionOutcome.textContent = ` · ${info.junction.outcome}`;
        pnJunctionBlock.style.display = "";
      } else {
        pnJunctionBlock.style.display = "none";
      }

      // Axis block
      if (info.axis) {
        const { vert, horiz } = info.axis;
        if (vert > 0 || horiz > 0) {
          const conflict = vert >= 2 || horiz >= 2;
          const perp = vert >= 1 && horiz >= 1;
          const label = conflict ? " same-axis conflict" : perp ? " perpendicular crossing" : "";
          const color = conflict ? "#ff6060" : perp ? "#60b0ff" : "#bbb";
          pnAxisBlock.style.color = color;
          pnAxisBlock.textContent = `axis: V=${vert} H=${horiz}${label}`;
          pnAxisBlock.style.display = "";
        } else {
          pnAxisBlock.style.display = "none";
        }
      } else {
        pnAxisBlock.style.display = "none";
      }

      // Ghost expanded block
      renderPinnedGhostExpanded(info);

      // Capped-sides expanded block (Phase 3a): one paragraph per
      // capped side, with the binding constraint spelled out in plain
      // language so the warning is self-explaining.
      if (info.cappedSides.length > 0) {
        const LIMIT_TEXT: Record<string, string> = {
          "tier-cap": "a faster inserter tier at the same slot count would cover this — max inserter tier is the binding constraint",
          "column-contest": "this side lost the shared inserter column to the other belt; that one column would have covered it",
          geometry: "the row shape offers no further usable slot (belt span / fixed tiles) — a template geometry limit",
        };
        pnCappedBlock.replaceChildren();
        const header = document.createElement("div");
        header.style.color = "#ffa060";
        header.textContent = `⚠ ${info.cappedSides.length} under-provisioned inserter side${info.cappedSides.length > 1 ? "s" : ""}`;
        pnCappedBlock.appendChild(header);
        for (const c of info.cappedSides) {
          const moved = (c.required - c.shortfall).toFixed(2);
          const row = document.createElement("div");
          row.style.cssText = "margin-top:2px;color:#ccc";
          row.textContent =
            `${c.sideIsOutput ? "output" : "input"}: ${c.placedCount}×${c.placedEntity} ` +
            `moves ${moved}/s of ${c.required.toFixed(2)}/s needed (short ${c.shortfall.toFixed(2)}/s) — ` +
            `${c.limit}: ${LIMIT_TEXT[c.limit] ?? c.limit}`;
          pnCappedBlock.appendChild(row);
        }
        pnCappedBlock.style.display = "";
      } else {
        pnCappedBlock.style.display = "none";
      }
    } else {
      pnJunctionBlock.style.display = "none";
      pnAxisBlock.style.display = "none";
      pnGhostBlock.style.display = "none";
      pnCappedBlock.style.display = "none";
    }

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
