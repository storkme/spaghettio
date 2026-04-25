import { Application, UPDATE_PRIORITY } from "pixi.js";
import { Viewport } from "pixi-viewport";

export const WORLD_SIZE = 3200;

export interface AppContext {
  app: Application;
  viewport: Viewport;
  /**
   * Schedule a single render on the next microtask. Multiple calls within
   * the same task coalesce into one render. Use for one-shot scene mutations
   * (renderLayout commits, hover dim changes, overlay toggles).
   */
  requestRender: () => void;
  /**
   * Mark the start of a sustained animation that wants the ticker running
   * every frame. Pairs with `endAnimating()`. Internal counter — overlapping
   * animations are safe as long as each begin matches exactly one end.
   * Use for ticker-driven animations (per-feature `app.ticker.add(tick)`
   * call sites, viewport drag/pinch/decelerate inertia).
   */
  beginAnimating: () => void;
  /** Pair with `beginAnimating()`. Stops the ticker when the counter hits 0. */
  endAnimating: () => void;
}

// --- Module-scoped re-exports ---
//
// Per-feature ticker users (streamingRenderer, phaseAnimation, improvementAnimation,
// issuesDialog) take `app: Application` rather than the full AppContext, but they
// still need to start/stop the ticker as part of their lifecycle. Rather than
// thread the controls through every function signature, we expose them as
// module-scoped functions populated by `createApp`. They are no-ops if invoked
// before `createApp` has run — which shouldn't happen in practice but means a
// stray import won't blow up at module load time.

let _requestRender: (() => void) | null = null;
let _beginAnimating: (() => void) | null = null;
let _endAnimating: (() => void) | null = null;

export function requestRender(): void {
  _requestRender?.();
}

export function beginAnimating(): void {
  _beginAnimating?.();
}

export function endAnimating(): void {
  _endAnimating?.();
}

export async function createApp(container: HTMLElement): Promise<AppContext> {
  const app = new Application();
  // `autoStart: false` stops Pixi from rendering on every rAF. We drive
  // renders explicitly via `requestRender()` for one-shot mutations and via
  // `beginAnimating()` / `endAnimating()` for sustained animations. See
  // `docs/web-render-perf-investigation.md` for the trace evidence.
  await app.init({
    resizeTo: container,
    background: 0x1e1e1e,
    antialias: true,
    autoStart: false,
    sharedTicker: false,
  });

  // `autoStart: true` would have wired this for us; we have to do it
  // ourselves. Without it, the ticker runs custom ticks (alpha animations,
  // viewport plugin updates) but never paints — so during streaming layout
  // commit and per-feature animations the scene mutates invisibly. LOW
  // priority puts the render after every other tick, so each tick's state
  // changes show up in the same frame.
  app.ticker.add(() => app.render(), null, UPDATE_PRIORITY.LOW);

  container.appendChild(app.canvas);

  app.canvas.addEventListener("contextmenu", (e) => e.preventDefault());

  const viewport = new Viewport({
    screenWidth: container.clientWidth,
    screenHeight: container.clientHeight,
    worldWidth: WORLD_SIZE,
    worldHeight: WORLD_SIZE,
    events: app.renderer.events,
  });

  viewport.drag({ mouseButtons: "left" }).pinch().wheel().decelerate();

  app.stage.addChild(viewport);

  // --- Render-on-demand controls ---

  let pendingRender = false;
  let activeAnimations = 0;
  const requestRenderImpl = (): void => {
    // If the ticker is running, the auto-render hook above will paint this
    // frame already — scheduling a microtask render on top of that just
    // doubles the cost. During pan/drag/animations we get pointermove +
    // viewport.on("moved") events firing at ~60Hz; without this guard, each
    // one queues an extra full app.render() (≈30ms on a 3000-entity layout)
    // on top of the ticker's own per-frame render.
    if (activeAnimations > 0) return;
    if (pendingRender) return;
    pendingRender = true;
    queueMicrotask(() => {
      pendingRender = false;
      app.render();
    });
  };

  const beginAnimatingImpl = (): void => {
    if (activeAnimations === 0) app.ticker.start();
    activeAnimations++;
  };
  const endAnimatingImpl = (): void => {
    if (activeAnimations === 0) {
      // Defensive: shouldn't happen if pairs are balanced, but if a *-end
      // event fires without a matching *-start (e.g. plugin lifecycle
      // edge case) we don't want to underflow.
      return;
    }
    activeAnimations--;
    if (activeAnimations === 0) app.ticker.stop();
  };

  // Wire the module-scoped re-exports. After this returns, the standalone
  // `requestRender` / `beginAnimating` / `endAnimating` exports become live.
  _requestRender = requestRenderImpl;
  _beginAnimating = beginAnimatingImpl;
  _endAnimating = endAnimatingImpl;

  // --- Viewport interaction wiring ---
  //
  // `moved` / `zoomed` fire after every position/zoom change (during drag,
  // pinch, decelerate, snap). Coalesced by the requestRender microtask.
  // For sustained motion that needs the viewport's plugins to update each
  // frame, we hold the ticker open via *-start / *-end event pairs.
  viewport.on("moved", requestRenderImpl);
  viewport.on("zoomed", requestRenderImpl);

  const interactionStarts = [
    "drag-start",
    "pinch-start",
    "snap-start",
    "snap-zoom-start",
    "bounce-x-start",
    "bounce-y-start",
  ] as const;
  const interactionEnds = [
    "drag-end",
    "pinch-end",
    "snap-end",
    "snap-zoom-end",
    "bounce-x-end",
    "bounce-y-end",
  ] as const;
  for (const e of interactionStarts) {
    (viewport as unknown as { on: (e: string, fn: () => void) => void }).on(e, beginAnimatingImpl);
  }
  for (const e of interactionEnds) {
    (viewport as unknown as { on: (e: string, fn: () => void) => void }).on(e, endAnimatingImpl);
  }
  // Wheel emits `wheel-start` but no matching end (it's a one-shot per
  // scroll). `moved`/`zoomed` already trigger requestRender, so no extra
  // wiring needed for plain wheel zoom. Decelerate-after-drag is handled
  // by drag-end → moved-end via the `bounce`/`snap` chain.

  window.addEventListener("resize", () => {
    viewport.resize(container.clientWidth, container.clientHeight, WORLD_SIZE, WORLD_SIZE);
    requestRenderImpl();
  });

  // Render once after init so the canvas isn't blank until something else
  // triggers a render.
  requestRenderImpl();

  return {
    app,
    viewport,
    requestRender: requestRenderImpl,
    beginAnimating: beginAnimatingImpl,
    endAnimating: endAnimatingImpl,
  };
}
