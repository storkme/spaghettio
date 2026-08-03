//! Factorio prototype constants the meter needs.
//!
//! **Provenance discipline (RFC-054).** Everything here is a *game
//! constant* — a fact about Factorio prototypes — never a derived rate.
//! The meter deliberately keeps its own copy rather than importing
//! `spaghettio_core::common`, because that module mixes prototype facts
//! with hand-calibrated estimates (`machine_feed_rate`, `belt_drop_rate`,
//! lane capacities, utilization factors) and the meter must not inherit
//! the engine's beliefs. See the crate manifest and `tests/kc4_independence.rs`.
//!
//! Each constant below carries its derivation so a reviewer can check it
//! against the game rather than against us.

/// Item spacing on a belt lane, in tiles. Factorio packs items on a
/// transport line every 0.25 tiles, so a 1×1 belt tile holds 4 items per
/// lane when fully compressed.
pub const ITEM_SPACING_TILES: f64 = 0.25;

/// Slots per lane per belt tile. `1.0 / ITEM_SPACING_TILES`.
pub const SLOTS_PER_TILE: usize = 4;

/// Ticks per second.
pub const TICKS_PER_SECOND: f64 = 60.0;

/// Unoriented footprint (width, height) in tiles, for the entities the
/// meter simulates.
///
/// Splitters are the one direction-dependent case (2 wide perpendicular to
/// facing, 1 deep along it — mechanics **S1**); see [`footprint_oriented`].
///
/// **Only call this once the name is known.** It falls back to 1×1, which
/// is a lie for anything bigger — use [`footprint_checked`] to establish
/// the name is modelled, then this for convenience. Both current callers
/// do exactly that: `blueprint_in::decode` rejects unknown names up front,
/// and `factory.rs` calls this behind an `is_crafting_machine` gate.
///
/// The doc used to claim, in bold, that unknown names were an error here —
/// describing `footprint_checked`'s behaviour, not this function's. Nothing
/// depended on the false version, but a future caller trusting it would
/// have got the silent wrong-tile placement it warned about.
pub fn footprint(name: &str) -> (u32, u32) {
    footprint_checked(name).unwrap_or((1, 1))
}

/// Footprint, or `None` for an entity the meter does not know.
///
/// **Unknown names are an error, not a 1×1 default.** A wrong footprint
/// puts an entity on the wrong tile, which silently corrupts every
/// adjacency the simulation depends on — the meter would then measure a
/// factory that isn't the one in the blueprint. This is the same argument
/// as the audit's GAP 2 (unknown machines solving silently at speed 1.0):
/// a loud refusal beats a plausible wrong number.
pub fn footprint_checked(name: &str) -> Option<(u32, u32)> {
    let f = match name {
        // 1x1: belts, undergrounds, inserters, poles, pipes.
        n if n.ends_with("transport-belt") || n.ends_with("underground-belt") => (1, 1),
        n if n.ends_with("inserter") => (1, 1),
        "medium-electric-pole" | "small-electric-pole" | "big-electric-pole" | "substation" => {
            match name {
                "big-electric-pole" => (2, 2),
                "substation" => (2, 2),
                _ => (1, 1),
            }
        }
        "pipe" | "pipe-to-ground" => (1, 1),
        // Splitters: 2x1 unoriented; orientation applied by the caller.
        n if n.ends_with("splitter") => (2, 1),
        // Crafting machines.
        "assembling-machine-1" | "assembling-machine-2" | "assembling-machine-3" => (3, 3),
        "electric-furnace"
        | "chemical-plant"
        | "centrifuge"
        | "electromagnetic-plant"
        | "biochamber" => (3, 3),
        "stone-furnace" | "steel-furnace" => (2, 2),
        "oil-refinery" | "foundry" | "cryogenic-plant" => (5, 5),
        "recycler" => (2, 3),
        // Containers used by boundary scaffolding.
        "wooden-chest" | "iron-chest" | "steel-chest" | "infinity-chest" => (1, 1),
        _ => return None,
    };
    Some(f)
}

/// Footprint with splitter orientation applied.
pub fn footprint_oriented(name: &str, horizontal_axis: bool) -> (u32, u32) {
    let (w, h) = footprint(name);
    if name.ends_with("splitter") && horizontal_axis {
        // Facing east/west: 1 wide, 2 tall.
        (h, w)
    } else {
        (w, h)
    }
}

/// True when this entity crafts a recipe (and therefore needs ingredient
/// buffers and a craft timer).
pub fn is_crafting_machine(name: &str) -> bool {
    matches!(
        name,
        "assembling-machine-1"
            | "assembling-machine-2"
            | "assembling-machine-3"
            | "electric-furnace"
            | "stone-furnace"
            | "steel-furnace"
            | "chemical-plant"
            | "oil-refinery"
            | "centrifuge"
            | "electromagnetic-plant"
            | "biochamber"
            | "foundry"
            | "cryogenic-plant"
            | "recycler"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeltTier {
    Yellow,
    Red,
    Blue,
    Turbo,
}

impl BeltTier {
    pub fn from_entity_name(name: &str) -> Option<Self> {
        // Underground/splitter variants share their tier's speed.
        let base = name
            .trim_end_matches("-transport-belt")
            .trim_end_matches("-underground-belt")
            .trim_end_matches("-splitter");
        match name {
            "transport-belt" | "underground-belt" | "splitter" => Some(BeltTier::Yellow),
            _ => match base {
                "fast" => Some(BeltTier::Red),
                "express" => Some(BeltTier::Blue),
                "turbo" => Some(BeltTier::Turbo),
                _ => None,
            },
        }
    }

    /// Belt speed in tiles per second.
    ///
    /// Cross-check against `factorio-mechanics.md` **B5** (throughput per
    /// belt, both lanes): `tiles/s × 4 slots/tile × 2 lanes`.
    ///   yellow 1.875 × 4 × 2 = 15/s ✓
    ///   red    3.75  × 4 × 2 = 30/s ✓
    ///   blue   5.625 × 4 × 2 = 45/s ✓
    ///   turbo  7.5   × 4 × 2 = 60/s ✓ (Space Age)
    ///
    /// Note the meter derives throughput from speed and spacing; it never
    /// reads a throughput table. That is the whole point — a belt's
    /// nominal rate is an *output* of the physics here, not an input.
    pub fn tiles_per_second(self) -> f64 {
        match self {
            BeltTier::Yellow => 1.875,
            BeltTier::Red => 3.75,
            BeltTier::Blue => 5.625,
            BeltTier::Turbo => 7.5,
        }
    }

    /// Slot advances per tick for one item on a free-flowing belt.
    pub fn slots_per_tick(self) -> f64 {
        self.tiles_per_second() / TICKS_PER_SECOND / ITEM_SPACING_TILES
    }
}

/// The direction a fluid moves through a port (relative to the emitting
/// recipe): a machine's `Input` ports feed its fluid ingredients; `Output`
/// ports carry its fluid products.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortIO {
    Input,
    Output,
}

/// A fluid port: `(dx, dy, io)` relative to the machine's top-left footprint
/// tile, for a North-facing (direction 0, unmirrored) machine.
pub type BaseFluidPort = (i32, i32, PortIO);

/// Base (North-facing) fluid-port geometry, a **game constant** (the Factorio
/// prototype `fluid_boxes`), replicated here under the same provenance
/// discipline as the footprints and belt tiers so the meter never inherits the
/// engine's derived fluid model (KC4). Orientation transforms (rotation by
/// `direction`) are applied by the caller.
///
/// Canonical tables are the draftsman-verified ones from
/// `spaghettio_core::fluid_ports` (same base values; the port *fluid* binding
/// is recipe-dependent and applied in `factory`), kept here so the meter has
/// one self-contained game-constants module. Only direction 0 (North) and 8
/// (South) occur in the corpus; the full cardinal rotation is provided anyway.
pub fn base_fluid_ports(name: &str) -> &'static [BaseFluidPort] {
    match name {
        "assembling-machine-2" | "assembling-machine-3" => {
            &[(1, -1, PortIO::Input), (1, 3, PortIO::Output)]
        }
        "chemical-plant" | "biochamber" => &[
            (0, -1, PortIO::Input),
            (2, -1, PortIO::Input),
            (0, 3, PortIO::Output),
            (2, 3, PortIO::Output),
        ],
        "oil-refinery" => &[
            (1, 5, PortIO::Input),
            (3, 5, PortIO::Input),
            (0, -1, PortIO::Output),
            (2, -1, PortIO::Output),
            (4, -1, PortIO::Output),
        ],
        "foundry" => &[
            (1, 5, PortIO::Input),
            (3, 5, PortIO::Input),
            (1, -1, PortIO::Output),
            (3, -1, PortIO::Output),
        ],
        "cryogenic-plant" => &[
            (0, 5, PortIO::Input),
            (2, 5, PortIO::Input),
            (4, 5, PortIO::Input),
            (0, -1, PortIO::Output),
            (2, -1, PortIO::Output),
            (4, -1, PortIO::Output),
        ],
        "electromagnetic-plant" => &[
            (-1, 2, PortIO::Input),
            (4, 1, PortIO::Input),
            (2, 4, PortIO::Output),
            (1, -1, PortIO::Output),
        ],
        _ => &[],
    }
}

/// Rotate a North-facing base port offset for a `w`×`w` footprint placed at
/// `dir`. Port offsets may lie one tile outside the footprint (dy == -1 or
/// dy == h), so we rotate the offset-vector about the footprint centre.
pub fn rotate_port(dir: crate::blueprint_in::Dir, dx: i32, dy: i32, w: i32) -> (i32, i32) {
    let c = (w - 1) / 2;
    let (vx, vy) = (dx - c, dy - c);
    // Each step rotates the offset-vector +90°, up->right (North->East), via
    // (vx, vy) -> (-vy, vx). North=0, East=1, South=2, West=3 steps.
    let (nvx, nvy) = match dir {
        crate::blueprint_in::Dir::North => (vx, vy),
        crate::blueprint_in::Dir::East => (-vy, vx),
        crate::blueprint_in::Dir::South => (-vx, -vy),
        crate::blueprint_in::Dir::West => (vy, -vx),
    };
    (c + nvx, c + nvy)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InserterKind {
    Regular,
    LongHanded,
    Fast,
    Bulk,
    Stack,
}

impl InserterKind {
    pub fn from_entity_name(name: &str) -> Option<Self> {
        match name {
            "inserter" => Some(InserterKind::Regular),
            "long-handed-inserter" => Some(InserterKind::LongHanded),
            "fast-inserter" => Some(InserterKind::Fast),
            "bulk-inserter" => Some(InserterKind::Bulk),
            "stack-inserter" => Some(InserterKind::Stack),
            _ => None,
        }
    }

    /// Prototype `rotation_speed`, in turns per tick.
    ///
    /// A full pick-and-drop cycle is one turn, so
    /// `swings/s = rotation_speed × 60` and `cycle_ticks = 1 / rotation_speed`.
    /// Cross-check against `factorio-mechanics.md` **I8**, whose throughputs
    /// are stated as `rotation_speed × 60 × items_per_swing`:
    ///   regular     0.014 × 60 × 1 = 0.84/s ✓
    ///   long-handed 0.020 × 60 × 1 = 1.20/s ✓
    ///   fast        0.040 × 60 × 1 = 2.40/s ✓ (I8 quotes 2.31 with extension delay)
    pub fn rotation_speed(self) -> f64 {
        match self {
            InserterKind::Regular => 0.014,
            InserterKind::LongHanded => 0.020,
            InserterKind::Fast | InserterKind::Bulk | InserterKind::Stack => 0.040,
        }
    }

    /// Full swing cycle in ticks.
    pub fn cycle_ticks(self) -> f64 {
        1.0 / self.rotation_speed()
    }

    /// Reach in tiles (pickup and drop are each this far from the inserter).
    /// `factorio-mechanics.md` **I3**/**I4**: long-handed is the only
    /// reach-2 inserter in vanilla (**I8a**).
    pub fn reach(self) -> i32 {
        match self {
            InserterKind::LongHanded => 2,
            _ => 1,
        }
    }

    /// Base hand size at inserter-capacity research level 0.
    ///
    /// **Deliberate divergence from the engine's table, recorded because
    /// it is exactly the kind of inherited belief KC4 exists to prevent.**
    /// `factorio-mechanics.md` I8 lists stack 5 / bulk 1, and its own
    /// "I8 wiki cross-check (2026-07-20, #313)" note records that the wiki
    /// says **stack 6 / bulk 2**, with the engine keeping the lower values
    /// *on purpose* — conservative under-crediting is the safe direction
    /// for a sizing ladder and a throughput validator.
    ///
    /// The meter is not sizing anything. Under-crediting would make it
    /// measure something the game does not do, so it takes the game-true
    /// values. Any resulting fast-vs-real disagreement belongs in
    /// `docs/meter-divergence.md`, not in a fudge here.
    pub fn base_hand_size(self) -> u32 {
        match self {
            InserterKind::Regular | InserterKind::LongHanded | InserterKind::Fast => 1,
            InserterKind::Bulk => 2,
            InserterKind::Stack => 6,
        }
    }

    /// Hand size at a declared inserter-capacity research level (0–7).
    ///
    /// Schedule per `factorio-mechanics.md` **I8b** (RFC-049, pinned from
    /// raw wikitext). Both ladders are **literal tables**, not closed
    /// forms: neither is expressible as a clean saturating expression, and
    /// an earlier draft's arithmetic silently mis-transcribed both (review,
    /// PR #458 — bulk over-credited at L1–L6, non-bulk wrong at L2 and
    /// L4–L6). Transcribe, don't derive.
    pub fn hand_size(self, level: u8) -> u32 {
        let level = level.min(7) as usize;
        match self {
            InserterKind::Regular | InserterKind::LongHanded | InserterKind::Fast => {
                NON_BULK_HAND_BY_LEVEL[level]
            }
            InserterKind::Bulk => BULK_HAND_BY_LEVEL[level],
            // Stack = bulk-class + 4 built-in (`stack_size_bonus: 4` in the
            // entity prototype), giving I8b's 6,7,8,9,10,12,14,16.
            InserterKind::Stack => BULK_HAND_BY_LEVEL[level] + 4,
        }
    }
}

/// Bulk-inserter hand size by inserter-capacity research level 0..=7.
/// `factorio-mechanics.md` I8b: +1 at L1–4, +2 at L5–7.
const BULK_HAND_BY_LEVEL: [u32; 8] = [2, 3, 4, 5, 6, 8, 10, 12];

/// Regular / long-handed / fast hand size by research level 0..=7.
/// `factorio-mechanics.md` I8b: +1 at L2 and L7 only.
///
/// **Known unmodelled**: I8b also notes Transport-belt-capacity-2 grants a
/// further literal "non-bulk inserter capacity +1" → max 4, which the
/// *engine* bundles at L7. That bundling is an engine modelling choice
/// rather than a measured fact, so the meter takes the literal chain and
/// leaves the extra +1 out. Candidate `docs/meter-divergence.md` entry;
/// revisit against measurement in PR 2/3 rather than by inheriting the
/// engine's decision.
const NON_BULK_HAND_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 2, 2, 2, 3];

#[cfg(test)]
mod tests {
    use super::*;

    /// The meter must *derive* the belt throughputs the mechanics doc
    /// states, not be told them. If this drifts, either a constant is
    /// wrong or the slot model is.
    #[test]
    fn belt_throughput_derives_from_speed_and_spacing() {
        let both_lanes = |t: BeltTier| t.tiles_per_second() / ITEM_SPACING_TILES * 2.0;
        assert_eq!(both_lanes(BeltTier::Yellow), 15.0);
        assert_eq!(both_lanes(BeltTier::Red), 30.0);
        assert_eq!(both_lanes(BeltTier::Blue), 45.0);
        assert_eq!(both_lanes(BeltTier::Turbo), 60.0);
    }

    #[test]
    fn inserter_swing_rates_match_mechanics_i8() {
        let per_s = |k: InserterKind| TICKS_PER_SECOND / k.cycle_ticks();
        assert!((per_s(InserterKind::Regular) - 0.84).abs() < 1e-9);
        assert!((per_s(InserterKind::LongHanded) - 1.20).abs() < 1e-9);
        assert!((per_s(InserterKind::Fast) - 2.40).abs() < 1e-9);
    }

    #[test]
    fn long_handed_is_the_only_reach_two_inserter() {
        for k in [
            InserterKind::Regular,
            InserterKind::Fast,
            InserterKind::Bulk,
            InserterKind::Stack,
        ] {
            assert_eq!(k.reach(), 1, "{k:?} should be reach-1 (I8a)");
        }
        assert_eq!(InserterKind::LongHanded.reach(), 2);
    }

    #[test]
    fn belt_tier_parses_all_variants() {
        for (name, want) in [
            ("transport-belt", BeltTier::Yellow),
            ("underground-belt", BeltTier::Yellow),
            ("splitter", BeltTier::Yellow),
            ("fast-transport-belt", BeltTier::Red),
            ("fast-underground-belt", BeltTier::Red),
            ("express-transport-belt", BeltTier::Blue),
            ("express-splitter", BeltTier::Blue),
            ("turbo-transport-belt", BeltTier::Turbo),
        ] {
            assert_eq!(BeltTier::from_entity_name(name), Some(want), "{name}");
        }
        assert_eq!(BeltTier::from_entity_name("assembling-machine-3"), None);
    }
}
