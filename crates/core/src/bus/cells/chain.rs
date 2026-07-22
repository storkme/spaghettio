//! RFC-051 Phase B: the linear/fan-out chain auto-placer.
//!
//! Generalizes the Phase-1 hand composers: cells are engine-generated
//! per chain recipe (K=1 — one cell per recipe sized by the chain
//! solve's machine counts; ratio quantization is a later optimization),
//! placed west→east in dependency order, wired with template corridors
//! only (straight, corner, UG-hop, 2→1 merge splitter, 1→2 fan-out
//! splitter). Fan-out past an intervening cell routes through a SOUTH
//! BYPASS lane below the cell band — the south side is empty except the
//! final drain, so the only crossings are known corridor rows, each
//! resolved by a local UG hop.
//!
//! Geometry-mode decision (Phase B, 2026-07-22): there is ONE geometry —
//! the calibrated (sim-kit-compatible) form, 4-tile feed pitch included.
//! The RFC's calibrated-twin invariant demands bit-identical
//! non-boundary entities between production and sim forms; a compacted
//! production form would shift cell origins and violate it. Bit-identity
//! outranks the area optimization (that overhead is ~24% and honest);
//! compacting under a translation-aware invariant is future work.

use rustc_hash::{FxHashMap, FxHashSet};
use crate::models::{BoundaryRecord, EntityDirection, LayoutResult, PlacedEntity, SolverResult};

use super::extract::{extract_cell, generate_cell_layout, Cell, Port};
use super::compose::stamp_path;

/// Feed columns need >=4 tiles of lateral separation (#363: sim-kit
/// rigs collide at construction below that).
const FEED_PITCH: i32 = 4;
/// North margin: boundary row at y=0, then clearance for feed corners.
const CELL_Y: i32 = 3;
/// Horizontal clearance east of each cell for merges/fan-out/corridors.
const CORRIDOR_GAP: i32 = 6;
/// Vertical lanes reserved on the east side of each slot's feed block
/// for bypass descents/ascents.
const VLANES: i32 = 2;

/// Why a solve is not chain-composable. Stable strings — the candidate
/// reports these as its `accepted_reason`.
pub fn chain_eligible(sr: &SolverResult) -> Result<(), String> {
    if sr.machines.is_empty() {
        return Err("cells: empty chain".into());
    }
    let mut producers: FxHashMap<&str, usize> = FxHashMap::default();
    for m in &sr.machines {
        if m.count <= 0.0 {
            return Err(format!("cells: zero-count spec {}", m.recipe));
        }
        for o in &m.outputs {
            if o.is_fluid {
                return Err(format!("cells: fluid output {} (solid-only Phase B)", o.item));
            }
            *producers.entry(o.item.as_str()).or_default() += 1;
        }
        for i in &m.inputs {
            if i.is_fluid {
                return Err(format!("cells: fluid input {} (solid-only Phase B)", i.item));
            }
        }
        if !m.self_loop.is_empty() {
            return Err(format!("cells: self-loop recipe {}", m.recipe));
        }
    }
    for (item, n) in &producers {
        if *n > 1 {
            return Err(format!("cells: {item} produced by {n} specs (need exactly 1)"));
        }
    }
    // Corridor capacity: every produced item rides ONE express corridor
    // (45/s). Run-matching (bundled corridors) is future work — refuse
    // honestly past the cap.
    for m in &sr.machines {
        for o in &m.outputs {
            let total = o.rate * m.count;
            if total > 45.0 + 1e-9 {
                return Err(format!(
                    "cells: {} at {total:.1}/s exceeds single-corridor capacity 45/s (run matching unimplemented)",
                    o.item
                ));
            }
        }
    }
    Ok(())
}

struct Placed {
    cell: Cell,
    /// Absolute x of the cell's west edge.
    x: i32,
    /// Absolute x of the slot's west edge (feed block start).
    slot_x: i32,
    /// Base x of this slot's vertical-lane strip (east of feed columns).
    vlane_base: i32,
    recipe: String,
    ext_inputs: Vec<String>,
}

fn port_abs(p: &Port, cell_x: i32) -> (i32, i32) {
    (cell_x + p.x, CELL_Y + p.y)
}

/// Crossing-aware corridor stamper. Horizontal runs hop under
/// registered vertical columns; vertical legs hop under registered
/// horizontal rows; whichever is stamped LATER does the hopping, so
/// stamp order between two crossing corridors doesn't matter. All hops
/// are span-3 UG pairs (within every belt tier's reach) — template
/// machinery only (kill 5).
struct Router {
    h_rows: Vec<(i32, i32, i32)>, // (y, x0, x1) inclusive
    v_cols: Vec<(i32, i32, i32)>, // (x, y0, y1) inclusive
}

impl Router {
    fn new() -> Self {
        Router { h_rows: Vec::new(), v_cols: Vec::new() }
    }

    /// Register an externally stamped column (feed columns).
    fn register_col(&mut self, x: i32, y0: i32, y1: i32) {
        self.v_cols.push((x, y0.min(y1), y0.max(y1)));
    }
    fn register_row(&mut self, y: i32, x0: i32, x1: i32) {
        self.h_rows.push((y, x0.min(x1), x0.max(x1)));
    }

    /// Eastward row from x0..=x1 at y, hopping under crossing columns.
    #[allow(clippy::too_many_arguments)]
    fn hrow(
        &mut self,
        out: &mut Vec<PlacedEntity>,
        y: i32,
        x0: i32,
        x1: i32,
        item: &str,
        belt: &str,
        ug: &str,
        seg: &str,
    ) {
        let mut cols: Vec<i32> = self
            .v_cols
            .iter()
            .filter(|(cx, cy0, cy1)| *cx > x0 && *cx < x1 && y >= *cy0 && y <= *cy1)
            .map(|(cx, _, _)| *cx)
            .collect();
        cols.sort_unstable();
        cols.dedup();
        let push_east = |out: &mut Vec<PlacedEntity>, xa: i32, xb: i32| {
            for x in xa..=xb {
                out.push(PlacedEntity {
                    name: belt.into(), x, y,
                    direction: EntityDirection::East,
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
        };
        // Cluster columns closer than 3 tiles: independent per-column
        // hops would share tiles (exit of one = entry of the next).
        let mut clusters: Vec<(i32, i32)> = Vec::new();
        for c in cols {
            match clusters.last_mut() {
                Some((_, hi)) if c - *hi < 3 => *hi = c,
                _ => clusters.push((c, c)),
            }
        }
        let mut x = x0;
        for (lo2, hi2) in clusters {
            assert!(hi2 - lo2 + 2 <= 9, "cells: hop cluster span exceeds express reach");
            if lo2 - 2 >= x {
                push_east(out, x, lo2 - 2);
            }
            for (hx, io) in [(lo2 - 1, "input"), (hi2 + 1, "output")] {
                out.push(PlacedEntity {
                    name: ug.into(),
                    x: hx,
                    y,
                    direction: EntityDirection::East,
                    io_type: Some(io.into()),
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
            x = hi2 + 2;
        }
        if x <= x1 {
            push_east(out, x, x1);
        }
        self.register_row(y, x0, x1);
    }

    /// Vertical leg from y0 toward y1 at x (either direction), hopping
    /// under crossing rows.
    #[allow(clippy::too_many_arguments)]
    fn vcol(
        &mut self,
        out: &mut Vec<PlacedEntity>,
        x: i32,
        y0: i32,
        y1: i32,
        item: &str,
        belt: &str,
        ug: &str,
        seg: &str,
    ) {
        let (lo, hi) = (y0.min(y1), y0.max(y1));
        let down = y1 > y0;
        let (dir, io_near, io_far) = if down {
            (EntityDirection::South, "input", "output")
        } else {
            (EntityDirection::North, "input", "output")
        };
        let mut rows: Vec<i32> = self
            .h_rows
            .iter()
            .filter(|(ry, rx0, rx1)| *ry > lo && *ry < hi && x >= *rx0 && x <= *rx1)
            .map(|(ry, _, _)| *ry)
            .collect();
        rows.sort_unstable();
        if !down {
            rows.reverse();
        }
        let step = if down { 1 } else { -1 };
        let push_v = |out: &mut Vec<PlacedEntity>, ya: i32, yb: i32| {
            let (lo2, hi2) = (ya.min(yb), ya.max(yb));
            for y in lo2..=hi2 {
                out.push(PlacedEntity {
                    name: belt.into(), x, y,
                    direction: dir,
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
        };
        let mut clusters: Vec<(i32, i32)> = Vec::new();
        for r in rows {
            match clusters.last_mut() {
                Some((_, last)) if (r - *last) * step < 3 => *last = r,
                _ => clusters.push((r, r)),
            }
        }
        let mut y = y0;
        for (first, last) in clusters {
            assert!((last - first).abs() + 2 <= 9, "cells: hop cluster span exceeds express reach");
            if (first - 2 * step - y) * step >= 0 {
                push_v(out, y, first - 2 * step);
            }
            for (hy, io) in [(first - step, io_near), (last + step, io_far)] {
                out.push(PlacedEntity {
                    name: ug.into(),
                    x,
                    y: hy,
                    direction: dir,
                    io_type: Some(io.into()),
                    carries: Some(item.into()),
                    segment_id: Some(seg.into()),
                    ..Default::default()
                });
            }
            y = last + 2 * step;
        }
        if (y1 - y) * step >= 0 {
            push_v(out, y, y1);
        }
        self.register_col(x, lo, hi);
    }

    /// North-facing corner belt at (x, y): single perpendicular input.
    fn corner_north(&mut self, out: &mut Vec<PlacedEntity>, x: i32, y: i32, item: &str, belt: &str, seg: &str) {
        out.push(PlacedEntity {
            name: belt.into(),
            x,
            y,
            direction: EntityDirection::North,
            carries: Some(item.into()),
            segment_id: Some(seg.into()),
            ..Default::default()
        });
        self.register_col(x, y, y);
    }

    /// East-facing corner belt at (x, y): single perpendicular input =
    /// lane-preserving corner (the post-review splitter-merge idiom).
    fn corner_east(&mut self, out: &mut Vec<PlacedEntity>, x: i32, y: i32, item: &str, belt: &str, seg: &str) {
        out.push(PlacedEntity {
            name: belt.into(),
            x,
            y,
            direction: EntityDirection::East,
            carries: Some(item.into()),
            segment_id: Some(seg.into()),
            ..Default::default()
        });
        self.register_row(y, x, x);
    }
}

/// Compose an eligible chain solve into one layout. K=1: one cell per
/// recipe at the chain rate. Returns the composed layout with boundary
/// records populated (calibrated orientation: north feeds, south drain,
/// west→east record order — #363).
pub fn compose_chain(sr: &SolverResult) -> Result<LayoutResult, String> {
    chain_eligible(sr)?;

    let produced: FxHashSet<&str> = sr
        .machines
        .iter()
        .flat_map(|m| m.outputs.iter().map(|o| o.item.as_str()))
        .collect();

    // Place producers-first, west→east. `sr.dependency_order` is
    // TARGET-FIRST (the solver's DFS pushes a recipe before recursing
    // into its ingredients), so reverse it; unlisted recipes go last.
    let mut specs: Vec<&crate::models::MachineSpec> = sr.machines.iter().collect();
    let pos: FxHashMap<&str, usize> = sr
        .dependency_order
        .iter()
        .enumerate()
        .map(|(i, r)| (r.as_str(), i))
        .collect();
    specs.sort_by_key(|m| match pos.get(m.recipe.as_str()) {
        Some(&i) => (0, std::cmp::Reverse(i)),
        None => (1, std::cmp::Reverse(usize::MAX)),
    });

    // Per-slot vertical-lane demand, from the bypass edge list (an edge
    // p→c descends in slot p+1 and ascends in slot c; sizing by the
    // slot's own fan-out under-counted ascents — the mil5-ore overlap
    // class).
    let n = specs.len();
    let mut lane_demand: Vec<i32> = vec![0; n];
    for (pi, m) in specs.iter().enumerate() {
        for o in &m.outputs {
            for (ci, c) in specs.iter().enumerate() {
                if ci != pi && c.inputs.iter().any(|i| i.item == o.item) && ci != pi + 1 {
                    if pi + 1 < n {
                        lane_demand[pi + 1] += 1;
                    }
                    lane_demand[ci] += 1;
                }
            }
        }
    }

    let mut entities: Vec<PlacedEntity> = Vec::new();
    let mut b_in: Vec<BoundaryRecord> = Vec::new();
    let mut b_out: Vec<BoundaryRecord> = Vec::new();
    let mut placed: Vec<Placed> = Vec::new();
    let mut cursor = 0i32;

    for m in &specs {
        let out_item = m
            .outputs
            .first()
            .ok_or_else(|| format!("cells: {} has no output", m.recipe))?
            .item
            .clone();
        // outputs[].rate is PER-MACHINE; the cell serves the whole spec.
        let rate = m.outputs[0].rate * m.count;
        let input_names: Vec<&str> = m.inputs.iter().map(|i| i.item.as_str()).collect();
        let (_csr, cl) = generate_cell_layout(&out_item, rate, &input_names);
        let cell = extract_cell(&cl);
        let ext_inputs: Vec<String> = m
            .inputs
            .iter()
            .filter(|i| !produced.contains(i.item.as_str()))
            .map(|i| i.item.clone())
            .collect();
        let n_feed_ports = cell
            .ports
            .iter()
            .filter(|q| q.inbound && ext_inputs.contains(&q.item))
            .count() as i32;

        let n_out_runs = cell.ports.iter().filter(|q| !q.inbound).count() as i32;
        let n_consumers = sr
            .machines
            .iter()
            .filter(|c| c.inputs.iter().any(|i| i.item == out_item))
            .count() as i32;
        // Gap: base + 2 per extra merge stage + 2 per extra fan-out stage.
        let gap = CORRIDOR_GAP + 2 * (n_out_runs - 1).max(0) + 2 * (n_consumers - 1).max(0);
        let slot_x = cursor;
        let feed_w = FEED_PITCH * n_feed_ports + 1;
        let vlane0 = slot_x + feed_w;
        let strip = VLANES + lane_demand[placed.len()];
        let x = vlane0 + strip + 1;
        cursor = x + cell.width + gap;

        for e in &cell.entities {
            let mut e = e.clone();
            e.x += x;
            e.y += CELL_Y;
            entities.push(e);
        }
        placed.push(Placed {
            cell,
            x,
            slot_x,
            vlane_base: vlane0,
            recipe: m.recipe.clone(),
            ext_inputs,
        });
    }

    let band_bottom = CELL_Y
        + placed.iter().map(|p| p.cell.height).max().unwrap_or(0)
        + 1;

    // --- External feeds: per cell, columns west of it (pitch 4), north
    // boundary at y=0, corner east into the port terminal. Inner column
    // serves the topmost port (no crossings among a slot's own feeds).
    let mut router = Router::new();
    for p in &placed {
        // MULTI-ROW cells expose one in-port PER ROW for the same item —
        // every port gets its own feed column (a single-port find left
        // second-row machines unfed: belt-flow-reachability caught it).
        let mut targets: Vec<(String, i32, i32)> = Vec::new();
        for item in &p.ext_inputs {
            let mut found = false;
            for port in p.cell.ports.iter().filter(|q| q.inbound && q.item == *item) {
                let (tx, ty) = port_abs(port, p.x);
                targets.push((item.clone(), tx, ty));
                found = true;
            }
            if !found {
                return Err(format!("cells: {} lacks in-port for {item}", p.recipe));
            }
        }
        targets.sort_by_key(|t| t.2);
        for (i, (item, tx, ty)) in targets.iter().enumerate() {
            let col_x = p.slot_x + targets.len() as i32 * FEED_PITCH
                - FEED_PITCH * i as i32 - FEED_PITCH + 1;
            stamp_path(
                &mut entities,
                &[(col_x, 0), (col_x, *ty), (tx - 1, *ty)],
                item,
                "express-transport-belt",
                &format!("feed:{item}:{}", p.recipe),
            );
            router.register_col(col_x, 0, *ty);
            router.register_row(*ty, col_x, tx - 1);
            b_in.push(BoundaryRecord {
                item: item.clone(),
                x: col_x,
                y: 0,
                direction: EntityDirection::South,
                is_fluid: false,
                entity: "express-transport-belt".into(),
            });
        }
    }

    // Bypass rows sit between the band bottom and the drain row, so the
    // sim's drain rig (which builds south of the drain head) never
    // collides with them. Count bypass edges up front.
    let n_bypass: i32 = placed
        .iter()
        .enumerate()
        .map(|(pi, _)| {
            let out_item = &specs[pi].outputs[0].item;
            placed
                .iter()
                .enumerate()
                .filter(|(ci, c)| {
                    *ci != pi
                        && *ci != pi + 1
                        && specs[*ci].inputs.iter().any(|i| i.item == *out_item)
                        && c.cell.ports.iter().any(|q| q.inbound && q.item == *out_item)
                })
                .count() as i32
        })
        .sum();
    let drain_row = band_bottom + n_bypass + 2;

    // --- Chain corridors: producer out → merge (if 2 runs) → fan-out
    // split (if 2 consumers) → per-consumer routing via the Router.
    let mut bypass_idx = 0i32;
    // Per-slot vertical-lane allocation: each bypass descent/ascent
    // claims a fresh lane in its slot's strip (two edges sharing a lane
    // was the mil5-ore overlap class).
    let mut lane_next: FxHashMap<usize, i32> = FxHashMap::default();
    let alloc_lane = |lane_next: &mut FxHashMap<usize, i32>, slot: usize, base: i32| -> i32 {
        let n = lane_next.entry(slot).or_insert(0);
        let x = base + *n;
        *n += 1;
        x
    };
    for (pi, p) in placed.iter().enumerate() {
        let out_item = specs[pi].outputs[0].item.clone();
        let consumers: Vec<usize> = placed
            .iter()
            .enumerate()
            .filter(|(ci, c)| {
                *ci != pi
                    && specs[*ci].inputs.iter().any(|i| i.item == out_item)
                    && c.cell.ports.iter().any(|q| q.inbound && q.item == out_item)
            })
            .map(|(ci, _)| ci)
            .collect();
        let outs: Vec<&Port> = p.cell.ports.iter().filter(|q| !q.inbound).collect();
        if consumers.is_empty() {
            // Final product: corner south past the band, drain record.
            let o1 = outs.first().ok_or_else(|| format!("cells: {} has no out port", p.recipe))?;
            let (ox, oy) = port_abs(o1, p.x);
            let drain_x = ox + 2;
            let seg = format!("out:{}", p.recipe);
            router.hrow(&mut entities, oy, ox + 1, drain_x - 1, &out_item,
                "transport-belt", "underground-belt", &seg);
            entities.push(PlacedEntity {
                name: "transport-belt".into(), x: drain_x, y: oy,
                direction: EntityDirection::South,
                carries: Some(out_item.clone()),
                segment_id: Some(seg.clone()), ..Default::default()
            });
            router.vcol(&mut entities, drain_x, oy + 1, drain_row, &out_item,
                "transport-belt", "underground-belt", &seg);
            b_out.push(BoundaryRecord {
                item: out_item.clone(),
                x: drain_x,
                y: drain_row,
                direction: EntityDirection::South,
                is_fluid: false,
                entity: "transport-belt".into(),
            });
            continue;
        }

        // Collect the cell's out-runs into ONE eastbound run via a
        // cascade of 2→1 splitters (below-approach corner idiom per
        // stage; the Router hops any crossings). Runs sorted by y — the
        // topmost is the accumulator row.
        let mut outs_sorted = outs.clone();
        outs_sorted.sort_by_key(|q| q.y);
        let (acc_x0, acc_y) = port_abs(outs_sorted[0], p.x);
        let base_sx = p.x + p.cell.width + 2;
        router.hrow(&mut entities, acc_y, acc_x0 + 1, base_sx - 1, &out_item,
            "express-transport-belt", "express-underground-belt", &format!("cc:a:{}", p.recipe));
        let mut run_x = base_sx;
        for (k, o) in outs_sorted.iter().enumerate().skip(1) {
            let (ox, oy) = port_abs(o, p.x);
            assert!(oy > acc_y + 1, "cells: merge assumes below-approach ({oy} vs {acc_y})");
            let seg = format!("cc:b{k}:{}", p.recipe);
            router.hrow(&mut entities, oy, ox + 1, run_x - 2, &out_item,
                "express-transport-belt", "express-underground-belt", &seg);
            router.corner_north(&mut entities, run_x - 1, oy, &out_item, "express-transport-belt", &seg);
            router.vcol(&mut entities, run_x - 1, oy - 1, acc_y + 2, &out_item,
                "express-transport-belt", "express-underground-belt", &seg);
            router.corner_east(&mut entities, run_x - 1, acc_y + 1, &out_item, "express-transport-belt", &seg);
            entities.push(PlacedEntity {
                name: "express-splitter".into(), x: run_x, y: acc_y,
                direction: EntityDirection::East,
                carries: Some(out_item.clone()),
                segment_id: Some(format!("cc:m{k}:{}", p.recipe)), ..Default::default()
            });
            run_x += 2;
            if k < outs_sorted.len() - 1 {
                // Bridge to the next merge stage's input tile.
                router.corner_east(&mut entities, run_x - 1, acc_y, &out_item, "fast-transport-belt", &format!("cc:a:{}", p.recipe));
            }
        }
        let run_y = acc_y;
        // After a merge cascade the collected flow's next free tile is
        // run_x - 1 (the last splitter sits at run_x - 2); with a single
        // out-run nothing was consumed east of the hrow, so it's run_x.
        let pass_x = if outs_sorted.len() > 1 { run_x - 1 } else { run_x };

        // Fan-out: a chain of 1→2 splitters, one per extra consumer.
        // Branch b exits south at splitter b's (x+1, y+1); the last
        // consumer takes the pass-through east output.
        let n_branches = consumers.len();
        let mut branch_origins: Vec<(i32, i32)> = Vec::new();
        let mut fx = pass_x;
        for b in 1..n_branches {
            entities.push(PlacedEntity {
                name: "express-splitter".into(), x: fx, y: run_y,
                direction: EntityDirection::East,
                carries: Some(out_item.clone()),
                segment_id: Some(format!("fan{b}:{}", p.recipe)), ..Default::default()
            });
            branch_origins.push((fx + 1, run_y + 1));
            if b < n_branches - 1 {
                router.corner_east(&mut entities, fx + 1, run_y, &out_item, "fast-transport-belt", &format!("fan:{}", p.recipe));
            }
            fx += 2;
        }
        // Pass-through (or the only) branch.
        branch_origins.push((if n_branches > 1 { fx - 1 } else { pass_x }, run_y));

        // Route each branch. Adjacent-east consumer: port-row corridor
        // (with a vertical jog on the consumer slot's first lane if the
        // rows differ). Farther consumer: south bypass under the band.
        let mut ordered = consumers.clone();
        ordered.sort_by_key(|ci| placed[*ci].x);
        for (bi, ci) in ordered.iter().enumerate() {
            let c = &placed[*ci];
            let port = c.cell.ports.iter()
                .find(|q| q.inbound && q.item == out_item)
                .expect("consumer port checked in eligibility");
            let (tx, ty) = port_abs(port, c.x);
            let (bx, by) = branch_origins[bi];
            let seg = format!("corr:{}:{}", p.recipe, c.recipe);
            if *ci == pi + 1 {
                if by == ty {
                    router.hrow(&mut entities, ty, bx, tx - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                } else {
                    // Early jog: one east tile at the branch origin, then
                    // vertical at bx+1 down/up to the TARGET port row, then
                    // east all the way. The stagger keeps a sibling
                    // fan-out branch's row clear of this jog column (it
                    // hops under it via the registry).
                    let vdir = (ty - by).signum();
                    router.corner_east(&mut entities, bx, by, &out_item, "express-transport-belt", &seg);
                    router.vcol(&mut entities, bx + 1, by, ty - vdir, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                    router.corner_east(&mut entities, bx + 1, ty, &out_item, "express-transport-belt", &seg);
                    router.hrow(&mut entities, ty, bx + 2, tx - 1, &out_item,
                        "express-transport-belt", "express-underground-belt", &seg);
                }
            } else {
                // South bypass below the cell band.
                let lane_down = alloc_lane(&mut lane_next, pi + 1, placed[pi + 1].vlane_base);
                let lane_up = alloc_lane(&mut lane_next, *ci, c.vlane_base);
                let by_y = band_bottom + 1 + bypass_idx;
                bypass_idx += 1;
                router.hrow(&mut entities, by, bx, lane_down - 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                router.vcol(&mut entities, lane_down, by, by_y - 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                router.corner_east(&mut entities, lane_down, by_y, &out_item, "express-transport-belt", &seg);
                router.hrow(&mut entities, by_y, lane_down + 1, lane_up - 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                entities.push(PlacedEntity {
                    name: "express-transport-belt".into(), x: lane_up, y: by_y,
                    direction: EntityDirection::North,
                    carries: Some(out_item.clone()),
                    segment_id: Some(seg.clone()), ..Default::default()
                });
                router.vcol(&mut entities, lane_up, by_y - 1, ty + 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
                router.corner_east(&mut entities, lane_up, ty, &out_item, "express-transport-belt", &seg);
                router.hrow(&mut entities, ty, lane_up + 1, tx - 1, &out_item,
                    "express-transport-belt", "express-underground-belt", &seg);
            }
        }
    }

    // --- Poles: per-cell trio down the corridor gap + a spanning line
    // along the band bottom (nudge-not-skip — Phase-1 pole lesson).
    let occupied: FxHashSet<(i32, i32)> = entities.iter().map(|e| (e.x, e.y)).collect();
    for p in &placed {
        let px = p.x + p.cell.width + CORRIDOR_GAP - 1;
        for y in [CELL_Y, CELL_Y + 7, CELL_Y + 14] {
            if y < band_bottom {
                let mut yy = y;
                while occupied.contains(&(px, yy)) {
                    yy += 1;
                }
                entities.push(PlacedEntity {
                    name: "medium-electric-pole".into(), x: px, y: yy,
                    direction: EntityDirection::North,
                    segment_id: Some("pole".into()), ..Default::default()
                });
            }
        }
    }
    let width = entities.iter().map(|e| e.x).max().unwrap_or(0) + 1;
    let mut px = 1;
    while px < width {
        for nudge in 0..5 {
            let x = px + nudge;
            if !occupied.contains(&(x, band_bottom)) {
                entities.push(PlacedEntity {
                    name: "medium-electric-pole".into(), x, y: band_bottom,
                    direction: EntityDirection::North,
                    segment_id: Some("pole".into()), ..Default::default()
                });
                break;
            }
        }
        px += 8;
    }

    let height = (entities.iter().map(|e| e.y).max().unwrap_or(0) + 1).max(band_bottom + 2);
    Ok(LayoutResult {
        entities,
        width,
        height,
        stacking: 1,
        boundary_inputs: {
            let mut b = b_in;
            b.sort_by_key(|r| r.x); // west→east (#363 rig-depth rule)
            b
        },
        boundary_outputs: b_out,
        ..Default::default()
    })
}
