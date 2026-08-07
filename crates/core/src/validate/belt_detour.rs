//! Belt-detour measurement and check (owner ask, 2026-08-01): find belt
//! runs that double back on themselves or otherwise travel far more than
//! the straight-line separation between their endpoints demands.
//!
//! [`measure_belt_runs`] is the pub, check-independent helper — it
//! decomposes a layout's belt/underground network into RUNS bounded by the
//! anchor points where an item's journey genuinely starts or ends (inserter
//! drop/pickup, splitter output/input, and belt merges/"sideloads"), and
//! reports each run's realized tile length against the Manhattan separation
//! of its endpoints. It is deliberately layout-agnostic (no `Severity`/
//! `ValidationIssue` in its signature) so it can be reused outside the
//! validator: `docs/rfc-064-spaghetti-objective.md`'s Transit metric
//! (§ Metrics (b)) needs the same per-edge realized-path length, computed
//! post-route on a validated `LayoutResult` — see that RFC before changing
//! the run-boundary rules below, since a semantic change here changes what
//! that metric measures too.
//!
//! Belts and undergrounds only, this round: pipe networks are meshes, not
//! runs, and direct-insertion has no belt to measure at all.
//!
//! ## Run boundaries
//!
//! A run starts at an **entry** anchor and ends (inclusive) at the first
//! **exit** anchor reached walking forward, or at a dead end:
//!
//! - **Entry**: an inserter drop tile (covers both a plain drop and a
//!   machine-output drop — same mechanism, an inserter placing an item onto
//!   a belt tile), a tile a splitter actually feeds (its aligned output, or
//!   a perpendicular receiver it sideloads onto), a "sideload" merge tile
//!   (a belt/UG tile fed by more than one upstream neighbor), or a tile
//!   with no belt predecessor at all (a boundary input or otherwise
//!   orphaned start).
//! - **Exit**: an inserter pickup tile, the last tile actually feeding into
//!   a splitter (aligned rear input — the only input a splitter accepts),
//!   a tile whose sole successor is itself an entry (this tile's flow
//!   merges into a new run starting there), or a dead end (no flow
//!   successor at all).
//!
//! Splitters are anchors, not run tiles: a run's `entry`/`exit` never sits
//! on a splitter's own footprint, and balancer internals are decomposed
//! into the short runs between their splitters rather than attributed to
//! one path through the whole balancer (survey brief, 2026-08-01).
//!
//! Because every non-anchor tile has exactly one predecessor and one
//! successor by construction (anything else makes it an anchor), no two
//! entries' walks ever visit the same tile — the total work across every
//! run is `O(belt tile count)`, not quadratic in run count.
//!
//! ## Derivation (RFC-065 Phase 1 slice 2)
//!
//! Since 2026-08-06 the decomposition consumes the connectivity IR
//! ([`crate::connectivity::derive_connectivity`]) instead of five inline
//! tile-scans: the forward step is the node's unique
//! `{BeltFlow, Sideload, UgSpan}` out-edge (distance = Manhattan between
//! the endpoint entities — 1 for surface flow, the span length
//! underground), and every anchor ingredient above is an edge kind. Flow
//! edges are game-truthful where the old tile-walk was orientation-blind,
//! which tightens five behaviors — D1-D4 on geometry the engine never
//! emits (or already rejects as Errors), D5 on live corpus geometry —
//! each pinned in the tests below with the old behavior asserted
//! alongside for contrast:
//!
//! - (D1) runs stop at head-on contacts (a conflict, not a flow edge; the
//!   tile-walk walked through them);
//! - (D2) nothing surface-feeds a UG exit (rear feed stalls in game; the
//!   walk flowed into the exit and counted the rear feeder as a merge);
//! - (D3) splitter anchors are orientation-true (the tile-walk marked any
//!   belt ahead/behind a footprint tile; on a perpendicular belt passing
//!   behind a splitter it cut the run and orphaned the downstream tail
//!   from measurement entirely);
//! - (D4) io-untagged undergrounds are invisible (`NodeClass::Other`,
//!   K65-1 posture: no role-guessing for hand-built entities; the dir map
//!   walked them as plain tiles);
//! - (D5) a UG entrance no longer phantom-feeds the surface tile directly
//!   ahead of it (its items go underground, but its dir-map entry made
//!   the tile-walk count it as a predecessor, cutting crossing lanes
//!   mid-run on weave geometry). The one CORPUS-REACHABLE divergence —
//!   the source of every healed run boundary in the migration
//!   differential.
//!
//! The pre-migration tile-walk survives verbatim in [`reference`] as the
//! differential oracle — corpus equivalence gates live in `e2e.rs`
//! (`belt_detour_migration_differential*`).

use rustc_hash::FxHashSet;

use crate::connectivity::{derive_connectivity, ConnectivityGraph, EdgeKind, NodeClass};
use crate::models::LayoutResult;

use super::{Severity, ValidationIssue};

/// One contiguous belt/underground run between two anchor tiles (see the
/// module doc for what bounds a run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeltRun {
    /// First tile of the run (an entry anchor).
    pub entry: (i32, i32),
    /// Last tile of the run, inclusive (an exit anchor, or a dead end).
    pub exit: (i32, i32),
    /// Tiles traversed, counting an underground pair's span as the tiles
    /// it spans (the items travel that distance even though only two
    /// entities occupy it).
    pub actual_length: i64,
    /// Manhattan distance between `entry` and `exit`.
    pub direct_distance: i64,
}

impl BeltRun {
    /// `actual_length / max(direct_distance, 1)`.
    pub fn efficiency(&self) -> f64 {
        self.actual_length as f64 / self.direct_distance.max(1) as f64
    }

    /// `actual_length - direct_distance`. Never negative: `actual_length`
    /// is a lattice-path length between the same two tiles `direct_distance`
    /// measures as a Manhattan distance, and Manhattan distance is a lower
    /// bound on any such path.
    pub fn excess(&self) -> i64 {
        self.actual_length - self.direct_distance
    }

    /// Integer midpoint of entry/exit, used to position a reported issue.
    pub fn midpoint(&self) -> (i32, i32) {
        ((self.entry.0 + self.exit.0) / 2, (self.entry.1 + self.exit.1) / 2)
    }
}

/// Decompose `layout`'s belt/underground network into measured runs,
/// deriving the connectivity graph internally. See the module doc for the
/// anchor-classification rules.
pub fn measure_belt_runs(layout: &LayoutResult) -> Vec<BeltRun> {
    let graph = derive_connectivity(layout);
    measure_belt_runs_on(layout, &graph)
}

/// [`measure_belt_runs`] over a caller-supplied graph — the seam a future
/// once-per-`validate()` shared derivation hoists through.
///
/// CONTRACT: `graph` must be derived from THIS `layout`
/// (`derive_connectivity(layout)`, unmutated since) — classes and edges
/// are entity-index-aligned, so a stale or foreign graph misindexes
/// silently. Asserted on the cheap alignment signal below (bot round 1 on
/// PR #583).
pub fn measure_belt_runs_on(layout: &LayoutResult, graph: &ConnectivityGraph) -> Vec<BeltRun> {
    let n = layout.entities.len();
    assert_eq!(
        graph.classes.len(),
        n,
        "measure_belt_runs_on: graph is not derived from this layout"
    );
    let belt_like = |i: usize| {
        matches!(
            graph.classes[i],
            NodeClass::SurfaceBelt | NodeClass::UgEntrance | NodeClass::UgExit
        )
    };
    let pos = |i: usize| (layout.entities[i].x, layout.entities[i].y);

    // Single pass over the typed edge list — every anchor ingredient the
    // old tile-walk derived by neighbor scanning is an edge kind here.
    let mut flow_out: Vec<Option<usize>> = vec![None; n];
    let mut in_flow: Vec<u32> = vec![0; n];
    let mut fed_by_splitter = vec![false; n];
    let mut feeds_splitter = vec![false; n];
    let mut dropped_on = vec![false; n];
    let mut picked_from = vec![false; n];
    for e in &graph.edges {
        match e.kind {
            EdgeKind::BeltFlow | EdgeKind::Sideload | EdgeKind::UgSpan => {
                if belt_like(e.src) && belt_like(e.dst) {
                    // A belt-like node emits at most one flow edge (one
                    // surface_flow call per belt/exit, one span per
                    // entrance). Guard shape (PR #583 bot rounds 1+2,
                    // pulling opposite ways): loud in debug — every test
                    // and CI run catches builder drift — but a
                    // deterministic first-edge degrade in release, because
                    // a report-only diagnostic must never abort production
                    // `validate()` or the fold/cut admission loops that
                    // call it per candidate.
                    if flow_out[e.src].is_some() {
                        debug_assert!(
                            false,
                            "derive_connectivity emitted two flow edges from one belt-like node"
                        );
                        continue;
                    }
                    flow_out[e.src] = Some(e.dst);
                    in_flow[e.dst] += 1;
                } else if graph.classes[e.src] == NodeClass::Splitter && belt_like(e.dst) {
                    // A splitter sideloading a perpendicular receiver
                    // still feeds it — entry, same as an aligned output.
                    fed_by_splitter[e.dst] = true;
                }
            }
            EdgeKind::SplitterOut => {
                if belt_like(e.dst) {
                    fed_by_splitter[e.dst] = true;
                }
            }
            EdgeKind::SplitterIn => {
                if belt_like(e.src) {
                    feeds_splitter[e.src] = true;
                }
            }
            EdgeKind::InserterDrop => {
                if belt_like(e.dst) {
                    dropped_on[e.dst] = true;
                }
            }
            EdgeKind::InserterPickup => {
                if belt_like(e.src) {
                    picked_from[e.src] = true;
                }
            }
        }
    }

    let is_entry =
        |i: usize| dropped_on[i] || fed_by_splitter[i] || in_flow[i] != 1;

    let mut entry_idx: Vec<usize> =
        (0..n).filter(|&i| belt_like(i) && is_entry(i)).collect();
    entry_idx.sort_by_key(|&i| pos(i));

    let mut runs = Vec::with_capacity(entry_idx.len());
    for start in entry_idx {
        let mut cur = start;
        let mut actual_length: i64 = 1;
        // Cycle guard: every non-anchor node has exactly one predecessor by
        // construction, so a single walk can only revisit a node if the
        // layout contains a true belt loop (already a separate hard error
        // from `belt_structural::check_belt_loops`) — stop rather than spin.
        let mut visited: FxHashSet<usize> = FxHashSet::default();
        visited.insert(cur);
        while !(picked_from[cur] || feeds_splitter[cur]) {
            let Some(next) = flow_out[cur] else { break };
            if is_entry(next) || visited.contains(&next) {
                break;
            }
            let (cx, cy) = pos(cur);
            let (nx, ny) = pos(next);
            actual_length += ((nx - cx).abs() + (ny - cy).abs()) as i64;
            cur = next;
            visited.insert(cur);
        }
        let (sx, sy) = pos(start);
        let (ex, ey) = pos(cur);
        runs.push(BeltRun {
            entry: (sx, sy),
            exit: (ex, ey),
            actual_length,
            direct_distance: ((ex - sx).abs() + (ey - sy).abs()) as i64,
        });
    }
    runs
}

/// The pre-migration tile-walk decomposition, retained VERBATIM as the
/// differential oracle for the RFC-065 Phase 1 slice-2 migration (same
/// discipline as `connectivity`'s naive-pairing reference). Test/probe use
/// only — production callers go through [`measure_belt_runs`].
#[doc(hidden)]
pub mod reference {
    use rustc_hash::FxHashSet;

    use crate::common::{
        dir_to_vec, inserter_reach, is_inserter, is_splitter, is_ug_belt, splitter_second_tile,
    };
    use crate::models::LayoutResult;
    use crate::validate::belt_flow::{belt_dir_map_from, build_splitter_siblings, build_ug_pairs};

    use super::BeltRun;

    /// The 2026-08-01 tile-walk `measure_belt_runs`, exactly as it shipped.
    pub fn measure_belt_runs_tilewalk(layout: &LayoutResult) -> Vec<BeltRun> {
        let belt_dir_map = belt_dir_map_from(&layout.entities);
        let ug_pairs = build_ug_pairs(layout);
        let splitter_tiles: FxHashSet<(i32, i32)> =
            build_splitter_siblings(layout).keys().copied().collect();

        let mut ug_input_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut ug_output_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
        for e in &layout.entities {
            if is_ug_belt(&e.name) {
                match e.io_type.as_deref() {
                    Some("input") => {
                        ug_input_tiles.insert((e.x, e.y));
                    }
                    Some("output") => {
                        ug_output_tiles.insert((e.x, e.y));
                    }
                    _ => {}
                }
            }
        }

        // One step forward: underground jump when `t` is a paired UG input,
        // otherwise the straight direction step. Never steps onto a splitter
        // tile — splitters are anchors, not run tiles (see module doc).
        let forward = |t: (i32, i32)| -> Option<((i32, i32), i64)> {
            if ug_input_tiles.contains(&t) {
                let out = *ug_pairs.get(&t)?;
                return if belt_dir_map.contains_key(&out) && !splitter_tiles.contains(&out) {
                    let dist = (out.0 - t.0).abs() as i64 + (out.1 - t.1).abs() as i64;
                    Some((out, dist))
                } else {
                    None
                };
            }
            let d = *belt_dir_map.get(&t)?;
            let (dx, dy) = dir_to_vec(d);
            let next = (t.0 + dx, t.1 + dy);
            if belt_dir_map.contains_key(&next) && !splitter_tiles.contains(&next) {
                Some((next, 1))
            } else {
                None
            }
        };

        // Every belt/UG neighbor (straight-behind OR sideload) whose own
        // direction points at `t`, plus the underground predecessor when `t`
        // is a paired UG output. Splitter neighbors are excluded — feeding
        // from a splitter is handled unconditionally by the entry/exit rules
        // below, not folded into this merge count.
        let predecessors = |t: (i32, i32)| -> Vec<(i32, i32)> {
            let mut preds = Vec::new();
            if ug_output_tiles.contains(&t) {
                if let Some(&inp) = ug_pairs.get(&t) {
                    if belt_dir_map.contains_key(&inp) && !splitter_tiles.contains(&inp) {
                        preds.push(inp);
                    }
                }
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (t.0 + dx, t.1 + dy);
                if splitter_tiles.contains(&n) {
                    continue;
                }
                if let Some(&nd) = belt_dir_map.get(&n) {
                    let (ndx, ndy) = dir_to_vec(nd);
                    if (n.0 + ndx, n.1 + ndy) == t {
                        preds.push(n);
                    }
                }
            }
            preds
        };

        // Inserter drop/pickup positions, reach-aware — the same computation
        // `inserters::check_inserter_direction` / `check_inserter_throughput`
        // use. Only counted when the tile is actually a belt/UG tile (not a
        // splitter footprint, not a machine tile).
        let mut inserter_drop: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut inserter_pickup: FxHashSet<(i32, i32)> = FxHashSet::default();
        for e in &layout.entities {
            if !is_inserter(&e.name) {
                continue;
            }
            let (dx, dy) = dir_to_vec(e.direction);
            let reach = inserter_reach(&e.name);
            let drop = (e.x + dx * reach, e.y + dy * reach);
            let pickup = (e.x - dx * reach, e.y - dy * reach);
            if belt_dir_map.contains_key(&drop) && !splitter_tiles.contains(&drop) {
                inserter_drop.insert(drop);
            }
            if belt_dir_map.contains_key(&pickup) && !splitter_tiles.contains(&pickup) {
                inserter_pickup.insert(pickup);
            }
        }

        // Splitter output (entry) / input (exit) neighbor tiles — both of a
        // splitter's two parallel footprint tiles count.
        let mut splitter_output: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut splitter_input: FxHashSet<(i32, i32)> = FxHashSet::default();
        for e in &layout.entities {
            if !is_splitter(&e.name) {
                continue;
            }
            let (dx, dy) = dir_to_vec(e.direction);
            for p in [(e.x, e.y), splitter_second_tile(e)] {
                let ahead = (p.0 + dx, p.1 + dy);
                let behind = (p.0 - dx, p.1 - dy);
                if belt_dir_map.contains_key(&ahead) && !splitter_tiles.contains(&ahead) {
                    splitter_output.insert(ahead);
                }
                if belt_dir_map.contains_key(&behind) && !splitter_tiles.contains(&behind) {
                    splitter_input.insert(behind);
                }
            }
        }

        // ENTRY_SET: inserter drops, splitter outputs, and any tile with
        // predecessor count != 1 (0 = boundary input / orphan start,
        // >=2 = sideload merge).
        let mut entries: FxHashSet<(i32, i32)> = FxHashSet::default();
        entries.extend(inserter_drop.iter().copied());
        entries.extend(splitter_output.iter().copied());
        for &t in belt_dir_map.keys() {
            if splitter_tiles.contains(&t) {
                continue;
            }
            if predecessors(t).len() != 1 {
                entries.insert(t);
            }
        }

        // EXIT_SET: inserter pickups, splitter inputs, dead ends (no forward
        // step), and tiles whose forward step lands on an entry (this tile's
        // own run ends by merging into the new run starting there).
        let mut exits: FxHashSet<(i32, i32)> = FxHashSet::default();
        exits.extend(inserter_pickup.iter().copied());
        exits.extend(splitter_input.iter().copied());
        for &t in belt_dir_map.keys() {
            if splitter_tiles.contains(&t) {
                continue;
            }
            match forward(t) {
                None => {
                    exits.insert(t);
                }
                Some((next, _)) if entries.contains(&next) => {
                    exits.insert(t);
                }
                _ => {}
            }
        }

        let mut sorted_entries: Vec<(i32, i32)> = entries
            .iter()
            .copied()
            .filter(|t| !splitter_tiles.contains(t))
            .collect();
        sorted_entries.sort();

        let mut runs = Vec::with_capacity(sorted_entries.len());
        for start in sorted_entries {
            let mut cur = start;
            let mut actual_length: i64 = 1;
            // Cycle guard: every non-anchor tile has exactly one predecessor by
            // construction, so a single walk can only revisit a tile if the
            // layout contains a true belt loop (already a separate hard error
            // from `belt_structural::check_belt_loops`) — stop rather than spin.
            let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
            visited.insert(cur);
            while !exits.contains(&cur) {
                match forward(cur) {
                    Some((next, dist)) if !visited.contains(&next) => {
                        actual_length += dist;
                        cur = next;
                        visited.insert(cur);
                    }
                    _ => break,
                }
            }
            let direct_distance =
                (cur.0 - start.0).abs() as i64 + (cur.1 - start.1).abs() as i64;
            runs.push(BeltRun { entry: start, exit: cur, actual_length, direct_distance });
        }
        runs
    }
}

// ---------------------------------------------------------------------------
// The check
// ---------------------------------------------------------------------------

/// Ratio floor for `belt-detour`: a run's `actual_length` must be at least
/// this many times its `direct_distance` to be flagged.
///
/// Calibrated against the 2026-08-01 corpus survey
/// (`scratchpad/belt_detour/survey.json`, 35 fixtures across the tier +
/// stress e2e corpus, 5543 runs measured — see the PR description for the
/// full distribution). The two floors are deliberately paired, not used
/// alone:
///
/// - `efficiency >= 2.0` alone fires on 344/5543 runs (6.2%) — but the
///   corpus's excess distribution is extremely tight (p50 = p90 = p95 =
///   1 tile; p99 ≈ 2 tiles; max = 45), so almost every one of those 344 is
///   a short belt clearing 2x on a single forced tile (a 2-tile run for a
///   1-tile gap) — exactly the "routine forced detour" the brief warns
///   against, not pathology.
/// - `excess >= 8` alone fires on 22/5543 runs (0.40%), but 13 of those are
///   LONG, mostly-direct belts (large `direct_distance`, low ratio) whose
///   absolute excess is just accumulated minor routing overhead — not a
///   doubled-back path either.
/// - The intersection — both floors — fires on exactly 9/5543 runs
///   (0.16%). All 9 are `advanced-circuit` layouts (`Pooled` and
///   `PartitionedDecomposed` strategies, at 1s/4s/5s/20s rates, both
///   from-plates and from-ore), several sharing near-identical entry/exit
///   coordinates across independently-generated fixtures — a real,
///   reproducible detour in how that recipe's rows route their last
///   segment, not noise (tracked in `docs/status.md` "Open tracking
///   issues", not yet root-caused). Ratios in this set range 2.0-6.25 and
///   excess 9-21 tiles; the next-worst runs sit at ratio <= 3.33 with
///   excess <= 7 or ratio 2.0 with excess <= 3 — a floor anywhere in
///   roughly [2.0, 3.17) paired with excess in [8, 9] selects the same 9
///   runs, which is why round numbers are used rather than the cluster's
///   exact boundary.
///
/// 0.16% is tighter than the brief's rough "worst few percent" prior, but
/// the brief explicitly defers to calibration from real data over that
/// prior, and a looser pair (e.g. ratio >= 1.5) would mostly re-admit the
/// short/long noise described above rather than surface more genuine
/// detours — the corpus simply doesn't have a "few percent" of pathological
/// runs, it has a clean 0.1%-ish tail concentrated in one recipe family.
/// See the PR body for the full top-20 list.
///
/// MIGRATION NOTE (RFC-065 Phase 1 slice 2, 2026-08-06): the figures
/// above describe the 2026-08-01 survey under the TILE-WALK
/// decomposition. The graph decomposition heals that walk's phantom run
/// cuts (620 fewer boundaries corpus-wide), and the SAME floors then
/// select the adjudicated set recorded in the RFC's decision log: four
/// additional true positives surfaced (three more of the same AC family,
/// plus kovarex's catalyst return line) and one phantom-bounded artifact
/// retired. The floors themselves are unchanged — the historical figures
/// stand as the record of that survey, and
/// `belt_detour_migration_differential` (e2e) pins the adjudicated
/// verdict drift exactly.
pub const DETOUR_RATIO_THRESHOLD: f64 = 2.0;

/// Absolute excess floor (tiles) for `belt-detour`, paired with
/// [`DETOUR_RATIO_THRESHOLD`]. Same calibration source and reasoning.
pub const DETOUR_EXCESS_TILES: i64 = 8;

/// Flags belt/underground runs that travel far more than the straight-line
/// separation between where they start and where they end — belts doubled
/// back on themselves, or routed the long way around when a short path
/// existed. One positioned `Warning` per offending run
/// (`docs/validator-reporting.md` rule 1): never a count folded into a
/// message.
///
/// Diagnostic only — thresholds are calibrated to the tail of the observed
/// distribution (see [`DETOUR_RATIO_THRESHOLD`]), not a hard physical
/// limit, so this never promotes to `Severity::Error`.
pub fn check_belt_detour(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues: Vec<ValidationIssue> = measure_belt_runs(layout)
        .into_iter()
        .filter(|r| r.efficiency() >= DETOUR_RATIO_THRESHOLD && r.excess() >= DETOUR_EXCESS_TILES)
        .map(|r| {
            let (mx, my) = r.midpoint();
            ValidationIssue::with_pos(
                Severity::Warning,
                "belt-detour",
                format!(
                    "belt run {} tiles for a {}-tile separation ({:.1}x) — ({},{}) to ({},{})",
                    r.actual_length,
                    r.direct_distance,
                    r.efficiency(),
                    r.entry.0,
                    r.entry.1,
                    r.exit.0,
                    r.exit.1,
                ),
                mx,
                my,
            )
        })
        .collect();
    issues.sort_by_key(|i| (i.x, i.y));
    issues
}

#[cfg(test)]
mod tests {
    use super::reference::measure_belt_runs_tilewalk;
    use super::*;
    use crate::models::{EntityDirection, PlacedEntity};

    fn belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".to_string(),
            x,
            y,
            direction: dir,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        }
    }

    fn inserter(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity { name: "inserter".to_string(), x, y, direction: dir, ..Default::default() }
    }

    fn ug(x: i32, y: i32, dir: EntityDirection, io: &str) -> PlacedEntity {
        PlacedEntity {
            name: "underground-belt".to_string(),
            x,
            y,
            direction: dir,
            io_type: Some(io.to_string()),
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        }
    }

    fn splitter(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "splitter".to_string(),
            x,
            y,
            direction: dir,
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        }
    }

    /// A straight belt run: inserter drop at (0,0), 5 tiles east, inserter
    /// pickup at (4,0). `actual_length` == `direct_distance` == 5,
    /// efficiency 1.0 — must never fire.
    #[test]
    fn straight_run_does_not_fire() {
        use EntityDirection::East;
        let mut entities = vec![inserter(-1, 0, East)];
        for x in 0..5 {
            entities.push(belt(x, 0, East));
        }
        entities.push(inserter(5, 0, East));
        let layout = LayoutResult { entities, width: 10, height: 3, ..Default::default() };

        let runs = measure_belt_runs(&layout);
        let run = runs
            .iter()
            .find(|r| r.entry == (0, 0))
            .unwrap_or_else(|| panic!("expected a run starting at (0,0): {runs:#?}"));
        assert_eq!(run.exit, (4, 0));
        assert_eq!(run.actual_length, 5);
        assert_eq!(run.direct_distance, 4);
        assert!(run.efficiency() < DETOUR_RATIO_THRESHOLD, "{run:?}");

        assert!(
            check_belt_detour(&layout).is_empty(),
            "a direct belt run must not fire the detour check"
        );
    }

    /// A belt that runs east 10 tiles, turns south one row, comes almost
    /// all the way back west, then turns south again into a pickup right
    /// next to where it started — a deliberately pathological doubled-back
    /// path. `direct_distance` (entry to exit) is tiny; `actual_length` is
    /// the full walked path — ratio and excess both blow past threshold.
    #[test]
    fn doubled_back_run_fires_with_correct_numbers() {
        use EntityDirection::{East, South, West};
        let mut entities = vec![inserter(-1, 0, East)];
        // Outbound leg: (0,0)..=(9,0) facing East, then turn south at x=10.
        for x in 0..10 {
            entities.push(belt(x, 0, East));
        }
        entities.push(belt(10, 0, South));
        // Return leg, one row down: (10,1)..=(2,1) facing West, then turn
        // south at x=1, ending the belt chain at (1,2).
        for x in (2..=10).rev() {
            entities.push(belt(x, 1, West));
        }
        entities.push(belt(1, 1, South));
        entities.push(belt(1, 2, South));
        // Pickup right next to the true start — small Manhattan gap, huge
        // walked distance.
        entities.push(inserter(1, 3, South));

        let layout = LayoutResult { entities, width: 15, height: 5, ..Default::default() };
        let runs = measure_belt_runs(&layout);
        let run = runs
            .iter()
            .find(|r| r.entry == (0, 0))
            .unwrap_or_else(|| panic!("expected a run starting at (0,0): {runs:#?}"));
        assert_eq!(run.exit, (1, 2));
        // 10 tiles row 0 (x=0..=9) + (10,0) + (10,1)..=(2,1) [9 tiles] +
        // (1,1) + (1,2) = 10 + 1 + 9 + 1 + 1 = 22 tiles traversed.
        assert_eq!(run.actual_length, 22);
        assert_eq!(run.direct_distance, 3); // |1-0| + |2-0|
        assert!(
            run.efficiency() >= DETOUR_RATIO_THRESHOLD && run.excess() >= DETOUR_EXCESS_TILES,
            "expected this pathological path to clear both thresholds: {run:?}"
        );

        let issues = check_belt_detour(&layout);
        assert_eq!(issues.len(), 1, "expected exactly one belt-detour issue: {issues:#?}");
        assert_eq!(issues[0].category, "belt-detour");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].x.is_some() && issues[0].y.is_some());
        assert!(
            issues[0].message.contains(&format!("{}", run.actual_length)),
            "message should carry actual_length: {}",
            issues[0].message
        );
        assert!(
            issues[0].message.contains(&format!("{}", run.direct_distance)),
            "message should carry direct_distance: {}",
            issues[0].message
        );
    }

    /// Splitters terminate runs on both sides — a run never crosses one,
    /// and the splitter's own footprint tiles never appear as an entry or
    /// exit.
    #[test]
    fn splitter_terminates_runs_on_both_sides() {
        use EntityDirection::East;
        let mut entities = vec![inserter(-1, 0, East)];
        for x in 0..3 {
            entities.push(belt(x, 0, East));
        }
        entities.push(splitter(3, 0, East));
        for x in 4..8 {
            entities.push(belt(x, 0, East));
        }
        entities.push(inserter(8, 0, East));

        let layout = LayoutResult { entities, width: 12, height: 3, ..Default::default() };
        let runs = measure_belt_runs(&layout);

        let splitter_tiles: rustc_hash::FxHashSet<(i32, i32)> =
            [(3, 0), (3, 1)].into_iter().collect();
        for r in &runs {
            assert!(!splitter_tiles.contains(&r.entry), "{r:?}");
            assert!(!splitter_tiles.contains(&r.exit), "{r:?}");
        }

        let before = runs.iter().find(|r| r.entry == (0, 0)).expect("run before splitter");
        assert_eq!(before.exit, (2, 0));
        let after = runs.iter().find(|r| r.entry == (4, 0)).expect("run after splitter");
        assert_eq!(after.exit, (7, 0));
    }

    /// Consumer-level pin for the RFC-065 Phase 1 pairing tightening
    /// (PR #574 bot round 3 asked for one at a `build_ug_pairs` consumer,
    /// not just the primitive): a cross-TIER entrance/exit on one axis is
    /// no longer a pair, so the run SEVERS at the entrance instead of
    /// jumping the span — two runs where the old direction-only pairing
    /// walked one.
    #[test]
    fn mixed_tier_underground_severs_the_run() {
        use EntityDirection::East;
        let mut entities = vec![inserter(-1, 0, East), belt(0, 0, East)];
        entities.push(ug(1, 0, East, "input"));
        entities.push(PlacedEntity {
            name: "fast-underground-belt".to_string(),
            x: 5,
            y: 0,
            direction: East,
            io_type: Some("output".to_string()),
            carries: Some("iron-plate".to_string()),
            ..Default::default()
        });
        entities.push(belt(6, 0, East));
        entities.push(inserter(7, 0, East));

        let layout = LayoutResult { entities, width: 10, height: 3, ..Default::default() };
        let runs = measure_belt_runs(&layout);
        let first = runs.iter().find(|r| r.entry == (0, 0)).expect("run from the feeder");
        assert_eq!(first.exit, (1, 0), "run must die at the unpaired entrance: {runs:#?}");
        assert!(
            runs.iter().any(|r| r.entry == (5, 0)),
            "the orphan exit must start its own run: {runs:#?}"
        );
    }

    /// An underground pair's span counts as the tiles it spans, not just
    /// the two entities.
    #[test]
    fn underground_pair_counts_spanned_tiles() {
        use EntityDirection::East;
        let mut entities = vec![inserter(-1, 0, East)];
        entities.push(belt(0, 0, East));
        entities.push(ug(1, 0, East, "input"));
        entities.push(ug(5, 0, East, "output"));
        entities.push(belt(6, 0, East));
        entities.push(inserter(7, 0, East));

        let layout = LayoutResult { entities, width: 10, height: 3, ..Default::default() };
        let runs = measure_belt_runs(&layout);
        let run = runs.iter().find(|r| r.entry == (0, 0)).expect("expected a run at (0,0)");
        assert_eq!(run.exit, (6, 0));
        // (0,0)->(1,0) = 1, UG (1,0)->(5,0) = 4, (5,0)->(6,0) = 1 => 6 tiles
        // traversed from entry inclusive: 1 (start) + 1 + 4 + 1 = 7.
        assert_eq!(run.actual_length, 7);
        assert_eq!(run.direct_distance, 6);
    }

    // -- RFC-065 Phase 1 slice-2 divergence pins (D1-D4). Each asserts the
    // -- graph-derived behavior AND the tile-walk oracle's old behavior, so
    // -- the exact semantic tightening stays visible. All four geometries
    // -- are engine-unreachable or already Error-class (see module doc).

    /// D1: a head-on contact carries no flow edge — the run stops before
    /// the facing belt. The tile-walk walked straight through it (head-on
    /// layouts are `check_belt_junctions` Errors regardless).
    #[test]
    fn d1_headon_stops_the_run_at_the_conflict() {
        use EntityDirection::{East, West};
        let layout = LayoutResult {
            entities: vec![belt(0, 0, East), belt(1, 0, East), belt(2, 0, West)],
            width: 5,
            height: 3,
            ..Default::default()
        };

        let runs = measure_belt_runs(&layout);
        let run = runs.iter().find(|r| r.entry == (0, 0)).expect("run at (0,0)");
        assert_eq!(run.exit, (1, 0), "run must stop before the head-on partner: {runs:#?}");
        assert_eq!(run.actual_length, 2);
        assert!(
            runs.iter().any(|r| r.entry == (2, 0) && r.exit == (2, 0)),
            "the facing belt is its own dead-end run: {runs:#?}"
        );

        // The oracle mishandled the head-on twice over: the facing belt
        // counted as a PREDECESSOR of (1,0) (a phantom "merge" that cut the
        // feeder's run at (0,0)), and the run from that fake entry then
        // walked THROUGH the head-on onto the facing belt.
        let old = measure_belt_runs_tilewalk(&layout);
        let old_feeder = old.iter().find(|r| r.entry == (0, 0)).expect("oracle run at (0,0)");
        assert_eq!(old_feeder.exit, (0, 0), "oracle cut the feeder at the phantom merge: {old:#?}");
        assert!(
            old.iter().any(|r| r.entry == (1, 0) && r.exit == (2, 0)),
            "oracle walked through the head-on: {old:#?}"
        );
    }

    /// D2: nothing surface-feeds a UG exit (rear feed stalls in game), so
    /// the span run flows through the exit uncut and the rear feeder dead-
    /// ends. The tile-walk counted the rear feeder as a merge predecessor,
    /// promoting the exit to an entry and chopping the span run there.
    #[test]
    fn d2_rear_fed_ug_exit_severs_the_feeder_not_the_span() {
        use EntityDirection::East;
        let layout = LayoutResult {
            entities: vec![
                ug(-1, 0, East, "input"),
                belt(0, 0, East), // on the span, rear-feeding the exit
                ug(1, 0, East, "output"),
                belt(2, 0, East),
            ],
            width: 6,
            height: 3,
            ..Default::default()
        };

        let runs = measure_belt_runs(&layout);
        let span = runs.iter().find(|r| r.entry == (-1, 0)).expect("span run");
        assert_eq!(
            (span.exit, span.actual_length),
            ((2, 0), 4),
            "span run must flow through the exit uncut: {runs:#?}"
        );
        let feeder = runs.iter().find(|r| r.entry == (0, 0)).expect("rear feeder run");
        assert_eq!(
            (feeder.exit, feeder.actual_length),
            ((0, 0), 1),
            "rear feeder must dead-end before the exit: {runs:#?}"
        );

        // The oracle promoted the rear-fed exit to an entry (2 predecessors)
        // and cut the span run at the entrance — the behavior D2 tightens.
        let old = measure_belt_runs_tilewalk(&layout);
        assert!(
            old.iter().any(|r| r.entry == (1, 0)),
            "oracle treated the rear-fed exit as a run entry: {old:#?}"
        );
        let old_span = old.iter().find(|r| r.entry == (-1, 0)).expect("oracle span run");
        assert_eq!(old_span.actual_length, 1, "oracle chopped the span run: {old:#?}");
    }

    /// D3: splitter anchors are orientation-true. A belt line passing
    /// perpendicular directly behind a splitter is NOT a splitter input —
    /// the run measures through continuously. The tile-walk marked the
    /// behind-tile an exit and ORPHANED the downstream tail from
    /// measurement entirely (no entry ever started there) — the latent
    /// hole this migration closes.
    #[test]
    fn d3_perpendicular_belt_behind_splitter_measures_through() {
        use EntityDirection::{East, North};
        // Splitter at (3,0)/(3,1) facing East; a northbound belt line runs
        // through its behind-tiles (2,1) and (2,0).
        let layout = LayoutResult {
            entities: vec![
                splitter(3, 0, East),
                belt(2, 3, North),
                belt(2, 2, North),
                belt(2, 1, North),
                belt(2, 0, North),
                belt(2, -1, North),
            ],
            width: 8,
            height: 6,
            ..Default::default()
        };

        let runs = measure_belt_runs(&layout);
        let run = runs.iter().find(|r| r.entry == (2, 3)).expect("northbound run");
        assert_eq!(
            (run.exit, run.actual_length),
            ((2, -1), 5),
            "perpendicular line must measure through uncut: {runs:#?}"
        );

        // The oracle cut the run at the splitter's behind-tile and never
        // measured the tail — no run reaches (2,-1).
        let old = measure_belt_runs_tilewalk(&layout);
        let old_run = old.iter().find(|r| r.entry == (2, 3)).expect("oracle northbound run");
        assert_eq!(old_run.exit, (2, 1), "oracle cut at the behind-tile: {old:#?}");
        assert!(
            !old.iter().any(|r| r.exit == (2, -1)),
            "oracle orphaned the downstream tail — no run measured it: {old:#?}"
        );
    }

    /// D4: an io-untagged underground is `NodeClass::Other` (K65-1: no
    /// role-guessing for hand-built entities) — runs sever at it. The
    /// tile-walk's dir map carried it as a plain tile and walked through.
    #[test]
    fn d4_untagged_underground_is_invisible() {
        use EntityDirection::East;
        let layout = LayoutResult {
            entities: vec![
                belt(0, 0, East),
                PlacedEntity {
                    name: "underground-belt".to_string(),
                    x: 1,
                    y: 0,
                    direction: East,
                    io_type: None,
                    carries: Some("iron-plate".to_string()),
                    ..Default::default()
                },
                belt(2, 0, East),
            ],
            width: 5,
            height: 3,
            ..Default::default()
        };

        let runs = measure_belt_runs(&layout);
        let run = runs.iter().find(|r| r.entry == (0, 0)).expect("feeder run");
        assert_eq!((run.exit, run.actual_length), ((0, 0), 1), "{runs:#?}");
        assert!(
            runs.iter().any(|r| r.entry == (2, 0)),
            "the belt past the untagged UG starts its own run: {runs:#?}"
        );

        let old = measure_belt_runs_tilewalk(&layout);
        let old_run = old.iter().find(|r| r.entry == (0, 0)).expect("oracle feeder run");
        assert_eq!(
            (old_run.exit, old_run.actual_length),
            ((2, 0), 3),
            "oracle walked the untagged UG as a plain tile: {old:#?}"
        );
    }

    /// D5: a UG entrance "points at" its ahead tile in the dir map but
    /// feeds it nothing (items go underground) — the tile-walk counted it
    /// as a predecessor anyway. On weave geometry (a crossing lane running
    /// through the tile directly ahead of an entrance — standard bus
    /// tap-off routing) that phantom predecessor made the crossing tile
    /// look like a 2-source merge and CUT the crossing lane mid-run. The
    /// graph has no such edge, so the lane measures continuously. Unlike
    /// D1-D4 this geometry IS engine-reachable — the corpus differential
    /// (`belt_detour_migration_differential` in e2e) quantifies it.
    #[test]
    fn d5_entrance_ahead_tile_is_not_phantom_fed() {
        use EntityDirection::{East, North};
        // Eastbound span (0,0)->(3,0); a northbound crossing lane passes
        // through (1,0) — the tile directly ahead of the entrance.
        let layout = LayoutResult {
            entities: vec![
                ug(0, 0, East, "input"),
                ug(3, 0, East, "output"),
                belt(4, 0, East),
                belt(1, 2, North),
                belt(1, 1, North),
                belt(1, 0, North),
                belt(1, -1, North),
            ],
            width: 8,
            height: 6,
            ..Default::default()
        };

        let runs = measure_belt_runs(&layout);
        let crossing = runs.iter().find(|r| r.entry == (1, 2)).expect("crossing run");
        assert_eq!(
            (crossing.exit, crossing.actual_length),
            ((1, -1), 4),
            "crossing lane must measure continuously through the weave: {runs:#?}"
        );

        // The oracle's phantom entrance-predecessor made (1,0) a 2-source
        // "merge" and cut the crossing lane there.
        let old = measure_belt_runs_tilewalk(&layout);
        assert!(
            old.iter().any(|r| r.entry == (1, 0)),
            "oracle cut the crossing lane at the entrance's ahead tile: {old:#?}"
        );
        let old_crossing = old.iter().find(|r| r.entry == (1, 2)).expect("oracle crossing run");
        assert_eq!(old_crossing.exit, (1, 1), "oracle run ends at the phantom merge: {old:#?}");
    }

    /// Clean-geometry differential: on game-legal flow (the only geometry
    /// the engine emits) the graph decomposition and the tile-walk oracle
    /// agree exactly — a composite fixture with drops, a paired span, an
    /// aligned splitter, sideload merges, and pickups.
    #[test]
    fn clean_geometry_matches_the_tilewalk_oracle_exactly() {
        use EntityDirection::{East, North};
        let mut entities = vec![inserter(-1, 0, East)];
        for x in 0..3 {
            entities.push(belt(x, 0, East));
        }
        entities.push(ug(3, 0, East, "input"));
        entities.push(ug(6, 0, East, "output"));
        entities.push(belt(7, 0, East));
        entities.push(splitter(8, 0, East));
        for x in 9..12 {
            entities.push(belt(x, 0, East));
        }
        entities.push(inserter(12, 0, East));
        // Second lane out of the splitter's lower tile, with a northbound
        // feeder sideloading into it mid-run.
        entities.push(belt(9, 1, East));
        entities.push(belt(10, 1, East));
        entities.push(belt(10, 2, North));
        entities.push(belt(11, 1, East));
        entities.push(inserter(12, 1, East));

        let layout = LayoutResult { entities, width: 16, height: 5, ..Default::default() };
        let new = measure_belt_runs(&layout);
        let old = measure_belt_runs_tilewalk(&layout);
        assert_eq!(new, old, "clean geometry must decompose identically");
        assert!(new.len() >= 4, "composite fixture should produce several runs: {new:#?}");
    }
}
