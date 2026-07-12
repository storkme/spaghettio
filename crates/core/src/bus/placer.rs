//! Stacks assembly rows vertically in dependency order.
//!
//! Port of `src/bus/placer.py`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::bus::inserter_ladder::{reassign_near_far, InserterTier};
use crate::bus::layout::RowLayout;
use crate::common::{belt_entity_for_rate, lane_capacity, machine_dims, BELT_TIERS};
use crate::models::{EntityDirection, MachineSpec, PlacedEntity, SolverResult};

/// Best available per-lane capacity across all belt tiers.
fn max_lane_capacity() -> f64 {
    BELT_TIERS
        .iter()
        .map(|(_, c)| *c / 2.0)
        .fold(0.0_f64, f64::max)
}

/// Resolve the effective input-side per-lane capacity given an optional tier cap.
fn effective_in_lane_cap(max_belt_tier: Option<&str>) -> f64 {
    match max_belt_tier {
        Some(tier) => lane_capacity(tier),
        None => max_lane_capacity(),
    }
}

/// Belt tier for a row's INPUT belt — always picks the maximum
/// allowed by `max_belt_tier`, regardless of per-row consumption rate.
///
/// The per-row consumption rate would let `belt_entity_for_rate` pick a
/// smaller tier, but the row's input belt is connected directly to the
/// bus tap-off, which uses the trunk's tier (sized for total demand
/// across all consumers). When the trunk is faster than the per-row
/// rate, picking the per-row tier creates a tier mismatch at the seam:
/// fast belt feeds yellow belt, validator flags lane-throughput
/// errors, and items physically back up at the boundary.
///
/// Always matching the user's max tier avoids the seam mismatch. Cost:
/// slightly more red/blue belts than the minimum needed for the per-
/// row throughput, which is acceptable since the user explicitly chose
/// that tier as the cap.
fn row_input_belt(max_belt_tier: Option<&str>) -> &'static str {
    belt_entity_for_rate(f64::INFINITY, max_belt_tier)
}

// (LANE_SPLIT_GAP deleted in the inline-bridge unification —
// templates now pack tight with the bridge stamped inline.)

/// Per-row metadata for `RowLayout::HorizontalStack` rows. Recorded by the
/// placer and consumed by `lane_planner` to allocate K input₀ trunks.
#[derive(Debug, Clone)]
pub struct HorizontalStackInfo {
    /// The high-demand item that gets `trunk_ys.len()` stacked trunks.
    pub input0_item: String,
    /// Y-coordinates of each input₀ trunk (top of the row), one entry
    /// per trunk. Tap-offs from the bus arrive at these ys.
    pub trunk_ys: Vec<i32>,
}

/// Where a row sits in the layout and what it contains.
#[derive(Debug, Clone)]
pub struct RowSpan {
    pub y_start: i32,
    pub y_end: i32, // exclusive
    pub spec: MachineSpec,
    pub machine_count: usize,
    /// Module index this producer row belongs to. `0` under
    /// `LayoutStrategy::Pooled` and for non-partitioned items;
    /// `> 0` when the partitioner has split a producer into K sibling
    /// rows. Read by `lane_planner` to key on `(item, module_id)`.
    /// See `docs/rfp-modular-production.md`.
    pub module_id: u32,
    pub input_belt_y: Vec<i32>,
    pub output_belt_y: i32,
    pub row_width: i32,
    pub fluid_port_ys: Vec<i32>,
    /// Per-fluid-item input port pipe positions (item, x, y).
    pub fluid_port_pipes: Vec<(String, i32, i32)>,
    /// Per-fluid-item output port pipe positions (item, x, y).
    pub fluid_output_port_pipes: Vec<(String, i32, i32)>,
    /// True when the row's output belts flow East (final-output rows).
    /// False when they flow West back toward the bus (intermediate
    /// producer rows feeding an item consumed further down the bus).
    pub output_east: bool,
    /// Leftmost x coordinate of the output belt run. For westward rows,
    /// items exit the row at `output_belt_x_min - 1`.
    pub output_belt_x_min: i32,
    /// Rightmost x coordinate of the output belt run. For eastward rows,
    /// items exit the row at `output_belt_x_max + 1`.
    pub output_belt_x_max: i32,
    /// `Some(_)` when this row uses `RowLayout::HorizontalStack`. The
    /// lane planner reads this to allocate K trunk lanes for the
    /// row's high-demand input. See `docs/rfp-horizontal-trunks.md`.
    pub horizontal_stack: Option<HorizontalStackInfo>,
    /// `Some((item, y))` when this row's spec has a SECOND solid output
    /// beyond the primary (which owns `output_belt_y`) — e.g.
    /// uranium-processing's uranium-238 surplus alongside uranium-235's
    /// target belt. `y` is the secondary belt's row. Only
    /// `RowKind::SingleInput` rows with 2+ solid outputs populate this
    /// today (RFP Fulgora D2b, `docs/rfp-fulgora-scrap.md`). Read by the
    /// step-7 solid-surplus merger (`ghost_router` step 7b).
    pub secondary_output_belt: Option<(String, i32)>,
    /// Per-item output belt y for rows that emit MANY single-item output
    /// belts, each at its own row (RFP Fulgora Phase 3,
    /// `docs/rfp-fulgora-scrap.md` D3): the scrap-recycling sushi-sorter
    /// row lifts each of the recycler's ~12 mixed outputs onto its own
    /// east-flowing belt at a distinct y. Generalises
    /// `secondary_output_belt` from one extra belt to N. Empty for every
    /// ordinary row. Read by the same three item→belt-y lookup sites the
    /// secondary belt uses: `lane_planner` source_y, `ghost_router`
    /// `row_exit_origin`, and the step-7b surplus merger. The helper
    /// [`RowSpan::output_belt_y_for`] centralises the lookup.
    pub sorted_output_belts: Vec<(String, i32)>,
}

impl RowSpan {
    /// The physical output-belt y for `item` on this row. Checks the
    /// per-item `sorted_output_belts` map first (scrap-recycling sushi
    /// sorter), then the single `secondary_output_belt` (D2b), then falls
    /// back to the row's primary `output_belt_y`. One helper so the lane
    /// planner, ghost router, and surplus merger can't disagree about
    /// which belt an item exits from.
    pub fn output_belt_y_for(&self, item: &str) -> i32 {
        if let Some((_, y)) = self.sorted_output_belts.iter().find(|(it, _)| it == item) {
            return *y;
        }
        if let Some((sec_item, sec_y)) = &self.secondary_output_belt {
            if sec_item == item {
                return *sec_y;
            }
        }
        self.output_belt_y
    }
}

/// Maximum machines in one row before output or input exceeds belt lane capacity.
///
/// Used for **fluid rows** and **3+ solid-input rows** where the output belt is
/// sideloaded from one side (filling only one lane), and no lane-split bridge is
/// placed on the input side.
///
/// Mechanics rules relied on:
/// - **B7** — straight feed into a belt loads both lanes normally.
/// - **B8** — sideloading fills only the near lane.
/// - **I5** — inserter drop targets the near lane of the receiving belt.
/// - **I6** — inserter pickup reads from both lanes; effective rate = full belt throughput.
///
/// **Output limit** (`out_lane_cap / rate`):
/// The row's output belt is sideloaded by an inserter (I5/B8), so only one lane
/// is ever filled. The effective output capacity is therefore a single lane:
/// 7.5/s (yellow), 15/s (red), or 22.5/s (blue).
///
/// **Input limit** (`in_lane_cap / rate * 2.0`):
/// The input tap-off feeds the input belt **straight** from the trunk (B7), so
/// both lanes carry items. Inserters picking from that belt consume from both
/// lanes (I6), giving an effective input capacity equal to the full belt
/// throughput. Because `in_lane_cap` is a per-lane figure, the factor of 2
/// converts it to total throughput: `in_lane_cap * 2.0 == belt_throughput`.
pub(crate) fn max_machines_for_belt(
    spec: &MachineSpec,
    belt_name: &str,
    max_belt_tier: Option<&str>,
) -> usize {
    let out_lane_cap = lane_capacity(belt_name);
    let in_lane_cap = effective_in_lane_cap(max_belt_tier);
    let mut max_m: f64 = 999.0;

    for out in &spec.outputs {
        if !out.is_fluid && out.rate > 0.0 {
            max_m = max_m.min((out_lane_cap / out.rate).floor());
        }
    }
    for inp in &spec.inputs {
        if !inp.is_fluid && inp.rate > 0.0 {
            max_m = max_m.min((in_lane_cap / inp.rate).floor() * 2.0);
        }
    }

    (max_m as usize).max(1)
}

/// Maximum machines when using BOTH belt lanes (lane-split output).
///
/// Used for **standard 1- or 2-solid-input rows** where a sideload bridge is
/// placed to fill both output lanes, effectively doubling output throughput.
/// Input capacity is more conservative: the tap-off sideloads into the input
/// belt, which (by B8) fills only one lane.
///
/// Mechanics rules relied on:
/// - **B7** — straight feed into a belt loads both lanes normally.
/// - **B8** — sideloading fills only the near lane.
/// - **I5** — inserter drop targets the near lane of the receiving belt.
/// - **I6** — inserter pickup reads from both lanes; effective rate = full belt throughput.
///
/// **Output limit** (`out_lane_cap / rate * 2.0`):
/// The sideload bridge feeds the output belt from both sides, filling both lanes
/// (B10). The usable output capacity is therefore the full belt throughput
/// (2 × per-lane). Factor of 2 converts per-lane capacity to total belt capacity.
///
/// **Input limit** (`in_lane_cap / rate * 2.0`):
/// The trunk tap-off runs at the same y as the row's input belt and connects
/// to its west end (B7 straight feed), so both lanes carry items. Factor of 2
/// converts per-lane capacity to full belt throughput, matching the output side.
pub(crate) fn max_machines_for_belt_both_lanes(
    spec: &MachineSpec,
    belt_name: &str,
    max_belt_tier: Option<&str>,
) -> usize {
    let out_lane_cap = lane_capacity(belt_name);
    let in_lane_cap = effective_in_lane_cap(max_belt_tier);
    let mut max_m: f64 = 999.0;

    for out in &spec.outputs {
        if !out.is_fluid && out.rate > 0.0 {
            max_m = max_m.min((out_lane_cap / out.rate).floor() * 2.0);
        }
    }
    for inp in &spec.inputs {
        if !inp.is_fluid && inp.rate > 0.0 {
            max_m = max_m.min((in_lane_cap / inp.rate).floor() * 2.0);
        }
    }

    (max_m as usize).max(1)
}

/// Per-row cap for `RowLayout::HorizontalStack` DualInput rows. Same as
/// `max_machines_for_belt_both_lanes` but skips the highest-rate solid
/// input (input₀) — that input is fed via K stacked input belts at the
/// top of the HS row, so its per-row demand is bounded by `K × belt_cap`,
/// not a single belt. The output belt and the low-demand input₁ are
/// still single belts and so still constrain machines per row.
pub(crate) fn max_machines_for_belt_horizontal_stack(
    spec: &MachineSpec,
    belt_name: &str,
    max_belt_tier: Option<&str>,
) -> usize {
    let out_lane_cap = lane_capacity(belt_name);
    let in_lane_cap = effective_in_lane_cap(max_belt_tier);
    let mut max_m: f64 = 999.0;

    for out in &spec.outputs {
        if !out.is_fluid && out.rate > 0.0 {
            max_m = max_m.min((out_lane_cap / out.rate).floor() * 2.0);
        }
    }

    let mut solid_inputs: Vec<&crate::models::ItemFlow> = spec.inputs.iter()
        .filter(|i| !i.is_fluid && i.rate > 0.0)
        .collect();
    solid_inputs.sort_by(|a, b| b.rate.partial_cmp(&a.rate).unwrap_or(std::cmp::Ordering::Equal));
    // Skip input₀ (highest rate) — handled by K stacked trunks.
    for inp in solid_inputs.iter().skip(1) {
        max_m = max_m.min((in_lane_cap / inp.rate).floor() * 2.0);
    }

    (max_m as usize).max(1)
}

/// Return machine specs ordered with upstream (producing) recipes first.
///
/// Performs a topological sort on solid-input dependencies so every producer
/// row sits above every consumer row (bus flow is SOUTH). Fluid dependencies
/// are ignored. Ties are broken by the solver's `dependency_order` (reversed).
pub(crate) fn order_specs<'a>(
    machines: &'a [MachineSpec],
    dependency_order: &[String],
) -> Vec<&'a MachineSpec> {
    // A single recipe may have multiple `MachineSpec`s when the partitioner
    // splits a producer into per-module siblings (same recipe, distinct
    // `outputs[0].module_id`). Collect all siblings; the final emit loop
    // orders them deterministically by module_id.
    let mut recipe_to_specs: FxHashMap<&str, Vec<&MachineSpec>> = FxHashMap::default();
    for m in machines {
        recipe_to_specs.entry(m.recipe.as_str()).or_default().push(m);
    }

    // item -> ALL recipes that produce it. The net-flow solver can return
    // several producers for one item (byproduct crediting — e.g. AOP and
    // basic-oil both supplying petroleum-gas); a single-value map would
    // drop the ordering edge to all but the last producer and let one be
    // placed below its consumer, breaking the lanes-run-south invariant.
    let mut producers: FxHashMap<&str, Vec<&str>> = FxHashMap::default();
    for m in machines {
        for out in &m.outputs {
            if !out.is_fluid {
                producers
                    .entry(out.item.as_str())
                    .or_default()
                    .push(m.recipe.as_str());
            }
        }
    }

    // consumer recipe -> set of producer recipes (solid only)
    let mut deps: FxHashMap<&str, FxHashSet<&str>> = machines
        .iter()
        .map(|m| (m.recipe.as_str(), FxHashSet::default()))
        .collect();

    for m in machines {
        for inp in &m.inputs {
            if inp.is_fluid {
                continue;
            }
            if let Some(prods) = producers.get(inp.item.as_str()) {
                for &prod_recipe in prods {
                    if prod_recipe != m.recipe.as_str() {
                        deps.entry(m.recipe.as_str()).or_default().insert(prod_recipe);
                    }
                }
            }
        }
    }

    // Stable tiebreak: earlier in reversed(dependency_order) wins
    let rev_order: Vec<&str> = dependency_order.iter().rev().map(|s| s.as_str()).collect();
    let mut rank: FxHashMap<&str, usize> = rev_order
        .iter()
        .enumerate()
        .map(|(i, &r)| (r, i))
        .collect();
    for m in machines {
        let next = rank.len();
        rank.entry(m.recipe.as_str()).or_insert(next);
    }

    let all_recipes: FxHashSet<&str> = machines.iter().map(|m| m.recipe.as_str()).collect();

    // Kahn's algorithm — always pop the lowest-rank ready recipe
    let mut remaining: FxHashMap<&str, FxHashSet<&str>> = deps
        .into_iter()
        .filter(|(r, _)| all_recipes.contains(r))
        .collect();

    let mut emitted: Vec<&str> = Vec::new();

    while !remaining.is_empty() {
        let mut ready: Vec<&str> = remaining
            .iter()
            .filter(|(_, d)| d.is_empty())
            .map(|(&r, _)| r)
            .collect();

        if ready.is_empty() {
            // Cycle (shouldn't happen for solid deps, but don't hang)
            ready = remaining.keys().copied().collect();
        }

        ready.sort_by_key(|r| rank.get(r).copied().unwrap_or(usize::MAX));
        let r = ready[0];
        emitted.push(r);
        remaining.remove(r);
        for deps_set in remaining.values_mut() {
            deps_set.remove(r);
        }
    }

    emitted
        .into_iter()
        .flat_map(|r| {
            let mut siblings = recipe_to_specs.remove(r).unwrap_or_default();
            siblings.sort_by_key(|s| s.outputs.first().map(|o| o.module_id).unwrap_or(0));
            siblings
        })
        .collect()
}

/// How a row's inputs/outputs are arranged (determines row height and belt positions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKind {
    /// One solid input (or no inputs) — standard 7-tile high row.
    SingleInput,
    /// Two solid inputs — 8-tile high row.
    DualInput,
    /// Two solid inputs + one fluid input — 9-tile high row.
    FluidDualInput,
    /// One solid input + one fluid input — 8-tile high row (T-shape vertical fluid column).
    FluidInput,
    /// Three solid inputs — 9-tile high row.
    TripleInput,
    /// Four solid inputs — 10-tile high row. Three input belts on the
    /// north side (top two regular, third with UG gaps so a long-handed
    /// inserter can sit on the belt row and reach two tiles further north),
    /// fourth input on the south side via a north-facing long-handed
    /// inserter (TripleInput-style).
    QuadInput,
    /// Oil refinery (fluid-only row).
    OilRefinery,
    /// 2+ distinct fluid inputs on a small (<5×5) machine, no solid input.
    /// Uses stacked-T pattern with UG-pipe-UG isolation flanks. Covers
    /// heavy-oil-cracking, light-oil-cracking, sulfur. See
    /// `docs/archive/rfp-multi-fluid-rows.md`.
    FluidMultiInput,
    /// Self-loop recipe (kovarex-class: an item appears on both sides of
    /// the recipe). `has_minor` is true for the 2-item shape (kovarex:
    /// a net-positive item recirculated via a priority-split corridor,
    /// plus a net-negative item that's ALSO the bus-tapped input, on its
    /// own recirculation belt); false for the 1-item shape (bacteria
    /// cultivations: one net-positive self-loop item plus an ordinary
    /// bus-tapped input). `has_fluid` is true when the recipe also has a
    /// single non-self-loop fluid ingredient (pentapod-egg's water,
    /// fish-breeding's water) — only legal alongside `has_minor == false`.
    /// See `templates::self_loop_row`.
    SelfLoop { has_minor: bool, has_fluid: bool },
    /// Layout-synthesized voider row (RFP Fulgora Phase 2,
    /// `docs/rfp-fulgora-scrap.md` D1): a recycler bank that
    /// self-consumes a solid surplus stream. `MachineSpec.voider ==
    /// true`. Non-square 2×4 machine, direct belt ejection (no output
    /// inserter), no declared bus output. See `templates::voider_row`.
    Voider,
    /// Scrap-recycling sushi-sorter row (RFP Fulgora Phase 3,
    /// `docs/rfp-fulgora-scrap.md` D3): a bank of `recycler`s running
    /// `scrap-recycling` ejects ~12 mixed products onto a `:sushi:` belt;
    /// a bank of filter inserters sorts each onto its own east-flowing
    /// output belt. Non-square 2×4 machine, direct belt ejection, one
    /// output belt PER item (`sorted_output_belts`). Always east-flowing.
    /// See `templates::scrap_recycling_row`.
    ScrapRecycling,
}

impl RowKind {
    /// Row height in tiles.
    pub fn row_height(&self) -> i32 {
        match self {
            RowKind::SingleInput => 7,
            RowKind::DualInput => 8,
            RowKind::FluidDualInput => 9,
            RowKind::FluidInput => 9,
            RowKind::TripleInput => 9,
            // Three north belts (y+0..y+2) + inserter row (y+3) +
            // 3-row machine (y+4..y+6) + south inserter (y+7) +
            // output belt (y+8) + south input belt (y+9) = 10.
            RowKind::QuadInput => 10,
            RowKind::OilRefinery => 7,
            // For 2 fluids + msz=3 + output (inserter+belt OR pipe row):
            // 2 trunk rows + 1 drop ext + 1 UG-out + 3 machine + 2 output = 9
            RowKind::FluidMultiInput => 9,
            // Mirrors `templates::self_loop_row`'s row-offset formulas
            // (msz=3, the only machine size self-loop recipes use today):
            // 1-item: far-return(0) + descent(1) + far(2) + near(3) +
            // ins(4) + machine(5-7) + out-ins(8) + collector(9) +
            // splitter-2nd-row(10) = 11. 2-item adds near2(4)/ins2(5)
            // (machine shifts to 6-8), a minor collector row, a
            // dedicated pass-through row for the major loop's own
            // straight-feed detour, and a near2 east-transit row = 14.
            // `has_fluid` (1-item shape only) inserts one extra
            // fluid-header row directly above the machine, shifting
            // every dy from the machine row down and adding 1 to the
            // total.
            RowKind::SelfLoop { has_minor, has_fluid } => {
                let base = if *has_minor { 14 } else { 11 };
                base + if *has_fluid { 1 } else { 0 }
            }
            // Collector/eject row (0) + 4-tall recycler (1..4) +
            // inserter row (5) + near/tap belt (6) + far/recirc belt
            // (7) = 8. See `templates::voider_row`.
            RowKind::Voider => 8,
            // Scrap input (0) + input inserters (1) + 4-tall recycler
            // (2..5) + sushi (6) + sort inserters (7) + one fan-out row per
            // sorted item. Height is dynamic; this is a lower bound (2
            // items) — the real height comes from
            // `templates::scrap_recycling_row`'s returned value.
            RowKind::ScrapRecycling => 8 + 2,
        }
    }
}

/// Classify a spec into a RowKind.
fn row_kind(spec: &MachineSpec) -> RowKind {
    // Voider rows (RFP Fulgora Phase 2, `docs/rfp-fulgora-scrap.md` D1)
    // short-circuit BEFORE the square-machine debug_assert below —
    // `recycler` is 2×4, non-square, and would trip that assert
    // (working as intended: any OTHER non-square machine reaching that
    // point is a real bug, not this one). Mirrors the self-loop
    // short-circuit immediately below for the same reason.
    if spec.voider {
        return RowKind::Voider;
    }

    // Scrap-recycling sushi-sorter rows (RFP Fulgora Phase 3) also
    // short-circuit before the square-machine assert — `recycler` is 2×4,
    // non-square. A non-voider recycler is always a scrap-recycling
    // producer (voider recyclers are caught above).
    if spec.entity == "recycler" {
        return RowKind::ScrapRecycling;
    }

    // Self-loop recipes (kovarex-class) short-circuit before the
    // ordinary solid/fluid counting cascade — `spec.inputs`/`outputs`
    // carry only NET flows for the self-loop item(s) (see
    // `models::MachineSpec` doc comment), which would otherwise
    // misclassify them. Ordinary (non-self-loop) ingredients like
    // pentapod-egg's water still land in `spec.inputs` normally, so a
    // fluid entry there means the row needs the fluid-header variant.
    if !spec.self_loop.is_empty() {
        let has_fluid = spec.inputs.iter().any(|f| f.is_fluid);
        return RowKind::SelfLoop { has_minor: spec.self_loop.len() > 1, has_fluid };
    }

    let solid_inputs = spec.inputs.iter().filter(|f| !f.is_fluid).count();
    let fluid_inputs = spec.inputs.iter().filter(|f| f.is_fluid).count();

    // "Large" vs "small" here is a single size threshold (5) with no
    // per-axis meaning of its own — every machine that can reach this
    // fluid-only classification today is square, so width and height
    // agree. Recycler (2×4) has no fluid inputs/outputs so it can never
    // hit this branch today, but if a future recipe change routes a
    // non-square machine here, this assert trips loudly instead of
    // silently picking the wrong axis.
    let (mw, mh) = machine_dims(&spec.entity);
    debug_assert_eq!(
        mw, mh,
        "row_kind's large/small fluid-machine split assumes square machines"
    );
    let machine_size = mw;

    // Large machines (5×5) with only fluid inputs use the dedicated fluid-only template.
    if solid_inputs == 0 && fluid_inputs > 0 && machine_size >= 5 {
        return RowKind::OilRefinery;
    }

    // Small machines (<5×5) with 0 solid + ≥2 fluid inputs use the stacked-T
    // multi-fluid template. Covers heavy-oil-cracking, light-oil-cracking, sulfur.
    if solid_inputs == 0 && fluid_inputs >= 2 && machine_size < 5 {
        return RowKind::FluidMultiInput;
    }

    // Small machines (<5×5) with 0 solid + exactly 1 fluid input (e.g. lubricant
    // on chemical-plant). Reuses the continuous-pipe `fluid_only_row` template
    // since a single fluid in/out doesn't need stacked-T isolation.
    if solid_inputs == 0 && fluid_inputs == 1 && machine_size < 5 {
        return RowKind::OilRefinery;
    }

    let has_fluid_dual_solid = solid_inputs == 2 && fluid_inputs == 1;
    let has_fluid = fluid_inputs > 0 && solid_inputs > 0 && !has_fluid_dual_solid;
    let has_triple_solid = solid_inputs == 3 && fluid_inputs == 0;
    let has_quad_solid = solid_inputs == 4 && fluid_inputs == 0;

    if has_fluid_dual_solid {
        RowKind::FluidDualInput
    } else if has_fluid {
        RowKind::FluidInput
    } else if has_quad_solid {
        RowKind::QuadInput
    } else if has_triple_solid {
        RowKind::TripleInput
    } else if solid_inputs <= 1 {
        RowKind::SingleInput
    } else {
        RowKind::DualInput
    }
}

/// Whether lane splitting is applicable to a spec/count combination.
///
/// SingleInput, DualInput, TripleInput, chemical-plant FluidInput, and
/// solid-output FluidDualInput rows all emit a `sideload_bridge` today.
/// FluidMultiInput, the AM2+-with-fluid branch of FluidInput, and
/// fluid-output FluidDualInput don't — they stay single lane until their
/// templates grow bridges (or in the fluid-output case, until there's
/// an analogous fluid-merging pattern to sideload).
fn can_lane_split(spec: &MachineSpec, count: usize) -> bool {
    if count < 2 {
        return false;
    }
    // Rows with 2+ solid outputs (RFP Fulgora D2b: uranium-processing's
    // U-235 target + U-238 surplus) need the sideload-bridge anchor's
    // columns at `output_row_dy - 1` for the secondary output's
    // long-handed extraction inserter — the bridge and the second
    // inserter both want the anchor machine's `mx+2` column, and the
    // bridge already claims every free column at that row for the
    // anchor. Disabling lane-split for these rows sidesteps the
    // collision; the rate that motivates a second output is typically
    // far under single-lane capacity anyway (see `secondary_output_belt`
    // doc comment). Revisit if a fixture needs both.
    if spec.outputs.iter().filter(|f| !f.is_fluid).count() >= 2 {
        return false;
    }
    let kind = row_kind(spec);
    let output_is_fluid =
        spec.outputs.iter().all(|f| f.is_fluid) && !spec.outputs.is_empty();
    let fluid_dual_input_lane_split_supported =
        matches!(kind, RowKind::FluidDualInput) && !output_is_fluid;
    let fluid_multi_input_lane_split_supported =
        matches!(kind, RowKind::FluidMultiInput) && !output_is_fluid;
    matches!(
        kind,
        RowKind::SingleInput
            | RowKind::DualInput
            | RowKind::TripleInput
            | RowKind::FluidInput,
    ) || fluid_dual_input_lane_split_supported
        || fluid_multi_input_lane_split_supported
}

/// Build one row of machines. Returns (entities, span, row_width).
///
/// Calls into the templates module to stamp the actual machine/inserter/belt entities.
pub(crate) fn build_one_row(
    spec: &MachineSpec,
    count: usize,
    bus_width: i32,
    y_cursor: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    output_east: bool,
    row_layout: RowLayout,
) -> (Vec<PlacedEntity>, RowSpan, i32) {
    use crate::bus::templates;

    let kind = row_kind(spec);
    // Scrap-recycling sushi-sorter rows always flow east (every per-item
    // belt reaches the east edge — see `templates::scrap_recycling_row`),
    // regardless of whether any output happens to be a final product.
    let output_east = output_east || matches!(kind, RowKind::ScrapRecycling);
    let lane_split = can_lane_split(spec, count);

    let solid_inputs: Vec<_> = spec.inputs.iter().filter(|f| !f.is_fluid).collect();
    let solid_outputs: Vec<_> = spec.outputs.iter().filter(|f| !f.is_fluid).collect();
    let fluid_inputs: Vec<_> = spec.inputs.iter().filter(|f| f.is_fluid).collect();
    let fluid_outputs: Vec<_> = spec.outputs.iter().filter(|f| f.is_fluid).collect();

    let output_is_fluid = solid_outputs.is_empty() && !fluid_outputs.is_empty();
    let output_item = if output_is_fluid {
        fluid_outputs.first().map(|f| f.item.as_str()).unwrap_or("")
    } else {
        solid_outputs.first().map(|f| f.item.as_str()).unwrap_or("")
    };

    let output_rate = solid_outputs.first().map(|f| f.rate * count as f64).unwrap_or(0.0);
    let out_belt = belt_entity_for_rate(
        output_rate * if lane_split { 1.0 } else { 2.0 },
        max_belt_tier,
    );

    // Second solid output (RFP Fulgora D2b): only `solid_outputs[0]` owns
    // `output_belt_y` today. When a spec has a second solid output (e.g.
    // uranium-processing's uranium-235 target + uranium-238 surplus),
    // size a belt for it too — `RowKind::SingleInput` is the only arm
    // that currently stamps it (see the match arm below); other kinds
    // leave this unused and `secondary_output_belt` stays `None`.
    // `can_lane_split` already forces `lane_split == false` whenever
    // `solid_outputs.len() >= 2`, so this is always single-lane (×2).
    let secondary_solid_output = solid_outputs.get(1);
    let secondary_belt_name: Option<&'static str> = secondary_solid_output
        .map(|f| belt_entity_for_rate(f.rate * count as f64 * 2.0, max_belt_tier));

    let mut fluid_port_ys: Vec<i32> = vec![];
    let mut fluid_port_pipes: Vec<(String, i32, i32)> = vec![];
    let mut fluid_output_port_pipes: Vec<(String, i32, i32)> = vec![];
    let mut horizontal_stack: Option<HorizontalStackInfo> = None;
    let mut secondary_output_belt: Option<(String, i32)> = None;
    let mut sorted_output_belts: Vec<(String, i32)> = Vec::new();

    let (row_ents, row_h, input_belt_ys, output_belt_y) = match &kind {
        RowKind::OilRefinery => {
            // dx port assignment and the `>= 5` split are both along the
            // machine's horizontal face, so this uses width; every machine
            // reaching this arm is square (asserted inside
            // `templates::fluid_only_row`).
            let msz = machine_dims(&spec.entity).0;
            // Port dx assignment depends on the machine.
            //
            // Oil-refinery (5×5, mirrored, direction=NORTH):
            //   Input box 1 at dx=1, input box 2 at dx=3.
            //   Output box 3 at dx=0, output box 4 at dx=2, output box 5 at dx=4.
            //   basic-oil-processing uses box 2 for crude-oil (dx=3) and box 3
            //   for petroleum-gas (dx=0). advanced-oil-processing uses boxes
            //   sequentially: inputs→[dx=1,dx=3], outputs→[dx=0,dx=2,dx=4].
            //
            // Chemical-plant (3×3, unmirrored, direction=NORTH; per
            // `validate/fluids.rs::fluid_ports`):
            //   Inputs at dx=0 and dx=2 (both on the north face).
            //   Outputs at dx=0 and dx=2 (both on the south face).
            //   Lubricant uses one input + one output, both at dx=0.
            let (input_dxs, output_dxs): (&[i32], &[i32]) = if msz >= 5 {
                let in_dxs: &[i32] = match fluid_inputs.len() {
                    0 => &[],
                    1 => &[3],
                    _ => &[1, 3],
                };
                let out_dxs: &[i32] = match fluid_outputs.len() {
                    0 => &[],
                    1 => &[0],
                    2 => &[0, 2],
                    _ => &[0, 2, 4],
                };
                (in_dxs, out_dxs)
            } else {
                let in_dxs: &[i32] = match fluid_inputs.len() {
                    0 => &[],
                    1 => &[0],
                    _ => &[0, 2],
                };
                let out_dxs: &[i32] = match fluid_outputs.len() {
                    0 => &[],
                    1 => &[0],
                    _ => &[0, 2],
                };
                (in_dxs, out_dxs)
            };
            let in_port_assignments: Vec<(i32, &str)> = input_dxs
                .iter()
                .zip(fluid_inputs.iter())
                .map(|(&dx, f)| (dx, f.item.as_str()))
                .collect();
            let out_port_assignments: Vec<(i32, &str)> = output_dxs
                .iter()
                .zip(fluid_outputs.iter())
                .map(|(&dx, f)| (dx, f.item.as_str()))
                .collect();
            let (ents, rh, in_port_pipes, out_port_pipes) = templates::fluid_only_row(
                &spec.recipe,
                &spec.entity,
                msz,
                count,
                y_cursor,
                bus_width,
                &in_port_assignments,
                &out_port_assignments,
            );
            fluid_port_ys = in_port_pipes.first().map(|&(_, _, py)| vec![py]).unwrap_or_default();
            fluid_port_pipes = in_port_pipes;
            fluid_output_port_pipes = out_port_pipes;
            let input_ys = vec![];
            let out_y = y_cursor + rh - 1;
            (ents, rh, input_ys, out_y)
        }
        RowKind::FluidDualInput => {
            let solid_item0 = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let solid_item1 = solid_inputs.get(1).map(|f| f.item.as_str()).unwrap_or("");
            let fluid_item = fluid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            // v3 extension of the reassignment lever (`docs/rfp-inserter-
            // sizing.md`): same hungrier-item-to-near swap as DualInput/
            // TripleInput. Geometrically identical positional pick
            // (source-confirmed) — the fluid PTG's column depends only on
            // `port_dx` (machine type), never on which solid item is
            // passed as input1/input2, so swapping the far/near role
            // never touches the fluid port.
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let item0_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let item1_rate = solid_inputs.get(1).map(|f| f.rate).unwrap_or(0.0) * utilization;
            let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let ((far_item, far_rate), (near_item, near_rate)) =
                reassign_near_far(solid_item0, item0_rate, solid_item1, item1_rate);
            let in_belt1 = row_input_belt(max_belt_tier);
            let in_belt2 = row_input_belt(max_belt_tier);
            let (mw, mh) = machine_dims(&spec.entity);
            let (ents, rh, in_port_pipes, out_port_pipes) = templates::fluid_dual_input_row(
                &spec.recipe,
                &spec.entity,
                mw,
                count,
                y_cursor,
                bus_width,
                (far_item, near_item),
                fluid_item,
                output_item,
                output_is_fluid,
                (in_belt1, in_belt2),
                out_belt,
                lane_split,
                output_east,
                far_rate,
                near_rate,
                output_rate_pm,
                max_inserter_tier,
            );
            let machine_y = y_cursor + 5;
            let output_y = machine_y + mh as i32;
            fluid_port_ys = in_port_pipes.first().map(|&(_, _, py)| vec![py]).unwrap_or_default();
            fluid_port_pipes = in_port_pipes;
            fluid_output_port_pipes = out_port_pipes;
            // Positional (far=y_cursor+2, near=y_cursor+3) mapped back to
            // `solid_inputs`' natural order by item identity, since
            // reassignment may have swapped which physical belt each item
            // lives on.
            let input_ys: Vec<i32> = solid_inputs
                .iter()
                .map(|f| if f.item == far_item { y_cursor + 2 } else { y_cursor + 3 })
                .collect();
            // For solid output, `output_y` from the template is the
            // OUTPUT INSERTER row; the actual output belt is one tile
            // further south at `output_y + 1` (see `templates::
            // fluid_dual_input_row` line 1599-1600 — inserter at
            // output_y, belt at output_y+1). For fluid output, the
            // template stamps a continuous pipe row at `output_y`
            // itself, so no offset.
            //
            // Storing the inserter y here used to leak through to the
            // output merger, which then placed its east-extension
            // belts one tile north of the row's actual belt-out and
            // produced belt-dead-end errors at every row's east edge
            // (e.g. processing-unit @ 2/s row east edges at (75, 174)
            // and (72, 184) before the fix).
            let out_y = if output_is_fluid {
                output_y
            } else {
                output_y + 1
            };
            (ents, rh, input_ys, out_y)
        }
        RowKind::FluidInput => {
            let solid_item = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let fluid_item = fluid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let solid_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let in_belt = row_input_belt(max_belt_tier);
            let (mw, mh) = machine_dims(&spec.entity);
            let (ents, rh, port_pipes) = templates::fluid_input_row(
                &spec.recipe,
                &spec.entity,
                mw,
                count,
                y_cursor,
                bus_width,
                solid_item,
                fluid_item,
                output_item,
                in_belt,
                out_belt,
                lane_split,
                output_east,
                solid_rate,
                output_rate_pm,
                max_inserter_tier,
            );
            fluid_port_ys = port_pipes.first().map(|&(_, _, py)| vec![py]).unwrap_or_default();
            fluid_port_pipes = port_pipes;
            // T-shape layout: trunk at y+0, belt at y+2, machine at y+4, output belt at y+8
            let input_ys = vec![y_cursor + 2];
            let out_y = y_cursor + 4 + mh as i32 + 1;
            (ents, rh, input_ys, out_y)
        }
        RowKind::SingleInput => {
            let input_item = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let in_belt = row_input_belt(max_belt_tier);
            let (mw, mh) = machine_dims(&spec.entity);
            let secondary = secondary_solid_output
                .zip(secondary_belt_name)
                .map(|(f, belt)| (f.item.as_str(), belt));

            // Utilization scaling: the SAME convention
            // `check_inserter_throughput` uses (ce732d9) — a fractional
            // `spec.count` runs each of its `ceil(count)` physical
            // machines at `count/ceil(count)`. The ladder must size to
            // this exact per-machine rate, or a fractional-count spec's
            // inserter picks would silently disagree with what the
            // validator checks against.
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let input_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let output_rate = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let secondary_rate = secondary_solid_output.map(|f| f.rate * utilization);

            let (ents, rh) = templates::single_input_row(
                &spec.recipe,
                &spec.entity,
                mw,
                count,
                y_cursor,
                bus_width,
                input_item,
                output_item,
                in_belt,
                out_belt,
                lane_split,
                output_east,
                secondary,
                input_rate,
                output_rate,
                secondary_rate,
                max_inserter_tier,
            );
            let input_ys = vec![y_cursor];
            let out_y = y_cursor + 2 + mh as i32 + 1;
            if let Some(f) = secondary_solid_output {
                // Mirrors `templates::single_input_row`'s secondary-belt
                // row offset: one row south of the primary output belt.
                secondary_output_belt = Some((f.item.clone(), out_y + 1));
            }
            (ents, rh, input_ys, out_y)
        }
        RowKind::TripleInput => {
            let item0 = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let item1 = solid_inputs.get(1).map(|f| f.item.as_str()).unwrap_or("");
            let item2 = solid_inputs.get(2).map(|f| f.item.as_str()).unwrap_or("");
            // Same reassignment lever as DualInput: item0/item1 are the
            // near-far pair (hungrier -> near); item2 (input3) is fixed
            // reach-2, never reassigned.
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let item0_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let item1_rate = solid_inputs.get(1).map(|f| f.rate).unwrap_or(0.0) * utilization;
            let item2_rate = solid_inputs.get(2).map(|f| f.rate).unwrap_or(0.0) * utilization;
            let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let ((far_item, far_rate), (near_item, near_rate)) =
                reassign_near_far(item0, item0_rate, item1, item1_rate);
            let in_belt1 = row_input_belt(max_belt_tier);
            let in_belt2 = row_input_belt(max_belt_tier);
            let in_belt3 = row_input_belt(max_belt_tier);
            let (mw, mh) = machine_dims(&spec.entity);
            let (ents, rh) = templates::triple_input_row(
                &spec.recipe,
                &spec.entity,
                mw,
                count,
                y_cursor,
                bus_width,
                (far_item, near_item, item2),
                output_item,
                (in_belt1, in_belt2, in_belt3),
                out_belt,
                lane_split,
                output_east,
                far_rate,
                near_rate,
                item2_rate,
                output_rate_pm,
                max_inserter_tier,
            );
            let input_ys: Vec<i32> = solid_inputs
                .iter()
                .map(|f| {
                    if f.item == item2 {
                        y_cursor + 3 + mh as i32 + 2
                    } else if f.item == far_item {
                        y_cursor
                    } else {
                        y_cursor + 1
                    }
                })
                .collect();
            let out_y = y_cursor + 3 + mh as i32 + 1;
            (ents, rh, input_ys, out_y)
        }
        RowKind::QuadInput => {
            let item0 = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let item1 = solid_inputs.get(1).map(|f| f.item.as_str()).unwrap_or("");
            let item2 = solid_inputs.get(2).map(|f| f.item.as_str()).unwrap_or("");
            let item3 = solid_inputs.get(3).map(|f| f.item.as_str()).unwrap_or("");
            // No reassignment lever here — QuadInput's near-far pairing
            // isn't item-swappable (inputs 1/2 are structurally north,
            // input3 dual-baseline, input4 structurally south).
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let item2_rate = solid_inputs.get(2).map(|f| f.rate).unwrap_or(0.0) * utilization;
            let item3_rate = solid_inputs.get(3).map(|f| f.rate).unwrap_or(0.0) * utilization;
            let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let in_belt1 = row_input_belt(max_belt_tier);
            let in_belt2 = row_input_belt(max_belt_tier);
            let in_belt3 = row_input_belt(max_belt_tier);
            let in_belt4 = row_input_belt(max_belt_tier);
            let (mw, mh) = machine_dims(&spec.entity);
            let (ents, rh) = templates::quad_input_row(
                &spec.recipe,
                &spec.entity,
                mw,
                count,
                y_cursor,
                bus_width,
                (item0, item1, item2, item3),
                output_item,
                (in_belt1, in_belt2, in_belt3, in_belt4),
                out_belt,
                lane_split,
                output_east,
                item2_rate,
                item3_rate,
                output_rate_pm,
                max_inserter_tier,
            );
            // input_belt_y[i] is where lane planner taps off lane.item
            // matching solid_inputs[i]. Layout (msz=3): belt 1 at y+0,
            // belt 2 at y+1, belt 3 at y+2, belt 4 (south) at y+9.
            let input_ys = vec![
                y_cursor,
                y_cursor + 1,
                y_cursor + 2,
                y_cursor + 4 + mh as i32 + 2,
            ];
            let out_y = y_cursor + 4 + mh as i32 + 1;
            (ents, rh, input_ys, out_y)
        }
        RowKind::FluidMultiInput => {
            // Chemical-plant fluid input port dxs: [0, 2] per the fluid-box
            // data in recipes.json. The 2 fluid inputs from the solver are
            // assigned to these ports in order.
            let msz = machine_dims(&spec.entity).0;
            let port_dxs: &[i32] = &[0, 2];
            let in_port_assignments: Vec<(i32, &str)> = port_dxs
                .iter()
                .zip(fluid_inputs.iter())
                .map(|(&dx, f)| (dx, f.item.as_str()))
                .collect();
            // Same for fluid outputs (sulfur has none, heavy/light-oil-cracking
            // has 1 — which goes to dx=1 centered on machine).
            let out_port_assignments: Vec<(i32, &str)> = fluid_outputs
                .iter()
                .map(|f| (1i32, f.item.as_str()))
                .collect();
            let solid_out = solid_outputs.first().map(|f| f.item.as_str());
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let (ents, rh, in_port_pipes, out_port_pipes) = templates::fluid_multi_input_row(
                &spec.recipe,
                &spec.entity,
                msz,
                count,
                y_cursor,
                bus_width,
                &in_port_assignments,
                solid_out,
                &out_port_assignments,
                Some(out_belt),
                lane_split,
                output_east,
                output_rate_pm,
                max_inserter_tier,
            );
            fluid_port_ys = in_port_pipes.iter().map(|&(_, _, py)| py).collect();
            fluid_port_ys.sort_unstable();
            fluid_port_ys.dedup();
            fluid_port_pipes = in_port_pipes;
            fluid_output_port_pipes = out_port_pipes;
            let input_ys = vec![];
            let out_y = y_cursor + rh - 1;
            (ents, rh, input_ys, out_y)
        }
        RowKind::DualInput => {
            let msz = machine_dims(&spec.entity).0;
            if matches!(row_layout, RowLayout::HorizontalStack) {
                // Re-rank inputs by per-machine demand so input₀ is the
                // high-demand item (the one that gets K stacked trunks).
                let mut ranked: Vec<&&crate::models::ItemFlow> = solid_inputs.iter().collect();
                ranked.sort_by(|a, b| b.rate.partial_cmp(&a.rate).unwrap_or(std::cmp::Ordering::Equal));
                let item0 = ranked.first().map(|f| f.item.as_str()).unwrap_or("");
                let item1 = ranked.get(1).map(|f| f.item.as_str()).unwrap_or("");
                let item0_per_machine = ranked.first().map(|f| f.rate).unwrap_or(0.0);
                let _item1_per_machine = ranked.get(1).map(|f| f.rate).unwrap_or(0.0);
                // Utilization-scaled rates for the inserter ladder (same
                // convention as the other branches) — kept separate from
                // `item0_per_machine` above, which drives belt-capacity
                // math and must stay the raw per-machine rate.
                let effective_count = spec.count.ceil().max(1.0);
                let utilization = (spec.count / effective_count).min(1.0);
                let near_rate_pm = item0_per_machine * utilization;
                let far_rate_pm = ranked.get(1).map(|f| f.rate).unwrap_or(0.0) * utilization;
                let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
                // Block size: how many machines a single full belt
                // (both lanes) can feed at this per-machine rate. Trunk
                // count: one trunk per block, since each trunk sources
                // one block. The rate-based formula
                // `ceil(total_rate / belt_cap)` undercounts when
                // `block_size`'s floor() leaves spare belt capacity per
                // block (e.g. 6 machines × 4.5/s = 27/s on a 30/s belt
                // → 3/s wasted per block).
                let in_lane_cap = effective_in_lane_cap(max_belt_tier);
                let belt_cap = in_lane_cap * 2.0;
                let block_size = if item0_per_machine > 0.0 {
                    ((belt_cap / item0_per_machine).floor() as usize).max(1)
                } else {
                    count
                };
                let k_trunks = count.div_ceil(block_size).max(1);
                let in_belt1 = belt_entity_for_rate(belt_cap, max_belt_tier);
                let in_belt2 = row_input_belt(max_belt_tier);
                crate::trace::emit(crate::trace::TraceEvent::RowLayoutSelected {
                    recipe: spec.recipe.clone(),
                    kind: "HorizontalStack".to_string(),
                    k_trunks,
                    block_size,
                });
                // Trunk-y ordering is REVERSED: lane 0 taps the trunk
                // closest to iron-plate (y_cursor + k_trunks - 1) and
                // dives at the leftmost dive column. Subsequent lanes
                // are above. This minimises E-UG crossings — every dive
                // only has to bridge iron-plate, since the trunks
                // *below* the diving trunk have already dove west and
                // the trunks *above* don't share rows with the dive's
                // south-belt path.
                horizontal_stack = Some(HorizontalStackInfo {
                    input0_item: item0.to_string(),
                    trunk_ys: (0..k_trunks as i32)
                        .rev()
                        .map(|k| y_cursor + k)
                        .collect(),
                });
                let (ents, rh) = templates::dual_input_row_horizontal(
                    &spec.recipe,
                    &spec.entity,
                    msz,
                    count,
                    y_cursor,
                    bus_width,
                    (item0, item1),
                    output_item,
                    (in_belt1, in_belt2),
                    out_belt,
                    k_trunks,
                    block_size,
                    output_east,
                    near_rate_pm,
                    far_rate_pm,
                    output_rate_pm,
                    max_inserter_tier,
                );
                // Map each spec.solid_input (natural order) to its tap-off
                // y position. High-demand (item0) sits on trunk 0 at y+0;
                // low-demand (item1) sits on the iron-plate row at y+K.
                // The lane planner currently allocates 1 lane per item, so
                // only trunk 0 is fed; K-1 stacked trunks remain empty
                // until the lane-planner work in `task #16` lands K-lane
                // allocation for HorizontalStack rows.
                let high_demand_item = item0.to_string();
                let input_ys: Vec<i32> = solid_inputs
                    .iter()
                    .map(|f| {
                        if f.item == high_demand_item {
                            y_cursor
                        } else {
                            y_cursor + k_trunks as i32
                        }
                    })
                    .collect();
                let out_y = y_cursor + rh - 1;
                (ents, rh, input_ys, out_y)
            } else {
                let item0 = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
                let item1 = solid_inputs.get(1).map(|f| f.item.as_str()).unwrap_or("");
                // Utilization scaling: same convention as SingleInput
                // above. Reassignment lever (`docs/rfp-inserter-sizing.md`
                // lever (b)): hungrier item goes near, where the full
                // tier ladder applies.
                let effective_count = spec.count.ceil().max(1.0);
                let utilization = (spec.count / effective_count).min(1.0);
                let item0_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
                let item1_rate = solid_inputs.get(1).map(|f| f.rate).unwrap_or(0.0) * utilization;
                let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
                let ((far_item, far_rate), (near_item, near_rate)) =
                    reassign_near_far(item0, item0_rate, item1, item1_rate);
                let in_belt1 = row_input_belt(max_belt_tier);
                let in_belt2 = row_input_belt(max_belt_tier);
                let (ents, rh) = templates::dual_input_row(
                    &spec.recipe,
                    &spec.entity,
                    msz,
                    count,
                    y_cursor,
                    bus_width,
                    (far_item, near_item),
                    output_item,
                    (in_belt1, in_belt2),
                    out_belt,
                    lane_split,
                    output_east,
                    far_rate,
                    near_rate,
                    output_rate_pm,
                    max_inserter_tier,
                );
                // Positional (far=y_cursor, near=y_cursor+1) mapped back to
                // `solid_inputs`' natural order by item identity, since
                // reassignment may have swapped which physical belt each
                // item lives on — same pattern the HorizontalStack branch
                // above already uses.
                let input_ys: Vec<i32> = solid_inputs
                    .iter()
                    .map(|f| if f.item == far_item { y_cursor } else { y_cursor + 1 })
                    .collect();
                let out_y = y_cursor + rh - 1;
                (ents, rh, input_ys, out_y)
            }
        }
        RowKind::SelfLoop { has_minor, has_fluid } => {
            let (mw, mh) = machine_dims(&spec.entity);
            let major = spec
                .self_loop
                .iter()
                .find(|f| f.net_rate > 0.0)
                .expect("self-loop spec must have a net-positive item (classify_self_loop guarantees this)");
            // Solid-only, not `spec.inputs.first()`: with a fluid
            // ingredient present (pentapod-egg's water), the ordinary
            // bus-tapped input must still be the SOLID one regardless of
            // its position relative to the fluid in `spec.inputs`.
            let near = solid_inputs.first().copied();
            let near_item = near.map(|f| f.item.as_str()).unwrap_or("");
            let near_net_rate = near.map(|f| f.rate).unwrap_or(0.0);
            let minor = if *has_minor {
                spec.self_loop
                    .iter()
                    .find(|f| f.net_rate < 0.0)
                    .map(|f| (f.consumed_rate, f.produced_rate))
            } else {
                None
            };
            let fluid_in = fluid_inputs.first().map(|f| (f.item.as_str(), f.rate));
            // Utilization scaling for the ladder-eligible (check-visible)
            // rates only — near_item's own inserter and the output
            // side(s). Major's/minor's INPUT demand stays unscaled here
            // too (harmless: their inserters are hardcoded, unaffected
            // by this factor) to avoid touching the existing belt-sizing
            // rates this call site's other locals still depend on.
            let effective_count = spec.count.ceil().max(1.0);
            let utilization = (spec.count / effective_count).min(1.0);
            let (ents, rh, fluid_input_port_pipes) = templates::self_loop_row(
                &spec.recipe,
                &spec.entity,
                mw,
                count,
                y_cursor,
                bus_width,
                &major.item,
                major.consumed_rate,
                major.produced_rate * utilization,
                near_item,
                near_net_rate * utilization,
                minor.map(|(c, p)| (c, p * utilization)),
                fluid_in,
                max_belt_tier,
                max_inserter_tier,
            );
            fluid_port_ys = fluid_input_port_pipes.first().map(|&(_, _, py)| vec![py]).unwrap_or_default();
            fluid_port_pipes = fluid_input_port_pipes;
            // Mirrors `templates::self_loop_row`'s row-offset formulas:
            // the bus tap-off lands on the near belt (dy=3); the row's
            // declared output is major's export tile, on the major
            // collector row (dy_out_ins + 1). `has_fluid` (1-item shape
            // only) inserts a fluid-header row directly above the
            // machine, shifting the machine row (and everything south)
            // down by 1.
            let dy_near = 3;
            let dy_machine = if *has_minor { 6 } else { 5 + if *has_fluid { 1 } else { 0 } };
            let dy_out_ins = dy_machine + mh as i32;
            let dy_major_collect = dy_out_ins + 1;
            let input_ys = vec![y_cursor + dy_near];
            let out_y = y_cursor + dy_major_collect;
            (ents, rh, input_ys, out_y)
        }
        RowKind::Voider => {
            // Voider specs (`bus::voider::synthesize_voiders`) carry
            // exactly one input: the surplus item, at the PER-MACHINE
            // tap rate (matches every other `MachineSpec`'s convention
            // — total = rate * count). The recirculated (far-belt) rate
            // isn't threaded through `MachineSpec` — it's re-derived
            // here from the same recipe data `synthesize_voiders` used,
            // via the shared `bus::voider::size_self_voider` sizing
            // function, so the two call sites can't drift out of sync.
            let item = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("");
            let near_rate_per_machine = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0);
            let near_total = near_rate_per_machine * count as f64;
            let far_rate = crate::bus::voider::size_self_voider(item, near_total)
                .map(|sizing| crate::bus::voider::far_rate_per_machine(&sizing, near_total))
                .unwrap_or(0.0);
            let (ents, rh) = templates::voider_row(
                &spec.recipe,
                item,
                count,
                y_cursor,
                bus_width,
                near_rate_per_machine,
                far_rate,
                max_belt_tier,
                max_inserter_tier,
            );
            // Mirrors `templates::voider_row`'s row-offset constants:
            // near/tap belt at dy=6 (bus tap-off lands here), far/recirc
            // belt at dy=7 (last row — used as a placeholder
            // `output_belt_y`; the row declares no real bus output).
            let input_ys = vec![y_cursor + 6];
            let out_y = y_cursor + 7;
            (ents, rh, input_ys, out_y)
        }
        RowKind::ScrapRecycling => {
            // Sushi sorter: one east-flowing output belt PER solid output
            // (`sorted_output_belts`), consumed by the ordinary lane
            // planner (items with consumers) and step-7b merger (surplus).
            // See `templates::scrap_recycling_row`.
            let input_item = solid_inputs.first().map(|f| f.item.as_str()).unwrap_or("scrap");
            let input_total = solid_inputs.first().map(|f| f.rate * count as f64).unwrap_or(0.0);
            let sorted_items: Vec<(String, f64)> = solid_outputs
                .iter()
                .map(|f| (f.item.clone(), f.rate * count as f64))
                .collect();
            let (ents, rh, sorted_belts) = templates::scrap_recycling_row(
                &spec.recipe,
                input_item,
                count,
                y_cursor,
                bus_width,
                input_total,
                &sorted_items,
                max_belt_tier,
                max_inserter_tier,
            );
            sorted_output_belts = sorted_belts;
            // Scrap input belt at dy=0 (the bus tap lands here). The
            // primary `output_belt_y` points at the first solid output's
            // own belt so the row-width scan below (which keys on
            // `output_item` at `output_belt_y`) finds a real east belt.
            let input_ys = vec![y_cursor];
            let out_y = sorted_output_belts
                .iter()
                .find(|(it, _)| it == output_item)
                .map(|(_, y)| *y)
                .unwrap_or(y_cursor + 8);
            (ents, rh, input_ys, out_y)
        }
    };

    // Stamp throughput rates onto row entities based on their carried item.
    let mut row_ents = row_ents;
    {
        let mut item_rates: FxHashMap<&str, f64> = FxHashMap::default();
        for f in &spec.inputs {
            item_rates.insert(&f.item, f.rate * count as f64);
        }
        for f in &spec.outputs {
            item_rates.insert(&f.item, f.rate * count as f64);
        }
        for ent in &mut row_ents {
            if ent.rate.is_some() {
                continue;
            }
            if let Some(item) = &ent.carries {
                if let Some(&r) = item_rates.get(item.as_str()) {
                    ent.rate = Some(r);
                }
            }
        }
    }

    // Fluid-only rows (`RowKind::OilRefinery`) with ≥3 distinct fluid outputs
    // use the staggered 3-trunk staircase template, whose machines are
    // spaced by `templates::fluid_only_row_pitch` rather than plain
    // `machine_size` (issue #277 — see that function's doc comment). Every
    // other row kind packs at `machine_size`. Must agree with the pitch the
    // template actually stamped with, or `default_max` below undercounts
    // the row width by `(count - 1)` tiles.
    //
    // This is a horizontal per-machine pitch (row width along x), so it
    // uses width, not height.
    let msz = machine_dims(&spec.entity).0 as i32;
    let machine_pitch: i32 = if matches!(kind, RowKind::OilRefinery) {
        let distinct_fluid_outputs = fluid_outputs
            .iter()
            .map(|f| f.item.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        templates::fluid_only_row_pitch(msz as u32, distinct_fluid_outputs)
    } else {
        msz
    };
    // Inline-bridge unification: machines pack tight (Strategy A) or
    // with a single 1-tile gap at the anchor (Strategy B,
    // triple_input_row only). The `default_max` below is a fallback
    // for fluid-only rows that emit no output belts and is consumed
    // only by downstream code that doesn't care about the exact gap;
    // hardcoding 0 here is correct for every template that uses it.
    let gap = 0;

    // Scan the emitted row entities for surface belts on the output belt row,
    // carrying the row's (solid) output item. This captures the exact x-range
    // of the exit belt run regardless of which template produced it.
    let (output_belt_x_min, output_belt_x_max) = {
        let mut min_x: Option<i32> = None;
        let mut max_x: Option<i32> = None;
        for ent in &row_ents {
            if ent.y != output_belt_y {
                continue;
            }
            if !matches!(
                ent.name.as_str(),
                "transport-belt" | "fast-transport-belt" | "express-transport-belt"
            ) {
                continue;
            }
            if ent.carries.as_deref() != Some(output_item) {
                continue;
            }
            let is_east_west = matches!(
                ent.direction,
                EntityDirection::East | EntityDirection::West
            );
            if !is_east_west {
                continue;
            }
            min_x = Some(min_x.map_or(ent.x, |m| m.min(ent.x)));
            max_x = Some(max_x.map_or(ent.x, |m| m.max(ent.x)));
        }
        // Fluid-only rows have no solid output belts. Default to the machine
        // x-range so downstream code gets sane values; nothing actually
        // consumes x_min/x_max for fluid rows.
        let default_min = bus_width;
        let default_max = bus_width + count as i32 * machine_pitch + gap - 1;
        (
            min_x.unwrap_or(default_min),
            max_x.unwrap_or(default_max),
        )
    };

    // Row width is the exclusive east-end of the output-belt run. The
    // naive `bus_width + count*pitch + gap` undercounts HorizontalStack
    // rows, which add per-block dive columns; the merger then leaves
    // the HS template's east-most belt dead-ending into empty tiles.
    let row_width = output_belt_x_max + 1;

    // Inherit module_id from the spec's primary solid output. Under
    // Pooled this is always 0; under PartitionedDecomposed the
    // partitioner has tagged the spec's outputs with the module index.
    let module_id = spec
        .outputs
        .iter()
        .find(|o| !o.is_fluid)
        .map(|o| o.module_id)
        .unwrap_or(0);
    let span = RowSpan {
        y_start: y_cursor,
        y_end: y_cursor + row_h,
        spec: spec.clone(),
        machine_count: count,
        module_id,
        input_belt_y: input_belt_ys,
        output_belt_y,
        row_width,
        fluid_port_ys,
        fluid_port_pipes,
        fluid_output_port_pipes,
        output_east,
        output_belt_x_min,
        output_belt_x_max,
        horizontal_stack,
        secondary_output_belt,
        sorted_output_belts,
    };

    (row_ents, span, row_width)
}

/// Place assembly rows stacked vertically.
///
/// When a recipe needs more machines than a single belt can handle,
/// the row is split into multiple sub-rows.
///
/// `extra_gap_after_row` maps a row index (into the `row_spans` returned by
/// an EARLIER call) to extra tile rows to insert south of that row.
///
/// Returns `(entities, row_spans, total_width, total_height)`.
pub fn place_rows(
    machines: &[MachineSpec],
    dependency_order: &[String],
    bus_width: i32,
    y_offset: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    final_output_items: Option<&FxHashSet<String>>,
    extra_gap_after_row: Option<&FxHashMap<usize, i32>>,
    row_layout: RowLayout,
) -> (Vec<PlacedEntity>, Vec<RowSpan>, i32, i32) {
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut row_spans: Vec<RowSpan> = Vec::new();
    let mut y_cursor = y_offset;
    let mut max_width: i32 = 0;

    let ordered = order_specs(machines, dependency_order);
    let empty_final: FxHashSet<String> = FxHashSet::default();
    let final_items = final_output_items.unwrap_or(&empty_final);
    let empty_gaps: FxHashMap<usize, i32> = FxHashMap::default();
    let extra_gaps = extra_gap_after_row.unwrap_or(&empty_gaps);

    for (spec_idx, spec) in ordered.iter().enumerate() {
        if spec_idx > 0 {
            y_cursor += 2; // gap between recipes for lane balancers
        }
        // Snap to nearest integer when within float drift of one — solver
        // math accumulates ulps in recursive ratio chains, so a recipe that
        // logically needs N machines often arrives as N + 1ulp, which a
        // naive ceil() would bump to N+1. The over-count silently wastes
        // a machine in most rows, and trips template assertions in others
        // (#277 utility-science-pack: 1.0000000000000002 advanced-oil-
        // processing → machine_count=2 → staggered-3-output panic).
        let total_count = {
            let c = spec.count;
            let snapped = if (c - c.round()).abs() < 1e-9 { c.round() } else { c.ceil() };
            (snapped as usize).max(1)
        };

        let solid_inputs_count = spec.inputs.iter().filter(|f| !f.is_fluid).count();
        let first_solid_output_rate = spec
            .outputs
            .iter()
            .find(|f| !f.is_fluid)
            .map(|f| f.rate)
            .unwrap_or(0.0);
        let output_rate = first_solid_output_rate * total_count as f64;
        let has_fluid = spec.inputs.iter().any(|f| f.is_fluid);

        // Row kinds whose templates do NOT emit a `sideload_bridge` stay
        // on single-lane output math. FluidInput on chemical-plant DOES
        // have a bridge (see `fluid_input_row`), so it joins the dual-lane
        // branch; other fluid row shapes (FluidDualInput, FluidMultiInput,
        // AM2-with-fluid FluidInput path) and triple-solid rows stay
        // single-lane until their templates grow bridges.
        let kind = row_kind(spec);
        let output_is_fluid = spec.outputs.iter().all(|f| f.is_fluid) && !spec.outputs.is_empty();
        // Multi-solid-output rows (RFP Fulgora D2b) never lane-split —
        // see the matching guard + comment in `can_lane_split`. Keep
        // this in sync with that function so `single_lane`'s belt-cap
        // math agrees with whether the template actually stamps a
        // bridge.
        let multi_solid_output = spec.outputs.iter().filter(|f| !f.is_fluid).count() >= 2;
        let has_bridge_template = !multi_solid_output
            && (matches!(
                kind,
                RowKind::SingleInput | RowKind::DualInput | RowKind::TripleInput
            ) || (matches!(kind, RowKind::FluidInput) && spec.entity == "chemical-plant")
                || (matches!(kind, RowKind::FluidDualInput) && !output_is_fluid)
                || (matches!(kind, RowKind::FluidMultiInput) && !output_is_fluid));
        let single_lane = !has_bridge_template;
        let _ = has_fluid;
        let _ = solid_inputs_count;
        let is_hs_dual = matches!(row_layout, RowLayout::HorizontalStack)
            && matches!(kind, RowKind::DualInput);
        let max_per_row = if single_lane {
            let ob = belt_entity_for_rate(output_rate * 2.0, max_belt_tier);
            max_machines_for_belt(spec, ob, max_belt_tier)
        } else if is_hs_dual {
            // HS feeds input₀ via K stacked trunks, so only output and
            // input₁ constrain machines per row.
            let ob = belt_entity_for_rate(output_rate, max_belt_tier);
            max_machines_for_belt_horizontal_stack(spec, ob, max_belt_tier)
        } else {
            let ob = belt_entity_for_rate(output_rate, max_belt_tier);
            max_machines_for_belt_both_lanes(spec, ob, max_belt_tier)
        };

        let is_final = spec
            .outputs
            .iter()
            .any(|o| !o.is_fluid && final_items.contains(o.item.as_str()));

        // Split into evenly-sized chunks driven by `max_per_row` —
        // the per-row machine cap that keeps each row's output rate
        // within its output belt's capacity. Applies uniformly to VS
        // and HS; HS rows that exceed the cap simply split into
        // multiple HS sub-rows (each with its own K-trunk stack at
        // the top and its own lane-balanced output belt).
        let n_rows = ((total_count as f64) / (max_per_row as f64)).ceil() as usize;
        if n_rows > 1 {
            crate::trace::emit(crate::trace::TraceEvent::RowSplit {
                recipe: spec.recipe.clone(),
                original_count: total_count,
                split_into: n_rows,
                reason: format!("max_per_row={max_per_row}, output_rate={output_rate:.1}/s"),
            });
        }
        let mut remaining = total_count;

        for ri in 0..n_rows {
            let chunk = ((remaining as f64) / (n_rows - ri) as f64).ceil() as usize;
            let (row_ents, span, width) = build_one_row(
                spec,
                chunk,
                bus_width,
                y_cursor,
                max_belt_tier,
                max_inserter_tier,
                is_final,
                row_layout,
            );
            let row_idx = row_spans.len();
            max_width = max_width.max(width);
            let y_end = span.y_end;
            entities.extend(row_ents);
            row_spans.push(span);
            y_cursor = y_end + extra_gaps.get(&row_idx).copied().unwrap_or(0);
            remaining -= chunk;
        }
    }

    crate::trace::emit(crate::trace::TraceEvent::RowsPlaced {
        rows: row_spans.iter().enumerate().map(|(i, rs)| crate::trace::RowInfo {
            index: i,
            recipe: rs.spec.recipe.clone(),
            machine: rs.spec.entity.clone(),
            machine_count: rs.machine_count,
            y_start: rs.y_start,
            y_end: rs.y_end,
            row_kind: format!("{:?}", row_kind(&rs.spec)),
        }).collect(),
    });

    (entities, row_spans, max_width, y_cursor)
}

/// Convenience wrapper that takes a `SolverResult` directly.
pub fn place_rows_from_result(
    result: &SolverResult,
    bus_width: i32,
    y_offset: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    final_output_items: Option<&FxHashSet<String>>,
    extra_gap_after_row: Option<&FxHashMap<usize, i32>>,
    row_layout: RowLayout,
) -> (Vec<PlacedEntity>, Vec<RowSpan>, i32, i32) {
    place_rows(
        &result.machines,
        &result.dependency_order,
        bus_width,
        y_offset,
        max_belt_tier,
        max_inserter_tier,
        final_output_items,
        extra_gap_after_row,
        row_layout,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ItemFlow;

    fn iron_plate_spec() -> MachineSpec {
        MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "iron-plate".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![ItemFlow {
                item: "iron-ore".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        }
    }

    fn iron_gear_spec() -> MachineSpec {
        MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "iron-gear-wheel".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 2.0,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        }
    }

    fn electronic_circuit_solver_result() -> SolverResult {
        // electronic-circuit needs copper-cable and iron-plate
        // copper-cable needs copper-plate
        // Rates are approximate but structure mirrors Python's solver output
        SolverResult {
            machines: vec![
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "electronic-circuit".to_string(),
                    self_loop: vec![], voider: false,
                    count: 3.0,
                    inputs: vec![
                        ItemFlow {
                            item: "iron-plate".to_string(),
                            rate: 1.0,
                            is_fluid: false,
                            module_id: 0,
                        },
                        ItemFlow {
                            item: "copper-cable".to_string(),
                            rate: 3.0,
                            is_fluid: false,
                            module_id: 0,
                        },
                    ],
                    outputs: vec![ItemFlow {
                        item: "electronic-circuit".to_string(),
                        rate: 1.5,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "assembling-machine-2".to_string(),
                    recipe: "copper-cable".to_string(),
                    self_loop: vec![], voider: false,
                    count: 3.0,
                    inputs: vec![ItemFlow {
                        item: "copper-plate".to_string(),
                        rate: 1.5,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "copper-cable".to_string(),
                        rate: 3.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "electric-furnace".to_string(),
                    recipe: "iron-plate".to_string(),
                    self_loop: vec![], voider: false,
                    count: 1.0,
                    inputs: vec![ItemFlow {
                        item: "iron-ore".to_string(),
                        rate: 1.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "iron-plate".to_string(),
                        rate: 1.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
                MachineSpec {
                    entity: "electric-furnace".to_string(),
                    recipe: "copper-plate".to_string(),
                    self_loop: vec![], voider: false,
                    count: 2.0,
                    inputs: vec![ItemFlow {
                        item: "copper-ore".to_string(),
                        rate: 2.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                    outputs: vec![ItemFlow {
                        item: "copper-plate".to_string(),
                        rate: 2.0,
                        is_fluid: false,
                        module_id: 0,
                    }],
                },
            ],
            external_inputs: vec![
                ItemFlow {
                    item: "iron-ore".to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                },
                ItemFlow {
                    item: "copper-ore".to_string(),
                    rate: 2.0,
                    is_fluid: false,
                    module_id: 0,
                },
            ],
            external_outputs: vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: 1.5,
                is_fluid: false,
                module_id: 0,
            }],
            surplus_outputs: vec![],
            dependency_order: vec![
                "iron-plate".to_string(),
                "copper-plate".to_string(),
                "copper-cable".to_string(),
                "electronic-circuit".to_string(),
            ],
        }
    }

    // ---- max_machines_for_belt tests ----

    #[test]
    fn max_machines_single_output_yellow_belt() {
        // rate=1.0/machine, lane_cap=7.5 → floor(7.5/1.0)=7 machines
        let spec = iron_plate_spec();
        assert_eq!(max_machines_for_belt(&spec, "transport-belt", None), 7);
    }

    #[test]
    fn max_machines_both_lanes_doubles_capacity() {
        // per_lane = floor(7.5 / 1.0) = 7, both lanes = 14
        let spec = iron_plate_spec();
        assert_eq!(
            max_machines_for_belt_both_lanes(&spec, "transport-belt", None),
            14
        );
    }

    #[test]
    fn max_machines_capped_at_one() {
        // rate > lane_cap → floor < 1 → clamped to 1
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "test".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![],
            outputs: vec![ItemFlow {
                item: "heavy-item".to_string(),
                rate: 100.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert_eq!(max_machines_for_belt(&spec, "transport-belt", None), 1);
    }

    #[test]
    fn test_max_machines_red_belt() {
        // rate=1.0/machine, lane_cap=15.0 → floor(15.0/1.0)=15 machines
        let spec = iron_plate_spec();
        assert_eq!(max_machines_for_belt(&spec, "fast-transport-belt", None), 15);
    }

    #[test]
    fn test_max_machines_blue_belt() {
        // rate=1.0/machine, lane_cap=22.5 → floor(22.5/1.0)=22 machines
        let spec = iron_plate_spec();
        assert_eq!(max_machines_for_belt(&spec, "express-transport-belt", None), 22);
    }

    #[test]
    fn test_max_machines_both_lanes_red_belt() {
        // Output (both lanes): floor(15.0 / 1.0) * 2 = 30
        // Input (both lanes, max_belt_tier=None → blue cap 22.5): floor(22.5 / 1.0) * 2 = 44
        // Output is the bottleneck → 30
        let spec = iron_plate_spec();
        assert_eq!(
            max_machines_for_belt_both_lanes(&spec, "fast-transport-belt", None),
            30
        );
    }

    // ---- order_specs tests ----

    #[test]
    fn order_specs_producer_before_consumer() {
        let machines = vec![iron_gear_spec(), iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let ordered = order_specs(&machines, &dep_order);
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].recipe, "iron-plate");
        assert_eq!(ordered[1].recipe, "iron-gear-wheel");
    }

    #[test]
    fn order_specs_tiebreak_by_dependency_order() {
        // Two unrelated recipes — should follow reversed dependency_order
        let spec_a = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "recipe-a".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![],
            outputs: vec![ItemFlow {
                item: "item-a".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        let spec_b = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "recipe-b".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![],
            outputs: vec![ItemFlow {
                item: "item-b".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        let machines = vec![spec_a, spec_b];
        // dependency_order: a then b → reversed: b then a → rank: b=0, a=1
        // → a should come after b
        let dep_order = vec!["recipe-a".to_string(), "recipe-b".to_string()];
        let ordered = order_specs(&machines, &dep_order);
        assert_eq!(ordered[0].recipe, "recipe-b");
        assert_eq!(ordered[1].recipe, "recipe-a");
    }

    // ---- place_rows tests ----

    #[test]
    fn place_rows_single_recipe_no_split() {
        let machines = vec![iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), None, None, RowLayout::default());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].machine_count, 1);
        assert_eq!(spans[0].spec.recipe, "iron-plate");
    }

    #[test]
    fn place_rows_two_recipes_ordered() {
        let machines = vec![iron_gear_spec(), iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), None, None, RowLayout::default());
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].spec.recipe, "iron-plate");
        assert_eq!(spans[1].spec.recipe, "iron-gear-wheel");
    }

    #[test]
    fn place_rows_gap_between_recipes() {
        // Second recipe starts at y_end_of_first + 2 (gap)
        let machines = vec![iron_plate_spec(), iron_gear_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), None, None, RowLayout::default());
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[1].y_start, spans[0].y_end + 2);
    }

    #[test]
    fn place_rows_y_offset() {
        let machines = vec![iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 5, None, InserterTier::default(), None, None, RowLayout::default());
        assert_eq!(spans[0].y_start, 5);
    }

    /// Done-when criterion: electronic-circuit solver result produces correct row grouping.
    ///
    /// The Python placer groups: copper-plate, iron-plate, copper-cable, electronic-circuit
    /// (4 rows for 4 recipes, no splitting needed at small counts).
    #[test]
    fn place_rows_electronic_circuit_row_grouping() {
        let result = electronic_circuit_solver_result();
        let (_, spans, _, _) = place_rows(
            &result.machines,
            &result.dependency_order,
            0,
            1,
            None,
            InserterTier::default(),
            None,
            None,
            RowLayout::default(),
        );

        // 4 distinct recipes → 4 rows (no splitting for these small counts)
        assert_eq!(
            spans.len(),
            4,
            "Expected 4 rows for electronic-circuit, got {}",
            spans.len()
        );

        // Producer recipes come before consumers
        let recipe_order: Vec<&str> = spans.iter().map(|s| s.spec.recipe.as_str()).collect();
        let ec_pos = recipe_order.iter().position(|&r| r == "electronic-circuit").unwrap();
        let cc_pos = recipe_order.iter().position(|&r| r == "copper-cable").unwrap();
        let ip_pos = recipe_order.iter().position(|&r| r == "iron-plate").unwrap();
        let cp_pos = recipe_order.iter().position(|&r| r == "copper-plate").unwrap();

        // copper-cable → electronic-circuit (solid dep)
        assert!(cc_pos < ec_pos, "copper-cable should come before electronic-circuit");
        // copper-plate → copper-cable (solid dep)
        assert!(cp_pos < cc_pos, "copper-plate should come before copper-cable");
        // iron-plate → electronic-circuit (solid dep)
        assert!(ip_pos < ec_pos, "iron-plate should come before electronic-circuit");
    }

    #[test]
    fn place_rows_split_when_exceeds_belt_capacity() {
        // 20 iron-plate machines at rate=1.0/each → total 20/s output.
        // Yellow belt lane cap = 7.5/s.
        // Output (both lanes): floor(7.5/1.0)*2 = 14 max.
        // Input (both lanes, straight feed): floor(7.5/1.0)*2 = 14 max.
        // max_per_row = 14. 20 machines → ceil(20/14) = 2 rows.
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "iron-plate".to_string(),
            self_loop: vec![], voider: false,
            count: 20.0,
            inputs: vec![ItemFlow {
                item: "iron-ore".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        let machines = vec![spec];
        let dep_order = vec!["iron-plate".to_string()];
        let (_, spans, _, _) = place_rows(
            &machines,
            &dep_order,
            0,
            0,
            Some("transport-belt"),
            InserterTier::default(),
            None,
            None,
            RowLayout::default(),
        );
        // 20 machines, max_per_row=14 → ceil(20/14) = 2 rows
        assert_eq!(spans.len(), 2, "Expected 2 rows due to belt lane capacity");
        let total: usize = spans.iter().map(|s| s.machine_count).sum();
        assert_eq!(total, 20);
    }

    /// Mirrors the Python test_even_row_splitting test.
    #[test]
    fn even_row_splitting_iron_gear_yellow_belt() {
        // iron-gear-wheel at 10/s with yellow belt constraint
        // This mirrors the Python test_even_row_splitting test
        // With 10 machines of iron-gear (output rate ~0.5/s per machine, total ~5/s):
        // The actual split depends on the spec rates, so we use a synthetic spec
        // that matches what Python's solver produces for iron-gear-wheel at 10/s.
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "iron-gear-wheel".to_string(),
            self_loop: vec![], voider: false,
            count: 16.0, // Forces a 2-row split with yellow belt
            inputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-gear-wheel".to_string(),
                rate: 0.5,
                is_fluid: false,
                module_id: 0,
            }],
        };
        // iron-plate spec (producer)
        let plate_spec = MachineSpec {
            entity: "electric-furnace".to_string(),
            recipe: "iron-plate".to_string(),
            self_loop: vec![], voider: false,
            count: 4.0,
            inputs: vec![ItemFlow {
                item: "iron-ore".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        let machines = vec![spec, plate_spec];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(
            &machines,
            &dep_order,
            0,
            1,
            Some("transport-belt"),
            InserterTier::default(),
            None,
            None,
            RowLayout::default(),
        );

        let gear_rows: Vec<_> = spans
            .iter()
            .filter(|s| s.spec.recipe == "iron-gear-wheel")
            .collect();

        // With 16 machines and yellow belt (both lanes = 14), we expect 2 rows
        if gear_rows.len() == 2 {
            let counts: Vec<usize> = gear_rows.iter().map(|s| s.machine_count).collect();
            assert_eq!(counts[0], counts[1], "Row split should be even: {:?}", counts);
        }
    }

    #[test]
    fn row_span_y_coordinates_are_consistent() {
        let machines = vec![iron_plate_spec(), iron_gear_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, total_height) =
            place_rows(&machines, &dep_order, 5, 0, None, InserterTier::default(), None, None, RowLayout::default());

        // Every span should have y_end > y_start
        for span in &spans {
            assert!(
                span.y_end > span.y_start,
                "y_end ({}) should be > y_start ({})",
                span.y_end,
                span.y_start
            );
        }

        // total_height should be at or above last row's y_end
        let last_y_end = spans.last().map(|s| s.y_end).unwrap_or(0);
        assert!(
            total_height >= last_y_end,
            "total_height {} < last y_end {}",
            total_height,
            last_y_end
        );
    }

    #[test]
    fn row_width_includes_bus_width() {
        let machines = vec![iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string()];
        let bus_width = 10;
        let (_, spans, max_width, _) =
            place_rows(&machines, &dep_order, bus_width, 0, None, InserterTier::default(), None, None, RowLayout::default());

        assert!(
            spans[0].row_width >= bus_width,
            "row_width should be >= bus_width"
        );
        assert_eq!(max_width, spans[0].row_width);
    }

    #[test]
    fn extra_gap_after_row_applied() {
        let machines = vec![iron_plate_spec(), iron_gear_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];

        let mut extra_gaps: FxHashMap<usize, i32> = FxHashMap::default();
        extra_gaps.insert(0, 5); // Add 5 extra tiles after first row

        let (_, spans_with_gap, _, _) = place_rows(
            &machines,
            &dep_order,
            0,
            0,
            None,
            InserterTier::default(),
            None,
            Some(&extra_gaps),
            RowLayout::default(),
        );
        let (_, spans_no_gap, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), None, None, RowLayout::default());

        // Second row should start 5 tiles later with gap
        assert_eq!(
            spans_with_gap[1].y_start,
            spans_no_gap[1].y_start + 5,
            "Extra gap should shift subsequent rows"
        );
    }

    #[test]
    fn single_input_row_kind() {
        let spec = iron_plate_spec();
        assert_eq!(row_kind(&spec), RowKind::SingleInput);
    }

    #[test]
    fn dual_input_row_kind() {
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![
                ItemFlow {
                    item: "iron-plate".to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                },
                ItemFlow {
                    item: "copper-cable".to_string(),
                    rate: 3.0,
                    is_fluid: false,
                    module_id: 0,
                },
            ],
            outputs: vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::DualInput);
    }

    #[test]
    fn fluid_input_row_kind() {
        let spec = MachineSpec {
            entity: "chemical-plant".to_string(),
            recipe: "plastic-bar".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![
                ItemFlow {
                    item: "coal".to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                },
                ItemFlow {
                    item: "petroleum-gas".to_string(),
                    rate: 2.0,
                    is_fluid: true,
                    module_id: 0,
                },
            ],
            outputs: vec![ItemFlow {
                item: "plastic-bar".to_string(),
                rate: 2.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::FluidInput);
    }

    #[test]
    fn oil_refinery_row_kind() {
        let spec = MachineSpec {
            entity: "oil-refinery".to_string(),
            recipe: "basic-oil-processing".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![ItemFlow {
                item: "crude-oil".to_string(),
                rate: 10.0,
                is_fluid: true,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "petroleum-gas".to_string(),
                rate: 4.5,
                is_fluid: true,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::OilRefinery);
    }

    #[test]
    fn heavy_oil_cracking_is_fluid_multi_input() {
        let spec = MachineSpec {
            entity: "chemical-plant".to_string(),
            recipe: "heavy-oil-cracking".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![
                ItemFlow { item: "water".to_string(), rate: 30.0, is_fluid: true, module_id: 0 },
                ItemFlow { item: "heavy-oil".to_string(), rate: 40.0, is_fluid: true, module_id: 0 },
            ],
            outputs: vec![ItemFlow {
                item: "light-oil".to_string(),
                rate: 30.0,
                is_fluid: true,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::FluidMultiInput);
    }

    #[test]
    fn sulfur_is_fluid_multi_input() {
        let spec = MachineSpec {
            entity: "chemical-plant".to_string(),
            recipe: "sulfur".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![
                ItemFlow { item: "water".to_string(), rate: 30.0, is_fluid: true, module_id: 0 },
                ItemFlow { item: "petroleum-gas".to_string(), rate: 30.0, is_fluid: true, module_id: 0 },
            ],
            outputs: vec![ItemFlow {
                item: "sulfur".to_string(),
                rate: 2.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::FluidMultiInput);
    }

    #[test]
    fn foundry_fluid_only_row_kind() {
        // Foundry (5×5) with fluid-only inputs should use OilRefinery template
        let spec = MachineSpec {
            entity: "foundry".to_string(),
            recipe: "molten-iron".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![ItemFlow {
                item: "iron-ore".to_string(),
                rate: 10.0,
                is_fluid: true,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "molten-iron".to_string(),
                rate: 5.0,
                is_fluid: true,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::OilRefinery);
    }

    #[test]
    fn foundry_solid_input_row_kind() {
        // Foundry (5×5) with solid inputs should use SingleInput, not OilRefinery
        let spec = MachineSpec {
            entity: "foundry".to_string(),
            recipe: "iron-plate".to_string(),
            self_loop: vec![], voider: false,
            count: 1.0,
            inputs: vec![ItemFlow {
                item: "iron-ore".to_string(),
                rate: 10.0,
                is_fluid: false,
                module_id: 0,
            }],
            outputs: vec![ItemFlow {
                item: "iron-plate".to_string(),
                rate: 10.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert_eq!(row_kind(&spec), RowKind::SingleInput);
    }

    #[test]
    fn lane_split_applies_for_dual_input() {
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false,
            count: 3.0,
            inputs: vec![
                ItemFlow {
                    item: "iron-plate".to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                },
                ItemFlow {
                    item: "copper-cable".to_string(),
                    rate: 3.0,
                    is_fluid: false,
                    module_id: 0,
                },
            ],
            outputs: vec![ItemFlow {
                item: "electronic-circuit".to_string(),
                rate: 1.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert!(can_lane_split(&spec, 3));
    }

    #[test]
    fn lane_split_applies_to_chemical_plant_fluid_rows() {
        // Chemical-plant fluid rows DO support lane splitting — the template
        // emits a `sideload_bridge` between two machine groups, matching the
        // SingleInput / DualInput row pattern. See the Phase-2 tier4 fix
        // landed to remove the artificial single-lane cap on plastic-bar.
        let spec = MachineSpec {
            entity: "chemical-plant".to_string(),
            recipe: "plastic-bar".to_string(),
            self_loop: vec![], voider: false,
            count: 3.0,
            inputs: vec![
                ItemFlow {
                    item: "coal".to_string(),
                    rate: 1.0,
                    is_fluid: false,
                    module_id: 0,
                },
                ItemFlow {
                    item: "petroleum-gas".to_string(),
                    rate: 2.0,
                    is_fluid: true,
                    module_id: 0,
                },
            ],
            outputs: vec![ItemFlow {
                item: "plastic-bar".to_string(),
                rate: 2.0,
                is_fluid: false,
                module_id: 0,
            }],
        };
        assert!(can_lane_split(&spec, 3));
    }

    #[test]
    fn lane_split_applies_to_am2_with_fluid() {
        // AM2/AM3 with a fluid input now uses the same unified T-junction
        // row template as chemical-plant, so it gains lane-split support
        // too. The template parameterises `port_dx` (1 for AM2/AM3 vs 0
        // for chemical-plant) so the UG pair lands on the correct column
        // and the inserter sits on a free column.
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "example".to_string(),
            self_loop: vec![], voider: false,
            count: 3.0,
            inputs: vec![
                ItemFlow { item: "widget".to_string(), rate: 1.0, is_fluid: false, module_id: 0 },
                ItemFlow { item: "lubricant".to_string(), rate: 2.0, is_fluid: true, module_id: 0 },
            ],
            outputs: vec![ItemFlow { item: "thing".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
        };
        assert!(can_lane_split(&spec, 3));
    }

    /// Pre-fix bug: `recipe_to_spec: HashMap<recipe, &MachineSpec>` silently
    /// dropped duplicate-recipe entries on insert (last-write-wins), and the
    /// final `filter_map` lookup returned only the surviving one. Result: if a
    /// strategy produced N siblings sharing a recipe (the partitioner does
    /// this for items with multiple consumers), only one made it through
    /// `order_specs` and the placer placed only that one's machines.
    #[test]
    fn order_specs_preserves_duplicate_recipes() {
        let cable = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "copper-cable".to_string(),
            self_loop: vec![], voider: false,
            count: 4.0,
            inputs: vec![ItemFlow { item: "copper-plate".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            outputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
        };
        let ec_a = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false,
            count: 5.0,
            inputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 3.0, is_fluid: false, module_id: 0 }],
            outputs: vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
        };
        let ec_b = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false,
            count: 7.0,
            inputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 3.0, is_fluid: false, module_id: 1 }],
            outputs: vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
        };
        let machines = vec![cable, ec_a, ec_b];
        let dep_order: Vec<String> = vec!["copper-cable".into(), "electronic-circuit".into()];
        let ordered = order_specs(&machines, &dep_order);

        assert_eq!(ordered.len(), 3, "all input specs must be preserved through topo sort");
        assert_eq!(ordered[0].recipe, "copper-cable");
        assert_eq!(ordered[1].recipe, "electronic-circuit");
        assert_eq!(ordered[2].recipe, "electronic-circuit");
        let mut counts: Vec<usize> = ordered[1..3].iter().map(|s| s.count as usize).collect();
        counts.sort();
        assert_eq!(counts, vec![5, 7], "both EC siblings must appear with their original counts");
    }
}
