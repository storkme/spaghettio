import { Assets, Cache, Container, Graphics, Sprite, Texture } from "pixi.js";
import type { LayoutResult, PlacedEntity, EntityDirection } from "../engine";
import {
  buildBeltGraph,
  traceBeltNetwork,
  findAdjacentInserters,
  findAdjacentMachines,
  drawBeltNetworkOverlay,
  type BeltGraph,
} from "./beltGraph";

export const TILE_PX = 32;

// Colors sourced from src/visualize.py _MACHINE_COLORS / _INFRA_COLORS
const MACHINE_COLORS: Record<string, number> = {
  "assembling-machine-1": 0x5a6e82,
  "assembling-machine-2": 0x4a6278,
  "assembling-machine-3": 0x3a526a,
  "stone-furnace": 0x8a6040,
  "steel-furnace": 0x7a5030,
  "electric-furnace": 0x6a5a80,
  "chemical-plant": 0x3a7a50,
  "oil-refinery": 0x5a3a8a,
  centrifuge: 0x3a7a80,
  lab: 0x4a6a50,
  "rocket-silo": 0x4a4a6a,
  foundry: 0x8a6a30,
  "electromagnetic-plant": 0x2a5a9a,
  "cryogenic-plant": 0x4a7a8a,
  biochamber: 0x4a7a3a,
  biolab: 0x3a6a5a,
  recycler: 0x6a5a4a,
  crusher: 0x5a4a3a,
  beacon: 0x4a6080,
  "storage-tank": 0x4a6a5a,
  "big-electric-pole": 0x8b6914,
  substation: 0x6a6a8b,
  "electric-mining-drill": 0x7a6a30,
};
const DEFAULT_MACHINE_COLOR = 0x4a5a6a;

// Belt / underground belt / splitter tier colors: [base, chevron].
// Chevron values are desaturated ~30% toward each tier's RGB mean so they
// read as a quieter directional cue against the belt body.
const BELT_COLORS: Record<string, [number, number]> = {
  "transport-belt": [0xa89030, 0xd3c885],
  "fast-transport-belt": [0xb03030, 0xde7070],
  "express-transport-belt": [0x3070b0, 0x83b0dc],
  "underground-belt": [0xa89030, 0xd3c885],
  "fast-underground-belt": [0xb03030, 0xde7070],
  "express-underground-belt": [0x3070b0, 0x83b0dc],
  splitter: [0xa89030, 0xd3c885],
  "fast-splitter": [0xb03030, 0xde7070],
  "express-splitter": [0x3070b0, 0x83b0dc],
};

const INSERTER_COLORS: Record<string, number> = {
  inserter: 0x6a8e3e,
  "fast-inserter": 0x4a90d0,
  "long-handed-inserter": 0xd04040,
};

const PIPE_COLOR = 0x8a8a8a;
const PIPE_TO_GROUND_COLOR = 0x6a6a6a;
const PIPE_BG = 0x1f1f1f;
const POLE_COLOR = 0xc0a030;
const POLE_BG = 0x2a2510;

// Item-to-color mapping for visual debugging.
//
// Raw palette below is more saturated than we want on-canvas — items
// should read as "tinted grey", not "bold colour", so belts tile into
// a readable backdrop instead of competing with overlays and chevrons.
// At module load each entry is pushed toward perceptual grey by
// `ITEM_SATURATION`; tweak that knob to dial the whole palette up/down.
const ITEM_SATURATION = 0.35;

const ITEM_PALETTE_RAW: Record<string, number> = {
  "iron-plate": 0x9a9a9a,
  "copper-plate": 0xd07840,
  "iron-gear-wheel": 0x707070,
  "copper-cable": 0xe06020,
  "electronic-circuit": 0x50c050,
  "advanced-circuit": 0xc05050,
  "processing-unit": 0x5050c0,
  "plastic-bar": 0x8080a0,
  "steel-plate": 0x707888,
  "iron-ore": 0xa07070,
  "copper-ore": 0xd08060,
  "coal": 0x404040,
  "stone": 0xa09070,
  "sulfur": 0xc0c040,
  "crude-oil": 0x303050,
  "water": 0x4060b0,
  "petroleum-gas": 0xa0a060,
  "light-oil": 0xa0a0b0,
  "heavy-oil": 0x705040,
  "sulfuric-acid": 0xb0b030,
  "lubricant": 0x60b060,
};

/** Blend a hex colour toward its Rec.709 luma grey. factor=1 keeps the
 *  colour as-is; factor=0 collapses to pure grey. */
function desaturate(hex: number, factor: number): number {
  const r = (hex >> 16) & 0xff;
  const g = (hex >> 8) & 0xff;
  const b = hex & 0xff;
  const y = 0.21 * r + 0.72 * g + 0.07 * b;
  const nr = Math.round(y + (r - y) * factor);
  const ng = Math.round(y + (g - y) * factor);
  const nb = Math.round(y + (b - y) * factor);
  return (nr << 16) | (ng << 8) | nb;
}

const ITEM_PALETTE: Record<string, number> = Object.fromEntries(
  Object.entries(ITEM_PALETTE_RAW).map(([k, v]) => [k, desaturate(v, ITEM_SATURATION)]),
);

function hslToHex(h: number, s: number, l: number): number {
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => {
    const k = (n + h * 12) % 12;
    return Math.round((l - a * Math.max(-1, Math.min(k - 3, 9 - k, 1))) * 255);
  };
  return (f(0) << 16) | (f(8) << 8) | f(4);
}

let itemColoringEnabled = true;
export function setItemColoring(enabled: boolean): void { itemColoringEnabled = enabled; }

/** Per-recipe input/output flows for machine overlays. */
interface RecipeFlows {
  inputs: { item: string; rate: number }[];
  outputs: { item: string; rate: number }[];
  machineCount: number;
}
let recipeFlowMap: Map<string, RecipeFlows> = new Map();

/** Look up per-machine flows for a recipe (returns undefined if not set). */
export function getRecipeFlows(recipe: string): RecipeFlows | undefined {
  return recipeFlowMap.get(recipe);
}

/** Call before renderLayout to provide solver result data for machine overlays. */
export function setRecipeFlows(machines: { recipe: string; count: number; inputs: { item: string; rate: number }[]; outputs: { item: string; rate: number }[] }[]): void {
  recipeFlowMap = new Map();
  for (const m of machines) {
    // Solver rates are already per-machine — use as-is.
    recipeFlowMap.set(m.recipe, {
      inputs: m.inputs.map(f => ({ item: f.item, rate: f.rate })),
      outputs: m.outputs.map(f => ({ item: f.item, rate: f.rate })),
      machineCount: Math.ceil(m.count),
    });
  }
}

export function itemColor(item: string | undefined): number {
  if (!itemColoringEnabled) return 0x777777;
  if (!item) return 0x666666;
  if (item in ITEM_PALETTE) return ITEM_PALETTE[item];
  let h = 0;
  for (let i = 0; i < item.length; i++) h = (((h << 5) - h) + item.charCodeAt(i)) | 0;
  const hue = (Math.abs(h) % 30) * 12;
  return hslToHex(hue / 360, 0.2, 0.48);
}

// [width, height] in tiles for multi-tile entities
export const MACHINE_SIZES: Record<string, [number, number]> = {
  "assembling-machine-1": [3, 3],
  "assembling-machine-2": [3, 3],
  "assembling-machine-3": [3, 3],
  "chemical-plant": [3, 3],
  "oil-refinery": [5, 5],
  "electric-furnace": [3, 3],
  "steel-furnace": [2, 2],
  "stone-furnace": [2, 2],
  centrifuge: [3, 3],
  lab: [3, 3],
  "rocket-silo": [9, 9],
  foundry: [5, 5],
  biochamber: [3, 3],
  biolab: [5, 5],
  "electromagnetic-plant": [4, 4],
  "cryogenic-plant": [5, 5],
  recycler: [2, 4],
  crusher: [2, 3],
  beacon: [3, 3],
  "storage-tank": [3, 3],
  "big-electric-pole": [2, 2],
  substation: [2, 2],
  "electric-mining-drill": [3, 3],
};

/** Convert a kebab-case slug to a display name: "assembling-machine-3" → "Assembling Machine 3" */
export function niceName(slug: string): string {
  return slug
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

// Entity-type sets derived from the lookup tables where possible
export const MACHINE_ENTITIES = new Set(Object.keys(MACHINE_SIZES));
export const INSERTER_ENTITIES = new Set(Object.keys(INSERTER_COLORS));
// Derived from BELT_COLORS keys by tier prefix
export const BELT_ENTITIES = new Set(
  Object.keys(BELT_COLORS).filter((k) => !k.includes("underground") && !k.includes("splitter"))
);
export const UG_BELT_ENTITIES = new Set(Object.keys(BELT_COLORS).filter((k) => k.includes("underground")));
export const SPLITTER_ENTITIES = new Set(Object.keys(BELT_COLORS).filter((k) => k.includes("splitter")));
export const PIPE_ENTITIES = new Set(["pipe", "pipe-to-ground"]);
export const POLE_ENTITIES = new Set(["medium-electric-pole", "small-electric-pole"]);

// Direction helpers

function dirAngle(dir?: EntityDirection): number {
  switch (dir) {
    case "East": return Math.PI / 2;
    case "South": return Math.PI;
    case "West": return (3 * Math.PI) / 2;
    default: return 0;
  }
}

function dirVec(dir?: EntityDirection): [number, number] {
  switch (dir) {
    case "East": return [1, 0];
    case "South": return [0, 1];
    case "West": return [-1, 0];
    default: return [0, -1];
  }
}

/** Companion tile offset for a splitter's 2×1 footprint. */
export function splitterCompanionOffset(dir?: EntityDirection): [number, number] {
  switch (dir) {
    case "South":
    case "North":
      return [1, 0]; // 2 wide horizontally
    case "East":
    case "West":
      return [0, 1]; // 2 tall vertically
    default:
      return [1, 0];
  }
}

/** Belt turn info: the perpendicular feed direction relative to our flow. */
interface BeltTurn {
  /** "cw" = feeder rotated clockwise from our direction (e.g. we go East, feeder comes from North). */
  turn: "cw" | "ccw";
}

/** Detect if belt `e` is a 90° turn: exactly one perpendicular feeder, no straight feeder. */
function detectBeltTurn(e: PlacedEntity, tileMap: Map<string, PlacedEntity>): BeltTurn | null {
  const d = e.direction ?? "North";
  const [mdx, mdy] = dirVec(d);
  let hasStraightFeeder = false;
  let perpFeeder: BeltTurn | null = null;

  for (const [dx, dy] of [[0, -1], [1, 0], [0, 1], [-1, 0]] as [number, number][]) {
    const nx = (e.x ?? 0) + dx;
    const ny = (e.y ?? 0) + dy;
    const nb = tileMap.get(`${nx},${ny}`);
    if (!nb) continue;
    const feeds =
      BELT_ENTITIES.has(nb.name) ||
      (UG_BELT_ENTITIES.has(nb.name) && nb.io_type === "output") ||
      SPLITTER_ENTITIES.has(nb.name);
    if (!feeds) continue;
    const [ndx, ndy] = dirVec(nb.direction);
    // Does neighbour's flow point at us?
    // For splitters registered at companion tiles, use the actual tile
    // position (nx, ny) instead of the anchor position (nb.x, nb.y).
    const nbFlowX = SPLITTER_ENTITIES.has(nb.name) ? nx : (nb.x ?? 0);
    const nbFlowY = SPLITTER_ENTITIES.has(nb.name) ? ny : (nb.y ?? 0);
    if (nbFlowX + ndx !== (e.x ?? 0) || nbFlowY + ndy !== (e.y ?? 0)) continue;
    if (nb.direction === d) {
      hasStraightFeeder = true;
    } else {
      // Cross product of feeder flow × our flow: positive = cw, negative = ccw
      const cross = ndx * mdy - ndy * mdx;
      if (cross !== 0) perpFeeder = { turn: cross > 0 ? "cw" : "ccw" };
    }
  }
  return perpFeeder && !hasStraightFeeder ? perpFeeder : null;
}

/** Darken a packed 0xRRGGBB color by `factor` (0 = black, 1 = unchanged). */
function darken(color: number, factor: number): number {
  const r = Math.round(((color >> 16) & 0xff) * factor);
  const gr = Math.round(((color >> 8) & 0xff) * factor);
  const b = Math.round((color & 0xff) * factor);
  return (r << 16) | (gr << 8) | b;
}

// Draw functions

const TILE_RADIUS = 3;

/** Dark body shared by all belt tiers (like the Factorio belt frame). */
const BELT_BODY = 0x3a3a3a;
const BELT_BORDER = 0x555555;

/** Visible belt occupies 90% of the tile, centred — leaves a thin gutter
 * on either side so the bus reads as discrete lanes rather than a
 * seamless solid. `BELT_INSET` is the per-side gap in px. */
const BELT_SCALE = 0.9;
const BELT_INSET = (TILE_PX * (1 - BELT_SCALE)) / 2;
const BELT_SIZE = TILE_PX * BELT_SCALE;

export function drawBelt(entity: PlacedEntity, turn: BeltTurn | null): Graphics {
  const g = new Graphics();
  const s = TILE_PX;
  const bs = BELT_SIZE;
  const [, chev] = BELT_COLORS[entity.name] ?? [0xa89030, 0xe0d070];
  const laneColor = itemColor(entity.carries);

  if (turn) {
    drawBeltCorner(g, s, chev, entity.direction, turn, laneColor);
  } else {
    // Dark belt body with subtle border, inset so a gutter frames every tile.
    g.rect(BELT_INSET, BELT_INSET, bs, bs).fill(BELT_BODY);
    g.setStrokeStyle({ width: 1, color: BELT_BORDER, alignment: 0 });
    g.rect(BELT_INSET, BELT_INSET, bs, bs).stroke();

    const rg = new Graphics();
    rg.x = s / 2;
    rg.y = s / 2;
    rg.rotation = dirAngle(entity.direction);

    // Lane stripes: left lane = left half, right lane = right half.
    rg.rect(-bs / 2, -bs / 2, bs / 2 - 1, bs).fill({ color: laneColor, alpha: 0.45 });
    rg.rect(1, -bs / 2, bs / 2 - 1, bs).fill({ color: laneColor, alpha: 0.45 });
    // Dark centre divider
    rg.rect(-1, -bs / 2, 2, bs).fill(0x0a0a0a);

    addBeltChevrons(rg, bs, chev);
    g.addChild(rg);
  }

  return g;
}

/** Draw a corner belt: dark body curving along the 90° path + 3 chevrons. */
function drawBeltCorner(
  g: Graphics,
  s: number,
  chev: number,
  direction: EntityDirection | undefined,
  turn: BeltTurn,
  laneColor: number,
): void {
  const cg = new Graphics();
  cg.x = s / 2;
  cg.y = s / 2;
  cg.rotation = dirAngle(direction);
  cg.scale.set(BELT_SCALE); // match straight-belt 90%-centred footprint

  const r = s / 2;
  const sign = turn.turn === "cw" ? 1 : -1;
  const cornerX = sign * r;
  const cornerY = -r;

  // Quarter-disc body centred at the inside corner, radius s.
  // Covers the whole tile except the outer corner, which falls beyond the arc.
  const startA = turn.turn === "ccw" ? 0 : Math.PI / 2;
  const endA = turn.turn === "ccw" ? Math.PI / 2 : Math.PI;
  cg.moveTo(cornerX, cornerY)
    .arc(cornerX, cornerY, s, startA, endA, false)
    .closePath()
    .fill(BELT_BODY);
  cg.setStrokeStyle({ width: 1, color: BELT_BORDER, alignment: 0 });
  cg.moveTo(cornerX, cornerY)
    .arc(cornerX, cornerY, s, startA, endA, false)
    .closePath()
    .stroke();

  // Lane stripes: inner arc sector + outer annular sector, matching straight belt alpha.
  const rMid = s * 0.5;
  const rDiv = 1.5; // divider half-width px

  // Inner lane: pie sector from corner to rMid - divider
  cg.moveTo(cornerX, cornerY)
    .arc(cornerX, cornerY, rMid - rDiv, startA, endA, false)
    .closePath()
    .fill({ color: laneColor, alpha: 0.45 });

  // Outer lane: annular sector from rMid + divider to s
  const csA = Math.cos(startA), snA = Math.sin(startA);
  const csE = Math.cos(endA), snE = Math.sin(endA);
  cg.moveTo(cornerX + (rMid + rDiv) * csA, cornerY + (rMid + rDiv) * snA)
    .lineTo(cornerX + s * csA, cornerY + s * snA)
    .arc(cornerX, cornerY, s, startA, endA, false)
    .lineTo(cornerX + (rMid + rDiv) * csE, cornerY + (rMid + rDiv) * snE)
    .arc(cornerX, cornerY, rMid + rDiv, endA, startA, true)
    .closePath()
    .fill({ color: laneColor, alpha: 0.45 });

  // 2 chevrons projected onto the arc: tip sits on the centreline, arms splay
  // to outer/inner radii along an angular offset BEHIND the tip (opposite flow).
  const chSize = s * 0.22;
  const lineW = Math.max(1, s * 0.07);
  const chevR = s * 0.5;
  const dA = chSize / chevR;
  const rOuter = chevR + chSize;
  const rInner = chevR - chSize;
  cg.setStrokeStyle({ width: lineW, color: chev, cap: "round", join: "round" });
  const aStart = Math.PI / 2;
  const aEnd = turn.turn === "cw" ? Math.PI : 0;
  for (const frac of [0.6]) {
    const aTip = aStart + frac * (aEnd - aStart);
    const aBack = turn.turn === "cw" ? aTip - dA : aTip + dA;
    const tipX = cornerX + chevR * Math.cos(aTip);
    const tipY = cornerY + chevR * Math.sin(aTip);
    const outerX = cornerX + rOuter * Math.cos(aBack);
    const outerY = cornerY + rOuter * Math.sin(aBack);
    const innerX = cornerX + rInner * Math.cos(aBack);
    const innerY = cornerY + rInner * Math.sin(aBack);
    cg.moveTo(outerX, outerY).lineTo(tipX, tipY).lineTo(innerX, innerY).stroke();
  }

  g.addChild(cg);
}

/** 2 direction chevrons stacked along the flow axis (local -y). */
function addBeltChevrons(g: Graphics, s: number, chevColor: number): void {
  const chevSize = s * 0.22;
  g.setStrokeStyle({ width: Math.max(1, s * 0.07), color: chevColor, cap: "round", join: "round" });
  for (const oy of [-s * 0.22, s * 0.22]) {
    g.moveTo(-chevSize, oy + chevSize * 0.5)
      .lineTo(0, oy - chevSize * 0.5)
      .lineTo(chevSize, oy + chevSize * 0.5)
      .stroke();
  }
}

function drawUndergroundBelt(entity: PlacedEntity): Graphics {
  const g = new Graphics();
  const s = TILE_PX;
  const [, chev] = BELT_COLORS[entity.name] ?? [0xa89030, 0xe0d070];
  const isInput = entity.io_type === "input";
  const half = s / 2;

  // In local coords (rotation=0 = North): flow direction = -y.
  // Input UG:  items flow in (-y), tunnel goes in flow direction. Open mouth at +y.
  // Output UG: items emerge in flow direction (-y), tunnel comes from +y. Open mouth at -y.
  // `surfaceSign` = +1 if the open mouth faces +y (input), -1 if it faces -y (output).
  const surfaceSign = isInput ? 1 : -1;

  const m = new Graphics();
  m.x = half;
  m.y = half;
  m.rotation = dirAngle(entity.direction);

  // Surface half: a trapezoid, wide at the open mouth (matches belt width)
  // and narrow where it meets the buried half. Communicates "items funnel
  // into/out of the ground" without a heavy frame around the tunnel.
  const laneC = itemColor(entity.carries);
  const mouthHalf = BELT_SIZE / 2;            // matches surface-belt 90% width
  const throatHalf = s * 0.25;                // ~half the tile width at the divider
  const mouthY = surfaceSign * half;
  const throatY = 0;
  m.moveTo(-mouthHalf, mouthY)
    .lineTo( mouthHalf, mouthY)
    .lineTo( throatHalf, throatY)
    .lineTo(-throatHalf, throatY)
    .closePath()
    .fill({ color: laneC, alpha: 0.7 });
  m.setStrokeStyle({ width: 1, color: BELT_BORDER, alpha: 0.8 });
  m.moveTo(-mouthHalf, mouthY)
    .lineTo(-throatHalf, throatY)
    .lineTo( throatHalf, throatY)
    .lineTo( mouthHalf, mouthY)
    .stroke();

  // Flow-direction triangle, centred on the tile. Sits on the surface side
  // so it points *out* of the ground for outputs and *into* the ground for
  // inputs — reads as motion, not a labelled arrow.
  const arrW = s * 0.38;
  const arrH = s * 0.3;
  const arrCy = surfaceSign * s * 0.22;
  const arrTipY = arrCy - arrH / 2; // tip always toward -y (flow)
  const arrBaseY = arrCy + arrH / 2;
  m.moveTo(0, arrTipY)
    .lineTo(arrW / 2, arrBaseY)
    .lineTo(-arrW / 2, arrBaseY)
    .closePath()
    .fill(chev);

  g.addChild(m);
  return g;
}

function drawSplitter(entity: PlacedEntity): Graphics {
  const g = new Graphics();
  const [base, chev] = BELT_COLORS[entity.name] ?? [0xa89030, 0xe0d070];
  // Splitters are 2 tiles wide perpendicular to travel direction
  const isNS = entity.direction === "North" || entity.direction === "South";
  const pw = isNS ? TILE_PX * 2 - 1 : TILE_PX - 1;
  const ph = isNS ? TILE_PX - 1 : TILE_PX * 2 - 1;
  const cell = isNS ? pw / 2 : ph / 2;
  const barThick = Math.max(2, Math.min(pw, ph) * 0.18);

  g.roundRect(0, 0, pw, ph, TILE_RADIUS).fill(base);
  // Item tint overlay
  g.roundRect(0, 0, pw, ph, TILE_RADIUS).fill({ color: itemColor(entity.carries), alpha: 0.3 });
  if (isNS) {
    g.rect(cell - barThick / 2, 0, barThick, ph).fill(darken(base, 0.5));
  } else {
    g.rect(0, cell - barThick / 2, pw, barThick).fill(darken(base, 0.5));
  }

  const angle = dirAngle(entity.direction);
  const chevSize = cell * 0.25;
  const lineW = Math.max(1, cell * 0.12);
  for (let half = 0; half < 2; half++) {
    const hcx = isNS ? cell * half + cell / 2 : pw / 2;
    const hcy = isNS ? ph / 2 : cell * half + cell / 2;
    const cg = new Graphics();
    cg.x = hcx;
    cg.y = hcy;
    cg.rotation = angle;
    cg.setStrokeStyle({ width: lineW, color: chev, cap: "round" });
    cg.moveTo(-chevSize, chevSize * 0.5).lineTo(0, -chevSize * 0.5).lineTo(chevSize, chevSize * 0.5).stroke();
    g.addChild(cg);
  }

  return g;
}

function drawInserter(entity: PlacedEntity): Graphics {
  const g = new Graphics();
  const s = TILE_PX - 1;
  const armColor = entity.carries ? itemColor(entity.carries) : (INSERTER_COLORS[entity.name] ?? 0x6a8e3e);

  g.roundRect(0, 0, s, s, TILE_RADIUS).fill(0x2a3a2a);

  const armG = new Graphics();
  armG.x = s / 2;
  armG.y = s / 2;
  armG.rotation = dirAngle(entity.direction);

  armG.circle(0, s * 0.2, s * 0.15).fill(0x444444);

  const armW = Math.max(1.5, s * 0.12);
  armG.setStrokeStyle({ width: armW, color: armColor, cap: "round" });
  armG.moveTo(0, s * 0.2).lineTo(0, -s * 0.35).stroke();

  const clawY = -s * 0.35;
  const clawW = s * 0.18;
  armG.moveTo(-clawW, clawY - clawW * 0.6)
    .lineTo(0, clawY)
    .lineTo(clawW, clawY - clawW * 0.6)
    .stroke();

  g.addChild(armG);
  return g;
}

// Pipe connection bitmask: N=1, E=2, S=4, W=8
const CONN_N = 1, CONN_E = 2, CONN_S = 4, CONN_W = 8;

/** Returns the surface-side unit vector of a pipe-to-ground entity.
 *  `direction` is the tunnel direction; the visible surface mouth is
 *  always opposite, matching `drawPipe`'s stub rendering. `io_type` is
 *  our internal pair-routing label — it doesn't affect which side is
 *  the surface. */
function ptgSurfaceDelta(entity: PlacedEntity): [number, number] {
  const [dx, dy] = dirVec(entity.direction);
  return [-dx, -dy];
}

/** Whether a regular pipe at (px,py) should connect to neighbour `nb`
 *  reached via offset (dx,dy). Pipes connect to all adjacent pipes, but
 *  only on a pipe-to-ground's surface side — not on its tunnel side.
 *  `(dx, dy)` is the offset from the pipe to the neighbour tile. */
function pipeConnectsToNeighbour(nb: PlacedEntity, dx: number, dy: number): boolean {
  if (nb.name === "pipe") return true;
  if (nb.name === "pipe-to-ground") {
    // Pipe is on PTG's surface side iff the offset from PTG back to
    // pipe (= -(dx,dy)) equals the PTG's surface delta.
    const [sx, sy] = ptgSurfaceDelta(nb);
    return -dx === sx && -dy === sy;
  }
  return false;
}

function drawPipe(entity: PlacedEntity, connections: number): Graphics {
  const g = new Graphics();
  const s = TILE_PX - 1;
  const isGround = entity.name === "pipe-to-ground";
  const pipeColor = isGround ? PIPE_TO_GROUND_COLOR : PIPE_COLOR;

  g.roundRect(0, 0, s, s, TILE_RADIUS).fill(PIPE_BG);

  const cx = s / 2;
  const cy = s / 2;
  const pipeWidth = Math.max(2, s * 0.4);

  if (isGround) {
    // pipe-to-ground: stub toward surface connection (opposite of underground direction)
    g.setStrokeStyle({ width: pipeWidth, color: pipeColor, cap: "round" });
    const [dx, dy] = dirVec(entity.direction);
    g.moveTo(cx, cy).lineTo(cx - dx * s / 2, cy - dy * s / 2).stroke();
    g.circle(cx, cy, pipeWidth * 0.4).fill(pipeColor);
    g.circle(cx, cy, pipeWidth * 0.25).fill(PIPE_BG);
  } else if (connections === 0) {
    // Isolated pipe: just a center dot
    g.circle(cx, cy, pipeWidth * 0.4).fill(pipeColor);
  } else {
    const hasN = !!(connections & CONN_N);
    const hasE = !!(connections & CONN_E);
    const hasS = !!(connections & CONN_S);
    const hasW = !!(connections & CONN_W);
    const count = (hasN ? 1 : 0) + (hasE ? 1 : 0) + (hasS ? 1 : 0) + (hasW ? 1 : 0);

    g.setStrokeStyle({ width: pipeWidth, color: pipeColor, cap: "round" });

    if (count === 1) {
      // Dead-end stub
      if (hasN) g.moveTo(cx, cy).lineTo(cx, 0).stroke();
      else if (hasE) g.moveTo(cx, cy).lineTo(s, cy).stroke();
      else if (hasS) g.moveTo(cx, cy).lineTo(cx, s).stroke();
      else g.moveTo(cx, cy).lineTo(0, cy).stroke();
      g.circle(cx, cy, pipeWidth * 0.4).fill(pipeColor);
    } else if (hasN && hasS && !hasE && !hasW) {
      // Straight N-S
      g.moveTo(cx, 0).lineTo(cx, s).stroke();
    } else if (hasE && hasW && !hasN && !hasS) {
      // Straight E-W
      g.moveTo(0, cy).lineTo(s, cy).stroke();
    } else if (count === 2) {
      // Corner — smooth quadratic curve bulging toward the tile centre
      // (convex w.r.t. the bend's diagonal). Controlling at the outer
      // corner used to make the curve hug the outer edge, reading as a
      // backwards "concave" bend.
      if (hasN && hasE) {
        g.moveTo(cx, 0).quadraticCurveTo(cx, cy, s, cy).stroke();
      } else if (hasE && hasS) {
        g.moveTo(s, cy).quadraticCurveTo(cx, cy, cx, s).stroke();
      } else if (hasS && hasW) {
        g.moveTo(cx, s).quadraticCurveTo(cx, cy, 0, cy).stroke();
      } else { // W+N
        g.moveTo(0, cy).quadraticCurveTo(cx, cy, cx, 0).stroke();
      }
    } else if (count === 3) {
      // T-junction: straight through the two opposite + stub for the third
      if (!hasW) { g.moveTo(cx, 0).lineTo(cx, s).stroke(); g.moveTo(cx, cy).lineTo(s, cy).stroke(); }
      else if (!hasS) { g.moveTo(0, cy).lineTo(s, cy).stroke(); g.moveTo(cx, cy).lineTo(cx, 0).stroke(); }
      else if (!hasE) { g.moveTo(cx, 0).lineTo(cx, s).stroke(); g.moveTo(cx, cy).lineTo(0, cy).stroke(); }
      else { g.moveTo(0, cy).lineTo(s, cy).stroke(); g.moveTo(cx, cy).lineTo(cx, s).stroke(); }
    } else {
      // Cross (4 connections)
      g.moveTo(cx, 0).lineTo(cx, s).stroke();
      g.moveTo(0, cy).lineTo(s, cy).stroke();
    }
  }

  return g;
}

function drawPole(): Graphics {
  const g = new Graphics();
  const s = TILE_PX - 1;

  g.roundRect(0, 0, s, s, TILE_RADIUS).fill(POLE_BG);

  const cx = s / 2;
  const cy = s / 2;
  const armLen = s * 0.38;
  const armW = Math.max(1.5, s * 0.2);

  g.rect(cx - armW / 2, cy - armLen, armW, armLen * 2).fill(POLE_COLOR);
  g.rect(cx - armLen, cy - armW / 2, armLen * 2, armW).fill(POLE_COLOR);
  g.circle(cx, cy, armW * 0.6).fill(0xe0c040);

  return g;
}

function drawMachine(entity: PlacedEntity): Graphics {
  const g = new Graphics();
  const [tw, th] = MACHINE_SIZES[entity.name] ?? [1, 1];
  const pw = tw * TILE_PX - 1;
  const ph = th * TILE_PX - 1;

  // Dotted outline showing the tile footprint
  g.setStrokeStyle({ width: 1, color: 0xaaaaaa, alpha: 0.4 });
  // Pixi v8 doesn't support native dash — draw a dotted rect manually
  const dash = 3;
  const gap = 3;
  for (const [x0, y0, x1, y1] of [
    [0, 0, pw, 0], [pw, 0, pw, ph], [pw, ph, 0, ph], [0, ph, 0, 0],
  ] as [number, number, number, number][]) {
    const len = Math.sqrt((x1 - x0) ** 2 + (y1 - y0) ** 2);
    const dx = (x1 - x0) / len;
    const dy = (y1 - y0) / len;
    let d = 0;
    while (d < len) {
      const end = Math.min(d + dash, len);
      g.moveTo(x0 + dx * d, y0 + dy * d).lineTo(x0 + dx * end, y0 + dy * end).stroke();
      d = end + gap;
    }
  }

  // Machine sprite — scaled to crop out the surrounding shadow / overhang
  // padding so the visible machine body fills ~90% of its tile footprint
  // (Factorio entity PNGs include shadow padding outside the footprint;
  // higher spriteScale crops more of that and zooms into the body).
  const spriteScale = 1.8;
  const frameTexture = tryGetTexture(`${import.meta.env.BASE_URL}entity-frames/${entity.name}.png`);
  if (frameTexture) {
    const sprite = new Sprite(frameTexture);
    const baseScale = TILE_PX / ENTITY_FRAME_TILE_PX;
    sprite.scale.set(baseScale * spriteScale);
    // At 1x the sprite fills [0..pw, 0..ph]. At 1.5x it fills [0..pw*1.5, 0..ph*1.5].
    // Shift so the centre stays at (pw/2, ph/2).
    sprite.x = -pw * (spriteScale - 1) / 2;
    sprite.y = -ph * (spriteScale - 1) / 2;
    g.addChild(sprite);
  } else {
    const iconTexture = tryGetTexture(`${import.meta.env.BASE_URL}icons/${entity.name}.png`);
    if (iconTexture) {
      const sprite = new Sprite(iconTexture);
      const iconSize = Math.min(pw, ph) * 0.8 * spriteScale;
      sprite.width = iconSize;
      sprite.height = iconSize;
      sprite.x = (pw - iconSize) / 2;
      sprite.y = (ph - iconSize) / 2;
      g.addChild(sprite);
    } else {
      const color = MACHINE_COLORS[entity.name] ?? DEFAULT_MACHINE_COLOR;
      g.roundRect(2, 2, pw - 4, ph - 4, 3).fill({ color, alpha: 0.5 });
    }
  }

  // Recipe panel removed (Phase 2). Recipe info is conveyed by icons on
  // inserters and pipes. A future hover-tooltip will show rates / recipe detail.

  return g;
}

function drawGenericEntity(): Graphics {
  const g = new Graphics();
  const s = TILE_PX - 1;
  g.rect(0, 0, s, s).fill(0x4a5a6a);
  g.setStrokeStyle({ width: 1, color: 0x000000, alpha: 0.4 });
  g.rect(0, 0, s, s).stroke();
  return g;
}

export function isBeltEntity(name: string): boolean {
  return BELT_ENTITIES.has(name) || UG_BELT_ENTITIES.has(name) || SPLITTER_ENTITIES.has(name);
}

// Pixel-per-tile used when generating entity-frame PNGs (see scripts/extract_entity_frames.py)
const ENTITY_FRAME_TILE_PX = 64;

export async function initEntityIcons(slugs: string[]): Promise<void> {
  const base = import.meta.env.BASE_URL;
  const urls = [
    ...slugs.map((s) => `${base}icons/${s}.png`),
    ...slugs.map((s) => `${base}entity-frames/${s}.png`),
  ];
  // Load individually and ignore 404s — not all slugs have sprites yet
  await Promise.allSettled(urls.map((url) => Assets.load(url)));
}

/** Preload item-icon PNGs for all slugs that may appear as belt/pipe carries values. */
export async function preloadCarriesIcons(slugs: string[]): Promise<void> {
  const base = import.meta.env.BASE_URL;
  await Promise.allSettled(slugs.map((s) => Assets.load(`${base}icons/${s}.png`)));
}

/** Extract every distinct `carries` slug from a list of placed entities.
 * Used to scope the carries-icon preload to just the items that actually
 * appear in the current layout — versus pre-loading every producible item
 * (which gates first paint by seconds on cold dev-server starts). */
export function extractCarriesFromEntities(entities: ReadonlyArray<{ carries?: string | null }>): string[] {
  const out = new Set<string>();
  for (const e of entities) {
    if (e.carries) out.add(e.carries);
  }
  return Array.from(out);
}

/** `Assets.get` warns when called with a key that wasn't loaded — and recipe /
 * entity slugs frequently have no PNG (multi-output recipes, fluid-only outputs,
 * entity frames not yet generated). Hot draw paths call this thousands of times
 * per layout, so the warns add real cost (each one captures a stack trace for
 * the inspector). Check the cache first; callers already null-check the result. */
function tryGetTexture(path: string): Texture | null {
  return Cache.has(path) ? (Assets.get<Texture>(path) ?? null) : null;
}

// Chain highlight controller returned by renderLayout

export interface HighlightController {
  /** Highlight all entities that carry the given item; dim everything else. */
  highlightItem(item: string | null): void;
  /** Highlight the connected belt network (upstream dashed, downstream solid). */
  highlightBeltNetwork(entity: PlacedEntity | null): void;
  /** Reset all entities to full opacity and remove any overlay. */
  clearHighlight(): void;
  /** Get the item chain key for an entity (its `carries` value, or recipe for machines). */
  chainKey(entity: PlacedEntity): string | null;
}

// ---------------------------------------------------------------------------
// Per-entity draw primitives (exported so streaming renderers can commit
// individual entities without re-rendering the full layout).
// ---------------------------------------------------------------------------

/** Fluid port positions per machine, mirroring
 *  `crates/core/src/validate/fluids.rs::fluid_ports`. Each tuple is
 *  `[rel_x, rel_y, type, when]` where `(rel_x, rel_y)` is the pipe-side
 *  tile relative to the machine's top-left, and `when` controls
 *  oil-refinery's mirror-state-dependent ports. Coordinates are
 *  *outside* the machine footprint — they're where a pipe sits to
 *  actually connect. */
type FluidPortDef = readonly [
  rel_x: number,
  rel_y: number,
  type: "input" | "output",
  when: "always" | "default" | "mirror",
];

const FLUID_PORTS: Record<string, readonly FluidPortDef[]> = {
  "assembling-machine-2": [
    [1, -1, "input", "always"],
    [1, 3, "output", "always"],
  ],
  "assembling-machine-3": [
    [1, -1, "input", "always"],
    [1, 3, "output", "always"],
  ],
  "chemical-plant": [
    [0, -1, "input", "always"],
    [2, -1, "input", "always"],
    [0, 3, "output", "always"],
    [2, 3, "output", "always"],
  ],
  "oil-refinery": [
    [1, 5, "input", "default"],
    [3, 5, "input", "default"],
    [0, -1, "output", "default"],
    [2, -1, "output", "default"],
    [4, -1, "output", "default"],
    [1, -1, "input", "mirror"],
    [3, -1, "input", "mirror"],
    [0, 5, "output", "mirror"],
    [2, 5, "output", "mirror"],
    [4, 5, "output", "mirror"],
  ],
};

/** Effective fluid ports for a machine, accounting for `mirror`. */
export function getFluidPorts(machine: PlacedEntity): readonly FluidPortDef[] {
  const all = FLUID_PORTS[machine.name];
  if (!all) return [];
  const mirror = machine.mirror ?? false;
  return all.filter(([, , , when]) =>
    when === "always" || (when === "default" && !mirror) || (when === "mirror" && mirror),
  );
}

/** True if the pipe at world tile `(px, py)` sits at one of `machine`'s
 *  fluid ports. Both input and output ports count — pipes don't
 *  distinguish flow direction at the connection geometry, only at the
 *  recipe level. */
function pipeIsAtFluidPort(px: number, py: number, machine: PlacedEntity): boolean {
  const mx = machine.x ?? 0;
  const my = machine.y ?? 0;
  for (const [rx, ry] of getFluidPorts(machine)) {
    if (mx + rx === px && my + ry === py) return true;
  }
  return false;
}

/** Context for `drawEntityGraphic` — carries the lookups the belt / pipe
 *  renderers need. `tileMap` is used by `detectBeltTurn` (belts) and
 *  pipe-connection detection; `machineByTile` lets pipe-connection
 *  detection look up the owning machine for a tile and validate the
 *  pipe is at one of the machine's fluid ports (only input/output
 *  port positions count as a connection — adjacency at any other
 *  machine tile is not a real fluid link in Factorio). */
export interface DrawContext {
  tileMap: Map<string, PlacedEntity>;
  machineByTile: Map<string, PlacedEntity>;
}

/** Create an empty `DrawContext`. Streaming renderers populate this as
 *  entities are committed. */
export function createDrawContext(): DrawContext {
  return { tileMap: new Map(), machineByTile: new Map() };
}

/** Update a `DrawContext` to include `entity`. Call before drawing its
 *  neighbours (belt turn detection / pipe connections read from the
 *  context). Idempotent. */
export function addEntityToDrawContext(entity: PlacedEntity, ctx: DrawContext): void {
  const x = entity.x ?? 0;
  const y = entity.y ?? 0;
  ctx.tileMap.set(`${x},${y}`, entity);
  if (SPLITTER_ENTITIES.has(entity.name)) {
    const [dx, dy] = splitterCompanionOffset(entity.direction);
    ctx.tileMap.set(`${x + dx},${y + dy}`, entity);
  }
  if (MACHINE_ENTITIES.has(entity.name)) {
    const [w, h] = MACHINE_SIZES[entity.name] ?? [1, 1];
    for (let dy = 0; dy < h; dy++) {
      for (let dx = 0; dx < w; dx++) {
        ctx.machineByTile.set(`${x + dx},${y + dy}`, entity);
      }
    }
  }
}

/** Draw a single entity. Mirrors the dispatch block inside
 *  `renderLayout` — kept in sync by having `renderLayout` call through
 *  this helper. Sets the Graphics' `x`/`y` so callers can just addChild.
 *  Returns the Graphics (single; no entity has multiple top-level
 *  graphics). */
export function drawEntityGraphic(entity: PlacedEntity, ctx: DrawContext): Graphics {
  let g: Graphics;
  if (BELT_ENTITIES.has(entity.name)) {
    g = drawBelt(entity, detectBeltTurn(entity, ctx.tileMap));
  } else if (UG_BELT_ENTITIES.has(entity.name)) {
    g = drawUndergroundBelt(entity);
  } else if (SPLITTER_ENTITIES.has(entity.name)) {
    g = drawSplitter(entity);
  } else if (INSERTER_ENTITIES.has(entity.name)) {
    g = drawInserter(entity);
  } else if (PIPE_ENTITIES.has(entity.name)) {
    let pipeConn = 0;
    if (entity.name === "pipe") {
      const ex = entity.x ?? 0;
      const ey = entity.y ?? 0;
      for (const [dx, dy, bit] of [[0, -1, CONN_N], [1, 0, CONN_E], [0, 1, CONN_S], [-1, 0, CONN_W]] as [number, number, number][]) {
        const key = `${ex + dx},${ey + dy}`;
        const nb = ctx.tileMap.get(key);
        if (nb && pipeConnectsToNeighbour(nb, dx, dy)) {
          pipeConn |= bit;
          continue;
        }
        // Pipe-to-machine: only count as a connection if the pipe sits
        // at one of this machine's dedicated fluid ports. Adjacency at
        // a non-port tile (e.g., a pipe hugging a chemical-plant's
        // mid-edge) is not a real fluid link in Factorio and shouldn't
        // render as connected.
        const machine = ctx.machineByTile.get(key);
        if (machine && pipeIsAtFluidPort(ex, ey, machine)) {
          pipeConn |= bit;
        }
      }
    }
    g = drawPipe(entity, pipeConn);
  } else if (POLE_ENTITIES.has(entity.name)) {
    g = drawPole();
  } else if (MACHINE_ENTITIES.has(entity.name)) {
    g = drawMachine(entity);
  } else {
    g = drawGenericEntity();
  }
  if (entity.quality && entity.quality !== "normal") {
    addQualityBadge(g, entity.quality);
  }
  g.x = (entity.x ?? 0) * TILE_PX;
  g.y = (entity.y ?? 0) * TILE_PX;
  return g;
}

/** Game-style quality tier colors (uncommon green, rare blue, epic
 *  purple, legendary orange). Fallback only — the real in-game badge
 *  icons are preferred (see [`addQualityBadge`]). */
const QUALITY_BADGE_COLORS: Record<string, number> = {
  uncommon: 0x4fca4f,
  rare: 0x4f8bca,
  epic: 0xa64fca,
  legendary: 0xe8a33d,
};

/** The game's own tier badge icons, extracted from the `__quality__`
 *  mod by `scripts/extract_icons.py` into `public/icons/`. "normal" is
 *  deliberately absent — normal entities carry no badge, in-game or
 *  here. */
export const QUALITY_BADGE_SLUGS = [
  "quality-uncommon",
  "quality-rare",
  "quality-epic",
  "quality-legendary",
] as const;

/** Preload the four in-game quality badge textures. Must run before the
 *  first render (same committed-particles caveat as carries icons:
 *  sprites drawn before their texture arrives never pick it up). */
export async function preloadQualityBadgeIcons(): Promise<void> {
  const base = import.meta.env.BASE_URL;
  await Promise.allSettled(
    QUALITY_BADGE_SLUGS.map((s) => Assets.load(`${base}icons/${s}.png`)),
  );
}

/** Badge marking a quality-stamped entity at the footprint's bottom-left,
 *  mirroring the in-game badge position. Prefers the actual game icon
 *  (`icons/quality-<tier>.png`); falls back to a tier-colored diamond if
 *  the texture isn't cached (e.g. cold load race). Only functional
 *  entities carry `quality` (rfp-build-quality functional-only
 *  stamping), so belts stay clean automatically. */
function addQualityBadge(g: Graphics, quality: string): void {
  const bounds = g.getLocalBounds();
  const pad = 1.5;
  const tex = tryGetTexture(`${import.meta.env.BASE_URL}icons/quality-${quality}.png`);
  if (tex) {
    const size = TILE_PX * 0.42;
    const badge = new Sprite(tex);
    badge.width = size;
    badge.height = size;
    badge.x = bounds.minX + pad;
    badge.y = bounds.maxY - size - pad;
    g.addChild(badge);
    return;
  }
  const color = QUALITY_BADGE_COLORS[quality];
  if (color === undefined) return;
  const r = TILE_PX * 0.14;
  const cx = bounds.minX + r + pad;
  const cy = bounds.maxY - r - pad;
  g.poly([cx, cy - r, cx + r, cy, cx, cy + r, cx - r, cy])
    .fill({ color, alpha: 0.95 })
    .stroke({ color: 0x1a1a1a, width: 1, alpha: 0.8 });
}

/** Draw an UG tunnel stripe between a paired UG input and output.
 *  Returns a single Graphics for the stripe (or null if the pair
 *  doesn't form a valid tunnel — not same item, not same direction,
 *  etc.). Scans forward from `input` up to `maxReach` tiles looking
 *  for the paired output. */
export function drawUgTunnelStripe(
  input: PlacedEntity,
  ugEntityMap: Map<string, PlacedEntity>,
  maxReach = 8,
): Graphics | null {
  if (!UG_BELT_ENTITIES.has(input.name) || input.io_type !== "input") return null;
  const [dx, dy] = dirVec(input.direction);
  const x = input.x ?? 0;
  const y = input.y ?? 0;
  for (let dist = 1; dist <= maxReach; dist++) {
    const te = ugEntityMap.get(`${x + dx * dist},${y + dy * dist}`);
    if (!te) continue;
    if (UG_BELT_ENTITIES.has(te.name) && te.name === input.name && te.direction === input.direction && te.io_type === "input") return null;
    if (UG_BELT_ENTITIES.has(te.name) && te.name === input.name && te.direction === input.direction && te.io_type === "output") {
      const [base] = BELT_COLORS[input.name] ?? [0xa89030, 0xe0d070];
      const tg = new Graphics();
      const isHoriz = Math.abs(dx) > 0;
      for (let i = 1; i < dist; i++) {
        const tx = (x + dx * i) * TILE_PX;
        const ty = (y + dy * i) * TILE_PX;
        if (isHoriz) {
          tg.rect(tx, ty + TILE_PX * 0.25, TILE_PX, TILE_PX * 0.5).fill({ color: base, alpha: 0.25 });
        } else {
          tg.rect(tx + TILE_PX * 0.25, ty, TILE_PX * 0.5, TILE_PX).fill({ color: base, alpha: 0.25 });
        }
      }
      return tg;
    }
  }
  return null;
}

// Main renderer

export function renderLayout(
  layout: LayoutResult,
  container: Container,
  // onHover is accepted for call-site compatibility but no longer wired to
  // per-entity Pixi events. Hover detection is now tile-based in main.ts.
  _onHover?: (entity: PlacedEntity | null) => void,
  onSelect?: (entity: PlacedEntity | null) => void,
  onEntityRendered?: (entity: PlacedEntity, graphics: Graphics[]) => void,
  /** Extra entities that are NOT drawn but DO participate in `detectBeltTurn`
   * as perpendicular-feeder candidates. Used by the SAT editor to feed
   * zone-boundary feeders into turn detection without actually rendering
   * them. Leave unset outside the editor. */
  turnFeederHints?: PlacedEntity[],
): HighlightController {
  container.removeChildren();

  // Build tile map for belt turn detection.
  // Splitters occupy 2 tiles (perpendicular to facing), so register both.
  const tileMap = new Map<string, PlacedEntity>();
  for (const e of layout.entities) {
    tileMap.set(`${e.x ?? 0},${e.y ?? 0}`, e);
    if (SPLITTER_ENTITIES.has(e.name)) {
      const [dx, dy] = splitterCompanionOffset(e.direction);
      tileMap.set(`${(e.x ?? 0) + dx},${(e.y ?? 0) + dy}`, e);
    }
  }
  // Phantom feeders: boundary-derived external belts that `detectBeltTurn`
  // inspects but we never draw. Added after real entities so we never
  // overwrite a painted tile with a hint.
  if (turnFeederHints) {
    for (const h of turnFeederHints) {
      const k = `${h.x ?? 0},${h.y ?? 0}`;
      if (!tileMap.has(k)) tileMap.set(k, h);
    }
  }

  // Build map of machine-occupied tiles → owning machine, for
  // pipe-to-machine port-validity checks (pipes connect only at the
  // machine's dedicated fluid ports, not any adjacent tile).
  const machineByTile = new Map<string, PlacedEntity>();
  for (const e of layout.entities) {
    if (MACHINE_ENTITIES.has(e.name)) {
      const [w, h] = MACHINE_SIZES[e.name] ?? [1, 1];
      const ex = e.x ?? 0;
      const ey = e.y ?? 0;
      for (let dy = 0; dy < h; dy++) {
        for (let dx = 0; dx < w; dx++) {
          machineByTile.set(`${ex + dx},${ey + dy}`, e);
        }
      }
    }
  }

  // Draw underground belt tunnels beneath all entities.
  // For each UG input, scan forward for its matching output and draw a
  // semi-transparent stripe through the tiles in between so it's clear
  // what passes under surface entities.
  {
    const ugEntityMap = new Map<string, PlacedEntity>();
    for (const e of layout.entities) {
      if (UG_BELT_ENTITIES.has(e.name)) {
        ugEntityMap.set(`${e.x ?? 0},${e.y ?? 0}`, e);
      }
    }
    const MAX_UG = 8;
    for (const e of layout.entities) {
      if (!UG_BELT_ENTITIES.has(e.name) || e.io_type !== "input") continue;
      const [dx, dy] = dirVec(e.direction);
      const x = e.x ?? 0;
      const y = e.y ?? 0;
      for (let dist = 1; dist <= MAX_UG; dist++) {
        const te = ugEntityMap.get(`${x + dx * dist},${y + dy * dist}`);
        if (!te) continue;
        if (UG_BELT_ENTITIES.has(te.name) && te.name === e.name && te.direction === e.direction && te.io_type === "input") break;
        if (UG_BELT_ENTITIES.has(te.name) && te.name === e.name && te.direction === e.direction && te.io_type === "output") {
          // Draw tunnel stripe through intermediate tiles
          const [base] = BELT_COLORS[e.name] ?? [0xa89030, 0xe0d070];
          const tg = new Graphics();
          const isHoriz = Math.abs(dx) > 0;
          for (let i = 1; i < dist; i++) {
            const tx = (x + dx * i) * TILE_PX;
            const ty = (y + dy * i) * TILE_PX;
            // Width matches the UG-mouth "throat" (~50% tile) so the
            // tunnel reads as a continuous pipe through the pair.
            if (isHoriz) {
              tg.rect(tx, ty + TILE_PX * 0.25, TILE_PX, TILE_PX * 0.5).fill({ color: base, alpha: 0.25 });
            } else {
              tg.rect(tx + TILE_PX * 0.25, ty, TILE_PX * 0.5, TILE_PX).fill({ color: base, alpha: 0.25 });
            }
          }
          container.addChild(tg);
          break;
        }
      }
    }
  }

  // Draw pipe-to-ground tunnel dashes between paired entities — same
  // idea as the UG belt stripe but dashed, to signal underground fluid
  // continuity beneath surface entities.
  {
    const ptgMap = new Map<string, PlacedEntity>();
    for (const e of layout.entities) {
      if (e.name === "pipe-to-ground") {
        ptgMap.set(`${e.x ?? 0},${e.y ?? 0}`, e);
      }
    }
    const MAX_PTG = 10;
    for (const e of layout.entities) {
      if (e.name !== "pipe-to-ground" || e.io_type !== "input") continue;
      const [dx, dy] = dirVec(e.direction);
      const x = e.x ?? 0;
      const y = e.y ?? 0;
      for (let dist = 2; dist <= MAX_PTG; dist++) {
        const te = ptgMap.get(`${x + dx * dist},${y + dy * dist}`);
        if (!te) continue;
        const [tdx, tdy] = dirVec(te.direction);
        // Pair = opposite-direction output on the same axis.
        if (te.io_type !== "output" || tdx !== -dx || tdy !== -dy) break;
        const tg = new Graphics();
        tg.setStrokeStyle({ width: 2, color: PIPE_COLOR, alpha: 0.55, cap: "round" });
        // Start just past input's tile edge in tunnel direction; end
        // just before output's tile edge. Leaves the PTG stubs as the
        // visible mouths.
        const x0 = (x + 0.5 + dx * 0.5) * TILE_PX;
        const y0 = (y + 0.5 + dy * 0.5) * TILE_PX;
        const totalPx = (dist - 1) * TILE_PX;
        const dash = 5;
        const gap = 3;
        let drawn = 0;
        while (drawn < totalPx) {
          const segEnd = Math.min(drawn + dash, totalPx);
          tg.moveTo(x0 + dx * drawn, y0 + dy * drawn)
            .lineTo(x0 + dx * segEnd, y0 + dy * segEnd)
            .stroke();
          drawn = segEnd + gap;
        }
        container.addChild(tg);
        break;
      }
    }
  }

  // Index: item name → list of Graphics in that chain
  const itemIndex = new Map<string, Graphics[]>();
  const allGraphics: Graphics[] = [];
  // Maps each Graphics object back to its tile key "x,y" for belt network lookup
  const graphicsKeyMap = new Map<Graphics, string>();

  for (const entity of layout.entities) {
    let g: Graphics;

    if (BELT_ENTITIES.has(entity.name)) {
      g = drawBelt(entity, detectBeltTurn(entity, tileMap));
    } else if (UG_BELT_ENTITIES.has(entity.name)) {
      g = drawUndergroundBelt(entity);
    } else if (SPLITTER_ENTITIES.has(entity.name)) {
      g = drawSplitter(entity);
    } else if (INSERTER_ENTITIES.has(entity.name)) {
      g = drawInserter(entity);
    } else if (PIPE_ENTITIES.has(entity.name)) {
      // Compute connections for regular pipes
      let pipeConn = 0;
      if (entity.name === "pipe") {
        const ex = entity.x ?? 0;
        const ey = entity.y ?? 0;
        for (const [dx, dy, bit] of [[0, -1, CONN_N], [1, 0, CONN_E], [0, 1, CONN_S], [-1, 0, CONN_W]] as [number, number, number][]) {
          // The top edge of the layout represents the external fluid source —
          // a pipe at y=0 visually connects "upward" as if fed from above, so
          // it renders as a straight pipe rather than a dead-end stub.
          if (dy === -1 && ey + dy < 0) {
            pipeConn |= bit;
            continue;
          }
          const key = `${ex + dx},${ey + dy}`;
          const nb = tileMap.get(key);
          if (nb && pipeConnectsToNeighbour(nb, dx, dy)) {
            pipeConn |= bit;
            continue;
          }
          const machine = machineByTile.get(key);
          if (machine && pipeIsAtFluidPort(ex, ey, machine)) pipeConn |= bit;
        }
      }
      g = drawPipe(entity, pipeConn);
    } else if (POLE_ENTITIES.has(entity.name)) {
      g = drawPole();
    } else if (MACHINE_ENTITIES.has(entity.name)) {
      g = drawMachine(entity);
    } else {
      g = drawGenericEntity();
    }

    g.x = (entity.x ?? 0) * TILE_PX;
    g.y = (entity.y ?? 0) * TILE_PX;

    // Make every entity interactive for click; hover is handled via tile-lookup
    // in the canvas pointermove handler (see main.ts) to avoid the per-frame
    // scene-walk cost of per-entity Pixi pointer events.
    if (onSelect) {
      g.eventMode = "static";
      g.cursor = "pointer";
      g.on("click", () => onSelect(entity));
    }

    // Register in item chain index
    const key = chainKey(entity);
    if (key) {
      if (!itemIndex.has(key)) itemIndex.set(key, []);
      itemIndex.get(key)!.push(g);
    }
    graphicsKeyMap.set(g, `${entity.x ?? 0},${entity.y ?? 0}`);
    allGraphics.push(g);

    container.addChild(g);

    // Item icons are now rendered as a separate ParticleContainer layer by
    // the particle-based rendering path (Phase 2). The old per-Graphics
    // carries-icon overlay (with % 5 heuristic) is removed.

    onEntityRendered?.(entity, [g]);
  }

  // Build belt connectivity graph (once per layout)
  const beltGraph: BeltGraph = buildBeltGraph(layout);

  // Overlay Graphics for belt network highlight — created/destroyed on hover
  let overlayGraphics: Graphics | null = null;

  function clearHighlightInternal(): void {
    if (overlayGraphics) {
      container.removeChild(overlayGraphics);
      overlayGraphics.destroy();
      overlayGraphics = null;
    }
    for (const g of allGraphics) g.alpha = 1;
  }

  return {
    highlightItem(item: string | null): void {
      clearHighlightInternal();
      if (!item) return;
      const highlighted = itemIndex.get(item);
      if (!highlighted || highlighted.length === 0) return;
      const highlightSet = new Set(highlighted);
      for (const g of allGraphics) {
        g.alpha = highlightSet.has(g) ? 1 : 0.15;
      }
    },

    highlightBeltNetwork(entity: PlacedEntity | null): void {
      clearHighlightInternal();
      if (!entity) return;

      const startKey = `${entity.x ?? 0},${entity.y ?? 0}`;
      // Resolve to anchor (handles clicking on splitter's second tile)
      const anchor = beltGraph.tileToAnchor.get(startKey) ?? startKey;
      if (!beltGraph.nodes.has(anchor)) return;

      const { downstream, upstream } = traceBeltNetwork(anchor, beltGraph);
      const allBelt = new Set([...downstream, ...upstream]);
      const inserters = findAdjacentInserters(allBelt, beltGraph.entityMap);
      const machines = findAdjacentMachines(inserters, beltGraph.entityMap);

      for (const g of allGraphics) {
        const k = graphicsKeyMap.get(g);
        if (!k) {
          g.alpha = 0.15;
          continue;
        }
        if (allBelt.has(k)) {
          g.alpha = 0.5;
        } else if (inserters.has(k)) {
          g.alpha = 0.9;
        } else if (machines.has(k)) {
          g.alpha = 0.75;
        } else {
          g.alpha = 0.15;
        }
      }

      overlayGraphics = new Graphics();
      drawBeltNetworkOverlay(overlayGraphics, downstream, upstream, anchor, beltGraph);
      container.addChild(overlayGraphics);
    },

    clearHighlight(): void {
      clearHighlightInternal();
    },

    chainKey,
  };
}

/** Get the item chain key for an entity. */
function chainKey(entity: PlacedEntity): string | null {
  // Belts, underground belts, splitters, inserters, pipes — use carries
  if (entity.carries) return entity.carries;
  // Machines — use recipe as chain key (connects to the items it produces)
  if (entity.recipe) return entity.recipe;
  return null;
}
