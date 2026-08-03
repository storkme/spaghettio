//! RFC-064 Phase 3 regression gates for explicitly selected packed layouts.
//!
//! These selections are deliberately stable shelf-search coordinates rather
//! than best-of-search choices. Keep the native control beside each candidate:
//! a validator regression here must be attributable to packed routing, not the
//! underlying solve or ordinary bus builder.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::bands::{PackOrder, PackSelection};
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::bus::row_rotation::{RotationOrder, RotationSelection};
use spaghettio_core::bus::transit::measure_realized_transit;
use spaghettio_core::common::QualityTier;
use spaghettio_core::models::SolverResult;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity, ValidationIssue};

fn solve(item: &str, rate: f64, inputs: &[&str], machine: &str) -> SolverResult {
    let inputs = inputs.iter().map(|item| (*item).to_string()).collect();
    solver::solve_with_palette_exclusions_and_quality(
        item,
        rate,
        &inputs,
        &MachinePalette::default(),
        machine,
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .unwrap_or_else(|error| panic!("{item}@{rate}/s should solve: {error}"))
}

fn router_options(selection: Option<PackSelection>) -> LayoutOptions {
    LayoutOptions {
        cell_composition: CellComposition::Off,
        direct_insertion: DirectInsertion::Off,
        horizontal_candidate: false,
        band_packing: selection.is_some(),
        band_pack_selection: selection,
        ..Default::default()
    }
}

fn validation_issues(
    layout: &spaghettio_core::models::LayoutResult,
    solver_result: &SolverResult,
) -> Vec<ValidationIssue> {
    match validate::validate(layout, Some(solver_result), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(error) => error.issues,
    }
}

fn validation_errors(issues: &[ValidationIssue]) -> String {
    use std::collections::BTreeMap;

    let mut by_category: BTreeMap<&str, Vec<&ValidationIssue>> = BTreeMap::new();
    for issue in issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
    {
        by_category.entry(&issue.category).or_default().push(issue);
    }

    by_category
        .into_iter()
        .map(|(category, issues)| {
            let entries = issues
                .iter()
                .map(|issue| {
                    format!(
                        "({}, {}): {}",
                        issue.x.map_or_else(|| "?".to_string(), |x| x.to_string()),
                        issue.y.map_or_else(|| "?".to_string(), |y| y.to_string()),
                        issue.message
                    )
                })
                .collect::<Vec<_>>()
                .join("\n    ");
            format!("{category}:\n    {entries}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_valid_and_transit_measurable(
    label: &str,
    layout: &spaghettio_core::models::LayoutResult,
    solver_result: &SolverResult,
) {
    let issues = validation_issues(layout, solver_result);
    let errors = validation_errors(&issues);
    let transit = measure_realized_transit(layout, solver_result, 0.5);

    // Keep both channels in one failure while the router is being repaired.
    // The validation details localize malformed tiles, while the transit
    // failure catches a disconnected consumer even after structural errors
    // have been eliminated.
    if errors.is_empty() && transit.is_ok() {
        return;
    }

    let mut failures = Vec::new();
    if !errors.is_empty() {
        failures.push(format!(
            "validation errors grouped by category, position, and message:\n{errors}"
        ));
    }
    if let Err(error) = transit {
        failures.push(format!("realized transit is not measurable: {error}"));
    }
    panic!(
        "{label} failed RFC-064 packed-router admissibility:\n{}",
        failures.join("\n")
    );
}

fn assert_packed_and_native_are_valid(
    label: &str,
    solver_result: &SolverResult,
    selection: PackSelection,
) {
    let native = layout::build_bus_layout(solver_result, router_options(None))
        .unwrap_or_else(|error| panic!("{label} native control should build: {error}"));
    assert_valid_and_transit_measurable(&format!("{label} native control"), &native, solver_result);

    let packed = layout::build_bus_layout(solver_result, router_options(Some(selection)))
        .unwrap_or_else(|error| panic!("{label} packed selection should build: {error}"));
    assert_valid_and_transit_measurable(
        &format!("{label} packed selection"),
        &packed,
        solver_result,
    );
}

#[test]
fn sci1_ore_selected_packed_router_zero_errors() {
    let solver_result = solve(
        "automation-science-pack",
        1.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-1",
    );
    assert_packed_and_native_are_valid(
        "sci1-ore",
        &solver_result,
        PackSelection {
            gap: 7,
            target_width: 30,
            order: PackOrder::Source,
        },
    );
}

#[test]
fn belt5_ore_selected_packed_router_zero_errors() {
    let solver_result = solve("transport-belt", 5.0, &["iron-ore"], "assembling-machine-2");
    assert_packed_and_native_are_valid(
        "belt5-ore",
        &solver_result,
        PackSelection {
            gap: 8,
            target_width: 36,
            order: PackOrder::HeightDescending,
        },
    );
}

#[test]
fn packed_selected_layout_preserves_planning_metadata_and_wire_mode() {
    let solver_result = solve(
        "automation-science-pack",
        1.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-1",
    );
    let mut opts = router_options(Some(PackSelection {
        gap: 7,
        target_width: 30,
        order: PackOrder::Source,
    }));
    opts.wire_mode = spaghettio_core::power_wires::WireMode::Tree;
    opts.stacking = 2;
    opts.max_inserter_tier = spaghettio_core::bus::inserter_ladder::InserterTier::Stack;
    opts.inserter_capacity = 7;

    let packed = layout::build_bus_layout(&solver_result, opts.clone())
        .expect("packed selection should preserve its declared planning context");
    assert_eq!(packed.wire_mode, opts.wire_mode);
    assert_eq!(packed.stacking, opts.stacking);
    assert_eq!(packed.inserter_capacity, opts.inserter_capacity);
    assert_eq!(
        packed.power_wires,
        Some(spaghettio_core::power_wires::compute_pole_wires(
            &packed.entities,
            opts.wire_mode,
        ))
    );
}

#[test]
fn sci2_rotation_aware_selected_is_retracted_for_validation_warnings() {
    let solver_result = solve(
        "logistic-science-pack",
        2.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-2",
    );
    // Source rows 2, 4, and 7 are iron-plate, transport-belt, and
    // logistic-science-pack.  Rotating those three is the stable best
    // shape-clearing member of the bounded search recorded in RFC-064.
    let refusal = layout::build_rotation_aware_row_layout_selected(
        &solver_result,
        router_options(None),
        &RotationSelection {
            rotation_mask: (1 << 2) | (1 << 4) | (1 << 7),
            gap: 6,
            target_width: 67,
            order: RotationOrder::HeightDescending,
            route_priority: Some("iron-gear-wheel".to_string()),
        },
    )
    .expect_err("the formerly selected layout has 18 validator warnings and must be retracted");
    assert!(
        refusal.contains("18 validation issues (0 Errors, 18 Warnings)"),
        "{refusal}"
    );
    assert!(refusal.contains("first underground-belt"), "{refusal}");
}

#[test]
#[ignore = "RFC-064 rotation-aware row search probe; the focused refusal above is the regression"]
fn probe_sci2_rotation_aware_row_search_finds_no_warning_free_candidate() {
    let solver_result = solve(
        "logistic-science-pack",
        2.0,
        &["iron-ore", "copper-ore"],
        "assembling-machine-2",
    );
    let refusal = layout::build_rotation_aware_row_layout(&solver_result, router_options(None))
        .expect_err("all routed rotation-aware plans carry validator warnings and must refuse");
    assert!(
        refusal.contains(
            "no shape-clearing, zero-issue, transit-measurable candidate among 99 structural plans / 1089 route orders"
        ),
        "{refusal}"
    );
    assert!(
        refusal.contains("46 routed, 46 validation-rejected, 0 transit-rejected"),
        "{refusal}"
    );
    println!("{refusal}");
}
