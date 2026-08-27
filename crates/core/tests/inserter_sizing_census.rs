//! RFC-073 Phase 0 — the inserter sizing census over the calibration-bank
//! corpus (`calibration_matrix::fixtures()`, the same builds the bank
//! measures). One CSV row per fixture: how many sides the ladder sized,
//! how full the input hands are (banded), the fullest one, and how many
//! keys were ambiguous across candidate builds. Join the rows onto the
//! measured bank (`scripts/calibration_evidence.py bank probe --csv`) by
//! fixture name to test the RFC's premise — fullness predicts deficit.
//!
//! Survey, not a gate: `cargo test --manifest-path crates/core/Cargo.toml
//! --test inserter_sizing_census -- --ignored --nocapture`.

use spaghettio_core::bus::sizing_census::{capture, side_loads, summarize, Summary};
use spaghettio_core::calibration_matrix::{build, fixtures};

#[test]
#[ignore = "survey — RFC-073 Phase 0 sizing census over the calibration bank"]
fn inserter_sizing_census_calibration_bank() {
    println!("fixture,{}", Summary::CSV_HEADER);
    let only = std::env::var("SPAGHETTIO_CENSUS_ONLY").ok();
    let mut failures = Vec::new();
    for f in fixtures().into_iter().filter(|f| only.as_deref().is_none_or(|o| o == f.name)) {
        let (built, events) = capture(|| build(&f));
        match built {
            Ok(b) => {
                let (loads, ambiguous) = side_loads(&events, &b.layout);
                if std::env::var("SPAGHETTIO_CENSUS_RAW").is_ok() {
                    for l in &loads {
                        println!("raw {}: {l:?}", f.name);
                    }
                }
                println!("{},{}", f.name, summarize(&loads, ambiguous));
            }
            Err(e) => {
                println!("{},BUILD-FAIL", f.name);
                failures.push(format!("{}: {e}", f.name));
            }
        }
    }
    for f in &failures {
        eprintln!("build failure: {f}");
    }
}
