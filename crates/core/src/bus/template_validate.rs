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
//! Public API: [`validate_template_lanes`].

use crate::bus::balancer_classify::BalancerTemplateRef;
use crate::common::lane_capacity;
use crate::models::{ItemFlow, LayoutResult, PlacedEntity, SolverResult};
use crate::validate::belt_flow::{
    check_lane_throughput, check_underground_belt_entry_sideload,
    check_underground_belt_pairs, check_underground_belt_sideloading,
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
pub fn validate_template_lanes(template: BalancerTemplateRef<'_>) -> Vec<ValidationIssue> {
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

    let mut issues = Vec::new();
    issues.extend(check_underground_belt_pairs(&layout));
    issues.extend(check_underground_belt_sideloading(&layout));
    issues.extend(check_underground_belt_entry_sideload(&layout));
    issues.extend(check_lane_throughput(&layout, Some(&solver)));
    issues
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
            },
            BalancerTemplateEntity {
                name: "underground-belt",
                x: 1,
                y: 0,
                direction: 2,
                io_type: Some("output"),
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
