//! Balancer graph synthesis from a target shape `(n, m)`.
//!
//! Implements Zhou-Chen-Bruck (2012) §VI.A and §VI.B: build a Huffman /
//! Knuth-Yao binary tree over the target distribution `{q_1, ..., q_m}`,
//! replace each internal node with a 2-output splitter implementing
//! `{w_l/(w_l+w_r), w_r/(w_l+w_r)}`, and treat each leaf as an output.
//! Non-dyadic distributions are realized via the §VI.A feedback
//! construction: pad to a power-of-2 denominator, route the leftover
//! atoms back to the network's starting point.
//!
//! For load balancers all `q_i = 1/m`. v1 supports two regimes:
//!
//! - **Dyadic** (`m` a power of 2): complete binary tree of vanilla
//!   `{1/2, 1/2}` splitters, `m-1` splitters total. No feedback.
//! - **Single-leftover** (`m+1` a power of 2, i.e. `m ∈ {3, 7, 15, ...}`):
//!   build the `(1, 2^n)` tree, redirect the last leaf as a feedback edge,
//!   and add one merger splitter combining `real_input + feedback` whose
//!   outputs feed the root's two in-ports. `m` splitters total.
//!
//! Larger leftovers (e.g. `m=5` with 3 feedbacks, `m=9` with 7) require a
//! `(k, 2)` merger sub-network that preserves flow conservation under our
//! all-fluid model — itself a non-trivial non-dyadic balancer. Deferred
//! until the placement engine can tell us whether canonical-graph quality
//! is the binding constraint (per the plan's "candidate-graph generation"
//! v2 lever).
//!
//! `n > 1` is also deferred: the symmetric `n = m` case wants Beneš
//! (out of scope), and `n < m` wants a merge-prefix into the (1, m)
//! synth tree.

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
    if n != 1 {
        return Err(SynthError::Unsupported { n, m });
    }
    if m.is_power_of_two() {
        return Ok(synth_one_to_pow2(m));
    }
    // m+1 a power of 2 → single feedback leaf, single merger splitter.
    if (m + 1).is_power_of_two() {
        return Ok(synth_one_to_pow2_minus_one(m));
    }
    Err(SynthError::Unsupported { n, m })
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

/// Build the `(1, m)` balancer for `m = 2^n - 1` (i.e. `m+1` is a power of
/// 2 and the §VI.A leftover is exactly 1).
///
/// Construction: take the `(1, 2^n)` complete-binary-tree, reroute the last
/// leaf (output index `m`) to be a feedback edge instead of a real output,
/// and prepend a merger splitter `M` whose two outputs feed the root's two
/// in-ports.
///
/// New splitter numbering: `M` is index 0; the original tree's splitters
/// shift to indices `1..=m-1` (so the original root becomes splitter 1).
/// Each splitter has `m` total = `(m-1) + 1` (tree + merger).
fn synth_one_to_pow2_minus_one(m: u32) -> BalancerGraph {
    debug_assert!(m >= 1 && (m + 1).is_power_of_two());
    let pow2 = m + 1;
    let inner_splitters = pow2 - 1; // splitters in the original (1, 2^n) tree
    let total_splitters = inner_splitters + 1; // +1 for the merger M

    let mut arcs: Vec<Arc> = Vec::new();

    // Merger M (splitter 0). M.in0 ← real_input; M.in1 ← feedback (added below).
    arcs.push(Arc {
        src: Source::Input(0),
        dst: Sink::Splitter { idx: 0, port: 0 },
    });
    // M's two outputs feed the root (which is splitter 1 in the new numbering).
    arcs.push(Arc {
        src: Source::Splitter { idx: 0, port: 0 },
        dst: Sink::Splitter { idx: 1, port: 0 },
    });
    arcs.push(Arc {
        src: Source::Splitter { idx: 0, port: 1 },
        dst: Sink::Splitter { idx: 1, port: 1 },
    });

    // Inner tree edges, with old splitter index `i` shifted to `i + 1`.
    // BFS-heap layout of the (1, 2^n) tree: parent `i` has children at
    // `2i+1` and `2i+2`; child idx >= inner_splitters means it's a leaf
    // mapping to output `child_idx - inner_splitters` in the dyadic case.
    // Here, leaf index `m` (the final leaf in BFS order) becomes the
    // single feedback edge instead of a real output.
    for old_i in 0..inner_splitters {
        let new_i = old_i + 1;
        for (port, child_old) in [(0u8, 2 * old_i + 1), (1u8, 2 * old_i + 2)] {
            if child_old < inner_splitters {
                let child_new = child_old + 1;
                arcs.push(Arc {
                    src: Source::Splitter { idx: new_i, port },
                    dst: Sink::Splitter {
                        idx: child_new,
                        port: 0,
                    },
                });
            } else {
                let leaf_idx = child_old - inner_splitters;
                if leaf_idx < m {
                    arcs.push(Arc {
                        src: Source::Splitter { idx: new_i, port },
                        dst: Sink::Output(leaf_idx),
                    });
                } else {
                    debug_assert_eq!(leaf_idx, m, "exactly one leftover leaf");
                    arcs.push(Arc {
                        src: Source::Splitter { idx: new_i, port },
                        dst: Sink::Splitter { idx: 0, port: 1 },
                    });
                }
            }
        }
    }

    // Inner non-root splitters (new idx 2..total_splitters) need a dummy on
    // their in-port 1. The root (new idx 1) has both in-ports wired by M.
    let mut next_dummy_input = 1u32;
    for new_i in 2..total_splitters {
        arcs.push(Arc {
            src: Source::Input(next_dummy_input),
            dst: Sink::Splitter { idx: new_i, port: 1 },
        });
        next_dummy_input += 1;
    }
    let total_inputs = next_dummy_input;
    let total_outputs = m;
    debug_assert_eq!(total_inputs, total_outputs);

    let mut input_caps = vec![1.0f64];
    input_caps.resize(total_inputs as usize, 0.0);
    let output_caps = vec![1.0f64; total_outputs as usize];

    let g = BalancerGraph {
        n_inputs: total_inputs,
        n_outputs: total_outputs,
        n_splitters: total_splitters,
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

    /// `synth(1, 3)`: smallest non-dyadic case. Single feedback leaf,
    /// single merger splitter. 4 splitters total.
    #[test]
    fn synth_1_3_single_feedback() {
        let g = synth(1, 3).unwrap();
        assert_eq!(g.n_splitters, 4); // 3 tree + 1 merger
        assert_eq!(g.n_inputs, 3); // 1 real + 2 dummies (for the 2 non-root tree splitters)
        assert_eq!(g.n_outputs, 3);
        let out = verify_balancer(&g).unwrap();
        for &t in &out.output_throughputs {
            assert!(approx_eq(t, 1.0 / 3.0), "output {} != 1/3", t);
        }
    }

    /// `synth(1, 7)`: leftover-1 case at depth 3. 7 + 1 = 8 splitters.
    #[test]
    fn synth_1_7_leftover_one() {
        let g = synth(1, 7).unwrap();
        assert_eq!(g.n_splitters, 8); // 7 tree + 1 merger
        assert_eq!(g.n_inputs, 7);
        assert_eq!(g.n_outputs, 7);
        let out = verify_balancer(&g).unwrap();
        for &t in &out.output_throughputs {
            assert!(approx_eq(t, 1.0 / 7.0));
        }
    }

    /// `synth(1, 15)`: leftover-1 at depth 4. Stress-test the construction
    /// at a larger size.
    #[test]
    fn synth_1_15_leftover_one() {
        let g = synth(1, 15).unwrap();
        assert_eq!(g.n_splitters, 16); // 15 tree + 1 merger
        let out = verify_balancer(&g).unwrap();
        for &t in &out.output_throughputs {
            assert!(approx_eq(t, 1.0 / 15.0));
        }
    }

    #[test]
    fn unsupported_shapes_error() {
        // n != 1
        assert!(matches!(
            synth(2, 4),
            Err(SynthError::Unsupported { n: 2, m: 4 })
        ));
        // m where neither m nor m+1 is a power of 2 — needs (k, 2) merger.
        assert!(matches!(
            synth(1, 5),
            Err(SynthError::Unsupported { n: 1, m: 5 })
        ));
        assert!(matches!(
            synth(1, 6),
            Err(SynthError::Unsupported { n: 1, m: 6 })
        ));
        assert!(matches!(
            synth(1, 9),
            Err(SynthError::Unsupported { n: 1, m: 9 })
        ));
        assert!(matches!(
            synth(1, 10),
            Err(SynthError::Unsupported { n: 1, m: 10 })
        ));
    }

    #[test]
    fn invalid_shapes_error() {
        assert!(matches!(synth(0, 4), Err(SynthError::InvalidShape { .. })));
        assert!(matches!(synth(1, 0), Err(SynthError::InvalidShape { .. })));
    }
}
