//! Lua scenario templating: ports `gen_harness_scenario.py` (the calibrated
//! prototype that measured 10.0/s on the tier-1 gear fixture — see
//! `docs/rfc-050-headless-sim-harness.md`'s "Empirical base") from Python
//! string templating to Rust, generalized to consume the REAL
//! `export_with_manifest` schema (`boundary_inputs`/`boundary_outputs` with
//! explicit `direction`/`entity` fields) instead of the pre-Phase-0 ad hoc
//! `feeds`/`drain` heuristic the prototype's manifest.json actually used.
//!
//! # Generalization from the calibrated south-only prototype
//!
//! The prototype hardcoded "westward jog, south-facing head" because every
//! fixture it ever ran (`iron-gear-wheel`@10/s, `electronic-circuit`@10/s,
//! the r120/r150 dogfood pair) only ever had south-facing `boundary_inputs`
//! (bus layouts always feed external inputs from the y=0 north edge,
//! flowing south into the row area — a structural invariant of
//! `crates/core/src/bus/placer.rs`'s row-based layout). This module
//! generalizes the SAME geometric mechanism to all four cardinal
//! directions via a small vector algebra (`outward`/`lateral` unit
//! vectors + a 90-degree rotation), rather than inventing a new mechanism:
//! every formula below was checked by hand against the literal prototype
//! source for the south case and reduces to the exact original numbers
//! (see `manifest::tests::rot90_matches_calibrated_drain_convention` and
//! this module's own golden-fragment tests). Non-south directions are
//! geometrically faithful but UNCALIBRATED — nothing has measured them
//! against a live server yet (`Manifest::has_uncalibrated_direction`
//! flags this for the report). Fluid boundaries used to carry the same
//! kind of flag, but that note went stale once #373 fixed the defect it
//! was warning about and later fixtures exercised the path clean — see
//! `Manifest::has_fluid_boundary` and #537.
//!
//! # Feed vs. drain pickup-side asymmetry
//!
//! Feed inserters move CHEST -> BELT (refilling); drain inserters move
//! BELT -> CHEST (draining). Since Factorio inserters read `direction` as
//! their PICKUP side, the two rigs use opposite axes for their
//! flanking-inserter geometry:
//! - feed: flanking axis = the head's own `direction` vector (the
//!   "into-layout" axis); pickup points toward the chest (same sign as
//!   the chest's own offset).
//! - drain: flanking axis = `rot90(direction)` (lateral to the exit
//!   belt's flow); pickup points toward the belt (opposite sign from the
//!   chest's offset).
//!
//! Getting this backwards was the exact shape of bug the RFC's Motivation
//! section describes for the export path; ported very deliberately here,
//! checked tile-by-tile against `gen_harness_scenario.py`'s literal
//! south-case numbers in this module's tests.

use crate::manifest::{rot90, BoundaryRecord, Manifest};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Baseline warmup before stability windows start (ticks). Chosen to match
/// `gen_harness_scenario.py`'s own `ev.tick == 3600` early-checkpoint
/// constant — the one number in the prototype that reads like a
/// deliberately chosen "warmup is roughly done" marker rather than an
/// arbitrary end-of-run value. Not given a precise number by the RFC text
/// ("base + 2x(W+H)x32 ticks"); this is the resolved base constant.
pub const BASE_WARMUP_TICKS: u32 = 3600;

/// Dim-scaling factor from the RFC design section 6: "base + 2x(W+H)x32".
const DIM_WARMUP_FACTOR: u32 = 32;

/// Floor on the stability-check window so tiny/fast fixtures don't judge
/// stability from a handful of ticks.
const MIN_WINDOW_TICKS: u32 = 600;

/// Expected item count per stability window (RFC: "windows sized so
/// expected items >= ~300"). This is a floor on the ACHIEVED sample, not
/// on a sample predicted from the planned rate — see `WINDOW_TICK_CAP_
/// FACTOR` and the item-driven checkpoint loop in the generated Lua.
pub const WINDOW_ITEM_FLOOR: f64 = 300.0;

/// Hard cap on how long one window may stay open waiting for
/// `WINDOW_ITEM_FLOOR` items, as a multiple of the planned-rate window
/// length.
///
/// Windows used to be sized purely from the PLANNED rate and closed on a
/// fixed tick count, so a factory at 40% of plan got 40% of the intended
/// sample and the 2% agreement test became unreachable: **the worse a
/// factory performed, the less measurable it became**, failing closed to
/// NO DATA (#454). Windows now close on accumulated items instead, and
/// this cap bounds the run for a factory so slow (below 1/4 of plan) that
/// waiting for a full sample would blow the tick budget. A window closed
/// by the cap is reported as short-sampled rather than silently folded
/// into a verdict.
const WINDOW_TICK_CAP_FACTOR: u32 = 4;

/// Consecutive windows that must all agree before the run is called
/// converged.
///
/// This was 2 — "the last step was small" — which a **decelerating ramp
/// always eventually passes**, at a point systematically short of its
/// asymptote. chem5 (a registered PASS) certified convergence on
/// 4.62 -> 4.92 -> 5.00/s: monotone, still climbing, and the final +1.6%
/// step slipped under the 2% tolerance. The trailing window then got
/// reported as "5.00/s EXACT at plan" when the whole measured span
/// averaged 4.84/s.
///
/// Three windows compared as a group (widest-vs-narrowest, not
/// pairwise) rejects that: a ramp accumulates its steps across the span
/// (+8.3% for chem5) while genuine noise cancels. This is also the
/// answer to the second question in #454 — convergence `true` at 160k
/// and `false` at 480k on identical geometry was one ramp measured at
/// two points, not an unstable factory.
pub const STABILITY_WINDOWS: u32 = 3;

/// Checkpoints needed before the convergence test can run at all: one
/// opening checkpoint plus `STABILITY_WINDOWS` closes. A tick ceiling
/// that cannot fit them makes convergence *structurally* impossible —
/// which is what `with_warmup` used to produce, re-flooring the ceiling
/// at `warmup + ONE window`. Every `--warmup` override past the default
/// ceiling therefore reported `converged: false` as a property of the
/// harness, not of the factory (#454).
const MIN_CHECKPOINTS: u32 = STABILITY_WINDOWS + 1;

/// Two consecutive stability windows must agree within this fraction to
/// call the run converged (RFC: "loop-until-stable ... within tolerance" —
/// not given a specific number; resolved to match KC2's own 2% scale).
const STABILITY_TOLERANCE: f64 = 0.02;

/// `base + 2*(W+H)*32`, rounded up to a multiple of 60 (the tick handler's
/// own cadence — a non-multiple-of-60 ceiling could never be hit exactly
/// by an `on_nth_tick(60, ...)` check).
pub fn default_warmup_ticks(width: i32, height: i32) -> u32 {
    let raw =
        BASE_WARMUP_TICKS + 2 * (width.max(0) as u32 + height.max(0) as u32) * DIM_WARMUP_FACTOR;
    round_up_60(raw)
}

/// Nominal window length: how long `WINDOW_ITEM_FLOOR` items take **at
/// plan**, floored at `MIN_WINDOW_TICKS`, rounded to a multiple of 60.
///
/// This is now only the *nominal* size — the scale from which the cap is
/// derived and the unit the tick budget is expressed in. The window a run
/// actually measures over closes on accumulated items (#454), so a
/// factory below plan gets a longer window rather than a thinner sample.
pub fn default_window_ticks(target_rate: f64) -> u32 {
    if target_rate <= 0.0 {
        return round_up_60(MIN_WINDOW_TICKS);
    }
    let seconds = WINDOW_ITEM_FLOOR / target_rate;
    let ticks = (seconds * 60.0).ceil() as u32;
    round_up_60(ticks.max(MIN_WINDOW_TICKS))
}

/// Longest a single window may stay open before closing short-sampled.
pub fn window_tick_cap(window_ticks: u32) -> u32 {
    window_ticks * WINDOW_TICK_CAP_FACTOR
}

/// Smallest ceiling that can still fit `MIN_CHECKPOINTS` checkpoints —
/// one opening checkpoint at warmup plus `STABILITY_WINDOWS` worst-case
/// (cap-length) windows. Below this the convergence test never runs and
/// the verdict says nothing about the factory (#454).
fn viable_end_tick(warmup: u32, window: u32) -> u32 {
    round_up_60(warmup + window_tick_cap(window) * STABILITY_WINDOWS)
}

fn round_up_60(t: u32) -> u32 {
    t.div_ceil(60) * 60
}

/// Floor on the derived wall-clock timeout — the previous fixed default,
/// kept so small fixtures are unaffected.
const MIN_TIMEOUT_SECS: u64 = 900;

/// Fixed allowance for server start, blueprint import and ghost revival,
/// none of which scale with the tick budget (48k ghosts is the worst
/// case measured).
const TIMEOUT_SETUP_SECS: u64 = 180;

/// How far short of the requested `game.speed` a run is allowed to fall
/// before the wall-clock net fires. Factorio's tick loop is effectively
/// single-threaded, so a big factory or a loaded box simply runs slower
/// than asked: a 48k-entity fixture was measured at ~290 ticks/s against
/// a requested 960 (`--speed 16`), i.e. ~3.3x short. 4x covers that with
/// margin.
const TIMEOUT_SLACK_FACTOR: f64 = 4.0;

/// Wall-clock safety net for a run, derived from its own tick budget.
///
/// This is a net for a hung or crashed server, NOT a second tick budget:
/// the scenario force-finalizes itself at `end_tick`. A timeout that
/// lands BEFORE the ceiling silently converts a non-converged report
/// (useful — it carries the rates and the drift) into no report at all,
/// because `launch_and_wait` returns `Err` and the server is killed
/// before anything is written. It would land there precisely on the
/// slow, underperforming runs this harness exists to measure, so the
/// default must scale with the budget rather than sit at a constant
/// (#464 review).
pub fn default_timeout_secs(end_tick: u32, speed: u32) -> u64 {
    let expected_secs = end_tick as f64 / (60.0 * speed.max(1) as f64);
    let derived = (expected_secs * TIMEOUT_SLACK_FACTOR).ceil() as u64 + TIMEOUT_SETUP_SECS;
    derived.max(MIN_TIMEOUT_SECS)
}

/// Parameters controlling one `run` invocation, independent of the
/// manifest itself.
#[derive(Debug, Clone)]
pub struct RunParams {
    /// Ceiling tick (`--ticks`; the ONE thing that can force-finalize a
    /// run that never stabilizes — KC4's wall-clock budget lives here).
    pub end_tick: u32,
    /// `game.speed` (RFC: live-server pacing is 60 UPS unless raised).
    pub speed: u32,
    pub warmup_ticks: u32,
    pub window_ticks: u32,
    pub scenario_name: String,
    /// `serve` only: reveal the map and speed the operator up on join.
    /// Never set for measurement runs — it changes force bonuses, and a
    /// measurement must run in the world the fixture declares.
    pub operator_qol: bool,
    /// `run --timeseries`: stream the per-window machine/item time-series to
    /// `script-output/timeseries.csv` LIVE during a measurement run, not just
    /// into the JSON report at finalize. Same format as `serve`'s streaming
    /// CSV (RFC-050 Phase 3 #537). Unlike `operator_qol` this does NOT change
    /// force bonuses or reveal the map — it is measurement-safe, so a
    /// long/grinding run can be watched and scored live (is it ramping toward
    /// plan, or flat-zero and dead?) without waiting for finalize.
    pub write_timeseries: bool,
}

impl RunParams {
    /// Build defaults from the manifest's own dims + target rate, leaving
    /// `end_tick`/`speed`/`scenario_name` for the caller (CLI flags or
    /// generated identity) to fill in.
    pub fn defaults_for(
        manifest: &Manifest,
        scenario_name: String,
        speed: u32,
        end_tick: Option<u32>,
    ) -> RunParams {
        let warmup = default_warmup_ticks(manifest.dims[0], manifest.dims[1]);
        let target_rate = manifest.targets.first().map(|t| t.rate).unwrap_or(1.0);
        let window = default_window_ticks(target_rate);
        // Ceiling must clear warmup plus enough worst-case windows to run
        // the convergence test with room to spare — a factory typically
        // needs several windows before the trailing group goes flat, and
        // at plan each window closes at its nominal length rather than
        // the cap, so this budget is rarely spent. An explicit `--ticks`
        // is still floored at viability, since a ceiling that cannot fit
        // `MIN_CHECKPOINTS` reports non-convergence by construction.
        let default_ceiling = round_up_60(warmup + window_tick_cap(window) * MIN_CHECKPOINTS * 2);
        RunParams {
            end_tick: end_tick
                .unwrap_or(default_ceiling)
                .max(viable_end_tick(warmup, window)),
            speed,
            warmup_ticks: warmup,
            window_ticks: window,
            scenario_name,
            operator_qol: false,
            write_timeseries: false,
        }
    }

    /// Enable the human-inspection conveniences (`serve`).
    pub fn with_operator_qol(mut self) -> RunParams {
        self.operator_qol = true;
        self
    }

    /// Enable LIVE per-window CSV streaming on a measurement run
    /// (`run --timeseries`). Independent of `operator_qol`.
    pub fn with_timeseries(mut self) -> RunParams {
        self.write_timeseries = true;
        self
    }

    /// Override the dim-scaled warmup (`--warmup`). The 2% stability
    /// windows cannot distinguish a slow buffer-fill drift from real
    /// convergence — deep-chain fixtures "converge" while trunk and tap
    /// buffers are still filling — so steady-state probes need
    /// measurement to start long after that transient. Rounded up to the
    /// tick handler's 60-tick cadence.
    ///
    /// The ceiling is re-floored to fit the whole convergence test, not
    /// (as it was) a single window: `+ window` left room for exactly ONE
    /// checkpoint where the test needs three, so every warmup override
    /// past the default ceiling was guaranteed to report
    /// `converged: false` regardless of the factory (#454).
    pub fn with_warmup(mut self, warmup: u32) -> RunParams {
        self.warmup_ticks = round_up_60(warmup);
        self.end_tick = self
            .end_tick
            .max(viable_end_tick(self.warmup_ticks, self.window_ticks));
        self
    }
}

/// A world-space cardinal vector, used only inside this module's Lua
/// codegen for `outward`/`lateral`/`into` axis arithmetic.
type Vec2 = (i32, i32);

fn neg((x, y): Vec2) -> Vec2 {
    (-x, -y)
}

/// Emit the shared `add_feed`/`add_drain` Lua functions plus the module-
/// proxy fulfillment helper. Shared (not unrolled per-boundary) so the
/// generated script stays small and each call site is a single line —
/// easier to golden-test and to eyeball when debugging a specific feed.
fn write_shared_functions(out: &mut String) {
    out.push_str(
        r#"
-- FEED rig: chest -> stack-inserter -> belt, staggered on a jog OUTWARD
-- from the boundary head (RFC-050 "6 legendary stack-inserter banks on a
-- westward jog per input head"), generalized from the calibrated
-- south-facing case via outward/lateral vector rotation (see scenario.rs
-- module docs for the derivation). `ox,oy` = outward unit vector (away
-- from the layout); `lx,ly` = lateral unit vector (rot90 of the belt's
-- own into-layout direction).
local function add_feed(s, force, head_x, head_y, ox, oy, lx, ly, depth, item, belt_name)
  local into_x, into_y = -ox, -oy
  local neg_lx, neg_ly = -lx, -ly
  local corner_x, corner_y = head_x + ox * depth, head_y + oy * depth
  -- outward extension: belts from the corner back down to the head,
  -- continuing the head's own into-layout flow direction.
  for t = 1, depth do
    s.create_entity{name = belt_name, position = {head_x + ox * t, head_y + oy * t},
                    direction = dir_from_vec(into_x, into_y), force = force}
  end
  -- lateral jog run: 12 tiles out from the corner, flowing back into it.
  for k = 1, 12 do
    s.create_entity{name = belt_name, position = {corner_x + neg_lx * k, corner_y + neg_ly * k},
                    direction = dir_from_vec(lx, ly), force = force}
  end
  -- 6-inserter bank on the 3 farthest jog tiles.
  local chests = {}
  for k = 10, 12 do
    local bx, by = corner_x + neg_lx * k, corner_y + neg_ly * k
    for _, side in ipairs({-1, 1}) do
      local cx, cy = bx + into_x * 2 * side, by + into_y * 2 * side
      local ix, iy = bx + into_x * side, by + into_y * side
      local c = s.create_entity{name = "steel-chest", position = {cx, cy}, force = force}
      s.create_entity{name = "stack-inserter", position = {ix, iy},
        direction = dir_from_vec(into_x * side, into_y * side), force = force, quality = "legendary"}
      table.insert(chests, c)
    end
  end
  -- local power island, further out along the jog run.
  local subx, suby = corner_x + neg_lx * 15, corner_y + neg_ly * 15
  local eeix, eeiy = corner_x + neg_lx * 18, corner_y + neg_ly * 18
  s.create_entity{name = "substation", position = {subx, suby}, force = force, quality = "legendary"}
  local eei = s.create_entity{name = "electric-energy-interface", position = {eeix, eeiy}, force = force}
  eei.electric_buffer_size = 1e13
  table.insert(storage.eeis, eei)
  storage.feeds[item] = storage.feeds[item] or {}
  table.insert(storage.feeds[item], {chests = chests, fed = 0})
end

-- Fluid FEED: infinity-pipe at the far end of an ISOLATED ug-pipe run.
-- A bare infinity-pipe on the tile beyond the port merges with the
-- neighboring feed's when two fluid ports are adjacent (pu3: crude@38
-- + water@39 became ONE network; crude won and the acid chain never
-- ran — K60-3 forensics 2026-07-31). ug-pipe bodies have no lateral
-- connections, so runs in adjacent columns stay isolated; the per-feed
-- `dist` stagger keeps the surface caps (the only merge-capable tiles)
-- non-adjacent. Runtime pipe-to-ground direction = the surface-opening
-- side (verified from a pasted port's sim dump: port dir=north, opening
-- north). Rate remains UNCALIBRATED (RFC-050), but no longer cross-feeds.
local function add_fluid_feed(s, force, head_x, head_y, ox, oy, dist, item)
  local ok, err = pcall(function()
    s.create_entity{name = "pipe-to-ground", position = {head_x + ox, head_y + oy},
                    direction = dir_from_vec(-ox, -oy), force = force}
    s.create_entity{name = "pipe-to-ground",
                    position = {head_x + ox * (1 + dist), head_y + oy * (1 + dist)},
                    direction = dir_from_vec(ox, oy), force = force}
    local ip = s.create_entity{name = "infinity-pipe",
                    position = {head_x + ox * (2 + dist), head_y + oy * (2 + dist)}, force = force}
    ip.set_infinity_pipe_filter{name = item, percentage = 1, mode = "exactly"}
  end)
  if not ok then storage.fluid_errors[item .. "@feed"] = tostring(err) end
end

-- DRAIN rig: extension belt (always express/blue -- "drain tier >= belt
-- tier or backpressure falsifies the run") + flanking stack-inserter bank
-- picking FROM the belt (pickup = belt side, per the artifact-boundary
-- inserter-direction lesson).
local function add_drain(s, force, exit_x, exit_y, fx, fy, lx, ly, ext_len, item)
  -- RFC-062 Phase 3 finding: on the EC+AC final-gate fixture, this rig's
  -- entire belt extension silently failed to place (create_entity
  -- returned nil for every one of 13 tiles) while its chest/inserter
  -- bank built exactly per formula -- the EXACT signature of the #345/
  -- PU@4 chunk-truncation class this codebase already has a documented
  -- precedent for ("the old fixed radius... silently truncated any
  -- fixture wider than ~768 tiles: build_blueprint creates no ghosts on
  -- ungenerated chunks... dead feed rigs, NO DATA", see the `gen_radius`
  -- comment above). The global `request_to_generate_chunks({0,0},
  -- gen_radius)` call at setup sizes its radius from the LAYOUT's own
  -- half-span, assuming every rig sits within that -- but RFC-062 lets a
  -- SECOND target's drain rig land at whichever edge that target's own
  -- physical exit happens to be on, which the single global radius may
  -- not reliably reach for every possible exit position and rig index
  -- (ext_len grows with boundary_outputs index, extending further out
  -- each time). Explicitly chunk-generate this specific rig's own
  -- footprint before placing anything in it -- redundant (cheap,
  -- idempotent) when the global call already covers it, a guarantee
  -- when it doesn't.
  s.request_to_generate_chunks({exit_x + fx * ext_len, exit_y + fy * ext_len}, 3)
  s.force_generate_chunk_requests()

  -- `create_entity` can still fail here (returns nil) for reasons beyond
  -- chunk generation, with NO exception and NO prior error handling on
  -- this loop -- unlike `add_fluid_void`/`add_fluid_feed`, which both
  -- wrap their placement in `pcall` and record a failure. Report any
  -- remaining failure loudly instead of silently, matching this
  -- project's own rule that a check going quiet is not evidence of
  -- success (docs/validator-reporting.md).
  for t = 1, ext_len do
    local placed = s.create_entity{name = "express-transport-belt", position = {exit_x + fx * t, exit_y + fy * t},
                    direction = dir_from_vec(fx, fy), force = force}
    if not placed then
      table.insert(storage.kit_errors, "drain rig for '" .. item .. "' at (" .. exit_x .. "," .. exit_y
        .. "): belt placement failed at extension tile " .. t .. "/" .. ext_len
        .. " (" .. (exit_x + fx * t) .. "," .. (exit_y + fy * t) .. ")")
    end
  end
  -- Rig capacity must comfortably exceed the drained rate: SIX
  -- pickup inserters at a low declared world pull ~13.8/s from the
  -- belt — EXACTLY the plateau chain-ec15-d1 measured, and the
  -- d-sweep's level-shape tracks the rig's own bulk bonus (#383
  -- re-attribution, 2026-07-24). Twelve positions doubles the bank;
  -- pickup-side headroom is cheap and never inflates a measurement.
  local chests = {}
  for t = ext_len - 8, ext_len do
    local bx, by = exit_x + fx * t, exit_y + fy * t
    for _, side in ipairs({-1, 1}) do
      local cx, cy = bx + lx * 2 * side, by + ly * 2 * side
      local ix, iy = bx + lx * side, by + ly * side
      local c = s.create_entity{name = "steel-chest", position = {cx, cy}, force = force}
      s.create_entity{name = "stack-inserter", position = {ix, iy},
        direction = dir_from_vec(-lx * side, -ly * side), force = force, quality = "legendary"}
      table.insert(chests, c)
    end
  end
  local subx, suby = exit_x + lx * 4 + fx, exit_y + ly * 4 + fy
  local eeix, eeiy = exit_x + lx * 7 + fx, exit_y + ly * 7 + fy
  s.create_entity{name = "substation", position = {subx, suby}, force = force, quality = "legendary"}
  local eei = s.create_entity{name = "electric-energy-interface", position = {eeix, eeiy}, force = force}
  eei.electric_buffer_size = 1e13
  table.insert(storage.eeis, eei)
  storage.drains[item] = storage.drains[item] or {}
  for _, c in ipairs(chests) do table.insert(storage.drains[item], c) end
end

-- Fluid surplus VOID (RFC: "infinity-pipe voids at every fluid surplus
-- exit -- undrained surplus dead-ends fill and stall AOP-class fixtures").
-- UNCALIBRATED. Pipes have no meaningful orientation for connectivity, so
-- this just tries the 4 adjacent tiles and takes the first placeable one.
local function add_fluid_void(s, force, x, y, item)
  local ok, err = pcall(function()
    for _, d in ipairs({{0, -1}, {0, 1}, {1, 0}, {-1, 0}}) do
      local px, py = x + d[1], y + d[2]
      if s.can_place_entity{name = "infinity-pipe", position = {px, py}} then
        local ip = s.create_entity{name = "infinity-pipe", position = {px, py}, force = force}
        ip.set_infinity_pipe_filter{name = item, percentage = 0, mode = "at-most"}
        return
      end
    end
    error("no placeable tile adjacent to surplus exit (" .. x .. "," .. y .. ")")
  end)
  if not ok then storage.fluid_errors[item .. "@void"] = tostring(err) end
end

-- Module proxies (RFC-050: "insert into get_module_inventory(), destroy
-- proxy, effect live" -- verified live by the kit-probe). Generic over
-- whatever modules the factory actually requested via `proxy.insert_plan`
-- (grouped per distinct (name, quality), matching the 2.0 insert-plan
-- shape `export_with_manifest`'s sibling `blueprint::export` emits).
local function fulfill_module_proxies(s)
  local n = 0
  for _, proxy in pairs(s.find_entities_filtered{type = "item-request-proxy"}) do
    if proxy.valid then
      local target = proxy.proxy_target
      if target and target.valid then
        local inv = target.get_module_inventory()
        if inv then
          for _, entry in pairs(proxy.insert_plan) do
            local count = 0
            for _, pos in pairs(entry.items.in_inventory) do count = count + (pos.count or 1) end
            if count > 0 then
              inv.insert{name = entry.id.name, count = count, quality = entry.id.quality}
            end
          end
        end
      end
      proxy.destroy()
      n = n + 1
    end
  end
  return n
end
"#,
    );
}

/// Assigns each solid (non-fluid) feed record a small "slot" index
/// (0, 1, 2, ...), grouped by direction and ordered along the LATERAL
/// axis — NOT by raw manifest order. `feed_call` turns a record's slot
/// into its `add_feed` depth stagger (`4 + 6*slot`).
///
/// #363's second live datum: `add_feed`'s westward jog belt row runs
/// `lateral*[1..12]` tiles out from its own corner (chest bank on
/// `[10..12]`), and a rig's own head->corner "outward column" collides
/// with a same-direction NEIGHBOR's jog row whenever that neighbor sits
/// within the jog's lateral reach and has depth >= the crossing rig's
/// jog height. Depth used to be `4 + 6*idx` on raw manifest order, so a
/// manifest that didn't happen to list heads in the jog's own travel
/// direction (west->east for south-facing heads, since
/// `lateral = rot90(south) = east`) could put a deep rig's jog row
/// directly over a shallower neighbor's column — `create_entity` in
/// script mode stacks silently there instead of failing loudly, so the
/// shadowed lanes just never fed (issue: "only the first-ordered rig per
/// group worked, at both 1-tile and 4-tile column spacing").
///
/// Sorting each direction group ascending along `lateral` before
/// slotting means every neighbor further "upstream" along the jog's own
/// travel direction always gets a strictly SMALLER depth than rigs
/// further along it, so no column can ever reach a jog row's height —
/// the issue's proven consumer-side workaround ("order boundary_inputs
/// west->east") becomes the harness's own default instead of a caller
/// obligation. Fluid records don't use this lateral jog mechanism (they
/// place a single adjacent infinity-pipe tile via `add_fluid_feed`,
/// staggered by depth instead) — but they DO consume a slot, ordered
/// first within each direction group, ahead of items (see the depth
/// layout below).
fn feed_slots(records: &[BoundaryRecord]) -> Vec<i32> {
    // ONE ladder per direction, fluids FIRST (PR #515 review finding: a
    // separate fluid counter staggered fluids only against each other, so
    // a fluid ug-run could land exactly on an item rig's jog row and
    // stack silently). Fluids take slots 0..f-1 — their occupied band
    // (out-tiles 1..=2+dist, dist = 2+2*slot, max 2+2f) sits strictly
    // below the shallowest item band ([2+6f, 6+6f] for item slot f), so
    // no item jog row or chest bank can ever reach a fluid column's
    // tiles. Items keep their lateral-sorted relative order (the #363
    // invariant), just offset by f.
    let mut by_dir: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
    for (i, rec) in records.iter().enumerate() {
        by_dir.entry(rec.direction).or_default().push(i);
    }
    let mut slots = vec![0i32; records.len()];
    for idxs in by_dir.into_values() {
        let lateral = rot90(records[idxs[0]].direction().vector());
        let mut idxs = idxs;
        // Fluids-first, then lateral position within each class.
        idxs.sort_by_key(|&i| {
            (
                !records[i].is_fluid,
                records[i].x * lateral.0 + records[i].y * lateral.1,
            )
        });
        for (slot, i) in idxs.into_iter().enumerate() {
            slots[i] = slot as i32;
        }
    }
    slots
}

/// One `add_feed`/`add_fluid_feed` call site, with all the outward/lateral
/// vector arithmetic resolved at Rust-codegen time (only the runtime-only
/// `storage.offx`/`storage.offy` piece stays symbolic in the emitted Lua).
fn feed_call(out: &mut String, idx: usize, slot: i32, rec: &BoundaryRecord) {
    let into = rec.direction().vector();
    let outward = neg(into);
    let lateral = rot90(into);
    // Depth stagger must exceed the bank's ±2 lateral chest offset, or
    // adjacent rigs' chest rows land on the same tile — create_entity in
    // script mode stacks entities silently, and a shared bank tile
    // cross-feeds ores (two overlapping chests at one tile poisoned the
    // logistic fixture's iron system with copper plates; #357 forensics
    // 2026-07-22). 4+4*slot put rig 0's chest row (depth+2) exactly on
    // rig 1's (depth-2); 6 per step keeps every rig's occupied band
    // [depth-2, depth+2] disjoint. `slot` (not raw manifest order — see
    // `feed_slots`) also keeps same-direction rigs' jog rows from
    // crossing a shallower neighbor's outward column (#363). The
    // Lua-side overlap audit backstops geometries this spacing can't
    // save.
    let depth = 4 + 6 * slot;
    let _ = writeln!(
        out,
        "  do\n    local head_x, head_y = {x} - LX0 + storage.offx, {y} - LY0 + storage.offy",
        x = rec.x,
        y = rec.y,
    );
    if rec.is_fluid {
        // Fluids hold the FRONT of the direction's ladder (see
        // `feed_slots`), so `slot` here is the fluid's rank 0..f-1 and
        // the staggered run length keeps every surface cap ≥2 tiles from
        // its neighbors while staying below every item band. The ug span
        // (gap = dist-1 = 1+2*slot) hits the game's 9-tile limit at
        // slot 4 — six or more fluid feeds on ONE boundary side would
        // need chained hops, which no manifest has ever required; fail
        // loudly rather than fabricate a rate.
        assert!(
            slot <= 4,
            "more than 5 fluid boundary feeds on one side (slot {slot} for {}): \
             the ug-run stagger cannot exceed the game's 9-tile span; extend \
             add_fluid_feed with chained hops before running this fixture",
            rec.item
        );
        let dist = 2 + 2 * slot;
        let _ = writeln!(
            out,
            "    add_fluid_feed(s, force, head_x, head_y, {ox}, {oy}, {dist}, \"{item}\")",
            ox = outward.0,
            oy = outward.1,
            item = rec.item,
        );
    } else {
        // A boundary head can be a splitter (the engine's record names
        // the entity at the head tile); the rig's own belts must be the
        // matching SURFACE BELT tier — building a jog out of 2-tile
        // splitter entities silently delivers nothing (#345: r150's
        // third copper spine was fed by a dead splitter-bodied rig).
        let belt = match rec.entity.as_str() {
            "splitter" => "transport-belt",
            "fast-splitter" => "fast-transport-belt",
            "express-splitter" => "express-transport-belt",
            "turbo-splitter" => "turbo-transport-belt",
            other => other,
        };
        let _ = writeln!(
            out,
            "    add_feed(s, force, head_x, head_y, {ox}, {oy}, {lx}, {ly}, {depth}, \"{item}\", \"{belt}\")",
            ox = outward.0,
            oy = outward.1,
            lx = lateral.0,
            ly = lateral.1,
            depth = depth,
            item = rec.item,
            belt = belt,
        );
    }
    let _ = writeln!(
        out,
        "  end -- feed[{idx}] {} at ({},{})",
        rec.item, rec.x, rec.y
    );
}

fn drain_call(out: &mut String, idx: usize, rec: &BoundaryRecord) {
    let flow = rec.direction().vector();
    let lateral = rot90(flow);
    // Base 11 (was 5): the widened 9-position drain bank needs
    // ext_len-8 >= 3 so every chest/inserter sits outside the layout.
    let ext_len = 11 + 2 * (idx as i32);
    let _ = writeln!(
        out,
        "  do\n    local exit_x, exit_y = {x} - LX0 + storage.offx, {y} - LY0 + storage.offy",
        x = rec.x,
        y = rec.y,
    );
    let _ = writeln!(
        out,
        "    add_drain(s, force, exit_x, exit_y, {fx}, {fy}, {lx}, {ly}, {ext}, \"{item}\")",
        fx = flow.0,
        fy = flow.1,
        lx = lateral.0,
        ly = lateral.1,
        ext = ext_len,
        item = rec.item,
    );
    let _ = writeln!(
        out,
        "  end -- drain[{idx}] {} at ({},{})",
        rec.item, rec.x, rec.y
    );
}

/// Build the full `control.lua` for one measurement run.
pub fn build_control_lua(manifest: &Manifest, bp: &str, params: &RunParams) -> String {
    let mut out = String::new();
    // RFC-062 Phase 3: every requested target, not just the first — the
    // per-item checkpoint series below (`checkpoint_items()`) tracks
    // produced+delivered for ALL of these, so every target gets an honest
    // delivered-rate verdict (report.rs), generalizing the 2026-07-24
    // first-target-only fix (#537 dossier) the same way that fix
    // generalized the single-scalar TARGET before it.
    let target_items: Vec<&str> = manifest.targets.iter().map(|t| t.item.as_str()).collect();

    let _ = writeln!(
        out,
        "-- Generated by spaghettio-sim (RFC-050). DO NOT EDIT."
    );
    let _ = writeln!(out, "-- label: {}", manifest.label);
    let _ = writeln!(out, "local BP = \"{bp}\"");
    {
        let items: Vec<String> = target_items.iter().map(|it| format!("\"{it}\"")).collect();
        let _ = writeln!(out, "local TARGETS = {{{}}}", items.join(", "));
    }
    // The FIRST target still drives window-closing/convergence timing
    // (unchanged from before this phase) — a run's length and stability
    // gating stay exactly what they were for every existing single-target
    // manifest (TARGETS[1] == the old scalar TARGET always). Every
    // target's own produced/delivered gets recorded at each window close
    // via `checkpoint_items()` below, not just this one.
    let _ = writeln!(out, "local TARGET = TARGETS[1] or \"\"");
    let _ = writeln!(out, "local END_TICK = {}", params.end_tick);
    let _ = writeln!(out, "local WARMUP_TICKS = {}", params.warmup_ticks);
    let _ = writeln!(out, "local WINDOW_TICKS = {}", params.window_ticks);
    let _ = writeln!(out, "local WINDOW_ITEM_FLOOR = {WINDOW_ITEM_FLOOR}");
    let _ = writeln!(out, "local WINDOW_MIN_TICKS = {MIN_WINDOW_TICKS}");
    let _ = writeln!(
        out,
        "local WINDOW_TICK_CAP = {}",
        window_tick_cap(params.window_ticks)
    );
    let _ = writeln!(out, "local STABILITY_TOL = {STABILITY_TOLERANCE}");
    let _ = writeln!(out, "local STABILITY_WINDOWS = {STABILITY_WINDOWS}");
    let _ = writeln!(
        out,
        "local LX0, LY0 = {}, {}",
        manifest.bbox_min[0], manifest.bbox_min[1]
    );
    let _ = writeln!(
        out,
        "local DIMS_X, DIMS_Y = {}, {}",
        manifest.dims[0], manifest.dims[1]
    );
    let _ = writeln!(
        out,
        "local INSERTER_CAPACITY = {}",
        manifest.inserter_capacity
    );
    let _ = writeln!(out, "local STACKING = {}", manifest.stacking);
    {
        let items: Vec<String> = manifest
            .planned_rates
            .keys()
            .map(|k| format!("\"{k}\""))
            .collect();
        let _ = writeln!(out, "local PLANNED_ITEMS = {{{}}}", items.join(", "));
    }
    // Per-machine/per-item time-series (#537 motivation): a rate-vs-time
    // series distinguishes "feed never arrived" (flat zero from tick 0)
    // from "buffer-fill mirage then jam" (ramp, then decay) at a glance —
    // neither shape is visible from the final aggregate alone. `run`
    // collects it into `storage.timeseries` for the JSON report
    // regardless of mode; CSV appending to script-output is `serve`-only
    // (`operator_qol`), so a human watching live gets a machine-readable
    // record on disk without a measurement run paying the extra file I/O.
    let _ = writeln!(
        out,
        "local WRITE_TIMESERIES_CSV = {}",
        params.operator_qol || params.write_timeseries
    );
    let _ = writeln!(out, "local TIMESERIES_CSV_FILE = \"timeseries.csv\"");

    out.push_str(
        r#"
local function dir_from_vec(dx, dy)
  if dx == 0 and dy == -1 then return defines.direction.north end
  if dx == 1 and dy == 0 then return defines.direction.east end
  if dx == 0 and dy == 1 then return defines.direction.south end
  if dx == -1 and dy == 0 then return defines.direction.west end
  error("non-cardinal vector (" .. dx .. "," .. dy .. ")")
end
"#,
    );

    write_shared_functions(&mut out);

    out.push_str(
        r#"
-- Pairing support: a joining player spawns on nauvis, but the paste
-- lives on the "lab" surface — teleport them there, centered on the
-- layout. Observation only; the measurement never reads player state.
script.on_event(defines.events.on_player_joined_game, function(ev)
  local p = game.get_player(ev.player_index)
  local s = game.get_surface("lab")
  if p and s then
    p.teleport({0, 0}, s)
    game.print("[spaghettio-sim] teleported " .. p.name .. " to the lab surface")
  end
"#,
    );

    // `serve` only, and it must live INSIDE the handler above: a second
    // `script.on_event` for the same event REPLACES the first, which
    // would silently drop the lab-surface teleport and strand the player
    // on nauvis looking at empty terrain.
    if params.operator_qol {
        out.push_str(
            r#"  if p and s then
    -- Chart the whole paste plus a margin so the map opens usable rather
    -- than black. storage.offx/offy is where the blueprint actually
    -- landed; chart `s`, the lab surface, not wherever the player spawned.
    local m = 64
    p.force.chart(s, {{storage.offx - m, storage.offy - m},
                      {storage.offx + DIMS_X + m, storage.offy + DIMS_Y + m}})
    p.force.character_running_speed_modifier = 5
    p.force.character_reach_distance_bonus = 24
    p.force.character_build_distance_bonus = 24
    p.force.character_resource_reach_distance_bonus = 24
    p.print("spaghettio serve: map charted, 6x run speed, +24 reach.")
    p.print("  /editor          free camera, no character (best for inspecting)")
    p.print("  /c game.speed=4  fast-forward to steady state")
    p.print("layout spans (" .. storage.offx .. "," .. storage.offy .. ") to ("
            .. (storage.offx + DIMS_X) .. "," .. (storage.offy + DIMS_Y) .. ")")
  end
"#,
        );
    }

    out.push_str(
        r#"end)

script.on_init(function()
  storage.eeis, storage.feeds, storage.fed_total = {}, {}, {}
  storage.drains, storage.drained_total = {}, {}
  storage.samples, storage.checkpoints = {}, {}
  storage.fluid_errors = {}
  storage.kit_errors = {}
  storage.finalized = false
  storage.converged = false
  -- Time-series bookkeeping (#537): storage.machine_last_crafts /
  -- storage.item_last_produced hold each machine's/item's cumulative
  -- counter AS OF the previous checkpoint, so every window's delta is a
  -- true per-window value rather than a running total re-derived later.
  storage.timeseries = {}
  storage.machine_last_crafts, storage.item_last_produced = {}, {}
  if WRITE_TIMESERIES_CSV then
    helpers.write_file(TIMESERIES_CSV_FILE,
      "tick,kind,unit,name,x,y,crafts_delta,status,item,produced_delta\n", false)
  end
  game.speed = "#,
    );
    let _ = writeln!(out, "{}", params.speed);
    out.push_str(
        r#"  local force = game.forces.player
  force.research_all_technologies()
  -- Tech-state parity (#370): the engine models inserter hands at the
  -- fixture's inserter_capacity level; research_all grants bonus 7 and
  -- out-provisions that assumption, masking genuine L-level shortfalls
  -- (and making inserter-throughput warnings untestable — #352). Set
  -- the force bonuses directly to the level's values — the two fields
  -- reproduce all three I8b hand tables exactly (probe-verified
  -- 2026-07-22): non-bulk hand = 1 + stack_size_bonus, bulk hand =
  -- 1 + bulk_bonus, stack-inserter hand = 5 + bulk_bonus. Direct
  -- assignment beats un-researching capacity techs, which left
  -- non-bulk one step high (an unidentified tech grants +1). The
  -- realized bonuses are dumped into the result for verification.
  local NB_BONUS = {0, 0, 1, 1, 1, 1, 1, 3}
  local BULK_BONUS = {1, 2, 3, 4, 5, 7, 9, 11}
  force.inserter_stack_size_bonus = NB_BONUS[INSERTER_CAPACITY + 1]
  force.bulk_inserter_capacity_bonus = BULK_BONUS[INSERTER_CAPACITY + 1]
  -- Parity self-audit: read the fields back on the next tick's init
  -- path is not available here, so verify immediately — if the engine
  -- rejected or clamped the assignment, measured rates would be taken
  -- in the wrong hand world; that invalidates the run exactly like a
  -- compromised kit (review finding on #376: a verification channel
  -- nobody checks is not a verification channel).
  if force.inserter_stack_size_bonus ~= NB_BONUS[INSERTER_CAPACITY + 1]
     or force.bulk_inserter_capacity_bonus ~= BULK_BONUS[INSERTER_CAPACITY + 1] then
    table.insert(storage.kit_errors, "tech-state parity assignment did not take: nb="
      .. force.inserter_stack_size_bonus .. " bulk=" .. force.bulk_inserter_capacity_bonus
      .. " for level " .. INSERTER_CAPACITY)
  end
  -- Belt-stacking parity (option A, decided 2026-07-23 with user: the
  -- sim's world matches the fixture's declared axes — early-game
  -- layouts with low tech and no stacking are a real deployment
  -- target, and research_all let stack inserters create 4-stacks on
  -- belts declared stacking=1, inflating every stack belt-drop
  -- measurement; #385 forensics). Same direct-assignment pattern as
  -- inserter capacity: belt stack size = 1 + belt_stack_size_bonus.
  force.belt_stack_size_bonus = STACKING - 1
  if force.belt_stack_size_bonus ~= STACKING - 1 then
    table.insert(storage.kit_errors, "belt-stacking parity assignment did not take: bonus="
      .. force.belt_stack_size_bonus .. " for declared S=" .. STACKING)
  end
  local s = game.create_surface("lab")
  s.generate_with_lab_tiles = true
  -- The paste is CENTERED on {0,0}, so generated chunks must cover
  -- the layout's HALF-span plus rig margin in every direction. The
  -- old fixed radius (12 chunks = 384 tiles) silently truncated any
  -- fixture wider than ~768 tiles: build_blueprint creates no ghosts
  -- on ungenerated chunks, and the PU@4 chain (2704 tiles wide) lost
  -- 2/3 of its entities that way — dead feed rigs, NO DATA (#345
  -- adjacent, RFC-052 increment 2 forensics).
  local gen_radius = math.max(12, math.ceil((math.max(DIMS_X, DIMS_Y) / 2 + 64) / 32) + 1)
  s.request_to_generate_chunks({0, 0}, gen_radius)
  s.force_generate_chunk_requests()

  local inv = game.create_inventory(1)
  local stack = inv[1]
  stack.set_stack("blueprint")
  storage.import_rc = stack.import_stack(BP)
  local ghosts = stack.build_blueprint{surface = s, force = force,
    position = {0, 0}, build_mode = defines.build_mode.superforced}
  storage.ghosts, storage.revived = #ghosts, 0
  for _, g in pairs(ghosts) do
    if g.valid then
      local _, e = g.revive()
      if e then storage.revived = storage.revived + 1 end
    end
  end

  -- world offset: paste is CENTERED on the build position, so derive the
  -- layout->world translation from the revived bbox min, anchored to the
  -- manifest's own bbox_min (LX0, LY0).
  local minx, miny = math.huge, math.huge
  for _, e in pairs(s.find_entities_filtered{force = force}) do
    if e.type ~= "character" then
      local bb = e.bounding_box
      if bb.left_top.x < minx then minx = bb.left_top.x end
      if bb.left_top.y < miny then miny = bb.left_top.y end
    end
  end
  storage.offx, storage.offy = math.floor(minx + 0.5), math.floor(miny + 0.5)

  storage.proxies_fulfilled = fulfill_module_proxies(s)

  -- Power every factory pole network: one hidden-electric-energy-
  -- interface placed AT (overlapping) a representative pole's own
  -- position. hidden-EEI has a 0x0 collision box, so this ALWAYS
  -- succeeds regardless of how densely packed the surrounding tiles are,
  -- and 0-distance-from-a-real-pole guarantees the auto-wire connection
  -- lands in the right network.
  --
  -- LIVE FINDING (this harness, EC10@84x90 fixture): the west-of-pole
  -- can_place scan this replaced (ported verbatim from
  -- gen_harness_scenario.py, which only ever ran on the small, sparse
  -- gear10 fixture) found SOME empty tile in a big dense layout and
  -- reported success, but the placed substation/EEI pair wasn't
  -- necessarily within wire reach of anything real -- 60 machines came
  -- back `no_power`, 0 items measured. The RFC's own empirical base
  -- names the fix directly: "hidden-electric-energy-interface -- 0x0
  -- collision box, placeable AT a pole's position -- avoids the 2x2
  -- siting problem in dense layouts". Adopted here after the scan
  -- version failed live; the boundary-kit feed/drain rigs' OWN local
  -- power islands (regular EEI+substation, built in open space the rig
  -- constructs itself) are unaffected -- they measured correctly on the
  -- gear10 PASS run.
  local nets = {}
  for _, p in pairs(s.find_entities_filtered{type = "electric-pole"}) do
    local id = p.electric_network_id
    if id then nets[id] = p end
  end
  storage.net_count, storage.factory_eeis = 0, 0
  for _, pole in pairs(nets) do
    storage.net_count = storage.net_count + 1
    local eei = s.create_entity{name = "hidden-electric-energy-interface", position = pole.position, force = force}
    if eei then
      eei.electric_buffer_size = 1e13
      table.insert(storage.eeis, eei)
      storage.factory_eeis = storage.factory_eeis + 1
    end
  end

"#,
    );

    let feed_depth_slots = feed_slots(&manifest.boundary_inputs);
    for (idx, rec) in manifest.boundary_inputs.iter().enumerate() {
        feed_call(&mut out, idx, feed_depth_slots[idx], rec);
    }
    for (idx, rec) in manifest.boundary_outputs.iter().enumerate() {
        drain_call(&mut out, idx, rec);
    }
    for (item, x, y) in &manifest.surplus_exits {
        let _ = writeln!(
            out,
            "  add_fluid_void(s, force, {x} - LX0 + storage.offx, {y} - LY0 + storage.offy, \"{item}\")",
        );
    }

    out.push_str(
        r#"  -- Kit overlap audit (#357): create_entity in script mode stacks
  -- entities silently; overlapping bank chests cross-feed items and
  -- poison the factory with wrong-item plugs. Any overlap invalidates
  -- the run — record it loudly.
  -- Attribution matters as much as detection (#499): the position-only form
  -- of this audit reported 14 bare tiles at negative coordinates, which read
  -- as harness geometry and sent the investigation into the kit's vector
  -- algebra. The actual cause was upstream — two IDENTICAL boundary_output
  -- records, so the harness dutifully built two drain rigs on one tile. Naming
  -- the owners would have pointed straight at it. storage.feeds/drains already
  -- hold per-rig chest lists, so no rig has to change signature to do this.
  do
    local owner = {}
    local function claim(c, label)
      if not (c and c.valid) then return end
      local key = math.floor(c.position.x) .. "," .. math.floor(c.position.y)
      if owner[key] then
        table.insert(storage.kit_errors, "overlapping kit chests at (" .. key
                     .. "): " .. owner[key] .. " vs " .. label)
      else
        owner[key] = label
      end
    end
    for item, banks in pairs(storage.feeds) do
      for bi, bank in ipairs(banks) do
        for _, c in ipairs(bank.chests) do
          claim(c, "feed[" .. item .. "] rig " .. bi)
        end
      end
    end
    for item, chests in pairs(storage.drains) do
      for _, c in ipairs(chests) do
        claim(c, "drain[" .. item .. "]")
      end
    end
    -- Backstop for chests no rig registered. Without it, a rig that stops
    -- recording its chests would silently disable the whole audit — the
    -- check-goes-quiet failure this repo keeps re-learning
    -- (docs/validator-reporting.md).
    local seen = {}
    for _, c in pairs(s.find_entities_filtered{name = "steel-chest"}) do
      local key = math.floor(c.position.x) .. "," .. math.floor(c.position.y)
      if seen[key] and not owner[key] then
        table.insert(storage.kit_errors,
                     "overlapping unattributed kit chests at (" .. key .. ")")
      end
      seen[key] = true
    end
  end
end)

local function stn(st)
  for k, v in pairs(defines.entity_status) do if v == st then return k end end
  return tostring(st)
end

local function dump_sim_state(s)
  local belts, machines, inserters, pipes = {}, {}, {}, {}
  for _, b in pairs(s.find_entities_filtered{type = {"transport-belt", "underground-belt", "splitter"}}) do
    local n = 0
    -- Per-line item detail (line index -> {{name, count}, ...}). Belt
    -- counts alone are ambiguous exactly when it matters: an inserter
    -- refusing a "full" belt usually means wrong item or wrong lane,
    -- and neither is visible from a bare total (#357 recon).
    local det = {}
    for li = 1, b.get_max_transport_line_index() do
      local tl = b.get_transport_line(li)
      n = n + tl.get_item_count()
      local lane = {}
      for k, v in pairs(tl.get_contents()) do
        if type(v) == "table" then
          lane[#lane + 1] = {v.name or tostring(k), v.count or 0}
        else
          lane[#lane + 1] = {tostring(k), v}
        end
      end
      det[li] = lane
    end
    if n > 0 then
      table.insert(belts, {math.floor(b.position.x - storage.offx) + LX0,
                           math.floor(b.position.y - storage.offy) + LY0, n, det})
    end
  end
  -- Pipe/fluid section (#364): every pipe-class entity with name,
  -- direction, and fluid contents — fluids were invisible in the dump,
  -- which is why the fluid-feed fault needed controlled attribution
  -- instead of a five-minute read.
  for _, p in pairs(s.find_entities_filtered{type = {"pipe", "pipe-to-ground", "infinity-pipe", "storage-tank", "pump"}}) do
    local fl = {}
    for fname, amt in pairs(p.get_fluid_contents()) do
      table.insert(fl, {fname, math.floor(amt * 10) / 10})
    end
    table.insert(pipes, {math.floor(p.position.x - storage.offx) + LX0,
                         math.floor(p.position.y - storage.offy) + LY0,
                         p.name, p.direction, fl})
  end
  for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
    -- Fluid contents (main's fluid-calibration arc, position 5) plus
    -- solid input/output inventory contents (#357 wrong-item forensics,
    -- position 6): belts flush transient contamination within seconds,
    -- machine inventories hold it until consumed.
    local mfl = {}
    for fname, amt in pairs(m.get_fluid_contents()) do
      table.insert(mfl, {fname, math.floor(amt * 10) / 10})
    end
    local inv = {}
    for _, invid in ipairs({defines.inventory.furnace_source, defines.inventory.assembling_machine_input,
                            defines.inventory.furnace_result, defines.inventory.assembling_machine_output}) do
      local i = m.get_inventory(invid)
      if i then
        for _, it in pairs(i.get_contents()) do
          local nm = it.name or "?"
          inv[nm] = (inv[nm] or 0) + (it.count or 0)
        end
      end
    end
    table.insert(machines, {math.floor(m.position.x - storage.offx) + LX0,
                            math.floor(m.position.y - storage.offy) + LY0, m.name, stn(m.status), mfl, inv})
  end
  for _, i in pairs(s.find_entities_filtered{type = "inserter"}) do
    table.insert(inserters, {math.floor(i.position.x - storage.offx) + LX0,
                             math.floor(i.position.y - storage.offy) + LY0, stn(i.status)})
  end
  -- UG pairing as the GAME resolved it (mis-pairs teleport items across
  -- lines) and splitter priority/filter state as revived — wrong-item
  -- forensics needs both (#357).
  local ugs, splitters = {}, {}
  for _, u in pairs(s.find_entities_filtered{type = "underground-belt"}) do
    local rec = {math.floor(u.position.x - storage.offx) + LX0,
                 math.floor(u.position.y - storage.offy) + LY0, u.belt_to_ground_type}
    local n = u.neighbours
    if n and n.valid then
      rec[4] = math.floor(n.position.x - storage.offx) + LX0
      rec[5] = math.floor(n.position.y - storage.offy) + LY0
    end
    table.insert(ugs, rec)
  end
  for _, sp in pairs(s.find_entities_filtered{type = "splitter"}) do
    table.insert(splitters, {math.floor(sp.position.x - storage.offx) + LX0,
                             math.floor(sp.position.y - storage.offy) + LY0,
                             tostring(sp.splitter_output_priority), tostring(sp.splitter_input_priority),
                             sp.splitter_filter and sp.splitter_filter.name or ""})
  end
  -- Kit chest census: overlapping feed chests on a contested bank tile
  -- are invisible on belts (each rig's refill keeps its own item topped
  -- up) but poison whichever inserter latches the wrong chest (#357).
  local chests = {}
  for _, c in pairs(s.find_entities_filtered{name = "steel-chest"}) do
    local contents = {}
    for _, it in pairs(c.get_inventory(defines.inventory.chest).get_contents()) do
      contents[it.name or "?"] = (contents[it.name or "?"] or 0) + (it.count or 0)
    end
    table.insert(chests, {math.floor(c.position.x - storage.offx) + LX0,
                          math.floor(c.position.y - storage.offy) + LY0, contents})
  end
  helpers.write_file("sim-state.json", helpers.table_to_json{
    offx = storage.offx, offy = storage.offy, fed = storage.fed_total,
    belts = belts, machines = machines, inserters = inserters, pipes = pipes,
    ugs = ugs, splitters = splitters, chests = chests}, false)
end

local function finalize(s, converged)
  storage.finalized = true
  storage.converged = converged
  -- Dead-rig audit (#345): a rig whose chest bank never drained beyond
  -- the initial fill moved nothing onto its belt — the rig body is
  -- broken (r150's splitter-bodied copper rig delivered zero for a
  -- whole steady-state run while its two siblings masked it in the
  -- per-item totals). 2600 > 6 chests x 400 initial fill.
  for item, banks in pairs(storage.feeds) do
    for bi, bank in ipairs(banks) do
      if bank.fed <= 2600 and game.tick > 7200 then
        table.insert(storage.kit_errors, "feed rig " .. bi .. " for '" .. item
          .. "' never drained past its initial fill (" .. bank.fed .. ") — rig dead (#345 class)")
      end
    end
  end
  dump_sim_state(s)
  local census = {}
  for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
    local st = m.status
    for k, v in pairs(defines.entity_status) do
      if v == st then census[k] = (census[k] or 0) + 1 end
    end
  end
  -- RFC-064 item 7 productivity-parity probe (2026-08-06).
  -- The meter models NO productivity at all (crates/meter/src/machine.rs
  -- deliberately takes nothing from module_policy and not
  -- effective_crafting_speed), while this scenario calls
  -- research_all_technologies() above and its tech-state parity block corrects
  -- only inserter capacity (#370) and belt stacking (#385). If the sim carries
  -- a productivity bonus the meter cannot see, the PU-from-ore -13% residual
  -- may reduce to that parity gap rather than any layout or belt defect.
  --
  -- Read DEFENSIVELY: the exact 2.0 API spelling is not assumed here. A field
  -- that does not exist reports "FIELD_ABSENT" instead of killing the run, so
  -- a wrong guess costs a re-run, not a lost measurement. Both candidate
  -- sources are probed separately -- force/research bonus per recipe, and
  -- per-machine module contents -- because they imply different fixes.
  local function probe(get)
    local ok, v = pcall(get)
    if not ok then return "FIELD_ABSENT" end
    if v == nil then return "NIL" end
    return v
  end
  -- MEASURED 2026-08-06, against a review claim that `LuaRecipe` exposes no
  -- such field and this channel "will always serialize as FIELD_ABSENT": it
  -- returned 0.1 for processing-unit and 0.0 for the five others in the same
  -- run. A non-existent field cannot produce a recipe-DISCRIMINATING value,
  -- so the read is real. That discrimination is also the strongest evidence
  -- the probe worked at all -- a broken probe returns uniform sentinels.
  local prod_force = {}
  for _, rn in ipairs({"processing-unit", "electronic-circuit", "advanced-circuit",
                       "iron-plate", "copper-plate", "copper-cable"}) do
    prod_force[rn] = probe(function()
      local r = game.forces.player.recipes[rn]
      if r == nil then return nil end
      return r.productivity_bonus
    end)
  end
  local prod_entity, prod_modules = {}, {}
  for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
    local rn = probe(function()
      local r = m.get_recipe()
      if r == nil then return nil end
      return r.name
    end)
    if type(rn) == "string" and rn ~= "NIL" and rn ~= "FIELD_ABSENT" then
      -- Aggregate across EVERY machine of the recipe, not just the first seen.
      -- First-seen made the reported bonus an arbitrary run-to-run pick when
      -- machines of one recipe carry different module loadouts, and a
      -- first-machine probe fault suppressed every later machine that might
      -- have exposed a real number (PR #580 review, 3/3). Numeric readings
      -- collapse to min/max so a heterogeneous set is visible as a spread;
      -- faults are counted separately so they cannot masquerade as zeros.
      local eb = probe(function() return m.productivity_bonus end)
      local agg = prod_entity[rn]
      if agg == nil then agg = {min = nil, max = nil, n = 0, faults = 0} end
      if type(eb) == "number" then
        if agg.min == nil or eb < agg.min then agg.min = eb end
        if agg.max == nil or eb > agg.max then agg.max = eb end
        agg.n = agg.n + 1
      else
        agg.faults = agg.faults + 1
      end
      prod_entity[rn] = agg
      local inv = probe(function() return m.get_module_inventory() end)
      if type(inv) ~= "string" then
        local contents = probe(function() return inv.get_contents() end)
        if type(contents) == "table" then
          -- 2.0 returns an array of {name=,count=}; 1.1 returned name->count.
          for k, c in pairs(contents) do
            local nm, ct
            if type(c) == "table" then nm, ct = c.name, c.count else nm, ct = k, c end
            if nm ~= nil then
              local key = rn .. "/" .. tostring(nm)
              prod_modules[key] = (prod_modules[key] or 0) + (ct or 1)
            end
          end
        end
      end
    end
  end
  helpers.write_file("harness-result.json", helpers.table_to_json{
    import_rc = storage.import_rc, ghosts = storage.ghosts, revived = storage.revived,
    factory_eeis = storage.factory_eeis, pole_networks = storage.net_count,
    proxies_fulfilled = storage.proxies_fulfilled,
    samples = storage.samples, checkpoints = storage.checkpoints,
    timeseries = storage.timeseries,
    machine_census = census, converged = storage.converged, final_tick = game.tick,
    fluid_errors = storage.fluid_errors, kit_errors = storage.kit_errors,
    -- Realized capacity bonuses after tech-state parity (#370) — the
    -- verification channel that the tech rollback actually took effect.
    inserter_stack_size_bonus = game.forces.player.inserter_stack_size_bonus,
    bulk_inserter_capacity_bonus = game.forces.player.bulk_inserter_capacity_bonus,
    belt_stack_size_bonus = game.forces.player.belt_stack_size_bonus,
    -- RFC-064 item 7 probe (see the block above). Three separate channels so
    -- a research source and a module source are distinguishable, and so an
    -- API-spelling miss is visible as FIELD_ABSENT rather than a silent zero.
    productivity_force = prod_force,
    productivity_entity = prod_entity,
    productivity_modules = prod_modules}, false)
  print("HARNESS_DONE")
  -- Deliberately NOT deregistering the tick handler here: runtime
  -- `script.on_nth_tick(60, nil)` makes the server's handler set differ
  -- from what a freshly-loaded client registers, and Factorio refuses
  -- the join ("mod event handlers are not identical ... level"). The
  -- handler's own `storage.finalized` guard makes it a no-op after
  -- this point, which is multiplayer-safe.
end

script.on_nth_tick(60, function(ev)
  if storage.finalized then return end
  for _, e in ipairs(storage.eeis) do if e.valid then e.energy = 1e13 end end
  for item, banks in pairs(storage.feeds) do
    for _, bank in ipairs(banks) do
      for _, c in ipairs(bank.chests) do
        if c.valid then
          local n = c.get_item_count(item)
          if n < 400 then
            local got = c.insert{name = item, count = 400 - n}
            storage.fed_total[item] = (storage.fed_total[item] or 0) + got
            bank.fed = bank.fed + got
          end
        end
      end
    end
  end
  for item, chests in pairs(storage.drains) do
    local got = 0
    for _, d in ipairs(chests) do
      if d.valid then
        local n = d.get_item_count(item)
        if n > 0 then d.remove_item{name = item, count = n}; got = got + n end
      end
    end
    storage.drained_total[item] = (storage.drained_total[item] or 0) + got
  end

  local s = game.get_surface("lab")
  local stats = game.forces.player.get_item_production_statistics(s)
  local fstats = game.forces.player.get_fluid_production_statistics(s)
  -- Fluid intermediates (mega-cells, RFC-052) live in the fluid
  -- statistics; get_input_count on the ITEM stats crashes with
  -- "Unknown item name" for them. Names that are only fluid
  -- prototypes route to fluid stats.
  local function produced_count(name)
    if prototypes.fluid[name] and not prototypes.item[name] then
      return fstats.get_input_count(name)
    end
    return stats.get_input_count(name)
  end

  -- RFC-062 Phase 3: per-target produced+delivered, sampled at every
  -- checkpoint window close (same cadence as the primary-target scalars
  -- below) so EVERY target gets its own honest rate series, not just
  -- TARGETS[1]. `report.rs` reads this per-item series for its verdicts;
  -- the flat `produced`/`delivered` fields stay exactly as they were
  -- (TARGETS[1] only) for older report-side code that hasn't been
  -- updated to read `items`.
  local function checkpoint_items()
    local out = {}
    for _, item in ipairs(TARGETS) do
      out[item] = {produced = produced_count(item), delivered = storage.drained_total[item] or 0}
    end
    return out
  end

  if ev.tick % 1200 == 0 then
    local produced = {}
    for _, item in ipairs(PLANNED_ITEMS) do produced[item] = produced_count(item) end
    table.insert(storage.samples, {tick = ev.tick, drained = storage.drained_total,
      produced = produced, fed = storage.fed_total})
  end

  -- Stability checkpoints. A window closes when it has ACCUMULATED
  -- WINDOW_ITEM_FLOOR items, or when it hits WINDOW_TICK_CAP -- item-
  -- driven, not tick-driven (#454). Sizing windows from the planned rate
  -- and closing them on a fixed tick count handed an underperforming
  -- factory a proportionally undersized sample, so the further below plan
  -- it ran the less measurable it became, failing closed to NO DATA.
  --
  -- The first checkpoint is taken exactly at WARMUP_TICKS (the handler's
  -- own 60-tick cadence divides it) and opens window 1. Checkpoints used
  -- to land on multiples of the window length in ABSOLUTE tick phase
  -- instead, so measurement began at an arbitrary offset after warmup and
  -- that offset moved with any --warmup override.
  if ev.tick >= WARMUP_TICKS then
    local n = #storage.checkpoints
    local produced = produced_count(TARGET)
    local delivered = storage.drained_total[TARGET] or 0
    if n == 0 then
      table.insert(storage.checkpoints, {tick = ev.tick, produced = produced,
        delivered = delivered, window_ticks = 0, window_items = 0, short_sampled = false,
        items = checkpoint_items()})
      -- Seed the per-window delta baselines AT the warmup boundary, not at
      -- tick 0. Without this, the FIRST closed window's machine/item delta
      -- is cumulative-since-tick-0 (all of warmup's production folded into
      -- one entry) — a warmup-contaminated first sample that miscredits a
      -- fast warmup as a huge window rate. Seeding here makes every
      -- subsequent closed-window delta (JSON timeseries AND --timeseries CSV)
      -- a true per-window value.
      for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
        storage.machine_last_crafts[m.unit_number] = m.products_finished or 0
      end
      for _, item in ipairs(PLANNED_ITEMS) do
        storage.item_last_produced[item] = produced_count(item)
      end
    else
      local prev = storage.checkpoints[n]
      local d_items = produced - prev.produced
      local d_ticks = ev.tick - prev.tick
      -- A fast target hits 300 items in well under the nominal window, so
      -- the item floor alone could close a window shorter than the
      -- producer's burst cycle -- reintroducing snapshot aliasing, which
      -- MIN_WINDOW_TICKS exists to prevent. Both floors must be met.
      local by_items = d_items >= WINDOW_ITEM_FLOOR and d_ticks >= WINDOW_MIN_TICKS
      if by_items or d_ticks >= WINDOW_TICK_CAP then
        table.insert(storage.checkpoints, {tick = ev.tick, produced = produced,
          delivered = delivered, window_ticks = d_ticks, window_items = d_items,
          short_sampled = not by_items, items = checkpoint_items()})
        n = n + 1

        -- Per-window machine + item time-series (#537): sampled on the
        -- SAME cadence as the checkpoint windows above (item-driven, not
        -- a fixed tick period), so a series entry always corresponds to
        -- one closed window. Machines are identified by unit_number —
        -- stable across samples even if two machines share a name and
        -- (after a crash/rebuild) even a position. `products_finished`
        -- is a live cumulative counter per machine; the delta against
        -- the value recorded at the previous checkpoint is this
        -- window's craft count, mirroring how the target/intermediate
        -- item rates are already derived from cumulative production
        -- stats above.
        do
          local ts_machines, csv_lines = {}, {}
          for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
            local uid = m.unit_number
            local crafts = m.products_finished or 0
            local prev_crafts = storage.machine_last_crafts[uid] or 0
            local delta = crafts - prev_crafts
            local status = stn(m.status)
            local mx = math.floor(m.position.x - storage.offx) + LX0
            local my = math.floor(m.position.y - storage.offy) + LY0
            table.insert(ts_machines, {unit = uid, name = m.name, x = mx, y = my,
              crafts_delta = delta, status = status})
            storage.machine_last_crafts[uid] = crafts
            if WRITE_TIMESERIES_CSV then
              table.insert(csv_lines,
                table.concat({ev.tick, "machine", uid, m.name, mx, my, delta, status, "", ""}, ","))
            end
          end
          local ts_items = {}
          for _, item in ipairs(PLANNED_ITEMS) do
            local cur = produced_count(item)
            local prev_item = storage.item_last_produced[item] or 0
            local idelta = cur - prev_item
            ts_items[item] = idelta
            storage.item_last_produced[item] = cur
            if WRITE_TIMESERIES_CSV then
              table.insert(csv_lines,
                table.concat({ev.tick, "item", "", "", "", "", "", "", item, idelta}, ","))
            end
          end
          table.insert(storage.timeseries, {tick = ev.tick, machines = ts_machines, items = ts_items})
          if WRITE_TIMESERIES_CSV and #csv_lines > 0 then
            helpers.write_file(TIMESERIES_CSV_FILE, table.concat(csv_lines, "\n") .. "\n", true)
          end
        end

        -- Convergence = the trailing STABILITY_WINDOWS window rates all
        -- agree, compared widest-vs-narrowest rather than pairwise.
        --
        -- Comparing only the LAST TWO -- "the last step was small" -- is
        -- passed by any decelerating ramp, and passes it BELOW the
        -- asymptote: chem5 certified 4.62 -> 4.92 -> 5.00/s and reported
        -- the last window as steady state. Across a span a ramp keeps
        -- accumulating (+8.3% there) while real noise cancels, so the
        -- group comparison rejects what the pairwise one waved through.
        if n >= STABILITY_WINDOWS + 1 then
          local lo, hi, ok = nil, nil, true
          for i = n - STABILITY_WINDOWS + 1, n do
            local a, b = storage.checkpoints[i - 1], storage.checkpoints[i]
            local dt = (b.tick - a.tick) / 60
            local r = dt > 0 and (b.produced - a.produced) / dt or 0
            if r <= 0 then
              ok = false
              break
            end
            if lo == nil or r < lo then lo = r end
            if hi == nil or r > hi then hi = r end
          end
          if ok and lo and lo > 0 and (hi - lo) / lo <= STABILITY_TOL then
            finalize(s, true)
            return
          end
        end
      end
    end
  end

  if not storage.finalized and ev.tick >= END_TICK then
    finalize(s, false)
  end
end)
"#,
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

    fn fixture() -> Manifest {
        Manifest::from_str(include_str!("../tests/fixtures/manifest_gear10.json")).unwrap()
    }

    #[test]
    fn warmup_scales_with_dims() {
        assert_eq!(default_warmup_ticks(0, 0), round_up_60(BASE_WARMUP_TICKS));
        // gear10: 53x34 -> base + 2*(87)*32 = 3600 + 5568 = 9168 -> round to 9180
        assert_eq!(
            default_warmup_ticks(53, 34),
            round_up_60(3600 + 2 * 87 * 32)
        );
    }

    #[test]
    fn parity_bonus_tables_and_level_are_emitted() {
        // Pins the Lua NB_BONUS/BULK_BONUS tables (#370). These are a
        // hand-copied projection of spaghettio_core's inserter_hand
        // tables (non-bulk hand − 1; bulk hand − 1 ≡ stack hand − 5 —
        // the two force fields cover all three tables); the harness
        // deliberately doesn't depend on core, so this test is the
        // drift guard. If core's I8b tables ever change, update BOTH
        // and re-run the calibration matrix (RFC-049 decision log).
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local NB_BONUS = {0, 0, 1, 1, 1, 1, 1, 3}"));
        assert!(lua.contains("local BULK_BONUS = {1, 2, 3, 4, 5, 7, 9, 11}"));
        assert!(lua.contains(&format!(
            "local INSERTER_CAPACITY = {}",
            m.inserter_capacity
        )));
        // The self-audit must reference kit_errors so a failed
        // assignment invalidates the run rather than passing silently.
        assert!(lua.contains("tech-state parity assignment did not take"));
        // Belt-stacking parity (option A): declared S drives the force
        // bonus; the self-audit must be able to invalidate the run.
        assert!(lua.contains(&format!("local STACKING = {}", m.stacking)));
        assert!(lua.contains("force.belt_stack_size_bonus = STACKING - 1"));
        assert!(lua.contains("belt-stacking parity assignment did not take"));
    }

    #[test]
    fn warmup_override_rounds_to_cadence_and_lifts_ceiling() {
        let p = RunParams {
            end_tick: 10_000,
            speed: 16,
            warmup_ticks: 3600,
            window_ticks: 1800,
            scenario_name: "t".into(),
            operator_qol: false,
            write_timeseries: false,
        }
        .with_warmup(216_001);
        assert_eq!(p.warmup_ticks, 216_060);
        // The ceiling must clear the whole convergence test, not one
        // window: `+ window_ticks` left room for a single checkpoint
        // where the test needs three, so a --warmup override reported
        // `converged: false` by construction (#454).
        assert_eq!(
            p.end_tick,
            216_060 + window_tick_cap(1800) * STABILITY_WINDOWS
        );
    }

    /// #454 regression. `mega-chain-usp2raw --warmup 480000` finished
    /// with exactly ONE checkpoint (`end_tick` 489,000 = warmup + one
    /// 9,000-tick window) and reported `converged: false` — a verdict
    /// about the tick budget that was read as a verdict about the
    /// factory. Any warmup must leave room for `MIN_CHECKPOINTS`.
    #[test]
    fn ceiling_always_fits_the_convergence_test() {
        let m = fixture();
        for warmup in [0, 3_600, 160_000, 216_001, 480_000, 1_000_000] {
            for explicit in [None, Some(1_000), Some(12_000)] {
                let p = RunParams::defaults_for(&m, "t".into(), 16, explicit).with_warmup(warmup);
                let measurable = p.end_tick - p.warmup_ticks;
                let worst_case_windows = window_tick_cap(p.window_ticks) * STABILITY_WINDOWS;
                assert!(
                    measurable >= worst_case_windows,
                    "warmup={warmup} explicit={explicit:?}: only {measurable} ticks after \
                     warmup, need {worst_case_windows} to close {STABILITY_WINDOWS} windows"
                );
            }
        }
    }

    /// The window the run measures over is chosen by accumulated items,
    /// so a factory below plan gets a longer window rather than a thinner
    /// sample. Pins the generated Lua's close condition (#454).
    #[test]
    fn checkpoints_close_on_items_not_on_absolute_tick_phase() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains(&format!("local WINDOW_ITEM_FLOOR = {WINDOW_ITEM_FLOOR}")));
        assert!(lua.contains(&format!(
            "local WINDOW_TICK_CAP = {}",
            window_tick_cap(params.window_ticks)
        )));
        // Closes on the item floor, with the tick cap as the bound — and
        // never shorter than MIN_WINDOW_TICKS, or a fast target would
        // reach 300 items inside its own burst cycle and alias.
        assert!(lua.contains(&format!("local WINDOW_MIN_TICKS = {MIN_WINDOW_TICKS}")));
        assert!(lua.contains(
            "local by_items = d_items >= WINDOW_ITEM_FLOOR and d_ticks >= WINDOW_MIN_TICKS"
        ));
        assert!(lua.contains("if by_items or d_ticks >= WINDOW_TICK_CAP then"));
        // Measurement opens exactly at warmup, not at whatever absolute
        // multiple of the window length happens to fall after it.
        assert!(lua.contains("if ev.tick >= WARMUP_TICKS then"));
        assert!(
            !lua.contains("ev.tick % WINDOW_TICKS == 0"),
            "absolute-phase checkpoint test still present"
        );
        // A capped window must be reportable as short-sampled.
        assert!(lua.contains("short_sampled = not by_items"));
        // Convergence compares a GROUP of windows widest-vs-narrowest,
        // not the last pair — a decelerating ramp passes any
        // last-step test once its slope flattens under tolerance.
        assert!(lua.contains(&format!("local STABILITY_WINDOWS = {STABILITY_WINDOWS}")));
        assert!(lua.contains("if n >= STABILITY_WINDOWS + 1 then"));
        assert!(lua.contains("(hi - lo) / lo <= STABILITY_TOL"));
    }

    /// The default wall-clock net must clear the run's own tick budget,
    /// or a slow run is killed before it can write the non-converged
    /// report that is the whole point of measuring it (#464 review).
    #[test]
    fn default_timeout_always_clears_the_tick_budget() {
        let m = fixture();
        for warmup in [0, 3_600, 160_000, 480_000] {
            for speed in [1, 8, 16, 64] {
                let p = RunParams::defaults_for(&m, "t".into(), speed, None).with_warmup(warmup);
                let timeout = default_timeout_secs(p.end_tick, p.speed);
                let at_requested_speed = p.end_tick as f64 / (60.0 * speed as f64);
                assert!(
                    timeout as f64 >= at_requested_speed,
                    "warmup={warmup} speed={speed}: timeout {timeout}s < {at_requested_speed:.0}s \
                     needed to reach end_tick {} even at the REQUESTED speed",
                    p.end_tick
                );
            }
        }
        // The usp2 case the review flagged: 447,960 ticks at speed 16 is
        // 466s at the requested rate and ~1545s at the ~290 ticks/s that
        // fixture actually achieves — the old fixed 900s fell between.
        assert!(default_timeout_secs(447_960, 16) > 1_545);
        // Small fixtures keep the previous default.
        assert_eq!(default_timeout_secs(12_000, 16), MIN_TIMEOUT_SECS);
    }

    #[test]
    fn window_floors_at_min_and_scales_inversely_with_rate() {
        // 10 items/s -> 300/10 = 30s = 1800 ticks
        assert_eq!(default_window_ticks(10.0), 1800);
        // 1000 items/s -> 0.3s -> floored to MIN_WINDOW_TICKS
        assert_eq!(default_window_ticks(1000.0), round_up_60(MIN_WINDOW_TICKS));
    }

    #[test]
    fn gear10_feed_reduces_to_calibrated_prototype_numbers() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-gear".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);

        // south-facing feed: outward=(0,-1), lateral=(1,0) -- matches the
        // literal gen_harness_scenario.py call shape (o=north, l=east).
        assert!(lua.contains(
            "add_feed(s, force, head_x, head_y, 0, -1, 1, 0, 4, \"iron-ore\", \"transport-belt\")"
        ));
        // Second rig at depth 10 (6-per-idx stagger, > the bank's ±2
        // chest offset — see feed_call's collision comment / #357).
        assert!(lua.contains(
            "add_feed(s, force, head_x, head_y, 0, -1, 1, 0, 10, \"iron-ore\", \"transport-belt\")"
        ));
        // head world-position translation, anchored to the manifest bbox_min
        assert!(
            lua.contains("local head_x, head_y = 1 - LX0 + storage.offx, 0 - LY0 + storage.offy")
        );
        assert!(
            lua.contains("local head_x, head_y = 2 - LX0 + storage.offx, 0 - LY0 + storage.offy")
        );
    }

    /// #363 regression: manifest_gear10.json's two south-facing iron-ore
    /// feeds are already listed west->east (x=1 then x=2), so the OLD
    /// idx-based depth happened to work on this fixture by accident. This
    /// test reverses the manifest order (x=2 first, x=1 second) — the
    /// same shape as the issue's second live datum ("depth grows with
    /// record order" instead of position) — and asserts the west head
    /// (x=1) still gets the SMALLER depth (4) and the east head (x=2)
    /// still gets the BIGGER depth (10), unchanged from forward order.
    /// Checks depth and head position as an adjacent two-line block (not
    /// independent substrings) so the assertion can't pass by each head
    /// merely appearing somewhere with some depth.
    #[test]
    fn feed_depth_follows_lateral_position_not_manifest_order() {
        let mut m = fixture();
        m.boundary_inputs.reverse();
        assert_eq!(
            m.boundary_inputs[0].x, 2,
            "sanity: manifest now lists east before west"
        );
        assert_eq!(m.boundary_inputs[1].x, 1);

        let params = RunParams::defaults_for(&m, "test-gear-rev".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);

        let west_block =
            "    local head_x, head_y = 1 - LX0 + storage.offx, 0 - LY0 + storage.offy\n    \
            add_feed(s, force, head_x, head_y, 0, -1, 1, 0, 4, \"iron-ore\", \"transport-belt\")";
        assert!(
            lua.contains(west_block),
            "west head (x=1) must keep the smaller depth after reordering:\n{lua}"
        );

        let east_block =
            "    local head_x, head_y = 2 - LX0 + storage.offx, 0 - LY0 + storage.offy\n    \
            add_feed(s, force, head_x, head_y, 0, -1, 1, 0, 10, \"iron-ore\", \"transport-belt\")";
        assert!(
            lua.contains(east_block),
            "east head (x=2) must keep the bigger depth after reordering:\n{lua}"
        );
    }

    /// Mirrors `add_feed`'s Lua tile placement (see the module docs above)
    /// so geometry can be checked for overlap without a live server.
    /// Belts, chests, and stack-inserters are all 1x1 in Factorio, so a
    /// tile-position set catches every real collision the shape #363's
    /// second datum describes ("the later create_entity calls fail
    /// silently"). The substation/EEI power island (further out along the
    /// jog than the chest bank) isn't included — it's never the colliding
    /// entity class in the issue's datum.
    fn feed_footprint(
        head: (i32, i32),
        into: (i32, i32),
        depth: i32,
    ) -> std::collections::HashSet<(i32, i32)> {
        let outward = neg(into);
        let lateral = rot90(into);
        let neg_lateral = neg(lateral);
        let corner = (head.0 + outward.0 * depth, head.1 + outward.1 * depth);
        let mut tiles = std::collections::HashSet::new();
        for t in 1..=depth {
            tiles.insert((head.0 + outward.0 * t, head.1 + outward.1 * t));
        }
        for k in 1..=12 {
            tiles.insert((corner.0 + neg_lateral.0 * k, corner.1 + neg_lateral.1 * k));
        }
        for k in 10..=12 {
            let b = (corner.0 + neg_lateral.0 * k, corner.1 + neg_lateral.1 * k);
            for side in [-1, 1] {
                tiles.insert((b.0 + into.0 * 2 * side, b.1 + into.1 * 2 * side));
                tiles.insert((b.0 + into.0 * side, b.1 + into.1 * side));
            }
        }
        tiles
    }

    /// The fluid feed's occupied tiles: the whole outward column through
    /// the infinity cap (out-tiles 1..=2+dist). Conservative — the ug
    /// gap tiles hold no entity — but claiming them keeps the disjointness
    /// argument independent of that detail.
    fn fluid_feed_footprint(
        head: (i32, i32),
        into: (i32, i32),
        dist: i32,
    ) -> std::collections::HashSet<(i32, i32)> {
        let outward = neg(into);
        (1..=2 + dist)
            .map(|t| (head.0 + outward.0 * t, head.1 + outward.1 * t))
            .collect()
    }

    fn south_feed(item: &str, x: i32) -> BoundaryRecord {
        BoundaryRecord {
            item: item.into(),
            x,
            y: 0,
            direction: 8, // south
            is_fluid: false,
            entity: "transport-belt".into(),
        }
    }

    fn south_fluid_feed(item: &str, x: i32) -> BoundaryRecord {
        BoundaryRecord {
            item: item.into(),
            x,
            y: 0,
            direction: 8, // south
            is_fluid: true,
            entity: "pipe-to-ground".into(),
        }
    }

    /// Asserts every record's footprint (item rig or fluid ug-run, using
    /// the slot `feed_slots` assigns it) is pairwise disjoint from every
    /// other's — the direct geometry-level check for #363's "rigs
    /// self-collide" datum, extended to fluid feeds (PR #515 review).
    fn assert_feed_footprints_disjoint(records: &[BoundaryRecord]) {
        let slots = feed_slots(records);
        let mut occupied: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for (i, rec) in records.iter().enumerate() {
            let into = rec.direction().vector();
            let depth = 4 + 6 * slots[i];
            let footprint = if rec.is_fluid {
                fluid_feed_footprint((rec.x, rec.y), into, 2 + 2 * slots[i])
            } else {
                feed_footprint((rec.x, rec.y), into, depth)
            };
            for tile in &footprint {
                assert!(
                    !occupied.contains(tile),
                    "rig for '{}' (x={}, slot={}, depth={}) collides with an earlier rig at tile {:?}",
                    rec.item,
                    rec.x,
                    slots[i],
                    depth,
                    tile
                );
            }
            occupied.extend(footprint);
        }
    }

    /// #363 second live datum, 1-tile lateral pitch, listed worst-case
    /// (east-to-west — adversarial to the old idx-based depth): "only the
    /// first-ordered rig per group worked, at both 1-tile and 4-tile
    /// column spacing".
    #[test]
    fn feed_footprints_disjoint_at_1_tile_pitch_reverse_order() {
        let records = vec![south_feed("a", 2), south_feed("b", 1), south_feed("c", 0)];
        assert_feed_footprints_disjoint(&records);
    }

    /// Same datum at the issue's other measured pitch (4 tiles).
    #[test]
    fn feed_footprints_disjoint_at_4_tile_pitch_reverse_order() {
        let records = vec![south_feed("a", 8), south_feed("b", 4), south_feed("c", 0)];
        assert_feed_footprints_disjoint(&records);
    }

    /// Forward order (already west->east) must also stay collision-free —
    /// the fix must not depend on which order happens to be adversarial.
    #[test]
    fn feed_footprints_disjoint_at_1_tile_pitch_forward_order() {
        let records = vec![south_feed("a", 0), south_feed("b", 1), south_feed("c", 2)];
        assert_feed_footprints_disjoint(&records);
    }

    /// Fluids and items interleaved on ONE boundary side (PR #515 review
    /// finding: the old separate fluid counter staggered fluids only
    /// against each other, so a fluid ug-run could land exactly on an
    /// item rig's jog row and stack silently). Fluids must take the
    /// ladder front — every fluid tile strictly below every item band —
    /// and the combined footprint set must be pairwise disjoint even at
    /// 1-tile pitch with fluid columns inside the item span.
    #[test]
    fn mixed_fluid_and_item_feeds_share_one_ladder_disjointly() {
        let records = vec![
            south_feed("iron-ore", 0),
            south_fluid_feed("crude-oil", 1),
            south_feed("copper-ore", 2),
            south_fluid_feed("water", 3),
            south_feed("coal", 4),
        ];
        assert_feed_footprints_disjoint(&records);
        let slots = feed_slots(&records);
        let fluid_slots: std::collections::BTreeSet<i32> = [slots[1], slots[3]].into();
        let item_slots: std::collections::BTreeSet<i32> = [slots[0], slots[2], slots[4]].into();
        assert_eq!(
            fluid_slots,
            [0, 1].into(),
            "fluids must hold the ladder front"
        );
        assert_eq!(
            item_slots,
            [2, 3, 4].into(),
            "items must shift above the fluids"
        );
    }

    /// A sixth same-side fluid would need a ug span beyond the game's
    /// 9-tile limit; the codegen must refuse loudly, not fabricate.
    #[test]
    #[should_panic(expected = "more than 5 fluid boundary feeds")]
    fn six_fluid_feeds_on_one_side_panic() {
        let mut m = fixture();
        for x in 10..16 {
            m.boundary_inputs.push(BoundaryRecord {
                item: format!("fluid-{x}"),
                x,
                y: 0,
                direction: 8,
                is_fluid: true,
                entity: "pipe-to-ground".into(),
            });
        }
        let params = RunParams::defaults_for(&m, "test-sixfluid".into(), 16, Some(18000));
        build_control_lua(&m, "0eNBPFAKE", &params);
    }

    #[test]
    fn gear10_drain_reduces_to_calibrated_prototype_numbers() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-gear".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);

        // south-facing exit: flow=(0,1), lateral=(1,0) -- matches
        // gen_harness_scenario.py's drain (fx=south, lx=east). ext_len
        // is 11 + 2*idx (idx=0 here): widened 2026-07-24 from 5 so the
        // bank spans 9 positions (`t = ext_len - 8, ext_len`) / 18
        // inserters, keeping every chest outside the layout.
        assert!(lua
            .contains("add_drain(s, force, exit_x, exit_y, 0, 1, 1, 0, 11, \"iron-gear-wheel\")"));
        assert!(
            lua.contains("local exit_x, exit_y = 13 - LX0 + storage.offx, 33 - LY0 + storage.offy")
        );
    }

    #[test]
    fn no_fluid_call_sites_when_no_fluid_boundaries() {
        // The shared helper *definitions* are always emitted (boilerplate,
        // cheap to keep even when unused); what must NOT appear without a
        // fluid boundary is a *call site*. Both the definition line and a
        // call site contain the substring "add_fluid_feed(s, force", so
        // distinguish by occurrence count: exactly 1 (the definition)
        // means zero call sites.
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-gear".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert_eq!(
            lua.matches("add_fluid_feed(s, force").count(),
            1,
            "only the definition, no calls"
        );
        assert_eq!(
            lua.matches("add_fluid_void(s, force").count(),
            1,
            "only the definition, no calls"
        );
    }

    #[test]
    fn fluid_input_uses_infinity_pipe_feed() {
        let mut m = fixture();
        m.boundary_inputs.push(BoundaryRecord {
            item: "water".into(),
            x: 5,
            y: 0,
            direction: 8,
            is_fluid: true,
            entity: "pipe".into(),
        });
        let params = RunParams::defaults_for(&m, "test-fluid".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("add_fluid_feed(s, force, head_x, head_y, 0, -1, 2, \"water\")"));
    }

    /// Two ADJACENT fluid ports must get different ug-run lengths, or
    /// their infinity caps sit side by side and merge into one network —
    /// the pu3 crude/water cross-feed (K60-3 forensics 2026-07-31).
    #[test]
    fn adjacent_fluid_feeds_get_staggered_run_lengths() {
        let mut m = fixture();
        for (x, item) in [(38, "crude-oil"), (39, "water")] {
            m.boundary_inputs.push(BoundaryRecord {
                item: item.into(),
                x,
                y: 0,
                direction: 8,
                is_fluid: true,
                entity: "pipe-to-ground".into(),
            });
        }
        let params = RunParams::defaults_for(&m, "test-fluid2".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        let crude =
            lua.contains("add_fluid_feed(s, force, head_x, head_y, 0, -1, 2, \"crude-oil\")");
        let water = lua.contains("add_fluid_feed(s, force, head_x, head_y, 0, -1, 4, \"water\")");
        let crude_swap =
            lua.contains("add_fluid_feed(s, force, head_x, head_y, 0, -1, 4, \"crude-oil\")");
        let water_swap =
            lua.contains("add_fluid_feed(s, force, head_x, head_y, 0, -1, 2, \"water\")");
        assert!(
            (crude && water) || (crude_swap && water_swap),
            "adjacent fluid feeds must get dist 2 and 4 (either order)"
        );
    }

    #[test]
    fn contains_calibrated_mechanics_markers() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-gear".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("research_all_technologies()"));
        assert!(lua.contains("build_mode.superforced"));
        assert!(lua.contains("quality = \"legendary\""));
        assert!(lua.contains("fulfill_module_proxies"));
        assert!(lua.contains("game.speed"));
        assert!(lua.contains("HARNESS_DONE"));
        assert!(lua.contains("local PLANNED_ITEMS ="));
        assert!(lua.contains("\"iron-gear-wheel\"") && lua.contains("\"iron-ore\""));
    }

    /// The operator conveniences must never leak into a measurement:
    /// they set force bonuses, and a measurement has to run in the world
    /// its fixture declares (the #370 tech-state parity argument).
    ///
    /// The QoL must also live INSIDE the existing join handler. A second
    /// `script.on_event` for the same event replaces the first, which
    /// would drop the lab-surface teleport and strand the player on
    /// nauvis — so this asserts there is still exactly ONE registration
    /// and that the teleport survives.
    #[test]
    fn operator_qol_is_serve_only_and_keeps_the_teleport() {
        let m = fixture();
        let plain = RunParams::defaults_for(&m, "t".into(), 16, Some(18000));
        assert!(
            !plain.operator_qol,
            "measurement runs must default to no QoL"
        );
        let plain_lua = build_control_lua(&m, "0eNBPFAKE", &plain);
        assert!(
            !plain_lua.contains("character_running_speed_modifier"),
            "a measurement scenario must not touch force bonuses"
        );
        assert!(
            !plain_lua.contains("chart("),
            "a measurement must not chart the map"
        );

        let served = RunParams::defaults_for(&m, "t".into(), 1, Some(18000)).with_operator_qol();
        let served_lua = build_control_lua(&m, "0eNBPFAKE", &served);
        assert!(served_lua.contains("character_running_speed_modifier"));
        assert!(
            served_lua.contains("chart("),
            "serve must chart the paste area"
        );

        // Exactly one handler, in both modes, and the teleport intact.
        for (label, lua) in [("measure", &plain_lua), ("serve", &served_lua)] {
            assert_eq!(
                lua.matches("defines.events.on_player_joined_game").count(),
                1,
                "{label}: a second on_player_joined_game registration would REPLACE the first"
            );
            assert!(
                lua.contains("teleported "),
                "{label}: the lab-surface teleport must survive"
            );
        }
    }

    /// #537: the checkpoint-window loop must sample per-machine crafts
    /// deltas + status and per-item produced deltas, and the finalize
    /// write must surface them under a `timeseries` key in
    /// harness-result.json (additive alongside `checkpoints`/`samples`).
    #[test]
    fn timeseries_sampling_present_in_generated_lua() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-ts".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("storage.timeseries = {}"));
        assert!(lua.contains("storage.machine_last_crafts, storage.item_last_produced = {}, {}"));
        // Sampled inside the checkpoint-close branch, over crafting
        // machines only (assembling-machine/furnace, same filter as the
        // existing sim-state dump and machine census).
        assert!(lua.contains(
            "for _, m in pairs(s.find_entities_filtered{type = {\"assembling-machine\", \"furnace\"}}) do"
        ));
        assert!(lua.contains("local crafts = m.products_finished or 0"));
        assert!(lua.contains("crafts_delta = delta, status = status"));
        assert!(lua.contains("table.insert(storage.timeseries, {tick = ev.tick, machines = ts_machines, items = ts_items})"));
        // Surfaced in the JSON report, additive next to the existing keys.
        assert!(lua.contains("timeseries = storage.timeseries,"));
        assert!(lua.contains("samples = storage.samples, checkpoints = storage.checkpoints,"));
    }

    /// CSV appending to script-output is `serve`-only (`operator_qol`) —
    /// a measurement run (`run`) must not pay the extra file I/O, and
    /// must not create the file at all.
    #[test]
    fn csv_timeseries_only_enabled_under_operator_qol() {
        let m = fixture();
        let plain = RunParams::defaults_for(&m, "t".into(), 16, Some(18000));
        let plain_lua = build_control_lua(&m, "0eNBPFAKE", &plain);
        assert!(plain_lua.contains("local WRITE_TIMESERIES_CSV = false"));

        let served = RunParams::defaults_for(&m, "t".into(), 1, Some(18000)).with_operator_qol();
        let served_lua = build_control_lua(&m, "0eNBPFAKE", &served);
        assert!(served_lua.contains("local WRITE_TIMESERIES_CSV = true"));

        // Both variants emit the SAME sampling code (runtime-gated), not
        // a duplicated code path per mode.
        assert!(plain_lua.contains("if WRITE_TIMESERIES_CSV then"));
        assert!(served_lua.contains("if WRITE_TIMESERIES_CSV then"));
        assert!(plain_lua.contains("local TIMESERIES_CSV_FILE = \"timeseries.csv\""));

        // The CSV header names the schema columns, written once at init.
        assert!(
            served_lua.contains("tick,kind,unit,name,x,y,crafts_delta,status,item,produced_delta")
        );
    }

    #[test]
    fn scenario_name_and_bp_are_embedded() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "my-scenario".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNSOMEBP", &params);
        assert!(lua.contains("local BP = \"0eNSOMEBP\""));
        // RFC-062 Phase 3: TARGET is now derived from TARGETS[1], not a
        // literal string — see `targets_generalized_to_array` below for
        // the array-emission + per-item checkpoint coverage.
        assert!(lua.contains("local TARGETS = {\"iron-gear-wheel\"}"));
        assert!(lua.contains("local TARGET = TARGETS[1] or \"\""));
    }

    /// RFC-062 Phase 3: the single `TARGET` Lua global generalizes to a
    /// `TARGETS` array (every requested target, not just the first), and
    /// every checkpoint insertion additionally records a per-item
    /// `items = {...}` sub-table (produced+delivered per target) via the
    /// new `checkpoint_items()` helper — report.rs's per-target verdicts
    /// depend on this. The flat `produced`/`delivered` fields (TARGETS[1]
    /// only) stay in place unchanged — additive, not a replacement.
    #[test]
    fn targets_generalized_to_array_with_per_item_checkpoints() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-multi".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local TARGETS = {\"iron-gear-wheel\"}"));
        assert!(lua.contains("local TARGET = TARGETS[1] or \"\""));
        assert!(lua.contains("local function checkpoint_items()"));
        assert!(lua.contains("for _, item in ipairs(TARGETS) do"));
        assert!(lua.contains(
            "out[item] = {produced = produced_count(item), delivered = storage.drained_total[item] or 0}"
        ));
        // Both checkpoint-insertion sites (the n==0 baseline and the
        // window-close branch) carry the new `items` key alongside the
        // existing scalar fields.
        assert!(lua.contains(
            "delivered = delivered, window_ticks = 0, window_items = 0, short_sampled = false,\n        items = checkpoint_items()"
        ));
        assert!(lua.contains("short_sampled = not by_items, items = checkpoint_items()"));
    }

    /// N >= 2 targets: `TARGETS` carries every one of them (in manifest
    /// order), and `TARGET`/window-closing still keys off the first —
    /// the per-item series is what makes the SECOND target's verdict
    /// honest, not a change to which target gates convergence.
    #[test]
    fn targets_array_carries_every_target_for_multi_target_manifest() {
        use crate::manifest::ItemRate;
        let mut m = fixture();
        m.targets = vec![
            ItemRate {
                item: "electronic-circuit".to_string(),
                rate: 10.0,
                is_fluid: false,
            },
            ItemRate {
                item: "advanced-circuit".to_string(),
                rate: 3.0,
                is_fluid: false,
            },
        ];
        let params = RunParams::defaults_for(&m, "test-ec-ac".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local TARGETS = {\"electronic-circuit\", \"advanced-circuit\"}"));
        assert!(lua.contains("local TARGET = TARGETS[1] or \"\""));
    }

    /// RFC-062 Phase 3 final-gate finding: `add_drain`'s belt-placement
    /// loop must record a `kit_errors` entry when `create_entity` fails,
    /// instead of continuing silently — the EC+AC final-gate run hit
    /// exactly this (chest/inserter bank built per formula, entire belt
    /// extension silently absent, zero error anywhere) and it cost real
    /// debugging time to find via `sim_state` frame-reading alone. This
    /// does not fix the underlying placement failure (root cause not yet
    /// identified — tracked as a followup); it only makes a future
    /// occurrence loud.
    #[test]
    fn drain_belt_placement_failure_is_recorded_not_silent() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-drain-err".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local placed = s.create_entity{name = \"express-transport-belt\""));
        assert!(lua.contains("if not placed then"));
        assert!(lua.contains("table.insert(storage.kit_errors, \"drain rig for '\" .. item .. \"'"));
    }

    /// RFC-062 Phase 3 final-gate finding: `add_drain` must explicitly
    /// chunk-generate its own rig footprint before placing anything in
    /// it, not rely solely on the global origin-centered
    /// `request_to_generate_chunks` call — the #345/PU@4 chunk-
    /// truncation class (see the `gen_radius` comment) silently drops
    /// entities placed on an ungenerated chunk, and a second/later-
    /// indexed drain rig can land at an edge the global radius doesn't
    /// reliably reach for every possible exit position.
    #[test]
    fn drain_rig_explicitly_generates_its_own_chunk_footprint() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-drain-chunks".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(
            lua.contains(
                "s.request_to_generate_chunks({exit_x + fx * ext_len, exit_y + fy * ext_len}, 3)"
            ),
            "add_drain must generate its own footprint before placing entities in it"
        );
    }

    /// `run --timeseries` must stream the per-window CSV without enabling any
    /// `serve`-only operator QoL (which changes force bonuses + reveals the
    /// map) — the whole point is a measurement-safe LIVE progress signal for
    /// long/grinding runs.
    #[test]
    fn run_timeseries_streams_csv_without_operator_qol() {
        let m = fixture();
        let run = RunParams::defaults_for(&m, "t".into(), 16, Some(18000)).with_timeseries();
        let lua = build_control_lua(&m, "0eNBPFAKE", &run);
        // CSV streaming turned on...
        assert!(
            lua.contains("local WRITE_TIMESERIES_CSV = true"),
            "run --timeseries must emit the live CSV"
        );
        // ...but NO operator QoL: no map reveal, no reach/speed bonus.
        assert!(
            !lua.contains("character_reach_distance_bonus"),
            "--timeseries must not enable operator QoL"
        );
        assert!(!lua.contains("character_running_speed_modifier"));

        // And `run` default stays measurement-quiet (CSV off), distinct from
        // `serve`.
        let plain = RunParams::defaults_for(&m, "t".into(), 16, Some(18000));
        assert!(build_control_lua(&m, "0eNBPFAKE", &plain)
            .contains("local WRITE_TIMESERIES_CSV = false"));
    }
}
