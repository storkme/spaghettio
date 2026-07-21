//! Inserter chain validity and direction checks.
//!
//! Port of `check_inserter_chains` and `check_inserter_direction` from
//! `src/validate.py`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{
    dir_to_vec, fluid_only_recipes, inserter_reach, inserter_throughput, is_inserter,
    is_machine_entity, machine_dims, machine_tiles, recycler_eject_tile,
    stack_inserter_belt_hand, stack_inserter_swings, utilization_for,
};
use crate::models::{LayoutResult, PlacedEntity, SolverResult};

/// RFC-046: throughput an inserter delivers when dropping onto a **belt**.
/// At the layout's belt stack size S > 1, a stack inserter's belt drops
/// use the `swings × belt-hand` decomposition (BS3: hand 6 rounded down
/// to a multiple of S — including the real 9.6/s dip at S=4). At S ≤ 1,
/// and for every other inserter type, this is the flat I8 constant —
/// bit-identical to pre-RFC behavior (kill 1). Machine drops never use
/// this: they are exact-hand (BS2), always the flat constant.
fn belt_drop_throughput(ins: &PlacedEntity, stacking: u8) -> f64 {
    let quality = ins.quality.unwrap_or_default();
    if stacking > 1 && ins.name == "stack-inserter" {
        stack_inserter_swings(quality) * stack_inserter_belt_hand(stacking)
    } else {
        inserter_throughput(&ins.name, quality)
    }
}

use super::{Severity, ValidationIssue};

// ── Helper: build machine tile set ───────────────────────────────────────────

fn build_machine_tile_set(layout: &LayoutResult) -> FxHashSet<(i32, i32)> {
    let mut tiles = FxHashSet::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            tiles.extend(machine_tiles(e.x, e.y, w, h));
        }
    }
    tiles
}

// ── Helper: shared exemption resolution ──────────────────────────────────────
//
// `resolve_row_spec` (the effective_rows position-based spec attribution)
// now lives in `super` (`validate::mod`) so `belt_flow` and `belt_structural`
// can share it too — see its doc comment there for the partition-sibling
// rationale.

/// Recyclers eject directly onto a belt (no output inserter is placed or
/// wanted), so their output side is exempt from throughput checks. Shared by
/// `check_inserter_throughput` and `check_inserter_item_throughput`.
fn is_recycler_direct_eject(e: &crate::models::PlacedEntity) -> bool {
    recycler_eject_tile(&e.name, e.x, e.y, e.direction).is_some()
}

// ── check_inserter_chains ─────────────────────────────────────────────────────

/// Check that every machine with solid I/O has at least one adjacent inserter.
///
/// Machines whose recipe only has fluid inputs/outputs are skipped.
pub fn check_inserter_chains(
    layout: &LayoutResult,
    solver_result: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let fluid_only = fluid_only_recipes(solver_result);

    // Separate inserter position sets by reach
    let mut short_inserter_positions: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut long_inserter_positions: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if is_inserter(&e.name) {
            if e.name == "long-handed-inserter" {
                long_inserter_positions.insert((e.x, e.y));
            } else {
                short_inserter_positions.insert((e.x, e.y));
            }
        }
    }

    // Check each machine exactly once
    let mut checked_machines: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        if !checked_machines.insert((e.x, e.y)) {
            continue;
        }

        // Skip fluid-only machines
        if let Some(recipe) = &e.recipe {
            if fluid_only.contains(recipe.as_str()) {
                continue;
            }
        }

        let (mw, mh) = machine_dims(&e.name);
        let (mw, mh) = (mw as i32, mh as i32);
        let mut has_inserter = false;

        // Short inserters: 1 tile from border → dx in [-1, mw], dy in [-1, mh]
        'outer_short: for dx in -1..=mw {
            for dy in -1..=mh {
                if short_inserter_positions.contains(&(e.x + dx, e.y + dy)) {
                    has_inserter = true;
                    break 'outer_short;
                }
            }
        }

        // Long-handed inserters: 2 tiles from border → dx in [-2, mw+1], dy in [-2, mh+1]
        if !has_inserter {
            'outer_long: for dx in -2..=(mw + 1) {
                for dy in -2..=(mh + 1) {
                    if long_inserter_positions.contains(&(e.x + dx, e.y + dy)) {
                        has_inserter = true;
                        break 'outer_long;
                    }
                }
            }
        }

        if !has_inserter {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "inserter",
                format!("{} at ({},{}): no inserter adjacent", e.name, e.x, e.y),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

// ── check_inserter_direction ──────────────────────────────────────────────────

/// Check that each inserter has its drop or pickup side pointing at a machine.
///
/// An inserter facing parallel to the nearest machine border won't transfer
/// items; that is reported as an error.
pub fn check_inserter_direction(layout: &LayoutResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let machine_tiles_set = build_machine_tile_set(layout);

    for e in &layout.entities {
        if !is_inserter(&e.name) {
            continue;
        }
        // Sushi sort inserters (RFC Fulgora Phase 3) are belt-to-belt by
        // design — they lift one item off a sushi belt onto its own lane
        // and touch no machine. The sushi boundary check owns their
        // correctness (`validate::sushi::check_sushi_boundary`).
        if super::sushi::is_sushi_sort_inserter(e.segment_id.as_deref()) {
            continue;
        }

        let (dx, dy) = dir_to_vec(e.direction);
        let (odx, ody) = (-dx, -dy);

        let reach = inserter_reach(&e.name);

        let drop_pos = (e.x + dx * reach, e.y + dy * reach);
        let pickup_pos = (e.x + odx * reach, e.y + ody * reach);

        let drop_touches = machine_tiles_set.contains(&drop_pos);
        let pickup_touches = machine_tiles_set.contains(&pickup_pos);

        if !drop_touches && !pickup_touches {
            issues.push(ValidationIssue::with_pos(
                Severity::Error,
                "inserter-direction",
                format!(
                    "inserter at ({},{}) facing {:?}: neither drop nor pickup side touches a machine",
                    e.x, e.y, e.direction
                ),
                e.x,
                e.y,
            ));
        }
    }

    issues
}

// ── check_inserter_throughput ─────────────────────────────────────────────────

/// Per-machine-side inserter throughput check (RFC `rfc-lane-demand-flow.md`
/// Phase 1, Component 2).
///
/// Every bus template feeds and drains a machine with **one regular inserter
/// per side** (~0.84/s, `docs/factorio-mechanics.md` table I8), but planned
/// per-machine rates routinely exceed that (e.g. a gear machine wants
/// 3.0 plates/s in and 1.5 gears/s out). The machine is then inserter-bound
/// regardless of belt delivery — a real cap that no other check sees. This
/// emits one warning per deficient machine side (input feeds / output
/// extraction) when the inserters on that side cannot collectively move the
/// side's required rate.
///
/// Required rates are utilization-scaled the same way `check_input_rate_delivery`
/// scales demand (ce732d9): the layout places `ceil(count)` physical machines
/// each running at `count/ceil(count)`, so a fractional-count spec's per-machine
/// rate is scaled down accordingly. Only SOLID inputs/outputs count — fluids
/// arrive by pipe, not inserter.
///
/// Exemptions: recyclers eject their output directly onto a belt with no output
/// inserter (`common::recycler_eject_tile`, the same knowledge
/// `check_output_belt_coverage` keys on), so their output side is skipped.
/// `:sushi-sort:` belt-to-belt inserters touch no machine tile, so the
/// input/output classification below never counts them — no special case
/// needed.
pub fn check_inserter_throughput(
    layout: &LayoutResult,
    solver_result: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let sr = match solver_result {
        Some(s) => s,
        None => return Vec::new(),
    };

    let machine_tiles_set = build_machine_tile_set(layout);
    let machine_by_tile = machine_origin_by_tile(layout);

    // Sum inserter throughput on each machine's input side (inserters that
    // drop INTO the machine) and output side (inserters that pick FROM it).
    let mut input_avail: FxHashMap<(i32, i32), f64> = FxHashMap::default();
    let mut input_count: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    let mut output_avail: FxHashMap<(i32, i32), f64> = FxHashMap::default();
    let mut output_count: FxHashMap<(i32, i32), usize> = FxHashMap::default();

    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);
        let rate = inserter_throughput(&ins.name, ins.quality.unwrap_or_default());

        if let Some(&mpos) = machine_by_tile.get(&drop_pos) {
            *input_avail.entry(mpos).or_insert(0.0) += rate;
            *input_count.entry(mpos).or_insert(0) += 1;
        }
        if let Some(&mpos) = machine_by_tile.get(&pickup_pos) {
            // A drop_pos inside the same machine would make this a
            // machine-internal inserter — ignore those (they don't extract
            // to a belt). Only count picks that actually leave the machine.
            if !machine_tiles_set.contains(&drop_pos) {
                // Extraction drops onto a belt → belt-drop rating (RFC-046).
                let rate = belt_drop_throughput(ins, layout.stacking);
                *output_avail.entry(mpos).or_insert(0.0) += rate;
                *output_count.entry(mpos).or_insert(0) += 1;
            }
        }
    }

    let recipe_to_spec: FxHashMap<&str, &crate::models::MachineSpec> =
        sr.machines.iter().map(|s| (s.recipe.as_str(), s)).collect();

    let mut issues = Vec::new();
    let mut checked: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        let mpos = (e.x, e.y);
        if !checked.insert(mpos) {
            continue;
        }
        let recipe = match e.recipe.as_deref() {
            Some(r) => r,
            None => continue,
        };
        let fallback_spec = match recipe_to_spec.get(recipe) {
            Some(s) => *s,
            None => continue,
        };
        // Attribution: prefer the exact sibling `MachineSpec` the layout
        // pipeline actually placed at this machine's row — see
        // `resolve_row_spec`'s doc comment for the partition-sibling
        // rationale (`docs/rfc-inserter-sizing.md` Phase 1 finding).
        let spec = super::resolve_row_spec(layout, recipe, e.y, fallback_spec);

        // Utilization scaling: the same convention check_input_rate_delivery
        // uses — a spec placed as ceil(count) physical machines runs each at
        // count/ceil(count).
        let utilization = utilization_for(spec);

        let required_in: f64 = spec
            .inputs
            .iter()
            .filter(|f| !f.is_fluid)
            .map(|f| f.rate * utilization)
            .sum();
        let required_out: f64 = spec
            .outputs
            .iter()
            .filter(|f| !f.is_fluid)
            .map(|f| f.rate * utilization)
            .sum();

        // Input side.
        if required_in > 0.0 {
            let avail = input_avail.get(&mpos).copied().unwrap_or(0.0);
            if avail < required_in - 0.02 {
                let n = input_count.get(&mpos).copied().unwrap_or(0);
                issues.push(ValidationIssue::with_pos(
                    Severity::Warning,
                    "inserter-throughput",
                    format!(
                        "{} at ({},{}): {} input inserter{} move {:.2}/s but machine needs \
                         {:.2}/s in — inserter-bound",
                        e.name, e.x, e.y, n, if n == 1 { "" } else { "s" }, avail, required_in
                    ),
                    e.x,
                    e.y,
                ));
            }
        }

        // Output side. Recyclers eject directly onto a belt (no output
        // inserter is placed or wanted), so their output side is exempt.
        let recycler_direct_eject = is_recycler_direct_eject(e);
        if required_out > 0.0 && !recycler_direct_eject {
            let avail = output_avail.get(&mpos).copied().unwrap_or(0.0);
            if avail < required_out - 0.02 {
                let n = output_count.get(&mpos).copied().unwrap_or(0);
                issues.push(ValidationIssue::with_pos(
                    Severity::Warning,
                    "inserter-throughput",
                    format!(
                        "{} at ({},{}): {} output inserter{} move {:.2}/s but machine needs \
                         {:.2}/s out — inserter-bound",
                        e.name, e.x, e.y, n, if n == 1 { "" } else { "s" }, avail, required_out
                    ),
                    e.x,
                    e.y,
                ));
            }
        }
    }

    issues
}

// ── check_inserter_item_throughput ────────────────────────────────────────────

/// Per-machine, per-solid-item inserter throughput check
/// (`docs/rfc-inserter-sizing.md` Phase 2's delta-review blocker).
///
/// [`check_inserter_throughput`] is item-blind: it sums a machine side's
/// inserters into one aggregate `avail` and compares that against one
/// aggregate `required`, never checking which item each inserter actually
/// carries. That was harmless while every side had at most one solid item
/// (`single_input_row`, Phase 1's structural guard), but multi-item sides
/// (dual/triple/quad-input rows, and the near/far ingredient-assignment
/// lever) make item identity load-bearing: a near-slot inserter's surplus
/// capacity can arithmetically "cover" a far-slot item's deficit in the
/// aggregate sum while the far item is, in reality, starving — a reach-1
/// inserter physically cannot pick up a far-belt item. This check closes
/// that gap: for every machine, for every solid input item (and every solid
/// output item), it sums only the inserters whose [`PlacedEntity::carries`]
/// matches that item and compares against that item's own utilization-
/// scaled per-machine rate.
///
/// Attribution mirrors [`check_inserter_throughput`] exactly: prefer the
/// row-positioned sibling spec from `layout.effective_rows`
/// (`docs/rfc-inserter-sizing.md` Phase 1 finding — partition siblings share
/// a recipe name but carry different utilizations), falling back to the
/// recipe-keyed spec when no row attribution is available (test scaffolding,
/// spaghetti-style layouts). Recyclers are exempt on the output side (direct
/// belt ejection, no output inserter — same knowledge
/// `check_output_belt_coverage`/`check_inserter_throughput` key on).
/// `:sushi-sort:` inserters are belt-to-belt by construction (RFC Fulgora
/// Phase 3) — neither their drop nor pickup side ever lands on a machine
/// tile, so the same drop/pickup-vs-machine-tile matching this check shares
/// with `check_inserter_throughput` excludes them with no special case,
/// exactly as documented there.
///
/// Known ceiling (recorded, not fixed here): self-loop machines' recirculated
/// "major" item (and, in the has-minor shape, its recirculated portion) lives
/// in `MachineSpec::self_loop`, never in `MachineSpec::inputs` — this check
/// only iterates `inputs`/`outputs`, so major-item demand is structurally
/// invisible to it, matching the same gap in `check_inserter_throughput`.
pub fn check_inserter_item_throughput(
    layout: &LayoutResult,
    solver_result: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let sr = match solver_result {
        Some(s) => s,
        None => return Vec::new(),
    };

    let machine_tiles_set = build_machine_tile_set(layout);
    let machine_by_tile = machine_origin_by_tile(layout);

    // Per (machine origin, item) avail, split by side — unlike
    // `check_inserter_throughput`'s single f64 accumulator per side, this
    // is keyed on the inserter's `carries` attribution.
    let mut input_avail: FxHashMap<((i32, i32), &str), f64> = FxHashMap::default();
    let mut output_avail: FxHashMap<((i32, i32), &str), f64> = FxHashMap::default();

    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let Some(item) = ins.carries.as_deref() else {
            continue;
        };
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let drop_pos = (ins.x + dx * reach, ins.y + dy * reach);
        let pickup_pos = (ins.x - dx * reach, ins.y - dy * reach);
        let rate = inserter_throughput(&ins.name, ins.quality.unwrap_or_default());

        if let Some(&mpos) = machine_by_tile.get(&drop_pos) {
            *input_avail.entry((mpos, item)).or_insert(0.0) += rate;
        }
        if let Some(&mpos) = machine_by_tile.get(&pickup_pos) {
            if !machine_tiles_set.contains(&drop_pos) {
                // Extraction drops onto a belt → belt-drop rating (RFC-046).
                let rate = belt_drop_throughput(ins, layout.stacking);
                *output_avail.entry((mpos, item)).or_insert(0.0) += rate;
            }
        }
    }

    let recipe_to_spec: FxHashMap<&str, &crate::models::MachineSpec> =
        sr.machines.iter().map(|s| (s.recipe.as_str(), s)).collect();

    let mut issues = Vec::new();
    let mut checked: FxHashSet<(i32, i32)> = FxHashSet::default();
    for e in &layout.entities {
        if !is_machine_entity(&e.name) {
            continue;
        }
        let mpos = (e.x, e.y);
        if !checked.insert(mpos) {
            continue;
        }
        let recipe = match e.recipe.as_deref() {
            Some(r) => r,
            None => continue,
        };
        let fallback_spec = match recipe_to_spec.get(recipe) {
            Some(s) => *s,
            None => continue,
        };
        // Same `effective_rows` position-based resolution as
        // `check_inserter_throughput` — see `resolve_row_spec`'s doc
        // comment for the partition-sibling rationale.
        let spec = super::resolve_row_spec(layout, recipe, e.y, fallback_spec);
        let utilization = utilization_for(spec);

        for f in spec.inputs.iter().filter(|f| !f.is_fluid) {
            let required = f.rate * utilization;
            if required <= 0.0 {
                continue;
            }
            let avail = input_avail.get(&(mpos, f.item.as_str())).copied().unwrap_or(0.0);
            if avail < required - 0.02 {
                issues.push(ValidationIssue::with_pos(
                    Severity::Warning,
                    "inserter-item-throughput",
                    format!(
                        "{} at ({},{}): item {} input inserters move {:.2}/s but machine needs \
                         {:.2}/s in — item-attribution-bound",
                        e.name, e.x, e.y, f.item, avail, required
                    ),
                    e.x,
                    e.y,
                )
                .with_detail(avail, required));
            }
        }

        let recycler_direct_eject = is_recycler_direct_eject(e);
        if !recycler_direct_eject {
            for f in spec.outputs.iter().filter(|f| !f.is_fluid) {
                let required = f.rate * utilization;
                if required <= 0.0 {
                    continue;
                }
                let avail = output_avail.get(&(mpos, f.item.as_str())).copied().unwrap_or(0.0);
                if avail < required - 0.02 {
                    issues.push(ValidationIssue::with_pos(
                        Severity::Warning,
                        "inserter-item-throughput",
                        format!(
                            "{} at ({},{}): item {} output inserters move {:.2}/s but machine needs \
                             {:.2}/s out — item-attribution-bound",
                            e.name, e.x, e.y, f.item, avail, required
                        ),
                        e.x,
                        e.y,
                    )
                    .with_detail(avail, required));
                }
            }
        }
    }

    issues
}

/// Map each machine tile → the machine's origin `(x, y)`.
fn machine_origin_by_tile(layout: &LayoutResult) -> FxHashMap<(i32, i32), (i32, i32)> {
    let mut by_tile = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            let (w, h) = machine_dims(&e.name);
            for t in machine_tiles(e.x, e.y, w, h) {
                by_tile.insert(t, (e.x, e.y));
            }
        }
    }
    by_tile
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntityDirection, ItemFlow, MachineSpec, PlacedEntity, SolverResult};

    fn solid_flow(item: &str) -> ItemFlow {
        ItemFlow { item: item.to_string(), rate: 1.0, is_fluid: false, module_id: 0 }
    }

    fn fluid_flow(item: &str) -> ItemFlow {
        ItemFlow { item: item.to_string(), rate: 1.0, is_fluid: true, module_id: 0 }
    }

    // ── belt_drop_throughput (RFC-046) ───────────────────────────────────────

    #[test]
    fn belt_drop_throughput_flat_at_s1_decomposed_above() {
        use crate::common::QualityTier;
        let stack = PlacedEntity { name: "stack-inserter".into(), ..Default::default() };
        let fast = PlacedEntity { name: "fast-inserter".into(), ..Default::default() };
        // S ≤ 1 is the flat I8 constant, bit-identical to pre-RFC (kill 1)
        // — including 0, the derived-Default sentinel on hand-built layouts.
        assert_eq!(
            belt_drop_throughput(&stack, 0),
            inserter_throughput("stack-inserter", QualityTier::Normal)
        );
        assert_eq!(belt_drop_throughput(&stack, 1), 12.0);
        // S=2,3: swings × hand 6 = 14.4/s; S=4: hand rounds down to 4 →
        // the real 9.6/s dip (BS3).
        assert!((belt_drop_throughput(&stack, 2) - 14.4).abs() < 1e-9);
        assert!((belt_drop_throughput(&stack, 3) - 14.4).abs() < 1e-9);
        assert!((belt_drop_throughput(&stack, 4) - 9.6).abs() < 1e-9);
        // Non-stack inserters never stack (BS5) — flat at any S.
        assert_eq!(
            belt_drop_throughput(&fast, 4),
            inserter_throughput("fast-inserter", QualityTier::Normal)
        );
        // Quality scales swings only (BS7): legendary ×2.5 at S=4 → 24/s.
        let legendary = PlacedEntity {
            name: "stack-inserter".into(),
            quality: Some(QualityTier::Legendary),
            ..Default::default()
        };
        assert!((belt_drop_throughput(&legendary, 4) - 24.0).abs() < 1e-9);
    }

    // ── check_inserter_direction ─────────────────────────────────────────────

    #[test]
    fn inserter_facing_machine_ok() {
        // 3x3 machine at (0,0); inserter at (1,-1) facing SOUTH → drops into (1,0)
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn inserter_facing_away_from_machine_ok() {
        // Inserter at (1,-1) facing NORTH → picks from machine at (1,0)
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::North,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn inserter_facing_parallel_error() {
        // Inserter at (1,-1) facing EAST → parallel to top border, neither side hits machine
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::East,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].category, "inserter-direction");
    }

    #[test]
    fn inserter_not_near_machine_error() {
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 10,
                    y: 10,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 20,
            height: 20,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn inserter_facing_electric_furnace_ok() {
        // 3x3 electric-furnace at (0,0); inserter at (1,-1) facing SOUTH
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "electric-furnace".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-plate".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn long_handed_inserter_direction_ok() {
        // long-handed-inserter at (1,-2) facing SOUTH reaches (1,0) which is inside machine
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "long-handed-inserter".into(),
                    x: 1,
                    y: -2,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 0, "long-handed should reach 2 tiles");
    }

    // ── check_inserter_chains ────────────────────────────────────────────────

    #[test]
    fn machine_without_inserter_error() {
        // Machine with no inserters nearby
        let lr = LayoutResult {
            entities: vec![PlacedEntity {
                name: "assembling-machine-1".into(),
                x: 0,
                y: 0,
                recipe: Some("iron-gear-wheel".into()),
                direction: EntityDirection::North,
                ..Default::default()
            }],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_chains(&lr, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "inserter");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn machine_with_adjacent_inserter_ok() {
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_chains(&lr, None);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn fluid_only_machine_skipped() {
        // Machine whose recipe only has fluid I/O should not require an inserter
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "oil-refinery".into(),
                recipe: "basic-oil-processing".into(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: 1.0,
                inputs: vec![fluid_flow("crude-oil")],
                outputs: vec![fluid_flow("petroleum-gas")],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let lr = LayoutResult {
            entities: vec![PlacedEntity {
                name: "oil-refinery".into(),
                x: 0,
                y: 0,
                recipe: Some("basic-oil-processing".into()),
                direction: EntityDirection::North,
                ..Default::default()
            }],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_chains(&lr, Some(&sr));
        assert_eq!(issues.len(), 0, "fluid-only machine should be skipped");
    }

    #[test]
    fn mixed_solid_fluid_machine_needs_inserter() {
        // Machine with one solid input → still needs an inserter
        let sr = SolverResult {
            machines: vec![MachineSpec {
                entity: "chemical-plant".into(),
                recipe: "plastic-bar".into(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: 1.0,
                inputs: vec![solid_flow("coal"), fluid_flow("petroleum-gas")],
                outputs: vec![solid_flow("plastic-bar")],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };
        let lr = LayoutResult {
            entities: vec![PlacedEntity {
                name: "chemical-plant".into(),
                x: 0,
                y: 0,
                recipe: Some("plastic-bar".into()),
                direction: EntityDirection::North,
                ..Default::default()
            }],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_chains(&lr, Some(&sr));
        assert_eq!(issues.len(), 1, "mixed recipe still needs an inserter");
    }

    #[test]
    fn long_handed_inserter_satisfies_chain_check() {
        // long-handed inserter 2 tiles from border counts
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                // 2 tiles above the top border (y = -2)
                PlacedEntity {
                    name: "long-handed-inserter".into(),
                    x: 1,
                    y: -2,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_chains(&lr, None);
        assert_eq!(issues.len(), 0, "long-handed inserter at -2 should satisfy chain");
    }

    #[test]
    fn reversed_inserter_direction_produces_issue() {
        // Done-when criterion: reversed inserter direction → expected issue
        // Inserter at (1,-1) facing WEST → drop at (0,-1), pickup at (2,-1), neither in machine
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::West,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_direction(&lr);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(!errors.is_empty(), "reversed inserter should produce at least one error");
        assert_eq!(errors[0].category, "inserter-direction");
        assert_eq!(errors[0].x, Some(1));
        assert_eq!(errors[0].y, Some(-1));
    }

    #[test]
    fn valid_layout_no_direction_issues() {
        // Done-when criterion: valid layout produces no issues
        // Machine at (0,0), inserter at (1,-1) facing South → drop at (1,0) which is inside machine
        let lr = LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                PlacedEntity {
                    name: "inserter".into(),
                    x: 1,
                    y: -1,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let dir_issues = check_inserter_direction(&lr);
        let chain_issues = check_inserter_chains(&lr, None);
        assert_eq!(dir_issues.len(), 0, "valid layout should produce no direction issues");
        assert_eq!(chain_issues.len(), 0, "valid layout should produce no chain issues");
    }

    #[test]
    fn no_solver_result_treats_all_machines_as_needing_inserters() {
        // Without solver_result, no recipes are skipped → machine needs inserter
        let lr = LayoutResult {
            entities: vec![PlacedEntity {
                name: "oil-refinery".into(),
                x: 0,
                y: 0,
                recipe: Some("basic-oil-processing".into()),
                direction: EntityDirection::North,
                ..Default::default()
            }],
            width: 10,
            height: 10,
            ..Default::default()
        };
        let issues = check_inserter_chains(&lr, None);
        // Without solver context, we can't know it's fluid-only → should flag
        assert_eq!(issues.len(), 1);
    }

    // ── check_inserter_throughput ─────────────────────────────────────────────

    fn gear_solver(input_rate: f64, output_rate: f64, count: f64) -> SolverResult {
        SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-1".into(),
                recipe: "iron-gear-wheel".into(),
                self_loop: vec![],
                voider: false,
                game_modules: Vec::new(),
                count,
                inputs: vec![ItemFlow {
                    item: "iron-plate".into(),
                    rate: input_rate,
                    is_fluid: false,
                    module_id: 0,
                }],
                outputs: vec![ItemFlow {
                    item: "iron-gear-wheel".into(),
                    rate: output_rate,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        }
    }

    /// 3x3 machine at (0,0) with one input inserter (drops into (1,0)) and one
    /// output inserter (picks from (1,2), drops to (1,4)).
    fn gear_machine_layout(input_inserter: &str, output_inserter: &str) -> LayoutResult {
        let input_reach = inserter_reach(input_inserter);
        LayoutResult {
            entities: vec![
                PlacedEntity {
                    name: "assembling-machine-1".into(),
                    x: 0,
                    y: 0,
                    recipe: Some("iron-gear-wheel".into()),
                    direction: EntityDirection::North,
                    ..Default::default()
                },
                // Input: faces South, reach tiles above the top border so the
                // drop lands on (1,0) inside the machine.
                PlacedEntity {
                    name: input_inserter.into(),
                    x: 1,
                    y: -input_reach,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
                // Output: faces South at (1,3); picks from (1,2) inside the
                // machine, drops to (1,4) outside.
                PlacedEntity {
                    name: output_inserter.into(),
                    x: 1,
                    y: 3,
                    direction: EntityDirection::South,
                    ..Default::default()
                },
            ],
            width: 20,
            height: 20,
            ..Default::default()
        }
    }

    fn throughput_warnings(issues: &[ValidationIssue]) -> Vec<&ValidationIssue> {
        issues
            .iter()
            .filter(|i| i.category == "inserter-throughput")
            .collect()
    }

    /// A 3x3 gear machine at `(0, y)` with a regular input inserter
    /// dropping into `(1, y)` and a regular output inserter picking from
    /// `(1, y+2)`.
    fn gear_machine_entities_at(y: i32) -> Vec<PlacedEntity> {
        vec![
            PlacedEntity {
                name: "assembling-machine-1".into(),
                x: 0,
                y,
                recipe: Some("iron-gear-wheel".into()),
                direction: EntityDirection::North,
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".into(),
                x: 1,
                y: y - 1,
                direction: EntityDirection::South,
                ..Default::default()
            },
            PlacedEntity {
                name: "inserter".into(),
                x: 1,
                y: y + 3,
                direction: EntityDirection::South,
                ..Default::default()
            },
        ]
    }

    /// A per-machine `iron-gear-wheel` `MachineSpec` sibling (as produced
    /// by `bus::partitioner::apply_partition_plan`, which varies `count`/
    /// `inputs`/`outputs` per sibling but keeps the recipe name shared).
    fn gear_sibling_spec(input_rate: f64, output_rate: f64, count: f64) -> MachineSpec {
        MachineSpec {
            entity: "assembling-machine-1".into(),
            recipe: "iron-gear-wheel".into(),
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
            count,
            inputs: vec![ItemFlow {
                item: "iron-plate".into(),
                rate: input_rate,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-gear-wheel".into(),
                rate: output_rate,
                is_fluid: false,
                module_id: 0,
            }],
        }
    }

    #[test]
    fn inserter_throughput_gear_machine_warns_both_sides() {
        // 3.0/s in via one regular inserter (0.84) → warns; 1.5/s out via one
        // regular inserter (0.84) → warns. Two warnings, one per side.
        let sr = gear_solver(3.0, 1.5, 1.0);
        let lr = gear_machine_layout("inserter", "inserter");
        let issues = check_inserter_throughput(&lr, Some(&sr));
        let warns = throughput_warnings(&issues);
        assert_eq!(warns.len(), 2, "both input and output sides deficient: {issues:?}");
        assert!(warns.iter().all(|w| w.severity == Severity::Warning));
        assert!(warns.iter().any(|w| w.message.contains("in —")));
        assert!(warns.iter().any(|w| w.message.contains("out —")));
    }

    #[test]
    fn inserter_throughput_within_capacity_no_warning() {
        // 0.5/s in and 0.5/s out, each below the 0.84/s regular-inserter cap.
        let sr = gear_solver(0.5, 0.5, 1.0);
        let lr = gear_machine_layout("inserter", "inserter");
        let issues = check_inserter_throughput(&lr, Some(&sr));
        assert!(throughput_warnings(&issues).is_empty(), "within cap: {issues:?}");
    }

    #[test]
    fn inserter_throughput_long_handed_credits_higher_rate() {
        // 1.0/s in sits between the regular (0.84) and long-handed (1.2) caps:
        // a regular inserter would warn, a long-handed one must not. Output at
        // 0.5/s via a regular inserter stays clean, isolating the input side.
        let sr = gear_solver(1.0, 0.5, 1.0);

        let regular = gear_machine_layout("inserter", "inserter");
        let regular_issues = check_inserter_throughput(&regular, Some(&sr));
        let regular_warns = throughput_warnings(&regular_issues);
        assert_eq!(regular_warns.len(), 1, "regular inserter under 1.0/s should warn");
        assert!(regular_warns[0].message.contains("in —"));

        let long = gear_machine_layout("long-handed-inserter", "inserter");
        let long_issues = check_inserter_throughput(&long, Some(&sr));
        let long_warns = throughput_warnings(&long_issues);
        assert!(
            long_warns.is_empty(),
            "long-handed (1.2/s) covers 1.0/s in and regular covers 0.5/s out: {long_warns:?}"
        );
    }

    #[test]
    fn inserter_throughput_utilization_scaled_no_warning() {
        // A half-count spec (count 0.5) runs its single physical machine at 50%
        // utilization, so a nominal 1.5/s in scales to 0.75/s — under the 0.84
        // regular cap — and 1.0/s out scales to 0.5/s. No warning despite the
        // nominal rates exceeding 0.84.
        let sr = gear_solver(1.5, 1.0, 0.5);
        let lr = gear_machine_layout("inserter", "inserter");
        let issues = check_inserter_throughput(&lr, Some(&sr));
        assert!(
            throughput_warnings(&issues).is_empty(),
            "utilization-scaled need under cap: {issues:?}"
        );
    }

    #[test]
    fn inserter_throughput_no_solver_is_noop() {
        let lr = gear_machine_layout("inserter", "inserter");
        assert!(check_inserter_throughput(&lr, None).is_empty());
    }

    // ── partition-sibling attribution (`EffectiveRow`) ─────────────────────

    /// `apply_partition_plan` splits one recipe into sibling `MachineSpec`s
    /// that share a recipe name but carry different per-machine rates.
    /// `sr.machines` here holds the single, collapsed/blended spec a
    /// recipe-name-keyed lookup would see (as `validate()` receives the
    /// pre-partition `SolverResult` — docs/rfc-inserter-sizing.md's Phase 1
    /// finding). Row A's true demand (0.5/s in, within a regular
    /// inserter's 0.84/s) is deliberately far below the blended spec's
    /// 2.0/s, and row B's true demand (3.0/s in) is deliberately far
    /// above it — so a name-keyed lookup landing on the blended spec for
    /// both rows could not produce "A clean, B warns" by coincidence.
    /// `LayoutResult::effective_rows` must resolve each row to its own
    /// true sibling by position for both outcomes to hold simultaneously.
    #[test]
    fn inserter_throughput_partition_siblings_disambiguated_by_row() {
        let blended_spec = gear_sibling_spec(2.0, 1.0, 3.0);
        let sr = SolverResult {
            machines: vec![blended_spec],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };

        let mut entities = gear_machine_entities_at(0); // row A: y in [0, 3)
        entities.extend(gear_machine_entities_at(10)); // row B: y in [10, 13)

        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            effective_rows: vec![
                crate::models::EffectiveRow {
                    y_start: 0,
                    y_end: 8,
                    spec: gear_sibling_spec(0.5, 0.5, 1.0),
                },
                crate::models::EffectiveRow {
                    y_start: 8,
                    y_end: 16,
                    // Output stays within the regular-inserter cap so the
                    // one expected warning isolates to the input side.
                    spec: gear_sibling_spec(3.0, 0.5, 1.0),
                },
            ],
            ..Default::default()
        };

        let issues = check_inserter_throughput(&lr, Some(&sr));
        let warns = throughput_warnings(&issues);
        assert_eq!(
            warns.len(),
            1,
            "row A (0.5/s, in-capacity) must stay clean, row B (3.0/s, over-capacity) must warn: {issues:?}"
        );
        assert!(warns[0].message.contains("(0,10)"), "the warning must land on row B's machine: {warns:?}");
        assert!(warns[0].message.contains("in —"));
    }

    /// A single, unpartitioned recipe: `effective_rows` carries exactly
    /// one row whose spec is identical to `sr.machines`' entry — the shape
    /// `layout_pass` produces whenever no partition sibling exists.
    /// Position-based attribution must resolve to the same spec the old
    /// recipe-name lookup would have, so the outcome is byte-identical to
    /// `inserter_throughput_gear_machine_warns_both_sides`.
    #[test]
    fn inserter_throughput_effective_rows_noop_for_unsplit_recipe() {
        let sr = gear_solver(3.0, 1.5, 1.0);
        let mut lr = gear_machine_layout("inserter", "inserter");
        lr.effective_rows = vec![crate::models::EffectiveRow {
            y_start: -1,
            y_end: 5,
            spec: sr.machines[0].clone(),
        }];

        let issues = check_inserter_throughput(&lr, Some(&sr));
        let warns = throughput_warnings(&issues);
        assert_eq!(warns.len(), 2, "unchanged from the no-effective_rows case: {issues:?}");
        assert!(warns.iter().any(|w| w.message.contains("in —")));
        assert!(warns.iter().any(|w| w.message.contains("out —")));
    }

    // ── check_inserter_item_throughput ─────────────────────────────────────

    fn item_throughput_warnings(issues: &[ValidationIssue]) -> Vec<&ValidationIssue> {
        issues
            .iter()
            .filter(|i| i.category == "inserter-item-throughput")
            .collect()
    }

    /// A 3x3 machine at `(0, y)` with a NEAR (reach-1) input inserter
    /// dropping into the machine's `(0, y)` corner tile, a FAR (reach-2)
    /// input inserter dropping into the `(2, y)` corner tile, and a
    /// regular output inserter picking from `(1, y+2)`. Mirrors the real
    /// dual_input_row shape closely enough for attribution testing: two
    /// distinct input items on distinct tiles, one item on the output.
    fn dual_input_machine_entities_at(
        y: i32,
        near_entity: &str,
        near_item: &str,
        far_entity: &str,
        far_item: &str,
        output_entity: &str,
        output_item: &str,
    ) -> Vec<PlacedEntity> {
        let near_reach = inserter_reach(near_entity);
        let far_reach = inserter_reach(far_entity);
        vec![
            PlacedEntity {
                name: "assembling-machine-2".into(),
                x: 0,
                y,
                recipe: Some("test-dual-recipe".into()),
                direction: EntityDirection::North,
                ..Default::default()
            },
            PlacedEntity {
                name: near_entity.into(),
                x: 0,
                y: y - near_reach,
                direction: EntityDirection::South,
                carries: Some(near_item.into()),
                ..Default::default()
            },
            PlacedEntity {
                name: far_entity.into(),
                x: 2,
                y: y - far_reach,
                direction: EntityDirection::South,
                carries: Some(far_item.into()),
                ..Default::default()
            },
            PlacedEntity {
                name: output_entity.into(),
                x: 1,
                y: y + 3,
                direction: EntityDirection::South,
                carries: Some(output_item.into()),
                ..Default::default()
            },
        ]
    }

    fn dual_input_spec(
        near_item: &str,
        near_rate: f64,
        far_item: &str,
        far_rate: f64,
        output_item: &str,
        output_rate: f64,
        count: f64,
    ) -> SolverResult {
        SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-2".into(),
                recipe: "test-dual-recipe".into(),
                self_loop: vec![],
                voider: false,
                game_modules: Vec::new(),
                count,
                inputs: vec![
                    ItemFlow { item: near_item.into(), rate: near_rate, is_fluid: false, module_id: 0 },
                    ItemFlow { item: far_item.into(), rate: far_rate, is_fluid: false, module_id: 0 },
                ],
                outputs: vec![ItemFlow {
                    item: output_item.into(),
                    rate: output_rate,
                    is_fluid: false,
                    module_id: 0,
                }],
            }],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        }
    }

    /// The RFC's canonical item-blindness failure: a near-slot inserter's
    /// spare capacity arithmetically "covers" a far-slot item's deficit in
    /// the AGGREGATE sum (`check_inserter_throughput` stays clean), while
    /// the far item is, in reality, starving — a reach-1 inserter cannot
    /// pick up a far-belt item no matter how much headroom it has. Numbers:
    /// avail = 0.84 (near, regular) + 1.2 (far, long-handed) = 2.04;
    /// required = 0.3 (near) + 1.5 (far) = 1.8 ≤ 2.04 → aggregate clean.
    /// But far's own avail (1.2) < far's own required (1.5) → per-item warn.
    #[test]
    fn inserter_item_throughput_masked_side_warns_per_item() {
        let sr = dual_input_spec("item-near", 0.3, "item-far", 1.5, "product", 0.5, 1.0);
        let entities =
            dual_input_machine_entities_at(0, "inserter", "item-near", "long-handed-inserter", "item-far", "inserter", "product");
        let lr = LayoutResult { entities, width: 20, height: 20, ..Default::default() };

        let agg_issues = check_inserter_throughput(&lr, Some(&sr));
        assert!(
            throughput_warnings(&agg_issues).is_empty(),
            "aggregate check must stay clean (2.04/s avail ≥ 1.8/s required): {agg_issues:?}"
        );

        let item_issues = check_inserter_item_throughput(&lr, Some(&sr));
        let warns = item_throughput_warnings(&item_issues);
        assert_eq!(warns.len(), 1, "only the far item should warn: {item_issues:?}");
        assert!(warns[0].message.contains("item-far"), "{:?}", warns[0].message);
        assert!(warns[0].message.contains("in —"));
    }

    #[test]
    fn inserter_item_throughput_clean_case_no_warnings() {
        // Both inputs comfortably within their own inserter's cap, output
        // likewise — no aggregate deficiency and no per-item deficiency.
        let sr = dual_input_spec("item-near", 0.3, "item-far", 1.0, "product", 0.4, 1.0);
        let entities =
            dual_input_machine_entities_at(0, "inserter", "item-near", "long-handed-inserter", "item-far", "inserter", "product");
        let lr = LayoutResult { entities, width: 20, height: 20, ..Default::default() };

        let issues = check_inserter_item_throughput(&lr, Some(&sr));
        assert!(item_throughput_warnings(&issues).is_empty(), "{issues:?}");
    }

    #[test]
    fn inserter_item_throughput_output_side_warns() {
        // Both inputs clean; output item demands more than the single
        // regular output inserter can move — must warn on the output side
        // only, isolating the branch from the input-side masking case.
        let sr = dual_input_spec("item-near", 0.3, "item-far", 1.0, "product", 5.0, 1.0);
        let entities =
            dual_input_machine_entities_at(0, "inserter", "item-near", "long-handed-inserter", "item-far", "inserter", "product");
        let lr = LayoutResult { entities, width: 20, height: 20, ..Default::default() };

        let issues = check_inserter_item_throughput(&lr, Some(&sr));
        let warns = item_throughput_warnings(&issues);
        assert_eq!(warns.len(), 1, "{issues:?}");
        assert!(warns[0].message.contains("product"));
        assert!(warns[0].message.contains("out —"));
        // RFC validation-explainability D1: structured pair mirrors the
        // check's own comparison — needed is the 5.0/s product demand, and
        // delivered is what the single regular inserter moves (< needed).
        let detail = warns[0]
            .detail
            .as_ref()
            .expect("inserter-item-throughput must carry IssueDetail");
        assert!((detail.needed - 5.0).abs() < 1e-9, "{detail:?}");
        assert!(detail.delivered < detail.needed, "{detail:?}");
    }

    /// Same partition-sibling scenario as
    /// `inserter_throughput_partition_siblings_disambiguated_by_row`, but
    /// for the per-item check: a blended `sr.machines` spec would produce
    /// the SAME (wrong) verdict for both rows, while `effective_rows`
    /// position-based resolution must disambiguate row A (clean, 0.5/s
    /// in, within a regular inserter's 0.84/s) from row B (warns, 3.0/s
    /// in, over cap) exactly as the aggregate check does.
    #[test]
    fn inserter_item_throughput_effective_rows_sibling_split() {
        let blended_spec = gear_sibling_spec(2.0, 1.0, 3.0);
        let sr = SolverResult {
            machines: vec![blended_spec],
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        };

        let mut entities = gear_machine_entities_at(0); // row A: y in [0, 3)
        entities.extend(gear_machine_entities_at(10)); // row B: y in [10, 13)

        let lr = LayoutResult {
            entities,
            width: 20,
            height: 20,
            effective_rows: vec![
                crate::models::EffectiveRow {
                    y_start: 0,
                    y_end: 8,
                    spec: gear_sibling_spec(0.5, 0.5, 1.0),
                },
                crate::models::EffectiveRow {
                    y_start: 8,
                    y_end: 16,
                    spec: gear_sibling_spec(3.0, 0.5, 1.0),
                },
            ],
            ..Default::default()
        };

        // `gear_machine_entities_at` places generic, unattributed inserters
        // (`carries: None`) — patch in the recipe-matching `carries` tag so
        // this test exercises attribution, not the "no carries → skip"
        // path. Each row is a fixed [machine, input, output] triple
        // (`gear_machine_entities_at`'s construction order), so index
        // parity — not a global y-threshold, which would misclassify row
        // B's input inserter (y=9) as "below" row A's output (y=3) — picks
        // input vs output correctly per row.
        let mut lr = lr;
        for (idx, e) in lr.entities.iter_mut().enumerate() {
            if e.name == "inserter" {
                e.carries = Some(if idx % 3 == 1 { "iron-plate".into() } else { "iron-gear-wheel".into() });
            }
        }

        let issues = check_inserter_item_throughput(&lr, Some(&sr));
        let warns = item_throughput_warnings(&issues);
        assert_eq!(
            warns.len(),
            1,
            "row A (0.5/s, in-capacity) must stay clean, row B (3.0/s, over-capacity) must warn: {issues:?}"
        );
        assert!(warns[0].message.contains("(0,10)"), "the warning must land on row B's machine: {warns:?}");
        assert!(warns[0].message.contains("iron-plate"));
        assert!(warns[0].message.contains("in —"));
    }
}
