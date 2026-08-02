//! RFC-064 Phase 3 — instrument probe for Unit B's gate numbers.
//!
//! Unit B measured deeply negative `AR_score`, `Transit(packed) = 0` via
//! total unattribution, and 8–10 regressed validation categories on every
//! fixture that packed at all. Before adjudicating that as the RFC's
//! "new, stronger falsification", this probe separates three hypotheses
//! (project discipline: attack the probe first — gate-crossing numbers are
//! not accepted until the instrument is cleared):
//!
//! (a) **Integration infidelity** — Unit A's `PackCandidate` path diverges
//!     from the `band_packing: true` flag path (the #507 falsification
//!     record). Test: byte-compare `PackCandidate(MinAreaUnderCap)` output
//!     against the flag path's output on the same solve.
//! (b) **Pre-existing scaffold debt** — the flag path's own output already
//!     fails validation the same way (RFC-063 recorded "inherited RFC-058
//!     correctness debt"; the full-validator lens was never pointed at the
//!     packed scaffold before). Test: validate the flag path's output.
//! (c) **`MinAspectRatio`-specific breakage** — the new objective produces
//!     plans the packed-net router can't legalize, where the area objective
//!     could. Test: compare validation category profiles between the two
//!     objectives' outputs.
//!
//! Run: `cargo test --manifest-path crates/core/Cargo.toml --release --test rfc064_phase3_probe -- --ignored --nocapture`

use std::collections::BTreeMap;

use rustc_hash::FxHashSet;

use spaghettio_core::bus::bands::PackObjective;
use spaghettio_core::bus::candidate_runner::{pack_candidate_plan, produce_plan};
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::common::QualityTier;
use spaghettio_core::models::{LayoutResult, SolverResult};
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle};

fn solve_fixture(item: &str, rate: f64, inputs: &[&str], machine: &str) -> SolverResult {
    let inputs_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    solver::solve_with_palette_exclusions_and_quality(
        item,
        rate,
        &inputs_set,
        &MachinePalette::default(),
        machine,
        &FxHashSet::default(),
        QualityTier::Normal,
    )
    .expect("fixture must solve")
}

fn issue_profile(layout: &LayoutResult, sr: &SolverResult) -> BTreeMap<String, usize> {
    let issues = match validate::validate(layout, Some(sr), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    let mut by_cat = BTreeMap::new();
    for i in &issues {
        *by_cat.entry(i.category.clone()).or_insert(0usize) += 1;
    }
    by_cat
}

fn json_of(layout: &LayoutResult) -> String {
    serde_json::to_string(layout).expect("layout serializes")
}

fn profile_line(name: &str, layout: &LayoutResult, sr: &SolverResult) -> String {
    let prof = issue_profile(layout, sr);
    let total: usize = prof.values().sum();
    format!(
        "  {name:<12} {}x{}  entities={}  issues={} {:?}",
        layout.width,
        layout.height,
        layout.entities.len(),
        total,
        prof
    )
}

#[test]
#[ignore = "instrument probe — run explicitly with --ignored --nocapture"]
fn probe_pack_instrument_fidelity_and_provenance_of_breakage() {
    let fixtures: &[(&str, &str, f64, &[&str], &str)] = &[
        ("sci1-ore", "automation-science-pack", 1.0, &["iron-ore", "copper-ore"], "assembling-machine-1"),
        ("sci2-ore", "logistic-science-pack", 2.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
        ("belt5-ore", "transport-belt", 5.0, &["iron-ore"], "assembling-machine-2"),
        ("insert3-ore", "inserter", 3.0, &["iron-ore", "copper-ore"], "assembling-machine-2"),
    ];

    for (label, item, rate, inputs, machine) in fixtures {
        let sr = solve_fixture(item, *rate, inputs, machine);

        let native = build_bus_layout(&sr, LayoutOptions::default()).expect("native builds");

        let flag_opts = LayoutOptions {
            band_packing: true,
            ..LayoutOptions::default()
        };
        let flag = build_bus_layout(&sr, flag_opts).expect("flag path builds");

        let area_plan = pack_candidate_plan("pack-area", PackObjective::MinAreaUnderCap);
        let pack_area = produce_plan(&area_plan, &sr, &LayoutOptions::default());

        let ar_plan = pack_candidate_plan("pack-ar", PackObjective::MinAspectRatio);
        let pack_ar = produce_plan(&ar_plan, &sr, &LayoutOptions::default());

        println!("== {label} ==");
        println!("{}", profile_line("native", &native, &sr));
        println!("{}", profile_line("flag-path", &flag, &sr));
        match &pack_area {
            Ok(l) => {
                let fidelity = if json_of(l) == json_of(&flag) {
                    "BYTE-IDENTICAL to flag path"
                } else {
                    "DIVERGES from flag path"
                };
                println!("{}   <- {fidelity}", profile_line("pack-area", l, &sr));
            }
            Err(e) => println!("  pack-area    REFUSED/ERR: {e}"),
        }
        match &pack_ar {
            Ok(l) => println!("{}", profile_line("pack-ar", l, &sr)),
            Err(e) => println!("  pack-ar      REFUSED/ERR: {e}"),
        }
        println!();
    }
}
