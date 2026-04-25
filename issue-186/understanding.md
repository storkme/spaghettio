# Issue #186: Investigate how 'timeline view' in UI works

## What the issue is asking

The issue author (storkme) is asking to investigate the timeline view in the web UI. Specifically:

1. **Label clarity** — milestone labels don't clearly match up with what events they represent
2. **Purpose clarity** — it's not clear what the timeline is doing / showing
3. **SAT improver gap** — after adding the SAT improver (auto-optimize), the timeline needs to be adapted to show that phase too
4. **Progress indication** — during solving/layout, there should be a progress bar showing that solving is still happening

## What exists today

### Timeline Scrubber (`web/src/ui/timelineScrubber.ts`)
A floating bar above the canvas with two modes:

**Live mode** (during streaming layout):
- 5 milestone chips: "Machines", "Ghost Routes", "Committed Routes", "Junctions", "Poles"
- Progress fill grows evenly across milestones (not time-weighted)
- Non-interactive
- Chips shown evenly spaced

**Scrub mode** (after streaming completes):
- Draggable seekbar with thumb
- Chips repositioned to true relative timestamps
- Snap-to-milestone behavior
- Used for stepping through the layout reveal

### Milestone mapping (streamingRenderer.ts)
| Milestone ID | Trace event that triggers it | What actually happens |
|---|---|---|
| machines | PhaseSnapshot{phase:"rows_placed"} | Machines/belts/inserters placed |
| ghost_routes | GhostSpecRouted | Ghost belts drawn on canvas |
| committed_routes | GhostSpecCommitted | Ghost belts swap to real entities |
| junctions | JunctionCommitted | SAT junction zones solved |
| poles | PhaseSnapshot{phase:"poles_placed"} | Power poles placed |

### Busy overlay (`web/src/ui/busyOverlay.ts`)
A spinner + "computing…" label in the top-right corner. Uses `onEngineActivity` to show/hide. Has a 120ms grace period to avoid flicker on fast layouts. **Notably: this module exists but is NOT imported or used in main.ts** — it's dead code.

### Auto-optimize (`web/src/main.ts` → `runAutoOptimize`)
After layout finishes, if there are SAT crossing zones, an automatic improvement pass runs. It streams `SatImprovement` events, queues them, and drains with rAF pacing. No timeline/progress indicator exists for this phase — it just morphs the layout silently.

### Streaming renderer (`web/src/renderer/streamingRenderer.ts`)
Handles live trace events during layout. Draws transient ghost belts, committed previews, and SAT cluster outline pulses. Records entity reveal timestamps for scrub mode.

### Trace overlay (`web/src/renderer/traceOverlay.ts`)
Post-layout overlay showing lanes, rows, balancers, tap-offs, route failures, ghost paths, cluster zones. Driven by `debug` checkbox in sidebar.

## Key findings

1. **Dead code**: `busyOverlay.ts` is not wired up — the spinner never appears
2. **Labels are phase-names, not event-descriptions**: "Machines" = rows placed, "Ghost Routes" = ghost belts drawn, etc. These are somewhat opaque to a user
3. **No SAT improver timeline**: The auto-optimize phase has zero visual progress indication. The layout just morphs silently with flash effects
4. **Progress fill is uniform, not time-weighted**: Each milestone gets equal progress weight (20% each), so a 5-second junction solve at milestone 4 gives the same visual progress as a 50ms one
5. **Timeline disappears immediately after streaming**: Once `finish()` is called, the scrubber arms but the user doesn't know optimization is happening
