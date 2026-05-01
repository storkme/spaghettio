//! End-to-end round-trip test for the [`CpSat`] placement engine.
//!
//! For each shape this engine claims to support, we:
//!   1. Synth the abstract `BalancerGraph` via `balancer::synth`.
//!   2. Call the CP-SAT engine to produce a `PlacedTemplate`.
//!   3. Convert to `OwnedTemplate` and run `topology_of_template` on it
//!      to recover the splitter graph from the placed entities.
//!   4. Convert the recovered graph back to a `BalancerGraph` via
//!      `balancer::bake::from_splitter_graph`.
//!   5. Run `verify_balancer` and assert the recovered graph is balanced
//!      at the expected `n/m` rate.
//!
//! Gated on `FUCKTORIO_RUN_CP_SAT=1` so CI doesn't try to install
//! `ortools`. Run locally with:
//!
//! ```text
//! FUCKTORIO_RUN_CP_SAT=1 cargo test --test cp_sat_round_trip
//! ```

use std::time::Duration;

use fucktorio_core::balancer::placement::cp_sat::CpSat;
use fucktorio_core::balancer::placement::{PlacementEngine, PlacementRequest};
use fucktorio_core::balancer::synth::synth;
use fucktorio_core::balancer::{from_splitter_graph, verify_balancer};
use fucktorio_core::bus::balancer_classify::{topology_of_template, BalancerTemplateRef};

fn maybe_engine() -> Option<CpSat> {
    if std::env::var("FUCKTORIO_RUN_CP_SAT").is_err() {
        return None;
    }
    let manifest = env!("CARGO_MANIFEST_DIR");
    let script = std::path::Path::new(manifest)
        .parent()
        .and_then(std::path::Path::parent)
        .map(|p| p.join("scripts").join("cp_sat_placer.py"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    Some(CpSat::default().with_script_path(script))
}

fn round_trip(n: u32, m: u32) {
    let Some(engine) = maybe_engine() else {
        return;
    };
    let graph = synth(n, m).unwrap_or_else(|e| panic!("synth({n}, {m}): {e:?}"));
    let req = PlacementRequest {
        graph: &graph,
        n,
        m,
        timeout: Duration::from_secs(10),
        seed: Some(42),
    };
    let result = engine
        .place(&req)
        .unwrap_or_else(|e| panic!("place({n}, {m}): {e:?}"));

    let owned = result
        .template
        .into_owned_template()
        .unwrap_or_else(|e| panic!("into_owned_template({n}, {m}): {e:?}"));

    let recovered = topology_of_template(BalancerTemplateRef {
        n_inputs: owned.n_inputs,
        n_outputs: owned.n_outputs,
        width: owned.width,
        height: owned.height,
        entities: &owned.entities,
        input_tiles: &owned.input_tiles,
        output_tiles: &owned.output_tiles,
    })
    .unwrap_or_else(|e| panic!("topology_of_template({n}, {m}): {e:?}"));

    let bg = from_splitter_graph(&recovered)
        .unwrap_or_else(|e| panic!("from_splitter_graph({n}, {m}): {e:?}"));

    let outcome = verify_balancer(&bg)
        .unwrap_or_else(|e| panic!("verify_balancer({n}, {m}): {e:?}"));

    let expected = n as f64 / m as f64;
    assert!(
        (outcome.real_output_throughput - expected).abs() < 1e-9,
        "({n}, {m}) expected {expected}, got {}",
        outcome.real_output_throughput
    );
}

#[test]
fn round_trip_1_1() {
    round_trip(1, 1);
}

#[test]
fn round_trip_1_2() {
    round_trip(1, 2);
}

#[test]
fn round_trip_2_1() {
    round_trip(2, 1);
}

#[test]
fn round_trip_2_2() {
    round_trip(2, 2);
}

#[test]
fn round_trip_1_4() {
    round_trip(1, 4);
}

#[test]
fn round_trip_1_8() {
    round_trip(1, 8);
}
