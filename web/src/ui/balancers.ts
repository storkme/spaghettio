/**
 * Balancer showcase QA tool — issue #274.
 *
 * Standalone route at `#/balancers` rendering every (n_inputs, n_outputs)
 * library entry in a 10×10 grid. Each cell shows the template captioned
 * with source provenance (Raynquist / compose / Factorio-SAT) plus the
 * construction strategy (e.g. `Lib(7, 1) → Lib(1, 2)`) so it's immediately
 * clear where each balancer came from.
 *
 * The "generated" side-panel was removed — `balancer_generate::generate`
 * is a stub that only handles trivial cases, and the compose pipeline
 * (the real generator) doesn't run in WASM. Browser users who want to see
 * a constructed layout look at the library entry, which IS the output of
 * compose for the shapes we've baked through it.
 *
 * URL state: `#/balancers?focus=n,m` highlights and scrolls to that
 * shape. Used for permalinks in code review / decision log entries.
 */

import { Application, Container } from "pixi.js";
import {
  TILE_PX,
  createDrawContext,
  addEntityToDrawContext,
  drawEntityGraphic,
  drawUgTunnelStripe,
  UG_BELT_ENTITIES,
} from "../renderer/entities";
import type { Engine, BalancerShowcaseCell, BalancerShowcaseTemplate } from "../engine";

const MAX_INPUTS = 10;
const MAX_OUTPUTS = 10;

// Display target for each rendered template panel. Real templates fit
// well under this — library (1, 8) is 8×5 = 256×160 px at TILE_PX=32 —
// and CSS scales down inside the panel if a compose-bake template
// over-runs (e.g. 9×20 = 288×640 px). Keeping the absolute pixel
// dimensions in the canvas means small templates aren't blurred up
// while big ones gracefully shrink to fit.
const PANEL_WIDTH_TILES = 12;
const PANEL_HEIGHT_TILES = 12;

export function renderBalancerShowcase(
  appRoot: HTMLElement,
  engine: Engine,
): void {
  // Take over the existing #app skeleton — hide the sidebar+canvas
  // panels, install our own scrollable host.
  appRoot.innerHTML = "";
  appRoot.style.display = "block";
  appRoot.style.height = "100vh";
  appRoot.style.width = "100vw";
  appRoot.style.overflow = "auto";

  const host = document.createElement("div");
  host.id = "balancer-showcase";
  host.style.padding = "24px 32px 64px";
  host.style.maxWidth = "1600px";
  host.style.margin = "0 auto";
  host.style.fontFamily =
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif";
  appRoot.appendChild(host);

  injectStyles();

  const header = document.createElement("header");
  header.className = "bs-header";
  header.innerHTML = `
    <div>
      <h1>Balancer template showcase</h1>
      <p>Library (Factorio-SAT) vs compose-bake generator, side-by-side. Issue #274.</p>
    </div>
    <a href="#/" class="bs-back">← back</a>
  `;
  host.appendChild(header);

  const grid = document.createElement("div");
  grid.className = "bs-grid";
  grid.style.setProperty("--bs-cols", String(MAX_OUTPUTS));
  host.appendChild(grid);

  const focus = parseFocus();
  const cellMap = new Map<string, HTMLElement>();

  // Pre-create one cell per (n, m) so the grid pops into the correct
  // shape immediately. Slots are filled as each template arrives.
  for (let n = 1; n <= MAX_INPUTS; n++) {
    for (let m = 1; m <= MAX_OUTPUTS; m++) {
      const cell = createCellSkeleton(n, m, focus);
      cellMap.set(cellKey(n, m), cell);
      grid.appendChild(cell);
    }
  }

  // If the URL focuses a specific shape, scroll it into view once the
  // grid is laid out.
  if (focus) {
    const target = cellMap.get(cellKey(focus.n, focus.m));
    if (target) {
      // Defer to the next frame so layout has settled.
      requestAnimationFrame(() => {
        target.scrollIntoView({ behavior: "auto", block: "center" });
      });
    }
  }

  void hydrate(engine, cellMap);
}

interface FocusShape {
  n: number;
  m: number;
}

function parseFocus(): FocusShape | null {
  // Hash format: `#/balancers?focus=n,m`
  const hash = window.location.hash;
  const idx = hash.indexOf("?");
  if (idx < 0) return null;
  const params = new URLSearchParams(hash.slice(idx + 1));
  const focusVal = params.get("focus");
  if (!focusVal) return null;
  const [nStr, mStr] = focusVal.split(",");
  const n = Number.parseInt(nStr, 10);
  const m = Number.parseInt(mStr, 10);
  if (
    !Number.isFinite(n) || !Number.isFinite(m) ||
    n < 1 || n > MAX_INPUTS || m < 1 || m > MAX_OUTPUTS
  ) {
    return null;
  }
  return { n, m };
}

function cellKey(n: number, m: number): string {
  return `${n},${m}`;
}

function createCellSkeleton(
  n: number,
  m: number,
  focus: FocusShape | null,
): HTMLElement {
  const cell = document.createElement("div");
  cell.className = "bs-cell";
  cell.dataset.n = String(n);
  cell.dataset.m = String(m);
  if (focus && focus.n === n && focus.m === m) {
    cell.classList.add("bs-focus");
  }

  cell.innerHTML = `
    <header class="bs-cell-header">
      <span class="bs-shape">(${n}, ${m})</span>
      <span class="bs-summary"></span>
    </header>
    <div class="bs-panels">
      <div class="bs-panel" data-side="library">
        <div class="bs-panel-caption">loading…</div>
        <div class="bs-panel-canvas"></div>
      </div>
    </div>
  `;

  return cell;
}

async function hydrate(
  engine: Engine,
  cellMap: Map<string, HTMLElement>,
): Promise<void> {
  let cells: BalancerShowcaseCell[];
  try {
    cells = await engine.balancerShowcase(MAX_INPUTS, MAX_OUTPUTS);
  } catch (err) {
    // Surface the failure in every cell so the bug is impossible to miss.
    for (const cell of cellMap.values()) {
      const summary = cell.querySelector(".bs-summary");
      if (summary) summary.textContent = "showcase fetch failed";
    }
    console.error("[balancers] showcase fetch failed", err);
    return;
  }

  // One off-screen Pixi app shared across all extractions. Spinning up
  // an Application per cell would burn through the WebGL context limit
  // (~16 contexts in browsers) immediately.
  const offApp = new Application();
  await offApp.init({
    width: PANEL_WIDTH_TILES * TILE_PX,
    height: PANEL_HEIGHT_TILES * TILE_PX,
    background: 0x1e1e1e,
    antialias: true,
    autoStart: false,
    sharedTicker: false,
    preference: "webgl",
  });

  for (const cellData of cells) {
    const dom = cellMap.get(cellKey(cellData.n_inputs, cellData.n_outputs));
    if (!dom) continue;

    fillPanel(offApp, dom, cellData.library);

    const summary = dom.querySelector(".bs-summary") as HTMLElement | null;
    if (summary) {
      summary.innerHTML = sourceSummary(cellData);
    }
  }

  // The off-screen app keeps a GL context alive until destroyed. We
  // don't need it past hydration — re-renders happen on reload — so
  // free the resources.
  offApp.destroy(true, { children: true, texture: true });
}

function fillPanel(
  offApp: Application,
  cellDom: HTMLElement,
  template: BalancerShowcaseTemplate | null,
): void {
  const panel = cellDom.querySelector(
    `.bs-panel[data-side="library"]`,
  ) as HTMLElement | null;
  if (!panel) return;
  const caption = panel.querySelector(".bs-panel-caption") as HTMLElement | null;
  const canvasHost = panel.querySelector(".bs-panel-canvas") as HTMLElement | null;
  if (!caption || !canvasHost) return;

  if (!template) {
    panel.classList.add("bs-empty");
    caption.textContent = "no library entry";
    canvasHost.innerHTML = "";
    return;
  }

  const entityCount = template.entities.length;
  const strategyLine = template.strategy
    ? `<div class="bs-strategy">${escapeHtml(template.strategy)}</div>`
    : "";
  const referenceLine = template.reference
    ? `<div class="bs-reference"><a href="${escapeHtml(template.reference)}" target="_blank" rel="noopener">ref</a></div>`
    : "";
  caption.innerHTML = `
    <div class="bs-source">${escapeHtml(template.source)}${referenceLine}</div>
    ${strategyLine}
    <div class="bs-stats">
      ${template.n_inputs} → ${template.n_outputs} ·
      ${template.width}×${template.height} ·
      <strong>${entityCount} ${entityCount === 1 ? "entity" : "entities"}</strong>
    </div>
  `;

  const rendered = renderTemplateCanvas(offApp, template);
  canvasHost.innerHTML = "";
  canvasHost.appendChild(rendered);
}

/**
 * Render `template` to a fresh `<canvas>` sized to its tile bounds.
 * Reuses the main app's per-entity drawing helpers so belts, splitters,
 * and undergrounds look identical to the generator route.
 *
 * The off-screen Pixi app is reused across extractions — for each
 * template we build a temporary container, render the scene with that
 * container as root, then snapshot the framebuffer into a new canvas.
 */
function renderTemplateCanvas(
  app: Application,
  template: BalancerShowcaseTemplate,
): HTMLCanvasElement {
  const widthPx = Math.max(1, template.width) * TILE_PX;
  const heightPx = Math.max(1, template.height) * TILE_PX;

  // Resize the renderer to fit this template exactly. PixiJS doesn't
  // mind being resized between renders — it just retargets the GL
  // viewport. Skip if already correct to avoid the resize cost.
  if (app.renderer.width !== widthPx || app.renderer.height !== heightPx) {
    app.renderer.resize(widthPx, heightPx);
  }

  const root = new Container();
  const ctx = createDrawContext();

  // Underground tunnel stripes go beneath the surface entities. Build a
  // small map of UG entities first so `drawUgTunnelStripe` can find
  // the matching output.
  const ugEntityMap = new Map<string, typeof template.entities[0]>();
  for (const e of template.entities) {
    if (UG_BELT_ENTITIES.has(e.name)) {
      ugEntityMap.set(`${e.x ?? 0},${e.y ?? 0}`, e);
    }
  }
  for (const e of template.entities) {
    if (!UG_BELT_ENTITIES.has(e.name) || e.io_type !== "input") continue;
    const stripe = drawUgTunnelStripe(e, ugEntityMap);
    if (stripe) root.addChild(stripe);
  }

  // Populate draw context first — `detectBeltTurn` reads neighbours
  // from it, so all entities must be registered before any are drawn.
  for (const e of template.entities) addEntityToDrawContext(e, ctx);
  for (const e of template.entities) {
    const g = drawEntityGraphic(e, ctx);
    root.addChild(g);
  }

  // `extract.canvas` returns the renderer's internal canvas — copy
  // pixels into a fresh canvas so this DOM element survives subsequent
  // extractions (otherwise every cell in the grid would point at the
  // same shared canvas, which then keeps mutating as we render the
  // next template).
  const sourceCanvas = app.renderer.extract.canvas(root) as HTMLCanvasElement;
  const out = document.createElement("canvas");
  out.width = sourceCanvas.width;
  out.height = sourceCanvas.height;
  const ctx2d = out.getContext("2d");
  if (ctx2d) ctx2d.drawImage(sourceCanvas, 0, 0);
  out.style.maxWidth = "100%";
  out.style.height = "auto";
  out.style.imageRendering = "pixelated";

  // Drop the children so Pixi can free their textures on the next
  // implicit GC pass. We don't `destroy()` because that recurses into
  // the shared atlas textures the entity helpers cache.
  root.removeChildren();

  return out;
}

function sourceSummary(cell: BalancerShowcaseCell): string {
  const lib = cell.library;
  if (!lib) return `<span class="bs-none">no template</span>`;
  // Color-code by source so the grid scans visually.
  const cls = (() => {
    if (lib.source.startsWith("Raynquist")) return "bs-source-raynquist";
    if (lib.source === "compose") return "bs-source-compose";
    if (lib.source === "Factorio-SAT") return "bs-source-fsat";
    return "bs-source-other";
  })();
  return `<span class="${cls}">${escapeHtml(lib.source)}</span>`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function injectStyles(): void {
  if (document.getElementById("bs-styles")) return;
  const style = document.createElement("style");
  style.id = "bs-styles";
  style.textContent = `
    #balancer-showcase {
      color: #d8d8d8;
    }
    .bs-header {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      margin-bottom: 24px;
      gap: 16px;
    }
    .bs-header h1 {
      font-size: 22px;
      margin: 0 0 4px;
      font-weight: 500;
      letter-spacing: 0.2px;
    }
    .bs-header p {
      margin: 0;
      color: #888;
      font-size: 13px;
    }
    .bs-back {
      color: #6a9ed8;
      text-decoration: none;
      font-size: 13px;
    }
    .bs-back:hover { text-decoration: underline; }

    .bs-grid {
      display: grid;
      grid-template-columns: repeat(var(--bs-cols, 10), minmax(0, 1fr));
      gap: 12px;
    }
    @media (max-width: 1400px) {
      .bs-grid { grid-template-columns: repeat(5, minmax(0, 1fr)); }
    }
    @media (max-width: 800px) {
      .bs-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    }

    .bs-cell {
      background: #232323;
      border: 1px solid #2e2e2e;
      border-radius: 6px;
      padding: 8px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      min-width: 0;
    }
    .bs-cell.bs-focus {
      border-color: #d39a3a;
      box-shadow: 0 0 0 2px rgba(211, 154, 58, 0.25);
    }
    .bs-cell-header {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      gap: 6px;
      font-size: 12px;
    }
    .bs-shape {
      font-weight: 600;
      color: #f0f0f0;
      font-variant-numeric: tabular-nums;
    }
    .bs-summary { color: #888; font-size: 11px; font-weight: 500; }
    .bs-summary .bs-source-raynquist { color: #d39a3a; }
    .bs-summary .bs-source-compose   { color: #4eb072; }
    .bs-summary .bs-source-fsat      { color: #6a9ed8; }
    .bs-summary .bs-source-other     { color: #aaa; }
    .bs-summary .bs-none             { color: #555; }

    .bs-panels {
      display: grid;
      grid-template-columns: 1fr;
      gap: 6px;
    }
    .bs-panel {
      background: #1a1a1a;
      border-radius: 4px;
      padding: 6px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      min-height: 80px;
    }
    .bs-panel.bs-empty {
      opacity: 0.45;
      justify-content: center;
      align-items: center;
      text-align: center;
    }
    .bs-panel-caption {
      font-size: 10px;
      line-height: 1.3;
      color: #aaa;
    }
    .bs-panel-caption .bs-source {
      font-weight: 500;
      color: #d8d8d8;
      display: flex;
      align-items: baseline;
      gap: 6px;
    }
    .bs-panel-caption .bs-strategy {
      color: #888;
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      font-size: 9px;
      margin-top: 2px;
      word-break: break-word;
    }
    .bs-panel-caption .bs-reference a {
      color: #6a9ed8;
      text-decoration: none;
      font-size: 9px;
    }
    .bs-panel-caption .bs-reference a:hover {
      text-decoration: underline;
    }
    .bs-panel-caption .bs-stats {
      color: #888;
      font-variant-numeric: tabular-nums;
      margin-top: 2px;
    }
    .bs-panel-caption strong {
      color: #d8d8d8;
      font-weight: 600;
    }
    .bs-panel-canvas {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 0;
      max-height: 320px;
      overflow: hidden;
    }
    .bs-panel-canvas canvas {
      display: block;
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }
  `;
  document.head.appendChild(style);
}
