export interface FormState {
  item: string;
  rate: number;
  /** null means "no machine in URL — caller should derive from item". */
  machine: string | null;
  inputs: string[];
  /** Max belt tier override, e.g. "transport-belt". null = auto. */
  belt: string | null;
  /** Layout strategy ("partitioned-decomposed").
   * null = pooled (today's default). See `docs/rfp-modular-production.md`.
   *
   * The legacy `"partitioned-per-consumer"` URL value is still accepted
   * on read (back-compat for bookmarked URLs) and normalised to
   * `"partitioned-decomposed"` here — the P1 strategy was strictly
   * dominated by P2 across the diag corpus and was hard-deleted. */
  strategy: string | null;
  /** Row layout ("horizontal-stack"). null = vertical-split (today's default).
   * See `docs/rfp-horizontal-trunks.md`. */
  rowLayout: string | null;
  /** User-added inputs beyond the DEFAULT_INPUTS list. */
  customInputs: string[];
}

/** Strategy values accepted on the URL and in `FormState.strategy`.
 *
 * `"partitioned-per-consumer"` is the legacy P1 alias — accepted on
 * read (bookmarked URLs continue to work) and normalised to
 * `"partitioned-decomposed"` by `readUrlState`. The P1 enum variant
 * was hard-deleted because it was strictly dominated by P2 across the
 * diag corpus. */
export const KNOWN_STRATEGIES = ["partitioned-per-consumer", "partitioned-decomposed"] as const;

/** Row-layout values accepted on the URL and in `FormState.rowLayout`. */
export const KNOWN_ROW_LAYOUTS = ["horizontal-stack"] as const;

/** Full list of input pills rendered in the sidebar. */
export const DEFAULT_INPUTS: string[] = [
  "iron-plate",
  "copper-plate",
  "steel-plate",
  "stone",
  "coal",
  "water",
  "crude-oil",
  "iron-ore",
  "copper-ore",
];

/** Subset that are checked by default. Plates are left unchecked so
 * the default view is a "from ore" starting point. */
export const DEFAULT_CHECKED_INPUTS: string[] = [
  "stone",
  "coal",
  "water",
  "crude-oil",
  "iron-ore",
  "copper-ore",
];

export const DEFAULT_ITEM = "iron-gear-wheel";
export const DEFAULT_RATE = 10;
export const DEFAULT_MACHINE = "assembling-machine-3";

export function readUrlState(): FormState {
  const params = new URLSearchParams(window.location.search);

  const item = params.get("item") ?? DEFAULT_ITEM;
  const rawRate = parseFloat(params.get("rate") ?? "");
  const rate = isNaN(rawRate) || rawRate <= 0 ? DEFAULT_RATE : rawRate;
  const machine = params.get("machine");
  const inParam = params.get("in");
  const inputs = inParam ? inParam.split(",").filter((s) => s.length > 0) : DEFAULT_CHECKED_INPUTS;
  const belt = params.get("belt");
  const rawStrategy = params.get("strategy");
  let strategy = rawStrategy && (KNOWN_STRATEGIES as readonly string[]).includes(rawStrategy) ? rawStrategy : null;
  // Normalise the legacy P1 alias to the surviving P2 string. Keeps
  // bookmarked `?strategy=partitioned-per-consumer` URLs working
  // without surfacing the deprecated value back into UI / WASM.
  if (strategy === "partitioned-per-consumer") strategy = "partitioned-decomposed";
  const rawRowLayout = params.get("row_layout");
  const rowLayout = rawRowLayout && (KNOWN_ROW_LAYOUTS as readonly string[]).includes(rawRowLayout) ? rawRowLayout : null;
  const ciParam = params.get("ci");
  const customInputs = ciParam ? ciParam.split(",").filter((s) => s.length > 0) : [];

  return { item, rate, machine, inputs, belt, strategy, rowLayout, customInputs };
}

export function writeUrlState(state: Omit<FormState, "machine"> & { machine: string }): void {
  const isDefault =
    state.item === DEFAULT_ITEM &&
    state.rate === DEFAULT_RATE &&
    state.machine === DEFAULT_MACHINE &&
    state.inputs.length === DEFAULT_CHECKED_INPUTS.length &&
    state.inputs.every((v, i) => v === DEFAULT_CHECKED_INPUTS[i]) &&
    !state.belt &&
    !state.strategy &&
    !state.rowLayout &&
    state.customInputs.length === 0;

  if (isDefault) {
    history.replaceState(null, "", window.location.pathname);
    return;
  }

  const params = new URLSearchParams();
  params.set("item", state.item);
  params.set("rate", String(state.rate));
  params.set("machine", state.machine);
  params.set("in", state.inputs.join(","));
  if (state.belt) params.set("belt", state.belt);
  if (state.strategy) params.set("strategy", state.strategy);
  if (state.rowLayout) params.set("row_layout", state.rowLayout);
  if (state.customInputs.length > 0) params.set("ci", state.customInputs.join(","));
  history.replaceState(null, "", "?" + params.toString());
}
