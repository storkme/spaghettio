---
name: url
description: Generate a fucktorio layout URL for localhost, the live GitHub Pages site, or a PR preview sub-path. Use when the user asks for a link to a specific recipe/rate/config, or says "/url". Accepts shorthand like "PU@3/s ore red P2" and expands it to a full URL.
allowed-tools: Read, Bash
---

# url — generate a fucktorio layout URL

The user asks for a URL to a specific layout configuration, e.g.:
- `/url PU@3/s ore red P2`
- `/url advanced-circuit 5/s plates yellow`
- `/url processing-unit 2/s am3 fast partitioned`
- `/url electronic-circuit 30/s from ore — live`
- `/url processing-unit 3/s ore red P2 pr-261`

Output **one URL** (or multiple if the user asked for several targets).

## Step 1 — parse the request

Extract these parameters from the user's freeform text. All have defaults.

### item
Map common shorthands:
- `PU` / `processing unit` → `processing-unit`
- `AC` / `advanced circuit` → `advanced-circuit`
- `EC` / `electronic circuit` → `electronic-circuit`
- `IGW` / `gear` / `iron gear` → `iron-gear-wheel`
- `plastic` → `plastic-bar`
- Full hyphenated names pass through unchanged.

Default: `processing-unit`

### rate
Parse `N/s` or bare number. Examples: `3/s` → `3`, `0.5/s` → `0.5`.
Default: `3`

### machine (`machine` URL param)
Map shorthands:
- `am1` / `AM1` → `assembling-machine-1`
- `am2` / `AM2` → `assembling-machine-2`
- `am3` / `AM3` / not specified → `assembling-machine-3`

Default: `assembling-machine-3`

### belt (`belt` URL param)
Map shorthands:
- `yellow` / `slow` / `standard` → `transport-belt`
- `red` / `fast` → `fast-transport-belt`
- `blue` / `express` → `express-transport-belt`
- Not specified → omit the `belt` param (auto-select)

### inputs (`in` URL param)
Map preset names to comma-separated item lists:

| Shorthand | Expanded `in=` value |
|-----------|----------------------|
| `ore red` / `from ore` / `ores` (red belt default) | `iron-ore,copper-ore,coal,water,crude-oil` |
| `ore yellow` / `ores yellow` | `iron-ore,copper-ore,coal,water,crude-oil` |
| `plates yellow` / `from plates` | `iron-plate,copper-plate,steel-plate,stone,coal,water,crude-oil` |
| `plates red` | `iron-plate,copper-plate,steel-plate,stone,coal,water,crude-oil` |
| No input preset given | use the scoreboard default for the item (see below) |

Default input sets per item when no preset is given:
- `processing-unit` → `iron-ore,copper-ore,coal,water,crude-oil`
- `advanced-circuit` → `iron-plate,copper-plate,coal,crude-oil,water`
- `electronic-circuit` → `iron-ore,copper-ore,coal,water,crude-oil`
- Other → `iron-plate,copper-plate,coal,water,crude-oil`

If the user explicitly lists items (e.g. `in=iron-plate,copper-plate`), use them verbatim.

### strategy (`strategy` URL param)
- `P2` / `partitioned` / `decomposed` → `partitioned-decomposed`
- Not specified / `pool` / `pooled` → omit the `strategy` param

### row_layout (`row_layout` URL param)
- `HS` / `horizontal` / `horizontal-stack` → `horizontal-stack`
- Not specified → omit the `row_layout` param

### target (which base URL to use)
- `localhost` / `local` / not specified → `http://localhost:5173/`
- `live` / `prod` / `github` → `https://storkme.github.io/fucktorio/`
- `pr-N` / `pr N` / `PR #N` → `https://storkme.github.io/fucktorio/pr-N/`

## Step 2 — build the URL

Construct `<base>?<params>` where params are added in this order (omit when value is the default or not set):

1. `item=<item>` — always include
2. `rate=<rate>` — always include
3. `machine=<machine>` — omit if `assembling-machine-3` (that's the UI default)
4. `belt=<belt>` — omit if not specified
5. `in=<inputs>` — always include
6. `strategy=<strategy>` — omit if not specified
7. `row_layout=<row_layout>` — omit if not specified

## Step 3 — output

Print the URL(s) as plain markdown links. If the user asked for multiple
targets (e.g. "local and live"), print both. No prose beyond the links
unless the user's request was ambiguous — then ask one clarifying question.

## Examples

`/url PU@3/s ore red P2`
→ `http://localhost:5173/?item=processing-unit&rate=3&belt=fast-transport-belt&in=iron-ore,copper-ore,coal,water,crude-oil&strategy=partitioned-decomposed`

`/url AC 5/s plates yellow`
→ `http://localhost:5173/?item=advanced-circuit&rate=5&machine=assembling-machine-2&belt=transport-belt&in=iron-plate,copper-plate,steel-plate,stone,coal,water,crude-oil`

`/url EC 30/s from ore — live`
→ `https://storkme.github.io/fucktorio/?item=electronic-circuit&rate=30&in=iron-ore,copper-ore,coal,water,crude-oil`

`/url PU@2/s am3 fast ore red P2 pr-261`
→ `https://storkme.github.io/fucktorio/pr-261/?item=processing-unit&rate=2&belt=fast-transport-belt&in=iron-ore,copper-ore,coal,water,crude-oil&strategy=partitioned-decomposed`

`/url PU@2/s ore red HS`
→ `http://localhost:5173/?item=processing-unit&rate=2&belt=fast-transport-belt&in=iron-ore,copper-ore,coal,water,crude-oil&row_layout=horizontal-stack`

## Reference: URL param names from `web/src/state.ts`

- `item` — recipe name
- `rate` — target rate (float)
- `machine` — machine entity name
- `belt` — max belt tier entity name (`transport-belt` / `fast-transport-belt` / `express-transport-belt`)
- `in` — comma-separated input items
- `strategy` — layout strategy (`partitioned-decomposed`)
- `row_layout` — row layout variant (`horizontal-stack`)
- `ci` — custom inputs beyond DEFAULT_INPUTS (rarely needed)

Live site: `https://storkme.github.io/fucktorio/`
PR previews: `https://storkme.github.io/fucktorio/pr-<N>/`
Dev server: `http://localhost:5173/`
