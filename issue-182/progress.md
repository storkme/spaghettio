# Issue #182 progress

## Completed
1. Investigated the code (regionOverlay.ts, regionClassify.ts, models.rs)
2. Posted explanatory comment on the issue
3. User asked to delete the markers (comment: "can we raise a PR to delete them?")
4. Initial PR #183 removed ALL port markers (circles, arrows, dashed connectors)
5. User refined: "just delete the circles, keep everything else, make arrows transparent"
6. Updated PR #183: circles removed, arrows kept at 35% alpha, dashed connectors kept
7. PR created: https://github.com/storkme/fucktorio/pull/234
8. Label 'agent-done' added to issue

## PR
https://github.com/storkme/fucktorio/pull/234 — "fix(web): remove boundary port markers from SAT zone overlay (#182)"

## Code location
- `web/src/renderer/regionOverlay.ts` — circles removed, arrows at 35% alpha, dashed connectors kept
- Port identity (item + IO) still available via hover in the tile inspector
