//! Bridge between [`SplitterGraph`] (recovered from a placed
//! `BalancerTemplate` via `bus::balancer_classify::recover_graph`) and
//! [`BalancerGraph`] (the canonical type our verifier consumes).
//!
//! The two representations differ:
//!   - `SplitterGraph` is **node-coarse**: a splitter is a single node and
//!     edges have no port concept; it tolerates splitters with arbitrary
//!     in/out degrees ≤ 2.
//!   - `BalancerGraph` is **port-fine**: each splitter has explicit port
//!     0/1 on each side, and validation requires every port to have exactly
//!     one arc.
//!
//! [`from_splitter_graph`] pads asymmetric `(n, m)` shapes with dummy I/O
//! nodes at capacity 0 (per Couëtoux et al. §1.1, end of subsection — adding
//! a fluid arc from a c=0 dummy is observationally equivalent to leaving a
//! splitter port unwired). Port assignment is arbitrary but stable: edges
//! are visited in graph order, port 0 is filled before port 1.

use thiserror::Error;

use crate::balancer::graph::{Arc, BalancerGraph, Sink, Source};
use crate::bus::balancer_classify::{NodeId, SplitterGraph};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BakeError {
    #[error("splitter {idx} has {count} incoming edges (expected ≤ 2)")]
    SplitterInDegree { idx: usize, count: usize },
    #[error("splitter {idx} has {count} outgoing edges (expected ≤ 2)")]
    SplitterOutDegree { idx: usize, count: usize },
    #[error("input port {idx} has {count} outgoing edges (expected exactly 1)")]
    InputDegree { idx: usize, count: usize },
    #[error("output port {idx} has {count} incoming edges (expected exactly 1)")]
    OutputDegree { idx: usize, count: usize },
    #[error("edge with OutputPort source or InputPort sink (graph malformed)")]
    Malformed,
}

/// Convert a [`SplitterGraph`] into a [`BalancerGraph`], padding asymmetric
/// shapes with dummy I/O nodes at capacity 0 to satisfy the splitter
/// 2-in / 2-out invariant.
#[allow(clippy::needless_range_loop)]
pub fn from_splitter_graph(g: &SplitterGraph) -> Result<BalancerGraph, BakeError> {
    // Tally degrees per splitter and per real input/output port.
    let mut splitter_in = vec![0usize; g.n_splitters];
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
            NodeId::Splitter(s) => splitter_in[*s] += 1,
            NodeId::InputPort(_) => return Err(BakeError::Malformed),
        }
    }
    for s in 0..g.n_splitters {
        if splitter_in[s] > 2 {
            return Err(BakeError::SplitterInDegree {
                idx: s,
                count: splitter_in[s],
            });
        }
        if splitter_out[s] > 2 {
            return Err(BakeError::SplitterOutDegree {
                idx: s,
                count: splitter_out[s],
            });
        }
    }
    for i in 0..g.n_inputs {
        if input_out[i] != 1 {
            return Err(BakeError::InputDegree {
                idx: i,
                count: input_out[i],
            });
        }
    }
    for o in 0..g.n_outputs {
        if output_in[o] != 1 {
            return Err(BakeError::OutputDegree {
                idx: o,
                count: output_in[o],
            });
        }
    }

    // Total dummy padding needed: one dummy input per missing splitter
    // in-port, one dummy output per missing splitter out-port. Arc-counting
    // (sum over splitters of (out_degree - in_degree) = n_outputs - n_inputs)
    // guarantees `total_inputs == total_outputs` after padding.
    let missing_in: u32 = (0..g.n_splitters)
        .map(|s| (2 - splitter_in[s]) as u32)
        .sum();
    let missing_out: u32 = (0..g.n_splitters)
        .map(|s| (2 - splitter_out[s]) as u32)
        .sum();
    let total_inputs = g.n_inputs as u32 + missing_in;
    let total_outputs = g.n_outputs as u32 + missing_out;
    debug_assert_eq!(total_inputs, total_outputs);

    // Walk edges in order, assigning splitter ports incrementally.
    let mut next_in_port = vec![0u8; g.n_splitters];
    let mut next_out_port = vec![0u8; g.n_splitters];
    let mut arcs = Vec::with_capacity(g.edges.len() + (missing_in + missing_out) as usize);
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
                let port = next_in_port[*s];
                next_in_port[*s] += 1;
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

    // Pad missing splitter ports with dummies (cap = 0).
    let mut next_dummy_input = g.n_inputs as u32;
    let mut next_dummy_output = g.n_outputs as u32;
    for s in 0..g.n_splitters {
        while next_in_port[s] < 2 {
            let port = next_in_port[s];
            arcs.push(Arc {
                src: Source::Input(next_dummy_input),
                dst: Sink::Splitter {
                    idx: s as u32,
                    port,
                },
            });
            next_dummy_input += 1;
            next_in_port[s] += 1;
        }
        while next_out_port[s] < 2 {
            let port = next_out_port[s];
            arcs.push(Arc {
                src: Source::Splitter {
                    idx: s as u32,
                    port,
                },
                dst: Sink::Output(next_dummy_output),
            });
            next_dummy_output += 1;
            next_out_port[s] += 1;
        }
    }
    debug_assert_eq!(next_dummy_input, total_inputs);
    debug_assert_eq!(next_dummy_output, total_outputs);

    let mut input_caps = vec![1.0_f64; g.n_inputs];
    input_caps.resize(total_inputs as usize, 0.0);
    let mut output_caps = vec![1.0_f64; g.n_outputs];
    output_caps.resize(total_outputs as usize, 0.0);

    let graph = BalancerGraph {
        n_inputs: total_inputs,
        n_outputs: total_outputs,
        n_splitters: g.n_splitters as u32,
        arcs,
        input_caps,
        output_caps,
    };
    // Defense in depth: surface any structural issue before the verifier
    // sees the graph.
    graph
        .validate()
        .expect("from_splitter_graph produced a structurally invalid graph");
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balancer::verify::verify_balancer;

    /// Hand-build a (1, 2) `SplitterGraph` matching what `recover_graph`
    /// would emit for a single-splitter Factorio template, convert, verify.
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
        // 1 real + 1 dummy input, 2 real + 0 dummy output.
        assert_eq!(bg.n_inputs, 2);
        assert_eq!(bg.n_outputs, 2);
        assert_eq!(bg.input_caps, vec![1.0, 0.0]);
        assert_eq!(bg.output_caps, vec![1.0, 1.0]);
        let outcome = verify_balancer(&bg).unwrap();
        assert!((outcome.real_output_throughput - 0.5).abs() < 1e-9);
    }

    /// (2, 1) merger: dummy output at cap 0 absorbs the second splitter
    /// out-port. Single real output sees rate 1.0 in the all-fluid model.
    #[test]
    fn convert_2_to_1() {
        let g = SplitterGraph {
            n_inputs: 2,
            n_outputs: 1,
            n_splitters: 1,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::InputPort(1), NodeId::Splitter(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
            ],
        };
        let bg = from_splitter_graph(&g).unwrap();
        assert_eq!(bg.n_inputs, 2);
        assert_eq!(bg.n_outputs, 2);
        assert_eq!(bg.output_caps, vec![1.0, 0.0]);
        let outcome = verify_balancer(&bg).unwrap();
        assert!((outcome.real_output_throughput - 1.0).abs() < 1e-9);
    }

    /// Symmetric (2, 2): no dummies needed.
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
        assert!(bg.input_caps.iter().all(|c| *c == 1.0));
        assert!(bg.output_caps.iter().all(|c| *c == 1.0));
        verify_balancer(&bg).unwrap();
    }

    /// Splitter with degree > 2 is rejected (shouldn't happen for real
    /// Factorio templates but guards against malformed inputs).
    #[test]
    fn rejects_overdegree_splitter() {
        let g = SplitterGraph {
            n_inputs: 3,
            n_outputs: 2,
            n_splitters: 1,
            edges: vec![
                (NodeId::InputPort(0), NodeId::Splitter(0)),
                (NodeId::InputPort(1), NodeId::Splitter(0)),
                (NodeId::InputPort(2), NodeId::Splitter(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(0)),
                (NodeId::Splitter(0), NodeId::OutputPort(1)),
            ],
        };
        assert!(matches!(
            from_splitter_graph(&g),
            Err(BakeError::SplitterInDegree { idx: 0, count: 3 })
        ));
    }
}
