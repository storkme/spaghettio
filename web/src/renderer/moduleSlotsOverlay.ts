import { Graphics, Container } from "pixi.js";
import { TILE_PX, MACHINE_SIZES, QUALITY_BADGE_COLORS } from "./entities";
import type { PlacedEntity } from "../engine";

/** Module slot overlay (RFC-044 Phase 2): an in-game-style row of module
 *  slot squares near each machine's bottom edge, filled from
 *  `PlacedEntity.items` — `module_slots(entity.name)` total slots,
 *  filled slots colored by module family, empty slots as neutral
 *  outlines. Read-only; reflects data only, nothing stamps modules into
 *  generated layouts until Phase 3 (today's source is imported
 *  blueprints).
 *
 *  Follows the `powerWiresOverlay`/`heatmapOverlay` shape: a fresh
 *  Graphics layer built straight from `LayoutResult.entities` positions
 *  on every layout commit. It never touches the particle atlas, so it
 *  needs no special handling for the two-renderer split (`web/CLAUDE.md`)
 *  — unlike a baked-texture feature, this one is just redrawn from
 *  scratch regardless of which renderer placed the entities.
 */

type ModuleFamily = "speed" | "productivity" | "efficiency" | "quality" | "unknown";

/** Family from the item name's prefix (`speed-module-3` → `speed`, etc).
 *  Recycling byproducts (`speed-module-recycling`) and the
 *  `empty-module-slot` placeholder never appear as real `items` entries;
 *  anything unrecognized (modded modules) falls through to `unknown`. */
function moduleFamily(itemName: string): ModuleFamily {
  if (itemName.startsWith("speed-module")) return "speed";
  if (itemName.startsWith("productivity-module")) return "productivity";
  if (itemName.startsWith("efficiency-module")) return "efficiency";
  if (itemName.startsWith("quality-module")) return "quality";
  return "unknown";
}

// In-game module family palette (speed blue, productivity red,
// efficiency green, quality teal); unrecognized/modded modules get
// neutral grey rather than silently misrepresenting a family.
const FAMILY_COLORS: Record<ModuleFamily, number> = {
  speed: 0x4a90d0,
  productivity: 0xd9432e,
  efficiency: 0x4caf50,
  quality: 0x2fb8b0,
  unknown: 0x8a8a8a,
};

const SLOT_SIZE = TILE_PX * 0.16;
const SLOT_GAP = TILE_PX * 0.05;
const ROW_PAD = 1.5;

export function renderModuleSlotsOverlay(
  entities: PlacedEntity[],
  slotsFor: (entityName: string) => number,
  container: Container,
): Container | null {
  const layer = new Container();
  // Purely decorative: never intercept pointer events — entity
  // hover/pin stays in charge, same convention as the other overlays.
  layer.eventMode = "none";
  let drawn = false;

  for (const entity of entities) {
    if (!entity.items || entity.items.length === 0) continue;
    const filledCount = entity.items.reduce((sum, it) => sum + it.count, 0);
    // Total slots is the greater of the static table and the actual
    // fill: entity classes module_slots doesn't cover yet (mining
    // drills — RFC-044 Phase 1 territory) still show every module
    // that's really there instead of clipping them to 0.
    const totalSlots = Math.max(slotsFor(entity.name), filledCount);
    if (totalSlots === 0) continue;

    const [w, h] = MACHINE_SIZES[entity.name] ?? [1, 1];
    const footprintW = w * TILE_PX;
    const footprintH = h * TILE_PX;
    const originX = (entity.x ?? 0) * TILE_PX;
    const originY = (entity.y ?? 0) * TILE_PX;
    const rowWidth = totalSlots * SLOT_SIZE + (totalSlots - 1) * SLOT_GAP;
    // Right-aligned along the bottom edge — the quality badge
    // (rfc-build-quality, `addQualityBadge`) already owns the
    // bottom-left corner of the same footprint.
    const rowX = originX + Math.max(footprintW - rowWidth - ROW_PAD, ROW_PAD);
    const rowY = originY + footprintH - SLOT_SIZE - ROW_PAD;

    const g = new Graphics();
    let filled = 0;
    for (const it of entity.items) {
      const color = FAMILY_COLORS[moduleFamily(it.item)];
      const qualityColor =
        it.quality && it.quality !== "normal" ? QUALITY_BADGE_COLORS[it.quality] : undefined;
      const borderColor = qualityColor ?? 0x000000;
      const borderAlpha = qualityColor !== undefined ? 0.95 : 0.35;
      const borderWidth = qualityColor !== undefined ? 1 : 0.75;
      for (let i = 0; i < it.count && filled < totalSlots; i++, filled++) {
        const sx = rowX + filled * (SLOT_SIZE + SLOT_GAP);
        g.rect(sx, rowY, SLOT_SIZE, SLOT_SIZE)
          .fill({ color, alpha: 0.9 })
          .stroke({ color: borderColor, width: borderWidth, alpha: borderAlpha });
      }
    }
    // Remaining empty slots: neutral outline, no fill — reads as "open"
    // against the machine body underneath.
    for (; filled < totalSlots; filled++) {
      const sx = rowX + filled * (SLOT_SIZE + SLOT_GAP);
      g.rect(sx, rowY, SLOT_SIZE, SLOT_SIZE)
        .fill({ color: 0x1a1a1a, alpha: 0.35 })
        .stroke({ color: 0x8a8a8a, width: 0.75, alpha: 0.55 });
    }
    layer.addChild(g);
    drawn = true;
  }

  if (!drawn) return null;
  container.addChild(layer);
  return layer;
}
