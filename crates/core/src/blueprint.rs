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
/// slots. See docs/rfp-fulgora-scrap.md Phase 0 "Filter entities" findings.
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
}

#[derive(Serialize)]
struct Blueprint<'a> {
    icons: [(); 0],
    entities: Vec<BlueprintEntity<'a>>,
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
                    // Footprint center from the shared `entity_size` (RFP
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
            }
        })
        .collect();

    let wrapper = BlueprintWrapper {
        blueprint: Blueprint {
            icons: [],
            entities,
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

    #[test]
    fn recycler_position_uses_non_square_center() {
        // Recycler is 2 wide × 4 tall (rfp-fulgora-scrap Phase 0). A
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
