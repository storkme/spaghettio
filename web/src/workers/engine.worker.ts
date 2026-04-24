import wasmInit, {
  init,
  solve,
  solve_fixture,
  all_producible_items,
  all_producer_machines,
  default_machine_for_item,
  export_blueprint,
  improve_region_streaming,
  layout,
  layout_traced,
  layout_streaming,
  parse_blueprint,
  validate_layout,
} from "../wasm-pkg/fucktorio_wasm.js";
import type {
  SolverResult,
  LayoutResult,
} from "../wasm-pkg/fucktorio_wasm.js";

type Request =
  | { id: number; method: "init" }
  | { id: number; method: "allProducibleItems" }
  | { id: number; method: "allProducerMachines" }
  | { id: number; method: "defaultMachinesForItems"; items: string[]; fallback: string }
  | {
      id: number;
      method: "solve";
      targetItem: string;
      targetRate: number;
      availableInputs: string[];
      machineEntity: string;
    }
  | { id: number; method: "layout"; result: SolverResult; maxBeltTier: string | null }
  | { id: number; method: "layoutTraced"; result: SolverResult; maxBeltTier: string | null }
  | { id: number; method: "layoutStreaming"; result: SolverResult; maxBeltTier: string | null }
  | { id: number; method: "exportBlueprint"; layout: LayoutResult; label: string }
  | {
      id: number;
      method: "validateLayout";
      layout: LayoutResult;
      solverResult: SolverResult | null;
    }
  | { id: number; method: "parseBlueprint"; bp: string }
  | { id: number; method: "solveFixture"; fixtureJson: string; pinsJson: string }
  | {
      id: number;
      method: "improveRegionStreaming";
      layout: LayoutResult;
      regionId: number;
      budgetMs: number;
      maxIters: number;
    };

let ready: Promise<void> | null = null;

function ensureReady(): Promise<void> {
  if (!ready) {
    ready = (async () => {
      await wasmInit();
      init();
    })();
  }
  return ready;
}

function post(id: number, ok: boolean, value: unknown): void {
  if (ok) {
    (self as unknown as Worker).postMessage({ id, ok: true, result: value });
  } else {
    (self as unknown as Worker).postMessage({ id, ok: false, error: value });
  }
}

self.onmessage = async (e: MessageEvent<Request>) => {
  const req = e.data;
  try {
    await ensureReady();
    let result: unknown;
    switch (req.method) {
      case "init":
        result = null;
        break;
      case "allProducibleItems":
        result = all_producible_items();
        break;
      case "allProducerMachines":
        result = all_producer_machines();
        break;
      case "defaultMachinesForItems": {
        const out: [string, string][] = [];
        for (const item of req.items) {
          out.push([item, default_machine_for_item(item, req.fallback)]);
        }
        result = out;
        break;
      }
      case "solve":
        result = solve(req.targetItem, req.targetRate, req.availableInputs, req.machineEntity);
        break;
      case "layout":
        result = layout(req.result, req.maxBeltTier ?? undefined);
        break;
      case "layoutTraced":
        result = layout_traced(req.result, req.maxBeltTier ?? undefined);
        break;
      case "layoutStreaming": {
        const id = req.id;
        // Batch events before postMessage. One postMessage per event
        // would over-saturate the main thread with structured-clone
        // overhead; batches amortise that cost. But too-large batches
        // hide the stream entirely: during junction solving the engine
        // emits ~10 events/sec, so BATCH_SIZE=64 would never auto-flush
        // during the 5-6s junction phase and every mid-run event would
        // land at the end. 8 keeps per-message overhead reasonable
        // while still flushing every ~800ms during slow phases.
        const BATCH_SIZE = 8;
        const TRACE_LOGS = (req as { traceLogs?: boolean }).traceLogs === true;
        const t0 = performance.now();
        let batch: unknown[] = [];
        let totalEmitted = 0;
        let totalFlushed = 0;
        const flushBatch = (): void => {
          if (batch.length === 0) return;
          totalFlushed += batch.length;
          if (TRACE_LOGS) {
            // Breakdown by phase so we can see *what* is streaming.
            const counts: Record<string, number> = {};
            for (const e of batch) {
              const p = (e as { phase?: string }).phase ?? "?";
              counts[p] = (counts[p] ?? 0) + 1;
            }
            // eslint-disable-next-line no-console
            console.log(
              `[worker t+${(performance.now() - t0).toFixed(0)}ms] flush ${batch.length} (total ${totalFlushed}):`,
              counts,
            );
          }
          (self as unknown as Worker).postMessage({ id, streamEvents: batch });
          batch = [];
        };
        const emit = (evt: unknown): void => {
          batch.push(evt);
          totalEmitted++;
          if (batch.length >= BATCH_SIZE) flushBatch();
        };
        try {
          result = layout_streaming(req.result, req.maxBeltTier ?? undefined, emit);
        } finally {
          flushBatch();
          if (TRACE_LOGS) {
            // eslint-disable-next-line no-console
            console.log(
              `[worker t+${(performance.now() - t0).toFixed(0)}ms] layout_streaming done. emitted=${totalEmitted} flushed=${totalFlushed}`,
            );
          }
        }
        break;
      }
      case "exportBlueprint":
        result = export_blueprint(req.layout, req.label);
        break;
      case "validateLayout":
        result = validate_layout(req.layout, req.solverResult ?? undefined, "Bus");
        break;
      case "parseBlueprint":
        result = parse_blueprint(req.bp);
        break;
      case "solveFixture":
        result = solve_fixture(req.fixtureJson, req.pinsJson);
        break;
      case "improveRegionStreaming": {
        // Streams SatImprovement events through the same batched channel
        // as layout_streaming. Each event carries the pruned entity list
        // for the zone at that descent step and the raw cost — the
        // frontend animates diffs against the previous event.
        const id = req.id;
        const BATCH_SIZE = 1;
        let batch: unknown[] = [];
        const flushBatch = (): void => {
          if (batch.length === 0) return;
          (self as unknown as Worker).postMessage({ id, streamEvents: batch });
          batch = [];
        };
        const emit = (evt: unknown): void => {
          batch.push(evt);
          if (batch.length >= BATCH_SIZE) flushBatch();
        };
        try {
          result = improve_region_streaming(
            req.layout,
            req.regionId,
            req.budgetMs,
            req.maxIters,
            emit,
          );
        } finally {
          flushBatch();
        }
        break;
      }
    }
    post(req.id, true, result);
  } catch (err) {
    post(req.id, false, err instanceof Error ? err.message : String(err));
  }
};
