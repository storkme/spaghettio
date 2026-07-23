//! LayoutResult → Factorio blueprint string.
//!
//! Format: `"0" + base64(zlib(JSON))`. The `"0"` is Factorio's version byte.

use std::io::Write;

use base64::Engine;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::Serialize;

use crate::models::LayoutResult;

/// Factorio 2.0 inserter `filter_count` — every inserter type (`inserter`,
/// `long-handed-inserter`, `bulk-inserter`, etc.) has exactly 5 filter
/// slots. See docs/rfc-fulgora-scrap.md Phase 0 "Filter entities" findings.
const MAX_INSERTER_FILTERS: usize = 5;

#[derive(Serialize)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
struct BlueprintFilter<'a> {
    index: u32,
    name: &'a str,
}

/// Machines whose engine-side "mirror" is a front-back port flip that the
/// GAME cannot express as `mirror` (game mirror = left-right). Their port
/// layouts are x-symmetric, so the flip is tile-identical to a 180°
/// rotation — the artifact boundary encodes it that way (#400).
fn mirror_is_rotation(name: &str) -> bool {
    matches!(name, "oil-refinery" | "foundry" | "cryogenic-plant")
}

#[derive(Serialize)]
struct BlueprintEntity<'a> {
    entity_number: usize,
    name: &'a str,
    position: Position,
    direction: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe: Option<&'a str>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    io_type: Option<&'a str>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    mirror: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_priority: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_priority: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_filters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<BlueprintFilter<'a>>>,
    /// Entity build quality (`quality :: string?` per lua-api
    /// BlueprintEntity; rfc-build-quality Phase 2). Omitted at normal —
    /// stamped upstream by the layout's functional-only stamp pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<&'static str>,
    /// Modules in this entity, as 2.0 insert plans (RFC-044 Phase 0).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    items: Vec<BlueprintInsertPlan<'a>>,
}

/// Lua-api `BlueprintInsertPlan`: one entry per distinct module
/// (item, quality) with explicit per-unit slot positions. This is the
/// ONLY module encoding Factorio 2.0 honors in a 2.0-versioned envelope
/// (which ours is — there is no 1.x-migration fallback for a name→count
/// map here).
#[derive(Serialize)]
struct BlueprintInsertPlan<'a> {
    id: BlueprintItemId<'a>,
    items: ItemInventoryPositions,
}

/// Lua-api `ItemIDAndQualityIDPair`. Quality omitted at normal.
#[derive(Serialize)]
struct BlueprintItemId<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<&'static str>,
}

#[derive(Serialize)]
struct ItemInventoryPositions {
    in_inventory: Vec<InventoryPosition>,
}

/// Lua-api `InventoryPosition`: `inventory` is the per-entity-class
/// `defines.inventory` id from `common::module_inventory_id`; `stack` is
/// the 0-based module slot. `count` is omitted (defaults to 1 — one
/// entry per module unit, matching the game's own emission).
#[derive(Serialize)]
struct InventoryPosition {
    inventory: u8,
    stack: u32,
}

#[derive(Serialize)]
struct Blueprint<'a> {
    icons: [(); 0],
    entities: Vec<BlueprintEntity<'a>>,
    /// Blueprint-level copper wire graph: each entry is
    /// `[entity_number_a, 5, entity_number_b, 5]` (connector 5 = pole copper).
    /// WITHOUT this, pasted poles are unwired islands and the factory is
    /// power-dead. Omitted entirely when there are <2 poles. See
    /// `crate::power_wires`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    wires: Vec<[u32; 4]>,
    item: &'static str,
    version: u64,
    label: &'a str,
}

#[derive(Serialize)]
struct BlueprintWrapper<'a> {
    blueprint: Blueprint<'a>,
}

/// [`export`] plus the RFC-050 verification manifest: everything the
/// simulation harness needs to feed, drain, and judge the factory —
/// boundary records (engine-emitted, never reconstructed from the
/// artifact), per-item planned rates from the solver, the layout bbox
/// origin for world-offset anchoring, and surplus exits for fluid
/// voiding. Returns `(blueprint_string, manifest_json)`.
pub fn export_with_manifest(
    layout: &LayoutResult,
    solver: &crate::models::SolverResult,
    label: &str,
) -> (String, serde_json::Value) {
    let bp = export(layout, label);
    let bbox_min_x = layout.entities.iter().map(|e| e.x).min().unwrap_or(0);
    let bbox_min_y = layout.entities.iter().map(|e| e.y).min().unwrap_or(0);
    let mut planned: std::collections::BTreeMap<&str, f64> = Default::default();
    for m in &solver.machines {
        for o in &m.outputs {
            *planned.entry(o.item.as_str()).or_default() += o.rate * m.count;
        }
    }
    let boundary = |r: &crate::models::BoundaryRecord| {
        serde_json::json!({
            "item": r.item, "x": r.x, "y": r.y,
            "direction": r.direction as u8,
            "is_fluid": r.is_fluid, "entity": r.entity,
        })
    };
    let manifest = serde_json::json!({
        "label": label,
        "targets": solver
            .external_outputs
            .iter()
            .map(|o| serde_json::json!({"item": o.item, "rate": o.rate}))
            .collect::<Vec<_>>(),
        "external_inputs": solver
            .external_inputs
            .iter()
            .map(|i| serde_json::json!({"item": i.item, "rate": i.rate, "is_fluid": i.is_fluid}))
            .collect::<Vec<_>>(),
        "planned_rates": planned,
        "boundary_inputs": layout.boundary_inputs.iter().map(boundary).collect::<Vec<_>>(),
        "boundary_outputs": layout.boundary_outputs.iter().map(boundary).collect::<Vec<_>>(),
        "surplus_exits": layout.surplus_exits,
        "bbox_min": [bbox_min_x, bbox_min_y],
        "dims": [layout.width, layout.height],
        "entities": layout.entities.len(),
        "stacking": layout.stacking,
        "inserter_capacity": layout.inserter_capacity,
    });
    (bp, manifest)
}

/// Convert a `LayoutResult` into an importable Factorio blueprint string.
pub fn export(layout: &LayoutResult, label: &str) -> String {
    let entities: Vec<BlueprintEntity> = layout
        .entities
        .iter()
        .enumerate()
        .map(|(i, ent)| {
            let filters = if ent.filters.is_empty() {
                None
            } else {
                debug_assert!(
                    ent.filters.len() <= MAX_INSERTER_FILTERS,
                    "inserter filter_count is {MAX_INSERTER_FILTERS}; got {} filters for {} at ({}, {})",
                    ent.filters.len(),
                    ent.name,
                    ent.x,
                    ent.y,
                );
                Some(
                    ent.filters
                        .iter()
                        .take(MAX_INSERTER_FILTERS)
                        .enumerate()
                        .map(|(i, name)| BlueprintFilter {
                            index: (i + 1) as u32,
                            name,
                        })
                        .collect(),
                )
            };
            BlueprintEntity {
                entity_number: i + 1,
                name: &ent.name,
                position: {
                    // Footprint center from the shared `entity_size` (RFC
                    // Phase 3a-i): a 2×2 substation exports at x+1.0, not the
                    // x+0.5 the old machine-only lookup produced.
                    //
                    // Splitters are direction-dependent (2 wide perpendicular
                    // to flow) and `entity_size` is direction-blind (1×1
                    // fallback) — without this arm every exported splitter
                    // sat half a tile off, the game snapped the ghost to an
                    // adjacent column, and belt connectivity silently broke
                    // in-game (found by the RFC-050 harness: the gear
                    // fixture's merger splitter buffered 56 items into a
                    // void). The shared `oriented_splitter_dims` covers the
                    // exact 2-wide family — NOT `lane-splitter`, which is
                    // 1×1 despite the name (PR #350 review).
                    let (w, h) = crate::common::oriented_splitter_dims(&ent.name, ent.direction)
                        .unwrap_or_else(|| crate::common::entity_size(&ent.name));
                    Position {
                        x: ent.x as f64 + w as f64 / 2.0,
                        y: ent.y as f64 + h as f64 / 2.0,
                    }
                },
                // GAME QUIRK (found by the RFC-050 harness, 2026-07-22):
                // Factorio reads an INSERTER's direction as its PICKUP
                // side ("inserters point backwards"), while the engine's
                // convention is drop-side. Flip 180° at the artifact
                // boundary — the parser un-flips on import — or every
                // exported inserter runs backwards in-game.
                //
                // PIPE-TO-GROUND is the same class (#364, sim-measured
                // 2026-07-22): the game's blueprint direction is the
                // SURFACE-opening side, while the engine's convention
                // (F5) is the underground-run side — exported pairs
                // faced each other and never connected, severing every
                // fluid feed in-game while validating clean.
                direction: if ent.name.contains("inserter") || ent.name == "pipe-to-ground" {
                    (ent.direction as u8 + 8) % 16
                } else if mirror_is_rotation(&ent.name) && ent.mirror {
                    // MIRRORED FLUID MACHINES are the same artifact-boundary
                    // class (#400, sim-measured 2026-07-23): the engine's
                    // "mirror" models a FRONT-BACK (y) port flip, but the
                    // game's `mirror` flag flips LEFT-RIGHT across the facing
                    // axis — an exported (North, mirror) refinery still has
                    // its inputs on the south in-game, so crude sat ON the
                    // engine's intended port tiles and never entered (total
                    // stall, first refinery measurement). For x-symmetric
                    // port layouts (refinery/foundry/cryo — asserted by the
                    // fluid_ports provenance script) the engine's y-flip is
                    // TILE-IDENTICAL to a 180° rotation, so export encodes it
                    // as (direction+8, mirror:false); the parser reverses it.
                    (ent.direction as u8 + 8) % 16
                } else {
                    ent.direction as u8
                },
                recipe: ent.recipe.as_deref(),
                io_type: ent.io_type.as_deref(),
                mirror: ent.mirror && !mirror_is_rotation(&ent.name),
                input_priority: ent.input_priority.as_deref(),
                output_priority: ent.output_priority.as_deref(),
                use_filters: filters.is_some().then_some(true),
                filters,
                quality: ent
                    .quality
                    .filter(|q| *q != crate::common::QualityTier::Normal)
                    .map(|q| q.name()),
                items: {
                    // Slot positions run sequentially across ALL modules
                    // in the entity: module A ×2 takes stacks 0,1; the
                    // next module starts at 2.
                    //
                    // Only MODULE-shaped requests are emitted: the parser
                    // collapses every item request (fuel, ammo) into
                    // `items`, and re-emitting those into the MODULE
                    // inventory would mis-target them (a coal request is
                    // not a module — retro-review m1). Dropping them
                    // matches pre-RFC-044 export behavior; full
                    // inventory/count fidelity is a recorded followup
                    // (docs/module-followups.md). count-0 entries carry
                    // no slot positions and are skipped outright.
                    let inventory = crate::common::module_inventory_id(&ent.name);
                    let mut stack = 0u32;
                    ent.items
                        .iter()
                        .filter(|m| {
                            m.count > 0 && crate::common::game_module_family(&m.item).is_some()
                        })
                        .map(|m| BlueprintInsertPlan {
                            id: BlueprintItemId {
                                name: &m.item,
                                quality: m
                                    .quality
                                    .filter(|q| *q != crate::common::QualityTier::Normal)
                                    .map(|q| q.name()),
                            },
                            items: ItemInventoryPositions {
                                in_inventory: (0..m.count)
                                    .map(|_| {
                                        let p = InventoryPosition { inventory, stack };
                                        stack += 1;
                                        p
                                    })
                                    .collect(),
                            },
                        })
                        .collect()
                },
            }
        })
        .collect();

    // Pole copper wires: 0-based entity index pairs; the blueprint numbers
    // entities from 1, so entity_number = index+1. Connector id 5 on both
    // ends = pole-to-pole copper. RFC-045 stored-graph contract: the STORED
    // `layout.power_wires` is authoritative (it indexes the same
    // `layout.entities` this export enumerates); `wires_for` falls back to
    // a dense derivation only for layouts that never computed wires.
    let wires: Vec<[u32; 4]> = crate::power_wires::wires_for(layout)
        .iter()
        .copied()
        .map(|(a, b)| {
            [
                a + 1,
                crate::power_wires::POLE_COPPER,
                b + 1,
                crate::power_wires::POLE_COPPER,
            ]
        })
        .collect();

    let wrapper = BlueprintWrapper {
        blueprint: Blueprint {
            icons: [],
            entities,
            wires,
            item: "blueprint",
            version: 562949955518464,
            label,
        },
    };

    let json = serde_json::to_vec(&wrapper).expect("blueprint serialization cannot fail");

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json).expect("zlib write cannot fail");
    let compressed = encoder.finish().expect("zlib finish cannot fail");

    let b64 = base64::engine::general_purpose::STANDARD.encode(&compressed);
    format!("0{b64}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, PlacedEntity};
    use std::io::Read;

    #[test]
    fn manifest_boundaries_match_real_layout() {
        // RFC-050 Phase 0: the engine's boundary records for the tier-1
        // gear fixture must name the same trunk heads and merger exit
        // the calibrated harness dogfood found empirically (heads at
        // layout x=1,2 y=0 southbound; one merger sink).
        use rustc_hash::FxHashSet;
        let inputs: FxHashSet<String> = ["iron-ore"].iter().map(|s| s.to_string()).collect();
        let solved = crate::solver::solve("iron-gear-wheel", 10.0, &inputs, "assembling-machine-3")
            .expect("solve");
        let layout = crate::bus::layout::build_bus_layout(
            &solved,
            crate::bus::layout::LayoutOptions {
                max_belt_tier: Some("transport-belt".into()),
                ..Default::default()
            },
        )
        .expect("layout");
        let (_bp, manifest) = export_with_manifest(&layout, &solved, "gear");

        let feeds = manifest["boundary_inputs"].as_array().unwrap();
        assert_eq!(feeds.len(), 2, "two external iron-ore lanes: {feeds:?}");
        for f in feeds {
            assert_eq!(f["item"], "iron-ore");
            assert_eq!(f["y"], 0);
            assert_eq!(f["direction"], 8, "trunk heads flow south");
        }
        let exits = manifest["boundary_outputs"].as_array().unwrap();
        assert_eq!(exits.len(), 1, "one merger sink: {exits:?}");
        assert_eq!(exits[0]["item"], "iron-gear-wheel");
        assert_eq!(manifest["planned_rates"]["iron-gear-wheel"].as_f64().unwrap().round(), 10.0);
        assert_eq!(manifest["bbox_min"][0], 1, "leftmost entity is the x=1 trunk head");
    }

    #[test]
    fn round_trip_small_fixture() {
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-3".into(),
                    x: 0,
                    y: 0,
                    direction: EntityDirection::North,
                    recipe: Some("iron-gear-wheel".into()),
                    io_type: None,
                    carries: None,
                    mirror: false,
                    segment_id: None,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "transport-belt".into(),
                    x: 3,
                    y: 0,
                    direction: EntityDirection::East,
                    recipe: None,
                    io_type: None,
                    carries: None,
                    mirror: false,
                    segment_id: None,
                    ..Default::default()
                },
            ],
            width: 4,
            height: 1,
            ..Default::default()
        };
        let s = export(&layout, "test");

        assert!(s.starts_with('0'));
        let b64 = &s[1..];
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["blueprint"]["label"], "test");
        let ents = parsed["blueprint"]["entities"].as_array().unwrap();
        assert_eq!(ents.len(), 2);
        assert_eq!(ents[0]["name"], "assembling-machine-3");
        assert_eq!(ents[0]["recipe"], "iron-gear-wheel");
        assert_eq!(ents[0]["entity_number"], 1);
        assert_eq!(ents[0]["direction"], 0);
        assert_eq!(ents[1]["entity_number"], 2);
        assert_eq!(ents[1]["direction"], 4);
        // 3x3 assembler at (0,0) → center at (1.5, 1.5)
        assert_eq!(ents[0]["position"]["x"], 1.5);
        assert_eq!(ents[0]["position"]["y"], 1.5);
        // 1x1 belt at (3,0) → center at (3.5, 0.5)
        assert_eq!(ents[1]["position"]["x"], 3.5);
        // mirror should be absent when false
        assert!(ents[0].get("mirror").is_none());
        // recipe should be absent for belt
        assert!(ents[1].get("recipe").is_none());
    }

    /// Splitters are 2 tiles wide PERPENDICULAR to flow; the exporter's
    /// center math must be direction-aware or every splitter sits half a
    /// tile off and the game snaps it into the wrong column, silently
    /// severing belt connectivity (found by the RFC-050 harness: the
    /// gear fixture's merger splitter buffered items into a void; the
    /// #345 fixture's balancers are full of these).
    #[test]
    fn splitter_export_center_is_direction_aware() {
        for (dir, cx, cy) in [
            (EntityDirection::South, 6.0, 3.5), // 2 wide, 1 tall
            (EntityDirection::North, 6.0, 3.5),
            (EntityDirection::East, 5.5, 4.0), // 1 wide, 2 tall
            (EntityDirection::West, 5.5, 4.0),
        ] {
            let layout = LayoutResult {
                entities: vec![PlacedEntity {
                    name: "express-splitter".into(),
                    x: 5,
                    y: 3,
                    direction: dir,
                    ..Default::default()
                }],
                width: 8,
                height: 6,
                ..Default::default()
            };
            let bp = decode_blueprint(&export(&layout, "split"));
            assert_eq!(bp["entities"][0]["position"]["x"], cx, "{dir:?}");
            assert_eq!(bp["entities"][0]["position"]["y"], cy, "{dir:?}");
        }
    }

    /// PR #350 review: the 2-wide family is exactly {splitter, fast-,
    /// express-, turbo-}; `lane-splitter` is 1×1 despite the name and
    /// must NOT be widened.
    #[test]
    fn splitter_family_subset_is_exact() {
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "turbo-splitter".into(),
                    x: 0,
                    y: 0,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "lane-splitter".into(),
                    x: 4,
                    y: 0,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 6,
            height: 2,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "fam"));
        // turbo: 2-wide → integer center on x.
        assert_eq!(bp["entities"][0]["position"]["x"], 1.0);
        assert_eq!(bp["entities"][0]["position"]["y"], 0.5);
        // lane-splitter: 1×1 → half centers on both axes.
        assert_eq!(bp["entities"][1]["position"]["x"], 4.5);
        assert_eq!(bp["entities"][1]["position"]["y"], 0.5);
    }

    /// Factorio reads an inserter's `direction` as its PICKUP side
    /// ("inserters point backwards" — found by the RFC-050 harness when
    /// every exported factory deadlocked in-sim: input inserters pulled
    /// from empty machines, outputs from empty belts). The engine's
    /// convention is drop-side, so export must flip 180° and ONLY for
    /// inserters.
    #[test]
    fn inserter_directions_flip_to_game_convention() {
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "stack-inserter".into(),
                    x: 0,
                    y: 0,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "long-handed-inserter".into(),
                    x: 2,
                    y: 0,
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "transport-belt".into(),
                    x: 4,
                    y: 0,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 5,
            height: 1,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "dirflip"));
        let ents = bp["entities"].as_array().unwrap();
        // Engine South (8) exports as game North (0) for inserters…
        assert_eq!(ents[0]["direction"], 0);
        // …engine North (0) as game South (8)…
        assert_eq!(ents[1]["direction"], 8);
        // …and non-inserters are untouched.
        assert_eq!(ents[2]["direction"], 8);
    }

    /// Pipe-to-ground shares the artifact-boundary flip (#364): the game's
    /// blueprint direction is the SURFACE-opening side; the engine's (F5)
    /// is the underground side. The engine's vertical drop pair (top
    /// "input" South tunneling down to bottom "output" North) must export
    /// facing AWAY from each other — top North (0), bottom South (8) — or
    /// the pair never connects in-game (sim-measured: both PTGs empty
    /// beside a full header pipe, machine fluid-starved).
    #[test]
    fn pipe_to_ground_directions_flip_to_game_convention() {
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "pipe-to-ground".into(),
                    x: 0,
                    y: 0,
                    direction: EntityDirection::South,
                    io_type: Some("input".into()),
                    ..Default::default()
                },
                PlacedEntity {
                    name: "pipe-to-ground".into(),
                    x: 0,
                    y: 2,
                    direction: EntityDirection::North,
                    io_type: Some("output".into()),
                    ..Default::default()
                },
                PlacedEntity {
                    name: "pipe".into(),
                    x: 1,
                    y: 0,
                    direction: EntityDirection::North,
                    ..Default::default()
                },
            ],
            width: 2,
            height: 3,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "ptgflip"));
        let ents = bp["entities"].as_array().unwrap();
        // Engine South (tunnel down) exports as game North (surface up)…
        assert_eq!(ents[0]["direction"], 0);
        // …engine North (faces the input) as game South (surface down)…
        assert_eq!(ents[1]["direction"], 8);
        // …and plain pipes are untouched.
        assert_eq!(ents[2]["direction"], 0);
    }

    /// The parser applies the inverse flip so imported game-convention
    /// inserters read correctly under engine semantics, and round-trips
    /// are identity.
    #[test]
    fn inserter_direction_round_trip_is_identity() {
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "fast-inserter".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::East,
                ..Default::default()
            }],
            width: 1,
            height: 1,
            ..Default::default()
        };
        let s = export(&layout, "rt");
        // Artifact carries the game convention (East 4 → West 12)…
        assert_eq!(decode_blueprint(&s)["entities"][0]["direction"], 12);
        // …and parsing restores the engine convention.
        let parsed = crate::blueprint_parser::parse_blueprint_string(&s).unwrap();
        assert_eq!(parsed.entities[0].direction, EntityDirection::East);
    }

    /// The exported module encoding must match the game's own emission
    /// byte-shape (draftsman 2.0.76 reference, RFC-044 Phase 0): one
    /// insert-plan per module id, one `in_inventory` entry per module
    /// unit with sequential 0-based stacks, NO `count` field, quality
    /// omitted at normal, `inventory` per entity class.
    #[test]
    fn module_items_match_insert_plan_reference_shape() {
        use crate::common::QualityTier;
        use crate::models::ModuleItem;

        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "assembling-machine-3".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".into()),
                items: vec![ModuleItem {
                    item: "productivity-module-3".into(),
                    count: 4,
                    quality: Some(QualityTier::Legendary),
                }],
                ..Default::default()
            }],
            width: 3,
            height: 3,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "ref"));
        let items = &bp["entities"][0]["items"];

        // Exact shape from the draftsman-emitted reference blueprint.
        let expected = serde_json::json!([{
            "id": {"name": "productivity-module-3", "quality": "legendary"},
            "items": {"in_inventory": [
                {"inventory": 4, "stack": 0},
                {"inventory": 4, "stack": 1},
                {"inventory": 4, "stack": 2},
                {"inventory": 4, "stack": 3}
            ]}
        }]);
        assert_eq!(items, &expected);
    }

    /// Export → parse round trip preserves modules (item, count, quality)
    /// across three entity classes, and the raw JSON carries the
    /// per-class inventory id (assembler=4, beacon=1, drill=2). NOTE:
    /// the parser discards the inventory field, so only the raw-JSON leg
    /// of this test sees a wrong id — and only for classes exercised
    /// here; the in-game paste anchor (RFC-044 KC2) is the real gate.
    #[test]
    fn moduled_entities_round_trip_with_quality() {
        use crate::common::QualityTier;
        use crate::models::ModuleItem;

        let mk = |name: &str, x: i32, items: Vec<ModuleItem>| PlacedEntity {
            name: name.into(),
            x,
            y: 0,
            direction: EntityDirection::North,
            items,
            ..Default::default()
        };
        let layout = LayoutResult {
            entities: vec![
                mk(
                    "assembling-machine-3",
                    0,
                    vec![
                        ModuleItem {
                            item: "productivity-module-3".into(),
                            count: 2,
                            quality: Some(QualityTier::Legendary),
                        },
                        // Normal quality: `quality` key must be absent,
                        // and stacks continue from the previous module.
                        ModuleItem {
                            item: "speed-module".into(),
                            count: 1,
                            quality: None,
                        },
                    ],
                ),
                mk(
                    "beacon",
                    4,
                    vec![ModuleItem {
                        item: "speed-module-3".into(),
                        count: 2,
                        quality: Some(QualityTier::Rare),
                    }],
                ),
                mk(
                    "electric-mining-drill",
                    8,
                    vec![ModuleItem {
                        item: "efficiency-module".into(),
                        count: 3,
                        quality: None,
                    }],
                ),
            ],
            width: 11,
            height: 3,
            ..Default::default()
        };
        let s = export(&layout, "modtrip");

        // Raw-JSON leg: per-class inventory ids + normal-quality omission
        // + sequential stacks across modules within one entity.
        let bp = decode_blueprint(&s);
        let ents = bp["entities"].as_array().unwrap();
        let asm = &ents[0]["items"];
        assert_eq!(asm[0]["id"]["quality"], "legendary");
        assert!(asm[1]["id"].get("quality").is_none());
        assert_eq!(asm[1]["items"]["in_inventory"][0]["stack"], 2);
        assert_eq!(asm[0]["items"]["in_inventory"][0]["inventory"], 4);
        assert_eq!(ents[1]["items"][0]["items"]["in_inventory"][0]["inventory"], 1);
        assert_eq!(ents[2]["items"][0]["items"]["in_inventory"][0]["inventory"], 2);

        // Parser leg: everything the model holds survives the trip.
        let parsed = crate::blueprint_parser::parse_blueprint_string(&s).unwrap();
        let find = |name: &str| {
            parsed
                .entities
                .iter()
                .find(|e| e.name == name)
                .unwrap_or_else(|| panic!("{name} missing after round trip"))
        };
        let asm = find("assembling-machine-3");
        assert_eq!(asm.items.len(), 2);
        assert_eq!(asm.items[0].item, "productivity-module-3");
        assert_eq!(asm.items[0].count, 2);
        assert_eq!(asm.items[0].quality, Some(QualityTier::Legendary));
        assert_eq!(asm.items[1].item, "speed-module");
        assert_eq!(asm.items[1].count, 1);
        assert_eq!(asm.items[1].quality, None);
        let beacon = find("beacon");
        assert_eq!(beacon.items[0].count, 2);
        assert_eq!(beacon.items[0].quality, Some(QualityTier::Rare));
        let drill = find("electric-mining-drill");
        assert_eq!(drill.items[0].count, 3);
        assert_eq!(drill.items[0].quality, None);
    }

    /// Pumpjack is a mining-drill prototype: its modules export to
    /// inventory 2, not the crafting default 4 (retro-review M1 — wrong
    /// id fails silently on paste and round-trips can't see it).
    #[test]
    fn pumpjack_modules_export_to_drill_inventory() {
        use crate::models::ModuleItem;
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "pumpjack".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                items: vec![ModuleItem {
                    item: "speed-module-3".into(),
                    count: 2,
                    quality: None,
                }],
                ..Default::default()
            }],
            width: 3,
            height: 3,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "pj"));
        let positions = &bp["entities"][0]["items"][0]["items"]["in_inventory"];
        assert_eq!(positions[0]["inventory"], 2);
        assert_eq!(positions[1]["inventory"], 2);
    }

    /// Non-module item requests (fuel, ammo) parsed off imported
    /// blueprints must NOT re-export as module-inventory insert plans —
    /// a coal request is not a module (retro-review m1; dropping them
    /// matches pre-RFC-044 behavior, full fidelity is a followup).
    #[test]
    fn non_module_item_requests_are_not_exported_as_modules() {
        use crate::models::ModuleItem;
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "stone-furnace".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                items: vec![
                    ModuleItem { item: "coal".into(), count: 50, quality: None },
                    // count-0 entries carry no positions; skipped too.
                    ModuleItem { item: "speed-module".into(), count: 0, quality: None },
                ],
                ..Default::default()
            }],
            width: 2,
            height: 2,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "fuel"));
        assert!(
            bp["entities"][0].get("items").is_none(),
            "fuel request leaked into module insert plans"
        );
    }

    /// Decode an exported blueprint string back to its JSON `blueprint` object.
    fn decode_blueprint(bp: &str) -> serde_json::Value {
        let b64 = &bp[1..];
        let compressed = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        parsed["blueprint"].clone()
    }

    #[test]
    fn export_emits_pole_copper_wires() {
        // A 2×2 cluster of medium poles at pitch 5 (all within reach 9) plus a
        // belt between them. The export MUST carry a blueprint-level `wires`
        // array — its absence was the power-dead bug. Each wire is
        // `[entity_number_a, 5, entity_number_b, 5]` with entity_number = idx+1.
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity { name: "medium-electric-pole".into(), x: 0, y: 0, ..Default::default() },
                PlacedEntity { name: "transport-belt".into(), x: 2, y: 0, ..Default::default() },
                PlacedEntity { name: "medium-electric-pole".into(), x: 5, y: 0, ..Default::default() },
                PlacedEntity { name: "medium-electric-pole".into(), x: 0, y: 5, ..Default::default() },
                PlacedEntity { name: "medium-electric-pole".into(), x: 5, y: 5, ..Default::default() },
            ],
            width: 6,
            height: 6,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "wires"));
        let wires = bp["wires"].as_array().expect("blueprint must have a wires array");
        assert!(!wires.is_empty(), "in-reach poles must produce copper wires");
        // The belt (entity_number 2) must never appear as a wire endpoint.
        for w in wires {
            let w = w.as_array().unwrap();
            assert_eq!(w.len(), 4, "each wire is a 4-tuple");
            assert_eq!(w[1], 5, "connector id a must be pole copper (5)");
            assert_eq!(w[3], 5, "connector id b must be pole copper (5)");
            assert_ne!(w[0], 2, "belt (entity_number 2) is not a pole");
            assert_ne!(w[2], 2, "belt (entity_number 2) is not a pole");
        }
        // The four poles are entity_numbers {1,3,4,5}; the wire graph must
        // connect all of them (compute_pole_wires drives both export + check).
        let pw = crate::power_wires::compute_pole_wires(&layout.entities, crate::power_wires::WireMode::Dense);
        assert_eq!(
            crate::power_wires::count_disconnected_poles(&layout.entities, &pw),
            0,
            "the four-pole cluster must form one copper network"
        );
    }

    #[test]
    fn export_omits_wires_key_for_single_pole() {
        // Fewer than 2 poles → no copper wires → the `wires` key is omitted
        // entirely (skip_serializing_if), matching draftsman's shape for a
        // wireless blueprint.
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "medium-electric-pole".into(),
                x: 0,
                y: 0,
                ..Default::default()
            }],
            width: 1,
            height: 1,
            ..Default::default()
        };
        let bp = decode_blueprint(&export(&layout, "one-pole"));
        assert!(bp.get("wires").is_none(), "single pole must not emit a wires key");
    }

    #[test]
    fn recycler_position_uses_non_square_center() {
        // Recycler is 2 wide × 4 tall (rfc-fulgora-scrap Phase 0). A
        // square-assuming position calc would center at (x+1, y+1) (using
        // width for both axes) or (x+2, y+2) (using height for both).
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "recycler".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                ..Default::default()
            }],
            width: 2,
            height: 4,
            ..Default::default()
        };
        let s = export(&layout, "test");
        let b64 = &s[1..];
        let compressed = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let ents = parsed["blueprint"]["entities"].as_array().unwrap();
        // 2×4 recycler at (0,0) → center at (1.0, 2.0), not (1.5, 1.5) or (2.0, 2.0).
        assert_eq!(ents[0]["position"]["x"], 1.0);
        assert_eq!(ents[0]["position"]["y"], 2.0);
    }

    #[test]
    fn priority_round_trips() {
        let layout = LayoutResult {
            entities: vec![PlacedEntity {
                name: "splitter".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::South,
                input_priority: Some("left".into()),
                output_priority: Some("right".into()),
                ..Default::default()
            }],
            width: 2,
            height: 1,
            ..Default::default()
        };
        let s = export(&layout, "priority_test");
        let b64 = &s[1..];
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let ents = parsed["blueprint"]["entities"].as_array().unwrap();
        assert_eq!(ents[0]["input_priority"], "left");
        assert_eq!(ents[0]["output_priority"], "right");

        // Absent when None.
        let bare = LayoutResult {
            entities: vec![PlacedEntity {
                name: "splitter".into(),
                x: 0,
                y: 0,
                direction: EntityDirection::South,
                ..Default::default()
            }],
            width: 2,
            height: 1,
            ..Default::default()
        };
        let s = export(&bare, "no_priority");
        let b64 = &s[1..];
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let ents = parsed["blueprint"]["entities"].as_array().unwrap();
        assert!(ents[0].get("input_priority").is_none());
        assert!(ents[0].get("output_priority").is_none());
    }
}
