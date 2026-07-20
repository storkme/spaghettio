//! Comparison harness for Mode D balancer-gen output vs Raynquist's
//! hand-tuned reference balancers.
//!
//! ## Why
//!
//! Mode D (`solve_synth_place` in `crates/balancer-gen/scripts/place.py`)
//! synthesises splitters + belt routing in one CP-SAT solve for atomic
//! shapes that don't decompose through the compose pipeline. Phase 2C
//! of `docs/rfc-ug-sideload-prevention.md` ports our four UG-correctness
//! constraints to Mode D; once that's done, we want an objective bar
//! for "is Mode D output good enough?".
//!
//! Raynquist's published balancer collection (as imported into our
//! library) is the gold standard. This harness compares Mode D's
//! output for each baseline shape against Raynquist's entry on three
//! axes:
//!
//!   1. **Lane-clean**: Mode D output must pass `validate_template_lanes`
//!      with 0 errors and 0 warnings. Raynquist's already does; Mode D
//!      regressing this would be a hard fail.
//!   2. **Balanced classification**: Mode D output must classify as
//!      `BalancerClass::Balanced` per `classify_ref`. Throughput-only
//!      or unbalanced output is a fail.
//!   3. **Compactness ratio**: Mode D's entity count and bounding box
//!      area must be within a multiplicative bound of Raynquist's. The
//!      initial bound is loose (2x); we tighten it as Mode D matures.
//!
//! ## Status
//!
//! Mode D doesn't have UG-correctness constraints yet — phase 2C work,
//! tracked in the RFC. This test is scaffolded with `#[ignore]` and a
//! TODO at the Mode D invocation site. When phase 2C lands, wire up
//! `run_mode_d_for_shape` and remove the `#[ignore]`.
//!
//! ## Usage
//!
//! ```sh
//! # Once Mode D is wired up:
//! cargo test --manifest-path crates/core/Cargo.toml \
//!     --test mode_d_baseline -- --ignored --nocapture
//! ```

use spaghettio_core::bus::balancer_classify::{classify, BalancerClass};
use spaghettio_core::bus::balancer_generate::OwnedTemplate;
use spaghettio_core::bus::balancer_library::balancer_templates;
use spaghettio_core::bus::template_validate::validate_template_lanes;
use spaghettio_core::validate::Severity;

/// Shapes for which we have a Raynquist baseline entry in the library.
/// Add to this list as more Raynquist imports land. Each shape's
/// baseline is the entry currently in `balancer_templates()`.
const RAYNQUIST_BASELINE_SHAPES: &[(u32, u32)] = &[
    // Imported in PR #288 from https://factoriobin.com/post/KafN8H7L/245
    (5, 1),
    (5, 2),
    (5, 3),
    (5, 4),
    // (5, 6), (3, 1) tu — paste was truncated; re-import follow-up.
    // Imported in PR #290
    (1, 9),
];

/// Per-shape compactness bound. Mode D's output must be no more than
/// `entity_ratio_max × baseline.entities` and `bbox_ratio_max ×
/// (baseline.width × baseline.height)`. Bounds start loose and tighten
/// as Mode D output quality improves; pin a regression by lowering
/// the ratio for shapes Mode D consistently matches.
#[derive(Debug, Clone, Copy)]
struct CompactnessBar {
    shape: (u32, u32),
    entity_ratio_max: f64,
    bbox_ratio_max: f64,
}

const COMPACTNESS_BARS: &[CompactnessBar] = &[
    // Initial bounds: 2x entity count, 2x bbox area. Generous; meant
    // to flag *gross* regressions (Mode D producing an order-of-
    // magnitude larger layout) rather than tight fit.
    CompactnessBar { shape: (5, 1), entity_ratio_max: 2.0, bbox_ratio_max: 2.0 },
    CompactnessBar { shape: (5, 2), entity_ratio_max: 2.0, bbox_ratio_max: 2.0 },
    CompactnessBar { shape: (5, 3), entity_ratio_max: 2.0, bbox_ratio_max: 2.0 },
    CompactnessBar { shape: (5, 4), entity_ratio_max: 2.0, bbox_ratio_max: 2.0 },
    CompactnessBar { shape: (1, 9), entity_ratio_max: 2.0, bbox_ratio_max: 2.0 },
];

/// Stub for Mode D invocation. Until phase 2C wires up the bake-side
/// integration, this returns `None` and the test is `#[ignore]`d.
///
/// When implemented, this should:
///   - Spawn `balancer-gen` (or call into it as a Rust library if we
///     ever expose one) with a "Mode D synth" request for `(n, m)`.
///   - Apply the four UG-correctness constraints (Phase 2C) to the
///     CP-SAT model — without those, Mode D output is meaningless to
///     compare since it'll have sideloads.
///   - Convert the response into an `OwnedTemplate` and return it.
///
/// See `docs/rfc-ug-sideload-prevention.md` § "Phase 2C: Mode D port".
fn run_mode_d_for_shape(_n: u32, _m: u32) -> Option<OwnedTemplate> {
    // TODO(phase 2c): wire up balancer-gen Mode D invocation.
    None
}

/// Bbox area (width × height) for a template-like object.
fn bbox_area(width: u32, height: u32) -> u64 {
    (width as u64) * (height as u64)
}

#[test]
#[ignore]
fn mode_d_matches_raynquist_baseline() {
    let library = balancer_templates();
    let mut failures: Vec<String> = Vec::new();
    let mut skipped: Vec<(u32, u32)> = Vec::new();

    for &shape in RAYNQUIST_BASELINE_SHAPES {
        let (n, m) = shape;
        let baseline = match library.get(&shape) {
            Some(t) => t,
            None => {
                failures.push(format!(
                    "({n}, {m}): baseline missing from library — RAYNQUIST_BASELINE_SHAPES \
                     must reflect actual library state",
                ));
                continue;
            }
        };

        let bar = match COMPACTNESS_BARS.iter().find(|b| b.shape == shape) {
            Some(b) => b,
            None => {
                failures.push(format!(
                    "({n}, {m}): no CompactnessBar entry — every baseline shape needs one",
                ));
                continue;
            }
        };

        let candidate = match run_mode_d_for_shape(n, m) {
            Some(t) => t,
            None => {
                skipped.push(shape);
                continue;
            }
        };

        // 1. Lane-clean check.
        let issues = validate_template_lanes(candidate.as_ref());
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .collect();
        let warnings: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Warning))
            .collect();
        if !errors.is_empty() || !warnings.is_empty() {
            failures.push(format!(
                "({n}, {m}): lane validation: {} errors, {} warnings — {:?}",
                errors.len(),
                warnings.len(),
                issues.iter().map(|i| &i.category).collect::<Vec<_>>()
            ));
            continue;
        }

        // 2. Balanced classification.
        match classify(&baseline) {
            Ok(report) if matches!(report.class, BalancerClass::Balanced) => {}
            Ok(report) => {
                failures.push(format!(
                    "({n}, {m}): baseline (Raynquist's) doesn't classify as Balanced (got {:?}) \
                     — baseline drift, investigate",
                    report.class
                ));
                continue;
            }
            Err(e) => {
                failures.push(format!(
                    "({n}, {m}): baseline classify failed: {e:?}"
                ));
                continue;
            }
        }
        // Once Mode D is wired up, classify the candidate too; for now
        // we only verify the baseline didn't drift.

        // 3. Compactness.
        let baseline_entities = baseline.entities.len() as f64;
        let baseline_area = bbox_area(baseline.width, baseline.height) as f64;
        let candidate_entities = candidate.entities.len() as f64;
        let candidate_area = bbox_area(candidate.width, candidate.height) as f64;

        let entity_ratio = candidate_entities / baseline_entities;
        let bbox_ratio = candidate_area / baseline_area;

        if entity_ratio > bar.entity_ratio_max {
            failures.push(format!(
                "({n}, {m}): entity count {} exceeds {:.2}x baseline ({}) — ratio {:.2}",
                candidate.entities.len(),
                bar.entity_ratio_max,
                baseline.entities.len(),
                entity_ratio
            ));
        }
        if bbox_ratio > bar.bbox_ratio_max {
            failures.push(format!(
                "({n}, {m}): bbox area {} exceeds {:.2}x baseline ({}) — ratio {:.2}",
                candidate_area as u64,
                bar.bbox_ratio_max,
                baseline_area as u64,
                bbox_ratio
            ));
        }
    }

    // Reporting. Skipped shapes are expected until Mode D is wired up;
    // the test `#[ignore]` flag means we only run with `--ignored`,
    // and at that point we want all shapes to actually compare.
    if !skipped.is_empty() {
        eprintln!(
            "Skipped {} shape(s) — run_mode_d_for_shape returned None: {skipped:?}",
            skipped.len()
        );
        eprintln!("(this is expected until Phase 2C of rfc-ug-sideload-prevention.md is implemented.)");
    }

    if !failures.is_empty() {
        for f in &failures {
            eprintln!("FAIL: {f}");
        }
        panic!(
            "Mode D vs Raynquist baseline: {} failure(s) across {} shapes",
            failures.len(),
            RAYNQUIST_BASELINE_SHAPES.len()
        );
    }

    if skipped.len() == RAYNQUIST_BASELINE_SHAPES.len() {
        // All shapes skipped — Mode D not wired up yet. Soft-pass with
        // a clear note in stderr; CI catches this via the `#[ignore]`
        // attribute, so the test only blocks merges once Mode D is in.
        eprintln!(
            "All {} baseline shapes skipped — Mode D invocation not yet implemented.",
            RAYNQUIST_BASELINE_SHAPES.len()
        );
    }
}

/// Sanity test that always runs (no `#[ignore]`): every entry in
/// RAYNQUIST_BASELINE_SHAPES has a CompactnessBar, and every shape
/// is actually present in the library. Catches drift in the constants.
#[test]
fn raynquist_baseline_constants_consistent() {
    let library = balancer_templates();
    for &shape in RAYNQUIST_BASELINE_SHAPES {
        assert!(
            library.contains_key(&shape),
            "RAYNQUIST_BASELINE_SHAPES contains {shape:?} but library doesn't"
        );
        assert!(
            COMPACTNESS_BARS.iter().any(|b| b.shape == shape),
            "RAYNQUIST_BASELINE_SHAPES contains {shape:?} but COMPACTNESS_BARS doesn't"
        );
    }
}
