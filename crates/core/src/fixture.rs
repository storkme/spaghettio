//! SAT-zone fixture schema (v1).
//!
//! Shared between the regression test harness
//! (`crates/core/tests/sat_fixtures.rs`), the WASM `solve_fixture`
//! binding, and the in-canvas SAT-zone editor. See
//! `crates/core/tests/sat_fixtures/README.md` for the schema docs and
//! the workflow for adding new fixtures.

use crate::bus::junction::BeltTier;
use crate::models::{EntityDirection, PlacedEntity};
use crate::sat::{CrossingZone, ZoneBoundary};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct Fixture {
    pub version: u32,
    pub name: String,
    pub notes: Option<String>,
    pub source_url: Option<String>,
    pub seed: [i32; 2],
    pub bbox: FixtureBbox,
    #[serde(default)]
    pub forbidden: Vec<[i32; 2]>,
    pub belt_tier: String,
    pub max_reach: u32,
    pub boundaries: Vec<FixtureBoundary>,
    pub expected: FixtureExpected,
    /// Informational only in v1 — carried along but not consumed.
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureBbox {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct FixtureBoundary {
    pub x: i32,
    pub y: i32,
    pub dir: String,
    pub item: String,
    #[serde(rename = "in")]
    pub is_input: bool,
    #[serde(default)]
    pub interior: bool,
}

#[derive(Debug, Deserialize)]
pub struct FixtureExpected {
    pub mode: String,
    /// Hard anti-regression ceiling for `solve` mode: the test fails
    /// if the solver's `solution_cost` exceeds this. Ratcheted down
    /// as the solver improves — bump in the same PR that earns the
    /// improvement. Absent on legacy fixtures, which only get the
    /// mode-based check.
    #[serde(default)]
    pub max_cost: Option<u32>,
    /// Aspirational target — the known-achievable cost (hand-painted
    /// reference or proved bound). Not enforced. Reported as
    /// "gap: N" in the harness output so headroom is visible on
    /// every run. Only moves when the reference is itself revised.
    #[serde(default)]
    pub optimal_cost: Option<u32>,
}

pub fn parse_direction(dir: &str) -> EntityDirection {
    match dir {
        "North" => EntityDirection::North,
        "East" => EntityDirection::East,
        "South" => EntityDirection::South,
        "West" => EntityDirection::West,
        other => panic!("unknown direction in fixture: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Region-solver fixture (v1): full `solve_crossing` inputs snapshot
// ---------------------------------------------------------------------------
//
// Built to test the grow-and-retry region solver end-to-end, not just
// the SAT encoder. See `crates/core/tests/region_fixtures/README.md`
// for the capture workflow and schema semantics.
//
// BTreeMap is deliberate — FxHashMap serialises in hash order, producing
// noisy JSON diffs when a fixture is re-captured. BTree gives stable
// alphabetical order.

/// Serializable snapshot of every input to `junction_solver::solve_crossing`
/// plus an expected-outcome block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionFixture {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub source_url: Option<String>,

    pub seeds: Vec<(i32, i32)>,
    pub initial_specs: Vec<String>,
    pub routed_paths: BTreeMap<String, Vec<(i32, i32)>>,
    pub hard_obstacles: Vec<(i32, i32)>,
    #[serde(default)]
    pub strict_obstacles: Vec<(i32, i32)>,
    #[serde(default)]
    pub unreleasable_obstacles: Vec<(i32, i32)>,
    pub spec_belt_tiers: BTreeMap<String, BeltTier>,
    pub spec_items: BTreeMap<String, String>,
    pub spec_exit_dirs: BTreeMap<String, EntityDirection>,
    #[serde(default)]
    pub placed_entities: Vec<PlacedEntity>,
    #[serde(default)]
    pub pending_crossings: Vec<(i32, i32)>,

    pub expected: RegionExpected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionExpected {
    /// `"solve"`, `"capped"`, or `"unsatisfiable"`.
    pub mode: String,
    /// Hard anti-regression ceiling for `solve` mode. Same semantics as
    /// `FixtureExpected::max_cost` on SAT fixtures.
    #[serde(default)]
    pub max_cost: Option<u32>,
    /// Aspirational target. Reported as "gap: N" by the harness.
    #[serde(default)]
    pub optimal_cost: Option<u32>,
    /// Entities that MUST appear in the solution. Each entry matches by
    /// `(x, y, carries)` — `name` and `direction` are asserted when set,
    /// ignored when empty/None. Use this to guard against missing-input
    /// bugs where the solver produces a cheap but broken layout that
    /// fails to re-stamp a required flow entity.
    #[serde(default)]
    pub required_entities: Vec<RequiredEntity>,
}

/// Fingerprint for asserting a placed entity survived the solve. Only
/// `x`, `y`, `carries` are required for a match; `name` and `direction`
/// tighten the assertion when provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredEntity {
    pub x: i32,
    pub y: i32,
    pub carries: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub direction: Option<EntityDirection>,
}

/// Result of replaying a `RegionFixture` through `solve_crossing`.
#[derive(Debug, Clone)]
pub struct RegionReplayResult {
    /// `Some(cost)` if the region solver produced a solution; `None`
    /// otherwise (capped or no strategy succeeded).
    pub cost: Option<u32>,
    /// Placed entities from the solution, empty when `cost` is `None`.
    pub entities: Vec<PlacedEntity>,
    /// True if the region hit `MAX_REGION_TILES` / the growth-cap path.
    /// Distinguishes "capped" from "no satisfiable strategy" outcomes.
    pub capped: bool,
}

/// Run the region solver against a captured fixture. Reconstructs every
/// `solve_crossing` argument from the fixture payload, rebuilds the
/// default strategy slice, and invokes the solver. Returns a compact
/// result the test harness can assert against.
///
/// Strategies mirror the production list built in `ghost_router.rs` so
/// the replay exercises the same code path a live layout would.
pub fn replay_region_fixture(fixture: &RegionFixture) -> RegionReplayResult {
    use crate::bus::junction_cost::solution_cost;
    use crate::bus::junction_sat_strategy::{SatConstraints, SatStrategy};
    use crate::bus::junction_solver::{solve_crossing, JunctionStrategy};
    use crate::trace::{self, TraceEvent};
    use rustc_hash::{FxHashMap, FxHashSet};

    let seeds: Vec<(i32, i32)> = fixture.seeds.clone();
    let initial_specs: Vec<&str> =
        fixture.initial_specs.iter().map(|s| s.as_str()).collect();

    let routed_paths: FxHashMap<String, Vec<(i32, i32)>> = fixture
        .routed_paths
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let hard_obstacles: FxHashSet<(i32, i32)> =
        fixture.hard_obstacles.iter().copied().collect();
    let strict_obstacles: FxHashSet<(i32, i32)> =
        fixture.strict_obstacles.iter().copied().collect();
    let unreleasable_obstacles: FxHashSet<(i32, i32)> =
        fixture.unreleasable_obstacles.iter().copied().collect();
    let pending_crossings: FxHashSet<(i32, i32)> =
        fixture.pending_crossings.iter().copied().collect();

    let spec_belt_tiers: FxHashMap<String, BeltTier> = fixture
        .spec_belt_tiers
        .iter()
        .map(|(k, &v)| (k.clone(), v))
        .collect();
    let spec_items: FxHashMap<String, String> = fixture
        .spec_items
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let spec_exit_dirs: FxHashMap<String, EntityDirection> = fixture
        .spec_exit_dirs
        .iter()
        .map(|(k, &v)| (k.clone(), v))
        .collect();

    // Production strategies: same list as `ghost_router.rs:1382-1406`.
    // Kept inline instead of lifted to a pub helper because the list
    // only has two call sites (production + this replay) and copying is
    // cheaper than widening the public surface.
    let perp = crate::bus::ghost_router::perpendicular_template_strategy();
    let sat_surface = SatStrategy::surface_only();
    let sat_1ug = SatStrategy::with("sat-1ug", SatConstraints::max_ug_ins(1));
    let sat_2ug = SatStrategy::with("sat-2ug", SatConstraints::max_ug_ins(2));
    let sat_full = SatStrategy::unrestricted();
    let strategies: [&dyn JunctionStrategy; 5] = [
        &*perp,
        &sat_surface,
        &sat_1ug,
        &sat_2ug,
        &sat_full,
    ];

    // Wrap solve_crossing in a trace scope so we can detect the "capped"
    // outcome (the growth loop emits `JunctionGrowthCapped` on the tile
    // cap / iter cap / bbox-expand-failed paths).
    let guard = trace::start_trace();
    let solution = solve_crossing(
        &seeds,
        &initial_specs,
        &routed_paths,
        &hard_obstacles,
        &strict_obstacles,
        &unreleasable_obstacles,
        &spec_belt_tiers,
        &spec_items,
        &spec_exit_dirs,
        &fixture.placed_entities,
        &strategies,
        &pending_crossings,
    );
    let events = trace::drain_events();
    drop(guard);

    let capped = events
        .iter()
        .any(|e| matches!(e, TraceEvent::JunctionGrowthCapped { .. }));

    match solution {
        Some(sol) => RegionReplayResult {
            cost: Some(solution_cost(&sol.entities)),
            entities: sol.entities,
            capped,
        },
        None => RegionReplayResult {
            cost: None,
            entities: Vec::new(),
            capped,
        },
    }
}

pub fn build_zone(fixture: &Fixture) -> CrossingZone {
    let boundaries: Vec<ZoneBoundary> = fixture
        .boundaries
        .iter()
        .map(|b| ZoneBoundary {
            x: b.x,
            y: b.y,
            direction: parse_direction(&b.dir),
            item: b.item.clone(),
            is_input: b.is_input,
            interior: b.interior,
            belt_tier: None,
            channel_id: 0,
        })
        .collect();

    let forced_empty: Vec<(i32, i32)> = fixture
        .forbidden
        .iter()
        .map(|&[x, y]| (x, y))
        .collect();

    CrossingZone {
        x: fixture.bbox.x,
        y: fixture.bbox.y,
        width: fixture.bbox.w,
        height: fixture.bbox.h,
        boundaries,
        forced_empty,
    }
}
