//! Lightweight append-only cache of SAT crossing-zone shapes.
//!
//! Writes one JSONL record per solved zone to:
//!   1. `$FUCKTORIO_ZONE_CACHE_PATH`            — if that env var is set (full path override)
//!   2. `$XDG_CACHE_HOME/fucktorio/sat-zones.jsonl`
//!   3. `$HOME/.cache/fucktorio/sat-zones.jsonl` — final fallback
//!
//! The signature is designed to be a future cache key, not just a histogram
//! bucket: it captures port connectivity (grouped by channel_id), forbidden
//! interior tiles, per-channel UG reach, and the `max_ug_ins` cap. Two zones
//! with the same signature have identical SAT problems, so a cached solution
//! from one would be valid for the other (modulo D4 replay).
//!
//! Gated behind `#[cfg(not(target_arch = "wasm32"))]` so it compiles to nothing
//! in WASM builds.

#![cfg(not(target_arch = "wasm32"))]

use crate::models::{EntityDirection, PlacedEntity};
use crate::sat::CrossingZone;

/// Internal edge classification used only for the canonical signature
/// computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    N,
    E,
    S,
    W,
}

/// Derive `(edge, offset)` for a port at `(px, py)` on the boundary of a
/// `(width, height)` zone anchored at `(zx, zy)`. Returns `None` if the
/// port point is outside the zone bbox.
fn edge_and_offset(
    zx: i32,
    zy: i32,
    w: u32,
    h: u32,
    px: i32,
    py: i32,
    fallback_dir: crate::models::EntityDirection,
) -> Option<(Edge, u32)> {
    let lx = px - zx;
    let ly = py - zy;
    if lx < 0 || ly < 0 || lx as u32 >= w || ly as u32 >= h {
        return None;
    }
    let lxu = lx as u32;
    let lyu = ly as u32;
    let on_north = lyu == 0;
    let on_south = lyu == h - 1;
    let on_west = lxu == 0;
    let on_east = lxu == w - 1;
    let edge = match (on_north, on_south, on_west, on_east) {
        (true, false, false, false) => Edge::N,
        (false, true, false, false) => Edge::S,
        (false, false, true, false) => Edge::W,
        (false, false, false, true) => Edge::E,
        // Corner/degenerate — break ties with flow direction.
        _ => {
            use crate::models::EntityDirection;
            match fallback_dir {
                EntityDirection::North => Edge::N,
                EntityDirection::South => Edge::S,
                EntityDirection::West => Edge::W,
                EntityDirection::East => Edge::E,
            }
        }
    };
    let offset = match edge {
        Edge::N | Edge::S => lxu,
        Edge::E | Edge::W => lyu,
    };
    Some((edge, offset))
}

/// SAT solver stats associated with a zone record. Mirrors
/// `sat::CrossingZoneStats` but kept local so `zone_cache` doesn't depend on
/// the full `sat` module's stats type for callers that just want to record.
pub struct ZoneStats {
    pub variables: u32,
    pub clauses: u32,
    pub solve_time_us: u64,
}
use std::cell::RefCell;
use std::io::Write as _;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Thread-local source tag so parallel tests each carry their own label
// without stomping on each other via a process-global env var.
thread_local! {
    static ZONE_SOURCE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Per-zone record buffered between flushes. Stored serialised because
/// `serde_json::Value` would balloon memory and we already need the line
/// bytes for the eventual write.
struct PendingRecord {
    /// One full JSONL line including trailing `\n`. Pre-encoded so the flush
    /// path is just byte writes.
    line: Vec<u8>,
}

/// Process-global record buffer. Records are appended via `record_zone` and
/// drained by `flush`. Keeping the lock contention to a brief
/// `Vec::push` of a pre-encoded line means parallel tests don't serialise
/// on the recorder.
static BUFFER: OnceLock<Mutex<Vec<PendingRecord>>> = OnceLock::new();

fn buffer() -> &'static Mutex<Vec<PendingRecord>> {
    BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

/// Set the per-thread zone source tag. Call this at the start of a test body.
pub fn set_thread_source(source: Option<&str>) {
    ZONE_SOURCE.with(|s| *s.borrow_mut() = source.map(|s| s.to_string()));
}

/// Resolve the JSONL output path.
fn resolve_cache_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("FUCKTORIO_ZONE_CACHE_PATH") {
        return std::path::PathBuf::from(p);
    }
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".cache"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from(".cache"));
    base.join("fucktorio").join("sat-zones.jsonl")
}

// ---------------------------------------------------------------------------
// Canonical orientation-invariant signature
// ---------------------------------------------------------------------------

/// Apply a D4 transform (rotation × reflection) to an `(edge, offset)` pair
/// in a `(w, h)` zone, returning the post-transform `(edge, offset, new_w, new_h)`.
///
/// `edge`: 0=N, 1=E, 2=S, 3=W. `rotation` in 0..4 (90° CW each), `reflect`
/// applied first (vertical-axis flip).
fn transform_port(
    edge: u8,
    offset: u32,
    w: u32,
    h: u32,
    rotation: u8,
    reflect: bool,
) -> (u8, u32, u32, u32) {
    let (edge, offset) = if reflect {
        match edge {
            0 => (0u8, w.saturating_sub(1).saturating_sub(offset)),
            1 => (3u8, offset),
            2 => (2u8, w.saturating_sub(1).saturating_sub(offset)),
            3 => (1u8, offset),
            _ => (edge, offset),
        }
    } else {
        (edge, offset)
    };

    let mut e = edge;
    let mut o = offset;
    let mut cur_w = w;
    let mut cur_h = h;
    for _ in 0..rotation {
        let (ne, no) = match e {
            0 => (1u8, o),
            1 => (2u8, cur_w.saturating_sub(1).saturating_sub(o)),
            2 => (3u8, cur_h.saturating_sub(1).saturating_sub(o)),
            3 => (0u8, o),
            _ => (e, o),
        };
        e = ne;
        o = no;
        std::mem::swap(&mut cur_w, &mut cur_h);
    }

    (e, o, cur_w, cur_h)
}

/// Apply a D4 transform to a local `(x, y)` tile in a `(w, h)` zone.
/// Reflection (vertical-axis flip) applied first, then `rotation` 90° CW
/// rotations. Returns position in the post-transform frame.
fn transform_tile(x: u32, y: u32, w: u32, h: u32, rotation: u8, reflect: bool) -> (u32, u32) {
    let (mut tx, mut ty) = if reflect {
        (w.saturating_sub(1).saturating_sub(x), y)
    } else {
        (x, y)
    };
    let mut cur_w = w;
    let mut cur_h = h;
    for _ in 0..rotation {
        // 90° CW: (x, y) in (w, h) → (h - 1 - y, x) in (h, w)
        let (nx, ny) = (cur_h.saturating_sub(1).saturating_sub(ty), tx);
        tx = nx;
        ty = ny;
        std::mem::swap(&mut cur_w, &mut cur_h);
    }
    (tx, ty)
}

/// Invert [`transform_tile`]. Given a tile in the post-transform `(tw, th)`
/// frame, return its position in the original `(w, h)` frame.
///
/// `T = R^rot ∘ Refl^refl` (refl first, then rot), so `T⁻¹ = Refl ∘ R^(4-rot)`:
/// rotate by `(4-rot)%4` 90° CW first, then apply reflect (involution).
fn transform_tile_inverse(x: u32, y: u32, tw: u32, th: u32, rotation: u8, reflect: bool) -> (u32, u32) {
    let inv_rot = (4 - rotation) % 4;
    let mut tx = x;
    let mut ty = y;
    let mut cur_w = tw;
    let mut cur_h = th;
    for _ in 0..inv_rot {
        let (nx, ny) = (cur_h.saturating_sub(1).saturating_sub(ty), tx);
        tx = nx;
        ty = ny;
        std::mem::swap(&mut cur_w, &mut cur_h);
    }
    if reflect {
        tx = cur_w.saturating_sub(1).saturating_sub(tx);
    }
    (tx, ty)
}

/// Apply a D4 transform to an `EntityDirection`. Reflection first
/// (vertical-axis flip: E↔W, N/S unchanged), then `rotation` 90° CW rotations
/// (N→E→S→W→N).
fn transform_direction(d: EntityDirection, rotation: u8, reflect: bool) -> EntityDirection {
    let d = if reflect {
        match d {
            EntityDirection::East => EntityDirection::West,
            EntityDirection::West => EntityDirection::East,
            other => other,
        }
    } else {
        d
    };
    let mut d = d;
    for _ in 0..rotation {
        d = match d {
            EntityDirection::North => EntityDirection::East,
            EntityDirection::East => EntityDirection::South,
            EntityDirection::South => EntityDirection::West,
            EntityDirection::West => EntityDirection::North,
        };
    }
    d
}

/// Invert [`transform_direction`].
fn transform_direction_inverse(d: EntityDirection, rotation: u8, reflect: bool) -> EntityDirection {
    let inv_rot = (4 - rotation) % 4;
    let mut d = d;
    for _ in 0..inv_rot {
        d = match d {
            EntityDirection::North => EntityDirection::East,
            EntityDirection::East => EntityDirection::South,
            EntityDirection::South => EntityDirection::West,
            EntityDirection::West => EntityDirection::North,
        };
    }
    if reflect {
        d = match d {
            EntityDirection::East => EntityDirection::West,
            EntityDirection::West => EntityDirection::East,
            other => other,
        };
    }
    d
}

/// Apply the D4 transform `(rotation, reflect)` to an entity. Input is
/// expected in zone-local coords (i.e. `entity.x` ∈ `[0, w)`); output is in
/// the post-transform frame (size `(tw, th)`).
///
/// Belts, undergrounds, splitters and any other 1×1 entity work cleanly here.
/// The cache scope only covers SAT-produced junction entities, all of which
/// are 1×1, so multi-tile entity rotation isn't a concern.
fn transform_entity(
    entity: &PlacedEntity,
    w: u32,
    h: u32,
    rotation: u8,
    reflect: bool,
) -> PlacedEntity {
    let (nx, ny) = transform_tile(entity.x as u32, entity.y as u32, w, h, rotation, reflect);
    let mut out = entity.clone();
    out.x = nx as i32;
    out.y = ny as i32;
    out.direction = transform_direction(entity.direction, rotation, reflect);
    out
}

/// Inverse of [`transform_entity`]. Input is in the post-transform frame
/// (size `(tw, th)`); output is in the original frame.
fn transform_entity_inverse(
    entity: &PlacedEntity,
    tw: u32,
    th: u32,
    rotation: u8,
    reflect: bool,
) -> PlacedEntity {
    let (nx, ny) = transform_tile_inverse(entity.x as u32, entity.y as u32, tw, th, rotation, reflect);
    let mut out = entity.clone();
    out.x = nx as i32;
    out.y = ny as i32;
    out.direction = transform_direction_inverse(entity.direction, rotation, reflect);
    out
}

/// Format a (edge, offset) port endpoint as e.g. `N2` or `E0`.
fn format_endpoint(edge: u8, offset: u32) -> String {
    let e = match edge {
        0 => 'N',
        1 => 'E',
        2 => 'S',
        3 => 'W',
        _ => '?',
    };
    format!("{}{}", e, offset)
}

/// Compute the canonical orientation-invariant signature for a SAT crossing
/// zone.
///
/// Format: `"{W}x{H}:{channels}|F:{forbidden}|UG:{cap}"`
///
/// - `channels`: per-channel tuples `"{in_endpoints}>{out_endpoints}@{reach}"`,
///   joined by `;`. Each endpoint list is sorted (e.g. `N1+S2>E0@5`), so
///   "channel A enters at N1 and S2, exits at E0, has reach 5".
/// - `forbidden`: sorted list of `"{x},{y}"` interior tiles, joined by `;`.
/// - `cap`: `max_ug_ins` value, or `*` for unlimited.
///
/// Tries all 8 D4 symmetries (4 rotations × 2 reflections), transforms ports
/// AND forbidden tiles together, and returns the lex-smallest rendering.
///
/// Thin wrapper around [`canonicalise`] that throws away the orientation tag.
pub fn canonical_signature(
    zone: &CrossingZone,
    channel_reaches: &[u32],
    max_ug_ins: Option<u32>,
) -> String {
    canonicalise(zone, channel_reaches, max_ug_ins).signature
}

/// Result of canonicalising a crossing zone — the signature plus enough
/// orientation metadata to round-trip a stored SAT solution back into the
/// original (or any same-shape) zone's coordinate frame.
#[derive(Debug, Clone)]
pub struct CanonicalForm {
    /// Lex-min rendering across all 8 D4 transforms — the cache key.
    pub signature: String,
    /// Number of 90° clockwise rotations applied to reach the canonical
    /// orientation. 0..4.
    pub rotation: u8,
    /// Whether a vertical-axis reflection is applied (before rotation) to
    /// reach the canonical orientation.
    pub reflect: bool,
    /// Canonical channel ordering: `channel_order[i]` is the source-zone
    /// `channel_id` whose channel string ended up at position `i` in the
    /// sorted-channel list inside the canonical signature.
    ///
    /// Used to map between item names (which live on `channel_id`s in the
    /// source zone) and canonical channel positions (which the cache stores).
    pub channel_order: Vec<u32>,
}

/// Canonicalise a crossing zone: compute its signature plus the orientation
/// and channel mapping required to translate stored SAT solutions back into
/// this zone's frame.
pub fn canonicalise(
    zone: &CrossingZone,
    channel_reaches: &[u32],
    max_ug_ins: Option<u32>,
) -> CanonicalForm {
    // Base port info: (edge, offset, channel_id, is_input)
    let base_ports: Vec<(u8, u32, u32, bool)> = zone
        .boundaries
        .iter()
        .filter_map(|b| {
            let (edge, offset) =
                edge_and_offset(zone.x, zone.y, zone.width, zone.height, b.x, b.y, b.direction)?;
            let edge_idx = match edge {
                Edge::N => 0,
                Edge::E => 1,
                Edge::S => 2,
                Edge::W => 3,
            };
            Some((edge_idx, offset, b.channel_id, b.is_input))
        })
        .collect();

    // Base forbidden tiles in local zone coords.
    let base_forbidden: Vec<(u32, u32)> = zone
        .forced_empty
        .iter()
        .filter_map(|&(wx, wy)| {
            let lx = wx - zone.x;
            let ly = wy - zone.y;
            if lx < 0 || ly < 0 || lx as u32 >= zone.width || ly as u32 >= zone.height {
                return None;
            }
            Some((lx as u32, ly as u32))
        })
        .collect();

    let cap_str = match max_ug_ins {
        None => "*".to_string(),
        Some(k) => k.to_string(),
    };

    // Track each candidate as (signature, rotation, reflect, channel_order).
    let mut candidates: Vec<(String, u8, bool, Vec<u32>)> = Vec::with_capacity(8);

    for rotation in 0u8..4 {
        for &reflect in &[false, true] {
            let (tw, th) = if rotation.is_multiple_of(2) {
                (zone.width, zone.height)
            } else {
                (zone.height, zone.width)
            };

            // Group transformed ports by channel_id.
            let mut by_channel: std::collections::BTreeMap<u32, (Vec<String>, Vec<String>)> =
                Default::default();
            for &(edge_idx, offset, channel_id, is_input) in &base_ports {
                let (ne, no, _, _) =
                    transform_port(edge_idx, offset, zone.width, zone.height, rotation, reflect);
                let ep = format_endpoint(ne, no);
                let entry = by_channel.entry(channel_id).or_default();
                if is_input {
                    entry.0.push(ep);
                } else {
                    entry.1.push(ep);
                }
            }

            // (channel_id, channel_string) — keep the id alongside so we
            // can read the canonical position → id mapping after sorting.
            let mut channel_pairs: Vec<(u32, String)> = by_channel
                .into_iter()
                .map(|(channel_id, (mut ins, mut outs))| {
                    ins.sort_unstable();
                    outs.sort_unstable();
                    let reach = channel_reaches
                        .get(channel_id as usize)
                        .copied()
                        .unwrap_or(0);
                    (
                        channel_id,
                        format!("{}>{}@{}", ins.join("+"), outs.join("+"), reach),
                    )
                })
                .collect();
            // Sort by string only — the channel_id label itself is not
            // canonical (it's the SAT-encoder's allocation order). Tie-break
            // on channel_id so equal strings produce a deterministic order.
            channel_pairs.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

            let channel_order: Vec<u32> = channel_pairs.iter().map(|(id, _)| *id).collect();
            let channel_strs: Vec<String> =
                channel_pairs.into_iter().map(|(_, s)| s).collect();

            let mut fb: Vec<(u32, u32)> = base_forbidden
                .iter()
                .map(|&(x, y)| transform_tile(x, y, zone.width, zone.height, rotation, reflect))
                .collect();
            fb.sort_unstable();
            let fb_str: Vec<String> = fb.iter().map(|(x, y)| format!("{},{}", x, y)).collect();

            let signature = format!(
                "{}x{}:{}|F:{}|UG:{}",
                tw,
                th,
                channel_strs.join(";"),
                fb_str.join(";"),
                cap_str
            );

            candidates.push((signature, rotation, reflect, channel_order));
        }
    }

    // Pick the canonical: lex-min by signature, tie-break on (rotation, reflect)
    // so repeated calls on the same zone always produce the same orientation.
    candidates.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });

    let (signature, rotation, reflect, channel_order) = candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| (String::new(), 0, false, Vec::new()));

    CanonicalForm {
        signature,
        rotation,
        reflect,
        channel_order,
    }
}

/// Buffer one zone record for later flush. Encodes the JSONL line eagerly
/// so the lock-held critical section is just a `Vec::push`. Silently no-ops
/// on serialisation error — this is telemetry, not correctness.
///
/// Use `flush` (or `flush_on_drop`) to write the buffered records to disk.
pub fn record_zone(
    zone: &CrossingZone,
    channel_reaches: &[u32],
    max_ug_ins: Option<u32>,
    stats: ZoneStats,
    source: Option<&str>,
) {
    let thread_src = ZONE_SOURCE.with(|s| s.borrow().clone());
    let effective_source: Option<String> = thread_src
        .or_else(|| source.map(|s| s.to_string()))
        .or_else(|| std::env::var("FUCKTORIO_ZONE_SOURCE").ok());

    let signature = canonical_signature(zone, channel_reaches, max_ug_ins);

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let record = serde_json::json!({
        "ts": ts,
        "signature": signature,
        "width": zone.width,
        "height": zone.height,
        "variables": stats.variables,
        "clauses": stats.clauses,
        "solve_time_us": stats.solve_time_us,
        "source": effective_source,
    });

    let Ok(mut line) = serde_json::to_string(&record) else { return };
    line.push('\n');

    if let Ok(mut buf) = buffer().lock() {
        buf.push(PendingRecord { line: line.into_bytes() });
    }
}

/// Flush the buffered records to disk.
///
/// Each record is written in a single `write_all` call against an `O_APPEND`
/// file. Lines are kept short (well under the 4 KiB POSIX `PIPE_BUF`
/// guarantee) so concurrent processes appending to the same file produce
/// interleaved-but-intact lines, never torn ones.
///
/// Silently no-ops on any I/O error — losing telemetry is preferable to
/// failing the calling test or CLI invocation.
///
/// Safe to call any number of times; only the records buffered since the
/// previous call are written.
pub fn flush() {
    let pending: Vec<PendingRecord> = match buffer().lock() {
        Ok(mut buf) => std::mem::take(&mut *buf),
        Err(_) => return,
    };
    if pending.is_empty() {
        return;
    }

    let path = resolve_cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(mut f) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
    else {
        // Couldn't open — drop the buffer rather than retry-spinning. The
        // records are already lost from the in-memory buffer (we drained
        // it); accept that. A retry path would risk unbounded growth if
        // the disk is permanently unwritable.
        return;
    };

    for record in &pending {
        // O_APPEND + single write_all per line: kernel guarantees the
        // append is atomic for buffers up to PIPE_BUF (4 KiB on Linux).
        // Our lines are typically 200-400 bytes, so this is safe even
        // with concurrent writers from other processes.
        let _ = f.write_all(&record.line);
    }
}

/// Number of records currently buffered, awaiting flush. Useful for sizing
/// diagnostics ("how much will the next flush write?").
pub fn pending_count() -> usize {
    buffer().lock().map(|b| b.len()).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Solution cache — record + lookup
// ---------------------------------------------------------------------------

/// A cached SAT solution stored by canonical signature.
///
/// `entities` are in canonical-frame local coords: x ∈ [0, canon_w), y ∈ [0,
/// canon_h), and each entity's `carries` is either `None` or a channel-token
/// `"ch{N}"` referring to position N in `channel_items`. `segment_id` is
/// stripped (rewritten at replay time using the new zone's coords).
#[derive(Debug, Clone)]
struct CacheEntry {
    entities: Vec<PlacedEntity>,
    /// Items by canonical channel position. `channel_items[i]` is the item
    /// that flowed through canonical-channel-position `i` at record time.
    /// At lookup time we replace each `"ch{i}"` token with the item flowing
    /// through that same canonical position in the new zone.
    channel_items: Vec<String>,
    /// Canonical zone dimensions — needed for the inverse-transform call.
    canon_w: u32,
    canon_h: u32,
}

/// Lazy-loaded read-side cache. First lookup (re)reads the on-disk JSONL,
/// in-session writes append directly so cache hits work for repeats inside
/// the same run too.
static LOOKUP: OnceLock<Mutex<std::collections::HashMap<String, CacheEntry>>> = OnceLock::new();

fn lookup_table() -> &'static Mutex<std::collections::HashMap<String, CacheEntry>> {
    LOOKUP.get_or_init(|| {
        let mut map = std::collections::HashMap::new();
        load_existing_jsonl(&mut map);
        Mutex::new(map)
    })
}

/// Slurp the on-disk JSONL into the lookup map. Silently skips malformed or
/// pre-cache-schema records (no `entities` field). Newer records win on
/// duplicate keys (file is append-only, so later lines reflect later
/// behaviour).
fn load_existing_jsonl(map: &mut std::collections::HashMap<String, CacheEntry>) {
    let path = resolve_cache_path();
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return;
    };
    for line in contents.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        let Some(sig) = value["signature"].as_str() else { continue };
        let Some(entities_v) = value.get("entities") else { continue };
        let Ok(entities) = serde_json::from_value::<Vec<PlacedEntity>>(entities_v.clone()) else { continue };
        let Some(items_v) = value.get("channel_items") else { continue };
        let Ok(channel_items) = serde_json::from_value::<Vec<String>>(items_v.clone()) else { continue };
        let canon_w = value["canon_w"].as_u64().unwrap_or(0) as u32;
        let canon_h = value["canon_h"].as_u64().unwrap_or(0) as u32;
        if canon_w == 0 || canon_h == 0 {
            continue;
        }
        map.insert(
            sig.to_string(),
            CacheEntry { entities, channel_items, canon_w, canon_h },
        );
    }
}

/// Whether to consult the cache on lookup. Cache hits are gated behind an
/// env var while we're verifying parity — `record_zone_with_solution` always
/// records regardless. Set `FUCKTORIO_USE_ZONE_CACHE=1` to opt in.
fn cache_enabled() -> bool {
    std::env::var("FUCKTORIO_USE_ZONE_CACHE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Build a `channel_id → item` map from boundaries. Boundaries sharing a
/// channel_id always carry the same item, so we just take the first.
fn channel_items_by_id(zone: &CrossingZone) -> std::collections::HashMap<u32, String> {
    let mut map: std::collections::HashMap<u32, String> = Default::default();
    for b in &zone.boundaries {
        map.entry(b.channel_id).or_insert_with(|| b.item.clone());
    }
    map
}

/// Build the canonical-position-indexed item list using a `CanonicalForm`.
fn canonical_channel_items(zone: &CrossingZone, form: &CanonicalForm) -> Vec<String> {
    let by_id = channel_items_by_id(zone);
    form.channel_order
        .iter()
        .map(|id| by_id.get(id).cloned().unwrap_or_default())
        .collect()
}

/// Rewrite an entity's `carries` from a real item to a canonical channel
/// token. Returns the entity with `carries` replaced by `Some("chN")` if the
/// item is found in `channel_items`, or unchanged otherwise.
fn entity_carries_to_token(mut e: PlacedEntity, channel_items: &[String]) -> PlacedEntity {
    if let Some(item) = e.carries.as_deref() {
        if let Some(pos) = channel_items.iter().position(|i| i == item) {
            e.carries = Some(format!("ch{pos}"));
        }
    }
    e
}

/// Reverse of `entity_carries_to_token`: replace `"chN"` with the actual
/// item at that canonical position.
fn entity_carries_from_token(mut e: PlacedEntity, channel_items: &[String]) -> PlacedEntity {
    if let Some(token) = e.carries.as_deref() {
        if let Some(idx_str) = token.strip_prefix("ch") {
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(item) = channel_items.get(idx) {
                    e.carries = Some(item.clone());
                }
            }
        }
    }
    e
}

/// Record a SAT-solved zone with its solution entities for future lookup.
/// Extends [`record_zone`] by also storing the entities (rewritten to
/// canonical-frame local coords with channel-token `carries`).
///
/// `entities` should be the post-cost-descent SAT output, in absolute world
/// coords (as returned by `solve_crossing_zone_per_channel`). We strip the
/// world-coord origin and apply the canonical transform before storing.
pub fn record_zone_with_solution(
    zone: &CrossingZone,
    channel_reaches: &[u32],
    max_ug_ins: Option<u32>,
    stats: ZoneStats,
    entities: &[PlacedEntity],
    source: Option<&str>,
) {
    let form = canonicalise(zone, channel_reaches, max_ug_ins);
    let channel_items = canonical_channel_items(zone, &form);

    // Convert entities: absolute → zone-local → canonical-frame.
    let (canon_w, canon_h) = if form.rotation.is_multiple_of(2) {
        (zone.width, zone.height)
    } else {
        (zone.height, zone.width)
    };

    let canonical_entities: Vec<PlacedEntity> = entities
        .iter()
        .filter_map(|e| {
            let mut local = e.clone();
            local.x = e.x - zone.x;
            local.y = e.y - zone.y;
            // Skip entities that escaped the bbox — shouldn't happen but be
            // defensive (we'd produce out-of-bounds coords on transform).
            if local.x < 0
                || local.y < 0
                || (local.x as u32) >= zone.width
                || (local.y as u32) >= zone.height
            {
                return None;
            }
            let transformed = transform_entity(&local, zone.width, zone.height, form.rotation, form.reflect);
            // Strip segment_id — it's per-zone-instance, regenerated at lookup.
            let mut e = transformed;
            e.segment_id = None;
            Some(entity_carries_to_token(e, &channel_items))
        })
        .collect();

    // Update in-memory cache so same-session repeats hit.
    if let Ok(mut tbl) = lookup_table().lock() {
        tbl.insert(
            form.signature.clone(),
            CacheEntry {
                entities: canonical_entities.clone(),
                channel_items: channel_items.clone(),
                canon_w,
                canon_h,
            },
        );
    }

    // JSONL line.
    let thread_src = ZONE_SOURCE.with(|s| s.borrow().clone());
    let effective_source: Option<String> = thread_src
        .or_else(|| source.map(|s| s.to_string()))
        .or_else(|| std::env::var("FUCKTORIO_ZONE_SOURCE").ok());

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let record = serde_json::json!({
        "ts": ts,
        "signature": form.signature,
        "width": zone.width,
        "height": zone.height,
        "canon_w": canon_w,
        "canon_h": canon_h,
        "variables": stats.variables,
        "clauses": stats.clauses,
        "solve_time_us": stats.solve_time_us,
        "source": effective_source,
        "entities": canonical_entities,
        "channel_items": channel_items,
    });

    let Ok(mut line) = serde_json::to_string(&record) else { return };
    line.push('\n');

    if let Ok(mut buf) = buffer().lock() {
        buf.push(PendingRecord { line: line.into_bytes() });
    }
}

/// Look up a previously-cached SAT solution for `zone`. Returns `Some` of a
/// fresh `Vec<PlacedEntity>` in absolute world coords matching what
/// `solve_crossing_zone_per_channel` would have returned, or `None` on miss
/// or if the cache is disabled (env-gated).
///
/// The returned entities have `segment_id` set to `"crossing:{x}:{y}"` to
/// mirror the SAT path's tagging.
pub fn lookup_zone(
    zone: &CrossingZone,
    channel_reaches: &[u32],
    max_ug_ins: Option<u32>,
) -> Option<Vec<PlacedEntity>> {
    if !cache_enabled() {
        return None;
    }

    let form = canonicalise(zone, channel_reaches, max_ug_ins);
    let entry = {
        let tbl = lookup_table().lock().ok()?;
        tbl.get(&form.signature).cloned()?
    };

    let new_channel_items = canonical_channel_items(zone, &form);

    // Channel-count mismatch would mean either a hash collision or a stale
    // entry from a different encoder; bail rather than return wrong items.
    if new_channel_items.len() != entry.channel_items.len() {
        return None;
    }

    // Inverse-transform entities: canonical-frame → zone-local → absolute.
    let segment_id = format!("crossing:{}:{}", zone.x, zone.y);
    let entities: Vec<PlacedEntity> = entry
        .entities
        .into_iter()
        .map(|e| {
            let local = transform_entity_inverse(&e, entry.canon_w, entry.canon_h, form.rotation, form.reflect);
            let mut out = entity_carries_from_token(local, &new_channel_items);
            out.x += zone.x;
            out.y += zone.y;
            out.segment_id = Some(segment_id.clone());
            out
        })
        .collect();

    Some(entities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityDirection;
    use crate::sat::ZoneBoundary;

    fn boundary(x: i32, y: i32, dir: EntityDirection, channel_id: u32, is_input: bool) -> ZoneBoundary {
        ZoneBoundary {
            x,
            y,
            direction: dir,
            item: format!("ch{}", channel_id),
            is_input,
            interior: false,
            belt_tier: None,
            channel_id,
        }
    }

    #[test]
    fn parallel_vs_crossed_have_distinct_signatures() {
        // 3x3 zone at (0,0). Two channels.
        // Parallel: ch0 N1->S1, ch1 W1->E1
        let parallel = CrossingZone {
            x: 0, y: 0, width: 3, height: 3,
            boundaries: vec![
                boundary(1, 0, EntityDirection::South, 0, true),  // N1 in
                boundary(1, 2, EntityDirection::South, 0, false), // S1 out
                boundary(0, 1, EntityDirection::East, 1, true),   // W1 in
                boundary(2, 1, EntityDirection::East, 1, false),  // E1 out
            ],
            forced_empty: vec![],
        };
        // Crossed: ch0 N1->E1, ch1 W1->S1
        let crossed = CrossingZone {
            x: 0, y: 0, width: 3, height: 3,
            boundaries: vec![
                boundary(1, 0, EntityDirection::South, 0, true),  // N1 in
                boundary(2, 1, EntityDirection::East, 0, false),  // E1 out
                boundary(0, 1, EntityDirection::East, 1, true),   // W1 in
                boundary(1, 2, EntityDirection::South, 1, false), // S1 out
            ],
            forced_empty: vec![],
        };
        let reaches = [5, 5];
        let sp = canonical_signature(&parallel, &reaches, None);
        let sc = canonical_signature(&crossed, &reaches, None);
        assert_ne!(sp, sc, "parallel and crossed must have different signatures");
    }

    #[test]
    fn rotation_invariant() {
        // 3x3 with one channel N1->S1.
        let z1 = CrossingZone {
            x: 0, y: 0, width: 3, height: 3,
            boundaries: vec![
                boundary(1, 0, EntityDirection::South, 0, true),
                boundary(1, 2, EntityDirection::South, 0, false),
            ],
            forced_empty: vec![],
        };
        // Same shape rotated 90° CW: W1->E1.
        let z2 = CrossingZone {
            x: 0, y: 0, width: 3, height: 3,
            boundaries: vec![
                boundary(0, 1, EntityDirection::East, 0, true),
                boundary(2, 1, EntityDirection::East, 0, false),
            ],
            forced_empty: vec![],
        };
        let reaches = [5];
        assert_eq!(
            canonical_signature(&z1, &reaches, None),
            canonical_signature(&z2, &reaches, None)
        );
    }

    #[test]
    fn forbidden_tiles_co_transform() {
        // Same connectivity but different forbidden tile placements: should
        // produce different signatures.
        let mk = |fb: Vec<(i32, i32)>| CrossingZone {
            x: 0, y: 0, width: 3, height: 3,
            boundaries: vec![
                boundary(1, 0, EntityDirection::South, 0, true),
                boundary(1, 2, EntityDirection::South, 0, false),
            ],
            forced_empty: fb,
        };
        let za = mk(vec![(0, 1)]); // forbidden on left
        let zb = mk(vec![(2, 1)]); // forbidden on right — same under reflect though
        // These two are reflection-equivalent, so should match.
        assert_eq!(
            canonical_signature(&za, &[5], None),
            canonical_signature(&zb, &[5], None)
        );

        // But a forbidden tile at the center should be different from one on
        // an edge.
        let zc = mk(vec![(1, 1)]);
        assert_ne!(
            canonical_signature(&za, &[5], None),
            canonical_signature(&zc, &[5], None)
        );
    }

    #[test]
    fn ug_cap_in_key() {
        let z = CrossingZone {
            x: 0, y: 0, width: 3, height: 3,
            boundaries: vec![
                boundary(1, 0, EntityDirection::South, 0, true),
                boundary(1, 2, EntityDirection::South, 0, false),
            ],
            forced_empty: vec![],
        };
        assert_ne!(
            canonical_signature(&z, &[5], None),
            canonical_signature(&z, &[5], Some(0))
        );
    }

    fn ent(name: &str, x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: name.to_string(),
            x,
            y,
            direction: dir,
            ..Default::default()
        }
    }

    /// Round-trip: forward then inverse with the same (rot, refl) is identity
    /// for every D4 element. Tests every (rot ∈ 0..4) × (refl ∈ {false, true}).
    #[test]
    fn transform_roundtrip_identity() {
        let original = [
            ent("transport-belt", 0, 0, EntityDirection::East),
            ent("transport-belt", 2, 1, EntityDirection::North),
            ent("fast-underground-belt", 4, 2, EntityDirection::South),
            ent("transport-belt", 1, 3, EntityDirection::West),
        ];
        let (w, h) = (5u32, 4u32);

        for rot in 0u8..4 {
            for &refl in &[false, true] {
                // Forward: original → canonical frame.
                let (tw, th) = if rot.is_multiple_of(2) { (w, h) } else { (h, w) };
                let canonical: Vec<PlacedEntity> = original
                    .iter()
                    .map(|e| transform_entity(e, w, h, rot, refl))
                    .collect();

                // Sanity: every canonical entity sits inside the post-transform bbox.
                for e in &canonical {
                    assert!(e.x >= 0 && (e.x as u32) < tw, "rot={rot} refl={refl}: x={} out of bounds (tw={})", e.x, tw);
                    assert!(e.y >= 0 && (e.y as u32) < th, "rot={rot} refl={refl}: y={} out of bounds (th={})", e.y, th);
                }

                // Inverse: canonical → original frame.
                let recovered: Vec<PlacedEntity> = canonical
                    .iter()
                    .map(|e| transform_entity_inverse(e, tw, th, rot, refl))
                    .collect();

                for (orig, rec) in original.iter().zip(recovered.iter()) {
                    assert_eq!(
                        (orig.x, orig.y, orig.direction),
                        (rec.x, rec.y, rec.direction),
                        "rot={rot} refl={refl} roundtrip mismatch on {} at ({},{}) dir={:?}",
                        orig.name, orig.x, orig.y, orig.direction
                    );
                }
            }
        }
    }

    /// Specific orientation: 90°CW rotation should map (1, 0) North in a
    /// 3×3 zone to (2, 1) East in the rotated 3×3 zone.
    #[test]
    fn transform_90cw_specific() {
        let e = ent("transport-belt", 1, 0, EntityDirection::North);
        let t = transform_entity(&e, 3, 3, 1, false);
        assert_eq!((t.x, t.y), (2, 1), "position mapping wrong");
        assert_eq!(t.direction, EntityDirection::East, "direction mapping wrong");
    }
}
