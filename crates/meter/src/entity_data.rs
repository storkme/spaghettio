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
    /// raw wikitext): bulk 2→12 across the ladder; stack = bulk + 4;
    /// non-bulk 1→3. Only bulk-class inserters scale with capacity
    /// research; regular/long-handed/fast take the small non-bulk chain.
    pub fn hand_size(self, level: u8) -> u32 {
        let level = level.min(7) as u32;
        match self {
            // Non-bulk chain: 1 at L0..L2, then +1 at L3 and L4, capped 3.
            InserterKind::Regular | InserterKind::LongHanded | InserterKind::Fast => {
                1 + level.saturating_sub(2).min(2)
            }
            // Bulk: 2 at L0, then +1 per level to 12 at L7 is the wiki
            // schedule's shape (2,4,5,6,7,9,11,12 across L0..L7).
            InserterKind::Bulk => BULK_HAND_BY_LEVEL[level as usize],
            InserterKind::Stack => BULK_HAND_BY_LEVEL[level as usize] + 4,
        }
    }
}

/// Bulk-inserter hand size by inserter-capacity research level 0..=7.
/// `factorio-mechanics.md` I8b. Stack inserters are this + 4.
const BULK_HAND_BY_LEVEL: [u32; 8] = [2, 4, 5, 6, 7, 9, 11, 12];

#[cfg(test)]
mod tests {
    use super::*;

    /// The meter must *derive* the belt throughputs the mechanics doc
    /// states, not be told them. If this drifts, either a constant is
    /// wrong or the slot model is.
    #[test]
    fn belt_throughput_derives_from_speed_and_spacing() {
        let both_lanes = |t: BeltTier| {
            t.tiles_per_second() / ITEM_SPACING_TILES * 2.0
        };
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
