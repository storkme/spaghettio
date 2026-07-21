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
                    let (w, h) = crate::common::entity_size(&ent.name);
                    Position {
                        x: ent.x as f64 + w as f64 / 2.0,
                        y: ent.y as f64 + h as f64 / 2.0,
                    }
                },
                direction: ent.direction as u8,
                recipe: ent.recipe.as_deref(),
                io_type: ent.io_type.as_deref(),
                mirror: ent.mirror,
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
                    let inventory = crate::common::module_inventory_id(&ent.name);
                    let mut stack = 0u32;
                    ent.items
                        .iter()
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

    // Pole copper wires: `compute_pole_wires` returns 0-based entity index
    // pairs; the blueprint numbers entities from 1, so entity_number = index+1.
    // Connector id 5 on both ends = pole-to-pole copper. Recomputed from the
    // same `layout.entities` the entity list above enumerates, so the indices
    // line up exactly.
    let wires: Vec<[u32; 4]> = crate::power_wires::compute_pole_wires(&layout.entities)
        .into_iter()
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
        let pw = crate::power_wires::compute_pole_wires(&layout.entities);
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
