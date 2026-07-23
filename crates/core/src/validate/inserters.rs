//! Inserter chain validity and direction checks.
//!
//! Port of `check_inserter_chains` and `check_inserter_direction` from
//! `src/validate.py`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{
    dir_to_vec, fluid_only_recipes, inserter_reach, is_inserter, is_machine_entity,
    is_splitter, is_surface_belt, is_ug_belt, machine_dims, machine_tiles, recycler_eject_tile,
    splitter_to_surface_tier, ug_to_surface_tier, utilization_for,
};
#[cfg(test)]
use crate::common::inserter_throughput;
use crate::models::{EntityDirection, LayoutResult, MachineSpec, PlacedEntity, SolverResult};

/// RFC-046: throughput an inserter delivers when dropping onto a **belt**.
/// At the layout's belt stack size S > 1, a stack inserter's belt drops
/// use the `swings × belt-hand` decomposition (BS3: hand 6 rounded down
/// to a multiple of S — including the real 9.6/s dip at S=4). At S ≤ 1,
/// and for every other inserter type, this is the flat I8 constant —
/// bit-identical to pre-RFC behavior (kill 1). Machine drops never use
/// this: they are exact-hand (BS2), always the flat constant.
/// RFC-049 extends this with the inserter-capacity research `level`
/// (from `LayoutResult.inserter_capacity`): stack inserters use the
/// swings × researched-belt-hand decomposition whenever EITHER axis is
/// active (S>1 or L>0 — `stack_inserter_belt_hand_at(0, S)` reduces to
/// the RFC-046 helper, so the S-only path is bit-identical); non-bulk
/// belt-drops at L>0 scale by the sim-corrected multiplier table. Bulk
/// inserters stay flat at every level — the engine never places them and
/// parsed blueprints get the conservative floor. #385 additionally caps
/// every credit at `LANE_UTILIZATION` of the TARGET belt's stacked lane
/// rate (a belt-dropping inserter loads one physical lane and cannot
/// out-run its slot cadence, sim-measured 2026-07-23) — hence
/// `target_belt`, the drop tile's belt entity name.
fn belt_drop_throughput(ins: &PlacedEntity, stacking: u8, level: u8, target_belt: &str) -> f64 {
    // Single source of truth shared with the sizing ladder — see
    // `common::belt_drop_rate` (constants-identity discipline: the ladder
    // and this check must never disagree on a belt-dropping inserter's rate).
    crate::common::belt_drop_rate(
        &ins.name,
        ins.quality.unwrap_or_default(),
        stacking,
        level,
        target_belt,
    )
}

/// Map every belt-ish tile (surface belt, underground belt, splitter) to
/// its surface belt-tier entity name, for #385's lane-cap lookup: the
/// drop tile's belt entity name isn't always a plain surface belt (an
/// inserter can drop onto a splitter or a UG entrance), so undergrounds
/// and splitters are normalized to their surface tier via
/// `ug_to_surface_tier`/`splitter_to_surface_tier` — the same mapping
/// `belt_entity_for_rate`'s callers use elsewhere. Positions with no
/// belt-ish entity are simply absent; callers fall back to the most
/// conservative tier ("transport-belt") when a lookup misses.
fn belt_tier_by_tile(layout: &LayoutResult) -> FxHashMap<(i32, i32), &str> {
    let mut by_tile = FxHashMap::default();
    for e in &layout.entities {
        let tier: &str = if is_ug_belt(&e.name) {
            ug_to_surface_tier(&e.name)
        } else if is_splitter(&e.name) {
            splitter_to_surface_tier(&e.name)
        } else if is_surface_belt(&e.name) {
            e.name.as_str()
        } else {
            continue;
        };
        by_tile.insert((e.x, e.y), tier);
        // Splitters span two tiles; register the second one too (review
        // finding on #394 — every other tile-map builder does this via
        // the same helper).
        if is_splitter(&e.name) {
            by_tile.insert(crate::common::splitter_second_tile(e), tier);
        }
    }
    by_tile
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
    let belt_tiles = belt_tier_by_tile(layout);

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
        // Input side (belt→machine): level-aware machine-feed rate
        // (RFC-049 Phase 2 — hand-ratio scaling, sim-calibrated
        // conservative; see common::machine_feed_rate). Output side
        // shadows this with the belt-drop rating below.
        let rate = crate::common::machine_feed_rate(
            &ins.name,
            ins.quality.unwrap_or_default(),
            layout.inserter_capacity,
        );

        if let Some(&mpos) = machine_by_tile.get(&drop_pos) {
            *input_avail.entry(mpos).or_insert(0.0) += rate;
            *input_count.entry(mpos).or_insert(0) += 1;
        }
        if let Some(&mpos) = machine_by_tile.get(&pickup_pos) {
            // A drop_pos inside the same machine would make this a
            // machine-internal inserter — ignore those (they don't extract
            // to a belt). Only count picks that actually leave the machine.
            if !machine_tiles_set.contains(&drop_pos) {
                // Extraction drops onto a belt → belt-drop rating (RFC-046),
                // lane-capped by the drop tile's own belt tier (#385) —
                // fall back to the most conservative tier when no belt is
                // found there (e.g. hand-built test layouts).
                let target_belt = belt_tiles.get(&drop_pos).copied().unwrap_or("transport-belt");
                let rate = belt_drop_throughput(ins, layout.stacking, layout.inserter_capacity, target_belt);
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
    let belt_tiles = belt_tier_by_tile(layout);

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
        // Input side (belt→machine): level-aware machine-feed rate
        // (RFC-049 Phase 2 — hand-ratio scaling, sim-calibrated
        // conservative; see common::machine_feed_rate). Output side
        // shadows this with the belt-drop rating below.
        let rate = crate::common::machine_feed_rate(
            &ins.name,
            ins.quality.unwrap_or_default(),
            layout.inserter_capacity,
        );

        if let Some(&mpos) = machine_by_tile.get(&drop_pos) {
            *input_avail.entry((mpos, item)).or_insert(0.0) += rate;
        }
        if let Some(&mpos) = machine_by_tile.get(&pickup_pos) {
            if !machine_tiles_set.contains(&drop_pos) {
                // Extraction drops onto a belt → belt-drop rating (RFC-046),
                // lane-capped by the drop tile's own belt tier (#385) —
                // fall back to the most conservative tier when no belt is
                // found there.
                let target_belt = belt_tiles.get(&drop_pos).copied().unwrap_or("transport-belt");
                let rate = belt_drop_throughput(ins, layout.stacking, layout.inserter_capacity, target_belt);
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

// ── check_row_output_lane_budget ─────────────────────────────────────────────

/// #385's second half: a machine row's belt-out realizes a sim-MEASURED
/// fraction of its lane capacity, not the nominal both-lane figure —
/// inserter drops fill only the far lane (I5), so an unbridged row
/// realizes ~0.95 × one lane regardless of inserter type, count, or
/// research level (7.40/s measured on yellow at S=1, parity world; the
/// constants and their calibration cells live at the computation site
/// below and in the RFC-047 decision log's 2026-07-23 row-calibration
/// entry). A midpoint sideload bridge redistributes flow across both
/// physical lanes and lifts the measured ceiling to ≥1.733 lanes
/// (13.00/s delivered at plan on yellow — the measured FLOOR; the band
/// up to the 2-lane nominal is unproven either way).
///
/// This is the row-AGGREGATE counterpart to `belt_drop_throughput`'s
/// #385 per-inserter lane cap: [`check_inserter_throughput`] and
/// [`check_inserter_item_throughput`] already cap each individual
/// inserter's own credit at the lane rate, but a row's PLANNED output
/// (the recipe's demand-driven production, independent of how many
/// inserters happen to feed the belt) can still exceed what the belt
/// itself physically carries even when every individual inserter's own
/// credit looks fine in isolation — the residual #385's decision log
/// (`docs/rfc-049-inserter-capacity-research.md`, 2026-07-23 entry)
/// flagged as still open: the regenerated ec10-L7 fixture measures
/// clean per-machine but the ROW aggregate (Σ of a row's belt-drops vs
/// the belt-out's realizable capacity) was never checked.
///
/// **Row identification.** Bus row templates (`bus::templates`) tag
/// every row's machines `row:<recipe>:machine` and its primary output
/// belt `row:<recipe>:belt-out` — but a recipe with enough machines to
/// need more than one physical row shares that exact literal segment
/// string across every row (confirmed empirically on `electronic-circuit
/// @10/s`: copper-cable places 3 physical rows of 5 machines each, ALL
/// tagged `row:copper-cable:machine`/`row:copper-cable:belt-out`), and
/// the cell-composition pipeline (`bus::cells`) never populates
/// `LayoutResult::effective_rows` at all (confirmed on the composed
/// EC@15 fixture: 3 independent cells' belt-out runs, tens of tiles
/// apart, all sharing the literal `row:electronic-circuit:belt-out`
/// string) — so a validator can't lean on `effective_rows` here the way
/// `resolve_row_spec` does elsewhere in this file.
///
/// Instead, each recipe's belt-out tiles are split into physical rows by
/// **tile adjacency** ([`cluster_tiles_by_adjacency`]): real construction
/// never places two unrelated rows'/cells' tiles edge-adjacent (always
/// several tiles of gap), while a row's own main line + midpoint
/// sideload bridge genuinely ARE edge-adjacent (the bridge's lift/drop
/// tiles touch the main line directly) — so adjacency is a robust,
/// pipeline-independent proxy for "same physical row" that needs no
/// per-pipeline plumbing.
///
/// Each machine is then attributed to a physical row via its OWN output
/// inserter's drop tile — the actual physical connection, not a
/// geometric proxy: for every inserter whose pickup side touches a
/// machine, if its drop side lands on one of THIS recipe's belt-out
/// clusters, that machine belongs to that row. (An earlier
/// nearest-cluster-by-distance draft was falsified empirically:
/// `place_rows` spaces rows uniformly, so a middle row's machines can
/// sit EXACTLY equidistant between its own belt-out and the previous
/// row's — ties a distance heuristic cannot break correctly. Physical
/// inserter connectivity has no such ambiguity.)
///
/// **Row inflow.** `recipe_to_spec` (keyed from `SolverResult::machines`,
/// always the pre-partition, blended, one-entry-per-recipe view per
/// `docs/rfc-inserter-sizing.md` Phase 1) gives the recipe's true total:
/// `per-machine rate × spec.count`. Each row's share of that total is
/// `(machines attributed to this row's belt-out cluster) / (total such
/// machines counted for the recipe across every cluster)` — the ratio
/// the placer's own row-splitting (for throughput, not just solver
/// partitioning) actually places, so this is exact regardless of which
/// mechanism produced multiple physical rows. Multi-output recipes: only
/// the primary item (whatever the belt-out tiles' `carries` field names)
/// is checked — the secondary output rides a separate `belt-out2`
/// segment this check never selects, and a machine whose only traceable
/// output inserter feeds THAT segment (not the primary belt-out) is
/// simply not attributed anywhere — never observed in practice, since
/// every machine has a primary-item output inserter. A cluster with zero
/// attributed machines is skipped rather than guessing a share.
///
/// **Bridge detection.** A row's belt-out tiles span exactly one
/// perpendicular coordinate (the single output-belt line) UNLESS a
/// midpoint sideload bridge is present, which stamps its own line one
/// tile off-axis (`bus::templates::sideload_bridge`'s `bridge_y =
/// output_row_dy - 1`) — verified against the logistic-science-pack
/// fixture's iron row (bridge tiles at y=24, main line at y=25). The
/// majority direction among the row's surface-belt tiles gives the
/// row's own travel axis; `lanes_loaded = 2` iff the tiles span more
/// than one value in the coordinate perpendicular to that axis
/// (y for a horizontal row, x for a vertical one), else `1`.
pub fn check_row_output_lane_budget(
    layout: &LayoutResult,
    solver_result: Option<&SolverResult>,
) -> Vec<ValidationIssue> {
    let sr = match solver_result {
        Some(s) => s,
        None => return Vec::new(),
    };

    let recipe_to_spec: FxHashMap<&str, &MachineSpec> =
        sr.machines.iter().map(|s| (s.recipe.as_str(), s)).collect();

    let mut belt_out_by_recipe: FxHashMap<&str, Vec<&PlacedEntity>> = FxHashMap::default();
    for e in &layout.entities {
        let Some(seg) = e.segment_id.as_deref() else {
            continue;
        };
        // Exact match only — `row:<recipe>:belt-out`, never the
        // `:belt-out2:<item>` secondary-output segment (multi-output
        // rows' secondary item is a different, always-single-lane
        // long-handed drop, out of this check's scope).
        if let Some(recipe) = seg.strip_prefix("row:").and_then(|s| s.strip_suffix(":belt-out")) {
            belt_out_by_recipe.entry(recipe).or_default().push(e);
        }
    }

    let mut recipes: Vec<&str> = belt_out_by_recipe.keys().copied().collect();
    recipes.sort_unstable();

    // Split each recipe's belt-out tiles into physical rows (clusters),
    // and index every tile to its cluster for the inserter-tracing pass
    // below.
    struct RecipeClusters<'a> {
        clusters: Vec<Vec<&'a PlacedEntity>>,
        tile_to_cluster: FxHashMap<(i32, i32), usize>,
    }
    let mut by_recipe: FxHashMap<&str, RecipeClusters> = FxHashMap::default();
    for &recipe in &recipes {
        let belt_tiles_all = &belt_out_by_recipe[recipe];
        let positions: Vec<(i32, i32)> = belt_tiles_all.iter().map(|e| (e.x, e.y)).collect();
        let components = cluster_tiles_by_adjacency(&positions);
        let num_clusters = components.iter().copied().max().map_or(0, |m| m + 1);
        let mut clusters: Vec<Vec<&PlacedEntity>> = vec![Vec::new(); num_clusters];
        let mut tile_to_cluster = FxHashMap::default();
        for (i, &e) in belt_tiles_all.iter().enumerate() {
            clusters[components[i]].push(e);
            tile_to_cluster.insert((e.x, e.y), components[i]);
        }
        by_recipe.insert(recipe, RecipeClusters { clusters, tile_to_cluster });
    }

    // Attribute each machine to a physical row via its own output
    // inserter's drop tile (see doc comment — physical connectivity, not
    // a distance heuristic).
    let machine_origin = machine_origin_by_tile(layout);
    let mut machine_recipe: FxHashMap<(i32, i32), &str> = FxHashMap::default();
    for e in &layout.entities {
        if is_machine_entity(&e.name) {
            if let Some(r) = e.recipe.as_deref() {
                machine_recipe.insert((e.x, e.y), r);
            }
        }
    }

    let mut machine_counts: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    let mut counted: FxHashSet<(&str, (i32, i32))> = FxHashSet::default();
    for ins in &layout.entities {
        if !is_inserter(&ins.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(ins.direction);
        let reach = inserter_reach(&ins.name);
        let pickup = (ins.x - dx * reach, ins.y - dy * reach);
        let drop = (ins.x + dx * reach, ins.y + dy * reach);
        let Some(&origin) = machine_origin.get(&pickup) else {
            continue;
        };
        let Some(&recipe) = machine_recipe.get(&origin) else {
            continue;
        };
        let Some(rc) = by_recipe.get(recipe) else {
            continue;
        };
        let Some(&cluster_idx) = rc.tile_to_cluster.get(&drop) else {
            continue;
        };
        if counted.insert((recipe, origin)) {
            machine_counts
                .entry(recipe)
                .or_insert_with(|| vec![0; rc.clusters.len()])[cluster_idx] += 1;
        }
    }

    const EPSILON: f64 = 0.02;
    let mut issues = Vec::new();
    for &recipe in &recipes {
        let Some(spec) = recipe_to_spec.get(recipe) else {
            continue;
        };
        let rc = &by_recipe[recipe];
        let Some(counts) = machine_counts.get(recipe) else {
            continue;
        };
        let total_machines: usize = counts.iter().sum();
        if total_machines == 0 {
            continue;
        }

        for (idx, belt_tiles) in rc.clusters.iter().enumerate() {
            let mcount = counts[idx];
            if mcount == 0 {
                continue;
            }

            // Primary output item: whatever this row's belt-out tiles
            // actually carry.
            let Some(item) = belt_tiles.iter().find_map(|e| e.carries.as_deref()) else {
                continue;
            };
            let Some(flow) = spec.outputs.iter().find(|f| !f.is_fluid && f.item == item) else {
                continue;
            };
            let total_recipe_rate = flow.rate * spec.count;
            let row_share = mcount as f64 / total_machines as f64;
            let inflow = total_recipe_rate * row_share;

            let belts: Vec<&PlacedEntity> = belt_tiles
                .iter()
                .copied()
                .filter(|e| crate::common::is_surface_belt(&e.name))
                .collect();
            if belts.is_empty() {
                continue;
            }

            let tier = row_belt_tier(&belts);
            let lanes_loaded = row_lanes_loaded(&belts);

            // Row-level realizable factors are sim-MEASURED (2026-07-23,
            // parity world; two adversarial-review rounds — the first
            // version's unmeasured ×2 was challenged, an interim ×1.5
            // derived from a consumption-limited cell was then itself
            // falsified by a dedicated ceiling cell):
            // - unbridged 3-machine row onto yellow: 7.40/s realized of
            //   the 7.5 lane rate, any inserter type/level → 0.95/lane.
            // - engine-midpoint-bridged single row (cable@13 uncommon,
            //   yellow S1): delivers 13.00/s at plan in-game →
            //   ≥ 1.733 lanes. 13.0 is the measured FLOOR; the band up
            //   to the 2-lane nominal is unproven either way (the
            //   probe above 13 generation-errors before it can run).
            // Both constants are floors at or under their measured cell.
            const ROW_LANE_FACTOR_UNBRIDGED: f64 = 0.95;
            const ROW_LANE_FACTOR_BRIDGED: f64 = 13.0 / 7.5; // 1.7333 measured floor
            let lane_factor = if lanes_loaded >= 2 {
                ROW_LANE_FACTOR_BRIDGED
            } else {
                ROW_LANE_FACTOR_UNBRIDGED
            };
            let realizable =
                crate::common::lane_capacity_stacked(tier, layout.stacking) * lane_factor;

            if inflow > realizable + EPSILON {
                let (ax, ay) = belts.iter().map(|e| (e.x, e.y)).min().unwrap();
                issues.push(
                    ValidationIssue::with_pos(
                        Severity::Warning,
                        "row-output-lane-budget",
                        format!(
                            "{} row output ({}) needs {:.2}/s but inserter-drop delivery onto {} \
                             ({} lane{} loaded) realizes only {:.2}/s measured — needs a midpoint \
                             bridge (measured 13.0/s floor on yellow) or a split output",
                            recipe,
                            item,
                            inflow,
                            tier,
                            lanes_loaded,
                            if lanes_loaded == 1 { "" } else { "s" },
                            realizable,
                        ),
                        ax,
                        ay,
                    )
                    .with_detail(realizable, inflow),
                );
            }
        }
    }

    issues
}

/// Splits `positions` into connected components under 4-adjacency
/// (sharing a tile edge), returning each position's component id by
/// index. Used to tell apart physically distinct rows/cells that
/// happen to share a segment-id string (see the caller's doc comment):
/// real layouts never place two unrelated rows' tiles edge-adjacent,
/// while a row's own main line and its midpoint sideload bridge (one
/// tile off-axis, touching the main line directly) always are.
fn cluster_tiles_by_adjacency(positions: &[(i32, i32)]) -> Vec<usize> {
    let index: FxHashMap<(i32, i32), usize> =
        positions.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let mut component = vec![usize::MAX; positions.len()];
    let mut next_id = 0usize;
    for start in 0..positions.len() {
        if component[start] != usize::MAX {
            continue;
        }
        let id = next_id;
        next_id += 1;
        let mut stack = vec![start];
        component[start] = id;
        while let Some(i) = stack.pop() {
            let (x, y) = positions[i];
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                if let Some(&j) = index.get(&(x + dx, y + dy)) {
                    if component[j] == usize::MAX {
                        component[j] = id;
                        stack.push(j);
                    }
                }
            }
        }
    }
    component
}

/// Most common surface-belt tier among a row's belt-out tiles (uniform
/// in practice — main line and bridge share one belt entity type).
/// Defaults to the conservative yellow tier if somehow mixed/empty.
fn row_belt_tier(belts: &[&PlacedEntity]) -> &'static str {
    let mut counts: FxHashMap<&str, usize> = FxHashMap::default();
    for e in belts {
        *counts.entry(e.name.as_str()).or_insert(0) += 1;
    }
    match counts.into_iter().max_by_key(|(_, c)| c.to_owned()).map(|(n, _)| n) {
        Some("fast-transport-belt") => "fast-transport-belt",
        Some("express-transport-belt") => "express-transport-belt",
        _ => "transport-belt",
    }
}

/// Whether a row's belt-out spans one physical lane or two. The
/// majority travel direction among the row's surface-belt tiles gives
/// the row's own axis (horizontal East/West for every bus row template
/// today); `2` iff the tiles span more than one coordinate perpendicular
/// to that axis (a midpoint sideload bridge stamps its own line one tile
/// off-axis — see this function's caller doc comment), else `1`.
fn row_lanes_loaded(belts: &[&PlacedEntity]) -> u8 {
    let (mut north, mut east, mut south, mut west) = (0usize, 0usize, 0usize, 0usize);
    for e in belts {
        match e.direction {
            EntityDirection::North => north += 1,
            EntityDirection::East => east += 1,
            EntityDirection::South => south += 1,
            EntityDirection::West => west += 1,
        }
    }
    let horizontal = (east + west) >= (north + south);
    let spread: FxHashSet<i32> =
        belts.iter().map(|e| if horizontal { e.y } else { e.x }).collect();
    if spread.len() > 1 {
        2
    } else {
        1
    }
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
        // Red (fast-transport-belt) target: its S≤4 lane cap (≥12.75/s at
        // S=1) is high enough that most of this test's swing values pass
        // through unclamped, isolating the swing-term math (#385's lane
        // cap is the separate, dedicated `belt_drop_throughput_lane_cap_*`
        // coverage below). Two assertions below CANNOT be preserved on any
        // belt at S=1 — see their comments.
        const RED: &str = "fast-transport-belt";
        let stack = PlacedEntity { name: "stack-inserter".into(), ..Default::default() };
        let fast = PlacedEntity { name: "fast-inserter".into(), ..Default::default() };
        // S ≤ 1 is the flat I8 constant, bit-identical to pre-RFC (kill 1)
        // — including 0, the derived-Default sentinel on hand-built layouts.
        assert_eq!(
            belt_drop_throughput(&stack, 0, 0, RED),
            inserter_throughput("stack-inserter", QualityTier::Normal)
        );
        assert_eq!(belt_drop_throughput(&stack, 1, 0, RED), 12.0);
        // S=2,3: swings × hand 6 = 14.4/s; S=4: hand rounds down to 4 →
        // the real 9.6/s dip (BS3).
        assert!((belt_drop_throughput(&stack, 2, 0, RED) - 14.4).abs() < 1e-9);
        assert!((belt_drop_throughput(&stack, 3, 0, RED) - 14.4).abs() < 1e-9);
        assert!((belt_drop_throughput(&stack, 4, 0, RED) - 9.6).abs() < 1e-9);
        // Non-stack inserters never stack (BS5) — flat at any S.
        assert_eq!(
            belt_drop_throughput(&fast, 4, 0, RED),
            inserter_throughput("fast-inserter", QualityTier::Normal)
        );
        // Quality scales swings only (BS7): legendary ×2.5 at S=4 → 24/s.
        let legendary = PlacedEntity {
            name: "stack-inserter".into(),
            quality: Some(QualityTier::Legendary),
            ..Default::default()
        };
        assert!((belt_drop_throughput(&legendary, 4, 0, RED) - 24.0).abs() < 1e-9);

        // RFC-049: research dimension. L7/S=4 heals the dip: 2.4×16 = 38.4
        // (red's S=4 cap is 51 — unaffected by #385).
        assert!((belt_drop_throughput(&stack, 4, 7, RED) - 38.4).abs() < 1e-9);
        // L3/S=4 dips (hand 9 → 8): 2.4×8 = 19.2.
        assert!((belt_drop_throughput(&stack, 4, 3, RED) - 19.2).abs() < 1e-9);
        // #385: L>0 without stacking still decomposes the swing term to
        // 2.4×16=38.4, but at S=1 EVERY belt's lane cap sits far below that
        // (red's is 12.75/s; even express's is only 19.125/s) — no S=1
        // belt can physically carry 38.4/s onto one lane, so the lane cap
        // now binds here regardless of target tier. Pre-#385 this asserted
        // the uncapped 38.4; the cap is exactly the fix.
        assert!((belt_drop_throughput(&stack, 1, 7, RED) - 12.75).abs() < 1e-9);
        // #385: non-bulk belt-drop now uses the sim-corrected multiplier
        // (2.67, not the raw 4.0 hand ratio) — fast L7 = 2.31×2.67 ≈ 6.17,
        // still under red's 12.75 cap so the swing term (not the cap) wins.
        assert!((belt_drop_throughput(&fast, 1, 7, RED) - 2.31 * 2.67).abs() < 1e-9);
        // Bulk stays flat at every level (conservative floor); red's cap
        // never binds for bulk's tiny flat rate.
        let bulk = PlacedEntity { name: "bulk-inserter".into(), ..Default::default() };
        assert_eq!(belt_drop_throughput(&bulk, 1, 7, RED), inserter_throughput("bulk-inserter", QualityTier::Normal));
    }

    /// #385: on a YELLOW target, a stack inserter's flat S=1/L=0 credit
    /// (12.0/s) exceeds the lane cap (6.375/s) — the headline break this
    /// issue exists to make. `check_inserter_throughput`'s belt lookup
    /// falls back to `"transport-belt"` when no belt is found at the drop
    /// tile, so a hand-built layout with no belt entities also gets the
    /// most conservative (yellow) cap.
    #[test]
    fn belt_drop_throughput_yellow_cap_binds() {
        let stack = PlacedEntity { name: "stack-inserter".into(), ..Default::default() };
        let got = belt_drop_throughput(&stack, 1, 0, "transport-belt");
        assert!((got - 6.375).abs() < 1e-9, "expected the yellow lane cap 6.375, got {got}");
        // No-belt-found fallback resolves to the same conservative value.
        let fallback = belt_drop_throughput(&stack, 1, 0, "transport-belt");
        assert_eq!(got, fallback);
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

    // ── check_row_output_lane_budget (#385 second half) ─────────────────────

    fn lane_budget_warnings(issues: &[ValidationIssue]) -> Vec<&ValidationIssue> {
        issues
            .iter()
            .filter(|i| i.category == "row-output-lane-budget")
            .collect()
    }

    fn row_output_spec(recipe: &str, item: &str, rate_per_machine: f64, count: f64) -> SolverResult {
        SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-1".into(),
                recipe: recipe.into(),
                self_loop: vec![],
                voider: false,
                game_modules: Vec::new(),
                count,
                inputs: vec![],
                outputs: vec![ItemFlow {
                    item: item.into(),
                    rate: rate_per_machine,
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

    /// A single-lane belt-out run: `count` tiles heading WEST at row `y`,
    /// all tagged `row:<recipe>:belt-out` and carrying `item`.
    fn belt_out_row(recipe: &str, item: &str, y: i32, count: i32, belt: &str) -> Vec<PlacedEntity> {
        (0..count)
            .map(|i| PlacedEntity {
                name: belt.into(),
                x: i,
                y,
                direction: EntityDirection::West,
                carries: Some(item.into()),
                segment_id: Some(format!("row:{recipe}:belt-out")),
                ..Default::default()
            })
            .collect()
    }

    /// Same as [`belt_out_row`] plus one tile at `y - 1` — the minimal
    /// synthetic stand-in for a midpoint sideload bridge's off-axis line
    /// (real bridges stamp 6 tiles; only the perpendicular-spread signal
    /// matters to `row_lanes_loaded`, so one extra tile at the bridge's
    /// row suffices).
    fn belt_out_row_bridged(recipe: &str, item: &str, y: i32, count: i32, belt: &str) -> Vec<PlacedEntity> {
        let mut v = belt_out_row(recipe, item, y, count, belt);
        v.push(PlacedEntity {
            name: belt.into(),
            x: 0,
            y: y - 1,
            direction: EntityDirection::West,
            carries: Some(item.into()),
            segment_id: Some(format!("row:{recipe}:belt-out")),
            ..Default::default()
        });
        v
    }

    /// `count` machines (origin at `x = i*4`, `y = machine_y`) tagged
    /// `row:<recipe>:machine`, each wired to a South-facing, reach-1
    /// output inserter whose pickup lands on the machine's own origin
    /// tile and whose drop lands at `(i*4, machine_y + 2)`. This is the
    /// exact physical connection `check_row_output_lane_budget` traces
    /// to attribute a machine to its row (see the check's doc comment:
    /// inserter connectivity, not a distance heuristic) — a synthetic
    /// row needs this wiring, not just co-located tiles, for the check
    /// to count it at all. Callers must place their belt-out tiles
    /// (`belt_out_row`) at `y = machine_y + 2`, wide enough to cover
    /// every `i*4` drop x.
    fn machine_row_with_output_inserters(recipe: &str, machine_y: i32, count: i32) -> Vec<PlacedEntity> {
        let mut v = Vec::new();
        for i in 0..count {
            let x = i * 4;
            v.push(PlacedEntity {
                name: "assembling-machine-1".into(),
                x,
                y: machine_y,
                recipe: Some(recipe.into()),
                segment_id: Some(format!("row:{recipe}:machine")),
                ..Default::default()
            });
            v.push(PlacedEntity {
                name: "inserter".into(),
                x,
                y: machine_y + 1,
                direction: EntityDirection::South,
                ..Default::default()
            });
        }
        v
    }

    #[test]
    fn row_lane_budget_fires_when_inflow_exceeds_single_lane_yellow() {
        // 2 machines × 5.0/s = 10.0/s inflow onto one lane of yellow
        // (7.125/s realizable) — the ec10 copper-plate shape from #385
        // (single row, no room on a bridge either since 15.0 > 13.0, but
        // this synthetic case isolates the single-lane, no-bridge path).
        let sr = row_output_spec("test-widget", "test-widget", 5.0, 2.0);
        let mut entities = machine_row_with_output_inserters("test-widget", 0, 2); // drops at x=0,4, y=2
        entities.extend(belt_out_row("test-widget", "test-widget", 2, 8, "transport-belt"));
        let lr = LayoutResult { entities, width: 20, height: 20, stacking: 1, ..Default::default() };
        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        let warns = lane_budget_warnings(&issues);
        assert_eq!(warns.len(), 1, "{issues:?}");
        assert!(warns[0].message.contains("test-widget"));
        assert!(warns[0].message.contains("1 lane"));
        assert!(warns[0].message.contains("transport-belt"));
        let detail = warns[0].detail.as_ref().expect("must carry IssueDetail");
        assert!((detail.needed - 10.0).abs() < 1e-9, "{detail:?}");
        assert!((detail.delivered - 7.125).abs() < 1e-9, "{detail:?}");
    }

    #[test]
    fn row_lane_budget_silent_within_single_lane_budget() {
        // 1 machine × 6.0/s = 6.0/s ≤ 7.125/s realizable — clean.
        let sr = row_output_spec("test-widget", "test-widget", 6.0, 1.0);
        let mut entities = machine_row_with_output_inserters("test-widget", 0, 1); // drop at x=0, y=2
        entities.extend(belt_out_row("test-widget", "test-widget", 2, 4, "transport-belt"));
        let lr = LayoutResult { entities, width: 20, height: 20, stacking: 1, ..Default::default() };
        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        assert!(lane_budget_warnings(&issues).is_empty(), "{issues:?}");
    }

    #[test]
    fn row_lane_budget_bridge_doubles_budget_to_silence_same_inflow() {
        // Identical 10.0/s inflow to the firing test above, but the
        // belt-out now has a second off-axis line (bridge) — two lanes
        // realize 13.0/s (measured bridged floor), which covers it.
        let sr = row_output_spec("test-widget", "test-widget", 5.0, 2.0);
        let mut entities = machine_row_with_output_inserters("test-widget", 0, 2); // drops at x=0,4, y=2
        entities.extend(belt_out_row_bridged("test-widget", "test-widget", 2, 8, "transport-belt"));
        let lr = LayoutResult { entities, width: 20, height: 20, stacking: 1, ..Default::default() };
        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        assert!(
            lane_budget_warnings(&issues).is_empty(),
            "bridged row should realize the 13.0/s measured floor (≥ 10.0/s): {issues:?}"
        );
    }

    #[test]
    fn row_lane_budget_epsilon_boundary() {
        // Exactly at the realizable figure (7.125/s) plus the check's own
        // 0.02 epsilon must NOT fire; a hair over must.
        let mut entities = machine_row_with_output_inserters("test-widget", 0, 1); // drop at x=0, y=2
        entities.extend(belt_out_row("test-widget", "test-widget", 2, 4, "transport-belt"));
        let lr = LayoutResult { entities, width: 20, height: 20, stacking: 1, ..Default::default() };

        let at_boundary = row_output_spec("test-widget", "test-widget", 7.145, 1.0); // 7.125 + 0.02
        let issues = check_row_output_lane_budget(&lr, Some(&at_boundary));
        assert!(
            lane_budget_warnings(&issues).is_empty(),
            "exactly realizable+epsilon must not fire: {issues:?}"
        );

        let past_boundary = row_output_spec("test-widget", "test-widget", 7.15, 1.0); // 7.125 + 0.025
        let issues = check_row_output_lane_budget(&lr, Some(&past_boundary));
        assert_eq!(
            lane_budget_warnings(&issues).len(),
            1,
            "just past realizable+epsilon must fire: {issues:?}"
        );
    }

    #[test]
    fn row_lane_budget_ignores_secondary_belt_out2_segment() {
        // A `belt-out2` (secondary-output) segment must never be picked up
        // as a primary belt-out — no group, no issue, regardless of how
        // far its own flow would exceed a lane budget. (No `machine`
        // wiring needed: absent a `row:<recipe>:belt-out` entry at all,
        // the recipe is never even considered.)
        let sr = row_output_spec("test-widget", "byproduct", 50.0, 1.0);
        let entities = vec![PlacedEntity {
            name: "transport-belt".into(),
            x: 0,
            y: 5,
            direction: EntityDirection::West,
            carries: Some("byproduct".into()),
            segment_id: Some("row:test-widget:belt-out2:byproduct".into()),
            ..Default::default()
        }];
        let lr = LayoutResult { entities, width: 20, height: 20, stacking: 1, ..Default::default() };
        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        assert!(lane_budget_warnings(&issues).is_empty(), "{issues:?}");
    }

    #[test]
    fn row_lane_budget_adjacency_disambiguates_same_recipe_rows_no_effective_rows() {
        // Two physical `copper-cable` rows sharing the literal segment
        // string `row:copper-cable:belt-out`/`:machine`, with NO
        // `effective_rows` populated at all — the cell-composition
        // pipeline shape confirmed on the real composed EC@15 fixture
        // (3 independent cells, tens of tiles apart, sharing one segment
        // string, `effective_rows.len() == 0`). Row A gets 1 of 6 total
        // machines (small share, stays clean); row B gets the other 5
        // (large share, must warn). Both rows' tiles must be split by
        // TILE ADJACENCY (`cluster_tiles_by_adjacency`) into distinct
        // physical rows — not merged into one recipe-wide bucket, which
        // would either double-count inflow or misdetect bridge geometry
        // across the two rows' unrelated, far-apart y-coordinates.
        let sr = row_output_spec("copper-cable", "copper-cable", 2.0, 6.0); // 12.0/s total

        // Row A: 1 machine, drop at (0, 4).
        let mut entities = machine_row_with_output_inserters("copper-cable", 2, 1);
        entities.extend(belt_out_row("copper-cable", "copper-cable", 4, 4, "transport-belt"));
        // Row B: 5 machines, drops at (0,4,8,12,16, y=24) — far from row A.
        entities.extend(machine_row_with_output_inserters("copper-cable", 22, 5));
        entities.extend(belt_out_row("copper-cable", "copper-cable", 24, 20, "transport-belt"));

        // Deliberately no `effective_rows` — proves the adjacency path,
        // not a fallback to some other row-band mechanism, did the split.
        let lr = LayoutResult { entities, width: 20, height: 40, stacking: 1, ..Default::default() };

        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        let warns = lane_budget_warnings(&issues);
        assert_eq!(
            warns.len(),
            1,
            "row A (1/6 share = 2.0/s) stays clean, row B (5/6 share = 10.0/s) warns: {issues:?}"
        );
        assert!(warns[0].y == Some(24), "warning must anchor to row B's own tiles: {warns:?}");
        let detail = warns[0].detail.as_ref().expect("must carry IssueDetail");
        assert!((detail.needed - 10.0).abs() < 1e-9, "{detail:?}");
    }

    #[test]
    fn row_lane_budget_ignores_separate_cell_at_different_x_same_y() {
        // The exact composed-EC@15 false-positive shape (RFC-051/048):
        // two "cells" at the SAME y but far apart in x, sharing the
        // literal `row:widget:belt-out`/`:machine` strings, no
        // `effective_rows`. Each cell's own demand (5.0/s) sits
        // comfortably under a single lane's 7.125/s budget — merging
        // them (10.0/s combined) would wrongly fire. Confirms adjacency
        // splits on x-gaps too, not just y-gaps, and that machine
        // attribution (via inserter drop tracing) stays local to each
        // cell rather than nearest-cluster guessing.
        let sr = row_output_spec("widget", "widget", 5.0, 2.0); // 10.0/s total, 2 machines
        let mut entities = machine_row_with_output_inserters("widget", 9, 1); // cell A: drop (0,11)
        entities.extend(belt_out_row("widget", "widget", 11, 4, "transport-belt")); // cell A belt-out x=0..3, y=11
        entities.extend({
            // cell B: same y=9/11 as cell A, but 40 tiles east — never
            // tile-adjacent to cell A's run.
            let mut m = machine_row_with_output_inserters("widget", 9, 1);
            for e in &mut m {
                e.x += 40;
            }
            let mut b = belt_out_row("widget", "widget", 11, 4, "transport-belt");
            for e in &mut b {
                e.x += 40;
            }
            m.extend(b);
            m
        });
        let lr = LayoutResult { entities, width: 60, height: 20, stacking: 1, ..Default::default() };

        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        assert!(
            lane_budget_warnings(&issues).is_empty(),
            "each cell's own 5.0/s sits under the 7.125/s single-lane budget: {issues:?}"
        );
    }

    #[test]
    fn cluster_tiles_by_adjacency_bridge_merges_far_rows_split() {
        // A row's main line + its one-tile-offset bridge line must land
        // in ONE component; a second, far-away run must be a SEPARATE
        // component — exactly the two invariants the caller's row split
        // depends on.
        let tiles: Vec<PlacedEntity> = belt_out_row_bridged("x", "x", 5, 4, "transport-belt");
        let mut positions: Vec<(i32, i32)> = tiles.iter().map(|e| (e.x, e.y)).collect();
        let main_and_bridge_len = positions.len();
        positions.push((100, 100)); // far, unconnected
        let components = cluster_tiles_by_adjacency(&positions);
        let first_component = components[0];
        assert!(
            components[..main_and_bridge_len].iter().all(|&c| c == first_component),
            "main line + bridge must be one component: {components:?}"
        );
        assert_ne!(
            components[main_and_bridge_len], first_component,
            "a far, unconnected tile must be its own component: {components:?}"
        );
    }

    #[test]
    fn row_lane_budget_no_solver_result_is_noop() {
        let mut entities = machine_row_with_output_inserters("test-widget", 0, 2);
        entities.extend(belt_out_row("test-widget", "test-widget", 2, 8, "transport-belt"));
        let lr = LayoutResult { entities, width: 20, height: 20, stacking: 1, ..Default::default() };
        assert!(check_row_output_lane_budget(&lr, None).is_empty());
    }

    #[test]
    fn row_lane_budget_no_machine_tiles_is_skipped() {
        // Belt-out tiles with no matching machine anywhere feeding them
        // via an output inserter — the check can't attribute a share, so
        // it must not guess (rather than, say, defaulting to the full
        // recipe total).
        let sr = row_output_spec("test-widget", "test-widget", 50.0, 1.0);
        let lr = LayoutResult {
            entities: belt_out_row("test-widget", "test-widget", 5, 4, "transport-belt"),
            width: 20,
            height: 20,
            stacking: 1,
            ..Default::default()
        };
        let issues = check_row_output_lane_budget(&lr, Some(&sr));
        assert!(lane_budget_warnings(&issues).is_empty(), "{issues:?}");
    }

    #[test]
    fn row_belt_tier_picks_majority_name() {
        let a = PlacedEntity { name: "fast-transport-belt".into(), ..Default::default() };
        let b = PlacedEntity { name: "fast-transport-belt".into(), ..Default::default() };
        let c = PlacedEntity { name: "transport-belt".into(), ..Default::default() };
        assert_eq!(row_belt_tier(&[&a, &b, &c]), "fast-transport-belt");
    }

    #[test]
    fn row_lanes_loaded_single_line_is_one() {
        let tiles: Vec<PlacedEntity> = belt_out_row("x", "x", 5, 4, "transport-belt");
        let refs: Vec<&PlacedEntity> = tiles.iter().collect();
        assert_eq!(row_lanes_loaded(&refs), 1);
    }

    #[test]
    fn row_lanes_loaded_bridge_is_two() {
        let tiles: Vec<PlacedEntity> = belt_out_row_bridged("x", "x", 5, 4, "transport-belt");
        let refs: Vec<&PlacedEntity> = tiles.iter().collect();
        assert_eq!(row_lanes_loaded(&refs), 2);
    }
}
