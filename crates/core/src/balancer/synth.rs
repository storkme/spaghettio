//! Balancer graph synthesis from a target shape `(n, m)`.
//!
//! Implements Zhou-Chen-Bruck (2012) §VI.A: pad a uniform target
//! distribution `{1/m, ..., 1/m}` to power-of-2 denominator
//! `{1/2^n × m, (2^n - m)/2^n × 1}`, build the Knuth-Yao complete binary
//! tree, and connect the leftover output(s) back to the input. Combined
//! with our multi-arc-port model relaxation, *all* leftover leaves
//! sideload onto a single merger splitter's in-port — no `(k, 2)` merger
//! sub-network needed.
//!
//! Construction for `(1, m)` with `m ≥ 2`:
//!
//! 1. `pow2 = next_power_of_two(m)`, `leftover = pow2 - m`.
//! 2. Build the `(1, pow2)` complete binary tree (`pow2 - 1` splitters in
//!    BFS-heap layout). Leaves 0..m become real outputs; leaves m..pow2
//!    become feedback edges.
//! 3. If `leftover > 0`: prepend a merger splitter M. M.in0 = real input;
//!    M.in1 = all `leftover` feedback edges sideloaded onto a single
//!    in-port (port rate = sum, multi-arc relaxation). M's two outputs
//!    feed the tree's root.
//! 4. If `leftover == 0` (dyadic m): no merger; real input directly to the
//!    root's in-port 0. Root's in-port 1 is empty (rate 0 by relaxation).
//!
//! Each leaf in the (1, pow2) tree carries rate `(1 + total_feedback) /
//! pow2`. With `total_feedback = leftover × (1/m)`, that's `pow2 / pow2 / m
//! = 1/m`. Real outputs all see `1/m`. ✓
//!
//! `n > 1` is still unsupported. The natural extensions are: symmetric
//! `n = m` via Beneš (no feedback needed), or `n < m` via a merge-prefix
//! into a `(1, m)` synth tree (each input feeds one of the merger's
//! in-ports, possibly with fan-in if `n > 2`).

use thiserror::Error;

use crate::balancer::graph::{Arc, BalancerGraph, Sink, Source};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SynthError {
    #[error("invalid shape: ({n}, {m}) — both must be ≥ 1")]
    InvalidShape { n: u32, m: u32 },
    #[error("synthesis for ({n}, {m}) is not yet supported in v1 (only n == 1)")]
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
    Ok(synth_one_to_m(m))
}

fn synth_one_to_m(m: u32) -> BalancerGraph {
    debug_assert!(m >= 1);

    if m == 1 {
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

    let pow2 = m.next_power_of_two();
    let leftover = pow2 - m;
    let n_inner = pow2 - 1; // splitters in the (1, pow2) tree
    let need_merger = leftover > 0;
    let inner_offset = if need_merger { 1u32 } else { 0u32 };
    let total_splitters = n_inner + inner_offset;

    let mut arcs: Vec<Arc> = Vec::new();

    if need_merger {
        // Merger M (idx 0). in0 = real input; in1 = all feedback edges
        // (sideloaded — multi-arc in-port).
        arcs.push(Arc {
            src: Source::Input(0),
            dst: Sink::Splitter { idx: 0, port: 0 },
        });
        // M's two outputs feed the inner tree's root (idx 1).
        arcs.push(Arc {
            src: Source::Splitter { idx: 0, port: 0 },
            dst: Sink::Splitter { idx: 1, port: 0 },
        });
        arcs.push(Arc {
            src: Source::Splitter { idx: 0, port: 1 },
            dst: Sink::Splitter { idx: 1, port: 1 },
        });
    } else {
        // Dyadic case: real input directly to root's in-port 0. The
        // root's in-port 1 is empty (no arcs); rate 0 by relaxation.
        arcs.push(Arc {
            src: Source::Input(0),
            dst: Sink::Splitter { idx: 0, port: 0 },
        });
    }

    // Inner tree in BFS-heap layout. Old idx i has children at 2i+1, 2i+2.
    // Map old → new by adding `inner_offset`.
    for old_i in 0..n_inner {
        let new_i = old_i + inner_offset;
        for (port, child_old) in [(0u8, 2 * old_i + 1), (1u8, 2 * old_i + 2)] {
            if child_old < n_inner {
                let child_new = child_old + inner_offset;
                arcs.push(Arc {
                    src: Source::Splitter { idx: new_i, port },
                    dst: Sink::Splitter {
                        idx: child_new,
                        port: 0,
                    },
                });
            } else {
                let leaf_idx = child_old - n_inner;
                if leaf_idx < m {
                    arcs.push(Arc {
                        src: Source::Splitter { idx: new_i, port },
                        dst: Sink::Output(leaf_idx),
                    });
                } else {
                    debug_assert!(need_merger, "leftover only exists when merger is added");
                    arcs.push(Arc {
                        src: Source::Splitter { idx: new_i, port },
                        dst: Sink::Splitter { idx: 0, port: 1 },
                    });
                }
            }
        }
    }

    let g = BalancerGraph::new(1, m, total_splitters, arcs);
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

    fn check_uniform(m: u32) {
        let g = synth(1, m).unwrap();
        let out = verify_balancer(&g).unwrap();
        assert_eq!(out.output_throughputs.len(), m as usize);
        let target = 1.0 / m as f64;
        for (i, &t) in out.output_throughputs.iter().enumerate() {
            assert!(
                approx_eq(t, target),
                "synth(1, {}) output {} = {} != {}",
                m,
                i,
                t,
                target
            );
        }
    }

    #[test]
    fn synth_1_1_passthrough() {
        let g = synth(1, 1).unwrap();
        assert_eq!(g.n_splitters, 0);
        assert_eq!(g.n_inputs, 1);
        assert_eq!(g.n_outputs, 1);
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 1.0));
    }

    /// Dyadic: m a power of 2. m-1 splitters, no merger.
    #[test]
    fn synth_dyadic_powers_of_two() {
        for m in [2u32, 4, 8, 16] {
            let g = synth(1, m).unwrap();
            assert_eq!(g.n_splitters, m - 1, "synth(1, {}) splitter count", m);
            assert_eq!(g.n_inputs, 1);
            assert_eq!(g.n_outputs, m);
            check_uniform(m);
        }
    }

    /// Single leftover: m = 2^n - 1. Adds 1 merger splitter.
    #[test]
    fn synth_single_leftover() {
        for m in [3u32, 7, 15] {
            let g = synth(1, m).unwrap();
            // pow2 = m+1; n_inner = m; +1 merger.
            assert_eq!(g.n_splitters, m + 1, "synth(1, {}) splitter count", m);
            check_uniform(m);
        }
    }

    /// Multi-leftover: leftover > 1. Merger sideloads multiple feedback
    /// edges onto its in-port 1.
    #[test]
    fn synth_multi_leftover() {
        // (1, 5): pow2=8, leftover=3. 7 inner + 1 merger = 8 splitters.
        let g = synth(1, 5).unwrap();
        assert_eq!(g.n_splitters, 8);
        check_uniform(5);

        // (1, 6): pow2=8, leftover=2. 8 splitters.
        let g = synth(1, 6).unwrap();
        assert_eq!(g.n_splitters, 8);
        check_uniform(6);

        // (1, 9): pow2=16, leftover=7. 15 inner + 1 merger = 16. Unblocks
        // a shape from issue #136.
        let g = synth(1, 9).unwrap();
        assert_eq!(g.n_splitters, 16);
        check_uniform(9);

        // (1, 10): pow2=16, leftover=6. 16 splitters. Also from #136.
        check_uniform(10);
    }

    /// Stress: synth and verify every (1, m) for m in 1..=16.
    #[test]
    fn synth_every_m_up_to_16() {
        for m in 1u32..=16 {
            check_uniform(m);
        }
    }

    /// Sideloaded merger: confirm M's in-port 1 has `leftover` arcs.
    #[test]
    fn merger_in_port_1_has_leftover_arcs() {
        let g = synth(1, 5).unwrap();
        // M is splitter idx 0; count arcs with dst Splitter{idx:0, port:1}.
        let count = g
            .arcs
            .iter()
            .filter(|a| matches!(a.dst, Sink::Splitter { idx: 0, port: 1 }))
            .count();
        assert_eq!(count, 3, "(1, 5) leftover = 3, expected 3 sideloaded feedbacks");

        let g = synth(1, 9).unwrap();
        let count = g
            .arcs
            .iter()
            .filter(|a| matches!(a.dst, Sink::Splitter { idx: 0, port: 1 }))
            .count();
        assert_eq!(count, 7, "(1, 9) leftover = 7");
    }

    #[test]
    fn unsupported_n_gt_1() {
        assert!(matches!(
            synth(2, 4),
            Err(SynthError::Unsupported { n: 2, m: 4 })
        ));
        assert!(matches!(
            synth(3, 5),
            Err(SynthError::Unsupported { .. })
        ));
    }

    #[test]
    fn invalid_shapes_error() {
        assert!(matches!(synth(0, 4), Err(SynthError::InvalidShape { .. })));
        assert!(matches!(synth(1, 0), Err(SynthError::InvalidShape { .. })));
    }

    #[test]
    fn synth_serde_round_trip() {
        let g = synth(1, 5).unwrap();
        let json = serde_json::to_string(&g).unwrap();
        let g2: BalancerGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(g, g2);
    }
}
