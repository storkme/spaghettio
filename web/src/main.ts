import { Container, Graphics } from "pixi.js";
import { createApp, WORLD_SIZE } from "./renderer/app";
import { drawGrid, updateGrid } from "./renderer/grid";
import { drawGraph } from "./renderer/graph";
import { initEntityIcons, preloadCarriesIcons, renderLayout, setItemColoring, TILE_PX, MACHINE_SIZES, SPLITTER_ENTITIES, splitterCompanionOffset, type HighlightController, extractCarriesFromEntities } from "./renderer/entities";
import { createParticleScene, renderLayoutAsParticles } from "./renderer/particleLayout";
import { renderInputLabels } from "./renderer/inputLabels";
import { createSelectionController, type SelectionController } from "./renderer/selection";
import { renderSidebar } from "./ui/sidebar";
import { initCorpusPanel } from "./ui/corpus";
import { renderLanding } from "./ui/landing";
import {
  setupSnapshotDropZone,
  decodeSnapshot,
} from "./ui/snapshotLoader";
import { initEngine, getEngine } from "./engine";
import { urlHasGeneratorState } from "./state";
import type { SolverResult, LayoutResult, PlacedEntity, ValidationIssue, SatImprovement } from "./engine";
import { renderTraceOverlay, getTracePhases, eventsUpToPhase, type TraceEvent, type PhaseSnapshot } from "./renderer/traceOverlay";
import { renderValidationOverlay } from "./renderer/validationOverlay";
import { renderStarvationHeatmap } from "./renderer/heatmapOverlay";
import { renderPowerWiresOverlay } from "./renderer/powerWiresOverlay";
import { renderRegionOverlayDetailed, type RegionOverlayItem } from "./renderer/regionOverlay";
import { renderJunctionZoneOverlay } from "./renderer/junctionZoneOverlay";
import { createSatZoneOverlay } from "./renderer/satZoneOverlay";
import { renderGhostTilesOverlay } from "./renderer/ghostTilesOverlay";
import { groupJunctionClusters, type JunctionCluster } from "./ui/junctionTrace";
import { createJunctionDebugger } from "./ui/junctionDebugger";
import { createSatEditor } from "./ui/satEditor";
import * as debugState from "./state/debugState";
import { createOverlayPanel } from "./ui/overlayPanel";
import { createRetryPanel } from "./ui/retryPanel";
import { createInspector } from "./ui/inspector";
import { buildTileContext, type TileContext } from "./ui/tileContext";
import { createSnapshotMode } from "./ui/snapshotMode";
import { renderLayoutPhaseAnimated, type PhaseAnimationHandle } from "./renderer/phaseAnimation";
import { spawnRegionFlash } from "./renderer/improvementAnimation";
import { createStreamingRenderer, type StreamingRendererHandle } from "./renderer/streamingRenderer";
import { createTimelineScrubber, type TimelineScrubberHandle } from "./ui/timelineScrubber";
import "./ui/timelineScrubber.css";
import "./ui/validationBadge.css";
import { attachBusyOverlay } from "./ui/busyOverlay";
import { logLayoutStats } from "./ui/layoutTimingLog";

const MACHINE_SLUGS = [
  "assembling-machine-1", "assembling-machine-2", "assembling-machine-3",
  "electric-furnace", "steel-furnace", "stone-furnace",
  "chemical-plant", "oil-refinery", "centrifuge", "lab", "rocket-silo",
  "foundry", "electromagnetic-plant", "cryogenic-plant", "biochamber", "biolab",
  "recycler", "crusher", "beacon", "storage-tank", "electric-mining-drill",
];

async function main(): Promise<void> {
  await initEngine();
  const engine = getEngine();
  // The slug↔short-id table is a static JSON snapshot import, no init
  // needed. See `web/src/shortIds.ts` and the Rust drift test
  // `short_ids::tests::snapshot_matches_algorithm`.
  await initEntityIcons(MACHINE_SLUGS);
  // Carries-icon preload is now scoped per-layout (see sidebar's solve flow
  // and renderLayoutOnCanvas). Pre-loading every producible item up front
  // gated first paint by ~5s on cold dev-server starts.

  const appRoot = document.getElementById("app")!;
  const hash = window.location.hash;
  const params = new URLSearchParams(window.location.search);

  // Balancer showcase QA tool — issue #274. Standalone route that
  // renders the library + compose-bake balancer templates side-by-side
  // for every (n_inputs, n_outputs) shape. Bypasses landing + generator
  // entirely.
  if (hash.startsWith("#/balancers")) {
    const { renderBalancerShowcase } = await import("./ui/balancers");
    renderBalancerShowcase(appRoot, engine);
    return;
  }

  // Skip the landing page when the URL carries any generator state. Both
  // legacy `?item=...` query strings and new `#/l/...` hash fragments
  // count — `urlHasGeneratorState` knows about both shapes.
  const skipLanding =
    hash.startsWith("#/layout") ||
    params.has("generator") ||
    urlHasGeneratorState();

  if (!skipLanding) {
    const landingHost = document.createElement("div");
    appRoot.appendChild(landingHost);

    renderLanding(landingHost, engine, {
      onOpenGenerator: () => {
        landingHost.remove();
        initGenerator(engine);
        window.history.replaceState({}, "", "#/layout");
      },
    });
    return;
  }

  initGenerator(engine);
}

async function initGenerator(engine: ReturnType<typeof getEngine>): Promise<void> {
  const container = document.getElementById("canvas-container");
  if (!container) throw new Error("Missing #canvas-container element");

  const appRoot = document.getElementById("app")!;
  appRoot.style.display = "flex";
  const sidebar = document.getElementById("sidebar");
  if (sidebar) sidebar.style.display = "";
  container.style.display = "";

  const { app, viewport, requestRender, beginAnimating, endAnimating } = await createApp(container);
  const gridGfx = drawGrid(viewport);
  // Tracks whether the grid has been animated in. Once revealed it stays
  // visible across subsequent layout loads — only the dimensions are
  // updated. The fade is a "this is the canvas" signal, shown once per
  // session.
  let gridRevealed = false;
  drawGraph(viewport, null);

  debugState.create();

  // --- Modules ---
  const overlayControls = createOverlayPanel(container);
  const { debugCb, colorCb, heatmapCb, powerWiresCb, regionsCb, soloRegionsCb, ghostTilesCb, traceOverlayCb } = overlayControls;
  const retryPanel = createRetryPanel(container);
  // Sync the item-coloring flag with the persisted checkbox state so
  // a user who turned colours off stays off across reloads.
  setItemColoring(colorCb.checked);

  // Sync __ANIM_LOGS with the debug checkbox. The flag is read by the
  // particle-layout animate helpers (Phase 1b+) and by streamingRenderer.ts
  // animation call sites. Mirrors the __TRACE_LOGS pattern.
  const syncAnimLogs = (): void => {
    (globalThis as { __ANIM_LOGS?: boolean }).__ANIM_LOGS = debugCb.checked;
  };
  debugCb.addEventListener("change", syncAnimLogs);
  syncAnimLogs();

  const inspector = createInspector(container);

  // Detailed PIXI overlay for the selected SAT zone. Added to the
  // viewport (not entityLayer) so the entityLayer-dim on select
  // doesn't drag the overlay down with it.
  const satZoneOverlay = createSatZoneOverlay();

  // Last-known selection. Used by the canvas pointerdown handler to
  // check whether a click landed outside the current bbox (→ deselect).
  let selectedJunction: { bboxX: number; bboxY: number; bboxW: number; bboxH: number } | null = null;

  const junctionDebugger = createJunctionDebugger(container, viewport, {
    onChange: (state) => {
      satZoneOverlay.update(state);
      if (state) {
        // Dim everything else so the SAT overlay pops. Edit mode dims
        // further (handled by the editor itself).
        entityLayer.alpha = satEditor.isActive() ? 0.2 : 0.35;
        const b = state.iter.bbox;
        selectedJunction = { bboxX: b.x, bboxY: b.y, bboxW: b.w, bboxH: b.h };
      } else {
        entityLayer.alpha = 1;
        selectedJunction = null;
        // If the user deselects the zone while editing, exit cleanly.
        if (satEditor.isActive()) satEditor.exit();
      }
      requestRender();
    },
    onEditRequested: (state) => {
      entityLayer.alpha = 0.2;
      satEditor.enter(state);
      requestRender();
    },
  });

  // Phase F SAT-zone editor — owns the painted + ghost PIXI layers,
  // toolbar inside the inline panel, hotkeys, and SAT-with-pins
  // validity loop. Created after junctionDebugger so the controls
  // reference is available, but the dependency the other way (jd
  // calling editor) flows through the onEditRequested callback above.
  const satEditor = createSatEditor({
    viewport,
    canvas: app.canvas as HTMLCanvasElement,
    engine,
    jd: junctionDebugger,
    satZoneOverlayLayer: satZoneOverlay.layer,
  });

  setupSnapshotDropZone(container, (snap) => snapshotMode.load(snap));

  /**
   * Render a layout using particles for all entity types. Clears `entityLayer`,
   * creates a fresh `ParticleScene`, attaches it, and commits all entities.
   * Returns a particle-aware `HighlightController`.
   *
   * Used by: non-streaming path in `renderLayoutOnCanvas`, and `runAutoOptimize`.
   */
  function renderLayoutWithParticles(layout: LayoutResult): HighlightController {
    entityLayer.removeChildren();
    const scene = createParticleScene();
    scene.attachTo(entityLayer);
    return renderLayoutAsParticles(layout, scene, app);
  }

  const entityLayer = new Container();
  // Cache the entity layer's GPU instruction buffer across frames. The static
  // bus layout (thousands of Graphics) doesn't change between renders, so
  // re-walking the scene graph and rebuilding draw calls every frame is
  // pure waste. Add/remove operations (renderLayout commit, overlay toggle)
  // invalidate the group; transform / alpha changes on children don't.
  entityLayer.isRenderGroup = true;
  // Disable Pixi's recursive hit-testing on the entity layer. Hover detection
  // is now done via tileEntityMap in the canvas pointermove handler; per-entity
  // Pixi events are no longer registered for hover (only click via onSelect).
  entityLayer.eventMode = "none";
  viewport.addChild(entityLayer);
  // External-input trunk labels (issue #196). Sits above the entity layer
  // so dimming the entities (solo regions, junction selection) doesn't
  // make the labels unreadable. Rebuilt by `renderLayoutOnCanvas`.
  const inputLabelsLayer = new Container();
  inputLabelsLayer.eventMode = "none";
  viewport.addChild(inputLabelsLayer);
  // SAT-zone detail overlay sits above the entity layer so the bbox,
  // boundary bars, and item icons always read on top of the belts.
  viewport.addChild(satZoneOverlay.layer);
  // Pin-highlight ring — drawn on top of everything so the user can
  // always see which tile the detail panel is describing.
  const pinHighlight = new Graphics();
  pinHighlight.label = "pin-highlight";
  viewport.addChild(pinHighlight);
  inspector.onPinChange((tile) => {
    pinHighlight.clear();
    if (tile) {
      const px = tile.x * TILE_PX;
      const py = tile.y * TILE_PX;
      pinHighlight.setStrokeStyle({ width: 2, color: 0x80c8ff, alpha: 0.95 });
      pinHighlight.rect(px - 2, py - 2, TILE_PX + 4, TILE_PX + 4).stroke();
    }
    requestRender();
  });
  viewport.moveCenter(WORLD_SIZE / 2, WORLD_SIZE / 2);

  // Click-to-inspect removed; pass no-op to renderLayout.
  const onSelect = (_entity: PlacedEntity | null): void => {};

  let hoveredEntity: PlacedEntity | null = null;
  function onHover(entity: PlacedEntity | null): void {
    hoveredEntity = entity;
    inspector.onHover(entity, entity?.x, entity?.y);
  }

  // Tile→entity map for canvas-level hover detection. Rebuilt whenever the
  // layout changes (see rebuildTileEntityMap calls). Replaces per-entity Pixi
  // pointer events to avoid the hitTestMoveRecursive scene-walk on every
  // pointermove.
  let tileEntityMap: Map<string, PlacedEntity> = new Map();

  function rebuildTileEntityMap(layout: LayoutResult): void {
    const m = new Map<string, PlacedEntity>();
    for (const e of layout.entities) {
      const x = e.x ?? 0;
      const y = e.y ?? 0;
      const sz = MACHINE_SIZES[e.name];
      if (sz) {
        const [w, h] = sz;
        for (let dy = 0; dy < h; dy++) {
          for (let dx = 0; dx < w; dx++) {
            m.set(`${x + dx},${y + dy}`, e);
          }
        }
      } else if (SPLITTER_ENTITIES.has(e.name)) {
        m.set(`${x},${y}`, e);
        const [cdx, cdy] = splitterCompanionOffset(e.direction);
        m.set(`${x + cdx},${y + cdy}`, e);
      } else {
        m.set(`${x},${y}`, e);
      }
    }
    tileEntityMap = m;
  }

  // Wraps a HighlightController so highlight changes (alpha mutations + the
  // overlay graphic) trigger render requests. Used at every renderLayout
  // call site below — the returned controller is what we pass to inspector.
  function wrapHighlight(ctrl: HighlightController): HighlightController {
    return {
      highlightItem: (item) => { ctrl.highlightItem(item); requestRender(); },
      highlightBeltNetwork: (e) => { ctrl.highlightBeltNetwork(e); requestRender(); },
      clearHighlight: () => { ctrl.clearHighlight(); requestRender(); },
      chainKey: ctrl.chainKey,
    };
  }

  // --- Sidebar toggles ---

  let soloRegionsActive = false;
  let soloSavedState: {
    colorChecked: boolean;
    regionsChecked: boolean;
    entityAlpha: number;
  } | null = null;

  let traceOverlayLayer: Container | null = null;
  let snapshotActive = false;
  let prevSnapshotEntityList: PlacedEntity[] = [];
  let seekAnimHandle: { cancel(): void } | null = null;
  let prevPhaseIndexForAnim = -1;

  function entityKey(e: PlacedEntity): string {
    return `${e.x},${e.y},${e.name},${e.recipe ?? ""}`;
  }

  function getSnapshotForPhase(
    events: TraceEvent[],
    phaseIndex: number,
  ): { entities: PlacedEntity[]; width: number; height: number } | null {
    const phases = getTracePhases(events);
    if (phaseIndex < 0 || phaseIndex >= phases.length) return null;
    const phaseName = phases[phaseIndex].name;
    for (const evt of events) {
      if (evt.phase === "PhaseSnapshot" && (evt as PhaseSnapshot).data.phase === phaseName) {
        return (evt as PhaseSnapshot).data;
      }
    }
    return null;
  }

  // The step-through bar was superseded by the timeline scrubber.
  // Keep a stub here so `updateTraceOverlay` and friends can still
  // call into it without a DOM or visible UI. `getPhaseIndex()` always
  // returning -1 disables the trace-snapshot codepath.
  const stepThrough = {
    update(): void {},
    getPhaseIndex(): number { return -1; },
    reset(): void {},
  };

  /** Animation log — gated on `window.__ANIM_LOGS`. */
  const animLog = (phase: string, detail: Record<string, unknown>): void => {
    if (!(globalThis as { __ANIM_LOGS?: boolean }).__ANIM_LOGS) return;
    // eslint-disable-next-line no-console
    console.log(`[anim t=${performance.now().toFixed(0)}ms] ${phase}`, detail);
  };

  /* Stagger-fade entities that are new in `nextList` relative to `prevList`.
   * Only called on consecutive forward phase steps (N → N+1).
   * Backward steps and jumps stay instant. */
  function runSeekAnimation(
    prevList: PlacedEntity[],
    nextList: PlacedEntity[],
    gfxMap: Map<string, Graphics[]>,
  ): { cancel(): void } {
    const prevKeys = new Set(prevList.map(entityKey));
    const added = nextList
      .filter(e => !prevKeys.has(entityKey(e)))
      .sort((a, b) => {
        const dy = (a.y ?? 0) - (b.y ?? 0);
        return dy !== 0 ? dy : (a.x ?? 0) - (b.x ?? 0);
      });

    if (added.length === 0) return { cancel() {} };

    animLog("seek_step", { added: added.length });

    const SEEK_FADE_MS = 160;
    const stagger = Math.min(7, 450 / added.length);
    const t0 = performance.now();
    let pointer = 0;
    let done = false;

    const reveals = added
      .map((e, i) => ({ gfx: gfxMap.get(entityKey(e)) ?? [], startMs: t0 + i * stagger }))
      .filter(r => r.gfx.length > 0);

    for (const r of reveals) for (const g of r.gfx) g.alpha = 0;

    const tick = (): void => {
      if (done) return;
      const now = performance.now();
      for (let i = pointer; i < reveals.length; i++) {
        const r = reveals[i];
        if (r.startMs > now) break;
        const t = Math.min(1, (now - r.startMs) / SEEK_FADE_MS);
        for (const g of r.gfx) g.alpha = t;
      }
      while (pointer < reveals.length &&
             performance.now() - reveals[pointer].startMs >= SEEK_FADE_MS) {
        for (const g of reveals[pointer].gfx) g.alpha = 1;
        pointer++;
      }
      if (pointer >= reveals.length) {
        done = true;
        app.ticker.remove(tick);
        endAnimating();
      }
    };

    app.ticker.add(tick);
    beginAnimating();
    return {
      cancel() {
        if (done) return;
        done = true;
        app.ticker.remove(tick);
        endAnimating();
        for (const r of reveals) for (const g of r.gfx) g.alpha = 1;
        requestRender();
      },
    };
  }

  function updateTraceOverlay(): void {
    if (traceOverlayLayer) {
      entityLayer.removeChild(traceOverlayLayer);
      traceOverlayLayer.destroy();
      traceOverlayLayer = null;
    }

    const phaseIndex = stepThrough.getPhaseIndex();
    const wantSnapshot = debugCb.checked && phaseIndex >= 0 && !!lastLayout?.trace;
    const snapshot = wantSnapshot
      ? getSnapshotForPhase(lastLayout!.trace as TraceEvent[], phaseIndex)
      : null;

    if (snapshot) {
      seekAnimHandle?.cancel();
      seekAnimHandle = null;
      phaseAnimHandle?.cancel();
      phaseAnimHandle = null;
      streamingHandle?.cancel();
      streamingHandle = null;
      timelineScrubber.reset();
      snapshotActive = true;

      const gfxMap = new Map<string, Graphics[]>();
      const ctrl = renderLayout(
        { ...lastLayout!, entities: snapshot.entities, width: snapshot.width, height: snapshot.height },
        entityLayer, onHover, onSelect,
        (entity, gfx) => { gfxMap.set(entityKey(entity), gfx); },
      );
      inspector.setHighlightController(wrapHighlight(ctrl));
      requestRender();

      // Animate only consecutive forward steps (N → N+1); jumps and backward stays instant.
      if (phaseIndex === prevPhaseIndexForAnim + 1 && prevSnapshotEntityList.length > 0) {
        seekAnimHandle = runSeekAnimation(prevSnapshotEntityList, snapshot.entities, gfxMap);
      }
      prevPhaseIndexForAnim = phaseIndex;
      prevSnapshotEntityList = snapshot.entities.slice();
    } else if (snapshotActive) {
      seekAnimHandle?.cancel();
      seekAnimHandle = null;
      snapshotActive = false;
      prevSnapshotEntityList = [];
      prevPhaseIndexForAnim = -1;
      if (lastLayout) {
        const ctrl = renderLayout(lastLayout, entityLayer, onHover, onSelect);
        inspector.setHighlightController(wrapHighlight(ctrl));
        requestRender();
      }
    }

    if (!debugCb.checked || !traceOverlayCb.checked || !lastLayout?.trace?.length) {
      stepThrough.update();
      requestRender();
      return;
    }
    const events = phaseIndex < 0
      ? (lastLayout.trace as TraceEvent[])
      : eventsUpToPhase(lastLayout.trace as TraceEvent[], phaseIndex);
    traceOverlayLayer = renderTraceOverlay(
      events,
      lastLayout.width ?? 0,
      lastLayout.height ?? 0,
      entityLayer,
      (text) => {
        inspector.setTooltipOverride(text ? `<span style="color:#8af">TRACE</span> ${text}` : null);
      },
    );
    stepThrough.update();
    requestRender();
  }

  let valOverlayLayer: Container | null = null;
  let cachedValidationIssues: ValidationIssue[] | null = null;
  let heatmapLayer: Container | null = null;
  let lastHeatmapIssues: ValidationIssue[] = [];
  let powerWiresLayer: Container | null = null;
  let lastTileCtx: TileContext | null = null;
  let validationInFlightFor: LayoutResult | null = null;

  // Top-left static badge that surfaces the issue count whenever a layout
  // has validation problems. No click handler yet (#209 deferred). Hidden
  // when there are no issues.
  const validationBadge = document.createElement("div");
  validationBadge.className = "validation-badge";
  validationBadge.style.display = "none";
  container.appendChild(validationBadge);

  function updateValidationBadge(issues: ValidationIssue[] | null): void {
    if (!issues || issues.length === 0) {
      validationBadge.style.display = "none";
      return;
    }
    const errors = issues.filter((i) => i.severity === "Error").length;
    const warnings = issues.length - errors;
    let label: string;
    if (errors > 0 && warnings > 0) {
      label = `⚠ ${errors} error${errors > 1 ? "s" : ""}, ${warnings} warning${warnings > 1 ? "s" : ""}`;
    } else if (errors > 0) {
      label = `⚠ ${errors} error${errors > 1 ? "s" : ""}`;
    } else {
      label = `⚠ ${warnings} warning${warnings > 1 ? "s" : ""}`;
    }
    validationBadge.textContent = label;
    validationBadge.classList.toggle("has-errors", errors > 0);
    validationBadge.style.display = "block";
  }

  let regionOverlayLayer: Container | null = null;
  let regionHitTest: ((wx: number, wy: number) => RegionOverlayItem | null) | null = null;
  let junctionOverlayLayer: Container | null = null;
  let junctionHitTest: ((wx: number, wy: number) => JunctionCluster | null) | null = null;
  let ghostTilesLayer: Container | null = null;

  // Pan + (conditionally) zoom to a tile. When the viewport is fitted to
  // a large layout the scale is well below 1 (one tile is ~a few pixels),
  // so a bare moveCenter is sub-pixel and looks like nothing happened.
  // Zoom in to PAN_TARGET_SCALE first when we're below it; if the user is
  // already zoomed in further, leave their zoom alone.
  const PAN_TARGET_SCALE = 1.0;
  function panToTile(x: number, y: number): void {
    const cx = x * TILE_PX + TILE_PX / 2;
    const cy = y * TILE_PX + TILE_PX / 2;
    if (viewport.scale.x < PAN_TARGET_SCALE) {
      viewport.setZoom(PAN_TARGET_SCALE, false);
    }
    viewport.moveCenter(cx, cy);
  }

  // Synthesise validation rows from layout-level data (router warnings +
  // unresolved-crossing regions) so they share a panel with the
  // post-layout validator output. Each Unresolved region becomes one
  // clickable row at the region's centre tile, grouped by carried item.
  function buildLayoutIssues(layout: LayoutResult): ValidationIssue[] {
    const out: ValidationIssue[] = [];
    for (const region of layout.regions ?? []) {
      if (region.kind !== "unresolved") continue;
      const cx = region.x + Math.floor(region.width / 2);
      const cy = region.y + Math.floor(region.height / 2);
      const item = region.ports?.find((p) => p.item)?.item ?? "unknown";
      out.push({
        severity: "Warning",
        category: `ghost-router · ${item}`,
        message: `unresolved crossing at (${cx}, ${cy})`,
        x: cx,
        y: cy,
      });
    }
    for (const w of layout.warnings ?? []) {
      // The aggregate ghost-router crossing count is replaced by the
      // per-region rows above — skip it so we don't double-report.
      if (/^ghost router:.*unresolved crossings/i.test(w)) continue;
      out.push({
        severity: "Warning",
        category: "layout",
        message: w,
        x: undefined,
        y: undefined,
      });
    }
    return out;
  }

  function updateValidationOverlay(): void {
    if (valOverlayLayer) {
      entityLayer.removeChild(valOverlayLayer);
      valOverlayLayer.destroy();
      valOverlayLayer = null;
    }

    // Always run validation when a layout is finalised. The visuals are
    // no longer gated on the Debug toggle (#209) — issues are surfaced as
    // border outlines + a top-left badge whenever they exist.
    if (lastLayout && !cachedValidationIssues && validationInFlightFor !== lastLayout) {
      const target = lastLayout;
      validationInFlightFor = target;
      engine
        .validateLayout(target, lastSolverResult)
        .then((issues) => {
          if (lastLayout !== target) return; // superseded
          cachedValidationIssues = issues;
          validationInFlightFor = null;
          updateValidationOverlay();
          tryRunAutoOptimize(target);
        })
        .catch(() => {
          if (lastLayout !== target) return;
          cachedValidationIssues = [];
          validationInFlightFor = null;
          updateValidationOverlay();
          tryRunAutoOptimize(target);
        });
    }

    const layoutIssues = lastLayout ? buildLayoutIssues(lastLayout) : [];
    const allIssues = [...(cachedValidationIssues ?? []), ...layoutIssues];
    // Cause rollup (Phase 3b): join a rate-shaped issue to the capped
    // -side event(s) at its anchor tile. Multiple capped sides at one
    // machine with differing limits report the union — never a guess.
    const causeOf = (issue: ValidationIssue): string | null => {
      if (issue.x == null || issue.y == null || !lastTileCtx) return null;
      const sides = lastTileCtx.lookup(issue.x, issue.y).cappedSides;
      if (sides.length === 0) return null;
      const limits = [...new Set(sides.map((s) => s.limit))].sort();
      return limits.join("+");
    };
    sidebarCtrl?.updateValidation(allIssues, panToTile, causeOf);
    updateValidationBadge(allIssues);
    updateHeatmapOverlay(allIssues);

    if (!lastLayout || allIssues.length === 0) {
      requestRender();
      return;
    }
    const result = renderValidationOverlay(
      allIssues,
      entityLayer,
      (text) => {
        inspector.setTooltipOverride(text ? `<span style="color:#f44">VALIDATION</span> ${text}` : null);
      },
    );
    valOverlayLayer = result.layer;
    requestRender();
  }

  /** Starvation heatmap (RFP validation-explainability Phase 1): tint
   *  machines by the structured delivered/needed ratio on rate-shaped
   *  issues. Rebuilt whenever validation results or the toggle change;
   *  `lastHeatmapIssues` remembers the issue set so the toggle listener
   *  can rebuild without re-running validation. */
  function updateHeatmapOverlay(issues?: ValidationIssue[]): void {
    if (issues) lastHeatmapIssues = issues;
    if (heatmapLayer) {
      entityLayer.removeChild(heatmapLayer);
      heatmapLayer.destroy({ children: true });
      heatmapLayer = null;
    }
    if (!heatmapCb.checked || !lastLayout || lastHeatmapIssues.length === 0) {
      requestRender();
      return;
    }
    heatmapLayer = renderStarvationHeatmap(lastHeatmapIssues, lastLayout.entities, entityLayer);
    requestRender();
  }

  /** Power-connectivity overlay: draw the pole copper-wire network
   *  (`layout.power_wires` — the exact graph the blueprint exports).
   *  Rebuilt whenever a layout lands or the toggle changes. */
  function updatePowerWiresOverlay(): void {
    if (powerWiresLayer) {
      entityLayer.removeChild(powerWiresLayer);
      powerWiresLayer.destroy({ children: true });
      powerWiresLayer = null;
    }
    if (!powerWiresCb.checked || !lastLayout?.power_wires?.length) {
      requestRender();
      return;
    }
    powerWiresLayer = renderPowerWiresOverlay(
      lastLayout.power_wires,
      lastLayout.entities,
      entityLayer,
    );
    requestRender();
  }

  function updateGhostTilesOverlay(): void {
    if (ghostTilesLayer) {
      viewport.removeChild(ghostTilesLayer);
      ghostTilesLayer.destroy({ children: true });
      ghostTilesLayer = null;
    }
    if (!debugCb.checked || !ghostTilesCb.checked || !lastLayout) {
      requestRender();
      return;
    }
    const layer = renderGhostTilesOverlay(lastLayout.trace);
    if (!layer) {
      requestRender();
      return;
    }
    ghostTilesLayer = layer;
    // Attach below the entity layer so belts/machines read on top of
    // the cyan wash. `addChildAt(layer, 0)` puts it at the bottom of
    // the viewport's z-order.
    viewport.addChildAt(ghostTilesLayer, 0);
    requestRender();
  }

  function updateRegionOverlay(): void {
    if (regionOverlayLayer) {
      entityLayer.removeChild(regionOverlayLayer);
      regionOverlayLayer.destroy();
      regionOverlayLayer = null;
    }
    if (junctionOverlayLayer) {
      entityLayer.removeChild(junctionOverlayLayer);
      junctionOverlayLayer.destroy();
      junctionOverlayLayer = null;
    }
    regionHitTest = null;
    junctionHitTest = null;
    if (!debugCb.checked || !regionsCb?.checked || !lastLayout) {
      requestRender();
      return;
    }

    if (lastLayout.regions && lastLayout.regions.length > 0) {
      const detailed = renderRegionOverlayDetailed(lastLayout);
      regionOverlayLayer = detailed.layer;
      regionHitTest = detailed.hitTest;
      entityLayer.addChild(regionOverlayLayer);
    }

    // Junction overlay: derived from trace events, drawn on top so it
    // takes click priority over the generic region rectangles.
    if (lastLayout.trace?.length) {
      const clusters = groupJunctionClusters(lastLayout.trace as TraceEvent[]);
      if (clusters.length > 0) {
        const jo = renderJunctionZoneOverlay(clusters);
        junctionOverlayLayer = jo.layer;
        junctionHitTest = jo.hitTest;
        entityLayer.addChild(junctionOverlayLayer);
      }
    }
    requestRender();
  }

  // --- Selection annotation bar ---
  const annotationBar = document.createElement("div");
  annotationBar.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px";
  container.appendChild(annotationBar);

  const annotationCount = document.createElement("div");
  annotationCount.style.cssText = "color:#00e0a0;margin-bottom:4px";
  annotationBar.appendChild(annotationCount);

  const annotationNote = document.createElement("textarea");
  annotationNote.placeholder = "Add a note\u2026";
  annotationNote.rows = 2;
  annotationNote.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px";
  annotationBar.appendChild(annotationNote);

  const annotationHint = document.createElement("div");
  annotationHint.style.cssText = "color:#777";
  annotationHint.textContent = "Ctrl+C to copy JSON";
  annotationBar.appendChild(annotationHint);

  let optimizeInFlight = false;

  let lastLayout: LayoutResult | null = null;
  // Most recent solver result, captured by `renderGraph(result)`. Used
  // by `renderLayoutOnCanvas` to drive the external-input trunk labels
  // (issue #196) — those need both the layout (for trunk positions)
  // and the solver result (for which items are external + their rates).
  let lastSolverResult: SolverResult | null = null;
  let selectionCtrl: SelectionController | null = null;
  let phaseAnimHandle: PhaseAnimationHandle | null = null;
  let streamingHandle: StreamingRendererHandle | null = null;

  // Floating timeline scrubber above the canvas. During live streaming
  // it shows milestone chips and a progress fill; after streaming it
  // becomes a draggable seekbar that drives streamingHandle.seekTo().
  const timelineScrubber: TimelineScrubberHandle = createTimelineScrubber(
    container,
    (virtualMs) => streamingHandle?.seekTo(virtualMs),
  );

  // Spinner that appears in the top-right while the WASM worker is
  // busy. Covers the gap between "click solve" and the first trace
  // event arriving (before the timeline scrubber has anything to show).
  attachBusyOverlay(container);

  const snapshotMode = createSnapshotMode({
    sidebarEl: document.getElementById("sidebar"),
    getSidebarCtrl: () => sidebarCtrl,
    renderLayoutOnCanvas,
    setCachedValidationIssues: (issues) => { cachedValidationIssues = issues; },
    updateValidationOverlay,
    panToTile,
    onDebugEnable: () => overlayControls.setDebugEnabled(true),
    onClear: () => {
      snapshotMode.clear();
      entityLayer.removeChildren();
      inputLabelsLayer.removeChildren();
      inspector.clearPin();
      inspector.setTileContext(null);
      lastLayout = null;
      lastSolverResult = null;
      retryPanel.update(null);
      tileEntityMap = new Map();
      cachedValidationIssues = null;
      drawGraph(viewport, null);
      viewport.moveCenter(WORLD_SIZE / 2, WORLD_SIZE / 2);
      updateValidationBadge(null);
      sidebarCtrl?.updateValidation([], panToTile);
      junctionDebugger.close();
    },
  });

  function onSelectionChange(entities: PlacedEntity[]): void {
    if (entities.length === 0) {
      annotationBar.style.display = "none";
      annotationNote.value = "";
    } else {
      annotationCount.textContent = `${entities.length} entit${entities.length === 1 ? "y" : "ies"} selected`;
      annotationBar.style.display = "block";
    }
  }

  /**
   * Round-robin "optimize every SAT zone" pass. Runs automatically
   * after a layout lands — one improvement attempt per zone per round,
   * stops when a full round yields zero wins. Visible UI is just the
   * status line + a cancel button; no start button. Cancellation
   * respawns the worker (see `cancelInFlight`).
   */
  /**
   * Auto-optimize pass with a decoupled animation queue.
   *
   * The solver streams every `SatImprovement` event into `queue`.
   * A rAF drain loop pulls one event at a time and only then applies
   * the entity splice + spawns a flash. Pacing is adaptive: consecutive
   * improvements to the same region get a growing delay so a busy
   * junction's improvements don't blur together, while switches to
   * fresh regions dequeue quickly.
   *
   * No status UI, no cancel button — if the user triggers a new solve,
   * the worker is superseded and the queue is abandoned.
   */
  async function runAutoOptimize(targetLayout: LayoutResult): Promise<void> {
    if (optimizeInFlight) return;
    const hasSatZones = (targetLayout.regions ?? []).some(
      (r) => (r as { kind?: string }).kind === "crossing_zone",
    );
    if (!hasSatZones) return;

    optimizeInFlight = true;
    timelineScrubber.markOptimizeState("active");

    // Tear down selection so renderLayout's removeChildren doesn't
    // orphan its overlays as we restamp tiles during the drain.
    if (selectionCtrl) {
      selectionCtrl.destroy();
      selectionCtrl = null;
    }

    const isBeltOrUg = (name: string): boolean =>
      name === "transport-belt" ||
      name === "fast-transport-belt" ||
      name === "express-transport-belt" ||
      name === "underground-belt" ||
      name === "fast-underground-belt" ||
      name === "express-underground-belt";

    type QueuedImp = { imp: SatImprovement };
    const queue: QueuedImp[] = [];

    // Dequeue pacing. Base gap between any two dequeues; each
    // consecutive dequeue on the same region adds a slowdown so you can
    // actually see each cheaper state land. Different region → reset.
    const BASE_GAP_MS = 130;
    const SAME_REGION_STEP_MS = 90;
    const SAME_REGION_MAX_MS = 520;

    let lastDequeueAt = 0;
    let lastRegionId = -1;
    let sameRegionRun = 0;
    let solverDone = false;
    let cancelled = false;
    let rafId: number | null = null;

    const applyImprovement = (imp: SatImprovement): void => {
      if (!lastLayout) return;
      const x0 = imp.zone_x;
      const y0 = imp.zone_y;
      const x1 = x0 + imp.zone_w;
      const y1 = y0 + imp.zone_h;
      const inBbox = (e: PlacedEntity): boolean => {
        const ex = e.x ?? 0;
        const ey = e.y ?? 0;
        return ex >= x0 && ex < x1 && ey >= y0 && ey < y1;
      };
      lastLayout.entities = lastLayout.entities
        .filter((e) => !(inBbox(e) && isBeltOrUg(e.name)))
        .concat(imp.entities);
      spawnRegionFlash(app, viewport, {
        x: x0,
        y: y0,
        w: imp.zone_w,
        h: imp.zone_h,
      });
      renderLayoutWithParticles(lastLayout);
      requestRender();
    };

    const drainDone = (): boolean => solverDone && queue.length === 0;

    const drainTick = (nowMs: number): void => {
      if (cancelled) return;
      while (queue.length > 0) {
        const head = queue[0];
        const sameRegion = head.imp.region_id === lastRegionId;
        const penalty = sameRegion
          ? Math.min(SAME_REGION_MAX_MS, sameRegionRun * SAME_REGION_STEP_MS)
          : 0;
        const requiredGap = BASE_GAP_MS + penalty;
        if (nowMs - lastDequeueAt < requiredGap) break;
        queue.shift();
        applyImprovement(head.imp);
        if (sameRegion) sameRegionRun += 1;
        else {
          sameRegionRun = 1;
          lastRegionId = head.imp.region_id;
        }
        lastDequeueAt = nowMs;
        // One dequeue per frame keeps the flashes visually paced even
        // when the queue is long.
        break;
      }
      if (drainDone()) {
        rafId = null;
        return;
      }
      rafId = requestAnimationFrame(drainTick);
    };

    rafId = requestAnimationFrame(drainTick);

    try {
      const finalLayout = await engine.optimizeAllRegions(targetLayout, {
        perRegionBudgetMs: 800,
        onImprovement: (imp) => {
          // iter=0 is the initial "snapshot" event (no cost drop and
          // the entities are just a pruned view of what's already
          // rendered — skipping avoids a subtle "half a factory"
          // regression if Rust/main-pipeline pruning diverge).
          if (imp.iter === 0) return;
          queue.push({ imp });
        },
      });
      solverDone = true;
      // Wait for the drain loop to finish animating the last items.
      await new Promise<void>((resolve) => {
        const waitForDrain = (): void => {
          if (queue.length === 0) resolve();
          else requestAnimationFrame(waitForDrain);
        };
        waitForDrain();
      });
      lastLayout = finalLayout;
      retryPanel.update(finalLayout);
      rebuildTileEntityMap(finalLayout);
      (window as unknown as { __layout?: LayoutResult }).__layout = finalLayout;
      // Final authoritative redraw with the committed layout.
      const optCtrl = renderLayoutWithParticles(finalLayout);
      inspector.setHighlightController(wrapHighlight(optCtrl));
      requestRender();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (!msg.includes("superseded")) {
        console.error("[auto-optimize] failed", err);
      }
    } finally {
      cancelled = true;
      solverDone = true;
      if (rafId !== null) cancelAnimationFrame(rafId);
      optimizeInFlight = false;
      timelineScrubber.markOptimizeState("done");
      if (lastLayout) {
        selectionCtrl = createSelectionController(
          app.canvas,
          viewport,
          entityLayer,
          lastLayout,
          onSelectionChange,
        );
      }
    }
  }

  app.canvas.addEventListener("pointermove", (e) => {
    const rect = app.canvas.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;
    const world = viewport.toWorld(sx, sy);
    const tx = Math.floor(world.x / TILE_PX);
    const ty = Math.floor(world.y / TILE_PX);
    // Cursor tile is tracked regardless of what's under the cursor so
    // the coord line stays visible even when a lane/row/ghost overlay
    // has installed a tooltip override.
    inspector.setCursorTile(tx, ty);
    // Tile-based hover detection — replaces per-entity Pixi pointer events.
    // Look up the entity at this tile and fire onHover only on change.
    const found = tileEntityMap.get(`${tx},${ty}`) ?? null;
    if (found !== hoveredEntity) onHover(found);
    if (!hoveredEntity) inspector.onHover(null, tx, ty);
  });

  app.canvas.addEventListener("pointerleave", () => {
    inspector.setCursorTile(null);
    if (hoveredEntity) onHover(null);
  });

  // Click handling for SAT regions + junction zones. Junction click
  // takes precedence: it opens the step-through modal. When a zone is
  // already selected, a click outside its bbox deselects it — matches
  // the "selected thing dims everything else" UX convention.
  //
  // Drag-vs-click discrimination: record pointer-down position, only
  // treat pointerup as a click if the pointer hasn't moved beyond
  // CLICK_DRAG_THRESHOLD_PX. Otherwise the user was panning the
  // viewport and shouldn't pin a tile.
  const CLICK_DRAG_THRESHOLD_PX = 4;
  let downState: { x: number; y: number; shifted: boolean } | null = null;
  app.canvas.addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || e.shiftKey || e.altKey || e.ctrlKey || e.metaKey) {
      downState = null;
      return;
    }
    downState = { x: e.clientX, y: e.clientY, shifted: false };
  });
  app.canvas.addEventListener("pointerup", (e) => {
    if (!downState) return;
    const dx = e.clientX - downState.x;
    const dy = e.clientY - downState.y;
    downState = null;
    if (Math.hypot(dx, dy) > CLICK_DRAG_THRESHOLD_PX) return; // it was a drag
    if (e.button !== 0 || e.shiftKey || e.altKey || e.ctrlKey || e.metaKey) return;
    const rect = app.canvas.getBoundingClientRect();
    const world = viewport.toWorld(e.clientX - rect.left, e.clientY - rect.top);
    const tx = Math.floor(world.x / TILE_PX);
    const ty = Math.floor(world.y / TILE_PX);

    if (!regionsCb.checked) {
      // Debug regions off — a bare canvas click pins the tile under
      // the cursor so the inspector freezes on its detail. Empty tiles
      // aren't pinnable: clicking one passes through without affecting
      // any current pin.
      const entity = hoveredEntity && hoveredEntity.x === tx && hoveredEntity.y === ty
        ? hoveredEntity : null;
      if (!entity) return;
      inspector.pinTile(entity, tx, ty);
      return;
    }

    const jc = junctionHitTest?.(world.x, world.y) ?? null;
    if (jc) {
      junctionDebugger.open(jc, lastLayout?.trace);
      return;
    }
    // Clicked off any junction zone. If a zone is currently selected
    // and the click wasn't on its (possibly-grown) bbox either, close
    // the debugger. We check the stored selection bbox because the
    // hit-test above uses the terminal bbox, which may differ from
    // the current iter's bbox.
    if (selectedJunction) {
      const wx = world.x / TILE_PX;
      const wy = world.y / TILE_PX;
      const inside =
        wx >= selectedJunction.bboxX &&
        wy >= selectedJunction.bboxY &&
        wx < selectedJunction.bboxX + selectedJunction.bboxW &&
        wy < selectedJunction.bboxY + selectedJunction.bboxH;
      if (!inside) {
        junctionDebugger.close();
        return;
      }
    }
    const it = regionHitTest?.(world.x, world.y) ?? null;
    if (it) {
      const cx = (it.region.x + it.region.width / 2) * TILE_PX;
      const cy = (it.region.y + it.region.height / 2) * TILE_PX;
      viewport.moveCenter(cx, cy);
      // Fall through — also pin the clicked tile so the inspector can
      // describe it in detail. Panning alone doesn't give the user any
      // detail; the combination is what they came for.
    }

    // Fell through every overlay — pin the tile so the inspector
    // keeps showing its full detail. Click on an already-pinned tile
    // to unpin. Empty tiles are not pinnable: a click on one leaves
    // the current pin (if any) alone.
    const current = inspector.getPinnedTile();
    if (current && current.x === tx && current.y === ty) {
      inspector.clearPin();
    } else {
      const entity = hoveredEntity && hoveredEntity.x === tx && hoveredEntity.y === ty
        ? hoveredEntity : null;
      if (!entity) return;
      inspector.pinTile(entity, tx, ty);
    }
  });

  // Shift held → pause viewport drag so selection box works
  document.addEventListener("keydown", (e) => {
    if (e.key === "Shift") viewport.plugins.pause("drag");
  });
  document.addEventListener("keyup", (e) => {
    if (e.key === "Shift") viewport.plugins.resume("drag");
  });
  window.addEventListener("blur", () => viewport.plugins.resume("drag"));

  function renderGraph(result: SolverResult | null): void {
    // Stop any in-flight animations before we destroy their graphics.
    phaseAnimHandle?.cancel();
    phaseAnimHandle = null;
    streamingHandle?.cancel();
    streamingHandle = null;
    timelineScrubber.reset();
    entityLayer.removeChildren();
    inputLabelsLayer.removeChildren();
    lastSolverResult = result;
    drawGraph(viewport, result);
    if (!result) {
      viewport.moveCenter(WORLD_SIZE / 2, WORLD_SIZE / 2);
    }
  }

  function startStreaming(): (evt: TraceEvent) => void {
    streamingHandle?.cancel();
    timelineScrubber.reset();
    // Remove the dependency graph — it sits on top of entityLayer in viewport's
    // child order, so it would hide streaming entities. Switch to entity view now.
    drawGraph(viewport, null);
    streamingHandle = createStreamingRenderer(entityLayer, app, onHover, onSelect);
    let viewportFitted = false;
    let gridReveal: ReturnType<typeof startGridReveal> | null = null;
    return (evt) => {
      const phase = (evt as { phase?: string }).phase;
      // On the first PhaseSnapshot, pan+zoom to frame the layout so streaming
      // entities are visible, and start drawing the grid as the "canvas" the
      // entities will be placed on. Subsequent PhaseSnapshots resize the grid
      // (the bus phase grows the layout sideways and downward) but don't
      // re-pan (user may have moved) and don't re-trigger the fade-in.
      if (phase === "PhaseSnapshot") {
        const d = (evt as { phase: string; data: { width: number; height: number } }).data;
        if (d.width > 0 && d.height > 0) {
          if (!viewportFitted) {
            viewport.fit(true, d.width * TILE_PX * 1.15, d.height * TILE_PX * 1.25);
            viewport.moveCenter(d.width * TILE_PX / 2, d.height * TILE_PX / 2);
            viewportFitted = true;
          }
          if (!gridReveal) {
            gridReveal = startGridReveal(d.width + 2, d.height + 2);
          } else {
            gridReveal.resize(d.width + 2, d.height + 2);
          }
          // First-fade-in is owned by the gridReveal handle; subsequent
          // layout loads skip the fade and just resize. The flag flips
          // when the fade animation finishes.
        }
      }
      streamingHandle?.onEvent(evt, (m) => {
        if (!streamingHandle) return;
        timelineScrubber.noteMilestone(m, streamingHandle.getTimeRange());
      });
    };
  }

  /** Reveal the grid as a top-to-bottom wipe. Returns a handle that can
   *  resize the target dimensions mid-animation (the bus-routed phase grows
   *  the layout). When the wipe completes, switches to imperative redraws
   *  on resize so subsequent PhaseSnapshots are cheap. */
  function startGridReveal(
    initialW: number,
    initialH: number,
  ): { cancel(): void; resize(w: number, h: number): void } {
    let w = initialW;
    let h = initialH;
    let done = false;

    // Fast path: the grid was already revealed once in this session. Just
    // redraw at the new size — no fade, no flicker.
    if (gridRevealed) {
      updateGrid(gridGfx, w, h);
      requestRender();
      done = true;
      return {
        cancel: () => {},
        resize(newW, newH) {
          updateGrid(gridGfx, newW, newH);
          requestRender();
        },
      };
    }

    const FADE_MS = 250;
    const start = performance.now();
    gridGfx.alpha = 1;
    updateGrid(gridGfx, w, h, 0);
    const tick = (): void => {
      if (done) return;
      const t = Math.min(1, (performance.now() - start) / FADE_MS);
      updateGrid(gridGfx, w, h, t);
      requestRender();
      if (t >= 1) {
        done = true;
        gridRevealed = true;
        app.ticker.remove(tick);
        endAnimating();
      }
    };
    app.ticker.add(tick);
    beginAnimating();
    return {
      cancel(): void {
        if (done) return;
        done = true;
        gridRevealed = true;
        app.ticker.remove(tick);
        endAnimating();
        updateGrid(gridGfx, w, h);
        requestRender();
      },
      resize(newW: number, newH: number): void {
        w = newW;
        h = newH;
        // While animating, the next tick redraws at the new size with the
        // current reveal fraction. After completion, redraw fully.
        if (done) {
          updateGrid(gridGfx, w, h);
          requestRender();
        }
      },
    };
  }

  function renderLayoutOnCanvas(layout: LayoutResult, solverResult?: SolverResult): void {
    // Scope-preload carries icons for whatever this layout actually carries.
    // Sidebar's solve flow already preloads before streaming starts, so by
    // the time we get here every needed icon is cached and this resolves
    // synchronously. The corpus / snapshot / blueprint-import paths skip
    // the sidebar entirely though, so we kick off a fire-and-forget load
    // here as a safety net — fine to render iconless on the first frame
    // for those paths since they don't stream.
    void preloadCarriesIcons(extractCarriesFromEntities(layout.entities));
    lastLayout = layout;
    retryPanel.update(layout);
    if (solverResult) lastSolverResult = solverResult;
    rebuildTileEntityMap(layout);
    (window as unknown as { __layout?: LayoutResult }).__layout = layout;
    logLayoutStats(layout);
    stepThrough.reset();
    snapshotActive = false;
    prevSnapshotEntityList = [];
    prevPhaseIndexForAnim = -1;
    seekAnimHandle?.cancel();
    seekAnimHandle = null;
    if (selectionCtrl) { selectionCtrl.destroy(); selectionCtrl = null; }
    phaseAnimHandle?.cancel();
    phaseAnimHandle = null;
    annotationBar.style.display = "none";
    annotationNote.value = "";
    cachedValidationIssues = null;
    drawGraph(viewport, null);

    let ctrl;

    if (streamingHandle?.hasCommittedEntities()) {
      // Streaming drew transient previews during layout. Hand off to
      // Streaming drew entity particles during layout. `finish()` stops
      // the live ticker and returns a particle-aware HighlightController.
      // Keep `streamingHandle` alive so the scrubber's `onSeek` callback
      // can drive `seekTo()`.
      ctrl = streamingHandle.finish(layout);
      timelineScrubber.arm(
        streamingHandle.getTimeRange(),
        streamingHandle.getMilestones(),
      );
    } else {
      // Non-streaming path: corpus, parsed blueprints, or fast layouts where
      // all snapshots arrived before any streaming frame could commit.
      streamingHandle?.cancel();
      streamingHandle = null;
      timelineScrubber.reset();
      const traceEvents = Array.isArray(layout.trace) ? layout.trace : [];
      const hasSnapshots = traceEvents.some(
        (e) => (e as { phase?: string }).phase === "PhaseSnapshot",
      );
      if (hasSnapshots) {
        const out = renderLayoutPhaseAnimated(layout, entityLayer, onHover, onSelect, app);
        ctrl = out.controller;
        phaseAnimHandle = out.handle;
      } else {
        // Use particle-based rendering for all non-streaming, non-animated paths.
        ctrl = renderLayoutWithParticles(layout);
      }
    }
    inspector.setHighlightController(wrapHighlight(ctrl));
    lastTileCtx = buildTileContext(layout.trace);
    inspector.setTileContext(lastTileCtx);
    inspector.clearPin();
    selectionCtrl = createSelectionController(app.canvas, viewport, entityLayer, layout, onSelectionChange);
    updateTraceOverlay();
    updateValidationOverlay();
    updateRegionOverlay();
    updateGhostTilesOverlay();
    updatePowerWiresOverlay();
    // External-input trunk labels (issue #196). Rebuilt on every layout
    // commit. Skips when there's no SolverResult — corpus / snapshot
    // load paths arrive without one and we don't fabricate labels there.
    renderInputLabels(inputLabelsLayer, layout, lastSolverResult);
    const w = layout.width ?? 0;
    const h = layout.height ?? 0;
    updateGrid(gridGfx, w + 2, h + 2);
    if (w > 0 && h > 0) {
      const pxW = w * 32;
      const pxH = h * 32;
      // Headroom above the layout for the input-trunk labels
      // (issue #196). The labels rise into negative world-y.
      const labelHeadroomPx = 6 * 32;
      viewport.fit(true, pxW * 1.1, (pxH + labelHeadroomPx) * 1.2);
      viewport.moveCenter(pxW / 2, (pxH - labelHeadroomPx) / 2);
    }
    if (soloRegionsActive) {
      entityLayer.alpha = 0.12;
    }
    requestRender();

    // Auto-optimize: kick off the round-robin pass after the initial
    // render has been painted. rAF gives PixiJS one tick to commit the
    // entity graphics before we start mutating them. Whichever finishes
    // last (rAF or async validation in updateValidationOverlay) ends up
    // calling tryRunAutoOptimize; the validation gate inside it skips
    // the optimize pass when the layout has any errors / warnings — no
    // point optimising a broken layout.
    requestAnimationFrame(() => {
      if (lastLayout === layout) tryRunAutoOptimize(layout);
    });
  }

  /** Run the SAT auto-optimize pass only when validation has completed
   *  for `layout` and reports no issues (validator output + layout-level
   *  warnings via `buildLayoutIssues`). If validation hasn't landed yet,
   *  this is a no-op — the validation completion handler retriggers
   *  this function. `runAutoOptimize` further guards against double-run
   *  via `optimizeInFlight`. */
  function tryRunAutoOptimize(layout: LayoutResult): void {
    if (lastLayout !== layout) return;
    if (validationInFlightFor === layout) return;
    if (cachedValidationIssues === null) return;
    const allIssues = [...cachedValidationIssues, ...buildLayoutIssues(layout)];
    if (allIssues.length > 0) return;
    void runAutoOptimize(layout);
  }

  // Ctrl+C / Ctrl+O keyboard shortcuts
  document.addEventListener("keydown", (e) => {
    if (!e.ctrlKey) return;
    if (e.key === "c") {
      if (!selectionCtrl || selectionCtrl.getSelected().length === 0) return;
      e.preventDefault();
      const params = sidebarCtrl?.getParams() ?? null;
      const json = selectionCtrl.buildJson(params, annotationNote.value.trim());
      navigator.clipboard.writeText(json).catch(() => undefined);
      annotationHint.textContent = "Copied!";
      setTimeout(() => { annotationHint.textContent = "Ctrl+C to copy JSON"; }, 2000);
    } else if (e.key === "o") {
      e.preventDefault();
      const input = document.createElement("input");
      input.type = "file";
      input.accept = ".fls";
      input.addEventListener("change", async () => {
        const file = input.files?.[0];
        if (!file) return;
        try {
          const text = await file.text();
          const snapshot = await decodeSnapshot(text);
          snapshotMode.load(snapshot);
        } catch (err) {
          alert(`Failed to load snapshot: ${err}`);
        }
      });
      input.click();
    }
  });

  const sidebarEl = document.getElementById("sidebar");
  let sidebarCtrl: ReturnType<typeof renderSidebar> | null = null;
  if (sidebarEl) {
    // ---- Tab bar ----
    const tabBar = document.createElement("div");
    tabBar.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";

    function makeTab(label: string): HTMLButtonElement {
      const btn = document.createElement("button");
      btn.textContent = label;
      btn.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s";
      return btn;
    }

    const tabGenerate = makeTab("Generate");
    const tabCorpus = makeTab("Corpus");
    tabBar.appendChild(tabGenerate);
    tabBar.appendChild(tabCorpus);

    const generatePanel = document.createElement("div");
    generatePanel.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";

    const corpusPanel = document.createElement("div");
    corpusPanel.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;";

    sidebarEl.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;";
    sidebarEl.appendChild(tabBar);
    sidebarEl.appendChild(generatePanel);
    sidebarEl.appendChild(corpusPanel);

    function switchTab(tab: "generate" | "corpus"): void {
      const isGenerate = tab === "generate";
      generatePanel.style.display = isGenerate ? "flex" : "none";
      corpusPanel.style.display = isGenerate ? "none" : "flex";
      tabGenerate.style.borderBottomColor = isGenerate ? "#569cd6" : "transparent";
      tabGenerate.style.color = isGenerate ? "#d4d4d4" : "#777";
      tabCorpus.style.borderBottomColor = isGenerate ? "transparent" : "#569cd6";
      tabCorpus.style.color = isGenerate ? "#777" : "#d4d4d4";
    }

    tabGenerate.onclick = () => switchTab("generate");
    tabCorpus.onclick = () => switchTab("corpus");
    switchTab("generate");

    sidebarCtrl = renderSidebar(generatePanel, engine, {
      renderGraph,
      renderLayout: renderLayoutOnCanvas,
      startStreaming,
    });

    // Wire overlay panel toggles
    debugCb.addEventListener("change", () => {
      stepThrough.reset();
      updateTraceOverlay();
      updateValidationOverlay();
      updateRegionOverlay();
      updateGhostTilesOverlay();
    });
    ghostTilesCb.addEventListener("change", () => {
      updateGhostTilesOverlay();
    });
    traceOverlayCb.addEventListener("change", () => {
      updateTraceOverlay();
    });
    colorCb.addEventListener("change", () => {
      setItemColoring(colorCb.checked);
      if (lastLayout) {
        renderLayoutOnCanvas(lastLayout);
      }
    });
    heatmapCb.addEventListener("change", () => {
      updateHeatmapOverlay();
    });
    powerWiresCb.addEventListener("change", () => {
      updatePowerWiresOverlay();
    });
    regionsCb.addEventListener("change", () => {
      updateRegionOverlay();
    });

    soloRegionsCb.addEventListener("change", () => {
      const finish = (): void => requestRender();
      if (soloRegionsCb.checked) {
        soloRegionsActive = true;
        soloSavedState = {
          colorChecked: colorCb.checked,
          regionsChecked: regionsCb.checked,
          entityAlpha: entityLayer.alpha,
        };

        if (!regionsCb.checked) {
          regionsCb.checked = true;
          updateRegionOverlay();
        }
        if (colorCb.checked) {
          colorCb.checked = false;
          setItemColoring(false);
          if (lastLayout) renderLayoutOnCanvas(lastLayout);
        }

        entityLayer.alpha = 0.12;
        updateRegionOverlay();
        finish();
      } else {
        soloRegionsActive = false;
        if (soloSavedState) {
          entityLayer.alpha = soloSavedState.entityAlpha;

          if (regionsCb.checked !== soloSavedState.regionsChecked) {
            regionsCb.checked = soloSavedState.regionsChecked;
            updateRegionOverlay();
          }
          if (colorCb.checked !== soloSavedState.colorChecked) {
            colorCb.checked = soloSavedState.colorChecked;
            setItemColoring(colorCb.checked);
            if (lastLayout) renderLayoutOnCanvas(lastLayout);
          }

          soloSavedState = null;
        }
        finish();
      }
    });

    initCorpusPanel(corpusPanel, renderLayoutOnCanvas);
  }
}

main().catch((err) => {
  console.error("Failed to initialize app:", err);
});
