/**
 * Connectivity graph for distance-based focus highlighting.
 *
 * Builds a single undirected adjacency list across every entity type:
 * belts, underground belts, splitters, inserters, machines, pipes, and
 * pipe-to-ground pairs. BFS over this graph produces per-entity hop
 * distances from a hovered entity.
 *
 * Build once per layout (cache in the highlight controller). O(N*4)
 * where N = entity count — well under 10 ms for 3000-entity layouts.
 */

import type { LayoutResult, PlacedEntity, EntityDirection } from "../engine";
import {
  BELT_ENTITIES,
  UG_BELT_ENTITIES,
  SPLITTER_ENTITIES,
  INSERTER_ENTITIES,
  MACHINE_ENTITIES,
  MACHINE_SIZES,
  PIPE_ENTITIES,
} from "./entities";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/** Entity key: `"x,y:name:recipe"` — matches `entityKey()` in particleLayout.ts */
export type EntityKey = string;

/** Undirected adjacency list. Each key maps to the keys of its neighbours. */
export type ConnectivityGraph = Map<EntityKey, EntityKey[]>;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function tk(x: number, y: number): string {
  return `${x},${y}`;
}

function eKey(e: PlacedEntity): EntityKey {
  return `${e.x ?? 0},${e.y ?? 0}:${e.name}:${e.recipe ?? ""}`;
}

function dirVec(dir?: EntityDirection): [number, number] {
  switch (dir) {
    case "East":  return [1, 0];
    case "South": return [0, 1];
    case "West":  return [-1, 0];
    default:      return [0, -1]; // North
  }
}

function addEdge(graph: ConnectivityGraph, a: EntityKey, b: EntityKey): void {
  if (a === b) return;
  let la = graph.get(a);
  if (!la) { la = []; graph.set(a, la); }
  if (!la.includes(b)) la.push(b);

  let lb = graph.get(b);
  if (!lb) { lb = []; graph.set(b, lb); }
  if (!lb.includes(a)) lb.push(a);
}

/** Coord → entity key for entities of all types (anchor tile only). */
function buildTileIndex(
  layout: LayoutResult,
): Map<string, EntityKey> {
  const idx = new Map<string, EntityKey>();
  for (const e of layout.entities) {
    const x = e.x ?? 0;
    const y = e.y ?? 0;
    const k = eKey(e);
    idx.set(tk(x, y), k);

    // Splitters occupy 2 tiles perpendicular to travel direction.
    if (SPLITTER_ENTITIES.has(e.name)) {
      const isNS = e.direction === "North" || e.direction === "South";
      idx.set(tk(x + (isNS ? 1 : 0), y + (isNS ? 0 : 1)), k);
    }

    // Multi-tile machines: every tile maps back to the anchor key.
    if (MACHINE_ENTITIES.has(e.name)) {
      const [w, h] = MACHINE_SIZES[e.name] ?? [1, 1];
      for (let dy = 0; dy < h; dy++) {
        for (let dx = 0; dx < w; dx++) {
          if (dx === 0 && dy === 0) continue;
          idx.set(tk(x + dx, y + dy), k);
        }
      }
    }
  }
  return idx;
}

// ---------------------------------------------------------------------------
// Graph construction
// ---------------------------------------------------------------------------

const MAX_UG_DIST = 9;

/**
 * Build a full connectivity graph for `layout`.
 * Every entity is a node. Edges are undirected (weight 1 per hop).
 *
 * Edge types (both directions added):
 *   belt → adjacent belt (straight + sideload)
 *   belt → UG output pairing
 *   splitter → its output/input belts
 *   inserter ↔ belt (on either pickup or drop tile)
 *   inserter ↔ machine (on the other side of the inserter, regular or long-handed)
 *   pipe ↔ adjacent pipe (4-way)
 *   pipe-to-ground → paired exit (same axis, opposite direction)
 *   machine ↔ adjacent pipe (fluid port tiles — any adjacent pipe counts)
 */
export function buildConnectivityGraph(layout: LayoutResult): ConnectivityGraph {
  const graph: ConnectivityGraph = new Map();
  const tileIdx = buildTileIndex(layout);

  // Ensure every entity has a node (even with no edges — isolated poles etc.)
  for (const e of layout.entities) {
    const k = eKey(e);
    if (!graph.has(k)) graph.set(k, []);
  }

  // Full entity list keyed by anchor tile for scanning neighbours.
  const entityByTile = new Map<string, PlacedEntity>();
  for (const e of layout.entities) {
    entityByTile.set(tk(e.x ?? 0, e.y ?? 0), e);
  }

  // ---------------------------------------------------------------------------
  // Pass: belt / UG / splitter edges
  // ---------------------------------------------------------------------------
  const OFFSETS: [number, number][] = [[0, -1], [1, 0], [0, 1], [-1, 0]];

  for (const e of layout.entities) {
    const x = e.x ?? 0;
    const y = e.y ?? 0;
    const k = eKey(e);
    const [dx, dy] = dirVec(e.direction);

    if (BELT_ENTITIES.has(e.name)) {
      // 1a. Forward neighbour (downstream).
      const fk = tileIdx.get(tk(x + dx, y + dy));
      if (fk) addEdge(graph, k, fk);

      // 1b. Backward neighbour (upstream = tile in reverse direction).
      const bk = tileIdx.get(tk(x - dx, y - dy));
      if (bk) addEdge(graph, k, bk);

      // 1c. Sideload inputs: belts on perpendicular sides whose flow direction
      //     points at this tile.
      for (const [ox, oy] of OFFSETS) {
        if ((ox === dx && oy === dy) || (ox === -dx && oy === -dy)) continue; // skip inline
        const nb = entityByTile.get(tk(x + ox, y + oy));
        if (!nb) continue;
        if (!BELT_ENTITIES.has(nb.name) && !UG_BELT_ENTITIES.has(nb.name) && !SPLITTER_ENTITIES.has(nb.name)) continue;
        const [ndx, ndy] = dirVec(nb.direction);
        // nb flows into this tile?
        // For a splitter the active tile is the neighbour tile itself (x+ox, y+oy);
        // for a regular belt it's the belt's anchor position.
        const srcX = SPLITTER_ENTITIES.has(nb.name) ? x + ox : (nb.x ?? 0);
        const srcY = SPLITTER_ENTITIES.has(nb.name) ? y + oy : (nb.y ?? 0);
        if (srcX + ndx === x && srcY + ndy === y) {
          addEdge(graph, k, eKey(nb));
        }
      }
    } else if (UG_BELT_ENTITIES.has(e.name)) {
      if (e.io_type === "input") {
        // Scan forward for the matching UG output.
        for (let dist = 1; dist <= MAX_UG_DIST; dist++) {
          const te = entityByTile.get(tk(x + dx * dist, y + dy * dist));
          if (!te) continue;
          if (UG_BELT_ENTITIES.has(te.name) && te.name === e.name && te.io_type === "input" && te.direction === e.direction) break; // blocked
          if (UG_BELT_ENTITIES.has(te.name) && te.name === e.name && te.io_type === "output" && te.direction === e.direction) {
            addEdge(graph, k, eKey(te));
            break;
          }
        }
      } else {
        // UG output → forward neighbour (like a regular belt).
        const fk = tileIdx.get(tk(x + dx, y + dy));
        if (fk) addEdge(graph, k, fk);
      }

      // Also connect to adjacent belts that feed into this UG.
      for (const [ox, oy] of OFFSETS) {
        const nb = entityByTile.get(tk(x + ox, y + oy));
        if (!nb) continue;
        if (!BELT_ENTITIES.has(nb.name) && !SPLITTER_ENTITIES.has(nb.name)) continue;
        const [ndx, ndy] = dirVec(nb.direction);
        if ((nb.x ?? 0) + ndx === x && (nb.y ?? 0) + ndy === y) {
          addEdge(graph, k, eKey(nb));
        }
      }
    } else if (SPLITTER_ENTITIES.has(e.name)) {
      // Splitter occupies 2 tiles. Its 2 output tiles are in the forward direction
      // from both halves.
      const isNS = e.direction === "North" || e.direction === "South";
      const [sdx, sdy] = isNS ? [1, 0] : [0, 1];
      for (const [ox, oy] of [[0, 0], [sdx, sdy]] as [number, number][]) {
        const outK = tileIdx.get(tk(x + ox + dx, y + oy + dy));
        if (outK && outK !== k) addEdge(graph, k, outK);
        const inK = tileIdx.get(tk(x + ox - dx, y + oy - dy));
        if (inK && inK !== k) addEdge(graph, k, inK);
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Pass: inserter ↔ belt / machine
  // ---------------------------------------------------------------------------
  for (const e of layout.entities) {
    if (!INSERTER_ENTITIES.has(e.name)) continue;
    const x = e.x ?? 0;
    const y = e.y ?? 0;
    const k = eKey(e);
    const [dx, dy] = dirVec(e.direction);

    // Inserter direction: `direction` points from pickup to drop.
    // Pickup side: behind the inserter (opposite of direction).
    // Drop side: in front (direction).
    // Long-handed: pickup is 2 tiles behind, drop is 2 tiles ahead.
    const isLong = e.name === "long-handed-inserter";
    const reach = isLong ? 2 : 1;

    const pickupK = tileIdx.get(tk(x - dx * reach, y - dy * reach));
    const dropK   = tileIdx.get(tk(x + dx * reach, y + dy * reach));

    if (pickupK) addEdge(graph, k, pickupK);
    if (dropK)   addEdge(graph, k, dropK);
  }

  // ---------------------------------------------------------------------------
  // Pass: pipe ↔ adjacent pipe / pipe-to-ground pairing
  // ---------------------------------------------------------------------------
  const MAX_PTG_DIST = 10;

  for (const e of layout.entities) {
    if (!PIPE_ENTITIES.has(e.name)) continue;
    const x = e.x ?? 0;
    const y = e.y ?? 0;
    const k = eKey(e);

    if (e.name === "pipe") {
      // Connect to all 4 neighbours that are pipe or pipe-to-ground surface side.
      for (const [ox, oy] of OFFSETS) {
        const nb = entityByTile.get(tk(x + ox, y + oy));
        if (!nb) continue;
        if (nb.name === "pipe") {
          addEdge(graph, k, eKey(nb));
        } else if (nb.name === "pipe-to-ground") {
          // Only connect on the PTG's surface side.
          // PTG tunnel direction = nb.direction; surface side is opposite.
          // The pipe (at x,y) reaches nb via offset [ox,oy].
          // Connection is valid iff the pipe is on the PTG's surface side:
          //   direction from PTG to pipe = (-ox,-oy) must equal surface delta = (-tdx,-tdy)
          //   => ox == tdx && oy == tdy  (where [tdx,tdy] = dirVec(nb.direction))
          const [tdx, tdy] = dirVec(nb.direction);
          if (ox === tdx && oy === tdy) {
            addEdge(graph, k, eKey(nb));
          }
        } else if (MACHINE_ENTITIES.has(nb.name)) {
          // Machine fluid port — any adjacent machine counts as connected.
          const nbK = tileIdx.get(tk(nb.x ?? 0, nb.y ?? 0));
          if (nbK) addEdge(graph, k, nbK);
        }
      }
    } else if (e.name === "pipe-to-ground") {
      if (e.io_type === "input") {
        // Scan forward for matching output.
        const [dx, dy] = dirVec(e.direction);
        for (let dist = 2; dist <= MAX_PTG_DIST; dist++) {
          const te = entityByTile.get(tk(x + dx * dist, y + dy * dist));
          if (!te) continue;
          if (te.name !== "pipe-to-ground") continue;
          const [tdx, tdy] = dirVec(te.direction);
          // Pair: opposite direction (output coming back).
          if (te.io_type === "output" && tdx === -dx && tdy === -dy) {
            addEdge(graph, k, eKey(te));
            break;
          }
          break; // any other PTG blocks the tunnel
        }
      }
      // Also connect to adjacent pipe on surface side.
      const [dx, dy] = dirVec(e.direction);
      const surfNb = entityByTile.get(tk(x - dx, y - dy));
      if (surfNb && surfNb.name === "pipe") {
        addEdge(graph, k, eKey(surfNb));
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Pass: machine ↔ adjacent pipe (fluid connections from machine side)
  // ---------------------------------------------------------------------------
  for (const e of layout.entities) {
    if (!MACHINE_ENTITIES.has(e.name)) continue;
    const x = e.x ?? 0;
    const y = e.y ?? 0;
    const k = eKey(e);
    const [mw, mh] = MACHINE_SIZES[e.name] ?? [1, 1];

    // Check the perimeter of the machine footprint for adjacent pipes.
    for (let dy = 0; dy < mh; dy++) {
      for (let dx = 0; dx < mw; dx++) {
        for (const [ox, oy] of OFFSETS) {
          const nx = x + dx + ox;
          const ny = y + dy + oy;
          // Only consider tiles outside the machine footprint.
          if (nx >= x && nx < x + mw && ny >= y && ny < y + mh) continue;
          const nb = entityByTile.get(tk(nx, ny));
          if (!nb) continue;
          if (PIPE_ENTITIES.has(nb.name)) {
            addEdge(graph, k, eKey(nb));
          }
        }
      }
    }
  }

  return graph;
}

// ---------------------------------------------------------------------------
// BFS
// ---------------------------------------------------------------------------

/**
 * BFS from `start` over `graph`. Returns a map of entity key → hop distance.
 * Unreachable nodes are absent from the map.
 */
export function bfsDistances(
  graph: ConnectivityGraph,
  start: EntityKey,
): Map<EntityKey, number> {
  const dist = new Map<EntityKey, number>();
  if (!graph.has(start)) return dist;
  dist.set(start, 0);
  const queue: EntityKey[] = [start];
  let head = 0;
  while (head < queue.length) {
    const cur = queue[head++];
    const d = dist.get(cur)!;
    for (const nb of graph.get(cur) ?? []) {
      if (!dist.has(nb)) {
        dist.set(nb, d + 1);
        queue.push(nb);
      }
    }
  }
  return dist;
}
