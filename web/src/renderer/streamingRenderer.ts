/**
 * Streaming layout renderer — Phase 2+3+4.
 *
 * Every entity type renders as a particle. Ghost belts use the
 * `ghostContainer` in the particle scene. `finish()` stops the live
 * ticker and builds a particle-aware `HighlightController`; it no
 * longer calls `renderLayout` or does any container surgery.
 *
 * ## Live phase
 *
 * As trace events arrive from the WASM worker, entity particles are
 * committed to the scene with staggered `revealAt` timestamps. The
 * live-phase ticker mutates `particle.alpha` each frame.
 *
 * Ghost-belt previews appear in the `ghostContainer` and fade out
 * when a committed entity lands on the same tile.
 *
 * ## Scrub phase (after finish())
 *
 * `finish(layout)` stops the ticker, builds the particle highlight
 * controller from `layout.entities`, and returns it. `seekTo(t)`
 * walks the `reveals` list (all particles) and sets alpha imperatively.
 *
 * ## Debug logging
 *
 * Set `window.__ANIM_LOGS = true` (or tick the debug checkbox) to
 * enable `[anim …]` console logs at every animation-start site.
 */

import type { Application, Container } from "pixi.js";
import { Graphics as PixiGraphics, Particle } from "pixi.js";
import type { LayoutResult } from "../engine";
import type { PlacedEntity, TraceEvent, EntityDirection } from "../wasm-pkg/fucktorio_wasm";
import {
  addEntityToDrawContext,
  createDrawContext,
  TILE_PX,
  type DrawContext,
  type HighlightController,
} from "./entities";
import { beginAnimating, endAnimating, requestRender } from "./app";
import {
  createParticleScene,
  commitEntityAsParticle,
  applyParticleReveals,
  getParticleReveals,
  createParticleHighlightController,
  addGhostParticle,
  removeGhostParticle,
  entityKey,
  type ParticleScene,
} from "./particleLayout";

// ---------------------------------------------------------------------------
// Timings
// ---------------------------------------------------------------------------

const FADE_IN_MS = 150;
const FADE_OUT_MS = 80;
const GHOST_TILE_STAGGER_MAX_MS = 4;
const GHOST_TILE_STAGGER_BUDGET_MS = 300;
const COMMITTED_TILE_STAGGER_MAX_MS = 4;
const COMMITTED_TILE_STAGGER_BUDGET_MS = 900;
const JUNCTION_TILE_STAGGER_MAX_MS = 6;
const JUNCTION_TILE_STAGGER_BUDGET_MS = 250;
const GHOST_CLUSTER_LIFETIME_MS = 800;
const GHOST_CLUSTER_FADE_MS = 250;

function stagger(count: number, maxMs: number, budgetMs: number): number {
  if (count <= 1) return maxMs;
  return Math.min(maxMs, budgetMs / count);
}

// ---------------------------------------------------------------------------
// Key helpers
// ---------------------------------------------------------------------------

function tileKey(x: number, y: number): string {
  return `${x},${y}`;
}

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
// Ghost state
// ---------------------------------------------------------------------------

interface GhostBeltState {
  specKey: string;
  fadeOutStartMs: number | null;
  fadeStartMs: number;
}

// ---------------------------------------------------------------------------
// Cluster outline state
// ---------------------------------------------------------------------------

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

/** Scrub-mode entry — now all particles, no Graphics branch. */
type Reveal = {
  kind: "particle";
  particle: Particle;
  iconParticle: Particle | undefined;
  revealAt: number;
};

// ---------------------------------------------------------------------------
// Debug logging
// ---------------------------------------------------------------------------

// (animLog is a closure-local helper defined inside createStreamingRenderer
// so it captures `clock` for the `t=` field.)

// ---------------------------------------------------------------------------
// Public handle
// ---------------------------------------------------------------------------

export type MilestoneId =
  | "machines"
  | "ghost_routes"
  | "committed_routes"
  | "junctions"
  | "poles"
  | "optimizing";

export interface Milestone {
  id: MilestoneId;
  virtualMs: number;
}

export interface StreamingRendererHandle {
  onEvent(evt: TraceEvent, onMilestone?: (m: Milestone) => void): void;
  hasCommittedEntities(): boolean;
  cancel(): void;
  finish(layout: LayoutResult): HighlightController;
  seekTo(virtualMs: number): void;
  getTimeRange(): { firstMs: number; lastMs: number };
  getMilestones(): Milestone[];
}

// ---------------------------------------------------------------------------
// Main factory
// ---------------------------------------------------------------------------

export function createStreamingRenderer(
  container: Container,
  app: Application,
  _onHover?: (e: PlacedEntity | null) => void,
  _onSelect?: (e: PlacedEntity | null) => void,
): StreamingRendererHandle {
  // Layer ordering (bottom-to-top):
  //   0. particleScene.beltContainer    — belt/pipe/inserter/pole particles
  //   1. particleScene.machineContainer — machine particles (above belts)
  //   2. particleScene.ghostContainer   — ghost belt previews
  //   3. particleScene.iconContainer    — item icon particles
  //   4. clusterOverlay                 — SAT cluster outline pulses (Graphics, cheap)
  //
  // Particle containers attach first via `attachTo`; clusterOverlay is added after.
  const particleScene: ParticleScene = createParticleScene();
  particleScene.attachTo(container);

  const clusterOverlay = new PixiGraphics();
  container.addChild(clusterOverlay);

  // Ghost belt state map: tileKey → GhostBeltState.
  const ghostStateByTile = new Map<string, GhostBeltState>();

  // Committed entity keys (used by the PhaseSnapshot safety net to skip
  // entities already routed through the particle path).
  const committedKeys = new Set<string>();

  // First-seen virtual-ms for each entity key — drives scrub-mode alpha.
  const revealByEntityKey = new Map<string, number>();

  const drawCtx: DrawContext = createDrawContext();

  let cancelled = false;
  let anySnapshot = false;

  let scrubMode = false;
  let scrubVirtualMs = 0;
  const clock = (): number => (scrubMode ? scrubVirtualMs : performance.now());

  let firstMs: number | null = null;
  let latestFadeEndMs = 0;
  const milestones = new Map<MilestoneId, Milestone>();
  let pendingMilestoneCallback: ((m: Milestone) => void) | null = null;

  let reveals: Reveal[] | null = null;

  const ghostClusters: GhostCluster[] = [];

  const traceLogs =
    (globalThis as { __TRACE_LOGS?: boolean }).__TRACE_LOGS === true;

  // ---------------------------------------------------------------------------
  // Debug animation logging (Phase 4)
  // ---------------------------------------------------------------------------

  const animLog = (phase: string, detail: Record<string, unknown>): void => {
    if (!(globalThis as { __ANIM_LOGS?: boolean }).__ANIM_LOGS) return;
    // eslint-disable-next-line no-console
    console.log(`[anim t=${clock().toFixed(0)}ms] ${phase}`, detail);
  };

  // ---------------------------------------------------------------------------
  // Time / milestone bookkeeping
  // ---------------------------------------------------------------------------

  function noteMilestoneNow(id: MilestoneId, at: number): void {
    const existing = milestones.get(id);
    const isNew = !existing;
    const m: Milestone = { id, virtualMs: at };
    milestones.set(id, m);
    if (isNew) pendingMilestoneCallback?.(m);
  }

  function touchTime(at: number, fadeIn: boolean): void {
    if (firstMs === null) firstMs = at;
    const end = at + (fadeIn ? FADE_IN_MS : FADE_OUT_MS);
    if (end > latestFadeEndMs) latestFadeEndMs = end;
  }

  // ---------------------------------------------------------------------------
  // Entity commit helpers
  // ---------------------------------------------------------------------------

  function commitEntitiesAsParticles(entities: PlacedEntity[], staggerMs: number): void {
    if (entities.length === 0) return;
    const now = clock();
    for (const e of entities) addEntityToDrawContext(e, drawCtx);
    const sorted = [...entities].sort((a, b) => {
      const dy = (a.y ?? 0) - (b.y ?? 0);
      return dy !== 0 ? dy : (a.x ?? 0) - (b.x ?? 0);
    });
    sorted.forEach((e, i) => {
      const revealAt = now + i * staggerMs;
      const k = entityKey(e);
      if (committedKeys.has(k)) return; // idempotent
      committedKeys.add(k);
      commitEntityAsParticle(particleScene, e, revealAt, drawCtx);
      if (!revealByEntityKey.has(k)) revealByEntityKey.set(k, revealAt);
      // Fade out any ghost at the same tile.
      removeGhostParticle(particleScene, e.x ?? 0, e.y ?? 0);
      const gs = ghostStateByTile.get(tileKey(e.x ?? 0, e.y ?? 0));
      if (gs && gs.fadeOutStartMs === null) {
        gs.fadeOutStartMs = revealAt;
        touchTime(revealAt, false);
      }
    });
    touchTime(now + (sorted.length - 1) * staggerMs, true);
  }

  // ---------------------------------------------------------------------------
  // Ghost belt helpers
  // ---------------------------------------------------------------------------

  function addGhostAt(
    x: number,
    y: number,
    direction: EntityDirection,
    item: string,
    specKey: string,
    revealAt: number,
  ): void {
    const tk = tileKey(x, y);
    const existing = ghostStateByTile.get(tk);
    if (existing && existing.specKey === specKey) return;

    addGhostParticle(particleScene, x, y, direction, item, revealAt, specKey);
    if (!existing) {
      ghostStateByTile.set(tk, { specKey, fadeStartMs: revealAt, fadeOutStartMs: null });
    }
    touchTime(revealAt, true);
  }

  // ---------------------------------------------------------------------------
  // Event handlers
  // ---------------------------------------------------------------------------

  function handleRowsPlaced(entities: PlacedEntity[]): void {
    const s = stagger(entities.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
    animLog("rows_placed", { count: entities.length, stagger_ms: s, span_ms: entities.length * s });
    commitEntitiesAsParticles(entities, s);
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
    animLog("ghost_routed", { spec_key: data.spec_key, item, tile_count: tiles.length, span_ms: tiles.length * s });
    for (let i = 0; i < tiles.length; i++) {
      const [x, y] = tiles[i];
      let dx = 0, dy = 0;
      if (i < tiles.length - 1) {
        dx = tiles[i + 1][0] - x;
        dy = tiles[i + 1][1] - y;
      } else if (i > 0) {
        dx = x - tiles[i - 1][0];
        dy = y - tiles[i - 1][1];
      }
      addGhostAt(x, y, dirFromDelta(dx, dy), item, data.spec_key, now + i * s);
    }
  }

  function handleGhostSpecCommitted(data: {
    spec_key: string;
    entities: PlacedEntity[];
  }): void {
    const count = data.entities.length;
    const s = stagger(count, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
    animLog("committed", { source: "spec", count, span_ms: count * s });
    commitEntitiesAsParticles(data.entities, s);
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
    // Fade out ghost belts inside the zone.
    const xMin = data.zone_x, yMin = data.zone_y;
    const xMax = data.zone_x + data.zone_w - 1;
    const yMax = data.zone_y + data.zone_h - 1;
    for (const [k, gs] of ghostStateByTile.entries()) {
      const [xs, ys] = k.split(",").map(Number);
      if (xs < xMin || xs > xMax || ys < yMin || ys > yMax) continue;
      if (gs.fadeOutStartMs === null) {
        gs.fadeOutStartMs = now;
        touchTime(now, false);
        removeGhostParticle(particleScene, xs, ys);
      }
    }

    // Also drop any clusterOverlay rectangle for this cluster.
    for (const gc of ghostClusters) {
      if (gc.clusterId === data.cluster_id) gc.cleared = true;
    }

    const count = data.entities.length;
    const js = stagger(count, JUNCTION_TILE_STAGGER_MAX_MS, JUNCTION_TILE_STAGGER_BUDGET_MS);
    animLog("junction", { cluster_id: data.cluster_id, zone: `${data.zone_x},${data.zone_y}+${data.zone_w}x${data.zone_h}`, count, span_ms: count * js });
    commitEntitiesAsParticles(data.entities, js);
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
      return;
    }
    if (data.phase === "bus_routed") {
      // Safety net: commit anything not yet seen via streaming events.
      const fresh = data.entities.filter((e) => !committedKeys.has(entityKey(e)));
      if (fresh.length > 0) {
        const s = stagger(fresh.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
        commitEntitiesAsParticles(fresh, s);
      }
      return;
    }
    if (data.phase === "poles_placed") {
      const fresh = data.entities.filter((e) => !committedKeys.has(entityKey(e)));
      if (fresh.length > 0) {
        const s = stagger(fresh.length, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
        commitEntitiesAsParticles(fresh, s);
      }
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
    animLog("cluster_outline", {
      cluster_id: data.cluster_id,
      zone: `${data.zone_x},${data.zone_y}+${data.zone_w}x${data.zone_h}`,
      lifetime_ms: GHOST_CLUSTER_LIFETIME_MS,
      fade_ms: GHOST_CLUSTER_FADE_MS,
    });
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

  // ---------------------------------------------------------------------------
  // Live-phase ticker
  // ---------------------------------------------------------------------------

  const tick = (): void => {
    if (cancelled) return;
    const now = clock();

    // Entity particle reveals.
    applyParticleReveals(particleScene, now);

    // Ghost belt alpha (fade in then fade out as committed entities replace them).
    for (const [tk, gs] of ghostStateByTile.entries()) {
      if (gs.fadeOutStartMs !== null && now >= gs.fadeOutStartMs) {
        // Ghost has been replaced; its particle was already removed by
        // removeGhostParticle in commitEntitiesAsParticles. Just clean up state.
        const [xs, ys] = tk.split(",").map(Number);
        const outT = (now - gs.fadeOutStartMs) / FADE_OUT_MS;
        if (outT >= 1) {
          ghostStateByTile.delete(tk);
        }
        // Ghost particle was already removed when the committed entity landed.
        void xs; void ys;
      }
      // Ghost alpha is set when the particle was created at revealAt;
      // the particle's alpha starts at 0 and is faded in by the ghost system below.
    }

    // Drive ghost particle alphas (fade in only — fade-out is handled by removal).
    for (const p of particleScene.ghostContainer.particleChildren as Particle[]) {
      // Ghost particles fade in over FADE_IN_MS from creation. Since we don't
      // track their revealAt here easily, ramp them toward 0.5 max alpha
      // once they've been in the scene. Use the particle's current alpha as state.
      if (p.alpha < 0.5) {
        p.alpha = Math.min(0.5, p.alpha + (16 / FADE_IN_MS));
      }
    }

    // Cluster outlines.
    clusterOverlay.clear();
    for (let i = ghostClusters.length - 1; i >= 0; i--) {
      const gc = ghostClusters[i];
      const age = now - gc.startMs;
      if (age < 0) continue;
      if (gc.cleared || age >= GHOST_CLUSTER_LIFETIME_MS) {
        const fadeSince = gc.cleared
          ? Math.max(age, GHOST_CLUSTER_LIFETIME_MS - GHOST_CLUSTER_FADE_MS)
          : age;
        if (fadeSince >= GHOST_CLUSTER_LIFETIME_MS) {
          ghostClusters.splice(i, 1);
          continue;
        }
        const t = Math.max(0, 1 - (fadeSince - (GHOST_CLUSTER_LIFETIME_MS - GHOST_CLUSTER_FADE_MS)) / GHOST_CLUSTER_FADE_MS);
        clusterOverlay.rect(gc.x * TILE_PX, gc.y * TILE_PX, gc.w * TILE_PX, gc.h * TILE_PX);
        clusterOverlay.stroke({ width: 2, color: 0x44ccff, alpha: 0.9 * t });
      } else {
        clusterOverlay.rect(gc.x * TILE_PX, gc.y * TILE_PX, gc.w * TILE_PX, gc.h * TILE_PX);
        clusterOverlay.stroke({ width: 2, color: 0x44ccff, alpha: 0.9 });
      }
    }
  };

  app.ticker.add(tick);
  beginAnimating();
  let tickerActive = true;

  // streaming start log
  animLog("streaming_start", {});

  // ---------------------------------------------------------------------------
  // Event dispatcher
  // ---------------------------------------------------------------------------

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
      // Stream-sibling events — all entities now go through the particle path.
      case "TrunkBeltCommitted": {
        const data = evt.data as { item: string; lane_x: number; is_fluid: boolean; entities: PlacedEntity[] };
        const count = data.entities.length;
        const s = stagger(count, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
        animLog("committed", { source: "trunk", count, span_ms: count * s });
        commitEntitiesAsParticles(data.entities, s);
        break;
      }
      case "BalancerCommitted": {
        const data = evt.data as { entities: PlacedEntity[] };
        const count = data.entities.length;
        const s = stagger(count, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
        animLog("committed", { source: "balancer", count, span_ms: count * s });
        commitEntitiesAsParticles(data.entities, s);
        break;
      }
      case "OutputMergerCommitted": {
        const data = evt.data as { entities: PlacedEntity[] };
        const count = data.entities.length;
        const s = stagger(count, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
        animLog("committed", { source: "merger", count, span_ms: count * s });
        commitEntitiesAsParticles(data.entities, s);
        break;
      }
      case "PolesCommitted": {
        const data = evt.data as { entities: PlacedEntity[] };
        const count = data.entities.length;
        const s = stagger(count, COMMITTED_TILE_STAGGER_MAX_MS, COMMITTED_TILE_STAGGER_BUDGET_MS);
        animLog("committed", { source: "poles", count, span_ms: count * s });
        commitEntitiesAsParticles(data.entities, s);
        break;
      }
      default:
        break;
    }
    pendingMilestoneCallback = null;
  }

  // ---------------------------------------------------------------------------
  // Handle
  // ---------------------------------------------------------------------------

  return {
    onEvent,
    hasCommittedEntities: () => anySnapshot,

    cancel(): void {
      if (cancelled) return;
      cancelled = true;
      app.ticker.remove(tick);
      if (tickerActive) { endAnimating(); tickerActive = false; }
      particleScene.clear();
      ghostStateByTile.clear();
      committedKeys.clear();
      ghostClusters.length = 0;
      clusterOverlay.clear();
      reveals = null;
      requestRender();
    },

    finish(layout: LayoutResult): HighlightController {
      // Stop live ticker — scrub alpha is now imperative via seekTo.
      app.ticker.remove(tick);
      if (tickerActive) { endAnimating(); tickerActive = false; }

      animLog("streaming_finish", {
        entity_count: particleScene.count(),
        latest_fade_end_ms: latestFadeEndMs,
      });

      // Build the scrub-mode reveals list from the particle map.
      const fallbackReveal = firstMs ?? 0;
      const list: Reveal[] = [];
      for (const { particle, iconParticle, revealAt } of getParticleReveals(particleScene)) {
        list.push({ kind: "particle", particle, iconParticle, revealAt });
      }
      // Any entities in layout.entities that never appeared in streamed events
      // (rare edge case) aren't in the particle map yet. Commit them now
      // and add them to the reveals list.
      const missingEntities = layout.entities.filter((e: PlacedEntity) => !committedKeys.has(entityKey(e)));
      if (missingEntities.length > 0) {
        for (const e of missingEntities) {
          addEntityToDrawContext(e, drawCtx);
        }
        for (const e of missingEntities) {
          commitEntityAsParticle(particleScene, e, fallbackReveal, drawCtx);
          committedKeys.add(entityKey(e));
        }
        // Now collect just the newly-added particles by re-iterating and
        // picking up anything not already in list (by checking against
        // the pre-built set).
        const alreadyInList = new Set(list.map(r => r.particle));
        for (const { particle, iconParticle, revealAt } of getParticleReveals(particleScene)) {
          if (!alreadyInList.has(particle)) {
            list.push({ kind: "particle", particle, iconParticle, revealAt });
          }
        }
      }

      reveals = list;
      scrubMode = true;
      scrubVirtualMs = latestFadeEndMs;

      // Apply alphas at the final virtual clock so the canvas settles.
      applyReveals(scrubVirtualMs);

      // Build particle-aware HighlightController from the final layout.
      return createParticleHighlightController(layout, app);
    },

    seekTo(virtualMs: number): void {
      if (cancelled) return;
      if (reveals === null) return;
      const lo = firstMs ?? 0;
      const hi = Math.max(latestFadeEndMs, lo);
      scrubVirtualMs = Math.min(hi, Math.max(lo, virtualMs));
      animLog("scrub", { virtualMs: scrubVirtualMs });
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
  // seekTo helper
  // ---------------------------------------------------------------------------

  function applyReveals(t: number): void {
    if (reveals === null) return;
    for (const r of reveals) {
      const elapsed = t - r.revealAt;
      const alpha =
        elapsed <= 0 ? 0 :
        elapsed >= FADE_IN_MS ? 1 :
        elapsed / FADE_IN_MS;
      r.particle.alpha = alpha;
      if (r.iconParticle) r.iconParticle.alpha = alpha;
    }
    requestRender();
  }
}
