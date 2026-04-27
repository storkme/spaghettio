//! Sub-pattern mining + recognition for SAT junction hinting.
//!
//! **Idea.** Most SAT crossing-zone solutions are assemblies of a few
//! recurring local primitives — perpendicular UG-bridges, L-turns, splitter
//! merges, fluid-isolating bridges, etc. If we can mine those primitives out
//! of the solved-zone corpus, we can recognize them in fresh problems and
//! pre-pin those tiles before SAT runs. Same SAT problem, much smaller
//! search tree.
//!
//! **Status.** This module is the data-side scaffolding: the [`SubPattern`]
//! type, the offline [`mine_palette`] function, normalization to a canonical
//! orientation/channel-labelling, and the env-gate [`palette_enabled`] for
//! the future hint-injection path. Recognition + injection are not wired in
//! yet — that lives in a follow-up commit once we've eyeballed what
//! `diag_mine_palette` actually finds in the cache corpus.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

use crate::models::{EntityDirection, PlacedEntity};

/// One entity in a normalized [`SubPattern`]. Coords are pattern-local
/// (`x`, `y` ∈ `[0, w) × [0, h)`); `channel` is a relabelling-invariant
/// 0..N that's independent of the zone's original `channel_id`s.
///
/// All five fields fit in 5 bytes — same encoding used by the persistent
/// [`crate::zone_cache`] tuples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NormalizedEntity {
    pub kind: u8,
    pub x: u8,
    pub y: u8,
    pub dir: u8,
    pub channel: u8,
}

/// A connected blob of entities in a single canonical orientation. Two
/// `SubPattern`s are equal iff they describe the same local shape after D4
/// rotation/reflection AND channel-relabelling — so all 8 orientations and
/// all permutations of channel labels collapse to one representative.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubPattern {
    /// Entities sorted in `(y, x, kind, dir, channel)` order. The sort
    /// itself is part of the canonical form.
    pub entities: Vec<NormalizedEntity>,
    pub w: u8,
    pub h: u8,
    pub n_channels: u8,
}

// ---------------------------------------------------------------------------
// Feature gate + instrumentation hooks
// ---------------------------------------------------------------------------

/// Whether the future hint-injection path is active. Default `false` —
/// flip with `FUCKTORIO_USE_SAT_HINTS=1`.
///
/// Reading the env var at every SAT call is cheap (env access is O(1) on
/// glibc/musl) and lets us A/B compare hint-on vs hint-off in the same
/// process via `set_var` from a diag test.
pub fn palette_enabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    { false }
    #[cfg(not(target_arch = "wasm32"))]
    {
        matches!(
            std::env::var("FUCKTORIO_USE_SAT_HINTS").as_deref(),
            Ok("1") | Ok("true") | Ok("True") | Ok("TRUE")
        )
    }
}

// ---------------------------------------------------------------------------
// Mining: corpus → palette
// ---------------------------------------------------------------------------

/// Default neighbourhood radius (BFS depth from each seed entity). 2 hops
/// gives 1 + 4 + 8 = 13 entities max per blob, which is the right scale for
/// "perpendicular crossing" / "L-turn" sized primitives without exploding
/// into zone-spanning chunks.
pub const DEFAULT_K_HOPS: usize = 2;

/// Default minimum frequency for inclusion in the palette: a pattern has to
/// appear in at least this many records to be kept. Cuts the long tail of
/// one-off shapes that won't help recognition.
pub const DEFAULT_MIN_FREQ: usize = 3;

/// Mine sub-patterns from a slice of cached records. For each entity in
/// each record, takes the `k_hops`-radius BFS neighbourhood (entities
/// connected by tile adjacency), normalizes via [`canonicalise`], and
/// counts each pattern at most once per record.
///
/// Returns `(pattern, occurrences)` pairs sorted by descending occurrences.
/// Patterns appearing in fewer than `min_freq` records are dropped.
///
/// Each `record_entities` element is one solved zone's full entity list (in
/// canonical-frame local coords with `chN` channel tokens — same shape that
/// `crate::zone_cache::DecodedRecord::entities` produces).
pub fn mine_palette(
    record_entities: &[Vec<PlacedEntity>],
    k_hops: usize,
    min_freq: usize,
) -> Vec<(SubPattern, usize)> {
    let mut hist: HashMap<SubPattern, usize> = HashMap::new();
    for entities in record_entities {
        let mut seen_this_record: HashSet<SubPattern> = HashSet::new();
        for seed in 0..entities.len() {
            let blob_indices = k_hop_neighborhood(seed, entities, k_hops);
            if blob_indices.len() < 2 {
                continue;
            }
            let blob: Vec<&PlacedEntity> = blob_indices.iter().map(|&i| &entities[i]).collect();
            let pattern = canonicalise(&blob);
            if seen_this_record.insert(pattern.clone()) {
                *hist.entry(pattern).or_default() += 1;
            }
        }
    }
    let mut filtered: Vec<(SubPattern, usize)> = hist
        .into_iter()
        .filter(|(_, n)| *n >= min_freq)
        .collect();
    filtered.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    filtered
}

/// BFS from `seed_idx` collecting every entity reachable within `k_hops`
/// tile-adjacency steps. Returns indices into `entities` (including the
/// seed itself).
fn k_hop_neighborhood(
    seed_idx: usize,
    entities: &[PlacedEntity],
    k_hops: usize,
) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut frontier: VecDeque<(usize, usize)> = VecDeque::new();  // (idx, depth)
    visited.insert(seed_idx);
    frontier.push_back((seed_idx, 0));

    // Build a tile → idx map once for fast adjacency lookup.
    let tile_index: HashMap<(i32, i32), usize> = entities
        .iter()
        .enumerate()
        .map(|(i, e)| ((e.x, e.y), i))
        .collect();

    while let Some((idx, depth)) = frontier.pop_front() {
        if depth >= k_hops {
            continue;
        }
        let e = &entities[idx];
        for &(dx, dy) in &[(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            if let Some(&neighbour) = tile_index.get(&(e.x + dx, e.y + dy)) {
                if visited.insert(neighbour) {
                    frontier.push_back((neighbour, depth + 1));
                }
            }
        }
    }
    visited.into_iter().collect()
}

/// Compute the canonical [`SubPattern`] for an arbitrary blob of entities.
/// Tries all 8 D4 orientations × shifts to (0,0), relabels channels in
/// (y, x)-encounter order under each transform, and returns the lex-min
/// representation. Two geometrically-equivalent blobs always produce the
/// same `SubPattern`.
pub fn canonicalise(blob: &[&PlacedEntity]) -> SubPattern {
    let mut best: Option<SubPattern> = None;
    for rotation in 0u8..4 {
        for &reflect in &[false, true] {
            let candidate = transform_and_normalize(blob, rotation, reflect);
            if best.as_ref().is_none_or(|b| candidate < *b) {
                best = Some(candidate);
            }
        }
    }
    best.unwrap_or(SubPattern { entities: Vec::new(), w: 0, h: 0, n_channels: 0 })
}

fn transform_and_normalize(
    blob: &[&PlacedEntity],
    rotation: u8,
    reflect: bool,
) -> SubPattern {
    // Step 1: apply D4 transform to each entity's (x, y) and direction.
    // We don't yet know the bounding box, so transform around an arbitrary
    // origin and shift afterwards. For (x, y) under reflect-then-rotate
    // applied to UNbounded coordinates, we use the linear part only:
    //   reflect: (x, y) -> (-x, y)
    //   90° CW rotate: (x, y) -> (-y, x)
    let mut tx: Vec<(i32, i32, u8, u8, Option<u32>)> = Vec::with_capacity(blob.len());
    for e in blob {
        let (mut x, mut y) = (e.x, e.y);
        if reflect {
            x = -x;
        }
        let mut dir = dir_to_u8(e.direction);
        if reflect {
            // E↔W under vertical flip; N/S unchanged.
            dir = match dir {
                1 => 3,
                3 => 1,
                other => other,
            };
        }
        for _ in 0..rotation {
            let (nx, ny) = (-y, x);
            x = nx;
            y = ny;
            dir = (dir + 1) % 4;  // N→E→S→W→N
        }
        let kind = kind_byte_for(e);
        let chan = parse_channel_token(e.carries.as_deref());
        tx.push((x, y, kind, dir, chan));
    }

    // Step 2: shift so min_x = min_y = 0.
    let min_x = tx.iter().map(|t| t.0).min().unwrap_or(0);
    let min_y = tx.iter().map(|t| t.1).min().unwrap_or(0);
    let max_x = tx.iter().map(|t| t.0).max().unwrap_or(0);
    let max_y = tx.iter().map(|t| t.1).max().unwrap_or(0);
    let w = (max_x - min_x + 1).clamp(0, 255) as u8;
    let h = (max_y - min_y + 1).clamp(0, 255) as u8;

    // Step 3: sort entities in (y, x, kind, dir, original_channel) order, so
    // channel-relabelling is deterministic.
    tx.sort_by(|a, b| {
        (a.1 - min_y, a.0 - min_x, a.2, a.3, a.4)
            .cmp(&(b.1 - min_y, b.0 - min_x, b.2, b.3, b.4))
    });

    // Step 4: relabel channels in encounter order.
    let mut channel_map: BTreeMap<u32, u8> = BTreeMap::new();
    let mut entities = Vec::with_capacity(tx.len());
    for (x, y, kind, dir, chan) in tx {
        let lx = (x - min_x).clamp(0, 255) as u8;
        let ly = (y - min_y).clamp(0, 255) as u8;
        let channel = match chan {
            None => u8::MAX,  // sentinel: no carries
            Some(orig) => {
                let next = channel_map.len() as u8;
                *channel_map.entry(orig).or_insert(next)
            }
        };
        entities.push(NormalizedEntity { kind, x: lx, y: ly, dir, channel });
    }
    let n_channels = channel_map.len() as u8;

    SubPattern { entities, w, h, n_channels }
}

fn dir_to_u8(d: EntityDirection) -> u8 {
    match d {
        EntityDirection::North => 0,
        EntityDirection::East => 1,
        EntityDirection::South => 2,
        EntityDirection::West => 3,
    }
}

/// Map an entity's `name + io_type` to the same 0..8 enum the binary cache
/// uses (yellow/red/blue × belt/UG-in/UG-out).
fn kind_byte_for(e: &PlacedEntity) -> u8 {
    let io = e.io_type.as_deref();
    match (e.name.as_str(), io) {
        ("transport-belt", _) => 0,
        ("fast-transport-belt", _) => 1,
        ("express-transport-belt", _) => 2,
        ("underground-belt", Some("input")) => 3,
        ("underground-belt", Some("output")) => 4,
        ("fast-underground-belt", Some("input")) => 5,
        ("fast-underground-belt", Some("output")) => 6,
        ("express-underground-belt", Some("input")) => 7,
        ("express-underground-belt", Some("output")) => 8,
        _ => u8::MAX,
    }
}

/// Parse a `"chN"` carries token into `Some(N)`; everything else (including
/// `None`) becomes `None`.
fn parse_channel_token(carries: Option<&str>) -> Option<u32> {
    carries
        .and_then(|s| s.strip_prefix("ch"))
        .and_then(|n| n.parse::<u32>().ok())
}

// ---------------------------------------------------------------------------
// Recognition: zone → candidate placements
// ---------------------------------------------------------------------------

/// One place in a zone where a palette pattern could be pinned. Carries
/// enough context to translate back into per-tile SAT unit clauses if the
/// future hint-injection path decides to commit the placement.
///
/// **Shadow-mode caveat.** Today we only check geometric feasibility (in
/// bounds, not on a forbidden tile, not on a boundary tile). We do NOT yet
/// verify that the pattern's belt directions are consistent with the
/// zone's per-channel flow direction. So `Placement` count is an
/// upper-bound — actually pinning these would over-constrain SAT. Real
/// validity checking lands with the SAT-injection commit.
#[derive(Debug, Clone)]
pub struct Placement {
    /// Index into the palette slice passed to [`recognise`].
    pub pattern_idx: usize,
    /// D4 rotation applied to the pattern (0..4).
    pub rotation: u8,
    /// D4 reflection applied (before rotation).
    pub reflect: bool,
    /// Zone-local origin where the pattern's `(0, 0)` corner sits after
    /// the D4 transform.
    pub origin_x: u8,
    pub origin_y: u8,
    /// Tiles (zone-local) the pattern would pin if accepted.
    pub tiles: Vec<(u8, u8)>,
}

/// Geometric inputs the recogniser needs from a zone, decoupled from
/// `crate::sat::CrossingZone` to keep the dependency graph acyclic.
pub struct ZoneShape<'a> {
    pub width: u8,
    pub height: u8,
    /// Forced-empty tiles in zone-local coords.
    pub forbidden: &'a HashSet<(u8, u8)>,
    /// Boundary port tiles in zone-local coords. The recogniser refuses to
    /// place pattern entities on these tiles — the encoder owns boundary
    /// constraints and any pin would either contradict them or be redundant.
    pub boundary: &'a HashSet<(u8, u8)>,
}

/// Find every palette pattern that could be pinned in `zone`, considering
/// all 8 D4 orientations and every (origin_x, origin_y) position. **Does
/// not validate flow direction or channel topology** — see the caveat on
/// [`Placement`].
///
/// Output is unfiltered (overlapping placements coexist). Use
/// [`greedy_cover`] to pick a non-overlapping subset.
pub fn recognise(zone: &ZoneShape<'_>, palette: &[SubPattern]) -> Vec<Placement> {
    let mut out = Vec::new();
    for (idx, pattern) in palette.iter().enumerate() {
        for rotation in 0u8..4 {
            for &reflect in &[false, true] {
                let (pw, ph) = if rotation.is_multiple_of(2) {
                    (pattern.w, pattern.h)
                } else {
                    (pattern.h, pattern.w)
                };
                if pw > zone.width || ph > zone.height {
                    continue;
                }
                let max_x = zone.width - pw;
                let max_y = zone.height - ph;
                for origin_x in 0..=max_x {
                    for origin_y in 0..=max_y {
                        if let Some(tiles) = try_place(
                            pattern, rotation, reflect, origin_x, origin_y,
                            pw, ph, zone,
                        ) {
                            out.push(Placement {
                                pattern_idx: idx,
                                rotation,
                                reflect,
                                origin_x,
                                origin_y,
                                tiles,
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

/// Try to place `pattern` at `(origin_x, origin_y)` with the given D4
/// transform. Returns the absolute zone-local tiles if every entity lands
/// on a free, in-bounds, non-boundary tile; otherwise `None`.
fn try_place(
    pattern: &SubPattern,
    rotation: u8,
    reflect: bool,
    origin_x: u8,
    origin_y: u8,
    pw: u8,
    ph: u8,
    zone: &ZoneShape<'_>,
) -> Option<Vec<(u8, u8)>> {
    let mut tiles = Vec::with_capacity(pattern.entities.len());
    for e in &pattern.entities {
        let (lx, ly) = transform_point(e.x, e.y, pattern.w, pattern.h, rotation, reflect);
        let zx = origin_x.checked_add(lx)?;
        let zy = origin_y.checked_add(ly)?;
        if zx >= origin_x + pw || zy >= origin_y + ph {
            return None;
        }
        let tile = (zx, zy);
        if zone.forbidden.contains(&tile) || zone.boundary.contains(&tile) {
            return None;
        }
        tiles.push(tile);
    }
    Some(tiles)
}

/// Apply the same D4 transform we use in [`canonicalise`] to a single
/// `(x, y)` point inside a `(w, h)` bbox; returns its position in the
/// post-transform `(tw, th)` bbox.
fn transform_point(x: u8, y: u8, w: u8, h: u8, rotation: u8, reflect: bool) -> (u8, u8) {
    let (mut x, mut y) = (x as i32, y as i32);
    if reflect {
        x = (w as i32 - 1) - x;
    }
    let mut cur_w = w as i32;
    let mut cur_h = h as i32;
    for _ in 0..rotation {
        let (nx, ny) = (cur_h - 1 - y, x);
        x = nx;
        y = ny;
        std::mem::swap(&mut cur_w, &mut cur_h);
    }
    (x.max(0) as u8, y.max(0) as u8)
}

/// Pick a non-overlapping subset of `placements` greedily. Higher-frequency
/// patterns (earlier in `palette`) are preferred; among equal-rank, larger
/// footprints win.
pub fn greedy_cover(placements: &[Placement], palette_freq: &[usize]) -> Vec<Placement> {
    let mut order: Vec<usize> = (0..placements.len()).collect();
    order.sort_by(|&a, &b| {
        let pa = &placements[a];
        let pb = &placements[b];
        let fa = palette_freq.get(pa.pattern_idx).copied().unwrap_or(0);
        let fb = palette_freq.get(pb.pattern_idx).copied().unwrap_or(0);
        fb.cmp(&fa).then(pb.tiles.len().cmp(&pa.tiles.len()))
    });
    let mut claimed: HashSet<(u8, u8)> = HashSet::new();
    let mut chosen = Vec::new();
    for i in order {
        let p = &placements[i];
        if p.tiles.iter().any(|t| claimed.contains(t)) {
            continue;
        }
        for &t in &p.tiles {
            claimed.insert(t);
        }
        chosen.push(p.clone());
    }
    chosen
}

// ---------------------------------------------------------------------------
// Process-wide palette: lazily mined from the embedded sat-zones.bin so the
// recogniser has something to work with on first call.
// ---------------------------------------------------------------------------

/// Pre-baked cache embedded in WASM builds — same blob the zone cache
/// uses for cold-start lookups.
#[cfg(target_arch = "wasm32")]
const EMBEDDED_CACHE: &[u8] = include_bytes!("../data/sat-zones.bin");

/// One palette entry: the pattern itself plus its mining-time frequency
/// (used by [`greedy_cover`] for tie-breaking).
#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub pattern: SubPattern,
    pub freq: usize,
}

/// Lazy-init shared palette. First call mines from the embedded
/// `sat-zones.bin` (and on native, the on-disk equivalent). Subsequent
/// calls reuse the cached `Vec`.
pub fn shared_palette() -> &'static Vec<PaletteEntry> {
    static PALETTE: OnceLock<Vec<PaletteEntry>> = OnceLock::new();
    PALETTE.get_or_init(load_palette_from_zone_cache)
}

#[cfg(target_arch = "wasm32")]
fn load_palette_from_zone_cache() -> Vec<PaletteEntry> {
    let records = crate::zone_cache::parse_records(EMBEDDED_CACHE);
    let entity_lists: Vec<Vec<PlacedEntity>> = records.into_iter().map(|r| r.entities).collect();
    mine_palette(&entity_lists, DEFAULT_K_HOPS, DEFAULT_MIN_FREQ)
        .into_iter()
        .map(|(pattern, freq)| PaletteEntry { pattern, freq })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_palette_from_zone_cache() -> Vec<PaletteEntry> {
    // On native, read the embedded blob from the source tree — same path
    // that's `include_bytes!`d for WASM. Falls back to empty on any I/O
    // error so this is never load-bearing for correctness.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/sat-zones.bin");
    let Ok(bytes) = std::fs::read(&path) else { return Vec::new() };
    let records = crate::zone_cache::parse_records(&bytes);
    let entity_lists: Vec<Vec<PlacedEntity>> = records.into_iter().map(|r| r.entities).collect();
    mine_palette(&entity_lists, DEFAULT_K_HOPS, DEFAULT_MIN_FREQ)
        .into_iter()
        .map(|(pattern, freq)| PaletteEntry { pattern, freq })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PlacedEntity;

    fn ent(name: &str, x: i32, y: i32, dir: EntityDirection, carries: Option<&str>, io: Option<&str>) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x, y, direction: dir,
            carries: carries.map(str::to_string),
            io_type: io.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn rotation_invariance() {
        // Two belts side-by-side, both flowing east, ch0.
        let h_blob = vec![
            ent("transport-belt", 0, 0, EntityDirection::East, Some("ch0"), None),
            ent("transport-belt", 1, 0, EntityDirection::East, Some("ch0"), None),
        ];
        // Same shape rotated 90° CW: now top-to-bottom, both flowing south.
        let v_blob = vec![
            ent("transport-belt", 0, 0, EntityDirection::South, Some("ch0"), None),
            ent("transport-belt", 0, 1, EntityDirection::South, Some("ch0"), None),
        ];
        let h_refs: Vec<_> = h_blob.iter().collect();
        let v_refs: Vec<_> = v_blob.iter().collect();
        assert_eq!(canonicalise(&h_refs), canonicalise(&v_refs));
    }

    #[test]
    fn channel_relabel_invariance() {
        // Two patterns with the same shape but different original channel ids.
        let a = vec![
            ent("transport-belt", 0, 0, EntityDirection::East, Some("ch3"), None),
            ent("transport-belt", 1, 0, EntityDirection::East, Some("ch3"), None),
        ];
        let b = vec![
            ent("transport-belt", 0, 0, EntityDirection::East, Some("ch7"), None),
            ent("transport-belt", 1, 0, EntityDirection::East, Some("ch7"), None),
        ];
        let a_refs: Vec<_> = a.iter().collect();
        let b_refs: Vec<_> = b.iter().collect();
        assert_eq!(canonicalise(&a_refs), canonicalise(&b_refs));
    }

    #[test]
    fn kind_distinguishes_patterns() {
        // Same geometry, different entity kind — should NOT match.
        let yellow = vec![
            ent("transport-belt", 0, 0, EntityDirection::East, Some("ch0"), None),
            ent("transport-belt", 1, 0, EntityDirection::East, Some("ch0"), None),
        ];
        let red = vec![
            ent("fast-transport-belt", 0, 0, EntityDirection::East, Some("ch0"), None),
            ent("fast-transport-belt", 1, 0, EntityDirection::East, Some("ch0"), None),
        ];
        let y_refs: Vec<_> = yellow.iter().collect();
        let r_refs: Vec<_> = red.iter().collect();
        assert_ne!(canonicalise(&y_refs), canonicalise(&r_refs));
    }

    #[test]
    fn perpendicular_crossing_match() {
        // A canonical "perpendicular crossing": ch0 going east, ch1 going south, crossing tile (1, 1).
        let mk = |ch0: &str, ch1: &str| vec![
            ent("transport-belt", 0, 1, EntityDirection::East, Some(ch0), None),
            ent("transport-belt", 1, 1, EntityDirection::East, Some(ch0), None),
            ent("transport-belt", 2, 1, EntityDirection::East, Some(ch0), None),
            ent("fast-underground-belt", 1, 0, EntityDirection::South, Some(ch1), Some("input")),
            ent("fast-underground-belt", 1, 2, EntityDirection::South, Some(ch1), Some("output")),
        ];
        let a: Vec<_> = mk("ch0", "ch1").into_iter().collect();
        let b: Vec<_> = mk("ch5", "ch9").into_iter().collect();  // different channel ids
        let a_refs: Vec<_> = a.iter().collect();
        let b_refs: Vec<_> = b.iter().collect();
        assert_eq!(canonicalise(&a_refs), canonicalise(&b_refs));
    }
}
