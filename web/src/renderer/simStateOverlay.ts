import { Graphics, Container } from "pixi.js";
import { TILE_PX, MACHINE_SIZES } from "./entities";
import type { SimState } from "../ui/simReportLoader";

/** Sim-state overlay (RFC-050 Phase 4): visualizes a `spaghettio-sim`
 *  report's live-factory snapshot over the CURRENT layout — "the harness
 *  finds that something is wrong; the overlay shows where."
 *
 *  Follows the `moduleSlotsOverlay`/`powerWiresOverlay` shape: a fresh
 *  Graphics layer built straight from the loaded report's coordinates
 *  (already layout-space per the harness's derived world offset — see
 *  `docs/rfc-050-headless-sim-harness.md`), redrawn from scratch on every
 *  load/toggle. Never touches the particle atlas, so it needs no special
 *  handling for the two-renderer split (`web/CLAUDE.md`).
 */

/** Machine status → tint. Names are the raw in-game `LuaEntity.status`
 *  values the harness records. Statuses outside this table (`no_fuel`,
 *  `disabled`, `frozen`, ...) fall back to neutral grey — same
 *  "unrecognized → neutral" convention as `moduleSlotsOverlay`'s unknown
 *  module families, rather than silently misrepresenting a status this
 *  overlay doesn't know about yet. */
const MACHINE_STATUS_COLORS: Record<string, number> = {
  working: 0x4caf50, // green
  item_ingredient_shortage: 0xd9432e, // red
  no_ingredients: 0xd9432e, // red
  full_output: 0xff9a3c, // orange
  no_power: 0x9b59b6, // purple
};
const MACHINE_STATUS_DEFAULT = 0x8a8a8a; // grey — "other"
const MACHINE_FILL_ALPHA = 0.4;

function machineColor(status: string): number {
  return MACHINE_STATUS_COLORS[status] ?? MACHINE_STATUS_DEFAULT;
}

/** Belt fill-level heat: yellow (low) → red (saturated). `count` is the
 *  harness's raw per-tile item tally; it isn't a strict per-tile-capacity
 *  count (observed values run well past a single tile's ~8-item physical
 *  max on the dogfood fixtures — a whole-line sum, not per-tile), so
 *  anything at/above `BELT_HEAT_CAP` just clamps to full red rather than
 *  needing an exact denominator. Empty belts (count 0) draw nothing. */
const BELT_HEAT_CAP = 8;
const BELT_HEAT_ALPHA = 0.5;
const HEAT_LOW = { r: 0xf5, g: 0xc5, b: 0x18 }; // yellow
const HEAT_HIGH = { r: 0xd9, g: 0x43, b: 0x2e }; // red

function beltHeatColor(count: number): number {
  const t = Math.max(0, Math.min(1, count / BELT_HEAT_CAP));
  const r = Math.round(HEAT_LOW.r + (HEAT_HIGH.r - HEAT_LOW.r) * t);
  const g = Math.round(HEAT_LOW.g + (HEAT_HIGH.g - HEAT_LOW.g) * t);
  const b = Math.round(HEAT_LOW.b + (HEAT_HIGH.b - HEAT_LOW.b) * t);
  return (r << 16) | (g << 8) | b;
}

const INSERTER_DOT_RADIUS = TILE_PX * 0.14;
const INSERTER_DOT_ALPHA = 0.9;

/** Dot color for an inserter status, or `null` to draw nothing (the
 *  `working` happy path, per the RFC-050 Phase 4 spec). Statuses beyond
 *  the three named cases (e.g. `waiting_for_more_items`, seen on the
 *  dogfood fixtures) get a neutral grey dot rather than vanishing
 *  silently — same "other → grey, never invisible" convention as the
 *  machine tint table above. */
function inserterDotColor(status: string): number | null {
  if (status === "working") return null;
  if (status.startsWith("waiting_for_source")) return 0xd9432e; // red
  if (status.startsWith("waiting_for_space")) return 0xff9a3c; // orange
  return 0x8a8a8a; // grey — other blocked/idle status
}

export function renderSimStateOverlay(simState: SimState, container: Container): Container | null {
  const layer = new Container();
  // Purely decorative: never intercept pointer events — entity
  // hover/pin stays in charge, same convention as the other overlays.
  layer.eventMode = "none";
  let drawn = false;

  // Belts drawn first so machine footprints + inserter dots read on top.
  const beltG = new Graphics();
  for (const [x, y, count] of simState.belts) {
    if (count <= 0) continue;
    beltG
      .rect(x * TILE_PX, y * TILE_PX, TILE_PX, TILE_PX)
      .fill({ color: beltHeatColor(count), alpha: BELT_HEAT_ALPHA });
    drawn = true;
  }
  layer.addChild(beltG);

  const machineG = new Graphics();
  for (const [x, y, name, status] of simState.machines) {
    const [w, h] = MACHINE_SIZES[name] ?? [1, 1];
    machineG
      .rect(x * TILE_PX, y * TILE_PX, w * TILE_PX, h * TILE_PX)
      .fill({ color: machineColor(status), alpha: MACHINE_FILL_ALPHA });
    drawn = true;
  }
  layer.addChild(machineG);

  const inserterG = new Graphics();
  for (const [x, y, status] of simState.inserters) {
    const color = inserterDotColor(status);
    if (color === null) continue;
    const cx = x * TILE_PX + TILE_PX / 2;
    const cy = y * TILE_PX + TILE_PX / 2;
    inserterG.circle(cx, cy, INSERTER_DOT_RADIUS).fill({ color, alpha: INSERTER_DOT_ALPHA });
    drawn = true;
  }
  layer.addChild(inserterG);

  if (!drawn) return null;
  container.addChild(layer);
  return layer;
}
