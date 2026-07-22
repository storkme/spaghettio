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
//! flags this for the report, same treatment as fluid boundaries).
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
/// expected items >= ~300").
const WINDOW_ITEM_FLOOR: f64 = 300.0;

/// Two consecutive stability windows must agree within this fraction to
/// call the run converged (RFC: "loop-until-stable ... within tolerance" —
/// not given a specific number; resolved to match KC2's own 2% scale).
const STABILITY_TOLERANCE: f64 = 0.02;

/// `base + 2*(W+H)*32`, rounded up to a multiple of 60 (the tick handler's
/// own cadence — a non-multiple-of-60 ceiling could never be hit exactly
/// by an `on_nth_tick(60, ...)` check).
pub fn default_warmup_ticks(width: i32, height: i32) -> u32 {
    let raw = BASE_WARMUP_TICKS + 2 * (width.max(0) as u32 + height.max(0) as u32) * DIM_WARMUP_FACTOR;
    round_up_60(raw)
}

/// Window length so `rate * window_seconds >= WINDOW_ITEM_FLOOR`, floored
/// at `MIN_WINDOW_TICKS`, rounded to a multiple of 60.
pub fn default_window_ticks(target_rate: f64) -> u32 {
    if target_rate <= 0.0 {
        return round_up_60(MIN_WINDOW_TICKS);
    }
    let seconds = WINDOW_ITEM_FLOOR / target_rate;
    let ticks = (seconds * 60.0).ceil() as u32;
    round_up_60(ticks.max(MIN_WINDOW_TICKS))
}

fn round_up_60(t: u32) -> u32 {
    t.div_ceil(60) * 60
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
}

impl RunParams {
    /// Build defaults from the manifest's own dims + target rate, leaving
    /// `end_tick`/`speed`/`scenario_name` for the caller (CLI flags or
    /// generated identity) to fill in.
    pub fn defaults_for(manifest: &Manifest, scenario_name: String, speed: u32, end_tick: Option<u32>) -> RunParams {
        let warmup = default_warmup_ticks(manifest.dims[0], manifest.dims[1]);
        let target_rate = manifest.targets.first().map(|t| t.rate).unwrap_or(1.0);
        let window = default_window_ticks(target_rate);
        // Ceiling must clear at least one warmup + one window, or the run
        // can never take a first stability sample.
        let default_ceiling = round_up_60(warmup + window * 4);
        RunParams {
            end_tick: end_tick.unwrap_or(default_ceiling).max(warmup + window),
            speed,
            warmup_ticks: warmup,
            window_ticks: window,
            scenario_name,
        }
    }

    /// Override the dim-scaled warmup (`--warmup`). The 2% stability
    /// windows cannot distinguish a slow buffer-fill drift from real
    /// convergence — deep-chain fixtures "converge" while trunk and tap
    /// buffers are still filling — so steady-state probes need
    /// measurement to start long after that transient. Rounded up to the
    /// tick handler's 60-tick cadence; the ceiling is re-floored so the
    /// run can still take one stability sample.
    pub fn with_warmup(mut self, warmup: u32) -> RunParams {
        self.warmup_ticks = round_up_60(warmup);
        self.end_tick = self.end_tick.max(self.warmup_ticks + self.window_ticks);
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
  for _, c in ipairs(chests) do table.insert(storage.feeds[item], c) end
end

-- Fluid FEED: infinity-pipe adjacent to the boundary tile, auto-maintained
-- by the game (no periodic script refill needed). UNCALIBRATED (RFC-050:
-- "mark clearly ... fluid paths are UNCALIBRATED, no fixture has exercised
-- them yet") — geometry and API verified live only for a standalone
-- infinity-pipe (kit-probe), never against a real fluid-consuming factory.
local function add_fluid_feed(s, force, head_x, head_y, ox, oy, item)
  local px, py = head_x + ox, head_y + oy
  local ok, err = pcall(function()
    local ip = s.create_entity{name = "infinity-pipe", position = {px, py}, force = force}
    ip.set_infinity_pipe_filter{name = item, percentage = 1, mode = "exactly"}
  end)
  if not ok then storage.fluid_errors[item .. "@feed"] = tostring(err) end
end

-- DRAIN rig: extension belt (always express/blue -- "drain tier >= belt
-- tier or backpressure falsifies the run") + flanking stack-inserter bank
-- picking FROM the belt (pickup = belt side, per the artifact-boundary
-- inserter-direction lesson).
local function add_drain(s, force, exit_x, exit_y, fx, fy, lx, ly, ext_len, item)
  for t = 1, ext_len do
    s.create_entity{name = "express-transport-belt", position = {exit_x + fx * t, exit_y + fy * t},
                    direction = dir_from_vec(fx, fy), force = force}
  end
  local chests = {}
  for t = ext_len - 2, ext_len do
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

/// One `add_feed`/`add_fluid_feed` call site, with all the outward/lateral
/// vector arithmetic resolved at Rust-codegen time (only the runtime-only
/// `storage.offx`/`storage.offy` piece stays symbolic in the emitted Lua).
fn feed_call(out: &mut String, idx: usize, rec: &BoundaryRecord) {
    let into = rec.direction().vector();
    let outward = neg(into);
    let lateral = rot90(into);
    // Depth stagger must exceed the bank's ±2 lateral chest offset, or
    // adjacent rigs' chest rows land on the same tile — create_entity in
    // script mode stacks entities silently, and a shared bank tile
    // cross-feeds ores (two overlapping chests at one tile poisoned the
    // logistic fixture's iron system with copper plates; #357 forensics
    // 2026-07-22). 4+4*idx put rig 0's chest row (depth+2) exactly on
    // rig 1's (depth-2); 6 per step keeps every rig's occupied band
    // [depth-2, depth+2] disjoint. The Lua-side overlap audit backstops
    // geometries this spacing can't save (heads 10-12 tiles apart put a
    // rig's outward column through a neighbor's bank).
    let depth = 4 + 6 * (idx as i32);
    let _ = writeln!(
        out,
        "  do\n    local head_x, head_y = {x} - LX0 + storage.offx, {y} - LY0 + storage.offy",
        x = rec.x,
        y = rec.y,
    );
    if rec.is_fluid {
        let _ = writeln!(
            out,
            "    add_fluid_feed(s, force, head_x, head_y, {ox}, {oy}, \"{item}\")",
            ox = outward.0,
            oy = outward.1,
            item = rec.item,
        );
    } else {
        let _ = writeln!(
            out,
            "    add_feed(s, force, head_x, head_y, {ox}, {oy}, {lx}, {ly}, {depth}, \"{item}\", \"{belt}\")",
            ox = outward.0,
            oy = outward.1,
            lx = lateral.0,
            ly = lateral.1,
            depth = depth,
            item = rec.item,
            belt = rec.entity,
        );
    }
    let _ = writeln!(out, "  end -- feed[{idx}] {} at ({},{})", rec.item, rec.x, rec.y);
}

fn drain_call(out: &mut String, idx: usize, rec: &BoundaryRecord) {
    let flow = rec.direction().vector();
    let lateral = rot90(flow);
    let ext_len = 5 + 2 * (idx as i32);
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
    let _ = writeln!(out, "  end -- drain[{idx}] {} at ({},{})", rec.item, rec.x, rec.y);
}

/// Build the full `control.lua` for one measurement run.
pub fn build_control_lua(manifest: &Manifest, bp: &str, params: &RunParams) -> String {
    let mut out = String::new();
    let target_item = manifest
        .targets
        .first()
        .map(|t| t.item.as_str())
        .unwrap_or("");

    let _ = writeln!(out, "-- Generated by spaghettio-sim (RFC-050). DO NOT EDIT.");
    let _ = writeln!(out, "-- label: {}", manifest.label);
    let _ = writeln!(out, "local BP = \"{bp}\"");
    let _ = writeln!(out, "local TARGET = \"{target_item}\"");
    let _ = writeln!(out, "local END_TICK = {}", params.end_tick);
    let _ = writeln!(out, "local WARMUP_TICKS = {}", params.warmup_ticks);
    let _ = writeln!(out, "local WINDOW_TICKS = {}", params.window_ticks);
    let _ = writeln!(out, "local STABILITY_TOL = {STABILITY_TOLERANCE}");
    let _ = writeln!(out, "local LX0, LY0 = {}, {}", manifest.bbox_min[0], manifest.bbox_min[1]);
    let _ = writeln!(out, "local DIMS_X, DIMS_Y = {}, {}", manifest.dims[0], manifest.dims[1]);
    {
        let items: Vec<String> = manifest.planned_rates.keys().map(|k| format!("\"{k}\"")).collect();
        let _ = writeln!(out, "local PLANNED_ITEMS = {{{}}}", items.join(", "));
    }

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
script.on_init(function()
  storage.eeis, storage.feeds, storage.fed_total = {}, {}, {}
  storage.drains, storage.drained_total = {}, {}
  storage.samples, storage.checkpoints = {}, {}
  storage.fluid_errors = {}
  storage.kit_errors = {}
  storage.finalized = false
  storage.converged = false
  game.speed = "#,
    );
    let _ = writeln!(out, "{}", params.speed);
    out.push_str(
        r#"  local force = game.forces.player
  force.research_all_technologies()
  local s = game.create_surface("lab")
  s.generate_with_lab_tiles = true
  s.request_to_generate_chunks({0, 0}, 12)
  s.request_to_generate_chunks({DIMS_X, DIMS_Y}, 12)
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

    for (idx, rec) in manifest.boundary_inputs.iter().enumerate() {
        feed_call(&mut out, idx, rec);
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
  do
    local seen = {}
    for _, c in pairs(s.find_entities_filtered{name = "steel-chest"}) do
      local key = math.floor(c.position.x) .. "," .. math.floor(c.position.y)
      if seen[key] then
        table.insert(storage.kit_errors, "overlapping kit chests at (" .. key .. ")")
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
  local belts, machines, inserters = {}, {}, {}
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
  for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
    -- Input/output inventory contents: belts flush transient
    -- contamination within seconds, machine inventories hold it until
    -- consumed — the durable witness for wrong-item forensics (#357).
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
                            math.floor(m.position.y - storage.offy) + LY0, m.name, stn(m.status), inv})
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
    belts = belts, machines = machines, inserters = inserters,
    ugs = ugs, splitters = splitters, chests = chests}, false)
end

local function finalize(s, converged)
  storage.finalized = true
  storage.converged = converged
  dump_sim_state(s)
  local census = {}
  for _, m in pairs(s.find_entities_filtered{type = {"assembling-machine", "furnace"}}) do
    local st = m.status
    for k, v in pairs(defines.entity_status) do
      if v == st then census[k] = (census[k] or 0) + 1 end
    end
  end
  helpers.write_file("harness-result.json", helpers.table_to_json{
    import_rc = storage.import_rc, ghosts = storage.ghosts, revived = storage.revived,
    factory_eeis = storage.factory_eeis, pole_networks = storage.net_count,
    proxies_fulfilled = storage.proxies_fulfilled,
    samples = storage.samples, checkpoints = storage.checkpoints,
    machine_census = census, converged = storage.converged, final_tick = game.tick,
    fluid_errors = storage.fluid_errors, kit_errors = storage.kit_errors}, false)
  print("HARNESS_DONE")
  script.on_nth_tick(60, nil)
end

script.on_nth_tick(60, function(ev)
  if storage.finalized then return end
  for _, e in ipairs(storage.eeis) do if e.valid then e.energy = 1e13 end end
  for item, chests in pairs(storage.feeds) do
    for _, c in ipairs(chests) do
      if c.valid then
        local n = c.get_item_count(item)
        if n < 400 then
          local got = c.insert{name = item, count = 400 - n}
          storage.fed_total[item] = (storage.fed_total[item] or 0) + got
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

  if ev.tick % 1200 == 0 then
    local produced = {}
    for _, item in ipairs(PLANNED_ITEMS) do produced[item] = stats.get_input_count(item) end
    table.insert(storage.samples, {tick = ev.tick, drained = storage.drained_total,
      produced = produced, fed = storage.fed_total})
  end

  if ev.tick >= WARMUP_TICKS and ev.tick % WINDOW_TICKS == 0 then
    local cp = {tick = ev.tick, produced = stats.get_input_count(TARGET),
      delivered = storage.drained_total[TARGET] or 0}
    table.insert(storage.checkpoints, cp)
    local n = #storage.checkpoints
    -- Need two consecutive WINDOW-length rates to compare (three
    -- checkpoints: a->b is window 1, b->c is window 2) -- "loop-until-
    -- stable", not a single retry.
    if n >= 3 then
      local a, b, c = storage.checkpoints[n - 2], storage.checkpoints[n - 1], storage.checkpoints[n]
      local dt1, dt2 = (b.tick - a.tick) / 60, (c.tick - b.tick) / 60
      local rate1 = dt1 > 0 and (b.produced - a.produced) / dt1 or 0
      local rate2 = dt2 > 0 and (c.produced - b.produced) / dt2 or 0
      if rate1 > 0 and math.abs(rate2 - rate1) / rate1 <= STABILITY_TOL then
        finalize(s, true)
        return
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
        assert_eq!(default_warmup_ticks(53, 34), round_up_60(3600 + 2 * 87 * 32));
    }

    #[test]
    fn warmup_override_rounds_to_cadence_and_lifts_ceiling() {
        let p = RunParams {
            end_tick: 10_000,
            speed: 16,
            warmup_ticks: 3600,
            window_ticks: 1800,
            scenario_name: "t".into(),
        }
        .with_warmup(216_001);
        assert_eq!(p.warmup_ticks, 216_060);
        assert_eq!(p.end_tick, 216_060 + 1800);
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
        assert!(lua.contains("add_feed(s, force, head_x, head_y, 0, -1, 1, 0, 4, \"iron-ore\", \"transport-belt\")"));
        // Second rig at depth 10 (6-per-idx stagger, > the bank's ±2
        // chest offset — see feed_call's collision comment / #357).
        assert!(lua.contains("add_feed(s, force, head_x, head_y, 0, -1, 1, 0, 10, \"iron-ore\", \"transport-belt\")"));
        // head world-position translation, anchored to the manifest bbox_min
        assert!(lua.contains("local head_x, head_y = 1 - LX0 + storage.offx, 0 - LY0 + storage.offy"));
        assert!(lua.contains("local head_x, head_y = 2 - LX0 + storage.offx, 0 - LY0 + storage.offy"));
    }

    #[test]
    fn gear10_drain_reduces_to_calibrated_prototype_numbers() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "test-gear".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNBPFAKE", &params);

        // south-facing exit: flow=(0,1), lateral=(1,0) -- matches
        // gen_harness_scenario.py's drain (fx=south, lx=east), ext_len=5.
        assert!(lua.contains("add_drain(s, force, exit_x, exit_y, 0, 1, 1, 0, 5, \"iron-gear-wheel\")"));
        assert!(lua.contains("local exit_x, exit_y = 13 - LX0 + storage.offx, 33 - LY0 + storage.offy"));
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
        assert_eq!(lua.matches("add_fluid_feed(s, force").count(), 1, "only the definition, no calls");
        assert_eq!(lua.matches("add_fluid_void(s, force").count(), 1, "only the definition, no calls");
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
        assert!(lua.contains("add_fluid_feed(s, force, head_x, head_y, 0, -1, \"water\")"));
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

    #[test]
    fn scenario_name_and_bp_are_embedded() {
        let m = fixture();
        let params = RunParams::defaults_for(&m, "my-scenario".into(), 16, Some(18000));
        let lua = build_control_lua(&m, "0eNSOMEBP", &params);
        assert!(lua.contains("local BP = \"0eNSOMEBP\""));
        assert!(lua.contains("local TARGET = \"iron-gear-wheel\""));
    }
}
