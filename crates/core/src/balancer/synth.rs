//! Balancer graph synthesis from a target shape `(n, m)`.
//!
//! Implements Zhou-Chen-Bruck (2012) §VI.A: pad a uniform target
//! distribution to power-of-2 denominator, build the Knuth-Yao complete
//! binary tree, connect leftover output(s) back to the input. Combined
//! with our multi-arc-port relaxation, all leftover leaves sideload onto
//! a single merger splitter's in-port — no `(k, 2)` merger sub-network
//! needed.
//!
//! Asymmetric `(n, m)` for `n > 1` is supported by sideloading all `n`
//! input arcs onto the merger's (or root's) in-port 0. Total input rate
//! `n` propagates through the tree to give each leaf rate `n/m`. Real
//! outputs uniform at `n/m`.
//!
//! ## Construction summary
//!
//! - `(n, 1)`: `n = 1` is a trivial passthrough; `n ≥ 2` uses a single
//!   merger splitter with all inputs sideloaded onto in-port 0 and one
//!   out-arc to the single real output.
//! - `(n, m)` for `m ≥ 2`:
//!   1. `pow2 = next_power_of_two(m)`, `leftover = pow2 - m`.
//!   2. Build the `(1, pow2)` complete binary tree (`pow2 - 1` splitters,
//!      BFS-heap layout). Leaves `0..m` become real outputs; leaves
//!      `m..pow2` become feedback edges.
//!   3. If `leftover > 0`: prepend merger M (idx 0). M.in0 gets all `n`
//!      input arcs sideloaded; M.in1 gets all `leftover` feedback edges
//!      sideloaded. M's two outputs feed the root's two in-ports.
//!   4. If `leftover == 0` (dyadic m): no merger; all `n` inputs sideload
//!      onto root.in0. Root.in1 is empty (rate 0 by relaxation).
//!
//! Each leaf carries rate `(n + total_feedback) / pow2`. With
//! `total_feedback = leftover × (n/m)`, that resolves to `n/m`. ✓

use thiserror::Error;

use crate::balancer::graph::{Arc, BalancerGraph, Sink, Source};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SynthError {
    #[error("invalid shape: ({n}, {m}) — both must be ≥ 1")]
    InvalidShape { n: u32, m: u32 },
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
    if m == 1 {
        return Ok(synth_n_to_one(n));
    }
    Ok(synth_n_to_m(n, m))
}

/// `(1, 1)` is a single arc; `(n, 1)` for `n ≥ 2` is one merger splitter
/// with all `n` inputs sideloaded onto in-port 0 and one out-arc to the
/// single real output. The splitter's in-port 1 and out-port 1 are both
/// empty (multi-arc relaxation).
fn synth_n_to_one(n: u32) -> BalancerGraph {
    debug_assert!(n >= 1);
    if n == 1 {
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
    let mut arcs: Vec<Arc> = (0..n)
        .map(|i| Arc {
            src: Source::Input(i),
            dst: Sink::Splitter { idx: 0, port: 0 },
        })
        .collect();
    arcs.push(Arc {
        src: Source::Splitter { idx: 0, port: 0 },
        dst: Sink::Output(0),
    });
    let g = BalancerGraph::new(n, 1, 1, arcs);
    debug_assert!(g.validate().is_ok());
    g
}

/// `(n, m)` with `m ≥ 2`. See module docstring for the construction.
fn synth_n_to_m(n: u32, m: u32) -> BalancerGraph {
    debug_assert!(m >= 2);
    let pow2 = m.next_power_of_two();
    let leftover = pow2 - m;
    let n_inner = pow2 - 1;
    let need_merger = leftover > 0;
    let inner_offset = if need_merger { 1u32 } else { 0u32 };
    let total_splitters = n_inner + inner_offset;

    let mut arcs: Vec<Arc> = Vec::new();

    // Sideload all n input arcs onto in-port 0 of either M (if merger) or
    // the root (if dyadic). Both live at splitter idx 0 in the new layout.
    for i in 0..n {
        arcs.push(Arc {
            src: Source::Input(i),
            dst: Sink::Splitter { idx: 0, port: 0 },
        });
    }

    if need_merger {
        // M's two outputs feed the inner tree's root (idx 1).
        arcs.push(Arc {
            src: Source::Splitter { idx: 0, port: 0 },
            dst: Sink::Splitter { idx: 1, port: 0 },
        });
        arcs.push(Arc {
            src: Source::Splitter { idx: 0, port: 1 },
            dst: Sink::Splitter { idx: 1, port: 1 },
        });
    }
    // Dyadic case: no merger. The n inputs already wired to root.in0.
    // Root.in1 stays empty (rate 0 by relaxation).

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

    let g = BalancerGraph::new(n, m, total_splitters, arcs);
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

    fn check_balanced(n: u32, m: u32) {
        let g = synth(n, m).unwrap();
        let out = verify_balancer(&g).unwrap();
        assert_eq!(out.output_throughputs.len(), m as usize);
        let target = n as f64 / m as f64;
        for (i, &t) in out.output_throughputs.iter().enumerate() {
            assert!(
                approx_eq(t, target),
                "synth({}, {}) output {} = {} != {}",
                n,
                m,
                i,
                t,
                target
            );
        }
    }

    // ── (1, m) cases — same as before ──────────────────────────────────

    #[test]
    fn synth_1_1_passthrough() {
        let g = synth(1, 1).unwrap();
        assert_eq!(g.n_splitters, 0);
        check_balanced(1, 1);
    }

    #[test]
    fn synth_1_m_dyadic() {
        for m in [2u32, 4, 8, 16] {
            let g = synth(1, m).unwrap();
            assert_eq!(g.n_splitters, m - 1);
            check_balanced(1, m);
        }
    }

    #[test]
    fn synth_1_m_non_dyadic() {
        for m in [3u32, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15] {
            check_balanced(1, m);
        }
    }

    // ── (n, 1) merger cases ────────────────────────────────────────────

    #[test]
    fn synth_n_to_one() {
        for n in 1u32..=10 {
            let g = synth(n, 1).unwrap();
            // (1, 1) has 0 splitters; (n, 1) for n ≥ 2 has 1.
            let expected_splitters = if n == 1 { 0 } else { 1 };
            assert_eq!(g.n_splitters, expected_splitters);
            // Output rate is n (sum of all inputs into single output).
            let out = verify_balancer(&g).unwrap();
            assert!(approx_eq(out.output_throughputs[0], n as f64));
        }
    }

    // ── (n, m) for n > 1, m > 1 ────────────────────────────────────────

    #[test]
    fn synth_n_n_symmetric() {
        // Every output gets n/n = 1.
        for n in [2u32, 3, 4, 5, 8] {
            check_balanced(n, n);
        }
    }

    #[test]
    fn synth_n_m_asymmetric() {
        // Issue #136 missing shapes that involve n > 1.
        for (n, m) in [(2u32, 9), (3, 10), (4, 9), (5, 9), (7, 9), (8, 9)] {
            check_balanced(n, m);
        }
    }

    /// Stress: every (n, m) for n ∈ 1..=10 and m ∈ 1..=10. Confirms the
    /// construction is exhaustive across the issue #136 size range.
    #[test]
    fn synth_every_shape_up_to_10x10() {
        for n in 1u32..=10 {
            for m in 1u32..=10 {
                check_balanced(n, m);
            }
        }
    }

    // ── Multi-arc port checks ──────────────────────────────────────────

    #[test]
    fn merger_in_port_1_has_leftover_arcs() {
        let g = synth(1, 5).unwrap();
        let count = g
            .arcs
            .iter()
            .filter(|a| matches!(a.dst, Sink::Splitter { idx: 0, port: 1 }))
            .count();
        assert_eq!(count, 3, "(1, 5) leftover = 3");

        let g = synth(1, 9).unwrap();
        let count = g
            .arcs
            .iter()
            .filter(|a| matches!(a.dst, Sink::Splitter { idx: 0, port: 1 }))
            .count();
        assert_eq!(count, 7, "(1, 9) leftover = 7");
    }

    #[test]
    fn merger_in_port_0_has_n_input_arcs() {
        for (n, m) in [(2u32, 9), (4, 9), (8, 9)] {
            let g = synth(n, m).unwrap();
            let count = g
                .arcs
                .iter()
                .filter(|a| matches!(a.dst, Sink::Splitter { idx: 0, port: 0 }))
                .count();
            assert_eq!(count, n as usize, "({}, {}) input sideload count", n, m);
        }
    }

    // ── Error cases ────────────────────────────────────────────────────

    #[test]
    fn invalid_shapes_error() {
        assert!(matches!(synth(0, 4), Err(SynthError::InvalidShape { .. })));
        assert!(matches!(synth(1, 0), Err(SynthError::InvalidShape { .. })));
        assert!(matches!(synth(0, 0), Err(SynthError::InvalidShape { .. })));
    }

    // ── Serde ──────────────────────────────────────────────────────────

    #[test]
    fn synth_serde_round_trip() {
        let g = synth(4, 9).unwrap();
        let json = serde_json::to_string(&g).unwrap();
        let g2: BalancerGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(g, g2);
    }
}
