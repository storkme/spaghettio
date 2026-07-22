export interface DebugState {
  master: boolean;
  stepThrough: boolean;
  satZones: boolean;
  soloRegions: boolean;
  ghostTiles: boolean;
  itemColors: boolean;
  traceOverlay: boolean;
  /** Starvation heatmap: tint machines by delivered/needed ratio. */
  heatmap: boolean;
  /** Power connectivity: draw the pole copper-wire network. */
  powerWires: boolean;
  /** Module slots (RFC-044 Phase 2): draw the in-game-style module slot
   *  row on entities carrying `items`. */
  moduleSlots: boolean;
  /** Sim-state overlay (RFC-050 Phase 4): tint machines/belts/inserters
   *  from a loaded `spaghettio-sim` report. The checkbox itself is
   *  disabled in the DOM (`overlayPanel.ts`) whenever no report is
   *  loaded — this flag just remembers the user's preference for next
   *  time one is. */
  simState: boolean;
}

type Subscriber = (state: DebugState) => void;

let state: DebugState = {
  master: false,
  stepThrough: true,
  satZones: false,
  soloRegions: false,
  ghostTiles: false,
  itemColors: true,
  traceOverlay: false,
  heatmap: false,
  powerWires: false,
  moduleSlots: true,
  simState: true,
};

const subs: Subscriber[] = [];

export function create(): void {
  const fromParam = new URLSearchParams(window.location.search).get("debug") === "1";
  const fromStorage = localStorage.getItem("fk-debug") === "1";
  const satFromStorage = localStorage.getItem("fk-sat-zones") === "1";
  const ghostFromStorage = localStorage.getItem("fk-ghost-tiles") === "1";
  const itemColorsStored = localStorage.getItem("fk-item-colors");
  const traceOverlayStored = localStorage.getItem("fk-trace-overlay") === "1";
  const heatmapStored = localStorage.getItem("fk-heatmap") === "1";
  const powerWiresStored = localStorage.getItem("fk-power-wires") === "1";
  const moduleSlotsStored = localStorage.getItem("fk-module-slots");
  const simStateStored = localStorage.getItem("fk-sim-state");
  state = {
    ...state,
    master: fromParam || fromStorage,
    satZones: satFromStorage,
    ghostTiles: ghostFromStorage,
    itemColors: itemColorsStored === null ? true : itemColorsStored === "1",
    traceOverlay: traceOverlayStored,
    heatmap: heatmapStored,
    powerWires: powerWiresStored,
    // Default ON (RFC-044 Phase 2) — the overlay only draws on entities
    // that actually carry `items`, so a generated layout with no modules
    // stays quiet by default; same "stored===null → default true"
    // pattern as itemColors.
    moduleSlots: moduleSlotsStored === null ? true : moduleSlotsStored === "1",
    // Default ON (RFC-050 Phase 4) — same reasoning: the overlay only
    // draws once a report is loaded (the DOM checkbox is disabled until
    // then, see `overlayPanel.ts`), so defaulting true is quiet until
    // it's meaningful.
    simState: simStateStored === null ? true : simStateStored === "1",
  };
}

export function get(): DebugState {
  return state;
}

export function set(patch: Partial<DebugState>): void {
  state = { ...state, ...patch };
  if ("master" in patch) {
    localStorage.setItem("fk-debug", patch.master ? "1" : "0");
  }
  if ("satZones" in patch) {
    localStorage.setItem("fk-sat-zones", patch.satZones ? "1" : "0");
  }
  if ("ghostTiles" in patch) {
    localStorage.setItem("fk-ghost-tiles", patch.ghostTiles ? "1" : "0");
  }
  if ("itemColors" in patch) {
    localStorage.setItem("fk-item-colors", patch.itemColors ? "1" : "0");
  }
  if ("traceOverlay" in patch) {
    localStorage.setItem("fk-trace-overlay", patch.traceOverlay ? "1" : "0");
  }
  if ("heatmap" in patch) {
    localStorage.setItem("fk-heatmap", patch.heatmap ? "1" : "0");
  }
  if ("powerWires" in patch) {
    localStorage.setItem("fk-power-wires", patch.powerWires ? "1" : "0");
  }
  if ("moduleSlots" in patch) {
    localStorage.setItem("fk-module-slots", patch.moduleSlots ? "1" : "0");
  }
  if ("simState" in patch) {
    localStorage.setItem("fk-sim-state", patch.simState ? "1" : "0");
  }
  for (const cb of subs) cb(state);
}

export function subscribe(cb: Subscriber): () => void {
  subs.push(cb);
  return () => {
    const i = subs.indexOf(cb);
    if (i >= 0) subs.splice(i, 1);
  };
}
