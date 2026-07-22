import * as debugState from "../state/debugState";
import "./overlayPanel.css";

export interface OverlayPanelControls {
  /** Check the master Debug toggle and reveal its sub-panel. */
  setDebugEnabled(on: boolean): void;
  debugCb: HTMLInputElement;
  colorCb: HTMLInputElement;
  heatmapCb: HTMLInputElement;
  powerWiresCb: HTMLInputElement;
  moduleSlotsCb: HTMLInputElement;
  regionsCb: HTMLInputElement;
  soloRegionsCb: HTMLInputElement;
  ghostTilesCb: HTMLInputElement;
  traceOverlayCb: HTMLInputElement;
  /** Sim-state overlay (RFC-050 Phase 4). Hidden file input behind the
   *  "Load sim report…" button; `simStateCb` starts disabled — there's
   *  nothing to show until a report loads (see `main.ts` wiring). */
  simReportFileInput: HTMLInputElement;
  simStateCb: HTMLInputElement;
}

function makeToggle(parent: HTMLElement, label: string, checked = false): HTMLInputElement {
  const cb = document.createElement("input");
  cb.type = "checkbox";
  cb.checked = checked;
  const wrap = document.createElement("div");
  wrap.className = "overlay-toggle";
  const lbl = document.createElement("label");
  lbl.appendChild(cb);
  lbl.appendChild(document.createTextNode(label));
  wrap.appendChild(lbl);
  parent.appendChild(wrap);
  return cb;
}

export function createOverlayPanel(container: HTMLElement): OverlayPanelControls {
  container.style.position = "relative";

  const panel = document.createElement("div");
  panel.className = "overlay-panel";

  const state = debugState.get();
  const debugCb = makeToggle(panel, "Debug", state.master);
  const colorCb = makeToggle(panel, "Item colours", state.itemColors);
  // User-facing (not under Debug): tint machines by delivered/needed.
  const heatmapCb = makeToggle(panel, "Starvation heatmap", state.heatmap);
  // User-facing (not under Debug): draw the pole copper-wire network.
  const powerWiresCb = makeToggle(panel, "Power wires", state.powerWires);
  // User-facing (not under Debug): draw module slots on machines
  // (RFC-044 Phase 2). Default on — quiet on generated layouts since
  // nothing stamps modules yet; imported blueprints are today's data.
  const moduleSlotsCb = makeToggle(panel, "Module slots", state.moduleSlots);

  // Sim-state overlay (RFC-050 Phase 4): a "Load sim report…" button
  // (hidden file input, triggered by click — same one-button-drives-a-
  // hidden-input shape as the Ctrl+O snapshot loader in `main.ts`) plus
  // the toggle. The load affordance lives right above the toggle since
  // the checkbox does nothing without a report loaded; `main.ts` starts
  // it `disabled` and flips that on report load/clear.
  const simReportRow = document.createElement("div");
  simReportRow.className = "overlay-sim-report-row";
  const simReportBtn = document.createElement("button");
  simReportBtn.type = "button";
  simReportBtn.textContent = "Load sim report…";
  simReportBtn.className = "overlay-sim-report-btn";
  const simReportFileInput = document.createElement("input");
  simReportFileInput.type = "file";
  simReportFileInput.accept = ".json,application/json";
  simReportFileInput.style.display = "none";
  simReportBtn.addEventListener("click", () => simReportFileInput.click());
  simReportRow.appendChild(simReportBtn);
  simReportRow.appendChild(simReportFileInput);
  panel.appendChild(simReportRow);

  const simStateCb = makeToggle(panel, "Sim state", state.simState);
  // Nothing to overlay until a report is loaded — `main.ts` re-enables
  // this once `simReportPanel`'s onChange fires with a report.
  simStateCb.disabled = true;

  const subPanel = document.createElement("div");
  subPanel.className = "overlay-sub-panel";
  subPanel.style.display = state.master ? "flex" : "none";

  const regionsCb = makeToggle(subPanel, "SAT Zones", state.satZones);
  const ghostTilesCb = makeToggle(subPanel, "Ghost tiles", state.ghostTiles);
  const traceOverlayCb = makeToggle(subPanel, "Trace overlay", state.traceOverlay);
  const soloRegionsCb = makeToggle(subPanel, "Solo regions", state.soloRegions);
  panel.appendChild(subPanel);

  container.appendChild(panel);

  debugCb.addEventListener("change", () => {
    subPanel.style.display = debugCb.checked ? "flex" : "none";
    debugState.set({ master: debugCb.checked });
  });

  // Persist the SAT Zones + Ghost tiles toggles alongside the master
  // Debug flag so a mid-session reload keeps those overlays visible.
  // Only user-initiated flips (change events) write to storage;
  // programmatic overrides like solo-mode save/restore deliberately
  // don't fire a change event and so won't persist.
  regionsCb.addEventListener("change", () => {
    debugState.set({ satZones: regionsCb.checked });
  });
  ghostTilesCb.addEventListener("change", () => {
    debugState.set({ ghostTiles: ghostTilesCb.checked });
  });
  traceOverlayCb.addEventListener("change", () => {
    debugState.set({ traceOverlay: traceOverlayCb.checked });
  });
  colorCb.addEventListener("change", () => {
    debugState.set({ itemColors: colorCb.checked });
  });
  heatmapCb.addEventListener("change", () => {
    debugState.set({ heatmap: heatmapCb.checked });
  });
  powerWiresCb.addEventListener("change", () => {
    debugState.set({ powerWires: powerWiresCb.checked });
  });
  moduleSlotsCb.addEventListener("change", () => {
    debugState.set({ moduleSlots: moduleSlotsCb.checked });
  });
  simStateCb.addEventListener("change", () => {
    debugState.set({ simState: simStateCb.checked });
  });

  return {
    setDebugEnabled(on: boolean): void {
      debugCb.checked = on;
      subPanel.style.display = on ? "flex" : "none";
      debugState.set({ master: on });
    },
    debugCb,
    colorCb,
    heatmapCb,
    powerWiresCb,
    moduleSlotsCb,
    regionsCb,
    soloRegionsCb,
    ghostTilesCb,
    traceOverlayCb,
    simReportFileInput,
    simStateCb,
  };
}
