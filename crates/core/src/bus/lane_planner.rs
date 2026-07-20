//! Lane planning: deciding how items flow between rows on the main bus.
//!
//! Takes the solver's machine list + the placer's row spans and
//! produces one `BusLane` per item that needs trunk routing, plus
//! zero or more `LaneFamily` balancer blocks. Lane column order is
//! chosen via `lane_order::optimize_lane_order` to minimise tap-off
//! crossings. See `docs/ghost-pipeline-contracts.md` for the phase
//! contract this module upholds.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::models::SolverResult;
use crate::bus::lane_order::optimize_lane_order;
use crate::bus::partitioner::PartitionPlan;
use crate::bus::placer::RowSpan;

const LANE_CAPACITY_TABLE: &[(&str, f64)] = &[
    ("transport-belt", 7.5),
    ("fast-transport-belt", 15.0),
    ("express-transport-belt", 22.5),
];

/// A single vertical lane on the main bus, carrying one item (or fluid) from its
/// source row(s) down to its consumer row(s). Lanes run SOUTH; at each consumer the
/// lane turns EAST via a tap-off. See `docs/ghost-pipeline-contracts.md` for the
/// phase-by-phase contract the router promises.
#[derive(Clone, Debug)]
pub struct BusLane {
    /// Item (or fluid) name this lane carries.
    pub item: String,
    /// Module index within the item: `0` under `LayoutStrategy::Pooled`
    /// (one lane family per item — today's behaviour). The
    /// `rfc-modular-production` strategies distinguish multiple
    /// `(item, module_id)` lanes per item under
    /// `PartitionedDecomposed`.
    pub module_id: u32,
    /// Column (x-coordinate) assigned to this lane in the layout.
    pub x: i32,
    /// Y-coordinate where items enter this lane (0 for external inputs, or the
    /// producer row's output y for intermediate items).
    pub source_y: i32,
    /// Indices into the row-spans list for rows that consume from this lane.
    pub consumer_rows: Vec<usize>,
    /// Primary producer row index, or `None` for externally supplied items.
    pub producer_row: Option<usize>,
    /// Total throughput (items/s or fluid/s) for belt/pipe tier selection.
    pub rate: f64,
    /// Whether this lane carries a fluid (pipe + underground-pipe) instead of items.
    pub is_fluid: bool,
    /// Y-coordinates where tap-offs turn this lane EAST into a consumer row.
    pub tap_off_ys: Vec<i32>,
    /// Additional producer row indices beyond the primary (e.g. multiple sub-rows
    /// producing the same item).
    pub extra_producer_rows: Vec<usize>,
    /// Y-coordinate of the lane-balancer splitter, or `None` if no balancer is needed.
    pub balancer_y: Option<i32>,
    /// Index into the [`LaneFamily`] list when this lane is fed by an N-to-M balancer.
    pub family_id: Option<usize>,
    /// `(row_index, x, y)` of each pipe-to-ground exit connecting a fluid producer
    /// to this lane.
    pub fluid_port_positions: Vec<(usize, i32, i32)>,
    /// `(row_index, x, y)` of each fluid producer's output pipe port.
    pub fluid_output_port_positions: Vec<(usize, i32, i32)>,
    /// `(y_start, y_end)` inclusive — the full vertical range occupied by a
    /// balancer family block. Trunk segments inside this range are skipped.
    pub family_balancer_range: Option<(i32, i32)>,
    /// `Some(idx)` when this lane is the `idx`-th of K stacked trunks
    /// for a `RowLayout::HorizontalStack` consumer. The lane taps off
    /// at the consumer row's `horizontal_stack.trunk_ys[idx]`. `None`
    /// for lanes not feeding HS consumers.
    pub hs_trunk_idx: Option<usize>,
    /// `Some(y)` when this lane carries a surplus byproduct
    /// (`SolverResult::surplus_outputs`) that must exit at the layout
    /// perimeter: the trunk is extended down to `y` (the south boundary)
    /// so the flow physically leaves the layout instead of stranding at
    /// the producer's port. Set for both pure-surplus lanes (no
    /// consumers) and dual-purpose lanes (consumed AND surplus) — one
    /// lane per item, one physical network. Phase 2 of
    /// docs/rfc-solver-net-flow.md.
    pub perimeter_exit_y: Option<i32>,
}

impl BusLane {
    fn new(
        item: String,
        source_y: i32,
        consumer_rows: Vec<usize>,
        producer_row: Option<usize>,
        rate: f64,
        is_fluid: bool,
    ) -> Self {
        Self {
            item,
            module_id: 0,
            x: 0,
            source_y,
            consumer_rows,
            producer_row,
            rate,
            is_fluid,
            tap_off_ys: Vec::new(),
            extra_producer_rows: Vec::new(),
            balancer_y: None,
            family_id: None,
            fluid_port_positions: Vec::new(),
            fluid_output_port_positions: Vec::new(),
            family_balancer_range: None,
            hs_trunk_idx: None,
            perimeter_exit_y: None,
        }
    }

    /// Collect all producer row indices for this lane.
    pub(crate) fn all_producers(&self) -> Vec<usize> {
        let mut rows = Vec::new();
        if let Some(pr) = self.producer_row {
            rows.push(pr);
        }
        rows.extend(&self.extra_producer_rows);
        rows
    }
}

/// An N-to-M balancer block that merges N producer outputs into M sibling trunk
/// lanes for one item, ensuring even distribution. Stamped as a pre-solved SAT
/// template from `balancer_library`.
#[derive(Clone, Debug)]
pub struct LaneFamily {
    /// Item name shared by all lanes in this family.
    pub item: String,
    /// Module index within the item: `0` under `LayoutStrategy::Pooled`
    /// (one family per item). Phase 1 of `rfc-modular-production`
    /// distinguishes multiple `(item, module_id)` families per item;
    /// see the RFC for the partitioning algorithm.
    pub module_id: u32,
    /// `(N producers, M lanes)` — the balancer shape.
    pub shape: (usize, usize),
    /// Row indices of the N producers feeding into this balancer.
    pub producer_rows: Vec<usize>,
    /// X-coordinates of the M output lanes, populated after column assignment.
    pub lane_xs: Vec<i32>,
    /// Y-coordinate of the first (topmost) row of the balancer block.
    pub balancer_y_start: i32,
    /// Y-coordinate of the last row of the balancer block (inclusive).
    pub balancer_y_end: i32,
    /// Combined throughput across all lanes, used for belt tier selection.
    pub total_rate: f64,
    /// When `true`, this family is a merge-and-tap fallback trunk (RFC
    /// `docs/rfc-merge-tap-trunks.md`): `shape.0` producers merge onto its
    /// single output lane via a splitter merge-tree
    /// (`balancer_generate::merge_tree`) instead of an `(N, M)` balancer
    /// template, because `shape` was not stampable. `shape.1 == 1` always
    /// (one output trunk per merge-tap family; `K` such families share the
    /// item). The ghost router stamps a merge-tree and routes feeders to its
    /// inputs instead of a balancer block.
    pub merge_tap: bool,
}


/// Build the full `(BusLane, LaneFamily)` set from a solver result and
/// placed row spans. Lanes are ordered left-to-right to minimise
/// tap-off crossings; families are created for any item whose rate
/// exceeds per-lane capacity OR whose producer/lane shape needs a
/// balancer block. Each lane's `family_id` indexes into the returned
/// family vector.
pub fn plan_bus_lanes(
    solver_result: &SolverResult,
    row_spans: &[RowSpan],
    max_belt_tier: Option<&str>,
    plan: Option<&PartitionPlan>,
    total_height: i32,
    merge_tap: bool,
) -> Result<(Vec<BusLane>, Vec<LaneFamily>), String> {
    // Fluid surplus items AND fluid targets must physically exit at the
    // south boundary (`total_height`) — see `BusLane::perimeter_exit_y`.
    // Fluid targets previously got no lane at all (the output merger is
    // solid-only), leaving their product stranded at row-local stubs —
    // game-dead, and split into disconnected per-row networks the moment
    // a second producer row exists. Surplus entries are always module_id
    // 0 (netflow emits no partitioning), so keying by item name alone is
    // safe here.
    let surplus_fluid_items: FxHashSet<&str> = solver_result
        .surplus_outputs
        .iter()
        .chain(solver_result.external_outputs.iter())
        .filter(|f| f.is_fluid)
        .map(|f| f.item.as_str())
        .collect();
    let mut lanes: Vec<BusLane> = Vec::new();
    // Keyed by `(item, module_id)`. `module_id == 0` under Pooled and
    // for non-partitioned items; > 0 distinguishes per-consumer modules
    // when `LayoutStrategy::PartitionedDecomposed` is in use.
    let mut seen_keys: FxHashSet<(String, u32)> = FxHashSet::default();

    // Build item_to_consumers map, keyed by `(item, module_id)`. Each
    // input's `module_id` already came from the partitioner (`0` under
    // Pooled). Multi-consumer items partitioned by Phase 1 land in
    // distinct `(item, k)` buckets here.
    let mut item_to_consumers: FxHashMap<(String, u32), Vec<usize>> = FxHashMap::default();
    for (idx, rs) in row_spans.iter().enumerate() {
        // Voider rows (RFC Fulgora Phase 2, `docs/rfc-fulgora-scrap.md`
        // D1) declare an `inputs` entry so `bus::placer::order_specs`
        // sorts them after their producer, but they're deliberately
        // invisible HERE: their producer row is typically EAST-flowing
        // (its primary output is the solve's own target), and the
        // ordinary west-trunk ret-spec assumes producers sit next to a
        // WEST-side trunk — forcing that shape routes a return belt
        // backwards across the row's own east-flowing output. Voider
        // rows get their physical connection from `ghost_router`'s Step
        // 7c instead, which reuses the same producer-gathering +
        // `merge_output_rows` machinery the D2a/D2b export path uses.
        if rs.spec.voider {
            continue;
        }
        for inp in &rs.spec.inputs {
            item_to_consumers
                .entry((inp.item.clone(), inp.module_id))
                .or_default()
                .push(idx);
        }
    }

    // External inputs. Items entering from outside the system are
    // implicitly module_id 0 (no upstream producer to tag them
    // otherwise). Both solid and fluid enter at the top of the bus
    // (source_y = 0).
    for ext in &solver_result.external_inputs {
        let key = (ext.item.clone(), 0u32);
        if seen_keys.contains(&key) {
            continue;
        }
        let consumers = item_to_consumers.get(&key).cloned().unwrap_or_default();
        if !consumers.is_empty() {
            lanes.push(BusLane::new(
                ext.item.clone(),
                0,
                consumers,
                None,
                ext.rate,
                ext.is_fluid,
            ));
            seen_keys.insert(key);
        }
    }

    // Intermediate items (solid AND fluid). Producers are keyed by
    // `(item, module_id)` so the partitioner's K modules of the same
    // item end up as K distinct lanes.
    let mut item_to_producers: BTreeMap<(String, u32), Vec<usize>> = BTreeMap::new();
    let mut item_to_rate: BTreeMap<(String, u32), f64> = BTreeMap::new();
    let mut item_is_fluid: BTreeMap<(String, u32), bool> = BTreeMap::new();

    for (idx, rs) in row_spans.iter().enumerate() {
        for out in &rs.spec.outputs {
            let key = (out.item.clone(), out.module_id);
            item_to_producers.entry(key.clone()).or_default().push(idx);
            *item_to_rate.entry(key.clone()).or_insert(0.0) += out.rate * rs.machine_count as f64;
            item_is_fluid.insert(key, out.is_fluid);
        }
    }

    for (key, producer_rows) in item_to_producers.iter() {
        if seen_keys.contains(key) {
            continue;
        }
        let consumers = item_to_consumers.get(key).cloned().unwrap_or_default();
        // A lane with zero consumers is normally pointless — except for a
        // registered fluid surplus, whose lane exists purely to carry the
        // byproduct to the perimeter exit (Phase 2, rfc-solver-net-flow).
        if consumers.is_empty() && !surplus_fluid_items.contains(key.0.as_str()) {
            continue;
        }
        let first_producer = producer_rows[0];
        let rate = item_to_rate.get(key).copied().unwrap_or(0.0);
        let is_fluid = item_is_fluid.get(key).copied().unwrap_or(false);
        let (item, module_id) = key.clone();
        // A row with a D2b secondary solid output or a scrap-recycling
        // sushi sorter (RFC Fulgora `docs/rfc-fulgora-scrap.md`) has
        // multiple physically distinct output belts at different y. Anchor
        // `source_y` at THIS item's own belt (via the shared
        // `output_belt_y_for` lookup) rather than the primary's
        // `output_belt_y` — otherwise the lane's vertical trunk-column
        // reservation (see `layout.rs`'s pole occupancy) anchors at the
        // wrong row.
        let source_y = row_spans[first_producer].output_belt_y_for(&item);
        lanes.push(BusLane {
            item,
            module_id,
            x: 0,
            source_y,
            consumer_rows: consumers,
            producer_row: Some(first_producer),
            rate,
            is_fluid,
            extra_producer_rows: producer_rows[1..].to_vec(),
            ..Default::default()
        });
        seen_keys.insert(key.clone());
    }

    // Split lanes that exceed max belt tier capacity
    let (mut lanes, mut families) = split_overflowing_lanes(&lanes, row_spans, max_belt_tier, plan, merge_tap)?;

    // Pre-compute tap-off ys before sorting
    for lane in &mut lanes {
        lane.tap_off_ys = find_tap_off_ys(lane, row_spans);
        if lane.is_fluid {
            // Collect fluid port pipe positions for tap-off routing.
            // Filter by lane.item so rows with multiple fluid ports (e.g. oil-refinery)
            // only contribute the port(s) for this specific fluid.
            for &ri in &lane.consumer_rows {
                let rs = &row_spans[ri];
                for (ref item, px, py) in &rs.fluid_port_pipes {
                    if *item == lane.item {
                        lane.fluid_port_positions.push((ri, *px, *py));
                    }
                }
            }
            // Collect producer-side output port pipes (also filtered by item).
            let mut producer_rows = Vec::new();
            if let Some(pr) = lane.producer_row {
                producer_rows.push(pr);
            }
            producer_rows.extend(&lane.extra_producer_rows);
            for ri in producer_rows {
                let rs = &row_spans[ri];
                for (ref item, px, py) in &rs.fluid_output_port_pipes {
                    if *item == lane.item {
                        lane.fluid_output_port_positions.push((ri, *px, *py));
                    }
                }
            }
        }
    }

    // Fluid externals enter at the top (source_y = 0) and their vertical
    // trunks are underground by default, so no tightening is needed. Former
    // logic here pulled source_y down to just above the first tap to avoid
    // collisions with adjacent fluid trunks — now handled by F5a isolation
    // on UG trunks in ghost_router step 3.6.

    // Mark surplus fluid lanes for perimeter exit. One pass covers both
    // shapes: pure-surplus lanes (created above with zero consumers) and
    // dual-purpose lanes (real consumers AND a surplus remainder) — the
    // same lane, the same trunk column, one physical fluid network.
    //
    // Exit ys are STAGGERED (+1 per surplus lane): every exit anchor is a
    // surface pipe, and two surplus lanes on adjacent columns with exits
    // at the same y would auto-merge into one cross-fluid network (F1) —
    // observed live on the forced-AOP fixture (heavy-oil x=1 / light-oil
    // x=2 both surfacing at total_height). Diagonal offsets don't merge:
    // pipes connect orthogonally only.
    if !surplus_fluid_items.is_empty() {
        let mut exit_offset: i32 = 0;
        for lane in &mut lanes {
            if lane.is_fluid && surplus_fluid_items.contains(lane.item.as_str()) {
                lane.perimeter_exit_y = Some(total_height + exit_offset);
                exit_offset += 1;
                // Consumer-less perimeter lanes inherit `source_y` from the
                // producer row's output BELT y — a solid-row concept that
                // lands inside/below fluid-only rows and injects a phantom
                // trunk anchor (observed: a stray mid-chain anchor breaking
                // the UG chain to the exit). Anchor at the first real port
                // instead.
                if lane.consumer_rows.is_empty() {
                    if let Some(min_port_y) = lane
                        .fluid_output_port_positions
                        .iter()
                        .map(|&(_, _, py)| py)
                        .min()
                    {
                        lane.source_y = lane.source_y.min(min_port_y);
                    }
                }
            }
        }
    }

    // Compute lane balancer positions for intermediate solid lanes
    for lane in &mut lanes {
        if lane.is_fluid {
            continue;
        }
        if !lane.consumer_rows.is_empty() {
            continue;
        }
        let all_producers = lane.all_producers();

        if all_producers.len() <= 1 {
            continue;
        }

        let last_sideload_y = all_producers.iter()
            .map(|&pri| row_spans[pri].output_belt_y)
            .max()
            .unwrap();
        let bal_y = last_sideload_y + 1;
        let tap_set: FxHashSet<i32> = lane.tap_off_ys.iter().copied().collect();
        if !tap_set.contains(&bal_y) && !tap_set.contains(&(bal_y + 1)) {
            lane.balancer_y = Some(bal_y);
        }
    }

    // Optimize lane left-to-right ordering
    lanes = optimize_lane_order(&lanes, row_spans);

    // Pure-surplus lanes (perimeter exit, zero consumers) take the WESTMOST
    // columns: nothing routes west of the first lane column, so a surplus
    // trunk there crosses zero belts on its way to the south boundary —
    // sidestepping pipe×belt bridge junctions entirely (the mechanism
    // rfc-pipe-belt-junctions documents as unfinished). Their east-branch
    // to the producer port reuses the ordinary step-3.7 branch machinery.
    // Dual-purpose lanes (consumers AND surplus) keep their optimized slot;
    // moving them would trade tap-off crossings for exit crossings.
    lanes.sort_by_key(|l| {
        !(l.perimeter_exit_y.is_some() && l.consumer_rows.is_empty()) as u8
    });

    // Assign x-columns with 1-tile spacing. Adjacent fluid lanes are fine
    // because their vertical trunks are underground-by-default (UG pairs) —
    // see `ghost_router` step 3.6. EXCEPTION: surplus lanes' exit spans
    // can surface-fill short anchor gaps (plain pipe, not UG), so each
    // pure-surplus lane takes an extra spacer column — adjacent surface
    // pipe columns of different fluids would merge (F1).
    // `bus_width_for_lanes` derives width from max x, so the spacers are
    // accounted for in bus width and pass-2 row placement.
    let mut extra = 0i32;
    for (i, lane) in lanes.iter_mut().enumerate() {
        lane.x = (i + 1) as i32 + extra;
        if lane.perimeter_exit_y.is_some() && lane.consumer_rows.is_empty() {
            extra += 1;
        }
    }

    // Fluid lanes surface (plain pipe) at their ANCHOR rows: port taps,
    // source, and perimeter exit. Two adjacent fluid columns surfacing at
    // the SAME y auto-merge (F1) — templates stagger a single row's ports
    // across fluids, but nothing staggers anchors across *rows* (e.g. a
    // cracking row's heavy INPUT port sharing y with its light OUTPUT
    // port puts the heavy and light lanes' anchors on one row). Insert a
    // spacer column between any adjacent fluid pair whose surface-anchor
    // sets intersect. `bus_width_for_lanes` follows max x, so the extra
    // width is accounted for downstream.
    let anchor_set = |lane: &BusLane| -> FxHashSet<i32> {
        let mut s = FxHashSet::default();
        for &(_, _, py) in &lane.fluid_port_positions {
            s.insert(py);
        }
        for &(_, _, py) in &lane.fluid_output_port_positions {
            s.insert(py);
        }
        if let Some(ey) = lane.perimeter_exit_y {
            s.insert(ey);
        }
        // `source_y` only SURFACES when the run to the first downstream
        // anchor is too short for a UG entry (gap ≤ 2 stamps surface
        // pipe(s); gap ≥ 3 with a non-tap source gets an F5a-safe UG-S
        // entry — e.g. every external input lane at source_y = 0).
        let first_downstream = s.iter().copied().min();
        match first_downstream {
            Some(d) if d - lane.source_y >= 3 && !s.contains(&lane.source_y) => {}
            _ => {
                s.insert(lane.source_y);
                s.insert(lane.source_y + 1);
            }
        }
        s
    };
    let mut shift = 0i32;
    for i in 0..lanes.len() {
        lanes[i].x += shift;
        if shift > 0 {
            // keep family lane_xs consistent below (recomputed after this).
        }
        if i + 1 < lanes.len()
            && lanes[i].is_fluid
            && lanes[i + 1].is_fluid
            && lanes[i + 1].x + shift == lanes[i].x + 1
        {
            let a = anchor_set(&lanes[i]);
            let b = anchor_set(&lanes[i + 1]);
            if a.intersection(&b).next().is_some() {
                shift += 1;
            }
        }
    }

    // Fill in lane_xs on each family
    for (fid, fam) in families.iter_mut().enumerate() {
        fam.lane_xs = lanes.iter()
            .filter(|ln| ln.family_id == Some(fid))
            .map(|ln| ln.x)
            .collect();
        fam.lane_xs.sort_unstable();

        // Verify contiguous columns
        if !fam.lane_xs.is_empty() {
            let expected: Vec<i32> = (fam.lane_xs[0]..fam.lane_xs[0] + fam.lane_xs.len() as i32).collect();
            if fam.lane_xs != expected {
                return Err(format!(
                    "Balancer for item {} shape {:?} needs contiguous lane columns, but lane x's are {:?}",
                    fam.item, fam.shape, fam.lane_xs
                ));
            }
        }
    }

    // Resolve balancer_y_end from actual template heights and propagate the
    // full balancer zone to each lane so trunks skip the entire zone.
    let templates = crate::bus::balancer_library::balancer_templates();
    for (fam_idx, fam) in families.iter_mut().enumerate() {
        let (n, m) = (fam.shape.0 as u32, fam.shape.1 as u32);
        // Find the effective template height: a merge-and-tap family occupies
        // its merge-tree's height (`2n-1`); passthrough (`(m, m)` — a single
        // south-facing belt per output column, see issue #268) takes priority
        // over the library so it consumes only one row instead of the library
        // template's 6+; otherwise direct match, then decomposition fallback.
        let tpl_height = if fam.merge_tap {
            Some(crate::bus::balancer_generate::merge_tree(n).height)
        } else if crate::bus::balancer::is_passthrough_shape(n, m) {
            Some(1u32)
        } else {
            templates.get(&(n, m)).map(|t| t.height)
                .or_else(|| {
                    // Decomposition: find divisor g where (n/g, m/g) has a template.
                    (1..=n).rev().find_map(|g| {
                        if n % g == 0 && m % g == 0 {
                            templates.get(&(n / g, m / g)).map(|t| t.height)
                        } else {
                            None
                        }
                    })
                })
        };
        if let Some(h) = tpl_height {
            fam.balancer_y_end = fam.balancer_y_start + h as i32 - 1;
            let range = (fam.balancer_y_start, fam.balancer_y_end);
            // Merge-tap families share one `(item, module_id)` across their K
            // sibling trunks, so match by the lane's own `family_id` to avoid K
            // siblings clobbering each other's ranges; the balancer path keeps
            // the historical `(item, module_id)` match byte-for-byte.
            for lane in lanes.iter_mut() {
                if fam.merge_tap {
                    if lane.family_id == Some(fam_idx) {
                        lane.family_balancer_range = Some(range);
                    }
                    continue;
                }
                // Filter by `(item, module_id)` not just item: under
                // `LayoutStrategy::PartitionedDecomposed`, a single item
                // can have K sibling families at different lane columns
                // and different y-ranges. Without the module_id check,
                // each iteration overwrites all sibling lanes' ranges,
                // so module 0's lanes inherit module 1's balancer
                // y-range (or vice versa) — the trunk renderer then
                // skips that y-range in module 0's columns even though
                // module 1's balancer is in *different* columns. Result
                // before this guard: 17-row gap of missing trunk belts,
                // belt-dead-end errors at the boundary.
                if lane.family_id.is_some()
                    && lane.item == fam.item
                    && lane.module_id == fam.module_id
                {
                    lane.family_balancer_range = Some(range);
                }
            }
        }
    }

    crate::trace::emit(crate::trace::TraceEvent::LanesPlanned {
        lanes: lanes.iter().map(|l| crate::trace::LaneInfo {
            item: l.item.clone(),
            x: l.x,
            rate: l.rate,
            is_fluid: l.is_fluid,
            source_y: l.source_y,
            tap_off_ys: l.tap_off_ys.clone(),
            consumer_rows: l.consumer_rows.clone(),
            producer_row: l.producer_row,
            extra_producer_rows: l.extra_producer_rows.clone(),
            family_id: l.family_id,
        }).collect(),
        families: families.iter().map(|f| crate::trace::FamilyInfo {
            item: f.item.clone(),
            module_id: f.module_id,
            shape: f.shape,
            lane_xs: f.lane_xs.clone(),
            balancer_y_start: f.balancer_y_start,
            balancer_y_end: f.balancer_y_end,
            total_rate: f.total_rate,
            producer_rows: f.producer_rows.clone(),
        }).collect(),
        bus_width: lanes.iter().map(|l| l.x).max().map(|x| x + 1).unwrap_or(0),
    });

    Ok((lanes, families))
}

impl Default for BusLane {
    fn default() -> Self {
        Self {
            item: String::new(),
            module_id: 0,
            x: 0,
            source_y: 0,
            consumer_rows: Vec::new(),
            producer_row: None,
            rate: 0.0,
            is_fluid: false,
            tap_off_ys: Vec::new(),
            extra_producer_rows: Vec::new(),
            balancer_y: None,
            family_id: None,
            fluid_port_positions: Vec::new(),
            fluid_output_port_positions: Vec::new(),
            family_balancer_range: None,
            hs_trunk_idx: None,
            perimeter_exit_y: None,
        }
    }
}


/// Deterministic largest-first bin-packing of weighted `items` (each
/// `(row_index, weight)`) into `k` bins by least-loaded-bin (RFC
/// `docs/rfc-merge-tap-trunks.md` D1). Sorts by weight descending (ties broken
/// by original position), then greedily places each row into the currently
/// least-loaded bin (ties → lowest bin index). Determinism is a hard project
/// contract, so every tie is resolved by a stable index rule. With
/// `items.len() >= k` every bin receives at least one row (the first `k`
/// largest rows fill the `k` empty bins). Returns the `k` bins (row indices in
/// placement order) and each bin's total weight.
fn bin_pack_rows(items: &[(usize, f64)], k: usize) -> (Vec<Vec<usize>>, Vec<f64>) {
    let k = k.max(1);
    let mut bins: Vec<Vec<usize>> = vec![Vec::new(); k];
    let mut load: Vec<f64> = vec![0.0; k];
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|&a, &b| {
        items[b].1
            .partial_cmp(&items[a].1)
            .unwrap_or(Ordering::Equal)
            .then(a.cmp(&b))
    });
    for &i in &order {
        let (row, w) = items[i];
        let target = (0..k)
            .min_by(|&p, &q| {
                load[p]
                    .partial_cmp(&load[q])
                    .unwrap_or(Ordering::Equal)
                    .then(p.cmp(&q))
            })
            .unwrap_or(0);
        bins[target].push(row);
        load[target] += w;
    }
    (bins, load)
}

/// Split lanes whose rate exceeds the available belt's per-lane capacity.
///
/// `plan`, when present, supplies a per-`(item, module_id)` lower bound
/// on `effective_n_splits` via `lane_count_override`. This is how
/// `apply_shape_fixes`'s `PadLanes` strategy reaches the lane planner:
/// the partition plan records the padded `m`, and we honour it here as
/// long as it's at least as large as the natural rate-and-consumer
/// derivation. Plan never *shrinks* lane counts; it can only force
/// extra empty pad lanes to materialise.
fn split_overflowing_lanes(
    lanes: &[BusLane],
    row_spans: &[RowSpan],
    max_belt_tier: Option<&str>,
    plan: Option<&PartitionPlan>,
    merge_tap: bool,
) -> Result<(Vec<BusLane>, Vec<LaneFamily>), String> {
    let default_cap = LANE_CAPACITY_TABLE.last().map(|(_, c)| *c).unwrap_or(15.0);
    let max_lane_cap = if let Some(tier) = max_belt_tier {
        LANE_CAPACITY_TABLE.iter()
            .find(|(name, _)| *name == tier)
            .map(|(_, cap)| *cap)
            .unwrap_or(default_cap)
    } else {
        default_cap
    };

    let mut result: Vec<BusLane> = Vec::new();
    let mut families: Vec<LaneFamily> = Vec::new();

    for lane in lanes {
        if lane.is_fluid {
            result.push(lane.clone());
            continue;
        }

        let n_splits = if lane.rate > max_lane_cap {
            ((lane.rate / max_lane_cap).ceil() as usize).max(1)
        } else {
            1
        };
        // External input lanes (no producer) can share a trunk across multiple
        // consumer rows — split only by capacity. Intermediate lanes still get
        // 1-per-consumer here (the round-robin below then degenerates to
        // identity). NOTE: the historical reason ("route_intermediate_lane
        // only handles tap_off_ys[0]") is stale — that direct-mode router was
        // deleted (5c8abe3); today's `route_bus_ghost` Step 4 iterates ALL
        // `tap_off_ys`, so multi-tap intermediate trunks are renderable. The
        // 1-per-consumer split is retained deliberately (the merge-and-tap
        // fallback is the path that shares an intermediate trunk).
        let is_external_input =
            lane.producer_row.is_none() && lane.extra_producer_rows.is_empty();
        let n_splits = if is_external_input {
            n_splits
        } else {
            n_splits.max(lane.consumer_rows.len())
        };

        // External inputs serving multiple consumers via fewer trunks: consolidation
        if is_external_input && lane.consumer_rows.len() > n_splits {
            crate::trace::emit(crate::trace::TraceEvent::LaneConsolidated {
                item: lane.item.clone(),
                rate: lane.rate,
                consumer_count: lane.consumer_rows.len(),
                n_trunk_lanes: n_splits,
                rate_per_lane: lane.rate / n_splits as f64,
            });
        }

        if n_splits <= 1 {
            result.push(lane.clone());
            continue;
        }

        // Pre-compute the consumer/producer shape so we can decide whether
        // to clamp the trunk count to the consumer-side bottleneck and
        // optionally insert a balancer family. The balancer feeds full
        // belts (both lanes), so when one is present we only need M =
        // consumer-count trunks instead of M = producer-count.
        let all_producer_rows = lane.all_producers();
        let n_producers = all_producer_rows.len();
        let is_collector = lane.consumer_rows.is_empty();

        // HorizontalStack consumer detection. For each consumer row,
        // compute how many trunks this item needs into that row:
        //   * HS row where this item is the row's input₀ → K trunks
        //     (one per stacked input₀ trunk in the HS layout).
        //   * Anything else → 1 trunk.
        // Build a flat list of `(consumer_row, Option<hs_trunk_idx>)`
        // entries — one entry per required trunk slot — so each split
        // lane can be assigned a specific consumer + trunk index.
        let consumer_trunk_assignments: Vec<(usize, Option<usize>)> = lane
            .consumer_rows
            .iter()
            .flat_map(|&ri| {
                let cr = &row_spans[ri];
                let hs_k = cr
                    .horizontal_stack
                    .as_ref()
                    .filter(|hs| hs.input0_item == lane.item)
                    .map(|hs| hs.trunk_ys.len());
                match hs_k {
                    Some(k) => (0..k).map(|t| (ri, Some(t))).collect::<Vec<_>>(),
                    None => vec![(ri, None)],
                }
            })
            .collect();
        let any_hs = consumer_trunk_assignments
            .iter()
            .any(|(_, t)| t.is_some());

        // Consumer-side trunk count: on the balancer path each consumer row
        // gets its own trunk. (The old "route_intermediate_lane only honors
        // tap_off_ys[0]" rationale is stale — that router was deleted in
        // 5c8abe3; `route_bus_ghost` Step 4 now iterates all `tap_off_ys`.
        // Multiple consumers per trunk is the merge-and-tap fallback's job.)
        // For collector lanes (no consumers) keep the capacity-derived count.
        // HS rows want K trunks for their input₀, expanding the trunk count
        // beyond `consumer_rows.len()`.
        let consumer_trunk_count = if is_collector {
            n_splits
        } else {
            consumer_trunk_assignments.len()
        };

        // Clamp the trunk count to the consumer-side bottleneck. Without
        // this, a per-lane-cap-driven `n_splits` greater than the
        // consumer count produced empty-consumer splits whose producers
        // were silently dropped (`continue` below). Each kept trunk now
        // runs at full-belt capacity, which only works if a balancer
        // family is stamped; without one, multiple producers fan-in via
        // `ret:` sideloads and the trunk's per-lane cap still applies.
        let full_belt_cap = max_lane_cap * 2.0;
        let clamp_to_consumers =
            !is_external_input && !is_collector && n_splits > consumer_trunk_count;

        // Plan-driven pad floor. `apply_shape_fixes` records the padded
        // `m` in the partition plan via `PadLanes`; we honour it here as
        // a lower bound on `effective_n_splits`. Only meaningful in the
        // intermediate non-HS, non-clamp path (HS rows pin a fixed slot
        // count; consumer-clamp is for the "rate exceeds consumer trunk
        // capacity" failure mode that pad doesn't address). External
        // inputs and collectors (which already hit the `is_external_input`
        // or `is_collector` paths) ignore the pad floor.
        let plan_pad_floor: usize = plan
            .and_then(|p| p.lane_count_override(&lane.item, lane.module_id))
            .map(|lc| lc as usize)
            .unwrap_or(0);

        let effective_n_splits = if any_hs {
            // HS consumer(s) want a fixed total trunk count.
            consumer_trunk_count
        } else if clamp_to_consumers {
            // Sanity: with M trunks each capped at full-belt rate, the
            // total rate must fit. Today's corpus (up to processing-unit
            // @ 2/s express belt) stays inside this envelope. Return Err
            // (rather than panic) when an experimental candidate
            // (e.g. `bus::decomposition_search::ModuleSizeSplit`) lands
            // a configuration this lane planner doesn't yet handle —
            // the caller can then fall back to a candidate that does.
            // Fixing properly requires a multi-stage balancer in this
            // path; tracked as the unblocker for K-DS1-2 in
            // `docs/rfc-decomposition-search.md`.
            if lane.rate > (consumer_trunk_count as f64) * full_belt_cap {
                return Err(format!(
                    "lane_planner: consumer-clamped fan-in for item {item} needs total_rate \
                     {rate}/s <= {m} consumer trunks * full-belt-cap {cap}/s = {limit}/s; \
                     not yet supported (multi-stage balancer not wired)",
                    item = lane.item,
                    rate = lane.rate,
                    m = consumer_trunk_count,
                    cap = full_belt_cap,
                    limit = (consumer_trunk_count as f64) * full_belt_cap,
                ));
            }
            consumer_trunk_count
        } else {
            n_splits.max(plan_pad_floor)
        };
        // Empty pad lanes (those with no consumer assignment) must
        // survive the `consumers.is_empty()` skip below when the plan
        // forced extra `m` past the natural rate-or-consumer count. Any
        // other path keeps the legacy skip-empty behaviour.
        let pad_active = !any_hs
            && !clamp_to_consumers
            && plan_pad_floor > n_splits
            && effective_n_splits > consumer_trunk_count;

        crate::trace::emit(crate::trace::TraceEvent::LaneSplit {
            item: lane.item.clone(),
            rate: lane.rate,
            max_lane_cap,
            n_splits: effective_n_splits,
        });

        // Distribute consumer rows across the effective trunks. With HS
        // consumers, each split serves exactly one (consumer, trunk_idx)
        // pair from `consumer_trunk_assignments`. Otherwise round-robin
        // over distinct consumer rows.
        let mut consumers_per_split: Vec<Vec<usize>> = vec![Vec::new(); effective_n_splits];
        let mut hs_idx_per_split: Vec<Option<usize>> = vec![None; effective_n_splits];
        if any_hs {
            for (i, &(ri, hs_idx)) in consumer_trunk_assignments
                .iter()
                .enumerate()
                .take(effective_n_splits)
            {
                consumers_per_split[i].push(ri);
                hs_idx_per_split[i] = hs_idx;
            }
        } else {
            for (i, &ri) in lane.consumer_rows.iter().enumerate() {
                consumers_per_split[i % effective_n_splits].push(ri);
            }
        }

        // Distribute producer rows by rate. With a balancer family every
        // producer feeds the family rather than a specific trunk, but we
        // still build this to satisfy the non-family path below.
        let mut producers_per_split: Vec<Vec<usize>> = vec![Vec::new(); effective_n_splits];
        let mut split_prod_rate: Vec<f64> = vec![0.0; effective_n_splits];

        for &pri in &all_producer_rows {
            let rs = &row_spans[pri];
            let prod_rate: f64 = rs.spec.outputs.iter()
                .filter(|o| o.item == lane.item)
                .map(|o| o.rate * rs.machine_count as f64)
                .sum();
            let target = split_prod_rate.iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            producers_per_split[target].push(pri);
            split_prod_rate[target] += prod_rate;
        }

        // Re-derive `n_lanes_with_consumers` against the effective shape.
        // With `clamp_to_consumers` set, every effective trunk has exactly
        // one consumer (M = consumer count). Without the clamp this falls
        // back to the original count that drove the legacy fan-out family.
        // When `pad_active` (the partition plan demanded extra trunks past
        // the natural rate-or-consumer count), include those pad lanes —
        // the family balancer's `m` must match the *physical* trunk count,
        // not the consumer-bearing-trunk count, so `apply_shape_fixes`'s
        // `PadLanes` strategy actually shifts `m` to a stampable shape.
        let n_lanes_with_consumers = if is_collector || pad_active {
            effective_n_splits
        } else {
            consumers_per_split.iter().filter(|c| !c.is_empty()).count()
        };

        // Merge-and-tap fallback (RFC docs/rfc-merge-tap-trunks.md). If the
        // (N, M) balancer this lane would otherwise be given has no stampable
        // template, retire it to K = ceil(rate / full_belt_cap) shared trunks:
        // each trunk's producer group merges via a splitter merge-tree and its
        // consumer group taps the trunk with priority splitters. Scoped to
        // genuine multi-producer, multi-consumer intermediate families —
        // HorizontalStack geometry and fan-out (N == 1) keep their existing
        // paths. Strictly additive: `shape_is_stampable` shapes are byte-for-
        // byte untouched, so nothing in the default corpus moves (only
        // utility@10/s's copper-cable / iron-plate families are unstampable).
        if merge_tap
            && !any_hs
            && !is_collector
            && n_producers >= 2
            && n_lanes_with_consumers >= 2
            && !crate::bus::balancer::shape_is_stampable(
                n_producers as u32,
                n_lanes_with_consumers as u32,
            )
        {
            // K throughput-sized trunks, clamped so no trunk is producer- or
            // consumer-empty (K <= min(N, M)). full_belt_cap is the pinned 2×
            // lane capacity (both lanes loaded — the balancer-present
            // precedent at lane_planner's cap computation above).
            let m_consumers = lane.consumer_rows.len();
            let k = ((lane.rate / full_belt_cap).ceil() as usize)
                .clamp(1, n_producers.min(m_consumers));

            let consumer_weights: Vec<(usize, f64)> = lane
                .consumer_rows
                .iter()
                .map(|&ri| {
                    let rs = &row_spans[ri];
                    let d: f64 = rs
                        .spec
                        .inputs
                        .iter()
                        .filter(|inp| inp.item == lane.item && !inp.is_fluid)
                        .map(|inp| inp.rate * rs.machine_count as f64)
                        .sum();
                    (ri, d)
                })
                .collect();
            let producer_weights: Vec<(usize, f64)> = all_producer_rows
                .iter()
                .map(|&pri| {
                    let rs = &row_spans[pri];
                    let p: f64 = rs
                        .spec
                        .outputs
                        .iter()
                        .filter(|o| o.item == lane.item)
                        .map(|o| o.rate * rs.machine_count as f64)
                        .sum();
                    (pri, p)
                })
                .collect();

            let (consumer_bins, consumer_bin_rate) = bin_pack_rows(&consumer_weights, k);
            let (producer_bins, producer_bin_rate) = bin_pack_rows(&producer_weights, k);

            // All K merge-trees sit in one horizontal band below every producer
            // row (the trunk-head zone a balancer would occupy); the trunk picks
            // up one row below. Resolved to the real per-trunk merge-tree height
            // when `family_balancer_range` is finalized.
            let band_y = all_producer_rows
                .iter()
                .map(|&p| row_spans[p].y_end)
                .max()
                .unwrap_or(0);

            // Pass-invariant event, deduped across the two `plan_bus_lanes`
            // passes by suppressing it on pass 2 (see `trace::
            // with_merge_tap_fallback_suppressed`); pass 1 always runs and
            // records it.
            if !crate::trace::merge_tap_fallback_suppressed() {
                crate::trace::emit(crate::trace::TraceEvent::MergeTapFallback {
                    item: lane.item.clone(),
                    module_id: lane.module_id,
                    shape: (n_producers, n_lanes_with_consumers),
                    k_trunks: k,
                    producers_per_trunk: producer_bins.iter().map(|b| b.len()).collect(),
                    consumers_per_trunk: consumer_bins.iter().map(|b| b.len()).collect(),
                });
            }

            for t in 0..k {
                let n_t = producer_bins[t].len();
                let fid = families.len();
                families.push(LaneFamily {
                    item: lane.item.clone(),
                    module_id: lane.module_id,
                    shape: (n_t, 1),
                    producer_rows: producer_bins[t].clone(),
                    lane_xs: Vec::new(),
                    balancer_y_start: band_y,
                    balancer_y_end: band_y,
                    total_rate: producer_bin_rate[t],
                    merge_tap: true,
                });
                // Consumers assigned to this trunk, in row order so the tap
                // sequence down the trunk is deterministic (find_tap_off_ys
                // resolves each to its y below).
                let mut consumers = consumer_bins[t].clone();
                consumers.sort_unstable();
                result.push(BusLane {
                    item: lane.item.clone(),
                    module_id: lane.module_id,
                    x: 0,
                    source_y: band_y + 1,
                    consumer_rows: consumers,
                    producer_row: None,
                    rate: producer_bin_rate[t].max(consumer_bin_rate[t]),
                    is_fluid: false,
                    family_id: Some(fid),
                    ..Default::default()
                });
            }
            continue;
        }

        let mut family_id: Option<usize> = None;
        let mut family_source_y: Option<i32> = None;

        // Stamp an N-to-M balancer for multi-trunk intermediate lanes.
        // The balancer absorbs N producers and feeds M trunks evenly so
        // every trunk runs at full-belt capacity.
        //   - Fan-out (N <= M): the original case (1→N, 2→3, etc.).
        //   - Parallel (N == M): added in a previous fix; without a
        //     family the inner trunks have no clean tile for `ret:`
        //     sideloads to land on.
        //   - Fan-in (N > M): new — when the unsplit lane wanted more
        //     trunks than consumers (`clamp_to_consumers`), we pinned
        //     M to the consumer count, so the family now lives at
        //     shape (N, consumer_count). Without the family, the
        //     producers in empty-consumer splits were silently dropped
        //     (producing belt-dead-end errors downstream).
        //   - HS fan-in to single trunk (any_hs, N > 1, M = 1): HS
        //     rows hold `force_single_row`, so DualInput consumers
        //     stay as one row even when producers split into multiple.
        //     Without a (N,1) family the producers sideload onto the
        //     single trunk, exceeding belt cap. Scoped to HS so
        //     existing VerticalSplit layouts (which work fine without
        //     this family) keep their golden hashes.
        let hs_single_trunk_fan_in = any_hs && n_producers > 1 && n_lanes_with_consumers == 1;
        if (n_producers >= 1 && n_lanes_with_consumers >= 2) || hs_single_trunk_fan_in {
            let shape = (n_producers, n_lanes_with_consumers);
            family_id = Some(families.len());

            let balancer_y_start = if n_producers == 1 {
                row_spans[all_producer_rows[0]].output_belt_y
            } else {
                all_producer_rows.iter()
                    .map(|&p| row_spans[p].y_end)
                    .max()
                    .unwrap_or(0)
            };

            families.push(LaneFamily {
                item: lane.item.clone(),
                // Inherit the parent lane's module_id so the split
                // family stays inside its module's identity. Under
                // Pooled this is always 0; under PartitionedDecomposed
                // it preserves the (item, module_id) keying that the
                // top of `plan_bus_lanes` relies on.
                module_id: lane.module_id,
                shape,
                producer_rows: all_producer_rows.to_vec(),
                lane_xs: Vec::new(),
                balancer_y_start,
                balancer_y_end: balancer_y_start,
                total_rate: lane.rate,
                merge_tap: false,
            });
            family_source_y = Some(balancer_y_start + 1);
        }

        // Create split lanes
        for si in 0..effective_n_splits {
            let consumers = consumers_per_split[si].clone();
            // HS lanes are always retained — every required trunk slot
            // needs its own BusLane. Plan-driven pad lanes are also
            // retained (empty-consumer trunks present so the family
            // balancer can stamp at the padded shape — apply_shape_fixes
            // PadLanes path).
            if !any_hs && !pad_active && consumers.is_empty() && !is_collector && si > 0 {
                continue;  // skip empty splits
            }
            let split_rate = lane.rate / effective_n_splits as f64;
            let hs_trunk_idx = hs_idx_per_split[si];

            if let Some(fid) = family_id {
                result.push(BusLane {
                    item: lane.item.clone(),
                    module_id: lane.module_id,
                    x: 0,
                    source_y: family_source_y.unwrap_or(0),
                    consumer_rows: consumers,
                    producer_row: None,
                    rate: split_rate,
                    is_fluid: false,
                    family_id: Some(fid),
                    hs_trunk_idx,
                    ..Default::default()
                });
                continue;
            }

            let prods = &producers_per_split[si];
            let first_prod = if prods.is_empty() { None } else { Some(prods[0]) };
            let extra_prods = if prods.len() > 1 { prods[1..].to_vec() } else { Vec::new() };
            let split_source_y = if prods.is_empty() {
                lane.source_y
            } else {
                prods.iter()
                    .map(|&p| row_spans[p].output_belt_y)
                    .min()
                    .unwrap_or(lane.source_y)
            };

            result.push(BusLane {
                item: lane.item.clone(),
                module_id: lane.module_id,
                x: 0,
                source_y: split_source_y,
                consumer_rows: consumers,
                producer_row: first_prod,
                rate: split_rate,
                is_fluid: false,
                extra_producer_rows: extra_prods,
                hs_trunk_idx,
                ..Default::default()
            });
        }
    }

    Ok((result, families))
}

/// Find y-coordinates where this lane taps off into consumer rows.
fn find_tap_off_ys(lane: &BusLane, row_spans: &[RowSpan]) -> Vec<i32> {
    let mut tap_ys: Vec<i32> = Vec::new();

    // HS input₀ lanes tap off at the consumer row's
    // `horizontal_stack.trunk_ys[trunk_idx]`. The lane's single consumer
    // is the HS row by construction (see `split_overflowing_lanes`).
    if let (Some(idx), Some(&ri)) = (lane.hs_trunk_idx, lane.consumer_rows.first()) {
        if let Some(hs) = row_spans[ri].horizontal_stack.as_ref() {
            if let Some(&y) = hs.trunk_ys.get(idx) {
                tap_ys.push(y);
                return tap_ys;
            }
        }
    }

    for &ri in &lane.consumer_rows {
        let rs = &row_spans[ri];
        if lane.is_fluid {
            // Fluid lanes tap off at the fluid port y positions
            if !rs.fluid_port_ys.is_empty() {
                tap_ys.push(rs.fluid_port_ys[0]);
            }
        } else {
            // Solid lanes
            let solid_inputs: Vec<_> = rs.spec.inputs.iter()
                .filter(|f| !f.is_fluid)
                .collect();
            for (input_idx, inp) in solid_inputs.iter().enumerate() {
                if inp.item == lane.item && input_idx < rs.input_belt_y.len() {
                    tap_ys.push(rs.input_belt_y[input_idx]);
                    break;
                }
            }
        }
    }

    tap_ys
}

/// Return the total bus width needed for the given lanes.
pub fn bus_width_for_lanes(lanes: &[BusLane]) -> i32 {
    // Derived from the max assigned column, not the lane count — surplus
    // lanes insert spacer columns (see the x-assignment loop), so count
    // and extent can differ. Identical to the old `len + 2` when columns
    // are gapless.
    match lanes.iter().map(|l| l.x).max() {
        None => 2,
        Some(max_x) => max_x + 2,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemFlow, MachineSpec};

    fn make_test_row_span(
        recipe: &str,
        y_start: i32,
        inputs: Vec<ItemFlow>,
        outputs: Vec<ItemFlow>,
        machine_count: usize,
        input_belt_y: Vec<i32>,
    ) -> RowSpan {
        RowSpan {
            y_start,
            y_end: y_start + 3,
            spec: MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: recipe.to_string(),
                self_loop: vec![], voider: false,
                count: machine_count as f64,
                inputs,
                outputs,
            },
            machine_count,
            module_id: 0,
            input_belt_y,
            output_belt_y: y_start + 2,
            row_width: 10,
            fluid_port_ys: Vec::new(),
            fluid_port_pipes: Vec::new(),
            fluid_output_port_pipes: Vec::new(),
            output_east: true,
            output_belt_x_min: 0,
            output_belt_x_max: 9,
            horizontal_stack: None,
            secondary_output_belt: None,
            sorted_output_belts: Vec::new(),
        }
    }

    #[test]
    fn test_bus_width_for_lanes_empty() {
        assert_eq!(bus_width_for_lanes(&[]), 2);
    }

    #[test]
    fn test_bus_width_for_lanes_single() {
        // Width derives from the max assigned column (production assigns
        // x = i+1, plus spacer columns for surplus lanes).
        let lane = BusLane {
            item: "iron-ore".to_string(),
            x: 1,
            ..Default::default()
        };
        assert_eq!(bus_width_for_lanes(&[lane]), 3);
    }

    #[test]
    fn test_bus_width_for_lanes_three() {
        let lanes: Vec<BusLane> = ["iron-ore", "copper-ore", "coal"]
            .iter()
            .enumerate()
            .map(|(i, item)| BusLane {
                item: item.to_string(),
                x: (i + 1) as i32,
                ..Default::default()
            })
            .collect();
        assert_eq!(bus_width_for_lanes(&lanes), 5);
    }

    #[test]
    fn test_bus_width_for_lanes_counts_spacer_gaps() {
        // A pure-surplus lane takes a spacer column; width must follow the
        // max x, not the lane count.
        let lanes: Vec<BusLane> = [1i32, 3, 4]
            .iter()
            .map(|&x| BusLane { item: format!("item-{x}"), x, ..Default::default() })
            .collect();
        assert_eq!(bus_width_for_lanes(&lanes), 6);
    }

    #[test]
    fn test_find_tap_off_ys_single_consumer() {
        let lane = BusLane {
            item: "iron-ore".to_string(),
            consumer_rows: vec![0],
            is_fluid: false,
            ..Default::default()
        };

        let row_span = make_test_row_span(
            "iron-plate",
            0,
            vec![ItemFlow { item: "iron-ore".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            1,
            vec![1],
        );

        let tap_ys = find_tap_off_ys(&lane, &[row_span]);
        assert_eq!(tap_ys, vec![1]);
    }


    fn make_solver_result_iron_gear_wheel() -> crate::models::SolverResult {
        crate::models::SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "iron-gear-wheel".to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![ItemFlow { item: "iron-plate".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
                outputs: vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            }],
            external_inputs: vec![ItemFlow { item: "iron-plate".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            external_outputs: vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            surplus_outputs: vec![],
            dependency_order: vec!["iron-gear-wheel".to_string()],
        }
    }

    fn make_solver_result_plastic_bar() -> crate::models::SolverResult {
        crate::models::SolverResult {
            machines: vec![MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: "plastic-bar".to_string(),
                self_loop: vec![], voider: false,
                count: 1.0,
                inputs: vec![
                    ItemFlow { item: "coal".to_string(), rate: 1.5, is_fluid: false, module_id: 0 },
                    ItemFlow { item: "petroleum-gas".to_string(), rate: 2.0, is_fluid: true, module_id: 0 },
                ],
                outputs: vec![ItemFlow { item: "plastic-bar".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            }],
            external_inputs: vec![
                ItemFlow { item: "coal".to_string(), rate: 1.5, is_fluid: false, module_id: 0 },
                ItemFlow { item: "petroleum-gas".to_string(), rate: 2.0, is_fluid: true, module_id: 0 },
            ],
            external_outputs: vec![ItemFlow { item: "plastic-bar".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            surplus_outputs: vec![],
            dependency_order: vec!["plastic-bar".to_string()],
        }
    }

    #[test]
    fn test_plan_bus_lanes_iron_gear_wheel_single_solid_input() {
        let sr = make_solver_result_iron_gear_wheel();

        // One consumer row for the iron-gear-wheel machine.
        let row_span = make_test_row_span(
            "iron-gear-wheel",
            5,
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            1,
            vec![6],  // input belt at y=6
        );

        let (lanes, families) = plan_bus_lanes(&sr, &[row_span], None, None, 40, false)
            .expect("plan_bus_lanes should succeed for iron-gear-wheel");

        // Should have exactly 1 lane for iron-plate
        assert_eq!(lanes.len(), 1, "Expected exactly 1 lane (iron-plate), got {:?}", lanes.iter().map(|l| &l.item).collect::<Vec<_>>());
        assert_eq!(lanes[0].item, "iron-plate");
        assert!(!lanes[0].is_fluid, "iron-plate lane should not be fluid");
        assert_eq!(families.len(), 0, "No balancer family needed for 1 external input");

        // Lane x should be assigned (>= 1)
        assert!(lanes[0].x >= 1, "Lane x should be >= 1 after assignment");
    }

    #[test]
    fn test_plan_bus_lanes_iron_gear_wheel_lane_count() {
        let sr = make_solver_result_iron_gear_wheel();

        let row_span = make_test_row_span(
            "iron-gear-wheel",
            5,
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            1,
            vec![6],
        );

        let (lanes, _families) = plan_bus_lanes(&sr, &[row_span], None, None, 40, false).unwrap();

        // iron-gear-wheel is the final output, not consumed internally, so no lane for it
        // Only iron-plate (the external input) needs a lane
        let item_names: Vec<&str> = lanes.iter().map(|l| l.item.as_str()).collect();
        assert!(item_names.contains(&"iron-plate"), "iron-plate lane expected");
        assert!(!item_names.contains(&"iron-gear-wheel"), "iron-gear-wheel is final output, should not get a bus lane");
    }

    #[test]
    fn test_plan_bus_lanes_plastic_bar_fluid_lane_created() {
        let sr = make_solver_result_plastic_bar();

        let row_span = make_test_row_span(
            "plastic-bar",
            5,
            vec![
                ItemFlow { item: "coal".to_string(), rate: 1.5, is_fluid: false, module_id: 0 },
                ItemFlow { item: "petroleum-gas".to_string(), rate: 2.0, is_fluid: true, module_id: 0 },
            ],
            vec![ItemFlow { item: "plastic-bar".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            1,
            vec![6, 7],  // two input belt y positions
        );

        let (lanes, _families) = plan_bus_lanes(&sr, &[row_span], None, None, 40, false)
            .expect("plan_bus_lanes should succeed for plastic-bar");

        // Should have lanes for coal and petroleum-gas (plastic-bar is final output)
        let item_names: Vec<&str> = lanes.iter().map(|l| l.item.as_str()).collect();
        assert!(item_names.contains(&"coal"), "coal lane expected");
        assert!(item_names.contains(&"petroleum-gas"), "petroleum-gas lane expected");

        // petroleum-gas lane must be fluid
        let pg_lane = lanes.iter().find(|l| l.item == "petroleum-gas")
            .expect("petroleum-gas lane must exist");
        assert!(pg_lane.is_fluid, "petroleum-gas lane must have is_fluid=true");

        // coal lane must not be fluid
        let coal_lane = lanes.iter().find(|l| l.item == "coal")
            .expect("coal lane must exist");
        assert!(!coal_lane.is_fluid, "coal lane must have is_fluid=false");
    }

    #[test]
    fn test_plan_bus_lanes_fluid_not_first() {
        // Solid lanes should come before fluid lanes in the ordering
        let sr = make_solver_result_plastic_bar();

        let row_span = make_test_row_span(
            "plastic-bar",
            5,
            vec![
                ItemFlow { item: "coal".to_string(), rate: 1.5, is_fluid: false, module_id: 0 },
                ItemFlow { item: "petroleum-gas".to_string(), rate: 2.0, is_fluid: true, module_id: 0 },
            ],
            vec![ItemFlow { item: "plastic-bar".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            1,
            vec![6, 7],
        );

        let (lanes, _families) = plan_bus_lanes(&sr, &[row_span], None, None, 40, false).unwrap();

        // optimize_lane_order puts solid before fluid
        let fluid_indices: Vec<usize> = lanes.iter().enumerate()
            .filter(|(_, l)| l.is_fluid)
            .map(|(i, _)| i)
            .collect();
        let solid_indices: Vec<usize> = lanes.iter().enumerate()
            .filter(|(_, l)| !l.is_fluid)
            .map(|(i, _)| i)
            .collect();

        if !fluid_indices.is_empty() && !solid_indices.is_empty() {
            let last_solid = *solid_indices.iter().max().unwrap();
            let first_fluid = *fluid_indices.iter().min().unwrap();
            assert!(last_solid < first_fluid, "All solid lanes should come before fluid lanes");
        }
    }

    #[test]
    fn test_plan_bus_lanes_consumer_row_must_have_tap_off_y() {
        let sr = make_solver_result_iron_gear_wheel();

        let row_span = make_test_row_span(
            "iron-gear-wheel",
            5,
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 1.0, is_fluid: false, module_id: 0 }],
            1,
            vec![6],
        );

        let (lanes, _families) = plan_bus_lanes(&sr, &[row_span], None, None, 40, false).unwrap();

        // The iron-plate lane has consumer row 0, so it should have a tap-off y
        let iron_plate_lane = lanes.iter().find(|l| l.item == "iron-plate").unwrap();
        assert!(!iron_plate_lane.consumer_rows.is_empty(), "iron-plate lane should have consumer rows");
        assert!(!iron_plate_lane.tap_off_ys.is_empty(), "iron-plate lane should have tap-off y after plan");
    }

    // -----------------------------------------------------------------------
    // route_lane / route_belt_lane tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_plan_bus_lanes_via_solver_iron_gear_wheel() {
        use crate::solver::solve;
        use rustc_hash::FxHashSet;

        let available: FxHashSet<String> = ["iron-plate"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let sr = solve("iron-gear-wheel", 10.0, &available, "assembling-machine-3")
            .expect("solver should succeed");

        // Build minimal row spans from solver machines
        let row_spans: Vec<RowSpan> = sr.machines.iter().enumerate().map(|(i, m)| {
            let input_belt_y: Vec<i32> = m.inputs.iter().enumerate()
                .filter(|(_, f)| !f.is_fluid)
                .map(|(idx, _)| (i * 5 + idx) as i32)
                .collect();
            RowSpan {
                y_start: (i * 5) as i32,
                y_end: (i * 5 + 3) as i32,
                spec: m.clone(),
                machine_count: m.count.ceil() as usize,
                module_id: 0,
                input_belt_y,
                output_belt_y: (i * 5 + 2) as i32,
                row_width: 10,
                fluid_port_ys: Vec::new(),
                fluid_port_pipes: Vec::new(),
                fluid_output_port_pipes: Vec::new(),
                output_east: true,
                output_belt_x_min: 0,
                output_belt_x_max: 9,
                horizontal_stack: None,
                secondary_output_belt: None,
                sorted_output_belts: Vec::new(),
            }
        }).collect();

        let (lanes, _families) = plan_bus_lanes(&sr, &row_spans, None, None, 40, false)
            .expect("plan_bus_lanes should succeed");

        // Must have at least one lane
        assert!(!lanes.is_empty(), "Expected at least one bus lane");

        // Each lane must have its x assigned (>= 1)
        for lane in &lanes {
            assert!(lane.x >= 1, "Lane x must be assigned >= 1, got x={} for item={}", lane.x, lane.item);
        }

        // No two lanes should share the same x column
        let xs: Vec<i32> = lanes.iter().map(|l| l.x).collect();
        let xs_set: std::collections::HashSet<i32> = xs.iter().copied().collect();
        assert_eq!(xs.len(), xs_set.len(), "All lane x columns must be unique");
    }
}
