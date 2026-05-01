//! Phase 3.1 spike: drive a Python+OR-Tools placement subprocess and
//! verify it can solve a known-good shape's splitter no-overlap problem
//! within the kill-criterion time budget.
//!
//! Belt routing isn't encoded here yet (phase 3.2). The spike's deliverable
//! is "CP-SAT can place N splitters in a W×H grid in <30s for the shapes
//! we care about" — confirming the build/IPC/solver stack works before
//! committing to the full encoding.
//!
//! Usage:
//!     cargo run -p balancer-gen
//!
//! Hits the (2, 3) library template by default. Run from repo root so
//! `crates/balancer-gen/scripts/place.py` is found relative to CWD.

use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use fucktorio_core::bus::balancer_classify::{
    classify_graph, topology_of_template, BalancerClass, BalancerTemplateRef, SplitterGraph,
};
use fucktorio_core::bus::balancer_library::balancer_templates;
use fucktorio_core::bus::balancer_topology::{
    clos_interleave, library_atom, parallel, series_permuted,
};

#[derive(Serialize)]
struct PlaceRequest {
    n_splitters: usize,
    bounds: (u32, u32),
}

#[derive(Deserialize, Debug)]
struct PlaceResponse {
    status: String,
    elapsed_s: f64,
    #[serde(default)]
    splitters: Vec<SplitterPos>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // x/y will be consumed by phase 3.2 entity emission
struct SplitterPos {
    x: i32,
    y: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Two test cases: a small library round-trip and a bigger composed
    // topology. Both need to fit under the 30s kill criterion.
    let templates = balancer_templates();

    let small_template = templates
        .get(&(2u32, 3u32))
        .ok_or("library missing (2, 3)")?;
    let small_topology = topology_of_template(BalancerTemplateRef::from(small_template))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;
    spike_run(
        "(2, 3) library round-trip",
        &small_topology,
        (small_template.width, small_template.height),
    )?;

    // Bigger case: the (4, 9) Clos composition we built in phase 3.0.
    // Bounds picked roomy enough to be feasible — refining is phase 3.2.
    let stage1 = parallel(&library_atom(1, 3).ok_or("library (1, 3) missing")?, 4);
    let stage2 = parallel(&library_atom(4, 3).ok_or("library (4, 3) missing")?, 3);
    let big = series_permuted(&stage1, &stage2, &clos_interleave(4, 3));
    let report = classify_graph(&big).map_err(|e| format!("classify_graph: {e:?}"))?;
    if report.class != BalancerClass::Balanced {
        return Err(format!("(4, 9) Clos composition expected MX3, got {:?}", report.class).into());
    }
    spike_run("(4, 9) Clos composition", &big, (24, 24))?;

    println!("\n✓ all spike runs passed");
    Ok(())
}

fn spike_run(
    label: &str,
    topology: &SplitterGraph,
    bounds: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n=== {label} ===  topology: {} splitters, {} edges  bounds: {}×{}",
        topology.n_splitters, topology.edges.len(), bounds.0, bounds.1
    );

    let req = PlaceRequest {
        n_splitters: topology.n_splitters,
        bounds,
    };
    let req_json = serde_json::to_string(&req)?;

    let script = "crates/balancer-gen/scripts/place.py";
    let mut child = Command::new("python3")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to spawn {script}: {e}"))?;

    child
        .stdin
        .as_mut()
        .ok_or("no stdin")?
        .write_all(req_json.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("placement script failed: {}", output.status).into());
    }
    let resp: PlaceResponse = serde_json::from_slice(&output.stdout)?;

    println!(
        "status: {}  elapsed: {:.3}s  placements: {}",
        resp.status,
        resp.elapsed_s,
        resp.splitters.len()
    );

    if resp.elapsed_s > 30.0 {
        return Err(format!(
            "kill criterion: solve time {:.1}s > 30s",
            resp.elapsed_s
        )
        .into());
    }
    if resp.status != "OPTIMAL" && resp.status != "FEASIBLE" {
        return Err(format!("solver returned {} (no placement)", resp.status).into());
    }
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
