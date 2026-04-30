// External-input trunk labels — issue #196 first phase.
//
// Renders a vertical, bottom-to-top label above each contiguous group of
// external-input trunk columns at the top of the layout. Each label is
// `[icon] {rate}/s {item name}` with the item name rendered at lower
// alpha. Font size scales with the group's tile span so wider groups read
// big and bold while single-column trunks stay legible without dominating.
//
// Trunks are detected by walking the topmost row of belt/pipe entities
// whose `carries` matches one of the SolverResult's external_inputs
// items. Groups merge contiguous columns sharing the same carries value.
// Output trunks are explicitly out of scope for this phase.

import { Container, Graphics, Sprite, Text, TextStyle } from "pixi.js";
import type { LayoutResult, PlacedEntity, SolverResult } from "../engine";
import { TILE_PX, BELT_ENTITIES, UG_BELT_ENTITIES, PIPE_ENTITIES, niceName } from "./entities";
import { getItemIconTexture } from "./atlas";

// How many tiles of vertical headroom to reserve above row 0 for labels.
// Labels rise upward from y=0 by at most this many tiles (in world space).
const LABEL_HEADROOM_TILES = 6;

// Padding (in tile units) between the icon, rate, and name segments. Small
// because the whole thing rotates so this becomes vertical spacing.
const SEGMENT_GAP_PX = 6;

// Font-size scaling. Labels grow gently with trunk width so a 5-column
// family reads slightly bigger than a single trunk, but stays in the same
// visual register — wide spread (e.g. 14→48) made the layout confusing.
const FONT_BASE_PX = 18;
const FONT_PER_EXTRA_TILE_PX = 2;
const FONT_MAX_PX = 26;

// Label rotation. Originally -π/2 (straight up) which made multi-group
// layouts hard to scan. -π/3 (-60°) reads like slanted text climbing
// up-and-to-the-right while still keeping labels mostly in their column's
// vertical headroom.
const LABEL_ROTATION = -Math.PI / 3;

// Per-group divider bar drawn flush above the trunk top. Makes the column
// span of each label visually obvious so adjacent groups don't blur
// together.
const DIVIDER_THICKNESS_PX = 2;
const DIVIDER_COLOR = 0xffffff;
const DIVIDER_ALPHA = 0.55;
// Inset from each end of the group span so neighbours don't appear to share
// a single bar — half a tile of horizontal padding feels right at TILE_PX=32.
const DIVIDER_INSET_PX = 4;

// External input trunks always have `source_y = 0` (lane_planner.rs:198),
// so their topmost trunk tile sits at the top of the layout. Anything with
// a higher topY is an intermediate column — drop it.
const INPUT_TRUNK_TOP_Y = 0;

const NAME_ALPHA = 0.6;

interface TrunkGroup {
  /** External-input slug (e.g. `"iron-plate"`, `"water"`). */
  item: string;
  /** Whether this trunk carries fluid (pipes) vs items (belts). */
  isFluid: boolean;
  /** Inclusive x range of trunk columns. */
  xMin: number;
  xMax: number;
  /** Topmost y of the trunk's first tile (used to anchor the label). */
  topY: number;
  /** Total throughput from solver's external_inputs entry. */
  rate: number;
}

function isInputTrunkEntity(e: PlacedEntity): boolean {
  if (!e.carries) return false;
  return (
    BELT_ENTITIES.has(e.name) ||
    UG_BELT_ENTITIES.has(e.name) ||
    PIPE_ENTITIES.has(e.name)
  );
}

/**
 * Find external-input trunk groups: scan every column for the topmost
 * tile carrying an external-input item, then merge contiguous columns
 * with matching carries into single groups.
 */
function findInputTrunks(
  layout: LayoutResult,
  solver: SolverResult,
): TrunkGroup[] {
  // Map external-input slug → solver-provided rate. Only items that show
  // up here qualify as "external input" trunks; intermediates carrying
  // the same item further down the bus are filtered out.
  const flowByItem = new Map<string, { rate: number; isFluid: boolean }>();
  for (const flow of solver.external_inputs) {
    flowByItem.set(flow.item, { rate: flow.rate, isFluid: !!flow.is_fluid });
  }
  if (flowByItem.size === 0) return [];

  // For each x column, the topmost trunk-bearing tile that carries an
  // external-input item. Lower y = higher up.
  const topByCol = new Map<number, { y: number; carries: string }>();
  for (const e of layout.entities) {
    if (!isInputTrunkEntity(e)) continue;
    const carries = e.carries!;
    if (!flowByItem.has(carries)) continue;
    const x = e.x ?? 0;
    const y = e.y ?? 0;
    const cur = topByCol.get(x);
    if (!cur || y < cur.y) {
      topByCol.set(x, { y, carries });
    }
  }
  if (topByCol.size === 0) return [];

  // Sort columns left to right and merge contiguous runs sharing carries.
  // Drop any column whose topmost trunk tile is below the layout's top row:
  // intermediate trunks that happen to carry an external-input item show up
  // here too, but they don't deserve an "input" label.
  const cols = Array.from(topByCol.entries())
    .filter(([, info]) => info.y === INPUT_TRUNK_TOP_Y)
    .map(([x, info]) => ({ x, ...info }))
    .sort((a, b) => a.x - b.x);

  const groups: TrunkGroup[] = [];
  let cur: TrunkGroup | null = null;
  for (const c of cols) {
    const flow = flowByItem.get(c.carries)!;
    if (
      cur &&
      cur.item === c.carries &&
      c.x === cur.xMax + 1
    ) {
      cur.xMax = c.x;
      cur.topY = Math.min(cur.topY, c.y);
    } else {
      if (cur) groups.push(cur);
      cur = {
        item: c.carries,
        isFluid: flow.isFluid,
        xMin: c.x,
        xMax: c.x,
        topY: c.y,
        rate: flow.rate,
      };
    }
  }
  if (cur) groups.push(cur);

  // If the same item gets split across N non-contiguous groups, the
  // solver's flow rate is the *total* delivered, which we'd otherwise
  // double-count by attaching it to each group. Divide it across groups.
  const groupCountByItem = new Map<string, number>();
  for (const g of groups) {
    groupCountByItem.set(g.item, (groupCountByItem.get(g.item) ?? 0) + 1);
  }
  for (const g of groups) {
    const n = groupCountByItem.get(g.item) ?? 1;
    if (n > 1) g.rate = g.rate / n;
  }

  return groups;
}

function formatRate(rate: number): string {
  // Match sidebar format: one decimal place, /s suffix.
  return `${rate.toFixed(1)}/s`;
}

/**
 * Build a single rotated label container for one trunk group. The
 * container is positioned so that, after `LABEL_ROTATION` is applied, it
 * reads from near the trunk's top edge climbing into the headroom above
 * the layout.
 *
 * Layout (pre-rotation, in local coords running left to right):
 *
 *   [icon][gap][rate ][gap][item name (dimmer)]
 *
 * Rotated by -60°, "left to right" becomes "lower-left to upper-right",
 * so the icon hugs the trunk and the rate + name lean diagonally up.
 */
function buildLabel(group: TrunkGroup): Container {
  const span = group.xMax - group.xMin + 1;
  const fontSize = Math.min(
    FONT_MAX_PX,
    FONT_BASE_PX + (span - 1) * FONT_PER_EXTRA_TILE_PX,
  );

  const labelContainer = new Container();
  labelContainer.eventMode = "none";

  // Icon — sized to roughly match the cap height of the text. Icons in
  // the atlas are square; scale them to `fontSize` px on a side.
  const iconTex = getItemIconTexture(group.item);
  const iconSprite = new Sprite(iconTex);
  iconSprite.width = fontSize;
  iconSprite.height = fontSize;
  iconSprite.x = 0;
  iconSprite.y = -fontSize / 2; // vertically centre on the local x-axis
  labelContainer.addChild(iconSprite);

  // Rate — bold, full opacity.
  const rateStyle = new TextStyle({
    fontFamily: "'JetBrains Mono','Consolas',monospace",
    fontSize,
    fontWeight: "bold",
    fill: 0xffffff,
    dropShadow: { color: 0x000000, distance: 1, blur: 3, alpha: 0.85 },
  });
  const rateText = new Text({ text: formatRate(group.rate), style: rateStyle });
  rateText.x = fontSize + SEGMENT_GAP_PX;
  rateText.y = -rateText.height / 2;
  labelContainer.addChild(rateText);

  // Item name — bold but lower opacity.
  const nameStyle = new TextStyle({
    fontFamily: "'JetBrains Mono','Consolas',monospace",
    fontSize,
    fontWeight: "bold",
    fill: 0xffffff,
    dropShadow: { color: 0x000000, distance: 1, blur: 3, alpha: 0.85 },
  });
  const nameText = new Text({ text: niceName(group.item), style: nameStyle });
  nameText.alpha = NAME_ALPHA;
  nameText.x = rateText.x + rateText.width + SEGMENT_GAP_PX;
  nameText.y = -nameText.height / 2;
  labelContainer.addChild(nameText);

  return labelContainer;
}

/**
 * Render external-input trunk labels into `layer`. Clears any previous
 * children. Safe to call repeatedly — designed to be invoked at the end
 * of every layout commit.
 */
export function renderInputLabels(
  layer: Container,
  layout: LayoutResult,
  solver: SolverResult | null,
): void {
  layer.removeChildren();
  if (!solver) return;

  const groups = findInputTrunks(layout, solver);
  if (groups.length === 0) return;

  for (const group of groups) {
    // Divider bar sits flush above the trunk's top tile, spanning just the
    // group's columns so adjacent groups read as visually distinct.
    const dividerY = group.topY * TILE_PX - DIVIDER_THICKNESS_PX;
    const dividerX = group.xMin * TILE_PX + DIVIDER_INSET_PX;
    const dividerW = (group.xMax - group.xMin + 1) * TILE_PX - 2 * DIVIDER_INSET_PX;
    if (dividerW > 0) {
      const divider = new Graphics();
      divider
        .rect(dividerX, dividerY, dividerW, DIVIDER_THICKNESS_PX)
        .fill({ color: DIVIDER_COLOR, alpha: DIVIDER_ALPHA });
      layer.addChild(divider);
    }

    const label = buildLabel(group);
    label.rotation = LABEL_ROTATION;

    // Anchor in world (pixel) space. The trunk's top tile sits at
    // (group.xMin .. group.xMax, group.topY). We want the label centred
    // horizontally over that tile range and standing in the headroom
    // above row 0 of the layout (i.e. negative world y).
    const groupCenterX =
      ((group.xMin + group.xMax + 1) / 2) * TILE_PX;
    // Place the rotated baseline ~half a tile above the trunk's top
    // tile. After rotation the label's positive local-x climbs up and
    // (with a non-90° tilt) slightly to the right, so positioning at
    // (centerX, topY - 0.5) makes the icon sit just above the trunk and
    // the rate + name rise into the negative-y headroom.
    const baselineY = group.topY * TILE_PX - TILE_PX * 0.5;

    label.x = groupCenterX;
    label.y = baselineY;

    layer.addChild(label);
  }

  // Reserve room above the layout: callers can use the headroom estimate
  // when fitting the viewport. We don't enforce it here — pan/zoom will
  // happily reveal labels that extend past the fitted view, and that's
  // fine for now.
  void LABEL_HEADROOM_TILES;
}
