# Issue #182: question about the junction/region interface

## What the issue is asking
User noticed "weird red and green circles on the junction interface" that appear to indicate inputs/outputs, and finds them "unhelpful." Follow-up comment asked to delete them.

## Resolution
Code change implemented: removed all boundary port circles from `regionOverlay.ts` (circles removed, arrows kept at 35% alpha, dashed connectors kept).

## PR
https://github.com/storkme/fucktorio/pull/234 — "fix(web): remove boundary port markers from SAT zone overlay (#182)"

## Code location
- `web/src/renderer/regionOverlay.ts` — entire port rendering section: circles removed, arrows at 35% alpha, dashed connectors kept
- Port identity (item + IO) still available via hover in the tile inspector
