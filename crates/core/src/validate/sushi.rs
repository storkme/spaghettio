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

    // Group sushi belt tiles by segment id; record the belt entity (tier).
    let mut seg_belt: FxHashMap<&str, &str> = FxHashMap::default();
    for e in &layout.entities {
        if !is_belt_entity(&e.name) {
            continue;
        }
        if let Some(seg) = e.segment_id.as_deref() {
            if seg.contains(SUSHI_MARKER) {
                seg_belt.entry(seg).or_insert(e.name.as_str());
            }
        }
    }

    for (seg, belt) in &seg_belt {
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
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
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
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
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
