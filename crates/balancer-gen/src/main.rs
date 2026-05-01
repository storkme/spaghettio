//! Phase 3.1 spike + 3.2 MVP: drive a Python+OR-Tools placement
//! subprocess for splitter no-overlap, then a Rust router emits a full
//! `OwnedTemplate` (splitters + boundary belts) for single-splitter
//! shapes and round-trips through `classify_ref` to confirm the
//! placement preserves the topology's class.
//!
//! Phase 3.2 MVP scope (this binary):
//!   - Single-splitter topologies only (`(1, 2)`, `(2, 2)`).
//!   - South-facing splitters.
//!   - Belt entities only at input/output port tiles — no inter-splitter
//!     routing yet (that's the next chunk of phase 3.2).
//!
//! Even with that scope, this is the first end-to-end round-trip:
//!     library template → topology → CP-SAT placement → entities →
//!     classify_ref → expected class.
//! If it closes for `(1, 2)` and `(2, 2)`, the framework is solid; the
//! remaining work is router complexity, not pipeline plumbing.

use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use fucktorio_core::bus::balancer_classify::{
    classify_graph, classify_ref, topology_of_template, BalancerClass, BalancerTemplateRef,
    SplitterGraph,
};
use fucktorio_core::bus::balancer_generate::OwnedTemplate;
use fucktorio_core::bus::balancer_library::{balancer_templates, BalancerTemplateEntity};
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

#[derive(Deserialize, Debug, Clone, Copy)]
struct SplitterPos {
    x: i32,
    y: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();

    // Phase 3.1: splitter no-overlap only, two cases.
    let small_template = templates
        .get(&(2u32, 3u32))
        .ok_or("library missing (2, 3)")?;
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

    // Phase 3.2 MVP: full placement → entity emission → classify round-trip
    // on single-splitter shapes.
    println!("\n=== phase 3.2 MVP: end-to-end round-trip ===");
    spike_round_trip("(1, 2)", (1, 2))?;
    spike_round_trip("(2, 2)", (2, 2))?;

    println!("\n✓ all spike runs passed");
    Ok(())
}

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
    let resp = run_solver(topology.n_splitters, bounds)?;
    println!(
        "status: {}  elapsed: {:.3}s  placements: {}",
        resp.status,
        resp.elapsed_s,
        resp.splitters.len()
    );
    enforce_kill_criteria(&resp, topology.n_splitters)?;
    Ok(())
}

/// Phase 3.2 MVP: place splitters via CP-SAT, emit boundary belts in
/// Rust, build an `OwnedTemplate`, run through `classify_ref` to confirm
/// the placement preserves the topology's class.
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
            "{label}: phase 3.2 MVP only handles single-splitter shapes \
             (got {} splitters)",
            topology.n_splitters
        )
        .into());
    }

    println!(
        "\n--- {label} ---  bounds: {}×{}  topology: {} splitters, {} edges",
        lib.width, lib.height, topology.n_splitters, topology.edges.len()
    );

    let resp = run_solver(topology.n_splitters, (lib.width, lib.height))?;
    enforce_kill_criteria(&resp, topology.n_splitters)?;

    let sp = resp.splitters[0];
    let placed = emit_single_splitter_template(shape, sp, (lib.width, lib.height))?;

    // Verify: the placed template must classify as MX3 (single-splitter
    // (m, n) shapes are all MX3 under the linear model).
    let class_report = classify_ref(placed.as_ref())
        .map_err(|e| format!("{label}: classify_ref on placed template: {e:?}"))?;
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

/// Build an `OwnedTemplate` for a single-splitter `(m, n)` shape:
/// south-facing splitter at the given position, belts at port tiles.
/// Uses the simple slot-assignment heuristic: the first `m` of the 2
/// input slots get input ports; the first `n` of the 2 output slots get
/// output ports.
fn emit_single_splitter_template(
    shape: (u32, u32),
    sp: SplitterPos,
    bounds: (u32, u32),
) -> Result<OwnedTemplate, String> {
    let (m, n) = shape;
    if m > 2 || n > 2 {
        return Err(format!("single-splitter only handles m,n ≤ 2; got {shape:?}"));
    }

    // South-facing splitter: anchor (sp.x, sp.y), second (sp.x+1, sp.y).
    // Input slots above (y-1), output slots below (y+1).
    let input_slots = [(sp.x, sp.y - 1), (sp.x + 1, sp.y - 1)];
    let output_slots = [(sp.x, sp.y + 1), (sp.x + 1, sp.y + 1)];

    let mut entities: Vec<BalancerTemplateEntity> = Vec::new();
    let mut input_tiles: Vec<(i32, i32)> = Vec::new();
    let mut output_tiles: Vec<(i32, i32)> = Vec::new();

    // Splitter entity. Static-string lifetimes are required by the
    // BalancerTemplateEntity struct, so we use the literal "splitter".
    entities.push(BalancerTemplateEntity {
        name: "splitter",
        x: sp.x,
        y: sp.y,
        direction: 4,
        io_type: None,
    });

    // Input port belts at the chosen input slots, all south-facing.
    for &slot in input_slots.iter().take(m as usize) {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: slot.0,
            y: slot.1,
            direction: 4,
            io_type: None,
        });
        input_tiles.push(slot);
    }

    // Output port belts at chosen output slots.
    for &slot in output_slots.iter().take(n as usize) {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: slot.0,
            y: slot.1,
            direction: 4,
            io_type: None,
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

fn run_solver(
    n_splitters: usize,
    bounds: (u32, u32),
) -> Result<PlaceResponse, Box<dyn std::error::Error>> {
    let req = PlaceRequest { n_splitters, bounds };
    let req_json = serde_json::to_string(&req)?;

    // Drive the placer via `uv run --no-project`: the script's PEP 723
    // header pins `ortools`, so the first invocation self-installs
    // dependencies into a uv-managed environment. Keeps the spike free
    // of "user must `pip install ortools` first" friction and matches
    // the convention PR #270's `scripts/cp_sat_placer.py` uses, so the
    // two entrypoints can consolidate later without rework.
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

fn enforce_kill_criteria(
    resp: &PlaceResponse,
    expected_splitters: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if resp.elapsed_s > 30.0 {
        return Err(format!("kill criterion: solve time {:.1}s > 30s", resp.elapsed_s).into());
    }
    if resp.status != "OPTIMAL" && resp.status != "FEASIBLE" {
        return Err(format!("solver returned {} (no placement)", resp.status).into());
    }
    if resp.splitters.len() != expected_splitters {
        return Err(format!(
            "got {} placements, expected {}",
            resp.splitters.len(),
            expected_splitters
        )
        .into());
    }
    Ok(())
}
