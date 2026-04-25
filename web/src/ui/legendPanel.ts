/**
 * Legend panel — bottom-left corner of the canvas container.
 *
 * Shows a concise visual key for every overlay currently active.  The set of
 * visible entries updates reactively:
 *   - Base entries (item colour swatch) are always shown when a layout is loaded.
 *   - Debug-only entries (ghost paths, crossings, failures, cluster zones, lane
 *     columns, row boundaries, balancer/merger/tap blocks, ghost-tile fill,
 *     validation markers, junction-zone outcomes) are only added when the
 *     corresponding toggle is on.
 *
 * Collapse state is persisted in localStorage under the key "fk-legend-collapsed".
 */

const STORAGE_KEY = "fk-legend-collapsed";

// ---------------------------------------------------------------------------
// Swatch helpers
// ---------------------------------------------------------------------------

/** Solid colour square swatch (12 × 12 px). */
function colorSwatch(hex: string, alpha = 1): HTMLElement {
  const s = document.createElement("span");
  s.style.cssText = [
    "display:inline-block",
    "width:12px",
    "height:12px",
    "border-radius:2px",
    "flex-shrink:0",
    `background:${hex}`,
    `opacity:${alpha}`,
    "vertical-align:middle",
  ].join(";");
  return s;
}

/** Thin horizontal line swatch. Mimics the polyline on canvas. */
function lineSwatch(hex: string, widthPx = 3, alpha = 1): HTMLElement {
  const wrap = document.createElement("span");
  wrap.style.cssText = "display:inline-block;width:22px;height:12px;flex-shrink:0;position:relative;vertical-align:middle";
  const line = document.createElement("span");
  line.style.cssText = [
    "position:absolute",
    "top:50%",
    `transform:translateY(-${Math.round(widthPx / 2)}px)`,
    "left:0",
    "right:0",
    `height:${widthPx}px`,
    `background:${hex}`,
    `opacity:${alpha}`,
    "border-radius:1px",
  ].join(";");
  wrap.appendChild(line);
  return wrap;
}

/** Diamond swatch — matches the crossing-tile diamonds on canvas. */
function diamondSwatch(hex: string, alpha = 1): HTMLElement {
  const wrap = document.createElement("span");
  wrap.style.cssText = "display:inline-block;width:12px;height:12px;flex-shrink:0;position:relative;vertical-align:middle";
  const d = document.createElement("span");
  d.style.cssText = [
    "position:absolute",
    "top:50%",
    "left:50%",
    "width:9px",
    "height:9px",
    `background:${hex}`,
    `opacity:${alpha}`,
    "transform:translate(-50%,-50%) rotate(45deg)",
  ].join(";");
  wrap.appendChild(d);
  return wrap;
}

/** Cross (✕) swatch — matches failure markers on canvas. */
function crossSwatch(hex: string): HTMLElement {
  const wrap = document.createElement("span");
  wrap.style.cssText = "display:inline-block;width:12px;height:12px;flex-shrink:0;position:relative;vertical-align:middle";
  const bar1 = document.createElement("span");
  const bar2 = document.createElement("span");
  const barBase = [
    "position:absolute",
    "top:50%",
    "left:50%",
    "width:10px",
    "height:2px",
    `background:${hex}`,
    "transform-origin:center",
  ].join(";");
  bar1.style.cssText = barBase + ";transform:translate(-50%,-50%) rotate(45deg)";
  bar2.style.cssText = barBase + ";transform:translate(-50%,-50%) rotate(-45deg)";
  wrap.appendChild(bar1);
  wrap.appendChild(bar2);
  return wrap;
}

/** Rectangle outline swatch. */
function rectOutlineSwatch(hex: string, fillAlpha = 0.12, strokeAlpha = 0.85): HTMLElement {
  const s = document.createElement("span");
  const [r, g, b] = hexToRgb(hex);
  s.style.cssText = [
    "display:inline-block",
    "width:18px",
    "height:10px",
    "flex-shrink:0",
    "border-radius:1px",
    `background:rgba(${r},${g},${b},${fillAlpha})`,
    `border:1.5px solid rgba(${r},${g},${b},${strokeAlpha})`,
    "vertical-align:middle",
  ].join(";");
  return s;
}

// ---------------------------------------------------------------------------
// Ghost-path palette swatch: a small coloured line strip showing 3 of the
// cycling palette colours so the user knows paths are individually coloured.
// ---------------------------------------------------------------------------

function ghostPaletteSwatch(): HTMLElement {
  // First 3 colors from the ghostPalette constant in traceOverlay.ts
  const COLORS = ["#569cd6", "#d0a040", "#6ac080"];
  const wrap = document.createElement("span");
  wrap.style.cssText = "display:inline-block;width:22px;height:12px;flex-shrink:0;position:relative;vertical-align:middle;overflow:hidden;border-radius:1px";
  COLORS.forEach((c, i) => {
    const strip = document.createElement("span");
    strip.style.cssText = [
      "position:absolute",
      `left:${i * 7}px`,
      "top:4px",
      "width:5px",
      "height:4px",
      `background:${c}`,
      "opacity:0.8",
    ].join(";");
    wrap.appendChild(strip);
  });
  return wrap;
}

function hexToRgb(hex: string): [number, number, number] {
  const n = parseInt(hex.replace("#", ""), 16);
  return [(n >> 16) & 0xff, (n >> 8) & 0xff, n & 0xff];
}

// ---------------------------------------------------------------------------
// Entry builder
// ---------------------------------------------------------------------------

interface LegendEntry {
  swatch: HTMLElement;
  label: string;
}

function makeRow(entry: LegendEntry): HTMLDivElement {
  const row = document.createElement("div");
  row.style.cssText = "display:flex;align-items:center;gap:5px;padding:1px 0;white-space:nowrap";
  row.appendChild(entry.swatch);
  const lbl = document.createElement("span");
  lbl.textContent = entry.label;
  row.appendChild(lbl);
  return row;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface LegendPanelControls {
  /** Call whenever the set of active overlays changes. */
  update(state: LegendPanelState): void;
  /** Remove the panel from the DOM. */
  destroy(): void;
}

export interface LegendPanelState {
  /** A layout is loaded and the panel should be visible. */
  hasLayout: boolean;
  debugMode: boolean;
  /** Trace events present (implies ghost paths / cluster zones may exist). */
  hasTrace: boolean;
  stepThrough: boolean;
  ghostTiles: boolean;
  satZones: boolean;
  traceOverlay: boolean;
}

export function createLegendPanel(container: HTMLElement): LegendPanelControls {
  // --- Outer wrapper ---
  const panel = document.createElement("div");
  panel.style.cssText = [
    "position:absolute",
    "bottom:8px",
    "left:8px",
    "background:rgba(0,0,0,0.62)",
    "color:#bbb",
    "font:10px/1.5 monospace",
    "padding:0",
    "border-radius:4px",
    "z-index:10",
    "user-select:none",
    "min-width:160px",
    "max-width:240px",
    "pointer-events:auto",
  ].join(";");

  // --- Header / collapse toggle ---
  const header = document.createElement("div");
  header.style.cssText = [
    "display:flex",
    "align-items:center",
    "justify-content:space-between",
    "padding:4px 7px",
    "cursor:pointer",
    "border-bottom:1px solid rgba(255,255,255,0.08)",
    "border-radius:4px 4px 0 0",
  ].join(";");
  header.title = "Toggle legend";

  const headerLabel = document.createElement("span");
  headerLabel.textContent = "Legend";
  headerLabel.style.cssText = "color:#888;font-size:10px;letter-spacing:0.5px;text-transform:uppercase";

  const chevron = document.createElement("span");
  chevron.style.cssText = "color:#555;font-size:9px;transition:transform 0.15s";

  header.appendChild(headerLabel);
  header.appendChild(chevron);

  // --- Body ---
  const body = document.createElement("div");
  body.style.cssText = "padding:4px 8px 6px";

  panel.appendChild(header);
  panel.appendChild(body);
  container.appendChild(panel);

  // --- Collapse state ---
  let collapsed = localStorage.getItem(STORAGE_KEY) === "1";

  function applyCollapsed(): void {
    body.style.display = collapsed ? "none" : "block";
    chevron.textContent = collapsed ? "▶" : "▼";
    header.style.borderBottom = collapsed
      ? "none"
      : "1px solid rgba(255,255,255,0.08)";
    header.style.borderRadius = collapsed ? "4px" : "4px 4px 0 0";
  }
  applyCollapsed();

  header.addEventListener("click", () => {
    collapsed = !collapsed;
    localStorage.setItem(STORAGE_KEY, collapsed ? "1" : "0");
    applyCollapsed();
  });

  // --- Update function ---
  function update(state: LegendPanelState): void {
    if (!state.hasLayout) {
      panel.style.display = "none";
      return;
    }
    panel.style.display = "block";

    body.innerHTML = "";
    const entries: LegendEntry[] = [];

    // ---- Always-visible base entries ----
    // These are present regardless of debug toggles.

    // ---- Debug-mode overlays (only shown when Debug is on) ----
    if (state.debugMode && state.hasTrace) {
      // Ghost paths — step-through mode draws them via renderTraceOverlay;
      // they also appear in the ghost routing overlay when debug is on.
      // Show when debug is on and there's trace data, regardless of step-through.
      if (state.stepThrough || state.traceOverlay) {
        entries.push({ swatch: ghostPaletteSwatch(), label: "Ghost path (per-spec colour)" });
        entries.push({ swatch: diamondSwatch("#ffdd00", 0.85), label: "Crossing: two specs collide (SAT)" });
        entries.push({ swatch: crossSwatch("#ff3333"), label: "Route / ghost spec failed" });
        // Cluster zones in step-through mode (from renderTraceOverlay)
        entries.push({ swatch: rectOutlineSwatch("#44aaff", 0.08, 0.6), label: "Cluster zone: SAT solved" });
        entries.push({ swatch: rectOutlineSwatch("#ff4444", 0.15, 0.9), label: "Cluster zone: SAT failed" });
        // Lane columns and row boundaries (always in trace overlay when trace present)
        entries.push({ swatch: colorSwatch("#44ff88", 0.3), label: "Bus lane (solid item)" });
        entries.push({ swatch: colorSwatch("#44aaff", 0.3), label: "Bus lane (fluid item)" });
        entries.push({ swatch: lineSwatch("#6a8a5a", 1, 0.45), label: "Row boundary" });
        entries.push({ swatch: rectOutlineSwatch("#aa44ff", 0.2, 0.5), label: "Balancer block" });
        entries.push({ swatch: lineSwatch("#88ff44", 2, 0.5), label: "Tap-off path" });
        entries.push({ swatch: rectOutlineSwatch("#ffcc44", 0.2, 0.5), label: "Output merger block" });
      }

      // Ghost-tile ambient fill
      if (state.ghostTiles) {
        entries.push({ swatch: colorSwatch("#40d0e0", 0.4), label: "Ghost router footprint" });
      }


      // SAT / Junction zones
      if (state.satZones) {
        entries.push({ swatch: rectOutlineSwatch("#3aa04a", 0.12, 0.85), label: "Junction zone: solved" });
        entries.push({ swatch: rectOutlineSwatch("#d4a03a", 0.12, 0.85), label: "Junction zone: capped" });
        entries.push({ swatch: rectOutlineSwatch("#c04040", 0.12, 0.85), label: "Junction zone: open (unsolved)" });
      }
    }

    if (entries.length === 0) {
      // Nothing to show — hide entirely so the panel doesn't clutter for
      // users who just want to see the layout without debug overlays.
      panel.style.display = "none";
      return;
    }

    for (const entry of entries) {
      body.appendChild(makeRow(entry));
    }
  }

  function destroy(): void {
    panel.remove();
  }

  return { update, destroy };
}
