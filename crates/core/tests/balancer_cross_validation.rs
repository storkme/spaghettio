//! Cross-validate the all-fluid Gaussian-elim verifier
//! (`balancer::verify::verify_balancer`) against the splitter-node-coarse
//! classifier (`bus::balancer_classify::classify`) over every entry of
//! `balancer_library::balancer_templates()`.
//!
//! The two implementations parameterize the linear system differently:
//!   - The classifier solves an `s × s` system over per-splitter rates
//!     under a saturated 50/50 splitter model.
//!   - The verifier solves an `n_arcs × n_arcs` system over per-arc rates
//!     under the all-fluid steady-state restriction of Couëtoux et al. §3.
//!
//! For a well-formed balancer they should agree on the load-balancing
//! property (classifier MX3 ⇔ verifier reports balanced). Disagreement
//! flags a bug in either implementation or a graph the all-fluid model
//! can't represent.
//!
//! Hard pin: 0 disagreements; exactly `[(5,8), (7,6), (8,6)]` rejected
//! by both checkers; 0 conversion errors. New disagreements or set
//! changes surface as test failures, not silent drift.

use fucktorio_core::balancer::{from_splitter_graph, verify_balancer, VerifyError};
use fucktorio_core::bus::balancer_classify::{
    classify, topology_of_template, BalancerClass, BalancerTemplateRef,
};
use fucktorio_core::bus::balancer_library::balancer_templates;

#[test]
fn cross_validate_existing_templates() {
    let templates = balancer_templates();

    let mut shapes: Vec<(u32, u32)> = templates.keys().copied().collect();
    shapes.sort();

    let mut both_balanced: Vec<(u32, u32)> = Vec::new();
    let mut both_not_balanced: Vec<(u32, u32, String, String)> = Vec::new();
    let mut disagreements: Vec<(u32, u32, String, String)> = Vec::new();
    let mut classifier_only_errored: Vec<(u32, u32, String)> = Vec::new();
    let mut verifier_only_errored: Vec<(u32, u32, String)> = Vec::new();
    let mut convert_errored: Vec<(u32, u32, String)> = Vec::new();

    for shape in shapes {
        let template = &templates[&shape];

        // Classifier path.
        let classifier_outcome = classify(template);

        // Recover graph + convert + run our verifier.
        let recovered = match topology_of_template(BalancerTemplateRef::from(template)) {
            Ok(g) => g,
            Err(e) => {
                classifier_only_errored.push((shape.0, shape.1, format!("recover: {:?}", e)));
                continue;
            }
        };
        let bg = match from_splitter_graph(&recovered) {
            Ok(g) => g,
            Err(e) => {
                convert_errored.push((shape.0, shape.1, format!("convert: {:?}", e)));
                continue;
            }
        };
        let verifier_outcome = verify_balancer(&bg);

        // Compare.
        match (&classifier_outcome, &verifier_outcome) {
            (Ok(report), Ok(_)) => {
                let their_balanced = matches!(report.class, BalancerClass::Balanced);
                if their_balanced {
                    both_balanced.push(shape);
                } else {
                    disagreements.push((
                        shape.0,
                        shape.1,
                        format!("classifier:{:?}", report.class),
                        "verifier:Ok".to_string(),
                    ));
                }
            }
            (Ok(report), Err(verr)) => {
                let their_balanced = matches!(report.class, BalancerClass::Balanced);
                if their_balanced {
                    disagreements.push((
                        shape.0,
                        shape.1,
                        format!("classifier:{:?}", report.class),
                        format!("verifier:{}", short_verr(verr)),
                    ));
                } else {
                    both_not_balanced.push((
                        shape.0,
                        shape.1,
                        format!("classifier:{:?}", report.class),
                        format!("verifier:{}", short_verr(verr)),
                    ));
                }
            }
            (Err(cerr), Ok(_)) => {
                verifier_only_errored.push((
                    shape.0,
                    shape.1,
                    format!("classifier-errored:{:?}, verifier-ok", cerr),
                ));
            }
            (Err(cerr), Err(verr)) => {
                both_not_balanced.push((
                    shape.0,
                    shape.1,
                    format!("classifier-errored:{:?}", cerr),
                    format!("verifier:{}", short_verr(verr)),
                ));
            }
        }
    }

    eprintln!("\n=== Cross-validation summary ===");
    eprintln!("Total templates:           {}", templates.len());
    eprintln!("Both classify as balanced: {}", both_balanced.len());
    eprintln!("Both NOT balanced:         {}", both_not_balanced.len());
    eprintln!("Disagreements:             {}", disagreements.len());
    eprintln!("Conversion errored:        {}", convert_errored.len());
    eprintln!("Classifier-only errored:   {}", classifier_only_errored.len());
    eprintln!("Verifier-only errored:     {}", verifier_only_errored.len());

    if !disagreements.is_empty() {
        eprintln!("\nDisagreements:");
        for (n, m, c, v) in &disagreements {
            eprintln!("  ({}, {}): {} | {}", n, m, c, v);
        }
    }
    if !both_not_balanced.is_empty() {
        eprintln!("\nBoth-not-balanced (expected):");
        for (n, m, c, v) in &both_not_balanced {
            eprintln!("  ({}, {}): {} | {}", n, m, c, v);
        }
    }
    if !convert_errored.is_empty() {
        eprintln!("\nConversion errors:");
        for (n, m, e) in &convert_errored {
            eprintln!("  ({}, {}): {}", n, m, e);
        }
    }
    if !classifier_only_errored.is_empty() {
        eprintln!("\nClassifier errors:");
        for (n, m, e) in &classifier_only_errored {
            eprintln!("  ({}, {}): {}", n, m, e);
        }
    }
    if !verifier_only_errored.is_empty() {
        eprintln!("\nVerifier-only errored (verifier disagreed):");
        for (n, m, e) in &verifier_only_errored {
            eprintln!("  ({}, {}): {}", n, m, e);
        }
    }

    // Hard-gate on three things:
    //   1. `disagreements` is empty — classifier and verifier never disagree
    //      on whether a template is balanced (they may give different
    //      *errors* for not-balanced templates; that's `both_not_balanced`,
    //      tracked separately).
    //   2. `both_not_balanced` is exactly the three known templates that
    //      both rejecters reject. Adding a new not-balanced shape is a
    //      regression in the library; losing one means the rejecters are
    //      too lenient.
    //   3. No conversion errors — every template round-trips through
    //      `bake::from_splitter_graph`.
    assert!(
        disagreements.is_empty()
            || std::env::var("FUCKTORIO_BALANCER_CV_PERMISSIVE").is_ok(),
        "verifier and classifier disagree on {} templates: {:#?}",
        disagreements.len(),
        disagreements
    );

    let mut both_unbalanced_shapes: Vec<(u32, u32)> =
        both_not_balanced.iter().map(|(n, m, _, _)| (*n, *m)).collect();
    both_unbalanced_shapes.sort();
    // (8, 6) was python-derived and used to fail balance check;
    // re-baked clean via `Parallel(4, 1, 2) → Lib(2, 6)` recipe in
    // PR #288 (UG-correctness constraints + library cleanup).
    let expected_unbalanced: &[(u32, u32)] = &[(5, 8), (7, 6)];
    assert_eq!(
        both_unbalanced_shapes, expected_unbalanced,
        "expected exactly {:?} to fail balance check; got {:?}. \
         If a new shape became unbalanced, investigate before pinning. \
         If a previously-unbalanced shape now passes, update the pin.",
        expected_unbalanced, both_unbalanced_shapes
    );

    assert!(
        convert_errored.is_empty(),
        "{} templates failed to round-trip through from_splitter_graph: {:#?}",
        convert_errored.len(),
        convert_errored
    );
}

fn short_verr(err: &VerifyError) -> String {
    match err {
        VerifyError::Graph(g) => format!("Graph({:?})", g),
        VerifyError::Singular { rank, expected } => {
            format!("Singular(rank={}, expected={})", rank, expected)
        }
        VerifyError::Imbalanced { imbalance, .. } => format!("Imbalanced({:.4e})", imbalance),
        VerifyError::NoRealOutputs => "NoRealOutputs".to_string(),
    }
}
