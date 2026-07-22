//! Shared data models for the Spaghettio pipeline.
//!
//! Rust port of `src/models.py`. These types flow through every pipeline stage:
//! solver → layout → blueprint export → validation.
//!
//! Key types:
//! - [`ItemFlow`] — an item (or fluid) flowing at a given rate
//! - [`MachineSpec`] — one recipe step: machine type, count, inputs/outputs
//! - [`SolverResult`] — the full solved production graph
//! - [`PlacedEntity`] — a single entity placed on the tile grid (belt, machine, inserter, etc.)
//! - [`LayoutResult`] — the complete spatial layout ready for blueprint export

use serde::{Deserialize, Serialize};

/// An item flowing at a certain rate.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFlow {
    pub item: String,
    pub rate: f64,
    pub is_fluid: bool,
    /// Module index when partitioning strategies are in use. `0` under
    /// `LayoutStrategy::Pooled` (one module per item — today's
    /// behaviour). The solver always emits `0`; the lane planner /
    /// placer rewrites this in Phase 1+ when partitioning multi-consumer
    /// items. See `docs/rfc-modular-production.md`.
    #[serde(default)]
    pub module_id: u32,
}

/// One production step: which machine, which recipe, how many.
///
/// For self-loop recipes (kovarex-class: an item on both sides of the
/// recipe), `inputs`/`outputs` carry NET external flows only — the item
/// appears in exactly one of them, signed by its net direction — so the
/// bus/lane/placer layers see an ordinary recipe. The raw row-internal
/// recirculation lives in `self_loop` for the row template to size the
/// loop-back belt. See docs/rfc-solver-net-flow.md Phase 2(c).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineSpec {
    pub entity: String,
    pub recipe: String,
    pub count: f64,
    pub inputs: Vec<ItemFlow>,
    pub outputs: Vec<ItemFlow>,
    /// Row-internal recirculated flows for self-loop recipes (empty for
    /// ordinary recipes).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub self_loop: Vec<SelfLoopFlow>,
    /// True for a layout-synthesized voider row (RFC Fulgora Phase 2,
    /// `docs/rfc-fulgora-scrap.md` D1) — a recycler bank that
    /// self-consumes a solid surplus stream down to nothing. Never set
    /// by the solver (voiding is a layout policy, not a solver
    /// objective); set only by `bus::voider::synthesize_voiders`.
    /// `inputs` carries the bus-visible tap demand (per-machine rate,
    /// matching every other `MachineSpec`'s convention); `outputs` is
    /// always empty — nothing is exported. Drives `RowKind::Voider`
    /// classification in `bus::placer::row_kind`.
    #[serde(default)]
    pub voider: bool,
    /// GAME modules planned for each machine of this spec (RFC-044
    /// Phase 3; empty when the module policy is `None` or the
    /// (machine, recipe) pair is ineligible). The layout's stamping
    /// post-pass copies this into `PlacedEntity.items`. Named
    /// `game_modules` to keep clear of the partition-module vocabulary
    /// (`ItemFlow::module_id`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_modules: Vec<ModuleItem>,
}

/// One self-referencing item of a self-loop recipe: raw per-machine
/// consumed/produced rates plus the net (already reflected in the owning
/// spec's `inputs`/`outputs`).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfLoopFlow {
    pub item: String,
    pub is_fluid: bool,
    /// Raw per-machine consumption (items/s), e.g. kovarex U-235: 40/60.
    pub consumed_rate: f64,
    /// Raw per-machine production (items/s), e.g. kovarex U-235: 41/60.
    pub produced_rate: f64,
    /// `produced − consumed` per machine; sign matches which of the
    /// spec's `inputs`/`outputs` carries the item.
    pub net_rate: f64,
}

/// Everything the solver produces — no positional data.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverResult {
    pub machines: Vec<MachineSpec>,
    pub external_inputs: Vec<ItemFlow>,
    /// The requested target item(s) only. Deliberately does NOT include
    /// byproduct surplus — `decomposition_search::compute_overproduction`
    /// and the step-7 output merger both assume entries here are targets.
    pub external_outputs: Vec<ItemFlow>,
    /// Byproduct produced beyond internal demand (net-flow solver only;
    /// always empty from the legacy tree walk). Routing these to the
    /// perimeter is Phase 2 of docs/rfc-solver-net-flow.md — until then a
    /// non-empty entry here means the layout physically strands the flow,
    /// which the port-extraction validator reports as an error.
    #[serde(default)]
    pub surplus_outputs: Vec<ItemFlow>,
    pub dependency_order: Vec<String>,
}

/// Matches Factorio's 16-way direction constants (we only use 4).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[derive(Default)]
pub enum EntityDirection {
    #[default]
    North = 0,
    East = 4,
    South = 8,
    West = 12,
}


/// A module/item inserted into an entity (e.g. speed-module-3 × 2 in a beacon).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleItem {
    pub item: String,
    pub count: u32,
    /// Quality of the module itself (`None` = normal, omitted from
    /// export). Parsed from the 2.0 insert-plan `id.quality` field;
    /// scales the module's beneficial effects (RFC-044 game-rule model).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<crate::common::QualityTier>,
}

/// A single entity placed in the blueprint grid.
///
/// Represents any game entity (belt, inserter, machine, pipe, pole, etc.) at a
/// specific tile position with an orientation. Flows through layout → blueprint export.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlacedEntity {
    /// Factorio entity prototype name (e.g. `"transport-belt"`, `"assembling-machine-2"`).
    pub name: String,
    /// Tile X coordinate (integer grid).
    #[serde(default)]
    pub x: i32,
    /// Tile Y coordinate (integer grid).
    #[serde(default)]
    pub y: i32,
    /// Facing direction (N/E/S/W). Corresponds to Factorio's 4-way direction
    /// constants (0/4/8/12).
    #[serde(default)]
    pub direction: EntityDirection,
    /// Recipe assigned to crafting machines (`None` for belts, inserters, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    /// I/O role tag for bus entities: `"input"`, `"output"`, or `"passthrough"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_type: Option<String>,
    /// Item or fluid name this belt/pipe is currently carrying.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carries: Option<String>,
    /// Factorio 2.0 build-quality tier of this entity (`None` = normal,
    /// omitted from export). Parsed from community blueprints since
    /// Phase 0 of `docs/rfc-build-quality.md`; stamped on functional
    /// entities (machines/inserters/poles) at export from Phase 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<crate::common::QualityTier>,
    /// Factorio Space Age fluid-box mirroring. When `true`, flips fluid port
    /// positions along the entity's primary axis, giving 8 orientations (4
    /// rotations × 2 mirrors). Ignored in Factorio 1.1. See `CLAUDE.md`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mirror: bool,
    /// Optional identifier linking this entity to a layout segment or balancer
    /// group for debugging/analysis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
    /// Throughput rate (items/s or fluid units/s) flowing through this entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
    /// Modules/items inserted into this entity (e.g. speed modules in a beacon).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ModuleItem>,
    /// Splitter input priority (`"left"` or `"right"`). Set on splitters
    /// where one input port should be preferred — under contention the
    /// non-priority input is back-pressured. Critical for balancer
    /// designs with feedback loops to avoid discrete-time stalls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_priority: Option<String>,
    /// Splitter output priority (`"left"` or `"right"`). Items go to the
    /// priority output first; when blocked they overflow to the other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_priority: Option<String>,
    /// Self-loop split-point marker (splitters only): the loop-back
    /// branch's demanded rate. The lane-rate walkers use this instead of
    /// the generic symmetric 50/50 model — the loop output receives
    /// `min(total, loop_priority_rate)` and the export side the
    /// remainder, matching a priority-output splitter feeding a
    /// saturated recirculation belt. Set only by the self-loop row
    /// template. See docs/rfc-solver-net-flow.md Phase 2(c).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_priority_rate: Option<f64>,
    /// Inserter item filter, whitelist mode only (v1 — no blacklist, no
    /// splitter `filter` field). Ordered item names, index 1-based when
    /// exported to the blueprint's `filters` array. Empty means the
    /// inserter has no filter (Factorio 2.0 default: `use_filters: false`,
    /// no `filters` field emitted). See docs/rfc-fulgora-scrap.md Phase 0
    /// "Filter entities" findings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<String>,
}

/// Whether a boundary port is an input into the region or an output from it.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortIo {
    Input,
    Output,
}

/// A point on or inside a region where a spec enters or exits. Stored in
/// absolute tile coordinates; `direction` encodes the flow direction at
/// that tile (which way items are physically moving).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortPoint {
    pub x: i32,
    pub y: i32,
    pub direction: EntityDirection,
}

/// A boundary port on a `LayoutRegion`: a point plus io/item metadata.
/// Replaces the old `PortSpec { edge, offset, … }` triple-encoding —
/// position is now stored in absolute coordinates and the edge is
/// derivable from the region's bbox when needed.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionPort {
    pub point: PortPoint,
    pub io: PortIo,
    /// Item carried through this port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    /// True iff this port sits on a Permanent entity's tile inside the
    /// bbox (see `ZoneBoundary::interior`). Needed to rebuild the zone
    /// spec faithfully when re-solving.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub interior: bool,
    /// Tier (surface-belt entity name, e.g. `"fast-transport-belt"`) of
    /// the external entity this port connects to, if known. Mirrors
    /// `ZoneBoundary::belt_tier` so the interactive improve pass can
    /// reconstruct per-boundary tiers when re-solving. Optional for
    /// backwards compatibility with older serialized layouts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belt_tier: Option<String>,
    /// Channel id — matches `ZoneBoundary::channel_id`. Ports that
    /// share a `channel_id` route on the same SAT-level flow. Carried
    /// through the serialized layout so the interactive improve pass
    /// reconstructs the same channel assignment the original solve used.
    #[serde(default)]
    pub channel_id: u32,
}

/// Origin/purpose of a `LayoutRegion`. Replaces the historical stringly-typed
/// `kind` field so classifier and histograms can exhaustive-match.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    /// Non-ghost SAT crossing-zone solver output.
    CrossingZone,
    /// Ghost-router corridor template (one horizontal spec crossing N trunks).
    CorridorTemplate,
    /// Ghost-router per-tile perpendicular crossing template.
    JunctionTemplate,
    /// Ghost-router crossing left for the junction solver to pick up.
    Unresolved,
}

/// Metadata about a resolved region in the layout (SAT crossing zone,
/// ghost-routed junction template, or unresolved placeholder).
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRegion {
    /// Stable per-layout id. Assigned sequentially when regions are
    /// emitted. Serialised so the frontend can address a specific region
    /// across worker boundaries (e.g. "improve this region").
    #[serde(default)]
    pub id: u32,
    pub kind: RegionKind,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// Boundary ports. Each port records the absolute (x, y) position,
    /// flow direction, io (input/output), and item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<RegionPort>,
    /// Tiles inside the bbox that must remain free of surface entities
    /// (tap-off passages). Populated for `CrossingZone` regions so the
    /// zone spec can be rebuilt for a re-solve; empty otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forced_empty: Vec<(i32, i32)>,
    /// Belt tier used when the zone was solved (e.g. `"transport-belt"`).
    /// Populated for `CrossingZone` regions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belt_tier: Option<String>,
    /// Underground-belt maximum reach that was used when the zone was
    /// solved. Populated for `CrossingZone` regions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ug_reach: Option<u32>,
}

/// Everything the layout engine produces — no rate data.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutResult {
    pub entities: Vec<PlacedEntity>,
    #[serde(default)]
    pub width: i32,
    #[serde(default)]
    pub height: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<LayoutRegion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<crate::trace::TraceEvent>>,
    /// `(item, x, y)` perimeter exit tile per routed fluid surplus lane —
    /// see `GhostRouteResult::surplus_exits`. Empty when nothing is
    /// surplus. Populated by the bus pipeline regardless of tracing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surplus_exits: Vec<(String, i32, i32)>,
    /// Solid surplus streams consumed by a synthesized voider row under
    /// `SurplusPolicy::Void` (RFC Fulgora Phase 2,
    /// `docs/rfc-fulgora-scrap.md` D1) — first-class, trace-independent,
    /// like `surplus_exits`. `check_stranded_byproducts` cross-checks
    /// each entry against real recycler entities rather than trusting
    /// this ledger alone. Empty under `SurplusPolicy::Export` or when
    /// nothing voided.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voided_streams: Vec<VoidedStream>,
    /// Per-row `(y_start, y_end, spec)` attribution — one entry per
    /// physical machine row the layout pipeline actually placed, first-
    /// class and trace-independent like `voided_streams`/`surplus_exits`.
    /// See [`EffectiveRow`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effective_rows: Vec<EffectiveRow>,
    /// Pole-to-pole copper wire graph as `(a, b)` index pairs into `entities`
    /// (`a < b`), from [`crate::power_wires::compute_pole_wires`]. RFC-045
    /// stored-graph contract: `Some` is AUTHORITATIVE — `blueprint::export`
    /// and the connectivity validator consume it verbatim via
    /// [`crate::power_wires::wires_for`] (so the overlay, the artifact, and
    /// the validated graph are the same object); `None` means "never
    /// computed" (hand-built results, pre-power-3c snapshots) and consumers
    /// fall back to a dense derivation. Recomputed after any post-layout
    /// entity-reordering splice (see `wasm-bindings::improve_region_streaming`)
    /// in [`LayoutResult::wire_mode`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_wires: Option<Vec<(u32, u32)>>,
    /// How this layout's `power_wires` were generated (RFC-045): the layout
    /// records its own wiring policy so post-layout recomputes honor it —
    /// without this, an improve-region pass on a `Tree` layout would
    /// silently re-densify it. `Dense`-default, skipped in serde when
    /// default (old snapshots deserialize to `Dense`).
    #[serde(default, skip_serializing_if = "wire_mode_is_default")]
    pub wire_mode: crate::power_wires::WireMode,
    /// Belt stack size this layout was planned at (RFC-046, BS1). The
    /// layout records its own value so validators and post-layout
    /// recomputes rate belts at the capacity the planner assumed.
    /// **`0` and `1` both mean unstacked**: the derived `Default` (and
    /// hand-built/parsed results, which have no research context) yield
    /// `0`, and every consumer goes through the `common::*_stacked`
    /// helpers, which clamp to 1..=4. Serde skips ≤1 and defaults
    /// missing to 1, so pre-RFC snapshots deserialize unstacked.
    #[serde(default = "stacking_default", skip_serializing_if = "stacking_is_default")]
    pub stacking: u8,
}

fn wire_mode_is_default(m: &crate::power_wires::WireMode) -> bool {
    *m == crate::power_wires::WireMode::default()
}

fn stacking_default() -> u8 {
    1
}

fn stacking_is_default(s: &u8) -> bool {
    *s <= 1
}

/// One solid surplus stream consumed by a layout-synthesized voider
/// recycler bank. See `LayoutResult::voided_streams` and
/// `bus::voider::synthesize_voiders`.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoidedStream {
    pub item: String,
    /// Surplus rate (items/s) the bank destroys.
    pub rate: f64,
    /// Recycler machine count placed for this bank.
    pub machines: usize,
    /// The recycling recipe used (`"<item>-recycling"`).
    pub recipe: String,
}

/// One physical machine row as actually placed by `bus::placer::place_rows`,
/// pairing its `y` band with the exact `MachineSpec` sibling that produced
/// it. Follow-up fix for `docs/rfc-inserter-sizing.md`'s Phase 1 finding:
/// under `LayoutStrategy::PartitionedDecomposed`, `apply_partition_plan`
/// can split one recipe into multiple sibling `MachineSpec`s sharing a
/// recipe name but carrying different `count`/utilization (and therefore
/// different per-machine required rates). A recipe-name-keyed lookup
/// (`sr.machines.iter().map(|s| (s.recipe.as_str(), s))`, used by several
/// validators) collapses those siblings into whichever one iterated last.
/// `place_rows` lays each sibling's machines into its own non-overlapping
/// `y` band, so this ledger — one entry per physical row, mirroring
/// `bus::placer::RowSpan` trimmed to what validation needs — lets a
/// validator resolve "which spec did THIS machine actually come from" by
/// position instead of by name. Populated for every row unconditionally,
/// not just partitioned ones, so consumers that fall back to a recipe-name
/// lookup when no row matches see byte-identical behavior wherever
/// partitioning never occurred.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveRow {
    pub y_start: i32,
    pub y_end: i32,
    pub spec: MachineSpec,
}
