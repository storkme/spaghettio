# Issue #186 Progress

## Completed

- Investigated the full timeline view implementation:
  - `web/src/ui/timelineScrubber.ts` — the scrubber component (live + scrub modes)
  - `web/src/renderer/streamingRenderer.ts` — milestone firing + time tracking
  - `web/src/renderer/traceOverlay.ts` — post-layout trace overlays
  - `web/src/ui/busyOverlay.ts` — dead code spinner
  - `web/src/main.ts` — integration point
  - `crates/core/src/trace.rs` — Rust trace event definitions
- Identified 4 concrete action items
- Filed 4 sub-issues:
  - #187: wire up busyOverlay
  - #188: time-weighted progress fill
  - #189: SAT auto-optimize timeline
  - #190: better milestone labels
- Added 'agent-done' label to #186
- Commented on #186 with findings

## Key findings summary

1. Timeline scrubber has two modes (live/scrub) driven by milestone events from streaming renderer
2. Busy overlay module exists but is never imported — dead code
3. Auto-optimize phase has zero progress indication
4. Progress fill is uniform (20% per milestone) not time-weighted
5. Labels are phase-names rather than user-friendly descriptions
