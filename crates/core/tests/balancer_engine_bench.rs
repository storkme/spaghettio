//! Bench-style comparison of placement engines across the (n, m) grid.
//!
//! Today there's only one real engine ([`LibraryLookup`]) and the
//! synth-as-baseline. Once Factorio-SAT and CP-SAT adapters land, they
//! plug into this same harness for side-by-side numbers.
//!
//! Reports per-shape: synth splitter count (from [`balancer::synth`]) vs
//! library splitter count (from [`bus::balancer_library`] via
//! [`LibraryLookup`]) — and which shapes the library is missing. The
//! synth-vs-library gap is the headline number we'll compare against
//! for new engines.

use std::time::Duration;

use fucktorio_core::balancer::placement::library_lookup::LibraryLookup;
use fucktorio_core::balancer::placement::{
    PlacementEngine, PlacementError, PlacementRequest,
};
use fucktorio_core::balancer::synth::synth;

#[test]
fn bench_library_vs_synth_for_n_m_up_to_10() {
    let engine = LibraryLookup;
    let timeout = Duration::from_secs(1);

    /// (entity_count, splitter_count, width, height) for a library entry.
    type LibraryRow = (u32, u32, u32, u32);
    let mut rows: Vec<(u32, u32, u32, Option<LibraryRow>)> = Vec::new();
    let mut library_missing: Vec<(u32, u32)> = Vec::new();

    for n in 1u32..=10 {
        for m in 1u32..=10 {
            let graph = synth(n, m).expect("synth covers 1..=10 × 1..=10");
            let synth_splitters = graph.n_splitters;

            let req = PlacementRequest {
                graph: &graph,
                n,
                m,
                timeout,
                seed: None,
            };
            let library = match engine.place(&req) {
                Ok(r) => Some((
                    r.template.entities.len() as u32,
                    splitter_count(&r.template) as u32,
                    r.template.width,
                    r.template.height,
                )),
                Err(PlacementError::ShapeNotAvailable { .. }) => {
                    library_missing.push((n, m));
                    None
                }
                Err(other) => panic!("library lookup failed for ({}, {}): {:?}", n, m, other),
            };
            rows.push((n, m, synth_splitters, library));
        }
    }

    eprintln!(
        "\n{:>5} {:>5} {:>10} {:>10} {:>10} {:>5} {:>5}  {}",
        "n", "m", "synth_S", "lib_S", "lib_E", "lib_W", "lib_H", "status"
    );
    for (n, m, synth_s, library) in &rows {
        // Status reflects what's *placeable* end-to-end, not just what
        // synth produces:
        //   - "library"      — `LibraryLookup` has a Factorio-SAT-baked
        //                       template; this shape is fully usable today.
        //   - "synth-only"   — synth produces a verified-balanced graph
        //                       but no placement engine can lay it out.
        //                       The CP-SAT placer (currently `(1, 1)` only)
        //                       is the path forward; until it lands more
        //                       shapes, these graphs are abstract-only.
        let status = if library.is_some() { "library" } else { "synth-only" };
        match library {
            Some((entities, splitters, w, h)) => eprintln!(
                "{:>5} {:>5} {:>10} {:>10} {:>10} {:>5} {:>5}  {}",
                n, m, synth_s, splitters, entities, w, h, status
            ),
            None => eprintln!(
                "{:>5} {:>5} {:>10} {:>10} {:>10} {:>5} {:>5}  {}",
                n, m, synth_s, "-", "-", "-", "-", status
            ),
        }
    }

    eprintln!("\nLibrary-missing shapes ({}):", library_missing.len());
    for (n, m) in &library_missing {
        eprintln!("  ({}, {}) — synth has {} splitters", n, m,
                  rows.iter().find(|r| r.0 == *n && r.1 == *m).unwrap().2);
    }

    // Splitter-count gap, descriptive only. Synth is a constructive
    // upper bound on splitter count under abstract all-fluid semantics;
    // the library is Factorio-SAT's choice under Factorio-realistic
    // constraints (belt cap, footprint, sideloading patterns). They
    // optimize different objectives, so one isn't strictly above the
    // other:
    //   - Library < Synth on shapes where SAT found a tighter splitter
    //     network within the spatial budget.
    //   - Library > Synth on shapes where SAT had to add splitters to
    //     satisfy belt-cap / backpressure constraints that the abstract
    //     model ignores (e.g., (n, 1) mergers — synth is one splitter,
    //     library is a tree that respects single-belt output rate).
    let mut max_lib_lower: i64 = 0;
    let mut max_synth_lower: i64 = 0;
    let mut covered = 0;
    for (_n, _m, synth_s, library) in &rows {
        if let Some((_, lib_s, _, _)) = library {
            let delta = *synth_s as i64 - *lib_s as i64;
            if delta > 0 {
                max_lib_lower = max_lib_lower.max(delta);
            } else {
                max_synth_lower = max_synth_lower.max(-delta);
            }
            covered += 1;
        }
    }
    eprintln!(
        "\nSplitter-count gap (over {} covered shapes):\n  \
         max where library is smaller: {}\n  \
         max where synth is smaller: {}",
        covered, max_lib_lower, max_synth_lower
    );
}

fn splitter_count(t: &fucktorio_core::balancer::placement::PlacedTemplate) -> usize {
    t.entities.iter().filter(|e| e.name == "splitter").count()
}
