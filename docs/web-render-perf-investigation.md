# Web app render performance investigation — 2026-04

Snapshot of findings from a Chrome DevTools trace captured while loading
`?item=processing-unit&rate=2&machine=assembling-machine-3&in=iron-plate,copper-plate,steel-plate,stone,coal,water,crude-oil,iron-ore,copper-ore`.
The trace covered ~5.5 s of wall-clock time on the dev server.

The investigation was triggered by a "things feel sluggish" report. The headline
finding is that **the engine work is not the problem** — it lands cleanly off the
main thread and finishes in tens of milliseconds. The sluggishness comes from
PixiJS rendering at full speed continuously, even when nothing in the scene is
changing. This doc captures the trace methodology, the steady-state numbers,
and the option space for future work, so the next person who picks up this
thread can skip the discovery phase.

## Summary of findings

| Where the time goes (5.46 s window) | ms | % |
|---|---|---|
| Idle | 2,047 | 38 |
| Pixi `_tick` → render loop (continuous, scene-static) | 2,574 | 47 |
| V8 program / module / GC overhead | ~500 | 9 |

Most of the rendering time is downstream of one root cause: **the Pixi ticker
runs every frame regardless of whether the scene changed.** With thousands of
`Graphics` children in the entity layer (each entity is its own Graphics
object), even an unchanged scene costs ~10 ms/frame to re-collect, re-batch,
and re-submit to WebGL.

After the initial render settles (~2 s in), CPU usage holds steady at
**~135 ms per 200 ms wall-clock, ~65–75% of one core, on a static scene**.

## Pipeline timing

Decomposing the trace by 200 ms bucket on the main thread:

| Time | Main-thread CPU | What's happening |
|---|---|---|
| 0–600 ms | ~165 ms | boot, Vite module loading |
| **600–1000 ms** | **~30 ms total** | **main thread idle while worker solves+lays out** |
| 1000–1200 ms | 104 ms | Pixi `app.init()` (54 ms blocking) + first stream events |
| 1200–1400 ms | 213 ms | initial layout render (one 145 ms rAF, see below) |
| 1400–2000 ms | ~270 ms | streaming commits + sidebar build |
| **2000–5400 ms** | **~135 ms / 200 ms** | **steady state — pure Pixi waste** |

Worker total CPU across the entire 5.4 s trace was **~55 ms**, with the longest
single task being a 5.6 ms WASM compile. The actual solve+layout for
`processing-unit @ rate=2` from raw inputs is sub-frame fast in WASM. The
`engine.worker.ts` handoff is correct; the user's mental model of "heavy stuff
runs in a worker" holds.

## The three notable single events

**The 143 ms initial-render frame** (around +1271 ms): one rAF spent ~91 ms in
`collectRenderables*` walking the scene graph and ~91 ms in
`_buildInstructions` rebuilding the GPU command list. That's first-render
fixed cost — Pixi has to traverse and batch every entity once. Not a steady-
state problem, but it does block input for ~9 frames at 60 Hz.

**A 54 ms `RunMicrotasks` at +1115 ms** turned out to be Pixi's `app.init()`:
12 ms `getExtensions`, 9.5 ms `initFromContext`, 9.2 ms `createContext`. WebGL
context creation is synchronous and not really our problem to fix, but it's
worth knowing about.

**A 45 ms `HandlePostMessage` at +1218 ms** was the streaming renderer
processing a `PhaseSnapshot/rows_placed` event from the worker:
`worker.onmessage` → `streamingRenderer.onEvent` → `handlePhaseSnapshot` →
`commitTransient` → `drawEntityGraphic`. With `BATCH_SIZE = 8` in
[`engine.worker.ts`](../web/src/workers/engine.worker.ts), each batch holds
the main thread for tens of ms. During the streaming phase, the user sees
~10 small stalls.

## How to reproduce the methodology

1. Open DevTools → Performance, click record.
2. Load a heavy URL (the `processing-unit @ rate=2` link above is good).
3. Stop after the layout settles + you've moved the mouse around a bit.
4. Save the trace as `Trace-*.json.gz`.
5. Decompress with `gunzip -c Trace-*.json.gz > trace.json`.
6. Use `jq` to extract:
   - All `traceEvents` and group by category.
   - `FireAnimationFrame` events with `dur > 0` to measure rAF cadence.
   - `RunTask` events on the main thread (`tid` matches `CrRendererMain`)
     bucketed by time to show CPU pressure.
7. To get a CPU profile, filter for `name == "ProfileChunk" and id == "0x1"`
   (where 0x1 is the main thread's profile id from the matching `Profile`
   event), concatenate the `nodes` arrays and reconstruct sample timestamps
   from `ts + cumsum(timeDeltas)`. Aggregate self-time and total time per
   node, walking parents to get inclusive cost.

The Node script template that worked well for this trace lives in the analysis
notes; the gist is:

```js
const samples = [];
for (const c of chunks) {
  let t = c.ts;
  for (let i = 0; i < c.samples.length; i++) {
    t += c.deltas[i] || 0;
    samples.push({ ts: t, nodeId: c.samples[i] });
  }
}
samples.sort((a, b) => a.ts - b.ts);
for (let i = 0; i < samples.length - 1; i++) samples[i].dur = samples[i+1].ts - samples[i].ts;
// then aggregate self-time per nodeId, walk parent chain for total time
```

## Pitfalls in the trace

- **18 dedicated worker threads** show up but only one (the
  `fucktorio-engine` worker) does meaningful work. The other 17 spin up and
  die in 1–5 ms each, all within a 25 ms burst at +484 ms. They look
  alarming but they're not from our code — most likely a Chrome extension's
  content scripts or Vite dev-server scaffolding. Confirm by checking
  `wasm_events` and `RunTask` totals per thread before chasing this.
- **rAF count looks doubled.** ~210 rAFs/s vs the expected 60–120 Hz refresh
  rate. The median rAF is 33 µs (no-op) and the heavy ones (5–12 ms) come at
  ~60/s, so there are two ticker loops scheduling: Pixi's app ticker plus
  pixi-viewport's deceleration plugin (which keeps requesting frames even
  when at rest). After render-on-demand lands, this number should drop to
  approximately zero outside of interactions.
- **`_setProgram` taking ~190 ms self-time** suggests batch breaks from
  shader switches (Graphics → Sprite → Graphics → ...). Worth investigating
  *if* render-on-demand alone doesn't restore the desired feel; otherwise
  it's premature.

## Option space for fixes

In rough order of impact-vs-effort:

### 1. Render-on-demand (~65% steady-state CPU → ~0%)

`web/src/renderer/app.ts`:

```ts
await app.init({ ..., autoStart: false, sharedTicker: false });
```

Then `app.render()` once after `renderLayout`, and on viewport `moved` /
`zoomed` / `decelerate` events. While interactions are active,
`app.ticker.start()`; when they settle, `app.ticker.stop()`. pixi-viewport
emits the events.

Per-feature tickers (`phaseAnimation`, `streamingRenderer`,
`improvementAnimation`, `issuesDialog` pulse) all currently rely on the
ticker being live; each one will need to either keep the ticker alive while
it has work or call `app.render()` explicitly after each tick.

This is a real architectural change — single PR, but with reach.

#### Implementation plan

Two abstractions in `web/src/renderer/app.ts`, exported both via the
`AppContext` and as module-level functions (so per-feature ticker users
can `import { beginAnimating, endAnimating } from "../renderer/app"`
without changing function signatures):

```ts
// Coalesced one-shot render. Multiple calls in the same task collapse
// into one. Use after one-shot scene mutations.
export function requestRender(): void;

// Refcounted ticker control. Use around sustained animations.
export function beginAnimating(): void;
export function endAnimating(): void;
```

Inside `createApp`, wire the viewport's events to keep the ticker open
during interactions:

| Event | Action |
|---|---|
| `moved`, `zoomed` | `requestRender()` |
| `drag-start`, `pinch-start`, `snap-start`, `snap-zoom-start`, `bounce-x-start`, `bounce-y-start` | `beginAnimating()` |
| `drag-end`, `pinch-end`, `snap-end`, `snap-zoom-end`, `bounce-x-end`, `bounce-y-end` | `endAnimating()` |
| Window `resize` | `requestRender()` |

Wheel emits `wheel-start` but no matching end; `moved`/`zoomed` cover
the resulting motion.

In `web/src/main.ts`, every code path that mutates the visible scene
needs a `requestRender()` afterwards. The choke points:

| Site | Why |
|---|---|
| End of `renderLayoutOnCanvas` (after overlay updates + `entityLayer.alpha` reset) | Initial layout commit |
| Each `renderLayout(...)` call site (5 total) | Re-render with new entities |
| End of `updateValidationOverlay`, `updateRegionOverlay`, `updateGhostTilesOverlay`, `updateTraceOverlay` (all return paths) | Overlay add/remove |
| `junctionDebugger.onChange` callback (sets `entityLayer.alpha`) | SAT-zone dim toggle |
| `inspector.onPinChange` (mutates `pinHighlight`) | Pin highlight |
| `soloRegionsCb.addEventListener("change", ...)` both branches | Solo-region dim |
| Wrap the `HighlightController` returned from every `renderLayout` so each method auto-`requestRender()`s | Hover dim path (called from `inspector.ts`) |

Per-feature ticker users — convert each to call `beginAnimating()` when
they `app.ticker.add(tick)` and `endAnimating()` when they
`app.ticker.remove(tick)`:

| File | Lifecycle notes |
|---|---|
| `web/src/renderer/streamingRenderer.ts` | tick added at start, removed in `cancel()` and `finish()`; track a local `tickerActive` boolean to avoid double-end |
| `web/src/renderer/phaseAnimation.ts` | tick added at start, removed when complete or via `cancel()` / `finish()`; **early-return when `scheduled.length === 0` must skip `app.ticker.add` entirely**, otherwise the begin has no matching end |
| `web/src/renderer/improvementAnimation.ts` | tick added on flash spawn, removed when faded out |
| `web/src/ui/issuesDialog.ts` (pulse) | `pulseCircle` adds, `clearPulse` removes |

Each removal path also needs a `requestRender()` so the final state is
painted (e.g. fade-out finishing `alpha = 0` for the flash, alpha back to
the resting value for pulse markers).

#### Failure mode and verification

The failure mode is "the canvas appears frozen until I move the mouse
or pan." That happens when a mutation occurs but no `requestRender()`
follows. Type-check, tests, and lint do not catch this. The only
reliable verification is exercising every interaction at the dev server.

Verification checklist for the PR:

- Initial load: layout renders, sidebar populates, all overlays settle.
- Pan / zoom / pinch: smooth motion during, no continued rendering
  after motion settles.
- Hover an entity: highlight appears immediately, dim applies to
  others, clears immediately when mouse leaves.
- Toggle each overlay (validation, regions, ghost tiles): each appears
  and disappears immediately.
- "Solo regions" mode: dim applies on enable, restores on disable.
- Pin a tile via the inspector: highlight ring renders.
- SAT zone selection: dim applies, edit mode dims further;
  deselection restores both.
- Streaming layout (load a fresh URL): transient previews + ghost
  routes appear progressively, settle into final layout.
- Phase animation (corpus / parsed blueprint): entities fade in
  staggered.
- Auto-optimize: region flashes during the optimize phase.
- Validation pulse: clicking an issue in the panel pulses its marker.
- Timeline scrub: drag the scrubber backwards/forwards through phases.

**Anti-test: leave the page idle and watch DevTools Performance.** Main
thread CPU should drop to near zero. If it doesn't, something is
calling `beginAnimating()` without a matching `endAnimating()`.

### 2. Mark static layers as render groups (already landed)

```ts
entityLayer.isRenderGroup = true;
```

Pixi v8 caches the GPU instruction buffer for descendants. Transform / alpha
mutations on children still render correctly without invalidating the cache.
Adding / removing children (e.g. `renderLayout`'s `removeChildren()` +
re-add) does invalidate, so this only helps in steady state, not during
layout commit. Already merged as part of this investigation.

### 3. Yield while building the scene

`renderLayout` builds all entity Graphics in one synchronous pass
(`entities.ts:980`). For the 143 ms first-render hit, chunk this with
`await scheduler.yield()` so the work spreads across 5–10 frames. The user
sees layout fade in instead of a single hard pause. Smaller change than (1),
but the win is "fewer dropped frames during commit," not "feels snappier
later."

### 4. OffscreenCanvas — move all of Pixi to a worker

The big lift. Pixi v8 supports it. Rendering, scene graph, batching, GL state
all move off-main-thread. The main thread keeps input handling, DOM updates
(sidebar/inspector), and forwards events to the render worker. Eliminates the
steady-state main-thread cost completely.

Browser support is broad (Chrome, Edge, Firefox 105+, Safari 16.4+). Cost is
substantial: `app.ts` and the entire `renderer/` directory move into a new
worker; events have to be serialized across the boundary; the inspector +
sidebar code that reads scene state needs a request/response bridge. Not
worth attempting until (1) is in place and shown insufficient.

### 5. Pre-built geometry in the engine worker

For each entity, generate vertex data (positions, colours, UVs) in the
engine worker and `postMessage` with `transfer:[buffer]` for zero-copy. Main
thread feeds it into a single `Mesh` per visual class instead of one
`Graphics` per entity. Drops the per-entity allocation cost during commit
and shrinks the scene graph from "thousands of children" to "a few meshes."

Lossy: per-entity alpha for hover would need uniforms or a separate dim
overlay shape. Probably worth it once (1) is shipping; not a near-term move.

## Smaller wins worth knowing about

- **Missing-icon warns** (already landed): `Assets.get` warns when the key
  isn't cached, and each warn captures a stack trace. We saw 13 warns during
  initial render. Fixed by checking `Cache.has(key)` before `Assets.get`.
  See `tryGetTexture()` in `entities.ts`.
- **Per-machine `new TextStyle(...)` allocations** in `entities.ts:760, 784`
  rebuild text style state for every machine. Hoist to module scope as a
  constant or memoise per `(fontSize, fill)` tuple. Modest win.
- **`improveRegionStreaming` uses `BATCH_SIZE = 1`** in
  `engine.worker.ts:178`. Fine while improvements come in slowly, but if
  optimisation ever speeds up this becomes the bottleneck.

## Ground rules for future trace analysis

- Decompress traces somewhere outside the repo (e.g. `/tmp/perf-trace/`)
  — uncompressed traces are tens of MB and we don't want them tracked.
- Always check whether the user's selected breadcrumb covers the whole trace
  or just a window — `metadata.modifications.initialBreadcrumb.window` has
  the µs range. Quoting numbers across the wrong window is the #1 way to
  mislead yourself.
- The CPU profile lives in `ProfileChunk` events keyed by `tid` + `id`. The
  `tid` on the chunks themselves is the *profiler* thread, not the sampled
  thread; match against the `tid` of the parent `Profile` event to find the
  main thread.
- Don't trust "X ms in `executeInstructions`" alone. Always also check
  whether the scene graph mutated (look at `removeChildren` / `addChild`
  in self-time) — re-rendering a static scene has different fixes from
  re-building one.
