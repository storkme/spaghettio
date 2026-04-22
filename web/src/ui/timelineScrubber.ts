/**
 * Timeline scrubber — floating bar above the Pixi canvas.
 *
 * Two modes:
 *   1. Live (default): milestone chips arranged evenly across the bar;
 *      a progress fill grows through them as `noteMilestone()` is called.
 *      The bar is non-interactive.
 *   2. Scrub (armed via `arm()` after streaming completes): chips are
 *      repositioned to their actual stream-relative timestamps; a
 *      draggable thumb appears. Dragging fires `onSeek(virtualMs)`.
 *      Milestones act as soft snap points — within SNAP_FRAC of a
 *      milestone the thumb snaps to it.
 *
 * Styling lives in `./timelineScrubber.css` (imported by `main.ts`).
 */

import type { Milestone, MilestoneId } from "../renderer/streamingRenderer";

const SNAP_FRAC = 0.03;
const MIN_DURATION_MS_FOR_SCRUB = 200;

/** Ordered list of every milestone the streaming renderer can produce.
 *  We render chips for all of them in live mode; if a layout doesn't
 *  produce one (e.g. no junctions), that chip stays un-reached. */
const MILESTONE_ORDER: MilestoneId[] = [
  "machines",
  "ghost_routes",
  "committed_routes",
  "junctions",
  "poles",
];

const MILESTONE_LABELS: Record<MilestoneId, string> = {
  machines: "Machines",
  ghost_routes: "Ghost Routes",
  committed_routes: "Committed Routes",
  junctions: "Junctions",
  poles: "Poles",
};

export interface TimelineScrubberHandle {
  /** Live mode: mark this milestone as reached. Advances progress. */
  noteMilestone(id: MilestoneId): void;
  /** Flip to scrub mode. `range` is the [firstMs, lastMs] virtual window;
   *  `milestones` is the list of milestones with their absolute virtual-ms
   *  timestamps. Milestones get positioned proportionally along the bar. */
  arm(range: { firstMs: number; lastMs: number }, milestones: Milestone[]): void;
  /** Go back to empty-live state, hide the bar. */
  reset(): void;
  destroy(): void;
}

export function createTimelineScrubber(
  container: HTMLElement,
  onSeek: (virtualMs: number) => void,
): TimelineScrubberHandle {
  const root = document.createElement("div");
  root.className = "timeline-scrubber";

  const chipsRow = document.createElement("div");
  chipsRow.className = "ts-chips";
  root.appendChild(chipsRow);

  const bar = document.createElement("div");
  bar.className = "ts-bar";
  root.appendChild(bar);

  const track = document.createElement("div");
  track.className = "ts-track";
  bar.appendChild(track);

  const fill = document.createElement("div");
  fill.className = "ts-fill";
  bar.appendChild(fill);

  const thumb = document.createElement("div");
  thumb.className = "ts-thumb";
  bar.appendChild(thumb);

  container.appendChild(root);

  // Chip elements indexed by milestone id.
  const chips = new Map<MilestoneId, HTMLDivElement>();
  for (const id of MILESTONE_ORDER) {
    const chip = document.createElement("div");
    chip.className = "ts-chip";
    chip.dataset["milestone"] = id;
    chip.textContent = MILESTONE_LABELS[id];
    chipsRow.appendChild(chip);
    chips.set(id, chip);
  }

  // State shared between modes.
  let armed = false;
  let range: { firstMs: number; lastMs: number } | null = null;
  let milestoneByFrac: Array<{ id: MilestoneId; frac: number }> = [];
  const reached = new Set<MilestoneId>();
  let activeChipId: MilestoneId | null = null;

  function setFillToFraction(frac: number): void {
    fill.style.width = `${frac * 100}%`;
    thumb.style.left = `${frac * 100}%`;
  }

  function setActiveChip(id: MilestoneId | null): void {
    if (activeChipId === id) return;
    if (activeChipId) chips.get(activeChipId)?.classList.remove("ts-chip--active");
    if (id) chips.get(id)?.classList.add("ts-chip--active");
    activeChipId = id;
  }

  function noteMilestone(id: MilestoneId): void {
    if (armed) return; // ignore during scrub mode — bar is user-controlled
    reached.add(id);
    chips.get(id)?.classList.add("ts-chip--reached");
    setActiveChip(id);
    // Progress fill: fraction of MILESTONE_ORDER reached.
    const idx = MILESTONE_ORDER.indexOf(id);
    setFillToFraction((idx + 1) / MILESTONE_ORDER.length);
    root.classList.add("ts-visible");
  }

  // ---------------------------------------------------------------------
  // Scrub mode
  // ---------------------------------------------------------------------

  function fracToVirtualMs(frac: number): number {
    if (!range) return 0;
    return range.firstMs + frac * (range.lastMs - range.firstMs);
  }

  function applySnap(frac: number): { frac: number; snapped: boolean } {
    for (const m of milestoneByFrac) {
      if (Math.abs(frac - m.frac) < SNAP_FRAC) return { frac: m.frac, snapped: true };
    }
    return { frac, snapped: false };
  }

  function updateFromPointer(clientX: number): void {
    if (!range) return;
    const rect = bar.getBoundingClientRect();
    const raw = (clientX - rect.left) / rect.width;
    const clamped = Math.min(1, Math.max(0, raw));
    const { frac, snapped } = applySnap(clamped);
    setFillToFraction(frac);
    if (snapped) {
      thumb.classList.add("ts-thumb--snapped");
    } else {
      thumb.classList.remove("ts-thumb--snapped");
    }
    onSeek(fracToVirtualMs(frac));
  }

  let activeMove: ((e: PointerEvent) => void) | null = null;
  let activeUp: ((e: PointerEvent) => void) | null = null;

  function onPointerDown(e: PointerEvent): void {
    if (!armed || !range) return;
    e.preventDefault();
    try {
      bar.setPointerCapture(e.pointerId);
    } catch {
      // Pointer-capture can fail silently on older engines; drag still
      // works via document listeners below.
    }
    const move = (ev: PointerEvent): void => updateFromPointer(ev.clientX);
    const up = (_ev: PointerEvent): void => {
      if (activeMove) document.removeEventListener("pointermove", activeMove);
      if (activeUp) document.removeEventListener("pointerup", activeUp);
      activeMove = null;
      activeUp = null;
      thumb.classList.remove("ts-thumb--snapped");
    };
    activeMove = move;
    activeUp = up;
    document.addEventListener("pointermove", move);
    document.addEventListener("pointerup", up, { once: true });
    updateFromPointer(e.clientX);
  }

  bar.addEventListener("pointerdown", onPointerDown);

  // ---------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------

  function arm(
    newRange: { firstMs: number; lastMs: number },
    milestones: Milestone[],
  ): void {
    const duration = newRange.lastMs - newRange.firstMs;
    if (duration < MIN_DURATION_MS_FOR_SCRUB || milestones.length === 0) {
      // Stream was too short to be worth scrubbing — hide.
      reset();
      return;
    }
    armed = true;
    range = newRange;
    root.classList.add("ts-scrub-mode");
    root.classList.add("ts-visible");
    milestoneByFrac = milestones.map((m) => ({
      id: m.id,
      frac: (m.virtualMs - newRange.firstMs) / duration,
    }));
    // Reposition chips to their true positions along the bar.
    chipsRow.style.justifyContent = "flex-start";
    chipsRow.style.position = "relative";
    for (const chip of chips.values()) {
      chip.style.position = "absolute";
      chip.style.transform = "translateX(-50%)";
    }
    // Clear any chip positioned from live mode.
    for (const id of MILESTONE_ORDER) {
      const chip = chips.get(id);
      if (!chip) continue;
      const found = milestoneByFrac.find((m) => m.id === id);
      if (found) {
        chip.style.left = `${found.frac * 100}%`;
        chip.style.display = "";
        chip.classList.add("ts-chip--reached");
      } else {
        chip.style.display = "none";
      }
    }
    // Rebuild the track tick-marks at true milestone positions so the
    // user can see where snap points live even if labels get shifted
    // for readability below.
    rebuildTicks();
    // Collision-resolve chip labels so overlapping milestones (common
    // when ghost-routing and committed-routing both finish within a
    // few ms of each other) stay legible. Shifts are purely visual:
    // the scrubber still snaps to the original `milestoneByFrac`
    // positions.
    requestAnimationFrame(resolveChipCollisions);
    // Initial thumb at end-of-stream (matches what the canvas shows
    // after finish()).
    setFillToFraction(1);
    setActiveChip(null);
  }

  // Independent tick marks rendered on the track at true milestone
  // positions. Chips labels may shift for readability, but the ticks
  // always sit at the real snap points.
  let ticksLayer: HTMLDivElement | null = null;
  function rebuildTicks(): void {
    if (ticksLayer) ticksLayer.remove();
    if (!range) return;
    const layer = document.createElement("div");
    layer.className = "ts-ticks";
    for (const m of milestoneByFrac) {
      const t = document.createElement("div");
      t.className = "ts-tick";
      t.style.left = `${m.frac * 100}%`;
      layer.appendChild(t);
    }
    bar.appendChild(layer);
    ticksLayer = layer;
  }

  function resolveChipCollisions(): void {
    if (!armed) return;
    const gap = 6;
    const barWidth = chipsRow.clientWidth;
    if (barWidth <= 0) return;
    const entries = MILESTONE_ORDER
      .map((id) => {
        const el = chips.get(id);
        if (!el || el.style.display === "none") return null;
        const originalFrac = milestoneByFrac.find((m) => m.id === id)?.frac ?? 0;
        return { el, originalFrac };
      })
      .filter((x): x is { el: HTMLDivElement; originalFrac: number } => x !== null);
    // Left-to-right collision resolution. Each chip gets pushed right
    // so its left edge clears the previous chip's right edge + gap.
    let cursorPx = -Infinity;
    for (const { el, originalFrac } of entries) {
      const halfW = el.offsetWidth / 2;
      const desiredCenter = originalFrac * barWidth;
      const minCenter = cursorPx + halfW + gap;
      const centerPx = Math.max(desiredCenter, minCenter);
      el.style.left = `${(centerPx / barWidth) * 100}%`;
      cursorPx = centerPx + halfW;
    }
  }

  function reset(): void {
    armed = false;
    range = null;
    milestoneByFrac = [];
    reached.clear();
    activeChipId = null;
    if (ticksLayer) {
      ticksLayer.remove();
      ticksLayer = null;
    }
    root.classList.remove("ts-visible", "ts-scrub-mode");
    fill.style.width = "0";
    thumb.style.left = "0";
    thumb.classList.remove("ts-thumb--snapped");
    // Restore chips to even-spaced live-mode layout.
    chipsRow.style.justifyContent = "space-between";
    chipsRow.style.position = "";
    for (const chip of chips.values()) {
      chip.style.position = "";
      chip.style.transform = "";
      chip.style.left = "";
      chip.style.display = "";
      chip.classList.remove("ts-chip--reached", "ts-chip--active");
    }
  }

  function destroy(): void {
    if (activeMove) document.removeEventListener("pointermove", activeMove);
    if (activeUp) document.removeEventListener("pointerup", activeUp);
    bar.removeEventListener("pointerdown", onPointerDown);
    root.remove();
  }

  return { noteMilestone, arm, reset, destroy };
}
