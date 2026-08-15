//! Cost-descent helper for SAT-solved crossing zones.
//!
//! Used by the interactive "Improve region" pass exposed by
//! `wasm-bindings` (up to 10 s wall-clock, streams each improvement
//! back to the browser so the user can watch the layout get cheaper).
//! `SatStrategy`'s in-layout descent is a SEPARATE inline loop in
//! `junction_sat_strategy.rs` — since #652 it is bounded by iteration
//! count alone (deterministic, no wall-clock), while this interactive
//! path keeps its deadline: reproducibility for the pipeline,
//! responsiveness for the user. (An earlier header here claimed
//! SatStrategy ran through this module — review round 2 caught that
//! as stale.)
//!
//! Both callers share the same descent SHAPE — start from a
//! known-feasible entity list, repeatedly re-solve with
//! `cap = current_cost - 1`, keep every strictly-cheaper layout —
//! but NOT the same bounds (round-4 review): this path stops on
//! UNSAT (provably optimal), a deadline, or an iteration cap, and
//! uses the natural cap; SatStrategy's inline loop additionally
//! clamps its cap under a size bound, so there UNSAT can mean
//! "none at or below the CLAMPED cap" — not proof of optimality.
//! The interactive path may therefore find layouts the pipeline's
//! bounded descent forfeits on oversized zones; that gap is the
//! accepted price of pipeline determinism (its results are
//! preview-only and are not written back to the zone cache).

use web_time::Instant;

use crate::bus::junction_cost::solution_cost;
use crate::models::{EntityDirection, LayoutRegion, PlacedEntity, PortIo, RegionKind};
use crate::sat::{solve_crossing_zone_with_cost_cap, CrossingZone, ZoneBoundary};

/// Public wrapper around `junction_sat_strategy::prune_dangling_sat_entities`
/// so external crates (wasm-bindings) can prune the descent output
/// without reaching into private modules.
pub fn prune_dangling(
    entities: Vec<PlacedEntity>,
    boundaries: &[ZoneBoundary],
    max_ug_reach: u32,
    zone_x: i32,
    zone_y: i32,
) -> Vec<PlacedEntity> {
    crate::bus::junction_sat_strategy::prune_dangling_sat_entities(
        entities, boundaries, max_ug_reach, zone_x, zone_y,
    )
}

/// Convenience: build a deadline from a wall-clock budget. Keeps
/// `web_time::Instant` use localised to this crate so callers (including
/// wasm-bindings) don't need to pull in `web_time` themselves.
pub fn deadline_in(millis: u64) -> Instant {
    Instant::now() + std::time::Duration::from_millis(millis)
}

/// One descent event. Fired once per strictly-cheaper solve found.
pub struct Improvement<'a> {
    pub entities: &'a [PlacedEntity],
    pub cost: u32,
    /// Microseconds spent inside the SAT solver for this iteration.
    pub solve_time_us: u64,
}

/// Stop condition that ended the descent. Returned so callers can tell
/// "provably optimal" apart from "we ran out of time".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// `cap - 1` solve returned UNSAT — the current best is optimal for
    /// this zone under the given constraints.
    Optimal,
    /// `cost` reached 0; nothing cheaper is expressible.
    CostZero,
    /// Hit the wall-clock deadline between iterations.
    Deadline,
    /// Hit `max_iters`.
    IterCap,
    /// Solver said SAT but the returned layout wasn't cheaper than the
    /// current best — safety bail, shouldn't happen if the cost model
    /// matches the encoder.
    WeightMismatch,
}

/// Run the cost-descent loop. Calls `on_improvement` every time a
/// strictly-cheaper layout is found (including the initial snapshot, so
/// the first event always reports the starting cost). Returns the final
/// best entities + the reason the loop stopped.
pub fn descend<F: FnMut(Improvement<'_>)>(
    zone: &CrossingZone,
    belt_tier: &str,
    max_ug_reach: u32,
    max_ug_ins: Option<u32>,
    initial: Vec<PlacedEntity>,
    deadline: Instant,
    max_iters: u32,
    mut on_improvement: F,
) -> (Vec<PlacedEntity>, StopReason) {
    let mut best = initial;
    let mut best_cost = solution_cost(&best);
    on_improvement(Improvement {
        entities: &best,
        cost: best_cost,
        solve_time_us: 0,
    });

    for _ in 0..max_iters {
        if Instant::now() >= deadline {
            return (best, StopReason::Deadline);
        }
        let Some(cap) = best_cost.checked_sub(1) else {
            return (best, StopReason::CostZero);
        };
        let (next_opt, stats) = solve_crossing_zone_with_cost_cap(
            zone,
            max_ug_reach,
            belt_tier,
            max_ug_ins,
            Some(cap),
        );
        match next_opt {
            Some(ents) => {
                let c = solution_cost(&ents);
                if c >= best_cost {
                    return (best, StopReason::WeightMismatch);
                }
                best = ents;
                best_cost = c;
                on_improvement(Improvement {
                    entities: &best,
                    cost: best_cost,
                    solve_time_us: stats.solve_time_us,
                });
            }
            None => return (best, StopReason::Optimal),
        }
    }
    (best, StopReason::IterCap)
}

/// Rebuild a `CrossingZone` from a `LayoutRegion` so the interactive
/// improve-region pass can hand it straight to the solver.
///
/// Returns `None` if the region isn't a SAT-solved zone (wrong `kind`)
/// or is missing the metadata the solver needs (`belt_tier`,
/// `max_ug_reach`).
pub fn rebuild_zone_from_region(region: &LayoutRegion) -> Option<(CrossingZone, String, u32)> {
    if region.kind != RegionKind::CrossingZone {
        return None;
    }
    let belt_tier = region.belt_tier.clone()?;
    let max_ug_reach = region.max_ug_reach?;

    let boundaries: Vec<ZoneBoundary> = region
        .ports
        .iter()
        .map(|p| ZoneBoundary {
            x: p.point.x,
            y: p.point.y,
            direction: p.point.direction,
            item: p.item.clone().unwrap_or_default(),
            is_input: matches!(p.io, PortIo::Input),
            interior: p.interior,
            belt_tier: p.belt_tier.clone(),
            channel_id: p.channel_id,
        })
        .collect();

    // Direction guard: ZoneBoundary expects a cardinal N/E/S/W. All
    // serialised RegionPort directions come from the original solve so
    // this is really just a belt-and-braces check.
    for b in &boundaries {
        debug_assert!(matches!(
            b.direction,
            EntityDirection::North
                | EntityDirection::East
                | EntityDirection::South
                | EntityDirection::West
        ));
    }

    let zone = CrossingZone {
        x: region.x,
        y: region.y,
        width: region.width as u32,
        height: region.height as u32,
        boundaries,
        forced_empty: region.forced_empty.clone(),
    };
    Some((zone, belt_tier, max_ug_reach))
}
