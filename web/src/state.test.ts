import { beforeEach, describe, expect, it } from "vitest";
import {
  DEFAULT_CHECKED_INPUTS,
  DEFAULT_ITEM,
  DEFAULT_MACHINES,
  DEFAULT_RATE,
  type FormState,
  readUrlState,
  urlHasGeneratorState,
  writeUrlState,
} from "./state.js";

// Tests poke `window.location` via `history.replaceState` — happy-dom
// supports both. Reset to a clean slate before each test so order
// independence is preserved.
beforeEach(() => {
  history.replaceState(null, "", "/");
});

function setUrl(suffix: string): void {
  history.replaceState(null, "", suffix);
}

function makeState(overrides: Partial<FormState>): FormState {
  return {
    item: DEFAULT_ITEM,
    rate: DEFAULT_RATE,
    machines: {},
    inputs: DEFAULT_CHECKED_INPUTS,
    belt: null,
    strategy: null,
    rowLayout: null,
    inserterTier: null,
    customInputs: [],
    ...overrides,
  };
}

describe("readUrlState — defaults", () => {
  it("returns defaults for an empty URL", () => {
    expect(readUrlState()).toEqual({
      item: DEFAULT_ITEM,
      rate: DEFAULT_RATE,
      machines: {},
      inputs: DEFAULT_CHECKED_INPUTS,
      belt: null,
      strategy: null,
      rowLayout: null,
      inserterTier: null,
      customInputs: [],
    });
  });
});

describe("readUrlState — hash form", () => {
  it("parses item + rate, fills in defaults", () => {
    setUrl("#/l/igw/10");
    const s = readUrlState();
    expect(s.item).toBe("iron-gear-wheel");
    expect(s.rate).toBe(10);
    expect(s.machines).toEqual({});
    expect(s.inputs).toEqual(DEFAULT_CHECKED_INPUTS);
    expect(s.belt).toBeNull();
  });

  it("decodes a fully-specified URL", () => {
    setUrl("#/l/acd/5/am1/ior,coo,coa,wat,cor/ftb");
    const s = readUrlState();
    expect(s.item).toBe("advanced-circuit");
    expect(s.rate).toBe(5);
    expect(s.machines.crafting).toBe("assembling-machine-1");
    expect(s.inputs).toEqual([
      "iron-ore",
      "copper-ore",
      "coal",
      "water",
      "crude-oil",
    ]);
    expect(s.belt).toBe("fast-transport-belt");
  });

  it("treats `_` and missing slots as 'use default'", () => {
    setUrl("#/l/_/5");
    const a = readUrlState();
    expect(a.item).toBe(DEFAULT_ITEM);
    expect(a.rate).toBe(5);

    setUrl("#/l/igw/10/_/_/_");
    const b = readUrlState();
    setUrl("#/l/igw/10");
    const c = readUrlState();
    expect(b).toEqual(c);
  });

  it("decodes extras (strategy, row layout, inserter tier, custom inputs)", () => {
    setUrl("#/l/acd/5?s=pd&rl=hs&it=f&ci=ipr,cpo");
    const s = readUrlState();
    expect(s.strategy).toBe("partitioned-decomposed");
    expect(s.rowLayout).toBe("horizontal-stack");
    expect(s.inserterTier).toBe("fast");
    expect(s.customInputs).toEqual(["iron-plate", "copper-plate"]);
  });

  it("falls back to legacy parser if any code is unknown", () => {
    // `zzqx` is not a real short code — the hash parser must reject the
    // whole URL rather than silently invent a slug. With nothing in the
    // query string either, the result is full defaults.
    setUrl("#/l/zzqx/5");
    expect(readUrlState()).toEqual({
      item: DEFAULT_ITEM,
      rate: DEFAULT_RATE,
      machines: {},
      inputs: DEFAULT_CHECKED_INPUTS,
      belt: null,
      strategy: null,
      rowLayout: null,
      inserterTier: null,
      customInputs: [],
    });
  });
});

describe("readUrlState — legacy query string", () => {
  it("still decodes the old `?item=...` form", () => {
    setUrl("?item=iron-plate&rate=5&machine=assembling-machine-3&in=iron-ore,copper-ore");
    const s = readUrlState();
    expect(s.item).toBe("iron-plate");
    expect(s.rate).toBe(5);
    expect(s.machines.crafting).toBe("assembling-machine-3");
    expect(s.inputs).toEqual(["iron-ore", "copper-ore"]);
  });

  it("normalises the deprecated P1 strategy alias", () => {
    setUrl("?item=advanced-circuit&rate=5&strategy=partitioned-per-consumer");
    expect(readUrlState().strategy).toBe("partitioned-decomposed");
  });

  it("decodes the full-word `?inserter_tier=` form", () => {
    setUrl("?item=advanced-circuit&rate=5&inserter_tier=regular");
    expect(readUrlState().inserterTier).toBe("regular");
  });

  it("rejects an unknown `?inserter_tier=` value", () => {
    setUrl("?item=advanced-circuit&rate=5&inserter_tier=bogus");
    expect(readUrlState().inserterTier).toBeNull();
  });
});

describe("writeUrlState → readUrlState round-trip", () => {
  function roundTrip(state: FormState): FormState {
    writeUrlState(state);
    return readUrlState();
  }

  it("default state collapses to a bare URL", () => {
    const state = makeState({
      machines: { crafting: DEFAULT_MACHINES.crafting },
    });
    writeUrlState(state);
    expect(window.location.hash).toBe("");
    expect(window.location.search).toBe("");
  });

  it("survives a typical 'item + rate' state", () => {
    const state = makeState({
      item: "iron-plate",
      rate: 5,
      machines: { crafting: DEFAULT_MACHINES.crafting },
    });
    const back = roundTrip(state);
    expect(back.item).toBe(state.item);
    expect(back.rate).toBe(state.rate);
    // machine matches default → omitted from URL, reader returns empty map,
    // sidebar derives from item.
    expect(back.machines).toEqual({});
    expect(back.inputs).toEqual(DEFAULT_CHECKED_INPUTS);
  });

  it("survives a fully-specified state with explicit inputs and belt", () => {
    const state = makeState({
      item: "advanced-circuit",
      rate: 5,
      machines: { crafting: "assembling-machine-1" },
      inputs: ["iron-ore", "copper-ore", "coal", "water", "crude-oil"],
      belt: "fast-transport-belt",
    });
    const back = roundTrip(state);
    expect(back.item).toBe(state.item);
    expect(back.rate).toBe(state.rate);
    expect(back.machines.crafting).toBe(state.machines.crafting);
    expect(back.inputs).toEqual(state.inputs);
    expect(back.belt).toBe(state.belt);
  });

  it("survives strategy + row layout + inserter tier + custom inputs", () => {
    const state = makeState({
      item: "processing-unit",
      rate: 2,
      machines: { crafting: "assembling-machine-3" },
      strategy: "partitioned-decomposed",
      rowLayout: "horizontal-stack",
      inserterTier: "regular",
      customInputs: ["iron-plate", "copper-plate"],
    });
    const back = roundTrip(state);
    expect(back.strategy).toBe("partitioned-decomposed");
    expect(back.rowLayout).toBe("horizontal-stack");
    expect(back.inserterTier).toBe("regular");
    expect(back.customInputs).toEqual(["iron-plate", "copper-plate"]);
  });

  it("stack (default) inserter tier is omitted from the URL", () => {
    const state = makeState({
      item: "iron-gear-wheel",
      rate: 7,
      machines: { crafting: DEFAULT_MACHINES.crafting },
      inserterTier: null,
    });
    writeUrlState(state);
    expect(window.location.hash).toBe("#/l/igw/7");
  });

  it("survives a non-default smelting machine via extras", () => {
    const state = makeState({
      item: "iron-plate",
      rate: 5,
      machines: { smelting: "stone-furnace" },
    });
    const back = roundTrip(state);
    expect(back.machines.smelting).toBe("stone-furnace");
    // Crafting unspecified → reader leaves it absent.
    expect(back.machines.crafting).toBeUndefined();
  });

  it("trims trailing skip slots in the emitted URL", () => {
    writeUrlState(
      makeState({
        item: "iron-gear-wheel",
        rate: 7,
        machines: { crafting: DEFAULT_MACHINES.crafting },
      }),
    );
    // No machine/inputs/belt slots written when they're at default —
    // makes shared URLs read cleanly.
    expect(window.location.hash).toBe("#/l/igw/7");
  });
});

describe("urlHasGeneratorState", () => {
  it("returns false for a bare URL", () => {
    expect(urlHasGeneratorState()).toBe(false);
  });
  it("recognises the new hash form", () => {
    setUrl("#/l/igw/10");
    expect(urlHasGeneratorState()).toBe(true);
  });
  it("recognises the legacy query form", () => {
    setUrl("?item=iron-plate&rate=5");
    expect(urlHasGeneratorState()).toBe(true);
  });
});
