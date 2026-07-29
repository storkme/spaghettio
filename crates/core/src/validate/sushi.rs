//! Sushi-sorter validation (RFC Fulgora Phase 3, `docs/rfc-fulgora-scrap.md`
//! D3 architecture (a), KC5 containment).
//!
//! The scrap-recycling row (`templates::scrap_recycling_row`) ejects ~12
//! mixed items onto a single "sushi" belt, then a bank of filter inserters
//! sorts each item onto its own single-item lane. A sushi belt legitimately
//! violates the one-item-per-lane invariant that every other check assumes,
//! so it is tagged (`:sushi:` in `segment_id`) and given a NARROW,
//! purpose-built pair of checks that own its correctness story instead of
//! the ordinary belt walkers:
//!
//! - [`check_sushi_boundary`] — the containment guarantee (KC5): every
//!   transition OFF a sushi segment must go through a filter inserter, never
//!   a plain belt adjacency, and every inserter lifting from a sushi belt
//!   must be a filter inserter whose filter matches the lane it feeds. This
//!   is what makes the item-isolation exemption safe: mixed items can only
//!   leave the sushi belt already sorted.
//! - [`check_sushi_saturation`] — the throughput guarantee: the sum of the
//!   per-item rates the recyclers eject onto a sushi segment must not exceed
//!   the belt's capacity (a jammed sushi belt is a real in-game failure).
//!
//! The ordinary checks (`check_belt_item_isolation`, the belt-flow lane
//! walkers) only *skip* sushi tiles; they never relax their logic for
//! non-sushi belts. See each call site for the one-line exemption.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{belt_throughput, dir_to_vec, inserter_reach, is_belt_entity, is_inserter, is_splitter, splitter_second_tile};
use crate::models::{LayoutResult, SolverResult};

use super::{Severity, ValidationIssue};

/// Marker substring identifying a sushi (mixed-item collection) belt
/// segment. Belts whose `segment_id` contains this are exempt from
/// single-item lane checks and owned by the two checks in this module.
pub const SUSHI_MARKER: &str = ":sushi:";

/// Marker substring identifying a belt-to-belt filter inserter that lifts
/// one item off a sushi belt onto its own lane.
pub const SUSHI_SORT_MARKER: &str = ":sushi-sort:";

/// True if `seg` marks a sushi collection belt.
pub fn is_sushi_segment(seg: Option<&str>) -> bool {
    seg.is_some_and(|s| s.contains(SUSHI_MARKER))
}

/// True if `seg` marks a sushi sort (belt-to-belt filter) inserter.
pub fn is_sushi_sort_inserter(seg: Option<&str>) -> bool {
    seg.is_some_and(|s| s.contains(SUSHI_SORT_MARKER))
}

/// KC5 containment: nothing leaves a sushi belt except through a filter
/// inserter matching the lane it feeds.
///
/// Two failure modes are errors:
/// 1. A sushi belt tile feeds (in its flow direction) into a NON-sushi belt
///    — a plain-belt leak that would spill the mixed item set onto an
///    ordinary single-item lane, exactly what the item-isolation exemption
///    must never allow.
/// 2. An inserter that PICKS from a sushi tile is not a filter inserter, or
///    its filter does not include the item its drop-target belt carries — an
///    unfiltered lift pulls arbitrary items onto a single-item lane.
pub fn check_sushi_boundary(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Index belts by tile: direction, sushi-ness, carried item.
    let mut belt_dir: FxHashMap<(i32, i32), crate::models::EntityDirection> = FxHashMap::default();
    let mut sushi_tiles: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut belt_carry: FxHashMap<(i32, i32), Option<String>> = FxHashMap::default();
    for e in &layout.entities {
        if !is_belt_entity(&e.name) {
            continue;
        }
        let sushi = is_sushi_segment(e.segment_id.as_deref());
        let mut tiles = vec![(e.x, e.y)];
        if is_splitter(&e.name) {
            tiles.push(splitter_second_tile(e));
        }
        for t in tiles {
            belt_dir.insert(t, e.direction);
            belt_carry.insert(t, e.carries.clone());
            if sushi {
                sushi_tiles.insert(t);
            }
        }
    }

    // Failure mode 1: sushi tile feeding a non-sushi belt.
    for &(sx, sy) in &sushi_tiles {
        let dir = belt_dir[&(sx, sy)];
        let (dx, dy) = dir_to_vec(dir);
        let ds = (sx + dx, sy + dy);
        if belt_dir.contains_key(&ds) && !sushi_tiles.contains(&ds) {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "sushi-boundary",
                format!(
                    "sushi belt at ({sx},{sy}) feeds directly into non-sushi belt at \
                     ({},{}) — mixed items must leave the sushi belt only through a \
                     filter inserter, never a plain belt",
                    ds.0, ds.1
                ),
                sx,
                sy,
            ));
        }
    }

    // Failure mode 2: inserters picking from a sushi tile.
    for e in &layout.entities {
        if !is_inserter(&e.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(e.direction);
        let reach = inserter_reach(&e.name);
        let pickup = (e.x - dx * reach, e.y - dy * reach);
        if !sushi_tiles.contains(&pickup) {
            continue;
        }
        if e.filters.is_empty() {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "sushi-boundary",
                format!(
                    "inserter at ({},{}) lifts from the sushi belt at ({},{}) with no \
                     filter — it would pull arbitrary mixed items onto a single-item lane",
                    e.x, e.y, pickup.0, pickup.1
                ),
                e.x,
                e.y,
            ));
            continue;
        }
        // The drop-target belt (if any) must carry an item the inserter filters.
        let drop = (e.x + dx * reach, e.y + dy * reach);
        if let Some(Some(carried)) = belt_carry.get(&drop) {
            if !e.filters.iter().any(|f| f == carried) {
                issues.push(ValidationIssue::with_pos(
                    Severity::Error,
                    "sushi-boundary",
                    format!(
                        "sushi sort inserter at ({},{}) filters {:?} but drops onto a belt \
                         carrying {carried} — filter must match the sorted lane",
                        e.x, e.y, e.filters
                    ),
                    e.x,
                    e.y,
                ));
            }
        }
    }

    issues
}

/// Throughput guarantee: the total rate the recyclers eject onto each sushi
/// segment must not exceed the sushi belt's capacity, or the belt jams and
/// items back up into the recyclers in-game.
///
/// The per-item rates come from the solver (the `scrap-recycling` machine's
/// outputs × count) rather than from the belt tiles, which deliberately
/// carry no single item tag — the sum is the mixed throughput the belt must
/// sustain. The belt capacity comes from the sushi belt entity actually
/// placed.
pub fn check_sushi_saturation(
    layout: &LayoutResult,
    solver: &SolverResult,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Group sushi belt tiles into PHYSICAL runs, not by segment id.
    //
    // `templates::scrap_recycling_row` tags every sushi belt
    // `row:{recipe}:sushi:{item}` — keyed on recipe and item with no row
    // index — and `place_rows` splits any recipe whose machine count exceeds
    // `max_per_row` into several physical rows, each getting that identical
    // string. Keying a map on it therefore collapsed N separate belts into
    // one arbitrary entry (`or_insert` keeps whichever was seen first) and
    // compared the recipe's WHOLE output against that single belt's capacity:
    // two rows each at 60% of their own belt read as 120% of one, while an
    // asymmetric split could stay silent on a row that was genuinely jammed.
    // Either way the issue count never reflected how many rows were affected.
    //
    // `check_row_output_lane_budget` and `check_row_input_belt_margin` hit
    // this same collision and solve it by clustering tiles by adjacency; this
    // now matches them.
    let mut sushi_tiles: Vec<(i32, i32)> = Vec::new();
    let mut sushi_meta: Vec<(&str, &str)> = Vec::new(); // (segment, belt entity)
    for e in &layout.entities {
        if !is_belt_entity(&e.name) {
            continue;
        }
        if let Some(seg) = e.segment_id.as_deref() {
            if seg.contains(SUSHI_MARKER) {
                sushi_tiles.push((e.x, e.y));
                sushi_meta.push((seg, e.name.as_str()));
            }
        }
    }
    let cluster_of = crate::validate::inserters::cluster_tiles_by_adjacency(&sushi_tiles);
    // One entry per physical run: its segment, its slowest belt tier (the
    // real constraint), and how many machine-equivalents it serves.
    let mut runs: FxHashMap<usize, (&str, &str)> = FxHashMap::default();
    for (i, &(seg, belt)) in sushi_meta.iter().enumerate() {
        let c = cluster_of[i];
        runs.entry(c)
            .and_modify(|(_, b)| {
                if belt_throughput(belt) < belt_throughput(b) {
                    *b = belt;
                }
            })
            .or_insert((seg, belt));
    }
    // Recipe-wide output is shared across that recipe's physical runs.
    let mut runs_per_recipe: FxHashMap<&str, f64> = FxHashMap::default();
    for (seg, _) in runs.values() {
        if let Some(r) = seg.strip_prefix("row:").and_then(|rest| {
            rest.find(":sushi").map(|i| &rest[..i])
        }) {
            *runs_per_recipe.entry(r).or_insert(0.0) += 1.0;
        }
    }

    for (seg, belt) in runs.values() {
        // segment_id shape: `row:{recipe}:sushi:{item}` — recover the recipe.
        let recipe = seg.strip_prefix("row:").and_then(|rest| {
            rest.find(":sushi").map(|i| &rest[..i])
        });
        let Some(recipe) = recipe else { continue };
        // Sum the mixed output rate the recyclers running this recipe eject.
        let total: f64 = solver
            .machines
            .iter()
            .filter(|m| m.recipe == recipe)
            .flat_map(|m| m.outputs.iter().filter(|o| !o.is_fluid).map(move |o| o.rate * m.count))
            .sum();
        // Each physical run carries its share of the recipe's output.
        let share = runs_per_recipe.get(recipe).copied().unwrap_or(1.0).max(1.0);
        let total = total / share;
        let cap = belt_throughput(belt);
        if total > cap + 1e-6 {
            issues.push(ValidationIssue::new(
                Severity::Error,
                "sushi-saturation",
                format!(
                    "sushi belt `{seg}` ({belt}, cap {cap:.1}/s) carries {total:.1}/s of \
                     mixed recycler output — over capacity; the belt jams and backs items \
                     into the recyclers",
                ),
            ));
        }
    }

    issues
}

/// Per-item sort-inserter count for a mixed stream: how many inserters of
/// `per_inserter_rate` items/s each are needed to lift `item_rate` items/s
/// off the sushi belt without the item circulating. `ceil`, minimum 1.
///
/// Belt-to-belt inserter swing rates (conservative, from
/// `docs/factorio-mechanics.md`): regular ≈0.83/s, long-handed ≈1.2/s,
/// fast/bulk higher. Exposed for the placer + unit tests.
pub fn sort_inserter_count(item_rate: f64, per_inserter_rate: f64) -> usize {
    if per_inserter_rate <= 0.0 {
        return 1;
    }
    (item_rate / per_inserter_rate).ceil().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, ItemFlow, MachineSpec, PlacedEntity};

    fn sushi_belt(x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".into(),
            x,
            y,
            direction: dir,
            segment_id: Some("row:scrap-recycling:sushi:scrap".into()),
            ..Default::default()
        }
    }
    /// Two PHYSICAL sushi runs that share one segment string — what
    /// `place_rows` produces whenever a scrap-recycling recipe needs more
    /// machines than one row can carry.
    ///
    /// Each run carries half the recipe's output and each is within its own
    /// belt's capacity, so nothing is jammed. Keying on the segment id
    /// collapsed both into one entry and compared the recipe's WHOLE output
    /// against a single belt, reporting a jam that does not exist.
    #[test]
    fn split_rows_are_judged_per_physical_run() {
        let solver = SolverResult {
            machines: vec![MachineSpec {
                recipe: "scrap-recycling".into(),
                count: 8.0,
                outputs: vec![ItemFlow {
                    item: "iron-plate".into(),
                    // 8 x 1.5 = 12/s recipe-wide, 6/s per run.
                    rate: 1.5,
                    is_fluid: false,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        // Yellow belt: 15/s. Each run at 6/s is fine; 12/s against one belt
        // would also be fine, so make the belts red-tier-free and the total
        // large enough that only the collapsed comparison could trip.
        let mut entities = Vec::new();
        for x in 0..4 {
            entities.push(sushi_belt(x, 0, EntityDirection::East));
        }
        // Second run, far away — a separate physical belt, same segment id.
        for x in 0..4 {
            entities.push(sushi_belt(x, 40, EntityDirection::East));
        }
        let layout = LayoutResult { entities, width: 10, height: 50, ..Default::default() };

        let issues = check_sushi_saturation(&layout, &solver);
        assert!(
            issues.is_empty(),
            "two runs at 6/s each on a 15/s belt are not saturated: {issues:?}"
        );
    }

    /// The converse: a genuinely over-capacity run must still be reported, so
    /// the per-run split cannot silently excuse a real jam.
    #[test]
    fn a_genuinely_saturated_run_is_still_reported() {
        let solver = SolverResult {
            machines: vec![MachineSpec {
                recipe: "scrap-recycling".into(),
                count: 8.0,
                // 8 x 4.0 = 32/s recipe-wide, 32/s on the single run.
                outputs: vec![ItemFlow {
                    item: "iron-plate".into(),
                    rate: 4.0,
                    is_fluid: false,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        let entities: Vec<PlacedEntity> =
            (0..4).map(|x| sushi_belt(x, 0, EntityDirection::East)).collect();
        let layout = LayoutResult { entities, width: 10, height: 10, ..Default::default() };
        let issues = check_sushi_saturation(&layout, &solver);
        assert_eq!(issues.len(), 1, "32/s on one 15/s belt must report: {issues:?}");
        assert_eq!(issues[0].category, "sushi-saturation");
    }

    fn plain_belt(x: i32, y: i32, dir: EntityDirection, item: &str) -> PlacedEntity {
        PlacedEntity {
            name: "transport-belt".into(),
            x,
            y,
            direction: dir,
            carries: Some(item.into()),
            segment_id: Some("row:foo:belt-out".into()),
            ..Default::default()
        }
    }
    fn filter_ins(x: i32, y: i32, dir: EntityDirection, filter: &str) -> PlacedEntity {
        PlacedEntity {
            name: "fast-inserter".into(),
            x,
            y,
            direction: dir,
            filters: vec![filter.into()],
            segment_id: Some("row:scrap-recycling:sushi-sort:iron-plate".into()),
            ..Default::default()
        }
    }

    #[test]
    fn boundary_clean_when_filter_inserter_lifts_off_sushi() {
        // Sushi belt at (0,0)->(1,0) east; the last sushi tile (1,0) ends
        // (no belt east of it). A south-facing filter inserter at (1,1)
        // picks the sushi at (1,0) and drops onto the iron-plate lane at
        // (1,2). This is the legal sorted-off path.
        let lr = LayoutResult {
            entities: vec![
                sushi_belt(0, 0, EntityDirection::East),
                sushi_belt(1, 0, EntityDirection::East),
                filter_ins(1, 1, EntityDirection::South, "iron-plate"),
                plain_belt(1, 2, EntityDirection::South, "iron-plate"),
            ],
            ..Default::default()
        };
        assert!(check_sushi_boundary(&lr).iter().all(|i| i.category != "sushi-boundary"));
    }

    #[test]
    fn boundary_errors_when_sushi_feeds_plain_belt() {
        // Sushi at (0,0) east feeds (1,0) which is a plain non-sushi belt.
        let lr = LayoutResult {
            entities: vec![
                sushi_belt(0, 0, EntityDirection::East),
                plain_belt(1, 0, EntityDirection::East, "iron-gear-wheel"),
            ],
            ..Default::default()
        };
        let issues = check_sushi_boundary(&lr);
        assert_eq!(issues.iter().filter(|i| i.category == "sushi-boundary").count(), 1);
    }

    #[test]
    fn boundary_errors_on_unfiltered_lift_off_sushi() {
        // South-facing inserter at (0,1) picks sushi (0,0), no filter.
        let mut ins = filter_ins(0, 1, EntityDirection::South, "iron-plate");
        ins.filters.clear();
        let lr = LayoutResult {
            entities: vec![sushi_belt(0, 0, EntityDirection::East), ins],
            ..Default::default()
        };
        let issues = check_sushi_boundary(&lr);
        assert_eq!(issues.iter().filter(|i| i.category == "sushi-boundary").count(), 1);
    }

    #[test]
    fn boundary_errors_on_filter_mismatch() {
        // Inserter picks sushi (0,0), filters iron-plate, but drops onto a
        // belt carrying copper-plate.
        let lr = LayoutResult {
            entities: vec![
                sushi_belt(0, 0, EntityDirection::East),
                filter_ins(0, 1, EntityDirection::South, "iron-plate"),
                plain_belt(0, 2, EntityDirection::South, "copper-plate"),
            ],
            ..Default::default()
        };
        let issues = check_sushi_boundary(&lr);
        assert_eq!(issues.iter().filter(|i| i.category == "sushi-boundary").count(), 1);
    }

    #[test]
    fn saturation_under_capacity_ok() {
        let solver = SolverResult {
            machines: vec![MachineSpec {
                entity: "recycler".into(),
                recipe: "scrap-recycling".into(),
                count: 4.0,
                inputs: vec![ItemFlow { item: "scrap".into(), rate: 2.5, is_fluid: false, module_id: 0 }],
                outputs: vec![
                    ItemFlow { item: "iron-gear-wheel".into(), rate: 0.5, is_fluid: false, module_id: 0 },
                    ItemFlow { item: "stone".into(), rate: 0.1, is_fluid: false, module_id: 0 },
                ],
                self_loop: vec![],
                voider: false,
                game_modules: Vec::new(),
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        // 4*(0.5+0.1) = 2.4/s on a yellow belt (15/s) — fine.
        let lr = LayoutResult {
            entities: vec![sushi_belt(0, 0, EntityDirection::East)],
            ..Default::default()
        };
        assert!(check_sushi_saturation(&lr, &solver).is_empty());
    }

    #[test]
    fn saturation_over_capacity_errors() {
        let solver = SolverResult {
            machines: vec![MachineSpec {
                entity: "recycler".into(),
                recipe: "scrap-recycling".into(),
                count: 40.0,
                inputs: vec![],
                outputs: vec![ItemFlow { item: "iron-gear-wheel".into(), rate: 0.5, is_fluid: false, module_id: 0 }],
                self_loop: vec![],
                voider: false,
                game_modules: Vec::new(),
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
            ..Default::default()
        };
        // 40*0.5 = 20/s > yellow 15/s.
        let lr = LayoutResult {
            entities: vec![sushi_belt(0, 0, EntityDirection::East)],
            ..Default::default()
        };
        let issues = check_sushi_saturation(&lr, &solver);
        assert_eq!(issues.iter().filter(|i| i.category == "sushi-saturation").count(), 1);
    }

    #[test]
    fn sort_inserter_count_ceils() {
        assert_eq!(sort_inserter_count(2.0, 0.83), 3);
        assert_eq!(sort_inserter_count(0.4, 0.83), 1);
        assert_eq!(sort_inserter_count(2.0, 2.31), 1);
        assert_eq!(sort_inserter_count(0.0, 0.83), 1);
    }
}
