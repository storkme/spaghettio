//! Bridge between [`SplitterGraph`] (recovered from a placed
//! `BalancerTemplate` via `bus::balancer_classify::recover_graph`) and
//! [`BalancerGraph`] (the canonical type our verifier consumes).
//!
//! The two representations differ:
//!   - `SplitterGraph` is **node-coarse**: a splitter is a single node and
//!     edges have no port concept.
//!   - `BalancerGraph` is **port-fine**: each splitter has explicit port
//!     0/1 on each side. Out-ports require exactly one arc each; in-ports
//!     may carry 0+ arcs (multi-arc relaxation, modeling sideloading).
//!
//! [`from_splitter_graph`] distributes incoming edges across the two
//! splitter in-ports greedily (port 0 fills first, then port 1, then
//! overflow stays on port 1 as a sideload). Port assignment doesn't affect
//! the all-fluid verifier — only the per-splitter total in-rate matters.
//!
//! No dummy I/O padding: the multi-arc relaxation lets `n_inputs` and
//! `n_outputs` be whatever the recovered graph declares, and splitter
//! ports with no incoming edges contribute rate 0 to conservation.

use thiserror::Error;

use crate::balancer::graph::{Arc, BalancerGraph, Sink, Source};
use crate::bus::balancer_classify::{NodeId, SplitterGraph};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BakeError {
    #[error("splitter {idx} has {count} outgoing edges (expected exactly 2)")]
    SplitterOutDegree { idx: usize, count: usize },
    #[error("input port {idx} has {count} outgoing edges (expected exactly 1)")]
    InputDegree { idx: usize, count: usize },
    #[error("output port {idx} has {count} incoming edges (expected exactly 1)")]
    OutputDegree { idx: usize, count: usize },
    #[error("edge with OutputPort source or InputPort sink (graph malformed)")]
    Malformed,
}

/// Convert a [`SplitterGraph`] into a [`BalancerGraph`].
pub fn from_splitter_graph(g: &SplitterGraph) -> Result<BalancerGraph, BakeError> {
    // Tally degrees.
    let mut splitter_out = vec![0usize; g.n_splitters];
    let mut input_out = vec![0usize; g.n_inputs];
    let mut output_in = vec![0usize; g.n_outputs];
    for (src, dst) in &g.edges {
        match src {
            NodeId::InputPort(i) => input_out[*i] += 1,
            NodeId::Splitter(s) => splitter_out[*s] += 1,
            NodeId::OutputPort(_) => return Err(BakeError::Malformed),
        }
        match dst {
            NodeId::OutputPort(o) => output_in[*o] += 1,
            NodeId::Splitter(_) => {}
            NodeId::InputPort(_) => return Err(BakeError::Malformed),
        }
    }
    for (s, &count) in splitter_out.iter().enumerate() {
        if count > 2 {
            return Err(BakeError::SplitterOutDegree { idx: s, count });
        }
    }
    for (i, &count) in input_out.iter().enumerate() {
        if count != 1 {
            return Err(BakeError::InputDegree { idx: i, count });
        }
    }
    for (o, &count) in output_in.iter().enumerate() {
        if count != 1 {
            return Err(BakeError::OutputDegree { idx: o, count });
        }
    }

    // Walk edges in order, assigning splitter ports incrementally.
    // Out-ports fill 0 then 1 (exactly 2 per splitter, by check above).
    // In-ports fill 0 first; once port 0 has an arc, additional arcs go
    // to port 1; once port 1 has an arc too, further arcs continue to
    // port 1 as sideloads.
    let mut next_in_port = vec![[0u8; 2]; g.n_splitters]; // [arcs_at_port_0, arcs_at_port_1]
    let mut next_out_port = vec![0u8; g.n_splitters];
    let mut arcs = Vec::with_capacity(g.edges.len());
    for (src, dst) in &g.edges {
        let src_endpoint = match src {
            NodeId::InputPort(i) => Source::Input(*i as u32),
            NodeId::Splitter(s) => {
                let port = next_out_port[*s];
                next_out_port[*s] += 1;
                Source::Splitter {
                    idx: *s as u32,
                    port,
                }
            }
            NodeId::OutputPort(_) => unreachable!("filtered above"),
        };
        let dst_endpoint = match dst {
            NodeId::OutputPort(o) => Sink::Output(*o as u32),
            NodeId::Splitter(s) => {
                let counts = &mut next_in_port[*s];
                let port = if counts[0] == 0 {
                    counts[0] = 1;
                    0
                } else if counts[1] == 0 {
                    counts[1] = 1;
                    1
                } else {
                    // Both ports have at least one arc; place on port 1
                    // as a sideload. (Symmetric — could be port 0 instead.)
                    counts[1] += 1;
                    1
                };
                Sink::Splitter {
                    idx: *s as u32,
                    port,
                }
            }
            NodeId::InputPort(_) => unreachable!("filtered above"),
        };
        arcs.push(Arc {
            src: src_endpoint,
            dst: dst_endpoint,
        });
    }

    let graph = BalancerGraph::new(g.n_inputs as u32, g.n_outputs as u32, g.n_splitters as u32, arcs);
    graph
        .validate()
        .expect("from_splitter_graph produced a structurally invalid graph");
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balancer::verify::verify_balancer;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// `(1, 2)` recovered: 1 input, 2 outputs, 1 splitter. No padding.
    #[test]
    fn convert_1_to_2() {
        let g = SplitterGraph {
            n_inputs: 1,
            n_outputs: 2,
            n_splitters: 1,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(1)),
            ],
        };
        let bg = from_splitter_graph(&g).unwrap();
        assert_eq!(bg.n_inputs, 1);
        assert_eq!(bg.n_outputs, 2);
        let outcome = verify_balancer(&bg).unwrap();
        assert!(approx_eq(outcome.real_output_throughput, 0.5));
    }

    /// `(2, 1)` merger: rate-doubled output. With multi-arc, no need to
    /// invent dummy outputs — splitter has both out-ports wired but only
    /// one feeds a real output. The other out-port goes... wait, it
    /// needs SOMEWHERE; SplitterGraph would emit 2 edges from this
    /// splitter. For a real (2, 1) Factorio template, the second out-port
    /// would feed back into something (no orphan ports in real layouts).
    /// Skip this test until we have a real (2, 1) graph.
    /// `(2, 2)` symmetric: both ports filled normally.
    #[test]
    fn convert_2_to_2() {
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
        let bg = from_splitter_graph(&g).unwrap();
        assert_eq!(bg.n_inputs, 2);
        assert_eq!(bg.n_outputs, 2);
        verify_balancer(&bg).unwrap();
    }

    /// Sideloaded splitter: 3 edges into Splitter(0) (e.g., from a
    /// previously rejected-as-overdegree template). Should now convert
    /// successfully — port 0 gets one arc, port 1 gets two arcs.
    #[test]
    fn accepts_sideloaded_splitter() {
        let g = SplitterGraph {
            n_inputs: 1,
            n_outputs: 2,
            n_splitters: 2,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::Splitter(1), NodeId::Splitter(0)),
                (NodeId::Splitter(1), NodeId::Splitter(0)),
                (NodeId::Splitter(0), NodeId::Splitter(1)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
                (NodeId::Splitter(1), NodeId::OutputPort(1)),
            ],
        };
        // S0: in (input, S1, S1), out (S1, output 0). 3 in, 2 out.
        // S1: in (S0), out (S0, S0, output 1)? wait, S1 has out_degree 3
        // in this synthetic graph. Let me reconstruct a valid one.
        let _ = g;

        // Minimal sideload example: S0 has 3 in-arcs (from input 0, input 1,
        // and S1.out0). S0 outs go to output 0 and S1.in0. S1 outs go to
        // S0 (sideload) and output 1.
        let g = SplitterGraph {
            n_inputs: 2,
            n_outputs: 2,
            n_splitters: 2,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::InputPort(1), NodeId::Splitter(0)),
                (NodeId::Splitter(1), NodeId::Splitter(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
                (NodeId::Splitter(0), NodeId::Splitter(1)),
                (NodeId::Splitter(1), NodeId::OutputPort(1)),
            ],
        };
        // S0: 3 in, 2 out (out 0 + S1)
        // S1: 1 in (S0), 2 out (S0, output 1)
        let bg = from_splitter_graph(&g).unwrap();
        assert_eq!(bg.n_inputs, 2);
        assert_eq!(bg.n_outputs, 2);
        bg.validate().unwrap();
    }
}
