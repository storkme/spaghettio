//! [`CpSat`] — placement engine that shells out to
//! `scripts/cp_sat_placer.py` (Google OR-tools CP-SAT via Python).
//!
//! ## Why subprocess + Python?
//!
//! The `cp_sat` Rust crate exists but requires the OR-tools C++ library
//! pre-installed at a known path (e.g. `/opt/ortools/include`) — heavier
//! setup than Python's self-contained binary wheel. The Python `ortools`
//! package ships the C++ libs in the wheel, installs trivially via uv,
//! and uses the same OR-tools backend the Rust crate would. So we trade
//! a subprocess hop (~5-10 ms per call, negligible at bake time) for
//! zero new system deps.
//!
//! ## Wire format
//!
//! Rust serializes a [`CpSatRequest`] to JSON on the script's stdin; the
//! script writes a [`CpSatResponse`] on stdout. Both schemas are pinned
//! by serde derives in this module.
//!
//! ## v1 scope
//!
//! The Python script in this commit only handles `(1, 1)` — a single
//! pass-through belt. Subsequent commits expand shape coverage; the
//! wire-format / subprocess machinery here doesn't change as more
//! shapes land.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::balancer::graph::BalancerGraph;
use crate::balancer::placement::{
    PlacedTemplate, PlacementEngine, PlacementError, PlacementRequest, PlacementResult,
};

/// CP-SAT engine instance. Carries the path to the Python script and
/// the `uv` binary used to invoke it. Defaults work for the project
/// layout (`scripts/cp_sat_placer.py` from the workspace root, `uv` on
/// PATH).
#[derive(Debug, Clone)]
pub struct CpSat {
    pub script_path: String,
    pub uv_binary: String,
}

impl Default for CpSat {
    fn default() -> Self {
        Self {
            script_path: "scripts/cp_sat_placer.py".to_string(),
            uv_binary: "uv".to_string(),
        }
    }
}

impl CpSat {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_script_path(mut self, path: impl Into<String>) -> Self {
        self.script_path = path.into();
        self
    }
}

/// Wire format: request from Rust to Python. Serialize-only; the
/// borrowed graph reference makes round-trip Deserialize impossible
/// and unnecessary (the script reads JSON via Python's `json` module).
#[derive(Debug, Clone, Serialize)]
pub struct CpSatRequest<'a> {
    pub graph: &'a BalancerGraph,
    pub n: u32,
    pub m: u32,
    pub timeout_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

/// Wire format: response from Python to Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CpSatResponse {
    Ok {
        template: PlacedTemplate,
        solve_wall_ms: u64,
    },
    Unsat,
    Timeout,
    /// Engine couldn't run for some reason — string is human-readable.
    Engine {
        message: String,
    },
    /// Shape isn't yet implemented in the Python placer.
    Unimplemented {
        message: String,
    },
}

impl PlacementEngine for CpSat {
    fn name(&self) -> &'static str {
        "cp_sat"
    }

    fn place(&self, req: &PlacementRequest<'_>) -> Result<PlacementResult, PlacementError> {
        let request = CpSatRequest {
            graph: req.graph,
            n: req.n,
            m: req.m,
            timeout_ms: req.timeout.as_millis() as u64,
            seed: req.seed,
        };
        let request_json = serde_json::to_string(&request)
            .map_err(|e| PlacementError::Engine(format!("serialize request: {}", e)))?;

        let started = Instant::now();
        let mut child = Command::new(&self.uv_binary)
            .arg("run")
            .arg("--no-project")
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PlacementError::Engine(format!("spawn {}: {}", self.uv_binary, e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(request_json.as_bytes())
                .map_err(|e| PlacementError::Engine(format!("write stdin: {}", e)))?;
        }
        // stdin dropped → script sees EOF.

        let output = child
            .wait_with_output()
            .map_err(|e| PlacementError::Engine(format!("wait: {}", e)))?;
        let elapsed = started.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PlacementError::Engine(format!(
                "script exited {} ({} ms): {}",
                output.status,
                elapsed.as_millis(),
                stderr.trim()
            )));
        }
        let response: CpSatResponse = serde_json::from_slice(&output.stdout)
            .map_err(|e| PlacementError::Engine(format!("parse response: {}", e)))?;

        match response {
            CpSatResponse::Ok {
                template,
                solve_wall_ms,
            } => Ok(PlacementResult {
                template,
                solve_wall_ms,
                engine_id: "cp_sat",
            }),
            CpSatResponse::Unsat => Err(PlacementError::Unsat),
            CpSatResponse::Timeout => Err(PlacementError::Timeout(req.timeout)),
            CpSatResponse::Engine { message } => Err(PlacementError::Engine(message)),
            CpSatResponse::Unimplemented { .. } => Err(PlacementError::ShapeNotAvailable {
                n: req.n,
                m: req.m,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::balancer::synth::synth;

    /// Round-trip CpSatRequest/Response through serde_json. Independent
    /// of whether the Python script is available.
    #[test]
    fn wire_format_round_trip_ok() {
        let g = synth(1, 1).unwrap();
        let req = CpSatRequest {
            graph: &g,
            n: 1,
            m: 1,
            timeout_ms: 1000,
            seed: Some(42),
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"timeout_ms\":1000"));
        assert!(s.contains("\"seed\":42"));
    }

    #[test]
    fn wire_format_round_trip_response() {
        let template = PlacedTemplate {
            n_inputs: 1,
            n_outputs: 1,
            width: 1,
            height: 2,
            entities: vec![],
            input_tiles: vec![(0, 0)],
            output_tiles: vec![(0, 1)],
            source_blueprint: None,
        };
        let r = CpSatResponse::Ok {
            template,
            solve_wall_ms: 5,
        };
        let s = serde_json::to_string(&r).unwrap();
        let r2: CpSatResponse = serde_json::from_str(&s).unwrap();
        match r2 {
            CpSatResponse::Ok { solve_wall_ms, .. } => assert_eq!(solve_wall_ms, 5),
            _ => panic!("expected Ok variant"),
        }
    }

    /// Script-running tests are gated on `FUCKTORIO_RUN_CP_SAT=1` because
    /// they need network access to fetch ortools on first run plus the
    /// Python script in the workspace, which CI may not have. Run
    /// locally with: FUCKTORIO_RUN_CP_SAT=1 cargo test --lib cp_sat.
    fn maybe_run() -> Option<CpSat> {
        if std::env::var("FUCKTORIO_RUN_CP_SAT").is_err() {
            return None;
        }
        // CARGO_MANIFEST_DIR points at crates/core/; the script lives at
        // ../../scripts/cp_sat_placer.py.
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

    #[test]
    fn end_to_end_1_1() {
        let Some(engine) = maybe_run() else {
            return;
        };
        let g = synth(1, 1).unwrap();
        let req = PlacementRequest {
            graph: &g,
            n: 1,
            m: 1,
            timeout: Duration::from_secs(10),
            seed: None,
        };
        let result = engine.place(&req).expect("(1, 1) should place");
        assert_eq!(result.engine_id, "cp_sat");
        assert_eq!(result.template.n_inputs, 1);
        assert_eq!(result.template.n_outputs, 1);
    }
}
