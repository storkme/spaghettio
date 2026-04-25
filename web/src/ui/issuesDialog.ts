import { Graphics, type Application } from "pixi.js";
import type { Viewport } from "pixi-viewport";
import { VALIDATION_BORDER_ALPHA } from "../renderer/validationOverlay";
import { TILE_PX } from "../renderer/entities";
import { beginAnimating, endAnimating, requestRender } from "../renderer/app";
import "./issuesDialog.css";

export interface ValidationIssueItem {
  severity: "Error" | "Warning";
  category: string;
  message: string;
  x?: number;
  y?: number;
}

export interface IssuesDialogControls {
  populate(issues: ValidationIssueItem[], debugOn: boolean): void;
  setVisible(visible: boolean): void;
  setCircleMap(map: Map<string, Graphics[]>): void;
  clearPulse(): void;
  panel: HTMLElement;
  onValClose: (() => void) | null;
  setOnValClose(cb: () => void): void;
}

export function createIssuesDialog(
  container: HTMLElement,
  app: Application,
  viewport: Viewport,
): IssuesDialogControls {
  const issuesPanel = document.createElement("div");
  issuesPanel.className = "issues-panel";
  container.appendChild(issuesPanel);

  const titleBar = document.createElement("div");
  titleBar.className = "issues-title-bar";

  const titleText = document.createElement("span");
  titleText.className = "issues-title-text";
  titleText.textContent = "Validation";
  titleBar.appendChild(titleText);

  const countBadge = document.createElement("span");
  countBadge.className = "issues-count-badge";
  titleBar.appendChild(countBadge);

  const closeBtn = document.createElement("span");
  closeBtn.className = "issues-close-btn";
  closeBtn.textContent = "\u00d7";
  titleBar.appendChild(closeBtn);

  issuesPanel.appendChild(titleBar);

  const body = document.createElement("div");
  body.className = "issues-body";
  issuesPanel.appendChild(body);

  // Dragging
  {
    let dragging = false;
    let offsetX = 0;
    let offsetY = 0;
    titleBar.addEventListener("pointerdown", (e) => {
      if ((e.target as HTMLElement) === closeBtn) return;
      dragging = true;
      const rect = issuesPanel.getBoundingClientRect();
      const containerRect = container.getBoundingClientRect();
      offsetX = e.clientX - rect.left + containerRect.left;
      offsetY = e.clientY - rect.top + containerRect.top;
      titleBar.setPointerCapture(e.pointerId);
      e.preventDefault();
    });
    titleBar.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      issuesPanel.style.left = `${e.clientX - offsetX}px`;
      issuesPanel.style.top = `${e.clientY - offsetY}px`;
      issuesPanel.style.right = "auto";
    });
    titleBar.addEventListener("pointerup", () => { dragging = false; });
  }

  let circleMap: Map<string, Graphics[]> = new Map();
  let activePulse: {
    markers: Graphics[];
    ring: Graphics;
    tickerFn: () => void;
  } | null = null;
  let pinnedRow: HTMLDivElement | null = null;
  let onValCloseCb: (() => void) | null = null;

  function clearPulse(): void {
    if (activePulse) {
      for (const m of activePulse.markers) m.alpha = VALIDATION_BORDER_ALPHA;
      app.ticker.remove(activePulse.tickerFn);
      activePulse.ring.destroy();
      endAnimating();
      activePulse = null;
      requestRender();
    }
  }

  /** Pulse the validation border + spawn an expanding ring radiating
   *  out from the tile to draw the eye, even far off-screen mid-pan. */
  function pulseCircle(key: string, tileX: number, tileY: number): void {
    clearPulse();
    const markers = circleMap.get(key);
    if (!markers || markers.length === 0) return;

    const cx = tileX * TILE_PX + TILE_PX / 2;
    const cy = tileY * TILE_PX + TILE_PX / 2;
    const ring = new Graphics();
    viewport.addChild(ring);

    const RING_PERIOD_MS = 900; // one expand cycle
    const RING_MAX_R = TILE_PX * 4;
    const BLINK_PERIOD_MS = 220;
    let elapsed = 0;
    let blinkElapsed = 0;
    let blinkOn = true;

    const tickerFn = (): void => {
      const dt = app.ticker.deltaMS;
      elapsed += dt;
      blinkElapsed += dt;

      // Border blink (fast)
      if (blinkElapsed >= BLINK_PERIOD_MS) {
        blinkElapsed -= BLINK_PERIOD_MS;
        blinkOn = !blinkOn;
        const alpha = blinkOn ? 1.0 : VALIDATION_BORDER_ALPHA;
        for (const m of markers) m.alpha = alpha;
      }

      // Expanding ring (slow). Loops a few times then settles.
      const phase = (elapsed % RING_PERIOD_MS) / RING_PERIOD_MS;
      const r = phase * RING_MAX_R;
      const ringAlpha = (1 - phase) * 0.7;
      ring.clear();
      ring.circle(cx, cy, r).stroke({ width: 3, color: 0xff4444, alpha: ringAlpha });
    };
    app.ticker.add(tickerFn);
    beginAnimating();
    activePulse = { markers, ring, tickerFn };
  }

  function unpinRow(): void {
    if (pinnedRow) {
      pinnedRow.style.background = "";
      pinnedRow = null;
    }
    clearPulse();
  }

  closeBtn.addEventListener("click", () => {
    onValCloseCb?.();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") unpinRow();
  });
  document.addEventListener("pointerdown", (e) => {
    if (pinnedRow && !issuesPanel.contains(e.target as Node)) unpinRow();
  });

  function populate(issues: ValidationIssueItem[], debugOn: boolean): void {
    body.innerHTML = "";
    pinnedRow = null;
    clearPulse();
    if (!debugOn || issues.length === 0) {
      issuesPanel.style.display = "none";
      return;
    }
    issuesPanel.style.display = "flex";
    const errors = issues.filter(i => i.severity === "Error").length;
    const warns = issues.length - errors;
    countBadge.textContent = errors > 0
      ? `${errors} error${errors > 1 ? "s" : ""}`
      : `${warns} warning${warns > 1 ? "s" : ""}`;
    countBadge.style.color = errors > 0 ? "#f66" : "#fa0";
    countBadge.style.background = errors > 0 ? "rgba(255,68,68,0.12)" : "rgba(255,170,0,0.12)";

    for (const issue of issues) {
      const row = document.createElement("div") as HTMLDivElement;
      row.className = "issues-row";
      if (issue.x == null || issue.y == null) row.classList.add("faded");

      const dot = document.createElement("span");
      dot.className = "issues-dot";
      dot.style.background = issue.severity === "Error" ? "#f44" : "#fa0";
      row.appendChild(dot);

      const cat = document.createElement("span");
      cat.className = "issues-category";
      cat.style.color = issue.severity === "Error" ? "#f66" : "#fa0";
      cat.textContent = issue.category;
      row.appendChild(cat);

      const msg = document.createElement("span");
      msg.className = "issues-message";
      msg.textContent = issue.message;
      row.appendChild(msg);

      if (issue.x != null && issue.y != null) {
        row.classList.add("has-pos");
        const key = `${issue.x},${issue.y}`;
        const ix = issue.x;
        const iy = issue.y;
        row.addEventListener("click", (e) => {
          e.stopPropagation();
          if (pinnedRow === row) {
            unpinRow();
          } else {
            unpinRow();
            pinnedRow = row;
            row.style.background = "rgba(255,255,255,0.08)";
            // Smoothly animate the viewport centre to the issue tile.
            // pixi-viewport's snap plugin runs as part of the viewport
            // ticker, so requestRender handles redraws automatically.
            viewport.snap(ix * TILE_PX + TILE_PX / 2, iy * TILE_PX + TILE_PX / 2, {
              time: 450,
              ease: "easeInOutQuad",
              removeOnComplete: true,
              removeOnInterrupt: true,
              forceStart: true,
            });
            beginAnimating();
            // Stop animating once snap finishes; pixi-viewport doesn't
            // notify us, so we just set a timeout matching the snap.
            window.setTimeout(() => endAnimating(), 480);
            pulseCircle(key, ix, iy);
          }
        });
      }
      body.appendChild(row);
    }
  }

  return {
    populate,
    setVisible(visible: boolean): void {
      issuesPanel.style.display = visible ? "flex" : "none";
    },
    setCircleMap(map: Map<string, Graphics[]>): void {
      circleMap = map;
    },
    clearPulse,
    panel: issuesPanel,
    onValClose: null,
    setOnValClose(cb: () => void): void {
      onValCloseCb = cb;
    },
  };
}
