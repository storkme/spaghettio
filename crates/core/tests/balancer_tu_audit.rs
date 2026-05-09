//! Throughput-unlimited (TU) audit across the entire baked balancer library.
//!
//! For every `(m, n)` template in
//! [`balancer_templates`](spaghettio_core::bus::balancer_library::balancer_templates),
//! runs [`check_throughput_unlimited`] and prints a markdown table of results.
//!
//! **Does NOT gate-on-pass by default** — we want to discover which library
//! entries are TU and which aren't before pinning a baseline. Set
//! `BALANCER_TU_AUDIT_FAIL=1` to enable the assert (useful for CI once a
//! baseline is established).
//!
//! Usage:
//!   cargo test --manifest-path crates/core/Cargo.toml \
//!       --test balancer_tu_audit -- --nocapture

use spaghettio_core::bus::balancer_classify::{classify, BalancerClass, BalancerTemplateRef};
use spaghettio_core::bus::balancer_library::balancer_templates;
use spaghettio_core::bus::template_validate::check_throughput_unlimited;

#[test]
fn audit_throughput_unlimited() {
    let templates = balancer_templates();

    let mut shapes: Vec<(u32, u32)> = templates.keys().copied().collect();
    shapes.sort();

    #[derive(Debug)]
    struct Row {
        shape: (u32, u32),
        tu_warnings: usize,
        classifier_class: String,
        warning_messages: Vec<String>,
    }

    let mut rows: Vec<Row> = Vec::new();

    for shape in &shapes {
        let template = &templates[shape];
        let issues = check_throughput_unlimited(BalancerTemplateRef::from(template));

        // Also run the classifier so we can cross-reference TU verdicts.
        let classifier_class = match classify(template) {
            Ok(r) => match r.class {
                BalancerClass::Balanced => "MX3-balanced",
                BalancerClass::ThroughputUnlimited => "MX2b-TU",
                BalancerClass::ThroughputBalancedRate => "MX2a-sat+rate",
                BalancerClass::ThroughputLimited => "MX1-limited",
            },
            Err(_) => "classify-error",
        }
        .to_string();

        let warning_messages: Vec<String> = issues.iter().map(|i| i.message.clone()).collect();
        let tu_warnings = issues.len();

        rows.push(Row {
            shape: *shape,
            tu_warnings,
            classifier_class,
            warning_messages,
        });
    }

    // ---- Per-template table ----
    eprintln!();
    eprintln!("# Balancer library TU (throughput-unlimited) audit");
    eprintln!();
    eprintln!("| (m, n) | TU warnings | classifier | verdict |");
    eprintln!("|--------|-------------|------------|---------|");
    for row in &rows {
        let verdict = if row.tu_warnings == 0 { "PASS" } else { "WARN" };
        eprintln!(
            "| ({}, {}) | {} | {} | {} |",
            row.shape.0, row.shape.1, row.tu_warnings, row.classifier_class, verdict
        );
    }

    // ---- Discrepancy analysis ----
    eprintln!();
    eprintln!("## Cross-reference: classifier TU vs lane-walker TU");
    eprintln!();
    eprintln!("Discrepancies indicate cases where classifier and lane-walker disagree:");
    eprintln!("| (m, n) | classifier | lane-walker says |");
    eprintln!("|--------|------------|-----------------|");
    let mut any_discrepancy = false;
    for row in &rows {
        let classifier_tu = matches!(
            row.classifier_class.as_str(),
            "MX2b-TU" | "MX3-balanced"
        );
        let walker_tu = row.tu_warnings == 0;
        if classifier_tu != walker_tu {
            any_discrepancy = true;
            let walker_verdict = if walker_tu { "PASS (walker)" } else { "WARN (walker)" };
            eprintln!(
                "| ({}, {}) | {} | {} |",
                row.shape.0, row.shape.1, row.classifier_class, walker_verdict
            );
            // Print the specific warnings
            for msg in &row.warning_messages {
                eprintln!("  > {msg}");
            }
        }
    }
    if !any_discrepancy {
        eprintln!("| _(none)_ | — | — |");
    }

    // ---- Summary ----
    let total = rows.len();
    let tu_pass = rows.iter().filter(|r| r.tu_warnings == 0).count();
    let tu_warn = rows.iter().filter(|r| r.tu_warnings > 0).count();
    let classifier_tu = rows
        .iter()
        .filter(|r| matches!(r.classifier_class.as_str(), "MX2b-TU" | "MX3-balanced"))
        .count();

    eprintln!();
    eprintln!("## Summary");
    eprintln!();
    eprintln!("- Templates audited: **{total}**");
    eprintln!("- Lane-walker TU pass (0 warnings): **{tu_pass}**");
    eprintln!("- Lane-walker TU warn (≥1 warning): **{tu_warn}**");
    eprintln!("- Classifier MX2b/MX3 (structural TU): **{classifier_tu}**");
    eprintln!();

    // Conditional gate: only fail if BALANCER_TU_AUDIT_FAIL=1.
    if std::env::var("BALANCER_TU_AUDIT_FAIL").is_ok() {
        assert_eq!(
            tu_warn,
            0,
            "TU audit: {tu_warn} template(s) have lane-walker TU warnings. \
             Run with --nocapture for the full report."
        );
    }
}

/// Focused test: verify that the (4, 4) Benes balancer (a known TU design) passes.
#[test]
fn benes_4x4_is_tu() {
    let templates = balancer_templates();
    let t = &templates[&(4, 4)];
    let issues = check_throughput_unlimited(BalancerTemplateRef::from(t));
    assert!(
        issues.is_empty(),
        "(4, 4) Benes is a known TU design and should pass the TU check. Got: {issues:#?}"
    );
}

/// Focused test: verify that the (2, 2) single-splitter balancer passes TU
/// (it's the simplest TU case — one splitter, both inputs/outputs).
#[test]
fn two_to_two_is_tu() {
    let templates = balancer_templates();
    let t = &templates[&(2, 2)];
    let issues = check_throughput_unlimited(BalancerTemplateRef::from(t));
    assert!(
        issues.is_empty(),
        "(2, 2) single-splitter should pass TU check. Got: {issues:#?}"
    );
}
