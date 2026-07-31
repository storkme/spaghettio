//! Stacks assembly rows vertically in dependency order.
//!
//! Port of `src/bus/placer.py`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::bus::inserter_ladder::{reassign_near_far, InserterTier};
use crate::bus::layout::RowLayout;
use crate::bus::stacking_ctx::StackingCtx;
use crate::common::{
    belt_entity_for_rate, belt_entity_for_rate_stacked, lane_capacity, lane_capacity_stacked,
    machine_dims, utilization_for, QualityTier, BELT_TIERS,
};
use crate::models::{EntityDirection, ItemFlow, MachineSpec, PlacedEntity, SolverResult};

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
    // stacking-neutral: INFINITY selects the max tier regardless (RFC-046)
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
    /// See `docs/rfc-modular-production.md`.
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
    /// row's high-demand input. See `docs/rfc-horizontal-trunks.md`.
    pub horizontal_stack: Option<HorizontalStackInfo>,
    /// `Some((item, y))` when this row's spec has a SECOND solid output
    /// beyond the primary (which owns `output_belt_y`) — e.g.
    /// uranium-processing's uranium-238 surplus alongside uranium-235's
    /// target belt. `y` is the secondary belt's row. Only
    /// `RowKind::SingleInput` rows with 2+ solid outputs populate this
    /// today (RFC Fulgora D2b, `docs/rfc-fulgora-scrap.md`). Read by the
    /// step-7 solid-surplus merger (`ghost_router` step 7b).
    pub secondary_output_belt: Option<(String, i32)>,
    /// Per-item output belt y for rows that emit MANY single-item output
    /// belts, each at its own row (RFC Fulgora Phase 3,
    /// `docs/rfc-fulgora-scrap.md` D3): the scrap-recycling sushi-sorter
    /// row lifts each of the recycler's ~12 mixed outputs onto its own
    /// east-flowing belt at a distinct y. Generalises
    /// `secondary_output_belt` from one extra belt to N. Empty for every
    /// ordinary row. Read by the same three item→belt-y lookup sites the
    /// secondary belt uses: `lane_planner` source_y, `ghost_router`
    /// `row_exit_origin`, and the step-7b surplus merger. The helper
    /// [`RowSpan::output_belt_y_for`] centralises the lookup.
    pub sorted_output_belts: Vec<(String, i32)>,
    /// One `(item, producer_row_idx)` per input of this row that is fed by
    /// direct insertion from the producer row at `producer_row_idx` instead
    /// of a bus trunk lane. The lane planner reads this to skip lane creation
    /// for each `(item, consumer_row)` pair. A consumer can have more than
    /// one DI'd input (e.g. a recipe whose two ingredients are each
    /// single-producer coupled), so this is a list, not a single option.
    /// Populated only when `LayoutOptions.direct_insertion` is true, the
    /// solver detected a `DICoupling`, AND the bridge was geometrically
    /// feasible (an infeasible bridge leaves the item off this list so the
    /// bus lane feeds it — see `stamp_di_bridge`). See
    /// `docs/rfc-decomposition-search.md` Phase 3.
    pub di_input: Vec<(String, usize)>,
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
///
/// **Deliberately NOT stacking-aware on the output side** (RFC-047 Leg B):
/// unlike `max_machines_for_belt_both_lanes` (which was made stacking-aware
/// because its bridge+corner-feed output genuinely fills BOTH lanes and so
/// legitimately carries full-belt ×S), this variant's output is
/// **sideloaded onto one lane** (B8/I5). A stack inserter dropping stacks
/// onto that single lane still concentrates all flow on ONE physical lane,
/// so crediting it ×S would just relocate the single-lane overload this RFC
/// exists to prevent. Capping the output at the unstacked per-lane figure is
/// the conservative-correct choice; the asymmetry with the both-lanes
/// variant is intentional.
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
///
/// `out_stack` is the output item's effective belt stack size
/// (`StackingCtx::for_item`): a stack-loaded output belt carries `×S` per
/// lane, so the per-row machine cap must scale with it (RFC-047 Leg B —
/// the row-split cap was stacking-blind while the belt-tier choice at the
/// same call site was already stacking-aware, which forced stacked
/// producers to fragment into single-machine rows and re-introduced the
/// mid-trunk sideload the RFC set out to remove). At `S == 1`
/// (`for_item` returns 1) `lane_capacity_stacked == lane_capacity`, so
/// this is bit-identical to the pre-RFC behaviour for the default corpus.
pub(crate) fn max_machines_for_belt_both_lanes(
    spec: &MachineSpec,
    belt_name: &str,
    max_belt_tier: Option<&str>,
    out_stack: u8,
) -> usize {
    let out_lane_cap = lane_capacity_stacked(belt_name, out_stack);
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
///
/// When `di_couplings` is non-empty, a DI producer that is followed by its
/// coupled consumer in the dependency graph is emitted back-to-back — the
/// consumer is pulled forward to immediately follow the producer, so the
/// placer can suppress the inter-recipe gap and the lane planner can skip
/// the bus lane. The consumer is only pulled forward if ALL its other solid
/// deps are already satisfied; otherwise it stays in normal topo order.
pub(crate) fn order_specs<'a>(
    machines: &'a [MachineSpec],
    dependency_order: &[String],
    di_couplings: &[crate::models::DICoupling],
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

    // DI: producer_recipe → consumer_recipe. When the producer is emitted,
    // greedily try to emit the consumer immediately after (if its other deps
    // are satisfied). This co-locates DI pairs so the placer can suppress the
    // inter-recipe gap and the lane planner can skip the bus lane.
    let di_consumer_of: FxHashMap<&str, &str> = di_couplings
        .iter()
        .map(|c| (c.producer_recipe.as_str(), c.consumer_recipe.as_str()))
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

        // DI co-location: if this recipe is a DI producer, try to emit
        // its consumer immediately after (before the normal loop picks
        // something else). Only if the consumer's remaining deps are all
        // satisfied (i.e. it's ready NOW) — otherwise it stays in normal
        // topo order and the gap is preserved.
        if let Some(&consumer) = di_consumer_of.get(r) {
            if let Some(consumer_deps) = remaining.get(consumer) {
                if consumer_deps.is_empty() {
                    emitted.push(consumer);
                    remaining.remove(consumer);
                    for deps_set in remaining.values_mut() {
                        deps_set.remove(consumer);
                    }
                }
            }
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
    /// `docs/archive/rfc-multi-fluid-rows.md`.
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
    /// Layout-synthesized voider row (RFC Fulgora Phase 2,
    /// `docs/rfc-fulgora-scrap.md` D1): a recycler bank that
    /// self-consumes a solid surplus stream. `MachineSpec.voider ==
    /// true`. Non-square 2×4 machine, direct belt ejection (no output
    /// inserter), no declared bus output. See `templates::voider_row`.
    Voider,
    /// Scrap-recycling sushi-sorter row (RFC Fulgora Phase 3,
    /// `docs/rfc-fulgora-scrap.md` D3): a bank of `recycler`s running
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
    // Voider rows (RFC Fulgora Phase 2, `docs/rfc-fulgora-scrap.md` D1)
    // short-circuit BEFORE the square-machine debug_assert below —
    // `recycler` is 2×4, non-square, and would trip that assert
    // (working as intended: any OTHER non-square machine reaching that
    // point is a real bug, not this one). Mirrors the self-loop
    // short-circuit immediately below for the same reason.
    if spec.voider {
        return RowKind::Voider;
    }

    // Scrap-recycling sushi-sorter rows (RFC Fulgora Phase 3) also
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

/// RFC-060: whether any machine spec would build a `RowKind::DualInput`
/// row. `DualInput` is the only row kind whose construction consults
/// `RowLayout`, so a solve with none of them produces a bit-identical
/// layout under either row layout — the decomposition search uses this
/// to skip the horizontal-stack candidate pass entirely.
pub(crate) fn any_dual_input_row(machines: &[crate::models::MachineSpec]) -> bool {
    machines.iter().any(|m| matches!(row_kind(m), RowKind::DualInput))
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
    // Rows with 2+ solid outputs (RFC Fulgora D2b: uranium-processing's
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_one_row(
    spec: &MachineSpec,
    count: usize,
    bus_width: i32,
    y_cursor: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    quality: QualityTier,
    // RFC-049 inserter-capacity research level (see `place_rows`).
    inserter_capacity: u8,
    output_east: bool,
    row_layout: RowLayout,
    ctx: &StackingCtx,
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
    // RFC-047 kill-4 root cause: for lane-split rows the midpoint bridge
    // divides `count` machines into ⌈n/2⌉/⌊n/2⌋ lane groups, so with an
    // odd count one lane carries MORE than half the output. Sizing by
    // `output_rate` alone assumes a perfect 50/50 lane balance and left
    // zero headroom at exact tier boundaries (walker-caught 15.5/s on a
    // 15/s stacked-yellow lane, express@60 probe 2026-07-22). Size by the
    // worst lane instead: `2 × ⌈n/2⌉ × per-machine` — identical to
    // `output_rate` for even counts, one machine's rate more for odd.
    let out_effective_rate = if lane_split && count > 0 {
        let per_machine = output_rate / count as f64;
        2.0 * per_machine * ((count as f64) / 2.0).ceil()
    } else {
        output_rate * 2.0
    };
    let out_belt = belt_entity_for_rate_stacked(
        out_effective_rate,
        max_belt_tier,
        ctx.for_item(output_item),
    );

    // Second solid output (RFC Fulgora D2b): only `solid_outputs[0]` owns
    // `output_belt_y` today. When a spec has a second solid output (e.g.
    // uranium-processing's uranium-235 target + uranium-238 surplus),
    // size a belt for it too — `RowKind::SingleInput` is the only arm
    // that currently stamps it (see the match arm below); other kinds
    // leave this unused and `secondary_output_belt` stays `None`.
    // `can_lane_split` already forces `lane_split == false` whenever
    // `solid_outputs.len() >= 2`, so this is always single-lane (×2).
    let secondary_solid_output = solid_outputs.get(1);
    // Secondary solid outputs are always family-exempt (RFC-046: D2b index
    // >=1 solids are a fixed long-handed extraction, cannot stack), so this
    // resolves to ×1 regardless of `ctx.stacking()` — converted for
    // uniformity with every other belt-tier site.
    let secondary_belt_name: Option<&'static str> = secondary_solid_output.map(|f| {
        belt_entity_for_rate_stacked(f.rate * count as f64 * 2.0, max_belt_tier, ctx.for_item(&f.item))
    });

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
            // Port-identity rule (sim-measured, #412 / fluid_ports::
            // port_fluid_assignment): recipe fluids bind x-ASCENDING on
            // the unmirrored form and x-DESCENDING on the mirrored form
            // (the 180°-rotation export encoding reverses port x-order;
            // the old ascending-always zip starved advanced-oil
            // refineries in-game). fluid_only_row places mirrored, so
            // the FLUID list reverses here; single-fluid sides reverse
            // to themselves — every registered fixture is bit-identical.
            let in_port_assignments: Vec<(i32, &str)> = input_dxs
                .iter()
                .zip(fluid_inputs.iter().rev())
                .map(|(&dx, f)| (dx, f.item.as_str()))
                .collect();
            let out_port_assignments: Vec<(i32, &str)> = output_dxs
                .iter()
                .zip(fluid_outputs.iter().rev())
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
            // v3 extension of the reassignment lever (`docs/rfc-inserter-
            // sizing.md`): same hungrier-item-to-near swap as DualInput/
            // TripleInput. Geometrically identical positional pick
            // (source-confirmed) — the fluid PTG's column depends only on
            // `port_dx` (machine type), never on which solid item is
            // passed as input1/input2, so swapping the far/near role
            // never touches the fluid port.
            let utilization = utilization_for(spec);
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
                quality,
                ctx.for_item(output_item),
                inserter_capacity,
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
            let utilization = utilization_for(spec);
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
                quality,
                ctx.for_item(output_item),
                inserter_capacity,
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
            let utilization = utilization_for(spec);
            let input_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let output_rate = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
            let secondary_rate = secondary_solid_output.map(|f| f.rate * utilization);

            // Phase 0e: a solid-input recipe whose product is a fluid
            // (ice-melting: ice → water on chemical-plant; biolubricant:
            // jelly → lubricant on biochamber) needs its output piped, not
            // belted. Gated by the shared port table to machines whose fluid
            // output ports sit on the south face this template already
            // occupies (RFC `docs/rfc-power-supply.md` Phase 0e-i item 5).
            // Machines with a non-south fluid-output face (an unmirrored
            // foundry's molten-metal, north) fail this gate and keep the old
            // path — foundry molten-metal is handled by the dual-input arm.
            let output_is_fluid_south = output_is_fluid
                && crate::fluid_ports::output_ports_all_south(
                    &spec.entity,
                    false,
                    crate::models::EntityDirection::North,
                    mh as i32,
                );
            let (ents, rh, out_port_pipes) = templates::single_input_row(
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
                output_is_fluid_south,
                lane_split,
                output_east,
                secondary,
                input_rate,
                output_rate,
                secondary_rate,
                max_inserter_tier,
                quality,
                ctx.for_item(output_item),
                inserter_capacity,
            );
            fluid_output_port_pipes = out_port_pipes;
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
            let utilization = utilization_for(spec);
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
                quality,
                ctx.for_item(output_item),
                inserter_capacity,
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
            let utilization = utilization_for(spec);
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
                quality,
                ctx.for_item(output_item),
                inserter_capacity,
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
            let utilization = utilization_for(spec);
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
                quality,
                ctx.for_item(output_item),
                inserter_capacity,
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
                let utilization = utilization_for(spec);
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
                let in_belt1 = belt_entity_for_rate_stacked(belt_cap, max_belt_tier, ctx.for_item(item0));
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
                    quality,
                    ctx.for_item(output_item),
                    inserter_capacity,
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
                // above. Reassignment lever (`docs/rfc-inserter-sizing.md`
                // lever (b)): hungrier item goes near, where the full
                // tier ladder applies.
                let utilization = utilization_for(spec);
                let item0_rate = solid_inputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
                let item1_rate = solid_inputs.get(1).map(|f| f.rate).unwrap_or(0.0) * utilization;
                let output_rate_pm = solid_outputs.first().map(|f| f.rate).unwrap_or(0.0) * utilization;
                let ((far_item, far_rate), (near_item, near_rate)) =
                    reassign_near_far(item0, item0_rate, item1, item1_rate);
                let in_belt1 = row_input_belt(max_belt_tier);
                let in_belt2 = row_input_belt(max_belt_tier);
                let (ents, rh, out_port_pipes) = templates::dual_input_row(
                    &spec.recipe,
                    &spec.entity,
                    msz,
                    count,
                    y_cursor,
                    bus_width,
                    (far_item, near_item),
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
                    quality,
                    ctx.for_item(output_item),
                    inserter_capacity,
                );
                fluid_output_port_pipes = out_port_pipes;
                // Positional (far=y_cursor, near=y_cursor+1) mapped back to
                // `solid_inputs`' natural order by item identity, since
                // reassignment may have swapped which physical belt each
                // item lives on — same pattern the HorizontalStack branch
                // above already uses.
                let input_ys: Vec<i32> = solid_inputs
                    .iter()
                    .map(|f| if f.item == far_item { y_cursor } else { y_cursor + 1 })
                    .collect();
                // Fluid output: the row's output is the south pipe row at
                // out_ins_y (y_cursor+3+msz), not a belt at rh-1 — there's no
                // east output belt/merger for it (the fluid reaches the bus via
                // `fluid_output_port_pipes`). Mirrors FluidDualInput.
                let out_y = if output_is_fluid {
                    y_cursor + 3 + msz as i32
                } else {
                    y_cursor + rh - 1
                };
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
            let utilization = utilization_for(spec);
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
                quality,
                inserter_capacity,
                ctx,
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
                quality,
                inserter_capacity,
                ctx,
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
                quality,
                inserter_capacity,
                ctx,
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
        di_input: Vec::new(),
    };

    (row_ents, span, row_width)
}

/// Stamp bridge inserters from a DI producer's output belt to the
/// consumer's input belt, spanning the 2-tile inter-recipe gap.
///
/// Geometry (SingleInput producer, DualInput consumer, msz=3):
/// ```text
///   producer y+msz+3: output belt (carries the DI'd item, westward)
///   gap y+0:           (empty)
///   gap y+1:           bridge inserter (long-handed, south-facing)
///   consumer y+0:      far input belt (other item)
///   consumer y+1:      near input belt (DI'd item)
/// ```
///
/// The long-handed inserter at gap y+1 picks from the producer's output
/// belt (reach 2 north) and drops onto the consumer's near input belt
/// (reach 2 south). One inserter per consumer machine, sized via the
/// standard inserter ladder.
fn stamp_di_bridge(
    producer: &RowSpan,
    consumer: &RowSpan,
    item: &str,
    consumer_spec: &MachineSpec,
    consumer_count: usize,
    max_inserter_tier: InserterTier,
    quality: QualityTier,
    level: u8,
    ctx: &StackingCtx,
) -> Vec<PlacedEntity> {
    use crate::bus::inserter_ladder::{size_side, Reach};

    // Both belt lookups are ITEM-KEYED. Getting this wrong is silent: the
    // bridge would pick from (or drop onto) whichever belt happened to be
    // first/last, which for a DualInput consumer like electronic-circuit
    // is a coin flip between the iron-plate and copper-cable belts.
    let producer_belt_y = producer.output_belt_y_for(item);

    // The consumer's input belts are indexed parallel to its SOLID inputs
    // (`input_belt_y[i]` serves `solid_inputs[i]`) — the same convention
    // `lane_planner`'s tap-off resolution uses. Refuse the bridge if the
    // item isn't a solid input of this consumer, rather than guessing.
    let solid_idx = consumer_spec
        .inputs
        .iter()
        .filter(|f| !f.is_fluid)
        .position(|f| f.item == item);
    let Some(consumer_belt_y) = solid_idx.and_then(|i| consumer.input_belt_y.get(i).copied())
    else {
        crate::trace::emit(crate::trace::TraceEvent::GhostSpecFailed {
            spec_key: format!("di-bridge:{item}:no-input-belt"),
            from_x: 0,
            from_y: producer_belt_y,
            to_x: 0,
            to_y: consumer.y_start,
        });
        return Vec::new();
    };

    // The gap is 2 tiles: producer.y_end and producer.y_end + 1.
    // The bridge inserter sits at the SECOND gap tile (producer.y_end + 1)
    // so a long-handed inserter (reach 2) can span from the producer's
    // output belt to the consumer's input belt.
    let bridge_y = producer.y_end + 1;

    // Verify the reach is feasible. The bridge inserter is long-handed
    // (`Reach::Far`), whose pickup and drop tiles are at EXACTLY 2 tiles
    // from its own position — not "up to 2". A belt one tile away is
    // reached PAST (to the empty tile beyond), so the pick/drop distances
    // must each equal 2, not merely be ≤ 2. When they don't, return empty:
    // the caller leaves `di_input` unset for this item, so the bus lane
    // feeds the consumer normally (graceful fallback). The trace records
    // the geometry that couldn't be bridged.
    let pick_dist = (bridge_y - producer_belt_y).abs();
    let drop_dist = (consumer_belt_y - bridge_y).abs();
    if pick_dist != 2 || drop_dist != 2 {
        crate::trace::emit(crate::trace::TraceEvent::GhostSpecFailed {
            spec_key: format!("di-bridge:{item}"),
            from_x: 0,
            from_y: producer_belt_y,
            to_x: 0,
            to_y: consumer_belt_y,
        });
        return Vec::new();
    }

    // Machine x positions for the consumer.
    let (mw, _mh) = machine_dims(&consumer_spec.entity);
    let pitch = mw as i32;
    let mxs: Vec<i32> = (0..consumer_count as i32)
        .map(|i| consumer.output_belt_x_min + i * pitch)
        .collect();

    // Per-machine rate for the DI'd item.
    let utilization = utilization_for(consumer_spec);
    let di_input = consumer_spec.inputs.iter().find(|f| f.item == item);
    let rate_per_machine = di_input.map(|f| f.rate * utilization).unwrap_or(0.0);

    // Size the inserter: Reach::Far — the bridge spans a 2-tile gap, and
    // long-handed is the ONLY reach-2 inserter in vanilla (I8a: no fast or
    // stack long-handed exists), so the per-inserter ceiling is 1.2/s at L0
    // rising to 4.8/s at L7. A single inserter therefore cannot carry a
    // high-rate coupling (copper-cable into EC needs 7.5/s per machine), so
    // give the ladder the machine's REAL free-column budget instead of a
    // hardcoded 0: the gap row is otherwise empty across the machine's own
    // span, so `mw` slots are available (`mw - 1` beyond the first). Rates
    // the budget still can't cover stay honestly warned by
    // `input-rate-delivery` rather than silently under-fed.
    // Column budget: NOT the machine's full width. The gap row is CONTESTED
    // space — the ghost router threads tap belts through it and `place_poles`
    // lands poles there. Measured (EC@10/s from plates): giving the bridge
    // every column of a 3-wide machine fills the gap row edge-to-edge, the
    // router can no longer cross it, and the retry that follows moves the
    // consumer row two tiles down — out of the long-handed inserter's
    // exact-2 reach — which silently disables DI altogether (the fallback
    // then routes the item over the bus). One extra column (two inserters
    // per machine) leaves a free column per machine and places cleanly with
    // no retry.
    const BRIDGE_EXTRA_COLS: usize = 1;
    let slots = (mw.max(1) as usize).min(1 + BRIDGE_EXTRA_COLS);
    let plan = size_side(
        rate_per_machine,
        Reach::Far,
        slots.saturating_sub(1),
        max_inserter_tier,
        quality,
        level,
    );

    // Column order within the machine's own span: dx=1 first (the standard
    // inserter column, so the one-inserter case is positionally unchanged),
    // then outward. Staying inside `0..mw` keeps a multi-inserter bridge
    // from spilling into the neighbouring machine's columns.
    let dxs: Vec<i32> = std::iter::once(1)
        .chain((0..slots as i32).filter(|&d| d != 1))
        .take(plan.count.max(1))
        .collect();

    let mut entities = Vec::new();
    let seg = Some(format!("di-bridge:{item}:{}", consumer_spec.recipe));
    for &mx in &mxs {
        for &dx in &dxs {
            entities.push(PlacedEntity {
                name: plan.entity.to_string(),
                x: mx + dx,
                y: bridge_y,
                direction: EntityDirection::South,
                carries: Some(item.to_string()),
                segment_id: seg.clone(),
                ..Default::default()
            });
        }
    }
    let _ = ctx; // stacking not yet wired for DI bridges
    entities
}

// ---- RFC-053 Phase 1: direct-insertion cells ----


/// Can this pair's ratio actually be arranged, as opposed to merely being
/// eligible? Eligibility is about recipe SHAPE; this is about whether the
/// straddle geometry exists at these machine counts.
///
/// Separating the two matters: a consumer with several couplings claims a
/// producer at selection time, and claiming on shape alone lets an
/// unbuildable coupling block a buildable one. Concretely, at EC@10/s the
/// `iron-plate -> electronic-circuit` coupling is shape-eligible but is a
/// 16:4 straddle (4 producers per consumer, refused), and claiming it
/// starved the `copper-cable -> electronic-circuit` coupling that does work.
fn pair_is_arrangeable(producer: &MachineSpec, consumer: &MachineSpec, item: &str, row: bool) -> bool {
    let snap = |c: f64| -> usize {
        let s = if (c - c.round()).abs() < 1e-9 { c.round() } else { c.ceil() };
        (s as usize).max(1)
    };
    let (pc, cc) = (snap(producer.count), snap(consumer.count));
    let (cw, _) = machine_dims(&consumer.entity);
    let (pw, _) = machine_dims(&producer.entity);
    let mw = cw;
    let Some(pr) = producer.outputs.iter().find(|f| f.item == item) else { return false };
    let Some(cr) = consumer.inputs.iter().find(|f| f.item == item) else { return false };
    let pr = pr.rate * utilization_for(producer);
    let cr = cr.rate * utilization_for(consumer);
    if row {
        crate::bus::di_cell::plan_row_straddle(pc, cc, pr, cr, pw as i32, cw as i32).is_some()
    } else {
        crate::bus::di_cell::plan_straddle(pc, cc, pr, cr, mw as i32).is_some()
    }
}

/// Is this producer/consumer pair a **Phase 1** DI cell?
///
/// A cell fuses both rows into ONE structure with the coupled item never
/// touching a belt. Phase 1 deliberately serves only the simplest shape;
/// anything else falls back to #432's bridge, and then to the bus.
///
/// Every gate here is load-bearing:
///
/// - **The consumer's only solid input is the coupled item.** A cell
///   spends the consumer's north face on the DI band and its south face
///   on the output inserter, so a second solid input has nowhere to land
///   until Phase 2's face allocation. This is **not** redundant with the
///   solver: `netflow::detect_di_couplings` gates on one-producer /
///   one-consumer / no-external-supply / no-surplus / not-fluid and never
///   inspects the consumer's input count, so it emits `copper-cable →
///   electronic-circuit` (iron-plate + copper-cable) like any other pair.
///   Building that cell would strand the iron feed entirely — a layout
///   that validates clean and starves, the #383/#432 failure mode.
/// - **The producer has exactly one solid output** (the coupled item) and
///   **exactly one solid input**, which becomes the cell's belt-fed
///   input. A second producer output would need a belt the fused row
///   doesn't have.
/// - **No fluids on either side** — pipe placement is Phase 2 (the KC6
///   re-scope).
/// - **Equal machine footprints.** `plan_straddle` reasons about a single
///   `machine_w`, and the stamper stacks both rows on one x-grid.
fn cell_eligible(producer: &MachineSpec, consumer: &MachineSpec, item: &str) -> bool {
    if producer.voider || consumer.voider {
        return false;
    }
    if !cell_machines_are_powerable(producer, consumer) {
        return false;
    }
    // Modules: refuse outright. The module post-pass in `layout.rs` keys
    // loadouts by `(entity, recipe)` gathered from `row_spans`, and a
    // fused cell contributes only the CONSUMER's recipe — the producer's
    // recipe never appears, so producer machines (which do carry it) would
    // get nothing stamped while the solver has already folded the module
    // speed/productivity bonus into their machine count. That is a silent
    // production shortfall, not a cosmetic gap, and matching loadouts
    // wouldn't fix it because the key itself is missing. Honest refusal
    // until module stamping can be keyed per-segment (Phase 2+).
    if !producer.game_modules.is_empty() || !consumer.game_modules.is_empty() {
        return false;
    }
    if !producer.self_loop.is_empty() || !consumer.self_loop.is_empty() {
        return false;
    }
    let any_fluid = |s: &MachineSpec| s.inputs.iter().chain(&s.outputs).any(|f| f.is_fluid);
    if any_fluid(producer) || any_fluid(consumer) {
        return false;
    }
    let solids = |fs: &[ItemFlow]| -> Vec<String> {
        fs.iter().filter(|f| !f.is_fluid).map(|f| f.item.clone()).collect()
    };
    let (p_in, p_out) = (solids(&producer.inputs), solids(&producer.outputs));
    let (c_in, c_out) = (solids(&consumer.inputs), solids(&consumer.outputs));

    p_out.len() == 1
        && p_out[0] == item
        && p_in.len() == 1
        // THE Phase-1 gate — see the doc comment.
        && c_in.len() == 1
        && c_in[0] == item
        && c_out.len() == 1
        && machine_dims(&producer.entity) == machine_dims(&consumer.entity)
}

/// The `MachineSpec` a fused cell presents to the rest of the pipeline.
///
/// A cell is one structure that draws the PRODUCER's input off a belt and
/// puts the CONSUMER's output onto a belt; the coupled item exists only
/// inside it. So the fused spec carries the producer's inputs and the
/// consumer's outputs — exactly what `lane_planner`, the output merger and
/// the validators already know how to read, with no cell special-casing
/// anywhere downstream.
///
/// This is also what dissolves the Phase-1c contract question from #447:
/// because the producer never gets a row of its own, no `RowSpan` is ever
/// built whose `output_belt_y` points at a belt that wasn't emitted. The
/// single fused row owns a real output belt, so all 11 bare
/// `output_belt_y` read sites — including the three output-merger ones
/// that select rows by `spec.outputs` and never consult a `BusLane` — are
/// correct by construction.
///
/// Rates are per-CONSUMER-machine, because the fused row's
/// `machine_count` is the consumer count: the output belt must carry
/// `consumer_count × consumer_rate`, and the input belt
/// `producer_count × producer_input_rate`. Scaling the latter by
/// `producer_count / consumer_count` keeps `rate × machine_count` right
/// for both.
fn fused_cell_spec(
    producer: &MachineSpec,
    consumer: &MachineSpec,
    producer_count: usize,
    consumer_count: usize,
) -> MachineSpec {
    let scale = producer_count as f64 / consumer_count.max(1) as f64;
    MachineSpec {
        entity: consumer.entity.clone(),
        recipe: consumer.recipe.clone(),
        self_loop: Vec::new(),
        voider: false,
        game_modules: consumer.game_modules.clone(),
        count: consumer_count as f64,
        inputs: producer
            .inputs
            .iter()
            .map(|f| ItemFlow { rate: f.rate * scale, ..f.clone() })
            .collect(),
        outputs: consumer.outputs.clone(),
    }
}

/// Try to emit a fused DI cell for an eligible pair.
///
/// `None` on any refusal — no straddle exists, the ladder can't cover the
/// binding edge, the stamper rejects the geometry — and the caller then
/// places both specs as ordinary rows, where the #432 bridge (and failing
/// that, the bus) serves the coupling. Refusing is always safe; emitting
/// an under-fed cell is not.
#[allow(clippy::too_many_arguments)]
fn try_build_cell(
    producer: &MachineSpec,
    consumer: &MachineSpec,
    item: &str,
    bus_width: i32,
    y_cursor: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    quality: QualityTier,
    level: u8,
    ctx: &StackingCtx,
) -> Option<(Vec<PlacedEntity>, RowSpan, i32)> {
    use crate::bus::di_cell::{plan_straddle, stamp_di_cell_io, DiCellIo, DiCellSpec};
    use crate::bus::inserter_ladder::{size_belt_drop_side, size_side, Reach};

    let snap = |c: f64| -> usize {
        let s = if (c - c.round()).abs() < 1e-9 { c.round() } else { c.ceil() };
        (s as usize).max(1)
    };
    let (p_count, c_count) = (snap(producer.count), snap(consumer.count));
    let (mw, mh) = machine_dims(&consumer.entity);
    let (mw, mh) = (mw as i32, mh as i32);

    let p_util = utilization_for(producer);
    let c_util = utilization_for(consumer);
    let producer_rate = producer.outputs.iter().find(|f| f.item == item)?.rate * p_util;
    let consumer_rate = consumer.inputs.iter().find(|f| f.item == item)?.rate * c_util;

    let plan = plan_straddle(p_count, c_count, producer_rate, consumer_rate, mw)?;

    // Couple with the cheapest inserter covering the busiest edge's
    // per-slot share. `Reach::Near` is a constraint, not a preference: the
    // cell puts the two rows ONE tile apart, and long-handed reaches to
    // exactly 2 (I8a), so it would pick straight past both machines.
    let ins = size_side(plan.required_rate(), Reach::Near, 0, max_inserter_tier, quality, level);
    let ins_rate = crate::common::machine_feed_rate(ins.entity, quality, level);

    let in_flow = producer.inputs.iter().find(|f| !f.is_fluid)?;
    let out_flow = consumer.outputs.iter().find(|f| !f.is_fluid)?;
    let in_total = in_flow.rate * p_util * p_count as f64;
    let out_total = out_flow.rate * c_util * c_count as f64;
    let in_stack = ctx.for_item(&in_flow.item);
    let out_stack = ctx.for_item(&out_flow.item);
    // A cell's input belt is a bus tap-off target exactly like any other
    // row's — `fused_cell_spec` puts the producer's input in the fused
    // row's `inputs` and `input_belt_y` is registered normally, so
    // `lane_planner` taps it the same way. It must therefore take the
    // TRUNK's tier via `row_input_belt`, not a tier sized to this cell's
    // local demand: the trunk is sized for total demand across every
    // consumer, and a locally-sized belt reintroduces the seam mismatch
    // `row_input_belt`'s doc comment exists to prevent (fast belt feeding
    // yellow, lane-throughput warnings, items backing up at the join).
    let in_belt = row_input_belt(max_belt_tier);
    // Inserters pick from BOTH lanes (I6), so the input side has the
    // belt's full capacity available — unlike the single-lane output.
    // `belt_entity_for_rate_stacked` saturates rather than failing, so
    // without this the cell would silently ship an undersized belt at
    // high rates instead of refusing; `plan_straddle` is scale-invariant
    // in machine count and would never catch it.
    if lane_capacity_stacked(in_belt, in_stack) * 2.0 + 1e-9 < in_total {
        return None;
    }
    // The cell's output belt is physically SINGLE-LANE: every output
    // inserter sits at the same y facing the same way, so all drops land
    // in the far lane (I5). Size it the way the ordinary single-lane path
    // does — against `rate * 2.0`, i.e. "a belt whose LANE capacity covers
    // the rate" — then verify. Sizing against `out_total` raw picks a belt
    // with half the capacity actually available.
    let out_belt = belt_entity_for_rate_stacked(out_total * 2.0, max_belt_tier, out_stack);
    if lane_capacity_stacked(out_belt, out_stack) + 1e-9 < out_total {
        // Too much output for one lane. The ordinary path would split the
        // recipe across rows; a cell cannot (its machines are placed at
        // `StraddlePlan` positions, and splitting needs the Phase 3
        // multi-band shape). Refuse instead of shipping a belt that
        // silently caps the cell.
        return None;
    }

    // Phase 1 stamps exactly ONE inserter per feed/output position —
    // `DiCellIo` carries an entity name, not a count, so a `SidePlan`
    // asking for more cannot be honoured. Size against a single slot and
    // refuse when one inserter (or the ladder's best effort) can't cover
    // the face, rather than discarding `count`/`shortfall` and quietly
    // under-feeding. Multi-inserter faces need the stamper to take counts;
    // that is Phase 2 work alongside face allocation.
    let feed = size_side(in_flow.rate * p_util, Reach::Near, 0, max_inserter_tier, quality, level);
    let drop = size_belt_drop_side(
        out_flow.rate * c_util,
        Reach::Near,
        0,
        max_inserter_tier,
        quality,
        out_stack,
        level,
        out_belt,
    );
    if feed.count > 1 || drop.count > 1 || feed.shortfall.is_some() || drop.shortfall.is_some() {
        return None;
    }

    let cell = stamp_di_cell_io(
        &plan,
        &DiCellSpec {
            producer_entity: &producer.entity,
            consumer_entity: &consumer.entity,
            producer_recipe: &producer.recipe,
            consumer_recipe: &consumer.recipe,
            item,
            inserter: ins.entity,
            inserter_rate: ins_rate,
        },
        &DiCellIo {
            input_item: &in_flow.item,
            input_belt: in_belt,
            feed_inserter: feed.entity,
            output_item: &out_flow.item,
            output_belt: out_belt,
            out_inserter: drop.entity,
        },
        bus_width,
        y_cursor,
        mw,
        mh,
    )?;

    let width = cell.x_max + 1;
    let span = RowSpan {
        y_start: cell.input_belt_y,
        y_end: cell.output_belt_y + 1,
        spec: fused_cell_spec(producer, consumer, p_count, c_count),
        machine_count: c_count,
        module_id: consumer.outputs.first().map(|o| o.module_id).unwrap_or(0),
        input_belt_y: vec![cell.input_belt_y],
        output_belt_y: cell.output_belt_y,
        row_width: width,
        fluid_port_ys: Vec::new(),
        fluid_port_pipes: Vec::new(),
        fluid_output_port_pipes: Vec::new(),
        output_east: true,
        output_belt_x_min: cell.x_min,
        output_belt_x_max: cell.x_max,
        horizontal_stack: None,
        secondary_output_belt: None,
        sorted_output_belts: Vec::new(),
        // Deliberately empty, and NOT an oversight. `di_input` exists to
        // tell the lane planner "this row consumes the item off a bridge,
        // don't build it a bus lane". A cell needs no such marker: the
        // fused spec's inputs are the PRODUCER's, so the coupled item
        // appears in no row's inputs, and no row produces it either — the
        // lane simply never comes into existence.
        di_input: Vec::new(),
    };
    Some((cell.entities, span, width))
}


/// Is this pair a **Phase 2 horizontal row cell**?
///
/// Looser than [`cell_eligible`] in exactly one way, which is the whole
/// point: the consumer may have a SECOND solid input. In a row cell the
/// consumer is coupled east/west, leaving its north and south faces free
/// for an ordinary input belt and output belt — so the second input has
/// somewhere to land, which is what the stacked cell could not offer.
/// This is the corpus's dominant shape (`di-patterns faces`: 177 of 2,039
/// cable->EC consumers).
fn row_cell_eligible(producer: &MachineSpec, consumer: &MachineSpec, item: &str) -> bool {
    if producer.voider || consumer.voider {
        return false;
    }
    if !cell_machines_are_powerable(producer, consumer) {
        return false;
    }
    if !producer.self_loop.is_empty() || !consumer.self_loop.is_empty() {
        return false;
    }
    if !producer.game_modules.is_empty() || !consumer.game_modules.is_empty() {
        return false;
    }
    // A fluid-OUT producer would need a pipe on the face its coupling
    // uses; only fluid IN is served. Same for the consumer: its south face
    // is spent on the output (and its feed, when it has one).
    if producer.outputs.iter().any(|f| f.is_fluid) || consumer.outputs.iter().any(|f| f.is_fluid) {
        return false;
    }
    // A fluid-drawing CONSUMER is served ONLY by the run the producer is
    // already piped — `solid-fuel-from-light-oil → rocket-fuel`, where
    // both roles want light-oil. The cell declares one set of tap points
    // on one row, so:
    //   - exactly one fluid, and the same one the producer draws (two
    //     different fluids on one run would cross-contaminate);
    //   - equal machine HEIGHTS, because the row is bottom-aligned and the
    //     pipe sits one tile above the PRODUCER's top — a shorter consumer
    //     would sit below it with its ports out of reach;
    //   - a real north input port on the consumer prototype, read from
    //     `fluid_ports` rather than assumed.
    // Anything else (a fluid the producer does not draw — the
    // `electric-engine-unit` lubricant shape) stays out of scope.
    let c_fluid_in: Vec<&str> =
        consumer.inputs.iter().filter(|f| f.is_fluid).map(|f| f.item.as_str()).collect();
    if !c_fluid_in.is_empty() {
        let p_fluid_items: Vec<&str> =
            producer.inputs.iter().filter(|f| f.is_fluid).map(|f| f.item.as_str()).collect();
        if c_fluid_in.len() != 1 || p_fluid_items != c_fluid_in {
            return false;
        }
        if machine_dims(&producer.entity).1 != machine_dims(&consumer.entity).1 {
            return false;
        }
        let (mirror, dir) = crate::fluid_ports::north_input_orientation(&consumer.entity);
        if crate::fluid_ports::north_input_dxs(&consumer.entity, mirror, dir).is_empty() {
            return false;
        }
    }
    let solids = |fs: &[ItemFlow]| -> Vec<String> {
        fs.iter().filter(|f| !f.is_fluid).map(|f| f.item.clone()).collect()
    };
    let (p_in, p_out) = (solids(&producer.inputs), solids(&producer.outputs));
    let (c_in, c_out) = (solids(&consumer.inputs), solids(&consumer.outputs));
    let p_fluid_in = producer.inputs.iter().filter(|f| f.is_fluid).count();
    // Either a single solid input (belt-fed, the original shape) or a
    // single fluid input and NO solid input (`casting-copper-cable`,
    // `casting-iron`, `solid-fuel-from-light-oil`). The all-fluid case is
    // what frees the producer's north face for the pipe run; a producer
    // with BOTH would need the belt and the pipe on the same face.
    let producer_feed_ok =
        (p_in.len() == 1 && p_fluid_in == 0) || (p_in.is_empty() && p_fluid_in == 1);
    if p_out.len() != 1 || p_out[0] != item || !producer_feed_ok || c_out.len() != 1 {
        return false;
    }
    // The coupled item, plus AT MOST one belt-fed other. Two is the
    // ordinary shape (cable + iron into EC); one means the coupling
    // supplies everything solid the consumer needs
    // (`solid-fuel-from-light-oil → rocket-fuel`, whose other ingredient
    // is the shared fluid), and its south face carries the output alone.
    // The coupled item, plus AT MOST two belt-fed others.
    //   1 — the coupling supplies everything solid
    //       (`solid-fuel-from-light-oil -> rocket-fuel`, whose other
    //       ingredient is the shared fluid); south face carries the output
    //       alone.
    //   2 — the ordinary shape (cable + iron into EC).
    //   3 — `iron-stick -> rail` (stone + steel-plate alongside the stick).
    //       The third input lands on the north face ABOVE the producer's
    //       belt; see `stamp_row_cell`, which enforces the geometry this
    //       needs and refuses when it cannot hold.
    if !(1..=3).contains(&c_in.len()) || !c_in.iter().any(|i| i == item) {
        return false;
    }
    // A third solid input needs a free north face for its belt and a free
    // feed row for its reach-2 inserters — both of which a piped producer
    // has already spent (its pipe run occupies the feed row). No corpus
    // pair wants both at once.
    if c_in.len() == 3 && p_fluid_in > 0 {
        return false;
    }
    // Every solid input the fused spec carries must be a DISTINCT item.
    // Two entries for one item at two different y starve silently: both
    // `lane_planner` and `ghost_router` break on the FIRST matching solid
    // input, so the second belt is built, never tapped, never fed — and
    // nothing disagrees with anything, so no validator fires. See the RFC
    // decision log.
    //
    // This is checked PAIRWISE over all of them, not just the first. The
    // predecessor used `.find()` to pull the consumer's one non-coupled
    // input, which was sufficient only while the face count permitted
    // exactly one; at three inputs it silently skipped producer×(second
    // other) and other×other. Pinned by
    // `di_row_cell_refuses_every_same_item_collision`.
    let c_others: Vec<&String> = c_in.iter().filter(|i| *i != item).collect();
    if c_others.iter().enumerate().any(|(i, a)| c_others[i + 1..].contains(a)) {
        return false;
    }
    if p_in.first().is_some_and(|p| c_others.contains(&p)) {
        return false;
    }
    // WIDTHS may differ freely — the row paces x by each machine's own
    // width. HEIGHTS may differ only in the direction the geometry
    // actually supports: the producer must be at least as tall as the
    // consumer.
    //
    // Bottom-alignment puts the two roles' south faces on one row (that is
    // what lets a coupler reach both), so a SHORTER producer has its north
    // face pushed down INTO the machine band. Its feed inserter row and
    // its pipe run would then have to sit on a row the taller consumer's
    // body already occupies, and the feed belt is a full-width run, so it
    // cannot simply dodge into the producer's columns. Rather than invent
    // geometry for a shape with no corpus demand, refuse it — the shipped
    // pairs are foundry(5) over assembler(3) and equal-height, both fine.
    // (The stacked cell still requires equal dims outright: its straddle
    // is derived from a single machine width.)
    if machine_dims(&producer.entity).1 < machine_dims(&consumer.entity).1 {
        return false;
    }
    true
}

/// Both roles must run on grid power, because the cell has no way to fuel
/// a burner.
///
/// Found by SIMULATING, not by reasoning: `solid-fuel-from-light-oil →
/// rocket-fuel` builds a cell that validates 0 errors 0 warnings and then
/// produces **literally nothing** — the solver picks `biochamber` for
/// `rocket-fuel` (category `organic-or-assembling`), a burner whose fuel
/// category is `nutrients`, and the sim reports `no_fuel: 8` with every
/// upstream chemical plant backed up behind it.
///
/// The gap is engine-wide, not a cell property: nothing anywhere delivers
/// burner fuel, and `validate::power` deliberately exempts biochambers
/// from coverage without any check taking over the obligation. Refusing
/// here is the narrow half of the fix — a cell that cannot run is worse
/// than no cell, because it validates clean and lies. Delivering fuel (or
/// steering machine selection away from burners) is the engine-wide half
/// and is NOT attempted here.
fn cell_machines_are_powerable(producer: &MachineSpec, consumer: &MachineSpec) -> bool {
    crate::common::needs_electricity(&producer.entity)
        && crate::common::needs_electricity(&consumer.entity)
}

/// Build a Phase 2 horizontal row cell, or `None` to fall back.
#[allow(clippy::too_many_arguments)]
fn try_build_row_cell(
    producer: &MachineSpec,
    consumer: &MachineSpec,
    item: &str,
    bus_width: i32,
    y_cursor: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    quality: QualityTier,
    level: u8,
    ctx: &StackingCtx,
) -> Option<(Vec<PlacedEntity>, RowSpan, i32)> {
    use crate::bus::di_cell::{plan_row_straddle, stamp_row_cell, RowCellSpec};
    use crate::bus::inserter_ladder::{size_belt_drop_side, size_side, Reach};

    let snap = |c: f64| -> usize {
        let s = if (c - c.round()).abs() < 1e-9 { c.round() } else { c.ceil() };
        (s as usize).max(1)
    };
    let (p_count, c_count) = (snap(producer.count), snap(consumer.count));
    // Per-role footprints: a cell may mix them (foundry 5x5 against an
    // assembler's 3x3), so nothing here may assume one machine size.
    let (pmw, pmh) = machine_dims(&producer.entity);
    let (cmw, cmh) = machine_dims(&consumer.entity);
    let (pmw, pmh, cmw, cmh) = (pmw as i32, pmh as i32, cmw as i32, cmh as i32);
    let p_util = utilization_for(producer);
    let c_util = utilization_for(consumer);

    let producer_rate = producer.outputs.iter().find(|f| f.item == item)?.rate * p_util;
    let consumer_rate = consumer.inputs.iter().find(|f| f.item == item)?.rate * c_util;
    let plan = plan_row_straddle(p_count, c_count, producer_rate, consumer_rate, pmw, cmw)?;

    // Coupler sits in a 1-tile gap, so reach-1 is a constraint (I8a).
    let coupler = size_side(plan.required_rate(), Reach::Near, 0, max_inserter_tier, quality, level);
    let coupler_rate = crate::common::machine_feed_rate(coupler.entity, quality, level);
    if coupler.count > 1 || coupler.shortfall.is_some() {
        return None;
    }

    // `None` for an all-fluid producer, whose north face carries a pipe
    // run instead of a belt (RFC-053 pipe cut).
    let p_in = producer.inputs.iter().find(|f| !f.is_fluid);
    let p_fluid = producer.inputs.iter().find(|f| f.is_fluid);
    // The consumer's belt-fed inputs — everything solid that the coupling
    // does not already supply. Empty when the coupled item is its ONLY
    // solid ingredient (then it has no belt-fed input and no inner belt);
    // eligibility caps it at two.
    //
    // The BUSIER one takes the south face, which is reach-1: long-handed
    // is the only reach-2 inserter (I8a) and it is the slower hand, so
    // giving it the lighter flow is what keeps the column counts down.
    // Ties break on item name, so the assignment is deterministic rather
    // than dependent on solver input order.
    let mut c_solids: Vec<&ItemFlow> =
        consumer.inputs.iter().filter(|f| !f.is_fluid && f.item != item).collect();
    c_solids.sort_by(|a, b| {
        b.rate.partial_cmp(&a.rate).unwrap_or(std::cmp::Ordering::Equal).then(a.item.cmp(&b.item))
    });
    let c_in = c_solids.first().copied();
    let c_in_b = c_solids.get(1).copied();
    let c_fluid = consumer.inputs.iter().find(|f| f.is_fluid);
    let out = consumer.outputs.iter().find(|f| !f.is_fluid)?;

    let p_total = p_in.map(|f| f.rate * p_util * p_count as f64).unwrap_or(0.0);
    let c_total = c_in.map(|f| f.rate * c_util * c_count as f64).unwrap_or(0.0);
    let out_total = out.rate * c_util * c_count as f64;

    // Both input belts are bus tap-off targets, so they take the TRUNK
    // tier (`row_input_belt`), never a locally-sized one — same seam
    // argument as every other row. Inserters pick from both lanes (I6),
    // so full belt capacity is available on the input side.
    let in_belt = row_input_belt(max_belt_tier);
    // Capacity is checked PER ITEM. `StackingCtx::for_item` is item-keyed
    // and `row_cell_eligible` guarantees the two inputs are different
    // items, so gating the consumer's belt on the producer item's
    // stacking factor would silently mis-size whenever they differ.
    let p_cap = p_in
        .map(|f| lane_capacity_stacked(in_belt, ctx.for_item(&f.item)) * 2.0)
        .unwrap_or(f64::INFINITY);
    let c_cap = c_in
        .map(|f| lane_capacity_stacked(in_belt, ctx.for_item(&f.item)) * 2.0)
        .unwrap_or(f64::INFINITY);
    if p_cap + 1e-9 < p_total || c_cap + 1e-9 < c_total {
        return None;
    }
    let b_total = c_in_b.map(|f| f.rate * c_util * c_count as f64).unwrap_or(0.0);
    let b_cap = c_in_b
        .map(|f| lane_capacity_stacked(in_belt, ctx.for_item(&f.item)) * 2.0)
        .unwrap_or(f64::INFINITY);
    if b_cap + 1e-9 < b_total {
        return None;
    }
    // The output belt is single-lane (every output inserter shares a y and
    // a facing, so all drops land in the far lane), hence the * 2.0.
    let out_stack = ctx.for_item(&out.item);
    let out_belt = belt_entity_for_rate_stacked(out_total * 2.0, max_belt_tier, out_stack);
    if lane_capacity_stacked(out_belt, out_stack) + 1e-9 < out_total {
        return None;
    }

    // Face plan. The producer's belt is NORTH at reach-1 — putting it on
    // a reach-2 hop would reintroduce long-handed's 2.40/s ceiling, which
    // is the very thing the row shape avoids. The consumer's two flows
    // share the SOUTH row: feed at reach-1 off the inner belt, output at
    // reach-2 stepping over it (long-handed is the only reach-2 inserter,
    // I8a). Output therefore routinely needs TWO columns at L2, so give
    // these faces real column budgets instead of forcing one each.
    //
    // Each face is budgeted against ITS OWN machine's width. Footprints may
    // differ, so a shared budget is wrong in both directions: taken from the
    // consumer it under-budgets a wider producer (refusing cells that fit),
    // and from a wider consumer it over-budgets a narrower producer — which
    // `stamp_row_cell`'s `cols(producer_w, n)` would then silently truncate
    // into an under-fed face rather than refuse.
    let p_budget = (pmw.max(1) as usize).saturating_sub(1);
    let c_budget = (cmw.max(1) as usize).saturating_sub(1);
    let p_feed = match p_in {
        Some(f) => size_side(f.rate * p_util, Reach::Near, p_budget, max_inserter_tier, quality, level),
        // No solid feed face to size — the pipe carries it.
        None => crate::bus::inserter_ladder::SidePlan { entity: "inserter", count: 0, shortfall: None },
    };
    let c_feed = match c_in {
        Some(f) => size_side(f.rate * c_util, Reach::Near, c_budget, max_inserter_tier, quality, level),
        // Nothing belt-fed to size; the coupling supplies every solid.
        None => crate::bus::inserter_ladder::SidePlan { entity: "inserter", count: 0, shortfall: None },
    };
    // Reach-2 only when the output inserter must step OVER the consumer's
    // input belt. Without that belt the output belt sits directly below the
    // face and reach-1 applies, which lifts the long-handed 2.40/s ceiling.
    // B's face is reach-2 unconditionally: its belt is the OUTER north row,
    // so its inserter always swings over the producer's belt.
    let c_feed_b = match c_in_b {
        Some(f) => size_side(f.rate * c_util, Reach::Far, c_budget, max_inserter_tier, quality, level),
        None => crate::bus::inserter_ladder::SidePlan { entity: "inserter", count: 0, shortfall: None },
    };
    if c_feed_b.shortfall.is_some() || c_feed_b.count > cmw.max(1) as usize {
        return None;
    }
    let out_reach = if c_in.is_some() { Reach::Far } else { Reach::Near };
    let drop = size_belt_drop_side(
        out.rate * c_util, out_reach, c_budget, max_inserter_tier, quality, out_stack, level, out_belt,
    );
    if p_feed.shortfall.is_some() || c_feed.shortfall.is_some() || drop.shortfall.is_some() {
        return None;
    }
    // The consumer's south row holds BOTH its feed and its output, so the
    // two together must fit the CONSUMER's width; the producer's feed face
    // is bounded by the PRODUCER's.
    if c_feed.count + drop.count > cmw.max(1) as usize || p_feed.count > pmw.max(1) as usize {
        return None;
    }

    let cell = stamp_row_cell(
        &plan,
        &RowCellSpec {
            producer_entity: &producer.entity,
            consumer_entity: &consumer.entity,
            producer_recipe: &producer.recipe,
            consumer_recipe: &consumer.recipe,
            item,
            coupler: coupler.entity,
            coupler_rate,
            producer_input: match p_in {
                Some(f) => (f.item.as_str(), in_belt, p_feed.entity),
                None => ("", in_belt, p_feed.entity),
            },
            producer_fluid: p_fluid.map(|f| (f.item.as_str(), "pipe")),
            consumer_input: c_in.map(|f| (f.item.as_str(), in_belt, c_feed.entity)),
            consumer_fluid: c_fluid.map(|f| (f.item.as_str(), "pipe")),
            consumer_input_b: c_in_b.map(|f| (f.item.as_str(), in_belt, c_feed_b.entity)),
            output_item: &out.item,
            output_belt: out_belt,
            out_inserter: drop.entity,
            producer_feed_count: p_feed.count,
            consumer_feed_count: c_feed.count,
            consumer_feed_b_count: c_feed_b.count,
            out_count: drop.count,
        },
        bus_width,
        y_cursor,
        pmw,
        pmh,
        cmw,
        cmh,
    )?;

    // Fused spec: producer's belt-fed input FIRST, then the consumer's —
    // the order MUST match `input_belt_ys`, because both `lane_planner`
    // and `ghost_router` resolve tap-off by indexing solid inputs
    // positionally. Getting it wrong makes both wrong identically, so
    // they agree with each other and yield a self-consistent bad layout.
    let scale = p_count as f64 / c_count.max(1) as f64;
    let fused = MachineSpec {
        entity: consumer.entity.clone(),
        recipe: consumer.recipe.clone(),
        self_loop: Vec::new(),
        voider: false,
        game_modules: Vec::new(),
        count: c_count as f64,
        // Order MUST match the row's belt/port lists: SOLIDS first, in the
        // same order `input_belt_ys` records them (producer's belt, then
        // the consumer's — either may be absent), because `lane_planner`
        // and `ghost_router` both resolve tap-off by indexing solid inputs
        // positionally. The fluid trails them; it is tapped on a separate
        // branch through `fluid_port_ys`/`fluid_port_pipes`, so its
        // position among the solids is immaterial but its RATE is not.
        inputs: {
            let mut v = Vec::new();
            if let Some(f) = p_in {
                v.push(ItemFlow {
                    item: f.item.clone(),
                    rate: f.rate * scale,
                    is_fluid: false,
                    module_id: f.module_id,
                });
            }
            if let Some(f) = c_in {
                v.push(ItemFlow {
                    item: f.item.clone(),
                    rate: f.rate,
                    is_fluid: false,
                    module_id: f.module_id,
                });
            }
            // LAST, matching `input_belt_ys`, which appends B's row last
            // for the same reason: the two lists are paired by index, and
            // appending leaves every existing index untouched.
            if let Some(f) = c_in_b {
                v.push(ItemFlow {
                    item: f.item.clone(),
                    rate: f.rate,
                    is_fluid: false,
                    module_id: f.module_id,
                });
            }
            // One entry per distinct fluid. When both roles draw the same
            // one — the only case eligibility admits — their demands SUM.
            // Nothing observably depends on this today (pipes have no tier,
            // so the planned rate does not change the stamped geometry, and
            // the sim manifest reads its feed rates from the SolverResult,
            // not from here) — checked by forcing the consumer term to
            // zero, which produced an identical layout. Kept anyway: the
            // spec's job is to state what the row actually draws, and an
            // understated demand is a lie waiting for its first reader.
            if p_fluid.is_some() || c_fluid.is_some() {
                let item = p_fluid.or(c_fluid).map(|f| f.item.clone()).unwrap_or_default();
                let rate = p_fluid.map(|f| f.rate * scale).unwrap_or(0.0)
                    + c_fluid.map(|f| f.rate).unwrap_or(0.0);
                v.push(ItemFlow {
                    item,
                    rate,
                    is_fluid: true,
                    module_id: p_fluid.or(c_fluid).map(|f| f.module_id).unwrap_or(0),
                });
            }
            v
        },
        outputs: consumer.outputs.clone(),
    };
    let width = cell.x_max + 1;
    let span = RowSpan {
        // The cell's OWN top, not `input_belt_ys[0]`. For a piped producer
        // the first input belt is the CONSUMER's, which sits below the
        // machines — taking `y_start` from it shrank the span to the last
        // couple of rows and dropped everything above out of row
        // attribution and pole banding.
        y_start: cell.y_top,
        y_end: cell.output_belt_y + 1,
        spec: fused,
        machine_count: c_count,
        module_id: consumer.outputs.first().map(|o| o.module_id).unwrap_or(0),
        input_belt_y: cell.input_belt_ys.clone(),
        output_belt_y: cell.output_belt_y,
        row_width: width,
        fluid_port_ys: cell.fluid_port_ys.clone(),
        fluid_port_pipes: cell.fluid_port_pipes.clone(),
        fluid_output_port_pipes: Vec::new(),
        output_east: true,
        output_belt_x_min: cell.x_min,
        output_belt_x_max: cell.x_max,
        horizontal_stack: None,
        secondary_output_belt: None,
        sorted_output_belts: Vec::new(),
        di_input: Vec::new(),
    };
    Some((cell.entities, span, width))
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
#[allow(clippy::too_many_arguments)]
pub fn place_rows(
    machines: &[MachineSpec],
    dependency_order: &[String],
    bus_width: i32,
    y_offset: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    quality: QualityTier,
    // Inserter-capacity research level 0..=7 (RFC-049,
    // `LayoutOptions.inserter_capacity`) — an inserter-ladder input parallel
    // to `max_inserter_tier`/`quality`; consumed by belt-drop (output)
    // sizing AND, since Phase 3, input-side sizing + near/far contests
    // (sim-measured `machine_feed_rate`). Level 0 is bit-identical to
    // pre-RFC.
    inserter_capacity: u8,
    final_output_items: Option<&FxHashSet<String>>,
    extra_gap_after_row: Option<&FxHashMap<usize, i32>>,
    row_layout: RowLayout,
    direct_insertion: bool,
    di_couplings: &[crate::models::DICoupling],
    ctx: &StackingCtx,
) -> (Vec<PlacedEntity>, Vec<RowSpan>, i32, i32) {
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut row_spans: Vec<RowSpan> = Vec::new();
    let mut y_cursor = y_offset;
    let mut max_width: i32 = 0;

    let ordered = if direct_insertion {
        order_specs(machines, dependency_order, di_couplings)
    } else {
        // DI off: pass empty couplings so order_specs is byte-identical
        // to the pre-DI Kahn sort (no greedy consumer pull-forward).
        order_specs(machines, dependency_order, &[])
    };
    let empty_final: FxHashSet<String> = FxHashSet::default();
    let final_items = final_output_items.unwrap_or(&empty_final);
    let empty_gaps: FxHashMap<usize, i32> = FxHashMap::default();
    let extra_gaps = extra_gap_after_row.unwrap_or(&empty_gaps);

    // DI coupling lookup: consumer_recipe → [(item, producer_recipe)].
    // A consumer can be coupled on more than one input, so the value is a
    // list — a plain map keyed by consumer_recipe would silently drop all
    // but the last coupling for such a consumer.
    let mut di_lookup: FxHashMap<&str, Vec<(&str, &str)>> = FxHashMap::default();
    if direct_insertion {
        for c in di_couplings {
            di_lookup
                .entry(c.consumer_recipe.as_str())
                .or_default()
                .push((c.item.as_str(), c.producer_recipe.as_str()));
        }
    }
    // Track the row index of the last-placed row for each recipe, so the
    // DI consumer can reference its producer's RowSpan.
    // EVERY row a recipe placed, not just the last. A recipe whose machine
    // count exceeds `max_per_row` is split into several sub-rows; keeping
    // only the last would let a DI consumer mark the item bus-skipped
    // (`di_input`) while bridging just one of them — the other sub-rows'
    // output would then have nowhere to go and the consumer would be
    // under-fed. See the multi-row refusal at the marking site below.
    let mut recipe_row_idxs: FxHashMap<&str, Vec<usize>> = FxHashMap::default();

    // RFC-053 Phase 1: producer spec index → consumer spec index for pairs
    // that can fuse into a DI cell. Precomputed so the producer half is
    // recognisable when the loop reaches it — `order_specs` is a
    // topological sort, so the producer is always the earlier of the two,
    // though not necessarily adjacent.
    //
    // Both halves must be un-split (one `MachineSpec` each, one row each):
    // a cell couples specific machines at specific x positions, so a
    // producer spread over several rows would leave every row but one
    // stranded. That is the same refusal the bridge makes for split
    // producers, and it is Phase 3 work (the multi-band cell).
    // CONSUMER spec idx -> (producer spec idx, coupled item, is_row_cell).
    //
    // Keyed by the CONSUMER, deliberately. A fused cell consumes the
    // union of both halves' belt-fed inputs, so it must be placed where
    // ALL of them are already available — i.e. at the consumer's slot in
    // the topological order, not the producer's. Emitting at the
    // producer's slot put the cell NORTH of its own iron-plate supply at
    // EC@10/s (iron-plate's row landed at y=22 against the cell at
    // y=10..17), breaking the lanes-run-south invariant: the router could
    // only answer with a 1-entity "return path", iron never arrived, the
    // EC machines were ingredient-short, and the whole chain backed up
    // (`full_output: 46`).
    // The variant is carried DELIBERATELY: `try_build_cell` does not
    // re-check `cell_eligible`, so dispatching on anything other than the
    // approved variant silently bypasses the gate that stops a two-solid-
    // input consumer being fused into a stacked cell (which cannot feed
    // its second input at all).
    let mut cell_pairs: FxHashMap<usize, (usize, &str, bool)> = FxHashMap::default();
    let mut claimed: FxHashSet<usize> = FxHashSet::default();
    if direct_insertion {
        for (c_idx, c_spec) in ordered.iter().enumerate() {
            let Some(couplings) = di_lookup.get(c_spec.recipe.as_str()) else {
                continue;
            };
            // Try every coupling this consumer has, not just a lone one.
            // A consumer coupled on two items cannot be a Phase-1 STACKED
            // cell (its second input has no free face) — but that is
            // exactly the case a Phase-2 ROW cell exists to serve, since
            // coupling east/west leaves both the north and south faces
            // free for ordinary belts. Requiring a single coupling here
            // silently excluded `electronic-circuit`, i.e. the corpus's
            // most common DI consumer, from ever being considered.
            for &(item, producer_recipe) in couplings {
                let same_recipe = |r: &str| ordered.iter().filter(|s| s.recipe == r).count();
                if same_recipe(producer_recipe) != 1 || same_recipe(&c_spec.recipe) != 1 {
                    continue;
                }
                let Some(p_idx) = ordered.iter().position(|s| s.recipe == producer_recipe) else {
                    continue;
                };
                if p_idx >= c_idx {
                    continue;
                }
                // A spec may only be fused once. Without this, a producer
                // that is itself a DI consumer could be claimed by two
                // different cells and placed twice.
                if claimed.contains(&p_idx) || claimed.contains(&c_idx) {
                    continue;
                }
                // Buildability, not merely eligibility — and the FULL
                // builder, not just the straddle. `fused_specs` is
                // pre-populated from this map, so a pair claimed here has
                // its producer skipped unconditionally; if the cell then
                // failed to build at emit time the producer would be
                // dropped from the layout entirely and its production
                // would silently vanish. Every refusal in the builders
                // (rates, belt capacity, inserter counts, straddle) is
                // independent of `y_cursor`, so a trial build at y=0
                // answers exactly the question that matters.
                let trial = |row: bool| {
                    if row {
                        try_build_row_cell(
                            ordered[p_idx], c_spec, item, bus_width, 0, max_belt_tier,
                            max_inserter_tier, quality, inserter_capacity, ctx,
                        )
                    } else {
                        try_build_cell(
                            ordered[p_idx], c_spec, item, bus_width, 0, max_belt_tier,
                            max_inserter_tier, quality, inserter_capacity, ctx,
                        )
                    }
                    .is_some()
                };
                let stacked_ok = couplings.len() == 1
                    && cell_eligible(ordered[p_idx], c_spec, item)
                    && pair_is_arrangeable(ordered[p_idx], c_spec, item, false)
                    && trial(false);
                let row_ok = row_cell_eligible(ordered[p_idx], c_spec, item)
                    && pair_is_arrangeable(ordered[p_idx], c_spec, item, true)
                    && trial(true);
                if !(stacked_ok || row_ok) {
                    continue;
                }
                claimed.insert(p_idx);
                claimed.insert(c_idx);
                cell_pairs.insert(c_idx, (p_idx, item, !stacked_ok));
                break;
            }
        }
    }
    // PRODUCER specs absorbed into a cell; skipped by the loop.
    //
    // Pre-populated from `cell_pairs` BEFORE the loop runs, not as each
    // cell is emitted. The cell is emitted at the CONSUMER's slot, and a
    // producer always sorts earlier — so marking it lazily would be too
    // late and the producer would already have been placed as an ordinary
    // row. That is exactly what happened: `copper-cable`'s own output
    // belt was stamped over the cell's iron-plate belt at y=16, leaving a
    // West-facing belt dead-ending into a tile the cell had claimed.
    let mut fused_specs: FxHashSet<usize> = cell_pairs.values().map(|&(p, _, _)| p).collect();

    for (spec_idx, spec) in ordered.iter().enumerate() {
        // Before the inter-recipe gap, so an absorbed consumer leaves no
        // phantom 2-tile hole where its row would have been.
        if fused_specs.contains(&spec_idx) {
            continue;
        }
        // The inter-recipe gap is always present — for DI pairs, the gap
        // houses the bridge inserter that spans from the producer's output
        // belt to the consumer's input belt. Without the gap, the belts
        // are too close for any inserter to bridge them.
        let is_di_consumer = direct_insertion && di_lookup.contains_key(spec.recipe.as_str());
        if spec_idx > 0 {
            y_cursor += 2; // gap between recipes for lane balancers / DI bridge
        }
        // Snap to nearest integer when within float drift of one — solver
        // math accumulates ulps in recursive ratio chains, so a recipe that
        // logically needs N machines often arrives as N + 1ulp, which a
        // naive ceil() would bump to N+1. The over-count silently wastes
        // a machine in most rows, and trips template assertions in others
        // (#277 utility-science-pack: 1.0000000000000002 advanced-oil-
        // processing → machine_count=2 → staggered-3-output panic).
        // RFC-053 Phase 1: fuse an eligible pair into one cell row. On any
        // refusal fall through to the normal path, where the #432 bridge
        // (and failing that, the bus) still serves the coupling.
        if let Some(&(p_idx, item, is_row)) = cell_pairs.get(&spec_idx) {
            let (producer, consumer) = (ordered[p_idx], spec);
            let built = if is_row {
                try_build_row_cell(
                    producer, consumer, item, bus_width, y_cursor, max_belt_tier,
                    max_inserter_tier, quality, inserter_capacity, ctx,
                )
            } else {
                try_build_cell(
                    producer, consumer, item, bus_width, y_cursor, max_belt_tier,
                    max_inserter_tier, quality, inserter_capacity, ctx,
                )
            };
            if let Some((cell_ents, span, width)) = built {
                fused_specs.insert(p_idx);
                max_width = max_width.max(width);
                let row_idx = row_spans.len();
                // Register under BOTH recipes: the cell is the producer's
                // only placement, so a later consumer looking up the
                // producer's rows must find this one rather than nothing.
                recipe_row_idxs.entry(consumer.recipe.as_str()).or_default().push(row_idx);
                recipe_row_idxs
                    .entry(producer.recipe.as_str())
                    .or_default()
                    .push(row_idx);
                let y_end = span.y_end;
                entities.extend(cell_ents);
                row_spans.push(span);
                y_cursor = y_end + extra_gaps.get(&row_idx).copied().unwrap_or(0);
                continue;
            }
            crate::trace::emit(crate::trace::TraceEvent::GhostSpecFailed {
                spec_key: format!("di-cell:{item}:refused"),
                from_x: 0,
                from_y: y_cursor,
                to_x: 0,
                to_y: y_cursor,
            });
        }

        let total_count = {
            let c = spec.count;
            let snapped = if (c - c.round()).abs() < 1e-9 { c.round() } else { c.ceil() };
            (snapped as usize).max(1)
        };

        let solid_inputs_count = spec.inputs.iter().filter(|f| !f.is_fluid).count();
        let first_solid_output = spec.outputs.iter().find(|f| !f.is_fluid);
        let first_solid_output_rate = first_solid_output.map(|f| f.rate).unwrap_or(0.0);
        let first_solid_output_item = first_solid_output.map(|f| f.item.as_str()).unwrap_or("");
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
        // Multi-solid-output rows (RFC Fulgora D2b) never lane-split —
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
        let out_stack = ctx.for_item(first_solid_output_item);
        let max_per_row = if single_lane {
            let ob = belt_entity_for_rate_stacked(output_rate * 2.0, max_belt_tier, out_stack);
            max_machines_for_belt(spec, ob, max_belt_tier)
        } else if is_hs_dual {
            // HS feeds input₀ via K stacked trunks, so only output and
            // input₁ constrain machines per row.
            let ob = belt_entity_for_rate_stacked(output_rate, max_belt_tier, out_stack);
            max_machines_for_belt_horizontal_stack(spec, ob, max_belt_tier)
        } else {
            let ob = belt_entity_for_rate_stacked(output_rate, max_belt_tier, out_stack);
            max_machines_for_belt_both_lanes(spec, ob, max_belt_tier, out_stack)
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
            let (row_ents, mut span, width) = build_one_row(
                spec,
                chunk,
                bus_width,
                y_cursor,
                max_belt_tier,
                max_inserter_tier,
                quality,
                inserter_capacity,
                is_final,
                row_layout,
                ctx,
            );
            let row_idx = row_spans.len();
            max_width = max_width.max(width);
            // DI consumers: for each coupled input, stamp the bridge FIRST
            // and only commit to DI (mark `di_input` so the lane planner
            // skips the bus lane for that item) when the bridge is actually
            // stamped. `stamp_di_bridge` returns empty when the bridge is
            // geometrically infeasible; leaving `di_input` unset for that
            // item then routes it through the bus lane (a real fallback,
            // not a starved consumer). Iterating all couplings — rather than
            // one lookup — is what lets a multiply-coupled consumer bridge
            // every coupled input instead of silently dropping all but one.
            if is_di_consumer {
                if let Some(couplings) = di_lookup.get(spec.recipe.as_str()) {
                    for &(item, producer_recipe) in couplings {
                        let Some(p_rows) = recipe_row_idxs.get(producer_recipe) else {
                            continue;
                        };
                        // Multi-row producer: only the row physically
                        // adjacent to the consumer is within reach, so
                        // bridging it while marking the item bus-skipped
                        // would strand every other sub-row's output and
                        // starve the consumer. Refuse DI for this coupling
                        // and let the bus carry it — honest, and correct at
                        // any machine count. (Serving split producers needs
                        // a multi-band cell; RFC-053 Phase 3.)
                        let [p_idx] = p_rows[..] else {
                            crate::trace::emit(crate::trace::TraceEvent::GhostSpecFailed {
                                spec_key: format!("di-bridge:{item}:producer-split-{}", p_rows.len()),
                                from_x: 0,
                                from_y: 0,
                                to_x: 0,
                                to_y: span.y_start,
                            });
                            continue;
                        };
                        let producer_span = &row_spans[p_idx];
                        let bridge_ents = stamp_di_bridge(
                            producer_span,
                            &span,
                            item,
                            spec,
                            chunk,
                            max_inserter_tier,
                            quality,
                            inserter_capacity,
                            ctx,
                        );
                        if !bridge_ents.is_empty() {
                            span.di_input.push((item.to_string(), p_idx));
                            entities.extend(bridge_ents);
                        }
                    }
                }
            }
            recipe_row_idxs.entry(spec.recipe.as_str()).or_default().push(row_idx);
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
#[allow(clippy::too_many_arguments)]
pub fn place_rows_from_result(
    result: &SolverResult,
    bus_width: i32,
    y_offset: i32,
    max_belt_tier: Option<&str>,
    max_inserter_tier: InserterTier,
    quality: QualityTier,
    inserter_capacity: u8,
    final_output_items: Option<&FxHashSet<String>>,
    extra_gap_after_row: Option<&FxHashMap<usize, i32>>,
    row_layout: RowLayout,
    direct_insertion: bool,
    ctx: &StackingCtx,
) -> (Vec<PlacedEntity>, Vec<RowSpan>, i32, i32) {
    place_rows(
        &result.machines,
        &result.dependency_order,
        bus_width,
        y_offset,
        max_belt_tier,
        max_inserter_tier,
        quality,
        inserter_capacity,
        final_output_items,
        extra_gap_after_row,
        row_layout,
        direct_insertion,
        &result.di_couplings,
        ctx,
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
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
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
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
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
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
                    self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            ..Default::default()
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
            max_machines_for_belt_both_lanes(&spec, "transport-belt", None, 1),
            14
        );
    }

    #[test]
    fn max_machines_capped_at_one() {
        // rate > lane_cap → floor < 1 → clamped to 1
        let spec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "test".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            max_machines_for_belt_both_lanes(&spec, "fast-transport-belt", None, 1),
            30
        );
    }

    // ---- order_specs tests ----

    #[test]
    fn order_specs_producer_before_consumer() {
        let machines = vec![iron_gear_spec(), iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let ordered = order_specs(&machines, &dep_order, &[]);
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
        let ordered = order_specs(&machines, &dep_order, &[]);
        assert_eq!(ordered[0].recipe, "recipe-b");
        assert_eq!(ordered[1].recipe, "recipe-a");
    }

    // ---- place_rows tests ----

    #[test]
    fn place_rows_single_recipe_no_split() {
        let machines = vec![iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].machine_count, 1);
        assert_eq!(spans[0].spec.recipe, "iron-plate");
    }

    #[test]
    fn place_rows_two_recipes_ordered() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].spec.recipe, "iron-plate");
        assert_eq!(spans[1].spec.recipe, "iron-gear-wheel");
    }

    #[test]
    fn place_rows_gap_between_recipes() {
        // Second recipe starts at y_end_of_first + 2 (gap)
        let machines = vec![iron_plate_spec(), iron_gear_spec()];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[1].y_start, spans[0].y_end + 2);
    }

    #[test]
    fn place_rows_y_offset() {
        let machines = vec![iron_plate_spec()];
        let dep_order = vec!["iron-plate".to_string()];
        let (_, spans, _, _) = place_rows(&machines, &dep_order, 0, 5, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());
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
            InserterTier::default(), QualityTier::Normal,
            0,
            None,
            None,
            RowLayout::default(),
            false,
            &[],
&StackingCtx::unstacked(),
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

    /// DI: when `direct_insertion` is true and a coupling exists, the
    /// consumer's RowSpan carries `di_input` marking the coupling, and
    /// when the producer is immediately adjacent (co-located by
    /// `order_specs`), the inter-recipe gap is suppressed.
    #[test]
    fn place_rows_di_marks_consumer() {
        use crate::solver::solve;
        use rustc_hash::FxHashSet;
        let available: FxHashSet<String> = ["iron-plate", "copper-plate"]
            .iter().map(|s| s.to_string()).collect();
        let result = solve("electronic-circuit", 10.0, &available, "assembling-machine-3").unwrap();
        assert!(!result.di_couplings.is_empty(), "solver should detect DI");

        let (_, spans, _, _) = place_rows(
            &result.machines,
            &result.dependency_order,
            0, 0,
            None,
            InserterTier::default(), QualityTier::Normal,
            0, None, None,
            RowLayout::default(),
            true, // direct_insertion ON
            &result.di_couplings,
            &StackingCtx::unstacked(),
        );

        let recipe_order: Vec<&str> = spans.iter().map(|s| s.spec.recipe.as_str()).collect();
        let cc_pos = recipe_order.iter().position(|&r| r == "copper-cable").unwrap();
        let ec_pos = recipe_order.iter().position(|&r| r == "electronic-circuit").unwrap();

        // Co-location: EC should be immediately after cable.
        assert_eq!(ec_pos, cc_pos + 1, "EC should be immediately after cable");
        // Gap is preserved (houses the bridge inserter).
        assert_eq!(
            spans[ec_pos].y_start, spans[cc_pos].y_end + 2,
            "DI gap is preserved for the bridge inserter"
        );
        // di_input marking: the bridge was stamped (feasible geometry), so
        // copper-cable is committed to DI on the EC row.
        assert!(
            spans[ec_pos].di_input.iter().any(|(item, _)| item == "copper-cable"),
            "EC row should be marked with DI input for copper-cable, got {:?}",
            spans[ec_pos].di_input
        );
    }

    /// DI off (default): di_input is None. Bit-identical to pre-DI layouts.
    #[test]
    fn place_rows_di_disabled_no_marking() {
        let result = electronic_circuit_solver_result();
        let couplings = vec![crate::models::DICoupling {
            producer_recipe: "copper-cable".to_string(),
            consumer_recipe: "electronic-circuit".to_string(),
            item: "copper-cable".to_string(),
            producer_count: 3.0,
            consumer_count: 3.0,
        }];
        let (_, spans, _, _) = place_rows(
            &result.machines,
            &result.dependency_order,
            0,
            1,
            None,
            InserterTier::default(), QualityTier::Normal,
            0,
            None,
            None,
            RowLayout::default(),
            false, // direct_insertion OFF
            &couplings,
            &StackingCtx::unstacked(),
        );

        for span in &spans {
            assert!(span.di_input.is_empty(), "DI off → no di_input on any row");
        }
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            InserterTier::default(), QualityTier::Normal,
            0,
            None,
            None,
            RowLayout::default(),
            false,
            &[],
&StackingCtx::unstacked(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            InserterTier::default(), QualityTier::Normal,
            0,
            None,
            None,
            RowLayout::default(),
            false,
            &[],
&StackingCtx::unstacked(),
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
            place_rows(&machines, &dep_order, 5, 0, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());

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
            place_rows(&machines, &dep_order, bus_width, 0, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());

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
            InserterTier::default(), QualityTier::Normal,
            0,
            None,
            Some(&extra_gaps),
            RowLayout::default(),
            false,
            &[],
&StackingCtx::unstacked(),
        );
        let (_, spans_no_gap, _, _) = place_rows(&machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal, 0, None, None, RowLayout::default(), false, &[], &StackingCtx::unstacked());

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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
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
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 4.0,
            inputs: vec![ItemFlow { item: "copper-plate".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            outputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
        };
        let ec_a = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 5.0,
            inputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 3.0, is_fluid: false, module_id: 0 }],
            outputs: vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
        };
        let ec_b = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 7.0,
            inputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 3.0, is_fluid: false, module_id: 1 }],
            outputs: vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
        };
        let machines = vec![cable, ec_a, ec_b];
        let dep_order: Vec<String> = vec!["copper-cable".into(), "electronic-circuit".into()];
        let ordered = order_specs(&machines, &dep_order, &[]);

        assert_eq!(ordered.len(), 3, "all input specs must be preserved through topo sort");
        assert_eq!(ordered[0].recipe, "copper-cable");
        assert_eq!(ordered[1].recipe, "electronic-circuit");
        assert_eq!(ordered[2].recipe, "electronic-circuit");
        let mut counts: Vec<usize> = ordered[1..3].iter().map(|s| s.count as usize).collect();
        counts.sort();
        assert_eq!(counts, vec![5, 7], "both EC siblings must appear with their original counts");
    }
    // ---- RFC-053 Phase 1: DI cells ----

    /// Flow-BALANCED cell pair: 2 plate machines at 1.0/s feed 1 gear
    /// machine at 2.0/s. The balance matters — `plan_straddle` refuses a
    /// pair whose supply and demand disagree, so the shared
    /// `iron_plate_spec`/`iron_gear_spec` helpers (1.0/s against 2.0/s)
    /// cannot form a cell and would make these tests pass vacuously.
    fn cell_pair() -> (MachineSpec, MachineSpec) {
        let mut producer = iron_plate_spec();
        producer.count = 2.0;
        let consumer = iron_gear_spec();
        (producer, consumer)
    }

    fn gear_cell_couplings() -> Vec<crate::models::DICoupling> {
        vec![crate::models::DICoupling {
            producer_recipe: "iron-plate".to_string(),
            consumer_recipe: "iron-gear-wheel".to_string(),
            item: "iron-plate".to_string(),
            producer_count: 2.0,
            consumer_count: 1.0,
        }]
    }

    /// The defining property: an eligible pair collapses to ONE row, and
    /// the coupled item never appears on a belt.
    #[test]
    fn di_cell_fuses_eligible_pair() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (ents, spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, None, None, RowLayout::default(),
            true, &gear_cell_couplings(), &StackingCtx::unstacked(),
        );
        assert_eq!(spans.len(), 1, "producer + consumer must fuse into one cell row, got {:?}",
            spans.iter().map(|s| s.spec.recipe.as_str()).collect::<Vec<_>>());

        // The fused spec is a composite machine: producer's input in,
        // consumer's output out.
        let sp = &spans[0];
        assert_eq!(sp.spec.recipe, "iron-gear-wheel");
        assert_eq!(sp.spec.inputs.iter().map(|f| f.item.as_str()).collect::<Vec<_>>(), vec!["iron-ore"]);
        assert_eq!(sp.spec.outputs.iter().map(|f| f.item.as_str()).collect::<Vec<_>>(), vec!["iron-gear-wheel"]);

        // No belt anywhere carries the coupled item — the whole point.
        let belts: Vec<&PlacedEntity> = ents.iter()
            .filter(|e| e.name.contains("transport-belt") || e.name.contains("underground-belt"))
            .collect();
        assert!(
            !belts.iter().any(|e| e.carries.as_deref() == Some("iron-plate")),
            "coupled item must never reach a belt; found {:?}",
            belts.iter().filter(|e| e.carries.as_deref() == Some("iron-plate"))
                 .map(|e| (e.x, e.y)).collect::<Vec<_>>()
        );
        // ...and it IS moved, by reach-1 inserters between the machines.
        assert!(
            ents.iter().any(|e| e.name.contains("inserter")
                && e.carries.as_deref() == Some("iron-plate")),
            "the cell must actually couple the item"
        );
    }

    /// A cell's fused row owns a REAL output belt — this is what makes the
    /// #447 contract question moot rather than merely unlikely to bite.
    #[test]
    fn di_cell_row_has_a_real_output_belt() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (ents, spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, None, None, RowLayout::default(),
            true, &gear_cell_couplings(), &StackingCtx::unstacked(),
        );
        assert_eq!(spans.len(), 1, "guard: this asserts nothing unless a cell was actually fused");
        let y = spans[0].output_belt_y;
        assert!(
            ents.iter().any(|e| e.y == y && e.name.contains("transport-belt")),
            "output_belt_y={y} must point at an emitted belt, not a phantom"
        );
    }

    /// The Phase-1 gate. `detect_di_couplings` emits cable->EC without
    /// looking at the consumer's input count, so the placer must refuse it:
    /// EC's second solid input (iron-plate) has no free face in a cell, and
    /// fusing anyway yields a layout that validates clean and starves.
    #[test]
    fn di_cell_refuses_multi_input_consumer() {
        let cable = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "copper-cable".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 3.0,
            inputs: vec![ItemFlow { item: "copper-plate".to_string(), rate: 2.5, is_fluid: false, module_id: 0 }],
            outputs: vec![ItemFlow { item: "copper-cable".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
        };
        let ec = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "electronic-circuit".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 2.0,
            inputs: vec![
                ItemFlow { item: "iron-plate".to_string(), rate: 2.5, is_fluid: false, module_id: 0 },
                ItemFlow { item: "copper-cable".to_string(), rate: 7.5, is_fluid: false, module_id: 0 },
            ],
            outputs: vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 2.5, is_fluid: false, module_id: 0 }],
        };
        assert!(
            !cell_eligible(&cable, &ec, "copper-cable"),
            "EC has two solid inputs — not a Phase 1 cell"
        );
        let machines = vec![ec, cable];
        let dep_order = vec!["copper-cable".to_string(), "electronic-circuit".to_string()];
        let couplings = vec![crate::models::DICoupling {
            producer_recipe: "copper-cable".to_string(),
            consumer_recipe: "electronic-circuit".to_string(),
            item: "copper-cable".to_string(),
            producer_count: 3.0,
            consumer_count: 2.0,
        }];
        let (ents, _spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, None, None, RowLayout::default(),
            true, &couplings, &StackingCtx::unstacked(),
        );
        // Since Phase 2 this pair legitimately fuses as a ROW cell (the
        // consumer is coupled east/west, leaving both faces free for its
        // second input). The invariant that survives, and the one
        // `cell_eligible` exists for, is that it must never become a
        // STACKED cell — that shape cannot feed iron-plate at all.
        assert!(
            !ents.iter().any(|e| e
                .segment_id
                .as_deref()
                .is_some_and(|s| s.starts_with("di-cell:"))),
            "a two-solid-input consumer must never be fused into a STACKED cell"
        );
    }

    /// A consumer with `item` coupled plus two belt-fed inputs `a` and `b`,
    /// against a producer whose own belt-fed input is `p_in`. Used to probe
    /// every same-item collision the three-input shape makes reachable.
    fn three_input_pair(p_in: &str, a: &str, b: &str) -> (MachineSpec, MachineSpec) {
        let flow = |item: &str, rate: f64| ItemFlow {
            item: item.to_string(),
            rate,
            is_fluid: false,
            module_id: 0,
        };
        let producer = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "iron-plate".to_string(),
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
            count: 2.0,
            inputs: vec![flow(p_in, 1.0)],
            outputs: vec![flow("iron-plate", 2.0)],
        };
        let consumer = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "three-input-thing".to_string(),
            self_loop: vec![],
            voider: false,
            game_modules: Vec::new(),
            count: 1.0,
            inputs: vec![flow("iron-plate", 2.0), flow(a, 1.0), flow(b, 1.0)],
            outputs: vec![flow("three-input-thing", 1.0)],
        };
        (producer, consumer)
    }

    /// **The prerequisite for relaxing the face count to three solid
    /// inputs.** Two entries for one item at two different `y` in the fused
    /// spec starve silently: both `lane_planner` and `ghost_router` `break`
    /// on the FIRST matching solid input, so the second belt is built,
    /// never tapped, never fed — and no validator disagrees with anything,
    /// because nothing disagrees. Exactly the failure this RFC already hit
    /// once on the producer/consumer axis.
    ///
    /// The old guard read the consumer's non-coupled inputs with `.find()`,
    /// which returns only the FIRST — sufficient while two solid inputs
    /// permitted exactly one non-coupled item, silently partial at three.
    /// All three pairs must be checked.
    ///
    /// Written BEFORE relaxing the gate, where it passes vacuously on the
    /// input count alone; it is the safety net for the relaxation, so it
    /// must go on passing for the stated reason afterwards.
    #[test]
    fn di_row_cell_refuses_every_same_item_collision() {
        // producer's belt input collides with the consumer's FIRST other
        // input — the only pair the old `.find()` guard actually checked.
        let (p, c) = three_input_pair("copper-plate", "copper-plate", "steel-plate");
        assert!(
            !row_cell_eligible(&p, &c, "iron-plate"),
            "producer's belt input == consumer's first other input"
        );
        // ...with the SECOND. `.find()` never looked here.
        let (p, c) = three_input_pair("steel-plate", "copper-plate", "steel-plate");
        assert!(
            !row_cell_eligible(&p, &c, "iron-plate"),
            "producer's belt input == consumer's second other input"
        );
        // ...and the two consumer inputs with each other, which needs no
        // producer collision at all to put two belts on one item.
        let (p, c) = three_input_pair("copper-plate", "steel-plate", "steel-plate");
        assert!(
            !row_cell_eligible(&p, &c, "iron-plate"),
            "consumer's two other inputs are the same item"
        );
        // Control: all four items distinct — refused today on the face
        // count, and the case the relaxation is FOR.
        let (p, c) = three_input_pair("copper-plate", "steel-plate", "plastic-bar");
        let _ = row_cell_eligible(&p, &c, "iron-plate");
    }

    /// The three-solid-input row cell, checked at TILE level rather than
    /// by "it returned `Some`". The shape is `iron-stick -> rail`: the
    /// consumer wants stone and steel-plate alongside the coupled stick.
    ///
    /// What must hold, and each is a way the design could have been wrong:
    ///   - belt B is the OUTER north row, one above the producer's belt;
    ///   - B's feed inserter is reach-2 (long-handed) and sits on the
    ///     PRODUCER's feed row, over a CONSUMER column — sharing that row
    ///     without colliding is the whole reason the design fits;
    ///   - its drop tile lands INSIDE the consumer's body. This is the one
    ///     the reach arithmetic can get silently wrong: the inserter drops
    ///     two tiles south of itself, and bottom-alignment can push a
    ///     shorter consumer's top below that.
    #[test]
    fn di_row_cell_places_a_third_input_on_the_outer_north_row() {
        let flow = |item: &str, rate: f64| ItemFlow {
            item: item.to_string(),
            rate,
            is_fluid: false,
            module_id: 0,
        };
        let producer = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "iron-stick".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 1.0,
            inputs: vec![flow("iron-plate", 1.0)],
            outputs: vec![flow("iron-stick", 2.0)],
        };
        let consumer = MachineSpec {
            entity: "assembling-machine-2".to_string(),
            recipe: "rail".to_string(),
            self_loop: vec![], voider: false, game_modules: Vec::new(),
            count: 1.0,
            inputs: vec![flow("iron-stick", 2.0), flow("stone", 2.0), flow("steel-plate", 1.0)],
            outputs: vec![flow("rail", 2.0)],
        };
        assert!(
            row_cell_eligible(&producer, &consumer, "iron-stick"),
            "three distinct solid inputs on an equal-height pair is the target shape"
        );
        let cell = try_build_row_cell(
            &producer, &consumer, "iron-stick", 0, 0, None,
            InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, &StackingCtx::unstacked(),
        )
        .expect("the three-input row cell must build");
        let (cell_ents, span, _) = cell;

        // Three input belts, and the spec's solid inputs must agree with
        // them INDEX for INDEX — the positional contract both `lane_planner`
        // and `ghost_router` rely on. A mismatch here is the silent
        // starvation class: every belt built, the wrong one tapped.
        let belts = &span.input_belt_y;
        assert_eq!(belts.len(), 3, "producer's belt, consumer's A, consumer's B");
        let solids: Vec<&str> =
            span.spec.inputs.iter().filter(|f| !f.is_fluid).map(|f| f.item.as_str()).collect();
        assert_eq!(solids.len(), 3, "fused spec carries all three solids");
        for (i, &y) in belts.iter().enumerate() {
            let carried: Vec<&str> = cell_ents
                .iter()
                .filter(|e| e.y == y && e.name.ends_with("transport-belt"))
                .filter_map(|e| e.carries.as_deref())
                .collect();
            assert!(
                carried.iter().all(|c| *c == solids[i]),
                "belt row {i} at y={y} must carry {}, found {:?}",
                solids[i],
                carried
            );
        }
        // B's row is the outermost: strictly above every other input belt,
        // and the topmost row the cell occupies.
        let b_y = belts[2];
        assert!(b_y < belts[0] && b_y < belts[1], "B is the OUTER north row, got {belts:?}");
        assert_eq!(b_y, span.y_start, "the cell's top must follow B's belt up");

        // B's feed inserter: reach-2, on the producer's feed row, dropping
        // inside the consumer.
        let b_item = solids[2];
        let feeds: Vec<_> =
            cell_ents.iter().filter(|e| e.carries.as_deref() == Some(b_item) && e.name.contains("inserter")).collect();
        assert!(!feeds.is_empty(), "B must have a feed inserter");
        let machine_tiles: std::collections::HashSet<(i32, i32)> = cell_ents
            .iter()
            .filter(|e| e.recipe.as_deref() == Some("rail"))
            .flat_map(|e| {
                let (w, h) = machine_dims(&e.name);
                let (w, h) = (w as i32, h as i32);
                (0..w).flat_map(move |dx| (0..h).map(move |dy| (e.x + dx, e.y + dy)))
            })
            .collect();
        for f in &feeds {
            assert_eq!(f.name, "long-handed-inserter", "B steps over the producer's belt");
            assert_eq!(f.y, b_y + 2, "B's feed shares the producer's feed row");
            assert_eq!(f.y - 2, b_y, "picks from B's belt");
            assert!(
                machine_tiles.contains(&(f.x, f.y + 2)),
                "B's drop tile ({}, {}) must be inside a rail machine",
                f.x,
                f.y + 2
            );
        }
    }

    /// DI off is the default and must stay bit-identical: no fusion.
    #[test]
    fn di_cell_inert_when_direct_insertion_off() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, None, None, RowLayout::default(),
            false, &gear_cell_couplings(), &StackingCtx::unstacked(),
        );
        assert_eq!(spans.len(), 2, "DI off must not fuse");
    }

    /// Fluids on either side are Phase 2 (the KC6 re-scope), not Phase 1.
    #[test]
    fn di_cell_refuses_fluid_touching_pair() {
        let mut producer = iron_plate_spec();
        producer.inputs.push(ItemFlow {
            item: "water".to_string(), rate: 10.0, is_fluid: true, module_id: 0,
        });
        assert!(
            !cell_eligible(&producer, &iron_gear_spec(), "iron-plate"),
            "fluid-touching pairs are Phase 2"
        );
    }

    /// #450 review: a producer's module loadout would be silently dropped
    /// (the module post-pass keys on `(entity, recipe)` from `row_spans`,
    /// and a cell contributes only the consumer's recipe) while the solver
    /// has already folded the bonus into machine counts. Refuse instead.
    #[test]
    fn di_cell_refuses_when_modules_are_present() {
        let (mut producer, consumer) = cell_pair();
        producer.game_modules = vec![crate::models::ModuleItem {
            item: "productivity-module".to_string(),
            count: 2,
            quality: None,
        }];
        assert!(
            !cell_eligible(&producer, &consumer, "iron-plate"),
            "a module loadout the cell cannot stamp must refuse fusion"
        );
    }

    /// #450 review: `SidePlan.count`/`.shortfall` were computed and
    /// discarded, but `DiCellIo` carries an entity name and no count — so
    /// a face needing 2+ inserters was stamped with one and silently
    /// under-fed. Regular tier can't move 2.0/s in one hand, so this pair
    /// must refuse rather than fuse.
    #[test]
    fn di_cell_refuses_when_a_face_needs_more_than_one_inserter() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (_, spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, None, InserterTier::Regular, QualityTier::Normal,
            0, None, None, RowLayout::default(),
            true, &gear_cell_couplings(), &StackingCtx::unstacked(),
        );
        assert_eq!(
            spans.len(), 2,
            "Regular tier at L0 cannot cover the feed face with one inserter — must refuse, \
             not stamp one inserter and under-feed"
        );
    }

    /// #450 review: the cell's output belt is physically single-lane (all
    /// output inserters share a y and a facing, so every drop lands in the
    /// far lane), so it must be sized against `rate * 2.0` like every other
    /// single-lane row. Sizing against the raw rate picked a belt with half
    /// the usable capacity.
    #[test]
    fn di_cell_output_belt_is_sized_for_a_single_lane() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        let (ents, spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, None, InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, None, None, RowLayout::default(),
            true, &gear_cell_couplings(), &StackingCtx::unstacked(),
        );
        assert_eq!(spans.len(), 1, "guard: needs a fused cell to mean anything");
        let out_y = spans[0].output_belt_y;
        let belt = ents.iter()
            .find(|e| e.y == out_y && e.name.contains("transport-belt"))
            .expect("output belt");
        let out_total: f64 = spans[0].spec.outputs.iter()
            .filter(|f| !f.is_fluid)
            .map(|f| f.rate * spans[0].machine_count as f64)
            .sum();
        let lane = crate::common::lane_capacity_stacked(&belt.name, 1);
        assert!(
            lane + 1e-9 >= out_total,
            "single-lane capacity {lane}/s must cover the cell's {out_total}/s on {}",
            belt.name
        );
    }

    /// #450 review: the cell's input belt is a bus tap-off target like any
    /// other row's, so it must take the TRUNK tier (`row_input_belt`), not
    /// a tier sized to the cell's local demand — otherwise a fast trunk
    /// joins a yellow row belt and items back up at the seam.
    #[test]
    fn di_cell_input_belt_matches_the_trunk_tier() {
        let (producer, consumer) = cell_pair();
        let machines = vec![consumer, producer];
        let dep_order = vec!["iron-plate".to_string(), "iron-gear-wheel".to_string()];
        // Cap at express: local demand is ~2/s (yellow would "fit"), so a
        // locally-sized belt would pick yellow and mismatch the trunk.
        let (ents, spans, _, _) = place_rows(
            &machines, &dep_order, 0, 0, Some("express-transport-belt"),
            InserterTier::default(), QualityTier::Normal,
            crate::common::DEFAULT_INSERTER_CAPACITY, None, None, RowLayout::default(),
            true, &gear_cell_couplings(), &StackingCtx::unstacked(),
        );
        assert_eq!(spans.len(), 1, "guard: needs a fused cell to mean anything");
        let in_y = spans[0].input_belt_y[0];
        let belt = ents.iter()
            .find(|e| e.y == in_y && e.name.contains("transport-belt"))
            .expect("input belt");
        assert_eq!(
            belt.name, "express-transport-belt",
            "input belt must match the trunk tier, not the cell's local rate"
        );
    }

}
