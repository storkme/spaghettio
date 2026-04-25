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
use std::time::{SystemTime, UNIX_EPOCH};

// Thread-local source tag so parallel tests each carry their own label
// without stomping on each other via a process-global env var.
thread_local! {
    static ZONE_SOURCE: RefCell<Option<String>> = const { RefCell::new(None) };
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
/// rotations.
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
pub fn canonical_signature(
    zone: &CrossingZone,
    channel_reaches: &[u32],
    max_ug_ins: Option<u32>,
) -> String {
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

    let mut candidates: Vec<String> = Vec::with_capacity(8);

    for rotation in 0u8..4 {
        for &reflect in &[false, true] {
            // Transformed dimensions
            let (tw, th) = if rotation.is_multiple_of(2) {
                (zone.width, zone.height)
            } else {
                (zone.height, zone.width)
            };

            // Transform ports.
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

            // Build per-channel strings: sort endpoints inside each channel,
            // attach reach.
            let mut channel_strs: Vec<String> = by_channel
                .into_iter()
                .map(|(channel_id, (mut ins, mut outs))| {
                    ins.sort_unstable();
                    outs.sort_unstable();
                    let reach = channel_reaches
                        .get(channel_id as usize)
                        .copied()
                        .unwrap_or(0);
                    format!("{}>{}@{}", ins.join("+"), outs.join("+"), reach)
                })
                .collect();
            // Channel labels are not invariant — sort the channel strings lex
            // so two zones that differ only in channel_id assignment collapse.
            channel_strs.sort_unstable();

            // Transform forbidden tiles.
            let mut fb: Vec<(u32, u32)> = base_forbidden
                .iter()
                .map(|&(x, y)| transform_tile(x, y, zone.width, zone.height, rotation, reflect))
                .collect();
            fb.sort_unstable();
            let fb_str: Vec<String> = fb.iter().map(|(x, y)| format!("{},{}", x, y)).collect();

            candidates.push(format!(
                "{}x{}:{}|F:{}|UG:{}",
                tw,
                th,
                channel_strs.join(";"),
                fb_str.join(";"),
                cap_str
            ));
        }
    }

    candidates.into_iter().min().unwrap_or_default()
}

/// Append one zone record to the cache JSONL file. Silently no-ops on any
/// I/O error — telemetry, not correctness.
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

    let mut line = serde_json::to_string(&record).unwrap_or_default();
    line.push('\n');

    let path = resolve_cache_path();

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(mut f) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
    {
        let _ = f.write_all(line.as_bytes());
    }
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
}
