//! RFC-073 Phase 0 — the inserter sizing census.
//!
//! `inserter_ladder` sizes every machine side with `required <= n·rate`
//! and NO margin, so a side can ship with a hand at exactly 100% of what
//! the ladder credits it. The ec@240 receipts (RFC-072 P2 unit 2) showed
//! that such a side starves the row's tail in the sim — one machine short
//! per copy at 92.6% of one hand, two at 100% — while the same cell at 52%
//! of two hands runs at plan. Before putting a margin INTO the ladder
//! (which re-shapes every layout whose sides sit near the brim), this
//! census answers the prior question: across the receipted corpora, how
//! full are the hands the ladder ships, and does fullness predict the
//! measured deficit?
//!
//! The instrument is `TraceEvent::InserterSideSized`, emitted by the row
//! templates (`bus::templates`) for every side they size, with the plan's
//! own credited capacity. Coverage is exactly that: the nine direct
//! `size_side` calls in `bus::placer` (the DI bridge, the fused/straddle
//! cells) emit nothing and are the census's recorded gap.
//! [`capture`] runs a build under a collector AND a sink: `build_bus_layout`
//! detaches the sink for its pass 1 and replays pass-1 events only when no
//! retry follows, so the sink sees exactly the shipped passes while the
//! collector (which the replay reads from) sees everything. Wrap the
//! PLAIN `build_bus_layout` (or a composer) — the `_traced` / `_streaming`
//! entries install their own collector and sink and would clobber
//! `capture`'s, returning an empty census. [`side_loads`]
//! then joins the sink's events onto the machines of a layout when the
//! events are in that layout's frame (a native build; the selection loop
//! builds several candidates, so an event is only a shipped side if its
//! machine origin + recipe is in the final layout), or takes them as-is
//! for a composed layout (cells are generated once in their own frame and
//! cloned/translated, so their coordinates never match the composition —
//! [`side_loads_unjoined`]). Either way a key that saw more than one
//! distinct plan is reported as ambiguous rather than silently resolved.
//! [`summarize`] bands the input sides by utilization; output (belt-drop)
//! sides are counted but not banded — their rate is a measured min-form
//! with the lane cap already in it, and no receipt implicates them.

use crate::common::QualityTier;
use crate::models::LayoutResult;
use crate::trace::{self, TraceEvent};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// One sized machine side as the templates recorded it.
#[derive(Clone, Debug, PartialEq)]
pub struct SideLoad {
    pub recipe: String,
    pub side_is_output: bool,
    pub item: String,
    pub required: f64,
    pub entity: String,
    pub count: usize,
    /// The ladder's credit at the level the layout was SIZED at.
    pub capacity: f64,
    pub machine_x: i32,
    pub machine_y: i32,
}

impl SideLoad {
    /// `required / capacity`; `+inf` for a side the ladder credited nothing
    /// (a zero-count placeholder), so it sorts as the fullest.
    pub fn utilization(&self) -> f64 {
        if self.capacity <= 0.0 {
            f64::INFINITY
        } else {
            self.required / self.capacity
        }
    }

    /// The same side re-priced at a different DECLARED level — the sim
    /// registry runs one geometry (sized at its `geo_cap`) in several
    /// declared worlds, and the hand that starves is the hand at the
    /// world's rate, not the sizing level's. Input sides re-price through
    /// `machine_feed_rate` (the table the ladder used); output sides are
    /// returned unchanged (their belt-drop rate needs the stacking and the
    /// target belt, which the event does not carry).
    ///
    /// Two assumptions, stated: (1) `QualityTier::Normal` — the event
    /// carries no quality, and every composer sizes at Normal today
    /// (`cells::chain::required_copies_at` does too); a non-Normal sizing
    /// would re-price wrong here without any signal. (2) `count` is the
    /// count the SIZING level chose: a geometry sized at L2 and run at L0
    /// keeps its L2 hand count at L0's per-hand rate, which is exactly the
    /// registry's world-mismatch question — but it means this is not
    /// "what the ladder would have placed at L0", which could be more
    /// hands. Read it as the fixed geometry in another world, nothing else.
    pub fn repriced(&self, level: u8) -> SideLoad {
        if self.side_is_output || self.count == 0 {
            return self.clone();
        }
        // A long-handed hand on an INPUT side is the reach-2 pickup, whose
        // credit the ladder derates (RFC-075 `FAR_PICKUP_FACTOR`); re-price
        // through the same function the ladder sizes with, so a census
        // fullness is read against the number that was actually credited.
        let reach = if self.entity == crate::bus::inserter_ladder::LONG_HANDED {
            crate::bus::inserter_ladder::Reach::Far
        } else {
            crate::bus::inserter_ladder::Reach::Near
        };
        let per_hand =
            crate::bus::inserter_ladder::far_pickup_rate(&self.entity, reach, QualityTier::Normal, level);
        SideLoad { capacity: self.count as f64 * per_hand, ..self.clone() }
    }

    fn from_event(ev: &TraceEvent) -> Option<SideLoad> {
        let TraceEvent::InserterSideSized {
            recipe,
            side_is_output,
            item,
            required,
            entity,
            count,
            capacity,
            machine_x,
            machine_y,
        } = ev
        else {
            return None;
        };
        Some(SideLoad {
            recipe: recipe.clone(),
            side_is_output: *side_is_output,
            item: item.clone(),
            required: *required,
            entity: entity.clone(),
            count: *count,
            capacity: *capacity,
            machine_x: *machine_x,
            machine_y: *machine_y,
        })
    }

    /// Dedupe key: one side of one machine. `entity` and `required` are
    /// part of it because a template can put the same item on a near AND
    /// a far hand of one machine (the 2-wide pair rows split one item's
    /// rate across a near hand and a far hand; a voider row can feed two
    /// long-handed hands the same item) and the event carries no reach —
    /// the two hands differ in what they are asked to move or what they
    /// are. The cost is the ambiguity signal's reach: a losing candidate
    /// that re-sizes the same side to a different tier or a different
    /// rate reads as a distinct side, not as ambiguity, so `ambiguous`
    /// counts only same-key plans that differ in count or capacity.
    fn key(&self) -> (String, bool, String, String, i32, i32, i64) {
        (
            self.recipe.clone(),
            self.side_is_output,
            self.item.clone(),
            self.entity.clone(),
            self.machine_x,
            self.machine_y,
            (self.required * 1e6).round() as i64,
        )
    }

    fn same_plan(&self, other: &SideLoad) -> bool {
        self.count == other.count && (self.capacity - other.capacity).abs() < 1e-9
    }
}

/// Run `f` with the sizing census ON, under a trace collector and a
/// recording sink; return its result with the events the SINK saw — the
/// shipped passes only (see the module doc). The collector is required:
/// `build_bus_layout` replays pass 1 to the sink out of the collector.
/// Not nestable inside an outer trace: `TraceGuard` clears the collector
/// on drop, so an enclosing trace would end here (debug-asserted).
pub fn capture<R>(f: impl FnOnce() -> R) -> (R, Vec<TraceEvent>) {
    debug_assert!(!trace::is_active(), "capture() must not run inside an outer trace");
    let buf: Rc<RefCell<Vec<TraceEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let _collector = trace::start_trace();
    let sink_buf = Rc::clone(&buf);
    let _sink = trace::set_sink(Box::new(move |e| {
        if matches!(e, TraceEvent::InserterSideSized { .. }) {
            sink_buf.borrow_mut().push(e.clone());
        }
    }));
    let r = trace::with_sizing_census(f);
    drop(_sink);
    let events = std::mem::take(&mut *buf.borrow_mut());
    (r, events)
}

type SideKey = (String, bool, String, String, i32, i32, i64);

fn dedupe(loads: impl Iterator<Item = SideLoad>) -> (Vec<SideLoad>, usize) {
    let mut by_key: FxHashMap<SideKey, SideLoad> = FxHashMap::default();
    let mut ambiguous: FxHashSet<SideKey> = Default::default();
    for load in loads {
        let key = load.key();
        if let Some(prev) = by_key.get(&key) {
            if !prev.same_plan(&load) {
                ambiguous.insert(key.clone());
            }
        }
        by_key.insert(key, load);
    }
    let mut out: Vec<SideLoad> = by_key.into_values().collect();
    out.sort_by(|a, b| {
        (a.machine_y, a.machine_x, a.side_is_output, &a.recipe, &a.item).cmp(&(
            b.machine_y,
            b.machine_x,
            b.side_is_output,
            &b.recipe,
            &b.item,
        ))
    });
    (out, ambiguous.len())
}

/// The sized sides of a NATIVE layout: every `InserterSideSized` event
/// whose `(recipe, machine_x, machine_y)` names a machine in `layout`, one
/// per (machine, side). Returns the loads and the number of keys that saw
/// more than one distinct plan (candidate builds sharing an origin) — for
/// those the LAST plan is kept, and the count is the census's honesty
/// figure.
pub fn side_loads(events: &[TraceEvent], layout: &LayoutResult) -> (Vec<SideLoad>, usize) {
    let machines: FxHashSet<(&str, i32, i32)> = layout
        .entities
        .iter()
        .filter_map(|e| e.recipe.as_deref().map(|r| (r, e.x, e.y)))
        .collect();
    dedupe(
        events
            .iter()
            .filter_map(SideLoad::from_event)
            .filter(|l| machines.contains(&(l.recipe.as_str(), l.machine_x, l.machine_y))),
    )
}

/// The sized sides of a COMPOSED layout, taken in the frame they were
/// emitted in (no join — see the module doc). The composer generates each
/// spec's cell once and clones it, so the loads describe one copy per
/// spec, not the whole composition; utilization is per side and needs no
/// scaling.
pub fn side_loads_unjoined(events: &[TraceEvent]) -> (Vec<SideLoad>, usize) {
    dedupe(events.iter().filter_map(SideLoad::from_event))
}

/// Utilization bands for input sides. The edges are the census's
/// hypotheses, not calibrated thresholds: 0.85 is the margin RFC-072's
/// grid quantizer ships, 0.926 (K=18, one short per copy) and 1.0 (K=20,
/// two short) are the receipted failures. Bands are `u < edge + 1e-9`,
/// closed on the top at every edge alike: a hand at exactly 0.85 is in
/// the first band (`≤0.85`), exactly 0.90 in the second, and exactly
/// its credit (1.000 — the K=20 / PU-from-ore class) in `0.95–1.00`;
/// only a plan the ladder itself could not cover (`required > capacity`
/// beyond the ladder's own EPS) reads as the `>1.00` shortfall band.
/// The band names in `CSV_HEADER` (`lt85` …) are shorthand for those
/// closed intervals; the fullest side's exact utilization is printed
/// alongside, so a hand at the 0.85 margin is never lost in a band.
pub const BAND_EDGES: [f64; 4] = [0.85, 0.90, 0.95, 1.0];

/// Per-fixture census row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Summary {
    pub sides_in: usize,
    pub sides_out: usize,
    pub ambiguous: usize,
    /// `[≤0.85, 0.85–0.90, 0.90–0.95, 0.95–1.00, >1.00 (shortfall)]`
    /// over input sides (each band closed at its top edge — `BAND_EDGES`).
    pub bands: [usize; 5],
    /// The fullest input side.
    pub max_in: Option<SideLoad>,
}

pub fn summarize(loads: &[SideLoad], ambiguous: usize) -> Summary {
    let mut s = Summary { ambiguous, ..Default::default() };
    for l in loads {
        if l.side_is_output {
            s.sides_out += 1;
            continue;
        }
        s.sides_in += 1;
        let u = l.utilization();
        let band = BAND_EDGES.iter().position(|&e| u < e + 1e-9).unwrap_or(4);
        s.bands[band] += 1;
        if s.max_in.as_ref().is_none_or(|m| u > m.utilization()) {
            s.max_in = Some(l.clone());
        }
    }
    s
}

impl Summary {
    /// CSV columns: `sides_in,sides_out,ambiguous,lt85,b85_90,b90_95,b95_100,short,max_util,max_recipe,max_side`.
    /// Unquoted; `max_util` prints `inf` for a zero-capacity side (no
    /// template emits one today) and the last three columns are empty
    /// when no input side exists — consumers join on the fixture column.
    /// For a COMPOSED layout the side counts are per generated cell (one
    /// copy per spec), not per fixture: ec75 and ec150 read identically
    /// because both seed the same per-copy cell.
    pub fn csv(&self) -> String {
        let (u, recipe, side) = match &self.max_in {
            Some(m) => (
                format!("{:.3}", m.utilization()),
                m.recipe.clone(),
                format!("{} {}x{} {:.2}/{:.2}", m.item, m.count, m.entity, m.required, m.capacity),
            ),
            None => ("".into(), "".into(), "".into()),
        };
        format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            self.sides_in,
            self.sides_out,
            self.ambiguous,
            self.bands[0],
            self.bands[1],
            self.bands[2],
            self.bands[3],
            self.bands[4],
            u,
            recipe,
            side
        )
    }

    pub const CSV_HEADER: &'static str =
        "sides_in,sides_out,ambiguous,lt85,b85_90,b90_95,b95_100,short,max_util,max_recipe,max_side";
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.csv())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PlacedEntity;

    #[allow(clippy::too_many_arguments)]
    fn sized(recipe: &str, out: bool, item: &str, req: f64, entity: &str, count: usize, cap: f64, x: i32, y: i32) -> TraceEvent {
        TraceEvent::InserterSideSized {
            recipe: recipe.into(),
            side_is_output: out,
            item: item.into(),
            required: req,
            entity: entity.into(),
            count,
            capacity: cap,
            machine_x: x,
            machine_y: y,
        }
    }

    fn layout_with(machines: &[(&str, i32, i32)]) -> LayoutResult {
        let mut l = LayoutResult::default();
        for &(r, x, y) in machines {
            l.entities.push(PlacedEntity {
                name: "assembling-machine-3".into(),
                x,
                y,
                recipe: Some(r.into()),
                ..Default::default()
            });
        }
        l
    }

    #[test]
    fn joins_only_shipped_machines_keeps_near_and_far_apart_and_flags_ambiguity() {
        let layout = layout_with(&[("electronic-circuit", 0, 2), ("electronic-circuit", 3, 2)]);
        let events = vec![
            // Near (cable) and far (iron) inputs of ONE machine: two sides.
            sized("electronic-circuit", false, "iron-plate", 2.4, "long-handed-inserter", 1, 2.4, 0, 2),
            sized("electronic-circuit", false, "copper-cable", 7.5, "stack-inserter", 1, 12.0, 0, 2),
            // A losing candidate planned the SAME side (same item, entity,
            // origin, required) with a different hand count: ambiguous,
            // last one wins.
            sized("electronic-circuit", false, "iron-plate", 2.4, "long-handed-inserter", 2, 4.8, 0, 2),
            sized("electronic-circuit", false, "iron-plate", 2.4, "long-handed-inserter", 1, 2.4, 0, 2),
            // The same item on a second hand of the same machine at a
            // different rate (a pair row's near/far split) is its own side.
            sized("electronic-circuit", false, "iron-plate", 2.0, "long-handed-inserter", 1, 2.4, 0, 2),
            // Re-stamped identically (a layout retry): not ambiguous.
            sized("electronic-circuit", false, "iron-plate", 1.0, "long-handed-inserter", 1, 2.4, 3, 2),
            sized("electronic-circuit", false, "iron-plate", 1.0, "long-handed-inserter", 1, 2.4, 3, 2),
            sized("electronic-circuit", true, "electronic-circuit", 1.0, "fast-inserter", 1, 2.31, 3, 2),
            // Not in the layout at all.
            sized("copper-cable", false, "copper-plate", 5.0, "stack-inserter", 1, 12.0, 9, 9),
        ];
        let (loads, ambiguous) = side_loads(&events, &layout);
        assert_eq!(ambiguous, 1);
        assert_eq!(loads.len(), 5);
        let s = summarize(&loads, ambiguous);
        assert_eq!((s.sides_in, s.sides_out), (4, 1));
        assert_eq!(
            s.bands,
            [3, 0, 0, 1, 0],
            "7.5/12, 2.0/2.4 and 1.0/2.4 sit at or under 0.85; 2.4/2.4 is the 0.95–1.00 band"
        );
        let m = s.max_in.expect("an input side");
        assert_eq!((m.machine_x, m.item.as_str(), m.required, m.count), (0, "iron-plate", 2.4, 1));
        // The unjoined form keeps the off-layout side too.
        assert_eq!(side_loads_unjoined(&events).0.len(), 6);
    }

    #[test]
    fn exact_edges_close_their_band_from_the_top() {
        let layout = layout_with(&[("r", 0, 0)]);
        let at = |u: f64, x: i32| sized("r", false, "i", u * 2.0, "fast-inserter", 1, 2.0, x, 0);
        let layout = {
            let mut l = layout;
            for x in 1..5 {
                l.entities.push(PlacedEntity { name: "m".into(), x, y: 0, recipe: Some("r".into()), ..Default::default() });
            }
            l
        };
        let events = vec![at(0.85, 0), at(0.90, 1), at(0.95, 2), at(1.0, 3), at(1.0 + 1e-6, 4)];
        let (loads, amb) = side_loads(&events, &layout);
        assert_eq!(summarize(&loads, amb).bands, [1, 1, 1, 1, 1]);
    }

    #[test]
    fn shortfall_lands_in_the_top_band_and_reprice_uses_the_declared_level() {
        let layout = layout_with(&[("rail", 0, 0)]);
        let events = vec![sized("rail", false, "stone", 3.0, "long-handed-inserter", 2, 4.8, 0, 0)];
        let (loads, amb) = side_loads(&events, &layout);
        assert_eq!(summarize(&loads, amb).bands, [1, 0, 0, 0, 0]);
        // At level 0 the long-handed PICKUP hand credits 1.2 × 0.85 = 1.02/s
        // (RFC-075; 2 × 1.02 = 2.04 < 3.0): the same geometry is a shortfall
        // in an L0 world, re-priced through the ladder's own credit.
        let l0: Vec<SideLoad> = loads.iter().map(|l| l.repriced(0)).collect();
        let per_hand = crate::bus::inserter_ladder::far_pickup_rate(
            "long-handed-inserter",
            crate::bus::inserter_ladder::Reach::Far,
            QualityTier::Normal,
            0,
        );
        assert!((per_hand - 1.02).abs() < 1e-9);
        assert!((l0[0].capacity - 2.0 * per_hand).abs() < 1e-9);
        assert_eq!(summarize(&l0, amb).bands, [0, 0, 0, 0, 1]);
        // Output sides never re-price.
        let out = SideLoad { side_is_output: true, ..loads[0].clone() };
        assert_eq!(out.repriced(0), out);
    }

    #[test]
    fn capture_returns_the_sinks_view_of_sized_sides_only() {
        let ((), events) = capture(|| {
            trace::emit(sized("rail", false, "stone", 3.0, "long-handed-inserter", 2, 4.8, 0, 0));
            trace::emit(TraceEvent::InserterSideCapped {
                recipe: "rail".into(),
                side_is_output: false,
                required: 3.0,
                placed_entity: "long-handed-inserter".into(),
                placed_count: 2,
                shortfall: 0.6,
                machine_x: 0,
                machine_y: 0,
                limit: "geometry".into(),
            });
        });
        assert_eq!(events.len(), 1);
        assert!(!trace::is_active(), "capture restores the untraced state");
    }
}
