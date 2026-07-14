// Per-tile aggregator: given a layout, indexes everything we know about
// a single tile (x,y) so the inspector can show it in one place instead
// of scattering on-canvas labels everywhere.
//
// Currently surfaces:
//   - ghost specs routed through the tile (item + direction at the tile)
//   - axis occupancy (V/H counts from the negotiated router)
//   - junction-cluster membership (which solver seed this tile belongs to,
//     if any, plus the outcome)

import type { TraceEvent } from "../engine";

export interface GhostSpecAtTile {
  item: string;
  specKey: string;
  direction: "N" | "S" | "E" | "W" | null;
  /** true if this tile is the start of the spec path. */
  isStart: boolean;
  /** true if this tile is the end of the spec path. */
  isEnd: boolean;
}

export interface AxisAtTile {
  vert: number;
  horiz: number;
}

export interface JunctionMembership {
  seedX: number;
  seedY: number;
  outcome: "Solved" | "Capped" | "Open";
}

/** A machine side the inserter ladder couldn't fully provision, keyed at
 *  the machine origin — the same tile rate-shaped validation warnings
 *  anchor at, so the inspector can explain a warning by plain lookup
 *  (RFP validation-explainability D2: join, never inference). */
export interface CappedSideAtTile {
  recipe: string;
  sideIsOutput: boolean;
  required: number;
  placedEntity: string;
  placedCount: number;
  shortfall: number;
  /** Binding constraint: "tier-cap" | "column-contest" | "geometry". */
  limit: string;
}

export interface TileInfo {
  ghosts: GhostSpecAtTile[];
  axis: AxisAtTile | null;
  junction: JunctionMembership | null;
  cappedSides: CappedSideAtTile[];
}

export interface TileContext {
  lookup(x: number, y: number): TileInfo;
}

function itemFromSpecKey(key: string): string {
  const i = key.indexOf(":");
  return i >= 0 ? key.slice(0, i) : key;
}

function tileDirection(ax: number, ay: number, bx: number, by: number): "N" | "S" | "E" | "W" | null {
  if (bx > ax) return "E";
  if (bx < ax) return "W";
  if (by > ay) return "S";
  if (by < ay) return "N";
  return null;
}

const EMPTY: TileInfo = { ghosts: [], axis: null, junction: null, cappedSides: [] };

export function buildTileContext(trace: readonly TraceEvent[] | undefined): TileContext {
  if (!trace || trace.length === 0) {
    return { lookup: () => EMPTY };
  }

  const ghostMap = new Map<string, GhostSpecAtTile[]>();
  const axisMap = new Map<string, AxisAtTile>();
  const junctionMap = new Map<string, JunctionMembership>();
  const cappedMap = new Map<string, CappedSideAtTile[]>();

  for (const evt of trace) {
    if (evt.phase === "GhostSpecRouted") {
      const { spec_key, tiles } = evt.data;
      const item = itemFromSpecKey(spec_key);
      if (!tiles || tiles.length === 0) continue;
      for (let i = 0; i < tiles.length; i++) {
        const [tx, ty] = tiles[i];
        let dir: "N" | "S" | "E" | "W" | null = null;
        if (i < tiles.length - 1) {
          dir = tileDirection(tx, ty, tiles[i + 1][0], tiles[i + 1][1]);
        } else if (i > 0) {
          dir = tileDirection(tiles[i - 1][0], tiles[i - 1][1], tx, ty);
        }
        const key = `${tx},${ty}`;
        const list = ghostMap.get(key);
        const entry: GhostSpecAtTile = {
          item,
          specKey: spec_key,
          direction: dir,
          isStart: i === 0,
          isEnd: i === tiles.length - 1,
        };
        if (list) list.push(entry);
        else ghostMap.set(key, [entry]);
      }
    } else if (evt.phase === "GhostAxisOccupancy") {
      for (const t of evt.data.tiles) {
        axisMap.set(`${t.x},${t.y}`, { vert: t.vert_count, horiz: t.horiz_count });
      }
    } else if (
      evt.phase === "JunctionSolved" ||
      evt.phase === "JunctionGrowthCapped"
    ) {
      // Seed-level outcome — we tag only the seed tile here; bbox-level
      // containment is surfaced via `junctionCluster.ts` downstream. For
      // now the seed match is enough to indicate "this is a junction".
      const d = evt.data;
      const outcome: JunctionMembership["outcome"] =
        evt.phase === "JunctionSolved" ? "Solved" :
        "Capped";
      junctionMap.set(`${d.tile_x},${d.tile_y}`, {
        seedX: d.tile_x, seedY: d.tile_y, outcome,
      });
    } else if (evt.phase === "InserterSideCapped") {
      const d = evt.data;
      const key = `${d.machine_x},${d.machine_y}`;
      const entry: CappedSideAtTile = {
        recipe: d.recipe,
        sideIsOutput: d.side_is_output,
        required: d.required,
        placedEntity: d.placed_entity,
        placedCount: d.placed_count,
        shortfall: d.shortfall,
        limit: d.limit,
      };
      const list = cappedMap.get(key);
      if (list) list.push(entry);
      else cappedMap.set(key, [entry]);
    } else if (evt.phase === "JunctionGrowthIteration") {
      // Tag every tile in the grown region with the seed. Later iters
      // overwrite earlier ones so the last-seen membership wins (fine
      // for display — we just want "which junction is this tile in").
      const d = evt.data;
      const seedKey = `${d.seed_x},${d.seed_y}`;
      for (const [tx, ty] of d.tiles) {
        const key = `${tx},${ty}`;
        if (!junctionMap.has(key) || junctionMap.get(key)!.seedX === d.seed_x) {
          junctionMap.set(key, {
            seedX: d.seed_x, seedY: d.seed_y,
            outcome: junctionMap.get(seedKey)?.outcome ?? "Open",
          });
        }
      }
    }
  }

  return {
    lookup(x: number, y: number): TileInfo {
      const key = `${x},${y}`;
      return {
        ghosts: ghostMap.get(key) ?? [],
        axis: axisMap.get(key) ?? null,
        junction: junctionMap.get(key) ?? null,
        cappedSides: cappedMap.get(key) ?? [],
      };
    },
  };
}
