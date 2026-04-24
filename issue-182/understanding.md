# Issue #182: question about the junction/region interface

## What the issue is asking
User noticed "weird red and green circles on the junction interface" that appear to indicate inputs/outputs, and finds them "unhelpful."

## Code location
- `web/src/renderer/regionOverlay.ts` lines 18-19: `INPUT_COLOR = 0x50c050` (green), `OUTPUT_COLOR = 0xd04040` (red)
- Drawn at each `RegionPort` position in `renderRegionOverlayDetailed()` around line 200
- Toggle via Debug → "SAT Zones" checkbox in overlay panel
- `crates/core/src/models.rs` `RegionPort` struct: `io: PortIo` (Input/Output), `item: Option<String>`, `point: PortPoint`

## Answer given
Explained that green = input ports, red = output ports, with directional arrows and dashed item-matching lines. These are debugging aids for the junction/SAT pipeline. Text labels were removed because of overlap. Port identity available via hover in inspector.

## Task type
Question/discussion — answered with a comment. No code changes needed.
