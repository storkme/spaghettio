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
            }
        })
        .collect();

    // Pole copper wires: 0-based entity index pairs; the blueprint numbers
    // entities from 1, so entity_number = index+1. Connector id 5 on both
    // ends = pole-to-pole copper. RFC-044 stored-graph contract: the STORED
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
