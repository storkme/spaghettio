import type {
  SolverResult,
  LayoutResult,
  PlacedEntity,
  ValidationIssue,
  TraceEvent,
} from "./wasm-pkg/fucktorio_wasm.js";

export type {
  SolverResult,
  MachineSpec,
  ItemFlow,
  LayoutResult,
  LayoutRegion,
  RegionKind,
  RegionPort,
  PortPoint,
  PortIo,
  PlacedEntity,
  EntityDirection,
  ValidationIssue,
  TraceEvent,
} from "./wasm-pkg/fucktorio_wasm.js";

/**
 * Result shape for `engine.solveFixture` — mirrors the
 * `SolveFixtureResponse` in `crates/wasm-bindings/src/lib.rs`.
 */
export interface SolveFixtureResult {
  entities: PlacedEntity[];
  cost: number;
  stats: SatStats;
}

export interface SatStats {
  variables: number;
  clauses: number;
  solve_time_us: number;
  zone_width: number;
  zone_height: number;
}

type WorkerResponse =
  | { id: number; ok: true; result: unknown }
  | { id: number; ok: false; error: string }
  | { id: number; streamEvents: unknown[] };

interface PendingEntry {
  resolve: (v: unknown) => void;
  reject: (e: Error) => void;
  onEvent?: (evt: TraceEvent) => void;
}

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, PendingEntry>();
let activeStreamingId: number | null = null;

let itemsCache: string[] = [];
let machinesCache: string[] = [];
let defaultMachineCache = new Map<string, string>();

let activeCountListeners = new Set<(active: number) => void>();
let activeCount = 0;

function onActive(delta: number): void {
  activeCount += delta;
  for (const cb of activeCountListeners) cb(activeCount);
}

/** Subscribe to engine activity (>0 while any RPC is in flight). Returns an unsubscribe fn. */
export function onEngineActivity(cb: (active: number) => void): () => void {
  activeCountListeners.add(cb);
  cb(activeCount);
  return () => activeCountListeners.delete(cb);
}

function call<T>(payload: Record<string, unknown>, onEvent?: (evt: TraceEvent) => void): Promise<T> {
  if (!worker) throw new Error("Engine not initialized — call initEngine() first");
  const id = ++nextId;
  onActive(+1);
  return new Promise<T>((resolve, reject) => {
    pending.set(id, {
      resolve: (v) => {
        onActive(-1);
        if (activeStreamingId === id) activeStreamingId = null;
        resolve(v as T);
      },
      reject: (e) => {
        onActive(-1);
        if (activeStreamingId === id) activeStreamingId = null;
        reject(e);
      },
      onEvent,
    });
    worker!.postMessage({ id, ...payload });
  });
}

export async function initEngine(): Promise<void> {
  if (worker) return;
  worker = new Worker(new URL("./workers/engine.worker.ts", import.meta.url), {
    type: "module",
    name: "fucktorio-engine",
  });
  worker.onmessage = (e: MessageEvent<WorkerResponse>) => {
    const { id } = e.data;
    const p = pending.get(id);
    if (!p) return;
    if ("streamEvents" in e.data) {
      // Partial batch of events during a streaming call — forward to listener,
      // keep pending open until the final response arrives.
      if ((globalThis as { __TRACE_LOGS?: boolean }).__TRACE_LOGS === true) {
        const counts: Record<string, number> = {};
        for (const evt of e.data.streamEvents) {
          const p2 = (evt as { phase?: string }).phase ?? "?";
          counts[p2] = (counts[p2] ?? 0) + 1;
        }
        // eslint-disable-next-line no-console
        console.log(
          `[main  t=${performance.now().toFixed(0)}ms] arrived ${e.data.streamEvents.length}:`,
          counts,
        );
      }
      if (p.onEvent) {
        for (const evt of e.data.streamEvents) p.onEvent(evt as TraceEvent);
      }
      return;
    }
    pending.delete(id);
    if (e.data.ok) p.resolve(e.data.result);
    else p.reject(new Error(e.data.error));
  };
  worker.onerror = (e) => {
    console.error("[engine.worker] error", e);
  };

  await call<null>({ method: "init" });
  itemsCache = await call<string[]>({ method: "allProducibleItems" });
  machinesCache = await call<string[]>({ method: "allProducerMachines" });
  const defaults = await call<[string, string][]>({
    method: "defaultMachinesForItems",
    items: itemsCache,
    fallback: "assembling-machine-3",
  });
  defaultMachineCache = new Map(defaults);
}

async function solve(
  targetItem: string,
  targetRate: number,
  availableInputs: string[],
  machineEntity: string,
): Promise<SolverResult> {
  // If a streaming layout is in flight, the user has just typed a new target
  // and is waiting for feedback — kill the old WASM work so solve isn't
  // queued behind a slow layout that's about to be thrown away anyway.
  if (activeStreamingId !== null) await supersedeWorker();
  return call<SolverResult>({
    method: "solve",
    targetItem,
    targetRate,
    availableInputs,
    machineEntity,
  });
}

function allProducibleItems(): string[] {
  return itemsCache;
}

function allProducerMachines(): string[] {
  return machinesCache;
}

function buildLayout(result: SolverResult, maxBeltTier?: string): Promise<LayoutResult> {
  return call<LayoutResult>({ method: "layout", result, maxBeltTier: maxBeltTier ?? null });
}

function buildLayoutTraced(result: SolverResult, maxBeltTier?: string): Promise<LayoutResult> {
  return call<LayoutResult>({ method: "layoutTraced", result, maxBeltTier: maxBeltTier ?? null });
}

/**
 * Kill the current worker and respawn a fresh one. Rejects all pending
 * promises so stale callers see the supersession. Used to cancel an
 * in-flight streaming layout when the user triggers a new solve.
 */
async function supersedeWorker(): Promise<void> {
  if (!worker) return;
  worker.terminate();
  worker = null;
  const superseded = new Error("Engine superseded by a newer request");
  for (const [, p] of pending) p.reject(superseded);
  pending.clear();
  activeStreamingId = null;
  await initEngine();
}

async function buildLayoutStreaming(
  result: SolverResult,
  maxBeltTier: string | undefined,
  onEvent: (evt: TraceEvent) => void,
): Promise<LayoutResult> {
  if (activeStreamingId !== null) {
    await supersedeWorker();
  }
  const id = ++nextId;
  activeStreamingId = id;
  onActive(+1);
  return new Promise<LayoutResult>((resolve, reject) => {
    pending.set(id, {
      resolve: (v) => {
        onActive(-1);
        if (activeStreamingId === id) activeStreamingId = null;
        resolve(v as LayoutResult);
      },
      reject: (e) => {
        onActive(-1);
        if (activeStreamingId === id) activeStreamingId = null;
        reject(e);
      },
      onEvent,
    });
    const traceLogs =
      (globalThis as { __TRACE_LOGS?: boolean }).__TRACE_LOGS === true;
    worker!.postMessage({
      id,
      method: "layoutStreaming",
      result,
      maxBeltTier: maxBeltTier ?? null,
      traceLogs,
    });
  });
}

function exportBlueprint(layout: LayoutResult, label: string): Promise<string> {
  return call<string>({ method: "exportBlueprint", layout, label });
}

function defaultMachineForItem(item: string, fallback: string): string {
  return defaultMachineCache.get(item) ?? fallback;
}

function validateLayout(
  layout: LayoutResult,
  solverResult: SolverResult | null,
): Promise<ValidationIssue[]> {
  return call<ValidationIssue[]>({ method: "validateLayout", layout, solverResult });
}

/**
 * Solve a SAT-zone fixture, optionally pinning a set of painted
 * entities as solver assumptions. Resolves to `null` when the solver
 * returns UNSAT or when any pin was rejected (out of bounds, on a
 * forbidden tile, unsupported entity, item not in the fixture).
 *
 * Used by the F2 SAT-zone editor to drive the live validity indicator
 * and the ghost-completion overlay (`entities \ pins`).
 */
function solveFixture(
  fixtureJson: string,
  pins: PlacedEntity[],
): Promise<SolveFixtureResult | null> {
  return call<SolveFixtureResult | null>({
    method: "solveFixture",
    fixtureJson,
    pinsJson: JSON.stringify(pins),
  });
}

export function parseBlueprint(bpString: string): Promise<LayoutResult> {
  return call<LayoutResult>({ method: "parseBlueprint", bp: bpString });
}

/**
 * One improvement step streamed out of the solver during `improveRegion`.
 * Mirrors `TraceEvent::SatImprovement` in Rust — the `region_id`,
 * `zone_x/y/w/h`, `cost`, `iter`, `solve_time_us` and pruned `entities`
 * for this descent step.
 */
export interface SatImprovement {
  region_id: number;
  zone_x: number;
  zone_y: number;
  zone_w: number;
  zone_h: number;
  cost: number;
  iter: number;
  solve_time_us: number;
  entities: PlacedEntity[];
}

/**
 * Run a long cost-descent pass on a single SAT crossing zone. The
 * promise resolves with the final `LayoutResult` (with the zone's
 * entities replaced by the best layout found). `onImprovement` fires
 * once per strictly-cheaper solve, including the initial snapshot at
 * `iter=0`, so the UI can animate the descent.
 *
 * `budgetMs` — wall-clock cap, clamped to [100, 60_000] server-side.
 * Typical UI call passes 10_000.
 *
 * `maxIters` — cap on descent steps. 0 means "unbounded" (server side
 * falls back to 1024). The round-robin `optimizeAllRegions` driver
 * passes 1 so each visit takes at most one improvement step.
 */
async function improveRegion(
  layoutIn: LayoutResult,
  regionId: number,
  budgetMs: number,
  onImprovement: (imp: SatImprovement) => void,
  maxIters: number = 0,
): Promise<LayoutResult> {
  if (activeStreamingId !== null) {
    await supersedeWorker();
  }
  if (!worker) throw new Error("Engine not initialized");
  const id = ++nextId;
  activeStreamingId = id;
  onActive(+1);
  return new Promise<LayoutResult>((resolve, reject) => {
    pending.set(id, {
      resolve: (v) => {
        onActive(-1);
        if (activeStreamingId === id) activeStreamingId = null;
        resolve(v as LayoutResult);
      },
      reject: (e) => {
        onActive(-1);
        if (activeStreamingId === id) activeStreamingId = null;
        reject(e);
      },
      onEvent: (evt) => {
        const anyEvt = evt as unknown as { phase?: string; data?: SatImprovement };
        if (anyEvt.phase === "SatImprovement" && anyEvt.data) {
          onImprovement(anyEvt.data);
        }
      },
    });
    worker!.postMessage({
      id,
      method: "improveRegionStreaming",
      layout: layoutIn,
      regionId,
      budgetMs,
      maxIters,
    });
  });
}

/**
 * Callbacks for `optimizeAllRegions`. Each is optional.
 *
 * - `onImprovement(imp)` — fires on every `SatImprovement` event
 *   streamed by `descend`. `imp.iter === 0` carries the region's
 *   starting entities (not a cost drop); filter to `iter > 0` for real
 *   improvements. Use this to feed an animation queue.
 */
export interface OptimizeAllOpts {
  perRegionBudgetMs: number;
  onImprovement?: (imp: SatImprovement) => void;
}

/**
 * Round-robin "Optimize all" over every `CrossingZone` region. Each
 * visit is exactly one SAT probe (`max_iters=1`) — that keeps any
 * single call bounded, since varisat can't be interrupted mid-solve and
 * tight cost caps can take tens of seconds on hard instances. Regions
 * that produce no improvement this round drop out; rounds stop when
 * the set is empty or a full pass yields zero improvements. The UI
 * paces the visual updates via its own queue, so the caller sees the
 * event stream but nothing round-specific.
 *
 * Returns the final `LayoutResult`. Rejects with "Engine superseded"
 * if cancelled via `cancelInFlight`.
 */
async function optimizeAllRegions(
  layoutIn: LayoutResult,
  opts: OptimizeAllOpts,
): Promise<LayoutResult> {
  let current = layoutIn;
  const active = new Set<number>(
    (current.regions ?? [])
      .filter((r) => (r as { kind?: string }).kind === "crossing_zone")
      .map((r) => (r as { id: number }).id),
  );
  while (active.size > 0) {
    let improvedThisRound = 0;
    const visitOrder = [...active].sort((a, b) => a - b);
    for (const regionId of visitOrder) {
      if (!active.has(regionId)) continue;
      let sawImprovement = false;
      current = await improveRegion(
        current,
        regionId,
        opts.perRegionBudgetMs,
        (imp) => {
          if (imp.iter > 0) sawImprovement = true;
          opts.onImprovement?.(imp);
        },
        1,
      );
      if (sawImprovement) improvedThisRound += 1;
      else active.delete(regionId);
    }
    if (improvedThisRound === 0) break;
  }
  return current;
}

/** Cancel any in-flight improveRegion / layoutStreaming by respawning the worker. */
export async function cancelInFlight(): Promise<void> {
  if (activeStreamingId !== null) {
    await supersedeWorker();
  }
}

export type Engine = {
  solve: typeof solve;
  allProducibleItems: typeof allProducibleItems;
  allProducerMachines: typeof allProducerMachines;
  buildLayout: typeof buildLayout;
  buildLayoutTraced: typeof buildLayoutTraced;
  buildLayoutStreaming: typeof buildLayoutStreaming;
  exportBlueprint: typeof exportBlueprint;
  defaultMachineForItem: typeof defaultMachineForItem;
  validateLayout: typeof validateLayout;
  solveFixture: typeof solveFixture;
  improveRegion: typeof improveRegion;
  optimizeAllRegions: typeof optimizeAllRegions;
};

export function getEngine(): Engine {
  return {
    solve,
    allProducibleItems,
    allProducerMachines,
    buildLayout,
    buildLayoutTraced,
    buildLayoutStreaming,
    exportBlueprint,
    defaultMachineForItem,
    validateLayout,
    solveFixture,
    improveRegion,
    optimizeAllRegions,
  };
}
