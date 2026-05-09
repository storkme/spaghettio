//! Classify a [`BalancerTemplate`] against the merger taxonomy
//! (MX1 / MX2 / MX3 — see [`docs/factorio-mechanics.md`]).
//!
//! Two checks:
//!   - **Composition (MX3)** — DAG-propagate per-input rates with the
//!     default 50/50 splitter model and check every output is uniform
//!     `1/n` mix of every input.
//!   - **Throughput-unlimited (MX2)** — Menger's theorem via two-direction
//!     max-flow: for every input subset `S`, check
//!     `max_flow(S → all) = min(|S|, n)`, and dually for every output
//!     subset `T`.
//!
//! Sideloads (B8 / U7) are accepted as valid flow merges. The walker
//! emits one edge per upstream flow source, and the linear-system
//! composition handles multi-feeder splitter inputs via flow conservation.
//! Lane-level semantics are an MX5 concern, separate from MX1/MX2/MX3.

use crate::bus::balancer_library::{BalancerTemplate, BalancerTemplateEntity};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Borrowed view over the fields a classifier (or generator-side verifier)
/// needs. Lets us run the classifier on either a static [`BalancerTemplate`]
/// from the library or a runtime-generated template held in `Vec`s without
/// duplicating the analysis code.
#[derive(Debug, Clone, Copy)]
pub struct BalancerTemplateRef<'a> {
    pub n_inputs: u32,
    pub n_outputs: u32,
    pub width: u32,
    pub height: u32,
    pub entities: &'a [BalancerTemplateEntity],
    pub input_tiles: &'a [(i32, i32)],
    pub output_tiles: &'a [(i32, i32)],
}

impl<'a> From<&'a BalancerTemplate> for BalancerTemplateRef<'a> {
    fn from(t: &'a BalancerTemplate) -> Self {
        BalancerTemplateRef {
            n_inputs: t.n_inputs,
            n_outputs: t.n_outputs,
            width: t.width,
            height: t.height,
            entities: t.entities,
            input_tiles: t.input_tiles,
            output_tiles: t.output_tiles,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BalancerClass {
    /// MX1 — outputs may starve under saturated input.
    ThroughputLimited,
    /// MX2a — saturation + balanced rate. Under all-saturated inputs and
    /// unblocked outputs, every output runs at exactly `total_input / n`.
    /// `max_flow(S → all)` = `min(|S|, n)` for every input subset `S`, but
    /// does *not* guarantee max-flow over output subsets.
    ///
    /// Sufficient for spaghettio's bus (homogeneous consumer rows). The
    /// throughput-priority generator targets this class.
    ThroughputBalancedRate,
    /// MX2b — full max-flow property: also holds over output subsets.
    /// Inputs reroute around blocked outputs through sibling paths.
    ThroughputUnlimited,
    /// MX3 — every output is a uniform `1/n` mix of every input
    /// (composition guarantee). Required only for mixed-content belts.
    Balanced,
}

#[derive(Debug, Clone)]
pub enum ClassifyError {
    /// Belt walk fell off the template footprint.
    DanglingBelt { from: (i32, i32) },
    /// Underground-belt input has no matching output downstream.
    UnpairedUg { input_at: (i32, i32) },
    /// Two entities share a tile.
    Overlap { tile: (i32, i32) },
    /// Composition propagation found a cycle (back-loop).
    Cycle { description: String },
    /// The linear system describing the saturated 50/50 splitter network is
    /// singular — usually a recirculation loop with no exit (or a structural
    /// degeneracy our model can't resolve). The simple composition model
    /// gives no answer for these templates.
    Singular,
    /// Other invariant violation.
    Malformed(String),
}

#[derive(Debug, Clone)]
pub struct Mx2Counterexample {
    pub direction: Mx2Direction,
    pub subset: Vec<usize>,
    pub realized: u32,
    pub expected: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mx2Direction {
    /// `max_flow(S → all outputs)` short of `min(|S|, n)`.
    InputSubset,
    /// `max_flow(all inputs → T)` short of `min(m, |T|)`.
    OutputSubset,
}

#[derive(Debug, Clone)]
pub struct ClassificationReport {
    pub class: BalancerClass,
    /// `composition[output_idx][input_idx]` = fraction of input k that
    /// reaches output j under the saturated 50/50 splitter model.
    pub composition: Vec<Vec<f64>>,
    /// First MX2 violation found, if any.
    pub mx2_counterexample: Option<Mx2Counterexample>,
}

/// Classify a single balancer template.
pub fn classify(template: &BalancerTemplate) -> Result<ClassificationReport, ClassifyError> {
    classify_ref(BalancerTemplateRef::from(template))
}

/// Classify any object with a [`BalancerTemplateRef`] view — used by the
/// runtime template generator to verify newly-built layouts.
pub fn classify_ref(template: BalancerTemplateRef<'_>) -> Result<ClassificationReport, ClassifyError> {
    let graph = recover_graph(template)?;
    classify_graph(&graph)
}

/// Extract the logical splitter graph from a balancer template — strips
/// physical positions and exposes the connectivity that the topology
/// generator and placement solver work with. Wraps the internal
/// `recover_graph` (which would otherwise still be accessible only via
/// `classify_ref`).
pub fn topology_of_template(
    template: BalancerTemplateRef<'_>,
) -> Result<SplitterGraph, ClassifyError> {
    recover_graph(template)
}

// ---------------------------------------------------------------------------
// Graph reconstruction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Cardinal(u8); // Factorio 1.0 4-way: 0=N, 2=E, 4=S, 6=W

impl Cardinal {
    fn step(self) -> (i32, i32) {
        match self.0 {
            0 => (0, -1),
            2 => (1, 0),
            4 => (0, 1),
            6 => (-1, 0),
            _ => unreachable!("invalid cardinal {}", self.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TileEntity {
    Belt { dir: Cardinal },
    SplitterAnchor { idx: usize },
    SplitterSecond { idx: usize },
    UgInput { dir: Cardinal, idx: usize },
    UgOutput { dir: Cardinal },
}

/// One node in the balancer's logical splitter graph. Public so the
/// topology generator in [`super::balancer_topology`] and any future
/// placement solver can construct graphs directly without going through
/// the entity-position layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
    InputPort(usize),
    OutputPort(usize),
    /// One whole splitter; flow through ≤ 2 (natural cap from edge count).
    Splitter(usize),
}

/// Logical splitter graph — abstract topology of an `(m, n)` balancer
/// with all physical-position information stripped away. Each splitter
/// is one node with up to 2 incoming edges and up to 2 outgoing edges.
#[derive(Debug, Clone)]
pub struct SplitterGraph {
    pub n_inputs: usize,
    pub n_outputs: usize,
    pub n_splitters: usize,
    /// Directed edges (from, to). Each edge carries one belt's worth of
    /// throughput (capacity 1).
    pub edges: Vec<(NodeId, NodeId)>,
}

/// One splitter that needs an input-priority annotation to avoid
/// discrete-time stalls in the real game. Returned by
/// [`detect_priority_needed`].
///
/// In our [`SplitterGraph`] model, splitters have two input ports
/// (`port 0` / `port 1`). Port assignment is the same convention used
/// by [`from_splitter_graph`]: incoming edges are matched to ports in
/// the order they appear in `graph.edges`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrioritySuggestion {
    /// Index into `SplitterGraph::n_splitters`.
    pub splitter: usize,
    /// Bitmask of feedback-loop ports: bit 0 = port 0 in feedback, bit 1
    /// = port 1 in feedback.
    pub feedback_ports: u8,
    /// Recommended priority port (0 or 1) — the *non*-feedback input
    /// gets priority so the feedback gets back-pressured. `None` when
    /// both ports are in feedback (symmetric case — a priority is still
    /// helpful but the choice is arbitrary).
    pub priority_port: Option<u8>,
}

/// Find splitters that have a feedback loop through one (or both) of
/// their inputs. Without an `input_priority` annotation in the placed
/// blueprint, these splitters can suffer transient discrete-time stalls
/// even when our scalar fluid-flow model says they balance perfectly.
///
/// Algorithm:
/// 1. Build forward reachability through splitters: for each splitter
///    S, the set of other splitters reachable from S's outputs.
/// 2. For each splitter S with two incoming edges, check whether the
///    edge sourcing each port comes from a splitter reachable from S.
///    A reachable source = an indirect cycle through S = a feedback
///    input that benefits from priority.
///
/// Pure feed-forward inputs aren't flagged. Single-input splitters
/// aren't flagged (no contention to resolve).
pub fn detect_priority_needed(graph: &SplitterGraph) -> Vec<PrioritySuggestion> {
    use rustc_hash::FxHashSet;

    // Forward adjacency: splitter idx → set of splitter targets
    let mut adj: Vec<FxHashSet<usize>> = vec![FxHashSet::default(); graph.n_splitters];
    for (src, dst) in &graph.edges {
        if let (NodeId::Splitter(si), NodeId::Splitter(di)) = (src, dst) {
            adj[*si].insert(*di);
        }
    }

    // For each splitter, BFS forward to compute the set of splitters
    // reachable from its outputs. (We could do this more efficiently
    // with SCCs, but n_splitters is small and BFS-per-node is fine.)
    let reach_from: Vec<FxHashSet<usize>> = (0..graph.n_splitters)
        .map(|s| {
            let mut seen = FxHashSet::default();
            let mut stack: Vec<usize> = adj[s].iter().copied().collect();
            while let Some(n) = stack.pop() {
                if seen.insert(n) {
                    for &t in &adj[n] {
                        if !seen.contains(&t) {
                            stack.push(t);
                        }
                    }
                }
            }
            seen
        })
        .collect();

    // For each splitter, group incoming edges by destination port using
    // the same convention as `from_splitter_graph`: first incoming edge
    // → port 0, second → port 1, further edges → port 1 sideloads.
    let mut in_port: Vec<[Option<NodeId>; 2]> = vec![[None, None]; graph.n_splitters];
    let mut next_port: Vec<[bool; 2]> = vec![[false, false]; graph.n_splitters];
    for (src, dst) in &graph.edges {
        if let NodeId::Splitter(s) = dst {
            if !next_port[*s][0] {
                next_port[*s][0] = true;
                in_port[*s][0] = Some(*src);
            } else if !next_port[*s][1] {
                next_port[*s][1] = true;
                in_port[*s][1] = Some(*src);
            }
        }
    }

    let mut out = Vec::new();
    for s in 0..graph.n_splitters {
        // Both ports must be wired for contention to matter.
        let (Some(p0_src), Some(p1_src)) = (in_port[s][0], in_port[s][1]) else {
            continue;
        };
        let in_feedback = |src: NodeId| -> bool {
            match src {
                NodeId::Splitter(si) => reach_from[s].contains(&si),
                _ => false,
            }
        };
        let p0_fb = in_feedback(p0_src);
        let p1_fb = in_feedback(p1_src);
        if !p0_fb && !p1_fb {
            continue;
        }
        let mut feedback_ports = 0u8;
        if p0_fb {
            feedback_ports |= 1;
        }
        if p1_fb {
            feedback_ports |= 2;
        }
        let priority_port = match (p0_fb, p1_fb) {
            (true, false) => Some(1),
            (false, true) => Some(0),
            _ => None,
        };
        out.push(PrioritySuggestion {
            splitter: s,
            feedback_ports,
            priority_port,
        });
    }
    out
}

/// Classify a logical splitter graph directly, skipping the
/// `recover_graph` step. Used by [`super::balancer_topology`] for graphs
/// constructed without a physical layout, and by phase 3 placement-solver
/// round-trip tests.
pub fn classify_graph(graph: &SplitterGraph) -> Result<ClassificationReport, ClassifyError> {
    let composition = compute_composition_matrix(graph)?;
    let m = graph.n_inputs;
    let n = graph.n_outputs;

    let target = 1.0 / n as f64;
    let is_mx3 = composition
        .iter()
        .all(|row| row.iter().all(|&v| (v - target).abs() < 1e-9));
    if is_mx3 {
        return Ok(ClassificationReport {
            class: BalancerClass::Balanced,
            composition,
            mx2_counterexample: None,
        });
    }

    let mx2a_counterexample = check_input_subsets(graph, m, n);
    if mx2a_counterexample.is_some() {
        return Ok(ClassificationReport {
            class: BalancerClass::ThroughputLimited,
            composition,
            mx2_counterexample: mx2a_counterexample,
        });
    }

    let mx2b_counterexample = check_output_subsets(graph, m, n);
    let class = if mx2b_counterexample.is_none() {
        BalancerClass::ThroughputUnlimited
    } else {
        BalancerClass::ThroughputBalancedRate
    };
    Ok(ClassificationReport {
        class,
        composition,
        mx2_counterexample: mx2b_counterexample,
    })
}

fn recover_graph(template: BalancerTemplateRef<'_>) -> Result<SplitterGraph, ClassifyError> {
    // ----- Build occupancy map -----
    let mut occ: FxHashMap<(i32, i32), TileEntity> = FxHashMap::default();
    let mut splitters: Vec<&BalancerTemplateEntity> = Vec::new();
    let mut ug_inputs: Vec<(i32, i32, Cardinal)> = Vec::new();

    let insert =
        |occ: &mut FxHashMap<(i32, i32), TileEntity>, tile: (i32, i32), e: TileEntity| -> Result<(), ClassifyError> {
            if occ.insert(tile, e).is_some() {
                Err(ClassifyError::Overlap { tile })
            } else {
                Ok(())
            }
        };

    for e in template.entities {
        let dir = Cardinal(e.direction);
        match e.name {
            "transport-belt" => {
                insert(&mut occ, (e.x, e.y), TileEntity::Belt { dir })?;
            }
            "splitter" => {
                let idx = splitters.len();
                splitters.push(e);
                let (sx, sy) = splitter_second(e.x, e.y, dir);
                insert(&mut occ, (e.x, e.y), TileEntity::SplitterAnchor { idx })?;
                insert(&mut occ, (sx, sy), TileEntity::SplitterSecond { idx })?;
            }
            "underground-belt" => match e.io_type {
                Some("input") => {
                    let idx = ug_inputs.len();
                    ug_inputs.push((e.x, e.y, dir));
                    insert(&mut occ, (e.x, e.y), TileEntity::UgInput { dir, idx })?;
                }
                Some("output") => {
                    insert(&mut occ, (e.x, e.y), TileEntity::UgOutput { dir })?;
                }
                _ => {
                    return Err(ClassifyError::Malformed(format!(
                        "underground-belt at ({}, {}) missing io_type",
                        e.x, e.y
                    )))
                }
            },
            other => {
                return Err(ClassifyError::Malformed(format!(
                    "unexpected entity '{other}' at ({}, {})",
                    e.x, e.y
                )))
            }
        }
    }

    // Pair UGs: for each input, walk forward in its direction until finding a
    // matching-direction UG output.
    let mut ug_pair: FxHashMap<usize, (i32, i32)> = FxHashMap::default();
    let max_search = (template.width + template.height) as i32 + 4;
    for (i, &(ix, iy, dir)) in ug_inputs.iter().enumerate() {
        let (dx, dy) = dir.step();
        let (mut tx, mut ty) = (ix + dx, iy + dy);
        let mut found = None;
        for _ in 0..max_search {
            if let Some(TileEntity::UgOutput { dir: out_dir }) = occ.get(&(tx, ty)) {
                if out_dir.0 == dir.0 {
                    found = Some((tx, ty));
                    break;
                }
            }
            tx += dx;
            ty += dy;
        }
        match found {
            Some(pos) => {
                ug_pair.insert(i, pos);
            }
            None => return Err(ClassifyError::UnpairedUg { input_at: (ix, iy) }),
        }
    }

    // Sideloads (B8 / U7) are accepted as valid flow merges. The walker
    // emits one edge per upstream flow source through any shared belt, so
    // multi-feeder tiles naturally produce one edge per feeder reaching the
    // downstream sink — flow conservation at splitters in the linear-system
    // composition handles the merge correctly. Lane-level semantics matter
    // for MX5 (lane throughput) but not for the belt-level MX1/MX2/MX3
    // classification done here.

    // ----- Build edges by walking forward from every flow source -----
    let mut edges: Vec<(NodeId, NodeId)> = Vec::new();

    // Input ports. A dangling input port (no downstream) drops its edge —
    // the input simply doesn't reach any output. This is captured by the
    // composition matrix returning zeroes for that input column.
    for (i, &(ix, iy)) in template.input_tiles.iter().enumerate() {
        if let Some(dst) = walk_into_neighbor(&occ, (ix, iy), &ug_pair, template)? {
            edges.push((NodeId::InputPort(i), dst));
        }
    }

    // Splitter outputs (≤2 per splitter; missing outputs drop their edge).
    for (idx, sp) in splitters.iter().enumerate() {
        let dir = Cardinal(sp.direction);
        let (dx, dy) = dir.step();
        let anchor_out = (sp.x + dx, sp.y + dy);
        let (ssx, ssy) = splitter_second(sp.x, sp.y, dir);
        let second_out = (ssx + dx, ssy + dy);
        for out_tile in [anchor_out, second_out] {
            if let Some(dst) = walk_into_neighbor(&occ, out_tile, &ug_pair, template)? {
                edges.push((NodeId::Splitter(idx), dst));
            }
        }
    }

    Ok(SplitterGraph {
        n_inputs: template.n_inputs as usize,
        n_outputs: template.n_outputs as usize,
        n_splitters: splitters.len(),
        edges,
    })
}

fn splitter_second(x: i32, y: i32, dir: Cardinal) -> (i32, i32) {
    match dir.0 {
        0 | 4 => (x + 1, y), // N/S → spans east-west
        _ => (x, y + 1),     // E/W → spans north-south
    }
}

/// Walk into `tile` and continue forward until reaching a sink (output port,
/// splitter input, or UG input that re-emerges and continues).
///
/// Returns `Ok(None)` for a dangling walk that ends on an empty tile, or
/// for a walk that loops back on itself (a literal belt cycle, possible
/// once side-loaded splitter outputs re-enter the network). Looping flow
/// has no well-defined sink in the saturated model — physically items
/// would just recirculate — so dropping the edge is the right behaviour
/// for our static analysis.
fn walk_into_neighbor(
    occ: &FxHashMap<(i32, i32), TileEntity>,
    mut tile: (i32, i32),
    ug_pair: &FxHashMap<usize, (i32, i32)>,
    template: BalancerTemplateRef<'_>,
) -> Result<Option<NodeId>, ClassifyError> {
    let mut visited: rustc_hash::FxHashSet<(i32, i32)> = rustc_hash::FxHashSet::default();
    loop {
        if let Some(out_idx) = template.output_tiles.iter().position(|&t| t == tile) {
            return Ok(Some(NodeId::OutputPort(out_idx)));
        }
        if !visited.insert(tile) {
            return Ok(None);
        }
        let Some(ent) = occ.get(&tile) else {
            return Ok(None);
        };
        match ent {
            TileEntity::Belt { dir } => {
                tile = step_tile(tile, *dir);
            }
            TileEntity::SplitterAnchor { idx, .. } | TileEntity::SplitterSecond { idx, .. } => {
                return Ok(Some(NodeId::Splitter(*idx)));
            }
            TileEntity::UgInput { idx, dir } => {
                let pair = ug_pair
                    .get(idx)
                    .ok_or(ClassifyError::UnpairedUg { input_at: tile })?;
                tile = step_tile(*pair, *dir);
            }
            TileEntity::UgOutput { dir } => {
                tile = step_tile(tile, *dir);
            }
        }
    }
}

fn step_tile(tile: (i32, i32), dir: Cardinal) -> (i32, i32) {
    let (dx, dy) = dir.step();
    (tile.0 + dx, tile.1 + dy)
}

// ---------------------------------------------------------------------------
// Composition matrix (MX3 check)
// ---------------------------------------------------------------------------

/// Build the m → n composition matrix under the saturated 50/50 splitter
/// model, by solving a linear system. This handles back-loops (universal-
/// balancer pattern) as well as feed-forward DAGs.
///
/// Variables: `x_i` = per-output-edge rate of splitter i.
/// For each splitter i: `out_degree(i) * x_i = sum of incoming-edge rates`.
/// Incoming edges from input port `p` contribute `1` if `p == k` (the input
/// being unit-tested), else `0`. Incoming edges from splitter j contribute
/// `x_j`.
///
/// In matrix form: `A x = b(k)`, with
///   `A[i][i] = out_degree(i)`,
///   `A[i][j] = -count_edges(splitter j → splitter i)` for `j != i`.
#[allow(clippy::needless_range_loop)]
fn compute_composition_matrix(graph: &SplitterGraph) -> Result<Vec<Vec<f64>>, ClassifyError> {
    let m = graph.n_inputs;
    let n = graph.n_outputs;
    let s = graph.n_splitters;

    // Pre-compute output degree per splitter.
    let mut out_degree = vec![0_i32; s];
    for (a, _) in &graph.edges {
        if let NodeId::Splitter(si) = a {
            out_degree[*si] += 1;
        }
    }

    // Build the LHS coefficient matrix A (independent of which input we're
    // unit-testing). A[i][i] = out_degree(i); A[i][j] -= count(j → i).
    let mut a_mat = vec![vec![0.0_f64; s]; s];
    for i in 0..s {
        a_mat[i][i] = out_degree[i] as f64;
    }
    for (src, dst) in &graph.edges {
        if let (NodeId::Splitter(j), NodeId::Splitter(i)) = (src, dst) {
            a_mat[*i][*j] -= 1.0;
        }
    }

    let mut composition = vec![vec![0.0_f64; m]; n];

    for k in 0..m {
        // Build per-input boundary vector b: b[i] = +1 for each edge
        // (InputPort(k) → Splitter(i)). Other input ports contribute 0.
        let mut b = vec![0.0_f64; s];
        for (src, dst) in &graph.edges {
            if let (NodeId::InputPort(p), NodeId::Splitter(i)) = (src, dst) {
                if *p == k {
                    b[*i] += 1.0;
                }
            }
        }

        // Solve A x = b. (Cloning A per-input keeps us simple; we could
        // factor once and back-substitute m times, but the cost is trivial
        // for s ≤ ~50.)
        let x = gauss_solve(&a_mat, &b).ok_or(ClassifyError::Singular)?;

        // Output port rates: sum of rates on incoming edges.
        // Edge from InputPort(p): contributes 1 if p == k else 0.
        // Edge from Splitter(j): contributes x[j].
        for j_out in 0..n {
            let mut r = 0.0_f64;
            for (src, dst) in &graph.edges {
                if let NodeId::OutputPort(j) = dst {
                    if *j == j_out {
                        match src {
                            NodeId::InputPort(p) => {
                                if *p == k {
                                    r += 1.0;
                                }
                            }
                            NodeId::Splitter(si) => {
                                r += x[*si];
                            }
                            NodeId::OutputPort(_) => unreachable!(),
                        }
                    }
                }
            }
            composition[j_out][k] = r;
        }
    }
    Ok(composition)
}

/// Gaussian elimination with partial pivoting. Returns `None` if the matrix
/// is singular (a row reduces to a near-zero pivot during elimination), so
/// the caller can distinguish "no solution" from "all-zero solution".
#[allow(clippy::needless_range_loop)]
fn gauss_solve(a_in: &[Vec<f64>], b_in: &[f64]) -> Option<Vec<f64>> {
    let n = a_in.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut a: Vec<Vec<f64>> = a_in.to_vec();
    let mut b: Vec<f64> = b_in.to_vec();
    for i in 0..n {
        let mut max_row = i;
        for r in (i + 1)..n {
            if a[r][i].abs() > a[max_row][i].abs() {
                max_row = r;
            }
        }
        a.swap(i, max_row);
        b.swap(i, max_row);
        if a[i][i].abs() < 1e-12 {
            return None;
        }
        for r in (i + 1)..n {
            let factor = a[r][i] / a[i][i];
            for c in i..n {
                a[r][c] -= factor * a[i][c];
            }
            b[r] -= factor * b[i];
        }
    }
    let mut x = vec![0.0_f64; n];
    for i in (0..n).rev() {
        let mut s = b[i];
        for j in (i + 1)..n {
            s -= a[i][j] * x[j];
        }
        x[i] = s / a[i][i];
    }
    Some(x)
}

// ---------------------------------------------------------------------------
// Max-flow (MX2 check) — Edmonds-Karp
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FlowGraph {
    adj: Vec<Vec<usize>>,
    edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Copy)]
struct FlowEdge {
    to: usize,
    cap: i32,
    flow: i32,
    rev: usize,
}

impl FlowGraph {
    fn with_nodes(n: usize) -> Self {
        Self {
            adj: vec![Vec::new(); n],
            edges: Vec::new(),
        }
    }
    fn add_edge(&mut self, from: usize, to: usize, cap: i32) {
        let f_idx = self.edges.len();
        let r_idx = f_idx + 1;
        self.edges.push(FlowEdge {
            to,
            cap,
            flow: 0,
            rev: r_idx,
        });
        self.edges.push(FlowEdge {
            to: from,
            cap: 0,
            flow: 0,
            rev: f_idx,
        });
        self.adj[from].push(f_idx);
        self.adj[to].push(r_idx);
    }
    fn max_flow(&mut self, source: usize, sink: usize) -> i32 {
        let mut total = 0;
        loop {
            let n = self.adj.len();
            let mut parent: Vec<Option<(usize, usize)>> = vec![None; n];
            parent[source] = Some((source, usize::MAX));
            let mut q: VecDeque<usize> = VecDeque::new();
            q.push_back(source);
            while let Some(u) = q.pop_front() {
                for &eid in &self.adj[u] {
                    let e = &self.edges[eid];
                    if parent[e.to].is_none() && e.cap - e.flow > 0 {
                        parent[e.to] = Some((u, eid));
                        q.push_back(e.to);
                    }
                }
            }
            if parent[sink].is_none() {
                break;
            }
            let mut bottleneck = i32::MAX;
            let mut v = sink;
            while v != source {
                let (u, eid) = parent[v].unwrap();
                let e = &self.edges[eid];
                bottleneck = bottleneck.min(e.cap - e.flow);
                v = u;
            }
            v = sink;
            while v != source {
                let (u, eid) = parent[v].unwrap();
                self.edges[eid].flow += bottleneck;
                let rev = self.edges[eid].rev;
                self.edges[rev].flow -= bottleneck;
                v = u;
            }
            total += bottleneck;
        }
        total
    }
}

/// Build the *base* flow graph: nodes for input ports, output ports,
/// splitters, plus a super-source (0) and super-sink (1). Source/sink edges
/// are added per-subset by the caller.
fn build_flow_graph(graph: &SplitterGraph) -> (FlowGraph, Vec<usize>, Vec<usize>) {
    let m = graph.n_inputs;
    let n = graph.n_outputs;
    let s_in_base = 2;
    let s_out_base = 2 + m;
    let sp_base = 2 + m + n;
    let total = sp_base + graph.n_splitters;

    let mut fg = FlowGraph::with_nodes(total);

    let id_of = |nd: NodeId| -> usize {
        match nd {
            NodeId::InputPort(i) => s_in_base + i,
            NodeId::OutputPort(j) => s_out_base + j,
            NodeId::Splitter(s) => sp_base + s,
        }
    };
    for (a, b) in &graph.edges {
        fg.add_edge(id_of(*a), id_of(*b), 1);
    }
    // Splitter natural cap = 2 (from edge counts). No node-splitting needed.

    let inputs: Vec<usize> = (0..m).map(|i| s_in_base + i).collect();
    let outputs: Vec<usize> = (0..n).map(|j| s_out_base + j).collect();
    (fg, inputs, outputs)
}

fn run_subset_flow(
    base: &FlowGraph,
    inputs: &[usize],
    outputs: &[usize],
    selected_inputs: &[usize],
    selected_outputs: &[usize],
) -> i32 {
    let mut fg = base.clone();
    for &i in selected_inputs {
        fg.add_edge(0, inputs[i], 1);
    }
    for &j in selected_outputs {
        fg.add_edge(outputs[j], 1, 1);
    }
    fg.max_flow(0, 1)
}

fn check_input_subsets(
    graph: &SplitterGraph,
    m: usize,
    n: usize,
) -> Option<Mx2Counterexample> {
    if m > 16 {
        return None;
    }
    let (base, inputs, outputs) = build_flow_graph(graph);
    let all_outputs: Vec<usize> = (0..n).collect();
    for mask in 1u64..(1u64 << m) {
        let s: Vec<usize> = (0..m).filter(|i| (mask >> i) & 1 == 1).collect();
        let expected = s.len().min(n) as i32;
        let actual = run_subset_flow(&base, &inputs, &outputs, &s, &all_outputs);
        if actual < expected {
            return Some(Mx2Counterexample {
                direction: Mx2Direction::InputSubset,
                subset: s,
                realized: actual.max(0) as u32,
                expected: expected as u32,
            });
        }
    }
    None
}

fn check_output_subsets(
    graph: &SplitterGraph,
    m: usize,
    n: usize,
) -> Option<Mx2Counterexample> {
    if n > 16 {
        return None;
    }
    let (base, inputs, outputs) = build_flow_graph(graph);
    let all_inputs: Vec<usize> = (0..m).collect();
    for mask in 1u64..(1u64 << n) {
        let t: Vec<usize> = (0..n).filter(|j| (mask >> j) & 1 == 1).collect();
        let expected = m.min(t.len()) as i32;
        let actual = run_subset_flow(&base, &inputs, &outputs, &all_inputs, &t);
        if actual < expected {
            return Some(Mx2Counterexample {
                direction: Mx2Direction::OutputSubset,
                subset: t,
                realized: actual.max(0) as u32,
                expected: expected as u32,
            });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::balancer_library::balancer_templates;

    /// `detect_priority_needed` flags splitters whose inputs are part of
    /// a feedback loop. The minimal example: two splitters mutually
    /// feeding each other's port 1 (Couëtoux Figure 1c-style).
    #[test]
    fn priority_detection_simple_feedback_loop() {
        // L: in (i0, R.out1), out (o0, R.in1)
        // R: in (i1, L.out1), out (o1, L.in1)
        let g = SplitterGraph {
            n_inputs: 2,
            n_outputs: 2,
            n_splitters: 2,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::InputPort(1), NodeId::Splitter(1)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
                (NodeId::Splitter(0), NodeId::Splitter(1)),
                (NodeId::Splitter(1), NodeId::OutputPort(1)),
                (NodeId::Splitter(1), NodeId::Splitter(0)),
            ],
        };
        let suggestions = detect_priority_needed(&g);
        assert_eq!(suggestions.len(), 2);
        // Both splitters: port 0 = external input (non-feedback),
        // port 1 = feedback from the other splitter.
        for s in &suggestions {
            assert_eq!(s.feedback_ports, 0b10, "expected port 1 in feedback");
            assert_eq!(s.priority_port, Some(0));
        }
    }

    /// Pure feed-forward graph (no feedback loops) → no priority
    /// suggestions.
    #[test]
    fn priority_detection_no_feedback() {
        // Single splitter: 2 inputs → 2 outputs, no loops.
        let g = SplitterGraph {
            n_inputs: 2,
            n_outputs: 2,
            n_splitters: 1,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::InputPort(1), NodeId::Splitter(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(1)),
            ],
        };
        assert!(detect_priority_needed(&g).is_empty());
    }

    #[test]
    fn classify_smoke_each_template() {
        // Each template either classifies cleanly with correct dimensions, or
        // returns a structural diagnostic. Print categorical counts so the
        // shape of the corpus is visible.
        let mut ok = 0;
        let mut cycle = 0;
        let mut dangling = 0;
        let mut unpaired_ug = 0;
        let mut overlap = 0;
        let mut malformed = 0;
        let mut singular = 0;
        for t in balancer_templates().values() {
            match classify(t) {
                Ok(r) => {
                    assert_eq!(r.composition.len(), t.n_outputs as usize);
                    for row in &r.composition {
                        assert_eq!(row.len(), t.n_inputs as usize);
                    }
                    ok += 1;
                }
                Err(ClassifyError::Cycle { .. }) => cycle += 1,
                Err(ClassifyError::DanglingBelt { .. }) => dangling += 1,
                Err(ClassifyError::UnpairedUg { .. }) => unpaired_ug += 1,
                Err(ClassifyError::Overlap { .. }) => overlap += 1,
                Err(ClassifyError::Malformed(_)) => malformed += 1,
                Err(ClassifyError::Singular) => singular += 1,
            }
        }
        assert!(ok > 0, "no templates classified");
        eprintln!(
            "classify smoke: ok={ok} cycle={cycle} dangling={dangling} \
             unpaired_ug={unpaired_ug} overlap={overlap} \
             malformed={malformed} singular={singular}"
        );
    }

    /// Diagnostic dump for the templates that don't classify as MX3.
    /// Kept as a runnable test (rather than removing) so future investigations
    /// don't need to re-derive the trace from scratch.
    #[test]
    fn investigate_mx1_and_mx2() {
        // Print full diagnostics for non-MX3 cases.
        for ((m, n), t) in balancer_templates() {
            let r = match classify(t) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if matches!(
                r.class,
                BalancerClass::ThroughputLimited | BalancerClass::ThroughputUnlimited
            ) {
                eprintln!();
                eprintln!("=== ({m}, {n}) class={:?} ===", r.class);
                eprintln!("composition (rows=outputs, cols=inputs):");
                for row in &r.composition {
                    let cells: Vec<String> =
                        row.iter().map(|v| format!("{v:.4}")).collect();
                    eprintln!("  [{}]", cells.join(", "));
                }
                if let Some(ce) = &r.mx2_counterexample {
                    eprintln!("mx2 counterexample: {ce:?}");
                }
            }
        }
    }

    /// Audit report: classify every template and print a markdown table
    /// of `(m, n) → class`. Run with `--nocapture` to copy into the RFP
    /// decision log. This test only asserts the classifier doesn't panic;
    /// the report itself is the deliverable.
    #[test]
    fn audit_report() {
        #[derive(Debug)]
        enum Outcome {
            Class(BalancerClass),
            Singular,
            Cycle,
            Other(String),
        }

        let mut rows: Vec<((u32, u32), Outcome, u32, u32)> = Vec::new();
        for ((m, n), t) in balancer_templates() {
            let entity_count = t.entities.len() as u32;
            let area = t.width * t.height;
            let outcome = match classify(t) {
                Ok(r) => Outcome::Class(r.class),
                Err(ClassifyError::Cycle { .. }) => Outcome::Cycle,
                Err(ClassifyError::Singular) => Outcome::Singular,
                Err(e) => Outcome::Other(format!("{e:?}")),
            };
            rows.push(((*m, *n), outcome, entity_count, area));
        }
        rows.sort_by_key(|((m, n), ..)| (*m, *n));

        let mut counts: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        eprintln!();
        eprintln!("| (m, n) | class | entities | footprint |");
        eprintln!("|--------|-------|----------|-----------|");
        for ((m, n), outcome, entities, area) in &rows {
            let label = match outcome {
                Outcome::Class(BalancerClass::Balanced) => {
                    *counts.entry("MX3 balanced").or_insert(0) += 1;
                    "MX3 balanced".to_string()
                }
                Outcome::Class(BalancerClass::ThroughputUnlimited) => {
                    *counts.entry("MX2b throughput-unlimited").or_insert(0) += 1;
                    "MX2b throughput-unlimited".to_string()
                }
                Outcome::Class(BalancerClass::ThroughputBalancedRate) => {
                    *counts
                        .entry("MX2a saturation + balanced rate")
                        .or_insert(0) += 1;
                    "MX2a sat+balanced".to_string()
                }
                Outcome::Class(BalancerClass::ThroughputLimited) => {
                    *counts.entry("MX1 throughput-limited").or_insert(0) += 1;
                    "MX1 throughput-limited".to_string()
                }
                Outcome::Cycle => {
                    *counts.entry("kill: cycle").or_insert(0) += 1;
                    "kill: cycle".to_string()
                }
                Outcome::Singular => {
                    *counts.entry("kill: singular linear system").or_insert(0) += 1;
                    "kill: singular".to_string()
                }
                Outcome::Other(s) => {
                    *counts.entry("kill: other").or_insert(0) += 1;
                    format!("kill: {s}")
                }
            };
            eprintln!("| ({m}, {n}) | {label} | {entities} | {area} |");
        }
        eprintln!();
        eprintln!("| class | count |");
        eprintln!("|-------|-------|");
        for (k, v) in &counts {
            eprintln!("| {k} | {v} |");
        }
        eprintln!("| total | {} |", rows.len());
        eprintln!();
    }

    #[test]
    fn one_to_two_is_balanced() {
        let t = &balancer_templates()[&(1, 2)];
        let r = classify(t).unwrap();
        assert_eq!(r.class, BalancerClass::Balanced);
    }

    #[test]
    fn two_to_two_is_balanced() {
        let t = &balancer_templates()[&(2, 2)];
        let r = classify(t).unwrap();
        assert_eq!(r.class, BalancerClass::Balanced);
    }
}
