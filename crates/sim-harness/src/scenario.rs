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
    /// Diagnostic mode: close one exact post-warmup window and suppress
    /// early convergence. Normal runs retain item-driven windows.
    pub fixed_window: bool,
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
    /// Diagnostic performance mode: retain only the belt-to-machine pickup
    /// trace and skip the per-tick belt-drop probes. This is useful for a
    /// long engine-vs-meter pickup comparison on large layouts; it changes
    /// no simulation state or measurement counters.
    pub pickup_trace_only: bool,
    /// Diagnostic mode: enable the expensive tick-synchronised belt-drop
    /// transition trace. Ordinary runs keep the cheaper 60-tick drop probe;
    /// this channel is opt-in because it queries every transport inserter on
    /// every game tick.
    pub drop_trace: bool,
    /// `serve` only: keep the boundary kit ALIVE after the scenario
    /// finalizes, so an inspected world keeps running instead of dying
    /// under the operator.
    ///
    /// Why this exists (2026-08-07, found by in-client observation): the
    /// kit's feed top-up, drain empty, and electric-interface recharge all
    /// live in one `on_nth_tick(60)` handler whose first line is
    /// `if storage.finalized then return end`. `finalize()` has TWO
    /// callers — the `END_TICK` ceiling and, far earlier, the
    /// **convergence** test. `serve` pushes `end_tick` out to ~a week of
    /// game time specifically so the world "does not finalize and stop
    /// while being inspected", but that only guards the ceiling path: a
    /// served world still self-finalized the moment its rates stabilized,
    /// typically minutes in. Past that point the feed chests were never
    /// refilled, the drains never emptied, and the power interfaces never
    /// recharged — the factory starved and stopped, and an operator who
    /// joined later was looking at a corpse while believing they were
    /// looking at the layout. Reported as "the input chests are empty and
    /// I can't see much happening".
    ///
    /// Deliberately a separate flag from [`RunParams::operator_qol`]:
    /// that one changes force bonuses and map reveal (never safe for a
    /// measurement), whereas this only decides whether the kit keeps
    /// feeding after the report is written. Keeping them distinct means a
    /// reader never has to wonder why map-reveal controls chest refills.
    ///
    /// **Never set for measurement runs.** There, finalize-stops-the-kit
    /// is correct: the checkpoints are taken before finalize, and letting
    /// the kit run on would keep mutating the world after the numbers it
    /// reports were sampled.
    pub keep_alive: bool,
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
            fixed_window: false,
            scenario_name,
            operator_qol: false,
            write_timeseries: false,
            pickup_trace_only: false,
            drop_trace: false,
            keep_alive: false,
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

    /// Keep only the pickup-event telemetry on a diagnostic run. See
    /// [`RunParams::pickup_trace_only`].
    pub fn with_pickup_trace_only(mut self) -> RunParams {
        self.pickup_trace_only = true;
        self
    }

    /// Enable the tick-synchronised belt-drop transition trace. See
    /// [`RunParams::drop_trace`].
    pub fn with_drop_trace(mut self) -> RunParams {
        self.drop_trace = true;
        self
    }

    /// Keep the boundary kit feeding after finalize (`serve`). See
    /// [`RunParams::keep_alive`] for why an inspected world needs this.
    pub fn with_keep_alive(mut self) -> RunParams {
        self.keep_alive = true;
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
        self.end_tick = if self.fixed_window {
            self.warmup_ticks.saturating_add(self.window_ticks)
        } else {
            self.end_tick
                .max(viable_end_tick(self.warmup_ticks, self.window_ticks))
        };
        self
    }

    /// Use one exact post-warmup window instead of item-driven convergence.
    /// This is an opt-in diagnostic so a long meter window can be compared
    /// with the real engine without changing ordinary verdict runs.
    pub fn with_fixed_window(mut self, window: u32) -> RunParams {
        self.window_ticks = round_up_60(window.max(MIN_WINDOW_TICKS));
        self.fixed_window = true;
        // Fixed-window mode intentionally has no convergence phase: its
        // complete budget is exactly warmup plus one requested window. Do
        // not retain the ordinary run's viability ceiling here, or a
        // diagnostic run continues for many extra windows after it has
        // already emitted its measurement.
        self.end_tick = self.warmup_ticks.saturating_add(self.window_ticks);
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
  -- (ext_len staggers within each lateral cluster of exits, extending
  -- further out each step). Explicitly chunk-generate this specific rig's own
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
  -- Power the BANK, not the extension head (RFC-072 Phase-2 forensics):
  -- ext_len staggers with cluster-local rig index (11 + 2*k — global
  -- manifest index until RFC-072 P2 unit 2), and with the substation
  -- anchored at the head its supply area ran out around rig 6 — the
  -- bank's legendary stack inserters sat unpowered, the extension
  -- filled, and every copy behind a far rig blocked at full_output
  -- (the ec150 composed strip: copies 6-11 dead at exactly -50%,
  -- while ec75's six rigs all fit in reach and PASSed). The bank
  -- spans t = ext_len-8 .. ext_len; center the substation on it.
  local bank_t = ext_len - 4
  local subx, suby = exit_x + fx * bank_t + lx * 4, exit_y + fy * bank_t + ly * 4
  local eeix, eeiy = exit_x + fx * bank_t + lx * 7, exit_y + fy * bank_t + ly * 7
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
/// Two rigs whose heads sit further apart along the lateral axis than
/// this can never touch — the item rig's one-sided halo is the 12-tile
/// jog plus the power island at jog tiles 15/18 (2×2 entities → reach
/// 19), the drain rig's is its lateral +7 EEI. Beyond it, restarting
/// the depth/extension ladder is safe; within it, the ladder staggers
/// exactly as before (#363/#357 invariants preserved per cluster).
const RIG_LATERAL_HALO: i32 = 21;

/// Assign each record within one band to a lateral CLUSTER, then hand
/// each cluster to `ladder` for slot assignment. `lat` is the record's
/// lateral coordinate; clusters break where consecutive (sorted)
/// records sit more than `RIG_LATERAL_HALO` apart.
fn assign_clustered(
    records: &[BoundaryRecord],
    mut idxs: Vec<usize>,
    lat: impl Fn(usize) -> i32,
    ladder: impl Fn(&[BoundaryRecord], &[usize], &mut [i32]),
    slots: &mut [i32],
) {
    idxs.sort_by_key(|&i| lat(i));
    let mut start = 0;
    for k in 1..=idxs.len() {
        if k == idxs.len() || lat(idxs[k]) - lat(idxs[k - 1]) > RIG_LATERAL_HALO {
            ladder(records, &idxs[start..k], slots);
            start = k;
        }
    }
}

fn feed_slots(records: &[BoundaryRecord]) -> Vec<i32> {
    // One ladder per (direction, flow-axis coordinate) BAND and lateral
    // CLUSTER; fluids FIRST within each cluster (PR #515 review finding:
    // a separate fluid counter staggered fluids only against each other,
    // so a fluid ug-run could land exactly on an item rig's jog row and
    // stack silently). Fluids take cluster slots 0..f-1 — their occupied
    // band (out-tiles 1..=2+dist, dist = 2+2*slot, max 2+2f) sits
    // strictly below the shallowest item band ([2+6f, 6+6f] for item
    // slot f), so no item jog row or chest bank can ever reach a fluid
    // column's tiles. Items keep their lateral-sorted relative order
    // (the #363 invariant), just offset by f.
    //
    // RFC-072 P2 unit 2 reshaped the ladder from ONE-PER-DIRECTION to
    // banded + clustered: a grid fixture has feed heads on INTERIOR
    // edges, where the old global ladder's depth (4 + 6·slot across
    // every same-direction feed in the manifest — 100+ tiles on a
    // 12-copy strip) cannot fit in any honest inter-strip clearance.
    // Records launched from different flow-axis coordinates never share
    // tiles (their rigs extend outward from different lines — the
    // near-parallel guard below refuses the one geometry where that
    // argument fails), and same-band records beyond RIG_LATERAL_HALO
    // can't reach each other's tiles, so neither needs a shared ladder.
    let mut by_band: BTreeMap<(u8, i32), Vec<usize>> = BTreeMap::new();
    for (i, rec) in records.iter().enumerate() {
        let into = rec.direction().vector();
        by_band
            .entry((rec.direction, rec.x * into.0 + rec.y * into.1))
            .or_default()
            .push(i);
    }
    let mut slots = vec![0i32; records.len()];
    for idxs in by_band.values().cloned() {
        let into = records[idxs[0]].direction().vector();
        let lateral = rot90(into);
        assign_clustered(
            records,
            idxs,
            |i| records[i].x * lateral.0 + records[i].y * lateral.1,
            |records, cluster, slots| {
                let lateral = rot90(records[cluster[0]].direction().vector());
                let mut cluster = cluster.to_vec();
                cluster.sort_by_key(|&i| {
                    (
                        !records[i].is_fluid,
                        records[i].x * lateral.0 + records[i].y * lateral.1,
                    )
                });
                for (slot, i) in cluster.into_iter().enumerate() {
                    slots[i] = slot as i32;
                }
            },
            &mut slots,
        );
    }
    // Near-parallel band guard: a feed rig extends OUTWARD (decreasing
    // flow coordinate) by its depth plus the ±2 bank overhang. Two
    // same-direction bands closer than the deeper band's reach, with
    // laterally overlapping spans, would interleave rigs the ladder
    // cannot separate — refuse loudly rather than stack entities
    // silently (the #357 failure mode).
    let bands: Vec<((u8, i32), &Vec<usize>)> =
        by_band.iter().map(|(k, v)| (*k, v)).collect();
    for a in 0..bands.len() {
        for b in (a + 1)..bands.len() {
            let ((dir_a, coord_a), idxs_a) = bands[a];
            let ((dir_b, coord_b), idxs_b) = bands[b];
            if dir_a != dir_b || coord_a == coord_b {
                continue;
            }
            // Rigs extend OUTWARD = toward decreasing flow coordinate,
            // so only the FARTHER band's rigs can reach the nearer one.
            let (near_coord, far_coord, far_idxs) = if coord_a < coord_b {
                (coord_a, coord_b, idxs_b)
            } else {
                (coord_b, coord_a, idxs_a)
            };
            let far_reach = far_idxs
                .iter()
                .map(|&i| 4 + 6 * slots[i] + 2)
                .max()
                .unwrap_or(0);
            let lateral = rot90(records[far_idxs[0]].direction().vector());
            let lat = |i: usize| records[i].x * lateral.0 + records[i].y * lateral.1;
            let span = |idxs: &[usize]| {
                idxs.iter().fold((i32::MAX, i32::MIN), |(lo, hi), &i| {
                    (lo.min(lat(i)), hi.max(lat(i)))
                })
            };
            let (amin, amax) = span(idxs_a);
            let (bmin, bmax) = span(idxs_b);
            let lateral_overlap = amin - RIG_LATERAL_HALO <= bmax + RIG_LATERAL_HALO
                && bmin - RIG_LATERAL_HALO <= amax + RIG_LATERAL_HALO;
            assert!(
                far_coord - near_coord > far_reach || !lateral_overlap,
                "feed bands at flow coords {near_coord} and {far_coord} (direction \
                 {dir_a}) are closer than the deeper band's {far_reach}-tile rig \
                 reach with overlapping lateral spans — the stagger cannot \
                 separate them; widen the clearance between these edges"
            );
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
        // Fluids hold the FRONT of their band+CLUSTER's ladder (see
        // `feed_slots`; per cluster since RFC-072 P2 unit 2 — six fluids
        // split across two lateral clusters legitimately pass, since caps
        // only merge when ADJACENT), so `slot` here is the fluid's rank 0..f-1 and
        // the staggered run length keeps every surface cap ≥2 tiles from
        // its neighbors while staying below every item band. The ug span
        // (gap = dist-1 = 1+2*slot) hits the game's 9-tile limit at
        // slot 4 — six or more fluid feeds on ONE boundary side would
        // need chained hops, which no manifest has ever required; fail
        // loudly rather than fabricate a rate.
        assert!(
            slot <= 4,
            "more than 5 fluid boundary feeds in one lateral cluster (slot {slot} for {}): \
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

/// Extension length per drain record — banded and clustered exactly
/// like `feed_slots` (RFC-072 P2 unit 2: the old global `11 + 2·idx`
/// grew with MANIFEST position, so an 18-exit grid's first strip would
/// push rigs ~45 tiles into the inter-strip clearance; exits beyond
/// RIG_LATERAL_HALO can't touch each other's banks, so they don't need
/// distinct extensions). Within a cluster the stagger is preserved:
/// two exits ≤4 tiles apart with EQUAL ext would land their ±2-lateral
/// chest banks on shared tiles (the #357 stacking class).
fn drain_ext_lens(records: &[BoundaryRecord]) -> Vec<i32> {
    let mut by_band: BTreeMap<(u8, i32), Vec<usize>> = BTreeMap::new();
    for (i, rec) in records.iter().enumerate() {
        let flow = rec.direction().vector();
        by_band
            .entry((rec.direction, rec.x * flow.0 + rec.y * flow.1))
            .or_default()
            .push(i);
    }
    let mut exts = vec![0i32; records.len()];
    for idxs in by_band.values().cloned() {
        let lateral = rot90(records[idxs[0]].direction().vector());
        assign_clustered(
            records,
            idxs,
            |i| records[i].x * lateral.0 + records[i].y * lateral.1,
            |records, cluster, exts| {
                let lateral = rot90(records[cluster[0]].direction().vector());
                let lat = |i: usize| records[i].x * lateral.0 + records[i].y * lateral.1;
                let mut cluster = cluster.to_vec();
                cluster.sort_by_key(|&i| lat(i));
                for (k, &i) in cluster.iter().enumerate() {
                    // Base 11 (was 5): the widened 9-position drain bank
                    // needs ext_len-8 >= 3 so every chest/inserter sits
                    // outside the layout. The 2-step stagger within a
                    // cluster keeps the old global scheme's SPACING for
                    // close exits (the old scheme indexed by manifest
                    // order, so a given pair's exact ext values can
                    // differ; kit parity is receipted, not assumed —
                    // ec22 re-measured under the clustered kit
                    // reproduces its old-kit report to three decimals,
                    // #731 round 1). KNOWN-IMPERFECT
                    // at close pitch, deliberately preserved (RFC-072 P2
                    // unit 2 audit): at pitch ≤ 8 a rig's ±2 bank tiles or
                    // its lateral +4..+8 power island can land on a
                    // neighbor's tiles, where create_entity fails per-tile
                    // and the over-provisioned 18-position bank absorbs
                    // the loss — ec23's pitch-1 exit pair measures PASS at
                    // −0.6% under exactly this degradation. A redesign
                    // (tip-anchored islands, flow-span staggering) would
                    // change verified kit geometry for every measured
                    // fixture; recorded as a harness follow-up instead.
                    exts[i] = 11 + 2 * k as i32;
                }
            },
            &mut exts,
        );
    }
    // Near-parallel band guard, mirroring `feed_slots`': drains extend
    // ALONG flow (increasing flow coordinate), so only the NEARER band
    // can reach the farther one.
    let bands: Vec<((u8, i32), &Vec<usize>)> =
        by_band.iter().map(|(k, v)| (*k, v)).collect();
    for a in 0..bands.len() {
        for b in (a + 1)..bands.len() {
            let ((dir_a, coord_a), idxs_a) = bands[a];
            let ((dir_b, coord_b), idxs_b) = bands[b];
            if dir_a != dir_b || coord_a == coord_b {
                continue;
            }
            let (near_coord, far_coord, near_idxs) = if coord_a < coord_b {
                (coord_a, coord_b, idxs_a)
            } else {
                (coord_b, coord_a, idxs_b)
            };
            let near_reach = near_idxs.iter().map(|&i| exts[i] + 1).max().unwrap_or(0);
            let lateral = rot90(records[near_idxs[0]].direction().vector());
            let lat = |i: usize| records[i].x * lateral.0 + records[i].y * lateral.1;
            let span = |idxs: &[usize]| {
                idxs.iter().fold((i32::MAX, i32::MIN), |(lo, hi), &i| {
                    (lo.min(lat(i)), hi.max(lat(i)))
                })
            };
            let (amin, amax) = span(idxs_a);
            let (bmin, bmax) = span(idxs_b);
            let lateral_overlap = amin - RIG_LATERAL_HALO <= bmax + RIG_LATERAL_HALO
                && bmin - RIG_LATERAL_HALO <= amax + RIG_LATERAL_HALO;
            assert!(
                far_coord - near_coord > near_reach || !lateral_overlap,
                "drain bands at flow coords {near_coord} and {far_coord} (direction \
                 {dir_a}) are closer than the nearer band's {near_reach}-tile rig \
                 reach with overlapping lateral spans — the stagger cannot \
                 separate them; widen the clearance between these edges"
            );
        }
    }
    exts
}

/// The item feed rig's occupied tiles, mirroring `add_feed` exactly:
/// the outward column, the 12-tile jog, the ±1/±2 chest+inserter bank
/// on jog tiles 10–12, and the substation/EEI (2×2 each) at jog 15/18.
pub(crate) fn feed_footprint(
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
    // 2×2 entities (substation, EEI) are CORNER-centred: `create_entity`
    // at integer (x, y) occupies tiles x-1..=x × y-1..=y (#731 round 2 —
    // the first cut claimed x..=x+1, one tile off toward the jog tip).
    for k in [15, 18] {
        let b = (corner.0 + neg_lateral.0 * k, corner.1 + neg_lateral.1 * k);
        for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
            tiles.insert((b.0 + dx, b.1 + dy));
        }
    }
    tiles
}

/// The fluid feed's occupied tiles: the whole outward column through
/// the infinity cap (out-tiles 1..=2+dist). Conservative — the ug gap
/// tiles hold no entity — but claiming them keeps the disjointness
/// argument independent of that detail.
pub(crate) fn fluid_feed_footprint(
    head: (i32, i32),
    into: (i32, i32),
    dist: i32,
) -> std::collections::HashSet<(i32, i32)> {
    let outward = neg(into);
    (1..=2 + dist)
        .map(|t| (head.0 + outward.0 * t, head.1 + outward.1 * t))
        .collect()
}

/// The drain rig's occupied tiles, mirroring `add_drain`: extension
/// column, ±1/±2 lateral bank over its last 9 tiles, substation/EEI
/// (2×2 each) at lateral +4/+7 of the bank center.
pub(crate) fn drain_footprint(
    head: (i32, i32),
    flow: (i32, i32),
    ext: i32,
) -> std::collections::HashSet<(i32, i32)> {
    let lateral = rot90(flow);
    let mut tiles = std::collections::HashSet::new();
    for t in 1..=ext {
        tiles.insert((head.0 + flow.0 * t, head.1 + flow.1 * t));
    }
    for t in (ext - 8)..=ext {
        for side in [-2, -1, 1, 2] {
            tiles.insert((
                head.0 + flow.0 * t + lateral.0 * side,
                head.1 + flow.1 * t + lateral.1 * side,
            ));
        }
    }
    // 2×2 entities are corner-centred: tiles x-1..=x × y-1..=y around
    // the integer position (#731 round 2).
    let bank_t = ext - 4;
    for off in [4, 7] {
        for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
            tiles.insert((
                head.0 + flow.0 * bank_t + lateral.0 * off + dx,
                head.1 + flow.1 * bank_t + lateral.1 * off + dy,
            ));
        }
    }
    tiles
}

/// Structural cross-type guard (RFC-072 P2 unit 2, #731 round 1): a
/// grid's inter-strip clearance hosts the upper strip's DRAIN rigs and
/// the lower strip's FEED rigs. The same-type ladders above can't see
/// each other, and the Lua-side kit audit only RECORDS an overlap — so
/// refuse at codegen if any feed-rig tile coincides with any drain-rig
/// tile. Same-type overlap inside a close-pitch drain cluster is the
/// documented known-imperfect case and is not judged here. KNOWN GAP
/// (#731 round 2): same-type rigs on PERPENDICULAR edges near a corner
/// (a north-edge feed's lateral jog vs a west-edge feed's outward
/// column) are checked by neither the band guards (same-direction
/// only) nor this one (cross-type only); no engine shape produces it
/// today (strips and grids feed north / drain south), and the
/// rig-vs-layout guard below is shape-agnostic. Extend this guard to
/// all-pairs rig disjointness before any composer emits such a corner.
fn assert_feed_drain_rigs_disjoint(
    inputs: &[BoundaryRecord],
    slots: &[i32],
    outputs: &[BoundaryRecord],
    exts: &[i32],
) {
    let mut drain_tiles: std::collections::HashMap<(i32, i32), &str> = Default::default();
    for (i, rec) in outputs.iter().enumerate() {
        for t in drain_footprint((rec.x, rec.y), rec.direction().vector(), exts[i]) {
            drain_tiles.insert(t, &rec.item);
        }
    }
    for (i, rec) in inputs.iter().enumerate() {
        let into = rec.direction().vector();
        let tiles = if rec.is_fluid {
            fluid_feed_footprint((rec.x, rec.y), into, 2 + 2 * slots[i])
        } else {
            feed_footprint((rec.x, rec.y), into, 4 + 6 * slots[i])
        };
        for t in tiles {
            if let Some(drain_item) = drain_tiles.get(&t) {
                panic!(
                    "feed rig for '{}' (head ({},{}), slot {}) and drain rig for '{drain_item}' \
                     share tile {t:?} — the clearance between these edges is too small for \
                     both rigs; widen it rather than let the kit stack entities",
                    rec.item, rec.x, rec.y, slots[i]
                );
            }
        }
    }
}

/// Every rig tile per record, in the manifest's layout frame.
fn rig_footprints(
    inputs: &[BoundaryRecord],
    slots: &[i32],
    outputs: &[BoundaryRecord],
    exts: &[i32],
) -> Vec<(String, std::collections::HashSet<(i32, i32)>)> {
    let mut rigs = Vec::new();
    for (i, rec) in inputs.iter().enumerate() {
        let into = rec.direction().vector();
        let tiles = if rec.is_fluid {
            fluid_feed_footprint((rec.x, rec.y), into, 2 + 2 * slots[i])
        } else {
            feed_footprint((rec.x, rec.y), into, 4 + 6 * slots[i])
        };
        rigs.push((format!("feed rig for '{}' at ({},{})", rec.item, rec.x, rec.y), tiles));
    }
    for (i, rec) in outputs.iter().enumerate() {
        let tiles = drain_footprint((rec.x, rec.y), rec.direction().vector(), exts[i]);
        rigs.push((format!("drain rig for '{}' at ({},{})", rec.item, rec.x, rec.y), tiles));
    }
    rigs
}

/// Structural rig-vs-LAYOUT guard (RFC-072 P2 unit 2, #731 round 1): a
/// rig launched from an INTERIOR boundary head (a grid's inter-strip
/// edges) extends into the bounding box, where the layout's own
/// entities may sit. `create_entity` on an occupied tile fails
/// silently for feed rigs (only drain belts log), so a too-small
/// clearance would ship as a starved strip with an empty kit_errors —
/// exactly the silent class this project refuses. Decode the blueprint
/// (the meter crate's parser), align frames via the record-integrity
/// contract (every boundary head must land on a decoded belt), and
/// refuse if any rig tile is occupied by the layout.
fn assert_rigs_clear_of_layout(
    bp: &str,
    inputs: &[BoundaryRecord],
    slots: &[i32],
    outputs: &[BoundaryRecord],
    exts: &[i32],
) {
    // A blueprint that does not decode is a REFUSAL in production
    // (#731 round 2): this guard exists to turn silent kit truncation
    // into a loud stop, and "the blueprint failed to parse" is inside
    // the class it backstops, not outside it. The one legitimate
    // non-decoding input is the unit tests' placeholder string
    // ("0eNBPFAKE"), which exercise other parts of the codegen — gated
    // on `cfg!(test)` so no production build can take the skip.
    let entities = match spaghettio_meter::blueprint_in::decode(bp) {
        Ok(e) => e,
        Err(err) if cfg!(test) => {
            eprintln!("rig-vs-layout guard skipped: placeholder blueprint ({err})");
            return;
        }
        Err(err) => panic!(
            "rig-vs-layout guard: the blueprint does not decode ({err}) — refusing to \
             place rigs blind rather than ship a kit that may sit on the layout"
        ),
    };
    let mut occupied: std::collections::HashSet<(i32, i32)> = Default::default();
    for e in &entities {
        let horizontal = matches!(
            e.direction,
            spaghettio_meter::blueprint_in::Dir::East | spaghettio_meter::blueprint_in::Dir::West
        );
        let (w, h) = spaghettio_meter::entity_data::footprint_oriented(&e.name, horizontal);
        for dx in 0..w as i32 {
            for dy in 0..h as i32 {
                occupied.insert((e.x + dx, e.y + dy));
            }
        }
    }
    // Frame alignment: the manifest's records and the blueprint's tiles
    // must share the layout frame — every boundary head sits ON a
    // layout belt by the record-integrity contract. A frame mismatch
    // would make this whole guard check the wrong tiles, so it is a
    // refusal, not a tolerance.
    for rec in inputs.iter().chain(outputs.iter()) {
        assert!(
            occupied.contains(&(rec.x, rec.y)),
            "boundary record for '{}' at ({},{}) lands on no decoded blueprint tile — \
             manifest and blueprint frames disagree; refusing to place rigs blind",
            rec.item, rec.x, rec.y
        );
    }
    assert_rigs_clear_of_occupied(&occupied, inputs, slots, outputs, exts);
}

/// The occupancy half of `assert_rigs_clear_of_layout`, split out so the
/// geometry is unit-testable without a blueprint.
fn assert_rigs_clear_of_occupied(
    occupied: &std::collections::HashSet<(i32, i32)>,
    inputs: &[BoundaryRecord],
    slots: &[i32],
    outputs: &[BoundaryRecord],
    exts: &[i32],
) {
    for (label, tiles) in rig_footprints(inputs, slots, outputs, exts) {
        if let Some(t) = tiles.iter().find(|t| occupied.contains(t)) {
            panic!(
                "{label} would place kit on layout tile {t:?} — the clearance around this \
                 boundary edge is smaller than the rig; feed placement failures are silent \
                 in-game, so refusing here instead"
            );
        }
    }
}

fn drain_call(out: &mut String, idx: usize, ext_len: i32, rec: &BoundaryRecord) {
    let flow = rec.direction().vector();
    let lateral = rot90(flow);
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
    let _ = writeln!(out, "local FIXED_WINDOW = {}", params.fixed_window);
    let _ = writeln!(out, "local WINDOW_ITEM_FLOOR = {WINDOW_ITEM_FLOOR}");
    let _ = writeln!(out, "local WINDOW_MIN_TICKS = {MIN_WINDOW_TICKS}");
    let _ = writeln!(
        out,
        "local WINDOW_TICK_CAP = {}",
        window_tick_cap(params.window_ticks)
    );
    let _ = writeln!(out, "local KEEP_ALIVE = {}", params.keep_alive);
    let _ = writeln!(
        out,
        "local PICKUP_TRACE_ONLY = {}",
        params.pickup_trace_only
    );
    let _ = writeln!(out, "local DROP_TRACE = {}", params.drop_trace);
    // Detailed belt-position and inserter admission telemetry is useful for
    // focused engine-vs-meter comparisons, but it is not part of an ordinary
    // production-rate run. Fixed-window runs are diagnostic by definition and
    // retain the rich snapshot used by their calibration workflow.
    let _ = writeln!(
        out,
        "local TRACE_FORENSICS = PICKUP_TRACE_ONLY or DROP_TRACE or FIXED_WINDOW"
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
    // Declared research productivity, per recipe. Emitted as a Lua table so
    // the scenario can check the sim's realized bonus against what the plan
    // and the meter assumed. Empty table when undeclared, which is every
    // manifest written before the axis existed.
    {
        let entries: Vec<String> = manifest
            .research_productivity
            .iter()
            .map(|(recipe, bonus)| format!("[\"{recipe}\"]={bonus}"))
            .collect();
        let _ = writeln!(
            out,
            "local DECLARED_PRODUCTIVITY = {{{}}}",
            entries.join(",")
        );
    }
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
  storage.drop_probes = {}
  storage.drop_event_trace = {}
  storage.drop_event_previous_held = {}
  storage.drop_event_previous_sample = {}
  storage.drop_event_inserters = nil
  storage.pickup_event_trace = {}
  storage.pickup_event_previous_held = {}
  storage.pickup_event_previous_item = {}
  storage.pickup_event_inserters = nil
  storage.pickup_event_inserter_seen = {}
  storage.pickup_event_warmup_refresh = false
  storage.drop_physics_probe = {}
  storage.curve_sideload_probe = nil
  storage.fixed_window_state_dumped = false
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
  -- Recipe-productivity parity (RFC-064 item 7, pinned 2026-08-23):
  -- research_all above also grants the per-recipe productivity
  -- technologies (+10%/level: plastic bar, processing unit, ...), while
  -- the engine plans and the meter measures at the DECLARED value -- 0
  -- for every undeclared recipe. The measurement-time check detected
  -- exactly this and excluded 9/35 rows of the first calibration
  -- campaign (every plastic-bearing fixture). The bonus itself is
  -- derived, not settable, but the TECHS are settable: un-research every
  -- technology whose productivity effects all target undeclared recipes,
  -- pinning realized to declared -- the same world-matches-declared-axes
  -- principle as the two assignments above (#370/#385 option A). A tech
  -- mixing declared and undeclared recipes cannot satisfy both and is
  -- recorded as a kit error rather than silently half-pinned (none exist
  -- in vanilla or Space Age, where these techs are single-recipe).
  -- Declared-nonzero recipes keep their techs; the measurement-time
  -- check still verifies their realized value independently.
  local pinned_recipes = {}
  for _, tech in pairs(force.technologies) do
    local okp, effects = pcall(function() return tech.prototype.effects end)
    if okp and effects ~= nil then
      local undeclared, hits_declared, other_effects = {}, false, false
      for _, eff in pairs(effects) do
        if eff.type == "change-recipe-productivity" then
          if (DECLARED_PRODUCTIVITY[eff.recipe] or 0) ~= 0 then
            hits_declared = true
          else
            table.insert(undeclared, eff.recipe)
          end
        else
          -- ANY other effect (a recipe unlock, a force bonus) makes the
          -- tech unsafe to un-research: pinning productivity must never
          -- lock a recipe or strip an unrelated bonus. Vanilla and Space
          -- Age productivity techs are pure, so this arm firing at all is
          -- itself reportable.
          other_effects = true
        end
      end
      if #undeclared > 0 then
        if hits_declared or other_effects then
          table.insert(storage.kit_errors,
            "recipe-productivity parity: tech '" .. tech.name
            .. "' cannot be pinned ("
            .. (hits_declared and "mixes declared and undeclared productivity recipes"
                or "carries non-productivity effects; un-researching would strip them")
            .. ")")
        else
          local okw = pcall(function() tech.researched = false end)
          if not okw then
            table.insert(storage.kit_errors,
              "recipe-productivity parity: could not un-research '" .. tech.name .. "'")
          end
          -- Belt-and-braces for multi-level techs, in its OWN pcall
          -- (#714 review): a level-assignment fault must not be reported
          -- as an un-research failure — the read-back below and the
          -- measurement-time check catch any residual bonus either way.
          pcall(function() tech.level = tech.prototype.level end)
          for _, r in pairs(undeclared) do pinned_recipes[r] = true end
        end
      end
    end
  end
  -- Same self-audit as the two parity assignments above (#376 review
  -- finding: a verification channel nobody checks is not one): read the
  -- realized bonus back rather than trusting the write. The
  -- measurement-time parity check remains as the independent verifier,
  -- scoped to what the layout actually crafts.
  for name, _ in pairs(pinned_recipes) do
    local recipe = force.recipes[name]
    if recipe ~= nil then
      local okb, realized = pcall(function() return recipe.productivity_bonus end)
      if okb and realized ~= nil and math.abs(realized) > 1e-6 then
        table.insert(storage.kit_errors,
          "recipe-productivity parity pin did not take: '" .. name
          .. "' realized " .. realized .. " after un-research")
      end
    end
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
    let drain_exts = drain_ext_lens(&manifest.boundary_outputs);
    assert_feed_drain_rigs_disjoint(
        &manifest.boundary_inputs,
        &feed_depth_slots,
        &manifest.boundary_outputs,
        &drain_exts,
    );
    assert_rigs_clear_of_layout(
        bp,
        &manifest.boundary_inputs,
        &feed_depth_slots,
        &manifest.boundary_outputs,
        &drain_exts,
    );
    for (idx, rec) in manifest.boundary_outputs.iter().enumerate() {
        drain_call(&mut out, idx, drain_exts[idx], rec);
    }
    for (item, x, y) in &manifest.surplus_exits {
        let _ = writeln!(
            out,
            "  add_fluid_void(s, force, {x} - LX0 + storage.offx, {y} - LY0 + storage.offy, \"{item}\")",
    );
    }

    out.push_str(
        r#"  -- Controlled belt admission probe.  This is deliberately a temporary
  -- engine-side fixture rather than a deduction from the production layout:
  -- force_insert_at gives us known item positions, and can_insert_at tells
  -- us exactly which gaps the game considers admissible for a new item.
  if DROP_TRACE or FIXED_WINDOW then
    do
    local probe_position = {x = storage.offx + DIMS_X + 8,
                            y = storage.offy + DIMS_Y + 8}
    local probe = s.create_entity{name = "express-transport-belt",
                                  position = probe_position,
                                  direction = defines.direction.east,
                                  force = force}
    if probe ~= nil then
      local line = probe.get_transport_line(1)
      local cases = {
        {label = "empty", positions = {}},
        {label = "one_left", positions = {0.4375}},
        {label = "one_target", positions = {0.5}},
        {label = "two_around_target", positions = {0.375, 0.625}},
        {label = "gap_at_target", positions = {0.25, 0.75}},
        {label = "full_quarter_grid", positions = {0, 0.25, 0.5, 0.75}}
      }
      for _, case in pairs(cases) do
        line.clear()
        local insert_results = {}
        for _, p in pairs(case.positions) do
          local ok, inserted = pcall(function()
            return line.force_insert_at(p, {name = "iron-plate", count = 1}, 1)
          end)
          table.insert(insert_results, {position = p,
                                        result = ok and (inserted and "yes" or "no") or "error"})
        end
        local checks = {}
        for _, p in pairs({0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1}) do
          local ok, can_insert = pcall(function() return line.can_insert_at(p) end)
          table.insert(checks, {position = p,
                                can_insert = ok and (can_insert and "yes" or "no") or "error"})
        end
        local detailed = {}
        for _, entry in pairs(line.get_detailed_contents()) do
          local name, count = nil, nil
          local stack_ok, stack = pcall(function() return entry.stack end)
          if stack_ok and stack ~= nil then
            local name_ok, stack_name = pcall(function() return stack.name end)
            local count_ok, stack_count = pcall(function() return stack.count end)
            if name_ok then name = stack_name end
            if count_ok then count = stack_count end
          end
          table.insert(detailed, {name = name, count = count,
                                  position = entry.position,
                                  unique_id = entry.unique_id})
        end
        table.insert(storage.drop_physics_probe, {
          label = case.label,
          line_length = line.line_length,
          total_segment_length = line.total_segment_length,
          insert_results = insert_results,
          checks = checks,
          detailed = detailed
        })
      end
      probe.destroy()
    end
    end
  end
"#,
    );

    out.push_str(
        r#"  -- Isolated curve-to-sideload probe. One feeder turns into the
  -- target from the side while a second feeder enters its back. This is
  -- the topology in which the production meter still has a lane-phase
  -- mismatch. Keep it outside the pasted layout and destroy it after a few
  -- belt phases; it must never participate in factory measurement.
  if DROP_TRACE or FIXED_WINDOW then
    do
    local px = storage.offx + DIMS_X + 16
    local py = storage.offy + DIMS_Y + 16
    local probe_entities = {}
    local function make(pos, direction)
      local b = s.create_entity{name = "express-transport-belt", position = pos,
                                direction = direction, force = force}
      if b then table.insert(probe_entities, b) end
      return b
    end
    local curve_source = make({px, py}, defines.direction.east)
    local curve = make({px + 1, py}, defines.direction.south)
    local target = make({px + 1, py + 1}, defines.direction.west)
    local back = make({px + 2, py + 1}, defines.direction.west)
    if curve_source and curve and target and back then
      local function fill(b, name)
        local line = b.get_transport_line(1)
        for _, position in pairs({0, 0.25, 0.5, 0.75}) do
          line.force_insert_at(position, {name = name, count = 1}, 1)
        end
      end
      fill(curve_source, "copper-cable")
      fill(back, "iron-plate")
      storage.curve_sideload_probe = {
        sample_ticks = {1, 2, 3, 4, 5, 10, 30}, next_sample = 1,
        curve_source = curve_source, curve = curve, target = target,
        back = back, entities = probe_entities
      }
    else
      for _, b in pairs(probe_entities) do if b.valid then b.destroy() end end
    end
    end
  end
"#,
    );

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

-- Sentinel for a JSON `null` in the belts array. Lua cannot hold a
-- `nil` in the middle (or at the tail) of an otherwise-sequential table --
-- assigning `nil` just removes the key, so `helpers.table_to_json` would
-- see a shorter array border and silently emit a 6-element tuple instead
-- of 7 for a plain (non-underground) belt. There is no documented null
-- sentinel `table_to_json` recognizes (checked against the Factorio forums
-- and the 2.0 LuaHelpers docs), so this dump builds the JSON normally with
-- this unique placeholder string in `ug_type`'s slot, then string-replaces
-- the quoted placeholder with a literal `null` in the finished JSON text.
-- The placeholder can never collide with a real value in this array
-- (item/entity names never contain it).
local SIM_STATE_NULL = "__spaghettio_sim_state_null__"
-- Periodic runtime evidence for the belt-drop discrepancy. The final
-- snapshot below tells us what a held inserter looked like once; this sample
-- keeps the destination's continuous insertion window visible across the
-- measurement, including the exact point and nearby offsets. Counts are
-- numeric because helpers.table_to_json may omit false-valued fields.
local DROP_PROBE_OFFSETS = {-2, -1.5, -1, -0.75, -0.5, -0.25, -0.125,
                            0, 0.125, 0.25, 0.5, 0.75, 1, 1.5, 2}
local function line_local_position(segment_position)
  -- get_item_insert_specification returns a coordinate on the connected
  -- segment, while can_insert_at consumes the coordinate on this line's
  -- one-tile local fragment.
  return segment_position - math.floor(segment_position)
end
local function sample_drop_probes(s)
  for _, i in pairs(s.find_entities_filtered{type = "inserter"}) do
    local target = i.drop_target
    if target ~= nil and target.valid then
      local ok, line_no, position = pcall(function()
        return target.get_item_insert_specification(i.drop_position)
      end)
      if ok and line_no ~= nil then
        local local_position = line_local_position(position)
        local rec = storage.drop_probes[i.unit_number]
        if rec == nil then
          rec = {unit_number = i.unit_number, samples = 0, held_samples = 0,
                 statuses = {}, segment_checks = {},
                 local_checks = {{position = local_position,
                                  yes = 0, no = 0, error = 0}}}
          for _, offset in pairs(DROP_PROBE_OFFSETS) do
            table.insert(rec.segment_checks, {offset = offset, yes = 0, no = 0, error = 0})
          end
          storage.drop_probes[i.unit_number] = rec
        end
        rec.samples = rec.samples + 1
        rec.line = line_no
        rec.position = position
        rec.local_position = local_position
        local map_ok, map_position = pcall(function()
          return target.get_line_item_position(line_no, position)
        end)
        if map_ok then
          rec.map_position = {x = map_position.x, y = map_position.y}
        end
        local status = stn(i.status)
        rec.statuses[status] = (rec.statuses[status] or 0) + 1
        local held_ok, held = pcall(function() return i.held_stack end)
        if held_ok and held ~= nil and held.valid_for_read then
          rec.held_samples = rec.held_samples + 1
        end
        local line = target.get_transport_line(line_no)
        rec.line_length = line.line_length
        rec.total_segment_length = line.total_segment_length
        for n, offset in pairs(DROP_PROBE_OFFSETS) do
          local check_ok, can_insert = pcall(function()
            return line.can_insert_at(position + offset)
          end)
          local check = rec.segment_checks[n]
          if not check_ok then
            check.error = check.error + 1
          elseif can_insert then
            check.yes = check.yes + 1
          else
            check.no = check.no + 1
          end
        end
        local check_ok, can_insert = pcall(function()
          return line.can_insert_at(local_position)
        end)
        local check = rec.local_checks[1]
        check.position = local_position
        if not check_ok then
          check.error = check.error + 1
        elseif can_insert then
          check.yes = check.yes + 1
        else
          check.no = check.no + 1
        end
      end
    end
  end
end

-- Tick-synchronised drop evidence. The older drop_probes channel is useful
-- for aggregate occupancy, but samples every 60 ticks and cannot tell a
-- successful deposit from a nearby free window. This channel records only
-- state transitions, so it can run every tick without producing a complete
-- copy of every inserter state.
local DROP_EVENT_LIMIT = 512
local DROP_EVENT_NEIGHBOUR_RADIUS = 2.5

local function drop_line_snapshot(target, line_no, position)
  local ok, line = pcall(function() return target.get_transport_line(line_no) end)
  if not ok or line == nil then return nil end
  local detailed_ok, detailed = pcall(function() return line.get_detailed_contents() end)
  if not detailed_ok or type(detailed) ~= "table" then return nil end
  local nearby = {}
  for _, entry in pairs(detailed) do
    if math.abs(entry.position - position) <= DROP_EVENT_NEIGHBOUR_RADIUS then
      local name, count = nil, nil
      local stack_ok, stack = pcall(function() return entry.stack end)
      if stack_ok and stack ~= nil then
        local name_ok, stack_name = pcall(function() return stack.name end)
        local count_ok, stack_count = pcall(function() return stack.count end)
        if name_ok then name = stack_name end
        if count_ok then count = stack_count end
      end
      table.insert(nearby, {name = name, count = count,
                             position = entry.position,
                             unique_id = entry.unique_id})
    end
  end
  table.sort(nearby, function(a, b) return a.position < b.position end)
  return nearby
end

local function sample_drop_events(s)
  if storage.drop_event_inserters == nil or #storage.drop_event_inserters == 0 then
    storage.drop_event_inserters = {}
    for _, candidate in pairs(s.find_entities_filtered{type = "inserter"}) do
      local target = candidate.drop_target
      if target ~= nil and target.valid
          and (target.type == "transport-belt"
            or target.type == "underground-belt"
            or target.type == "splitter") then
        table.insert(storage.drop_event_inserters, candidate)
      end
    end
  end
  for _, i in pairs(storage.drop_event_inserters) do
    if i.valid then
      local target = i.drop_target
      local held_ok, held_stack = pcall(function() return i.held_stack end)
      local held = 0
      if held_ok and held_stack ~= nil and held_stack.valid_for_read then
        held = held_stack.count
      end
      local previous = storage.drop_event_previous_held[i.unit_number]
      storage.drop_event_previous_held[i.unit_number] = held
      if target == nil or not target.valid then goto continue end

      local spec_ok, line_no, position = pcall(function()
        return target.get_item_insert_specification(i.drop_position)
      end)
      if not spec_ok or line_no == nil then goto continue end
      local line_ok, line = pcall(function() return target.get_transport_line(line_no) end)
      if not line_ok or line == nil then goto continue end
      local can_ok, can_insert = pcall(function() return line.can_insert_at(position) end)
      local can_state = "error"
      if can_ok then can_state = can_insert and "yes" or "no" end
      -- get_item_insert_specification returns a coordinate along the
      -- connected segment.  can_insert_at on the line object expects that
      -- line's local coordinate; for the belt lines used here this is the
      -- within-tile fractional part.
      local local_position = line_local_position(position)
      local local_can_ok, local_can_insert = pcall(function()
        return line.can_insert_at(local_position)
      end)
      local local_can_state = "error"
      if local_can_ok then local_can_state = local_can_insert and "yes" or "no" end
      local status = stn(i.status)
      local rec = storage.drop_event_trace[i.unit_number]
      if rec == nil then
        rec = {unit_number = i.unit_number, samples = 0, held_ticks = 0,
               blocked_ticks = 0, accepted_items = 0, blocked_events = 0,
               events = {}, events_truncated = 0}
        storage.drop_event_trace[i.unit_number] = rec
      end
      rec.samples = rec.samples + 1
      if held > 0 then rec.held_ticks = rec.held_ticks + 1 end
      if held > 0 and local_can_ok and local_can_insert then
        rec.can_insert_yes = (rec.can_insert_yes or 0) + 1
      elseif held > 0 and local_can_ok then
        rec.can_insert_no = (rec.can_insert_no or 0) + 1
      end

      -- A decrease in held_stack is the engine-side evidence that at least
      -- one item was accepted during the preceding tick. A held stack that
      -- remains unchanged while the inserter reports destination-space wait
      -- is the corresponding blocked event.
      local accepted = previous ~= nil and previous > held and previous - held or 0
      local blocked = previous ~= nil and previous > 0 and previous == held
                    and status == "waiting_for_space_in_destination"
      local previous_sample = storage.drop_event_previous_sample[i.unit_number]
      local current_sample = {
        tick = game.tick,
        held = held,
        status = status,
        line = line_no,
        position = position,
        local_position = local_position,
        line_length = line.line_length,
        total_segment_length = line.total_segment_length,
        raw_can_insert = can_state,
        can_insert = local_can_state
      }
      if accepted > 0 or blocked then
        if accepted > 0 then rec.accepted_items = rec.accepted_items + accepted end
        if blocked then
          rec.blocked_events = rec.blocked_events + 1
          rec.blocked_ticks = rec.blocked_ticks + 1
        end
        if #rec.events < DROP_EVENT_LIMIT then
          local after = {
            tick = current_sample.tick,
            held = current_sample.held,
            status = current_sample.status,
            line = current_sample.line,
            position = current_sample.position,
            local_position = current_sample.local_position,
            line_length = current_sample.line_length,
            total_segment_length = current_sample.total_segment_length,
            raw_can_insert = current_sample.raw_can_insert,
            can_insert = current_sample.can_insert,
            nearby = drop_line_snapshot(target, line_no, position)
          }
          table.insert(rec.events, {
            tick = game.tick,
            kind = accepted > 0 and "accepted" or "blocked",
            accepted_items = accepted,
            held = held,
            previous_held = previous,
            status = status,
            line = line_no,
            position = position,
            local_position = local_position,
            line_length = line.line_length,
            total_segment_length = line.total_segment_length,
            raw_can_insert = can_state,
            can_insert = local_can_state,
            before = previous_sample,
            after = after
          })
        else
          rec.events_truncated = rec.events_truncated + 1
        end
      end
      storage.drop_event_previous_sample[i.unit_number] = current_sample
    end
    ::continue::
  end
end

-- Tick-synchronised pickup evidence for the opposite side of the
-- meter/sim comparison.  For a belt -> machine inserter, a rise in the
-- held stack means the inserter picked an item from the belt; a fall means
-- it delivered an item into the machine.  Keep complete counters and only
-- cap the forensic transition list, matching drop_event_trace.
local PICKUP_EVENT_LIMIT = 512

local function pickup_machine_recipe(machine)
  local ok, recipe = pcall(function() return machine.get_recipe() end)
  if not ok or recipe == nil then return nil end
  local name_ok, name = pcall(function() return recipe.name end)
  return name_ok and name or nil
end

local function pickup_is_transport(entity)
  if entity == nil or not entity.valid then return false end
  local name = entity.name or ""
  return name == "splitter"
      or string.find(name, "transport-belt", 1, true) ~= nil
      or string.find(name, "underground-belt", 1, true) ~= nil
end

local function pickup_is_machine(entity)
  if entity == nil or not entity.valid then return false end
  local name = entity.name or ""
  -- Chemical plants, oil refineries, centrifuges, and the other recipe
  -- buildings all expose the Factorio prototype type
  -- "assembling-machine". Match that type rather than enumerating names;
  -- the name fallbacks preserve compatibility with modded prototypes that
  -- expose a more specific runtime type.
  return entity.type == "assembling-machine" or entity.type == "furnace"
      or string.find(name, "assembling-machine", 1, true) ~= nil
      or string.find(name, "furnace", 1, true) ~= nil
end

local function sample_pickup_events(s, sample_tick)
  -- Resolve the belt-to-machine population once.  The trace is sampled on
  -- every measurement tick, and re-running a surface-wide inserter query at
  -- that cadence made a full red replay CPU-bound before its steady window.
  -- Keep the population as LuaEntity references, just as drop_event_trace
  -- does; validity is checked below so blueprint cleanup remains safe.
  -- Blueprint revival can leave pickup targets unresolved during the first
  -- tick. Build the initial population once, then refresh it at the warmup
  -- boundary so late-attached machine targets are included without paying a
  -- surface-wide query on every pickup sample.
  local warmup_refresh = sample_tick >= WARMUP_TICKS
      and not storage.pickup_event_warmup_refresh
  if storage.pickup_event_inserters == nil or warmup_refresh then
    storage.pickup_event_inserters = storage.pickup_event_inserters or {}
    storage.pickup_event_inserter_seen = storage.pickup_event_inserter_seen or {}
    for _, candidate in pairs(s.find_entities_filtered{type = "inserter"}) do
      local pickup_target = candidate.pickup_target
      local drop_target = candidate.drop_target
      if pickup_target ~= nil and pickup_target.valid
          and drop_target ~= nil and drop_target.valid
          and pickup_is_transport(pickup_target)
          and pickup_is_machine(drop_target)
          and not storage.pickup_event_inserter_seen[candidate.unit_number] then
        table.insert(storage.pickup_event_inserters, candidate)
        storage.pickup_event_inserter_seen[candidate.unit_number] = true
      end
    end
    if warmup_refresh then storage.pickup_event_warmup_refresh = true end
  end
  for _, i in pairs(storage.pickup_event_inserters) do
    if not i.valid then goto continue end
    local pickup_target = i.pickup_target
    local drop_target = i.drop_target
    if pickup_target == nil or not pickup_target.valid
        or drop_target == nil or not drop_target.valid
        or not pickup_is_transport(pickup_target)
        or not pickup_is_machine(drop_target) then
      goto continue
    end

    local held_ok, held_stack = pcall(function() return i.held_stack end)
    local held = 0
    if held_ok and held_stack ~= nil and held_stack.valid_for_read then
      held = held_stack.count
    end
    local held_item = nil
    if held_ok and held_stack ~= nil and held_stack.valid_for_read then
      held_item = held_stack.name
    end
    local previous = storage.pickup_event_previous_held[i.unit_number]
    local previous_item = storage.pickup_event_previous_item[i.unit_number]
    storage.pickup_event_previous_held[i.unit_number] = held
    storage.pickup_event_previous_item[i.unit_number] = held_item

    local rec = storage.pickup_event_trace[i.unit_number]
    if rec == nil then
      rec = {unit_number = i.unit_number, samples = 0, held_ticks = 0,
             picked_items = 0, delivered_items = 0, pickup_events = 0,
             delivery_events = 0, events = {}, events_truncated = 0,
             measurement_picked_items = 0, measurement_delivered_items = 0,
             items = {},
             machine_recipe = pickup_machine_recipe(drop_target),
             pickup_target = pickup_target.name,
             drop_target = drop_target.name,
             pickup_x = pickup_target.position.x,
             pickup_y = pickup_target.position.y,
             machine_x = drop_target.position.x,
             machine_y = drop_target.position.y}
      storage.pickup_event_trace[i.unit_number] = rec
    end
    if sample_tick >= WARMUP_TICKS and rec.measurement_start_picked == nil then
      rec.measurement_start_picked = rec.picked_items
      rec.measurement_start_delivered = rec.delivered_items
    end
    rec.samples = rec.samples + 1
    if held > 0 then rec.held_ticks = rec.held_ticks + 1 end
    local picked = previous ~= nil and held > previous and held - previous or 0
    local delivered = previous ~= nil and previous > held and previous - held or 0
    local event_item = picked > 0 and held_item or previous_item
    local item_rec = nil
    if event_item ~= nil and (picked > 0 or delivered > 0) then
      item_rec = rec.items[event_item]
      if item_rec == nil then
        item_rec = {picked_items = 0, delivered_items = 0}
        rec.items[event_item] = item_rec
      end
      -- Establish the item baseline before this sample is added, just like
      -- the record-level baseline above. Initialising it after the update
      -- and subtracting only this transfer made the first post-warmup item
      -- disagree with the aggregate by one event.
      if sample_tick >= WARMUP_TICKS and item_rec.measurement_start_picked == nil then
        item_rec.measurement_start_picked = item_rec.picked_items
        item_rec.measurement_start_delivered = item_rec.delivered_items
      end
    end
    if picked > 0 then
      rec.picked_items = rec.picked_items + picked
      rec.pickup_events = rec.pickup_events + 1
    end
    if delivered > 0 then
      rec.delivered_items = rec.delivered_items + delivered
      rec.delivery_events = rec.delivery_events + 1
    end
    if rec.measurement_start_picked ~= nil then
      rec.measurement_picked_items = rec.picked_items - rec.measurement_start_picked
      rec.measurement_delivered_items = rec.delivered_items - rec.measurement_start_delivered
    end
    if item_rec ~= nil then
      item_rec.picked_items = item_rec.picked_items + picked
      item_rec.delivered_items = item_rec.delivered_items + delivered
      if item_rec.measurement_start_picked ~= nil then
        item_rec.measurement_picked_items = item_rec.picked_items - item_rec.measurement_start_picked
        item_rec.measurement_delivered_items = item_rec.delivered_items - item_rec.measurement_start_delivered
      end
    end
    if picked > 0 or delivered > 0 then
      if #rec.events < PICKUP_EVENT_LIMIT then
        table.insert(rec.events, {
          tick = game.tick,
          kind = picked > 0 and "picked" or "delivered",
          picked_items = picked,
          delivered_items = delivered,
          item = event_item,
          held = held,
          previous_held = previous,
          status = stn(i.status)
        })
      else
        rec.events_truncated = rec.events_truncated + 1
      end
    end
    ::continue::
  end
end

local function dump_sim_state(s)
  local belts, belt_positions, machines, inserters, inserter_trace, pipes = {}, {}, {}, {}, {}, {}
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

      if TRACE_FORENSICS then
        -- `get_detailed_contents()` is the game's continuous line position
        -- view. Keep this in a separate additive channel: the older `belts`
        -- shape is consumed by existing forensic scripts and only carries
        -- compressed counts.
        local detailed_ok, detailed = pcall(function() return tl.get_detailed_contents() end)
        if detailed_ok and type(detailed) == "table" then
          local positions = {}
          for _, entry in pairs(detailed) do
            local item_map_ok, item_map = pcall(function()
              -- LuaTransportLine exposes this one-argument form; the
              -- entity form below is the one that also needs a line index.
              return tl.get_line_item_position(entry.position)
            end)
            local item_map_position = nil
            if item_map_ok then
              item_map_position = {x = item_map.x, y = item_map.y}
            end
            local name, count = nil, nil
            local stack_ok, stack = pcall(function() return entry.stack end)
            if stack_ok and stack ~= nil then
              local name_ok, stack_name = pcall(function() return stack.name end)
              local count_ok, stack_count = pcall(function() return stack.count end)
              if name_ok then name = stack_name end
              if count_ok then count = stack_count end
            end
            table.insert(positions, {
              name = name,
              count = count,
              position = entry.position,
              map_position = item_map_position
            })
          end
          table.insert(belt_positions, {
            x = math.floor(b.position.x - storage.offx) + LX0,
            y = math.floor(b.position.y - storage.offy) + LY0,
            lane = li,
            items = positions
          })
        end
      end
    end
    -- Name + direction (belts never carried these, unlike pipes which got
    -- them for the same reason in #364 a few lines below) and, for
    -- underground belts, which end of the pair this is. `direction` is the
    -- raw 2.0 16-way `defines.direction` integer, unremapped (0=north,
    -- 4=east, 8=south, 12=west) -- see docs/sim-harness.md for the
    -- encoding note. Every belt is now emitted, including `n == 0`: an
    -- empty belt is the primary localization signal for a dried-up lane,
    -- and the old `if n > 0` guard hid exactly those tiles.
    local ug_type = SIM_STATE_NULL
    if b.type == "underground-belt" then
      ug_type = b.belt_to_ground_type
    end
    table.insert(belts, {math.floor(b.position.x - storage.offx) + LX0,
                         math.floor(b.position.y - storage.offy) + LY0, n, det,
                         b.name, b.direction, ug_type})
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

    if TRACE_FORENSICS then
      -- Diagnostic channel for the meter/sim belt-drop discrepancy. Keep the
      -- legacy three-field `inserters` census above stable; this richer record
      -- is deliberately additive and converts every Lua object to plain data
      -- before JSON serialization. Positions include both raw world values and
      -- the layout coordinates used by the other state-dump sections.
      local function point_record(p)
        if p == nil then return nil end
        return {
          world = {x = p.x, y = p.y},
          layout = {x = math.floor(p.x - storage.offx) + LX0,
                    y = math.floor(p.y - storage.offy) + LY0}
        }
      end
      local function read_point(get)
        local ok, p = pcall(get)
        if not ok then return nil end
        return point_record(p)
      end
      local function target_record(get)
        local ok, target = pcall(get)
        if not ok or target == nil or not target.valid then return nil end
        return {
          name = target.name,
          position = point_record(target.position)
        }
      end
      local held = nil
      local held_ok, stack = pcall(function() return i.held_stack end)
      if held_ok and stack ~= nil then
        local readable = pcall(function() return stack.valid_for_read end)
        if readable and stack.valid_for_read then
          held = {name = stack.name, count = stack.count}
        end
      end
      local drop_specification = nil
      local spec_ok, spec_line, spec_position = pcall(function()
        local target = i.drop_target
        if target == nil then return nil, nil end
        return target.get_item_insert_specification(i.drop_position)
      end)
      if spec_ok and spec_line ~= nil then
        local segment_checks = {}
        local target = i.drop_target
        local line = target.get_transport_line(spec_line)
        local map_ok, map_position = pcall(function()
          return target.get_line_item_position(spec_line, spec_position)
        end)
        for _, offset in pairs({-0.5, -0.25, -0.125, 0, 0.125, 0.25, 0.5}) do
          local check_ok, can_insert = pcall(function()
            return line.can_insert_at(spec_position + offset)
          end)
          local result = "error"
          if check_ok then
            result = can_insert and "yes" or "no"
          end
          table.insert(segment_checks, {offset = offset,
                                can_insert = result,
                                map_position = (function()
                                  local ok, p = pcall(function()
                                    return target.get_line_item_position(spec_line,
                                                                          spec_position + offset)
                                  end)
                                  if not ok then return nil end
                                  return point_record(p)
                                end)()})
        end
        local local_position = line_local_position(spec_position)
        local check_ok, can_insert = pcall(function()
          return line.can_insert_at(local_position)
        end)
        local result = "error"
        if check_ok then
          result = can_insert and "yes" or "no"
        end
        local local_checks = {{position = local_position, can_insert = result}}
        drop_specification = {line = spec_line, position = spec_position,
                              local_position = local_position,
                              line_length = line.line_length,
                              total_segment_length = line.total_segment_length,
                              map_position = map_ok and point_record(map_position) or nil,
                              segment_position_checks = segment_checks,
                              local_can_insert_checks = local_checks}
      end
      table.insert(inserter_trace, {
        name = i.name,
        position = point_record(i.position),
        status = stn(i.status),
        held_stack = held,
        held_stack_position = read_point(function() return i.held_stack_position end),
        pickup_position = read_point(function() return i.pickup_position end),
        drop_position = read_point(function() return i.drop_position end),
        drop_specification = drop_specification,
        pickup_target = target_record(function() return i.pickup_target end),
        drop_target = target_record(function() return i.drop_target end),
        drop_probe = storage.drop_probes[i.unit_number]
      })
    end
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
  -- The live traces are keyed by inserter unit number for O(1) updates, but
  -- Factorio's JSON helper treats a sparse numeric-keyed table as an empty
  -- object.  Materialise them as stable arrays before serialization or the
  -- forensic channels silently disappear from sim-state.json.
  local function trace_values(trace)
    local values = {}
    for _, rec in pairs(trace) do table.insert(values, rec) end
    table.sort(values, function(a, b)
      return (a.unit_number or 0) < (b.unit_number or 0)
    end)
    return values
  end
  local sim_state_json = helpers.table_to_json{
    offx = storage.offx, offy = storage.offy, fed = storage.fed_total,
    belts = belts, belt_positions = belt_positions,
    machines = machines, inserters = inserters,
    inserter_trace = inserter_trace, drop_probes = trace_values(storage.drop_probes),
    drop_event_inserter_count = storage.drop_event_inserters and #storage.drop_event_inserters or 0,
    pickup_event_inserter_count = storage.pickup_event_inserters and #storage.pickup_event_inserters or 0,
    drop_event_trace = trace_values(storage.drop_event_trace),
    pickup_event_trace = trace_values(storage.pickup_event_trace),
    drop_physics_probe = storage.drop_physics_probe, pipes = pipes,
    ugs = ugs, splitters = splitters, chests = chests}
  -- Convert the belts' ug_type sentinel (see SIM_STATE_NULL above) to a
  -- real JSON null now that the whole structure has been serialized.
  -- Anchored to an array tail (`,"<sentinel>"]`) so only the ug_type slot
  -- can ever match, whatever item or entity names a mod might introduce.
  sim_state_json = sim_state_json:gsub(',%s*"' .. SIM_STATE_NULL .. '"%s*%]', ",null]")
  helpers.write_file("sim-state.json", sim_state_json, false)
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
  -- In fixed-window mode the first closed measurement window is END_TICK,
  -- so its diagnostic snapshot is already the final snapshot. Reusing it
  -- avoids a duplicate full belt/inserter scan and file write on that tick.
  if not (FIXED_WINDOW and storage.fixed_window_state_dumped) then
    dump_sim_state(s)
  end
  local census = {}
  for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
    local st = m.status
    for k, v in pairs(defines.entity_status) do
      if v == st then census[k] = (census[k] or 0) + 1 end
    end
  end
  -- Research-productivity parity (RFC-064 Phase 2 item 7). Since
  -- 2026-08-23 the init path PINS this axis (the bonus is derived, but
  -- the productivity TECHS are settable -- see the un-research loop next
  -- to the #370/#385 assignments); this block stays as the independent
  -- measurement-time verifier of that pin, and of any declared-nonzero
  -- value the pin deliberately leaves researched. The engine
  -- plans and the meter measures at the DECLARED value; if the sim's world
  -- disagrees, every rate comparison in this run is against a plan built for a
  -- different world, and that is what item 7 turned out to be: the sim carried
  -- +10% on processing-unit that nothing on the engine side modelled, and the
  -- resulting -13% was chased as a belt defect across three sessions.
  --
  -- Undeclared recipes are checked against 0: a manifest that says nothing is
  -- asserting no research productivity, which is what the engine assumed.
  --
  -- Scoped to recipes this factory actually CRAFTS. Iterating every recipe in
  -- the force flags things like steel-plate and low-density-structure that the
  -- layout never builds — true but irrelevant, and a kit error that cries wolf
  -- gets ignored, which is the failure mode this whole check exists to avoid.
  local crafted = {}
  for _, m in pairs(s.find_entities_filtered{
    type = {"assembling-machine", "furnace", "chemical-plant", "oil-refinery"}
  }) do
    local ok, r = pcall(function() return m.get_recipe() end)
    if ok and r ~= nil then crafted[r.name] = true end
  end
  for name, _ in pairs(crafted) do
    local recipe = game.forces.player.recipes[name]
    local declared = DECLARED_PRODUCTIVITY[name] or 0
    local ok, realized = pcall(function() return recipe.productivity_bonus end)
    if ok and realized ~= nil and math.abs(realized - declared) > 1e-6 then
      table.insert(storage.kit_errors,
        "research-productivity parity: '" .. name .. "' realized "
        .. realized .. " but the manifest declares " .. declared
        .. " -- rates for this run are not comparable against a plan built at "
        .. "the declared value (RFC-064 item 7)")
    end
  end

  -- RFC-064 item 7 productivity-parity probe (2026-08-06).
  -- The meter models NO productivity at all (crates/meter/src/machine.rs
  -- deliberately takes nothing from module_policy and not
  -- effective_crafting_speed), while this scenario calls
  -- research_all_technologies() above and its tech-state parity block corrects
  -- inserter capacity (#370), belt stacking (#385) and, since 2026-08-23,
  -- undeclared recipe productivity (the item-7 pin). If the sim carries
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
  -- The list spans the WHOLE tier5_processing_unit_from_ore chain, not just
  -- its assembler legs: plastic-bar, sulfur and the oil steps are crafted in
  -- chemical plants / refineries, and a productivity bonus on any of them
  -- would move the target rate just as surely (PR #580 review). Probing a
  -- recipe the run never crafts is harmless -- the printer only reports
  -- bonuses for recipes the layout actually contains.
  for _, rn in ipairs({"processing-unit", "electronic-circuit", "advanced-circuit",
                       "iron-plate", "copper-plate", "copper-cable",
                       "plastic-bar", "sulfur", "sulfuric-acid",
                       "basic-oil-processing", "advanced-oil-processing"}) do
    prod_force[rn] = probe(function()
      local r = game.forces.player.recipes[rn]
      if r == nil then return nil end
      return r.productivity_bonus
    end)
  end
  local prod_entity, prod_modules = {}, {}
  for _, m in pairs(s.find_entities_filtered{
    type = {"assembling-machine", "furnace", "chemical-plant", "oil-refinery"}
  }) do
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
            -- Productivity-family ONLY. Counting speed/efficiency/quality
            -- modules here made a speed-moduled layout print a BOOSTED banner
            -- with an empty boost list (PR #580 review, 3/3) -- a module is
            -- only a productivity parity gap if it grants productivity.
            if nm ~= nil and string.find(tostring(nm), "productivity", 1, true) then
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
  -- handler's own finalize guards keep it multiplayer-safe: its
  -- MEASUREMENT half is a no-op after this point unconditionally, and
  -- its kit-upkeep half is a no-op too EXCEPT under KEEP_ALIVE, where
  -- `serve` deliberately keeps feeding the factory so an inspected
  -- world stays alive.
end

-- Boundary-kit upkeep: power, feed top-up, drain empty. Everything that
-- keeps the factory alive is in here, so whether this runs after finalize
-- decides whether an inspected world stays a factory or becomes a corpse.
--
-- Under `serve` (KEEP_ALIVE) it keeps running: `finalize` fires on
-- CONVERGENCE as well as at END_TICK, and serve's whole point is that a
-- human can look at a live factory long after it has stabilized. Under a
-- measurement run it must stop, because the report's numbers were sampled
-- at the checkpoints and the kit must not keep mutating the world past the
-- moment it was measured.
if TRACE_FORENSICS then
script.on_nth_tick(1, function(ev)
  if storage.finalized == true then return end
  local s = game.get_surface("lab")
  if s then
    local probe = storage.curve_sideload_probe
    if probe and probe.next_sample <= #probe.sample_ticks
       and ev.tick >= probe.sample_ticks[probe.next_sample] then
      local function line_snapshot(b)
        local lines = {}
        for li = 1, b.get_max_transport_line_index() do
          local detailed = {}
          for _, entry in pairs(b.get_transport_line(li).get_detailed_contents()) do
            local name, count = nil, nil
            local stack_ok, stack = pcall(function() return entry.stack end)
            if stack_ok and stack ~= nil then
              local name_ok, stack_name = pcall(function() return stack.name end)
              local count_ok, stack_count = pcall(function() return stack.count end)
              if name_ok then name = stack_name end
              if count_ok then count = stack_count end
            end
            table.insert(detailed, {name = name, count = count,
                                    position = entry.position})
          end
          table.insert(lines, detailed)
        end
        return lines
      end
      table.insert(storage.drop_physics_probe, {
        label = "curve_sideload_tick_" .. probe.sample_ticks[probe.next_sample],
        tick = ev.tick,
        curve_source = line_snapshot(probe.curve_source),
        curve = line_snapshot(probe.curve),
        target = line_snapshot(probe.target),
        back = line_snapshot(probe.back)
      })
      probe.next_sample = probe.next_sample + 1
      if probe.next_sample > #probe.sample_ticks then
        for _, b in pairs(probe.entities) do if b.valid then b.destroy() end end
        storage.curve_sideload_probe = nil
      end
    end
    if DROP_TRACE then sample_drop_events(s) end
    -- Pickup-only runs pay for this channel at tick resolution so a fast
    -- inserter cannot complete a whole hand cycle between samples. Ordinary
    -- runs do not collect pickup telemetry at all; the 60-tick handler below
    -- is reserved for the lower-cost drop probe.
    if PICKUP_TRACE_ONLY then sample_pickup_events(s, ev.tick) end
  end
end)
end

script.on_nth_tick(60, function(ev)
  if storage.finalized and not KEEP_ALIVE then return end
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

  -- END OF KIT UPKEEP. Everything past here is MEASUREMENT — sampling,
  -- checkpoint windows, the convergence test, and `finalize` itself — and
  -- it must stop at finalize even under KEEP_ALIVE.
  --
  -- Not merging this into the guard at the top of the handler: that one
  -- lets `serve` keep the factory fed after finalize, and if it also let
  -- the measurement half run on, the convergence test would keep passing
  -- on a still-running world and call `finalize` again at every WINDOW
  -- CLOSE — rewriting the report, re-appending the dead-rig audit's
  -- kit_errors, and reprinting HARNESS_DONE. Measured on the first cut of
  -- this fix (2026-08-07): 257 finalizes in one 400s serve at speed 32,
  -- i.e. window cadence (~1 per 3100 ticks), NOT the handler's own 60-tick
  -- cadence — the convergence test lives inside the window-close branch,
  -- not at the top of the handler. The Lua-text unit test could not see
  -- any of this; only a live server could.
  if storage.finalized then return end

  local s = game.get_surface("lab")
  if DROP_TRACE or FIXED_WINDOW then sample_drop_probes(s) end
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
    -- These samples run from tick 0, so they are the ONLY view of the warmup
    -- ramp: the checkpoint rows below cannot open until warmup ends, by
    -- design (they exist to test convergence, which is meaningless mid-ramp).
    -- The ramp is where a stage's start offset lives — belt transit plus
    -- buffer fill — and where "was the warmup long enough" is answerable at
    -- all, so mirror them into the live CSV rather than only into the
    -- end-of-run result. `sample` rows carry the CUMULATIVE count; consumers
    -- derive rates from consecutive rows (docs/sim-harness-forensics.md).
    local sample_csv = {}
    for _, item in ipairs(PLANNED_ITEMS) do
      local cur = produced_count(item)
      produced[item] = cur
      if WRITE_TIMESERIES_CSV then
        table.insert(sample_csv,
          table.concat({ev.tick, "sample", "", "", "", "", "", "", item, cur}, ","))
      end
    end
    table.insert(storage.samples, {tick = ev.tick, drained = storage.drained_total,
      produced = produced, fed = storage.fed_total})
    if WRITE_TIMESERIES_CSV and #sample_csv > 0 then
      helpers.write_file(TIMESERIES_CSV_FILE, table.concat(sample_csv, "\n") .. "\n", true)
    end
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
      local by_items = (not FIXED_WINDOW) and d_items >= WINDOW_ITEM_FLOOR
                       and d_ticks >= WINDOW_MIN_TICKS
      local by_fixed_window = FIXED_WINDOW and d_ticks >= WINDOW_TICKS
      if by_items or by_fixed_window or (not FIXED_WINDOW and d_ticks >= WINDOW_TICK_CAP) then
        table.insert(storage.checkpoints, {tick = ev.tick, produced = produced,
          delivered = delivered, window_ticks = d_ticks, window_items = d_items,
          short_sampled = not by_items and not by_fixed_window, items = checkpoint_items()})
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

        -- Fixed-window diagnostics need the rich state even when the
        -- layout's derived END_TICK is much later than the requested
        -- measurement window.  Dump once at the first closed window so a
        -- CPU-bound run can still expose belt positions and drop events
        -- before the harness timeout; the normal final dump remains the
        -- authoritative output for ordinary runs.
        if FIXED_WINDOW and not storage.fixed_window_state_dumped then
          dump_sim_state(s)
          storage.fixed_window_state_dumped = true
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
        if not FIXED_WINDOW and n >= STABILITY_WINDOWS + 1 then
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
            fixed_window: false,
            scenario_name: "t".into(),
            operator_qol: false,
            write_timeseries: false,
            pickup_trace_only: false,
            drop_trace: false,
            keep_alive: false,
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
            "local by_items = (not FIXED_WINDOW) and d_items >= WINDOW_ITEM_FLOOR"
        ));
        assert!(lua.contains("local by_fixed_window = FIXED_WINDOW and d_ticks >= WINDOW_TICKS"));
        assert!(lua.contains("not FIXED_WINDOW and d_ticks >= WINDOW_TICK_CAP"));
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
        assert!(lua.contains("if not FIXED_WINDOW and n >= STABILITY_WINDOWS + 1 then"));
        assert!(lua.contains("(hi - lo) / lo <= STABILITY_TOL"));
    }

    #[test]
    fn fixed_window_disables_convergence_and_uses_the_requested_budget() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "fixed".into(), 32, Some(18_000))
            .with_warmup(108_000)
            .with_fixed_window(216_000);
        assert!(params.fixed_window);
        assert_eq!(params.window_ticks, 216_000);
        assert_eq!(params.end_tick, 324_000);
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local FIXED_WINDOW = true"));
        assert!(lua.contains("local by_fixed_window = FIXED_WINDOW and d_ticks >= WINDOW_TICKS"));
        assert!(lua.contains("if not FIXED_WINDOW and n >= STABILITY_WINDOWS + 1 then"));
        assert!(lua.contains("short_sampled = not by_items and not by_fixed_window"));
        assert!(lua.contains("storage.fixed_window_state_dumped = false"));
        assert!(lua.contains("if FIXED_WINDOW and not storage.fixed_window_state_dumped then"));
        assert!(lua.contains("storage.fixed_window_state_dumped = true"));
        assert!(lua.contains(
            "if not (FIXED_WINDOW and storage.fixed_window_state_dumped) then"
        ));
        assert!(lua.contains("storage.pickup_event_trace = {}"));
        assert!(lua.contains("local function sample_pickup_events(s, sample_tick)"));
        assert!(lua.contains("local function pickup_is_transport(entity)"));
        assert!(lua.contains("return entity.type == \"assembling-machine\" or entity.type == \"furnace\""));
        assert!(lua.contains("pickup_event_inserter_count = storage.pickup_event_inserters"));
        assert!(lua.contains("storage.pickup_event_inserters = nil"));
        assert!(lua.contains("local warmup_refresh = sample_tick >= WARMUP_TICKS"));
        assert!(lua.contains("storage.pickup_event_inserter_seen[candidate.unit_number]"));
        assert!(lua.contains("storage.pickup_event_warmup_refresh = false"));
        assert!(lua.contains("local function trace_values(trace)"));
        assert!(lua.contains("drop_probes = trace_values(storage.drop_probes)"));
        assert!(lua.contains("drop_event_trace = trace_values(storage.drop_event_trace)"));
        assert!(lua.contains("pickup_event_trace = trace_values(storage.pickup_event_trace)"));
        assert!(lua.contains("storage.pickup_event_previous_item = {}"));
        assert!(lua.contains("item_rec.delivered_items = item_rec.delivered_items + delivered"));
        assert!(lua.contains("item_rec.measurement_start_picked = item_rec.picked_items"));
        assert!(!lua.contains("item_rec.measurement_start_picked = item_rec.picked_items - picked"));
        assert!(lua.contains("measurement_delivered_items = 0"));
        assert!(lua.contains("sample_tick >= WARMUP_TICKS"));
        assert!(lua.contains("if DROP_TRACE then sample_drop_events(s) end"));
        assert!(lua.contains("if PICKUP_TRACE_ONLY then sample_pickup_events(s, ev.tick) end"));
        assert!(
            !lua.contains("if not PICKUP_TRACE_ONLY then sample_pickup_events(s, ev.tick) end"),
            "ordinary runs must not pay for pickup telemetry"
        );
        assert!(lua.contains("local PICKUP_TRACE_ONLY = false"));
        assert!(lua.contains("local DROP_TRACE = false"));
        assert!(lua.contains(
            "local TRACE_FORENSICS = PICKUP_TRACE_ONLY or DROP_TRACE or FIXED_WINDOW"
        ));
        assert!(lua.contains("if DROP_TRACE or FIXED_WINDOW then"));
        assert!(lua.contains("if TRACE_FORENSICS then\nscript.on_nth_tick(1, function(ev)"));
        assert!(lua.contains("if DROP_TRACE or FIXED_WINDOW then sample_drop_probes(s) end"));
        assert!(lua.contains("local_position = line_local_position(spec_position)"));
        assert!(lua.contains("return line.can_insert_at(local_position)"));
        assert!(!lua.contains("return line.can_insert_at(position_sample)"));
        let drop_trace_lua = build_control_lua(&m, "0eNBPFAKE", &params.with_drop_trace());
        assert!(drop_trace_lua.contains("local DROP_TRACE = true"));
    }

    #[test]
    fn fixed_window_caps_an_ordinary_derived_ceiling_to_one_window() {
        let m = fixture();
        let params =
            RunParams::defaults_for(&m, "fixed-cap".into(), 32, None).with_fixed_window(600);
        assert!(params.fixed_window);
        assert_eq!(params.window_ticks, 600);
        assert_eq!(params.end_tick, params.warmup_ticks + params.window_ticks);

        let reordered = RunParams::defaults_for(&m, "fixed-cap".into(), 32, None)
            .with_fixed_window(600)
            .with_warmup(108_001);
        assert_eq!(
            reordered.end_tick,
            reordered.warmup_ticks + reordered.window_ticks
        );
    }

    #[test]
    fn belt_position_trace_reads_the_nested_detailed_stack() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "belt-trace".into(), 16, Some(18_000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local stack_ok, stack = pcall(function() return entry.stack end)"));
        assert!(lua.contains("if name_ok then name = stack_name end"));
        assert!(lua.contains("if count_ok then count = stack_count end"));
        assert!(!lua.contains("name = entry.name"));
        assert!(!lua.contains("count = entry.count"));
    }

    #[test]
    fn ordinary_runs_do_not_register_forensic_tick_work() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "ordinary".into(), 16, Some(18_000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(lua.contains("local TRACE_FORENSICS = PICKUP_TRACE_ONLY or DROP_TRACE or FIXED_WINDOW"));
        assert!(lua.contains("local PICKUP_TRACE_ONLY = false"));
        assert!(lua.contains("local DROP_TRACE = false"));
        assert!(lua.contains("if TRACE_FORENSICS then\nscript.on_nth_tick(1, function(ev)"));
        assert!(lua.contains("if DROP_TRACE or FIXED_WINDOW then sample_drop_probes(s) end"));
    }

    /// An inspected world must keep being fed after it finalizes.
    ///
    /// The bug this pins (2026-08-07, found in-client): the kit's
    /// feed/drain/power upkeep is one `on_nth_tick(60)` handler guarded by
    /// `storage.finalized`, and `finalize` fires on CONVERGENCE as well as
    /// at `END_TICK`. `serve` pushed `end_tick` out to ~a week to stop the
    /// world dying mid-inspection, which guarded only the ceiling path —
    /// so a served world still starved itself minutes in, and an operator
    /// joining later saw empty feed chests and an idle factory.
    ///
    /// Asserted in BOTH directions deliberately: a keep-alive that also
    /// leaked into measurement runs would let the kit keep mutating the
    /// world after the checkpoints were sampled, which is a subtler and
    /// worse bug than the one being fixed.
    #[test]
    fn serve_keeps_the_kit_alive_after_finalize_and_run_does_not() {
        let m = fixture();

        let served = RunParams::defaults_for(&m, "t".into(), 1, Some(36_000_000))
            .with_operator_qol()
            .with_keep_alive();
        assert!(served.keep_alive);
        let lua = build_control_lua(&m, "bp", &served);
        assert!(lua.contains("local KEEP_ALIVE = true"));
        assert!(
            lua.contains("if storage.finalized and not KEEP_ALIVE then return end"),
            "upkeep handler must consult KEEP_ALIVE, not `finalized` alone"
        );
        // The measurement half keeps an UNCONDITIONAL finalize guard: under
        // KEEP_ALIVE the world runs on, so without it the convergence test
        // re-fires every 60 ticks and calls `finalize` forever (observed
        // live on this fix's first cut). Exactly one of each guard.
        assert_eq!(
            lua.matches("if storage.finalized and not KEEP_ALIVE then return end")
                .count(),
            1,
            "kit-upkeep guard must appear exactly once"
        );
        assert_eq!(
            lua.matches("if storage.finalized then return end").count(),
            1,
            "measurement half must keep exactly one unconditional finalize guard"
        );
        // ...and the upkeep guard must come FIRST: the other order would
        // return before the kit is fed, restoring the original bug.
        let upkeep_guard = lua
            .find("if storage.finalized and not KEEP_ALIVE then return end")
            .expect("upkeep guard present");
        let measure_guard = lua
            .find("if storage.finalized then return end")
            .expect("measurement guard present");
        assert!(
            upkeep_guard < measure_guard,
            "kit upkeep must run before the measurement half returns"
        );

        // Pin the STRUCTURE, not just the two strings: string presence
        // cannot tell this handler from a text-equivalent one that guards
        // the wrong statements (review finding on this PR). Each half's
        // real work must sit on the correct side of the two guards —
        // power/feed/drain between them, the convergence test after.
        // Search WITHIN the handler, not the whole script: `finalize` and
        // the kit audit iterate `storage.feeds` too, so a bare
        // `lua.find(..)` matches one of those earlier copies and the
        // assertion silently checks the wrong statement. (Caught by this
        // very test on first write — the same not-unique-string trap the
        // rest of this fix kept hitting.)
        let handler = &lua[upkeep_guard..];
        let measure_rel = measure_guard - upkeep_guard;
        for kit_stmt in [
            "e.energy = 1e13",                              // power upkeep
            "for item, banks in pairs(storage.feeds) do",   // feed top-up
            "for item, chests in pairs(storage.drains) do", // drain empty
        ] {
            let at = handler
                .find(kit_stmt)
                .unwrap_or_else(|| panic!("{kit_stmt} missing from the upkeep handler"));
            assert!(
                at < measure_rel,
                "{kit_stmt} must run under KEEP_ALIVE (before the measurement \
                 guard), else a served world starves after convergence"
            );
        }
        let convergence = lua.find("finalize(s, true)").expect("convergence finalize");
        assert!(
            convergence > measure_guard,
            "the convergence test must sit AFTER the unconditional guard, \
             or KEEP_ALIVE lets it re-finalize at every window close"
        );

        // The SEPARATION invariant, not just membership: no kit upkeep may
        // appear below the measurement guard. Listing three statements
        // above only pins the upkeep that exists today — a fourth
        // mechanism added below the guard would re-introduce the
        // starvation bug with this test still green (review finding, 3/3).
        // Asserting the negative catches that case without needing to know
        // what the mechanism is.
        //
        // NEW KIT UPKEEP GOES BETWEEN THE TWO GUARDS. If you are here
        // because this assertion failed, that is why.
        let below_measure_guard = &lua[measure_guard..];
        for kit_marker in ["storage.eeis", "storage.feeds", "storage.drains"] {
            assert!(
                !below_measure_guard.contains(kit_marker),
                "{kit_marker} appears below the measurement guard: kit upkeep \
                 placed there is skipped once a served world converges, which \
                 is exactly the bug this test exists to prevent"
            );
        }

        // A measurement run must still stop its kit at finalize.
        let measured = RunParams::defaults_for(&m, "t".into(), 32, None);
        assert!(
            !measured.keep_alive,
            "measurement runs must never keep-alive"
        );
        assert!(build_control_lua(&m, "bp", &measured).contains("local KEEP_ALIVE = false"));

        // The ceiling is not, and never was, sufficient on its own — the
        // convergence caller is what actually kills a served world.
        assert!(
            lua.contains("finalize(s, true)"),
            "convergence finalize still present: keep-alive is the fix, not removing it"
        );
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

    /// RFC-072 P2 unit 2: two same-direction feed bands at different
    /// flow coordinates (a grid's outer north edge and its interior
    /// gap edge) get INDEPENDENT ladders — the old global ladder gave
    /// the second band depths the inter-strip clearance cannot hold —
    /// and their combined footprints stay disjoint.
    #[test]
    fn interior_band_feeds_get_their_own_ladder() {
        let mut records = Vec::new();
        for x in [0, 4, 8] {
            records.push(south_feed(&format!("outer-{x}"), x));
        }
        for x in [0, 4, 8] {
            let mut r = south_feed(&format!("inner-{x}"), x);
            r.y = 57; // one strip height + clearance below the outer edge
            records.push(r);
        }
        let slots = feed_slots(&records);
        assert_eq!(
            slots,
            vec![0, 1, 2, 0, 1, 2],
            "each band must restart its ladder"
        );
        assert_feed_footprints_disjoint(&records);
    }

    /// RFC-072 P2 unit 2: same-band feeds beyond RIG_LATERAL_HALO
    /// restart the ladder — a 9-copy strip's per-copy feed clusters
    /// must not accumulate depth across the whole strip (the old
    /// behavior: 18 feeds → depth 106).
    #[test]
    fn lateral_clusters_reset_the_ladder() {
        let mut records = Vec::new();
        for copy in 0..9 {
            for input in 0..2 {
                records.push(south_feed(&format!("c{copy}-i{input}"), copy * 30 + input * 4));
            }
        }
        let slots = feed_slots(&records);
        assert_eq!(
            slots.iter().max(),
            Some(&1),
            "per-copy clusters of 2 must cap the ladder at slot 1 \
             (max depth 10), got slots {slots:?}"
        );
        assert_feed_footprints_disjoint(&records);
    }

    /// Two same-direction bands closer than the deeper band's rig
    /// reach, laterally overlapping, cannot be separated by any
    /// stagger — the codegen must refuse loudly (the #357 silent
    /// entity-stacking class), not build overlapping rigs.
    #[test]
    #[should_panic(expected = "the stagger cannot separate them")]
    fn near_parallel_feed_bands_refuse() {
        let mut inner = south_feed("inner", 0);
        inner.y = 6; // within the outer band's depth-4 rig reach
        let records = vec![south_feed("outer", 0), inner];
        feed_slots(&records);
    }

    fn south_drain(item: &str, x: i32, y: i32) -> BoundaryRecord {
        BoundaryRecord {
            item: item.into(),
            x,
            y,
            direction: 8, // flowing south, rig extends south
            is_fluid: false,
            entity: "express-transport-belt".into(),
        }
    }

    /// RFC-072 P2 unit 2: drain extensions stagger per lateral cluster,
    /// not per manifest index — far-apart exits all get the base 11
    /// (the grid's inter-strip clearance is sized to that), close exits
    /// keep the old scheme's 2-step spacing exactly as measured. Rigs
    /// beyond RIG_LATERAL_HALO must be fully tile-disjoint; WITHIN a
    /// cluster full disjointness is deliberately NOT the contract (see
    /// the known-imperfect note in `drain_ext_lens` — ec23's pitch-1
    /// pair measures PASS under per-tile placement loss).
    #[test]
    fn drain_extensions_cluster_and_far_rigs_stay_disjoint() {
        let records = vec![
            south_drain("a", 0, 20),
            south_drain("b", 4, 20),   // same cluster as a — 2-step stagger
            south_drain("c", 100, 20), // own cluster — back to base
            south_drain("d", 200, 20), // own cluster — back to base
        ];
        let exts = drain_ext_lens(&records);
        assert_eq!(exts, vec![11, 13, 11, 11]);
        // Cross-cluster disjointness: every tile of rigs a+b vs c vs d.
        let mut clusters: Vec<std::collections::HashSet<(i32, i32)>> = Vec::new();
        let groups: [&[usize]; 3] = [&[0, 1], &[2], &[3]];
        for g in groups {
            let mut tiles = std::collections::HashSet::new();
            for &i in g {
                let rec = &records[i];
                tiles.extend(drain_footprint(
                    (rec.x, rec.y),
                    rec.direction().vector(),
                    exts[i],
                ));
            }
            clusters.push(tiles);
        }
        for a in 0..clusters.len() {
            for b in (a + 1)..clusters.len() {
                assert!(
                    clusters[a].is_disjoint(&clusters[b]),
                    "clusters {a} and {b} share tiles"
                );
            }
        }
    }

    /// Drain bands closer along the flow axis than the nearer band's
    /// extension reach must refuse loudly.
    #[test]
    #[should_panic(expected = "the stagger cannot separate them")]
    fn near_parallel_drain_bands_refuse() {
        let records = vec![south_drain("near", 0, 0), south_drain("far", 0, 8)];
        drain_ext_lens(&records);
    }

    /// A grid-shaped manifest: strip 0's exits at its south edge (y=16)
    /// draining SOUTH into the clearance, strip 1's feeds at its north
    /// edge (y=49) whose rigs extend NORTH into the same clearance. Per
    /// slot the feed columns sit west of the cell and the drain column
    /// east of it. At the engine's 32-tile clearance the cross-type
    /// guard passes; below it, it must refuse by name.
    fn grid_manifest(clearance: i32) -> (Vec<BoundaryRecord>, Vec<BoundaryRecord>) {
        let strip_h = 17;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for copy in 0..3 {
            let slot_x = copy * 60;
            for k in 0..2 {
                inputs.push(south_feed(&format!("ore-{copy}-{k}"), slot_x + k * 4));
                let mut inner = south_feed(&format!("ore-inner-{copy}-{k}"), slot_x + k * 4);
                inner.y = strip_h + clearance;
                inputs.push(inner);
            }
            outputs.push(south_drain(&format!("ec-{copy}"), slot_x + 30, strip_h - 1));
            outputs.push(south_drain(
                &format!("ec-inner-{copy}"),
                slot_x + 30,
                2 * strip_h + clearance - 1,
            ));
        }
        (inputs, outputs)
    }

    #[test]
    fn grid_clearance_keeps_feed_and_drain_rigs_disjoint() {
        let (inputs, outputs) = grid_manifest(32);
        let slots = feed_slots(&inputs);
        let exts = drain_ext_lens(&outputs);
        assert_feed_drain_rigs_disjoint(&inputs, &slots, &outputs, &exts);
        assert_feed_footprints_disjoint(&inputs);
    }

    /// The two strips' occupied bands (every tile of a 17-tall strip
    /// across the manifest's width), as the layout decode would report.
    fn grid_occupied(clearance: i32) -> std::collections::HashSet<(i32, i32)> {
        let mut occ = std::collections::HashSet::new();
        for x in -4..200 {
            for y in 0..17 {
                occ.insert((x, y));
                occ.insert((x, y + 17 + clearance));
            }
        }
        occ
    }

    #[test]
    fn grid_clearance_keeps_rigs_off_both_strips() {
        let (inputs, outputs) = grid_manifest(32);
        let slots = feed_slots(&inputs);
        let exts = drain_ext_lens(&outputs);
        assert_rigs_clear_of_occupied(&grid_occupied(32), &inputs, &slots, &outputs, &exts);
    }

    /// At clearance 8 the inner feed rigs (depth up to 10, plus the ±2
    /// bank) reach north INTO strip 0's machine band — the silent kit
    /// failure the layout guard exists to refuse by name.
    #[test]
    #[should_panic(expected = "would place kit on layout tile")]
    fn undersized_grid_clearance_refuses_rigs_inside_a_strip() {
        let (inputs, outputs) = grid_manifest(8);
        let slots = feed_slots(&inputs);
        let exts = drain_ext_lens(&outputs);
        assert_rigs_clear_of_occupied(&grid_occupied(8), &inputs, &slots, &outputs, &exts);
    }

    /// The DEFAULT-PATH corpus check (#731 round 2): every guard, against
    /// real engine exports committed under `tests/fixtures/<label>/`
    /// (`bp.txt` + `manifest-real.json`) — a single strip (chain-ec15-d2,
    /// the calibration-family fixture) and a real two-strip GRID
    /// (grid-ec180p, ec@180 from plates, 544×64, 0 errors / 0 warnings)
    /// whose interior-edge rigs are exactly what the guards exist for.
    /// Runs in CI on every push; the bank-wide sweep below stays the
    /// operator's deep check.
    #[test]
    fn codegen_guards_accept_committed_real_fixtures() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut checked = Vec::new();
        for entry in std::fs::read_dir(&root).expect("tests/fixtures").flatten() {
            let mp = entry.path().join("manifest-real.json");
            let Ok(text) = std::fs::read_to_string(&mp) else { continue };
            let m: crate::manifest::Manifest =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{mp:?}: {e}"));
            let bp = std::fs::read_to_string(entry.path().join("bp.txt"))
                .unwrap_or_else(|e| panic!("{:?}/bp.txt: {e}", entry.path()));
            let slots = feed_slots(&m.boundary_inputs);
            let exts = drain_ext_lens(&m.boundary_outputs);
            assert_feed_drain_rigs_disjoint(&m.boundary_inputs, &slots, &m.boundary_outputs, &exts);
            assert_rigs_clear_of_layout(&bp, &m.boundary_inputs, &slots, &m.boundary_outputs, &exts);
            checked.push(entry.file_name().to_string_lossy().to_string());
        }
        checked.sort();
        assert_eq!(
            checked,
            vec!["chain-ec15-d2".to_string(), "grid-ec180p".to_string()],
            "both committed real fixtures must be present and pass every guard"
        );
    }

    /// The grid fixture's interior edge is real: its second strip's feed
    /// heads sit at a nonzero y, so the banded ladder (not the old global
    /// one) is what makes its rigs fit — pin that shape so a regression
    /// to a single global ladder fails here, not in a sim.
    #[test]
    fn committed_grid_fixture_has_interior_feed_band() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grid-ec180p");
        let m: crate::manifest::Manifest = serde_json::from_str(
            &std::fs::read_to_string(root.join("manifest-real.json")).expect("manifest"),
        )
        .expect("parse");
        let ys: std::collections::BTreeSet<i32> = m.boundary_inputs.iter().map(|r| r.y).collect();
        assert_eq!(ys.len(), 2, "two feed bands (outer + interior), got {ys:?}");
        let slots = feed_slots(&m.boundary_inputs);
        assert!(
            slots.iter().max().copied().unwrap_or(0) <= 2,
            "per-copy clusters must keep the ladder shallow, got {slots:?}"
        );
    }

    /// Codegen over every manifest in a real bank — the rig guards
    /// (near-parallel bands, cross-type overlap, rig-vs-layout) must
    /// accept every fixture the sim has ever legitimately measured.
    /// Ignored: needs SPAGHETTIO_CALIBRATION_BANK pointing at a measured
    /// bank with per-fixture `manifest-real.json` + `bp.txt`.
    ///
    /// KNOWN REFUSALS (found by this sweep, 2026-08-26): fluid feed
    /// records whose head entity is a bare `pipe` carry direction 0 —
    /// a pipe has no direction, so the engine records North — and the
    /// harness's "outward = −into" then points INTO the layout: the
    /// fluid rig is built on top of the factory. Exactly these three
    /// fixtures are the calibration bank's three "non-converged 0.000"
    /// rows; those were never layout measurements. The list can only
    /// SHRINK: the fix is engine-side (record the real into-layout flow
    /// direction for pipe heads), followed by a manifest re-bless and
    /// re-measure of the three rows — a followup, not this unit.
    #[test]
    #[ignore]
    fn codegen_accepts_every_bank_manifest() {
        const KNOWN_REFUSED: &[&str] =
            &["tier3_heavy_oil_cracking", "tier3_sulfuric_acid", "tier3_plastic_bar"];
        let bank = std::env::var("SPAGHETTIO_CALIBRATION_BANK")
            .expect("set SPAGHETTIO_CALIBRATION_BANK to a measured bank dir");
        let mut checked = 0;
        let mut refused: Vec<(String, String)> = Vec::new();
        for entry in std::fs::read_dir(&bank).expect("read bank").flatten() {
            let mp = entry.path().join("manifest-real.json");
            let Ok(text) = std::fs::read_to_string(&mp) else { continue };
            let label = entry.file_name().to_string_lossy().to_string();
            let m: crate::manifest::Manifest =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{mp:?}: {e}"));
            let bp = std::fs::read_to_string(entry.path().join("bp.txt"))
                .unwrap_or_else(|e| panic!("{:?}/bp.txt: {e}", entry.path()));
            let outcome = std::panic::catch_unwind(|| {
                let slots = feed_slots(&m.boundary_inputs);
                let exts = drain_ext_lens(&m.boundary_outputs);
                assert_feed_drain_rigs_disjoint(&m.boundary_inputs, &slots, &m.boundary_outputs, &exts);
                assert_rigs_clear_of_layout(&bp, &m.boundary_inputs, &slots, &m.boundary_outputs, &exts);
            });
            checked += 1;
            if let Err(e) = outcome {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                refused.push((label, msg));
            }
        }
        assert!(checked > 0, "no manifests found under {bank}");
        let unexpected: Vec<&(String, String)> = refused
            .iter()
            .filter(|(l, _)| !KNOWN_REFUSED.contains(&l.as_str()))
            .collect();
        eprintln!(
            "codegen guards: {checked} manifests, {} refused ({} known)",
            refused.len(),
            refused.len() - unexpected.len()
        );
        for (l, m) in &refused {
            eprintln!("  {l}: {m}");
        }
        assert!(
            unexpected.is_empty(),
            "guards refused fixtures outside the known set: {unexpected:?}"
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
        assert!(lua.contains(
            "short_sampled = not by_items and not by_fixed_window, items = checkpoint_items()"
        ));
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

    /// The parity block must actually be emitted into the scenario.
    ///
    /// Added because a local adversarial review found this PR shipped ZERO
    /// template tests, against this module's own ~25 `build_control_lua`
    /// string-pin precedent — deleting the whole parity block still left
    /// 73/73 green. These pin the three pieces that make the check work at
    /// all: the declared table, the crafted-recipe scoping, and the kit-error
    /// on disagreement.
    #[test]
    fn research_productivity_parity_block_is_emitted() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-prod-parity".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);

        assert!(
            lua.contains("local DECLARED_PRODUCTIVITY = {"),
            "the declared axis must reach the scenario as a table"
        );
        assert!(
            lua.contains("research-productivity parity:"),
            "a disagreement must raise a kit error, which is what forces NO DATA"
        );
        assert!(
            lua.contains("storage.kit_errors"),
            "the parity finding must go to kit_errors, not a bare print"
        );
        // Scoped to what the layout crafts: iterating every force recipe
        // flagged steel-plate and low-density-structure on a PU fixture, and a
        // kit error that cries wolf gets ignored.
        assert!(
            lua.contains("crafted[r.name] = true"),
            "the check must be scoped to recipes the layout actually crafts"
        );
    }

    /// The item-7 PIN (2026-08-23): the init path un-researches the
    /// per-recipe productivity techs for undeclared recipes, so the world
    /// realizes the declared value instead of research_all's +10%/level.
    /// This pins the three pieces that make the pin trustworthy: the effect
    /// filter, the read-back self-audit, and the mixed-tech refusal.
    #[test]
    fn research_productivity_pin_is_emitted() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-prod-pin".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);

        assert!(
            lua.contains("change-recipe-productivity"),
            "the pin must select techs by their productivity effect, not by name"
        );
        assert!(
            lua.contains("recipe-productivity parity pin did not take:"),
            "the pin must read the realized bonus back — a write nobody verifies \
             is the #376 finding again"
        );
        assert!(
            lua.contains("mixes declared and undeclared productivity recipes"),
            "a tech spanning declared and undeclared recipes must refuse loudly, \
             not half-pin"
        );
        // The pin must run BEFORE the blueprint is stamped so machines are
        // never crafted-into under the wrong bonus: it lives in the init
        // path next to the #370/#385 assignments, which precede surface
        // creation.
        let pin_at = lua
            .find("change-recipe-productivity")
            .expect("pin present");
        let surface_at = lua
            .find("game.create_surface(\"lab\")")
            .expect("surface creation present");
        assert!(
            pin_at < surface_at,
            "the pin must precede surface creation / blueprint stamping"
        );
    }

    /// A declared value must survive into the emitted table, not just an empty
    /// one — the empty case would pass every assertion above.
    #[test]
    fn declared_productivity_values_reach_the_lua_table() {
        let mut m = fixture();
        m.research_productivity
            .insert("processing-unit".to_string(), 0.10);
        let params = RunParams::defaults_for(&m, "test-prod-declared".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);
        assert!(
            lua.contains(r#"["processing-unit"]=0.1"#),
            "declared entries must be emitted verbatim; got the table line: {:?}",
            lua.lines().find(|l| l.contains("DECLARED_PRODUCTIVITY"))
        );
    }
}
