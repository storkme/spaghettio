//! Blueprint ingestion — the meter's own decode of the exported artifact.
//!
//! # Why this does not use `spaghettio_core::blueprint_parser`
//!
//! Because inserter direction is an *interpretation*, and it is the one
//! this project has already got catastrophically wrong.
//!
//! Factorio reads an inserter's `direction` as its **pickup** side
//! ("inserters point backwards"). The engine's internal convention is the
//! **drop** side. `blueprint.rs` flips on export; `blueprint_parser` flips
//! back on import. Both flips are believed correct — but RFC-050's
//! motivation records what happened when a shared convention was wrong:
//! every blueprint the project had ever exported had all inserters running
//! backwards in-game, invisible to the validator, the renderer *and* the
//! mechanics doc, because all three shared the engine's convention. Three
//! agreeing artifacts were secretly one assumption.
//!
//! A meter that imports the engine's un-flip inherits that assumption. So
//! this module decodes the blueprint envelope itself and applies
//! **Factorio's** rule directly: `direction` is where the hand picks up,
//! and the drop tile is opposite. If the engine's export flip is ever
//! wrong again, the meter disagrees with the engine instead of agreeing
//! with it — which is the entire point of the instrument.
//!
//! This is the same reasoning as KC4, applied to a convention rather than
//! a constant. Cheap to honour: the envelope is base64 + zlib + JSON.

use std::io::Read;

use base64::Engine as _;
use serde::Deserialize;

/// A 4-way facing, as stored in the blueprint (`0/4/8/12`).
///
/// Factorio 2.0 also uses 16-way values for some entities; the engine only
/// ever emits the four cardinals, and anything else is refused loudly
/// rather than rounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    pub fn from_blueprint(raw: u16) -> Result<Self, String> {
        match raw {
            0 => Ok(Dir::North),
            4 => Ok(Dir::East),
            8 => Ok(Dir::South),
            12 => Ok(Dir::West),
            other => Err(format!(
                "unsupported blueprint direction {other} (meter handles the four \
                 cardinals the engine emits; 16-way rotations are not modelled)"
            )),
        }
    }

    /// Unit step in this direction, in tile coordinates (y grows south).
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::North => (0, -1),
            Dir::East => (1, 0),
            Dir::South => (0, 1),
            Dir::West => (-1, 0),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::East => Dir::West,
            Dir::West => Dir::East,
        }
    }

    /// True when the two facings lie on the same axis.
    pub fn same_axis(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Dir::North | Dir::South, Dir::North | Dir::South)
                | (Dir::East | Dir::West, Dir::East | Dir::West)
        )
    }
}

/// One entity as it appears in the blueprint, in tile coordinates.
#[derive(Debug, Clone)]
pub struct RawEntity {
    pub name: String,
    /// Top-left tile of the entity's footprint.
    pub x: i32,
    pub y: i32,
    pub direction: Dir,
    pub recipe: Option<String>,
    /// Underground-belt half: `"input"` (entrance) or `"output"` (exit).
    pub io_type: Option<String>,
    /// Blueprint `mirror` flag (Factorio 2.0, fluid-box machines). Absent
    /// means false ON THE WIRE — but note the engine exporter never emits
    /// it for oil-refinery/foundry/cryogenic-plant (their mirror is
    /// encoded as a tile-identical 180° rotation instead), so absence is
    /// NOT proof of unmirroredness for those three; `factory.rs` keeps
    /// its name heuristic for them. Parsed 2026-08-21 (offpath B2) so an
    /// EXPLICIT community `mirror: true` is honored on any machine.
    pub mirror: bool,
}

impl RawEntity {
    /// Where an **inserter** picks up from, applying Factorio's rule.
    ///
    /// `direction` is the pickup side. Reach is 1 for every inserter except
    /// long-handed, which is 2 (mechanics I3/I4/I8a).
    pub fn inserter_pickup_tile(&self, reach: i32) -> (i32, i32) {
        let (dx, dy) = self.direction.delta();
        (self.x + dx * reach, self.y + dy * reach)
    }

    /// Where an inserter drops — the opposite side, same reach.
    pub fn inserter_drop_tile(&self, reach: i32) -> (i32, i32) {
        let (dx, dy) = self.direction.opposite().delta();
        (self.x + dx * reach, self.y + dy * reach)
    }
}

#[derive(Deserialize)]
struct Envelope {
    blueprint: Blueprint,
}

#[derive(Deserialize)]
struct Blueprint {
    #[serde(default)]
    entities: Vec<Entity>,
}

#[derive(Deserialize)]
struct Entity {
    name: String,
    position: Position,
    #[serde(default)]
    direction: u16,
    #[serde(default)]
    recipe: Option<String>,
    #[serde(default, rename = "type")]
    io_type: Option<String>,
    #[serde(default)]
    mirror: bool,
}

#[derive(Deserialize)]
struct Position {
    x: f64,
    y: f64,
}

/// Decode a Factorio 2.0 blueprint string into entities in tile coordinates.
///
/// The envelope is `'0'` + base64(zlib(JSON)). Positions are entity
/// *centres* in Factorio's continuous coordinates: a 1×1 entity sits at
/// `x.5`, a 3×3 at an integer. Converting to the top-left tile is
/// `floor(x - w/2)`, which is why footprint width has to be known here.
pub fn decode(bp: &str) -> Result<Vec<RawEntity>, String> {
    let bp = bp.trim();
    let body = bp
        .strip_prefix('0')
        .ok_or_else(|| "blueprint string does not start with the version byte '0'".to_string())?;
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(body)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    let mut json = String::new();
    flate2::read::ZlibDecoder::new(&compressed[..])
        .read_to_string(&mut json)
        .map_err(|e| format!("zlib decompress failed: {e}"))?;
    let env: Envelope =
        serde_json::from_str(&json).map_err(|e| format!("blueprint JSON parse failed: {e}"))?;

    let mut out = Vec::with_capacity(env.blueprint.entities.len());
    let mut unknown: Vec<String> = Vec::new();
    for e in env.blueprint.entities {
        if crate::entity_data::footprint_checked(&e.name).is_none() {
            if !unknown.contains(&e.name) {
                unknown.push(e.name.clone());
            }
            continue;
        }
        let dir = Dir::from_blueprint(e.direction)?;
        let horizontal = matches!(dir, Dir::East | Dir::West);
        let (w, h) = crate::entity_data::footprint_oriented(&e.name, horizontal);
        // Centre -> top-left tile.
        let x = (e.position.x - w as f64 / 2.0).round() as i32;
        let y = (e.position.y - h as f64 / 2.0).round() as i32;
        out.push(RawEntity {
            name: e.name,
            x,
            y,
            direction: dir,
            recipe: e.recipe,
            io_type: e.io_type,
            mirror: e.mirror,
        });
    }
    if !unknown.is_empty() {
        // Loud, not silent. An entity the meter cannot place is an entity
        // whose adjacency it would get wrong.
        return Err(format!(
            "blueprint contains {} entity type(s) the meter does not model: {}. \
             Add footprints in entity_data.rs rather than letting them default — \
             a wrong footprint corrupts every adjacency downstream.",
            unknown.len(),
            unknown.join(", ")
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_deltas_and_opposites() {
        assert_eq!(Dir::North.delta(), (0, -1));
        assert_eq!(Dir::South.delta(), (0, 1));
        assert_eq!(Dir::East.delta(), (1, 0));
        assert_eq!(Dir::West.delta(), (-1, 0));
        for d in [Dir::North, Dir::East, Dir::South, Dir::West] {
            assert_eq!(d.opposite().opposite(), d);
            assert!(d.same_axis(d.opposite()));
        }
        assert!(!Dir::North.same_axis(Dir::East));
    }

    /// The convention this module exists to own: `direction` is the PICKUP
    /// side, the drop is opposite. Getting this backwards is the #348 bug.
    #[test]
    fn inserter_direction_is_the_pickup_side() {
        let ins = RawEntity {
            name: "fast-inserter".into(),
            x: 10,
            y: 10,
            direction: Dir::North,
            recipe: None,
            io_type: None,
            mirror: false,
        };
        // Facing north => picks from the tile to the north, drops south.
        assert_eq!(ins.inserter_pickup_tile(1), (10, 9));
        assert_eq!(ins.inserter_drop_tile(1), (10, 11));
        // Long-handed spans two tiles on each side.
        assert_eq!(ins.inserter_pickup_tile(2), (10, 8));
        assert_eq!(ins.inserter_drop_tile(2), (10, 12));
    }

    #[test]
    fn rejects_non_cardinal_directions() {
        assert!(Dir::from_blueprint(2).is_err());
        assert!(Dir::from_blueprint(0).is_ok());
    }

    #[test]
    fn rejects_a_malformed_envelope() {
        assert!(decode("not-a-blueprint").is_err());
        assert!(decode("0!!!!").is_err());
    }
}
