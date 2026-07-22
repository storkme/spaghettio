//! Factorio blueprint string → LayoutResult.
//!
//! Reverse of `blueprint.rs`. Decodes `"0" + base64(zlib(JSON))` and converts
//! the Factorio entity format (center-based float positions, raw direction ints)
//! into our tile-grid `LayoutResult`.

use std::io::Read;

use base64::Engine;
use flate2::read::ZlibDecoder;
use serde::Deserialize;

use crate::models::{EntityDirection, LayoutResult, PlacedEntity};

// ---- Raw Factorio blueprint JSON types ----

#[derive(Deserialize)]
struct BpRoot {
    blueprint: Option<BpData>,
    blueprint_book: Option<BpBook>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BpBook {
    #[serde(default)]
    blueprints: Vec<BpBookEntry>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BpBookEntry {
    blueprint: Option<BpData>,
    blueprint_book: Option<BpBook>,
    #[serde(default)]
    index: u32,
}

#[derive(Deserialize)]
struct BpData {
    #[serde(default)]
    entities: Vec<BpEntity>,
    /// Blueprint-level wire graph. Each entry is
    /// `[entity_number_a, connector_a, entity_number_b, connector_b]`.
    /// Pole copper wires use connector 5 on both ends
    /// ([`crate::power_wires::POLE_COPPER`]); circuit wires use other ids and
    /// are ignored here. `Vec<Vec<i64>>` (not a fixed `[_; 4]`) so a malformed
    /// entry in a community blueprint is skipped, not a hard parse error.
    #[serde(default)]
    wires: Vec<Vec<i64>>,
    #[serde(default)]
    label: Option<String>,
}

/// Parsed module item. All three Factorio formats collapse into this.
/// Only the 2.0 insert-plan format carries a quality; 1.x formats leave
/// it `None` (= normal).
struct BpEntityItem {
    item: String,
    count: u32,
    quality: Option<String>,
}

/// Factorio uses multiple formats for items within an entity:
/// - 1.x array:  `[{"item": "speed-module-3", "count": 2}]`
/// - 1.x map:    `{"speed-module-3": 2}`
/// - 2.0 array:  `[{"id": {"name": "efficiency-module"}, "items": {...}}]`
///
/// This enum handles all of them via a custom deserializer.
#[derive(Default)]
struct BpEntityItems(Vec<BpEntityItem>);

/// Helper for the 2.0 `"id"` field which can be a string or `{"name": "..."}`.
fn extract_id(val: &serde_json::Value) -> Option<String> {
    match val {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => map
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

impl<'de> serde::Deserialize<'de> for BpEntityItems {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de;

        struct ItemsVisitor;
        impl<'de> de::Visitor<'de> for ItemsVisitor {
            type Value = BpEntityItems;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("array of items or map of item_name→count")
            }

            // Array format (both 1.x and 2.0)
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut items = Vec::new();
                while let Some(obj) = seq.next_element::<serde_json::Value>()? {
                    if let Some(item) = parse_item_value(&obj) {
                        items.push(item);
                    }
                }
                Ok(BpEntityItems(items))
            }

            // Map format (1.x old style: {"speed-module-3": 2})
            fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut items = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, u32>()? {
                    items.push(BpEntityItem {
                        item: key,
                        count: value,
                        quality: None,
                    });
                }
                Ok(BpEntityItems(items))
            }
        }

        deserializer.deserialize_any(ItemsVisitor)
    }
}

/// Parse a single item entry from either 1.x or 2.0 format.
fn parse_item_value(val: &serde_json::Value) -> Option<BpEntityItem> {
    let obj = val.as_object()?;

    // 1.x format: {"item": "speed-module-3", "count": 2}
    if let Some(item_name) = obj.get("item").and_then(|v| v.as_str()) {
        let count = obj
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
        return Some(BpEntityItem {
            item: item_name.to_string(),
            count,
            quality: None,
        });
    }

    // 2.0 format: {"id": {"name": "efficiency-module", "quality": "rare"?}, "items": {...}}
    if let Some(id_val) = obj.get("id") {
        if let Some(item_name) = extract_id(id_val) {
            // Count from nested items.in_inventory array length, or default 1
            let count = obj
                .get("items")
                .and_then(|v| v.get("in_inventory"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.len() as u32)
                .unwrap_or(1);
            let quality = id_val
                .get("quality")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return Some(BpEntityItem {
                item: item_name,
                count,
                quality,
            });
        }
    }

    None
}

/// Factorio 2.0 `filters` array entry: `{"index": 1, "name": "iron-plate"}`.
/// 1-indexed; order is not guaranteed by the format so entries are sorted
/// by `index` before being collapsed into `PlacedEntity.filters`.
#[derive(Deserialize)]
struct BpFilter {
    index: u32,
    name: String,
}

#[derive(Deserialize)]
struct BpEntity {
    /// Factorio's explicit 1-based id, referenced by the blueprint `wires`
    /// array. Absent in some hand-written JSON — falls back to array position.
    #[serde(default)]
    entity_number: Option<u64>,
    name: String,
    position: BpPosition,
    #[serde(default)]
    direction: u8,
    recipe: Option<String>,
    /// "input" | "output" for underground belts / pipe-to-ground
    #[serde(rename = "type")]
    io_type: Option<String>,
    /// Modules/items inserted into this entity (handles both array and map formats).
    #[serde(default)]
    items: BpEntityItems,
    /// Inserter item filter (whitelist mode only — v1 has no blacklist
    /// support). `use_filters` itself isn't needed: presence of a non-empty
    /// `filters` array is sufficient to reconstruct `PlacedEntity.filters`.
    #[serde(default)]
    filters: Vec<BpFilter>,
    /// Factorio 2.0 entity build quality (`quality :: string?` per
    /// lua-api BlueprintEntity; absent = normal). Silently dropped before
    /// `docs/rfc-build-quality.md` Phase 0, which misreported quality-built
    /// community blueprints. Kept as the raw string here; the conversion
    /// maps the five vanilla names via `QualityTier::from_name` (modded
    /// quality names fall back to `None`/normal — vanilla-only fidelity).
    /// Permissively deserialized: before Phase 0 this key was an *unknown
    /// field* serde ignored wholesale, so a non-string value (corrupted /
    /// modded JSON) must degrade to `None`, not fail the whole blueprint
    /// parse (adversarial-review finding, 2026-07-20).
    #[serde(default, deserialize_with = "de_quality_permissive")]
    quality: Option<String>,
}

/// See `BpEntity.quality`: any valid JSON value is accepted; only strings
/// carry through. Restores the pre-Phase-0 "garbage here never breaks the
/// parse" behavior while still reading well-formed quality fields.
fn de_quality_permissive<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(v.as_str().map(str::to_owned))
}

#[derive(Deserialize)]
struct BpPosition {
    x: f64,
    y: f64,
}

// ---- Entity footprint lookup ----

/// Returns (width_tiles, height_tiles) for entities that aren't 1×1.
/// Direction is needed for splitters (2 tiles perpendicular to flow).
fn entity_footprint(name: &str, direction: EntityDirection) -> (i32, i32) {
    match name {
        "assembling-machine-1"
        | "assembling-machine-2"
        | "assembling-machine-3"
        | "chemical-plant"
        | "electric-furnace"
        | "centrifuge"
        | "lab"
        | "beacon"
        | "storage-tank"
        | "electric-mining-drill"
        | "biochamber" => (3, 3),

        "oil-refinery" | "foundry" | "biolab" | "cryogenic-plant" => (5, 5),
        "rocket-silo" => (9, 9),
        "big-electric-pole" | "substation" | "steel-furnace" => (2, 2),
        "electromagnetic-plant" => (4, 4),
        "recycler" => (2, 4),
        "crusher" => (2, 3),

        // Splitters: 2 tiles wide perpendicular to flow direction. The
        // shared helper covers turbo-splitter too (115 corpus instances
        // were misparsed half a tile off before PR #350's review caught
        // the exporter/parser table divergence) and correctly excludes
        // the 1×1 lane-splitter.
        "splitter" | "fast-splitter" | "express-splitter" | "turbo-splitter" => {
            let (w, h) = crate::common::oriented_splitter_dims(name, direction)
                .expect("splitter family covered by oriented_splitter_dims");
            (w as i32, h as i32)
        }

        _ => (1, 1),
    }
}

// ---- Direction parsing ----

/// Map Factorio direction integer to EntityDirection.
///
/// Modern Factorio (≥0.17) uses 0/4/8/12 for N/E/S/W.
/// Older versions (0-7 eight-way): 0=N, 2=E, 4=S, 6=W.
/// We handle both by treating even values in 0-6 range as old-format
/// and values 8/12 as unambiguously modern.
fn parse_direction(d: u8) -> EntityDirection {
    match d {
        0 => EntityDirection::North,
        4 => EntityDirection::East,
        8 => EntityDirection::South,
        12 => EntityDirection::West,
        // Old 8-way format
        2 => EntityDirection::East,
        6 => EntityDirection::West,
        _ => EntityDirection::North,
    }
}

/// 180° rotation in `EntityDirection` space — the inserter
/// pickup-vs-drop convention flip (see the import site and the
/// exporter's mirror-image flip).
fn flip180(d: EntityDirection) -> EntityDirection {
    match d {
        EntityDirection::North => EntityDirection::South,
        EntityDirection::East => EntityDirection::West,
        EntityDirection::South => EntityDirection::North,
        EntityDirection::West => EntityDirection::East,
    }
}

// ---- Public API ----

/// A parsed blueprint with an optional label.
#[derive(Debug, Clone)]
pub struct ParsedBlueprint {
    pub label: Option<String>,
    pub layout: LayoutResult,
}

/// Decode a blueprint string to its JSON root.
fn decode_bp_string(bp: &str) -> Result<BpRoot, String> {
    let bp = bp.trim();
    if !bp.starts_with('0') {
        return Err("Blueprint string must start with '0'".into());
    }

    let b64 = &bp[1..];
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("base64 decode error: {e}"))?;

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .map_err(|e| format!("zlib decompress error: {e}"))?;

    serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {e}"))
}

/// Convert a `BpData` to a `LayoutResult`, normalizing positions to (0,0).
fn bp_data_to_layout(bp_data: BpData) -> LayoutResult {
    let wires_raw = bp_data.wires;
    let mut entities: Vec<PlacedEntity> = Vec::with_capacity(bp_data.entities.len());
    // entity_number (explicit, else positional 1-based) → 0-based index in
    // `entities`, so the `wires` array can be resolved to entity indices.
    let mut num_to_idx: std::collections::HashMap<u64, usize> =
        std::collections::HashMap::with_capacity(bp_data.entities.len());

    for (pos, raw) in bp_data.entities.into_iter().enumerate() {
        let entity_number = raw.entity_number.unwrap_or((pos + 1) as u64);
        num_to_idx.insert(entity_number, entities.len());
        // GAME QUIRK (RFC-050 harness, 2026-07-22): Factorio's inserter
        // direction is the PICKUP side; the engine's convention is
        // drop-side. Un-flip on import so imported inserters read
        // correctly under engine semantics (the exporter flips back).
        // The flip happens AFTER parsing, in EntityDirection space —
        // raw-byte arithmetic would destroy legacy v0/v1 8-way values
        // (E=2/W=6 both landed on the catch-all) and could overflow on
        // garbage bytes (PR #348 review).
        let parsed = parse_direction(raw.direction);
        // Pipe-to-ground shares the flip (#364): game direction is the
        // surface-opening side, engine convention is the underground side.
        let dir = if raw.name.contains("inserter") || raw.name == "pipe-to-ground" {
            flip180(parsed)
        } else {
            parsed
        };
        let (w, h) = entity_footprint(&raw.name, dir);

        // Factorio stores center position; convert to top-left tile
        let x = (raw.position.x - w as f64 / 2.0).round() as i32;
        let y = (raw.position.y - h as f64 / 2.0).round() as i32;

        let items: Vec<crate::models::ModuleItem> = raw
            .items
            .0
            .into_iter()
            .map(|it| crate::models::ModuleItem {
                item: it.item,
                count: it.count,
                // Same permissiveness as entity quality: unknown (modded)
                // quality names fall back to None/normal.
                quality: it
                    .quality
                    .as_deref()
                    .and_then(crate::common::QualityTier::from_name),
            })
            .collect();

        let mut raw_filters = raw.filters;
        raw_filters.sort_by_key(|f| f.index);
        let filters: Vec<String> = raw_filters.into_iter().map(|f| f.name).collect();

        entities.push(PlacedEntity {
            name: raw.name,
            loop_priority_rate: None,
            x,
            y,
            direction: dir,
            recipe: raw.recipe,
            io_type: raw.io_type,
            carries: None,
            mirror: false,
            segment_id: None,
            rate: None,
            items,
            input_priority: None,
            output_priority: None,
            filters,
            quality: raw.quality.as_deref().and_then(crate::common::QualityTier::from_name),
        });
    }

    if entities.is_empty() {
        return LayoutResult::default();
    }

    // Compute bounding box and normalize to (0, 0)
    let min_x = entities.iter().map(|e| e.x).min().unwrap_or(0);
    let min_y = entities.iter().map(|e| e.y).min().unwrap_or(0);

    for e in &mut entities {
        e.x -= min_x;
        e.y -= min_y;
    }

    let max_x = entities
        .iter()
        .map(|e| {
            let (w, _) = entity_footprint(&e.name, e.direction);
            e.x + w - 1
        })
        .max()
        .unwrap_or(0);
    let max_y = entities
        .iter()
        .map(|e| {
            let (_, h) = entity_footprint(&e.name, e.direction);
            e.y + h - 1
        })
        .max()
        .unwrap_or(0);

    // Resolve the blueprint `wires` array into pole-copper index pairs for
    // `LayoutResult::power_wires`. Keep only copper (connector 5) edges whose
    // endpoints are both electric poles; normalize to `(lo, hi)`, sorted +
    // deduped so the result is deterministic regardless of source ordering.
    let copper = crate::power_wires::POLE_COPPER as i64;
    let mut power_wires: Vec<(u32, u32)> = Vec::new();
    for w in &wires_raw {
        if w.len() != 4 || w[1] != copper || w[3] != copper {
            continue;
        }
        let (Ok(a_num), Ok(b_num)) = (u64::try_from(w[0]), u64::try_from(w[2])) else {
            continue;
        };
        let (Some(&ia), Some(&ib)) = (num_to_idx.get(&a_num), num_to_idx.get(&b_num)) else {
            continue;
        };
        if ia == ib
            || !crate::power_wires::is_pole(&entities[ia].name)
            || !crate::power_wires::is_pole(&entities[ib].name)
        {
            continue;
        }
        let (lo, hi) = if ia < ib { (ia, ib) } else { (ib, ia) };
        power_wires.push((lo as u32, hi as u32));
    }
    power_wires.sort_unstable();
    power_wires.dedup();

    LayoutResult {
        entities,
        width: max_x + 1,
        height: max_y + 1,
        power_wires: Some(power_wires),
        ..Default::default()
    }
}

/// Recursively collect all blueprints from a book (which may contain nested books).
fn collect_blueprints(book: BpBook, results: &mut Vec<ParsedBlueprint>) {
    for entry in book.blueprints {
        if let Some(bp_data) = entry.blueprint {
            let label = bp_data.label.clone();
            results.push(ParsedBlueprint {
                label,
                layout: bp_data_to_layout(bp_data),
            });
        }
        if let Some(nested_book) = entry.blueprint_book {
            collect_blueprints(nested_book, results);
        }
    }
}

/// Parse a Factorio blueprint string into a `LayoutResult`.
///
/// The blueprint string must start with `'0'` (Factorio's version prefix).
/// Returns an error if the string is malformed or is a blueprint book.
///
/// Entity positions are normalized to start at (0, 0).
pub fn parse_blueprint_string(bp: &str) -> Result<LayoutResult, String> {
    let root = decode_bp_string(bp)?;

    if root.blueprint_book.is_some() {
        return Err(
            "Blueprint books are not supported — use parse_blueprint_string_any() instead".into(),
        );
    }

    let bp_data = root
        .blueprint
        .ok_or("not a blueprint (missing 'blueprint' key)")?;

    Ok(bp_data_to_layout(bp_data))
}

/// Parse a blueprint string that may be a single blueprint or a blueprint book.
///
/// Returns one or more `ParsedBlueprint`s. Books are flattened recursively.
pub fn parse_blueprint_string_any(bp: &str) -> Result<Vec<ParsedBlueprint>, String> {
    let root = decode_bp_string(bp)?;

    if let Some(book) = root.blueprint_book {
        let mut results = Vec::new();
        collect_blueprints(book, &mut results);
        if results.is_empty() {
            return Err("blueprint book contains no blueprints".into());
        }
        Ok(results)
    } else if let Some(bp_data) = root.blueprint {
        let label = bp_data.label.clone();
        Ok(vec![ParsedBlueprint {
            label,
            layout: bp_data_to_layout(bp_data),
        }])
    } else {
        Err("not a blueprint or blueprint book".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint;
    use crate::models::{EntityDirection, PlacedEntity};

    /// Encode raw blueprint JSON into the `"0" + base64(zlib(JSON))`
    /// envelope. Synthetic-fixture path for format fields our own export
    /// doesn't emit yet (e.g. `quality` before rfc-build-quality Phase 2)
    /// — parsing must not be blocked on finding a community artifact.
    fn encode_envelope(json: &serde_json::Value) -> String {
        use base64::Engine;
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(json.to_string().as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        format!(
            "0{}",
            base64::engine::general_purpose::STANDARD.encode(&compressed)
        )
    }

    /// RFC-045 round-trip: a Tree-mode wire graph survives export → parse
    /// verbatim (the parser reads the artifact's wires array; it never
    /// recomputes), and re-export is byte-identical.
    #[test]
    fn tree_wires_round_trip_through_export_and_parse() {
        use crate::power_wires::{compute_pole_wires, WireMode};
        let ents: Vec<PlacedEntity> = (0..4)
            .map(|i| PlacedEntity {
                name: "medium-electric-pole".to_string(),
                x: i * 7,
                y: 0,
                ..Default::default()
            })
            .collect();
        let mut layout = LayoutResult::default();
        layout.width = 25;
        layout.height = 1;
        layout.entities = ents;
        layout.wire_mode = WireMode::Tree;
        layout.power_wires = Some(compute_pole_wires(&layout.entities, WireMode::Tree));
        let tree = layout.power_wires.clone().unwrap();
        assert_eq!(tree.len(), 3);

        let bp = blueprint::export(&layout, "tree-mode");
        let parsed = parse_blueprint_string(&bp).expect("should parse");
        assert_eq!(parsed.power_wires.as_deref(), Some(tree.as_slice()));
        // Re-export from the parsed layout: same wires bytes (parser stored
        // Some, so export consumes it verbatim — no re-densify).
        let bp2 = blueprint::export(&parsed, "tree-mode");
        assert_eq!(bp, bp2, "tree export must be a fixed point of export→parse→export");
    }

    /// RFC-046 Phase 0 spot-check: the 2.0 `override_stack_size` field
    /// (uint8, per-inserter hand-size override; lua-api BlueprintEntity)
    /// is tolerated on import. We deliberately never emit it — exports
    /// inherit the importing force's research — but community blueprints
    /// carry it and must parse.
    #[test]
    fn override_stack_size_field_is_tolerated_on_import() {
        let bp = encode_envelope(&serde_json::json!({
            "blueprint": {
                "item": "blueprint",
                "version": 562949955518464u64,
                "entities": [
                    {"entity_number": 1, "name": "stack-inserter",
                     "position": {"x": 0.5, "y": 0.5}, "direction": 4,
                     "override_stack_size": 1},
                ]
            }
        }));
        let parsed = parse_blueprint_string(&bp).expect("should parse");
        assert_eq!(parsed.entities.len(), 1);
        assert_eq!(parsed.entities[0].name, "stack-inserter");
    }

    /// PR #348 review MAJOR: the pickup→drop un-flip must happen in
    /// EntityDirection space, after legacy 8-way decoding — raw-byte
    /// `+8` collapsed legacy E(2)/W(6) onto the catch-all.
    #[test]
    fn legacy_8way_inserters_unflip_correctly() {
        let bp = encode_envelope(&serde_json::json!({
            "blueprint": {
                "item": "blueprint",
                "entities": [
                    // Legacy East (2): game pickup=east → engine drop-side = West.
                    {"entity_number": 1, "name": "fast-inserter",
                     "position": {"x": 0.5, "y": 0.5}, "direction": 2},
                    // Legacy West (6) → engine East.
                    {"entity_number": 2, "name": "inserter",
                     "position": {"x": 2.5, "y": 0.5}, "direction": 6},
                    // Legacy East belt stays East (no flip for non-inserters).
                    {"entity_number": 3, "name": "transport-belt",
                     "position": {"x": 4.5, "y": 0.5}, "direction": 2},
                ]
            }
        }));
        let parsed = parse_blueprint_string(&bp).expect("should parse");
        assert_eq!(parsed.entities[0].direction, EntityDirection::West);
        assert_eq!(parsed.entities[1].direction, EntityDirection::East);
        assert_eq!(parsed.entities[2].direction, EntityDirection::East);
    }

    /// Pipe-to-ground un-flips on import (#364): a game pair faces AWAY
    /// from each other (surface openings outward); engine convention is
    /// the underground side, so both flip 180° — and plain pipes don't.
    #[test]
    fn pipe_to_ground_unflips_on_import() {
        let bp = encode_envelope(&serde_json::json!({
            "blueprint": {
                "item": "blueprint",
                "entities": [
                    // Game North (surface up) → engine South (tunnel down).
                    {"entity_number": 1, "name": "pipe-to-ground",
                     "position": {"x": 0.5, "y": 0.5}, "direction": 0},
                    // Game South (surface down) → engine North.
                    {"entity_number": 2, "name": "pipe-to-ground",
                     "position": {"x": 0.5, "y": 2.5}, "direction": 8},
                    {"entity_number": 3, "name": "pipe",
                     "position": {"x": 2.5, "y": 0.5}, "direction": 0},
                ]
            }
        }));
        let parsed = parse_blueprint_string(&bp).expect("should parse");
        assert_eq!(parsed.entities[0].direction, EntityDirection::South);
        assert_eq!(parsed.entities[1].direction, EntityDirection::North);
        assert_eq!(parsed.entities[2].direction, EntityDirection::North);
    }

    /// PR #348 review MINOR: garbage direction bytes must not panic
    /// (the old raw `+8` overflowed u8 for values ≥ 248 in debug).
    #[test]
    fn garbage_direction_bytes_do_not_panic() {
        let bp = encode_envelope(&serde_json::json!({
            "blueprint": {
                "item": "blueprint",
                "entities": [
                    {"entity_number": 1, "name": "inserter",
                     "position": {"x": 0.5, "y": 0.5}, "direction": 250},
                ]
            }
        }));
        let parsed = parse_blueprint_string(&bp).expect("should parse");
        // Catch-all North, flipped to engine South.
        assert_eq!(parsed.entities[0].direction, EntityDirection::South);
    }

    #[test]
    fn parses_entity_quality() {
        use crate::common::QualityTier;
        // Per lua-api BlueprintEntity: `quality :: string?`, sibling to
        // name/position; absent means normal.
        let bp = encode_envelope(&serde_json::json!({
            "blueprint": {
                "item": "blueprint",
                "version": 562949955518464u64,
                "entities": [
                    {"entity_number": 1, "name": "assembling-machine-3",
                     "position": {"x": 1.5, "y": 1.5}, "quality": "legendary"},
                    {"entity_number": 2, "name": "fast-inserter",
                     "position": {"x": 3.5, "y": 0.5}, "quality": "rare"},
                    {"entity_number": 3, "name": "transport-belt",
                     "position": {"x": 4.5, "y": 0.5}},
                    {"entity_number": 4, "name": "medium-electric-pole",
                     "position": {"x": 4.5, "y": 1.5}, "quality": "modded-mythic"},
                    {"entity_number": 5, "name": "small-electric-pole",
                     "position": {"x": 4.5, "y": 2.5}, "quality": 5},
                    {"entity_number": 6, "name": "long-handed-inserter",
                     "position": {"x": 4.5, "y": 3.5}, "quality": {"name": "legendary"}},
                ]
            }
        }));

        let parsed = parse_blueprint_string(&bp).expect("should parse");
        let by_name = |n: &str| {
            parsed
                .entities
                .iter()
                .find(|e| e.name == n)
                .unwrap_or_else(|| panic!("missing {n}"))
        };

        assert_eq!(by_name("assembling-machine-3").quality, Some(QualityTier::Legendary));
        assert_eq!(by_name("fast-inserter").quality, Some(QualityTier::Rare));
        // Absent field = normal = None.
        assert_eq!(by_name("transport-belt").quality, None);
        // Modded quality names degrade to None (vanilla-only fidelity).
        assert_eq!(by_name("medium-electric-pole").quality, None);
        // Non-string quality values (corrupted/modded JSON) degrade to
        // None instead of failing the whole parse — pre-Phase-0, this key
        // was an ignored unknown field, and a malformed entity must not
        // nuke an otherwise-parseable blueprint (analyzer robustness).
        assert_eq!(by_name("small-electric-pole").quality, None);
        assert_eq!(by_name("long-handed-inserter").quality, None);
    }

    #[test]
    fn round_trip_simple() {
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-2".into(),
                    x: 0,
                    y: 0,
                    direction: EntityDirection::North,
                    recipe: Some("iron-gear-wheel".into()),
                    ..Default::default()
                },
                PlacedEntity {
                    name: "transport-belt".into(),
                    x: 3,
                    y: 1,
                    direction: EntityDirection::East,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "underground-belt".into(),
                    x: 4,
                    y: 1,
                    direction: EntityDirection::East,
                    io_type: Some("input".into()),
                    ..Default::default()
                },
            ],
            width: 5,
            height: 3,
            ..Default::default()
        };

        let bp_string = blueprint::export(&layout, "test");
        let parsed = parse_blueprint_string(&bp_string).expect("should parse");

        // After round-trip, entities should be at the same positions
        // (origin may shift if export uses center positions for multi-tile)
        assert_eq!(parsed.entities.len(), 3);

        // Find the assembling machine
        let machine = parsed
            .entities
            .iter()
            .find(|e| e.name == "assembling-machine-2")
            .expect("should have assembling machine");
        assert_eq!(machine.recipe.as_deref(), Some("iron-gear-wheel"));

        // Find the underground belt
        let ug = parsed
            .entities
            .iter()
            .find(|e| e.name == "underground-belt")
            .expect("should have underground belt");
        assert_eq!(ug.io_type.as_deref(), Some("input"));
        assert!(matches!(ug.direction, EntityDirection::East));
    }

    #[test]
    fn pole_wires_round_trip_simple() {
        // Three medium poles in a line at pitch 7 (≤ reach 9): adjacent pairs
        // wire, the ends do not (d=14 > 9), but all three are one network.
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity { name: "medium-electric-pole".into(), x: 0, y: 0, ..Default::default() },
                PlacedEntity { name: "medium-electric-pole".into(), x: 7, y: 0, ..Default::default() },
                PlacedEntity { name: "medium-electric-pole".into(), x: 14, y: 0, ..Default::default() },
            ],
            width: 15,
            height: 1,
            ..Default::default()
        };
        let emitted = crate::power_wires::compute_pole_wires(&layout.entities, crate::power_wires::WireMode::Dense);
        assert_eq!(emitted, vec![(0, 1), (1, 2)]);

        let bp_string = blueprint::export(&layout, "pole-wires");
        let parsed = parse_blueprint_string(&bp_string).expect("should parse");
        // Wires must survive export → parse (before the fix: empty).
        assert_eq!(parsed.power_wires.as_deref(), Some(emitted.as_slice()), "power_wires must round-trip");
        assert_eq!(
            crate::power_wires::count_disconnected_poles(&parsed.entities, parsed.power_wires.as_deref().unwrap_or(&[])),
            0,
            "all three poles are one network after round-trip"
        );
    }

    #[test]
    fn pole_wires_round_trip_dense_grid() {
        // A 5×5 grid of medium poles at pitch 6 (both axes) — the dense,
        // mutually-overlapping field a real bus produces. Every pole reaches
        // its 4- and 8-neighbours (pitch 6 and 6√2≈8.49, both ≤ 9), so the
        // whole 25-pole field must be one connected copper network, and the
        // exact wire set must round-trip through export → parse.
        let mut entities = Vec::new();
        for gy in 0..5 {
            for gx in 0..5 {
                entities.push(PlacedEntity {
                    name: "medium-electric-pole".into(),
                    x: gx * 6,
                    y: gy * 6,
                    ..Default::default()
                });
            }
        }
        let layout = LayoutResult { entities, width: 25, height: 25, ..Default::default() };

        let emitted = crate::power_wires::compute_pole_wires(&layout.entities, crate::power_wires::WireMode::Dense);
        assert!(!emitted.is_empty(), "dense grid must wire");
        assert_eq!(
            crate::power_wires::count_disconnected_poles(&layout.entities, &emitted),
            0,
            "all 25 poles must be one network"
        );

        let bp_string = blueprint::export(&layout, "dense-grid");
        let parsed = parse_blueprint_string(&bp_string).expect("should parse");
        assert_eq!(parsed.power_wires.as_deref(), Some(emitted.as_slice()), "dense wire set must round-trip");
        assert_eq!(
            crate::power_wires::count_disconnected_poles(&parsed.entities, parsed.power_wires.as_deref().unwrap_or(&[])),
            0,
            "the 25-pole network stays connected after round-trip"
        );
    }

    #[test]
    fn filtered_inserter_round_trips() {
        let layout = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "bulk-inserter".into(),
                    x: 0,
                    y: 0,
                    direction: EntityDirection::North,
                    filters: vec!["iron-plate".into(), "copper-plate".into()],
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 2,
                    y: 0,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 3,
            height: 1,
            ..Default::default()
        };

        let bp_string = blueprint::export(&layout, "filter_test");

        // Byte-level check: the filtered inserter emits use_filters/filters,
        // the unfiltered one emits neither field.
        let b64 = &bp_string[1..];
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        std::io::Read::read_to_string(&mut decoder, &mut json_str).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let ents = parsed["blueprint"]["entities"].as_array().unwrap();
        let filtered_json = ents
            .iter()
            .find(|e| e["name"] == "bulk-inserter")
            .expect("bulk-inserter entity");
        assert_eq!(filtered_json["use_filters"], true);
        assert_eq!(
            filtered_json["filters"],
            serde_json::json!([
                {"index": 1, "name": "iron-plate"},
                {"index": 2, "name": "copper-plate"},
            ])
        );
        let plain_json = ents
            .iter()
            .find(|e| e["name"] == "inserter")
            .expect("plain inserter entity");
        assert!(plain_json.get("use_filters").is_none());
        assert!(plain_json.get("filters").is_none());

        // Full round trip: export -> parse -> filters survive byte-for-byte.
        let parsed_layout = parse_blueprint_string(&bp_string).expect("should parse");
        let filtered = parsed_layout
            .entities
            .iter()
            .find(|e| e.name == "bulk-inserter")
            .expect("bulk-inserter entity");
        assert_eq!(
            filtered.filters,
            vec!["iron-plate".to_string(), "copper-plate".to_string()]
        );
        let plain = parsed_layout
            .entities
            .iter()
            .find(|e| e.name == "inserter")
            .expect("plain inserter entity");
        assert!(plain.filters.is_empty());
    }

    #[test]
    fn rejects_non_blueprint() {
        assert!(parse_blueprint_string("1invalidstring").is_err());
        assert!(parse_blueprint_string("").is_err());
    }

    #[test]
    fn parses_entity_items() {
        // Manually construct a minimal blueprint JSON with items
        let bp_json = serde_json::json!({
            "blueprint": {
                "entities": [
                    {
                        "entity_number": 1,
                        "name": "beacon",
                        "position": {"x": 0.5, "y": 0.5},
                        "items": [
                            {"item": "speed-module-3", "count": 2}
                        ]
                    },
                    {
                        "entity_number": 2,
                        "name": "assembling-machine-3",
                        "position": {"x": 4.5, "y": 0.5},
                        "recipe": "iron-gear-wheel",
                        "items": [
                            {"item": "productivity-module-3", "count": 4}
                        ]
                    }
                ],
                "item": "blueprint",
                "version": 562949954076673u64
            }
        });

        // Encode as blueprint string: "0" + base64(zlib(json))
        let json_bytes = serde_json::to_vec(&bp_json).unwrap();
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, &json_bytes).unwrap();
        let compressed = encoder.finish().unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&compressed);
        let bp_string = format!("0{}", b64);

        let layout = parse_blueprint_string(&bp_string).unwrap();
        assert_eq!(layout.entities.len(), 2);

        let beacon = layout.entities.iter().find(|e| e.name == "beacon").unwrap();
        assert_eq!(beacon.items.len(), 1);
        assert_eq!(beacon.items[0].item, "speed-module-3");
        assert_eq!(beacon.items[0].count, 2);

        let machine = layout
            .entities
            .iter()
            .find(|e| e.name == "assembling-machine-3")
            .unwrap();
        assert_eq!(machine.items.len(), 1);
        assert_eq!(machine.items[0].item, "productivity-module-3");
        assert_eq!(machine.items[0].count, 4);
    }

    #[test]
    fn parses_insert_plan_items_with_quality() {
        // 2.0 insert-plan format (BlueprintInsertPlan): count comes from
        // the in_inventory array length, quality from id.quality. Also
        // covers the bare-string id variant and explicit "normal".
        use base64::Engine;
        let bp_json = serde_json::json!({
            "blueprint": {
                "entities": [
                    {
                        "entity_number": 1,
                        "name": "assembling-machine-3",
                        "position": {"x": 1.5, "y": 1.5},
                        "recipe": "iron-gear-wheel",
                        "items": [
                            {
                                "id": {"name": "productivity-module-3", "quality": "legendary"},
                                "items": {"in_inventory": [
                                    {"inventory": 4, "stack": 0},
                                    {"inventory": 4, "stack": 1}
                                ]}
                            },
                            {
                                "id": {"name": "speed-module-2", "quality": "normal"},
                                "items": {"in_inventory": [{"inventory": 4, "stack": 2}]}
                            },
                            {
                                "id": "speed-module",
                                "items": {"in_inventory": [{"inventory": 4, "stack": 3}]}
                            }
                        ],
                        "item": "blueprint"
                    }
                ],
                "item": "blueprint",
                "version": 562949954076673u64
            }
        });

        let json_bytes = serde_json::to_vec(&bp_json).unwrap();
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, &json_bytes).unwrap();
        let compressed = encoder.finish().unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&compressed);
        let layout = parse_blueprint_string(&format!("0{}", b64)).unwrap();

        let machine = &layout.entities[0];
        assert_eq!(machine.items.len(), 3);

        assert_eq!(machine.items[0].item, "productivity-module-3");
        assert_eq!(machine.items[0].count, 2);
        assert_eq!(
            machine.items[0].quality,
            Some(crate::common::QualityTier::Legendary)
        );

        assert_eq!(machine.items[1].item, "speed-module-2");
        assert_eq!(machine.items[1].count, 1);
        assert_eq!(
            machine.items[1].quality,
            Some(crate::common::QualityTier::Normal)
        );

        // Bare-string id: no quality information.
        assert_eq!(machine.items[2].item, "speed-module");
        assert_eq!(machine.items[2].count, 1);
        assert_eq!(machine.items[2].quality, None);
    }
}
