//! Phase-1 audit of lane-correctness across the entire baked balancer
//! library (`docs/rfp-balancer-bake-lane-validation.md`).
//!
//! For every `(m, n)` template in
//! [`balancer_templates`](fucktorio_core::bus::balancer_library::balancer_templates),
//! synthesise a minimal `LayoutResult` and run the lane-aware
//! validators (UG pair / UG sideload / UG entry-sideload /
//! lane-throughput). Print a markdown taxonomy table to stderr.
//!
//! This test does not assert on the issue counts: the first run is
//! discovery; the report itself is the deliverable. Run with
//! `--nocapture` (or pass `BALANCER_AUDIT_FAIL=1` once we know what the
//! baseline is) to inspect findings.
//!
//! Pattern mirrors `balancer_classify::tests::audit_report` (line 811
//! of that module).
//!
//! Usage:
//!   cargo test --manifest-path crates/core/Cargo.toml \
//!       --test balancer_lane_audit -- --nocapture

use std::collections::BTreeMap;

use fucktorio_core::bus::balancer_classify::BalancerTemplateRef;
use fucktorio_core::bus::balancer_library::balancer_templates;
use fucktorio_core::bus::template_validate::validate_template_lanes;
use fucktorio_core::validate::Severity;

#[test]
fn audit_lane_correctness() {
    let templates = balancer_templates();

    let mut shapes: Vec<(u32, u32)> = templates.keys().copied().collect();
    shapes.sort();

    // Per-template summary row: (m, n, error_count, warning_count, error_categories)
    #[derive(Debug)]
    struct Row {
        shape: (u32, u32),
        errors: usize,
        warnings: usize,
        error_categories: Vec<String>,
    }
    let mut rows: Vec<Row> = Vec::new();

    // Aggregates: category -> total count across all templates
    let mut error_by_category: BTreeMap<String, usize> = BTreeMap::new();
    let mut warning_by_category: BTreeMap<String, usize> = BTreeMap::new();

    // Templates affected per category (for "how many templates show this bug")
    let mut error_templates_by_category: BTreeMap<String, Vec<(u32, u32)>> = BTreeMap::new();

    for shape in &shapes {
        let template = &templates[shape];
        let issues = validate_template_lanes(BalancerTemplateRef::from(template));

        let mut errors = 0usize;
        let mut warnings = 0usize;
        let mut local_error_cats: Vec<String> = Vec::new();
        for issue in &issues {
            match issue.severity {
                Severity::Error => {
                    errors += 1;
                    *error_by_category.entry(issue.category.clone()).or_insert(0) += 1;
                    if !local_error_cats.contains(&issue.category) {
                        local_error_cats.push(issue.category.clone());
                    }
                }
                Severity::Warning => {
                    warnings += 1;
                    *warning_by_category.entry(issue.category.clone()).or_insert(0) += 1;
                }
            }
        }
        for cat in &local_error_cats {
            error_templates_by_category
                .entry(cat.clone())
                .or_default()
                .push(*shape);
        }
        rows.push(Row {
            shape: *shape,
            errors,
            warnings,
            error_categories: local_error_cats,
        });
    }

    // ---- Per-template table ----
    eprintln!();
    eprintln!("# Balancer library lane-validation audit");
    eprintln!();
    eprintln!("| (m, n) | errors | warnings | error categories |");
    eprintln!("|--------|--------|----------|------------------|");
    for row in &rows {
        let cats = if row.error_categories.is_empty() {
            "—".to_string()
        } else {
            row.error_categories.join(", ")
        };
        eprintln!(
            "| ({}, {}) | {} | {} | {} |",
            row.shape.0, row.shape.1, row.errors, row.warnings, cats
        );
    }

    // ---- Summary by category ----
    eprintln!();
    eprintln!("## Errors by category");
    eprintln!();
    eprintln!("| category | total occurrences | templates affected |");
    eprintln!("|----------|------------------:|-------------------:|");
    for (cat, count) in &error_by_category {
        let templates_hit = error_templates_by_category
            .get(cat)
            .map(|v| v.len())
            .unwrap_or(0);
        eprintln!("| {cat} | {count} | {templates_hit} |");
    }
    if error_by_category.is_empty() {
        eprintln!("| _(none)_ | 0 | 0 |");
    }

    eprintln!();
    eprintln!("## Warnings by category");
    eprintln!();
    eprintln!("| category | total occurrences |");
    eprintln!("|----------|------------------:|");
    for (cat, count) in &warning_by_category {
        eprintln!("| {cat} | {count} |");
    }
    if warning_by_category.is_empty() {
        eprintln!("| _(none)_ | 0 |");
    }

    // ---- Worst-offender ranking ----
    let mut by_errors: Vec<&Row> = rows.iter().filter(|r| r.errors > 0).collect();
    by_errors.sort_by(|a, b| b.errors.cmp(&a.errors).then(a.shape.cmp(&b.shape)));
    eprintln!();
    eprintln!("## Top 10 worst-offender templates");
    eprintln!();
    if by_errors.is_empty() {
        eprintln!("_No templates have errors. Library is lane-clean by this validator._");
    } else {
        eprintln!("| rank | (m, n) | errors | warnings |");
        eprintln!("|-----:|--------|-------:|---------:|");
        for (i, row) in by_errors.iter().take(10).enumerate() {
            eprintln!(
                "| {} | ({}, {}) | {} | {} |",
                i + 1,
                row.shape.0,
                row.shape.1,
                row.errors,
                row.warnings,
            );
        }
    }

    // ---- High-level totals ----
    let total_templates = rows.len();
    let templates_with_errors = rows.iter().filter(|r| r.errors > 0).count();
    let templates_with_warnings = rows.iter().filter(|r| r.warnings > 0).count();
    let total_errors: usize = rows.iter().map(|r| r.errors).sum();
    let total_warnings: usize = rows.iter().map(|r| r.warnings).sum();
    eprintln!();
    eprintln!("## Totals");
    eprintln!();
    eprintln!("- Templates audited: **{total_templates}**");
    eprintln!(
        "- Templates with errors: **{templates_with_errors}** ({:.1}%)",
        100.0 * templates_with_errors as f64 / total_templates.max(1) as f64
    );
    eprintln!("- Templates with warnings: **{templates_with_warnings}**");
    eprintln!("- Total error issues: **{total_errors}**");
    eprintln!("- Total warning issues: **{total_warnings}**");
    eprintln!();

    // Optional fail-on-error gate: useful once we know the baseline. For
    // discovery we leave the test green so the report always prints.
    if std::env::var("BALANCER_AUDIT_FAIL").is_ok() {
        assert_eq!(
            templates_with_errors, 0,
            "BALANCER_AUDIT_FAIL set: {templates_with_errors} templates have lane errors"
        );
    }
}
