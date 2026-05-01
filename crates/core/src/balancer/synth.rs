//! Balancer graph synthesis from a target shape `(n, m)`.
//!
//! Implements Zhou-Chen-Bruck (2012) §VI.B: build a Huffman binary tree
//! over the target distribution `{q_1, ..., q_m}`, replace each internal
//! node with a 2-output splitter implementing `{w_l/(w_l+w_r),
//! w_r/(w_l+w_r)}`, and treat each leaf as an output.
//!
//! For load balancers, every output gets the same rate so all leaves have
//! equal weight `1/m`. Two regimes:
//!
//! - **Dyadic** (`m` a power of 2): Huffman gives a complete binary tree
//!   of vanilla `{1/2, 1/2}` splitters. `m-1` splitters total. No feedback.
//! - **Non-dyadic**: some internal nodes need non-50/50 splitters
//!   (e.g. `{2/3, 1/3}`), which require recirculating "leftover" output
//!   back to the input per Zhou-Chen-Bruck §V / §VI.A. Not yet implemented.
//!
//! v1 scope: `n == 1` and `m` a power of 2. Other shapes return
//! [`SynthError::Unsupported`]. Extending to non-dyadic `m` and `n > 1`
//! is the obvious follow-up — synth-graph quality is the load-bearing
//! lever for placement (per the plan's "future levers" notes).

use thiserror::Error;

use crate::balancer::graph::{Arc, BalancerGraph, Sink, Source};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SynthError {
    #[error("invalid shape: ({n}, {m}) — both must be ≥ 1")]
    InvalidShape { n: u32, m: u32 },
    #[error(
        "synthesis for ({n}, {m}) is not yet supported in v1 \
         (only n == 1 and m a power of 2)"
    )]
    Unsupported { n: u32, m: u32 },
}

/// Synthesize a load-balancer for `n` inputs and `m` outputs.
///
/// Returns a [`BalancerGraph`] that, when verified at unit input capacity,
/// produces uniform output throughput `n/m`. Splitter ports are assigned
/// canonically; the topology is deterministic for a given `(n, m)`.
pub fn synth(n: u32, m: u32) -> Result<BalancerGraph, SynthError> {
    if n == 0 || m == 0 {
        return Err(SynthError::InvalidShape { n, m });
    }
    if n != 1 || !m.is_power_of_two() {
        return Err(SynthError::Unsupported { n, m });
    }
    Ok(synth_one_to_pow2(m))
}

/// Build the (1, m) balancer for `m` a power of 2 as a complete binary
/// tree of 50/50 splitters in BFS-heap layout. Every splitter port that
/// isn't wired to the real input or to a child receives a dummy input at
/// capacity 0.
fn synth_one_to_pow2(m: u32) -> BalancerGraph {
    debug_assert!(m >= 1 && m.is_power_of_two());

    if m == 1 {
        // Trivial: direct passthrough. No splitters.
        return BalancerGraph::new(
            1,
            1,
            0,
            vec![Arc {
                src: Source::Input(0),
                dst: Sink::Output(0),
            }],
        );
    }

    let n_splitters = m - 1;

    // Splitters in BFS-heap layout: splitter `i` has children at `2i+1`
    // (left) and `2i+2` (right). When a child index is ≥ n_splitters, it
    // is a leaf and maps to output `child_index - n_splitters`.
    let mut arcs: Vec<Arc> = Vec::with_capacity((m - 1 + 1 + m) as usize);

    // Root (splitter 0) receives the real input on in-port 0.
    arcs.push(Arc {
        src: Source::Input(0),
        dst: Sink::Splitter { idx: 0, port: 0 },
    });

    // For each splitter, emit edges from its two out-ports to its children.
    for i in 0..n_splitters {
        for (port, child) in [(0u8, 2 * i + 1), (1u8, 2 * i + 2)] {
            if child < n_splitters {
                arcs.push(Arc {
                    src: Source::Splitter { idx: i, port },
                    dst: Sink::Splitter {
                        idx: child,
                        port: 0,
                    },
                });
            } else {
                arcs.push(Arc {
                    src: Source::Splitter { idx: i, port },
                    dst: Sink::Output(child - n_splitters),
                });
            }
        }
    }

    // Every splitter's in-port 1 needs a dummy input (cap 0). Internal
    // splitters' in-port 0 is already wired to a parent's out-port; the
    // root's in-port 0 carries the real input.
    let total_inputs = 1 + n_splitters; // 1 real + (m-1) dummies = m
    let total_outputs = m;
    debug_assert_eq!(total_inputs, total_outputs);

    for s in 0..n_splitters {
        arcs.push(Arc {
            src: Source::Input(1 + s),
            dst: Sink::Splitter { idx: s, port: 1 },
        });
    }

    let mut input_caps = vec![1.0f64];
    input_caps.resize(total_inputs as usize, 0.0);
    let output_caps = vec![1.0f64; total_outputs as usize];

    let g = BalancerGraph {
        n_inputs: total_inputs,
        n_outputs: total_outputs,
        n_splitters,
        arcs,
        input_caps,
        output_caps,
    };
    debug_assert!(g.validate().is_ok());
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balancer::verify::verify_balancer;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// `synth(1, 1)` is the trivial passthrough — no splitters, one arc.
    #[test]
    fn synth_1_1_passthrough() {
        let g = synth(1, 1).unwrap();
        assert_eq!(g.n_inputs, 1);
        assert_eq!(g.n_outputs, 1);
        assert_eq!(g.n_splitters, 0);
        assert_eq!(g.arcs.len(), 1);
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 1.0));
    }

    /// `synth(1, 2)`: single 50/50 splitter. Each output 0.5.
    #[test]
    fn synth_1_2_single_splitter() {
        let g = synth(1, 2).unwrap();
        assert_eq!(g.n_splitters, 1);
        assert_eq!(g.n_inputs, 2); // 1 real + 1 dummy
        assert_eq!(g.n_outputs, 2);
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 0.5));
        assert!(approx_eq(out.max_imbalance, 0.0));
    }

    /// `synth(1, 4)`: 3-splitter complete binary tree. Each output 0.25.
    #[test]
    fn synth_1_4_complete_tree() {
        let g = synth(1, 4).unwrap();
        assert_eq!(g.n_splitters, 3);
        assert_eq!(g.n_inputs, 4); // 1 real + 3 dummies
        assert_eq!(g.n_outputs, 4);
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 0.25));
        for &t in &out.output_throughputs {
            assert!(approx_eq(t, 0.25));
        }
    }

    /// `synth(1, 8)`: 7-splitter complete binary tree. Each output 0.125.
    #[test]
    fn synth_1_8_three_levels() {
        let g = synth(1, 8).unwrap();
        assert_eq!(g.n_splitters, 7);
        assert_eq!(g.n_inputs, 8);
        assert_eq!(g.n_outputs, 8);
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 0.125));
    }

    /// `synth(1, 16)`: confirms the recurrence holds at depth 4.
    #[test]
    fn synth_1_16() {
        let g = synth(1, 16).unwrap();
        assert_eq!(g.n_splitters, 15);
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 1.0 / 16.0));
    }

    /// Round-trip the synthesized graph through serde to confirm it's
    /// stable for use as a build-time artifact (e.g., balancer_library).
    #[test]
    fn synth_serde_round_trip() {
        let g = synth(1, 4).unwrap();
        let json = serde_json::to_string(&g).unwrap();
        let g2: BalancerGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(g, g2);
    }

    #[test]
    fn unsupported_shapes_error() {
        // n != 1
        assert!(matches!(
            synth(2, 4),
            Err(SynthError::Unsupported { n: 2, m: 4 })
        ));
        // m not a power of 2 — non-dyadic, needs §VI.A feedback.
        assert!(matches!(
            synth(1, 3),
            Err(SynthError::Unsupported { n: 1, m: 3 })
        ));
        assert!(matches!(
            synth(1, 5),
            Err(SynthError::Unsupported { n: 1, m: 5 })
        ));
        assert!(matches!(
            synth(1, 9),
            Err(SynthError::Unsupported { n: 1, m: 9 })
        ));
    }

    #[test]
    fn invalid_shapes_error() {
        assert!(matches!(synth(0, 4), Err(SynthError::InvalidShape { .. })));
        assert!(matches!(synth(1, 0), Err(SynthError::InvalidShape { .. })));
    }
}
