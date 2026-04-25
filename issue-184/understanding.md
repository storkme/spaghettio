# Issue #184: shift+drag doesn't let you start a selection if you're starting on an empty tile

## What the issue is asking
Shift+drag selection should work when starting the drag gesture on an empty tile (a tile with no entity). Currently it doesn't — the drag rectangle never appears.

## Root cause
Commit `034c259` (PR #162) added an early return in `onDown` that checks `if (!tileMap.has(`${tx},${ty}`)) return;`. This was intended to prevent shift+click on empty space from clearing selection, but it also prevented shift+drag from starting at all on empty tiles.

## Fix applied
1. Removed the early-return check in `onDown` — shift+drag now starts from any tile
2. In `onUp`, the shift+click branch now checks if the click is on an entity before clearing selection — empty-space clicks are pure navigation

## PR
https://github.com/storkme/fucktorio/pull/185
