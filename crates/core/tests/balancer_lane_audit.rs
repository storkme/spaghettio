//! Audit of lane-correctness across the entire baked balancer
//! library (`docs/rfp-balancer-bake-lane-validation.md`).
//!
//! For every `(m, n)` template in
//! [`balancer_templates`](spaghettio_core::bus::balancer_library::balancer_templates),
//! synthesise a minimal `LayoutResult` and run the lane-aware
//! validators (UG pair / UG sideload / UG entry-sideload /
//! lane-throughput). Print a markdown taxonomy table to stderr.
//!
//! **Gates by default**: as of the (7, 2) re-bake (#285) the library
//! is fully lane-clean (0 errors / 0 templates affected across all
//! 75 entries). The test now asserts that baseline holds — any future
//! change that adds a lane-throughput error to the audit will break
//! this test. Set `BALANCER_AUDIT_NO_FAIL=1` to suppress the assert
//! during exploratory work (e.g. baking new shapes, regenerating the
//! library).
//!
//! Pattern mirrors `balancer_classify::tests::audit_report` (line 811
//! of that module).
//!
//! Usage:
//!   cargo test --manifest-path crates/core/Cargo.toml \
//!       --test balancer_lane_audit -- --nocapture

use std::collections::BTreeMap;

use spaghettio_core::bus::balancer_classify::BalancerTemplateRef;
use spaghettio_core::bus::balancer_library::balancer_templates;
use spaghettio_core::bus::template_validate::validate_template_lanes;
use spaghettio_core::validate::Severity;

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

    // Gate-on-by-default. The (7, 2) re-bake (#285) brought the library
    // to 0 errors / 0 templates — the audit's job now is to keep it
    // there. Set `BALANCER_AUDIT_NO_FAIL=1` to suppress the assert
    // during exploratory work (baking new shapes, regenerating the
    // library) where transient errors are expected.
    if std::env::var("BALANCER_AUDIT_NO_FAIL").is_err() {
        assert_eq!(
            total_errors, 0,
            "balancer library audit: {total_errors} lane errors across {templates_with_errors} \
             template(s). Set BALANCER_AUDIT_NO_FAIL=1 to suppress this assert. Full report \
             above (run with --nocapture)."
        );
    }
}

/// Run the lane audit at multiple saturation levels. Discovery test:
/// the existing `audit_lane_correctness` validates at 100% saturation;
/// this variant runs the same checks at 25%, 50%, and 75% to surface
/// any rate-dependent issues that don't appear at full load.
///
/// With the iterative walker (#283), lane rates scale linearly with
/// input rate, so partial saturation should never produce
/// lane-throughput errors absent at full saturation. The three UG
/// validators are topological and rate-independent, so they should
/// also report the same issues at every level. Confirming that
/// invariant is the test's job.
#[test]
fn audit_lane_correctness_partial() {
    use spaghettio_core::bus::template_validate::validate_template_lanes_at;

    let templates = balancer_templates();
    let saturations: &[f64] = &[0.25, 0.5, 0.75];

    let mut shapes: Vec<(u32, u32)> = templates.keys().copied().collect();
    shapes.sort();

    eprintln!();
    eprintln!("# Partial-saturation audit");
    eprintln!();
    eprintln!("| saturation | errors | warnings | templates affected |");
    eprintln!("|-----------:|-------:|---------:|-------------------:|");

    for &fraction in saturations {
        let mut errors = 0usize;
        let mut warnings = 0usize;
        let mut templates_with_issues = 0usize;

        for shape in &shapes {
            let template = &templates[shape];
            let issues = validate_template_lanes_at(BalancerTemplateRef::from(template), fraction);
            let e = issues.iter().filter(|i| matches!(i.severity, Severity::Error)).count();
            let w = issues.iter().filter(|i| matches!(i.severity, Severity::Warning)).count();
            errors += e;
            warnings += w;
            if e + w > 0 {
                templates_with_issues += 1;
            }
        }
        eprintln!(
            "| {:.0}% | {} | {} | {} |",
            fraction * 100.0,
            errors,
            warnings,
            templates_with_issues
        );
    }

    // Gate: at every saturation level, no errors. The iterative walker
    // scaling property means partial-load errors imply a walker bug —
    // a regression we want to catch immediately. Warnings are
    // topological and unaffected by rate, so they're not gated here
    // (the main audit handles that).
    for &fraction in saturations {
        let total_errors: usize = shapes
            .iter()
            .map(|shape| {
                validate_template_lanes_at(
                    BalancerTemplateRef::from(&templates[shape]),
                    fraction,
                )
            })
            .map(|issues| {
                issues
                    .iter()
                    .filter(|i| matches!(i.severity, Severity::Error))
                    .count()
            })
            .sum();
        assert_eq!(
            total_errors, 0,
            "Lane-throughput errors at {:.0}% saturation: walker should scale linearly \
             — partial-load errors imply a walker regression.",
            fraction * 100.0
        );
    }
}

/// Print detailed error messages for a single template — useful for debugging
/// remaining lane-throughput issues. Run with `BALANCER_DEBUG_SHAPE=(m,n)` env var.
#[test]
fn debug_single_shape() {
    let shape_str = match std::env::var("BALANCER_DEBUG_SHAPE") {
        Ok(s) => s,
        Err(_) => return, // only runs when env var is set
    };
    // Parse "(m, n)" format
    let nums: Vec<u32> = shape_str
        .trim_matches(|c| c == '(' || c == ')')
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if nums.len() != 2 {
        panic!("BALANCER_DEBUG_SHAPE must be '(m, n)', got: {shape_str}");
    }
    let (m, n) = (nums[0], nums[1]);
    let templates = balancer_templates();
    let t = templates.get(&(m, n)).unwrap_or_else(|| panic!("template ({m},{n}) not in library"));
    let issues = validate_template_lanes(BalancerTemplateRef::from(t));
    eprintln!("\n=== ({m}, {n}) lane issues ({} total) ===", issues.len());
    for issue in &issues {
        eprintln!("  {:?} [{}] at ({:?},{:?}): {}", issue.severity, issue.category, issue.x, issue.y, issue.message);
    }

    // Dump the entity layout as ASCII art so we can visualise what's at the
    // suspect coordinates without decoding the source blueprint.
    eprintln!("\n=== ({m}, {n}) entity grid (W={}, H={}) ===", t.width, t.height);
    eprintln!("input_tiles = {:?}", t.input_tiles);
    eprintln!("output_tiles = {:?}", t.output_tiles);
    let mut grid: Vec<Vec<String>> =
        (0..t.height).map(|_| (0..t.width).map(|_| ".".to_string()).collect()).collect();
    for e in t.entities.iter() {
        let glyph = match (e.name, e.io_type, e.direction) {
            ("transport-belt", _, 0) => "↑",
            ("transport-belt", _, 2) => "→",
            ("transport-belt", _, 4) => "↓",
            ("transport-belt", _, 6) => "←",
            ("splitter", _, 0) => "S↑",
            ("splitter", _, 2) => "S→",
            ("splitter", _, 4) => "S↓",
            ("splitter", _, 6) => "S←",
            ("underground-belt", Some("input"), 0) => "U↑i",
            ("underground-belt", Some("input"), 2) => "U→i",
            ("underground-belt", Some("input"), 4) => "U↓i",
            ("underground-belt", Some("input"), 6) => "U←i",
            ("underground-belt", Some("output"), 0) => "U↑o",
            ("underground-belt", Some("output"), 2) => "U→o",
            ("underground-belt", Some("output"), 4) => "U↓o",
            ("underground-belt", Some("output"), 6) => "U←o",
            _ => "?",
        };
        if (e.x as usize) < grid[0].len() && (e.y as usize) < grid.len() {
            grid[e.y as usize][e.x as usize] = glyph.to_string();
        }
    }
    eprint!("    ");
    for x in 0..t.width {
        eprint!("{:>3} ", x);
    }
    eprintln!();
    for (y, row) in grid.iter().enumerate() {
        eprint!("{:>3}: ", y);
        for cell in row {
            eprint!("{:>3} ", cell);
        }
        eprintln!();
    }

    // Also list the raw entity records around the first issue coordinate so
    // we can see what's driving the rate value.
    if let Some(first) = issues.iter().find(|i| matches!(i.severity, spaghettio_core::validate::Severity::Error)) {
        let (Some(ix), Some(iy)) = (first.x, first.y) else { return };
        eprintln!("\n=== entities within ±2 of ({ix}, {iy}) ===");
        for e in t.entities.iter() {
            if (e.x - ix).abs() <= 2 && (e.y - iy).abs() <= 2 {
                eprintln!("  ({}, {}) name={} dir={} io={:?}", e.x, e.y, e.name, e.direction, e.io_type);
            }
        }
    }

    // Compute and dump the full lane-rate map so we can see propagation.
    use spaghettio_core::bus::template_validate::compute_template_lane_rates;
    let rates = compute_template_lane_rates(BalancerTemplateRef::from(t));
    eprintln!("\n=== lane rates (left, right) per tile ===");
    let mut keys: Vec<_> = rates.keys().copied().collect();
    keys.sort_by_key(|&(x, y)| (y, x));
    for (x, y) in keys {
        let [l, r] = rates[&(x, y)];
        if l > 0.01 || r > 0.01 {
            eprintln!("  ({x:>2}, {y:>2}): L={l:6.3}  R={r:6.3}");
        }
    }
}
