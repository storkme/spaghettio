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

use crate::models::{EntityDirection, PlacedEntity};
use crate::zone_cache::DecodedRecord;

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
pub fn mine_palette(
    records: &[DecodedRecord],
    k_hops: usize,
    min_freq: usize,
) -> Vec<(SubPattern, usize)> {
    let mut hist: HashMap<SubPattern, usize> = HashMap::new();
    for rec in records {
        let mut seen_this_record: HashSet<SubPattern> = HashSet::new();
        for seed in 0..rec.entities.len() {
            let blob_indices = k_hop_neighborhood(seed, &rec.entities, k_hops);
            if blob_indices.len() < 2 {
                continue;
            }
            let blob: Vec<&PlacedEntity> = blob_indices.iter().map(|&i| &rec.entities[i]).collect();
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
