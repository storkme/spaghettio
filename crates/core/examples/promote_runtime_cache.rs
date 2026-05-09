//! Promote runtime SAT zone cache → embedded corpus.
//!
//! Reads `~/.cache/spaghettio/sat-zones.bin` (the file the solver writes to
//! at runtime), merges its entries with the existing embedded corpus at
//! `crates/core/data/sat-zones.bin`, deduplicates by canonical signature
//! (runtime entries overwrite embedded ones on collision), and writes the
//! merged result back to `crates/core/data/sat-zones.bin`.
//!
//! Usage:
//!   cargo run --manifest-path crates/core/Cargo.toml --example promote_runtime_cache
//!
//! Or to promote from a custom runtime cache path:
//!   SPAGHETTIO_ZONE_CACHE_PATH=/path/to/sat-zones.bin \
//!       cargo run --manifest-path crates/core/Cargo.toml --example promote_runtime_cache
//!
//! After promotion, commit `crates/core/data/sat-zones.bin` to embed the
//! expanded corpus in the next build.
//!
//! Regeneration recipe (start clean to maximise coverage):
//!   rm -f ~/.cache/spaghettio/sat-zones.bin
//!   cargo test --manifest-path crates/core/Cargo.toml
//!   cargo test --manifest-path crates/core/Cargo.toml --test science_gauntlet \
//!       -- --ignored --nocapture
//!   cargo run --manifest-path crates/core/Cargo.toml --example promote_runtime_cache

use spaghettio_core::zone_cache::{encode_record, parse_records, ENCODER_VERSION};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Resolve the runtime cache path using the same logic as `zone_cache`
/// (env override → XDG → HOME fallback).
fn runtime_cache_path() -> PathBuf {
    if let Ok(p) = std::env::var("SPAGHETTIO_ZONE_CACHE_PATH") {
        return PathBuf::from(p);
    }
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".cache"))
        })
        .unwrap_or_else(|| PathBuf::from(".cache"));
    base.join("spaghettio").join("sat-zones.bin")
}

fn main() {
    // Path to the embedded corpus (relative to Cargo manifest dir).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let embedded_path = PathBuf::from(&manifest_dir).join("data").join("sat-zones.bin");
    let runtime_path = runtime_cache_path();

    // -----------------------------------------------------------------------
    // Load embedded corpus
    // -----------------------------------------------------------------------
    let embedded_bytes = match std::fs::read(&embedded_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "warn: could not read embedded corpus at {}: {}",
                embedded_path.display(),
                e
            );
            Vec::new()
        }
    };
    let embedded_records = parse_records(&embedded_bytes);
    let embedded_count = embedded_records.len();
    println!(
        "embedded corpus: {} entries ({} bytes)",
        embedded_count,
        embedded_bytes.len()
    );

    // -----------------------------------------------------------------------
    // Load runtime cache
    // -----------------------------------------------------------------------
    let runtime_bytes = match std::fs::read(&runtime_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "error: could not read runtime cache at {}: {}",
                runtime_path.display(),
                e
            );
            eprintln!("Have you run the test suite first to warm the cache?");
            eprintln!(
                "  rm -f ~/.cache/spaghettio/sat-zones.bin && cargo test --manifest-path crates/core/Cargo.toml"
            );
            std::process::exit(1);
        }
    };
    let runtime_records = parse_records(&runtime_bytes);
    let runtime_count = runtime_records.len();
    println!(
        "runtime cache:   {} entries ({} bytes) at {}",
        runtime_count,
        runtime_bytes.len(),
        runtime_path.display()
    );

    // -----------------------------------------------------------------------
    // Merge: embedded first (baseline), runtime overwrites on collision.
    // Use BTreeMap<signature → record_index> so we can detect duplicates and
    // iterate in deterministic order for a reproducible output file.
    // -----------------------------------------------------------------------
    // We store the raw records by signature, newest wins.
    use spaghettio_core::zone_cache::DecodedRecord;
    let mut by_sig: BTreeMap<String, DecodedRecord> = BTreeMap::new();

    for rec in embedded_records {
        by_sig.insert(rec.signature.clone(), rec);
    }
    let mut new_entries = 0usize;
    let mut updated_entries = 0usize;
    for rec in runtime_records {
        if by_sig.contains_key(&rec.signature) {
            updated_entries += 1;
        } else {
            new_entries += 1;
        }
        by_sig.insert(rec.signature.clone(), rec);
    }

    let total = by_sig.len();
    println!(
        "merged:          {} total ({} new, {} updated)",
        total, new_entries, updated_entries
    );

    // -----------------------------------------------------------------------
    // Re-encode and write
    // -----------------------------------------------------------------------
    let mut out_bytes: Vec<u8> = Vec::with_capacity(embedded_bytes.len() + runtime_bytes.len());
    for rec in by_sig.values() {
        let tuples: Vec<[i32; 5]> = rec
            .entities
            .iter()
            .filter_map(|e| {
                let kind = match (e.name.as_str(), e.io_type.as_deref()) {
                    ("transport-belt", None | Some("passthrough")) => 0i32,
                    ("fast-transport-belt", None | Some("passthrough")) => 1,
                    ("express-transport-belt", None | Some("passthrough")) => 2,
                    ("underground-belt", Some("input")) => 3,
                    ("underground-belt", Some("output")) => 4,
                    ("fast-underground-belt", Some("input")) => 5,
                    ("fast-underground-belt", Some("output")) => 6,
                    ("express-underground-belt", Some("input")) => 7,
                    ("express-underground-belt", Some("output")) => 8,
                    _ => return None,
                };
                let dir = match e.direction {
                    spaghettio_core::models::EntityDirection::North => 0i32,
                    spaghettio_core::models::EntityDirection::East => 1,
                    spaghettio_core::models::EntityDirection::South => 2,
                    spaghettio_core::models::EntityDirection::West => 3,
                };
                let carries_idx = match e.carries.as_deref() {
                    None => -1i32,
                    Some(t) => t
                        .strip_prefix("ch")
                        .and_then(|n| n.parse::<i32>().ok())
                        .unwrap_or(-1),
                };
                Some([kind, e.x, e.y, dir, carries_idx])
            })
            .collect();

        encode_record(
            &mut out_bytes,
            rec.ts,
            &rec.signature,
            rec.source.as_deref(),
            rec.canon_w,
            rec.canon_h,
            rec.variables,
            rec.clauses,
            rec.solve_time_us,
            ENCODER_VERSION,
            &rec.channel_items,
            &tuples,
            &rec.outcome,
        );
    }

    match std::fs::write(&embedded_path, &out_bytes) {
        Ok(()) => {
            println!(
                "wrote {} entries ({} bytes) to {}",
                total,
                out_bytes.len(),
                embedded_path.display()
            );
        }
        Err(e) => {
            eprintln!("error: could not write {}: {}", embedded_path.display(), e);
            std::process::exit(1);
        }
    }

    // Summary for the PR
    let solved_after = by_sig.values().filter(|r| matches!(r.outcome, spaghettio_core::zone_cache::CachedOutcome::Solved)).count();
    let unsat_after  = by_sig.values().filter(|r| matches!(r.outcome, spaghettio_core::zone_cache::CachedOutcome::Unsat)).count();
    let timeo_after  = by_sig.values().filter(|r| matches!(r.outcome, spaghettio_core::zone_cache::CachedOutcome::Timeout{..})).count();
    println!();
    println!("=== promotion summary ===");
    println!(
        "  embedded before: {} entries ({} bytes)",
        embedded_count,
        embedded_bytes.len()
    );
    println!(
        "  runtime cache:   {} entries ({} bytes)",
        runtime_count,
        runtime_bytes.len()
    );
    println!(
        "  embedded after:  {} entries ({} bytes) — solved={} unsat={} timeout={}",
        total,
        out_bytes.len(),
        solved_after,
        unsat_after,
        timeo_after,
    );
    println!("  new entries added: {}", new_entries);
    println!("  entries updated:   {}", updated_entries);
}
