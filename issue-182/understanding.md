# Issue #182: question about the junction/region interface

## What the issue is asking
User noticed "weird red and green circles on the junction interface" that appear to indicate inputs/outputs, and finds them "unhelpful." Follow-up comment asked to delete them.

## Resolution
Code change implemented: removed all boundary port markers from `regionOverlay.ts` (circles, arrows, dashed connectors, + 5 helper functions = 162 lines removed).

## PR
https://github.com/storkme/fucktorio/pull/183 — "fix(web): remove boundary port markers from SAT zone overlay"

## Code location
- `web/src/renderer/regionOverlay.ts` — entire port rendering section removed
- Port identity (item + IO) still available via hover in the tile inspector
