//! Lane-aware validation of standalone balancer templates.
//!
//! Phase 1 of [`docs/rfp-balancer-bake-lane-validation.md`]. Synthesises a
//! minimal [`LayoutResult`] from a [`BalancerTemplateRef`] and runs the
//! lane-aware validators from [`crate::validate::belt_flow`] against it.
//! Catches lane-imbalance and underground-belt sideloading bugs that
//! [`balancer_classify::classify`] (which models splitters as default
//! 50/50 distributors over single-lane flow) cannot detect.
//!
//! Lane rules being checked (see memories `feedback_belt_lane_rules.md`,
//! `feedback_sideload_ug.md`):
//!   - UG inputs must be fed straight, not from the side, or only one
//!     lane gets loaded.
//!   - UG exits must not collide head-on with another belt.
//!   - UG pairs must be reachable within max-reach distance, with no
//!     intercepting UG of the same tier+direction in between.
//!   - No surface belt's lane may exceed `lane_capacity(belt_name)` once
//!     the topology has propagated saturated input rates downstream.
//!
//! Public API: [`validate_template_lanes`], [`check_throughput_unlimited`].

use crate::bus::balancer_classify::BalancerTemplateRef;
use crate::common::lane_capacity;
use crate::models::{ItemFlow, LayoutResult, PlacedEntity, SolverResult};
use crate::validate::belt_flow::{
    check_lane_throughput, check_underground_belt_entry_sideload,
    check_underground_belt_pairs, check_underground_belt_sideloading,
    compute_lane_rates,
};
use crate::validate::ValidationIssue;

/// Sentinel item used to seed the saturated-source rate propagation in
/// [`compute_lane_rates`](crate::validate::belt_flow::compute_lane_rates).
/// Choosing a name that won't collide with any real recipe keeps the
/// audit independent from the recipe DB.
const TEST_ITEM: &str = "__balancer_audit_item";

/// Run the lane-aware validators against a standalone balancer template.
///
/// The synthesis is deliberately small (well under the 50 LOC kill-criterion
/// budget): convert template entities to [`PlacedEntity`]s, mark every belt
/// as carrying a sentinel item, and feed a synthetic [`SolverResult`] whose
/// only external input is that sentinel item at a saturating rate.
///
/// The saturation rate is `belt_throughput * min(N, M)` so neither input
/// nor output belts are intentionally over-driven — any per-lane throughput
/// excess in the result indicates a real lane-imbalance bug in the
/// template, not a contrived input-rate setup.
///
/// All entities are stamped at the yellow tier (`transport-belt`) — the
/// templates are tier-agnostic and yellow has the lowest per-lane capacity
/// (`7.5/s`), which is the conservative threshold for "would this layout
/// over-saturate a lane in the worst case."
///
/// Returns the concatenated issue list from all four lane-relevant
/// checkers. An empty list means the template passes lane validation.
///
/// Validates at full saturation. For partial-saturation testing, see
/// [`validate_template_lanes_at`].
pub fn validate_template_lanes(template: BalancerTemplateRef<'_>) -> Vec<ValidationIssue> {
    validate_template_lanes_at(template, 1.0)
}

/// Like [`validate_template_lanes`], but parameterised by a saturation
/// fraction in `[0.0, 1.0]`. The seeded input rate is
/// `belt_throughput * min(N, M) * saturation_fraction`.
///
/// Used to verify lane correctness at partial loads. The three UG-belt
/// validators are topological and rate-independent; only
/// `check_lane_throughput` is rate-sensitive. With the iterative walker
/// (PR #283) rate scaling is linear, so partial saturation should never
/// surface lane-throughput errors that don't also appear at full
/// saturation. This function mainly serves as a regression guard
/// against future walker changes that break that invariant.
///
/// `saturation_fraction` is clamped to `[0.0, 1.0]`.
pub fn validate_template_lanes_at(
    template: BalancerTemplateRef<'_>,
    saturation_fraction: f64,
) -> Vec<ValidationIssue> {
    let fraction = saturation_fraction.clamp(0.0, 1.0);

    let entities = synthesize_entities(template);
    let layout = LayoutResult {
        entities,
        width: template.width as i32,
        height: template.height as i32,
        ..Default::default()
    };

    // Saturate inputs at the lowest belt tier (yellow). belt_throughput =
    // 2 * lane_capacity = 15.0/s. min(N, M) caps the total so we don't
    // intentionally over-drive a merger — any excess per-lane finding is
    // therefore a real layout bug.
    let belt_throughput = lane_capacity("transport-belt") * 2.0;
    let saturate =
        belt_throughput * template.n_inputs.min(template.n_outputs) as f64 * fraction;
    let solver = SolverResult {
        machines: Vec::new(),
        external_inputs: vec![ItemFlow {
            item: TEST_ITEM.to_string(),
            rate: saturate,
            is_fluid: false,
            module_id: 0,
        }],
        external_outputs: Vec::new(),
        dependency_order: Vec::new(),
    };

    let mut issues = Vec::new();
    issues.extend(check_underground_belt_pairs(&layout));
    issues.extend(check_underground_belt_sideloading(&layout));
    issues.extend(check_underground_belt_entry_sideload(&layout));
    issues.extend(check_lane_throughput(&layout, Some(&solver)));
    issues
}

/// Debug helper — synthesise the same layout as `validate_template_lanes`
/// and return the per-tile lane-rate map. Used by `debug_single_shape` in
/// `tests/balancer_lane_audit.rs` to trace propagation.
pub fn compute_template_lane_rates(
    template: BalancerTemplateRef<'_>,
) -> rustc_hash::FxHashMap<(i32, i32), [f64; 2]> {
    let entities = synthesize_entities(template);
    let layout = LayoutResult {
        entities,
        width: template.width as i32,
        height: template.height as i32,
        ..Default::default()
    };
    let belt_throughput = lane_capacity("transport-belt") * 2.0;
    let saturate = belt_throughput * template.n_inputs.min(template.n_outputs) as f64;
    let solver = SolverResult {
        machines: Vec::new(),
        external_inputs: vec![ItemFlow {
            item: TEST_ITEM.to_string(),
            rate: saturate,
            is_fluid: false,
            module_id: 0,
        }],
        external_outputs: Vec::new(),
        dependency_order: Vec::new(),
    };
    compute_lane_rates(&layout, Some(&solver))
}

/// Convert template entities to [`PlacedEntity`]s. Only the *input
/// tiles* (per [`BalancerTemplateRef::input_tiles`]) carry the sentinel
/// item — every other entity stays `carries: None`.
///
/// This matters for the rate-propagation seed in
/// [`compute_lane_rates`](crate::validate::belt_flow::compute_lane_rates):
/// it picks "source" tiles by `belt_carries.get(pos) == Some(item)
/// && !feeders.contains(pos)`. If we marked splitter tiles as
/// carrying the item, any splitter whose left half has no upstream
/// belt (common for the first splitter in any template) would also be
/// classified as a source, double-injecting the external rate and
/// producing spurious lane-throughput findings. Restricting the
/// `carries` tag to the explicit input tiles ensures only those tiles
/// seed external rate.
fn synthesize_entities(template: BalancerTemplateRef<'_>) -> Vec<PlacedEntity> {
    let input_set: std::collections::HashSet<(i32, i32)> =
        template.input_tiles.iter().copied().collect();
    template
        .entities
        .iter()
        .map(|e| {
            let item = if input_set.contains(&(e.x, e.y)) {
                Some(TEST_ITEM)
            } else {
                None
            };
            // Yellow tier across the board. Stamping at origin (0, 0)
            // keeps coordinates identical to the template's relative
            // offsets for easier debugging when an issue is reported.
            e.stamp(
                0,
                0,
                "transport-belt",
                "splitter",
                "underground-belt",
                item,
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Throughput-unlimited (TU) check
// ---------------------------------------------------------------------------

/// Run the throughput-unlimited (TU) check against a standalone balancer
/// template.
///
/// ## What is TU?
///
/// A balancer is *throughput-unlimited* if, for every subset of `k` active
/// inputs (1 ≤ k ≤ n), each of the `m` outputs receives exactly
/// `k * belt_rate / m` — the theoretically maximum achievable under equal
/// splitting. A non-TU balancer may internally bottleneck and deliver less
/// total throughput than the inputs supply.
///
/// ## How we check it
///
/// We use the existing lane-rate walker ([`compute_lane_rates`]) as the rate
/// model, which is already used for lane-throughput validation. For three
/// representative input-subset sizes (k=1, k=⌊n/2⌋, k=n-1) we:
///
/// 1. Synthesise a `LayoutResult` where only the first `k` input tiles
///    carry the sentinel item (as sources).
/// 2. Set the external input rate to `k × belt_throughput` (so each active
///    source injects exactly one belt's worth).
/// 3. Run the rate-walker and read the combined (left + right) rate at each
///    output tile.
/// 4. Check all output totals are equal to `k × belt_throughput / m`
///    (within a tolerance of `0.5/s`).
///
/// Issues are emitted at `Severity::Warning` since TU is a desirable quality
/// attribute but not a hard correctness requirement for the bus engine.
///
/// ## Limitations
///
/// The lane-walker uses a static continuous model. For balancers with
/// feedback loops that the cycle-breaker cannot resolve, some output tiles
/// may converge to 0 rate — these cases are flagged as "inconclusive" rather
/// than a definitive TU failure. The max-flow-based
/// [`BalancerClass`](crate::bus::balancer_classify::BalancerClass) check
/// in `balancer_classify` is more reliable for detecting TU structurally;
/// this check is complementary in that it tests the *lane-level* behaviour
/// under partial loading.
///
/// Mergers (n > m, e.g. (4,1)) that lack splitter priority annotations will
/// inherently fail this check under partial input — standard 50/50 splitters
/// route flow to dead-end outputs rather than concentrating it. Raynquist-
/// style TU mergers use `input_priority`/`output_priority` to route around
/// this.
pub fn check_throughput_unlimited(template: BalancerTemplateRef<'_>) -> Vec<ValidationIssue> {
    use crate::validate::Severity;

    let n = template.n_inputs as usize;
    let m = template.n_outputs as usize;
    let belt_throughput = lane_capacity("transport-belt") * 2.0; // 15.0 /s for yellow

    // Trivially TU: single input or single output, nothing to check under
    // partial-input scenarios (only one scenario is k=n=1, which is always
    // trivially satisfied).
    if n <= 1 || m == 0 {
        return Vec::new();
    }

    // Representative subset sizes: k=1, k=n/2 (floor), k=n-1.
    // Deduplicated so we don't run the same k twice for small n.
    let mut ks: Vec<usize> = Vec::new();
    ks.push(1);
    if n / 2 > 1 && n / 2 < n - 1 {
        ks.push(n / 2);
    }
    if n - 1 > 1 {
        ks.push(n - 1);
    }
    ks.dedup();

    let mut issues = Vec::new();

    for k in ks {
        let result = run_partial_scenario(template, k, belt_throughput);
        match result {
            PartialScenarioResult::Ok => {} // TU holds for this k
            PartialScenarioResult::NonUniform {
                expected,
                min_output,
                max_output,
            } => {
                issues.push(ValidationIssue::new(
                    Severity::Warning,
                    "throughput-unlimited",
                    format!(
                        "({n}, {m}) balancer: with {k}/{n} inputs active, outputs are not \
                         uniform (expected {expected:.2}/s each, got [{min_output:.2}, \
                         {max_output:.2}]/s range). NOT throughput-unlimited.",
                    ),
                ));
            }
            PartialScenarioResult::LowThroughputNotTu {
                expected_total,
                actual_total,
            } => {
                issues.push(ValidationIssue::new(
                    Severity::Warning,
                    "throughput-unlimited",
                    format!(
                        "({n}, {m}) balancer: with {k}/{n} inputs active, total output \
                         {actual_total:.2}/s < expected {expected_total:.2}/s (splitter \
                         network routes {:.0}% of flow to unreachable dead-end outputs). \
                         NOT throughput-unlimited; needs priority annotations to concentrate \
                         partial-input flow.",
                        100.0 * actual_total / expected_total.max(0.001),
                    ),
                ));
            }
            PartialScenarioResult::WalkerStall {
                expected_total,
            } => {
                issues.push(ValidationIssue::new(
                    Severity::Warning,
                    "throughput-unlimited",
                    format!(
                        "({n}, {m}) balancer: with {k}/{n} inputs active, rate-walker \
                         produced 0/s total output (expected {expected_total:.2}/s). \
                         Likely a feedback loop the cycle-breaker could not resolve. \
                         TU status inconclusive — check classifier for structural verdict.",
                    ),
                ));
            }
        }
    }

    issues
}

#[derive(Debug)]
enum PartialScenarioResult {
    /// All outputs are uniform at the expected rate.
    Ok,
    /// Outputs differ more than tolerance (non-uniform distribution).
    NonUniform {
        expected: f64,
        min_output: f64,
        max_output: f64,
    },
    /// Total output is significantly below `k * belt_throughput`, but nonzero —
    /// flow is being routed to dead-end splitter outputs (non-TU, not a stall).
    /// Typical for mergers (n > m) without priority annotations.
    LowThroughputNotTu {
        expected_total: f64,
        actual_total: f64,
    },
    /// Total output is effectively zero despite nonzero input — the rate-walker
    /// stalled due to an unresolvable feedback loop or cycle-breaker failure.
    /// TU status is inconclusive for this scenario.
    WalkerStall {
        expected_total: f64,
    },
}

/// Simulate the template with `k` of `n` active inputs (the first `k`
/// by index order) and return whether outputs are uniform at the expected
/// rate `k * belt_throughput / m`.
///
/// Uses only the first `k` input tiles so the "subset" is deterministic.
fn run_partial_scenario(
    template: BalancerTemplateRef<'_>,
    k: usize,
    belt_throughput: f64,
) -> PartialScenarioResult {
    let m = template.n_outputs as usize;

    // Set of input tiles that are active (first k by index).
    let active_inputs: std::collections::HashSet<(i32, i32)> = template
        .input_tiles
        .iter()
        .take(k)
        .copied()
        .collect();

    // Synthesise entities: only active input tiles carry the item.
    let entities: Vec<PlacedEntity> = template
        .entities
        .iter()
        .map(|e| {
            let item = if active_inputs.contains(&(e.x, e.y)) {
                Some(TEST_ITEM)
            } else {
                None
            };
            e.stamp(0, 0, "transport-belt", "splitter", "underground-belt", item)
        })
        .collect();

    let layout = LayoutResult {
        entities,
        width: template.width as i32,
        height: template.height as i32,
        ..Default::default()
    };

    // Total external rate = k belts worth. The walker distributes this
    // evenly across all k source tiles → each active input gets belt_throughput.
    let total_input_rate = (k as f64) * belt_throughput;
    let solver = SolverResult {
        machines: Vec::new(),
        external_inputs: vec![ItemFlow {
            item: TEST_ITEM.to_string(),
            rate: total_input_rate,
            is_fluid: false,
            module_id: 0,
        }],
        external_outputs: Vec::new(),
        dependency_order: Vec::new(),
    };

    let lane_rates = compute_lane_rates(&layout, Some(&solver));

    // Sum left+right lanes at each output tile.
    let output_totals: Vec<f64> = template
        .output_tiles
        .iter()
        .map(|&pos| {
            lane_rates
                .get(&pos)
                .map(|&[l, r]| l + r)
                .unwrap_or(0.0)
        })
        .collect();

    let total_output: f64 = output_totals.iter().sum();
    let expected_total = total_input_rate;
    // Tolerance: 5% of expected total or 0.1 /s, whichever is larger.
    let throughput_tolerance = (expected_total * 0.05_f64).max(0.1);
    // "Stall" threshold: output is effectively zero (< 1% of expected).
    let stall_threshold = expected_total * 0.01_f64;

    // Check 1: total output is effectively zero — the walker stalled.
    // This happens when feedback loops in the template prevent any rate
    // from propagating to the output tiles under partial input conditions.
    if total_output < stall_threshold {
        return PartialScenarioResult::WalkerStall { expected_total };
    }

    // Check 2: total output is below expected but nonzero — the splitter
    // network routes some fraction of flow to dead-end outputs (non-TU,
    // not a stall). This is the normal case for mergers without priority.
    if total_output < expected_total - throughput_tolerance {
        return PartialScenarioResult::LowThroughputNotTu {
            expected_total,
            actual_total: total_output,
        };
    }

    // Check 3: all outputs are uniform at the expected per-output rate (TU).
    let expected_per_output = (k as f64) * belt_throughput / (m as f64);
    let uniformity_tolerance = belt_throughput * 0.05_f64; // 5% of one belt

    let min_out = output_totals
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_out = output_totals
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    if (max_out - min_out) > uniformity_tolerance
        || (max_out - expected_per_output).abs() > uniformity_tolerance
        || (min_out - expected_per_output).abs() > uniformity_tolerance
    {
        return PartialScenarioResult::NonUniform {
            expected: expected_per_output,
            min_output: min_out,
            max_output: max_out,
        };
    }

    PartialScenarioResult::Ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::balancer_library::balancer_templates;
    use crate::validate::Severity;

    #[test]
    fn benes_4x4_has_no_lane_issues() {
        // (4, 4) Benes — no UGs, all-symmetric. Should pass cleanly. This
        // is the canary that the validator isn't generating spurious
        // findings on a known-good template.
        let templates = balancer_templates();
        let t = &templates[&(4, 4)];
        let issues = validate_template_lanes(t.into());
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .collect();
        assert!(
            errors.is_empty(),
            "(4, 4) Benes should have no lane errors, got: {errors:#?}"
        );
    }

    #[test]
    fn one_to_two_passes() {
        let templates = balancer_templates();
        let t = &templates[&(1, 2)];
        let issues = validate_template_lanes(t.into());
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .collect();
        assert!(
            errors.is_empty(),
            "(1, 2) should have no lane errors, got: {errors:#?}"
        );
    }

    /// A synthetic template with an L=1 underground-belt pair (input at x=0,
    /// output at x=1, both going east — no transit tile underground). This
    /// is structurally invalid in Factorio (minimum reach is L=2).
    ///
    /// The lane gate in `bake_missing_shapes` must reject any template that
    /// emits a `Severity::Error` with `category == "underground-belt"`.
    #[test]
    fn l1_ug_pair_is_rejected_by_gate() {
        use crate::bus::balancer_classify::BalancerTemplateRef;
        use crate::bus::balancer_library::BalancerTemplateEntity;

        // Two underground belts going east (direction=2), adjacent at x=0
        // and x=1 — the minimum L=1 pair that Factorio rejects.
        let entities = [
            BalancerTemplateEntity {
                name: "underground-belt",
                x: 0,
                y: 0,
                direction: 2,
                io_type: Some("input"),
                input_priority: None,
                output_priority: None,
            },
            BalancerTemplateEntity {
                name: "underground-belt",
                x: 1,
                y: 0,
                direction: 2,
                io_type: Some("output"),
                input_priority: None,
                output_priority: None,
            },
        ];
        let input_tiles = [(0i32, 0i32)];
        let output_tiles = [(1i32, 0i32)];

        let tref = BalancerTemplateRef {
            n_inputs: 1,
            n_outputs: 1,
            width: 2,
            height: 1,
            entities: &entities,
            input_tiles: &input_tiles,
            output_tiles: &output_tiles,
        };

        let issues = validate_template_lanes(tref);
        let ug_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error && i.category == "underground-belt")
            .collect();
        assert!(
            !ug_errors.is_empty(),
            "L=1 UG pair should produce at least one underground-belt Error, got: {issues:#?}"
        );
    }
}
