//! Structured trace event collection for the bus layout pipeline.
//!
//! Thread-local collector — zero overhead when no trace is active.
//! Use `start_trace()` to begin collection, `emit()` to record events,
//! and `drain_events()` to retrieve them.

use std::cell::{Cell, RefCell};

use crate::models::PlacedEntity;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Collector
// ---------------------------------------------------------------------------

thread_local! {
    static COLLECTOR: RefCell<Option<Vec<TraceEvent>>> = const { RefCell::new(None) };
    static SINK: RefCell<Option<Box<dyn FnMut(&TraceEvent)>>> = const { RefCell::new(None) };
    /// Suppress event recording within a `with_muted` scope. Used by
    /// junction-blame retries so the speculative re-solves don't pollute
    /// the real event stream with phantom JunctionGrowth* etc events.
    static MUTED: Cell<bool> = const { Cell::new(false) };
    /// Suppress ONLY `MergeTapFallback` emission within a
    /// `with_merge_tap_fallback_suppressed` scope. The layout runs
    /// `plan_bus_lanes` twice (a provisional pass-1 then the real pass-2),
    /// and the fallback event is pass-invariant (its item/shape/K/bins derive
    /// from solver counts+rates, not row geometry). Pass 1 always runs and
    /// records it; pass 2 is wrapped in the suppressor so it isn't recorded
    /// twice. Unlike `with_muted` this leaves pass 2's other (authoritative)
    /// events intact. See `bus::layout::layout_pass`.
    static SUPPRESS_MERGE_TAP_FALLBACK: Cell<bool> = const { Cell::new(false) };
}

/// Start trace collection for the current thread. Returns a guard that
/// cleans up on drop.
pub fn start_trace() -> TraceGuard {
    COLLECTOR.with(|c| *c.borrow_mut() = Some(Vec::new()));
    TraceGuard
}

/// RAII guard — clears the collector on drop.
pub struct TraceGuard;

impl Drop for TraceGuard {
    fn drop(&mut self) {
        COLLECTOR.with(|c| *c.borrow_mut() = None);
    }
}

/// Install a sink that sees every emitted event as it happens.
/// Coexists with the collector — both fire on each emit. Returns a guard
/// that removes the sink on drop.
pub fn set_sink(sink: Box<dyn FnMut(&TraceEvent)>) -> SinkGuard {
    SINK.with(|s| *s.borrow_mut() = Some(sink));
    SinkGuard
}

/// Atomically replace the active sink with `new_sink` (or `None` to
/// disable streaming) and return whatever sink was previously active.
/// Used by `build_bus_layout` to install a buffering sink for pass 1
/// so the streaming consumer never sees events from a layout pass that
/// was abandoned by retry. Caller is responsible for restoring the
/// returned sink (or letting it drop) at the right moment.
pub fn swap_sink(
    new_sink: Option<Box<dyn FnMut(&TraceEvent)>>,
) -> Option<Box<dyn FnMut(&TraceEvent)>> {
    SINK.with(|s| std::mem::replace(&mut *s.borrow_mut(), new_sink))
}

/// RAII guard — clears the sink on drop.
pub struct SinkGuard;

impl Drop for SinkGuard {
    fn drop(&mut self) {
        SINK.with(|s| *s.borrow_mut() = None);
    }
}

/// Emit a trace event. No-op if neither a collector nor a sink is active,
/// or if `with_muted` is in effect on this thread.
pub fn emit(event: TraceEvent) {
    if MUTED.with(|m| m.get()) {
        return;
    }
    SINK.with(|s| {
        if let Some(ref mut sink) = *s.borrow_mut() {
            sink(&event);
        }
    });
    COLLECTOR.with(|c| {
        if let Some(ref mut events) = *c.borrow_mut() {
            events.push(event);
        }
    });
}

/// Run `f` with event emission suppressed on this thread. Used by
/// junction-blame retries so speculative re-solves don't pollute the
/// real event stream.
///
/// RAII via Drop guard — a panic inside `f` still restores the
/// previous mute state, which matters for callers that wrap unstable
/// code (e.g. `bus::decomposition_search::select_best_decomposition`
/// catching panics from candidate `produce` calls).
pub fn with_muted<F: FnOnce() -> R, R>(f: F) -> R {
    struct MuteGuard(bool);
    impl Drop for MuteGuard {
        fn drop(&mut self) {
            MUTED.with(|m| m.set(self.0));
        }
    }
    let prev = MUTED.with(|m| m.replace(true));
    let _guard = MuteGuard(prev);
    f()
}

/// True when `MergeTapFallback` emission is suppressed on this thread. Emit
/// sites for that event gate on `!merge_tap_fallback_suppressed()`.
pub fn merge_tap_fallback_suppressed() -> bool {
    SUPPRESS_MERGE_TAP_FALLBACK.with(|c| c.get())
}

/// Run `f` with `MergeTapFallback` emission suppressed on this thread — used
/// to dedup the two-pass `plan_bus_lanes` double-emit (pass 1 records the
/// pass-invariant event; pass 2 is wrapped here). RAII-restored and
/// panic-safe, mirroring `with_muted`.
pub fn with_merge_tap_fallback_suppressed<F: FnOnce() -> R, R>(f: F) -> R {
    struct Guard(bool);
    impl Drop for Guard {
        fn drop(&mut self) {
            SUPPRESS_MERGE_TAP_FALLBACK.with(|c| c.set(self.0));
        }
    }
    let prev = SUPPRESS_MERGE_TAP_FALLBACK.with(|c| c.replace(true));
    let _guard = Guard(prev);
    f()
}

/// Drain collected events from the current thread.
pub fn drain_events() -> Vec<TraceEvent> {
    COLLECTOR.with(|c| c.borrow_mut().take().unwrap_or_default())
}

/// Check if a trace is currently active.
#[allow(dead_code)]
pub fn is_active() -> bool {
    COLLECTOR.with(|c| c.borrow().is_some())
}

thread_local! {
    /// The sizing census is ON: `InserterSideSized` is built and emitted
    /// (RFC-073 Phase 0). Opt-in per scope via `with_sizing_census`
    /// (`bus::sizing_census::capture`), never by a collector or sink
    /// being present — the web's streaming solve installs both on every
    /// interactive layout, so a "someone is listening" gate would build
    /// one event per machine side (~18k allocations on the ec@240 grid)
    /// and serialize them to the browser for nothing (#735 rounds 1–2).
    static SIZING_CENSUS: Cell<bool> = const { Cell::new(false) };
}

/// Is the sizing census on for this thread? The per-side sizing event
/// is built only when this is true.
pub fn sizing_census_enabled() -> bool {
    SIZING_CENSUS.with(|c| c.get())
}

/// Run `f` with the sizing census on (RAII — restored on panic too).
pub fn with_sizing_census<F: FnOnce() -> R, R>(f: F) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            SIZING_CENSUS.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(SIZING_CENSUS.with(|c| c.replace(true)));
    f()
}

/// Number of events currently in the collector (0 if none active).
/// Lets the layout-retry loop snapshot the collector size before its
/// inner pass, then read only events emitted by that pass.
pub fn peek_events_len() -> usize {
    COLLECTOR.with(|c| c.borrow().as_ref().map(|v| v.len()).unwrap_or(0))
}

/// Clone of every event emitted at or after `start` (0 if no collector
/// active). Non-destructive; the collector keeps its contents.
pub fn peek_events_since(start: usize) -> Vec<TraceEvent> {
    COLLECTOR.with(|c| {
        c.borrow()
            .as_ref()
            .map(|v| v.iter().skip(start).cloned().collect())
            .unwrap_or_default()
    })
}

/// Drop every event from index `len` onward (no-op if no collector
/// active or `len` already past the end). Used by the layout-retry loop
/// to discard the failed first pass before emitting the retried pass.
/// The streaming sink still saw the discarded events live; only the
/// `result.trace` snapshot is affected.
pub fn truncate_events(len: usize) {
    COLLECTOR.with(|c| {
        if let Some(ref mut events) = *c.borrow_mut() {
            events.truncate(len);
        }
    });
}

/// Remove `InserterSideCapped` events at collector index >= `start`,
/// keeping everything else. Used by `layout_pass`'s two-pass rows/lanes
/// placement: when the width-corrected pass 2 runs, pass 1's capped-side
/// events describe machines at coordinates that no longer exist and would
/// mis-anchor the per-tile attribution join (RFC validation-explainability
/// D2). Other pass-1 events stay — the phase timeline deliberately shows
/// both passes (see `place_rows_1`/`place_rows_2` PhaseTime events).
pub fn remove_capped_events_since(start: usize) {
    COLLECTOR.with(|c| {
        if let Some(ref mut events) = *c.borrow_mut() {
            if start >= events.len() {
                return;
            }
            let tail: Vec<TraceEvent> = events
                .drain(start..)
                .filter(|e| !matches!(e, TraceEvent::InserterSideCapped { .. }))
                .collect();
            events.extend(tail);
        }
    });
}

// ---------------------------------------------------------------------------
// Trace event types
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "phase", content = "data")]
pub enum TraceEvent {
    // Phase 1: Row Placement
    RowsPlaced {
        rows: Vec<RowInfo>,
    },
    /// A DI spec was eligible in a coupling that could not claim it, because
    /// the spec was already fused into another cell (RFC-059 phase 1, output
    /// 2/3). This is the CONTENTION signal: the dispatcher resolves such a
    /// clash by iteration order, and a binary P0-vs-P1 layout diff cannot
    /// distinguish "nothing was contended" from "contended, and both orders
    /// happened to agree" — which is why kill criterion 1 gates on this rather
    /// than on the diff alone.
    DiCouplingContended {
        /// The spec that was already claimed, blocking this coupling.
        contended_spec: String,
        /// The coupling that lost: `producer -> consumer on item`.
        loser_producer: String,
        loser_consumer: String,
        loser_item: String,
        /// Whether the blocked index was the producer or the consumer.
        blocked_side: String,
    },
    /// A DI coupling was rejected BEFORE the contention check, so iteration
    /// order never got to decide it (RFC-059 phase 1). Recorded because a
    /// contention census is uninterpretable without it: a target reporting
    /// zero contention because DI never engaged at all is a different fact
    /// from one reporting zero because nothing was ever double-claimed, and
    /// kill criterion 1 turns on exactly that distinction.
    DiCouplingRefused {
        producer: String,
        consumer: String,
        item: String,
        /// `split-rows`, `producer-missing`, `producer-not-upstream`, or
        /// `not-buildable`.
        reason: String,
    },
    /// Which claim order the DI candidate kept, and what both arms scored
    /// (RFC-059). Emitted once per DI candidate that built under both orders.
    ///
    /// Exists because the choice is otherwise invisible: on the corpus the two
    /// arms agree on all but a handful of targets, so "DI won" says nothing
    /// about which order produced it, and a regression that quietly stopped
    /// running the second arm would look identical to one where both arms tied.
    DiClaimOrderChosen {
        /// `upstream` or `downstream`.
        order: String,
        upstream_entities: usize,
        downstream_entities: usize,
        upstream_warnings: usize,
        downstream_warnings: usize,
    },
    /// A DI coupling successfully claimed both its specs — the per-coupling
    /// outcome kill criterion 2 tests an estimator's ranking against.
    DiCouplingClaimed {
        producer: String,
        consumer: String,
        item: String,
        /// `row` or `stacked`.
        variant: String,
    },
    RowSplit {
        recipe: String,
        original_count: usize,
        split_into: usize,
        reason: String,
    },
    /// Records which row-layout variant the placer picked for a given
    /// recipe row. Fires once per row when the placer decides between
    /// `VerticalSplit` (today's default) and `HorizontalStack`. See
    /// `docs/rfc-horizontal-trunks.md` §Verification.
    RowLayoutSelected {
        recipe: String,
        kind: String,
        /// Number of stacked input₀ trunks for `HorizontalStack`; `1` for
        /// `VerticalSplit` (one input belt per input).
        k_trunks: usize,
        /// Machines per sub-row block. `0` for `VerticalSplit`.
        block_size: usize,
    },

    /// `bus::inserter_ladder::size_side` couldn't cover a machine side's
    /// planned rate even with every free column used at the richest tier
    /// `max_inserter_tier` allows. The layout still gets built (best-
    /// effort placement, no failure) and `check_inserter_throughput`
    /// keeps its honest warning — this event just names the cap that
    /// caused it. See `docs/rfc-inserter-sizing.md` Design.
    InserterSideCapped {
        recipe: String,
        side_is_output: bool,
        required: f64,
        placed_entity: String,
        placed_count: usize,
        shortfall: f64,
        /// Machine origin, so validation warnings anchored at the machine
        /// can join this event per-tile (RFC validation-explainability D2).
        machine_x: i32,
        machine_y: i32,
        /// Why the side capped: `"tier-cap"` (a richer tier at the same
        /// budget would cover — `max_inserter_tier` is binding),
        /// `"column-contest"` (the side lost the shared near/far column
        /// and that one column would have covered), or `"geometry"`
        /// (the row shape offers no further slots). Derived centrally in
        /// `inserter_ladder::capped_limit`, never guessed post-hoc.
        limit: String,
    },

    /// One machine side was sized by `bus::inserter_ladder` (RFC-073
    /// Phase 0, the sizing census). Emitted for every side the ROW
    /// TEMPLATES size (`bus::templates`, through `emit_side_trace` and
    /// the quad row's mirrored input3), covered or not —
    /// `InserterSideCapped` above is the shortfall subset. NOT emitted
    /// by the nine direct `size_side` calls in `bus::placer` (the DI
    /// bridge and the fused/straddle cells) — the census's recorded gap.
    /// The census reads `required / capacity` per side to find the hands
    /// the ladder fills to the brim; `capacity` is the plan's own credit
    /// at the level the layout was sized at (`SidePlan::capacity`), and
    /// `(entity, count)` lets a consumer re-price the side at a different
    /// declared level. Same machine-origin anchor as the capped event.
    /// Built only inside `trace::with_sizing_census` (the census's own
    /// `capture`) — never on an ordinary traced or streaming build, so
    /// it does not appear in snapshots or the browser's trace.
    InserterSideSized {
        recipe: String,
        side_is_output: bool,
        /// The item this side moves — a machine's near and far inputs are
        /// distinct sides at the same origin.
        item: String,
        required: f64,
        entity: String,
        count: usize,
        capacity: f64,
        machine_x: i32,
        machine_y: i32,
    },

    // Phase 2: Lane Planning
    LanesPlanned {
        lanes: Vec<LaneInfo>,
        families: Vec<FamilyInfo>,
        bus_width: i32,
    },

    // `LayoutStrategy::PartitionedDecomposed` partitioned an item into
    // `modules` distinct lane families (one per consuming recipe-row).
    // Fires zero or one time per partitioned item; absent for items with
    // K=1 consumer rows. See `docs/rfc-modular-production.md`.
    ModulePartitioned {
        item: String,
        /// Number of `(item, module_id)` lane families allocated. Equal
        /// to the consumer-row count for this item.
        modules: u32,
        /// Per-module lane count, parallel to module_id 0..modules.
        lanes_per_module: Vec<usize>,
    },

    // The partitioner's 75%-utilization gate rejected a proposed
    // partition. Layout is produced but invalid; surfaced as a loud
    // warning so the user sees the strategy didn't fit, rather than a
    // silent fall-back to Pooled.
    PartitionRejectedByUtilization {
        item: String,
        module_id: u32,
        /// Maximum per-lane utilization in [0.0, 1.0]. Above 0.75 trips
        /// the gate.
        lane_util: f64,
        belt_tier: String,
    },

    // `LayoutStrategy::PartitionedDecomposed` sharded an oversized
    // module into N sub-modules of ≤8 lanes each. Fires once per
    // sharded module. K2-1 / K2-2 instrumentation per
    // `docs/rfc-modular-production.md`.
    ShardSplit {
        item: String,
        /// Recipe consuming from this module. For K=1 items not in
        /// Phase 1's plan, the single consumer recipe.
        consumer_recipe: String,
        /// Pre-shard lane count (the value that exceeded 8 and
        /// triggered the split).
        original_lane_count: u32,
        /// Number of shards the module was split into = ⌈original / 8⌉.
        shards: u32,
        /// Per-shard lane count, parallel to shard module_id 0..shards.
        lanes_per_shard: Vec<usize>,
    },

    // Phase 2 cost-benefit gate: would-be shard count exceeded
    // `MAX_SHARDS_PER_MODULE`, so the partitioner kept the module
    // intact. The downstream balancer may not have a template wide
    // enough, but the alternative (multiplying consumer rows by
    // ⌈lane_count / 8⌉) was judged worse. Helps explain why Phase 2
    // sometimes leaves a wide trunk that Phase 1 also produced.
    ShardSkipped {
        item: String,
        consumer_recipe: String,
        lane_count: u32,
        /// What the shard count would have been without the gate.
        would_be_shards: u32,
        max_shards: u32,
    },

    // Shape-aware fix applied to a module whose `(n, m)` shape was not
    // stampable by the balancer library + gcd-decomposition fallback.
    // `strategy` is the name of the strategy that produced the fix
    // ("pad-lanes", "shard"); `kind` describes what was done.
    //
    // Fires from `apply_shape_fixes` in `bus/partitioner.rs` after the
    // existing Phase 2 oversize / K=1 sharding passes. When `kind` is
    // `"none"` and the shape was unstampable, no strategy could fix it
    // — the layout will dead-end and the
    // `missing-balancer-template` validator warning will surface.
    ShapeFixApplied {
        item: String,
        consumer_recipe: String,
        n: u32,
        original_m: u32,
        strategy: String,
        kind: String,
        /// For PadLanes: the new lane count (>= original_m).
        /// For Shard: total lane count across shards.
        new_total_lanes: u32,
    },

    // An item with an unstampable (n, m) shape was enrolled in
    // `plan.modules` by `build_k1_enrollment_plan`
    // (`bus/decomposition_search.rs`). Since RFC-069 Phase A2 the event
    // fires from BOTH arms: the K=1 arm (one module, `module_id=0`,
    // lane_count = the warning-shape pad) and the multi-consumer arm
    // (one event per enrolled per-consumer module, lane counts as
    // `apply_shape_fixes` left them). `n_producers` is the pooled
    // producer count THE EMITTING ARM'S shape decision used — the K=1
    // arm reports the warning's family `n`; the multi arm reports
    // `producer_count_estimate` (the raw machine count its shape-fix
    // pass consulted). Never a per-consumer split (#721 rounds 2-3).
    // Without this enrollment, coprime-trap shapes
    // (e.g. (4, 9) for copper-plate on ec35/PU) silently dead-end at
    // balancer stamp time.
    K1ItemEnrolled {
        item: String,
        consumer_recipe: String,
        n_producers: u32,
        lane_count: u32,
    },

    // Decomposition produced shards whose lane count doesn't tile
    // cleanly with consumer demand (multi-consumer K2-2 case from the
    // RFC). Fires when a consumer's tap from a shard is uneven —
    // e.g. a 7-lane consumer tapping from a (6, 6) shard split.
    // For single-consumer modules this never fires by construction
    // (uniform demand divides cleanly).
    LumpyShardTap {
        item: String,
        consumer_recipe: String,
        /// Lanes the consumer needs from this specific shard.
        consumer_lanes_in_shard: u32,
        /// The shard's total lane width.
        shard_lane_count: u32,
    },
    LaneSplit {
        item: String,
        rate: f64,
        max_lane_cap: f64,
        n_splits: usize,
    },
    LaneOrderOptimized {
        ordering: Vec<String>,
        crossing_score: usize,
    },

    // Phase 3: Bus Routing
    CrossingZoneSolved {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        solve_time_us: u64,
    },
    CrossingZoneSkipped {
        tap_item: String,
        tap_x: i32,
        tap_y: i32,
        reason: String,
    },
    /// #652 fail-sever: an unresolved crossing's feeder belt pointed
    /// into a belt carrying a DIFFERENT item (a flat sideload-merge —
    /// the shape that dumps one flow onto a lane of another and fans
    /// out as mass downstream lane-throughput errors). The feeder was
    /// dropped so the flow dead-ends visibly one tile short instead.
    /// One event per severed feeder entity.
    CrossingSevered {
        x: i32,
        y: i32,
        item: String,
        into_item: String,
    },
    /// #652 retry-with-teeth: a cluster's solution collided with
    /// differing committed entities (context-conflict at commit), and
    /// the cluster was re-solved once with the conflicting tiles as a
    /// forbidden-override (forbidden regardless of entity kind — the
    /// surface-belt exemption does not apply). `resolved == true`
    /// means the retry produced a conflict-free solution that was
    /// committed; `false` means the retry failed (no solution, or a
    /// solution that still conflicted) and the cluster fell through to
    /// the cap/fail-sever machinery. One event per retried cluster,
    /// tagged with the cluster's seed tile.
    CrossingConflictRetried {
        x: i32,
        y: i32,
        conflict_tiles: usize,
        resolved: bool,
    },
    /// A balancer block was requested for a lane family. `template_found
    /// == false` is not necessarily a bug by itself — it means no direct
    /// template, gcd-decomposition, nor the runtime generator could
    /// realize this `(N, M)` shape, so `stamp_family_balancer` legitimately
    /// returned no entities. But the feeder-spec generator in
    /// `route_bus_ghost` (`crates/core/src/bus/ghost_router.rs`) mirrors
    /// the same passthrough/template/decomposition search (not the
    /// runtime-generator fallback — see `FeederSpecsSkipped`) to find each
    /// producer row's balancer input tile; when this event's
    /// `template_found` is `false` for a family, that search comes up
    /// empty too, and the family's producer rows dead-end with no feeder
    /// belts routed at all. See `FeederSpecsSkipped`, which fires at that
    /// downstream site.
    BalancerStamped {
        item: String,
        shape: (usize, usize),
        y_start: i32,
        y_end: i32,
        template_found: bool,
    },
    /// Feeder-spec generation for a lane family (`route_bus_ghost`,
    /// `crates/core/src/bus/ghost_router.rs`) computed an empty
    /// `input_xs` — no direct template, gcd-decomposition, nor
    /// passthrough-shape rule could place any balancer input tile for
    /// this family's `(N, M)` shape. In practice this coincides with
    /// `BalancerStamped { template_found: false }` for the same family
    /// (its direct consequence): without input tiles, the guard at the
    /// skip site silently skips the entire per-producer feeder loop,
    /// every one of the family's `producer_rows` gets no feeder belt, and
    /// those producer rows' output belts dead-end with no other trace
    /// signal. Fires at most once per family (checked on the leftmost
    /// lane), and only when the family actually has producer rows to
    /// route — a family with zero producer rows has nothing to dead-end
    /// and does not fire this.
    ///
    /// Caveat for future maintainers: this input_xs search does NOT
    /// mirror `stamp_family_balancer`'s third fallback, the Phase 2.0
    /// runtime template generator (`balancer_generate::generate`). Today
    /// that's a distinction without a difference — every shape whose
    /// generator output passes the `width <= m` guard is (verified by
    /// `crate::bus::balancer::shape_is_stampable`'s own property test)
    /// already reachable via gcd-decomposition through the same 1↔2/2↔1
    /// atom templates, so `template_found` and `input_xs` non-emptiness
    /// never actually diverge. If the generator ever grows coverage the
    /// decomposition search can't reach (e.g. non-2 fan ratios), this
    /// event would start firing on families where `template_found` was
    /// `true` — a real bug (balancer stamped, but never fed) distinct
    /// from the legitimate-miss case this doc otherwise describes.
    FeederSpecsSkipped {
        item: String,
        module_id: u32,
        producer_rows: usize,
        shape: (usize, usize),
    },
    /// A merge-tap feeder could not be routed UNDER a foreign trunk block:
    /// the block is wider than the underground reach, so no single UG hop
    /// spans it AND no surface tile inside it can host a belt (its neighbours
    /// are foreign trunk on both sides). Rather than emit a severed
    /// half-bridge that dumps the feeder's item onto the foreign lane (the
    /// silent-drop `bridge_feeder_under_foreign_trunks` used to produce when
    /// two UG hops met on one trapped tile), the feeder is skipped entirely —
    /// its producer output dead-ends, an honest and visible failure. The
    /// categorical fix is lane ordering: place merge-tap trunks on the
    /// producer side of wide external-input blocks so feeders never cross
    /// them. One `FeederBridgeUnbridgeable` fires per skipped feeder.
    FeederBridgeUnbridgeable {
        item: String,
        module_id: u32,
        /// East-west span of the foreign block the feeder must cross.
        span: i32,
        /// Underground reach for the feeder's belt tier (max bridgeable gap).
        reach: i32,
    },
    /// A merge-tap TAP spec could not be routed cleanly under the foreign
    /// trunk columns it crosses (same foreign-trunk bridge as feeders, but a
    /// tap fans EAST from its trunk to a consumer). When the crossing is
    /// unbridgeable — a foreign block wider than the UG reach with no legal
    /// surface inside it — the tap is skipped entirely rather than surfaced
    /// onto the foreign lane. Skipping a tap dead-ends the consumer it fed
    /// (honest, validator-visible starvation) instead of contaminating the
    /// crossed trunk. Like the feeder case, the categorical fix is lane
    /// ordering. One `TapBridgeUnbridgeable` fires per skipped tap.
    TapBridgeUnbridgeable {
        item: String,
        module_id: u32,
        /// East-west span of the foreign block the tap must cross.
        span: i32,
        /// Underground reach for the tap's belt tier (max bridgeable gap).
        reach: i32,
    },
    /// Phase 2.0 runtime template generator produced a layout for a shape
    /// that wasn't directly served by the library (and decomposition
    /// either missed it or the generator's output was preferred). Useful
    /// for measuring the generator's reach in real layouts.
    BalancerGenerated {
        item: String,
        shape: (usize, usize),
        entity_count: usize,
        width: u32,
        height: u32,
    },
    /// Stream sibling of `BalancerStamped` — carries the actual entity batch
    /// so the live renderer can reveal a balancer cascade progressively
    /// instead of dumping it via the `bus_routed` safety net at the end.
    BalancerCommitted {
        item: String,
        shape: (usize, usize),
        entities: Vec<PlacedEntity>,
    },
    /// One emission per per-lane stamp pass during Steps 2 (tap-off
    /// splitters and continue-belts), 3.5 (solid trunk segments), and 3.6
    /// (fluid trunks). `is_fluid` distinguishes the source step but the
    /// renderer treats them uniformly. Each lane therefore emits two events
    /// for solid lanes (Step 2 and Step 3.5) and one event for fluid lanes
    /// (Step 3.6).
    TrunkBeltCommitted {
        item: String,
        lane_x: i32,
        is_fluid: bool,
        entities: Vec<PlacedEntity>,
    },
    LaneRouted {
        item: String,
        x: i32,
        is_fluid: bool,
        trunk_segments: usize,
        tapoffs: usize,
    },
    TapoffRouted {
        item: String,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        path_len: usize,
    },

    // Phase 4: Output Merging
    OutputMerged {
        item: String,
        rows: Vec<usize>,
        merge_y: i32,
    },
    /// Stream sibling of `OutputMerged` — carries the merger entity batch
    /// (belts + splitters with `merger:{item}` segment id) so the live
    /// renderer can reveal them progressively.
    OutputMergerCommitted {
        item: String,
        entities: Vec<PlacedEntity>,
    },
    MergerBlockPlaced {
        item: String,
        lanes: usize,
        block_y: i32,
        block_height: i32,
    },

    // Phase 5: Power Poles
    PolesPlaced {
        count: usize,
        strategy: String,
    },
    /// Stream sibling of `PolesPlaced` — carries the pole entity batch so
    /// the live renderer can reveal them progressively.
    PolesCommitted {
        entities: Vec<PlacedEntity>,
    },

    // Phase boundary markers
    PhaseComplete {
        phase: String,
        entity_count: usize,
    },
    /// Full entity snapshot at a phase boundary (only emitted when tracing is active).
    PhaseSnapshot {
        phase: String,
        entities: Vec<PlacedEntity>,
        width: i32,
        height: i32,
    },

    // Phase timing (wall-clock milliseconds per major phase)
    PhaseTime {
        phase: String,
        duration_ms: u64,
    },

    // Negotiate (A*) summary
    NegotiateComplete {
        specs: usize,
        iterations: u32,
        duration_ms: u64,
    },

    // Solver output — emitted at the start of build_bus_layout
    SolverCompleted {
        recipe_count: usize,
        machine_count: usize,
        external_input_count: usize,
        external_output_count: usize,
        machines: Vec<MachineTrace>,
    },

    /// #461 part (a) production-path fix (`solver.rs`): the initial solve
    /// placed a burner machine (the engine delivers no fuel to any burner)
    /// for one or more recipes whose product item has at least one OTHER
    /// producer that CAN run on an electric machine, so those recipes were
    /// added to the exclusion set and the target was re-solved once. This
    /// event is always emitted when a re-solve is ATTEMPTED.
    ///
    /// `burners_before` is the number of `!needs_electricity` machine
    /// ENTRIES (not `Σ count` — a count of distinct burner-recipe rows in
    /// `SolverResult::machines`, matching how this whole mechanism reasons
    /// about "how many burner recipes are in play") in the ORIGINAL
    /// result. `burners_after` is the same count for the re-solve's
    /// result, or `None` if the re-solve errored — distinguishing the
    /// three possible outcomes `accepted` alone collapses: errored
    /// (`burners_after: None`), re-solved but not fewer burners
    /// (`Some(n) if n >= burners_before`), or genuinely fewer
    /// (`Some(n) if n < burners_before`). `accepted` is `true` only in the
    /// last case — the re-solve is accepted whenever it *reduces* the
    /// burner count, not only when it eliminates every burner: a
    /// multi-target solve can mix a STEERABLE burner (e.g.
    /// `rocket-fuel-from-jelly`, which has an electric alternative) with
    /// an UNSTEERABLE one in the same result (e.g. `pentapod-egg`,
    /// biochamber-only) — a re-solve that removes the steerable one while
    /// leaving the unsteerable one in place still has fewer burners than
    /// the original and must be kept, even though it isn't burner-free.
    /// When `accepted` is `false`, the ORIGINAL result is what actually
    /// got used (see `phase0e1_biolubricant_biochamber`, whose re-solve
    /// wanders into an 11-machine-type plan carrying MORE burners — three
    /// OTHER biochamber recipes — than the one it was trying to avoid).
    /// Absence of this event for a solve means no burner machine had an
    /// electric alternative to steer toward at all — no re-solve was even
    /// attempted.
    BurnerRecipeExcluded {
        target_item: String,
        excluded_recipes: Vec<String>,
        accepted: bool,
        burners_before: usize,
        burners_after: Option<usize>,
    },

    /// The layout pipeline ran once, hit `JunctionGrowthCapped` events,
    /// and is being re-run with extra vertical gap inserted after each
    /// row whose successor junction couldn't fit. Emitted at the start
    /// of the retried pass, so the trace stream that reaches the UI
    /// records that a retry happened and which rows got widened.
    LayoutRetried {
        /// `(row_index, extra_tiles)` pairs — the same map that's plumbed
        /// into `place_rows::extra_gap_after_row` for the retry.
        gaps: Vec<(usize, i32)>,
        /// Number of `JunctionGrowthCapped` events seen on the original pass.
        caps_before: usize,
        /// Recipe name for each row that got widened (parallel to `gaps`).
        /// Lets the UI label the panel without cross-referencing other events.
        recipes: Vec<String>,
    },

    /// The reactive power-repair pass (RFC `docs/rfc-power-reservation.md`
    /// Phase 3a-ii / 3b) re-ran the full pipeline with widened substation bands,
    /// but `place_poles` STILL reported uncovered electric inserters afterward —
    /// the widen-plus-substation repair did NOT converge and this layout ships
    /// power-broken. Every corpus fixture converges (the four gating pins +
    /// kovarex + USP all reach zero uncovered), so this event firing means a
    /// genuinely-new starved geometry with no pinning fixture. It is the loud,
    /// release-surviving alarm the Phase 3a-ii review asked for: a
    /// `debug_assert` would be skipped in release builds and ship the break
    /// silently, so the non-convergence is surfaced as a trace event (lands in
    /// snapshots / can drive a scoreboard) instead. No corpus case emits it.
    ReactivePassNotConverged {
        /// Electric inserters STILL uncovered after the repair pass.
        uncovered_count: usize,
        /// A capped, sorted sample of the still-uncovered inserter tiles for triage.
        sample: Vec<(i32, i32)>,
    },

    // Validation results — emitted by validate() after all checks run
    ValidationCompleted {
        error_count: usize,
        warning_count: usize,
        issues: Vec<ValidationIssueTrace>,
    },

    // Surplus byproduct lane physically extended to the layout perimeter
    // (Phase 2 of rfc-solver-net-flow). Consumed by the stranded-byproduct
    // validator, which cross-checks a pipe entity actually exists at (x, y).
    SurplusRouted {
        item: String,
        /// Trunk column x.
        x: i32,
        /// South-boundary exit y the trunk was extended to.
        y: i32,
    },

    // RFC Fulgora Phase 2 (docs/rfc-fulgora-scrap.md D1): a solid surplus
    // stream resolved to a self-voider recipe (`<item>-recycling`: X ->
    // fraction*X) and a recycler bank was synthesized to consume it.
    VoiderSynthesized {
        item: String,
        /// Surplus rate (items/s) the bank was sized to destroy.
        rate: f64,
        /// Recycler machine count placed for this bank.
        machines: usize,
    },

    // A solid surplus stream under `SurplusPolicy::Void` could not be
    // resolved to a synthesizable voider shape (no `<item>-recycling`
    // recipe, or a multi-output / non-self cascade) and fell back to
    // ordinary `SurplusPolicy::Export` routing instead of being silently
    // dropped.
    VoiderFallbackExport {
        item: String,
        reason: String,
    },

    // External input lane consolidation — N consumer rows served by M trunk lanes
    LaneConsolidated {
        item: String,
        /// Total rate this item is consumed at
        rate: f64,
        /// Number of recipe rows that consume this item
        consumer_count: usize,
        /// Number of trunk lanes used (< consumer_count means sharing)
        n_trunk_lanes: usize,
        rate_per_lane: f64,
    },

    /// A lane family's `(N, M)` shape had no balancer template
    /// (`shape_is_stampable` returned false), so the merge-and-tap fallback
    /// replaced it with `K = ceil(rate / full_belt_cap)` shared trunks: each
    /// trunk's producer group merges via a splitter merge-tree
    /// (`balancer_generate::merge_tree`) and its consumer group taps the trunk
    /// with priority splitters (RFC `docs/rfc-merge-tap-trunks.md`). Emitted
    /// once per family that takes the fallback so the activation is
    /// one-grep diagnosable. `producers_per_trunk[i]` / `consumers_per_trunk[i]`
    /// are the bin-packing assignment counts for trunk `i`.
    MergeTapFallback {
        item: String,
        module_id: u32,
        /// The unstampable `(N producers, M consumers)` shape that triggered
        /// the fallback.
        shape: (usize, usize),
        /// `K` — the throughput-sized trunk count.
        k_trunks: usize,
        /// Producer-row count assigned to each of the `K` trunks.
        producers_per_trunk: Vec<usize>,
        /// Consumer-row count assigned to each of the `K` trunks.
        consumers_per_trunk: Vec<usize>,
    },

    // SAT crossing zone removed because it conflicted with a splitter stamp tile
    CrossingZoneConflict {
        /// The crossing segment ID that was removed
        segment_id: String,
        /// Tile position of the conflict
        conflict_x: i32,
        conflict_y: i32,
    },

    // Fluid trunk gap-fill failed: the UG-in/UG-out pair needed to bridge
    // a gap between two anchors couldn't be placed because the candidate
    // tiles were blocked. The trunk will have a physical break here; the
    // `fluid-network` validator will surface it as a hard error. Emitted
    // by `route_bus_ghost` step 3.6 fluid-trunk emission.
    FluidTrunkBreak {
        item: String,
        trunk_x: i32,
        y_start: i32,
        y_end: i32,
        reason: String,
    },

    // `build_bus_layout` is retrying place_rows → plan_bus_lanes → route_bus
    // after seeing dropped bridges from the previous attempt. `attempt` is
    // the retry number (1 = first retry, so second overall attempt).
    BridgeRetry {
        attempt: u32,
        dropped_count: usize,
        extra_gap_updates: usize,
    },

    // All retries exhausted (hit MAX_BRIDGE_RETRIES) but bridges are still
    // being dropped. Layout will render with the current (possibly broken)
    // state and the validator will flag remaining issues.
    BridgeRetryExhausted {
        final_dropped_count: usize,
        max_retries: u32,
    },

    // Per-band measurement emitted after a successful route_bus. One event
    // per adjacent row pair. Used by the compaction baseline/scoreboard to
    // measure total inter-row gap tiles before any shrinking is applied.
    InterRowBand {
        upper_row_idx: usize,
        lower_row_idx: usize,
        band_y_start: i32,
        band_y_end: i32,
        gap_height: i32,
        trunk_count: usize,
        distinct_items: usize,
    },

    /// Per-pole placement slack (RFC `docs/rfc-power-supply.md` Phase 2),
    /// emitted once for every pole after `place_poles` finishes. `alternatives`
    /// is the number of FREE tiles in the pole's own row within ±POLE_RANGE
    /// (same y, Chebyshev x) excluding its own tile and every other pole — the
    /// census's `local_alternatives`, measured live in-engine (on the final
    /// pole set, so it matches the post-hoc census computation exactly) so the
    /// stress scoreboard surfaces power-placement fragility (zero-slack poles)
    /// in the golden diff whenever a future densification change erodes it.
    PoleSlack {
        x: i32,
        y: i32,
        alternatives: i32,
    },

    // Ghost routing (Phase 2) — emitted by route_bus_ghost in ghost_router.rs
    GhostRoutingComplete {
        entity_count: usize,
        cluster_count: usize,
        max_cluster_tiles: usize,
        unroutable_count: usize,
    },
    GhostSpecRouted {
        spec_key: String,
        path_len: usize,
        crossings: usize,
        turns: usize,
        tiles: Vec<(i32, i32)>,
        crossing_tiles: Vec<(i32, i32)>,
    },
    GhostSpecFailed {
        spec_key: String,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },
    /// #526: `stamp_di_bridge` shifted a bridge column downstream of a DI
    /// cell producer's LAST drop (`RowSpan::output_feed_x_min`) — emitted
    /// only when `shift > 0`, i.e. the naive consumer-aligned column would
    /// have missed some of the cell's total output (permanently, not just
    /// occasionally — belts are one-directional). `columns` counts how many
    /// bridge inserters moved for this producer/consumer pair.
    DiBridgeShifted {
        item: String,
        producer_recipe: String,
        consumer_recipe: String,
        shift: i32,
        columns: usize,
    },

    // Ghost routing (Phase 3) — emitted by resolve_clusters in ghost_router.rs
    GhostClusterSolved {
        cluster_id: usize,
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        boundary_count: usize,
        variables: u32,
        clauses: u32,
        solve_time_us: u64,
    },
    GhostClusterFailed {
        cluster_id: usize,
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        boundary_count: usize,
    },

    // Emitted after a junction solution is stamped and ghost-surface
    // tiles belonging to participating specs are evicted from inside
    // the footprint. `participating_count` is the number of specs the
    // strategy claimed authority over; `released_count` is how many
    // ghost tiles the release call actually evicted (may include
    // ghosts from earlier phases that still held a GhostSurface claim).
    GhostResidueCleared {
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        participating_count: usize,
        released_count: usize,
    },

    // Sync-gap assertion. After a junction solution is stamped, any
    // `ghost:*` entity still in the local entity list whose (x,y)
    // sits inside the footprint is a leaked ghost — the release set
    // missed it, or a new code path pushes it after the sync. Never
    // fires on a healthy pipeline; its presence in a snapshot is the
    // signal to investigate.
    GhostResidueLeaked {
        zone_x: i32,
        zone_y: i32,
        leaked_tiles: Vec<(i32, i32)>,
    },

    // `JunctionTemplateRejected` (emitted by the perpendicular-template
    // rung's `try_bridge`/`bridge_belt_over_pipe`) was DELETED 2026-08-22
    // (#689/#691) along with the rung itself: production-unreachable on
    // both shapes it handled (belt×belt two-item crossings die on
    // junction_solver's item-conflict gate before the rung's single-tile
    // window; pipe×belt never seeds — `keys_at_tile` filters pipes out
    // before the rung could see them), and #691's census hard-asserted
    // 0 of 111 seeds matched its remaining hypothesis. See
    // `docs/offpath-code-followups.md` G1 for the full evidence trail.
    // Precedent: #632 A4 (`RouteFailure`/`BridgeDropped`) — a variant
    // with zero emitters after a deletion is removed, not kept dark.

    // DIAGNOSTIC: fires once per `build_bus_layout` run, after row
    // placement + lane planning. Captures a compact fingerprint of the
    // layout's geometric decisions so we can compare native-vs-WASM
    // output and pin down where target-dependent iteration order
    // leaks into the pipeline. Not used by any renderer; purely for
    // root-causing reproducibility bugs.
    PipelineDiagnostics {
        /// Solver's dependency_order in iteration order.
        dep_order: Vec<String>,
        /// Row layout fingerprint, row-index order. Each entry packs
        /// `recipe,y_start,y_end` into a single string so the trace
        /// serialises cleanly through `tsify_next` (which chokes on
        /// heterogeneous tuples in `Vec`).
        rows: Vec<String>,
        /// Bus lane layout fingerprint, lane-order. Each entry packs
        /// `item,x,rate,is_fluid`.
        lanes: Vec<String>,
    },

    // Emitted by `junction_solver::solve_crossing` when a strategy
    // accepts the junction and its solution is chosen as the winner for
    // this growth iteration. Terminal event — at most one per cluster.
    JunctionSolved {
        tile_x: i32,
        tile_y: i32,
        strategy: String,
        growth_iter: usize,
        region_tiles: usize,
    },

    // Emitted once per variant whose strategy produces a walker-valid
    // solution. Multiple candidates per iter are expected — the cost
    // score decides which one `JunctionSolved` will ship. The loser
    // candidates exist only in the trace, not in the final layout.
    JunctionCandidateSolved {
        tile_x: i32,
        tile_y: i32,
        strategy: String,
        growth_iter: usize,
        /// `""` for the primary attempt on the current region,
        /// `"variant-west"` / `-north` / `-east` / `-south` for the
        /// speculative single-side expansions.
        variant: String,
        region_tiles: usize,
        cost: u32,
    },

    // Emitted at the point `solve_crossing` picks the cheapest candidate
    // across all variants of a single growth iter. `considered` is the
    // full `(variant_label, cost)` list the selector chose from, in the
    // order candidates were produced — the debugger uses it to show why
    // a particular variant won and what the alternatives cost.
    JunctionVariantChosen {
        tile_x: i32,
        tile_y: i32,
        iter: usize,
        variant: String,
        cost: u32,
        considered: Vec<(String, u32)>,
    },
    // Emitted when the growth loop gives up: either frontier exhausted
    // (all participating belts fully consumed) or tile cap hit.
    JunctionGrowthCapped {
        tile_x: i32,
        tile_y: i32,
        iters: usize,
        region_tiles: usize,
        reason: String,
    },

    // Diagnostic: when a cluster fails to solve, which spec(s) made
    // the difference? Emitted once per failed cluster (gated on
    // `SPAGHETTIO_BLAME_JUNCTIONS=1`). Each event names one spec whose
    // removal lets the rest of the cluster solve. Multiple events for
    // one cluster mean any of those individual removals would unblock
    // it; zero events mean no single-spec removal helps (multi-spec
    // entanglement, or a structurally unsolvable cluster).
    JunctionBlamedSpec {
        /// Seed of the failed cluster.
        cluster_x: i32,
        cluster_y: i32,
        /// Total participating specs in the cluster.
        participating: usize,
        /// The spec whose removal would have let the cluster solve.
        spec_key: String,
        spec_item: String,
        /// Direction string ("North"/"East"/"South"/"West") at the
        /// initial cluster tile, or empty if not classifiable.
        spec_direction: String,
    },
    // Emitted when the region walker rejects a strategy's proposed
    // solution because it would break a routed path that touches the
    // region's footprint. Caller treats this the same as the strategy
    // returning `None`: fall through to the next strategy, and if all
    // strategies fail (or are vetoed), grow and retry.
    RegionWalkerVeto {
        tile_x: i32,
        tile_y: i32,
        strategy: String,
        growth_iter: usize,
        /// Variant label (see `JunctionGrowthIteration::variant`). Empty
        /// for the primary attempt at this iter.
        variant: String,
        /// Segment id of the first broken path (there may be more).
        broken_segment: String,
        /// Tile where the walker's check fired for that path.
        break_tile_x: i32,
        break_tile_y: i32,
        /// Total number of breaks (one per affected path that failed).
        break_count: usize,
    },

    // Junction solver step-through instrumentation.
    // These fire alongside the coarser `JunctionSolved` /
    // `JunctionGrowthCapped` / `RegionWalkerVeto` events to give a full
    // per-iteration view of the growth loop and each strategy attempt.
    // Designed for CLI replay + UI step-through.
    /// Emitted once per `solve_crossing` call, at entry (iteration 0
    /// not yet attempted). Reports the seed and the specs that will
    /// participate.
    JunctionGrowthStarted {
        seed_x: i32,
        seed_y: i32,
        participating: Vec<ParticipatingSpec>,
        /// Stamped entities within `seed_bbox + 1` perimeter that could
        /// physically affect the zone (splitters, belts, UG belts).
        /// Useful for understanding external feeds before growth starts.
        nearby_stamped: Vec<StampedNeighbor>,
    },

    /// Emitted at the start of each growth iteration, *before*
    /// strategies are tried. Reports the full zone state at that
    /// moment.
    JunctionGrowthIteration {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        /// Sub-iteration label. Empty string for the primary attempt on
        /// the current region; otherwise names a speculative single-side
        /// expansion variant ("variant-west", "variant-north",
        /// "variant-east", "variant-south"). The debugger groups the
        /// per-iter state keyed by `(iter, variant)` so variants don't
        /// overwrite each other.
        variant: String,
        bbox_x: i32,
        bbox_y: i32,
        bbox_w: u32,
        bbox_h: u32,
        tiles: Vec<(i32, i32)>,
        forbidden_tiles: Vec<(i32, i32)>,
        boundaries: Vec<BoundarySnapshot>,
        participating: Vec<String>,
        encountered: Vec<String>,
    },

    /// Emitted after each strategy.try_solve call within an iteration.
    /// One per (iter, strategy) pair. Carries the outcome verdict —
    /// includes walker-veto as Vetoed, template-rejection as Rejected,
    /// SAT UNSAT as Unsatisfiable, success as Solved.
    JunctionStrategyAttempt {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        /// Variant label (see `JunctionGrowthIteration::variant`). Empty
        /// for the primary attempt at this iter.
        variant: String,
        strategy: String,
        outcome: String,
        detail: String,
        elapsed_us: u64,
    },

    /// Emitted by the SAT strategy every time the SAT solver is invoked
    /// (`solve_crossing_zone_per_channel` / `_with_cost_cap` — the legacy
    /// `solve_crossing_zone` wrapper this comment once named was deleted
    /// 2026-08-20, offpath Tier 1), with the full invocation signature.
    /// This is enough to
    /// replay a single SAT solve in isolation (outside the larger
    /// junction solver). Complements JunctionStrategyAttempt with
    /// SAT-specific numbers.
    // Emitted once per cost-descent iteration in the SAT strategy.
    // `descent_iter` is 0-indexed; `cap` is the hard cost cap used
    // on that attempt. `satisfied=true` means SAT found a layout
    // within the cap (descent continues with a tighter cap);
    // `satisfied=false` means UNSAT (descent halts, prior best is
    // optimal at this cap — note `cap` may have been CLAMPED below
    // `best_cost - 1` by the descent size breaker, in which case
    // UNSAT proves nothing about the band between the clamped cap
    // and the natural one; see the HONEST LIMIT note at the breaker
    // in `junction_sat_strategy.rs`). Terminal: at most
    // `cost_descent_max_iters` per winning SAT invocation.
    SatCostDescent {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        variant: String,
        descent_iter: u8,
        cap: u32,
        satisfied: bool,
        solve_time_us: u64,
        /// New best cost when this descent step improved on the prior
        /// best. `None` on UNSAT or the safety-bail branch (SAT but cost
        /// didn't drop). Lets analyzers measure descent deltas and
        /// detect stalls without re-computing cost.
        cost_after: Option<u32>,
    },

    SatInvocation {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        /// Variant label (see `JunctionGrowthIteration::variant`). Empty
        /// for the primary attempt at this iter.
        variant: String,
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        boundaries: Vec<BoundarySnapshot>,
        forced_empty: Vec<(i32, i32)>,
        belt_tier: String,
        max_reach: u32,
        satisfied: bool,
        variables: u32,
        clauses: u32,
        solve_time_us: u64,
        entities_raw: usize,
        /// Cost of the raw SAT solution, before the cost-descent loop
        /// tightens it. `None` when `satisfied=false`. Analyzers
        /// compare against the final `cost_after` of the last
        /// improving `SatCostDescent` event to measure descent savings.
        initial_cost: Option<u32>,
        /// Entities SAT produced, captured before `prune_dangling_sat_entities`.
        /// Empty when `satisfied=false`. Lets the junction debugger render
        /// the candidate layout — especially useful on walker veto, where
        /// the solution is otherwise discarded.
        proposed_entities: Vec<SatProposedEntity>,
    },

    // Phase-1 instrumentation: emitted after all ghost specs are routed but
    // before crossing resolution. Reports per-tile axis occupancy so we can
    // see same-axis conflicts (Phase 2 negotiation target).
    GhostAxisOccupancy {
        tiles: Vec<GhostAxisOccupancyTile>,
        same_axis_conflict_count: u32,
        perpendicular_crossing_count: u32,
    },

    // Phase-2 negotiation: emitted once per iteration of the negotiation
    // loop in `route_bus_ghost`. The loop bumps a per-tile per-axis cost
    // grid each time it sees same-axis pile-ups, and re-routes until the
    // conflict count stops improving.
    GhostNegotiationIteration {
        iter: u32,
        same_axis_conflict_count: u32,
        perpendicular_crossing_count: u32,
        unroutable_count: u32,
        cost_grid_size: u32,
    },

    /// Emitted by `region_reimprove::descend` once per strictly-cheaper
    /// layout found during an interactive improve-region pass. The
    /// frontend uses these to animate the zone morphing toward an
    /// optimal layout. Not emitted by the in-layout descent (which uses
    /// `SatCostDescent` for its per-iteration trace).
    SatImprovement {
        /// `LayoutRegion.id` of the zone being improved.
        region_id: u32,
        /// Absolute bbox of the zone — redundant with the LayoutRegion
        /// but convenient for frontend consumers that don't want to
        /// re-look-up the region on every event.
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        /// Total belt+UG cost of `entities` under `junction_cost::solution_cost`.
        cost: u32,
        /// Descent iteration — 0 means the initial (pre-descent) snapshot.
        iter: u32,
        /// Microseconds spent in the solver for this iteration. 0 for
        /// the initial snapshot.
        solve_time_us: u64,
        /// Full entity list for the zone at this descent step. Replaces
        /// whatever was at these tiles before.
        entities: Vec<PlacedEntity>,
    },

    /// Emitted by `region_reimprove::descend` (via the WASM binding) when
    /// a descent terminates with `StopReason::Optimal` — the cap-1 probe
    /// returned UNSAT, so the current layout is provably the cheapest
    /// solution for this zone. Carries the canonical signature plus a
    /// single-record binary blob in the same format used by
    /// `crates/core/data/sat-zones.bin`, so the frontend can persist the
    /// result to localStorage and seed it back into the cache on next
    /// boot via [`crate::zone_cache::install_prebaked`].
    SatOptimumProven {
        /// `LayoutRegion.id` of the zone whose descent just proved optimal.
        region_id: u32,
        /// Cache key for this zone (canonical, orientation-invariant).
        signature: String,
        /// Single-record binary blob — concatenable with other records
        /// to form a full cache file.
        record_bytes: Vec<u8>,
    },

    // SAT solution pruned of dangling (unreachable / dead-end) belt entities.
    SatPruned {
        zone_x: i32,
        zone_y: i32,
        total: usize,
        kept: usize,
    },

    /// Defensive guard for issue #163. Emitted when a participating spec's
    /// boundary set, after `topology_boundaries` + chain-head augmentation,
    /// has at least one OUT boundary but zero IN boundaries for the item.
    /// This is the "items appear from thin air" failure mode: SAT would
    /// solve the under-constrained zone with output-only flows.
    ///
    /// The event is observational only — `try_solve` does NOT reject the
    /// solve based on it, because the asymmetry can also arise legitimately
    /// for `Encountered` specs whose IN boundary is provided by an
    /// upstream cluster's commit. Future regressions in
    /// `topology_boundaries` boundary derivation that re-introduce the
    /// silent "no IN boundary" path will surface in the snapshot
    /// debugger as this event firing on a participating spec, instead
    /// of a silent broken layout.
    SatBoundariesAsymmetric {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        variant: String,
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        item: String,
    },

    // Emitted by the final ghost-router render pass once a spec's path
    // has been materialised into belt/UG entities. Carries the full
    // entity list so a streaming renderer can swap its per-tile "ghost
    // belt" placeholders for the real rendered entities (with correct
    // turns, UG pairs, etc.). Fires once per spec, after
    // `GhostSpecRouted` for that spec.
    GhostSpecCommitted {
        spec_key: String,
        entities: Vec<PlacedEntity>,
    },

    // Emitted by the ghost-router's junction-solver loop after a
    // cluster's SAT solution has been stamped into the layout. Carries
    // the entities the solver placed inside the zone + the spec keys
    // whose prior ghost-routed belts inside the zone are now
    // invalidated (participating). A streaming renderer uses this to
    // fade out the ghost belts inside the footprint and fade in the
    // real SAT-placed entities.
    JunctionCommitted {
        cluster_id: usize,
        zone_x: i32,
        zone_y: i32,
        zone_w: u32,
        zone_h: u32,
        entities: Vec<PlacedEntity>,
        participating: Vec<String>,
    },

    // Eviction-strategy events. Eviction runs as a fallback after SAT
    // returns UNSAT for every variant on this iteration. Each recipe
    // selects one or more participating specs, routes them around or
    // through the bbox via A*/geometric pre-pass, then re-invokes SAT
    // on the reduced spec set.
    EvictionAttempted {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        recipe: String,
        candidate_spec_keys: Vec<String>,
        region_tiles: usize,
        boundary_count_before: usize,
    },
    EvictionRouteFailed {
        seed_x: i32,
        seed_y: i32,
        recipe: String,
        spec_key: String,
        reason: String,
        elapsed_us: u64,
    },
    EvictionSatFailed {
        seed_x: i32,
        seed_y: i32,
        recipe: String,
        evicted_spec_keys: Vec<String>,
        elapsed_us: u64,
    },
    EvictionSucceeded {
        seed_x: i32,
        seed_y: i32,
        iter: usize,
        recipe: String,
        evicted_spec_keys: Vec<String>,
        boundary_count_after: usize,
        sat_us: u64,
        route_us: u64,
        total_us: u64,
        /// Per-evicted-spec metrics — Manhattan length of the evicted
        /// route, turn count of the rendered path, item, belt tier name.
        /// Captured for the recipe-grid pattern table.
        metrics: Vec<EvictionSpecMetric>,
    },
    EvictionBudgetExhausted {
        seed_x: i32,
        seed_y: i32,
        recipes_tried: usize,
        total_us: u64,
    },

    // Decomposition-search layer (see `docs/rfc-decomposition-search.md`).
    // The search-and-score loop scores each `DecompositionCandidate` it
    // evaluates and emits one of these per candidate; then a single
    // `DecompositionChosen` once the winner is selected. With Phase 0's
    // single `NativeCandidate` catalogue, exactly one
    // `DecompositionCandidateScored` and one `DecompositionChosen` fire
    // per layout call.
    DecompositionCandidateScored {
        name: String,
        density: f64,
        overproduction: f64,
        entity_count: usize,
        score: f64,
        /// True iff hard constraints (demand met, all balancer shapes
        /// resolvable) are satisfied. Phase 0 stub always emits `true`;
        /// Phase 1b activates the actual rejection logic.
        accepted: bool,
        /// When `accepted == false`, a short tag describing why
        /// (e.g. `"missing-balancer-template"`). `None` when accepted.
        accepted_reason: Option<String>,
    },
    DecompositionChosen {
        name: String,
        score: f64,
    },

    // RFC-070 Phase 0b (#689 W1b): the SELECTION SCOREBOARD. One
    // `SelectionCandidateEvaluated` per candidate slot of
    // `select_best_decomposition` (all seven, every call — including the
    // ones that never ran), then one `SelectionDecided` naming the winner
    // and the precedence stage that picked it. The two are emitted
    // adjacently at the very end of the selection, so a stream walker can
    // pair them by "flush the pending candidates when a `SelectionDecided`
    // arrives" without a nested selection (a candidate whose `produce`
    // runs its own search, replayed inside the winner's events) splicing
    // itself into the outer block.
    //
    // Purely observational: the fields RECORD what each measurement site
    // already computed on the decision path, never a recomputation, which
    // is why so many are `Option`. A `None` means "no site computed
    // this for this candidate on this call" — an oracle GAP, not a zero.
    // Reading one as 0 is the `unwrap_or(0)` trap that has silently
    // reported "no findings" here before.
    SelectionCandidateEvaluated {
        /// Candidate name, matching `DecompositionCandidate::name`.
        name: String,
        outcome: SelectionCandidateOutcome,
        /// `produce()`'s own error text when `outcome` is `Refused`, or the
        /// caught-panic tag when `Panicked`. `None` otherwise — a candidate
        /// that was never run carries no reason string, because the gating
        /// predicate lives at the call site, not in `produce`.
        reason: Option<String>,
        /// Soft score (`score_layout`) and its acceptance verdict — `Some`
        /// iff the candidate produced a layout. `accepted`
        /// carries only the `missing-balancer-template` hard gate; it is
        /// NOT a validation verdict.
        score: Option<f64>,
        accepted: Option<bool>,
        accepted_reason: Option<String>,
        /// `IssueCounts` as computed by whichever measurement site ran
        /// first — `errors` / `selection_warning_count` /
        /// `LayoutResult.warnings.len()`. All three are `None` together
        /// when no site measured this candidate's counts.
        errors: Option<usize>,
        selection_warnings: Option<usize>,
        layout_warnings: Option<usize>,
        /// Which site FIRST computed the counts above (`"di-vs-native"`,
        /// `"horizontal-vs-native"`, `"clean-flags"`) — provenance of the
        /// number, **not** the site that decided. Recording is
        /// first-write-wins and several sites can measure the same
        /// candidate's counts, so a later policy stage can inherit an earlier tag;
        /// the value is identical either way (same deterministic call on
        /// the same layout). For "who decided", read `SelectionDecided`'s
        /// stage.
        counts_source: Option<String>,
        /// `ErrorKinds` as measured for the merge-tap profile. A row is a
        /// per-candidate summary of every projection measured for it, not
        /// the record of a separate decision. The decision is
        /// `SelectionDecided::stage`, and only that.
        contamination_errors: Option<usize>,
        starvation_errors: Option<usize>,
        structural_errors: Option<usize>,
        /// RFC-071 B2: the class that DOMINATES the quality key must be
        /// visible in the stream — a walker replaying a shipped
        /// selection would otherwise see a winner flip with no visible
        /// cause (#716 review round 1). `serde(default)` so pre-B2 `.fls`
        /// snapshots keep decoding (round 2) — absent reads as None,
        /// which is truthful: nothing measured the class then.
        #[serde(default)]
        route_severed_errors: Option<usize>,
        /// RFC-071 B3 (#717 review round 2): the verification standing
        /// the best-error-free ordering ranks on — a decision-serving
        /// projection like every other field here; a row-replay that
        /// defaulted it to false would reconstruct the OPPOSITE of the
        /// shipped #700 rule. `serde(default)` keeps pre-B3 snapshots
        /// decoding (absent = false = the pre-B3 world, truthfully).
        #[serde(default)]
        unverified_geometry: bool,
    },
    SelectionDecided {
        winner: String,
        stage: SelectionStage,
    },

    // RFC-069 gap-convergence pass (layout.rs pass 3): fired once when
    // pass 2's re-planned families needed different balancer gaps than
    // the placement consumed and a third place+plan iteration ran with
    // the converged needs. `converged == false` means the THIRD pass's
    // families disagree again (an oscillating gap map — a new fixture
    // class to diagnose, deliberately not looped on); `applied` is the
    // `(last_producer_row, extra_gap)` map the final placement consumed —
    // the pass-2 balancer needs MERGED with any retry slack (#722
    // round 1: the two differ on retried fixtures).
    GapConvergence {
        converged: bool,
        applied: Vec<(usize, i32)>,
    },

    // RFC-072 Phase 1: a sibling lane group's consumer assignment was
    // repaired because a NON-LAST tap's splitter tile (x+1, tap_y-1)
    // fell inside a sibling trunk column still occupied at that y —
    // the round-robin assignment would have committed a sourceless tap
    // run (six dead machines on the cable-90 specimen, sim −50.2%).
    // `reassigned` maps lane x → its new consumer row set (contiguous
    // y-blocks, topmost block on the rightmost sibling so its column
    // ends in an immediate turn). Fires once per repaired group.
    TapAssignmentRepaired {
        item: String,
        module_id: u32,
        reassigned: Vec<(i32, Vec<usize>)>,
    },

    // The repair detected a collision it could NOT clear (foreign
    // occupant, or every sibling permutation re-collides) and restored
    // the original assignment. The layout ships with the collision —
    // this event is the loud hook for the router-loudness follow-up
    // recorded in RFC-072's decision log (#727 round 2).
    TapAssignmentUnrepairable {
        item: String,
        module_id: u32,
    },

    // RFC-072 Phase 2 unit 2: the cell chain's quantization exceeded one
    // strip's K_MAX and composed as a GRID of stacked independent strips.
    // `copies_per_strip` is the balanced split (sums to the chain's K);
    // `clearance` is the inter-strip kit band; `pole_bridges` is how many
    // poles `repair_pole_network` added to join the strips' islands.
    CellGridComposed {
        copies_per_strip: Vec<i32>,
        clearance: i32,
        pole_bridges: usize,
    },

    // `ModuleSizeSplit` candidate (see `docs/rfc-decomposition-search.md`)
    // applied a k-way split to one module of the partition plan. Fires
    // once per split module per `produce()` call. With Phase 1's k=2,
    // a single original `(item, recipe)` module spawns two events with
    // the same `original_module_id` and dense reassigned sub-ids.
    ModuleSizeSplitApplied {
        item: String,
        consumer_recipe: String,
        original_module_id: u32,
        k_splits: u32,
        new_module_id: u32,
        rate: f64,
        lane_count: u32,
    },

    // Partitioner cap-driven split: a module's per-trunk rate would
    // bust full belt capacity (`lane_capacity * 2`), so it was split
    // into `k_splits = ceil(rate / full_belt_cap)` sibling sub-modules
    // each with sub-cap rate. Fires once per resulting sibling. Runs
    // unconditionally under `LayoutStrategy::PartitionedDecomposed`
    // after Phase 2 sharding; preempts the lane planner's
    // consumer-clamped fan-in panic for cases where the planner would
    // otherwise be asked to clamp a multi-producer-row lane onto a
    // single-belt consumer trunk above its capacity.
    ModuleCapSplitApplied {
        item: String,
        consumer_recipe: String,
        original_module_id: u32,
        k_splits: u32,
        new_module_id: u32,
        original_rate: f64,
        new_rate: f64,
        full_belt_cap: f64,
    },

    /// Census-only instrumentation (offpath-code-followups.md G1 follow-up,
    /// #689 W1d): emitted once per junction seed (once per outer
    /// cluster-loop iteration in `ghost_router` that reaches
    /// `junction_solver::solve_crossing` — NOT re-emitted by the muted
    /// context-conflict retry, which reuses this same `keys_at_tile`
    /// without recomputing it) — purely observational, no effect on
    /// routing (gated behind `SPAGHETTIO_JUNCTION_SEED_CENSUS`, so
    /// production pays nothing by default — see the emit site).
    ///
    /// `n_specs`/`n_distinct_items` are CLUSTER-WIDE: the union of every
    /// spec whose path touches ANY tile in the cluster, which can span
    /// multiple tiles already at seed time (before `solve_crossing`'s own
    /// growth). `n_specs > n_distinct_items` therefore means "this
    /// cluster's participants include an item-sharing pair SOMEWHERE in
    /// the cluster" — not "one tile has two same-item specs", which is
    /// the rung's actual `tile_count == 1` predicate. `cluster_tile_count`
    /// (the cluster's tile count at seed time) lets a consumer recover the
    /// precise question: when `cluster_tile_count == 1`, the cluster IS a
    /// single tile, so `keys_at_tile` is exactly the spec set at that one
    /// tile and `n_specs > n_distinct_items` becomes a true single-tile
    /// same-item crossing (#689 W1d review round 1 caught the conflation
    /// in an earlier version of this doc comment).
    ///
    /// `has_pipe` is measured over the RAW spec set touching the
    /// cluster's tiles *before* `keys_at_tile`'s `SpecKind::Pipe` filter
    /// runs (that filter is why pipes can never appear in
    /// `n_specs`/`n_distinct_items` — see #687) — kept as a corroborating
    /// receipt of that finding, not a new hypothesis.
    JunctionSeedCensus {
        seed_x: i32,
        seed_y: i32,
        /// Number of tiles in this seed's cluster at seed time (before any
        /// growth). `1` means `n_specs`/`n_distinct_items` describe a
        /// single physical tile; `>1` means they're a cluster-wide union
        /// that may conflate specs from different tiles.
        cluster_tile_count: usize,
        n_specs: usize,
        n_distinct_items: usize,
        has_pipe: bool,
    },
}

// ---------------------------------------------------------------------------
// Summary structs (lightweight, serializable versions of internal types)
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipatingSpec {
    pub key: String,
    pub item: String,
    pub initial_tile_x: i32,
    pub initial_tile_y: i32,
    /// Full path tile count (for context on how much can be grown into
    /// the region from each end of this spec).
    pub path_len: usize,
    /// Initial frontier (start, end) index into the path.
    pub initial_start: usize,
    pub initial_end: usize,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StampedNeighbor {
    pub x: i32,
    pub y: i32,
    pub name: String,
    /// Direction the entity faces (belts / splitters / UG).
    pub direction: String,
    pub carries: Option<String>,
    pub segment_id: Option<String>,
    /// True if this entity's output would land on a tile within the
    /// initial seed's 1-tile perimeter (hint for "this might sideload").
    pub feeds_seed_area: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySnapshot {
    pub x: i32,
    pub y: i32,
    pub direction: String,
    pub item: String,
    pub is_input: bool,
    /// True iff the strategy moved this boundary onto a Permanent
    /// entity's tile inside the bbox (in `forced_empty`). The encoder
    /// then propagates flow constraints to the in-zone neighbour rather
    /// than placing an entity at this tile.
    pub interior: bool,
    /// Spec key that produced this boundary. Useful for correlating a
    /// growth iteration with the specs' movement frontiers.
    pub spec_key: String,
    /// Whether this boundary comes from a spec that seeded the cluster
    /// (`"participating"`) or a spec that merely passes through the
    /// cluster's bbox (`"encountered"`). Encountered specs contribute
    /// boundary pairs so SAT can route them instead of treating their
    /// belts as forbidden obstacles.
    pub origin: String,
    /// If a physical external feeder landed items on this tile, the
    /// feeder's entity name + output direction. `None` means no
    /// external feeder — SAT will assume native (opposite(direction))
    /// arrival.
    pub external_feeder: Option<ExternalFeederSnapshot>,
    /// Surface-belt-tier name (`"transport-belt"` /
    /// `"fast-transport-belt"` / `"express-transport-belt"`) of the
    /// external entity this boundary connects to, if known. Mirrors
    /// `ZoneBoundary::belt_tier`. Used by solve-time entity stamping to
    /// pick the right belt/UG entity name for this channel. `None`
    /// means "unknown — use the zone's default tier."
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belt_tier: Option<String>,
    /// Channel id — matches `ZoneBoundary::channel_id`. Boundaries that
    /// share a channel_id route on the same SAT flow. Inputs and
    /// outputs of the same channel are the IN/OUT pair the encoder
    /// will connect. Surfaced in debug JSON to make tier-based pairings
    /// visually obvious.
    #[serde(default)]
    pub channel_id: u32,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFeederSnapshot {
    pub entity_name: String,
    pub entity_x: i32,
    pub entity_y: i32,
    pub direction: String,
}

/// Minimal view of an entity SAT proposed for a crossing zone. Captured
/// pre-prune so the junction debugger can show exactly what SAT produced
/// — including entities that the dangling-prune step later drops.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatProposedEntity {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carries: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_type: Option<String>,
}

/// Per-evicted-spec metric attached to `EvictionSucceeded`. Captures
/// shape of the evicted route so the diagnostic sweep can ask "what
/// kind of specs do we win on?".
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionSpecMetric {
    pub spec_key: String,
    pub item: String,
    pub belt_tier: String,
    /// Manhattan distance between the spec's entry tile and exit tile.
    pub manhattan_len: u32,
    /// Number of direction changes along the rendered route.
    pub turn_count: u32,
    /// Number of entities the route emitted.
    pub entity_count: usize,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostAxisOccupancyTile {
    pub x: i32,
    pub y: i32,
    /// Number of routed specs whose axis at this tile is Vertical (N/S).
    pub vert_count: u32,
    /// Number of routed specs whose axis at this tile is Horizontal (E/W).
    pub horiz_count: u32,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowInfo {
    pub index: usize,
    pub recipe: String,
    pub machine: String,
    pub machine_count: usize,
    pub y_start: i32,
    pub y_end: i32,
    pub row_kind: String,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneInfo {
    pub item: String,
    pub x: i32,
    pub rate: f64,
    pub is_fluid: bool,
    pub source_y: i32,
    pub tap_off_ys: Vec<i32>,
    pub consumer_rows: Vec<usize>,
    pub producer_row: Option<usize>,
    pub extra_producer_rows: Vec<usize>,
    pub family_id: Option<usize>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyInfo {
    pub item: String,
    /// `0` under `LayoutStrategy::Pooled`. Distinguishes multiple
    /// `(item, module_id)` families per item under the partitioning
    /// strategies — see `docs/rfc-modular-production.md`.
    #[serde(default)]
    pub module_id: u32,
    pub shape: (usize, usize),
    pub lane_xs: Vec<i32>,
    pub balancer_y_start: i32,
    pub balancer_y_end: i32,
    pub total_rate: f64,
    pub producer_rows: Vec<usize>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineTrace {
    pub recipe: String,
    pub machine: String,
    /// Fractional machine count (e.g. 2.4 → ceil to 3 in practice)
    pub count: f64,
    /// Total output rate of this machine group (items/s)
    pub rate: f64,
}

/// What happened to one candidate slot of `select_best_decomposition`.
/// `NotRun` and `Refused` are DIFFERENT facts: `NotRun` means the call
/// site's gating predicate was false so `produce` was never called (no
/// layout pass was paid for), while `Refused` means `produce` ran and
/// returned `Err` — including the three arms that self-validate and
/// refuse their own error-carrying layout.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionCandidateOutcome {
    Produced,
    Refused,
    Panicked,
    NotRun,
}

/// The policy stage of `select_best_decomposition` that picked the winner.
/// This CLOSED enum is exhaustive by construction: every return path of
/// the policy maps to exactly one variant, so a future stage added without
/// a variant here fails to compile.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionStage {
    /// The policy's scoped Pooled merge-tap decision
    /// (`ErrorKindCounts::quality_key`). Covers both outcomes of that
    /// stage: merge-tap won, or native held against it.
    MergeTap,
    /// The policy's DI / horizontal-stack component-wise `IssueCounts`
    /// floor.
    ScopedPairwise,
    /// The policy's error-free validation tier (#392).
    BestErrorFree,
    /// The policy's best-soft-score stage among accepted candidates.
    BestAccepted,
    /// The policy's positional fallback: first candidate that produced
    /// anything.
    FirstProduced,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssueTrace {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `remove_capped_events_since` drops only `InserterSideCapped` at or
    /// after `start`, preserving order of everything else — the pass-2
    /// scrub must not disturb earlier events or non-capped pass-1 events
    /// (RFC validation-explainability D2).
    #[test]
    fn remove_capped_events_since_is_selective() {
        let _guard = start_trace();
        let capped = |x: i32| TraceEvent::InserterSideCapped {
            recipe: "electronic-circuit".into(),
            side_is_output: false,
            required: 1.0,
            placed_entity: "long-handed-inserter".into(),
            placed_count: 1,
            shortfall: 0.2,
            machine_x: x,
            machine_y: 0,
            limit: "geometry".into(),
        };
        emit(capped(1)); // before `start` — must survive
        let start = peek_events_len();
        emit(capped(2)); // in range — must be removed
        emit(TraceEvent::PhaseTime {
            phase: "place_rows_1".into(),
            duration_ms: 1,
        });
        emit(capped(3)); // in range — must be removed

        remove_capped_events_since(start);

        let events = peek_events_since(0);
        assert_eq!(events.len(), 2, "{events:?}");
        assert!(matches!(
            &events[0],
            TraceEvent::InserterSideCapped { machine_x: 1, .. }
        ));
        assert!(matches!(&events[1], TraceEvent::PhaseTime { .. }));
    }
}
