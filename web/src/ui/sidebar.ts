import type { Engine, SolverResult, LayoutResult, ItemFlow, ValidationIssue, TraceEvent } from "../engine.js";
import { readUrlState, writeUrlState, DEFAULT_INPUTS, DEFAULT_MACHINES } from "../state.js";
import { beltTierForRate, hexToCss } from "../renderer/colors.js";
import { niceName, setRecipeFlows, preloadCarriesIcons } from "../renderer/entities.js";
import "./sidebar.css";

/** Marker the Rust solver prepends to `SolverError::IncompatibleMachine`
 * messages. Must stay in sync with `INCOMPATIBLE_MACHINE_PREFIX` in
 * `crates/core/src/solver.rs`. The sidebar splits on it to route the
 * message to the dedicated config-error banner. */
const INCOMPATIBLE_MACHINE_MARKER = "[INCOMPATIBLE_MACHINE]";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function itemIcon(slug: string, size = 14): HTMLImageElement {
  const img = document.createElement("img");
  img.src = `${import.meta.env.BASE_URL}icons/${slug}.png`;
  img.width = size;
  img.height = size;
  img.style.cssText = "image-rendering:pixelated";
  img.onerror = () => { img.style.display = "none"; };
  return img;
}

function makeOption(value: string, defaultValue: string): HTMLOptionElement {
  const opt = document.createElement("option");
  opt.value = value;
  opt.textContent = niceName(value);
  if (value === defaultValue) opt.selected = true;
  return opt;
}

/** Create a section block with icon + title. */
function makeSection(
  iconSvg: string,
  title: string,
  extra?: string,
): { section: HTMLDivElement; body: HTMLDivElement; countEl: HTMLSpanElement | null } {
  const section = document.createElement("div");
  section.className = "sb-section";

  const header = document.createElement("div");
  header.className = "sb-section-header";

  const iconEl = document.createElement("span");
  iconEl.className = "sb-section-icon";
  iconEl.innerHTML = iconSvg;
  header.appendChild(iconEl);

  const titleEl = document.createElement("span");
  titleEl.textContent = title;
  header.appendChild(titleEl);

  let countEl: HTMLSpanElement | null = null;
  if (extra !== undefined) {
    countEl = document.createElement("span");
    countEl.className = "sb-section-count";
    countEl.textContent = extra;
    header.appendChild(countEl);
  }

  section.appendChild(header);

  const body = document.createElement("div");
  section.appendChild(body);

  return { section, body, countEl };
}

function appendFlows(
  container: HTMLElement,
  flows: ItemFlow[],
  className: string,
  prefix: string,
): void {
  for (const flow of flows) {
    const el = document.createElement("div");
    el.className = `sb-machine-flow ${className}`;
    if (prefix) el.appendChild(document.createTextNode(prefix));
    el.appendChild(itemIcon(flow.item, 13));
    el.appendChild(document.createTextNode(niceName(flow.item)));
    const rateSpan = document.createElement("span");
    rateSpan.className = "flow-rate";
    const tier = beltTierForRate(flow.rate);
    const rateColor = tier ? hexToCss(tier.color) : "#f88";
    rateSpan.style.color = rateColor;
    rateSpan.textContent = `${flow.rate.toFixed(1)}/s`;
    el.appendChild(rateSpan);
    container.appendChild(el);
  }
}

// Fluid items that get blue-tinted tag pills
const FLUID_ITEMS = new Set(["water", "crude-oil", "petroleum-gas", "light-oil", "heavy-oil", "sulfuric-acid", "lubricant", "steam"]);

// ---------------------------------------------------------------------------
// Rich item picker
// ---------------------------------------------------------------------------

const RECENT_ITEMS_KEY = "spaghettio-recent-items";
const RECENT_MAX = 5;

function getRecentItems(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_ITEMS_KEY);
    return raw ? (JSON.parse(raw) as string[]) : [];
  } catch { return []; }
}

function pushRecentItem(slug: string): void {
  const recent = getRecentItems().filter((r) => r !== slug);
  recent.unshift(slug);
  if (recent.length > RECENT_MAX) recent.length = RECENT_MAX;
  try { localStorage.setItem(RECENT_ITEMS_KEY, JSON.stringify(recent)); } catch { /* quota */ }
}

function makeItemPicker(
  allItems: string[],
  initial: string,
  onChange: (item: string) => void,
): { el: HTMLDivElement; getValue(): string; setValue(item: string): void; setInvalid(v: boolean): void } {
  let selectedItem = initial;
  let isOpen = false;
  let highlightedEl: HTMLElement | null = null;

  const container = document.createElement("div");
  container.className = "sb-item-picker";

  const valueEl = document.createElement("div");
  valueEl.className = "sb-picker-value";

  const arrowEl = document.createElement("span");
  arrowEl.className = "sb-picker-arrow";
  arrowEl.textContent = "▾";

  container.appendChild(valueEl);
  container.appendChild(arrowEl);

  const dropdown = document.createElement("div");
  dropdown.className = "sb-picker-dropdown";
  dropdown.style.display = "none";
  container.appendChild(dropdown);

  const searchInput = document.createElement("input");
  searchInput.type = "text";
  searchInput.className = "sb-picker-search";
  searchInput.placeholder = "Search items…";
  dropdown.appendChild(searchInput);

  const listEl = document.createElement("div");
  listEl.className = "sb-picker-list";
  dropdown.appendChild(listEl);

  function updateValueDisplay(): void {
    valueEl.innerHTML = "";
    if (selectedItem) {
      valueEl.appendChild(itemIcon(selectedItem, 14));
      const txt = document.createElement("span");
      txt.textContent = niceName(selectedItem);
      valueEl.appendChild(txt);
    } else {
      const placeholder = document.createElement("span");
      placeholder.className = "sb-picker-placeholder";
      placeholder.textContent = "Select item…";
      valueEl.appendChild(placeholder);
    }
  }

  function makeListItem(slug: string): HTMLElement {
    const el = document.createElement("div");
    el.className = "sb-picker-item" + (slug === selectedItem ? " selected" : "");
    el.dataset.slug = slug;
    el.appendChild(itemIcon(slug, 14));
    const lbl = document.createElement("span");
    lbl.textContent = niceName(slug);
    el.appendChild(lbl);
    el.addEventListener("mousedown", (e) => {
      e.preventDefault();
      selectItem(slug);
    });
    return el;
  }

  function renderList(query: string): void {
    listEl.innerHTML = "";
    highlightedEl = null;
    const q = query.trim().toLowerCase();
    const filtered = q
      ? allItems.filter((s) => s.includes(q) || niceName(s).toLowerCase().includes(q))
      : allItems;

    if (!q) {
      const recent = getRecentItems().filter((r) => allItems.includes(r));
      if (recent.length > 0) {
        const label = document.createElement("div");
        label.className = "sb-picker-section-label";
        label.textContent = "Recent";
        listEl.appendChild(label);
        for (const slug of recent) listEl.appendChild(makeListItem(slug));
        const divider = document.createElement("div");
        divider.className = "sb-picker-divider";
        listEl.appendChild(divider);
      }
    }

    for (const slug of filtered) listEl.appendChild(makeListItem(slug));

    if (!q && selectedItem) {
      const sel = listEl.querySelector<HTMLElement>(`[data-slug="${selectedItem}"]`);
      if (sel) sel.scrollIntoView({ block: "nearest" });
    }
  }

  function selectItem(slug: string): void {
    selectedItem = slug;
    pushRecentItem(slug);
    container.classList.remove("item-invalid");
    updateValueDisplay();
    closeDropdown();
    onChange(slug);
  }

  function openDropdown(): void {
    isOpen = true;
    container.classList.add("open");
    dropdown.style.display = "";
    arrowEl.textContent = "▴";
    searchInput.value = "";
    renderList("");
    requestAnimationFrame(() => searchInput.focus());
  }

  function closeDropdown(): void {
    isOpen = false;
    container.classList.remove("open");
    dropdown.style.display = "none";
    arrowEl.textContent = "▾";
    highlightedEl = null;
  }

  function moveFocus(dir: 1 | -1): void {
    const items = listEl.querySelectorAll<HTMLElement>(".sb-picker-item");
    if (!items.length) return;
    const arr = Array.from(items);
    let idx = highlightedEl ? arr.indexOf(highlightedEl) : -1;
    idx = Math.max(0, Math.min(arr.length - 1, idx + dir));
    highlightedEl?.classList.remove("highlighted");
    highlightedEl = arr[idx];
    highlightedEl.classList.add("highlighted");
    highlightedEl.scrollIntoView({ block: "nearest" });
  }

  container.addEventListener("mousedown", (e) => {
    if (dropdown.contains(e.target as Node)) return;
    e.preventDefault();
    if (isOpen) closeDropdown(); else openDropdown();
  });

  searchInput.addEventListener("input", () => renderList(searchInput.value));

  searchInput.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown") { e.preventDefault(); moveFocus(1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); moveFocus(-1); }
    else if (e.key === "Enter") {
      if (highlightedEl?.dataset.slug) selectItem(highlightedEl.dataset.slug);
    } else if (e.key === "Escape") {
      closeDropdown();
    }
  });

  document.addEventListener("mousedown", (e) => {
    if (isOpen && !container.contains(e.target as Node)) closeDropdown();
  });

  updateValueDisplay();

  return {
    el: container,
    getValue: () => selectedItem,
    setValue(slug: string) {
      selectedItem = slug;
      updateValueDisplay();
    },
    setInvalid(v: boolean) {
      container.classList.toggle("item-invalid", v);
    },
  };
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

export interface SidebarCallbacks {
  renderGraph: (result: SolverResult | null) => void;
  renderLayout: (layout: LayoutResult, solverResult: SolverResult) => void;
  /** Begin a new streaming layout render. Returns the per-event callback that
   *  sidebar passes into `engine.buildLayoutStreaming`. Cancels any prior
   *  streaming render first. */
  startStreaming: () => (evt: TraceEvent) => void;
}

export interface SidebarParams {
  item: string;
  rate: number;
  machine?: string;
  inputs?: string[];
  customInputs?: string[];
  belt?: string | null;
}

export function renderSidebar(
  el: HTMLElement,
  engine: Engine,
  callbacks: SidebarCallbacks,
): { getParams(): SidebarParams | null; setParams(params: SidebarParams, opts?: { skipAutoSolve?: boolean }): void; updateValidation(issues: ValidationIssue[], onPanToTile: (x: number, y: number) => void): void } {
  el.innerHTML = "";

  const inner = document.createElement("div");
  inner.className = "sidebar-inner";

  // ==================== TARGET ====================
  const { section: targetSection, body: targetBody } = makeSection(
    `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>`,
    "Target",
  );

  const allItems = engine.allProducibleItems();
  const itemSet = new Set(allItems);

  function makeField(label: string, control: HTMLElement): HTMLDivElement {
    const row = document.createElement("div");
    row.className = "sb-field";
    const lbl = document.createElement("span");
    lbl.className = "sb-field-label";
    lbl.textContent = label;
    row.appendChild(lbl);
    control.style.flex = "1";
    control.style.minWidth = "0";
    row.appendChild(control);
    return row;
  }

  // Item — rich picker
  const picker = makeItemPicker(allItems, "", () => scheduleAutoSolve());
  picker.el.style.cssText = "margin-bottom:6px";
  targetBody.appendChild(picker.el);

  // Per-category machine palette. Each editable category gets a `<select>`
  // tagged with `data-cat="<category>"` so the solve handler can build the
  // palette in one pass. Categories with only one Space Age option today
  // render as a read-only label so the user can see which machine each
  // recipe class will use.
  type EditableMachine = {
    category: string;
    label: string;
    options: { value: string; disabled?: boolean; title?: string }[];
  };
  const EDITABLE_MACHINES: EditableMachine[] = [
    {
      category: "crafting",
      label: "Assembler",
      options: [
        { value: "assembling-machine-1" },
        { value: "assembling-machine-2" },
        { value: "assembling-machine-3" },
      ],
    },
    {
      category: "smelting",
      label: "Furnace",
      options: [
        { value: "electric-furnace" },
        { value: "stone-furnace", disabled: true, title: "Requires fuel routing — coming later" },
      ],
    },
  ];
  const READONLY_MACHINES: { label: string; machine: string }[] = [
    { label: "Foundry", machine: "foundry" },
    { label: "EM Plant", machine: "electromagnetic-plant" },
    { label: "Chemical Plant", machine: "chemical-plant" },
    { label: "Oil Refinery", machine: "oil-refinery" },
    { label: "Cryogenic Plant", machine: "cryogenic-plant" },
    { label: "Biochamber", machine: "biochamber" },
  ];

  const machineSelects = new Map<string, HTMLSelectElement>();
  for (const m of EDITABLE_MACHINES) {
    const sel = document.createElement("select");
    sel.className = "sb-select";
    sel.dataset.cat = m.category;
    const defaultValue = DEFAULT_MACHINES[m.category] ?? "";
    for (const opt of m.options) {
      const o = makeOption(opt.value, defaultValue);
      if (opt.disabled) o.disabled = true;
      if (opt.title) o.title = opt.title;
      sel.appendChild(o);
    }
    targetBody.appendChild(makeField(m.label, sel));
    machineSelects.set(m.category, sel);
  }
  // Kept around for the back-compat readers below (auto-pick on item change,
  // setParams snapshot restore). These all act on the assembler tier.
  const machineSelect = machineSelects.get("crafting")!;
  for (const ro of READONLY_MACHINES) {
    const span = document.createElement("span");
    span.className = "sb-machine-readonly";
    span.textContent = niceName(ro.machine);
    targetBody.appendChild(makeField(ro.label, span));
  }

  function buildPalette(): Record<string, string> {
    const palette: Record<string, string> = {};
    for (const [cat, sel] of machineSelects) {
      if (sel.value) palette[cat] = sel.value;
    }
    return palette;
  }

  // Belt tier (Auto / Yellow / Red / Blue) — moved up from the former Layout section
  const beltSelect = document.createElement("select");
  beltSelect.className = "sb-select";
  [
    ["Auto", ""],
    ["Yellow 15/s", "transport-belt"],
    ["Red 30/s", "fast-transport-belt"],
    ["Blue 45/s", "express-transport-belt"],
  ].forEach(([label, value]) => {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = label;
    beltSelect.appendChild(opt);
  });
  targetBody.appendChild(makeField("Belt", beltSelect));

  // Layout strategy. Phase 0b of `rfp-modular-production` shipped the
  // dropdown; the surviving `partitioned-decomposed` variant produces
  // strictly ≤ Pooled errors on every case in the corpus. The deprecated
  // `partitioned-per-consumer` (P1) option was dropped from the UI when
  // the P1 enum variant was hard-deleted; bookmarked URLs still load
  // via the back-compat alias in `state.ts`.
  const strategySelect = document.createElement("select");
  strategySelect.className = "sb-select";
  ([
    ["Pooled (default)", ""],
    ["Partitioned + decomposed", "partitioned-decomposed"],
  ] as const).forEach(([label, value]) => {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = label;
    strategySelect.appendChild(opt);
  });
  targetBody.appendChild(makeField("Strategy", strategySelect));

  // Row layout. See `docs/rfp-horizontal-trunks.md`. Default is the
  // existing vertical-split behaviour; horizontal-stack is being
  // developed under that RFP and currently only handles dual-input
  // solid recipes (other row kinds fall back to vertical-split).
  const rowLayoutSelect = document.createElement("select");
  rowLayoutSelect.className = "sb-select";
  ([
    ["Vertical split (today)", ""],
    ["Horizontal stack (RFP)", "horizontal-stack"],
  ] as const).forEach(([label, value]) => {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = label;
    rowLayoutSelect.appendChild(opt);
  });
  targetBody.appendChild(makeField("Row layout", rowLayoutSelect));

  // Rate (numeric with /s suffix)
  const rateRow = document.createElement("div");
  rateRow.className = "sb-field";
  const rateLabel = document.createElement("span");
  rateLabel.className = "sb-field-label";
  rateLabel.textContent = "Rate";
  rateRow.appendChild(rateLabel);
  const rateInput = document.createElement("input");
  rateInput.type = "number";
  rateInput.className = "sb-input";
  rateInput.step = "0.5";
  rateInput.min = "0.1";
  rateInput.style.cssText = "flex:1;min-width:0";
  rateInput.placeholder = "10";
  rateRow.appendChild(rateInput);
  const rateSuffix = document.createElement("span");
  rateSuffix.className = "sb-rate-suffix";
  rateSuffix.textContent = "/s";
  rateRow.appendChild(rateSuffix);
  targetBody.appendChild(rateRow);

  inner.appendChild(targetSection);

  // ==================== INPUTS ====================
  const { section: inputsSection, body: inputsBody } = makeSection(
    `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>`,
    "Inputs",
  );

  const tagsWrap = document.createElement("div");
  tagsWrap.className = "sb-tags";

  const checkboxes = new Map<string, HTMLInputElement>();
  DEFAULT_INPUTS.forEach((inp) => {
    const tag = document.createElement("label");
    tag.className = `sb-tag${FLUID_ITEMS.has(inp) ? " fluid" : ""}`;

    const checkSpan = document.createElement("span");
    checkSpan.className = "sb-tag-check";
    checkSpan.textContent = "✓";

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.value = inp;
    cb.style.display = "none";
    checkboxes.set(inp, cb);

    tag.appendChild(checkSpan);
    tag.appendChild(itemIcon(inp, 14));
    tag.appendChild(document.createTextNode(niceName(inp)));
    tag.appendChild(cb);

    cb.addEventListener("change", () => {
      tag.classList.toggle("active", cb.checked);
    });

    tagsWrap.appendChild(tag);
  });
  inputsBody.appendChild(tagsWrap);

  // Custom inputs — user-added items beyond DEFAULT_INPUTS
  let customInputs: string[] = [];

  const customTagsWrap = document.createElement("div");
  customTagsWrap.className = "sb-tags sb-custom-tags";
  inputsBody.appendChild(customTagsWrap);

  // Datalist for custom-input search field (excludes DEFAULT_INPUTS)
  const customInputDatalist = document.createElement("datalist");
  customInputDatalist.id = "spaghettio-custom-inputs-datalist";
  const defaultInputSet = new Set(DEFAULT_INPUTS);
  allItems.filter((it) => !defaultInputSet.has(it)).forEach((it) => {
    const opt = document.createElement("option");
    opt.value = it;
    customInputDatalist.appendChild(opt);
  });
  inputsBody.appendChild(customInputDatalist);

  const customInputField = document.createElement("input");
  customInputField.type = "text";
  customInputField.className = "sb-input sb-custom-input-field";
  customInputField.setAttribute("list", "spaghettio-custom-inputs-datalist");
  customInputField.autocomplete = "off";
  customInputField.placeholder = "+ add input…";
  inputsBody.appendChild(customInputField);

  function renderCustomTag(item: string): void {
    const tag = document.createElement("div");
    tag.className = `sb-tag sb-custom-tag active${FLUID_ITEMS.has(item) ? " fluid" : ""}`;
    tag.dataset.item = item;
    tag.appendChild(itemIcon(item, 14));
    tag.appendChild(document.createTextNode(niceName(item)));
    const removeBtn = document.createElement("span");
    removeBtn.className = "sb-tag-remove";
    removeBtn.textContent = "×";
    removeBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      customInputs = customInputs.filter((x) => x !== item);
      tag.remove();
      scheduleAutoSolve();
    });
    tag.appendChild(removeBtn);
    customTagsWrap.appendChild(tag);
  }

  function tryAddCustomInput(raw: string): void {
    const val = raw.trim();
    if (!val || !itemSet.has(val)) return;
    if (defaultInputSet.has(val) || customInputs.includes(val)) return;
    customInputs.push(val);
    renderCustomTag(val);
    customInputField.value = "";
    scheduleAutoSolve();
  }

  customInputField.addEventListener("keydown", (e) => {
    if (e.key === "Enter") tryAddCustomInput(customInputField.value);
  });

  customInputField.addEventListener("change", () => {
    // datalist selection fires a change event
    tryAddCustomInput(customInputField.value);
  });

  inner.appendChild(inputsSection);

  // ==================== SOLVER ====================
  const { section: solverSection, body: solverBody, countEl: solverCount } = makeSection(
    `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>`,
    "Solver",
    "",
  );

  const resultContainer = document.createElement("div");
  solverBody.appendChild(resultContainer);
  inner.appendChild(solverSection);

  // Copy-blueprint action — appended to solver body when a layout is ready.
  const blueprintSection = document.createElement("div");
  blueprintSection.className = "sb-actions";
  blueprintSection.style.display = "none";

  const copyBtn = document.createElement("button");
  copyBtn.className = "sb-btn sb-btn-secondary";
  copyBtn.textContent = "Copy Blueprint";
  copyBtn.style.flex = "1";
  blueprintSection.appendChild(copyBtn);

  const copyStatus = document.createElement("div");
  copyStatus.className = "sb-copy-status";
  blueprintSection.appendChild(copyStatus);
  solverBody.appendChild(blueprintSection);

  // ==================== VALIDATION ====================
  const { section: valSection, body: valBody, countEl: valCountEl } = makeSection(
    `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>`,
    "Validation",
    "",
  );
  valSection.style.display = "none";
  inner.appendChild(valSection);

  el.appendChild(inner);

  // ==================== State init ====================
  const urlState = readUrlState();
  picker.setValue(urlState.item);
  rateInput.value = String(urlState.rate);
  // Per-category palette. Each `<select>` only keeps a URL value if it's a
  // valid (non-disabled) option for its category, so a hand-edited URL or a
  // legacy `?machine=`-derived value that points at a removed machine falls
  // back to the default rather than rendering an empty select.
  const ASSEMBLER_TIERS = new Set(["assembling-machine-1", "assembling-machine-2", "assembling-machine-3"]);
  for (const [cat, sel] of machineSelects) {
    const urlValue = urlState.machines[cat];
    const validOptions = new Set(
      Array.from(sel.options).filter((o) => !o.disabled).map((o) => o.value),
    );
    sel.value = urlValue && validOptions.has(urlValue)
      ? urlValue
      : (DEFAULT_MACHINES[cat] ?? sel.options[0]?.value ?? "");
  }
  checkboxes.forEach((cb, name) => {
    cb.checked = urlState.inputs.includes(name);
    const tag = cb.closest(".sb-tag") as HTMLLabelElement;
    if (tag) tag.classList.toggle("active", cb.checked);
  });
  if (urlState.belt) beltSelect.value = urlState.belt;
  if (urlState.strategy) strategySelect.value = urlState.strategy;
  if (urlState.rowLayout) rowLayoutSelect.value = urlState.rowLayout;
  // Restore custom inputs from URL
  for (const item of urlState.customInputs) {
    if (itemSet.has(item) && !defaultInputSet.has(item) && !customInputs.includes(item)) {
      customInputs.push(item);
      renderCustomTag(item);
    }
  }

  // Pre-flight error banner. Rendered above the solver result; populated
  // by `setConfigError(msg)`. Today nothing writes to it — the next PR
  // (incompatible-machine preflight + layout-error promotion) wires it up.
  const configErrorDiv = document.createElement("div");
  configErrorDiv.className = "sb-config-error";
  configErrorDiv.style.display = "none";
  resultContainer.before(configErrorDiv);
  function setConfigError(message: string | null): void {
    if (message) {
      configErrorDiv.textContent = message;
      configErrorDiv.style.display = "";
    } else {
      configErrorDiv.textContent = "";
      configErrorDiv.style.display = "none";
    }
  }

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let previousItem = urlState.item;
  let currentLayout: LayoutResult | null = null;
  let solveGeneration = 0;

  function scheduleAutoSolve(): void {
    if (debounceTimer !== null) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      runSolve().catch((err) => console.error("runSolve failed:", err));
    }, 150);
  }

  async function runSolve(): Promise<void> {
    const targetItem = picker.getValue();
    const targetRate = parseFloat(rateInput.value);
    const checkedDefaults = DEFAULT_INPUTS.filter((inp) => checkboxes.get(inp)?.checked);
    const availableInputs = [...checkedDefaults, ...customInputs];

    if (!itemSet.has(targetItem)) {
      picker.setInvalid(true);
      return;
    }
    picker.setInvalid(false);

    if (isNaN(targetRate) || targetRate <= 0) return;

    if (targetItem !== previousItem) {
      // Auto-pick assembler tier from the recipe's category mapping. Only
      // applies to the crafting `<select>` — the per-category palette
      // doesn't (yet) feed `defaultMachineForItem`.
      const suggestedMachine = engine.defaultMachineForItem(targetItem, machineSelect.value);
      if (ASSEMBLER_TIERS.has(suggestedMachine)) machineSelect.value = suggestedMachine;
      previousItem = targetItem;
    }

    const palette = buildPalette();

    writeUrlState({
      item: targetItem,
      rate: targetRate,
      machines: palette,
      inputs: checkedDefaults,
      belt: beltSelect.value || null,
      strategy: strategySelect.value || null,
      rowLayout: rowLayoutSelect.value || null,
      customInputs,
    });

    const gen = ++solveGeneration;
    resultContainer.innerHTML = "";
    setConfigError(null);
    currentLayout = null;
    blueprintSection.style.display = "none";

    let result: SolverResult;
    try {
      result = await engine.solve(
        targetItem,
        targetRate,
        availableInputs,
        palette,
        palette.crafting ?? DEFAULT_MACHINES.crafting,
      );
    } catch (err) {
      if (gen !== solveGeneration) return;
      callbacks.renderGraph(null);
      if (solverCount) solverCount.textContent = "error";
      const msg = String(err instanceof Error ? err.message : err);
      // Pre-flight machine/category errors carry a marker prefix from the
      // Rust solver. Route them to the dedicated config-error banner so
      // they sit above the result container; everything else stays inline.
      if (msg.includes(INCOMPATIBLE_MACHINE_MARKER)) {
        const idx = msg.indexOf(INCOMPATIBLE_MACHINE_MARKER);
        const cleaned = msg.slice(idx + INCOMPATIBLE_MACHINE_MARKER.length).trim();
        setConfigError(cleaned);
      } else {
        const errDiv = document.createElement("div");
        errDiv.className = "sb-result-error";
        errDiv.textContent = msg;
        resultContainer.appendChild(errDiv);
      }
      return;
    }
    if (gen !== solveGeneration) return;

    renderResult(resultContainer, result);
    callbacks.renderGraph(result);
    const totalMachines = result.machines.reduce((sum, m) => sum + Math.ceil(m.count), 0);
    if (solverCount) solverCount.textContent = `${totalMachines} machines`;

    // Scoped carries-icon preload before streaming kicks in. The streaming
    // renderer commits ghost / trunk belts incrementally with their carries
    // icon, and committed particles don't pick up textures that arrive
    // later — so icons must be cached before the first event fires. The
    // set is the union of every recipe input/output and external in/out,
    // typically <20 items.
    const carriesItems = new Set<string>();
    for (const m of result.machines) {
      for (const i of m.inputs) carriesItems.add(i.item);
      for (const o of m.outputs) carriesItems.add(o.item);
    }
    for (const e of result.external_inputs) carriesItems.add(e.item);
    for (const e of result.external_outputs) carriesItems.add(e.item);
    await preloadCarriesIcons(Array.from(carriesItems));
    if (gen !== solveGeneration) return;

    let layout: LayoutResult;
    try {
      const maxTier = beltSelect.value || undefined;
      const strategy = strategySelect.value || undefined;
      const rowLayout = rowLayoutSelect.value || undefined;
      const onEvent = callbacks.startStreaming();
      layout = await engine.buildLayoutStreaming(result, maxTier, strategy, rowLayout, onEvent);
    } catch (err) {
      if (gen !== solveGeneration) return;
      const errDiv = document.createElement("div");
      errDiv.className = "sb-result-error";
      errDiv.textContent = `Layout error: ${err}`;
      resultContainer.appendChild(errDiv);
      return;
    }
    if (gen !== solveGeneration) return;

    currentLayout = layout;
    setRecipeFlows(result.machines);
    callbacks.renderLayout(layout, result);
    // Layout-level warnings (missing balancer templates, unresolved
    // ghost-router crossings) now surface in the Validation panel below
    // — kept off the result container so there's a single source of truth.
    blueprintSection.style.display = layout.warnings?.length ? "none" : "flex";
  }

  copyBtn.addEventListener("click", async () => {
    if (!currentLayout) return;
    const bp = await engine.exportBlueprint(currentLayout, picker.getValue());
    await navigator.clipboard.writeText(bp);
    copyStatus.textContent = "Copied!";
    setTimeout(() => { copyStatus.textContent = ""; }, 2000);
  });

  rateInput.addEventListener("input", scheduleAutoSolve);
  for (const sel of machineSelects.values()) {
    sel.addEventListener("change", scheduleAutoSolve);
  }
  beltSelect.addEventListener("change", scheduleAutoSolve);
  strategySelect.addEventListener("change", scheduleAutoSolve);
  rowLayoutSelect.addEventListener("change", scheduleAutoSolve);
  checkboxes.forEach((cb) => cb.addEventListener("change", scheduleAutoSolve));

  runSolve().catch((err) => console.error("runSolve failed:", err));

  return {
    getParams() {
      const item = picker.getValue();
      const rate = parseFloat(rateInput.value);
      if (!item || isNaN(rate) || rate <= 0) return null;
      return { item, rate };
    },
    setParams(params, opts) {
      picker.setValue(params.item);
      rateInput.value = String(params.rate);
      if (params.machine && ASSEMBLER_TIERS.has(params.machine)) {
        machineSelect.value = params.machine;
      } else {
        machineSelect.value = "assembling-machine-3";
      }
      if (params.inputs) {
        checkboxes.forEach((cb, name) => {
          cb.checked = params.inputs!.includes(name);
          const tag = cb.closest(".sb-tag") as HTMLLabelElement;
          if (tag) tag.classList.toggle("active", cb.checked);
        });
      }
      if (params.belt) {
        beltSelect.value = params.belt;
      } else {
        beltSelect.value = "";
      }
      // Restore custom inputs
      customTagsWrap.innerHTML = "";
      customInputs = [];
      for (const item of (params.customInputs ?? [])) {
        if (itemSet.has(item) && !defaultInputSet.has(item) && !customInputs.includes(item)) {
          customInputs.push(item);
          renderCustomTag(item);
        }
      }
      previousItem = params.item;
      if (!opts?.skipAutoSolve) {
        scheduleAutoSolve();
      }
    },
    updateValidation(issues: ValidationIssue[], onPanToTile: (x: number, y: number) => void) {
      valBody.innerHTML = "";
      if (issues.length === 0) {
        valSection.style.display = "none";
        if (valCountEl) valCountEl.textContent = "";
        return;
      }
      valSection.style.display = "";

      const errors = issues.filter(i => i.severity === "Error").length;
      const warns = issues.length - errors;
      if (valCountEl) {
        if (errors > 0) {
          valCountEl.textContent = `${errors} error${errors !== 1 ? "s" : ""}`;
          valCountEl.style.color = "#f66";
        } else {
          valCountEl.textContent = `${warns} warning${warns !== 1 ? "s" : ""}`;
          valCountEl.style.color = "#fa0";
        }
      }

      // Group by category
      const groups = new Map<string, ValidationIssue[]>();
      for (const issue of issues) {
        let g = groups.get(issue.category);
        if (!g) { g = []; groups.set(issue.category, g); }
        g.push(issue);
      }

      for (const [category, groupIssues] of groups) {
        const hasErrors = groupIssues.some(i => i.severity === "Error");
        const dotColor = hasErrors ? "#f44" : "#fa0";
        const firstWithPos = groupIssues.find(i => i.x != null && i.y != null);

        const groupEl = document.createElement("div");
        groupEl.className = "sb-val-group";

        const header = document.createElement("div");
        header.className = "sb-val-group-header";

        const chevron = document.createElement("span");
        chevron.className = "sb-val-group-chevron";
        chevron.textContent = "▾"; // down triangle (open)
        // Chevron toggles collapse without panning. Click on the rest of
        // the header pans to the first positional issue in the group.
        chevron.addEventListener("click", (e) => {
          e.stopPropagation();
          const collapsed = body.style.display === "none";
          body.style.display = collapsed ? "" : "none";
          chevron.textContent = collapsed ? "▾" : "▸";
        });
        header.appendChild(chevron);

        const dot = document.createElement("span");
        dot.className = "sb-val-group-dot";
        dot.style.background = dotColor;
        header.appendChild(dot);

        const name = document.createElement("span");
        name.className = "sb-val-group-name";
        name.textContent = category;
        header.appendChild(name);

        const count = document.createElement("span");
        count.className = "sb-val-group-count";
        count.textContent = String(groupIssues.length);
        header.appendChild(count);

        const body = document.createElement("div");
        body.className = "sb-val-group-body";

        if (firstWithPos) {
          header.classList.add("clickable");
          header.addEventListener("click", () => {
            onPanToTile(firstWithPos.x!, firstWithPos.y!);
          });
        }

        for (const issue of groupIssues) {
          const row = document.createElement("div");
          const hasPos = issue.x != null && issue.y != null;
          row.className = "sb-val-issue" + (hasPos ? " clickable" : "");

          const msg = document.createElement("span");
          msg.className = "sb-val-issue-msg";
          msg.textContent = issue.message;
          row.appendChild(msg);

          if (hasPos) {
            const coord = document.createElement("span");
            coord.className = "sb-val-issue-coord";
            coord.textContent = `${issue.x}, ${issue.y}`;
            row.appendChild(coord);
            row.addEventListener("click", (e) => {
              e.stopPropagation();
              const wasPinned = row.classList.contains("pinned");
              valBody.querySelectorAll(".sb-val-issue.pinned").forEach(el => el.classList.remove("pinned"));
              if (!wasPinned) {
                row.classList.add("pinned");
              }
              onPanToTile(issue.x!, issue.y!);
            });
          } else {
            row.style.opacity = "0.6";
          }
          body.appendChild(row);
        }

        groupEl.appendChild(header);
        groupEl.appendChild(body);
        valBody.appendChild(groupEl);
      }
    },
  };
}

// ---------------------------------------------------------------------------
// Result renderer — external inputs at top, then grouped machines
// ---------------------------------------------------------------------------

function renderResult(container: HTMLElement, result: SolverResult): void {
  // External inputs at top
  if (result.external_inputs.length > 0) {
    const extTitle = document.createElement("div");
    extTitle.className = "sb-ext-section-title";
    extTitle.textContent = "External inputs";
    container.appendChild(extTitle);

    for (const flow of result.external_inputs) {
      const row = document.createElement("div");
      row.className = "sb-ext-flow";
      row.appendChild(itemIcon(flow.item, 14));
      row.appendChild(document.createTextNode(niceName(flow.item)));
      const rateSpan = document.createElement("span");
      rateSpan.className = "sb-ext-rate";
      rateSpan.textContent = `${flow.rate.toFixed(1)}/s`;
      row.appendChild(rateSpan);
      container.appendChild(row);
    }

    const divider = document.createElement("div");
    divider.className = "sb-divider";
    container.appendChild(divider);
  }

  // Group machines by entity type (e.g. all assembling-machine-2 together)
  const groups = new Map<string, typeof result.machines>();
  for (const m of result.machines) {
    let group = groups.get(m.entity);
    if (!group) { group = []; groups.set(m.entity, group); }
    group.push(m);
  }

  for (const [entity, machines] of groups) {
    const totalCount = machines.reduce((s, m) => s + Math.ceil(m.count), 0);

    const groupEl = document.createElement("div");
    groupEl.className = "sb-machine-group";

    const header = document.createElement("div");
    header.className = "sb-machine-group-header";
    header.appendChild(itemIcon(entity, 16));
    const nameSpan = document.createElement("span");
    nameSpan.className = "sb-machine-group-name";
    nameSpan.textContent = niceName(entity);
    header.appendChild(nameSpan);
    const countSpan = document.createElement("span");
    countSpan.className = "sb-machine-group-count";
    countSpan.textContent = `×${totalCount}`;
    header.appendChild(countSpan);
    groupEl.appendChild(header);

    const body = document.createElement("div");
    body.className = "sb-machine-group-body";

    for (const machine of machines) {
      const recipeRow = document.createElement("div");
      recipeRow.className = "sb-machine-flow";
      recipeRow.style.cssText = "color:#6b7280;margin-bottom:2px";
      recipeRow.appendChild(document.createTextNode("→ "));
      recipeRow.appendChild(itemIcon(machine.recipe, 13));
      recipeRow.appendChild(document.createTextNode(niceName(machine.recipe)));
      body.appendChild(recipeRow);

      appendFlows(body, machine.inputs, "flow-in", "▶ ");
      appendFlows(body, machine.outputs, "flow-out", "◀ ");
    }

    groupEl.appendChild(body);
    container.appendChild(groupEl);
  }

  // Status bar: totals + external outputs
  const statusDiv = document.createElement("div");
  statusDiv.className = "sb-status";
  statusDiv.style.cssText = "margin-top:6px";

  const totalMachines = result.machines.reduce((sum, m) => sum + Math.ceil(m.count), 0);
  const chainDepth = result.dependency_order.length;

  const machinesSpan = document.createElement("span");
  machinesSpan.textContent = `${totalMachines} machines`;
  statusDiv.appendChild(machinesSpan);

  const depthSpan = document.createElement("span");
  depthSpan.textContent = `depth ${chainDepth}`;
  statusDiv.appendChild(depthSpan);

  container.appendChild(statusDiv);

  // External outputs
  if (result.external_outputs.length > 0) {
    for (const flow of result.external_outputs) {
      const row = document.createElement("div");
      row.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8";
      row.appendChild(itemIcon(flow.item, 13));
      row.appendChild(document.createTextNode(niceName(flow.item)));
      const tier = beltTierForRate(flow.rate);
      if (tier) {
        const tierColor = hexToCss(tier.color);
        row.appendChild(document.createTextNode(`${flow.rate.toFixed(1)}/s`));
        const chip = document.createElement("span");
        chip.className = "sb-belt-chip";
        chip.style.borderColor = tierColor;
        chip.style.color = tierColor;
        chip.textContent = tier.name;
        row.appendChild(chip);
      } else {
        row.appendChild(document.createTextNode(`${flow.rate.toFixed(1)}/s`));
        const warn = document.createElement("span");
        warn.className = "sb-belt-overflow";
        warn.textContent = "⚠ overflow";
        row.appendChild(warn);
      }
      container.appendChild(row);
    }
  }
}
