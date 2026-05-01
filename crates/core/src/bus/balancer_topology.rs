//! Logical topology generators for balancer graphs (phase 3.0 of
//! [`docs/rfp-balancer-graph-place.md`]).
//!
//! Emits [`SplitterGraph`] instances that describe an `(m, n)` balancer's
//! splitter network *without* committing to a physical placement. The
//! placement solver in a future phase consumes these graphs and produces
//! `BalancerTemplate`-compatible layouts via CP-SAT; this module is
//! independently testable against [`classify_graph`] without any placement
//! logic.
//!
//! Phase 3.0 scope:
//!   - `passthrough(m)` — m parallel paths, no splitters.
//!   - `one_to_two()`, `two_to_one()` — single-splitter atoms.
//!   - `parallel(graphs)` — disjoint composition.
//!   - `series(a, b)` — feed `a`'s outputs into `b`'s inputs (1-to-1
//!     wiring, no permutation).
//!
//! Coprime topologies (back-loop universal-balancer constructions) are
//! deferred to phase 3.1+. They will need a more elaborate construction
//! recipe — likely composing the existing library's `(1, 3)` / `(4, 3)`
//! topologies via [`recover_topology_from_library`] until the standalone
//! generator catches up.

use crate::bus::balancer_classify::{
    topology_of_template, BalancerTemplateRef, ClassifyError, NodeId, SplitterGraph,
};
use crate::bus::balancer_library::balancer_templates;

/// Bootstrap atom: extract the logical topology from a known-good library
/// template. Used by the Clos-style composers to build coprime balancers
/// out of small library atoms (e.g. `library_atom(1, 3)` for the back-loop
/// `1→3` building block) before the standalone universal-balancer
/// generator catches up.
///
/// Returns `None` if the library doesn't have `(m, n)`.
pub fn library_atom(m: u32, n: u32) -> Option<SplitterGraph> {
    let t = balancer_templates().get(&(m, n))?;
    topology_of_template(BalancerTemplateRef::from(t)).ok()
}

/// Same as [`library_atom`] but returns the underlying error if the
/// recovery fails — useful when you want to surface a malformed-library
/// finding rather than silently fall back.
pub fn library_atom_strict(m: u32, n: u32) -> Result<SplitterGraph, LibraryAtomError> {
    let templates = balancer_templates();
    let t = templates
        .get(&(m, n))
        .ok_or(LibraryAtomError::ShapeNotInLibrary { m, n })?;
    topology_of_template(BalancerTemplateRef::from(t))
        .map_err(LibraryAtomError::Recovery)
}

#[derive(Debug)]
pub enum LibraryAtomError {
    ShapeNotInLibrary { m: u32, n: u32 },
    Recovery(ClassifyError),
}

/// `m` parallel paths from input port `i` to output port `i`. No
/// splitters. Trivially MX2b — every input has a unique output, max-flow
/// holds in both directions.
pub fn passthrough(m: usize) -> SplitterGraph {
    let edges: Vec<_> = (0..m)
        .map(|i| (NodeId::InputPort(i), NodeId::OutputPort(i)))
        .collect();
    SplitterGraph {
        n_inputs: m,
        n_outputs: m,
        n_splitters: 0,
        edges,
    }
}

/// Single splitter taking 1 input and producing 2 outputs.
/// Splitter index 0 has 1 incoming edge from input port 0 and 2 outgoing
/// edges to output ports 0 and 1. Classifier reports MX3 (each output is
/// 1/n = 1/2 of the single input).
pub fn one_to_two() -> SplitterGraph {
    SplitterGraph {
        n_inputs: 1,
        n_outputs: 2,
        n_splitters: 1,
        edges: vec![
            (NodeId::InputPort(0), NodeId::Splitter(0)),
            (NodeId::Splitter(0), NodeId::OutputPort(0)),
            (NodeId::Splitter(0), NodeId::OutputPort(1)),
        ],
    }
}

/// Single splitter taking 2 inputs and producing 1 output. The splitter's
/// other output is intentionally absent (no edge) — items there would be
/// lost in physical placement, but per S5 the splitter routes everything
/// to the connected output, capping throughput at 1.
pub fn two_to_one() -> SplitterGraph {
    SplitterGraph {
        n_inputs: 2,
        n_outputs: 1,
        n_splitters: 1,
        edges: vec![
            (NodeId::InputPort(0), NodeId::Splitter(0)),
            (NodeId::InputPort(1), NodeId::Splitter(0)),
            (NodeId::Splitter(0), NodeId::OutputPort(0)),
        ],
    }
}

/// Stack `count` copies of `atom` into a disjoint parallel composition.
/// Inputs and outputs are renumbered so copy `c` of an `(m_a, n_a)` atom
/// occupies inputs `c·m_a..(c+1)·m_a` and outputs `c·n_a..(c+1)·n_a`.
/// The result has `count·m_a` inputs, `count·n_a` outputs, and
/// `count·n_splitters_a` splitters.
///
/// Classification: same as a single atom for input subsets confined to one
/// copy. For input subsets that span copies, it's still saturating
/// (each copy delivers independently). For output subsets confined to one
/// copy, max-flow is bounded by that copy's `m_a` (not the full `m`),
/// which is why disjoint parallel atoms are MX2a not MX2b.
pub fn parallel(atom: &SplitterGraph, count: usize) -> SplitterGraph {
    let m_a = atom.n_inputs;
    let n_a = atom.n_outputs;
    let s_a = atom.n_splitters;

    let mut edges: Vec<(NodeId, NodeId)> = Vec::with_capacity(atom.edges.len() * count);
    for c in 0..count {
        let in_off = c * m_a;
        let out_off = c * n_a;
        let sp_off = c * s_a;
        for &(a, b) in &atom.edges {
            edges.push((shift_node(a, in_off, out_off, sp_off), shift_node(b, in_off, out_off, sp_off)));
        }
    }
    SplitterGraph {
        n_inputs: m_a * count,
        n_outputs: n_a * count,
        n_splitters: s_a * count,
        edges,
    }
}

/// Connect `a`'s outputs 1-to-1 to `b`'s inputs (requires
/// `a.n_outputs == b.n_inputs`). Equivalent to
/// [`series_permuted(a, b, &identity)`].
pub fn series(a: &SplitterGraph, b: &SplitterGraph) -> SplitterGraph {
    let identity: Vec<usize> = (0..a.n_outputs).collect();
    series_permuted(a, b, &identity)
}

/// Connect `a`'s outputs to `b`'s inputs through a permutation: edge
/// `a.OutputPort(i) → b.InputPort(perm[i])`. `perm` must be a valid
/// permutation of `0..a.n_outputs`.
///
/// This is the Clos-style composer: stage 1's parallel-tree outputs are
/// re-ordered so each stage-2 sub-balancer receives one belt from each
/// stage-1 tree, mixing inputs across the network for true balanced
/// composition.
pub fn series_permuted(
    a: &SplitterGraph,
    b: &SplitterGraph,
    perm: &[usize],
) -> SplitterGraph {
    assert_eq!(a.n_outputs, b.n_inputs, "series_permuted: a.n_outputs must equal b.n_inputs");
    assert_eq!(perm.len(), a.n_outputs, "perm length must match a.n_outputs");
    {
        let mut seen = vec![false; perm.len()];
        for &p in perm {
            assert!(p < perm.len(), "perm entry {p} out of range");
            assert!(!seen[p], "perm has duplicate entry {p}");
            seen[p] = true;
        }
    }

    let s_a = a.n_splitters;
    let mut edges: Vec<(NodeId, NodeId)> =
        Vec::with_capacity(a.edges.len() + b.edges.len());

    // Resolve each b.InputPort(j) to its downstream destination in b.
    let mut b_input_dest: Vec<Option<NodeId>> = vec![None; b.n_inputs];
    for &(src, dst) in &b.edges {
        if let NodeId::InputPort(j) = src {
            b_input_dest[j] = Some(shift_b_node(dst, s_a));
        }
    }

    // Rewrite `a.OutputPort(i)` destinations to `b_input_dest[perm[i]]`.
    for &(src, dst) in &a.edges {
        let new_dst = match dst {
            NodeId::OutputPort(i) => b_input_dest[perm[i]]
                .expect("b's InputPort has no outgoing edge"),
            other => other,
        };
        edges.push((src, new_dst));
    }

    // Carry over b's internal edges (splitter→splitter, splitter→output).
    // Skip b's input-port edges: they were folded above.
    for &(src, dst) in &b.edges {
        if matches!(src, NodeId::InputPort(_)) {
            continue;
        }
        edges.push((shift_b_node(src, s_a), shift_b_node(dst, s_a)));
    }

    SplitterGraph {
        n_inputs: a.n_inputs,
        n_outputs: b.n_outputs,
        n_splitters: a.n_splitters + b.n_splitters,
        edges,
    }
}

/// Permutation that re-orders parallel-tree outputs so each downstream
/// sub-balancer in a Clos network receives one belt from each tree.
///
/// For `m` parallel trees each with `k` outputs (so `m·k` total outputs
/// at the output of stage 1), and a stage 2 of `k` parallel sub-balancers
/// each taking `m` inputs, the right permutation maps stage-1 output
/// `i = tree·k + slot` to stage-2 input `slot·m + tree`.
///
/// Returns a `Vec<usize>` of length `m * k`.
pub fn clos_interleave(m: usize, k: usize) -> Vec<usize> {
    let mut perm = vec![0usize; m * k];
    for tree in 0..m {
        for slot in 0..k {
            perm[tree * k + slot] = slot * m + tree;
        }
    }
    perm
}

fn shift_node(n: NodeId, in_off: usize, out_off: usize, sp_off: usize) -> NodeId {
    match n {
        NodeId::InputPort(i) => NodeId::InputPort(i + in_off),
        NodeId::OutputPort(j) => NodeId::OutputPort(j + out_off),
        NodeId::Splitter(s) => NodeId::Splitter(s + sp_off),
    }
}

fn shift_b_node(n: NodeId, s_a: usize) -> NodeId {
    match n {
        // b's input ports were folded into a's outputs; should not appear.
        NodeId::InputPort(_) => n,
        NodeId::OutputPort(j) => NodeId::OutputPort(j),
        NodeId::Splitter(s) => NodeId::Splitter(s + s_a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::balancer_classify::{classify_graph, BalancerClass};

    #[test]
    fn passthrough_classifies_as_mx2b() {
        for m in 2..=10 {
            let g = passthrough(m);
            let r = classify_graph(&g).unwrap();
            assert_eq!(
                r.class,
                BalancerClass::ThroughputUnlimited,
                "passthrough({m}) class = {:?}",
                r.class
            );
            assert_eq!(g.n_splitters, 0);
            assert_eq!(g.edges.len(), m);
        }
    }

    #[test]
    fn one_to_two_is_mx3() {
        let g = one_to_two();
        let r = classify_graph(&g).unwrap();
        assert_eq!(r.class, BalancerClass::Balanced);
        // Composition: each output gets 1/2 of the single input.
        assert!((r.composition[0][0] - 0.5).abs() < 1e-9);
        assert!((r.composition[1][0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn two_to_one_is_mx3_under_linear_model() {
        // Both inputs deliver a unit; the linear model treats the
        // single-output splitter as out_degree=1, so each input
        // contributes 1.0 to the output. n=1 so target = 1/n = 1.
        let g = two_to_one();
        let r = classify_graph(&g).unwrap();
        assert_eq!(r.class, BalancerClass::Balanced);
    }

    #[test]
    fn parallel_one_to_two_is_mx2a() {
        // Disjoint atoms: each output is fed by exactly one input. MX2a
        // (saturation + balanced rate) but not MX2b (output-subsets
        // confined to one tree have max-flow 1, not |T|).
        for m in 2..=5 {
            let g = parallel(&one_to_two(), m);
            let r = classify_graph(&g).unwrap();
            assert!(
                matches!(r.class, BalancerClass::ThroughputBalancedRate),
                "parallel one_to_two ×{m}: {:?}",
                r.class
            );
        }
    }

    #[test]
    fn series_chains_correctly() {
        // 1 → 2 → 4 (via two stacked 1→2 atoms in series doesn't typecheck
        // because a.n_outputs (2) != b.n_inputs (1) for one_to_two).
        // Use parallel(one_to_two(), 2) + one_to_two() — no, dimensions
        // still mismatch. Instead: series(one_to_two(), parallel(one_to_two(), 2)):
        // a is (1, 2), b is (2, 4). a.n_outputs == b.n_inputs == 2. ✓
        // Result is (1, 4) — a binary 1→4 tree.
        let a = one_to_two();
        let b = parallel(&one_to_two(), 2);
        let g = series(&a, &b);
        assert_eq!(g.n_inputs, 1);
        assert_eq!(g.n_outputs, 4);
        assert_eq!(g.n_splitters, 3);

        let r = classify_graph(&g).unwrap();
        assert_eq!(r.class, BalancerClass::Balanced);
        for j in 0..4 {
            assert!(
                (r.composition[j][0] - 0.25).abs() < 1e-9,
                "1→4 binary tree: output {j} contribution from input 0 = {}",
                r.composition[j][0]
            );
        }
    }

    /// Round-trip: classify_graph on a topology recovered from a library
    /// template must match classify on the original template. Sanity-check
    /// that the `SplitterGraph` abstraction is faithful to physical
    /// templates — phase 3.1's placement solver will rely on this contract.
    #[test]
    fn round_trip_library_templates() {
        use crate::bus::balancer_classify::classify;
        use crate::bus::balancer_library::balancer_templates;

        for &(m, n) in &[(1, 2), (2, 2), (1, 3), (2, 3), (4, 4), (3, 5), (4, 8)] {
            let t = &balancer_templates()[&(m, n)];
            let original = classify(t).unwrap();
            let topology = library_atom(m, n).unwrap();
            let recovered = classify_graph(&topology).unwrap();
            assert_eq!(
                original.class, recovered.class,
                "({m}, {n}): classify={:?} but classify_graph(topology)={:?}",
                original.class, recovered.class
            );
        }
    }

    #[test]
    fn library_atom_missing_shape_is_none() {
        // (9, 9) is not in the library; my generator's passthrough is
        // separate. library_atom should report None.
        assert!(library_atom(9, 9).is_none());
    }

    /// The headline coprime case: `(4, 9)` via Clos composition of
    /// library atoms. This confirms the topology layer can express
    /// coprime balancers correctly *before* any placement work.
    ///
    /// Construction:
    ///   stage1 = parallel(library(1, 3), 4)         # (4, 12)
    ///   stage2 = parallel(library(4, 3), 3)         # (12, 9)
    ///   network = series_permuted(stage1, stage2, clos_interleave(4, 3))
    ///
    /// The interleave makes each sub-balancer in stage 2 take one belt
    /// from each tree in stage 1 — the symmetric mixing that buys MX3.
    #[test]
    fn coprime_4_9_via_clos_composition() {
        let stage1 = parallel(&library_atom(1, 3).unwrap(), 4);
        assert_eq!(stage1.n_inputs, 4);
        assert_eq!(stage1.n_outputs, 12);

        let stage2 = parallel(&library_atom(4, 3).unwrap(), 3);
        assert_eq!(stage2.n_inputs, 12);
        assert_eq!(stage2.n_outputs, 9);

        let perm = clos_interleave(4, 3);
        let network = series_permuted(&stage1, &stage2, &perm);
        assert_eq!(network.n_inputs, 4);
        assert_eq!(network.n_outputs, 9);

        let report = classify_graph(&network).unwrap();
        // For a Clos composition of MX3 atoms with the symmetric
        // interleave, every output is the 1/4 mixed combination of every
        // input — that's MX3.
        assert_eq!(
            report.class,
            BalancerClass::Balanced,
            "(4, 9) Clos composition class = {:?}, composition = {:?}",
            report.class,
            report.composition
        );

        // Splitter count: 4 × library(1,3).n_splitters
        // + 3 × library(4,3).n_splitters.
        let lib_1_3 = library_atom(1, 3).unwrap();
        let lib_4_3 = library_atom(4, 3).unwrap();
        let expected = 4 * lib_1_3.n_splitters + 3 * lib_4_3.n_splitters;
        assert_eq!(network.n_splitters, expected);
        eprintln!(
            "(4, 9) Clos: {} splitters ({}+{}), {} edges",
            network.n_splitters,
            4 * lib_1_3.n_splitters,
            3 * lib_4_3.n_splitters,
            network.edges.len()
        );
    }

    /// Sanity-check: the same Clos pattern works for other coprime
    /// shapes. `(3, 7)` via parallel(library(1, k), 3) + parallel(library(3, k'), ?) —
    /// not directly applicable since k must divide n cleanly. Instead use
    /// `(3, 12)` via parallel(library(1, 4), 3) + parallel(library(3, 3), 4)?
    /// `(3, 3)` square is trivial. Let's try `(3, 5)` via library
    /// composition: (3, 15) / (15, 5) = (3, 5).
    /// (3, 15) = 3 × (1, 5). library has (1, 5).
    /// (15, 5) = 5 × (3, 1). library has (3, 1).
    /// clos_interleave(3, 5) on stage 1's 15 outputs → stage 2's 15 inputs.
    #[test]
    fn coprime_3_5_via_clos_composition() {
        let stage1 = parallel(&library_atom(1, 5).unwrap(), 3);
        assert_eq!(stage1.n_inputs, 3);
        assert_eq!(stage1.n_outputs, 15);

        let stage2 = parallel(&library_atom(3, 1).unwrap(), 5);
        assert_eq!(stage2.n_inputs, 15);
        assert_eq!(stage2.n_outputs, 5);

        let perm = clos_interleave(3, 5);
        let network = series_permuted(&stage1, &stage2, &perm);
        assert_eq!(network.n_inputs, 3);
        assert_eq!(network.n_outputs, 5);

        let report = classify_graph(&network).unwrap();
        assert_eq!(
            report.class,
            BalancerClass::Balanced,
            "(3, 5) Clos: class = {:?}",
            report.class
        );

        eprintln!(
            "(3, 5) Clos: {} splitters, {} edges",
            network.n_splitters,
            network.edges.len()
        );
    }
}
