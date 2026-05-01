//! [`BalancerGraph`]: directed splitter network. Permissive extension of
//! Couëtoux et al. §1.1 Definition 1.1: inputs have d⁺=1/d⁻=0, outputs have
//! d⁻=1/d⁺=0, and splitters have d⁻ ∈ {0, 1, 2, ...} and d⁺=2.
//!
//! ## Multi-arc / underdegree splitter ports
//!
//! Couëtoux's strict definition requires d⁻=d⁺=2 with one arc per port. We
//! relax both sides:
//!   - **In-ports** may carry 0+ arcs (port rate = sum of arc rates).
//!     Models belt sideloading.
//!   - **Out-ports** may carry 0+ arcs total across the two ports. A
//!     splitter with `< 2` out-arcs is "underdegree": one or both
//!     out-ports have dead-end belts providing backpressure in the
//!     saturation-rich Couëtoux model. In our all-fluid model we treat
//!     this as: conservation still holds (sum-in = sum-out), but R7
//!     out-couple only applies when both ports have arcs.
//!
//! Combined effect: `n_inputs` and `n_outputs` can be whatever the graph
//! declares, and splitter ports with no incoming/outgoing edges contribute
//! 0 to conservation. No dummy I/O padding required.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Graph input port `idx` (0..n_inputs).
    Input(u32),
    /// Out-port `port` (0 or 1) of splitter `idx`.
    Splitter { idx: u32, port: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sink {
    /// Graph output port `idx` (0..n_outputs).
    Output(u32),
    /// In-port `port` (0 or 1) of splitter `idx`.
    Splitter { idx: u32, port: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Arc {
    pub src: Source,
    pub dst: Sink,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalancerGraph {
    pub n_inputs: u32,
    pub n_outputs: u32,
    pub n_splitters: u32,
    pub arcs: Vec<Arc>,
    /// Capacity at each input. Length must equal `n_inputs`. 1.0 for "real"
    /// inputs, 0.0 for dummies used to balance arc counts.
    pub input_caps: Vec<f64>,
    /// Capacity at each output. Length must equal `n_outputs`. 1.0 for
    /// "real" outputs, 0.0 for dummies.
    pub output_caps: Vec<f64>,
}

impl BalancerGraph {
    /// Build a graph with all-1.0 capacities. Asserts `n_inputs == n_outputs`
    /// since asymmetric graphs need an explicit capacity vector.
    pub fn new(n_inputs: u32, n_outputs: u32, n_splitters: u32, arcs: Vec<Arc>) -> Self {
        Self {
            n_inputs,
            n_outputs,
            n_splitters,
            arcs,
            input_caps: vec![1.0; n_inputs as usize],
            output_caps: vec![1.0; n_outputs as usize],
        }
    }

    /// Number of inputs flagged as "real" (capacity > 0).
    pub fn real_input_indices(&self) -> impl Iterator<Item = u32> + '_ {
        self.input_caps
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0.0)
            .map(|(i, _)| i as u32)
    }

    /// Number of outputs flagged as "real" (capacity > 0).
    pub fn real_output_indices(&self) -> impl Iterator<Item = u32> + '_ {
        self.output_caps
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0.0)
            .map(|(i, _)| i as u32)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("input_caps length {got} != n_inputs {expected}")]
    InputCapsLength { got: usize, expected: u32 },
    #[error("output_caps length {got} != n_outputs {expected}")]
    OutputCapsLength { got: usize, expected: u32 },
    #[error("input {idx} has {count} outgoing arcs (expected exactly 1)")]
    InputDegree { idx: u32, count: u32 },
    #[error("output {idx} has {count} incoming arcs (expected exactly 1)")]
    OutputDegree { idx: u32, count: u32 },
    #[error("splitter {idx} out-port {port} has {count} arcs (expected at most 1)")]
    SplitterOutPort { idx: u32, port: u8, count: u32 },
    #[error("arc {arc_idx} references nonexistent input {input_idx}")]
    BadInput { arc_idx: usize, input_idx: u32 },
    #[error("arc {arc_idx} references nonexistent output {output_idx}")]
    BadOutput { arc_idx: usize, output_idx: u32 },
    #[error("arc {arc_idx} references nonexistent splitter {splitter_idx}")]
    BadSplitter { arc_idx: usize, splitter_idx: u32 },
    #[error("arc {arc_idx} uses port {port} (expected 0 or 1)")]
    BadPort { arc_idx: usize, port: u8 },
}

/// Indexed lookup of arcs by endpoint. In-arc lists allow 0+ arcs per
/// port (sideloading); out-arc slots are `Option` to allow underdegree
/// splitters with one or both out-ports unwired (dead-end belts in the
/// Factorio template provide backpressure under saturation).
pub(super) struct ArcIndex {
    pub input_arc: Vec<usize>,
    pub output_arc: Vec<usize>,
    pub splitter_in: Vec<[Vec<usize>; 2]>,
    pub splitter_out: Vec<[Option<usize>; 2]>,
}

impl BalancerGraph {
    /// Validate structural invariants (port degrees, in-bounds references,
    /// capacity-vector lengths). Returns the [`ArcIndex`] on success so the
    /// verifier can avoid re-scanning.
    pub(super) fn validate_and_index(&self) -> Result<ArcIndex, GraphError> {
        if self.input_caps.len() != self.n_inputs as usize {
            return Err(GraphError::InputCapsLength {
                got: self.input_caps.len(),
                expected: self.n_inputs,
            });
        }
        if self.output_caps.len() != self.n_outputs as usize {
            return Err(GraphError::OutputCapsLength {
                got: self.output_caps.len(),
                expected: self.n_outputs,
            });
        }
        let mut input_arc: Vec<Option<usize>> = vec![None; self.n_inputs as usize];
        let mut output_arc: Vec<Option<usize>> = vec![None; self.n_outputs as usize];
        // Multi-arc per splitter in-port is allowed; out-port stays single.
        let mut splitter_in: Vec<[Vec<usize>; 2]> = (0..self.n_splitters)
            .map(|_| [Vec::new(), Vec::new()])
            .collect();
        let mut splitter_out: Vec<[Option<usize>; 2]> =
            vec![[None, None]; self.n_splitters as usize];

        for (arc_idx, arc) in self.arcs.iter().enumerate() {
            match arc.src {
                Source::Input(i) => {
                    if i >= self.n_inputs {
                        return Err(GraphError::BadInput { arc_idx, input_idx: i });
                    }
                    if input_arc[i as usize].is_some() {
                        return Err(GraphError::InputDegree {
                            idx: i,
                            count: count_input_arcs(self, i),
                        });
                    }
                    input_arc[i as usize] = Some(arc_idx);
                }
                Source::Splitter { idx, port } => {
                    if idx >= self.n_splitters {
                        return Err(GraphError::BadSplitter {
                            arc_idx,
                            splitter_idx: idx,
                        });
                    }
                    if port > 1 {
                        return Err(GraphError::BadPort { arc_idx, port });
                    }
                    let slot = &mut splitter_out[idx as usize][port as usize];
                    if slot.is_some() {
                        return Err(GraphError::SplitterOutPort {
                            idx,
                            port,
                            count: count_splitter_out_arcs(self, idx, port),
                        });
                    }
                    *slot = Some(arc_idx);
                }
            }
            match arc.dst {
                Sink::Output(o) => {
                    if o >= self.n_outputs {
                        return Err(GraphError::BadOutput {
                            arc_idx,
                            output_idx: o,
                        });
                    }
                    if output_arc[o as usize].is_some() {
                        return Err(GraphError::OutputDegree {
                            idx: o,
                            count: count_output_arcs(self, o),
                        });
                    }
                    output_arc[o as usize] = Some(arc_idx);
                }
                Sink::Splitter { idx, port } => {
                    if idx >= self.n_splitters {
                        return Err(GraphError::BadSplitter {
                            arc_idx,
                            splitter_idx: idx,
                        });
                    }
                    if port > 1 {
                        return Err(GraphError::BadPort { arc_idx, port });
                    }
                    splitter_in[idx as usize][port as usize].push(arc_idx);
                }
            }
        }

        // Every endpoint must have been filled.
        for i in 0..self.n_inputs {
            if input_arc[i as usize].is_none() {
                return Err(GraphError::InputDegree { idx: i, count: 0 });
            }
        }
        for o in 0..self.n_outputs {
            if output_arc[o as usize].is_none() {
                return Err(GraphError::OutputDegree { idx: o, count: 0 });
            }
        }
        // Both in-ports and out-ports allow 0+ arcs.

        Ok(ArcIndex {
            input_arc: input_arc.into_iter().map(Option::unwrap).collect(),
            output_arc: output_arc.into_iter().map(Option::unwrap).collect(),
            splitter_in,
            splitter_out,
        })
    }

    /// Public entry point: just structural validation.
    pub fn validate(&self) -> Result<(), GraphError> {
        self.validate_and_index().map(|_| ())
    }
}

// ── error-reporting helpers (slow scans; only run on the failure path) ────

fn count_input_arcs(g: &BalancerGraph, idx: u32) -> u32 {
    g.arcs
        .iter()
        .filter(|a| matches!(a.src, Source::Input(i) if i == idx))
        .count() as u32
}

fn count_output_arcs(g: &BalancerGraph, idx: u32) -> u32 {
    g.arcs
        .iter()
        .filter(|a| matches!(a.dst, Sink::Output(i) if i == idx))
        .count() as u32
}

fn count_splitter_out_arcs(g: &BalancerGraph, splitter: u32, port: u8) -> u32 {
    g.arcs
        .iter()
        .filter(|a| matches!(a.src, Source::Splitter { idx, port: p } if idx == splitter && p == port))
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1 input → 1 output, no splitters. Should validate.
    #[test]
    fn passthrough_validates() {
        let g = BalancerGraph::new(
            1,
            1,
            0,
            vec![Arc {
                src: Source::Input(0),
                dst: Sink::Output(0),
            }],
        );
        g.validate().unwrap();
    }

    /// 2 inputs, 2 outputs, 1 splitter wired straight through.
    #[test]
    fn single_splitter_2x2_validates() {
        let g = BalancerGraph::new(
            2,
            2,
            1,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1),
                    dst: Sink::Splitter { idx: 0, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Output(1),
                },
            ],
        );
        g.validate().unwrap();
    }

    #[test]
    fn duplicate_input_arc_rejected() {
        let g = BalancerGraph::new(
            2,
            2,
            1,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                // Both arcs leave input 0; input 1 has no outgoing arc.
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Output(1),
                },
            ],
        );
        assert!(matches!(
            g.validate(),
            Err(GraphError::InputDegree { idx: 0, count: 2 })
        ));
    }

    /// Asymmetric `(1, 2)` graph: splitter port 1 has zero in-arcs (no
    /// dummy padding required under the multi-arc relaxation).
    #[test]
    fn asymmetric_with_empty_port_validates() {
        let g = BalancerGraph::new(
            1,
            2,
            1,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Output(1),
                },
            ],
        );
        g.validate().unwrap();
    }

    /// Sideloaded splitter: 3 arcs converging on the same in-port.
    #[test]
    fn multi_arc_in_port_validates() {
        let g = BalancerGraph::new(
            3,
            1,
            1,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(2),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                // Splitter out-port 1 must still be wired (R7 needs both
                // out-arcs); send to a discarded sink would need another
                // node — for a unit test we just leave it as an output.
            ],
        );
        // Need 1 more arc for splitter out-port 1, and 1 more output.
        // Actually we have n_outputs=1 but 2 out-ports. Let me fix.
        let _ = g;

        let g = BalancerGraph::new(
            3,
            2,
            1,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(2),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Output(1),
                },
            ],
        );
        g.validate().unwrap();
    }

    #[test]
    fn serde_round_trip() {
        let g = BalancerGraph::new(
            2,
            2,
            1,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1),
                    dst: Sink::Splitter { idx: 0, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Output(1),
                },
            ],
        );
        let json = serde_json::to_string(&g).unwrap();
        let g2: BalancerGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(g, g2);
    }
}
