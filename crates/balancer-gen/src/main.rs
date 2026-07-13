//! Phase 3.1 / 3.2 / 3.2A spike — drive a Python+OR-Tools placement
//! subprocess for splitter no-overlap (3.1) and belt routing (3.2A),
//! then verify via classify_ref.
//!
//! Three modes exercised here:
//!   - Mode A — splitter no-overlap only on `(2, 3)` and `(4, 9)` Clos
//!     topologies (legacy 3.1 spike).
//!   - Mode B — single-splitter end-to-end round-trip on `(1, 2)` and
//!     `(2, 2)` (3.2 MVP).
//!   - **Mode C — flow-conservation belt routing** on multi-splitter
//!     library shapes: takes splitter positions and directions from
//!     the library template, sends the topology to CP-SAT (which picks
//!     slot assignments as variables, phase 3.2A.2), reassembles the
//!     entity list, verifies via classify_ref.

use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use spaghettio_core::bus::balancer_classify::{
    classify_graph, classify_ref, topology_of_template, BalancerClass, BalancerTemplateRef,
    NodeId, SplitterGraph,
};
use spaghettio_core::bus::balancer_generate::OwnedTemplate;
use spaghettio_core::bus::balancer_library::{balancer_templates, BalancerTemplateEntity};
use spaghettio_core::bus::balancer_topology::{
    clos_interleave, library_atom, parallel, series_permuted,
};
use spaghettio_core::bus::template_validate::validate_template_lanes;
use spaghettio_core::validate::Severity;
// Aliased import — `parallel` is a graph-level combinator from
// balancer_topology; `compose_parallel` (defined locally) is the
// template-level combinator. Both are needed in the same fn for the
// (4, 9) Clos stress test.

// ---------------------------------------------------------------------------
// Protocol with `scripts/place.py`
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(untagged)]
enum PlaceRequest {
    OverlapOnly {
        n_splitters: usize,
        bounds: (u32, u32),
    },
    Routing {
        bounds: (u32, u32),
        splitter_positions: Vec<SplitterPosOut>,
        input_port_tiles: Vec<(i32, i32)>,
        output_port_tiles: Vec<(i32, i32)>,
        edges: Vec<EdgeReq>,
    },
    SynthPlace {
        bounds: (u32, u32),
        n_inputs: u32,
        n_outputs: u32,
        n_splitters: usize,
        edges: Vec<EdgeReq>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allow_dirs: Option<Vec<u8>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_time_s: Option<f64>,
    },
    PureRouting {
        kind: &'static str,
        bounds: (u32, u32),
        input_port_tiles: Vec<(i32, i32)>,
        output_port_tiles: Vec<(i32, i32)>,
        edges: Vec<EdgeReq>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_time_s: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        encoding: Option<&'static str>,
        /// UG reach in transit tiles (yellow=4, red=6, blue=8).
        /// `None` defaults to 4 (yellow) in the Python solver — identical to
        /// the previous hardcoded `UG_MAX_REACH = 5` (L_max_in_loop = reach + 1).
        #[serde(skip_serializing_if = "Option::is_none")]
        ug_max_reach: Option<u32>,
    },
}

#[derive(Serialize, Clone)]
struct SplitterPosOut {
    x: i32,
    y: i32,
    dir: u8,
}

#[derive(Serialize, Clone)]
struct EdgeReq {
    src_kind: &'static str,
    src_idx: usize,
    dst_kind: &'static str,
    dst_idx: usize,
}

#[derive(Deserialize, Debug)]
struct PlaceResponse {
    status: String,
    elapsed_s: f64,
    #[serde(default)]
    splitters: Vec<SplitterPos>,
    #[serde(default)]
    belts: Vec<BeltOutput>,
    #[serde(default)]
    ugs: Vec<UgOutput>,
    #[serde(default)]
    input_port_tiles: Vec<[i32; 2]>,
    #[serde(default)]
    output_port_tiles: Vec<[i32; 2]>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // edge_idx exposed for future debugging
struct UgOutput {
    x: i32,
    y: i32,
    dir: u8,
    io: String, // "input" or "output"
    edge_idx: usize,
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct SplitterPos {
    x: i32,
    y: i32,
    #[serde(default)]
    dir: u8,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[allow(dead_code)] // edge_idx exposed for future debugging / merging
struct BeltOutput {
    x: i32,
    y: i32,
    dir: u8,
    edge_idx: usize,
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("SPAGHETTIO_DEBUG_2_2").is_ok() {
        println!("=== phase 4.4 debug: (2, 2) Clos via compose_* ===");
        return debug_compose_clos_2_2();
    }
    if std::env::var("SPAGHETTIO_DEBUG_4_9").is_ok() {
        println!("=== phase 4.4: (4, 9) Clos via compose_* ===");
        return stress_compose_clos_4_9();
    }
    if std::env::var("SPAGHETTIO_BAKE_1_9").is_ok() {
        println!("=== phase 3.4 spike: (1, 9) via compose + codegen ===");
        return bake_compose_1_9();
    }
    if std::env::var("SPAGHETTIO_BAKE_BATCH").is_ok() {
        println!("=== phase 3.4: bake missing shapes ===");
        return bake_missing_shapes();
    }
    if let Ok(mode_d_value) = std::env::var("SPAGHETTIO_MODE_D_ONLY") {
        // Fast path: skip phases 3.1, 3.2 (round-trips), 3.2A/B/C
        // (flow-conservation routing) and run only phase 3.2D (Mode D
        // synth-place). Used to validate Mode D constraints in
        // isolation without paying for the upstream spike phases.
        //
        // Value formats:
        //   - "1" / unset-but-present: run a curated set of small shapes.
        //   - "all": run every shape currently in `balancer_templates()`.
        //   - "(n,m);(n2,m2);...": run only the listed shapes.
        println!("=== Mode D-only: skipping earlier spike phases ===");

        let curated: Vec<(String, (u32, u32))> = vec![
            ("(1, 2)".into(), (1, 2)),
            ("(2, 2)".into(), (2, 2)),
            ("(2, 4)".into(), (2, 4)),
            ("(1, 4)".into(), (1, 4)),
            ("(4, 4)".into(), (4, 4)),
            ("(1, 8)".into(), (1, 8)),
            ("(5, 1)".into(), (5, 1)),
        ];

        let shapes: Vec<(String, (u32, u32))> = match mode_d_value.as_str() {
            "" | "1" => curated,
            "all" => {
                let mut shapes: Vec<(u32, u32)> =
                    balancer_templates().keys().copied().collect();
                shapes.sort();
                shapes
                    .into_iter()
                    .map(|s| (format!("({}, {})", s.0, s.1), s))
                    .collect()
            }
            spec => spec
                .split(';')
                .filter_map(|tok| {
                    let nums: Vec<u32> = tok
                        .trim_matches(|c: char| !c.is_ascii_digit())
                        .split(|c: char| !c.is_ascii_digit())
                        .filter(|p| !p.is_empty())
                        .filter_map(|p| p.parse().ok())
                        .collect();
                    if nums.len() == 2 {
                        Some((format!("({}, {})", nums[0], nums[1]), (nums[0], nums[1])))
                    } else {
                        None
                    }
                })
                .collect(),
        };

        if shapes.is_empty() {
            return Err(format!(
                "SPAGHETTIO_MODE_D_ONLY={mode_d_value:?} parsed to zero shapes"
            )
            .into());
        }

        for (label, shape) in &shapes {
            match spike_synth_place(label, *shape, None) {
                Ok(()) => {}
                Err(e) => println!("  ✗ {label}: {e}"),
            }
        }
        return Ok(());
    }
    let templates = balancer_templates();

    // Mode A — phase 3.1 splitter no-overlap.
    let small_template = templates.get(&(2u32, 3u32)).ok_or("library missing (2, 3)")?;
    let small_topology = topology_of_template(BalancerTemplateRef::from(small_template))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;
    spike_run_overlap_only(
        "(2, 3) library round-trip [overlap only]",
        &small_topology,
        (small_template.width, small_template.height),
    )?;

    let stage1 = parallel(&library_atom(1, 3).ok_or("library (1, 3) missing")?, 4);
    let stage2 = parallel(&library_atom(4, 3).ok_or("library (4, 3) missing")?, 3);
    let big = series_permuted(&stage1, &stage2, &clos_interleave(4, 3));
    let report = classify_graph(&big).map_err(|e| format!("classify_graph: {e:?}"))?;
    if report.class != BalancerClass::Balanced {
        return Err(format!("(4, 9) Clos expected MX3, got {:?}", report.class).into());
    }
    spike_run_overlap_only("(4, 9) Clos composition [overlap only]", &big, (24, 24))?;

    // Mode B — single-splitter MVP round-trip.
    println!("\n=== phase 3.2 MVP: single-splitter end-to-end round-trip ===");
    spike_round_trip("(1, 2)", (1, 2))?;
    spike_round_trip("(2, 2)", (2, 2))?;

    // Mode C — phase 3.2A / 3.2B / 3.2C multi-splitter belt routing.
    // CP-SAT picks slot assignments as variables (3.2A.2). Continues past
    // failures so we can see the coverage matrix.
    println!("\n=== phase 3.2A / 3.2B / 3.2C: flow-conservation belt routing ===");
    let mut routing_results: Vec<(String, Result<(), String>)> = Vec::new();
    for &(label, shape) in &[
        ("(2, 2)", (2u32, 2u32)),
        ("(2, 4)", (2, 4)),
        ("(1, 4)", (1, 4)),
        ("(1, 3)", (1, 3)),
        ("(2, 3)", (2, 3)),
        ("(4, 8)", (4, 8)),
        ("(3, 5)", (3, 5)),
        ("(5, 3)", (5, 3)),
    ] {
        let result = spike_routing_round_trip(label, shape).map_err(|e| e.to_string());
        if result.is_err() {
            println!("  ✗ {label}: {}", result.as_ref().err().unwrap());
        }
        routing_results.push((label.to_string(), result));
    }
    let pass = routing_results.iter().filter(|(_, r)| r.is_ok()).count();
    let total = routing_results.len();
    println!("\nrouting round-trip: {pass}/{total} pass");
    for (label, r) in &routing_results {
        let icon = if r.is_ok() { "✓" } else { "✗" };
        println!("  {icon} {label}");
    }
    if pass == 0 {
        return Err("no routing round-trips passed".into());
    }

    // Mode D — phase 3.2D synth-place at fixed bbox. CP-SAT picks
    // splitter positions, IO port columns, slot assignments, and belt +
    // UG routing all in one shot, with a minimum-entity-count objective
    // to keep paths tight. All-south splitters; UGs restricted to south.
    println!("\n=== phase 3.2D: synth-place at fixed bbox ===");
    let mut synth_results: Vec<(String, Result<(), String>)> = Vec::new();
    println!("--- default: south-only splitters ---");
    for &(label, shape) in &[
        ("(1, 2)", (1u32, 2u32)),
        ("(2, 2)", (2, 2)),
        ("(2, 4)", (2, 4)),
        ("(1, 4)", (1, 4)),
        ("(4, 4)", (4, 4)),
        ("(1, 8)", (1, 8)),
    ] {
        let result = spike_synth_place(label, shape, None).map_err(|e| e.to_string());
        if result.is_err() {
            println!("  ✗ {label}: {}", result.as_ref().err().unwrap());
        }
        synth_results.push((label.to_string(), result));
    }
    println!("\n--- 3.2D.3: full direction freedom ---");
    {
        let label = "(1, 3)";
        let shape = (1u32, 3u32);
        let result =
            spike_synth_place(label, shape, Some(vec![0u8, 2, 4, 6])).map_err(|e| e.to_string());
        if result.is_err() {
            println!("  ✗ {label}: {}", result.as_ref().err().unwrap());
        }
        synth_results.push((label.to_string(), result));
    }
    let synth_pass = synth_results.iter().filter(|(_, r)| r.is_ok()).count();
    let synth_total = synth_results.len();
    println!("\nsynth-place: {synth_pass}/{synth_total} pass");
    for (label, r) in &synth_results {
        let icon = if r.is_ok() { "✓" } else { "✗" };
        println!("  {icon} {label}");
    }

    // Phase 3.3 — bench Mode D vs library on south-only shapes.
    // For each shape: solve at library bbox, then try shrinking by 1 row
    // and by 1 col to see whether Mode D can find tighter than library.
    println!("\n=== phase 3.3: bench Mode D vs library ===");
    let bench_shapes: &[(u32, u32)] = &[
        (1, 2), (2, 2), (2, 4), (1, 4), (4, 4), (1, 8),
    ];
    let mut rows: Vec<BenchRow> = Vec::new();
    for &shape in bench_shapes {
        match bench_synth_place(shape) {
            Ok(row) => rows.push(row),
            Err(e) => println!("  ✗ {shape:?}: {e}"),
        }
    }
    println!();
    println!(
        "  {:<8} | {:<8} {:>8} | {:>8} {:>9} | {:>8} {:>10} {:>10}",
        "shape", "lib_bbox", "lib_ents", "modeD_e", "solve_s", "min_bbox", "min_area", "shrink_s"
    );
    println!("  {:-<86}", "");
    for r in &rows {
        let lib_bbox = format!("{}×{}", r.lib_w, r.lib_h);
        let min_bbox = format!("{}×{}", r.min_w, r.min_h);
        let lib_area = r.lib_w * r.lib_h;
        let min_area = r.min_w * r.min_h;
        let area_str = if min_area < lib_area {
            format!("{} (-{})", min_area, lib_area - min_area)
        } else {
            format!("{}", min_area)
        };
        println!(
            "  {:<8} | {:<8} {:>8} | {:>8} {:>7.2}s | {:>8} {:>10} {:>8.1}s",
            format!("{:?}", r.shape),
            lib_bbox,
            r.lib_entities,
            r.mode_d_entities,
            r.mode_d_solve_s,
            min_bbox,
            area_str,
            r.shrink_total_s,
        );
    }

    // Phase 4.1 — compose_parallel smoke test.
    println!("\n=== phase 4.1: compose_parallel ===");
    if let Err(e) = smoke_compose_parallel() {
        println!("  ✗ smoke test: {e}");
    }

    // Phase 4.4 debug — minimal Clos compose reproducer for the
    // Singular-classification bug seen on the (4, 9) case. Same composition
    // pattern (parallel + clos_interleave + parallel) but only 4 splitters,
    // junction routing trivial.
    println!("\n=== phase 4.4 debug: (2, 2) Clos via compose_* ===");
    if let Err(e) = debug_compose_clos_2_2() {
        println!("  ✗ debug compose: {e}");
    }

    // Phase 4.4 — (4, 9) Clos via composition combinator.
    // Same topology as the phase 3.3 stress test (33 splitters, 67 edges)
    // that OOM'd Mode D, but built by composing library atoms with the
    // pure-routing junction in between.
    println!("\n=== phase 4.4: (4, 9) Clos via compose_parallel + compose_series ===");
    if let Err(e) = stress_compose_clos_4_9() {
        println!("  ✗ compose stress: {e}");
    }

    // Phase 3.3 stress test — (4, 9) Clos composition.
    // 33 splitters, 67 edges. Library doesn't have (4, 9). Topology
    // built from existing series_permuted(parallel(library_atom(1, 3),
    // 4), parallel(library_atom(4, 3), 3), clos_interleave(4, 3)) and
    // verified MX3 by classify_graph above. Tests whether Mode D can
    // place a 30+ splitter graph at all — answers the open question
    // gating phase 3.4 (bake into library_extra).
    println!("\n=== phase 3.3 stress: (4, 9) Clos composition ===");
    if let Err(e) = stress_clos_4_9() {
        println!("  ✗ stress test: {e}");
    }

    println!("\n✓ all spike runs passed");
    Ok(())
}

fn stress_clos_4_9() -> Result<(), Box<dyn std::error::Error>> {
    let stage1 = parallel(&library_atom(1, 3).ok_or("library (1, 3) missing")?, 4);
    let stage2 = parallel(&library_atom(4, 3).ok_or("library (4, 3) missing")?, 3);
    let big = series_permuted(&stage1, &stage2, &clos_interleave(4, 3));
    let report = classify_graph(&big).map_err(|e| format!("classify_graph: {e:?}"))?;
    if report.class != BalancerClass::Balanced {
        return Err(format!("(4, 9) Clos expected MX3, got {:?}", report.class).into());
    }
    println!(
        "  topology: {} splitters, {} edges (classified {:?})",
        big.n_splitters,
        big.edges.len(),
        report.class
    );

    let timeout = 600.0_f64;

    // Try increasingly tight bboxes. Mode D's model size is O(W*H*E)
    // for routing arcs alone — at 24×24 with 67 edges the model OOMs
    // during construction. Walk down from synth's bbox heuristic.
    let bbox_candidates: &[(u32, u32)] = &[(16, 16), (12, 18), (10, 20), (9, 24)];

    for &bbox in bbox_candidates {
        println!(
            "  attempting Mode D at {}×{} with {:.0}s budget...",
            bbox.0, bbox.1, timeout
        );
        match try_clos_at(&big, bbox, timeout) {
            Ok(()) => {
                println!("  ✓ (4, 9) Clos placed at {}×{}", bbox.0, bbox.1);
                return Ok(());
            }
            Err(e) => println!("    {e}"),
        }
    }
    Err("(4, 9) Clos failed at every candidate bbox".into())
}

fn try_clos_at(
    big: &SplitterGraph,
    bbox: (u32, u32),
    timeout: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let edge_reqs = build_edge_reqs(big)?;

    let req = PlaceRequest::SynthPlace {
        bounds: bbox,
        n_inputs: 4,
        n_outputs: 9,
        n_splitters: big.n_splitters,
        edges: edge_reqs,
        allow_dirs: None,
        max_time_s: Some(timeout),
    };
    let resp = run_solver(&req)?;
    println!(
        "  status={} elapsed={:.2}s splitters={} belts={} ugs={}",
        resp.status,
        resp.elapsed_s,
        resp.splitters.len(),
        resp.belts.len(),
        resp.ugs.len()
    );

    if resp.status != "OPTIMAL" && resp.status != "FEASIBLE" {
        return Err(format!(
            "(4, 9) Clos returned {} at {bbox:?} after {:.1}s",
            resp.status, resp.elapsed_s
        )
        .into());
    }

    println!(
        "  → {} entities total in {}×{} bbox",
        resp.splitters.len() + resp.belts.len() + resp.ugs.len(),
        bbox.0,
        bbox.1
    );
    Ok(())
}

struct BenchRow {
    shape: (u32, u32),
    lib_w: u32,
    lib_h: u32,
    lib_entities: usize,
    mode_d_entities: usize,
    mode_d_solve_s: f64,
    min_w: u32,
    min_h: u32,
    shrink_total_s: f64,
}

fn bench_synth_place(shape: (u32, u32)) -> Result<BenchRow, Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let lib = templates
        .get(&shape)
        .ok_or_else(|| format!("library missing {shape:?}"))?;
    let topology = topology_of_template(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;
    let lib_entities = lib.entities.len();

    // Run at library bbox.
    let lib_resp = solve_synth(&topology, shape, (lib.width, lib.height))?;
    let mode_d_entities = lib_resp.splitters.len() + lib_resp.belts.len() + lib_resp.ugs.len();
    let mode_d_solve_s = lib_resp.elapsed_s;

    // Shrink loop: greedy linear shrink. Try -1 height, then -1 width, repeat
    // until both shrinks fail.
    let (mut min_w, mut min_h) = (lib.width, lib.height);
    let shrink_t0 = std::time::Instant::now();
    loop {
        let mut shrank = false;
        if min_h > 3 {
            let candidate = (min_w, min_h - 1);
            if solve_synth(&topology, shape, candidate).is_ok() {
                min_h -= 1;
                shrank = true;
            }
        }
        if min_w > std::cmp::max(shape.0, shape.1) {
            let candidate = (min_w - 1, min_h);
            if solve_synth(&topology, shape, candidate).is_ok() {
                min_w -= 1;
                shrank = true;
            }
        }
        if !shrank {
            break;
        }
    }
    let shrink_total_s = shrink_t0.elapsed().as_secs_f64();

    Ok(BenchRow {
        shape,
        lib_w: lib.width,
        lib_h: lib.height,
        lib_entities,
        mode_d_entities,
        mode_d_solve_s,
        min_w,
        min_h,
        shrink_total_s,
    })
}

fn solve_synth(
    topology: &SplitterGraph,
    shape: (u32, u32),
    bbox: (u32, u32),
) -> Result<PlaceResponse, Box<dyn std::error::Error>> {
    let edge_reqs = build_edge_reqs(topology)?;
    let req = PlaceRequest::SynthPlace {
        bounds: bbox,
        n_inputs: shape.0,
        n_outputs: shape.1,
        n_splitters: topology.n_splitters,
        edges: edge_reqs,
        allow_dirs: None,
        max_time_s: None,
    };
    let resp = run_solver(&req)?;
    if resp.status != "OPTIMAL" && resp.status != "FEASIBLE" {
        return Err(format!("status {} at {:?}", resp.status, bbox).into());
    }
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Mode A — splitter no-overlap only
// ---------------------------------------------------------------------------

fn spike_run_overlap_only(
    label: &str,
    topology: &SplitterGraph,
    bounds: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n=== {label} ===  topology: {} splitters, {} edges  bounds: {}×{}",
        topology.n_splitters,
        topology.edges.len(),
        bounds.0,
        bounds.1
    );
    let req = PlaceRequest::OverlapOnly {
        n_splitters: topology.n_splitters,
        bounds,
    };
    let resp = run_solver(&req)?;
    println!(
        "status: {}  elapsed: {:.3}s  placements: {}",
        resp.status,
        resp.elapsed_s,
        resp.splitters.len()
    );
    enforce_kill_criteria(&resp)?;
    if resp.splitters.len() != topology.n_splitters {
        return Err(format!(
            "got {} placements, expected {}",
            resp.splitters.len(),
            topology.n_splitters
        )
        .into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Mode B — single-splitter MVP
// ---------------------------------------------------------------------------

fn spike_round_trip(
    label: &str,
    shape: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let lib = templates
        .get(&shape)
        .ok_or_else(|| format!("library missing {shape:?}"))?;
    let topology = topology_of_template(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;
    if topology.n_splitters != 1 {
        return Err(format!(
            "{label}: MVP only handles single-splitter shapes (got {})",
            topology.n_splitters
        )
        .into());
    }

    println!(
        "\n--- {label} ---  bounds: {}×{}  topology: {} splitters, {} edges",
        lib.width,
        lib.height,
        topology.n_splitters,
        topology.edges.len()
    );
    let req = PlaceRequest::OverlapOnly {
        n_splitters: topology.n_splitters,
        bounds: (lib.width, lib.height),
    };
    let resp = run_solver(&req)?;
    enforce_kill_criteria(&resp)?;
    let sp = resp.splitters[0];
    let placed = emit_single_splitter_template(shape, sp, (lib.width, lib.height))?;

    let class_report =
        classify_ref(placed.as_ref()).map_err(|e| format!("classify_ref: {e:?}"))?;
    if class_report.class != BalancerClass::Balanced {
        return Err(format!(
            "{label}: round-trip class = {:?}, expected Balanced",
            class_report.class
        )
        .into());
    }
    println!(
        "  splitter ({}, {})  → {} entities, classified {:?} ✓",
        sp.x,
        sp.y,
        placed.entities.len(),
        class_report.class
    );
    Ok(())
}

fn emit_single_splitter_template(
    shape: (u32, u32),
    sp: SplitterPos,
    bounds: (u32, u32),
) -> Result<OwnedTemplate, String> {
    let (m, n) = shape;
    if m > 2 || n > 2 {
        return Err(format!("MVP only handles single-splitter shapes; got {shape:?}"));
    }
    let input_slots = [(sp.x, sp.y - 1), (sp.x + 1, sp.y - 1)];
    let output_slots = [(sp.x, sp.y + 1), (sp.x + 1, sp.y + 1)];

    let mut entities: Vec<BalancerTemplateEntity> = Vec::new();
    let mut input_tiles: Vec<(i32, i32)> = Vec::new();
    let mut output_tiles: Vec<(i32, i32)> = Vec::new();

    entities.push(BalancerTemplateEntity {
        name: "splitter",
        x: sp.x,
        y: sp.y,
        direction: 4,
        io_type: None, input_priority: None, output_priority: None,
    });
    for &slot in input_slots.iter().take(m as usize) {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: slot.0,
            y: slot.1,
            direction: 4,
            io_type: None, input_priority: None, output_priority: None,
        });
        input_tiles.push(slot);
    }
    for &slot in output_slots.iter().take(n as usize) {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: slot.0,
            y: slot.1,
            direction: 4,
            io_type: None, input_priority: None, output_priority: None,
        });
        output_tiles.push(slot);
    }
    Ok(OwnedTemplate {
        n_inputs: m,
        n_outputs: n,
        width: bounds.0,
        height: bounds.1,
        entities,
        input_tiles,
        output_tiles,
    })
}

// ---------------------------------------------------------------------------
// Mode C — phase 3.2A.1 flow-conservation routing
// ---------------------------------------------------------------------------

fn spike_routing_round_trip(
    label: &str,
    shape: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let lib = templates
        .get(&shape)
        .ok_or_else(|| format!("library missing {shape:?}"))?;
    let topology = topology_of_template(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;

    // 3.2C handles all four splitter directions plus UGs.
    let splitter_positions: Vec<SplitterPosOut> = lib
        .entities
        .iter()
        .filter(|e| e.name == "splitter")
        .map(|e| SplitterPosOut {
            x: e.x,
            y: e.y,
            dir: e.direction,
        })
        .collect();
    if splitter_positions.len() != topology.n_splitters {
        return Err(format!(
            "{label}: lib has {} splitters but topology has {}",
            splitter_positions.len(),
            topology.n_splitters
        )
        .into());
    }

    let edge_reqs = build_edge_reqs(&topology)?;

    let req = PlaceRequest::Routing {
        bounds: (lib.width, lib.height),
        splitter_positions: splitter_positions.clone(),
        input_port_tiles: lib.input_tiles.to_vec(),
        output_port_tiles: lib.output_tiles.to_vec(),
        edges: edge_reqs,
    };

    println!(
        "\n--- {label} ---  bounds: {}×{}  splitters: {}  edges: {}",
        lib.width,
        lib.height,
        splitter_positions.len(),
        topology.edges.len()
    );
    let resp = run_solver(&req)?;
    enforce_kill_criteria(&resp)?;

    println!(
        "  routing: status={} elapsed={:.3}s belts={}",
        resp.status,
        resp.elapsed_s,
        resp.belts.len()
    );

    let placed = assemble_template_from_routing(
        shape, lib, &splitter_positions, &resp.belts, &resp.ugs,
    )?;
    let class_report = classify_ref(placed.as_ref())
        .map_err(|e| format!("{label}: classify_ref on assembled template: {e:?}"))?;

    let original = classify_ref(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("{label}: classify original library template: {e:?}"))?;
    if class_report.class != original.class {
        return Err(format!(
            "{label}: assembled class {:?} differs from library class {:?}",
            class_report.class, original.class
        )
        .into());
    }
    println!(
        "  assembled {} entities, classified {:?} (matches library) ✓",
        placed.entities.len(),
        class_report.class
    );
    Ok(())
}

/// Build per-edge requests for the CP-SAT solver. No slot info — the
/// solver picks slot assignments as variables (phase 3.2A.2).
fn build_edge_reqs(
    topology: &SplitterGraph,
) -> Result<Vec<EdgeReq>, Box<dyn std::error::Error>> {
    let mut edge_reqs = Vec::with_capacity(topology.edges.len());
    for (a, b) in &topology.edges {
        let (src_kind, src_idx) = match a {
            NodeId::InputPort(i) => ("InputPort", *i),
            NodeId::Splitter(s) => ("Splitter", *s),
            NodeId::OutputPort(_) => {
                return Err("invalid: edge sourced from OutputPort".into())
            }
        };
        let (dst_kind, dst_idx) = match b {
            NodeId::OutputPort(j) => ("OutputPort", *j),
            NodeId::Splitter(s) => ("Splitter", *s),
            NodeId::InputPort(_) => {
                return Err("invalid: edge destined for InputPort".into())
            }
        };
        edge_reqs.push(EdgeReq {
            src_kind,
            src_idx,
            dst_kind,
            dst_idx,
        });
    }
    Ok(edge_reqs)
}

// ---------------------------------------------------------------------------
// Mode D — phase 3.2D.1 synth-place
// ---------------------------------------------------------------------------

fn spike_synth_place(
    label: &str,
    shape: (u32, u32),
    allow_dirs: Option<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let lib = templates
        .get(&shape)
        .ok_or_else(|| format!("library missing {shape:?}"))?;
    let topology = topology_of_template(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;
    let edge_reqs = build_edge_reqs(&topology)?;

    let label_suffix = match &allow_dirs {
        Some(_) => " [4-dir]",
        None => "",
    };
    // Optional bbox slack: expand width and height by N (each) so Mode D
    // has more room. Used by `scripts/overnight_mode_d_bench.sh` to retry
    // after an INFEASIBLE at library bbox. Library entries (especially
    // Factorio-SAT-generated ones) are often very tight; Mode D's
    // current encoding may need 1-2 cells of slack to find a placement.
    let slack: u32 = std::env::var("SPAGHETTIO_MODE_D_BBOX_SLACK")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let bounds = (lib.width + slack, lib.height + slack);
    // Optional solver budget override (seconds). Bumps the Python CP-SAT
    // `max_time_in_seconds` and matches it against the Rust-side kill
    // check below. Default leaves both at their hard-coded values.
    let budget_override: Option<f64> = std::env::var("SPAGHETTIO_MODE_D_BUDGET_S")
        .ok()
        .and_then(|v| v.parse().ok());
    let req = PlaceRequest::SynthPlace {
        bounds,
        n_inputs: shape.0,
        n_outputs: shape.1,
        n_splitters: topology.n_splitters,
        edges: edge_reqs,
        allow_dirs,
        max_time_s: budget_override,
    };

    let slack_suffix = if slack > 0 {
        format!(" [+{slack} slack]")
    } else {
        String::new()
    };
    println!(
        "\n--- {label}{label_suffix}{slack_suffix} ---  bbox: {}×{}  splitters: {}  edges: {}",
        bounds.0,
        bounds.1,
        topology.n_splitters,
        topology.edges.len()
    );
    let resp = run_solver(&req)?;
    let default_budget = if label_suffix.is_empty() { 60.0 } else { 120.0 };
    let budget = budget_override.unwrap_or(default_budget);
    if resp.elapsed_s > budget {
        return Err(format!("kill: solve time {:.1}s > {}s", resp.elapsed_s, budget).into());
    }
    if resp.status != "OPTIMAL" && resp.status != "FEASIBLE" {
        return Err(format!("solver returned {} (no placement)", resp.status).into());
    }

    println!(
        "  synth-place: status={} elapsed={:.3}s splitters={} belts={} ugs={}",
        resp.status,
        resp.elapsed_s,
        resp.splitters.len(),
        resp.belts.len(),
        resp.ugs.len(),
    );

    let splitter_positions: Vec<SplitterPosOut> = resp
        .splitters
        .iter()
        .map(|sp| SplitterPosOut {
            x: sp.x,
            y: sp.y,
            dir: sp.dir,
        })
        .collect();
    let placed = assemble_template_from_routing(
        shape,
        lib,
        &splitter_positions,
        &resp.belts,
        &resp.ugs,
    )?;
    // Override IO tiles with what the solver picked.
    let placed = OwnedTemplate {
        n_inputs: placed.n_inputs,
        n_outputs: placed.n_outputs,
        width: placed.width,
        height: placed.height,
        entities: placed.entities,
        input_tiles: resp
            .input_port_tiles
            .iter()
            .map(|t| (t[0], t[1]))
            .collect(),
        output_tiles: resp
            .output_port_tiles
            .iter()
            .map(|t| (t[0], t[1]))
            .collect(),
    };

    let class_report = classify_ref(placed.as_ref())
        .map_err(|e| format!("{label}: classify_ref on synth-placed template: {e:?}"))?;
    let original = classify_ref(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("{label}: classify original library template: {e:?}"))?;
    // Bus minimum is MX2a (ThroughputBalancedRate). Library entries are typically MX3
    // because the seed corpus was Raynquist-derived, but Mode D's CP-SAT model only
    // enforces saturated rate balance — accepting any class at or above the bus
    // requirement avoids rejecting correct-for-bus solutions purely on composition.
    if !class_acceptable_for_bus(class_report.class) {
        return Err(format!(
            "{label}: synth-placed class {:?} is below bus minimum (TBR or stronger required); library was {:?}",
            class_report.class, original.class
        )
        .into());
    }
    let comparison = if class_report.class == original.class {
        "matches library".to_string()
    } else {
        format!("library was {:?}, accepted as ≥ TBR", original.class)
    };
    println!(
        "  ✓ {label}: classified {:?} ({comparison}); IO {:?}→{:?}",
        class_report.class, resp.input_port_tiles, resp.output_port_tiles,
    );
    Ok(())
}

/// MX2a (saturated + balanced rate) is the minimum class needed for spaghettio's
/// homogeneous-row bus. MX2b and MX3 are stronger and also acceptable. MX1 is
/// not — outputs may starve.
fn class_acceptable_for_bus(class: BalancerClass) -> bool {
    matches!(
        class,
        BalancerClass::ThroughputBalancedRate
            | BalancerClass::ThroughputUnlimited
            | BalancerClass::Balanced
    )
}

fn assemble_template_from_routing(
    shape: (u32, u32),
    lib: &spaghettio_core::bus::balancer_library::BalancerTemplate,
    splitter_positions: &[SplitterPosOut],
    belts: &[BeltOutput],
    ugs: &[UgOutput],
) -> Result<OwnedTemplate, Box<dyn std::error::Error>> {
    let mut entities: Vec<BalancerTemplateEntity> = Vec::new();
    for sp in splitter_positions {
        entities.push(BalancerTemplateEntity {
            name: "splitter",
            x: sp.x,
            y: sp.y,
            direction: sp.dir,
            io_type: None, input_priority: None, output_priority: None,
        });
    }
    for b in belts {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: b.x,
            y: b.y,
            direction: b.dir,
            io_type: None, input_priority: None, output_priority: None,
        });
    }
    for u in ugs {
        let io_type: &'static str = match u.io.as_str() {
            "input" => "input",
            "output" => "output",
            other => return Err(format!("invalid UG io_type: {other}").into()),
        };
        entities.push(BalancerTemplateEntity {
            name: "underground-belt",
            x: u.x,
            y: u.y,
            direction: u.dir,
            io_type: Some(io_type), input_priority: None, output_priority: None,
        });
    }
    Ok(OwnedTemplate {
        n_inputs: shape.0,
        n_outputs: shape.1,
        width: lib.width,
        height: lib.height,
        entities,
        input_tiles: lib.input_tiles.to_vec(),
        output_tiles: lib.output_tiles.to_vec(),
    })
}

// ---------------------------------------------------------------------------
// Phase 4 — Composition combinator
// ---------------------------------------------------------------------------

fn stress_compose_clos_4_9() -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let atom_1_3 = templates
        .get(&(1u32, 3u32))
        .ok_or("library missing (1, 3)")?;
    let atom_4_3 = templates
        .get(&(4u32, 3u32))
        .ok_or("library missing (4, 3)")?;

    let stage1 = compose_parallel(BalancerTemplateRef::from(atom_1_3), 4);
    println!(
        "  stage1: parallel((1, 3), 4) = {}×{}, {} inputs, {} outputs, {} entities",
        stage1.width, stage1.height, stage1.n_inputs, stage1.n_outputs, stage1.entities.len()
    );
    let stage2 = compose_parallel(BalancerTemplateRef::from(atom_4_3), 3);
    println!(
        "  stage2: parallel((4, 3), 3) = {}×{}, {} inputs, {} outputs, {} entities",
        stage2.width, stage2.height, stage2.n_inputs, stage2.n_outputs, stage2.entities.len()
    );

    let perm = clos_interleave(4, 3);
    println!("  clos_interleave(4, 3) perm: {perm:?}");

    let t0 = std::time::Instant::now();
    let (composed, _) = compose_series(stage1.as_ref(), stage2.as_ref(), &perm, 1, 20)?;
    let elapsed = t0.elapsed().as_secs_f64();
    let junction_h = composed.height - stage1.height - stage2.height;
    println!(
        "  composed: {}×{}, junction_height={} (compose+route in {:.1}s)",
        composed.width, composed.height, junction_h, elapsed
    );
    println!("    {} entities total", composed.entities.len());

    let report =
        classify_ref(composed.as_ref()).map_err(|e| format!("classify_ref: {e:?}"))?;
    println!("    classified: {:?}", report.class);

    if report.class != BalancerClass::Balanced {
        return Err(format!(
            "(4, 9) Clos via compose_*: classified {:?}, expected Balanced",
            report.class
        )
        .into());
    }
    println!("  ✓ (4, 9) Clos placed via composition combinator and verified MX3");
    Ok(())
}

/// Reduced reproducer for the (4, 9) Clos compose Singular bug. Builds the
/// smallest possible Clos composition — `parallel((1, 2), 2)` →
/// `clos_interleave(2, 2)` → `parallel((2, 1), 2)` — which is a (2, 2)
/// Beneš-style 2x2 swap network with 4 splitters and a 4-belt junction
/// permutation [0, 2, 1, 3]. Both via canonical graph combinators
/// (`parallel`/`series_permuted`) and via the template-level combinators
/// (`compose_parallel`/`compose_series`); diffs the two recovered
/// topologies so the failure mode is visible.
fn debug_compose_clos_2_2() -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let atom_1_2 = templates.get(&(1u32, 2u32)).ok_or("library missing (1, 2)")?;
    let atom_2_1 = templates.get(&(2u32, 1u32)).ok_or("library missing (2, 1)")?;

    // Canonical graph topology — the source of truth.
    let stage1_graph = parallel(&library_atom(1, 2).unwrap(), 2);
    let stage2_graph = parallel(&library_atom(2, 1).unwrap(), 2);
    let perm = clos_interleave(2, 2);
    println!("  perm = {perm:?}");
    let canonical = series_permuted(&stage1_graph, &stage2_graph, &perm);
    let canonical_class = classify_graph(&canonical)
        .map_err(|e| format!("classify_graph(canonical): {e:?}"))?;
    println!(
        "  canonical: {} splitters, {} edges, classified {:?}",
        canonical.n_splitters,
        canonical.edges.len(),
        canonical_class.class
    );
    print_graph_summary("  canonical", &canonical);

    // Template-level composition.
    let stage1 = compose_parallel(BalancerTemplateRef::from(atom_1_2), 2);
    let stage2 = compose_parallel(BalancerTemplateRef::from(atom_2_1), 2);
    println!(
        "  stage1: {}×{}, in={}, out={}, entities={}",
        stage1.width, stage1.height, stage1.n_inputs, stage1.n_outputs, stage1.entities.len()
    );
    println!(
        "  stage2: {}×{}, in={}, out={}, entities={}",
        stage2.width, stage2.height, stage2.n_inputs, stage2.n_outputs, stage2.entities.len()
    );

    // Recovered topology of stage1 / stage2 alone.
    let stage1_recovered = topology_of_template(stage1.as_ref())
        .map_err(|e| format!("topology_of_template(stage1): {e:?}"))?;
    print_graph_summary("  stage1.recovered", &stage1_recovered);
    let stage2_recovered = topology_of_template(stage2.as_ref())
        .map_err(|e| format!("topology_of_template(stage2): {e:?}"))?;
    print_graph_summary("  stage2.recovered", &stage2_recovered);

    let (composed, _) = compose_series(stage1.as_ref(), stage2.as_ref(), &perm, 1, 8)?;
    let junction_h = composed.height - stage1.height - stage2.height;
    println!(
        "  composed: {}×{}, junction_height={}, in={}, out={}",
        composed.width, composed.height, junction_h, composed.n_inputs, composed.n_outputs
    );

    // Dump junction belts/UGs for inspection.
    print_junction_entities(&composed, stage1.height, junction_h);

    let recovered = topology_of_template(composed.as_ref())
        .map_err(|e| format!("topology_of_template(composed): {e:?}"))?;
    println!(
        "  recovered: {} splitters, {} edges",
        recovered.n_splitters,
        recovered.edges.len()
    );
    print_graph_summary("  recovered", &recovered);

    let class = match classify_ref(composed.as_ref()) {
        Ok(r) => format!("{:?}", r.class),
        Err(e) => format!("ERR {e:?}"),
    };
    println!("  composed classified: {class}");

    if class == "Balanced" {
        println!("  ✓ (2, 2) Clos compose works");
    } else {
        println!("  ✗ (2, 2) Clos compose: classified {class}, expected Balanced");
    }
    Ok(())
}

fn print_graph_summary(label: &str, g: &SplitterGraph) {
    let mut in_deg = vec![0usize; g.n_splitters];
    let mut out_deg = vec![0usize; g.n_splitters];
    let mut in_from_input = vec![0usize; g.n_splitters];
    let mut out_to_output = vec![0usize; g.n_splitters];
    for &(src, dst) in &g.edges {
        if let NodeId::Splitter(s) = src {
            out_deg[s] += 1;
            if matches!(dst, NodeId::OutputPort(_)) {
                out_to_output[s] += 1;
            }
        }
        if let NodeId::Splitter(s) = dst {
            in_deg[s] += 1;
            if matches!(src, NodeId::InputPort(_)) {
                in_from_input[s] += 1;
            }
        }
    }
    println!(
        "{label}: in/out={}, splitters={}, edges={}",
        g.n_inputs, g.n_splitters, g.edges.len()
    );
    for s in 0..g.n_splitters {
        println!(
            "{label}:   S{s} in={} (in_from_input={}) out={} (out_to_output={})",
            in_deg[s], in_from_input[s], out_deg[s], out_to_output[s]
        );
    }
    for &(src, dst) in &g.edges {
        println!("{label}:   {src:?} -> {dst:?}");
    }
}

fn print_junction_entities(t: &OwnedTemplate, top_height: u32, jh: u32) {
    let y0 = top_height as i32;
    let y1 = (top_height + jh) as i32;
    println!("  junction entities (y in [{y0}, {y1})):");
    for e in &t.entities {
        if e.y >= y0 && e.y < y1 {
            let kind = match e.io_type {
                Some("input") => "UG-in",
                Some("output") => "UG-out",
                _ => match e.name {
                    "transport-belt" => "belt",
                    "splitter" => "split",
                    other => other,
                },
            };
            println!("    ({}, {}) {kind} dir={}", e.x, e.y, e.direction);
        }
    }
}

/// Phase 3.4 spike — generate (1, 9) by composing the library (1, 3) atom
/// with parallel((1, 3), 3) under an identity perm, verify it classifies as
/// MX3, and emit the Rust source for `crates/core/src/bus/balancer_library.rs`.
fn bake_compose_1_9() -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let atom_1_3 = templates.get(&(1u32, 3u32)).ok_or("library missing (1, 3)")?;

    // stage1 is just the library (1, 3) itself; stage2 is 3 copies of it.
    let stage2 = compose_parallel(BalancerTemplateRef::from(atom_1_3), 3);
    let perm: Vec<usize> = vec![0, 1, 2]; // identity — fanout tree, no swap.
    println!("  perm = {perm:?} (identity)");

    let t0 = std::time::Instant::now();
    let (composed, _) = compose_series(
        BalancerTemplateRef::from(atom_1_3),
        stage2.as_ref(),
        &perm,
        1,
        8,
    )?;
    let elapsed = t0.elapsed().as_secs_f64();
    let junction_h = composed.height - atom_1_3.height - stage2.height;
    println!(
        "  composed: {}×{}, junction_height={} (compose+route in {:.2}s)",
        composed.width, composed.height, junction_h, elapsed
    );
    println!(
        "    inputs={}, outputs={}, entities={}",
        composed.n_inputs,
        composed.n_outputs,
        composed.entities.len()
    );

    let report = classify_ref(composed.as_ref())
        .map_err(|e| format!("classify_ref(composed): {e:?}"))?;
    if report.class != BalancerClass::Balanced {
        return Err(format!(
            "(1, 9) compose: classified {:?}, expected Balanced",
            report.class
        )
        .into());
    }
    println!("  ✓ (1, 9) classified Balanced");

    let source = emit_template_rust_source(&composed, (1, 9));
    println!("\n----- begin generated source for balancer_library.rs -----");
    print!("{source}");
    println!("----- end generated source -----");
    Ok(())
}

/// Emit Rust source for a `BalancerTemplate` constant suitable for pasting
/// into `crates/core/src/bus/balancer_library.rs`. Produces three statics
/// (`T_<N>_<M>_ENTITIES`, `T_<N>_<M>_INPUT`, `T_<N>_<M>_OUTPUT`) and the
/// matching `m.insert((<N>, <M>), BalancerTemplate { ... })` line. Caller
/// is responsible for splicing the output into the right place in the file.
fn emit_template_rust_source(t: &OwnedTemplate, shape: (u32, u32)) -> String {
    let (n, m) = shape;
    let mut out = String::new();

    // ENTITIES
    out.push_str(&format!(
        "static T_{n}_{m}_ENTITIES: &[BalancerTemplateEntity] = &[\n"
    ));
    for e in &t.entities {
        let io_str = match e.io_type {
            Some("input") => "Some(\"input\")",
            Some("output") => "Some(\"output\")",
            Some(other) => panic!("unexpected io_type {other:?}"),
            None => "None",
        };
        out.push_str(&format!(
            "    BalancerTemplateEntity {{ name: {:?}, x: {}, y: {}, direction: {}, io_type: {}, input_priority: {:?}, output_priority: {:?} }},\n",
            e.name, e.x, e.y, e.direction, io_str, e.input_priority, e.output_priority
        ));
    }
    out.push_str("];\n");

    // INPUT / OUTPUT tile lists
    out.push_str(&format!(
        "static T_{n}_{m}_INPUT: &[(i32, i32)] = &{:?};\n",
        t.input_tiles
    ));
    out.push_str(&format!(
        "static T_{n}_{m}_OUTPUT: &[(i32, i32)] = &{:?};\n",
        t.output_tiles
    ));

    // m.insert(...)
    out.push_str(&format!(
        "\n    m.insert(({n}, {m}), BalancerTemplate {{\n\
         \x20\x20\x20\x20    n_inputs: {n}, n_outputs: {m}, width: {w}, height: {h},\n\
         \x20\x20\x20\x20    entities: T_{n}_{m}_ENTITIES, input_tiles: T_{n}_{m}_INPUT, output_tiles: T_{n}_{m}_OUTPUT,\n\
         \x20\x20\x20\x20    source_blueprint: \"\",\n\
         \x20\x20\x20\x20}});\n",
        w = t.width,
        h = t.height,
    ));

    out
}

// ---------------------------------------------------------------------------
// Phase 3.4 — recipe-driven baking of missing library shapes
// ---------------------------------------------------------------------------

/// One half of a compose recipe: either pull a library shape directly, or
/// stamp `k` copies of a library shape side-by-side.
#[derive(Debug, Clone, Copy)]
enum Stage {
    Lib(u32, u32),
    Parallel(u32, u32, u32), // (m, n, k)
}

#[derive(Debug, Clone)]
enum Perm {
    Identity,
    Clos(usize, usize), // (m, k) for clos_interleave
}

#[derive(Debug, Clone)]
struct Recipe {
    shape: (u32, u32),
    stage1: Stage,
    stage2: Stage,
    perm: Perm,
    /// Search bounds for the junction-routing solver. Identity perms can
    /// usually solve at jh=1; Clos perms need a larger search window.
    max_jh: u32,
}

fn build_stage(s: Stage) -> Result<OwnedTemplate, Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    match s {
        Stage::Lib(m, n) => {
            let t = templates
                .get(&(m, n))
                .ok_or_else(|| format!("library missing ({m}, {n})"))?;
            // Wrap as OwnedTemplate so the caller has a uniform handle.
            Ok(OwnedTemplate {
                n_inputs: t.n_inputs,
                n_outputs: t.n_outputs,
                width: t.width,
                height: t.height,
                entities: t.entities.to_vec(),
                input_tiles: t.input_tiles.to_vec(),
                output_tiles: t.output_tiles.to_vec(),
            })
        }
        Stage::Parallel(m, n, k) => {
            let atom = templates
                .get(&(m, n))
                .ok_or_else(|| format!("library missing ({m}, {n}) for parallel"))?;
            Ok(compose_parallel(BalancerTemplateRef::from(atom), k))
        }
    }
}

fn run_recipe_from_jh(r: &Recipe, min_jh: u32) -> Result<(OwnedTemplate, u32), Box<dyn std::error::Error>> {
    let stage1 = build_stage(r.stage1)?;
    let stage2 = build_stage(r.stage2)?;
    if stage1.n_outputs != stage2.n_inputs {
        return Err(format!(
            "recipe {:?}: stage1.n_outputs={} != stage2.n_inputs={}",
            r.shape, stage1.n_outputs, stage2.n_inputs
        )
        .into());
    }
    if stage1.n_inputs != r.shape.0 || stage2.n_outputs != r.shape.1 {
        return Err(format!(
            "recipe {:?}: stage1.n_inputs={} stage2.n_outputs={} ≠ target shape",
            r.shape, stage1.n_inputs, stage2.n_outputs
        )
        .into());
    }
    let perm: Vec<usize> = match &r.perm {
        Perm::Identity => (0..stage1.n_outputs as usize).collect(),
        Perm::Clos(m, k) => clos_interleave(*m, *k),
    };
    if perm.len() != stage1.n_outputs as usize {
        return Err(format!(
            "recipe {:?}: perm length {} != stage1.n_outputs {}",
            r.shape,
            perm.len(),
            stage1.n_outputs
        )
        .into());
    }
    compose_series(stage1.as_ref(), stage2.as_ref(), &perm, min_jh, r.max_jh)
}

fn bake_missing_shapes() -> Result<(), Box<dyn std::error::Error>> {
    // Pattern key:
    //   - merger (n, 1): parallel((k, 1), j) → (j, 1) with identity, j*k = n.
    //   - fanout (1, m): (1, j) → parallel((1, k), j) with identity, j*k = m.
    //   - merge-then-balance (n, p): parallel((k, 1), j) → (j, p) with identity.
    //   - Clos (m, n) with k|n: parallel((1, k), m) → parallel((m, n/k), k) with clos_interleave(m, k).
    let recipes: Vec<Recipe> = vec![
        // (9, 1) — pure 9-input merger.
        Recipe {
            shape: (9, 1),
            stage1: Stage::Parallel(3, 1, 3),
            stage2: Stage::Lib(3, 1),
            perm: Perm::Identity,
            max_jh: 8,
        },
        // (1, 10) — fanout via (1, 5) → parallel((1, 2), 5).
        Recipe {
            shape: (1, 10),
            stage1: Stage::Lib(1, 5),
            stage2: Stage::Parallel(1, 2, 5),
            perm: Perm::Identity,
            max_jh: 8,
        },
        // (10, 1) — merger via parallel((5, 1), 2) → (2, 1).
        Recipe {
            shape: (10, 1),
            stage1: Stage::Parallel(5, 1, 2),
            stage2: Stage::Lib(2, 1),
            perm: Perm::Identity,
            max_jh: 8,
        },
        // (2, 9) — Clos via parallel((1, 3), 2) → parallel((2, 3), 3).
        Recipe {
            shape: (2, 9),
            stage1: Stage::Parallel(1, 3, 2),
            stage2: Stage::Parallel(2, 3, 3),
            perm: Perm::Clos(2, 3),
            max_jh: 16,
        },
        // (7, 2) — re-bake via Lib(7, 1) → Lib(1, 2). The python-derived
        // (7, 2) in balancer_library.rs sideloads a UG input (validator
        // emits `belt sideloads into UG input — only one lane loaded`),
        // which the new lane gate would have rejected. Re-baking with the
        // gate active produces a clean composition.
        Recipe {
            shape: (7, 2),
            stage1: Stage::Lib(7, 1),
            stage2: Stage::Lib(1, 2),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // Re-bakes for python-derived shapes that have UG-sideload
        // warnings on the symmetric (n, m) ↔ (m, n) rotation. Each
        // recipe here decomposes through CLEAN library atoms so the
        // compose pipeline produces a sideload-free layout.

        // (1, 9) — fanout via (1, 3) → parallel((1, 3), 3).
        Recipe {
            shape: (1, 9),
            stage1: Stage::Lib(1, 3),
            stage2: Stage::Parallel(1, 3, 3),
            perm: Perm::Identity,
            max_jh: 8,
        },
        // (3, 2) — merge-then-balance via Lib(3, 1) → Lib(1, 2).
        Recipe {
            shape: (3, 2),
            stage1: Stage::Lib(3, 1),
            stage2: Stage::Lib(1, 2),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (6, 2) — parallel((3, 1), 2) → Lib(2, 2). Avoids the dirty
        // (6, 1) python-derived atom by using two clean (3, 1)s.
        Recipe {
            shape: (6, 2),
            stage1: Stage::Parallel(3, 1, 2),
            stage2: Stage::Lib(2, 2),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (7, 3) — Lib(7, 1) → Lib(1, 3). (7, 1) is clean.
        Recipe {
            shape: (7, 3),
            stage1: Stage::Lib(7, 1),
            stage2: Stage::Lib(1, 3),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (8, 2) — parallel((4, 1), 2) → Lib(2, 2). Avoids the dirty
        // (8, 1) python-derived atom.
        Recipe {
            shape: (8, 2),
            stage1: Stage::Parallel(4, 1, 2),
            stage2: Stage::Lib(2, 2),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (6, 1) — parallel((3, 1), 2) → Lib(2, 1). (3, 1) and (2, 1)
        // are clean atoms.
        Recipe {
            shape: (6, 1),
            stage1: Stage::Parallel(3, 1, 2),
            stage2: Stage::Lib(2, 1),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (8, 1) — parallel((4, 1), 2) → Lib(2, 1). Avoids the dirty
        // (5, 1) prime atom.
        Recipe {
            shape: (8, 1),
            stage1: Stage::Parallel(4, 1, 2),
            stage2: Stage::Lib(2, 1),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (8, 6) — parallel((4, 1), 2) → Lib(2, 6). (2, 6) is a clean
        // library atom.
        Recipe {
            shape: (8, 6),
            stage1: Stage::Parallel(4, 1, 2),
            stage2: Stage::Lib(2, 6),
            perm: Perm::Identity,
            max_jh: 4,
        },
        // (9, 2) — merge-then-balance via parallel((3, 1), 3) → (3, 2).
        Recipe {
            shape: (9, 2),
            stage1: Stage::Parallel(3, 1, 3),
            stage2: Stage::Lib(3, 2),
            perm: Perm::Identity,
            max_jh: 8,
        },
        // (9, 3..=8) — same merge-then-balance pattern through (3, m) atoms.
        Recipe { shape: (9, 3), stage1: Stage::Parallel(3, 1, 3), stage2: Stage::Lib(3, 3), perm: Perm::Identity, max_jh: 8 },
        Recipe { shape: (9, 4), stage1: Stage::Parallel(3, 1, 3), stage2: Stage::Lib(3, 4), perm: Perm::Identity, max_jh: 8 },
        Recipe { shape: (9, 5), stage1: Stage::Parallel(3, 1, 3), stage2: Stage::Lib(3, 5), perm: Perm::Identity, max_jh: 8 },
        Recipe { shape: (9, 6), stage1: Stage::Parallel(3, 1, 3), stage2: Stage::Lib(3, 6), perm: Perm::Identity, max_jh: 8 },
        Recipe { shape: (9, 7), stage1: Stage::Parallel(3, 1, 3), stage2: Stage::Lib(3, 7), perm: Perm::Identity, max_jh: 8 },
        Recipe { shape: (9, 8), stage1: Stage::Parallel(3, 1, 3), stage2: Stage::Lib(3, 8), perm: Perm::Identity, max_jh: 8 },
        // (m, 9) Clos for m in 3..=9 — same pattern as (2, 9):
        // parallel((1, 3), m) → parallel((m, 3), 3) with clos_interleave(m, 3).
        // Issue #136. max_jh=24 is generous; (4, 9) finds jh=9.
        Recipe { shape: (3, 9), stage1: Stage::Parallel(1, 3, 3), stage2: Stage::Parallel(3, 3, 3), perm: Perm::Clos(3, 3), max_jh: 24 },
        Recipe { shape: (4, 9), stage1: Stage::Parallel(1, 3, 4), stage2: Stage::Parallel(4, 3, 3), perm: Perm::Clos(4, 3), max_jh: 24 },
        Recipe { shape: (5, 9), stage1: Stage::Parallel(1, 3, 5), stage2: Stage::Parallel(5, 3, 3), perm: Perm::Clos(5, 3), max_jh: 24 },
        Recipe { shape: (6, 9), stage1: Stage::Parallel(1, 3, 6), stage2: Stage::Parallel(6, 3, 3), perm: Perm::Clos(6, 3), max_jh: 24 },
        Recipe { shape: (7, 9), stage1: Stage::Parallel(1, 3, 7), stage2: Stage::Parallel(7, 3, 3), perm: Perm::Clos(7, 3), max_jh: 24 },
        Recipe { shape: (8, 9), stage1: Stage::Parallel(1, 3, 8), stage2: Stage::Parallel(8, 3, 3), perm: Perm::Clos(8, 3), max_jh: 24 },
        Recipe { shape: (9, 9), stage1: Stage::Parallel(1, 3, 9), stage2: Stage::Parallel(9, 3, 3), perm: Perm::Clos(9, 3), max_jh: 24 },

        // === utility@10/s balancer-gap spike (uncommitted) ===
        // Demanded by the science_scaling_gauntlet utility-science-pack@10/s
        // cell (35 dead-end producer rows across 3 families). See
        // crates/core/examples/census_missing_balancer_shapes.rs for the
        // full-corpus census this spike ran to confirm these are the only
        // missing shapes across all 24 gauntlet cells.

        // (12, 7) — electronic-circuit. merge-then-balance: both atoms
        // ((4, 1) and (3, 7)) already in the library.
        Recipe { shape: (12, 7), stage1: Stage::Parallel(4, 1, 3), stage2: Stage::Lib(3, 7), perm: Perm::Identity, max_jh: 8 },

        // (15, 7) — NOT itself a demanded shape. Intermediate atom for the
        // (15, 14) Clos below (n=15 exceeds the library's direct-atom
        // range, so the Clos second stage needs this baked first).
        // merge-then-balance: (3, 1) and (5, 7) already in the library.
        Recipe { shape: (15, 7), stage1: Stage::Parallel(3, 1, 5), stage2: Stage::Lib(5, 7), perm: Perm::Identity, max_jh: 16 },

        // (15, 14) — copper-cable. Clos via parallel((1,2), 15) →
        // parallel((15,7), 2) with clos_interleave(15, 2). Needs (15, 7)
        // baked above and synced into the library first (run this as a
        // second pass after (15, 7) lands).
        Recipe { shape: (15, 14), stage1: Stage::Parallel(1, 2, 15), stage2: Stage::Parallel(15, 7, 2), perm: Perm::Clos(15, 2), max_jh: 24 },
    ];

    // SPAGHETTIO_BAKE_ONLY: semicolon-separated `(m,n)` pairs. When set,
    // restrict this run to exactly those shapes — skips the (large) backlog
    // of not-yet-synced precedent recipes earlier in the list so a targeted
    // spike doesn't pay for unrelated multi-hour bakes.
    // Example: SPAGHETTIO_BAKE_ONLY='(12,7);(15,7)'
    let bake_only: HashSet<(u32, u32)> = std::env::var("SPAGHETTIO_BAKE_ONLY")
        .ok()
        .map(|s| {
            s.split(';')
                .filter_map(|tok| {
                    let nums: Vec<u32> = tok
                        .trim_matches(|c: char| !c.is_ascii_digit())
                        .split(|c: char| !c.is_ascii_digit())
                        .filter(|p| !p.is_empty())
                        .filter_map(|p| p.parse().ok())
                        .collect();
                    if nums.len() == 2 {
                        Some((nums[0], nums[1]))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let recipes: Vec<Recipe> = if bake_only.is_empty() {
        recipes
    } else {
        recipes.into_iter().filter(|r| bake_only.contains(&r.shape)).collect()
    };

    let mut all_source = String::new();
    let mut shapes_done: Vec<(u32, u32)> = Vec::new();
    let mut shapes_failed: Vec<((u32, u32), String)> = Vec::new();
    let mut shapes_skipped: Vec<(u32, u32)> = Vec::new();
    let library = balancer_templates();
    let total = recipes.len();
    // Optional re-bake override: semicolon-separated `(m,n)` pairs that
    // should be re-generated even though they exist in the library. Used
    // to fix grandfathered-in templates that fail the current lane gate.
    // Example: SPAGHETTIO_REBAKE_SHAPES='(7,2);(9,2)'
    let force_rebake: HashSet<(u32, u32)> = std::env::var("SPAGHETTIO_REBAKE_SHAPES")
        .ok()
        .map(|s| {
            s.split(';')
                .filter_map(|tok| {
                    let nums: Vec<u32> = tok
                        .trim_matches(|c: char| !c.is_ascii_digit())
                        .split(|c: char| !c.is_ascii_digit())
                        .filter(|p| !p.is_empty())
                        .filter_map(|p| p.parse().ok())
                        .collect();
                    if nums.len() == 2 {
                        Some((nums[0], nums[1]))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    'outer: for (idx, r) in recipes.iter().enumerate() {
        let (m, n) = r.shape;
        if library.contains_key(&(m, n)) && !force_rebake.contains(&(m, n)) {
            println!("\n--- [{}/{total}] ({m}, {n}): SKIP (already in library) ---", idx + 1);
            shapes_skipped.push((m, n));
            continue;
        }
        if force_rebake.contains(&(m, n)) {
            println!("\n--- [{}/{total}] ({m}, {n}): FORCE-REBAKE (SPAGHETTIO_REBAKE_SHAPES) ---", idx + 1);
        }
        println!("\n--- [{}/{total}] ({m}, {n}): {:?} → {:?} via {:?} ---",
                 idx + 1, r.stage1, r.stage2, r.perm);

        // Bounded retry: if lane validation finds underground-belt errors, bump
        // min_jh past the used value and retry up to 3 times total.
        let mut min_jh: u32 = 1;
        let mut composed_opt: Option<OwnedTemplate> = None;
        'bake: for attempt in 1u32..=3 {
            let t0 = std::time::Instant::now();
            let (composed, used_jh) = match run_recipe_from_jh(r, min_jh) {
                Ok(c) => c,
                Err(e) => {
                    println!("  ✗ compose (attempt {attempt}): {e}");
                    shapes_failed.push(((m, n), format!("compose: {e}")));
                    continue 'outer;
                }
            };
            let elapsed = t0.elapsed().as_secs_f64();
            println!(
                "  attempt {attempt}: composed {}×{}, {} entities, jh={used_jh} ({:.2}s)",
                composed.width, composed.height, composed.entities.len(), elapsed
            );

            // Lane gate: only block on underground-belt Severity::Error.
            // Lane-throughput artifacts (category != "underground-belt") are
            // validator noise on standalone templates — do not gate on them.
            let issues = validate_template_lanes(composed.as_ref());
            // Gate on both errors and warnings in the underground-belt
            // category. UG-sideload warnings ("Belt sideloads into UG input
            // — only one lane loaded") are real throughput flaws — the UG
            // carries half capacity. A bake that produces them isn't a
            // viable balancer even if classify_ref says "Balanced".
            let ug_issues: Vec<_> = issues
                .iter()
                .filter(|i| {
                    matches!(i.severity, Severity::Error | Severity::Warning)
                        && i.category == "underground-belt"
                })
                .collect();
            if ug_issues.is_empty() {
                composed_opt = Some(composed);
                break 'bake;
            }
            println!(
                "  ✗ lane gate ({} ug issue(s) at jh={used_jh}), next min_jh={}",
                ug_issues.len(),
                used_jh + 1
            );
            for e in &ug_issues {
                println!("    {:?}: {} at ({:?}, {:?})", e.severity, e.message, e.x, e.y);
            }
            min_jh = used_jh + 1;
        }

        let composed = match composed_opt {
            Some(c) => c,
            None => {
                shapes_failed.push(((m, n), "lane gate: exhausted 3 attempts".to_string()));
                continue;
            }
        };

        let report = match classify_ref(composed.as_ref()) {
            Ok(r) => r,
            Err(e) => {
                println!("  ✗ classify_ref: {e:?}");
                shapes_failed.push(((m, n), format!("classify_ref: {e:?}")));
                continue;
            }
        };
        println!("  classified: {:?}", report.class);
        if report.class != BalancerClass::Balanced {
            shapes_failed.push(((m, n), format!("classified {:?}", report.class)));
            continue;
        }

        let header = format!(
            "\n// === ({m}, {n}) — generated by balancer-gen bake_missing_shapes ===\n"
        );
        let body = emit_template_rust_source(&composed, (m, n));
        // Emit per-shape immediately so an interrupted run is still
        // recoverable from the log — the final summary repeats it all.
        println!("\n----- begin generated source ({m}, {n}) -----");
        print!("{header}{body}");
        println!("----- end generated source ({m}, {n}) -----");
        all_source.push_str(&header);
        all_source.push_str(&body);
        shapes_done.push((m, n));
    }

    println!("\n=== summary ===");
    println!(
        "  baked: {} / {} shapes ({} skipped, {} failed)",
        shapes_done.len(),
        total,
        shapes_skipped.len(),
        shapes_failed.len()
    );
    for (m, n) in &shapes_done {
        println!("    ✓ ({m}, {n})");
    }
    for (m, n) in &shapes_skipped {
        println!("    – ({m}, {n}) skipped");
    }
    for ((m, n), reason) in &shapes_failed {
        println!("    ✗ ({m}, {n}): {reason}");
    }
    println!("\n----- begin generated source -----");
    print!("{all_source}");
    println!("----- end generated source -----");

    if !shapes_failed.is_empty() {
        return Err(format!("{} shapes failed", shapes_failed.len()).into());
    }
    Ok(())
}

fn smoke_compose_parallel() -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let atom = templates.get(&(1u32, 3u32)).ok_or("library missing (1, 3)")?;
    let composed = compose_parallel(BalancerTemplateRef::from(atom), 4);
    println!(
        "  compose_parallel((1, 3), 4): {}×{}, {} inputs, {} outputs, {} entities",
        composed.width, composed.height,
        composed.n_inputs, composed.n_outputs,
        composed.entities.len(),
    );
    if composed.n_inputs != 4 || composed.n_outputs != 12 {
        return Err(format!(
            "expected (4, 12) IO, got ({}, {})",
            composed.n_inputs, composed.n_outputs
        ).into());
    }
    if composed.width != atom.width * 4 {
        return Err(format!(
            "expected width {}, got {}",
            atom.width * 4, composed.width
        ).into());
    }
    if composed.entities.len() != atom.entities.len() * 4 {
        return Err(format!(
            "expected {} entities, got {}",
            atom.entities.len() * 4, composed.entities.len()
        ).into());
    }
    // Each copy is independent — the composed graph is k disjoint atoms.
    // Classification: this is NOT an MX3 (4, 12) balancer — it's 4
    // separate (1, 3) atoms. The classifier should still parse it; we
    // just don't assert MX3 here.
    let report = classify_ref(composed.as_ref())
        .map_err(|e| format!("classify_ref on parallel composition: {e:?}"))?;
    println!("  classified {:?} (parallel of MX3 atoms — not itself a balancer)", report.class);
    Ok(())
}

/// Place `k` copies of `template` side-by-side along the x-axis.
///
/// IO tiles are concatenated in copy order: input port `c * t.n_inputs + i`
/// is the i-th input port of the c-th copy, and similarly for outputs.
/// All entity coordinates are shifted by `c * template.width` for copy c.
fn compose_parallel(template: BalancerTemplateRef<'_>, k: u32) -> OwnedTemplate {
    let mut entities: Vec<BalancerTemplateEntity> = Vec::with_capacity(template.entities.len() * k as usize);
    let mut input_tiles: Vec<(i32, i32)> = Vec::with_capacity(template.input_tiles.len() * k as usize);
    let mut output_tiles: Vec<(i32, i32)> = Vec::with_capacity(template.output_tiles.len() * k as usize);
    for copy in 0..k {
        let x_off = (copy * template.width) as i32;
        for e in template.entities {
            entities.push(BalancerTemplateEntity {
                name: e.name,
                x: e.x + x_off,
                y: e.y,
                direction: e.direction,
                io_type: e.io_type, input_priority: None, output_priority: None,
            });
        }
        for &(x, y) in template.input_tiles {
            input_tiles.push((x + x_off, y));
        }
        for &(x, y) in template.output_tiles {
            output_tiles.push((x + x_off, y));
        }
    }
    OwnedTemplate {
        n_inputs: template.n_inputs * k,
        n_outputs: template.n_outputs * k,
        width: template.width * k,
        height: template.height,
        entities,
        input_tiles,
        output_tiles,
    }
}

/// Stack `top` over `bot` with a junction region in between that
/// implements the permutation `perm`: `top.output_tiles[i]` flow lands
/// at `bot.input_tiles[perm[i]]`.
///
/// `junction_height` is a starting guess; the function grows the
/// junction up to `max_junction_height` until `solve_pure_routing`
/// finds a feasible layout.
///
/// Width alignment: composed width is `max(top.width, bot.width)`.
/// Both stages are left-aligned (x_pad = 0). MVP scope; if widths
/// differ significantly, the narrower stage trails empty cells.
fn compose_series(
    top: BalancerTemplateRef<'_>,
    bot: BalancerTemplateRef<'_>,
    perm: &[usize],
    initial_junction_height: u32,
    max_junction_height: u32,
) -> Result<(OwnedTemplate, u32), Box<dyn std::error::Error>> {
    if top.output_tiles.len() != perm.len() {
        return Err(format!(
            "compose_series: top has {} outputs but perm has {} entries",
            top.output_tiles.len(), perm.len()
        ).into());
    }
    if bot.input_tiles.len() != perm.len() {
        return Err(format!(
            "compose_series: bot has {} inputs but perm has {} entries",
            bot.input_tiles.len(), perm.len()
        ).into());
    }
    let composed_width = std::cmp::max(top.width, bot.width);

    // Junction-routing source/dest tiles (in junction-local coords).
    // Source tiles: y=0 in junction-local, x = top.output_tiles[i].x.
    // Dest tiles: y=junction_height-1 in junction-local, x = bot.input_tiles[j].x.
    let junction_input_tiles: Vec<(i32, i32)> = top
        .output_tiles
        .iter()
        .map(|&(x, _)| (x, 0))
        .collect();
    let edges: Vec<EdgeReq> = (0..perm.len())
        .map(|i| EdgeReq {
            src_kind: "InputPort",
            src_idx: i,
            dst_kind: "OutputPort",
            dst_idx: perm[i],
        })
        .collect();

    // Heuristic starting jh: identity perms route at jh=1, but a non-trivial
    // permutation routing typically needs ~ceil(log2(N)) + 1 rows of slack
    // for the cross-overs. Skipping the obviously-infeasible early jh values
    // is a multi-minute saving on larger Clos shapes (the (4, 9) compose ate
    // most of its 1173s burning through jh=1..8 before finding 9).
    let is_identity = perm.iter().enumerate().all(|(i, &p)| i == p);
    let heuristic_start: u32 = if is_identity {
        1
    } else {
        let n = perm.len() as f64;
        ((n.log2().ceil() as u32) + 1).max(1)
    };
    let effective_start = std::cmp::max(initial_junction_height, heuristic_start);
    if effective_start > initial_junction_height {
        eprintln!(
            "  compose_series: heuristic bumped initial_jh {initial_junction_height} → {effective_start} (perm len={}, identity={is_identity})",
            perm.len()
        );
    }

    // Two-budget search (RFP rfp-balancer-jh-search.md):
    // - SHORT_TIMEOUT: fast infeasibility proof. INFEASIBLE under SHORT is a
    //   real CP-SAT proof; advance immediately without spending more time.
    // - LONG_TIMEOUT: used only when SHORT returns UNKNOWN (ambiguous). Retry
    //   same jh once with the full budget before advancing.
    // This replaces the old stratified 90s→240s→600s scheme, which was slow
    // at proving infeasible jhs (esp. jh=8 on (4,9) which burned ~120s there).
    const SHORT_TIMEOUT: f64 = 30.0;
    const LONG_TIMEOUT: f64 = 600.0;

    // Encoding selector is constant for the whole compose_series call.
    let encoding: Option<&'static str> = match std::env::var("SPAGHETTIO_PURE_ROUTING_ENCODING").as_deref() {
        Ok("circuit") => Some("circuit"),
        _ => None,
    };

    let mut last_err: String = String::new();
    for jh in effective_start..=max_junction_height {
        let junction_output_tiles: Vec<(i32, i32)> = bot
            .input_tiles
            .iter()
            .map(|&(x, _)| (x, (jh - 1) as i32))
            .collect();

        // Inner retry loop: start with SHORT; escalate to LONG on UNKNOWN.
        // Uses a labeled block so `break 'attempt` can carry a value out.
        let resp_opt: Option<PlaceResponse> = 'attempt: {
            let mut timeout_s = SHORT_TIMEOUT;
            loop {
                let req = PlaceRequest::PureRouting {
                    kind: "pure_routing",
                    bounds: (composed_width, jh),
                    input_port_tiles: junction_input_tiles.clone(),
                    output_port_tiles: junction_output_tiles.clone(),
                    edges: edges.clone(),
                    max_time_s: Some(timeout_s),
                    encoding,
                    ug_max_reach: None, // None = default yellow (reach=4); future PR sweeps to red/blue
                };
                let budget_tag = if timeout_s <= SHORT_TIMEOUT { "[SHORT]" } else { "[LONG]" };
                let t_attempt = std::time::Instant::now();
                let resp = match run_solver(&req) {
                    Ok(r) => r,
                    Err(e) => {
                        last_err = format!("subprocess error at junction_height={jh}: {e}");
                        break 'attempt None;
                    }
                };
                eprintln!(
                    "  compose_series: {budget_tag} jh={jh} status={} solver_elapsed={:.1}s wall={:.1}s timeout={:.0}s",
                    resp.status,
                    resp.elapsed_s,
                    t_attempt.elapsed().as_secs_f64(),
                    timeout_s
                );
                match resp.status.as_str() {
                    "OPTIMAL" | "FEASIBLE" => break 'attempt Some(resp),
                    "INFEASIBLE" => {
                        // Real proof — advance to next jh immediately.
                        last_err = format!(
                            "junction_height={jh}: {budget_tag} solver returned INFEASIBLE after {:.2}s",
                            resp.elapsed_s
                        );
                        break 'attempt None;
                    }
                    _ => {
                        // UNKNOWN — ambiguous. If we haven't tried LONG yet,
                        // retry same jh with the full budget. If LONG also
                        // returns UNKNOWN, both budgets are exhausted: advance
                        // (matches the old behaviour of advancing on UNKNOWN).
                        if timeout_s >= LONG_TIMEOUT {
                            last_err = format!(
                                "junction_height={jh}: both budgets exhausted (UNKNOWN) after {:.2}s",
                                resp.elapsed_s
                            );
                            break 'attempt None;
                        }
                        // Escalate to LONG and retry.
                        timeout_s = LONG_TIMEOUT;
                    }
                }
            }
        };

        let Some(resp) = resp_opt else { continue; };

        // Found feasible junction. Assemble composed template.
        let junction_y_off = top.height as i32;
        let bot_y_off = (top.height + jh) as i32;

        let mut entities: Vec<BalancerTemplateEntity> = Vec::with_capacity(
            top.entities.len() + bot.entities.len() + resp.belts.len() + resp.ugs.len(),
        );
        for e in top.entities {
            entities.push(BalancerTemplateEntity {
                name: e.name,
                x: e.x,
                y: e.y,
                direction: e.direction,
                io_type: e.io_type, input_priority: None, output_priority: None,
            });
        }
        for b in &resp.belts {
            entities.push(BalancerTemplateEntity {
                name: "transport-belt",
                x: b.x,
                y: b.y + junction_y_off,
                direction: b.dir,
                io_type: None, input_priority: None, output_priority: None,
            });
        }
        for u in &resp.ugs {
            let io_type: &'static str = match u.io.as_str() {
                "input" => "input",
                "output" => "output",
                other => return Err(format!("invalid UG io_type: {other}").into()),
            };
            entities.push(BalancerTemplateEntity {
                name: "underground-belt",
                x: u.x,
                y: u.y + junction_y_off,
                direction: u.dir,
                io_type: Some(io_type), input_priority: None, output_priority: None,
            });
        }
        for e in bot.entities {
            entities.push(BalancerTemplateEntity {
                name: e.name,
                x: e.x,
                y: e.y + bot_y_off,
                direction: e.direction,
                io_type: e.io_type, input_priority: None, output_priority: None,
            });
        }

        let composed_input_tiles: Vec<(i32, i32)> = top.input_tiles.to_vec();
        let composed_output_tiles: Vec<(i32, i32)> = bot
            .output_tiles
            .iter()
            .map(|&(x, y)| (x, y + bot_y_off))
            .collect();

        return Ok((OwnedTemplate {
            n_inputs: top.n_inputs,
            n_outputs: bot.n_outputs,
            width: composed_width,
            height: top.height + jh + bot.height,
            entities,
            input_tiles: composed_input_tiles,
            output_tiles: composed_output_tiles,
        }, jh));
    }
    Err(format!(
        "compose_series: no feasible junction_height in [{initial_junction_height}, {max_junction_height}]: {last_err}"
    ).into())
}

// ---------------------------------------------------------------------------
// Subprocess plumbing
// ---------------------------------------------------------------------------

fn run_solver(req: &PlaceRequest) -> Result<PlaceResponse, Box<dyn std::error::Error>> {
    let req_json = serde_json::to_string(req)?;
    let script = "crates/balancer-gen/scripts/place.py";
    let mut child = Command::new("uv")
        .args(["run", "--no-project", "--script", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to spawn `uv run {script}`: {e}"))?;
    child
        .stdin
        .as_mut()
        .ok_or("no stdin")?
        .write_all(req_json.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("placement script failed: {}", output.status).into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn enforce_kill_criteria(resp: &PlaceResponse) -> Result<(), Box<dyn std::error::Error>> {
    if resp.elapsed_s > 30.0 {
        return Err(format!("kill criterion: solve time {:.1}s > 30s", resp.elapsed_s).into());
    }
    if resp.status != "OPTIMAL" && resp.status != "FEASIBLE" {
        return Err(format!("solver returned {} (no placement)", resp.status).into());
    }
    Ok(())
}
