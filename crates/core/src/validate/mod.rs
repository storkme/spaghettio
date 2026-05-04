//! Functional blueprint validation.
//!
//! Port of `src/validate.py` — foundation types and top-level `validate()` dispatcher.

pub mod belt_flow;
pub mod inserters;
mod fluids;
pub mod power;
pub mod underground;

pub use fluids::{
    check_fluid_network_connectivity, check_fluid_port_connectivity, check_pipe_isolation,
};

pub mod belt_structural;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::{LayoutResult, RegionKind, SolverResult};
use power::{check_pole_network_connectivity, check_power_coverage};
use rustc_hash::FxHashSet;

use belt_flow::{
    check_belt_connectivity, check_belt_flow_path,
    check_belt_flow_reachability, check_belt_junctions, check_belt_network_topology,
    check_input_rate_delivery, check_underground_belt_entry_sideload,
    check_underground_belt_pairs, check_underground_belt_sideloading,
};

/// Layout style: affects which validation checks run and how.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutStyle {
    /// Constraint-based spaghetti layout (default).
    #[default]
    Spaghetti,
    /// Deterministic row-based main-bus layout.
    Bus,
}

/// Severity level of a single validation finding.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// A single validation finding, mirroring Python's `ValidationIssue` dataclass.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    /// Category tag, e.g. `"pipe-isolation"`, `"fluid-connectivity"`, `"inserter"`, `"power"`.
    pub category: String,
    pub message: String,
    /// Optional grid position associated with the issue.
    pub x: Option<i32>,
    pub y: Option<i32>,
}

impl ValidationIssue {
    /// Construct a new issue without an associated position.
    pub fn new(severity: Severity, category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            category: category.into(),
            message: message.into(),
            x: None,
            y: None,
        }
    }

    /// Construct a new issue with an associated grid position.
    pub fn with_pos(
        severity: Severity,
        category: impl Into<String>,
        message: impl Into<String>,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            severity,
            category: category.into(),
            message: message.into(),
            x: Some(x),
            y: Some(y),
        }
    }
}

/// Raised when critical validation issues block blueprint generation.
///
/// `issues` contains the full list — both errors and warnings — so callers
/// that want a complete picture (e.g. scoreboards) don't lose warning
/// counts when an error is also present. The `Display` impl only renders
/// the error subset to keep the "Validation failed" message focused on
/// what actually blocked generation.
#[derive(Debug, Error)]
#[error("Validation failed:\n{}", format_errors(.issues))]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationError {
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }
}

fn format_errors(issues: &[ValidationIssue]) -> String {
    issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("  [{}] {}", i.severity.as_str(), i.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Tile set covered by `RegionKind::Unresolved` regions in the layout.
/// These come from clusters where the ghost-router junction solver
/// gave up (`JunctionGrowthCapped`); the speculatively-routed ghost
/// belts inside are orphans, not real layout features. Validators that
/// flag belt-to-belt adjacency consult this set so they don't pile
/// follow-on errors onto a single underlying junction failure.
pub fn unresolved_region_tiles(layout: &LayoutResult) -> FxHashSet<(i32, i32)> {
    let mut tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    for r in &layout.regions {
        if r.kind != RegionKind::Unresolved {
            continue;
        }
        for dx in 0..r.width {
            for dy in 0..r.height {
                tiles.insert((r.x + dx, r.y + dy));
            }
        }
    }
    tiles
}

/// Emits one error per connected component of unresolved tiles. The
/// ghost router emits an `Unresolved` region per individual tile, so a
/// single failed junction often appears as a cluster of 1×1 regions —
/// emitting one error per region inflated counts (a 10-tile failed
/// crossing counted as 10 errors). This BFS-coalesces adjacent
/// unresolved tiles so each underlying junction failure surfaces as
/// one error. Region-tiles inside the cluster are still excluded from
/// `belt-item-isolation` so orphan ghosts don't pile follow-on noise on
/// top.
pub fn check_unresolved_junctions(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let tiles = unresolved_region_tiles(layout);
    if tiles.is_empty() {
        return Vec::new();
    }
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut components: Vec<((i32, i32), usize)> = Vec::new();
    for &start in &tiles {
        if visited.contains(&start) {
            continue;
        }
        let mut queue = vec![start];
        let mut size = 0usize;
        let mut anchor = start;
        while let Some(t) = queue.pop() {
            if !visited.insert(t) {
                continue;
            }
            size += 1;
            if t < anchor {
                anchor = t;
            }
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let n = (t.0 + dx, t.1 + dy);
                if tiles.contains(&n) && !visited.contains(&n) {
                    queue.push(n);
                }
            }
        }
        components.push((anchor, size));
    }
    components.sort();
    components
        .into_iter()
        .map(|((x, y), size)| {
            ValidationIssue::with_pos(
                Severity::Error,
                "unresolved-junction",
                format!(
                    "Junction solver could not resolve a crossing near ({},{}) \
                     covering {} tile{}. Orphan ghost belts in this cluster are \
                     excluded from belt-adjacency checks.",
                    x,
                    y,
                    size,
                    if size == 1 { "" } else { "s" },
                ),
                x,
                y,
            )
        })
        .collect()
}

/// Surface "balancer template missing" as a warning per affected family.
///
/// Background: when `stamp_family_balancer` finds neither a direct
/// `(n, m)` template nor a gcd-decomposable `(n/g, m/g)` template, it
/// returns an empty entity vec and the producer→trunk handoff is silently
/// dropped. The downstream symptom is dead-end belts at the row's exit
/// column (see PU@3/s ore red copper-plate (4, 9) — issue #136 / PR #257).
///
/// `BalancerStamped { template_found: false }` trace events flag exactly
/// this case. Read them off `layout.trace` and emit a warning per shape so
/// users see "missing balancer template (4, 9) for copper-plate" instead
/// of having to chase the dead-end belts back to their cause.
///
/// Warning, not Error — the layout is still rendered (with broken
/// connectivity), and Pool fallback can sometimes produce a valid
/// alternative. The downstream belt-dead-end errors fire too if connectivity
/// is genuinely broken.
pub fn check_balancer_template_coverage(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let Some(trace) = layout.trace.as_ref() else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    for ev in trace {
        if let crate::trace::TraceEvent::BalancerStamped {
            item, shape, template_found, ..
        } = ev
        {
            if !*template_found {
                issues.push(ValidationIssue::new(
                    Severity::Warning,
                    "missing-balancer-template",
                    format!(
                        "no balancer template for shape ({}, {}) for item {item}; \
                         producer→trunk handoff dropped (downstream belts will dead-end)",
                        shape.0, shape.1,
                    ),
                ));
            }
        }
    }
    issues
}

/// Count "No N→M balancer template for X" warnings on a layout.
///
/// These warnings are emitted inline by `bus::layout::layout_pass` when a
/// `LaneFamily`'s `(n, m)` shape has no direct template AND no gcd-
/// decomposition path. Cheap proxy used by the decomposition-search
/// hard-constraint check (`docs/rfp-decomposition-search.md`) — avoids
/// running the full validator just to spot unstampable shapes.
///
/// Reads `LayoutResult.warnings` directly (no trace dependency, unlike
/// `check_balancer_template_coverage`).
pub fn count_missing_balancer_template_warnings(layout: &LayoutResult) -> usize {
    layout
        .warnings
        .iter()
        .filter(|w| w.contains("balancer template"))
        .count()
}

/// Run all functional validation checks on a layout.
///
/// Returns a list of issues found.  Returns `Err(ValidationError)` if any
/// error-severity issues are present.
pub fn validate(
    layout_result: &LayoutResult,
    solver_result: Option<&SolverResult>,
    layout_style: LayoutStyle,
) -> Result<Vec<ValidationIssue>, ValidationError> {
    use rayon::prelude::*;

    let layout = layout_result;
    let solver = solver_result;

    // Individual validation checks must NOT call `trace::emit` — the
    // trace collector is thread-local, so events raised from a rayon
    // worker thread would either panic (if the thread-local isn't
    // initialised there) or silently vanish. The only trace emit from
    // this function is the terminal `ValidationCompleted` below, which
    // runs on the caller's thread after `par_iter` collects. If you
    // ever need per-check tracing, gather the data into the returned
    // `ValidationIssue` list and emit once from here.
    let checks: Vec<Box<dyn Fn() -> Vec<ValidationIssue> + Send + Sync>> = vec![
        Box::new(|| check_power_coverage(layout)),
        Box::new(|| check_pole_network_connectivity(layout)),
        Box::new(|| inserters::check_inserter_chains(layout, solver)),
        Box::new(|| inserters::check_inserter_direction(layout)),
        Box::new(|| check_pipe_isolation(layout)),
        Box::new(|| check_fluid_port_connectivity(layout, layout_style)),
        Box::new(|| check_fluid_network_connectivity(layout)),
        Box::new(|| check_belt_connectivity(layout, solver)),
        Box::new(|| check_belt_flow_path(layout, solver, layout_style)),
        Box::new(|| belt_structural::check_entity_overlaps(layout)),
        Box::new(|| belt_structural::check_belt_throughput(layout)),
        Box::new(|| belt_structural::check_output_belt_coverage(layout, solver)),
        Box::new(|| if layout_style == LayoutStyle::Spaghetti {
            check_belt_network_topology(layout, solver)
        } else {
            vec![]
        }),
        Box::new(|| check_belt_junctions(layout)),
        Box::new(|| check_underground_belt_pairs(layout)),
        Box::new(|| check_underground_belt_sideloading(layout)),
        Box::new(|| check_underground_belt_entry_sideload(layout)),
        Box::new(|| belt_structural::check_belt_dead_ends(layout)),
        Box::new(|| belt_structural::check_belt_loops(layout)),
        Box::new(|| belt_structural::check_belt_item_isolation(layout)),
        Box::new(|| check_unresolved_junctions(layout)),
        Box::new(|| belt_structural::check_belt_inserter_conflict(layout)),
        Box::new(|| check_belt_flow_reachability(layout, solver, layout_style)),
        Box::new(|| belt_structural::check_lane_throughput(layout, solver)),
        Box::new(|| check_input_rate_delivery(layout, solver)),
        Box::new(|| check_balancer_template_coverage(layout)),
    ];

    let issues: Vec<ValidationIssue> = checks.par_iter().flat_map(|f| f()).collect();

    let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warning_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();
    crate::trace::emit(crate::trace::TraceEvent::ValidationCompleted {
        error_count,
        warning_count,
        issues: issues.iter().map(|i| crate::trace::ValidationIssueTrace {
            severity: i.severity.as_str().to_string(),
            category: i.category.clone(),
            message: i.message.clone(),
            x: i.x,
            y: i.y,
        }).collect(),
    });

    let any_errors = issues.iter().any(|i| i.severity == Severity::Error);
    if any_errors {
        // Pass the full issues list (errors + warnings) so callers that
        // do `Err(e) => e.issues` keep an accurate picture. Without this,
        // a single masking error silently dropped every warning produced
        // in the same run (issue #298).
        return Err(ValidationError::new(issues));
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, LayoutResult, PlacedEntity};

    fn empty_layout() -> LayoutResult {
        LayoutResult {
            entities: vec![],
            width: 0,
            height: 0,
            ..Default::default()
        }
    }

    fn layout_with_machine() -> LayoutResult {
        LayoutResult {
            entities: vec![PlacedEntity {
                name: "assembling-machine-1".to_string(),
                x: 0,
                y: 0,
                direction: EntityDirection::North,
                recipe: Some("iron-gear-wheel".to_string()),
                io_type: None,
                carries: None,
                mirror: false,
                segment_id: None,
                ..Default::default()
            }],
            width: 10,
            height: 10,
            ..Default::default()
        }
    }

    #[test]
    fn severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
    }

    #[test]
    fn issue_new_has_no_position() {
        let issue = ValidationIssue::new(Severity::Error, "pipe-isolation", "test message");
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, "pipe-isolation");
        assert_eq!(issue.message, "test message");
        assert_eq!(issue.x, None);
        assert_eq!(issue.y, None);
    }

    #[test]
    fn issue_with_pos_stores_coordinates() {
        let issue = ValidationIssue::with_pos(Severity::Warning, "power", "no pole", 3, 7);
        assert_eq!(issue.severity, Severity::Warning);
        assert_eq!(issue.x, Some(3));
        assert_eq!(issue.y, Some(7));
    }

    #[test]
    fn validation_error_contains_issues() {
        let issues = vec![
            ValidationIssue::new(Severity::Error, "pipe-isolation", "fluids merged"),
            ValidationIssue::new(Severity::Error, "power", "no coverage"),
        ];
        let err = ValidationError::new(issues.clone());
        assert_eq!(err.issues.len(), 2);
        assert_eq!(err.issues[0].category, "pipe-isolation");
    }

    #[test]
    fn validation_error_message_format() {
        let issues = vec![ValidationIssue::new(Severity::Error, "power", "no pole nearby")];
        let err = ValidationError::new(issues);
        let msg = err.to_string();
        assert!(msg.contains("Validation failed:"));
        assert!(msg.contains("[error]"));
        assert!(msg.contains("no pole nearby"));
    }

    #[test]
    fn validate_empty_layout_returns_ok_with_no_poles_warning() {
        let lr = empty_layout();
        let result = validate(&lr, None, LayoutStyle::Spaghetti);
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "power");
    }

    #[test]
    fn validate_with_machine_returns_errors() {
        let lr = layout_with_machine();
        let result = validate(&lr, None, LayoutStyle::Bus);
        assert!(result.is_err(), "expected errors for a machine with no belts");
    }

    #[test]
    fn validation_error_carries_warnings_alongside_errors() {
        // Regression test for #298: when both errors and warnings exist,
        // the Err path used to drop the warnings, hiding pre-existing
        // issues from any caller that checked `e.issues.len()`.
        let issues = vec![
            ValidationIssue::new(Severity::Error, "pipe-isolation", "fluids merged"),
            ValidationIssue::new(Severity::Warning, "input-rate-delivery", "slow input"),
            ValidationIssue::new(Severity::Warning, "belt-flow-reachability", "stranded furnace"),
        ];
        let err = ValidationError::new(issues);
        assert_eq!(err.issues.len(), 3, "all issues must survive on Err path");
        assert_eq!(
            err.issues.iter().filter(|i| i.severity == Severity::Error).count(),
            1
        );
        assert_eq!(
            err.issues.iter().filter(|i| i.severity == Severity::Warning).count(),
            2
        );
        // Display should still focus on errors only.
        let msg = err.to_string();
        assert!(msg.contains("fluids merged"), "error message must surface");
        assert!(!msg.contains("slow input"), "warnings shouldn't pollute error message");
    }

    #[test]
    fn validate_default_layout_style_is_spaghetti() {
        assert_eq!(LayoutStyle::default(), LayoutStyle::Spaghetti);
    }

    #[test]
    fn layout_style_equality() {
        assert_eq!(LayoutStyle::Bus, LayoutStyle::Bus);
        assert_ne!(LayoutStyle::Bus, LayoutStyle::Spaghetti);
    }
}
