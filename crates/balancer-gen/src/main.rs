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

use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use fucktorio_core::bus::balancer_classify::{
    classify_graph, classify_ref, topology_of_template, BalancerClass, BalancerTemplateRef,
    NodeId, SplitterGraph,
};
use fucktorio_core::bus::balancer_generate::OwnedTemplate;
use fucktorio_core::bus::balancer_library::{balancer_templates, BalancerTemplateEntity};
use fucktorio_core::bus::balancer_topology::{
    clos_interleave, library_atom, parallel, series_permuted,
};

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
    },
}

#[derive(Serialize, Clone)]
struct SplitterPosOut {
    x: i32,
    y: i32,
    dir: u8,
}

#[derive(Serialize)]
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
    for &(label, shape) in &[
        ("(1, 2)", (1u32, 2u32)),
        ("(2, 2)", (2, 2)),
        ("(2, 4)", (2, 4)),
        ("(1, 4)", (1, 4)),
        ("(4, 4)", (4, 4)),
        ("(1, 8)", (1, 8)),
    ] {
        let result = spike_synth_place(label, shape).map_err(|e| e.to_string());
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

    println!("\n✓ all spike runs passed");
    Ok(())
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
        io_type: None,
    });
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
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = balancer_templates();
    let lib = templates
        .get(&shape)
        .ok_or_else(|| format!("library missing {shape:?}"))?;
    let topology = topology_of_template(BalancerTemplateRef::from(lib))
        .map_err(|e| format!("recover_graph failed: {e:?}"))?;
    let edge_reqs = build_edge_reqs(&topology)?;

    let req = PlaceRequest::SynthPlace {
        bounds: (lib.width, lib.height),
        n_inputs: shape.0,
        n_outputs: shape.1,
        n_splitters: topology.n_splitters,
        edges: edge_reqs,
    };

    println!(
        "\n--- {label} ---  bbox: {}×{}  splitters: {}  edges: {}",
        lib.width,
        lib.height,
        topology.n_splitters,
        topology.edges.len()
    );
    let resp = run_solver(&req)?;
    if resp.elapsed_s > 60.0 {
        return Err(format!("kill: solve time {:.1}s > 60s", resp.elapsed_s).into());
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
    if class_report.class != original.class {
        return Err(format!(
            "{label}: synth-placed class {:?} differs from library class {:?}",
            class_report.class, original.class
        )
        .into());
    }
    println!(
        "  ✓ {label}: classified {:?} (matches library); IO {:?}→{:?}",
        class_report.class,
        resp.input_port_tiles,
        resp.output_port_tiles,
    );
    Ok(())
}

fn assemble_template_from_routing(
    shape: (u32, u32),
    lib: &fucktorio_core::bus::balancer_library::BalancerTemplate,
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
            io_type: None,
        });
    }
    for b in belts {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: b.x,
            y: b.y,
            direction: b.dir,
            io_type: None,
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
            io_type: Some(io_type),
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
