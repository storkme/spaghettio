import { shortIdForSlug, slugForShortId } from "./shortIds.js";

export interface FormState {
  item: string;
  rate: number;
  /** Per-recipe-category machine override. Sparse — categories absent here
   * fall through to the hardcoded mapping in the Rust solver, which itself
   * falls through to `DEFAULT_MACHINES.crafting`. URL keys are listed in
   * `URL_KEY_BY_CATEGORY`; the legacy `?machine=` param (single-string
   * AM tier) is migrated into `machines.crafting` on read. */
  machines: Record<string, string>;
  inputs: string[];
  /** Max belt tier override, e.g. "transport-belt". null = auto. */
  belt: string | null;
  /** Layout strategy ("partitioned-decomposed").
   * null = pooled (today's default). See `docs/rfc-modular-production.md`.
   *
   * The legacy `"partitioned-per-consumer"` URL value is still accepted
   * on read (back-compat for bookmarked URLs) and normalised to
   * `"partitioned-decomposed"` here — the P1 strategy was strictly
   * dominated by P2 across the diag corpus and was hard-deleted. */
  strategy: string | null;
  /** Row layout ("horizontal-stack"). null = vertical-split (today's default).
   * See `docs/rfc-horizontal-trunks.md`. */
  rowLayout: string | null;
  /** Max inserter tier override ("regular" | "fast"). null = stack
   * (today's default) — mirrors `belt`'s "null = auto" semantics, except
   * the un-set value IS the richest tier here (there's no cheaper
   * "auto"), so null means "no cap". See `docs/rfc-inserter-sizing.md`. */
  inserterTier: string | null;
  /** Build quality of placed entities ("uncommon" | "rare" | "epic" |
   * "legendary"). null = normal (today's default) — same null-is-default
   * convention as `inserterTier`. See `docs/rfc-build-quality.md`. */
  quality: string | null;
  /** Pole wiring mode ("tree"). null = dense (today's default) — same
   * null-is-default convention as `quality`. See RFC-045. */
  wireMode: string | null;
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

/** Inserter-tier cap values accepted on the URL and in
 * `FormState.inserterTier`. `"stack"` (the default) is intentionally
 * absent — same convention as `KNOWN_STRATEGIES` omitting "pooled": the
 * default is represented by `null`, never an explicit value. */
export const KNOWN_INSERTER_TIERS = ["regular", "fast"] as const;

/** Build-quality values accepted on the URL and in `FormState.quality`.
 * `"normal"` (the default) is intentionally absent — represented by
 * `null`, same convention as `KNOWN_INSERTER_TIERS`. */
export const KNOWN_QUALITIES = ["uncommon", "rare", "epic", "legendary"] as const;

/** Wire-mode values accepted on the URL and in `FormState.wireMode`.
 * `"dense"` (the default) is intentionally absent — represented by
 * `null`, same convention as `KNOWN_QUALITIES`. */
export const KNOWN_WIRE_MODES = ["tree"] as const;

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
/** Per-category default machine. Read by the sidebar to populate selectors
 * and by the engine to fill the `default_machine` argument. The crafting
 * entry doubles as the "fall-through" machine for any category not present
 * in the palette and not hardcoded in `recipe_db::machine_for_recipe`. */
export const DEFAULT_MACHINES: Record<string, string> = {
  crafting: "assembling-machine-3",
  smelting: "electric-furnace",
};

/** Categories the user can edit in the URL, with their short URL keys. */
export const URL_KEY_BY_CATEGORY: Record<string, string> = {
  crafting: "craft",
  smelting: "smelt",
};

/** Back-compat alias: the legacy single-machine URL parameter used to
 * configure the assembler tier. Read into `machines.crafting`. */
const LEGACY_MACHINE_KEY = "machine";

// Hash-form (Bucket B) URL scheme:
//
//   #/l/<item>/<rate>/<machine>/<inputs>/<belt>?<extras>
//
// Each path slot uses short codes from `shortIds.ts` (or `_` to mean "use
// default"). Inputs are `,`-separated. The `?<extras>` segment is optional
// and carries less-common params (`s=` strategy, `rl=` row layout, `ci=`
// custom inputs). `,` is safe in both URL paths and form-encoded query
// strings, so the same separator works in both places — `+` would be
// silently rewritten to a space when extras are parsed via
// URLSearchParams, breaking hand-edited URLs.
//
// Both this scheme and the legacy `?item=...&rate=...&...` query string
// are accepted on read for at least one release; new URLs are always
// written in hash form.
const HASH_PREFIX = "#/l/";
const SKIP_TOKEN = "_";
const INPUT_SEPARATOR = ",";

const STRATEGY_SHORT_TO_FULL: Record<string, string> = {
  pd: "partitioned-decomposed",
};
const STRATEGY_FULL_TO_SHORT: Record<string, string> = {
  "partitioned-decomposed": "pd",
};

const ROW_LAYOUT_SHORT_TO_FULL: Record<string, string> = {
  hs: "horizontal-stack",
};
const ROW_LAYOUT_FULL_TO_SHORT: Record<string, string> = {
  "horizontal-stack": "hs",
};

const INSERTER_TIER_SHORT_TO_FULL: Record<string, string> = {
  r: "regular",
  f: "fast",
};
const INSERTER_TIER_FULL_TO_SHORT: Record<string, string> = {
  regular: "r",
  fast: "f",
};

const QUALITY_SHORT_TO_FULL: Record<string, string> = {
  u: "uncommon",
  r: "rare",
  e: "epic",
  l: "legendary",
};
const QUALITY_FULL_TO_SHORT: Record<string, string> = {
  uncommon: "u",
  rare: "r",
  epic: "e",
  legendary: "l",
};

const WIRE_MODE_SHORT_TO_FULL: Record<string, string> = { t: "tree" };
const WIRE_MODE_FULL_TO_SHORT: Record<string, string> = { tree: "t" };

function slugToCode(slug: string): string {
  // Fall back to the slug itself if it's not in the table — keeps
  // serialization total (e.g. an unknown / modded item still produces a
  // usable URL, just a longer one). The decoder accepts known codes only;
  // an unknown token in the URL means we bail back to the legacy parser.
  return shortIdForSlug(slug) ?? slug;
}

/** Decode a single short-code path token. Returns null for unrecognised
 * tokens; callers treat that as "this hash form is malformed, fall back
 * to the legacy parser" rather than silently inventing a slug. */
function codeToSlug(code: string): string | null {
  return slugForShortId(code);
}

function readHashState(): FormState | null {
  const hash = window.location.hash;
  if (!hash.startsWith(HASH_PREFIX)) return null;

  // Split off the `?extras` portion, if present.
  const rest = hash.slice(HASH_PREFIX.length);
  const qIdx = rest.indexOf("?");
  const path = qIdx >= 0 ? rest.slice(0, qIdx) : rest;
  const extrasStr = qIdx >= 0 ? rest.slice(qIdx + 1) : "";

  // Path: <item>/<rate>/<machine>/<inputs>/<belt>. Trailing slots may be
  // omitted by truncation (e.g. just `#/l/ipr/5`); missing slots fall back
  // to defaults. Empty intermediate slots = SKIP_TOKEN = use default.
  const parts = path.split("/");
  const get = (i: number): string | null => {
    const v = parts[i];
    if (v === undefined || v === "" || v === SKIP_TOKEN) return null;
    return v;
  };

  // Decode every code-bearing slot. Unknown codes return null — we bail
  // out of the hash parser and let the caller fall back to the legacy
  // query-string form (or defaults), rather than treating gibberish as a
  // literal slug.
  const itemCode = get(0);
  let item: string;
  if (itemCode) {
    const decoded = codeToSlug(itemCode);
    if (decoded === null) return null;
    item = decoded;
  } else {
    item = DEFAULT_ITEM;
  }

  const rateRaw = get(1);
  const rateParsed = rateRaw !== null ? parseFloat(rateRaw) : NaN;
  const rate = !isNaN(rateParsed) && rateParsed > 0 ? rateParsed : DEFAULT_RATE;

  const machineCode = get(2);
  const machines: Record<string, string> = {};
  if (machineCode) {
    const decoded = codeToSlug(machineCode);
    if (decoded === null) return null;
    // Slot 2 is the crafting tier — the canonical "the user changed the
    // assembler" knob. Other categories ride in extras (see below).
    machines.crafting = decoded;
  }

  const inputsRaw = get(3);
  let inputs: string[];
  if (inputsRaw) {
    const tokens = inputsRaw.split(INPUT_SEPARATOR).filter((s) => s.length > 0);
    const decoded: string[] = [];
    for (const t of tokens) {
      const slug = codeToSlug(t);
      if (slug === null) return null;
      decoded.push(slug);
    }
    inputs = decoded;
  } else {
    inputs = DEFAULT_CHECKED_INPUTS;
  }

  const beltCode = get(4);
  let belt: string | null = null;
  if (beltCode) {
    belt = codeToSlug(beltCode);
    if (belt === null) return null;
  }

  const extras = new URLSearchParams(extrasStr);
  const sShort = extras.get("s");
  const strategy = sShort ? STRATEGY_SHORT_TO_FULL[sShort] ?? null : null;
  const rlShort = extras.get("rl");
  const rowLayout = rlShort ? ROW_LAYOUT_SHORT_TO_FULL[rlShort] ?? null : null;
  const itShort = extras.get("it");
  const inserterTier = itShort ? INSERTER_TIER_SHORT_TO_FULL[itShort] ?? null : null;
  const qShort = extras.get("q");
  const quality = qShort ? QUALITY_SHORT_TO_FULL[qShort] ?? null : null;
  const wShort = extras.get("w");
  const wireMode = wShort ? WIRE_MODE_SHORT_TO_FULL[wShort] ?? null : null;
  const ciRaw = extras.get("ci");
  let customInputs: string[] = [];
  if (ciRaw) {
    const tokens = ciRaw.split(INPUT_SEPARATOR).filter((s) => s.length > 0);
    for (const t of tokens) {
      const slug = codeToSlug(t);
      if (slug === null) return null;
      customInputs.push(slug);
    }
  }

  // Per-category machines other than crafting (which lives in slot 2)
  // ride as `<urlKey>=<short-code>` extras — smelting today, plus any
  // future category in `URL_KEY_BY_CATEGORY`.
  for (const [category, urlKey] of Object.entries(URL_KEY_BY_CATEGORY)) {
    if (category === "crafting") continue;
    const code = extras.get(urlKey);
    if (!code) continue;
    const slug = codeToSlug(code);
    if (slug === null) return null;
    machines[category] = slug;
  }

  return { item, rate, machines, inputs, belt, strategy, rowLayout, inserterTier, quality, wireMode, customInputs };
}

function readQueryState(): FormState {
  const params = new URLSearchParams(window.location.search);

  const item = params.get("item") ?? DEFAULT_ITEM;
  const rawRate = parseFloat(params.get("rate") ?? "");
  const rate = isNaN(rawRate) || rawRate <= 0 ? DEFAULT_RATE : rawRate;

  // Per-category machine palette. Read each editable category's URL key,
  // then fold in the legacy `?machine=` value as `machines.crafting` if
  // the new `?craft=` key is absent.
  const machines: Record<string, string> = {};
  for (const [category, urlKey] of Object.entries(URL_KEY_BY_CATEGORY)) {
    const value = params.get(urlKey);
    if (value) machines[category] = value;
  }
  const legacy = params.get(LEGACY_MACHINE_KEY);
  if (legacy && !machines.crafting) machines.crafting = legacy;

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
  const rawInserterTier = params.get("inserter_tier");
  const inserterTier =
    rawInserterTier && (KNOWN_INSERTER_TIERS as readonly string[]).includes(rawInserterTier)
      ? rawInserterTier
      : null;
  const rawWireMode = params.get("wire_mode");
  const wireMode =
    rawWireMode && (KNOWN_WIRE_MODES as readonly string[]).includes(rawWireMode)
      ? rawWireMode
      : null;
  const rawQuality = params.get("quality");
  const quality =
    rawQuality && (KNOWN_QUALITIES as readonly string[]).includes(rawQuality)
      ? rawQuality
      : null;
  const ciParam = params.get("ci");
  const customInputs = ciParam ? ciParam.split(",").filter((s) => s.length > 0) : [];

  return { item, rate, machines, inputs, belt, strategy, rowLayout, inserterTier, quality, wireMode, customInputs };
}

export function readUrlState(): FormState {
  // New hash form takes precedence when both happen to be present.
  return readHashState() ?? readQueryState();
}

/** Detect whether the URL carries enough state to skip the landing page.
 * Used by `main.ts` before WASM is ready, so this only sniffs the URL
 * shape — it doesn't decode short codes. */
export function urlHasGeneratorState(): boolean {
  if (window.location.hash.startsWith(HASH_PREFIX)) return true;
  const params = new URLSearchParams(window.location.search);
  return (
    params.has("item") ||
    params.has("rate") ||
    params.has("machine") ||
    params.has("in") ||
    params.has("belt")
  );
}

function machinesAreDefault(machines: Record<string, string>): boolean {
  // Defaults are equal when every editable category either matches its
  // default exactly or is absent from the palette.
  for (const [category, urlKey] of Object.entries(URL_KEY_BY_CATEGORY)) {
    void urlKey;
    const picked = machines[category];
    if (picked && picked !== DEFAULT_MACHINES[category]) return false;
  }
  // Reject any palette entry not in URL_KEY_BY_CATEGORY (shouldn't happen
  // through the UI, but a hand-edited URL could).
  for (const category of Object.keys(machines)) {
    if (!(category in URL_KEY_BY_CATEGORY)) return false;
  }
  return true;
}

function formatHashState(state: FormState): string {
  const itemCode = slugToCode(state.item);
  const rate = String(state.rate);
  // Slot 2 carries the crafting tier — the canonical assembler-tier knob.
  // Other categories (smelting today) ride as extras when non-default.
  const crafting = state.machines.crafting;
  const machineCode =
    !crafting || crafting === DEFAULT_MACHINES.crafting
      ? SKIP_TOKEN
      : slugToCode(crafting);
  const inputsAreDefault =
    state.inputs.length === DEFAULT_CHECKED_INPUTS.length &&
    state.inputs.every((v, i) => v === DEFAULT_CHECKED_INPUTS[i]);
  const inputsCode =
    state.inputs.length === 0 || inputsAreDefault
      ? SKIP_TOKEN
      : state.inputs.map(slugToCode).join(INPUT_SEPARATOR);
  const beltCode = state.belt ? slugToCode(state.belt) : SKIP_TOKEN;

  const extras = new URLSearchParams();
  if (state.strategy && STRATEGY_FULL_TO_SHORT[state.strategy]) {
    extras.set("s", STRATEGY_FULL_TO_SHORT[state.strategy]);
  }
  if (state.rowLayout && ROW_LAYOUT_FULL_TO_SHORT[state.rowLayout]) {
    extras.set("rl", ROW_LAYOUT_FULL_TO_SHORT[state.rowLayout]);
  }
  if (state.inserterTier && INSERTER_TIER_FULL_TO_SHORT[state.inserterTier]) {
    extras.set("it", INSERTER_TIER_FULL_TO_SHORT[state.inserterTier]);
  }
  if (state.quality && QUALITY_FULL_TO_SHORT[state.quality]) {
    extras.set("q", QUALITY_FULL_TO_SHORT[state.quality]);
  }
  if (state.wireMode && WIRE_MODE_FULL_TO_SHORT[state.wireMode]) {
    extras.set("w", WIRE_MODE_FULL_TO_SHORT[state.wireMode]);
  }
  if (state.customInputs.length > 0) {
    extras.set(
      "ci",
      state.customInputs.map(slugToCode).join(INPUT_SEPARATOR),
    );
  }
  // Per-category machines other than crafting — encode as
  // `<urlKey>=<short-code>` extras using the same `URL_KEY_BY_CATEGORY`
  // table the query-form writer uses, so a hand-edited hash URL reads
  // the same as a hand-edited query URL for any non-default smelter.
  for (const [category, urlKey] of Object.entries(URL_KEY_BY_CATEGORY)) {
    if (category === "crafting") continue;
    const picked = state.machines[category];
    if (picked && picked !== DEFAULT_MACHINES[category]) {
      extras.set(urlKey, slugToCode(picked));
    }
  }

  // Trim trailing skip-token slots when no extras follow — produces
  // `#/l/ipr/5` instead of `#/l/ipr/5/_/_/_` for the common case where
  // only item + rate diverge from defaults. Reader treats missing slots
  // and `_` slots identically, so this stays round-trip-safe.
  const slots = [itemCode, rate, machineCode, inputsCode, beltCode];
  const extrasStr = extras.toString();
  if (extrasStr.length === 0) {
    while (slots.length > 2 && slots[slots.length - 1] === SKIP_TOKEN) {
      slots.pop();
    }
  }
  let path = `${HASH_PREFIX}${slots.join("/")}`;
  if (extrasStr.length > 0) path += `?${extrasStr}`;
  return path;
}

export function writeUrlState(state: FormState): void {
  const isDefault =
    state.item === DEFAULT_ITEM &&
    state.rate === DEFAULT_RATE &&
    machinesAreDefault(state.machines) &&
    state.inputs.length === DEFAULT_CHECKED_INPUTS.length &&
    state.inputs.every((v, i) => v === DEFAULT_CHECKED_INPUTS[i]) &&
    !state.belt &&
    !state.strategy &&
    !state.rowLayout &&
    !state.inserterTier &&
    state.customInputs.length === 0;

  // Drop any stale `?...` query string when transitioning to hash-form
  // URLs, otherwise legacy params would shadow the hash on next read.
  const cleanPath = window.location.pathname;

  if (isDefault) {
    history.replaceState(null, "", cleanPath);
    return;
  }

  history.replaceState(null, "", cleanPath + formatHashState(state));
}
