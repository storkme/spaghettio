/**
 * Streaming layout renderer — progressive reveal during layout, then
 * hands off to `renderLayout` as the authoritative renderer.
 *
 * Two phases:
 *
 * ## Live phase (onEvent drives everything)
 *
 * As trace events arrive from the WASM worker we draw *transient* eye-
 * candy: ghost-path overlays for `GhostSpecRouted`, per-tile committed
 * previews for `GhostSpecCommitted` / `JunctionCommitted` /
 * `PhaseSnapshot`, and cluster-outline pulses for `GhostClusterSolved`.
 *
 * These transient graphics only need to approximate the final layout —
 * they exist to give the user something to watch during the 5–6 s SAT
 * phase. They are all destroyed at `finish()`.
 *
 * While drawing the transient previews we record each entity's first-
 * seen virtual-ms into `revealByEntityKey`. That timestamp map is the
 * only state that survives into the scrub phase.
 *
 * ## Scrub phase (finish() + seekTo())
 *
 * At `finish(layout)` the transient graphics are discarded and
 * `renderLayout(layout, container, …)` draws the authoritative
 * `layout.entities`. The full tile-map is built in a single pass so
 * belt-turn detection is correct by construction.
 *
 * We capture each rendered entity Graphics via `renderLayout`'s
 * `onEntityRendered` hook, cross them with `revealByEntityKey` to get
 * per-entity reveal timestamps, and from then on `seekTo(t)` just
 * walks the list setting alpha imperatively.
 *
 * `renderLayout`'s returned `HighlightController` is the real
 * hover-highlighting one — belt-network traces light up on hover,
 * matching non-streaming rendering paths.
 *
 * See `docs/archive/rfp-streaming-reconciliation.md` for the full rationale.
 */

import type { Application, Container, Graphics } from "pixi.js";
import { Container as PixiContainer, Graphics as PixiGraphics } from "pixi.js";
import type { LayoutResult } from "../engine";
import type { PlacedEntity, TraceEvent, EntityDirection } from "../wasm-pkg/fucktorio_wasm";
import {
  drawEntityGraphic,
  addEntityToDrawContext,
  createDrawContext,
  drawUgTunnelStripe,
  drawBelt,
  itemColor,
  renderLayout,
  TILE_PX,
  type DrawContext,
  type HighlightController,
} from "./entities";

// ---------------------------------------------------------------------------
// Timings
// ---------------------------------------------------------------------------

const FADE_IN_MS = 150;
const FADE_OUT_MS = 80;
// Max per-entity stagger (applied only when the phase is small enough
// that entity-count × stagger fits inside the phase budget). Above
// the budget, the actual stagger is shrunk to `budget / count` so a
// bulky phase doesn't take forever. Keeps the eye-candy on small
// layouts while scaling gracefully on big ones.
const GHOST_TILE_STAGGER_MAX_MS = 4;
const GHOST_TILE_STAGGER_BUDGET_MS = 300;
const COMMITTED_TILE_STAGGER_MAX_MS = 4;
const COMMITTED_TILE_STAGGER_BUDGET_MS = 900;
const JUNCTION_TILE_STAGGER_MAX_MS = 6;
const JUNCTION_TILE_STAGGER_BUDGET_MS = 250;
/** GhostClusterSolved outline box stays this long before fading. */
const GHOST_CLUSTER_LIFETIME_MS = 800;
const GHOST_CLUSTER_FADE_MS = 250;

/** Compute a per-entity stagger: `max` ms when the phase is small,
 *  shrunk to `budget / count` once a naive `max` would exceed the
 *  budget. Mirrors the pacing idea in `phaseAnimation.ts:178`. */
function stagger(count: number, maxMs: number, budgetMs: number): number {
  if (count <= 1) return maxMs;
  return Math.min(maxMs, budgetMs / count);
}

// ---------------------------------------------------------------------------
// Key helpers
// ---------------------------------------------------------------------------

function entityKey(e: PlacedEntity): string {
  return `${e.x ?? 0},${e.y ?? 0},${e.name},${e.recipe ?? ""}`;
}

function tileKey(x: number, y: number): string {
  return `${x},${y}`;
}

/** Pull the item name out of a spec_key like "trunk:iron-plate:3" or
 *  "tap:copper-cable:7:45". Parts are colon-separated; [1] is the item. */
function itemFromSpecKey(specKey: string): string {
  return specKey.split(":")[1] ?? "";
}

function dirFromDelta(dx: number, dy: number): EntityDirection {
  if (dx > 0) return "East" as EntityDirection;
  if (dx < 0) return "West" as EntityDirection;
  if (dy > 0) return "South" as EntityDirection;
  return "North" as EntityDirection;
}

// ---------------------------------------------------------------------------
// Tracked-graphic state (live phase only)
// ---------------------------------------------------------------------------

/** Ghost belt at a single tile. `specKey` records which spec laid this
 *  down; a subsequent spec can add its own overlay (crossing) without
 *  touching the existing ghost. */
interface GhostBelt {
  gfx: Graphics;
  overlayGfx?: Graphics;
  specKey: string;
  overlaySpecKey?: string;
  fadeStartMs: number;
  fadeOutStartMs: number | null;
}

/** A transient committed entity drawn during live streaming. Discarded
 *  at `finish()` — `renderLayout` redraws it authoritatively. */
interface TransientCommitted {
  gfx: Graphics;
  ambientGfx: Graphics | null;
  fadeStartMs: number;
  fadeOutStartMs: number | null;
  entity: PlacedEntity;
}

interface GhostCluster {
  x: number;
  y: number;
  w: number;
  h: number;
  startMs: number;
  clusterId: number;
  cleared: boolean;
}

// ---------------------------------------------------------------------------
// Scrub-phase state
// ---------------------------------------------------------------------------

interface Reveal {
  graphic: Graphics;
  revealAt: number;
}

// ---------------------------------------------------------------------------
// Public handle
// ---------------------------------------------------------------------------

export type MilestoneId =
  | "machines"
  | "ghost_routes"
  | "committed_routes"
  | "junctions"
  | "poles";

export interface Milestone {
  id: MilestoneId;
  label: string;
  /** Absolute virtual ms when this milestone's trigger event first arrived. */
  virtualMs: number;
}

export interface StreamingRendererHandle {
  onEvent(evt: TraceEvent, onMilestone?: (m: Milestone) => void): void;
  /** True once any PhaseSnapshot has been processed — lets main.ts
   *  decide whether to skip the non-streaming phaseAnimation fallback. */
  hasCommittedEntities(): boolean;
  /** Stop immediately, drop graphics. */
  cancel(): void;
  /** Hand off from live preview to authoritative render. Destroys all
   *  transient streaming graphics, calls `renderLayout` to draw the
   *  real `layout.entities`, and returns its `HighlightController` for
   *  the caller to wire up. After `finish()`, `seekTo` is the only
   *  way to update the canvas. */
  finish(layout: LayoutResult): HighlightController;
  /** Set the virtual clock to `virtualMs` and recompute every fade.
   *  Only meaningful after `finish()` has been called. */
  seekTo(virtualMs: number): void;
  /** After `finish()`: the [first, last] virtual-ms range this stream
   *  spans. Before `finish()`: { firstMs, lastMs } reflecting the
   *  earliest event seen and the latest fade-end scheduled so far. */
  getTimeRange(): { firstMs: number; lastMs: number };
  /** Milestones recorded so far, sorted by virtualMs. */
  getMilestones(): Milestone[];
}

const MILESTONE_LABELS: Record<MilestoneId, string> = {
  machines: "Machines",
  ghost_routes: "Ghost Routes",
  committed_routes: "Committed Routes",
  junctions: "Junctions",
  poles: "Poles",
};

// ---------------------------------------------------------------------------
// Main factory
// ---------------------------------------------------------------------------

export function createStreamingRenderer(
  container: Container,
  app: Application,
  onHover?: (e: PlacedEntity | null) => void,
  onSelect?: (e: PlacedEntity | null) => void,
): StreamingRendererHandle {
  // Layer ordering (bottom-to-top) during live phase:
  //   1. committedLayer — transient real-entity previews
  //   2. ghostLayer     — transient ghost belts (faded over when
  //                       a committed preview arrives at the same tile)
  //   3. clusterOverlay — single Graphics for SAT cluster outline pulses
  //
  // Putting committed below ghost is intentional: crossfading a ghost →
  // committed lets the committed entity emerge from behind the fading
  // ghost, rather than flashing on top.
  //
  // At `finish()`, `renderLayout` clears the container (removing all
  // three of these) and draws the authoritative entities directly on
  // `container`.
  const committedLayer = new PixiContainer();
  const ghostLayer = new PixiContainer();
  const clusterOverlay = new PixiGraphics();
  container.addChild(committedLayer);
  container.addChild(ghostLayer);
  container.addChild(clusterOverlay);

  const ghostBeltByTile = new Map<string, GhostBelt>();
  const transientByKey = new Map<string, TransientCommitted>();
  const transientByTile = new Map<string, TransientCommitted>();
  const ghostClusters: GhostCluster[] = [];

  // First-seen virtual-ms for each final-entity-keyed reveal. Populated
  // during live streaming whenever a transient preview for that key is
  // committed. Survives into scrub mode to drive `seekTo` alpha ramps.
  const revealByEntityKey = new Map<string, number>();

  const drawCtx: DrawContext = createDrawContext();

  let cancelled = false;
  let anySnapshot = false;

  // Virtual clock. In live mode this is just wall-clock time from
  // `performance.now()`. In scrub mode (after `finish()`) it's fixed
  // to `scrubVirtualMs` which the scrubber sets via `seekTo`. All fade
  // math in live handlers goes through `clock()` so both modes share
  // the same timeline.
  let scrubMode = false;
  let scrubVirtualMs = 0;
  const clock = (): number => (scrubMode ? scrubVirtualMs : performance.now());

  // Time range & milestone bookkeeping.
  let firstMs: number | null = null;
  let latestFadeEndMs = 0;
  const milestones = new Map<MilestoneId, Milestone>();
  let pendingMilestoneCallback: ((m: Milestone) => void) | null = null;

  // Scrub-phase reveals. Populated by `finish()`; iterated by `seekTo`.
  let reveals: Reveal[] | null = null;

  function noteMilestoneNow(id: MilestoneId, at: number): void {
    // Always record the LATEST occurrence so the chip marks where the
    // phase *ended*, not where it started. Junction solving fires
    // `JunctionCommitted` repeatedly across several seconds — placing
    // the chip at the first one is misleading.
    const existing = milestones.get(id);
    const isNew = !existing;
    const m: Milestone = { id, label: MILESTONE_LABELS[id], virtualMs: at };
    milestones.set(id, m);
    // Only fire the live-mode callback once, the first time we see
    // this milestone — so the scrubber's progress bar advances one
    // step per milestone, not on every subsequent fire.
    if (isNew) pendingMilestoneCallback?.(m);
  }

  function touchTime(at: number, fadeIn: boolean): void {
    if (firstMs === null) firstMs = at;
    const end = at + (fadeIn ? FADE_IN_MS : FADE_OUT_MS);
    if (end > latestFadeEndMs) latestFadeEndMs = end;
  }

  const traceLogs =
    (globalThis as { __TRACE_LOGS?: boolean }).__TRACE_LOGS === true;

  // -------------------------------------------------------------------------
  // Transient-commit helpers (live phase)
  // -------------------------------------------------------------------------

  /** Synthesise a minimal PlacedEntity suitable for `drawBelt(e, null)`. */
  function synthGhostBeltEntity(
    x: number,
    y: number,
    direction: EntityDirection,
    item: string,
  ): PlacedEntity {
    return {
      name: "transport-belt",
      x,
      y,
      direction,
      recipe: null,
      carries: item,
      segment_id: null,
      io_type: null,
    } as unknown as PlacedEntity;
  }

  function addGhostBelt(
    x: number,
    y: number,
    direction: EntityDirection,
    item: string,
    specKey: string,
    revealAt: number,
  ): void {
    const k = tileKey(x, y);
    const existing = ghostBeltByTile.get(k);
    if (existing) {
      // Crossing — add an overlay if a different spec lays claim.
      if (!existing.overlayGfx && existing.specKey !== specKey) {
        const belt = synthGhostBeltEntity(x, y, direction, item);
        const g = drawBelt(belt, null);
        g.x = x * TILE_PX;
        g.y = y * TILE_PX;
        g.alpha = 0;
        g.tint = itemColor(item);
        ghostLayer.addChild(g);
        existing.overlayGfx = g;
        existing.overlaySpecKey = specKey;
      }
      return;
    }
    const belt = synthGhostBeltEntity(x, y, direction, item);
    const g = drawBelt(belt, null);
    g.x = x * TILE_PX;
    g.y = y * TILE_PX;
    g.alpha = 0;
    // Tint the belt body by item colour so crossings show up visibly.
    g.tint = itemColor(item);
    ghostLayer.addChild(g);
    ghostBeltByTile.set(k, {
      gfx: g,
      specKey,
      fadeStartMs: revealAt,
      fadeOutStartMs: null,
    });
    touchTime(revealAt, true);
  }

  function fadeOutGhostAt(x: number, y: number, now: number): void {
    const k = tileKey(x, y);
    const gb = ghostBeltByTile.get(k);
    if (!gb) return;
    if (gb.fadeOutStartMs === null) {
      gb.fadeOutStartMs = now;
      touchTime(now, false);
    }
  }

  function commitTransient(
    entity: PlacedEntity,
    revealAt: number,
    ambientGfx: Graphics | null = null,
  ): void {
    const k = entityKey(entity);
    // Record the earliest reveal timestamp for this entity key so the
    // scrub-phase alpha ramp reproduces the original reveal order.
    if (!revealByEntityKey.has(k)) revealByEntityKey.set(k, revealAt);
    if (transientByKey.has(k)) return; // idempotent on the preview gfx
    addEntityToDrawContext(entity, drawCtx);
    const g = drawEntityGraphic(entity, drawCtx);
    g.alpha = 0;
    committedLayer.addChild(g);
    if (ambientGfx) {
      ambientGfx.alpha = 0;
      committedLayer.addChildAt(ambientGfx, 0);
    }
    const committed: TransientCommitted = {
      gfx: g,
      ambientGfx,
      fadeStartMs: revealAt,
      fadeOutStartMs: null,
      entity,
    };
    transientByKey.set(k, committed);
    transientByTile.set(tileKey(entity.x ?? 0, entity.y ?? 0), committed);
    touchTime(revealAt, true);
    // Also fade out any ghost belt at the same tile so the ghost
    // placeholder vanishes as the real entity arrives.
    fadeOutGhostAt(entity.x ?? 0, entity.y ?? 0, revealAt);
  }

  // `handleJunctionCommitted` fades out transient previews inside the
  // zone; track those fade-outs too for the range calculation.
  function fadeOutTransientByKey(k: string, now: number): void {
    const c = transientByKey.get(k);
    if (!c) return;
    if (c.fadeOutStartMs === null) {
      c.fadeOutStartMs = now;
      touchTime(now, false);
    }
  }

  // -------------------------------------------------------------------------
  // Event handlers (live phase)
  // -------------------------------------------------------------------------

  function handleRowsPlaced(entities: PlacedEntity[]): void {
    const now = clock();
    // Pre-populate the draw context with every entity in this batch
    // before drawing any of them. Otherwise `detectBeltTurn` on the
    // first few belts won't see their neighbours, and sideloads /
    // corner belts render as straight.
    for (const e of entities) addEntityToDrawContext(e, drawCtx);
    // Sort NW→SE so the machine placement reveal sweeps top-left →
    // bottom-right at a comfortable rate.
    const sorted = [...entities].sort((a, b) => {
      const dy = (a.y ?? 0) - (b.y ?? 0);
      if (dy !== 0) return dy;
      return (a.x ?? 0) - (b.x ?? 0);
    });
    const s = stagger(sorted.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
    sorted.forEach((e, i) => {
      commitTransient(e, now + i * s);
    });
  }

  function handleGhostSpecRouted(data: {
    spec_key: string;
    tiles: [number, number][];
  }): void {
    const now = clock();
    const item = itemFromSpecKey(data.spec_key);
    const tiles = data.tiles;
    if (tiles.length === 0) return;
    const s = stagger(tiles.length, GHOST_TILE_STAGGER_MAX_MS, GHOST_TILE_STAGGER_BUDGET_MS);
    for (let i = 0; i < tiles.length; i++) {
      const [x, y] = tiles[i];
      // Direction: towards the next tile; for the last tile, infer
      // from the previous tile so the endpoint doesn't render
      // mis-oriented.
      let dx = 0;
      let dy = 0;
      if (i < tiles.length - 1) {
        dx = tiles[i + 1][0] - x;
        dy = tiles[i + 1][1] - y;
      } else if (i > 0) {
        dx = x - tiles[i - 1][0];
        dy = y - tiles[i - 1][1];
      }
      addGhostBelt(
        x,
        y,
        dirFromDelta(dx, dy),
        item,
        data.spec_key,
        now + i * s,
      );
    }
  }

  function handleGhostSpecCommitted(data: {
    spec_key: string;
    entities: PlacedEntity[];
  }): void {
    const now = clock();
    // Populate draw ctx with every entity in this batch up-front so
    // turn detection sees sideload neighbours when rendering each belt.
    for (const e of data.entities) addEntityToDrawContext(e, drawCtx);
    // Sort entities by their path position (approximately) via NW→SE.
    const sorted = [...data.entities].sort((a, b) => {
      const dy = (a.y ?? 0) - (b.y ?? 0);
      if (dy !== 0) return dy;
      return (a.x ?? 0) - (b.x ?? 0);
    });
    const s = stagger(sorted.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
    sorted.forEach((e, i) => {
      commitTransient(e, now + i * s);
    });
    // NOTE: `commitTransient` fades out the ghost belt at each tile.
    // Tiles the spec's render_path skipped (UG-hidden middle tiles)
    // keep their ghost belt at ~50% alpha — the eye-candy feedback
    // "something still flows here". When a junction cluster covers
    // those tiles later, `handleJunctionCommitted` fades them out.
  }

  function handleJunctionCommitted(data: {
    cluster_id: number;
    zone_x: number;
    zone_y: number;
    zone_w: number;
    zone_h: number;
    entities: PlacedEntity[];
    participating: string[];
  }): void {
    const now = clock();
    // Fade out any ghost belt OR prior transient preview inside the
    // zone whose segment id belongs to a participating spec.
    const participating = new Set(data.participating);
    const xMin = data.zone_x;
    const yMin = data.zone_y;
    const xMax = data.zone_x + (data.zone_w as number) - 1;
    const yMax = data.zone_y + (data.zone_h as number) - 1;
    // Ghost belts: fade any within zone.
    for (const [k, gb] of ghostBeltByTile.entries()) {
      const [xs, ys] = k.split(",").map(Number);
      if (xs < xMin || xs > xMax || ys < yMin || ys > yMax) continue;
      if (gb.fadeOutStartMs === null) {
        gb.fadeOutStartMs = now;
        touchTime(now, false);
      }
    }
    // Transient previews with a `ghost:*` segment id from a
    // participating spec: fade out to be replaced by this cluster's
    // SAT entities.
    for (const [k, c] of transientByKey.entries()) {
      if (c.fadeOutStartMs !== null) continue;
      const ex = c.entity.x ?? 0;
      const ey = c.entity.y ?? 0;
      if (ex < xMin || ex > xMax || ey < yMin || ey > yMax) continue;
      const seg = c.entity.segment_id ?? "";
      if (seg.startsWith("ghost:")) {
        const specKey = seg.slice("ghost:".length);
        if (participating.has(specKey)) fadeOutTransientByKey(k, now);
      }
    }

    // Also drop any clusterOverlay rectangle for this cluster.
    for (const gc of ghostClusters) {
      if (gc.clusterId === data.cluster_id) gc.cleared = true;
    }

    // Commit the SAT-placed entities with NW→SE stagger. Pre-populate
    // the draw ctx so belt-turn detection works within this batch.
    for (const e of data.entities) addEntityToDrawContext(e, drawCtx);
    const sorted = [...data.entities].sort((a, b) => {
      const dy = (a.y ?? 0) - (b.y ?? 0);
      if (dy !== 0) return dy;
      return (a.x ?? 0) - (b.x ?? 0);
    });

    // Build a per-cluster UG map so tunnel stripes can be drawn for
    // any paired UG belts the solver placed inside this zone.
    const clusterUgMap = new Map<string, PlacedEntity>();
    for (const e of data.entities) {
      if (e.name?.endsWith("underground-belt")) {
        clusterUgMap.set(tileKey(e.x ?? 0, e.y ?? 0), e);
      }
    }

    const js = stagger(sorted.length, JUNCTION_TILE_STAGGER_MAX_MS, JUNCTION_TILE_STAGGER_BUDGET_MS);
    sorted.forEach((e, i) => {
      const revealAt = now + i * js;
      let ambient: Graphics | null = null;
      if (e.name?.endsWith("underground-belt") && e.io_type === "input") {
        ambient = drawUgTunnelStripe(e, clusterUgMap);
      }
      commitTransient(e, revealAt, ambient);
    });
  }

  function handlePhaseSnapshot(data: {
    phase: string;
    entities: PlacedEntity[];
  }): void {
    anySnapshot = true;
    if (data.phase === "rows_placed") {
      handleRowsPlaced(data.entities);
      return;
    }
    if (data.phase === "lanes_planned") {
      return; // identical set to rows_placed
    }
    if (data.phase === "bus_routed") {
      // Safety net: commit anything not yet seen. In the happy path
      // every belt is already committed via GhostSpecCommitted or
      // JunctionCommitted; this catches edge cases.
      const now = clock();
      const fresh = data.entities.filter((e) => !transientByKey.has(entityKey(e)));
      for (const e of fresh) addEntityToDrawContext(e, drawCtx);
      const s = stagger(fresh.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
      fresh.forEach((e, i) => {
        commitTransient(e, now + i * s);
      });
      return;
    }
    if (data.phase === "poles_placed") {
      const now = clock();
      const fresh = data.entities.filter((e) => !transientByKey.has(entityKey(e)));
      for (const e of fresh) addEntityToDrawContext(e, drawCtx);
      fresh.sort((a, b) => {
        const dy = (a.y ?? 0) - (b.y ?? 0);
        if (dy !== 0) return dy;
        return (a.x ?? 0) - (b.x ?? 0);
      });
      const s = stagger(fresh.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
      fresh.forEach((e, i) => {
        commitTransient(e, now + i * s);
      });
      return;
    }
  }

  function handleGhostClusterSolved(data: {
    cluster_id: number;
    zone_x: number;
    zone_y: number;
    zone_w: number;
    zone_h: number;
  }): void {
    const now = clock();
    ghostClusters.push({
      clusterId: data.cluster_id,
      x: data.zone_x,
      y: data.zone_y,
      w: data.zone_w,
      h: data.zone_h,
      startMs: now,
      cleared: false,
    });
    touchTime(now, true);
  }

  // -------------------------------------------------------------------------
  // Live-phase ticker
  // -------------------------------------------------------------------------
  //
  // Runs while transient graphics exist. Computes per-entity alpha from
  // fadeStartMs / fadeOutStartMs and GCs fully-faded-out graphics. Stops
  // running at `finish()` — scrub-phase alpha is computed on demand by
  // `seekTo`, not per-frame.

  const tick = (): void => {
    if (cancelled) return;
    const now = clock();

    // Ghost belts.
    for (const [k, gb] of ghostBeltByTile.entries()) {
      let alpha: number;
      if (gb.fadeOutStartMs !== null && now >= gb.fadeOutStartMs) {
        const outT = (now - gb.fadeOutStartMs) / FADE_OUT_MS;
        if (outT >= 1) {
          ghostLayer.removeChild(gb.gfx);
          gb.gfx.destroy();
          if (gb.overlayGfx) {
            ghostLayer.removeChild(gb.overlayGfx);
            gb.overlayGfx.destroy();
          }
          ghostBeltByTile.delete(k);
          continue;
        }
        alpha = Math.max(0, 1 - outT);
      } else {
        const inT = Math.min(1, Math.max(0, (now - gb.fadeStartMs) / FADE_IN_MS));
        alpha = inT;
      }
      gb.gfx.alpha = alpha * 0.5;
      if (gb.overlayGfx) gb.overlayGfx.alpha = alpha * 0.5;
    }

    // Transient previews.
    for (const [k, c] of transientByKey.entries()) {
      let alpha: number;
      if (c.fadeOutStartMs !== null && now >= c.fadeOutStartMs) {
        const outT = (now - c.fadeOutStartMs) / FADE_OUT_MS;
        if (outT >= 1) {
          committedLayer.removeChild(c.gfx);
          c.gfx.destroy();
          if (c.ambientGfx) {
            committedLayer.removeChild(c.ambientGfx);
            c.ambientGfx.destroy();
          }
          transientByKey.delete(k);
          const tk = tileKey(c.entity.x ?? 0, c.entity.y ?? 0);
          if (transientByTile.get(tk) === c) transientByTile.delete(tk);
          continue;
        }
        alpha = Math.max(0, 1 - outT);
      } else {
        const inT = Math.min(1, Math.max(0, (now - c.fadeStartMs) / FADE_IN_MS));
        alpha = inT;
      }
      c.gfx.alpha = alpha;
      if (c.ambientGfx) c.ambientGfx.alpha = alpha;
    }

    // Cluster outlines (diagnostic pulse).
    clusterOverlay.clear();
    for (let i = ghostClusters.length - 1; i >= 0; i--) {
      const gc = ghostClusters[i];
      const age = now - gc.startMs;
      if (age < 0) continue;
      if (gc.cleared || age >= GHOST_CLUSTER_LIFETIME_MS) {
        const fadeSince = gc.cleared ? Math.max(age, GHOST_CLUSTER_LIFETIME_MS - GHOST_CLUSTER_FADE_MS) : age;
        if (fadeSince >= GHOST_CLUSTER_LIFETIME_MS) {
          ghostClusters.splice(i, 1);
          continue;
        }
        const t = Math.max(0, 1 - (fadeSince - (GHOST_CLUSTER_LIFETIME_MS - GHOST_CLUSTER_FADE_MS)) / GHOST_CLUSTER_FADE_MS);
        const alpha = 0.9 * t;
        clusterOverlay.rect(gc.x * TILE_PX, gc.y * TILE_PX, gc.w * TILE_PX, gc.h * TILE_PX);
        clusterOverlay.stroke({ width: 2, color: 0x44ccff, alpha });
      } else {
        clusterOverlay.rect(gc.x * TILE_PX, gc.y * TILE_PX, gc.w * TILE_PX, gc.h * TILE_PX);
        clusterOverlay.stroke({ width: 2, color: 0x44ccff, alpha: 0.9 });
      }
    }
  };

  app.ticker.add(tick);

  // -------------------------------------------------------------------------
  // Event dispatcher
  // -------------------------------------------------------------------------

  function onEvent(evt: TraceEvent, onMilestone?: (m: Milestone) => void): void {
    if (cancelled || scrubMode) return;
    pendingMilestoneCallback = onMilestone ?? null;
    if (traceLogs) {
      // eslint-disable-next-line no-console
      console.log(
        `[stream t=${clock().toFixed(0)}] ${evt.phase}`,
        "data" in evt ? (evt as { data: unknown }).data : undefined,
      );
    }
    const nowAtEvent = clock();
    if (firstMs === null) firstMs = nowAtEvent;
    switch (evt.phase) {
      case "PhaseSnapshot": {
        const d = evt.data as { phase: string; entities: PlacedEntity[] };
        if (d.phase === "rows_placed") noteMilestoneNow("machines", nowAtEvent);
        if (d.phase === "poles_placed") noteMilestoneNow("poles", nowAtEvent);
        handlePhaseSnapshot(d);
        break;
      }
      case "GhostSpecRouted":
        noteMilestoneNow("ghost_routes", nowAtEvent);
        handleGhostSpecRouted(evt.data as { spec_key: string; tiles: [number, number][] });
        break;
      case "GhostSpecCommitted":
        noteMilestoneNow("committed_routes", nowAtEvent);
        handleGhostSpecCommitted(evt.data as { spec_key: string; entities: PlacedEntity[] });
        break;
      case "JunctionCommitted":
        noteMilestoneNow("junctions", nowAtEvent);
        handleJunctionCommitted(evt.data as {
          cluster_id: number;
          zone_x: number;
          zone_y: number;
          zone_w: number;
          zone_h: number;
          entities: PlacedEntity[];
          participating: string[];
        });
        break;
      case "GhostClusterSolved":
        handleGhostClusterSolved(evt.data as {
          cluster_id: number;
          zone_x: number;
          zone_y: number;
          zone_w: number;
          zone_h: number;
        });
        break;
      default:
        break;
    }
    pendingMilestoneCallback = null;
  }

  // -------------------------------------------------------------------------
  // Handle
  // -------------------------------------------------------------------------

  return {
    onEvent,
    hasCommittedEntities: () => anySnapshot,
    cancel(): void {
      if (cancelled) return;
      cancelled = true;
      app.ticker.remove(tick);
      committedLayer.removeChildren();
      ghostLayer.removeChildren();
      clusterOverlay.clear();
      ghostBeltByTile.clear();
      transientByKey.clear();
      transientByTile.clear();
      ghostClusters.length = 0;
      reveals = null;
    },

    finish(layout: LayoutResult): HighlightController {
      // Tear down the live-phase ticker — scrub alpha is now imperative.
      app.ticker.remove(tick);

      // `renderLayout` calls `container.removeChildren()` which wipes
      // committedLayer, ghostLayer, clusterOverlay, and every transient
      // graphic they hold. No manual cleanup needed.
      const entityGfxByKey = new Map<string, Graphics[]>();
      const controller = renderLayout(
        layout,
        container,
        onHover,
        onSelect,
        (entity, gfx) => {
          entityGfxByKey.set(entityKey(entity), gfx);
        },
      );

      // Entity-owned graphics register their reveal timestamps. Fall
      // back to `firstMs` for any entity that never appeared in a
      // streamed event (rare — bulk snapshot edge cases). Using
      // `firstMs` keeps the entity visible at settle; using
      // `latestFadeEndMs` would leave it at alpha=0 forever because
      // `scrubVirtualMs == latestFadeEndMs` at settle gives zero
      // elapsed time.
      const fallbackReveal = firstMs ?? 0;
      const list: Reveal[] = [];
      for (const [k, gfxArr] of entityGfxByKey) {
        const revealAt = revealByEntityKey.get(k) ?? fallbackReveal;
        for (const g of gfxArr) list.push({ graphic: g, revealAt });
      }

      // Ambient graphics (UG tunnel stripes drawn directly on
      // `container` before the entity loop) aren't entity-owned. Fade
      // them in at `firstMs` — same "earliest visible" heuristic as
      // `phaseAnimation.ts:190–198`. `firstMs` is already the earliest
      // event time (set once on first event arrival).
      const entityOwned = new Set<Graphics>();
      for (const arr of entityGfxByKey.values()) {
        for (const g of arr) entityOwned.add(g);
      }
      const ambientRevealAt = firstMs ?? 0;
      for (const child of container.children) {
        const g = child as Graphics;
        if (!entityOwned.has(g)) {
          list.push({ graphic: g, revealAt: ambientRevealAt });
        }
      }

      reveals = list;
      scrubMode = true;
      scrubVirtualMs = latestFadeEndMs;

      // Apply alphas at the final virtual clock so the canvas settles
      // on the fully-revealed state.
      applyReveals(scrubVirtualMs);

      return controller;
    },

    seekTo(virtualMs: number): void {
      if (cancelled) return;
      // Ignore seeks before `finish()` was called — scrubber is only
      // armed post-finish in practice.
      if (reveals === null) return;
      const lo = firstMs ?? 0;
      const hi = Math.max(latestFadeEndMs, lo);
      scrubVirtualMs = Math.min(hi, Math.max(lo, virtualMs));
      applyReveals(scrubVirtualMs);
    },

    getTimeRange(): { firstMs: number; lastMs: number } {
      return { firstMs: firstMs ?? 0, lastMs: latestFadeEndMs };
    },
    getMilestones(): Milestone[] {
      return Array.from(milestones.values()).sort(
        (a, b) => a.virtualMs - b.virtualMs,
      );
    },
  };

  // ---------------------------------------------------------------------------
  // seekTo helper: set alpha on every reveal based on `t` vs `revealAt`.
  // ---------------------------------------------------------------------------

  function applyReveals(t: number): void {
    if (reveals === null) return;
    for (const r of reveals) {
      const elapsed = t - r.revealAt;
      const alpha =
        elapsed <= 0 ? 0 :
        elapsed >= FADE_IN_MS ? 1 :
        elapsed / FADE_IN_MS;
      r.graphic.alpha = alpha;
      // Don't let invisible entities capture hover during scrub.
      r.graphic.eventMode = alpha > 0.01 ? "static" : "none";
    }
  }
}
