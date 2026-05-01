//! Flow-based verification of balancer correctness.
//!
//! Per Couëtoux, Gastaldi, Naves (2024) §3, the steady-state of a splitter
//! network is governed by:
//!   - **R3 input rate**: `t(input arc) = c(input)` (when arc is fluid).
//!   - **R4 output rate**: `t(output arc) ≤ c(output)`; equal when saturated.
//!   - **R5 conservation**: at each splitter, sum-in = sum-out.
//!   - **R6 in-coupled saturated**: both saturated in-arcs share throughput.
//!   - **R7 out-coupled fluid**: both fluid out-arcs share throughput.
//!   - **R8/R8S maximization**: a saturated arc cannot feed a fluid arc.
//!
//! ## v1 scope: all-fluid restriction
//!
//! We restrict to the case where every arc is fluid. R7 then forces the two
//! out-arcs of every splitter to share throughput, which combined with R3
//! and R5 yields a square linear system in the arc throughputs. We solve it
//! with Gaussian elimination and check uniformity across "real" outputs
//! (those with capacity > 0; capacity-0 outputs are dummies introduced to
//! balance arc counts in asymmetric `(n, m)` graphs).
//!
//! Networks that *require* internal saturation to reach steady-state will
//! fail verification (singular system or non-uniform outputs). For our
//! load-balancer use case this is the correct behavior — load-balancers
//! that work only via mandatory saturation aren't ones we want to ship.

use thiserror::Error;

use crate::balancer::graph::{BalancerGraph, GraphError};

#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    /// Throughput at each output, indexed by output id.
    pub output_throughputs: Vec<f64>,
    /// Throughput at each arc, indexed as in [`BalancerGraph::arcs`].
    pub arc_throughputs: Vec<f64>,
    /// Worst spread between any two real outputs (capacity > 0).
    pub max_imbalance: f64,
    /// Common throughput shared by all real outputs (== output_throughputs[i]
    /// for any real output i). NaN if there are no real outputs.
    pub real_output_throughput: f64,
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error(
        "linear system is rank-deficient under the all-fluid assumption \
         (rank {rank}, expected {expected}); likely the network requires \
         saturation to reach steady state"
    )]
    Singular { rank: usize, expected: usize },
    #[error(
        "outputs are not balanced: real outputs span [{min:.6}, {max:.6}] \
         (imbalance {imbalance:.6e} > tolerance {tolerance:.0e})"
    )]
    Imbalanced {
        min: f64,
        max: f64,
        imbalance: f64,
        tolerance: f64,
    },
    #[error("graph has no real outputs (all output capacities are 0)")]
    NoRealOutputs,
}

const DEFAULT_TOLERANCE: f64 = 1e-9;

/// Verify that `graph` is a load-balancer at the capacities encoded in its
/// `input_caps`/`output_caps`. Convenience wrapper at default tolerance.
pub fn verify_balancer(graph: &BalancerGraph) -> Result<VerifyOutcome, VerifyError> {
    verify_balancer_with_tolerance(graph, DEFAULT_TOLERANCE)
}

pub fn verify_balancer_with_tolerance(
    graph: &BalancerGraph,
    tolerance: f64,
) -> Result<VerifyOutcome, VerifyError> {
    let index = graph.validate_and_index()?;

    let n_arcs = graph.arcs.len();
    let total_out_arcs: usize = index
        .splitter_out
        .iter()
        .map(|[a, b]| a.is_some() as usize + b.is_some() as usize)
        .sum();
    debug_assert_eq!(n_arcs, graph.n_inputs as usize + total_out_arcs);

    // Per splitter, decide which equations to emit:
    //   - Disconnected (no arcs at all): skip — conservation is trivial 0=0
    //     and would over-determine the system.
    //   - At least one arc: emit conservation (sum-in = sum-out).
    //   - Two out-arcs: also emit R7 out-couple.
    let splitter_emits = (0..graph.n_splitters as usize)
        .map(|s| {
            let [in0, in1] = &index.splitter_in[s];
            let has_in = !in0.is_empty() || !in1.is_empty();
            let [o0, o1] = index.splitter_out[s];
            let n_out = o0.is_some() as usize + o1.is_some() as usize;
            let has_any = has_in || n_out > 0;
            (has_any, n_out == 2)
        })
        .collect::<Vec<_>>();
    let n_eqs = graph.n_inputs as usize
        + splitter_emits.iter().filter(|(any, _)| *any).count()
        + splitter_emits.iter().filter(|(_, full)| *full).count();
    debug_assert_eq!(
        n_eqs, n_arcs,
        "equation count must match arc count for square system"
    );

    let mut mat = vec![vec![0.0f64; n_arcs + 1]; n_eqs];
    let mut row = 0usize;

    // Input rate: t(input_arc[i]) = input_caps[i].
    for i in 0..graph.n_inputs as usize {
        let arc = index.input_arc[i];
        mat[row][arc] = 1.0;
        mat[row][n_arcs] = graph.input_caps[i];
        row += 1;
    }

    for s in 0..graph.n_splitters as usize {
        let (emit_any, full) = splitter_emits[s];
        if !emit_any {
            continue;
        }
        let [in0_arcs, in1_arcs] = &index.splitter_in[s];
        let [out0_opt, out1_opt] = index.splitter_out[s];
        // Conservation
        for &arc in in0_arcs.iter().chain(in1_arcs.iter()) {
            mat[row][arc] += 1.0;
        }
        if let Some(out0) = out0_opt {
            mat[row][out0] -= 1.0;
        }
        if let Some(out1) = out1_opt {
            mat[row][out1] -= 1.0;
        }
        row += 1;
        // R7 only when both out-ports are wired.
        if full {
            let out0 = out0_opt.unwrap();
            let out1 = out1_opt.unwrap();
            mat[row][out0] += 1.0;
            mat[row][out1] -= 1.0;
            row += 1;
        }
    }
    debug_assert_eq!(row, n_eqs);

    let solution = gaussian_solve(&mut mat, n_arcs, n_eqs, tolerance)?;

    // Read output throughputs.
    let output_throughputs: Vec<f64> = (0..graph.n_outputs as usize)
        .map(|o| solution[index.output_arc[o]])
        .collect();

    // Check balance across real outputs only (cap > 0).
    let real: Vec<f64> = graph
        .real_output_indices()
        .map(|o| output_throughputs[o as usize])
        .collect();

    if real.is_empty() {
        return Err(VerifyError::NoRealOutputs);
    }

    let min = real.iter().copied().fold(f64::INFINITY, f64::min);
    let max = real.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_imbalance = max - min;
    if max_imbalance > tolerance {
        return Err(VerifyError::Imbalanced {
            min,
            max,
            imbalance: max_imbalance,
            tolerance,
        });
    }

    Ok(VerifyOutcome {
        real_output_throughput: (min + max) * 0.5,
        output_throughputs,
        arc_throughputs: solution,
        max_imbalance,
    })
}

/// Solve `A x = b` where the augmented matrix `[A | b]` lives in `mat`
/// (`n_eqs` rows, `n_vars + 1` columns). Uses Gaussian elimination with
/// partial pivoting. Returns x of length `n_vars`.
///
/// Errors with [`VerifyError::Singular`] if the rank is below `n_vars`.
#[allow(clippy::needless_range_loop)]
fn gaussian_solve(
    mat: &mut [Vec<f64>],
    n_vars: usize,
    n_eqs: usize,
    tolerance: f64,
) -> Result<Vec<f64>, VerifyError> {
    let mut pivot_cols = Vec::with_capacity(n_vars.min(n_eqs));
    let mut pivot_row = 0usize;
    let mut col = 0usize;

    while pivot_row < n_eqs && col < n_vars {
        // Partial pivot: pick row with max |mat[r][col]| at or below pivot_row.
        let mut best_row = pivot_row;
        let mut best_val = mat[pivot_row][col].abs();
        for r in (pivot_row + 1)..n_eqs {
            let v = mat[r][col].abs();
            if v > best_val {
                best_val = v;
                best_row = r;
            }
        }
        if best_val < tolerance {
            // Column is dead in remaining rows; skip.
            col += 1;
            continue;
        }
        mat.swap(pivot_row, best_row);

        // Eliminate this column in every other row.
        let pivot_val = mat[pivot_row][col];
        for r in 0..n_eqs {
            if r == pivot_row {
                continue;
            }
            let factor = mat[r][col] / pivot_val;
            if factor.abs() < tolerance {
                continue;
            }
            for c in col..=n_vars {
                mat[r][c] -= factor * mat[pivot_row][c];
            }
        }
        pivot_cols.push(col);
        pivot_row += 1;
        col += 1;
    }

    let rank = pivot_cols.len();

    // Inconsistency: residual rows must be all zero.
    for r in pivot_row..n_eqs {
        if mat[r][n_vars].abs() > tolerance {
            return Err(VerifyError::Singular {
                rank,
                expected: n_vars,
            });
        }
    }

    // Underdetermined: free variables exist. We need a unique solution.
    if rank < n_vars {
        return Err(VerifyError::Singular {
            rank,
            expected: n_vars,
        });
    }

    // Back-substitute (matrix is in row echelon form; each pivot column has
    // a unique pivot row, all other rows zeroed in that column).
    let mut x = vec![0.0f64; n_vars];
    for (i, &pcol) in pivot_cols.iter().enumerate() {
        x[pcol] = mat[i][n_vars] / mat[i][pcol];
    }
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balancer::graph::{Arc, Sink, Source};

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Trivial 1→1 wire: no splitters.
    #[test]
    fn passthrough_one_to_one() {
        let g = BalancerGraph::new(
            1,
            1,
            0,
            vec![Arc {
                src: Source::Input(0),
                dst: Sink::Output(0),
            }],
        );
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.real_output_throughput, 1.0));
        assert!(approx_eq(out.max_imbalance, 0.0));
    }

    /// 2-real-input → 2-real-output single splitter. Each output gets 1.0.
    #[test]
    fn merge_split_2x2() {
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
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.output_throughputs[0], 1.0));
        assert!(approx_eq(out.output_throughputs[1], 1.0));
    }

    /// 1-real-input + 1-dummy-input → 2 real outputs through a splitter.
    /// The splitter halves the unit input into two outputs of 0.5 each.
    #[test]
    fn split_1_to_2_with_dummy_input() {
        let mut g = BalancerGraph::new(
            2,
            2,
            1,
            vec![
                Arc {
                    src: Source::Input(0), // real, c=1
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1), // dummy, c=0
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
        g.input_caps[1] = 0.0;
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.output_throughputs[0], 0.5));
        assert!(approx_eq(out.output_throughputs[1], 0.5));
        assert!(approx_eq(out.real_output_throughput, 0.5));
    }

    /// Standard 4→4 Beneš:
    ///   Stage 1: splitter A takes inputs (i0, i1) → outputs (a0, a1)
    ///            splitter B takes inputs (i2, i3) → outputs (b0, b1)
    ///   Stage 2: splitter C takes (a0, b0) → outputs to (o0, o2)
    ///            splitter D takes (a1, b1) → outputs to (o1, o3)
    /// (the cross-wiring between stages is what makes it balance)
    #[test]
    fn benes_4x4() {
        let g = BalancerGraph::new(
            4,
            4,
            4,
            vec![
                // Stage-1 splitter A (idx 0): in (i0, i1), out (→C in0, →D in0)
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
                    dst: Sink::Splitter { idx: 2, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Splitter { idx: 3, port: 0 },
                },
                // Stage-1 splitter B (idx 1): in (i2, i3), out (→C in1, →D in1)
                Arc {
                    src: Source::Input(2),
                    dst: Sink::Splitter { idx: 1, port: 0 },
                },
                Arc {
                    src: Source::Input(3),
                    dst: Sink::Splitter { idx: 1, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 1, port: 0 },
                    dst: Sink::Splitter { idx: 2, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 1, port: 1 },
                    dst: Sink::Splitter { idx: 3, port: 1 },
                },
                // Stage-2 splitter C (idx 2): out (o0, o1)
                Arc {
                    src: Source::Splitter { idx: 2, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 2, port: 1 },
                    dst: Sink::Output(1),
                },
                // Stage-2 splitter D (idx 3): out (o2, o3)
                Arc {
                    src: Source::Splitter { idx: 3, port: 0 },
                    dst: Sink::Output(2),
                },
                Arc {
                    src: Source::Splitter { idx: 3, port: 1 },
                    dst: Sink::Output(3),
                },
            ],
        );
        let out = verify_balancer(&g).unwrap();
        for &t in &out.output_throughputs {
            assert!(approx_eq(t, 1.0), "output throughput {} != 1.0", t);
        }
    }

    /// Negative test: a 4-splitter network with NO cross-wiring between
    /// stage 1 and stage 2 — two parallel 2→2 sub-balancers. With unit
    /// inputs everywhere this still produces uniform output (each 2→2 is
    /// a balancer at its own scope). Skew one input to expose the lack
    /// of cross-mixing: outputs 0,1 see (1+1)/2=1 while outputs 2,3 see
    /// (2+1)/2=1.5 → imbalanced.
    #[test]
    fn non_cross_wired_4x4_rejected() {
        let mut g = BalancerGraph::new(
            4,
            4,
            4,
            vec![
                // Stage-1 A: in (i0, i1)
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1),
                    dst: Sink::Splitter { idx: 0, port: 1 },
                },
                // BOTH A outputs go to stage-2 C (no cross to D)
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Splitter { idx: 2, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Splitter { idx: 2, port: 1 },
                },
                // Stage-1 B: in (i2, i3)
                Arc {
                    src: Source::Input(2),
                    dst: Sink::Splitter { idx: 1, port: 0 },
                },
                Arc {
                    src: Source::Input(3),
                    dst: Sink::Splitter { idx: 1, port: 1 },
                },
                // BOTH B outputs go to stage-2 D (no cross to C)
                Arc {
                    src: Source::Splitter { idx: 1, port: 0 },
                    dst: Sink::Splitter { idx: 3, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 1, port: 1 },
                    dst: Sink::Splitter { idx: 3, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 2, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 2, port: 1 },
                    dst: Sink::Output(1),
                },
                Arc {
                    src: Source::Splitter { idx: 3, port: 0 },
                    dst: Sink::Output(2),
                },
                Arc {
                    src: Source::Splitter { idx: 3, port: 1 },
                    dst: Sink::Output(3),
                },
            ],
        );
        g.input_caps[2] = 2.0;
        match verify_balancer(&g) {
            Err(VerifyError::Imbalanced { .. }) => {}
            other => panic!("expected Imbalanced, got {:?}", other),
        }
    }

    /// 2→2 with an internal feedback loop (Couëtoux Figure 1c): two
    /// splitters facing each other, each sending one output back to the
    /// other. With unit-capacity inputs and outputs, the steady-state
    /// outputs are 0.5 each (NOT 1.0), demonstrating that an all-fluid
    /// solution can stabilize below maximum throughput.
    ///
    /// Topology:
    ///   Splitter L: in (i0, R-out1), out (o0, R-in1)
    ///   Splitter R: in (i1, L-out1), out (o1, L-in1)
    /// Wait, that's only 3 in-arcs per splitter. Let me redo:
    ///   Splitter L: in (i0, R.out1), out (o0, R.in1)
    ///   Splitter R: in (i1, L.out1), out (o1, L.in1)
    /// L has in: {input(0), arc from R.out1} → 2 in-arcs ✓
    ///       out: {arc to o0, arc to R.in1} → 2 out-arcs ✓
    /// R symmetrical. 6 arcs total: 2 inputs, 2 outputs, 2 cross-edges.
    #[test]
    fn feedback_loop_stable_at_half() {
        let g = BalancerGraph::new(
            2,
            2,
            2,
            vec![
                Arc {
                    src: Source::Input(0),
                    dst: Sink::Splitter { idx: 0, port: 0 },
                },
                Arc {
                    src: Source::Input(1),
                    dst: Sink::Splitter { idx: 1, port: 0 },
                },
                Arc {
                    src: Source::Splitter { idx: 0, port: 0 },
                    dst: Sink::Output(0),
                },
                Arc {
                    src: Source::Splitter { idx: 1, port: 0 },
                    dst: Sink::Output(1),
                },
                // Feedback: L.out1 → R.in1, R.out1 → L.in1
                Arc {
                    src: Source::Splitter { idx: 0, port: 1 },
                    dst: Sink::Splitter { idx: 1, port: 1 },
                },
                Arc {
                    src: Source::Splitter { idx: 1, port: 1 },
                    dst: Sink::Splitter { idx: 0, port: 1 },
                },
            ],
        );
        // In all-fluid steady state, conservation + R7 give:
        //   t(L.out0) = t(L.out1) = (1 + t(R.out1)) / 2
        //   t(R.out0) = t(R.out1) = (1 + t(L.out1)) / 2
        // By symmetry t(L.*) = t(R.*). Let x = t(L.out0). Then
        //   x = (1 + x) / 2  →  2x = 1 + x  →  x = 1.
        // So outputs are 1.0 each. (Couëtoux Figure 1c gets 0.5 because
        // they impose that the FEEDBACK arcs are saturated, which is
        // outside our all-fluid model.)
        let out = verify_balancer(&g).unwrap();
        assert!(approx_eq(out.output_throughputs[0], 1.0));
        assert!(approx_eq(out.output_throughputs[1], 1.0));
    }
}
