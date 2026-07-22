/**
 * Sim report loader — parses the JSON report emitted by
 * `spaghettio-sim run --out report.json` (RFC-050 Phase 4).
 *
 * Unlike `.fls` layout snapshots (`snapshotLoader.ts`), the sim report is
 * plain JSON with no wire-format envelope — the harness is a separate
 * offline Rust binary, not something the web app round-trips. Shape is
 * validated defensively at load time: a malformed or unrelated JSON file
 * must surface a readable error, never a crash or a silently-blank
 * overlay. See `docs/rfc-050-headless-sim-harness.md` ("The report" +
 * "Sim-state debug tooling" sections) for the field origins.
 */

// ---------------------------------------------------------------------------
// Types (mirror the harness's `Report` + sim-state dump JSON — Rust side
// lives in `crates/sim-harness`)
// ---------------------------------------------------------------------------

export interface SimReportItem {
  item: string;
  is_target: boolean;
  planned_rate: number;
  measured_produced_rate: number | null;
  measured_delivered_rate: number | null;
  delta_pct_produced: number | null;
  delta_pct_delivered: number | null;
  verdict: "PASS" | "WARN" | "FAIL" | null;
}

export interface SimReportSummary {
  overall_verdict: string;
  label: string;
  entities: number;
  ghosts: number;
  revived: number;
  import_rc: number;
  converged: boolean;
  items: SimReportItem[];
  machine_census: Record<string, number>;
  external_inputs: [item: string, rate: number, fluid: boolean][];
  pole_networks: number;
  proxies_fulfilled: number;
  factory_eeis: number;
  fluid_fed: boolean;
  fluid_errors: Record<string, unknown>;
  stacking: number;
  inserter_capacity: number;
  uncalibrated_direction: boolean;
  final_tick: number;
}

export interface SimRunParams {
  scenario_name: string;
  speed: number;
  warmup_ticks: number;
  window_ticks: number;
  end_tick: number;
}

/** `[x, y, count]` — layout-space tile coordinate + item tally. Not
 *  strictly per-tile-capacity bounded (see `simStateOverlay.ts`). */
export type SimBeltEntry = [x: number, y: number, count: number];
/** `[x, y, name, status]` — `status` is the raw in-game `LuaEntity.status`
 *  name (`working`, `item_ingredient_shortage`, `no_power`, ...). */
export type SimMachineEntry = [x: number, y: number, name: string, status: string];
/** `[x, y, status]` — same status vocabulary, inserter-relevant subset. */
export type SimInserterEntry = [x: number, y: number, status: string];

export interface SimState {
  belts: SimBeltEntry[];
  machines: SimMachineEntry[];
  inserters: SimInserterEntry[];
  /** World→layout offset the harness derived from the revived bbox min.
   *  `belts`/`machines`/`inserters` coordinates are already layout-space
   *  (offset applied); kept here for provenance/debugging, not needed to
   *  render the overlay. */
  offx: number;
  offy: number;
  fed?: Record<string, number>;
}

export interface SimReport {
  game_version: string;
  run_params: SimRunParams;
  report: SimReportSummary;
  sim_state: SimState;
  /** Per-window checkpoint samples + census — opaque to the overlay,
   *  kept for anyone inspecting the raw file. */
  raw_result: unknown;
}

// ---------------------------------------------------------------------------
// Defensive parsing
// ---------------------------------------------------------------------------

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function isTupleArray(v: unknown, minLen: number): v is unknown[][] {
  return Array.isArray(v) && v.every((e) => Array.isArray(e) && e.length >= minLen);
}

/**
 * Parse + validate a sim report JSON string. Throws a descriptive `Error`
 * (never a raw `SyntaxError`/`TypeError`) on anything that doesn't look
 * like a report — the caller is expected to display `err.message` rather
 * than crash.
 */
export function parseSimReport(jsonText: string): SimReport {
  let raw: unknown;
  try {
    raw = JSON.parse(jsonText);
  } catch {
    throw new Error("Not valid JSON.");
  }

  if (!isPlainObject(raw)) {
    throw new Error("Expected a JSON object at the top level.");
  }

  const problems: string[] = [];

  if (typeof raw.game_version !== "string") problems.push("missing `game_version` (string)");

  const report = raw.report;
  if (!isPlainObject(report)) {
    problems.push("missing `report` object");
  } else {
    if (typeof report.overall_verdict !== "string") problems.push("`report.overall_verdict` missing");
    if (typeof report.entities !== "number") problems.push("`report.entities` missing");
    if (!Array.isArray(report.items)) problems.push("`report.items` is not an array");
    else {
      for (const [i, item] of report.items.entries()) {
        if (!isPlainObject(item) || typeof item.item !== "string" || typeof item.planned_rate !== "number") {
          problems.push(`\`report.items[${i}]\` is malformed`);
          break;
        }
      }
    }
  }

  const runParams = raw.run_params;
  if (!isPlainObject(runParams)) {
    problems.push("missing `run_params` object");
  }

  const simState = raw.sim_state;
  if (!isPlainObject(simState)) {
    problems.push("missing `sim_state` object");
  } else {
    if (!isTupleArray(simState.belts, 3)) problems.push("`sim_state.belts` is not an array of [x,y,count]");
    if (!isTupleArray(simState.machines, 4)) problems.push("`sim_state.machines` is not an array of [x,y,name,status]");
    if (!isTupleArray(simState.inserters, 3)) problems.push("`sim_state.inserters` is not an array of [x,y,status]");
    if (typeof simState.offx !== "number" || typeof simState.offy !== "number") {
      problems.push("`sim_state.offx`/`offy` missing");
    }
  }

  if (problems.length > 0) {
    throw new Error(`Not a valid sim report: ${problems.join("; ")}.`);
  }

  return raw as unknown as SimReport;
}
