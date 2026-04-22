import { Container, Graphics } from "pixi.js";
import { createApp, WORLD_SIZE } from "./renderer/app";
import { drawGrid, updateGrid } from "./renderer/grid";
import { drawGraph } from "./renderer/graph";
import { initEntityIcons, renderLayout, setItemColoring, itemColor, TILE_PX } from "./renderer/entities";
import { createSelectionController, type SelectionController } from "./renderer/selection";
import { renderSidebar } from "./ui/sidebar";
import { initCorpusPanel } from "./ui/corpus";
import { renderLanding } from "./ui/landing";
import {
  setupSnapshotDropZone,
  decodeSnapshot,
} from "./ui/snapshotLoader";
import { initEngine, getEngine } from "./engine";
import type { SolverResult, LayoutResult, PlacedEntity, ValidationIssue } from "./engine";
import { renderTraceOverlay, getTracePhases, eventsUpToPhase, type TraceEvent, type PhaseSnapshot } from "./renderer/traceOverlay";
import { renderValidationOverlay } from "./renderer/validationOverlay";
import { renderRegionOverlayDetailed, type RegionOverlayItem } from "./renderer/regionOverlay";
import { renderJunctionZoneOverlay } from "./renderer/junctionZoneOverlay";
import { createSatZoneOverlay } from "./renderer/satZoneOverlay";
import { renderGhostTilesOverlay } from "./renderer/ghostTilesOverlay";
import { groupJunctionClusters, type JunctionCluster } from "./ui/junctionTrace";
import { createJunctionDebugger } from "./ui/junctionDebugger";
import { createSatEditor } from "./ui/satEditor";
import * as debugState from "./state/debugState";
import { createOverlayPanel } from "./ui/overlayPanel";
import { createIssuesDialog } from "./ui/issuesDialog";
import { createInspector } from "./ui/inspector";
import { buildTileContext } from "./ui/tileContext";
import { createSnapshotMode } from "./ui/snapshotMode";
import { createLegendPanel, type LegendPanelControls, type LegendPanelState } from "./ui/legendPanel";
import { renderLayoutPhaseAnimated, type PhaseAnimationHandle } from "./renderer/phaseAnimation";
import { createStreamingRenderer, type StreamingRendererHandle } from "./renderer/streamingRenderer";
import { createTimelineScrubber, type TimelineScrubberHandle } from "./ui/timelineScrubber";
import "./ui/timelineScrubber.css";
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
  await initEntityIcons(MACHINE_SLUGS);

  const appRoot = document.getElementById("app")!;
  const hash = window.location.hash;
  const params = new URLSearchParams(window.location.search);
  // Skip the landing page when the URL carries any generator state:
  // item/rate/machine/in/belt. Lets shared links (e.g. layout URLs
  // pasted into chat) open straight into the generator without the
  // extra click through the landing screen.
  const hasGeneratorParams =
    params.has("item") ||
    params.has("rate") ||
    params.has("machine") ||
    params.has("in") ||
    params.has("belt");
  const skipLanding =
    hash.startsWith("#/layout") ||
    params.has("generator") ||
    hasGeneratorParams;

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

  const { app, viewport } = await createApp(container);
  const gridGfx = drawGrid(viewport);
  drawGraph(viewport, null);

  debugState.create();

  // --- Modules ---
  const overlayControls = createOverlayPanel(container);
  const { debugCb, colorCb, valCb, regionsCb, soloRegionsCb, ghostTilesCb } = overlayControls;
  // Sync the item-coloring flag with the persisted checkbox state so
  // a user who turned colours off stays off across reloads.
  setItemColoring(colorCb.checked);

  const overlayLegend: LegendPanelControls = createLegendPanel(container);

  function getLegendState(): LegendPanelState {
    return {
      hasLayout: !!lastLayout,
      debugMode: debugCb.checked,
      hasTrace: !!(lastLayout?.trace?.length),
      stepThrough: false,
      validation: valCb.checked,
      ghostTiles: ghostTilesCb.checked,
      satZones: regionsCb.checked,
    };
  }

  function updateLegend(): void {
    overlayLegend.update(getLegendState());
  }

  const inspector = createInspector(container);

  const issuesDialog = createIssuesDialog(container, app, viewport);
  issuesDialog.setOnValClose(() => {
    valCb.checked = false;
    updateValidationOverlay();
  });

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
    },
    onEditRequested: (state) => {
      entityLayer.alpha = 0.2;
      satEditor.enter(state);
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

  const entityLayer = new Container();
  viewport.addChild(entityLayer);
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
    if (!tile) return;
    const px = tile.x * TILE_PX;
    const py = tile.y * TILE_PX;
    pinHighlight.setStrokeStyle({ width: 2, color: 0x80c8ff, alpha: 0.95 });
    pinHighlight.rect(px - 2, py - 2, TILE_PX + 4, TILE_PX + 4).stroke();
  });
  viewport.moveCenter(WORLD_SIZE / 2, WORLD_SIZE / 2);

  // Click-to-inspect removed; pass no-op to renderLayout.
  const onSelect = (_entity: PlacedEntity | null): void => {};

  let hoveredEntity: PlacedEntity | null = null;
  function onHover(entity: PlacedEntity | null): void {
    hoveredEntity = entity;
    inspector.onHover(entity, entity?.x, entity?.y);
  }

  // --- Sidebar toggles ---

  let soloRegionsActive = false;
  let soloSavedState: {
    colorChecked: boolean;
    valChecked: boolean;
    regionsChecked: boolean;
    entityAlpha: number;
  } | null = null;

  let traceOverlayLayer: Container | null = null;
  let snapshotActive = false;
  let prevSnapshotEntities: Set<string> | null = null;

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
      // Step-through is about to replace entities wholesale; stop any
      // in-progress phase/streaming animation so its ticker doesn't write
      // alphas on about-to-be-destroyed graphics.
      phaseAnimHandle?.cancel();
      phaseAnimHandle = null;
      streamingHandle?.cancel();
      streamingHandle = null;
      timelineScrubber.reset();
      snapshotActive = true;
      const ctrl = renderLayout(
        { ...lastLayout!, entities: snapshot.entities, width: snapshot.width, height: snapshot.height },
        entityLayer, onHover, onSelect,
      );
      inspector.setHighlightController(ctrl);
      const newKeys = new Set(snapshot.entities.map(entityKey));
      const prev = prevSnapshotEntities;
      if (prev) {
        const added = snapshot.entities.filter(e => !prev.has(entityKey(e)));
        const addedPositions = new Set(added.map(e => `${e.x},${e.y}`));
        for (const child of entityLayer.children) {
          if (!("tint" in child) || addedPositions.size === 0) continue;
          const g = child as { x: number; y: number; tint: number };
          const tx = Math.round(g.x / TILE_PX);
          const ty = Math.round(g.y / TILE_PX);
          if (addedPositions.has(`${tx},${ty}`)) {
            g.tint = 0x44ff88;
            setTimeout(() => { g.tint = 0xffffff; }, 1000);
          }
        }
      }
      prevSnapshotEntities = newKeys;
    } else if (snapshotActive) {
      snapshotActive = false;
      prevSnapshotEntities = null;
      if (lastLayout) {
        const ctrl = renderLayout(lastLayout, entityLayer, onHover, onSelect);
        inspector.setHighlightController(ctrl);
      }
    }

    if (!debugCb.checked || !lastLayout?.trace?.length) {
      stepThrough.update();
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
  }

  let valOverlayLayer: Container | null = null;
  let valCircleMap: Map<string, Graphics[]> = new Map();
  let cachedValidationIssues: ValidationIssue[] | null = null;
  let validationInFlightFor: LayoutResult | null = null;

  let regionOverlayLayer: Container | null = null;
  let regionHitTest: ((wx: number, wy: number) => RegionOverlayItem | null) | null = null;
  let junctionOverlayLayer: Container | null = null;
  let junctionHitTest: ((wx: number, wy: number) => JunctionCluster | null) | null = null;
  let ghostTilesLayer: Container | null = null;

  function panToTile(x: number, y: number): void {
    viewport.moveCenter(x * TILE_PX + TILE_PX / 2, y * TILE_PX + TILE_PX / 2);
  }

  function updateValidationOverlay(): void {
    if (valOverlayLayer) {
      entityLayer.removeChild(valOverlayLayer);
      valOverlayLayer.destroy();
      valOverlayLayer = null;
      valCircleMap = new Map();
    }
    issuesDialog.clearPulse();
    issuesDialog.setCircleMap(valCircleMap);

    // If we don't have cached issues yet for the current layout, kick off a
    // validate in the worker and re-render when it lands. Guard against stale
    // results by checking lastLayout identity when the promise resolves.
    if (lastLayout && !cachedValidationIssues && validationInFlightFor !== lastLayout) {
      const target = lastLayout;
      validationInFlightFor = target;
      engine
        .validateLayout(target, null)
        .then((issues) => {
          if (lastLayout !== target) return; // superseded
          cachedValidationIssues = issues;
          validationInFlightFor = null;
          updateValidationOverlay();
        })
        .catch(() => {
          if (lastLayout !== target) return;
          cachedValidationIssues = [];
          validationInFlightFor = null;
          updateValidationOverlay();
        });
    }

    sidebarCtrl?.updateValidation(cachedValidationIssues ?? [], panToTile);

    if (!debugCb.checked || !valCb.checked || !lastLayout) {
      issuesDialog.populate(cachedValidationIssues ?? [], debugCb.checked, valCb.checked);
      return;
    }
    if (!cachedValidationIssues || cachedValidationIssues.length === 0) {
      issuesDialog.populate([], debugCb.checked, valCb.checked);
      return;
    }
    const result = renderValidationOverlay(
      cachedValidationIssues,
      entityLayer,
      (text) => {
        inspector.setTooltipOverride(text ? `<span style="color:#f44">VALIDATION</span> ${text}` : null);
      },
    );
    valOverlayLayer = result.layer;
    valCircleMap = result.circleMap;
    issuesDialog.setCircleMap(valCircleMap);
    issuesDialog.populate(cachedValidationIssues, debugCb.checked, valCb.checked);
  }

  function updateGhostTilesOverlay(): void {
    if (ghostTilesLayer) {
      viewport.removeChild(ghostTilesLayer);
      ghostTilesLayer.destroy({ children: true });
      ghostTilesLayer = null;
    }
    if (!debugCb.checked || !ghostTilesCb.checked || !lastLayout) return;
    const layer = renderGhostTilesOverlay(lastLayout.trace);
    if (!layer) return;
    ghostTilesLayer = layer;
    // Attach below the entity layer so belts/machines read on top of
    // the cyan wash. `addChildAt(layer, 0)` puts it at the bottom of
    // the viewport's z-order.
    viewport.addChildAt(ghostTilesLayer, 0);
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
    if (!debugCb.checked || !regionsCb?.checked || !lastLayout) return;

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
  }

  // --- Item color legend (bottom-left) ---
  const legendEl = document.createElement("div");
  legendEl.style.cssText = "position:absolute;bottom:8px;left:8px;background:rgba(0,0,0,0.6);color:#ccc;font:11px monospace;padding:4px 8px;border-radius:3px;pointer-events:none;z-index:10;display:none;max-height:300px;overflow-y:auto";
  container.appendChild(legendEl);

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

  let lastLayout: LayoutResult | null = null;
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

  const snapshotMode = createSnapshotMode({
    sidebarEl: document.getElementById("sidebar"),
    getSidebarCtrl: () => sidebarCtrl,
    renderLayoutOnCanvas,
    setCachedValidationIssues: (issues) => { cachedValidationIssues = issues; },
    updateValidationOverlay,
    panToTile,
    onDebugEnable: () => overlayControls.setDebugEnabled(true),
    onValEnable: () => { valCb.checked = true; },
    onClear: () => {
      snapshotMode.clear();
      entityLayer.removeChildren();
      inspector.clearPin();
      inspector.setTileContext(null);
      lastLayout = null;
      cachedValidationIssues = null;
      drawGraph(viewport, null);
      viewport.moveCenter(WORLD_SIZE / 2, WORLD_SIZE / 2);
      legendEl.style.display = "none";
      updateLegend();
      issuesDialog.setVisible(false);
      issuesDialog.populate([], false, false);
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
    if (!hoveredEntity) inspector.onHover(null, tx, ty);
  });

  app.canvas.addEventListener("pointerleave", () => {
    inspector.setCursorTile(null);
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
      // the cursor so the inspector freezes on its detail.
      const entity = hoveredEntity && hoveredEntity.x === tx && hoveredEntity.y === ty
        ? hoveredEntity : null;
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
    // to unpin.
    const current = inspector.getPinnedTile();
    if (current && current.x === tx && current.y === ty) {
      inspector.clearPin();
    } else {
      const entity = hoveredEntity && hoveredEntity.x === tx && hoveredEntity.y === ty
        ? hoveredEntity : null;
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
    drawGraph(viewport, result);
    legendEl.style.display = "none";
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
    return (evt) => {
      // On the first PhaseSnapshot, pan+zoom to frame the layout so streaming
      // entities are visible. Subsequent snaps don't re-pan (user may have moved).
      if (!viewportFitted && (evt as { phase?: string }).phase === "PhaseSnapshot") {
        const d = (evt as { phase: string; data: { width: number; height: number } }).data;
        if (d.width > 0 && d.height > 0) {
          viewport.fit(true, d.width * TILE_PX * 1.15, d.height * TILE_PX * 1.25);
          viewport.moveCenter(d.width * TILE_PX / 2, d.height * TILE_PX / 2);
          viewportFitted = true;
        }
      }
      streamingHandle?.onEvent(evt, (m) => timelineScrubber.noteMilestone(m.id));
    };
  }

  function buildLegend(layout: LayoutResult): void {
    legendEl.innerHTML = "";
    const items = new Set<string>();
    for (const e of layout.entities) {
      if (e.carries) items.add(e.carries);
    }
    if (items.size === 0 || !colorCb.checked) {
      legendEl.style.display = "none";
      return;
    }
    const sorted = Array.from(items).sort();
    for (const item of sorted) {
      const row = document.createElement("div");
      row.style.cssText = "display:flex;align-items:center;gap:5px;padding:1px 0";
      const swatch = document.createElement("span");
      const color = itemColor(item);
      const hex = "#" + color.toString(16).padStart(6, "0");
      swatch.style.cssText = `display:inline-block;width:12px;height:12px;background:${hex};border-radius:2px;flex-shrink:0`;
      row.appendChild(swatch);
      const label = document.createElement("span");
      label.textContent = item;
      row.appendChild(label);
      legendEl.appendChild(row);
    }
    legendEl.style.display = "block";
  }

  function renderLayoutOnCanvas(layout: LayoutResult): void {
    lastLayout = layout;
    (window as unknown as { __layout?: LayoutResult }).__layout = layout;
    logLayoutStats(layout);
    stepThrough.reset();
    snapshotActive = false;
    prevSnapshotEntities = null;
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
      // the authoritative `renderLayout` — this destroys the transient
      // graphics, draws `layout.entities`, and returns the real
      // HighlightController. Keep `streamingHandle` alive so the
      // scrubber's `onSeek` callback can drive `seekTo()`.
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
        ctrl = renderLayout(layout, entityLayer, onHover, onSelect);
      }
    }
    inspector.setHighlightController(ctrl);
    inspector.setTileContext(buildTileContext(layout.trace));
    inspector.clearPin();
    selectionCtrl = createSelectionController(app.canvas, viewport, entityLayer, layout, onSelectionChange);
    buildLegend(layout);
    updateTraceOverlay();
    updateValidationOverlay();
    updateRegionOverlay();
    updateGhostTilesOverlay();
    updateLegend();
    const w = layout.width ?? 0;
    const h = layout.height ?? 0;
    updateGrid(gridGfx, w + 2, h + 2);
    if (w > 0 && h > 0) {
      const pxW = w * 32;
      const pxH = h * 32;
      viewport.fit(true, pxW * 1.1, pxH * 1.2);
      viewport.moveCenter(pxW / 2, pxH / 2);
    }
    if (soloRegionsActive) {
      entityLayer.alpha = 0.12;
    }
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
    }, {
      getDebugMode: () => debugCb.checked,
    });

    // Wire overlay panel toggles
    debugCb.addEventListener("change", () => {
      stepThrough.reset();
      updateTraceOverlay();
      updateValidationOverlay();
      updateRegionOverlay();
      updateGhostTilesOverlay();
      updateLegend();
    });
    ghostTilesCb.addEventListener("change", () => {
      updateGhostTilesOverlay();
      updateLegend();
    });
    colorCb.addEventListener("change", () => {
      setItemColoring(colorCb.checked);
      if (!colorCb.checked) {
        legendEl.style.display = "none";
      }
      if (lastLayout) {
        renderLayoutOnCanvas(lastLayout);
      }
    });
    valCb.addEventListener("change", () => {
      updateValidationOverlay();
      updateLegend();
    });
    regionsCb.addEventListener("change", () => {
      updateRegionOverlay();
      updateLegend();
    });

    soloRegionsCb.addEventListener("change", () => {
      if (soloRegionsCb.checked) {
        soloRegionsActive = true;
        soloSavedState = {
          colorChecked: colorCb.checked,
          valChecked: valCb.checked,
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
        if (valCb.checked) {
          valCb.checked = false;
          updateValidationOverlay();
        }

        entityLayer.alpha = 0.12;
        updateRegionOverlay();
      } else {
        soloRegionsActive = false;
        if (soloSavedState) {
          entityLayer.alpha = soloSavedState.entityAlpha;

          if (regionsCb.checked !== soloSavedState.regionsChecked) {
            regionsCb.checked = soloSavedState.regionsChecked;
            updateRegionOverlay();
          }
          if (valCb.checked !== soloSavedState.valChecked) {
            valCb.checked = soloSavedState.valChecked;
            updateValidationOverlay();
          }
          if (colorCb.checked !== soloSavedState.colorChecked) {
            colorCb.checked = soloSavedState.colorChecked;
            setItemColoring(colorCb.checked);
            if (lastLayout) renderLayoutOnCanvas(lastLayout);
          }

          soloSavedState = null;
        }
      }
    });

    initCorpusPanel(corpusPanel, renderLayoutOnCanvas);
  }
}

main().catch((err) => {
  console.error("Failed to initialize app:", err);
});
